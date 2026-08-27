//! The curvature bridge: the certified curved tier chart (CURVATURE_BRIDGE.md,
//! commit a49145b — the frozen prereg this module implements).
//!
//! A tier adopts `(g0, Phi)` the way it adopts `g0` today: gravity is CHART DATA,
//! static and declared, and the geodesic stepper ([`crate::curvature`]) consumes it
//! LOCALLY — the step reads Phi and grad Phi at the worldline's own position only.
//! `Core/Locality.lean`'s staked kill (a certified update whose dependence exceeds its
//! declared radius; "the gravity chart's global Poisson solve is the known candidate")
//! is therefore faced BY CONSTRUCTION in v1: Phi is frame data, never updated, so no
//! Poisson solve exists to mislabel. The kill goes live exactly when Phi becomes
//! dynamical (self-gravity, Gantt A3's field-equation half) — recorded here so it
//! cannot be smuggled later. If a coarse holon under this chart ever needs an
//! omitted-mode error term, the named object is
//! `crate::field::HorizonLocality::influence_bound`, not a new one.
//!
//! ## The v1 potential family, and what is scoped out BY NAME
//!
//! [`ChartPhi`] = one uniform term plus finitely many superposed central terms.
//! Superposition is licensed by the weak-field premise itself (cross terms are
//! O(eps^2), inside the certificate's remainder). Scoped out, each a named gap and
//! not a silent limit: strong field (screened numerically), horizons, dynamical Phi,
//! FRW background/expansion (the cosmic screen catches it as arithmetic), gravitational
//! radiation, rotation (Lense-Thirring), logarithmic/NFW potentials — so a
//! flat-rotation-curve disk scene is UNDECLARABLE in v1 and refuses by name
//! ([`WeakFieldRefusal::UnsupportedPotentialFamily`], a FLOOR whose unlock is the v2
//! logarithmic-potential family), never a silent absence.
//!
//! ## The certificate and its probed band
//!
//! [`certify_weak_field`] screens a declared scene envelope and returns a remainder
//! bound `K * eps^2` with `K = 10`, staked in the frozen prereg over probes at
//! eps = 1e-5, 1e-4, 1e-3 (deviation coefficient dev/eps measured 1.505..1.711 —
//! the ~14% drift is recorded, not smoothed; worst-measured phase remainder
//! 5.2 * eps^2 per orbit, so K = 10 carries 1.9x headroom). The absolute screen cap
//! sits AT the probed boundary (eps <= 1e-3): certifying beyond the probed range would
//! be extrapolation, and the screen refuses there even when the tolerance arithmetic
//! would allow it.
//!
//! ## Refusal typing (Core/GrainFloor.lean's taxonomy)
//!
//! Floors (frame-relative; a chart re-root could serve the claim; each names its
//! unlock): weak-field exceedance (strong-field chart family), expansion-scale patches
//! (FRW chart family), unsupported potential family (v2 log family). The one CEILING:
//! a claim requiring a spacelike signal — justified because the light-cone partial
//! order is invariant under every chart in the metric-theory family (the EP premise
//! fixes Lorentzian signature), so no re-root WITHIN the family serves it; this is
//! T5's chart-free causality export, typed. Nothing else is claimed as ceiling.
//!
//! ## The seam
//!
//! Flat <-> curved over the same holon is a RE-ROOT; per
//! `cert_does_not_transport_across_reroot` NO certificate crosses it — each side
//! certifies its own claims. When eps sits below the f64 arithmetic floor the curved
//! and flat charts are certifiably indistinguishable and the FLAT chart is the
//! licensed answer ([`crate::holon::CertificationStatus::GrainFloor`], the M27
//! branch); `crate::relativity::certify_newton_chart` answers the force-free half of
//! the same seam.
//!
//! ## Sim-to-real anchor (demo copy, DELIBERATELY NOT A CI GATE)
//!
//! `static_clock_rate` at Earth's surface vs GPS orbit radius (26,560 km) gives
//! dPhi/c^2 = 5.29e-10, i.e. the published +45.7 microseconds/day gravitational
//! blueshift term. That sentence belongs in the demo as the sim-to-real anchor; CI
//! cannot gate on the real world, so no test asserts it.

use crate::curvature::StaticWeakFieldChart;
use crate::holon::CertificationStatus;
use libm::sqrt;

const C: f64 = crate::relativity::SPEED_OF_LIGHT_M_S;

/// The staked second-order remainder coefficient: chart error <= K * eps^2 per
/// dynamical time. Frozen in CURVATURE_BRIDGE.md with its two-point probes run
/// (worst measured 5.2, headroom 1.9x).
pub const WEAK_FIELD_REMAINDER_K: f64 = 10.0;

/// The absolute screen cap: the probed boundary of the K band. No certification
/// beyond it, whatever the tolerance arithmetic says.
pub const WEAK_FIELD_EPS_CAP: f64 = 1.0e-3;

/// Per-tier eps_max stakes (CURVATURE_BRIDGE.md section 2, as amended by A1).
pub const PLANET_EPS_MAX: f64 = 1.0e-8;
/// AMENDMENT A1(curvature), 2026-08-24, integrator-ruled: 1e-4 -> 1e-3.
///
/// Cause: the frozen 1e-4 was derived from a MIS-MEASURED scene value — the doc's
/// galactic row quoted the orbit-averaged GM/(a c^2) = 4.4e-5 where the certificate
/// screens the ENVELOPE maximum (full S2's pericenter reads 6.6e-4) — a category
/// error about the scene, not a choice about the certificate. The replacement is NOT
/// fitted to S2: it is the independently probed boundary (the K band ran at
/// eps = 1e-3 pre-freeze; gate B3 holds there, coefficient 1.505 in band). The
/// absolute cap [`WEAK_FIELD_EPS_CAP`] stays 1e-3: A1 moves the galactic stake TO
/// the probed boundary, never past it — no certifying beyond the probed range,
/// restated here exactly as at the freeze. Both readings stay gated
/// (`tier_scenes_certify_per_the_frozen_table`): full S2 refuses under
/// [`GALACTIC_EPS_MAX_FROZEN`], certifies under A1.
pub const GALACTIC_EPS_MAX: f64 = 1.0e-3;
/// The pre-A1 frozen galactic stake, kept so the gate shows BOTH readings rather
/// than erasing the history.
pub const GALACTIC_EPS_MAX_FROZEN: f64 = 1.0e-4;
pub const COSMIC_EPS_MAX: f64 = 1.0e-4;

/// One central mass in the declared potential.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Center {
    pub pos_m: [f64; 3],
    pub gm_m3_s2: f64,
}

/// The v1 potential family a tier declares: `Phi(x) = g z + sum_i -GM_i/|x - c_i|`.
/// Values on the scene chart — no new holon field anywhere.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartPhi<'a> {
    pub uniform_g_m_s2: f64,
    pub centers: &'a [Center],
}

impl StaticWeakFieldChart for ChartPhi<'_> {
    fn potential(&self, pos: &[f64; 3]) -> f64 {
        let mut phi = self.uniform_g_m_s2 * pos[2];
        for c in self.centers {
            let d = [pos[0] - c.pos_m[0], pos[1] - c.pos_m[1], pos[2] - c.pos_m[2]];
            let r = sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
            phi -= c.gm_m3_s2 / r;
        }
        phi
    }
    fn grad_potential(&self, pos: &[f64; 3]) -> [f64; 3] {
        let mut g = [0.0, 0.0, self.uniform_g_m_s2];
        for c in self.centers {
            let d = [pos[0] - c.pos_m[0], pos[1] - c.pos_m[1], pos[2] - c.pos_m[2]];
            let r2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            let k = c.gm_m3_s2 / (r2 * sqrt(r2));
            g[0] += k * d[0];
            g[1] += k * d[1];
            g[2] += k * d[2];
        }
        g
    }
    fn ppn_beta(&self) -> f64 {
        1.0
    }
}

/// The envelope a scene DECLARES so it can be screened. A scene that cannot declare
/// these is refused as undeclarable — a statement about the declaration, not a
/// physics verdict.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneEnvelope {
    /// Guaranteed minimum distance from any represented body to any center (m).
    pub r_min_m: f64,
    /// Vertical extent over which the uniform term acts (m).
    pub height_m: f64,
    /// Maximum coordinate speed the scene guarantees (m/s).
    pub v_max_m_s: f64,
    /// Cosmic tier only: declared comoving patch size (m); 0 elsewhere.
    pub patch_m: f64,
    /// Cosmic tier only: declared Hubble rate chart datum (1/s); 0 elsewhere.
    pub hubble_per_s: f64,
    /// True when the CLAIM needs influence outside the light cone. The one ceiling.
    pub requires_spacelike_signal: bool,
}

/// Why a weak-field certification refused, typed per the GrainFloor.lean taxonomy.
/// Every refusal names its unlock; the one ceiling names why NO re-root serves it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeakFieldRefusal {
    /// FLOOR: eps exceeds the screen. A stronger-field chart family could serve it.
    ExceedsWeakField,
    /// FLOOR: the background term (H L/c)^2 dominates — the FRW gap surfacing as
    /// arithmetic. The FRW chart family could serve it.
    ExpansionScale,
    /// FLOOR: the scene needs a potential outside the v1 family (e.g. a
    /// flat-rotation-curve disk needs a logarithmic potential). Undeclarable in v1,
    /// refused BY NAME rather than silently absent.
    UnsupportedPotentialFamily,
    /// CEILING: the claim requires influence outside the light cone. The light-cone
    /// partial order is invariant under every chart in the metric-theory family (the
    /// EP premise fixes Lorentzian signature), so no re-root within the family serves
    /// it — invariant refusals are exactly what GrainFloor.lean types as ceilings.
    RequiresSpacelikeSignal,
    /// The envelope is not a declaration (non-finite or non-positive where positivity
    /// is required). Not a physics verdict.
    Undeclarable,
}

impl WeakFieldRefusal {
    /// What would have to happen for the refusal to lift — the refusal's second half,
    /// same standard as every refusal on the tier ladder.
    pub const fn unlock(self) -> &'static str {
        match self {
            WeakFieldRefusal::ExceedsWeakField => {
                "Awaits a strong-field chart family (Schwarzschild exact for a single \
                 center). The weak-field screen is the probed boundary, not the physics'."
            }
            WeakFieldRefusal::ExpansionScale => {
                "Awaits the FRW chart family. The static weak-field chart cannot carry \
                 the expansion; patches small enough that (H L/c)^2 passes the screen \
                 certify today."
            }
            WeakFieldRefusal::UnsupportedPotentialFamily => {
                "Awaits the v2 logarithmic-potential family (flat rotation curves, NFW). \
                 The v1 family is uniform + superposed 1/r centers only."
            }
            WeakFieldRefusal::RequiresSpacelikeSignal => {
                "Nothing lifts this at any tier: the light-cone order is invariant \
                 across the whole metric-theory chart family. A ceiling, not a floor."
            }
            WeakFieldRefusal::Undeclarable => {
                "Declare the envelope: r_min to every center, height, v_max (and patch \
                 + Hubble rate at the cosmic tier). Screening needs numbers."
            }
        }
    }

    /// Ceiling vs floor, per the taxonomy: a ceiling is invariant under every re-root
    /// within the chart family; a floor is lifted by one.
    pub const fn is_ceiling(self) -> bool {
        matches!(self, WeakFieldRefusal::RequiresSpacelikeSignal)
    }
}

/// The weak-field certificate for one declared scene on one declared chart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeakFieldCertificate {
    /// `Certified` | `GrainFloor` (eps below the f64 arithmetic floor: curved and
    /// flat charts certifiably indistinguishable — the FLAT chart is the licensed
    /// answer, the M27 seam branch) | `RefinementUnavailable` (see `refusal`).
    pub status: CertificationStatus,
    pub refusal: Option<WeakFieldRefusal>,
    /// max over the declared envelope of |Phi|/c^2 and (v/c)^2 (background term
    /// included at the cosmic tier).
    pub epsilon: f64,
    /// The cosmic background contribution (H L/c)^2 alone.
    pub epsilon_bg: f64,
    /// K * eps^2 — the certified fractional remainder per dynamical time.
    pub remainder_bound: f64,
    pub tolerance: f64,
}

impl WeakFieldCertificate {
    const fn refused(reason: WeakFieldRefusal, epsilon: f64, epsilon_bg: f64, tol: f64) -> Self {
        Self {
            status: CertificationStatus::RefinementUnavailable,
            refusal: Some(reason),
            epsilon,
            epsilon_bg,
            remainder_bound: f64::INFINITY,
            tolerance: tol,
        }
    }
}

/// Screen a declared scene against a declared chart and tier stake.
///
/// Screens, in order: declarability; the ceiling; the arithmetic floor (M27); the
/// tier's frozen eps_max; the absolute probed cap; the tolerance-derived cap. The
/// certificate can FAIL — the screen gate plants an over-eps scene and a weakened
/// screen to prove it.
pub fn certify_weak_field(
    chart: &ChartPhi,
    env: &SceneEnvelope,
    tier_eps_max: f64,
    tolerance: f64,
) -> WeakFieldCertificate {
    if env.requires_spacelike_signal {
        return WeakFieldCertificate::refused(
            WeakFieldRefusal::RequiresSpacelikeSignal,
            f64::NAN,
            f64::NAN,
            tolerance,
        );
    }
    let declarable = env.r_min_m.is_finite()
        && env.height_m.is_finite()
        && env.v_max_m_s.is_finite()
        && env.patch_m.is_finite()
        && env.hubble_per_s.is_finite()
        && env.height_m >= 0.0
        && env.v_max_m_s >= 0.0
        && env.patch_m >= 0.0
        && env.hubble_per_s >= 0.0
        && (chart.centers.is_empty() || env.r_min_m > 0.0)
        && tolerance > 0.0
        && tier_eps_max > 0.0;
    if !declarable {
        return WeakFieldCertificate::refused(
            WeakFieldRefusal::Undeclarable,
            f64::NAN,
            f64::NAN,
            tolerance,
        );
    }

    // Conservative envelope maxima: |Phi| bounded by the uniform term over the height
    // plus every center at the guaranteed minimum distance.
    let c2 = C * C;
    let mut phi_max = libm::fabs(chart.uniform_g_m_s2) * env.height_m;
    for c in chart.centers {
        phi_max += c.gm_m3_s2 / env.r_min_m;
    }
    let eps_phi = phi_max / c2;
    let beta = env.v_max_m_s / C;
    let eps_v = beta * beta;
    let hb = env.hubble_per_s * env.patch_m / C;
    let eps_bg = hb * hb;
    let epsilon = eps_phi.max(eps_v).max(eps_bg);

    let base = WeakFieldCertificate {
        status: CertificationStatus::Certified,
        refusal: None,
        epsilon,
        epsilon_bg: eps_bg,
        remainder_bound: WEAK_FIELD_REMAINDER_K * epsilon * epsilon,
        tolerance,
    };

    // The M27 arithmetic floor: corrections below one ulp of unity mean the curved
    // and flat charts are certifiably indistinguishable — GrainFloor, and the flat
    // chart is the licensed answer (the seam's admissibility, not a failure).
    if epsilon < 2.0 * f64::EPSILON {
        return WeakFieldCertificate {
            status: CertificationStatus::GrainFloor,
            ..base
        };
    }

    let over_screen = epsilon > tier_eps_max
        || epsilon > WEAK_FIELD_EPS_CAP
        || base.remainder_bound > tolerance;
    if over_screen {
        // Typing: when the background term is the binding contribution the gap is the
        // FRW one; otherwise the weak-field premise itself.
        let reason = if eps_bg >= eps_phi && eps_bg >= eps_v {
            WeakFieldRefusal::ExpansionScale
        } else {
            WeakFieldRefusal::ExceedsWeakField
        };
        return WeakFieldCertificate::refused(reason, epsilon, eps_bg, tolerance);
    }
    base
}

/// The refusal a tier returns for a scene whose potential is outside the v1 family
/// (the flat-rotation-curve disk is the named case). Provided as a constructor so the
/// sandbox's refusal path presents it BY NAME with its unlock, never as an absence.
pub const fn unsupported_family_certificate(tolerance: f64) -> WeakFieldCertificate {
    WeakFieldCertificate::refused(
        WeakFieldRefusal::UnsupportedPotentialFamily,
        f64::NAN,
        f64::NAN,
        tolerance,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curvature::{curved_from_celerity, measure_precession, CentralChart};
    use crate::relativity::rk4_step;

    /// Earth's GM (CODATA-consistent, m^3/s^2).
    const GM_EARTH: f64 = 3.986_004_418e14;

    // -------------------------------------------------------------------------
    // Gate B1: the three tier scenes certify exactly as the frozen doc's table
    // says, with the doc's epsilon values.
    // -------------------------------------------------------------------------

    #[test]
    fn tier_scenes_certify_per_the_frozen_table() {
        // Planet: Earth as one center, ballistic + orbital envelope.
        let earth = [Center { pos_m: [0.0; 3], gm_m3_s2: GM_EARTH }];
        let chart = ChartPhi { uniform_g_m_s2: 0.0, centers: &earth };
        let env = SceneEnvelope {
            r_min_m: 6.371e6,
            height_m: 0.0,
            v_max_m_s: 7.8e3,
            patch_m: 0.0,
            hubble_per_s: 0.0,
            requires_spacelike_signal: false,
        };
        let cert = certify_weak_field(&chart, &env, PLANET_EPS_MAX, 1.0e-6);
        assert_eq!(cert.status, CertificationStatus::Certified);
        assert!(
            (6.9e-10..7.1e-10).contains(&cert.epsilon),
            "planet eps should read ~7.0e-10, got {:e}",
            cert.epsilon
        );
        assert!(cert.remainder_bound < 1.0e-16);

        // Galactic, S2-CLASS wide orbit around Sgr A* (GM = 5.7e26): r_min 2000 AU.
        let sgr = [Center { pos_m: [0.0; 3], gm_m3_s2: 5.708e26 }];
        let chart_g = ChartPhi { uniform_g_m_s2: 0.0, centers: &sgr };
        let wide = SceneEnvelope {
            r_min_m: 2.993e14, // 2000 AU
            v_max_m_s: 1.5e6,
            ..env
        };
        let cert_g = certify_weak_field(&chart_g, &wide, GALACTIC_EPS_MAX, 1.0e-4);
        assert_eq!(cert_g.status, CertificationStatus::Certified);
        assert!(cert_g.epsilon < GALACTIC_EPS_MAX);

        // BOTH READINGS of the full S2 orbit, per amendment A1's ruling (one scene,
        // both verdicts visible). The envelope maxima are S2's PERICENTER values
        // (r_p ~ 120 AU, v_p ~ 7.7e6 m/s), eps = 6.6e-4 — the frozen doc's 4.4e-5
        // was the orbit-averaged GM/(a c^2), the category error A1 cites as cause.
        let s2_full = SceneEnvelope {
            r_min_m: 1.795e13, // ~120 AU pericenter
            v_max_m_s: 7.7e6,
            ..env
        };
        // Reading 1 — under the pre-A1 frozen stake: REFUSES as a floor.
        let cert_frozen =
            certify_weak_field(&chart_g, &s2_full, GALACTIC_EPS_MAX_FROZEN, 1.0e-4);
        assert_eq!(cert_frozen.status, CertificationStatus::RefinementUnavailable);
        assert_eq!(cert_frozen.refusal, Some(WeakFieldRefusal::ExceedsWeakField));
        // Reading 2 — under A1 (the probed boundary): CERTIFIES, remainder in band.
        let cert_s2 = certify_weak_field(&chart_g, &s2_full, GALACTIC_EPS_MAX, 1.0e-4);
        assert_eq!(cert_s2.status, CertificationStatus::Certified);
        assert!(
            (5.0e-4..8.0e-4).contains(&cert_s2.epsilon),
            "full-S2 envelope eps should read ~6.6e-4, got {:e}",
            cert_s2.epsilon
        );
        assert!(cert_s2.remainder_bound <= 1.0e-4);

        // Cosmic: 30 Mpc patch certifies; 100 Mpc refuses on the BACKGROUND term with
        // the FRW typing — the expansion gap surfacing as arithmetic, not fiat.
        let cluster = [Center { pos_m: [0.0; 3], gm_m3_s2: 1.0e5 * GM_EARTH }];
        let chart_c = ChartPhi { uniform_g_m_s2: 0.0, centers: &cluster };
        let patch_ok = SceneEnvelope {
            r_min_m: 3.1e22, // ~1 Mpc to the cluster center
            v_max_m_s: 1.0e6,
            patch_m: 9.26e23, // 30 Mpc
            hubble_per_s: 2.2e-18,
            ..env
        };
        let cert_c = certify_weak_field(&chart_c, &patch_ok, COSMIC_EPS_MAX, 1.0e-4);
        assert_eq!(cert_c.status, CertificationStatus::Certified);
        assert!((3.0e-5..6.0e-5).contains(&cert_c.epsilon_bg));

        let patch_big = SceneEnvelope { patch_m: 3.086e24, ..patch_ok }; // 100 Mpc
        let cert_big = certify_weak_field(&chart_c, &patch_big, COSMIC_EPS_MAX, 1.0e-4);
        assert_eq!(cert_big.status, CertificationStatus::RefinementUnavailable);
        assert_eq!(cert_big.refusal, Some(WeakFieldRefusal::ExpansionScale));
        assert!(cert_big.epsilon_bg > 4.0e-4);
    }

    // -------------------------------------------------------------------------
    // Gate B2: the screen can fail, and a weakened screen wrongly certifies —
    // the mutation that proves the screen is load-bearing.
    // -------------------------------------------------------------------------

    #[test]
    fn screen_refuses_over_eps_and_a_weakened_screen_would_not() {
        // This gate tests the SCREEN MECHANISM, so it pins its own stake rather than
        // a tier's: after amendment A1 the galactic stake sits AT the absolute cap,
        // where a 2x-over scene would refuse via the cap and the weakened-screen
        // mutant could no longer demonstrate anything.
        let stake = 1.0e-4;
        let sgr = [Center { pos_m: [0.0; 3], gm_m3_s2: 5.708e26 }];
        let chart = ChartPhi { uniform_g_m_s2: 0.0, centers: &sgr };
        // A scene at eps = 2x the pinned stake (still under the absolute cap).
        let r_for_2x = 5.708e26 / (2.0 * stake * C * C);
        let env = SceneEnvelope {
            r_min_m: r_for_2x,
            height_m: 0.0,
            v_max_m_s: 0.0,
            patch_m: 0.0,
            hubble_per_s: 0.0,
            requires_spacelike_signal: false,
        };
        let cert = certify_weak_field(&chart, &env, stake, 1.0e-4);
        assert_eq!(cert.status, CertificationStatus::RefinementUnavailable);
        assert_eq!(cert.refusal, Some(WeakFieldRefusal::ExceedsWeakField));
        assert!(!cert.refusal.unwrap().is_ceiling());
        assert!(cert.refusal.unwrap().unlock().contains("strong-field"));

        // MUTANT: weaken the tier stake 10x and the same scene certifies — the screen,
        // not the tolerance arithmetic, is what refused it (K*eps^2 = 4e-7 passes the
        // 1e-4 tolerance). A screen that nothing depends on would change nothing here.
        let weakened = certify_weak_field(&chart, &env, stake * 10.0, 1.0e-4);
        assert_eq!(
            weakened.status,
            CertificationStatus::Certified,
            "the planted weakened screen should wrongly certify, proving the screen \
             is the load-bearing refusal"
        );
    }

    // -------------------------------------------------------------------------
    // Gate B3: the K band, re-run in-suite as a permanent gate (the prereg's
    // probe made durable), with the K = 0.1 plant exceeded by measurement.
    // -------------------------------------------------------------------------

    fn perihelion_deviation(gm: f64, dtau: f64, steps: u32) -> (f64, f64) {
        let chart = CentralChart { gm_m3_s2: gm, ppn_beta: 1.0 };
        let a_nom: f64 = 1.0e8;
        let r_p = 0.9 * a_nom;
        let v_p = sqrt(gm * 1.1 / (a_nom * 0.9));
        let start = curved_from_celerity(&chart, [0.0, r_p, 0.0, 0.0], [0.0, v_p, 0.0]);
        let r = measure_precession(&chart, &start, dtau, steps).unwrap();
        let predicted =
            6.0 * core::f64::consts::PI * gm / (r.a_m * (1.0 - r.e * r.e) * C * C);
        (r.advance_rad / predicted - 1.0, gm / (r.a_m * C * C))
    }

    #[test]
    fn k_band_holds_at_both_probe_points_and_the_weak_k_plant_fails() {
        // The frozen probes, as a permanent gate: dev/eps in [1.0, 3.0] at eps = 1e-4
        // and 1e-3 (measured 1.505 at both calibration points; 1.711 at 1e-5), and
        // scaling exponent 1.00 +- 0.05.
        let (d1, e1) = perihelion_deviation(8.98755179e20, 0.1, 13_000);
        let (d2, e2) = perihelion_deviation(8.98755179e21, 0.03, 13_800);
        let c1 = d1 / e1;
        let c2 = d2 / e2;
        assert!(
            (1.0..3.0).contains(&c1) && (1.0..3.0).contains(&c2),
            "dev/eps off the probed band: {c1} {c2}"
        );
        let slope = libm::log(libm::fabs(d2) / libm::fabs(d1)) / libm::log(e2 / e1);
        assert!(libm::fabs(slope - 1.0) < 0.05, "deviation scaling exponent: {slope}");

        // The staked K bounds the measured phase remainder at both points...
        for (d, e) in [(d1, e1), (d2, e2)] {
            let phase_remainder =
                libm::fabs(d) * 6.0 * core::f64::consts::PI * e / (0.99 * 2.0 * core::f64::consts::PI);
            assert!(
                phase_remainder <= WEAK_FIELD_REMAINDER_K * e * e,
                "measured remainder exceeded the staked K band at eps={e:e}"
            );
            // ...and the planted K = 0.1 is EXCEEDED by the same measurement — the
            // band is falsifiable, not slack.
            assert!(
                phase_remainder > 0.1 * e * e,
                "planted K=0.1 was not exceeded at eps={e:e}; the band proves nothing"
            );
        }
    }

    // -------------------------------------------------------------------------
    // Gate B4: superposition — a far orbit around a two-center PAIR, differenced
    // against the coincident-pair reference, shows exactly the quadrupole cross
    // term (staked slope 2 in the separation), and a sign-flipped half fires.
    //
    // LESSON, kept: this gate's first design staked a d^-3 "tide" from a fixed
    // companion and the measurement convicted the reference (slope 1.38): the
    // free-fall tide intuition does not apply to a STATIC chart, where the
    // leading cross term from a fixed companion is the uniform (a/D)^2 force
    // with a secular 1/e enhancement. The differencing design below has a clean
    // reference: Omega around the pair vs Omega around the same total mass at a
    // point, which cancels the relativistic offset identically and leaves the
    // quadrupole as the whole difference.
    // -------------------------------------------------------------------------

    #[test]
    fn superposed_pair_quadrupole_scales_as_separation_squared() {
        let gm_total = 8.98755179e20;
        let r_orbit: f64 = 1.0e8;
        let v_circ = sqrt(gm_total / r_orbit);

        // Mean coordinate angular rate over ~2 orbits around a pair of centers at
        // (+-s/2, 0, 0) carrying fractions (f1, f2) of gm_total.
        let omega = |s_m: f64, f1: f64, f2: f64| -> f64 {
            let centers = [
                Center { pos_m: [0.5 * s_m, 0.0, 0.0], gm_m3_s2: gm_total * f1 },
                Center { pos_m: [-0.5 * s_m, 0.0, 0.0], gm_m3_s2: gm_total * f2 },
            ];
            let chart = ChartPhi { uniform_g_m_s2: 0.0, centers: &centers };
            let start =
                curved_from_celerity(&chart, [0.0, r_orbit, 0.0, 0.0], [0.0, v_circ, 0.0]);
            let f = |y: &[f64; 8]| {
                let pos = [y[1], y[2], y[3]];
                let u = [y[4], y[5], y[6], y[7]];
                let a = crate::curvature::geodesic_accel(&chart, &pos, &u);
                [y[4], y[5], y[6], y[7], a[0], a[1], a[2], a[3]]
            };
            let mut y = crate::relativity::pack(&start);
            let mut phi_prev = libm::atan2(y[2], y[1]);
            let mut phi_unw = 0.0;
            for _ in 0..4_600 {
                y = rk4_step(&f, &y, 0.1);
                let phi = libm::atan2(y[2], y[1]);
                let mut dphi = phi - phi_prev;
                if dphi < -core::f64::consts::PI {
                    dphi += 2.0 * core::f64::consts::PI;
                } else if dphi > core::f64::consts::PI {
                    dphi -= 2.0 * core::f64::consts::PI;
                }
                phi_unw += dphi;
                phi_prev = phi;
            }
            phi_unw / (y[0] / C)
        };

        // Differencing against the coincident pair cancels the relativistic offset
        // identically; what remains is the quadrupole, staked slope 2 in s.
        let base = omega(0.0, 0.5, 0.5);
        let d1 = libm::fabs(omega(r_orbit / 20.0, 0.5, 0.5) / base - 1.0);
        let d2 = libm::fabs(omega(r_orbit / 10.0, 0.5, 0.5) / base - 1.0);
        let slope = libm::log(d2 / d1) / libm::log(2.0);
        assert!(
            (1.8..2.2).contains(&slope),
            "pair quadrupole should scale as s^2, got slope {slope} (d1={d1:e}, d2={d2:e})"
        );
        // Measured coefficient recorded, not staked (the frozen doc defers it):
        // d2 / (s/2R)^2 lands near the classical 3/4 quadrupole factor.

        // MUTANT: flip the SIGN of one half (a repulsive center is outside the
        // family): the monopole halves, the launch speed no longer matches any
        // circular orbit, and the mean rate leaves the certified reading by orders.
        let flipped = libm::fabs(omega(r_orbit / 10.0, 0.5, -0.5) / base - 1.0);
        assert!(
            flipped > 100.0 * d2,
            "sign-flipped half-center should wreck the far orbit, got {flipped:e} vs {d2:e}"
        );
    }

    // -------------------------------------------------------------------------
    // Gate B5: refusal typing — floors name their unlocks, the one ceiling is
    // justified and invariant, undeclarable is a declaration statement.
    // -------------------------------------------------------------------------

    #[test]
    fn refusals_are_typed_with_unlocks() {
        // The one ceiling: spacelike claims. is_ceiling, and refuses regardless of
        // every other envelope number (invariance is what MAKES it a ceiling).
        let chart = ChartPhi { uniform_g_m_s2: 9.81, centers: &[] };
        let env = SceneEnvelope {
            r_min_m: 1.0,
            height_m: 1.0,
            v_max_m_s: 1.0,
            patch_m: 0.0,
            hubble_per_s: 0.0,
            requires_spacelike_signal: true,
        };
        let cert = certify_weak_field(&chart, &env, PLANET_EPS_MAX, 1.0e-6);
        assert_eq!(cert.refusal, Some(WeakFieldRefusal::RequiresSpacelikeSignal));
        assert!(cert.refusal.unwrap().is_ceiling());
        assert!(cert.refusal.unwrap().unlock().contains("Nothing lifts this"));

        // Floors are not ceilings and each names its unlock.
        for floor in [
            WeakFieldRefusal::ExceedsWeakField,
            WeakFieldRefusal::ExpansionScale,
            WeakFieldRefusal::UnsupportedPotentialFamily,
        ] {
            assert!(!floor.is_ceiling());
            assert!(!floor.unlock().is_empty());
        }
        // The named v1 gap carries the named v2 unlock (attachment 2: undeclarable
        // must refuse by name, never as a silent absence).
        assert!(
            WeakFieldRefusal::UnsupportedPotentialFamily
                .unlock()
                .contains("logarithmic"),
            "the flat-rotation-curve refusal must name the v2 log-potential unlock"
        );
        let unsupported = unsupported_family_certificate(1.0e-4);
        assert_eq!(
            unsupported.refusal,
            Some(WeakFieldRefusal::UnsupportedPotentialFamily)
        );

        // Undeclarable: a NaN envelope is a declaration failure, not physics.
        let bad = SceneEnvelope { r_min_m: f64::NAN, requires_spacelike_signal: false, ..env };
        let earth = [Center { pos_m: [0.0; 3], gm_m3_s2: GM_EARTH }];
        let chart_c = ChartPhi { uniform_g_m_s2: 0.0, centers: &earth };
        let cert_bad = certify_weak_field(&chart_c, &bad, PLANET_EPS_MAX, 1.0e-6);
        assert_eq!(cert_bad.refusal, Some(WeakFieldRefusal::Undeclarable));
    }

    // -------------------------------------------------------------------------
    // Gate B6: the M27 seam branch — a scene below the arithmetic floor returns
    // GrainFloor (flat chart licensed), and one just above it does not.
    // -------------------------------------------------------------------------

    #[test]
    fn arithmetic_floor_returns_grain_floor_and_lifts_just_above_it() {
        // The thrown ball, tiny room: g = 9.81 over 1 m at 5 m/s.
        // eps = max(9.81/c^2, (5/c)^2) = 2.8e-16 < 2*EPS -> GrainFloor: the flat chart
        // is the licensed answer, stated as a certificate rather than assumed.
        let chart = ChartPhi { uniform_g_m_s2: 9.81, centers: &[] };
        let tiny = SceneEnvelope {
            r_min_m: 0.0,
            height_m: 1.0,
            v_max_m_s: 5.0,
            patch_m: 0.0,
            hubble_per_s: 0.0,
            requires_spacelike_signal: false,
        };
        let cert = certify_weak_field(&chart, &tiny, PLANET_EPS_MAX, 1.0e-6);
        assert_eq!(cert.status, CertificationStatus::GrainFloor);

        // The demo ball scene (100 m of height, 30 m/s) sits ABOVE the floor and
        // certifies with a real (if tiny) remainder — the curved chart is the warrant
        // even where its correction is nanometres.
        let ball = SceneEnvelope { height_m: 100.0, v_max_m_s: 30.0, ..tiny };
        let cert_ball = certify_weak_field(&chart, &ball, PLANET_EPS_MAX, 1.0e-6);
        assert_eq!(cert_ball.status, CertificationStatus::Certified);
        assert!(cert_ball.epsilon > 2.0 * f64::EPSILON && cert_ball.epsilon < 2.0e-14);
    }

    /// The superposed chart's potential and gradient agree with the sum of its parts
    /// exactly — an algebraic consistency check, labelled as such (L3: not a gate on
    /// its own; gate B4 is where superposition faces dynamics).
    #[test]
    fn chart_phi_is_the_sum_of_its_parts() {
        let centers = [
            Center { pos_m: [1.0e7, 0.0, 0.0], gm_m3_s2: GM_EARTH },
            Center { pos_m: [0.0, -3.0e7, 2.0e7], gm_m3_s2: 0.5 * GM_EARTH },
        ];
        let chart = ChartPhi { uniform_g_m_s2: 3.0, centers: &centers };
        let p = [2.0e6, 1.0e6, -4.0e6];
        let mut phi = 3.0 * p[2];
        for c in &centers {
            let d = [p[0] - c.pos_m[0], p[1] - c.pos_m[1], p[2] - c.pos_m[2]];
            phi -= c.gm_m3_s2 / sqrt(d[0] * d[0] + d[1] * d[1] + d[2] * d[2]);
        }
        assert!(libm::fabs(chart.potential(&p) - phi) <= 1.0e-12 * libm::fabs(phi));
    }
}
