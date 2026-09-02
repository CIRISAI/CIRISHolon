//! **The GPU device-class arm of the determinant solve** — the provider that builds
//! [`GpuLaneSigma`](crate::lanes::GpuLaneSigma) for `holon-chem`'s own Davidson, and the error
//! type every device refusal is spoken in.
//!
//! RESOURCE_DESIGN **D0**: the device class belongs to the ARTIFACT, not to the schedule. This
//! module is the device side of that rule. It implements `holon_chem::sigma_op::SigmaProvider`
//! from OUTSIDE `holon-chem`, which is the whole architecture: the chemistry crate names the
//! contract and stays portable enough to ship into a browser, and the CUDA dependency lives here,
//! out of the workspace, behind a build script that shells out to nvcc.
//!
//! # What is on the device and what is not
//!
//! The sigma is. The Davidson driver is not — the subspace, the Rayleigh–Ritz, the restart and
//! the preconditioner stay host-side in `holon_chem::tier::davidson_eigh_from_op_sub`, so one
//! engine runs under two devices with one argument swapped. Whether the driver's own vector
//! algebra should move device-side for spaces where the vectors are large is a measured
//! question (the lane referee at 16 million determinants is where it is asked), not one this
//! file answers.
//!
//! # Determinism is a property of the construction, then MEASURED
//!
//! The lane kernel is a gather with no reduction in it and no library call, compiled without
//! fused multiply-add: its bits are the host kernel's bits. That is the claim; the evidence is
//! `holon_chem::sigma_op::bit_identity_over_runs` and the host-vs-device `to_bits` comparison in
//! `tests/lanes_sigma.rs`, run on the operator that will actually be used.

use std::sync::Arc;

use cudarc::driver::{CudaContext, LaunchConfig};

use holon_chem::fci::{CiInts, FciSpace};
use holon_chem::lanes::LaneTables;
use holon_chem::sigma_op::{DeviceClass, SigmaOp, SigmaProvider};

use crate::lanes::GpuLaneSigma;

#[derive(Debug)]
pub enum FciGpuError {
    Driver(cudarc::driver::DriverError),
    /// The operator's footprint against what the card reports free, BEFORE anything is
    /// allocated. A refusal, never an attempt that fails partway.
    NotEnoughVram { need_bytes: u64, free_bytes: u64 },
}

impl From<cudarc::driver::DriverError> for FciGpuError {
    fn from(e: cudarc::driver::DriverError) -> Self {
        FciGpuError::Driver(e)
    }
}

impl std::fmt::Display for FciGpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FciGpuError::Driver(e) => write!(f, "cuda driver: {e}"),
            FciGpuError::NotEnoughVram { need_bytes, free_bytes } => write!(
                f,
                "this operator needs {:.1} MB of VRAM and the device reports {:.1} MB free. \
                 REFUSED rather than attempted: an allocation that fails partway leaves a \
                 sibling process on this card holding the consequence.",
                *need_bytes as f64 / 1e6,
                *free_bytes as f64 / 1e6
            ),
        }
    }
}

impl std::error::Error for FciGpuError {}

/// The device-bound source of operators. One per context; each `op_for` builds and uploads the
/// tables for one integral set.
pub struct GpuSigmaProvider {
    ctx: Arc<CudaContext>,
}

impl GpuSigmaProvider {
    /// Open the device. **REFUSES rather than falling back** (D5): a driver that is present and
    /// a CUDA that is broken is a half-visible device, and quietly running on the host would
    /// report a completed run while nothing recorded that the registered path was never taken.
    pub fn new(ordinal: usize) -> Result<GpuSigmaProvider, FciGpuError> {
        let ctx = CudaContext::new(ordinal)?;
        Ok(GpuSigmaProvider { ctx })
    }

    pub fn context(&self) -> &Arc<CudaContext> {
        &self.ctx
    }

    /// Free and total device memory, in bytes. Used for the pre-allocation check and by the
    /// VRAM probe.
    pub fn mem_info(&self) -> Result<(usize, usize), FciGpuError> {
        self.ctx.bind_to_thread()?;
        Ok(cudarc::driver::result::mem_get_info()?)
    }

    /// **How many device operators of these tables the card can hold** with `reserve_mib` held
    /// back for whatever else runs on it.
    ///
    /// Derived, not declared: the footprint is [`GpuLaneSigma::bytes_for`] — the same arithmetic
    /// the operator's own pre-allocation check uses — and free VRAM is a live reading. Returns 0
    /// when not even one fits, which a caller must treat as a REFUSAL of device-class work for
    /// that space, never as a reason to fall back to the host (D4). The reserve has no default
    /// because the right reserve depends on what else is running.
    pub fn max_workers_for(&self, tables: &LaneTables<f64>, reserve_mib: u64) -> Result<usize, FciGpuError> {
        let per_worker = GpuLaneSigma::bytes_for(tables);
        let (free, _total) = self.mem_info()?;
        let usable = (free as u64).saturating_sub(reserve_mib.saturating_mul(1 << 20));
        Ok((usable / per_worker.max(1)) as usize)
    }

    /// Build the device operator for one integral set on a chemistry space, checking VRAM FIRST.
    pub fn build(&self, space: &FciSpace, ci: &CiInts) -> Result<GpuLaneSigma, FciGpuError> {
        GpuLaneSigma::new(&self.ctx, &LaneTables::for_ci(space, ci), 0)
    }
}

impl SigmaProvider for GpuSigmaProvider {
    fn device(&self) -> DeviceClass {
        DeviceClass::Gpu
    }

    fn op_for<'a>(&self, space: &'a FciSpace, ci: &'a CiInts) -> Result<Box<dyn SigmaOp<f64> + 'a>, String> {
        // The operator owns its device memory, so it does not borrow from `space`/`ci` — but
        // the trait's lifetime allows a host operator to borrow, and this one simply does not
        // need to.
        self.build(space, ci)
            .map(|op| Box::new(op) as Box<dyn SigmaOp<f64> + 'a>)
            .map_err(|e| e.to_string())
    }
}

/// The kernels are grid-stride, so the launch SHAPE cannot reach the answer — every thread sums
/// over a fixed index range in a fixed order whatever the grid is. That is what makes the block
/// and grid counts a scheduling choice rather than part of the artifact, and it is the same
/// argument `holon-gpu`'s fold makes for its reduction.
pub(crate) fn grid_for(total: i64) -> LaunchConfig {
    const BLOCK: u32 = 256;
    let want = ((total as u64).div_ceil(BLOCK as u64)) as u32;
    LaunchConfig {
        grid_dim: (want.clamp(1, 4096), 1, 1),
        block_dim: (BLOCK, 1, 1),
        shared_mem_bytes: 0,
    }
}
