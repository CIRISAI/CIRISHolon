//! THE RIGID CLOCK: the fine engine's rule, fed the retained modes.
//!
//! `holon-render/src/clock.rs` derives the fine step from the BONDED table: `omega_e` off the
//! O–H curve's own minimum, `dt_reference = period / 64`, and `hold_exactness` halves `dt`
//! until `omega_env * dt <= 2pi/64` with `omega_env` from the bonded curves' curvature at the
//! scene's energy. Every input is intramolecular, which is why rigid water is a clock change
//! and not only a mode removal (the fourth review, correction 4). This module is the same
//! rule — the same accuracy target, the same halving hold — with the retained modes as its
//! inputs: the body's mass under the contact stiffness for translation, and its principal
//! inertia under the contact stiffness at the site's lever arm for rotation.
//!
//! What it is NOT: a licence. `allow_dt_growth` is not here and never will be; a step this
//! clock does not admit is not taken.

/// Steps per period of the fastest retained mode, the fine clock's own `64`.
pub const STEPS_PER_PERIOD: f64 = 64.0;

/// The accuracy target `omega * dt` the reference step meets by construction, `2pi / 64`.
pub fn accuracy_target() -> f64 {
    core::f64::consts::TAU / STEPS_PER_PERIOD
}

/// The stiffness environment a body sits in: the largest curvature any of its sites sees
/// from the intermolecular law (hartree per bohr²) — a MEASURED input, read off the served
/// law at the scene's energy the way the fine envelope is, never typed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StiffnessEnvelope {
    pub k_max: f64,
}

/// What the clock read: the retained translational and rotational frequencies, the reference
/// step they imply, and the step the hold admits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClockReading {
    pub omega_translation: f64,
    pub omega_rotation: f64,
    /// The largest of the retained frequencies: the one the step is set by.
    pub omega_max: f64,
    /// `period / STEPS_PER_PERIOD` at `omega_max`.
    pub dt_reference: f64,
    /// The step in force after the hold: `dt_reference` halved until `omega_max * dt <= target`.
    /// Equal to `dt_reference` by construction when the envelope that set `omega_max` is the
    /// one the hold checks; smaller when a caller hands a reference from elsewhere.
    pub dt: f64,
    /// `omega_max * dt`: how much accuracy the step buys.
    pub omega_dt: f64,
}

/// Derive the rigid clock. `mass` and the principal `inertia` are the body's; `lever_max` is
/// the largest distance of any site from the centre of mass; `k_max` the envelope. A site
/// under stiffness `k` at lever `rho` is a torsional stiffness `k rho²` about the body's
/// centre, so `omega_rot = sqrt(k rho² / I_min)`.
pub fn derive(mass: f64, inertia: [f64; 3], lever_max: f64, envelope: &StiffnessEnvelope) -> ClockReading {
    let k = envelope.k_max.max(0.0);
    let omega_translation = (k / mass).sqrt();
    let i_min = inertia.iter().cloned().fold(f64::INFINITY, f64::min);
    let omega_rotation = if i_min > 0.0 { (k * lever_max * lever_max / i_min).sqrt() } else { 0.0 };
    let omega_max = omega_translation.max(omega_rotation);
    let dt_reference = if omega_max > 0.0 { core::f64::consts::TAU / omega_max / STEPS_PER_PERIOD } else { f64::INFINITY };
    let dt = hold(dt_reference, omega_max);
    ClockReading { omega_translation, omega_rotation, omega_max, dt_reference, dt, omega_dt: omega_max * dt }
}

/// HOLD = EXACTNESS, the fine clock's own rule: halve the step until `omega * dt <= target`,
/// bounded at 2^-20 so a pathological envelope cannot spin here.
pub fn hold(dt_reference: f64, omega: f64) -> f64 {
    let target = accuracy_target();
    let mut dt = dt_reference;
    if omega <= 0.0 || !dt.is_finite() {
        return dt;
    }
    for _ in 0..20 {
        if omega * dt <= target * (1.0 + 1e-12) {
            break;
        }
        dt *= 0.5;
    }
    dt
}

#[cfg(test)]
mod tests {
    use super::*;

    // Water in electron masses and bohr: O 15.9949146196 u, H 1.00782503207 u, at the
    // engine's pinned monomer (r = 1.94357384 bohr, theta = 1.6887434 rad). Principal
    // inertias computed from that geometry by `rigid::RigidWater`; here they are typed as the
    // test's declared inputs so this module's tests need no other module.
    const M_WATER: f64 = 32831.2;
    const I_WATER: [f64; 3] = [3.87e3, 8.16e3, 1.203e4];
    const LEVER: f64 = 1.85;

    #[test]
    fn the_hold_meets_the_target_by_construction() {
        let r = derive(M_WATER, I_WATER, LEVER, &StiffnessEnvelope { k_max: 0.05 });
        assert!(r.omega_dt <= accuracy_target() * (1.0 + 1e-12));
        assert!((r.dt - r.dt_reference).abs() <= 1e-15 * r.dt_reference, "the reference already meets the target");
        assert!(r.omega_rotation > r.omega_translation, "a light lever on a light inertia is the fast mode");
    }

    #[test]
    fn a_reference_from_elsewhere_is_halved_until_it_meets_the_target() {
        let omega = 1.0e-2;
        let dt = hold(8.0 * accuracy_target() / omega, omega);
        assert!((dt - accuracy_target() / omega).abs() <= 1e-12 * dt, "three halvings exactly");
    }

    #[test]
    fn the_rigid_clock_is_slower_than_the_fine_o_h_clock_by_the_ratio_of_periods() {
        // The fine clock on this box: dt_reference 4.309924 au from the O-H curve (period
        // 6.67 fs ~ 5,000 cm^-1), refined by 4 to 1.077481 au by the envelope. A rigid body
        // under a contact stiffness of 0.05 Ha/bohr^2 (a hydrogen bond's order of magnitude,
        // a DECLARED input here and a measured one in the adapter) rotates at a period an
        // order of magnitude longer, and the clock says so with the same rule.
        let fine_dt = 1.077481;
        let r = derive(M_WATER, I_WATER, LEVER, &StiffnessEnvelope { k_max: 0.05 });
        assert!(r.dt > 5.0 * fine_dt, "rigid dt {} against fine {}", r.dt, fine_dt);
        assert!(r.dt < 100.0 * fine_dt, "and not absurdly so: {}", r.dt / fine_dt);
    }

    #[test]
    fn a_stiffer_envelope_never_buys_a_larger_step() {
        let a = derive(M_WATER, I_WATER, LEVER, &StiffnessEnvelope { k_max: 0.01 });
        let b = derive(M_WATER, I_WATER, LEVER, &StiffnessEnvelope { k_max: 0.04 });
        assert!(b.dt < a.dt);
        assert!((a.dt / b.dt - 2.0).abs() < 1e-12, "omega goes as sqrt(k): 4x the stiffness is half the step");
    }

    #[test]
    fn a_zero_envelope_has_no_clock_and_says_so() {
        let r = derive(M_WATER, I_WATER, LEVER, &StiffnessEnvelope { k_max: 0.0 });
        assert_eq!(r.omega_max, 0.0);
        assert!(r.dt.is_infinite(), "no stiffness, no bound - the caller must supply one");
    }
}
