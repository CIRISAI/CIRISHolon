//! THE AFFINE ENGINE — one port, every consumer.
//!
//! A stabilizer branch in affine (phase-polynomial) form:
//!
//! ```text
//! amplitude(x) = γ · i^{Σ d_a u_a} · (−1)^{Σ_{a<b} J_ab u_a u_b}   on x = R u ⊕ h
//! ```
//!
//! with `R`'s columns INDEPENDENT (so `u ↔ x` is a bijection on the support —
//! every query below needs it, and [`Affine::amplitude`] says so loudly) and
//! `γ` exact in `Z[ω]·2^{−m/2}` ([`crate::ledger::Cyc`]). No floating point
//! enters the exact path; `Cyc::to_complex` is the only exit. The form is
//! Dehaene–De Moor 2003 and Van den Nest 2010 — credited, not ours.
//!
//! # Why this file exists
//!
//! `holon-qasm::magic::Affine` is the certified reference (QASM-2 record, five
//! of five against qiskit), and `holon-qasm` is a DEV-dependency of this crate:
//! `src/` cannot see it, and the isolation gate that keeps it that way is
//! load-bearing (workspace manifest). So the engine has to be transplanted. It
//! was transplanted THREE TIMES, once per lane — pruning, sampling, and the
//! BG-decomposed magic tier — because the lanes ran in parallel and each
//! needed one thing the others did not:
//!
//! | lane | what it added |
//! |---|---|
//! | `prune` | the canonical form: [`Affine::canonicalize`], [`Affine::canon_key`], [`Affine::fingerprint`] |
//! | `sample` | pairwise [`overlap`] and basis-state [`Affine::project`] |
//! | `magic` | [`Affine::attach`], the block-decomposition loader |
//!
//! This module is the UNION of the three, and it is the only copy. The lanes
//! re-export it under the names they were written against, so their call sites
//! and their referees read unchanged.
//!
//! The three copies were diffed method by method before they were merged, and
//! they agreed on every value-producing line. What differed was gauge plumbing
//! (which lane carried which planted mutation), instrumentation ([`GaussStats`]
//! is the sampler's coverage meter), assertion strength, and spelling. Those
//! are reconciled here, each at its site.
//!
//! # The campaign-bought subtleties, kept with their provenance
//!
//! Three lines in this file are not obvious and were each paid for by a
//! measured failure upstream. They carry their receipts:
//!
//! * the odd-δ Gauss sum's XOR expansion — pairwise `J` flips across the
//!   neighbour set, not just per-variable `d` bumps (measured: a sign error on
//!   `amp(1,1)` of `h,cx,h,s,h`, the entangled HSH sandwich);
//! * the column-dependence repair after a row clear (measured: err 0.375 on
//!   `h,h,h,h,cx,h`), and the LOUD rank assertion in [`Affine::amplitude`]
//!   that refuses to answer if the repair ever fails;
//! * the one-shot denominator alignment inside `Cyc::add` (measured: a hang at
//!   Δ ≥ 3 when a loop of √2-multiplies fought `normalize`) — that one lives
//!   in [`crate::ledger`], where the ring does.
//!
//! # One merge law
//!
//! Every accumulation here is `crate::merge::MergeLedger` on `Cyc` — the
//! branch fold behind [`overlap_bruteforce`], the odd-δ Gauss sum's
//! `Σ_w i^{δw}`, and the credit-against-debit posting that [`cyc_eq`] uses to
//! decide exact equality. There is no second addition path in this file.
//!
//! # The layout, and what it is allowed to change
//!
//! `R`, `J` and `h` are FLAT bit-packed buffers (`BitMat`, [`crate::plane::BitPlane`]) —
//! the layout tier 1 already runs on — not the reference's `Vec<Vec<bool>>`.
//! One contiguous allocation each, `stride` words per row, so a row operation
//! is a stride-1 word loop and the per-bit pointer chase is gone. The
//! mathematics did not move: every gate, the Gauss sum, the canonical form and
//! the amplitude are the same operations on the same values, and the
//! conformance gates against `holon-qasm::magic` (`tests/pipeline.rs`,
//! `tests/prune.rs`, `tests/sample.rs`) are what say so rather than a claim
//! here. Two algorithmic moves came with it, each a THEOREM about the same
//! answer, never an approximation:
//!
//! * [`Affine::amplitudes_agree`] asks one pair of states for `k + n`
//!   amplitudes in a row. It now factors the elimination once per state
//!   (`ColSolve`) instead of re-running it per probe — sound because `R u = b`
//!   has a UNIQUE solution when `R`'s columns are independent, which the
//!   affine invariant already requires and this code still asserts.
//! * `Affine::dependent_subset` answers `None` whenever some row's support is
//!   exactly `{a}`, without eliminating: every combination of the other columns
//!   reads zero at that row and column a reads one. A proof of the same answer.
//!
//! The reference's indexing idioms (`for c in 0..k` over parallel `Vec`s) are
//! kept wherever the layout did not force a change, so a diff against
//! `holon-qasm::magic` still shows the intended differences and nothing else.
//!
//! Zero runtime dependencies (`std` only).

use crate::ledger::Cyc;
use crate::merge::MergeLedger;
use crate::plane::BitPlane;

// ------------------------------------------------------------------ ring helpers
//
// `ledger::Cyc` carries mul/add/to_complex. The powers of ω and i, and an
// equality that is honest across denominators, live here rather than in the
// ledger because they are the engine's business, not the ring's.

/// `i^k` as a ring element (`i = ω²`).
pub fn i_pow(k: u8) -> Cyc {
    let mut c = [0i128; 4];
    match k % 4 {
        0 => c[0] = 1,
        1 => c[2] = 1,
        2 => c[0] = -1,
        _ => c[2] = -1,
    }
    Cyc { c, m: 0 }
}

/// `ω^k`, `ω = e^{iπ/4}`, `ω⁴ = −1`.
pub fn omega_pow(k: u8) -> Cyc {
    let mut c = [0i128; 4];
    let k = k % 8;
    if k < 4 {
        c[k as usize] = 1;
    } else {
        c[(k - 4) as usize] = -1;
    }
    Cyc { c, m: 0 }
}

pub fn cyc_is_zero(a: Cyc) -> bool {
    a.c.iter().all(|&x| x == 0)
}

pub fn cyc_neg(a: Cyc) -> Cyc {
    Cyc { c: [-a.c[0], -a.c[1], -a.c[2], -a.c[3]], m: a.m }
}

/// Complex conjugate: `ω̄ = ω⁻¹ = −ω³`, so `[c0,c1,c2,c3] ↦ [c0,−c3,−c2,−c1]`.
pub fn cyc_conj(a: Cyc) -> Cyc {
    Cyc { c: [a.c[0], -a.c[3], -a.c[2], -a.c[1]], m: a.m }
}

pub fn cyc_sub(a: Cyc, b: Cyc) -> Cyc {
    a.merge(cyc_neg(b))
}

/// EXACT equality ACROSS denominators. `Cyc`'s derived `PartialEq` is NOT a
/// value equality: `√2` has two fixed points of `normalize()`
/// (`{c:[2,0,0,0], m:1}` and `{c:[0,1,0,-1], m:0}`) and the derive calls them
/// different. `{1,ω,ω²,ω³}` is a Z-basis, so at a COMMON `m` the coefficient
/// vector is unique — and posting the credit against the debit goes through
/// the ledger's one-shot denominator alignment, which is faithful.
pub fn cyc_eq(a: Cyc, b: Cyc) -> bool {
    cyc_is_zero(cyc_sub(a, b))
}

// ------------------------------------------------------------------ gates

/// The magic-tier gate set. Declared here rather than imported because
/// `holon-qasm` is a DEV-dependency of this crate: `src/` cannot see it, and
/// the isolation gate that keeps it that way is load-bearing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gate {
    X(usize),
    Z(usize),
    S(usize),
    Sdg(usize),
    H(usize),
    Cx(usize, usize),
    T(usize),
    Tdg(usize),
}

impl Gate {
    pub fn is_t(&self) -> bool {
        matches!(self, Gate::T(_) | Gate::Tdg(_))
    }
}

/// The CLIFFORD-only gate set — the same alphabet with `T`/`T†` removed, so
/// that a consumer which branches on `T` itself (the sampler does) cannot be
/// handed one by accident. `Gate` is the superset; the conversion is total in
/// the direction that is total.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Clif {
    X(usize),
    Z(usize),
    S(usize),
    Sdg(usize),
    H(usize),
    Cx(usize, usize),
}

impl From<Clif> for Gate {
    fn from(c: Clif) -> Gate {
        match c {
            Clif::X(q) => Gate::X(q),
            Clif::Z(q) => Gate::Z(q),
            Clif::S(q) => Gate::S(q),
            Clif::Sdg(q) => Gate::Sdg(q),
            Clif::H(q) => Gate::H(q),
            Clif::Cx(c, t) => Gate::Cx(c, t),
        }
    }
}

// ------------------------------------------------------------------ mutations

/// Planted defects, so a conformance harness can prove it would catch a wrong
/// implementation (the `Mutation` discipline of `holon-qasm`). Default = clean.
///
/// This is the UNION of the three lanes' gauges. A lane sets only the flags it
/// gauges and leaves the rest at `false`, which is exactly the clean engine.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mutations {
    /// Drop the `S` gate's pairwise `J` flips.
    pub drop_s_cross: bool,
    /// Use the wrong odd-δ Gauss-sum phase: `i^δ` instead of `1 + i^δ`, which
    /// drops the `1+` structure. The classic phase slip — it leaves every
    /// magnitude plausible, which is why it is gauged rather than trusted.
    pub wrong_gauss: bool,
    /// [`Affine::flip`] drops the `γ ·= i^{d_p}` factor: the extracted global
    /// scalar is wrong, so a merged weight uses a WRONG PHASE RATIO. Invisible
    /// to the canonical-form check and to the amplitude cross-check (both
    /// states are genuinely equal); only conformance against the naive sum can
    /// see it.
    pub flip_drops_gamma: bool,
    /// [`Affine::canon_key`] omits `h`, so states on DIFFERENT cosets look
    /// equal. The amplitude cross-check is the only remaining guard and must
    /// fire — this is the "never trust the hash alone" defect.
    pub key_ignores_h: bool,
    /// The duplicate-branch merge adds `w₂ · i` instead of `w₂`: a wrong phase
    /// ratio applied at the merge itself, so it fires only on circuits where a
    /// merge happens.
    ///
    /// Read by the DRIVER (`prune::merge_block`), never by the engine — see
    /// the note on `skip_verify`.
    pub merge_phase: bool,
    /// Skip the exact amplitude cross-check on a key match. Only useful paired
    /// with `key_ignores_h`, to let the bad merge through to conformance.
    ///
    /// Read by the DRIVER (`prune::merge_block`), never by the engine. The two
    /// driver flags ride on the state's `Mutations` so that ONE value describes
    /// one gauged run end to end, which is what the gauge matrix reports on.
    pub skip_verify: bool,
}

/// Coverage meter for the Gauss-sum branches.
///
/// "The planted mutation fired" is evidence about the odd-δ Gauss sum only if
/// the odd-δ branch was actually REACHED, and most random state pairs never
/// reach it — they are annihilated by an inconsistent constraint first. So the
/// gauge counts coverage rather than trusting it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GaussStats {
    /// `Σ_{u_a}` with δ odd — the `(1+i^δ)` prefactor, where the mutation lives.
    pub odd_steps: u64,
    /// `Σ_{u_a}` with δ even — a parity CONSTRAINT rather than a phase.
    pub even_steps: u64,
    /// Sums that collapsed to zero (an unsatisfiable parity constraint).
    pub annihilated: u64,
    /// Overlaps that never reached a Gauss sum: disjoint affine subspaces.
    pub inconsistent: u64,
}

// --------------------------------------------------------------- bit matrix
//
// The layout tier 1 already runs on, brought to the affine engine: ONE flat
// contiguous buffer instead of a `Vec<Vec<bool>>`. A `Vec<Vec<bool>>` costs a
// pointer chase and a bounds check per BIT; this costs one bounds check per
// ROW and moves 64 bits per instruction.

/// A row-major bit matrix, `rows × cols`, laid out flat: row `r` occupies
/// `w[r*stride .. r*stride+stride]`, and `stride*64 ≥ cols`. Bits at index
/// `≥ cols` are INVARIANTLY zero — that is what lets every row loop run to
/// `stride` with no masking, and what makes `xor_rows` a plain stride-1 word
/// loop. Column operations are strided bit walks over the same buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
struct BitMat {
    w: Vec<u64>,
    rows: usize,
    cols: usize,
    stride: usize,
}

#[inline(always)]
fn low_mask(bits: usize) -> u64 {
    if bits >= 64 {
        !0u64
    } else if bits == 0 {
        0
    } else {
        (1u64 << bits) - 1
    }
}

/// Slide a packed row's bits down past the deleted column (`wc` is its word,
/// `keep` masks the bits below it) — the "delete one bit column" primitive.
#[inline(always)]
fn drop_bit(row: &mut [u64], wc: usize, keep: u64) {
    let last = row.len() - 1;
    let x = row[wc];
    let nxt = if wc < last { row[wc + 1] & 1 } else { 0 };
    row[wc] = (x & keep) | ((x >> 1) & !keep) | (nxt << 63);
    for i in wc + 1..last {
        row[i] = (row[i] >> 1) | ((row[i + 1] & 1) << 63);
    }
    if wc < last {
        row[last] >>= 1;
    }
}

/// The same slide, writing into a different row — one pass that deletes the
/// bit AND compacts the row upward.
#[inline(always)]
fn drop_bit_into(dst: &mut [u64], src: &[u64], wc: usize, keep: u64) {
    let last = src.len() - 1;
    dst[..wc].copy_from_slice(&src[..wc]);
    let x = src[wc];
    let nxt = if wc < last { src[wc + 1] & 1 } else { 0 };
    dst[wc] = (x & keep) | ((x >> 1) & !keep) | (nxt << 63);
    for i in wc + 1..last {
        dst[i] = (src[i] >> 1) | ((src[i + 1] & 1) << 63);
    }
    if wc < last {
        dst[last] = src[last] >> 1;
    }
}

impl BitMat {
    fn new(rows: usize, cols: usize) -> Self {
        let stride = cols.div_ceil(64);
        BitMat { w: vec![0; rows * stride], rows, cols, stride }
    }

    #[inline(always)]
    fn row(&self, r: usize) -> &[u64] {
        &self.w[r * self.stride..r * self.stride + self.stride]
    }

    #[inline(always)]
    fn row_mut(&mut self, r: usize) -> &mut [u64] {
        let s = self.stride;
        &mut self.w[r * s..r * s + s]
    }

    #[inline(always)]
    fn get(&self, r: usize, c: usize) -> bool {
        self.w[r * self.stride + (c >> 6)] >> (c & 63) & 1 == 1
    }

    #[inline(always)]
    fn set(&mut self, r: usize, c: usize, v: bool) {
        let i = r * self.stride + (c >> 6);
        let b = 1u64 << (c & 63);
        if v {
            self.w[i] |= b;
        } else {
            self.w[i] &= !b;
        }
    }

    #[inline(always)]
    fn toggle(&mut self, r: usize, c: usize) {
        self.w[r * self.stride + (c >> 6)] ^= 1u64 << (c & 63);
    }

    /// `row_dst ^= row_src` — the stride-1 word loop the whole engine's
    /// elimination work funnels through.
    #[inline]
    fn xor_rows(&mut self, dst: usize, src: usize) {
        let s = self.stride;
        let (a, b) = (dst * s, src * s);
        for i in 0..s {
            self.w[a + i] ^= self.w[b + i];
        }
    }

    /// `row_dst ^= mask`, then force the diagonal bit back to zero. The one
    /// move behind every "flip `J_ab` across a set" step: doing it for every
    /// member of the set toggles each unordered pair exactly once per side.
    #[inline]
    fn xor_row_with(&mut self, dst: usize, mask: &[u64], clear: usize) {
        let s = self.stride;
        let a = dst * s;
        for i in 0..s {
            self.w[a + i] ^= mask[i];
        }
        self.w[a + (clear >> 6)] &= !(1u64 << (clear & 63));
    }

    /// `col_dst ^= col_src` over every row — the F₂ elementary column
    /// operation, as one branchless pass over the flat buffer with the word
    /// and bit offsets hoisted out of the loop.
    fn xor_col(&mut self, dst: usize, src: usize) {
        let s = self.stride;
        let (ws, bs) = (src >> 6, src & 63);
        let (wd, bd) = (dst >> 6, dst & 63);
        for row in self.w.chunks_exact_mut(s) {
            let bit = (row[ws] >> bs) & 1;
            row[wd] ^= bit << bd;
        }
    }

    fn swap_rows(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        let s = self.stride;
        for i in 0..s {
            self.w.swap(a * s + i, b * s + i);
        }
    }

    fn swap_cols(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        for r in 0..self.rows {
            let (x, y) = (self.get(r, a), self.get(r, b));
            if x != y {
                self.toggle(r, a);
                self.toggle(r, b);
            }
        }
    }

    fn grow_stride(&mut self, ns: usize) {
        let mut nw = vec![0u64; self.rows * ns];
        for r in 0..self.rows {
            nw[r * ns..r * ns + self.stride]
                .copy_from_slice(&self.w[r * self.stride..(r + 1) * self.stride]);
        }
        self.w = nw;
        self.stride = ns;
    }

    fn push_col(&mut self) {
        if self.cols + 1 > self.stride * 64 {
            self.grow_stride((self.stride * 2).max(1));
        }
        self.cols += 1;
    }

    fn push_row(&mut self) {
        self.w.resize((self.rows + 1) * self.stride, 0);
        self.rows += 1;
    }

    /// Delete bit column `c`, sliding every higher bit of every row down one.
    /// Word shifts, not a per-row `Vec::remove` of bytes.
    fn remove_col(&mut self, c: usize) {
        let (wc, bc) = (c >> 6, c & 63);
        let s = self.stride;
        let keep = low_mask(bc);
        for row in self.w.chunks_exact_mut(s) {
            drop_bit(row, wc, keep);
        }
        self.cols -= 1;
    }

    /// Delete row `i` AND column `i` in ONE pass — the move a symmetric matrix
    /// makes when a variable leaves. The two-step form (drain the row, then
    /// walk every row again to slide the column out) touches the buffer twice;
    /// this slides and compacts together.
    fn remove_row_and_col(&mut self, idx: usize) {
        debug_assert_eq!(self.rows, self.cols);
        let s = self.stride;
        let (wc, keep) = (idx >> 6, low_mask(idx & 63));
        let rows = self.rows;
        for row in self.w[..idx * s].chunks_exact_mut(s) {
            drop_bit(row, wc, keep);
        }
        for r in idx + 1..rows {
            let (lo, hi) = self.w.split_at_mut(r * s);
            drop_bit_into(&mut lo[(r - 1) * s..], &hi[..s], wc, keep);
        }
        self.w.truncate((rows - 1) * s);
        self.rows -= 1;
        self.cols -= 1;
    }

    /// Is any row's bit `c` set? The column scan the canonical form asks for.
    fn col_any(&self, c: usize) -> bool {
        let (wc, bc) = (c >> 6, c & 63);
        let m = 1u64 << bc;
        self.w.chunks_exact(self.stride).any(|row| row[wc] & m != 0)
    }

    /// The set bits of row `r`, ascending — the support scan every gate needs.
    fn row_support(&self, r: usize, out: &mut Vec<usize>) {
        out.clear();
        let row = self.row(r);
        for (i, &word) in row.iter().enumerate() {
            let mut x = word;
            while x != 0 {
                out.push(i * 64 + x.trailing_zeros() as usize);
                x &= x - 1;
            }
        }
    }

    /// The lowest set bit of row `r` at index `≥ from`, if any.
    fn row_first_set_from(&self, r: usize, from: usize) -> Option<usize> {
        let row = self.row(r);
        let mut i = from >> 6;
        if i >= row.len() {
            return None;
        }
        let mut x = row[i] & !low_mask(from & 63);
        loop {
            if x != 0 {
                return Some(i * 64 + x.trailing_zeros() as usize);
            }
            i += 1;
            if i >= row.len() {
                return None;
            }
            x = row[i];
        }
    }
}

// ------------------------------------------------------------------ affine state

/// One stabilizer branch in affine form: `amplitude(x) = γ·i^{d·u}·(−1)^{Q_J(u)}`
/// on `x = R u ⊕ h`, `u ∈ F₂^k`, `R`'s columns independent.
///
/// `R` and `J` are flat `BitMat`s and `h` is a [`crate::plane::BitPlane`]: one contiguous
/// buffer each, so a row operation is a stride-1 word loop. The mathematics is
/// untouched — the layout is the only thing that changed, and the conformance
/// gates against `holon-qasm` are what say so.
#[derive(Clone, Debug)]
pub struct Affine {
    n: usize,
    /// `R`: n rows × k columns, `x = R u ⊕ h`.
    r: BitMat,
    h: BitPlane,
    /// `d_a mod 4` (the i-power linear part), one per column.
    d: Vec<u8>,
    /// `J_{ab}` (symmetric, diagonal invariantly zero): `(−1)^{J u_a u_b}`.
    j: BitMat,
    gamma: Cyc,
    zero: bool,
    mutations: Mutations,
}

impl Affine {
    /// `|0…0⟩` on n qubits.
    pub fn new(n: usize) -> Self {
        Affine::with_mutations(n, Mutations::default())
    }

    pub fn with_mutations(n: usize, mutations: Mutations) -> Self {
        Affine {
            n,
            r: BitMat::new(n, 0),
            h: BitPlane::zeros(n),
            d: Vec::new(),
            j: BitMat::new(0, 0),
            gamma: Cyc::ONE,
            zero: false,
            mutations,
        }
    }

    pub fn n_qubits(&self) -> usize {
        self.n
    }

    /// The affine dimension: the number of free parameters `u`, and `2^k` is
    /// the width of the state's support.
    pub fn k(&self) -> usize {
        self.d.len()
    }

    /// True when the branch has been annihilated (an inconsistent projection
    /// or a vanishing Gauss sum). Such a branch contributes exactly nothing.
    pub fn is_zero(&self) -> bool {
        self.zero
    }

    pub fn gamma(&self) -> Cyc {
        self.gamma
    }

    /// The J-neighbours of column `a`, ascending. `J`'s diagonal is invariantly
    /// zero, so "set bit of row a" and "`b ≠ a` with `J_ab`" are the same set.
    fn j_neighbours(&self, a: usize, out: &mut Vec<usize>) {
        debug_assert!(!self.j.get(a, a), "J's diagonal must stay clear");
        self.j.row_support(a, out);
    }

    // ---------------------------------------------------------- gauge moves

    /// `u_a := u_a ⊕ u_b` — the F₂ elementary column operation `col_b ^= col_a`,
    /// with the phase polynomial carried along. A bijection of `F₂^k`, so any
    /// sum over `u` is preserved exactly.
    ///
    /// Not to be confused with `merge::fold`: this one carries no ledger.
    fn fold(&mut self, a: usize, b: usize) {
        assert_ne!(a, b);
        self.r.xor_col(b, a);
        let da = self.d[a];
        let jab_old = self.j.get(a, b);
        self.d[b] = (self.d[b] + da) % 4;
        if da & 1 == 1 {
            let v = !jab_old;
            self.j.set(a, b, v);
            self.j.set(b, a, v);
        }
        // `J_ac` for c ∉ {a,b} flips into row b — one word loop for row b, and
        // one bit toggle per neighbour for the symmetric column, in the SAME
        // pass. Rows a and b are masked out of the neighbour set, so the
        // symmetric toggles cannot touch either of the two rows this loop
        // reads and writes, and no scratch list is needed.
        let s = self.j.stride;
        let (base_a, base_b) = (a * s, b * s);
        let (wb, bb) = (b >> 6, 1u64 << (b & 63));
        for i in 0..s {
            let mut m = self.j.w[base_a + i];
            if a >> 6 == i {
                m &= !(1u64 << (a & 63));
            }
            if i == wb {
                m &= !bb;
            }
            self.j.w[base_b + i] ^= m;
            let mut t = m;
            while t != 0 {
                let c = i * 64 + t.trailing_zeros() as usize;
                self.j.w[c * s + wb] ^= bb;
                t &= t - 1;
            }
        }
        self.d[b] = (self.d[b] + if jab_old { 2 } else { 0 }) % 4;
    }

    /// `u_p := 1 ⊕ u_p` — move the coset origin along column p.
    ///
    /// `Σ_a d_a u_a` picks up `d_p − d_p u'_p`, so a global `i^{d_p}` comes out
    /// and `d_p ↦ −d_p`. The J-terms `J_ap u_a (1 ⊕ u'_p)` split (mod 2) into
    /// the same J-term plus a LINEAR `J_ap u'_a`, so every J-neighbour of p gets
    /// `d += 2`; J itself is unchanged. (Same bookkeeping [`Affine::pin_remove`]
    /// does for the value it pins — this keeps the column.)
    fn flip(&mut self, p: usize) {
        if !self.mutations.flip_drops_gamma {
            self.gamma = self.gamma.mul_i_pow(self.d[p]);
        }
        let mut nb = Vec::new();
        self.j_neighbours(p, &mut nb);
        for &a in &nb {
            self.d[a] = (self.d[a] + 2) % 4;
        }
        self.d[p] = (4 - self.d[p] % 4) % 4;
        self.h_xor_col(p);
    }

    /// `h ^= col_p` — the coset origin moving along one column, as a single
    /// pass over `R`'s flat buffer that assembles 64 rows of `h` per word.
    fn h_xor_col(&mut self, p: usize) {
        let (s, n) = (self.r.stride, self.n);
        let (wp, bp) = (p >> 6, p & 63);
        let rows = &self.r.w;
        for (blk, hw) in self.h.words.iter_mut().enumerate() {
            let lo = blk * 64;
            let hi = (lo + 64).min(n);
            let mut acc = 0u64;
            for (i, row) in rows[lo * s..hi * s].chunks_exact(s).enumerate() {
                acc |= ((row[wp] >> bp) & 1) << i;
            }
            *hw ^= acc;
        }
    }

    fn swap_cols(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        self.r.swap_cols(a, b);
        self.d.swap(a, b);
        self.j.swap_rows(a, b);
        self.j.swap_cols(a, b);
    }

    /// Remove column a with `u_a` pinned to `val`.
    fn pin_remove(&mut self, a: usize, val: bool) {
        if val {
            self.h_xor_col(a);
            self.gamma = self.gamma.mul_i_pow(self.d[a]);
            let mut nb = Vec::new();
            self.j_neighbours(a, &mut nb);
            for &c in &nb {
                self.d[c] = (self.d[c] + 2) % 4;
            }
        }
        self.remove_col(a);
    }

    fn remove_col(&mut self, a: usize) {
        self.r.remove_col(a);
        self.d.remove(a);
        self.j.remove_row_and_col(a);
    }

    /// Toggle `J_{b b'}` across every unordered pair of `set`. Done as one row
    /// XOR per member against the set's own mask: each pair is toggled once in
    /// each of its two rows, which is exactly what symmetry wants.
    fn flip_j_across(&mut self, set: &[usize]) {
        if set.len() < 2 {
            return;
        }
        let s = self.j.stride;
        let mut mask = vec![0u64; s];
        for &a in set {
            mask[a >> 6] |= 1u64 << (a & 63);
        }
        for &a in set {
            self.j.xor_row_with(a, &mask, a);
        }
    }

    /// Sum out phase-only column a (its R column is all-zero):
    /// `Σ_w i^{δw} (−1)^{w·Λ}`, `Λ = Σ_{b∈L} u_b`, `L` = the J-neighbours of a.
    ///
    /// `stats` is the sampler's coverage meter; every other caller passes a
    /// throwaway, which costs nothing and keeps ONE Gauss sum in the tree.
    fn gauss_sum_out(&mut self, a: usize, stats: &mut GaussStats) {
        debug_assert!(
            !self.r.col_any(a),
            "gauss_sum_out on a column that still carries an x-dependence"
        );
        let delta = self.d[a];
        let mut l: Vec<usize> = Vec::new();
        self.j_neighbours(a, &mut l);
        match delta % 4 {
            0 | 2 => {
                // Σ_{u_a} (±1)^{u_a}(−1)^{u_a Λ} = 2·[Λ ≡ eps]: a CONSTRAINT,
                // not a phase.
                stats.even_steps += 1;
                let eps = delta == 2; // the constraint Λ ≡ eps
                if l.is_empty() {
                    if eps {
                        stats.annihilated += 1;
                        self.zero = true;
                        self.remove_col(a);
                        return;
                    }
                    self.gamma.m -= 2; // ×2
                    self.remove_col(a);
                } else {
                    // Impose Σ_L u = eps: fold the rest of L onto c = l[0]
                    // (which makes Λ = u_c exactly), then pin u_c.
                    let c = l[0];
                    for &b in &l[1..] {
                        self.fold(c, b);
                    }
                    self.gamma.m -= 2;
                    self.remove_col(a);
                    let c_adj = if c > a { c - 1 } else { c };
                    self.pin_remove(c_adj, eps);
                }
            }
            _ => {
                // δ odd: ×(1+i^δ), and each b∈L gets d_b += δ+2.
                stats.odd_steps += 1;
                // Σ_w i^{δw} over w ∈ {0,1} is itself a branch sum, so it goes
                // through the one merge law and not an ad-hoc ring add.
                let phase = if self.mutations.wrong_gauss {
                    i_pow(delta) // PLANTED WRONG: drops the 1+ structure
                } else {
                    Cyc::ONE.merge(i_pow(delta))
                };
                self.gamma = self.gamma.mul(phase);
                // (−i^δ)^{Λ mod 2} with Λ = Σ_L u: the XOR expansion — the
                // identity (1+i^δ(−1)^Λ) = (1+i^δ)(−i^δ)^Λ holds only for
                // Λ ∈ {0,1}, and Λ here is an integer sum, so the factorised
                // form needs i^{(δ+2)(⊕_L u)}: per-variable phases PLUS
                // pairwise (−1)^{u_a u_b} flips across L. Concretely
                // ⊕_L u = Σ u_b − 2Σ_{b<b'} u_b u_b' + 4(…) and i^{4(…)} = 1,
                // so the XOR is EXACTLY d-bumps plus pairwise J-flips
                // (measured upstream as a sign error on amp(1,1) of
                // h,cx,h,s,h — the entangled HSH sandwich).
                for &b in &l {
                    self.d[b] = (self.d[b] + delta + 2) % 4;
                }
                self.flip_j_across(&l);
                self.remove_col(a);
            }
        }
    }

    // ---------------------------------------------------------- Clifford gates

    pub fn x(&mut self, q: usize) {
        self.h.flip(q);
    }

    pub fn z(&mut self, q: usize) {
        if self.h.get(q) {
            self.gamma = self.gamma.mul_i_pow(2);
        }
        let mut sup = Vec::new();
        self.r.row_support(q, &mut sup);
        for &a in &sup {
            self.d[a] = (self.d[a] + 2) % 4;
        }
    }

    pub fn s(&mut self, q: usize) {
        // i^{x_q}: γ·i^h, d_a += 1+2h for a ∈ A, J_ab ^= 1 for a<b ∈ A.
        let mut a_set = Vec::new();
        self.r.row_support(q, &mut a_set);
        if self.h.get(q) {
            self.gamma = self.gamma.mul_i_pow(1);
        }
        let bump = if self.h.get(q) { 3 } else { 1 };
        for &a in &a_set {
            self.d[a] = (self.d[a] + bump) % 4;
        }
        if !self.mutations.drop_s_cross {
            self.flip_j_across(&a_set);
        }
    }

    pub fn sdg(&mut self, q: usize) {
        self.s(q);
        self.s(q);
        self.s(q);
    }

    pub fn cx(&mut self, c: usize, t: usize) {
        // `row_t ^= row_c` over every column at once — bits past k are
        // invariantly zero, so the whole stride is safe to XOR.
        self.r.xor_rows(t, c);
        if self.h.get(c) {
            self.h.flip(t);
        }
    }

    pub fn h_gate(&mut self, q: usize) {
        // Reduce row q to at most one supporting column a*.
        let mut support = Vec::new();
        self.r.row_support(q, &mut support);
        let a_star = if support.is_empty() {
            None
        } else {
            let a = support[0];
            for &b in &support[1..] {
                self.fold(a, b);
            }
            Some(a)
        };
        // New variable v; phase (−1)^{(u_{a*} ⊕ h_q)·v}.
        let v = self.k();
        self.r.push_col();
        self.d.push(if self.h.get(q) { 2 } else { 0 });
        self.j.push_col();
        self.j.push_row();
        if let Some(a) = a_star {
            self.j.set(a, v, true);
            self.j.set(v, a, true);
        }
        self.r.row_mut(q).fill(0);
        self.r.set(q, v, true);
        self.h.set(q, false);
        self.gamma.m += 1;
        // Row-clearing can break column independence: col a* may now equal an
        // XOR of other columns (two columns that differed only at row q
        // collide once the row is cleared — measured upstream as err 0.375 on
        // h,h,h,h,cx,h). If so, fold that subset into a* until it is all-zero,
        // then Gauss-sum it out; the amplitude query REQUIRES independent
        // columns and says so, loudly.
        if let Some(a) = a_star {
            if self.r.col_any(a) {
                if let Some(subset) = self.dependent_subset(a) {
                    for b in subset {
                        self.fold(b, a);
                    }
                }
            }
            if !self.r.col_any(a) {
                self.gauss_sum_out(a, &mut GaussStats::default());
            }
        }
    }

    /// A row whose support is EXACTLY `{a}` proves column a is independent of
    /// the others: every combination of the others reads zero there, and this
    /// column reads one. A proof, not a heuristic — and the common case, which
    /// is why it is worth `O(n)` word loads to look for.
    fn has_private_row(&self, a: usize) -> bool {
        let (wa, ba) = (a >> 6, 1u64 << (a & 63));
        self.r
            .w
            .chunks_exact(self.r.stride)
            .any(|row| row[wa] == ba && row.iter().enumerate().all(|(i, &w)| i == wa || w == 0))
    }

    /// If column a is an XOR of other columns, return that subset.
    ///
    /// Solve `[cols ≠ a] x = col_a` over F₂ by reduced row elimination on the
    /// flat matrix: the pivot columns are the greedy left-to-right independent
    /// set and the free variables are zero, so the returned subset is the same
    /// particular solution the nested-`Vec` version produced — with the row
    /// operations now stride-1 word XORs and no per-step row clone.
    ///
    /// The elimination is `O(n·k·rank/64)` and it was measured at 94% of the
    /// tier's runtime once the amplitude path was factored — while answering
    /// `None` 4500 times out of 4501, and every one of those 4500 had a
    /// private row standing right there. So the private row is checked first.
    /// It is a proof of the same answer, so nothing about the result moves.
    fn dependent_subset(&self, a: usize) -> Option<Vec<usize>> {
        if self.has_private_row(a) {
            return None;
        }
        self.dependent_subset_eliminate(a)
    }

    /// [`Affine::dependent_subset`] without the private-row shortcut — the
    /// elimination on its own, kept reachable so the shortcut can be gauged
    /// against it (`private_row_agrees_with_elimination`).
    fn dependent_subset_eliminate(&self, a: usize) -> Option<Vec<usize>> {
        let n = self.n;
        let m = self.k() - 1;
        let mut mat = self.r.clone();
        let mut rhs = BitPlane::zeros(n);
        for r in 0..n {
            if mat.get(r, a) {
                rhs.set(r, true);
            }
        }
        mat.remove_col(a);
        let mut piv = vec![usize::MAX; m];
        let mut rr = 0usize;
        for col in 0..m {
            let Some(p) = (rr..n).find(|&p| mat.get(p, col)) else {
                continue;
            };
            mat.swap_rows(rr, p);
            let (x, y) = (rhs.get(rr), rhs.get(p));
            rhs.set(rr, y);
            rhs.set(p, x);
            let rv = rhs.get(rr);
            for p2 in 0..n {
                if p2 != rr && mat.get(p2, col) {
                    mat.xor_rows(p2, rr);
                    let nv = rhs.get(p2) ^ rv;
                    rhs.set(p2, nv);
                }
            }
            piv[col] = rr;
            rr += 1;
        }
        if (rr..n).any(|r| rhs.get(r)) {
            return None; // independent
        }
        let mut subset = Vec::new();
        for col in 0..m {
            if piv[col] != usize::MAX && rhs.get(piv[col]) {
                subset.push(if col < a { col } else { col + 1 });
            }
        }
        Some(subset)
    }

    /// Apply a Clifford gate. `T`/`T†` are the driver's business — they branch,
    /// and a branch is not a state update.
    ///
    /// Generic over the two alphabets: `Gate` (which can name `T`, and is
    /// refused here) and `Clif` (which cannot, so the refusal is unreachable
    /// by construction at those call sites).
    pub fn apply<G: Into<Gate>>(&mut self, g: G) {
        if self.zero {
            return;
        }
        match g.into() {
            Gate::X(q) => self.x(q),
            Gate::Z(q) => self.z(q),
            Gate::S(q) => self.s(q),
            Gate::Sdg(q) => self.sdg(q),
            Gate::H(q) => self.h_gate(q),
            Gate::Cx(c, t) => self.cx(c, t),
            Gate::T(_) | Gate::Tdg(_) => panic!("magic tier branches must be Clifford"),
        }
    }

    /// Tensor a stabilizer block onto the named qubits — the loader a block
    /// decomposition needs to install one of its terms.
    ///
    /// `cols[a]` is column a as a bitmask over `qubits` (bit b ⇒ `R[qubits[b]][a]`),
    /// `h_mask` likewise over `qubits`, `d` and `j` are the block's phase
    /// polynomial. The qubits must be untouched (`|0⟩`, no columns) or the
    /// affine invariant is not this method's to keep.
    pub fn attach(
        &mut self,
        qubits: &[usize],
        cols: &[u32],
        h_mask: u32,
        d: &[u8],
        j: &[Vec<bool>],
    ) {
        let base = self.k();
        for (ci, &mask) in cols.iter().enumerate() {
            let v = self.k();
            self.r.push_col();
            self.d.push(d[ci]);
            self.j.push_col();
            self.j.push_row();
            for (bi, &q) in qubits.iter().enumerate() {
                if (mask >> bi) & 1 == 1 {
                    self.r.set(q, v, true);
                }
            }
        }
        let kt = cols.len();
        for a in 0..kt {
            for b in a + 1..kt {
                if j[a][b] {
                    self.j.set(base + a, base + b, true);
                    self.j.set(base + b, base + a, true);
                }
            }
        }
        for (bi, &q) in qubits.iter().enumerate() {
            if (h_mask >> bi) & 1 == 1 {
                self.h.flip(q);
            }
        }
    }

    /// `γ · i^{Σ d_a u_a} · (−1)^{Σ_{a<b} J_ab u_a u_b}` at a PACKED parameter
    /// `u` (`j.stride` words). The quadratic form is one popcount per set bit
    /// of `u` against `J`'s row, masked to the strictly-higher columns, instead
    /// of the `k²` bit tests the nested form paid.
    fn phase_at(&self, u: &[u64]) -> Cyc {
        let uw = self.j.stride;
        let mut ip: u8 = 0;
        let mut sign = false;
        for w in 0..uw {
            let mut x = u[w];
            while x != 0 {
                let a = w * 64 + x.trailing_zeros() as usize;
                ip = (ip + self.d[a]) % 4;
                let jr = self.j.row(a);
                let hi = if (a & 63) == 63 { 0 } else { !0u64 << ((a & 63) + 1) };
                let mut acc = (jr[w] & u[w] & hi).count_ones();
                for w2 in w + 1..uw {
                    acc += (jr[w2] & u[w2]).count_ones();
                }
                sign ^= acc & 1 == 1;
                x &= x - 1;
            }
        }
        let mut amp = self.gamma.mul_i_pow(ip);
        if sign {
            amp = amp.mul_i_pow(2);
        }
        amp
    }

    /// Solve `R u = b` by reduced row elimination on the flat matrix, exactly
    /// the system [`Affine::amplitude`] has always solved. `None` means `b` is
    /// off the column space; the rank refusal is the caller's, so that the
    /// "off the coset" answer still comes back BEFORE the invariant assertion.
    fn solve_u(&self, b: &BitPlane) -> Option<Vec<u64>> {
        let (n, k) = (self.n, self.k());
        let mut aug = self.r.clone();
        let mut rhs = b.clone();
        let mut pivot_row = vec![usize::MAX; k];
        let mut rr = 0usize;
        for col in 0..k {
            let Some(p) = (rr..n).find(|&p| aug.get(p, col)) else {
                continue;
            };
            aug.swap_rows(rr, p);
            let (x, y) = (rhs.get(rr), rhs.get(p));
            rhs.set(rr, y);
            rhs.set(p, x);
            let rv = rhs.get(rr);
            for p2 in 0..n {
                if p2 != rr && aug.get(p2, col) {
                    aug.xor_rows(p2, rr);
                    let nv = rhs.get(p2) ^ rv;
                    rhs.set(p2, nv);
                }
            }
            pivot_row[col] = rr;
            rr += 1;
        }
        for row in rr..n {
            if rhs.get(row) {
                return None; // y is off the affine subspace
            }
        }
        assert!(
            (0..k).all(|col| pivot_row[col] != usize::MAX),
            "affine invariant broken: R has dependent columns (rank < k)"
        );
        let mut u = vec![0u64; self.j.stride];
        for col in 0..k {
            if rhs.get(pivot_row[col]) {
                u[col >> 6] |= 1u64 << (col & 63);
            }
        }
        Some(u)
    }

    /// Exact amplitude of basis state `y` (bit i = qubit i).
    pub fn amplitude(&self, y: &[bool]) -> Cyc {
        if self.zero {
            return Cyc::ZERO;
        }
        // Solve R u = y ⊕ h.
        let mut b = self.h.clone();
        for row in 0..self.n {
            if y[row] {
                b.flip(row);
            }
        }
        match self.solve_u(&b) {
            None => Cyc::ZERO,
            Some(u) => self.phase_at(&u),
        }
    }

    // ---------------------------------------------------------- projection

    /// Apply the computational-basis projector `Π_{x_q = v}`.
    ///
    /// An affine state projected on one coordinate is another affine state (or
    /// zero): pinning `x_q` pins one variable, exactly the way a measurement
    /// outcome does. The result is NOT renormalised — its norm is the branch's
    /// contribution to `P(x_q = v)`, which is the whole point.
    pub fn project(&mut self, q: usize, v: bool) {
        if self.zero {
            return;
        }
        let mut support = Vec::new();
        self.r.row_support(q, &mut support);
        if support.is_empty() {
            // The support already has a definite x_q = h_q.
            if self.h.get(q) != v {
                self.zero = true;
            }
            return;
        }
        let a = support[0];
        for &b in &support[1..] {
            self.fold(a, b);
        }
        debug_assert!((0..self.k()).all(|c| self.r.get(q, c) == (c == a)));
        let val = v ^ self.h.get(q);
        self.pin_remove(a, val);
        debug_assert_eq!(self.h.get(q), v);
        debug_assert!((0..self.k()).all(|c| !self.r.get(q, c)));
    }

    pub fn projected(&self, q: usize, v: bool) -> Affine {
        let mut s = self.clone();
        s.project(q, v);
        s
    }

    /// Read `(r, h)` as a linear SYSTEM `r·u = h` rather than a parametrisation,
    /// eliminate it by change of variable, then Gauss-sum every remaining
    /// variable out. Returns `Σ_{u : r u = h} γ·i^{d·u}·(−1)^{Q_J(u)}`.
    ///
    /// [`Affine::fold`] and [`Affine::pin_remove`] happen to be correct for BOTH
    /// readings — a parametrisation's column update and a constraint's column
    /// update are the same F₂ operation — which is why the constraint can be
    /// dispatched with the state machinery instead of a separate solver.
    fn solve_and_sum(mut self, stats: &mut GaussStats) -> Cyc {
        if self.zero {
            return Cyc::ZERO;
        }
        // 1. Eliminate every constraint row.
        loop {
            let mut target: Option<(usize, usize)> = None;
            for row in 0..self.n {
                if let Some(col) = self.r.row_first_set_from(row, 0) {
                    target = Some((row, col));
                    break;
                }
            }
            let (row, a) = match target {
                Some(t) => t,
                None => break,
            };
            let mut sup = Vec::new();
            self.r.row_support(row, &mut sup);
            let others: Vec<usize> = sup.into_iter().filter(|&b| b != a).collect();
            for b in others {
                self.fold(a, b);
            }
            debug_assert!((0..self.k()).all(|c| self.r.get(row, c) == (c == a)));
            let val = self.h.get(row);
            self.pin_remove(a, val);
            debug_assert!(!self.h.get(row));
        }
        // 2. A surviving 0 = 1 row means the subspaces are disjoint.
        if self.h.any() {
            stats.inconsistent += 1;
            return Cyc::ZERO;
        }
        // 3. Sum out the free variables.
        while self.k() > 0 {
            self.gauss_sum_out(0, stats);
            if self.zero {
                return Cyc::ZERO;
            }
        }
        self.gamma
    }

    // ---------------------------------------------------------- canonical form

    /// Reduced column echelon form of `R`, phase polynomial carried along.
    /// Returns the (pivot row, pivot column) pairs, pivot columns in order.
    fn rcef(&mut self) -> Vec<(usize, usize)> {
        let mut pivots = Vec::new();
        let mut p = 0usize;
        let mut sup: Vec<usize> = Vec::new();
        for row in 0..self.n {
            if p >= self.k() {
                break;
            }
            let Some(c) = self.r.row_first_set_from(row, p) else {
                continue;
            };
            self.swap_cols(c, p);
            // The row's support is a snapshot: `fold(p, c2)` clears exactly bit
            // c2 of this row and touches no other column of it, so folding the
            // snapshot in ascending order is the same sequence the re-reading
            // loop performed — and the ORDER is load-bearing, because `fold`
            // carries the phase polynomial.
            self.r.row_support(row, &mut sup);
            let targets: Vec<usize> = sup.iter().copied().filter(|&c2| c2 != p).collect();
            for c2 in targets {
                self.fold(p, c2); // col_{c2} ^= col_p
            }
            pivots.push((row, p));
            p += 1;
        }
        pivots
    }

    /// Put the state in canonical form and RETURN the stripped global scalar.
    ///
    /// Afterwards `gamma == Cyc::ONE`, `R` is RCEF with full column rank, `h` is
    /// zero on every pivot row, and `(d, J)` is forced. Two states are equal up
    /// to a global scalar iff their canonical forms agree — so the caller
    /// multiplies the returned scalar into the branch weight and compares the
    /// rest byte for byte. A zero-flagged state returns `Cyc::ZERO`.
    pub fn canonicalize(&mut self) -> Cyc {
        loop {
            if self.zero {
                self.gamma = Cyc::ONE;
                return Cyc::ZERO;
            }
            let pivots = self.rcef();
            // Any column left all-zero is dependent (rank < k): it is a
            // phase-only variable, so sum it out and start over. k strictly
            // decreases, so this terminates.
            if let Some(c) = (0..self.k()).find(|&c| !self.r.col_any(c)) {
                self.gauss_sum_out(c, &mut GaussStats::default());
                continue;
            }
            for &(row, col) in &pivots {
                if self.h.get(row) {
                    self.flip(col);
                }
            }
            break;
        }
        let g = self.gamma;
        self.gamma = Cyc::ONE;
        g
    }

    /// Byte encoding of the canonical form. Equal keys ⇔ equal states up to a
    /// global scalar — PROVIDED [`Affine::canonicalize`] ran first
    /// (debug-asserted).
    ///
    /// The bitstream is byte-for-byte what the nested-`Vec` encoder produced;
    /// only the way the bits get there changed (whole packed ranges at a time
    /// instead of one `push` per bit).
    pub fn canon_key(&self) -> Vec<u8> {
        debug_assert!(
            self.zero || cyc_eq(self.gamma, Cyc::ONE),
            "canon_key on a non-canonical state"
        );
        let k = self.k();
        let mut out = Vec::with_capacity(8 + self.n * (k / 8 + 1) + k + k * k / 8 + 2);
        out.push(self.zero as u8);
        out.extend_from_slice(&(self.n as u32).to_le_bytes());
        out.extend_from_slice(&(k as u32).to_le_bytes());
        let mut bits = BitWriter::new(&mut out);
        for row in 0..self.n {
            bits.push_range(self.r.row(row), 0, k);
        }
        if !self.mutations.key_ignores_h {
            bits.push_range(&self.h.words, 0, self.n);
        }
        for a in 0..k {
            bits.push_range(self.j.row(a), a + 1, k);
        }
        bits.finish();
        out.extend_from_slice(&self.d);
        out
    }

    /// FNV-1a of the canonical key. A hash, and treated as one: a bucket hit is
    /// only ever a hint, never a decision.
    pub fn fingerprint(&self) -> u64 {
        fnv1a(&self.canon_key())
    }

    /// The independent cross-check: do these two canonical states give the SAME
    /// exact amplitude at real basis states, through the same
    /// [`Affine::amplitude`] solver the branch sum uses? Runs the determining
    /// set `{0} ∪ {e_a} ∪ {e_a ⊕ e_b}` (capped at `budget` points), plus two
    /// points off the coset where both must read exactly zero.
    ///
    /// THE HOT PATH of the whole magic tier: this asks ONE pair of states for
    /// `k + n` amplitudes in a row, and re-eliminating `R` from scratch for
    /// every one of them was measured at 95–99% of the tier's runtime. So the
    /// elimination is FACTORED — one `ColSolve` per state, reused across every
    /// probe. The values are the same values: the solution of `R u = b` is
    /// unique when `R` has independent columns (which the affine invariant
    /// requires and this code still asserts), so a factored solve and a fresh
    /// elimination cannot disagree.
    pub fn amplitudes_agree(&self, other: &Affine, budget: usize) -> bool {
        if self.n != other.n {
            return false;
        }
        let k = self.k();
        let sa = ColSolve::build(self);
        let sb = ColSolve::build(other);
        let nw = self.h.words.len();
        let mut y = vec![0u64; nw];
        let mut scratch = vec![0u64; nw];
        let mut checked = 0usize;

        // `probe(u)` of the nested version, with `y = point(u)` built in words:
        // `point(0) = h`, and each set `u_a` XORs in column a.
        let agree = |y: &[u64], scratch: &mut Vec<u64>| -> bool {
            let av = amp_packed(self, &sa, y, scratch);
            let bv = amp_packed(other, &sb, y, scratch);
            cyc_eq(av, bv)
        };

        y.copy_from_slice(&self.h.words);
        if !agree(&y, &mut scratch) {
            return false;
        }
        checked += 1;
        for a in 0..k {
            if checked >= budget {
                break;
            }
            y.copy_from_slice(&self.h.words);
            sa.xor_col(&mut y, a);
            if !agree(&y, &mut scratch) {
                return false;
            }
            checked += 1;
        }
        'pairs: for a in 0..k {
            for b in a + 1..k {
                if checked >= budget {
                    break 'pairs;
                }
                y.copy_from_slice(&self.h.words);
                sa.xor_col(&mut y, a);
                sa.xor_col(&mut y, b);
                if !agree(&y, &mut scratch) {
                    return false;
                }
                checked += 1;
            }
        }
        // Off-coset probes: flip a non-pivot bit of a coset point. If the two
        // states have different supports (the `key_ignores_h` defect), one of
        // these — or the on-coset probes above — must disagree.
        y.copy_from_slice(&self.h.words);
        for q in 0..self.n {
            y[q >> 6] ^= 1u64 << (q & 63);
            if !agree(&y, &mut scratch) {
                return false;
            }
            y[q >> 6] ^= 1u64 << (q & 63);
        }
        true
    }
}

/// `state.amplitude(y)` for a PACKED `y`, through a prebuilt [`ColSolve`].
/// Same three answers in the same order as the unfactored method: zero if the
/// branch is annihilated, zero if `y` is off the coset, otherwise the phase —
/// with the rank refusal after the coset test, exactly where it was.
fn amp_packed(st: &Affine, s: &ColSolve, y: &[u64], scratch: &mut Vec<u64>) -> Cyc {
    if st.zero {
        return Cyc::ZERO;
    }
    scratch.clear();
    scratch.extend_from_slice(y);
    for (a, b) in scratch.iter_mut().zip(&st.h.words) {
        *a ^= *b;
    }
    match s.solve(scratch) {
        None => Cyc::ZERO,
        Some(u) => {
            assert!(
                s.rank == s.k,
                "affine invariant broken: R has dependent columns (rank < k)"
            );
            st.phase_at(&u)
        }
    }
}

/// A REUSABLE exact solver for `R u = b`: the greedy left-to-right F₂ column
/// basis of `R`, packed.
///
/// Building it costs one transpose and one basis pass; after that every solve
/// is a handful of stride-1 word XORs rather than a fresh `O(n·k)` elimination.
/// The pivot columns it keeps are the columns not in the span of their
/// predecessors — the SAME set reduced row echelon form picks — and inside the
/// span of independent columns a representation is unique, so `solve` returns
/// the identical `u` the elimination returned. No approximation, no tolerance:
/// this is the same exact linear algebra over F₂, done once instead of `k + n`
/// times.
struct ColSolve {
    nw: usize,
    uw: usize,
    k: usize,
    /// The raw columns of `R`, `nw` words each — `point(u)` reads these.
    cols: Vec<u64>,
    basis: Vec<u64>,
    comb: Vec<u64>,
    /// leading bit ↦ basis index, `usize::MAX` for none.
    by_pivot: Vec<usize>,
    rank: usize,
}

impl ColSolve {
    fn build(st: &Affine) -> ColSolve {
        let (n, k) = (st.n, st.k());
        let nw = st.h.words.len();
        let uw = st.j.stride;
        // Transpose R once: the flat row buffer walked set bit by set bit.
        let mut cols = vec![0u64; k * nw];
        for row in 0..n {
            let (rw, rb) = (row >> 6, 1u64 << (row & 63));
            for (i, &word) in st.r.row(row).iter().enumerate() {
                let mut x = word;
                while x != 0 {
                    let c = i * 64 + x.trailing_zeros() as usize;
                    cols[c * nw + rw] |= rb;
                    x &= x - 1;
                }
            }
        }
        let mut s = ColSolve {
            nw,
            uw,
            k,
            cols,
            basis: vec![0u64; k * nw],
            comb: vec![0u64; k * uw],
            by_pivot: vec![usize::MAX; n],
            rank: 0,
        };
        let mut v = vec![0u64; nw];
        let mut m = vec![0u64; uw];
        for c in 0..k {
            v.copy_from_slice(&s.cols[c * nw..(c + 1) * nw]);
            m.iter_mut().for_each(|x| *x = 0);
            m[c >> 6] |= 1u64 << (c & 63);
            if let Some(b) = s.reduce(&mut v, &mut m) {
                let r = s.rank;
                s.basis[r * nw..(r + 1) * nw].copy_from_slice(&v);
                s.comb[r * uw..(r + 1) * uw].copy_from_slice(&m);
                s.by_pivot[b] = r;
                s.rank = r + 1;
            }
        }
        s
    }

    /// Reduce `v` against the basis, carrying the combination `m`. Returns the
    /// leading bit of the irreducible remainder, or `None` when `v` reached 0
    /// (in which case `m` is the exact combination that produces the input).
    fn reduce(&self, v: &mut [u64], m: &mut [u64]) -> Option<usize> {
        let mut w = self.nw;
        loop {
            while w > 0 && v[w - 1] == 0 {
                w -= 1;
            }
            if w == 0 {
                return None;
            }
            let b = (w - 1) * 64 + (63 - v[w - 1].leading_zeros() as usize);
            let i = self.by_pivot[b];
            if i == usize::MAX {
                return Some(b);
            }
            let bb = &self.basis[i * self.nw..i * self.nw + self.nw];
            for (x, y) in v.iter_mut().zip(bb) {
                *x ^= *y;
            }
            let cc = &self.comb[i * self.uw..i * self.uw + self.uw];
            for (x, y) in m.iter_mut().zip(cc) {
                *x ^= *y;
            }
        }
    }

    /// `R u = b`, or `None` when `b` is off the column space.
    fn solve(&self, b: &[u64]) -> Option<Vec<u64>> {
        let mut v = b.to_vec();
        let mut m = vec![0u64; self.uw];
        match self.reduce(&mut v, &mut m) {
            Some(_) => None,
            None => Some(m),
        }
    }

    /// `y ^= col_a` — the packed form of `point`'s column accumulation.
    #[inline]
    fn xor_col(&self, y: &mut [u64], a: usize) {
        let c = &self.cols[a * self.nw..a * self.nw + self.nw];
        for (x, z) in y.iter_mut().zip(c) {
            *x ^= *z;
        }
    }
}

/// FNV-1a over bytes. The one hash in the engine, so a bucket index and a
/// state fingerprint cannot drift apart.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hsh: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hsh ^= *b as u64;
        hsh = hsh.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hsh
}

/// LSB-first bit packer. `push_range` takes a whole run of bits out of a
/// packed source word by word; the byte stream it emits is identical to the
/// one a bit-at-a-time writer emits, which is what keeps [`Affine::canon_key`]
/// stable across this change.
struct BitWriter<'a> {
    out: &'a mut Vec<u8>,
    cur: u64,
    n: usize,
}

impl<'a> BitWriter<'a> {
    fn new(out: &'a mut Vec<u8>) -> Self {
        BitWriter { out, cur: 0, n: 0 }
    }
    #[cfg(test)]
    fn push(&mut self, b: bool) {
        self.cur |= (b as u64) << self.n;
        self.n += 1;
        if self.n == 64 {
            self.out.extend_from_slice(&self.cur.to_le_bytes());
            self.cur = 0;
            self.n = 0;
        }
    }
    fn push_range(&mut self, src: &[u64], from: usize, to: usize) {
        let mut i = from;
        while i < to {
            let (w, b) = (i >> 6, i & 63);
            let want = (to - i).min(64 - b).min(64 - self.n);
            let chunk = (src[w] >> b) & low_mask(want);
            self.cur |= chunk << self.n;
            self.n += want;
            i += want;
            if self.n == 64 {
                self.out.extend_from_slice(&self.cur.to_le_bytes());
                self.cur = 0;
                self.n = 0;
            }
        }
    }
    fn finish(self) {
        let bytes = self.n.div_ceil(8);
        if bytes > 0 {
            self.out.extend_from_slice(&self.cur.to_le_bytes()[..bytes]);
        }
    }
}

// ----------------------------------------------------------------- overlaps

/// Exact `⟨φ|φ'⟩` between two affine states on the same register.
///
/// `O(n·k² + k³)` with `k = k_φ + k_φ' ≤ 2n` — no enumeration of the subspaces.
///
/// `⟨φ|φ'⟩ = Σ_x amp_φ(x)‾ · amp_φ'(x)` is a quadratic-form Gauss sum over F₂
/// restricted to the solution set of `R u ⊕ R' u' = h ⊕ h'`, and it is
/// evaluated in three exact moves: eliminate the constraint by CHANGE OF
/// VARIABLE (`fold`/`pin_remove` transform R and the phase polynomial in
/// lockstep, so the sum is preserved), Gauss-sum the free variables one at a
/// time, and return `γ` — with every variable summed out, `γ` IS the overlap.
pub fn overlap(a: &Affine, b: &Affine) -> Cyc {
    overlap_gauged(a, b, false, &mut GaussStats::default())
}

/// [`overlap`] with the Gauss-sum mutation and the coverage meter exposed, for
/// the gauge. `wrong_gauss` plants the phase error; `stats` records which Gauss
/// branches the call actually reached.
///
/// The mutation is applied to the COMBINED state built here, never to either
/// input — so a gauged overlap cannot perturb the states the sampler holds.
pub fn overlap_gauged(
    a: &Affine,
    b: &Affine,
    wrong_gauss: bool,
    stats: &mut GaussStats,
) -> Cyc {
    assert_eq!(a.n, b.n, "overlap of states on different registers");
    if a.zero || b.zero {
        stats.inconsistent += 1;
        return Cyc::ZERO;
    }
    let (ka, kb) = (a.k(), b.k());
    let k = ka + kb;
    let n = a.n;
    // Constraint: R_a u ⊕ R_b u' = h_a ⊕ h_b (the intersection of the two
    // affine subspaces, parametrised jointly).
    let mut r = BitMat::new(n, k);
    let mut h = BitPlane::zeros(n);
    for row in 0..n {
        for c in 0..ka {
            if a.r.get(row, c) {
                r.set(row, c, true);
            }
        }
        for c in 0..kb {
            if b.r.get(row, c) {
                r.set(row, ka + c, true);
            }
        }
    }
    for (x, (p, q)) in h.words.iter_mut().zip(a.h.words.iter().zip(&b.h.words)) {
        *x = *p ^ *q;
    }
    // Phase: conj on the left flips the sign of the i-powers; (−1)^{Q_J} is
    // real, so J is carried over unchanged, block-diagonal in (u, u').
    let mut d = vec![0u8; k];
    for c in 0..ka {
        d[c] = (4 - a.d[c] % 4) % 4;
    }
    d[ka..ka + kb].copy_from_slice(&b.d[..kb]);
    let mut j = BitMat::new(k, k);
    for x in 0..ka {
        for y in 0..ka {
            if a.j.get(x, y) {
                j.set(x, y, true);
            }
        }
    }
    for x in 0..kb {
        for y in 0..kb {
            if b.j.get(x, y) {
                j.set(ka + x, ka + y, true);
            }
        }
    }
    let comb = Affine {
        n,
        r,
        h,
        d,
        j,
        gamma: cyc_conj(a.gamma).mul(b.gamma),
        zero: false,
        mutations: Mutations { wrong_gauss, ..Mutations::default() },
    };
    comb.solve_and_sum(stats)
}

/// The referee: `⟨φ|φ'⟩` by summing over all `2^n` basis states. Exact but
/// exponential in the REGISTER, so it exists to test [`overlap`], never to be
/// called by it.
pub fn overlap_bruteforce(a: &Affine, b: &Affine) -> Cyc {
    assert_eq!(a.n, b.n);
    let n = a.n;
    assert!(n <= 20, "brute-force overlap is 2^n by construction");
    crate::merge::fold((0..(1usize << n)).map(|idx| {
        let y: Vec<bool> = (0..n).map(|q| idx >> q & 1 == 1).collect();
        let av = a.amplitude(&y);
        if cyc_is_zero(av) {
            return Cyc::ZERO;
        }
        cyc_conj(av).mul(b.amplitude(&y))
    }))
}

// --------------------------------------------------------------------- rng

/// splitmix64 — a deterministic, seed-reproducible stream. Small on purpose:
/// the crate carries zero runtime dependencies.
#[derive(Clone, Debug)]
pub struct Rng {
    s: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng { s: seed }
    }
    pub fn next_u64(&mut self) -> u64 {
        self.s = self.s.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.s;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// A 32-bit dyadic deviate R, meaning `R/2³² ∈ [0,1)`.
    pub fn next_dyadic32(&mut self) -> u64 {
        self.next_u64() >> 32
    }
    /// A draw in `[0, n)`. Modulo, so it is very slightly biased for `n` that
    /// does not divide `2^64` — fine for picking a qubit or a gate kind, and
    /// NOT fine for a sampling decision, which is why the sampler's own draw
    /// goes through [`Rng::next_dyadic32`] and an exact comparison instead.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

#[cfg(test)]
mod layout_conformance {
    use super::*;

    /// The packed bit writer must emit the byte stream a bit-at-a-time writer
    /// emits — `canon_key`'s stability across the layout change rests on it.
    #[test]
    fn push_range_matches_bit_pushes() {
        let src: Vec<u64> = vec![
            0x0123_4567_89ab_cdef,
            0xfedc_ba98_7654_3210,
            0xdead_beef_cafe_f00d,
        ];
        for lead in [0usize, 1, 5, 7, 8, 63, 64, 100] {
            for from in [0usize, 1, 7, 63, 64, 65, 127] {
                for to in [from, from + 1, from + 7, from + 64, from + 130] {
                    if to > 192 {
                        continue;
                    }
                    let mut a = Vec::new();
                    let mut wa = BitWriter::new(&mut a);
                    for i in 0..lead {
                        wa.push(i % 3 == 0);
                    }
                    wa.push_range(&src, from, to);
                    wa.finish();
                    let mut b = Vec::new();
                    let mut wb = BitWriter::new(&mut b);
                    for i in 0..lead {
                        wb.push(i % 3 == 0);
                    }
                    for i in from..to {
                        wb.push(src[i >> 6] >> (i & 63) & 1 == 1);
                    }
                    wb.finish();
                    assert_eq!(a, b, "lead={lead} from={from} to={to}");
                }
            }
        }
    }

    /// `remove_col` must slide the higher bits down and nothing else.
    #[test]
    fn remove_col_slides_bits() {
        for cols in [1usize, 5, 63, 64, 65, 130] {
            for victim in [0usize, 1, 63, 64, 100] {
                if victim >= cols {
                    continue;
                }
                let mut m = BitMat::new(3, cols);
                let bit = |r: usize, c: usize| (r * 7 + c * 3 + c / 5) % 3 == 0;
                for r in 0..3 {
                    for c in 0..cols {
                        m.set(r, c, bit(r, c));
                    }
                }
                m.remove_col(victim);
                assert_eq!(m.cols, cols - 1);
                for r in 0..3 {
                    for c in 0..cols - 1 {
                        let want = if c < victim { bit(r, c) } else { bit(r, c + 1) };
                        assert_eq!(m.get(r, c), want, "cols={cols} victim={victim} r={r} c={c}");
                    }
                    // Nothing may survive past the new width.
                    for c in cols - 1..m.stride * 64 {
                        assert!(!m.get(r, c), "cols={cols} victim={victim}: stale bit at {c}");
                    }
                }
            }
        }
    }

    /// The fused symmetric delete must drop exactly row `i` and column `i`.
    #[test]
    fn remove_row_and_col_drops_one_of_each() {
        let bit = |r: usize, c: usize| (r * 13 + c * 7 + r * c) % 5 == 0;
        for k in [1usize, 2, 7, 63, 64, 65, 130] {
            for victim in [0usize, 1, 63, 64, 100] {
                if victim >= k {
                    continue;
                }
                let mut m = BitMat::new(k, k);
                for r in 0..k {
                    for c in 0..k {
                        m.set(r, c, bit(r, c));
                    }
                }
                m.remove_row_and_col(victim);
                assert_eq!((m.rows, m.cols), (k - 1, k - 1));
                for r in 0..k - 1 {
                    let sr = if r < victim { r } else { r + 1 };
                    for c in 0..k - 1 {
                        let sc = if c < victim { c } else { c + 1 };
                        assert_eq!(m.get(r, c), bit(sr, sc), "k={k} victim={victim} r={r} c={c}");
                    }
                    for c in k - 1..m.stride * 64 {
                        assert!(!m.get(r, c), "k={k} victim={victim}: stale bit at {c}");
                    }
                }
            }
        }
    }

    /// The private-row shortcut must never disagree with the elimination it
    /// replaces: whenever it fires, the elimination has to say `None` too, and
    /// whenever it does not fire the two are the same call.
    #[test]
    fn private_row_agrees_with_elimination() {
        let mut rng = Rng::new(0x5EED_D00D);
        let mut fired = 0usize;
        let mut checked = 0usize;
        for n in [3usize, 5, 8] {
            for _ in 0..60 {
                let mut st = Affine::new(n);
                for _ in 0..5 * n {
                    let q = rng.below(n);
                    let mut q2 = rng.below(n);
                    while q2 == q {
                        q2 = rng.below(n);
                    }
                    match rng.below(5) {
                        0 => st.x(q),
                        1 => st.z(q),
                        2 => st.s(q),
                        3 => st.h_gate(q),
                        _ => st.cx(q, q2),
                    }
                }
                for a in 0..st.k() {
                    checked += 1;
                    let elim = st.dependent_subset_eliminate(a);
                    if st.has_private_row(a) {
                        fired += 1;
                        assert!(elim.is_none(), "private row claimed independence, elimination did not");
                    }
                    assert_eq!(st.dependent_subset(a), elim, "n={n} a={a}");
                }
            }
        }
        assert!(fired * 4 > checked, "the shortcut never fired: {fired} of {checked}");
    }

    /// The factored solver and the fresh elimination must return the SAME `u`
    /// — the claim the whole `amplitudes_agree` speedup rests on.
    #[test]
    fn factored_solver_matches_elimination() {
        let mut rng = Rng::new(0xA1F1_9E5E);
        for n in [3usize, 5, 9] {
            for _ in 0..40 {
                let mut st = Affine::new(n);
                for _ in 0..6 * n {
                    let q = rng.below(n);
                    let mut q2 = rng.below(n);
                    while q2 == q {
                        q2 = rng.below(n);
                    }
                    match rng.below(5) {
                        0 => st.x(q),
                        1 => st.z(q),
                        2 => st.s(q),
                        3 => st.h_gate(q),
                        _ => st.cx(q, q2),
                    }
                }
                let s = ColSolve::build(&st);
                let mut scratch = Vec::new();
                for idx in 0..(1usize << n) {
                    let y: Vec<bool> = (0..n).map(|q| idx >> q & 1 == 1).collect();
                    let mut yw = vec![0u64; st.h.words.len()];
                    for (q, &b) in y.iter().enumerate() {
                        if b {
                            yw[q >> 6] |= 1u64 << (q & 63);
                        }
                    }
                    assert_eq!(
                        st.amplitude(&y),
                        amp_packed(&st, &s, &yw, &mut scratch),
                        "n={n} idx={idx}"
                    );
                }
            }
        }
    }
}
