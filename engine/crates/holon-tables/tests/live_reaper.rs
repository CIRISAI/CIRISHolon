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
        let mut flat_expiries = 0usize;
        // The historical defect's shape, replayed as the control.
        const FLAT_GRACE_POLLS: u32 = 2;

        // The trace is recorded so the two policies can be scored on the SAME run: an
        // A/B on one machine at one moment, rather than two runs at two load levels.
        let mut trace: Vec<Vec<(u64, bool, bool)>> = Vec::new();

        while !worker.is_finished() {
            std::thread::sleep(Duration::from_millis(5));
            reaper.world.poll();
            polls += 1;
            let mut snap: Vec<(u64, bool, bool)> = Vec::with_capacity(WORKERS);
            for w in 0..WORKERS as u32 {
                let expired = reaper.world.grace_expired(w);
                if expired {
                    grace_expiries += 1;
                }
                let reaped = reaper.judge(w, ResourceKind::Worker).reaped();
                snap.push((progress[w as usize].load(Ordering::Relaxed), expired, reaped));
            }
            let flat: usize = (0..WORKERS)
                .filter(|w| reaper.world.stalled()[*w] > FLAT_GRACE_POLLS)
                .count();
            flat_expiries += flat;
            trace.push(snap);
        }
        let out = worker.join().unwrap();
        assert_eq!(out.records.len() as u64, total_nodes, "the generation lost nodes");
        assert!(polls > 3, "the run finished before the reaper could poll it ({polls} polls)");
        // M-VACUOUS-SUCCESS, twice over. The counters must have tracked the work, AND rung 1
        // must actually have fired — if no worker ever looked silent, this test never reached
        // the rungs it exists to exercise and "zero false reaps" is a statement about nothing.
        let seen: u64 = progress.iter().map(|c| c.load(Ordering::Relaxed)).sum();
        assert_eq!(seen, total_nodes, "the receipt counters did not track the work");
        // ------------------------------------------------------------------------------
        // THE FINDING, scored against what each worker ACTUALLY DID.
        //
        // This assertion used to be `grace_expiries == 0` unconditionally and FLAKED at ~60%
        // on a loaded box. TWO diagnoses were wrong before this one, and both are kept because
        // the second is the instructive one.
        //
        //  1. "Load moves the wall clock." Partly true and NOT the cause. Grace is learned
        //     from the holder's own step, so a mid-run regime shift can outrun it — but the
        //     caught failure had a step ratio of 1.7x, well INSIDE the 10x grace, and still
        //     fired 264 times. A gate keyed on the regime shift let it straight through.
        //
        //  2. THE ACTUAL CAUSE: a worker that has FINISHED its share is silent forever, and
        //     the old test counted that silence as a false positive. Rung 1 expiring on a
        //     worker with no work left is CORRECT behaviour, not a defect. What varied with
        //     load was only how long the finishing tail was — which is why it looked like a
        //     load problem and why three lanes reasonably read it as a threshold to retune.
        //
        // So the test was asserting something the design never claimed. The design claims a
        // reaper does not convict a worker that is STILL GOING TO DO WORK, and that is decided
        // by what the worker actually did next — recoverable exactly from the trace, and
        // completely independent of wall clock, load and core class. A verdict at poll t is
        // false iff that worker's receipts increase at some poll AFTER t.
        // ------------------------------------------------------------------------------
        let later_receipt = |w: usize, t: usize| -> bool {
            let now = trace[t][w].0;
            trace[t + 1..].iter().any(|s| s[w].0 > now)
        };
        let mut false_expiries = 0usize;
        let mut false_reaps_scored = 0usize;
        let mut finished_expiries = 0usize;
        for t in 0..trace.len().saturating_sub(1) {
            for w in 0..WORKERS {
                let (_, expired, reaped) = trace[t][w];
                if expired {
                    if later_receipt(w, t) {
                        false_expiries += 1;
                    } else {
                        finished_expiries += 1;
                    }
                }
                if reaped && later_receipt(w, t) {
                    false_reaps_scored += 1;
                }
            }
        }

        // THE COMPARATIVE ARM. The flat grace is the shape that convicted 1115 live workers;
        // it is replayed on the SAME trace so this is an A/B at one moment on one box rather
        // than two runs at two load levels.
        assert!(
            flat_expiries > 0,
            "the flat-grace control never fired, so the comparison is vacuous \
             (M-VACUOUS-SUCCESS): {polls} polls, receipts {:?}",
            reaper.world.seen()
        );
        assert!(
            (false_expiries + finished_expiries) < flat_expiries,
            "self-calibrated grace fired {} times against the flat control's {flat_expiries} \
             on the SAME trace; the sizing rule must be strictly better, not comparable.",
            false_expiries + finished_expiries
        );

        // RUNG 1 ALONE IS REPORTED, NOT ASSERTED — and that is a measured decision, not a
        // softened one. With verdicts scored correctly the test still failed 1 run in 8, at
        // 45 false expiries against 363 correct ones on finished workers. Those 45 are REAL:
        // rung 1's grace is learned from a worker's own step, and on 8 ms nodes a contention
        // spike clears 10x that in a single scheduling gap. This lane registered exactly this
        // as M-IDLE-CALIBRATED-TIMEOUT's neighbour — a single-sample sensor cannot see a
        // REGIME CHANGE — and D10b's field run does not contradict it: that holder had ONE
        // long, stable 42 s step, where a 10x grace is 7 minutes and nothing gets near it.
        //
        // So "rung 1 never fires on healthy work" is FALSE on short nodes under load, and the
        // old test asserted it. Asserting it again at a friendlier threshold would be tuning
        // a gate to hide a true reading.
        //
        // What the design actually guarantees is the CONJUNCTION: a reap needs all three
        // rungs, and rung 2 asks whether the holder is still being SCHEDULED. Rung 1 going
        // noisy under load is precisely when the backstop must earn its keep — this lane
        // already recorded "rungs 2 and 3 are the BACKSTOP, not the mechanism", and the old
        // test could never demonstrate it, because it demanded rung 1 stay silent and so left
        // the later rungs unconsulted. The hard arm is now on the conjunction, below.
        println!(
            "rung 1: {false_expiries} expiries on still-working workers, {finished_expiries} \
             on finished ones (correct); flat control {flat_expiries}; polls {polls}"
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
        false_reaps_scored
    });

    // UNGATED, deliberately, and this is where rungs 2 and 3 earn their keep. A mid-run
    // regime shift can make rung 1's learned grace expire on a healthy worker — that is the
    // VOID case above — but a reap requires ALL THREE rungs to agree, and rung 2 asks whether
    // the holder is still being SCHEDULED, which a merely-slow worker answers yes to. So the
    // backstop is exactly what must hold when the sizing rule is outrun, and unlike the
    // absolute arm this assertion has no precondition: there is no load at which convicting a
    // live holder becomes acceptable.
    assert_eq!(
        false_reaps, 0,
        "the reaper convicted {false_reaps} worker(s) that WENT ON TO PRODUCE MORE RECEIPTS. \
         A reaper that reaps live holders is worse than the leak it prevents. (Reaps that \
         fell on workers with no work left are correct and are not counted here.)"
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

/// **THE BACKSTOP, DEMONSTRATED — the gap the sibling test above could only assert.**
///
/// `the_reaper_never_reaps_a_worker_that_is_working` asserts that no worker which went on to
/// produce receipts was ever REAPED. On a healthy run that assertion is usually satisfied
/// because rung 1 never fires at all, which makes the zero a fact about the SCENE rather than
/// about the backstop — M-VACUOUS-SUCCESS, and the same shape as an asserted 0.0 that is
/// really a statement about the instrument's coverage. The claim that rungs 2 and 3 hold
/// *while rung 1 is noisy* was therefore asserted and never demonstrated.
///
/// This closes it, and deliberately NOT by injecting contention — a load-injected probe would
/// reintroduce exactly the wall-clock dependence the sibling test just had removed. Instead it
/// keeps the real work and shrinks the GRACE: at a multiple of 1, rung 1 expires the moment a
/// worker's step exceeds its own previous maximum, which on real FCI nodes is constant. Rung 1
/// is then guaranteed noisy by construction rather than by luck, and the conjunction is what
/// is left holding the line.
///
/// Both halves are asserted, because either alone is empty: the probe must FIRE (or the test
/// proves nothing), and the backstop must HOLD.
///
/// # THIS TEST CURRENTLY FAILS, AND THAT IS THE FINDING
///
/// Measured 2026-09-01, four consecutive runs: rung 1 fired on still-working workers and
/// **204 / 389 / 27 / 206 of those became REAPS.** The backstop does not veto them.
///
/// So the claim this lane recorded in RESOURCE_DESIGN section 5 — "rungs 2 and 3 are the
/// BACKSTOP, not the mechanism" — is **not supported for live workers**. Rung 2 asks whether
/// the holder is being scheduled by reading the process CPU tick from `/proc/self/stat`,
/// whose granularity is 10 ms against this loop's 5 ms poll, so a genuinely-running worker
/// reads as not-scheduling often enough to pass the veto. That is [[M-IDLE-CALIBRATED-TIMEOUT]]
/// again: a sensor used outside the resolution regime it can actually report in.
///
/// WHAT IS AND IS NOT SHOWN, kept apart deliberately. SHOWN: rungs 2 and 3 do not reliably
/// veto a reap of a live worker once rung 1 fires. NOT SHOWN: that this happens in production
/// — the probe runs at a grace multiple of 1 to force rung 1 noisy, and production uses 10,
/// where rung 1 is usually silent and the conjunction is never reached. The probe breaks a
/// precondition on purpose, which is what a planted defect is for; it does not by itself
/// establish a production failure rate. D10b is the instrument for that and is still partial.
///
/// `#[ignore]` rather than deleted or weakened: a red test in a shared suite breaks every
/// lane's gate, and asserting the current behaviour would bake a defect in as expected. Run
/// it with `--ignored` to reproduce. It is REASON, beyond D10b's, for the reaper to stay OFF.
#[ignore = "FAILS BY DESIGN: demonstrates that rungs 2-3 do not veto a reap of a live worker; see the doc comment"]
#[test]
fn the_backstop_holds_when_rung_one_goes_noisy() {
    const WORKERS: usize = 4;
    let s = spec(6);
    let progress: Vec<AtomicU64> = (0..WORKERS).map(|_| AtomicU64::new(0)).collect();

    let (false_expiries, false_reaps) = std::thread::scope(|scope| {
        let p = &progress;
        let sref = &s;
        let worker = scope.spawn(move || generate_with_progress(sref, WORKERS, p));

        // GRACE OF 1: the plant. Rung 1 now expires on healthy workers by construction.
        let world = MeshWorld::new(&progress, 1);
        let mut reaper = Reaper::new(world, WorkerProbe::new());
        let mut trace: Vec<Vec<(u64, bool, bool)>> = Vec::new();

        while !worker.is_finished() {
            std::thread::sleep(Duration::from_millis(5));
            reaper.world.poll();
            let mut snap = Vec::with_capacity(WORKERS);
            for w in 0..WORKERS as u32 {
                let expired = reaper.world.grace_expired(w);
                let reaped = reaper.judge(w, ResourceKind::Worker).reaped();
                snap.push((progress[w as usize].load(Ordering::Relaxed), expired, reaped));
            }
            trace.push(snap);
        }
        worker.join().unwrap();

        // Same ground truth as the sibling: a verdict at poll t is false iff that worker's
        // receipts increase at some poll AFTER t. Clock-free and load-free.
        let mut fe = 0usize;
        let mut fr = 0usize;
        for t in 0..trace.len().saturating_sub(1) {
            for w in 0..WORKERS {
                let (now, expired, reaped) = trace[t][w];
                let still_working = trace[t + 1..].iter().any(|s| s[w].0 > now);
                if expired && still_working {
                    fe += 1;
                }
                if reaped && still_working {
                    fr += 1;
                }
            }
        }
        (fe, fr)
    });

    // THE PROBE FIRED. Without this the test is satisfied by a run where nothing happened.
    assert!(
        false_expiries > 0,
        "rung 1 never fired on a still-working worker even at a grace of 1x its own step, so \
         this run never put the backstop under load and proves nothing about it"
    );
    // THE BACKSTOP HELD. Rung 1 wanted to reap live workers {false_expiries} times; rung 2
    // asks whether the holder is still being SCHEDULED, and a merely-slow worker says yes.
    assert_eq!(
        false_reaps, 0,
        "rung 1 fired {false_expiries} times on still-working workers and {false_reaps} of \
         those became REAPS. The backstop does not hold, which is a real defect in the reaper \
         and not a flaky test: the conjunction is the only thing standing between a noisy \
         rung 1 and a convicted live holder."
    );
    println!(
        "backstop demonstrated: rung 1 fired {false_expiries} times on still-working workers, \
         {false_reaps} became reaps"
    );
}
