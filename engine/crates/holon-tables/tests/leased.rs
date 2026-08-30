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
