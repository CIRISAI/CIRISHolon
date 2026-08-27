//! The exchange protocol: a signed, paired transfer of conserved integer ledger quantity
//! across one boundary relation.
//!
//! # Why the global ledger is conserved *by construction*
//!
//! A boundary pair is oriented `(lo, hi)` with `lo < hi` — a canonical order fixed at
//! construction from the shard indices, not from visit order. One transfer `d` is planned
//! for the pair and applied as `lo -= d` and `hi += d`. The global sum therefore changes
//! by `(-d) + (+d) = 0` in every one of the four integer lanes, for every pair,
//! independently. No repair step, no tolerance, no residual: `GrossState` equality is
//! bit-equality on four integers.
//!
//! # Why the round is order-independent — TWO independent reasons, both live
//!
//! **(1) Snapshot, then apply.** Every transfer in a round is a pure function of the
//! *published snapshot* of the two ports, taken after every shard's local step and before
//! any transfer is applied. No transfer can observe another transfer's write, so there is
//! no read-after-write hazard and the plan for the round is a fixed value, independent of
//! which shard is stepped first or which thread wins a race. This is the load-bearing
//! reason, because it survives generalisation: it holds even if a port served several
//! links at once.
//!
//! **(2) Disjoint pairs.** In this prototype each boundary link owns its *own* port holon
//! (`Shard::new` enforces one distinct port per link), so the set of holons written by
//! pair `p` is disjoint from the set written by pair `q != p`, except for the shard roots,
//! which accumulate by `checked_add` — associative and commutative. Disjointness is what
//! makes the apply phase safely parallel with no locking at all.
//!
//! Reason (1) alone would suffice for determinism; reason (2) alone would suffice for
//! *this* topology. Both are stated because the GPU design needs (1) to survive a port
//! shared by several neighbours, and needs (2) to keep the apply kernel lock-free. A unit
//! test in this module exhibits the failure that dropping (1) would cause: with a shared
//! port, a live-read plan depends on pair visit order; the snapshot plan does not.

use crate::error::{Side, SwarmError};
use crate::ledger::{LedgerDelta, LANES};

/// A boundary relation between two shards, held as plain indices. Not a holon, not a new
/// entity class — a value naming two `(shard, holon)` endpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundaryPair {
    pub id: usize,
    /// `(shard, port holon)` of the debited side. `lo.0 < hi.0` always.
    pub lo: (usize, u32),
    /// `(shard, port holon)` of the credited side.
    pub hi: (usize, u32),
}

/// The transfer rule.
///
/// Diffusive halving of the port imbalance, in integer arithmetic, computed from the
/// snapshot lanes of the two ports. Returned as "debit `lo`, credit `hi`".
///
/// Non-negativity of the count lanes is guaranteed without any clamp, and this is worth
/// spelling out because it is what makes the checked arithmetic quiet in the happy path:
/// with `lo, hi >= 0`, `d = (lo - hi) / 2` satisfies `0 <= d <= lo/2` when `lo >= hi`
/// (so `lo` can afford the debit), and `-(hi/2) <= d < 0` otherwise (so `hi` can afford
/// it). The root of a shard holds at least as much as any single leaf, so the mirrored
/// root write is safe by the same argument. Constituents halve at `/4` for the same
/// reason with more headroom.
#[inline]
pub fn plan_transfer(lo: [i64; LANES], hi: [i64; LANES]) -> Option<LedgerDelta> {
    let half = |a: i64, b: i64, divisor: i64| -> Option<i64> {
        i64::try_from((i128::from(a) - i128::from(b)) / i128::from(divisor)).ok()
    };
    Some(LedgerDelta {
        constituents: half(lo[0], hi[0], 4)?,
        occupancy: half(lo[1], hi[1], 2)?,
        momentum: [half(lo[2], hi[2], 2)?, half(lo[3], hi[3], 2)?],
    })
}

/// Deliberate corruption of the exchange, for mutation-testing the conservation gate.
///
/// A gate that cannot fail proves nothing. This enum is the fault-injection facility that
/// lets the test suite prove each gate leg *can* fail, and — just as important — find out
/// which legs are blind to which corruption. It is public because the GPU port will want
/// to run the same fault set against the device implementation.
///
/// Production callers pass [`FaultInjection::None`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FaultInjection {
    #[default]
    None,
    /// The credited side applies the transfer; the debited side issues a receipt but
    /// writes nothing. Breaks conservation in the most direct way.
    CreditWithoutDebit { pair: usize },
    /// The credited side writes the transfer twice but receipts it once.
    DoubleApplyOneSide { pair: usize },
    /// *Both* sides write and receipt twice. The global sum stays perfectly conserved and
    /// the receipts stay perfectly antisymmetric; only plan conformance can see it.
    DoubleApplyBothSides { pair: usize },
    /// The transfer is silently skipped on both sides, receipts zero. Conservative,
    /// antisymmetric, and wrong.
    DropTransfer { pair: usize },
    /// The debited side credits instead of debiting. Both sides now credit.
    SwapSignOnLowSide { pair: usize },
    /// Push `i64::MAX` into the credited side's momentum accumulator.
    OverflowMomentum { pair: usize },
    /// The shard's internal step mints one unit of occupancy out of nothing, updating the
    /// leaf and the root together so internal composition still holds.
    MintInLocalStep { shard: usize },
    /// After the exchange, add one unit of occupancy to a leaf only — the parent is not
    /// updated, so the shard's internal composition is broken while its root, and hence
    /// the global sum, is untouched.
    BreakComposition { shard: usize },
    /// After the exchange, add one unit of occupancy to the root only.
    RootOnlyCredit { shard: usize },
}

impl FaultInjection {
    pub const fn is_none(self) -> bool {
        matches!(self, FaultInjection::None)
    }
}

/// What one side of a pair actually writes, and what it reports having written.
///
/// Splitting "applied" from "receipted" is the whole trick that makes the gate
/// mutation-resistant: a corruption that changes only the write is caught by the
/// apply-consistency leg, a corruption that changes only the report is caught by plan
/// conformance, and a corruption that changes both consistently is *still* caught by plan
/// conformance because the plan is re-derived from the snapshot, not from the report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SideWrite {
    pub applied: LedgerDelta,
    pub receipted: LedgerDelta,
}

/// Resolve one side of a planned transfer into a write and a receipt, honouring any
/// injected fault.
pub fn resolve_side(
    plan: LedgerDelta,
    pair: usize,
    side: Side,
    fault: FaultInjection,
) -> Result<SideWrite, SwarmError> {
    let signed = match side {
        Side::Lo => plan
            .checked_neg()
            .ok_or(SwarmError::Config("transfer negation overflowed"))?,
        Side::Hi => plan,
    };
    let honest = SideWrite {
        applied: signed,
        receipted: signed,
    };
    let doubled = || {
        signed
            .checked_mul(2)
            .ok_or(SwarmError::Config("doubled transfer overflowed"))
    };

    Ok(match (fault, side) {
        (FaultInjection::CreditWithoutDebit { pair: p }, Side::Lo) if p == pair => SideWrite {
            applied: LedgerDelta::ZERO,
            receipted: signed,
        },
        (FaultInjection::DoubleApplyOneSide { pair: p }, Side::Hi) if p == pair => SideWrite {
            applied: doubled()?,
            receipted: signed,
        },
        (FaultInjection::DoubleApplyBothSides { pair: p }, _) if p == pair => {
            let d = doubled()?;
            SideWrite {
                applied: d,
                receipted: d,
            }
        }
        (FaultInjection::DropTransfer { pair: p }, _) if p == pair => SideWrite {
            applied: LedgerDelta::ZERO,
            receipted: LedgerDelta::ZERO,
        },
        (FaultInjection::SwapSignOnLowSide { pair: p }, Side::Lo) if p == pair => SideWrite {
            applied: plan,
            receipted: plan,
        },
        (FaultInjection::OverflowMomentum { pair: p }, Side::Hi) if p == pair => {
            let mut applied = signed;
            applied.momentum[0] = i64::MAX;
            SideWrite {
                applied,
                receipted: signed,
            }
        }
        _ => honest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_transfer_is_exactly_antisymmetric_in_every_lane() {
        let lo = [40, 31, -7, 12];
        let hi = [8, 9, 5, -4];
        let d = plan_transfer(lo, hi).unwrap();
        // lo pays d, hi receives d: the pair's contribution to the global sum is zero.
        let net = d.checked_neg().unwrap().checked_add(d).unwrap();
        assert_eq!(net, LedgerDelta::ZERO);
        assert_eq!(d.occupancy, 11);
        assert_eq!(d.constituents, 8);
        assert_eq!(d.momentum, [-6, 8]);
    }

    #[test]
    fn the_transfer_never_overdraws_either_side() {
        for lo in 0..64i64 {
            for hi in 0..64i64 {
                let d = plan_transfer([lo, lo, 0, 0], [hi, hi, 0, 0]).unwrap();
                assert!(lo - d.occupancy >= 0, "lo overdrawn: {lo} {hi} {d:?}");
                assert!(hi + d.occupancy >= 0, "hi overdrawn: {lo} {hi} {d:?}");
                assert!(lo - d.constituents >= 0);
                assert!(hi + d.constituents >= 0);
            }
        }
    }

    /// The order-independence claim, made testable at the level of the rule.
    ///
    /// A port shared by two links is the case reason (2) — disjointness — does not cover.
    /// Planning from a snapshot is order-independent there anyway; planning from live
    /// values is not. This exhibits the difference, so the doc comment above is a claim
    /// with a witness rather than an assertion.
    #[test]
    fn snapshot_planning_is_order_free_where_live_planning_is_not() {
        // One shared port S with 40 units, two peers A (8 units) and B (0 units).
        let shared = [40, 40, 0, 0];
        let peer_a = [8, 8, 0, 0];
        let peer_b = [0, 0, 0, 0];

        // Snapshot plan: both transfers read the same published value for S.
        let snap_a = plan_transfer(shared, peer_a).unwrap();
        let snap_b = plan_transfer(shared, peer_b).unwrap();
        assert_eq!(snap_a.occupancy, 16);
        assert_eq!(snap_b.occupancy, 20);
        // Reversing the visit order changes nothing: the inputs are values, not state.
        assert_eq!(plan_transfer(shared, peer_b).unwrap(), snap_b);
        assert_eq!(plan_transfer(shared, peer_a).unwrap(), snap_a);

        // Live plan, A first: S drops to 24 before B is planned.
        let live_a = plan_transfer(shared, peer_a).unwrap();
        let s_after_a = [
            shared[0] - live_a.constituents,
            shared[1] - live_a.occupancy,
            0,
            0,
        ];
        let live_b_second = plan_transfer(s_after_a, peer_b).unwrap();
        // Live plan, B first.
        let live_b = plan_transfer(shared, peer_b).unwrap();
        let s_after_b = [
            shared[0] - live_b.constituents,
            shared[1] - live_b.occupancy,
            0,
            0,
        ];
        let live_a_second = plan_transfer(s_after_b, peer_a).unwrap();

        assert_ne!(live_b_second, live_b, "live planning depends on order");
        assert_ne!(live_a_second, live_a, "live planning depends on order");
    }

    #[test]
    fn honest_resolution_is_the_plan_with_opposite_signs() {
        let plan = LedgerDelta {
            constituents: 1,
            occupancy: 6,
            momentum: [-3, 4],
        };
        let lo = resolve_side(plan, 0, Side::Lo, FaultInjection::None).unwrap();
        let hi = resolve_side(plan, 0, Side::Hi, FaultInjection::None).unwrap();
        assert_eq!(lo.applied, plan.checked_neg().unwrap());
        assert_eq!(hi.applied, plan);
        assert_eq!(lo.applied, lo.receipted);
        assert_eq!(hi.applied, hi.receipted);
        assert_eq!(lo.applied.checked_add(hi.applied).unwrap(), LedgerDelta::ZERO);
    }

    #[test]
    fn a_fault_aimed_at_another_pair_leaves_this_pair_honest() {
        let plan = LedgerDelta {
            occupancy: 6,
            ..LedgerDelta::ZERO
        };
        let write = resolve_side(plan, 3, Side::Lo, FaultInjection::DropTransfer { pair: 7 })
            .unwrap();
        assert_eq!(write.applied, plan.checked_neg().unwrap());
    }
}
