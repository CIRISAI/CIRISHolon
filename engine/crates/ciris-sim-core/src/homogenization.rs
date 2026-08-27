//! Continuum-to-lattice homogenization for the mechanical holon chart.
//!
//! This module closes the narrow form of P2 without inventing another calibration
//! constant.  The visible wall topology is a square grid with one alternating diagonal
//! per cell.  A central-force spring on that stencil has the two-dimensional Cauchy
//! restriction `lambda = mu`, hence `nu = 1/3`: one scalar bond stiffness cannot realize
//! the descriptor's independent `(E, nu)` values.
//!
//! The smallest repair is still relation-local: each live relation carries a normal and
//! a tangential elastic stiffness.  Homogenizing the alternating-diagonal cell under
//! small strain gives, for plane stress,
//!
//! ```text
//! k_n = t (lambda + mu)
//! k_t = t (mu - lambda)
//! lambda = E nu / (1 - nu^2)
//! mu     = E / (2 (1 + nu))
//! ```
//!
//! and the inverse homogenization recovers `E` and `nu` exactly.  `k_t >= 0` requires
//! `nu <= 1/3` for this particular stencil; a material outside that range is a genuine
//! chart misfit and must choose a richer realization rather than accept a negative
//! spring.
//!
//! Fracture adds a separate resolution condition.  Mapping tensile strength to a bond
//! tributary area `A = h t` and fracture energy to the corresponding crack area gives
//! `F_peak = f_t h t` and `G_bond = G_F h t`.  A bilinear cohesive law whose initial
//! stiffness is the homogenized normal stiffness exists only when its failure opening
//! exceeds its peak opening.  The thickness cancels, leaving the explicit grain bound
//!
//! ```text
//! h < 2 G_F (lambda + mu) / f_t^2 .
//! ```
//!
//! A coarser chart is therefore refused, not patched with a hand-tuned cohesive number.

use crate::material::IsotropicMaterial;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaneStressModuli {
    pub lambda_pa: f64,
    pub mu_pa: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LatticeElasticLaw {
    /// Effective out-of-plane thickness represented by one node cell.
    pub thickness_m: f64,
    /// Normal elastic stiffness of every relation.
    pub normal_stiffness_n_m: f64,
    /// Tangential elastic stiffness of every relation.
    pub tangential_stiffness_n_m: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DerivedCohesiveLaw {
    pub peak_force_n: f64,
    pub fracture_energy_j: f64,
    pub opening_at_peak_m: f64,
    pub opening_at_failure_m: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomogenizationError {
    InvalidMaterial,
    InvalidGeometry,
    UnsupportedPoissonRatio,
    CohesiveUnderResolved,
}

/// Plane-stress Lamé moduli of the descriptor.
pub fn plane_stress_moduli(material: &IsotropicMaterial) -> Result<PlaneStressModuli, HomogenizationError> {
    material.validate().map_err(|_| HomogenizationError::InvalidMaterial)?;
    let e = material.young_modulus_pa;
    let nu = material.poisson_ratio;
    let denom = 1.0 - nu * nu;
    if !denom.is_finite() || denom <= 0.0 {
        return Err(HomogenizationError::InvalidMaterial);
    }
    Ok(PlaneStressModuli {
        lambda_pa: e * nu / denom,
        mu_pa: e / (2.0 * (1.0 + nu)),
    })
}

/// Thickness represented by one square node cell, from mass conservation
/// `m_node = rho h^2 t`.
pub fn represented_thickness_m(
    material: &IsotropicMaterial,
    node_mass_kg: f64,
    spacing_m: f64,
) -> Result<f64, HomogenizationError> {
    material.validate().map_err(|_| HomogenizationError::InvalidMaterial)?;
    if !node_mass_kg.is_finite()
        || !spacing_m.is_finite()
        || node_mass_kg <= 0.0
        || spacing_m <= 0.0
    {
        return Err(HomogenizationError::InvalidGeometry);
    }
    let thickness = node_mass_kg / (material.density_kg_m3 * spacing_m * spacing_m);
    if !thickness.is_finite() || thickness <= 0.0 {
        return Err(HomogenizationError::InvalidGeometry);
    }
    Ok(thickness)
}

/// Derive the two relation stiffnesses that homogenize the square + alternating-diagonal
/// lattice exactly to the descriptor's plane-stress `(E, nu)` response.
pub fn derive_lattice_elastic_law(
    material: &IsotropicMaterial,
    node_mass_kg: f64,
    spacing_m: f64,
) -> Result<LatticeElasticLaw, HomogenizationError> {
    let moduli = plane_stress_moduli(material)?;
    let thickness = represented_thickness_m(material, node_mass_kg, spacing_m)?;
    if moduli.mu_pa < moduli.lambda_pa {
        // This stencil would require a negative tangential spring.  Refuse the chart.
        return Err(HomogenizationError::UnsupportedPoissonRatio);
    }
    Ok(LatticeElasticLaw {
        thickness_m: thickness,
        normal_stiffness_n_m: thickness * (moduli.lambda_pa + moduli.mu_pa),
        tangential_stiffness_n_m: thickness * (moduli.mu_pa - moduli.lambda_pa),
    })
}

/// Largest spacing for which the existing triangular/bilinear cohesive shape can carry
/// `(E, nu, f_t, G_F)` while sharing the homogenized normal elastic stiffness.
pub fn max_bilinear_spacing_m(
    material: &IsotropicMaterial,
) -> Result<f64, HomogenizationError> {
    let moduli = plane_stress_moduli(material)?;
    let ft = material.tensile_strength_pa;
    let h = 2.0 * material.fracture_energy_j_m2 * (moduli.lambda_pa + moduli.mu_pa)
        / (ft * ft);
    if !h.is_finite() || h <= 0.0 {
        return Err(HomogenizationError::InvalidMaterial);
    }
    Ok(h)
}

/// Derive the cohesive peak and fracture work at this grain.  A grain at or above the
/// resolution limit is rejected: at that spacing `delta_failure <= delta_peak`, so no
/// positive softening branch exists and any returned number would be papering over P2.
pub fn derive_bilinear_cohesive_law(
    material: &IsotropicMaterial,
    node_mass_kg: f64,
    spacing_m: f64,
) -> Result<DerivedCohesiveLaw, HomogenizationError> {
    let elastic = derive_lattice_elastic_law(material, node_mass_kg, spacing_m)?;
    let h_max = max_bilinear_spacing_m(material)?;
    if spacing_m >= h_max {
        return Err(HomogenizationError::CohesiveUnderResolved);
    }
    let tributary_area = spacing_m * elastic.thickness_m;
    let peak_force = material.tensile_strength_pa * tributary_area;
    let fracture_energy = material.fracture_energy_j_m2 * tributary_area;
    let opening_at_peak = peak_force / elastic.normal_stiffness_n_m;
    let opening_at_failure = 2.0 * fracture_energy / peak_force;
    if !(opening_at_failure > opening_at_peak) {
        return Err(HomogenizationError::CohesiveUnderResolved);
    }
    Ok(DerivedCohesiveLaw {
        peak_force_n: peak_force,
        fracture_energy_j: fracture_energy,
        opening_at_peak_m: opening_at_peak,
        opening_at_failure_m: opening_at_failure,
    })
}

/// Read back the continuum `(E, nu)` implied by a derived lattice law.  This is the
/// executable homogenization certificate: forward and inverse maps must close.
pub fn effective_plane_stress_constants(law: &LatticeElasticLaw) -> (f64, f64) {
    let t = law.thickness_m;
    let lambda = (law.normal_stiffness_n_m - law.tangential_stiffness_n_m) / (2.0 * t);
    let mu = (law.normal_stiffness_n_m + law.tangential_stiffness_n_m) / (2.0 * t);
    let e = 4.0 * mu * (lambda + mu) / (lambda + 2.0 * mu);
    let nu = lambda / (lambda + 2.0 * mu);
    (e, nu)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NODE_MASS_KG: f64 = 0.72;
    const CURRENT_SPACING_M: f64 = 0.245;

    fn relative(a: f64, b: f64) -> f64 {
        (a - b).abs() / b.abs().max(1.0)
    }

    #[test]
    fn two_stiffness_lattice_recovers_descriptor_e_and_nu() {
        let material = IsotropicMaterial::DEMO_CALIBRATION;
        let law = derive_lattice_elastic_law(&material, NODE_MASS_KG, CURRENT_SPACING_M).unwrap();
        let (e, nu) = effective_plane_stress_constants(&law);
        assert!(relative(e, material.young_modulus_pa) < 1.0e-12, "E {e}");
        assert!((nu - material.poisson_ratio).abs() < 1.0e-12, "nu {nu}");
        assert!(law.normal_stiffness_n_m > 1.0e8);
        assert!(law.tangential_stiffness_n_m > 1.0e7);
    }

    #[test]
    fn current_demo_grain_is_correctly_refused_for_bilinear_fracture() {
        let material = IsotropicMaterial::DEMO_CALIBRATION;
        let h_max = max_bilinear_spacing_m(&material).unwrap();
        assert!((h_max - 0.180_921_052_631_578_93).abs() < 1.0e-12, "h_max {h_max}");
        assert_eq!(
            derive_bilinear_cohesive_law(&material, NODE_MASS_KG, CURRENT_SPACING_M),
            Err(HomogenizationError::CohesiveUnderResolved)
        );
    }

    #[test]
    fn finer_grain_has_a_positive_softening_interval() {
        let material = IsotropicMaterial::DEMO_CALIBRATION;
        let law = derive_bilinear_cohesive_law(&material, NODE_MASS_KG, 0.15).unwrap();
        assert!(law.opening_at_failure_m > law.opening_at_peak_m);
        assert!(law.peak_force_n > 0.0);
        assert!(law.fracture_energy_j > 0.0);
    }

    #[test]
    fn stencil_refuses_a_poisson_ratio_that_needs_negative_shear_stiffness() {
        let mut material = IsotropicMaterial::DEMO_CALIBRATION;
        material.poisson_ratio = 0.40;
        assert_eq!(
            derive_lattice_elastic_law(&material, NODE_MASS_KG, CURRENT_SPACING_M),
            Err(HomogenizationError::UnsupportedPoissonRatio)
        );
    }
}
