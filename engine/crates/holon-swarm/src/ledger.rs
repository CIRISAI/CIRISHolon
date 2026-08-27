//! Signed integer deltas over the REG+ [`GrossState`] ledger.
//!
//! `GrossState` is a purely additive integer record (`constituents: u64`,
//! `occupancy: u64`, `momentum: [i64; 2]`). Sharding needs the *signed* companion: a
//! quantity that can be debited from one arena and credited to another. That is
//! [`LedgerDelta`]. Every operation here is checked — an overflow is a typed `None`,
//! never a wrap and never a panic, because on the GPU port the same arithmetic runs in
//! a kernel where a silent wrap would corrupt the global ledger invisibly.

use ciris_sim_core::regplus::GrossState;

/// Number of independent integer lanes in the REG+ ledger:
/// `constituents, occupancy, momentum[0], momentum[1]`.
///
/// The lane form is what crosses a shard boundary (and, in the GPU design, what lives in
/// a device buffer): four `i64` per holon, no pointers, no floats.
pub const LANES: usize = 4;

/// A signed movement of conserved ledger quantity.
///
/// The sign convention used everywhere in this crate: a transfer `d` for a boundary pair
/// `(lo, hi)` means **`lo` is debited `d` and `hi` is credited `d`**. Applying `-d` and
/// `+d` to the two sides is what makes the global sum invariant *by construction* rather
/// than by a post-hoc repair.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LedgerDelta {
    pub constituents: i64,
    pub occupancy: i64,
    pub momentum: [i64; 2],
}

impl LedgerDelta {
    pub const ZERO: Self = Self {
        constituents: 0,
        occupancy: 0,
        momentum: [0, 0],
    };

    pub const fn is_zero(&self) -> bool {
        self.constituents == 0
            && self.occupancy == 0
            && self.momentum[0] == 0
            && self.momentum[1] == 0
    }

    /// Exact negation. `i64::MIN` has no positive counterpart, so this is checked.
    pub fn checked_neg(self) -> Option<Self> {
        Some(Self {
            constituents: self.constituents.checked_neg()?,
            occupancy: self.occupancy.checked_neg()?,
            momentum: [
                self.momentum[0].checked_neg()?,
                self.momentum[1].checked_neg()?,
            ],
        })
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            constituents: self.constituents.checked_add(other.constituents)?,
            occupancy: self.occupancy.checked_add(other.occupancy)?,
            momentum: [
                self.momentum[0].checked_add(other.momentum[0])?,
                self.momentum[1].checked_add(other.momentum[1])?,
            ],
        })
    }

    pub fn checked_mul(self, factor: i64) -> Option<Self> {
        Some(Self {
            constituents: self.constituents.checked_mul(factor)?,
            occupancy: self.occupancy.checked_mul(factor)?,
            momentum: [
                self.momentum[0].checked_mul(factor)?,
                self.momentum[1].checked_mul(factor)?,
            ],
        })
    }

    pub const fn to_lanes(self) -> [i64; LANES] {
        [
            self.constituents,
            self.occupancy,
            self.momentum[0],
            self.momentum[1],
        ]
    }

    pub const fn from_lanes(lanes: [i64; LANES]) -> Self {
        Self {
            constituents: lanes[0],
            occupancy: lanes[1],
            momentum: [lanes[2], lanes[3]],
        }
    }
}

/// The unsigned ledger read as signed lanes. Fails only if a `u64` field exceeds
/// `i64::MAX`, which the cross-host REG+ ledger would already be unable to net.
pub fn gross_to_lanes(gross: GrossState) -> Option<[i64; LANES]> {
    Some([
        i64::try_from(gross.constituents).ok()?,
        i64::try_from(gross.occupancy).ok()?,
        gross.momentum[0],
        gross.momentum[1],
    ])
}

/// Inverse of [`gross_to_lanes`]. Fails on a negative count lane — which is exactly the
/// "you took more than was there" failure a shard exchange must not be able to hide.
pub fn lanes_to_gross(lanes: [i64; LANES]) -> Option<GrossState> {
    Some(GrossState {
        constituents: u64::try_from(lanes[0]).ok()?,
        occupancy: u64::try_from(lanes[1]).ok()?,
        momentum: [lanes[2], lanes[3]],
    })
}

/// Apply a signed delta to an unsigned REG+ ledger entry, checked in every lane.
///
/// `u64::checked_add_signed` returns `None` on both overflow and underflow, so a debit
/// larger than the entry holds is rejected rather than wrapped to a huge occupancy.
pub fn apply_delta(gross: GrossState, delta: LedgerDelta) -> Option<GrossState> {
    Some(GrossState {
        constituents: gross.constituents.checked_add_signed(delta.constituents)?,
        occupancy: gross.occupancy.checked_add_signed(delta.occupancy)?,
        momentum: [
            gross.momentum[0].checked_add(delta.momentum[0])?,
            gross.momentum[1].checked_add(delta.momentum[1])?,
        ],
    })
}

/// `after - before` as a signed delta. Used by the gate to compare what a shard's root
/// ledger actually moved by against what its receipts claim it moved by.
pub fn delta_between(after: GrossState, before: GrossState) -> Option<LedgerDelta> {
    let sub = |a: u64, b: u64| -> Option<i64> {
        i64::try_from(i128::from(a) - i128::from(b)).ok()
    };
    Some(LedgerDelta {
        constituents: sub(after.constituents, before.constituents)?,
        occupancy: sub(after.occupancy, before.occupancy)?,
        momentum: [
            after.momentum[0].checked_sub(before.momentum[0])?,
            after.momentum[1].checked_sub(before.momentum[1])?,
        ],
    })
}

/// Deterministic integer mixer (SplitMix64 finaliser). Supplies the local step's stir so
/// the shard interiors keep moving instead of freezing at a uniform fixed point — which
/// would make the determinism harness vacuous. It is a pure function of the round and
/// the holon index, so it is identical under every thread schedule.
#[inline]
pub fn mix64(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debit_larger_than_the_entry_is_rejected_not_wrapped() {
        let entry = GrossState::aggregate(1, 5, [0, 0]);
        assert_eq!(
            apply_delta(
                entry,
                LedgerDelta {
                    occupancy: -6,
                    ..LedgerDelta::ZERO
                }
            ),
            None
        );
        assert_eq!(
            apply_delta(
                entry,
                LedgerDelta {
                    occupancy: -5,
                    ..LedgerDelta::ZERO
                }
            )
            .unwrap()
            .occupancy,
            0
        );
    }

    #[test]
    fn momentum_overflow_is_rejected_not_wrapped() {
        let entry = GrossState::aggregate(1, 1, [1, 0]);
        assert_eq!(
            apply_delta(
                entry,
                LedgerDelta {
                    momentum: [i64::MAX, 0],
                    ..LedgerDelta::ZERO
                }
            ),
            None
        );
    }

    #[test]
    fn opposite_signs_compose_to_zero_in_every_lane() {
        let d = LedgerDelta {
            constituents: -3,
            occupancy: 17,
            momentum: [-9, 42],
        };
        assert_eq!(d.checked_add(d.checked_neg().unwrap()).unwrap(), LedgerDelta::ZERO);
    }
}
