//! Conformance and measurement for `holon::prune`.
//!
//! The referee is the CERTIFIED engine: `holon_qasm::magic`'s affine branch sum,
//! taken as a dev-dependency. Conformance is stated at the EXACT level — the
//! reference's `magic_amplitude` converts to `(f64,f64)` at the end, so this
//! harness re-runs its branch loop keeping `Cyc`, and compares ring elements.
//! An `assert_eq!` on `Cyc` would be wrong (the derived `PartialEq` calls the
//! two normalize-fixed-points of √2 different), so every comparison goes through
//! `prune::cyc_eq`, which subtracts and tests for zero.

use holon::ledger::Cyc as LCyc;
use holon::merge::{fold, MergeLedger};
use holon::prune::{self, Gate as PG, Mutations, PruneConfig};
use holon_qasm::magic::{Affine as RefAffine, Cyc as RefCyc};
use holon_qasm::Gate as QGate;

// ---------------------------------------------------------------- referee

fn to_qasm(g: PG) -> QGate {
    match g {
        PG::X(q) => QGate::X(q),
        PG::Z(q) => QGate::Z(q),
        PG::S(q) => QGate::S(q),
        PG::Sdg(q) => QGate::Sdg(q),
        PG::H(q) => QGate::H(q),
        PG::Cx(c, t) => QGate::Cx(c, t),
        PG::T(q) => QGate::T(q),
        PG::Tdg(q) => QGate::Tdg(q),
    }
}

fn conv(c: RefCyc) -> LCyc {
    LCyc { c: c.c, m: c.m }
}

/// The naive 2^t branch sum through the CERTIFIED reference, kept exact.
fn reference_state_vector(n: usize, gates: &[PG]) -> Vec<LCyc> {
    let t_count = gates.iter().filter(|g| g.is_t()).count();
    let dim = 1usize << n;
    let mut acc = vec![RefCyc::ZERO; dim];
    let mut y = vec![false; n];
    for branch in 0..(1usize << t_count) {
        let mut st = RefAffine::new(n);
        let mut coeff = RefCyc::ONE;
        let mut ti = 0usize;
        for &g in gates {
            match g {
                PG::T(q) | PG::Tdg(q) => {
                    let dag = matches!(g, PG::Tdg(_));
                    let z_branch = branch >> ti & 1 == 1;
                    ti += 1;
                    let (ci, cz) = if !dag {
                        (RefCyc { c: [1, 1, 0, 0], m: 2 }, RefCyc { c: [1, -1, 0, 0], m: 2 })
                    } else {
                        (RefCyc { c: [1, 0, 0, -1], m: 2 }, RefCyc { c: [1, 0, 0, 1], m: 2 })
                    };
                    if z_branch {
                        coeff = coeff.mul(cz);
                        st.apply(QGate::Z(q));
                    } else {
                        coeff = coeff.mul(ci);
                    }
                }
                other => st.apply(to_qasm(other)),
            }
        }
        for (idx, slot) in acc.iter_mut().enumerate() {
            for (q, yq) in y.iter_mut().enumerate() {
                *yq = idx >> q & 1 == 1;
            }
            let a = st.amplitude(&y);
            if a != RefCyc::ZERO {
                *slot = slot.add_fixed(coeff.mul(a));
            }
        }
    }
    acc.into_iter().map(conv).collect()
}

/// `true` iff the pruned sum reproduces the reference EXACTLY on every basis
/// state. Returns the first disagreeing index for reporting.
fn conforms(n: usize, gates: &[PG], cfg: &PruneConfig) -> Result<prune::PruneStats, usize> {
    let want = reference_state_vector(n, gates);
    let got = prune::run_pruned(n, gates, cfg);
    let sv = got.state_vector();
    for idx in 0..want.len() {
        if !prune::cyc_eq(sv[idx], want[idx]) {
            return Err(idx);
        }
    }
    Ok(got.stats)
}

// ---------------------------------------------------------------- circuits

/// xorshift64* — deterministic, zero deps, so every number in this file is
/// reproducible from its seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// Class (a): random Clifford+T. `t` T-gates dropped at random positions into a
/// random Clifford body of length `clifford_len`.
fn random_clifford_t(rng: &mut Rng, n: usize, clifford_len: usize, t: usize) -> Vec<PG> {
    let mut body: Vec<PG> = Vec::with_capacity(clifford_len + t);
    for _ in 0..clifford_len {
        let q = rng.below(n);
        body.push(match rng.below(6) {
            0 => PG::H(q),
            1 => PG::S(q),
            2 => PG::Sdg(q),
            3 => PG::X(q),
            4 => PG::Z(q),
            _ => {
                if n < 2 {
                    PG::H(q)
                } else {
                    let mut c = rng.below(n);
                    if c == q {
                        c = (q + 1) % n;
                    }
                    PG::Cx(c, q)
                }
            }
        });
    }
    for _ in 0..t {
        let pos = rng.below(body.len() + 1);
        let q = rng.below(n);
        let g = if rng.below(2) == 0 { PG::T(q) } else { PG::Tdg(q) };
        body.insert(pos, g);
    }
    body
}

/// Class (b): `pairs` adjacent T;T pairs on one qubit with NO intervening gate.
/// T² = S exactly, so each pair is one Clifford wearing two T's — a collapse the
/// merge machinery has to DISCOVER, never being told about it.
fn adjacent_tt(rng: &mut Rng, n: usize, clifford_len: usize, pairs: usize) -> Vec<PG> {
    let mut out: Vec<PG> = Vec::new();
    for _ in 0..clifford_len {
        let q = rng.below(n);
        out.push(match rng.below(4) {
            0 => PG::H(q),
            1 => PG::S(q),
            2 => PG::X(q),
            _ => {
                if n < 2 {
                    PG::H(q)
                } else {
                    let mut c = rng.below(n);
                    if c == q {
                        c = (q + 1) % n;
                    }
                    PG::Cx(c, q)
                }
            }
        });
    }
    for _ in 0..pairs {
        let q = rng.below(n);
        let pos = rng.below(out.len() + 1);
        out.insert(pos, PG::T(q));
        out.insert(pos, PG::T(q));
    }
    out
}

/// Class (c): hidden-shift-LIKE. `H^n`, a diagonal Clifford+T oracle conjugated
/// by the shift, `H^n`, a second diagonal oracle, `H^n`. Labelled "-like" on
/// purpose: it wears the hidden-shift shape (bent-function-ish CZ pairing plus a
/// diagonal T layer, sandwiched by Hadamards) without being a certified
/// hidden-shift instance.
fn hidden_shift_like(rng: &mut Rng, n: usize, t_per_layer: usize) -> Vec<PG> {
    assert!(n.is_multiple_of(2));
    let shift: Vec<bool> = (0..n).map(|_| rng.below(2) == 1).collect();
    let mut g: Vec<PG> = Vec::new();
    for q in 0..n {
        g.push(PG::H(q));
    }
    for (q, &s) in shift.iter().enumerate() {
        if s {
            g.push(PG::X(q));
        }
    }
    for i in 0..n / 2 {
        prune::push_cz(&mut g, 2 * i, 2 * i + 1);
    }
    for j in 0..t_per_layer {
        g.push(PG::T(j % n));
    }
    for (q, &s) in shift.iter().enumerate() {
        if s {
            g.push(PG::X(q));
        }
    }
    for q in 0..n {
        g.push(PG::H(q));
    }
    for i in 0..n / 2 {
        prune::push_cz(&mut g, 2 * i, 2 * i + 1);
    }
    for j in 0..t_per_layer {
        g.push(PG::Tdg(j % n));
    }
    for q in 0..n {
        g.push(PG::H(q));
    }
    g
}

// ---------------------------------------------------------------- conformance

#[test]
fn clifford_only_conformance() {
    let mut rng = Rng(0x51ff_c0de_0001);
    for _ in 0..40 {
        let n = 2 + rng.below(4);
        let c = random_clifford_t(&mut rng, n, 14, 0);
        let stats = conforms(n, &c, &PruneConfig::default())
            .unwrap_or_else(|i| panic!("clifford-only mismatch at basis state {i}"));
        assert_eq!(stats.naive_branches, 1);
        assert_eq!(stats.hash_collisions_rejected, 0);
        assert_eq!(stats.verify_rejections, 0);
    }
}

/// The headline bar: 80 random Clifford+T circuits, pruned sum EXACTLY equal to
/// the certified reference's naive sum on every basis state.
#[test]
fn random_clifford_t_conformance() {
    let mut rng = Rng(0x51ff_c0de_0002);
    let cfg = PruneConfig { verify_points: usize::MAX, ..PruneConfig::default() };
    let mut total_merged = 0usize;
    let mut total_zero = 0usize;
    for case in 0..80 {
        let n = 2 + rng.below(4); // 2..=5
        let t = 1 + rng.below(8); // 1..=8
        let c = random_clifford_t(&mut rng, n, 12, t);
        let stats = conforms(n, &c, &cfg)
            .unwrap_or_else(|i| panic!("case {case} (n={n}, t={t}) mismatch at basis state {i}"));
        assert_eq!(stats.hash_collisions_rejected, 0, "case {case}: hash collision");
        assert_eq!(stats.verify_rejections, 0, "case {case}: verifier rejected a key match");
        assert!(stats.final_branches as u128 <= stats.naive_branches);
        total_merged += stats.merged_duplicates;
        total_zero += stats.zero_states_dropped;
    }
    println!(
        "random Clifford+T conformance: 80/80 exact; {total_merged} duplicates merged, \
         {total_zero} annihilated branches dropped across the sweep"
    );
}

/// Same 80-circuit bar with a LARGER block: merging every 3 T-gates instead of
/// every one. Block size must not change the answer, only the working set.
#[test]
fn block_size_invariance() {
    let mut rng = Rng(0x51ff_c0de_0003);
    for case in 0..30 {
        let n = 2 + rng.below(3);
        let t = 2 + rng.below(6);
        let c = random_clifford_t(&mut rng, n, 12, t);
        let want = reference_state_vector(n, &c);
        for block in [1usize, 2, 3, 64] {
            let cfg = PruneConfig { merge_every: block, ..PruneConfig::default() };
            let sv = prune::run_pruned(n, &c, &cfg).state_vector();
            for idx in 0..want.len() {
                assert!(
                    prune::cyc_eq(sv[idx], want[idx]),
                    "case {case}, block {block}: mismatch at basis state {idx}"
                );
            }
        }
    }
}

/// The pruned sum and the merge-disabled control must agree exactly — the merge
/// is the only difference between them, so this isolates it from the port.
#[test]
fn merge_matches_its_own_control() {
    let mut rng = Rng(0x51ff_c0de_0004);
    for _ in 0..30 {
        let n = 2 + rng.below(3);
        let c = adjacent_tt(&mut rng, n, 10, 3);
        let pruned = prune::run_pruned(n, &c, &PruneConfig::default()).state_vector();
        let naive = prune::run_naive(n, &c).state_vector();
        for idx in 0..pruned.len() {
            assert!(prune::cyc_eq(pruned[idx], naive[idx]), "merge changed the sum at {idx}");
        }
    }
}

// ---------------------------------------------------------------- the T;T case

/// `T;T` on one qubit with nothing between is `S` exactly. Nothing in
/// `prune.rs` knows that identity; the merge has to find it.
///
/// The exact rank, derived rather than guessed: `p` adjacent pairs on one qubit
/// is `T^{2p} = S^p = diag(1, i^p) = a·I + b·Z` with `a = (1+i^p)/2` and
/// `b = (1−i^p)/2`. So off a Z-eigenstate the 4^p naive branches collapse to
///
/// * `2` when `p` is ODD (`S`, `S³` — both `a` and `b` nonzero), and
/// * `1` when `p` is EVEN (`S² = Z` has `a = 0`; `S⁴ = I` has `b = 0`).
///
/// The even case is the interesting one, and it was not the number this test
/// first asserted: the duplicate merge alone gives 2, and the surviving branch
/// is removed by the merged weight being EXACTLY zero. That branch is worth
/// ~1e-16 in floating point and would live forever. Only an exact ledger drops
/// it, which is the whole thesis of the module.
#[test]
fn adjacent_tt_collapses() {
    for p in 1..=4usize {
        let mut g = vec![PG::H(0), PG::H(1), PG::Cx(0, 1)];
        for _ in 0..p {
            g.push(PG::T(0));
            g.push(PG::T(0));
        }
        let stats = conforms(2, &g, &PruneConfig::default())
            .unwrap_or_else(|i| panic!("T;T x{p} mismatch at {i}"));
        assert_eq!(stats.naive_branches, 1u128 << (2 * p));
        let want = if p % 2 == 1 { 2 } else { 1 };
        assert_eq!(
            stats.final_branches, want,
            "T;T x{p} (4^{p} = {} naive): expected rank {want}",
            stats.naive_branches
        );
        assert!(
            stats.merged_duplicates > 0,
            "T;T x{p}: the duplicate merge did no work"
        );
        if p % 2 == 0 {
            assert!(
                stats.exact_cancellations > 0,
                "T;T x{p}: S^{p} is a single Pauli, so a merged weight must vanish EXACTLY"
            );
        }
    }

    // On a Z-EIGENSTATE the collapse is total: Z|φ⟩ = |φ⟩ up to a scalar, so
    // every pair folds 4 → 1 however many T's the circuit carries.
    for p in 1..=6usize {
        let mut g = vec![PG::X(0)];
        for _ in 0..p {
            g.push(PG::T(0));
            g.push(PG::T(0));
        }
        let stats = conforms(2, &g, &PruneConfig::default())
            .unwrap_or_else(|i| panic!("Z-eigenstate T;T x{p} mismatch at {i}"));
        assert_eq!(stats.final_branches, 1, "Z-eigenstate T;T x{p} should be rank 1");
        assert_eq!(stats.naive_branches, 1u128 << (2 * p));
    }

    // The control: two T's that share no qubit and no entanglement. The four
    // branches are (I,Z)⊗(I,Z) on |++⟩ — four genuinely distinct states — so a
    // merge here would mean the machinery merges things it must not.
    //
    // Two tempting controls are NOT controls, and both were tried here first:
    // `H,T,H,T` on one qubit collapses to 2 (the middle H sends the branch
    // states to |0⟩/|1⟩, and the second T then acts on a Z-EIGENSTATE, so it
    // does not split the state at all), and `H,T,S,T` collapses too
    // (`S|−⟩ = ZS|+⟩`). "An intervening gate" is not what stops a collapse.
    let g = vec![PG::H(0), PG::H(1), PG::T(0), PG::T(1)];
    let stats = conforms(2, &g, &PruneConfig::default()).expect("control must conform");
    assert_eq!(stats.naive_branches, 4);
    assert_eq!(
        stats.final_branches, 4,
        "four distinct product states must not merge"
    );
    assert_eq!(stats.merged_duplicates, 0);
    assert_eq!(stats.exact_cancellations, 0);
}

// ---------------------------------------------------------------- planted defects

/// A wrong phase ratio at the merge itself (`w₂ · i` instead of `w₂`). The
/// states really are equal, so no structural check can see it — conformance is
/// the only thing that can, and it must.
#[test]
fn planted_wrong_merge_phase_fires() {
    let cfg = PruneConfig {
        mutations: Mutations { merge_phase: true, ..Mutations::default() },
        ..PruneConfig::default()
    };
    let g = vec![PG::H(0), PG::H(1), PG::Cx(0, 1), PG::T(0), PG::T(0)];
    // sanity: the clean build passes this circuit and DOES merge
    let clean = conforms(2, &g, &PruneConfig::default()).expect("clean build must conform");
    assert!(clean.merged_duplicates > 0, "the gauge needs a circuit that actually merges");
    assert!(
        conforms(2, &g, &cfg).is_err(),
        "planted wrong-phase-ratio merge did NOT fire conformance"
    );
}

/// A wrong extracted global scalar (`flip` drops its `i^{d_p}`). This is the
/// "phase ratio" defect at its source: the ratio λ = γ₂/γ₁ is carried by the
/// stripped γ, so corrupting the strip corrupts every merge that follows.
#[test]
fn planted_dropped_gamma_fires() {
    let cfg = PruneConfig {
        mutations: Mutations { flip_drops_gamma: true, ..Mutations::default() },
        ..PruneConfig::default()
    };
    let mut rng = Rng(0x51ff_c0de_0005);
    let mut fired = 0;
    let mut tried = 0;
    for _ in 0..40 {
        let n = 2 + rng.below(3);
        let t = 1 + rng.below(4);
        let c = random_clifford_t(&mut rng, n, 12, t);
        if conforms(n, &c, &PruneConfig::default()).is_err() {
            panic!("clean build failed its own gauge circuit");
        }
        tried += 1;
        if conforms(n, &c, &cfg).is_err() {
            fired += 1;
        }
    }
    assert!(fired > 0, "planted dropped-gamma never fired across {tried} circuits");
    println!("planted flip_drops_gamma: fired on {fired}/{tried} circuits");
}

/// "Never trust the hash alone." With `h` dropped from the canonical key, states
/// on DIFFERENT cosets compare equal — and the exact amplitude cross-check is
/// the only thing left standing between that and a wrong answer. It must catch
/// every one: with verification ON the merges are rejected and the answer stays
/// exact; with verification OFF the answer breaks.
#[test]
fn key_ignoring_h_is_caught_by_amplitude_check() {
    let mut rng = Rng(0x51ff_c0de_0006);
    let guarded = PruneConfig {
        verify_points: usize::MAX,
        mutations: Mutations { key_ignores_h: true, ..Mutations::default() },
        ..PruneConfig::default()
    };
    let unguarded = PruneConfig {
        mutations: Mutations {
            key_ignores_h: true,
            skip_verify: true,
            ..Mutations::default()
        },
        ..PruneConfig::default()
    };
    let mut rejections = 0usize;
    let mut unguarded_breaks = 0usize;
    let mut guarded_breaks = 0usize;
    for _ in 0..60 {
        let n = 2 + rng.below(3);
        let t = 2 + rng.below(5);
        let c = random_clifford_t(&mut rng, n, 12, t);
        match conforms(n, &c, &guarded) {
            Ok(s) => rejections += s.verify_rejections,
            Err(_) => guarded_breaks += 1,
        }
        if conforms(n, &c, &unguarded).is_err() {
            unguarded_breaks += 1;
        }
    }
    assert!(
        rejections > 0,
        "the h-blind key never produced a false match — the gauge proves nothing"
    );
    assert_eq!(
        guarded_breaks, 0,
        "the amplitude cross-check let {guarded_breaks} bad merges through"
    );
    assert!(
        unguarded_breaks > 0,
        "with the cross-check OFF the h-blind key should have broken the answer"
    );
    println!(
        "hash-alone gauge: {rejections} false key-matches caught by the amplitude check; \
         with the check disabled, {unguarded_breaks}/60 circuits gave a WRONG answer"
    );
}

// ---------------------------------------------------------------- measurement

/// One measured cell: a circuit class at a fixed `(n, t)`.
struct Cell {
    n: usize,
    t: usize,
    runs: usize,
    naive: u128,
    /// `2^min(t,n)` — the Pauli-orbit ceiling `pauli_orbit_bound_holds` checks.
    ceiling: usize,
    finals: Vec<usize>,
    merged: usize,
    zeros: usize,
    cancels: usize,
}

impl Cell {
    fn mean_final(&self) -> f64 {
        self.finals.iter().sum::<usize>() as f64 / self.runs as f64
    }
    fn max_final(&self) -> usize {
        *self.finals.iter().max().unwrap()
    }
    fn survival(&self) -> f64 {
        self.mean_final() / self.naive as f64
    }
    fn row(&self) -> String {
        format!(
            "  n={:<2} t={:<3} 2^t={:<7} ceiling={:<5} | mean {:>7.1}  max {:<5} | \
survival {:>8.4}%  speedup {:>9.1}x | merged {:<6} zero {:<3} cancel {:<5}",
            self.n,
            self.t,
            self.naive,
            self.ceiling,
            self.mean_final(),
            self.max_final(),
            100.0 * self.survival(),
            self.naive as f64 / self.mean_final().max(1.0),
            self.merged,
            self.zeros,
            self.cancels,
        )
    }
}

fn measure_cell(
    label_seed: u64,
    n: usize,
    t: usize,
    runs: usize,
    mut build: impl FnMut(&mut Rng, usize, usize) -> Vec<PG>,
) -> Cell {
    let mut rng = Rng(label_seed);
    let mut cell = Cell {
        n,
        t,
        runs,
        naive: 1u128 << t,
        ceiling: 1usize << n.min(t),
        finals: Vec::new(),
        merged: 0,
        zeros: 0,
        cancels: 0,
    };
    for _ in 0..runs {
        let c = build(&mut rng, n, t);
        assert_eq!(c.iter().filter(|g| g.is_t()).count(), t, "builder missed its T-count");
        let s = prune::run_pruned(n, &c, &PruneConfig::default()).stats;
        assert_eq!(s.hash_collisions_rejected, 0);
        assert_eq!(s.verify_rejections, 0);
        assert!(s.final_branches <= cell.ceiling, "n={n} t={t}: ceiling breached");
        cell.finals.push(s.final_branches);
        cell.merged += s.merged_duplicates;
        cell.zeros += s.zero_states_dropped;
        cell.cancels += s.exact_cancellations;
    }
    cell
}

/// T's placed only AFTER a full scrambling Clifford layer, so no T ever lands on
/// a Z-eigenstate. This is the pessimistic random case — placing T's uniformly
/// at random puts some of them near the start, where the state is still close to
/// |0…0⟩ and the T does not split the state at all.
fn scrambled_then_t(rng: &mut Rng, n: usize, t: usize) -> Vec<PG> {
    let mut g = random_clifford_t(rng, n, 6 * n, 0);
    for _ in 0..t {
        g.push(if rng.below(2) == 0 { PG::T(rng.below(n)) } else { PG::Tdg(rng.below(n)) });
        for _ in 0..3 {
            let mut layer = random_clifford_t(rng, n, 1, 0);
            g.append(&mut layer);
        }
    }
    g
}

/// The deliverable: measured branch counts against the naive `2^t`, per circuit
/// class and per `(n, t)` cell. Honest numbers, and the ceiling they run into is
/// printed beside them so the rates cannot be read as more than they are.
///
/// `cargo test -p holon --release --test prune -- --nocapture measure_prune_rates`
#[test]
fn measure_prune_rates() {
    println!();
    println!("=== EXACT BRANCH PRUNING — achieved branch counts vs naive 2^t ===");
    println!("ceiling = 2^min(t,n), the Pauli-orbit bound (see pauli_orbit_bound_holds)");

    println!("\n(a) random Clifford+T, T's at uniformly random positions:");
    for &(n, t) in &[(4, 8), (4, 12), (6, 8), (6, 12), (8, 8), (8, 12)] {
        let c = measure_cell(0xbeef_0001 + (n * 100 + t) as u64, n, t, 60, |r, n, t| {
            random_clifford_t(r, n, 4 * n, t)
        });
        println!("{}", c.row());
    }

    println!("\n(a') random Clifford+T, every T after a scrambling layer (pessimistic):");
    for &(n, t) in &[(4, 8), (4, 12), (6, 8), (6, 12), (8, 8), (8, 12)] {
        let c = measure_cell(0xbeef_0100 + (n * 100 + t) as u64, n, t, 60, scrambled_then_t);
        println!("{}", c.row());
    }

    println!("\n(b) adjacent T;T pairs (T² = S — the exact collapse, discovered not told):");
    for &(n, pairs) in &[(3, 2), (3, 4), (4, 4), (4, 6), (6, 6)] {
        let c = measure_cell(0xbeef_1000 + (n * 100 + pairs) as u64, n, 2 * pairs, 60, |r, n, t| {
            adjacent_tt(r, n, 3 * n, t / 2)
        });
        println!("{}", c.row());
    }

    println!("\n(c) hidden-shift-like (H^n · diagonal Clifford+T oracle · H^n · oracle · H^n):");
    for &(n, tpl) in &[(4, 3), (4, 5), (6, 4), (6, 6)] {
        let c = measure_cell(0xbeef_2000 + (n * 100 + tpl) as u64, n, 2 * tpl, 40, |r, n, t| {
            hidden_shift_like(r, n, t / 2)
        });
        println!("{}", c.row());
    }

    println!("\n(d) T-heavy at small n — where the ceiling is the whole story:");
    for &(n, t) in &[(2, 16), (3, 16), (4, 16)] {
        let c = measure_cell(0xbeef_3000 + (n * 100 + t) as u64, n, t, 20, scrambled_then_t);
        println!("{}", c.row());
    }

    // Working-set check: with merge_every = 1 the peak never exceeds 2x the
    // largest merged set, which is what makes the breadth-first walk affordable.
    let mut rng = Rng(0xbeef_4444);
    let mut worst_ratio = 0.0f64;
    let mut total_zero_states = 0usize;
    for _ in 0..60 {
        let n = 4 + rng.below(3);
        let c = scrambled_then_t(&mut rng, n, 12);
        let s = prune::run_pruned(n, &c, &PruneConfig::default()).stats;
        let max_after = s.blocks.iter().map(|b| b.2).max().unwrap_or(1);
        worst_ratio = worst_ratio.max(s.peak_working_set as f64 / max_after.max(1) as f64);
        total_zero_states += s.zero_states_dropped;
    }
    println!("\npeak working set / largest merged set, worst over 60 runs: {worst_ratio:.2}x");
    println!(
        "annihilated-branch drops across every class above: {total_zero_states} \
(optimization 1 is structurally unreachable here — a branch is a Clifford orbit \
of |0..0> followed by Paulis, so it is always a normalized state)"
    );
    println!();
}

/// A slower sweep that also CONFORMS every measured structured circuit against
/// the reference, so the reported structured rates are not just fast, they are
/// right. Kept separate because the reference costs 2^t · 2^n.
#[test]
fn structured_rates_conform() {
    let mut rng = Rng(0xbeef_4000);
    let cfg = PruneConfig { verify_points: usize::MAX, ..PruneConfig::default() };
    let mut checked = 0;
    for pairs in [1usize, 2, 3] {
        for _ in 0..12 {
            let n = 2 + rng.below(3);
            let c = adjacent_tt(&mut rng, n, 8, pairs);
            let s = conforms(n, &c, &cfg)
                .unwrap_or_else(|i| panic!("T;T x{pairs} mismatch at basis state {i}"));
            assert!(
                (s.final_branches as u128) <= s.naive_branches,
                "pruning cannot increase the branch count"
            );
            checked += 1;
        }
    }
    for t_per_layer in [1usize, 2, 3] {
        for _ in 0..6 {
            let c = hidden_shift_like(&mut rng, 4, t_per_layer);
            conforms(4, &c, &cfg)
                .unwrap_or_else(|i| panic!("hidden-shift-like mismatch at basis state {i}"));
            checked += 1;
        }
    }
    println!("structured circuits conformed against the reference: {checked}");
}

/// The structural reason the rates below are what they are, checked rather than
/// asserted in prose.
///
/// Every branch differs from every other by a product of the `Z`'s the
/// T-expansion inserted. Pushing those `Z`'s forward through the remaining
/// Clifford gates turns each into a PAULI, so branch `b`'s state is `P_b|ψ⟩` for
/// one common stabilizer state `|ψ⟩`. Two Paulis give the same state up to a
/// scalar exactly when they differ by an element of `|ψ⟩`'s stabilizer group, so
/// the whole branch set lives in a Pauli orbit of at most `2^n` states — and
/// `b ↦ P_b` mod phase is a homomorphism from `F₂^t`, so the count is `2^r` with
/// `r ≤ min(t, n)`.
///
/// The merge therefore CANNOT return more than `2^min(t,n)` branches, whatever
/// the T-count. That is a hard ceiling this test checks on every circuit class,
/// and it is the honest explanation of the random-circuit rate: the pruning is
/// not finding lucky coincidences, it is discovering that the magic tier's
/// branch set was never bigger than the qubit count allows.
#[test]
fn pauli_orbit_bound_holds() {
    let mut rng = Rng(0xb0_u64.wrapping_mul(0x9e37_79b9));
    let mut tight = 0usize;
    let mut total = 0usize;
    for _ in 0..200 {
        let n = 2 + rng.below(6); // 2..=7
        let t = 1 + rng.below(12); // 1..=12
        let c = random_clifford_t(&mut rng, n, 4 * n, t);
        let s = prune::run_pruned(n, &c, &PruneConfig::default()).stats;
        let bound = 1usize << n.min(t);
        assert!(
            s.final_branches <= bound,
            "n={n} t={t}: {} branches survived a 2^min(t,n) = {bound} ceiling",
            s.final_branches
        );
        if s.final_branches == bound {
            tight += 1;
        }
        total += 1;
    }
    println!("Pauli-orbit ceiling 2^min(t,n): held on {total}/{total}, attained on {tight}");
}

// ---------------------------------------------------------------- exact invariants

/// Complex conjugate in Z[ω]: `conj(ω^k) = ω^{8−k}` and `ω⁴ = −1`, so
/// `[c0,c1,c2,c3] ↦ [c0,−c3,−c2,−c1]`.
fn conj(x: LCyc) -> LCyc {
    LCyc { c: [x.c[0], -x.c[3], -x.c[2], -x.c[1]], m: x.m }
}

/// `Σ_y |a_y|²` as a ring element. On a correct branch sum this is EXACTLY
/// `Cyc::ONE` — not 1 ± 1e-15.
///
/// It is a NECESSARY condition and nothing more. It was originally written here
/// as the certificate for the deep cells, on the reasoning that it consults no
/// reference and so scales where the naive `2^t` reference does not. Gauging it
/// killed that use: `defect_visibility_matrix` measures it blind to 0/33 of the
/// `flip_drops_gamma` defects that conformance catches, and blind to more than
/// half of the others. It is kept as a cheap always-on sanity check, and the
/// deep cells are certified by `deepest_measured_cells_conform` instead — which
/// simply pays the 2 seconds the reference costs at t=16.
fn exact_norm(sv: &[LCyc]) -> LCyc {
    fold(sv.iter().map(|&a| a.mul(conj(a))))
}

/// The pruned sum is a STATE: its probabilities sum to exactly one, in the ring,
/// across every measured circuit class at the sizes the rate table reports. A
/// necessary condition, not a certificate — see `exact_norm`.
#[test]
fn pruned_sum_is_exactly_normalized() {
    let mut rng = Rng(0x4e0f_3a91_0001);
    let mut checked = 0usize;
    for &(n, t) in &[(4usize, 12usize), (6, 12), (8, 12), (3, 16), (4, 16)] {
        for _ in 0..8 {
            let c = scrambled_then_t(&mut rng, n, t);
            let sv = prune::run_pruned(n, &c, &PruneConfig::default()).state_vector();
            assert!(
                prune::cyc_eq(exact_norm(&sv), LCyc::ONE),
                "random n={n} t={t}: Σ|a|² is not exactly 1"
            );
            checked += 1;
        }
    }
    for &(n, pairs) in &[(4usize, 6usize), (6, 6)] {
        for _ in 0..8 {
            let c = adjacent_tt(&mut rng, n, 3 * n, pairs);
            let sv = prune::run_pruned(n, &c, &PruneConfig::default()).state_vector();
            assert!(prune::cyc_eq(exact_norm(&sv), LCyc::ONE), "T;T n={n}: Σ|a|² ≠ 1");
            checked += 1;
        }
    }
    for &(n, tpl) in &[(4usize, 5usize), (6, 6)] {
        for _ in 0..8 {
            let c = hidden_shift_like(&mut rng, n, tpl);
            let sv = prune::run_pruned(n, &c, &PruneConfig::default()).state_vector();
            assert!(prune::cyc_eq(exact_norm(&sv), LCyc::ONE), "hidden-shift n={n}: Σ|a|² ≠ 1");
            checked += 1;
        }
    }
    println!("exact normalization Σ|a|² = 1 (in the ring): {checked}/{checked} circuits");
}

/// Conformance in the regime the rate table actually reports, where the naive
/// reference costs 2^t · 2^n. Smaller sample than the 80-circuit sweep because
/// each case is ~10^5 reference amplitude solves, and pointless to skip because
/// the small-t sweep cannot see a defect that only appears deep in a merge
/// chain.
#[test]
fn heavy_conformance() {
    let mut rng = Rng(0xdee9_c0de_0002);
    let cfg = PruneConfig { verify_points: usize::MAX, ..PruneConfig::default() };
    let mut checked = 0usize;
    for &(n, t) in &[(4usize, 10usize), (4, 12), (5, 10), (3, 14)] {
        for case in 0..5 {
            let c = scrambled_then_t(&mut rng, n, t);
            conforms(n, &c, &cfg).unwrap_or_else(|i| {
                panic!("heavy case {case} (n={n}, t={t}) mismatch at basis state {i}")
            });
            checked += 1;
        }
    }
    for &(n, pairs) in &[(4usize, 5usize), (3, 6)] {
        for _ in 0..5 {
            let c = adjacent_tt(&mut rng, n, 3 * n, pairs);
            conforms(n, &c, &cfg)
                .unwrap_or_else(|i| panic!("heavy T;T (n={n}) mismatch at {i}"));
            checked += 1;
        }
    }
    println!("heavy conformance (t up to 14) against the certified reference: {checked}/{checked}");
}

/// What each instrument can actually SEE, measured rather than assumed.
///
/// The row that matters is `flip_drops_gamma`: it changes the answer on 33 of 40
/// circuits, conformance catches all 33, and the exact-normalization invariant
/// catches ZERO of them. A per-branch unit phase error can move probability
/// between amplitudes and still leave `Σ|a|²` exactly 1, so normalization is not
/// a certificate for this module and is not used as one.
///
/// The bar asserted here: conformance against the certified reference catches
/// EVERY defect that changes the state — no defect is state-changing and
/// invisible.
#[test]
fn defect_visibility_matrix() {
    let mut rng = Rng(0x9a11_7e57_0003);
    let mut norm_total = 0usize;
    let mut norm_caught = 0usize;
    for (name, mutations) in [
        ("flip_drops_gamma", Mutations { flip_drops_gamma: true, ..Mutations::default() }),
        ("merge_phase", Mutations { merge_phase: true, ..Mutations::default() }),
        (
            "key_ignores_h+skip_verify",
            Mutations { key_ignores_h: true, skip_verify: true, ..Mutations::default() },
        ),
    ] {
        let cfg = PruneConfig { mutations, ..PruneConfig::default() };
        let (mut differs, mut conf_broke, mut norm_broke) = (0usize, 0usize, 0usize);
        for _ in 0..40 {
            let n = 3 + rng.below(2);
            let c = scrambled_then_t(&mut rng, n, 6);
            let clean = prune::run_pruned(n, &c, &PruneConfig::default()).state_vector();
            assert!(
                prune::cyc_eq(exact_norm(&clean), LCyc::ONE),
                "clean build failed its own gauge circuit"
            );
            let dirty = prune::run_pruned(n, &c, &cfg).state_vector();
            let changed = !clean.iter().zip(&dirty).all(|(a, b)| prune::cyc_eq(*a, *b));
            if changed {
                differs += 1;
            }
            if !prune::cyc_eq(exact_norm(&dirty), LCyc::ONE) {
                norm_broke += 1;
            }
            let conf_failed = conforms(n, &c, &cfg).is_err();
            if conf_failed {
                conf_broke += 1;
            }
            assert_eq!(
                changed, conf_failed,
                "`{name}`: conformance and the state disagreed about whether the answer moved"
            );
        }
        assert!(differs > 0, "`{name}` never changed the answer — a vacuous gauge");
        assert_eq!(conf_broke, differs, "`{name}`: conformance missed a state-changing defect");
        norm_total += differs;
        norm_caught += norm_broke;
        println!(
            "defect `{name:<25}`: changed the answer {differs}/40, \
conformance caught {conf_broke}/{differs}, normalization caught {norm_broke}/{differs}"
        );
    }
    assert!(
        norm_caught < norm_total,
        "normalization now catches everything — re-check whether it can be promoted"
    );
    println!(
        "conformance {norm_total}/{norm_total} vs normalization {norm_caught}/{norm_total}: \
the reference is the certificate, the invariant is a sanity check"
    );
}

/// The deepest cells of the rate table (`t = 16`), conformed against the
/// CERTIFIED reference rather than against a proxy. The naive `2^16` sum costs
/// about two seconds per circuit at these widths — cheap enough that there is no
/// excuse for certifying the headline numbers with a weaker instrument.
#[test]
fn deepest_measured_cells_conform() {
    let mut rng = Rng(0xdeef_0010_0016);
    let cfg = PruneConfig { verify_points: usize::MAX, ..PruneConfig::default() };
    for &(n, t) in &[(2usize, 16usize), (3, 16), (4, 16)] {
        for case in 0..2 {
            let c = scrambled_then_t(&mut rng, n, t);
            let s = conforms(n, &c, &cfg).unwrap_or_else(|i| {
                panic!("deep cell case {case} (n={n}, t={t}) mismatch at basis state {i}")
            });
            assert_eq!(s.naive_branches, 1u128 << t);
            assert!(s.final_branches <= 1usize << n);
            println!(
                "deep cell n={n} t={t}: {} of {} branches survive, exact against the reference",
                s.final_branches, s.naive_branches
            );
        }
    }
    // The same cells for the two structured classes.
    for pairs in [6usize, 8] {
        let c = adjacent_tt(&mut rng, 4, 12, pairs);
        let s = conforms(4, &c, &cfg)
            .unwrap_or_else(|i| panic!("deep T;T x{pairs} mismatch at {i}"));
        println!(
            "deep cell T;T x{pairs} (n=4, t={}): {} of {} branches survive, exact",
            2 * pairs,
            s.final_branches,
            s.naive_branches
        );
    }
}

/// The default `verify_points = 64` budget caps the amplitude cross-check, not
/// the DECISION: the decision is the exact canonical-key comparison. So running
/// the whole determining set must not change a single measured branch count. If
/// it did, the budget would be silently choosing answers.
#[test]
fn verify_budget_does_not_change_the_answer() {
    let mut rng = Rng(0xbd6e_7000_0004);
    let full = PruneConfig { verify_points: usize::MAX, ..PruneConfig::default() };
    let cheap = PruneConfig { verify_points: 4, ..PruneConfig::default() };
    for &(n, t) in &[(4usize, 12usize), (6, 12), (8, 12), (4, 16)] {
        for _ in 0..6 {
            let c = scrambled_then_t(&mut rng, n, t);
            let a = prune::run_pruned(n, &c, &PruneConfig::default()).stats;
            let b = prune::run_pruned(n, &c, &full).stats;
            let d = prune::run_pruned(n, &c, &cheap).stats;
            assert_eq!(a.final_branches, b.final_branches, "n={n} t={t}: budget 64 vs full");
            assert_eq!(a.final_branches, d.final_branches, "n={n} t={t}: budget 64 vs 4");
            assert_eq!(a.exact_cancellations, b.exact_cancellations);
            assert_eq!(a.verify_rejections, 0);
            assert_eq!(b.verify_rejections, 0);
        }
    }
    println!("verify budget {{4, 64, full}} agree on every measured branch count");
}

/// What routing the branch fold through `merge::MergeLedger` actually buys.
///
/// The law is associative and commutative, so ANY ordering or sharding of the
/// fold must give the same exact answer without coordination — which is the
/// warrant `BranchSource`'s doc comment claims when it says the mesh can shard
/// this. Claiming it from the trait bound alone would be a re-assertion; this
/// exercises it on real pruned branch lists: forward, reversed, and split into
/// two shards folded independently and then merged.
#[test]
fn branch_fold_is_shardable() {
    let mut rng = Rng(0x5a4d_0007_0007);
    let mut checked = 0usize;
    for &(n, t) in &[(4usize, 12usize), (6, 12), (8, 12)] {
        for _ in 0..6 {
            let c = scrambled_then_t(&mut rng, n, t);
            let sum = prune::run_pruned(n, &c, &PruneConfig::default());
            if sum.branches.len() < 2 {
                // Pruned all the way to rank 1: a one-term fold cannot be
                // reordered, so the case is vacuous rather than passing. The
                // count below is what keeps this test from being all vacua.
                continue;
            }
            for idx in 0..(1usize << n) {
                let y: Vec<bool> = (0..n).map(|q| idx >> q & 1 == 1).collect();
                let terms: Vec<LCyc> = sum
                    .branches
                    .iter()
                    .map(|b| b.weight.mul(b.state.amplitude(&y)))
                    .collect();
                let forward = fold(terms.iter().copied());
                let reversed = fold(terms.iter().rev().copied());
                let (l, r) = terms.split_at(terms.len() / 2);
                let sharded = fold(l.iter().copied()).merge(fold(r.iter().copied()));
                assert!(prune::cyc_eq(forward, reversed), "fold order changed the answer");
                assert!(prune::cyc_eq(forward, sharded), "sharding changed the answer");
                assert!(
                    prune::cyc_eq(forward, sum.amplitude(&y)),
                    "PrunedSum::amplitude disagrees with the bare fold"
                );
            }
            checked += 1;
        }
    }
    assert!(
        checked >= 12,
        "only {checked} circuits had more than one branch — too many vacuous cases \
for this to be evidence about ordering"
    );
    println!("branch fold shardable (forward = reversed = 2-shard) on {checked} circuits");
}
