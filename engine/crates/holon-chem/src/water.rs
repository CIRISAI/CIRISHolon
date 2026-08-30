//! The three-body term of the many-body expansion for (O, H, H), tabulated.
//!
//! # What this is for
//!
//! `trimer.rs` made hydrogen's valence emerge: sixteen hydrogens stop condensing into a
//! droplet and become eight molecules, because the third body pays. That table is
//! homonuclear, and its scope fence named the heteronuclear successor. This is it — the
//! first three-body surface with two different elements on it, so that water's SHAPE, its
//! exactly-two hydrogens, and its FORMATION in a cooling gas can come out of the same
//! three declared inputs (`Z`, the masses, the STO-3G contraction) with no molecular
//! preset anywhere:
//!
//! ```text
//! V2_AB(r) = E(AB; r) - E(A) - E(B)
//! dE3      = E(OHH) - [E(O) + 2 E(H)] - V2_OH(x) - V2_OH(y) - V2_HH(z)
//!          = E(OHH) + E(O) + 2 E(H) - E(OH; x) - E(OH; y) - E(HH; z)
//! ```
//!
//! # The coordinates, and what the heteronuclear case changes
//!
//! `dE3` for three hydrogens is totally symmetric in the triangle's three sides, and
//! `trimer.rs` exploits that by sorting all three. Here the symmetry is `H <-> H`
//! EXCHANGE ONLY: oxygen is a distinct vertex, so the two O-H sides may be swapped and
//! the H-H side may not be swapped with either. The table is therefore built on
//!
//! ```text
//! x = min(r_OH1, r_OH2),  y = max(r_OH1, r_OH2),  u = cos theta_HOH
//! ```
//!
//! — the two O-H sides sorted, and the cosine of the angle AT OXYGEN, gridded through
//! `c = sqrt(1 - u)`. Every point of the box `[R_LO, R_HI]^2 x [C_LO, C_HI]` is a
//! realisable geometry (`z^2 = x^2 + y^2 - 2 x y
//! u` is non-negative for every `|u| <= 1`), so no stencil can reach a node that is not a
//! molecule.
//!
//! Evaluation SORTS the two O-H sides first, which is exact in floating point, so the
//! value and the gradient are invariant under `H <-> H` bit-for-bit rather than to within
//! a tolerance. That is plant (ii)'s target and it passes by construction.
//!
//! ## One thing this table has that the H3 table does not: no sort kink
//!
//! `TrimerTable` composes its interpolant with a sort over ALL THREE sides, and the
//! surface is exactly symmetric in its first two table arguments but only
//! symmetric-to-interpolation-error in the third — so its force has a small
//! discontinuity where the second and third sorted sides cross, which it measures and
//! reports. Here the third coordinate is the H-H side, which never enters the sort, and
//! the only exchange is between the two axes the table IS exactly symmetric in. The kink
//! is therefore expected at roundoff, and [`WaterTable::sort_kink`] is measured anyway:
//! an expectation that is never checked is not a property.
//!
//! # The third coordinate, and the singularity the chain rule has to route around
//!
//! `trimer.rs` uses `c = sqrt(1 - u)` and records the measurement that chose it: at
//! `x = y` the third side is `z = x sqrt(2) c` EXACTLY, so a uniform `c` grid is a
//! uniform `z` grid there, where a uniform `u` grid is uniform in `z^2` and is coarsest
//! precisely where `z` is smallest and the surface steepest.
//!
//! That argument transfers. What does NOT transfer is the domain. A SORTED hydrogen
//! triple has `u <= 1/2`, so H3's grid never goes near `u = 1`; here `u = 1` means both
//! hydrogens on one ray from the oxygen, which is not an exotic corner but exactly the
//! geometry of a hydrogen molecule approaching an oxygen atom head-on — the reaction this
//! campaign exists to watch. And at `u = 1` the `c` parameterisation is singular: `dE3`
//! is analytic in `u`, so `dF/dc` vanishes proportionally to `c`, and the chain rule back
//! to the sides needs `dF/du = -F_c/(2c)`, a `0/0` at the collinear point.
//!
//! Both coordinates were measured rather than argued about. `examples/s2_third.rs`
//! compares them one-dimensionally at five staked `(x, y)`, against exact node values and
//! an exact `dE3/du` from the dual-number route, on BOTH the value and the derivative the
//! force loop actually reads. `c` wins or ties nearly everywhere — at 25 nodes its worst
//! value error is 2.5e-3 against `u`'s 3.9e-3 and its worst derivative error 5.9e-2
//! against 1.2e-1 — because the better node placement outweighs the `1/c` amplification.
//! At 49 nodes they tie at 1.28e-3 and 1.27e-3.
//!
//! So the grid is in `c`, and the SINGULARITY is handled where it belongs, in the chain
//! rule: every derivative this table returns is converted to `u` at the clamp point, and
//! the only division is by `max(c, C_LO)` with `C_LO = 0.05`. Below that fence the
//! surface is extended LINEARLY IN `u` — never in `c`, which is what would put the `1/c`
//! back — so an exactly collinear approach meets a finite force rather than an infinity.
//!
//! # Where the tail lives (and why `R_HI` bounds the LARGER O-H side)
//!
//! `dE3` vanishes exactly when some atom is far from BOTH of the others, which is the
//! statement that two of the three sides are long — SATURATION-1's AMENDMENT A1, and a
//! fact about geometry rather than about species, so it transfers unchanged. What does
//! not transfer is the box: A1's domain is `s2 <= R_cut` on the second-smallest side, and
//! the smallest box in the two O-H sides that CONTAINS that domain is `x, y <= 2 R_cut`,
//! because `s2 <= R_cut` forces every side below `2 R_cut` through the triangle
//! inequality.
//!
//! Measured (`examples/s2_domain.rs`), the worst `|dE3|` anywhere on the shell
//! `max(O-H) = b` is 2.29e-3 Ha at `b = 9`, 1.02e-4 at `b = 12`, 3.25e-5 at `b = 13` and
//! 9.71e-6 at `b = 14` — so `R_HI = 14` is the first integer shell inside the prereg's
//! 1e-5 truncation stake, and it is `2 x 7` with `R_cut = 7`. The worst point on every
//! shell is the near-collinear chain `O - H1 - H2` with both links at about `b/2`, which
//! is what makes a bound on the O-H sides alone the wrong instrument to reason with and
//! the right one to build with.
//!
//! # The closed-angle fence
//!
//! `C_LO > 0` because at `c = 0` with `x = y` the two hydrogens coincide, the basis goes
//! linearly dependent and there is no surface. The `1/z` nuclear repulsion cancels
//! between `E(OHH)` and `E(HH)` by construction, so `dE3` itself stays finite and
//! saturating all the way down — measured smooth to `c = 0.005`, two hydrogens 0.005 bohr
//! apart — but the f64 route's Davidson residual degrades from 1e-10 at `c = 0.05` to
//! 2.3e-10 at `c = 0.02`, 1.8e-9 at `c = 0.01` and 2.2e-8 at `c = 0.005`. The fence is
//! therefore at `C_LO = 0.05`, where the solve is still clean, and the sliver inside it
//! (`theta < 4.05 degrees`) is extended linearly in `u`. That extension is exact to
//! second order — `dF/du` is measured constant to 1% over the last decade of `c` — and it
//! is a genuine first-order Taylor rather than a clamp, so the returned gradient is
//! exactly the gradient of the returned value there.
//!
//! # The interpolant
//!
//! Tensor-product Catmull-Rom, the same scheme and the same [`crate::trimer::cr_weights`]
//! the H3 table ships, so there is one interpolation scheme in this crate and not two.
//! Forces come from differentiating it analytically, so the dynamics' energy function IS
//! the tabulated surface and conservation holds against it exactly.
//!
//! # How the table is FILLED, and the fence that puts on the record
//!
//! One (O, H, H) point is 441 determinants through [`crate::pair::solve_geometry`] and
//! costs about 50 ms — a thousand times an H3 point, which is nine determinants on a
//! bespoke s-only path. So this table is NOT built at page load the way H3's is; it is
//! generated natively by `examples/s2_table.rs` and committed, and the browser streams
//! the committed artifact. That is a real difference from SATURATION-1 and it is stated
//! rather than left to be discovered. What is NOT different: the numbers still come from
//! this crate's own solver, the artifact names its producer and its grid rule, and
//! `tests/water.rs` recomputes a staked subset of nodes through [`stream_water_table`]
//! and requires BIT-IDENTITY — a tolerance there would be measuring the tolerance.

use crate::dual::D2;
use crate::elements::{HYDROGEN, OXYGEN};
use crate::pair::{atom_energy, pair_point, solve_geometry};
use crate::trimer::cr_weights;

// ================================================================ the grid
//
// Every constant below is a MEASUREMENT, and the measuring example is named beside it.

/// Grid lower edge in either O-H side, bohr. Below the staked domain floor of 0.9 on
/// purpose: a query AT 0.9 is then interior to the grid rather than on its boundary,
/// where the node slopes are one-sided.
pub const R_LO: f64 = 0.7;

/// Grid upper edge in either O-H side, bohr — the truncation radius. See the module
/// header: `examples/s2_domain.rs` measures 9.71e-6 Ha as the worst `|dE3|` on this
/// shell, inside the prereg's 1e-5 stake, and 14 is `2 x 7` with 7 the second-smallest
/// side's own cut.
pub const R_HI: f64 = 14.0;

/// Stretch of the side axis: `r = R_LO + (R_HI - R_LO) (e^{a tau} - 1)/(e^a - 1)`. The
/// surface is steep at contact and flat in the tail, so an exponential stretch puts the
/// knots where the curvature is; it is used INSTEAD of a power law because `dtau/dr`
/// stays finite at the lower edge, and a coordinate singularity there would be a force
/// singularity.
///
/// MEASURED by `examples/s2_grid.rs`, which builds one fine table per candidate and reads
/// the held-out error of every coarser grid inside it: at 49 x 49 x 25 the held-out
/// maximum is 7.84e-4 Ha at `a = 2`, 6.72e-4 at `a = 3` and 8.00e-4 at `a = 4`.
pub const STRETCH_A: f64 = 3.0;

/// The closed-angle fence, `c = sqrt(1 - cos theta_HOH)`. See the module header: not a
/// property of the surface, which stays smooth past it, but of the f64 solve, whose
/// residual degrades as the two hydrogens approach. Inside it the surface is extended
/// linearly in `u`.
pub const C_LO: f64 = 0.05;

/// The collinear H-O-H end, `theta = 180 degrees`, `u = -1`. A hard edge of the geometry:
/// `u < -1` violates the triangle inequality, so there is no node beyond it and the last
/// interval uses the one-sided node slope.
pub const C_HI: f64 = core::f64::consts::SQRT_2;

/// The `u` the fence stands for. Every query with `u` above this is extrapolated.
pub const U_FENCE: f64 = 1.0 - C_LO * C_LO;

/// Nodes per side axis, and per `c`.
pub const NR: usize = 65;
pub const NU: usize = 33;

/// Total node count.
pub const N_NODES: usize = NR * NR * NU;

/// Nodes actually SOLVED: the sorted half `i <= j`. The mirror node takes the same float
/// rather than a second rounding of the same number, which is what makes the `H <-> H`
/// symmetry exact in the stored table and not only in the evaluation.
pub const N_SOLVED: usize = NR * (NR + 1) / 2 * NU;

/// The label this crate puts on the surface it computed. Says the model, the arithmetic
/// and the route, and deliberately does NOT say "exact", which would be a claim about the
/// world rather than about the basis.
pub const WATER_PROVENANCE: &str =
    "engine-computed STO-3G FCI (O,H,H) three-body term, general N-centre route, f64";

/// Node index for `(i, j, k)` — `x` slowest, `u` fastest.
#[inline]
pub const fn node_index(i: usize, j: usize, k: usize) -> usize {
    (i * NR + j) * NU + k
}

#[inline]
pub fn r_of_tau(tau: f64) -> f64 {
    R_LO + (R_HI - R_LO) * ((STRETCH_A * tau).exp() - 1.0) / (STRETCH_A.exp() - 1.0)
}

#[inline]
pub fn tau_of_r(r: f64) -> f64 {
    (1.0 + (r - R_LO) * (STRETCH_A.exp() - 1.0) / (R_HI - R_LO)).ln() / STRETCH_A
}

/// `dtau/dr`. Finite everywhere on `[R_LO, R_HI]` — that is the whole reason the stretch
/// is exponential rather than a power law.
#[inline]
pub fn dtau_dr(r: f64) -> f64 {
    let k = (STRETCH_A.exp() - 1.0) / (R_HI - R_LO);
    k / (STRETCH_A * (1.0 + (r - R_LO) * k))
}

/// The O-H side at side-axis node `i`.
#[inline]
pub fn node_r(i: usize) -> f64 {
    r_of_tau(i as f64 / (NR - 1) as f64)
}

/// The third coordinate at angle-axis node `k`, and the cosine it stands for.
#[inline]
pub fn node_c(k: usize) -> f64 {
    C_LO + (C_HI - C_LO) * k as f64 / (NU - 1) as f64
}

#[inline]
pub fn node_u(k: usize) -> f64 {
    let c = node_c(k);
    1.0 - c * c
}

/// The geometry a node stands for: the two O-H sides and the H-H side.
pub fn node_geometry(i: usize, j: usize, k: usize) -> (f64, f64, f64) {
    let (x, y, u) = (node_r(i), node_r(j), node_u(k));
    (x, y, hh_side(x, y, u))
}

/// The H-H side implied by the two O-H sides and the angle at oxygen.
#[inline]
pub fn hh_side(x: f64, y: f64, u: f64) -> f64 {
    (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt()
}

// ================================================================ dE3 itself

fn c3(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

/// Total energy of one (O, H, H) geometry: oxygen at the origin, one hydrogen along `+x`
/// at `x`, the other at `y` with `cos(theta_HOH) = u`.
///
/// The placement is written once, here, and the 50-digit referee
/// (`conformance/atomworld/saturation2_referee.py`) constructs the same one, so the two
/// implementations differ in arithmetic and not in geometry.
pub fn ohh_energy(x: f64, y: f64, u: f64) -> f64 {
    // In f64, never through a dual number: `sqrt(1 - u^2)` is an exact zero at the
    // collinear ends, where `D2::sqrt` forms `0/0` in the derivative slots and poisons
    // the geometry with a NaN even when the input derivative is zero.
    let sn = (1.0 - u * u).max(0.0).sqrt();
    solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![c3(0.0, 0.0, 0.0), c3(x, 0.0, 0.0), c3(y * u, y * sn, 0.0)],
    )
    .e
    .v
}

/// The three-body interaction at O-H sides `(x, y)` and angle cosine `u`, hartree, with
/// the reference energies supplied by the caller.
///
/// `e_ox` and `e_oy` are the O-H pair energies at `x` and `y`; the table build caches
/// them over the node list, since the two side axes only ever take `NR` distinct values.
pub fn de3_with(x: f64, y: f64, u: f64, e_o: f64, e_h: f64, e_ox: f64, e_oy: f64) -> f64 {
    let z = hh_side(x, y, u);
    ohh_energy(x, y, u) + e_o + 2.0 * e_h
        - e_ox
        - e_oy
        - pair_point(HYDROGEN, HYDROGEN, z).e
}

/// The same, solving everything from scratch. For a test that wants one named geometry.
pub fn de3(x: f64, y: f64, u: f64) -> f64 {
    de3_with(
        x,
        y,
        u,
        atom_energy(OXYGEN),
        atom_energy(HYDROGEN),
        pair_point(OXYGEN, HYDROGEN, x).e,
        pair_point(OXYGEN, HYDROGEN, y).e,
    )
}

/// What the whole table says about itself.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterMeta {
    pub n_nodes: usize,
    pub nr: usize,
    pub nu: usize,
    pub r_lo: f64,
    pub r_hi: f64,
    pub c_lo: f64,
    pub c_hi: f64,
    pub stretch_a: f64,
    /// The isolated-atom energies the whole expansion is referenced to.
    pub e_o_atom: f64,
    pub e_h_atom: f64,
    /// Largest `|dE3|` on any node.
    pub peak: f64,
    /// Electronic-structure solves the build actually paid for.
    pub solves: usize,
}

impl WaterMeta {
    pub const fn empty() -> Self {
        Self {
            n_nodes: N_NODES,
            nr: NR,
            nu: NU,
            r_lo: R_LO,
            r_hi: R_HI,
            c_lo: C_LO,
            c_hi: C_HI,
            stretch_a: STRETCH_A,
            e_o_atom: 0.0,
            e_h_atom: 0.0,
            peak: 0.0,
            solves: 0,
        }
    }
}

/// Compute the table node by node, handing each to `push` as `(index, x, y, u, dE3)`.
///
/// Only `i <= j` is solved; the mirror node is handed over with the SAME float rather
/// than a second rounding of one number, so the `H <-> H` symmetry is a property of the
/// stored values and not only of the lookup. `push` returning false aborts.
pub fn stream_water_table<F>(mut push: F) -> Option<WaterMeta>
where
    F: FnMut(usize, f64, f64, f64, f64) -> bool,
{
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    // The two side axes only ever take `NR` distinct values, so their O-H pair energies
    // are solved once each. The H-H side is a continuum and is not cacheable.
    let mut v_cache = [0.0f64; NR];
    for (i, v) in v_cache.iter_mut().enumerate() {
        *v = pair_point(OXYGEN, HYDROGEN, node_r(i)).e;
    }
    let mut peak = 0.0f64;
    let mut solves = 0usize;
    for i in 0..NR {
        for j in i..NR {
            let (x, y) = (node_r(i), node_r(j));
            for k in 0..NU {
                let u = node_u(k);
                let d = de3_with(x, y, u, e_o, e_h, v_cache[i], v_cache[j]);
                solves += 1;
                if d.abs() > peak {
                    peak = d.abs();
                }
                if !push(node_index(i, j, k), x, y, u, d) {
                    return None;
                }
                if i != j && !push(node_index(j, i, k), y, x, u, d) {
                    return None;
                }
            }
        }
    }
    Some(WaterMeta {
        n_nodes: N_NODES,
        nr: NR,
        nu: NU,
        r_lo: R_LO,
        r_hi: R_HI,
        c_lo: C_LO,
        c_hi: C_HI,
        stretch_a: STRETCH_A,
        e_o_atom: e_o,
        e_h_atom: e_h,
        peak,
        solves,
    })
}

// ================================================================ the interpolant

/// Widening factor on the measured curvature envelope, with the same derivation
/// [`crate::trimer::ENVELOPE_WIDENING`] carries: the scheme's Lebesgue constant is at
/// most 1.25 per axis, so two cubic axes can overshoot by `1.25^2 = 1.6`, and 4 clears
/// that by 2.5x with the cell-centre-versus-edge difference along the third axis inside
/// the same margin.
pub const ENVELOPE_WIDENING: f64 = 4.0;

/// The tabulated (O, H, H) three-body surface, and the forces read off it.
///
/// `loaded` is false until every node has arrived, so a half-filled table contributes
/// nothing rather than nonsense — and a `Sim` that never fills one behaves EXACTLY as it
/// did before this module existed, because [`WaterTable::eval`] returns an identical zero
/// and adding zero to a finite float changes no bit.
#[derive(Clone)]
pub struct WaterTable {
    v: Vec<f64>,
    filled: usize,
    pub loaded: bool,
    pub meta: WaterMeta,
    /// ABSOLUTE cap on the interpolant's second derivative in SIDE coordinates,
    /// hartree/bohr^2 — the row-sum norm of the 3x3 side-space Hessian, maximised over
    /// the grid and widened.
    pub curvature_envelope: f64,
    /// LOCAL cap, per bohr: `||H|| <= curvature_per_gradient * max_a |dF/ds_a|` everywhere
    /// the sample reached. The drift bound takes the smaller of the two, which keeps it a
    /// live reading of the configuration rather than a constant pinned to a corner of the
    /// table the trajectory never visits.
    pub curvature_per_gradient: f64,
    /// Largest jump in `dF/ds` across the `H <-> H` sort boundary (the two O-H sides
    /// equal), hartree/bohr.
    ///
    /// Expected at ROUNDOFF here, unlike `TrimerTable::sort_kink`: the H-H side never
    /// enters the sort, and the table is exactly symmetric in the only pair that does.
    /// Measured anyway — an expectation that is never checked is not a property.
    pub sort_kink: f64,
}

impl Default for WaterTable {
    fn default() -> Self {
        Self::empty()
    }
}

/// Index-space read of the interpolant: the value, the three first partials with respect
/// to the fractional node index along each axis, and the two mixed partials the
/// closed-angle extension needs.
#[derive(Clone, Copy, Default)]
struct GridRead {
    f: f64,
    fx: f64,
    fy: f64,
    fu: f64,
    fxu: f64,
    fyu: f64,
}

impl WaterTable {
    /// `const` so `holon-render`'s `static SIM: Mutex<Sim> = Mutex::new(Sim::empty())`
    /// can hold one. The node array is a `Vec` rather than the fixed `[f64; N_NODES]`
    /// [`crate::trimer::TrimerTable`] uses: at 49 x 49 x 25 this table is 480 KB and at
    /// the sizes the grid sweep is choosing between it can be twice that, which is a lot
    /// to carry inline in a `Sim` that gets moved. Empty until `begin`, so the const
    /// constructor allocates nothing.
    pub const fn empty() -> Self {
        Self {
            v: Vec::new(),
            filled: 0,
            loaded: false,
            meta: WaterMeta::empty(),
            curvature_envelope: 0.0,
            curvature_per_gradient: 0.0,
            sort_kink: 0.0,
        }
    }

    pub fn begin(&mut self) {
        self.v.clear();
        self.v.resize(N_NODES, 0.0);
        self.filled = 0;
        self.loaded = false;
        self.curvature_envelope = 0.0;
        self.curvature_per_gradient = 0.0;
        self.sort_kink = 0.0;
    }

    pub fn knot(&mut self, index: usize, value: f64) -> bool {
        if index >= N_NODES || !value.is_finite() || self.v.len() != N_NODES {
            return false;
        }
        self.v[index] = value;
        self.filled += 1;
        true
    }

    /// Close the table: adopt the metadata and MEASURE the envelopes the drift bound
    /// needs. Returns false if any node is missing or the grid rule disagrees.
    pub fn finish(&mut self, meta: WaterMeta) -> bool {
        if self.filled < N_NODES || meta.n_nodes != N_NODES || meta.nr != NR || meta.nu != NU {
            return false;
        }
        self.meta = meta;
        self.loaded = true;
        self.measure_envelopes();
        true
    }

    /// Raw node value, for the tests and the plants.
    pub fn node(&self, i: usize, j: usize, k: usize) -> f64 {
        self.v[node_index(i, j, k)]
    }

    /// Overwrite one node, on BOTH sides of the diagonal so the stored symmetry survives.
    /// The plants use it to mutate a table on purpose; nothing in the dynamics does.
    pub fn set_node(&mut self, i: usize, j: usize, k: usize, value: f64) {
        self.v[node_index(i, j, k)] = value;
        self.v[node_index(j, i, k)] = value;
        self.measure_envelopes();
    }

    /// Overwrite ONE node without its mirror — deliberately breaking the `H <-> H`
    /// symmetry. Plant (ii)'s desymmetrised table, and only the plant.
    pub fn set_node_asymmetric(&mut self, i: usize, j: usize, k: usize, value: f64) {
        self.v[node_index(i, j, k)] = value;
        self.measure_envelopes();
    }

    /// Negate the whole surface — plant (i), and only the plant.
    pub fn negate(&mut self) {
        for x in self.v.iter_mut() {
            *x = -*x;
        }
        self.measure_envelopes();
    }

    /// The surface and its three side-derivatives at one (O, H, H) triangle, hartree and
    /// hartree/bohr.
    ///
    /// The arguments are the two O-H sides and the H-H side, IN THAT ORDER — unlike
    /// [`crate::trimer::TrimerTable::eval`], which takes three interchangeable sides,
    /// because here they are not interchangeable. The returned gradient is in the same
    /// order.
    ///
    /// Returns an exact zero outside the domain — the truncation gate T2 gauges — and
    /// below `R_LO`, or above `U_HI`, extends the surface linearly and C1 at the edge, so
    /// a violent collision or an exactly collinear approach meets a continuous force
    /// rather than a cliff.
    // The negated comparisons reject NaN as well as an out-of-domain triangle, which
    // `a <= b` would silently accept and hand to the interpolator. Same fence, same
    // reason, as `holon_chem::table::stream_table`'s.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn eval(&self, r_oh1: f64, r_oh2: f64, r_hh: f64) -> (f64, [f64; 3]) {
        if !self.loaded {
            return (0.0, [0.0; 3]);
        }
        // Sort the two O-H sides, exactly, carrying whether they swapped. Two elements,
        // one comparison, so the two orders agree bit-for-bit rather than to a tolerance.
        let swapped = r_oh1 > r_oh2;
        let (x, y) = if swapped {
            (r_oh2, r_oh1)
        } else {
            (r_oh1, r_oh2)
        };
        let z = r_hh;
        if !(y <= R_HI) || !(x > 0.0) || !(z >= 0.0) {
            return (0.0, [0.0; 3]);
        }
        let (v, g) = self.eval_sorted(x, y, z);
        let out = if swapped {
            [g[1], g[0], g[2]]
        } else {
            [g[0], g[1], g[2]]
        };
        (v, out)
    }

    /// The surface with the two O-H sides already ordered `x <= y`. Public to the
    /// module's own envelope measurement, which must not re-sort.
    ///
    /// # The one place the coordinate singularity lives, and how it is fenced
    ///
    /// The grid is in `c = sqrt(1 - u)` and the sides need `u`. Both conversions —
    /// `dF/du = F_c dc/du` with `dc/du = -1/(2c)`, and the extensions — are done at the
    /// CLAMP POINT, whose `c` is at least `C_LO`. So the only division is by a number
    /// bounded below by 0.05, and the extension inside the fence is linear in `u`. That
    /// is what makes the force finite at an exactly collinear head-on approach, where a
    /// `c`-space extension would diverge.
    fn eval_sorted(&self, x: f64, y: f64, z: f64) -> (f64, [f64; 3]) {
        let u = ((x * x + y * y - z * z) / (2.0 * x * y)).clamp(-1.0, 1.0);
        let c = (1.0 - u).max(0.0).sqrt();

        // Clamp into the grid; the displacements drive the linear extensions. `c` is
        // clamped only from below: `c > C_HI` needs `u < -1`, which is not a triangle.
        let xq = x.max(R_LO);
        let yq = y.max(R_LO);
        let cq = c.max(C_LO);
        let uq = 1.0 - cq * cq;
        let r = self.eval_grid(xq, yq, cq);

        // Index-space derivatives to physical ones, and `c` to `u` in the same step, so
        // everything below this line is in the coordinate the sides are chained through.
        let sr = (NR - 1) as f64;
        let sc = (NU - 1) as f64 / (C_HI - C_LO);
        let dc_du = -0.5 / cq;
        let fx = r.fx * sr * dtau_dr(xq);
        let fy = r.fy * sr * dtau_dr(yq);
        let fu = r.fu * sc * dc_du;
        let fxu = r.fxu * sr * dtau_dr(xq) * sc * dc_du;
        let fyu = r.fyu * sr * dtau_dr(yq) * sc * dc_du;

        // First-order extension in whichever coordinates were clamped, WITH the two mixed
        // terms, so the returned gradient is exactly the gradient of the returned value in
        // every clamped region and not only in the interior. When nothing is clamped every
        // displacement is zero and this reduces to the interpolant itself.
        let (dx, dy, du) = (x - xq, y - yq, u - uq);
        let f = r.f + dx * fx + dy * fy + du * fu + dx * du * fxu + dy * du * fyu;
        let fx = fx + du * fxu;
        let fy = fy + du * fyu;
        let fu = fu + dx * fxu + dy * fyu;

        // Chain rule from (x, y, u) to the three sides:
        //   du/dx = (x^2 - y^2 + z^2) / (2 x^2 y)
        //   du/dy = (y^2 - x^2 + z^2) / (2 x y^2)
        //   du/dz = -z / (x y)
        // All three are finite everywhere on the domain, including at both collinear ends.
        let (x2, y2, z2) = (x * x, y * y, z * z);
        let du_dx = (x2 - y2 + z2) / (2.0 * x2 * y);
        let du_dy = (y2 - x2 + z2) / (2.0 * x * y2);
        let du_dz = -z / (x * y);
        (f, [fx + fu * du_dx, fy + fu * du_dy, fu * du_dz])
    }

    /// The interpolant in its own INDEX coordinates.
    fn eval_grid(&self, x: f64, y: f64, c: f64) -> GridRead {
        let tx = (tau_of_r(x) * (NR - 1) as f64).clamp(0.0, (NR - 1) as f64);
        let ty = (tau_of_r(y) * (NR - 1) as f64).clamp(0.0, (NR - 1) as f64);
        let tc = ((c - C_LO) / (C_HI - C_LO) * (NU - 1) as f64).clamp(0.0, (NU - 1) as f64);
        let (bx, wx, dwx) = cr_weights(NR, tx);
        let (by, wy, dwy) = cr_weights(NR, ty);
        let (bu, wu, dwu) = cr_weights(NU, tc);

        // Contract x first into a 4x4 slab, once for the value weights and once for the
        // derivative weights; the y and u contractions are then four-term sums.
        let mut p = [[0.0f64; 4]; 4];
        let mut q = [[0.0f64; 4]; 4];
        for (b, (pb, qb)) in p.iter_mut().zip(q.iter_mut()).enumerate() {
            for (e, (pv, qv)) in pb.iter_mut().zip(qb.iter_mut()).enumerate() {
                let mut acc = 0.0;
                let mut dacc = 0.0;
                for a in 0..4 {
                    let v = self.v[node_index(bx + a, by + b, bu + e)];
                    acc += wx[a] * v;
                    dacc += dwx[a] * v;
                }
                *pv = acc;
                *qv = dacc;
            }
        }
        let mut out = GridRead::default();
        for e in 0..4 {
            let mut sv = 0.0;
            let mut sd = 0.0;
            let mut sq = 0.0;
            for b in 0..4 {
                sv += wy[b] * p[b][e];
                sd += dwy[b] * p[b][e];
                sq += wy[b] * q[b][e];
            }
            out.f += wu[e] * sv;
            out.fu += dwu[e] * sv;
            out.fy += wu[e] * sd;
            out.fx += wu[e] * sq;
            out.fyu += dwu[e] * sd;
            out.fxu += dwu[e] * sq;
        }
        out
    }

    /// Measure the two curvature envelopes and the sort kink, once, when the table
    /// closes. The derivation is [`crate::trimer::TrimerTable`]'s and is not repeated;
    /// what differs is that the sample here stays inside the grid without having to
    /// avoid a sort boundary, because the only sort is between two axes the table is
    /// exactly symmetric in.
    fn measure_envelopes(&mut self) {
        const HH: f64 = 1e-3;
        let mut k_abs = 0.0f64;
        let mut per_grad = 0.0f64;
        for i in 0..(NR - 1) {
            for j in i..(NR - 1) {
                for k in 0..(NU - 1) {
                    let x = 0.5 * (node_r(i) + node_r(i + 1));
                    let y = 0.5 * (node_r(j) + node_r(j + 1));
                    let u = 1.0 - {
                        let cm = 0.5 * (node_c(k) + node_c(k + 1));
                        cm * cm
                    };
                    let z = hh_side(x, y, u);
                    // A finite difference must not step outside the domain, and must not
                    // straddle the `x = y` swap: `eval_sorted` does not re-sort, so a
                    // step that crossed it would be read on the wrong branch.
                    if y > R_HI - HH || x < R_LO + HH || y - x < 4.0 * HH || z < 4.0 * HH {
                        continue;
                    }
                    let mut rows = 0.0f64;
                    for a in 0..3 {
                        let mut lo = [x, y, z];
                        let mut hi = [x, y, z];
                        lo[a] -= HH;
                        hi[a] += HH;
                        let (_, glo) = self.eval_sorted(lo[0], lo[1], lo[2]);
                        let (_, ghi) = self.eval_sorted(hi[0], hi[1], hi[2]);
                        let row: f64 = (0..3)
                            .map(|b| ((ghi[b] - glo[b]) / (2.0 * HH)).abs())
                            .sum();
                        rows = rows.max(row);
                    }
                    if rows > k_abs {
                        k_abs = rows;
                    }
                    let (_, g) = self.eval_sorted(x, y, z);
                    let gmax = g.iter().fold(0.0f64, |m, v| m.max(v.abs()));
                    if gmax > 1e-9 {
                        per_grad = per_grad.max(rows / gmax);
                    }
                }
            }
        }
        self.curvature_envelope = ENVELOPE_WIDENING * k_abs;
        self.curvature_per_gradient = ENVELOPE_WIDENING * per_grad;
        self.sort_kink = self.measure_sort_kink();
    }

    /// The force discontinuity at the `H <-> H` sort boundary: the largest `|dF/ds|` jump
    /// across the surface `r_OH1 = r_OH2`. Reported, not bounded away — and expected at
    /// roundoff, because the table is exactly symmetric in the pair that is sorted.
    fn measure_sort_kink(&self) -> f64 {
        const EPS: f64 = 1e-6;
        let mut worst = 0.0f64;
        for i in 1..(NR - 1) {
            for k in 0..NU {
                let r = node_r(i);
                let u = node_u(k);
                let z = hh_side(r, r, u);
                if z < 1e-3 {
                    continue;
                }
                let (_, ga) = self.eval(r - EPS, r + EPS, z);
                let (_, gb) = self.eval(r + EPS, r - EPS, z);
                // The exchange swaps the first two gradient slots and fixes the third.
                for (a, b) in [(0usize, 1usize), (1, 0), (2, 2)] {
                    worst = worst.max((ga[a] - gb[b]).abs());
                }
            }
        }
        worst
    }
}

// ================================================================ the artifact
//
// One (O, H, H) point is 441 determinants and about 50 ms, so the table is generated
// natively and committed rather than rebuilt at load. The text below is what gets
// committed: the grid rule it was built to, the reference energies it is measured
// against, and the sorted half of the node values as raw IEEE-754 bit patterns.
//
// BIT PATTERNS rather than decimal digits, for the reason `tests/data/w1_baseline.txt`
// records: two f64 with the same bits are the same number; two that print the same to
// seventeen digits need not be. The gate that checks this artifact recomputes a staked
// subset and requires equality, and a decimal round-trip would put a tolerance where no
// tolerance belongs.

/// Header key for the grid rule, written and then REQUIRED by the loader. A table built
/// to a different grid is refused rather than silently interpolated on the wrong axes.
const GRID_KEY: &str = "grid";

/// Render the table as the committed artifact.
pub fn to_text(t: &WaterTable) -> String {
    let m = &t.meta;
    let mut s = String::with_capacity(N_SOLVED * 18 + 512);
    s.push_str("# SATURATION-2 (O,H,H) three-body table\n");
    s.push_str("# producer: holon-chem examples/s2_table.rs\n");
    s.push_str(&format!("# provenance: {WATER_PROVENANCE}\n"));
    s.push_str(&format!(
        "# {GRID_KEY}: NR={NR} NU={NU} R_LO={R_LO} R_HI={R_HI} STRETCH_A={STRETCH_A} C_LO={C_LO} C_HI={C_HI:.17}\n"
    ));
    s.push_str(&format!("# e_o_atom: {:016x}\n", m.e_o_atom.to_bits()));
    s.push_str(&format!("# e_h_atom: {:016x}\n", m.e_h_atom.to_bits()));
    s.push_str(&format!("# peak: {:016x}\n", m.peak.to_bits()));
    s.push_str(&format!("# solves: {}\n", m.solves));
    s.push_str("# values: the sorted half i <= j, i slowest then j then k; the mirror\n");
    s.push_str("# node (j, i, k) takes the SAME float and is not stored twice.\n");
    for i in 0..NR {
        for j in i..NR {
            for k in 0..NU {
                s.push_str(&format!("{:016x}\n", t.node(i, j, k).to_bits()));
            }
        }
    }
    s
}

/// The grid line the loader requires to match.
pub fn grid_line() -> String {
    format!(
        "# {GRID_KEY}: NR={NR} NU={NU} R_LO={R_LO} R_HI={R_HI} STRETCH_A={STRETCH_A} C_LO={C_LO} C_HI={C_HI:.17}"
    )
}

/// Parse the committed artifact back into a table.
///
/// Refuses a file whose grid line is not this build's, which is the difference between
/// loading a table and loading numbers.
pub fn from_text(src: &str) -> Option<WaterTable> {
    let want = grid_line();
    let mut e_o = None;
    let mut e_h = None;
    let mut peak = None;
    let mut solves = None;
    let mut grid_ok = false;
    let mut vals: Vec<f64> = Vec::with_capacity(N_SOLVED);
    for line in src.lines() {
        let line = line.trim_end();
        if let Some(rest) = line.strip_prefix('#') {
            let rest = rest.trim();
            if line == want {
                grid_ok = true;
            } else if let Some(h) = rest.strip_prefix("e_o_atom:") {
                e_o = u64::from_str_radix(h.trim(), 16).ok().map(f64::from_bits);
            } else if let Some(h) = rest.strip_prefix("e_h_atom:") {
                e_h = u64::from_str_radix(h.trim(), 16).ok().map(f64::from_bits);
            } else if let Some(h) = rest.strip_prefix("peak:") {
                peak = u64::from_str_radix(h.trim(), 16).ok().map(f64::from_bits);
            } else if let Some(h) = rest.strip_prefix("solves:") {
                solves = h.trim().parse::<usize>().ok();
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        vals.push(f64::from_bits(u64::from_str_radix(line, 16).ok()?));
    }
    if !grid_ok || vals.len() != N_SOLVED {
        return None;
    }
    let mut t = WaterTable::empty();
    t.begin();
    let mut at = 0usize;
    for i in 0..NR {
        for j in i..NR {
            for k in 0..NU {
                let v = vals[at];
                at += 1;
                if !t.knot(node_index(i, j, k), v) {
                    return None;
                }
                if i != j && !t.knot(node_index(j, i, k), v) {
                    return None;
                }
            }
        }
    }
    let meta = WaterMeta {
        e_o_atom: e_o?,
        e_h_atom: e_h?,
        peak: peak?,
        solves: solves?,
        ..WaterMeta::empty()
    };
    if !t.finish(meta) {
        return None;
    }
    Some(t)
}

/// Build the whole table natively, single-threaded. `examples/s2_table.rs` is the
/// threaded producer; this is the convenience a test with time to spare uses.
pub fn generate() -> Option<WaterTable> {
    let mut t = WaterTable::empty();
    t.begin();
    let meta = stream_water_table(|i, _x, _y, _u, v| t.knot(i, v))?;
    if !t.finish(meta) {
        return None;
    }
    Some(t)
}
