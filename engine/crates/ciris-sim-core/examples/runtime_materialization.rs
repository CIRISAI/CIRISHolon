use ciris_sim_core::holon::{Channels, Decomposition, HolonError};
use ciris_sim_core::mechanical::{SphereContactModel, SphereView};
use ciris_sim_core::regplus::GrossState;
use ciris_sim_core::runtime::{
    certify_runtime_adaptive, RuntimeArena, RuntimeArenaBuilder, RuntimeHolonSpec,
    RuntimeMaterializer, NO_RUNTIME_HOLON,
};

fn gross(elements: u64, direction: i64) -> GrossState {
    GrossState::aggregate(elements, 2 * elements, [direction * elements as i64, 0])
}

fn split_gross(total: GrossState) -> (GrossState, GrossState) {
    let boundary = GrossState::aggregate(
        total.constituents / 4,
        total.occupancy / 4,
        [total.momentum[0] / 4, total.momentum[1] / 4],
    );
    let interior = GrossState::aggregate(
        total.constituents - boundary.constituents,
        total.occupancy - boundary.occupancy,
        [
            total.momentum[0] - boundary.momentum[0],
            total.momentum[1] - boundary.momentum[1],
        ],
    );
    (boundary, interior)
}

struct ProceduralBoundary;

impl RuntimeMaterializer for ProceduralBoundary {
    fn materialize(&mut self, arena: &mut RuntimeArena, node: usize) -> Result<bool, HolonError> {
        let parent = *arena.holon(node).ok_or(HolonError::InvalidParent)?;
        if parent.decomposition != Decomposition::Latent || parent.grain_units == 1 {
            return Ok(false);
        }
        let grain_units = parent.grain_units / 2;
        let decomposition = if grain_units == 1 {
            Decomposition::Leaf
        } else {
            Decomposition::Latent
        };
        let (boundary_gross, interior_gross) = split_gross(parent.gross);
        let children = [
            RuntimeHolonSpec {
                parent: node as u32,
                depth: parent.depth + 1,
                grain_units,
                gross: boundary_gross,
                whole: &[],
                channels: parent.channels,
                boundary: true,
                decomposition,
            },
            RuntimeHolonSpec {
                parent: node as u32,
                depth: parent.depth + 1,
                grain_units,
                gross: interior_gross,
                whole: &[],
                channels: parent.channels,
                boundary: false,
                decomposition,
            },
        ];
        arena.materialize(node, &children)?;
        Ok(true)
    }
}

fn main() {
    let channels = Channels::REG_PLUS
        .union(Channels::MECHANICAL)
        .union(Channels::NARRATIVE)
        .union(Channels::AGENTIC);
    let person = gross(1_000_000, 1);
    let npc = gross(64_000, -1);
    let mut builder = RuntimeArenaBuilder::with_capacity(15, 8);
    builder
        .push(RuntimeHolonSpec {
            parent: NO_RUNTIME_HOLON,
            depth: 0,
            grain_units: 16,
            gross: person.combine(npc),
            whole: &[0.5, -0.2],
            channels,
            boundary: true,
            decomposition: Decomposition::Expanded,
        })
        .unwrap();
    builder
        .push(RuntimeHolonSpec {
            parent: 0,
            depth: 1,
            grain_units: 8,
            gross: person,
            whole: &[1.0, -1.2, 0.8],
            channels,
            boundary: true,
            decomposition: Decomposition::Latent,
        })
        .unwrap();
    builder
        .push(RuntimeHolonSpec {
            parent: 0,
            depth: 1,
            grain_units: 8,
            gross: npc,
            whole: &[0.75, 1.0, -0.2],
            channels,
            boundary: true,
            decomposition: Decomposition::Latent,
        })
        .unwrap();
    let mut arena = builder.build(0).unwrap();

    let mut contact = SphereContactModel::new_runtime(
        &arena,
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
    .unwrap();
    let exact_time = contact.exact_contact_time();
    let result = certify_runtime_adaptive(
        &mut arena,
        &mut contact,
        &mut ProceduralBoundary,
        0.001,
        1.0e-12,
    )
    .unwrap();
    let certificate = result.certificate;

    println!(
        "encounter: {} terminal elements; resident holons {} (started with 3)",
        arena.holon(0).unwrap().gross.constituents,
        arena.len()
    );
    println!(
        "status {:?}; materializations {}; evaluations {}",
        certificate.status, result.materializations, certificate.evaluations
    );
    println!(
        "active frontier {}; person grain {}; NPC grain {}",
        certificate.frontier.active_count(),
        certificate.frontier.represented_grain(&arena, 1),
        certificate.frontier.represented_grain(&arena, 2)
    );
    println!(
        "boundary error {:.6}%; conservation residual {:.3e}",
        100.0 * certificate.macro_error_bound,
        certificate.conservation_residual
    );
    println!(
        "contact time {:.9} (exact {:.9}); impulse {:.6}",
        certificate.observables[0], exact_time, certificate.observables[1]
    );
}
