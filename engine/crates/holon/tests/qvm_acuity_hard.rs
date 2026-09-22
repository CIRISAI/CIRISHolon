//! QVM-ACUITY-1, THE HARD PART: the stakes S2–S5 and the plants PQ-3, PQ-4,
//! PQ-5 of `conformance/qasm/QVM_ACUITY1_PREREG.md`, one test per stake and
//! per plant, named for it.
//!
//! The claim under test is the third clause of the QVM's arithmetic: the hard
//! part is PRICED before it is run (`expected_branches(t_eff)`), evaluated in
//! a DECLARED order with a running remainder bound, and stopped as soon as the
//! demanded acuity is reached — with the unevaluated tail carried as a
//! certificate, `|value − truth| ≤ R_k ≤ ε`.
//!
//! THE REFEREES, and both are needed:
//!
//! * the FULL exact branch sum (`ε = 0`, exact `Z[ω]` arithmetic), which is
//!   the same machinery as the thing under test and so can only referee the
//!   TRUNCATION, not the decomposition;
//! * an exact statevector written HERE, `2^n` complex amplitudes evolved gate
//!   by gate, which shares no code with `magic5`, `affine` or `mesh` and so
//!   referees the decomposition itself.
//!
//! The plants carry the teeth: PQ-3 checks the bound is a BOUND (never below
//! the truth) and not a wild one (within 4× on a carrier with known tails),
//! PQ-4 plants a sign-flipped branch so that a passing certificate is shown to
//! be falsifiable, PQ-5 convicts a corrupted shard by name.
//!
//! S4 and S5 are timing and are `#[ignore]`d: `cargo test --release -p holon
//! -- --ignored --nocapture s4_` / `s5_`, on cores 21–27.

use holon::acuity::{
    self, budgeted_amplitude, budgeted_amplitude_sharded, convict_shards, merge_partials,
    price_line, BudgetPlan, Bounded, SignFlip,
};
use holon::ledger::Cyc;
use holon::magic::{cyc_eq, Circuit, Gate};
use holon::magic5::{expected_branches, Magic5Source};
use holon::mesh;
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

/// The prereg's instance family: `t` T-gates placed at random among
/// `n_clifford` Clifford gates over `{h, s, sdg, x, z, cx, t, tdg}`.
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

fn random_y(rng: &mut Rng, n: usize) -> Vec<bool> {
    (0..n).map(|_| rng.below(2) == 1).collect()
}

// ------------------------------------------------- the independent referee

/// `⟨y|C|0^n⟩` by DENSE STATEVECTOR: `2^n` complex amplitudes, evolved gate by
/// gate, sharing nothing with the decomposition under test — not `magic5`, not
/// `affine`, not `mesh`, not even the exact ring. Qubit `i` is bit `i` of the
/// index, which is `Affine::amplitude`'s convention.
fn statevector_amplitude(c: &Circuit, y: &[bool]) -> (f64, f64) {
    let n = c.n_qubits;
    let dim = 1usize << n;
    let mut re = vec![0.0f64; dim];
    let mut im = vec![0.0f64; dim];
    re[0] = 1.0;
    let s = std::f64::consts::FRAC_1_SQRT_2;
    for g in &c.gates {
        match *g {
            Gate::X(q) => {
                for x in 0..dim {
                    if x >> q & 1 == 0 {
                        let z = x | (1 << q);
                        re.swap(x, z);
                        im.swap(x, z);
                    }
                }
            }
            Gate::Z(q) => {
                for x in 0..dim {
                    if x >> q & 1 == 1 {
                        re[x] = -re[x];
                        im[x] = -im[x];
                    }
                }
            }
            Gate::S(q) | Gate::Sdg(q) => {
                let sgn = if matches!(*g, Gate::S(_)) { 1.0 } else { -1.0 };
                for x in 0..dim {
                    if x >> q & 1 == 1 {
                        let (a, b) = (re[x], im[x]);
                        // ±i · (a + ib)
                        re[x] = -sgn * b;
                        im[x] = sgn * a;
                    }
                }
            }
            Gate::T(q) | Gate::Tdg(q) => {
                let sgn = if matches!(*g, Gate::T(_)) { 1.0 } else { -1.0 };
                let (wr, wi) = (s, sgn * s); // ω^{±1} = (1 ± i)/√2
                for x in 0..dim {
                    if x >> q & 1 == 1 {
                        let (a, b) = (re[x], im[x]);
                        re[x] = a * wr - b * wi;
                        im[x] = a * wi + b * wr;
                    }
                }
            }
            Gate::H(q) => {
                for x in 0..dim {
                    if x >> q & 1 == 0 {
                        let z = x | (1 << q);
                        let (a0, b0, a1, b1) = (re[x], im[x], re[z], im[z]);
                        re[x] = (a0 + a1) * s;
                        im[x] = (b0 + b1) * s;
                        re[z] = (a0 - a1) * s;
                        im[z] = (b0 - b1) * s;
                    }
                }
            }
            Gate::Cx(ctrl, tgt) => {
                for x in 0..dim {
                    if x >> ctrl & 1 == 1 && x >> tgt & 1 == 0 {
                        let z = x | (1 << tgt);
                        re.swap(x, z);
                        im.swap(x, z);
                    }
                }
            }
        }
    }
    let idx: usize = y.iter().enumerate().fold(0, |a, (i, &b)| a | ((b as usize) << i));
    (re[idx], im[idx])
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// The f64 exit of an exact `Cyc` costs one rounding of coefficients that can
/// reach `2^{n+t}`; this is the slack allowed for THAT, and nothing else. The
/// exact claims in this file are made on `Cyc` structs, not on floats.
const F64_EXIT: f64 = 1e-9;

fn is_zero_cyc(x: Cyc) -> bool {
    x.c.iter().all(|&v| v == 0)
}

/// A `y` the circuit actually reaches.
///
/// M-PLANT-SECTOR, enforced rather than assumed: a plant on a branch whose
/// affine support misses `y` plants nothing, because that branch contributes
/// exactly zero and the sign of zero is zero. An amplitude that vanishes is a
/// perfectly good answer and a useless carrier, so the carrier is CHOSEN to be
/// nonzero and the choice is part of the test.
fn y_with_signal(rng: &mut Rng, n: usize, src: &Magic5Source) -> Vec<bool> {
    for _ in 0..256 {
        let y = random_y(rng, n);
        let (re, im) = mesh::fold_amplitude(src, &y, 1).to_complex();
        if re.hypot(im) > 1e-6 {
            return y;
        }
    }
    panic!("no y with a live amplitude in 256 draws — the carrier is the problem");
}

// ================================================================== S2

/// **S2 — the price is stated before the run.** `N_pred = expected_branches(t)`
/// is printed before any branch is evaluated, and the executed count at
/// `ε = 0` equals it EXACTLY. Kill: `N_exec ≠ N_pred`.
#[test]
fn s2_the_executed_count_at_eps_zero_is_the_price() {
    println!("# S2: the price, and what was spent against it");
    println!("{:>4} {:>8} {:>8} {:>8}", "t", "N_pred", "k(eps=0)", "N");
    for &t in &[8usize, 12, 16, 20] {
        let mut rng = Rng(0xACE1_7000 + t as u64);
        let n = 8;
        let c = random_circuit(&mut rng, n, 6 * n, t);
        let y = random_y(&mut rng, n);
        let n_pred = expected_branches(t);

        // The price line is stated from the T-count ALONE — no source built.
        assert_eq!(price_line(t), format!("price: t_eff={t} N_pred={n_pred}"));

        // Both bounds, because the executed count at ε = 0 must not depend on
        // which certificate the source carries.
        let src = acuity::source_for(&c, &y);
        let apriori = budgeted_amplitude(&src, &y, 0.0, 1);
        let certified = {
            let b = Bounded::new(acuity::source_for(&c, &y), src.scalar_bounds());
            budgeted_amplitude(&b, &y, 0.0, 4)
        };

        println!("{t:>4} {n_pred:>8} {:>8} {:>8}", apriori.evaluated, apriori.total);
        assert_eq!(apriori.total, n_pred, "S2 KILLED: N ≠ N_pred at t={t}");
        assert_eq!(apriori.evaluated, n_pred, "S2 KILLED: N_exec ≠ N_pred at t={t}");
        assert_eq!(certified.evaluated, n_pred, "S2 KILLED: N_exec ≠ N_pred at t={t}");
        assert_eq!(apriori.remainder, 0.0, "ε = 0 must leave nothing uncounted");
        assert_eq!(apriori.order, acuity::ORDER);

        // …and the full budgeted sum IS the mesh's full fold, bit for bit.
        let full = mesh::fold_amplitude(&src, &y, 1);
        assert_eq!(apriori.value, full, "the ε=0 budgeted sum is the full fold");
        assert_eq!(certified.value, full, "the ε=0 budgeted sum is the full fold");
    }
}

// ================================================================== S3

/// **S3 — the budgeted sum meets the acuity with a certificate.**
/// `|value − referee| ≤ R_k ≤ ε` on every instance, against BOTH referees
/// (the full exact branch sum, and the independent statevector), at every
/// `ε ∈ {10⁻¹, 10⁻², 10⁻³}`, for `n ∈ {12, 16}`, `t ∈ {8, 12, 16}`, three
/// seeds — and bit-identical under sharding.
#[test]
fn s3_the_certificate_holds_against_both_referees() {
    println!("# S3: |value − referee| ≤ R_k ≤ eps, and the executed fraction");
    println!(
        "{:>3} {:>3} {:>5} {:>8} {:>10} {:>6} {:>11} {:>11} {:>11}",
        "n", "t", "seed", "eps", "bound", "k/N", "R_k", "|v-full|", "|v-state|"
    );
    let mut worst_ratio: f64 = 0.0;
    for &n in &[12usize, 16] {
        for &t in &[8usize, 12, 16] {
            for seed in 0..3u64 {
                let mut rng = Rng(0xACE1_7300 ^ (n as u64) << 32 ^ (t as u64) << 16 ^ seed);
                let c = random_circuit(&mut rng, n, 20 * n, t);
                let y = random_y(&mut rng, n);

                // Referee 1: the independent statevector.
                let state = statevector_amplitude(&c, &y);
                // Referee 2: the FULL exact branch sum (ε = 0).
                let src = acuity::source_for(&c, &y);
                let full = budgeted_amplitude(&src, &y, 0.0, 1);
                assert_eq!(full.evaluated, expected_branches(t));
                assert!(
                    dist(full.value_f64, state) <= F64_EXIT,
                    "the decomposition disagrees with the statevector at n={n} t={t} seed={seed}: \
                     {:?} vs {:?}",
                    full.value_f64,
                    state
                );

                let certified = Bounded::new(acuity::source_for(&c, &y), src.scalar_bounds());
                let plan = BudgetPlan::of(&certified);
                let apriori_plan = BudgetPlan::of(&src);

                for &eps in &[1e-1f64, 1e-2, 1e-3] {
                    for (name, out) in [
                        ("scalar", acuity::budgeted_amplitude_with(&certified, &plan, &y, eps, 4)),
                        ("coeff", acuity::budgeted_amplitude_with(&src, &apriori_plan, &y, eps, 4)),
                    ] {
                        let d_full = dist(out.value_f64, full.value_f64);
                        let d_state = dist(out.value_f64, state);
                        println!(
                            "{n:>3} {t:>3} {seed:>5} {eps:>8.0e} {name:>10} {:>6.3} {:>11.3e} \
                             {:>11.3e} {:>11.3e}",
                            out.fraction(),
                            out.remainder,
                            d_full,
                            d_state
                        );
                        assert!(
                            out.remainder <= eps,
                            "S3 KILLED: R_k = {} > eps = {eps}",
                            out.remainder
                        );
                        assert!(
                            d_full <= out.remainder + F64_EXIT,
                            "S3 KILLED: |value − full sum| = {d_full} > R_k = {}",
                            out.remainder
                        );
                        assert!(
                            d_state <= out.remainder + F64_EXIT,
                            "S3 KILLED: |value − statevector| = {d_state} > R_k = {}",
                            out.remainder
                        );
                        if out.remainder > 0.0 && d_full > 0.0 {
                            worst_ratio = worst_ratio.max(out.remainder / d_full);
                        }
                        // The stopping decision is made on the deterministic
                        // prefix, so the answer is the same STRUCT at every S.
                        for s in [1usize, 2, 8] {
                            let alt = if name == "scalar" {
                                acuity::budgeted_amplitude_with(&certified, &plan, &y, eps, s)
                            } else {
                                acuity::budgeted_amplitude_with(&src, &apriori_plan, &y, eps, s)
                            };
                            assert_eq!(alt.evaluated, out.evaluated, "S changed the prefix");
                            assert_eq!(alt.value, out.value, "S changed the value at S={s}");
                        }
                    }
                }
            }
        }
    }
    println!("# worst R_k / |value − referee| over the sweep: {worst_ratio:.3e}");
}

// ================================================================== PQ-3

/// A synthetic branch set with KNOWN tails: branch `p` carries the exact value
/// `sign(p)·2^{−p}` and declares the bound `2^{−p}`. The carrier's sector is
/// nonzero by construction (every branch is nonzero), and the true remainder
/// is a geometric series computed in closed form here.
struct KnownTails {
    vals: Vec<Cyc>,
    bounds: Vec<f64>,
}

impl BranchSource for KnownTails {
    fn n_branches(&self) -> u64 {
        self.vals.len() as u64
    }
    fn amplitude_of(&self, b: u64, _y: &[bool]) -> Cyc {
        self.vals[b as usize]
    }
    fn n_qubits(&self) -> usize {
        1
    }
    fn branch_bound(&self, b: u64) -> f64 {
        self.bounds[b as usize]
    }
}

impl KnownTails {
    /// `alternating = false`: every branch aligned, so the triangle inequality
    /// is ATTAINED and `R_k` equals the true remainder exactly — the bound
    /// cannot be improved. `alternating = true`: signs alternate, so the true
    /// remainder is `⅔·2^{−k}` against `R_k = 2^{−k+1}` — a ratio of exactly
    /// 3, inside the plant's 4×.
    fn new(n: usize, alternating: bool) -> Self {
        let mut vals = Vec::with_capacity(n);
        let mut bounds = Vec::with_capacity(n);
        for p in 0..n {
            let sgn = if alternating && p % 2 == 1 { -1 } else { 1 };
            vals.push(Cyc { c: [sgn, 0, 0, 0], m: 2 * p as i32 });
            bounds.push(2f64.powi(-(p as i32)));
        }
        KnownTails { vals, bounds }
    }

    /// `|Σ_{p ≥ k} v_p|`, exact, by the same ring the engine uses.
    fn true_remainder(&self, k: usize) -> f64 {
        let acc = holon::merge::fold(self.vals[k..].iter().copied());
        let (re, im) = acc.to_complex();
        re.hypot(im)
    }
}

/// **PQ-3 — the remainder bound is a BOUND, and not a wild one.** `R_k` is
/// never below the true remainder at any `k`, and never above `4×` it.
#[test]
fn pq3_the_remainder_is_a_bound_and_within_four_times() {
    println!("# PQ-3: R_k against the true remainder on known tails");
    for &alternating in &[false, true] {
        let src = KnownTails::new(24, alternating);
        let plan = BudgetPlan::of(&src);
        // The declared order on a strictly decreasing bound is the identity.
        assert_eq!(plan.order, (0..24u64).collect::<Vec<_>>());
        let mut worst: f64 = 0.0;
        for k in 0..=24usize {
            let r = plan.remainder(k);
            let truth = src.true_remainder(k);
            assert!(
                r >= truth * (1.0 - 1e-12),
                "PQ-3 FAILED: R_{k} = {r} is BELOW the true remainder {truth} \
                 (alternating={alternating}) — the bound is an estimate"
            );
            if truth > 0.0 {
                let ratio = r / truth;
                worst = worst.max(ratio);
                assert!(
                    ratio <= 4.0,
                    "PQ-3 FAILED: R_{k}/truth = {ratio} > 4 (alternating={alternating})"
                );
            }
        }
        println!("#   alternating={alternating}: worst R_k/true remainder = {worst:.6}");
        assert!(
            (worst - if alternating { 3.0 } else { 1.0 }).abs() < 1e-6,
            "the closed form moved: worst ratio {worst}"
        );

        // …and the budget on this carrier certifies what it claims.
        let full = budgeted_amplitude(&src, &[false], 0.0, 1);
        for &eps in &[1e-1f64, 1e-3, 1e-6] {
            let out = budgeted_amplitude(&src, &[false], eps, 3);
            assert!(out.remainder <= eps);
            assert!(dist(out.value_f64, full.value_f64) <= out.remainder + F64_EXIT);
            assert!(out.evaluated < out.total, "eps={eps} must truncate this carrier");
        }
    }
}

// ================================================================== PQ-4

/// **PQ-4 — a planted wrong branch.** One term's sign is flipped; the `ε = 0`
/// sum then disagrees with the referee by more than `R_k` (which is `0` at
/// `ε = 0`), so the certificate is not vacuous. The carrier's sector is
/// nonzero by construction: the flipped branch's own amplitude is asserted
/// nonzero before the plant is read.
#[test]
fn pq4_a_planted_sign_flip_breaks_the_certificate() {
    println!("# PQ-4: the planted branch, the disagreement, and R_k");
    for &t in &[8usize, 12] {
        let mut rng = Rng(0xACE1_7400 + t as u64);
        let n = 10;
        let c = random_circuit(&mut rng, n, 20 * n, t);
        let raw = Magic5Source::new(&c);
        let y = y_with_signal(&mut rng, n, &raw);
        let src = Bounded::new(acuity::source_for(&c, &y), raw.scalar_bounds());
        let plan = BudgetPlan::of(&src);

        let clean = budgeted_amplitude(&src, &y, 0.0, 1);
        let state = statevector_amplitude(&c, &y);
        assert!(
            dist(clean.value_f64, state) <= F64_EXIT,
            "the clean sum must agree with the referee before a plant is read"
        );

        // The plant: the EARLIEST branch in the declared order that is live at
        // this `y` — so the carrier's sector is nonzero by construction
        // (M-PLANT-SECTOR) and the plant is inside every evaluated prefix.
        // A branch whose affine support misses `y` contributes exactly zero,
        // and flipping the sign of zero plants nothing.
        let Some((pos, planted_branch)) = plan
            .order
            .iter()
            .copied()
            .enumerate()
            .find(|&(_, b)| !is_zero_cyc(src.amplitude_of(b, &y)))
        else {
            panic!("PQ-4's carrier is empty: no branch is live at this y");
        };
        let term = src.amplitude_of(planted_branch, &y);
        let (tr, ti) = term.to_complex();
        assert!(tr.hypot(ti) > 1e-12, "PQ-4's carrier is empty at branch {planted_branch}");
        let flipped = SignFlip { inner: &src, branch: planted_branch };

        let out = budgeted_amplitude(&flipped, &y, 0.0, 4);
        let disagreement = dist(out.value_f64, state);
        println!(
            "#   t={t} branch={planted_branch} |2·term|={:.6e} disagreement={disagreement:.6e} \
             R_k={}",
            2.0 * tr.hypot(ti),
            out.remainder
        );
        assert_eq!(out.remainder, 0.0, "ε = 0 must certify a zero remainder");
        assert!(
            disagreement > out.remainder,
            "PQ-4 FAILED: the planted branch did not break the sum"
        );
        assert!(
            (disagreement - 2.0 * tr.hypot(ti)).abs() < 1e-9,
            "the plant moved the sum by something other than twice the term"
        );
        // Non-vacuity at a LIVE budget too: where the plant is inside the
        // evaluated prefix, the truncated sum is broken by more than the
        // remainder it certifies — the certificate is falsifiable, not just
        // arithmetic about an empty tail.
        for &eps in &[1e-2f64, 1e-3, 1e-4] {
            let k = plan.stop_at(eps);
            if pos >= k {
                continue;
            }
            let o = acuity::budgeted_amplitude_with(&flipped, &plan, &y, eps, 2);
            let d = dist(o.value_f64, state);
            println!("#   t={t} eps={eps:.0e} k={k} disagreement={d:.6e} R_k={:.6e}", o.remainder);
            assert!(
                d > o.remainder,
                "PQ-4 FAILED at eps={eps}: disagreement {d} ≤ R_k {}",
                o.remainder
            );
        }
    }
}

// ================================================================== PQ-5

/// **PQ-5 — the mesh.** A corrupted shard's partial is convicted BY NAME, and
/// a scrambled shard order changes nothing — not the value, not the struct.
#[test]
fn pq5_a_corrupted_shard_is_convicted_and_the_order_is_not() {
    let mut rng = Rng(0xACE1_7500);
    let (n, t) = (10usize, 12usize);
    let c = random_circuit(&mut rng, n, 20 * n, t);
    let y = y_with_signal(&mut rng, n, &Magic5Source::new(&c));
    let src = acuity::source_for(&c, &y);
    let plan = BudgetPlan::of(&src);

    for &shards in &[2usize, 4, 7] {
        let (budgeted, fold) = budgeted_amplitude_sharded(&src, &plan, &y, 0.0, shards);
        // The shard fold IS the mesh's fold, bit for bit.
        assert_eq!(
            fold.value,
            mesh::fold_amplitude(&src, &y, shards),
            "the shard-partial fold diverged from mesh::fold_amplitude"
        );
        assert_eq!(budgeted.value, fold.value);
        assert_eq!(fold.ranges.len(), shards.min(plan.order.len()));
        assert!(convict_shards(&src, &plan, &y, &fold).is_empty(), "a clean fold was convicted");

        // The corruption: one shard's ledger, off by one in the ring — on a
        // shard whose own partial is NONZERO, so the plant acts on a live
        // sector and not on an empty range (M-PLANT-SECTOR).
        assert!(!is_zero_cyc(fold.value), "PQ-5's carrier is empty: the fold is zero");
        let guilty = fold
            .partials
            .iter()
            .position(|p| !is_zero_cyc(*p))
            .expect("PQ-5's carrier is empty: every shard folds to zero");
        let mut bad = fold.clone();
        bad.partials[guilty] = bad.partials[guilty].add(Cyc::ONE);
        let convicted = convict_shards(&src, &plan, &y, &bad);
        assert_eq!(convicted.len(), 1, "exactly one shard is guilty");
        assert_eq!(convicted[0].shard, guilty, "the wrong shard was convicted");
        let named = format!("{}", convicted[0]);
        assert!(
            named.contains(&format!("shard {guilty} of {shards}")),
            "the conviction does not name the shard: {named}"
        );
        println!("# PQ-5 convicted: {named}");

        // The scramble: every rotation, and a reversal, of the shard order.
        for rot in 0..fold.partials.len() {
            let mut scrambled: Vec<Cyc> = fold.partials.clone();
            scrambled.rotate_left(rot);
            assert_eq!(
                merge_partials(&scrambled),
                fold.value,
                "rotating the shard order by {rot} changed the ledger"
            );
        }
        let mut reversed = fold.partials.clone();
        reversed.reverse();
        assert_eq!(merge_partials(&reversed), fold.value, "reversal changed the ledger");
    }
}

// ================================================================== S4

/// **S4 — cost follows the price, and the mesh pays where the work is.**
/// `#[ignore]`d: a wall-clock stake, run by hand on the pinned cores.
///
/// `taskset -c 21-27 cargo test --release -p holon -- --ignored --nocapture s4_`
#[test]
#[ignore = "timing: run by hand on cores 21-27"]
fn s4_the_mesh_pays_where_the_work_is() {
    // Two sizes, because a ratio at one size cannot tell a parallel section
    // that does not scale from a fixed cost that does not shrink: `S = 7` is
    // the number of cores actually pinned (21–27), and `S = 8` over-subscribes
    // them on purpose because the prereg names it.
    for &(n, t) in &[(20usize, 20usize), (24, 24)] {
        s4_one_instance(n, t);
    }
}

fn s4_one_instance(n: usize, t: usize) {
    let n_branches = expected_branches(t);
    assert!(n_branches >= 243, "S4 wants N ≥ 3^5; t={t} gives {n_branches}");
    let mut rng = Rng(0xACE1_7600 + t as u64);
    let c = random_circuit(&mut rng, n, 20 * n, t);
    let y = random_y(&mut rng, n);
    let src = acuity::source_for(&c, &y);
    let certified = Bounded::new(acuity::source_for(&c, &y), src.scalar_bounds());
    let plan = BudgetPlan::of(&src);
    let cplan = BudgetPlan::of(&certified);

    // THE MINIMUM, not the mean, is the arm's own cost: this box runs other
    // campaigns (the rule is cores 21–27, and `taskset` pins US there but
    // does not evict anyone else), so every sample is its own work plus
    // whatever the box stole. The minimum over 25 is the least-contaminated
    // estimate; the median is printed beside it so the contamination is
    // visible rather than averaged in.
    println!("# S4: the eps=0 fold, n={n} t={t} N={n_branches}, min & median of 25");
    let reps = 25;
    let mut base = f64::NAN;
    let mut first: Option<Cyc> = None;
    println!("{:>3} {:>12} {:>12} {:>10} {:>8}", "S", "min ms", "median ms", "ratio", "bits");
    for &s in &[1usize, 2, 4, 7, 8] {
        let mut walls = Vec::new();
        let mut value = Cyc::ZERO;
        for _ in 0..reps {
            let t0 = std::time::Instant::now();
            let out = acuity::budgeted_amplitude_with(&src, &plan, &y, 0.0, s);
            walls.push(t0.elapsed().as_secs_f64() * 1e3);
            value = out.value;
        }
        walls.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (w, med) = (walls[0], walls[walls.len() / 2]);
        if s == 1 {
            base = w;
            first = Some(value);
        }
        let same = first.map(|f| f == value).unwrap_or(false);
        println!(
            "{s:>3} {w:>12.4} {med:>12.4} {:>10.4} {:>8}",
            w / base,
            if same { "same" } else { "DIFF" }
        );
        assert!(same, "S4: sharding changed the ledger at S={s}");
    }

    println!("# S4: executed fraction against eps (k/N of N={n_branches})");
    println!("{:>10} {:>12} {:>12} {:>14} {:>14}", "eps", "k/N scalar", "k/N coeff", "R_k scalar", "R_k coeff");
    for &eps in &[0.0f64, 1e-1, 1e-2, 1e-3, 1e-4, 1e-6] {
        let a = acuity::budgeted_amplitude_with(&certified, &cplan, &y, eps, 1);
        let b = acuity::budgeted_amplitude_with(&src, &plan, &y, eps, 1);
        println!(
            "{eps:>10.0e} {:>12.4} {:>12.4} {:>14.4e} {:>14.4e}",
            a.fraction(),
            b.fraction(),
            a.remainder,
            b.remainder
        );
    }
}

// ================================================================== S5

/// **S5 — the comparison class.** The full sum's wall against `t`, fitted for
/// an exponent and reported against the published `2^{0.396 t}`. `#[ignore]`d.
///
/// `taskset -c 21-27 cargo test --release -p holon -- --ignored --nocapture s5_`
#[test]
#[ignore = "timing: run by hand on cores 21-27"]
fn s5_the_full_sum_against_the_published_exponent() {
    let n = 12usize;
    println!("# S5: the FULL exact branch sum's wall against t, n={n}, 20n Clifford gates");
    println!(
        "{:>4} {:>8} {:>12} {:>12} {:>12} {:>10}",
        "t", "N", "build ms", "fold ms", "total ms", "log2 tot"
    );
    let mut pts: Vec<(f64, f64, f64)> = Vec::new(); // (t, log2 total, log2 N)
    for t in (8..=28).step_by(2) {
        let mut rng = Rng(0xACE1_7700 + t as u64);
        let c = random_circuit(&mut rng, n, 20 * n, t);
        let y = random_y(&mut rng, n);
        let reps = if t >= 24 { 3 } else { 5 };
        let mut builds = Vec::new();
        let mut folds = Vec::new();
        let mut nb = 0u64;
        for _ in 0..reps {
            let t0 = std::time::Instant::now();
            let src = acuity::source_for(&c, &y);
            builds.push(t0.elapsed().as_secs_f64() * 1e3);
            let t1 = std::time::Instant::now();
            let out = budgeted_amplitude(&src, &y, 0.0, 1);
            folds.push(t1.elapsed().as_secs_f64() * 1e3);
            nb = out.total;
        }
        builds.sort_by(|a, b| a.partial_cmp(b).unwrap());
        folds.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (bw, fw) = (builds[reps / 2], folds[reps / 2]);
        let total = bw + fw;
        assert_eq!(nb, expected_branches(t));
        println!("{t:>4} {nb:>8} {bw:>12.3} {fw:>12.3} {total:>12.3} {:>10.4}", total.log2());
        pts.push((t as f64, total.log2(), (nb as f64).log2()));
    }
    let fit = |f: &dyn Fn(&(f64, f64, f64)) -> f64| -> f64 {
        let k = pts.len() as f64;
        let sx: f64 = pts.iter().map(|p| p.0).sum();
        let sy: f64 = pts.iter().map(f).sum();
        let sxx: f64 = pts.iter().map(|p| p.0 * p.0).sum();
        let sxy: f64 = pts.iter().map(|p| p.0 * f(p)).sum();
        (k * sxy - sx * sy) / (k * sxx - sx * sx)
    };
    let a_wall = fit(&|p| p.1);
    let a_branches = fit(&|p| p.2);
    // The third fit is the ACCOUNTING for the gap: the wall is the branch
    // count times the cost of ONE branch, and that cost is not constant in
    // `t` — the gadget widens the register to `n + t` qubits and adds `t` CX
    // gates, so a branch at `t = 28` is a bigger job than a branch at `t = 8`.
    // `a_wall ≈ a_branches + a_per_branch` is the identity to read.
    let a_per_branch = fit(&|p| p.1 - p.2);
    let published = 0.396_240_6f64;
    println!(
        "# S5 fitted exponent (wall)      = {a_wall:.4}  vs published 0.396, gap {:.4}",
        (a_wall - published).abs()
    );
    println!(
        "# S5 fitted exponent (branches)  = {a_branches:.4}  vs published 0.396, gap {:.4}",
        (a_branches - published).abs()
    );
    println!(
        "# S5 per-branch cost exponent    = {a_per_branch:.4}  (the whole gap:          {a_branches:.4} + {a_per_branch:.4} = {:.4})",
        a_branches + a_per_branch
    );
}

// ---------------------------------------------------------------- the seam

/// The API the cheap half calls, exercised end to end exactly as documented:
/// build the source from a located circuit, state the price, budget the sum.
#[test]
fn the_cheap_halfs_seam_is_what_it_says_it_is() {
    let mut rng = Rng(0xACE1_7800);
    let (n, t) = (8usize, 8usize);
    let c = random_circuit(&mut rng, n, 20 * n, t);
    let y = random_y(&mut rng, n);

    // 1. the price, from the T-count alone, before any source exists
    assert_eq!(price_line(t), format!("price: t_eff=8 N_pred={}", expected_branches(8)));
    // 2. the source, from the located circuit
    let src = acuity::certified_source_for(&c, &y);
    assert_eq!(src.n_branches(), expected_branches(t));
    // 3. the budgeted sum, folded across the mesh
    let out = budgeted_amplitude(&src, &y, 1e-3, 4);
    assert!(out.remainder <= 1e-3);
    assert_eq!(out.order, acuity::ORDER);
    let state = statevector_amplitude(&c, &y);
    assert!(dist(out.value_f64, state) <= out.remainder + F64_EXIT);
    // 4. the marginal: one plan, many y — the plan does not depend on y.
    let plan = BudgetPlan::of(&src);
    for basis in 0..16u32 {
        let mut yy = y.clone();
        for (q, b) in yy.iter_mut().enumerate().take(4) {
            *b = basis >> q & 1 == 1;
        }
        let o = acuity::budgeted_amplitude_with(&src, &plan, &yy, 1e-3, 4);
        let s = statevector_amplitude(&c, &yy);
        assert!(dist(o.value_f64, s) <= o.remainder + F64_EXIT, "marginal leg {basis}");
    }
    // 5. `cyc_eq` on the exact value, for a caller that wants the ring.
    let full = budgeted_amplitude(&src, &y, 0.0, 1);
    assert!(cyc_eq(full.value, mesh::fold_amplitude(&src, &y, 8)));
    // 6. THE OBJECT-SAFE CALL, pinned: the cheap half holds its backend behind
    //    a trait object, so `budgeted_amplitude` must take `&dyn BranchSource`
    //    as readily as `&impl BranchSource`. This line is the regression test
    //    for the `?Sized` bound; deleting it is how that bound gets lost.
    let obj: &dyn BranchSource = &src;
    let via_dyn = budgeted_amplitude(obj, &y, 0.0, 4);
    assert_eq!(via_dyn.value, full.value);
    assert_eq!(via_dyn.evaluated, full.evaluated);
    // 7. the PRICED call — the two lines the CLI prints, in order, on stderr.
    let priced = acuity::budgeted_amplitude_priced(&src, &y, 1e-3, 4, t);
    assert_eq!(priced.value, out.value);
    assert_eq!(priced.executed_line(), out.executed_line());
    assert!(priced.executed_line().starts_with(&format!("executed: k={} of N={}", out.evaluated, out.total)));
}
