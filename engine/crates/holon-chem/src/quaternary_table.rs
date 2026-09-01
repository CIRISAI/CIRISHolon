//! The four-body `(O,H,H,H)` term, tabulated: coordinates, canonical form, and the
//! six-dimensional interpolant the trajectory loop reads instead of solving.
//!
//! # Why not the six distances
//!
//! Four atoms have `3*4 - 6 = 6` internal degrees of freedom, so six coordinates is the
//! right COUNT — but the six interatomic distances are the wrong COORDINATES, and the
//! measurement is not close. Over a box `R_OH [1.2,6.0]^3 x R_HH [0.9,12.0]^3` sampled at
//! `21^6 = 85,766,121` points, **6.53%** of the box is a realisable geometry: 86.99% fails
//! `|cos| <= 1` outright and a further 6.48% fails the Gram condition
//! (`examples/de4_price.rs`). A tensor-product interpolant there would stand almost
//! entirely on configurations that do not exist.
//!
//! The coordinates are therefore the three O-H distances and the three cosines of the
//! H-O-H angles, `(R1, R2, R3, u12, u23, u31)`. The cosine IS the coordinate, so
//! `|cos| <= 1` holds by construction and the `86.99%` failure disappears; the H-H
//! distances come back out as `R_ij = sqrt(R_i^2 + R_j^2 - 2 R_i R_j u_ij)`. What survives
//! is the embeddability condition alone — three unit vectors exist in `R^3` only where the
//! Gram determinant
//!
//! ```text
//! G = 1 + 2 u12 u23 u31 - u12^2 - u23^2 - u31^2  >=  0
//! ```
//!
//! is non-negative, which is `pi^2/16 = 61.69%` of the cosine cube. That 38% is handled by
//! [`elliptope_scale`] rather than by pretending it is not there; see below.
//!
//! # The canonical form, and why the old one was not one
//!
//! Relabelling the three hydrogens is a group of order six acting on a labelled geometry:
//! `sigma` sends `R_a -> R_{sigma(a)}` AND `u_ab -> u_{sigma(a)sigma(b)}`, simultaneously.
//! The predecessor [`crate::quaternary::sort_ohhh_internals`] sorts the three O-H distances
//! and the three H-H distances INDEPENDENTLY, which is invariance under `S3 x S3`, order
//! 36. Thirty-six over six is six, so it generically hands SIX distinct geometries one
//! address — measured, two geometries at one address whose `dE4` differ by `6.355e-3` Ha.
//! Its own test passes, because invariance under more than the group is still invariance
//! and a test that checks invariance never checks injectivity.
//!
//! [`canonical_ohhh`] is the lexicographic least of the six relabelled 6-tuples. It is
//! comparisons only, so the same geometry presented under any labelling produces the
//! identical array bit for bit, and it separates the pair the old one merged.
//!
//! # The continuation, and why it has to be equivariant
//!
//! A grid node outside the elliptope is not a geometry, but a stencil standing next to the
//! boundary still has to read something there. [`elliptope_scale`] scales the cosine triple
//! radially toward the origin — `u -> t*u` for the largest `t <= 1` with `G(t) >= 0` —
//! which lands on the nearest configuration along the ray and is CONTINUOUS at the
//! boundary (`t = 1` there). It matters that scaling COMMUTES with permuting the three
//! cosines: the continuation is therefore `S3`-equivariant, the orbit fill stays exact, and
//! bit-exact permutation invariance survives the continuation instead of being broken by
//! it. A continuation that treated one cosine specially would destroy that, which is what
//! plant (ii) of `DE4_TABLE_PREREG.md` tests in both directions.
//!
//! Continued nodes are MARKED. They are not geometries and are excluded from every
//! accuracy statistic by construction rather than by a filter someone has to remember.
//!
//! # The interpolant
//!
//! Tensor-product Catmull-Rom over six axes, exactly the scheme the three-body tables use:
//! cubic Hermite whose node slopes are centred differences, one-sided at the ends. It is
//! C1 by construction — each node's slope is one fixed linear functional of the node
//! values, so both cells meeting at a node agree — and it needs node VALUES only, so the
//! table streams in as a flat array with no mixed-derivative columns to keep true. Forces
//! come from differentiating the interpolant analytically, so the dynamics' energy function
//! IS the tabulated surface and the ledger closes against it.
//!
//! The stencil is `4^6 = 4096` nodes. Contracted axis by axis that is ~5.5k operations per
//! output and seven outputs (value plus six partials), so a few tens of microseconds —
//! against a measured MEAN of 9.84 SECONDS for the four-solve path it replaces.

use crate::trimer::cr_weights;

// ================================================================= the frozen grid
//
// These are `conformance/water_observatory/DE4_TABLE_PREREG.md`'s frozen domain. They are
// constants rather than parameters because the grid is part of the table's identity: a
// file whose grid line does not match this build's is refused rather than reinterpreted.

/// Shortest O-H distance on the grid.
pub const R_LO: f64 = 0.9;
/// Longest O-H distance: `quaternary::R_CUT`, the MEASURED far-field cutoff (`|dE4|` is
/// 4.9e-5 Ha at 6.1 bohr, 1.7e-6 Ha by 9). The three-body table's 15.0 does not transfer.
pub const R_HI: f64 = 6.0;
/// Exponential stretch on the radial axes, matching the three-body tables' `a = 3`.
pub const STRETCH_A: f64 = 3.0;
/// Most negative cosine on the grid: H-O-H fully open (collinear).
pub const U_LO: f64 = -1.0;
/// Closed-angle fence, `1 - C_LO^2` with the (O,H,H) table's own `C_LO = 0.05`. Slice 6 of
/// the seam scan measured `d3[E_FCI] = 2.46e5` as `u -> 1`: a coordinate collapse, not a
/// state crossing, and a fence is what the three-body tables already put there.
pub const U_HI: f64 = 0.9975;
/// Nodes per radial axis.
pub const NR: usize = 13;
/// Nodes per cosine axis.
pub const NU: usize = 11;
/// Nodes in the full box.
pub const N_NODES: usize = NR * NR * NR * NU * NU * NU;

/// The label this table carries. It says the model, the arithmetic and the route, and it
/// deliberately does not say "exact", which would be a claim about the world.
pub const QUATERNARY_PROVENANCE: &str =
    "engine-computed STO-3G FCI (O,H,H,H) four-body term, general N-centre route, f64";

/// `exp(STRETCH_A) - 1`, the stretch's normaliser.
fn expa1() -> f64 {
    STRETCH_A.exp() - 1.0
}

/// Physical radius of radial node `i`.
pub fn node_r(i: usize) -> f64 {
    let t = i as f64 / (NR - 1) as f64;
    R_LO + (R_HI - R_LO) * ((STRETCH_A * t).exp() - 1.0) / expa1()
}

/// The stretch's inverse: the grid parameter `t in [0,1]` of a physical radius.
pub fn tau_of_r(r: f64) -> f64 {
    let z = (r - R_LO) / (R_HI - R_LO) * expa1() + 1.0;
    z.max(1e-300).ln() / STRETCH_A
}

/// `dt/dr`, needed to chain the interpolant's index-space slope into physical space.
pub fn dtau_dr(r: f64) -> f64 {
    let z = (r - R_LO) / (R_HI - R_LO) * expa1() + 1.0;
    expa1() / (STRETCH_A * (R_HI - R_LO) * z.max(1e-300))
}

/// Physical cosine of cosine-node `k`.
pub fn node_u(k: usize) -> f64 {
    U_LO + (U_HI - U_LO) * k as f64 / (NU - 1) as f64
}

/// The canonical linear index of a node. Radial axes are slowest, cosine axes fastest, and
/// within each group the order is `(1, 2, 3)` and `(12, 23, 31)`.
#[inline]
pub fn node_index(i: [usize; 3], k: [usize; 3]) -> usize {
    ((((i[0] * NR + i[1]) * NR + i[2]) * NU + k[0]) * NU + k[1]) * NU + k[2]
}

// ===================================================================== the symmetry

/// The six relabellings of the three hydrogens, as images of `(0, 1, 2)`.
pub const PERMS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

/// Which slot of a `[u12, u23, u31]` array holds the cosine of the pair `{a, b}`.
#[inline]
pub fn pair_slot(a: usize, b: usize) -> usize {
    match (a.min(b), a.max(b)) {
        (0, 1) => 0,
        (1, 2) => 1,
        (0, 2) => 2,
        _ => unreachable!("a hydrogen pair is one of three"),
    }
}

/// Apply a relabelling to a coordinate pair, returning the relabelled `(R, u)`.
///
/// `sigma[a]` is the hydrogen that ends up in slot `a`, so the new `R_a` is the old
/// `R_{sigma[a]}` and the new cosine of `{a,b}` is the old cosine of `{sigma[a], sigma[b]}`
/// — the two move together, which is exactly the fact the old sorted form lost.
#[inline]
pub fn apply_perm(r: [f64; 3], u: [f64; 3], sigma: [usize; 3]) -> ([f64; 3], [f64; 3]) {
    (
        [r[sigma[0]], r[sigma[1]], r[sigma[2]]],
        [
            u[pair_slot(sigma[0], sigma[1])],
            u[pair_slot(sigma[1], sigma[2])],
            u[pair_slot(sigma[2], sigma[0])],
        ],
    )
}

/// The canonical representative of a geometry's relabelling orbit: the lexicographic least
/// of the six relabelled 6-tuples, with the relabelling that achieved it.
///
/// Comparisons only, so this is exact in f64 and the same geometry presented under any of
/// the six labellings produces the identical arrays BIT FOR BIT, not to within a tolerance.
/// The returned permutation is what un-permutes a gradient back to the caller's labelling.
pub fn canonical_ohhh(r: [f64; 3], u: [f64; 3]) -> ([f64; 3], [f64; 3], [usize; 3]) {
    let mut best_r = [0.0f64; 3];
    let mut best_u = [0.0f64; 3];
    let mut best_s = [0usize; 3];
    let mut have = false;
    for &s in PERMS.iter() {
        let (cr, cu) = apply_perm(r, u, s);
        if !have {
            best_r = cr;
            best_u = cu;
            best_s = s;
            have = true;
            continue;
        }
        // Lexicographic on the concatenated 6-tuple (R first, then the cosines).
        let mut take = false;
        for t in 0..6 {
            let (a, b) = if t < 3 {
                (cr[t], best_r[t])
            } else {
                (cu[t - 3], best_u[t - 3])
            };
            if a < b {
                take = true;
                break;
            }
            if a > b {
                break;
            }
        }
        if take {
            best_r = cr;
            best_u = cu;
            best_s = s;
        }
    }
    (best_r, best_u, best_s)
}

/// The same, on grid INDICES rather than physical coordinates. Used to fill the orbit of a
/// solved node, and exact for the same reason (integer comparisons).
pub fn canonical_index(i: [usize; 3], k: [usize; 3]) -> ([usize; 3], [usize; 3]) {
    let mut best: Option<([usize; 3], [usize; 3])> = None;
    for &s in PERMS.iter() {
        let ci = [i[s[0]], i[s[1]], i[s[2]]];
        let ck = [
            k[pair_slot(s[0], s[1])],
            k[pair_slot(s[1], s[2])],
            k[pair_slot(s[2], s[0])],
        ];
        best = Some(match best {
            None => (ci, ck),
            Some((bi, bk)) => {
                let cand = [ci[0], ci[1], ci[2], ck[0], ck[1], ck[2]];
                let cur = [bi[0], bi[1], bi[2], bk[0], bk[1], bk[2]];
                if cand < cur {
                    (ci, ck)
                } else {
                    (bi, bk)
                }
            }
        });
    }
    best.unwrap()
}

// ================================================================== embeddability

/// The Gram determinant of the three unit vectors from O. Non-negative exactly where the
/// three cosines describe three directions that exist in `R^3`.
#[inline]
pub fn gram_det(u: [f64; 3]) -> f64 {
    1.0 + 2.0 * u[0] * u[1] * u[2] - u[0] * u[0] - u[1] * u[1] - u[2] * u[2]
}

/// The largest `t in [0,1]` with `G(t*u) >= 0`, and the scaled triple.
///
/// `G(t*u) = 1 + 2 t^3 P - t^2 Q` with `P = u12 u23 u31` and `Q = sum u^2`, which is `1` at
/// `t = 0` and continuous, so a bisection on `[0, 1]` always brackets. Scaling commutes
/// with permuting the three cosines, so this continuation is `S3`-equivariant and the
/// orbit fill stays exact — that is the whole reason for choosing a RADIAL projection
/// rather than a nearest-point one.
pub fn elliptope_scale(u: [f64; 3]) -> ([f64; 3], f64) {
    if gram_det(u) >= 0.0 {
        return (u, 1.0);
    }
    let p = u[0] * u[1] * u[2];
    let q = u[0] * u[0] + u[1] * u[1] + u[2] * u[2];
    let g = |t: f64| 1.0 + 2.0 * t * t * t * p - t * t * q;
    // g(0) = 1 > 0 and g(1) < 0 here, so the root is bracketed.
    let (mut lo, mut hi) = (0.0f64, 1.0f64);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if g(mid) >= 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    ([lo * u[0], lo * u[1], lo * u[2]], lo)
}

/// Place four atoms from three O-H distances and the three H-O-H cosines.
///
/// O sits at the origin and the three hydrogens along unit vectors whose Gram matrix is
/// the cosine matrix. The vectors come from a Cholesky factorisation of that matrix, which
/// is the textbook construction and — unlike building an explicit frame — does not divide
/// by `sin(theta_12)` and so does not degenerate when the first two hydrogens are
/// collinear with O.
///
/// Returns `None` only when the cosines are not embeddable even after the tolerance; call
/// [`elliptope_scale`] first if the caller wants the continuation instead.
pub fn embed_ohhh(r: [f64; 3], u: [f64; 3]) -> Option<[[f64; 3]; 4]> {
    // The planar configurations sit exactly ON the boundary, and the equilateral C3v point
    // among them evaluates to -2.2e-16 once a divide has happened. A degenerate geometry is
    // not an unrealisable one, so the fence is a tolerance and the value is clamped.
    const TOL: f64 = -1e-12;
    if !(gram_det(u) >= TOL) {
        return None;
    }
    // Cholesky of [[1,u12,u31],[u12,1,u23],[u31,u23,1]].
    let l00 = 1.0;
    let l10 = u[0];
    let l20 = u[2];
    let l11sq = 1.0 - l10 * l10;
    if !(l11sq >= TOL) {
        return None;
    }
    let l11 = l11sq.max(0.0).sqrt();
    let l21 = if l11 > 1e-14 {
        (u[1] - l20 * l10) / l11
    } else {
        // The first two directions coincide or are antipodal; the third's component in the
        // vanished direction is not determined by the cosines. Zero is the continuation.
        0.0
    };
    let l22sq = 1.0 - l20 * l20 - l21 * l21;
    if !(l22sq >= TOL) {
        return None;
    }
    let l22 = l22sq.max(0.0).sqrt();
    Some([
        [0.0, 0.0, 0.0],
        [r[0] * l00, 0.0, 0.0],
        [r[1] * l10, r[1] * l11, 0.0],
        [r[2] * l20, r[2] * l21, r[2] * l22],
    ])
}

/// The `(R, u)` coordinates of a Cartesian quadruple, O first.
pub fn internals_ohhh(c: &[[f64; 3]; 4]) -> ([f64; 3], [f64; 3]) {
    let v = |a: usize| {
        [
            c[a][0] - c[0][0],
            c[a][1] - c[0][1],
            c[a][2] - c[0][2],
        ]
    };
    let (v1, v2, v3) = (v(1), v(2), v(3));
    let n = |a: [f64; 3]| (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().max(1e-12);
    let (r1, r2, r3) = (n(v1), n(v2), n(v3));
    let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    (
        [r1, r2, r3],
        [
            (dot(v1, v2) / (r1 * r2)).clamp(-1.0, 1.0),
            (dot(v2, v3) / (r2 * r3)).clamp(-1.0, 1.0),
            (dot(v3, v1) / (r3 * r1)).clamp(-1.0, 1.0),
        ],
    )
}

// ======================================================================= the table

/// What the whole surface carries rather than one node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuaternaryMeta {
    pub n_nodes: usize,
    pub nr: usize,
    pub nu: usize,
    pub r_lo: f64,
    pub r_hi: f64,
    pub u_lo: f64,
    pub u_hi: f64,
    pub stretch_a: f64,
    /// Largest `|dE4|` on the grid.
    pub peak: f64,
    /// Nodes actually solved (the canonical, embeddable ones).
    pub solves: usize,
    /// Nodes filled by the equivariant radial continuation: not geometries.
    pub continued: usize,
}

impl QuaternaryMeta {
    pub fn empty() -> Self {
        Self {
            n_nodes: 0,
            nr: NR,
            nu: NU,
            r_lo: R_LO,
            r_hi: R_HI,
            u_lo: U_LO,
            u_hi: U_HI,
            stretch_a: STRETCH_A,
            peak: 0.0,
            solves: 0,
            continued: 0,
        }
    }
}

/// The tabulated four-body surface.
#[derive(Clone)]
pub struct QuaternaryTable {
    v: Vec<f64>,
    /// Per node: was this a real geometry, or the continuation? Parallel to `v`.
    real: Vec<bool>,
    /// Per node: has anything been written here at all? Separate from `real` and from the
    /// value, because a node whose `dE4` is genuinely zero is written, and inferring
    /// writtenness from `v[idx] == 0.0` would miscount exactly the far-field nodes the
    /// cutoff makes most common.
    written: Vec<bool>,
    filled: usize,
    pub loaded: bool,
    pub meta: QuaternaryMeta,
}

impl Default for QuaternaryTable {
    fn default() -> Self {
        Self::empty()
    }
}

impl QuaternaryTable {
    pub fn empty() -> Self {
        Self {
            v: Vec::new(),
            real: Vec::new(),
            written: Vec::new(),
            filled: 0,
            loaded: false,
            meta: QuaternaryMeta::empty(),
        }
    }

    /// Begin filling a table: allocates the full box.
    pub fn begin() -> Self {
        Self {
            v: vec![0.0; N_NODES],
            real: vec![false; N_NODES],
            written: vec![false; N_NODES],
            filled: 0,
            loaded: false,
            meta: QuaternaryMeta::empty(),
        }
    }

    /// Write one node AND its whole relabelling orbit. Returns how many box entries it
    /// touched, which is the orbit size (1, 3 or 6).
    ///
    /// What the orbit fill buys is the STENCIL, not the symmetry. `eval` canonicalises
    /// before it looks anything up, so all six presentations of a geometry reach
    /// `eval_grid` with identical bits and the returned value is bit-identical whatever the
    /// table contains — which means a symmetry test against `eval` is blind to the table's
    /// contents and cannot convict a wrong orbit fill. (Measured: a 3.05e-5 Ha corruption
    /// planted in one node moved the symmetry reading by exactly zero.) The fill's job is
    /// that the stencil near a relabelling boundary stands on real values from both sides,
    /// and the check with the power to convict it is the CONTINUITY gate C1, which moved by
    /// 4.1e-7 Ha/bohr on that same plant — five orders above its band.
    pub fn set_orbit(&mut self, i: [usize; 3], k: [usize; 3], value: f64, real: bool) -> usize {
        let mut touched = 0usize;
        let mut seen: Vec<usize> = Vec::with_capacity(6);
        for &s in PERMS.iter() {
            // sigma sends slot a to hydrogen s[a]; the image node's slot a holds i[s[a]].
            let ii = [i[s[0]], i[s[1]], i[s[2]]];
            let kk = [
                k[pair_slot(s[0], s[1])],
                k[pair_slot(s[1], s[2])],
                k[pair_slot(s[2], s[0])],
            ];
            let idx = node_index(ii, kk);
            if seen.contains(&idx) {
                continue;
            }
            seen.push(idx);
            if !self.written[idx] {
                self.filled += 1;
                self.written[idx] = true;
            }
            self.v[idx] = value;
            self.real[idx] = real;
            touched += 1;
        }
        touched
    }

    pub fn node(&self, i: [usize; 3], k: [usize; 3]) -> f64 {
        self.v[node_index(i, k)]
    }

    pub fn finish(&mut self, meta: QuaternaryMeta) -> bool {
        if self.v.len() != N_NODES {
            return false;
        }
        let mut peak = 0.0f64;
        for x in self.v.iter() {
            if !x.is_finite() {
                return false;
            }
            if x.abs() > peak {
                peak = x.abs();
            }
        }
        self.meta = QuaternaryMeta {
            n_nodes: N_NODES,
            peak,
            ..meta
        };
        self.loaded = true;
        true
    }

    /// How many box entries carry a value.
    pub fn filled(&self) -> usize {
        self.filled
    }

    /// The surface and its six partial derivatives at a canonical coordinate, in index
    /// space. Private because callers want [`Self::eval`], which handles the symmetry.
    fn eval_grid(&self, r: [f64; 3], u: [f64; 3]) -> (f64, [f64; 6]) {
        // Index-space coordinates, clamped to the domain. WHICH AXES CLAMPED IS RECORDED,
        // because a clamp makes the value constant along that axis and the derivative of a
        // constant is zero -- reporting a nonzero slope there would hand the integrator a
        // force for a direction the energy does not actually depend on. Getting this wrong
        // is not a rounding matter: `dtau_dr` evaluated at an unclamped radius runs into
        // its own `1e-300` floor below `R_LO - (R_HI - R_LO)/(exp(a) - 1) = 0.633` bohr and
        // returns ~4.6e296 Ha/bohr, which would destroy a trajectory in one step. Measured
        // by `examples/de4_certify.rs`, probe D1.
        let mut clamped = [false; 6];
        let mut tr = [0.0f64; 3];
        for a in 0..3 {
            let raw = tau_of_r(r[a]) * (NR - 1) as f64;
            tr[a] = raw.clamp(0.0, (NR - 1) as f64);
            clamped[a] = tr[a] != raw;
        }
        let mut tu = [0.0f64; 3];
        for a in 0..3 {
            let raw = (u[a] - U_LO) / (U_HI - U_LO) * (NU - 1) as f64;
            tu[a] = raw.clamp(0.0, (NU - 1) as f64);
            clamped[3 + a] = tu[a] != raw;
        }

        // Per-axis Catmull-Rom base index, value weights and index-derivative weights.
        let mut base = [0usize; 6];
        let mut w = [[0.0f64; 4]; 6];
        let mut dw = [[0.0f64; 4]; 6];
        for a in 0..3 {
            let (b, ww, dd) = cr_weights(NR, tr[a]);
            base[a] = b;
            w[a] = ww;
            dw[a] = dd;
        }
        for a in 0..3 {
            let (b, ww, dd) = cr_weights(NU, tu[a]);
            base[3 + a] = b;
            w[3 + a] = ww;
            dw[3 + a] = dd;
        }

        // Gather the 4^6 stencil once, then contract it seven times.
        let mut stencil = [0.0f64; 4096];
        let mut p = 0usize;
        for a0 in 0..4 {
            // `cr_weights` clamps its base to `n - 4`, so `base + 3` is always in range;
            // no clamp is needed here and adding one would hide a future off-by-one.
            let i0 = base[0] + a0;
            for a1 in 0..4 {
                let i1 = base[1] + a1;
                for a2 in 0..4 {
                    let i2 = base[2] + a2;
                    for a3 in 0..4 {
                        let k0 = base[3] + a3;
                        for a4 in 0..4 {
                            let k1 = base[4] + a4;
                            for a5 in 0..4 {
                                let k2 = base[5] + a5;
                                stencil[p] = self.v[node_index([i0, i1, i2], [k0, k1, k2])];
                                p += 1;
                            }
                        }
                    }
                }
            }
        }

        // Contract axis by axis, fastest axis first. `which` selects the derivative axis;
        // 6 means "no derivative", i.e. the value.
        let contract = |which: usize| -> f64 {
            let mut buf = stencil;
            let mut len = 4096usize;
            for axis in (0..6).rev() {
                let ww = if axis == which { &dw[axis] } else { &w[axis] };
                let out = len / 4;
                for q in 0..out {
                    let s = q * 4;
                    buf[q] = buf[s] * ww[0]
                        + buf[s + 1] * ww[1]
                        + buf[s + 2] * ww[2]
                        + buf[s + 3] * ww[3];
                }
                len = out;
            }
            buf[0]
        };

        let value = contract(6);
        let mut d = [0.0f64; 6];
        for a in 0..6 {
            d[a] = contract(a);
        }
        // Chain index-space slopes into physical space, and zero every axis whose
        // coordinate was clamped: along such an axis the returned VALUE does not vary, so
        // the returned gradient must not either. Without this the gradient is not the
        // derivative of the value the same call returned.
        for a in 0..3 {
            d[a] = if clamped[a] { 0.0 } else { d[a] * dtau_dr(r[a]) * (NR - 1) as f64 };
            d[3 + a] = if clamped[3 + a] {
                0.0
            } else {
                d[3 + a] * (NU - 1) as f64 / (U_HI - U_LO)
            };
        }
        (value, d)
    }

    /// The four-body term and its six partial derivatives at a geometry given by its three
    /// O-H distances and three H-O-H cosines, in the CALLER's hydrogen labelling.
    ///
    /// Returns exactly zero outside the domain, which is the cutoff, not an approximation:
    /// `|dE4|` is 4.9e-5 Ha at 6.1 bohr by measurement.
    pub fn eval(&self, r: [f64; 3], u: [f64; 3]) -> (f64, [f64; 6]) {
        if !self.loaded {
            return (0.0, [0.0; 6]);
        }
        // The cutoff fence. Negated comparisons so a NaN is rejected rather than accepted.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        if !(r[0] <= R_HI) || !(r[1] <= R_HI) || !(r[2] <= R_HI) {
            return (0.0, [0.0; 6]);
        }
        let (cr, cu, sigma) = canonical_ohhh(r, u);
        let (val, d) = self.eval_grid(cr, cu);
        // Un-permute: the canonical slot `a` carried the caller's hydrogen `sigma[a]`, and
        // the canonical cosine slot for `{a,b}` carried the caller's pair `{s[a], s[b]}`.
        let mut out = [0.0f64; 6];
        for a in 0..3 {
            out[sigma[a]] = d[a];
        }
        for a in 0..3 {
            let b = (a + 1) % 3;
            out[3 + pair_slot(sigma[a], sigma[b])] = d[3 + a];
        }
        (val, out)
    }

    /// The four-body term and the Cartesian force on each atom, O first.
    ///
    /// The chain rule from `(R, u)` to positions is done here rather than by the caller so
    /// no consumer has to know the sign convention: the returned array is the FORCE,
    /// `-dE/dx`, matching the renderer's contract.
    pub fn eval_cartesian(&self, c: &[[f64; 3]; 4]) -> (f64, [[f64; 3]; 4]) {
        let (r, u) = internals_ohhh(c);
        let (val, d) = self.eval(r, u);
        if val == 0.0 && d.iter().all(|x| *x == 0.0) {
            return (0.0, [[0.0; 3]; 4]);
        }
        // Unit vectors and their lengths.
        let mut e = [[0.0f64; 3]; 3];
        for a in 0..3 {
            let len = r[a].max(1e-12);
            for t in 0..3 {
                e[a][t] = (c[a + 1][t] - c[0][t]) / len;
            }
        }
        let mut f = [[0.0f64; 3]; 4];
        // Radial part: dE/dR_a acts along e_a.
        for a in 0..3 {
            for t in 0..3 {
                let g = d[a] * e[a][t];
                f[a + 1][t] -= g;
                f[0][t] += g;
            }
        }
        // Angular part: for u_ab = e_a . e_b,
        //   d u_ab / d x_a = (e_b - u_ab e_a) / R_a,   and symmetrically in b.
        for (slot, (a, b)) in [(0usize, 1usize), (1, 2), (2, 0)].iter().enumerate() {
            let (a, b) = (*a, *b);
            let gu = d[3 + slot];
            if gu == 0.0 {
                continue;
            }
            let uab = e[a][0] * e[b][0] + e[a][1] * e[b][1] + e[a][2] * e[b][2];
            for t in 0..3 {
                let da = (e[b][t] - uab * e[a][t]) / r[a].max(1e-12);
                let db = (e[a][t] - uab * e[b][t]) / r[b].max(1e-12);
                f[a + 1][t] -= gu * da;
                f[b + 1][t] -= gu * db;
                f[0][t] += gu * (da + db);
            }
        }
        (val, f)
    }
}

// ============================================================ the staked witness set

/// The 40 staked held-out witness geometries of `DE4_TABLE_PREREG.md`, built exactly as
/// `tests/quaternary.rs` builds them: a water monomer at its own minimum plus a third
/// hydrogen on eight directions at five radii, with the close-contact ones dropped.
///
/// This lives in the library rather than in an example because THREE consumers need the
/// identical set — `examples/de4_price.rs` priced it, `examples/de4_certify.rs` gates T1,
/// T2 and T3 on it, and `tests/quaternary.rs` stakes it. A second copy of a held-out set
/// is a held-out set that can silently drift, which is WORKBENCH_FSD.md clause WB-8.7's
/// DRY residual with the accuracy claim resting on it.
pub fn staked_witnesses() -> Vec<[[f64; 3]; 4]> {
    const R_W: f64 = 1.9435740105;
    const TH_W: f64 = 96.75788837;
    let pi = std::f64::consts::PI;
    let th = TH_W * pi / 180.0;
    let o = [0.0f64, 0.0, 0.0];
    let h1 = [R_W * (th / 2.0).cos(), R_W * (th / 2.0).sin(), 0.0];
    let h2 = [R_W * (th / 2.0).cos(), -R_W * (th / 2.0).sin(), 0.0];
    let dirs: [[f64; 3]; 8] = [
        [-1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [-0.7071, 0.0, 0.7071],
        [-0.7071, 0.0, -0.7071],
        [0.5774, 0.5774, 0.5774],
    ];
    let radii = [1.4f64, 1.8, 2.2, 2.8, 3.6];
    let dist = |a: &[f64; 3], b: &[f64; 3]| {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };
    let mut out = Vec::new();
    for d in dirs.iter() {
        let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        for r in radii.iter() {
            let p = [d[0] / n * r, d[1] / n * r, d[2] / n * r];
            if dist(&o, &p) < 0.9 || dist(&h1, &p) < 0.9 || dist(&h2, &p) < 0.9 {
                continue;
            }
            out.push([o, h1, h2, p]);
        }
    }
    out
}

// ================================================================= reading an artifact

/// The string value of a top-level JSON key, without a JSON library.
///
/// `holon-chem` carries no serde (see `Cargo.toml`'s dependency rule), and
/// [`crate::water::from_text`] reads its own artifact by scanning lines for prefixes. This
/// is that same spirit one format up: find the key, take the quoted run after the colon.
/// Deliberately shallow — it finds the FIRST occurrence, which is correct for this schema
/// because every key it is asked for is top-level and unique, and a nested duplicate would
/// be a different file than the one this build writes.
fn json_str<'a>(src: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{key}\"");
    let at = src.find(&pat)? + pat.len();
    let rest = &src[at..];
    let colon = rest.find(':')?;
    let after = &rest[colon + 1..];
    let open = after.find('"')?;
    let tail = &after[open + 1..];
    let close = tail.find('"')?;
    Some(&tail[..close])
}

/// The integer value of a top-level JSON key.
fn json_usize(src: &str, key: &str) -> Option<usize> {
    let pat = format!("\"{key}\"");
    let at = src.find(&pat)? + pat.len();
    let rest = &src[at..];
    let colon = rest.find(':')?;
    let after = rest[colon + 1..].trim_start();
    let end = after
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after.len());
    if end == 0 {
        return None;
    }
    after[..end].parse().ok()
}

impl QuaternaryTable {
    /// Read a generated artifact — `holon-tables`' `de4_table` output, schema
    /// `DE4TABLE/quaternary-table/v1`.
    ///
    /// # What this REFUSES, and why the refusal is the point
    ///
    /// The artifact's `grid_line` must be a **byte-exact** match for this build's
    /// [`grid_line`]. Nothing here reinterprets a file onto a different grid: the values
    /// are node values and a node is only a node relative to its axes, so a file whose
    /// grid line differs is a list of numbers, not a table. That is exactly the rule
    /// [`crate::water::from_text`] enforces on the three-body artifact, and it is enforced
    /// here for the same reason and by the same mechanism (compare, do not adapt).
    ///
    /// The value array is `values_hex`, 16-hex-digit IEEE-754 bit patterns in canonical
    /// [`node_index`] order over the FULL box — bits rather than decimal because a decimal
    /// round-trip would put a tolerance where the artifact is compared for bit-identity.
    /// A count that is not exactly [`N_NODES`] is refused.
    ///
    /// # What the artifact CANNOT carry, and how that is handled
    ///
    /// The file records `voided` as a COUNT and not per node, so the per-node
    /// continued/real flag is not recoverable from it by any loader. It is not guessed:
    /// it is RECOMPUTED from the definition, since a node is a continuation exactly when
    /// its cosine triple is outside the elliptope — `gram_det < 0` — which is a property
    /// of the grid, not of the run. The artifact's own `voided` count is carried through
    /// in `meta.continued` so a consumer can compare the two and see a disagreement.
    pub fn from_artifact(src: &str) -> Option<QuaternaryTable> {
        // The grid line, byte-exact or nothing. `de4_table` writes it through
        // `.replace('"', "'")`; `grid_line()` contains no quote, so the two are the same
        // bytes and this comparison is the one the writer intended.
        let want = grid_line();
        let have = json_str(src, "grid_line")?;
        if have.as_bytes() != want.as_bytes() {
            return None;
        }

        // The values. One pass over the bytes after the `values_hex` key: a 70 MB artifact
        // is not the place for a per-token allocation.
        let key = "\"values_hex\"";
        let at = src.find(key)? + key.len();
        let rest = &src[at..];
        let open = rest.find('[')?;
        let body = &rest[open + 1..];
        let close = body.find(']')?;
        let body = &body[..close];

        let mut v: Vec<f64> = Vec::with_capacity(N_NODES);
        let bytes = body.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            if bytes[i] != b'"' {
                i += 1;
                continue;
            }
            let s = i + 1;
            let mut e = s;
            while e < bytes.len() && bytes[e] != b'"' {
                e += 1;
            }
            if e >= bytes.len() {
                return None;
            }
            let tok = &body[s..e];
            if tok.len() != 16 {
                return None;
            }
            v.push(f64::from_bits(u64::from_str_radix(tok, 16).ok()?));
            if v.len() > N_NODES {
                return None;
            }
            i = e + 1;
        }
        if v.len() != N_NODES {
            return None;
        }

        // The continuation flag, recomputed from the definition rather than trusted.
        let mut real = vec![false; N_NODES];
        for k0 in 0..NU {
            for k1 in 0..NU {
                for k2 in 0..NU {
                    let ok = gram_det([node_u(k0), node_u(k1), node_u(k2)]) >= 0.0;
                    for i0 in 0..NR {
                        for i1 in 0..NR {
                            for i2 in 0..NR {
                                real[node_index([i0, i1, i2], [k0, k1, k2])] = ok;
                            }
                        }
                    }
                }
            }
        }

        let mut t = QuaternaryTable {
            v,
            real,
            written: vec![true; N_NODES],
            filled: N_NODES,
            loaded: false,
            meta: QuaternaryMeta::empty(),
        };
        let meta = QuaternaryMeta {
            solves: json_usize(src, "solved").unwrap_or(0),
            continued: json_usize(src, "voided").unwrap_or(0),
            ..QuaternaryMeta::empty()
        };
        if !t.finish(meta) {
            return None;
        }
        Some(t)
    }

    /// Whether node `idx` is a real geometry rather than the equivariant continuation.
    pub fn is_real(&self, i: [usize; 3], k: [usize; 3]) -> bool {
        self.real[node_index(i, k)]
    }
}

/// The one line that IS this table's manifest: a file whose grid line is not a byte-exact
/// match for this build's is refused rather than reinterpreted, which is the difference
/// between loading a table and loading numbers.
pub fn grid_line() -> String {
    format!(
        "# grid: NR={} NU={} R_LO={} R_HI={} STRETCH_A={} U_LO={} U_HI={}",
        NR, NU, R_LO, R_HI, STRETCH_A, U_LO, U_HI
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_form_is_s3_invariant_and_separates_what_sorting_merged() {
        let r = [1.9f64, 2.4, 3.0];
        let ua = [2.6f64 * 0.0 - 0.3, -0.1, 0.2];
        // Every relabelling of one geometry lands on the identical canonical tuple.
        let (br, bu, _) = canonical_ohhh(r, ua);
        for &s in PERMS.iter() {
            let (pr, pu) = apply_perm(r, ua, s);
            let (cr, cu, _) = canonical_ohhh(pr, pu);
            assert_eq!(cr, br, "relabelling {s:?} moved the canonical R");
            assert_eq!(cu, bu, "relabelling {s:?} moved the canonical u");
        }
        // Two geometries that independent sorting merges must NOT share a canonical
        // address: same R triple, the cosines attached to different pairs. Independent
        // sorting sends both to (sorted R, sorted u); this must not.
        let ub = [ua[0], ua[2], ua[1]];
        let (ra, uaa, _) = canonical_ohhh(r, ua);
        let (rb, ubb, _) = canonical_ohhh(r, ub);
        assert_ne!(
            (ra, uaa),
            (rb, ubb),
            "canonical_ohhh merged two geometries that independent sorting also merged, \
             which is the defect it exists to repair"
        );
    }

    #[test]
    fn the_continuation_commutes_with_relabelling() {
        // A cosine triple well outside the elliptope.
        let u = [-0.9f64, -0.9, -0.9];
        assert!(gram_det(u) < 0.0, "the test point must be outside, or it tests nothing");
        let (su, t) = elliptope_scale(u);
        assert!(t > 0.0 && t < 1.0);
        assert!(gram_det(su) >= -1e-12);
        for &s in PERMS.iter() {
            let pu = [
                u[pair_slot(s[0], s[1])],
                u[pair_slot(s[1], s[2])],
                u[pair_slot(s[2], s[0])],
            ];
            let (psu, pt) = elliptope_scale(pu);
            assert_eq!(pt, t, "the scale must not depend on the labelling");
            let expect = [
                su[pair_slot(s[0], s[1])],
                su[pair_slot(s[1], s[2])],
                su[pair_slot(s[2], s[0])],
            ];
            assert_eq!(psu, expect, "the continuation is not S3-equivariant");
        }
    }

    #[test]
    fn embedding_reproduces_its_own_coordinates() {
        for &(r, u) in [
            ([1.9435740105f64, 1.9435740105, 3.0], [-0.117f64, -0.2, -0.2]),
            ([2.0, 2.5, 3.1], [-0.5, -0.3, 0.1]),
            ([2.2, 2.2, 2.2], [-0.5, -0.5, -0.5]), // the exactly-planar C3v point
        ]
        .iter()
        {
            let g = embed_ohhh(r, u).expect("embeddable");
            let (br, bu) = internals_ohhh(&g);
            for a in 0..3 {
                assert!((br[a] - r[a]).abs() < 1e-10, "R{a}: {} vs {}", br[a], r[a]);
                assert!((bu[a] - u[a]).abs() < 1e-10, "u{a}: {} vs {}", bu[a], u[a]);
            }
        }
    }

    #[test]
    fn the_grid_axes_round_trip() {
        for i in 0..NR {
            let r = node_r(i);
            let t = tau_of_r(r);
            assert!(
                (t - i as f64 / (NR - 1) as f64).abs() < 1e-12,
                "radial axis does not invert at node {i}"
            );
        }
        assert!((node_r(0) - R_LO).abs() < 1e-12);
        assert!((node_r(NR - 1) - R_HI).abs() < 1e-12);
        assert!((node_u(0) - U_LO).abs() < 1e-15);
        assert!((node_u(NU - 1) - U_HI).abs() < 1e-15);
    }

    /// The whole artifact, written the way `holon-tables`' `de4_table` writes it, over a
    /// caller-supplied grid line. Small values so the string stays cheap to build.
    fn synth_artifact_src(grid_line_text: &str) -> String {
        let mut s = String::with_capacity(N_NODES * 24 + 1024);
        s.push_str("{\n");
        s.push_str("  \"schema\": \"DE4TABLE/quaternary-table/v1\",\n");
        s.push_str(&format!("  \"grid_line\": \"{grid_line_text}\",\n"));
        s.push_str("  \"solved\": 7, \"mirrored\": 11, \"voided\": 13,\n");
        s.push_str("  \"values_hex\": [\n");
        for n in 0..N_NODES {
            let v = (n % 97) as f64 * 1e-6;
            let comma = if n + 1 == N_NODES { "" } else { "," };
            s.push_str(&format!("    \"{:016x}\"{}\n", v.to_bits(), comma));
        }
        s.push_str("  ]\n}\n");
        s
    }

    #[test]
    fn a_mismatched_grid_line_is_refused_and_a_matching_one_loads() {
        // The refusal first, and on a grid line that differs by ONE character — the point
        // is byte-exactness, not plausibility. Built cheaply: the loader must reject on the
        // grid line before it ever looks at the values, so this source carries none.
        let bad = grid_line().replace("NR=13", "NR=15");
        assert_ne!(bad, grid_line(), "the mutation must actually change the line");
        let stub = format!("{{\n  \"grid_line\": \"{bad}\",\n  \"values_hex\": [\n  ]\n}}\n");
        assert!(
            QuaternaryTable::from_artifact(&stub).is_none(),
            "a table on a different grid was loaded as if it were this one"
        );

        // And the matching one loads, with the values landing at the right addresses.
        let good = synth_artifact_src(&grid_line());
        let t = QuaternaryTable::from_artifact(&good).expect("a matching artifact must load");
        assert!(t.loaded);
        assert_eq!(t.filled(), N_NODES);
        assert_eq!(t.meta.n_nodes, N_NODES);
        assert_eq!(t.meta.solves, 7);
        assert_eq!(t.meta.continued, 13);
        for &(i, k) in [
            ([0usize, 0, 0], [0usize, 0, 0]),
            ([1, 0, 2], [3, 4, 5]),
            ([NR - 1, NR - 1, NR - 1], [NU - 1, NU - 1, NU - 1]),
        ]
        .iter()
        {
            let n = node_index(i, k);
            assert_eq!(t.node(i, k), (n % 97) as f64 * 1e-6, "node {n} came back wrong");
        }

        // A truncated value array is refused too: a short table is not a table.
        let short = good.replacen("    \"0000000000000000\",\n", "", 1);
        assert_ne!(short.len(), good.len(), "the truncation must actually remove a node");
        assert!(
            QuaternaryTable::from_artifact(&short).is_none(),
            "an artifact one node short was accepted"
        );
    }

    #[test]
    fn a_clamped_axis_reports_no_force_along_itself() {
        // The gradient a call returns must be the derivative of the value that same call
        // returns. Outside the box the interpolant clamps, so the value stops varying and
        // the slope must go with it. Before this was fixed the chain factor `dtau_dr` was
        // still evaluated at the UNCLAMPED radius, and below 0.633 bohr -- where its own
        // 1e-300 floor bites -- it returned ~4.6e296 Ha/bohr, which is one integrator step
        // away from destroying a trajectory.
        let mut t = QuaternaryTable::begin();
        for i0 in 0..NR {
            for i1 in 0..NR {
                for i2 in 0..NR {
                    for k0 in 0..NU {
                        for k1 in 0..NU {
                            for k2 in 0..NU {
                                let (ci, ck) = canonical_index([i0, i1, i2], [k0, k1, k2]);
                                if (ci, ck) != ([i0, i1, i2], [k0, k1, k2]) {
                                    continue;
                                }
                                let r = [node_r(i0), node_r(i1), node_r(i2)];
                                let u = [node_u(k0), node_u(k1), node_u(k2)];
                                let v = r[0] * r[1] * r[2] + u[0] + u[1] + u[2];
                                t.set_orbit([i0, i1, i2], [k0, k1, k2], v, true);
                            }
                        }
                    }
                }
            }
        }
        assert!(t.finish(QuaternaryMeta::empty()));

        // Below R_LO on one axis, and below the chain factor's pole on another.
        for &probe in [0.80f64, 0.50, 0.10].iter() {
            let (_, d) = t.eval([probe, 2.4, 3.1], [-0.3, 0.1, -0.55]);
            assert_eq!(
                d[0], 0.0,
                "R1 = {probe} is below R_LO = {R_LO}, so the value does not vary with R1 \
                 and the reported force must be exactly zero, not {}",
                d[0]
            );
            assert!(d.iter().all(|x| x.is_finite()), "a clamped axis produced a non-finite gradient");
        }
        // Above the closed-angle fence, which a near-collinear H-O-H genuinely reaches.
        let (_, d) = t.eval([1.9, 2.4, 3.1], [0.9990, 0.1, -0.55]);
        assert_eq!(d[3], 0.0, "u12 above U_HI = {U_HI} must report no force along u12");
        // And the CONTROL: strictly inside the box the slope is NOT zeroed, or the fix
        // would be a gate that always passes by reporting nothing.
        let (_, d) = t.eval([1.8, 2.4, 3.1], [-0.3, 0.1, -0.55]);
        assert!(
            d[0] != 0.0 && d[3] != 0.0,
            "inside the box the gradient must survive; zeroing everything would make the \
             clamp fix vacuous"
        );
    }

    #[test]
    fn an_orbit_fill_makes_eval_bit_exactly_symmetric() {
        let mut t = QuaternaryTable::begin();
        // A cheap analytic surface that is genuinely S3-symmetric, so the only thing under
        // test is the addressing: if set_orbit or the un-permutation were wrong, the six
        // relabellings would disagree.
        for i0 in 0..NR {
            for i1 in 0..NR {
                for i2 in 0..NR {
                    for k0 in 0..NU {
                        for k1 in 0..NU {
                            for k2 in 0..NU {
                                let (ci, ck) = canonical_index([i0, i1, i2], [k0, k1, k2]);
                                if (ci, ck) != ([i0, i1, i2], [k0, k1, k2]) {
                                    continue;
                                }
                                let r = [node_r(i0), node_r(i1), node_r(i2)];
                                let u = [node_u(k0), node_u(k1), node_u(k2)];
                                let val = r[0] * r[1] * r[2] + u[0] + u[1] + u[2];
                                t.set_orbit([i0, i1, i2], [k0, k1, k2], val, true);
                            }
                        }
                    }
                }
            }
        }
        assert!(t.finish(QuaternaryMeta::empty()));
        let r = [1.7f64, 2.3, 3.1];
        let u = [-0.3f64, 0.1, -0.55];
        let (v0, d0) = t.eval(r, u);
        for &s in PERMS.iter() {
            let (pr, pu) = apply_perm(r, u, s);
            let (v, d) = t.eval(pr, pu);
            assert_eq!(v, v0, "relabelling {s:?} moved the value");
            // The gradient must come back permuted the same way.
            for a in 0..3 {
                assert_eq!(d[a], d0[s[a]], "relabelling {s:?} mis-permuted dR{a}");
            }
            for a in 0..3 {
                let b = (a + 1) % 3;
                assert_eq!(
                    d[3 + pair_slot(a, b)],
                    d0[3 + pair_slot(s[a], s[b])],
                    "relabelling {s:?} mis-permuted du"
                );
            }
        }
    }
}
