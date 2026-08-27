//! EXACT branch pruning for magic-tier stabilizer branch sums.
//!
//! The magic tier writes a Clifford+T circuit as Σ_b c_b |φ_b⟩ over 2^t affine
//! stabilizer branches (`holon-qasm`'s certified `magic` module is the
//! reference; the affine form is Dehaene–De Moor 2003 / Van den Nest 2010,
//! credited there). Two branches can be removed from that sum WITHOUT
//! approximation:
//!
//! 1. ANNIHILATED BRANCH — the branch state is the zero vector (a Gauss sum
//!    over an inconsistent constraint). The reference already skips these.
//! 2. DUPLICATE STATE — two branches whose affine states are THE SAME state up
//!    to a global scalar. Then c₁|φ₁⟩ + c₂|φ₂⟩ = (c₁ + c₂λ)|φ₁⟩ exactly, one
//!    branch disappears, and nothing is lost.
//!
//! Both are optimizations only an EXACT ledger can take. In floating point "the
//! same state up to a scalar" is a tolerance question, and the merged weight
//! would carry the tolerance forward into every later merge. Here the test is a
//! byte comparison of a canonical form over F₂ and Z/4, and the merged weight is
//! an addition in Z[ω]·2^{−m/2} — so a merged sum is not an approximation of the
//! naive sum, it IS the naive sum.
//!
//! # The canonical form (why the fingerprint is sound AND complete)
//!
//! A branch state is `amp(x) = γ · i^{Σ d_a u_a} · (−1)^{Σ_{a<b} J_ab u_a u_b}`
//! on `x = R u ⊕ h`, zero off the coset. The parametrization has exactly two
//! gauge freedoms, and each is killed by a normal form:
//!
//! * COLUMN BASIS `u ↦ M u`, `M ∈ GL(k,2)`. Killed by putting `R` in reduced
//!   column echelon form: RCEF is unique for a given column space, and since `R`
//!   has full column rank, `R M = R` forces `M = I`.
//! * COSET ORIGIN `h ↦ h ⊕ R u₀`. Killed by flipping pivot variables until `h`
//!   is zero on every pivot row; any nonzero element of `col(R)` is nonzero on
//!   some pivot row, so that representative is unique.
//!
//! With both fixed, `(d, J)` is forced by the state itself: `g(e_a) = i^{d_a}`
//! pins `d`, and `g(e_a ⊕ e_b) = i^{d_a + d_b}(−1)^{J_ab}` pins `J`. So
//!
//! > two branches carry the same state up to a global scalar **iff** their
//! > canonical `(R, h, d, J)` tuples are equal.
//!
//! The fingerprint is a hash of that tuple — sound because equality is then
//! re-checked on the full canonical bytes (the hash is never trusted), complete
//! because the tuple is a true normal form.
//!
//! # No division: the scalar ratio is extracted, not computed
//!
//! The brief's "phase ratio" `λ = γ₂/γ₁` never has to be divided out. `γ` is a
//! global factor, so canonicalization STRIPS it: the state is normalized to
//! `γ = 1` and the extracted `γ` is multiplied into the branch weight. Merging
//! is then `c₁.merge(c₂ · λ)` — a single exact ledger post — and `λ` is present
//! implicitly and exactly as `γ₂/γ₁`. This also means a merge can produce an
//! EXACT ZERO weight (`w₁ + w₂ = 0`), which drops another branch; in floating
//! point that cancellation reads as ~1e-16 and the branch survives forever.
//!
//! # One merge law, no bespoke accumulation
//!
//! Every accumulation in this module is `merge::MergeLedger` on `Cyc` — the
//! duplicate-branch coefficient merge, the branch fold behind `amplitude` (via
//! `merge::fold`), the odd-δ Gauss sum's `Σ_w i^{δw}`, and the credit-against-
//! debit posting that `cyc_eq` uses to decide exact equality. There is no
//! second addition path here.
//!
//! What that buys is not tidiness: the law is associative and commutative, so
//! the branch fold can be sharded across the mesh in any order without
//! coordination. `branch_fold_is_shardable` exercises that on real pruned branch
//! lists (forward, reversed, two-shard) rather than inferring it from the trait
//! bound.
//!
//! # What the merge actually finds: the Pauli-orbit ceiling
//!
//! This module was written expecting duplicates to be rare COINCIDENCES, so that
//! random Clifford+T would prune near 0% and only structured circuits would pay.
//! The measurement (`tests/prune.rs::measure_prune_rates`) says otherwise, and
//! the reason is structural, not lucky:
//!
//! Branch `b` differs from branch `b'` only by the `Z`'s the T-expansion
//! inserted. Push those `Z`'s forward through the remaining Clifford gates and
//! each becomes a PAULI, so every branch state is `P_b|ψ⟩` for ONE common
//! stabilizer state `|ψ⟩`. Two Paulis give the same state up to a scalar exactly
//! when they differ by an element of `|ψ⟩`'s stabilizer group, and `b ↦ P_b` mod
//! phase is a homomorphism out of `F₂^t`. So the branch set is a Pauli orbit:
//!
//! > **the merged branch count never exceeds `2^min(t, n)`, whatever the
//! > T-count** — checked on every circuit in `pauli_orbit_bound_holds`.
//!
//! That ceiling, not coincidence, is what the merge is discovering, and it is
//! why the measured rates are large everywhere rather than only on structured
//! input. The honest reading: this optimization buys a lot when `t > n` and
//! nothing at all when `t ≤ n`, which is exactly the regime where `2^t` was
//! affordable anyway. It moves the magic tier's wall from the T-count to the
//! QUBIT count; it does not remove a wall.
//!
//! # What is NOT claimed
//!
//! This is a state-equality merge. It is NOT the Bravyi–Gosset stabilizer-rank
//! decomposition (2^{~0.48t}), which finds a smaller SPANNING set rather than
//! collapsing coincidences, and which beats this one whenever `t < ~2n`. That
//! remains the named next improvement.
//!
//! The annihilated-branch drop (optimization 1) fired ZERO times across every
//! measured class, and cannot fire in this setting: a branch is a Clifford orbit
//! of `|0…0⟩` followed by Paulis, hence always a normalized state. The
//! reference's `zero` guard is defensive, not load-bearing, for branch sums that
//! start from a basis state. It is kept for the same reason.
//!
//! Zero runtime dependencies (`std` only).

use crate::ledger::Cyc;
use crate::merge::{fold, MergeLedger};
use std::collections::HashMap;

// ------------------------------------------------------------------ the engine
//
// The affine stabilizer engine — the state `(R, h, d, J, γ)`, the Clifford
// updates, the canonical form, and the ring helpers — lives in
// `crate::affine`, which is the ONE port of `holon-qasm::magic::Affine` in
// this crate. It is re-exported here under the names this module and its
// referees were written against.
//
// The canonical form (`canonicalize` / `canon_key` / `fingerprint`) was this
// lane's contribution to that union; the module header above is its warrant.

pub use crate::affine::{
    cyc_eq, cyc_is_zero, cyc_neg, i_pow, Affine, Gate, GaussStats, Mutations,
};

/// `CZ(c,t)` as `H(t) · CX(c,t) · H(t)`, appended to `out`. Convenience for the
/// structured circuit classes; CZ is not primitive in the reference gate set.
pub fn push_cz(out: &mut Vec<Gate>, c: usize, t: usize) {
    out.push(Gate::H(t));
    out.push(Gate::Cx(c, t));
    out.push(Gate::H(t));
}

// ------------------------------------------------------------------ driver

#[derive(Clone, Copy, Debug)]
pub struct PruneConfig {
    /// T-gates per block: duplicates are merged at each block boundary.
    /// `1` merges after every T (the aggressive default); `t` merges once at the
    /// end, which is the naive sum plus one pass.
    pub merge_every: usize,
    /// Exact amplitude cross-checks per candidate merge. `usize::MAX` runs the
    /// whole determining set.
    pub verify_points: usize,
    /// Refuse to grow the working set past this — a magic tier that silently
    /// blows up is worse than one that stops.
    pub max_working_set: usize,
    /// Canonicalize and drop annihilated branches, but never merge duplicates.
    /// The in-module control for a prune rate.
    pub disable_merge: bool,
    pub mutations: Mutations,
}

impl Default for PruneConfig {
    fn default() -> Self {
        PruneConfig {
            merge_every: 1,
            verify_points: 64,
            max_working_set: 1 << 22,
            disable_merge: false,
            mutations: Mutations::default(),
        }
    }
}

/// What the pruning actually achieved. Every field is a count, not a rate; the
/// rates are derived so the raw numbers stay auditable.
#[derive(Clone, Debug, Default)]
pub struct PruneStats {
    pub t_count: usize,
    /// `2^t`, saturating (the naive branch count).
    pub naive_branches: u128,
    /// Branches surviving the final merge.
    pub final_branches: usize,
    /// Largest working set held at any moment.
    pub peak_working_set: usize,
    /// Duplicate states folded into an existing branch.
    pub merged_duplicates: usize,
    /// Branches dropped because the state was annihilated (`zero`).
    pub zero_states_dropped: usize,
    /// Branches dropped because a merge produced an EXACT zero weight. Only an
    /// exact ledger can see these.
    pub exact_cancellations: usize,
    /// Fingerprint bucket hits rejected by the full-key comparison. Should be 0;
    /// a nonzero count means the hash is doing work it must not be trusted for.
    pub hash_collisions_rejected: usize,
    /// Key matches rejected by the exact amplitude cross-check. Should be 0 on a
    /// clean build; nonzero means the canonical form is unsound (or planted).
    pub verify_rejections: usize,
    /// `(t-gates consumed, branches before merge, branches after)` per block.
    pub blocks: Vec<(usize, usize, usize)>,
}

impl PruneStats {
    /// Surviving fraction of the naive `2^t`. 1.0 = nothing pruned.
    pub fn survival(&self) -> f64 {
        if self.naive_branches == 0 {
            return 1.0;
        }
        self.final_branches as f64 / self.naive_branches as f64
    }
    /// `log2(2^t / final)` — the T-gates the pruning gave back.
    pub fn t_saved(&self) -> f64 {
        if self.final_branches == 0 {
            return self.t_count as f64;
        }
        self.t_count as f64 - (self.final_branches as f64).log2()
    }
}

#[derive(Clone)]
pub struct Branch {
    /// `c_b · γ_b` — the T-expansion coefficient with the state's global scalar
    /// already folded in, which is what makes merging a plain addition.
    pub weight: Cyc,
    pub state: Affine,
}

/// The pruned branch sum: `Σ_b weight_b · |φ̂_b⟩` with every `|φ̂_b⟩` canonical,
/// distinct, and nonzero. Exactly equal to the naive sum.
pub struct PrunedSum {
    pub n: usize,
    pub branches: Vec<Branch>,
    pub stats: PruneStats,
}

impl PrunedSum {
    /// Exact amplitude of one basis state: `final_branches · poly(n)` work.
    ///
    /// The branch fold is `merge::fold`, so it is associative and commutative by
    /// the one law and therefore shardable across the mesh without coordination
    /// — which is the warrant `BranchSource`'s doc comment claims for it.
    pub fn amplitude(&self, y: &[bool]) -> Cyc {
        fold(self.branches.iter().filter_map(|b| {
            let a = b.state.amplitude(y);
            if cyc_is_zero(a) {
                None
            } else {
                Some(b.weight.mul(a))
            }
        }))
    }

    /// Full exact state vector, index `i` = the basis state with bit q = i>>q&1.
    /// Costs `2^n` by construction — for conformance on small n, not for use.
    pub fn state_vector(&self) -> Vec<Cyc> {
        let dim = 1usize << self.n;
        let mut out = vec![Cyc::ZERO; dim];
        let mut y = vec![false; self.n];
        for (idx, slot) in out.iter_mut().enumerate() {
            for (q, yq) in y.iter_mut().enumerate() {
                *yq = idx >> q & 1 == 1;
            }
            *slot = self.amplitude(&y);
        }
        out
    }
}

impl crate::BranchSource for PrunedSum {
    fn n_branches(&self) -> u64 {
        self.branches.len() as u64
    }
    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        let b = &self.branches[branch as usize];
        b.weight.mul(b.state.amplitude(y))
    }
    fn n_qubits(&self) -> usize {
        self.n
    }
}

/// `2^t`, saturating. `t` past 127 is not a number anyone is going to reach.
pub fn naive_branch_count(gates: &[Gate]) -> u128 {
    let t = gates.iter().filter(|g| g.is_t()).count();
    if t >= 127 {
        u128::MAX
    } else {
        1u128 << t
    }
}

/// `T = ((1+ω)/2) I + ((1−ω)/2) Z`; `T† ` with `ω ↦ ω⁻¹ = −ω³`.
fn t_coeffs(dagger: bool) -> (Cyc, Cyc) {
    if !dagger {
        (Cyc { c: [1, 1, 0, 0], m: 2 }, Cyc { c: [1, -1, 0, 0], m: 2 })
    } else {
        (Cyc { c: [1, 0, 0, -1], m: 2 }, Cyc { c: [1, 0, 0, 1], m: 2 })
    }
}

/// Canonicalize, drop annihilated branches, merge duplicates, drop exact
/// cancellations. Every surviving branch is canonical and pairwise distinct.
fn merge_block(branches: &mut Vec<Branch>, cfg: &PruneConfig, stats: &mut PruneStats) {
    let before = branches.len();
    let mut kept: Vec<Branch> = Vec::with_capacity(before);
    let mut keys: Vec<Vec<u8>> = Vec::with_capacity(before);
    let mut buckets: HashMap<u64, Vec<usize>> = HashMap::new();

    for mut b in branches.drain(..) {
        let g = b.state.canonicalize();
        if b.state.is_zero() {
            stats.zero_states_dropped += 1;
            continue;
        }
        b.weight = b.weight.mul(g);
        if cyc_is_zero(b.weight) {
            // A T-expansion coefficient is never zero and γ is never zero, so
            // this only happens if a caller handed us a zero weight.
            stats.exact_cancellations += 1;
            continue;
        }
        if cfg.disable_merge {
            kept.push(b);
            continue;
        }
        let key = b.state.canon_key();
        // The bucket index is `Affine::fingerprint`'s own hash over the very
        // bytes just computed — one FNV-1a in the tree, so an index and a
        // state fingerprint cannot drift apart.
        let fp = crate::affine::fnv1a(&key);
        let mut target = None;
        for &idx in buckets.get(&fp).map(|v| v.as_slice()).unwrap_or(&[]) {
            if keys[idx] != key {
                // The hash agreed and the states did not. This is exactly the
                // case the brief refuses to trust a hash for.
                stats.hash_collisions_rejected += 1;
                continue;
            }
            if !cfg.mutations.skip_verify
                && !kept[idx].state.amplitudes_agree(&b.state, cfg.verify_points)
            {
                stats.verify_rejections += 1;
                continue;
            }
            target = Some(idx);
            break;
        }
        match target {
            Some(idx) => {
                // THE duplicate-branch coefficient merge, and it is the one
                // merge law: `c₁.merge(c₂ · λ)`. The phase ratio λ = γ₂/γ₁ is
                // already inside `b.weight` — canonicalization multiplied the
                // stripped γ in — so the ledger op is all that is left.
                let posted = if cfg.mutations.merge_phase {
                    b.weight.mul(i_pow(1))
                } else {
                    b.weight
                };
                kept[idx].weight = kept[idx].weight.merge(posted);
                stats.merged_duplicates += 1;
            }
            None => {
                buckets.entry(fp).or_default().push(kept.len());
                keys.push(key);
                kept.push(b);
            }
        }
    }

    // Exact cancellation: a merged weight that is exactly zero removes the
    // branch outright. Floating point reads these as ~1e-16 and keeps them.
    let mut out = Vec::with_capacity(kept.len());
    for b in kept {
        if cyc_is_zero(b.weight) {
            stats.exact_cancellations += 1;
        } else {
            out.push(b);
        }
    }
    stats.blocks.push((stats.blocks.len(), before, out.len()));
    *branches = out;
}

/// Run the circuit as a PRUNED branch sum.
///
/// Breadth-first over T-choices in blocks of `cfg.merge_every`, merging at each
/// block boundary. This is the same enumeration a depth-first walk performs, but
/// the working set is bounded by `merged · 2^block` instead of the recursion's
/// full subtree — with `merge_every = 1` the working set never exceeds twice the
/// merged count, which is the whole point.
pub fn run_pruned(n: usize, gates: &[Gate], cfg: &PruneConfig) -> PrunedSum {
    let mut stats = PruneStats {
        t_count: gates.iter().filter(|g| g.is_t()).count(),
        naive_branches: naive_branch_count(gates),
        ..PruneStats::default()
    };

    let mut branches = vec![Branch {
        weight: Cyc::ONE,
        state: Affine::with_mutations(n, cfg.mutations),
    }];
    let block = cfg.merge_every.max(1);
    let mut since_merge = 0usize;

    for &g in gates {
        match g {
            Gate::T(q) | Gate::Tdg(q) => {
                let (ci, cz) = t_coeffs(matches!(g, Gate::Tdg(_)));
                assert!(
                    branches.len() * 2 <= cfg.max_working_set,
                    "pruned working set would exceed max_working_set ({}) at T #{}",
                    cfg.max_working_set,
                    since_merge
                );
                let mut next = Vec::with_capacity(branches.len() * 2);
                for b in branches.drain(..) {
                    let mut zb = b.clone();
                    zb.weight = zb.weight.mul(cz);
                    zb.state.z(q);
                    let mut ib = b;
                    ib.weight = ib.weight.mul(ci);
                    next.push(ib);
                    next.push(zb);
                }
                branches = next;
                stats.peak_working_set = stats.peak_working_set.max(branches.len());
                since_merge += 1;
                if since_merge >= block {
                    merge_block(&mut branches, cfg, &mut stats);
                    since_merge = 0;
                }
            }
            other => {
                for b in &mut branches {
                    b.state.apply(other);
                }
            }
        }
    }
    merge_block(&mut branches, cfg, &mut stats);
    stats.peak_working_set = stats.peak_working_set.max(branches.len());
    stats.final_branches = branches.len();
    PrunedSum { n, branches, stats }
}

/// The unpruned sum: the same code path with merging switched off, so the only
/// branches it loses are the annihilated ones the reference already skips. This
/// is the in-module control a prune rate is quoted against; `holon-qasm::magic`
/// is the INDEPENDENT referee, and conformance is stated against that.
pub fn run_naive(n: usize, gates: &[Gate]) -> PrunedSum {
    let cfg = PruneConfig { disable_merge: true, ..PruneConfig::default() };
    run_pruned(n, gates, &cfg)
}
