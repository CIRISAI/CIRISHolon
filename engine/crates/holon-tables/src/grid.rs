//! The node set, the region decomposition, and the traversal — all three canonical
//! functions of the grid alone.
//!
//! This module is where G1's bit-identity actually comes from. Nothing here knows how many
//! workers exist, and that is the point: if the partition or the traversal could see the
//! worker count, the warm-start chain would change with it, and the measured `3.4e-13` to
//! `4.3e-12` hartree warm-vs-cold spread would become a difference between a 1-worker and
//! a 32-worker table.

/// A node's canonical index into the whole table. Stable, worker-count-independent.
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
    /// A serpentine (boustrophedon) walk reverses every other row, so **consecutive nodes
    /// in the traversal are always grid-adjacent**. It is just as canonical — it is a
    /// function of the region's coordinates and nothing else — and it is strictly the
    /// better guess everywhere the lexicographic walk differs.
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
            // Reverse j on every other i-plane, and k on every other (i, j) row: that is
            // what makes every consecutive pair adjacent, including across the folds.
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
