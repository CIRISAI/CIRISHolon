//! holon-gpu — THE SAME FOLD, ON CUDA.
//!
//! `holon::merge` states the one law: ledgers fold associatively and
//! commutatively, so any sharding, ordering, or distribution of a fold is
//! deterministic without coordination. `holon::mesh` is that law cut across OS
//! threads. This crate is that law cut across a GPU, and the point of it is
//! that the GPU changes nothing about the argument — a warp schedule is just
//! another sharding, and the ledger does not know it happened.
//!
//! What the GPU DOES change is what "deterministic" costs. The mesh's own
//! header is careful that exact `Z[omega]` addition gives order-independence of
//! the VALUE, and order-independence of the REPRESENTATION only when no partial
//! sum cancels to zero. A device reduction has far more partial sums than a
//! five-way shard, so this crate takes the ring out of the reduction instead of
//! hoping: the host aligns the whole batch to one denominator exponent, and the
//! device sums `i128` coefficient lanes. Two's-complement integer addition is
//! associative and commutative unconditionally, so bit-identity across
//! block/grid configurations is a property of the arithmetic and not a
//! measurement that came out lucky. It is measured anyway — that is
//! `tests/determinism.rs`.
//!
//! ```text
//!   AffineDesc  <- Affine::canon_key + gamma        (desc.rs, decode + CPU twin)
//!   GpuBatch    <- aligned to one exponent, uploaded (gpu.rs)
//!   fold(y)     -> Cyc, exact, schedule-independent
//! ```
//!
//! The reference in every test is `holon::mesh::fold_amplitude` over the same
//! `BranchSource` — the CPU fold this mirrors, not a hand-rolled expectation.
//!
//! ISOLATION: this crate is outside the workspace and carries its own empty
//! `[workspace]` table; `ci-gates.sh` cannot reach it, deliberately, because its
//! tests need a CUDA device. See `Cargo.toml`'s header.

pub mod cpu;
pub mod desc;
/// The device-class provider for `holon-chem`'s determinant solve (RESOURCE_DESIGN
/// D0) and its error type. The dependency runs this way round because `holon-chem`
/// ships into a browser and cannot take CUDA: it names the contract, this crate
/// satisfies it. The operator itself is `lanes::GpuLaneSigma`.
pub mod fci;
pub mod gpu;
/// The lane sigma on the device: `holon_chem::lanes::sigma_det` transliterated, bit-identical
/// to the host shards by construction and by gate.
pub mod lanes;
/// The GPU as a LEASABLE resource: VRAM leased through `holon-resource` with its
/// quantitative boundary declared (D3b), and a device that vanishes under a live
/// lease CONVICTED rather than swallowed (D9).
pub mod lease;
/// The VRAM probe `holon-resource` deliberately does not have — D2's *attempt the
/// thing*, in the device's own vocabulary.
pub mod probe;
pub mod ring;
pub mod synth;

pub use desc::{AffineDesc, DescError, DescSource};
pub use fci::{FciGpuError, GpuSigmaProvider};
pub use gpu::{GpuBatch, GpuError, GpuFolder, Shape};
pub use lanes::GpuLaneSigma;
pub use lease::{LeasedGpuError, LeasedGpuProvider, LeasedGpuSigma, VramCompetitor, VramLease};
pub use probe::{ReportedFreeProbe, VramProbe};

/// `/proc/loadavg`'s first field, or `f64::NAN` where it is not readable.
///
/// Every CPU timing this crate reports carries one of these. The machine it was
/// measured on runs other campaigns, so a CPU number without the load it was
/// taken under is not a measurement, it is an anecdote.
pub fn loadavg() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(f64::NAN)
}
