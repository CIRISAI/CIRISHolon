//! Weight, by certificate.
//!
//! The three outer tiers used to refuse: the engine's relativity chart is flat, and its
//! own integration frame said every scene with weight sat outside the T5 certificate.
//! `ciris_sim_core::bridge` closed that, so those tiers now declare a weak-field chart
//! and a scene envelope, and `bridge::certify_weak_field` screens them. Weight pulls
//! here because a certificate says it may, at a measured `epsilon` with a bounded
//! remainder — not because gravity was switched on.
//!
//! # What a tier declares
//!
//! Values on the scene chart, exactly as everywhere else on this ladder: a potential
//! from the v1 family (uniform term plus superposed 1/r centers) and an envelope the
//! scene guarantees about itself. Nothing here is a new holon field, and the gravity
//! charts write NOTHING into the REG+ ledger's occupancy or momentum lanes —
//! `tests::gravity_charts_write_no_occupancy_or_momentum` keeps that true, because the
//! cap headline in `tier` depends on it.
//!
//! # Why each tier carries two scenes
//!
//! Because one number cannot say what the certificate says. Each gravity tier declares a
//! scene that CERTIFIES and one that does not, and the pair is the point:
//!
//! * **planet** — Earth's own potential with a 100 m throw certifies at `eps = 6.96e-10`;
//!   the same throw with only the local uniform field over the ball's own millimetres
//!   falls below the f64 arithmetic floor, where curved and flat charts are certifiably
//!   indistinguishable and the FLAT one is the licensed answer. Both are honest and they
//!   are different sentences.
//! * **galactic** — S2 around Sgr A*, orbit-averaged, certifies under both stakes; the
//!   FULL S2 envelope at perihelion REFUSES under the frozen `1e-4` stake and CERTIFIES
//!   under the A1 amendment's `1e-3`. That is a live amendment shown as one rather than
//!   quietly adopted.
//! * **cosmic** — a 30 Mpc comoving patch certifies; a 100 Mpc patch refuses on the
//!   expansion-scale screen, BY NAME, with the FRW unlock. The refusal is the demo.

use ciris_sim_core::bridge::{
    certify_weak_field, unsupported_family_certificate, Center, ChartPhi, SceneEnvelope,
    WeakFieldCertificate, COSMIC_EPS_MAX, GALACTIC_EPS_MAX, GALACTIC_EPS_MAX_FROZEN,
    PLANET_EPS_MAX,
};

/// Earth's gravitational parameter, m^3/s^2. IERS/WGS-84 GM.
pub const GM_EARTH: f64 = 3.986_004_418e14;
/// Earth's mean radius, m.
pub const R_EARTH: f64 = 6.371e6;
/// Standard gravity at the surface, m/s^2. The uniform term of the local chart.
pub const G_SURFACE: f64 = 9.806_65;

/// Solar gravitational parameter, m^3/s^2. IAU 2015 nominal.
pub const GM_SUN: f64 = 1.327_124_400_18e20;
/// Astronomical unit, m. IAU 2012, exact by definition.
pub const AU_M: f64 = 1.495_978_707e11;
/// Sgr A* mass in solar masses (GRAVITY Collaboration).
pub const SGR_A_SOLAR_MASSES: f64 = 4.3e6;
/// S2's semi-major axis and eccentricity (GRAVITY Collaboration).
pub const S2_SEMI_MAJOR_AU: f64 = 970.0;
pub const S2_ECCENTRICITY: f64 = 0.88;

/// Kiloparsec, m.
pub const KPC_M: f64 = 3.085_677_581_491_367_3e19;

/// Megaparsec, m.
pub const MPC_M: f64 = 3.085_677_581_491_367_3e22;
/// Hubble constant, 1/s, from 67.4 km/s/Mpc (Planck 2018).
pub const HUBBLE_PER_S: f64 = 67.4e3 / MPC_M;

/// Sgr A*'s gravitational parameter, m^3/s^2.
pub const GM_SGR_A: f64 = SGR_A_SOLAR_MASSES * GM_SUN;

const NO_CENTERS: [Center; 0] = [];

const EARTH_CENTER: [Center; 1] = [Center {
    pos_m: [0.0, 0.0, 0.0],
    gm_m3_s2: GM_EARTH,
}];

const SGR_A_CENTER: [Center; 1] = [Center {
    pos_m: [0.0, 0.0, 0.0],
    gm_m3_s2: GM_SGR_A,
}];

/// The potential family a scene REQUIRES.
///
/// A scene does not only declare values — it declares what SHAPE of potential its claim
/// needs. The v1 family is a uniform term plus superposed 1/r centers and nothing else,
/// so a claim that needs a different shape cannot be expressed in it at all. Saying which
/// family is wanted is what lets that be refused BY NAME instead of being silently absent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PotentialFamily {
    /// Uniform + superposed 1/r centers. What `ChartPhi` can express.
    V1Superposed,
    /// A logarithmic potential — the flat-rotation-curve disk, and NFW. Outside v1.
    LogarithmicDisk,
}

/// One declared scene on one declared chart: what the tier is claiming about, and what
/// it guarantees so the claim can be screened.
#[derive(Clone, Copy, Debug)]
pub struct GravityScene {
    pub name: &'static str,
    /// What this scene is, for a general reader.
    pub plain: &'static str,
    pub uniform_g_m_s2: f64,
    pub centers: &'static [Center],
    pub envelope: SceneEnvelope,
    /// The stake this scene is screened against. Normally the tier's, but the galactic
    /// tier deliberately carries one scene at the FROZEN stake and one at the amended
    /// one, so the amendment is visible rather than assumed.
    pub tier_eps_max: f64,
    pub tolerance: f64,
    /// The uniform acceleration the geodesic stepper feels near the scene's bodies,
    /// m/s^2. Read from the chart at the envelope rather than declared independently.
    pub local_g_m_s2: f64,
    /// How wide the scene is to LOOK, in metres — which is not the tier's domain.
    ///
    /// The tier's domain is the extent of its LEDGER: the Earth, the galaxy, the
    /// observable universe. The claim lives somewhere else entirely — a hundred metres
    /// of thrown ball inside a 12,742 km planet, one star's orbit inside a hundred
    /// thousand light years. Drawing the claim at the ledger's extent would render every
    /// one of these as a single pixel, and pretending the two extents are the same
    /// number is the kind of quiet conflation this ladder exists to avoid.
    pub view_m: f64,
    /// Where the body starts and how fast, in the chart's own coordinates: position
    /// (m) and celerity (m/s). `None` for a scene with nothing to throw.
    pub body: Option<([f64; 3], [f64; 3])>,
    /// Proper-time step for the geodesic integration, s. Pinned per scene, because the
    /// dynamical times here span seventeen orders of magnitude.
    pub dtau_s: f64,
    /// Where `view_m` is centred, in the chart's own coordinates. The launch point for a
    /// ballistic scene; the central mass for an orbital one.
    pub view_center: [f64; 2],
    /// Which two of the chart's three axes the scene is drawn in. DECLARED, not inferred
    /// from the chart's shape: the first version worked out "ballistic or orbital" from
    /// whether a uniform term was present, which stopped being a reliable signal the
    /// moment a scene had a centre and no uniform term.
    pub plane: [usize; 2],
    /// The potential family this scene's claim requires.
    pub family: PotentialFamily,
}

impl GravityScene {
    pub fn chart(&self) -> ChartPhi<'_> {
        ChartPhi {
            uniform_g_m_s2: self.uniform_g_m_s2,
            centers: self.centers,
        }
    }

    /// Screen this scene. The certificate can refuse, and three of the six declared
    /// scenes do.
    pub fn certify(&self) -> WeakFieldCertificate {
        // A scene needing a potential the v1 family cannot express is refused for THAT
        // reason, before any screening. Screening it against a chart that does not
        // describe it would produce an epsilon about the wrong potential — a number
        // that looks like a verdict and is about nothing.
        if self.family != PotentialFamily::V1Superposed {
            return unsupported_family_certificate(self.tolerance);
        }
        certify_weak_field(&self.chart(), &self.envelope, self.tier_eps_max, self.tolerance)
    }

    /// The observer's claim on this scene: the smallest feature distinguishable on the
    /// stage at the scene's own view extent.
    pub fn acuity_m(&self) -> f64 {
        self.view_m * crate::tier::ACUITY_PIXELS / crate::tier::STAGE_PIXELS
    }
}

const fn envelope(r_min_m: f64, height_m: f64, v_max_m_s: f64) -> SceneEnvelope {
    SceneEnvelope {
        r_min_m,
        height_m,
        v_max_m_s,
        patch_m: 0.0,
        hubble_per_s: 0.0,
        requires_spacelike_signal: false,
    }
}

/// The planet tier's scenes: a thrown ball under Earth's potential, and the same throw
/// read on the local uniform chart alone.
pub const PLANET_SCENES: [GravityScene; 2] = [
    GravityScene {
        name: "thrown ball, Earth's potential",
        plain: "A ball thrown a hundred metres, with Earth's own gravitational \
                potential declared. Weight pulls here by certificate.",
        // ZERO uniform term. Earth's centre is the whole of this chart's gravity, and
        // its gradient at the surface IS g. Declaring both a uniform 9.81 AND the centre
        // — which the first version did — makes the ball fall at 19.63 m/s^2, and the
        // only symptom is a flight time exactly half of what it should be. Two sources
        // for one field is a double count, however physical each looks alone.
        uniform_g_m_s2: 0.0,
        centers: &EARTH_CENTER,
        envelope: envelope(R_EARTH, 100.0, 30.0),
        tier_eps_max: PLANET_EPS_MAX,
        tolerance: 1.0e-6,
        local_g_m_s2: GM_EARTH / (R_EARTH * R_EARTH),
        view_m: 200.0,
        // On the surface, thrown at 30 m/s and 45 degrees.
        body: Some((
            [0.0, 0.0, R_EARTH],
            [21.213_203_435_596_43, 0.0, 21.213_203_435_596_43],
        )),
        dtau_s: 1.0e-2,
        view_center: [0.0, R_EARTH],
        plane: [0, 2],
        family: PotentialFamily::V1Superposed,
    },
    GravityScene {
        name: "the ball alone, local field only",
        plain: "The same ball over its own millimetres, with only the local uniform \
                field. Too small for curvature to be tellable from flatness.",
        uniform_g_m_s2: G_SURFACE,
        centers: &NO_CENTERS,
        envelope: envelope(1.0, 1.0e-3, 0.05),
        tier_eps_max: PLANET_EPS_MAX,
        tolerance: 1.0e-6,
        local_g_m_s2: G_SURFACE,
        view_m: 4.0e-3,
        // NOTHING is thrown here, and that is not an omission. This scene's claim is
        // about a ball over its own millimetres, and its envelope declares exactly that
        // height; a ball actually released would fall twelve centimetres in the time the
        // demo runs and break the envelope it was screened against. What the scene has
        // to show is the CERTIFICATE — that at this scale the curved and flat charts are
        // indistinguishable in f64 and the flat one is licensed — and a trajectory would
        // add nothing to it except a declaration that is no longer true.
        body: None,
        dtau_s: 0.0,
        view_center: [0.0, 1.0e-3],
        plane: [0, 2],
        family: PotentialFamily::V1Superposed,
    },
];

/// The galactic tier's scenes: S2 around Sgr A* read twice, and one the v1 family
/// cannot express at all.
pub const GALACTIC_SCENES: [GravityScene; 3] = [
    GravityScene {
        name: "S2 orbit-averaged, frozen stake",
        plain: "The star S2 going round the black hole at the centre of the galaxy, \
                taken at its average distance.",
        uniform_g_m_s2: 0.0,
        centers: &SGR_A_CENTER,
        // Circular speed at the semi-major axis, sqrt(GM/a) = 1.983e6 m/s. Checked
        // against the orbit rather than trusted: the first value written here was
        // 5.6e6, which is 2.8x too fast, and the only symptom was the scene refusing —
        // a wrong speed reads exactly like a real physics verdict.
        envelope: envelope(S2_SEMI_MAJOR_AU * AU_M, 0.0, 1.9831e6),
        tier_eps_max: GALACTIC_EPS_MAX_FROZEN,
        tolerance: 1.0e-4,
        // GM/a^2. Two hundredths of Earth's g, at nine hundred astronomical units from
        // four million suns.
        local_g_m_s2: 2.710_103_000_479_919_4e-2,
        view_m: 4.0 * S2_SEMI_MAJOR_AU * AU_M,
        // At the semi-major axis, moving across the radius at the circular speed.
        body: Some((
            [S2_SEMI_MAJOR_AU * AU_M, 0.0, 0.0],
            [0.0, 1.9831e6, 0.0],
        )),
        dtau_s: 2.0e5,
        view_center: [0.0, 0.0],
        plane: [0, 1],
        family: PotentialFamily::V1Superposed,
    },
    GravityScene {
        name: "S2 full orbit to perihelion",
        plain: "The same star at its closest approach, where it moves at two and a half \
                percent of the speed of light.",
        uniform_g_m_s2: 0.0,
        centers: &SGR_A_CENTER,
        // Perihelion speed, sqrt(GM(1+e)/(a(1-e))) = 7.849e6 m/s.
        envelope: envelope(
            S2_SEMI_MAJOR_AU * AU_M * (1.0 - S2_ECCENTRICITY),
            0.0,
            7.8494e6,
        ),
        tier_eps_max: GALACTIC_EPS_MAX,
        tolerance: 1.0e-4,
        // GM/r_peri^2, seventy times stronger than at the semi-major axis.
        local_g_m_s2: 1.882_015_972_555_499_7,
        view_m: 4.0 * S2_SEMI_MAJOR_AU * AU_M,
        // At perihelion, moving across the radius at the perihelion speed. The orbit
        // this traces is the whole ellipse, which is why its envelope is the whole one.
        body: Some((
            [S2_SEMI_MAJOR_AU * AU_M * (1.0 - S2_ECCENTRICITY), 0.0, 0.0],
            [0.0, 7.8494e6, 0.0],
        )),
        dtau_s: 5.0e4,
        view_center: [0.0, 0.0],
        plane: [0, 1],
        family: PotentialFamily::V1Superposed,
    },
    GravityScene {
        name: "the disk's flat rotation curve",
        plain: "The galaxy's outer stars, which all circle at the same speed however far \
                out they are. Nothing built from point masses and a uniform pull can \
                produce that, so this chart does not describe it — and says so.",
        // These values describe what is being ASKED ABOUT, not a chart that answers it.
        // The refusal happens before any of them is screened: writing a superposition
        // that roughly fits a flat curve would produce an epsilon about the wrong
        // potential, and a number that looks like a verdict and is about nothing is
        // worse than a refusal.
        uniform_g_m_s2: 0.0,
        centers: &NO_CENTERS,
        envelope: envelope(3.0 * KPC_M, 0.0, 2.2e5),
        tier_eps_max: GALACTIC_EPS_MAX,
        tolerance: 1.0e-4,
        local_g_m_s2: 0.0,
        view_m: 60.0 * KPC_M,
        body: None,
        dtau_s: 0.0,
        view_center: [0.0, 0.0],
        plane: [0, 1],
        // The one scene on the whole ladder refused for what it NEEDS rather than for
        // how big it is.
        family: PotentialFamily::LogarithmicDisk,
    },
];

/// The cosmic tier's scenes: a patch that certifies and one that does not.
pub const COSMIC_SCENES: [GravityScene; 2] = [
    GravityScene {
        name: "30 Mpc comoving patch",
        plain: "A patch of the universe a hundred million light years across — small \
                enough that the expansion does not spoil a static chart.",
        uniform_g_m_s2: 0.0,
        centers: &NO_CENTERS,
        envelope: SceneEnvelope {
            r_min_m: 1.0,
            height_m: 0.0,
            v_max_m_s: 3.0e5,
            patch_m: 30.0 * MPC_M,
            hubble_per_s: HUBBLE_PER_S,
            requires_spacelike_signal: false,
        },
        tier_eps_max: COSMIC_EPS_MAX,
        tolerance: 1.0e-4,
        local_g_m_s2: 0.0,
        view_m: 30.0 * MPC_M,
        // Nothing is thrown: the claim is about whether a STATIC chart can carry this
        // patch at all, and that is answered by the screen, not by a trajectory.
        body: None,
        dtau_s: 0.0,
        view_center: [0.0, 0.0],
        plane: [0, 1],
        family: PotentialFamily::V1Superposed,
    },
    GravityScene {
        name: "100 Mpc comoving patch",
        plain: "Three times wider, and the expansion of space itself is now the biggest \
                term. A static chart cannot carry it.",
        uniform_g_m_s2: 0.0,
        centers: &NO_CENTERS,
        envelope: SceneEnvelope {
            r_min_m: 1.0,
            height_m: 0.0,
            v_max_m_s: 3.0e5,
            patch_m: 100.0 * MPC_M,
            hubble_per_s: HUBBLE_PER_S,
            requires_spacelike_signal: false,
        },
        tier_eps_max: COSMIC_EPS_MAX,
        tolerance: 1.0e-4,
        local_g_m_s2: 0.0,
        view_m: 100.0 * MPC_M,
        body: None,
        dtau_s: 0.0,
        view_center: [0.0, 0.0],
        plane: [0, 1],
        family: PotentialFamily::V1Superposed,
    },
];

/// The scenes a tier declares, or an empty slice for a tier with no gravity chart.
pub fn scenes_for(tier: crate::tier::TierId) -> &'static [GravityScene] {
    match tier {
        crate::tier::TierId::Planet => &PLANET_SCENES,
        crate::tier::TierId::Galactic => &GALACTIC_SCENES,
        crate::tier::TierId::Cosmic => &COSMIC_SCENES,
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciris_sim_core::bridge::WeakFieldRefusal;
    use ciris_sim_core::holon::CertificationStatus;

    /// The planet tier's two readings are two DIFFERENT sentences, and both are honest.
    #[test]
    fn the_planet_reads_certified_above_the_arithmetic_floor_and_flat_below() {
        let thrown = PLANET_SCENES[0].certify();
        assert_eq!(thrown.status, CertificationStatus::Certified);
        assert!(
            (thrown.epsilon / 6.96e-10 - 1.0).abs() < 0.01,
            "Earth's potential should screen at eps ~6.96e-10, got {:e}",
            thrown.epsilon
        );
        assert!(
            (thrown.remainder_bound / 4.85e-18 - 1.0).abs() < 0.02,
            "remainder should be ~4.85e-18, got {:e}",
            thrown.remainder_bound
        );

        let alone = PLANET_SCENES[1].certify();
        assert_eq!(
            alone.status,
            CertificationStatus::GrainFloor,
            "over its own millimetres the ball is below the f64 floor, where the FLAT \
             chart is the licensed answer"
        );
        assert!(alone.refusal.is_none(), "the floor is a licence, not a refusal");
    }

    /// The A1 amendment, shown as an amendment: the same star refuses under the frozen
    /// stake and certifies under the amended one. Neither reading is hidden.
    #[test]
    fn the_full_s2_envelope_refuses_frozen_and_certifies_under_a1() {
        let averaged = GALACTIC_SCENES[0].certify();
        assert_eq!(averaged.status, CertificationStatus::Certified);
        assert!(
            (averaged.epsilon / 4.38e-5 - 1.0).abs() < 0.05,
            "orbit-averaged S2 should screen at ~4.4e-5, got {:e}",
            averaged.epsilon
        );

        let full = GALACTIC_SCENES[1].certify();
        assert_eq!(
            full.status,
            CertificationStatus::Certified,
            "the full envelope certifies under A1's 1e-3"
        );
        assert!(
            full.epsilon > GALACTIC_EPS_MAX_FROZEN,
            "and it must be ABOVE the frozen stake, or the amendment demonstrates \
             nothing: eps {:e} against {GALACTIC_EPS_MAX_FROZEN:e}",
            full.epsilon
        );

        // The same scene screened against the frozen stake must refuse — that is the
        // other half of the amendment and it is checked, not asserted in prose.
        let frozen = certify_weak_field(
            &GALACTIC_SCENES[1].chart(),
            &GALACTIC_SCENES[1].envelope,
            GALACTIC_EPS_MAX_FROZEN,
            GALACTIC_SCENES[1].tolerance,
        );
        assert_eq!(frozen.status, CertificationStatus::RefinementUnavailable);
        assert_eq!(frozen.refusal, Some(WeakFieldRefusal::ExceedsWeakField));
    }

    /// The cosmic refusal is the demo: a patch three times wider refuses BY NAME on the
    /// expansion screen, with the FRW unlock, rather than being quietly absent.
    #[test]
    fn the_oversized_cosmic_patch_refuses_on_the_expansion_screen() {
        let ok = COSMIC_SCENES[0].certify();
        assert_eq!(ok.status, CertificationStatus::Certified);
        assert!(
            (ok.epsilon / 4.55e-5 - 1.0).abs() < 0.05,
            "a 30 Mpc patch should screen at ~4.5e-5, got {:e}",
            ok.epsilon
        );

        let big = COSMIC_SCENES[1].certify();
        assert_eq!(big.status, CertificationStatus::RefinementUnavailable);
        assert_eq!(
            big.refusal,
            Some(WeakFieldRefusal::ExpansionScale),
            "the binding term is the background one, so the refusal must be typed as \
             the FRW gap and not as a generic weak-field failure"
        );
        assert!(
            !big.refusal.unwrap().is_ceiling(),
            "expansion scale is a FLOOR — the FRW family lifts it"
        );
        assert!(big.refusal.unwrap().unlock().contains("FRW"));
    }

    /// The declared orbital speeds are the orbit's, checked against it.
    ///
    /// Both are literals, because `sqrt` is not available in a const context — so they
    /// are checked here instead of trusted. The first value written for the averaged
    /// scene was 2.8x too fast, and its only symptom was that scene refusing: a wrong
    /// speed is indistinguishable from a real verdict, which is exactly why the number
    /// cannot be left unchecked.
    #[test]
    fn the_declared_orbital_speeds_are_the_orbits() {
        let a = S2_SEMI_MAJOR_AU * AU_M;

        let circular = (GM_SGR_A / a).sqrt();
        let declared = GALACTIC_SCENES[0].envelope.v_max_m_s;
        assert!(
            (declared / circular - 1.0).abs() < 1.0e-3,
            "averaged scene declares {declared:e} m/s against a circular speed of \
             {circular:e}"
        );

        let peri_r = a * (1.0 - S2_ECCENTRICITY);
        let peri_v = (GM_SGR_A * (1.0 + S2_ECCENTRICITY) / peri_r).sqrt();
        let declared = GALACTIC_SCENES[1].envelope.v_max_m_s;
        assert!(
            (declared / peri_v - 1.0).abs() < 1.0e-3,
            "perihelion scene declares {declared:e} m/s against {peri_v:e}"
        );
        assert!(
            (GALACTIC_SCENES[1].envelope.r_min_m / peri_r - 1.0).abs() < 1.0e-9,
            "and its r_min must be the perihelion distance"
        );
    }

    /// A scene's declared local gravity must be the gravity its OWN CHART produces.
    ///
    /// This is the gate that would have caught a double count immediately. The planet
    /// scene first declared a uniform 9.80665 alongside Earth's centre — two sources for
    /// one field — and the ball fell at 19.63 m/s^2. Nothing looked wrong: both terms are
    /// individually correct, the certificate still screened at the right epsilon, and the
    /// only tell was a flight time exactly half of the textbook one.
    #[test]
    fn a_scene_declares_the_gravity_its_own_chart_produces() {
        use ciris_sim_core::curvature::StaticWeakFieldChart;
        for scenes in [
            &PLANET_SCENES[..],
            &GALACTIC_SCENES[..],
            &COSMIC_SCENES[..],
        ] {
            for scene in scenes {
                let Some((pos, _)) = scene.body else {
                    continue;
                };
                let grad = scene.chart().grad_potential(&pos);
                let magnitude =
                    (grad[0] * grad[0] + grad[1] * grad[1] + grad[2] * grad[2]).sqrt();
                assert!(
                    (magnitude / scene.local_g_m_s2 - 1.0).abs() < 1.0e-6,
                    "{}: declares a local g of {:e} m/s^2 while its chart produces \
                     {magnitude:e} at the body's start",
                    scene.name,
                    scene.local_g_m_s2
                );
            }
        }
    }

    /// A scene with a body must declare an envelope that body's motion respects.
    ///
    /// The "ball alone" scene originally carried one, and its ball fell twelve
    /// centimetres through an envelope declaring a height of one millimetre — the scene
    /// would have been screened against a guarantee its own motion broke. A scene that
    /// cannot move inside its declaration should not move.
    #[test]
    fn a_scene_with_a_body_can_move_inside_its_own_envelope() {
        for scenes in [
            &PLANET_SCENES[..],
            &GALACTIC_SCENES[..],
            &COSMIC_SCENES[..],
        ] {
            for scene in scenes {
                let Some((_, celerity)) = scene.body else {
                    continue;
                };
                let speed = (celerity[0] * celerity[0]
                    + celerity[1] * celerity[1]
                    + celerity[2] * celerity[2])
                    .sqrt();
                assert!(
                    speed <= scene.envelope.v_max_m_s * 1.001,
                    "{}: launches at {speed:e} m/s against a declared v_max of {:e}",
                    scene.name,
                    scene.envelope.v_max_m_s
                );
                // Ballistic rise, v^2/2g, must fit the declared height — for scenes
                // drawn in a VERTICAL plane, where rising is a thing that happens. An
                // orbit has local gravity too and does not rise; applying the check by
                // the presence of gravity rather than by the declared plane fails S2 for
                // not being a thrown ball.
                if scene.plane[1] == 2 && scene.local_g_m_s2 > 0.0 {
                    let rise = speed * speed / (2.0 * scene.local_g_m_s2);
                    assert!(
                        rise <= scene.envelope.height_m * 1.001,
                        "{}: rises {rise:e} m through a declared height of {:e}",
                        scene.name,
                        scene.envelope.height_m
                    );
                }
            }
        }
    }

    /// The seventh scene: refused for the SHAPE of potential it needs, not its size.
    ///
    /// `UnsupportedPotentialFamily` was wired, typed, and gated from the day the bridge
    /// landed, and no declared scene had ever produced one — a gate that has never fired
    /// is a gate nobody has shown to work. A flat rotation curve is the named case: no
    /// superposition of point masses and a uniform term yields one, so the v1 family
    /// cannot express the claim at all, and the honest answer is to say WHICH family
    /// would rather than to screen it against a chart that does not describe it.
    #[test]
    fn the_flat_rotation_curve_is_refused_for_its_family_not_its_size() {
        let disk = GALACTIC_SCENES[2];
        assert_eq!(disk.family, PotentialFamily::LogarithmicDisk);
        let certificate = disk.certify();
        assert_eq!(certificate.status, CertificationStatus::RefinementUnavailable);
        assert_eq!(
            certificate.refusal,
            Some(WeakFieldRefusal::UnsupportedPotentialFamily)
        );
        assert!(
            certificate
                .refusal
                .unwrap()
                .unlock()
                .contains("v2 logarithmic-potential"),
            "the refusal must name the family that would lift it"
        );
        assert!(
            !certificate.refusal.unwrap().is_ceiling(),
            "the v2 log-potential family lifts this — it is a floor"
        );

        // And the refusal is about the FAMILY, not the numbers: the scene's own envelope
        // would screen perfectly well if v1 could express its potential. Refusing on
        // size when the real reason is shape would name the wrong unlock.
        let as_if_v1 = certify_weak_field(
            &disk.chart(),
            &disk.envelope,
            disk.tier_eps_max,
            disk.tolerance,
        );
        assert_eq!(
            as_if_v1.status,
            CertificationStatus::Certified,
            "the envelope is well inside the screen; only the potential's SHAPE is the \
             problem, which is exactly why the family check runs first"
        );
    }

    /// A gravity chart writes NOTHING into the REG+ ledger's occupancy or momentum
    /// lanes.
    ///
    /// The cap headline in `tier` — three and a half grains of sand, rather than the
    /// 0.59 a full-REG+ chart gets — rests entirely on this crate's charts leaving those
    /// two lanes empty. Adding three tiers that evaluate is exactly the kind of change
    /// that could quietly start writing to them, so the claim is re-checked here rather
    /// than assumed to have survived.
    #[test]
    fn gravity_charts_write_no_occupancy_or_momentum() {
        use crate::sim::Session;
        use crate::tier::TierId;
        for id in [TierId::Planet, TierId::Galactic, TierId::Cosmic] {
            for index in 0..scenes_for(id).len() {
                let mut session = Session::new(id);
                session.set_gravity_scene(index);
                session.throw(0.5, 0.4, 0.6);
                for _ in 0..60 {
                    session.step(1.0 / 60.0);
                }
                for (holon, record) in session.arena().holons().iter().enumerate() {
                    assert_eq!(
                        record.gross.occupancy, 0,
                        "{id:?} scene {index}: holon {holon} wrote occupancy"
                    );
                    assert_eq!(
                        record.gross.momentum,
                        [0, 0],
                        "{id:?} scene {index}: holon {holon} wrote momentum"
                    );
                }
                session.arena().validate().expect("the ledger still composes");
            }
        }
    }

    /// Every refusal in the taxonomy names an unlock, and exactly one is a ceiling.
    ///
    /// A refusal added later without an unlock would be a shortfall wearing a refusal's
    /// clothes; a second ceiling would mean something had been mistyped as invariant
    /// when a different chart would in fact lift it.
    #[test]
    fn every_weak_field_refusal_names_an_unlock_and_one_is_a_ceiling() {
        let all = [
            WeakFieldRefusal::ExceedsWeakField,
            WeakFieldRefusal::ExpansionScale,
            WeakFieldRefusal::UnsupportedPotentialFamily,
            WeakFieldRefusal::RequiresSpacelikeSignal,
            WeakFieldRefusal::Undeclarable,
        ];
        for refusal in all {
            assert!(
                refusal.unlock().len() > 40,
                "{refusal:?} does not name what would lift it"
            );
        }
        let ceilings: Vec<_> = all.into_iter().filter(|r| r.is_ceiling()).collect();
        assert_eq!(
            ceilings,
            vec![WeakFieldRefusal::RequiresSpacelikeSignal],
            "exactly one refusal is invariant under every re-root in the family"
        );
    }

    /// Every declared scene screens to a definite verdict. A scene that comes back
    /// `Undeclarable` is a scene this crate failed to describe, not a physics result.
    #[test]
    fn every_declared_scene_is_declarable() {
        for (tier, scenes) in [
            ("planet", &PLANET_SCENES[..]),
            ("galactic", &GALACTIC_SCENES[..]),
            ("cosmic", &COSMIC_SCENES[..]),
        ] {
            for scene in scenes {
                let certificate = scene.certify();
                assert_ne!(
                    certificate.refusal,
                    Some(WeakFieldRefusal::Undeclarable),
                    "{tier}/{}: the envelope is not a declaration",
                    scene.name
                );
            }
        }
    }
}
