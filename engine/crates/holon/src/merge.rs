//! THE ONE MERGE LAW — there is no other transaction mechanism in the holon.
//!
//! Ledgers fold associatively and commutatively, so ANY sharding, ordering,
//! or distribution of a fold is deterministic without coordination — that is
//! the entire replacement for per-tier ACID, and it is a LAW WITH TESTS, not
//! a convention. Commits are arena APPENDS (identity is the index forever;
//! nothing is updated in place). Validity is the CERTIFICATE, not a lock.
//! Every tier's accumulation — mesh shard reduction, duplicate-branch
//! merging, sampler pair sums, coarse child aggregation, rent accounting —
//! is an instance of this one trait. A tier that cannot live on it is a
//! MISFIT against the theory and gets reported upstream as one.

use crate::ledger::Cyc;

pub trait MergeLedger: Sized + Clone + PartialEq + std::fmt::Debug {
    fn empty() -> Self;
    fn merge(self, other: Self) -> Self;
}

/// Tier-2 ledger: exact cyclotomic amplitudes.
impl MergeLedger for Cyc {
    fn empty() -> Self {
        Cyc::ZERO
    }
    fn merge(self, other: Self) -> Self {
        self.add(other)
    }
}

/// Tier-1 ledger: phases mod 4.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SignLedger(pub u8);
impl MergeLedger for SignLedger {
    fn empty() -> Self {
        SignLedger(0)
    }
    fn merge(self, other: Self) -> Self {
        SignLedger((self.0 + other.0) % 4)
    }
}

/// ℝ-tier ledger: rent/budget accounting. NOTE the fence: f64 addition is
/// NOT exactly associative, so this ledger's merge law holds only to a
/// declared tolerance — which is a conditioning statement, priced by
/// `sum_perturb_le`, and the reason the exact tiers keep exact rings. The
/// law test for this impl checks associativity WITHIN the tolerance and
/// would fire on a genuinely order-dependent accumulation.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RentLedger {
    pub paid: f64,
}
impl MergeLedger for RentLedger {
    fn empty() -> Self {
        RentLedger { paid: 0.0 }
    }
    fn merge(self, other: Self) -> Self {
        RentLedger { paid: self.paid + other.paid }
    }
}

/// The one fold — used by every consumer, shardable by construction.
pub fn fold<L: MergeLedger>(items: impl IntoIterator<Item = L>) -> L {
    items.into_iter().fold(L::empty(), |a, b| a.merge(b))
}
