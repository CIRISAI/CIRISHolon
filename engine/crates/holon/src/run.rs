//! THE PRODUCTION PATH — structural cash-in of every landed gain.
//!
//! One entry point per task; the defaults ARE the optimizations. No caller
//! chooses between "fast" and "correct" variants, because there is only the
//! certified fast path: packed planes for tier 1, block-deduplicated branch
//! sums for the magic tier (measured 27–1350× over naive, EXACT), and
//! shard-invariant mesh folds through the one merge law. holon-qasm remains
//! the frozen referee; this module is what runs.

use crate::mesh;
use crate::prune::{run_pruned, Gate, PruneConfig, PrunedSum};
use crate::BranchSource;
use crate::ledger::Cyc;

// NOTE: `PrunedSum` already implements `BranchSource` in prune.rs — the
// lanes converged on the one contract independently, which is the DRY law
// doing its job. This module only composes.

/// Default pipeline configuration: aggressive block dedup, full merge
/// verification, refusal past the working-set cap (never silent growth).
pub fn default_config() -> PruneConfig {
    PruneConfig::default()
}

/// Exact amplitude of one basis state — the production path: prune-dedup the
/// branch space, then shard-fold across available parallelism.
pub fn amplitude(n: usize, gates: &[Gate], y: &[bool]) -> Cyc {
    let sum = run_pruned(n, gates, &default_config());
    let shards = std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1);
    mesh::fold_amplitude(&sum, y, shards.min(sum.branches.len().max(1)))
}

/// The same, with explicit shard count (the mesh determinism tests exercise
/// shard-invariance; production picks parallelism, correctness never moves).
pub fn amplitude_sharded(n: usize, gates: &[Gate], y: &[bool], shards: usize) -> Cyc {
    let sum = run_pruned(n, gates, &default_config());
    mesh::fold_amplitude(&sum, y, shards.max(1))
}
