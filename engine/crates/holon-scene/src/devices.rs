//! Who lives in a household of a given size, and the devices it holds (DEVICEGEN-1,
//! `conformance/scene/DEVICEGEN1_PREREG.md`).
//!
//! One of 12 cells (connection x any computer x any speaker or streaming device) is drawn
//! for the household from one table, so those three are drawn TOGETHER; smartphones are
//! drawn per person; every other kind per household from a histogram given the cell.
//! The section law: persons by group sum to the household's size, for every seed
//! (`ScenePopulation.pop_total`). Integers only.

use crate::seed::Stream;
use crate::verdict::Verdict;

/// Kinds held per household, in the order of `HouseholdDevices::counts`.
pub const KINDS: [&str; 8] = ["laptop", "desktop", "tablet", "television", "set-top box", "streaming device", "games console", "smart speaker"];

pub const BUILDING_HOUSE: u8 = 0;
pub const BUILDING_APARTMENT: u8 = 1;
pub const BUILDING_UNKNOWN: u8 = 2;

const ANY: u32 = 3;

/// The key of one table row. `table`: 0 composition, 1 cells, 2..=9 the kinds, 10 television
/// hours, 11 weekdays someone is home, 12 smartphones per person.
pub fn key(table: u32, size: u32, cell: u32, building: u32, kids: u32) -> u32 {
    (((table * 8 + size) * 32 + cell) * 4 + building) * 4 + kids
}

/// Tables as sorted (key, values) rows. Values are shares in parts per million, except the
/// composition table, whose values alternate a code (adults * 100 + seniors * 10 +
/// children) and its share.
pub struct Tables<'a> {
    pub rows: &'a [(u32, &'a [u32])],
}

impl<'a> Tables<'a> {
    fn get(&self, k: u32) -> Option<&'a [u32]> {
        self.rows.binary_search_by_key(&k, |r| r.0).ok().map(|i| self.rows[i].1)
    }
}

fn pick(st: &mut Stream, shares: &[u32]) -> usize {
    let total: u64 = shares.iter().map(|&x| x as u64).sum();
    if total == 0 {
        return 0;
    }
    let mut u = st.below(total);
    for (i, &w) in shares.iter().enumerate() {
        if u < w as u64 {
            return i;
        }
        u -= w as u64;
    }
    shares.len() - 1
}

/// What a call returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseholdDevices {
    pub adults: u8,
    pub seniors: u8,
    pub children: u8,
    /// The household's size was above the survey's top code; the extra persons are adults.
    pub above_top_code: bool,
    /// Smartphones held by adults 18 to 64, adults over 64, and children.
    pub phones: [u8; 3],
    /// Counts in the order of `KINDS`.
    pub counts: [u8; 8],
    /// 0 a fixed line, 1 mobile data only, 2 no internet at home.
    pub connection: u8,
    pub tv_hours: u8,
    pub weekdays_someone_home: u8,
    /// Which fallback level each kind's histogram came from (0 the full stratum).
    pub fallback: [u8; 8],
}

impl HouseholdDevices {
    /// A gateway is DECLARED present with a fixed line. Whether it can host is unknown.
    pub fn gateway_declared(&self) -> bool {
        self.connection == 0
    }
    /// A desktop can host (declared). Whether it is on all day is unknown.
    pub fn has_desktop(&self) -> bool {
        self.counts[1] > 0
    }
    pub fn persons(&self) -> u32 {
        self.adults as u32 + self.seniors as u32 + self.children as u32
    }
}

fn size_band(persons: u32) -> u32 {
    persons.clamp(1, 5)
}

/// Generate one household from `tables`. `building` is 0 house, 1 apartment, 2 unknown.
pub fn generate_with(tables: &Tables, seed: u64, persons: u32, building: u8) -> HouseholdDevices {
    let mut out = HouseholdDevices {
        adults: 0, seniors: 0, children: 0, above_top_code: false, phones: [0; 3], counts: [0; 8],
        connection: 2, tv_hours: 0, weekdays_someone_home: 0, fallback: [0; 8],
    };
    if persons == 0 {
        return out;
    }
    let mut st = Stream::new(seed);
    let b = (building as u32).min(2);
    let b = if b == 2 { ANY } else { b };
    // 1. who lives there
    let s7 = persons.min(7);
    let comp = tables.get(key(0, s7, 0, b, 0)).or_else(|| tables.get(key(0, s7, 0, ANY, 0))).expect("composition row");
    let shares: Vec<u32> = comp.iter().skip(1).step_by(2).copied().collect();
    let code = comp[2 * pick(&mut st, &shares)];
    let (a, o, c) = (code / 100, (code / 10) % 10, code % 10);
    let extra = persons - s7;
    out.adults = (a + extra).min(255) as u8;
    out.seniors = o as u8;
    out.children = c as u8;
    out.above_top_code = extra > 0;
    if a + o + c != s7 {
        // a top-coded row may sum below the size: the remainder are adults
        out.adults = (out.adults as u32 + (s7 - (a + o + c).min(s7))).min(255) as u8;
    }
    // 2. the cell
    let s5 = size_band(persons);
    let cells = tables.get(key(1, s5, 0, b, 0)).or_else(|| tables.get(key(1, s5, 0, ANY, 0))).expect("cell row");
    let cell = pick(&mut st, cells) as u32;
    let (conn, comp_any, spk_any) = (cell / 4, (cell / 2) % 2, cell % 2);
    out.connection = conn as u8;
    // 3. smartphones per person
    for (g, n) in [(0u32, out.adults), (1, out.seniors), (2, out.children)] {
        if let Some(p) = tables.get(key(12, g, conn, 0, 0)) {
            for _ in 0..n {
                out.phones[g as usize] = out.phones[g as usize].saturating_add(pick(&mut st, p) as u8);
            }
        }
    }
    // 4. the other kinds
    let kids = if out.children > 0 { 1 } else { 0 };
    for k in 0..8u32 {
        let levels = [
            key(2 + k, s5, cell, b, kids),
            key(2 + k, s5, cell, b, ANY),
            key(2 + k, s5, cell, ANY, ANY),
            key(2 + k, s5, 12 + conn * 2 + comp_any, ANY, ANY),
            key(2 + k, s5, 18, ANY, ANY),
        ];
        for (lvl, kk) in levels.iter().enumerate() {
            if let Some(h) = tables.get(*kk) {
                out.counts[k as usize] = pick(&mut st, h) as u8;
                out.fallback[k as usize] = lvl as u8;
                break;
            }
        }
    }
    // consistency with the cell
    if comp_any == 0 {
        out.counts[0] = 0;
        out.counts[1] = 0;
        out.counts[2] = 0;
    } else if out.counts[0] + out.counts[1] + out.counts[2] == 0 {
        out.counts[0] = 1;
    }
    if spk_any == 0 {
        out.counts[5] = 0;
        out.counts[7] = 0;
    } else if out.counts[5] + out.counts[7] == 0 {
        out.counts[5] = 1;
    }
    // 5. hours
    if let Some(h) = tables.get(key(10, s5, 0, 0, 0)) {
        out.tv_hours = if out.counts[3] > 0 { pick(&mut st, h) as u8 } else { 0 };
    }
    if let Some(h) = tables.get(key(11, s5, 0, 0, 0)) {
        out.weekdays_someone_home = pick(&mut st, h) as u8;
    }
    out
}

/// One consumer's verdict for a call: a statement about the rule over many households.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Call {
    pub verdict: Verdict,
    pub reason: &'static str,
}

/// The notice that must travel with anything produced from the compiled-in tables.
pub const NOTICE_US: &str = "Source: U.S. Energy Information Administration, Residential Energy Consumption Survey 2020 public-use microdata. A derived work; not endorsed by the agency.";

/// The compiled-in tables: United States, 2020, fitted on all records.
pub fn us_2020() -> Tables<'static> {
    Tables { rows: crate::devices_tables::ROWS }
}

/// Generate a household from the compiled-in tables, with the two verdicts DEVICEGEN-1
/// issued (`conformance/scene/DEVICEGEN1_RESULTS.md`): counts per kind, and whether the
/// household has nothing to host on. Each is a statement about the rule over many
/// households in the United States in 2020, never about one draw.
pub fn generate(seed: u64, persons: u32, building: u8, in_country: bool) -> (HouseholdDevices, Call, Call) {
    let h = generate_with(&us_2020(), seed, persons, building);
    if !in_country {
        let c = Call { verdict: Verdict::Unresolved, reason: "outside the country read (United States, 2020)" };
        return (h, c, c);
    }
    (
        h,
        Call { verdict: Verdict::Unresolved, reason: "2020: each kind certified pooled over household sizes; by size 32 of 45 cells certified, 12 unresolved, 1 refused; by 2024 set-top boxes had fallen 40 % and streaming devices risen 33 %" },
        Call { verdict: Verdict::Certified, reason: "2020: the share of households with no computer and no fixed line, and its clustering, reproduced on states never seen and not carried by a rival that draws each kind alone" },
    )
}

/// One number for 1,000 fixed calls on the compiled-in tables, to compare builds.
pub fn digest() -> u64 {
    use crate::seed::mix;
    let t = us_2020();
    let mut d = 0u64;
    for i in 0..1000u64 {
        let h = generate_with(&t, mix(i), (i % 9) as u32, (i % 3) as u8);
        d = mix(d ^ h.persons() as u64);
        for x in h.counts.iter().chain(h.phones.iter()) {
            d = mix(d ^ *x as u64);
        }
        d = mix(d ^ h.connection as u64);
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::mix;

    fn toy() -> Vec<(u32, Vec<u32>)> {
        let mut rows: Vec<(u32, Vec<u32>)> = Vec::new();
        for s in 1..=7u32 {
            // everyone an adult, or (for s >= 2) one child
            let mut v = vec![s * 100, 600_000];
            if s >= 2 {
                v.extend([(s - 1) * 100 + 1, 400_000]);
            } else {
                v[1] = 1_000_000;
            }
            rows.push((key(0, s, 0, ANY, 0), v));
        }
        for s in 1..=5u32 {
            // cells: half "fixed line, computer, speaker" (cell 3), half "none, no computer, no speaker" (cell 8)
            let mut c = vec![0u32; 12];
            c[3] = 500_000;
            c[8] = 500_000;
            rows.push((key(1, s, 0, ANY, 0), c));
            for k in 0..8u32 {
                rows.push((key(2 + k, s, 18, ANY, ANY), vec![300_000, 400_000, 300_000, 0, 0, 0, 0, 0, 0]));
            }
            rows.push((key(10, s, 0, 0, 0), vec![0, 500_000, 500_000]));
            rows.push((key(11, s, 0, 0, 0), vec![100_000, 0, 0, 0, 0, 900_000]));
        }
        for g in 0..3u32 {
            for conn in 0..3u32 {
                rows.push((key(12, g, conn, 0, 0), vec![0, 0, 1_000_000]));
            }
        }
        rows.sort_by_key(|r| r.0);
        rows
    }

    fn with<R>(f: impl Fn(&Tables) -> R) -> R {
        let owned = toy();
        let rows: Vec<(u32, &[u32])> = owned.iter().map(|r| (r.0, r.1.as_slice())).collect();
        f(&Tables { rows: &rows })
    }

    /// P-PERSONS: persons by group sum to the size, and sizes above the top code are flagged.
    #[test]
    fn persons_sum_to_the_size() {
        with(|t| {
            for persons in 0..=9u32 {
                for i in 0..2000u64 {
                    let h = generate_with(t, mix(i), persons, BUILDING_UNKNOWN);
                    assert_eq!(h.persons(), persons);
                    assert_eq!(h.above_top_code, persons > 7);
                }
            }
        });
    }

    /// P-CONSISTENT: a "no computer" cell holds no laptop, desktop or tablet; a "computer" cell holds one.
    #[test]
    fn counts_agree_with_the_cell() {
        with(|t| {
            let (mut none, mut some) = (0, 0);
            for i in 0..4000u64 {
                let h = generate_with(t, mix(i), 3, BUILDING_HOUSE);
                let comp = h.counts[0] + h.counts[1] + h.counts[2];
                let spk = h.counts[5] + h.counts[7];
                if h.connection == 2 {
                    assert_eq!(comp, 0);
                    assert_eq!(spk, 0);
                    none += 1;
                } else {
                    assert!(comp > 0 && spk > 0);
                    some += 1;
                }
            }
            assert!(none > 1500 && some > 1500);
        });
    }

    /// P-TWOPHONES: persons who each hold two phones give more phones than persons.
    #[test]
    fn two_phones_each() {
        with(|t| {
            let h = generate_with(t, 5, 4, BUILDING_APARTMENT);
            let phones: u32 = h.phones.iter().map(|&x| x as u32).sum();
            assert_eq!(phones, 8);
        });
    }

    /// P-SEED.
    #[test]
    fn one_seed_one_household() {
        with(|t| {
            assert_eq!(generate_with(t, 77, 4, 0), generate_with(t, 77, 4, 0));
            let differ = (0..500u64).filter(|&i| generate_with(t, mix(i), 4, 0) != generate_with(t, mix(i + 9000), 4, 0)).count();
            assert!(differ > 400, "{differ}");
        });
    }

    /// P-THIN: a missing stratum falls back, and the level used is recorded.
    #[test]
    fn a_thin_stratum_falls_back() {
        with(|t| {
            let h = generate_with(t, 3, 2, BUILDING_HOUSE);
            assert!(h.fallback.iter().all(|&l| l == 4));
        });
    }

    #[test]
    fn nobody_holds_nothing() {
        with(|t| {
            let h = generate_with(t, 1, 0, 0);
            assert_eq!(h.persons(), 0);
            assert_eq!(h.counts, [0; 8]);
            assert_eq!(h.phones, [0; 3]);
        });
    }
    #[test]
    fn the_compiled_tables_obey_the_law() {
        let t = us_2020();
        assert!(t.rows.windows(2).all(|w| w[0].0 < w[1].0), "rows sorted by key");
        for persons in 0..=10u32 {
            for b in 0..3u8 {
                for i in 0..500u64 {
                    let h = generate_with(&t, mix(i), persons, b);
                    assert_eq!(h.persons(), persons);
                    let comp = h.counts[0] + h.counts[1] + h.counts[2];
                    assert!(h.connection <= 2 && h.tv_hours <= 24 && h.weekdays_someone_home <= 5);
                    let _ = comp;
                }
            }
        }
    }

    #[test]
    fn verdicts_are_as_issued() {
        let (_, counts, host) = generate(1, 3, BUILDING_HOUSE, true);
        assert_eq!(counts.verdict, Verdict::Unresolved);
        assert_eq!(host.verdict, Verdict::Certified);
        let (_, a, b) = generate(1, 3, BUILDING_HOUSE, false);
        assert_eq!(a.verdict, Verdict::Unresolved);
        assert_eq!(b.verdict, Verdict::Unresolved);
        assert_eq!(digest(), digest());
    }
}
