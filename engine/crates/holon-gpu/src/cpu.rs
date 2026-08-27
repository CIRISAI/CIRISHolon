//! The tightest honest CPU arm, so the GPU's speedup is quoted against the best
//! CPU and not against the most convenient one.
//!
//! `holon::BranchSource::amplitude_of` takes `y: &[bool]`, so a packed
//! implementation re-packs `y` into a `u64` once PER BRANCH — an O(n) loop
//! sitting next to an O(n*k) solve, which at small `n` is a visible fraction of
//! the work and is an artifact of the trait boundary rather than of the
//! algorithm. Quoting a GPU speedup against that would be quoting it against a
//! baseline defect.
//!
//! [`fold_packed`] hoists the pack out of the branch loop and is otherwise the
//! same fold: `holon::mesh::shard_ranges` for the cut, `holon::merge::fold` for
//! both tiers of the accumulation, ascending branch order inside a shard,
//! shard-index order across them. It is the one merge law with `y` packed once,
//! and nothing else — no rayon, no atomics, no completion-order reduction.

use holon::ledger::Cyc;
use holon::merge::{self, MergeLedger};
use holon::mesh::shard_ranges;

use crate::desc::AffineDesc;

/// `Sigma_b coeff_b * <y|phi_b>` over `descs`, across `shards` OS threads.
///
/// `shards <= 1` runs on the calling thread with no spawn at all — the same
/// convention `holon::mesh::fold_amplitude` uses, so the serial arm is charged
/// no thread cost it does not pay.
pub fn fold_packed(descs: &[AffineDesc], y: u64, shards: usize) -> Cyc {
    let ranges = shard_ranges(descs.len() as u64, shards);
    match ranges.len() {
        0 => Cyc::empty(),
        1 => merge::fold(descs.iter().map(|d| d.amplitude(y))),
        n => {
            let mut partial = vec![Cyc::empty(); n];
            std::thread::scope(|scope| {
                for (slot, range) in partial.iter_mut().zip(ranges) {
                    let chunk = &descs[range.start as usize..range.end as usize];
                    scope.spawn(move || {
                        *slot = merge::fold(chunk.iter().map(|d| d.amplitude(y)));
                    });
                }
            });
            merge::fold(partial)
        }
    }
}
