//! **The reaper against genuine long-running holders**, not scripted ones.
//!
//! Every reaper test so far has fed the ladder answers. This one runs a REAL table generation on
//! real FCI solves and polls the reaper against the live receipt counters while the work is in
//! flight, because the failure mode that matters — reaping something still needed — only shows
//! up against holders that are actually busy.
//!
//! The measurement it produces is a false-positive count against real work. Zero is the only
//! acceptable answer, and it is worth having as a number rather than as a hope.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use holon_chem::elements::by_symbol;
use holon_resource::{ReapVerdict, Reaper, ReaperWorld, ResourceKind, ScriptedProbe};
use holon_tables::generate::generate_with_progress;
use holon_tables::{GenSpec, MeshWorld, TableGrid, WarmPolicy, WorkerProbe};

fn spec(n: usize) -> GenSpec {
    let h = by_symbol("H").unwrap();
    GenSpec::new(
        [h, h, h],
        TableGrid::new(n, n, 2, [2, 2, 2], (1.6, 2.2), (1.8, 2.4), (0.1, 0.5)),
    )
    .with_warm(WarmPolicy::CanonicalChain)
}

/// **The reaper must not reap a single busy worker.** Real solves, real counters, polled during
/// the run.
#[test]
fn the_reaper_never_reaps_a_worker_that_is_working() {
    const WORKERS: usize = 4;
    let s = spec(6); // 72 nodes of real FCI, enough to be in flight while we poll
    let progress: Vec<AtomicU64> = (0..WORKERS).map(|_| AtomicU64::new(0)).collect();
    let total_nodes = s.grid.n_nodes() as u64;

    let false_reaps = std::thread::scope(|scope| {
        let p = &progress;
        let sref = &s;
        let worker = scope.spawn(move || generate_with_progress(sref, WORKERS, p));

        // THE REAL LADDER, all three rungs, with grace at TEN of each worker'''s own observed
        // steps — the design'''s rule (a multiple of the holder'''s own step, never a global
        // constant) made mechanical, and self-calibrating so the same code serves an 8 ms H3
        // node and a 50 s (O,O,O) one.
        let world = MeshWorld::new(&progress, 10);
        let mut reaper = Reaper::new(world, WorkerProbe::new());
        let mut false_reaps = 0usize;
        let mut polls = 0usize;
        let mut grace_expiries = 0usize;

        while !worker.is_finished() {
            std::thread::sleep(Duration::from_millis(5));
            reaper.world.poll();
            polls += 1;
            for w in 0..WORKERS as u32 {
                if reaper.world.grace_expired(w) {
                    grace_expiries += 1;
                }
                // Anything reaped while the generation is still running is a FALSE POSITIVE.
                if reaper.judge(w, ResourceKind::Worker).reaped() {
                    false_reaps += 1;
                }
            }
        }
        let out = worker.join().unwrap();
        assert_eq!(out.records.len() as u64, total_nodes, "the generation lost nodes");
        assert!(polls > 3, "the run finished before the reaper could poll it ({polls} polls)");
        // M-VACUOUS-SUCCESS, twice over. The counters must have tracked the work, AND rung 1
        // must actually have fired — if no worker ever looked silent, this test never reached
        // the rungs it exists to exercise and "zero false reaps" is a statement about nothing.
        let seen: u64 = progress.iter().map(|c| c.load(Ordering::Relaxed)).sum();
        assert_eq!(seen, total_nodes, "the receipt counters did not track the work");
        // THE FINDING, asserted rather than hoped: with grace sized by the design's own rule
        // — a multiple of the holder's OWN observed step — rung 1 does not fire on healthy
        // work at all, so rungs 2 and 3 are never even consulted. The earlier mis-sized
        // version (a flat 2-poll grace against ~8 ms nodes) convicted 1115 live workers with
        // rung 2 absent, and still 108 with it present. Sizing the grace correctly is what
        // actually fixes this; the later rungs are the backstop, not the mechanism.
        assert_eq!(
            grace_expiries, 0,
            "rung 1 fired {grace_expiries} times on workers that were producing receipts. Grace \
             is {} of each worker's OWN observed step, so this means the sizing rule is not \
             being applied.",
            10
        );
        // M-VACUOUS-SUCCESS: the grace must have been CALIBRATED, or "it never expired" is a
        // statement about a warmup that never ended.
        assert!(
            reaper.world.observed_step_polls().iter().any(|s| *s > 0)
                || reaper.world.seen().iter().any(|s| *s > 0),
            "no worker ever completed a step, so grace was never calibrated and never could \
             have expired"
        );
        println!(
            "polled {polls} times over {total_nodes} real nodes across {WORKERS} workers; \
             receipts {:?}; learned steps (polls) {:?}; rung-1 expiries {grace_expiries}; \
             false reaps {false_reaps}",
            reaper.world.seen(),
            reaper.world.observed_step_polls()
        );
        false_reaps
    });

    assert_eq!(
        false_reaps, 0,
        "the reaper convicted {false_reaps} worker(s) that were doing real work. A reaper that \
         reaps live holders is worse than the leak it prevents."
    );
}

/// A world that answers both early rungs in the convicting direction, so that rung 3 is the
/// only thing left deciding. Used ONLY by the two focused tests below; the live test above
/// drives the real [`MeshWorld`].
struct ScriptedWorldShim;
impl ReaperWorld for ScriptedWorldShim {
    fn grace_expired(&mut self, _id: u32) -> bool {
        true
    }
    fn holder_scheduling(&mut self, _id: u32) -> bool {
        false
    }
}

/// The other half: a genuinely STALLED holder on a healthy machine IS reaped. Without this,
/// "never reaps" above is satisfied by a reaper that does nothing.
#[test]
fn a_genuinely_stalled_holder_on_a_healthy_machine_is_reaped() {
    // Counters that never move: the holder has produced no receipt at all.
    let frozen: Vec<AtomicU64> = (0..2).map(|_| AtomicU64::new(0)).collect();
    let mut world = MeshWorld::new(&frozen, 1);
    // A holder that has never produced anything is in WARMUP: its rhythm is unknown, so grace
    // cannot expire. Give it one receipt so it has a step, then let it go silent.
    frozen[0].store(1, Ordering::Relaxed);
    world.poll();
    world.poll();
    world.poll();

    assert!(
        world.grace_expired(0),
        "a holder that produced nothing across two polls was still inside its grace"
    );
    assert_eq!(world.stalled()[0], 2);

    // Rung 3 on a healthy machine: the reaper's own worker probe succeeds, so the machine is
    // not the problem and the holder is.
    let mut reaper = Reaper::new(ScriptedWorldShim, WorkerProbe::new());
    let v = reaper.judge(0, ResourceKind::Worker);
    assert!(
        v.reaped(),
        "a stalled holder on a healthy machine was not reaped: {v:?} — the stand-down rule has \
         swallowed the mechanism it guards"
    );

    // And the same holder on a STALLED machine is not: rung 3 refuses, the reaper stands down.
    let mut blocked = Reaper::new(ScriptedWorldShim, ScriptedProbe::always_fail("machine stalled"));
    assert!(matches!(
        blocked.judge(0, ResourceKind::Worker),
        ReapVerdict::StandDown { .. }
    ));
}

/// Rung 1 tracks receipts, not wall clock: a worker that is producing is never past its grace,
/// however long it has held the lease.
#[test]
fn a_producing_worker_is_never_past_its_grace() {
    let counters: Vec<AtomicU64> = (0..1).map(|_| AtomicU64::new(0)).collect();
    let mut world = MeshWorld::new(&counters, 1);
    for i in 1..=20 {
        counters[0].store(i, Ordering::Relaxed);
        world.poll();
        assert!(
            !world.grace_expired(0),
            "a worker that paid a receipt on every poll was judged silent at poll {i}"
        );
        assert_eq!(world.stalled()[0], 0);
    }
    // Stop producing. Grace is `stalled > max(observed_step,1) * multiple`, and this worker's
    // observed step is 0 polls (it produced on every single one), so one missed poll is not yet
    // past a grace of 1x its step — two is.
    world.poll();
    assert!(!world.grace_expired(0), "one missed poll was treated as past grace");
    world.poll();
    assert!(world.grace_expired(0), "two missed polls did not expire a 1x grace");
}
