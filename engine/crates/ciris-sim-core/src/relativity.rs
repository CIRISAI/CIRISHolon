//! Special relativity in a declared chart: worldlines, the 4-momentum ledger, and the
//! SR -> Newton certificate (Gantt A4).
//!
//! ## Scope (P8, binding)
//!
//! Free particles, geodesics (in [`crate::curvature`]), and CONTACT collisions ONLY.
//! No long-range forces are represented at this tier, and that is not an omission to be
//! papered over later: the Currie-Jordan-Sudarshan no-interaction theorem (RMP 35, 350
//! (1963)) forbids covariant instantaneous forces — a force-mediating relation must be a
//! dynamical holon with its own gross ledger entries (momentum in flight, M16). That
//! relation-momentum machinery is deliberately NOT built in this slice. Any covariant
//! force query against this module is out of scope by theorem, not by TODO.
//!
//! ## Chart discipline
//!
//! Signature (+,-,-,-), SI units, `x^mu = (ct, x, y, z)` in metres, `u^mu = dx^mu/dtau`
//! in m/s. Every function documents which chart its statement lives in. The chart
//! annotation is Record-axis freight (the L8/M15 "chart is a live 12+2 risk" flag): a
//! total 4-momentum is a sum over a constant-t slice OF THIS CHART's foliation, and the
//! foliation-relativity of totals (von Laue) is recorded here rather than routed around.
//! Totals reported by [`total_momentum`] are declared on the integration chart's
//! constant-t foliation and are exactly additive there; they are not chart-free facts.
//!
//! ## The mass shell is a certificate here, not an identity (the section 3.5 repair)
//!
//! DESCRIPTOR_CHAIN.md section 3.5 demotes the mass-shell residual to a representation
//! invariant in any (x, gamma v) representation, because deriving `u0` from the spatial
//! velocity satisfies `u.u = c^2` algebraically on arbitrarily wrong trajectories. This
//! module takes the other horn: `u0` is EVOLVED as an independent degree of freedom by
//! the integrator, so `u.u - c^2` genuinely drifts under integration error and is gated
//! as a certificate. The test
//! `shell_identity_on_a_wrecked_trajectory_vs_shell_certificate` demonstrates both
//! horns: a deliberately wrecked trajectory has residual ~0 when `u0` is recomputed from
//! the shell, and a large residual when `u0` is carried independently.
//!
//! ## Cancellation-safe representations (pinned by section 3.5)
//!
//! * `gamma - 1` is computed as `|u_spatial|^2 / (c^2 (gamma + 1))`, never as a
//!   subtraction — at beta = 1e-4 the naive form loses seven digits.
//! * The CM momentum p* uses the fully factored Kallen form
//!   `[M-(m1+m2)][M+(m1+m2)][M-(m1-m2)][M+(m1-m2)]`. Given an f64 `M`, the small
//!   factor `M-(m1+m2)` is an EXACT subtraction near threshold (Sterbenz), so the
//!   factored form extracts every digit the input carries — precisely what the
//!   expanded Kallen polynomial destroys by cancelling through terms of size `M^4`
//!   (the planted defect in the gate). Separately, ROUNDING `M` upstream of the call
//!   is itself catastrophic near threshold (one ulp of `M` moves p* by tens of
//!   percent — measured in the gate), so the near-threshold API takes the threshold
//!   EXCESS `M-(m1+m2)` as the primitive.
//! * `cosh(s)-1` is `2 sinh^2(s/2)`; `sqrt(1+e)-1` is `e/(sqrt(1+e)+1)`.
//!
//! ## The 4-momentum ledger and the REG+ misfit (recorded, not routed around)
//!
//! [`FourMomentum`] is exactly additive and commutes with every Lorentz boost
//! (linearity: boost of the sum = sum of the boosts, gated). It CANNOT ride
//! [`crate::regplus::GrossState`]: that ledger's momentum is `[i64; 2]` — the exact
//! integer FHP sector label proved in `Core/Lattice.lean` — and a continuum f64 `P^mu`
//! does not embed in it without a quantisation decision that belongs to the
//! `sector_table_is_pmu_table` relabeling, which is proved only for the single-speed
//! lattice gas in its own chart. Until that decision is made deliberately, the SR ledger
//! is carried as this parallel additive object, and the mismatch is a recorded misfit,
//! not a workaround.
//!
//! ## Determinism
//!
//! Fixed evaluation order, no allocation, f64 only: bit-identical replay, gated by test.

use crate::holon::CertificationStatus;
use libm::{atanh, cosh, sinh, sqrt};

/// Exact SI speed of light.
pub const SPEED_OF_LIGHT_M_S: f64 = 299_792_458.0;

const C: f64 = SPEED_OF_LIGHT_M_S;

/// Declared chart for a worldline or a ledger total. Record-axis freight (M15): this
/// names the foliation a statement lives on; it is not a new frame axis on the holon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChartId(pub u32);

/// Flat Minkowski chart, signature (+,-,-,-), constant-t foliation.
pub const CHART_MINKOWSKI: ChartId = ChartId(1);
/// The force-free Newtonian chart licensed by [`certify_newton_chart`].
pub const CHART_NEWTONIAN_LIMIT: ChartId = ChartId(2);

/// Minkowski inner product, signature (+,-,-,-).
pub const fn minkowski_dot(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3]
}

/// One worldline in a declared chart: position 4-vector and 4-velocity.
///
/// `u[0]` is an independent degree of freedom of the integrator (see module docs); the
/// shell residual [`Worldline::shell_residual`] is therefore a certificate observable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Worldline {
    /// `x^mu = (ct, x, y, z)` in metres.
    pub x: [f64; 4],
    /// `u^mu = dx^mu/dtau` in m/s; on shell, `u.u = c^2`.
    pub u: [f64; 4],
}

impl Worldline {
    /// Build from spatial celerity `w = gamma v` (m/s). This is the cancellation-safe
    /// entry: `u0 = c sqrt(1 + |w|^2/c^2)` never subtracts, and `gamma - 1` is
    /// recoverable to full precision at any speed.
    pub fn from_celerity(x: [f64; 4], w: [f64; 3]) -> Self {
        let w2 = w[0] * w[0] + w[1] * w[1] + w[2] * w[2];
        let u0 = C * sqrt(1.0 + w2 / (C * C));
        Self { x, u: [u0, w[0], w[1], w[2]] }
    }

    /// Lorentz factor read from the independently evolved `u0`.
    pub fn gamma(&self) -> f64 {
        self.u[0] / C
    }

    /// `gamma - 1` computed without subtraction: `|u_spatial|^2 / (c^2 (gamma + 1))`.
    /// Exact identity on shell; at beta = 1e-4 the naive `gamma - 1` loses ~7 digits
    /// and this form loses none.
    pub fn gamma_minus_one(&self) -> f64 {
        let w2 = self.u[1] * self.u[1] + self.u[2] * self.u[2] + self.u[3] * self.u[3];
        w2 / (C * C * (self.gamma() + 1.0))
    }

    /// Coordinate velocity `v^i = u^i c / u^0` (m/s).
    pub fn coordinate_velocity(&self) -> [f64; 3] {
        let s = C / self.u[0];
        [self.u[1] * s, self.u[2] * s, self.u[3] * s]
    }

    /// Relative shell residual `(u.u - c^2)/c^2`. A certificate observable because
    /// `u0` is evolved independently; recomputing `u0` from the shell would pin this to
    /// zero on arbitrarily wrong trajectories (the section 3.5 warning, demonstrated in
    /// the tests).
    pub fn shell_residual(&self) -> f64 {
        (minkowski_dot(&self.u, &self.u) - C * C) / (C * C)
    }

    /// Kinetic energy `m c^2 (gamma - 1)` in J, via the cancellation-safe form.
    pub fn kinetic_energy_j(&self, mass_kg: f64) -> f64 {
        mass_kg * C * C * self.gamma_minus_one()
    }
}

/// Additive energy-momentum ledger entry: `p^mu = (E/c, p_x, p_y, p_z)` in kg m/s.
///
/// Conserved gross quantity for free motion and contact collisions in this slice. See
/// the module docs for why it does not ride `GrossState` (recorded misfit).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FourMomentum {
    pub p: [f64; 4],
}

impl FourMomentum {
    pub const ZERO: Self = Self { p: [0.0; 4] };

    /// The 4-momentum `m u^mu` of a worldline (kg m/s).
    pub fn of(mass_kg: f64, w: &Worldline) -> Self {
        Self {
            p: [mass_kg * w.u[0], mass_kg * w.u[1], mass_kg * w.u[2], mass_kg * w.u[3]],
        }
    }

    /// Exact additive composition (the ledger operation).
    pub fn add(self, other: Self) -> Self {
        Self {
            p: [
                self.p[0] + other.p[0],
                self.p[1] + other.p[1],
                self.p[2] + other.p[2],
                self.p[3] + other.p[3],
            ],
        }
    }

    /// Minkowski norm `p.p` in (kg m/s)^2; `= (m c)^2` on shell.
    pub fn dot_self(&self) -> f64 {
        minkowski_dot(&self.p, &self.p)
    }

    /// Invariant mass `sqrt(p.p)/c` in kg (composite `M >= sum m_i`).
    pub fn invariant_mass_kg(&self) -> f64 {
        sqrt(self.dot_self()) / C
    }
}

/// Ledger total over a constant-t slice of the declared chart. Exactly additive THERE;
/// foliation-relative in general (von Laue, L8/M15) — the chart annotation is the
/// Record-axis fact that makes this an honest number.
pub fn total_momentum(entries: &[FourMomentum]) -> FourMomentum {
    let mut total = FourMomentum::ZERO;
    for e in entries {
        total = total.add(*e);
    }
    total
}

/// Pure Lorentz boost of any 4-vector by velocity `beta = v/c` (|beta| < 1).
///
/// `gamma - 1` is carried as `gamma^2 beta^2 / (gamma + 1)` so low-velocity boosts do
/// not cancel. Linear in the boosted vector by construction; linearity and invariance
/// of `p.p` are separately gated (a wrong-but-linear boost passes linearity and fails
/// invariance — the tests keep both because either alone is an unfailable gate for the
/// other defect class).
pub fn boost(v: &[f64; 4], beta: &[f64; 3]) -> [f64; 4] {
    let b2 = beta[0] * beta[0] + beta[1] * beta[1] + beta[2] * beta[2];
    if b2 == 0.0 {
        return *v;
    }
    let gamma = 1.0 / sqrt(1.0 - b2);
    let gm1 = gamma * gamma * b2 / (gamma + 1.0);
    let bp = beta[0] * v[1] + beta[1] * v[2] + beta[2] * v[3];
    let t = gamma * (v[0] - bp);
    let k = gm1 * bp / b2 - gamma * v[0];
    [
        t,
        v[1] + k * beta[0],
        v[2] + k * beta[1],
        v[3] + k * beta[2],
    ]
}

// ---------------------------------------------------------------------------------
// Integrator: fixed-step RK4 over (x^mu, u^mu) in proper time.
// ---------------------------------------------------------------------------------

/// One classical RK4 step of the 8-dimensional state `y = (x^mu, u^mu)` with
/// derivative `f`. Fixed evaluation order; shared with [`crate::curvature`].
pub fn rk4_step<F: Fn(&[f64; 8]) -> [f64; 8]>(f: &F, y: &[f64; 8], h: f64) -> [f64; 8] {
    let k1 = f(y);
    let mut y2 = [0.0; 8];
    for i in 0..8 {
        y2[i] = y[i] + 0.5 * h * k1[i];
    }
    let k2 = f(&y2);
    let mut y3 = [0.0; 8];
    for i in 0..8 {
        y3[i] = y[i] + 0.5 * h * k2[i];
    }
    let k3 = f(&y3);
    let mut y4 = [0.0; 8];
    for i in 0..8 {
        y4[i] = y[i] + h * k3[i];
    }
    let k4 = f(&y4);
    let mut out = [0.0; 8];
    for i in 0..8 {
        out[i] = y[i] + (h / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
    }
    out
}

pub fn pack(w: &Worldline) -> [f64; 8] {
    [w.x[0], w.x[1], w.x[2], w.x[3], w.u[0], w.u[1], w.u[2], w.u[3]]
}

pub fn unpack(y: &[f64; 8]) -> Worldline {
    Worldline {
        x: [y[0], y[1], y[2], y[3]],
        u: [y[4], y[5], y[6], y[7]],
    }
}

/// Uniform boost field: 4-acceleration `a^mu = (alpha/c)(u.e, u^0 e)` for a unit
/// spatial direction `e`. `u.a = 0` identically, so the field is shell-compatible for
/// ANY velocity; the proper-acceleration magnitude equals `alpha` exactly when the
/// velocity is parallel to `e` (the hyperbolic-motion configuration every closed-form
/// gate below uses). This is the boost generator of the Lorentz algebra, not a
/// long-range force: it is the one homogeneous field SR admits without a mediating
/// relation, which is why it is the only non-free segment in this slice's scope.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoostField {
    pub alpha_m_s2: f64,
    /// Unit spatial direction.
    pub dir: [f64; 3],
}

impl BoostField {
    pub fn proper_accel(&self, u: &[f64; 4]) -> [f64; 4] {
        let k = self.alpha_m_s2 / C;
        let ue = u[1] * self.dir[0] + u[2] * self.dir[1] + u[3] * self.dir[2];
        [k * ue, k * u[0] * self.dir[0], k * u[0] * self.dir[1], k * u[0] * self.dir[2]]
    }

    fn derivative(&self, y: &[f64; 8]) -> [f64; 8] {
        let u = [y[4], y[5], y[6], y[7]];
        let a = self.proper_accel(&u);
        [y[4], y[5], y[6], y[7], a[0], a[1], a[2], a[3]]
    }
}

/// Integrate `steps` fixed RK4 steps of size `dtau` under a uniform boost field.
/// `alpha = 0` is exact free flight. dt (here dtau) is PINNED by the caller: every gate
/// below states its dtau and gates the error envelope at that dtau, per section 3.5's
/// trajectory-clause repair.
pub fn integrate_boost_field(
    field: &BoostField,
    start: &Worldline,
    dtau_s: f64,
    steps: u32,
) -> Worldline {
    let mut y = pack(start);
    let f = |y: &[f64; 8]| field.derivative(y);
    let mut i = 0;
    while i < steps {
        y = rk4_step(&f, &y, dtau_s);
        i += 1;
    }
    unpack(&y)
}

// ---------------------------------------------------------------------------------
// Closed forms: hyperbolic motion (constant proper acceleration from rest).
// ---------------------------------------------------------------------------------

/// Hyperbolic motion closed form at proper time tau: start at rest at the origin,
/// proper acceleration `alpha` along +x. `cosh(s)-1` is evaluated as `2 sinh^2(s/2)`
/// so small-rapidity positions keep all their digits.
pub fn hyperbolic_worldline(alpha_m_s2: f64, tau_s: f64) -> Worldline {
    let s = alpha_m_s2 * tau_s / C;
    let half = sinh(0.5 * s);
    let x = (C * C / alpha_m_s2) * 2.0 * half * half;
    let ct = (C * C / alpha_m_s2) * sinh(s);
    Worldline {
        x: [ct, x, 0.0, 0.0],
        u: [C * cosh(s), C * sinh(s), 0.0, 0.0],
    }
}

/// Hyperbolic motion position at COORDINATE time t: `x(t) = (c^2/alpha)(sqrt(1+z^2)-1)`
/// with `z = alpha t / c`, evaluated as `z^2/(sqrt(1+z^2)+1)` (no subtraction).
pub fn hyperbolic_position_at_t(alpha_m_s2: f64, t_s: f64) -> f64 {
    let z = alpha_m_s2 * t_s / C;
    let z2 = z * z;
    (C * C / alpha_m_s2) * z2 / (sqrt(1.0 + z2) + 1.0)
}

/// Coordinate speed beta(t) on the hyperbolic worldline: `z/sqrt(1+z^2)`.
pub fn hyperbolic_beta_at_t(alpha_m_s2: f64, t_s: f64) -> f64 {
    let z = alpha_m_s2 * t_s / C;
    z / sqrt(1.0 + z * z)
}

/// The Newtonian chart's position for the same segment: `a t^2 / 2`.
pub fn newtonian_position_at_t(alpha_m_s2: f64, t_s: f64) -> f64 {
    0.5 * alpha_m_s2 * t_s * t_s
}

/// The certified remainder bound `|x_SR(t) - a t^2/2| <= a^3 t^4 / (8 c^2)`.
///
/// Proof of the bound: with `e = (a t/c)^2`, `x_SR - x_N = (c^2/a)[sqrt(1+e)-1-e/2]`,
/// and `f(e) = 1 + e/2 - sqrt(1+e)` satisfies `0 <= f(e) <= e^2/8` for ALL `e >= 0`
/// (`d/de [e^2/8 - f] = e/4 - 1/2 + 1/(2 sqrt(1+e)) >= 0`, equality at 0). The bound
/// therefore never lies; what fails as beta grows is the CERTIFICATE, which refuses
/// when the bound exceeds the caller's tolerance — that refusal is the gate's teeth
/// and is mutation-tested.
pub fn newton_position_remainder_bound(alpha_m_s2: f64, t_s: f64) -> f64 {
    let a = alpha_m_s2;
    a * a * a * t_s * t_s * t_s * t_s / (8.0 * C * C)
}

// ---------------------------------------------------------------------------------
// Contact collisions: 1D relativistic elastic, closed form.
// ---------------------------------------------------------------------------------

/// Elastic head-on collision of two masses moving along x, in closed form via
/// rapidities: `y = asinh(w/c)` from the celerity is exact and stable; the CM rapidity
/// satisfies `tanh(y_cm) = P c / E`; the 1D elastic outcome in the CM frame reverses
/// both momenta, so the lab-frame outgoing rapidities are the reflection
/// `y' = 2 y_cm - y` (any mass ratio). Inputs and outputs are x-celerities `w = gamma v`
/// in m/s.
///
/// The identity map (no collision) conserves every invariant too — invariants alone
/// cannot reject it (an L3 unfailable-gate trap). The closed form pins the physical
/// exchange branch; the gates check BOTH the invariants and this closed form.
pub fn elastic_collide_1d(m1_kg: f64, w1: f64, m2_kg: f64, w2: f64) -> (f64, f64) {
    let y1 = libm::asinh(w1 / C);
    let y2 = libm::asinh(w2 / C);
    let e_over_c = m1_kg * C * cosh(y1) + m2_kg * C * cosh(y2);
    let px = m1_kg * C * sinh(y1) + m2_kg * C * sinh(y2);
    let y_cm = atanh(px / e_over_c);
    let y1p = 2.0 * y_cm - y1;
    let y2p = 2.0 * y_cm - y2;
    (C * sinh(y1p), C * sinh(y2p))
}

/// CM momentum `p*` from the invariant mass `M` (kg) of the pair, fully factored:
///
/// `p* = c sqrt([M-(m1+m2)][M+(m1+m2)][M-(m1-m2)][M+(m1-m2)]) / (2M)`.
///
/// The factored form is pinned by section 3.5: near threshold, `M - (m1+m2)` is an
/// EXACT f64 subtraction (Sterbenz), so this form extracts every digit `M` carries;
/// the EXPANDED Kallen polynomial
/// `M^4 + m1^4 + m2^4 - 2M^2 m1^2 - 2M^2 m2^2 - 2m1^2 m2^2` cancels through terms of
/// size `M^4` and loses the small factor in their rounding (it can even go negative
/// above threshold, making `p*` NaN). The gate plants the expanded form and shows it
/// fail. When the CALLER's `M` was itself rounded from a real-valued construction,
/// prefer [`cm_momentum_from_excess`]: one ulp of `M` moves p* by tens of percent at
/// the deep threshold, so the excess is the primitive that must cross the interface.
pub fn cm_momentum(m1_kg: f64, m2_kg: f64, m_invariant_kg: f64) -> f64 {
    let m = m_invariant_kg;
    let sum = m1_kg + m2_kg;
    let dif = m1_kg - m2_kg;
    C * sqrt((m - sum) * (m + sum) * (m - dif) * (m + dif)) / (2.0 * m)
}

/// CM momentum with the threshold excess `M - (m1+m2)` (kg) as the primitive input.
/// Near threshold this is the representation that must cross the interface: rounding
/// the excess into a single `M` first loses it at ulp granularity.
pub fn cm_momentum_from_excess(m1_kg: f64, m2_kg: f64, excess_kg: f64) -> f64 {
    let sum = m1_kg + m2_kg;
    let dif = m1_kg - m2_kg;
    let m = sum + excess_kg;
    C * sqrt(excess_kg * (m + sum) * (m - dif) * (m + dif)) / (2.0 * m)
}

// ---------------------------------------------------------------------------------
// Doppler and aberration (cheap closed-form kinematics).
// ---------------------------------------------------------------------------------

/// Relativistic Doppler factor `nu_obs/nu_src = 1/(gamma (1 - beta cos_theta))` for a
/// source receding-at-angle: `theta` is measured in the observer frame between the
/// source velocity and the line of sight. Transverse (`cos_theta = 0`) gives exactly
/// `1/gamma`.
pub fn doppler_factor(beta: f64, cos_theta: f64) -> f64 {
    let gamma = 1.0 / sqrt(1.0 - beta * beta);
    1.0 / (gamma * (1.0 - beta * cos_theta))
}

/// Relativistic aberration: `cos_theta' = (cos_theta - beta)/(1 - beta cos_theta)`.
/// Its own inverse under `beta -> -beta` (gated).
pub fn aberrate_cos(beta: f64, cos_theta: f64) -> f64 {
    (cos_theta - beta) / (1.0 - beta * cos_theta)
}

// ---------------------------------------------------------------------------------
// The SR -> Newton certificate (A4).
// ---------------------------------------------------------------------------------

/// The A4 certificate for one force-free / uniform-boost-field segment of duration `t`
/// at proper acceleration `alpha` from rest.
///
/// Composition: the numeric SR integrator is gated against the hyperbolic closed form
/// separately (trajectory clause); this certificate is the CHART-level statement — the
/// closed-form SR trajectory lies within `a^3 t^4/(8 c^2)` of the Newtonian chart — so
/// the numeric trajectory is within bound + integrator envelope of Newton.
///
/// SCOPE: force-free and uniform-boost-field segments only. It does NOT cover weighted
/// scenes: weight is the curved tier's ([`crate::curvature`]), per the frame decision
/// that gravity is chart data and Gantt A3. The uniform-field weak chart's Newtonian
/// limit is certified THERE, with its own remainder.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NewtonChartCertificate {
    /// `Certified`: the Newtonian chart is licensed within `rel_tolerance`.
    /// `GrainFloor`: the two charts are certifiably indistinguishable at working
    /// precision — the M27 arithmetic grain floor (beta^2 below 2*eps_f64, i.e.
    /// v below ~6.3 m/s, the section 3.5 corrected arithmetic).
    /// `RefinementUnavailable`: the remainder exceeds tolerance; SR required.
    pub status: CertificationStatus,
    /// Coordinate speed at segment end.
    pub beta_end: f64,
    /// Absolute position remainder bound `a^3 t^4/(8c^2)` (m).
    pub position_remainder_bound_m: f64,
    /// The same bound relative to the Newtonian position: `(a t/c)^2 / 4`.
    pub relative_position_remainder: f64,
    /// Relative momentum defect `gamma - 1` at segment end (the section 3.5 momentum
    /// observable, coefficient 1/2 in beta^2).
    pub momentum_defect: f64,
}

/// Certify the Newtonian chart for a rest-start constant-proper-acceleration segment.
/// The certificate can FAIL (refuse) — raising beta until it does is the mutation test.
pub fn certify_newton_chart(
    alpha_m_s2: f64,
    duration_s: f64,
    rel_tolerance: f64,
) -> NewtonChartCertificate {
    let z = alpha_m_s2 * duration_s / C;
    let z2 = z * z;
    let beta_end = if z == 0.0 { 0.0 } else { z / sqrt(1.0 + z2) };
    let gamma_minus_one = z2 / (sqrt(1.0 + z2) + 1.0);
    let cert = NewtonChartCertificate {
        status: CertificationStatus::RefinementUnavailable,
        beta_end,
        position_remainder_bound_m: newton_position_remainder_bound(alpha_m_s2, duration_s),
        relative_position_remainder: z2 / 4.0,
        momentum_defect: gamma_minus_one,
    };
    // The corrected beta-floor arithmetic (section 3.5): beta^2 < 2*eps means the
    // relativistic correction is below one ulp of unity — the charts are certifiably
    // indistinguishable at working precision. v < ~6.3 m/s, NOT 6 km/s.
    let beta2 = beta_end * beta_end;
    if beta2 < 2.0 * f64::EPSILON {
        return NewtonChartCertificate { status: CertificationStatus::GrainFloor, ..cert };
    }
    // The momentum defect (z^2/2) is always the binding one vs position (z^2/4).
    if gamma_minus_one <= rel_tolerance && cert.relative_position_remainder <= rel_tolerance {
        return NewtonChartCertificate { status: CertificationStatus::Certified, ..cert };
    }
    cert
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every gate here pins its dtau and states its pre-staked band BEFORE the measured
    // number; each has a mutation partner that must FAIL the same assertion, because a
    // gate that cannot fail proves nothing.

    /// Planted lower-order integrator (explicit Euler) — the trajectory-clause control:
    /// at the pinned dtau it must fail the envelope RK4 passes.
    fn euler_step<F: Fn(&[f64; 8]) -> [f64; 8]>(f: &F, y: &[f64; 8], h: f64) -> [f64; 8] {
        let k = f(y);
        let mut out = [0.0; 8];
        for i in 0..8 {
            out[i] = y[i] + h * k[i];
        }
        out
    }

    fn rest_at_origin() -> Worldline {
        Worldline::from_celerity([0.0; 4], [0.0; 3])
    }

    const PLUS_X: [f64; 3] = [1.0, 0.0, 0.0];

    // -------------------------------------------------------------------------
    // Gate 1: hyperbolic motion — trajectory clause at pinned dtau, then the
    // convergence SLOPE AND COEFFICIENT staked against the derived RK4 constant.
    // -------------------------------------------------------------------------

    /// Relative error of the numerically integrated u^1 against the closed form after
    /// total proper time `tau`, at fixed dtau.
    fn hyperbolic_u1_error(alpha: f64, tau: f64, steps: u32) -> f64 {
        let field = BoostField { alpha_m_s2: alpha, dir: PLUS_X };
        let dtau = tau / steps as f64;
        let end = integrate_boost_field(&field, &rest_at_origin(), dtau, steps);
        let exact = hyperbolic_worldline(alpha, tau);
        libm::fabs(end.u[1] - exact.u[1]) / exact.u[1]
    }

    #[test]
    fn hyperbolic_trajectory_clause_at_pinned_dtau() {
        // Pinned: alpha = c (so alpha/c = 1 s^-1), total rapidity 2, dtau = 2e-3 s,
        // 1000 steps. Predicted RK4 truncation ~ tau (alpha/c)(dtau alpha/c)^4/120
        // = 2.7e-13 relative; rounding floor ~ n*eps ~ 2e-13. Gate (section 3.5):
        // max(|dx|/L_ref, |du|/u_ref) < 1e-9 at the pinned dtau.
        let alpha = C;
        let tau = 2.0;
        let steps = 1000_u32;
        let dtau = tau / steps as f64;
        let field = BoostField { alpha_m_s2: alpha, dir: PLUS_X };

        let mut y = pack(&rest_at_origin());
        let f = |y: &[f64; 8]| field.derivative(y);
        let exact_end = hyperbolic_worldline(alpha, tau);
        let l_ref = exact_end.x[1];
        let u_ref = exact_end.u[1];
        let mut worst = 0.0_f64;
        for i in 1..=steps {
            y = rk4_step(&f, &y, dtau);
            let ex = hyperbolic_worldline(alpha, dtau * i as f64);
            let dx = libm::fabs(y[1] - ex.x[1]) / l_ref;
            let du = libm::fabs(y[5] - ex.u[1]) / u_ref;
            worst = worst.max(dx).max(du);
        }
        assert!(worst < 1.0e-9, "trajectory clause failed: {worst:e}");

        // Planted-error control: Euler at the SAME pinned dtau must fail the same
        // envelope (predicted ~ tau(alpha/c)(dtau alpha/c)/2 = 2e-3 >> 1e-9). Without
        // this control the clause is passable by shrinking dtau cost-free.
        let mut y = pack(&rest_at_origin());
        let mut worst_euler = 0.0_f64;
        for i in 1..=steps {
            y = euler_step(&f, &y, dtau);
            let ex = hyperbolic_worldline(alpha, dtau * i as f64);
            worst_euler = worst_euler.max(libm::fabs(y[5] - ex.u[1]) / u_ref);
        }
        assert!(
            worst_euler > 1.0e-9,
            "planted Euler PASSED the envelope ({worst_euler:e}); the gate has no teeth"
        );
    }

    #[test]
    fn hyperbolic_convergence_coefficient_staked_not_just_slope() {
        // The ODE is linear (u' = (alpha/c) K u, K the boost generator, eigenvalues
        // +-1), so the RK4 global relative error on the growing mode has a DERIVED
        // closed-form coefficient: err(h) = tau (alpha/c)^5 h^4 / 120 at leading
        // order. Pre-staked BEFORE running: slope 4.00 +- 0.05 over z = h*alpha/c in
        // {0.01, 0.02, 0.04}; coefficient within +-10% of tau(alpha/c)^5/120 at the
        // smallest h (derived corrections: +3.7% decaying-mode, -5z/6 next order).
        // A wrong-tableau RK4 with error ~h^3 passes a loose envelope but fails THIS
        // stake — which is exactly why section 3.5 demands the coefficient.
        let alpha = C;
        let tau = 2.0;
        let e1 = hyperbolic_u1_error(alpha, tau, 200); // h = 0.01
        let e2 = hyperbolic_u1_error(alpha, tau, 100); // h = 0.02
        let e3 = hyperbolic_u1_error(alpha, tau, 50); // h = 0.04

        let slope_a = libm::log(e2 / e1) / libm::log(2.0);
        let slope_b = libm::log(e3 / e2) / libm::log(2.0);
        assert!(
            libm::fabs(slope_a - 4.0) < 0.05 && libm::fabs(slope_b - 4.0) < 0.05,
            "convergence slope off: {slope_a} {slope_b}"
        );

        let h = 0.01;
        let staked = tau * libm::pow(alpha / C, 5.0) * h * h * h * h / 120.0;
        let ratio = e1 / staked;
        assert!(
            (0.9..1.1).contains(&ratio),
            "convergence coefficient off staked 1/120 constant: ratio {ratio}"
        );

        // Mutation: a defective integrator whose error is O(h^3) (one wrong RK4 weight)
        // must fail the slope stake.
        let bad_rk4 = |f: &dyn Fn(&[f64; 8]) -> [f64; 8], y: &[f64; 8], h: f64| {
            let k1 = f(y);
            let mut y2 = [0.0; 8];
            for i in 0..8 {
                y2[i] = y[i] + 0.5 * h * k1[i];
            }
            let k2 = f(&y2);
            let mut y3 = [0.0; 8];
            for i in 0..8 {
                y3[i] = y[i] + 0.5 * h * k2[i];
            }
            let k3 = f(&y3);
            let mut y4 = [0.0; 8];
            for i in 0..8 {
                y4[i] = y[i] + h * k3[i];
            }
            let k4 = f(&y4);
            let mut out = [0.0; 8];
            for i in 0..8 {
                // 1/5 instead of 1/6 on the k1 weight: local error becomes O(h^2).
                out[i] = y[i] + h * (k1[i] / 5.0 + k2[i] / 3.0 + k3[i] / 3.0 + k4[i] / 6.0);
            }
            out
        };
        let field = BoostField { alpha_m_s2: alpha, dir: PLUS_X };
        let f = |y: &[f64; 8]| field.derivative(y);
        let bad_err = |steps: u32| {
            let dtau = tau / steps as f64;
            let mut y = pack(&rest_at_origin());
            for _ in 0..steps {
                y = bad_rk4(&f, &y, dtau);
            }
            let exact = hyperbolic_worldline(alpha, tau);
            libm::fabs(y[5] - exact.u[1]) / exact.u[1]
        };
        let b1 = bad_err(200);
        let b2 = bad_err(100);
        let bad_slope = libm::log(b2 / b1) / libm::log(2.0);
        assert!(
            libm::fabs(bad_slope - 4.0) > 0.05,
            "planted wrong-weight RK4 passed the slope stake ({bad_slope})"
        );
    }

    // -------------------------------------------------------------------------
    // Gate 2: the mass shell as a certificate — and the demonstration that the
    // derived-u0 representation makes it an unfailable identity (section 3.5).
    // -------------------------------------------------------------------------

    #[test]
    fn shell_identity_on_a_wrecked_trajectory_vs_shell_certificate() {
        let alpha = C;
        let tau = 2.0;
        let steps = 1000_u32;
        let dtau = tau / steps as f64;
        let field = BoostField { alpha_m_s2: alpha, dir: PLUS_X };
        let f = |y: &[f64; 8]| field.derivative(y);

        // Independently evolved u0: the residual is a genuine certificate. Gate at
        // 1e-12 at the pinned dtau (measured drift is truncation + rounding).
        let end = integrate_boost_field(&field, &rest_at_origin(), dtau, steps);
        assert!(
            libm::fabs(end.shell_residual()) < 1.0e-12,
            "shell certificate failed: {:e}",
            end.shell_residual()
        );

        // Mutation A: drop the time component of the 4-acceleration (the dropped-gamma
        // defect). The trajectory is wrong AND the certificate fires.
        let f_dropped = |y: &[f64; 8]| {
            let mut d = f(y);
            d[4] = 0.0; // du0/dtau planted to zero
            d
        };
        let mut y = pack(&rest_at_origin());
        for _ in 0..steps {
            y = rk4_step(&f_dropped, &y, dtau);
        }
        let wrecked = unpack(&y);
        assert!(
            libm::fabs(wrecked.shell_residual()) > 1.0e-3,
            "dropped-gamma defect did not fire the shell certificate"
        );

        // Mutation B — the section 3.5 warning made executable: recompute u0 from the
        // shell on that SAME wrecked trajectory and the residual is ~0 while the
        // trajectory error is enormous. An algebraic identity is not a gate.
        let repaired = Worldline::from_celerity(wrecked.x, [wrecked.u[1], wrecked.u[2], wrecked.u[3]]);
        let exact = hyperbolic_worldline(alpha, tau);
        let trajectory_error = libm::fabs(repaired.x[1] - exact.x[1]) / exact.x[1];
        assert!(
            libm::fabs(repaired.shell_residual()) < 1.0e-14 && trajectory_error > 1.0e-2,
            "the derived-u0 representation should satisfy the shell on a wrong \
             trajectory: residual {:e}, trajectory error {:e}",
            repaired.shell_residual(),
            trajectory_error
        );
    }

    // -------------------------------------------------------------------------
    // Gate 3: elastic collision — invariants AND closed form (either alone is
    // blind to a defect class), plus s+t+u.
    // -------------------------------------------------------------------------

    /// Numeric cross-check route: boost to CM, reflect, boost back.
    fn collide_via_boosts(m1: f64, w1: f64, m2: f64, w2: f64) -> (f64, f64) {
        let a = Worldline::from_celerity([0.0; 4], [w1, 0.0, 0.0]);
        let b = Worldline::from_celerity([0.0; 4], [w2, 0.0, 0.0]);
        let p1 = FourMomentum::of(m1, &a);
        let p2 = FourMomentum::of(m2, &b);
        let tot = p1.add(p2);
        let beta_cm = [tot.p[1] / tot.p[0], 0.0, 0.0];
        let q1 = boost(&p1.p, &beta_cm);
        let q2 = boost(&p2.p, &beta_cm);
        let r1 = [q1[0], -q1[1], q1[2], q1[3]];
        let r2 = [q2[0], -q2[1], q2[2], q2[3]];
        let neg = [-beta_cm[0], 0.0, 0.0];
        let o1 = boost(&r1, &neg);
        let o2 = boost(&r2, &neg);
        (o1[1] / m1, o2[1] / m2)
    }

    #[test]
    fn elastic_collision_invariants_and_closed_form() {
        let (m1, m2) = (1.0, 3.0);
        let (w1, w2) = (0.8 * C, -0.3 * C);
        let a_in = Worldline::from_celerity([0.0; 4], [w1, 0.0, 0.0]);
        let b_in = Worldline::from_celerity([0.0; 4], [w2, 0.0, 0.0]);
        let p1 = FourMomentum::of(m1, &a_in);
        let p2 = FourMomentum::of(m2, &b_in);
        let before = p1.add(p2);
        let s_before = before.dot_self();

        let (w1p, w2p) = elastic_collide_1d(m1, w1, m2, w2);
        let a_out = Worldline::from_celerity([0.0; 4], [w1p, 0.0, 0.0]);
        let b_out = Worldline::from_celerity([0.0; 4], [w2p, 0.0, 0.0]);
        let q1 = FourMomentum::of(m1, &a_out);
        let q2 = FourMomentum::of(m2, &b_out);
        let after = q1.add(q2);

        // PDG-style invariants: s and every P^mu component conserved.
        let s_rel = libm::fabs(after.dot_self() - s_before) / s_before;
        assert!(s_rel < 1.0e-12, "s not conserved: {s_rel:e}");
        for k in 0..4 {
            let rel = libm::fabs(after.p[k] - before.p[k]) / libm::fabs(before.p[0]);
            assert!(rel < 1.0e-12, "P^{k} not conserved: {rel:e}");
        }

        // s + t + u = (2 m1^2 + 2 m2^2) c^2 on the collision event.
        let d1 = [p1.p[0] - q1.p[0], p1.p[1] - q1.p[1], p1.p[2] - q1.p[2], p1.p[3] - q1.p[3]];
        let d2 = [p1.p[0] - q2.p[0], p1.p[1] - q2.p[1], p1.p[2] - q2.p[2], p1.p[3] - q2.p[3]];
        let t_mand = minkowski_dot(&d1, &d1);
        let u_mand = minkowski_dot(&d2, &d2);
        let stu = s_before + t_mand + u_mand;
        let expected = 2.0 * (m1 * m1 + m2 * m2) * C * C;
        assert!(
            libm::fabs(stu - expected) / expected < 1.0e-9,
            "s+t+u identity: {stu} vs {expected}"
        );

        // Two independent routes agree.
        let (v1p, v2p) = collide_via_boosts(m1, w1, m2, w2);
        assert!(libm::fabs(v1p - w1p) / C < 1.0e-12 && libm::fabs(v2p - w2p) / C < 1.0e-12);

        // Mutation A: the identity map (planted "no exchange") conserves s and P too —
        // the invariants gate is provably blind to it — but fails the closed form.
        // This is why the gate is invariants AND closed form.
        assert!(libm::fabs(w1 - w1p) / C > 1.0e-3, "collision was a no-op");

        // Mutation B: dropped gamma in the boost route (a wrong-but-linear boost)
        // breaks the p.p invariance certificate.
        let bad_boost = |v: &[f64; 4], beta: &[f64; 3]| {
            // gamma planted to 1 on the time row.
            let bp = beta[0] * v[1] + beta[1] * v[2] + beta[2] * v[3];
            [v[0] - bp, v[1] - beta[0] * v[0], v[2], v[3]]
        };
        let q = bad_boost(&p1.p, &[0.5, 0.0, 0.0]);
        let inv_defect = libm::fabs(minkowski_dot(&q, &q) - p1.dot_self()) / p1.dot_self();
        assert!(inv_defect > 1.0e-3, "dropped-gamma boost preserved p.p ({inv_defect:e})");
    }

    /// Stable threshold excess for a CM momentum q: sum of
    /// `d_i = sqrt((mi c)^2 + q^2) - mi c`, each computed as `q^2/(sqrt(..) + mi c)` —
    /// no subtraction anywhere. This is the primitive the near-threshold API needs.
    fn threshold_excess(m1: f64, m2: f64, q: f64) -> f64 {
        let d1 = q * q / (sqrt(m1 * C * m1 * C + q * q) + m1 * C);
        let d2 = q * q / (sqrt(m2 * C * m2 * C + q * q) + m2 * C);
        (d1 + d2) / C
    }

    /// The planted defect section 3.5 names: the expanded Kallen polynomial.
    fn cm_momentum_expanded_kallen(m1: f64, m2: f64, m_inv: f64) -> f64 {
        let lambda = m_inv * m_inv * m_inv * m_inv + m1 * m1 * m1 * m1 + m2 * m2 * m2 * m2
            - 2.0 * m_inv * m_inv * m1 * m1
            - 2.0 * m_inv * m_inv * m2 * m2
            - 2.0 * m1 * m1 * m2 * m2;
        if lambda >= 0.0 { C * sqrt(lambda) / (2.0 * m_inv) } else { f64::NAN }
    }

    #[test]
    fn cm_momentum_factorization_survives_threshold() {
        // Proton + pion masses (kg-scaled). The first run of this gate falsified the
        // lazy version of the claim: at q = 1e-7*m2*c the expanded polynomial happened
        // to land within 7.4e-4 of the truth — a fixed input buys one lucky rounding.
        // The version below is deterministic: it probes an exact ulp-spaced grid of M
        // just above threshold, where Sterbenz makes the factored form's small factor
        // M-(m1+m2) EXACT, and the expanded polynomial's cancellation error is O(1) of
        // the true Kallen value at EVERY small k, not just one.
        let (m1, m2) = (0.938, 0.139);

        // Moderate depth: q = 1e-5 * m2 * c. Excess route exact; factored-from-M
        // inside 1e-3 of the true q (the excess sits ~360 ulps up, M carries it).
        let q = 1.0e-5 * m2 * C;
        let excess = threshold_excess(m1, m2, q);
        let p_excess = cm_momentum_from_excess(m1, m2, excess);
        assert!(
            libm::fabs(p_excess - q) / q < 1.0e-12,
            "excess-primitive p* lost digits: {:e}",
            libm::fabs(p_excess - q) / q
        );
        let p_factored = cm_momentum(m1, m2, m1 + m2 + excess);
        let factored_rel = libm::fabs(p_factored - q) / q;
        assert!(factored_rel < 1.0e-3, "factored p* off at moderate depth: {factored_rel:e}");

        // Deep threshold, exact ulp grid: thr = fl(m1+m2) is in [1,2), so its ulp is
        // exactly f64::EPSILON and M_k = thr + k*EPSILON is exact for small k, with
        // excess_k = k*EPSILON recovered EXACTLY by the factored subtraction
        // (Sterbenz). Reference p* comes from the excess API on that exact excess.
        let thr = m1 + m2;
        let mut expanded_worst = 0.0_f64;
        for k in 1..=5_u32 {
            let excess_k = k as f64 * f64::EPSILON;
            let m_k = thr + excess_k;
            let reference = cm_momentum_from_excess(m1, m2, excess_k);
            let factored_err =
                libm::fabs(cm_momentum(m1, m2, m_k) - reference) / reference;
            assert!(
                factored_err < 1.0e-6,
                "factored p* lost the exact ulp excess at k={k}: {factored_err:e}"
            );
            let p_exp = cm_momentum_expanded_kallen(m1, m2, m_k);
            let exp_err = if p_exp.is_finite() {
                libm::fabs(p_exp - reference) / reference
            } else {
                1.0 // went negative above threshold: NaN, the catastrophic face
            };
            expanded_worst = expanded_worst.max(exp_err);
        }
        assert!(
            expanded_worst > 0.02,
            "expanded Kallen polynomial tracked the factored form on the ulp grid \
             (worst {expanded_worst:e}); the pin is dead"
        );

        // The upstream-rounding half of the pin, measured: one ulp of M (k=1 -> k=2)
        // moves p* by sqrt(2)-1 = 41% at the deep threshold, so an interface that
        // rounds the excess into a single M before the call has already lost the
        // answer. The excess API is the repair.
        let p1 = cm_momentum(m1, m2, thr + f64::EPSILON);
        let p2 = cm_momentum(m1, m2, thr + 2.0 * f64::EPSILON);
        assert!(
            libm::fabs(p2 / p1 - libm::sqrt(2.0)) < 1.0e-6,
            "one-ulp sensitivity should be exactly sqrt(2): {}",
            p2 / p1
        );
    }

    // -------------------------------------------------------------------------
    // Gate 4: the ledger commutes with boosts — linearity AND invariance, with
    // the defect class each one is blind to made explicit.
    // -------------------------------------------------------------------------

    #[test]
    fn ledger_commutes_with_boosts() {
        let entries = [
            FourMomentum::of(1.0, &Worldline::from_celerity([0.0; 4], [0.3 * C, -0.1 * C, 0.2 * C])),
            FourMomentum::of(2.5, &Worldline::from_celerity([0.0; 4], [-0.6 * C, 0.4 * C, 0.0])),
            FourMomentum::of(0.5, &Worldline::from_celerity([0.0; 4], [0.0, 0.0, 0.9 * C])),
        ];
        let beta = [0.5, -0.2, 0.1];
        let total = total_momentum(&entries);
        let boosted_total = boost(&total.p, &beta);
        let mut total_of_boosted = [0.0; 4];
        for e in &entries {
            let b = boost(&e.p, &beta);
            for k in 0..4 {
                total_of_boosted[k] += b[k];
            }
        }
        for k in 0..4 {
            let rel = libm::fabs(boosted_total[k] - total_of_boosted[k])
                / libm::fabs(boosted_total[0]);
            assert!(rel < 1.0e-13, "Lambda(sum p) != sum Lambda p at component {k}: {rel:e}");
        }
        // Invariance of p.p per entry — the certificate a wrong-but-linear boost fails
        // (linearity alone cannot catch it; see the collision gate's Mutation B).
        for e in &entries {
            let b = boost(&e.p, &beta);
            let rel = libm::fabs(minkowski_dot(&b, &b) - e.dot_self()) / libm::fabs(e.dot_self());
            assert!(rel < 1.0e-12, "boost broke p.p: {rel:e}");
        }
    }

    #[test]
    fn doppler_and_aberration_closed_forms() {
        let beta = 0.6;
        let gamma = 1.0 / sqrt(1.0 - beta * beta);
        // Transverse Doppler is exactly 1/gamma.
        assert!(libm::fabs(doppler_factor(beta, 0.0) - 1.0 / gamma) < 1.0e-15);
        // Aberration round-trips under beta -> -beta.
        let c0 = 0.3;
        let there = aberrate_cos(beta, c0);
        let back = aberrate_cos(-beta, there);
        assert!(libm::fabs(back - c0) < 1.0e-15, "aberration round-trip: {back}");
        // Aberration preserves null directions exactly: light stays light.
        assert!(aberrate_cos(beta, 1.0) == 1.0 && aberrate_cos(beta, -1.0) == -1.0);
        // Collinear boosts compose by rapidity addition.
        let (b1, b2) = (0.5, 0.7);
        let composed = (b1 + b2) / (1.0 + b1 * b2);
        assert!(libm::fabs(libm::atanh(b1) + libm::atanh(b2) - libm::atanh(composed)) < 1.0e-15);
        // Mutation, with the measured lesson kept: a sign-defective aberration formula
        // is STILL a Mobius map whose beta -> -beta partner inverts it, so it PASSES
        // the round-trip — the round-trip alone is an unfailable gate for this entire
        // defect class (L3). The null fixed point is the gate with teeth: the defect
        // maps cos=1 to (1-beta)/(1+beta) != 1.
        let bad = |beta: f64, c: f64| (c + beta) / (1.0 - beta * c);
        let bad_back = bad(-beta, bad(beta, c0));
        assert!(
            libm::fabs(bad_back - c0) < 1.0e-12,
            "expected the defective Mobius map to round-trip (it inverts itself)"
        );
        assert!(
            libm::fabs(bad(beta, -1.0) - (-1.0)) > 1.0e-3,
            "planted aberration defect passed the null-fixed-point gate"
        );
    }

    // -------------------------------------------------------------------------
    // Gate 5 (A4): the SR -> Newton certificate — coefficient, tightness, teeth,
    // the arithmetic grain floor, and the numeric composition.
    // -------------------------------------------------------------------------

    #[test]
    fn newton_remainder_coefficient_and_slope_staked() {
        // Pre-staked: |x_SR - x_N| / t^4 -> a^3/(8 c^2), within 1% at z = 1e-3;
        // log-log slope in t of the remainder = 4.00 +- 0.05; tightness ratio
        // measured/bound in [0.9, 1.0] for z in [1e-3, 0.3] (derived: ratio ~ 1 - e/2).
        let alpha = 10.0;
        let t_at = |z: f64| z * C / alpha;

        let rem = |t: f64| {
            libm::fabs(hyperbolic_position_at_t(alpha, t) - newtonian_position_at_t(alpha, t))
        };
        let t1 = t_at(1.0e-3);
        let coeff = rem(t1) / (t1 * t1 * t1 * t1);
        let staked = alpha * alpha * alpha / (8.0 * C * C);
        assert!(
            libm::fabs(coeff / staked - 1.0) < 0.01,
            "remainder coefficient off a^3/(8c^2): ratio {}",
            coeff / staked
        );
        let t2 = t_at(2.0e-3);
        let slope = libm::log(rem(t2) / rem(t1)) / libm::log(2.0);
        assert!(libm::fabs(slope - 4.0) < 0.05, "remainder slope: {slope}");

        for z in [1.0e-3, 1.0e-2, 0.1, 0.3] {
            let t = t_at(z);
            let ratio = rem(t) / newton_position_remainder_bound(alpha, t);
            assert!(
                ratio <= 1.0 && ratio > 0.9,
                "bound not tight or violated at z={z}: ratio {ratio}"
            );
        }

        // Mutation: a planted WEAKENED bound (coefficient 1/80 instead of 1/8) is
        // exceeded by the measured remainder — the comparison has teeth.
        let t = t_at(0.1);
        let planted = newton_position_remainder_bound(alpha, t) / 10.0;
        assert!(rem(t) > planted, "planted weakened bound was not exceeded");
    }

    #[test]
    fn newton_momentum_defect_slope_and_coefficient() {
        // Section 3.5 limit-order clause: log-log slope of (p_SR - p_N)/p_N = gamma-1
        // over beta in [1e-3, 0.3] staked at 2.000 +- 0.05, coefficient staked at 1/2.
        // gamma-1 via the cancellation-safe form.
        let gm1 = |beta: f64| {
            let w = beta / sqrt(1.0 - beta * beta) * C; // celerity
            Worldline::from_celerity([0.0; 4], [w, 0.0, 0.0]).gamma_minus_one()
        };
        let (b_lo, b_hi) = (1.0e-3, 0.3);
        let slope = libm::log(gm1(b_hi) / gm1(b_lo)) / libm::log(b_hi / b_lo);
        assert!(libm::fabs(slope - 2.0) < 0.05, "momentum defect slope: {slope}");
        let coeff = gm1(1.0e-3) / (1.0e-3 * 1.0e-3);
        assert!(
            (0.499..0.501).contains(&coeff),
            "momentum defect coefficient off 1/2: {coeff}"
        );
        // Control: the naive subtraction loses the coefficient at beta = 1e-8 while
        // the safe form holds it — the reason the representation is pinned.
        let beta = 1.0e-8;
        let naive = 1.0 / sqrt(1.0 - beta * beta) - 1.0;
        let safe = gm1(beta) / (beta * beta);
        assert!((0.499..0.501).contains(&safe));
        assert!(
            libm::fabs(naive / (beta * beta) - 0.5) > 0.05,
            "naive gamma-1 unexpectedly kept its digits at beta=1e-8"
        );
    }

    #[test]
    fn newton_certificate_numeric_composition_at_low_beta() {
        // At beta_end ~ 1e-4 the NUMERIC SR trajectory must agree with the Newtonian
        // chart within the certified bound plus a STATED rounding envelope. The
        // envelope is not slack to hide behind: the first run of this gate measured
        // that the a^3 t^4/(8c^2) bound is tight to 1 part in ~1e12 at these betas
        // (ratio = 1 - (a t/c)^2/2), so accumulated f64 rounding (~2e-10 relative
        // here) violates the bare bound. The corridor is therefore
        // bound + 100*eps*|x| at the pinned dtau, and the planted-defect control
        // below exits it by many orders, so the corridor still has teeth.
        let alpha = 10.0;
        let z_end = 1.0e-4;
        let tau_end = (C / alpha) * libm::asinh(z_end);
        let steps = 2000_u32;
        let dtau = tau_end / steps as f64;
        let field = BoostField { alpha_m_s2: alpha, dir: PLUS_X };
        let f = |y: &[f64; 8]| field.derivative(y);
        let mut y = pack(&rest_at_origin());
        for _ in 0..steps {
            y = rk4_step(&f, &y, dtau);
            let t_i = y[0] / C;
            let newton = newtonian_position_at_t(alpha, t_i);
            let bound = newton_position_remainder_bound(alpha, t_i);
            let rounding = 100.0 * f64::EPSILON * libm::fabs(y[1]);
            assert!(
                libm::fabs(y[1] - newton) <= bound + rounding,
                "numeric SR left the certified Newton corridor at t={t_i}"
            );
        }

        // Planted-defect control: a Newtonian chart with the coefficient wrong by one
        // part in 1e5 (0.500005 a t^2) exits the corridor immediately — the corridor
        // is a real gate, not an envelope wide enough to pass anything.
        let mut y = pack(&rest_at_origin());
        let mut defect_caught = false;
        for _ in 0..steps {
            y = rk4_step(&f, &y, dtau);
            let t_i = y[0] / C;
            let planted = 0.500005 * alpha * t_i * t_i;
            let bound = newton_position_remainder_bound(alpha, t_i);
            let rounding = 100.0 * f64::EPSILON * libm::fabs(y[1]);
            if libm::fabs(y[1] - planted) > bound + rounding {
                defect_caught = true;
                break;
            }
        }
        assert!(defect_caught, "planted wrong-coefficient Newton chart stayed in corridor");
    }

    #[test]
    fn newton_certificate_status_teeth_and_grain_floor() {
        let tol = 1.0e-8;
        let alpha = 10.0;
        let t_for = |beta: f64| beta * C / alpha; // z ~ beta at small beta

        // Certified at beta = 1e-4 (defect 5e-9 <= 1e-8).
        let cert = certify_newton_chart(alpha, t_for(1.0e-4), tol);
        assert_eq!(cert.status, CertificationStatus::Certified);
        assert!(libm::fabs(cert.momentum_defect / 5.0e-9 - 1.0) < 0.01);

        // REFUSES at beta = 1e-3 (defect 5e-7 > 1e-8): the certificate's teeth —
        // raising beta until it fails is the mutation test the task demands.
        let refused = certify_newton_chart(alpha, t_for(1.0e-3), tol);
        assert_eq!(refused.status, CertificationStatus::RefinementUnavailable);

        // The corrected beta-floor arithmetic: v = 1 m/s is BELOW the ~6.3 m/s
        // arithmetic floor (GrainFloor — M27: charts indistinguishable at working
        // precision); v = 100 m/s is above it and certifies normally.
        let floor = certify_newton_chart(alpha, 1.0 / alpha, tol); // v_end = 1 m/s
        assert_eq!(floor.status, CertificationStatus::GrainFloor);
        let above = certify_newton_chart(alpha, 100.0 / alpha, tol); // v_end = 100 m/s
        assert_eq!(above.status, CertificationStatus::Certified);
    }

    #[test]
    fn replay_is_bit_identical() {
        let field = BoostField { alpha_m_s2: C, dir: PLUS_X };
        let a = integrate_boost_field(&field, &rest_at_origin(), 1.0e-3, 500);
        let b = integrate_boost_field(&field, &rest_at_origin(), 1.0e-3, 500);
        for k in 0..4 {
            assert!(a.x[k].to_bits() == b.x[k].to_bits() && a.u[k].to_bits() == b.u[k].to_bits());
        }
    }
}
