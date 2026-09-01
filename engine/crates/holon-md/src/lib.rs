//! # holon-md
//!
//! **The molecular-dynamics force evaluation, run on leased workers, producing bit-for-bit
//! the numbers the serial run produces.**
//!
//! Operator order, T3: single-threaded dynamics is banned. This crate is how the ban is
//! obeyed without buying it with reproducibility — which is the trade almost every
//! parallel force loop makes, usually without saying so.
//!
//! ## Why the answer does not depend on the worker count
//!
//! Floating-point addition is not associative, so `(a + b) + c` and `a + (b + c)` are
//! different numbers. A force loop that lets each worker accumulate into a shared array
//! therefore produces a different total for every scheduling, and a run sharded four ways
//! disagrees with the same run sharded eight ways in the last bits — which then grows,
//! because molecular dynamics is chaotic. That is not a rounding detail; it means a
//! checkpoint cannot replay and a bug cannot be reproduced.
//!
//! The engine avoids it by splitting the loop in two (see `holon_render::sim::PairTerm`):
//!
//! * **EVALUATION** — the interpolants, the switch, the composition dispatch. Pure,
//!   `&self`, no accumulation, and essentially all of the cost. Term `k` depends on
//!   nothing but the state and pair `k`, so it can be computed anywhere.
//! * **ACCUMULATION** — the sums. Walks the terms in index order, on one thread, always.
//!
//! So the workers decide WHEN each term is computed and never in what order the terms are
//! added. `tests/bit_identity.rs` holds one configuration against 1, 2, 3, 5 and 8 workers
//! and requires the bits to agree, and against the serial reference as well.
//!
//! The corollary is worth stating because it bounds the speedup honestly: the accumulation
//! is serial and is `O(P)`, so Amdahl's law applies to it. It is a handful of adds against
//! hundreds of flops of interpolation, so the serial fraction is small — but it is not
//! zero, and this crate reports the measured scaling rather than claiming linearity.
//!
//! ## Workers are LEASED, not spawned
//!
//! `holon-resource`'s discipline (RESOURCE-1, D1/D2): probe by ATTEMPTING the thing, never
//! by asking what the machine is configured to offer. `holon_tables::WorkerProbe` spawns a
//! thread, runs it and joins it — so a pool of eight here means the OS has actually
//! produced eight threads, not that `available_parallelism` said so. The books balance
//! (`opened == released + convicted + live`) before the pool is considered healthy, and a
//! refused probe refuses the POOL rather than silently running with fewer.
//!
//! The rent is real work: each worker pays a receipt of the terms it evaluated. A worker
//! that evaluated nothing paid nothing, and the ledger says so — which is also how
//! `tests/leased.rs` catches a pool that is nominally eight and effectively one.

use holon_render::sim::{ForceExecutor, PairTerm, Sim, TripleTerm};
use holon_resource::{Arena, LeaseError, LeaseId, Receipt, ResourceKind};
use holon_tables::WorkerProbe;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Why a pool could not be built. A refusal, never a quiet downgrade: a pool that asked for
/// eight workers and got three is a different machine from the one the caller described,
/// and the caller is the one who gets to decide what to do about it.
#[derive(Debug)]
pub enum PoolError {
    /// The resource layer refused a worker lease. Carries the underlying refusal, which
    /// names what was probed and why it failed.
    Refused(LeaseError),
    /// Fewer than one worker was asked for.
    Empty,
}

impl PoolError {
    pub fn plain(&self) -> String {
        match self {
            PoolError::Refused(e) => format!(
                "a worker lease was refused, so the pool was not built: {}",
                e.message()
            ),
            PoolError::Empty => "a pool of zero workers is not a pool".to_string(),
        }
    }
}

/// A pool of leased workers that evaluates force terms.
///
/// Holds its leases for its lifetime and releases them on [`WorkerPool::retire`]. It does
/// NOT hold threads: `std::thread::scope` starts them per evaluation and joins them before
/// returning, which is what lets the workers borrow `&Sim` without an `Arc` and without a
/// lifetime that outlives the scene.
///
/// The cost of starting a thread per evaluation is real and is measured rather than waved
/// at — see `examples/scaling.rs`. On this box it is tens of microseconds against a force
/// evaluation of milliseconds at the scales T3 exists for, which is why the simpler
/// scoped-thread design was chosen over a resident pool with a channel. At smaller scenes
/// it is NOT negligible and the pool says so: [`WorkerPool::worth_it`] is the honest
/// predicate, and a caller below it should stay serial.
pub struct WorkerPool {
    leases: Vec<LeaseId>,
    arena: Arena,
    /// Terms evaluated by each worker SINCE THE LAST RESET — cumulative, not per-call.
    ///
    /// It was per-call, and that was wrong twice over. The rent is work done, so a receipt
    /// written from the last evaluation's count charges for one frame of a run that did
    /// hundreds. And the M-VACUOUS-SUCCESS reading — did more than one worker do anything —
    /// is a question about the RUN: at a chunk count near the worker count, one thread can
    /// legitimately claim every chunk of a single evaluation before the others have
    /// started, and a per-call reading calls that a serial pool when it is a small job.
    progress: Vec<AtomicUsize>,
}

impl WorkerPool {
    /// Build a pool of `workers`, each admitted by a REAL probe.
    ///
    /// The probe spawns a thread, makes it do a scrap of work, and joins it. If the OS will
    /// not give us the eighth thread, this returns `Refused` with that fact rather than a
    /// pool of seven — a capacity reading is not a probe, and neither is a partial success.
    pub fn new(workers: usize) -> Result<WorkerPool, PoolError> {
        if workers == 0 {
            return Err(PoolError::Empty);
        }
        let mut arena = Arena::new();
        let mut probe = WorkerProbe::new();
        let mut leases = Vec::with_capacity(workers);
        for _ in 0..workers {
            match arena.lease(&mut probe, None, ResourceKind::Worker, 1) {
                Ok(id) => leases.push(id),
                Err(e) => {
                    // A partial pool leaks its leases unless they are released here, and an
                    // abandoned lease is exactly what the ledger identity exists to convict.
                    for id in leases {
                        let _ = arena.release(id);
                    }
                    return Err(PoolError::Refused(e));
                }
            }
        }
        let progress = (0..workers).map(|_| AtomicUsize::new(0)).collect();
        Ok(WorkerPool {
            leases,
            arena,
            progress,
        })
    }

    /// A pool sized to the machine — with the reading treated as a HINT and every worker
    /// still admitted by its own probe.
    pub fn sized_to_machine() -> Result<WorkerPool, PoolError> {
        let hint = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        WorkerPool::new(hint.max(1))
    }

    pub fn workers(&self) -> usize {
        self.leases.len()
    }

    /// Terms each worker has evaluated since the last [`WorkerPool::reset_progress`].
    ///
    /// Reported so a caller can see whether the pool is doing anything. A pool of eight in
    /// which one worker did all the work is a serial run with seven idle leases, and it
    /// would otherwise pass every correctness gate this crate has.
    pub fn progress(&self) -> Vec<usize> {
        self.progress
            .iter()
            .map(|c| c.load(Ordering::Relaxed))
            .collect()
    }

    /// Zero the work counters. Does NOT unpay rent — a receipt already written is a
    /// receipt; this only restarts the reading.
    pub fn reset_progress(&self) {
        for c in self.progress.iter() {
            c.store(0, Ordering::Relaxed);
        }
    }

    /// Whether a job of `terms` terms is worth handing to this pool.
    ///
    /// **STAKED, NOT MEASURED, and the reason is recorded rather than glossed.** The
    /// intended derivation is `examples/scaling.rs`: find the scene size at which the
    /// speedup crosses one and set the constant from it. That example runs and prints, and
    /// its numbers on this box are NOT usable — a sibling lane holds 32 threads for the
    /// ozone tabulation, and the measurement shows it (N = 2744 timed FASTER than
    /// N = 1728, which is impossible; 8 workers read slower than 2 at every size). That is
    /// M-CONTENDED-BASELINE, and the honest response is to leave the constant staked and
    /// say so, not to bank a number the box cannot support.
    ///
    /// What IS established without a quiet box is correctness: `tests/bit_identity.rs`
    /// compares bits, and contention cannot change a bit.
    ///
    /// Stated as a predicate the caller can read rather than as a silent fallback: "we
    /// quietly ran it serially" and "we told you to" are different, and only the second one
    /// lets a benchmark be believed.
    pub fn worth_it(&self, terms: usize) -> bool {
        terms >= 4 * holon_render::sim::FORCE_CHUNK * self.workers()
    }

    /// The lease ledger. `opened == released + convicted + live` is exact over integers,
    /// so a leak is a proof rather than a suspicion.
    pub fn ledger(&self) -> holon_resource::Ledger {
        self.arena.ledger()
    }

    pub fn balances(&self) -> bool {
        self.arena.balances()
    }

    /// Release every lease. Consumes the pool, because a pool whose leases are released is
    /// not a pool any more and holding one would be holding a receipt for nothing.
    pub fn retire(mut self) -> holon_resource::Ledger {
        for id in std::mem::take(&mut self.leases) {
            let _ = self.arena.release(id);
        }
        self.arena.ledger()
    }

    /// Pay the rent for the work done: one receipt per worker, equal to the terms it
    /// evaluated since the counters were last reset.
    fn pay(&mut self) {
        for (k, id) in self.leases.iter().enumerate() {
            let n = self.progress[k].load(Ordering::Relaxed) as u64;
            if n > 0 {
                let _ = self.arena.pay_rent(*id, Receipt(n));
            }
        }
    }

    /// The shared shape of both evaluation passes: hand chunks out to `workers` threads,
    /// join them, and count what each did.
    ///
    /// WORK STEALING, by an atomic counter rather than a static split. A static split would
    /// give every worker the same NUMBER of terms, and terms do not cost the same — a
    /// triple that falls outside every table's domain costs a comparison, one inside costs
    /// an interpolation and its three derivatives, and which is which depends on where the
    /// atoms are. A static split therefore ends every frame waiting for whichever worker
    /// drew the compact region.
    ///
    /// Stealing changes WHO computes a chunk and never WHICH chunk it is, so the term array
    /// is filled identically whatever the interleaving. That is the whole reason the
    /// evaluate/accumulate split was worth making.
    fn run_chunks<T, F>(&self, data: &mut [T], chunk: usize, f: F)
    where
        T: Send,
        F: Fn(usize, &mut [T]) + Sync,
    {
        let chunk = chunk.max(1);
        let n_workers = self.leases.len();
        if data.is_empty() {
            return;
        }
        let mut parts: Vec<&mut [T]> = data.chunks_mut(chunk).collect();
        let n_chunks = parts.len();
        if n_workers == 1 || n_chunks == 1 {
            let total = data_len_of(&parts);
            for (ci, part) in parts.iter_mut().enumerate() {
                f(ci * chunk, part);
            }
            self.progress[0].fetch_add(total, Ordering::Relaxed);
            return;
        }

        let next = AtomicUsize::new(0);
        // Each chunk is handed out at most once, so the mutable slices are disjoint by
        // construction. `SendPtr` is how that fact is carried across the thread boundary:
        // the borrow checker cannot see "at most once" in an atomic counter, and the
        // alternative — a mutex around the array — would serialise exactly the part that
        // is meant to go wide.
        let cells: Vec<SendPtr<T>> = parts
            .iter_mut()
            .map(|p| SendPtr {
                ptr: p.as_mut_ptr(),
                len: p.len(),
            })
            .collect();
        let progress = &self.progress;
        let f = &f;
        let cells = &cells;
        let next = &next;
        std::thread::scope(|scope| {
            for w in 0..n_workers {
                scope.spawn(move || {
                    let mut done = 0usize;
                    loop {
                        let ci = next.fetch_add(1, Ordering::Relaxed);
                        if ci >= n_chunks {
                            break;
                        }
                        let cell = &cells[ci];
                        // SAFETY: `next` hands out each `ci` exactly once, so no two
                        // threads ever hold the same `cell`; the slices came from
                        // `chunks_mut` on one array and are pairwise disjoint; and the
                        // scope joins every thread before `data`'s borrow ends.
                        let part: &mut [T] =
                            unsafe { std::slice::from_raw_parts_mut(cell.ptr, cell.len) };
                        f(ci * chunk, part);
                        done += cell.len;
                    }
                    progress[w].fetch_add(done, Ordering::Relaxed);
                });
            }
        });
    }
}

fn data_len_of<T>(parts: &[&mut [T]]) -> usize {
    parts.iter().map(|p| p.len()).sum()
}

/// A raw pointer to one chunk, carried across a thread boundary.
///
/// `*mut T` is not `Send`, and correctly so in general. It is sound HERE because the atomic
/// counter in `run_chunks` guarantees each chunk is claimed by exactly one thread and
/// `thread::scope` guarantees every thread is joined before the borrow ends. The unsafety
/// is confined to this struct and the one `from_raw_parts_mut` that reads it, which is the
/// smallest surface the design admits.
struct SendPtr<T> {
    ptr: *mut T,
    len: usize,
}

// SAFETY: see `SendPtr`'s own doc and the SAFETY comment at the dereference. The pointer is
// never duplicated, never outlives the scope, and is claimed once.
unsafe impl<T: Send> Send for SendPtr<T> {}
unsafe impl<T: Send> Sync for SendPtr<T> {}

impl ForceExecutor for WorkerPool {
    fn eval_pairs(&self, sim: &Sim, terms: &mut [PairTerm], chunk: usize) {
        self.run_chunks(terms, chunk, |base, part| sim.eval_pair_chunk(base, part));
    }

    fn eval_triples(&self, sim: &Sim, terms: &mut [TripleTerm], chunk: usize) {
        self.run_chunks(terms, chunk, |base, part| sim.eval_triple_chunk(base, part));
    }

    fn workers(&self) -> usize {
        self.leases.len()
    }
}

/// Run `frames` frames of `substeps` each with the force evaluation on `pool`, paying the
/// rent as it goes.
///
/// The pool is installed for the duration and taken back out, so the caller keeps ownership
/// of it and its ledger. A run that panics leaves the executor installed and the leases
/// held — which is the honest failure, because the leases WERE taken and the ledger should
/// say so.
pub fn run_frames(
    sim: &mut Sim,
    pool: WorkerPool,
    frames: usize,
    substeps: u32,
) -> (WorkerPool, Vec<usize>) {
    let mut pool = pool;
    pool.reset_progress();
    let boxed: Box<dyn ForceExecutor + Send + Sync> = Box::new(PoolHandle::new(&pool));
    sim.set_executor(Some(boxed));
    for _ in 0..frames {
        sim.step_frame(substeps);
    }
    sim.set_executor(None);
    let progress = pool.progress();
    pool.pay();
    (pool, progress)
}

/// A borrow of a pool, boxed for the engine's executor slot.
///
/// The engine's seam takes an owned `Box<dyn ForceExecutor>` because it must survive across
/// `&mut Sim` calls, and the pool is the caller's. This carries a raw borrow with the same
/// discipline as `SendPtr`: `run_frames` installs it, uses it, and removes it before the
/// pool it points at can move.
struct PoolHandle {
    pool: *const WorkerPool,
}

impl PoolHandle {
    fn new(pool: &WorkerPool) -> PoolHandle {
        PoolHandle { pool }
    }
    #[inline]
    fn get(&self) -> &WorkerPool {
        // SAFETY: `run_frames` owns the pool for the whole time the handle is installed and
        // removes the handle before returning, so the pointer is live at every call.
        unsafe { &*self.pool }
    }
}

// SAFETY: `WorkerPool` is `Sync` in use — every method the handle calls takes `&self` and
// touches only atomics and immutable state — and the handle never outlives `run_frames`.
unsafe impl Send for PoolHandle {}
unsafe impl Sync for PoolHandle {}

impl ForceExecutor for PoolHandle {
    fn eval_pairs(&self, sim: &Sim, terms: &mut [PairTerm], chunk: usize) {
        self.get().eval_pairs(sim, terms, chunk)
    }
    fn eval_triples(&self, sim: &Sim, terms: &mut [TripleTerm], chunk: usize) {
        self.get().eval_triples(sim, terms, chunk)
    }
    fn workers(&self) -> usize {
        self.get().workers()
    }
}
