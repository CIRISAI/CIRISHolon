//! The sharded generator: regions to workers, warm chains inside regions, one record and
//! one digest lane per node.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

use holon_resource::{Arena, LeaseError, LeaseId, Probe, Receipt, ResourceKind};

use holon_chem::dual::D2;
use holon_chem::elements::Species;
use holon_chem::fci::solve_determinant_from;
use holon_chem::pair::geometry_problem;

use crate::digest::{Certificate, Digest};
use crate::grid::{NodeId, RegionId, TableGrid};
use crate::mutation::Mutation;
use crate::node::{exit_code, void_reason, NodeRecord, NodeStatus};

/// Where each node's Davidson starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WarmPolicy {
    /// Every node cold. The reference arm: slowest, and the one the warm arms are compared
    /// against for bit-identity and for iteration count.
    AllCold,
    /// The canonical chain: a region's first node in traversal order is cold, every later
    /// node starts from its canonical predecessor's converged vector.
    ///
    /// This is the production policy, and the reason it is defined on the REGION rather
    /// than on the worker is the whole of this crate's header.
    CanonicalChain,
}

/// One table generation run, fully specified.
#[derive(Clone, Debug)]
pub struct GenSpec {
    pub species: [Species; 3],
    pub grid: TableGrid,
    pub warm: WarmPolicy,
    /// Davidson iteration cap for every node. Exceeding it VOIDs the node loudly
    /// (M-BUDGET-LAUNDER) rather than publishing wherever the solve had reached.
    pub max_iter: usize,
    pub mutation: Option<Mutation>,
}

impl GenSpec {
    pub fn new(species: [Species; 3], grid: TableGrid) -> Self {
        Self {
            species,
            grid,
            warm: WarmPolicy::CanonicalChain,
            max_iter: 1200,
            mutation: None,
        }
    }

    pub fn with_warm(mut self, warm: WarmPolicy) -> Self {
        self.warm = warm;
        self
    }

    pub fn with_mutation(mut self, m: Option<Mutation>) -> Self {
        self.mutation = m;
        self
    }
}

/// What a run produced.
#[derive(Debug)]
pub struct GenOutcome {
    /// The assembled table, in canonical node order. This is the artifact compared
    /// bit-for-bit across worker counts.
    pub records: Vec<NodeRecord>,
    /// One digest per WORKER — the per-shard partials of `shardedFold`. Which regions
    /// landed in which partial varies with the schedule; their fold does not, and that is
    /// `shardedFold_invariant`.
    pub shard_digests: Vec<Digest>,
    /// The certificate: the assembled table's own digest against the fold of the shards.
    pub certificate: Certificate,
    pub workers: usize,
    /// How many nodes were solved cold and how many warm — the locality sweep's numerator
    /// and denominator.
    pub cold_solves: usize,
    pub warm_solves: usize,
    /// Total Davidson iterations over the run: the quantity a warm start is supposed to
    /// reduce, and the only honest way to report a speedup that is not a wall-clock
    /// measurement taken on a contended machine.
    pub total_davidson_iters: u64,
    pub voided: usize,
}

impl GenOutcome {
    /// The table's own bytes, for a bit-identity comparison that cannot be fooled by a
    /// `PartialEq` that skips a field.
    pub fn table_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.records.len() * 40);
        for r in &self.records {
            out.extend_from_slice(&r.node.to_le_bytes());
            out.extend_from_slice(&r.energy_bits.to_le_bytes());
            out.extend_from_slice(&r.d1_bits.to_le_bytes());
            out.extend_from_slice(&r.d2_bits.to_le_bytes());
            out.extend_from_slice(&r.davidson_iters.to_le_bytes());
            out.extend_from_slice(&r.cg_iters.to_le_bytes());
            out.push(r.exit_code);
            out.extend_from_slice(&r.status_code().to_le_bytes());
        }
        out
    }

    pub fn digest(&self) -> Digest {
        Digest::of_table(&self.records)
    }
}

/// The triangle at a node, in the coordinates `holon-chem::trimer` uses.
fn triangle(x: f64, y: f64, u: f64) -> Vec<[D2; 3]> {
    let s = (1.0 - u * u).max(0.0).sqrt();
    vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
        [D2::c(x), D2::c(0.0), D2::c(0.0)],
        [D2::c(y * u), D2::c(y * s), D2::c(0.0)],
    ]
}

/// A deliberately wrong start vector for plant (iii): deterministic, and with no
/// relationship to any answer.
fn wrong_start(n: usize) -> Vec<f64> {
    let mut v = vec![0.0f64; n];
    let mut seed = 0xdead_beef_1234_5678u64;
    for x in v.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *x = ((seed >> 33) as f64 / (1u64 << 31) as f64) - 0.5;
    }
    v
}

/// Where one node's Davidson is to start.
enum Start<'a> {
    Cold,
    Warm(&'a [f64]),
    /// Plant (iii): a deliberately wrong vector, built inside [`solve_node`] because only
    /// there is the determinant count known. Building it outside would mean assembling the
    /// geometry twice, and a plant that costs a second assembly invites someone to remove
    /// it for being slow.
    WrongPlant,
}

/// Solve one node, returning its record and its converged vector (for the next node's warm
/// start).
fn solve_node(spec: &GenSpec, node: NodeId, start: Start<'_>) -> (NodeRecord, Vec<f64>) {
    let (x, y, u) = spec.grid.geometry(node);
    let (space, mo, nuc) = geometry_problem(&spec.species, triangle(x, y, u));
    // M-VACUOUS-SUCCESS: an empty space would make every guard below pass by vacuity.
    assert!(
        space.n_det > 0,
        "node {node}: the CI space is empty, so no guard below would mean anything"
    );

    let planted;
    let start_slice: Option<&[f64]> = match start {
        Start::Cold => None,
        Start::Warm(v) => Some(v),
        Start::WrongPlant => {
            planted = wrong_start(space.n_det);
            Some(&planted)
        }
    };
    let sol = solve_determinant_from(&space, &mo, start_slice);

    // The variational guard lives on the Solution (`variational_margin`), computed by the
    // solver from the diagonal it already holds. M-VACUOUS-SUCCESS again: if it is absent
    // the guard is not running, and a node that cannot be guarded must not score.
    assert!(
        sol.variational_margin.is_some(),
        "node {node}: the solve returned no variational margin, so the only guard that can \
         catch a wrong-eigenvector convergence is not running"
    );
    let status = match void_reason(&sol) {
        Some(r) => NodeStatus::Void(r),
        None => NodeStatus::Ok,
    };

    // The published energy is the TOTAL: electronic plus nuclear repulsion, the same
    // composition `pair::solve_basis` makes. The guard above compares the ELECTRONIC energy
    // against the electronic diagonal, because mixing the two would compare quantities that
    // differ by a constant.
    let total = sol.e + nuc;

    let record = NodeRecord {
        node,
        energy_bits: total.v.to_bits(),
        d1_bits: total.d.to_bits(),
        d2_bits: total.e.to_bits(),
        davidson_iters: sol.davidson_iters as u32,
        cg_iters: sol.cg_iters as u32,
        exit_code: exit_code(sol.exit),
        status,
        warm: start_slice.is_some(),
    };
    (record, sol.vector)
}

/// Generate the table across `workers` OS threads.
///
/// Regions are handed out from a shared counter, so which worker takes which region is
/// genuinely nondeterministic from run to run. That is deliberate: it makes the
/// shard-invariance claim a statement about an actually-varying schedule rather than about
/// a fixed assignment that happens to be reproducible.
pub fn generate(spec: &GenSpec, workers: usize) -> GenOutcome {
    let counters: Vec<AtomicU64> = (0..workers.max(1)).map(|_| AtomicU64::new(0)).collect();
    generate_with_progress(spec, workers, &counters)
}

/// [`generate`], with a live per-worker progress counter each worker bumps as nodes land.
///
/// Separate rather than an `Option` parameter because the counters are what let a reaper tell a
/// slow holder from an idle one, and a run that could silently omit them is a run whose holders
/// look dead.
pub fn generate_with_progress(
    spec: &GenSpec,
    workers: usize,
    progress: &[AtomicU64],
) -> GenOutcome {
    assert!(workers >= 1, "a run needs at least one worker");
    assert_eq!(
        progress.len(),
        workers,
        "one progress counter per worker, or a worker's work is invisible to the reaper"
    );
    let grid = spec.grid;
    let n_regions = grid.n_regions();
    let n_nodes = grid.n_nodes();

    let mut order: Vec<RegionId> = (0..n_regions as RegionId).collect();
    if spec.mutation == Some(Mutation::ReverseRegionOrder) {
        order.reverse();
    }

    // `DAVIDSON_MAX_ITER` is a process-wide global. It is set ONCE here, before any worker
    // exists, and restored after they have all joined — never from inside a worker. Writing
    // it per node from parallel threads would be a data race that made the solve's budget
    // depend on the schedule, which is precisely the nondeterminism this crate exists to
    // rule out; it would have manufactured the defect the gate is looking for.
    //
    // Two concurrent `generate` calls in one process would still collide on it. They are
    // not supported, and this is the reason.
    let prev_cap = holon_chem::fci::DAVIDSON_MAX_ITER.swap(spec.max_iter, Ordering::Relaxed);

    let next = AtomicUsize::new(0);
    // One slot per node, filled exactly once. A Mutex per run (not per node) guards the
    // assembly; the solves happen outside it.
    let slots: Mutex<Vec<Option<NodeRecord>>> = Mutex::new(vec![None; n_nodes]);
    let worker_digests: Mutex<Vec<Digest>> = Mutex::new(vec![Digest::ZERO; workers]);
    let counters = Mutex::new((0usize, 0usize, 0u64)); // cold, warm, iters

    std::thread::scope(|scope| {
        for w in 0..workers {
            let next = &next;
            let slots = &slots;
            let worker_digests = &worker_digests;
            let counters = &counters;
            let order = &order;
            scope.spawn(move || {
                let mut mine = Digest::ZERO;
                let (mut cold, mut warm, mut iters) = (0usize, 0usize, 0u64);
                // The WorkerLocalWarmStart mutation's state: the last vector THIS worker
                // produced, regardless of which region it came from. That is the defect.
                let mut worker_last: Option<Vec<f64>> = None;

                loop {
                    let idx = next.fetch_add(1, Ordering::Relaxed);
                    if idx >= order.len() {
                        break;
                    }
                    let region = order[idx];
                    let nodes = grid.region_nodes(region);
                    // M-VACUOUS-SUCCESS: a region with no nodes would let a worker report
                    // success having done nothing.
                    assert!(
                        !nodes.is_empty(),
                        "region {region} is empty; the partition is malformed"
                    );

                    let mut chain: Option<Vec<f64>> = None;
                    for (pos, &node) in nodes.iter().enumerate() {
                        let warm_source: Option<&Vec<f64>> = match spec.mutation {
                            Some(Mutation::WorkerLocalWarmStart) => worker_last.as_ref(),
                            _ => match spec.warm {
                                WarmPolicy::AllCold => None,
                                // The region's FIRST node in canonical traversal order is
                                // the cold seed; every later node starts from its canonical
                                // predecessor inside this same region.
                                WarmPolicy::CanonicalChain if pos > 0 => chain.as_ref(),
                                WarmPolicy::CanonicalChain => None,
                            },
                        };
                        let start = match spec.mutation {
                            Some(Mutation::WrongWarmStart { node: target }) if target == node => {
                                Start::WrongPlant
                            }
                            Some(Mutation::WrongWarmStartAll) => Start::WrongPlant,
                            _ => match warm_source {
                                Some(v) => Start::Warm(v),
                                None => Start::Cold,
                            },
                        };

                        let (record, vector) = solve_node(spec, node, start);
                        if record.warm {
                            warm += 1;
                        } else {
                            cold += 1;
                        }
                        iters += record.davidson_iters as u64;
                        mine = mine.merge(Digest::of_record(&record));
                        chain = Some(vector.clone());
                        worker_last = Some(vector);

                        let mut s = slots.lock().unwrap();
                        assert!(
                            s[node as usize].is_none(),
                            "node {node} was solved twice; the partition is not a partition"
                        );
                        s[node as usize] = Some(record);
                        drop(s);
                        // Rent, paid as the work lands rather than at the end: a shard that
                        // reported nothing for hours would look IDLE to the reaper.
                        progress[w].fetch_add(1, Ordering::Relaxed);
                    }
                }
                worker_digests.lock().unwrap()[w] = mine;
                let mut c = counters.lock().unwrap();
                c.0 += cold;
                c.1 += warm;
                c.2 += iters;
            });
        }
    });

    holon_chem::fci::DAVIDSON_MAX_ITER.store(prev_cap, Ordering::Relaxed);

    let slots = slots.into_inner().unwrap();
    let mut records: Vec<NodeRecord> = Vec::with_capacity(n_nodes);
    for (i, s) in slots.into_iter().enumerate() {
        records.push(s.unwrap_or_else(|| panic!("node {i} was never solved")));
    }
    // M-VACUOUS-SUCCESS: the generator asserts its work count against the grid it was
    // given, so a run that solved nothing cannot report a clean certificate.
    assert_eq!(
        records.len(),
        n_nodes,
        "the generator assembled {} records for a grid of {n_nodes} nodes",
        records.len()
    );

    let shard_digests = worker_digests.into_inner().unwrap();
    let (cold_solves, warm_solves, total_davidson_iters) = counters.into_inner().unwrap();

    // Plant (iv) is applied HERE — after the shards have reported their digests and before
    // the assembled table is certified. That is the real threat model: a shard's result is
    // corrupted in transit or in assembly, so the workers' own accounting still says the
    // work was done correctly.
    if let Some(Mutation::CorruptNode { node, bit }) = spec.mutation {
        let r = records
            .iter_mut()
            .find(|r| r.node == node)
            .unwrap_or_else(|| panic!("plant (iv) targets node {node}, which is not in the table"));
        let before = r.energy_bits;
        r.energy_bits ^= 1u64 << bit;
        // M-PLANT-OBS: the carrier is asserted to have actually moved before the plant is
        // scored. A corruption that changed nothing would make the conviction meaningless.
        assert_ne!(
            before, r.energy_bits,
            "plant (iv) flipped bit {bit} of node {node} and the energy did not change"
        );
    }

    let certificate = Certificate::check(Digest::of_table(&records), &shard_digests);
    let voided = records.iter().filter(|r| !r.is_ok()).count();

    GenOutcome {
        records,
        shard_digests,
        certificate,
        workers,
        cold_solves,
        warm_solves,
        total_davidson_iters,
        voided,
    }
}

/// Generate a table **through the resource layer**: a probed worker lease per shard, receipts
/// paid as real work lands, and books that must balance before the table is handed back.
///
/// This is the production entry point; [`generate`] remains the bare one the G1 gate drives.
///
/// # Why the receipts accrue LIVE rather than at the end
///
/// Receipts are the rent (RESOURCE_DESIGN §9 Q1). If a shard paid its rent only on completion,
/// a multi-hour shard would be a holder that has produced no receipt for hours — which is
/// exactly what the reaper's rung 1 is looking at, and it would be reaping the campaign's own
/// tables while they were running. So each worker owns an [`AtomicU64`] it bumps as every node
/// lands, and [`LeasedRun::progress`] exposes those counters so a reaper can read PROGRESS
/// rather than SILENCE.
///
/// The honest limit of this first integration: the arena's rent is settled from those counters
/// after the scope joins, because `Arena` is not `Sync` and a lease-per-node round trip would
/// cost more than the node. The counters are live; the ledger's copy of them is not yet. A
/// reaper wired to the counters would be correct today; one wired to the ledger would not.
///
/// # Errors
///
/// Refuses if any worker's lease is refused, and refuses AFTER the run if the books do not
/// balance — a table whose leases leaked is not published on the grounds that its numbers
/// looked fine.
pub fn generate_leased<P: Probe>(
    spec: &GenSpec,
    workers: usize,
    arena: &mut Arena,
    probe: &mut P,
) -> Result<LeasedRun, LeaseError> {
    assert!(workers >= 1, "a run needs at least one worker");

    // Lease every worker BEFORE any work starts: probe first, then spawn. A refusal here costs
    // nothing and leaves no entry.
    let mut leases = Vec::with_capacity(workers);
    for _ in 0..workers {
        match arena.lease(probe, None, ResourceKind::Worker, 1) {
            Ok(id) => leases.push(id),
            Err(e) => {
                // Release what we took. A partial lease that is abandoned is the leak the
                // ledger identity exists to catch, so we do not create one on the way out.
                for l in leases.drain(..) {
                    let _ = arena.release(l);
                }
                return Err(e);
            }
        }
    }

    let progress: Vec<AtomicU64> = (0..workers).map(|_| AtomicU64::new(0)).collect();
    let outcome = generate_with_progress(spec, workers, &progress);

    // Settle the rent from the live counters, then release leaf-to-root.
    let mut paid = Vec::with_capacity(workers);
    for (lease, counter) in leases.iter().zip(progress.iter()) {
        let n = counter.load(Ordering::Relaxed);
        arena.pay_rent(*lease, Receipt(n))?;
        paid.push(n);
    }
    for l in &leases {
        arena.release(*l)?;
    }

    // The books must balance before the table is published. M-VACUOUS-SUCCESS at the resource
    // layer: a run that leaked a lease has not done what it says it did.
    if !arena.balances() {
        return Err(LeaseError::Refused {
            kind: ResourceKind::Worker,
            amount: workers as u64,
            why: "the run's lease books did not balance; a table whose leases leaked is not \
                  published on the grounds that its numbers looked fine",
        });
    }
    // And the rent must account for every node, or the counters and the table disagree.
    let total_paid: u64 = paid.iter().sum();
    assert_eq!(
        total_paid as usize,
        outcome.records.len(),
        "receipts total {total_paid} against {} solved nodes; the rent and the table disagree, \
         so one of them is not counting the work",
        outcome.records.len()
    );

    Ok(LeasedRun {
        outcome,
        leases,
        nodes_per_worker: paid,
    })
}

/// A run and the leases it was made under.
#[derive(Debug)]
pub struct LeasedRun {
    pub outcome: GenOutcome,
    pub leases: Vec<LeaseId>,
    /// Nodes solved per worker — the receipts, and the load-balance reading for free.
    pub nodes_per_worker: Vec<u64>,
}

impl LeasedRun {
    /// The live progress counters' final values. A reaper reads these to tell SLOW from IDLE.
    pub fn progress(&self) -> &[u64] {
        &self.nodes_per_worker
    }
}
