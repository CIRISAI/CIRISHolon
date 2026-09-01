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
use crate::grid::{NdGrid, NodeId, RegionId, TableGrid};
use crate::mutation::Mutation;
use crate::node::{exit_code, void_reason, NodeRecord, NodeStatus, VoidReason, NOT_SOLVED_EXIT};
use crate::surface::{Realised, Surface, TrimerSurface};

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
    ///
    /// When the surface declares a symmetry there is ONE MORE lane on the end: the
    /// mirror-fill pass (see [`generate_surface_with_progress`]), which is a genuine shard
    /// of the run's work that no worker did. It reports its own lane rather than being
    /// folded into somebody else's, because a lane is what `shardedFold_invariant` says can
    /// be added without moving the fold; and it is absent rather than zero when there are no
    /// mirrors, so a table with no symmetry has exactly the lanes it always had.
    pub shard_digests: Vec<Digest>,
    /// The certificate: the assembled table's own digest against the fold of the shards.
    pub certificate: Certificate,
    pub workers: usize,
    /// How many nodes were solved cold and how many warm — the locality sweep's numerator
    /// and denominator.
    pub cold_solves: usize,
    pub warm_solves: usize,
    /// How many nodes were FILLED from their symmetry orbit's representative rather than
    /// solved. Zero unless the surface declares a [`Surface::canonical`] other than the
    /// identity.
    ///
    /// `cold_solves + warm_solves + mirrored == records.len()`, always. This is the
    /// measurement the symmetry reduction is worth: `records.len() / (records.len() -
    /// mirrored)` is the solve-count factor saved.
    pub mirrored: usize,
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
    /// Plant (iii): a deliberately wrong vector, built inside [`solve_surface_node`] because only
    /// there is the determinant count known. Building it outside would mean assembling the
    /// geometry twice, and a plant that costs a second assembly invites someone to remove
    /// it for being slow.
    WrongPlant,
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
    let surface = TrimerSurface::new(spec.species);
    generate_surface_with_progress(&trimer_spec(spec, &surface), workers, progress)
}

/// The 3-axis trimer run, said in the folded generator's own terms.
///
/// This is the whole of what used to be a second generator. `NdGrid::from_table_grid` is
/// bit-identical to the `TableGrid` it folds on every one of the canonical functions —
/// asserted node by node in `tests/nd_bit_identity.rs` — and `TrimerSurface::realise` is the
/// old `triangle()` arithmetic character for character, so the numbers cannot move.
fn trimer_spec<'a>(spec: &GenSpec, surface: &'a TrimerSurface) -> SurfaceSpec<'a, TrimerSurface> {
    SurfaceSpec {
        surface,
        grid: NdGrid::from_table_grid(&spec.grid),
        warm: spec.warm,
        max_iter: spec.max_iter,
        mutation: spec.mutation,
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
    let surface = TrimerSurface::new(spec.species);
    generate_surface_leased(&trimer_spec(spec, &surface), workers, arena, probe)
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

// ===========================================================================
// The folded generator: one leased pipeline, any number of axes, any surface
// ===========================================================================

/// One table generation run over an arbitrary [`Surface`], fully specified.
///
/// The dimension-generic sibling of [`GenSpec`]. `GenSpec` is not deleted — it is the
/// 3-body campaign's front door and its callers are committed artifacts — but it no longer
/// carries a generator of its own: [`generate_with_progress`] builds a [`TrimerSurface`]
/// and an [`NdGrid`] and comes straight back here. There is exactly one solve loop.
pub struct SurfaceSpec<'s, S: Surface + ?Sized> {
    pub surface: &'s S,
    pub grid: NdGrid,
    pub warm: WarmPolicy,
    /// Davidson iteration cap for every node. Exceeding it VOIDs the node loudly
    /// (M-BUDGET-LAUNDER) rather than publishing wherever the solve had reached.
    pub max_iter: usize,
    pub mutation: Option<Mutation>,
}

impl<'s, S: Surface + ?Sized> SurfaceSpec<'s, S> {
    pub fn new(surface: &'s S, grid: NdGrid) -> SurfaceSpec<'s, S> {
        assert_eq!(
            surface.dim(),
            grid.axes.len(),
            "the surface reads {} coordinates and the grid has {} axes; a mismatch here \
             would silently feed a node's coordinates to the wrong slot of the geometry",
            surface.dim(),
            grid.axes.len()
        );
        SurfaceSpec {
            surface,
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

/// Solve one node of an arbitrary surface.
///
/// Returns the record and, when there was a solve to carry, its converged vector for the
/// next node's warm start. A [`Realised::Refused`] node has no vector: chaining from a
/// solve that did not happen is how a table comes to depend on which nodes were skipped.
fn solve_surface_node<S: Surface + ?Sized>(
    spec: &SurfaceSpec<'_, S>,
    node: NodeId,
    start: Start<'_>,
) -> (NodeRecord, Option<Vec<f64>>) {
    let coords = spec.grid.geometry(node);
    // THE SEAM. Everything the composition knows is on the other side of this call, and
    // everything the mesh knows is on this one.
    let (centers, from, is_geometry) = match spec.surface.realise(&coords) {
        Realised::Geometry(c) => (c, None, true),
        // A continued node is FILLED (a stencil must not read a hole) and NEVER SCORED (it
        // is not the point it claims to be). The exclusion is carried in the record's own
        // status rather than by a filter downstream, so no accuracy statistic can include it
        // by forgetting.
        Realised::Continued { centers, from } => (centers, Some(from), false),
        Realised::Refused => {
            return (
                NodeRecord {
                    node,
                    energy_bits: f64::NAN.to_bits(),
                    d1_bits: f64::NAN.to_bits(),
                    d2_bits: f64::NAN.to_bits(),
                    davidson_iters: 0,
                    cg_iters: 0,
                    exit_code: NOT_SOLVED_EXIT,
                    status: NodeStatus::Void(VoidReason::Unrealisable),
                    warm: false,
                    mirrored: false,
                },
                None,
            );
        }
    };

    let d2_centers: Vec<[D2; 3]> = centers
        .iter()
        .map(|c| [D2::c(c[0]), D2::c(c[1]), D2::c(c[2])])
        .collect();
    let (space, mo, nuc) = geometry_problem(spec.surface.species(), d2_centers);
    // M-VACUOUS-SUCCESS: an empty space would make every guard below pass by vacuity.
    // Asserted for GEOMETRIES only — a continued node is already VOID by construction, so
    // there is no guard of its below to be made vacuous, and refusing the whole run because
    // a point outside the embeddable set produced an odd space would be refusing for the
    // wrong reason.
    if is_geometry {
        assert!(
            space.n_det > 0,
            "node {node}: the CI space is empty, so no guard below would mean anything"
        );
    }

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
    if is_geometry {
        assert!(
            sol.variational_margin.is_some(),
            "node {node}: the solve returned no variational margin, so the only guard that \
             can catch a wrong-eigenvector convergence is not running"
        );
    }
    let status = if is_geometry {
        match void_reason(&sol) {
            Some(r) => NodeStatus::Void(r),
            None => NodeStatus::Ok,
        }
    } else {
        NodeStatus::Void(VoidReason::NotAGeometry)
    };

    // The converged energy is the TOTAL: electronic plus nuclear repulsion, the same
    // composition `pair::solve_basis` makes. The guard above compares the ELECTRONIC energy
    // against the electronic diagonal, because mixing the two would compare quantities that
    // differ by a constant. What is STORED is whatever the surface says to store — the total
    // for a 3-body table, `E_total - E_MBE3` for a 4-body one.
    let total = sol.e + nuc;
    // A continued node is subtracted at the coordinates it was REALLY realised at. Feeding
    // the requested coordinates to a many-body reference the solve never saw would mix two
    // different geometries into one stored number.
    let stored = spec
        .surface
        .subtract(from.as_deref().unwrap_or(&coords), total.v);

    let record = NodeRecord {
        node,
        energy_bits: stored.to_bits(),
        d1_bits: total.d.to_bits(),
        d2_bits: total.e.to_bits(),
        davidson_iters: sol.davidson_iters as u32,
        cg_iters: sol.cg_iters as u32,
        exit_code: exit_code(sol.exit),
        status,
        warm: start_slice.is_some(),
        // This function only ever SOLVES. Mirrors are made in the generator's second pass,
        // out of a solved record, and never here — so there is one place in the crate that
        // can produce a record without a solve behind it.
        mirrored: false,
    };
    (record, Some(sol.vector))
}

/// The grid's orbit map: `rep[n]` is the node whose solve node `n` takes its value from.
///
/// `rep[n] == n` exactly for the representatives. With no symmetry declared that is every
/// node, and the whole reduction collapses to the identity — which is what makes the folded
/// generator's existing behaviour bit-identical rather than merely equivalent.
///
/// # What is checked here, and why here
///
/// Before any worker exists, because a malformed orbit map is a table-wide defect and the
/// place to refuse it is the one place that can name the node: the map must return one
/// in-range index per axis, and it must be IDEMPOTENT. A representative that was itself
/// mirrored would leave a node filled from a slot nobody solved; the generator's assembly
/// would report "node was never solved" from a place that cannot explain why.
fn orbit_map<S: Surface + ?Sized>(spec: &SurfaceSpec<'_, S>) -> Vec<NodeId> {
    let grid = &spec.grid;
    let n_nodes = grid.n_nodes();
    let mut rep: Vec<NodeId> = Vec::with_capacity(n_nodes);
    for n in 0..n_nodes {
        let idx = grid.coords(n as NodeId);
        let c = spec.surface.canonical(&idx);
        assert_eq!(
            c.len(),
            idx.len(),
            "node {n}: the surface's canonical map returned {} indices for a {}-axis grid",
            c.len(),
            idx.len()
        );
        for (d, (&ci, a)) in c.iter().zip(grid.axes.iter()).enumerate() {
            assert!(
                ci < a.n,
                "node {n}: the surface's canonical map put index {ci} on axis {d}, which has \
                 only {} nodes",
                a.n
            );
        }
        rep.push(grid.node_id(&c));
    }
    for n in 0..n_nodes {
        let r = rep[n] as usize;
        assert_eq!(
            rep[r],
            r as NodeId,
            "node {n}'s representative is node {r}, which is itself mirrored onto node {}. A \
             canonical map must be IDEMPOTENT — canonical(canonical(x)) == canonical(x) — or \
             a node would be filled from a slot no worker ever solved.",
            rep[r]
        );
    }
    rep
}

/// [`generate_surface_with_progress`] with counters nobody reads.
pub fn generate_surface<S: Surface + ?Sized>(
    spec: &SurfaceSpec<'_, S>,
    workers: usize,
) -> GenOutcome {
    let counters: Vec<AtomicU64> = (0..workers.max(1)).map(|_| AtomicU64::new(0)).collect();
    generate_surface_with_progress(spec, workers, &counters)
}

/// **The generator.** Regions to workers, warm chains inside regions, one record and one
/// digest lane per node — for any surface over any number of axes.
///
/// Every discipline the 3-axis path had is here and is here ONCE: the canonical partition
/// from the grid alone, the per-worker progress counters, the single global
/// `DAVIDSON_MAX_ITER` swap outside the scope, the one mutex over the slot vector with the
/// solves outside it, the solved-twice assert, and the M-VACUOUS-SUCCESS asserts (which now
/// apply to nodes that are actually geometries — see [`solve_surface_node`]).
///
/// # The symmetry reduction, and why it is TWO PASSES rather than one
///
/// A surface may declare an orbit map ([`Surface::canonical`]). A node that is not its own
/// representative is not solved; it is filled from the representative's record. The obvious
/// implementation — fill as you go — is WRONG, and the reason is the shape of the work
/// partition rather than anything about the symmetry: a region's nodes go to one worker, but
/// a node's representative is a relabelling of its coordinates and so generally lands in a
/// DIFFERENT region owned by a DIFFERENT worker. Filling in-line would mean either reading a
/// slot that is not written yet, or waiting for another worker — and a result that depends
/// on who got there first is precisely what this crate exists to rule out.
///
/// So:
///
/// * **pass 1** solves the representatives, partitioned into regions exactly as before, warm
///   chains intact — a region's chain is now over the representatives it contains, in the
///   same canonical traversal order, which is still a pure function of the grid and the
///   declared symmetry and still knows nothing about workers;
/// * **pass 2** fills every mirror from the completed slot table. It is pure memory, it runs
///   on this thread after the join, in ascending node order, and it therefore cannot see the
///   schedule at all.
///
/// With no symmetry declared every node is its own representative, pass 2 does nothing, and
/// the run is the one it always was — bit for bit, which is the acceptance bar.
pub fn generate_surface_with_progress<S: Surface + ?Sized>(
    spec: &SurfaceSpec<'_, S>,
    workers: usize,
    progress: &[AtomicU64],
) -> GenOutcome {
    assert!(workers >= 1, "a run needs at least one worker");
    assert_eq!(
        progress.len(),
        workers,
        "one progress counter per worker, or a worker's work is invisible to the reaper"
    );
    assert_eq!(
        spec.surface.dim(),
        spec.grid.axes.len(),
        "the surface reads {} coordinates and the grid has {} axes",
        spec.surface.dim(),
        spec.grid.axes.len()
    );
    let grid = &spec.grid;
    let n_regions = grid.n_regions();
    let n_nodes = grid.n_nodes();

    let mut order: Vec<RegionId> = (0..n_regions as RegionId).collect();
    if spec.mutation == Some(Mutation::ReverseRegionOrder) {
        order.reverse();
    }

    // THE ORBIT MAP, resolved ONCE, before any worker exists. `rep[n]` is the node whose
    // solve node `n` takes its value from, and `rep[n] == n` exactly for the representatives
    // pass 1 solves. Like the partition, it is a pure function of the grid and the surface —
    // never of the worker count and never of the schedule.
    let rep = orbit_map(spec);
    let n_mirrors = (0..n_nodes).filter(|&n| rep[n] != n as NodeId).count();

    // `DAVIDSON_MAX_ITER` is a process-wide global. It is set ONCE here, before any worker
    // exists, and restored after they have all joined — never from inside a worker. Writing
    // it per node from parallel threads would be a data race that made the solve's budget
    // depend on the schedule, which is precisely the nondeterminism this crate exists to
    // rule out; it would have manufactured the defect the gate is looking for.
    //
    // Two concurrent generator calls in one process would still collide on it. They are
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
            let rep = &rep;
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
                    for &node in nodes.iter() {
                        // PASS 1 SOLVES REPRESENTATIVES ONLY. A node that is not its own
                        // representative is filled in pass 2 from a record that may be
                        // solved by a different worker, in a different region, later — so it
                        // is not touched here, and it is not on this region's warm chain
                        // either. With no symmetry declared this never fires.
                        if rep[node as usize] != node {
                            continue;
                        }
                        let warm_source: Option<&Vec<f64>> = match spec.mutation {
                            Some(Mutation::WorkerLocalWarmStart) => worker_last.as_ref(),
                            _ => match spec.warm {
                                WarmPolicy::AllCold => None,
                                // The region's FIRST SOLVED node in canonical traversal
                                // order is the cold seed; every later one starts from its
                                // canonical predecessor inside this same region. `chain` is
                                // `None` until the region's first solve lands, which is what
                                // makes that seed cold — and it is the identical rule the
                                // pre-symmetry code wrote as `pos > 0`, since `chain` was
                                // `None` at `pos == 0` too.
                                WarmPolicy::CanonicalChain => chain.as_ref(),
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

                        let (record, vector) = solve_surface_node(spec, node, start);
                        if record.warm {
                            warm += 1;
                        } else {
                            cold += 1;
                        }
                        iters += record.davidson_iters as u64;
                        mine = mine.merge(Digest::of_record(&record));
                        if let Some(v) = vector {
                            chain = Some(v.clone());
                            worker_last = Some(v);
                        }

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

    let mut slots = slots.into_inner().unwrap();

    // ------------------------------------------------------------------ PASS 2: the mirrors
    //
    // Pure memory work, on this thread, after every worker has joined, in ascending node
    // order. Nothing here can see which worker solved which region or when: the input is the
    // completed slot table and the orbit map, both of which are fixed before the workers
    // start. That is what makes the filled half of the table as schedule-free as the solved
    // half, without a single cross-worker wait.
    let mut mirror_digest = Digest::ZERO;
    let mut mirrored = 0usize;
    for n in 0..n_nodes {
        let r = rep[n] as usize;
        if r == n {
            continue;
        }
        let src = slots[r].unwrap_or_else(|| {
            panic!(
                "node {n} is mirrored from node {r}, which no worker solved; the orbit map \
                 and the partition disagree"
            )
        });
        assert!(
            slots[n].is_none(),
            "node {n} was both solved and mirrored; pass 1 solved a node that is not its own \
             representative"
        );
        // A BIT-FOR-BIT COPY, readdressed. Not a recomputation: recomputing would put a
        // second, differently-rounded number at a point the table says is the same point,
        // and the whole warrant for skipping the solve is that it IS the same point.
        let record = NodeRecord {
            node: n as NodeId,
            mirrored: true,
            ..src
        };
        mirror_digest = mirror_digest.merge(Digest::of_record(&record));
        slots[n] = Some(record);
        mirrored += 1;
    }
    assert_eq!(
        mirrored, n_mirrors,
        "the fill pass wrote {mirrored} mirrors where the orbit map named {n_mirrors}"
    );

    let mut records: Vec<NodeRecord> = Vec::with_capacity(n_nodes);
    for (i, s) in slots.into_iter().enumerate() {
        records.push(s.unwrap_or_else(|| {
            panic!("node {i} was neither solved nor filled from a representative")
        }));
    }
    // M-VACUOUS-SUCCESS: the generator asserts its work count against the grid it was
    // given, so a run that solved nothing cannot report a clean certificate.
    assert_eq!(
        records.len(),
        n_nodes,
        "the generator assembled {} records for a grid of {n_nodes} nodes",
        records.len()
    );

    let mut shard_digests = worker_digests.into_inner().unwrap();
    // The fill pass is a SHARD of the run's work that no worker did, so it reports its own
    // lane rather than being folded into somebody else's — a lane is exactly what
    // `shardedFold_invariant` says can be added without moving the fold. It is ABSENT rather
    // than zero when nothing was mirrored, so a run with no symmetry has the lanes it always
    // had and an all-zero lane never stands in for a pass that did not run.
    if mirrored > 0 {
        shard_digests.push(mirror_digest);
    }
    let (cold_solves, warm_solves, total_davidson_iters) = counters.into_inner().unwrap();
    // M-VACUOUS-SUCCESS: every node is accounted for as a solve or as a fill, or the
    // reduction has quietly dropped work while producing a full-looking table.
    assert_eq!(
        cold_solves + warm_solves + mirrored,
        n_nodes,
        "{cold_solves} cold + {warm_solves} warm + {mirrored} mirrored does not account for \
         the grid's {n_nodes} nodes"
    );

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
        mirrored,
        total_davidson_iters,
        voided,
    }
}

/// Generate an arbitrary surface's table **through the resource layer** — the production
/// entry point for every composition.
///
/// Identical in discipline to [`generate_leased`], which now delegates to it: every worker
/// is leased and probed BEFORE any work starts, a refusal releases what was taken rather
/// than abandoning it, receipts accrue LIVE from the per-worker counters so a reaper sees
/// progress rather than silence, rent is settled and the leases released leaf-to-root after
/// the join, and the books must balance and account for every node before the table is
/// handed back.
///
/// # Errors
///
/// Refuses if any worker's lease is refused, and refuses AFTER the run if the books do not
/// balance — a table whose leases leaked is not published on the grounds that its numbers
/// looked fine.
pub fn generate_surface_leased<S: Surface + ?Sized, P: Probe>(
    spec: &SurfaceSpec<'_, S>,
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
    let outcome = generate_surface_with_progress(spec, workers, &progress);

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
    // And the rent must account for every node the WORKERS did, or the counters and the table
    // disagree. Mirrors are deliberately not rent: they are the fill pass's memory work, done
    // after every lease has stopped being held for a solve, and charging a worker for a node
    // it never solved would make the receipts stop meaning "work landed here" — which is the
    // one thing a reaper reads them for.
    let total_paid: u64 = paid.iter().sum();
    let solved = outcome.records.len() - outcome.mirrored;
    assert_eq!(
        total_paid as usize,
        solved,
        "receipts total {total_paid} against {solved} solved nodes ({} of {} filled from a \
         symmetry orbit's representative); the rent and the table disagree, so one of them is \
         not counting the work",
        outcome.mirrored,
        outcome.records.len()
    );

    Ok(LeasedRun {
        outcome,
        leases,
        nodes_per_worker: paid,
    })
}
