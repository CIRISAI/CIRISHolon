//! The mutation set: the deliberate defects that make G1's gate able to fail.
//!
//! # Why this exists in the shape it does
//!
//! `holon-mesh`'s header names the trap and this crate inherits it. A reordered merge over
//! exact lanes produces the IDENTICAL result, so "reorder the work and assert the answer
//! changed" cannot pass against a correct implementation — a test built that way would be
//! measuring nothing while looking rigorous.
//!
//! The set is therefore SPLIT, and it is only the split that proves anything:
//!
//! | mutation | must the table move? | what it would mean if it did the other thing |
//! |---|---|---|
//! | [`Mutation::ReverseRegionOrder`] | **no** | the design leaks the schedule into the numbers |
//! | [`Mutation::WorkerLocalWarmStart`] | **yes** | the canonical region decomposition is unnecessary, and the warm-start measurement that motivated it was wrong |
//! | [`Mutation::CorruptNode`] | convicted by the digest | the certificate is decorative |
//! | [`Mutation::WrongWarmStart`] | the node VOIDs | a bad warm start writes a silently wrong table entry |
//!
//! A gate that only ran the "must not move" half would pass on an implementation that
//! ignored its inputs. A gate that only ran the "must move" half would pass on one that was
//! nondeterministic. Both halves, or neither is evidence.

/// A deliberate defect, applied to one generation run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mutation {
    /// Hand the regions out in reverse order.
    ///
    /// This MUST NOT change the assembled table. It is the reordering the region design is
    /// built to absorb, and it is the control for the mutation below: without it, a
    /// generator that simply ignored the warm start would pass the whole set.
    ReverseRegionOrder,

    /// Warm-start each node from whatever THIS WORKER solved last, rather than from the
    /// node's canonical predecessor inside its region.
    ///
    /// This is the design defect the region decomposition exists to prevent, and it MUST
    /// change the assembled table when the worker count changes. It is the natural
    /// implementation — it needs no bookkeeping and it warm-starts every node including the
    /// region seeds, so it looks strictly better — and it silently makes the table a
    /// function of the worker count.
    WorkerLocalWarmStart,

    /// Flip one bit of one node's energy in the assembled table AFTER the shard digests
    /// have been taken. Plant (iv).
    ///
    /// The digest must convict it. `bit` is an index into the 64 bits of the `f64`; low
    /// bits are the harder case and are what the plant should normally use, because a
    /// corruption that changed the energy visibly would be caught by anything.
    CorruptNode { node: u32, bit: u32 },

    /// Feed one node a deliberately wrong warm start — a vector with no relationship to the
    /// answer. Plant (iii).
    ///
    /// The node must VOID. What it must never do is write a different number into the table
    /// while looking healthy, which is exactly what it does without the variational guard:
    /// measured at 7.47 hartree of error with an ordinary-looking residual and the ordinary
    /// exit reason.
    WrongWarmStart { node: u32 },

    /// Feed EVERY node a deliberately wrong warm start.
    ///
    /// Exists because the trap is GEOMETRY-DEPENDENT, which was not obvious and cost this
    /// gate a false alarm. The same random start vector that traps a `(H,H,Cl)` solve 7.47
    /// hartree above the ground state at one geometry converges to the correct answer,
    /// within 3.3e-12 hartree, at another on the same grid. Whether a wrong start gets lost
    /// is a property of the level spacing where it is dropped, not of the species.
    ///
    /// Planting one node therefore samples the trap rather than testing it. Planting all of
    /// them measures HOW OFTEN the sector is non-empty and asserts the guard fires on every
    /// occasion that it is — which is the claim worth making.
    WrongWarmStartAll,
}

impl Mutation {
    /// A one-line name for a log or an assertion message.
    pub fn label(&self) -> String {
        match self {
            Mutation::ReverseRegionOrder => "reverse-region-order".into(),
            Mutation::WorkerLocalWarmStart => "worker-local-warm-start".into(),
            Mutation::CorruptNode { node, bit } => format!("corrupt-node({node},bit {bit})"),
            Mutation::WrongWarmStart { node } => format!("wrong-warm-start({node})"),
            Mutation::WrongWarmStartAll => "wrong-warm-start(all)".into(),
        }
    }

    /// Whether this mutation is expected to change the assembled table.
    ///
    /// Stated on the mutation itself rather than in each test, so the expectation travels
    /// with the defect and a test cannot quietly assert the convenient direction.
    pub fn must_change_the_table(&self) -> bool {
        match self {
            Mutation::ReverseRegionOrder => false,
            Mutation::WorkerLocalWarmStart => true,
            // The corruption is applied to the assembled table, so of course the table
            // differs; what is under test is the CERTIFICATE, not the table.
            Mutation::CorruptNode { .. } => true,
            // The node VOIDs, which is itself a change to the record.
            Mutation::WrongWarmStart { .. } => true,
            Mutation::WrongWarmStartAll => true,
        }
    }
}
