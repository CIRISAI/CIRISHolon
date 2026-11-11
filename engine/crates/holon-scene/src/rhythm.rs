//! Rhythms: the population law in time.
//!
//! A [`Series`] is what a cell holds in each time bin, what moved between cells during each
//! bin, and a source per cell per bin. The law (`SceneRhythm.Continuity`):
//!
//! ```text
//! next = now − what leaves + what arrives + source
//! ```
//!
//! One type, two consumers on the day it was written. A simulated town asks who is where in
//! each hour of the week: the source is declared zero and the check refuses a person who
//! appears from nowhere. A lattice gauge run asks how a charge moves under a field: for
//! electric charge the source is zero, and for the handed charge the source is the anomaly,
//! which [`implied_source`] reads off as whatever the counts and flows fail to account for.
//!
//! The same series is what a renderer draws (cells that fill and empty, arrows along the
//! flows) and what an instrument reads. It is not converted between the two uses.
//!
//! Integers only. A consumer with real-valued readings fixes its unit first.

/// Counts, flows and sources over `bins` time bins on `cells` cells.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Series {
    pub cells: usize,
    /// `counts[t][c]`, for `t` in `0..=bins`.
    pub counts: Vec<Vec<i64>>,
    /// `flows[t]`: `(from, to, amount)` during bin `t`, for `t` in `0..bins`.
    pub flows: Vec<Vec<(usize, usize, i64)>>,
    /// `source[t][c]` during bin `t`.
    pub source: Vec<Vec<i64>>,
}

/// One place the law failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    pub bin: usize,
    pub cell: usize,
    pub expected: i64,
    pub found: i64,
}

fn net(cells: usize, flows: &[(usize, usize, i64)]) -> Vec<i64> {
    let mut n = vec![0i64; cells];
    for &(a, b, w) in flows {
        n[a] -= w;
        n[b] += w;
    }
    n
}

/// Check the law in every bin and every cell. Returns EVERY violation.
pub fn check_continuity(s: &Series) -> Vec<Violation> {
    let mut out = Vec::new();
    for t in 0..s.flows.len() {
        let n = net(s.cells, &s.flows[t]);
        for c in 0..s.cells {
            let expected = s.counts[t][c] + n[c] + s.source[t][c];
            if expected != s.counts[t + 1][c] {
                out.push(Violation { bin: t, cell: c, expected, found: s.counts[t + 1][c] });
            }
        }
    }
    out
}

/// The source the counts and flows imply: what each cell gained that no flow brought.
/// `SceneRhythm.source_is_the_defect`.
pub fn implied_source(cells: usize, counts: &[Vec<i64>], flows: &[Vec<(usize, usize, i64)>]) -> Vec<Vec<i64>> {
    (0..flows.len())
        .map(|t| {
            let n = net(cells, &flows[t]);
            (0..cells).map(|c| counts[t + 1][c] - counts[t][c] - n[c]).collect()
        })
        .collect()
}

/// The same series read on coarse cells. Flows inside one coarse cell become a flow from it
/// to itself and cancel in the law. `SceneRhythm.continuity_coarse`.
pub fn coarsen(s: &Series, parent_of: &[usize], n_coarse: usize) -> Series {
    let fold = |v: &Vec<i64>| {
        let mut o = vec![0i64; n_coarse];
        for (c, &x) in v.iter().enumerate() {
            o[parent_of[c]] += x;
        }
        o
    };
    Series {
        cells: n_coarse,
        counts: s.counts.iter().map(fold).collect(),
        flows: s
            .flows
            .iter()
            .map(|f| f.iter().map(|&(a, b, w)| (parent_of[a], parent_of[b], w)).collect())
            .collect(),
        source: s.source.iter().map(fold).collect(),
    }
}

/// The total held in each bin.
pub fn totals(s: &Series) -> Vec<i64> {
    s.counts.iter().map(|v| v.iter().sum()).collect()
}

/// Does the series return to where it started? A weekly rhythm must.
pub fn is_periodic(s: &Series) -> bool {
    s.counts.first() == s.counts.last()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::Stream;

    /// A random lawful series: start from random counts, move random amounts, add a source.
    fn random_series(seed: u64, cells: usize, bins: usize, with_source: bool) -> Series {
        let mut r = Stream::new(seed);
        let mut counts = vec![(0..cells).map(|_| r.below(500) as i64).collect::<Vec<_>>()];
        let (mut flows, mut source) = (Vec::new(), Vec::new());
        for _ in 0..bins {
            let f: Vec<(usize, usize, i64)> = (0..2 * cells)
                .map(|_| (r.below(cells as u64) as usize, r.below(cells as u64) as usize, r.below(20) as i64))
                .collect();
            let src: Vec<i64> = (0..cells).map(|_| if with_source { r.below(7) as i64 - 3 } else { 0 }).collect();
            let n = net(cells, &f);
            let last = counts.last().unwrap().clone();
            counts.push((0..cells).map(|c| last[c] + n[c] + src[c]).collect());
            flows.push(f);
            source.push(src);
        }
        Series { cells, counts, flows, source }
    }

    #[test]
    fn a_lawful_series_has_no_violation() {
        for seed in 0..30 {
            assert_eq!(check_continuity(&random_series(seed, 12, 24, true)), vec![]);
        }
    }

    #[test]
    fn with_no_source_the_total_never_changes() {
        for seed in 0..30 {
            let t = totals(&random_series(seed, 9, 168, false));
            assert!(t.iter().all(|&x| x == t[0]));
        }
    }

    #[test]
    fn with_a_source_the_total_changes_by_exactly_the_source() {
        let s = random_series(4, 10, 50, true);
        let t = totals(&s);
        for k in 0..50 {
            assert_eq!(t[k + 1] - t[k], s.source[k].iter().sum::<i64>());
        }
    }

    #[test]
    fn the_law_survives_reading_coarsely() {
        for seed in 0..30 {
            let s = random_series(seed, 24, 40, true);
            let mut r = Stream::new(seed + 1000);
            let parent: Vec<usize> = (0..24).map(|_| r.below(5) as usize).collect();
            let c = coarsen(&s, &parent, 5);
            assert_eq!(check_continuity(&c), vec![]);
            assert_eq!(totals(&c), totals(&s));
        }
    }

    #[test]
    fn the_source_is_read_off_as_the_defect() {
        let s = random_series(9, 8, 30, true);
        assert_eq!(implied_source(s.cells, &s.counts, &s.flows), s.source);
    }

    #[test]
    fn someone_appearing_from_nowhere_is_refused_and_named() {
        let mut s = random_series(2, 6, 10, false);
        s.counts[4][3] += 1;
        let v = check_continuity(&s);
        // the extra person breaks the bin that arrives at them and the bin that leaves
        assert_eq!(v.len(), 2);
        assert!(v.iter().all(|x| x.cell == 3));
        assert_eq!((v[0].bin, v[1].bin), (3, 4));
    }

    #[test]
    fn one_cells_record_shifted_by_a_step_is_refused_and_named() {
        // the plant ANOMALY-0 stakes: a series misaligned in time by one bin
        let mut s = random_series(11, 6, 20, false);
        let col: Vec<i64> = s.counts.iter().map(|v| v[2]).collect();
        for t in 1..s.counts.len() {
            s.counts[t][2] = col[t - 1];
        }
        let v = check_continuity(&s);
        assert!(!v.is_empty());
        assert!(v.iter().all(|x| x.cell == 2));
    }

    #[test]
    fn a_week_that_returns_is_periodic() {
        // 5 places, everyone goes out along a ring and comes back
        let mut counts = vec![vec![100i64, 0, 0, 0, 0]];
        let mut flows = Vec::new();
        for t in 0..10 {
            let (a, b) = if t < 5 { (t % 5, (t + 1) % 5) } else { ((10 - t) % 5, (9 - t) % 5) };
            let amt = counts.last().unwrap()[a];
            flows.push(vec![(a, b, amt)]);
            let mut next = counts.last().unwrap().clone();
            next[a] -= amt;
            next[b] += amt;
            counts.push(next);
        }
        let s = Series { cells: 5, counts, flows, source: vec![vec![0; 5]; 10] };
        assert_eq!(check_continuity(&s), vec![]);
        assert!(is_periodic(&s));
    }
}
