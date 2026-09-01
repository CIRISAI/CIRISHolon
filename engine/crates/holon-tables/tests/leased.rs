//! Table generation **through the resource layer**: real worker leases, real probes, receipts
//! paid from real work, and books that must balance before a table is published.
//!
//! This is the first integration in which `holon-resource`'s probes are REAL rather than
//! injected — [`WorkerProbe`] spawns a thread, runs it and joins it, so a lease here is admitted
//! by the OS actually producing a worker rather than by a capacity reading agreeing that it
//! could.

use holon_chem::elements::by_symbol;
use holon_resource::{Arena, LeaseState, ProbeVerdict, Probe, ResourceKind, ScriptedProbe};
use holon_tables::generate::generate_leased;
use holon_tables::{GenSpec, TableGrid, WarmPolicy, WorkerProbe};

fn tiny_spec() -> GenSpec {
    let h = by_symbol("H").unwrap();
    // A cheap real system: H3 at 9 determinants. The physics is not what is under test here —
    // the lease plumbing is — so this is deliberately the smallest REAL path available.
    // 4x4x2 = 32 nodes in 2x2x2 regions = FOUR regions. The region count is the thing that
    // matters here: with one region only one worker can ever get work, and the lease accounting
    // would be trivially right while the mesh was serial. (Measured the hard way twice — the
    // same configuration trap failed the debug gate earlier today.)
    GenSpec::new(
        [h, h, h],
        TableGrid::new(4, 4, 2, [2, 2, 2], (1.6, 2.2), (1.8, 2.4), (0.1, 0.5)),
    )
    .with_warm(WarmPolicy::CanonicalChain)
}

/// The happy path: every worker is leased through a REAL probe, every node is paid for, and the
/// books balance.
#[test]
fn a_leased_run_pays_for_every_node_and_balances_its_books() {
    let spec = tiny_spec();
    let mut arena = Arena::new();
    let mut probe = WorkerProbe::new();

    let run = generate_leased(&spec, 4, &mut arena, &mut probe).expect("the leased run refused");

    // One lease per worker, all released by the end.
    assert_eq!(run.leases.len(), 4);
    for l in &run.leases {
        assert_eq!(arena.get(*l).unwrap().state, LeaseState::Released);
    }

    // THE IDENTITY: opened == released + convicted + live, exact over integers.
    assert!(arena.balances(), "books did not balance: {:?}", arena.ledger());
    assert_eq!(arena.ledger().opened, 4);
    assert_eq!(arena.ledger().released, 4);
    assert_eq!(arena.ledger().convicted, 0);
    assert_eq!(arena.live_count(), 0);

    // RECEIPTS ARE THE RENT: the rent paid equals the work done, node for node.
    let nodes = run.outcome.records.len() as u64;
    assert_eq!(arena.ledger().rent.0, nodes);
    assert_eq!(run.progress().iter().sum::<u64>(), nodes);

    // M-VACUOUS-SUCCESS: the run must actually have done something, and with four workers on
    // four regions more than one of them must have.
    assert_eq!(nodes, 32, "the grid did not produce its nodes");
    assert_eq!(spec.grid.n_regions(), 4, "the grid must cut into more regions than one worker");
    assert!(
        run.progress().iter().filter(|n| **n > 0).count() >= 2,
        "only one worker did any work ({:?}); the lease accounting would be trivially right \
         while the mesh was serial",
        run.progress()
    );
    assert!(run.outcome.certificate.is_clean());
}

/// A refused worker probe refuses the RUN, and leaks nothing on the way out.
///
/// The partial-lease path is the one worth testing: the run takes some leases, then a later
/// probe refuses, and the leases already taken must be released rather than abandoned — an
/// abandoned lease is precisely the leak the ledger identity exists to convict.
#[test]
fn a_refused_worker_refuses_the_run_and_leaks_nothing() {
    let spec = tiny_spec();
    let mut arena = Arena::new();

    // Two workers admitted, then the OS says no.
    let mut flaky = ScriptedProbe {
        answers: vec![
            ProbeVerdict::Pass("worker started"),
            ProbeVerdict::Pass("worker started"),
        ],
        calls: 0,
        default: ProbeVerdict::Fail("the OS refused a thread"),
    };

    let err = generate_leased(&spec, 4, &mut arena, &mut flaky)
        .expect_err("the run proceeded despite a refused worker");
    assert!(err.message().contains("the OS refused a thread"), "{}", err.message());

    // The two leases that WERE taken are released, not abandoned.
    assert!(
        arena.balances(),
        "a partially-leased run leaked: {:?}",
        arena.ledger()
    );
    assert_eq!(arena.live_count(), 0);
    assert_eq!(arena.ledger().opened, 2);
    assert_eq!(arena.ledger().released, 2);
}

/// The real probe is genuinely consulted — it is not decoration around a spawn that would have
/// happened anyway.
#[test]
fn the_real_worker_probe_is_what_admits_the_lease() {
    let mut p = WorkerProbe::new();
    // It passes by ATTEMPTING: spawn, run, join.
    assert!(p.probe(ResourceKind::Worker, 1).passed());
    assert!(p.reported_parallelism > 0, "no parallelism reported at all");

    // And a run cannot be leased through a probe that refuses workers, however healthy the
    // machine is — the probe is the authority, not the machine's capacity.
    let spec = tiny_spec();
    let mut arena = Arena::new();
    let mut refuses = ScriptedProbe::always_fail("no");
    assert!(generate_leased(&spec, 1, &mut arena, &mut refuses).is_err());
    assert_eq!(arena.ledger().opened, 0);
    assert!(arena.balances());
}

/// The leased path produces the SAME table as the bare one. The resource layer schedules; it
/// does not touch the numbers.
#[test]
fn leasing_does_not_change_the_table() {
    let spec = tiny_spec();
    let bare = holon_tables::generate(&spec, 4);

    let mut arena = Arena::new();
    let mut probe = WorkerProbe::new();
    let leased = generate_leased(&spec, 4, &mut arena, &mut probe).unwrap();

    assert_eq!(
        bare.table_bytes(),
        leased.outcome.table_bytes(),
        "the table changed when it was generated under leases; the resource layer has reached \
         the numbers, which is the one thing it must never do"
    );
    assert_eq!(bare.digest(), leased.outcome.digest());
}

// ---------------------------------------------------------------------------
// The fold: the SAME leased pipeline over four atoms and six axes
// ---------------------------------------------------------------------------

use holon_tables::grid::{Axis, NdGrid, Serpentine};
use holon_tables::{
    generate_surface, generate_surface_leased, DistanceTetramer, NodeStatus, SurfaceSpec,
    VoidReason,
};

/// A 4-atom H4 surface over its six interatomic distances, on a grid small enough to be a
/// unit test and ragged enough to be a real one.
///
/// `[1.4, 3.0]` on every axis is chosen so the box straddles the embeddability boundary:
/// e.g. `d03 = 3.0` with `d01 = d13 = 1.4` violates a triangle inequality outright, so the
/// [`holon_tables::Realised::Continued`] branch is genuinely exercised rather than declared.
fn tetramer_grid() -> NdGrid {
    NdGrid::new(vec![
        Axis::linear(2, 1.4, 3.0, 2),
        Axis::linear(2, 1.4, 3.0, 2),
        Axis::linear(2, 1.4, 3.0, 2),
        Axis::linear(2, 1.4, 3.0, 1),
        Axis::linear(2, 1.4, 3.0, 1),
        Axis::linear(2, 1.4, 3.0, 1),
    ])
    // A new surface takes the rule that is actually adjacent; see `Serpentine`.
    .with_serpentine(Serpentine::Reflected)
}

/// **The deliverable.** Four atoms, six axes, through the SAME leased generator: every
/// worker probed and leased before any work starts, receipts paid node by node, the books
/// balanced and accounting for every node before the table is handed back.
#[test]
fn a_leased_four_body_run_pays_for_every_node_and_balances_its_books() {
    let h = by_symbol("H").unwrap();
    let surface = DistanceTetramer::new([h, h, h, h]);
    let spec = SurfaceSpec::new(&surface, tetramer_grid());
    assert_eq!(spec.grid.n_nodes(), 64);
    assert_eq!(
        spec.grid.n_regions(),
        8,
        "the grid must cut into more regions than one worker, or the lease accounting is \
         trivially right while the mesh is serial"
    );

    let mut arena = Arena::new();
    let mut probe = WorkerProbe::new();
    let run = generate_surface_leased(&spec, 4, &mut arena, &mut probe)
        .expect("the leased four-body run refused");

    assert_eq!(run.leases.len(), 4);
    for l in &run.leases {
        assert_eq!(arena.get(*l).unwrap().state, LeaseState::Released);
    }
    assert!(arena.balances(), "books did not balance: {:?}", arena.ledger());
    assert_eq!(arena.ledger().opened, 4);
    assert_eq!(arena.ledger().released, 4);
    assert_eq!(arena.ledger().convicted, 0);
    assert_eq!(arena.live_count(), 0);

    let nodes = run.outcome.records.len() as u64;
    assert_eq!(nodes, 64, "the six-axis grid did not produce its nodes");
    assert_eq!(arena.ledger().rent.0, nodes, "the rent did not account for every node");
    assert_eq!(run.progress().iter().sum::<u64>(), nodes);
    assert!(
        run.progress().iter().filter(|n| **n > 0).count() >= 2,
        "only one worker did any work ({:?})",
        run.progress()
    );
    assert!(run.outcome.certificate.is_clean());

    // THE THIRD CASE, asserted non-vacuous in BOTH directions. If nothing was continued the
    // branch is untested; if nothing was a geometry the surface is broken and "every node
    // VOIDed" would be passing for the wrong reason.
    let continued = run
        .outcome
        .records
        .iter()
        .filter(|r| r.status == NodeStatus::Void(VoidReason::NotAGeometry))
        .count();
    let refused = run
        .outcome
        .records
        .iter()
        .filter(|r| r.status == NodeStatus::Void(VoidReason::Unrealisable))
        .count();
    let scored = run.outcome.records.iter().filter(|r| r.is_ok()).count();
    assert!(
        continued > 0,
        "no node fell outside the embeddable region, so the Continued branch was never taken \
         and this test says nothing about it"
    );
    assert!(
        refused > 0,
        "no node was refused, so the Refused branch was never taken. On this box six corners \
         clamp two nuclei onto one point; if none does now, the separation guard is not \
         running and the next run will panic inside a worker thread instead."
    );
    assert!(scored > 0, "not one of the 64 nodes was a geometry that scored");
    // A refused node carries NO energy and NO solver's name: it was never solved.
    for r in run.outcome.records.iter() {
        if r.status == NodeStatus::Void(VoidReason::Unrealisable) {
            assert!(r.energy().is_nan(), "a refused node reported an energy");
            assert_eq!(r.exit_code, holon_tables::node::NOT_SOLVED_EXIT);
            assert_eq!(r.davidson_iters, 0);
        }
    }
    // A continued node is excluded from any accuracy statistic BY CONSTRUCTION: it carries
    // its status in the record, so `is_ok` is false and no filter has to remember it.
    assert!(
        run.outcome
            .records
            .iter()
            .all(|r| r.status != NodeStatus::Void(VoidReason::NotAGeometry) || !r.is_ok()),
        "a continued node reported itself as a scored one"
    );
    println!(
        "four-body leased run: {nodes} nodes, {scored} scored, {continued} continued (not a \
         geometry), {refused} refused (unrealisable), {} other VOID; receipts {:?}; digest {}",
        run.outcome.voided - continued - refused,
        run.progress(),
        run.outcome.digest().hex()
    );
}

/// **G1 on six axes.** The four-body table is bit-identical across worker counts, which is
/// the whole claim the mesh exists to make and it now has to hold for a composition the
/// generator was never written for.
#[test]
fn the_four_body_table_is_bit_identical_across_worker_counts() {
    let h = by_symbol("H").unwrap();
    let surface = DistanceTetramer::new([h, h, h, h]);
    let spec = SurfaceSpec::new(&surface, tetramer_grid());

    let one = generate_surface(&spec, 1);
    let four = generate_surface(&spec, 4);
    let many = generate_surface(&spec, 8);

    for (label, run) in [("1", &one), ("4", &four), ("8", &many)] {
        assert!(
            run.certificate.is_clean(),
            "the {label}-worker four-body run's certificate was not clean: {:?}",
            run.certificate
        );
        assert_eq!(run.records.len(), 64);
        assert!(run.total_davidson_iters > 0, "the {label}-worker run did no Davidson work");
    }
    assert_eq!(one.table_bytes(), four.table_bytes(), "1 and 4 workers disagree");
    assert_eq!(one.table_bytes(), many.table_bytes(), "1 and 8 workers disagree");
    assert_eq!(one.digest(), four.digest());
    assert_eq!(one.digest(), many.digest());
    println!(
        "four-body table bit-identical over {} nodes at 1/4/8 workers; digest {}",
        one.records.len(),
        one.digest().hex()
    );
}

/// The leased four-body path produces the same table as the bare one, exactly as the 3-body
/// pair does: the resource layer schedules, it does not touch the numbers.
#[test]
fn leasing_does_not_change_the_four_body_table() {
    let h = by_symbol("H").unwrap();
    let surface = DistanceTetramer::new([h, h, h, h]);
    let spec = SurfaceSpec::new(&surface, tetramer_grid());

    let bare = generate_surface(&spec, 4);
    let mut arena = Arena::new();
    let mut probe = WorkerProbe::new();
    let leased = generate_surface_leased(&spec, 4, &mut arena, &mut probe).unwrap();

    assert_eq!(
        bare.table_bytes(),
        leased.outcome.table_bytes(),
        "the four-body table changed when it was generated under leases"
    );
    assert_eq!(bare.digest(), leased.outcome.digest());
}
