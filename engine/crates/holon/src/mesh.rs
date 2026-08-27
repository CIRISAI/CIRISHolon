//! THE MESH — the branch fold as a one-level recursion of the holon.
//!
//! A magic-tier amplitude is a sum over stabilizer branches:
//! `⟨y|ψ⟩ = Σ_b coeff_b · ⟨y|φ_b⟩`, each term exact in `Z[ω]·2^{−m/2}`
//! (`ledger::Cyc`). The mesh is what happens when that sum is given
//! CHILDREN: the branch index space is cut into contiguous ranges, one per
//! shard, and each shard is a child holon in the only sense the data object
//! recognises —
//!
//! * it carries its own **ledger** (a partial `Cyc`, the same ring as the
//!   parent's, not a lossy summary of it);
//! * its **chart** is its branch range, declared up front by
//!   [`shard_ranges`] and never negotiated at run time;
//! * the **merge is exact**, and it is not this module's invention —
//!   `merge::fold` over `MergeLedger`, the one merge law
//!   (`src/merge.rs`), is the parent's own ledger operation, so composing
//!   children costs the parent nothing in precision and there is no
//!   reconciliation step to get wrong. The mesh implements no accumulation
//!   of its own: shard-local folds and the shard reduction are the same
//!   `merge::fold` call twice, once per tier of the recursion;
//! * the **certificate** is not a comment in this header. It is
//!   `tests/mesh.rs::fold_is_shard_invariant`: the same circuit, the same
//!   `y`, at shards ∈ {1, 2, 3, 7, 16}, must return the *same struct* — not
//!   the same float, the same `Cyc` — and must agree with the certified
//!   reference (`holon_qasm::magic::magic_amplitude`) as a complex number.
//!   A tier without its square is not a tier.
//!
//! **Why sharding is allowed to be free.** The merge law: exact `Z[ω]`
//! addition is associative and commutative, so the VALUE of the fold does
//! not depend on how it is cut. That is the warrant, and — read carefully —
//! it is a claim about VALUES.
//! Bit-identity of the *representation* is a strictly stronger claim, because
//! `Cyc` is normalised only by even powers of two (`ledger::Cyc::normalize`
//! halves while `m ≥ 2`): `1` and `√2/√2` are equal numbers with different
//! coefficient vectors, so a value can wear two normalised faces that differ
//! by one factor of `√2`. Aligning to the maximum `m` is what makes the
//! representation path-independent, and the one place a fold can lose the
//! parity of that maximum is a partial sum that cancels to EXACTLY zero
//! (zero normalises to `m = 0`, forgetting the `m` it came from). Different
//! shardings produce different partial sums, so that is the one crack through
//! which two shardings could return equal numbers in unequal structs. The
//! test does not assume it stays shut: it checks struct equality directly,
//! and `cancelling_partial_sums_are_the_representation_boundary` exhibits the
//! crack on hand-built ring elements so the boundary is on the record rather
//! than in a footnote.
//!
//! That test is ALSO a counterexample to the merge law's own wording, which
//! claims order-independence for `Cyc` under exact `PartialEq`
//! (`tests/laws.rs::merge_laws_hold_for_every_ledger` checks it with `a == b`
//! on a random draw that happens to miss this). Three elements are enough to
//! break it. The scope the law actually has for the tier-2 ledger is
//! order-independence of the VALUE, plus order-independence of the
//! REPRESENTATION whenever no partial sum cancels to zero — which is every
//! branch sum this crate has measured, and not a theorem. The durable repair
//! is an odd-`√2` reduction inside `ledger::Cyc::normalize`; [`canonicalize`]
//! below is that reduction, kept here as a tested remedy rather than applied,
//! because the ledger's own normal form is not the mesh's to change.
//!
//! **Determinism by construction.** Nothing here depends on how the threads
//! interleave: the cut is a pure function of `(n_branches, shards)`, each
//! worker folds its own range in ascending branch order into a private
//! accumulator, and the shard results are merged in SHARD-INDEX order after
//! every worker has finished. No atomics, no shared accumulator, no
//! completion-order reduction — the schedule is not an input.
//!
//! Zero dependencies: `std::thread::scope` only, no rayon.

use crate::ledger::Cyc;
use crate::merge::{self, MergeLedger};
use crate::BranchSource;
use core::ops::Range;

/// The chart: `[0, n_branches)` cut into contiguous, gap-free, ascending
/// ranges, one per shard. A pure function of its two arguments — this is
/// where the mesh's determinism actually lives.
///
/// `shards` is clamped to at least 1 and at most `n_branches`, so no range is
/// empty and the returned length is the number of children that will run.
/// `n_branches == 0` yields no children at all.
pub fn shard_ranges(n_branches: u64, shards: usize) -> Vec<Range<u64>> {
    if n_branches == 0 {
        return Vec::new();
    }
    let s = (shards.max(1) as u64).min(n_branches);
    (0..s)
        .map(|i| {
            // u128 so the products cannot overflow for any u64 branch count.
            let lo = (i as u128 * n_branches as u128 / s as u128) as u64;
            let hi = ((i + 1) as u128 * n_branches as u128 / s as u128) as u64;
            lo..hi
        })
        .collect()
}

/// The canonical face of a ledger entry: divide out every factor of `√2` the
/// ring allows, so that equal values are equal structs.
///
/// `Cyc::normalize` halves only while `m ≥ 2`, so it removes even powers of
/// two and stops — leaving `1 = ([1,0,0,0], m=0)` and
/// `1 = ([0,1,0,−1], m=1)` as two normalised faces of one number. Dividing by
/// `√2 = ω − ω³` is exact in `Z[ω]` exactly when `c₀ ≡ c₂` and `c₁ ≡ c₃`
/// (mod 2), and this applies it until it no longer can.
///
/// **Not applied by [`fold_amplitude`], deliberately.** The mesh returns
/// precisely what the sequential fold returns, so a caller comparing the two
/// sees no difference the mesh invented; and the right home for a canonical
/// form is `ledger::Cyc` itself, not one of its consumers. This function
/// exists so the boundary in the module header has a tested remedy attached
/// rather than a suggestion — see
/// `tests/mesh.rs::cancelling_partial_sums_are_the_representation_boundary`.
pub fn canonicalize(x: Cyc) -> Cyc {
    if x.c.iter().all(|&v| v == 0) {
        return Cyc::ZERO;
    }
    let mut out = x;
    while out.m >= 1
        && (out.c[0] - out.c[2]) % 2 == 0
        && (out.c[1] - out.c[3]) % 2 == 0
    {
        // √2·[t₀,t₁,t₂,t₃] = [t₁−t₃, t₀+t₂, t₁+t₃, t₂−t₀]; invert it.
        let c = out.c;
        out.c = [
            (c[1] - c[3]) / 2,
            (c[0] + c[2]) / 2,
            (c[1] + c[3]) / 2,
            (c[2] - c[0]) / 2,
        ];
        out.m -= 1;
    }
    out
}

/// One child's whole life: fold `range` into a private ledger, in ascending
/// branch order. Public because a shard is a holon in its own right — the
/// same call the parent makes, one tier down.
pub fn fold_range<S: BranchSource>(src: &S, y: &[bool], range: Range<u64>) -> Cyc {
    // `merge::fold` seeded with `MergeLedger::empty`, over an ASCENDING range:
    // the one law, no bespoke accumulation anywhere in this module.
    merge::fold(range.map(|b| src.amplitude_of(b, y)))
}

/// `Σ_b coeff_b · ⟨y|φ_b⟩`, folded across `shards` OS threads, exactly.
///
/// Deterministic for every `shards`: see the module header. `shards <= 1`
/// runs on the calling thread with no spawn at all, so it is the honest
/// serial baseline for a speedup curve — the thread costs are charged to the
/// parallel arms, where they are actually paid.
pub fn fold_amplitude<S: BranchSource>(src: &S, y: &[bool], shards: usize) -> Cyc {
    assert_eq!(
        y.len(),
        src.n_qubits(),
        "mesh: |y| must be the source's qubit count"
    );
    let ranges = shard_ranges(src.n_branches(), shards);
    match ranges.len() {
        0 => Cyc::empty(),
        1 => fold_range(src, y, ranges[0].clone()),
        n => {
            // One slot per child, written by exactly one worker. Disjoint
            // &mut borrows, so no lock and no atomic is involved in the
            // accumulation — and none is involved in the merge either,
            // because the merge happens after the scope has joined.
            let mut partial = vec![Cyc::empty(); n];
            std::thread::scope(|scope| {
                for (slot, range) in partial.iter_mut().zip(ranges) {
                    scope.spawn(move || {
                        *slot = fold_range(src, y, range);
                    });
                }
            });
            // The same law again, over the children in SHARD-INDEX order —
            // never in completion order.
            merge::fold(partial)
        }
    }
}
