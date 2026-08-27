//! The mesh: ONE scene, sharded across arenas, with a halo whose depth is a theorem.
//!
//! # The scheme
//!
//! A shard owns a rectangular block of cells and holds a **halo** — scratch copies of the
//! cells within `n·r` of what it owns. Between exchanges it runs `n` colour sweeps. During
//! those sweeps it updates its halo copies too, and they decay from the outside in: after
//! `k` sweeps everything at halo depth `≤ n − k` is still exact, so after all `n` sweeps the
//! owned cells (depth 0) are exact. Then the halo is refreshed and the cycle repeats.
//!
//! **A shard writes authoritatively only what it owns.** Halo copies are scratch, recomputed
//! from their owners at every exchange, and never read back by anyone. That is what makes
//! conservation exact without a transfer protocol on top: for a boundary adjacency, the shard
//! owning `lo` applies `−d` to its owned cell, the shard owning `hi` applies `+d` to its
//! owned cell, and both compute the same `d` from bit-identical integer inputs. The scene
//! total moves by zero, in all four lanes, by arithmetic.
//!
//! # Why the halo depth is `n·r` and not a tuning parameter
//!
//! `CIRISOntology/Core/Locality.lean`:
//!
//! * `depends_within_comp` — locality composes and the radii ADD;
//! * `iterate_depends_within` — `n` steps of a radius-`r` update depend within `n·r`.
//!
//! So `n·r` **suffices**, and the code asserts the halo is built to exactly that. Whether it
//! is also *necessary* is not something the Lean says, and this crate does not assume it:
//! [`Mutation::HaloOneShallowerThanHorizon`] builds `n·r − 1` and the gate measures whether
//! the answer moves. The bound's tightness is a measurement here, not a citation.
//!
//! # Single tier, asserted
//!
//! Every shard declares the same `g0` — this is a refinement relation, never a re-root.
//! `Core/GrainFloor.lean::cert_does_not_transport_across_reroot` is why that is a fence:
//! nothing in the engine certifies a join across a re-root, so a multi-tier mesh would be
//! uncertified by construction (`SANDBOX_4090` G4). [`MeshSpec`] carries one `g0` for the
//! whole scene and there is no way to express a second.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::{Barrier, Mutex};

use ciris_sim_core::holon::{Channels, Decomposition, HolonError};
use ciris_sim_core::regplus::GrossState;
use ciris_sim_core::runtime::{RuntimeArena, RuntimeHolonSpec, NO_RUNTIME_HOLON};
use holon_swarm::ledger::{apply_delta, gross_to_lanes, lanes_to_gross, LANES};

use crate::error::MeshError;
use crate::grid::{edges_of_colour, neighbours, Grid, Partition, EDGE_COLOURS, RADIUS};
use crate::mutation::{Mutation, VisitOrder};
use crate::state::{advance_energy, plan, seed_energy, seed_gross};

/// How a mesh is built and run. All values; nothing here is a type per shard.
#[derive(Clone, Debug)]
pub struct MeshSpec {
    pub grid: Grid,
    /// Block cuts on each axis. Shard count is `nx * ny * nz`.
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    /// Colour sweeps between halo refreshes — the `n` of `n·r`.
    pub colours_per_exchange: usize,
    /// The scene's ONE declared grain, in metres. Single tier: there is no second field.
    pub g0_m: f64,
    pub order: VisitOrder,
    pub mutation: Mutation,
}

impl MeshSpec {
    /// A flat cut: no division on `z`. For a `d = 1` scene this is the whole story.
    pub fn new(grid: Grid, nx: usize, ny: usize) -> Self {
        Self::new_3d(grid, nx, ny, 1)
    }

    pub fn new_3d(grid: Grid, nx: usize, ny: usize, nz: usize) -> Self {
        Self {
            grid,
            nx,
            ny,
            nz,
            colours_per_exchange: 1,
            g0_m: 5.0e-4,
            order: VisitOrder::Natural,
            mutation: Mutation::None,
        }
    }

    pub fn with_colours_per_exchange(mut self, n: usize) -> Self {
        self.colours_per_exchange = n.max(1);
        self
    }

    pub fn with_order(mut self, order: VisitOrder) -> Self {
        self.order = order;
        self
    }

    pub fn with_mutation(mut self, mutation: Mutation) -> Self {
        self.mutation = mutation;
        self
    }

    /// The horizon this spec's shards must carry: `n · r`, and nothing about it is tunable.
    pub fn horizon(&self) -> usize {
        self.colours_per_exchange * RADIUS
    }
}

/// One adjacency as a shard sees it: working-set indices, plus whether each side is owned.
#[derive(Clone, Copy, Debug)]
struct WorkEdge {
    lo: u32,
    hi: u32,
    /// True when the two endpoints have different owners — a boundary relation.
    cross: bool,
}

/// One shard: an arena over the cells it owns, plus a halo of scratch copies.
pub struct MeshShard {
    index: usize,
    arena: RuntimeArena,
    /// Working set: owned cells first (ascending global index), then halo (ascending).
    cells: Vec<u32>,
    owned_count: usize,
    /// BFS depth from the owned set. `0` for owned, `1..=horizon` for halo.
    depth: Vec<u8>,
    /// Live ledger, index-aligned with `cells`. The arena's STORED gross states never
    /// change — `SANDBOX_4090` §6 weakness (2), carried rather than rediscovered, which is
    /// why the composition check below reads this and not `arena.validate()`.
    gross: Vec<GrossState>,
    energy: Vec<f64>,
    /// Evaluable edges per colour, precomputed.
    colour_edges: Vec<Vec<WorkEdge>>,
    /// Deepest halo cell actually read during a sweep — instrumentation for the locality gate.
    max_read_depth: usize,
}

impl MeshShard {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn arena(&self) -> &RuntimeArena {
        &self.arena
    }

    pub fn owned_cells(&self) -> &[u32] {
        &self.cells[..self.owned_count]
    }

    pub fn halo_cells(&self) -> &[u32] {
        &self.cells[self.owned_count..]
    }

    pub fn max_read_depth(&self) -> usize {
        self.max_read_depth
    }

    /// Edges this shard evaluates across a full colour cycle — the cost of one round's work,
    /// counted rather than estimated from cell count.
    ///
    /// This is the quantity a scheduler must balance. `GrainFloor.lean`'s
    /// `demand_not_function_of_geometry` warns that it is NOT a function of geometry in
    /// general; whether it is one for THIS scene is a measurement, not an assumption, and
    /// `examples/imbalance.rs` is where that gets asked.
    pub fn work_per_round(&self) -> usize {
        self.colour_edges.iter().map(|e| e.len()).sum()
    }

    /// Exact sum of the cells this shard owns. Halo copies are excluded by construction —
    /// counting them would double-count every boundary cell in the scene total.
    pub fn owned_total(&self) -> Option<GrossState> {
        let mut total = GrossState::ZERO;
        for g in &self.gross[..self.owned_count] {
            total = total.checked_combine(*g)?;
        }
        Some(total)
    }
}

/// The mesh. Holds every shard and the board they publish through.
pub struct Mesh {
    spec: MeshSpec,
    partition: Partition,
    shards: Vec<MeshShard>,
    /// Global-cell-indexed published values: the round snapshot, exactly `holon-swarm`'s
    /// board with a spatial index instead of a pair index.
    board_gross: Vec<GrossState>,
    board_energy: Vec<f64>,
    opening_total: GrossState,
    steps_run: usize,
}

impl Mesh {
    pub fn new(spec: MeshSpec) -> Result<Self, MeshError> {
        if spec.grid.is_empty() {
            return Err(MeshError::Config("a mesh needs a non-empty grid"));
        }
        if !(spec.g0_m.is_finite() && spec.g0_m > 0.0) {
            return Err(MeshError::Config("g0 must be a positive length"));
        }
        let grid = spec.grid;
        let partition = Partition::blocks(grid, spec.nx, spec.ny, spec.nz);

        // The horizon, and the ONE place a mutation is allowed to shrink it.
        let horizon = match spec.mutation {
            Mutation::HaloOneShallowerThanHorizon => spec.horizon().saturating_sub(1),
            _ => spec.horizon(),
        };

        let mut shards = Vec::with_capacity(partition.shard_count());
        for index in 0..partition.shard_count() {
            shards.push(build_shard(grid, &partition, index, horizon)?);
        }

        let mut opening_total = GrossState::ZERO;
        for shard in &shards {
            opening_total = opening_total
                .checked_combine(
                    shard
                        .owned_total()
                        .ok_or(MeshError::Core(HolonError::GrossStateDoesNotCompose))?,
                )
                .ok_or(MeshError::Core(HolonError::GrossStateDoesNotCompose))?;
        }

        Ok(Self {
            spec,
            partition,
            shards,
            board_gross: (0..grid.len() as u32).map(seed_gross).collect(),
            board_energy: (0..grid.len() as u32).map(seed_energy).collect(),
            opening_total,
            steps_run: 0,
        })
    }

    pub fn spec(&self) -> &MeshSpec {
        &self.spec
    }

    pub fn shards(&self) -> &[MeshShard] {
        &self.shards
    }

    pub fn partition(&self) -> &Partition {
        &self.partition
    }

    pub fn opening_total(&self) -> GrossState {
        self.opening_total
    }

    /// The scene's total, summed over shard-owned cells only.
    pub fn total(&self) -> Result<GrossState, MeshError> {
        let mut total = GrossState::ZERO;
        for shard in &self.shards {
            total = total
                .checked_combine(
                    shard
                        .owned_total()
                        .ok_or(MeshError::Core(HolonError::GrossStateDoesNotCompose))?,
                )
                .ok_or(MeshError::Core(HolonError::GrossStateDoesNotCompose))?;
        }
        Ok(total)
    }

    /// Every cell's ledger, in GLOBAL index order — the fingerprint the gate compares
    /// against the unsharded reference. Assembling it from the shards is what makes the
    /// comparison meaningful: it is the mesh's own answer, re-indexed, not a copy of the
    /// board.
    pub fn cells(&self) -> Vec<GrossState> {
        let mut out = vec![GrossState::ZERO; self.spec.grid.len()];
        for shard in &self.shards {
            for (slot, &cell) in shard.owned_cells().iter().enumerate() {
                out[cell as usize] = shard.gross[slot];
            }
        }
        out
    }

    pub fn energies(&self) -> Vec<f64> {
        let mut out = vec![0.0; self.spec.grid.len()];
        for shard in &self.shards {
            for (slot, &cell) in shard.owned_cells().iter().enumerate() {
                out[cell as usize] = shard.energy[slot];
            }
        }
        out
    }

    /// Deepest halo cell any shard read. The locality gate reports this and asserts it
    /// against the declared horizon.
    pub fn max_read_depth(&self) -> usize {
        self.shards.iter().map(|s| s.max_read_depth).max().unwrap_or(0)
    }

    // ------------------------------------------------------------------ sequential path

    /// Run `colour_steps` sweeps, single-threaded.
    pub fn run_sequential(&mut self, colour_steps: usize) -> Result<(), MeshError> {
        let n = self.spec.colours_per_exchange;
        let mutation = self.spec.mutation;
        for step in 0..colour_steps {
            if step % n == 0 && mutation != Mutation::HaloReadsLivePeers {
                let skip = mutation == Mutation::HaloRefreshSkipped && step / n == 1;
                if !skip {
                    self.publish();
                    self.refresh_halos();
                }
            }
            self.sweep_all(step)?;
            self.check_scene(step)?;
        }
        self.steps_run += colour_steps;
        Ok(())
    }

    fn publish(&mut self) {
        for shard in &self.shards {
            for (slot, &cell) in shard.owned_cells().iter().enumerate() {
                self.board_gross[cell as usize] = shard.gross[slot];
                self.board_energy[cell as usize] = shard.energy[slot];
            }
        }
    }

    fn refresh_halos(&mut self) {
        for shard in &mut self.shards {
            for slot in shard.owned_count..shard.cells.len() {
                let cell = shard.cells[slot] as usize;
                shard.gross[slot] = self.board_gross[cell];
                shard.energy[slot] = self.board_energy[cell];
            }
        }
    }

    /// Refresh one shard's halo from the peers' LIVE state. Only the live-read mutation calls
    /// this; the correct path goes through [`Self::publish`] then [`Self::refresh_halos`].
    fn refresh_halo_of_from_live(&mut self, shard: usize) {
        let cells: Vec<u32> = self.shards[shard].halo_cells().to_vec();
        let mut values = Vec::with_capacity(cells.len());
        for &cell in &cells {
            let owner = self.partition.owner(cell) as usize;
            let slot = (self.partition.local_id(cell) - 1) as usize;
            values.push((self.shards[owner].gross[slot], self.shards[owner].energy[slot]));
        }
        let target = &mut self.shards[shard];
        let base = target.owned_count;
        for (i, (g, e)) in values.into_iter().enumerate() {
            target.gross[base + i] = g;
            target.energy[base + i] = e;
        }
    }

    fn sweep_all(&mut self, step: usize) -> Result<(), MeshError> {
        let colour = step % EDGE_COLOURS;
        let mutation = self.spec.mutation;
        let visit = self.spec.order.sequence(self.shards.len());
        for &s in &visit {
            if mutation == Mutation::HaloReadsLivePeers {
                // The defect: read the peers' state as it is RIGHT NOW, after whichever
                // shards came earlier in the visit order have already swept. The correct
                // path publishes a snapshot before any shard moves, which is what makes the
                // plan a value rather than a race.
                self.refresh_halo_of_from_live(s);
            }
            sweep_shard(&mut self.shards[s], colour, self.spec.order, mutation)?;
        }
        if mutation == Mutation::CrossShardFloatReduction {
            // M1b: the banned cross-lane float reduction, planted. Every owned cell adds a
            // SIGNED term into one accumulator IN SHARD VISIT ORDER, and the total feeds
            // back. Float addition is not associative, so this makes the answer
            // schedule-dependent — which is what makes the reorder harness able to fail.
            //
            // TWO properties the instrument needs, and both had to be earned — the first two
            // attempts at this mutation were unobservable, which is the exact failure the
            // whole mutation exists to prevent, so the reasoning is kept rather than tidied:
            //
            // (1) **Observable.** A sum of ~200 similar-sized values is very nearly
            //     order-insensitive — its ordering spread is about ONE ULP of the total, and
            //     any feedback scaling rounds that straight back away. So the summands here
            //     span a wide dynamic range on purpose: the momentum terms drive the running
            //     total to ~1e14 while the occupancy terms are ~1e1, so each small term is
            //     absorbed against a large and ORDER-DEPENDENT running magnitude. The spread
            //     is then hundreds of ULPs instead of one.
            //
            // (2) **Bounded.** `acc` reads the INTEGER lanes, never the energy it feeds. An
            //     earlier version multiplied by energy and the feedback amplified itself
            //     ~26x per sweep, driving every cell to the same 1e17 value — an instrument
            //     that destroys the per-cell structure it is supposed to be measuring.
            //     A third attempt failed too, and for a reason worth keeping: summing
            //     `momentum * 1e12 + occupancy` is a sum of INTEGERS below 2^53, and float
            //     addition of those is EXACT — so it was perfectly order-independent. The
            //     summands have to be genuinely inexact AND span decades, so that each small
            //     term is only PARTIALLY absorbed into a large running total. Terms all the
            //     same size are nearly order-insensitive; terms entirely lost are
            //     order-insensitive too. Partial absorption is where non-associativity bites.
            let mut acc = 0.0f64;
            let mut weight = 0.0f64;
            for &s in &visit {
                let shard = &self.shards[s];
                for slot in 0..shard.owned_count {
                    let decade = 10f64.powi((slot % 13) as i32);
                    acc += shard.energy[slot] * decade;
                    weight += decade;
                }
            }
            // Normalised to a WEIGHTED MEAN, so the feedback is the size of an energy rather
            // than the size of the inflated sum. A fourth failed attempt is recorded here
            // too: dividing by a constant instead of by the weight left a gain of ~1e4 per
            // sweep, and the runaway drove every cell to the same 1e48 value — at which point
            // the two orders agreed again and the mutation stopped firing. Total gain per
            // sweep is now 0.5 (from `advance_energy`) + 0.3, which is below one.
            let feedback = if weight > 0.0 { 0.3 * acc / weight } else { 0.0 };
            for shard in &mut self.shards {
                for slot in 0..shard.owned_count {
                    shard.energy[slot] += feedback;
                }
            }
        }
        Ok(())
    }

    fn check_scene(&self, step: usize) -> Result<(), MeshError> {
        let total = self.total()?;
        if total != self.opening_total {
            return Err(MeshError::SceneMinted { step });
        }
        Ok(())
    }

    // -------------------------------------------------------------------- threaded path
    //
    // Written separately from the sequential path, and deliberately so: if the two were one
    // function behind a flag, "threaded == sequential" would be a tautology. Shards are
    // partitioned into contiguous chunks, one chunk per worker; every board slot has exactly
    // one writer per exchange, so the publish phase needs no locking.

    pub fn run_threaded(&mut self, colour_steps: usize, threads: usize) -> Result<(), MeshError> {
        if colour_steps == 0 {
            return Ok(());
        }
        if self.spec.mutation == Mutation::CrossShardFloatReduction {
            return Err(MeshError::Config(
                "the planted float reduction is a sequential-path instrument; running it \
                 threaded would measure thread scheduling, not merge order",
            ));
        }
        let grid = self.spec.grid;
        let n = self.spec.colours_per_exchange;
        let order = self.spec.order;
        let mutation = self.spec.mutation;
        let shard_count = self.shards.len();
        let chunk_size = shard_count.div_ceil(threads.max(1));
        let worker_count = shard_count.div_ceil(chunk_size);
        let barrier = Barrier::new(worker_count);
        let board = Board::new(grid.len());
        let shards = &mut self.shards;

        let panicked = std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(worker_count);
            for chunk in shards.chunks_mut(chunk_size) {
                let barrier = &barrier;
                let board = &board;
                handles.push(scope.spawn(move || {
                    for step in 0..colour_steps {
                        if step % n == 0 {
                            let skip = mutation == Mutation::HaloRefreshSkipped && step / n == 1;
                            if !skip && !board.failed() {
                                for shard in chunk.iter() {
                                    for (slot, &cell) in shard.owned_cells().iter().enumerate() {
                                        board.publish(cell as usize, shard.gross[slot], shard.energy[slot]);
                                    }
                                }
                            }
                            barrier.wait();
                            if !skip && !board.failed() {
                                for shard in chunk.iter_mut() {
                                    for slot in shard.owned_count..shard.cells.len() {
                                        let cell = shard.cells[slot] as usize;
                                        let (g, e) = board.read(cell);
                                        match g {
                                            Some(g) => {
                                                shard.gross[slot] = g;
                                                shard.energy[slot] = e;
                                            }
                                            None => board.fail(MeshError::LedgerRange {
                                                cell: cell as u32,
                                            }),
                                        }
                                    }
                                }
                            }
                            barrier.wait();
                        }
                        if !board.failed() {
                            let colour = step % EDGE_COLOURS;
                            for shard in chunk.iter_mut() {
                                if let Err(e) = sweep_shard(shard, colour, order, mutation) {
                                    board.fail(e);
                                    break;
                                }
                            }
                        }
                        barrier.wait();
                        if board.failed() {
                            break;
                        }
                    }
                }));
            }
            handles.into_iter().any(|h| h.join().is_err())
        });

        if panicked {
            return Err(MeshError::WorkerPanicked { threads });
        }
        if let Some(error) = board.first_error() {
            return Err(error);
        }
        let total = self.total()?;
        if total != self.opening_total {
            return Err(MeshError::SceneMinted { step: colour_steps });
        }
        self.steps_run += colour_steps;
        Ok(())
    }
}

/// One shard's sweep of one colour. Shared by both execution paths because it is the shard's
/// own interior work — the thing D4 calls "deterministic by construction, one block, fixed
/// loop order". What is written twice is the SCHEDULING around it, which is where a
/// concurrency defect would live.
fn sweep_shard(
    shard: &mut MeshShard,
    colour: usize,
    order: VisitOrder,
    mutation: Mutation,
) -> Result<(), MeshError> {
    let edges = &shard.colour_edges[colour];
    let visit = order.sequence(edges.len());
    for &e in &visit {
        let edge = edges[e];
        let (lo, hi) = (edge.lo as usize, edge.hi as usize);
        let depth = shard.depth[lo].max(shard.depth[hi]) as usize;
        if depth > shard.max_read_depth {
            shard.max_read_depth = depth;
        }
        let mut d = plan(shard.gross[lo], shard.gross[hi]).ok_or(MeshError::LedgerRange {
            cell: shard.cells[lo],
        })?;
        if mutation == Mutation::DoubleTransferBothSides && edge.cross {
            // `SANDBOX_4090` §6's finding, planted here: both sides apply twice. The scene
            // total is still exactly conserved and the two sides are still exactly
            // antisymmetric — every balance-based leg passes. Only comparison against the
            // re-derived answer sees it.
            d = d
                .checked_mul(2)
                .ok_or(MeshError::LedgerRange { cell: shard.cells[lo] })?;
        }
        if mutation == Mutation::PairOrientedByVisitOrder && edge.cross && shard.depth[hi] == 0 {
            // The defect: each shard debits WHATEVER END IT OWNS, instead of debiting the end
            // the global index makes `lo`. Both shards then believe they are the debited
            // side, so one adjacency moves quantity out of both ends. Locally sensible,
            // globally incoherent — the realistic form of an orientation bug.
            d = d.checked_neg().ok_or(MeshError::LedgerRange {
                cell: shard.cells[hi],
            })?;
        }
        let debit = d.checked_neg().ok_or(MeshError::LedgerRange {
            cell: shard.cells[lo],
        })?;
        shard.gross[lo] = apply_delta(shard.gross[lo], debit).ok_or(MeshError::LedgerRange {
            cell: shard.cells[lo],
        })?;
        shard.gross[hi] = apply_delta(shard.gross[hi], d).ok_or(MeshError::LedgerRange {
            cell: shard.cells[hi],
        })?;
    }
    // Whole-state advance: per cell, over the whole working set (halo copies included, so
    // they stay usable for the next colour). Reads one cell each, so this loop's order is
    // irrelevant — which is the property the float mutation removes.
    for slot in 0..shard.cells.len() {
        shard.energy[slot] = advance_energy(shard.energy[slot], shard.gross[slot]);
    }
    Ok(())
}

fn build_shard(
    grid: Grid,
    partition: &Partition,
    index: usize,
    horizon: usize,
) -> Result<MeshShard, MeshError> {
    let owned: Vec<u32> = partition.owned(index).to_vec();
    if owned.is_empty() {
        return Err(MeshError::Config("a shard owns no cells"));
    }
    let owned_count = owned.len();

    // Halo by breadth-first search over the 4-neighbour stencil, to `horizon` hops. This is
    // `Core/Locality.lean`'s ball, built explicitly.
    let mut depth_of = vec![u8::MAX; grid.len()];
    let mut frontier: Vec<u32> = Vec::new();
    for &c in &owned {
        depth_of[c as usize] = 0;
        frontier.push(c);
    }
    let mut halo: Vec<u32> = Vec::new();
    for d in 1..=horizon {
        let mut next = Vec::new();
        for &c in &frontier {
            for nb in neighbours(grid, c) {
                if depth_of[nb as usize] == u8::MAX {
                    depth_of[nb as usize] = d as u8;
                    halo.push(nb);
                    next.push(nb);
                }
            }
        }
        frontier = next;
    }
    halo.sort_unstable();

    let mut cells = owned.clone();
    cells.extend_from_slice(&halo);
    let depth: Vec<u8> = cells.iter().map(|&c| depth_of[c as usize]).collect();
    let gross: Vec<GrossState> = cells.iter().map(|&c| seed_gross(c)).collect();
    let energy: Vec<f64> = cells.iter().map(|&c| seed_energy(c)).collect();

    // Working index of a global cell, for this shard only.
    let mut work_of = vec![u32::MAX; grid.len()];
    for (slot, &c) in cells.iter().enumerate() {
        work_of[c as usize] = slot as u32;
    }

    // Evaluable edges per colour: those with BOTH endpoints in the working set. An edge with
    // one endpoint beyond the halo is skipped — its present endpoint is at the outermost
    // depth, whose correctness the horizon argument does not need and does not claim.
    let mut colour_edges = Vec::with_capacity(EDGE_COLOURS);
    for colour in 0..EDGE_COLOURS {
        let mut list = Vec::new();
        for edge in edges_of_colour(grid, colour) {
            let (a, b) = (work_of[edge.lo as usize], work_of[edge.hi as usize]);
            if a == u32::MAX || b == u32::MAX {
                continue;
            }
            list.push(WorkEdge {
                lo: a,
                hi: b,
                cross: partition.owner(edge.lo) != partition.owner(edge.hi),
            });
        }
        colour_edges.push(list);
    }

    // The arena covers the OWNED cells only. Halo copies are scratch and must never enter a
    // ledger that is summed into the scene total, or every boundary cell would be counted
    // twice.
    let mut root = GrossState::ZERO;
    for g in &gross[..owned_count] {
        root = root
            .checked_combine(*g)
            .ok_or(MeshError::Core(HolonError::GrossStateDoesNotCompose))?;
    }
    let channels = Channels::REG_PLUS.union(Channels::MECHANICAL);
    let mut specs = Vec::with_capacity(owned_count + 1);
    specs.push(RuntimeHolonSpec {
        parent: NO_RUNTIME_HOLON,
        depth: 0,
        grain_units: 2,
        gross: root,
        whole: &[],
        channels,
        boundary: true,
        decomposition: Decomposition::Expanded,
    });
    for (slot, g) in gross[..owned_count].iter().enumerate() {
        specs.push(RuntimeHolonSpec {
            parent: 0,
            depth: 1,
            grain_units: 1,
            gross: *g,
            whole: &[],
            channels,
            // A boundary port is an ordinary holon wearing the flag it already has: an owned
            // cell with at least one neighbour owned by someone else.
            boundary: neighbours(grid, cells[slot])
                .into_iter()
                .any(|nb| partition.owner(nb) != index as u32),
            decomposition: Decomposition::Leaf,
        });
    }
    let arena = RuntimeArena::from_specs(&specs, 0)?;

    Ok(MeshShard {
        index,
        arena,
        cells,
        owned_count,
        depth,
        gross,
        energy,
        colour_edges,
        max_read_depth: 0,
    })
}

/// The shared board: four integer lanes plus the whole-state scalar's raw bits, per cell.
/// No floats are ADDED here — the scalar is carried as `u64` bits and only ever copied, so
/// the board cannot become a float reduction by accident.
struct Board {
    lanes: Vec<AtomicI64>,
    energy: Vec<AtomicU64>,
    failed: AtomicBool,
    errors: Mutex<Vec<MeshError>>,
}

impl Board {
    fn new(cells: usize) -> Self {
        Self {
            lanes: (0..cells * LANES).map(|_| AtomicI64::new(0)).collect(),
            energy: (0..cells).map(|_| AtomicU64::new(0)).collect(),
            failed: AtomicBool::new(false),
            errors: Mutex::new(Vec::new()),
        }
    }

    #[inline]
    fn failed(&self) -> bool {
        self.failed.load(Ordering::Acquire)
    }

    fn fail(&self, error: MeshError) {
        if let Ok(mut errors) = self.errors.lock() {
            errors.push(error);
        }
        self.failed.store(true, Ordering::Release);
    }

    fn first_error(&self) -> Option<MeshError> {
        self.errors.lock().ok().and_then(|e| e.first().cloned())
    }

    #[inline]
    fn publish(&self, cell: usize, gross: GrossState, energy: f64) {
        if let Some(lanes) = gross_to_lanes(gross) {
            for (offset, lane) in lanes.iter().enumerate() {
                self.lanes[cell * LANES + offset].store(*lane, Ordering::Release);
            }
        } else {
            self.fail(MeshError::LedgerRange { cell: cell as u32 });
        }
        self.energy[cell].store(energy.to_bits(), Ordering::Release);
    }

    #[inline]
    fn read(&self, cell: usize) -> (Option<GrossState>, f64) {
        let mut lanes = [0i64; LANES];
        for (offset, lane) in lanes.iter_mut().enumerate() {
            *lane = self.lanes[cell * LANES + offset].load(Ordering::Acquire);
        }
        (
            lanes_to_gross(lanes),
            f64::from_bits(self.energy[cell].load(Ordering::Acquire)),
        )
    }
}
