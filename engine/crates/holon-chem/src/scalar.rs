//! The arithmetic tier seam: one solver logic, written once, generic over the scalar.
//!
//! The 2026-08-30 escalation measured that every non-trivial all-electron solve stagnates
//! at 96–100% of a hardcoded 1e-10 — the f64 tier's edge, written three times under three
//! names. The ruling (user-directed): the edge is a TIER BOUNDARY, and a residual that
//! must go deeper does not get there by tuning constants — it gets there on the next tier
//! of arithmetic. This module is that tier's seam: the [`Scalar`] trait is what
//! `sigma_direct`, `jacobi_eigh` and the Davidson driver in [`crate::tier`] are generic
//! over, so the f64 production path and the double-double refinement path are ONE body of
//! logic monomorphised twice, not two implementations that can drift.
//!
//! What stays deliberately separate, and why it is not a DRY violation: the
//! connected-determinant checker (`sigma_reference`) and the mpmath referee share the
//! ALGEBRA but not the code — their evidential value IS their independence. The
//! duplication this module removes is the forbidden kind: the same factorisation
//! hand-written per arithmetic type.

use core::ops::{Add, Div, Mul, Neg, Sub};

/// The f64 tier's expansion floor. Re-exported by `fci::DAVIDSON_EXPANSION_FLOOR`
/// (the name every gate reads) — ONE numeric source, aliased, never repeated.
pub const F64_EXPANSION_FLOOR: f64 = 1e-10;

/// The double-double tier's expansion floor, PROVISIONAL until calibrated.
///
/// eps for double-double is 2⁻¹⁰⁴ ≈ 4.9e-32; accumulated roundoff at the largest staked
/// scale (|E| ~ 6e3 Ha, n_det ~ 1e6) sits near 5e-26·‖H‖-ish. 1e-24 leaves two orders of
/// headroom. Calibration protocol: solve the two SMALL formerly-refused atoms (Te 729
/// dets, Sb 9,477) and read where their residuals actually pin before staking anything on
/// the large ones. If measured stagnation sits above this floor, the constant moves and
/// says so in its own history — that is instrument calibration, not result-gating.
pub const DD_EXPANSION_FLOOR: f64 = 1e-24;

/// What the generic solver needs from a number. Implemented by `f64` (the production
/// tier — every method is the native operation, so monomorphisation reproduces the
/// pre-generic code bit for bit) and by [`Dd`].
pub trait Scalar:
    Copy
    + PartialOrd
    + core::fmt::Debug
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    const ZERO: Self;
    const ONE: Self;
    fn from_f64(v: f64) -> Self;
    fn to_f64(self) -> f64;
    /// `self * k` where `k` is an exactly-representable coefficient (a sign, a half,
    /// a power of two). Kept separate from full multiplication so the f64 path is the
    /// literal native product and the Dd path can skip a cross term it knows is exact.
    fn scale(self, k: f64) -> Self;
    fn abs(self) -> Self;
    fn sqrt(self) -> Self;
    fn is_zero(self) -> bool;
    /// The tier edge: the orthogonalised-candidate norm below which a new Davidson
    /// direction is roundoff, compared in f64 because the FLOOR is an f64-scale fact
    /// about the tier, not a value that needs the tier's own precision.
    fn expansion_floor() -> f64;
    /// The Jacobi sweep's off-diagonal machine-zero, squared-sum scale.
    fn jacobi_off_floor() -> f64;
}

impl Scalar for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    #[inline(always)]
    fn from_f64(v: f64) -> Self {
        v
    }
    #[inline(always)]
    fn to_f64(self) -> f64 {
        self
    }
    #[inline(always)]
    fn scale(self, k: f64) -> Self {
        self * k
    }
    #[inline(always)]
    fn abs(self) -> Self {
        f64::abs(self)
    }
    #[inline(always)]
    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }
    #[inline(always)]
    fn is_zero(self) -> bool {
        self == 0.0
    }
    #[inline(always)]
    fn expansion_floor() -> f64 {
        F64_EXPANSION_FLOOR
    }
    #[inline(always)]
    fn jacobi_off_floor() -> f64 {
        1e-30
    }
}

/// Double-double: an unevaluated sum `hi + lo` of two f64 with `|lo| ≤ ulp(hi)/2`,
/// carrying ~32 significant digits. Dekker/Knuth error-free transformations; products
/// use `f64::mul_add` (exactly rounded FMA), which makes `two_prod` two operations.
///
/// This is the OVERFLOW TIER for the Davidson floor — self-contained, no dependency,
/// ~10–20× f64 cost. It is not the referee: the referee's 50-digit mpmath route stays an
/// independent implementation on purpose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dd {
    pub hi: f64,
    pub lo: f64,
}

#[inline(always)]
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    (s, (a - (s - bb)) + (b - bb))
}

#[inline(always)]
fn quick_two_sum(a: f64, b: f64) -> (f64, f64) {
    // requires |a| >= |b| or a == 0
    let s = a + b;
    (s, b - (s - a))
}

#[inline(always)]
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    (p, a.mul_add(b, -p))
}

impl Dd {
    #[inline(always)]
    pub const fn from_f64_const(v: f64) -> Dd {
        Dd { hi: v, lo: 0.0 }
    }
}

impl Add for Dd {
    type Output = Dd;
    #[inline]
    fn add(self, o: Dd) -> Dd {
        let (s, e) = two_sum(self.hi, o.hi);
        let e = e + self.lo + o.lo;
        let (hi, lo) = quick_two_sum(s, e);
        Dd { hi, lo }
    }
}

impl Sub for Dd {
    type Output = Dd;
    #[inline]
    fn sub(self, o: Dd) -> Dd {
        self + (-o)
    }
}

impl Mul for Dd {
    type Output = Dd;
    #[inline]
    fn mul(self, o: Dd) -> Dd {
        let (p, e) = two_prod(self.hi, o.hi);
        let e = e + (self.hi * o.lo + self.lo * o.hi);
        let (hi, lo) = quick_two_sum(p, e);
        Dd { hi, lo }
    }
}

impl Div for Dd {
    type Output = Dd;
    #[inline]
    fn div(self, o: Dd) -> Dd {
        // Two Newton-corrected long-division steps: q1 approximates, the exactly
        // computed remainder feeds q2, then q3 catches the last ulp of lo.
        let q1 = self.hi / o.hi;
        let r = self - o * Dd::from_f64_const(q1);
        let q2 = r.hi / o.hi;
        let r2 = r - o * Dd::from_f64_const(q2);
        let q3 = r2.hi / o.hi;
        let (s, e) = quick_two_sum(q1, q2);
        Dd { hi: s, lo: e } + Dd::from_f64_const(q3)
    }
}

impl Neg for Dd {
    type Output = Dd;
    #[inline(always)]
    fn neg(self) -> Dd {
        Dd { hi: -self.hi, lo: -self.lo }
    }
}

impl PartialOrd for Dd {
    #[inline]
    fn partial_cmp(&self, o: &Dd) -> Option<core::cmp::Ordering> {
        match self.hi.partial_cmp(&o.hi) {
            Some(core::cmp::Ordering::Equal) => self.lo.partial_cmp(&o.lo),
            other => other,
        }
    }
}

impl Scalar for Dd {
    const ZERO: Self = Dd { hi: 0.0, lo: 0.0 };
    const ONE: Self = Dd { hi: 1.0, lo: 0.0 };
    #[inline(always)]
    fn from_f64(v: f64) -> Self {
        Dd { hi: v, lo: 0.0 }
    }
    #[inline(always)]
    fn to_f64(self) -> f64 {
        self.hi + self.lo
    }
    #[inline]
    fn scale(self, k: f64) -> Self {
        let (p, e) = two_prod(self.hi, k);
        let e = e + self.lo * k;
        let (hi, lo) = quick_two_sum(p, e);
        Dd { hi, lo }
    }
    #[inline(always)]
    fn abs(self) -> Self {
        if self.hi < 0.0 || (self.hi == 0.0 && self.lo < 0.0) {
            -self
        } else {
            self
        }
    }
    #[inline]
    fn sqrt(self) -> Self {
        if self.hi == 0.0 && self.lo == 0.0 {
            return Dd::ZERO;
        }
        // One DD-space Newton step from the f64 seed doubles its precision, which is
        // exactly the gap between f64 and double-double.
        let y0 = self.hi.sqrt();
        let y = Dd::from_f64_const(y0);
        let r = self - y * y;
        y + r.scale(0.5 / y0)
    }
    #[inline(always)]
    fn is_zero(self) -> bool {
        self.hi == 0.0 && self.lo == 0.0
    }
    #[inline(always)]
    fn expansion_floor() -> f64 {
        DD_EXPANSION_FLOOR
    }
    #[inline(always)]
    fn jacobi_off_floor() -> f64 {
        1e-60
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dd(v: f64) -> Dd {
        Dd::from_f64(v)
    }

    /// The defining capability f64 lacks: absorb a unit into 1e16 and get it back.
    #[test]
    fn keeps_what_f64_drops() {
        let big = 1e16;
        assert_eq!((big + 1.0) - big, 0.0, "if f64 kept this, the test tests nothing");
        let s = dd(big) + dd(1.0);
        let back = s - dd(big);
        assert_eq!(back.to_f64(), 1.0);
    }

    #[test]
    fn add_sub_roundtrip_tight() {
        let a = dd(1.0) / dd(3.0);
        let b = dd(1.0) / dd(7.0);
        let r = (a + b) - b - a;
        assert!(r.to_f64().abs() < 1e-30, "residue {}", r.to_f64());
    }

    #[test]
    fn mul_matches_known_expansion() {
        // (1/3)*3 == 1 to DD precision
        let t = dd(1.0) / dd(3.0);
        let r = t.scale(3.0) - Dd::ONE;
        assert!(r.to_f64().abs() < 1e-31, "residue {}", r.to_f64());
        let r2 = t * dd(3.0) - Dd::ONE;
        assert!(r2.to_f64().abs() < 1e-31, "residue {}", r2.to_f64());
    }

    #[test]
    fn div_newton_converges() {
        let x = dd(std::f64::consts::PI);
        let y = dd(std::f64::consts::E);
        let r = (x / y) * y - x;
        assert!(r.to_f64().abs() < 1e-30, "residue {}", r.to_f64());
    }

    #[test]
    fn sqrt_squares_back() {
        for v in [2.0, 3.0, 1e-8, 1e8, 0.5, 6547.122368855257] {
            let s = dd(v).sqrt();
            let r = s * s - dd(v);
            assert!(
                r.to_f64().abs() < 1e-28 * v.max(1.0),
                "sqrt({v}) residue {}",
                r.to_f64()
            );
        }
        assert!(Dd::ZERO.sqrt().is_zero());
    }

    #[test]
    fn ordering_uses_both_limbs() {
        let a = dd(1.0) + dd(1e-20);
        let b = dd(1.0);
        assert!(a > b, "the lo limb must decide ties");
        assert!((-a) < (-b));
        assert_eq!(a.abs().to_f64(), (-a).abs().to_f64());
    }

    #[test]
    fn f64_impl_is_native() {
        // The f64 instantiation must BE the native operation — this is the bit-identity
        // contract the generic solver relies on.
        assert_eq!(<f64 as Scalar>::from_f64(0.1), 0.1);
        assert_eq!(0.3f64.scale(0.5), 0.3 * 0.5);
        assert_eq!(<f64 as Scalar>::expansion_floor(), F64_EXPANSION_FLOOR);
    }

    /// A dot-product-shaped accumulation at DD keeps ~30 digits where f64 keeps 16:
    /// sum 1e5 copies of 0.1 minus the exact rational answer.
    #[test]
    fn accumulation_error_bounded() {
        let n = 100_000;
        let mut acc = Dd::ZERO;
        for _ in 0..n {
            acc = acc + dd(0.1);
        }
        // 0.1 is not exact in binary; the DD sum must equal n * fl(0.1) to DD precision,
        // i.e. the error vs n*dd(0.1) computed by scaling must be ~1e-27, not f64's ~1e-12.
        let direct = dd(0.1).scale(n as f64);
        let r = acc - direct;
        assert!(r.to_f64().abs() < 1e-26, "residue {}", r.to_f64());
    }
}
