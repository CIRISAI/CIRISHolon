//! Zero-import WebAssembly Component adapter for `ciris-sim-core`.
//!
//! The adapter is deliberately a control-plane boundary: typed WIT records enter once,
//! become the core's packed runtime arena, and produce a typed certificate. A Bevy or
//! browser renderer should link `ciris-sim-core` directly when simulation occurs every
//! frame, avoiding canonical-ABI copies in the hot loop.

wit_bindgen::generate!({
    world: "ciris-holon",
    path: "wit",
});

use ciris_sim_core::holon::{
    CertificationStatus as CoreStatus, Channels, Decomposition as CoreDecomposition, HolonError,
};
use ciris_sim_core::mechanical::{
    MechanicalError, SphereContactModel, SphereView as CoreSphereView,
};
use ciris_sim_core::regplus::GrossState as CoreGrossState;
use ciris_sim_core::runtime::{
    certify_runtime, RuntimeArenaBuilder, RuntimeHolonSpec, NO_RUNTIME_HOLON,
};

fn holon_error(error: HolonError) -> SimulationError {
    match error {
        HolonError::Empty | HolonError::Capacity => SimulationError::Capacity,
        HolonError::InvalidRoot => SimulationError::InvalidRoot,
        HolonError::MultipleRoots => SimulationError::MultipleRoots,
        HolonError::InvalidParent => SimulationError::InvalidParent,
        HolonError::InvalidDepth => SimulationError::InvalidDepth,
        HolonError::InvalidGrain => SimulationError::InvalidGrain,
        HolonError::InvalidDecomposition => SimulationError::InvalidDecomposition,
        HolonError::GrossStateDoesNotCompose => SimulationError::GrossStateDoesNotCompose,
        HolonError::FrontierDoesNotCoverRoot => SimulationError::FrontierDoesNotCoverRoot,
    }
}

fn mechanical_error(error: MechanicalError) -> SimulationError {
    match error {
        MechanicalError::InvalidHolon => SimulationError::InvalidHolon,
        MechanicalError::InvalidRadius
        | MechanicalError::InvalidMass
        | MechanicalError::NotApproaching
        | MechanicalError::InitiallyOverlapping
        | MechanicalError::InvalidSamples => SimulationError::InvalidContact,
    }
}

fn core_view(view: &SphereView) -> CoreSphereView {
    CoreSphereView {
        root: view.root as usize,
        radius: view.radius,
        center: view.center,
        velocity: view.velocity,
        mass: view.mass,
        sample_phase: view.sample_phase,
    }
}

fn certify_impl(
    scene: Holarchy,
    contact: SphereContact,
    tolerance: Tolerances,
) -> Result<ResolutionCertificate, SimulationError> {
    if !tolerance.macro_error.is_finite()
        || !tolerance.conservation.is_finite()
        || tolerance.macro_error < 0.0
        || tolerance.conservation < 0.0
    {
        return Err(SimulationError::InvalidTolerance);
    }

    let whole_scalars = scene
        .holons
        .iter()
        .try_fold(0_usize, |total, node| total.checked_add(node.whole.len()));
    let mut builder = RuntimeArenaBuilder::with_capacity(
        scene.holons.len(),
        whole_scalars.ok_or(SimulationError::Capacity)?,
    );
    for node in &scene.holons {
        let decomposition = match node.decomposition {
            Decomposition::Leaf => CoreDecomposition::Leaf,
            Decomposition::Latent => CoreDecomposition::Latent,
            Decomposition::Expanded => CoreDecomposition::Expanded,
        };
        builder
            .push(RuntimeHolonSpec {
                parent: node.parent.unwrap_or(NO_RUNTIME_HOLON),
                depth: node.depth,
                grain_units: node.grain_units,
                gross: CoreGrossState::aggregate(
                    node.gross.constituents,
                    node.gross.occupancy,
                    [node.gross.momentum_x, node.gross.momentum_y],
                ),
                whole: &node.whole,
                channels: Channels::from_bits(node.channels),
                boundary: node.boundary,
                decomposition,
            })
            .map_err(holon_error)?;
    }
    let arena = builder.build(scene.root).map_err(holon_error)?;

    let left = core_view(&contact.left);
    let right = core_view(&contact.right);
    let mut model = SphereContactModel::new_runtime(
        &arena,
        left,
        right,
        contact.restitution,
        [contact.normal.x, contact.normal.y, contact.normal.z],
        contact.samples_at_floor as usize,
    )
    .map_err(mechanical_error)?;
    let exact_contact_time = model.exact_contact_time();
    let certificate = certify_runtime(
        &arena,
        &mut model,
        tolerance.macro_error,
        tolerance.conservation,
    );

    let status = match certificate.status {
        CoreStatus::Certified => CertificationStatus::Certified,
        CoreStatus::GrainFloor => CertificationStatus::GrainFloor,
        CoreStatus::RefinementUnavailable => CertificationStatus::RefinementUnavailable,
    };
    let represented_left_grain = certificate.frontier.represented_grain(&arena, left.root);
    let represented_right_grain = certificate.frontier.represented_grain(&arena, right.root);
    let active_frontier = certificate
        .frontier
        .active_indices()
        .map(|node| u32::try_from(node).map_err(|_| SimulationError::Capacity))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ResolutionCertificate {
        status,
        active_frontier,
        represented_left_grain,
        represented_right_grain,
        contact_time: certificate.observables[0],
        exact_contact_time,
        impulse: certificate.observables[1],
        support: certificate.observables[2],
        macro_error_bound: certificate.macro_error_bound,
        conservation_residual: certificate.conservation_residual,
        evaluations: u32::try_from(certificate.evaluations)
            .map_err(|_| SimulationError::Capacity)?,
    })
}

struct Component;

impl Guest for Component {
    fn certify_sphere(
        scene: Holarchy,
        contact: SphereContact,
        tolerance: Tolerances,
    ) -> Result<ResolutionCertificate, SimulationError> {
        certify_impl(scene, contact, tolerance)
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    fn gross(elements: u64) -> GrossState {
        GrossState {
            constituents: elements,
            occupancy: 2 * elements,
            momentum_x: elements as i64,
            momentum_y: 0,
        }
    }

    #[test]
    fn typed_boundary_builds_and_certifies_a_scene() {
        let scene = Holarchy {
            root: 0,
            holons: vec![
                Holon {
                    parent: None,
                    depth: 0,
                    grain_units: 2,
                    gross: gross(12),
                    whole: vec![1.0],
                    channels: Channels::REG_PLUS.union(Channels::MECHANICAL).bits(),
                    boundary: true,
                    decomposition: Decomposition::Expanded,
                },
                Holon {
                    parent: Some(0),
                    depth: 1,
                    grain_units: 1,
                    gross: gross(8),
                    whole: vec![],
                    channels: Channels::REG_PLUS.union(Channels::MECHANICAL).bits(),
                    boundary: true,
                    decomposition: Decomposition::Leaf,
                },
                Holon {
                    parent: Some(0),
                    depth: 1,
                    grain_units: 1,
                    gross: gross(4),
                    whole: vec![],
                    channels: Channels::REG_PLUS.union(Channels::MECHANICAL).bits(),
                    boundary: true,
                    decomposition: Decomposition::Leaf,
                },
            ],
        };
        let contact = SphereContact {
            left: SphereView {
                root: 1,
                radius: 1.0,
                center: -1.2,
                velocity: 0.8,
                mass: 8.0,
                sample_phase: 0.13,
            },
            right: SphereView {
                root: 2,
                radius: 0.75,
                center: 1.0,
                velocity: -0.2,
                mass: 4.0,
                sample_phase: 0.71,
            },
            restitution: 0.8,
            normal: Vec3 {
                x: 0.73,
                y: 0.41,
                z: 0.547,
            },
            samples_at_floor: 8_192,
        };
        let certificate = certify_impl(
            scene,
            contact,
            Tolerances {
                macro_error: 0.001,
                conservation: 1.0e-12,
            },
        )
        .unwrap();
        assert_eq!(certificate.status, CertificationStatus::Certified);
        assert_eq!(certificate.active_frontier, vec![0]);
        assert!(certificate.macro_error_bound <= 0.001);
    }
}
