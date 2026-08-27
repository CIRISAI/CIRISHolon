//! **A gate that cannot fail proves nothing.** This file is where each leg is shown to have
//! teeth — and where the one mutation that *must not* fire is separated from the ones that
//! must.
//!
//! `MESH_DESIGN.md` §6 names the trap and it is worth restating at the top of the file that
//! encodes it: the brief asks that a deliberately reordered merge be caught, and taken
//! literally that test cannot pass. With a per-colour perfect matching over integer lanes,
//! **reordering produces the identical result** — that is the design working. A mutation
//! asserting "reorder changed the answer" would fail against a correct implementation, and
//! weakening it until it passes ships a gate that cannot fail.
//!
//! So the reorder mutation is split across two files:
//!
//! * `bit_identity.rs::reordering_the_merge_moves_no_bit` — must NOT fire. That IS the claim.
//! * [`a_planted_cross_shard_float_reduction_makes_reordering_visible`] — must fire. That is
//!   what proves the reorder harness can fail at all, and that `SANDBOX_4090` D4's ban on
//!   cross-lane float reductions is enforced rather than merely written down.
//!
//! Neither half means anything alone.

use ciris_sim_core::regplus::GrossState;
use holon_mesh::{compare_to_reference, Grid, Mesh, MeshSpec, Mutation, VisitOrder};

/// Run a mesh to completion and return its full fingerprint.
fn fingerprint(spec: MeshSpec, steps: usize) -> (Vec<GrossState>, Vec<u64>, GrossState) {
    let mut mesh = Mesh::new(spec).expect("mesh built");
    // A mutation may break conservation mid-run; that is what is being measured, so the
    // scene check's verdict is returned rather than unwrapped.
    let _ = mesh.run_sequential(steps);
    let total = mesh.total().unwrap_or(GrossState::ZERO);
    (
        mesh.cells(),
        mesh.energies().iter().map(|e| e.to_bits()).collect(),
        total,
    )
}

fn base(mutation: Mutation, n: usize) -> MeshSpec {
    MeshSpec::new(Grid::new(16, 12), 4, 3)
        .with_colours_per_exchange(n)
        .with_mutation(mutation)
}

/// The same, on a scene with real extent on all three axes, so all SIX edge colours are live.
/// A flat scene leaves the two z-colours empty, which is fine for most mutations and is
/// exactly what made the horizon claim over-general.
fn base_3d(mutation: Mutation, n: usize) -> MeshSpec {
    MeshSpec::new_3d(Grid::new_3d(8, 8, 8), 2, 2, 2)
        .with_colours_per_exchange(n)
        .with_mutation(mutation)
}

/// [`must_fire`], on the 3D scene.
fn must_fire_3d(mutation: Mutation, n: usize, steps: usize) {
    match compare_to_reference(base_3d(mutation, n), steps, None) {
        Ok(Err(_)) | Err(_) => {}
        Ok(Ok(())) => panic!(
            "MUTATION NOT CAUGHT IN 3D: {mutation:?} at n={n} produced the unsharded answer."
        ),
    }
}

/// Assert a mutation is CAUGHT by the gate — the meshed run stops matching the unsharded one.
fn must_fire(mutation: Mutation, n: usize, steps: usize) {
    let verdict = compare_to_reference(base(mutation, n), steps, None);
    match verdict {
        // Caught by disagreement with the reference.
        Ok(Err(_)) => {}
        // Caught by refusal — a mutation that drives a lane out of range is still caught.
        Err(_) => {}
        Ok(Ok(())) => panic!(
            "MUTATION NOT CAUGHT: {mutation:?} at n={n} produced the unsharded answer. \
             The gate is blind to it."
        ),
    }
}

// ---------------------------------------------------------------- the reorder pair, half 2

/// **M1b.** Plant the banned cross-shard f64 reduction — every cell's energy summed into one
/// accumulator in shard visit order, fed back into the cells — and reordering becomes
/// visible, because float addition is not associative.
///
/// This is the test that gives `reordering_the_merge_moves_no_bit` its meaning. Without it,
/// that test passes because nothing is being checked.
#[test]
fn a_planted_cross_shard_float_reduction_makes_reordering_visible() {
    let steps = 12;
    let natural = fingerprint(
        base(Mutation::CrossShardFloatReduction, 1).with_order(VisitOrder::Natural),
        steps,
    );
    let reversed = fingerprint(
        base(Mutation::CrossShardFloatReduction, 1).with_order(VisitOrder::Reversed),
        steps,
    );
    let strided = fingerprint(
        base(Mutation::CrossShardFloatReduction, 1).with_order(VisitOrder::Strided),
        steps,
    );

    assert_ne!(
        natural.1, reversed.1,
        "the planted float reduction did not make reordering visible; the reorder harness \
         has no teeth and `reordering_the_merge_moves_no_bit` is vacuous"
    );
    assert_ne!(natural.1, strided.1, "strided order left the float sum unchanged");

    // And the tell that it is the FLOAT path specifically: the integer ledger is untouched by
    // the reordering, exactly as the design says it must be. The two lanes behave differently
    // under the same reordering, which is the whole distinction the mutation exists to draw.
    assert_eq!(
        natural.0, reversed.0,
        "reordering moved an INTEGER lane; integer + is exactly associative, so this would \
         mean the mesh is non-deterministic rather than that the float mutation worked"
    );
}

/// The float mutation is refused on the threaded path, and refused explicitly rather than
/// quietly producing a number. Running it threaded would measure thread scheduling instead of
/// merge order, and a mutation that measures the wrong thing is worse than none.
#[test]
fn the_float_mutation_is_refused_on_the_threaded_path() {
    let mut mesh = Mesh::new(base(Mutation::CrossShardFloatReduction, 1)).expect("built");
    assert!(
        mesh.run_threaded(4, 4).is_err(),
        "the float instrument must refuse the threaded path, not silently run on it"
    );
}

// ------------------------------------------------------------------- the boundary protocol

/// **M4 — `SANDBOX_4090` §6's finding, re-run in the mesh.**
///
/// Both sides of a boundary transfer apply twice. The scene total is still **exactly**
/// conserved and the two sides are still **exactly** antisymmetric, so every balance-based
/// leg passes. Only comparison against the independently re-derived answer sees it.
///
/// This is the single most important row in the mutation table, because it is the one that
/// refutes the obvious gate design.
#[test]
fn doubling_the_transfer_on_both_sides_is_caught_only_by_re_derivation() {
    must_fire(Mutation::DoubleTransferBothSides, 1, 10);

    // The other half of the finding, stated as an assertion rather than as a warning: the
    // balance-based gate passes on this mutation. If a future refactor made the total move,
    // this test would stop demonstrating what it exists to demonstrate.
    let (_, _, total) = fingerprint(base(Mutation::DoubleTransferBothSides, 1), 10);
    let opening = Mesh::new(base(Mutation::None, 1))
        .expect("built")
        .opening_total();
    assert_eq!(
        total, opening,
        "the doubled transfer should stay perfectly conserved — that is precisely why a \
         global-sum gate is blind to it"
    );
}

/// **M6.** Each shard orients the pair from its own view — "I debit whichever end I own" —
/// so both shards think they are the debited side. Locally sensible, globally incoherent.
#[test]
fn orienting_a_pair_from_the_local_view_is_caught() {
    must_fire(Mutation::PairOrientedByVisitOrder, 1, 10);
}

/// **M2.** The halo is refreshed after the sweep that needed it, so every shard planned its
/// boundary against neighbours that had already moved.
#[test]
fn reading_live_peers_instead_of_a_snapshot_is_caught() {
    must_fire(Mutation::HaloReadsLivePeers, 2, 12);
}

/// **M3.** One exchange's refresh is skipped entirely and the shards step on stale data.
#[test]
fn skipping_a_halo_refresh_is_caught() {
    must_fire(Mutation::HaloRefreshSkipped, 2, 12);
}

// ------------------------------------------------------------------------- the horizon

/// **M5, and it is a MEASUREMENT rather than a citation — with a claim of this lane's KILLED
/// along the way.**
///
/// `Core/Locality.lean::iterate_depends_within` proves `n·r` SUFFICES. It does not say `n·r`
/// is necessary. So the crate does not assume it: build the halo one cell shallower and ask
/// whether the answer moves.
///
/// An earlier revision of `MESH_DESIGN.md` §10.3 read *"the bound is TIGHT on this stencil"*.
/// **That is false and this test is what killed it.** It was measured when the colour
/// decomposition had four colours (2D only); generalising to six exposed `n` values where a
/// halo of `n·r − 1` is perfectly sufficient. Swept over five geometries × `n = 1…8`
/// (`C` = caught, `-` = not caught):
///
/// ```text
///   flat  16x12x1 : n1:C n2:C n3:-  n4:C n5:-  n6:-  n7:- n8:-
///   cube    8x8x8 : n1:C n2:C n3:C  n4:C n5:-  n6:C  n7:- n8:-
///   cube 12x12x12 : n1:C n2:C n3:C  n4:C n5:-  n6:C  n7:- n8:-
///   slab  12x8x6  : n1:C n2:C n3:-  n4:C n5:C  n6:-  n7:- n8:-
///   slab  12x8x4  : n1:C n2:C n3:C  n4:C n5:C  n6:C  n7:- n8:-
/// ```
///
/// **Why `n·r` over-counts:** one colour sweep moves data across only *half* the edges of
/// *one* axis, so its effective radius is strictly less than the 1 that `n·r` charges it. How
/// much less depends on which colours the exchange window happens to contain, which is why
/// the answer is geometry- and `n`-dependent rather than uniform.
///
/// None of this weakens the mesh: `n·r` remains PROVED sufficient, the gate confirms
/// `meshed == unsharded` at every `n` with the full halo, and a conservative halo is the safe
/// direction to be wrong in. What died is only the claim that it could not be smaller.
#[test]
fn a_shallower_halo_is_caught_at_the_depths_where_the_bound_is_load_bearing() {
    // n = 1, 2, 4 are caught on EVERY geometry swept. These are the depths at which the
    // horizon is doing real work, and they are what the assertion may rely on.
    for n in [1usize, 2, 4] {
        must_fire(Mutation::HaloOneShallowerThanHorizon, n, 16);
        must_fire_3d(Mutation::HaloOneShallowerThanHorizon, n, 16);
    }
}

/// **The negative, pinned so it cannot be quietly re-claimed as tightness.**
///
/// At `n = 8` a halo of `n·r − 1 = 7` is sufficient on every geometry swept — the mutation
/// does NOT fire. This is an observation about the bound's slack, not a correctness property,
/// and it is asserted so that the falsification stays in the record. If this ever starts
/// firing, the bound got tighter and §10.3 must be revisited rather than left stale.
#[test]
fn at_large_n_a_shallower_halo_is_sufficient_and_the_bound_has_slack() {
    let flat = MeshSpec::new(Grid::new(16, 12), 4, 3)
        .with_colours_per_exchange(8)
        .with_mutation(Mutation::HaloOneShallowerThanHorizon);
    assert!(
        matches!(compare_to_reference(flat, 24, None), Ok(Ok(()))),
        "n=8 on a flat scene: a halo of 7 was expected to suffice; if it no longer does, the \
         horizon's slack has changed and MESH_DESIGN §10.3 needs re-measuring"
    );

    let cube = MeshSpec::new_3d(Grid::new_3d(8, 8, 8), 2, 2, 2)
        .with_colours_per_exchange(8)
        .with_mutation(Mutation::HaloOneShallowerThanHorizon);
    assert!(
        matches!(compare_to_reference(cube, 24, None), Ok(Ok(()))),
        "n=8 on a cubic scene: a halo of 7 was expected to suffice"
    );
}

/// At `n = 1` the shallower halo is no halo at all, so a shard cannot even see its
/// neighbours. Caught for a different reason than the `n ≥ 2` case, and worth separating so
/// the two are not confused for one result.
#[test]
fn no_halo_at_all_is_caught() {
    must_fire(Mutation::HaloOneShallowerThanHorizon, 1, 8);
}

// ----------------------------------------------------------------------------- must NOT fire

/// **M7.** Load balance is a scheduling decision and must not be a physical one. Changing the
/// partition — which is what a claim-based balancer does every few rounds — must move no bit.
///
/// Listed with the mutations on purpose: it is a control, and controls belong next to the
/// things they control for.
#[test]
fn changing_the_partition_moves_no_bit() {
    let steps = 12;
    let grid = Grid::new(24, 12);
    let reference = fingerprint(MeshSpec::new(grid, 1, 1), steps);
    for (nx, ny) in [(2, 1), (3, 2), (4, 3), (6, 4), (8, 6), (12, 12)] {
        let other = fingerprint(MeshSpec::new(grid, nx, ny), steps);
        assert_eq!(
            reference.0, other.0,
            "partition {nx}x{ny} changed an integer lane; balance must be schedule, not physics"
        );
        assert_eq!(
            reference.1, other.1,
            "partition {nx}x{ny} changed the whole-state"
        );
        assert_eq!(reference.2, other.2, "partition {nx}x{ny} changed the scene total");
    }
}

/// The mutation set as a whole: every entry that must fire, does. Run together so the table in
/// `MESH_DESIGN.md` §6 has a single executable counterpart rather than a scattered one.
#[test]
fn every_must_fire_mutation_fires() {
    let table = [
        (Mutation::DoubleTransferBothSides, 1usize),
        (Mutation::PairOrientedByVisitOrder, 1),
        (Mutation::HaloReadsLivePeers, 2),
        (Mutation::HaloRefreshSkipped, 2),
        (Mutation::HaloOneShallowerThanHorizon, 2),
        // n=2 is caught on every geometry swept; see the horizon tests for why not every n is.
    ];
    for (mutation, n) in table {
        must_fire(mutation, n, 12);
    }
}
