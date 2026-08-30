//! `erf`, `erfc`, and the Boys function `F_m`, written here because the crate has no
//! dependencies and Rust's core has no error function.
//!
//! # The one design rule: never subtract two nearly-equal positive numbers
//!
//! Every branch below is chosen so the sum being accumulated has all-positive terms.
//! That is not stylistic. The naive route to the Boys function — `F0(t) =
//! sqrt(pi/4t) erf(sqrt t)` — is a ratio of two vanishing quantities as `t -> 0`, and
//! the naive route to `erfc` — `1 - erf(x)` — loses every digit that `erf` has gained
//! as `erf -> 1`. Both produce plausible-looking numbers with a third of the digits
//! gone, which is exactly the kind of error a curve gate cannot see.
//!
//! Measured errors, and how they were measured, are in `tests/special.rs`; the
//! constants below record the bound each branch is gated at.

use crate::dual::D2;

/// `sqrt(pi)`. A literal rather than `PI.sqrt()` so the series' scale factor is a
/// compile-time constant; `tests/special.rs` pins it to `PI.sqrt()` so it cannot drift
/// from the expression it stands for.
const SQRT_PI: f64 = 1.772_453_850_905_515_9_f64;

/// Exposed only so the pinning test can see it. Not part of the numerical API.
#[doc(hidden)]
pub const SQRT_PI_FOR_TEST: f64 = SQRT_PI;
const PI: f64 = core::f64::consts::PI;

/// Crossover between the ascending series and the `erf`-based closed form for `F_m`.
///
/// The series has all-positive terms and so is exact to roundoff at any `t`, but its
/// term count grows like `t`. At `t = 25` the closed form's own correction term
/// `erfc(sqrt t)` has fallen to `~1.6e-12` of the answer, so the continued fraction that
/// supplies it only needs four digits of its own to leave `F0` correct to full f64 —
/// which is why the handover is safe here and would not be at, say, `t = 4`.
pub const BOYS_SERIES_MAX_T: f64 = 25.0;

/// Crossover between the ascending series for `erf` and the continued fraction for
/// `erfc`. Below it `erf` is the accurate object and `erfc = 1 - erf`; above it `erfc`
/// is, and `erf = 1 - erfc`. At `x = 1.5` both sides are `O(1)` fractions of each other
/// (`erf = 0.966`, `erfc = 0.0339`), so neither subtraction loses more than two digits.
pub const ERF_SERIES_MAX_X: f64 = 1.5;

/// Relative tolerance at which the all-positive series and the continued fraction stop.
/// Below f64 epsilon on purpose: the loops are cheap and stopping at `eps` leaves the
/// last term's own rounding in the answer.
const CONVERGE: f64 = 1e-18;

/// Guard for the modified-Lentz recurrence, per Numerical Recipes: a denominator that
/// lands on zero is nudged rather than dividing by it.
const LENTZ_TINY: f64 = 1e-300;

/// `erf(x) = (2x/sqrt(pi)) e^{-x^2} sum_{i>=0} (2x^2)^i / (2i+1)!!`
///
/// The Kummer-transformed series, whose terms are ALL POSITIVE — unlike the plain
/// Maclaurin series `sum (-1)^i x^{2i+1}/(i!(2i+1))`, which alternates and loses
/// `~x^2/ln(10)` digits to cancellation before it converges.
fn erf_series(x: f64) -> f64 {
    let t = x * x;
    let mut term = 1.0f64;
    let mut sum = term;
    let mut i = 0usize;
    loop {
        i += 1;
        term *= (2.0 * t) / ((2 * i + 1) as f64);
        sum += term;
        if term < CONVERGE * sum || i > 400 {
            break;
        }
    }
    2.0 * x / SQRT_PI * (-t).exp() * sum
}

/// `erfc(x)` for `x >= ERF_SERIES_MAX_X`, by the Legendre continued fraction
///
/// ```text
/// erfc(x) = e^{-x^2}/sqrt(pi) * 1/(x + (1/2)/(x + 1/(x + (3/2)/(x + 2/(x + ...)))))
/// ```
///
/// evaluated with the modified Lentz recurrence (partial numerators `a_1 = 1`,
/// `a_j = (j-1)/2`; every partial denominator is `x`).
fn erfc_cf(x: f64) -> f64 {
    let mut f = LENTZ_TINY;
    let mut c = f;
    let mut d = 0.0f64;
    let mut j = 1usize;
    while j < 400 {
        let a = if j == 1 { 1.0 } else { (j - 1) as f64 / 2.0 };
        d = x + a * d;
        if d.abs() < LENTZ_TINY {
            d = LENTZ_TINY;
        }
        c = x + a / c;
        if c.abs() < LENTZ_TINY {
            c = LENTZ_TINY;
        }
        d = 1.0 / d;
        let delta = c * d;
        f *= delta;
        if (delta - 1.0).abs() < 1e-17 {
            break;
        }
        j += 1;
    }
    (-x * x).exp() / SQRT_PI * f
}

/// The error function.
///
/// Measured max relative error (see `tests/special.rs`): `1.1e-15` on `(0, 1.5]` against
/// the closed forms it is checked on, `3.5e-15` on `[1.5, 6]` inherited from `erfc`.
pub fn erf(x: f64) -> f64 {
    if x < 0.0 {
        return -erf(-x);
    }
    if x <= ERF_SERIES_MAX_X {
        erf_series(x)
    } else {
        1.0 - erfc_cf(x)
    }
}

/// The complementary error function.
///
/// Measured max relative error: `3.5e-15` for `x` in `[1, 10]`, rising to `~x^2 * eps`
/// beyond that. The growth is NOT a defect of the continued fraction: `erfc(x)` carries
/// the factor `e^{-x^2}`, and rounding `x^2` to f64 at all costs `exp` a relative
/// `x^2 * eps` — `6.9e-14` at `x = 25`. No f64 implementation does better, and the one
/// caller that reaches that range (`boys` above `t = 25`) uses the value as a correction
/// of relative size `1e-12`, where four correct digits are already six more than needed.
pub fn erfc(x: f64) -> f64 {
    if x < ERF_SERIES_MAX_X {
        1.0 - erf(x)
    } else {
        erfc_cf(x)
    }
}

/// `F_0`, `F_1` and `F_2` of the Boys function
///
/// ```text
/// F_m(t) = int_0^1 u^{2m} exp(-t u^2) du
/// ```
///
/// returned together because the derivative rules need all three from one argument
/// (`F_0' = -F_1`, `F_0'' = F_2`) and they share their work.
///
/// # The two branches
///
/// The three-term relation `F_{m+1} = ((2m+1) F_m - e^{-t}) / (2t)` is stable in ONE
/// direction at a time and the direction flips, so there are two implementations:
/// [`boys012_series`] below `BOYS_SERIES_MAX_T` and [`boys012_closed`] above it. Each
/// documents why its recursion runs the way it does. Both are public so a test can
/// evaluate them at the SAME argument — comparing across the seam instead measures the
/// function's own slope and would call any crossover agreement.
pub fn boys012(t: f64) -> [f64; 3] {
    if t <= BOYS_SERIES_MAX_T {
        boys012_series(t)
    } else {
        boys012_closed(t)
    }
}

/// The `t <= 25` branch: `F_2` from its own all-positive series, then DOWNWARD
/// recursion `F_m = (2t F_{m+1} + e^{-t}) / (2m+1)`.
///
/// Downward is the addition of two positive numbers, so it cannot cancel. Upward here
/// would subtract `e^{-t} ~ 1` from `(2m+1) F_m ~ 1` and divide by a small `2t`, which
/// loses everything as `t -> 0`.
///
/// Correct at ANY `t`, and exposed separately so the two branches can be compared at the
/// same argument rather than across a step. Its cost grows like `t` and its intermediate
/// sum grows like `e^t`, so it overflows somewhere past `t = 700`; that is why the
/// crossover exists at all, and why this is not the shipped path everywhere.
pub fn boys012_series(t: f64) -> [f64; 3] {
    let et = (-t).exp();
    // F_2(t) = e^{-t} sum_i (2t)^i * 3!! / (2*2 + 2i + 1)!!; the i = 0 term is
    // 3!!/5!! = 3/15 = 1/5, and every later term is positive.
    let mut term = 1.0 / 5.0;
    let mut sum = term;
    let mut i = 0usize;
    loop {
        i += 1;
        term *= (2.0 * t) / ((2 * i + 5) as f64);
        sum += term;
        if term < CONVERGE * sum || i > 4000 {
            break;
        }
    }
    let f2 = et * sum;
    let f1 = (2.0 * t * f2 + et) / 3.0;
    let f0 = 2.0 * t * f1 + et;
    [f0, f1, f2]
}

/// The `t > 25` branch: `F_0` from the closed form `sqrt(pi/4t) erf(sqrt t)`, then
/// UPWARD recursion `F_{m+1} = ((2m+1) F_m - e^{-t}) / (2t)`.
///
/// Upward is a subtraction, and here it is a safe one: at `t = 25`, `e^{-t} = 1.4e-11`
/// against `F_0 = 0.18`, so nothing cancels. Below `t ~ 1` the same subtraction would
/// take `1 - 1` and the branch would be worthless, which is the other half of why the
/// crossover sits where it does.
pub fn boys012_closed(t: f64) -> [f64; 3] {
    let et = (-t).exp();
    let f0 = (PI / (4.0 * t)).sqrt() * (1.0 - erfc_cf(t.sqrt()));
    let f1 = (f0 - et) / (2.0 * t);
    let f2 = (3.0 * f1 - et) / (2.0 * t);
    [f0, f1, f2]
}

/// `F_0(t)` alone.
pub fn boys0(t: f64) -> f64 {
    boys012(t)[0]
}

/// `F_0` of a differentiated argument.
///
/// The derivative rules are exact rather than numerical: `dF_0/dt = -F_1` and
/// `d2F_0/dt2 = +F_2` follow from differentiating under the integral sign, so the chain
/// rule closes on other Boys functions of the SAME argument, which `boys012` has already
/// computed.
pub fn boys0_d2(t: D2) -> D2 {
    let [f0, f1, f2] = boys012(t.v);
    t.compose(f0, -f1, f2)
}

// ----------------------------------------------------------- the general Boys ladder
//
// The H2 path above needs `F_0`, `F_1`, `F_2` and nothing else, because s-type integrals
// reach `F_0` and second-order differentiation reaches two rungs past it. The first row
// brings p functions, and a (pp|pp) quartet's Hermite Coulomb tensor reaches `F_4`; two
// more rungs for `D2` puts the requirement at `F_6`. The ladder below is the same two
// branches, at general order.
//
// `boys012` is left EXACTLY as it was. It is the function the pinned H2 referee gate
// grades, and a "generalisation" that changed its arithmetic by one rounding would move
// a number the gate has already measured. The two are checked against each other in
// `tests/md.rs` instead, at the same argument, which is the honest way to claim they are
// the same function.

/// Highest Boys order this module supplies: `F_0 .. F_16`.
///
/// Set by the worst case with f-orbitals (l=3) plus the derivative
/// rungs: a (ff|ff) quartet needs `F_12`, `D2` needs `F_{m+2}` (F_14) to close its chain rule,
/// and spare rungs are kept.
pub const BOYS_MAX_M: usize = 16;

/// `F_0(t) .. F_{m_max}(t)`, all orders returned together because they share their work
/// and the recursions need each other.
///
/// Same two branches and the same reasoning as [`boys012`]: below `BOYS_SERIES_MAX_T`
/// the top order comes from its own all-positive series and the rest by DOWNWARD
/// recursion (an addition of two positive numbers); above it `F_0` comes from the
/// closed form and the rest by UPWARD recursion (a subtraction that is safe only once
/// `e^{-t}` is negligible against `F_0`).
///
/// Orders above `m_max` are returned as zero and must not be read.
pub fn boys_upto(m_max: usize, t: f64) -> [f64; BOYS_MAX_M + 1] {
    assert!(m_max <= BOYS_MAX_M, "Boys order {m_max} above BOYS_MAX_M");
    let mut f = [0.0f64; BOYS_MAX_M + 1];
    let et = (-t).exp();
    if t <= BOYS_SERIES_MAX_T {
        // F_m(t) = e^{-t} sum_{i>=0} (2t)^i (2m-1)!! / (2m+2i+1)!!, whose i = 0 term is
        // 1/(2m+1) and whose every later term is positive.
        let m = m_max;
        let mut term = 1.0 / ((2 * m + 1) as f64);
        let mut sum = term;
        let mut i = 0usize;
        loop {
            i += 1;
            term *= (2.0 * t) / ((2 * m + 2 * i + 1) as f64);
            sum += term;
            if term < CONVERGE * sum || i > 4000 {
                break;
            }
        }
        f[m] = et * sum;
        for k in (0..m).rev() {
            f[k] = (2.0 * t * f[k + 1] + et) / ((2 * k + 1) as f64);
        }
    } else {
        f[0] = (PI / (4.0 * t)).sqrt() * (1.0 - erfc_cf(t.sqrt()));
        for k in 0..m_max {
            f[k + 1] = (((2 * k + 1) as f64) * f[k] - et) / (2.0 * t);
        }
    }
    f
}

/// The same ladder of a differentiated argument: `F_m(t(R))` carrying `d/dR` and
/// `d2/dR2`, for every `m` up to `m_max`.
///
/// `dF_m/dt = -F_{m+1}` and `d2F_m/dt2 = +F_{m+2}` — which is why the value array is
/// computed two rungs higher than the caller asked for, and why an order-`m` request
/// costs nothing beyond that.
pub fn boys_d2_upto(m_max: usize, t: D2) -> [D2; BOYS_MAX_M + 1] {
    assert!(
        m_max + 2 <= BOYS_MAX_M,
        "Boys order {m_max} + 2 derivative rungs above BOYS_MAX_M"
    );
    let f = boys_upto(m_max + 2, t.v);
    let mut out = [D2::c(0.0); BOYS_MAX_M + 1];
    for (m, o) in out.iter_mut().enumerate().take(m_max + 1) {
        *o = t.compose(f[m], -f[m + 1], f[m + 2]);
    }
    out
}
