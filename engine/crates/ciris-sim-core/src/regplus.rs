//! Executable REG+ conservation labels.
//!
//! `Core/Lattice.lean` proves the six-direction local state has 64 occupancies split
//! into 53 `(N, P)` sectors. This module carries that exact finite object into the
//! runtime and extends its additive conserved label to recursively composed CIRIS
//! holons. It does not invent a collision law: transitions are supplied elsewhere and
//! can be checked here for sector preservation.

/// The six FHP directions in the axial integer coordinates used by
/// `CIRISOntology.Core.Lattice.dir`.
pub const DIRECTIONS: [[i64; 2]; 6] = [[1, 0], [0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1]];

/// One of the 53 local REG+ conservation sectors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SectorLabel {
    pub occupancy: u8,
    pub momentum: [i8; 2],
}

/// Additive REG+ gross state for a recursively composed holon.
///
/// `constituents` counts terminal holons represented by this holon. `occupancy`
/// and `momentum` are the sums of the local `(N, P)` conservation labels. Whole-only
/// information deliberately does not live here; it belongs to the holon and is
/// not required to factor through the children.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GrossState {
    pub constituents: u64,
    pub occupancy: u64,
    pub momentum: [i64; 2],
}

impl GrossState {
    pub const ZERO: Self = Self {
        constituents: 0,
        occupancy: 0,
        momentum: [0, 0],
    };

    /// The gross state of one terminal holon carrying a local REG+ occupancy.
    pub const fn leaf(local: u8) -> Self {
        let sector = sector(local);
        Self {
            constituents: 1,
            occupancy: sector.occupancy as u64,
            momentum: [sector.momentum[0] as i64, sector.momentum[1] as i64],
        }
    }

    /// A supplied aggregate, useful when the recursively implied children are latent
    /// rather than resident in the fixed-capacity refinement window.
    pub const fn aggregate(constituents: u64, occupancy: u64, momentum: [i64; 2]) -> Self {
        Self {
            constituents,
            occupancy,
            momentum,
        }
    }

    /// Exact additive composition of two gross states.
    pub const fn combine(self, other: Self) -> Self {
        Self {
            constituents: self.constituents + other.constituents,
            occupancy: self.occupancy + other.occupancy,
            momentum: [
                self.momentum[0] + other.momentum[0],
                self.momentum[1] + other.momentum[1],
            ],
        }
    }

    /// Checked composition for untrusted/runtime holarchies. Overflow means the
    /// aggregate cannot be represented by the cross-host `u64`/`s64` REG+ ledger.
    pub fn checked_combine(self, other: Self) -> Option<Self> {
        Some(Self {
            constituents: self.constituents.checked_add(other.constituents)?,
            occupancy: self.occupancy.checked_add(other.occupancy)?,
            momentum: [
                self.momentum[0].checked_add(other.momentum[0])?,
                self.momentum[1].checked_add(other.momentum[1])?,
            ],
        })
    }
}

/// Is direction `direction` occupied in the six-bit local state?
pub const fn occupied(local: u8, direction: usize) -> bool {
    assert!(local < 64 && direction < 6);
    local & (1 << direction) != 0
}

/// Compute the exact local `(N, P)` label proved in `Core/Lattice.lean`.
pub const fn sector(local: u8) -> SectorLabel {
    assert!(local < 64);
    let mut occupancy = 0_u8;
    let mut momentum = [0_i8; 2];
    let mut direction = 0;
    while direction < 6 {
        if occupied(local, direction) {
            occupancy += 1;
            momentum[0] += DIRECTIONS[direction][0] as i8;
            momentum[1] += DIRECTIONS[direction][1] as i8;
        }
        direction += 1;
    }
    SectorLabel {
        occupancy,
        momentum,
    }
}

/// Whether a supplied local transition obeys REG+ conservation.
pub const fn transition_preserves_sector(before: u8, after: u8) -> bool {
    let a = sector(before);
    let b = sector(after);
    a.occupancy == b.occupancy && a.momentum[0] == b.momentum[0] && a.momentum[1] == b.momentum[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_sector_table_matches_the_lean_theorem() {
        let mut labels = [SectorLabel::default(); 64];
        let mut dimensions = [0_u8; 64];
        let mut sectors = 0;
        for local in 0..64_u8 {
            let label = sector(local);
            let mut found = None;
            for i in 0..sectors {
                if labels[i] == label {
                    found = Some(i);
                    break;
                }
            }
            let index = match found {
                Some(index) => index,
                None => {
                    labels[sectors] = label;
                    sectors += 1;
                    sectors - 1
                }
            };
            dimensions[index] += 1;
        }

        let mut dim_1 = 0;
        let mut dim_2 = 0;
        let mut dim_3 = 0;
        for dimension in &dimensions[..sectors] {
            match dimension {
                1 => dim_1 += 1,
                2 => dim_2 += 1,
                3 => dim_3 += 1,
                _ => panic!("unexpected REG+ sector dimension {dimension}"),
            }
        }
        assert_eq!((sectors, dim_1, dim_2, dim_3), (53, 44, 7, 2));
    }

    #[test]
    fn opposite_pair_is_the_zero_momentum_two_occupancy_sector() {
        for local in [9, 18, 36] {
            assert_eq!(
                sector(local),
                SectorLabel {
                    occupancy: 2,
                    momentum: [0, 0],
                }
            );
        }
    }

    #[test]
    fn gross_composition_is_additive() {
        let composed = GrossState::leaf(9)
            .combine(GrossState::leaf(18))
            .combine(GrossState::leaf(36));
        assert_eq!(composed.constituents, 3);
        assert_eq!(composed.occupancy, 6);
        assert_eq!(composed.momentum, [0, 0]);
    }

    #[test]
    fn checked_composition_rejects_cross_host_integer_overflow() {
        let maximum = GrossState::aggregate(u64::MAX, u64::MAX, [i64::MAX, i64::MIN]);
        assert_eq!(maximum.checked_combine(GrossState::leaf(1)), None);
    }
}
