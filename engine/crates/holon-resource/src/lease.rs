//! The lease tree: a child holon per allocation, probed at birth, ledgered while it lives,
//! released leaf-to-root when the need ends.

use crate::ledger::{Ledger, Receipt};
use crate::probe::{Probe, ProbeVerdict, ResourceKind};

/// The declared recursion cap (RESOURCE_DESIGN D7).
///
/// Four covers every structure the engine currently has — scene → shard → worker →
/// kernel-allocation. Without a cap, a workload that shards spawning child leases is a fork bomb
/// in this design's own vocabulary. Raising it is a deliberate edit with a case attached, which
/// is why it is a named constant and not a parameter.
pub const MAX_DEPTH: u32 = 4;

pub type LeaseId = u32;

/// Where a lease is in its life. `Refused` and `Convicted` are deliberately distinct: the first
/// is *we asked and the answer was no*, a normal and frequent outcome; the second is *we held a
/// valid lease and the resource went away underneath it*, which is a violation the audit sees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeaseState {
    Leased,
    Active,
    Idle,
    Released,
    Convicted,
}

/// Why a lease could not be granted, or why one ended badly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LeaseError {
    /// The probe said no. Normal, cheap, frequent — REFUSED, not convicted.
    Refused {
        kind: ResourceKind,
        amount: u64,
        why: &'static str,
    },
    /// D7: the recursion cap. Carries the CHAIN, because "too deep" without the path is a
    /// message that tells you a rule fired and nothing about what fired it.
    DepthExceeded { cap: u32, chain: Vec<LeaseId> },
    /// A caller referenced a lease that does not exist.
    NoSuchLease(LeaseId),
    /// A caller acted on a lease that had already ended.
    AlreadyEnded(LeaseId, LeaseState),
}

impl LeaseError {
    /// The message a human reads. The chain is spelled out for `DepthExceeded` — that is the
    /// whole point of carrying it.
    pub fn message(&self) -> String {
        match self {
            LeaseError::Refused { kind, amount, why } => {
                format!("refused: {kind:?} x{amount} — {why}")
            }
            LeaseError::DepthExceeded { cap, chain } => format!(
                "VOID: lease recursion exceeded its declared cap of {cap}. The chain was {}. A \
                 runaway lease tree is a fork bomb wearing this design's vocabulary; raise \
                 MAX_DEPTH deliberately, with a case, or fix the caller that is nesting.",
                chain
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            ),
            LeaseError::NoSuchLease(id) => format!("no such lease: {id}"),
            LeaseError::AlreadyEnded(id, st) => format!("lease {id} already ended as {st:?}"),
        }
    }
}

/// One allocation, as a child holon.
#[derive(Clone, Debug)]
pub struct Lease {
    pub id: LeaseId,
    pub parent: Option<LeaseId>,
    pub kind: ResourceKind,
    pub amount: u64,
    pub depth: u32,
    pub state: LeaseState,
    /// The rent this holder has paid: receipts of real work, not heartbeats (§9 Q1 — a
    /// heartbeat with no work product is not rent).
    pub rent: Receipt,
    pub children: Vec<LeaseId>,
    /// What the probe found when this lease was admitted. Kept because a lease is a RECEIPT FOR
    /// RENT PAID (D3) and the record of what was checked, against what, is the part that is
    /// guaranteed forever.
    pub admitted_on: &'static str,
}

/// The lease tree and its books.
pub struct Arena {
    leases: Vec<Option<Lease>>,
    ledger: Ledger,
    /// Convictions, with evidence, in the order they happened. A convicted child must SURFACE in
    /// its parent's accounting rather than vanishing (D9), and this is what the audit reads.
    convictions: Vec<(LeaseId, &'static str)>,
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Arena {
    pub fn new() -> Arena {
        Arena {
            leases: Vec::new(),
            ledger: Ledger::default(),
            convictions: Vec::new(),
        }
    }

    pub fn ledger(&self) -> Ledger {
        self.ledger
    }

    pub fn convictions(&self) -> &[(LeaseId, &'static str)] {
        &self.convictions
    }

    pub fn get(&self, id: LeaseId) -> Option<&Lease> {
        self.leases.get(id as usize).and_then(|s| s.as_ref())
    }

    /// The chain from a lease to the root, root-first. Used by the depth-cap message.
    pub fn chain(&self, from: Option<LeaseId>) -> Vec<LeaseId> {
        let mut out = Vec::new();
        let mut cur = from;
        while let Some(id) = cur {
            out.push(id);
            cur = self.get(id).and_then(|l| l.parent);
        }
        out.reverse();
        out
    }

    /// Leases still outstanding — the number the ledger identity is checked against.
    pub fn live_count(&self) -> u64 {
        self.leases
            .iter()
            .flatten()
            .filter(|l| {
                matches!(
                    l.state,
                    LeaseState::Leased | LeaseState::Active | LeaseState::Idle
                )
            })
            .count() as u64
    }

    /// **The audit.** `opened == released + convicted + live`, exact over integers.
    pub fn balances(&self) -> bool {
        self.ledger.balances(self.live_count())
    }

    /// Take a lease: PROBE first, then check the cap, then grant.
    ///
    /// The order is the design's (D1: discovery is a hint, the probe is the authority). The probe
    /// runs before anything is recorded, so a refusal costs nothing and leaves no entry.
    pub fn lease<P: Probe>(
        &mut self,
        probe: &mut P,
        parent: Option<LeaseId>,
        kind: ResourceKind,
        amount: u64,
    ) -> Result<LeaseId, LeaseError> {
        let depth = match parent {
            None => 0,
            Some(p) => {
                let parent_lease = self.get(p).ok_or(LeaseError::NoSuchLease(p))?;
                parent_lease.depth + 1
            }
        };
        if depth >= MAX_DEPTH {
            let mut chain = self.chain(parent);
            chain.push(u32::MAX); // the request that would have been
            return Err(LeaseError::DepthExceeded {
                cap: MAX_DEPTH,
                chain,
            });
        }

        match probe.probe(kind, amount) {
            ProbeVerdict::Fail(why) => Err(LeaseError::Refused { kind, amount, why }),
            ProbeVerdict::Pass(admitted_on) => {
                let id = self.leases.len() as LeaseId;
                self.leases.push(Some(Lease {
                    id,
                    parent,
                    kind,
                    amount,
                    depth,
                    state: LeaseState::Leased,
                    rent: Receipt::ZERO,
                    children: Vec::new(),
                    admitted_on,
                }));
                if let Some(p) = parent {
                    if let Some(pl) = self.leases[p as usize].as_mut() {
                        pl.children.push(id);
                    }
                }
                self.ledger.opened += 1;
                Ok(id)
            }
        }
    }

    /// Pay rent: write a receipt of REAL WORK.
    ///
    /// §9 Q1 — the receipts ARE the rent. A working holder writes receipts anyway, so rent costs
    /// nothing extra; a holder producing no receipts is not paying, whatever else it is doing.
    pub fn pay_rent(&mut self, id: LeaseId, receipt: Receipt) -> Result<(), LeaseError> {
        let l = self
            .leases
            .get_mut(id as usize)
            .and_then(|s| s.as_mut())
            .ok_or(LeaseError::NoSuchLease(id))?;
        if matches!(l.state, LeaseState::Released | LeaseState::Convicted) {
            return Err(LeaseError::AlreadyEnded(id, l.state));
        }
        l.rent = l.rent.merge(receipt);
        l.state = if receipt.0 > 0 {
            LeaseState::Active
        } else {
            LeaseState::Idle
        };
        self.ledger.rent = self.ledger.rent.merge(receipt);
        Ok(())
    }

    /// Release a lease, **children first** (D9).
    ///
    /// Leaf-to-root: a parent cannot be released while it still holds children, because the
    /// children's resources are held THROUGH it. Receipts compose upward as the tree unwinds.
    pub fn release(&mut self, id: LeaseId) -> Result<u64, LeaseError> {
        let lease = self.get(id).ok_or(LeaseError::NoSuchLease(id))?;
        if matches!(lease.state, LeaseState::Released | LeaseState::Convicted) {
            return Err(LeaseError::AlreadyEnded(id, lease.state));
        }
        let children = lease.children.clone();
        let mut n = 0;
        for c in children {
            // A child already ended (released or convicted) is not an error here — conviction is
            // one of the ways a child legitimately leaves the tree, and it has already been
            // accounted.
            if let Some(cl) = self.get(c) {
                if !matches!(cl.state, LeaseState::Released | LeaseState::Convicted) {
                    n += self.release(c)?;
                }
            }
        }
        let l = self.leases[id as usize].as_mut().unwrap();
        l.state = LeaseState::Released;
        self.ledger.released += 1;
        Ok(n + 1)
    }

    /// Convict a lease: we held it validly and the resource went away underneath us.
    ///
    /// D9 — the conviction SURFACES. It moves the parent's books (through the shared ledger) and
    /// is recorded with its evidence, rather than being absorbed as a silent error.
    pub fn convict(&mut self, id: LeaseId, evidence: &'static str) -> Result<(), LeaseError> {
        let lease = self.get(id).ok_or(LeaseError::NoSuchLease(id))?;
        if matches!(lease.state, LeaseState::Released | LeaseState::Convicted) {
            return Err(LeaseError::AlreadyEnded(id, lease.state));
        }
        // A convicted parent takes its children with it: they were held through it.
        let children = lease.children.clone();
        for c in children {
            if let Some(cl) = self.get(c) {
                if !matches!(cl.state, LeaseState::Released | LeaseState::Convicted) {
                    self.convict(c, "parent convicted")?;
                }
            }
        }
        let l = self.leases[id as usize].as_mut().unwrap();
        l.state = LeaseState::Convicted;
        self.ledger.convicted += 1;
        self.convictions.push((id, evidence));
        Ok(())
    }

    /// Record a reaping. Counted in the ledger so a reaping cannot happen without moving a
    /// number the audit reads.
    pub(crate) fn note_reaped(&mut self) {
        self.ledger.reaped += 1;
    }
}
