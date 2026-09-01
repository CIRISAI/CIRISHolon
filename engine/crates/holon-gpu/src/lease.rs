//! **The GPU as a leasable resource**: VRAM leased through `holon-resource` before a device
//! operator exists, with the lease's QUANTITATIVE boundary declared (RESOURCE_DESIGN **D3b**)
//! and a mid-solve device failure CONVICTED rather than swallowed (**D9**).
//!
//! # Why the lease comes before the operator and not after
//!
//! `GpuFciSigma::new` computes its own footprint and refuses if the card cannot hold it. That is
//! necessary and it is not a lease: it records nothing, it is invisible to the audit, and two
//! operators built in the same process would each pass their own check and then contend. A lease
//! is an ENTRY IN THE PARENT'S BOOKS — probed at birth, ledgered while it lives, released when
//! the need ends — and the books are what the audit reads.
//!
//! # What the lease guarantees, which is less than it looks like
//!
//! D3: a lease is a receipt for rent paid, not a promise about the future. The probe buys
//! validity NOW; every USE is itself a probe; the write that fails is the authoritative reading.
//! For VRAM that is not a nicety — another process can take the card between the probe and the
//! allocation, and the honest response is refuse-and-release rather than retry-forever.
//!
//! D3b: the lease states a quantitative boundary — the MEBIBYTES this operator will hold,
//! computed from the space rather than guessed — and the receipt records where it stopped. A
//! caller needing a bigger space does not get it by editing a constant; it leases again, and the
//! second lease can be refused.
//!
//! # REFUSED and CONVICTED are different, and the difference is the whole file
//!
//! **REFUSED** is *we asked and the answer was no* — normal, cheap, frequent, and what a
//! too-large space gets. **CONVICTED** is *we held a valid lease and the resource went away
//! underneath us* — a violation the audit must see. A device failure in the middle of a solve is
//! the second kind, and reporting it as an ordinary error would lose exactly the distinction the
//! ledger exists to keep.

use std::sync::Arc;

use cudarc::driver::CudaContext;
use holon_chem::fci::{CiInts, FciSpace};
use holon_chem::sigma_op::{DeviceClass, SigmaOp, SigmaProvider};
use holon_resource::probe::ResourceKind;
use holon_resource::{Arena, LeaseError, LeaseId};

use crate::fci::{vram_bytes_for, FciGpuError, GpuFciSigma};
use crate::probe::VramProbe;

/// One mebibyte, the unit the VRAM lease is denominated in.
///
/// Bytes would not fit the ledger's integer story comfortably at this scale, and the receipt is
/// read by humans. The conversion rounds UP: a lease that asked for less than it takes is a
/// lease that did not bound anything.
const MIB: u64 = 1 << 20;

/// A VRAM lease that has been granted and not yet used.
///
/// Carries its DECLARED QUANTITATIVE BOUNDARY (D3b): what it covers, in MiB, computed from the
/// space rather than guessed. It is deliberately not `Copy` — a lease used twice is two holders
/// on one entry, and the books would not know.
#[derive(Debug)]
pub struct VramLease {
    pub id: LeaseId,
    pub mib: u64,
}

/// A GPU-VRAM lease, and the operator held through it.
///
/// The operator is INSIDE the lease rather than beside it, because that is the relationship: the
/// device memory is held *through* the lease, so releasing the lease and dropping the operator
/// are one event and cannot get out of step.
pub struct LeasedGpuSigma {
    lease: LeaseId,
    op: GpuFciSigma,
    /// The declared quantitative boundary — D3b. What this lease covers and nothing beyond.
    pub mib: u64,
}

impl LeasedGpuSigma {
    pub fn lease_id(&self) -> LeaseId {
        self.lease
    }
    pub fn op(&mut self) -> &mut GpuFciSigma {
        &mut self.op
    }
}

impl SigmaOp<f64> for LeasedGpuSigma {
    fn n_det(&self) -> usize {
        self.op.n_det()
    }
    fn device(&self) -> DeviceClass {
        DeviceClass::Gpu
    }
    fn apply(&mut self, c: &[f64], sigma: &mut [f64]) {
        self.op.apply(c, sigma)
    }
}

/// Why a leased device operator could not be produced.
#[derive(Debug)]
pub enum LeasedGpuError {
    /// The lease layer said no. Normal and cheap — the card is busy, or the ask is too big.
    Lease(LeaseError),
    /// The lease was granted and the device still could not build the operator. This is the
    /// interesting one: the probe passed and the real thing failed, which D3(2) says is the
    /// authoritative reading, and the lease is CONVICTED rather than released.
    ConvictedOnBuild { lease: LeaseId, why: FciGpuError },
}

impl std::fmt::Display for LeasedGpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeasedGpuError::Lease(e) => write!(f, "{}", e.message()),
            LeasedGpuError::ConvictedOnBuild { lease, why } => write!(
                f,
                "CONVICTED lease {lease}: the VRAM probe passed and the allocation then failed \
                 — {why}. The probe bought validity at probe time and nothing after; the \
                 allocation that failed is the authoritative reading."
            ),
        }
    }
}

/// The leasing provider: **probe, lease, then build** — and refuse at any of the three.
///
/// This is `holon-resource`'s D11 (*consult, then probe, then lease*) with the consult step
/// belonging to the caller who chose the device class: by the time a solve reaches here the
/// class is DECLARED on the artifact, so there is no crossover to consult and no choice for
/// dispatch to make. What remains is the probe and the lease, and both can still say no.
pub struct LeasedGpuProvider<'a> {
    ctx: Arc<CudaContext>,
    arena: std::cell::RefCell<&'a mut Arena>,
    probe: std::cell::RefCell<VramProbe>,
    parent: Option<LeaseId>,
}

impl<'a> LeasedGpuProvider<'a> {
    pub fn new(
        ordinal: usize,
        arena: &'a mut Arena,
        parent: Option<LeaseId>,
    ) -> Result<LeasedGpuProvider<'a>, cudarc::driver::DriverError> {
        let ctx = CudaContext::new(ordinal)?;
        let probe = VramProbe::on(ctx.clone());
        Ok(LeasedGpuProvider {
            ctx,
            arena: std::cell::RefCell::new(arena),
            probe: std::cell::RefCell::new(probe),
            parent,
        })
    }

    pub fn context(&self) -> &Arc<CudaContext> {
        &self.ctx
    }

    /// The declared boundary for a space, in MiB, rounded UP. Public because a caller may want
    /// to know what it is about to ask for before asking.
    pub fn mib_for(space: &FciSpace) -> u64 {
        vram_bytes_for(space).map(|b| b.div_ceil(MIB)).unwrap_or(u64::MAX)
    }

    /// **Step one: probe and lease.** No device operator yet.
    ///
    /// Separated from the build on purpose, and the separation is D3 rather than tidiness: *the
    /// probe at lease time buys validity NOW, and nothing after.* A real holder leases at the
    /// start of a solve and uses the memory throughout it, and everything that can go wrong in
    /// that gap is what a lease is a receipt about. Collapsing the two into one call would make
    /// the gap unreachable — including to the plant that has to demonstrate it.
    ///
    /// The order inside is D1's: the probe is the authority and runs before anything is
    /// recorded, so a refusal costs nothing and leaves no ledger entry.
    pub fn take_lease(&self, space: &FciSpace) -> Result<VramLease, LeaseError> {
        let mib = Self::mib_for(space);
        let mut arena = self.arena.borrow_mut();
        let mut probe = self.probe.borrow_mut();
        let id = arena.lease(&mut *probe, self.parent, ResourceKind::Vram, mib)?;
        Ok(VramLease { id, mib })
    }

    /// **Step two: use it.** The build is the USE, and the use is itself a probe (D3(2)).
    ///
    /// If it fails, the lease is CONVICTED — we held a valid lease and the resource was not
    /// there — and the conviction surfaces in the books rather than being returned as an
    /// ordinary error and forgotten.
    pub fn build_on(
        &self,
        lease: VramLease,
        space: &FciSpace,
        ci: &CiInts,
    ) -> Result<LeasedGpuSigma, LeasedGpuError> {
        let mut arena = self.arena.borrow_mut();
        match GpuFciSigma::new(&self.ctx, space, ci) {
            Ok(op) => {
                // The receipt is the rent (§9 Q1): a receipt of REAL WORK, and the work this
                // lease exists for is the determinants it can now apply the Hamiltonian to.
                let _ = arena.pay_rent(lease.id, holon_resource::Receipt(space.n_det as u64));
                Ok(LeasedGpuSigma {
                    lease: lease.id,
                    op,
                    mib: lease.mib,
                })
            }
            Err(why) => {
                let _ = arena.convict(
                    lease.id,
                    "the VRAM probe passed and the operator's allocation then failed",
                );
                Err(LeasedGpuError::ConvictedOnBuild {
                    lease: lease.id,
                    why,
                })
            }
        }
    }

    /// Both steps, for the ordinary caller who has nothing to do in between.
    pub fn lease_and_build(
        &self,
        space: &FciSpace,
        ci: &CiInts,
    ) -> Result<LeasedGpuSigma, LeasedGpuError> {
        let lease = self.take_lease(space).map_err(LeasedGpuError::Lease)?;
        self.build_on(lease, space, ci)
    }

    /// Release a lease and its operator together, leaf-to-root (D9).
    pub fn release(&self, held: LeasedGpuSigma) -> Result<(), LeaseError> {
        let LeasedGpuSigma { lease, op, .. } = held;
        drop(op);
        self.arena.borrow_mut().release(lease).map(|_| ())
    }

    /// Convict a lease whose resource vanished under it, with the evidence.
    ///
    /// The caller reaches for this when a USE fails mid-solve. It is separate from `release`
    /// because the two move different numbers in the ledger, and an audit that could not tell a
    /// clean release from a resource that disappeared would have lost the only thing it is for.
    pub fn convict(
        &self,
        held: LeasedGpuSigma,
        evidence: &'static str,
    ) -> Result<(), LeaseError> {
        let LeasedGpuSigma { lease, op, .. } = held;
        drop(op);
        self.arena.borrow_mut().convict(lease, evidence)
    }

    /// Whether the books balance: `opened == released + convicted + live`, exact over integers.
    pub fn balances(&self) -> bool {
        self.arena.borrow().balances()
    }

    pub fn ledger(&self) -> holon_resource::Ledger {
        self.arena.borrow().ledger()
    }

    /// What state a lease this provider took is in.
    ///
    /// The arena is borrowed for the provider's lifetime, so a caller cannot read it directly;
    /// this is the window. It exists because an assertion about a ledger COUNT and an assertion
    /// about the LEASE's own state are different claims — a count could move for another lease
    /// — and a plant that could only check the count would be checking the weaker one.
    pub fn lease_state(&self, id: LeaseId) -> Option<holon_resource::LeaseState> {
        self.arena.borrow().get(id).map(|l| l.state)
    }
}

impl SigmaProvider for LeasedGpuProvider<'_> {
    fn device(&self) -> DeviceClass {
        DeviceClass::Gpu
    }

    fn op_for<'b>(
        &self,
        space: &'b FciSpace,
        ci: &'b CiInts,
    ) -> Result<Box<dyn SigmaOp<f64> + 'b>, String> {
        self.lease_and_build(space, ci)
            .map(|held| Box::new(held) as Box<dyn SigmaOp<f64> + 'b>)
            // D4: the refusal is LOUD and names what was asked and what was found. It never
            // falls back to the host — a `Solution` stamped `Gpu` that a CPU computed would be
            // worse than either device.
            .map_err(|e| {
                format!(
                    "REFUSED the GPU sigma for a {}-determinant space needing {} MiB: {e}. \
                     No fallback: a bit-gated artifact's class is not a runtime choice.",
                    space.n_det,
                    Self::mib_for(space)
                )
            })
    }
}

/// **A competitor that takes VRAM out from under a live lease** — the yank, as a real
/// allocation on the real card rather than a scripted answer.
///
/// This is the D3b acceptance plant's instrument, and it is deliberately BOUNDED: it takes what
/// is free MINUS a declared reserve, holds it, and gives it back. A plant that cornered the card
/// would be testing the lease layer by breaking the machine, and the browser's GPU process is on
/// this device too.
pub struct VramCompetitor {
    _slab: cudarc::driver::CudaSlice<u8>,
    pub took_mib: u64,
}

impl VramCompetitor {
    /// Take everything free except `reserve_mib`.
    ///
    /// Returns `None` if the card does not have enough free memory for the plant to mean
    /// anything — which is a VOID, not a pass: on a busy card there is no yank to demonstrate
    /// and a test that reported success would be scoring an empty sector.
    pub fn take_all_but(
        ctx: &Arc<CudaContext>,
        reserve_mib: u64,
    ) -> Result<Option<VramCompetitor>, cudarc::driver::DriverError> {
        ctx.bind_to_thread()?;
        let (free, _total) = cudarc::driver::result::mem_get_info()?;
        let free_mib = free as u64 / MIB;
        if free_mib <= reserve_mib {
            return Ok(None);
        }
        let take = free_mib - reserve_mib;
        let stream = ctx.new_stream()?;
        // SAFETY: uninitialised device memory, never read, dropped with the competitor.
        let slab = unsafe { stream.alloc::<u8>((take * MIB) as usize) }?;
        stream.synchronize()?;
        Ok(Some(VramCompetitor {
            _slab: slab,
            took_mib: take,
        }))
    }
}
