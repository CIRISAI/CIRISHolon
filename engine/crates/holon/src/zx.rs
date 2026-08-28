//! GRAPH REWRITING — the layer BENCHMARKS entry sixteen proved is the magic
//! tier's prerequisite, built CIRIS-native.
//!
//! The measurement that forced this: on random Clifford+T circuits, ZX
//! rewriting removes 63–79% of the T-count while our block-local passes
//! remove 2–8%, because Hadamards are frequent enough on realistic circuits
//! that block-local methods have nothing to merge. In a graph-like ZX
//! diagram a Hadamard is an EDGE DECORATION, not a barrier — that is the
//! whole reason the technique reaches further.
//!
//! ALGORITHM CREDIT, per the house rule: this implements the graph-theoretic
//! simplification of Duncan, Kissinger, Perdrix and van de Wetering
//! ("Graph-theoretic Simplification of Quantum Circuits with the
//! ZX-calculus", Quantum 4, 279 (2020)) together with the phase-teleportation
//! gadget machinery of Kissinger and van de Wetering (arXiv:1903.10477). The
//! reference implementation is quizx (Apache-2.0, Kissinger–van de Wetering);
//! this is our own implementation of their published rules — the rule set,
//! the driver order (`interior_clifford_simp` ⊂ `clifford_simp` ⊂
//! `full_reduce`) and the scalar bookkeeping follow theirs so the two are
//! comparable reading-for-reading, and quizx remains the benchmark.
//!
//! THE CIRIS-NATIVE PART, and it is two things:
//!
//! 1. Adjacency is BIT-PACKED in TWO planes — `adj` (an edge is present) and
//!    `had` (that edge is a Hadamard edge) — one `u64` word per 64 vertices.
//!    Local complementation becomes one word-parallel XOR sweep per
//!    neighbour, and pivoting three of them; the *scalar* those rewrites owe
//!    (√2^−2 per edge the toggle destroys) falls out of the same sweep as a
//!    POPCOUNT, so the exact bookkeeping costs nothing extra. That is the
//!    discipline that beat stim at tier 1, applied to a graph.
//! 2. The scalar is carried in the holon's own exact ring `ledger::Cyc`
//!    (`Z[ω]·2^{−m/2}`, ω = e^{iπ/4}) — never a float. So a reduced diagram
//!    is not merely T-count-equivalent to the circuit, it is EQUAL to it, and
//!    `tests/zx.rs` checks that equality exactly on every basis state.
//!
//! Phases are EXACT: an integer multiple of π/4 (mod 8), never a float — so
//! "is this spider Clifford" is an integer test, not a tolerance.
//!
//! WHAT THIS SHIPS (stated plainly, because the deliverables differ in
//! strength): a certified **T-count oracle** and a certified **scalar**, not a
//! circuit extractor. `full_reduce` takes a diagram to normal form and
//! `t_count` reads the surviving non-Clifford spiders; `eval` computes the
//! diagram's exact value. There is NO extraction back to a circuit — the
//! reduced diagram is not re-synthesised, so this cannot yet hand a shorter
//! circuit to the runner. The exactness gate is therefore stated on the
//! diagram: for a plugged (closed) diagram, `eval` before reduction, `eval`
//! after reduction, and `run::amplitude` on the same circuit all agree
//! EXACTLY as ring elements.
//!
//! THE DEFECT THIS REPLACES, recorded because entry seventeen located it and
//! the location was wrong in one specific way: the old pass had a bespoke
//! `gadgetize()` that split a T-spider into hub+carrier and then looked for a
//! pivot. That is not the published rule and it cannot fire. In the real
//! algorithm gadgets are not manufactured — they are the RESIDUE of pivoting:
//! `gen_pivot` unfuses a non-Pauli phase onto a pendant spider *and then
//! pivots the vertex away*, and the hub that survives the pivot is the
//! gadget. A gadgetizer that does not pivot the carrier's host out of the
//! graph produces nothing, which is exactly what was measured.

use crate::affine::Gate;
use crate::ledger::Cyc;
use crate::qasm::Surface;
use std::collections::HashMap;

// ---------------------------------------------------------------- bit words

#[inline]
fn bit(v: &[u64], i: usize) -> bool {
    v[i >> 6] >> (i & 63) & 1 == 1
}

#[inline]
fn set_bit(v: &mut [u64], i: usize, on: bool) {
    if on {
        v[i >> 6] |= 1u64 << (i & 63);
    } else {
        v[i >> 6] &= !(1u64 << (i & 63));
    }
}

fn bits_of(mask: &[u64]) -> Vec<usize> {
    let mut out = Vec::new();
    for (w, &word) in mask.iter().enumerate() {
        let mut x = word;
        while x != 0 {
            out.push(w * 64 + x.trailing_zeros() as usize);
            x &= x - 1;
        }
    }
    out
}

// ------------------------------------------------------------- ring helpers

/// `ω^k` exactly, ω = e^{iπ/4}; the reduction is `ω⁴ = −1`.
#[inline]
fn omega(k: i64) -> Cyc {
    let k = k.rem_euclid(8) as usize;
    let mut c = [0i128; 4];
    if k < 4 {
        c[k] = 1;
    } else {
        c[k - 4] = -1;
    }
    Cyc { c, m: 0 }
}

/// Exact ring equality by DIFFERENCE, not by struct: two `Cyc` values can
/// denote the same complex number with different `(c, m)` when the `m`
/// parities differ (√2 = ω − ω³ is a ring element), so `PartialEq` is the
/// wrong test and subtraction is the right one.
pub fn cyc_eq(a: Cyc, b: Cyc) -> bool {
    a.add(b.mul(omega(4))).c.iter().all(|&x| x == 0)
}

// ------------------------------------------------------------------- graph

/// A ZX diagram in graph-like form: every spider is Z-type, phases are exact
/// eighths of a turn (units of π/4, mod 8), and every edge between two
/// spiders is a Hadamard edge. Boundary vertices (circuit inputs and outputs)
/// carry no phase and may hold either edge type.
#[derive(Clone, Debug)]
pub struct ZxGraph {
    /// `phase[v]` in units of π/4, mod 8.
    pub phase: Vec<i64>,
    /// Boundary vertices are never the target of a rewrite.
    boundary: Vec<bool>,
    /// Bit-packed: `adj[v]` has bit `u` set iff an edge (v,u) exists.
    adj: Vec<Vec<u64>>,
    /// Bit-packed: `had[v]` has bit `u` set iff that edge is a Hadamard edge.
    /// A subset of `adj` at all times.
    had: Vec<Vec<u64>>,
    /// Removed vertices are tombstoned; `compact` reclaims them.
    alive: Vec<bool>,
    words: usize,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
    /// The diagram's global scalar, EXACT.
    pub scalar: Cyc,
}

impl ZxGraph {
    fn with_capacity(cap: usize) -> ZxGraph {
        let words = cap.div_ceil(64).max(1);
        ZxGraph {
            phase: Vec::with_capacity(cap),
            boundary: Vec::with_capacity(cap),
            adj: Vec::with_capacity(cap),
            had: Vec::with_capacity(cap),
            alive: Vec::with_capacity(cap),
            words,
            inputs: Vec::new(),
            outputs: Vec::new(),
            scalar: Cyc::ONE,
        }
    }

    fn add_vertex(&mut self, phase: i64, boundary: bool) -> usize {
        let v = self.phase.len();
        if v >= self.words * 64 {
            self.words = (v >> 6) + 1;
            for a in self.adj.iter_mut() {
                a.push(0);
            }
            for h in self.had.iter_mut() {
                h.push(0);
            }
        }
        self.phase.push(phase.rem_euclid(8));
        self.boundary.push(boundary);
        self.alive.push(true);
        self.adj.push(vec![0u64; self.words]);
        self.had.push(vec![0u64; self.words]);
        v
    }

    // ------------------------------------------------------------ accessors

    #[inline]
    fn has_edge(&self, u: usize, v: usize) -> bool {
        bit(&self.adj[u], v)
    }

    #[inline]
    fn is_h(&self, u: usize, v: usize) -> bool {
        bit(&self.had[u], v)
    }

    fn set_edge(&mut self, u: usize, v: usize, e: Option<bool>) {
        debug_assert_ne!(u, v, "self-loops are reduced away, never stored");
        let (present, h) = match e {
            None => (false, false),
            Some(x) => (true, x),
        };
        set_bit(&mut self.adj[u], v, present);
        set_bit(&mut self.adj[v], u, present);
        set_bit(&mut self.had[u], v, h);
        set_bit(&mut self.had[v], u, h);
    }

    fn neighbours(&self, v: usize) -> Vec<usize> {
        bits_of(&self.adj[v])
    }

    fn degree(&self, v: usize) -> usize {
        self.adj[v].iter().map(|w| w.count_ones() as usize).sum()
    }

    fn remove(&mut self, v: usize) {
        for u in self.neighbours(v) {
            set_bit(&mut self.adj[u], v, false);
            set_bit(&mut self.had[u], v, false);
        }
        for w in self.adj[v].iter_mut() {
            *w = 0;
        }
        for w in self.had[v].iter_mut() {
            *w = 0;
        }
        self.alive[v] = false;
    }

    /// The published `add_edge_smart`: adding an edge where one already
    /// exists is not an error, it is a rewrite — parallel Hadamard edges
    /// cancel, a mixed pair costs a π and a √2, and a Hadamard self-loop is a
    /// π. Every case pays its exact scalar.
    fn add_edge_smart(&mut self, u: usize, v: usize, new_h: bool) {
        if u == v {
            debug_assert!(!self.boundary[u], "no self-loops on a boundary");
            if new_h {
                self.phase[u] = (self.phase[u] + 4).rem_euclid(8);
                self.scalar.m += 1; // ×√2⁻¹
            }
            return;
        }
        match (self.has_edge(u, v), self.is_h(u, v), new_h) {
            (false, _, h) => self.set_edge(u, v, Some(h)),
            (true, false, false) => {}
            (true, true, true) => {
                self.set_edge(u, v, None);
                self.scalar.m += 2; // ×√2⁻²
            }
            (true, h0, _) => {
                // one N and one H edge in parallel: keep the N, pay a π.
                debug_assert!(!self.boundary[u], "boundaries never gain a phase");
                if h0 {
                    self.set_edge(u, v, Some(false));
                }
                self.phase[u] = (self.phase[u] + 4).rem_euclid(8);
                self.scalar.m += 1;
            }
        }
    }

    /// XOR `mask` into row `u` on BOTH planes, returning how many edges the
    /// flip DESTROYED. Only legal where every edge the mask can touch is a
    /// Hadamard edge between two spiders — the invariant that holds
    /// throughout the rewriting phase, because spider fusion runs to fixpoint
    /// first and no rewrite ever creates a plain edge between two spiders.
    ///
    /// The popcount is the point: the destroyed-edge count is the diagram's
    /// √2 debt, and it is read off the same words the XOR writes.
    fn xor_row_h(&mut self, u: usize, mask: &[u64]) -> u32 {
        let mut destroyed = 0;
        for i in 0..self.words {
            let m = mask[i];
            debug_assert_eq!(
                self.adj[u][i] & m & !self.had[u][i],
                0,
                "xor_row_h met a plain edge between two spiders: fusion is not at fixpoint"
            );
            destroyed += (self.adj[u][i] & m).count_ones();
            self.adj[u][i] ^= m;
            self.had[u][i] ^= m;
        }
        destroyed
    }

    /// Reclaim tombstones and shrink the word width. The published simplifier
    /// packs inside its loops for the same reason: after the first Clifford
    /// pass the graph is 5–6× smaller and every later sweep is that much
    /// cheaper.
    pub fn compact(&mut self) {
        if self.alive.iter().all(|&a| a) {
            return;
        }
        let n = self.alive.len();
        let mut map = vec![usize::MAX; n];
        let mut k = 0;
        for (v, &a) in self.alive.iter().enumerate() {
            if a {
                map[v] = k;
                k += 1;
            }
        }
        let words = k.div_ceil(64).max(1);
        let mut phase = Vec::with_capacity(k);
        let mut boundary = Vec::with_capacity(k);
        let mut adj = Vec::with_capacity(k);
        let mut had = Vec::with_capacity(k);
        for v in 0..n {
            if !self.alive[v] {
                continue;
            }
            phase.push(self.phase[v]);
            boundary.push(self.boundary[v]);
            let mut a = vec![0u64; words];
            let mut h = vec![0u64; words];
            for u in self.neighbours(v) {
                set_bit(&mut a, map[u], true);
                if self.is_h(v, u) {
                    set_bit(&mut h, map[u], true);
                }
            }
            adj.push(a);
            had.push(h);
        }
        self.inputs = self.inputs.iter().map(|&b| map[b]).collect();
        self.outputs = self.outputs.iter().map(|&b| map[b]).collect();
        self.alive = vec![true; k];
        self.phase = phase;
        self.boundary = boundary;
        self.adj = adj;
        self.had = had;
        self.words = words;
    }

    // ---------------------------------------------------------- diagnostics

    /// T-count: spiders whose phase is an ODD multiple of π/4. This is the
    /// published metric (`tcount`): the count of non-Clifford spiders.
    pub fn t_count(&self) -> usize {
        (0..self.phase.len())
            .filter(|&v| self.alive[v] && !self.boundary[v] && self.phase[v].rem_euclid(2) == 1)
            .count()
    }

    pub fn n_spiders(&self) -> usize {
        (0..self.phase.len()).filter(|&v| self.alive[v]).count()
    }

    /// How many phase gadgets exist, and how many supports carry more than
    /// one — the diagnostic that measured the old pass's defect as a flat
    /// zero.
    pub fn gadget_stats(&self) -> (usize, usize) {
        let map = self.gadget_map();
        (map.values().map(|v| v.len()).sum(), map.values().filter(|v| v.len() > 1).count())
    }

    // ---------------------------------------------------------------- rules

    fn check_spider_fusion(&self, u: usize, v: usize) -> bool {
        u != v
            && self.alive[u]
            && self.alive[v]
            && !self.boundary[u]
            && !self.boundary[v]
            && self.has_edge(u, v)
            && !self.is_h(u, v)
    }

    /// SPIDER FUSION: two spiders joined by a plain edge are one spider whose
    /// phase is the sum. `v` is absorbed into `u`.
    fn spider_fusion(&mut self, u: usize, v: usize) {
        for w in self.neighbours(v) {
            if w != u {
                let h = self.is_h(v, w);
                self.add_edge_smart(u, w, h);
            }
        }
        self.phase[u] = (self.phase[u] + self.phase[v]).rem_euclid(8);
        self.remove(v);
    }

    fn check_remove_id(&self, v: usize) -> bool {
        self.alive[v] && !self.boundary[v] && self.phase[v].rem_euclid(8) == 0 && self.degree(v) == 2
    }

    /// IDENTITY REMOVAL: a phase-0 spider of degree 2 is a wire. Its two
    /// edges compose, so the surviving edge type is their PARITY.
    fn remove_id(&mut self, v: usize) {
        let n = self.neighbours(v);
        let (a, b) = (n[0], n[1]);
        let e = self.is_h(v, a) ^ self.is_h(v, b);
        self.remove(v);
        self.add_edge_smart(a, b, e);
    }

    /// Is every edge at `v` a Hadamard edge to another spider? (The
    /// "interior" precondition of the Clifford rules.)
    fn all_h_to_spider(&self, v: usize) -> bool {
        self.neighbours(v).into_iter().all(|u| !self.boundary[u] && self.is_h(v, u))
    }

    fn check_local_comp(&self, v: usize) -> bool {
        self.alive[v]
            && !self.boundary[v]
            && matches!(self.phase[v].rem_euclid(8), 2 | 6)
            && self.all_h_to_spider(v)
    }

    /// LOCAL COMPLEMENTATION — the word-parallel rewrite. Delete `v`,
    /// complement the Hadamard edges among its neighbourhood, and shift every
    /// neighbour's phase by −a.
    fn local_comp(&mut self, v: usize) {
        let p = self.phase[v].rem_euclid(8);
        let nbrs = self.neighbours(v);
        let mask = self.adj[v].clone();
        let mut destroyed = 0u32;
        for &u in &nbrs {
            let mut m = mask.clone();
            set_bit(&mut m, u, false);
            destroyed += self.xor_row_h(u, &m);
        }
        // Each destroyed edge is counted once from each endpoint, and each
        // costs √2⁻²; the two factors of two cancel.
        self.scalar.m += destroyed as i32;
        for &u in &nbrs {
            self.phase[u] = (self.phase[u] - p).rem_euclid(8);
        }
        self.remove(v);
        let x = nbrs.len() as i32;
        self.scalar.m -= (x - 1) * (x - 2) / 2;
        // ω^{p/2} on the SIGNED representative: +π/2 ↦ ω, −π/2 ↦ ω⁻¹.
        self.scalar = self.scalar.mul(omega(if p == 2 { 1 } else { -1 }));
    }

    fn check_pivot(&self, u: usize, v: usize) -> bool {
        u != v
            && self.alive[u]
            && self.alive[v]
            && !self.boundary[u]
            && !self.boundary[v]
            && self.has_edge(u, v)
            && self.is_h(u, v)
            && matches!(self.phase[u].rem_euclid(8), 0 | 4)
            && matches!(self.phase[v].rem_euclid(8), 0 | 4)
            && self.all_h_to_spider(u)
            && self.all_h_to_spider(v)
    }

    /// PIVOT — three word-parallel neighbourhood sweeps, then delete both
    /// spiders. `A` is exclusive to `v0`, `B` exclusive to `v1`, `C` shared;
    /// A×B, A×C and B×C are toggled, `C` additionally picks up a π (the
    /// Hadamard self-loop the published rule generates) and its own √2 debt.
    fn pivot(&mut self, v0: usize, v1: usize) {
        let d0 = self.degree(v0) as i32;
        let d1 = self.degree(v1) as i32;
        let mut m0 = self.adj[v0].clone();
        set_bit(&mut m0, v1, false);
        let mut m1 = self.adj[v1].clone();
        set_bit(&mut m1, v0, false);
        let mc: Vec<u64> = m0.iter().zip(&m1).map(|(a, b)| a & b).collect();
        let ma: Vec<u64> = m0.iter().zip(&m1).map(|(a, b)| a & !b).collect();
        let mb: Vec<u64> = m1.iter().zip(&m0).map(|(a, b)| a & !b).collect();
        let mab: Vec<u64> = ma.iter().zip(&mb).map(|(a, b)| a | b).collect();
        let mac: Vec<u64> = ma.iter().zip(&mc).map(|(a, b)| a | b).collect();
        let mbc: Vec<u64> = mb.iter().zip(&mc).map(|(a, b)| a | b).collect();
        let (a, b, c) = (bits_of(&ma), bits_of(&mb), bits_of(&mc));

        let mut destroyed = 0u32;
        for &x in &a {
            destroyed += self.xor_row_h(x, &mbc);
        }
        for &x in &b {
            destroyed += self.xor_row_h(x, &mac);
        }
        for &x in &c {
            destroyed += self.xor_row_h(x, &mab);
        }
        self.scalar.m += destroyed as i32;

        let (p0, p1) = (self.phase[v0].rem_euclid(8), self.phase[v1].rem_euclid(8));
        for &x in &a {
            self.phase[x] = (self.phase[x] + p1).rem_euclid(8);
        }
        for &x in &b {
            self.phase[x] = (self.phase[x] + p0).rem_euclid(8);
        }
        for &x in &c {
            self.phase[x] = (self.phase[x] + p0 + p1 + 4).rem_euclid(8);
        }
        let cn = c.len() as i32;
        self.scalar.m += cn; // one √2⁻¹ per Hadamard self-loop on C
        self.scalar.m += cn * (cn - 1); // √2⁻² per unordered pair inside C
        self.scalar.m -= (d0 - 2) * (d1 - 2);
        if p0 == 4 && p1 == 4 {
            self.scalar = self.scalar.mul(omega(4));
        }
        self.remove(v0);
        self.remove(v1);
    }

    /// Unfuse a non-Pauli phase onto a pendant spider, leaving `v` at phase
    /// zero. THIS IS THE GADGET SOURCE: the pendant becomes the gadget's
    /// phase carrier once `v` is pivoted away.
    fn unfuse_gadget(&mut self, v: usize) {
        let p = self.phase[v].rem_euclid(8);
        if p == 0 || p == 4 {
            return;
        }
        let hub = self.add_vertex(0, false);
        let carrier = self.add_vertex(p, false);
        self.phase[v] = 0;
        self.set_edge(v, hub, Some(true));
        self.set_edge(hub, carrier, Some(true));
    }

    /// Insert an identity spider so `v` is no longer adjacent to boundary
    /// `b`, making `v` interior and therefore pivotable.
    fn unfuse_boundary(&mut self, v: usize, b: usize) {
        if !self.boundary[b] || !self.has_edge(v, b) {
            return;
        }
        let et = self.is_h(v, b);
        let w = self.add_vertex(0, false);
        self.set_edge(v, b, None);
        self.set_edge(v, w, Some(true));
        self.set_edge(w, b, Some(!et));
    }

    /// Every edge at `v` is either a Hadamard edge to a spider or any edge to
    /// a boundary — the precondition `gen_pivot` can repair.
    fn gen_pivot_shape(&self, v: usize) -> bool {
        self.neighbours(v).into_iter().all(|w| self.boundary[w] || self.is_h(v, w))
    }

    /// Interior, Pauli-phased, and not a gadget hub (no pendant neighbour).
    fn is_interior_pauli(&self, v: usize) -> bool {
        matches!(self.phase[v].rem_euclid(8), 0 | 4)
            && self.neighbours(v).into_iter().all(|n| !self.boundary[n] && self.degree(n) > 1)
    }

    /// The reducing form of the generic pivot check: at least one endpoint is
    /// an interior Pauli spider, so applying the rule strictly decreases
    /// their number and the loop terminates.
    fn check_gen_pivot_reduce(&self, v0: usize, v1: usize) -> bool {
        v0 != v1
            && self.alive[v0]
            && self.alive[v1]
            && !self.boundary[v0]
            && !self.boundary[v1]
            && self.has_edge(v0, v1)
            && self.is_h(v0, v1)
            && self.gen_pivot_shape(v0)
            && self.gen_pivot_shape(v1)
            && (self.is_interior_pauli(v0) || self.is_interior_pauli(v1))
    }

    /// GENERIC PIVOT: repair both endpoints (unfuse non-Pauli phases into
    /// gadgets, push boundaries one hop away), then pivot. The hubs that
    /// survive the pivot ARE the phase gadgets — nothing else manufactures
    /// them.
    fn gen_pivot(&mut self, v0: usize, v1: usize) {
        for &v in &[v0, v1] {
            let nbrs = self.neighbours(v);
            self.unfuse_gadget(v);
            for b in nbrs {
                self.unfuse_boundary(v, b);
            }
        }
        self.pivot(v0, v1);
    }

    /// Gadgets keyed by SUPPORT: a pendant spider (the phase carrier) hangs
    /// off a phase-0 hub whose other Hadamard-edge neighbours are the
    /// support. Two gadgets on the same support are one gadget.
    fn gadget_map(&self) -> HashMap<Vec<usize>, Vec<(usize, usize)>> {
        let mut map: HashMap<Vec<usize>, Vec<(usize, usize)>> = HashMap::new();
        for v in 0..self.phase.len() {
            if !self.alive[v] || self.boundary[v] || self.degree(v) != 1 {
                continue;
            }
            let hub = self.neighbours(v)[0];
            if self.boundary[hub] || self.phase[hub].rem_euclid(8) != 0 {
                continue;
            }
            // A hub touching a boundary is NOT a fusable gadget. The
            // published routine keys the support on the hub's spider
            // neighbours alone, which would let two hubs with identical
            // spider support but different BOUNDARY attachments fuse — and
            // fusing deletes the hub, orphaning the boundary. Refusing here
            // is strictly more conservative and costs nothing measured: the
            // T-counts are unchanged on all 22 head-to-head circuits.
            if self.neighbours(hub).into_iter().any(|n| self.boundary[n]) {
                continue;
            }
            let mut support: Vec<usize> = self
                .neighbours(hub)
                .into_iter()
                .filter(|&n| n != v && self.is_h(hub, n))
                .collect();
            support.sort_unstable();
            map.entry(support).or_default().push((hub, v));
        }
        map
    }

    /// GADGET FUSION — the rewrite that actually removes T. Bit-packed
    /// supports make "same support" an exact comparison, so this is a
    /// group-by rather than a set-equality search.
    pub fn fuse_gadgets(&mut self) -> bool {
        let map = self.gadget_map();
        let mut fused = false;
        for (support, gs) in map {
            if gs.len() < 2 {
                continue;
            }
            // A hub carrying two pendants appears under two different keys;
            // skip anything an earlier group already retired.
            if gs.iter().any(|&(h, c)| !self.alive[h] || !self.alive[c]) {
                continue;
            }
            fused = true;
            let mut ph = 0;
            for &(hub, carrier) in gs.iter().skip(1) {
                ph += self.phase[carrier];
                self.remove(hub);
                self.remove(carrier);
            }
            let keep = gs[0].1;
            self.phase[keep] = (self.phase[keep] + ph).rem_euclid(8);
            let num = gs.len() as i32;
            let degree = support.len() as i32;
            self.scalar.m += (num - 1) * (degree - 1);
        }
        fused
    }

    /// π-COPY on a gadget carrier: pushes the hub's π onto the carrier so the
    /// hub returns to phase 0 and becomes fusable again.
    fn pi_copy(&mut self, v: usize) {
        let p = self.phase[v].rem_euclid(8);
        self.scalar = self.scalar.mul(omega(p));
        self.phase[v] = (-p).rem_euclid(8);
        for u in self.neighbours(v) {
            self.phase[u] = (self.phase[u] + 4).rem_euclid(8);
        }
    }

    /// Clear π phases out of gadget hubs. Keyed by HUB, so a hub carrying
    /// several pendants is flipped exactly once.
    fn remove_gadget_pi(&mut self) -> bool {
        let mut hubs: HashMap<usize, usize> = HashMap::new();
        for v in 0..self.phase.len() {
            if !self.alive[v] || self.boundary[v] || self.degree(v) != 1 {
                continue;
            }
            let hub = self.neighbours(v)[0];
            if !self.boundary[hub] && self.is_h(v, hub) && self.phase[hub].rem_euclid(8) == 4 {
                hubs.insert(hub, v);
            }
        }
        let matched = !hubs.is_empty();
        for (_, carrier) in hubs {
            self.pi_copy(carrier);
        }
        matched
    }

    /// An isolated spider is a scalar: `1 + e^{ip}`.
    fn remove_single(&mut self, v: usize) {
        let s = Cyc::ONE.add(omega(self.phase[v]));
        self.scalar = self.scalar.mul(s);
        self.remove(v);
    }

    /// A connected pair of pendant spiders is a scalar — one value for the
    /// plain edge, another for the Hadamard edge.
    fn remove_pair(&mut self, u: usize, v: usize) {
        let (p0, p1) = (self.phase[u], self.phase[v]);
        let s = if self.is_h(u, v) {
            self.scalar.m += 1;
            Cyc::ONE.add(omega(p0)).add(omega(p1)).add(omega(p0 + p1 + 4))
        } else {
            Cyc::ONE.add(omega(p0 + p1))
        };
        self.scalar = self.scalar.mul(s);
        self.remove(u);
        self.remove(v);
    }

    // -------------------------------------------------------------- drivers

    fn spider_simp(&mut self) -> bool {
        let mut any = false;
        loop {
            let mut m = false;
            for u in 0..self.phase.len() {
                if !self.alive[u] || self.boundary[u] {
                    continue;
                }
                let hit = self.neighbours(u).into_iter().find(|&v| self.check_spider_fusion(u, v));
                if let Some(v) = hit {
                    self.spider_fusion(u, v);
                    m = true;
                }
            }
            if !m {
                return any;
            }
            any = true;
        }
    }

    fn id_simp(&mut self) -> bool {
        let mut any = false;
        loop {
            let mut m = false;
            for v in 0..self.phase.len() {
                if self.check_remove_id(v) {
                    self.remove_id(v);
                    m = true;
                }
            }
            if !m {
                return any;
            }
            any = true;
        }
    }

    fn local_comp_simp(&mut self) -> bool {
        let mut any = false;
        loop {
            let mut m = false;
            for v in 0..self.phase.len() {
                if self.check_local_comp(v) {
                    self.local_comp(v);
                    m = true;
                }
            }
            if !m {
                return any;
            }
            any = true;
        }
    }

    fn pivot_simp(&mut self) -> bool {
        let mut any = false;
        loop {
            let mut m = false;
            for u in 0..self.phase.len() {
                if !self.alive[u] {
                    continue;
                }
                let hit = self.neighbours(u).into_iter().find(|&v| self.check_pivot(u, v));
                if let Some(v) = hit {
                    self.pivot(u, v);
                    m = true;
                }
            }
            if !m {
                return any;
            }
            any = true;
        }
    }

    fn gen_pivot_simp(&mut self) -> bool {
        let mut any = false;
        loop {
            let mut m = false;
            for u in 0..self.phase.len() {
                if !self.alive[u] {
                    continue;
                }
                let hit =
                    self.neighbours(u).into_iter().find(|&v| self.check_gen_pivot_reduce(u, v));
                if let Some(v) = hit {
                    self.gen_pivot(u, v);
                    m = true;
                }
            }
            if !m {
                return any;
            }
            any = true;
        }
    }

    fn scalar_simp(&mut self) -> bool {
        let mut any = false;
        for v in 0..self.phase.len() {
            if self.alive[v] && !self.boundary[v] && self.degree(v) == 0 {
                self.remove_single(v);
                any = true;
            }
        }
        for u in 0..self.phase.len() {
            if !self.alive[u] || self.boundary[u] || self.degree(u) != 1 {
                continue;
            }
            let v = self.neighbours(u)[0];
            if v > u && !self.boundary[v] && self.degree(v) == 1 {
                self.remove_pair(u, v);
                any = true;
            }
        }
        any
    }

    /// The interior Clifford loop: fusion, identity removal, pivoting and
    /// local complementation to fixpoint.
    ///
    /// NOTE, stated exactly because BENCHMARKS entry seventeen stated it too
    /// broadly: the GRAPH rewrites in this loop cannot change the T-count.
    /// Local complementation shifts neighbour phases by ±π/2 and pivoting by
    /// 0 or π — both EVEN in units of π/4 — so neither can change a phase's
    /// parity, and no amount of Clifford rewriting alone reaches the
    /// published 63–79%. What CAN move the count here is spider fusion and
    /// the scalar rules: two odd phases fusing make an even one, and a
    /// pendant pair is absorbed into the scalar outright. Both only ever
    /// REMOVE. The reduction that needs the gadget layer is everything
    /// beyond that, and it is the bulk of it.
    pub fn interior_clifford_simp(&mut self) -> bool {
        self.spider_simp();
        let mut any = false;
        let mut m = true;
        while m {
            m = self.id_simp();
            m |= self.spider_simp();
            m |= self.pivot_simp();
            m |= self.local_comp_simp();
            m |= self.scalar_simp();
            any |= m;
            self.compact();
        }
        any
    }

    /// Interior Clifford simplification plus the generic (boundary- and
    /// gadget-repairing) pivot, to fixpoint.
    pub fn clifford_simp(&mut self) -> bool {
        let mut any = false;
        let mut m = true;
        while m {
            m = self.interior_clifford_simp();
            m |= self.gen_pivot_simp();
            any |= m;
        }
        any
    }

    /// FULL REDUCTION: Clifford simplification, gadget fusion and gadget-π
    /// removal, iterated to fixpoint — `full_simp` in the published
    /// vocabulary. This is the pass that moves T-count.
    pub fn full_reduce(&mut self) {
        // A fence, not a tolerance: the published loop terminates (the
        // reducing pivot check strictly decreases the interior Pauli count),
        // and if it ever did not, stopping early can only OVER-report the
        // T-count — never corrupt the diagram.
        let mut settled = false;
        for _ in 0..1024 {
            let mut m = self.clifford_simp();
            m |= self.fuse_gadgets();
            m |= self.remove_gadget_pi();
            if !m {
                settled = true;
                break;
            }
        }
        debug_assert!(settled, "full_reduce hit its iteration fence without reaching a fixpoint");
        self.compact();
    }

    // -------------------------------------------------------------- plugging

    /// Fix a boundary to a computational basis value. `|0⟩` is the phase-0
    /// pendant X-spider over `√2`; in the all-Z presentation that is a
    /// phase-0 (or π) spider with the incident edge type TOGGLED, times √2⁻¹.
    fn plug(&mut self, b: usize, val: bool) {
        let nbrs = self.neighbours(b);
        debug_assert_eq!(nbrs.len(), 1, "a boundary has exactly one edge");
        let u = nbrs[0];
        let h = self.is_h(b, u);
        self.boundary[b] = false;
        self.phase[b] = if val { 4 } else { 0 };
        self.set_edge(b, u, Some(!h));
        self.scalar.m += 1;
    }

    pub fn plug_inputs(&mut self, vals: &[bool]) {
        assert_eq!(vals.len(), self.inputs.len());
        for (i, &v) in self.inputs.clone().iter().zip(vals) {
            self.plug(*i, v);
        }
        self.inputs.clear();
    }

    pub fn plug_outputs(&mut self, vals: &[bool]) {
        assert_eq!(vals.len(), self.outputs.len());
        for (o, &v) in self.outputs.clone().iter().zip(vals) {
            self.plug(*o, v);
        }
        self.outputs.clear();
    }

    /// Is this diagram the bare identity — every input wired straight to its
    /// own output by a plain edge, and no spiders left at all?
    pub fn is_identity_wiring(&self) -> bool {
        if self.inputs.len() != self.outputs.len() {
            return false;
        }
        let live: Vec<usize> = (0..self.phase.len()).filter(|&v| self.alive[v]).collect();
        if live.len() != self.inputs.len() + self.outputs.len() {
            return false;
        }
        if live.iter().any(|&v| !self.boundary[v]) {
            return false;
        }
        self.inputs.iter().zip(&self.outputs).all(|(&i, &o)| {
            self.degree(i) == 1 && self.degree(o) == 1 && self.has_edge(i, o) && !self.is_h(i, o)
        })
    }

    pub fn is_closed(&self) -> bool {
        (0..self.phase.len()).all(|v| !self.alive[v] || !self.boundary[v])
    }

    // ------------------------------------------------------------ evaluation

    /// The diagram's EXACT value, as a ring element, for a CLOSED diagram
    /// (every boundary plugged).
    ///
    /// The semantics, stated once so the scalar conventions above are
    /// checkable rather than asserted: a spider `v` carries a variable
    /// `s_v ∈ {0,1}` and contributes `ω^{phase_v · s_v}`; a Hadamard edge
    /// contributes `2^{−1/2}(−1)^{s_u s_v}`; a plain edge identifies its two
    /// variables. The value is the sum over all assignments, times the global
    /// scalar.
    ///
    /// Evaluated by BUCKET ELIMINATION in a greedy min-degree order, so the
    /// cost is exponential in the diagram's induced width rather than in its
    /// vertex count.
    pub fn eval(&self) -> Cyc {
        assert!(self.is_closed(), "eval needs a closed diagram: plug the boundaries first");
        let n = self.phase.len();

        // Plain edges identify variables: union-find over the N-edges.
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(p: &mut Vec<usize>, x: usize) -> usize {
            let mut r = x;
            while p[r] != r {
                r = p[r];
            }
            let mut c = x;
            while p[c] != r {
                let nx = p[c];
                p[c] = r;
                c = nx;
            }
            r
        }
        for v in 0..n {
            if !self.alive[v] {
                continue;
            }
            for u in self.neighbours(v) {
                if u > v && !self.is_h(v, u) {
                    let (a, b) = (find(&mut parent, v), find(&mut parent, u));
                    if a != b {
                        parent[a] = b;
                    }
                }
            }
        }

        // Class representatives become the variables.
        let mut var_of = vec![usize::MAX; n];
        let mut nvars = 0;
        let mut phase = Vec::new();
        for v in 0..n {
            if !self.alive[v] {
                continue;
            }
            let r = find(&mut parent, v);
            if var_of[r] == usize::MAX {
                var_of[r] = nvars;
                nvars += 1;
                phase.push(0i64);
            }
            var_of[v] = var_of[r];
            phase[var_of[r]] += self.phase[v];
        }
        let mut scalar = self.scalar;

        // Hadamard-edge multiplicities between variable classes; an edge that
        // has become a self-loop is a π and a √2⁻¹.
        let mut mult: HashMap<(usize, usize), u32> = HashMap::new();
        for v in 0..n {
            if !self.alive[v] {
                continue;
            }
            for u in self.neighbours(v) {
                if u <= v || !self.is_h(v, u) {
                    continue;
                }
                let (a, b) = (var_of[v], var_of[u]);
                if a == b {
                    phase[a] += 4;
                    scalar.m += 1;
                } else {
                    *mult.entry((a.min(b), a.max(b))).or_insert(0) += 1;
                }
            }
        }

        // Factor list: unary phases, then one binary per adjacent pair.
        let mut factors: Vec<(Vec<usize>, Vec<Cyc>)> = Vec::new();
        for a in 0..nvars {
            factors.push((vec![a], vec![Cyc::ONE, omega(phase[a])]));
        }
        for (&(a, b), &k) in mult.iter() {
            scalar.m += k as i32; // (2^{-1/2})^k
            let sign = if k % 2 == 0 { Cyc::ONE } else { omega(4) };
            factors.push((vec![a, b], vec![Cyc::ONE, Cyc::ONE, Cyc::ONE, sign]));
        }

        // Greedy min-degree elimination order over the interaction graph.
        let mut nbr: Vec<std::collections::BTreeSet<usize>> =
            vec![std::collections::BTreeSet::new(); nvars];
        for (&(a, b), _) in mult.iter() {
            nbr[a].insert(b);
            nbr[b].insert(a);
        }
        let mut order = Vec::with_capacity(nvars);
        let mut gone = vec![false; nvars];
        for _ in 0..nvars {
            let pick = (0..nvars)
                .filter(|&v| !gone[v])
                .min_by_key(|&v| nbr[v].iter().filter(|&&u| !gone[u]).count())
                .unwrap();
            gone[pick] = true;
            let live: Vec<usize> = nbr[pick].iter().copied().filter(|&u| !gone[u]).collect();
            for i in 0..live.len() {
                for j in (i + 1)..live.len() {
                    nbr[live[i]].insert(live[j]);
                    nbr[live[j]].insert(live[i]);
                }
            }
            order.push(pick);
        }

        for v in order {
            let (mut hit, keep): (Vec<_>, Vec<_>) =
                factors.into_iter().partition(|(vs, _)| vs.contains(&v));
            factors = keep;
            if hit.is_empty() {
                // A free variable: sums to 2.
                scalar = scalar.mul(Cyc { c: [2, 0, 0, 0], m: 0 });
                continue;
            }
            // Union of the scopes, with `v` last so summing it out is a fold
            // over the top half of the table.
            let mut scope: Vec<usize> = Vec::new();
            for (vs, _) in &hit {
                for &x in vs {
                    if x != v && !scope.contains(&x) {
                        scope.push(x);
                    }
                }
            }
            assert!(
                scope.len() < 20,
                "zx::eval: induced width {} exceeds the evaluator's envelope",
                scope.len()
            );
            let mut full = scope.clone();
            full.push(v);
            let size = 1usize << full.len();
            let mut table = vec![Cyc::ONE; size];
            for (vs, t) in hit.drain(..) {
                let pos: Vec<usize> = vs.iter().map(|x| full.iter().position(|y| y == x).unwrap()).collect();
                for (idx, cell) in table.iter_mut().enumerate() {
                    let mut j = 0usize;
                    for (k, &p) in pos.iter().enumerate() {
                        j |= ((idx >> p) & 1) << k;
                    }
                    *cell = cell.mul(t[j]);
                }
            }
            let half = size >> 1;
            let mut out = vec![Cyc::ZERO; half];
            for (idx, cell) in out.iter_mut().enumerate() {
                *cell = table[idx].add(table[idx | half]);
            }
            if scope.is_empty() {
                scalar = scalar.mul(out[0]);
            } else {
                factors.push((scope, out));
            }
        }
        for (_, t) in factors {
            scalar = scalar.mul(t[0]);
        }
        scalar
    }
}

// ---------------------------------------------------------------- extraction

/// A dense GF(2) matrix with bit-packed rows — the same discipline as the
/// diagram's adjacency, because frontier Gaussian elimination is the same
/// word-parallel XOR sweep.
#[derive(Clone)]
struct BitMat {
    rows: usize,
    cols: usize,
    words: usize,
    d: Vec<u64>,
}

impl BitMat {
    fn zero(rows: usize, cols: usize) -> BitMat {
        let words = cols.div_ceil(64).max(1);
        BitMat { rows, cols, words, d: vec![0u64; rows * words] }
    }

    fn identity(n: usize) -> BitMat {
        let mut m = BitMat::zero(n, n);
        for i in 0..n {
            m.set(i, i);
        }
        m
    }

    #[inline]
    fn get(&self, r: usize, c: usize) -> bool {
        self.d[r * self.words + (c >> 6)] >> (c & 63) & 1 == 1
    }

    #[inline]
    fn set(&mut self, r: usize, c: usize) {
        self.d[r * self.words + (c >> 6)] |= 1u64 << (c & 63);
    }

    /// `row[dst] ^= row[src]`
    fn add_row(&mut self, src: usize, dst: usize) {
        let (a, b) = (src * self.words, dst * self.words);
        for i in 0..self.words {
            self.d[b + i] ^= self.d[a + i];
        }
    }

    fn swap_rows(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        for i in 0..self.words {
            self.d.swap(a * self.words + i, b * self.words + i);
        }
    }

    fn row_weight(&self, r: usize) -> u32 {
        self.d[r * self.words..(r + 1) * self.words].iter().map(|w| w.count_ones()).sum()
    }

    fn row_bits(&self, r: usize) -> Vec<usize> {
        bits_of(&self.d[r * self.words..(r + 1) * self.words])
    }
}

/// Gauss–Jordan over GF(2), mirroring every row operation into `proxy` so
/// that `proxy[i]` names exactly which ORIGINAL rows XOR to give `self[i]`.
fn gauss_jordan(m: &mut BitMat, proxy: &mut BitMat) {
    let mut pivot = 0;
    for col in 0..m.cols {
        if pivot == m.rows {
            break;
        }
        let Some(r) = (pivot..m.rows).find(|&r| m.get(r, col)) else { continue };
        m.swap_rows(pivot, r);
        proxy.swap_rows(pivot, r);
        for r2 in 0..m.rows {
            if r2 != pivot && m.get(r2, col) {
                m.add_row(pivot, r2);
                proxy.add_row(pivot, r2);
            }
        }
        pivot += 1;
    }
}

/// The result of extraction: a circuit in the surface alphabet, plus the
/// residual global scalar. The contract is an EQUATION, not an
/// approximation — the original diagram equals `scalar · circuit`.
#[derive(Clone, Debug)]
pub struct Extraction {
    pub gates: Vec<Surface>,
    pub scalar: Cyc,
}

impl ZxGraph {
    /// Toggle the Hadamard neighbourhood of `v` by `mask`, keeping the
    /// mirrored halves in step, and return how many edges the flip
    /// DESTROYED. Unlike `xor_row_h` this is ASYMMETRIC — only `v`'s
    /// neighbourhood changes — which is what a frontier row operation is.
    fn xor_neighbourhood(&mut self, v: usize, mask: &[u64]) -> u32 {
        let destroyed = self.xor_row_h(v, mask);
        for w in bits_of(mask) {
            let on = bit(&self.adj[v], w);
            set_bit(&mut self.adj[w], v, on);
            set_bit(&mut self.had[w], v, on);
        }
        destroyed
    }

    /// A frontier vertex is a boundary-adjacent Pauli spider — the
    /// precondition for pivoting a gadget off the frontier.
    fn is_boundary_pauli(&self, v: usize) -> bool {
        matches!(self.phase[v].rem_euclid(8), 0 | 4)
            && self.neighbours(v).into_iter().any(|n| self.boundary[n])
    }

    fn check_boundary_pivot(&self, v0: usize, v1: usize) -> bool {
        v0 != v1
            && self.alive[v0]
            && self.alive[v1]
            && !self.boundary[v0]
            && !self.boundary[v1]
            && self.has_edge(v0, v1)
            && self.is_h(v0, v1)
            && self.gen_pivot_shape(v0)
            && self.gen_pivot_shape(v1)
            && self.is_boundary_pauli(v0)
    }

    /// CIRCUIT EXTRACTION — hand the reduced diagram back as a circuit.
    ///
    /// ALGORITHM CREDIT: the frontier/gflow extraction of Duncan, Kissinger,
    /// Perdrix and van de Wetering (Quantum 4, 279 (2020) §5), in the refined
    /// form of Backens, Miller-Bakewell, de Felice, Lorenz and van de Wetering
    /// ("There and back again: A circuit extraction tale", Quantum 5, 421
    /// (2021)); quizx's `extract.rs` (Apache-2.0) is the reference
    /// implementation and the benchmark. This is our own implementation of
    /// their published procedure over the bit-packed representation.
    ///
    /// The shape: walk a FRONTIER of spiders adjacent to the outputs inward.
    /// At each step peel off what is already a gate (a Hadamard edge to the
    /// output, a phase on the frontier, an edge between two frontier
    /// vertices), pivot away any phase gadget touching the frontier, and then
    /// consume any frontier spider that has become a plain wire. When none
    /// has, Gaussian elimination over GF(2) on the frontier's biadjacency
    /// MAKES one — and the row operations that do it are exactly the CNOTs.
    ///
    /// EXACTNESS: every step pays its scalar into `self.scalar`, including
    /// the two that are easy to miss — a frontier–frontier Hadamard edge
    /// traded for a CZ owes a √2⁻¹ (the edge is normalized, the gate is not),
    /// and a row operation owes `√2^{destroyed − created}` because it changes
    /// how many Hadamard edges the diagram contains. The returned `scalar`
    /// closes the equation.
    pub fn extract(&mut self) -> Result<Extraction, String> {
        let nq = self.outputs.len();
        if nq == 0 {
            return Err("zx: cannot extract a circuit from a diagram with no outputs".into());
        }
        let mut circuit: std::collections::VecDeque<Surface> = std::collections::VecDeque::new();

        // Phase gadgets present at the start. Extraction only ever removes
        // them; it never creates one, so this set is a shrinking budget.
        let mut gadgets: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for v in 0..self.phase.len() {
            if self.alive[v] && !self.boundary[v] && self.degree(v) == 1 {
                let hub = self.neighbours(v)[0];
                if !self.boundary[hub] {
                    gadgets.insert(hub);
                }
            }
        }

        // frontier[i] = (qubit, spider)
        let mut frontier: Vec<(usize, usize)> = Vec::new();
        let fence = 64 * self.n_spiders() + 1024;
        for _ in 0..fence {
            // ---- PREPROCESSING: peel gates off the outputs, build the frontier
            frontier.clear();
            for q in 0..nq {
                let o = self.outputs[q];
                let nbrs = self.neighbours(o);
                if nbrs.len() != 1 {
                    return Err(format!(
                        "zx: output {o} has degree {} — the diagram is not a circuit",
                        nbrs.len()
                    ));
                }
                let v = nbrs[0];
                if self.is_h(o, v) {
                    circuit.push_front(Surface::H(q));
                    self.set_edge(o, v, Some(false));
                }
                if self.boundary[v] {
                    continue; // this wire is fully extracted
                }
                frontier.push((q, v));
                let p = self.phase[v].rem_euclid(8);
                if p != 0 {
                    circuit.push_front(Surface::DiagPow(p, q));
                    self.phase[v] = 0;
                }
            }
            // Frontier-to-frontier edges and input padding need the frontier
            // complete, so they run in a second sweep.
            for i in 0..frontier.len() {
                let (q, v) = frontier[i];
                for n in self.neighbours(v) {
                    if n == self.outputs[q] {
                        continue;
                    }
                    if self.boundary[n] {
                        if !self.inputs.contains(&n) {
                            return Err(format!("zx: spider {v} joins two outputs"));
                        }
                        // A frontier vertex that is more than a bare wire
                        // needs its input pushed one hop away.
                        if self.degree(v) > 2 {
                            self.unfuse_boundary(v, n);
                        }
                    } else if let Some(&(r, _)) = frontier.iter().find(|&&(_, w)| w == n) {
                        self.set_edge(v, n, None);
                        self.scalar.m += 1; // the edge is normalized, CZ is not
                        circuit.push_front(Surface::Cz(q, r));
                    }
                }
            }
            if frontier.is_empty() {
                // ---- FINAL PERMUTATION
                self.permutation_to_gates(&mut circuit)?;
                return Ok(Extraction {
                    gates: circuit.into_iter().collect(),
                    scalar: self.scalar,
                });
            }

            // ---- GADGET PHASE: pivot any gadget touching the frontier away
            let mut fixed = false;
            'gad: for &(_, v) in &frontier {
                for n in self.neighbours(v) {
                    if gadgets.contains(&n) {
                        if !self.check_boundary_pivot(v, n) {
                            return Err(format!(
                                "zx: gadget {n} on the frontier at {v} is not pivotable"
                            ));
                        }
                        self.gen_pivot(v, n);
                        gadgets.remove(&n);
                        fixed = true;
                        break 'gad;
                    }
                }
            }
            if fixed {
                continue;
            }

            // ---- MAIN PHASE: consume frontier spiders that are now wires
            if self.consume_frontier(&frontier) {
                continue;
            }
            // ...and if none is, make one with CNOTs.
            self.frontier_gauss(&frontier, &mut circuit)?;
            if self.consume_frontier(&frontier) {
                continue;
            }
            return Err("zx: no extractable vertex — the diagram has no gflow".into());
        }
        Err("zx: extraction hit its iteration fence".into())
    }

    /// A frontier spider that is a phase-free degree-2 wire is removed, which
    /// walks the output one step inward.
    fn consume_frontier(&mut self, frontier: &[(usize, usize)]) -> bool {
        let mut found = false;
        for &(_, v) in frontier {
            if self.check_remove_id(v) {
                self.remove_id(v);
                found = true;
            }
        }
        found
    }

    /// Gaussian elimination on the frontier's biadjacency matrix. The row
    /// operations that isolate one frontier spider ARE the CNOTs, and the
    /// √2 they move is read off the same popcount that performs them.
    fn frontier_gauss(
        &mut self,
        frontier: &[(usize, usize)],
        circuit: &mut std::collections::VecDeque<Surface>,
    ) -> Result<(), String> {
        // Columns: every spider the frontier reaches by a Hadamard edge.
        let mut cols: Vec<usize> = Vec::new();
        for &(_, v) in frontier {
            for w in bits_of(&self.had[v]) {
                if !cols.contains(&w) {
                    cols.push(w);
                }
            }
        }
        if cols.is_empty() {
            return Err("zx: frontier has no interior neighbours".into());
        }
        let mut m = BitMat::zero(frontier.len(), cols.len());
        for (i, &(_, v)) in frontier.iter().enumerate() {
            for (j, &w) in cols.iter().enumerate() {
                if self.is_h(v, w) {
                    m.set(i, j);
                }
            }
        }
        let mut reduced = m.clone();
        let mut ops = BitMat::identity(frontier.len());
        gauss_jordan(&mut reduced, &mut ops);

        // The extractable rows are the weight-1 ones; take the one whose
        // recipe touches the fewest frontier qubits, so the fewest CNOTs.
        let mut best: Option<(u32, usize)> = None;
        for i in 0..reduced.rows {
            if reduced.row_weight(i) == 1 {
                let w = ops.row_weight(i);
                if best.is_none_or(|(bw, _)| w < bw) {
                    best = Some((w, i));
                }
            }
        }
        let Some((_, row)) = best else {
            return Err("zx: no frontier vertex can be isolated — the diagram has no gflow".into());
        };
        let recipe = ops.row_bits(row);
        if recipe.len() < 2 {
            return Err("zx: frontier elimination made no progress".into());
        }
        // The control is the row we accumulate into; every other row in the
        // recipe is a CNOT target.
        let control = recipe[0];
        let (cq, cv) = frontier[control];
        for &i in &recipe[1..] {
            let (tq, tv) = frontier[i];
            let mask = self.had[tv].clone();
            let size = mask.iter().map(|w| w.count_ones()).sum::<u32>() as i32;
            let destroyed = self.xor_neighbourhood(cv, &mask) as i32;
            // √2^{destroyed − created}, created = size − destroyed
            self.scalar.m += 2 * destroyed - size;
            circuit.push_front(Surface::Cx(cq, tq));
        }
        Ok(())
    }

    /// The residual diagram is a PERMUTATION of wires — every output reaches
    /// exactly one input and vice versa, because a circuit is unitary. Sort
    /// it to the identity; each transposition is a SWAP gate.
    ///
    /// This deliberately does NOT do general Gaussian elimination. The first
    /// version did, mirroring the reference implementation, and mutation
    /// testing showed its CNOT branch was UNREACHABLE: a permutation matrix
    /// has exactly one 1 per column, so after the swap there is never a
    /// second row to clear, and reversing that CNOT's direction changed
    /// nothing any test could see. An unreachable branch is an untestable
    /// branch, so the condition that would have entered it is a REFUSAL now —
    /// if it ever fires the diagram was not a unitary circuit, and the caller
    /// needs to hear that rather than receive gates.
    fn permutation_to_gates(
        &self,
        circuit: &mut std::collections::VecDeque<Surface>,
    ) -> Result<(), String> {
        let nq = self.outputs.len();
        // src[q] = the input feeding output q
        let mut src = vec![usize::MAX; nq];
        for (i, &o) in self.outputs.iter().enumerate() {
            let nbrs = self.neighbours(o);
            if nbrs.len() != 1 {
                return Err(format!("zx: residual output {o} has degree {}", nbrs.len()));
            }
            let inp = nbrs[0];
            if self.is_h(o, inp) {
                return Err("zx: residual wire is not a plain edge".into());
            }
            let Some(j) = self.inputs.iter().position(|&x| x == inp) else {
                return Err(format!("zx: residual output {o} is not wired to an input"));
            };
            if src.contains(&j) {
                return Err("zx: two outputs share an input — the diagram is not unitary".into());
            }
            src[i] = j;
        }
        // Sort `src` to the identity by transpositions; each one is a SWAP.
        for q in 0..nq {
            if src[q] == q {
                continue;
            }
            let Some(r) = (q + 1..nq).find(|&r| src[r] == q) else {
                return Err("zx: residual wiring is not a permutation".into());
            };
            src.swap(q, r);
            circuit.push_front(Surface::Swap(q, r));
        }
        Ok(())
    }
}

/// Reduce a circuit as a ZX diagram and hand it back as a circuit.
pub fn extract_circuit(n: usize, surface: &[Surface]) -> Result<Extraction, String> {
    let mut g = from_surface(n, surface)?;
    g.full_reduce();
    g.extract()
}

/// The ADJOINT of a surface circuit: reverse the order, invert each gate.
/// Every gate in the Clifford+T surface alphabet has an inverse in it.
pub fn adjoint(surface: &[Surface]) -> Vec<Surface> {
    use Surface::*;
    surface
        .iter()
        .rev()
        .map(|&g| match g {
            S(q) => Sdg(q),
            Sdg(q) => S(q),
            T(q) => Tdg(q),
            Tdg(q) => T(q),
            Sx(q) => Sxdg(q),
            Sxdg(q) => Sx(q),
            DiagPow(k, q) => DiagPow(-k, q),
            RzPow(k, q) => RzPow(-k, q),
            Face(s, q) => Face(-s, q),
            // self-inverse: X, Y, Z, H, CX, CZ, SWAP, CCX, CCZ
            other => other,
        })
        .collect()
}

/// CERTIFY AN EXTRACTION AT ANY SIZE.
///
/// `tests/zx.rs` checks extraction against the certified runner entry by
/// entry over the whole 2^n × 2^n matrix — but only to five qubits, because
/// the runner sums branches and the benchmark circuits carry hundreds of T
/// gates. This is the check that does not care about size: compose the
/// original circuit with the ADJOINT of its own extraction and reduce. If
/// extraction was exact the composite is the identity times a scalar, and
/// reduction says so; the returned scalar must equal the extraction's.
///
/// This is not a tautology — the composite goes through the reduction path,
/// which never calls the extractor, so it is an independent reader of the
/// extractor's output. It is the same shape as quizx's own `full1` test.
pub fn certify_extraction(n: usize, surface: &[Surface]) -> Result<Cyc, String> {
    let ex = extract_circuit(n, surface)?;
    let mut composed = surface.to_vec();
    composed.extend(adjoint(&ex.gates));
    let mut g = from_surface(n, &composed)?;
    g.full_reduce();
    if !g.is_identity_wiring() {
        return Err(format!(
            "extraction is NOT exact: circuit · extracted† reduced to {} spiders with T-count {}, \
             not to the identity",
            g.n_spiders(),
            g.t_count()
        ));
    }
    // circuit = s·extracted  ⟹  circuit · extracted† = s·(extracted·extracted†) = s·I,
    // and the composite's own scalar is that s.
    if !cyc_eq(g.scalar, ex.scalar) {
        return Err(format!(
            "extraction's scalar {:?} disagrees with the composite's {:?}",
            ex.scalar.to_complex(),
            g.scalar.to_complex()
        ));
    }
    Ok(ex.scalar)
}


// ------------------------------------------------------------- construction

/// Circuit → graph-like ZX diagram, natively all-Z.
///
/// Each qubit's world line is a chain of spiders; a Hadamard is an EDGE
/// DECORATION carried in `pending_h` and absorbed into the next edge rather
/// than becoming a vertex. `CX(a,b) = H_b · CZ(a,b) · H_b` and CZ is one
/// Hadamard edge, so the construction never produces an X spider and no
/// colour-change pass is needed. Consecutive diagonal gates with no Hadamard
/// between them fuse ON THE WAY IN, which is why the graph never has to be
/// built at gate scale.
pub fn from_core(n: usize, gates: &[Gate]) -> ZxGraph {
    // A TIGHT vertex bound, because the bit-packed word width is set from it
    // and every early sweep pays that width per row: three per qubit (both
    // boundaries and the opening spider), two per CX, at most one per phase
    // gate, and ZERO per Hadamard — a Hadamard is an edge decoration here.
    let n_cx = gates.iter().filter(|g| matches!(g, Gate::Cx(..))).count();
    let n_phase = gates.iter().filter(|g| !matches!(g, Gate::Cx(..) | Gate::H(_))).count();
    let mut g = ZxGraph::with_capacity(3 * n + 2 * n_cx + n_phase + 8);
    let mut frontier = Vec::with_capacity(n);
    for _ in 0..n {
        let b = g.add_vertex(0, true);
        let v = g.add_vertex(0, false);
        g.set_edge(b, v, Some(false));
        g.inputs.push(b);
        frontier.push(v);
    }
    let mut pending_h = vec![false; n];

    // Extend qubit q's wire by a fresh spider, absorbing any pending
    // Hadamard into the new edge; returns the spider that now ends the wire.
    fn extend(
        g: &mut ZxGraph,
        frontier: &mut [usize],
        pending_h: &mut [bool],
        q: usize,
    ) -> usize {
        let v = g.add_vertex(0, false);
        g.set_edge(frontier[q], v, Some(pending_h[q]));
        pending_h[q] = false;
        frontier[q] = v;
        v
    }

    // A diagonal phase either lands on the current spider (no Hadamard has
    // intervened, so the two fuse) or opens a new one.
    fn phase_on(
        g: &mut ZxGraph,
        frontier: &mut [usize],
        pending_h: &mut [bool],
        q: usize,
        k: i64,
    ) {
        let v = if pending_h[q] { extend(g, frontier, pending_h, q) } else { frontier[q] };
        g.phase[v] = (g.phase[v] + k).rem_euclid(8);
    }

    for &gate in gates {
        match gate {
            Gate::H(q) => pending_h[q] = !pending_h[q],
            Gate::Z(q) => phase_on(&mut g, &mut frontier, &mut pending_h, q, 4),
            Gate::S(q) => phase_on(&mut g, &mut frontier, &mut pending_h, q, 2),
            Gate::Sdg(q) => phase_on(&mut g, &mut frontier, &mut pending_h, q, 6),
            Gate::T(q) => phase_on(&mut g, &mut frontier, &mut pending_h, q, 1),
            Gate::Tdg(q) => phase_on(&mut g, &mut frontier, &mut pending_h, q, 7),
            // X = H·Z·H, so it is a π between two edge decorations.
            Gate::X(q) => {
                pending_h[q] = !pending_h[q];
                phase_on(&mut g, &mut frontier, &mut pending_h, q, 4);
                pending_h[q] = !pending_h[q];
            }
            // CX(a,b) = H_b · CZ(a,b) · H_b, and CZ is one Hadamard edge.
            //
            // The √2 is not decoration. A Hadamard EDGE is the NORMALIZED
            // Hadamard, 2^{-1/2}(−1)^{s_u s_v} — which is exactly the H GATE,
            // so an H gate costs nothing — but CZ is diag(1,1,1,−1) with no
            // normalization, so building it out of one H edge under-counts by
            // 2^{-1/2} and the diagram owes a √2 back.
            Gate::Cx(a, b) => {
                pending_h[b] = !pending_h[b];
                let va = extend(&mut g, &mut frontier, &mut pending_h, a);
                let vb = extend(&mut g, &mut frontier, &mut pending_h, b);
                g.set_edge(va, vb, Some(true));
                g.scalar.m -= 1;
                pending_h[b] = !pending_h[b];
            }
        }
    }
    for q in 0..n {
        let b = g.add_vertex(0, true);
        g.set_edge(frontier[q], b, Some(pending_h[q]));
        g.outputs.push(b);
    }
    g
}

/// Surface program → graph-like ZX diagram. Surface gates are lowered through
/// `qasm::lower` first, so the whole superset alphabet (CCZ, CCX, SWAP, …) is
/// in scope; the returned `i64` is the lowering's ζ16 global-phase power,
/// which the diagram does NOT carry (it is a global phase, and `lower`'s
/// callers already account for it).
pub fn from_surface_with_phase(n: usize, surface: &[Surface]) -> Result<(ZxGraph, i64), String> {
    if let Some(bad) = surface.iter().find(|g| matches!(g, Surface::Face(..) | Surface::Rot(_))) {
        return Err(format!(
            "zx: {bad:?} is outside the graph-rewriting fragment (Clifford+T only); \
             route it to the face/symbolic engines"
        ));
    }
    let (core, phase16) = crate::qasm::lower(surface);
    Ok((from_core(n, &core), phase16))
}

pub fn from_surface(n: usize, surface: &[Surface]) -> Result<ZxGraph, String> {
    from_surface_with_phase(n, surface).map(|(g, _)| g)
}

/// T-count after CLIFFORD simplification only. Kept because it is the honest
/// control for `full_reduced_t_count`: by the parity theorem it can never
/// move, and a run in which it does is a bug in the Clifford layer.
pub fn simplified_t_count(n: usize, surface: &[Surface]) -> Result<usize, String> {
    let mut g = from_surface(n, surface)?;
    g.clifford_simp();
    Ok(g.t_count())
}

/// T-count after FULL reduction of the OPEN circuit diagram (boundaries left
/// free) — the reduction available to a circuit optimiser.
pub fn full_reduced_t_count(n: usize, surface: &[Surface]) -> Result<usize, String> {
    let mut g = from_surface(n, surface)?;
    g.full_reduce();
    Ok(g.t_count())
}

/// T-count after full reduction of the CLOSED diagram `⟨y|C|x⟩` — the metric
/// that prices a stabiliser decomposition, and the one BENCHMARKS entry
/// sixteen measured quizx by.
pub fn amplitude_t_count(
    n: usize,
    surface: &[Surface],
    x: &[bool],
    y: &[bool],
) -> Result<usize, String> {
    let mut g = from_surface(n, surface)?;
    g.plug_inputs(x);
    g.plug_outputs(y);
    g.full_reduce();
    Ok(g.t_count())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qasm::Surface::*;

    #[test]
    fn graph_builds_and_counts_t_gates() {
        let prog = vec![H(0), T(0), Cx(0, 1), T(1), H(1), T(0)];
        let g = from_surface(2, &prog).unwrap();
        assert_eq!(g.t_count(), 3, "three odd-phase spiders");
    }

    #[test]
    fn simplification_never_increases_t_count() {
        let progs: Vec<(usize, Vec<Surface>)> = vec![
            (2, vec![H(0), T(0), Cx(0, 1), Tdg(1), H(1), S(0)]),
            (3, vec![H(0), H(1), Cz(0, 1), T(2), Cx(2, 0), S(1), T(0), H(2)]),
            (3, vec![T(0), T(0), Cx(0, 1), Cx(0, 1), Tdg(0), Tdg(0)]),
        ];
        for (n, p) in progs {
            let before = from_surface(n, &p).unwrap().t_count();
            let after = full_reduced_t_count(n, &p).unwrap();
            assert!(after <= before, "T-count grew: {before} → {after}");
        }
    }

    #[test]
    fn local_complementation_and_pivot_preserve_phase_parity() {
        // The parity theorem, as a test, on the two rules it is about: build
        // a diagram, run ONLY lcomp and pivot, and the T-count must not move.
        let p = vec![H(0), T(0), Cx(0, 1), S(1), H(1), T(1), Cz(0, 1), Tdg(0), H(0), T(1)];
        let mut g = from_surface(2, &p).unwrap();
        let before = g.t_count();
        g.pivot_simp();
        g.local_comp_simp();
        assert_eq!(g.t_count(), before, "a Clifford graph rewrite changed a phase parity");
    }

    #[test]
    fn non_clifford_t_gates_are_refused_not_approximated() {
        let e = from_surface(1, &[Rot(0)]).unwrap_err();
        assert!(e.contains("outside the graph-rewriting fragment"));
    }

    #[test]
    fn gadgets_actually_appear() {
        // The regression that names the old defect: full reduction on a
        // circuit with repeated T's on a shared support must MAKE gadgets.
        let p = vec![T(0), Cx(1, 0), T(0), Cx(1, 0), T(0), T(1)];
        let mut g = from_surface(2, &p).unwrap();
        g.plug_inputs(&[false, false]);
        g.plug_outputs(&[false, false]);
        g.clifford_simp();
        g.gen_pivot_simp();
        assert!(g.gadget_stats().0 > 0 || g.t_count() < 4, "no gadget was ever produced");
    }
}
