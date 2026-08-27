//! Conformance for the magic tier: the Bravyi–Gosset block source against the
//! naive 2^t branch sum against the CERTIFIED holon-qasm magic tier.
//!
//! Three independent implementations must agree on every amplitude. Two of
//! them (naive, BG) are exact in Z[ω], so they are compared IN THE RING —
//! bit-exact, not within a tolerance; the referee returns f64 and is compared
//! at 1e-10 as specified.
//!
//! The harness carries its own teeth: `planted_mutation_is_caught` shows the
//! comparison fails when the engine is wrong, so a pass means something.

use holon::magic::{
    self, cyc_eq, is_zero, A6_PROVENANCE, A6_RANK, A6_WIDTH, BgSource, Circuit, Gate, Mutation,
    NaiveSource,
};
use holon::BranchSource;

// ---------------------------------------------------------------- seeded rng

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn random_circuit(rng: &mut Rng, n: usize, n_clifford: usize, t: usize) -> Circuit {
    let mut gates = Vec::with_capacity(n_clifford + t);
    let (mut t_left, mut c_left) = (t, n_clifford);
    while t_left + c_left > 0 {
        let take_t = if t_left == 0 {
            false
        } else if c_left == 0 {
            true
        } else {
            rng.below((t_left + c_left) as u64) < t_left as u64
        };
        let q = rng.below(n as u64) as usize;
        if take_t {
            t_left -= 1;
            gates.push(if rng.below(2) == 0 { Gate::T(q) } else { Gate::Tdg(q) });
        } else {
            c_left -= 1;
            gates.push(match rng.below(6) {
                0 => Gate::H(q),
                1 => Gate::S(q),
                2 => Gate::Sdg(q),
                3 => Gate::X(q),
                4 => Gate::Z(q),
                _ => {
                    let mut t2 = rng.below(n as u64) as usize;
                    if t2 == q {
                        t2 = (q + 1) % n;
                    }
                    Gate::Cx(q, t2)
                }
            });
        }
    }
    Circuit { n_qubits: n, gates }
}

fn to_qasm(c: &Circuit) -> holon_qasm::Circuit {
    use holon_qasm::Gate as Q;
    holon_qasm::Circuit {
        n_qubits: c.n_qubits,
        n_clbits: 0,
        gates: c
            .gates
            .iter()
            .map(|g| match *g {
                Gate::X(q) => Q::X(q),
                Gate::Z(q) => Q::Z(q),
                Gate::H(q) => Q::H(q),
                Gate::S(q) => Q::S(q),
                Gate::Sdg(q) => Q::Sdg(q),
                Gate::Cx(a, b) => Q::Cx(a, b),
                Gate::T(q) => Q::T(q),
                Gate::Tdg(q) => Q::Tdg(q),
            })
            .collect(),
        measures: vec![],
    }
}

fn bits(idx: usize, n: usize) -> Vec<bool> {
    (0..n).map(|q| (idx >> q) & 1 == 1).collect()
}

// ---------------------------------------------------------------- conformance

#[test]
fn bg_matches_naive_and_the_certified_reference() {
    let mut rng = Rng(0x5EED_1234_ABCD_0001);
    let mut worst_ref = 0.0f64;
    let mut worst_pair = 0.0f64;
    let mut checked = 0usize;
    let mut circuits = 0usize;

    for i in 0..72usize {
        let n = 2 + i % 5; // 2..=6
        let t = i % 13; // 0..=12
        let depth = 6 + (i % 11);
        let c = random_circuit(&mut rng, n, depth, t);
        let qc = to_qasm(&c);

        let naive = NaiveSource::new(&c);
        let bg = BgSource::new(&c);
        assert_eq!(naive.n_qubits(), n);
        assert_eq!(bg.n_qubits(), n);

        // ≥ 8 random basis states (all of them when the register is tiny).
        let dim = 1usize << n;
        let n_probe = 8.min(dim);
        let mut probes: Vec<usize> = Vec::new();
        while probes.len() < n_probe {
            let y = rng.below(dim as u64) as usize;
            if !probes.contains(&y) {
                probes.push(y);
            }
        }

        for &idx in &probes {
            let y = bits(idx, n);
            let an = magic::amplitude(&naive, &y);
            let ab = magic::amplitude(&bg, &y);

            // Exact ring equality: both are exact, so nothing is rounded away.
            assert!(
                cyc_eq(an, ab),
                "EXACT mismatch at circuit {i} state {idx}: naive {an:?} vs bg {ab:?}\n{:?}",
                c.gates
            );

            let (rn, in_) = an.to_complex();
            let (rb, ib) = ab.to_complex();
            let (rr, ir) = holon_qasm::magic::magic_amplitude(&qc, &y, false, false);

            let dn = ((rn - rr).powi(2) + (in_ - ir).powi(2)).sqrt();
            let db = ((rb - rr).powi(2) + (ib - ir).powi(2)).sqrt();
            let dp = ((rn - rb).powi(2) + (in_ - ib).powi(2)).sqrt();
            assert!(dn < 1e-10, "naive vs reference {dn:e} at circuit {i} state {idx}");
            assert!(db < 1e-10, "BG vs reference {db:e} at circuit {i} state {idx}");
            worst_ref = worst_ref.max(dn.max(db));
            worst_pair = worst_pair.max(dp);
            checked += 1;
        }
        circuits += 1;
    }

    assert!(circuits >= 60, "conformance must cover at least 60 circuits, got {circuits}");
    println!(
        "[magic conformance] {circuits} circuits, {checked} amplitudes, \
         worst |Δ| vs certified reference = {worst_ref:e}, worst |Δ| BG vs naive = {worst_pair:e}"
    );
}

/// The branch count is the deliverable. Assert its shape, and print the cost
/// curve integration needs to compute the achieved exponent.
#[test]
fn branch_count_is_the_achieved_rank() {
    println!("[magic cost] A6_WIDTH = {A6_WIDTH}, A6_RANK = {A6_RANK}");
    println!("[magic cost] provenance: {A6_PROVENANCE}");
    println!("[magic cost]  t  naive=2^t   bg branches   exponent log2(bg)/t");
    let mut rng = Rng(0xC0FF_EE00_1234_5678);

    // The expectation is written out here INDEPENDENTLY of block_plan, so this
    // is a check and not an echo: six-wide blocks only when they beat three
    // pair blocks, then pairs, then one leftover.
    let expected = |t: usize| -> u64 {
        let (six, rest) = if A6_RANK < 8 { (t / A6_WIDTH, t % A6_WIDTH) } else { (0, t) };
        (A6_RANK as u64).pow(six as u32) * (1u64 << rest.div_ceil(2))
    };

    for t in 0..=18usize {
        let c = random_circuit(&mut rng, 4, 8, t);
        let bg = BgSource::new(&c);
        assert_eq!(bg.n_branches(), expected(t), "branch count shape at t={t}");
        assert_eq!(NaiveSource::new(&c).n_branches(), 1u64 << t);
        println!(
            "[magic cost] {t:3}  {:9}   {:9}   {:.4}",
            1u64 << t,
            bg.n_branches(),
            bg.exponent()
        );
    }

    // The point of the exercise: strictly cheaper than the naive sum from two
    // T gates up. The exponent is 0.5 exactly at even t and pays the ⌈t/2⌉
    // ceiling at odd t — it approaches 0.5 from above, it does not sit at it.
    for t in 2..=18usize {
        let c = random_circuit(&mut rng, 4, 8, t);
        let bg = BgSource::new(&c);
        assert!(bg.n_branches() < 1u64 << t, "no reduction at t={t}");
        assert!(bg.n_branches() <= 1u64 << t.div_ceil(2), "above the ⌈t/2⌉ ceiling at t={t}");
        if t % 2 == 0 {
            assert!((bg.exponent() - 0.5).abs() < 1e-12, "exponent {} at t={t}", bg.exponent());
        }
    }
}

/// The decomposition table is re-derived, not remembered: this recomputes
/// Σ_j c_j φ_j(x) in the exact ring at all 64 basis states.
#[test]
fn decomposition_verified_exactly() {
    assert!(magic::decomposition_is_exact(&magic::a6_terms(), A6_WIDTH));
    assert!(magic::decomposition_is_exact(&magic::a1_terms(), 1));
    assert_eq!(magic::a6_terms().len(), A6_RANK);
    // Every term must be a genuine stabilizer state: 2^k support, unit-modulus
    // phases. (A term with an empty support would be a silent rank saving.)
    for term in magic::a6_terms() {
        let mut support = 0usize;
        for x in 0..(1u32 << A6_WIDTH) {
            if !is_zero(term.amplitude(x)) {
                support += 1;
            }
        }
        assert_eq!(support, 1usize << term.cols.len(), "term support is not 2^k");
    }
}

/// Total probability is an ARITHMETIC invariant here, not a tolerance: the
/// branch sums are exact, so Σ_y |amp|² lands on 1 up to the final f64 cast.
#[test]
fn probabilities_sum_to_one() {
    let mut rng = Rng(0x1111_2222_3333_4444);
    for i in 0..12usize {
        let n = 2 + i % 4;
        let t = i % 7;
        let c = random_circuit(&mut rng, n, 7 + i, t);
        for (name, total) in [
            ("naive", {
                let s = NaiveSource::new(&c);
                (0..(1usize << n))
                    .map(|idx| {
                        let (re, im) = magic::amplitude(&s, &bits(idx, n)).to_complex();
                        re * re + im * im
                    })
                    .sum::<f64>()
            }),
            ("bg", {
                let s = BgSource::new(&c);
                (0..(1usize << n))
                    .map(|idx| {
                        let (re, im) = magic::amplitude(&s, &bits(idx, n)).to_complex();
                        re * re + im * im
                    })
                    .sum::<f64>()
            }),
        ] {
            assert!((total - 1.0).abs() < 1e-9, "{name} circuit {i}: total {total}");
        }
    }
}

/// The gauge: a planted defect in the shared affine engine must be CAUGHT by
/// this comparison. If it is not, the passing tests above mean nothing.
#[test]
fn planted_mutation_is_caught() {
    let mut rng = Rng(0x9999_8888_7777_6666);
    for mutation in [
        Mutation { drop_s_cross: true, wrong_gauss: false },
        Mutation { drop_s_cross: false, wrong_gauss: true },
    ] {
        let mut caught_naive = false;
        let mut caught_bg = false;
        for i in 0..40usize {
            let n = 2 + i % 4;
            let t = i % 5;
            let c = random_circuit(&mut rng, n, 8 + i % 6, t);
            let qc = to_qasm(&c);
            let bad_n = NaiveSource::with_mutation(&c, mutation);
            let bad_b = BgSource::with_mutation(&c, mutation);
            for idx in 0..(1usize << n) {
                let y = bits(idx, n);
                let (rr, ir) = holon_qasm::magic::magic_amplitude(&qc, &y, false, false);
                let (rn, in_) = magic::amplitude(&bad_n, &y).to_complex();
                let (rb, ib) = magic::amplitude(&bad_b, &y).to_complex();
                if ((rn - rr).powi(2) + (in_ - ir).powi(2)).sqrt() > 1e-9 {
                    caught_naive = true;
                }
                if ((rb - rr).powi(2) + (ib - ir).powi(2)).sqrt() > 1e-9 {
                    caught_bg = true;
                }
            }
            if caught_naive && caught_bg {
                break;
            }
        }
        assert!(caught_naive, "mutation {mutation:?} not caught in the naive source");
        assert!(caught_bg, "mutation {mutation:?} not caught in the BG source");
    }
}
