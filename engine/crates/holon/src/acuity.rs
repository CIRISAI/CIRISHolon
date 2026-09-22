//! ACUITY — the hard part, priced before it is run and stopped when the
//! demanded accuracy is reached, with the unevaluated tail carried as a
//! CERTIFICATE rather than a hope.
//!
//! The QVM's arithmetic (OBJECT.md, "The procedure, as arithmetic"): the
//! cheap part is the Clifford sector and runs exact; the hard part is a sum
//! over stabilizer branches whose count is PRICED before the run by the
//! T-count oracle ([`crate::magic5::expected_branches`]); and the acuity `ε`
//! demanded of the output is the budget that says how much of that sum to do.
//! This module is the third clause: the branch sum with a running remainder
//! bound, evaluated in a declared order, folded by [`crate::mesh`].
//!
//! # The declared order, and why it is this one
//!
//! Branches are evaluated in **descending order of their own certified
//! bound, ties broken by ascending mixed-radix branch index**
//! ([`ORDER`]). Two reasons, and neither is taste:
//!
//! * **It makes the bound as tight as a bound of this shape can be.** The
//!   remainder carried after `k` branches is `R_k = Σ_{p ≥ k} bound(o(p))`,
//!   a suffix sum of a fixed multiset of non-negative numbers. Sorting
//!   descending minimises *every* suffix simultaneously — it is the greedy
//!   optimum, not a heuristic, so no other order reaches a given `ε` sooner
//!   under this bound.
//! * **It degenerates to the mesh's native order when nothing is known.**
//!   [`BranchSource::branch_bound`]'s default is `+∞`; all bounds equal means
//!   the tie-break decides, and the tie-break is the ascending branch index
//!   the mesh already shards on. A source that derives no bound therefore
//!   evaluates its branches in exactly the order [`crate::mesh`] would, and
//!   truncates never — slow, never wrong.
//!
//! The order is a pure function of the source, NOT of `y`, `ε` or the shard
//! count. That is what makes the stopping decision shard-independent: the
//! prefix `[0, k)` is chosen before any thread starts, and the answer at
//! `S = 8` is the same struct as at `S = 1`.
//!
//! # Where the bound comes from
//!
//! A branch contributes `coeff_b · ⟨y|φ_b⟩`. Two bounds are derived in this
//! crate, both honest, and they differ by a factor that is the whole story of
//! this campaign:
//!
//! * **The a-priori coefficient bound** ([`crate::magic5::Magic5Source`]'s
//!   [`BranchSource::branch_bound`]): the states `magic5` attaches are
//!   UNNORMALISED (`γ = 1` on a support of `2^k`), so `‖φ_b‖ = 2^{k/2}`, and
//!   every gate that follows — the decomposition's Clifford frames and the
//!   circuit itself — is unitary and moves neither. Hence
//!   `|⟨y|φ_b⟩| ≤ ‖φ_b‖ = 2^{k/2}` and the bound is `|coeff_b|·2^{k/2}`: the
//!   sum of the remaining coefficients, normalised, exactly as the
//!   decomposition hands them over. It costs NO branch evaluation. It is also
//!   loose by `2^{rank/2}` — the amplitude of an `n`-qubit state is `2^{-n/2}`
//!   sized while this bound knows only the norm — and at the campaign's sizes
//!   that factor is `10³`–`10⁵`, so it never truncates. That is a measurement,
//!   reported, not a defect hidden.
//! * **The certified scalar bound** ([`crate::magic5::Magic5Source::scalar_bound`],
//!   installed by [`Bounded`]): an affine state's amplitude is `γ·i^p` on its
//!   support and `0` off it, so `|coeff_b·⟨y|φ_b⟩|` is EXACTLY `|coeff_b·γ_b|`
//!   or exactly `0`, for every `y`. `γ_b` costs one affine evolution per
//!   branch and — this is the point — does not depend on `y`, so it is paid
//!   once per circuit and amortised over every amplitude the observable needs.
//!   This is the bound that actually truncates.
//!
//! Both are UPPER bounds; the certificate is `|value − truth| ≤ R_k` in each
//! case. Which one a source installs is the source's business; this module
//! only sums them, rounding every addition UP (see [`nudge_up`]) so that f64
//! arithmetic cannot make the certificate a fraction of an ulp too small.

use crate::ledger::Cyc;
use crate::magic::Circuit;
use crate::magic5::{expected_branches, Magic5Source};
use crate::merge::{self, MergeLedger};
use crate::mesh;
use crate::BranchSource;
use core::ops::Range;

/// The declared evaluation order, carried in every [`Budgeted`] so a result
/// cannot be read without it.
pub const ORDER: &str = "bound-descending, ties by ascending branch index";

/// The next representable double above `x`. Every per-branch bound and every
/// partial sum of bounds is passed through this, so a remainder reported as
/// `R` is `≥` the exact real sum of the bounds it is made of. A certificate
/// that is one ulp optimistic is still a certificate that lied.
pub fn nudge_up(x: f64) -> f64 {
    if x.is_finite() && x > 0.0 {
        f64::from_bits(x.to_bits() + 1)
    } else {
        x
    }
}

// --------------------------------------------------------------- the result

/// The budgeted sum's whole answer: the value, the certificate, and the price
/// against what was actually spent.
#[derive(Clone, Debug)]
pub struct Budgeted {
    /// The exact partial sum over the evaluated prefix, in `Z[ω]·2^{−m/2}`.
    pub value: Cyc,
    /// `value` as a complex number — the only exit to floating point.
    pub value_f64: (f64, f64),
    /// `R_k`: the certified bound on `|truth − value|`. Zero iff every
    /// branch was evaluated.
    pub remainder: f64,
    /// `k` — branches evaluated.
    pub evaluated: u64,
    /// `N` — branches the price named.
    pub total: u64,
    /// [`ORDER`], carried with the number.
    pub order: &'static str,
}

impl Budgeted {
    /// The line printed AFTER the run, the counterpart of [`price_line`].
    pub fn executed_line(&self) -> String {
        format!(
            "executed: k={} of N={} remainder={:.6e}",
            self.evaluated, self.total, self.remainder
        )
    }

    /// `k/N` — the fraction of the price that was actually spent.
    pub fn fraction(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.evaluated as f64 / self.total as f64
        }
    }
}

/// The line printed BEFORE any branch is evaluated: the price, from the
/// T-count alone. `t_eff` is the located T-count (S1's output), so this is
/// the certificate that the cost was named before it was paid.
pub fn price_line(t_eff: usize) -> String {
    format!("price: t_eff={} N_pred={}", t_eff, expected_branches(t_eff))
}

// ----------------------------------------------------------------- the plan

/// The order and the remainder profile — everything about the budget that
/// does not depend on `y`, computed once.
///
/// Build it once per source and reuse it across every amplitude the
/// observable needs ([`budgeted_amplitude_with`]); [`budgeted_amplitude`]
/// builds one per call, which is right for a single query and wasteful for a
/// marginal.
#[derive(Clone, Debug)]
pub struct BudgetPlan {
    /// Branch indices in evaluation order.
    pub order: Vec<u64>,
    /// `bounds[p]` is the bound of `order[p]`, descending.
    pub bounds: Vec<f64>,
    /// `suffix[k] = Σ_{p ≥ k} bounds[p]`, rounded up; `suffix[N] = 0`.
    pub suffix: Vec<f64>,
}

impl BudgetPlan {
    /// One call to [`BranchSource::branch_bound`] per branch, then a sort.
    pub fn of<S: BranchSource + ?Sized>(src: &S) -> BudgetPlan {
        let n = src.n_branches();
        let mut ix: Vec<(f64, u64)> = (0..n).map(|b| (src.branch_bound(b), b)).collect();
        // `total_cmp` is a TOTAL order on f64 — no `unwrap` on a partial one,
        // and a NaN bound sorts deterministically instead of scrambling the
        // order (it then poisons the suffix sums, so the budget refuses to
        // truncate, which is the right failure).
        ix.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let order: Vec<u64> = ix.iter().map(|&(_, b)| b).collect();
        let bounds: Vec<f64> = ix.iter().map(|&(v, _)| v).collect();
        let mut suffix = vec![0.0f64; bounds.len() + 1];
        for k in (0..bounds.len()).rev() {
            suffix[k] = nudge_up(suffix[k + 1] + bounds[k]);
        }
        BudgetPlan { order, bounds, suffix }
    }

    /// `N`.
    pub fn n_branches(&self) -> u64 {
        self.order.len() as u64
    }

    /// `R_k`.
    pub fn remainder(&self, k: usize) -> f64 {
        self.suffix[k]
    }

    /// The first `k` with `R_k ≤ ε` — the stopping rule, and nothing else.
    ///
    /// `R` is non-increasing and `R_N = 0`, so the scan always terminates;
    /// `ε = 0` therefore evaluates every branch whose bound is nonzero, and
    /// a branch whose bound is exactly zero contributes exactly zero and is
    /// correctly skipped.
    pub fn stop_at(&self, eps: f64) -> usize {
        assert!(eps >= 0.0, "acuity: ε must be non-negative, got {eps}");
        (0..=self.bounds.len())
            .find(|&k| self.suffix[k] <= eps)
            .unwrap_or(self.bounds.len())
    }
}

// --------------------------------------------------------------- the prefix

/// The evaluated prefix as a [`BranchSource`] in its own right: position `p`
/// is branch `order[p]`. This is what the mesh shards, so the mesh shards
/// POSITIONS in the declared order and never raw branch indices — which is
/// why the stopping decision and the sharding are independent.
pub struct Prefix<'a, S: BranchSource + ?Sized> {
    src: &'a S,
    order: &'a [u64],
}

impl<'a, S: BranchSource + ?Sized> Prefix<'a, S> {
    pub fn new(src: &'a S, order: &'a [u64]) -> Self {
        Prefix { src, order }
    }
}

impl<S: BranchSource + ?Sized> BranchSource for Prefix<'_, S> {
    fn n_branches(&self) -> u64 {
        self.order.len() as u64
    }
    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        self.src.amplitude_of(self.order[branch as usize], y)
    }
    fn n_qubits(&self) -> usize {
        self.src.n_qubits()
    }
    fn branch_bound(&self, branch: u64) -> f64 {
        self.src.branch_bound(self.order[branch as usize])
    }
}

// ------------------------------------------------------------ the budgeted sum

/// `⟨y|C|0⟩` to acuity `ε`, with the remainder certified.
///
/// Branches are evaluated in [`ORDER`], the running remainder `R_k` bounds
/// the sum of everything not yet evaluated, and evaluation stops at the first
/// `k` with `R_k ≤ ε`. `ε = 0` evaluates everything. With `shards > 1` the
/// prefix is folded by [`crate::mesh::fold_amplitude`], bit-identically to
/// `shards = 1` — the stopping decision is made on the deterministic prefix,
/// so the answer does not depend on `S`.
///
/// The certificate: `|⟨y|C|0⟩ − value| ≤ remainder`, whenever the source's
/// [`BranchSource::branch_bound`] is an upper bound on its own branches.
pub fn budgeted_amplitude<S: BranchSource + ?Sized>(
    src: &S,
    y: &[bool],
    eps: f64,
    shards: usize,
) -> Budgeted {
    let plan = BudgetPlan::of(src);
    budgeted_amplitude_with(src, &plan, y, eps, shards)
}

/// [`budgeted_amplitude`] against a plan built once — the shape to use when
/// the observable needs more than one amplitude (a marginal is `2^4` of
/// them), because the plan does not depend on `y`.
pub fn budgeted_amplitude_with<S: BranchSource + ?Sized>(
    src: &S,
    plan: &BudgetPlan,
    y: &[bool],
    eps: f64,
    shards: usize,
) -> Budgeted {
    assert_eq!(y.len(), src.n_qubits(), "acuity: |y| must be the source's qubit count");
    assert_eq!(
        plan.n_branches(),
        src.n_branches(),
        "acuity: the plan was built for a different source"
    );
    let k = plan.stop_at(eps);
    let prefix = Prefix::new(src, &plan.order[..k]);
    let value = mesh::fold_amplitude(&prefix, y, shards);
    Budgeted {
        value,
        value_f64: value.to_complex(),
        remainder: plan.remainder(k),
        evaluated: k as u64,
        total: plan.n_branches(),
        order: ORDER,
    }
}

/// [`budgeted_amplitude`] with the price and the execution printed to stderr,
/// in that order and on either side of the work: `t_eff` is the located
/// T-count, so the first line is the price stated BEFORE any branch is
/// evaluated and the second is what was spent against it.
pub fn budgeted_amplitude_priced<S: BranchSource + ?Sized>(
    src: &S,
    y: &[bool],
    eps: f64,
    shards: usize,
    t_eff: usize,
) -> Budgeted {
    // The price FIRST, before even the plan is built — the plan costs one
    // `branch_bound` per branch and evaluates none, but "before any branch is
    // evaluated" is a claim about ordering and this is the ordering.
    eprintln!("{}", price_line(t_eff));
    let plan = BudgetPlan::of(src);
    let out = budgeted_amplitude_with(src, &plan, y, eps, shards);
    eprintln!("{}", out.executed_line());
    out
}

// ------------------------------------------------------------- the shard fold

/// The mesh fold with its children's ledgers kept: one partial per shard, in
/// shard-index order, plus the merge of them.
///
/// [`mesh::fold_amplitude`] returns only the parent's number; a shard cannot
/// be convicted from that, so this runs the same two calls the mesh makes —
/// [`mesh::shard_ranges`] for the chart, [`mesh::fold_range`] per child — and
/// keeps the children. The value is bit-identical to
/// [`mesh::fold_amplitude`]'s and `tests/qvm_acuity_hard.rs` says so.
#[derive(Clone, Debug)]
pub struct ShardFold {
    /// Ranges over PREFIX POSITIONS, not raw branch indices.
    pub ranges: Vec<Range<u64>>,
    pub partials: Vec<Cyc>,
    pub value: Cyc,
}

/// Merge shard partials by the one merge law. Order-independent in value;
/// `tests/qvm_acuity_hard.rs::pq5_*` checks it is order-independent in the
/// STRUCT on this campaign's data too.
pub fn merge_partials(partials: &[Cyc]) -> Cyc {
    merge::fold(partials.iter().copied())
}

/// [`budgeted_amplitude_with`], keeping the shards' own ledgers.
pub fn budgeted_amplitude_sharded<S: BranchSource + ?Sized>(
    src: &S,
    plan: &BudgetPlan,
    y: &[bool],
    eps: f64,
    shards: usize,
) -> (Budgeted, ShardFold) {
    assert_eq!(y.len(), src.n_qubits(), "acuity: |y| must be the source's qubit count");
    let k = plan.stop_at(eps);
    let prefix = Prefix::new(src, &plan.order[..k]);
    let ranges = mesh::shard_ranges(k as u64, shards);
    let mut partials = vec![Cyc::empty(); ranges.len()];
    if ranges.len() <= 1 {
        for (slot, range) in partials.iter_mut().zip(ranges.iter()) {
            *slot = mesh::fold_range(&prefix, y, range.clone());
        }
    } else {
        std::thread::scope(|scope| {
            for (slot, range) in partials.iter_mut().zip(ranges.iter()) {
                let p = &prefix;
                let r = range.clone();
                scope.spawn(move || {
                    *slot = mesh::fold_range(p, y, r);
                });
            }
        });
    }
    let value = merge_partials(&partials);
    let budgeted = Budgeted {
        value,
        value_f64: value.to_complex(),
        remainder: plan.remainder(k),
        evaluated: k as u64,
        total: plan.n_branches(),
        order: ORDER,
    };
    (budgeted, ShardFold { ranges, partials, value })
}

/// A shard whose claimed partial is not the partial its own range folds to.
#[derive(Clone, Debug)]
pub struct Convicted {
    pub shard: usize,
    pub of: usize,
    pub range: Range<u64>,
    pub claimed: Cyc,
    pub recomputed: Cyc,
}

impl std::fmt::Display for Convicted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "shard {} of {} over prefix positions [{}, {}): claimed {:?}, folds to {:?}",
            self.shard, self.of, self.range.start, self.range.end, self.claimed, self.recomputed
        )
    }
}

/// Re-fold every shard's range and NAME the ones whose ledger does not match.
///
/// The conviction is per child and by name: a corrupted partial cannot hide
/// inside the parent's number, because the parent's number is not what is
/// checked — the child's is.
pub fn convict_shards<S: BranchSource + ?Sized>(
    src: &S,
    plan: &BudgetPlan,
    y: &[bool],
    fold: &ShardFold,
) -> Vec<Convicted> {
    let k: u64 = fold.ranges.last().map(|r| r.end).unwrap_or(0);
    let prefix = Prefix::new(src, &plan.order[..k as usize]);
    let mut out = Vec::new();
    for (i, range) in fold.ranges.iter().enumerate() {
        let recomputed = mesh::fold_range(&prefix, y, range.clone());
        if recomputed != fold.partials[i] {
            out.push(Convicted {
                shard: i,
                of: fold.ranges.len(),
                range: range.clone(),
                claimed: fold.partials[i],
                recomputed,
            });
        }
    }
    out
}

// ------------------------------------------------------------------ wrappers

/// A source carrying per-branch bounds computed once and held.
///
/// The certified scalar bound costs one affine evolution per branch and does
/// not depend on `y`; this is where that cost is paid once and amortised.
/// Also the seam a caller with its own bound (a sharper one, or a
/// deliberately blunt one for a control) plugs into.
pub struct Bounded<S: BranchSource> {
    inner: S,
    bounds: Vec<f64>,
}

impl<S: BranchSource> Bounded<S> {
    /// `bounds[b]` must be an upper bound on `|amplitude_of(b, y)|` for every
    /// `y`. Nothing here can check that — the plants do.
    pub fn new(inner: S, bounds: Vec<f64>) -> Self {
        assert_eq!(
            bounds.len() as u64,
            inner.n_branches(),
            "acuity: one bound per branch, or the certificate is a guess"
        );
        Bounded { inner, bounds }
    }

    pub fn inner(&self) -> &S {
        &self.inner
    }

    pub fn bounds(&self) -> &[f64] {
        &self.bounds
    }
}

impl<S: BranchSource> BranchSource for Bounded<S> {
    fn n_branches(&self) -> u64 {
        self.inner.n_branches()
    }
    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        self.inner.amplitude_of(branch, y)
    }
    fn n_qubits(&self) -> usize {
        self.inner.n_qubits()
    }
    fn branch_bound(&self, branch: u64) -> f64 {
        self.bounds[branch as usize]
    }
}

/// THE PLANTED MUTATION for the certificate: one branch's term, sign-flipped.
///
/// The bound is passed through UNCHANGED — the planted branch still claims
/// the bound it had — which is exactly what makes the plant a test of the
/// certificate rather than of the plant: at `ε = 0` the remainder is `0` and
/// any disagreement with the referee exceeds it.
pub struct SignFlip<'a, S: BranchSource + ?Sized> {
    pub inner: &'a S,
    pub branch: u64,
}

impl<S: BranchSource + ?Sized> BranchSource for SignFlip<'_, S> {
    fn n_branches(&self) -> u64 {
        self.inner.n_branches()
    }
    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        let a = self.inner.amplitude_of(branch, y);
        if branch == self.branch {
            Cyc { c: [-a.c[0], -a.c[1], -a.c[2], -a.c[3]], m: a.m }
        } else {
            a
        }
    }
    fn n_qubits(&self) -> usize {
        self.inner.n_qubits()
    }
    fn branch_bound(&self, branch: u64) -> f64 {
        self.inner.branch_bound(branch)
    }
}

// -------------------------------------------------------- building a source

/// THE ENTRY POINT for a located circuit: the Magic5FromCat branch source for
/// `⟨y|C|0^n⟩`.
///
/// `y` is the observable's bitstring; it is taken here only to refuse a width
/// mismatch at the door, because the source itself answers any `y` of the
/// circuit's width (the decomposition is of `|A^{⊗t}⟩`, not of the query).
/// The price this source will charge is
/// [`expected_branches`]`(circuit.t_count())` and
/// [`price_line`] is how it is stated.
///
/// Carries the DEFAULT bound — the a-priori coefficient bound, which costs
/// nothing and truncates nothing at these sizes. Use [`certified_source_for`]
/// to get one that truncates.
pub fn source_for(circuit: &Circuit, y: &[bool]) -> Magic5Source {
    assert_eq!(
        y.len(),
        circuit.n_qubits,
        "acuity: |y| must be the circuit's qubit count"
    );
    Magic5Source::new(circuit)
}

/// [`source_for`] with the certified scalar bounds installed: one affine
/// evolution per branch, paid once, `y`-independent, and the bound that makes
/// `ε > 0` actually stop early.
pub fn certified_source_for(circuit: &Circuit, y: &[bool]) -> Bounded<Magic5Source> {
    let src = source_for(circuit, y);
    let bounds = src.scalar_bounds();
    Bounded::new(src, bounds)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A branch source with values the test writes down, so the machinery is
    /// checked against arithmetic done by hand.
    struct Fixed {
        vals: Vec<Cyc>,
        bounds: Vec<f64>,
    }
    impl BranchSource for Fixed {
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

    #[test]
    fn the_order_is_descending_by_bound_and_then_by_index() {
        let src = Fixed {
            vals: vec![Cyc::ONE; 5],
            bounds: vec![1.0, 4.0, 4.0, 0.5, 2.0],
        };
        let plan = BudgetPlan::of(&src);
        assert_eq!(plan.order, vec![1, 2, 4, 0, 3]);
        assert!(plan.suffix[0] >= 11.5 && plan.suffix[0] < 11.5 + 1e-12);
        assert_eq!(plan.suffix[5], 0.0);
    }

    #[test]
    fn no_bound_means_no_truncation_and_the_mesh_order() {
        struct Blind(usize);
        impl BranchSource for Blind {
            fn n_branches(&self) -> u64 {
                self.0 as u64
            }
            fn amplitude_of(&self, _b: u64, _y: &[bool]) -> Cyc {
                Cyc::ZERO
            }
            fn n_qubits(&self) -> usize {
                1
            }
        }
        let src = Blind(7);
        let plan = BudgetPlan::of(&src);
        assert_eq!(plan.order, vec![0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(plan.stop_at(1e9), 7, "an unbounded source must not truncate");
    }

    #[test]
    fn the_remainder_is_rounded_up_never_down() {
        // 0.1 three times: the exact sum is not representable, and the
        // certificate must land above it, never below.
        let src = Fixed { vals: vec![Cyc::ZERO; 3], bounds: vec![0.1; 3] };
        let plan = BudgetPlan::of(&src);
        assert!(plan.suffix[0] >= 0.30000000000000004, "remainder rounded down");
    }
}
