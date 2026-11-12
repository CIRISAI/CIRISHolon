//! Households in a unit: whole households whose sizes sum to the unit's people, for every
//! seed. The first generator this crate ships for a consumer to call (HOUSEGEN-1,
//! `conformance/scene/HOUSEGEN1_PREREG.md`).
//!
//! The section law here is `ScenePopulation.pop_total` with the households as the children
//! and their sizes as the populations. The draw rescales nothing: sizes come one at a time
//! from a row of shares, and only the last household is touched to make the sum exact.

use crate::seed::{mix, Stream};
use crate::verdict::Verdict;

/// Pick an index from shares in any common unit (parts per million here).
fn pick(st: &mut Stream, shares: &[u64], total: u64) -> usize {
    let mut u = st.below(total);
    for (k, &w) in shares.iter().enumerate() {
        if u < w {
            return k;
        }
        u -= w;
    }
    shares.len() - 1
}

/// Draw household sizes summing to `people`.
///
/// `shares[k]` is the share of households of size `sizes[k]`. When a drawn size `s` exceeds
/// the `r` people left, a household of `r` is kept with probability `r/s`; otherwise the
/// `r` people join one earlier household. A unit with no earlier household keeps `r`.
pub fn draw(seed: u64, people: u64, shares: &[u64], sizes: &[u8]) -> Vec<u8> {
    assert_eq!(shares.len(), sizes.len(), "one size per share");
    assert!(sizes.iter().all(|&s| s > 0), "no empty household");
    let total: u64 = shares.iter().sum();
    assert!(total > 0, "a row with no households");
    let mut st = Stream::new(seed);
    let mut out: Vec<u8> = Vec::new();
    let mut left = people;
    while left > 0 {
        let s = sizes[pick(&mut st, shares, total)] as u64;
        if s <= left {
            out.push(s as u8);
            left -= s;
        } else {
            let r = left;
            if out.is_empty() || st.below(s) < r {
                out.push(r as u8);
            } else {
                let j = st.below(out.len() as u64) as usize;
                out[j] += r as u8;
            }
            left = 0;
        }
    }
    out
}

/// The draft's draw, kept as a PLANT: the last household simply takes what is left. It
/// obeys the section law and is biased toward small households (HOUSEGEN-1's review).
pub fn draw_truncating(seed: u64, people: u64, shares: &[u64], sizes: &[u8]) -> Vec<u8> {
    let total: u64 = shares.iter().sum();
    let mut st = Stream::new(seed);
    let mut out: Vec<u8> = Vec::new();
    let mut left = people;
    while left > 0 {
        let s = (sizes[pick(&mut st, shares, total)] as u64).min(left);
        out.push(s as u8);
        left -= s;
    }
    out
}

/// One number for 1,000 fixed calls, to compare builds (native against wasm32).
pub fn digest(shares: &[u64], sizes: &[u8]) -> u64 {
    let mut h = 0u64;
    for i in 0..1000u64 {
        let v = draw(mix(i), (i * 37) % 2001, shares, sizes);
        h = mix(h ^ v.len() as u64);
        for s in v {
            h = mix(h ^ s as u64);
        }
    }
    h
}

/// The reference a row comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reference {
    /// Zensus 2022 on its 1 km grid, by the tile's own density class (HOUSEHOLD-0, Germany).
    Germany2022,
    /// ACS 2020-2024 by census-tract density class (HOUSEHOLD-0, United States).
    UnitedStates2024,
}

/// Notices that must travel with anything produced from the rows.
pub const NOTICE_DE: &str = "Source: Statistische Aemter des Bundes und der Laender, Zensus 2022, Ergebnisse in Gitterzellen; Data licence Germany - attribution - Version 2.0 (dl-de/by-2-0), www.govdata.de/dl-de/by-2-0; data changed: aggregated by density class.";
pub const NOTICE_US: &str = "Source: U.S. Census Bureau, 2020-2024 American Community Survey 5-Year Estimates, Tables B25008 and B25009. A derived work; not endorsed or certified by the Census Bureau.";

/// Germany, classes 1..=5 (village, suburban, terraced, apartment blocks, dense fabric):
/// shares of households in parts per million at sizes 1, 2, 3, 4, 5 and 6-or-more, fitted
/// on ALL cells (`conformance/scene/tables/housegen1/rows.json`). The village row's last
/// three shares come from columns that failed the disclosure check.
pub const DE_PPM: [[u64; 6]; 5] = [
    [323_365, 352_902, 152_510, 113_690, 37_753, 19_780],
    [359_185, 341_645, 142_815, 107_542, 33_115, 15_698],
    [455_845, 297_434, 117_192, 85_793, 28_513, 15_223],
    [563_116, 239_882, 95_242, 64_333, 22_981, 14_446],
    [615_464, 210_298, 86_439, 55_289, 18_435, 14_075],
];
/// The top band is generated as 7 (its residual mean in the census is 7.4).
pub const DE_SIZES: [u8; 6] = [1, 2, 3, 4, 5, 7];

/// United States, the same five classes BY CENSUS TRACT: sizes 1..6 and 7-or-more
/// (`conformance/scene/tables/household0_us/household_table.json`, tract key).
pub const US_PPM: [[u64; 7]; 5] = [
    [255_238, 377_663, 149_835, 124_017, 57_864, 21_849, 13_534],
    [285_493, 340_975, 154_633, 128_882, 57_497, 20_756, 11_763],
    [299_563, 306_221, 158_932, 129_482, 62_651, 25_455, 17_696],
    [344_038, 283_416, 148_764, 113_994, 60_032, 26_560, 23_197],
    [381_954, 296_377, 141_412, 97_934, 45_544, 19_845, 16_934],
];
/// The top band is generated as 8, a declared value; the survey gives it no usable mean.
pub const US_SIZES: [u8; 7] = [1, 2, 3, 4, 5, 6, 8];

/// Below this many people the draw's own bias was not measured (HOUSEGEN-1, S2).
pub const UNIT_FLOOR: u64 = 10;

/// One consumer's verdict for a call. It is a statement about the rule over many units of
/// the class, never about one draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Call {
    pub verdict: Verdict,
    pub reason: &'static str,
}

/// What a call returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Households {
    /// One entry per household; they sum to the people passed.
    pub sizes: Vec<u8>,
    /// Consumer: households per unit (the mean household size).
    pub per_unit: Call,
    /// Consumer: household composition (the share of one-person households).
    pub composition: Call,
    pub notice: &'static str,
}

fn row(class: u8, reference: Reference) -> (&'static [u64], &'static [u8]) {
    assert!((1..=5).contains(&class), "class 1..=5");
    let c = class as usize - 1;
    match reference {
        Reference::Germany2022 => (&DE_PPM[c], &DE_SIZES),
        Reference::UnitedStates2024 => (&US_PPM[c], &US_SIZES),
    }
}

/// The two verdicts HOUSEGEN-1 issued, by reference and class (`HOUSEGEN1_RESULTS.md`).
pub fn verdicts(people: u64, class: u8, reference: Reference, in_country: bool) -> (Call, Call) {
    let both = |verdict, reason| (Call { verdict, reason }, Call { verdict, reason });
    match reference {
        Reference::UnitedStates2024 => both(Verdict::Unresolved, "key is a census tract; not shown to transfer to a tile"),
        Reference::Germany2022 if !in_country => both(Verdict::Unresolved, "outside the country read"),
        Reference::Germany2022 if people < UNIT_FLOOR => both(Verdict::Unresolved, "below the unit floor"),
        Reference::Germany2022 => match class {
            1 => (
                Call { verdict: Verdict::Unresolved, reason: "held-out regions differ by more than the budget resolves" },
                Call { verdict: Verdict::Certified, reason: "rows fitted on three bands, scored on the tiles of the other three, within budget; a statement about the rule over many units, not one draw" },
            ),
            2 => both(Verdict::Unresolved, "held-out regions differ by more than the budget resolves"),
            _ => both(Verdict::Unresolved, "read, not certified: too few independent regions"),
        },
    }
}

/// Households in a 1 km tile of `class` (1..=5) holding `people`. The sizes sum to `people`
/// for every seed; the verdicts say what HOUSEGEN-1 found for this reference and class.
pub fn generate(seed: u64, people: u64, class: u8, reference: Reference, in_country: bool) -> Households {
    let (shares, sizes) = row(class, reference);
    let (per_unit, composition) = verdicts(people, class, reference, in_country);
    let notice = match reference {
        Reference::Germany2022 => NOTICE_DE,
        Reference::UnitedStates2024 => NOTICE_US,
    };
    Households { sizes: draw(seed, people, shares, sizes), per_unit, composition, notice }
}

/// The number of households the row expects for `people`, rounded half up: an integer a
/// consumer may apportion from above before any unit is generated.
pub fn expected_households(people: u64, class: u8, reference: Reference) -> u64 {
    let (shares, sizes) = row(class, reference);
    let total: u128 = shares.iter().map(|&x| x as u128).sum();
    let mean_num: u128 = shares.iter().zip(sizes).map(|(&w, &s)| w as u128 * s as u128).sum();
    ((2 * people as u128 * total + mean_num) / (2 * mean_num)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROW: [u64; 6] = [434_010, 305_330, 124_260, 91_074, 29_674, 15_652];
    const SIZES: [u8; 6] = [1, 2, 3, 4, 5, 7];

    fn sum(v: &[u8]) -> u64 {
        v.iter().map(|&s| s as u64).sum()
    }

    /// G0: the section law, 100,000 calls, people from 0 to 2,000.
    #[test]
    fn sizes_sum_to_people_for_every_seed() {
        let mut st = Stream::new(20261111);
        for i in 0..100_000u64 {
            let people = st.below(2001);
            let v = draw(mix(i), people, &ROW, &SIZES);
            assert_eq!(sum(&v), people);
            assert!(v.iter().all(|&s| s > 0));
        }
    }

    /// P-ZERO.
    #[test]
    fn nobody_gives_no_household() {
        assert!(draw(7, 0, &ROW, &SIZES).is_empty());
    }

    /// P-ONES.
    #[test]
    fn a_row_of_ones_gives_one_household_each() {
        for n in [1u64, 2, 17, 300] {
            let v = draw(n, n, &[1_000_000], &[1]);
            assert_eq!(v.len() as u64, n);
            assert!(v.iter().all(|&s| s == 1));
        }
    }

    /// P-FOURS: units of 10 from a row of fours give two or three households, and both occur.
    #[test]
    fn a_row_of_fours_in_units_of_ten() {
        let (mut two, mut three) = (0, 0);
        for seed in 0..1000u64 {
            let v = draw(mix(seed), 10, &[1_000_000], &[4]);
            assert_eq!(sum(&v), 10);
            match v.len() {
                2 => two += 1,
                3 => {
                    three += 1;
                    assert_eq!(v, vec![4, 4, 2]);
                }
                n => panic!("{n} households"),
            }
        }
        assert!(two > 300 && three > 300, "{two} {three}");
    }

    /// P-SEED.
    #[test]
    fn one_seed_one_scene() {
        assert_eq!(draw(99, 300, &ROW, &SIZES), draw(99, 300, &ROW, &SIZES));
        let differ = (0..1000u64).filter(|&i| draw(mix(i), 300, &ROW, &SIZES) != draw(mix(i + 5000), 300, &ROW, &SIZES)).count();
        assert!(differ >= 990, "{differ}");
    }

    /// P-BROKEN: without the last step the law fails, so the test above is not vacuous.
    #[test]
    fn without_the_last_step_the_law_fails() {
        fn broken(seed: u64, people: u64) -> Vec<u8> {
            let total: u64 = ROW.iter().sum();
            let mut st = Stream::new(seed);
            let (mut out, mut left) = (Vec::new(), people);
            while left > 0 {
                let s = SIZES[pick(&mut st, &ROW, total)] as u64;
                out.push(s as u8);
                left = left.saturating_sub(s);
            }
            out
        }
        let bad = (0..1000u64).filter(|&i| sum(&broken(mix(i), 150)) != 150).count();
        assert!(bad > 100, "{bad}");
    }

    /// The plant draw also obeys the law; it is its bias that S2's scoring must see.
    #[test]
    fn the_truncating_plant_obeys_the_law_and_adds_households() {
        let (mut a, mut b) = (0usize, 0usize);
        for i in 0..5000u64 {
            let t = draw_truncating(mix(i), 8, &ROW, &SIZES);
            assert_eq!(sum(&t), 8);
            a += t.len();
            b += draw(mix(i), 8, &ROW, &SIZES).len();
        }
        assert!(a > b, "{a} {b}");
    }

    #[test]
    fn the_digest_is_stable() {
        assert_eq!(digest(&ROW, &SIZES), digest(&ROW, &SIZES));
    }
    #[test]
    fn rows_are_whole() {
        for r in DE_PPM.iter() {
            assert_eq!(r.iter().sum::<u64>(), 1_000_000);
        }
        for r in US_PPM.iter() {
            assert!((r.iter().sum::<u64>() as i64 - 1_000_000).abs() <= 1);
        }
    }

    #[test]
    fn generate_obeys_the_law_for_each_reference_and_class() {
        for reference in [Reference::Germany2022, Reference::UnitedStates2024] {
            for class in 1..=5u8 {
                for i in 0..2000u64 {
                    let people = (i * 13) % 1500;
                    let h = generate(mix(i), people, class, reference, true);
                    assert_eq!(sum(&h.sizes), people);
                }
            }
        }
    }

    #[test]
    fn only_what_was_certified_is_certified() {
        let mut certified = 0;
        for reference in [Reference::Germany2022, Reference::UnitedStates2024] {
            for class in 1..=5u8 {
                for in_country in [true, false] {
                    for people in [0u64, 9, 10, 500] {
                        let (a, b) = verdicts(people, class, reference, in_country);
                        for (which, c) in [(0, a), (1, b)] {
                            assert_ne!(c.verdict, Verdict::Refused);
                            if c.verdict == Verdict::Certified {
                                certified += 1;
                                assert!(reference == Reference::Germany2022 && in_country && class == 1 && which == 1 && people >= UNIT_FLOOR);
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(certified, 2);
        assert_eq!(generate(1, 500, 1, Reference::UnitedStates2024, true).composition.verdict, Verdict::Unresolved);
    }

    /// The expected count is what the draw gives on average, to the draw's own small bias.
    #[test]
    fn the_expected_count_matches_the_draw() {
        for class in 1..=5u8 {
            let people = 2000u64;
            let n: u64 = (0..400u64).map(|i| generate(mix(i), people, class, Reference::Germany2022, true).sizes.len() as u64).sum();
            let want = 400 * expected_households(people, class, Reference::Germany2022);
            let off = (n as f64 / want as f64 - 1.0).abs();
            assert!(off < 0.005, "class {class}: {n} against {want}");
        }
        assert_eq!(expected_households(0, 3, Reference::Germany2022), 0);
    }
}
