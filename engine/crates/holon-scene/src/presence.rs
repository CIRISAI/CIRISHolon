//! Where a person is in each hour of a day (PRESENCEGEN-1 and its successor PRESENCEGEN-2,
//! `conformance/scene/PRESENCEGEN2_PREREG.md`, `conformance/scene/PRESENCEGEN2_RESULTS.md`).
//!
//! Two generators over the six states of PRESENCE-0. `Chain`: a chain over the state and
//! one bit (has the person left home today). `Frame`: a kind of day is drawn, then the
//! hour first left and the hours until last home from one aggregated table, then a short
//! chain between them; before and after, a two-state chain at home.
//! The law: one state per person per hour (`SceneRhythm.conserved`). Integers only.
//!
//! Slot `s` of a day is clock hour `(4 + s) mod 24`. States: 0 home asleep, 1 home awake,
//! 2 workplace or school, 3 elsewhere awake, 4 elsewhere asleep, 5 travelling.

use crate::devices::{Call, Tables};
use crate::verdict::Verdict;
use crate::seed::Stream;

pub const KIND_HOME: u8 = 0;
pub const KIND_WORK_EARLY: u8 = 1;
pub const KIND_WORK_LATE: u8 = 2;
pub const KIND_OUT: u8 = 3;
/// A day from the chain has no kind.
pub const KIND_NONE: u8 = 4;

const ANY: u32 = 4;
const ANY_SLOT: u32 = 24;
const NO_FRAME: usize = 13;
const POOLED_SLOT: u32 = 31;
const GROUP_NONE: u32 = 6;
const GROUP_AFTER: u32 = 7;

/// Table ids.
pub const T_KIND: u32 = 1;
pub const T_INIT: u32 = 2;
pub const T_CHAIN: u32 = 3;
pub const T_HOME_INIT: u32 = 4;
pub const T_HOME_CHAIN: u32 = 5;
pub const T_LEFT: u32 = 6;
pub const T_AWAY: u32 = 7;
pub const T_INSIDE_INIT: u32 = 8;
pub const T_INSIDE: u32 = 9;
pub const T_CHANGES: u32 = 10;
pub const T_EMPLOYED: u32 = 11;
// PRESENCEGEN-2: the fill at home keyed on its place in the day (x: 0..5 before the frame,
// 6 a day with no outing, 7 after the frame), the state on return, six states within the frame.
pub const T_H2_BAND: u32 = 12;
pub const T_H2_COUNT: u32 = 13;
pub const T_H2_INIT_BAND: u32 = 14;
pub const T_H2_INIT_COUNT: u32 = 15;
pub const T_RETURN: u32 = 16;
pub const T_INSIDE6_INIT: u32 = 17;
pub const T_INSIDE6: u32 = 18;

/// The key of one table row.
pub fn key(table: u32, employed: u32, x: u32, slot: u32, from: u32) -> u32 {
    (((table * 2 + employed) * 8 + x) * 32 + slot) * 16 + from
}

/// Which generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Generator {
    Chain,
    /// PRESENCEGEN-1's frame: one chain at home for everyone. Kept as a comparator.
    Frame,
    /// PRESENCEGEN-2's fill, the hours before leaving keyed by the four-hour band of the hour first left.
    FrameBand,
    /// PRESENCEGEN-2's fill, the hours before leaving keyed by the hours until leaving.
    FrameCountdown,
}

/// One generated day.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Day {
    pub states: [u8; 24],
    pub kind: u8,
    /// Changes of place in the day, drawn from the diaries' minute-level count for the kind.
    pub place_changes: u8,
    /// A row was borrowed from a coarser table.
    pub fallback: bool,
    /// Which of the eight groups at home borrowed a row (bit `x`).
    pub fallback_groups: u8,
    /// The frame ran past the day's end and was cut.
    pub clipped: bool,
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

fn get<'a>(t: &Tables<'a>, k: u32) -> Option<&'a [u32]> {
    t.rows.binary_search_by_key(&k, |r| r.0).ok().map(|i| t.rows[i].1)
}

/// Inside states of a frame, in table order.
const INSIDE: [u8; 6] = [2, 3, 4, 5, 1, 0];

/// A row of the fill at home: the keyed row, or the next by the frozen order (an earlier
/// band; a longer countdown), or the row over everyone at home. A borrowed row flags the day.
#[allow(clippy::too_many_arguments)]
fn home_row<'a>(t: &Tables<'a>, g: Generator, e: u32, table: u32, pooled: u32, x: u32, slot: u32, from: u32, out: &mut Day) -> Option<&'a [u32]> {
    if g != Generator::Frame {
        let mut xx = x;
        loop {
            if let Some(r) = get(t, key(table, e, xx, slot, from)) {
                if xx != x {
                    out.fallback = true;
                    out.fallback_groups |= 1 << x;
                }
                return Some(r);
            }
            if x >= GROUP_NONE {
                break;
            }
            if g == Generator::FrameBand {
                if xx == 0 {
                    break;
                }
                xx -= 1;
            } else {
                if xx == 5 {
                    break;
                }
                xx += 1;
            }
        }
        out.fallback = true;
        out.fallback_groups |= 1 << x;
    }
    get(t, key(pooled, e, 0, slot, from))
}

/// Generate one day. `day_type`: 0 Monday to Thursday, 1 Friday, 2 Saturday, 3 Sunday.
/// `week_same_kind`: a declared, unmeasured knob; when set, the kind of a working-week day
/// is drawn from `person_seed` and so is the same on each such day.
pub fn day_with(t: &Tables, g: Generator, seed: u64, employed: bool, day_type: u8, person_seed: u64, week_same_kind: bool) -> Day {
    let e = employed as u32;
    let d = (day_type as u32).min(3);
    let mut st = Stream::new(seed);
    let mut out = Day { states: [0; 24], kind: KIND_NONE, place_changes: 0, fallback: false, fallback_groups: 0, clipped: false };
    match g {
        Generator::Chain => {
            let init = get(t, key(T_INIT, e, d, 0, 0)).expect("initial row");
            let mut s = pick(&mut st, init) as u32;
            let mut left = (s > 1) as u32;
            out.states[0] = s as u8;
            for slot in 0..23u32 {
                let from = s * 2 + left;
                let row = match get(t, key(T_CHAIN, e, d, slot, from)) {
                    Some(r) => r,
                    None => {
                        out.fallback = true;
                        match get(t, key(T_CHAIN, e, ANY, slot, from)).or_else(|| get(t, key(T_CHAIN, e, ANY, ANY_SLOT, from))) {
                            Some(r) => r,
                            None => {
                                out.states[slot as usize + 1] = s as u8;
                                continue;
                            }
                        }
                    }
                };
                s = pick(&mut st, row) as u32;
                left |= (s > 1) as u32;
                out.states[slot as usize + 1] = s as u8;
            }
        }
        Generator::Frame | Generator::FrameBand | Generator::FrameCountdown => {
            let keyed = g != Generator::Frame;
            let kinds = get(t, key(T_KIND, e, d, 0, 0)).expect("kind row");
            let kind = if week_same_kind && d <= 1 {
                let mut ps = Stream::new(person_seed);
                pick(&mut ps, kinds) as u32
            } else {
                pick(&mut st, kinds) as u32
            };
            out.kind = kind as u8;
            // the frame
            let (mut first, mut last) = (24u32, 24u32);
            if kind != KIND_HOME as u32 {
                let lrow = get(t, key(T_LEFT, e, kind, 0, 0)).expect("first-left row");
                let cell = pick(&mut st, lrow);
                if cell != NO_FRAME {
                    // the day's first two hours have a cell each; two-hour bins after
                    first = if cell < 2 { cell as u32 } else { (cell as u32 - 1) * 2 + st.below(2) as u32 };
                    let arow = match get(t, key(T_AWAY, e, kind, cell as u32, 0)) {
                        Some(r) => r,
                        None => {
                            out.fallback = true;
                            get(t, key(T_AWAY, e, kind, POOLED_SLOT, 0)).expect("pooled away row")
                        }
                    };
                    // one-hour cells (PRESENCEGEN-1's CORRECTION of 2026-11-12)
                    let dur = pick(&mut st, arow) as u32 + 1;
                    if first + dur > 24 {
                        out.clipped = true;
                    }
                    last = (first + dur).min(24);
                }
            }
            // A transition into an hour is keyed on that hour's group.
            // The group of an hour at home: six before the frame (by band or by countdown),
            // a day with no outing, after the frame.
            let group = |slot: u32| -> u32 {
                if first >= 24 {
                    GROUP_NONE
                } else if slot >= last {
                    GROUP_AFTER
                } else if g == Generator::FrameBand {
                    first / 4
                } else {
                    (first - slot).min(6) - 1
                }
            };
            let (t_init, t_chain) = match g {
                Generator::FrameBand => (T_H2_INIT_BAND, T_H2_BAND),
                _ => (T_H2_INIT_COUNT, T_H2_COUNT),
            };
            let mut h = 1u32;
            if first > 0 {
                let hi = home_row(t, g, e, t_init, T_HOME_INIT, group(0), 0, 0, &mut out).expect("home initial row");
                h = pick(&mut st, hi) as u32;
            }
            let mut inside = 0usize;
            for slot in 0..24u32 {
                if slot >= first && slot < last {
                    if slot == first {
                        let keyed_init = if keyed { get(t, key(T_INSIDE6_INIT, e, kind, (first != 0) as u32, 0)) } else { None };
                        let ii = keyed_init.or_else(|| get(t, key(T_INSIDE_INIT, e, kind, 0, 0))).expect("inside initial row");
                        inside = pick(&mut st, ii);
                    } else {
                        let tb = if keyed { T_INSIDE6 } else { T_INSIDE };
                        let row = match get(t, key(tb, e, kind, slot - 1, inside as u32)) {
                            Some(r) => Some(r),
                            None => {
                                out.fallback = true;
                                get(t, key(tb, e, kind, ANY_SLOT, inside as u32))
                            }
                        };
                        if let Some(r) = row {
                            inside = pick(&mut st, r);
                        }
                    }
                    // the frame's first and last hours are away from home by definition
                    if (slot + 1 == last || slot == first) && INSIDE[inside] <= 1 {
                        inside = 3;
                    }
                    out.states[slot as usize] = INSIDE[inside];
                    h = 1;
                } else {
                    if slot == last {
                        // the state on return: drawn for the keyed fills, awake for the plain frame
                        if keyed {
                            let r = get(t, key(T_RETURN, e, 0, slot / 2, 0)).or_else(|| {
                                out.fallback = true;
                                out.fallback_groups |= 1 << GROUP_AFTER;
                                get(t, key(T_RETURN, e, 0, POOLED_SLOT, 0))
                            });
                            if let Some(r) = r {
                                h = pick(&mut st, r) as u32;
                            }
                        }
                    } else if slot > 0 {
                        if let Some(r) = home_row(t, g, e, t_chain, T_HOME_CHAIN, group(slot), slot - 1, h, &mut out) {
                            h = pick(&mut st, r) as u32;
                        }
                    }
                    out.states[slot as usize] = h as u8;
                }
            }
            if let Some(c) = get(t, key(T_CHANGES, e, kind, 0, 0)) {
                out.place_changes = pick(&mut st, c) as u8;
            }
        }
    }
    out
}

/// The notice that must travel with anything produced from the compiled-in tables.
pub const NOTICE_US: &str = "Source: U.S. Bureau of Labor Statistics, American Time Use Survey, 2003-2024 multi-year microdata files. A derived work; not endorsed by the Bureau.";

/// The compiled-in tables: United States, diaries of 2022 to 2024.
pub fn us_2022_2024() -> Tables<'static> {
    Tables { rows: crate::presence_tables::ROWS }
}

/// One day of one person aged 15 or over, from the compiled-in tables, with the verdict on
/// the generator asked for (PRESENCEGEN-2, scored on the diaries of 2024). A verdict is a
/// statement about many generated days, never about one. Nobody under 15 has a diary: a
/// child gets no day from this function.
///
/// - `Chain`: within budget on each reading, and the one that carries day types by hour.
/// - `FrameCountdown`: returns a kind of day and takes the `week_same_kind` knob. Its tables
///   are pooled over day types; Saturday's hours are missed and the call says so.
/// - `Frame`: refused (its days are too awake; PRESENCEGEN-1's gate).
/// - `FrameBand`: its keyed tables are not compiled in.
pub fn day(g: Generator, seed: u64, employed: bool, day_type: u8, person_seed: u64, week_same_kind: bool, in_country: bool) -> (Day, Call) {
    let d = day_with(&us_2022_2024(), g, seed, employed, day_type, person_seed, week_same_kind);
    let (verdict, reason) = if !in_country {
        (Verdict::Unresolved, "outside the country read (United States, 2022 to 2024)")
    } else {
        match (g, day_type.min(3)) {
            (Generator::Chain, _) => (Verdict::Certified, "scored on the diaries of 2024 within budget: spells away, three far-apart hours, hours asleep, sleep before leaving; hours by day type reproduced"),
            (Generator::FrameCountdown, 0) => (Verdict::Certified, "scored on the diaries of 2024 within budget: spells away, three far-apart hours, hours asleep, sleep before leaving; Monday to Thursday"),
            (Generator::FrameCountdown, 2) => (Verdict::Refused, "tables pooled over day types: Saturday's hours at home awake are within budget in 11 of 24"),
            (Generator::FrameCountdown, _) => (Verdict::Unresolved, "tables pooled over day types; Friday and Sunday were not read apart"),
            (Generator::Frame, _) => (Verdict::Refused, "one chain at home for everyone: awake 0.645 of the day against 0.623"),
            (Generator::FrameBand, _) => (Verdict::Unresolved, "the tables for this key are not compiled in"),
        }
    };
    (d, Call { verdict, reason })
}

/// The share employed, in parts per million, by age band: 0 = 15 to 24, 1 = 25 to 64,
/// 2 = 65 and over. For a caller that holds ages and not employment.
pub fn employed_ppm(age_band: u8) -> u32 {
    get(&us_2022_2024(), key(T_EMPLOYED, 0, (age_band as u32).min(2), 0, 0)).map(|r| r[0]).unwrap_or(0)
}

/// A number folding 1,000 fixed calls of each shipped generator. One value on every target.
pub fn digest() -> u64 {
    use crate::seed::mix;
    let t = us_2022_2024();
    let mut dg = 0u64;
    for g in [Generator::Chain, Generator::FrameCountdown] {
        for i in 0..1000u64 {
            let d = day_with(&t, g, mix(i), i % 2 == 0, (i % 4) as u8, mix(i / 5), i % 3 == 0);
            for x in d.states.iter() {
                dg = mix(dg ^ *x as u64);
            }
            dg = mix(dg ^ d.kind as u64 ^ ((d.place_changes as u64) << 8));
        }
    }
    dg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::mix;

    fn toy() -> Vec<(u32, Vec<u32>)> {
        let mut rows: Vec<(u32, Vec<u32>)> = Vec::new();
        for e in 0..2u32 {
            for d in 0..4u32 {
                rows.push((key(T_KIND, e, d, 0, 0), vec![250_000, 400_000, 100_000, 250_000]));
                rows.push((key(T_INIT, e, d, 0, 0), vec![900_000, 100_000, 0, 0, 0, 0]));
            }
            rows.push((key(T_HOME_INIT, e, 0, 0, 0), vec![900_000, 100_000]));
            for slot in 0..24u32 {
                for from in 0..2u32 {
                    let wake = if (3..18).contains(&slot) { 900_000 } else { 100_000 };
                    rows.push((key(T_HOME_CHAIN, e, 0, slot, from), vec![1_000_000 - wake, wake]));
                }
            }
            for kind in 1..4u32 {
                let mut l = vec![0u32; 14];
                l[3] = 700_000;
                l[6] = 200_000;
                l[13] = 100_000;
                rows.push((key(T_LEFT, e, kind, 0, 0), l));
                let mut a = vec![0u32; 24];
                a[if kind == 3 { 1 } else { 8 }] = 1_000_000;
                rows.push((key(T_AWAY, e, kind, POOLED_SLOT, 0), a));
                rows.push((key(T_INSIDE_INIT, e, kind, 0, 0), vec![0, 0, 0, 1_000_000, 0]));
                for from in 0..5u32 {
                    rows.push((key(T_INSIDE, e, kind, ANY_SLOT, from), vec![600_000, 200_000, 0, 100_000, 100_000]));
                }
                rows.push((key(T_CHANGES, e, kind, 0, 0), vec![0, 0, 500_000, 300_000, 200_000]));
            }
            // a chain that leaves home at slot 4 with probability one half, returns at slot 13
            for slot in 0..23u32 {
                for from in 0..12u32 {
                    let s = from / 2;
                    let mut r = vec![0u32; 6];
                    if s <= 1 && slot == 4 && from % 2 == 0 {
                        r[1] = 500_000;
                        r[2] = 500_000;
                    } else if s > 1 && slot == 13 {
                        r[1] = 1_000_000;
                    } else {
                        r[if s == 0 && slot >= 3 { 1 } else { s as usize }] = 1_000_000;
                    }
                    rows.push((key(T_CHAIN, e, ANY, slot, from), r));
                }
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

    /// P-ONE: one state per hour, a known state; a HOME day never leaves home.
    #[test]
    fn one_state_per_hour() {
        with(|t| {
            for i in 0..5000u64 {
                for g in [Generator::Chain, Generator::Frame] {
                    let d = day_with(t, g, mix(i), i % 2 == 0, (i % 4) as u8, 0, false);
                    assert!(d.states.iter().all(|&s| s <= 5));
                    if d.kind == KIND_HOME {
                        assert!(d.states.iter().all(|&s| s <= 1));
                    }
                }
            }
        });
    }

    /// The frame's hours away are a single run bounded by home, and its last hour is away.
    #[test]
    fn the_frame_is_respected() {
        with(|t| {
            let mut framed = 0;
            for i in 0..4000u64 {
                let d = day_with(t, Generator::Frame, mix(i), true, 0, 0, false);
                let away: Vec<usize> = (0..24).filter(|&s| d.states[s] > 1).collect();
                if let (Some(&a), Some(&b)) = (away.first(), away.last()) {
                    framed += 1;
                    assert!(d.states[..a].iter().all(|&s| s <= 1));
                    assert!(d.states[b + 1..].iter().all(|&s| s <= 1));
                    assert!(a == 4 || a == 5 || a == 10 || a == 11, "first left {a}");
                }
            }
            assert!(framed > 2000);
        });
    }

    /// P-SEED.
    #[test]
    fn one_seed_one_day() {
        with(|t| {
            assert_eq!(day_with(t, Generator::Frame, 9, true, 0, 1, false), day_with(t, Generator::Frame, 9, true, 0, 1, false));
            let differ = (0..500u64).filter(|&i| day_with(t, Generator::Frame, mix(i), true, 0, 0, false) != day_with(t, Generator::Frame, mix(i + 7000), true, 0, 0, false)).count();
            assert!(differ > 300, "{differ}");
        });
    }

    /// P-WEEK: with the knob set, a person's working-week days share a kind.
    #[test]
    fn the_week_knob() {
        with(|t| {
            for p in 0..200u64 {
                let kinds: Vec<u8> = (0..5u64).map(|day| day_with(t, Generator::Frame, mix(p * 10 + day), true, if day == 4 { 1 } else { 0 }, mix(p + 99), true).kind).collect();
                assert!(kinds.iter().all(|&k| k == kinds[0]));
            }
            let varied = (0..200u64).filter(|&p| {
                let kinds: Vec<u8> = (0..5u64).map(|day| day_with(t, Generator::Frame, mix(p * 10 + day), true, 0, mix(p + 99), false).kind).collect();
                kinds.iter().any(|&k| k != kinds[0])
            }).count();
            assert!(varied > 150);
        });
    }

    /// P-BORROW: a chain row missing for the day type is borrowed and the day is flagged.
    #[test]
    fn a_borrowed_row_is_flagged() {
        with(|t| {
            let d = day_with(t, Generator::Chain, 3, true, 2, 0, false);
            assert!(d.fallback);
        });
    }

    /// PRESENCEGEN-2: with a countdown row saying "awake in the hour before leaving", nobody
    /// who leaves after the first hour is asleep in that hour; the plain frame has some.
    #[test]
    fn the_countdown_wakes_before_leaving() {
        let mut owned = toy();
        for e in 0..2u32 {
            for slot in 0..24u32 {
                for from in 0..2u32 {
                    owned.push((key(T_H2_COUNT, e, 0, slot, from), vec![0, 1_000_000]));
                }
            }
        }
        owned.sort_by_key(|r| r.0);
        let rows: Vec<(u32, &[u32])> = owned.iter().map(|r| (r.0, r.1.as_slice())).collect();
        let t = Tables { rows: &rows };
        let asleep_before = |g: Generator| {
            (0..2000u64)
                .filter(|&i| {
                    let d = day_with(&t, g, mix(i), true, 0, 0, false);
                    match (0..24).find(|&s| d.states[s] > 1) {
                        Some(f) if f >= 2 => d.states[f - 1] == 0,
                        _ => false,
                    }
                })
                .count()
        };
        // the row into the hour before leaving sends everyone awake
        assert_eq!(asleep_before(Generator::FrameCountdown), 0);
        assert!(asleep_before(Generator::Frame) > 0);
    }

    /// PRESENCEGEN-2: the state on return is drawn, and a group with no keyed row is flagged.
    #[test]
    fn the_return_is_drawn_and_a_borrowed_group_is_flagged() {
        let mut owned = toy();
        for e in 0..2u32 {
            owned.push((key(T_RETURN, e, 0, POOLED_SLOT, 0), vec![1_000_000, 0]));
        }
        owned.sort_by_key(|r| r.0);
        let rows: Vec<(u32, &[u32])> = owned.iter().map(|r| (r.0, r.1.as_slice())).collect();
        let t = Tables { rows: &rows };
        let mut seen = 0;
        for i in 0..500u64 {
            let d = day_with(&t, Generator::FrameBand, mix(i), true, 0, 0, false);
            let away: Vec<usize> = (0..24).filter(|&s| d.states[s] > 1).collect();
            if let Some(&b) = away.last() {
                if b + 1 < 24 {
                    assert_eq!(d.states[b + 1], 0, "asleep on return");
                    seen += 1;
                }
            }
            assert!(d.fallback && d.fallback_groups != 0);
            assert_eq!(d.states.len(), 24);
        }
        assert!(seen > 100);
    }

    /// The shipped tables: one state per hour, a day at home never leaves, and the verdicts.
    #[test]
    fn the_shipped_tables_hold_the_law() {
        for i in 0..4000u64 {
            for g in [Generator::Chain, Generator::FrameCountdown] {
                let (d, _) = day(g, mix(i), i % 2 == 0, (i % 4) as u8, 0, false, true);
                assert!(d.states.iter().all(|&s| s <= 5));
                if d.kind == KIND_HOME {
                    assert!(d.states.iter().all(|&s| s <= 1));
                }
            }
        }
        assert_eq!(day(Generator::Chain, 1, true, 2, 0, false, true).1.verdict, Verdict::Certified);
        assert_eq!(day(Generator::FrameCountdown, 1, true, 2, 0, false, true).1.verdict, Verdict::Refused);
        assert_eq!(day(Generator::Frame, 1, true, 0, 0, false, true).1.verdict, Verdict::Refused);
        assert_eq!(day(Generator::Chain, 1, true, 0, 0, false, false).1.verdict, Verdict::Unresolved);
        assert!(employed_ppm(1) > employed_ppm(0) && employed_ppm(0) > employed_ppm(2));
        assert_eq!(digest(), 2016341816221085133, "one digest on every target");
    }
}
