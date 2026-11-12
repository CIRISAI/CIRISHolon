//! BUILDGEN-1: a dwelling drawn from the joint of what it is and what it uses.
//!
//! The order of the draw (`conformance/scene/BUILDGEN1_PREREG.md`): type; a normal score for
//! vintage and its band; heating fuel, wall material, roof material and split-level from
//! tables given type, zone and era; then a rank copula, continuing from the vintage score,
//! over storeys (houses alone), floor area, electricity and the positive part of non-electric
//! energy. Total energy is electricity's 3.412 thousand Btu per kWh plus non-electric.
//!
//! Integers only. Each table row's first value is the pooling level it was fitted at.
use crate::normal_table::NORMAL_Q16;
use crate::seed::Stream;

pub const NOT_APPLICABLE: u8 = 255;
pub const TYPES: usize = 5; // 0 mobile home, 1 detached, 2 attached, 3 apartment in 2 to 4, 4 apartment in 5 or more
pub const KNOTS: usize = 33;

pub const T_TYPE: u32 = 1;
pub const T_VINTAGE: u32 = 2;
pub const T_FUEL: u32 = 3;
pub const T_WALL: u32 = 4;
pub const T_ROOF: u32 = 5;
pub const T_SPLIT: u32 = 6;
pub const T_STOREYS: u32 = 7;
pub const T_AREA: u32 = 8;
pub const T_KWH: u32 = 9;
pub const T_NONELEC: u32 = 10;
pub const T_ZERO: u32 = 11;
pub const T_CHOL: u32 = 12;
pub const T_ENDUSE: u32 = 13;

/// The key of one row: a table and up to four small indices (a < 8, b < 16, c < 8, d < 8).
pub fn key(table: u32, a: u32, b: u32, c: u32, d: u32) -> u32 {
    debug_assert!(a < 8 && b < 16 && c < 8 && d < 8);
    (((table * 8 + a) * 16 + b) * 8 + c) * 8 + d
}

pub struct Tables<'a> {
    pub rows: &'a [(u32, &'a [u32])],
}

impl<'a> Tables<'a> {
    /// The row's pooling level and its values.
    fn get(&self, k: u32) -> (u8, &'a [u32]) {
        let i = self.rows.binary_search_by_key(&k, |r| r.0).expect("a table row is missing");
        let r = self.rows[i].1;
        (r[0] as u8, &r[1..])
    }
}

/// The era of a vintage band (0 to 8): before 1980, 1980 to 1999, 2000 to 2020.
pub fn era(band: u8) -> u8 {
    match band {
        0..=3 => 0,
        4..=5 => 1,
        _ => 2,
    }
}

/// The fuel group of a heating fuel (0 electricity, 1 natural gas, 2 propane, 3 fuel oil,
/// 4 wood, 5 other, 6 none): 0 electricity, 1 gas, 2 other, 3 no heating.
pub fn fuel_group(fuel: u8) -> u8 {
    match fuel {
        0 => 0,
        1 => 1,
        6 => 3,
        _ => 2,
    }
}

pub fn is_house(kind: u8) -> bool {
    kind == 1 || kind == 2
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

/// The category whose cumulative share holds the 32-bit uniform `u`.
fn pick_by_u(shares: &[u32], u: u32) -> usize {
    let total: u64 = shares.iter().map(|&x| x as u64).sum();
    if total == 0 {
        return 0;
    }
    let target = ((u as u64) * total) >> 32;
    let mut acc = 0u64;
    for (i, &w) in shares.iter().enumerate() {
        acc += w as u64;
        if target < acc {
            return i;
        }
    }
    shares.len() - 1
}

/// The normal score of a 32-bit uniform, in 16-bit fixed point.
pub fn normal_of(u: u32) -> i32 {
    let i = (u >> 22) as usize;
    let f = (u & 0x3F_FFFF) as i64;
    let (a, b) = (NORMAL_Q16[i] as i64, NORMAL_Q16[i + 1] as i64);
    (a + (((b - a) * f) >> 22)) as i32
}

/// The 32-bit uniform of a normal score: the same table, inverted.
pub fn uniform_of(z: i32) -> u32 {
    let z = z.clamp(NORMAL_Q16[0], NORMAL_Q16[1024]);
    if z >= NORMAL_Q16[1024] {
        return u32::MAX;
    }
    let (mut lo, mut hi) = (0usize, 1024usize);
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if NORMAL_Q16[mid] <= z {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let (a, b) = (NORMAL_Q16[lo] as i64, NORMAL_Q16[lo + 1] as i64);
    let f = (((z as i64 - a) << 22) / (b - a)).min(0x3F_FFFF);
    ((lo as u32) << 22) | f as u32
}

/// A value at the 32-bit uniform `u` on `KNOTS` quantile knots, linear between knots.
fn quantile(knots: &[u32], u: u32) -> u32 {
    let n = (knots.len() - 1) as u64;
    let pos = (u as u64) * n; // 32.32 fixed point over the knot index
    let i = (pos >> 32) as usize;
    let f = pos & 0xFFFF_FFFF;
    let (a, b) = (knots[i] as u64, knots[(i + 1).min(knots.len() - 1)] as u64);
    if b >= a {
        (a + (((b - a) * f) >> 32)) as u32
    } else {
        a as u32
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dwelling {
    /// 0 mobile home, 1 detached house, 2 attached house, 3 apartment in a building of 2 to 4
    /// units, 4 apartment in a building of 5 or more.
    pub kind: u8,
    /// 0 before 1950, then the decades to 6 the 2000s, 7 2010 to 2015, 8 2016 to 2020.
    pub vintage: u8,
    /// 0 electricity, 1 natural gas, 2 propane, 3 fuel oil, 4 wood or pellets, 5 other, 6 none.
    pub fuel: u8,
    /// 0 brick, 1 wood, 2 siding, 3 stucco, 4 shingle, 5 stone, 6 concrete block, 7 other.
    pub wall: u8,
    /// 0 ceramic or clay tiles, 1 wood shingles, 2 metal, 3 slate, 4 composition shingles,
    /// 5 concrete tiles, 6 other; `NOT_APPLICABLE` for an apartment in a building of 5 or more.
    pub roof: u8,
    /// 1, 2 or 3 (3 or more); 0 with `NOT_APPLICABLE` semantics for a dwelling that is not a house.
    pub storeys: u8,
    /// 1 split-level, 0 not; `NOT_APPLICABLE` for a dwelling that is not a house.
    pub split_level: u8,
    /// Energy-consuming floor area in square feet.
    pub area_sqft: u32,
    pub kwh: u32,
    /// Thousand Btu in 2020 of fuels other than electricity.
    pub nonelec_kbtu: u32,
    pub total_kbtu: u32,
    pub heating_kbtu: u32,
    pub cooling_kbtu: u32,
    /// Pooling levels: type, vintage, fuel, wall, roof, storeys, area, electricity, non-electric.
    pub levels: [u8; 9],
}

impl Dwelling {
    /// The roof's tone, ASSUMED from its material (no source gives colour): 0 red tile,
    /// 1 brown tile, 2 weathered wood, 3 grey slate, 4 dark shingle, 5 light metal,
    /// 6 flat concrete, 7 neutral.
    pub fn roof_tone_assumed(&self) -> u8 {
        match self.roof {
            0 => 0,
            5 => 1,
            1 => 2,
            3 => 3,
            4 => 4,
            2 => 5,
            NOT_APPLICABLE => 6,
            _ => 7,
        }
    }
    /// Units in the building as the type gives them: 0 one, 1 one attached, 2 two to four,
    /// 3 five or more. Counts and floors of an apartment building are not in the survey.
    pub fn units_in_building(&self) -> u8 {
        match self.kind {
            0 | 1 => 0,
            2 => 1,
            3 => 2,
            _ => 3,
        }
    }
}

/// Generate one dwelling. `zone` 0..8 is the survey's climate zone, `urban` 0..3 its urban type.
pub fn generate_with(t: &Tables, seed: u64, zone: u8, urban: u8) -> Dwelling {
    let (zone, urban) = ((zone as u32).min(7), (urban as u32).min(2));
    let mut st = Stream::new(seed);
    let mut levels = [0u8; 9];
    // 1. type
    let (lv, sh) = t.get(key(T_TYPE, 0, zone, urban, 0));
    levels[0] = lv;
    let kind = pick(&mut st, sh) as u32;
    let house = is_house(kind as u8);
    // 2. the vintage score and its band
    let dim = if house { 5 } else { 4 };
    let mut z = [0i64; 5];
    z[0] = normal_of((st.next_u64() >> 32) as u32) as i64;
    let (lv, sh) = t.get(key(T_VINTAGE, kind, zone, 0, 0));
    levels[1] = lv;
    let vintage = pick_by_u(sh, uniform_of(z[0] as i32)) as u8;
    let e = era(vintage) as u32;
    // 3. the chain
    let (lv, sh) = t.get(key(T_FUEL, kind, zone, e, 0));
    levels[2] = lv;
    let fuel = pick(&mut st, sh) as u8;
    let fg = fuel_group(fuel) as u32;
    let (lv, sh) = t.get(key(T_WALL, kind, zone, e, 0));
    levels[3] = lv;
    let wall = pick(&mut st, sh) as u8;
    let roof = if kind == 4 {
        NOT_APPLICABLE
    } else {
        let (lv, sh) = t.get(key(T_ROOF, wall as u32, zone, e, 0));
        levels[4] = lv;
        pick(&mut st, sh) as u8
    };
    let split_level = if house { pick(&mut st, t.get(key(T_SPLIT, kind, 0, e, 0)).1) as u8 } else { NOT_APPLICABLE };
    // 4. the copula: correlated scores from independent ones by the type's lower-triangular factor
    for zi in z.iter_mut().take(dim).skip(1) {
        *zi = normal_of((st.next_u64() >> 32) as u32) as i64;
    }
    let chol = t.get(key(T_CHOL, kind, 0, 0, 0)).1;
    let mut u = [0u32; 5];
    let mut at = 0usize;
    for i in 0..dim {
        let mut y = 0i64;
        for zj in z.iter().take(i + 1) {
            y += ((chol[at] as i32 as i64) * *zj) >> 16;
            at += 1;
        }
        u[i] = uniform_of(y.clamp(i32::MIN as i64, i32::MAX as i64) as i32);
    }
    let mut next = 1usize;
    let storeys = if house {
        let (lv, sh) = t.get(key(T_STOREYS, kind, zone, 0, 0));
        levels[5] = lv;
        next += 1;
        pick_by_u(sh, u[1]) as u8 + 1
    } else {
        0
    };
    let (lv, kn) = t.get(key(T_AREA, kind, zone, 0, 0));
    levels[6] = lv;
    let area_sqft = quantile(kn, u[next]);
    let (lv, kn) = t.get(key(T_KWH, kind, zone, fg, 0));
    levels[7] = lv;
    let kwh = quantile(kn, u[next + 1]);
    // 5. non-electric energy: zero by its own table, else its positive part from the copula
    let (lv, kn) = t.get(key(T_NONELEC, kind, zone, fg, 0));
    levels[8] = lv;
    let zero = pick(&mut st, t.get(key(T_ZERO, kind, zone, fg, 0)).1) == 0;
    let nonelec_kbtu = if zero { 0 } else { quantile(kn, u[next + 2]) };
    let elec_kbtu = ((kwh as u64 * 3412 + 500) / 1000) as u32;
    let total_kbtu = elec_kbtu + nonelec_kbtu;
    // 6. end uses, outside the copula
    let eu = t.get(key(T_ENDUSE, kind, zone, fg, 0)).1;
    let heating_kbtu = ((total_kbtu as u64 * eu[0].min(1_000_000) as u64) / 1_000_000) as u32;
    let cooling_kbtu = (((total_kbtu as u64 * eu[1].min(1_000_000) as u64) / 1_000_000) as u32).min(elec_kbtu).min(total_kbtu - heating_kbtu);
    Dwelling { kind: kind as u8, vintage, fuel, wall, roof, storeys, split_level, area_sqft, kwh, nonelec_kbtu, total_kbtu, heating_kbtu, cooling_kbtu, levels }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_normal_table_inverts_itself() {
        // a score comes back to within one unit of its 16-bit resolution; a uniform to within the
        // width that one unit of score spans at the middle of the table (about 2^15)
        for &u in &[0u32, 1, 1 << 22, 0x8000_0000, 0xC123_4567, u32::MAX - 5, u32::MAX] {
            let z = normal_of(u);
            assert!((normal_of(uniform_of(z)) - z).abs() <= 1, "score {z} from {u}");
            assert!((uniform_of(z) as i64 - u as i64).abs() <= 1 << 15, "{u} came back as {}", uniform_of(z));
        }
        assert_eq!(normal_of(0x8000_0000), 0);
        assert!(normal_of(0) == NORMAL_Q16[0] && normal_of(u32::MAX) < NORMAL_Q16[1024]);
        assert!((1..=1024).all(|i| NORMAL_Q16[i] > NORMAL_Q16[i - 1]));
    }

    #[test]
    fn quantile_runs_from_the_first_knot_to_the_last() {
        let k: Vec<u32> = (0..KNOTS as u32).map(|i| 100 + 10 * i).collect();
        assert_eq!(quantile(&k, 0), 100);
        assert_eq!(quantile(&k, u32::MAX), 100 + 10 * 32 - 1);
        assert_eq!(quantile(&k, 0x8000_0000), 100 + 10 * 16);
    }

    /// A small planted world: every key present, houses with storeys, big apartments with no roof.
    fn planted() -> Vec<(u32, Vec<u32>)> {
        let mut rows = Vec::new();
        let knots = |lo: u32, step: u32| -> Vec<u32> { std::iter::once(0).chain((0..KNOTS as u32).map(|i| lo + step * i)).collect() };
        for zone in 0..8 {
            for urban in 0..3 {
                rows.push((key(T_TYPE, 0, zone, urban, 0), vec![0, 1, 5, 2, 1, 3]));
            }
            for kind in 0..5 {
                rows.push((key(T_VINTAGE, kind, zone, 0, 0), vec![0, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
                rows.push((key(T_STOREYS, kind, zone, 0, 0), vec![0, 5, 4, 1]));
                rows.push((key(T_AREA, kind, zone, 0, 0), knots(500, 60)));
                for e in 0..3 {
                    rows.push((key(T_FUEL, kind, zone, e, 0), vec![0, 3, 4, 1, 1, 1, 0, 1]));
                    rows.push((key(T_WALL, kind, zone, e, 0), vec![0, 2, 2, 3, 1, 1, 1, 1, 1]));
                }
                for fg in 0..4 {
                    rows.push((key(T_KWH, kind, zone, fg, 0), knots(2000, 400)));
                    rows.push((key(T_NONELEC, kind, zone, fg, 0), knots(5000, 2500)));
                    rows.push((key(T_ZERO, kind, zone, fg, 0), vec![0, if fg == 0 { 9 } else { 1 }, if fg == 0 { 1 } else { 9 }]));
                    rows.push((key(T_ENDUSE, kind, zone, fg, 0), vec![0, 400_000, 100_000]));
                }
            }
            for wall in 0..8 {
                for e in 0..3 {
                    rows.push((key(T_ROOF, wall, zone, e, 0), vec![0, 1, 1, 2, 1, 6, 1, 1]));
                }
            }
        }
        for kind in 0..5u32 {
            for e in 0..3 {
                rows.push((key(T_SPLIT, kind, 0, e, 0), vec![0, 9, 1]));
            }
            let d = if is_house(kind as u8) { 5 } else { 4 };
            let mut c = vec![0u32];
            for i in 0..d {
                for j in 0..=i {
                    c.push(if i == j { 65536 } else { 0 });
                }
            }
            rows.push((key(T_CHOL, kind, 0, 0, 0), c));
        }
        rows.sort_by_key(|r| r.0);
        rows
    }

    #[test]
    fn the_section_law_and_what_is_not_applicable() {
        let owned = planted();
        let rows: Vec<(u32, &[u32])> = owned.iter().map(|r| (r.0, r.1.as_slice())).collect();
        let t = Tables { rows: &rows };
        let mut kinds = [0u32; 5];
        for i in 0..20_000u64 {
            let d = generate_with(&t, crate::seed::mix(i), (i % 8) as u8, (i % 3) as u8);
            kinds[d.kind as usize] += 1;
            assert!(d.total_kbtu as u64 * 1000 + 500 >= d.kwh as u64 * 3412, "total is at least electricity");
            assert!(d.heating_kbtu + d.cooling_kbtu <= d.total_kbtu);
            assert_eq!(is_house(d.kind), d.storeys != 0);
            assert_eq!(is_house(d.kind), d.split_level != NOT_APPLICABLE);
            assert_eq!(d.kind == 4, d.roof == NOT_APPLICABLE);
            assert!(d.vintage < 9 && d.wall < 8 && d.fuel < 7);
            assert_eq!(generate_with(&t, crate::seed::mix(i), (i % 8) as u8, (i % 3) as u8), d);
        }
        assert!(kinds.iter().all(|&k| k > 500), "each type is drawn: {kinds:?}");
    }
}
