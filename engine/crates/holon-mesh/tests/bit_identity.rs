//! **The gate.** A meshed run is bit-identical to the unsharded run of the same scene.
//!
//! Not 99.9%. Every cell's four integer ledger lanes compared by equality, every whole-state
//! f64 compared by `to_bits()`. `SANDBOX_4090` D4 is what is being cashed here: parallel
//! across arenas, sequential within, fixed merge order.
//!
//! The reference is written independently of the mesh (`reference.rs` knows nothing about
//! shards, halos, boards, or threads), so agreement is evidence rather than tautology.

use holon_mesh::{compare_to_reference, Grid, Mesh, MeshSpec, Mutation, Reference, VisitOrder};

fn gate(spec: MeshSpec, steps: usize, threads: Option<usize>) {
    let label = format!(
        "{}x{} grid, {}x{} shards, n={}, order {:?}, threads {:?}",
        spec.grid.w, spec.grid.h, spec.nx, spec.ny, spec.colours_per_exchange, spec.order, threads
    );
    match compare_to_reference(spec, steps, threads).expect("mesh ran") {
        Ok(()) => {}
        Err(d) => panic!("MESHED != UNSHARDED at {label}: {d:?}"),
    }
}

/// The control. One shard holding everything must agree trivially; if it does not, the defect
/// is in the stepper and every other result in this file would be measuring the wrong thing.
#[test]
fn a_single_shard_mesh_equals_the_unsharded_reference() {
    gate(MeshSpec::new(Grid::new(12, 9), 1, 1), 16, None);
}

/// The claim itself, over partitions that actually cut through the scene.
#[test]
fn meshed_equals_unsharded_across_partitions() {
    for (nx, ny) in [(1, 1), (2, 1), (1, 2), (2, 2), (3, 2), (4, 4), (8, 3)] {
        gate(MeshSpec::new(Grid::new(16, 12), nx, ny), 12, None);
    }
}

/// Deeper halos, more colours between exchanges. `n·r` is `Core/Locality.lean`'s bound and
/// this is the arm that says it SUFFICES; `locality.rs` is the arm that asks whether it is
/// tight.
#[test]
fn meshed_equals_unsharded_across_horizons() {
    for n in [1usize, 2, 3, 4, 6] {
        gate(
            MeshSpec::new(Grid::new(16, 16), 4, 4).with_colours_per_exchange(n),
            12,
            None,
        );
    }
}

/// **M1a — the reorder mutation's must-NOT-fire half.**
///
/// Reordering shard and edge visits must move no bit. This IS the determinism claim: a
/// colour is a perfect matching and the lanes are integers, so order cannot matter. If this
/// test ever fires, the mesh is non-deterministic.
///
/// On its own it is a test that passes because nothing is being checked — which is why
/// `mutations.rs::a_planted_cross_shard_float_reduction_makes_reordering_visible` exists.
#[test]
fn reordering_the_merge_moves_no_bit() {
    for order in [VisitOrder::Natural, VisitOrder::Reversed, VisitOrder::Strided] {
        for n in [1usize, 3] {
            gate(
                MeshSpec::new(Grid::new(14, 10), 3, 3)
                    .with_colours_per_exchange(n)
                    .with_order(order),
                12,
                None,
            );
        }
    }
}

/// The threaded path, against the same unsharded reference — not merely against the
/// sequential mesh. Two independently written schedulers agreeing with a third independently
/// written serial implementation is the strongest form of the claim available.
#[test]
fn threaded_meshed_equals_unsharded_at_every_thread_count() {
    for threads in [1usize, 2, 4, 8, 16] {
        gate(MeshSpec::new(Grid::new(16, 16), 4, 4), 12, Some(threads));
    }
}

#[test]
fn threaded_meshed_equals_unsharded_across_horizons_and_orders() {
    for threads in [1usize, 3, 8] {
        for n in [1usize, 2, 4] {
            for order in [VisitOrder::Natural, VisitOrder::Reversed] {
                gate(
                    MeshSpec::new(Grid::new(12, 12), 3, 2)
                        .with_colours_per_exchange(n)
                        .with_order(order),
                    8,
                    Some(threads),
                );
            }
        }
    }
}

/// Repeats, because a race that shows up one run in ten is not caught by one run.
#[test]
fn the_threaded_result_is_stable_across_repeats() {
    let spec = MeshSpec::new(Grid::new(16, 16), 4, 4).with_colours_per_exchange(2);
    let mut first: Option<(Vec<_>, Vec<u64>)> = None;
    for _ in 0..8 {
        let mut mesh = Mesh::new(spec.clone()).expect("mesh built");
        mesh.run_threaded(12, 8).expect("mesh ran");
        let cells = mesh.cells();
        let bits: Vec<u64> = mesh.energies().iter().map(|e| e.to_bits()).collect();
        match &first {
            None => first = Some((cells, bits)),
            Some((c0, b0)) => {
                assert_eq!(&cells, c0, "threaded run is not reproducible");
                assert_eq!(&bits, b0, "whole-state differs between identical runs");
            }
        }
    }
}

/// Conservation as arithmetic, not as a residual: the scene total is bit-identical in all
/// four lanes after every sweep. There is no epsilon in this crate.
#[test]
fn the_scene_total_is_exactly_conserved_in_all_four_lanes() {
    let spec = MeshSpec::new(Grid::new(16, 12), 4, 3).with_colours_per_exchange(2);
    let mut mesh = Mesh::new(spec).expect("mesh built");
    let opening = mesh.opening_total();
    mesh.run_sequential(20).expect("mesh ran");
    assert_eq!(mesh.total().expect("total"), opening);

    // And the reference agrees on what that total is, which is what makes the two runs
    // comparisons of one scene rather than of two.
    let mut reference = Reference::new(Grid::new(16, 12)).expect("reference built");
    assert_eq!(reference.total(), opening);
    reference.run(20).expect("reference ran");
    assert_eq!(reference.total(), opening);
}

/// The scene must actually move. A gate over a scene that never changes would pass for every
/// implementation, correct or not.
#[test]
fn the_scene_is_not_trivially_static() {
    let grid = Grid::new(12, 12);
    let mut reference = Reference::new(grid).expect("reference built");
    let before: Vec<_> = reference.cells().to_vec();
    let energy_before: Vec<u64> = reference.energies().iter().map(|e| e.to_bits()).collect();
    reference.run(12).expect("reference ran");
    let moved = reference
        .cells()
        .iter()
        .zip(&before)
        .filter(|(a, b)| a != b)
        .count();
    let energy_moved = reference
        .energies()
        .iter()
        .zip(&energy_before)
        .filter(|(a, b)| a.to_bits() != **b)
        .count();
    assert!(
        moved > grid.len() / 2,
        "only {moved} of {} cells moved; the gate would be vacuous",
        grid.len()
    );
    assert!(energy_moved > grid.len() / 2, "whole-state barely moved");
}

/// A mesh built with no mutation must declare a halo exactly `n·r` deep, and must never read
/// deeper than that. The horizon is `Core/Locality.lean`'s and it is not a tuning knob.
#[test]
fn the_halo_is_exactly_the_declared_horizon() {
    for n in [1usize, 2, 4] {
        let spec = MeshSpec::new(Grid::new(16, 16), 4, 4).with_colours_per_exchange(n);
        let horizon = spec.horizon();
        let mut mesh = Mesh::new(spec).expect("mesh built");
        mesh.run_sequential(8).expect("mesh ran");
        assert_eq!(
            mesh.max_read_depth(),
            horizon,
            "n={n}: read depth should be exactly the n*r horizon"
        );
    }
}

/// Mutations must be reachable from the public API, or the mutation suite is testing a
/// private fork of the code rather than the code.
#[test]
fn a_mutated_spec_still_builds_and_runs() {
    for mutation in [
        Mutation::DoubleTransferBothSides,
        Mutation::HaloOneShallowerThanHorizon,
        Mutation::PairOrientedByVisitOrder,
        Mutation::HaloRefreshSkipped,
        Mutation::HaloReadsLivePeers,
    ] {
        let spec = MeshSpec::new(Grid::new(8, 8), 2, 2)
            .with_colours_per_exchange(2)
            .with_mutation(mutation);
        let mut mesh = Mesh::new(spec).expect("mutated mesh built");
        // A mutation may break conservation; that is the point. It must not panic.
        let _ = mesh.run_sequential(6);
    }
}

// ---------------------------------------------------------------------------- 3D

/// **The gate on a scene with real extent on all three axes.**
///
/// This is what the lane exists for (`MESH_DESIGN.md` §0: shard ONE 3D scene across cores so
/// the sandbox can afford 3D). The 2D scene is the `d = 1` degeneration of the same object, so
/// these are literally the same assertions on a thicker grid — not a parallel code path.
#[test]
fn meshed_equals_unsharded_on_a_three_d_scene() {
    for (nx, ny, nz) in [(1, 1, 1), (2, 1, 1), (2, 2, 1), (2, 2, 2), (3, 2, 2), (4, 3, 2)] {
        gate(MeshSpec::new_3d(Grid::new_3d(12, 8, 6), nx, ny, nz), 12, None);
    }
}

/// 3D, deeper halos. Every one of the six colours is live here, where a flat scene leaves the
/// two z-colours empty.
#[test]
fn meshed_equals_unsharded_in_three_d_across_horizons() {
    for n in [1usize, 2, 3, 4, 6] {
        gate(
            MeshSpec::new_3d(Grid::new_3d(8, 8, 8), 2, 2, 2).with_colours_per_exchange(n),
            12,
            None,
        );
    }
}

/// 3D, threaded, against the unsharded reference — the full claim in the dimension that
/// matters.
#[test]
fn threaded_meshed_equals_unsharded_on_a_three_d_scene() {
    for threads in [1usize, 2, 4, 8, 16] {
        gate(
            MeshSpec::new_3d(Grid::new_3d(12, 12, 12), 3, 2, 2).with_colours_per_exchange(2),
            10,
            Some(threads),
        );
    }
}

/// Reordering moves no bit in 3D either. The must-NOT-fire half of the reorder pair, at the
/// dimension the mesh is for.
#[test]
fn reordering_the_merge_moves_no_bit_in_three_d() {
    for order in [VisitOrder::Natural, VisitOrder::Reversed, VisitOrder::Strided] {
        gate(
            MeshSpec::new_3d(Grid::new_3d(8, 8, 8), 2, 2, 2)
                .with_colours_per_exchange(3)
                .with_order(order),
            12,
            None,
        );
    }
}

/// A 3D scene must actually exercise its third axis. Without this, every 3D test above could
/// be passing on a scene that behaves like a stack of independent 2D slices.
#[test]
fn the_third_axis_actually_carries_interaction() {
    let grid = Grid::new_3d(6, 6, 6);
    let flat = Grid::new_3d(6, 6, 1);
    let z_edges: usize = (4..6).map(|c| holon_mesh::grid::edges_of_colour(grid, c).len()).sum();
    assert!(z_edges > 0, "the 3D scene has no z-adjacency at all");
    assert_eq!(
        (4..6)
            .map(|c| holon_mesh::grid::edges_of_colour(flat, c).len())
            .sum::<usize>(),
        0,
        "a flat scene must have no z-adjacency, or `d = 1` is not the 2D case"
    );

    // And a cut ON the z axis must produce boundary work, or sharding in z is a no-op.
    let part = holon_mesh::Partition::blocks(grid, 1, 1, 2);
    let cross: usize = (0..holon_mesh::EDGE_COLOURS)
        .map(|c| part.cross_edges(c).len())
        .sum();
    assert!(cross > 0, "a z-only cut produced no boundary");
}
