//! MESH-CLIFFORD-1 — the Clifford tier cut across cores by QUBIT COLUMNS.
//!
//! `coladaptive.rs` is one dense tableau on one core, as stim is. `mesh.rs`
//! shards a SUM (the magic tier's branch fold), where the merge law makes any
//! cut free. A tableau is not a sum, so this file has to earn the same freedom
//! a different way, and the whole design is about where that is possible:
//!
//! **The cut** (`MESH_CLIFFORD_PREREG.md` §1). The shard is a contiguous range
//! of qubit COLUMNS: with `S` shards over `n` qubits, shard `s` owns columns
//! `[s·n/S, (s+1)·n/S)`. One column is `2n/64` contiguous words in each plane,
//! so the cut is a cut of the allocation and not a view of it.
//!
//! **Why the sign register does not break it.** Every gate WRITES the sign
//! register `r` (one bit per row) and NONE of them reads it: `H`/`S` do
//! `r ^= x&z`, `X` does `r ^= z`, `CX` does `r ^= x_c & z_t & ~(x_t ^ z_c)`.
//! A write-only XOR accumulator is a ledger under `merge`'s law — so each
//! shard keeps a PRIVATE partial `r`, folds its own gates into it, and the
//! parent XORs the partials **in shard order**. No atomic, no shared
//! accumulator, no completion-order reduction; the schedule is not an input.
//! That is the same discipline `mesh::fold_amplitude` uses one tier down.
//!
//! **Why gates can run at all.** A single-qubit gate touches one column, a CX
//! two. Gates are therefore buffered and cut into LAYERS — a layer is a
//! maximal run of consecutive gates whose columns are pairwise disjoint — and
//! inside a layer every gate reads and writes only its own columns, so the
//! layer's effect is independent of the order the gates are applied in and of
//! which thread applies them. The layering is a pure function of the gate
//! stream (greedy, first collision ends the layer), so it is not a schedule
//! either. A layer that is too small to pay for the spawn runs on the calling
//! thread; that cannot change the answer, only the wall.
//!
//! **CX across a boundary** is the one communication. The two columns are
//! copied into an exchange buffer (`4 · 2n/64` words: `x_c, x_t, z_c, z_t`),
//! the gate runs on the copies, and they are written back. That is a real
//! cost — at `d = 141` one crossing gate moves ~40 KB — so the crossing count
//! and the exchange traffic are REPORTED per run and never assumed (prereg
//! G2). See the note in the prereg's `### Notes on building` about what the
//! crossing fraction actually is under the flagship's natural qubit numbering.
//!
//! **Measurement.** The determinism scan is a read of one column, so it is
//! shard-local and needs no communication at all. When the outcome is random
//! the cascade is a ROWSUM — `row_i *= pivot` over a set of rows — and a
//! rowsum decomposes over column ranges exactly: the XOR of the planes is
//! per-word, and Aaronson–Gottesman's phase `g` is `(Σ plus − Σ minus) mod 4`
//! over popcounts of per-word masks, so each shard computes a PARTIAL
//! `(plus, minus)` on its own word range and the parent folds them in shard
//! order. Integer addition is associative and commutative, so the fold's
//! VALUE is order-free; the declared order is what makes the REPRESENTATION
//! (which partial was added first) irrelevant as well, and plant P2 is the
//! test that says so rather than the comment.
//!
//! **The boundary is snapped to 64 columns**, and this is the one place the
//! cut as written in §1 could not be implemented literally. The row-major
//! reference packs a row's `n` qubits into `n/64` words; a shard boundary in
//! the middle of a word would put two shards in one `u64`, and a rowsum could
//! then only be split with an atomic (forbidden) or a read-modify-write race
//! (wrong). So the prereg's boundary `s·n/S` is rounded to the nearest
//! multiple of 64 — at `d = 141, S = 8` that moves a boundary by at most 63
//! of 4970 columns. The cut is otherwise exactly §1's.
//!
//! **The gate is bit-identity, not speed.** Every outcome bit in circuit
//! order and every bit of the final tableau must be identical at every `S`,
//! and identical to the unsharded `ColAdaptive`, which is itself gated against
//! the row-major `PackedTableau`. `tests/mesh_clifford_plants.rs` is that gate
//! plus the four plants.

use crate::coladaptive::ScanStats;
use crate::phase::{Phase, PhaseProfile};
use crate::tableau::{PackedTableau, PauliRow};

/// THE CHART: `[0, n)` cut into contiguous, gap-free, ascending column ranges,
/// one per shard, declared up front and never negotiated at run time — the
/// same contract `mesh::shard_ranges` gives the branch fold.
///
/// Interior boundaries are the prereg's `s·n/S` rounded to the nearest
/// multiple of 64 (see the module header: the row-major reference's word
/// grain). Boundaries that collide after rounding are dropped, so
/// [`ShardCut::shards`] is the number of shards that will actually run and
/// may be smaller than the number requested — the caller is told which, and
/// nothing downstream ever assumes the requested value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShardCut {
    pub n: usize,
    /// `shards + 1` ascending boundaries, `bounds[0] = 0`, `bounds[S] = n`.
    pub bounds: Vec<usize>,
}

impl ShardCut {
    pub fn new(n: usize, shards: usize) -> Self {
        assert!(n > 0, "a cut needs qubits");
        let s = shards.max(1);
        let mut bounds = Vec::with_capacity(s + 1);
        bounds.push(0usize);
        for i in 1..s {
            let raw = i * n / s;
            let snapped = raw.div_ceil(64) * 64;
            let b = snapped.min(n);
            if b > *bounds.last().expect("non-empty") && b < n {
                bounds.push(b);
            }
        }
        bounds.push(n);
        ShardCut { n, bounds }
    }

    /// The number of shards that will RUN (never the number requested).
    #[inline]
    pub fn shards(&self) -> usize {
        self.bounds.len() - 1
    }

    #[inline]
    pub fn range(&self, s: usize) -> std::ops::Range<usize> {
        self.bounds[s]..self.bounds[s + 1]
    }

    /// Which shard owns column `q`.
    #[inline]
    pub fn shard_of(&self, q: usize) -> usize {
        debug_assert!(q < self.n);
        self.bounds.partition_point(|&b| b <= q) - 1
    }

    /// The shard's range in ROW-major word units (the reference's packing of
    /// the qubit axis). Well-defined because the boundaries are word-snapped.
    #[inline]
    pub fn row_words(&self, s: usize) -> std::ops::Range<usize> {
        self.bounds[s] / 64..self.bounds[s + 1].div_ceil(64)
    }
}

/// What the cut COST, counted rather than modelled. Reported per run.
#[derive(Clone, Copy, Debug, Default)]
pub struct MeshStats {
    /// Shards actually running (post-snap, post-clamp).
    pub shards: usize,
    /// CX gates applied, total.
    pub cx_total: u64,
    /// ...of which crossed a shard boundary and paid the column exchange.
    pub cx_crossing: u64,
    /// Gate layers executed (a layer is a maximal run of column-disjoint
    /// gates; the unit of parallelism in the gate phase).
    pub layers: u64,
    /// ...of which were big enough to pay for threads.
    pub layers_parallel: u64,
    /// Words copied into and out of the exchange buffer (each crossing gate
    /// moves `4 · 2n/64` words in and the same back out).
    pub exchange_words: u64,
    /// High-water mark of the exchange buffer itself, in bytes — the only
    /// memory the cut adds to the unsharded working set.
    pub exchange_peak_bytes: u64,
    /// Rows multiplied by a pivot in collapse cascades (the rowsum work).
    pub rowsum_rows: u64,
    /// Cascades folded across shards...
    pub rowsums_parallel: u64,
    /// ...and cascades too small to be worth the spawn.
    pub rowsums_serial: u64,
}

impl MeshStats {
    pub fn crossing_fraction(&self) -> f64 {
        if self.cx_total == 0 {
            0.0
        } else {
            self.cx_crossing as f64 / self.cx_total as f64
        }
    }
}

/// A buffered gate. Buffering is what makes a layer — and a layer is what
/// makes the gate phase shardable at all (one gate touches one shard).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    H(usize),
    S(usize),
    Sdg(usize),
    X(usize),
    Z(usize),
    Cx(usize, usize),
}

impl Op {
    #[inline]
    fn cols(&self) -> (usize, Option<usize>) {
        match *self {
            Op::H(q) | Op::S(q) | Op::Sdg(q) | Op::X(q) | Op::Z(q) => (q, None),
            Op::Cx(c, t) => (c, Some(t)),
        }
    }
}

// ---------------------------------------------------------------------------
// The kernels. Byte for byte the ones in `coltableau.rs`, lifted to take raw
// column slices so a thread can hold its own shard's columns and nothing else.
// They are duplicated rather than shared because the unsharded engine is the
// REFERENCE this file is gated against, and a reference you edit to make the
// new path work is not a reference.
// ---------------------------------------------------------------------------

#[inline]
fn k_h(x: &mut [u64], z: &mut [u64], r: &mut [u64]) {
    for ((xr, zr), rr) in x.iter_mut().zip(z.iter_mut()).zip(r.iter_mut()) {
        *rr ^= *xr & *zr;
        std::mem::swap(xr, zr);
    }
}

#[inline]
fn k_s(x: &[u64], z: &mut [u64], r: &mut [u64]) {
    for ((xr, zr), rr) in x.iter().zip(z.iter_mut()).zip(r.iter_mut()) {
        *rr ^= *xr & *zr;
        *zr ^= *xr;
    }
}

#[inline]
fn k_sdg(x: &[u64], z: &mut [u64], r: &mut [u64]) {
    for ((xr, zr), rr) in x.iter().zip(z.iter_mut()).zip(r.iter_mut()) {
        *rr ^= *xr & !*zr;
        *zr ^= *xr;
    }
}

#[inline]
fn k_x(z: &[u64], r: &mut [u64]) {
    for (rr, zr) in r.iter_mut().zip(z.iter()) {
        *rr ^= *zr;
    }
}

#[inline]
fn k_z(x: &[u64], r: &mut [u64]) {
    for (rr, xr) in r.iter_mut().zip(x.iter()) {
        *rr ^= *xr;
    }
}

#[inline]
fn k_cx(xc: &[u64], xt: &mut [u64], zc: &mut [u64], zt: &[u64], r: &mut [u64]) {
    for i in 0..xc.len() {
        let (xcw, zcw) = (xc[i], zc[i]);
        let (xtw, ztw) = (xt[i], zt[i]);
        r[i] ^= xcw & ztw & !(xtw ^ zcw);
        xt[i] = xtw ^ xcw;
        zc[i] = zcw ^ ztw;
    }
}

/// Disjoint mutable views of two columns in one flat plane (`coltableau`'s).
#[inline(always)]
fn two_cols(
    plane: &mut [u64],
    a: usize,
    b: usize,
    words: usize,
) -> (&mut [u64], &mut [u64]) {
    if a < b {
        let (lo, hi) = plane.split_at_mut(b * words);
        (&mut lo[a * words..(a + 1) * words], &mut hi[..words])
    } else {
        let (lo, hi) = plane.split_at_mut(a * words);
        let (bcol, acol) = (&mut lo[b * words..(b + 1) * words], &mut hi[..words]);
        (acol, bcol)
    }
}

/// One plane cut into the chart's column ranges — the physical partition, as
/// `S` disjoint `&mut` slices that the borrow checker itself certifies do not
/// overlap. This is the whole of the "no shared state" claim, checked by the
/// compiler rather than asserted in a comment.
fn split_cols<'a>(plane: &'a mut [u64], bounds: &[usize], words: usize) -> Vec<&'a mut [u64]> {
    let mut out = Vec::with_capacity(bounds.len() - 1);
    let mut rest: &mut [u64] = plane;
    for w in bounds.windows(2) {
        let take = std::mem::take(&mut rest);
        let (a, b) = take.split_at_mut((w[1] - w[0]) * words);
        out.push(a);
        rest = b;
    }
    out
}

fn splitmix(state: &mut u64) -> bool {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) & 1 == 1
}

/// Where the identity check found the corruption — plant P1's verdict, which
/// must NAME the shard and the column rather than merely report "differs".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Divergence {
    /// The shard that owns the column — `None` only for the sign register,
    /// which is a row object and belongs to no single shard.
    pub shard: Option<usize>,
    pub column: Option<usize>,
    pub row: usize,
    pub plane: &'static str,
}

impl std::fmt::Display for Divergence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.shard, self.column) {
            (Some(s), Some(c)) => write!(
                f,
                "shard {s} column {c} row {} ({} plane) differs from the reference",
                self.row, self.plane
            ),
            _ => write!(
                f,
                "the sign register differs from the reference at row {}",
                self.row
            ),
        }
    }
}

/// The column-major adaptive Clifford engine, cut into `S` column shards.
///
/// The API is `ColAdaptive`'s, with one addition the cut forces: gates are
/// BUFFERED until the next state read (a measurement batch, `to_packed`, a
/// logical observable), because a layer of column-disjoint gates is the unit
/// of parallelism and a single gate is not. Nothing outside this file has to
/// know that: `begin_batch` flushes, and so does every read.
pub struct ShardedColAdaptive {
    pub n: usize,
    cut: ShardCut,
    /// Words per column (`2n` rows).
    words: usize,
    x: Vec<u64>,
    z: Vec<u64>,
    /// The folded sign register: authoritative whenever `pending` is empty.
    r: Vec<u64>,
    /// One private sign accumulator per shard — never shared, never atomic.
    rpart: Vec<Vec<u64>>,
    /// The order the shard partials are folded in. `0..S` unless a plant
    /// scrambles it (P2), which must change nothing.
    fold_order: Vec<usize>,
    pending: Vec<Op>,
    layer_stamp: Vec<u64>,
    stamp: u64,
    /// The column exchange, reused across layers and reported in `mesh`.
    exchange: Vec<u64>,
    /// Layers smaller than this (gates × words) run on the calling thread.
    par_min_work: usize,
    /// The re-aim (`--transpose-parallel`): cut the column→row transpose by
    /// row blocks across `S` threads. Off by default; bit-identical either way.
    par_store: bool,
    packed: Option<PackedTableau>,
    packed_valid: bool,
    in_batch: bool,
    mirror_x_valid: bool,
    mirror_full_valid: bool,
    dirty: bool,
    rng: u64,
    /// P3's carrier: when set, the declared stream is consumed OUT of circuit
    /// order (pairwise swapped), which must change the record.
    scramble_stream: bool,
    stash: Option<bool>,
    pub seed: u64,
    pub stats: ScanStats,
    pub mesh: MeshStats,
    /// The phase timer (MESH-CLIFFORD-1's profile). Off unless enabled.
    pub prof: PhaseProfile,
}

impl ShardedColAdaptive {
    pub fn new(n: usize, seed: u64, shards: usize) -> Self {
        let cut = ShardCut::new(n, shards);
        let s = cut.shards();
        let words = (2 * n).div_ceil(64);
        let mut e = ShardedColAdaptive {
            n,
            cut,
            words,
            x: vec![0u64; n * words],
            z: vec![0u64; n * words],
            r: vec![0u64; words],
            rpart: vec![vec![0u64; words]; s],
            fold_order: (0..s).collect(),
            pending: Vec::new(),
            layer_stamp: vec![0u64; n],
            stamp: 0,
            exchange: Vec::new(),
            par_min_work: 4096,
            par_store: false,
            packed: None,
            packed_valid: false,
            in_batch: false,
            mirror_x_valid: false,
            mirror_full_valid: false,
            dirty: false,
            rng: seed,
            scramble_stream: false,
            stash: None,
            seed,
            stats: ScanStats::default(),
            mesh: MeshStats { shards: s, ..Default::default() },
            prof: PhaseProfile::default(),
        };
        for i in 0..n {
            e.x[i * words + (i >> 6)] |= 1 << (i & 63); // destabilizer i = X_i
            let st = n + i;
            e.z[i * words + (st >> 6)] |= 1 << (st & 63); // stabilizer i = Z_i
        }
        e
    }

    pub fn cut(&self) -> &ShardCut {
        &self.cut
    }

    pub fn shards(&self) -> usize {
        self.cut.shards()
    }

    /// P2's carrier: fold the shard partials in a scrambled order. The record
    /// and the phase bit must not move.
    pub fn set_fold_order(&mut self, order: Vec<usize>) {
        let mut seen = vec![false; self.cut.shards()];
        assert_eq!(order.len(), self.cut.shards(), "fold order must cover every shard");
        for &s in &order {
            assert!(!std::mem::replace(&mut seen[s], true), "fold order repeats shard {s}");
        }
        self.fold_order = order;
    }

    /// P3's carrier: consume the declared random stream out of circuit order.
    pub fn set_stream_scramble(&mut self, on: bool) {
        self.scramble_stream = on;
    }

    /// Below this much work (gates × words per column) a layer runs on the
    /// calling thread. Tests set it to 0 to force the threaded path on small
    /// circuits; it cannot change an answer either way.
    pub fn set_parallel_min_work(&mut self, w: usize) {
        self.par_min_work = w;
    }

    /// Turn the phase timer on or off. It cannot change an answer.
    pub fn set_profiling(&mut self, on: bool) {
        self.prof.enabled = on;
    }

    /// The re-aim: run the column→row transpose on `S` threads, cut by row
    /// blocks rather than by the column chart. It cannot change an answer.
    pub fn set_parallel_transpose(&mut self, on: bool) {
        self.par_store = on;
    }

    // ---- the gate phase: buffer, then layer ----

    #[inline]
    fn push(&mut self, op: Op) {
        debug_assert!(!self.in_batch, "gates belong outside a measurement batch");
        self.pending.push(op);
        self.packed_valid = false;
    }

    pub fn h(&mut self, q: usize) {
        self.push(Op::H(q));
    }
    pub fn s(&mut self, q: usize) {
        self.push(Op::S(q));
    }
    pub fn sdg(&mut self, q: usize) {
        self.push(Op::Sdg(q));
    }
    pub fn x_gate(&mut self, q: usize) {
        self.push(Op::X(q));
    }
    pub fn z_gate(&mut self, q: usize) {
        self.push(Op::Z(q));
    }
    pub fn cx(&mut self, c: usize, t: usize) {
        assert_ne!(c, t, "cx: control equals target");
        self.mesh.cx_total += 1;
        if self.cut.shard_of(c) != self.cut.shard_of(t) {
            self.mesh.cx_crossing += 1;
        }
        self.push(Op::Cx(c, t));
    }

    /// Apply every buffered gate. Public because a caller timing the gate
    /// phase separately from the measurement phase (the flagship does) must
    /// be able to say where one ends.
    pub fn flush(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let ops = std::mem::take(&mut self.pending);
        if self.cut.shards() == 1 {
            self.mesh.layers += 1;
            self.run_serial(&ops);
        } else {
            let mut i = 0usize;
            let mut t = self.prof.start();
            while i < ops.len() {
                self.stamp += 1;
                let stamp = self.stamp;
                let mut j = i;
                while j < ops.len() {
                    let (a, b) = ops[j].cols();
                    if self.layer_stamp[a] == stamp || b.is_some_and(|b| self.layer_stamp[b] == stamp)
                    {
                        break;
                    }
                    self.layer_stamp[a] = stamp;
                    if let Some(b) = b {
                        self.layer_stamp[b] = stamp;
                    }
                    j += 1;
                }
                self.mesh.layers += 1;
                self.prof.stop(Phase::GateLayering, t);
                self.run_layer(&ops[i..j]);
                t = self.prof.start();
                i = j;
            }
            self.prof.stop(Phase::GateLayering, t);
        }
        let mut ops = ops;
        ops.clear();
        self.pending = ops;
    }

    /// The calling thread, in gate order — the S = 1 path and the small-layer
    /// path. Identical arithmetic to the threaded one; only the accumulator
    /// differs (the register itself instead of a partial).
    fn run_serial(&mut self, ops: &[Op]) {
        let w = self.words;
        for &op in ops {
            let t = self.prof.start();
            match op {
                Op::H(q) => {
                    // Two planes, two allocations: no aliasing to prove.
                    let (xs, ze) = (q * w, (q + 1) * w);
                    k_h(&mut self.x[xs..ze], &mut self.z[xs..ze], &mut self.r);
                }
                Op::S(q) => {
                    let (xs, ze) = (q * w, (q + 1) * w);
                    let (x, z) = (&self.x[xs..ze], &mut self.z[xs..ze]);
                    k_s(x, z, &mut self.r);
                }
                Op::Sdg(q) => {
                    let (xs, ze) = (q * w, (q + 1) * w);
                    let (x, z) = (&self.x[xs..ze], &mut self.z[xs..ze]);
                    k_sdg(x, z, &mut self.r);
                }
                Op::X(q) => k_x(&self.z[q * w..(q + 1) * w], &mut self.r),
                Op::Z(q) => k_z(&self.x[q * w..(q + 1) * w], &mut self.r),
                Op::Cx(c, t) => {
                    let (xc, xt) = two_cols(&mut self.x, c, t, w);
                    let (zc, zt) = two_cols(&mut self.z, c, t, w);
                    k_cx(xc, xt, zc, zt, &mut self.r);
                }
            }
            if t.is_some() {
                let ph = match op {
                    Op::Cx(c, t) if self.cut.shard_of(c) != self.cut.shard_of(t) => Phase::CxCross,
                    Op::Cx(..) => Phase::CxLocal,
                    _ => Phase::Gate1q,
                };
                self.prof.stop(ph, t);
            }
        }
    }

    /// One layer of column-disjoint gates across the shards: gather the
    /// crossing columns, run every shard on its own memory with its own sign
    /// partial, scatter the crossings back, fold the partials in shard order.
    fn run_layer(&mut self, ops: &[Op]) {
        let s = self.cut.shards();
        let words = self.words;
        if s == 1 || ops.len() * words < self.par_min_work {
            self.run_serial(ops);
            return;
        }
        self.mesh.layers_parallel += 1;
        let t = self.prof.start();

        // ---- who does what: a pure function of the layer ----
        let mut local: Vec<Vec<Op>> = vec![Vec::new(); s];
        let mut cross: Vec<(usize, usize)> = Vec::new();
        for &op in ops {
            match op {
                Op::Cx(c, t) => {
                    let (sc, st) = (self.cut.shard_of(c), self.cut.shard_of(t));
                    if sc == st {
                        local[sc].push(op);
                    } else {
                        cross.push((c, t));
                    }
                }
                other => {
                    let (q, _) = other.cols();
                    local[self.cut.shard_of(q)].push(other);
                }
            }
        }

        // Single-qubit gates first, then in-shard CX, so a thread can time
        // the two kinds with one clock pair each. Inside a layer every gate
        // owns its columns and the sign partial is an XOR, so the order is
        // unobservable (the same argument that lets the layer be threaded).
        for l in local.iter_mut() {
            l.sort_by_key(|op| matches!(op, Op::Cx(..)));
        }
        let t = self.prof.stop(Phase::GateLayering, t);

        // ---- gather: one copy each way, and it is counted ----
        let need = cross.len() * 4 * words;
        if self.exchange.len() < need {
            self.exchange.resize(need, 0);
            self.mesh.exchange_peak_bytes = self.mesh.exchange_peak_bytes.max((need * 8) as u64);
        }
        for (k, &(c, t)) in cross.iter().enumerate() {
            let b = k * 4 * words;
            let (xc, xt) = (c * words, t * words);
            self.exchange[b..b + words].copy_from_slice(&self.x[xc..xc + words]);
            self.exchange[b + words..b + 2 * words].copy_from_slice(&self.x[xt..xt + words]);
            self.exchange[b + 2 * words..b + 3 * words].copy_from_slice(&self.z[xc..xc + words]);
            self.exchange[b + 3 * words..b + 4 * words].copy_from_slice(&self.z[xt..xt + words]);
        }
        self.mesh.exchange_words += (2 * need) as u64;

        // Crossing gates go to shards in contiguous blocks, so each shard's
        // exchange slice is contiguous too. The assignment is arbitrary and
        // says so: a crossing gate works on private copies, so which thread
        // runs it cannot be observed.
        let m = cross.len();
        let bounds = self.cut.bounds.clone();
        let mut ex_rest: &mut [u64] = &mut self.exchange[..m * 4 * words];
        let mut ex_slices: Vec<&mut [u64]> = Vec::with_capacity(s);
        let mut assigned: Vec<usize> = Vec::with_capacity(s);
        let mut done = 0usize;
        for i in 0..s {
            let hi = (i + 1) * m / s;
            let take = std::mem::take(&mut ex_rest);
            let (a, b) = take.split_at_mut((hi - done) * 4 * words);
            ex_slices.push(a);
            assigned.push(hi - done);
            ex_rest = b;
            done = hi;
        }

        let xs = split_cols(&mut self.x, &bounds, words);
        let zs = split_cols(&mut self.z, &bounds, words);
        let rps: Vec<&mut Vec<u64>> = self.rpart.iter_mut().collect();
        let t = self.prof.stop(Phase::CxCross, t);
        let timing = t.is_some();
        // Per-thread busy nanoseconds on (single-qubit, in-shard CX, crossing CX).
        let mut busy: Vec<[u64; 3]> = vec![[0u64; 3]; s];
        let busy_slots: Vec<&mut [u64; 3]> = busy.iter_mut().collect();

        std::thread::scope(|scope| {
            for ((((((xs_s, zs_s), rp), ops_s), ex_s), s_idx), busy_s) in xs
                .into_iter()
                .zip(zs)
                .zip(rps)
                .zip(local)
                .zip(ex_slices)
                .zip(0..s)
                .zip(busy_slots)
            {
                let lo = bounds[s_idx];
                let n_cross = assigned[s_idx];
                scope.spawn(move || {
                    let rp = rp.as_mut_slice();
                    let lap = |from: Option<std::time::Instant>, slot: &mut u64| {
                        from.map(|f| {
                            let now = std::time::Instant::now();
                            *slot += now.duration_since(f).as_nanos() as u64;
                            now
                        })
                    };
                    let mut tk = if timing { Some(std::time::Instant::now()) } else { None };
                    let mut in_cx = false;
                    for op in &ops_s {
                        if !in_cx && matches!(op, Op::Cx(..)) {
                            tk = lap(tk, &mut busy_s[0]);
                            in_cx = true;
                        }
                        match *op {
                            Op::H(q) => {
                                let o = (q - lo) * words;
                                k_h(&mut xs_s[o..o + words], &mut zs_s[o..o + words], rp);
                            }
                            Op::S(q) => {
                                let o = (q - lo) * words;
                                k_s(&xs_s[o..o + words], &mut zs_s[o..o + words], rp);
                            }
                            Op::Sdg(q) => {
                                let o = (q - lo) * words;
                                k_sdg(&xs_s[o..o + words], &mut zs_s[o..o + words], rp);
                            }
                            Op::X(q) => {
                                let o = (q - lo) * words;
                                k_x(&zs_s[o..o + words], rp);
                            }
                            Op::Z(q) => {
                                let o = (q - lo) * words;
                                k_z(&xs_s[o..o + words], rp);
                            }
                            Op::Cx(c, t) => {
                                let (xc, xt) = two_cols(xs_s, c - lo, t - lo, words);
                                let (zc, zt) = two_cols(zs_s, c - lo, t - lo, words);
                                k_cx(xc, xt, zc, zt, rp);
                            }
                        }
                    }
                    tk = lap(tk, &mut busy_s[usize::from(in_cx)]);
                    // ...and the crossing gates, on the exchanged copies.
                    for g in 0..n_cross {
                        let b = g * 4 * words;
                        let (xpart, zpart) = ex_s[b..b + 4 * words].split_at_mut(2 * words);
                        let (xc, xt) = xpart.split_at_mut(words);
                        let (zc, zt) = zpart.split_at_mut(words);
                        k_cx(xc, xt, zc, zt, rp);
                    }
                    lap(tk, &mut busy_s[2]);
                });
            }
        });
        if let Some(t0) = t {
            // Split the region's calling-thread wall in proportion to what the
            // threads were busy doing; spawn/join slack follows the same split.
            let region = t0.elapsed().as_nanos() as u64;
            let tot = busy
                .iter()
                .fold([0u64; 3], |a, b| [a[0] + b[0], a[1] + b[1], a[2] + b[2]]);
            let sum = tot[0] + tot[1] + tot[2];
            match ((region * tot[0]).checked_div(sum), (region * tot[1]).checked_div(sum)) {
                (Some(a), Some(b)) => {
                    self.prof.add_ns(Phase::Gate1q, a);
                    self.prof.add_ns(Phase::CxLocal, b);
                    self.prof.add_ns(Phase::CxCross, region - a - b);
                }
                _ => self.prof.add_ns(Phase::GateLayering, region),
            }
        }
        let t = self.prof.start();

        // ---- scatter the exchanged columns back ----
        for (k, &(c, t)) in cross.iter().enumerate() {
            let b = k * 4 * words;
            let (xc, xt) = (c * words, t * words);
            self.x[xc..xc + words].copy_from_slice(&self.exchange[b..b + words]);
            self.x[xt..xt + words].copy_from_slice(&self.exchange[b + words..b + 2 * words]);
            self.z[xc..xc + words].copy_from_slice(&self.exchange[b + 2 * words..b + 3 * words]);
            self.z[xt..xt + words].copy_from_slice(&self.exchange[b + 3 * words..b + 4 * words]);
        }

        let t = self.prof.stop(Phase::CxCross, t);

        // ---- the fold: XOR the partials in the DECLARED order, then clear ----
        for &si in &self.fold_order {
            for (dst, src) in self.r.iter_mut().zip(&self.rpart[si]) {
                *dst ^= *src;
            }
            for w in self.rpart[si].iter_mut() {
                *w = 0;
            }
        }
        self.prof.stop(Phase::GateLayering, t);
    }

    // ---- the column reads: every one of them is shard-local ----

    #[inline]
    fn x_col(&self, q: usize) -> &[u64] {
        &self.x[q * self.words..(q + 1) * self.words]
    }

    /// First row in `[lo, hi)` with X-support on `q` (`coltableau`'s scan).
    fn first_x_row_in(&self, q: usize, lo: usize, hi: usize) -> Option<usize> {
        if lo >= hi {
            return None;
        }
        let col = self.x_col(q);
        let (wlo, whi) = (lo >> 6, (hi - 1) >> 6);
        for (w, &cw) in col.iter().enumerate().take(whi + 1).skip(wlo) {
            let mut word = cw;
            if w == wlo {
                let b = lo & 63;
                if b != 0 {
                    word &= !0u64 << b;
                }
            }
            if w == whi {
                let b = hi & 63;
                if b != 0 {
                    word &= !(!0u64 << b);
                }
            }
            if word != 0 {
                return Some(w * 64 + word.trailing_zeros() as usize);
            }
        }
        None
    }

    fn x_rows_in(&self, q: usize, lo: usize, hi: usize, out: &mut Vec<usize>) {
        if lo >= hi {
            return;
        }
        let col = self.x_col(q);
        let (wlo, whi) = (lo >> 6, (hi - 1) >> 6);
        for (w, &cw) in col.iter().enumerate().take(whi + 1).skip(wlo) {
            let mut word = cw;
            if w == wlo {
                let b = lo & 63;
                if b != 0 {
                    word &= !0u64 << b;
                }
            }
            if w == whi {
                let b = hi & 63;
                if b != 0 {
                    word &= !(!0u64 << b);
                }
            }
            while word != 0 {
                out.push(w * 64 + word.trailing_zeros() as usize);
                word &= word - 1;
            }
        }
    }

    #[inline]
    fn sign_bit(&self, row: usize) -> bool {
        self.r[row >> 6] >> (row & 63) & 1 == 1
    }

    #[inline]
    fn xor_into_x_column(&mut self, q: usize, mask: &[u64]) {
        let w = self.words;
        let col = &mut self.x[q * w..(q + 1) * w];
        for (c, m) in col.iter_mut().zip(mask) {
            *c ^= *m;
        }
    }

    #[inline]
    fn flip_x_bit(&mut self, q: usize, row: usize) {
        self.x[q * self.words + (row >> 6)] ^= 1 << (row & 63);
    }

    // ---- the transposes ----

    /// Column-major → row-major. This direction does NOT decompose over the
    /// column cut: every shard would write different WORDS of the same rows,
    /// which is a partition of each row's allocation rather than of the
    /// tableau's, so by default it stays on the calling thread. It is rare by
    /// design — a batch that never needs a row never calls it — but the
    /// profile of 2026-09-23 measured it at 42–57 % of the flagship's wall at
    /// `d = 141`, the largest single phase, so `set_parallel_transpose(true)`
    /// (`--transpose-parallel`) re-aims it: the OUTPUT is cut by ROW BLOCKS
    /// (64 rows each, the transpose's own grain) into `S` contiguous runs,
    /// every thread reads the whole shared column tableau and writes only its
    /// own rows. Each output word is still produced by exactly the same
    /// 64×64 block transpose from exactly the same input words, so the result
    /// is the same bits whichever thread writes them; G1 is run with it on.
    fn store_to_packed(&self, out: &mut PackedTableau) {
        let s = self.cut.shards();
        if !self.par_store || s == 1 {
            store_rows(&self.x, &self.z, &self.r, self.n, self.words, &mut out.rows, 0);
            return;
        }
        let words = self.words;
        let (x, z, r, n) = (&self.x, &self.z, &self.r, self.n);
        // Row blocks `[i·words/S, (i+1)·words/S)`, 64 rows each: contiguous,
        // gap-free, and a pure function of (n, S).
        let mut rest: &mut [PauliRow] = &mut out.rows[..];
        let mut at = 0usize;
        std::thread::scope(|scope| {
            for i in 0..s {
                let rb_hi = (i + 1) * words / s;
                let row_hi = (rb_hi * 64).min(2 * n);
                let take = std::mem::take(&mut rest);
                let (mine, tail) = take.split_at_mut(row_hi - at);
                rest = tail;
                let row_lo = at;
                at = row_hi;
                if mine.is_empty() {
                    continue;
                }
                scope.spawn(move || store_rows(x, z, r, n, words, mine, row_lo));
            }
        });
    }

    /// Row-major → column-major, and THIS one is the chart's own direction:
    /// each shard writes only its own columns, reading the shared reference.
    fn load_from_packed(&mut self, p: &PackedTableau) {
        let n = self.n;
        let words = self.words;
        let nrows = 2 * n;
        let s = self.cut.shards();
        let bounds = self.cut.bounds.clone();
        let run = |xs: &mut [u64], zs: &mut [u64], qb_lo: usize, qb_hi: usize, col_lo: usize| {
            let mut bx = [0u64; 64];
            let mut bz = [0u64; 64];
            for qb0 in (qb_lo..qb_hi).step_by(TILE) {
                for rb0 in (0..words).step_by(TILE) {
                    for qb in qb0..qb_hi.min(qb0 + TILE) {
                        for rb in rb0..words.min(rb0 + TILE) {
                            let base = rb * 64;
                            for j in 0..64 {
                                let row = base + j;
                                if row < nrows {
                                    bx[63 - j] = p.rows[row].x.words[qb];
                                    bz[63 - j] = p.rows[row].z.words[qb];
                                } else {
                                    bx[63 - j] = 0;
                                    bz[63 - j] = 0;
                                }
                            }
                            transpose64(&mut bx);
                            transpose64(&mut bz);
                            for i in 0..64 {
                                let q = qb * 64 + i;
                                if q < n && q >= col_lo {
                                    xs[(q - col_lo) * words + rb] = bx[63 - i];
                                    zs[(q - col_lo) * words + rb] = bz[63 - i];
                                }
                            }
                        }
                    }
                }
            }
        };
        if s == 1 {
            let (x, z) = (&mut self.x, &mut self.z);
            run(x, z, 0, n.div_ceil(64), 0);
        } else {
            let xs = split_cols(&mut self.x, &bounds, words);
            let zs = split_cols(&mut self.z, &bounds, words);
            let run = &run;
            std::thread::scope(|scope| {
                for ((xs_s, zs_s), si) in xs.into_iter().zip(zs).zip(0..s) {
                    let (lo, hi) = (bounds[si], bounds[si + 1]);
                    scope.spawn(move || {
                        run(xs_s, zs_s, lo / 64, hi.div_ceil(64), lo);
                    });
                }
            });
        }
        for w in self.r.iter_mut() {
            *w = 0;
        }
        for (row, pr) in p.rows.iter().enumerate() {
            if pr.r % 4 == 2 {
                self.r[row >> 6] |= 1 << (row & 63);
            }
        }
    }

    /// Read-only peek at the state as the certified reference type.
    pub fn to_packed(&mut self) -> PackedTableau {
        self.flush();
        let mut out = PackedTableau::new(self.n);
        self.store_to_packed(&mut out);
        out
    }

    /// Set the state from a reference tableau (tests and profiling).
    pub fn load_state(&mut self, p: &PackedTableau) {
        assert!(!self.in_batch, "load_state inside an open batch");
        self.pending.clear();
        self.load_from_packed(p);
        self.packed_valid = false;
    }

    pub fn reference_allocated(&self) -> bool {
        self.packed.is_some()
    }

    /// Is the column engine's X plane currently a faithful mirror? (The
    /// mirror-patch invariant's own gate reads this.)
    pub fn mirror_x_valid(&self) -> bool {
        self.mirror_x_valid
    }

    pub fn packed_valid(&self) -> bool {
        self.packed_valid
    }

    pub fn packed_ref(&self) -> Option<&PackedTableau> {
        self.packed.as_ref()
    }

    /// Column `q` of the X plane, as a bitvector over all `2n` rows.
    pub fn x_column(&self, q: usize) -> &[u64] {
        self.x_col(q)
    }

    // ---- plant P1: the carrier and the conviction ----

    /// Flip one bit of one shard's columns — a corrupted shard, planted.
    pub fn plant_x_bit_flip(&mut self, column: usize, row: usize) {
        assert!(column < self.n && row < 2 * self.n);
        self.flush();
        self.flip_x_bit(column, row);
    }

    /// The identity check, with a VERDICT: the first column (scanned in
    /// column order, so the answer is a function of the state and not of a
    /// schedule) where this engine disagrees with the reference tableau, and
    /// the shard that owns it. `None` is bit-identity.
    pub fn first_divergence(&mut self, reference: &PackedTableau) -> Option<Divergence> {
        self.flush();
        assert_eq!(self.n, reference.n, "identity check across different sizes");
        for q in 0..self.n {
            for row in 0..2 * self.n {
                let mine_x = self.x_col(q)[row >> 6] >> (row & 63) & 1 == 1;
                if mine_x != reference.rows[row].x.get(q) {
                    return Some(Divergence {
                        shard: Some(self.cut.shard_of(q)),
                        column: Some(q),
                        row,
                        plane: "X",
                    });
                }
                let mine_z = self.z[q * self.words + (row >> 6)] >> (row & 63) & 1 == 1;
                if mine_z != reference.rows[row].z.get(q) {
                    return Some(Divergence {
                        shard: Some(self.cut.shard_of(q)),
                        column: Some(q),
                        row,
                        plane: "Z",
                    });
                }
            }
        }
        for row in 0..2 * self.n {
            if (self.sign_bit(row) as u8) * 2 != reference.rows[row].r % 4 {
                return Some(Divergence { shard: None, column: None, row, plane: "sign" });
            }
        }
        None
    }

    // ---- the measurement phase ----

    pub fn begin_batch(&mut self) {
        assert!(!self.in_batch, "batch already open");
        self.flush();
        self.in_batch = true;
        self.mirror_x_valid = true;
        self.mirror_full_valid = true;
        self.dirty = false;
    }

    pub fn end_batch(&mut self) {
        assert!(self.in_batch, "no batch open");
        if self.dirty {
            let packed = self.packed.take().expect("dirty without a reference");
            let t = self.prof.start();
            self.load_from_packed(&packed);
            self.prof.stop(Phase::TransposeR2C, t);
            self.packed = Some(packed);
            self.mirror_x_valid = true;
            self.mirror_full_valid = true;
            self.dirty = false;
        }
        self.in_batch = false;
    }

    fn ensure_packed(&mut self) {
        if self.packed_valid {
            return;
        }
        debug_assert!(!self.dirty, "reference stale while it is authoritative");
        let n = self.n;
        let t = self.prof.start();
        if self.packed.is_none() {
            self.packed = Some(PackedTableau::new(n));
        }
        let mut buf = self.packed.take().expect("just ensured");
        let t = self.prof.stop(Phase::ReferenceAlloc, t);
        self.store_to_packed(&mut buf);
        self.prof.stop(Phase::TransposeC2R, t);
        self.packed = Some(buf);
        self.packed_valid = true;
        self.stats.transposes += 1;
    }

    /// ONE declared stream, consumed in circuit order — independent of `S`.
    fn next_bit(&mut self) -> bool {
        if self.scramble_stream {
            // P3, deliberately wrong: the pair is consumed backwards.
            if let Some(b) = self.stash.take() {
                return b;
            }
            let first = splitmix(&mut self.rng);
            let second = splitmix(&mut self.rng);
            self.stash = Some(first);
            return second;
        }
        splitmix(&mut self.rng)
    }

    /// Measure qubit `q` in the computational basis, inside an open batch.
    /// Returns `(outcome, was_deterministic)` — `ColAdaptive::measure`'s
    /// contract, and its arithmetic, with the rowsum cut across shards.
    pub fn measure(&mut self, q: usize) -> (bool, bool) {
        assert!(self.in_batch, "measure outside a batch");
        let n = self.n;

        // ---- the determinism question: a read of column q, shard-local ----
        let t_scan = self.prof.start();
        let pivot = if self.mirror_x_valid {
            self.stats.scan_fast += 1;
            self.first_x_row_in(q, n, 2 * n)
        } else {
            self.stats.scan_fallback += 1;
            let packed = self.packed.as_ref().expect("fallback without a reference");
            (n..2 * n).find(|&p| packed.rows[p].x.get(q))
        };

        // ---- the O(1) case: one sign bit, no row, no transpose, no mesh ----
        if pivot.is_none() && self.mirror_full_valid {
            let mut hits = Vec::new();
            self.x_rows_in(q, 0, n, &mut hits);
            if hits.len() <= 1 {
                self.prof.stop(Phase::Scan, t_scan);
                self.stats.deterministic += 1;
                self.stats.product_terms += hits.len() as u64;
                self.stats.single_term += 1;
                let out = match hits.first() {
                    None => false,
                    Some(&i) => self.sign_bit(i + n),
                };
                return (out, true);
            }
        }

        self.prof.stop(Phase::Scan, t_scan);
        self.ensure_packed();

        match pivot {
            // ---- RANDOM: the coin, then the rowsum cascade across shards ----
            Some(p) => {
                let t = self.prof.start();
                let outcome = self.next_bit();
                let t_setup = self.prof.stop(Phase::Rng, t);
                let mut packed = self.packed.take().expect("reference materialized");
                let pivot_row = packed.rows[p].clone();
                let pw = pivot_row.x.popcount() as u64;
                self.stats.pivot_weight += pw;
                self.stats.pivot_weight_max = self.stats.pivot_weight_max.max(pw);

                // The update mask: column q over all 2n rows, minus the pivot.
                let mask: Option<Vec<u64>> = if self.mirror_x_valid {
                    let mut m = self.x_col(q).to_vec();
                    m[p >> 6] &= !(1u64 << (p & 63));
                    Some(m)
                } else {
                    None
                };

                let mut idxs: Vec<usize> = Vec::new();
                match &mask {
                    Some(m) => {
                        for (w, &word) in m.iter().enumerate() {
                            let mut bits = word;
                            while bits != 0 {
                                idxs.push(w * 64 + bits.trailing_zeros() as usize);
                                bits &= bits - 1;
                            }
                        }
                    }
                    None => {
                        for i in 0..2 * n {
                            if i != p && packed.rows[i].x.get(q) {
                                idxs.push(i);
                            }
                        }
                    }
                }
                self.stats.cascade_terms += idxs.len() as u64;
                self.mesh.rowsum_rows += idxs.len() as u64;
                self.prof.stop(Phase::RowsumSerial, t_setup);
                self.rowsum(&mut packed, &idxs, &pivot_row);
                let t = self.prof.start();

                let old_destab_x = packed.rows[p - n].x.clone();
                packed.rows[p - n] = pivot_row.clone();
                let mut fresh = PauliRow::identity(n);
                fresh.z.set(q, true);
                fresh.r = if outcome { 2 } else { 0 };
                packed.rows[p] = fresh;
                self.packed = Some(packed);

                // ---- patch the X mirror, or give up on it honestly ----
                // (`coladaptive.rs`'s derivation, unchanged: the patch is
                // bounded by the pivot's X-weight plus the old
                // destabilizer's, and every touched column is by definition
                // inside exactly one shard.)
                const PATCH_BUDGET: u32 = 256;
                let mut flip2 = old_destab_x;
                flip2.xor_assign(&pivot_row.x);
                let patch_cost = pivot_row.x.popcount() * 2 + flip2.popcount();
                let t = self.prof.stop(Phase::RowsumSerial, t);

                match mask {
                    Some(m) if patch_cost <= PATCH_BUDGET => {
                        for (w, &word) in pivot_row.x.words.iter().enumerate() {
                            let mut bits = word;
                            while bits != 0 {
                                let c = w * 64 + bits.trailing_zeros() as usize;
                                bits &= bits - 1;
                                self.xor_into_x_column(c, &m);
                                self.flip_x_bit(c, p);
                            }
                        }
                        for (w, &word) in flip2.words.iter().enumerate() {
                            let mut bits = word;
                            while bits != 0 {
                                let c = w * 64 + bits.trailing_zeros() as usize;
                                bits &= bits - 1;
                                self.flip_x_bit(c, p - n);
                            }
                        }
                        self.stats.mirror_patched += 1;
                        self.mirror_x_valid = true;
                    }
                    _ => {
                        self.mirror_x_valid = false;
                        self.stats.mirror_dropped += 1;
                    }
                }
                self.prof.stop(Phase::MirrorPatch, t);
                self.mirror_full_valid = false;
                self.dirty = true;
                self.stats.random += 1;
                (outcome, false)
            }
            // ---- DETERMINISTIC: read-only, so the mirror survives ----
            None => {
                let t = self.prof.start();
                let packed = self.packed.as_ref().expect("reference materialized");
                let mut scratch = PauliRow::identity(n);
                let mut terms = 0u64;
                if self.mirror_x_valid {
                    let mut hits = Vec::new();
                    self.x_rows_in(q, 0, n, &mut hits);
                    for i in hits {
                        scratch.mul_assign(&packed.rows[i + n]);
                        terms += 1;
                    }
                } else {
                    for i in 0..n {
                        if packed.rows[i].x.get(q) {
                            scratch.mul_assign(&packed.rows[i + n]);
                            terms += 1;
                        }
                    }
                }
                self.stats.deterministic += 1;
                self.stats.product_terms += terms;
                self.prof.stop(Phase::DetProduct, t);
                (scratch.r % 4 == 2, true)
            }
        }
    }

    /// THE ROWSUM, cut across the chart: `rows[i] *= pivot` for every `i` in
    /// `idxs`, with each shard doing its own column range and the phase
    /// folded over shards IN SHARD ORDER.
    ///
    /// Exactness, which is the whole point: `PauliRow::mul_assign`'s phase is
    /// `g = (Σ_w plus_w − Σ_w minus_w) mod 4` where the sums run over the
    /// row's words, and the planes update by per-word XOR. Both decompose
    /// over any partition of the words, and the chart's boundaries are
    /// word-aligned, so a shard's partial is a partial of exactly that sum.
    /// The folded result is not an approximation of the sequential one; it is
    /// the same integer.
    fn rowsum(&mut self, packed: &mut PackedTableau, idxs: &[usize], pivot_row: &PauliRow) {
        let s = self.cut.shards();
        let rw = self.n.div_ceil(64);
        if idxs.is_empty() {
            return;
        }
        if s == 1 || idxs.len() * rw < self.par_min_work {
            self.mesh.rowsums_serial += 1;
            let t = self.prof.start();
            for &i in idxs {
                packed.rows[i].mul_assign(pivot_row);
            }
            self.prof.stop(Phase::RowsumSerial, t);
            return;
        }
        self.mesh.rowsums_parallel += 1;
        let t = self.prof.start();

        // Disjoint `&mut` to each updated row, then each row's words cut at
        // the chart's boundaries: S slices per row, none overlapping.
        let mut per_shard: Vec<Vec<(&mut [u64], &mut [u64])>> =
            (0..s).map(|_| Vec::with_capacity(idxs.len())).collect();
        {
            let mut rest: &mut [PauliRow] = &mut packed.rows[..];
            let mut prev = 0usize;
            for &i in idxs {
                let take = std::mem::take(&mut rest);
                let (_skip, tail) = take.split_at_mut(i - prev);
                let (row, tail2) = tail.split_first_mut().expect("row exists");
                rest = tail2;
                prev = i + 1;
                let mut xrest: &mut [u64] = &mut row.x.words[..];
                let mut zrest: &mut [u64] = &mut row.z.words[..];
                let mut at = 0usize;
                for (si, shard) in per_shard.iter_mut().enumerate() {
                    let hi = self.cut.row_words(si).end;
                    let takex = std::mem::take(&mut xrest);
                    let (xa, xb) = takex.split_at_mut(hi - at);
                    let takez = std::mem::take(&mut zrest);
                    let (za, zb) = takez.split_at_mut(hi - at);
                    shard.push((xa, za));
                    xrest = xb;
                    zrest = zb;
                    at = hi;
                }
            }
        }

        let mut partials: Vec<Vec<(u32, u32)>> = vec![Vec::new(); s];
        let word_ranges: Vec<std::ops::Range<usize>> =
            (0..s).map(|si| self.cut.row_words(si)).collect();
        let px = &pivot_row.x.words;
        let pz = &pivot_row.z.words;
        std::thread::scope(|scope| {
            for ((rows, out), wr) in per_shard
                .into_iter()
                .zip(partials.iter_mut())
                .zip(word_ranges)
            {
                scope.spawn(move || {
                    let (pxs, pzs) = (&px[wr.clone()], &pz[wr]);
                    out.reserve(rows.len());
                    for (rx, rz) in rows {
                        let mut plus = 0u32;
                        let mut minus = 0u32;
                        for i in 0..pxs.len() {
                            let (x1, z1) = (pxs[i], pzs[i]);
                            let (x2, z2) = (rx[i], rz[i]);
                            let p = (!x1 & z1 & x2 & !z2)
                                | (x1 & z1 & !x2 & z2)
                                | (x1 & !z1 & x2 & z2);
                            let m = (!x1 & z1 & x2 & z2)
                                | (x1 & z1 & x2 & !z2)
                                | (x1 & !z1 & !x2 & z2);
                            plus += p.count_ones();
                            minus += m.count_ones();
                            rx[i] = x2 ^ x1;
                            rz[i] = z2 ^ z1;
                        }
                        out.push((plus, minus));
                    }
                });
            }
        });

        let t = self.prof.stop(Phase::RowsumPartial, t);
        // ---- the fold, in the DECLARED shard order, on the parent ----
        for (k, &i) in idxs.iter().enumerate() {
            let (mut plus, mut minus) = (0u64, 0u64);
            for &si in &self.fold_order {
                let (p, m) = partials[si][k];
                plus += p as u64;
                minus += m as u64;
            }
            let g = (plus as i64 - minus as i64).rem_euclid(4) as u8;
            let row = &mut packed.rows[i];
            row.r = (row.r + pivot_row.r + g) % 4;
        }
        self.prof.stop(Phase::RowsumFold, t);
    }

    /// The value of the Pauli-Z STRING `∏_q Z_q`, if the state determines it
    /// (`ColAdaptive::z_string_value`, column-side and allocation-free in the
    /// common case).
    pub fn z_string_value(&mut self, qubits: &[usize]) -> Option<bool> {
        assert!(!self.in_batch, "z_string_value inside an open batch");
        self.flush();
        let n = self.n;
        let words = self.words;

        let mut mask = vec![0u64; words];
        for &q in qubits {
            let col = self.x_col(q);
            for (m, c) in mask.iter_mut().zip(col) {
                *m ^= *c;
            }
        }
        for row in n..2 * n {
            if mask[row >> 6] >> (row & 63) & 1 == 1 {
                return None;
            }
        }
        let mut hits = Vec::new();
        for (w, &mw) in mask.iter().enumerate().take(((n.saturating_sub(1)) >> 6) + 1) {
            let mut bits = mw;
            if w == (n - 1) >> 6 {
                let b = n & 63;
                if b != 0 {
                    bits &= !(!0u64 << b);
                }
            }
            while bits != 0 {
                hits.push(w * 64 + bits.trailing_zeros() as usize);
                bits &= bits - 1;
            }
        }
        match hits.len() {
            0 => return Some(false),
            1 => return Some(self.sign_bit(hits[0] + n)),
            _ => {}
        }
        if !self.packed_valid {
            let t = self.prof.start();
            if self.packed.is_none() {
                self.packed = Some(PackedTableau::new(n));
            }
            let mut buf = self.packed.take().expect("just ensured");
            let t = self.prof.stop(Phase::ReferenceAlloc, t);
            self.store_to_packed(&mut buf);
            self.prof.stop(Phase::TransposeC2R, t);
            self.packed = Some(buf);
            self.packed_valid = true;
            self.stats.transposes += 1;
        }
        let packed = self.packed.as_ref().expect("materialized");
        let mut scratch = PauliRow::identity(n);
        for i in hits {
            scratch.mul_assign(&packed.rows[i + n]);
        }
        Some(scratch.r % 4 == 2)
    }
}

/// Column-major → row-major for the rows `[row_lo, row_lo + rows.len())`
/// (`row_lo` a multiple of 64): `coltableau::store_to_packed`'s blocked nest,
/// restricted to the row blocks this slice owns. Called on the whole range it
/// IS the serial transpose; called per row block it is the re-aimed one.
fn store_rows(
    x: &[u64],
    z: &[u64],
    r: &[u64],
    n: usize,
    words: usize,
    rows: &mut [PauliRow],
    row_lo: usize,
) {
    debug_assert_eq!(row_lo % 64, 0);
    let nq_words = n.div_ceil(64);
    let row_hi = row_lo + rows.len();
    let (rb_lo, rb_hi) = (row_lo / 64, row_hi.div_ceil(64));
    let mut bx = [0u64; 64];
    let mut bz = [0u64; 64];
    for qb0 in (0..nq_words).step_by(TILE) {
        for rb0 in (rb_lo..rb_hi).step_by(TILE) {
            for qb in qb0..nq_words.min(qb0 + TILE) {
                for rb in rb0..rb_hi.min(rb0 + TILE) {
                    for i in 0..64 {
                        let q = qb * 64 + i;
                        if q < n {
                            bx[63 - i] = x[q * words + rb];
                            bz[63 - i] = z[q * words + rb];
                        } else {
                            bx[63 - i] = 0;
                            bz[63 - i] = 0;
                        }
                    }
                    transpose64(&mut bx);
                    transpose64(&mut bz);
                    let base = rb * 64;
                    for j in 0..64 {
                        let row = base + j;
                        if row < row_hi {
                            rows[row - row_lo].x.words[qb] = bx[63 - j];
                            rows[row - row_lo].z.words[qb] = bz[63 - j];
                        }
                    }
                }
            }
        }
    }
    for (k, pr) in rows.iter_mut().enumerate() {
        let row = row_lo + k;
        pr.r = ((r[row >> 6] >> (row & 63) & 1) as u8) * 2;
    }
}

/// Blocking factor for the transpose loop nests (`coltableau`'s TILE).
const TILE: usize = 8;

/// In-register 64×64 bit-matrix transpose (Hacker's Delight §7-3).
fn transpose64(a: &mut [u64; 64]) {
    let mut j = 32usize;
    let mut m: u64 = 0x0000_0000_FFFF_FFFF;
    while j != 0 {
        let mut k = 0usize;
        while k < 64 {
            let t = (a[k] ^ (a[k + j] >> j)) & m;
            a[k] ^= t;
            a[k + j] ^= t << j;
            k = (k + j + 1) & !j;
        }
        j >>= 1;
        m ^= m << j;
    }
}

/// The prereg's G2 number for a circuit that has not been run yet: how many
/// of these CX pairs cross a shard boundary under `cut`, and how many there
/// are. The engine counts the same thing while it runs (`MeshStats`), and the
/// two must agree — a cut whose crossing count depends on the execution would
/// not be a cut.
pub fn crossing_count(
    cut: &ShardCut,
    pairs: impl IntoIterator<Item = (usize, usize)>,
) -> (u64, u64) {
    let mut crossing = 0u64;
    let mut total = 0u64;
    for (c, t) in pairs {
        total += 1;
        if cut.shard_of(c) != cut.shard_of(t) {
            crossing += 1;
        }
    }
    (crossing, total)
}

/// A stable digest of a measurement record — every outcome bit in circuit
/// order, and nothing else. The harness compares it across `S`, so it must
/// not see `S`, a timing, or a thread count: FNV-1a over the bits, one byte
/// per bit, so the LENGTH of the record is part of the digest too.
pub fn record_hash(bits: &[bool]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bits {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h ^= bits.len() as u64;
    h.wrapping_mul(0x0000_0100_0000_01b3)
}

#[cfg(test)]
mod cut_tests {
    use super::*;

    #[test]
    fn the_chart_is_contiguous_gap_free_and_word_snapped() {
        for n in [1usize, 7, 64, 65, 255, 4049, 39761] {
            for s in [1usize, 2, 3, 4, 8, 16] {
                let cut = ShardCut::new(n, s);
                assert_eq!(cut.bounds[0], 0);
                assert_eq!(*cut.bounds.last().unwrap(), n);
                assert!(cut.shards() <= s, "n={n} s={s}: more shards than asked for");
                assert!(cut.shards() >= 1);
                for w in cut.bounds.windows(2) {
                    assert!(w[0] < w[1], "n={n} s={s}: empty or descending range");
                }
                for &b in &cut.bounds[1..cut.bounds.len() - 1] {
                    assert_eq!(b % 64, 0, "n={n} s={s}: boundary {b} is not word-snapped");
                }
                for q in 0..n {
                    let si = cut.shard_of(q);
                    assert!(cut.range(si).contains(&q), "n={n} s={s}: q={q} misfiled");
                }
                // Row-word ranges partition the qubit axis exactly.
                let mut at = 0usize;
                for si in 0..cut.shards() {
                    let r = cut.row_words(si);
                    assert_eq!(r.start, at, "n={n} s={s}: row-word gap at shard {si}");
                    at = r.end;
                }
                assert_eq!(at, n.div_ceil(64));
            }
        }
    }
}
