//! Dispatch: **consult, then probe, then lease** — three steps, each of which can refuse
//! (RESOURCE_DESIGN D11), plus the registry those decisions cite.
//!
//! # The rule that shapes this module
//!
//! **D0 — device class belongs to the ARTIFACT, not the schedule.** A faster arithmetic route
//! that changes BITS changes what was produced: SATURATION-3 G2 measured the GPU and CPU sigmas
//! agreeing to `3.033e-15` relative while **91.0% of 207,025 entries differ bitwise**. Both are
//! correct answers; they are not the same artifact.
//!
//! So "prefer whichever device wins at this size" — the naive dispatch policy — makes the
//! trailing bits of every result a function of where a measured crossover happens to sit, and
//! that crossover moves with load, driver version and library kernel selection. A table built on
//! a busy afternoon would differ from the same table built on a quiet one, and every bit-identity
//! gate in the engine would go red for a reason that is not a defect.
//!
//! Hence: a bit-gated workload DECLARES its device class and dispatch may choose only within it.
//! A bit-gated workload with no declared class is refused outright — there is no safe choice to
//! make. (M-DEVICE-CLASS.)
//!
//! # Floats live here and never leave
//!
//! Throughputs and spreads are `f64` because they are measurements. They are REPORTED and never
//! CERTIFIED: nothing in this module writes into a [`crate::ledger`] receipt, which takes
//! integers only (D8). A rate is not rent.

use crate::lease::{Arena, LeaseError, LeaseId};
use crate::probe::{Probe, ProbeVerdict, ResourceKind};

/// Which arithmetic the work would run on. Part of the ARTIFACT for bit-gated workloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeviceClass {
    Cpu,
    Gpu,
}

/// What determinism gate a registered kernel carries. Adoption for bit-gated work requires one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Determinism {
    /// Fixed reduction order; run-to-run bit-identical, verified over `runs_checked` runs.
    FixedOrder { runs_checked: u32 },
    /// Agreement with the reference bounded and DECLARED, but not bit-identical. Usable for work
    /// that is not bit-gated; never a substitute for a fixed order when it is.
    BoundedAgreement {
        relative: f64,
        bitwise_differing_fraction: f64,
    },
    /// No gate at all.
    None,
}

impl Determinism {
    /// Whether this gate is strong enough for work whose output is compared bit-for-bit.
    pub fn admits_bit_gated_work(&self) -> bool {
        matches!(self, Determinism::FixedOrder { .. })
    }
}

/// The registry key. **Size is IN the key** (M-CACHE-KIND): crossovers are per-size facts, and a
/// small-size entry admitting a large dispatch is existence standing in for certification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorkloadKey {
    pub workload: &'static str,
    pub size: u64,
    pub device: DeviceClass,
}

/// One registered measurement. Mean AND spread, per §9 Q4 — one constant cannot separate a 2×
/// thermal throttle from a 10× lie, so the tolerance is derived from the kernel's own observed
/// variability.
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    pub key: WorkloadKey,
    /// Throughput, work units per second.
    pub mean: f64,
    /// Observed spread of that throughput at registration.
    pub spread: f64,
    /// Declared conviction factor: convict outside `k * spread`.
    pub k: f64,
    pub instrument: &'static str,
    pub determinism: Determinism,
}

/// The verdict of re-timing a workload against what was registered (D12).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpotCheck {
    /// Within `k * spread` of the registered mean.
    Consistent { observed: f64, mean: f64 },
    /// Outside it. The ENTRY is convicted, not the run — the registry lied about the kernel.
    Convicted {
        observed: f64,
        mean: f64,
        tolerance: f64,
    },
}

impl SpotCheck {
    pub fn convicted(&self) -> bool {
        matches!(self, SpotCheck::Convicted { .. })
    }
}

/// Which of the three steps refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Consult,
    ProbeDevice,
    Lease,
}

/// The outcome of a dispatch decision. Logged whichever way it goes, with the entry it cited.
#[derive(Clone, Debug)]
pub enum Dispatch {
    Use {
        device: DeviceClass,
        lease: LeaseId,
        cited_mean: f64,
        cited_instrument: &'static str,
    },
    Refused {
        step: Step,
        why: String,
    },
}

impl Dispatch {
    pub fn message(&self) -> String {
        match self {
            Dispatch::Use {
                device,
                lease,
                cited_mean,
                cited_instrument,
            } => format!(
                "DISPATCH {device:?} (lease {lease}) citing {cited_mean} /s measured by \
                 {cited_instrument}"
            ),
            Dispatch::Refused { step, why } => format!("REFUSED at {step:?}: {why}"),
        }
    }

    pub fn used(&self) -> Option<DeviceClass> {
        match self {
            Dispatch::Use { device, .. } => Some(*device),
            _ => None,
        }
    }
}

/// A unit of work asking to be placed.
#[derive(Clone, Copy, Debug)]
pub struct Workload {
    pub name: &'static str,
    pub size: u64,
    /// Whether this workload's output is compared bit-for-bit anywhere downstream.
    pub bit_gated: bool,
    /// D0: the artifact's declared device class. Dispatch may not leave it.
    pub declared_class: Option<DeviceClass>,
}

/// The registry and the dispatcher.
#[derive(Default)]
pub struct Registry {
    entries: Vec<Entry>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            entries: Vec::new(),
        }
    }

    pub fn register(&mut self, e: Entry) {
        self.entries.push(e);
    }

    /// Look up an entry. **Exact key only.** A near-miss on size is not a hit — returning the
    /// closest entry is precisely the cache-kind defect, where existence stands in for
    /// certification.
    pub fn get(&self, key: &WorkloadKey) -> Option<&Entry> {
        self.entries.iter().find(|e| &e.key == key)
    }

    /// D12: re-time a workload and check it against its registration. Convicts the ENTRY.
    pub fn spot_check(&self, key: &WorkloadKey, observed: f64) -> Option<SpotCheck> {
        let e = self.get(key)?;
        let tolerance = e.k * e.spread;
        Some(if (observed - e.mean).abs() <= tolerance {
            SpotCheck::Consistent {
                observed,
                mean: e.mean,
            }
        } else {
            SpotCheck::Convicted {
                observed,
                mean: e.mean,
                tolerance,
            }
        })
    }

    /// **CONSULT, THEN PROBE, THEN LEASE.** Three steps, each separately refusable, each named
    /// in the refusal so a wrong choice is traceable to the step that made it.
    pub fn dispatch<P: Probe>(
        &self,
        work: &Workload,
        arena: &mut Arena,
        probe: &mut P,
    ) -> Dispatch {
        // ---- D0. Which classes may this workload even consider?
        let candidates: Vec<DeviceClass> = match (work.bit_gated, work.declared_class) {
            // Bit-gated with a declared class: that class ONLY.
            (true, Some(c)) => vec![c],
            // Bit-gated with no declared class: there is no safe choice to make.
            (true, None) => {
                return Dispatch::Refused {
                    step: Step::Consult,
                    why: format!(
                        "'{}' is bit-gated and declares no device class. Two devices can agree \
                         numerically and differ on 91% of entries bitwise, so dispatching it \
                         would make its output a function of the schedule. Declare the class on \
                         the artifact.",
                        work.name
                    ),
                }
            }
            // Not bit-gated: free choice, and the chosen device is logged with the result.
            (false, Some(c)) => vec![c],
            (false, None) => vec![DeviceClass::Gpu, DeviceClass::Cpu],
        };

        // ---- STEP 1: CONSULT. Exact key, size included.
        let mut best: Option<&Entry> = None;
        for c in &candidates {
            let key = WorkloadKey {
                workload: work.name,
                size: work.size,
                device: *c,
            };
            let Some(e) = self.get(&key) else { continue };
            if work.bit_gated && !e.determinism.admits_bit_gated_work() {
                return Dispatch::Refused {
                    step: Step::Consult,
                    why: format!(
                        "'{}' is bit-gated and {:?}'s entry carries {:?}, which is not a fixed \
                         reduction order. A bounded agreement is not a substitute when the output \
                         is compared bit-for-bit.",
                        work.name, c, e.determinism
                    ),
                };
            }
            if best.is_none_or(|b| e.mean > b.mean) {
                best = Some(e);
            }
        }
        let Some(entry) = best else {
            return Dispatch::Refused {
                step: Step::Consult,
                why: format!(
                    "no registered measurement for '{}' AT SIZE {} on any admissible device. \
                     Crossovers are per-size facts; an entry at another size is a different \
                     measurement and is not consulted.",
                    work.name, work.size
                ),
            };
        };

        // ---- STEP 2: PROBE THE DEVICE. A registration is a memory of a measurement;
        //      calibrations are rented, and the device may be gone since.
        let kind = match entry.key.device {
            DeviceClass::Gpu => ResourceKind::Vram,
            DeviceClass::Cpu => ResourceKind::Worker,
        };
        if let ProbeVerdict::Fail(why) = probe.probe(kind, 1) {
            return Dispatch::Refused {
                step: Step::ProbeDevice,
                why: format!(
                    "{:?} probe failed: {why}. REFUSED rather than falling back — a silent \
                     fallback would report the run as complete while nothing recorded that the \
                     registered path was never taken.",
                    entry.key.device
                ),
            };
        }

        // ---- STEP 3: LEASE. Still refusable.
        match arena.lease(probe, None, kind, 1) {
            Ok(lease) => Dispatch::Use {
                device: entry.key.device,
                lease,
                cited_mean: entry.mean,
                cited_instrument: entry.instrument,
            },
            Err(LeaseError::Refused { why, .. }) => Dispatch::Refused {
                step: Step::Lease,
                why: why.to_string(),
            },
            Err(e) => Dispatch::Refused {
                step: Step::Lease,
                why: e.message(),
            },
        }
    }
}
