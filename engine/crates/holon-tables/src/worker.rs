//! The mesh's own worker probe — the one `holon-resource` deliberately refuses to supply.
//!
//! `holon-resource::AttemptProbe` returns `Fail` for [`ResourceKind::Worker`] with "no worker
//! probe in this crate; the pool owner supplies one". This crate is the pool owner, so this is
//! that probe.
//!
//! # It attempts the thing (D2)
//!
//! The tempting implementation asks `std::thread::available_parallelism()` and compares. That is
//! a *liveness* probe wearing a resource probe's clothes: it reports what the machine is
//! configured to offer, not what it will actually give us now. The disk-full window is the
//! standing warning — every writer on that box was scheduling, healthy by any such check, and
//! unable to write a byte.
//!
//! So this probe **spawns a thread, makes it do a scrap of work, and joins it.** If the OS will
//! not give us a thread, or the thread cannot run, the probe fails — which is the fact the lease
//! exists to guarantee. It costs a fork and a join, against a shard that is about to run for
//! minutes.

use holon_resource::{Probe, ProbeVerdict, ResourceKind};

/// Probes a worker by actually starting one.
pub struct WorkerProbe {
    /// Available parallelism at construction, reported for the log. NOT the admission test —
    /// see the module header for why a capacity reading is not a probe.
    pub reported_parallelism: usize,
}

impl Default for WorkerProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerProbe {
    pub fn new() -> WorkerProbe {
        WorkerProbe {
            reported_parallelism: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(0),
        }
    }
}

impl Probe for WorkerProbe {
    fn probe(&mut self, kind: ResourceKind, _amount: u64) -> ProbeVerdict {
        match kind {
            ResourceKind::Worker => {
                // Attempt it: spawn, compute something the optimiser cannot discard, join.
                match std::thread::Builder::new()
                    .name("holon-tables-probe".into())
                    .spawn(|| std::hint::black_box(1u64 + 1))
                {
                    Err(_) => ProbeVerdict::Fail("the OS refused a thread"),
                    Ok(h) => match h.join() {
                        Ok(2) => ProbeVerdict::Pass("spawned a worker, ran it, and joined it"),
                        Ok(_) => ProbeVerdict::Fail("the probe thread returned the wrong value"),
                        Err(_) => ProbeVerdict::Fail("the probe thread panicked"),
                    },
                }
            }
            // Everything else is somebody else's to probe, and refusing is honest: a probe that
            // checks nothing is worse than an absent one.
            _ => ProbeVerdict::Fail("the table mesh probes workers only"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The probe passes by actually starting a worker, and refuses kinds it does not own.
    #[test]
    fn the_worker_probe_starts_a_worker_and_refuses_what_it_does_not_own() {
        let mut p = WorkerProbe::new();
        assert!(p.probe(ResourceKind::Worker, 1).passed());
        // M-VACUOUS-SUCCESS: it must not pass everything.
        for k in [ResourceKind::Vram, ResourceKind::Disk, ResourceKind::Ram] {
            assert!(
                !p.probe(k, 1).passed(),
                "the worker probe claimed to have checked {k:?}, which it cannot"
            );
        }
    }

    /// The capacity reading is REPORTED and is not the admission test — stated as a test so the
    /// two cannot quietly merge.
    #[test]
    fn parallelism_is_reported_and_is_not_the_gate() {
        let mut p = WorkerProbe::new();
        let before = p.reported_parallelism;
        assert!(p.probe(ResourceKind::Worker, u64::MAX).passed(),
            "a request for u64::MAX workers was refused by a CAPACITY comparison; this probe \
             admits on whether a worker STARTS, and the amount is not its business");
        assert_eq!(p.reported_parallelism, before);
    }
}
