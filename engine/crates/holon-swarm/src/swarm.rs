//! The swarm: N independent holon engines, one exchange round at a time.
//!
//! Two execution paths are provided *on purpose*, and they are written independently:
//!
//! * [`Swarm::run_rounds_sequential`] — plain loops, plain vectors, no threads, no atomics.
//!   This is the reference.
//! * [`Swarm::run_rounds_threaded`] — persistent scoped worker threads, contiguous shard
//!   chunks, three `Barrier`s per round, and a shared `AtomicI64` board for the round
//!   snapshot, the receipts and the published roots.
//!
//! The determinism claim is that these two produce **bit-identical** ledgers, per shard
//! and globally, for any thread count. If they were one code path with a flag, that claim
//! would be nearly vacuous; written twice, agreement is evidence.
//!
//! Threads are spawned once per *run*, not once per round: at 16 shards a spawn-per-phase
//! design would pay ~25 us x 2 per round, which is more than an entire round of real work.
//! The barrier is the synchronisation cost that remains, and the bench reports it.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Barrier, Mutex};

use ciris_sim_core::regplus::GrossState;

use crate::error::{Side, SwarmError};
use crate::exchange::{plan_transfer, resolve_side, BoundaryPair, FaultInjection};
use crate::gate::{
    cross_shard_legs, leg1_local_conservation, shard_local_legs, ConservationGate, GateLevel,
};
use crate::ledger::{apply_delta, gross_to_lanes, LedgerDelta, LANES};
use crate::shard::{Shard, ShardLink};

/// A swarm configuration, as values: how many shards, how big each is, and which shards
/// share a boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwarmSpec {
    pub shards: usize,
    pub leaves_per_shard: usize,
    /// Unordered shard-index pairs. Canonicalised to `lo < hi` at build time so the
    /// transfer's sign convention never depends on how the topology was written down.
    pub topology: Vec<(usize, usize)>,
    pub gate: GateLevel,
}

impl SwarmSpec {
    /// A ring: shard `i` shares a boundary with shard `i + 1 mod n`. One pair for n = 2,
    /// n pairs for n >= 3, none for n = 1.
    pub fn ring(shards: usize, leaves_per_shard: usize) -> Self {
        let topology = match shards {
            0 | 1 => Vec::new(),
            2 => vec![(0, 1)],
            n => (0..n).map(|i| (i, (i + 1) % n)).collect(),
        };
        Self {
            shards,
            leaves_per_shard,
            topology,
            gate: GateLevel::default(),
        }
    }

    /// A line: shard `i` shares a boundary with shard `i + 1`. Fewer pairs than a ring,
    /// and the end shards have exactly one link — useful for isolating a single pair.
    pub fn line(shards: usize, leaves_per_shard: usize) -> Self {
        Self {
            shards,
            leaves_per_shard,
            topology: (0..shards.saturating_sub(1)).map(|i| (i, i + 1)).collect(),
            gate: GateLevel::default(),
        }
    }

    /// Shards with no boundary relations at all. The exchange phase is a no-op, so this
    /// isolates the cost of the shards' internal work from the cost of the boundary
    /// protocol — the decomposition the GPU design needs in order to know whether the
    /// exchange or the kernel is the thing to optimise.
    pub fn isolated(shards: usize, leaves_per_shard: usize) -> Self {
        Self {
            shards,
            leaves_per_shard,
            topology: Vec::new(),
            gate: GateLevel::default(),
        }
    }

    pub fn with_gate(mut self, gate: GateLevel) -> Self {
        self.gate = gate;
        self
    }
}

/// Visit order for the sequential path. Used to *prove* order-independence by running the
/// same round with the shards and pairs visited in different orders and demanding
/// bit-identical results.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoundOrder {
    pub shards: Vec<usize>,
    pub pairs: Vec<usize>,
}

impl RoundOrder {
    pub fn natural(shards: usize, pairs: usize) -> Self {
        Self {
            shards: (0..shards).collect(),
            pairs: (0..pairs).collect(),
        }
    }

    pub fn reversed(shards: usize, pairs: usize) -> Self {
        Self {
            shards: (0..shards).rev().collect(),
            pairs: (0..pairs).rev().collect(),
        }
    }

    /// A deterministic non-trivial permutation (a stride walk over a coprime step).
    pub fn strided(shards: usize, pairs: usize) -> Self {
        fn walk(n: usize) -> Vec<usize> {
            if n == 0 {
                return Vec::new();
            }
            let step = if n.is_multiple_of(3) { 5 } else { 3 };
            let mut out = Vec::with_capacity(n);
            let mut seen = vec![false; n];
            let mut i = 0;
            while out.len() < n {
                if !seen[i] {
                    seen[i] = true;
                    out.push(i);
                }
                i = (i + step) % n;
                if seen[i] {
                    // Fall forward to the first unseen index so the walk always terminates.
                    if let Some(next) = (0..n).find(|j| !seen[*j]) {
                        i = next;
                    } else {
                        break;
                    }
                }
            }
            out
        }
        Self {
            shards: walk(shards),
            pairs: walk(pairs),
        }
    }
}

pub struct Swarm {
    shards: Vec<Shard>,
    pairs: Vec<BoundaryPair>,
    gate: ConservationGate,
    round: u64,
}

impl Swarm {
    pub fn new(spec: &SwarmSpec) -> Result<Self, SwarmError> {
        if spec.shards == 0 {
            return Err(SwarmError::Config("a swarm needs at least one shard"));
        }
        let mut links: Vec<Vec<ShardLink>> = vec![Vec::new(); spec.shards];
        let mut pairs = Vec::with_capacity(spec.topology.len());
        for (id, &(a, b)) in spec.topology.iter().enumerate() {
            if a == b {
                return Err(SwarmError::Config("a shard cannot border itself"));
            }
            if a >= spec.shards || b >= spec.shards {
                return Err(SwarmError::Config("topology names a shard that does not exist"));
            }
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            // One distinct port holon per link: port ids are allocated in link order, so
            // pair p writes a holon no other pair writes. That disjointness is what makes
            // the apply phase lock-free.
            let lo_holon = (links[lo].len() + 1) as u32;
            let hi_holon = (links[hi].len() + 1) as u32;
            links[lo].push(ShardLink {
                pair: id,
                side: Side::Lo,
                peer: hi,
                holon: lo_holon,
            });
            links[hi].push(ShardLink {
                pair: id,
                side: Side::Hi,
                peer: lo,
                holon: hi_holon,
            });
            pairs.push(BoundaryPair {
                id,
                lo: (lo, lo_holon),
                hi: (hi, hi_holon),
            });
        }

        let mut shards = Vec::with_capacity(spec.shards);
        for (index, shard_links) in links.into_iter().enumerate() {
            shards.push(Shard::new(index, spec.leaves_per_shard, shard_links)?);
        }

        let mut baseline = GrossState::ZERO;
        for shard in &shards {
            baseline = baseline
                .checked_combine(shard.root_ledger())
                .ok_or(SwarmError::GlobalLedgerOverflow { round: 0 })?;
        }
        Ok(Self {
            shards,
            pairs,
            gate: ConservationGate::new(baseline, spec.gate),
            round: 0,
        })
    }

    pub fn shards(&self) -> &[Shard] {
        &self.shards
    }

    pub fn pairs(&self) -> &[BoundaryPair] {
        &self.pairs
    }

    pub fn round(&self) -> u64 {
        self.round
    }

    pub fn gate(&self) -> &ConservationGate {
        &self.gate
    }

    /// The global REG+ ledger: the exact sum over every shard's root.
    pub fn global_ledger(&self) -> Result<GrossState, SwarmError> {
        let mut total = GrossState::ZERO;
        for shard in &self.shards {
            total = total
                .checked_combine(shard.root_ledger())
                .ok_or(SwarmError::GlobalLedgerOverflow { round: self.round })?;
        }
        Ok(total)
    }

    /// Every shard's root ledger, in shard order — the per-shard fingerprint the
    /// determinism harness compares.
    pub fn shard_ledgers(&self) -> Vec<GrossState> {
        self.shards.iter().map(|s| s.root_ledger()).collect()
    }

    /// Every holon's ledger in every shard, plus every whole-state scalar's raw bits.
    /// The strongest fingerprint available: nothing about the swarm is outside it.
    pub fn full_fingerprint(&self) -> (Vec<GrossState>, Vec<u64>) {
        let mut ledgers = Vec::new();
        let mut bits = Vec::new();
        for shard in &self.shards {
            ledgers.extend_from_slice(shard.ledger());
            bits.extend(shard.whole_bits());
        }
        (ledgers, bits)
    }

    // ---------------------------------------------------------------- sequential path

    pub fn run_rounds_sequential(
        &mut self,
        rounds: u64,
        fault: FaultInjection,
    ) -> Result<(), SwarmError> {
        let order = RoundOrder::natural(self.shards.len(), self.pairs.len());
        for _ in 0..rounds {
            self.step_round_sequential(&order, fault)?;
        }
        Ok(())
    }

    /// One exchange round, single-threaded, with an explicit visit order.
    pub fn step_round_sequential(
        &mut self,
        order: &RoundOrder,
        fault: FaultInjection,
    ) -> Result<(), SwarmError> {
        let round = self.round;

        // ---- Phase A: every shard steps its own interior. Shards are disjoint, so the
        // visit order cannot matter; L1 checks each one minted nothing.
        for &s in &order.shards {
            let before = self.shards[s].root_ledger();
            self.shards[s].local_step(round, fault)?;
            let after = self.shards[s].root_ledger();
            self.shards[s].post_local = after;
            leg1_local_conservation(round, s, before, after)?;
        }

        // ---- Publish the round snapshot. EVERY port is read here, before ANY transfer is
        // applied. This is the load-bearing step for order-independence.
        let mut snapshot = vec![[0i64; LANES]; self.pairs.len() * 2];
        for pair in &self.pairs {
            snapshot[pair.id * 2 + Side::Lo.index()] =
                port_lanes(&self.shards[pair.lo.0], pair.lo.1, round)?;
            snapshot[pair.id * 2 + Side::Hi.index()] =
                port_lanes(&self.shards[pair.hi.0], pair.hi.1, round)?;
        }

        // ---- Phase B: plan from the snapshot, apply with opposite signs.
        for shard in &mut self.shards {
            shard.receipts.iter_mut().for_each(|r| *r = LedgerDelta::ZERO);
        }
        let mut receipts = vec![LedgerDelta::ZERO; self.pairs.len() * 2];
        for &p in &order.pairs {
            let pair = self.pairs[p];
            let plan = plan_transfer(
                snapshot[p * 2 + Side::Lo.index()],
                snapshot[p * 2 + Side::Hi.index()],
            )
            .ok_or(SwarmError::GlobalLedgerOverflow { round })?;

            for (side, endpoint) in [(Side::Lo, pair.lo), (Side::Hi, pair.hi)] {
                let write = resolve_side(plan, p, side, fault)?;
                let shard = &mut self.shards[endpoint.0];
                shard.apply_at_port(round, endpoint.1, write.applied)?;
                let slot = shard
                    .links
                    .iter()
                    .position(|link| link.pair == p)
                    .ok_or(SwarmError::Config("pair not found on its own shard"))?;
                shard.receipts[slot] = write.receipted;
                receipts[p * 2 + side.index()] = write.receipted;
            }
        }

        self.apply_post_exchange_faults(round, fault)?;

        // ---- The gate.
        for &s in &order.shards {
            shard_local_legs(&self.gate, round, &self.shards[s])?;
        }
        let roots = self.root_lanes(round)?;
        cross_shard_legs(&self.gate, round, &self.pairs, &snapshot, &receipts, &roots)?;

        self.round += 1;
        Ok(())
    }

    fn apply_post_exchange_faults(
        &mut self,
        round: u64,
        fault: FaultInjection,
    ) -> Result<(), SwarmError> {
        let one = LedgerDelta {
            occupancy: 1,
            ..LedgerDelta::ZERO
        };
        match fault {
            FaultInjection::BreakComposition { shard } if shard < self.shards.len() => {
                self.shards[shard].inject_raw(round, 1, one)
            }
            FaultInjection::RootOnlyCredit { shard } if shard < self.shards.len() => {
                let root = self.shards[shard].root_id();
                self.shards[shard].inject_raw(round, root, one)
            }
            _ => Ok(()),
        }
    }

    fn root_lanes(&self, round: u64) -> Result<Vec<[i64; LANES]>, SwarmError> {
        self.shards
            .iter()
            .map(|shard| {
                gross_to_lanes(shard.root_ledger())
                    .ok_or(SwarmError::GlobalLedgerOverflow { round })
            })
            .collect()
    }

    // ------------------------------------------------------------------ threaded path

    /// Run `rounds` exchange rounds across `threads` persistent worker threads.
    ///
    /// Shards are partitioned into contiguous chunks, one chunk per worker. Because each
    /// pair writes its own two port holons and each shard's root is written only by that
    /// shard's owner, the apply phase needs no locks: the only shared writes are the
    /// `AtomicI64` board, and every board slot has exactly one writer per round.
    pub fn run_rounds_threaded(
        &mut self,
        rounds: u64,
        threads: usize,
        fault: FaultInjection,
    ) -> Result<(), SwarmError> {
        if rounds == 0 {
            return Ok(());
        }
        let Swarm {
            shards,
            pairs,
            gate,
            round,
        } = self;
        let shard_count = shards.len();
        let chunk_size = shard_count.div_ceil(threads.max(1));
        let worker_count = shard_count.div_ceil(chunk_size);
        let barrier = Barrier::new(worker_count);
        let board = Board::new(pairs.len(), shard_count);
        let start_round = *round;
        let pairs: &[BoundaryPair] = pairs;
        let gate: &ConservationGate = gate;

        let panicked = std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(worker_count);
            for chunk in shards.chunks_mut(chunk_size) {
                let barrier = &barrier;
                let board = &board;
                handles.push(scope.spawn(move || {
                    for r in 0..rounds {
                        let round = start_round + r;

                        // ---- Phase A: local steps + publish this round's port snapshot.
                        if !board.failed() {
                            let outcome = (|| -> Result<(), SwarmError> {
                                for shard in chunk.iter_mut() {
                                    let before = shard.root_ledger();
                                    shard.local_step(round, fault)?;
                                    let after = shard.root_ledger();
                                    shard.post_local = after;
                                    leg1_local_conservation(round, shard.index(), before, after)?;
                                    for slot in 0..shard.links.len() {
                                        let link = shard.links[slot];
                                        let lanes = port_lanes(shard, link.holon, round)?;
                                        board.publish_snapshot(link.pair, link.side, lanes);
                                    }
                                }
                                Ok(())
                            })();
                            if let Err(e) = outcome {
                                board.fail(e);
                            }
                        }
                        barrier.wait();

                        // ---- Phase B: plan from the snapshot, apply, receipt, publish.
                        // Both sides re-plan the SAME transfer from the SAME published
                        // values. Redundant recomputation instead of a shared plan buffer:
                        // integer arithmetic on identical inputs is bit-identical, so the
                        // two sides cannot disagree, and no writer/reader handoff is needed.
                        if !board.failed() {
                            let outcome = (|| -> Result<(), SwarmError> {
                                for shard in chunk.iter_mut() {
                                    for slot in 0..shard.links.len() {
                                        let link = shard.links[slot];
                                        let plan = plan_transfer(
                                            board.read_snapshot(link.pair, Side::Lo),
                                            board.read_snapshot(link.pair, Side::Hi),
                                        )
                                        .ok_or(SwarmError::GlobalLedgerOverflow { round })?;
                                        let write = resolve_side(plan, link.pair, link.side, fault)?;
                                        shard.apply_at_port(round, link.holon, write.applied)?;
                                        shard.receipts[slot] = write.receipted;
                                        board.publish_receipt(
                                            link.pair,
                                            link.side,
                                            write.receipted,
                                        );
                                    }
                                    apply_post_exchange_fault_to(shard, round, fault)?;
                                    shard_local_legs(gate, round, shard)?;
                                    let lanes = gross_to_lanes(shard.root_ledger())
                                        .ok_or(SwarmError::GlobalLedgerOverflow { round })?;
                                    board.publish_root(shard.index(), lanes);
                                }
                                Ok(())
                            })();
                            if let Err(e) = outcome {
                                board.fail(e);
                            }
                        }
                        let leader = barrier.wait().is_leader();

                        // ---- Phase C: one worker runs the cross-shard legs. Read-only, so
                        // which worker wins the barrier cannot affect the result.
                        if leader && !board.failed() {
                            let snapshot = board.snapshot_vec();
                            let receipts = board.receipts_vec();
                            let roots = board.roots_vec();
                            if let Err(e) =
                                cross_shard_legs(gate, round, pairs, &snapshot, &receipts, &roots)
                            {
                                board.fail(e);
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
            return Err(SwarmError::WorkerPanicked { threads });
        }
        if let Some(error) = board.first_error() {
            return Err(error);
        }
        *round = start_round + rounds;
        Ok(())
    }
}

fn port_lanes(shard: &Shard, holon: u32, round: u64) -> Result<[i64; LANES], SwarmError> {
    gross_to_lanes(shard.ledger()[holon as usize]).ok_or(SwarmError::LedgerOverflow {
        round,
        shard: shard.index(),
        holon,
    })
}

fn apply_post_exchange_fault_to(
    shard: &mut Shard,
    round: u64,
    fault: FaultInjection,
) -> Result<(), SwarmError> {
    let one = LedgerDelta {
        occupancy: 1,
        ..LedgerDelta::ZERO
    };
    match fault {
        FaultInjection::BreakComposition { shard: target } if target == shard.index() => {
            shard.inject_raw(round, 1, one)
        }
        FaultInjection::RootOnlyCredit { shard: target } if target == shard.index() => {
            let root = shard.root_id();
            shard.inject_raw(round, root, one)
        }
        _ => Ok(()),
    }
}

/// The shared board: everything a round needs to publish so that another worker (or the
/// gate) can read it. Four `i64` lanes per slot, no floats, no pointers — the same shape a
/// GPU staging buffer would have.
struct Board {
    snapshot: Vec<AtomicI64>,
    receipts: Vec<AtomicI64>,
    roots: Vec<AtomicI64>,
    failed: AtomicBool,
    errors: Mutex<Vec<SwarmError>>,
}

impl Board {
    fn new(pairs: usize, shards: usize) -> Self {
        let sides = pairs * 2 * LANES;
        Self {
            snapshot: (0..sides).map(|_| AtomicI64::new(0)).collect(),
            receipts: (0..sides).map(|_| AtomicI64::new(0)).collect(),
            roots: (0..shards * LANES).map(|_| AtomicI64::new(0)).collect(),
            failed: AtomicBool::new(false),
            errors: Mutex::new(Vec::new()),
        }
    }

    #[inline]
    fn failed(&self) -> bool {
        self.failed.load(Ordering::Acquire)
    }

    fn fail(&self, error: SwarmError) {
        if let Ok(mut errors) = self.errors.lock() {
            errors.push(error);
        }
        self.failed.store(true, Ordering::Release);
    }

    fn first_error(&self) -> Option<SwarmError> {
        self.errors.lock().ok().and_then(|e| e.first().cloned())
    }

    #[inline]
    fn publish_snapshot(&self, pair: usize, side: Side, lanes: [i64; LANES]) {
        store(&self.snapshot, (pair * 2 + side.index()) * LANES, lanes);
    }

    #[inline]
    fn read_snapshot(&self, pair: usize, side: Side) -> [i64; LANES] {
        load(&self.snapshot, (pair * 2 + side.index()) * LANES)
    }

    #[inline]
    fn publish_receipt(&self, pair: usize, side: Side, receipt: LedgerDelta) {
        store(
            &self.receipts,
            (pair * 2 + side.index()) * LANES,
            receipt.to_lanes(),
        );
    }

    #[inline]
    fn publish_root(&self, shard: usize, lanes: [i64; LANES]) {
        store(&self.roots, shard * LANES, lanes);
    }

    fn snapshot_vec(&self) -> Vec<[i64; LANES]> {
        (0..self.snapshot.len() / LANES)
            .map(|i| load(&self.snapshot, i * LANES))
            .collect()
    }

    fn receipts_vec(&self) -> Vec<LedgerDelta> {
        (0..self.receipts.len() / LANES)
            .map(|i| LedgerDelta::from_lanes(load(&self.receipts, i * LANES)))
            .collect()
    }

    fn roots_vec(&self) -> Vec<[i64; LANES]> {
        (0..self.roots.len() / LANES)
            .map(|i| load(&self.roots, i * LANES))
            .collect()
    }
}

#[inline]
fn store(slots: &[AtomicI64], base: usize, lanes: [i64; LANES]) {
    for (offset, lane) in lanes.iter().enumerate() {
        slots[base + offset].store(*lane, Ordering::Release);
    }
}

#[inline]
fn load(slots: &[AtomicI64], base: usize) -> [i64; LANES] {
    let mut lanes = [0i64; LANES];
    for (offset, lane) in lanes.iter_mut().enumerate() {
        *lane = slots[base + offset].load(Ordering::Acquire);
    }
    lanes
}

/// Convenience for hosts: apply a delta to a gross state, re-exported so a caller need not
/// depend on the ledger module directly.
pub fn checked_apply(gross: GrossState, delta: LedgerDelta) -> Option<GrossState> {
    apply_delta(gross, delta)
}
