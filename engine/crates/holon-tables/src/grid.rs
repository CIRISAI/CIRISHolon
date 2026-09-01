//! The node set, the region decomposition, and the traversal — all three canonical
//! functions of the grid alone.
//!
//! This module is where G1's bit-identity actually comes from. Nothing here knows how many
//! workers exist, and that is the point: if the partition or the traversal could see the
//! worker count, the warm-start chain would change with it, and the measured `3.4e-13` to
//! `4.3e-12` hartree warm-vs-cold spread would become a difference between a 1-worker and
//! a 32-worker table.

/// A node's canonical index into the whole table. Stable, worker-count-independent.
///
/// # Why this is still `u32` after the N-axis fold
///
/// Six axes can multiply past `u32` where three never could, so the question was forced by
/// [`NdGrid`] and is answered here rather than in a commit message.
///
/// It stays `u32`. Widening it would change [`crate::GenOutcome::table_bytes`]'s layout —
/// four bytes per node become eight — and that byte string is the artifact the whole
/// campaign's bit-identity comparison is taken over, so the widening would be a silent
/// change to every committed table's strictest comparator. (The DIGEST would survive, since
/// `Digest::of_record` already hashes `r.node as u64`; the byte string would not.)
///
/// And the ceiling is not a real constraint: `u32::MAX` nodes is 4.3e9 electronic-structure
/// solves, against a committed SATURATION-2 table of 105,105. A grid that overflowed this
/// would need terabytes for the slot vector alone. [`NdGrid::new`] therefore asserts the
/// node count fits, loudly and at construction, rather than wrapping into two nodes sharing
/// a slot — which the partition's own assert would report as "solved twice" from a place
/// that could not explain it.
pub type NodeId = u32;

/// A region's canonical index. Regions are the unit of work handed to a worker AND the
/// unit a warm-start chain lives inside.
pub type RegionId = u32;

/// The table's grid: a box in the trimer coordinates `(x, y, u)` cut into canonical
/// regions.
///
/// # The coordinates
///
/// `x` and `y` are the two SHORTEST sides of the triangle and `u` the cosine of the angle
/// between them — `holon-chem/src/trimer.rs`'s own coordinates, chosen there because every
/// point of the box is a realisable triangle and so a stencil can never fall into a hole.
/// This crate inherits them rather than inventing a parameterisation, so the mesh is
/// exercised on the surface the tables are actually built on.
///
/// # The region shape is a declared constant, not a tuning knob
///
/// `region` is part of the table's identity. Change it and the warm-start chains change,
/// so the last bits of the table change — legitimately, since a different chain is a
/// different (equally valid) computation, but NOT silently: the region shape is hashed
/// into the digest by [`crate::generate`], so two tables built with different region
/// shapes are visibly different artifacts rather than mysteriously disagreeing ones.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableGrid {
    pub nx: usize,
    pub ny: usize,
    pub nu: usize,
    /// Region edge lengths in `(i, j, k)`. The last region along an axis is short when the
    /// edge does not divide the extent; that is handled rather than forbidden, because
    /// forcing the grid to be divisible would make the region shape reach back into the
    /// physics domain.
    pub region: [usize; 3],
    /// The box in the physical coordinates. Node `(i,j,k)` sits at the linear interpolant
    /// of these; the production domain derivation is the tables lane's business (G0/T1),
    /// and this crate takes whatever box it is handed.
    pub x_lo: f64,
    pub x_hi: f64,
    pub y_lo: f64,
    pub y_hi: f64,
    pub u_lo: f64,
    pub u_hi: f64,
}

impl TableGrid {
    /// A grid with the given extents and region shape, on a declared box.
    ///
    /// # Panics
    ///
    /// On a zero extent or a zero region edge. A zero-volume region would put every node in
    /// its own region, silently turning the whole table cold — a performance collapse that
    /// would look like a correct table, which is the worst kind.
    pub fn new(
        nx: usize,
        ny: usize,
        nu: usize,
        region: [usize; 3],
        (x_lo, x_hi): (f64, f64),
        (y_lo, y_hi): (f64, f64),
        (u_lo, u_hi): (f64, f64),
    ) -> Self {
        assert!(
            nx > 0 && ny > 0 && nu > 0,
            "a grid extent of zero has no nodes to shard"
        );
        assert!(
            region.iter().all(|&r| r > 0),
            "a region edge of zero would put every node in its own region and turn the \
             whole table cold while still producing a correct-looking result"
        );
        assert!(
            x_hi > x_lo && y_hi > y_lo && u_hi > u_lo,
            "the grid box must be non-degenerate in every coordinate"
        );
        Self {
            nx,
            ny,
            nu,
            region,
            x_lo,
            x_hi,
            y_lo,
            y_hi,
            u_lo,
            u_hi,
        }
    }

    /// Total nodes.
    pub fn n_nodes(&self) -> usize {
        self.nx * self.ny * self.nu
    }

    /// The canonical linear index of `(i, j, k)`.
    pub fn node_id(&self, i: usize, j: usize, k: usize) -> NodeId {
        debug_assert!(i < self.nx && j < self.ny && k < self.nu);
        ((i * self.ny + j) * self.nu + k) as NodeId
    }

    /// The `(i, j, k)` of a canonical index.
    pub fn coords(&self, id: NodeId) -> (usize, usize, usize) {
        let id = id as usize;
        let k = id % self.nu;
        let j = (id / self.nu) % self.ny;
        let i = id / (self.nu * self.ny);
        (i, j, k)
    }

    /// The physical geometry of a node: the two short sides and the cosine between them.
    pub fn geometry(&self, id: NodeId) -> (f64, f64, f64) {
        let (i, j, k) = self.coords(id);
        let f = |lo: f64, hi: f64, idx: usize, n: usize| {
            if n == 1 {
                lo
            } else {
                lo + (hi - lo) * (idx as f64) / ((n - 1) as f64)
            }
        };
        (
            f(self.x_lo, self.x_hi, i, self.nx),
            f(self.y_lo, self.y_hi, j, self.ny),
            f(self.u_lo, self.u_hi, k, self.nu),
        )
    }

    /// How many regions the grid divides into along each axis.
    pub fn region_extents(&self) -> (usize, usize, usize) {
        (
            self.nx.div_ceil(self.region[0]),
            self.ny.div_ceil(self.region[1]),
            self.nu.div_ceil(self.region[2]),
        )
    }

    /// Total regions — the number of independent work units, and NOT a function of the
    /// worker count.
    pub fn n_regions(&self) -> usize {
        let (a, b, c) = self.region_extents();
        a * b * c
    }

    /// Which region a node belongs to. A pure function of the node and the grid.
    pub fn region_of(&self, id: NodeId) -> RegionId {
        let (i, j, k) = self.coords(id);
        let (_, rb, rc) = self.region_extents();
        let (ri, rj, rk) = (
            i / self.region[0],
            j / self.region[1],
            k / self.region[2],
        );
        ((ri * rb + rj) * rc + rk) as RegionId
    }

    /// The nodes of one region, **in canonical traversal order**.
    ///
    /// # Why serpentine and not lexicographic
    ///
    /// The traversal decides what each node's warm start is: node `n` starts from the
    /// converged vector of node `n-1` in this list. Under a plain lexicographic walk the
    /// step from `(i, j, k_max)` to `(i, j+1, k_min)` jumps the whole width of the region
    /// in `k`, so one node in every row gets a warm start from a geometry as far away as
    /// the region is wide — the worst guess in the region, handed out `region_y * region_x`
    /// times.
    ///
    /// A serpentine (boustrophedon) walk reverses every other row, so consecutive nodes in
    /// the traversal are USUALLY grid-adjacent. It is just as canonical — a function of the
    /// region's coordinates and nothing else — and it is the better guess everywhere the
    /// lexicographic walk differs.
    ///
    /// **CORRECTED 2026-09-01: this comment used to claim adjacency UNCONDITIONALLY, and
    /// that claim is false.** This is the sum-parity rule, which is adjacent iff every axis
    /// STRICTLY BETWEEN the first and the last has odd region extent — see
    /// [`NdGrid::adjacency_is_guaranteed`] for the derivation. **The production region shape
    /// `[2, 2, 2]` FAILS it**, measured: the walk breaks once per i-fold, e.g.
    /// `(0,1,0) -> (1,1,1)` at Manhattan distance 2. Empirically 0 breaks at every odd
    /// middle extent tested and 1–3 breaks at every even one.
    ///
    /// Nothing is changed here and nothing should be: these tables are gated on
    /// bit-identity, so altering the traversal alters every recorded digest, and the
    /// property the invariant bought (warm-start locality) is not in use — WARM IS OFF on
    /// all four production tables. [`Serpentine::Reflected`] is adjacent for any extents and
    /// is what new surfaces should use. What was wrong was the COMMENT, not the walk.
    pub fn region_nodes(&self, r: RegionId) -> Vec<NodeId> {
        let (_, rb, rc) = self.region_extents();
        let r = r as usize;
        let rk = r % rc;
        let rj = (r / rc) % rb;
        let ri = r / (rc * rb);

        let i0 = ri * self.region[0];
        let j0 = rj * self.region[1];
        let k0 = rk * self.region[2];
        let i1 = (i0 + self.region[0]).min(self.nx);
        let j1 = (j0 + self.region[1]).min(self.ny);
        let k1 = (k0 + self.region[2]).min(self.nu);

        let mut out = Vec::with_capacity((i1 - i0) * (j1 - j0) * (k1 - k0));
        for (ii, i) in (i0..i1).enumerate() {
            // Reverse j on every other i-plane, and k on every other (i, j) row. This makes
            // consecutive pairs adjacent WITHIN a plane always, and across the folds only
            // when the middle axis has odd extent — see the corrected note above. Do not
            // "fix" it here: the digests are the artifact's identity.
            let js: Vec<usize> = if ii % 2 == 0 {
                (j0..j1).collect()
            } else {
                (j0..j1).rev().collect()
            };
            for (jj, j) in js.into_iter().enumerate() {
                let ks: Vec<usize> = if (ii + jj) % 2 == 0 {
                    (k0..k1).collect()
                } else {
                    (k0..k1).rev().collect()
                };
                for k in ks {
                    out.push(self.node_id(i, j, k));
                }
            }
        }
        out
    }

    /// Every region's node list, region index ascending. The whole work partition.
    pub fn partition(&self) -> Vec<Vec<NodeId>> {
        (0..self.n_regions() as RegionId)
            .map(|r| self.region_nodes(r))
            .collect()
    }
}

// ===========================================================================
// The dimension-generic grid (WB-8.7: one leased tabulation pipeline, N axes)
// ===========================================================================
//
// `TableGrid` above is the 3-axis trimer grid. It is not deleted and it is not
// re-derived: `NdGrid` is a strict generalisation of it, and
// `NdGrid::from_table_grid` plus `tests/nd_bit_identity.rs` prove — exhaustively,
// node by node and bit by bit — that the two agree on every one of the five
// canonical functions (`n_nodes`, `node_id`, `coords`, `geometry`, `region_of`,
// `region_nodes`). That proof is the whole warrant for the fold: the 3-body
// tables are gated on bit-identity, and the generator now goes through `NdGrid`.

/// How an axis's node index becomes a physical coordinate.
///
/// The two maps here are the only two the campaign actually uses, and both are
/// written in the EXACT expression order of the code they replace, because the
/// tables are gated on bit-identity and `a * b / c` is not `a * (b / c)` in `f64`:
///
/// * [`AxisMap::Linear`] is `TableGrid::geometry`'s interpolant, `n == 1` special
///   case included;
/// * [`AxisMap::ExpStretch`] is `holon_chem::{water, ooh, ozone}::r_of_tau`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AxisMap {
    /// `phys = lo + (hi - lo) * i / (n - 1)`, and `lo` when `n == 1`.
    Linear,
    /// `phys = lo + (hi - lo) * (exp(a*t) - 1) / (exp(a) - 1)` with `t = i / (n - 1)`,
    /// and `lo` when `n == 1`.
    ///
    /// The exponential stretch is what puts the nodes where the potential is: dense
    /// at short range where the curvature is, sparse in the tail where it is not.
    ExpStretch { a: f64 },
}

impl AxisMap {
    /// The physical coordinate of node `i` on an axis of `n` nodes spanning `[lo, hi]`.
    ///
    /// **Written to be bit-identical to the code it folds.** `Linear` is character for
    /// character `TableGrid::geometry`'s closure; changing the association here would
    /// move the last bits of every committed 3-body table.
    #[inline]
    pub fn coord(self, i: usize, n: usize, lo: f64, hi: f64) -> f64 {
        if n == 1 {
            return lo;
        }
        match self {
            AxisMap::Linear => lo + (hi - lo) * (i as f64) / ((n - 1) as f64),
            AxisMap::ExpStretch { a } => {
                let t = i as f64 / (n - 1) as f64;
                lo + (hi - lo) * ((a * t).exp() - 1.0) / (a.exp() - 1.0)
            }
        }
    }
}

/// One axis of an [`NdGrid`]: how many nodes, over what box, under what map, cut into
/// regions of what edge length.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Axis {
    pub n: usize,
    pub lo: f64,
    pub hi: f64,
    pub map: AxisMap,
    /// Region edge length along this axis. Part of the table's identity for the same
    /// reason `TableGrid::region` is: it decides the warm-start chains.
    pub region: usize,
}

impl Axis {
    pub fn linear(n: usize, lo: f64, hi: f64, region: usize) -> Axis {
        Axis { n, lo, hi, map: AxisMap::Linear, region }
    }

    pub fn stretched(n: usize, lo: f64, hi: f64, a: f64, region: usize) -> Axis {
        Axis { n, lo, hi, map: AxisMap::ExpStretch { a }, region }
    }

    /// This axis's physical coordinate at node index `i`.
    #[inline]
    pub fn coord(&self, i: usize) -> f64 {
        self.map.coord(i, self.n, self.lo, self.hi)
    }
}

/// Which serpentine rule a grid traverses a region with.
///
/// # Why there are two, and why the default is the weaker one
///
/// [`Serpentine::SumParity`] is what `TableGrid::region_nodes` does: axis `d` is
/// reversed iff the sum of the enumerated traversal positions of axes `0..d` is odd.
/// It is the DEFAULT because the committed 3-body tables were built with it and they
/// are gated on bit-identity — a better traversal is still a different table.
///
/// It is **not** unconditionally adjacent, which the comment it inherits claims it is.
/// See [`NdGrid::adjacency_is_guaranteed`]: the sum rule only reproduces the true
/// boustrophedon when every axis strictly between the first and the last has ODD
/// region extents. The production region shape `[2, 2, 2]` does not, and its traversal
/// takes a distance-2 step at every `i`-plane fold. That is a real (small) warm-start
/// cost in the existing tables, and it is recorded rather than silently repaired.
///
/// [`Serpentine::Reflected`] is the correct rule — axis `d` is reversed iff the ORDINAL
/// of the `d`-slab is odd, i.e. iff the mixed-radix number formed by the traversal
/// positions of axes `0..d` is odd. It is adjacent for any extents, and it agrees with
/// the sum rule exactly when the sum rule is adjacent. New surfaces should use it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Serpentine {
    /// The rule `TableGrid` uses. Adjacent only under the parity condition.
    SumParity,
    /// The true reflected boustrophedon. Adjacent unconditionally.
    Reflected,
}

/// A box in `d` coordinates cut into canonical regions — the dimension-generic form of
/// [`TableGrid`].
///
/// Index order is axis 0 slowest, last axis fastest, matching `((i*ny + j)*nu + k)`.
#[derive(Clone, Debug, PartialEq)]
pub struct NdGrid {
    pub axes: Vec<Axis>,
    pub serpentine: Serpentine,
}

impl NdGrid {
    /// A grid over the given axes, traversed by the legacy [`Serpentine::SumParity`]
    /// rule so that a 3-axis linear grid is bit-identical to the [`TableGrid`] it folds.
    ///
    /// # Panics
    ///
    /// On no axes, a zero extent, a zero region edge, a degenerate box, or a node count
    /// that does not fit in [`NodeId`].
    pub fn new(axes: Vec<Axis>) -> NdGrid {
        assert!(!axes.is_empty(), "a grid with no axes has no nodes to shard");
        for (d, a) in axes.iter().enumerate() {
            assert!(a.n > 0, "axis {d}: a grid extent of zero has no nodes to shard");
            assert!(
                a.region > 0,
                "axis {d}: a region edge of zero would put every node in its own region \
                 and turn the whole table cold while still producing a correct-looking \
                 result"
            );
            assert!(
                a.hi > a.lo,
                "axis {d}: the grid box must be non-degenerate in every coordinate"
            );
        }
        let g = NdGrid { axes, serpentine: Serpentine::SumParity };
        // THE OVERFLOW GUARD. `NodeId` is deliberately still `u32` (see the type's own
        // doc), and six axes can multiply past it in a way three never could. A silent
        // wrap here would alias two nodes onto one slot, which the partition assert would
        // report as "solved twice" from a place that cannot explain it.
        let mut n: usize = 1;
        for a in &g.axes {
            n = n
                .checked_mul(a.n)
                .expect("the grid's node count overflowed usize");
        }
        assert!(
            n <= u32::MAX as usize,
            "this grid has {n} nodes, which does not fit in a u32 NodeId (max {}). \
             Widening NodeId would change `GenOutcome::table_bytes`'s layout and so the \
             bit-identity comparison every committed table is gated on; a grid this large \
             is {n} electronic-structure solves and is not the thing to widen it for.",
            u32::MAX
        );
        g
    }

    /// The same grid, traversed by the true reflected boustrophedon.
    pub fn with_serpentine(mut self, s: Serpentine) -> NdGrid {
        self.serpentine = s;
        self
    }

    /// The 3-axis linear grid a [`TableGrid`] describes.
    ///
    /// Bit-identical on every canonical function — asserted node by node in
    /// `tests/nd_bit_identity.rs`, which is the acceptance argument for the fold.
    pub fn from_table_grid(g: &TableGrid) -> NdGrid {
        NdGrid::new(vec![
            Axis::linear(g.nx, g.x_lo, g.x_hi, g.region[0]),
            Axis::linear(g.ny, g.y_lo, g.y_hi, g.region[1]),
            Axis::linear(g.nu, g.u_lo, g.u_hi, g.region[2]),
        ])
    }

    /// How many coordinates a node has.
    pub fn dim(&self) -> usize {
        self.axes.len()
    }

    /// Total nodes.
    pub fn n_nodes(&self) -> usize {
        self.axes.iter().map(|a| a.n).product()
    }

    /// The canonical linear index of a coordinate tuple, axis 0 slowest.
    pub fn node_id(&self, c: &[usize]) -> NodeId {
        debug_assert_eq!(c.len(), self.axes.len());
        let mut id = 0usize;
        for (a, &ci) in self.axes.iter().zip(c.iter()) {
            debug_assert!(ci < a.n);
            id = id * a.n + ci;
        }
        id as NodeId
    }

    /// The coordinate tuple of a canonical index.
    pub fn coords(&self, id: NodeId) -> Vec<usize> {
        let mut rem = id as usize;
        let mut out = vec![0usize; self.axes.len()];
        for d in (0..self.axes.len()).rev() {
            out[d] = rem % self.axes[d].n;
            rem /= self.axes[d].n;
        }
        out
    }

    /// The physical coordinates of a node.
    pub fn geometry(&self, id: NodeId) -> Vec<f64> {
        let c = self.coords(id);
        self.axes
            .iter()
            .zip(c.iter())
            .map(|(a, &i)| a.coord(i))
            .collect()
    }

    /// How many regions the grid divides into along each axis.
    pub fn region_extents(&self) -> Vec<usize> {
        self.axes.iter().map(|a| a.n.div_ceil(a.region)).collect()
    }

    /// Total regions — the number of independent work units, and NOT a function of the
    /// worker count.
    pub fn n_regions(&self) -> usize {
        self.region_extents().iter().product()
    }

    /// Which region a node belongs to. A pure function of the node and the grid.
    pub fn region_of(&self, id: NodeId) -> RegionId {
        let c = self.coords(id);
        let rext = self.region_extents();
        let mut r = 0usize;
        for d in 0..self.axes.len() {
            r = r * rext[d] + c[d] / self.axes[d].region;
        }
        r as RegionId
    }

    /// The region's corner and extent along every axis.
    fn region_box(&self, r: RegionId) -> (Vec<usize>, Vec<usize>) {
        let rext = self.region_extents();
        let d = self.axes.len();
        let mut rem = r as usize;
        let mut rc = vec![0usize; d];
        for a in (0..d).rev() {
            rc[a] = rem % rext[a];
            rem /= rext[a];
        }
        let mut lo = vec![0usize; d];
        let mut len = vec![0usize; d];
        for a in 0..d {
            lo[a] = rc[a] * self.axes[a].region;
            len[a] = (lo[a] + self.axes[a].region).min(self.axes[a].n) - lo[a];
        }
        (lo, len)
    }

    /// Whether axis `axis` is walked backwards, given the traversal positions `p` of the
    /// axes before it and the region's extents `len`.
    ///
    /// Axis 0 is never reversed under either rule (both reduce to an empty product).
    #[inline]
    fn axis_reversed(&self, axis: usize, p: &[usize], len: &[usize]) -> bool {
        match self.serpentine {
            // The legacy rule: the PARITY OF THE SUM of the enumerated traversal
            // positions. In 3-D this is `ii` for axis 1 and `ii + jj` for axis 2 —
            // `TableGrid::region_nodes`, character for character.
            Serpentine::SumParity => p[..axis].iter().sum::<usize>() % 2 == 1,
            // The true rule: the parity of the slab's ORDINAL, i.e. of the mixed-radix
            // number with digits `p[0..axis]` and radices `len[0..axis]`. Reduced mod 2
            // as it is built, so nothing can overflow.
            Serpentine::Reflected => {
                let mut ord = 0usize;
                for e in 0..axis {
                    ord = (ord * len[e] + p[e]) % 2;
                }
                ord == 1
            }
        }
    }

    /// The nodes of one region, **in canonical traversal order** — the generalised
    /// serpentine.
    ///
    /// Axis `d` is traversed reversed iff [`NdGrid::axis_reversed`] says so; the odometer
    /// runs axis 0 slowest and the last axis fastest, exactly as the index order does.
    /// See [`Serpentine`] for what each rule guarantees.
    pub fn region_nodes(&self, r: RegionId) -> Vec<NodeId> {
        let d = self.axes.len();
        let (lo, len) = self.region_box(r);
        let total: usize = len.iter().product();
        let mut out = Vec::with_capacity(total);
        let mut p = vec![0usize; d];
        let mut idx = vec![0usize; d];
        loop {
            for a in 0..d {
                idx[a] = if self.axis_reversed(a, &p, &len) {
                    lo[a] + len[a] - 1 - p[a]
                } else {
                    lo[a] + p[a]
                };
            }
            out.push(self.node_id(&idx));
            // Increment the odometer, last axis fastest.
            let mut a = d;
            loop {
                if a == 0 {
                    return out;
                }
                a -= 1;
                p[a] += 1;
                if p[a] < len[a] {
                    break;
                }
                p[a] = 0;
            }
        }
    }

    /// Every region's node list, region index ascending. The whole work partition.
    pub fn partition(&self) -> Vec<Vec<NodeId>> {
        (0..self.n_regions() as RegionId)
            .map(|r| self.region_nodes(r))
            .collect()
    }

    /// Whether **every** region's traversal is guaranteed to step only between
    /// grid-adjacent nodes.
    ///
    /// [`Serpentine::Reflected`] always is. [`Serpentine::SumParity`] is adjacent iff
    /// every axis STRICTLY BETWEEN the first and the last has odd region extents —
    /// including the short last region, when the edge does not divide the axis.
    ///
    /// The derivation, in one line: on a carry into axis `d`, axis `e > d` must have its
    /// reversal flag flip to keep its index still, and under the sum rule that flag's
    /// parity moves by `1 + sum_{d < f < e} (len[f] - 1)`, which is odd iff every
    /// intervening `len[f]` is odd. The true rule moves it by exactly 1 always.
    ///
    /// This is REPORTED rather than enforced, because the 3-body tables were built under
    /// a region shape that fails it and they are gated on bit-identity.
    pub fn adjacency_is_guaranteed(&self) -> bool {
        if self.serpentine == Serpentine::Reflected {
            return true;
        }
        let d = self.axes.len();
        if d < 3 {
            return true;
        }
        let rext = self.region_extents();
        (1..d - 1).all(|f| {
            let a = &self.axes[f];
            let full_ok = rext[f] < 2 || a.region % 2 == 1;
            let last = a.n - (rext[f] - 1) * a.region;
            full_ok && last % 2 == 1
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(nx: usize, ny: usize, nu: usize, region: [usize; 3]) -> TableGrid {
        TableGrid::new(nx, ny, nu, region, (1.0, 3.0), (1.0, 3.0), (-0.5, 0.5))
    }

    /// The partition must be a partition: every node exactly once, no node twice.
    ///
    /// This is the property the digest's conviction rests on — a dropped node and a
    /// duplicated node are both corruptions the digest is supposed to catch, so the
    /// partition itself must not be producing them.
    #[test]
    fn partition_covers_every_node_exactly_once() {
        for dims in [(1, 1, 1), (4, 4, 3), (5, 3, 7), (9, 2, 2), (3, 3, 3)] {
            for region in [[1, 1, 1], [2, 2, 2], [3, 1, 2], [4, 4, 4], [9, 9, 9]] {
                let g = grid(dims.0, dims.1, dims.2, region);
                let mut seen = vec![0u32; g.n_nodes()];
                for r in 0..g.n_regions() as RegionId {
                    for n in g.region_nodes(r) {
                        seen[n as usize] += 1;
                        assert_eq!(
                            g.region_of(n),
                            r,
                            "node {n} was listed in region {r} but region_of says otherwise"
                        );
                    }
                }
                assert!(
                    seen.iter().all(|&c| c == 1),
                    "dims {dims:?} region {region:?}: partition is not a partition ({} \
                     nodes covered != once)",
                    seen.iter().filter(|&&c| c != 1).count()
                );
            }
        }
    }

    /// Consecutive nodes in a traversal are grid-adjacent — the property that makes the
    /// warm start a good guess, and the whole reason for the serpentine walk.
    #[test]
    fn traversal_steps_are_adjacent() {
        let g = grid(6, 5, 4, [3, 5, 4]);
        for r in 0..g.n_regions() as RegionId {
            let nodes = g.region_nodes(r);
            for w in nodes.windows(2) {
                let a = g.coords(w[0]);
                let b = g.coords(w[1]);
                let d = a.0.abs_diff(b.0) + a.1.abs_diff(b.1) + a.2.abs_diff(b.2);
                assert_eq!(
                    d, 1,
                    "traversal step {:?} -> {:?} is not adjacent (distance {d}); every \
                     node but the seed would not be warm-started from its neighbour",
                    a, b
                );
            }
        }
    }

    /// The decomposition does not know the worker count — stated as a test because it is
    /// the load-bearing property and it would be easy to break by adding a parameter.
    #[test]
    fn partition_is_independent_of_everything_but_the_grid() {
        let g = grid(7, 4, 5, [3, 2, 2]);
        let a = g.partition();
        let b = g.partition();
        assert_eq!(a, b, "the partition is not a pure function of the grid");
    }
}
