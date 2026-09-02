//! The tier's own charts: occupancy → coarse fields, on ITS lattice.
//!
//! For `b | L`, `v_b` sends a micro-state to the field of block gross states, summing the
//! conserved label over each `b × b` block exactly as `GrossState::combine` does. The family
//! has both degenerate ends inside it — `v_1` is the per-cell sector field and `v_L` is the
//! single global conserved label — which is why the closure reading is a CURVE along `b` and
//! never one number.
//!
//! # `v_L` closes, and that is the trap, not the result
//!
//! At `b = L` the chart is the conserved total and is Held by G1–G3 alone. Rung 2 flagged
//! exactly this shape — one cell closes by conservation alone, which is an existence theorem
//! in field-chart clothes. It is carried here LABELLED, so that a defect curve cannot quietly
//! end in a success.

use crate::lattice::Lattice;
use crate::state::Model;

/// A coarse field: one `(N, Px, Py)` per block, in row-major block order.
pub type Field = Vec<[i64; 3]>;

/// The block chart `v_b`.
#[derive(Clone, Copy, Debug)]
pub struct BlockChart {
    pub b: usize,
    pub l: usize,
}

impl BlockChart {
    /// `b` must divide `L`, or the blocks are not a partition and the chart is not a view.
    pub fn new(b: usize, l: usize) -> Option<Self> {
        (b >= 1 && b <= l && l % b == 0).then_some(Self { b, l })
    }

    #[inline]
    pub fn blocks_per_side(&self) -> usize {
        self.l / self.b
    }

    /// Is this the global chart? Then it is Held by conservation alone and any closure
    /// reading on it is VACUOUS. Callers must label it; nothing here hides it.
    #[inline]
    pub fn is_vacuous_by_conservation(&self) -> bool {
        self.b == self.l
    }

    /// `W(b) = 1 − max(0, b−2)² / b²` for `b < L`, and `0` at `b = L`.
    ///
    /// The geometric bound of LG_PREREG §5.3, derived rather than fitted: the collision is a
    /// bijection fixing `(N,P)` and streaming moves each particle exactly one cell, so a cell
    /// whose neighbours all lie in its own block cannot produce a witness. On a torus a
    /// single block has no inter-block edges, which is why the SAME formula reports the
    /// `b = L` end as exactly zero — the vacuity is arithmetic, not commentary.
    pub fn predicted_witness_rate(b: usize, l: usize) -> f64 {
        if b >= l {
            return 0.0;
        }
        let interior = b.saturating_sub(2);
        1.0 - (interior * interior) as f64 / (b * b) as f64
    }

    /// Apply the chart.
    pub fn apply(&self, model: &Model, cells: &[u8]) -> Field {
        let nb = self.blocks_per_side();
        let mut out = vec![[0i64; 3]; nb * nb];
        for i in 0..self.l {
            for j in 0..self.l {
                let (n, x, y) = model.label(cells[i * self.l + j]);
                let acc = &mut out[(i / self.b) * nb + (j / self.b)];
                acc[0] += n as i64;
                acc[1] += x;
                acc[2] += y;
            }
        }
        out
    }

    pub fn apply_to(&self, g: &Lattice) -> Field {
        self.apply(&g.model, &g.cells)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chart_family_has_both_degenerate_ends_inside_it() {
        let m = Model::fhp6();
        let g = Lattice::seeded(m.clone(), 16, 7, 0.4, m.fhp_i(true));
        let cell = BlockChart::new(1, 16).unwrap().apply_to(&g);
        assert_eq!(cell.len(), 256);
        let global = BlockChart::new(16, 16).unwrap();
        assert!(global.is_vacuous_by_conservation());
        let f = global.apply_to(&g);
        assert_eq!(f.len(), 1);
        let l = g.ledger();
        assert_eq!(f[0], [l.mass, l.momentum[0], l.momentum[1]]);
    }

    /// Every chart in the family is a coarsening of every finer one: the block sums agree.
    /// A "chart" that did not would not be a view of the same micro-state.
    #[test]
    fn coarser_charts_sum_the_finer_ones() {
        let m = Model::fhp6();
        let g = Lattice::seeded(m.clone(), 16, 7, 0.4, m.fhp_i(true));
        for &(fine, coarse) in &[(1usize, 2usize), (2, 4), (4, 8), (8, 16)] {
            let vf = BlockChart::new(fine, 16).unwrap().apply_to(&g);
            let vc = BlockChart::new(coarse, 16).unwrap().apply_to(&g);
            let (nf, nc) = (16 / fine, 16 / coarse);
            let mut rolled = vec![[0i64; 3]; nc * nc];
            for bi in 0..nf {
                for bj in 0..nf {
                    let t = &mut rolled[(bi * fine / coarse) * nc + (bj * fine / coarse)];
                    for k in 0..3 {
                        t[k] += vf[bi * nf + bj][k];
                    }
                }
            }
            assert_eq!(rolled, vc, "b={fine} does not roll up into b={coarse}");
        }
    }

    /// The predicted curve, pinned against the frozen reference table of LG_PREREG §5.3.
    #[test]
    fn the_predicted_curve_matches_the_frozen_reference() {
        let expect = [
            (1usize, 1.0),
            (2, 1.0),
            (4, 0.75),
            (8, 0.4375),
            (16, 0.234375),
            (32, 0.12109375),
            (64, 0.0),
        ];
        for (b, w) in expect {
            assert_eq!(BlockChart::predicted_witness_rate(b, 64), w, "b={b}");
        }
    }

    #[test]
    fn a_chart_whose_block_does_not_divide_the_lattice_is_refused() {
        assert!(BlockChart::new(3, 64).is_none());
        assert!(BlockChart::new(0, 64).is_none());
        assert!(BlockChart::new(128, 64).is_none());
    }
}
