//! A PHASE TIMER for the Clifford engines — MESH-CLIFFORD-1's profile.
//!
//! The results document deferred G3's window pending one question: where
//! does the wall GO, and which of those places is parallel under the column
//! cut? This answers it by accumulating monotonic wall time per phase, on the
//! calling thread, around whole phases — never a syscall per word, and a
//! vDSO `clock_gettime` pair (~40 ns) per gate at most. Off by default, and
//! off means one predictable branch: `start()` returns `None` and `stop()`
//! does nothing with it.
//!
//! Wall is attributed where it is SPENT on the calling thread. A threaded
//! region (a gate layer, a threaded rowsum) is one span of calling-thread
//! wall; inside a gate layer the span is split among single-qubit / in-shard
//! CX / crossing CX in proportion to the threads' own busy time on each, so
//! the phases still sum to wall and the split says which kind of work the
//! threads were doing while the caller waited.

use std::time::Instant;

/// The phases, in the order they are printed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Phase {
    /// Single-qubit gates (H, S, S†, X, Z), including the deferred resets.
    Gate1q = 0,
    /// CX with both columns in one shard (every CX when unsharded).
    CxLocal,
    /// CX across a shard boundary, INCLUDING the column exchange (gather and
    /// scatter copies on the calling thread).
    CxCross,
    /// The gate phase's own bookkeeping: buffering, layering, thread spawn
    /// and join not attributed to a kind, and the sign-partial fold.
    GateLayering,
    /// The determinism scan: "does any stabilizer anticommute with Z_q?",
    /// plus the destabilizer hit-set read — both column reads.
    Scan,
    /// Collapse rowsums on the calling thread (S = 1, and cascades too small
    /// to thread), plus the cascade's own setup (mask, row list, pivot copy).
    RowsumSerial,
    /// Collapse rowsums threaded across shards: the in-shard partials.
    RowsumPartial,
    /// ...and the parent's fold of the shard partials' phases.
    RowsumFold,
    /// Deterministic multi-term destabilizer products (row-major).
    DetProduct,
    /// Column→row transpose: materializing the row-major reference.
    TransposeC2R,
    /// Row→column transpose: the end-of-batch rebuild of the column engine.
    TransposeR2C,
    /// First-touch allocation of the row-major reference (once per run).
    ReferenceAlloc,
    /// The X-mirror patch after a collapse.
    MirrorPatch,
    /// The random-bit draw.
    Rng,
}

pub const N_PHASES: usize = 14;

pub const PHASE_NAMES: [&str; N_PHASES] = [
    "gate_1q",
    "cx_in_shard",
    "cx_cross_shard",
    "gate_layering",
    "scan",
    "rowsum_serial",
    "rowsum_partial",
    "rowsum_fold",
    "det_product",
    "transpose_col_to_row",
    "transpose_row_to_col",
    "reference_alloc",
    "mirror_patch",
    "rng",
];

/// Accumulated wall per phase, in nanoseconds, and a call count.
#[derive(Clone, Copy, Debug, Default)]
pub struct PhaseProfile {
    pub enabled: bool,
    pub ns: [u64; N_PHASES],
    pub calls: [u64; N_PHASES],
}

impl PhaseProfile {
    #[inline(always)]
    pub fn start(&self) -> Option<Instant> {
        if self.enabled {
            Some(Instant::now())
        } else {
            None
        }
    }

    /// Charge the time since `t` to `p`, and return a fresh `Instant` so a
    /// run of consecutive phases costs one clock read per boundary.
    #[inline(always)]
    pub fn stop(&mut self, p: Phase, t: Option<Instant>) -> Option<Instant> {
        match t {
            Some(t0) => {
                let now = Instant::now();
                self.ns[p as usize] += now.duration_since(t0).as_nanos() as u64;
                self.calls[p as usize] += 1;
                Some(now)
            }
            None => None,
        }
    }

    /// Charge an already-measured span (used to split a threaded region).
    #[inline(always)]
    pub fn add_ns(&mut self, p: Phase, ns: u64) {
        self.ns[p as usize] += ns;
        self.calls[p as usize] += 1;
    }

    pub fn seconds(&self, p: usize) -> f64 {
        self.ns[p] as f64 * 1e-9
    }

    pub fn total_seconds(&self) -> f64 {
        self.ns.iter().sum::<u64>() as f64 * 1e-9
    }
}
