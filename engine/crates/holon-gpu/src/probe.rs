//! **The VRAM probe `holon-resource` deliberately does not have.**
//!
//! `holon-resource` is zero-CUDA by design and says so in its own `AttemptProbe`: *"no VRAM
//! probe in this crate; the GPU owner must supply one"*. This is the GPU owner keeping that
//! bargain, and it is written to RESOURCE_DESIGN **D2**:
//!
//! > A probe must ATTEMPT the thing, or measure the headroom for the thing, and never infer
//! > availability from the holder's liveness.
//!
//! # Why it allocates instead of reading `mem_get_info`
//!
//! The founding case is this machine on 2026-08-30: the 4090 reported idle with 16,376 MiB
//! free while the root filesystem sat at 100%. A reported number is a report. `cuMemGetInfo`
//! can say there are 8 GB free and `cuMemAlloc` can still fail — fragmentation, another
//! process taking it in the microseconds between, an ECC retirement, a context that is not what
//! the caller thinks it is. So this probe takes the memory and gives it back. That is cheap
//! precisely because it is about to be done for real anyway.
//!
//! It does the headroom check too, and refuses on EITHER: a probe that allocates a byte and
//! declares 8 GB available has attempted the wrong thing.
//!
//! # Non-VRAM kinds are REFUSED, not passed
//!
//! This probe knows about one resource. Asked about disk or workers it says so and fails,
//! rather than passing on a question it did not ask — a probe that returns `Pass` for
//! everything it does not understand is worse than no probe, because a lease then carries a
//! receipt for a check nobody made.

use std::sync::Arc;

use cudarc::driver::CudaContext;
use holon_resource::probe::{Probe, ProbeVerdict, ResourceKind};

/// A probe that really allocates on a real device.
///
/// `amount` is in **mebibytes**, because the lease ledger takes integers (D8) and bytes at this
/// scale do not fit a comfortable integer story — a lease for "512" reads as 512 MiB in the
/// receipt and in the log.
pub struct VramProbe {
    ctx: Arc<CudaContext>,
    /// The last reading, so a refusal can say what it found rather than only that it refused.
    pub last_free_mib: u64,
    pub last_total_mib: u64,
    /// Probes attempted, and how many really allocated. Counted because a probe that stopped
    /// allocating and started reporting would otherwise look identical from outside.
    pub attempts: u64,
    pub allocations: u64,
}

impl VramProbe {
    /// Open device `ordinal`. Fails LOUDLY (D5): a driver present and a CUDA broken must refuse,
    /// never quietly hand back a probe that will pass on the host.
    pub fn new(ordinal: usize) -> Result<VramProbe, cudarc::driver::DriverError> {
        let ctx = CudaContext::new(ordinal)?;
        Ok(VramProbe {
            ctx,
            last_free_mib: 0,
            last_total_mib: 0,
            attempts: 0,
            allocations: 0,
        })
    }

    /// Share an already-open context — the normal case, since the operator that is about to use
    /// the device has one.
    pub fn on(ctx: Arc<CudaContext>) -> VramProbe {
        VramProbe {
            ctx,
            last_free_mib: 0,
            last_total_mib: 0,
            attempts: 0,
            allocations: 0,
        }
    }

    pub fn context(&self) -> &Arc<CudaContext> {
        &self.ctx
    }

    /// Free and total device memory in MiB, or `None` if the device cannot be asked — which is
    /// itself a reading, and the one a caller most needs not to have defaulted away.
    pub fn mem_info_mib(&mut self) -> Option<(u64, u64)> {
        self.ctx.bind_to_thread().ok()?;
        let (free, total) = cudarc::driver::result::mem_get_info().ok()?;
        self.last_free_mib = free as u64 / (1 << 20);
        self.last_total_mib = total as u64 / (1 << 20);
        Some((self.last_free_mib, self.last_total_mib))
    }

    /// The attempt itself: take `mib` mebibytes on the device, then give them back.
    ///
    /// The allocation is NOT written to. Touching it would cost the bandwidth of the thing being
    /// probed for and prove nothing extra — CUDA's allocator commits at `cuMemAlloc`, unlike a
    /// host allocator that can hand back address space it has not backed. That difference is why
    /// the RAM probe in `holon-resource` touches its sample and this one does not, and it is
    /// stated here so the asymmetry reads as a decision rather than an omission.
    fn attempt(&mut self, mib: u64) -> ProbeVerdict {
        self.attempts += 1;
        if self.ctx.bind_to_thread().is_err() {
            return ProbeVerdict::Fail("the CUDA context could not be bound to this thread");
        }
        let stream = match self.ctx.new_stream() {
            Ok(s) => s,
            Err(_) => return ProbeVerdict::Fail("the device refused a stream"),
        };
        let bytes = (mib as usize).saturating_mul(1 << 20).max(1);
        // SAFETY: uninitialised device memory that is dropped without being read.
        let slab = unsafe { stream.alloc::<u8>(bytes) };
        match slab {
            Ok(s) => {
                if stream.synchronize().is_err() {
                    return ProbeVerdict::Fail("the device did not synchronise after allocating");
                }
                drop(s);
                self.allocations += 1;
                ProbeVerdict::Pass("allocated the requested VRAM on the device and freed it")
            }
            Err(_) => ProbeVerdict::Fail(
                "the device refused the allocation. The write that fails is the authoritative \
                 reading: whatever free memory was reported, this much could not be taken.",
            ),
        }
    }
}

impl Probe for VramProbe {
    fn probe(&mut self, kind: ResourceKind, amount: u64) -> ProbeVerdict {
        if kind != ResourceKind::Vram {
            // Refusing is honest. Passing would put a receipt for an unmade check into a lease.
            return ProbeVerdict::Fail(
                "this probe knows only VRAM; it refuses rather than passing on a resource it \
                 did not test",
            );
        }
        // Headroom first, because it is free and because it produces a better refusal message
        // than an allocation failure does. It is NOT the verdict — D2 says a probe attempts the
        // thing, and a reported number is a report.
        match self.mem_info_mib() {
            Some((free, _)) if free < amount => ProbeVerdict::Fail(
                "the device reports less free VRAM than the lease asks for",
            ),
            None => ProbeVerdict::Fail("the device could not be asked how much VRAM is free"),
            Some(_) => self.attempt(amount),
        }
    }
}

/// **THE WRONG VRAM PROBE, kept on purpose** — the D2 foil, in the device's own vocabulary.
///
/// It asks `cuMemGetInfo` and answers from the number. On a healthy card it is right every
/// time, which is exactly what makes it dangerous: it is a report about the past, and the moment
/// it is wrong is the moment an allocation was going to fail anyway. `holon-resource` keeps
/// `LivenessProbe` for the same reason — *a wrong implementation that nobody can point at gets
/// rebuilt*.
///
/// Nothing dispatches through this. It exists so the plant can show the two probes disagreeing.
pub struct ReportedFreeProbe {
    pub ctx: Arc<CudaContext>,
}

impl Probe for ReportedFreeProbe {
    fn probe(&mut self, kind: ResourceKind, amount: u64) -> ProbeVerdict {
        if kind != ResourceKind::Vram {
            return ProbeVerdict::Fail("not VRAM");
        }
        if self.ctx.bind_to_thread().is_err() {
            return ProbeVerdict::Fail("context not bindable");
        }
        match cudarc::driver::result::mem_get_info() {
            Ok((free, _)) if (free as u64 / (1 << 20)) >= amount => {
                ProbeVerdict::Pass("the device REPORTED enough free VRAM")
            }
            Ok(_) => ProbeVerdict::Fail("the device reported less free VRAM than asked"),
            Err(_) => ProbeVerdict::Fail("mem_get_info failed"),
        }
    }
}
