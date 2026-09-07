//! The closure adapter's gate: the shared type reproduces `bond_graph`'s reading BIT FOR
//! BIT on FLUID-1's own carrier, and the block chart's edge reads through the same trait.
//!
//! The reproduction is the whole point of the adapter. `orientation::bond_graph` stays the
//! campaign's readout and is not touched; if `holon-closure::phase` and it ever disagree on
//! a scene the freeze runs on, one rule has become two and this test says so.

use holon_lattice::chart::BlockChart;
use holon_lattice::closure::{agrees_with_bond_graph, bond_edges, chart_view, closures, occupied_slots, phase, TIER};
use holon_lattice::lattice::Lattice;
use holon_lattice::orientation::{bond_graph, OrientationLattice, OrientationRules};
use holon_lattice::state::Model;
use holon_closure::{BlockView, Edge, TierId};

/// FLUID-1's chart on a torus of side `l`, stepped `steps` times. The amplitude table and
/// the retention are the campaign's own test values; nothing here is a reading, it is a
/// configuration to reproduce a readout on.
fn scene(l: usize, seed: u64, steps: usize) -> OrientationLattice {
    let m = Model::fhp6();
    let law = m.fhp_i(true);
    let lat = Lattice::seeded(m, l, seed, 0.2, law);
    let rules = OrientationRules {
        bonds_enabled: true,
        amplitude: [1.0, 1.3, 1.28, 1.42, 1.28, 1.3],
        rent: OrientationRules::rent_from_retention(0.5526),
        cold: false,
        stream: true,
    };
    let mut g = OrientationLattice::from_lattice(lat, seed, rules);
    for _ in 0..steps {
        g.step();
    }
    g
}

/// THE ADAPTER TEST the brief asks for: a 500-step run at `L = 64`, and the same bond count
/// and the same largest component out of the shared type.
#[test]
fn the_shared_phase_reproduces_bond_graph_on_a_500_step_l64_run() {
    let g = scene(64, 0x464c_5549_4431, 500);
    let bg = bond_graph(&g);
    assert!(bg.bonds > 0, "the scene has bonds to reproduce: {} bonds", bg.bonds);
    assert!(bg.particles > 0);

    let (bg2, p) = agrees_with_bond_graph(&g).unwrap_or_else(|why| {
        panic!("the shared phase and bond_graph disagree on a 500-step L=64 run: {why}")
    });
    // stated again field by field, so a reader of this test sees what "bit for bit" covers
    assert_eq!(p.nodes, bg2.particles);
    assert_eq!(p.edges, bg2.bonds);
    assert_eq!(p.largest, bg2.largest);
    assert_eq!(p.largest_fraction.to_bits(), bg2.largest_fraction.to_bits());
    assert_eq!(p.largest_winds, bg2.largest_winds);
    assert_eq!(p.largest_spans, bg2.largest_spans);
    assert_eq!(p.any_spans, bg2.any_spans);
    assert_eq!(p.degree.to_bits(), bg2.bonds_per_particle_degree.to_bits());
    assert_eq!(phase(&g), p, "the public entry point is the same reading");
}

/// And on more than one configuration, so the agreement is not one seed's accident.
#[test]
fn the_two_readings_agree_across_sizes_seeds_and_step_counts() {
    for (l, seed, steps) in
        [(16usize, 0x11u64, 0usize), (16, 0x11, 50), (32, 0x2f, 200), (64, 0x9e37, 100)]
    {
        let g = scene(l, seed, steps);
        agrees_with_bond_graph(&g)
            .unwrap_or_else(|why| panic!("L={l} seed={seed:#x} steps={steps}: {why}"));
    }
}

#[test]
fn a_particle_is_a_one_member_closure_on_the_fluid_tier() {
    let g = scene(16, 0x11, 30);
    let cs = closures(&g);
    assert_eq!(TIER, TierId::FLUID_ELEMENT);
    assert_eq!(cs.len(), occupied_slots(&g).len());
    assert_eq!(cs.len(), bond_graph(&g).particles);
    assert!(cs.iter().all(|c| c.len() == 1));
}

/// Every edge names a declared node, so nothing is silently dropped by the phase reading.
#[test]
fn every_bond_edge_names_a_node_inside_the_carrier() {
    let g = scene(32, 0x2f, 120);
    let n = occupied_slots(&g).len() as u32;
    let e = bond_edges(&g);
    assert_eq!(e.len(), bond_graph(&g).bonds);
    assert!(e.iter().all(|x| x.a < n && x.b < n));
    assert!(e.iter().all(|x| x.a != x.b), "a bond joined a particle to itself");
}

/// The block chart reads as a `BlockView`, and the vacuous end carries its label through.
#[test]
fn the_block_chart_is_a_view_whose_edge_can_be_read() {
    let m = Model::fhp6();
    let law = m.fhp_i(true);
    let base = Lattice::seeded(m, 16, 0x77, 0.3, law);

    let v = chart_view(&base, BlockChart::new(4, 16).unwrap(), 1);
    assert_eq!(v.cells(), 16, "16 blocks at b = 4 on L = 16");
    assert_eq!(v.b(), 4);
    assert_eq!(v.period(), Some([16.0, 16.0]), "the lattice is a torus and says so");
    assert!(!v.vacuous());

    let e = Edge::at(&v);
    assert_eq!(e.b, 4);
    assert_eq!(e.cells_total, 16);
    assert!(!e.empty, "the fluid tier's chart does not close at b = 4: FLUID-1 W");
    assert!(e.fraction > 0.0 && e.fraction <= 1.0);
    assert!(e.width.is_finite());

    // b = L is held by conservation alone, and the label survives the trait.
    let global = chart_view(&base, BlockChart::new(16, 16).unwrap(), 1);
    assert!(global.vacuous());
    assert!(Edge::at(&global).vacuous);
}

/// The per-block reading is bounded above by the chart's own defect law: a block cannot
/// split more often than the geometry allows, and a block that never splits is a block
/// whose predicted rate is zero.
#[test]
fn no_block_splits_where_the_charts_own_law_predicts_no_witness() {
    let m = Model::fhp6();
    let law = m.fhp_i(true);
    let base = Lattice::seeded(m, 16, 0x77, 0.3, law);
    let global = chart_view(&base, BlockChart::new(16, 16).unwrap(), 1);
    assert_eq!(BlockChart::predicted_witness_rate(16, 16), 0.0);
    assert!(
        global.splits.iter().all(|&s| !s),
        "the global chart split, where its own law says no witness can exist"
    );
}
