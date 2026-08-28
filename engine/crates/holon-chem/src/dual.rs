//! Second-order forward-mode differentiation in ONE variable.
//!
//! # Why this and not a finite difference
//!
//! The renderer needs `dE/dR` and `d2E/dR2` to the same accuracy as `E`, and the two
//! usual routes both give that up. A central difference costs half the digits of `E`
//! (best case `~1e-8` on a curve whose values are good to `1e-15`), and differentiating
//! the Hermite interpolant instead measures the interpolant rather than the model. What
//! follows is the third route: the chain rule applied to the closed forms themselves.
//! Every arithmetic step of the integral evaluation carries its own first and second
//! derivative alongside the value, so the numbers that come out are the ANALYTIC
//! derivatives of the model, accurate to f64 roundoff and not to a step size.
//!
//! # Why the value is not perturbed by carrying them
//!
//! `D2`'s value component is computed by exactly the operations a plain `f64` path
//! would use — `add` is `a.v + b.v`, `mul` is `a.v * b.v`, `sqrt` is `a.v.sqrt()` — so
//! running the model in `D2` returns the same `E` as running it in `f64`, bit for bit.
//! That is what lets one implementation serve both the curve and its derivatives with
//! no second copy of the physics to keep true.
//!
//! Only the operations the model actually uses are implemented. There is no `ln`, no
//! `powf`, no trigonometry: adding one would mean adding its derivative rules, and an
//! unused rule is an untested rule.

use core::ops::{Add, Div, Mul, Neg, Sub};

/// A value carried with its first and second derivative with respect to the single
/// independent variable: `(f, f', f'')`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct D2 {
    pub v: f64,
    pub d: f64,
    pub e: f64,
}

impl D2 {
    /// A constant: derivative zero.
    pub const fn c(v: f64) -> Self {
        Self { v, d: 0.0, e: 0.0 }
    }

    /// THE independent variable, seeded at `x`.
    pub const fn var(x: f64) -> Self {
        Self {
            v: x,
            d: 1.0,
            e: 0.0,
        }
    }

    pub const fn new(v: f64, d: f64, e: f64) -> Self {
        Self { v, d, e }
    }

    /// `exp(g)`: `f' = e^g g'`, `f'' = e^g (g'^2 + g'')`.
    pub fn exp(self) -> Self {
        let v = self.v.exp();
        Self {
            v,
            d: v * self.d,
            e: v * (self.d * self.d + self.e),
        }
    }

    /// `sqrt(g)`, from `f^2 = g`: `f' = g'/(2f)`, `f'' = (g'' - 2 f'^2)/(2f)`.
    pub fn sqrt(self) -> Self {
        let v = self.v.sqrt();
        let d = self.d / (2.0 * v);
        Self {
            v,
            d,
            e: (self.e - 2.0 * d * d) / (2.0 * v),
        }
    }

    /// Compose with an outer scalar function supplied as `(h(x), h'(x), h''(x))`.
    /// Used for the Boys function, whose derivatives are other Boys functions rather
    /// than expressions in its own value.
    pub fn compose(self, h: f64, h1: f64, h2: f64) -> Self {
        Self {
            v: h,
            d: h1 * self.d,
            e: h2 * self.d * self.d + h1 * self.e,
        }
    }
}

impl Add for D2 {
    type Output = D2;
    fn add(self, o: D2) -> D2 {
        D2 {
            v: self.v + o.v,
            d: self.d + o.d,
            e: self.e + o.e,
        }
    }
}

impl Sub for D2 {
    type Output = D2;
    fn sub(self, o: D2) -> D2 {
        D2 {
            v: self.v - o.v,
            d: self.d - o.d,
            e: self.e - o.e,
        }
    }
}

impl Neg for D2 {
    type Output = D2;
    fn neg(self) -> D2 {
        D2 {
            v: -self.v,
            d: -self.d,
            e: -self.e,
        }
    }
}

impl Mul for D2 {
    type Output = D2;
    fn mul(self, o: D2) -> D2 {
        D2 {
            v: self.v * o.v,
            d: self.d * o.v + self.v * o.d,
            e: self.e * o.v + 2.0 * self.d * o.d + self.v * o.e,
        }
    }
}

impl Div for D2 {
    type Output = D2;
    fn div(self, o: D2) -> D2 {
        // f = u/w  =>  f' = (u' - f w')/w,  f'' = (u'' - 2 f' w' - f w'')/w
        let v = self.v / o.v;
        let d = (self.d - v * o.d) / o.v;
        D2 {
            v,
            d,
            e: (self.e - 2.0 * d * o.d - v * o.e) / o.v,
        }
    }
}

// Scalar mixes. A scalar is a constant, so these are strictly cheaper than promoting it
// with `D2::c` and going through the general rule — the zero derivative terms drop out
// at compile time rather than being multiplied by zero at runtime.
impl Add<f64> for D2 {
    type Output = D2;
    fn add(self, k: f64) -> D2 {
        D2 {
            v: self.v + k,
            d: self.d,
            e: self.e,
        }
    }
}

impl Sub<f64> for D2 {
    type Output = D2;
    fn sub(self, k: f64) -> D2 {
        D2 {
            v: self.v - k,
            d: self.d,
            e: self.e,
        }
    }
}

impl Sub<D2> for f64 {
    type Output = D2;
    fn sub(self, o: D2) -> D2 {
        D2 {
            v: self - o.v,
            d: -o.d,
            e: -o.e,
        }
    }
}

impl Mul<f64> for D2 {
    type Output = D2;
    fn mul(self, k: f64) -> D2 {
        D2 {
            v: self.v * k,
            d: self.d * k,
            e: self.e * k,
        }
    }
}

impl Mul<D2> for f64 {
    type Output = D2;
    fn mul(self, o: D2) -> D2 {
        o * self
    }
}

impl Div<D2> for f64 {
    type Output = D2;
    fn div(self, o: D2) -> D2 {
        D2::c(self) / o
    }
}
