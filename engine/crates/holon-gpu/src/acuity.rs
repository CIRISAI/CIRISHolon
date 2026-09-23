//! **The budgeted prefix, folded on the device** — QVM-GPUFOLD-1's backend for
//! `holon::acuity`.
//!
//! `holon::acuity` decides WHAT to fold: `BudgetPlan::stop_at(ε)` names a prefix
//! `order[..k]` of the declared branch order, on the host, from the plan alone.
//! This module only folds that prefix, and does it with the machinery the rest
//! of this crate already gates: every branch decoded ONCE into an
//! [`AffineDesc`] (y-independent, like the plan), the prefix's descriptors
//! gathered in plan order and uploaded as ONE [`GpuBatch`], and
//! [`GpuBatch::fold`] run for the query. The batch stays resident for as long as
//! the same prefix is asked for, so the many amplitudes an observable needs pay
//! the upload once.
//!
//! Two implementations of `holon::acuity::DeviceFold`, and they declare
//! DIFFERENT classes on purpose:
//!
//! * [`GpuBranchFold`] — the device. Class `Gpu`.
//! * [`CpuTwinFold`] — the same descriptors folded on the host by
//!   [`crate::cpu::fold_packed`]. Class `Cpu`. It is the CPU twin that convicts a
//!   corrupted device lane (PG-1), and it is what G2 uses to show that a host
//!   run of the same plan cannot be passed off as a device artifact even when
//!   it goes through the same `DeviceFold` door.
//!
//! No fallback: a [`GpuBranchFold`] that cannot reach its device returns an
//! error through `DeviceFoldError`, and the budgeted call fails with it.

use std::cell::RefCell;

use holon::acuity::{AffineBranches, DeviceClass, DeviceFold, DeviceFoldError};
use holon::ledger::Cyc;

use crate::cpu;
use crate::desc::{pack_y, AffineDesc, DescError};
use crate::gpu::{GpuBatch, GpuError, GpuFolder, Shape};

/// Every branch of `src`, decoded in branch-index order. One affine state
/// clone and one `canon_key` decode per branch; paid once per source.
pub fn descs_of<S: AffineBranches + ?Sized>(src: &S) -> Result<Vec<AffineDesc>, DescError> {
    (0..src.n_branches())
        .map(|b| {
            let (w, st) = src.affine_branch(b);
            AffineDesc::from_branch(w, &st)
        })
        .collect()
}

fn gather(descs: &[AffineDesc], prefix: &[u64]) -> Vec<AffineDesc> {
    prefix.iter().map(|&b| descs[b as usize].clone()).collect()
}

fn gpu_err(e: GpuError) -> DeviceFoldError {
    DeviceFoldError { class: DeviceClass::Gpu, why: e.to_string() }
}

/// The prefix a batch was uploaded for, and the batch. Keyed on the WHOLE
/// prefix (the branch indices in order), not on its length: two plans with the
/// same `k` are different batches.
struct Resident {
    prefix: Vec<u64>,
    batch: GpuBatch,
}

/// What the resident batch looked like on upload — the flags `gpu.rs`'s header
/// says the struct-equality argument rests on, reported rather than assumed.
#[derive(Clone, Copy, Debug)]
pub struct ResidentInfo {
    pub branches: usize,
    pub m_common: i32,
    pub exponent_uniform: bool,
    pub parity_uniform: bool,
    pub bytes: usize,
}

/// A device lane that disagrees with the host's twin, named.
#[derive(Clone, Debug)]
pub struct LaneConviction {
    /// The device's fold of the prefix, and the CPU twin's.
    pub device: Cyc,
    pub twin: Cyc,
    /// Every resident limb that is not what the host uploaded:
    /// `(prefix position, branch index, limb)`, limb `2p + hi` of lane `p`.
    pub lanes: Vec<(usize, u64, usize)>,
}

impl std::fmt::Display for LaneConviction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "device fold {:?} != CPU twin {:?}", self.device, self.twin)?;
        for (pos, br, limb) in &self.lanes {
            write!(
                f,
                "; CONVICTED prefix position {pos} (branch {br}), lane {} {} limb",
                limb / 2,
                if limb % 2 == 0 { "low" } else { "high" }
            )?;
        }
        Ok(())
    }
}

/// The GPU backend: one source's branches as descriptors, and a resident batch.
pub struct GpuBranchFold<'f> {
    folder: &'f GpuFolder,
    descs: Vec<AffineDesc>,
    n_qubits: usize,
    /// `None` = [`Shape::for_batch`]. Settable because PG-2's whole content is
    /// that changing it changes nothing.
    shape: RefCell<Option<Shape>>,
    resident: RefCell<Option<Resident>>,
}

impl<'f> GpuBranchFold<'f> {
    /// Decode every branch of `src` for the device. Fails on a branch the
    /// descriptor cannot hold (more than 64 wires), never truncates.
    pub fn new<S: AffineBranches + ?Sized>(
        folder: &'f GpuFolder,
        src: &S,
    ) -> Result<Self, DescError> {
        Ok(Self::from_descs(folder, descs_of(src)?, src.n_qubits()))
    }

    pub fn from_descs(folder: &'f GpuFolder, descs: Vec<AffineDesc>, n_qubits: usize) -> Self {
        GpuBranchFold {
            folder,
            descs,
            n_qubits,
            shape: RefCell::new(None),
            resident: RefCell::new(None),
        }
    }

    pub fn descs(&self) -> &[AffineDesc] {
        &self.descs
    }

    pub fn set_shape(&self, shape: Option<Shape>) {
        *self.shape.borrow_mut() = shape;
    }

    fn shape_for(&self, b: usize) -> Shape {
        self.shape.borrow().unwrap_or_else(|| Shape::for_batch(b))
    }

    /// Drop the resident batch, so the next fold pays the upload (the COLD
    /// query G3 reports).
    pub fn evict(&self) {
        *self.resident.borrow_mut() = None;
    }

    /// Upload `prefix` unless it is already the resident one. `prefix` must be
    /// nonempty (an empty prefix never reaches the device).
    pub fn ensure_resident(&self, prefix: &[u64]) -> Result<ResidentInfo, GpuError> {
        let mut slot = self.resident.borrow_mut();
        let stale = match slot.as_ref() {
            Some(r) => r.prefix.as_slice() != prefix,
            None => true,
        };
        if stale {
            *slot = None; // free the old batch before allocating the new one
            let batch = GpuBatch::upload(self.folder, &gather(&self.descs, prefix))?;
            *slot = Some(Resident { prefix: prefix.to_vec(), batch });
        }
        let b = &slot.as_ref().expect("resident above").batch;
        Ok(ResidentInfo {
            branches: b.b,
            m_common: b.m_common,
            exponent_uniform: b.exponent_uniform,
            parity_uniform: b.parity_uniform,
            bytes: b.bytes,
        })
    }

    /// The resident batch's fold at `y` with the kernel's own event time — the
    /// bench's warm query. `prefix` must already be resident.
    pub fn fold_resident_timed(&self, prefix: &[u64], y: &[bool]) -> Result<(Cyc, f32), GpuError> {
        self.ensure_resident(prefix)?;
        let slot = self.resident.borrow();
        let r = slot.as_ref().expect("resident above");
        r.batch.fold_timed(self.folder, pack_y(y), self.shape_for(r.batch.b))
    }

    /// PG-1: XOR `mask` into ONE resident limb of prefix position `position`,
    /// on the device. `prefix` is made resident first.
    pub fn plant_lane(
        &self,
        prefix: &[u64],
        position: usize,
        limb: usize,
        mask: u64,
    ) -> Result<(), GpuError> {
        assert_ne!(mask, 0, "a zero mask plants nothing");
        self.ensure_resident(prefix)?;
        let mut slot = self.resident.borrow_mut();
        let r = slot.as_mut().expect("resident above");
        let expected = r.batch.expected_base(&gather(&self.descs, prefix));
        let v = expected[limb * r.batch.b + position] ^ mask;
        r.batch.plant_base_limb(self.folder, position, limb, v)
    }

    /// The CPU twin's audit of the resident prefix at `y`: the device fold
    /// against [`cpu::fold_packed`] over the same descriptors, and — whatever
    /// the folds say — every resident limb against what the host uploaded.
    /// `None` when both agree.
    pub fn audit(
        &self,
        prefix: &[u64],
        y: &[bool],
    ) -> Result<Option<LaneConviction>, DeviceFoldError> {
        let gathered = gather(&self.descs, prefix);
        let twin = cpu::fold_packed(&gathered, pack_y(y), 1);
        // A fold that FAILS inside an audit is not a conviction — it is the
        // device failing, and it propagates as the error it is.
        let device = self.fold_prefix(prefix, y)?;
        if prefix.is_empty() {
            return Ok((device != twin).then(|| LaneConviction { device, twin, lanes: vec![] }));
        }
        let slot = self.resident.borrow();
        let r = slot.as_ref().expect("resident after a nonempty fold");
        let on_card = r.batch.read_base(self.folder).map_err(gpu_err)?;
        let want = r.batch.expected_base(&gathered);
        let b = r.batch.b;
        let mut lanes = Vec::new();
        for (i, (&got, &exp)) in on_card.iter().zip(want.iter()).enumerate() {
            if got != exp {
                let (limb, pos) = (i / b, i % b);
                lanes.push((pos, prefix[pos], limb));
            }
        }
        if device == twin && lanes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(LaneConviction { device, twin, lanes }))
        }
    }
}

impl DeviceFold for GpuBranchFold<'_> {
    fn class(&self) -> DeviceClass {
        DeviceClass::Gpu
    }
    fn n_branches(&self) -> u64 {
        self.descs.len() as u64
    }
    fn n_qubits(&self) -> usize {
        self.n_qubits
    }
    fn fold_prefix(&self, prefix: &[u64], y: &[bool]) -> Result<Cyc, DeviceFoldError> {
        if prefix.is_empty() {
            return Ok(Cyc::ZERO); // `MergeLedger::empty` for `Cyc`, the mesh's empty fold
        }
        self.ensure_resident(prefix).map_err(gpu_err)?;
        let slot = self.resident.borrow();
        let r = slot.as_ref().expect("resident above");
        r.batch
            .fold(self.folder, pack_y(y), self.shape_for(r.batch.b))
            .map_err(gpu_err)
    }
}

/// The CPU twin: the same descriptors, folded on the host. Declares `Cpu`.
pub struct CpuTwinFold {
    descs: Vec<AffineDesc>,
    n_qubits: usize,
    shards: usize,
}

impl CpuTwinFold {
    pub fn new<S: AffineBranches + ?Sized>(src: &S, shards: usize) -> Result<Self, DescError> {
        Ok(CpuTwinFold { descs: descs_of(src)?, n_qubits: src.n_qubits(), shards })
    }
    pub fn from_descs(descs: Vec<AffineDesc>, n_qubits: usize, shards: usize) -> Self {
        CpuTwinFold { descs, n_qubits, shards }
    }
}

impl DeviceFold for CpuTwinFold {
    fn class(&self) -> DeviceClass {
        DeviceClass::Cpu
    }
    fn n_branches(&self) -> u64 {
        self.descs.len() as u64
    }
    fn n_qubits(&self) -> usize {
        self.n_qubits
    }
    fn fold_prefix(&self, prefix: &[u64], y: &[bool]) -> Result<Cyc, DeviceFoldError> {
        Ok(cpu::fold_packed(&gather(&self.descs, prefix), pack_y(y), self.shards))
    }
}
