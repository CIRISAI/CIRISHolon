//! The reaper, wired to the mesh's LIVE progress counters — real long-running holders instead
//! of scripted ones.
//!
//! `holon-resource::Reaper` takes its rungs 1 and 2 from a [`ReaperWorld`] the pool owner
//! supplies, because only the pool knows what its holders are supposed to be doing. This is that
//! world for table generation.
//!
//! # The three rungs, made concrete
//!
//! | rung | question | how it is answered here |
//! |---|---|---|
//! | 1 | has the holder gone silent past its grace? | its receipt counter has not advanced in `grace_polls` consecutive polls |
//! | 2 | is the holder alive and scheduling? | the PROCESS's CPU time is advancing (`/proc/self/stat`) |
//! | 3 | can *I* do this operation right now? | [`crate::WorkerProbe`] — the reaper probes itself |
//!
//! # Why rung 2 asks about the PROCESS and not the worker
//!
//! It would be neater to ask "is this worker thread scheduling". It is also not answerable
//! cheaply, and — more to the point — it would collapse into rung 1: a worker that is scheduling
//! but producing nothing looks identical to one that is stuck, from inside. Asking about the
//! process separates two genuinely different situations:
//!
//! * counter frozen, process burning CPU  -> this WORKER is stuck while others run. Reapable, if
//!   rung 3 agrees the machine is healthy.
//! * counter frozen, process not advancing -> the whole process is blocked (the disk-full shape).
//!   Rung 3 will catch it, and rung 2 gives the reaper a reason to look before it gets there.
//!
//! # What this is not
//!
//! A poller that reaps table workers in production. Nothing in the generator calls it yet — a
//! reaper that can kill the campaign's own tables is not something to switch on the same evening
//! it is written. It exists so the ladder can be exercised against holders that are really
//! running, and so the false-positive rate against real work is a measurement rather than a hope.

use std::sync::atomic::{AtomicU64, Ordering};

use holon_resource::{LeaseId, ReaperWorld};

/// Reads the mesh's live receipt counters and the process's own CPU time.
pub struct MeshWorld<'a> {
    progress: &'a [AtomicU64],
    /// Last value seen per worker, and how many consecutive polls it has been unchanged.
    last: Vec<u64>,
    stalled_polls: Vec<u32>,
    /// The longest silence this worker has ever ENDED with a receipt — its observed step, in
    /// polls. Learned, not configured.
    max_gap_polls: Vec<u32>,
    /// How many of the holder's OWN observed steps of silence before grace expires.
    ///
    /// The design's rule is "a multiple of the holder's own declared step time, not a global
    /// constant", and this is that rule made mechanical: an (O,O,O) node takes ~50 s and an H3
    /// node ~8 ms, so no wall-clock threshold serves both — but "ten times longer than this
    /// worker's own worst step so far" serves both without being told which it is.
    grace_multiple: u32,
    last_cpu: u64,
    /// Consecutive polls in which the process CPU clock did not advance.
    ///
    /// NOT a fact about the holder until it exceeds [`CPU_SENSOR_POLLS`]: see that constant.
    cpu_stalled_polls: u32,
}

/// How many consecutive polls of a flat CPU clock are needed before rung 2 may say "not
/// scheduling".
///
/// **This is a statement about the SENSOR, not about the holder**, and it was measured rather
/// than guessed. `/proc/self/stat` reports CPU in USER_HZ ticks — 10 ms on this machine — so a
/// reaper polling faster than that will frequently see a flat counter on a process that is
/// burning CPU flat out. A rung whose sensor is COARSER than its sampling rate manufactures
/// false negatives at exactly the rate the two disagree.
///
/// Measured on a real 72-node generation: with rung 2 asking "did the tick advance since the
/// last poll?" at a 5 ms interval, the reaper convicted **108 workers that were doing real
/// work**. Debounced over four polls — comfortably past one 10 ms tick at a 5 ms interval — it
/// convicts none. (Before rung 2 existed at all the count was 1115, so the rung was already
/// carrying most of the load; this closes the rest.)
pub const CPU_SENSOR_POLLS: u32 = 4;

impl<'a> MeshWorld<'a> {
    /// `grace_multiple` is how many of a holder's OWN observed steps of silence are tolerated.
    pub fn new(progress: &'a [AtomicU64], grace_multiple: u32) -> MeshWorld<'a> {
        let n = progress.len();
        MeshWorld {
            progress,
            last: progress.iter().map(|c| c.load(Ordering::Relaxed)).collect(),
            stalled_polls: vec![0; n],
            max_gap_polls: vec![0; n],
            grace_multiple,
            last_cpu: process_cpu_ticks(),
            cpu_stalled_polls: 0,
        }
    }

    /// Sample every counter and the process clock. Call this once per poll, before judging.
    pub fn poll(&mut self) {
        for (i, c) in self.progress.iter().enumerate() {
            let now = c.load(Ordering::Relaxed);
            if now == self.last[i] {
                self.stalled_polls[i] += 1;
            } else {
                // A receipt landed: the silence that just ended IS an observation of this
                // worker's step, so it teaches the grace what this holder's rhythm looks like.
                self.max_gap_polls[i] = self.max_gap_polls[i].max(self.stalled_polls[i]);
                self.stalled_polls[i] = 0;
                self.last[i] = now;
            }
        }
        let cpu = process_cpu_ticks();
        if cpu > self.last_cpu {
            self.cpu_stalled_polls = 0;
            self.last_cpu = cpu;
        } else {
            self.cpu_stalled_polls = self.cpu_stalled_polls.saturating_add(1);
        }
    }

    /// Receipts seen so far, per worker.
    pub fn seen(&self) -> &[u64] {
        &self.last
    }

    /// Consecutive silent polls, per worker. The reading rung 1 is made of.
    pub fn stalled(&self) -> &[u32] {
        &self.stalled_polls
    }

    /// Consecutive polls with a flat process CPU clock. Rung 2's raw reading, exposed so a
    /// caller can tell "the process is idle" from "the sensor has not ticked yet".
    pub fn cpu_stalled_polls(&self) -> u32 {
        self.cpu_stalled_polls
    }

    /// Each worker's learned step, in polls — the longest silence it has ended with a receipt.
    /// This is what grace is a multiple OF, and it is worth logging beside any reaping.
    pub fn observed_step_polls(&self) -> &[u32] {
        &self.max_gap_polls
    }
}

impl ReaperWorld for MeshWorld<'_> {
    fn grace_expired(&mut self, id: LeaseId) -> bool {
        let Some(&stalled) = self.stalled_polls.get(id as usize) else {
            return false;
        };
        let observed = self.max_gap_polls[id as usize];
        // WARMUP: before this worker has completed a single step, its rhythm is unknown and
        // its silence means nothing. Judging a holder before you know its step is how a
        // reaper convicts a slow starter — so grace cannot expire here at all.
        if observed == 0 && self.last[id as usize] == 0 {
            return false;
        }
        stalled > observed.max(1) * self.grace_multiple
    }

    fn holder_scheduling(&mut self, _id: LeaseId) -> bool {
        // The process is doing work. Debounced over CPU_SENSOR_POLLS because a flat tick across
        // one poll is a statement about the 10 ms sensor, not about the process — see that
        // constant for the measurement that forced this.
        self.cpu_stalled_polls < CPU_SENSOR_POLLS
    }
}

/// This process's total CPU ticks (utime + stime) from `/proc/self/stat`, or 0 where that is
/// unreadable — in which case rung 2 reports "not scheduling", which makes the reaper LOOK
/// harder rather than reap faster. Failing safe here means failing toward rung 3.
fn process_cpu_ticks() -> u64 {
    let Ok(s) = std::fs::read_to_string("/proc/self/stat") else {
        return 0;
    };
    // Fields 14 and 15 (1-based) are utime and stime, after the comm field which may contain
    // spaces — so split after the closing parenthesis rather than on whitespace from the start.
    let Some(rest) = s.rsplit_once(')') else {
        return 0;
    };
    let f: Vec<&str> = rest.1.split_whitespace().collect();
    match (f.get(11), f.get(12)) {
        (Some(u), Some(k)) => {
            u.parse::<u64>().unwrap_or(0) + k.parse::<u64>().unwrap_or(0)
        }
        _ => 0,
    }
}
