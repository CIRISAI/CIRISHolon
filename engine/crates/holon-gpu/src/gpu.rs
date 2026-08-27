//! The CUDA executor: upload a batch of branch descriptors once, fold it for
//! any `y`, exactly.
//!
//! # The determinism argument
//!
//! The mesh's warrant (`holon::mesh`) is that exact `Z[omega]` addition is
//! associative and commutative, so how the fold is cut does not change its
//! VALUE. On the GPU the schedule is not merely uncoordinated, it is
//! unobservable — but the mesh's own header records that VALUE-equality is the
//! weaker claim, and that bit-identity of the REPRESENTATION is what a struct
//! comparison actually tests. This module gets the stronger one, and it gets it
//! by removing the ring from the reduction entirely:
//!
//! 1. Every branch's contribution is `BASE_b` rotated by `omega^{r_b}`
//!    (`desc::AffineDesc`), so no ring multiplication happens per `y`.
//! 2. The host aligns every `BASE_b` to ONE common exponent `M = max_b m_b`
//!    before upload, using the ledger's own alignment (`ring::align_to`). After
//!    that the fold is four independent sums of `i128` integers.
//! 3. Two's-complement integer addition is associative and commutative
//!    unconditionally — on overflow as much as off it. So the four sums are
//!    invariant under the warp-shuffle tree's shape, the block size, the grid
//!    size, and which branch landed on which thread. There is no atomic in the
//!    accumulation and no completion-order reduction; there does not need to be.
//! 4. `ring::from_lanes` puts the result back in the ledger's normal form.
//!
//! Determinism therefore does not depend on the batch being free of overflow.
//! CORRECTNESS does, so [`GpuBatch::upload`] refuses a batch whose worst case
//! could exceed `i128` rather than wrapping quietly.
//!
//! # Where this can differ from the CPU fold, and it is not a rounding
//!
//! `Cyc::normalize` divides out only EVEN powers of two, so a value has two
//! normalized faces — one per parity of `m` — and `1 = ([1,0,0,0], 0)` and
//! `1 = ([0,1,0,-1], 1)` are both fixed points. The sequential CPU fold's final
//! `m` carries the parity its own path arrived at; this module's carries the
//! parity of `M`.
//!
//! The sufficient condition is PARITY-uniformity, not exponent-uniformity, and
//! stating the stronger one was this module's own first error — corrected here
//! because a real `run_pruned` batch (n = 24, T = 18) came back with mixed
//! exponents AND bit-identical results, which the stronger claim could not
//! account for. [`GpuBatch::parity_uniform`] is the flag that matters;
//! [`GpuBatch::exponent_uniform`] is kept because it is cheap and informative,
//! not because anything rests on it.
//!
//! When every `m_b` shares one parity, the two folds coincide as STRUCTS:
//! `Cyc::add` takes `m = max(m_acc, m_b)`, `normalize` only ever steps `m` down
//! by two, so the CPU accumulator's parity is pinned at the common parity from
//! its first nonzero term onward — which is the parity of `M`. When parities
//! differ, the two folds can return equal NUMBERS in unequal STRUCTS, exactly as
//! `holon::mesh`'s header describes for shard-cancelling partial sums. That is
//! not a hypothetical here: `tests/determinism.rs`'s mixed-parity test MEASURES
//! it, and the measurement is that struct equality holds on 7 of 18 probes while
//! value equality and canonical-face equality hold on all 18. The remedy exists
//! and is not this module's to invent: `holon::mesh::canonicalize`. Nothing in
//! this crate applies it silently — the test suite compares structs directly and
//! reports the parity flag alongside, rather than canonicalizing the question
//! away.

use std::cell::RefCell;
use std::sync::Arc;

use cudarc::driver::{CudaContext, CudaFunction, CudaModule, CudaSlice, CudaStream, LaunchConfig, PushKernelArg};
use cudarc::nvrtc::Ptx;
use holon::ledger::Cyc;

use crate::desc::{AffineDesc, K_CAP, N_CAP, R_RANK_BAD};
use crate::ring;

/// PTX for `kernels/fold.cu`, produced by `build.rs`.
const PTX: &str = include_str!("../kernels/fold.ptx");

#[derive(Debug)]
pub enum GpuError {
    Driver(cudarc::driver::DriverError),
    Empty,
    Ragged { expected: usize, got: usize },
    TooManyQubits(usize),
    TooManyColumns(usize),
    /// The worst-case sum would not fit in `i128`. Reported, never wrapped.
    WouldOverflow { branches: usize, max_coeff: u128 },
    /// A block size the shuffle reduction cannot honour.
    BadBlock(u32),
    /// The device found a branch whose `R` has dependent columns — the `Affine`
    /// invariant the CPU asserts.
    RankDeficient,
}

impl From<cudarc::driver::DriverError> for GpuError {
    fn from(e: cudarc::driver::DriverError) -> Self {
        GpuError::Driver(e)
    }
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuError::Driver(e) => write!(f, "cuda driver: {e}"),
            GpuError::Empty => write!(f, "empty batch"),
            GpuError::Ragged { expected, got } => {
                write!(f, "batch mixes qubit counts: expected {expected}, found {got}")
            }
            GpuError::TooManyQubits(n) => write!(f, "n = {n} exceeds the cap of {N_CAP}"),
            GpuError::TooManyColumns(k) => write!(f, "k = {k} exceeds the cap of {K_CAP}"),
            GpuError::WouldOverflow { branches, max_coeff } => write!(
                f,
                "{branches} branches with coefficients up to {max_coeff} could overflow \
                 i128 in the reduction; the fold would still be deterministic but it \
                 would no longer be exact, so it is refused"
            ),
            GpuError::BadBlock(b) => {
                write!(f, "block size {b} must be a nonzero multiple of 32 and at most 1024")
            }
            GpuError::RankDeficient => {
                write!(f, "a branch's R has dependent columns (rank < k): Affine invariant broken")
            }
        }
    }
}

impl std::error::Error for GpuError {}

/// The launch shape. It is a PARAMETER and not a tuning detail, because varying
/// it is how the determinism test earns its name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    pub block: u32,
    pub grid: u32,
}

impl Shape {
    /// The shuffle reduction uses the full warp mask, so a block that is not a
    /// whole number of warps would be undefined behaviour rather than a slower
    /// answer. Refused, in one place, for both entry points.
    fn check(self) -> Result<u32, GpuError> {
        if self.block == 0 || !self.block.is_multiple_of(32) || self.block > 1024 {
            return Err(GpuError::BadBlock(self.block));
        }
        Ok(self.grid.max(1))
    }

    /// A reasonable default for `b` branches: 256-wide blocks, enough of them to
    /// fill the device, capped so the grid-stride loop does the rest.
    pub fn for_batch(b: usize) -> Shape {
        let block = 256u32;
        let grid = ((b as u32).div_ceil(block)).clamp(1, 4096);
        Shape { block, grid }
    }
}

/// A loaded module and its four entry points. One per process is plenty.
pub struct GpuFolder {
    pub ctx: Arc<CudaContext>,
    pub stream: Arc<CudaStream>,
    #[allow(dead_code)]
    module: Arc<CudaModule>,
    fold: [CudaFunction; 2],   // [n<=32, n<=64]
    codes: [CudaFunction; 2],
}

impl GpuFolder {
    pub fn new(ordinal: usize) -> Result<Self, GpuError> {
        let ctx = CudaContext::new(ordinal)?;
        let stream = ctx.default_stream();
        let module = ctx.load_module(Ptx::from_src(PTX))?;
        let fold = [
            module.load_function("fold_affine_n32")?,
            module.load_function("fold_affine_n64")?,
        ];
        let codes = [
            module.load_function("rotation_codes_n32")?,
            module.load_function("rotation_codes_n64")?,
        ];
        Ok(GpuFolder { ctx, stream, module, fold, codes })
    }

    /// Free / total device memory, in bytes.
    pub fn mem_info(&self) -> Result<(usize, usize), GpuError> {
        Ok(cudarc::driver::result::mem_get_info()?)
    }

    pub fn name(&self) -> String {
        self.ctx.name().unwrap_or_else(|_| "<unknown>".into())
    }
}

/// A batch resident on the device. Upload once, fold for many `y`.
pub struct GpuBatch {
    pub n: usize,
    pub b: usize,
    /// The common exponent every `BASE_b` was aligned to: `max_b m_b`.
    pub m_common: i32,
    /// True when every branch already carried `m_common`, so no alignment shift
    /// was applied to any of them. Informative, and NOT the load-bearing
    /// condition — a real `run_pruned` batch has come back false here and
    /// bit-identical anyway.
    pub exponent_uniform: bool,
    /// True when every branch's `m_b` shares one parity. THIS is the condition
    /// under which struct equality with the sequential CPU fold is guaranteed
    /// rather than merely observed; see the module header.
    pub parity_uniform: bool,
    /// Bytes resident on the device for this batch.
    pub bytes: usize,
    rrow: CudaSlice<u64>,
    hbits: CudaSlice<u64>,
    jrow: CudaSlice<u64>,
    dpack: CudaSlice<u64>,
    base: CudaSlice<u64>,
    kk: CudaSlice<u32>,
    /// Which of the two kernel specializations this batch's `n` selects.
    variant: usize,
    /// Reused output buffer and its host landing pad. A per-fold `alloc_zeros`
    /// plus a fresh `Vec` was most of the wall clock on small batches — a
    /// benchmark that measures the allocator instead of the kernel is a
    /// benchmark of the wrong thing. `RefCell` and not `&mut self` because a
    /// `GpuBatch` is uploaded once and folded from a `&` for many `y`; it is
    /// already not `Sync` (a `CudaSlice` is not), so nothing is being widened.
    scratch: RefCell<Scratch>,
}

#[derive(Default)]
struct Scratch {
    dev: Option<CudaSlice<u64>>,
    dev_len: usize,
    host: Vec<u64>,
}

impl GpuBatch {
    /// Transpose the descriptors into the device's strided layout, align the
    /// ring elements to one exponent, and upload.
    pub fn upload(f: &GpuFolder, descs: &[AffineDesc]) -> Result<GpuBatch, GpuError> {
        if descs.is_empty() {
            return Err(GpuError::Empty);
        }
        let n = descs[0].n;
        if n > N_CAP {
            return Err(GpuError::TooManyQubits(n));
        }
        let b = descs.len();
        let kmax = descs.iter().map(|d| d.k).max().unwrap_or(0);
        if kmax > K_CAP {
            return Err(GpuError::TooManyColumns(kmax));
        }
        for d in descs {
            if d.n != n {
                return Err(GpuError::Ragged { expected: n, got: d.n });
            }
        }

        // --- one exponent for the whole batch.
        let m_common = descs.iter().map(|d| d.base.m).max().unwrap_or(0);
        let exponent_uniform = descs.iter().all(|d| d.base.m == m_common);
        let parity_uniform = descs
            .iter()
            .all(|d| d.base.m.rem_euclid(2) == m_common.rem_euclid(2));

        // --- the overflow guard. `b * max|c|` is the worst case for one lane
        //     (every branch's largest coefficient landing on the same lane with
        //     the same sign); refuse rather than wrap.
        let mut max_coeff: u128 = 0;
        let mut base_host = vec![0u64; 8 * b];
        for (i, d) in descs.iter().enumerate() {
            let c = ring::align_to(d.base, m_common);
            max_coeff = max_coeff.max(ring::magnitude(&c));
            for (p, &v) in c.iter().enumerate() {
                let uv = v as u128;
                base_host[(2 * p) * b + i] = uv as u64;
                base_host[(2 * p + 1) * b + i] = (uv >> 64) as u64;
            }
        }
        if max_coeff
            .checked_mul(b as u128)
            .map(|w| w > (1u128 << 126))
            .unwrap_or(true)
        {
            return Err(GpuError::WouldOverflow { branches: b, max_coeff });
        }

        // --- the strided transpose.
        let mut rrow = vec![0u64; n * b];
        let mut hbits = vec![0u64; b];
        let mut jrow = vec![0u64; kmax.max(1) * b];
        let mut dpack = vec![0u64; 2 * b];
        let mut kk = vec![0u32; b];
        for (i, d) in descs.iter().enumerate() {
            for row in 0..n {
                rrow[row * b + i] = d.r_rows[row];
            }
            hbits[i] = d.h;
            for a in 0..d.k {
                jrow[a * b + i] = d.j_rows[a];
            }
            dpack[i] = d.d[0];
            dpack[b + i] = d.d[1];
            kk[i] = if d.zero { u32::MAX } else { d.k as u32 };
        }

        let bytes = (rrow.len() + hbits.len() + jrow.len() + dpack.len() + base_host.len()) * 8
            + kk.len() * 4;
        let s = &f.stream;
        Ok(GpuBatch {
            n,
            b,
            m_common,
            exponent_uniform,
            parity_uniform,
            bytes,
            rrow: s.clone_htod(&rrow)?,
            hbits: s.clone_htod(&hbits)?,
            jrow: s.clone_htod(&jrow)?,
            dpack: s.clone_htod(&dpack)?,
            base: s.clone_htod(&base_host)?,
            kk: s.clone_htod(&kk)?,
            variant: usize::from(n > 32),
            scratch: RefCell::new(Scratch::default()),
        })
    }

    /// `Sigma_b coeff_b * <y|phi_b>`, exact, on the device.
    ///
    /// `shape` is honoured as given — this is not a "suggestion" the launcher
    /// overrides, because the determinism test's whole content is that the
    /// answer does not move when it changes.
    pub fn fold(&self, f: &GpuFolder, y: u64, shape: Shape) -> Result<Cyc, GpuError> {
        let grid = shape.check()?;
        let want = 9 * grid as usize; // 8 limb lanes + the per-block rank flag
        let mut sc = self.scratch.borrow_mut();
        // `!=` and not `<`: `memcpy_dtoh` requires the two lengths to match, so
        // a buffer kept from a LARGER grid is not reusable for a smaller one.
        // (Measured, not reasoned: reusing on `<` failed cudarc's
        // `dst.len() >= src.len()` assertion the moment the determinism test
        // stepped from a 4096-block shape down to a 1-block one.) In production
        // the shape does not change, so this reallocates once.
        if sc.dev_len != want {
            // Every slot is written by every launch, so this never needs zeroing
            // between folds; it is allocated zeroed once and then reused.
            sc.dev = Some(f.stream.alloc_zeros::<u64>(want)?);
            sc.dev_len = want;
            sc.host.resize(want, 0);
        }
        let out = sc.dev.as_mut().expect("scratch allocated above");

        let n = self.n as u32;
        let b = self.b as u32;
        let mut builder = f.stream.launch_builder(&f.fold[self.variant]);
        builder
            .arg(&self.rrow)
            .arg(&self.hbits)
            .arg(&self.jrow)
            .arg(&self.dpack)
            .arg(&self.base)
            .arg(&self.kk)
            .arg(out)
            .arg(&y)
            .arg(&n)
            .arg(&b);
        unsafe {
            builder.launch(LaunchConfig {
                grid_dim: (grid, 1, 1),
                block_dim: (shape.block, 1, 1),
                shared_mem_bytes: 0,
            })
        }?;

        // Split the borrow: the device buffer and the host landing pad are
        // different fields, and the copy needs one of each.
        let sc = &mut *sc;
        let dev = sc.dev.as_ref().expect("allocated above");
        let parts = &mut sc.host[..want];
        f.stream.memcpy_dtoh(dev, parts)?;

        let g = grid as usize;
        if parts[8 * g..9 * g].iter().any(|&v| v != 0) {
            return Err(GpuError::RankDeficient);
        }

        // The block partials, summed in block order. Integer adds, so the order
        // is a convention and not a decision — see the module header.
        let mut c = [0i128; 4];
        for (p, slot) in c.iter_mut().enumerate() {
            let mut acc: i128 = 0;
            for blk in 0..g {
                let lo = parts[(2 * p) * g + blk] as u128;
                let hi = parts[(2 * p + 1) * g + blk] as u128;
                acc = acc.wrapping_add(((hi << 64) | lo) as i128);
            }
            *slot = acc;
        }
        Ok(ring::from_lanes(c, self.m_common))
    }

    /// The per-branch rotation codes, for conformance rather than for use: a sum
    /// can agree while two branch errors cancel, and this is what rules that out.
    pub fn rotation_codes(&self, f: &GpuFolder, y: u64, shape: Shape) -> Result<Vec<u8>, GpuError> {
        let grid = shape.check()?;
        let mut out = f.stream.alloc_zeros::<u8>(self.b)?;
        let n = self.n as u32;
        let b = self.b as u32;
        let mut builder = f.stream.launch_builder(&f.codes[self.variant]);
        builder
            .arg(&self.rrow)
            .arg(&self.hbits)
            .arg(&self.jrow)
            .arg(&self.dpack)
            .arg(&self.kk)
            .arg(&mut out)
            .arg(&y)
            .arg(&n)
            .arg(&b);
        unsafe {
            builder.launch(LaunchConfig {
                grid_dim: (grid, 1, 1),
                block_dim: (shape.block, 1, 1),
                shared_mem_bytes: 0,
            })
        }?;
        let codes = f.stream.clone_dtoh(&out)?;
        if codes.contains(&R_RANK_BAD) {
            return Err(GpuError::RankDeficient);
        }
        Ok(codes)
    }
}
