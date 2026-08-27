//! THE PRODUCTION PATH — structural cash-in of every landed gain.
//!
//! One entry point per task; the defaults ARE the optimizations. No caller
//! chooses between "fast" and "correct" variants, because there is only the
//! certified fast path: packed planes for tier 1, block-deduplicated branch
//! sums for the magic tier (measured 27–1350× over naive, EXACT), and
//! shard-invariant mesh folds through the one merge law. holon-qasm remains
//! the frozen referee; this module is what runs.

use crate::mesh;
use crate::prune::{run_pruned, Gate, PruneConfig};
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

/// The residue-carried amplitude: the ring scaled to the circuit, NO
/// coefficient envelope anywhere in the fold. This is not a fallback and not
/// a workaround — it is the object recursing (each prime a child holon under
/// the one merge law; see `residue.rs`). Use it when the router's bound says
/// the direct ring is too narrow — or always, if latency is not the point.
pub fn amplitude_unbounded(
    n: usize,
    gates: &[Gate],
    y: &[bool],
    shards: usize,
) -> crate::residue::ResReading {
    let sum = run_pruned(n, gates, &default_config());
    let k = crate::residue::primes_for_bits(ring_bits_bound(gates, sum.branches.len() as u64));
    crate::residue::fold_amplitude_res(&sum, y, shards.max(1), k).reconstruct()
}

/// Conservative coefficient-bit bound for the fold: per-branch lane headroom,
/// plus alignment growth (m spread ≤ one per gate, shift = Δ/2 bits), plus
/// the log of the branch count. Deliberately generous — primes are cheap and
/// the bound only picks how many children carry the value.
pub fn ring_bits_bound(gates: &[Gate], n_branches: u64) -> usize {
    64 + gates.len() / 2 + (64 - n_branches.leading_zeros() as usize) + 8
}

/// The width router: direct i128 ring when the bound fits (fast lane, its
/// refusal now unreachable by construction), residue carrier otherwise. The
/// enum is the honest return type — the exact value either fits the direct
/// ring or arrives as a reconstructed reading; nothing is capped.
pub enum Amplitude {
    Direct(Cyc),
    Residue(crate::residue::ResReading),
}

pub fn amplitude_auto(n: usize, gates: &[Gate], y: &[bool], shards: usize) -> Amplitude {
    let sum = run_pruned(n, gates, &default_config());
    let bits = ring_bits_bound(gates, sum.branches.len() as u64);
    if bits < 120 {
        Amplitude::Direct(mesh::fold_amplitude(&sum, y, shards.max(1)))
    } else {
        let k = crate::residue::primes_for_bits(bits);
        Amplitude::Residue(
            crate::residue::fold_amplitude_res(&sum, y, shards.max(1), k).reconstruct(),
        )
    }
}
