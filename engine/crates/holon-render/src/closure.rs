//! The molecular tier's units as `holon-closure`'s closures.
//!
//! # What it is
//!
//! A thin adapter over [`Sim::units_reading`] and nothing else. That reading is FIELD-3's
//! unit assignment — each hydrogen to the oxygen its O–H curve holds lowest, a water unit
//! an oxygen with exactly two — and it stays exactly where it is: `sim.rs` is untouched,
//! `assign_units` still writes `unit_of`, `legality_radius` still consults the reading, and
//! the LIQUID-1 door still counts 128. What this module adds is the SHAPE: the same units,
//! typed as the object `holon-lattice` and `holon-lens` also carry, so a water unit, a
//! bonded lattice pair and an H-bond component stop being three types.
//!
//! # What it declares
//!
//! * **A closure at this tier is one water unit**, its members the atom indices of the
//!   oxygen and its two hydrogens, ascending. The oxygen's own index is its [`ClosureId`],
//!   which is `units_reading`'s own convention (`units[o] == o` marks a unit's root), so the
//!   two representations name a unit by the same number.
//! * **The six channels are `holon-closure`'s [`Channel`], and they are this crate's
//!   [`ChannelId`]**. Two declarations of one table is how two tables come to disagree, so
//!   [`channel_of`] maps between them and [`the_two_channel_tables_agree`] asserts the plain
//!   name, the kind and the ORDER agree row by row.
//!
//!   [`the_two_channel_tables_agree`]: crate::closure::tests
//! * **A free atom is not a closure of one.** `units_reading` marks an atom outside every
//!   unit with [`crate::seam::FREE`], and [`free_atoms`] reports those separately: a free
//!   hydrogen is a unit that dissolved, which is the rewrite production's own signal
//!   (LIQUID-1's MAXBOND screen, fence I-5), and folding it in as a one-atom closure would
//!   hide exactly that.
//!
//! # What it refuses
//!
//! * **It computes no assignment.** Every member here comes out of `units_reading`; this
//!   module cannot disagree with it because it has no rule of its own.
//! * **It serves no ledger row.** The closures come back with the ledger UNSERVED on all
//!   six channels. The molecular tier's six were measured on the dimer, not on a liquid
//!   frame, and a row filled in here with a zero would read as "measured zero" when it means
//!   "nobody looked" — the distinction [`holon_closure::LedgerRow`] exists to keep.
//! * **It sets no rent.** CT-2 measured the dimer's retention; a rent stamped on a unit by
//!   an adapter would be a measurement nobody made.

use crate::channel::ChannelId;
use crate::seam::FREE;
use crate::sim::Sim;
use holon_closure::{Channel, Closure, ClosureId, TierId};

/// The tier a water unit lives on.
pub const TIER: TierId = TierId::MOLECULAR;

/// This crate's channel key for one of `holon-closure`'s six, and back.
///
/// The two enums are in the SAME order — rate order, `GANTT2.md`'s vocabulary table — and
/// the test below asserts it row by row rather than trusting the two lists to stay aligned.
pub const fn channel_of(id: ChannelId) -> Channel {
    match id {
        ChannelId::Field => Channel::Presence,
        ChannelId::Induction => Channel::Accommodation,
        ChannelId::PairDispersion => Channel::Attunement,
        ChannelId::ThreeBody => Channel::Concert,
        ChannelId::Exchange => Channel::Refusal,
        ChannelId::ChargeTransfer => Channel::Sharing,
    }
}

/// A unit reading as closures: one per unit, members ascending, in ascending oxygen order.
///
/// `units` is [`Sim::units_reading`]'s output: `units[o] == o` marks a unit root,
/// `units[h] == o` puts atom `h` in that unit, and [`crate::seam::FREE`] marks an atom in
/// none. The closure's [`ClosureId`] is the root's own atom index, so `closures[k].0` is the
/// oxygen this crate already names the unit by.
pub fn closures_from_units(units: &[u32]) -> Vec<(ClosureId, Closure)> {
    let mut out = Vec::new();
    for (root, &u) in units.iter().enumerate() {
        if u != root as u32 {
            continue;
        }
        let members: Vec<u32> =
            units.iter().enumerate().filter(|(_, &v)| v == root as u32).map(|(i, _)| i as u32).collect();
        let c = Closure::new(TIER, members).expect("a unit root is a member of its own unit");
        out.push((root as ClosureId, c));
    }
    out
}

/// The scene's units as closures, read off the sim's own assignment.
pub fn unit_closures(sim: &Sim) -> Vec<(ClosureId, Closure)> {
    closures_from_units(&sim.units_reading()[..sim.n])
}

/// The atoms inside no unit. A free hydrogen is a dissolved unit, not a closure of one.
pub fn free_atoms(units: &[u32]) -> Vec<u32> {
    units.iter().enumerate().filter(|(_, &u)| u == FREE).map(|(i, _)| i as u32).collect()
}

/// The unit reading rebuilt from the closures — the round trip that makes "bit for bit"
/// checkable rather than asserted.
///
/// `n` is the atom count, because the closures do not carry the atoms they exclude.
pub fn units_from_closures(n: usize, closures: &[(ClosureId, Closure)]) -> Vec<u32> {
    let mut out = vec![FREE; n];
    for (root, c) in closures {
        for &m in c.members() {
            out[m as usize] = *root;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel::CHANNELS;

    /// ONE channel table, checked to be one. A rename on either side fails here rather than
    /// producing two answers about one channel.
    #[test]
    fn the_two_channel_tables_agree() {
        assert_eq!(CHANNELS.len(), holon_closure::CHANNELS.len());
        for (i, row) in CHANNELS.iter().enumerate() {
            let c = channel_of(row.id);
            assert_eq!(c, holon_closure::CHANNELS[i], "row {i} is a different channel");
            assert_eq!(c.index(), i, "{} is stored at a different row", row.plain);
            assert_eq!(c.plain(), row.plain, "row {i}'s plain name");
            assert_eq!(c.kind(), format!("{:?}", row.kind), "row {i}'s kind");
            assert_eq!(c.arity(), row.arity, "row {i}'s arity");
        }
    }

    /// The reading and the closures are the same statement, both ways.
    #[test]
    fn the_units_round_trip_through_the_closures() {
        // three waters and one free hydrogen: O H H | O H H | O H H | H
        let z: Vec<u32> = vec![8, 1, 1, 8, 1, 1, 8, 1, 1, 1];
        let units: Vec<u32> = vec![0, 0, 0, 3, 3, 3, 6, 6, 6, FREE];
        let cs = closures_from_units(&units);
        assert_eq!(cs.len(), 3);
        assert_eq!(cs[0].0, 0);
        assert_eq!(cs[0].1.members(), &[0, 1, 2]);
        assert_eq!(cs[2].1.members(), &[6, 7, 8]);
        assert!(cs.iter().all(|(_, c)| c.tier == TIER));
        assert_eq!(free_atoms(&units), vec![9]);
        assert_eq!(units_from_closures(z.len(), &cs), units);
    }

    /// An adapter that filled the ledger in would be inventing a measurement.
    #[test]
    fn a_units_closure_carries_no_ledger_and_no_rent() {
        let units: Vec<u32> = vec![0, 0, 0];
        let cs = closures_from_units(&units);
        assert!(cs[0].1.ledger.is_empty(), "nobody measured a row here");
        assert_eq!(cs[0].1.rent, None, "CT-2 measured a retention; this adapter did not");
    }

    #[test]
    fn a_scene_with_no_units_gives_no_closures_and_names_every_free_atom() {
        let units: Vec<u32> = vec![FREE, FREE, FREE];
        assert!(closures_from_units(&units).is_empty());
        assert_eq!(free_atoms(&units), vec![0, 1, 2]);
    }
}
