//! The reaper: the backstop for the explicit release path, and the component most able to do
//! harm.
//!
//! # The one thing it must not do
//!
//! Reaping something still needed turns the resource layer into a saboteur — strictly worse than
//! the leak it was preventing. So a missed heartbeat is a signal to LOOK, never a verdict
//! (RESOURCE_DESIGN D10), and conviction requires evidence in the same way a corrupted shard
//! does.
//!
//! # Why there are three rungs and not one
//!
//! The founding case is this machine on 2026-08-30. With the root filesystem at 100%, **every
//! writer was blocked and none was progressing.** A reaper trusting a timeout would have
//! reclaimed the entire machine's work — correctly observing that nothing was advancing, and
//! completely misdiagnosing why.
//!
//! Rung 3 is the fix and it is cheap: **the reaper probes ITSELF.** Before convicting a holder
//! for failing operation class *X*, it attempts *X*. If its own attempt fails, the problem is the
//! machine and it stands down. That is D2 applied to the reaper, one syscall, no global scan —
//! "is anything else progressing?" reduces to "can I, right now, do the thing I am about to
//! convict someone for failing to do?"

use crate::lease::{Arena, LeaseId, LeaseState};
use crate::probe::{Probe, ProbeVerdict, ResourceKind};

/// Everything the reaper looked at. Logged whatever the verdict, because a reaping without its
/// evidence is exactly the timeout-verdict this design forbids.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReapEvidence {
    /// Rung 1 — has the holder been silent for longer than a multiple of its OWN declared step
    /// time? Not a global constant: a holder whose step is 50 s is not late at 10 s.
    pub grace_expired: bool,
    /// Rung 2 — is the holder's process alive and scheduling?
    pub holder_scheduling: bool,
    /// Rung 3 — the reaper's own attempt at the same operation class.
    pub reaper_own_probe: ProbeVerdict,
}

/// What the reaper decided, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReapVerdict {
    /// All three rungs cleared: the holder is genuinely idle on a healthy machine.
    Reap { evidence: ReapEvidence },
    /// Rung 3 failed — the machine is the problem, not the holder. Reclaim NOTHING.
    StandDown {
        why: &'static str,
        evidence: ReapEvidence,
    },
    /// Rungs 1 or 2 say the holder is fine.
    Keep {
        why: &'static str,
        evidence: ReapEvidence,
    },
}

impl ReapVerdict {
    pub fn evidence(&self) -> ReapEvidence {
        match self {
            ReapVerdict::Reap { evidence }
            | ReapVerdict::StandDown { evidence, .. }
            | ReapVerdict::Keep { evidence, .. } => *evidence,
        }
    }

    pub fn reaped(&self) -> bool {
        matches!(self, ReapVerdict::Reap { .. })
    }

    /// The log line. Every reaping is logged with all three answers.
    pub fn message(&self, id: LeaseId) -> String {
        let e = self.evidence();
        let head = match self {
            ReapVerdict::Reap { .. } => format!("REAPED lease {id}"),
            ReapVerdict::StandDown { why, .. } => {
                format!("STOOD DOWN on lease {id} — {why}")
            }
            ReapVerdict::Keep { why, .. } => format!("KEPT lease {id} — {why}"),
        };
        format!(
            "{head} [grace_expired={} holder_scheduling={} reaper_own_probe={:?}]",
            e.grace_expired, e.holder_scheduling, e.reaper_own_probe
        )
    }
}

/// How the reaper learns about the world. Every input is injected, because a reaper that can
/// only be exercised on a healthy machine has never been exercised at all — and the failure it
/// guards against only appears on an unhealthy one.
pub trait ReaperWorld {
    /// Rung 1: has this holder been silent past a multiple of its own declared step time?
    fn grace_expired(&mut self, id: LeaseId) -> bool;
    /// Rung 2: is the holder's process alive and scheduling?
    fn holder_scheduling(&mut self, id: LeaseId) -> bool;
}

/// The reaper. Owns rung 3 as a real [`Probe`] so that "the reaper probes itself" is the
/// mechanism rather than a comment.
pub struct Reaper<W: ReaperWorld, P: Probe> {
    pub world: W,
    pub own_probe: P,
}

impl<W: ReaperWorld, P: Probe> Reaper<W, P> {
    pub fn new(world: W, own_probe: P) -> Self {
        Reaper { world, own_probe }
    }

    /// Judge one lease. Rungs in order, and rung 3 is consulted whenever rungs 1 and 2 would
    /// otherwise convict — which is exactly when getting it wrong is expensive.
    pub fn judge(&mut self, id: LeaseId, kind: ResourceKind) -> ReapVerdict {
        let grace_expired = self.world.grace_expired(id);
        let holder_scheduling = self.world.holder_scheduling(id);

        if !grace_expired {
            return ReapVerdict::Keep {
                why: "within its grace period",
                evidence: ReapEvidence {
                    grace_expired,
                    holder_scheduling,
                    reaper_own_probe: ProbeVerdict::Pass("not consulted"),
                },
            };
        }
        if holder_scheduling {
            return ReapVerdict::Keep {
                why: "past grace, but the holder is alive and scheduling — slow, not idle",
                evidence: ReapEvidence {
                    grace_expired,
                    holder_scheduling,
                    reaper_own_probe: ProbeVerdict::Pass("not consulted"),
                },
            };
        }

        // RUNG 3. Both earlier rungs point at conviction, which is precisely when the reaper
        // must check whether it is the machine rather than the holder.
        let own = self.own_probe.probe(kind, 1);
        let evidence = ReapEvidence {
            grace_expired,
            holder_scheduling,
            reaper_own_probe: own,
        };
        if !own.passed() {
            return ReapVerdict::StandDown {
                why: "the reaper's OWN attempt at this operation class failed, so the machine is \
                      the problem and not the holder; reclaiming here would destroy work that is \
                      merely blocked",
                evidence,
            };
        }
        ReapVerdict::Reap { evidence }
    }

    /// Judge and, if the verdict is `Reap`, convict the lease with its evidence. Returns the
    /// verdict either way so the caller logs it whatever happened.
    pub fn sweep_one(
        &mut self,
        arena: &mut Arena,
        id: LeaseId,
        kind: ResourceKind,
    ) -> ReapVerdict {
        let verdict = self.judge(id, kind);
        if verdict.reaped() {
            // A reaped lease is CONVICTED, not quietly released: it ended without its owner
            // saying so, and the audit must be able to tell those apart.
            let _ = arena.convict(id, "reaped: idle past grace on a healthy machine");
            arena.note_reaped();
        }
        verdict
    }

    /// Sweep every outstanding lease of a kind.
    pub fn sweep(&mut self, arena: &mut Arena, kind: ResourceKind) -> Vec<(LeaseId, ReapVerdict)> {
        let candidates: Vec<LeaseId> = (0..)
            .map_while(|i| arena.get(i).map(|l| (i, l.state, l.kind)))
            .filter(|(_, st, k)| {
                *k == kind
                    && matches!(st, LeaseState::Leased | LeaseState::Active | LeaseState::Idle)
            })
            .map(|(i, _, _)| i)
            .collect();
        candidates
            .into_iter()
            .map(|id| {
                let v = self.sweep_one(arena, id, kind);
                (id, v)
            })
            .collect()
    }
}

/// A world whose answers the test supplies.
pub struct ScriptedWorld {
    pub grace_expired: bool,
    pub holder_scheduling: bool,
}

impl ReaperWorld for ScriptedWorld {
    fn grace_expired(&mut self, _id: LeaseId) -> bool {
        self.grace_expired
    }
    fn holder_scheduling(&mut self, _id: LeaseId) -> bool {
        self.holder_scheduling
    }
}
