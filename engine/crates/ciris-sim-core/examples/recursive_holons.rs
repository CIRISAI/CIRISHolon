use ciris_sim_core::holon::{certify, Channels, Decomposition, Holon, HolonArena, NO_HOLON};
use ciris_sim_core::mechanical::{SphereContactModel, SphereView};
use ciris_sim_core::regplus::GrossState;

const CAP: usize = 15;
const W: usize = 4;

fn gross(elements: u64, direction: i64) -> GrossState {
    GrossState::aggregate(elements, 2 * elements, [direction * elements as i64, 0])
}

fn encounter() -> HolonArena<CAP, W> {
    let holon_channels = Channels::REG_PLUS
        .union(Channels::MECHANICAL)
        .union(Channels::NARRATIVE)
        .union(Channels::AGENTIC);
    let encounter_channels = holon_channels.union(Channels::QUANTUM);
    let person_elements = 1_000_000;
    let npc_elements = 64_000;

    // Resident refinement window:
    //   0 encounter
    //   ├─ 1 person, with one recursively refinable contact-side branch
    //   └─ 2 NPC, with one recursively refinable contact-side branch
    // Each latent interior carries its full REG+ aggregate without being enumerated.
    let nodes = [
        Holon::new(
            NO_HOLON,
            0,
            16,
            gross(person_elements, 1).combine(gross(npc_elements, -1)),
            [0.91, 0.09, 0.0, 0.0],
            encounter_channels,
            true,
            Decomposition::Expanded,
        ),
        Holon::new(
            0,
            1,
            8,
            gross(person_elements, 1),
            [1.0, -1.2, 0.8, 0.2],
            holon_channels,
            true,
            Decomposition::Expanded,
        ),
        Holon::new(
            0,
            1,
            8,
            gross(npc_elements, -1),
            [0.75, 1.0, -0.2, 0.1],
            holon_channels,
            true,
            Decomposition::Expanded,
        ),
        Holon::new(
            1,
            2,
            4,
            gross(10_000, 1),
            [0.0; W],
            holon_channels,
            true,
            Decomposition::Expanded,
        ),
        Holon::new(
            1,
            2,
            4,
            gross(990_000, 1),
            [0.0; W],
            holon_channels,
            false,
            Decomposition::Latent,
        ),
        Holon::new(
            3,
            3,
            2,
            gross(2_500, 1),
            [0.0; W],
            holon_channels,
            true,
            Decomposition::Expanded,
        ),
        Holon::new(
            3,
            3,
            2,
            gross(7_500, 1),
            [0.0; W],
            holon_channels,
            false,
            Decomposition::Latent,
        ),
        Holon::new(
            5,
            4,
            1,
            gross(625, 1),
            [0.0; W],
            holon_channels,
            true,
            Decomposition::Leaf,
        ),
        Holon::new(
            5,
            4,
            1,
            gross(1_875, 1),
            [0.0; W],
            holon_channels,
            false,
            Decomposition::Leaf,
        ),
        Holon::new(
            2,
            2,
            4,
            gross(640, -1),
            [0.0; W],
            holon_channels,
            true,
            Decomposition::Expanded,
        ),
        Holon::new(
            2,
            2,
            4,
            gross(63_360, -1),
            [0.0; W],
            holon_channels,
            false,
            Decomposition::Latent,
        ),
        Holon::new(
            9,
            3,
            2,
            gross(160, -1),
            [0.0; W],
            holon_channels,
            true,
            Decomposition::Expanded,
        ),
        Holon::new(
            9,
            3,
            2,
            gross(480, -1),
            [0.0; W],
            holon_channels,
            false,
            Decomposition::Latent,
        ),
        Holon::new(
            11,
            4,
            1,
            gross(40, -1),
            [0.0; W],
            holon_channels,
            true,
            Decomposition::Leaf,
        ),
        Holon::new(
            11,
            4,
            1,
            gross(120, -1),
            [0.0; W],
            holon_channels,
            false,
            Decomposition::Leaf,
        ),
    ];
    HolonArena::from_holons(nodes, CAP, 0).expect("valid recursive encounter")
}

fn main() {
    let arena = encounter();
    let mut model = SphereContactModel::new(
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
    .expect("valid mechanical realization");

    let certificate = certify(&arena, &mut model, 0.001, 1.0e-12);
    let root = arena.holon(arena.root()).unwrap();
    println!("CIRISHOLON — gross plus certified boundary");
    println!(
        "encounter: {} CIRIS elements; REG+ occupancy={}; momentum={:?}",
        root.gross.constituents, root.gross.occupancy, root.gross.momentum
    );
    println!("person: 1,000,000 elements; NPC: 64,000 elements");
    println!(
        "resident holons: {} (latent interiors stay aggregated)",
        arena.len()
    );
    println!("certificate: {:?}", certificate.status);
    println!("model evaluations: {}", certificate.evaluations);
    println!(
        "active frontier: {} holons; person grain={}g0; NPC grain={}g0",
        certificate.frontier.active_count(&arena),
        certificate.frontier.represented_grain(&arena, 1),
        certificate.frontier.represented_grain(&arena, 2),
    );
    println!(
        "boundary error={:.6}% conservation residual={:.3e}",
        100.0 * certificate.macro_error_bound,
        certificate.conservation_residual,
    );
    println!(
        "contact time={:.9} (exact {:.9}); gross impulse={:.6}",
        certificate.observables[0],
        model.exact_contact_time(),
        certificate.observables[1],
    );
}
