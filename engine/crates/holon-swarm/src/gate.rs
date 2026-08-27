//! The conservation gate: seven legs, each independently mutation-tested.
//!
//! The gate exists because "the ledger is conserved by construction" is a claim about the
//! code as written, and the code as written is exactly what a GPU port will replace. So
//! the gate re-derives the invariant from the observable state after every exchange round
//! and refuses to take the exchange's word for anything.
//!
//! | leg | what it re-derives | blind to |
//! |-----|--------------------|----------|
//! | L1 | a shard's internal step left its own root ledger untouched | anything at the boundary |
//! | L2 | the two receipts of a pair are exact negatives | a *symmetric* corruption |
//! | L3 | each receipt equals the transfer re-planned from the published snapshot | nothing in the exchange; blind to writes that do not match their receipt |
//! | L4 | a shard's root moved by exactly the sum of the receipts it issued | a corruption that also rewrites the receipt |
//! | L5 | the global sum over roots is bit-identical to the opening balance | any redistribution that nets to zero |
//! | L6 | inside each shard, parent == exact sum of resident children (live overlay) | anything that keeps composition |
//! | L7 | the arena passes `ciris-sim-core`'s own validator | **the live ledger overlay entirely** (structural form) |
//!
//! The row that matters most is L3. A global-sum gate — the obvious design — passes a
//! corruption that doubles both sides of a transfer, or drops both sides: those are still
//! perfectly conserved, just not the exchange that was specified. Re-planning from the
//! snapshot is the only leg that can see it, and the mutation suite proves that by showing
//! L1/L2/L4/L5/L6 all pass on `DoubleApplyBothSides` while L3 fires.
//!
//! The row that is most honest about its limits is L7. `RuntimeArena::validate()` reads
//! the gross states stored in the arena headers, and the exchange writes the *overlay*
//! (see `crate::shard`). The structural form of L7 therefore cannot see a ledger
//! corruption at all; `GateLevel::Paranoid` rebuilds an arena from the live overlay so
//! the core's own validator judges current values. Both facts are pinned by tests.

use ciris_sim_core::regplus::GrossState;

use crate::error::{Side, SwarmError};
use crate::exchange::{plan_transfer, BoundaryPair};
use crate::ledger::{delta_between, lanes_to_gross, LedgerDelta, LANES};
use crate::shard::Shard;

/// How much of the gate to run. Every level runs L1–L5; the levels differ in how much
/// per-holon work they add on top. Reported costs for all three are in the bench binary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GateLevel {
    /// L1–L5. `O(shards + pairs)` — independent of holon count.
    Ledger,
    /// L1–L6 plus the structural form of L7. `O(holons)`, no allocation on the ledger legs.
    #[default]
    Full,
    /// L1–L6 plus the rebuild form of L7: the live overlay is pushed back through
    /// `RuntimeArena::from_specs`, which runs the core's `validate()`. `O(holons)` with
    /// two allocations per shard per round.
    Paranoid,
}

impl GateLevel {
    pub const fn checks_composition(self) -> bool {
        !matches!(self, GateLevel::Ledger)
    }
}

/// Holds the swarm's opening global balance and re-derives conservation against it.
#[derive(Clone, Copy, Debug)]
pub struct ConservationGate {
    baseline: GrossState,
    level: GateLevel,
}

impl ConservationGate {
    pub fn new(baseline: GrossState, level: GateLevel) -> Self {
        Self { baseline, level }
    }

    pub fn baseline(&self) -> GrossState {
        self.baseline
    }

    pub fn level(&self) -> GateLevel {
        self.level
    }
}

/// **L1** — the internal step redistributes, it does not mint or burn.
pub fn leg1_local_conservation(
    round: u64,
    shard: usize,
    before: GrossState,
    after: GrossState,
) -> Result<(), SwarmError> {
    if before == after {
        Ok(())
    } else {
        Err(SwarmError::LocalStepNotConserving {
            round,
            shard,
            before,
            after,
        })
    }
}

/// **L2** — the two receipts of a pair are exact negatives in every lane.
pub fn leg2_pair_antisymmetry(
    round: u64,
    pair: usize,
    lo: LedgerDelta,
    hi: LedgerDelta,
) -> Result<(), SwarmError> {
    let net = lo.checked_add(hi);
    if net == Some(LedgerDelta::ZERO) {
        Ok(())
    } else {
        Err(SwarmError::PairNotAntisymmetric {
            round,
            pair,
            lo,
            hi,
        })
    }
}

/// **L3** — the receipt equals the transfer re-planned from the published snapshot.
pub fn leg3_plan_conformance(
    round: u64,
    pair: usize,
    side: Side,
    planned: LedgerDelta,
    receipted: LedgerDelta,
) -> Result<(), SwarmError> {
    if planned == receipted {
        Ok(())
    } else {
        Err(SwarmError::ReceiptDoesNotMatchPlan {
            round,
            pair,
            side,
            planned,
            receipted,
        })
    }
}

/// **L4** — the shard's root moved by exactly the sum of the receipts it issued.
pub fn leg4_apply_consistency(
    round: u64,
    shard: usize,
    root_delta: LedgerDelta,
    receipt_sum: LedgerDelta,
) -> Result<(), SwarmError> {
    if root_delta == receipt_sum {
        Ok(())
    } else {
        Err(SwarmError::ApplyInconsistent {
            round,
            shard,
            root_delta,
            receipt_sum,
        })
    }
}

/// **L5** — the global ledger is bit-identical to the opening balance.
pub fn leg5_global_conservation(
    round: u64,
    expected: GrossState,
    observed: GrossState,
) -> Result<(), SwarmError> {
    if expected == observed {
        Ok(())
    } else {
        Err(SwarmError::GlobalLedgerNotConserved {
            round,
            expected,
            observed,
        })
    }
}

/// Run every shard-local leg the level asks for: L4, L6, L7.
pub fn shard_local_legs(
    gate: &ConservationGate,
    round: u64,
    shard: &Shard,
) -> Result<(), SwarmError> {
    let root_delta = delta_between(shard.root_ledger(), shard.post_local).ok_or(
        SwarmError::LedgerOverflow {
            round,
            shard: shard.index(),
            holon: 0,
        },
    )?;
    let mut receipt_sum = LedgerDelta::ZERO;
    for receipt in &shard.receipts {
        receipt_sum = receipt_sum
            .checked_add(*receipt)
            .ok_or(SwarmError::LedgerOverflow {
                round,
                shard: shard.index(),
                holon: 0,
            })?;
    }
    leg4_apply_consistency(round, shard.index(), root_delta, receipt_sum)?;

    match gate.level() {
        GateLevel::Ledger => {}
        GateLevel::Full => {
            shard.check_composition(round)?;
            shard.validate_structure(round)?;
        }
        GateLevel::Paranoid => {
            shard.check_composition(round)?;
            shard.revalidate_through_core(round)?;
        }
    }
    Ok(())
}

/// Run the cross-shard legs: L2, L3 over every pair, and L5 over the summed roots.
///
/// `snapshot` and `receipts` are indexed `[pair * 2 + side]`; `roots` is indexed by shard.
/// The threaded path fills them from atomics, the sequential path from plain vectors —
/// the leg logic is shared so the two paths cannot drift apart.
pub fn cross_shard_legs(
    gate: &ConservationGate,
    round: u64,
    pairs: &[BoundaryPair],
    snapshot: &[[i64; LANES]],
    receipts: &[LedgerDelta],
    roots: &[[i64; LANES]],
) -> Result<(), SwarmError> {
    for pair in pairs {
        let lo = receipts[pair.id * 2 + Side::Lo.index()];
        let hi = receipts[pair.id * 2 + Side::Hi.index()];
        leg2_pair_antisymmetry(round, pair.id, lo, hi)?;

        let plan = plan_transfer(
            snapshot[pair.id * 2 + Side::Lo.index()],
            snapshot[pair.id * 2 + Side::Hi.index()],
        )
        .ok_or(SwarmError::GlobalLedgerOverflow { round })?;
        let debit = plan
            .checked_neg()
            .ok_or(SwarmError::GlobalLedgerOverflow { round })?;
        leg3_plan_conformance(round, pair.id, Side::Lo, debit, lo)?;
        leg3_plan_conformance(round, pair.id, Side::Hi, plan, hi)?;
    }

    let mut total = GrossState::ZERO;
    for lanes in roots {
        let gross =
            lanes_to_gross(*lanes).ok_or(SwarmError::GlobalLedgerOverflow { round })?;
        total = total
            .checked_combine(gross)
            .ok_or(SwarmError::GlobalLedgerOverflow { round })?;
    }
    leg5_global_conservation(round, gate.baseline(), total)
}
