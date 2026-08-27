//! The first curvature slice: weak-field static metric charts and geodesic motion.
//!
//! ## Where the equivalence principle enters — ONCE, as a premise, cited
//!
//! This module is the single place the equivalence principle enters the engine, and it
//! enters as the METRIC-THEORY PREMISE, not as a theorem: free fall is geodesic motion
//! in the chart
//!
//! ```text
//! ds^2 = (1 + 2 Phi/c^2) c^2 dt^2  -  (1 - 2 Phi/c^2) (dx^2 + dy^2 + dz^2)
//! ```
//!
//! (signature (+,-,-,-), matching the T5 chart descriptor; the central chart may carry
//! the second-order `2 beta_ppn (Phi/c^2)^2` term in `g_00` — see [`CentralChart`]).
//! The premise is a GR axiom, not an SR theorem — exactly as DESCRIPTOR_CHAIN.md
//! section 3.5 requires it to be imported (Einstein 1907, the uniform-acceleration
//! equivalence; Will, Living Rev. Relativity 17 (2014) 4, for the modern EEP statement
//! and the PPN bookkeeping). EVERYTHING downstream is derived, not assumed:
//!
//! * the Newtonian limit — geodesics in the uniform chart reproduce `-g zhat` with a
//!   remainder whose leading coefficients are STAKED and measured (2 on `Phi/c^2`,
//!   1/3 on the `g^3 t^4/c^2` term), not just bounded;
//! * universality of free fall — the geodesic equation below contains no mass
//!   parameter AT ALL, so two bodies of any masses follow bit-identical geodesics;
//!   this REPLACES the demo's bolted-on per-body universality check at the foundation
//!   level (the gate is still run, and still mutation-tested with a planted
//!   mass-coupled fifth force);
//! * gravitational time dilation — the thrown-clock closed form
//!   `(tau - T)/T = v0^2/(6 c^2)` (the gravitational term `+g<z>/c^2` minus the SR
//!   term `-<v^2>/(2c^2)` integrated over the ballistic arc);
//! * perihelion precession — `6 pi GM/(a (1-e^2) c^2)` at `beta_ppn = 1`, and the
//!   first-order truncation `beta_ppn = 0` is a planted defect with a PREDICTED
//!   failure signature (ratio 4/3: the precession observable reads `g_00` at second
//!   order, `(2 + 2 gamma_ppn - beta_ppn)/3`).
//!
//! ## The bridge to the demo (the P1 closure statement)
//!
//! `holon-ball-game`'s `CHART_GRAVITY_M_S2 = 1.8` IS the Newtonian limit of
//! [`UniformChart`] `{ g: 1.8 }`: what was a bolted-on chart constant with the
//! equivalence principle entering by citation only is now the certified limit of a
//! geodesic derivation, with the remainder measured here. The game is deliberately NOT
//! modified in this slice; the wiring belongs to the postponed game slice.
//!
//! ## Christoffel symbols, hand-derived (no symbolic machinery)
//!
//! For the static diagonal metric `g_00 = A(x)`, `g_ij = -B(x) delta_ij` in
//! coordinates `x^0 = ct`:
//!
//! ```text
//! Gamma^0_{0i} = A_,i / (2A)
//! Gamma^i_{00} = A_,i / (2B)
//! Gamma^i_{jk} = [delta_ik B_,j + delta_ij B_,k - delta_jk B_,i] / (2B)
//! ```
//!
//! all others zero (static: no d/dt; diagonal: no mixed terms). Derivation of the
//! spatial symbol from `Gamma^i_{jk} = (1/2) g^{ii'} (g_{i'j,k} + g_{i'k,j} - g_{jk,i'})`
//! with `g^{ii} = -1/B`: each `g_{i'j,k} = -B_,k delta_i'j`, so the bracket is
//! `-(B_,k delta_ij + B_,j delta_ik - B_,i delta_jk)` and the `-1/B` flips the sign.
//! The geodesic equation `du^mu/dtau = -Gamma^mu_{ab} u^a u^b` then reads
//!
//! ```text
//! du^0/dtau = -(u^0/A)  (gradA . u_sp)
//! du^i/dtau = -A_,i (u^0)^2/(2B) - [2 u^i (gradB . u_sp) - B_,i |u_sp|^2] / (2B)
//! ```
//!
//! With `A = 1 + 2 Phi/c^2` (+ optional second order) and `B = 1 - 2 Phi/c^2`, the
//! slow-motion coordinate acceleration works out to
//! `zddot = -g(1 + 2 Phi/c^2) + 3 g beta_z^2 + O(2nd order)` in the uniform chart —
//! the `2` from `1/B` (one part space curvature), the `3` as `1 (Gamma^z_zz) +
//! 2 (Gamma^0_{0z} feedback)`. Both coefficients are staked and measured in the gates,
//! and the planted `B == 1` defect (space curvature dropped) shifts them to 0 and 2
//! with predicted signatures.
//!
//! ## Scope and misfits (recorded, not routed around)
//!
//! Free fall only: geodesics of a FIXED external metric. Self-gravity (mass sourcing
//! Phi) stays open under Gantt A3's field-equation half; nothing here sources the
//! metric. Energy bookkeeping: [`energy_per_mass`] is conserved along a geodesic
//! because THIS chart is static (a timelike Killing vector) — it is chart state, not a
//! chart-free total; summing [`crate::relativity::FourMomentum`] entries across
//! separated events in a curved chart is foliation-relative (the M15/L8 landmine) and
//! is deliberately NOT built here.

use crate::relativity::{rk4_step, Worldline, SPEED_OF_LIGHT_M_S};
use libm::sqrt;

const C: f64 = SPEED_OF_LIGHT_M_S;

/// Chart id for the uniform weak-field chart (Record-axis annotation, M15).
pub const CHART_WEAKFIELD_UNIFORM: crate::relativity::ChartId = crate::relativity::ChartId(3);
/// Chart id for the central weak-field chart.
pub const CHART_WEAKFIELD_CENTRAL: crate::relativity::ChartId = crate::relativity::ChartId(4);

/// A static weak-field chart: a Newtonian potential `Phi(x)` (m^2/s^2), its gradient,
/// and the PPN beta coefficient carried by `g_00`'s second-order term.
pub trait StaticWeakFieldChart {
    fn potential(&self, pos: &[f64; 3]) -> f64;
    fn grad_potential(&self, pos: &[f64; 3]) -> [f64; 3];
    /// Coefficient of `2 beta (Phi/c^2)^2` in `g_00`. GR: 1. The task's first-order
    /// metric form: 0. Only observables reading `g_00` at second order (perihelion
    /// precession) can tell the difference — which is exactly the gate that fires on
    /// the truncated chart, with the ratio-4/3 signature.
    fn ppn_beta(&self) -> f64;
}

/// Uniform field: `Phi = g z`, so the Newtonian limit is acceleration `-g zhat`
/// (a body released from rest falls toward -z for `g > 0`). This is the task's
/// first-order metric form exactly (`ppn_beta = 0`); the uniform-chart gates stake
/// their coefficients against THIS chart's hand derivation, where the second-order
/// `g_00` term would shift the staked potential coefficient from 2 to 4.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UniformChart {
    pub g_m_s2: f64,
}

impl StaticWeakFieldChart for UniformChart {
    fn potential(&self, pos: &[f64; 3]) -> f64 {
        self.g_m_s2 * pos[2]
    }
    fn grad_potential(&self, _pos: &[f64; 3]) -> [f64; 3] {
        [0.0, 0.0, self.g_m_s2]
    }
    fn ppn_beta(&self) -> f64 {
        0.0
    }
}

/// Central field: `Phi = -GM/r`. `ppn_beta = 1` is the GR chart the precession gate
/// certifies; `ppn_beta = 0` is the first-order truncation, kept constructible because
/// it is the perihelion gate's planted defect (predicted signature 4/3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CentralChart {
    pub gm_m3_s2: f64,
    pub ppn_beta: f64,
}

impl StaticWeakFieldChart for CentralChart {
    fn potential(&self, pos: &[f64; 3]) -> f64 {
        let r = sqrt(pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]);
        -self.gm_m3_s2 / r
    }
    fn grad_potential(&self, pos: &[f64; 3]) -> [f64; 3] {
        let r2 = pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2];
        let r = sqrt(r2);
        let k = self.gm_m3_s2 / (r2 * r);
        [k * pos[0], k * pos[1], k * pos[2]]
    }
    fn ppn_beta(&self) -> f64 {
        self.ppn_beta
    }
}

/// Metric functions and gradients at a point: `(A, B, gradA, gradB)`.
fn metric_ab<Ch: StaticWeakFieldChart>(chart: &Ch, pos: &[f64; 3]) -> (f64, f64, [f64; 3], [f64; 3]) {
    let c2 = C * C;
    let phi = chart.potential(pos) / c2;
    let grad = chart.grad_potential(pos);
    let beta = chart.ppn_beta();
    let a = 1.0 + 2.0 * phi + 2.0 * beta * phi * phi;
    let b = 1.0 - 2.0 * phi;
    let da_scale = 2.0 * (1.0 + 2.0 * beta * phi) / c2;
    let db_scale = -2.0 / c2;
    (
        a,
        b,
        [grad[0] * da_scale, grad[1] * da_scale, grad[2] * da_scale],
        [grad[0] * db_scale, grad[1] * db_scale, grad[2] * db_scale],
    )
}

/// Geodesic 4-acceleration `du^mu/dtau` from the hand-derived Christoffels (module
/// docs). No mass parameter appears — universality of free fall is BY CONSTRUCTION,
/// and the gate checks the construction.
pub fn geodesic_accel<Ch: StaticWeakFieldChart>(chart: &Ch, pos: &[f64; 3], u: &[f64; 4]) -> [f64; 4] {
    let (a, b, da, db) = metric_ab(chart, pos);
    let usp = [u[1], u[2], u[3]];
    let da_dot_u = da[0] * usp[0] + da[1] * usp[1] + da[2] * usp[2];
    let db_dot_u = db[0] * usp[0] + db[1] * usp[1] + db[2] * usp[2];
    let usp2 = usp[0] * usp[0] + usp[1] * usp[1] + usp[2] * usp[2];
    let du0 = -(u[0] / a) * da_dot_u;
    let mut out = [du0, 0.0, 0.0, 0.0];
    for i in 0..3 {
        out[i + 1] = -da[i] * u[0] * u[0] / (2.0 * b)
            - (2.0 * usp[i] * db_dot_u - db[i] * usp2) / (2.0 * b);
    }
    out
}

fn derivative<Ch: StaticWeakFieldChart>(chart: &Ch, y: &[f64; 8]) -> [f64; 8] {
    let pos = [y[1], y[2], y[3]];
    let u = [y[4], y[5], y[6], y[7]];
    let a = geodesic_accel(chart, &pos, &u);
    [y[4], y[5], y[6], y[7], a[0], a[1], a[2], a[3]]
}

/// Integrate `steps` fixed RK4 steps of proper-time size `dtau`. dtau is PINNED by
/// each gate, which states its envelope at that dtau (section 3.5 discipline, same as
/// the flat integrator's gates).
pub fn integrate_geodesic<Ch: StaticWeakFieldChart>(
    chart: &Ch,
    start: &Worldline,
    dtau_s: f64,
    steps: u32,
) -> Worldline {
    let mut y = crate::relativity::pack(start);
    let f = |y: &[f64; 8]| derivative(chart, y);
    let mut i = 0;
    while i < steps {
        y = rk4_step(&f, &y, dtau_s);
        i += 1;
    }
    crate::relativity::unpack(&y)
}

/// Build a worldline ON SHELL IN THIS CHART from spatial celerity `w`:
/// `u^0 = sqrt((c^2 + B |w|^2)/A)`. The flat constructor
/// [`Worldline::from_celerity`] uses the Minkowski shell and enters a curved chart
/// off-shell by exactly `2 Phi/c^2` — measured at 2.2e-4 by the perihelion gate's
/// first run, which is why this constructor exists.
pub fn curved_from_celerity<Ch: StaticWeakFieldChart>(
    chart: &Ch,
    x: [f64; 4],
    w: [f64; 3],
) -> Worldline {
    let pos = [x[1], x[2], x[3]];
    let (a, b, _, _) = metric_ab(chart, &pos);
    let w2 = w[0] * w[0] + w[1] * w[1] + w[2] * w[2];
    let u0 = sqrt((C * C + b * w2) / a);
    Worldline { x, u: [u0, w[0], w[1], w[2]] }
}

/// Curved-chart normalization residual `(A (u^0)^2 - B |u_sp|^2 - c^2)/c^2` — the
/// curved analogue of the flat mass shell, and a certificate for the same reason:
/// `u^0` is evolved independently, so the residual genuinely drifts.
pub fn normalization_residual<Ch: StaticWeakFieldChart>(chart: &Ch, w: &Worldline) -> f64 {
    let pos = [w.x[1], w.x[2], w.x[3]];
    let (a, b, _, _) = metric_ab(chart, &pos);
    let usp2 = w.u[1] * w.u[1] + w.u[2] * w.u[2] + w.u[3] * w.u[3];
    (a * w.u[0] * w.u[0] - b * usp2 - C * C) / (C * C)
}

/// Conserved energy per unit mass `e = A u^0` (m/s) of the static chart's timelike
/// Killing vector. Chart state, not a chart-free total (see module misfit notes).
pub fn energy_per_mass<Ch: StaticWeakFieldChart>(chart: &Ch, w: &Worldline) -> f64 {
    let pos = [w.x[1], w.x[2], w.x[3]];
    let (a, _, _, _) = metric_ab(chart, &pos);
    a * w.u[0]
}

/// Conserved z angular momentum per unit mass `l = B (x u^y - y u^x)` (m^2/s) about
/// the center of a central chart.
pub fn angular_momentum_z<Ch: StaticWeakFieldChart>(chart: &Ch, w: &Worldline) -> f64 {
    let pos = [w.x[1], w.x[2], w.x[3]];
    let (_, b, _, _) = metric_ab(chart, &pos);
    b * (w.x[1] * w.u[2] - w.x[2] * w.u[1])
}

/// Static-observer clock rate `dtau/dt = sqrt(A)` at a point — the two-height
/// gravitational time dilation closed form. On its own this is algebra, not a gate
/// (an unfailable check is not a gate, L3); the ballistic proper-time gate is where
/// the integrator is actually held to the clock physics.
pub fn static_clock_rate<Ch: StaticWeakFieldChart>(chart: &Ch, pos: &[f64; 3]) -> f64 {
    let (a, _, _, _) = metric_ab(chart, pos);
    sqrt(a)
}

/// One reading of the perihelion instrument: measured apsidal advance per orbit and
/// the orbit parameters it was measured on, plus the conservation certificates of the
/// same run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrecessionReading {
    /// Measured advance per orbit beyond 2 pi, radians.
    pub advance_rad: f64,
    /// Semi-major axis measured from the flown orbit's extrema (m).
    pub a_m: f64,
    /// Eccentricity measured from the flown orbit's extrema.
    pub e: f64,
    /// Worst relative drift of the static chart's Killing quantities (e, l_z).
    pub worst_killing_drift: f64,
    /// Worst curved-chart normalization residual over the run.
    pub worst_normalization: f64,
    /// Number of perihelion passages detected.
    pub perihelia: u32,
}

/// The perihelion instrument, promoted to API: integrate an orbit at pinned `dtau`
/// and measure the apsidal advance per orbit by quadratic vertex interpolation of the
/// r^2 minima, with the Killing-drift and normalization certificates taken on the
/// same run. Returns `None` when fewer than three perihelia are detected (an orbit
/// this instrument cannot read, e.g. a plunge — never silently extrapolated).
///
/// This is the instrument behind the first-PN gate, the K-band probes of
/// CURVATURE_BRIDGE.md (dev/eps measured 1.505..1.711 over eps in [1e-5, 1e-3]), and
/// the sandbox's future scene validation.
pub fn measure_precession<Ch: StaticWeakFieldChart>(
    chart: &Ch,
    start: &Worldline,
    dtau: f64,
    steps: u32,
) -> Option<PrecessionReading> {
    let f = |y: &[f64; 8]| derivative(chart, y);
    let mut y = crate::relativity::pack(start);
    let w0 = crate::relativity::unpack(&y);
    let e0 = energy_per_mass(chart, &w0);
    let l0 = angular_momentum_z(chart, &w0);

    let mut worst_drift = 0.0_f64;
    let mut worst_norm = 0.0_f64;
    let mut phi_prev = libm::atan2(y[2], y[1]);
    let mut phi_unwrapped = phi_prev;
    let mut ring: [(f64, f64, f64); 3] = [(0.0, 0.0, 0.0); 3];
    let mut count = 0_usize;
    let mut first_peri: Option<(f64, f64)> = None;
    let mut last_peri: Option<(f64, f64)> = None;
    let mut peri_count = 0_u32;
    let mut r_min: f64 = f64::MAX;
    let mut r_max: f64 = 0.0;

    for i in 0..steps {
        y = rk4_step(&f, &y, dtau);
        let tau = (i + 1) as f64 * dtau;
        let w = crate::relativity::unpack(&y);
        let r2 = y[1] * y[1] + y[2] * y[2];
        let r = sqrt(r2);
        r_min = r_min.min(r);
        r_max = r_max.max(r);

        let phi = libm::atan2(y[2], y[1]);
        let mut dphi = phi - phi_prev;
        if dphi < -core::f64::consts::PI {
            dphi += 2.0 * core::f64::consts::PI;
        } else if dphi > core::f64::consts::PI {
            dphi -= 2.0 * core::f64::consts::PI;
        }
        phi_unwrapped += dphi;
        phi_prev = phi;

        worst_drift = worst_drift
            .max(libm::fabs(energy_per_mass(chart, &w) / e0 - 1.0))
            .max(libm::fabs(angular_momentum_z(chart, &w) / l0 - 1.0));
        worst_norm = worst_norm.max(libm::fabs(normalization_residual(chart, &w)));

        ring[count % 3] = (r2, phi_unwrapped, tau);
        count += 1;
        if count >= 3 {
            let a = ring[(count - 3) % 3];
            let b = ring[(count - 2) % 3];
            let c = ring[(count - 1) % 3];
            if b.0 < a.0 && b.0 < c.0 {
                let denom = a.0 - 2.0 * b.0 + c.0;
                let s = 0.5 * (a.0 - c.0) / denom;
                let tau_star = b.2 + s * dtau;
                let phi_star =
                    b.1 + 0.5 * s * (c.1 - a.1) + 0.5 * s * s * (a.1 - 2.0 * b.1 + c.1);
                if first_peri.is_none() {
                    first_peri = Some((tau_star, phi_star));
                }
                last_peri = Some((tau_star, phi_star));
                peri_count += 1;
            }
        }
    }

    let (first, last) = (first_peri?, last_peri?);
    if peri_count < 3 {
        return None;
    }
    let orbits = (peri_count - 1) as f64;
    Some(PrecessionReading {
        advance_rad: (last.1 - first.1) / orbits - 2.0 * core::f64::consts::PI,
        a_m: 0.5 * (r_min + r_max),
        e: (r_max - r_min) / (r_max + r_min),
        worst_killing_drift: worst_drift,
        worst_normalization: worst_norm,
        perihelia: peri_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relativity::{integrate_boost_field, pack, unpack, BoostField, FourMomentum};

    fn at_rest(pos: [f64; 3]) -> Worldline {
        Worldline::from_celerity([0.0, pos[0], pos[1], pos[2]], [0.0; 3])
    }

    // -------------------------------------------------------------------------
    // Gate 6: the flat limit — g = 0 reduces the geodesic integrator to free SR
    // motion exactly.
    // -------------------------------------------------------------------------

    #[test]
    fn flat_limit_reduces_to_free_sr_motion() {
        let chart = UniformChart { g_m_s2: 0.0 };
        let start = Worldline::from_celerity([0.0; 4], [0.3 * C, 0.1 * C, -0.2 * C]);
        let curved = integrate_geodesic(&chart, &start, 0.01, 500);
        let free = integrate_boost_field(
            &BoostField { alpha_m_s2: 0.0, dir: [1.0, 0.0, 0.0] },
            &start,
            0.01,
            500,
        );
        for k in 0..4 {
            assert!(curved.x[k] == free.x[k] && curved.u[k] == free.u[k]);
        }
    }

    // -------------------------------------------------------------------------
    // Gate 7: the Newtonian limit of the uniform chart, with BOTH remainder
    // coefficients staked (2 on Phi/c^2, 1/3 on g^3 t^4/c^2) — and the planted
    // B == 1 defect (space curvature dropped) firing both with PREDICTED
    // signatures (0 and 1/2). This is the A3 closure gate: the demo's g is now
    // the limit of this derivation.
    // -------------------------------------------------------------------------

    /// Planted defect: same chart, space curvature dropped (B forced to 1). A
    /// hand-recomputation of the RHS with dB = 0 — one term of the derivation removed,
    /// exactly the "wrong metric" mutation the gate must catch.
    fn derivative_b_planted_flat(chart: &UniformChart, y: &[f64; 8]) -> [f64; 8] {
        let pos = [y[1], y[2], y[3]];
        let u0 = y[4];
        let c2 = C * C;
        let phi = chart.potential(&pos) / c2;
        let grad = chart.grad_potential(&pos);
        let a = 1.0 + 2.0 * phi;
        let da = [0.0, 0.0, 2.0 * grad[2] / c2];
        let da_dot_u = da[2] * y[7];
        let du0 = -(u0 / a) * da_dot_u;
        [
            y[4],
            y[5],
            y[6],
            y[7],
            du0,
            -da[0] * u0 * u0 / 2.0,
            -da[1] * u0 * u0 / 2.0,
            -da[2] * u0 * u0 / 2.0,
        ]
    }

    fn drop_remainder<F: Fn(&[f64; 8]) -> [f64; 8]>(
        f: &F,
        z0: f64,
        g: f64,
        dtau: f64,
        t_target: f64,
    ) -> f64 {
        // Integrate a drop from rest at z0 and return z(t) - (z0 - g t^2/2) at the
        // target coordinate time (Hermite-interpolated).
        let mut y = pack(&at_rest([0.0, 0.0, z0]));
        loop {
            let prev = y;
            y = rk4_step(f, &y, dtau);
            if y[0] / C >= t_target {
                let t_prev = prev[0] / C;
                let dt_step = y[0] / C - t_prev;
                let dt = t_target - t_prev;
                let v_prev = prev[7] * C / prev[4];
                let v_next = y[7] * C / y[4];
                let acc = (v_next - v_prev) / dt_step;
                let z = prev[3] + v_prev * dt + 0.5 * acc * dt * dt;
                return z - (z0 - 0.5 * g * t_target * t_target);
            }
        }
    }

    #[test]
    fn newtonian_limit_coefficients_staked_and_measured() {
        // Chart values chosen to put both staked terms far above the f64 floor while
        // keeping Phi/c^2 and beta^2 in the weak regime: g = 1000 m/s^2.
        let g = 1000.0;
        let chart = UniformChart { g_m_s2: g };
        let f = |y: &[f64; 8]| derivative(&chart, y);

        // Stake (i), pre-registered from the hand derivation: dropping from rest at
        // z0, the remainder over the Newtonian chart is r = -(2 Phi0/c^2)(g t^2/2)
        // at short times. Coefficient staked at 2.00 +- 5%.
        let z0 = 5.0e7;
        let t = 10.0;
        let r = drop_remainder(&f, z0, g, 1.0e-3, t);
        let staked_i = -(2.0 * g * z0 / (C * C)) * (0.5 * g * t * t);
        let ratio_i = r / staked_i;
        assert!(
            (0.95..1.05).contains(&ratio_i),
            "potential coefficient off staked 2: ratio {ratio_i}"
        );

        // Stake (ii): from rest at z0 = 0 the remainder is r = g^3 t^4/(3 c^2)
        // (1 part Gamma^z_zz + 2 parts Gamma^0_{0z} feedback + 1 part from the depth
        // gained, /12 -> 1/3). Coefficient staked at 1/3 +- 5%, slope staked 4 +- 0.1.
        let r1 = drop_remainder(&f, 0.0, g, 1.0e-3, 50.0);
        let r2 = drop_remainder(&f, 0.0, g, 1.0e-3, 100.0);
        let t4 = 100.0 * 100.0 * 100.0 * 100.0;
        let staked_ii = g * g * g * t4 / (3.0 * C * C);
        let ratio_ii = r2 / staked_ii;
        assert!(
            (0.95..1.05).contains(&ratio_ii),
            "quartic coefficient off staked 1/3: ratio {ratio_ii}"
        );
        let slope = libm::log(r2 / r1) / libm::log(2.0);
        assert!(libm::fabs(slope - 4.0) < 0.1, "remainder slope: {slope}");

        // Planted defect 1: space curvature dropped (B == 1). Predicted signatures,
        // derived in advance: stake (i) collapses toward 0 (the 2 came entirely from
        // 1/B), stake (ii) falls to 1/2 of the true value (velocity coefficient
        // 2 instead of 3, depth term gone). Both stakes must fire.
        let fb = |y: &[f64; 8]| derivative_b_planted_flat(&chart, y);
        let r_b = drop_remainder(&fb, z0, g, 1.0e-3, t);
        let ratio_b_i = r_b / staked_i;
        assert!(
            !(0.95..1.05).contains(&ratio_b_i),
            "B==1 defect passed the potential-coefficient stake ({ratio_b_i})"
        );
        let r_b2 = drop_remainder(&fb, 0.0, g, 1.0e-3, 100.0);
        let ratio_b_ii = r_b2 / staked_ii;
        assert!(
            !(0.95..1.05).contains(&ratio_b_ii) && (0.4..0.6).contains(&ratio_b_ii),
            "B==1 defect signature should be ~1/2 on the quartic stake: {ratio_b_ii}"
        );

        // Planted defect 2: flipped Christoffel sign (the body falls up). The
        // remainder leaves the staked band by orders of magnitude.
        let f_flip = |y: &[f64; 8]| {
            let mut d = f(y);
            d[5] = -d[5];
            d[6] = -d[6];
            d[7] = -d[7];
            d
        };
        let r_flip = drop_remainder(&f_flip, z0, g, 1.0e-3, t);
        assert!(
            libm::fabs(r_flip / staked_i) > 1.0e3,
            "flipped-sign defect stayed near the staked remainder"
        );
    }

    // -------------------------------------------------------------------------
    // Gate 8: gravitational time dilation, integrated — the thrown clock ages
    // more by exactly v0^2/(6 c^2) over a ballistic arc (the g<z>/c^2 term minus
    // the <v^2>/2c^2 term). Closed form, integrator in the loop.
    // -------------------------------------------------------------------------

    #[test]
    fn ballistic_proper_time_closed_form() {
        let g = 1000.0;
        let chart = UniformChart { g_m_s2: g };
        let v0 = 1.0e5;
        let start = Worldline::from_celerity([0.0; 4], [0.0, 0.0, v0]);
        let dtau = 1.0e-2;
        let f = |y: &[f64; 8]| derivative(&chart, y);

        // Integrate to the return crossing z = 0, interpolating both tau and t.
        let measure = |f: &dyn Fn(&[f64; 8]) -> [f64; 8]| -> f64 {
            let mut y = pack(&start);
            let mut tau = 0.0;
            loop {
                let prev = y;
                let tau_prev = tau;
                y = rk4_step(&f, &y, dtau);
                tau += dtau;
                if tau > 1.0 && y[3] <= 0.0 {
                    // Quadratic Hermite in tau for the crossing: z' = u_z, z'' ~ -g.
                    let zp = prev[3];
                    let up = prev[7];
                    let acc = (y[7] - prev[7]) / dtau;
                    // Solve zp + up s + acc s^2/2 = 0 for s in [0, dtau].
                    let disc = sqrt(up * up - 2.0 * acc * zp);
                    let s = (-up - disc) / acc;
                    let tau_cross = tau_prev + s;
                    // Coordinate time at the crossing from u0.
                    let du0 = (y[4] - prev[4]) / dtau;
                    let t_cross = (prev[0] + prev[4] * s + 0.5 * du0 * s * s) / C;
                    return (tau_cross - t_cross) / t_cross;
                }
            }
        };

        let staked = v0 * v0 / (6.0 * C * C);
        let measured = measure(&f);
        let ratio = measured / staked;
        assert!(
            libm::fabs(ratio - 1.0) < 1.0e-3,
            "thrown-clock dilation off closed form v0^2/(6c^2): ratio {ratio}"
        );

        // Mutation: flip the sign of du^0/dtau (the Gamma^0_{0z} plant). Derived
        // signature: the flip integrates u^0 = c gamma0 * A instead of c gamma0 / A,
        // so (tau-T)/T = (-v0^2/2 - 2 v0^2/3)/c^2 = -(7/6) v0^2/c^2 — ratio exactly
        // -7. (The first run of this gate measured -7.000008 against a hand-waved -3
        // stake; the derivation above replaced the hand-wave and the measurement
        // confirms it to 1e-6.)
        let f_flip = |y: &[f64; 8]| {
            let mut d = f(y);
            d[4] = -d[4];
            d
        };
        let mutated_ratio = measure(&f_flip) / staked;
        assert!(
            libm::fabs(mutated_ratio - 1.0) > 1.0
                && (-7.5..-6.5).contains(&mutated_ratio),
            "Gamma^0 sign plant should read ratio ~ -7, got {mutated_ratio}"
        );
    }

    // -------------------------------------------------------------------------
    // Gate 9: universality of free fall — no mass parameter exists in the
    // geodesic equation, so two masses follow bit-identical geodesics. Replaces
    // the demo's bolted-on universality check at the foundation level. The
    // planted mass-coupled fifth force fires the stated 1e-12 gate.
    // -------------------------------------------------------------------------

    #[test]
    fn universality_two_masses_identical_geodesics() {
        let chart = CentralChart { gm_m3_s2: 8.98755179e20, ppn_beta: 1.0 };
        let start = curved_from_celerity(&chart, [0.0, 9.0e7, 0.0, 0.0], [0.0, 3.3e6, 0.0]);
        let (m_light, m_heavy) = (1.0, 1000.0);

        // The full pipeline including the ledger: mass enters p^mu = m u^mu ONLY.
        let end = integrate_geodesic(&chart, &start, 0.05, 2000);
        let end_again = integrate_geodesic(&chart, &start, 0.05, 2000);
        let p_light = FourMomentum::of(m_light, &end);
        let p_heavy = FourMomentum::of(m_heavy, &end_again);
        for k in 0..4 {
            // Bit-identical trajectories (mass never reached the integrator)...
            assert!(end.x[k].to_bits() == end_again.x[k].to_bits());
            // ...and the ledger scales exactly with mass.
            assert!(
                libm::fabs(p_heavy.p[k] - 1000.0 * p_light.p[k])
                    <= 1.0e-12 * libm::fabs(p_heavy.p[0])
            );
        }
        // The stated foundation gate: identical to 1e-12 (bit identity implies it;
        // stated so the replacement of the demo gate has an explicit tolerance).
        for k in 0..4 {
            assert!(libm::fabs(end.x[k] - end_again.x[k]) <= 1.0e-12 * libm::fabs(end.x[0]));
        }

        // Planted mass-coupled fifth force: acceleration scaled by (1 + 1e-6 (m/m0-1)).
        // The heavy body now falls differently and the 1e-12 gate fires.
        let mass_ratio = m_heavy / m_light;
        let f_fifth = |y: &[f64; 8]| {
            let pos = [y[1], y[2], y[3]];
            let u = [y[4], y[5], y[6], y[7]];
            let a = geodesic_accel(&chart, &pos, &u);
            let s = 1.0 + 1.0e-6 * (mass_ratio - 1.0);
            [y[4], y[5], y[6], y[7], a[0] * s, a[1] * s, a[2] * s, a[3] * s]
        };
        let mut y = pack(&start);
        for _ in 0..2000 {
            y = rk4_step(&f_fifth, &y, 0.05);
        }
        let heavy_defect = unpack(&y);
        let mut worst = 0.0_f64;
        for k in 0..4 {
            worst = worst.max(libm::fabs(heavy_defect.x[k] - end.x[k]) / libm::fabs(end.x[0]));
        }
        assert!(
            worst > 1.0e-12,
            "planted fifth force did not fire the universality gate ({worst:e})"
        );
    }

    // -------------------------------------------------------------------------
    // Gate 10: perihelion precession against 6 pi GM/(a(1-e^2)c^2), with the
    // conserved-quantity and normalization certificates on the same run, and the
    // first-order-metric plant firing at its PREDICTED 4/3 signature.
    // -------------------------------------------------------------------------

    /// Tuple adaptor over the promoted API instrument [`measure_precession`], keeping
    /// this module's gate bodies unchanged.
    fn precession<Ch: StaticWeakFieldChart>(
        chart: &Ch,
        start: &Worldline,
        dtau: f64,
        steps: u32,
    ) -> (f64, f64, f64, f64, f64) {
        let r = measure_precession(chart, start, dtau, steps)
            .expect("gate orbit must yield >= 3 perihelia");
        (
            r.advance_rad,
            r.a_m,
            r.e,
            r.worst_killing_drift,
            r.worst_normalization,
        )
    }

    #[test]
    fn perihelion_precession_first_pn() {
        // GM/(a c^2) = 1e-4, e ~ 0.1: precession ~ 1.9e-3 rad/orbit, measured over
        // ~6 orbits at dtau pinned to T/2100. Tolerance pre-registered at 1%
        // (perturbative formula; second-order corrections O(GM/(a c^2)) ~ 1e-4).
        let gm = 8.98755179e20;
        let chart = CentralChart { gm_m3_s2: gm, ppn_beta: 1.0 };
        let r_p = 9.0e7;
        let v_p = sqrt(gm * 1.1 / (1.0e8 * 0.9));
        let start = curved_from_celerity(&chart, [0.0, r_p, 0.0, 0.0], [0.0, v_p, 0.0]);
        let dtau = 0.1;
        let steps = 13_000;

        let (advance, a, e, drift, norm) = precession(&chart, &start, dtau, steps);
        let predicted = 6.0 * core::f64::consts::PI * gm / (a * (1.0 - e * e) * C * C);
        let ratio = advance / predicted;
        assert!(
            libm::fabs(ratio - 1.0) < 0.01,
            "perihelion advance off 6 pi GM/(a(1-e^2)c^2): ratio {ratio}"
        );

        // Conservation certificates on the same run, at the pinned dtau. Envelope
        // 1e-9: RK4's own linear-in-steps truncation at this dtau is ~3e-11
        // (13000 * (omega dtau)^5/120), so 1e-9 is a real ceiling with margin while a
        // planted first-order integrator sits at ~1e-3.
        assert!(drift < 1.0e-9, "Killing-vector drift: {drift:e}");
        assert!(norm < 1.0e-9, "normalization residual: {norm:e}");

        // Planted defect A — the first-order metric (ppn_beta = 0): the precession
        // observable reads g_00 at second order, and the PPN count
        // (2 + 2 gamma - beta)/3 predicts the truncated chart reads exactly 4/3 of
        // GR. Staked in advance: ratio in [1.28, 1.39], and the 1% gate fires.
        let truncated = CentralChart { gm_m3_s2: gm, ppn_beta: 0.0 };
        let (advance_t, a_t, e_t, _, _) = precession(&truncated, &start, dtau, steps);
        let predicted_t = 6.0 * core::f64::consts::PI * gm / (a_t * (1.0 - e_t * e_t) * C * C);
        let ratio_t = advance_t / predicted_t;
        assert!(
            (1.28..1.39).contains(&ratio_t),
            "first-order-metric plant should read 4/3, got {ratio_t}"
        );

        // Planted defect B — a wrong force power, mild enough to keep a measurable
        // orbit: force ~ r^-(2+eps) with eps = 1e-3. Classical apsidal-angle theory
        // (pi/sqrt(1-eps)) predicts a NEWTONIAN precession of pi*eps ~ 3.1e-3
        // rad/orbit — larger than the whole GR signal (1.9e-3) — so the gate cannot
        // miss it. Expected ratio ~ 1 + pi*eps/predicted ~ 2.6.
        struct WrongPower {
            gm: f64,
            r_ref: f64,
            eps: f64,
        }
        impl StaticWeakFieldChart for WrongPower {
            fn potential(&self, pos: &[f64; 3]) -> f64 {
                let r = sqrt(pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]);
                -self.gm / r
            }
            fn grad_potential(&self, pos: &[f64; 3]) -> [f64; 3] {
                let r2 = pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2];
                let r = sqrt(r2);
                // Planted: magnitude (GM/r^2)(r_ref/r)^eps instead of GM/r^2.
                let k = self.gm / (r2 * r) * libm::pow(self.r_ref / r, self.eps);
                [k * pos[0], k * pos[1], k * pos[2]]
            }
            fn ppn_beta(&self) -> f64 {
                1.0
            }
        }
        let wrong = WrongPower { gm, r_ref: r_p, eps: 1.0e-3 };
        let (advance_w, a_w, e_w, _, _) = precession(&wrong, &start, dtau, steps);
        let predicted_w = 6.0 * core::f64::consts::PI * gm / (a_w * (1.0 - e_w * e_w) * C * C);
        let ratio_w = advance_w / predicted_w;
        assert!(
            libm::fabs(ratio_w - 1.0) > 0.1 && (1.8..3.6).contains(&ratio_w),
            "wrong-power plant off its predicted ~2.6 signature: {ratio_w}"
        );
    }

    #[test]
    fn static_clock_rate_two_heights() {
        // The two-height closed form as a consistency check of the metric function
        // (labelled: algebra, not a gate on its own — the ballistic gate holds the
        // integrator to clock physics).
        // Values chosen so the effect (g dh/c^2 ~ 1.1e-9) sits far above the ulp floor
        // while the second-order term (phi^2/2 ~ 6e-19 relative) stays far below the
        // tolerance. The first run of this check used Earth-like g dh (1.1e-13) and
        // failed on the arithmetic grain floor — the beta-floor lesson applies to the
        // clock function too.
        let chart = UniformChart { g_m_s2: 1.0e4 };
        let low = static_clock_rate(&chart, &[0.0, 0.0, 0.0]);
        let high = static_clock_rate(&chart, &[0.0, 0.0, 1.0e4]);
        let predicted = 1.0e4 * 1.0e4 / (C * C);
        assert!(libm::fabs((high / low - 1.0) / predicted - 1.0) < 1.0e-6);
    }

    #[test]
    fn curved_replay_is_bit_identical() {
        let chart = CentralChart { gm_m3_s2: 8.98755179e20, ppn_beta: 1.0 };
        let start = curved_from_celerity(&chart, [0.0, 9.0e7, 0.0, 0.0], [0.0, 3.3e6, 0.0]);
        let a = integrate_geodesic(&chart, &start, 0.05, 1000);
        let b = integrate_geodesic(&chart, &start, 0.05, 1000);
        for k in 0..4 {
            assert!(a.x[k].to_bits() == b.x[k].to_bits() && a.u[k].to_bits() == b.u[k].to_bits());
        }
    }
}
