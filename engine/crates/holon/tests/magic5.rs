//! Conformance for MAGIC5-FROM-CAT (Kissinger–van de Wetering–Vilmart,
//! arXiv:2202.09202; reference implementation quizx, Apache-2.0 — both credited
//! in `magic5.rs`).
//!
//! THE REGRESSION GATE is `magic5_equals_the_current_path_exactly`: on random
//! Clifford+T circuits small enough for both paths, the Magic5 source and the
//! existing BG/naive sources must agree BIT-FOR-BIT in `Z[ω]·2^{−m/2}`. Not
//! within a tolerance — the same ring element. A new decomposition that is
//! merely close is a wrong decomposition.
//!
//! `holon-qasm`'s certified magic tier is the INDEPENDENT referee (the
//! `tests/pipeline.rs` pattern), consulted at 1e-10 because it returns f64.
//!
//! The harness carries its own teeth: `planted_mutation_is_caught` shows the
//! comparison fails when the engine is wrong, so a pass means something.

use holon::magic::{self, cyc_eq, BgSource, Circuit, Gate, Mutation, NaiveSource};
use holon::magic5::{
    expected_branches, magic5_is_exact, register_is_exact, Magic5Plan, Magic5Source, MAGIC5_ALPHA,
    MAGIC5_CONSUMED, MAGIC5_RANK, MAGIC5_WIDTH,
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

// ------------------------------------------------------- THE REGRESSION GATE

/// EXACT equality with the current path. No tolerance anywhere on this line.
#[test]
fn magic5_equals_the_current_path_exactly() {
    let mut rng = Rng(0x0A15_0000_0000_0001);
    let mut circuits = 0usize;
    let mut checked = 0usize;
    let mut nonzero = 0usize;
    let mut worst_ref = 0.0f64;

    for i in 0..80usize {
        let n = 2 + i % 5; // 2..=6, dense enough that amplitudes are not all zero
        let t = i % 11; // 0..=10 — small enough for the 2^t naive path
        let depth = 6 + (i % 11);
        let c = random_circuit(&mut rng, n, depth, t);
        let qc = to_qasm(&c);

        let naive = NaiveSource::new(&c);
        let bg = BgSource::new(&c);
        let m5 = Magic5Source::new(&c);
        assert_eq!(m5.n_qubits(), n);
        assert_eq!(m5.n_branches(), expected_branches(t), "branch count at t={t}");

        for idx in 0..(1usize << n) {
            let y = bits(idx, n);
            let an = magic::amplitude(&naive, &y);
            let ab = magic::amplitude(&bg, &y);
            let a5 = magic::amplitude(&m5, &y);

            assert!(
                cyc_eq(an, a5),
                "EXACT mismatch vs naive at circuit {i} (t={t}) state {idx}: \
                 naive {an:?} vs magic5 {a5:?}\n{:?}",
                c.gates
            );
            assert!(
                cyc_eq(ab, a5),
                "EXACT mismatch vs BG at circuit {i} (t={t}) state {idx}: \
                 bg {ab:?} vs magic5 {a5:?}\n{:?}",
                c.gates
            );

            if !holon::affine::cyc_is_zero(a5) {
                nonzero += 1;
            }

            // The INDEPENDENT referee: holon-qasm's certified magic tier.
            let (r5, i5) = a5.to_complex();
            let (rr, ir) = holon_qasm::magic::magic_amplitude(&qc, &y, false, false);
            let d = ((r5 - rr).powi(2) + (i5 - ir).powi(2)).sqrt();
            assert!(d < 1e-10, "magic5 vs certified reference {d:e} at circuit {i} state {idx}");
            worst_ref = worst_ref.max(d);
            checked += 1;
        }
        circuits += 1;
    }

    assert!(circuits >= 72, "gate must cover at least 72 circuits, got {circuits}");
    // A gate that only ever compared zeros would pass vacuously.
    assert!(nonzero >= 200, "gate saw only {nonzero} nonzero amplitudes — near-vacuous");
    println!(
        "[magic5 gate] {circuits} circuits, {checked} amplitudes ({nonzero} nonzero), \
         EXACT ring agreement with naive AND BG on every one; \
         worst |Δ| vs certified reference = {worst_ref:e}"
    );
}

/// The same gate on a WIDE register (the brief's n ≤ 24): the branch count is
/// set by `t` alone, so the qubit count is free — but the affine solve is not,
/// and this is where a width bug would show. An H-prefix keeps the support
/// full so the comparison is not comparing zeros.
#[test]
fn magic5_matches_on_a_wide_register() {
    let mut rng = Rng(0xBEEF_0000_0000_0024);
    let mut nonzero = 0usize;
    for (n, t) in [(16usize, 9usize), (24, 10)] {
        // H on every wire, then Cliffords that cannot COLLAPSE the support (a
        // later H can, and a collapsed support makes the comparison compare
        // zeros). So R stays full rank and every y is in the coset.
        let mut gates: Vec<Gate> = (0..n).map(Gate::H).collect();
        let mut t_left = t;
        for k in 0..(4 * n) {
            let q = rng.below(n as u64) as usize;
            if t_left > 0 && k % 5 == 0 {
                t_left -= 1;
                gates.push(if rng.below(2) == 0 { Gate::T(q) } else { Gate::Tdg(q) });
                continue;
            }
            gates.push(match rng.below(5) {
                0 => Gate::S(q),
                1 => Gate::Sdg(q),
                2 => Gate::X(q),
                3 => Gate::Z(q),
                _ => {
                    let mut t2 = rng.below(n as u64) as usize;
                    if t2 == q {
                        t2 = (q + 1) % n;
                    }
                    Gate::Cx(q, t2)
                }
            });
        }
        while t_left > 0 {
            t_left -= 1;
            gates.push(Gate::T(rng.below(n as u64) as usize));
        }
        let c = Circuit { n_qubits: n, gates };
        assert_eq!(c.t_count(), t);

        let naive = NaiveSource::new(&c);
        let m5 = Magic5Source::new(&c);
        assert_eq!(m5.n_branches(), expected_branches(t));

        for _ in 0..12 {
            let y: Vec<bool> = (0..n).map(|_| rng.below(2) == 1).collect();
            let an = magic::amplitude(&naive, &y);
            let a5 = magic::amplitude(&m5, &y);
            assert!(cyc_eq(an, a5), "wide n={n} t={t}: naive {an:?} vs magic5 {a5:?}");
            if !holon::affine::cyc_is_zero(a5) {
                nonzero += 1;
            }
        }
        println!(
            "[magic5 wide] n={n} t={t}: naive {} branches, magic5 {} — exact agreement",
            naive.n_branches(),
            m5.n_branches()
        );
    }
    assert!(nonzero >= 8, "wide gate saw only {nonzero} nonzero amplitudes");
}

/// Deeper `t`, where the plan is several rounds rather than one: Magic5 against
/// the naive sum in the exact ring on the WHOLE register. The referee is not
/// consulted here because naive is already pinned to it above — this is the
/// recursion-composition check on live circuits.
#[test]
fn deep_t_rounds_compose() {
    let mut rng = Rng(0xD33F_0000_1234_5678);
    for (n, t) in [(3usize, 11usize), (4, 12), (2, 13), (3, 14)] {
        let c = random_circuit(&mut rng, n, 10, t);
        let naive = NaiveSource::new(&c);
        let m5 = Magic5Source::new(&c);
        for idx in 0..(1usize << n) {
            let y = bits(idx, n);
            assert!(
                cyc_eq(magic::amplitude(&naive, &y), magic::amplitude(&m5, &y)),
                "t={t} state {idx}"
            );
        }
        println!(
            "[magic5 deep] n={n} t={t}: naive {} branches, magic5 {} ({} rounds) — \
             exact agreement on all {} states",
            naive.n_branches(),
            m5.n_branches(),
            m5.rounds(),
            1usize << n
        );
    }
}

// ------------------------------------------------------------ the exactness gates

/// The decomposition is re-derived, not remembered — both rungs.
#[test]
fn decomposition_verified_exactly() {
    assert!(magic5_is_exact(), "the five-qubit identity");
    for t in 0..=10usize {
        assert!(register_is_exact(t), "the recursive plan at t={t}");
    }
}

/// Structure of the plan: five in, one back, every round; and the keeper is the
/// SAME wire for all three terms, which is what makes the branch space a
/// deterministic mixed-radix index the mesh can shard by index alone.
#[test]
fn the_plan_is_branch_independent_and_consumes_four() {
    for t in 0..=40usize {
        let plan = Magic5Plan::new(7, t);
        let mut live = t;
        for grp in &plan.rounds {
            assert!(live >= MAGIC5_WIDTH, "a round fired at t_live={live}");
            live -= MAGIC5_CONSUMED;
            assert_eq!(grp.len(), MAGIC5_WIDTH);
        }
        assert!(live < MAGIC5_WIDTH, "a round was left on the table at t_live={live}");
        let tail_wires: usize = plan.tail.iter().map(|(q, _)| q.len()).sum();
        assert_eq!(tail_wires, live, "tail width at t={t}");

        let radices = plan.radices();
        assert_eq!(radices.len(), plan.rounds.len() + plan.tail.len());
        for r in &radices[..plan.rounds.len()] {
            assert_eq!(*r, MAGIC5_RANK as u64);
        }
        // Every wire the plan names is a magic wire, and no wire is named twice
        // as a keeper-and-consumed pair within one round.
        for grp in &plan.rounds {
            let mut seen = grp.to_vec();
            seen.sort_unstable();
            seen.dedup();
            assert_eq!(seen.len(), MAGIC5_WIDTH, "a round reused a wire");
            for &q in grp {
                assert!((7..7 + t).contains(&q), "round wire {q} is not a magic wire");
            }
        }
    }
}

// ------------------------------------------------------------ the measurement

/// The branch count is the deliverable. Print the cost table and the REALIZED
/// exponent — the finite-`t` number, never the asymptote.
#[test]
fn branch_counts_and_realized_exponent() {
    println!(
        "[magic5 cost] rule: {MAGIC5_WIDTH} in, 1 back, {MAGIC5_RANK} terms \
         => N(t) = 3·N(t−{MAGIC5_CONSUMED}); asymptotic alpha = log2(3)/4 = {MAGIC5_ALPHA:.6} \
         (an ASYMPTOTE, not a measurement)"
    );
    println!(
        "[magic5 cost]    t      naive 2^t   BG 2^ceil(t/2)        magic5   vs BG   realized alpha"
    );
    for t in [1usize, 4, 5, 6, 8, 10, 12, 16, 20, 24, 28, 32, 40, 48, 56, 64] {
        let plan = Magic5Plan::new(0, t);
        let m5 = plan.n_branches();
        assert_eq!(m5, expected_branches(t), "branch count at t={t}");
        let bg = 1u64 << t.div_ceil(2);
        let realized = (m5 as f64).log2() / t as f64;
        println!(
            "[magic5 cost] {t:4} {:>14} {bg:>16} {m5:>13} {:7.2}x   {realized:.4}",
            format!("2^{t}"),
            bg as f64 / m5 as f64
        );
        // The realized exponent is ABOVE the asymptote at every finite t.
        assert!(realized > MAGIC5_ALPHA, "realized {realized} claims the asymptote at t={t}");
    }

    // Strictly cheaper than the current path from the first round onwards, and
    // never worse below it (the tail IS the current path).
    for t in 0..=64usize {
        let m5 = expected_branches(t);
        let bg = 1u64 << t.div_ceil(2);
        if t >= MAGIC5_WIDTH {
            assert!(m5 < bg, "no reduction at t={t}: magic5 {m5} vs BG {bg}");
        } else {
            assert_eq!(m5, bg, "the sub-5 tail must BE the current path at t={t}");
        }
    }

    // The headline, spelled out so it cannot drift: 3^15·4 against 2^32.
    assert_eq!(expected_branches(64), 57_395_628);
    let gain = (1u64 << 32) as f64 / expected_branches(64) as f64;
    assert!((gain - 74.83).abs() < 0.01, "t=64 gain {gain}");
    println!("[magic5 cost] t=64: 2^32 = 4294967296 -> 3^15·4 = 57395628, {gain:.2}x fewer branches");
}

// ------------------------------------------------------------------- the gauge

/// A planted defect in the shared affine engine must be CAUGHT by this
/// comparison. If it is not, the passing tests above mean nothing.
#[test]
fn planted_mutation_is_caught() {
    let mut rng = Rng(0x5151_2424_9696_3737);
    for mutation in [
        Mutation { drop_s_cross: true, wrong_gauss: false },
        Mutation { drop_s_cross: false, wrong_gauss: true },
    ] {
        let mut caught = false;
        for i in 0..40usize {
            let n = 2 + i % 4;
            let t = 5 + i % 6;
            let c = random_circuit(&mut rng, n, 8 + i % 7, t);
            let clean = NaiveSource::new(&c);
            let bad = Magic5Source::with_mutation(&c, mutation);
            for idx in 0..(1usize << n) {
                let y = bits(idx, n);
                if !cyc_eq(magic::amplitude(&clean, &y), magic::amplitude(&bad, &y)) {
                    caught = true;
                    break;
                }
            }
            if caught {
                break;
            }
        }
        assert!(caught, "planted mutation {mutation:?} was NOT caught by the magic5 gate");
    }
    println!("[magic5 gauge] both planted engine defects caught");
}

/// Total probability is an ARITHMETIC invariant here, not a tolerance: the
/// branch sums are exact, so Σ_y |amp|² lands on 1 up to the final f64 cast.
#[test]
fn probabilities_sum_to_one() {
    let mut rng = Rng(0xA1B2_C3D4_E5F6_0789);
    for i in 0..14usize {
        let n = 2 + i % 4;
        let t = i % 12;
        let c = random_circuit(&mut rng, n, 7 + i, t);
        let s = Magic5Source::new(&c);
        let total: f64 = (0..(1usize << n))
            .map(|idx| {
                let (re, im) = magic::amplitude(&s, &bits(idx, n)).to_complex();
                re * re + im * im
            })
            .sum();
        assert!((total - 1.0).abs() < 1e-9, "circuit {i} (n={n}, t={t}): total {total}");
    }
}

/// The mixed-radix index space is what the mesh shards on, so the fold must be
/// INVARIANT under shard count — not approximately, identically. That contract
/// is the reason the recursion was made an INDEX rather than a tree walk: a
/// per-branch schedule that depended on earlier digits could not be sharded by
/// index at all.
#[test]
fn the_mixed_radix_index_shards_invariantly() {
    let mut rng = Rng(0x5A5A_0F0F_1234_9999);
    for i in 0..10usize {
        let n = 2 + i % 4;
        let t = 5 + i % 8;
        let c = random_circuit(&mut rng, n, 9 + i, t);
        let m5 = Magic5Source::new(&c);
        let nb = m5.n_branches();
        for idx in 0..(1usize << n) {
            let y = bits(idx, n);
            let one = holon::mesh::fold_amplitude(&m5, &y, 1);
            for shards in [2usize, 3, 5, 7, 16] {
                let many = holon::mesh::fold_amplitude(&m5, &y, shards);
                assert!(
                    cyc_eq(one, many),
                    "shard count {shards} moved the answer at circuit {i} state {idx} \
                     ({nb} branches)"
                );
            }
            // And the sharded fold is the plain branch sum, exactly.
            assert!(cyc_eq(one, magic::amplitude(&m5, &y)));
        }
    }
    println!("[magic5 mesh] fold is shard-invariant over the mixed-radix branch index");
}
