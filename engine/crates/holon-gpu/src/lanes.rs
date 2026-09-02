//! The device arm of `holon_chem::lanes`: the same flat tables, uploaded once, and the kernel
//! `kernels/lanes_sigma.cu` walking them exactly as the host body does.
//!
//! This is THE device operator for every determinant space the engine solves — chemistry's two
//! spin lanes and a gauge theory's colour lanes alike — and, because the kernel is a gather with
//! no reduction and no library call, it is bit-identical to the host shards (measured in
//! `tests/lanes_sigma.rs` and `tests/fci_sigma.rs`).
//!
//! Refusals are loud (D4/D5): no VRAM is an error naming the bytes, never a host fallback.

use std::sync::Arc;

use cudarc::driver::{CudaContext, CudaFunction, CudaSlice, CudaStream, PushKernelArg};
use cudarc::nvrtc::Ptx;
use holon_chem::budget::DAVIDSON_SUBSPACE_MAX;
use holon_chem::lanes::{solve_lanes_in, LaneSolution, LaneTables};
use holon_chem::sigma_op::{DeviceClass, SigmaOp};

use crate::fci::{grid_for, FciGpuError};
use crate::vecspace::DeviceSpace;

/// PTX for `kernels/lanes_sigma.cu`, produced by `build.rs` with `-fmad=false`.
const PTX: &str = include_str!("../kernels/lanes_sigma.ptx");
/// The kernel's source, read for ONE number: its `HOLON_MAX_LANES` must equal the host's
/// `MAX_LANES`, and a mismatch is refused at operator build rather than found in a launch.
const CU: &str = include_str!("../kernels/lanes_sigma.cu");

fn kernel_max_lanes() -> usize {
    CU.lines()
        .find_map(|l| l.strip_prefix("#define HOLON_MAX_LANES "))
        .and_then(|v| v.trim().parse().ok())
        .expect("lanes_sigma.cu declares HOLON_MAX_LANES")
}

pub struct GpuLaneSigma {
    stream: Arc<CudaStream>,
    func: CudaFunction,
    n_det: usize,
    n_lanes: i32,
    /// The operator's own vectors for the host-driven path; `None` only while a launch borrows
    /// them. The device-resident path applies to external vectors and never touches these.
    d_c: Option<CudaSlice<f64>>,
    d_sigma: Option<CudaSlice<f64>>,
    lane_n: CudaSlice<i32>,
    lane_ns: CudaSlice<i32>,
    lane_stride: CudaSlice<i64>,
    lane_off_singles: CudaSlice<i64>,
    lane_off_at: CudaSlice<i64>,
    lane_off_h: CudaSlice<i32>,
    lane_pair_ptr: CudaSlice<i32>,
    singles_tp: CudaSlice<i32>,
    singles_sign: CudaSlice<f64>,
    singles_j: CudaSlice<i32>,
    at_sign: CudaSlice<f64>,
    at_j: CudaSlice<i32>,
    h: CudaSlice<f64>,
    pair_m: CudaSlice<i32>,
    pair_half: CudaSlice<f64>,
    pair_row_off: CudaSlice<i32>,
    pair_ent_off: CudaSlice<i32>,
    row_ptr: CudaSlice<i32>,
    ent_sr: CudaSlice<i32>,
    ent_coef: CudaSlice<f64>,
}

/// An empty table still needs a device pointer the kernel can hold; one element of padding
/// keeps every upload a real allocation.
fn upload<T: cudarc::driver::DeviceRepr + Copy + Default + Unpin>(
    stream: &Arc<CudaStream>,
    v: &[T],
) -> Result<CudaSlice<T>, FciGpuError> {
    if v.is_empty() {
        Ok(stream.clone_htod(&[T::default()])?)
    } else {
        Ok(stream.clone_htod(v)?)
    }
}

impl GpuLaneSigma {
    /// Device bytes this operator needs: the tables plus the two vectors.
    pub fn bytes_for(t: &LaneTables<f64>) -> u64 {
        t.bytes() + 2 * 8 * t.n_det as u64 + 4096
    }

    /// Build and upload, checking VRAM FIRST against `reserve_mib` held back for whatever else
    /// runs on the card.
    pub fn new(ctx: &Arc<CudaContext>, t: &LaneTables<f64>, reserve_mib: u64) -> Result<GpuLaneSigma, FciGpuError> {
        assert_eq!(
            kernel_max_lanes(),
            holon_chem::lanes::MAX_LANES,
            "lanes_sigma.cu's HOLON_MAX_LANES and holon_chem::lanes::MAX_LANES have drifted apart"
        );
        assert!(t.n_lanes <= holon_chem::lanes::MAX_LANES, "{} lanes exceed the kernel's register array", t.n_lanes);
        ctx.bind_to_thread()?;
        let (free, _total) = cudarc::driver::result::mem_get_info()?;
        let need = Self::bytes_for(t);
        if need.saturating_add(reserve_mib << 20) > free as u64 {
            return Err(FciGpuError::NotEnoughVram { need_bytes: need, free_bytes: free as u64 });
        }
        let stream = ctx.new_stream()?;
        let module = ctx.load_module(Ptx::from_src(PTX))?;
        let func = module.load_function("holon_lanes_sigma")?;
        let op = GpuLaneSigma {
            n_det: t.n_det,
            n_lanes: t.n_lanes as i32,
            d_c: Some(unsafe { stream.alloc::<f64>(t.n_det)? }),
            d_sigma: Some(unsafe { stream.alloc::<f64>(t.n_det)? }),
            lane_n: upload(&stream, &t.lane_n)?,
            lane_ns: upload(&stream, &t.lane_ns)?,
            lane_stride: upload(&stream, &t.lane_stride)?,
            lane_off_singles: upload(&stream, &t.lane_off_singles)?,
            lane_off_at: upload(&stream, &t.lane_off_at)?,
            lane_off_h: upload(&stream, &t.lane_off_h)?,
            lane_pair_ptr: upload(&stream, &t.lane_pair_ptr)?,
            singles_tp: upload(&stream, &t.singles_tp)?,
            singles_sign: upload(&stream, &t.singles_sign)?,
            singles_j: upload(&stream, &t.singles_j)?,
            at_sign: upload(&stream, &t.at_sign)?,
            at_j: upload(&stream, &t.at_j)?,
            h: upload(&stream, &t.h)?,
            pair_m: upload(&stream, &t.pair_m)?,
            pair_half: upload(&stream, &t.pair_half)?,
            pair_row_off: upload(&stream, &t.pair_row_off)?,
            pair_ent_off: upload(&stream, &t.pair_ent_off)?,
            row_ptr: upload(&stream, &t.row_ptr)?,
            ent_sr: upload(&stream, &t.ent_sr)?,
            ent_coef: upload(&stream, &t.ent_coef)?,
            stream,
            func,
        };
        op.stream.synchronize()?;
        Ok(op)
    }

    /// The kernel on the operator's own vectors.
    fn launch(&mut self, diag: i32) -> Result<(), FciGpuError> {
        let c = self.d_c.take().expect("operator vectors present");
        let mut s = self.d_sigma.take().expect("operator vectors present");
        let r = self.launch_on(diag, &c, &mut s);
        self.d_c = Some(c);
        self.d_sigma = Some(s);
        r
    }

    /// The kernel on ANY resident vectors: `sigma = H c` (`diag = 0`) or the diagonal into
    /// `sigma` (`diag = 1`, `c` unread). No copy, no synchronisation: stream order carries it.
    fn launch_on(&mut self, diag: i32, c: &CudaSlice<f64>, sigma: &mut CudaSlice<f64>) -> Result<(), FciGpuError> {
        assert_eq!(c.len(), self.n_det, "input vector is not this operator's dimension");
        assert_eq!(sigma.len(), self.n_det, "output vector is not this operator's dimension");
        let n_det = self.n_det as i64;
        let n_lanes = self.n_lanes;
        let cfg = grid_for(n_det);
        let mut b = self.stream.launch_builder(&self.func);
        b.arg(sigma)
            .arg(c)
            .arg(&n_det)
            .arg(&n_lanes)
            .arg(&diag)
            .arg(&self.lane_n)
            .arg(&self.lane_ns)
            .arg(&self.lane_stride)
            .arg(&self.lane_off_singles)
            .arg(&self.lane_off_at)
            .arg(&self.lane_off_h)
            .arg(&self.lane_pair_ptr)
            .arg(&self.singles_tp)
            .arg(&self.singles_sign)
            .arg(&self.singles_j)
            .arg(&self.at_sign)
            .arg(&self.at_j)
            .arg(&self.h)
            .arg(&self.pair_m)
            .arg(&self.pair_half)
            .arg(&self.pair_row_off)
            .arg(&self.pair_ent_off)
            .arg(&self.row_ptr)
            .arg(&self.ent_sr)
            .arg(&self.ent_coef);
        unsafe { b.launch(cfg)? };
        Ok(())
    }

    /// `sigma = H c` between resident vectors — the device-resident solve's operator. The
    /// vectors stay on the card; the stream orders the launch after whatever produced `c`.
    pub fn apply_on(&mut self, c: &CudaSlice<f64>, sigma: &mut CudaSlice<f64>) -> Result<(), FciGpuError> {
        self.launch_on(0, c, sigma)
    }

    /// `<k|H|k>` into a resident vector.
    pub fn diagonal_on(&mut self, out: &mut CudaSlice<f64>) -> Result<(), FciGpuError> {
        let c = self.d_c.take().expect("operator vectors present");
        let r = self.launch_on(1, &c, out);
        self.d_c = Some(c);
        r
    }

    /// The stream the operator launches on; a resident solve builds its space on the same one.
    pub fn stream(&self) -> &Arc<CudaStream> {
        &self.stream
    }

    /// `sigma = H c`, reporting the driver's errors rather than panicking through them.
    pub fn try_apply(&mut self, c: &[f64], sigma: &mut [f64]) -> Result<(), FciGpuError> {
        assert_eq!(c.len(), self.n_det, "input vector is not this operator's dimension");
        assert_eq!(sigma.len(), self.n_det, "output vector is not this operator's dimension");
        let mut dc = self.d_c.take().expect("operator vectors present");
        self.stream.memcpy_htod(c, &mut dc)?;
        self.d_c = Some(dc);
        self.launch(0)?;
        self.stream.memcpy_dtoh(self.d_sigma.as_ref().expect("operator vectors present"), sigma)?;
        self.stream.synchronize()?;
        Ok(())
    }

    /// `<k|H|k>` for every `k`, from the same walk (the kernel's `diag` switch).
    pub fn diagonal(&mut self) -> Result<Vec<f64>, FciGpuError> {
        self.launch(1)?;
        let mut d = vec![0.0f64; self.n_det];
        self.stream.memcpy_dtoh(self.d_sigma.as_ref().expect("operator vectors present"), &mut d)?;
        self.stream.synchronize()?;
        Ok(d)
    }

    /// Device-resident timing: `reps` applications with no host round trip, returning seconds
    /// per application. What the device does, as distinct from what the PCIe bus costs.
    pub fn seconds_per_sigma_resident(&mut self, reps: usize) -> Result<f64, FciGpuError> {
        self.launch(0)?;
        self.stream.synchronize()?;
        let t0 = std::time::Instant::now();
        for _ in 0..reps {
            self.launch(0)?;
        }
        self.stream.synchronize()?;
        Ok(t0.elapsed().as_secs_f64() / reps.max(1) as f64)
    }
}

impl SigmaOp<f64> for GpuLaneSigma {
    fn n_det(&self) -> usize {
        self.n_det
    }
    fn device(&self) -> DeviceClass {
        DeviceClass::Gpu
    }
    fn apply(&mut self, c: &[f64], sigma: &mut [f64]) {
        // A device that vanished under a live solve is a conviction, not a number to carry on with.
        self.try_apply(c, sigma).expect("device sigma failed mid-solve");
    }
}

/// Bytes a device-resident Davidson holds beyond the operator: `2·max_sub + 8` vectors, the
/// same count the host door prices, plus the partials scratch.
pub fn resident_solve_bytes(n_det: usize, max_sub: usize) -> u64 {
    let vectors = 2 * max_sub.max(2) + 8;
    (n_det as u64) * (vectors as u64) * 8 + (n_det.div_ceil(holon_chem::vecspace::DOT_BLOCK) as u64) * ((DAVIDSON_SUBSPACE_MAX + 1) as u64) * 8
}

/// THE DEVICE-RESIDENT SOLVE: tables uploaded once, the Davidson's vectors on the card, the
/// physics kernel applied between them, the host holding only the `m×m` eigenproblem. The
/// same body as the host solve (`holon_chem::tier::davidson_in`) on `DeviceSpace`, so the
/// answer is the host's answer to the bit (`tests/vecspace.rs`).
///
/// The admission is VRAM's: the operator's footprint plus the solve's vectors against the
/// card's free memory less `reserve_mib`, refused loudly (D4) rather than attempted.
pub fn solve_lanes_on_device(
    ctx: &Arc<CudaContext>,
    tables: &LaneTables<f64>,
    reserve_mib: u64,
    start: Option<&[f64]>,
    budget: usize,
    max_sub: usize,
) -> Result<LaneSolution, FciGpuError> {
    ctx.bind_to_thread()?;
    let (free, _total) = cudarc::driver::result::mem_get_info()?;
    let need = GpuLaneSigma::bytes_for(tables) + resident_solve_bytes(tables.n_det, max_sub);
    if need.saturating_add(reserve_mib << 20) > free as u64 {
        return Err(FciGpuError::NotEnoughVram { need_bytes: need, free_bytes: free as u64 });
    }
    let mut op = GpuLaneSigma::new(ctx, tables, 0)?;
    let sp = DeviceSpace::new(ctx)?;
    let diag_host = op.diagonal()?;
    let mut apply = |c: &CudaSlice<f64>, s: &mut CudaSlice<f64>| {
        // A device that vanished under a live solve is a conviction, not a number to carry on with.
        op.apply_on(c, s).expect("device sigma failed mid-solve");
    };
    Ok(solve_lanes_in(&sp, &mut apply, &diag_host, start, budget, max_sub, DeviceClass::Gpu))
}
