//! Probes. **A probe attempts the thing, or measures the headroom for the thing** — it never
//! infers availability from the holder's liveness (RESOURCE_DESIGN D2).
//!
//! # The founding case, and why this is a rule rather than a preference
//!
//! On 2026-08-30 this machine's 4090 reported idle with 16,376 MiB free while the root
//! filesystem sat at 100% with 60 MB left. Init-time discovery would have recorded a healthy
//! machine. Every disk write on it was failing.
//!
//! The subtle half is what a probe asks:
//!
//! * *"is the holder's process alive and scheduling?"* — passed for every writer on the machine
//!   that afternoon. They were all healthy. They were all failing.
//! * *"can I actually write a byte and remove it?"* — failed correctly.
//!
//! [`LivenessProbe`] exists in this file precisely because it is the WRONG one: it is the foil
//! the D2 plant needs, and a wrong implementation that nobody can point at gets rebuilt.

use std::fmt;

/// What is being leased. Arithmetic precision is a resource like any other — the `f64` tier's
/// expansion floor is a lease boundary (RESOURCE_DESIGN §2.3), and a request past it is an
/// overflow that leases the next rung rather than a retry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    Ram,
    Vram,
    Disk,
    Worker,
    /// A tier of arithmetic. `amount` is the negative base-10 exponent of the residual being
    /// asked for, so 10 means "I need 1e-10" — integers, because the ledger takes no floats.
    Arithmetic,
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// A probe's answer. `Pass` carries WHAT WAS CHECKED, because a lease is a receipt for rent paid
/// and the record of what was verified is the part guaranteed forever (D3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeVerdict {
    Pass(&'static str),
    Fail(&'static str),
}

impl ProbeVerdict {
    pub fn passed(&self) -> bool {
        matches!(self, ProbeVerdict::Pass(_))
    }
}

/// The probe interface. Injectable so that every refusal rule can be planted — a rule that has
/// never fired has never been demonstrated to gate (D13).
pub trait Probe {
    fn probe(&mut self, kind: ResourceKind, amount: u64) -> ProbeVerdict;
}

/// What a caller does when a probe fails: refuse LOUDLY, or degrade in a way that is STATED.
///
/// D4 — there is no third option. A silent fallback to a slower path is the vacuous-success
/// shape: the run completes, the number looks fine, and nothing records that the fast path was
/// never taken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Response {
    Refuse {
        kind: ResourceKind,
        asked: u64,
        why: &'static str,
    },
    /// A degrade that NAMES what was asked, what was found, and what path is being taken
    /// instead. The fields are mandatory because a `Degrade` with nothing in it is a silent
    /// fallback with a label.
    Degrade {
        kind: ResourceKind,
        asked: u64,
        found: &'static str,
        taking: &'static str,
    },
}

impl Response {
    /// The line that goes in the log. A Degrade that cannot say what it is doing instead is not
    /// a Degrade, and this is where that shows up.
    pub fn message(&self) -> String {
        match self {
            Response::Refuse { kind, asked, why } => {
                format!("REFUSED {kind} x{asked}: {why}")
            }
            Response::Degrade {
                kind,
                asked,
                found,
                taking,
            } => format!("DEGRADE {kind} x{asked}: found {found}; taking {taking}"),
        }
    }
}

/// A probe that ATTEMPTS the operation — the correct shape.
///
/// Disk: write a byte into the target directory and unlink it. RAM: allocate and touch. These
/// are cheap because they are about to be done for real anyway.
pub struct AttemptProbe {
    pub disk_dir: std::path::PathBuf,
}

impl AttemptProbe {
    pub fn new<P: Into<std::path::PathBuf>>(disk_dir: P) -> Self {
        AttemptProbe {
            disk_dir: disk_dir.into(),
        }
    }
}

impl Probe for AttemptProbe {
    fn probe(&mut self, kind: ResourceKind, amount: u64) -> ProbeVerdict {
        match kind {
            ResourceKind::Disk => {
                // Attempt the thing. The write that fails is the authoritative reading (D3(2));
                // nothing about free-space arithmetic is as trustworthy as trying it.
                let path = self.disk_dir.join(".holon-resource-probe");
                match std::fs::write(&path, b"1") {
                    Ok(()) => {
                        let _ = std::fs::remove_file(&path);
                        ProbeVerdict::Pass("wrote and removed a byte in the target directory")
                    }
                    Err(_) => ProbeVerdict::Fail("could not write a byte to the target directory"),
                }
            }
            ResourceKind::Ram => {
                // MEASURE THE HEADROOM, THEN ATTEMPT THE ADDRESS SPACE — and never commit
                // the pages. The first version of this arm reserved and page-touched the
                // FULL amount, which is not a probe but the allocation itself: on a shared
                // box it committed real memory and the OOM killer answered (2026-09-02,
                // periodic_availability, exit 137). D2 allows either attempting the thing
                // or measuring the headroom for it; for RAM the honest probe is both halves
                // without the third: read what the kernel says is available, refuse if the
                // ask exceeds it, then reserve the address space and touch a bounded sample
                // so an allocator that lies about reservations is still caught.
                if let Some(avail) = available_ram_bytes() {
                    if amount > avail {
                        return ProbeVerdict::Fail("the ask exceeds the kernel's available RAM");
                    }
                }
                // `as usize` would silently truncate on a 32-bit host — wasm32 — and admit
                // an 8 GB ask as a small one; the browser found that one (2026-09-02).
                let Ok(n) = usize::try_from(amount) else {
                    return ProbeVerdict::Fail("the ask exceeds this host's address space");
                };
                let mut v: Vec<u8> = Vec::new();
                if v.try_reserve_exact(n).is_err() {
                    return ProbeVerdict::Fail("the allocator refused the requested reservation");
                }
                let sample = n.min(1 << 20);
                v.resize(sample, 0);
                let mut i = 0usize;
                while i < sample {
                    v[i] = 1;
                    i += 4096;
                }
                if v.capacity() >= n {
                    ProbeVerdict::Pass("headroom measured and the address space reserved")
                } else {
                    ProbeVerdict::Fail("the reservation did not materialise")
                }
            }
            // Vram, Worker and Arithmetic have no in-crate attempt: this layer has no CUDA and
            // no thread pool, deliberately (zero dependencies). A caller that owns those probes
            // supplies its own; refusing here is honest, and passing would be a probe that
            // checked nothing.
            other => ProbeVerdict::Fail(match other {
                ResourceKind::Vram => "no VRAM probe in this crate; the GPU owner must supply one",
                ResourceKind::Worker => "no worker probe in this crate; the pool owner supplies one",
                _ => "no arithmetic-tier probe in this crate; the solver owns its own floors",
            }),
        }
    }
}

/// The kernel's own reading of available RAM (`MemAvailable` in `/proc/meminfo`), or
/// `None` where there is no such file — wasm, or a platform without procfs — in which
/// case the address-space attempt below is the whole probe and says so by passing on
/// less evidence.
pub fn available_ram_bytes() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("MemAvailable:") {
            let kb: u64 = rest.trim().trim_end_matches("kB").trim().parse().ok()?;
            return Some(kb * 1024);
        }
    }
    None
}

/// **THE WRONG PROBE, kept on purpose.** Asks whether the HOLDER is alive rather than whether
/// the RESOURCE is available.
///
/// This is the D2 plant's foil. On a machine whose filesystem is full it passes for every writer
/// — all healthy, all failing — which is exactly when it is most confidently wrong. It is `pub`
/// so a test can point at it; nothing in this crate dispatches through it.
pub struct LivenessProbe {
    pub holder_is_scheduling: bool,
}

impl Probe for LivenessProbe {
    fn probe(&mut self, _kind: ResourceKind, _amount: u64) -> ProbeVerdict {
        if self.holder_is_scheduling {
            ProbeVerdict::Pass("the holder's process is alive and scheduling")
        } else {
            ProbeVerdict::Fail("the holder's process is not scheduling")
        }
    }
}

/// A probe whose answers are supplied by the test. The plants' instrument.
pub struct ScriptedProbe {
    pub answers: Vec<ProbeVerdict>,
    pub calls: usize,
    /// Used once `answers` is exhausted.
    pub default: ProbeVerdict,
}

impl ScriptedProbe {
    pub fn always_pass() -> Self {
        ScriptedProbe {
            answers: Vec::new(),
            calls: 0,
            default: ProbeVerdict::Pass("scripted"),
        }
    }
    pub fn always_fail(why: &'static str) -> Self {
        ScriptedProbe {
            answers: Vec::new(),
            calls: 0,
            default: ProbeVerdict::Fail(why),
        }
    }
}

impl Probe for ScriptedProbe {
    fn probe(&mut self, _kind: ResourceKind, _amount: u64) -> ProbeVerdict {
        let v = if self.calls < self.answers.len() {
            self.answers[self.calls]
        } else {
            self.default
        };
        self.calls += 1;
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **PLANT D2 — a holder alive and scheduling while its resource is exhausted.**
    ///
    /// The liveness probe PASSES here, which is the defect. The attempting probe is the one that
    /// gets it right. Carrier asserted first: the liveness probe must be capable of failing at
    /// all, or "it always passes" would satisfy this test trivially.
    #[test]
    fn plant_d2_liveness_passes_exactly_where_it_is_most_wrong() {
        // the carrier: the wrong probe is not a constant
        let mut dead = LivenessProbe {
            holder_is_scheduling: false,
        };
        assert!(!dead.probe(ResourceKind::Disk, 1).passed());

        // the plant: holder healthy, resource gone
        let mut alive = LivenessProbe {
            holder_is_scheduling: true,
        };
        assert!(
            alive.probe(ResourceKind::Disk, 1).passed(),
            "the liveness probe must pass here — that is the defect being demonstrated"
        );

        // and the correct probe, pointed at a directory that cannot be written
        let mut attempt = AttemptProbe::new("/proc/nonexistent-holon-resource-probe-dir");
        assert!(
            !attempt.probe(ResourceKind::Disk, 1).passed(),
            "the attempting probe passed on an unwritable directory; it is not attempting"
        );
    }

    /// The attempting probe works where the resource IS available — the other half, without
    /// which "it always fails" would pass the test above.
    #[test]
    fn the_attempting_probe_passes_on_a_writable_directory() {
        let mut p = AttemptProbe::new(std::env::temp_dir());
        assert!(p.probe(ResourceKind::Disk, 1).passed());
        assert!(p.probe(ResourceKind::Ram, 4096).passed());
    }

    /// D4: a Degrade must name what it is doing instead. A struct that cannot be built without
    /// those fields is the enforcement; this checks the message actually carries them.
    #[test]
    fn a_degrade_states_what_it_is_taking_instead() {
        let d = Response::Degrade {
            kind: ResourceKind::Vram,
            asked: 512,
            found: "no CUDA device",
            taking: "the CPU mesh path",
        };
        let m = d.message();
        assert!(m.contains("no CUDA device") && m.contains("CPU mesh path"), "{m}");
    }
}
