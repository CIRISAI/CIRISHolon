//! **The GPU device-class arm of the determinant solve** — `sigma = H c` on CUDA, driving
//! `holon-chem`'s own Davidson.
//!
//! RESOURCE_DESIGN **D0**: the device class belongs to the ARTIFACT, not to the schedule. This
//! module is the device side of that rule. It implements `holon_chem::sigma_op::SigmaOp` and
//! `SigmaProvider` from OUTSIDE `holon-chem`, which is the whole architecture: the chemistry
//! crate names the contract and stays portable enough to ship into a browser, and the CUDA
//! dependency lives here, out of the workspace, behind a build script that shells out to nvcc.
//!
//! # What is on the device and what is not
//!
//! The sigma is. The Davidson driver is not — the subspace, the Rayleigh–Ritz, the restart and
//! the preconditioner all stay host-side in `holon_chem::tier::davidson_eigh_from_op`, exactly
//! as they were. That is the split the measurement supports: at the `(O,O,O)` scale one sigma
//! is ~15 ms of device compute against ~0.5 ms of PCIe for the round trip, so moving the driver
//! would buy 3% and cost the ability to run one engine under two devices. Moving the whole loop
//! device-side is a real question for spaces where the vector is large and the iteration count
//! high; it is NOT answered here and nothing in this file pretends it is.
//!
//! # The c-independent tables are built once
//!
//! `F_a`, `F_b` and `A` depend on the Hamiltonian and not on the vector, so they are built and
//! uploaded when the operator is constructed and re-used for every application. That is the
//! reason this is an OBJECT rather than a function: a per-call kernel would rebuild 42 MB of
//! tables on every Davidson iteration.
//!
//! # Determinism is pinned, then MEASURED
//!
//! There is not one atomic in `kernels/fci_sigma.cu` (see its header for why the scatters
//! invert into gathers). cuBLAS is pinned to `CUBLAS_PEDANTIC_MATH` with a fixed workspace, so
//! its kernel selection cannot vary with whatever happens to be free at the time. Neither of
//! those is evidence. The evidence is `holon_chem::sigma_op::bit_identity_over_runs` run on the
//! operator that will actually be used, and `tests/fci_sigma.rs` runs it.

use std::sync::Arc;

use cudarc::cublas::{CudaBlas, Gemm, GemmConfig, StridedBatchedConfig};
use cudarc::cublas::sys as cublas_sys;
use cudarc::driver::{
    CudaContext, CudaFunction, CudaSlice, CudaStream, LaunchConfig, PushKernelArg,
};
use cudarc::nvrtc::Ptx;

use holon_chem::fci::{CiInts, FciSpace};
use holon_chem::sigma_op::{DeviceClass, SigmaOp, SigmaProvider};

/// PTX for `kernels/fci_sigma.cu`, produced by `build.rs`.
const PTX: &str = include_str!("../kernels/fci_sigma.ptx");

/// cuBLAS workspace, fixed so kernel selection cannot vary with what is free.
///
/// 4 MiB is cuBLAS's own default for this device class. It is pinned rather than left to the
/// library because the heuristic that picks a GEMM kernel reads the available workspace, and a
/// kernel chosen differently is a different reduction order — which is a different artifact, not
/// a different speed.
const CUBLAS_WORKSPACE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug)]
pub enum FciGpuError {
    Driver(cudarc::driver::DriverError),
    Cublas(cudarc::cublas::result::CublasError),
    /// The excitation lists are not uniform in length. The flat device layout depends on it, and
    /// a ragged list would silently corrupt every stride rather than fail.
    RaggedSingles { spin: &'static str, at: usize, expected: usize, got: usize },
    /// Two source strings claim one `(kl, ib)` slot. The gather formulation rests on the
    /// excitation map being invertible; if it is not, a summation has been silently dropped and
    /// the answer would be wrong in a way a tolerance check might not localise.
    GatherNotInjective { kl: usize, ib: usize },
    /// The transposed alpha list is ragged, which the same uniformity argument forbids.
    TransposeRagged { ia: usize, got: usize, expected: usize },
    /// The device does not have room. REFUSED with the arithmetic, never attempted and OOMed —
    /// a sibling process on this card should not die because this one guessed.
    NotEnoughVram { need_bytes: u64, free_bytes: u64 },
    /// The space is too large for the flat 32-bit index the kernels use.
    TooLarge { what: &'static str, value: u64 },
}

impl From<cudarc::driver::DriverError> for FciGpuError {
    fn from(e: cudarc::driver::DriverError) -> Self {
        FciGpuError::Driver(e)
    }
}
impl From<cudarc::cublas::result::CublasError> for FciGpuError {
    fn from(e: cudarc::cublas::result::CublasError) -> Self {
        FciGpuError::Cublas(e)
    }
}

impl std::fmt::Display for FciGpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FciGpuError::Driver(e) => write!(f, "cuda driver: {e}"),
            FciGpuError::Cublas(e) => write!(f, "cublas: {e:?}"),
            FciGpuError::RaggedSingles { spin, at, expected, got } => write!(
                f,
                "the {spin} excitation lists are ragged: string {at} has {got} singles against \
                 {expected}. The flat device layout is strided on that count, so this would \
                 corrupt every row rather than fail."
            ),
            FciGpuError::GatherNotInjective { kl, ib } => write!(
                f,
                "two source strings map to (kl={kl}, ib={ib}); the beta gather is not \
                 injective, so inverting the scatter has dropped a summation"
            ),
            FciGpuError::TransposeRagged { ia, got, expected } => write!(
                f,
                "the transposed alpha singles are ragged at ia={ia}: {got} against {expected}"
            ),
            FciGpuError::NotEnoughVram { need_bytes, free_bytes } => write!(
                f,
                "this operator needs {:.1} MB of VRAM and the device reports {:.1} MB free. \
                 REFUSED rather than attempted: an allocation that fails partway leaves a \
                 sibling process on this card holding the consequence.",
                *need_bytes as f64 / 1e6,
                *free_bytes as f64 / 1e6
            ),
            FciGpuError::TooLarge { what, value } => write!(
                f,
                "{what} = {value} does not fit the 32-bit index the kernels use"
            ),
        }
    }
}

impl std::error::Error for FciGpuError {}

/// The device operator for ONE Hamiltonian.
///
/// Holds the c-independent tables on the device and the scratch that one application writes
/// through. Built by [`GpuSigmaProvider::op_for`]; there is no other constructor that skips the
/// VRAM check.
pub struct GpuFciSigma {
    stream: Arc<CudaStream>,
    blas: CudaBlas,
    build_t: CudaFunction,
    gather_mixed: CudaFunction,

    na: i32,
    nb: i32,
    n2: i32,
    ns_a: i32,
    n_det: usize,

    // c-independent, uploaded once
    d_fa: CudaSlice<f64>,
    d_fb: CudaSlice<f64>,
    d_a: CudaSlice<f64>,
    d_src_jb: CudaSlice<i32>,
    d_src_sign: CudaSlice<f64>,
    d_at_ja: CudaSlice<i32>,
    d_at_m: CudaSlice<i32>,
    d_at_sign: CudaSlice<f64>,

    // per-application scratch
    d_t: CudaSlice<f64>,
    d_d: CudaSlice<f64>,
    d_c: CudaSlice<f64>,
    d_sigma: CudaSlice<f64>,

    // held so cuBLAS's pinned workspace outlives the handle
    _workspace: CudaSlice<u8>,
}

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

    /// **How many GPU-class workers this card can actually hold for a given space.**
    ///
    /// F.2 needs this and it is a REFUSAL, not a tuning knob. A GPU-class table generator
    /// cannot be handed a worker count the way the CPU one can: each worker needs its OWN
    /// device operator, holding its own copy of the c-independent tables, because
    /// `GpuFciSigma` owns device buffers a second thread cannot share. So the bound is VRAM
    /// per operator, not cores.
    ///
    /// **MEASURED, correcting a claim this comment first carried.** At the `(O,O,O)` scale the
    /// footprint is **480.5 MiB per worker** against **15,683 MiB free**, so with a 1 GiB
    /// reserve **30 workers fit** — not the 2–3 first written here. That figure was arithmetic
    /// run backwards (16 GB over 0.5 GB is 32, not 2), and it was repeated into two lanes'
    /// inboxes and a results document before this function measured it.
    ///
    /// **VRAM is therefore NOT the reason GPU-class generation does not help a table.** The
    /// reason is the one that was measured: the sigma is 4% of a Davidson iteration, so
    /// Amdahl caps a whole-table speedup near 3% even with the device free — and 30 workers
    /// sharing one device serialise that 4% among themselves, which can make it negative. The
    /// conclusion survives on one leg instead of two, and the leg that failed is the one that
    /// had been stated most confidently.
    ///
    /// # Why this is derived and not declared
    ///
    /// A constant would be wrong on the next card and wrong again on the next space. The
    /// footprint is already computed exactly by [`vram_bytes_for`] — the same arithmetic the
    /// operator's own pre-allocation check uses — and free VRAM is a live reading. Deriving
    /// the answer from both is the only version that stays true when either moves.
    ///
    /// # The reserve, and why it is not zero
    ///
    /// `reserve_mib` is held back rather than filled. This card runs the browser's GPU
    /// process too, and a generator that consumed every free byte would be correct about its
    /// own arithmetic and hostile to everything else on the machine. The caller states it;
    /// there is no default, because the right reserve depends on what else is running and a
    /// default would be a guess wearing a policy's clothes.
    ///
    /// Returns 0 when not even one worker fits — which a caller must treat as a REFUSAL of
    /// GPU-class generation for that space, not as a reason to fall back to the host (D4: no
    /// silent fallback across classes).
    pub fn max_workers_for(
        &self,
        space: &FciSpace,
        reserve_mib: u64,
    ) -> Result<usize, FciGpuError> {
        let per_worker = vram_bytes_for(space)?;
        if per_worker == 0 {
            return Ok(0);
        }
        let (free, _total) = self.mem_info()?;
        let usable = (free as u64).saturating_sub(reserve_mib.saturating_mul(1 << 20));
        Ok((usable / per_worker) as usize)
    }

    /// Build the device operator for one integral set, checking VRAM FIRST.
    pub fn build(
        &self,
        space: &FciSpace,
        ci: &CiInts,
    ) -> Result<GpuFciSigma, FciGpuError> {
        GpuFciSigma::new(&self.ctx, space, ci)
    }
}

impl SigmaProvider for GpuSigmaProvider {
    fn device(&self) -> DeviceClass {
        DeviceClass::Gpu
    }

    fn op_for<'a>(
        &self,
        space: &'a FciSpace,
        ci: &'a CiInts,
    ) -> Result<Box<dyn SigmaOp<f64> + 'a>, String> {
        // The operator owns its device memory, so it does not borrow from `space`/`ci` — but
        // the trait's lifetime allows a host operator to borrow, and this one simply does not
        // need to.
        self.build(space, ci)
            .map(|op| Box::new(op) as Box<dyn SigmaOp<f64> + 'a>)
            .map_err(|e| e.to_string())
    }
}

/// Bytes this operator will hold on the device, computed BEFORE anything is allocated.
///
/// Exposed because the resource layer leases against it: a lease states a quantitative boundary
/// (D3b), and "however much it turns out to need" is not one.
pub fn vram_bytes_for(space: &FciSpace) -> Result<u64, FciGpuError> {
    let n2 = (space.n_orb * space.n_orb) as u64;
    let na = space.alpha.len() as u64;
    let nb = space.beta.len() as u64;
    let ns_a = space.alpha.singles.first().map(|s| s.len()).unwrap_or(0) as u64;
    let n_det = space.n_det as u64;
    let f64s = na * n2 * nb            // T
        + na * ns_a * n2               // A
        + na * ns_a * nb               // D
        + na * na + nb * nb            // F_a, F_b
        + n2 * nb                      // src_sign
        + na * ns_a                    // at_sign
        + 2 * n_det;                   // c, sigma
    let i32s = n2 * nb + 2 * na * ns_a; // src_jb, at_ja, at_m
    Ok(f64s * 8 + i32s * 4 + CUBLAS_WORKSPACE_BYTES as u64)
}

impl GpuFciSigma {
    /// Build the operator directly, without a lease.
    ///
    /// `pub(crate)` and not `pub`: it checks its own footprint against the card and refuses, but
    /// a check is not a lease — it records nothing and two operators built this way in one
    /// process would each pass and then contend. The public routes are
    /// [`GpuSigmaProvider::build`] (unleased, for benchmarks and gates) and
    /// [`crate::lease::LeasedGpuProvider`] (leased, for production).
    pub(crate) fn new(
        ctx: &Arc<CudaContext>,
        space: &FciSpace,
        ci: &CiInts,
    ) -> Result<GpuFciSigma, FciGpuError> {
        let n = space.n_orb;
        let n2 = n * n;
        let na = space.alpha.len();
        let nb = space.beta.len();
        let n_det = space.n_det;

        // The flat device layout is strided on a UNIFORM singles count. A string with `e` of
        // `n` orbitals occupied has exactly `e*(n-e)+e` single excitations, every string alike
        // — so this holds, and it is asserted rather than assumed because a ragged list would
        // corrupt every stride silently instead of failing.
        let ns_a = space.alpha.singles.first().map(|s| s.len()).unwrap_or(0);
        let ns_b = space.beta.singles.first().map(|s| s.len()).unwrap_or(0);
        for (spin, strings, expected) in [
            ("alpha", &space.alpha, ns_a),
            ("beta", &space.beta, ns_b),
        ] {
            if let Some((at, row)) = strings
                .singles
                .iter()
                .enumerate()
                .find(|(_, r)| r.len() != expected)
            {
                return Err(FciGpuError::RaggedSingles {
                    spin,
                    at,
                    expected,
                    got: row.len(),
                });
            }
        }

        // The GEMM dimensions cross the cuBLAS ABI as `int`, so a space past 2^31 in any of
        // them would be truncated rather than refused. The flat `T` index is `long` inside the
        // kernel and is not the constraint; these are.
        for (what, v) in [
            ("n_det", n_det as u64),
            ("n_beta_strings", nb as u64),
            ("n_alpha_strings", na as u64),
        ] {
            if v > i32::MAX as u64 {
                return Err(FciGpuError::TooLarge { what, value: v });
            }
        }

        // ---- the VRAM check, BEFORE any allocation (D2: measure the headroom for the thing).
        let need = vram_bytes_for(space)?;
        ctx.bind_to_thread()?;
        let (free, _total) = cudarc::driver::result::mem_get_info()?;
        if need > free as u64 {
            return Err(FciGpuError::NotEnoughVram {
                need_bytes: need,
                free_bytes: free as u64,
            });
        }

        // ---- host-side precomputation, all c-independent.

        // The beta gather map: for a fixed (kl, ib) the source string jb is unique, because
        // a+_p a_q |jb> = s|ib> inverts. Injectivity is CHECKED — the whole atomics-free
        // construction rests on it.
        let mut src_jb = vec![-1i32; n2 * nb];
        let mut src_sign = vec![0.0f64; n2 * nb];
        for (jb, row) in space.beta.singles.iter().enumerate() {
            for &(pq, sign, dst) in row.iter() {
                let e = pq as usize * nb + dst as usize;
                if src_jb[e] != -1 {
                    return Err(FciGpuError::GatherNotInjective {
                        kl: pq as usize,
                        ib: dst as usize,
                    });
                }
                src_jb[e] = jb as i32;
                src_sign[e] = sign;
            }
        }

        // A[ja][m][kl] = g[pq(ja,m)][kl] — the rows of g this alpha string's excitations select.
        let mut a_tab = vec![0.0f64; na * ns_a * n2];
        for (ja, row) in space.alpha.singles.iter().enumerate() {
            for (m, &(pq, _sign, _dst)) in row.iter().enumerate() {
                let dst = (ja * ns_a + m) * n2;
                let src = pq as usize * n2;
                a_tab[dst..dst + n2].copy_from_slice(&ci.g[src..src + n2]);
            }
        }

        // The transposed alpha singles, so the mixed block GATHERS instead of scattering.
        let mut at_ja = vec![0i32; na * ns_a];
        let mut at_m = vec![0i32; na * ns_a];
        let mut at_sign = vec![0.0f64; na * ns_a];
        let mut fill = vec![0usize; na];
        for (ja, row) in space.alpha.singles.iter().enumerate() {
            for (m, &(_pq, sign, dst)) in row.iter().enumerate() {
                let ia = dst as usize;
                if fill[ia] >= ns_a {
                    return Err(FciGpuError::TransposeRagged {
                        ia,
                        got: fill[ia] + 1,
                        expected: ns_a,
                    });
                }
                let e = ia * ns_a + fill[ia];
                fill[ia] += 1;
                at_ja[e] = ja as i32;
                at_m[e] = m as i32;
                at_sign[e] = sign;
            }
        }
        if let Some((ia, got)) = fill.iter().enumerate().find(|(_, c)| **c != ns_a) {
            return Err(FciGpuError::TransposeRagged {
                ia,
                got: *got,
                expected: ns_a,
            });
        }

        // The two same-spin matrices, built exactly as `sigma_direct_t` builds its inner `f`.
        let mut fb = vec![0.0f64; nb * nb];
        build_same_spin(&mut fb, &space.beta.singles, &ci.k, &ci.g, n2, nb);
        let mut fa = vec![0.0f64; na * na];
        build_same_spin(&mut fa, &space.alpha.singles, &ci.k, &ci.g, n2, na);

        // ---- the device.
        let stream = ctx.new_stream()?;
        let module = ctx.load_module(Ptx::from_src(PTX))?;
        let build_t = module.load_function("holon_fci_build_T")?;
        let gather_mixed = module.load_function("holon_fci_gather_mixed")?;

        let blas = CudaBlas::new(stream.clone())?;
        // PIN the library. Pedantic math forbids any faster-but-different path, and a fixed
        // workspace stops the kernel-selection heuristic from reading whatever happens to be
        // free — both are inputs to WHICH kernel runs, and a different kernel is a different
        // reduction order.
        let workspace: CudaSlice<u8> = unsafe { stream.alloc::<u8>(CUBLAS_WORKSPACE_BYTES)? };
        unsafe {
            cublas_sys::cublasSetMathMode(*blas.handle(), cublas_sys::cublasMath_t::CUBLAS_PEDANTIC_MATH)
                .result()
                .map_err(FciGpuError::Cublas)?;
            let (ptr, _rec) = {
                use cudarc::driver::DevicePtr;
                workspace.device_ptr(&stream)
            };
            cublas_sys::cublasSetWorkspace_v2(
                *blas.handle(),
                ptr as *mut std::ffi::c_void,
                CUBLAS_WORKSPACE_BYTES,
            )
            .result()
            .map_err(FciGpuError::Cublas)?;
        }

        let d_fa = stream.clone_htod(&fa)?;
        let d_fb = stream.clone_htod(&fb)?;
        let d_a = stream.clone_htod(&a_tab)?;
        let d_src_jb = stream.clone_htod(&src_jb)?;
        let d_src_sign = stream.clone_htod(&src_sign)?;
        let d_at_ja = stream.clone_htod(&at_ja)?;
        let d_at_m = stream.clone_htod(&at_m)?;
        let d_at_sign = stream.clone_htod(&at_sign)?;

        let d_t: CudaSlice<f64> = unsafe { stream.alloc::<f64>(na * n2 * nb)? };
        let d_d: CudaSlice<f64> = unsafe { stream.alloc::<f64>(na * ns_a * nb)? };
        let d_c: CudaSlice<f64> = unsafe { stream.alloc::<f64>(n_det)? };
        let d_sigma: CudaSlice<f64> = unsafe { stream.alloc::<f64>(n_det)? };
        stream.synchronize()?;

        Ok(GpuFciSigma {
            stream,
            blas,
            build_t,
            gather_mixed,
            na: na as i32,
            nb: nb as i32,
            n2: n2 as i32,
            ns_a: ns_a as i32,
            n_det,
            d_fa,
            d_fb,
            d_a,
            d_src_jb,
            d_src_sign,
            d_at_ja,
            d_at_m,
            d_at_sign,
            d_t,
            d_d,
            d_c,
            d_sigma,
            _workspace: workspace,
        })
    }

    /// One sigma, entirely on the device, in a FIXED order: beta same-spin, alpha same-spin,
    /// then mixed. The order is fixed because it is part of the answer's last bits.
    fn sigma_on_device(&mut self) -> Result<(), FciGpuError> {
        let (na, nb, n2, ns_a) = (self.na, self.nb, self.n2, self.ns_a);

        // Sigma = C * F_b   (row-major m=na, k=nb, n=nb, expressed to a column-major library
        // by swapping the operands — the standard identity, not a transposition of the data)
        unsafe {
            self.blas.gemm(
                GemmConfig {
                    transa: cublas_sys::cublasOperation_t::CUBLAS_OP_N,
                    transb: cublas_sys::cublasOperation_t::CUBLAS_OP_N,
                    m: nb,
                    n: na,
                    k: nb,
                    alpha: 1.0f64,
                    lda: nb,
                    ldb: nb,
                    beta: 0.0f64,
                    ldc: nb,
                },
                &self.d_fb,
                &self.d_c,
                &mut self.d_sigma,
            )?;
        }

        // Sigma += F_a^T * C
        unsafe {
            self.blas.gemm(
                GemmConfig {
                    transa: cublas_sys::cublasOperation_t::CUBLAS_OP_N,
                    transb: cublas_sys::cublasOperation_t::CUBLAS_OP_T,
                    m: nb,
                    n: na,
                    k: na,
                    alpha: 1.0f64,
                    lda: nb,
                    ldb: na,
                    beta: 1.0f64,
                    ldc: nb,
                },
                &self.d_c,
                &self.d_fa,
                &mut self.d_sigma,
            )?;
        }

        // T <- gather(c)
        let total = (na as i64) * (n2 as i64) * (nb as i64);
        let cfg = grid_for(total);
        let mut b = self.stream.launch_builder(&self.build_t);
        b.arg(&mut self.d_t)
            .arg(&self.d_c)
            .arg(&self.d_src_jb)
            .arg(&self.d_src_sign)
            .arg(&na)
            .arg(&nb)
            .arg(&n2);
        unsafe { b.launch(cfg)? };

        // D[ja] = A[ja] * T[ja], batched over the alpha strings
        unsafe {
            self.blas.gemm_strided_batched(
                StridedBatchedConfig {
                    gemm: GemmConfig {
                        transa: cublas_sys::cublasOperation_t::CUBLAS_OP_N,
                        transb: cublas_sys::cublasOperation_t::CUBLAS_OP_N,
                        m: nb,
                        n: ns_a,
                        k: n2,
                        alpha: 1.0f64,
                        lda: nb,
                        ldb: n2,
                        beta: 0.0f64,
                        ldc: nb,
                    },
                    batch_size: na,
                    stride_a: (n2 as i64) * (nb as i64),
                    stride_b: (ns_a as i64) * (n2 as i64),
                    stride_c: (ns_a as i64) * (nb as i64),
                },
                &self.d_t,
                &self.d_a,
                &mut self.d_d,
            )?;
        }

        // Sigma += gather(D)
        let cfg = grid_for((na as i64) * (nb as i64));
        let mut b = self.stream.launch_builder(&self.gather_mixed);
        b.arg(&mut self.d_sigma)
            .arg(&self.d_d)
            .arg(&self.d_at_ja)
            .arg(&self.d_at_m)
            .arg(&self.d_at_sign)
            .arg(&na)
            .arg(&nb)
            .arg(&ns_a);
        unsafe { b.launch(cfg)? };

        Ok(())
    }

    /// `sigma = H c`, reporting the driver's own errors rather than panicking through them.
    ///
    /// [`SigmaOp::apply`] cannot return an error — the Davidson driver has no channel for one —
    /// so this is the entry point for a caller that wants to handle a mid-solve device failure.
    /// `apply` calls this and panics on `Err`, which is the honest translation: a device that
    /// vanished under a live solve is a CONVICTION, not a number to carry on with.
    pub fn try_apply(&mut self, c: &[f64], sigma: &mut [f64]) -> Result<(), FciGpuError> {
        assert_eq!(c.len(), self.n_det, "input vector is not this operator's dimension");
        assert_eq!(sigma.len(), self.n_det, "output vector is not this operator's dimension");
        self.stream.memcpy_htod(c, &mut self.d_c)?;
        self.sigma_on_device()?;
        self.stream.memcpy_dtoh(&self.d_sigma, sigma)?;
        self.stream.synchronize()?;
        Ok(())
    }

    /// The device-resident timing loop: `reps` applications with no host round trip.
    ///
    /// Separate from [`Self::try_apply`] because the two measure different things and quoting
    /// one as the other is how a PCIe cost disappears from a benchmark. G2 measured both and
    /// found the round trip to be 0.5 ms against 15 ms of compute; that is a fact about this
    /// size and is re-measured rather than carried forward.
    pub fn time_kernel_only(&mut self, reps: usize) -> Result<f64, FciGpuError> {
        self.stream.synchronize()?;
        let t0 = std::time::Instant::now();
        for _ in 0..reps {
            self.sigma_on_device()?;
        }
        self.stream.synchronize()?;
        Ok(t0.elapsed().as_secs_f64() / reps as f64)
    }

    /// Upload `c` once, so a timing loop measures the kernel rather than the transfer.
    pub fn preload(&mut self, c: &[f64]) -> Result<(), FciGpuError> {
        assert_eq!(c.len(), self.n_det);
        self.stream.memcpy_htod(c, &mut self.d_c)?;
        self.stream.synchronize()?;
        Ok(())
    }
}

impl SigmaOp<f64> for GpuFciSigma {
    fn n_det(&self) -> usize {
        self.n_det
    }
    fn device(&self) -> DeviceClass {
        DeviceClass::Gpu
    }
    fn apply(&mut self, c: &[f64], sigma: &mut [f64]) {
        // A device failure mid-solve is not a number to continue from. The lease layer's word
        // for it is CONVICTED — we held a valid lease and the resource went away underneath us
        // — and the honest translation into a driver with no error channel is to stop.
        self.try_apply(c, sigma)
            .unwrap_or_else(|e| panic!("the GPU sigma failed mid-solve: {e}"));
    }
}

/// The same-spin matrix `F`, built exactly as `sigma_direct_t`'s inner `f` loop builds it.
///
/// Written once and used for both spins: the two blocks differ only in which excitation lists
/// and which dimension they run over, and writing the loop twice is how the two come to
/// disagree after an edit to one of them.
fn build_same_spin(
    f: &mut [f64],
    singles: &[Vec<(u16, f64, u32)>],
    k: &[f64],
    g: &[f64],
    n2: usize,
    dim: usize,
) {
    for (j, row) in singles.iter().enumerate() {
        let out = &mut f[j * dim..(j + 1) * dim];
        for &(kl, s1, kdst) in row.iter() {
            out[kdst as usize] += s1 * k[kl as usize];
            for &(ij, s2, idst) in singles[kdst as usize].iter() {
                out[idst as usize] += 0.5 * s1 * s2 * g[ij as usize * n2 + kl as usize];
            }
        }
    }
}

/// A grid that covers `total` elements with a grid-stride loop.
///
/// The kernels are grid-stride, so the launch SHAPE cannot reach the answer — every thread sums
/// over a fixed index range in a fixed order whatever the grid is. That is what makes the block
/// and grid counts a scheduling choice rather than part of the artifact, and it is the same
/// argument `holon-gpu`'s fold makes for its reduction.
fn grid_for(total: i64) -> LaunchConfig {
    const BLOCK: u32 = 256;
    let want = ((total as u64).div_ceil(BLOCK as u64)) as u32;
    LaunchConfig {
        grid_dim: (want.clamp(1, 4096), 1, 1),
        block_dim: (BLOCK, 1, 1),
        shared_mem_bytes: 0,
    }
}
