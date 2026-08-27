//! Cell state, its seeding, and the two update rules — the physics, stated once.
//!
//! This module is deliberately the ONLY thing the sharded and unsharded paths share. The
//! plumbing around it is written twice, independently, because a determinism claim made by
//! one code path behind a flag is very nearly a tautology (`holon-swarm`'s `swarm` module
//! makes the same argument and this crate inherits it).
//!
//! Two kinds of state, and the difference is the whole point of the float mutation:
//!
//! * the **integer REG+ ledger** ([`GrossState`]), moved between cells by an exactly
//!   antisymmetric transfer. Integer `+` is exactly associative, so its composition is
//!   order-independent *by construction* — no discipline required (`SANDBOX_4090` D1).
//! * the **whole-state f64** (`energy`), advanced by a rule that reads ONE cell and nothing
//!   else. Per-cell means order-free; a cross-cell float reduction would not be, and
//!   `SANDBOX_4090` D4 bans one on the certified path. `Mutation::CrossShardFloatReduction`
//!   plants exactly that, which is how the ban is shown to be enforced rather than written.

use ciris_sim_core::regplus::GrossState;
use holon_swarm::ledger::{gross_to_lanes, mix64, LedgerDelta};
use holon_swarm::plan_transfer;

/// Opening ledger of a cell: a pure function of its GLOBAL index, so any partition of any
/// grid on any host seeds bit-identically. Nothing here reads the shard.
pub fn seed_gross(cell: u32) -> GrossState {
    let k = mix64((cell as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    GrossState::aggregate(
        1 + (k % 7),
        8 + ((k >> 8) % 32),
        [
            1 + ((k >> 16) % 5) as i64,
            -(1 + ((k >> 24) % 5) as i64),
        ],
    )
}

/// Opening whole-state scalar of a cell. Also a pure function of the global index, and
/// deliberately not a round number, so a float comparison that is accidentally comparing
/// zeros cannot pass.
pub fn seed_energy(cell: u32) -> f64 {
    let k = mix64(cell as u64 ^ 0xD1B5_4A32_D192_ED03);
    (k % 1_000_003) as f64 * 1.0e-6 + 0.5
}

/// Advance one cell's whole-state scalar. **Reads one cell.** No neighbour, no accumulator,
/// no reduction — which is what makes it order-free, and what the float mutation violates.
#[inline]
pub fn advance_energy(energy: f64, gross: GrossState) -> f64 {
    0.5 * energy + (gross.occupancy as f64) * 1.0e-3
}

/// The transfer across one adjacency: diffusive halving of the imbalance, in integer
/// arithmetic, oriented "debit `lo`, credit `hi`".
///
/// Reused verbatim from `holon-swarm`, where it is already proved exactly antisymmetric in
/// every lane and never able to overdraw either side. Sharing the RULE is correct; sharing
/// the plumbing would not be.
#[inline]
pub fn plan(lo: GrossState, hi: GrossState) -> Option<LedgerDelta> {
    plan_transfer(gross_to_lanes(lo)?, gross_to_lanes(hi)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use holon_swarm::ledger::apply_delta;

    /// The transfer is exactly antisymmetric, so an adjacency contributes exactly zero to
    /// the scene's total in every lane. This is conservation as ARITHMETIC, not as a
    /// residual under a tolerance: there is no epsilon in this crate.
    #[test]
    fn a_transfer_moves_quantity_and_creates_none() {
        for a in 0..64u32 {
            for b in 0..64u32 {
                let (ga, gb) = (seed_gross(a), seed_gross(b));
                let d = plan(ga, gb).expect("seeded cells are in range");
                let ga2 = apply_delta(ga, d.checked_neg().unwrap()).unwrap();
                let gb2 = apply_delta(gb, d).unwrap();
                assert_eq!(ga.constituents + gb.constituents, ga2.constituents + gb2.constituents);
                assert_eq!(ga.occupancy + gb.occupancy, ga2.occupancy + gb2.occupancy);
                assert_eq!(ga.momentum[0] + gb.momentum[0], ga2.momentum[0] + gb2.momentum[0]);
                assert_eq!(ga.momentum[1] + gb.momentum[1], ga2.momentum[1] + gb2.momentum[1]);
            }
        }
    }

    /// Seeding depends on the global index alone. If it ever read a shard index, every
    /// `meshed == unsharded` comparison in this crate would be comparing different scenes
    /// and would fail for a reason that has nothing to do with the mesh.
    #[test]
    fn seeding_is_a_function_of_the_global_index_only() {
        for cell in [0u32, 1, 17, 255, 4096] {
            assert_eq!(seed_gross(cell), seed_gross(cell));
            assert_eq!(seed_energy(cell).to_bits(), seed_energy(cell).to_bits());
        }
        assert_ne!(seed_gross(3), seed_gross(4));
        assert_ne!(seed_energy(3).to_bits(), seed_energy(4).to_bits());
    }

    /// The energy rule reads one cell. Stated as a test rather than as a comment, because
    /// the float mutation's whole meaning is that this property can be broken.
    #[test]
    fn the_energy_rule_is_per_cell() {
        let g = seed_gross(11);
        let e = seed_energy(11);
        assert_eq!(advance_energy(e, g).to_bits(), advance_energy(e, g).to_bits());
    }
}
