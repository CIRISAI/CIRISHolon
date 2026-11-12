//! The call a consumer makes: `children(request) -> response`.
//!
//! `FSD_WORLD_MODEL.md` section 10. A consumer names a holon by its population and class, a
//! seed, and the registry rows it reads. It gets back children whose populations sum to the
//! parent's, and ONE VERDICT PER REQUESTED ROW, ALWAYS: a consumer cannot receive a number
//! without the word that says whether to trust it.
//!
//! What exists today, and this module says so in its answers rather than in a comment:
//! * the exact half (apportionment, the section law) for every row of type population;
//! * one statistical generator with a reference behind it: [`incell`], which divides a
//!   1/8 degree cell's people over tiles of each density class by the weights read in
//!   UNEVEN-0 from GHS-POP 2020. It is a reading of the reference used as weights, not a
//!   scored generator, so its verdict is `Unresolved`, with that reason;
//! * for every other row: `Unresolved`, with the reason "no generator" or "no reference".
//!
//! A row moves from `Unresolved` to `Certified` or `Refused` only when a generator has been
//! scored against a frozen reference inside a declared envelope. Nothing here does that yet.

use crate::apportion::apportion;
use crate::seed::{child_seed, Stream};
use crate::verdict::Verdict;

/// Households in a unit (HOUSEGEN-1): `world::household::generate`.
pub use crate::household;
/// Devices in a household (DEVICEGEN-1): `world::devices::generate`.
pub use crate::devices;
/// A person's day by the hour (PRESENCEGEN-2): `world::presence::day`.
pub use crate::presence;

/// The four types of row (FSD section 10.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowType {
    Population,
    Relationship,
    Attribute,
    Rhythm,
}

/// One registry row: a name a consumer may ask for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub name: &'static str,
    pub ty: RowType,
    /// Who reads it: a consumer's model, or `"decoration"`.
    pub consumer: &'static str,
    /// The generator that produces it, if any.
    pub generator: Option<&'static str>,
    /// The frozen reference behind that generator, if any.
    pub reference: Option<&'static str>,
    /// Has the generator been scored against the reference inside an envelope? `None` = no.
    pub scored: Option<bool>,
}

/// The answer for one requested row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub name: String,
    pub verdict: Verdict,
    pub reason: &'static str,
    pub reference: Option<&'static str>,
}

/// Resolve the verdict of every requested name. One answer per name, in order, always.
pub fn verdicts(registry: &[Row], want: &[&str], in_envelope: bool) -> Vec<Answer> {
    want.iter()
        .map(|&name| {
            let (verdict, reason, reference) = match registry.iter().find(|r| r.name == name) {
                None => (Verdict::Unresolved, "not a registry row", None),
                Some(r) if r.consumer == "decoration" => (Verdict::Unresolved, "decoration: a label, no certificate owed", r.reference),
                Some(r) if r.generator.is_none() => (Verdict::Unresolved, "no generator", None),
                Some(r) if r.reference.is_none() => (Verdict::Unresolved, "no reference", None),
                Some(r) if !in_envelope => (Verdict::Refused, "outside the reference's envelope", r.reference),
                Some(r) => match r.scored {
                    None => (Verdict::Unresolved, "a reading of the reference used as weights; no generator has been scored against it", r.reference),
                    Some(true) => (Verdict::Certified, "scored within budget", r.reference),
                    Some(false) => (Verdict::Refused, "scored outside budget", r.reference),
                },
            };
            Answer { name: name.to_string(), verdict, reason, reference }
        })
        .collect()
}

/// The registry as it stands. Short on purpose: it lists what this crate can answer for, and
/// a consumer's own file (the twin's `couplings.json`) lists what is wanted.
pub const REGISTRY: &[Row] = &[
    Row { name: "people.per_tile", ty: RowType::Population, consumer: "households per leaf", generator: Some("incell.v1"), reference: Some("UNEVEN-0"), scored: None },
    Row { name: "tile.density_class", ty: RowType::Attribute, consumer: "street class of a leaf", generator: Some("incell.v1"), reference: Some("UNEVEN-0"), scored: None },
    Row { name: "households.per_tile", ty: RowType::Population, consumer: "households per leaf", generator: Some("apportion"), reference: None, scored: None },
    Row { name: "persons.per_household", ty: RowType::Population, consumer: "device counts", generator: None, reference: None, scored: None },
    Row { name: "devices.per_household", ty: RowType::Population, consumer: "mesh", generator: None, reference: None, scored: None },
    Row { name: "router.nat_class", ty: RowType::Attribute, consumer: "reachability", generator: None, reference: None, scored: None },
    Row { name: "street.neighbour_distance", ty: RowType::Attribute, consumer: "decoration", generator: None, reference: None, scored: None },
    Row { name: "presence.people", ty: RowType::Rhythm, consumer: "activity by hour", generator: None, reference: None, scored: None },
];

/// Dividing a cell's people over tiles of each density class.
pub mod incell {
    use super::*;

    /// The notice that must travel with anything produced from these weights.
    pub const NOTICE: &str = "GHS-POP R2023A, European Commission, Joint Research Centre; Schiavina, Freire, Carioli and MacManus (2023), doi:10.2905/2FF68A52-5B5B-4A22-8F40-C41DA8332CFE; CC BY 4.0.";

    /// Cell classes 1..=5: village, suburban, terraced, apartment blocks, dense fabric.
    /// Tile classes 0..=5: empty, then the same five.
    ///
    /// UNEVEN-0 (`conformance/scene/tables/uneven0/transition_table.json`, grain 1 pixel, each
    /// cell counted once), in parts per million. Row = cell class, column = tile class.
    pub const TILES_PPM: [[u64; 6]; 5] = [
        [662_838, 304_219, 29_179, 3_482, 225, 33],
        [226_467, 409_551, 276_240, 74_584, 9_791, 3_359],
        [120_489, 236_696, 270_983, 249_844, 79_113, 42_878],
        [67_587, 118_343, 140_381, 214_856, 209_466, 249_368],
        [39_045, 59_578, 84_408, 148_650, 146_299, 522_020],
    ];
    pub const PEOPLE_PPM: [[u64; 6]; 5] = [
        [4_523, 564_636, 314_415, 100_190, 13_237, 3_092],
        [33, 73_840, 351_423, 354_357, 129_258, 91_090],
        [3, 8_838, 94_366, 339_586, 250_814, 306_393],
        [0, 1_264, 13_898, 98_656, 235_539, 650_642],
        [0, 309, 4_038, 32_883, 85_853, 876_917],
    ];

    /// One generated tile.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Tile {
        pub index: usize,
        pub class: u8,
        pub people: u64,
    }

    /// Divide `people` over `n_tiles` tiles of a cell of class `cell_class` (1..=5).
    ///
    /// How many tiles get each class follows the table's tile shares; how many people each
    /// class gets follows its people shares; within a class people are split by a seeded
    /// weight; which tile gets which class is a seeded shuffle. A class that was given people
    /// and no tile hands them to the nearest class that has a tile, so nothing is lost. The
    /// tiles' people sum to `people` for every seed.
    pub fn divide(cell_class: u8, people: u64, n_tiles: usize, seed: u64) -> Vec<Tile> {
        assert!((1..=5).contains(&cell_class), "cell class 1..=5");
        if n_tiles == 0 {
            return Vec::new();
        }
        let c = cell_class as usize - 1;
        let tile_counts = apportion(n_tiles as u64, &TILES_PPM[c]);
        let mut class_people = apportion(people, &PEOPLE_PPM[c]);
        for k in 0..6 {
            if class_people[k] > 0 && tile_counts[k] == 0 {
                let to = (0..6).filter(|&j| tile_counts[j] > 0 && j != 0).min_by_key(|&j| (j as i64 - k as i64).abs())
                    .or_else(|| (0..6).find(|&j| tile_counts[j] > 0)).expect("some class holds a tile");
                class_people[to] += class_people[k];
                class_people[k] = 0;
            }
        }
        let mut order: Vec<usize> = (0..n_tiles).collect();
        let mut r = Stream::new(seed);
        for i in (1..n_tiles).rev() {
            order.swap(i, r.below(i as u64 + 1) as usize);
        }
        let mut out = Vec::with_capacity(n_tiles);
        let mut at = 0usize;
        for k in 0..6 {
            let n = tile_counts[k] as usize;
            if n == 0 {
                continue;
            }
            let mut w = Stream::new(child_seed(seed, k as u64));
            let weights: Vec<u64> = (0..n).map(|_| 50 + w.below(100)).collect();
            let split = apportion(class_people[k], &weights);
            for j in 0..n {
                out.push(Tile { index: order[at], class: k as u8, people: split[j] });
                at += 1;
            }
        }
        out.sort_by_key(|t| t.index);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::incell::*;
    use super::*;

    #[test]
    fn one_answer_per_requested_row_always() {
        let want = ["people.per_tile", "router.nat_class", "street.neighbour_distance", "no.such.row", "households.per_tile"];
        let a = verdicts(REGISTRY, &want, true);
        assert_eq!(a.len(), want.len());
        assert_eq!(a.iter().map(|x| x.reason).collect::<Vec<_>>(), vec![
            "a reading of the reference used as weights; no generator has been scored against it",
            "no generator",
            "decoration: a label, no certificate owed",
            "not a registry row",
            "no reference",
        ]);
        assert!(a.iter().all(|x| x.verdict == Verdict::Unresolved));
    }

    #[test]
    fn nothing_is_certified_today_and_outside_the_envelope_is_refused() {
        let all: Vec<&str> = REGISTRY.iter().map(|r| r.name).collect();
        assert!(verdicts(REGISTRY, &all, true).iter().all(|a| a.verdict != Verdict::Certified));
        let a = verdicts(REGISTRY, &["people.per_tile"], false);
        assert_eq!((a[0].verdict, a[0].reason), (Verdict::Refused, "outside the reference's envelope"));
    }

    #[test]
    fn the_tables_are_shares() {
        for c in 0..5 {
            for t in [&TILES_PPM[c], &PEOPLE_PPM[c]] {
                let s: u64 = t.iter().sum();
                assert!((999_700..=1_000_300).contains(&s), "row sums to {s}");
            }
        }
    }

    #[test]
    fn the_tiles_sum_to_the_cell_for_every_seed_and_class() {
        for class in 1..=5u8 {
            for seed in 0..200u64 {
                for (people, n) in [(0u64, 225usize), (1, 225), (37, 4), (54_321, 225), (2_000_000, 240), (999, 1)] {
                    let t = divide(class, people, n, seed);
                    assert_eq!(t.len(), n);
                    assert_eq!(t.iter().map(|x| x.people).sum::<u64>(), people);
                    assert_eq!(t.iter().map(|x| x.index).collect::<Vec<_>>(), (0..n).collect::<Vec<_>>());
                }
            }
        }
    }

    #[test]
    fn a_suburban_cell_comes_out_as_the_reference_read_it() {
        // 225 tiles, 50,000 people: tile and people shares within one tile and 1e-3 of the table
        let t = divide(2, 50_000, 225, 9);
        for k in 0..6usize {
            let n = t.iter().filter(|x| x.class == k as u8).count() as f64 / 225.0;
            let p = t.iter().filter(|x| x.class == k as u8).map(|x| x.people).sum::<u64>() as f64 / 50_000.0;
            assert!((n - TILES_PPM[1][k] as f64 / 1e6).abs() < 1.0 / 225.0 + 1e-3, "tiles class {k}: {n}");
            assert!((p - PEOPLE_PPM[1][k] as f64 / 1e6).abs() < 1e-3, "people class {k}: {p}");
        }
        // and it is not the even spread: most people are not in suburban tiles
        let own = t.iter().filter(|x| x.class == 2).map(|x| x.people).sum::<u64>();
        assert!(own * 2 < 50_000);
    }

    #[test]
    fn the_same_seed_gives_the_same_cell_and_another_seed_moves_it() {
        assert_eq!(divide(3, 80_000, 225, 5), divide(3, 80_000, 225, 5));
        assert_ne!(divide(3, 80_000, 225, 5), divide(3, 80_000, 225, 6));
    }

    #[test]
    fn empty_tiles_hold_nobody_when_any_other_tile_exists() {
        let t = divide(1, 10_000, 225, 3);
        assert!(t.iter().filter(|x| x.class == 0).all(|x| x.people <= 50));   // the table gives empty tiles 0.45 % of people
        assert!(t.iter().filter(|x| x.class == 0).count() > 140);
    }
}
