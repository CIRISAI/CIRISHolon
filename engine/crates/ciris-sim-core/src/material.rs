//! Material traits and breakable relations for mechanical realizations.
//!
//! A material is not a second kind of object. [`MaterialBinding`] says that one holon
//! is interpreted through another, descriptor holon (for example, the holon named
//! "stone") by this mechanical chart. Likewise a connection is itself addressable as
//! a relation holon. This keeps bodies, descriptions, and bonds inside the same recursive
//! identity system while giving the Newtonian evaluator the constitutive numbers it
//! needs.

use crate::holon::NO_HOLON;
use crate::regplus::GrossState;

/// Isotropic small-strain properties read by a mechanical realization.
///
/// Values use SI units. A named material such as stone should normally be a measured,
/// warranted descriptor holon whose properties populate this chart; the illustrative
/// preset below is deliberately not a claim about every kind of stone.
///
/// Dissipation is split into two separately-warranted fields (amendment A5,
/// DESCRIPTOR_CHAIN.md §4.2): `material_damping_ratio` may carry only values with a
/// physics-tier ancestor, while `solver_damping_ratio` is named numerical
/// stabilization with no such ancestor. Restitution is deliberately NOT a field here:
/// it is a pair/velocity/geometry OUTCOME of a contact, so contact charts (for
/// example [`crate::mechanical::SphereContactModel`]) take it per contact pair.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IsotropicMaterial {
    pub density_kg_m3: f64,
    pub young_modulus_pa: f64,
    pub poisson_ratio: f64,
    /// Intrinsic material dissipation as a viscous damping ratio, zeta = 1/(2Q).
    ///
    /// Physically warranted field: only values with a measured ancestry (resonant-bar /
    /// ultrasonic attenuation class benchmarks) belong here. For intrinsic dry granite
    /// the seismic/ultrasonic quality factor is Q ~ 1e2–1e3, so zeta ~ 5e-4–5e-3.
    pub material_damping_ratio: f64,
    /// Numerical stabilization for whatever discrete solver consumes this chart.
    ///
    /// UNWARRANTED BY ANY PHYSICS TIER, and named as such (A5). This is solver
    /// configuration, not a material constant: it has no benchmark ancestor and must
    /// never be exported as a property of the material. It exists so that solver
    /// damping is declared where it acts instead of wearing material-constant costume.
    pub solver_damping_ratio: f64,
    pub tensile_strength_pa: f64,
    pub compressive_strength_pa: f64,
    pub fracture_energy_j_m2: f64,
}

impl IsotropicMaterial {
    /// A demo CALIBRATION, not a specimen: these values match no named published
    /// granite record. The programme's ground-truth granite (Lac du Bonnet, Martin &
    /// Chandler 1994, Int. J. Rock Mech. Min. Sci. 31) measures 200±22 MPa uniaxial
    /// compressive strength against this preset's 95 MPa, so the preset must not be
    /// read as "granite" or as any warranted stone (coherence fix C4,
    /// DESCRIPTOR_CHAIN.md §4.2). It is a transparent set of stone-scale numbers that
    /// exercises the chart. A production descriptor resolves a warranted specimen
    /// record — a descriptor holon carrying Record provenance — before any claim
    /// rides on these values.
    pub const DEMO_CALIBRATION: Self = Self {
        density_kg_m3: 2_650.0,
        young_modulus_pa: 45.0e9,
        poisson_ratio: 0.24,
        // zeta = 1/(2Q) at Q ≈ 250, mid-band of the granite intrinsic Q ~ 1e2–1e3
        // attenuation literature (A5 warrant note above).
        material_damping_ratio: 2.0e-3,
        // The legacy `damping_ratio` 0.055 retained under its honest name: a solver
        // stabilization number with no physics ancestor (it is 1–2 orders more
        // dissipative than intrinsic granite permits).
        solver_damping_ratio: 0.055,
        tensile_strength_pa: 6.0e6,
        compressive_strength_pa: 95.0e6,
        fracture_energy_j_m2: 110.0,
    };

    /// Lac du Bonnet granite (pink; Underground Research Laboratory, Pinawa,
    /// Manitoba) — a PINNED published specimen record (P4), per-field sources below.
    /// This is the programme's ground-truth granite; `DEMO_CALIBRATION` above remains
    /// the honest unpinned stage preset.
    ///
    /// Field warrants:
    /// - density 2638 kg/m3: 164.7 lb/ft3 bulk density, ASTM C97, Cold Spring /
    ///   NBGQA Lac du Bonnet quarry test data.
    /// - E = 66.5 GPa, nu = 0.31: 20-test laboratory averages, Eberhardt, Stead,
    ///   Stimpson & Read 1998, Can. Geotech. J. 35(2):222-233 (as tabulated in
    ///   Szczepanik, Milne, Kostakis & Eberhardt, ISRM 2003, Table I). Note this lab
    ///   nu exceeds the crack-free VRH ~0.227 of DESCRIPTOR_CHAIN §3.4 — it is a
    ///   measured tangent ratio on cracked specimens, quoted as measured.
    /// - tensile 6.9 MPa: DIRECT tension average (Brazilian average is 8.8 MPa),
    ///   "Factors Controlling the Difference in Brazilian and Direct Tensile
    ///   Strengths of the Lac du Bonnet Granite", Rock Mech. Rock Eng. 2019,
    ///   doi:10.1007/s00603-019-01946-x.
    /// - compressive 200 MPa (±22): Martin & Chandler 1994, Int. J. Rock Mech. Min.
    ///   Sci. 31(6):643-659 (Eberhardt et al.'s 206.9 MPa 20-test average is
    ///   consistent).
    /// - fracture energy 30.6 J/m2: DERIVED, not measured — G = K_Ic²(1-nu²)/E at
    ///   K_Ic = 1.5 MPa·√m, the midpoint of the 1.14-1.89 MPa·√m static range
    ///   measured across Barre AND Lac du Bonnet granites (Nasseri et al., Pure
    ///   Appl. Geophys. 163, 2006, doi:10.1007/s00024-006-0064-8 — the accessible
    ///   source does not separate the two rocks). This is the INITIATION-scale
    ///   energy release rate; no Lac-du-Bonnet-specific work-of-fracture G_F was
    ///   found in the accessible literature, and granite-class work-of-fracture
    ///   values run 3-4x higher. The weakest field in this record — flagged, not
    ///   rounded up.
    /// - material_damping_ratio 2.0e-3: CLASS warrant only (granite Q ~ 1e2-1e3,
    ///   zeta = 1/(2Q)); no Lac-du-Bonnet-specific attenuation measurement found.
    /// - solver_damping_ratio 0: solver stabilization is not a specimen property;
    ///   no solver has claimed a value against this record.
    ///
    /// Consequence worth knowing (asserted in test): at these values 2·l_ch =
    /// 2·E·G/f_t² ≈ 0.086 m, so the demo's 0.245 m lattice is OUTSIDE the bilinear
    /// validity domain — `CohesiveLaw::from_continuum` correctly REFUSES this record
    /// at demo spacing. Pinning the specimen does not license the frontier.
    pub const LAC_DU_BONNET: Self = Self {
        density_kg_m3: 2_638.0,
        young_modulus_pa: 66.5e9,
        poisson_ratio: 0.31,
        material_damping_ratio: 2.0e-3,
        solver_damping_ratio: 0.0,
        tensile_strength_pa: 6.9e6,
        compressive_strength_pa: 200.0e6,
        fracture_energy_j_m2: 30.6,
    };

    pub fn validate(&self) -> Result<(), MaterialError> {
        let finite = self.density_kg_m3.is_finite()
            && self.young_modulus_pa.is_finite()
            && self.poisson_ratio.is_finite()
            && self.material_damping_ratio.is_finite()
            && self.solver_damping_ratio.is_finite()
            && self.tensile_strength_pa.is_finite()
            && self.compressive_strength_pa.is_finite()
            && self.fracture_energy_j_m2.is_finite();
        if !finite
            || self.density_kg_m3 <= 0.0
            || self.young_modulus_pa <= 0.0
            || !(-1.0..0.5).contains(&self.poisson_ratio)
            || !(0.0..1.0).contains(&self.material_damping_ratio)
            || self.solver_damping_ratio < 0.0
            || self.tensile_strength_pa <= 0.0
            || self.compressive_strength_pa <= 0.0
            || self.fracture_energy_j_m2 <= 0.0
        {
            return Err(MaterialError::InvalidMaterial);
        }
        Ok(())
    }
}

/// Mechanical interpretation of a "made of" relation. Both ends are ordinary holons.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialBinding {
    /// The body whose mechanical response is being evaluated.
    pub subject_holon: usize,
    /// A descriptor holon such as a warranted sample, grade, or named "stone" profile.
    pub descriptor_holon: usize,
    pub properties: IsotropicMaterial,
}

impl MaterialBinding {
    pub fn validate(&self) -> Result<(), MaterialError> {
        if self.subject_holon == NO_HOLON || self.descriptor_holon == NO_HOLON {
            return Err(MaterialError::InvalidHolon);
        }
        self.properties.validate()
    }
}

/// Bilinear cohesive-zone law for one connection between two holons.
///
/// Stiffness is force per opening, peak force starts softening, and fracture energy is
/// the total work required to fully separate this discrete connection. Lower peak force
/// or fracture energy is the precise meaning of "weakly connected" in this realization.
/// The friction coefficient governs the Coulomb slider on the closed, partially
/// decohered interface (see the regime table on [`CohesiveBond`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CohesiveLaw {
    pub stiffness_n_m: f64,
    pub damping_n_s_m: f64,
    pub peak_force_n: f64,
    pub fracture_energy_j: f64,
    /// Coulomb friction coefficient for the closed interface. Rock-joint values are
    /// Byerlee-class, 0.6–0.85; the T4 spec's McClintock–Walsh inversion of the demo
    /// strength ratio gives 0.74.
    pub friction_coefficient: f64,
}

impl CohesiveLaw {
    pub fn opening_at_peak(&self) -> f64 {
        self.peak_force_n / self.stiffness_n_m
    }

    pub fn opening_at_failure(&self) -> f64 {
        2.0 * self.fracture_energy_j / self.peak_force_n
    }

    pub fn validate(&self) -> Result<(), MaterialError> {
        let finite = self.stiffness_n_m.is_finite()
            && self.damping_n_s_m.is_finite()
            && self.peak_force_n.is_finite()
            && self.fracture_energy_j.is_finite()
            && self.friction_coefficient.is_finite();
        if !finite
            || self.stiffness_n_m <= 0.0
            || self.damping_n_s_m < 0.0
            || self.peak_force_n <= 0.0
            || self.fracture_energy_j <= 0.0
            || self.friction_coefficient < 0.0
            || self.opening_at_failure() <= self.opening_at_peak()
        {
            return Err(MaterialError::InvalidCohesiveLaw);
        }
        Ok(())
    }

    /// Derive this bond's cohesive law from the continuum values of the material it
    /// realizes — T4's homogenization certificate run DOWNWARD (P2). Given a lattice
    /// in which this bond carries attributed cross-section `A` over rest length `L`,
    /// standard cohesive-zone similarity gives, step by step:
    ///
    ///   1. stiffness `k = E·A/L` — the bond is the axial stiffness of its material
    ///      column;
    ///   2. peak force `F = f_t·A` — the column starts failing when its traction
    ///      reaches the tensile strength;
    ///   3. fracture energy `G = G_F·A` — fully separating the column's cross-section
    ///      costs the continuum fracture energy per unit area.
    ///
    /// The law's own accessors then give `δ_peak = f_t·L/E` and `δ_fail = 2·G_F/f_t`,
    /// so the work under the bilinear curve is `½·F·δ_fail = G_F·A` exactly — the
    /// self-consistency gate, mutation-tested.
    ///
    /// Validity: the bilinear shape needs `δ_fail > δ_peak`, i.e. `L < 2·E·G_F/f_t²`
    /// `= 2·ℓ_ch` — the characteristic-length bookkeeping of DESCRIPTOR_CHAIN §3.4. A
    /// coarser lattice is REFUSED here rather than silently snap-backed. (Mesh
    /// OBJECTIVITY wants `L ≤ ℓ_ch/10`; a frontier inside the validity domain but
    /// coarser than that is the C2-recorded GrainFloor status, not a reason to fudge
    /// the law.)
    ///
    /// Dissipation and friction are NOT derived (A5, separately warranted):
    /// `damping_n_s_m` is the caller's solver/regularization value at the scale it
    /// runs at, `friction_coefficient` carries its own Byerlee-class warrant.
    pub fn from_continuum(
        material: &IsotropicMaterial,
        bond_length_m: f64,
        attributed_area_m2: f64,
        damping_n_s_m: f64,
        friction_coefficient: f64,
    ) -> Result<Self, MaterialError> {
        material.validate()?;
        if !bond_length_m.is_finite()
            || bond_length_m <= 0.0
            || !attributed_area_m2.is_finite()
            || attributed_area_m2 <= 0.0
        {
            return Err(MaterialError::InvalidCohesiveLaw);
        }
        let law = Self {
            stiffness_n_m: material.young_modulus_pa * attributed_area_m2 / bond_length_m,
            damping_n_s_m,
            peak_force_n: material.tensile_strength_pa * attributed_area_m2,
            fracture_energy_j: material.fracture_energy_j_m2 * attributed_area_m2,
            friction_coefficient,
        };
        law.validate()?;
        Ok(law)
    }

    /// A NAMED similarity map for stage use (the A5 idiom applied to the whole law):
    /// forces scale by `force_scale`, openings by `opening_scale`, so stiffness maps
    /// by `force/opening`, peak by `force`, and fracture energy by `force·opening` —
    /// the same constitutive curve with relabeled axes. The G_F bookkeeping survives
    /// in scaled units: work per (area · force_scale · opening_scale) still equals
    /// the continuum G_F, and the brittleness ratio `δ_fail/δ_peak = 2ℓ_ch/L` is
    /// invariant. Dissipation is NOT mapped — it is solver configuration declared at
    /// the scale where it acts (A5); friction is dimensionless and invariant.
    pub fn stage_scaled(self, force_scale: f64, opening_scale: f64) -> Self {
        Self {
            stiffness_n_m: self.stiffness_n_m * force_scale / opening_scale,
            damping_n_s_m: self.damping_n_s_m,
            peak_force_n: self.peak_force_n * force_scale,
            fracture_energy_j: self.fracture_energy_j * force_scale * opening_scale,
            friction_coefficient: self.friction_coefficient,
        }
    }

    /// Scale resistance without changing the elastic stiffness of the adjoining matter.
    pub const fn weakened(self, strength: f64, toughness: f64) -> Self {
        Self {
            stiffness_n_m: self.stiffness_n_m,
            damping_n_s_m: self.damping_n_s_m,
            peak_force_n: self.peak_force_n * strength,
            fracture_energy_j: self.fracture_energy_j * toughness,
            friction_coefficient: self.friction_coefficient,
        }
    }
}

/// Stateful relation holon. Damage is irreversible: zero is intact and one is a crack.
///
/// # Interface ownership per (damage, contact-state) regime — amendment A3
///
/// Exactly ONE owner and ONE law govern each regime; there is no regime in which two
/// laws in two places act on the same interface, and the D = 1 handoff changes the
/// owner without a jump in the constitutive response:
///
/// | damage D  | contact state         | owner          | law |
/// |-----------|-----------------------|----------------|-----|
/// | D < 1     | open (extension ≥ 0)  | `CohesiveBond` | bilinear cohesive traction `(1−D)·(K·δ + c·δ̇)`, damage monotone in maximum opening |
/// | D < 1     | closed (extension < 0)| `CohesiveBond` | crack closure: FULL-stiffness compression `K·δ + c·δ̇` (compression is not degraded by opening-driven damage — closed crack faces bear load), plus a Coulomb slider capped at `D·μ·|F_n|` |
/// | D = 1     | any                   | contact solver | the bond returns zero forever; unilateral contact and Coulomb friction between the now-free bodies belong to the contact solver, under the handoff CONTRACT below |
///
/// The slider cap `D·μ·|F_n|` is the single-owner resolution of the previously
/// ambiguous 0 < D < 1 closed regime: it vanishes at D = 0 (a fully bonded interface
/// has no failed surface to slide on — tangential resistance there is carried by the
/// elastic network), and it tends to the contact solver's Coulomb capacity `μ·|F_n|`
/// as D → 1, so the ownership transfer at full failure is continuous. Damage `D` and
/// `maximum_opening_m` are Record-axis: irreversible history of the relation.
///
/// Jurisdiction corollary (the standard DEM choice, stated so no pair has two
/// owners): solver contact applies ONLY to pairs not joined by a live (D < 1) bond —
/// while the bond exists it owns the closed regime above, so bonded pairs are exempt
/// from solver contact, and the solver's jurisdiction is exactly the union of fully
/// failed (D = 1) pairs and never-bonded pairs.
///
/// D = 1 row CONTRACT (force continuity across the handoff): the contact solver's
/// Coulomb capacity on a failed interface must START at exactly the `μ·|F_n|` the
/// live slider tends to as D → 1, with the same viscous regularization —
/// [`Self::failed_contact_friction_force`] IS that limit, reading the dead
/// relation's own law. In the normal direction continuity holds by construction:
/// damage advances only while OPEN, so D reaches 1 exactly where the cohesive
/// traction has softened to zero and the separation exceeds contact range — the two
/// normal laws never both act at the crossing. A hidden jump at this seam would
/// masquerade as fracture dynamics, which is why both halves of the contract are
/// gated by tests, not just stated.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CohesiveBond {
    pub relation_holon: usize,
    pub a_holon: usize,
    pub b_holon: usize,
    pub rest_length_m: f64,
    pub law: CohesiveLaw,
    damage: f64,
    maximum_opening_m: f64,
}

impl CohesiveBond {
    pub fn new(
        relation_holon: usize,
        a_holon: usize,
        b_holon: usize,
        rest_length_m: f64,
        law: CohesiveLaw,
    ) -> Result<Self, MaterialError> {
        if relation_holon == NO_HOLON
            || a_holon == NO_HOLON
            || b_holon == NO_HOLON
            || a_holon == b_holon
        {
            return Err(MaterialError::InvalidHolon);
        }
        if !rest_length_m.is_finite() || rest_length_m <= 0.0 {
            return Err(MaterialError::InvalidRestLength);
        }
        law.validate()?;
        Ok(Self {
            relation_holon,
            a_holon,
            b_holon,
            rest_length_m,
            law,
            damage: 0.0,
            maximum_opening_m: 0.0,
        })
    }

    pub const fn damage(&self) -> f64 {
        self.damage
    }

    pub const fn is_broken(&self) -> bool {
        self.damage >= 1.0
    }

    pub const fn maximum_opening_m(&self) -> f64 {
        self.maximum_opening_m
    }

    /// Advance irreversible damage and return the axial force magnitude along the bond
    /// direction (positive = tension pulling the endpoints together).
    ///
    /// This implements the normal-direction column of the regime table above: open
    /// bonds soften bilinearly, closed bonds carry FULL-stiffness compression (crack
    /// closure), and a fully broken relation returns zero — post-failure contact
    /// belongs to the contact solver, not to the failed bond.
    pub fn axial_force(&mut self, extension_m: f64, relative_speed_m_s: f64) -> f64 {
        if !extension_m.is_finite() || !relative_speed_m_s.is_finite() || self.is_broken() {
            return 0.0;
        }

        if extension_m < 0.0 {
            // CLOSED regime, bond owner: opening-driven damage does not degrade
            // compression, because the closed crack faces are back in contact.
            return self.law.stiffness_n_m * extension_m
                + self.law.damping_n_s_m * relative_speed_m_s;
        }

        let opening = extension_m;
        self.maximum_opening_m = self.maximum_opening_m.max(opening);
        let peak = self.law.opening_at_peak();
        let failure = self.law.opening_at_failure();
        if self.maximum_opening_m >= failure {
            self.damage = 1.0;
            return 0.0;
        }
        if self.maximum_opening_m > peak {
            // Standard bilinear cohesive damage: (1-D) K delta follows the descending
            // line from peak traction to zero at the failure opening.
            let target = failure * (self.maximum_opening_m - peak)
                / (self.maximum_opening_m * (failure - peak));
            self.damage = self.damage.max(target.clamp(0.0, 1.0));
        }

        let intact = 1.0 - self.damage;
        intact
            * (self.law.stiffness_n_m * extension_m + self.law.damping_n_s_m * relative_speed_m_s)
    }

    /// Coulomb slider on the closed, partially decohered interface (A3 regime table,
    /// tangential column). Returns a force magnitude ≥ 0 to be applied OPPOSITE the
    /// tangential sliding direction.
    ///
    /// `axial_force_n` is the value just returned by [`Self::axial_force`]; the slider
    /// engages only while it is compressive (the interface is closed). The capacity is
    /// `D·μ·|F_n|` — zero on an intact bond, the full contact-solver Coulomb capacity
    /// in the D → 1 limit — and the slider is viscosity-regularized for explicit
    /// integration using the law's damping coefficient: `min(c·|v_t|, D·μ·|F_n|)`.
    /// A broken bond returns zero: friction between separated bodies is owned by the
    /// contact solver.
    pub fn closed_friction_force(&self, axial_force_n: f64, tangential_speed_m_s: f64) -> f64 {
        if !axial_force_n.is_finite()
            || !tangential_speed_m_s.is_finite()
            || self.is_broken()
            || axial_force_n >= 0.0
        {
            return 0.0;
        }
        let capacity = self.damage * self.law.friction_coefficient * (-axial_force_n);
        let viscous = self.law.damping_n_s_m * tangential_speed_m_s.abs();
        viscous.min(capacity)
    }

    /// The contact solver's Coulomb friction on a FULLY FAILED interface — the D = 1
    /// row's handoff contract made executable. The crack face inherits the dead
    /// relation's tribology: capacity `μ·|F_n|` with the same viscous regularization
    /// as the live slider, so this is EXACTLY the D → 1 limit of
    /// [`Self::closed_friction_force`] — no force discontinuity hides at the moment
    /// of full failure. `contact_normal_force_n` is the contact solver's compressive
    /// push magnitude (≥ 0). Returns zero unless the bond is broken: while the bond
    /// lives, its own slider owns the tangential channel.
    pub fn failed_contact_friction_force(
        &self,
        contact_normal_force_n: f64,
        tangential_speed_m_s: f64,
    ) -> f64 {
        if !contact_normal_force_n.is_finite()
            || !tangential_speed_m_s.is_finite()
            || !self.is_broken()
            || contact_normal_force_n <= 0.0
        {
            return 0.0;
        }
        let capacity = self.law.friction_coefficient * contact_normal_force_n;
        let viscous = self.law.damping_n_s_m * tangential_speed_m_s.abs();
        viscous.min(capacity)
    }
}

/// Damage/Record tag carried by every rigid export: the summary of the relation
/// network's irreversible history. Damage and maximum openings are Record-axis, so
/// this tag is what keeps the past provable across the tier boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageRecord {
    pub bond_count: u32,
    pub broken_count: u32,
    pub mean_damage: f64,
    pub max_damage: f64,
    pub total_maximum_opening_m: f64,
}

impl DamageRecord {
    pub fn from_bonds<'a>(bonds: impl IntoIterator<Item = &'a CohesiveBond>) -> Self {
        let mut bond_count = 0u32;
        let mut broken_count = 0u32;
        let mut damage_sum = 0.0f64;
        let mut max_damage = 0.0f64;
        let mut total_maximum_opening_m = 0.0f64;
        for bond in bonds {
            bond_count += 1;
            if bond.is_broken() {
                broken_count += 1;
            }
            damage_sum += bond.damage();
            max_damage = max_damage.max(bond.damage());
            total_maximum_opening_m += bond.maximum_opening_m();
        }
        let mean_damage = if bond_count == 0 {
            0.0
        } else {
            damage_sum / bond_count as f64
        };
        Self {
            bond_count,
            broken_count,
            mean_damage,
            max_damage,
            total_maximum_opening_m,
        }
    }
}

/// The engine's OWN rigid chart over a bonded holon — amendment A4.
///
/// When a holon is healthy enough to be stepped as a rigid body, the export is THIS
/// chart over the SAME holon: the wall never stops being a holon by being certified
/// healthy. Rapier (or any external rigid-body engine) remains a limiting-case
/// CONTROL to compare against, never the definition of the export.
///
/// The tuple carries a [`DamageRecord`] tag. This instantiates
/// `repairable_does_not_factor` at the T4/T5 seam: a pristine wall and one pre-loaded
/// to 0.9 of critical (all D < 1) agree in every current-configuration view — same
/// mass, same geometry, empty crack observable `{r | D = 1}` — yet whether the
/// loading past can still be proven downstream depends on what the export retains.
/// Dropping the tag would make the two walls export identically, erasing the
/// provability of history at the tier boundary (misfit L18, repaired here).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigidChartExport {
    pub subject_holon: usize,
    /// The holon's additive REG+ ledger, carried EXPLICITLY: a chart over a holon
    /// keeps the holon's identity, and the MEET-2 seam gates ledger identity exactly
    /// across every chart handoff.
    pub gross: GrossState,
    pub mass_kg: f64,
    pub record: DamageRecord,
}

impl RigidChartExport {
    pub fn over<'a>(
        subject_holon: usize,
        gross: GrossState,
        mass_kg: f64,
        bonds: impl IntoIterator<Item = &'a CohesiveBond>,
    ) -> Result<Self, MaterialError> {
        if subject_holon == NO_HOLON {
            return Err(MaterialError::InvalidHolon);
        }
        if !mass_kg.is_finite() || mass_kg <= 0.0 {
            return Err(MaterialError::InvalidMaterial);
        }
        Ok(Self {
            subject_holon,
            gross,
            mass_kg,
            record: DamageRecord::from_bonds(bonds),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialError {
    InvalidHolon,
    InvalidMaterial,
    InvalidCohesiveLaw,
    InvalidRestLength,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strong_law() -> CohesiveLaw {
        CohesiveLaw {
            stiffness_n_m: 1_000.0,
            damping_n_s_m: 2.0,
            peak_force_n: 10.0,
            fracture_energy_j: 0.1,
            friction_coefficient: 0.74,
        }
    }

    #[test]
    fn stone_is_a_valid_mechanical_chart() {
        let binding = MaterialBinding {
            subject_holon: 7,
            descriptor_holon: 11,
            properties: IsotropicMaterial::DEMO_CALIBRATION,
        };
        assert_eq!(binding.validate(), Ok(()));
    }

    #[test]
    fn material_dissipation_sits_in_the_intrinsic_rock_band() {
        // A5: the physically warranted field must carry a value in the band its own
        // warrant states (granite Q ~ 1e2–1e3, zeta = 1/(2Q)), and the legacy solver
        // number must live outside that band under its honest name.
        let preset = IsotropicMaterial::DEMO_CALIBRATION;
        assert!((5.0e-4..=5.0e-3).contains(&preset.material_damping_ratio));
        assert!(preset.solver_damping_ratio > 5.0e-3);
        assert_eq!(preset.validate(), Ok(()));
    }

    #[test]
    fn weak_relation_fails_before_strong_relation() {
        let strong = strong_law();
        let weak = strong.weakened(0.4, 0.2);
        let mut strong_bond = CohesiveBond::new(20, 1, 2, 1.0, strong).unwrap();
        let mut weak_bond = CohesiveBond::new(21, 1, 2, 1.0, weak).unwrap();

        weak_bond.axial_force(0.011, 0.0);
        strong_bond.axial_force(0.011, 0.0);

        assert!(weak_bond.is_broken());
        assert!(!strong_bond.is_broken());
    }

    #[test]
    fn damage_is_irreversible() {
        let mut bond = CohesiveBond::new(20, 1, 2, 1.0, strong_law()).unwrap();
        bond.axial_force(0.015, 0.0);
        let damaged = bond.damage();
        bond.axial_force(0.0, 0.0);
        assert!(damaged > 0.0 && damaged < 1.0);
        assert_eq!(bond.damage(), damaged);
    }

    #[test]
    fn partially_damaged_closed_bond_has_one_owner() {
        // A3: the D ∈ (0,1) closed regime that was previously ambiguous. The bond is
        // the single owner: full-stiffness crack-closure compression plus a Coulomb
        // slider capped at D·μ·|F_n|. At D = 1 both channels return zero because the
        // contact solver takes ownership.
        let law = strong_law();
        let mut bond = CohesiveBond::new(20, 1, 2, 1.0, law).unwrap();
        bond.axial_force(0.015, 0.0);
        let damage = bond.damage();
        assert!(damage > 0.0 && damage < 1.0);

        // Crack closure: compression carries FULL stiffness, not (1-D)-scaled.
        let compressive = bond.axial_force(-0.004, 0.0);
        assert_eq!(compressive, -0.004 * law.stiffness_n_m);
        assert_eq!(bond.damage(), damage, "closure must not advance damage");

        // The slider saturates at the Coulomb capacity D·μ·|F_n| ...
        let capacity = damage * law.friction_coefficient * compressive.abs();
        let sliding = bond.closed_friction_force(compressive, 1.0e3);
        assert!((sliding - capacity).abs() <= 1.0e-12 * capacity);
        // ... is viscosity-regularized below it ...
        let creeping = bond.closed_friction_force(compressive, 1.0e-6);
        assert!(creeping > 0.0 && creeping < capacity);
        // ... and never engages in tension (the interface is open, cohesion owns it).
        assert_eq!(bond.closed_friction_force(5.0, 1.0e3), 0.0);

        // An intact bond has no failed surface to slide on: capacity is zero at D = 0.
        let intact = CohesiveBond::new(21, 1, 2, 1.0, law).unwrap();
        assert_eq!(intact.closed_friction_force(-4.0, 1.0e3), 0.0);

        // At D = 1 the bond owns nothing; post-failure contact and friction belong to
        // the contact solver.
        let mut broken = CohesiveBond::new(22, 1, 2, 1.0, law).unwrap();
        broken.axial_force(1.0, 0.0);
        assert!(broken.is_broken());
        assert_eq!(broken.axial_force(-0.004, 0.0), 0.0);
        assert_eq!(broken.closed_friction_force(-4.0, 1.0e3), 0.0);
    }

    #[test]
    fn lac_du_bonnet_record_is_valid_and_refused_at_demo_spacing() {
        // P4: the pinned specimen record validates, differs from the stage preset,
        // and — because its initiation-scale fracture energy makes it far more
        // brittle — is correctly REFUSED by the derivation at the demo's 0.245 m
        // spacing (2*l_ch ~ 0.086 m). A pinned record does not license a frontier.
        let record = IsotropicMaterial::LAC_DU_BONNET;
        assert_eq!(record.validate(), Ok(()));
        assert_ne!(record, IsotropicMaterial::DEMO_CALIBRATION);
        let l_ch = record.young_modulus_pa * record.fracture_energy_j_m2
            / (record.tensile_strength_pa * record.tensile_strength_pa);
        assert!(2.0 * l_ch < 0.245);
        assert_eq!(
            CohesiveLaw::from_continuum(&record, 0.245, 0.245 * 0.245, 2.6, 0.74),
            Err(MaterialError::InvalidCohesiveLaw)
        );
        // Inside its own validity domain the derivation accepts the record.
        assert!(CohesiveLaw::from_continuum(&record, l_ch / 10.0, 0.01, 2.6, 0.74).is_ok());
    }

    #[test]
    fn derived_law_reproduces_the_continuum() {
        // P2 self-consistency gates. E and f_t round-trip through the derivation, and
        // the work under the bilinear curve per unit attributed area equals G_F —
        // both mutation-tested against attribution perturbations.
        let material = IsotropicMaterial::DEMO_CALIBRATION;
        let length = 0.245;
        let area = 0.245 * 0.245;
        let law = CohesiveLaw::from_continuum(&material, length, area, 2.6, 0.74).unwrap();

        let young = law.stiffness_n_m * length / area;
        assert!((young - material.young_modulus_pa).abs() <= 1.0e-9 * material.young_modulus_pa);
        let tensile = law.peak_force_n / area;
        assert!(
            (tensile - material.tensile_strength_pa).abs()
                <= 1.0e-9 * material.tensile_strength_pa
        );
        let work_per_area = 0.5 * law.peak_force_n * law.opening_at_failure() / area;
        assert!(
            (work_per_area - material.fracture_energy_j_m2).abs()
                <= 1.0e-9 * material.fracture_energy_j_m2,
            "bilinear work {work_per_area} J/m2 must equal continuum G_F"
        );

        // Planted-error control (the attribution-perturbation gate, permanent): a law
        // whose energy attribution is off by 10% must FAIL the work-per-area
        // identity, and one whose stiffness attribution is off must fail the E
        // round-trip — the gates can fire. (Source-level mutations of
        // from_continuum's three attribution lines were also run and fired; see the
        // P2 report.)
        let mis_energy = CohesiveLaw {
            fracture_energy_j: law.fracture_energy_j * 0.9,
            ..law
        };
        let mis_work = 0.5 * mis_energy.peak_force_n * mis_energy.opening_at_failure() / area;
        assert!(
            (mis_work - material.fracture_energy_j_m2).abs()
                > 0.05 * material.fracture_energy_j_m2,
            "the G_F gate cannot fire on a mis-attributed energy"
        );
        let mis_stiffness = CohesiveLaw {
            stiffness_n_m: law.stiffness_n_m * 1.1,
            ..law
        };
        assert!(
            (mis_stiffness.stiffness_n_m * length / area - material.young_modulus_pa).abs()
                > 0.05 * material.young_modulus_pa,
            "the E gate cannot fire on a mis-attributed stiffness"
        );

        // The similarity map conserves the G_F bookkeeping in scaled units and the
        // brittleness ratio exactly.
        let scaled = law.stage_scaled(3.0e-5, 700.0);
        let scaled_work = 0.5 * scaled.peak_force_n * scaled.opening_at_failure();
        assert!(
            (scaled_work / (area * 3.0e-5 * 700.0) - material.fracture_energy_j_m2).abs()
                <= 1.0e-9 * material.fracture_energy_j_m2
        );
        let ratio = law.opening_at_failure() / law.opening_at_peak();
        let scaled_ratio = scaled.opening_at_failure() / scaled.opening_at_peak();
        assert!((ratio - scaled_ratio).abs() <= 1.0e-9 * ratio);

        // Validity: a lattice coarser than 2 l_ch is refused, never fudged. The demo
        // geometry sits 12% inside the domain; its sqrt(2) diagonal is OUTSIDE.
        let l_ch = material.young_modulus_pa * material.fracture_energy_j_m2
            / (material.tensile_strength_pa * material.tensile_strength_pa);
        assert!(length < 2.0 * l_ch && length * core::f64::consts::SQRT_2 > 2.0 * l_ch);
        assert_eq!(
            CohesiveLaw::from_continuum(&material, 2.1 * l_ch, area, 2.6, 0.74),
            Err(MaterialError::InvalidCohesiveLaw)
        );
        assert_eq!(
            CohesiveLaw::from_continuum(
                &material,
                length * core::f64::consts::SQRT_2,
                area,
                2.6,
                0.74
            ),
            Err(MaterialError::InvalidCohesiveLaw)
        );
    }

    #[test]
    fn force_is_continuous_across_the_failure_handoff() {
        // D = 1 row contract, trajectory half: drive a bond through failure in small
        // opening increments and assert the interface force never jumps by more than
        // the bilinear slope allows — the crossing into contact-solver ownership must
        // carry no hidden discontinuity that would masquerade as fracture dynamics.
        let law = strong_law();
        let mut bond = CohesiveBond::new(20, 1, 2, 1.0, law).unwrap();
        let failure = law.opening_at_failure();
        let step = 1.0e-6;
        // The steepest legitimate slope is max(loading K, softening peak/(failure-peak)).
        let softening_slope = law.peak_force_n / (failure - law.opening_at_peak());
        let slope_bound = law.stiffness_n_m.max(softening_slope);

        let mut extension = 0.0;
        let mut previous = 0.0;
        let mut crossing_jump = f64::NAN;
        while extension < failure + 10.0 * step {
            extension += step;
            let was_broken = bond.is_broken();
            let force = bond.axial_force(extension, 0.0);
            if !was_broken && bond.is_broken() {
                crossing_jump = (force - previous).abs();
                // Both tangential channels are zero at the crossing: the interface is
                // open, so neither the live slider nor the failed-contact law acts.
                assert_eq!(bond.closed_friction_force(force, 1.0e3), 0.0);
                assert_eq!(bond.failed_contact_friction_force(-force, 1.0e3), 0.0);
            }
            previous = force;
        }
        assert!(bond.is_broken(), "the ramp must actually cross D = 1");
        assert!(
            crossing_jump <= 2.0 * slope_bound * step,
            "force discontinuity {crossing_jump} N at the D = 1 handoff \
             (slope-allowed {})",
            2.0 * slope_bound * step
        );
    }

    #[test]
    fn slider_capacity_meets_the_contact_solver_at_full_failure() {
        // D = 1 row contract, constitutive half: at a fixed closed, sliding
        // configuration the live slider's capacity D·μ·|N| must tend to the failed
        // interface's contact Coulomb capacity μ·|N| as D → 1⁻, and the two laws
        // must be identical in the viscous-regularized regime.
        let law = strong_law();
        let failure = law.opening_at_failure();

        let mut nearly = CohesiveBond::new(20, 1, 2, 1.0, law).unwrap();
        nearly.axial_force(failure * (1.0 - 1.0e-9), 0.0);
        let d = nearly.damage();
        assert!(d < 1.0 && 1.0 - d < 1.0e-6);

        let mut broken = CohesiveBond::new(21, 1, 2, 1.0, law).unwrap();
        broken.axial_force(failure + 1.0, 0.0);
        assert!(broken.is_broken());

        let normal = 40.0;
        // Saturated regime: the capacities differ by exactly (1 - D)·μ·|N|.
        let live = nearly.closed_friction_force(-normal, 1.0e3);
        let failed = broken.failed_contact_friction_force(normal, 1.0e3);
        assert_eq!(failed, law.friction_coefficient * normal);
        assert!(
            (live - failed).abs() <= (1.0 - d) * law.friction_coefficient * normal + 1.0e-12,
            "capacity gap {} exceeds the (1-D) margin",
            (live - failed).abs()
        );
        // Viscous regime: below both caps the regularized laws are bit-identical.
        assert_eq!(
            nearly.closed_friction_force(-normal, 1.0e-6),
            broken.failed_contact_friction_force(normal, 1.0e-6)
        );
    }

    #[test]
    fn subcritical_damage_reaches_the_rigid_export() {
        // A4: a pristine wall and one pre-loaded to 0.9 of critical (all D < 1) must
        // export DIFFERENTLY, and the difference must live entirely in the Record tag
        // — `repairable_does_not_factor` at the T4/T5 seam.
        let law = strong_law();
        let pristine = [
            CohesiveBond::new(20, 1, 2, 1.0, law).unwrap(),
            CohesiveBond::new(21, 2, 3, 1.0, law).unwrap(),
        ];
        let mut loaded = pristine;
        let peak = law.opening_at_peak();
        let failure = law.opening_at_failure();
        loaded[0].axial_force(peak + 0.9 * (failure - peak), 0.0);
        assert!(loaded[0].damage() > 0.0 && loaded[0].damage() < 1.0);

        let gross = GrossState::aggregate(1_000, 0, [0, 0]);
        let pristine_export = RigidChartExport::over(2, gross, 100.0, pristine.iter()).unwrap();
        let loaded_export = RigidChartExport::over(2, gross, 100.0, loaded.iter()).unwrap();

        // Every current-configuration view agrees ...
        assert_eq!(loaded_export.subject_holon, pristine_export.subject_holon);
        assert_eq!(loaded_export.mass_kg, pristine_export.mass_kg);
        assert_eq!(loaded_export.record.broken_count, 0);
        assert_eq!(
            loaded_export.record.bond_count,
            pristine_export.record.bond_count
        );
        // ... and the exports still differ, through the Record alone.
        assert_ne!(loaded_export, pristine_export);
        assert!(loaded_export.record.mean_damage > 0.0);
        assert!(loaded_export.record.max_damage < 1.0);

        // Determinism: re-exporting the same state is bit-identical.
        assert_eq!(
            RigidChartExport::over(2, gross, 100.0, pristine.iter()).unwrap(),
            pristine_export
        );
    }
}
