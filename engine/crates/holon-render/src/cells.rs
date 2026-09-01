//! THE CELL LIST: the scene bucketed by position, so every interaction loop is
//! cutoff-local instead of global.
//!
//! T3 scale-up (FSD-W1 §10, "OWED"). Before this module the engine enumerated every pair
//! (`N²/2`), every triple (`N³/6`) and every quadruple (`N⁴/24`) on every force
//! evaluation. At the sixteen atoms the old `MAX_ATOMS` cap allowed, that is 120 pairs,
//! 560 triples and 1,820 quadruples and nobody notices. At the thousands the workbench
//! needs it is 5·10⁵, 1.7·10⁸ and 4·10¹⁰ — the quadruple loop alone would take longer per
//! frame than the campaign has run in total.
//!
//! # The two halves of locality, and only one of them is free
//!
//! **The many-body sectors are exactly zero outside their tables' domains.**
//! `TrimerTable::eval` returns `(0, [0;3])` when the middle side exceeds `R_HI`;
//! `WaterTable`, `OohTable` and `OzoneTable` return the same outside theirs; the
//! four-body sector's own `R_CUT` switch is already in the engine. So skipping a distant
//! triple does not approximate it — it computes the same exact zero without paying for
//! the interpolant. **The three- and four-body loops are therefore cutoff-local
//! unconditionally, and the numbers do not move.** That is the free half.
//!
//! **The pair curve has no such radius.** Past its last knot `PotentialTable::eval`
//! continues as `hi_a·exp(-hi_b·dr)`, which is small but never zero, so a pair cutoff is a
//! TRUNCATION of the physics rather than a discovery about it. It is therefore opt-in
//! ([`Sim::set_pair_cutoff`](crate::sim::Sim::set_pair_cutoff)), it applies the same C²
//! quintic switch the four-body sector already uses so that the truncated potential is
//! still a potential and the energy gate stays exact, and the energy it drops is reported
//! rather than assumed negligible. A scene that declares no cutoff enumerates every pair
//! and is bit-for-bit the engine that existed before this module.
//!
//! # Canonical order, which is not a nicety
//!
//! The neighbour list is emitted in ascending `(i, j)` — the same order the complete
//! double loop produced — and the triple and quadruple enumerations are sorted into their
//! own ascending index order before evaluation. Floating-point addition is not
//! associative, so the order of the force and energy sums IS part of the answer. Fixing
//! it canonically is what makes three separate claims true at once: a cutoff-local run
//! agrees bit-for-bit with the complete run on the terms they share; a checkpoint replays
//! bit-identically; and a run sharded across workers cannot depend on how many there were
//! (the same argument `holon-tables` makes for the table mesh, one level down).
//!
//! M-CHEAPER-THAN-ITS-PRICE: the sort is `O(P log d)` over per-atom lists of length `d`,
//! the local density, against interpolant evaluations that cost hundreds of flops each.

use crate::sim::Atom;

/// Which enumeration the force loops actually ran.
///
/// Reported, not inferred: "the cell list was used" and "the cell list was USEFUL" are
/// different facts, and a scene whose box is smaller than three cutoffs across gets the
/// complete loop because a cell decomposition of it would visit every cell anyway.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Route {
    /// Every pair enumerated. Bit-for-bit the pre-T3 engine.
    Complete,
    /// Cells of edge at least the cutoff, 27-cell stencil.
    Cells,
}

/// Which route the caller will ACCEPT, as opposed to which one the geometry suggests.
///
/// [`RoutePolicy::Complete`] exists for the same reason `holon-mesh` keeps
/// `run_sequential` beside `run_threaded` and compares them: the complete enumeration is
/// the REFERENCE, and a fast route that nothing can be held against is a fast route
/// nobody has checked. It is a declared control, not a debug flag — `tests/t3_scale.rs`
/// runs one configuration down both and requires the energies and forces to agree bit for
/// bit.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RoutePolicy {
    /// Cells when the geometry admits them, complete otherwise.
    #[default]
    Auto,
    /// The complete enumeration, whatever the geometry admits.
    Complete,
}

/// The box the separations are measured in, and whether it wraps.
///
/// Carried as a small value rather than read off `Sim` so that the minimum-image
/// convention has EXACTLY ONE implementation ([`BoxGeom::delta`]) that the pair loop, the
/// triple loop, the quadruple loop, the bond reading and the cell list all call. The
/// alternative — each loop subtracting its own coordinates — is how three of them come to
/// disagree about where an atom is.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BoxGeom {
    pub lx: f64,
    pub ly: f64,
    pub lz: f64,
    pub periodic: bool,
}

impl BoxGeom {
    #[inline]
    pub fn new(lx: f64, ly: f64, lz: f64, periodic: bool) -> Self {
        Self {
            lx,
            ly,
            lz,
            periodic,
        }
    }

    /// The shortest box edge — the quantity the cutoff must stay under half of.
    #[inline]
    pub fn min_edge(&self) -> f64 {
        self.lx.min(self.ly).min(self.lz)
    }

    /// THE MINIMUM-IMAGE CONVENTION, and the only place it lives.
    ///
    /// `b - a`, reduced into `[-L/2, L/2)` on each periodic axis. Non-periodic returns the
    /// raw difference, which is the exact float the engine has always computed — the
    /// `periodic` branch is not taken and no bit of the open-box arithmetic changes.
    ///
    /// The reduction is `d - L·round(d/L)`. `round` (half away from zero) rather than a
    /// `while` loop because the loop's cost depends on how far the atom has wandered,
    /// which would make the force loop's timing a function of the trajectory; and rather
    /// than `rem_euclid`, which would put the result in `[0, L)` and need a second
    /// correction. On an atom inside the box `d/L` is in `(-1, 1)`, `round` gives
    /// `-1`, `0` or `1`, and the correction is one multiply-subtract.
    ///
    /// VALIDITY: this is the minimum image only while the cutoff is at most `L/2` — past
    /// that, two images of the same partner are inside the cutoff and there is no such
    /// thing as "the" image. `Sim::pbc_ok` states that condition and refuses rather than
    /// letting the reduction lie.
    #[inline]
    pub fn delta(&self, a: (f64, f64, f64), b: (f64, f64, f64)) -> (f64, f64, f64) {
        let mut dx = b.0 - a.0;
        let mut dy = b.1 - a.1;
        let mut dz = b.2 - a.2;
        if self.periodic {
            dx -= self.lx * (dx / self.lx).round();
            dy -= self.ly * (dy / self.ly).round();
            dz -= self.lz * (dz / self.lz).round();
        }
        (dx, dy, dz)
    }

    /// Fold a coordinate back into `[0, L)`.
    ///
    /// Used on the DRIFT step so an atom that crosses a face re-enters through the
    /// opposite one. The subtraction is exact for a coordinate at most one box outside
    /// (Sterbenz: `x - L` is exactly representable for `L/2 ≤ x ≤ 2L`), which is every
    /// coordinate a stable timestep can produce; the loop is there for the pathological
    /// case rather than the normal one.
    #[inline]
    pub fn wrap1(x: f64, l: f64) -> f64 {
        if !(l > 0.0) || !x.is_finite() {
            return x;
        }
        let mut x = x;
        while x >= l {
            x -= l;
        }
        while x < 0.0 {
            x += l;
        }
        x
    }

    /// [`BoxGeom::wrap1`] on all three axes. A no-op when the box does not wrap.
    #[inline]
    pub fn wrap(&self, p: (f64, f64, f64)) -> (f64, f64, f64) {
        if !self.periodic {
            return p;
        }
        (
            Self::wrap1(p.0, self.lx),
            Self::wrap1(p.1, self.ly),
            Self::wrap1(p.2, self.lz),
        )
    }
}

/// One neighbour pair, with the displacement already reduced.
///
/// The displacement is carried rather than recomputed because under periodic boundaries
/// `b - a` is NOT the separation and a loop that recomputes it naively gets a different
/// (wrong) answer. Storing it means the minimum image is applied once, in one place.
#[derive(Clone, Copy, Debug, Default)]
pub struct NeighbourPair {
    pub i: u32,
    pub j: u32,
    /// `atoms[j] - atoms[i]`, minimum-imaged.
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
    /// `|d|`, floored at `1e-9` exactly as the pre-T3 loops floored it.
    pub r: f64,
}

/// The neighbour pairs of one force evaluation, in ascending `(i, j)`.
pub struct Neighbours {
    pub pairs: Vec<NeighbourPair>,
    /// `start[i]..start[i + 1]` are the pairs whose FIRST atom is `i`. Length `n + 1`.
    pub start: Vec<u32>,
    /// THE SYMMETRIC ADJACENCY: `adj[adj_start[i]..adj_start[i+1]]` is EVERY neighbour of
    /// `i`, of lower and higher index alike, ascending.
    ///
    /// `pairs` holds each pair once, under its lower index, which is what a pair loop
    /// wants and what a HUB loop cannot use. The three-body enumeration asks "which atoms
    /// does `h` see?" and the answer is not "the ones with a bigger index" — a triple
    /// whose only qualifying vertex is its highest-indexed atom would be dropped, silently
    /// and only sometimes. So the reverse direction is materialised rather than searched
    /// for.
    pub adj: Vec<u32>,
    pub adj_start: Vec<u32>,
    /// `|r|` alongside `adj`, so a hub loop does not recompute a separation the pair loop
    /// already has.
    pub adj_r: Vec<f64>,
    /// The radius the list was built at.
    pub cutoff: f64,
    /// Whether every pair in the scene is in the list — true on the complete route, and
    /// true on the cell route only if no pair was actually dropped.
    pub complete: bool,
    pub route: Route,
}

impl Default for Neighbours {
    fn default() -> Self {
        Self::empty()
    }
}

impl Default for CellList {
    fn default() -> Self {
        Self::empty()
    }
}

impl Neighbours {
    pub const fn empty() -> Self {
        Self {
            pairs: Vec::new(),
            start: Vec::new(),
            adj: Vec::new(),
            adj_start: Vec::new(),
            adj_r: Vec::new(),
            cutoff: f64::INFINITY,
            complete: true,
            route: Route::Complete,
        }
    }

    /// Every neighbour of `i`, ascending, with its separation.
    #[inline]
    pub fn adj_of(&self, i: usize) -> (&[u32], &[f64]) {
        if i + 1 >= self.adj_start.len() {
            return (&[], &[]);
        }
        let a = self.adj_start[i] as usize;
        let b = self.adj_start[i + 1] as usize;
        (&self.adj[a..b], &self.adj_r[a..b])
    }

    /// Fill [`Neighbours::adj`] from [`Neighbours::pairs`]: counting sort, two passes, no
    /// per-atom allocation. Each pair is written twice, once under each endpoint.
    fn build_adjacency(&mut self, n: usize) {
        self.adj_start.clear();
        self.adj_start.resize(n + 1, 0);
        for p in self.pairs.iter() {
            self.adj_start[p.i as usize + 1] += 1;
            self.adj_start[p.j as usize + 1] += 1;
        }
        for i in 0..n {
            self.adj_start[i + 1] += self.adj_start[i];
        }
        let total = *self.adj_start.last().unwrap_or(&0) as usize;
        self.adj.clear();
        self.adj.resize(total, 0);
        self.adj_r.clear();
        self.adj_r.resize(total, 0.0);
        // `cursor` walks each atom's span. Reusing `adj_start` as the cursor and repairing
        // it afterwards would save the vector and cost the reader the invariant.
        let mut cursor: Vec<u32> = self.adj_start[..n].to_vec();
        for p in self.pairs.iter() {
            let (i, j) = (p.i as usize, p.j as usize);
            let a = cursor[i] as usize;
            self.adj[a] = p.j;
            self.adj_r[a] = p.r;
            cursor[i] += 1;
            let b = cursor[j] as usize;
            self.adj[b] = p.i;
            self.adj_r[b] = p.r;
            cursor[j] += 1;
        }
        // Each atom's span holds its higher-indexed neighbours in ascending order (they
        // arrived from `pairs`, which is sorted) interleaved with its lower-indexed ones,
        // which arrived in the order their own rows were written — also ascending. The two
        // runs are each sorted; the span as a whole is not, so it is sorted here. Ascending
        // adjacency is what makes the hub enumeration's `(j, k)` order canonical.
        for i in 0..n {
            let a = self.adj_start[i] as usize;
            let b = self.adj_start[i + 1] as usize;
            // Sort the two runs together, carrying the radii. Spans are the local density,
            // so an insertion sort in place is cheaper than allocating a permutation.
            for k in (a + 1)..b {
                let mut m = k;
                while m > a && self.adj[m - 1] > self.adj[m] {
                    self.adj.swap(m - 1, m);
                    self.adj_r.swap(m - 1, m);
                    m -= 1;
                }
            }
        }
    }

    /// The pairs whose first atom is `i`, i.e. its neighbours of higher index.
    #[inline]
    pub fn of(&self, i: usize) -> &[NeighbourPair] {
        if i + 1 >= self.start.len() {
            return &[];
        }
        let a = self.start[i] as usize;
        let b = self.start[i + 1] as usize;
        &self.pairs[a..b]
    }

    /// The recorded separation of the pair `(a, b)` in either order, or `None` when they
    /// are not neighbours.
    ///
    /// Binary search inside the smaller index's slice, which is sorted by construction.
    /// Used by the triple enumeration to decide which vertex is the canonical hub, so it
    /// runs once per candidate triple and has to be cheap.
    pub fn separation(&self, a: usize, b: usize) -> Option<f64> {
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        let slice = self.of(lo);
        slice
            .binary_search_by_key(&(hi as u32), |p| p.j)
            .ok()
            .map(|k| slice[k].r)
    }
}

/// Atoms bucketed into cells of edge at least the cutoff.
pub struct CellList {
    /// Cells along each axis.
    nc: [usize; 3],
    /// Cell edge lengths.
    cell: [f64; 3],
    /// The origin the cells are measured from — the box origin when periodic, the atoms'
    /// own lower bound when not, so an open scene that has drifted out of the nominal box
    /// still buckets correctly.
    origin: [f64; 3],
    geom: BoxGeom,
    cutoff: f64,
    route: Route,
    policy: RoutePolicy,
    /// `head[c]` is the first atom in cell `c`, or `u32::MAX`.
    head: Vec<u32>,
    /// `next[i]` is the next atom in `i`'s cell, or `u32::MAX`.
    next: Vec<u32>,
    /// Scratch for the per-atom neighbour gather, kept so a force evaluation allocates
    /// nothing after the first.
    scratch: Vec<NeighbourPair>,
}

const NIL: u32 = u32::MAX;

/// The fewest cells per axis the 27-cell stencil needs to visit each neighbour once.
///
/// At three cells the stencil's `-1, 0, +1` offsets are distinct even after wrapping. At
/// two they collide — `+1` and `-1` name the same cell — so the same pair would be
/// enumerated twice, which is not a rounding problem but a doubled force. Below this the
/// route falls back to the complete enumeration, which is correct at any size and is what
/// a box that small should be running anyway.
const MIN_CELLS_PER_AXIS: usize = 3;

impl CellList {
    pub const fn empty() -> Self {
        Self {
            nc: [1, 1, 1],
            cell: [0.0; 3],
            origin: [0.0; 3],
            geom: BoxGeom {
                lx: 0.0,
                ly: 0.0,
                lz: 0.0,
                periodic: false,
            },
            cutoff: f64::INFINITY,
            route: Route::Complete,
            policy: RoutePolicy::Auto,
            head: Vec::new(),
            next: Vec::new(),
            scratch: Vec::new(),
        }
    }

    pub fn route(&self) -> Route {
        self.route
    }

    pub fn policy(&self) -> RoutePolicy {
        self.policy
    }

    pub fn set_policy(&mut self, policy: RoutePolicy) {
        self.policy = policy;
    }

    pub fn cells_per_axis(&self) -> [usize; 3] {
        self.nc
    }

    /// Total cells. Reported so a caller can see the memory the decomposition costs.
    pub fn cell_count(&self) -> usize {
        self.nc[0] * self.nc[1] * self.nc[2]
    }

    #[inline]
    fn cell_of(&self, p: (f64, f64, f64)) -> usize {
        let f = |v: f64, o: f64, s: f64, n: usize| -> usize {
            if !(s > 0.0) {
                return 0;
            }
            let k = ((v - o) / s).floor();
            if !k.is_finite() {
                return 0;
            }
            // `rem_euclid` on the integer keeps a periodic scene's stragglers in range
            // without a branch per axis, and clamps rather than wraps when open.
            if self.geom.periodic {
                (k as i64).rem_euclid(n as i64) as usize
            } else {
                (k as i64).clamp(0, n as i64 - 1) as usize
            }
        };
        let ix = f(p.0, self.origin[0], self.cell[0], self.nc[0]);
        let iy = f(p.1, self.origin[1], self.cell[1], self.nc[1]);
        let iz = f(p.2, self.origin[2], self.cell[2], self.nc[2]);
        (iz * self.nc[1] + iy) * self.nc[0] + ix
    }

    /// Rebuild the decomposition for this scene, box and cutoff.
    ///
    /// Chooses the route: cells when the box admits at least [`MIN_CELLS_PER_AXIS`] on
    /// every axis at this cutoff AND the scene is big enough for the bookkeeping to be
    /// worth it, the complete enumeration otherwise.
    pub fn rebuild(&mut self, atoms: &[Atom], geom: BoxGeom, cutoff: f64) {
        let n = atoms.len();
        self.geom = geom;
        self.cutoff = cutoff;

        // The extent the cells must cover. Periodic: the box, exactly, because the wrap
        // is defined against it. Open/walled: the atoms' own bounding box, because atoms
        // are not confined to the nominal box and a decomposition that assumed they were
        // would bucket an escapee into a cell it is not in.
        let (origin, extent) = if geom.periodic {
            ([0.0, 0.0, 0.0], [geom.lx, geom.ly, geom.lz])
        } else {
            let mut lo = [f64::INFINITY; 3];
            let mut hi = [f64::NEG_INFINITY; 3];
            for a in atoms {
                let p = [a.x, a.y, a.z];
                for k in 0..3 {
                    if p[k] < lo[k] {
                        lo[k] = p[k];
                    }
                    if p[k] > hi[k] {
                        hi[k] = p[k];
                    }
                }
            }
            if !lo[0].is_finite() {
                lo = [0.0; 3];
                hi = [0.0; 3];
            }
            // A hair of margin so an atom exactly on the upper face lands in the last
            // cell rather than one past it.
            (
                lo,
                [
                    (hi[0] - lo[0]) * (1.0 + 1e-12) + 1e-12,
                    (hi[1] - lo[1]) * (1.0 + 1e-12) + 1e-12,
                    (hi[2] - lo[2]) * (1.0 + 1e-12) + 1e-12,
                ],
            )
        };

        let mut nc = [1usize; 3];
        let mut ok = cutoff.is_finite() && cutoff > 0.0 && self.policy == RoutePolicy::Auto;
        for k in 0..3 {
            let c = if ok { (extent[k] / cutoff).floor() } else { 0.0 };
            let c = if c.is_finite() && c >= 1.0 {
                (c as usize).min(1 << 20)
            } else {
                1
            };
            nc[k] = c;
            if c < MIN_CELLS_PER_AXIS {
                ok = false;
            }
        }
        // Below this the stencil visits a large fraction of the scene anyway and the
        // bookkeeping costs more than it saves. Stated as a number rather than hidden in
        // a condition; it is a COST threshold, never a correctness one — both routes
        // produce the same list.
        if n < 64 {
            ok = false;
        }
        if !ok {
            self.route = Route::Complete;
            self.nc = [1, 1, 1];
            self.head.clear();
            self.next.clear();
            return;
        }

        self.route = Route::Cells;
        self.nc = nc;
        self.origin = origin;
        for k in 0..3 {
            self.cell[k] = extent[k] / nc[k] as f64;
        }
        let cells = nc[0] * nc[1] * nc[2];
        self.head.clear();
        self.head.resize(cells, NIL);
        self.next.clear();
        self.next.resize(n, NIL);
        // Inserted in DESCENDING index so each cell's chain comes out ascending, which
        // makes the per-atom gather nearly sorted and the sort below almost free.
        for i in (0..n).rev() {
            let c = self.cell_of((atoms[i].x, atoms[i].y, atoms[i].z));
            self.next[i] = self.head[c];
            self.head[c] = i as u32;
        }
    }

    /// Build the neighbour list: every pair `(i, j)` with `i < j` and `|r_ij| <= cutoff`,
    /// in ascending `(i, j)`.
    ///
    /// On [`Route::Complete`] this is the pre-T3 double loop, in its order, with its
    /// arithmetic — including the `1e-9` floor on `r` — so an undeclared-cutoff scene
    /// gets the identical floats.
    pub fn build_neighbours(&mut self, atoms: &[Atom], out: &mut Neighbours) {
        let n = atoms.len();
        out.pairs.clear();
        out.start.clear();
        out.start.reserve(n + 1);
        out.cutoff = self.cutoff;
        out.route = self.route;
        out.complete = true;

        let geom = self.geom;
        let cut2 = if self.cutoff.is_finite() {
            self.cutoff * self.cutoff
        } else {
            f64::INFINITY
        };

        match self.route {
            Route::Complete => {
                for i in 0..n {
                    out.start.push(out.pairs.len() as u32);
                    let a = (atoms[i].x, atoms[i].y, atoms[i].z);
                    for j in (i + 1)..n {
                        let b = (atoms[j].x, atoms[j].y, atoms[j].z);
                        let (dx, dy, dz) = geom.delta(a, b);
                        let r2 = dx * dx + dy * dy + dz * dz;
                        if r2 > cut2 {
                            out.complete = false;
                            continue;
                        }
                        let r = r2.sqrt().max(1e-9);
                        out.pairs.push(NeighbourPair {
                            i: i as u32,
                            j: j as u32,
                            dx,
                            dy,
                            dz,
                            r,
                        });
                    }
                }
            }
            Route::Cells => {
                let [ncx, ncy, ncz] = self.nc;
                let mut scratch = core::mem::take(&mut self.scratch);
                for i in 0..n {
                    out.start.push(out.pairs.len() as u32);
                    scratch.clear();
                    let a = (atoms[i].x, atoms[i].y, atoms[i].z);
                    let c = self.cell_of(a);
                    let cx = c % ncx;
                    let cy = (c / ncx) % ncy;
                    let cz = c / (ncx * ncy);
                    for dz_c in -1i64..=1 {
                        let z = Self::axis(cz, dz_c, ncz, geom.periodic);
                        let Some(z) = z else { continue };
                        for dy_c in -1i64..=1 {
                            let y = Self::axis(cy, dy_c, ncy, geom.periodic);
                            let Some(y) = y else { continue };
                            for dx_c in -1i64..=1 {
                                let x = Self::axis(cx, dx_c, ncx, geom.periodic);
                                let Some(x) = x else { continue };
                                let cc = (z * ncy + y) * ncx + x;
                                let mut j = self.head[cc];
                                while j != NIL {
                                    let ju = j as usize;
                                    j = self.next[ju];
                                    if ju <= i {
                                        continue;
                                    }
                                    let b = (atoms[ju].x, atoms[ju].y, atoms[ju].z);
                                    let (ddx, ddy, ddz) = geom.delta(a, b);
                                    let r2 = ddx * ddx + ddy * ddy + ddz * ddz;
                                    if r2 > cut2 {
                                        continue;
                                    }
                                    scratch.push(NeighbourPair {
                                        i: i as u32,
                                        j: ju as u32,
                                        dx: ddx,
                                        dy: ddy,
                                        dz: ddz,
                                        r: r2.sqrt().max(1e-9),
                                    });
                                }
                            }
                        }
                    }
                    // CANONICAL ORDER. The stencil visits cells in a geometric order that
                    // has nothing to do with atom index; the sums downstream are
                    // floating-point and therefore order-dependent, so the order is
                    // imposed rather than inherited from the geometry.
                    scratch.sort_unstable_by_key(|p| p.j);
                    out.pairs.extend_from_slice(&scratch);
                }
                out.complete = false;
                self.scratch = scratch;
            }
        }
        out.start.push(out.pairs.len() as u32);
        out.build_adjacency(n);
    }

    /// One stencil axis step: wrapped when periodic, dropped when it leaves an open box.
    #[inline]
    fn axis(c: usize, d: i64, n: usize, periodic: bool) -> Option<usize> {
        let v = c as i64 + d;
        if periodic {
            Some(v.rem_euclid(n as i64) as usize)
        } else if v < 0 || v >= n as i64 {
            None
        } else {
            Some(v as usize)
        }
    }
}

/// The C² quintic switch, and the ONE place its shape lives.
///
/// `S(u) = 1 - 10u³ + 15u⁴ - 6u⁵` on `u = (r - r_in)/(r_cut - r_in)`, which is `1` with
/// zero first and second derivative at `r_in` and `0` with zero first and second
/// derivative at `r_cut`. Multiplying a potential by it produces a potential that is
/// still C¹ in the force — so a truncated interaction is still CONSERVATIVE and the
/// energy gate stays an exact statement rather than an approximate one.
///
/// The four-body sector wrote this shape inline first (`R_IN = 5`, `R_CUT = 6` on the O-H
/// distances). The pair truncation needs the identical function, so it is factored here
/// and both call it: a second copy of a switching function is a second place for the
/// exponents to be wrong, and the symptom would be an energy leak nobody could locate.
///
/// Returns `(S, dS/dr, d²S/dr²)`. The second derivative is returned because the drift
/// bound is built from curvatures: a switched pair term's curvature is
/// `S·U'' + 2·S'·U' + S''·U`, and a bound that used only the first two would be a bound
/// with a term missing rather than a bound.
#[inline]
pub fn switch_c2(r: f64, r_in: f64, r_cut: f64) -> (f64, f64, f64) {
    if r <= r_in {
        return (1.0, 0.0, 0.0);
    }
    if r >= r_cut {
        return (0.0, 0.0, 0.0);
    }
    let w = r_cut - r_in;
    let u = (r - r_in) / w;
    let u2 = u * u;
    let u3 = u2 * u;
    let s = 1.0 - 10.0 * u3 + 15.0 * u3 * u - 6.0 * u3 * u2;
    let ds = (-30.0 * u2 + 60.0 * u3 - 30.0 * u3 * u) / w;
    let dds = (-60.0 * u + 180.0 * u2 - 120.0 * u3) / (w * w);
    (s, ds, dds)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The switch is 1 at the inner edge, 0 at the outer, and its derivative vanishes at
    /// both — the property that makes a truncated potential still a potential.
    #[test]
    fn the_switch_is_flat_at_both_ends() {
        let (s, ds, dds) = switch_c2(5.0, 5.0, 6.0);
        assert_eq!(s, 1.0);
        assert_eq!(ds, 0.0);
        assert_eq!(dds, 0.0);
        let (s, ds, dds) = switch_c2(6.0, 5.0, 6.0);
        assert_eq!(s, 0.0);
        assert_eq!(ds, 0.0);
        assert_eq!(dds, 0.0);
        // Just inside each edge the derivative is still tiny — C², not merely C⁰.
        let (_, ds_in, _) = switch_c2(5.001, 5.0, 6.0);
        let (_, ds_out, _) = switch_c2(5.999, 5.0, 6.0);
        assert!(ds_in.abs() < 1e-4, "dS/dr at the inner edge is {ds_in}");
        assert!(ds_out.abs() < 1e-4, "dS/dr at the outer edge is {ds_out}");
        // And it is monotone down across the window.
        let mut last = 1.0;
        for k in 0..=100 {
            let r = 5.0 + k as f64 * 0.01;
            let (s, _, _) = switch_c2(r, 5.0, 6.0);
            assert!(s <= last + 1e-15, "the switch rose at r = {r}");
            last = s;
        }
    }

    /// The analytic derivative agrees with a finite difference of the value.
    #[test]
    fn the_switch_derivative_is_the_derivative() {
        let h = 1e-6;
        for k in 1..100 {
            let r = 5.0 + k as f64 * 0.01;
            let (_, ds, dds) = switch_c2(r, 5.0, 6.0);
            let fd = (switch_c2(r + h, 5.0, 6.0).0 - switch_c2(r - h, 5.0, 6.0).0) / (2.0 * h);
            assert!(
                (ds - fd).abs() < 1e-6,
                "at r = {r}: analytic {ds}, finite difference {fd}"
            );
            let fd2 = (switch_c2(r + h, 5.0, 6.0).1 - switch_c2(r - h, 5.0, 6.0).1) / (2.0 * h);
            assert!(
                (dds - fd2).abs() < 1e-4,
                "at r = {r}: analytic S'' {dds}, finite difference {fd2}"
            );
        }
    }

    /// Minimum image: a separation longer than half the box comes back as the short way
    /// round, and an open box is untouched.
    #[test]
    fn the_minimum_image_takes_the_short_way() {
        let g = BoxGeom::new(10.0, 10.0, 10.0, true);
        let (dx, _, _) = g.delta((1.0, 0.0, 0.0), (9.0, 0.0, 0.0));
        assert_eq!(dx, -2.0, "9 - 1 = 8 is the long way round a box of 10");
        let (dx, _, _) = g.delta((1.0, 0.0, 0.0), (3.0, 0.0, 0.0));
        assert_eq!(dx, 2.0);

        let open = BoxGeom::new(10.0, 10.0, 10.0, false);
        let (dx, _, _) = open.delta((1.0, 0.0, 0.0), (9.0, 0.0, 0.0));
        assert_eq!(dx, 8.0, "an open box has no images to choose between");
    }

    /// The wrap folds a coordinate one box out back to where it came from, EXACTLY, when
    /// the shift itself was exact. See `Sim::pbc_translation_residual` for why the
    /// exactness of the shift is a precondition that gets checked rather than assumed.
    #[test]
    fn the_wrap_is_exact_for_an_exact_shift() {
        let l = 32.0;
        for x in [0.0f64, 0.5, 8.5, 12.25, 31.75] {
            let shifted = x + l;
            assert_eq!(shifted - l, x, "the shift of {x} by {l} was not exact");
            assert_eq!(BoxGeom::wrap1(shifted, l), x);
            assert_eq!(BoxGeom::wrap1(x - l, l), x);
        }
    }
}
