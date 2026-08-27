//! Referees for `holon::sample` — the exact branch-sum sampler.
//!
//! Three independent referees, none of them the code under test:
//!
//! 1. **`holon-qasm::magic`** (the certified affine engine, dev-dependency)
//!    refereeing the affine-state PORT, amplitude by amplitude.
//! 2. **Brute force** (Σ over all 2^n basis states) refereeing the exact
//!    pairwise overlap — the crux routine — at EXACT ring equality, never a
//!    tolerance.
//! 3. **`run_magic`** (the certified 2^n branch sum) refereeing the exact
//!    conditional probabilities, and then the empirical shot frequencies.
//!
//! Plus the gauge: a planted wrong Gauss-sum phase must make referee 2 fire.
//! A test suite that cannot fail on a planted defect is not evidence.
//!
//! NOTE on the merge law: `src/sample.rs` routes every accumulation through
//! `holon::merge::fold`, but these referees deliberately accumulate with
//! `Cyc::add` directly. A referee that shared the fold with the code it
//! referees would agree with it about a broken fold. `gram_fold_is_shard_invariant`
//! is the test that DOES exercise the law, and it is the only one that should.

use holon::ledger::Cyc;
use holon::sample::*;
use holon::BranchSource;
use holon_qasm::magic::{run_magic, Affine as CertAffine};
use holon_qasm::{Circuit, Gate};
use std::collections::BTreeMap;

// ------------------------------------------------------------------ helpers

fn clif_to_gate(g: Clif) -> Gate {
    match g {
        Clif::X(q) => Gate::X(q),
        Clif::Z(q) => Gate::Z(q),
        Clif::S(q) => Gate::S(q),
        Clif::Sdg(q) => Gate::Sdg(q),
        Clif::H(q) => Gate::H(q),
        Clif::Cx(c, t) => Gate::Cx(c, t),
    }
}

/// Weighted toward H and S on purpose: H is what CREATES affine dimension
/// (and so free variables for the Gauss sum to eat), S is what makes the
/// linear part d ODD (and so routes the sum through the branch the planted
/// mutation lives in). A uniform gate mix leaves both starved — measured on
/// a first pass of this suite: 28 odd-δ steps across 400 pairs.
fn random_clifford(rng: &mut Rng, n: usize) -> Clif {
    let table: &[u8] = if n >= 2 {
        &[0, 1, 2, 2, 3, 4, 4, 4, 5, 5, 5]
    } else {
        &[0, 1, 2, 2, 3, 4, 4, 4]
    };
    let pick = table[(rng.next_u64() as usize) % table.len()];
    let q = (rng.next_u64() as usize) % n;
    match pick {
        0 => Clif::X(q),
        1 => Clif::Z(q),
        2 => Clif::S(q),
        3 => Clif::Sdg(q),
        4 => Clif::H(q),
        _ => {
            let c = q;
            let mut t = (rng.next_u64() as usize) % n;
            if t == c {
                t = (t + 1) % n;
            }
            Clif::Cx(c, t)
        }
    }
}

fn random_clifford_state(rng: &mut Rng, n: usize, depth: usize) -> AffineState {
    let mut st = AffineState::new(n);
    for _ in 0..depth {
        st.apply(random_clifford(rng, n));
    }
    st
}

/// Any basis state in the support, by search. Only used to pick projections
/// the state can actually survive — the sampler never projects onto a
/// zero-probability outcome either, so projecting blindly would test a case
/// that never arises and starve the ones that do.
fn a_support_state(st: &AffineState) -> Option<Vec<bool>> {
    let n = st.n_qubits();
    (0..(1usize << n))
        .map(|idx| (0..n).map(|q| idx >> q & 1 == 1).collect::<Vec<bool>>())
        .find(|y| !cyc_is_zero(st.amplitude(y)))
}

/// A random PAIR of affine states, drawn from three families on purpose.
///
/// Independent random states are the wrong test on their own: their affine
/// subspaces are usually disjoint, so the overlap is zero before any Gauss
/// sum runs, and a suite made only of those referees almost nothing (measured
/// on a first pass: 309 of 400 pairs zero, the planted mutation reached 14
/// times). The families below are what the SAMPLER actually feeds `overlap`:
/// branches of one circuit that differ by a few Z insertions, then projected
/// on a COMMON prefix.
///
///  - family 0: two unrelated Clifford states — the disjoint case, and the
///    only one that exercises the inconsistent-constraint path.
///  - family 1: a shared Clifford prefix, then divergent tails — the shape a
///    branch list has.
///  - family 2: family 1, then both projected on the same qubits to the same
///    values, drawn from the LEFT state's support — exactly what a
///    conditional step computes.
fn random_pair(rng: &mut Rng, n: usize, depth: usize) -> (AffineState, AffineState) {
    match rng.next_u64() % 3 {
        0 => (random_clifford_state(rng, n, depth), random_clifford_state(rng, n, depth)),
        fam => {
            let base = random_clifford_state(rng, n, depth);
            let (mut a, mut b) = (base.clone(), base);
            for _ in 0..1 + (rng.next_u64() as usize) % 3 {
                a.apply(random_clifford(rng, n));
                b.apply(random_clifford(rng, n));
            }
            if fam == 2 {
                if let Some(y) = a_support_state(&a) {
                    for _ in 0..1 + (rng.next_u64() as usize) % n {
                        let q = (rng.next_u64() as usize) % n;
                        a.project(q, y[q]);
                        b.project(q, y[q]);
                    }
                }
            }
            (a, b)
        }
    }
}

fn cert_cyc(c: holon_qasm::magic::Cyc) -> Cyc {
    Cyc { c: c.c, m: c.m }
}

fn re(c: Cyc) -> f64 {
    let (r, i) = c.to_complex();
    assert!(i.abs() < 1e-12, "expected a real ring element, imag = {i}");
    r
}

/// A test circuit: the Clifford+T gate list plus the register width.
struct Tc {
    n: usize,
    gates: Vec<Gate>,
}

impl Tc {
    fn circuit(&self) -> Circuit {
        Circuit {
            n_qubits: self.n,
            n_clbits: self.n,
            gates: self.gates.clone(),
            measures: (0..self.n).map(|q| (q, q)).collect(),
        }
    }
    fn magic_state(&self) -> MagicState {
        let mut st = MagicState::new(self.n);
        for g in &self.gates {
            match *g {
                Gate::X(q) => st.apply(Clif::X(q)),
                Gate::Z(q) => st.apply(Clif::Z(q)),
                Gate::S(q) => st.apply(Clif::S(q)),
                Gate::Sdg(q) => st.apply(Clif::Sdg(q)),
                Gate::H(q) => st.apply(Clif::H(q)),
                Gate::Cx(c, t) => st.apply(Clif::Cx(c, t)),
                Gate::T(q) => st.t(q),
                Gate::Tdg(q) => st.tdg(q),
                Gate::Ccx(..) => panic!("magic tier branches must be Clifford+T"),
            }
        }
        st
    }
}

/// Random Clifford+T circuits inside the stated scope: n ≤ 5, t ≤ 4.
fn random_tc(rng: &mut Rng, n: usize, depth: usize, t_count: usize) -> Tc {
    let mut gates: Vec<Gate> = Vec::new();
    let mut t_left = t_count;
    for i in 0..depth {
        // Spread the T gates through the circuit rather than trailing them,
        // so the branches genuinely diverge before the Cliffords entangle.
        let remaining = depth - i;
        if t_left > 0 && (rng.next_u64() as usize) % remaining < t_left {
            let q = (rng.next_u64() as usize) % n;
            gates.push(if rng.next_u64() % 2 == 0 { Gate::T(q) } else { Gate::Tdg(q) });
            t_left -= 1;
        } else {
            gates.push(clif_to_gate(random_clifford(rng, n)));
        }
    }
    for _ in 0..t_left {
        gates.push(Gate::T((rng.next_u64() as usize) % n));
    }
    Tc { n, gates }
}

fn all_strings(n: usize) -> Vec<Vec<bool>> {
    (0..(1usize << n))
        .map(|idx| (0..n).map(|q| idx >> q & 1 == 1).collect())
        .collect()
}

// ------------------------------------------- referee 1: the port itself

/// The transplanted affine state must agree with the certified engine on
/// EVERY amplitude of EVERY random Clifford circuit — exactly, in the ring.
#[test]
fn affine_port_matches_certified_engine() {
    let mut rng = Rng::new(0x5EED_0001);
    let mut checked = 0usize;
    for trial in 0..200 {
        let n = 1 + (trial % 5);
        let depth = 4 + trial % 13;
        let mut mine = AffineState::new(n);
        let mut theirs = CertAffine::new(n);
        for _ in 0..depth {
            let g = random_clifford(&mut rng, n);
            mine.apply(g);
            theirs.apply(clif_to_gate(g));
        }
        for y in all_strings(n) {
            let a = mine.amplitude(&y);
            let b = cert_cyc(theirs.amplitude(&y));
            assert!(
                cyc_eq(a, b),
                "port diverged from the certified engine at trial {trial}, y={y:?}: {a:?} vs {b:?}"
            );
            checked += 1;
        }
    }
    assert!(checked >= 200);
}

// ------------------------------------------- referee 2: overlap vs brute force

/// The crux. `overlap` is O(k³) and closed-form; `overlap_bruteforce` is a
/// 2^n sum over basis states. They must agree EXACTLY, on states that have
/// been projected (so the subspaces genuinely differ in dimension and offset)
/// as well as on full stabilizer states.
#[test]
fn overlap_matches_bruteforce() {
    let mut rng = Rng::new(0x5EED_0002);
    let pairs = 400;
    let mut nonzero = 0usize;
    let mut zero = 0usize;
    let mut max_k = 0usize;
    let mut cover = GaussStats::default();
    for trial in 0..pairs {
        let n = 2 + (trial % 9); // 2..=10
        let (a, b) = random_pair(&mut rng, n, 4 * n + trial % 7);
        max_k = max_k.max(a.k() + b.k());
        let fast = overlap_gauged(&a, &b, false, &mut cover);
        let slow = overlap_bruteforce(&a, &b);
        assert!(
            cyc_eq(fast, slow),
            "overlap mismatch at trial {trial} (n={n}, k={}+{}): fast={fast:?} slow={slow:?}",
            a.k(),
            b.k()
        );
        if cyc_is_zero(slow) {
            zero += 1;
        } else {
            nonzero += 1;
        }
    }
    eprintln!(
        "overlap vs brute force: {pairs} pairs, {nonzero} nonzero, {zero} zero, \
         max k_a+k_b = {max_k}, coverage {cover:?}"
    );
    // A suite where every answer is zero, or where the odd-δ Gauss branch is
    // never reached, would pass without refereeing the crux.
    assert!(nonzero >= pairs / 2, "only {nonzero}/{pairs} pairs had nonzero overlap");
    assert!(zero >= 1, "no disjoint-subspace pair was exercised");
    assert!(max_k >= 8, "joint variable count never exceeded {max_k}");
    assert!(cover.odd_steps >= 100, "odd-δ Gauss branch reached only {} times", cover.odd_steps);
    assert!(cover.even_steps >= 100, "even-δ Gauss branch reached only {} times", cover.even_steps);
    assert!(cover.annihilated >= 1, "the vanishing-Gauss-sum path was never reached");
    assert!(cover.inconsistent >= 1, "the disjoint-subspace path was never reached");
}

/// ⟨φ|φ'⟩ = conj(⟨φ'|φ⟩), and ⟨φ|φ⟩ is real and non-negative. `norm_sq`
/// ASSUMES the first of these (it accumulates t + t̄ instead of both
/// triangles), so it is tested here rather than assumed there.
#[test]
fn overlap_is_hermitian_and_psd_on_the_diagonal() {
    let mut rng = Rng::new(0x5EED_0003);
    for trial in 0..200 {
        let n = 2 + (trial % 4);
        let (a, b) = random_pair(&mut rng, n, 4 * n + trial % 7);
        assert!(cyc_eq(overlap(&a, &b), cyc_conj(overlap(&b, &a))), "not Hermitian at {trial}");
        let d = overlap(&a, &a);
        let mut over = false;
        assert!(
            cyc_real_cmp(d, Cyc::ZERO, &mut over) != std::cmp::Ordering::Less,
            "negative self-overlap at trial {trial}: {d:?}"
        );
        assert!(!over);
    }
}

// ------------------------------------------- referee 2b: the planted mutation

/// The gauge. A wrong Gauss-sum phase — dropping the `1 +` from the odd-δ
/// prefactor (1 + i^δ), the single most plausible way to be silently wrong —
/// must make `overlap_matches_bruteforce` fire. If it does not, that test is
/// not evidence about the Gauss sum.
#[test]
fn planted_wrong_gauss_fires_the_overlap_test() {
    let mut rng = Rng::new(0x5EED_0002); // the SAME stream as the test it gauges
    let pairs = 400;
    let mut fired = 0usize;
    let mut reached = 0usize;
    let mut survived_nonzero = 0usize;
    let mut survived_zero = 0usize;
    for trial in 0..pairs {
        let n = 2 + (trial % 9);
        let (a, b) = random_pair(&mut rng, n, 4 * n + trial % 7);
        let slow = overlap_bruteforce(&a, &b);
        let mut stats = GaussStats::default();
        let mutated = overlap_gauged(&a, &b, true, &mut stats);
        let hit_odd = stats.odd_steps > 0;
        if hit_odd {
            reached += 1;
        }
        if !cyc_eq(mutated, slow) {
            fired += 1;
        } else if hit_odd {
            // Survivals are expected ONLY when the true overlap is zero: the
            // mutation is a multiplicative change to γ, so a sum that a later
            // parity constraint annihilates reads zero either way. A survivor
            // with a nonzero true overlap would be a real hole in the gauge.
            if cyc_is_zero(slow) {
                survived_zero += 1;
            } else {
                survived_nonzero += 1;
            }
        }
    }
    eprintln!(
        "planted wrong Gauss sum: fired on {fired}/{pairs} pairs; {reached} reached the odd-δ \
         branch; survivors {survived_zero} (true overlap zero) + {survived_nonzero} (nonzero)"
    );
    assert!(reached >= 100, "the mutated branch was reached only {reached} times");
    assert!(fired > 0, "the planted wrong Gauss phase was INVISIBLE to the brute-force referee");
    assert_eq!(
        survived_nonzero, 0,
        "the mutation hid on {survived_nonzero} pairs with a NONZERO true overlap"
    );
}

// ------------------------------------------- referee 3: probabilities

/// A state built by unitaries from |0…0⟩ has ⟨ψ|ψ⟩ = 1 EXACTLY — an
/// arithmetic invariant of the ring, not a tolerance. This is the Gram sum
/// (branches are not orthogonal) so it is a real test of `overlap`.
#[test]
fn norm_is_exactly_one() {
    let mut rng = Rng::new(0x5EED_0004);
    for trial in 0..40 {
        let n = 1 + trial % 4;
        let t = trial % 5;
        let tc = random_tc(&mut rng, n, 6 + trial % 8, t);
        let st = tc.magic_state();
        let norm = st.norm_sq();
        assert!(cyc_eq(norm, Cyc::ONE), "norm ≠ 1 at trial {trial} (n={n}, t={t}): {norm:?}");
    }
}

/// The conditional chain's probabilities must equal the certified 2^n branch
/// sum's, string by string, and must sum to exactly 1 in the ring.
#[test]
fn exact_probs_match_certified_branch_sum() {
    let mut rng = Rng::new(0x5EED_0005);
    let mut worst = 0.0f64;
    let mut worst_at = String::new();
    for trial in 0..60 {
        let n = 1 + trial % 5;
        let t = trial % 5;
        let tc = random_tc(&mut rng, n, 6 + trial % 10, t);
        let reference = run_magic(&tc.circuit(), false, false);
        let mut sampler = Sampler::new(tc.magic_state());
        assert!(cyc_eq(sampler.total_weight(), Cyc::ONE));
        let mut total = Cyc::ZERO;
        for x in all_strings(n) {
            let p = sampler.exact_prob(&x);
            total = total.add(p);
            let mine = re(p);
            let theirs = *reference.get(&bitstring_key(&x)).unwrap_or(&0.0);
            let dev = (mine - theirs).abs();
            if dev > worst {
                worst = dev;
                worst_at = format!("trial {trial} n={n} t={t} x={}", bitstring_key(&x));
            }
            assert!(dev < 1e-12, "P mismatch: mine={mine} certified={theirs} at trial {trial}");
        }
        assert!(cyc_eq(total, Cyc::ONE), "conditional chain does not sum to 1 at trial {trial}");
    }
    eprintln!("exact probs vs certified branch sum: worst |Δ| = {worst:.3e} at {worst_at}");
}

/// `MagicState` is a `BranchSource`, and the contract's fold must reproduce
/// the amplitude the certified engine reports.
#[test]
fn branch_source_contract_folds_to_the_amplitude() {
    let mut rng = Rng::new(0x5EED_0006);
    for trial in 0..30 {
        let n = 1 + trial % 4;
        let tc = random_tc(&mut rng, n, 6 + trial % 8, trial % 5);
        let st = tc.magic_state();
        let c = tc.circuit();
        for x in all_strings(n) {
            let mut acc = Cyc::ZERO;
            for b in 0..st.n_branches() {
                acc = acc.add(st.amplitude_of(b, &x));
            }
            assert!(cyc_eq(acc, st.amplitude(&x)));
            let (re_c, im_c) = holon_qasm::magic::magic_amplitude(&c, &x, false, false);
            let (re_m, im_m) = acc.to_complex();
            assert!(
                (re_c - re_m).abs() < 1e-12 && (im_c - im_m).abs() < 1e-12,
                "BranchSource fold ≠ certified amplitude at trial {trial}"
            );
        }
        assert_eq!(st.n_qubits(), n);
    }
}

// ------------------------------------------- referee 3b: the shots themselves

fn chi_square(counts: &BTreeMap<String, u64>, exact: &BTreeMap<String, f64>, shots: u64) -> (f64, usize, f64) {
    let mut chi2 = 0.0;
    let mut df = 0usize;
    let mut max_dev = 0.0f64;
    for (key, &p) in exact {
        let o = *counts.get(key).unwrap_or(&0) as f64;
        let e = p * shots as f64;
        if p > 0.0 {
            df += 1;
            chi2 += (o - e) * (o - e) / e;
        }
        max_dev = max_dev.max((o / shots as f64 - p).abs());
    }
    for (key, &o) in counts {
        assert!(
            exact.get(key).copied().unwrap_or(0.0) > 0.0,
            "sampled {key} ({o} times) but its exact probability is ZERO"
        );
    }
    (chi2, df.saturating_sub(1), max_dev)
}

/// End to end: 20k shots per circuit, empirical frequencies against the
/// certified exact probabilities. The chi-square bound is df + 10√(2df) + 20,
/// which is far out in the upper tail (p ≪ 10⁻⁸) — a loose bound on purpose,
/// because the claim under test is "the sampler draws from the right
/// distribution", not "these particular 20k draws are typical".
#[test]
fn sampled_frequencies_match_exact_probabilities() {
    let shots = 20_000u64;
    let cases: Vec<Tc> = vec![
        // |+⟩T|+⟩ style: a single T on a superposition — the (2±√2)/4 split.
        Tc { n: 1, gates: vec![Gate::H(0), Gate::T(0), Gate::H(0)] },
        // Bell pair with a T on each side.
        Tc {
            n: 2,
            gates: vec![Gate::H(0), Gate::Cx(0, 1), Gate::T(0), Gate::T(1), Gate::H(0), Gate::H(1)],
        },
        // 3 qubits, 4 T gates, entangling Cliffords between them.
        Tc {
            n: 3,
            gates: vec![
                Gate::H(0), Gate::H(1), Gate::H(2),
                Gate::T(0), Gate::Cx(0, 1), Gate::T(1), Gate::Cx(1, 2),
                Gate::Tdg(2), Gate::H(1), Gate::T(2), Gate::Cx(2, 0), Gate::H(0), Gate::S(2),
            ],
        },
        // 5 qubits, 4 T gates — the top of the stated scope.
        Tc {
            n: 5,
            gates: vec![
                Gate::H(0), Gate::H(1), Gate::H(2), Gate::H(3), Gate::H(4),
                Gate::T(0), Gate::Cx(0, 1), Gate::T(2), Gate::Cx(2, 3),
                Gate::Cx(3, 4), Gate::T(4), Gate::H(1), Gate::Tdg(1),
                Gate::Cx(1, 0), Gate::H(3), Gate::S(4), Gate::H(2),
            ],
        },
    ];
    for (i, tc) in cases.iter().enumerate() {
        let exact = run_magic(&tc.circuit(), false, false);
        let st = tc.magic_state();
        let n_branches = st.n_branches();
        let mut sampler = Sampler::new(st);
        assert!(cyc_eq(sampler.total_weight(), Cyc::ONE));
        let counts = sampler.sample_counts(shots as usize, 0xC0FF_EE00 + i as u64);
        assert_eq!(counts.values().sum::<u64>(), shots);
        let (chi2, df, max_dev) = chi_square(&counts, &exact, shots);
        let bound = df as f64 + 10.0 * (2.0 * df as f64).sqrt() + 20.0;
        eprintln!(
            "case {i}: n={} t={} branches={} outcomes={} chi2={chi2:.2} (df={df}, bound={bound:.1}) \
             max|Δfreq|={max_dev:.4} nodes={} overlaps={}",
            tc.n,
            tc.gates.iter().filter(|g| matches!(g, Gate::T(_) | Gate::Tdg(_))).count(),
            n_branches,
            df + 1,
            sampler.cache_nodes(),
            sampler.overlaps()
        );
        assert!(chi2 < bound, "case {i}: chi2 {chi2} exceeds {bound} (df {df})");
        // Every sampling decision must have been exact integer arithmetic.
        assert_eq!(sampler.approx_compares(), 0, "case {i} fell back to f64 in a decision");
        // The cache must have amortised: far fewer nodes than shots × n.
        assert!(sampler.cache_nodes() < (1 << (tc.n + 1)));
    }
}

/// Deterministic given the seed, and a different seed gives a different
/// stream (otherwise "deterministic" would be satisfied by a constant).
#[test]
fn sampler_is_deterministic() {
    let tc = Tc {
        n: 3,
        gates: vec![Gate::H(0), Gate::Cx(0, 1), Gate::T(1), Gate::H(2), Gate::T(2), Gate::Cx(2, 0)],
    };
    let a = Sampler::new(tc.magic_state()).sample(500, 7);
    let b = Sampler::new(tc.magic_state()).sample(500, 7);
    let c = Sampler::new(tc.magic_state()).sample(500, 8);
    assert_eq!(a, b, "same seed gave a different stream");
    assert_ne!(a, c, "different seeds gave the same stream");
}

/// A deterministic outcome must be sampled with certainty, and a
/// zero-probability outcome never — the two ends the frequency test is
/// weakest at.
#[test]
fn deterministic_and_forbidden_outcomes() {
    // X on qubit 1 only: the only outcome is 010 (MSB-first over 3 qubits).
    let tc = Tc { n: 3, gates: vec![Gate::X(1)] };
    let mut s = Sampler::new(tc.magic_state());
    let counts = s.sample_counts(1000, 1);
    assert_eq!(counts.len(), 1);
    assert_eq!(counts["010"], 1000);

    // A Bell pair forbids 01 and 10 exactly.
    let tc = Tc { n: 2, gates: vec![Gate::H(0), Gate::Cx(0, 1)] };
    let mut s = Sampler::new(tc.magic_state());
    assert!(cyc_is_zero(s.exact_prob(&[true, false])));
    assert!(cyc_is_zero(s.exact_prob(&[false, true])));
    let counts = s.sample_counts(4000, 2);
    assert_eq!(counts.len(), 2);
    assert!(counts.contains_key("00") && counts.contains_key("11"));
    let n00 = counts["00"] as f64 / 4000.0;
    assert!((n00 - 0.5).abs() < 0.04, "Bell split off: {n00}");
}

/// The exact conditional weight of a partial prefix must equal the sum of the
/// full-string probabilities below it — the marginalisation identity, in the
/// ring rather than in floating point.
#[test]
fn prefix_weights_marginalise_exactly() {
    let mut rng = Rng::new(0x5EED_0007);
    for trial in 0..20 {
        let n = 2 + trial % 4;
        let tc = random_tc(&mut rng, n, 6 + trial % 8, trial % 4);
        let mut s = Sampler::new(tc.magic_state());
        for cut in 0..n {
            for pidx in 0..(1usize << cut) {
                let prefix: Vec<bool> = (0..cut).map(|i| pidx >> i & 1 == 1).collect();
                let w = s.prefix_weight(&prefix);
                let mut acc = Cyc::ZERO;
                for sidx in 0..(1usize << (n - cut)) {
                    let mut x = prefix.clone();
                    for i in 0..(n - cut) {
                        x.push(sidx >> i & 1 == 1);
                    }
                    acc = acc.add(s.exact_prob(&x));
                }
                assert!(cyc_eq(w, acc), "marginalisation failed at trial {trial}, prefix {prefix:?}");
            }
        }
    }
}

/// Bias check across seeds. One chi-square in the upper tail is unremarkable
/// and proves nothing either way; a MEAN that sits above the degrees of
/// freedom is bias. E[χ²_df] = df and sd of the mean over K seeds is
/// √(2df/K), so this is a ~4σ two-sided band on the sampler being right.
///
/// It also re-uses ONE `Sampler` across every seed, which is the cache doing
/// its job: 60 × 2000 = 120k shots for the Gram work of one conditional tree.
#[test]
fn sampler_is_unbiased_across_seeds() {
    let tc = Tc {
        n: 3,
        gates: vec![
            Gate::H(0), Gate::H(1), Gate::H(2),
            Gate::T(0), Gate::Cx(0, 1), Gate::T(1), Gate::Cx(1, 2),
            Gate::Tdg(2), Gate::H(1), Gate::T(2), Gate::Cx(2, 0), Gate::H(0), Gate::S(2),
        ],
    };
    let exact = run_magic(&tc.circuit(), false, false);
    let mut sampler = Sampler::new(tc.magic_state());
    let seeds = 60u64;
    let shots = 2_000u64;
    let mut sum = 0.0;
    let mut df_seen = 0usize;
    // The FIRST batch is what fills the conditional tree; every batch after it
    // must be pure cache, which is the claim `overlaps_after_first` checks.
    let mut overlaps_after_first = 0u64;
    for seed in 0..seeds {
        let counts = sampler.sample_counts(shots as usize, 0xB1A5_0000 + seed);
        if seed == 0 {
            overlaps_after_first = sampler.overlaps();
        }
        let (chi2, df, _) = chi_square(&counts, &exact, shots);
        sum += chi2;
        df_seen = df;
    }
    let mean = sum / seeds as f64;
    let sd = (2.0 * df_seen as f64 / seeds as f64).sqrt();
    eprintln!(
        "bias check: mean χ² = {mean:.3} over {seeds} seeds (df = {df_seen}, expected {df_seen} ± {sd:.3}); \
         {} overlaps for {} shots",
        sampler.overlaps(),
        seeds * shots
    );
    assert!(
        (mean - df_seen as f64).abs() < 4.0 * sd,
        "mean χ² {mean} is {:.1}σ from df {df_seen} — the sampler is biased",
        (mean - df_seen as f64).abs() / sd
    );
    // After the tree is built, extra shots must cost NO further Gram work.
    assert_eq!(
        sampler.overlaps(),
        overlaps_after_first,
        "the conditional cache is not amortising across seeds"
    );
    assert_eq!(sampler.approx_compares(), 0);
}

/// The conditional chain and the amplitude route share NO code below the
/// affine state: one goes through projection + Gram overlaps + Gauss sums,
/// the other through linear solve + phase evaluation. They must agree
/// EXACTLY in the ring, not to a tolerance — which is a stronger statement
/// than the comparison against `run_magic`, since that one has to go through
/// f64 to compare at all.
#[test]
fn conditional_chain_equals_the_amplitude_route_exactly() {
    let mut rng = Rng::new(0x5EED_0008);
    for trial in 0..40 {
        let n = 1 + trial % 5;
        let t = trial % 5;
        let tc = random_tc(&mut rng, n, 6 + trial % 10, t);
        let st = tc.magic_state();
        let mut sampler = Sampler::new(st.clone());
        for x in all_strings(n) {
            let via_overlaps = sampler.exact_prob(&x);
            let via_amplitude = cyc_abs_sq(st.amplitude(&x));
            assert!(
                cyc_eq(via_overlaps, via_amplitude),
                "routes disagree at trial {trial} (n={n}, t={t}), x={}: {via_overlaps:?} vs {via_amplitude:?}",
                bitstring_key(&x)
            );
        }
    }
}

/// The cost curve, measured rather than asserted — run with
/// `cargo test -p holon --release --test sample cost_curve -- --ignored --nocapture`.
///
/// Ignored by default because it is a measurement, not a check: the quadratic
/// in branch count is the honest price of the exact route and there is no
/// pass/fail to attach to it. What it prints is the scope this module is
/// actually usable in.
#[test]
#[ignore]
fn cost_curve() {
    use std::time::Instant;
    eprintln!("{:>3} {:>3} {:>9} {:>10} {:>12} {:>12}", "n", "t", "branches", "overlaps", "build", "1000 shots");
    for &(n, t) in &[
        (4usize, 0usize), (4, 2), (4, 4), (4, 6), (4, 8), (4, 10),
        (6, 4), (6, 8), (8, 4), (8, 8), (10, 6), (12, 6),
    ] {
        let mut rng = Rng::new(0xC057_0000 + (n * 100 + t) as u64);
        let tc = random_tc(&mut rng, n, 6 * n, t);
        let st = tc.magic_state();
        let branches = st.n_branches();
        let t0 = Instant::now();
        let mut s = Sampler::new(st);
        let build = t0.elapsed();
        let t1 = Instant::now();
        let _ = s.sample(1000, 42);
        let shots = t1.elapsed();
        eprintln!(
            "{n:>3} {t:>3} {branches:>9} {:>10} {:>12?} {:>12?}",
            s.overlaps(),
            build,
            shots
        );
    }
}

/// The upper end of the working scope, where the exact ring coefficients are
/// largest and the i128 comparison is closest to its limit. Everything that
/// is claimed exact must still be exact here: unit norm in the ring, the
/// conditional split, agreement with the certified branch sum, and NO
/// fallback to floating point in any sampling decision.
#[test]
fn exactness_holds_at_the_top_of_scope() {
    let mut rng = Rng::new(0x5EED_0009);
    for (n, t) in [(6usize, 6usize), (5, 7)] {
        let tc = random_tc(&mut rng, n, 6 * n, t);
        let reference = run_magic(&tc.circuit(), false, false);
        let st = tc.magic_state();
        let branches = st.n_branches();
        let mut s = Sampler::new(st);
        assert!(cyc_eq(s.total_weight(), Cyc::ONE), "norm ≠ 1 at n={n}, t={t}");
        let mut worst = 0.0f64;
        let mut total = Cyc::ZERO;
        for x in all_strings(n) {
            let p = s.exact_prob(&x);
            total = total.add(p);
            let theirs = *reference.get(&bitstring_key(&x)).unwrap_or(&0.0);
            worst = worst.max((re(p) - theirs).abs());
        }
        assert!(cyc_eq(total, Cyc::ONE));
        assert!(worst < 1e-12, "worst |Δ| = {worst} at n={n}, t={t}");
        let counts = s.sample_counts(2_000, 0xA11_0000 + n as u64);
        assert_eq!(counts.values().sum::<u64>(), 2_000);
        assert_eq!(s.approx_compares(), 0, "fell back to f64 at n={n}, t={t}");
        eprintln!(
            "scope n={n} t={t}: {branches} branches, {} overlaps, {} cache nodes, worst |Δ|={worst:.2e}, \
             {} distinct outcomes",
            s.overlaps(),
            s.cache_nodes(),
            counts.len()
        );
    }
}

/// The merge law's warrant, exercised on the sampler's own accumulation.
///
/// `holon::merge` names "sampler pair sums" as an instance of the one fold,
/// and the payoff of that claim is shardability: the Gram terms are
/// independent ledger entries, so re-ordering the branch list — which
/// re-orders AND re-pairs every term in the fold — must land on the same
/// value. A failure here would be a misfit to report against the law, not a
/// tolerance to widen.
#[test]
fn gram_fold_is_shard_invariant() {
    let mut rng = Rng::new(0x5EED_000A);
    let mut identical_repr = 0usize;
    let mut trials = 0usize;
    for trial in 0..30 {
        let n = 2 + trial % 4;
        let tc = random_tc(&mut rng, n, 6 + trial % 8, 1 + trial % 4);
        let st = tc.magic_state();
        let base = st.norm_sq();

        // Reversed, rotated, and odd-then-even: three different pairings of
        // the same Gram matrix.
        let bs: Vec<_> = st.branches().to_vec();
        let rev: Vec<_> = bs.iter().rev().cloned().collect();
        let rot: Vec<_> = bs[bs.len() / 2..].iter().chain(&bs[..bs.len() / 2]).cloned().collect();
        let inter: Vec<_> = bs
            .iter()
            .skip(1)
            .step_by(2)
            .chain(bs.iter().step_by(2))
            .cloned()
            .collect();
        for (name, perm) in [("reversed", rev), ("rotated", rot), ("interleaved", inter)] {
            let other = MagicState::from_branches(n, perm);
            let got = other.norm_sq();
            assert!(
                cyc_eq(got, base),
                "the Gram fold is order-DEPENDENT ({name}) at trial {trial}: {got:?} vs {base:?}"
            );
            trials += 1;
            if got == base {
                identical_repr += 1;
            }
        }
    }
    eprintln!(
        "shard invariance: {trials} re-orderings all equal in value; {identical_repr} also \
         bit-identical in representation"
    );
    assert!(trials >= 90);
}
