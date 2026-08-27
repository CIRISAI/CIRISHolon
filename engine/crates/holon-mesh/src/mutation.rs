//! Deliberate defects, and the honest bookkeeping of which ones MUST fire and which ones
//! must NOT.
//!
//! # The trap this enum exists to avoid
//!
//! `MESH_DESIGN.md` §6 states it and this module is where it becomes code. The brief asks
//! that "a deliberately reordered merge be caught". Taken literally that test cannot pass,
//! and the reason is the design working rather than failing: with a per-colour matching and
//! integer lanes, **reordering the merge produces the identical result**. A mutation that
//! reorders and asserts the answer changed would fail against a correct implementation, and
//! the tempting repair — weakening it until it passes — ships a gate that cannot fail.
//!
//! So the reorder mutation is split, and both halves are required:
//!
//! * [`VisitOrder`] changes on their own **must not** change any bit. That IS the determinism
//!   claim; if reordering moved a bit, the mesh would be non-deterministic.
//! * [`Mutation::CrossShardFloatReduction`] plants the thing `SANDBOX_4090` D4 bans — an f64
//!   accumulation across cells, summed in visit order — and then reordering **must** change
//!   the bits. That is what proves the reorder harness has teeth, and that the ban is
//!   enforced rather than merely written down.
//!
//! Without the second half the first is a test that passes because nothing is being checked.

/// The order in which a sweep visits its edges and a mesh visits its shards.
///
/// Correct behaviour is invariance under all of these. They are not a feature; they are the
/// instrument that measures the determinism claim.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VisitOrder {
    #[default]
    Natural,
    Reversed,
    /// A deterministic non-trivial permutation — a stride walk over a coprime step, so it is
    /// neither the identity nor its reverse.
    Strided,
}

impl VisitOrder {
    /// The visiting sequence over `n` items.
    pub fn sequence(self, n: usize) -> Vec<usize> {
        match self {
            VisitOrder::Natural => (0..n).collect(),
            VisitOrder::Reversed => (0..n).rev().collect(),
            VisitOrder::Strided => {
                if n == 0 {
                    return Vec::new();
                }
                let step = if n % 3 == 0 { 5 } else { 3 };
                let mut out = Vec::with_capacity(n);
                let mut seen = vec![false; n];
                let mut i = 0;
                while out.len() < n {
                    if !seen[i] {
                        seen[i] = true;
                        out.push(i);
                    }
                    i = (i + step) % n;
                    if seen[i] {
                        match (0..n).find(|j| !seen[*j]) {
                            Some(next) => i = next,
                            None => break,
                        }
                    }
                }
                out
            }
        }
    }
}

/// A planted defect. Production callers pass [`Mutation::None`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mutation {
    #[default]
    None,

    /// **M1b — the one that gives the reorder test its teeth.** Every cell adds its energy
    /// into a shared f64 accumulator in visit order, and the accumulator feeds back into the
    /// cell. Float addition is not associative, so the result depends on the order — which is
    /// precisely the cross-lane float reduction `SANDBOX_4090` D4 bans on the certified path.
    CrossShardFloatReduction,

    /// **M2 — the live-read defect.** Each shard refreshes its halo from its peers' CURRENT
    /// values immediately before sweeping, interleaved with the sweep, instead of from a
    /// snapshot published before any shard moved. Shards visited later then see peers that
    /// have already advanced.
    ///
    /// This is `SANDBOX_4090` D3's first reason — snapshot, then apply — with the reason
    /// removed. `holon-swarm::exchange`'s
    /// `snapshot_planning_is_order_free_where_live_planning_is_not` is the same defect at the
    /// level of the rule; this is it at the level of the schedule.
    ///
    /// **An earlier version of this mutation did not fire**, and the reason is worth keeping:
    /// it merely shifted the refresh boundary by one sweep while still giving each halo
    /// exactly `n` sweeps of use, so the horizon invariant was preserved and the answer was
    /// correct. A defect that preserves the invariant is not a defect. That near-miss is why
    /// this enum's variants are each checked to fire rather than assumed to.
    HaloReadsLivePeers,

    /// **M3** — skip the halo refresh entirely on one exchange, so the shard steps on stale
    /// neighbour data.
    HaloRefreshSkipped,

    /// **M4** — the `SANDBOX_4090` §6 finding, re-run in the mesh: both sides of a boundary
    /// transfer apply twice. Perfectly conserved, perfectly antisymmetric, and wrong — every
    /// balance-based leg passes and only re-planning sees it.
    DoubleTransferBothSides,

    /// **M5** — allocate a halo one cell SHALLOWER than `Core/Locality.lean`'s `n·r` bound.
    /// The bound says `n·r` suffices; this asks whether it is tight, and the answer is a
    /// measurement rather than a citation.
    HaloOneShallowerThanHorizon,

    /// **M6** — orient a boundary pair by which shard reached it first instead of by global
    /// cell index, so the transfer's sign depends on the schedule.
    PairOrientedByVisitOrder,
}

impl Mutation {
    pub const fn is_none(self) -> bool {
        matches!(self, Mutation::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_visit_order_is_a_permutation() {
        for n in [0usize, 1, 2, 3, 7, 12, 33, 64] {
            for order in [VisitOrder::Natural, VisitOrder::Reversed, VisitOrder::Strided] {
                let seq = order.sequence(n);
                assert_eq!(seq.len(), n, "{order:?} at {n} lost items");
                let mut sorted = seq.clone();
                sorted.sort_unstable();
                assert_eq!(sorted, (0..n).collect::<Vec<_>>(), "{order:?} at {n}");
            }
        }
    }

    /// A "permutation" that is secretly the identity would make every reorder test vacuous.
    #[test]
    fn strided_is_neither_the_identity_nor_the_reverse() {
        let n = 16;
        let strided = VisitOrder::Strided.sequence(n);
        assert_ne!(strided, VisitOrder::Natural.sequence(n));
        assert_ne!(strided, VisitOrder::Reversed.sequence(n));
    }
}
