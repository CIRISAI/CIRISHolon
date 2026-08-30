//! The ledger: integer receipts, and the exact identity that makes a leak a PROOF.
//!
//! # Why every number in here is an integer
//!
//! `MergeLaw.lean`'s theorems (`shardedFold_invariant`, `digest_convicts`) are stated over an
//! `AddCommMonoid` and they are EXACT. Counts and bytes live in one; wall-clock seconds and
//! throughput ratios do not, because float addition is not associative and a "certificate" over
//! floats would be a certificate whose value depends on the order its shards were merged.
//!
//! So the design rule (RESOURCE_DESIGN D8) is enforced here rather than documented here: the
//! certificate carries `u64` only. [`Receipt::from_measurement`] is the one door a float can
//! approach, and it REFUSES anything that is not an exact non-negative integer — the plant that
//! smuggles `2.5` through it must come back `Err`.
//!
//! # The identity
//!
//! ```text
//!     opened  ==  released + convicted + live
//! ```
//!
//! Exact, over integers, at every level of the recursion. A leak is a NONZERO RESIDUAL — which
//! makes "this run leaked a lease" a proof rather than a suspicion, and is the whole reason the
//! receipts had to be integers.

/// Why a receipt could not be made from a measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiptError {
    /// The value was not finite. A NaN or an infinity in the books is not a small error; it is
    /// an accounting system that has stopped meaning anything.
    NotFinite,
    /// The value was negative. Work done does not go backwards.
    Negative,
    /// The value had a fractional part. THIS is the smuggled float: 2.5 bytes moved, 0.5 nodes
    /// solved. Rounding it would put a number in the certificate that no measurement produced.
    Fractional,
    /// The value was too large for `u64`.
    OutOfRange,
}

/// A unit of rent: an integer count of work actually done.
///
/// Constructed from an integer directly, or from a float ONLY through
/// [`Receipt::from_measurement`], which refuses anything inexact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct Receipt(pub u64);

impl Receipt {
    pub const ZERO: Receipt = Receipt(0);

    /// The honest door for a float.
    ///
    /// A caller holding a measured quantity — bytes moved, nodes solved — may only turn it into
    /// rent if it is exactly an integer. Everything else is refused with the reason, because the
    /// alternative is a certificate containing a number that rounding invented.
    pub fn from_measurement(v: f64) -> Result<Receipt, ReceiptError> {
        if !v.is_finite() {
            return Err(ReceiptError::NotFinite);
        }
        if v < 0.0 {
            return Err(ReceiptError::Negative);
        }
        if v.fract() != 0.0 {
            return Err(ReceiptError::Fractional);
        }
        if v > u64::MAX as f64 {
            return Err(ReceiptError::OutOfRange);
        }
        Ok(Receipt(v as u64))
    }

    /// Wrapping addition, which is associative and commutative unconditionally — the same
    /// argument `holon-gpu` makes for taking the ring out of a device reduction, and what lets
    /// receipts fold up the recursion in any order.
    #[inline]
    pub fn merge(self, other: Receipt) -> Receipt {
        Receipt(self.0.wrapping_add(other.0))
    }

    pub fn fold<I: IntoIterator<Item = Receipt>>(items: I) -> Receipt {
        items.into_iter().fold(Receipt::ZERO, Receipt::merge)
    }
}

/// The books for one resource class.
///
/// Counts only. Nothing in here is a duration, a rate, or a utilisation — those are REPORTED
/// elsewhere and never certified.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ledger {
    pub opened: u64,
    pub released: u64,
    pub convicted: u64,
    /// Total rent paid into this class: the sum of every receipt written by every holder.
    pub rent: Receipt,
    /// Reapings, with their evidence recorded separately by the caller. Counted here so a
    /// reaping cannot happen without moving a number the audit reads.
    pub reaped: u64,
}

impl Ledger {
    /// Leases still outstanding. `None` if the books are inconsistent — which cannot happen
    /// through this type's own API, and is therefore a corruption signal rather than an
    /// arithmetic possibility.
    pub fn live(&self) -> Option<u64> {
        self.opened.checked_sub(self.released + self.convicted)
    }

    /// **The identity.** `opened == released + convicted + live`, exact over integers.
    ///
    /// Returns the residual: `0` for consistent books, nonzero for a leak, `None` for books that
    /// have gone backwards. This is `digest_convicts` at the accounting layer — a nonzero
    /// residual is a PROOF that a lease was opened and never accounted for, not a heuristic.
    pub fn residual(&self, expected_live: u64) -> Option<u64> {
        let live = self.live()?;
        Some(live.abs_diff(expected_live))
    }

    /// Whether the books balance against a known count of outstanding leases.
    pub fn balances(&self, expected_live: u64) -> bool {
        self.residual(expected_live) == Some(0)
    }

    /// Fold two ledgers — a child's into its parent's. Associative and commutative, so the order
    /// children are absorbed in cannot change the parent's books.
    pub fn merge(self, other: Ledger) -> Ledger {
        Ledger {
            opened: self.opened.wrapping_add(other.opened),
            released: self.released.wrapping_add(other.released),
            convicted: self.convicted.wrapping_add(other.convicted),
            rent: self.rent.merge(other.rent),
            reaped: self.reaped.wrapping_add(other.reaped),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **PLANT D8 — a float smuggled into a certificate is REFUSED.**
    ///
    /// Carrier asserted first (M-PLANT-SECTOR): the door must accept a legitimate integral
    /// measurement, or "it refuses everything" would pass this test while being useless.
    #[test]
    fn plant_d8_a_smuggled_float_is_refused() {
        // the carrier: the door works for real measurements
        assert_eq!(Receipt::from_measurement(4096.0), Ok(Receipt(4096)));
        assert_eq!(Receipt::from_measurement(0.0), Ok(Receipt(0)));

        // the plant: every inexact value, refused WITH ITS REASON
        assert_eq!(
            Receipt::from_measurement(2.5),
            Err(ReceiptError::Fractional),
            "2.5 units of work entered the certificate; rounding would have invented a number \
             no measurement produced"
        );
        assert_eq!(Receipt::from_measurement(-1.0), Err(ReceiptError::Negative));
        assert_eq!(Receipt::from_measurement(f64::NAN), Err(ReceiptError::NotFinite));
        assert_eq!(
            Receipt::from_measurement(f64::INFINITY),
            Err(ReceiptError::NotFinite)
        );
        assert_eq!(
            Receipt::from_measurement(1e30),
            Err(ReceiptError::OutOfRange)
        );

        // A wall-clock duration is the realistic smuggling route, and it is refused.
        let elapsed_seconds = 0.15_f64;
        assert!(
            Receipt::from_measurement(elapsed_seconds).is_err(),
            "a duration became rent; durations are reported, never certified"
        );
    }

    /// The identity holds, and folding children in any order gives the same books.
    #[test]
    fn the_identity_is_exact_and_order_free() {
        let children = [
            Ledger { opened: 5, released: 3, convicted: 1, rent: Receipt(70), reaped: 0 },
            Ledger { opened: 9, released: 9, convicted: 0, rent: Receipt(12), reaped: 1 },
            Ledger { opened: 2, released: 0, convicted: 2, rent: Receipt(3), reaped: 0 },
        ];
        let forward = children.iter().copied().fold(Ledger::default(), Ledger::merge);
        let reverse = children.iter().rev().copied().fold(Ledger::default(), Ledger::merge);
        assert_eq!(forward, reverse, "the books saw the order children were absorbed in");

        // opened 16, released 12, convicted 3 -> 1 live
        assert_eq!(forward.live(), Some(1));
        assert!(forward.balances(1));
        assert_eq!(forward.residual(1), Some(0));
    }

    /// A leak is a nonzero residual, and it is a proof rather than a suspicion.
    #[test]
    fn a_leak_is_a_nonzero_residual() {
        let leaked = Ledger { opened: 4, released: 1, convicted: 0, ..Default::default() };
        // Three leases are outstanding; a caller that believes only one is live is short two.
        assert_eq!(leaked.live(), Some(3));
        assert_eq!(leaked.residual(1), Some(2));
        assert!(!leaked.balances(1));
    }

    /// Books that have gone backwards are a corruption signal, not an underflow panic.
    #[test]
    fn inconsistent_books_report_rather_than_wrap() {
        let broken = Ledger { opened: 1, released: 5, convicted: 0, ..Default::default() };
        assert_eq!(broken.live(), None, "a subtraction underflowed into a plausible count");
        assert_eq!(broken.residual(0), None);
    }
}
