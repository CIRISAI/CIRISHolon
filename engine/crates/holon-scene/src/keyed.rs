//! BUILDGEN-2: a dwelling keyed by US STATE, beside BUILDGEN-1's `generate_with` (which is
//! pinned by `the_shipped_generator_is_pinned` and not changed).
//!
//! A PLACE is the nation (0), a state by its 2-digit federal code (1 to 56), or one of the
//! survey's 10 census divisions (60 to 69). A row is looked up at the state, then at its
//! division, then at the nation, and the first that exists is used: the fitting code writes
//! a state's row only where the state has households of its own to blend in
//! (`conformance/scene/BUILDGEN2_PREREG.md`). Each row begins with 2 weights in 1/255ths:
//! how much of the row is the state's own households, and how much of what is left is the
//! division's.
//!
//! Integers only. The draw order and the copula are BUILDGEN-1's; the climate zone has left
//! the key. A code that is not one of the 51 is REFUSED: a US survey says nothing of a
//! place outside the US.
use crate::buildings::{era, fuel_group, is_house, normal_of, pick, pick_by_u, quantile, uniform_of, Dwelling, NOT_APPLICABLE};
use crate::buildings::{T_AREA, T_CHOL, T_ENDUSE, T_FUEL, T_KWH, T_NONELEC, T_ROOF, T_SPLIT, T_STOREYS, T_TYPE, T_VINTAGE, T_WALL, T_ZERO};
use crate::seed::Stream;

pub const NATION: u32 = 0;
pub const DIVISION_BASE: u32 = 60;

/// The key of one row: a table, a place (< 128) and three small indices (a < 8, b < 16, c < 8).
pub fn key2(table: u32, place: u32, a: u32, b: u32, c: u32) -> u64 {
    debug_assert!(place < 128 && a < 8 && b < 16 && c < 8);
    ((((table as u64 * 128 + place as u64) * 8 + a as u64) * 16 + b as u64) * 8) + c as u64
}

/// The survey's division of a state, 0 to 9 in the alphabetical order of the divisions' names:
/// East North Central, East South Central, Middle Atlantic, Mountain North, Mountain South,
/// New England, Pacific, South Atlantic, West North Central, West South Central.
/// `None` for a code that is not a state or the District of Columbia.
pub fn division_of(state: u8) -> Option<u8> {
    Some(match state {
        17 | 18 | 26 | 39 | 55 => 0,
        1 | 21 | 28 | 47 => 1,
        34 | 36 | 42 => 2,
        8 | 16 | 30 | 49 | 56 => 3,
        4 | 32 | 35 => 4,
        9 | 23 | 25 | 33 | 44 | 50 => 5,
        2 | 6 | 15 | 41 | 53 => 6,
        10 | 11 | 12 | 13 | 24 | 37 | 45 | 51 | 54 => 7,
        19 | 20 | 27 | 29 | 31 | 38 | 46 => 8,
        5 | 22 | 40 | 48 => 9,
        _ => return None,
    })
}

pub struct Tables2<'a> {
    pub rows: &'a [(u64, &'a [u32])],
}

/// A row as found: the 2 weights and the values.
#[derive(Clone, Copy, Debug)]
pub struct Row<'a> {
    pub own: u8,
    pub division: u8,
    pub values: &'a [u32],
}

impl<'a> Tables2<'a> {
    /// The row at exactly this key, or `None`.
    pub fn find(&self, k: u64) -> Option<Row<'a>> {
        let i = self.rows.binary_search_by_key(&k, |r| r.0).ok()?;
        let r = self.rows[i].1;
        if r.len() < 3 {
            return None;
        }
        Some(Row { own: r[0].min(255) as u8, division: r[1].min(255) as u8, values: &r[2..] })
    }
    /// The row for a state: its own, else its division's, else the nation's.
    pub fn at(&self, table: u32, state: u32, division: u32, a: u32, b: u32, c: u32) -> Option<Row<'a>> {
        self.find(key2(table, state, a, b, c))
            .or_else(|| self.find(key2(table, DIVISION_BASE + division, a, b, c)))
            .or_else(|| self.find(key2(table, NATION, a, b, c)))
    }
}

/// A dwelling with, for each of 9 fields (type, vintage, fuel, wall, roof, storeys, area,
/// electricity, non-electric), the weight of the state's own households in the row it was
/// drawn from and the division's weight in what is left, each in 1/255ths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keyed {
    pub dwelling: Dwelling,
    pub own: [u8; 9],
    pub division: [u8; 9],
    /// The copula is normal and the survey's upper tail is heavier (BUILDGEN-1: 1.6 % of
    /// detached houses in the top 5 % of both size and energy against 0.8 % generated).
    pub tail_understated: bool,
    /// Whether the correlation matrix was the division's own (else the nation's).
    pub matrix_of_division: bool,
}

/// Generate one dwelling for a state (2-digit federal code) and an urban type (0 to 2).
/// `None` if the code is not one of the 51, or if a row the chain needs exists at no level.
pub fn generate_keyed(t: &Tables2, seed: u64, state: u8, urban: u8) -> Option<Keyed> {
    let dv = division_of(state)? as u32;
    let (s, urban) = (state as u32, (urban as u32).min(2));
    let mut st = Stream::new(seed);
    let (mut own, mut division) = ([0u8; 9], [0u8; 9]);
    let mut note = |i: usize, r: &Row| {
        own[i] = r.own;
        division[i] = r.division;
    };
    // 1. type
    let r = t.at(T_TYPE, s, dv, urban, 0, 0)?;
    note(0, &r);
    let kind = pick(&mut st, r.values) as u32;
    let house = is_house(kind as u8);
    // 2. the vintage score and its band
    let dim = if house { 5 } else { 4 };
    let mut z = [0i64; 5];
    z[0] = normal_of((st.next_u64() >> 32) as u32) as i64;
    let r = t.at(T_VINTAGE, s, dv, kind, 0, 0)?;
    note(1, &r);
    let vintage = pick_by_u(r.values, uniform_of(z[0] as i32)) as u8;
    let e = era(vintage) as u32;
    // 3. the chain
    let r = t.at(T_FUEL, s, dv, kind, e, 0)?;
    note(2, &r);
    let fuel = pick(&mut st, r.values) as u8;
    let fg = fuel_group(fuel) as u32;
    let r = t.at(T_WALL, s, dv, kind, e, 0)?;
    note(3, &r);
    let wall = pick(&mut st, r.values) as u8;
    let roof = if kind == 4 {
        NOT_APPLICABLE
    } else {
        let r = t.at(T_ROOF, s, dv, 0, wall as u32, 0)?;
        note(4, &r);
        pick(&mut st, r.values) as u8
    };
    let split_level = if house { pick(&mut st, t.find(key2(T_SPLIT, NATION, kind, e, 0))?.values) as u8 } else { NOT_APPLICABLE };
    // 4. the copula: the division's factor for this type if it has one, else the nation's
    for zi in z.iter_mut().take(dim).skip(1) {
        *zi = normal_of((st.next_u64() >> 32) as u32) as i64;
    }
    let (chol, matrix_of_division) = match t.find(key2(T_CHOL, DIVISION_BASE + dv, kind, 0, 0)) {
        Some(r) => (r.values, true),
        None => (t.find(key2(T_CHOL, NATION, kind, 0, 0))?.values, false),
    };
    if chol.len() < dim * (dim + 1) / 2 {
        return None;
    }
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
        let r = t.at(T_STOREYS, s, dv, kind, 0, 0)?;
        note(5, &r);
        next += 1;
        pick_by_u(r.values, u[1]) as u8 + 1
    } else {
        0
    };
    let r = t.at(T_AREA, s, dv, kind, 0, 0)?;
    note(6, &r);
    let area_sqft = quantile(r.values, u[next]);
    let r = t.at(T_KWH, s, dv, kind, fg, 0)?;
    note(7, &r);
    let kwh = quantile(r.values, u[next + 1]);
    // 5. non-electric energy: zero by its own table, else its positive part from the copula
    let zero = pick(&mut st, t.at(T_ZERO, s, dv, kind, fg, 0)?.values) == 0;
    let nonelec_kbtu = if zero {
        0
    } else {
        let r = t.at(T_NONELEC, s, dv, kind, fg, 0)?;
        note(8, &r);
        quantile(r.values, u[next + 2])
    };
    let elec_kbtu = ((kwh as u64 * 3412 + 500) / 1000) as u32;
    let total_kbtu = elec_kbtu + nonelec_kbtu;
    // 6. end uses, outside the copula
    let eu = t.at(T_ENDUSE, s, dv, kind, fg, 0)?.values;
    if eu.len() < 2 {
        return None;
    }
    let heating_kbtu = ((total_kbtu as u64 * eu[0].min(1_000_000) as u64) / 1_000_000) as u32;
    let cooling_kbtu = (((total_kbtu as u64 * eu[1].min(1_000_000) as u64) / 1_000_000) as u32).min(elec_kbtu).min(total_kbtu - heating_kbtu);
    let dwelling = Dwelling { kind: kind as u8, vintage, fuel, wall, roof, storeys, split_level, area_sqft, kwh, nonelec_kbtu, total_kbtu, heating_kbtu, cooling_kbtu, levels: [0; 9] };
    Some(Keyed { dwelling, own, division, tail_understated: true, matrix_of_division })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buildings::KNOTS;

    /// A planted country. The nation has every row. Division 9 (West South Central) has its own
    /// fuel rows; state 48 has its own fuel row for detached houses built before 1980 alone.
    fn planted(with_type_4: bool) -> Vec<(u64, Vec<u32>)> {
        let mut rows = Vec::new();
        let knots = |lo: u32, step: u32| -> Vec<u32> { [0, 0].into_iter().chain((0..KNOTS as u32).map(|i| lo + step * i)).collect() };
        for urban in 0..3 {
            rows.push((key2(T_TYPE, NATION, urban, 0, 0), vec![0, 0, 1, 5, 2, 1, if with_type_4 { 3 } else { 0 }]));
        }
        for kind in 0..5 {
            rows.push((key2(T_VINTAGE, NATION, kind, 0, 0), vec![0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1]));
            rows.push((key2(T_STOREYS, NATION, kind, 0, 0), vec![0, 0, 5, 4, 1]));
            rows.push((key2(T_AREA, NATION, kind, 0, 0), knots(500, 60)));
            for e in 0..3 {
                rows.push((key2(T_FUEL, NATION, kind, e, 0), vec![0, 0, 3, 4, 1, 1, 1, 0, 1]));
                rows.push((key2(T_FUEL, DIVISION_BASE + 9, kind, e, 0), vec![0, 200, 8, 1, 0, 0, 0, 0, 0]));
                rows.push((key2(T_WALL, NATION, kind, e, 0), vec![0, 0, 2, 2, 3, 1, 1, 1, 1, 1]));
                rows.push((key2(T_SPLIT, NATION, kind, e, 0), vec![0, 0, 9, 1]));
            }
            for fg in 0..4 {
                rows.push((key2(T_KWH, NATION, kind, fg, 0), knots(2000, 400)));
                rows.push((key2(T_NONELEC, NATION, kind, fg, 0), knots(5000, 2500)));
                rows.push((key2(T_ZERO, NATION, kind, fg, 0), vec![0, 0, if fg == 0 { 9 } else { 1 }, if fg == 0 { 1 } else { 9 }]));
                rows.push((key2(T_ENDUSE, NATION, kind, fg, 0), vec![0, 0, 400_000, 100_000]));
            }
            let d = if is_house(kind as u8) { 5 } else { 4 };
            let mut c = vec![0u32, 0];
            for i in 0..d {
                for j in 0..=i {
                    c.push(if i == j { 65536 } else { 0 });
                }
            }
            rows.push((key2(T_CHOL, NATION, kind, 0, 0), c));
        }
        for wall in 0..8 {
            rows.push((key2(T_ROOF, NATION, 0, wall, 0), vec![0, 0, 1, 1, 2, 1, 6, 1, 1]));
        }
        rows.push((key2(T_FUEL, 48, 1, 0, 0), vec![230, 128, 0, 1, 0, 0, 0, 0, 0]));
        rows.sort_by_key(|r| r.0);
        rows
    }

    fn with<R>(owned: &[(u64, Vec<u32>)], f: impl FnOnce(&Tables2) -> R) -> R {
        let rows: Vec<(u64, &[u32])> = owned.iter().map(|r| (r.0, r.1.as_slice())).collect();
        f(&Tables2 { rows: &rows })
    }

    #[test]
    fn keys_do_not_collide_and_each_state_has_a_division() {
        let mut seen = std::collections::BTreeSet::new();
        for table in 1..14 {
            for place in [0u32, 1, 56, 60, 69, 127] {
                for a in [0u32, 7] {
                    for b in [0u32, 15] {
                        for c in [0u32, 7] {
                            assert!(seen.insert(key2(table, place, a, b, c)));
                        }
                    }
                }
            }
        }
        let states: Vec<u8> = (0..=255u8).filter(|&s| division_of(s).is_some()).collect();
        assert_eq!(states.len(), 51);
        let mut per = [0u32; 10];
        for &s in &states {
            per[division_of(s).unwrap() as usize] += 1;
        }
        assert_eq!(per, [5, 4, 3, 5, 3, 6, 5, 9, 7, 4]);
        assert!(states.iter().all(|&s| s >= 1 && s <= 56) && ![3u8, 7, 14, 43, 52].iter().any(|s| states.contains(s)));
    }

    #[test]
    fn a_place_that_is_not_a_state_is_refused_and_an_absent_type_is_never_drawn() {
        let owned = planted(false);
        with(&owned, |t| {
            for bad in [0u8, 3, 60, 72, 255] {
                assert!(generate_keyed(t, 1, bad, 0).is_none(), "{bad}");
            }
            for i in 0..20_000u64 {
                let k = generate_keyed(t, crate::seed::mix(i), 36, (i % 3) as u8).unwrap();
                assert_ne!(k.dwelling.kind, 4, "a type with no share at the nation");
            }
        });
    }

    #[test]
    fn a_row_is_the_states_then_the_divisions_then_the_nations() {
        let owned = planted(true);
        with(&owned, |t| {
            let (mut tx_old, mut tx_old_gas, mut la_elec, mut la, mut ny_elec, mut ny) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
            for i in 0..60_000u64 {
                let sd = crate::seed::mix(i);
                let k = generate_keyed(t, sd, 48, 0).unwrap();
                let d = &k.dwelling;
                if d.kind == 1 && era(d.vintage) == 0 {
                    tx_old += 1;
                    tx_old_gas += (d.fuel == 1) as u32;
                    assert_eq!((k.own[2], k.division[2]), (230, 128), "the state's own row reports its weights");
                } else {
                    assert_eq!((k.own[2], k.division[2]), (0, 200), "the division's row");
                }
                let k = generate_keyed(t, sd, 22, 0).unwrap();
                la += 1;
                la_elec += (k.dwelling.fuel == 0) as u32;
                let k = generate_keyed(t, sd, 36, 0).unwrap();
                ny += 1;
                ny_elec += (k.dwelling.fuel == 0) as u32;
                assert_eq!(k.own[2], 0);
                assert!(k.tail_understated && !k.matrix_of_division);
            }
            assert!(tx_old > 3000 && tx_old_gas == tx_old, "the state's own row is all gas: {tx_old_gas} of {tx_old}");
            let (a, b) = (la_elec as f64 / la as f64, ny_elec as f64 / ny as f64);
            assert!((a - 8.0 / 9.0).abs() < 0.01, "a state with no row of its own draws from its division: {a}");
            assert!((b - 3.0 / 11.0).abs() < 0.01, "a state whose division has none draws from the nation: {b}");
        });
    }

    #[test]
    fn the_section_law_holds_and_a_draw_repeats() {
        let owned = planted(true);
        with(&owned, |t| {
            let states: Vec<u8> = (1..=56u8).filter(|&s| division_of(s).is_some()).collect();
            let mut kinds = [0u32; 5];
            for i in 0..20_000u64 {
                let (sd, s, u) = (crate::seed::mix(i), states[(i % 51) as usize], (i % 3) as u8);
                let k = generate_keyed(t, sd, s, u).unwrap();
                let d = &k.dwelling;
                kinds[d.kind as usize] += 1;
                assert!(d.total_kbtu as u64 * 1000 + 500 >= d.kwh as u64 * 3412);
                assert!(d.heating_kbtu + d.cooling_kbtu <= d.total_kbtu);
                assert_eq!(is_house(d.kind), d.storeys != 0);
                assert_eq!(is_house(d.kind), d.split_level != NOT_APPLICABLE);
                assert_eq!(d.kind == 4, d.roof == NOT_APPLICABLE);
                assert!(d.vintage < 9 && d.wall < 8 && d.fuel < 7);
                assert_eq!(generate_keyed(t, sd, s, u).unwrap(), k);
            }
            assert!(kinds.iter().all(|&k| k > 500), "{kinds:?}");
        });
    }

    #[test]
    fn a_missing_row_refuses_and_a_divisions_matrix_is_used_where_it_exists() {
        let mut owned = planted(true);
        // a strong division matrix for detached houses in division 6: area follows vintage
        let mut c = vec![0u32, 0];
        for i in 0..5 {
            for j in 0..=i {
                c.push(if (i == j && i != 2) || (i == 2 && j == 0) { 65536 } else { 0 });
            }
        }
        owned.push((key2(T_CHOL, DIVISION_BASE + 6, 1, 0, 0), c));
        owned.sort_by_key(|r| r.0);
        with(&owned, |t| {
            let (mut n, mut agree) = (0u32, 0u32);
            for i in 0..30_000u64 {
                let k = generate_keyed(t, crate::seed::mix(i), 6, 0).unwrap();
                let d = &k.dwelling;
                assert_eq!(k.matrix_of_division, d.kind == 1);
                if d.kind == 1 {
                    n += 1;
                    agree += ((d.vintage >= 5) == (d.area_sqft >= 500 + 60 * 16)) as u32;
                }
            }
            assert!(agree as f64 / n as f64 > 0.9, "area ranks with vintage under the division's matrix: {agree} of {n}");
        });
        let cut: Vec<(u64, Vec<u32>)> = planted(true).into_iter().filter(|r| r.0 != key2(T_AREA, NATION, 1, 0, 0)).collect();
        with(&cut, |t| {
            let (mut none, mut some) = (0u32, 0u32);
            for i in 0..2000u64 {
                match generate_keyed(t, crate::seed::mix(i), 36, 0) {
                    None => none += 1,
                    Some(k) => {
                        some += 1;
                        assert_ne!(k.dwelling.kind, 1);
                    }
                }
            }
            assert!(none > 500 && some > 500, "a detached house with no area row is refused, not invented: {none} {some}");
        });
    }
}
