//! The edge at a resolution: the cells whose fiber the dynamics splits.
//!
//! # What it is
//!
//! GANTT2's EDGE-2, and node LG's instrument read as a SET instead of as a rate. `holon-
//! lattice`'s probe measures a witness RATE — how often a fiber-preserving perturbation
//! changes the stepped chart — and reports one number per block size. The same measurement
//! read per cell is a set: the cells where a witness exists are the cells the coarse view
//! does not close over, and on a carrier with an interface in it those cells ARE the
//! interface. `Core/Closure.lean`'s `viewClosed_iff_never_splits` is the definition being
//! used: closed means two states agreeing under the view still agree after the step, so
//! "splits" is exactly its negation, localised.
//!
//! # What it declares
//!
//! [`BlockView`] is what a chart must be able to say for the edge to be read at all: its
//! resolution, how many cells it has, whether the dynamics splits each cell's fiber, where
//! each cell is, whether the carrier wraps, and whether the chart is vacuous. The trait is
//! stated HERE and implemented THERE — `holon-lattice::chart::BlockChart` implements it
//! without this crate ever depending on `holon-lattice`, which is the only arrangement that
//! lets `holon-render` and `holon-lens` share the type.
//!
//! # What it refuses
//!
//! * **An empty edge is not a zero-width edge.** [`Edge::empty`] is carried and the width is
//!   `NaN` and the position `None` when nothing split. A chart at which no cell splits is a
//!   verdict about the chart — the view closes there — and M-EMPTY-SECTOR's rule is that a
//!   measured exact zero on the staked configuration voids what it gates rather than
//!   feeding it a number.
//! * **A vacuous chart's edge is not a result.** At `b = L` a block chart is the conserved
//!   total and is held by conservation alone; `holon-lattice`'s `BlockChart` labels that and
//!   so does [`Edge::vacuous`], so a defect curve read through this type cannot quietly end
//!   in a success.
//! * **A centroid on a torus is a circular mean, not an arithmetic one.** The arithmetic
//!   mean of positions that wrap is a point in the middle of the empty half. Where
//!   [`BlockView::period`] says the carrier wraps, the centroid is the circular mean and
//!   every spread is a minimum-image distance. Where it says the carrier is open, both are
//!   the plain ones.
//! * **The width is a SPREAD and says so.** With no interface normal declared, an isotropic
//!   root-mean-square spread about the centroid is what the cell set supports; it is the
//!   right number for "does the width converge as b grows" (EDGE-2's kill) and it is NOT an
//!   interface thickness along a normal. A tier that has a normal measures that separately.

/// What a chart must be able to say for its edge to be read at a resolution.
///
/// Implemented by the tier, never by this crate. Positions are in the carrier's own
/// coordinates and the two axes are whatever the carrier's two axes are.
pub trait BlockView {
    /// The chart's resolution: the block size, in carrier cells.
    fn b(&self) -> usize;

    /// How many cells the chart has at this resolution.
    fn cells(&self) -> usize;

    /// Does the dynamics SPLIT this cell's fiber — is there a pair of micro-states agreeing
    /// under the chart here and disagreeing after the step? The negation of
    /// `Core/Closure.lean`'s `ViewClosed`, localised to one cell.
    fn splits(&self, cell: usize) -> bool;

    /// Where the cell is, in the carrier's own two coordinates.
    fn position(&self, cell: usize) -> [f64; 2];

    /// The carrier's period on each axis if it wraps, `None` if it is open. A torus MUST
    /// say so: the centroid and every spread below change when it does.
    fn period(&self) -> Option<[f64; 2]> {
        None
    }

    /// Is this chart held by conservation alone — the `b = L` end? A reading on it is
    /// vacuous and this carries the label through.
    fn vacuous(&self) -> bool {
        false
    }
}

/// The edge at one resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    /// The resolution this was read at.
    pub b: usize,
    /// How many cells the chart has.
    pub cells_total: usize,
    /// The cells whose fiber the dynamics splits, ascending.
    pub cells: Vec<usize>,
    /// `cells.len() / cells_total`.
    pub fraction: f64,
    /// The centroid of the edge cells, circular where the carrier wraps. `None` when the
    /// edge is empty.
    pub position: Option<[f64; 2]>,
    /// The root-mean-square spread of the edge cells about that centroid, in the carrier's
    /// own length unit, minimum-image where the carrier wraps. `NaN` when the edge is
    /// empty — an absent edge has no width.
    pub width: f64,
    /// Nothing split. A verdict about the chart, not a measurement of an edge.
    pub empty: bool,
    /// The chart is the conserved total and is held by conservation alone.
    pub vacuous: bool,
}

impl Edge {
    /// Read the edge at the view's own resolution.
    pub fn at<V: BlockView + ?Sized>(view: &V) -> Edge {
        let n = view.cells();
        let cells: Vec<usize> = (0..n).filter(|&c| view.splits(c)).collect();
        let fraction = if n == 0 { f64::NAN } else { cells.len() as f64 / n as f64 };
        let empty = cells.is_empty();
        if empty {
            return Edge {
                b: view.b(),
                cells_total: n,
                cells,
                fraction,
                position: None,
                width: f64::NAN,
                empty,
                vacuous: view.vacuous(),
            };
        }
        let period = view.period();
        let pts: Vec<[f64; 2]> = cells.iter().map(|&c| view.position(c)).collect();
        let centre = centroid(&pts, period);
        let mut sq = 0.0f64;
        for p in &pts {
            let d = separation(*p, centre, period);
            sq += d[0] * d[0] + d[1] * d[1];
        }
        let width = (sq / pts.len() as f64).sqrt();
        Edge {
            b: view.b(),
            cells_total: n,
            cells,
            fraction,
            position: Some(centre),
            width,
            empty,
            vacuous: view.vacuous(),
        }
    }

    /// The edge over a family of charts, one reading per resolution — EDGE-2's curve. The
    /// caller supplies the family because which resolutions exist is the tier's business
    /// (`b` must divide `L` on a block chart).
    pub fn sweep<'a, V: BlockView + 'a>(views: impl IntoIterator<Item = &'a V>) -> Vec<Edge> {
        views.into_iter().map(Edge::at).collect()
    }
}

/// `b − a` under the minimum image where the carrier wraps, plain where it does not.
fn separation(a: [f64; 2], b: [f64; 2], period: Option<[f64; 2]>) -> [f64; 2] {
    let mut d = [a[0] - b[0], a[1] - b[1]];
    if let Some(l) = period {
        for k in 0..2 {
            if l[k].is_finite() && l[k] > 0.0 {
                d[k] -= l[k] * (d[k] / l[k]).round();
            }
        }
    }
    d
}

/// The centroid: circular on a wrapping axis, arithmetic on an open one.
///
/// The circular mean is the angle of the summed unit vectors, mapped back into `[0, L)`.
/// Where the summed vector is the zero vector the mean is undefined — points spread evenly
/// round the ring have no centre — and the arithmetic mean is used instead and is
/// meaningless; the case is left rather than hidden because an [`Edge`] that wraps the whole
/// torus is not an interface and its width is the number that says so.
fn centroid(pts: &[[f64; 2]], period: Option<[f64; 2]>) -> [f64; 2] {
    let n = pts.len() as f64;
    let mut out = [0.0f64; 2];
    for k in 0..2 {
        let wrap = period.map(|l| l[k]).filter(|l| l.is_finite() && *l > 0.0);
        match wrap {
            Some(l) => {
                let (mut cs, mut sn) = (0.0f64, 0.0f64);
                for p in pts {
                    let t = 2.0 * core::f64::consts::PI * p[k] / l;
                    cs += t.cos();
                    sn += t.sin();
                }
                if cs == 0.0 && sn == 0.0 {
                    out[k] = pts.iter().map(|p| p[k]).sum::<f64>() / n;
                } else {
                    let mut a = sn.atan2(cs);
                    if a < 0.0 {
                        a += 2.0 * core::f64::consts::PI;
                    }
                    out[k] = a * l / (2.0 * core::f64::consts::PI);
                }
            }
            None => out[k] = pts.iter().map(|p| p[k]).sum::<f64>() / n,
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A square block chart over an `l × l` carrier, with a declared split set.
    struct Square {
        b: usize,
        side: usize,
        splitting: Vec<bool>,
        wraps: bool,
        vacuous: bool,
    }

    impl Square {
        fn new(b: usize, side: usize, wraps: bool) -> Square {
            Square { b, side, splitting: vec![false; side * side], wraps, vacuous: false }
        }
        fn split(mut self, cells: &[usize]) -> Square {
            for &c in cells {
                self.splitting[c] = true;
            }
            self
        }
    }

    impl BlockView for Square {
        fn b(&self) -> usize {
            self.b
        }
        fn cells(&self) -> usize {
            self.side * self.side
        }
        fn splits(&self, cell: usize) -> bool {
            self.splitting[cell]
        }
        fn position(&self, cell: usize) -> [f64; 2] {
            [(cell % self.side) as f64, (cell / self.side) as f64]
        }
        fn period(&self) -> Option<[f64; 2]> {
            self.wraps.then_some([self.side as f64, self.side as f64])
        }
        fn vacuous(&self) -> bool {
            self.vacuous
        }
    }

    #[test]
    fn an_edge_is_the_set_of_cells_that_split() {
        let v = Square::new(4, 8, false).split(&[3, 11, 19]);
        let e = Edge::at(&v);
        assert_eq!(e.cells, vec![3, 11, 19]);
        assert_eq!(e.cells_total, 64);
        assert_eq!(e.b, 4);
        assert!((e.fraction - 3.0 / 64.0).abs() < 1e-15);
        assert!(!e.empty);
    }

    /// M-EMPTY-SECTOR: nothing split is a verdict about the chart, not a zero-width edge.
    #[test]
    fn an_empty_edge_reads_empty_and_carries_no_width() {
        let v = Square::new(4, 8, false);
        let e = Edge::at(&v);
        assert!(e.empty);
        assert_eq!(e.position, None);
        assert!(e.width.is_nan(), "an absent edge has no width");
        assert_eq!(e.fraction, 0.0);
    }

    #[test]
    fn a_vacuous_chart_carries_its_label_through() {
        let mut v = Square::new(8, 1, false).split(&[0]);
        v.vacuous = true;
        assert!(Edge::at(&v).vacuous);
    }

    /// A column of cells has zero spread across the column and a real spread along it.
    #[test]
    fn the_width_is_the_spread_about_the_centroid() {
        // cells (2,0), (2,1), (2,2), (2,3) of an 8-wide open chart: x = 2 exactly
        let v = Square::new(1, 8, false).split(&[2, 10, 18, 26]);
        let e = Edge::at(&v);
        let c = e.position.unwrap();
        assert!((c[0] - 2.0).abs() < 1e-12);
        assert!((c[1] - 1.5).abs() < 1e-12);
        // spread is the y spread alone: rms of (-1.5,-0.5,0.5,1.5) = sqrt(1.25)
        assert!((e.width - 1.25f64.sqrt()).abs() < 1e-12, "width {}", e.width);
    }

    /// The centroid of a band straddling the wrap is IN the band, not opposite it.
    #[test]
    fn a_wrapping_edge_has_a_circular_centroid() {
        // an 8-wide torus, x in {7, 0}: the centre is the seam at 7.5, not 3.5
        let open = Square::new(1, 8, false).split(&[7, 0]);
        let torus = Square::new(1, 8, true).split(&[7, 0]);
        let (eo, et) = (Edge::at(&open), Edge::at(&torus));
        assert!((eo.position.unwrap()[0] - 3.5).abs() < 1e-12, "the open mean is the arithmetic one");
        assert!(
            (et.position.unwrap()[0] - 7.5).abs() < 1e-9,
            "the circular mean sits on the seam, got {}",
            et.position.unwrap()[0]
        );
        assert!(et.width < eo.width, "and the wrapping width is the smaller one");
        assert!((et.width - 0.5).abs() < 1e-9, "width {}", et.width);
    }

    /// EDGE-2's curve: one reading per resolution, in the order the family was given.
    #[test]
    fn a_sweep_is_one_reading_per_resolution() {
        let vs = [
            Square::new(2, 8, true).split(&[0, 1]),
            Square::new(4, 4, true).split(&[0]),
            Square::new(8, 2, true),
        ];
        let sweep = Edge::sweep(vs.iter());
        assert_eq!(sweep.iter().map(|e| e.b).collect::<Vec<_>>(), vec![2, 4, 8]);
        assert_eq!(sweep.iter().map(|e| e.cells.len()).collect::<Vec<_>>(), vec![2, 1, 0]);
        assert!(sweep[2].empty);
    }
}
