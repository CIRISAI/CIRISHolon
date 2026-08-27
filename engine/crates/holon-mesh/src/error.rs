//! Failures the mesh can report. Every one names where it happened; none is a bare `bool`.

use ciris_sim_core::holon::HolonError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MeshError {
    Config(&'static str),
    Core(HolonError),
    /// A ledger lane left the range the cross-host `u64`/`i64` REG+ ledger can represent.
    /// Refused, never wrapped.
    LedgerRange {
        cell: u32,
    },
    /// A shard's owned total moved during a sweep that only redistributes. Minting or
    /// burning, caught locally.
    ShardMinted {
        step: usize,
        shard: usize,
    },
    /// The scene's total moved. Conservation broken globally.
    SceneMinted {
        step: usize,
    },
    /// A shard read a cell further from its owned set than `Core/Locality.lean`'s `n·r`
    /// horizon permits.
    HorizonExceeded {
        shard: usize,
        depth: usize,
        bound: usize,
    },
    /// The live overlay stopped composing: a root is not the exact sum of its cells.
    CompositionBroken {
        step: usize,
        shard: usize,
    },
    WorkerPanicked {
        threads: usize,
    },
}

impl From<HolonError> for MeshError {
    fn from(source: HolonError) -> Self {
        Self::Core(source)
    }
}
