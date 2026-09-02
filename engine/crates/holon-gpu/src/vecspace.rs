//! The vector space on the device: `holon_chem::vecspace::VectorSpace<f64>` over device-resident
//! vectors, every row program a kernel in `kernels/vec.cu`, every reduction under the same block
//! law as the host — so the Davidson body in `holon_chem::tier::davidson_in` runs on the card
//! unchanged and lands on the same bits the host does (measured, `tests/vecspace.rs`).
//!
//! The card holds the basis, its images, the diagonal and the work vectors; the host holds the
//! `m×m` Gram matrix and its eigenproblem. Per iteration the bus carries `m` scalars up and
//! `m + 1` scalars down. The physics kernel (`GpuLaneSigma::apply_on`) applies to the same
//! resident vectors, so a whole solve never copies a vector to the host until the answer.

use std::cell::RefCell;
use std::sync::Arc;

use cudarc::driver::{CudaContext, CudaFunction, CudaSlice, CudaStream, DevicePtr, PushKernelArg};
use cudarc::nvrtc::Ptx;
use holon_chem::vecspace::{VectorSpace, DOT_BLOCK};

use crate::fci::{grid_for, FciGpuError};

const PTX: &str = include_str!("../kernels/vec.ptx");
const CU: &str = include_str!("../kernels/vec.cu");

/// The largest subspace the row programs hold in registers; `HOLON_MAX_M` in the kernel.
pub const MAX_M: usize = 48;

fn kernel_define(name: &str) -> usize {
    let key = format!("#define {name} ");
    CU.lines()
        .find_map(|l| l.strip_prefix(key.as_str()))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or_else(|| panic!("vec.cu declares {name}"))
}

pub struct DeviceSpace {
    pub stream: Arc<CudaStream>,
    f_dot: CudaFunction,
    f_sum: CudaFunction,
    f_scale: CudaFunction,
    f_axpy: CudaFunction,
    f_ritz: CudaFunction,
    f_deflate: CudaFunction,
    f_deflate_norm: CudaFunction,
    f_gram: CudaFunction,
    partials: RefCell<CudaSlice<f64>>,
    out: RefCell<CudaSlice<f64>>,
}

impl DeviceSpace {
    pub fn new(ctx: &Arc<CudaContext>) -> Result<DeviceSpace, FciGpuError> {
        assert_eq!(kernel_define("HOLON_DOT_BLOCK"), DOT_BLOCK, "vec.cu's block and vecspace::DOT_BLOCK have drifted apart");
        assert_eq!(kernel_define("HOLON_MAX_M"), MAX_M, "vec.cu's HOLON_MAX_M and DeviceSpace::MAX_M have drifted apart");
        ctx.bind_to_thread()?;
        let stream = ctx.new_stream()?;
        let module = ctx.load_module(Ptx::from_src(PTX))?;
        let f = |n: &str| module.load_function(n);
        let partials = stream.alloc_zeros::<f64>(1)?;
        let out = stream.alloc_zeros::<f64>(MAX_M + 1)?;
        Ok(DeviceSpace {
            f_dot: f("holon_vec_partials_dot")?,
            f_sum: f("holon_vec_sum_partials")?,
            f_scale: f("holon_vec_scale")?,
            f_axpy: f("holon_vec_axpy")?,
            f_ritz: f("holon_vec_ritz")?,
            f_deflate: f("holon_vec_deflate")?,
            f_deflate_norm: f("holon_vec_deflate_norm")?,
            f_gram: f("holon_vec_gram")?,
            stream,
            partials: RefCell::new(partials),
            out: RefCell::new(out),
        })
    }

    fn nblocks(n: usize) -> usize {
        n.div_ceil(DOT_BLOCK).max(1)
    }

    /// The partials buffer, grown to `need` doubles when the space is first asked for more.
    fn ensure_partials(&self, need: usize) {
        let mut p = self.partials.borrow_mut();
        if p.len() < need {
            *p = self.stream.alloc_zeros::<f64>(need).expect("device partials");
        }
    }

    /// The device addresses of `m` resident vectors, as a small table the kernels index.
    fn ptr_table(&self, vs: &[CudaSlice<f64>]) -> CudaSlice<u64> {
        assert!(vs.len() <= MAX_M, "{} basis vectors exceed the kernels' register array ({MAX_M})", vs.len());
        let ptrs: Vec<u64> = vs
            .iter()
            .map(|v| {
                let (p, _guard) = v.device_ptr(&self.stream);
                p
            })
            .collect();
        let table = if ptrs.is_empty() { vec![0u64] } else { ptrs };
        self.stream.clone_htod(&table).expect("pointer table upload")
    }

    /// The law's second half on the device: `k` reductions summed in block order, downloaded.
    fn reduce(&self, nblocks: usize, k: usize) -> Vec<f64> {
        let nb = nblocks as i64;
        let kk = k as i32;
        let p = self.partials.borrow();
        let mut out = self.out.borrow_mut();
        {
            let mut b = self.stream.launch_builder(&self.f_sum);
            b.arg(&*p).arg(&nb).arg(&kk).arg(&mut *out);
            let cfg = cudarc::driver::LaunchConfig { grid_dim: (1, 1, 1), block_dim: (MAX_M as u32 + 1, 1, 1), shared_mem_bytes: 0 };
            unsafe { b.launch(cfg).expect("sum launch") };
        }
        let mut host = vec![0.0f64; MAX_M + 1];
        self.stream.memcpy_dtoh(&*out, &mut host).expect("reduction download");
        self.stream.synchronize().expect("device sync");
        host.truncate(k);
        host
    }
}

impl VectorSpace<f64> for DeviceSpace {
    type V = CudaSlice<f64>;

    fn len(&self, v: &CudaSlice<f64>) -> usize {
        v.len()
    }
    fn zeros(&self, n: usize) -> CudaSlice<f64> {
        self.stream.alloc_zeros::<f64>(n.max(1)).expect("device vector")
    }
    fn upload(&self, s: &[f64]) -> CudaSlice<f64> {
        if s.is_empty() {
            return self.zeros(1);
        }
        self.stream.clone_htod(s).expect("device upload")
    }
    fn download(&self, v: &CudaSlice<f64>) -> Vec<f64> {
        let mut h = vec![0.0f64; v.len()];
        self.stream.memcpy_dtoh(v, &mut h).expect("device download");
        self.stream.synchronize().expect("device sync");
        h
    }
    fn copy(&self, v: &CudaSlice<f64>) -> CudaSlice<f64> {
        let mut out = self.stream.alloc_zeros::<f64>(v.len()).expect("device vector");
        self.stream.memcpy_dtod(v, &mut out).expect("device copy");
        out
    }

    fn dot(&self, a: &CudaSlice<f64>, b: &CudaSlice<f64>) -> f64 {
        assert_eq!(a.len(), b.len());
        let n = a.len();
        let nb = Self::nblocks(n);
        self.ensure_partials(nb);
        {
            let mut p = self.partials.borrow_mut();
            let nn = n as i64;
            let mut lb = self.stream.launch_builder(&self.f_dot);
            lb.arg(a).arg(b).arg(&mut *p).arg(&nn);
            unsafe { lb.launch(grid_for(nb as i64)).expect("dot launch") };
        }
        self.reduce(nb, 1)[0]
    }

    fn scale(&self, a: f64, x: &mut CudaSlice<f64>) {
        let n = x.len() as i64;
        let mut lb = self.stream.launch_builder(&self.f_scale);
        lb.arg(&a).arg(x).arg(&n);
        unsafe { lb.launch(grid_for(n)).expect("scale launch") };
    }

    fn axpy(&self, a: f64, x: &CudaSlice<f64>, y: &mut CudaSlice<f64>) {
        assert_eq!(x.len(), y.len());
        let n = y.len() as i64;
        let mut lb = self.stream.launch_builder(&self.f_axpy);
        lb.arg(&a).arg(x).arg(y).arg(&n);
        unsafe { lb.launch(grid_for(n)).expect("axpy launch") };
    }

    fn ritz(&self, basis: &[CudaSlice<f64>], hbasis: &[CudaSlice<f64>], y: &[f64], theta: f64, diag: &CudaSlice<f64>, x: &mut CudaSlice<f64>, r: &mut CudaSlice<f64>, corr: &mut CudaSlice<f64>) -> (f64, Vec<f64>) {
        let m = basis.len();
        let n = diag.len();
        let nb = Self::nblocks(n);
        self.ensure_partials(nb * (1 + m));
        let bp = self.ptr_table(basis);
        let hbp = self.ptr_table(hbasis);
        let yd = self.upload(y);
        {
            let mut p = self.partials.borrow_mut();
            let (mm, nn) = (m as i32, n as i64);
            let mut lb = self.stream.launch_builder(&self.f_ritz);
            lb.arg(&bp).arg(&hbp).arg(&mm).arg(&yd).arg(&theta).arg(diag).arg(x).arg(r).arg(corr).arg(&mut *p).arg(&nn);
            unsafe { lb.launch(grid_for(nb as i64)).expect("ritz launch") };
        }
        let red = self.reduce(nb, 1 + m);
        (red[0], red[1..].to_vec())
    }

    fn deflate(&self, basis: &[CudaSlice<f64>], p: &[f64], w: &mut CudaSlice<f64>) -> Vec<f64> {
        let m = basis.len();
        let n = w.len();
        let nb = Self::nblocks(n);
        self.ensure_partials(nb * m.max(1));
        let bp = self.ptr_table(basis);
        let pd = self.upload(p);
        {
            let mut part = self.partials.borrow_mut();
            let (mm, nn) = (m as i32, n as i64);
            let mut lb = self.stream.launch_builder(&self.f_deflate);
            lb.arg(&bp).arg(&mm).arg(&pd).arg(w).arg(&mut *part).arg(&nn);
            unsafe { lb.launch(grid_for(nb as i64)).expect("deflate launch") };
        }
        self.reduce(nb, m)
    }

    fn deflate_norm(&self, basis: &[CudaSlice<f64>], p: &[f64], w: &mut CudaSlice<f64>) -> f64 {
        let m = basis.len();
        let n = w.len();
        let nb = Self::nblocks(n);
        self.ensure_partials(nb);
        let bp = self.ptr_table(basis);
        let pd = self.upload(p);
        {
            let mut part = self.partials.borrow_mut();
            let (mm, nn) = (m as i32, n as i64);
            let mut lb = self.stream.launch_builder(&self.f_deflate_norm);
            lb.arg(&bp).arg(&mm).arg(&pd).arg(w).arg(&mut *part).arg(&nn);
            unsafe { lb.launch(grid_for(nb as i64)).expect("deflate_norm launch") };
        }
        self.reduce(nb, 1)[0]
    }

    fn gram_row(&self, basis: &[CudaSlice<f64>], v: &CudaSlice<f64>) -> Vec<f64> {
        let m = basis.len();
        let n = v.len();
        let nb = Self::nblocks(n);
        self.ensure_partials(nb * m.max(1));
        let bp = self.ptr_table(basis);
        {
            let mut part = self.partials.borrow_mut();
            let (mm, nn) = (m as i32, n as i64);
            let mut lb = self.stream.launch_builder(&self.f_gram);
            lb.arg(&bp).arg(&mm).arg(v).arg(&mut *part).arg(&nn);
            unsafe { lb.launch(grid_for(nb as i64)).expect("gram launch") };
        }
        self.reduce(nb, m)
    }
}
