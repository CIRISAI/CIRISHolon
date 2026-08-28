//! The STO-3G hydrogen basis, and the closed-form integrals over s-type Gaussians.
//!
//! # What is derivation and what is model definition
//!
//! Everything in this file except six decimal numbers is closed-form mathematics,
//! implemented from the formulae named in each function's doc comment. The six numbers
//! — three exponents and three contraction coefficients — are the MODEL, not a result:
//! a basis set is a choice, and this one's choice is stated here and nowhere else. No
//! energy, no bond length and no well depth is quoted anywhere in this crate; they are
//! all computed from these six numbers and the formulae below.
//!
//! # The geometry convention
//!
//! All basis functions are s-type and the molecule is linear, so only positions along
//! the internuclear axis matter and centres are single coordinates. They arrive as [`D2`]
//! because in the H2 problem a centre IS a function of the separation `R` — the
//! derivative of every integral with respect to `R` therefore falls out of the same
//! expression that computes it, with no separate differentiated copy to keep in step.
//!
//! # Provenance of the formulae
//!
//! Gaussian product theorem, and the standard s-type primitive integrals over it:
//!
//! ```text
//! exp(-a|r-A|^2) exp(-b|r-B|^2) = K_ab exp(-p|r-P|^2)
//! p = a + b,   P = (aA + bB)/p,   K_ab = exp(-(ab/p)|A-B|^2)
//! ```
//!
//! These are textbook (Boys 1950; Szabo & Ostlund, *Modern Quantum Chemistry*, App. A).
//! They are implemented, not cited: the point of this crate is that the browser computes
//! the curve rather than reading someone's table of it.

use crate::dual::D2;
use crate::special::boys0_d2;

const PI: f64 = core::f64::consts::PI;

/// `pi^{5/2}`, the two-electron prefactor's transcendental part.
///
/// Written as a literal because it is a CONSTANT that `powf` was being asked to
/// recompute 486 times per knot — the single biggest cost in the curve before it was
/// hoisted. `tests/model.rs` pins it to `PI.powf(2.5)` so the literal cannot drift from
/// the expression it stands for.
pub const PI_POW_2_5: f64 = 17.493418327624862;

/// MODEL DEFINITION: the STO-3G hydrogen 1s contraction exponents.
pub const H_EXPONENTS: [f64; 3] = [3.42525091, 0.62391373, 0.16885540];

/// MODEL DEFINITION: the matching contraction coefficients, as tabulated.
pub const H_COEFFS: [f64; 3] = [0.15432897, 0.53532814, 0.44463454];

pub const MODEL_NAME: &str = "H2/STO-3G/FCI";

/// One primitive Gaussian: its exponent and its normalisation.
///
/// The normalisation is carried rather than recomputed. `(2a/pi)^{3/4}` is a `powf`, the
/// exponents never change, and the two-electron loop asks for four of them per call —
/// so computing it once per basis is the difference between a curve that generates in
/// milliseconds and one that does not. The VALUE is unchanged: `norm` is exactly the
/// f64 `prim_norm(alpha)` returns, so every integral below is bit-for-bit what it was
/// when it called `prim_norm` itself.
#[derive(Clone, Copy, Debug)]
pub struct Prim {
    pub alpha: f64,
    pub norm: f64,
}

impl Prim {
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha,
            norm: prim_norm(alpha),
        }
    }
}

/// A contracted s-type basis function: three primitives sharing one centre.
///
/// The coefficients stored are the tabulated ones RESCALED so that `<chi|chi> = 1` at
/// working precision. The tabulated values are rounded to eight decimals and do not
/// normalise exactly; renormalising is standard practice and is what the referee does,
/// so it is part of matching the model rather than an improvement on it.
#[derive(Clone, Copy, Debug)]
pub struct Contraction {
    pub prim: [Prim; 3],
    pub coeff: [f64; 3],
    /// `<chi|chi>` BEFORE renormalisation — a diagnostic on the tabulated data, and the
    /// cheapest single number that catches a mistyped coefficient.
    pub raw_norm: f64,
}

/// Normalisation of a primitive s-type Gaussian `g_a(r) = N_a exp(-a|r-A|^2)`:
/// `N_a = (2a/pi)^{3/4}`.
pub fn prim_norm(a: f64) -> f64 {
    (2.0 * a / PI).powf(0.75)
}

/// The hydrogen 1s contraction, renormalised.
pub fn sto3g_hydrogen() -> Contraction {
    let mut raw = 0.0f64;
    for i in 0..3 {
        for j in 0..3 {
            let p = H_EXPONENTS[i] + H_EXPONENTS[j];
            // Same-centre overlap: the Gaussian factor is exp(0) = 1, so this is
            // independent of geometry and the rescaling below is a constant of the
            // basis rather than a function of R.
            raw += H_COEFFS[i]
                * H_COEFFS[j]
                * prim_norm(H_EXPONENTS[i])
                * prim_norm(H_EXPONENTS[j])
                * (PI / p).powf(1.5);
        }
    }
    let scale = 1.0 / raw.sqrt();
    Contraction {
        prim: [
            Prim::new(H_EXPONENTS[0]),
            Prim::new(H_EXPONENTS[1]),
            Prim::new(H_EXPONENTS[2]),
        ],
        coeff: [
            H_COEFFS[0] * scale,
            H_COEFFS[1] * scale,
            H_COEFFS[2] * scale,
        ],
        raw_norm: raw,
    }
}

/// `S = N_a N_b (pi/p)^{3/2} K_ab`
pub fn prim_overlap(a: Prim, ca: D2, b: Prim, cb: D2) -> D2 {
    let p = a.alpha + b.alpha;
    let mu = a.alpha * b.alpha / p;
    let d = ca - cb;
    (-(mu * (d * d))).exp() * (a.norm * b.norm * (PI / p).powf(1.5))
}

/// `T = N_a N_b mu (3 - 2 mu |A-B|^2) (pi/p)^{3/2} K_ab`, with `mu = ab/p`.
pub fn prim_kinetic(a: Prim, ca: D2, b: Prim, cb: D2) -> D2 {
    let p = a.alpha + b.alpha;
    let mu = a.alpha * b.alpha / p;
    let d = ca - cb;
    let d2 = d * d;
    (3.0 - mu * 2.0 * d2)
        * (-(mu * d2)).exp()
        * (a.norm * b.norm * mu * (PI / p).powf(1.5))
}

/// `V = -Z N_a N_b (2 pi / p) K_ab F_0(p |P-C|^2)`
pub fn prim_nuclear(a: Prim, ca: D2, b: Prim, cb: D2, cc: D2, z: f64) -> D2 {
    let p = a.alpha + b.alpha;
    let mu = a.alpha * b.alpha / p;
    let pc = (ca * a.alpha + cb * b.alpha) * (1.0 / p);
    let d = ca - cb;
    let pmc = pc - cc;
    let t = (pmc * pmc) * p;
    (-(mu * (d * d))).exp() * boys0_d2(t) * (-z * a.norm * b.norm * (2.0 * PI / p))
}

/// Chemist-notation two-electron integral over primitives,
/// `(ab|cd) = int int a(1) b(1) r12^{-1} c(2) d(2)`:
///
/// ```text
/// = N_a N_b N_c N_d * 2 pi^{5/2} / (p q sqrt(p+q)) * K_ab K_cd * F_0( pq/(p+q) |P-Q|^2 )
/// ```
#[allow(clippy::too_many_arguments)]
pub fn prim_eri(a: Prim, ca: D2, b: Prim, cb: D2, c: Prim, cc: D2, d: Prim, cd: D2) -> D2 {
    let p = a.alpha + b.alpha;
    let q = c.alpha + d.alpha;
    let pc = (ca * a.alpha + cb * b.alpha) * (1.0 / p);
    let qc = (cc * c.alpha + cd * d.alpha) * (1.0 / q);
    let dab = ca - cb;
    let dcd = cc - cd;
    let k_ab = (-((dab * dab) * (a.alpha * b.alpha / p))).exp();
    let k_cd = (-((dcd * dcd) * (c.alpha * d.alpha / q))).exp();
    let pq = pc - qc;
    let t = (pq * pq) * (p * q / (p + q));
    let pref =
        a.norm * b.norm * c.norm * d.norm * 2.0 * PI_POW_2_5 / (p * q * (p + q).sqrt());
    k_ab * k_cd * boys0_d2(t) * pref
}
