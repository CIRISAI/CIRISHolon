//! Typed failures. Library code never panics on a conservation violation: every gate leg
//! returns one of these so a host can log, retry, or halt.

use core::fmt;

use ciris_sim_core::holon::HolonError;
use ciris_sim_core::regplus::GrossState;

use crate::ledger::LedgerDelta;

/// Which end of a boundary pair. `Lo` is the shard with the smaller index; the pair's
/// canonical orientation is `(lo, hi)` with `lo < hi`, so the sign convention does not
/// depend on which side happens to be visited first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Lo,
    Hi,
}

impl Side {
    pub const fn index(self) -> usize {
        match self {
            Side::Lo => 0,
            Side::Hi => 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwarmError {
    /// A misconfigured swarm specification (bad shard count, bad topology, empty shard).
    Config(&'static str),
    /// `ciris-sim-core` rejected an arena.
    Core(HolonError),

    /// Checked integer arithmetic refused a ledger write. Gate leg: arithmetic.
    LedgerOverflow {
        round: u64,
        shard: usize,
        holon: u32,
    },
    /// The global sum over shard roots could not be represented. Gate leg: arithmetic.
    GlobalLedgerOverflow { round: u64 },

    /// **L1** — a shard's internal step changed its own root ledger. Internal work must
    /// redistribute within the shard, never mint or burn.
    LocalStepNotConserving {
        round: u64,
        shard: usize,
        before: GrossState,
        after: GrossState,
    },
    /// **L2** — the two receipts for one boundary pair are not exact negatives, so the
    /// transfer was not a paired debit/credit.
    PairNotAntisymmetric {
        round: u64,
        pair: usize,
        lo: LedgerDelta,
        hi: LedgerDelta,
    },
    /// **L3** — a receipt disagrees with the transfer independently re-planned from the
    /// published round snapshot. This is the leg that catches a *symmetric* corruption
    /// (both sides doubled, both sides dropped), which every ledger-sum check passes.
    ReceiptDoesNotMatchPlan {
        round: u64,
        pair: usize,
        side: Side,
        planned: LedgerDelta,
        receipted: LedgerDelta,
    },
    /// **L4** — a shard's root ledger moved by something other than the sum of the
    /// receipts it issued.
    ApplyInconsistent {
        round: u64,
        shard: usize,
        root_delta: LedgerDelta,
        receipt_sum: LedgerDelta,
    },
    /// **L5** — the global ledger summed over every shard root is not bit-identical to
    /// the pre-round global ledger.
    GlobalLedgerNotConserved {
        round: u64,
        expected: GrossState,
        observed: GrossState,
    },
    /// **L6** — inside one shard, a holon's ledger is no longer the exact sum of its
    /// resident children's ledgers.
    CompositionBroken {
        round: u64,
        shard: usize,
        holon: u32,
        declared: GrossState,
        composed: GrossState,
    },
    /// **L7** — the shard's arena failed `ciris-sim-core`'s own validator.
    ShardStructureInvalid {
        round: u64,
        shard: usize,
        source: HolonError,
    },
    /// A worker thread panicked. Reported rather than re-panicked.
    WorkerPanicked { threads: usize },
}

impl From<HolonError> for SwarmError {
    fn from(source: HolonError) -> Self {
        SwarmError::Core(source)
    }
}

impl fmt::Display for SwarmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SwarmError::Config(what) => write!(f, "invalid swarm configuration: {what}"),
            SwarmError::Core(e) => write!(f, "ciris-sim-core rejected the arena: {e:?}"),
            SwarmError::LedgerOverflow {
                round,
                shard,
                holon,
            } => write!(
                f,
                "round {round}: checked ledger arithmetic overflowed at shard {shard} holon {holon}"
            ),
            SwarmError::GlobalLedgerOverflow { round } => {
                write!(f, "round {round}: global ledger sum overflowed")
            }
            SwarmError::LocalStepNotConserving {
                round,
                shard,
                before,
                after,
            } => write!(
                f,
                "round {round}: L1 shard {shard} internal step changed its own root ledger \
                 {before:?} -> {after:?}"
            ),
            SwarmError::PairNotAntisymmetric {
                round,
                pair,
                lo,
                hi,
            } => write!(
                f,
                "round {round}: L2 boundary pair {pair} receipts are not opposite: \
                 lo={lo:?} hi={hi:?}"
            ),
            SwarmError::ReceiptDoesNotMatchPlan {
                round,
                pair,
                side,
                planned,
                receipted,
            } => write!(
                f,
                "round {round}: L3 boundary pair {pair} side {side:?} receipted \
                 {receipted:?} but the snapshot plans {planned:?}"
            ),
            SwarmError::ApplyInconsistent {
                round,
                shard,
                root_delta,
                receipt_sum,
            } => write!(
                f,
                "round {round}: L4 shard {shard} root moved by {root_delta:?} but issued \
                 receipts summing to {receipt_sum:?}"
            ),
            SwarmError::GlobalLedgerNotConserved {
                round,
                expected,
                observed,
            } => write!(
                f,
                "round {round}: L5 global ledger not conserved: expected {expected:?} \
                 observed {observed:?}"
            ),
            SwarmError::CompositionBroken {
                round,
                shard,
                holon,
                declared,
                composed,
            } => write!(
                f,
                "round {round}: L6 shard {shard} holon {holon} declares {declared:?} but its \
                 children compose to {composed:?}"
            ),
            SwarmError::ShardStructureInvalid {
                round,
                shard,
                source,
            } => write!(
                f,
                "round {round}: L7 shard {shard} arena rejected by ciris-sim-core: {source:?}"
            ),
            SwarmError::WorkerPanicked { threads } => {
                write!(f, "a worker thread panicked (thread count {threads})")
            }
        }
    }
}

impl std::error::Error for SwarmError {}
