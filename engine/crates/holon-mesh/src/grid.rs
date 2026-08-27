//! The scene's geometry, and the one combinatorial fact the whole gate rests on.
//!
//! A scene is a `w × h × d` grid of cells at ONE tier — one declared `g0`, no re-root anywhere
//! (`MESH_DESIGN.md` §1, and `CIRISOntology/Core/GrainFloor.lean`'s
//! `cert_does_not_transport_across_reroot` is why that is a fence rather than a preference).
//! **`d = 1` is the 2D scene**: it is the same object with one axis of thickness one, not a
//! separate code path, which is what lets the 2D gate and the 3D gate be literally the same
//! test.
//!
//! # The fact: an edge colour is a PERFECT MATCHING, and that is what makes sharding free
//!
//! Cell adjacency is the face stencil, radius `r = 1` — four neighbours in 2D, six in 3D. Its
//! edges partition by `(axis, parity of the lower endpoint's coordinate on that axis)`:
//!
//! ```text
//!   colour 0: x-edges (x,y,z)-(x+1,y,z) with x even      colour 1: same, x odd
//!   colour 2: y-edges (x,y,z)-(x,y+1,z) with y even      colour 3: same, y odd
//!   colour 4: z-edges (x,y,z)-(x,y,z+1) with z even      colour 5: same, z odd
//! ```
//!
//! **Δ = 6 in 3D and the decomposition uses exactly 6 colours, meeting Vizing's lower bound
//! exactly** — `MESH_DESIGN.md` §5.1 predicted this and
//! [`tests::three_d_edges_decompose_into_exactly_six_perfect_matchings`] measures it. In 2D
//! the z-colours are empty and the count degenerates to 4, which is the same statement at
//! `d = 1`.
//!
//! **Within one colour, every cell appears in at most one edge.** Proved by
//! [`tests::each_colour_is_a_perfect_matching`] over every cell of several grids in both 2D
//! and 3D, and it is the load-bearing property of this design, because it yields both halves
//! of the gate at once:
//!
//! * **applying a colour's edges in any order gives the same answer** — edge `e`'s two writes
//!   touch no cell edge `f` reads, so there is no read-after-write hazard to order;
//! * **partitioning the grid into shards cannot change the answer** — an edge's plan is a
//!   function of its two endpoints' pre-colour values, and which shard owns them is not an
//!   input to that function.
//!
//! The second point is the whole reason `meshed == unsharded` is achievable bit-for-bit
//! rather than approximately. It is also a real constraint the mesh imposes on any solver
//! that wants to ride it, and `MESH_DESIGN.md` states it: a sweep whose writes are visible to
//! later reads *within the same phase* (a true Gauss-Seidel) is not shardable bit-identically,
//! because then shard boundaries become physically visible. Red/black is fine — each colour is
//! a Jacobi phase — and that is what this module encodes.

/// Number of edge colours for the face stencil: three axes × two parities. Δ of the 3D grid
/// graph, and the number of perfect matchings its edges decompose into. In a `d = 1` scene the
/// last two are empty.
pub const EDGE_COLOURS: usize = 6;

/// Interaction radius of the stencil, in cells. Every dependence bound in this crate is
/// `n * RADIUS` for `n` sub-steps, which is `Core/Locality.lean::iterate_depends_within`.
pub const RADIUS: usize = 1;

/// A `w × h × d` single-tier scene. Cell `(x, y, z)` has global index `(z * h + y) * w + x`,
/// and global index order is the canonical order everything in this crate falls back to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    pub w: usize,
    pub h: usize,
    pub d: usize,
}

impl Grid {
    /// A flat scene: one cell thick in `z`. The 2D case, expressed as the 3D one.
    pub const fn new(w: usize, h: usize) -> Self {
        Self { w, h, d: 1 }
    }

    pub const fn new_3d(w: usize, h: usize, d: usize) -> Self {
        Self { w, h, d }
    }

    pub const fn len(&self) -> usize {
        self.w * self.h * self.d
    }

    pub const fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0 || self.d == 0
    }

    /// True when the scene has real extent on all three axes.
    pub const fn is_3d(&self) -> bool {
        self.d > 1
    }

    #[inline]
    pub const fn index(&self, x: usize, y: usize, z: usize) -> u32 {
        ((z * self.h + y) * self.w + x) as u32
    }

    #[inline]
    pub const fn coord(&self, index: u32) -> (usize, usize, usize) {
        let i = index as usize;
        let plane = self.w * self.h;
        (i % self.w, (i % plane) / self.w, i / plane)
    }

    /// Extent along one axis.
    pub const fn extent(&self, axis: usize) -> usize {
        match axis {
            0 => self.w,
            1 => self.h,
            _ => self.d,
        }
    }
}

/// One adjacency, oriented `lo < hi` by GLOBAL CELL INDEX — never by visit order, never by
/// which shard was reached first. The orientation fixes the transfer's sign convention.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub lo: u32,
    pub hi: u32,
}

impl Edge {
    fn new(a: u32, b: u32) -> Self {
        if a < b {
            Self { lo: a, hi: b }
        } else {
            Self { lo: b, hi: a }
        }
    }
}

/// Every edge of one colour, in ascending `(lo, hi)` order.
///
/// `colour` is `axis * 2 + parity`: 0/1 are the x matchings, 2/3 the y, 4/5 the z.
pub fn edges_of_colour(grid: Grid, colour: usize) -> Vec<Edge> {
    let mut edges = Vec::new();
    if grid.is_empty() || colour >= EDGE_COLOURS {
        return edges;
    }
    let axis = colour / 2;
    let parity = colour % 2;
    // Walk the two axes that are not the edge's axis, and step the edge's axis by two from
    // its parity so that every cell is in at most one edge of this colour.
    let (a0, a1) = match axis {
        0 => (1usize, 2usize),
        1 => (0, 2),
        _ => (0, 1),
    };
    let mut t = parity;
    while t + 1 < grid.extent(axis) {
        for u in 0..grid.extent(a0) {
            for v in 0..grid.extent(a1) {
                let mut c = [0usize; 3];
                c[axis] = t;
                c[a0] = u;
                c[a1] = v;
                let lo = grid.index(c[0], c[1], c[2]);
                c[axis] = t + 1;
                let hi = grid.index(c[0], c[1], c[2]);
                edges.push(Edge::new(lo, hi));
            }
        }
        t += 2;
    }
    edges.sort_unstable_by_key(|e| (e.lo, e.hi));
    edges
}

/// The six face neighbours of a cell, clipped at the scene's boundary.
pub fn neighbours(grid: Grid, cell: u32) -> Vec<u32> {
    let (x, y, z) = grid.coord(cell);
    let mut out = Vec::with_capacity(6);
    if x + 1 < grid.w {
        out.push(grid.index(x + 1, y, z));
    }
    if x > 0 {
        out.push(grid.index(x - 1, y, z));
    }
    if y + 1 < grid.h {
        out.push(grid.index(x, y + 1, z));
    }
    if y > 0 {
        out.push(grid.index(x, y - 1, z));
    }
    if z + 1 < grid.d {
        out.push(grid.index(x, y, z + 1));
    }
    if z > 0 {
        out.push(grid.index(x, y, z - 1));
    }
    out
}

/// A partition of the grid into rectangular-block shards, and the map from cell to owner.
///
/// The shard shape is an axis-aligned block, which is `MESH_DESIGN.md` §1's "a shard is a
/// complete set of sub-octrees at one level". Blocks are cut on all three axes, so shard
/// counts are `nx * ny * nz`.
#[derive(Clone, Debug)]
pub struct Partition {
    grid: Grid,
    owner: Vec<u32>,
    owned: Vec<Vec<u32>>,
    slot: Vec<u32>,
}

impl Partition {
    pub fn blocks(grid: Grid, nx: usize, ny: usize, nz: usize) -> Self {
        let nx = nx.max(1).min(grid.w.max(1));
        let ny = ny.max(1).min(grid.h.max(1));
        let nz = nz.max(1).min(grid.d.max(1));
        let mut owner = vec![0u32; grid.len()];
        for z in 0..grid.d {
            let bz = z * nz / grid.d;
            for y in 0..grid.h {
                let by = y * ny / grid.h;
                for x in 0..grid.w {
                    let bx = x * nx / grid.w;
                    owner[grid.index(x, y, z) as usize] = ((bz * ny + by) * nx + bx) as u32;
                }
            }
        }
        // Fill `owned` in ascending global index so a shard's local ids are a deterministic
        // function of the partition alone, never of iteration order.
        let mut owned = vec![Vec::new(); nx * ny * nz];
        let mut slot = vec![0u32; grid.len()];
        for cell in 0..grid.len() as u32 {
            let shard = owner[cell as usize] as usize;
            slot[cell as usize] = owned[shard].len() as u32;
            owned[shard].push(cell);
        }
        Self {
            grid,
            owner,
            owned,
            slot,
        }
    }

    /// One shard holding the whole grid — the degenerate partition, used as a control.
    pub fn single(grid: Grid) -> Self {
        Self::blocks(grid, 1, 1, 1)
    }

    pub fn grid(&self) -> Grid {
        self.grid
    }

    pub fn shard_count(&self) -> usize {
        self.owned.len()
    }

    #[inline]
    pub fn owner(&self, cell: u32) -> u32 {
        self.owner[cell as usize]
    }

    /// Local holon id of a cell inside its owning shard's arena. Arena id 0 is the root, so
    /// owned cell `k` is holon `k + 1`.
    #[inline]
    pub fn local_id(&self, cell: u32) -> u32 {
        self.slot[cell as usize] + 1
    }

    pub fn owned(&self, shard: usize) -> &[u32] {
        &self.owned[shard]
    }

    pub fn cross_edges(&self, colour: usize) -> Vec<Edge> {
        edges_of_colour(self.grid, colour)
            .into_iter()
            .filter(|e| self.owner(e.lo) != self.owner(e.hi))
            .collect()
    }

    pub fn interior_edges(&self, colour: usize) -> Vec<Edge> {
        edges_of_colour(self.grid, colour)
            .into_iter()
            .filter(|e| self.owner(e.lo) == self.owner(e.hi))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    const FLAT: [(usize, usize, usize); 6] = [
        (1, 1, 1),
        (2, 3, 1),
        (5, 5, 1),
        (8, 4, 1),
        (7, 9, 1),
        (16, 16, 1),
    ];
    const SOLID: [(usize, usize, usize); 5] = [
        (2, 2, 2),
        (3, 3, 3),
        (4, 5, 6),
        (8, 4, 2),
        (5, 7, 3),
    ];

    /// **The load-bearing property, in both dimensions.** Within one colour every cell is in
    /// at most one edge. Everything else in this crate — order-freedom of the apply phase, and
    /// the claim that partitioning cannot change the answer — is downstream of this.
    #[test]
    fn each_colour_is_a_perfect_matching() {
        for (w, h, d) in FLAT.iter().chain(SOLID.iter()) {
            let grid = Grid::new_3d(*w, *h, *d);
            for colour in 0..EDGE_COLOURS {
                let mut seen = BTreeSet::new();
                for edge in edges_of_colour(grid, colour) {
                    assert!(
                        seen.insert(edge.lo),
                        "{w}x{h}x{d} colour {colour}: cell {} is in two edges",
                        edge.lo
                    );
                    assert!(
                        seen.insert(edge.hi),
                        "{w}x{h}x{d} colour {colour}: cell {} is in two edges",
                        edge.hi
                    );
                }
            }
        }
    }

    /// The colours together are exactly the adjacency graph: nothing counted twice, nothing
    /// missed. A colouring that dropped edges would still pass the matching test while quietly
    /// making part of the scene not interact.
    #[test]
    fn the_colours_cover_every_adjacency_exactly_once() {
        for (w, h, d) in FLAT.iter().chain(SOLID.iter()) {
            let grid = Grid::new_3d(*w, *h, *d);
            let mut all = BTreeSet::new();
            let mut count = 0;
            for colour in 0..EDGE_COLOURS {
                for edge in edges_of_colour(grid, colour) {
                    assert!(all.insert((edge.lo, edge.hi)), "edge counted twice");
                    count += 1;
                }
            }
            // A w x h x d face-adjacency grid has (w-1)hd + w(h-1)d + wh(d-1) edges.
            let expected = (w - 1) * h * d + w * (h - 1) * d + w * h * (d - 1);
            assert_eq!(count, expected, "{w}x{h}x{d}: wrong edge count");

            // And the colouring agrees with the neighbour stencil the mesh actually reads.
            let mut stencil = 0;
            for cell in 0..grid.len() as u32 {
                stencil += neighbours(grid, cell).len();
            }
            assert_eq!(stencil, 2 * count, "stencil and colouring disagree");
        }
    }

    /// **`MESH_DESIGN.md` §5.1's prediction, measured.** In 3D the edges decompose into
    /// exactly SIX non-empty perfect matchings — Δ = 6, meeting Vizing's lower bound exactly,
    /// where 2D needs four. This is what sets the number of exchange sub-rounds.
    #[test]
    fn three_d_edges_decompose_into_exactly_six_perfect_matchings() {
        let grid = Grid::new_3d(6, 6, 6);
        let non_empty = (0..EDGE_COLOURS)
            .filter(|c| !edges_of_colour(grid, *c).is_empty())
            .count();
        assert_eq!(non_empty, 6, "3D should use all six colours");

        // Vizing: a simple graph needs Δ or Δ+1 colours. Δ here is 6 (an interior cell has
        // six face neighbours), so six is the LOWER bound and the construction attains it.
        let max_degree = (0..grid.len() as u32)
            .map(|c| neighbours(grid, c).len())
            .max()
            .unwrap();
        assert_eq!(max_degree, 6, "interior cells have six face neighbours");
        assert_eq!(non_empty, max_degree, "the decomposition attains Vizing's lower bound");
    }

    /// A flat scene uses four: the z-colours are empty, so 2D is the `d = 1` degeneration of
    /// the same statement rather than a separate rule.
    #[test]
    fn a_flat_scene_uses_four_colours_and_the_z_colours_are_empty() {
        let grid = Grid::new(8, 8);
        assert!(edges_of_colour(grid, 4).is_empty());
        assert!(edges_of_colour(grid, 5).is_empty());
        let non_empty = (0..EDGE_COLOURS)
            .filter(|c| !edges_of_colour(grid, *c).is_empty())
            .count();
        assert_eq!(non_empty, 4);
    }

    #[test]
    fn edges_are_oriented_by_global_index_not_construction_order() {
        let grid = Grid::new_3d(6, 5, 4);
        for colour in 0..EDGE_COLOURS {
            for edge in edges_of_colour(grid, colour) {
                assert!(edge.lo < edge.hi);
            }
        }
    }

    /// Indexing round-trips. A coord/index pair that disagreed would silently mis-place every
    /// cell in the scene while every count above still passed.
    #[test]
    fn index_and_coord_round_trip() {
        let grid = Grid::new_3d(7, 5, 3);
        for cell in 0..grid.len() as u32 {
            let (x, y, z) = grid.coord(cell);
            assert_eq!(grid.index(x, y, z), cell);
            assert!(x < grid.w && y < grid.h && z < grid.d);
        }
    }

    #[test]
    fn a_partition_owns_every_cell_exactly_once() {
        let grid = Grid::new_3d(12, 8, 6);
        for (nx, ny, nz) in [(1, 1, 1), (2, 2, 2), (3, 2, 1), (4, 4, 3), (2, 1, 6)] {
            let part = Partition::blocks(grid, nx, ny, nz);
            let mut total = 0;
            for shard in 0..part.shard_count() {
                for &cell in part.owned(shard) {
                    assert_eq!(part.owner(cell) as usize, shard);
                    total += 1;
                }
            }
            assert_eq!(total, grid.len(), "{nx}x{ny}x{nz} lost or duplicated cells");
        }
    }

    /// Interior and cross edges partition the colour's edges — no edge in neither list, none
    /// in both. The mesh steps exactly these two lists, so a gap would be a dropped
    /// interaction.
    #[test]
    fn interior_and_cross_edges_partition_the_colour() {
        let grid = Grid::new_3d(10, 10, 6);
        let part = Partition::blocks(grid, 3, 3, 2);
        let mut total_cross = 0;
        for colour in 0..EDGE_COLOURS {
            let all = edges_of_colour(grid, colour).len();
            let interior = part.interior_edges(colour).len();
            let cross = part.cross_edges(colour).len();
            assert_eq!(interior + cross, all, "colour {colour} lost edges");
            total_cross += cross;
        }
        // Not every colour need have a boundary, and demanding one was wrong: a cut lands at
        // a single coordinate, so it falls in ONE parity class. `nz = 2` on `d = 6` cuts
        // between z = 2 and z = 3, which is colour 4, leaving colour 5 with no cross edges at
        // all. That is the geometry, not a defect. What must hold is that the partition is
        // exact per colour, and that the scene has a boundary somewhere.
        assert!(total_cross > 0, "this partition has no boundary at all to test");
    }

    /// Every axis that is actually CUT contributes cross edges on at least one of its two
    /// colours. This is the check the previous test was reaching for, stated so it can pass.
    #[test]
    fn every_cut_axis_produces_a_boundary() {
        let grid = Grid::new_3d(10, 10, 6);
        let part = Partition::blocks(grid, 3, 3, 2);
        for axis in 0..3 {
            let cross: usize = (0..2)
                .map(|parity| part.cross_edges(axis * 2 + parity).len())
                .sum();
            assert!(cross > 0, "axis {axis} is cut but produced no boundary");
        }
    }

    /// **The 3D surface-to-volume penalty, measured rather than asserted.**
    ///
    /// `MESH_DESIGN.md` §2 predicted that at equal holon count a 3D shard pays relatively more
    /// boundary than a 2D one, because boundary scales as `M^(2/3)` in 3D against `M^(1/2)` in
    /// 2D. Measured here on shards of ~equal cell count: the cross-edge fraction really is
    /// several times larger in 3D. This is the cost the occlusion argument of §0 buys back —
    /// a surface-dominated resident set makes the mesh's exchange scale 2D-like even in a 3D
    /// scene.
    #[test]
    fn a_three_d_shard_pays_more_boundary_than_a_two_d_shard_of_the_same_size() {
        // 4096 cells per shard either way: 64x64 flat in 8x8 blocks, 16^3 solid in 2x2x2.
        let flat = Grid::new(64 * 8, 64 * 8);
        let flat_part = Partition::blocks(flat, 8, 8, 1);
        let solid = Grid::new_3d(16 * 2, 16 * 2, 16 * 2);
        let solid_part = Partition::blocks(solid, 2, 2, 2);

        let fraction = |grid: Grid, part: &Partition| -> f64 {
            let (mut cross, mut all) = (0usize, 0usize);
            for colour in 0..EDGE_COLOURS {
                all += edges_of_colour(grid, colour).len();
                cross += part.cross_edges(colour).len();
            }
            cross as f64 / all as f64
        };
        let flat_f = fraction(flat, &flat_part);
        let solid_f = fraction(solid, &solid_part);
        assert!(
            solid_f > 2.0 * flat_f,
            "3D boundary fraction {solid_f:.4} should far exceed 2D's {flat_f:.4} at equal \
             shard size; if this ever stops holding, §2's surface-to-volume argument is wrong"
        );
    }
}
