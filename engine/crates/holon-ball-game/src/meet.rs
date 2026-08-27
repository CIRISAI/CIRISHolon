//! MEET-2 (PROGRAM.md Path N-c): the T4 rigid chart and the T5 Newtonian chart are
//! two charts over the SAME holon, and this module certifies the seam.
//!
//! The claim: a wall exported through [`RigidChartExport`] and evolved in the T5
//! Newtonian chart is the same holon under two charts — identical gross ledger,
//! identical damage Record, dynamics agreeing within the COMPOSED error bound
//! (rigid-chart approximation + T5 chart remainder). Scope is T5's coverage today:
//! weightless or uniform-g segments only. Rapier is the limiting-case CONTROL
//! (engine-compare `meet_control`), never the reference — the reference for a
//! uniform-g rigid segment is the ballistic closed form.
//!
//! Error composition, both components with named provenance:
//! * RIGID side: the resident wall's centre of mass under uniform gravity obeys
//!   EXACTLY the solver-damped recursion `v' = (v + g dt)/(1 + lambda dt)` — the
//!   momentum-ledger gate proved every internal force (bonds, tangential springs,
//!   sliders, node contact) sums to zero — so the rigid-chart approximation error is
//!   the damper deviation from ballistic, computed by running that recursion, not
//!   estimated. Declared with 1e-6 relative fp headroom for the 288-node summation.
//! * T5 side: the uniform chart's Newtonian-limit remainder with the coefficients
//!   STAKED AND MEASURED in `curvature.rs` Gate 7 — `(2 g z0/c^2)(g t^2/2)` from
//!   release height plus `g^3 t^4 / (3 c^2)` from rest — composed here as data,
//!   not re-derived. A segment whose relative T5 remainder exceeds
//!   [`T5_RELATIVE_TOLERANCE`] is REFUSED: SR required, the Newtonian chart is not
//!   licensed there.
//!
//! P^mu, honored by refusal: no relativistic four-momentum is minted for game
//! entities. Whether the GrossState sector table relabels to P^mu outside the T5
//! chart's own interior (`sector_table_is_pmu_table` is chart-internal) is an OPEN
//! integrator decision — [`four_momentum_of_export`] refuses rather than improvises.

use ciris_sim_core::holon::CertificationStatus;
use ciris_sim_core::material::RigidChartExport;
use ciris_sim_core::relativity::SPEED_OF_LIGHT_M_S;

/// Relative T5-remainder tolerance for licensing the Newtonian chart on a segment.
pub const T5_RELATIVE_TOLERANCE: f64 = 1.0e-6;

/// Declared fp accretion floor for the Record's opening odometer across a contact-free
/// segment. The resident chart's arithmetic mints ~1e-16 m spurious maximum openings
/// per bond as translated positions re-round (measured ~1e-13 summed over 797 bonds
/// per second of flight); the Record is MONOTONE, so the seam gate demands exact
/// damage fields, monotone opening, and accretion below this floor — far beneath any
/// physical opening (the demo's peak opening is 2.4e-2 m).
pub const RECORD_OPENING_NOISE_FLOOR_M: f64 = 1.0e-9;

/// One weightless or uniform-g segment of the T5 Newtonian chart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeetSegment {
    /// Uniform gravitational acceleration over the segment (0 = weightless). For the
    /// demo wall this is the scene chart's g times the named stage knob — the seam
    /// compares charts over the same holon under the same declared field.
    pub g_m_s2: f64,
    /// Release height above the uniform chart's potential origin (enters the
    /// staked first remainder coefficient).
    pub start_height_m: f64,
    pub duration_s: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeetCertificate {
    pub status: CertificationStatus,
    /// Exact gate: subject, gross ledger, and mass identical across the handoff.
    pub ledger_identical: bool,
    /// The damage Record survived the contact-free segment: damage fields exact,
    /// opening odometer monotone with accretion below the declared fp floor. Erased
    /// or minted history refuses.
    pub record_intact: bool,
    pub rigid_side_bound_m: f64,
    pub t5_side_bound_m: f64,
    pub measured_deviation_m: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeetError {
    /// The GrossState-to-P^mu relabeling outside the T5 chart interior is an open
    /// integrator decision; minting a four-momentum for a game entity would
    /// improvise it.
    RelativisticMomentumUndecided,
}

/// REFUSAL, not a stub: see [`MeetError::RelativisticMomentumUndecided`].
pub fn four_momentum_of_export(_export: &RigidChartExport) -> Result<[f64; 4], MeetError> {
    Err(MeetError::RelativisticMomentumUndecided)
}

/// The rigid chart's approximation error for a resident wall's centre of mass over a
/// uniform-g segment: the exact solver-damper deviation from ballistic, by running
/// the same recursion the integrator applies, plus 1e-6 relative fp headroom.
pub fn rigid_side_damper_bound_m(
    g_m_s2: f64,
    duration_s: f64,
    damping_per_s: f64,
    dt_s: f64,
) -> f64 {
    let steps = (duration_s / dt_s).round() as u64;
    let mut damped_v = 0.0_f64;
    let mut damped_x = 0.0_f64;
    let mut ballistic_v = 0.0_f64;
    let mut ballistic_x = 0.0_f64;
    for _ in 0..steps {
        damped_v = (damped_v + g_m_s2 * dt_s) / (1.0 + damping_per_s * dt_s);
        damped_x += damped_v * dt_s;
        ballistic_v += g_m_s2 * dt_s;
        ballistic_x += ballistic_v * dt_s;
    }
    // The resident integrator IS the damped recursion; the T5 rigid body is compared
    // against the continuous ballistic closed form, so the discrete-vs-continuous
    // ballistic gap (g t dt / 2) is part of the rigid chart's declared error too.
    let closed_form = 0.5 * g_m_s2 * duration_s * duration_s;
    let bound = (damped_x - ballistic_x).abs() + (ballistic_x - closed_form).abs();
    bound * (1.0 + 1.0e-6) + f64::EPSILON
}

/// The T5 uniform-chart Newtonian-limit remainder over the segment — the two
/// coefficients staked and measured in `curvature.rs` Gate 7, composed as data.
pub fn t5_side_remainder_bound_m(segment: &MeetSegment) -> f64 {
    let c2 = SPEED_OF_LIGHT_M_S * SPEED_OF_LIGHT_M_S;
    let g = segment.g_m_s2.abs();
    let t = segment.duration_s;
    let height_term =
        (2.0 * g * segment.start_height_m.abs() / c2) * (0.5 * g * t * t);
    let rest_term = g * g * g * t * t * t * t / (3.0 * c2);
    height_term + rest_term
}

/// The Record survival gate: damage fields exact, opening odometer monotone with
/// accretion below [`RECORD_OPENING_NOISE_FLOOR_M`].
pub fn record_intact(exported: &RigidChartExport, returned: &RigidChartExport) -> bool {
    let out = &exported.record;
    let back = &returned.record;
    out.bond_count == back.bond_count
        && out.broken_count == back.broken_count
        && out.mean_damage == back.mean_damage
        && out.max_damage == back.max_damage
        && back.total_maximum_opening_m >= out.total_maximum_opening_m
        && back.total_maximum_opening_m - out.total_maximum_opening_m
            <= RECORD_OPENING_NOISE_FLOOR_M
}

/// Certify one pass through the seam. `Certified` requires ALL of: exact ledger
/// identity, the Record surviving intact, the T5 chart licensed on the segment
/// (relative remainder within [`T5_RELATIVE_TOLERANCE`]), and the measured
/// centre-of-mass deviation within the COMPOSED bound. Anything else refuses.
pub fn certify_meet(
    exported: &RigidChartExport,
    returned: &RigidChartExport,
    segment: &MeetSegment,
    measured_deviation_m: f64,
    rigid_side_bound_m: f64,
) -> MeetCertificate {
    let ledger_identical = exported.subject_holon == returned.subject_holon
        && exported.gross == returned.gross
        && exported.mass_kg == returned.mass_kg;
    let record_intact = record_intact(exported, returned);
    let t5_side_bound_m = t5_side_remainder_bound_m(segment);

    let trajectory_scale = (0.5 * segment.g_m_s2.abs() * segment.duration_s
        * segment.duration_s)
        .max(f64::EPSILON);
    let t5_licensed = t5_side_bound_m / trajectory_scale <= T5_RELATIVE_TOLERANCE;

    let within_bound = measured_deviation_m.is_finite()
        && measured_deviation_m <= rigid_side_bound_m + t5_side_bound_m;

    let status = if ledger_identical && record_intact && t5_licensed && within_bound {
        CertificationStatus::Certified
    } else {
        CertificationStatus::RefinementUnavailable
    };
    MeetCertificate {
        status,
        ledger_identical,
        record_intact,
        rigid_side_bound_m,
        t5_side_bound_m,
        measured_deviation_m,
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support;
    use super::*;
    use ciris_sim_core::regplus::GrossState;

    fn free_wall_flight(
        pre_damage_bonds: usize,
        steps: u32,
    ) -> (RigidChartExport, RigidChartExport, MeetSegment, f64, f64) {
        let (exported, returned, com_drop, com_drift, start_height) =
            test_support::free_wall_com_flight(pre_damage_bonds, steps);
        let g = test_support::wall_effective_g();
        let t = steps as f64 * test_support::fixed_step();
        let segment = MeetSegment {
            g_m_s2: g,
            start_height_m: start_height,
            duration_s: t,
        };
        let ballistic_drop = 0.5 * g * t * t;
        let measured = ((com_drop - ballistic_drop).powi(2) + com_drift.powi(2)).sqrt();
        let rigid_bound = rigid_side_damper_bound_m(
            g,
            t,
            test_support::solver_velocity_damping_per_s(),
            test_support::fixed_step(),
        );
        (exported, returned, segment, measured, rigid_bound)
    }

    #[test]
    fn wall_is_one_holon_under_two_charts() {
        // The MEET-2 claim on the pristine wall over a 1 s uniform-g free segment:
        // exact ledger and Record identity, and the resident centre of mass within
        // the composed (damper + staked T5 remainder) bound of the T5 ballistic
        // trajectory.
        let (exported, returned, segment, measured, rigid_bound) = free_wall_flight(0, 600);
        let cert = certify_meet(&exported, &returned, &segment, measured, rigid_bound);
        assert_eq!(cert.status, CertificationStatus::Certified, "{cert:?}");
        assert!(cert.ledger_identical && cert.record_intact);
        assert!(cert.measured_deviation_m <= cert.rigid_side_bound_m + cert.t5_side_bound_m);
        // The composed bound is real, not slack: the measured deviation is within it
        // but NOT trivially zero (the damper genuinely bends the resident COM).
        assert!(cert.measured_deviation_m > 1.0e-9);
    }

    #[test]
    fn record_survives_the_seam_and_distinguishes_history() {
        // A4 extended through the seam: a pre-damaged wall (all D < 1) and a
        // pristine wall each round-trip with their Record INTACT, and the two
        // round-trips differ — history stays provable across the chart handoff.
        let (pristine_out, pristine_back, ..) = free_wall_flight(0, 120);
        let (damaged_out, damaged_back, segment, measured, rigid_bound) =
            free_wall_flight(30, 120);
        assert!(record_intact(&pristine_out, &pristine_back));
        assert!(record_intact(&damaged_out, &damaged_back));
        assert!(damaged_back.record.mean_damage > pristine_back.record.mean_damage);
        assert_eq!(damaged_back.record.broken_count, 0, "pre-damage is sub-critical");
        let cert = certify_meet(&damaged_out, &damaged_back, &segment, measured, rigid_bound);
        assert_eq!(cert.status, CertificationStatus::Certified, "{cert:?}");
    }

    #[test]
    fn meet_certificate_can_refuse() {
        // Planted-error controls: every gate of the composed certificate can fire.
        let (exported, returned, segment, measured, rigid_bound) = free_wall_flight(0, 120);

        // (1) Trajectory overshoot: measured deviation past the composed bound.
        let total = rigid_bound + t5_side_remainder_bound_m(&segment);
        let overshoot =
            certify_meet(&exported, &returned, &segment, measured + 2.0 * total, rigid_bound);
        assert_eq!(overshoot.status, CertificationStatus::RefinementUnavailable);

        // (2) Doctored ledger: a different gross must refuse.
        let mut doctored = returned;
        doctored.gross = GrossState::aggregate(999_999, 0, [0, 0]);
        let ledger = certify_meet(&exported, &doctored, &segment, measured, rigid_bound);
        assert!(!ledger.ledger_identical);
        assert_eq!(ledger.status, CertificationStatus::RefinementUnavailable);

        // (3) Doctored Record, both directions: minted damage refuses, and erased
        // opening history (the odometer rolled BACK) refuses on a damaged flight.
        let mut minted = returned;
        minted.record.mean_damage += 0.125;
        let record = certify_meet(&exported, &minted, &segment, measured, rigid_bound);
        assert!(!record.record_intact);
        assert_eq!(record.status, CertificationStatus::RefinementUnavailable);
        let (damaged_out, damaged_back, dseg, dmeas, dbound) = free_wall_flight(30, 120);
        let mut erased = damaged_back;
        erased.record.total_maximum_opening_m = 0.0;
        let record = certify_meet(&damaged_out, &erased, &dseg, dmeas, dbound);
        assert!(!record.record_intact);
        assert_eq!(record.status, CertificationStatus::RefinementUnavailable);

        // (4) Relativistic segment: the T5 chart is not licensed where its own
        // staked remainder exceeds tolerance — SR required, the meet refuses.
        let relativistic = MeetSegment {
            g_m_s2: SPEED_OF_LIGHT_M_S,
            start_height_m: 0.0,
            duration_s: 2.0,
        };
        let sr = certify_meet(&exported, &returned, &relativistic, 0.0, f64::MAX);
        assert_eq!(sr.status, CertificationStatus::RefinementUnavailable);
    }

    #[test]
    fn relativistic_momentum_is_refused_not_improvised() {
        let (exported, ..) = free_wall_flight(0, 1);
        assert_eq!(
            four_momentum_of_export(&exported),
            Err(MeetError::RelativisticMomentumUndecided)
        );
    }
}
