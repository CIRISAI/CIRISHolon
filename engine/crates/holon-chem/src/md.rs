//! McMurchie–Davidson integrals over contracted Cartesian Gaussians with `l <= 1`.
//!
//! # Why a second integral module and not an extension of `sto3g.rs`
//!
//! `sto3g.rs` holds the s-only closed forms, and it is the arithmetic the pinned 50-digit
//! H2 referee has already graded to 5e-15 hartree. Rewriting it in a general scheme
//! would move every one of those numbers by a rounding and turn a measured gate into a
//! re-measured one. So the general scheme is written HERE, beside it, and the two are
//! checked against each other at the arguments they share (`tests/md.rs`): same overlap,
//! same kinetic, same nuclear attraction, same two-electron integral, on the same
//! hydrogen contraction, to f64 roundoff. Agreement there is what licenses the new code
//! to carry the elements the old code cannot.
//!
//! # The scheme, in one paragraph
//!
//! A product of two Cartesian Gaussians is re-expanded in HERMITE Gaussians about their
//! centre of charge — coefficients `E_t^{ij}`, built by a two-term recursion — because
//! every integral of a Hermite Gaussian against `1/r` is a derivative of the Boys
//! function, and derivatives of the Boys function satisfy their own recursion (the
//! `R_{tuv}` tensor). The result is that one recursion pair generates overlap, kinetic,
//! nuclear attraction and the two-electron integral for every angular momentum, with no
//! separate closed form per case to transcribe and mistype.
//!
//! Provenance: T. Helgaker, P. Jørgensen, J. Olsen, *Molecular Electronic-Structure
//! Theory* (Wiley 2000), ch. 9; originally L. E. McMurchie and E. R. Davidson,
//! *J. Comput. Phys.* **26**, 218 (1978). Implemented from the recursions, not copied
//! from any code.
//!
//! # Differentiation
//!
//! Every centre coordinate is a [`D2`], so the whole scheme carries `d/dR` and `d2/dR2`
//! alongside the value with no differentiated copy to keep in step — the same choice
//! `sto3g.rs` makes, for the same reason. In a diatomic only the `z` coordinate of the
//! second centre is the variable; the other five are constants and their derivative
//! terms drop out arithmetically.

use crate::dual::D2;
use crate::special::boys_d2_upto;

const PI: f64 = core::f64::consts::PI;

/// `pi^{5/2}`, the two-electron prefactor's transcendental part. The same constant
/// `sto3g.rs` declares, referenced rather than re-typed.
pub const PI_POW_2_5: f64 = crate::sto3g::PI_POW_2_5;

/// Highest angular momentum per basis function. `1` is the first row, `2` supports
/// d-orbitals for the second row and transition elements.
pub const LMAX: usize = 2;

/// Largest Cartesian power the `E` table must reach.
///
/// Not `2 * LMAX`. The kinetic-energy operator differentiates the KET twice, so the
/// overlap table is asked for `j + 2`, which at `LMAX = 2` is 4.
const IMAX: usize = 4;

/// Hermite index bound: `t <= i + j <= 2 * IMAX`, plus one slot the recursion reads past
/// its own top.
const TMAX: usize = 12;

/// Largest total Hermite order the `R` tensor is built to: a `(dd|dd)` quartet has bra
/// order `<= 4` and ket order `<= 4`.
const RMAX: usize = 8;

/// `E_t^{ij}` for one Cartesian direction, indexed `[i][j][t]`.
///
/// Entries outside `t <= i + j` are zero and are read as zero by the recursions, which
/// is what lets the loops run over a rectangle instead of a triangle.
pub struct ETable {
    pub e: [[[D2; TMAX]; IMAX + 1]; IMAX + 1],
}

/// Build the Hermite expansion coefficients for one Cartesian direction.
///
/// ```text
/// E_0^{00}   = exp(-mu X_AB^2),      mu = ab/p,  p = a + b
/// E_t^{i+1,j} = (1/2p) E_{t-1}^{ij} + X_PA E_t^{ij} + (t+1) E_{t+1}^{ij}
/// E_t^{i,j+1} = (1/2p) E_{t-1}^{ij} + X_PB E_t^{ij} + (t+1) E_{t+1}^{ij}
/// ```
pub fn e_table(a: f64, b: f64, xa: D2, xb: D2) -> ETable {
    let p = a + b;
    let mu = a * b / p;
    let inv2p = 1.0 / (2.0 * p);
    let xab = xa - xb;
    let xp = (xa * a + xb * b) * (1.0 / p);
    let xpa = xp - xa;
    let xpb = xp - xb;

    let mut e = [[[D2::c(0.0); TMAX]; IMAX + 1]; IMAX + 1];
    e[0][0][0] = (-(mu * (xab * xab))).exp();

    // Raise `i` along the j = 0 column first: the (i, j) recursion below then has a full
    // left edge to walk from, and neither recursion ever reads an entry it has not yet
    // written.
    for i in 0..IMAX {
        for t in 0..(TMAX - 1) {
            let mut acc = xpa * e[i][0][t] + e[i][0][t + 1] * ((t + 1) as f64);
            if t > 0 {
                acc = acc + e[i][0][t - 1] * inv2p;
            }
            e[i + 1][0][t] = acc;
        }
    }
    // Both loops index rather than iterate because each step WRITES `e[..][j + 1][..]`
    // while READING `e[..][j][..]` and `e[..][j][t + 1]`: the recursion walks the table it
    // is filling, which an iterator over the same slice cannot express.
    #[allow(clippy::needless_range_loop)]
    for i in 0..=IMAX {
        for j in 0..IMAX {
            for t in 0..(TMAX - 1) {
                let mut acc = xpb * e[i][j][t] + e[i][j][t + 1] * ((t + 1) as f64);
                if t > 0 {
                    acc = acc + e[i][j][t - 1] * inv2p;
                }
                e[i][j + 1][t] = acc;
            }
        }
    }
    ETable { e }
}

/// The Hermite Coulomb integrals `R_{tuv}`, indexed `[t][u][v]`.
pub struct RTensor {
    pub r: [[[D2; RMAX + 1]; RMAX + 1]; RMAX + 1],
    work: Box<[[[[D2; RMAX + 1]; RMAX + 1]; RMAX + 1]; RMAX + 1]>,
}

/// Build `R^0_{tuv}(alpha, D)` for every `t + u + v <= lmax`.
///
/// ```text
/// R^n_{000}    = (-2 alpha)^n F_n(alpha |D|^2)
/// R^n_{t+1uv}  = t R^{n+1}_{t-1,u,v} + D_x R^{n+1}_{tuv}      (and likewise u, v)
/// ```
///
/// The auxiliary index `n` is a scaffold: only `n = 0` is an integral, and the tower is
/// built downward in `n` as the total order rises so that each entry is written from
/// entries already in hand.
pub fn r_tensor(lmax: usize, alpha: f64, d: [D2; 3], out: &mut RTensor) {
    debug_assert!(lmax <= RMAX);
    let t2 = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]) * alpha;
    let f = boys_d2_upto(lmax, t2);

    let work = &mut *out.work;
    let mut scale = 1.0f64;
    for (n, fw) in work.iter_mut().enumerate().take(lmax + 1) {
        fw[0][0][0] = f[n] * scale;
        scale *= -2.0 * alpha;
    }

    for order in 1..=lmax {
        for t in 0..=order {
            for u in 0..=(order - t) {
                let v = order - t - u;
                for n in 0..=(lmax - order) {
                    // Decrement whichever index is nonzero; the three routes give the
                    // same tensor, so the choice is only about which entries exist yet.
                    let val = if t > 0 {
                        let mut x = d[0] * work[n + 1][t - 1][u][v];
                        if t > 1 {
                            x = x + work[n + 1][t - 2][u][v] * ((t - 1) as f64);
                        }
                        x
                    } else if u > 0 {
                        let mut x = d[1] * work[n + 1][t][u - 1][v];
                        if u > 1 {
                            x = x + work[n + 1][t][u - 2][v] * ((u - 1) as f64);
                        }
                        x
                    } else {
                        let mut x = d[2] * work[n + 1][t][u][v - 1];
                        if v > 1 {
                            x = x + work[n + 1][t][u][v - 2] * ((v - 1) as f64);
                        }
                        x
                    };
                    work[n][t][u][v] = val;
                }
            }
        }
    }
    for t in 0..=lmax {
        for u in 0..=(lmax - t) {
            for v in 0..=(lmax - t - u) {
                out.r[t][u][v] = work[0][t][u][v];
            }
        }
    }
}

impl RTensor {
    pub fn zero() -> Self {
        RTensor {
            r: [[[D2::c(0.0); RMAX + 1]; RMAX + 1]; RMAX + 1],
            work: Box::new([[[[D2::c(0.0); RMAX + 1]; RMAX + 1]; RMAX + 1]; RMAX + 1]),
        }
    }
}

/// Normalisation of a primitive Cartesian Gaussian with total angular momentum `l`.
///
/// `N = (2a/pi)^{3/4} (4a)^{l/2}` — the general factor divided by
/// `sqrt((2i-1)!!(2j-1)!!(2k-1)!!)`, which is 1 for every `l <= 1` because at most one
/// Cartesian power is nonzero and `(2*1-1)!! = 1`. At `l = 0` this is exactly
/// [`crate::sto3g::prim_norm`], and `tests/md.rs` pins that.
pub fn prim_norm_l(a: f64, l: u8) -> f64 {
    let base = (2.0 * a / PI).powf(0.75);
    match l {
        0 => base,
        1 => base * (4.0 * a).sqrt(),
        2 => base * (4.0 * a),
        _ => panic!("md.rs: l = {l} is not supported"),
    }
}

/// The Cartesian components of a shell, in basis order.
///
/// `p` is ordered `px, py, pz`.
/// `d` has 6 Cartesian components: `xx, yy, zz, xy, xz, yz`.
pub fn cartesian_components(l: u8) -> &'static [[u8; 3]] {
    match l {
        0 => &[[0, 0, 0]],
        1 => &[[1, 0, 0], [0, 1, 0], [0, 0, 1]],
        2 => &[
            [2, 0, 0],
            [0, 2, 0],
            [0, 0, 2],
            [1, 1, 0],
            [1, 0, 1],
            [0, 1, 1],
        ],
        _ => panic!("md.rs: l = {l} is not supported"),
    }
}

/// The 5x6 matrix taking a d shell's six normalised CARTESIAN components to its five
/// normalised SPHERICAL ones, in the crate's component order `xx, yy, zz, xy, xz, yz`.
///
/// # Why a minimal basis must do this
///
/// The six Cartesian d functions do not span an `l = 2` space. They span the five real
/// solid harmonics PLUS one more, `(x^2 + y^2 + z^2) exp(-a r^2)`, which is spherically
/// symmetric and therefore `l = 0`. It is not a 1s Gaussian — its radial factor is `r^2`
/// — but its angular momentum is zero, so keeping all six gives every d shell a spurious
/// sixth function of the wrong symmetry.
///
/// That is not a small distortion in a MINIMAL basis, where the whole premise is one
/// function per occupied orbital. It is the difference between krypton being a closed
/// shell of eighteen functions holding thirty-six electrons — one determinant, exactly —
/// and krypton being nineteen functions with two holes in them, which is 361 determinants
/// and no longer a closed shell at all. ELEMENTS-3's E1 asserts the former; the arithmetic
/// throughout its prereg (Br2 at 36^2, HBr at 19^2, HI at 28^2, Xe2 at 54 orbitals) is the
/// five-component count. So the five-component basis is what the contract means.
///
/// # Where these numbers come from
///
/// They are derived, not tabulated. Writing the crate's six normalised Cartesian functions
/// as `g1..g6`, their overlaps are not zero — `<g1|g2> = 1/3` among the three squares, and
/// the three products are orthonormal — so the combinations have to be normalised against
/// that metric rather than by inspection:
///
/// ```text
/// d(z^2)     ∝ 2z^2 - x^2 - y^2   has norm^2 4 in the g metric, so  (2 g3 - g1 - g2) / 2
/// d(x^2-y^2) ∝ x^2 - y^2          has norm^2 4/3,              so  sqrt(3)/2 (g1 - g2)
/// d(xy), d(xz), d(yz)                                          are already orthonormal
/// ```
///
/// and the discarded direction is exactly `g1 + g2 + g3`, the s-type one.
pub const SPHERICAL_D: [[f64; 6]; 5] = [
    // z^2
    [-0.5, -0.5, 1.0, 0.0, 0.0, 0.0],
    // xz
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
    // yz
    [0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
    // x^2 - y^2, with sqrt(3)/2
    [0.8660254037844386, -0.8660254037844386, 0.0, 0.0, 0.0, 0.0],
    // xy
    [0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
];

/// Spherical functions per shell: 1 for s, 3 for p, 5 for d.
pub fn spherical_components(l: u8) -> usize {
    match l {
        0 => 1,
        1 => 3,
        2 => 5,
        _ => panic!("md.rs: l = {l} is not supported"),
    }
}

/// Normalization scale factor for off-diagonal Cartesian components (e.g. sqrt(3) for xy, xz, yz).
#[inline]
pub fn cart_factor(p: [u8; 3]) -> f64 {
    match p {
        [1, 1, 0] | [1, 0, 1] | [0, 1, 1] => 1.7320508075688772,
        _ => 1.0,
    }
}

/// One Hermite term of a bra or ket pair: the index triple and its coefficient.
#[derive(Clone, Copy)]
struct HTerm {
    t: usize,
    u: usize,
    v: usize,
    c: D2,
}

/// Expand one Cartesian function pair into its Hermite terms.
///
/// The `E` factorises over the three directions, so the term list is the product of
/// three short lists — at `l <= 2` never more than 125 entries, which is why the double
/// loop over bra and ket terms below is cheap enough to sit inside the primitive quartet
/// loop.
fn hermite_terms(
    ex: &ETable,
    ey: &ETable,
    ez: &ETable,
    la: [u8; 3],
    lb: [u8; 3],
    out: &mut [HTerm],
) -> usize {
    let (ix, jx) = (la[0] as usize, lb[0] as usize);
    let (iy, jy) = (la[1] as usize, lb[1] as usize);
    let (iz, jz) = (la[2] as usize, lb[2] as usize);
    let mut n = 0;
    for t in 0..=(ix + jx) {
        let cx = ex.e[ix][jx][t];
        for u in 0..=(iy + jy) {
            let cxy = cx * ey.e[iy][jy][u];
            for v in 0..=(iz + jz) {
                out[n] = HTerm {
                    t,
                    u,
                    v,
                    c: cxy * ez.e[iz][jz][v],
                };
                n += 1;
            }
        }
    }
    n
}

// ------------------------------------------------------------------ the basis

/// A contracted shell placed on a centre.
#[derive(Clone, Debug)]
pub struct ShellOn {
    pub center: usize,
    pub l: u8,
    pub alpha: [f64; 3],
    /// Contraction coefficients with the PRIMITIVE normalisation already folded in, and
    /// the whole shell rescaled so `<chi|chi> = 1`.
    pub coeff: [f64; 3],
    /// `<chi|chi>` BEFORE that rescaling — a diagnostic on the declared data, and the
    /// cheapest single number that catches a mistyped exponent or coefficient.
    pub raw_norm: f64,
    /// Index of this shell's first basis function.
    pub first: usize,
}

impl ShellOn {
    pub fn n_functions(&self) -> usize {
        cartesian_components(self.l).len()
    }
}

/// Everything the integral routines need about a geometry: where the nuclei are, what
/// charges they carry, and which contracted shells sit on them.
pub struct Basis {
    pub centers: Vec<[D2; 3]>,
    pub charges: Vec<f64>,
    pub shells: Vec<ShellOn>,
    /// Total contracted basis functions the rest of the engine sees: SPHERICAL, five
    /// components per d shell. This is the dimension of every integral array and of the
    /// FCI space built on them.
    pub n: usize,
    /// Total CARTESIAN functions, six per d shell. An internal quantity: the integral
    /// recursions are Cartesian, so assembly happens at this size and is then projected
    /// down to `n`. Equal to `n` whenever the basis has no d shell.
    pub n_cart: usize,
    /// The `n x n_cart` projection from Cartesian to spherical, row-major, or `None` when
    /// the two coincide and the projection would be the identity. Block diagonal by shell:
    /// identity for s and p, [`SPHERICAL_D`] for d.
    pub sph: Option<Vec<f64>>,
    /// `(shell index, Cartesian powers)` for each CARTESIAN basis function, in order.
    pub funcs: Vec<(usize, [u8; 3])>,
}

impl Basis {
    /// Assemble from a list of `(centre index, l, exponents, tabulated coefficients)`.
    ///
    /// The tabulated coefficients are multiplied by the primitive normalisations and
    /// then the whole shell is rescaled to unit self-overlap. Both steps are standard
    /// and both are what the referee does; neither is an improvement on the declared
    /// basis, they are part of what "the STO-3G basis" means.
    pub fn assemble(
        centers: Vec<[D2; 3]>,
        charges: Vec<f64>,
        decls: &[(usize, u8, [f64; 3], [f64; 3])],
    ) -> Basis {
        let mut shells = Vec::with_capacity(decls.len());
        let mut funcs = Vec::new();
        let mut first = 0usize;
        for &(center, l, alpha, coeff) in decls {
            let mut c = [0.0f64; 3];
            for k in 0..3 {
                c[k] = coeff[k] * prim_norm_l(alpha[k], l);
            }
            // Self-overlap of the contraction at zero separation. For a Cartesian power
            // `l` in one direction the same-centre primitive overlap is
            // `(pi/p)^{3/2} (2p)^{-l}`, which at l = 0 is the s-shell expression
            // `sto3g.rs` uses.
            let mut raw = 0.0f64;
            for i in 0..3 {
                for j in 0..3 {
                    let p = alpha[i] + alpha[j];
                    let mut s = (PI / p).powf(1.5);
                    if l == 1 {
                        s /= 2.0 * p;
                    } else if l == 2 {
                        s *= 3.0 / (4.0 * p * p);
                    }
                    raw += c[i] * c[j] * s;
                }
            }
            let scale = 1.0 / raw.sqrt();
            for ci in c.iter_mut() {
                *ci *= scale;
            }
            for &powers in cartesian_components(l) {
                funcs.push((shells.len(), powers));
            }
            let nf = cartesian_components(l).len();
            shells.push(ShellOn {
                center,
                l,
                alpha,
                coeff: c,
                raw_norm: raw,
                first,
            });
            first += nf;
        }
        let n_cart = funcs.len();
        let n: usize = shells.iter().map(|sh| spherical_components(sh.l)).sum();
        // Only build the projection when it would do something. An identity matrix that
        // every integral is multiplied by is a cost and a place for a bug, and the first
        // eighteen elements have no d shell at all -- which is also what keeps every
        // ELEMENTS-1 and MIXTURES-1 number bit-identical under this change.
        let sph = if n == n_cart {
            None
        } else {
            let mut p = vec![0.0f64; n * n_cart];
            let (mut row, mut col) = (0usize, 0usize);
            for sh in shells.iter() {
                match sh.l {
                    2 => {
                        for (r, coeffs) in SPHERICAL_D.iter().enumerate() {
                            for (c, &x) in coeffs.iter().enumerate() {
                                p[(row + r) * n_cart + col + c] = x;
                            }
                        }
                        row += 5;
                        col += 6;
                    }
                    _ => {
                        let k = cartesian_components(sh.l).len();
                        for i in 0..k {
                            p[(row + i) * n_cart + col + i] = 1.0;
                        }
                        row += k;
                        col += k;
                    }
                }
            }
            debug_assert_eq!((row, col), (n, n_cart));
            Some(p)
        };
        Basis {
            centers,
            charges,
            shells,
            n,
            n_cart,
            sph,
            funcs,
        }
    }

    /// Nuclear repulsion `sum_{A<B} Z_A Z_B / R_AB`, carrying its derivatives.
    pub fn nuclear_repulsion(&self) -> D2 {
        let mut acc = D2::c(0.0);
        for a in 0..self.centers.len() {
            for b in (a + 1)..self.centers.len() {
                let d = [
                    self.centers[a][0] - self.centers[b][0],
                    self.centers[a][1] - self.centers[b][1],
                    self.centers[a][2] - self.centers[b][2],
                ];
                let r2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
                acc = acc + (self.charges[a] * self.charges[b]) / r2.sqrt();
            }
        }
        acc
    }
}

/// The one- and two-electron integrals of a geometry, in the AO basis.
pub struct AoIntegrals {
    pub n: usize,
    /// Overlap, row-major `n x n`.
    pub s: Vec<D2>,
    /// Kinetic energy.
    pub t: Vec<D2>,
    /// Nuclear attraction, summed over all centres.
    pub v: Vec<D2>,
    /// `(ij|kl)` in chemist notation, row-major `n^4`.
    pub eri: Vec<D2>,
}

impl AoIntegrals {
    #[inline]
    pub fn h(&self, i: usize, j: usize) -> D2 {
        self.t[i * self.n + j] + self.v[i * self.n + j]
    }
    #[inline]
    pub fn g(&self, i: usize, j: usize, k: usize, l: usize) -> D2 {
        let n = self.n;
        self.eri[((i * n + j) * n + k) * n + l]
    }
}

/// Compute every AO integral of a basis.
pub fn ao_integrals(b: &Basis) -> AoIntegrals {
    let n = b.n_cart;
    let mut s = vec![D2::c(0.0); n * n];
    let mut t = vec![D2::c(0.0); n * n];
    let mut v = vec![D2::c(0.0); n * n];

    let mut rt = RTensor::zero();

    for (si, sa) in b.shells.iter().enumerate() {
        for sb in b.shells.iter().take(si + 1) {
            let ca = b.centers[sa.center];
            let cb = b.centers[sb.center];
            for pa in 0..3 {
                for pb in 0..3 {
                    let (a, bb) = (sa.alpha[pa], sb.alpha[pb]);
                    let p = a + bb;
                    let ex = e_table(a, bb, ca[0], cb[0]);
                    let ey = e_table(a, bb, ca[1], cb[1]);
                    let ez = e_table(a, bb, ca[2], cb[2]);
                    let pref_s = (PI / p).powf(1.5);
                    let pc = [
                        (ca[0] * a + cb[0] * bb) * (1.0 / p),
                        (ca[1] * a + cb[1] * bb) * (1.0 / p),
                        (ca[2] * a + cb[2] * bb) * (1.0 / p),
                    ];

                    for (fa, &la) in cartesian_components(sa.l).iter().enumerate() {
                        for (fb, &lb) in cartesian_components(sb.l).iter().enumerate() {
                            let i = sa.first + fa;
                            let j = sb.first + fb;
                            if j > i {
                                continue;
                            }
                            let w = sa.coeff[pa] * sb.coeff[pb] * cart_factor(la) * cart_factor(lb);
                            // --- overlap: the product of three 1-D Hermite zeroth terms
                            let sx = ex.e[la[0] as usize][lb[0] as usize][0];
                            let sy = ey.e[la[1] as usize][lb[1] as usize][0];
                            let sz = ez.e[la[2] as usize][lb[2] as usize][0];
                            s[i * n + j] = s[i * n + j] + sx * sy * sz * (w * pref_s);

                            // --- kinetic: -1/2 <a|nabla^2|b>, one direction at a time.
                            //
                            // d2/dx2 [ (x-B)^j e^{-b(x-B)^2} ] =
                            //   j(j-1)(x-B)^{j-2} - 2b(2j+1)(x-B)^j + 4b^2 (x-B)^{j+2},
                            // all three terms being overlaps at shifted powers, which is
                            // why the `E` table is built two powers past `LMAX`.
                            let mut tk = D2::c(0.0);
                            for dir in 0..3 {
                                let (etab, o1, o2) = match dir {
                                    0 => (&ex, sy, sz),
                                    1 => (&ey, sx, sz),
                                    _ => (&ez, sx, sy),
                                };
                                let ii = la[dir] as usize;
                                let jj = lb[dir] as usize;
                                let mut d2 = etab.e[ii][jj + 2][0] * (4.0 * bb * bb)
                                    - etab.e[ii][jj][0] * (2.0 * bb * ((2 * jj + 1) as f64));
                                if jj >= 2 {
                                    d2 = d2 + etab.e[ii][jj - 2][0] * ((jj * (jj - 1)) as f64);
                                }
                                tk = tk + d2 * o1 * o2;
                            }
                            t[i * n + j] = t[i * n + j] + tk * (-0.5 * w * pref_s);

                            // --- nuclear attraction, summed over centres
                            let mut terms = [HTerm {
                                t: 0,
                                u: 0,
                                v: 0,
                                c: D2::c(0.0),
                            }; 64];
                            let nt = hermite_terms(&ex, &ey, &ez, la, lb, &mut terms);
                            let order = (sa.l + sb.l) as usize;
                            let mut vacc = D2::c(0.0);
                            for (ic, &cc) in b.centers.iter().enumerate() {
                                let d = [pc[0] - cc[0], pc[1] - cc[1], pc[2] - cc[2]];
                                r_tensor(order, p, d, &mut rt);
                                let mut acc = D2::c(0.0);
                                for term in terms.iter().take(nt) {
                                    acc = acc + term.c * rt.r[term.t][term.u][term.v];
                                }
                                vacc = vacc - acc * b.charges[ic];
                            }
                            v[i * n + j] = v[i * n + j] + vacc * (w * 2.0 * PI / p);
                        }
                    }
                }
            }
        }
    }
    // Symmetrise: the loops above filled the lower triangle only, and every one of these
    // operators is Hermitian over real functions.
    for i in 0..n {
        for j in 0..i {
            s[j * n + i] = s[i * n + j];
            t[j * n + i] = t[i * n + j];
            v[j * n + i] = v[i * n + j];
        }
    }

    let eri = eri_all(b);
    let cart = AoIntegrals { n, s, t, v, eri };
    match &b.sph {
        None => cart,
        Some(p) => project_to_spherical(&cart, p, b.n),
    }
}

/// Project a Cartesian integral set through `p` (`n_pure x n_cart`, row-major).
///
/// The two-index arrays become `P X P^T`. The four-index array is transformed ONE INDEX AT
/// A TIME -- four passes of `O(n_cart^3 n_pure)` rather than one contraction of
/// `O(n_cart^4 n_pure^4)`, which at 58 Cartesian functions is the difference between
/// seconds and never.
fn project_to_spherical(cart: &AoIntegrals, p: &[f64], n: usize) -> AoIntegrals {
    let m = cart.n;
    let two = |x: &[D2]| -> Vec<D2> {
        // (P X) then (P X) P^T, so the intermediate is n x m rather than m x m.
        let mut px = vec![D2::c(0.0); n * m];
        for a in 0..n {
            for i in 0..m {
                if p[a * m + i] == 0.0 {
                    continue;
                }
                let w = p[a * m + i];
                for j in 0..m {
                    px[a * m + j] = px[a * m + j] + x[i * m + j] * w;
                }
            }
        }
        let mut out = vec![D2::c(0.0); n * n];
        for a in 0..n {
            for bq in 0..n {
                let mut acc = D2::c(0.0);
                for j in 0..m {
                    let w = p[bq * m + j];
                    if w != 0.0 {
                        acc = acc + px[a * m + j] * w;
                    }
                }
                out[a * n + bq] = acc;
            }
        }
        out
    };

    // Four sweeps over the ERI array, each replacing one Cartesian index by a spherical
    // one. `cur` shrinks as the sweeps proceed, so the later passes are the cheap ones.
    let mut cur = cart.eri.clone();
    let mut dims = [m, m, m, m];
    for axis in 0..4 {
        let [_d0, d1, d2, d3] = dims;
        let old = dims[axis];
        let mut next_dims = dims;
        next_dims[axis] = n;
        let [e0, e1, e2, e3] = next_dims;
        let mut next = vec![D2::c(0.0); e0 * e1 * e2 * e3];
        for i0 in 0..e0 {
            for i1 in 0..e1 {
                for i2 in 0..e2 {
                    for i3 in 0..e3 {
                        let idx = [i0, i1, i2, i3];
                        let mut acc = D2::c(0.0);
                        for k in 0..old {
                            let w = p[idx[axis] * m + k];
                            if w == 0.0 {
                                continue;
                            }
                            let mut src = idx;
                            src[axis] = k;
                            acc = acc
                                + cur[((src[0] * d1 + src[1]) * d2 + src[2]) * d3 + src[3]] * w;
                        }
                        next[((i0 * e1 + i1) * e2 + i2) * e3 + i3] = acc;
                    }
                }
            }
        }
        cur = next;
        dims = next_dims;
    }

    // Restore the eight-fold symmetry EXACTLY, the same way `eri_all` establishes it.
    //
    // The four sweeps above are mathematically symmetric but not numerically: element
    // (i,j,k,l) and element (j,i,k,l) accumulate the same terms in different orders, so
    // they can differ in the last bits. `eri_all` deliberately computes each
    // symmetry-unique quartet once and writes one value to all eight permutations, and
    // the comment there says why -- the FCI relies on `(ij|kl) == (kl|ij)` being exact in
    // the output rather than approximate. A projection that quietly downgraded that to
    // "equal to within rounding" would take the property away and leave the comment.
    let idx = |i: usize, j: usize, k: usize, l: usize| ((i * n + j) * n + k) * n + l;
    for i in 0..n {
        for j in 0..=i {
            for k in 0..n {
                for l in 0..=k {
                    if (i * n + j) < (k * n + l) {
                        continue;
                    }
                    let x = cur[idx(i, j, k, l)];
                    for &(a, b, c, d) in &[
                        (i, j, k, l),
                        (j, i, k, l),
                        (i, j, l, k),
                        (j, i, l, k),
                        (k, l, i, j),
                        (l, k, i, j),
                        (k, l, j, i),
                        (l, k, j, i),
                    ] {
                        cur[idx(a, b, c, d)] = x;
                    }
                }
            }
        }
    }

    // The one-electron operators are Hermitian over real functions, and `P X P^T` is
    // symmetric in exact arithmetic but not in floating point for the same reason. Take
    // the lower triangle as authoritative, again matching `ao_integrals`.
    let symmetrise = |mut x: Vec<D2>| -> Vec<D2> {
        for i in 0..n {
            for j in 0..i {
                x[j * n + i] = x[i * n + j];
            }
        }
        x
    };

    AoIntegrals {
        n,
        s: symmetrise(two(&cart.s)),
        t: symmetrise(two(&cart.t)),
        v: symmetrise(two(&cart.v)),
        eri: cur,
    }
}

/// Every two-electron integral `(ij|kl)`, computed once per symmetry-unique quartet and
/// written to all eight permutations.
///
/// Computing the eight independently would be eight slightly different roundings of one
/// number; writing one value everywhere is what makes `(ij|kl) == (kl|ij)` EXACT in the
/// output rather than approximate, which the FCI code below relies on.
pub fn eri_all(b: &Basis) -> Vec<D2> {
    let n = b.n_cart;
    let mut eri = vec![D2::c(0.0); n * n * n * n];
    let mut rt = RTensor::zero();

    let ns = b.shells.len();
    for si in 0..ns {
        for sj in 0..=si {
            for sk in 0..=si {
                let sl_max = if sk == si { sj } else { sk };
                for sl in 0..=sl_max {
                    accumulate_shell_quartet(b, si, sj, sk, sl, &mut rt, &mut eri);
                }
            }
        }
    }
    eri
}

#[allow(clippy::too_many_arguments)]
fn accumulate_shell_quartet(
    b: &Basis,
    si: usize,
    sj: usize,
    sk: usize,
    sl: usize,
    rt: &mut RTensor,
    eri: &mut [D2],
) {
    let n = b.n_cart;
    let (sa, sb, sc, sd) = (&b.shells[si], &b.shells[sj], &b.shells[sk], &b.shells[sl]);
    let (ca, cb) = (b.centers[sa.center], b.centers[sb.center]);
    let (cc, cd) = (b.centers[sc.center], b.centers[sd.center]);
    let bra_order = (sa.l + sb.l) as usize;
    let ket_order = (sc.l + sd.l) as usize;
    let order = bra_order + ket_order;

    let comp_a = cartesian_components(sa.l);
    let comp_b = cartesian_components(sb.l);
    let comp_c = cartesian_components(sc.l);
    let comp_d = cartesian_components(sd.l);

    // One accumulator per basis-function combination in this quartet, so the eight-fold
    // permutation write happens once with a fully contracted value.
    let mut acc = [[[[D2::c(0.0); 6]; 6]; 6]; 6];

    let zero = HTerm {
        t: 0,
        u: 0,
        v: 0,
        c: D2::c(0.0),
    };
    let mut bra_terms = [zero; 64];
    let mut ket_terms = [zero; 64];

    for pa in 0..3 {
        for pb in 0..3 {
            let (a, bb) = (sa.alpha[pa], sb.alpha[pb]);
            let p = a + bb;
            let wab = sa.coeff[pa] * sb.coeff[pb];
            let ex = e_table(a, bb, ca[0], cb[0]);
            let ey = e_table(a, bb, ca[1], cb[1]);
            let ez = e_table(a, bb, ca[2], cb[2]);
            let pp = [
                (ca[0] * a + cb[0] * bb) * (1.0 / p),
                (ca[1] * a + cb[1] * bb) * (1.0 / p),
                (ca[2] * a + cb[2] * bb) * (1.0 / p),
            ];
            for pc in 0..3 {
                for pd in 0..3 {
                    let (c, d) = (sc.alpha[pc], sd.alpha[pd]);
                    let q = c + d;
                    let w = wab * sc.coeff[pc] * sd.coeff[pd];
                    let fx = e_table(c, d, cc[0], cd[0]);
                    let fy = e_table(c, d, cc[1], cd[1]);
                    let fz = e_table(c, d, cc[2], cd[2]);
                    let qq = [
                        (cc[0] * c + cd[0] * d) * (1.0 / q),
                        (cc[1] * c + cd[1] * d) * (1.0 / q),
                        (cc[2] * c + cd[2] * d) * (1.0 / q),
                    ];
                    let alpha = p * q / (p + q);
                    r_tensor(
                        order,
                        alpha,
                        [pp[0] - qq[0], pp[1] - qq[1], pp[2] - qq[2]],
                        rt,
                    );
                    let pref = w * 2.0 * PI_POW_2_5 / (p * q * (p + q).sqrt());

                    for (ia, &la) in comp_a.iter().enumerate() {
                        for (ib, &lb) in comp_b.iter().enumerate() {
                            let nb = hermite_terms(&ex, &ey, &ez, la, lb, &mut bra_terms);
                            for (ic, &lc) in comp_c.iter().enumerate() {
                                for (id, &ld) in comp_d.iter().enumerate() {
                                    let nk =
                                        hermite_terms(&fx, &fy, &fz, lc, ld, &mut ket_terms);
                                    let mut sum = D2::c(0.0);
                                    for bt in bra_terms.iter().take(nb) {
                                        for kt in ket_terms.iter().take(nk) {
                                            // (-1)^{tau+nu+phi}: the ket's Hermite
                                            // expansion is about ITS own centre, and the
                                            // Coulomb kernel differentiates the second
                                            // electron's coordinate the other way round.
                                            let parity = (kt.t + kt.u + kt.v) & 1;
                                            let r = rt.r[bt.t + kt.t][bt.u + kt.u][bt.v + kt.v];
                                            let term = bt.c * kt.c * r;
                                            sum = if parity == 1 { sum - term } else { sum + term };
                                        }
                                    }
                                    acc[ia][ib][ic][id] = acc[ia][ib][ic][id] + sum * pref;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for (ia, &la) in comp_a.iter().enumerate() {
        for (ib, &lb) in comp_b.iter().enumerate() {
            for (ic, &lc) in comp_c.iter().enumerate() {
                for (id, &ld) in comp_d.iter().enumerate() {
                    let fabcd = cart_factor(la) * cart_factor(lb) * cart_factor(lc) * cart_factor(ld);
                    let val = acc[ia][ib][ic][id] * fabcd;
                    let (i, j, k, l) = (
                        sa.first + ia,
                        sb.first + ib,
                        sc.first + ic,
                        sd.first + id,
                    );
                    for &(p, q, r, s) in &[
                        (i, j, k, l),
                        (j, i, k, l),
                        (i, j, l, k),
                        (j, i, l, k),
                        (k, l, i, j),
                        (l, k, i, j),
                        (k, l, j, i),
                        (l, k, j, i),
                    ] {
                        eri[((p * n + q) * n + r) * n + s] = val;
                    }
                }
            }
        }
    }
}

/// `<chi_mu| r^2 |chi_nu>` for a basis whose shells all sit on ONE centre at the origin.
///
/// # Why this exists and why it is restricted
///
/// P1's third radius rule needs a size derived from the atom's own electron density rather
/// than from a homonuclear curve, because most mid-row homonuclear dimers cannot be
/// computed at all. The quantity is `sqrt(<r^2> / N)`, the root-mean-square distance of an
/// electron from the nucleus, and `<r^2>` is a ONE-ELECTRON operator — so it needs one
/// matrix and no new solver.
///
/// The restriction to a single centre at the origin is what makes it four lines of closed
/// form instead of a second moment-integral recursion. With every function on the same
/// centre, `x` and `x_A` and `x_B` are the same variable, so multiplying by `x^2` is
/// exactly raising the Cartesian power by two — and the same-centre integral of a Gaussian
/// monomial is elementary:
///
/// ```text
/// int x^L exp(-p x^2) dx = (L-1)!! / (2p)^{L/2} * sqrt(pi/p)   for even L, 0 for odd
/// ```
///
/// An atom is at the origin, which is the only case P1 asks for. The assertion below
/// refuses anything else rather than returning a number computed for a geometry it does
/// not describe.
pub fn atomic_r2(b: &Basis) -> Vec<f64> {
    for c in b.centers.iter() {
        assert!(
            c[0].v == 0.0 && c[1].v == 0.0 && c[2].v == 0.0,
            "atomic_r2 is the SINGLE-CENTRE form and every shell must sit at the origin; \
             a centre at ({}, {}, {}) would need the general moment recursion",
            c[0].v,
            c[1].v,
            c[2].v
        );
    }
    // int x^L exp(-p x^2) dx, zero for odd L.
    fn moment(l: usize, p: f64) -> f64 {
        if l % 2 == 1 {
            return 0.0;
        }
        let mut df = 1.0f64; // (l-1)!!
        let mut k = l as i64 - 1;
        while k > 1 {
            df *= k as f64;
            k -= 2;
        }
        df / (2.0 * p).powi((l / 2) as i32) * (PI / p).sqrt()
    }

    let n = b.n_cart;
    let mut out = vec![0.0f64; n * n];
    for (si, sa) in b.shells.iter().enumerate() {
        for (sj, sb) in b.shells.iter().enumerate() {
            let _ = (si, sj);
            for (fa, &la) in cartesian_components(sa.l).iter().enumerate() {
                for (fb, &lb) in cartesian_components(sb.l).iter().enumerate() {
                    let (i, j) = (sa.first + fa, sb.first + fb);
                    let w = cart_factor(la) * cart_factor(lb);
                    let mut acc = 0.0f64;
                    for pa in 0..3 {
                        for pb in 0..3 {
                            let p = sa.alpha[pa] + sb.alpha[pb];
                            let c = sa.coeff[pa] * sb.coeff[pb];
                            // r^2 = x^2 + y^2 + z^2: three terms, each raising one
                            // Cartesian power by two and leaving the other two alone.
                            for dir in 0..3 {
                                let mut t = c;
                                for k in 0..3 {
                                    let bump = if k == dir { 2 } else { 0 };
                                    t *= moment(la[k] as usize + lb[k] as usize + bump, p);
                                }
                                acc += t;
                            }
                        }
                    }
                    out[i * n + j] += acc * w;
                }
            }
        }
    }

    // Project to the declared basis, exactly as `ao_integrals` does, so the matrix is
    // indexed the same way as everything else the CI sees.
    match &b.sph {
        None => out,
        Some(p) => {
            let m = n;
            let k = b.n;
            let mut px = vec![0.0f64; k * m];
            for a in 0..k {
                for i in 0..m {
                    let wv = p[a * m + i];
                    if wv != 0.0 {
                        for jj in 0..m {
                            px[a * m + jj] += wv * out[i * m + jj];
                        }
                    }
                }
            }
            let mut res = vec![0.0f64; k * k];
            for a in 0..k {
                for bq in 0..k {
                    let mut acc = 0.0;
                    for jj in 0..m {
                        let wv = p[bq * m + jj];
                        if wv != 0.0 {
                            acc += px[a * m + jj] * wv;
                        }
                    }
                    res[a * k + bq] = acc;
                }
            }
            res
        }
    }
}
