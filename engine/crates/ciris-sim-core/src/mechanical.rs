//! A classical-mechanical realization over recursive holons.
//!
//! This is one evaluator, not a privileged holon type. It treats two selected
//! subholons as macroscopic spheres: REG+ gross state supplies composition, while a
//! progressively sampled sphere boundary supplies contact time. The contact impulse is
//! the exact unequal-mass restitution law. The resulting error bound is an analytic
//! example of [`crate::holon::BoundaryModel`]: only the boundary representation is
//! refined, while arbitrarily large latent interiors remain aggregated.

use crate::holon::{BoundaryModel, Evaluation, Frontier, HolonArena};
#[cfg(feature = "alloc")]
use crate::runtime::{RuntimeArena, RuntimeBoundaryModel, RuntimeFrontier};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphereView {
    pub root: usize,
    pub radius: f64,
    /// Centre coordinate along the contact normal.
    pub center: f64,
    /// Signed velocity along the contact normal.
    pub velocity: f64,
    pub mass: f64,
    /// A deterministic orientation offset for the boundary sampling lattice.
    pub sample_phase: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MechanicalError {
    InvalidHolon,
    InvalidRadius,
    InvalidMass,
    NotApproaching,
    InitiallyOverlapping,
    InvalidSamples,
}

/// Gross unequal-mass impact plus an adaptively refined sphere boundary.
///
/// Restitution is supplied per contact pair: it is a pair/velocity/geometry OUTCOME
/// of a collision, not a material constant, which is why it is deliberately not a
/// field of [`crate::material::IsotropicMaterial`] (amendment A5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphereContactModel {
    pub left: SphereView,
    pub right: SphereView,
    pub restitution: f64,
    normal: [f64; 3],
    samples_at_floor: usize,
}

impl SphereContactModel {
    pub fn new<const CAP: usize, const W: usize>(
        arena: &HolonArena<CAP, W>,
        left: SphereView,
        right: SphereView,
        restitution: f64,
        normal: [f64; 3],
        samples_at_floor: usize,
    ) -> Result<Self, MechanicalError> {
        if arena.holon(left.root).is_none() || arena.holon(right.root).is_none() {
            return Err(MechanicalError::InvalidHolon);
        }
        Self::checked(left, right, restitution, normal, samples_at_floor)
    }

    /// Construct the same realization over the runtime-sized arena used by Rust/WASM
    /// hosts. Solver mathematics is shared with the allocator-free path.
    #[cfg(feature = "alloc")]
    pub fn new_runtime(
        arena: &RuntimeArena,
        left: SphereView,
        right: SphereView,
        restitution: f64,
        normal: [f64; 3],
        samples_at_floor: usize,
    ) -> Result<Self, MechanicalError> {
        if arena.holon(left.root).is_none() || arena.holon(right.root).is_none() {
            return Err(MechanicalError::InvalidHolon);
        }
        Self::checked(left, right, restitution, normal, samples_at_floor)
    }

    fn checked(
        left: SphereView,
        right: SphereView,
        restitution: f64,
        normal: [f64; 3],
        samples_at_floor: usize,
    ) -> Result<Self, MechanicalError> {
        if !left.radius.is_finite()
            || !right.radius.is_finite()
            || left.radius <= 0.0
            || right.radius <= 0.0
        {
            return Err(MechanicalError::InvalidRadius);
        }
        let inverse_mass_sum = 1.0 / left.mass + 1.0 / right.mass;
        if !left.mass.is_finite()
            || !right.mass.is_finite()
            || left.mass <= 0.0
            || right.mass <= 0.0
            || !inverse_mass_sum.is_finite()
            || inverse_mass_sum <= 0.0
        {
            return Err(MechanicalError::InvalidMass);
        }
        if !left.center.is_finite()
            || !right.center.is_finite()
            || !left.velocity.is_finite()
            || !right.velocity.is_finite()
            || !left.sample_phase.is_finite()
            || !right.sample_phase.is_finite()
            || !restitution.is_finite()
            || normal.iter().any(|component| !component.is_finite())
        {
            return Err(MechanicalError::InvalidHolon);
        }
        let closing_speed = left.velocity - right.velocity;
        if !closing_speed.is_finite() || closing_speed <= 0.0 {
            return Err(MechanicalError::NotApproaching);
        }
        let initial_gap = right.center - left.center - left.radius - right.radius;
        if !initial_gap.is_finite() || initial_gap <= 0.0 {
            return Err(MechanicalError::InitiallyOverlapping);
        }
        if !(initial_gap / closing_speed).is_finite() {
            return Err(MechanicalError::InvalidHolon);
        }
        if samples_at_floor < 12 {
            return Err(MechanicalError::InvalidSamples);
        }
        let norm =
            libm::sqrt(normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]);
        if norm <= 1.0e-12 {
            return Err(MechanicalError::InvalidHolon);
        }
        Ok(Self {
            left,
            right,
            restitution: restitution.clamp(0.0, 1.0),
            normal: [normal[0] / norm, normal[1] / norm, normal[2] / norm],
            samples_at_floor,
        })
    }

    pub fn exact_contact_time(&self) -> f64 {
        (self.right.center - self.left.center - self.left.radius - self.right.radius)
            / (self.left.velocity - self.right.velocity)
    }

    pub fn gross_impulse(&self) -> f64 {
        (1.0 + self.restitution) * (self.left.velocity - self.right.velocity)
            / (1.0 / self.left.mass + 1.0 / self.right.mass)
    }

    fn samples_for_grain(&self, grain_units: u32) -> usize {
        let divisor = grain_units as usize * grain_units as usize;
        (self.samples_at_floor / divisor).max(12)
    }

    fn unit_support(&self, samples: usize, direction: [f64; 3], phase: f64) -> f64 {
        let golden_angle = core::f64::consts::PI * (3.0 - libm::sqrt(5.0));
        let mut support = -1.0_f64;
        for i in 0..samples {
            let y = 1.0 - 2.0 * (i as f64 + 0.5) / samples as f64;
            let ring = libm::sqrt((1.0 - y * y).max(0.0));
            let azimuth = golden_angle * i as f64 + phase;
            let point = [ring * libm::cos(azimuth), y, ring * libm::sin(azimuth)];
            let projected =
                point[0] * direction[0] + point[1] * direction[1] + point[2] * direction[2];
            support = support.max(projected);
        }
        support
    }

    fn conservation_residual(&self, impulse: f64) -> f64 {
        let before = self.left.mass * self.left.velocity + self.right.mass * self.right.velocity;
        let left_after = self.left.velocity - impulse / self.left.mass;
        let right_after = self.right.velocity + impulse / self.right.mass;
        let after = self.left.mass * left_after + self.right.mass * right_after;
        let scale = (self.left.mass * self.left.velocity.abs()
            + self.right.mass * self.right.velocity.abs())
        .max(1.0e-12);
        (after - before).abs() / scale
    }

    fn evaluate_at_grains(&self, left_grain: u32, right_grain: u32) -> Evaluation<3> {
        let left_support = self.left.radius
            * self.unit_support(
                self.samples_for_grain(left_grain),
                self.normal,
                self.left.sample_phase,
            );
        let right_direction = [-self.normal[0], -self.normal[1], -self.normal[2]];
        let right_support = self.right.radius
            * self.unit_support(
                self.samples_for_grain(right_grain),
                right_direction,
                self.right.sample_phase,
            );
        let support = left_support + right_support;
        let exact_support = self.left.radius + self.right.radius;
        let closing_speed = self.left.velocity - self.right.velocity;
        let contact_time = (self.right.center - self.left.center - support) / closing_speed;
        let impulse = self.gross_impulse();

        Evaluation {
            observables: [contact_time, impulse, support],
            macro_error_bound: (exact_support - support).abs() / exact_support,
            conservation_residual: self.conservation_residual(impulse),
        }
    }

    fn touches_view<const CAP: usize, const W: usize>(
        arena: &HolonArena<CAP, W>,
        node: usize,
        view_root: usize,
    ) -> bool {
        arena.is_descendant_or_self(node, view_root) || arena.is_descendant_or_self(view_root, node)
    }

    #[cfg(feature = "alloc")]
    fn touches_runtime_view(arena: &RuntimeArena, node: usize, view_root: usize) -> bool {
        arena.is_descendant_or_self(node, view_root) || arena.is_descendant_or_self(view_root, node)
    }
}

impl<const CAP: usize, const W: usize> BoundaryModel<CAP, W, 3> for SphereContactModel {
    fn evaluate(&mut self, arena: &HolonArena<CAP, W>, frontier: &Frontier<CAP>) -> Evaluation<3> {
        let left_grain = frontier.represented_grain(arena, self.left.root);
        let right_grain = frontier.represented_grain(arena, self.right.root);
        self.evaluate_at_grains(left_grain, right_grain)
    }

    fn refinement_priority(
        &self,
        arena: &HolonArena<CAP, W>,
        frontier: &Frontier<CAP>,
        node: usize,
    ) -> f64 {
        if !frontier.is_active(node) {
            return 0.0;
        }
        if Self::touches_view(arena, node, self.left.root)
            || Self::touches_view(arena, node, self.right.root)
        {
            arena
                .holon(node)
                .map_or(0.0, |node| node.grain_units as f64)
        } else {
            0.0
        }
    }
}

#[cfg(feature = "alloc")]
impl RuntimeBoundaryModel<3> for SphereContactModel {
    fn evaluate(&mut self, arena: &RuntimeArena, frontier: &RuntimeFrontier) -> Evaluation<3> {
        let left_grain = frontier.represented_grain(arena, self.left.root);
        let right_grain = frontier.represented_grain(arena, self.right.root);
        self.evaluate_at_grains(left_grain, right_grain)
    }

    fn refinement_priority(
        &self,
        arena: &RuntimeArena,
        frontier: &RuntimeFrontier,
        node: usize,
    ) -> f64 {
        if !frontier.is_active(node) {
            return 0.0;
        }
        if Self::touches_runtime_view(arena, node, self.left.root)
            || Self::touches_runtime_view(arena, node, self.right.root)
        {
            arena
                .holon(node)
                .map_or(0.0, |node| node.grain_units as f64)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holon::{certify, Channels, Decomposition, Holon, HolonArena, NO_HOLON};
    use crate::regplus::GrossState;

    const CAP: usize = 15;
    const W: usize = 4;

    fn gross(elements: u64, direction: i64) -> GrossState {
        GrossState::aggregate(elements, 2 * elements, [direction * elements as i64, 0])
    }

    fn hierarchy() -> HolonArena<CAP, W> {
        let channels = Channels::REG_PLUS.union(Channels::MECHANICAL);
        let encounter_channels = channels.union(Channels::NARRATIVE);
        let x = 1_000_000;
        let z = 64_000;
        let nodes = [
            Holon::new(
                NO_HOLON,
                0,
                16,
                gross(x, 1).combine(gross(z, -1)),
                [0.0; W],
                encounter_channels,
                true,
                Decomposition::Expanded,
            ),
            Holon::new(
                0,
                1,
                8,
                gross(x, 1),
                [1.0, -1.2, 0.8, 0.2],
                channels,
                true,
                Decomposition::Expanded,
            ),
            Holon::new(
                0,
                1,
                8,
                gross(z, -1),
                [0.75, 1.0, -0.2, 0.1],
                channels,
                true,
                Decomposition::Expanded,
            ),
            Holon::new(
                1,
                2,
                4,
                gross(10_000, 1),
                [0.0; W],
                channels,
                true,
                Decomposition::Expanded,
            ),
            Holon::new(
                1,
                2,
                4,
                gross(990_000, 1),
                [0.0; W],
                channels,
                false,
                Decomposition::Latent,
            ),
            Holon::new(
                3,
                3,
                2,
                gross(2_500, 1),
                [0.0; W],
                channels,
                true,
                Decomposition::Expanded,
            ),
            Holon::new(
                3,
                3,
                2,
                gross(7_500, 1),
                [0.0; W],
                channels,
                false,
                Decomposition::Latent,
            ),
            Holon::new(
                5,
                4,
                1,
                gross(625, 1),
                [0.0; W],
                channels,
                true,
                Decomposition::Leaf,
            ),
            Holon::new(
                5,
                4,
                1,
                gross(1_875, 1),
                [0.0; W],
                channels,
                false,
                Decomposition::Leaf,
            ),
            Holon::new(
                2,
                2,
                4,
                gross(640, -1),
                [0.0; W],
                channels,
                true,
                Decomposition::Expanded,
            ),
            Holon::new(
                2,
                2,
                4,
                gross(63_360, -1),
                [0.0; W],
                channels,
                false,
                Decomposition::Latent,
            ),
            Holon::new(
                9,
                3,
                2,
                gross(160, -1),
                [0.0; W],
                channels,
                true,
                Decomposition::Expanded,
            ),
            Holon::new(
                9,
                3,
                2,
                gross(480, -1),
                [0.0; W],
                channels,
                false,
                Decomposition::Latent,
            ),
            Holon::new(
                11,
                4,
                1,
                gross(40, -1),
                [0.0; W],
                channels,
                true,
                Decomposition::Leaf,
            ),
            Holon::new(
                11,
                4,
                1,
                gross(120, -1),
                [0.0; W],
                channels,
                false,
                Decomposition::Leaf,
            ),
        ];
        HolonArena::from_holons(nodes, CAP, 0).unwrap()
    }

    fn model(arena: &HolonArena<CAP, W>) -> SphereContactModel {
        SphereContactModel::new(
            arena,
            SphereView {
                root: 1,
                radius: 1.0,
                center: -1.2,
                velocity: 0.8,
                mass: 1_000_000.0,
                sample_phase: 0.13,
            },
            SphereView {
                root: 2,
                radius: 0.75,
                center: 1.0,
                velocity: -0.2,
                mass: 64_000.0,
                sample_phase: 0.71,
            },
            0.8,
            [0.73, 0.41, 0.547],
            8_192,
        )
        .unwrap()
    }

    #[test]
    fn adaptive_boundary_reaches_macro_parity_without_expanding_interiors() {
        let arena = hierarchy();
        let mut model = model(&arena);
        let certificate = certify(&arena, &mut model, 0.001, 1.0e-12);
        assert!(certificate.passed(), "{certificate:?}");
        assert!(certificate.macro_error_bound <= 0.001);
        assert!(certificate.conservation_residual <= 1.0e-12);
        assert!(certificate.frontier.active_count(&arena) < CAP);
        assert!(certificate.frontier.represented_grain(&arena, 1) <= 2);
        assert!(certificate.frontier.represented_grain(&arena, 2) <= 2);
    }

    #[test]
    fn gross_impact_conserves_momentum_for_unequal_holons() {
        let arena = hierarchy();
        let model = model(&arena);
        let impulse = model.gross_impulse();
        assert!(model.conservation_residual(impulse) < 1.0e-15);
    }

    #[test]
    fn non_finite_contacts_are_rejected_at_the_boundary() {
        let arena = hierarchy();
        let mut invalid = model(&arena).left;
        invalid.velocity = f64::NAN;
        assert_eq!(
            SphereContactModel::new(
                &arena,
                invalid,
                model(&arena).right,
                0.8,
                [1.0, 0.0, 0.0],
                128,
            ),
            Err(MechanicalError::InvalidHolon)
        );

        let mut invalid = model(&arena).left;
        invalid.radius = f64::INFINITY;
        assert_eq!(
            SphereContactModel::new(
                &arena,
                invalid,
                model(&arena).right,
                0.8,
                [1.0, 0.0, 0.0],
                128,
            ),
            Err(MechanicalError::InvalidRadius)
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn runtime_and_const_arenas_produce_bit_identical_certificates() {
        use crate::runtime::{
            certify_runtime, RuntimeArenaBuilder, RuntimeHolonSpec, NO_RUNTIME_HOLON,
        };

        let fixed = hierarchy();
        let mut builder = RuntimeArenaBuilder::with_capacity(CAP, CAP * W);
        for i in 0..fixed.len() {
            let node = fixed.holon(i).unwrap();
            builder
                .push(RuntimeHolonSpec {
                    parent: if node.parent == NO_HOLON {
                        NO_RUNTIME_HOLON
                    } else {
                        node.parent as u32
                    },
                    depth: node.depth,
                    grain_units: node.grain_units,
                    gross: node.gross,
                    whole: &node.whole,
                    channels: node.channels,
                    boundary: node.boundary,
                    decomposition: node.decomposition,
                })
                .unwrap();
        }
        let runtime = builder.build(fixed.root() as u32).unwrap();

        let mut fixed_model = model(&fixed);
        let mut runtime_model = SphereContactModel::new_runtime(
            &runtime,
            fixed_model.left,
            fixed_model.right,
            fixed_model.restitution,
            fixed_model.normal,
            fixed_model.samples_at_floor,
        )
        .unwrap();
        let fixed_certificate = certify(&fixed, &mut fixed_model, 0.001, 1.0e-12);
        let runtime_certificate = certify_runtime(&runtime, &mut runtime_model, 0.001, 1.0e-12);

        assert_eq!(runtime_certificate.status, fixed_certificate.status);
        assert_eq!(
            runtime_certificate.observables,
            fixed_certificate.observables
        );
        assert_eq!(
            runtime_certificate.macro_error_bound.to_bits(),
            fixed_certificate.macro_error_bound.to_bits()
        );
        assert_eq!(
            runtime_certificate.conservation_residual.to_bits(),
            fixed_certificate.conservation_residual.to_bits()
        );
        assert_eq!(
            runtime_certificate.evaluations,
            fixed_certificate.evaluations
        );
        assert_eq!(
            runtime_certificate.frontier.active_count(),
            fixed_certificate.frontier.active_count(&fixed)
        );
        assert_eq!(
            runtime_certificate.frontier.represented_grain(&runtime, 1),
            fixed_certificate.frontier.represented_grain(&fixed, 1)
        );
        assert_eq!(
            runtime_certificate.frontier.represented_grain(&runtime, 2),
            fixed_certificate.frontier.represented_grain(&fixed, 2)
        );
    }
}
