//! The labelled two-site operator of `q8-mps` on the device — E14 item 5b.
//!
//! [`GpuTwoSite`] is a [`q8_mps::blocks::TwoSiteBackend`]: asked once per local eigensolve
//! for a matvec closure over a [`BlockPlan`], it flattens the plan to its
//! [`CompactPlan`], uploads every table and the three slot arrays ONCE, and then each call
//! moves ψ up, runs the four stage kernels of `kernels/mps_blocks.cu`, and moves Hψ down.
//! Priced before a byte is allocated (`CompactPlan::bytes` against the card's free VRAM
//! minus a reserve) and refused by name; bit-identical to the host reference by gate
//! (`tests/mps_blocks.rs`), and therefore to the dense operator.

use std::sync::Arc;

use cudarc::driver::{CudaContext, CudaFunction, CudaSlice, CudaStream, PushKernelArg};
use cudarc::nvrtc::Ptx;
use q8_mps::blocks::{BlockPlan, CompactPlan, TwoSiteBackend};
use q8_mps::mpo::MpoSite;

use crate::fci::{grid_for, FciGpuError};

const PTX: &str = include_str!("../kernels/mps_blocks.ptx");

/// The device backend: one context, its stream, the four kernels, a VRAM reserve.
pub struct GpuTwoSite {
    ctx: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    k1: CudaFunction,
    k2: CudaFunction,
    k3: CudaFunction,
    k4: CudaFunction,
    reserve_mib: u64,
    /// Every refusal this backend issued, for the caller to read: a refused plan falls back
    /// to the host loops rather than failing the sweep, and the count says how often.
    pub refusals: std::sync::Mutex<Vec<String>>,
}

impl GpuTwoSite {
    pub fn new(ordinal: usize, reserve_mib: u64) -> Result<Self, FciGpuError> {
        let ctx = CudaContext::new(ordinal)?;
        let stream = ctx.default_stream();
        let module = ctx.load_module(Ptx::from_src(PTX))?;
        Ok(Self {
            k1: module.load_function("q8_stage1")?,
            k2: module.load_function("q8_stage2")?,
            k3: module.load_function("q8_stage3")?,
            k4: module.load_function("q8_stage4")?,
            ctx,
            stream,
            reserve_mib,
            refusals: std::sync::Mutex::new(Vec::new()),
        })
    }

    pub fn context(&self) -> &Arc<CudaContext> {
        &self.ctx
    }

    /// Upload a compact plan; refused by name if it does not fit beside the reserve.
    pub fn upload(&self, c: &CompactPlan) -> Result<DevicePlan, FciGpuError> {
        let need = c.bytes();
        let (free, _total) = cudarc::driver::result::mem_get_info()?;
        let free = (free as u64).saturating_sub(self.reserve_mib << 20);
        if need > free {
            return Err(FciGpuError::NotEnoughVram { need_bytes: need, free_bytes: free });
        }
        let s = &self.stream;
        let up_i32 = |v: &[i32]| -> Result<CudaSlice<i32>, FciGpuError> { Ok(if v.is_empty() { s.clone_htod(&[0i32])? } else { s.clone_htod(v)? }) };
        let up_i64 = |v: &[i64]| -> Result<CudaSlice<i64>, FciGpuError> { Ok(if v.is_empty() { s.clone_htod(&[0i64])? } else { s.clone_htod(v)? }) };
        let up_f64 = |v: &[f64]| -> Result<CudaSlice<f64>, FciGpuError> { Ok(if v.is_empty() { s.clone_htod(&[0f64])? } else { s.clone_htod(v)? }) };
        let max_rb = (0..c.n_rblocks()).map(|rb| (c.r_off[rb + 1] - c.r_off[rb]) as usize).max().unwrap_or(1).max(1);
        Ok(DevicePlan {
            chi_l: c.chi_l,
            chi_r: c.chi_r,
            d_l: c.d_l,
            d_mid: c.d_mid,
            nr: c.right_chan.len(),
            nl: c.left_chan.len(),
            nlb: c.n_lblocks(),
            nrb: c.n_rblocks(),
            max_rb,
            l_idx: up_i32(&c.l_idx)?,
            l_off: up_i32(&c.l_off)?,
            l_block_of: up_i32(&c.l_block_of)?,
            l_pos_of: up_i32(&c.l_pos_of)?,
            r_idx: up_i32(&c.r_idx)?,
            r_off: up_i32(&c.r_off)?,
            cut: up_i32(&c.cut)?,
            right_map: up_i32(&c.right_map)?,
            rtile: up_f64(&c.rtile)?,
            rtile_off: up_i64(&c.rtile_off)?,
            left_chan: up_i32(&c.left_chan)?,
            left_map: up_i32(&c.left_map)?,
            ltile: up_f64(&c.ltile)?,
            ltile_off: up_i64(&c.ltile_off)?,
            t1_off: up_i64(&c.t1_off)?,
            t2_off: up_i64(&c.t2_off)?,
            t2_rb: up_i32(&c.t2_rb)?,
            t2_contrib_off: up_i32(&c.t2_contrib_off)?,
            t2_contrib_ci: up_i32(&c.t2_contrib_ci)?,
            t2_contrib_b: up_i32(&c.t2_contrib_b)?,
            t2_contrib_w: up_f64(&c.t2_contrib_w)?,
            t3_off: up_i64(&c.t3_off)?,
            t3_rb: up_i32(&c.t3_rb)?,
            t3_contrib_off: up_i32(&c.t3_contrib_off)?,
            t3_contrib_c1p: up_i32(&c.t3_contrib_c1p)?,
            t3_contrib_a: up_i32(&c.t3_contrib_a)?,
            t3_contrib_w: up_f64(&c.t3_contrib_w)?,
            t1: unsafe { s.alloc::<f64>(c.t1_len.max(1))? },
            t2: unsafe { s.alloc::<f64>(c.t2_len.max(1))? },
            t3: unsafe { s.alloc::<f64>(c.t3_len.max(1))? },
            psi: unsafe { s.alloc::<f64>(c.chi_l * 4 * c.chi_r)? },
            out: s.alloc_zeros::<f64>(c.chi_l * 4 * c.chi_r)?,
        })
    }

    /// One matvec on an uploaded plan: ψ up, four launches, Hψ down.
    pub fn apply(&self, p: &mut DevicePlan, psi: &[f64]) -> Result<Vec<f64>, FciGpuError> {
        let n = p.chi_l * 4 * p.chi_r;
        assert_eq!(psi.len(), n);
        let s = &self.stream;
        s.memcpy_htod(psi, &mut p.psi)?;
        // out is fully rewritten only at live elements; dead elements must read 0 every call
        s.memset_zeros(&mut p.out)?;
        let (chi_l, chi_r, nlb, nrb, max_rb) = (p.chi_l as i32, p.chi_r as i32, p.nlb as i32, p.nrb as i32, p.max_rb as i32);
        {
            let nr = p.nr as i32;
            let total = (p.nr * p.chi_l * 4 * p.max_rb) as i64;
            let mut b = s.launch_builder(&self.k1);
            b.arg(&mut p.t1).arg(&p.psi).arg(&nr).arg(&chi_l).arg(&chi_r).arg(&nrb).arg(&max_rb)
                .arg(&p.l_block_of).arg(&p.cut).arg(&p.r_idx).arg(&p.r_off)
                .arg(&p.right_map).arg(&p.rtile).arg(&p.rtile_off).arg(&p.t1_off);
            unsafe { b.launch(grid_for(total))? };
        }
        {
            let d_mid = p.d_mid as i32;
            let total = (p.d_mid * p.chi_l * 4 * p.max_rb) as i64;
            let mut b = s.launch_builder(&self.k2);
            b.arg(&mut p.t2).arg(&p.t1).arg(&d_mid).arg(&chi_l).arg(&nlb).arg(&max_rb)
                .arg(&p.l_block_of).arg(&p.r_off).arg(&p.t2_rb).arg(&p.t2_off)
                .arg(&p.t2_contrib_off).arg(&p.t2_contrib_ci).arg(&p.t2_contrib_b).arg(&p.t2_contrib_w)
                .arg(&p.t1_off);
            unsafe { b.launch(grid_for(total))? };
        }
        {
            let d_l = p.d_l as i32;
            let total = (p.d_l * p.chi_l * 4 * p.max_rb) as i64;
            let mut b = s.launch_builder(&self.k3);
            b.arg(&mut p.t3).arg(&p.t2).arg(&d_l).arg(&chi_l).arg(&nlb).arg(&max_rb)
                .arg(&p.l_block_of).arg(&p.r_off).arg(&p.t3_rb).arg(&p.t3_off)
                .arg(&p.t3_contrib_off).arg(&p.t3_contrib_c1p).arg(&p.t3_contrib_a).arg(&p.t3_contrib_w)
                .arg(&p.t2_off);
            unsafe { b.launch(grid_for(total))? };
        }
        {
            let nl = p.nl as i32;
            let total = (p.chi_l * 4 * p.max_rb) as i64;
            let mut b = s.launch_builder(&self.k4);
            b.arg(&mut p.out).arg(&p.t3).arg(&nl).arg(&chi_l).arg(&chi_r).arg(&nlb).arg(&max_rb)
                .arg(&p.l_block_of).arg(&p.l_pos_of).arg(&p.l_idx).arg(&p.l_off)
                .arg(&p.r_idx).arg(&p.r_off).arg(&p.cut)
                .arg(&p.left_chan).arg(&p.left_map).arg(&p.ltile).arg(&p.ltile_off)
                .arg(&p.t3_off);
            unsafe { b.launch(grid_for(total))? };
        }
        let mut out = vec![0.0; n];
        s.memcpy_dtoh(&p.out, &mut out)?;
        Ok(out)
    }
}

/// An uploaded plan: every table and the slot arrays resident on the card.
pub struct DevicePlan {
    chi_l: usize,
    chi_r: usize,
    d_l: usize,
    d_mid: usize,
    nr: usize,
    nl: usize,
    nlb: usize,
    nrb: usize,
    max_rb: usize,
    l_idx: CudaSlice<i32>,
    l_off: CudaSlice<i32>,
    l_block_of: CudaSlice<i32>,
    l_pos_of: CudaSlice<i32>,
    r_idx: CudaSlice<i32>,
    r_off: CudaSlice<i32>,
    cut: CudaSlice<i32>,
    right_map: CudaSlice<i32>,
    rtile: CudaSlice<f64>,
    rtile_off: CudaSlice<i64>,
    left_chan: CudaSlice<i32>,
    left_map: CudaSlice<i32>,
    ltile: CudaSlice<f64>,
    ltile_off: CudaSlice<i64>,
    t1_off: CudaSlice<i64>,
    t2_off: CudaSlice<i64>,
    t2_rb: CudaSlice<i32>,
    t2_contrib_off: CudaSlice<i32>,
    t2_contrib_ci: CudaSlice<i32>,
    t2_contrib_b: CudaSlice<i32>,
    t2_contrib_w: CudaSlice<f64>,
    t3_off: CudaSlice<i64>,
    t3_rb: CudaSlice<i32>,
    t3_contrib_off: CudaSlice<i32>,
    t3_contrib_c1p: CudaSlice<i32>,
    t3_contrib_a: CudaSlice<i32>,
    t3_contrib_w: CudaSlice<f64>,
    t1: CudaSlice<f64>,
    t2: CudaSlice<f64>,
    t3: CudaSlice<f64>,
    psi: CudaSlice<f64>,
    out: CudaSlice<f64>,
}

impl TwoSiteBackend for GpuTwoSite {
    fn matvec<'a>(&'a self, plan: &'a BlockPlan, w1: &'a MpoSite, w2: &'a MpoSite) -> Box<dyn Fn(&[f64]) -> Vec<f64> + 'a> {
        // a plan the card cannot hold, or one whose layout is refused, runs on the host loops
        // and is COUNTED: a silent fallback would let a device claim cover a host run
        let dev = CompactPlan::build(plan, w1, w2).and_then(|c| match self.upload(&c) {
            Ok(d) => Some(std::sync::Mutex::new(d)),
            Err(e) => {
                self.refusals.lock().unwrap().push(format!("{e:?}"));
                None
            }
        });
        let left_dummy: q8_mps::mps::Env = Vec::new();
        let _ = left_dummy;
        Box::new(move |psi: &[f64]| match &dev {
            Some(d) => self.apply(&mut d.lock().unwrap(), psi).expect("a launch on an uploaded plan"),
            None => plan.apply_host_only(w1, w2, psi),
        })
    }
}
