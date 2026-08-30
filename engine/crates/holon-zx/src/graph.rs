//! ZX-calculus Graph representation and core operations.
//!
//! Represents ZX diagrams with Z spiders, X spiders, and boundary vertices,
//! connected by Normal (plain) or Hadamard (dashed) edges.
//! Supports exact phases in units of π/4 (mod 8) for Clifford+T circuits,
//! and exact algebraic scalars in Z[ω]·2^{-m/2} (via `holon::ledger::Cyc`).

use holon::affine::Gate;
use holon::ledger::Cyc;
use holon::qasm::Surface;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpiderType {
    Z,
    X,
    Boundary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeType {
    Normal,
    Hadamard,
}

/// Bit manipulation helpers for compact adjacency.
#[inline]
pub(crate) fn bit(v: &[u64], i: usize) -> bool {
    v[i >> 6] >> (i & 63) & 1 == 1
}

#[inline]
pub(crate) fn set_bit(v: &mut [u64], i: usize, on: bool) {
    if on {
        v[i >> 6] |= 1u64 << (i & 63);
    } else {
        v[i >> 6] &= !(1u64 << (i & 63));
    }
}

pub(crate) fn bits_of(mask: &[u64]) -> Vec<usize> {
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

/// `ω^k` exactly, ω = e^{iπ/4}; reduction `ω⁴ = −1`.
#[inline]
pub fn omega(k: i64) -> Cyc {
    let k = k.rem_euclid(8) as usize;
    let mut c = [0i128; 4];
    if k < 4 {
        c[k] = 1;
    } else {
        c[k - 4] = -1;
    }
    Cyc { c, m: 0 }
}

/// Exact ring equality by difference: a - b == 0.
pub fn cyc_eq(a: Cyc, b: Cyc) -> bool {
    a.add(b.mul(omega(4))).c.iter().all(|&x| x == 0)
}

/// A ZX diagram supporting Z spiders, X spiders, and boundary vertices,
/// with Normal and Hadamard edges, and exact scalar tracking.
#[derive(Clone, Debug)]
pub struct ZxGraph {
    /// Spider types: Z, X, or Boundary.
    pub types: Vec<SpiderType>,
    /// Phase in units of π/4, mod 8.
    pub phase: Vec<i64>,
    /// Bit-packed adjacency: `adj[v]` has bit `u` set iff an edge (v, u) exists.
    pub adj: Vec<Vec<u64>>,
    /// Bit-packed Hadamard flags: `had[v]` has bit `u` set iff (v, u) is a Hadamard edge.
    pub had: Vec<Vec<u64>>,
    /// Tombstone flags for vertex deletion.
    pub alive: Vec<bool>,
    pub words: usize,
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    /// The diagram's exact global scalar.
    pub scalar: Cyc,
}

impl ZxGraph {
    pub fn with_capacity(cap: usize) -> Self {
        let words = cap.div_ceil(64).max(1);
        ZxGraph {
            types: Vec::with_capacity(cap),
            phase: Vec::with_capacity(cap),
            adj: Vec::with_capacity(cap),
            had: Vec::with_capacity(cap),
            alive: Vec::with_capacity(cap),
            words,
            inputs: Vec::new(),
            outputs: Vec::new(),
            scalar: Cyc::ONE,
        }
    }

    pub fn add_vertex(&mut self, stype: SpiderType, phase: i64) -> usize {
        let v = self.types.len();
        if v >= self.words * 64 {
            self.words = (v >> 6) + 1;
            for a in self.adj.iter_mut() {
                a.push(0);
            }
            for h in self.had.iter_mut() {
                h.push(0);
            }
        }
        self.types.push(stype);
        self.phase.push(phase.rem_euclid(8));
        self.alive.push(true);
        self.adj.push(vec![0u64; self.words]);
        self.had.push(vec![0u64; self.words]);
        v
    }

    pub fn add_boundary(&mut self) -> usize {
        self.add_vertex(SpiderType::Boundary, 0)
    }

    pub fn add_z_spider(&mut self, phase: i64) -> usize {
        self.add_vertex(SpiderType::Z, phase)
    }

    pub fn add_x_spider(&mut self, phase: i64) -> usize {
        self.add_vertex(SpiderType::X, phase)
    }

    #[inline]
    pub fn is_boundary(&self, v: usize) -> bool {
        self.types[v] == SpiderType::Boundary
    }

    #[inline]
    pub fn is_z(&self, v: usize) -> bool {
        self.types[v] == SpiderType::Z
    }

    #[inline]
    pub fn is_x(&self, v: usize) -> bool {
        self.types[v] == SpiderType::X
    }

    #[inline]
    pub fn has_edge(&self, u: usize, v: usize) -> bool {
        bit(&self.adj[u], v)
    }

    #[inline]
    pub fn is_h(&self, u: usize, v: usize) -> bool {
        bit(&self.had[u], v)
    }

    pub fn edge_type(&self, u: usize, v: usize) -> Option<EdgeType> {
        if !self.has_edge(u, v) {
            None
        } else if self.is_h(u, v) {
            Some(EdgeType::Hadamard)
        } else {
            Some(EdgeType::Normal)
        }
    }

    pub fn set_edge(&mut self, u: usize, v: usize, e: Option<EdgeType>) {
        debug_assert_ne!(u, v, "self-loops are reduced into scalars/phases");
        let (present, h) = match e {
            None => (false, false),
            Some(EdgeType::Normal) => (true, false),
            Some(EdgeType::Hadamard) => (true, true),
        };
        set_bit(&mut self.adj[u], v, present);
        set_bit(&mut self.adj[v], u, present);
        set_bit(&mut self.had[u], v, h);
        set_bit(&mut self.had[v], u, h);
    }

    pub fn add_edge(&mut self, u: usize, v: usize, etype: EdgeType) {
        self.add_edge_smart(u, v, etype == EdgeType::Hadamard);
    }

    pub fn neighbours(&self, v: usize) -> Vec<usize> {
        bits_of(&self.adj[v])
    }

    pub fn degree(&self, v: usize) -> usize {
        self.adj[v].iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn remove(&mut self, v: usize) {
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

    /// Smart edge addition accounting for parallel edge simplifications and self-loops.
    pub fn add_edge_smart(&mut self, u: usize, v: usize, new_h: bool) {
        if u == v {
            debug_assert!(!self.is_boundary(u), "no self-loops on a boundary");
            if new_h {
                self.phase[u] = (self.phase[u] + 4).rem_euclid(8);
                self.scalar.m += 1; // ×√2⁻¹
            }
            return;
        }
        match (self.has_edge(u, v), self.is_h(u, v), new_h) {
            (false, _, h) => {
                self.set_edge(u, v, if h { Some(EdgeType::Hadamard) } else { Some(EdgeType::Normal) })
            }
            (true, false, false) => {}
            (true, true, true) => {
                self.set_edge(u, v, None);
                self.scalar.m += 2; // ×√2⁻²
            }
            (true, h0, _) => {
                debug_assert!(!self.is_boundary(u), "boundaries never gain a phase");
                if h0 {
                    self.set_edge(u, v, Some(EdgeType::Normal));
                }
                self.phase[u] = (self.phase[u] + 4).rem_euclid(8);
                self.scalar.m += 1;
            }
        }
    }

    /// XOR row mask into Hadamard edges of vertex u.
    pub fn xor_row_h(&mut self, u: usize, mask: &[u64]) -> u32 {
        let mut destroyed = 0;
        for i in 0..self.words {
            let m = mask[i];
            destroyed += (self.adj[u][i] & m).count_ones();
            self.adj[u][i] ^= m;
            self.had[u][i] ^= m;
        }
        destroyed
    }

    /// Asymmetric neighbourhood toggle for frontier elimination.
    pub fn xor_neighbourhood(&mut self, v: usize, mask: &[u64]) -> u32 {
        let destroyed = self.xor_row_h(v, mask);
        for w in bits_of(mask) {
            let on = bit(&self.adj[v], w);
            set_bit(&mut self.adj[w], v, on);
            set_bit(&mut self.had[w], v, on);
        }
        destroyed
    }

    /// Convert any X-spiders in the diagram to Z-spiders via the color-change rule:
    /// X(α) with incident edges becomes Z(α) with all incident edge types toggled
    /// (Normal <-> Hadamard).
    pub fn to_graph_like(&mut self) {
        for v in 0..self.types.len() {
            if self.alive[v] && self.types[v] == SpiderType::X {
                self.types[v] = SpiderType::Z;
                let nbrs = self.neighbours(v);
                for u in nbrs {
                    let was_h = self.is_h(v, u);
                    set_bit(&mut self.had[v], u, !was_h);
                    set_bit(&mut self.had[u], v, !was_h);
                }
            }
        }
    }

    /// Reclaim tombstones and re-index vertices compactly.
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
        let mut types = Vec::with_capacity(k);
        let mut phase = Vec::with_capacity(k);
        let mut adj = Vec::with_capacity(k);
        let mut had = Vec::with_capacity(k);
        for v in 0..n {
            if !self.alive[v] {
                continue;
            }
            types.push(self.types[v]);
            phase.push(self.phase[v]);
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
        self.types = types;
        self.phase = phase;
        self.adj = adj;
        self.had = had;
        self.words = words;
    }

    /// Count of non-Clifford spiders (odd multiples of π/4).
    pub fn t_count(&self) -> usize {
        (0..self.phase.len())
            .filter(|&v| self.alive[v] && !self.is_boundary(v) && self.phase[v].rem_euclid(2) == 1)
            .count()
    }

    pub fn n_spiders(&self) -> usize {
        (0..self.types.len()).filter(|&v| self.alive[v]).count()
    }

    pub fn is_closed(&self) -> bool {
        (0..self.types.len()).all(|v| !self.alive[v] || !self.is_boundary(v))
    }

    pub fn plug(&mut self, b: usize, val: bool) {
        let nbrs = self.neighbours(b);
        debug_assert_eq!(nbrs.len(), 1, "a boundary has exactly one edge");
        let u = nbrs[0];
        let h = self.is_h(b, u);
        self.types[b] = SpiderType::Z;
        self.phase[b] = if val { 4 } else { 0 };
        self.set_edge(
            b,
            u,
            if !h {
                Some(EdgeType::Hadamard)
            } else {
                Some(EdgeType::Normal)
            },
        );
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

    pub fn is_identity_wiring(&self) -> bool {
        if self.inputs.len() != self.outputs.len() {
            return false;
        }
        let live: Vec<usize> = (0..self.types.len()).filter(|&v| self.alive[v]).collect();
        if live.len() != self.inputs.len() + self.outputs.len() {
            return false;
        }
        if live.iter().any(|&v| !self.is_boundary(v)) {
            return false;
        }
        self.inputs.iter().zip(&self.outputs).all(|(&i, &o)| {
            self.degree(i) == 1 && self.degree(o) == 1 && self.has_edge(i, o) && !self.is_h(i, o)
        })
    }

    /// Evaluates the exact value of a closed diagram using bucket elimination.
    pub fn eval(&self) -> Cyc {
        assert!(
            self.is_closed(),
            "eval needs a closed diagram: plug the boundaries first"
        );
        let n = self.phase.len();

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

        let mut factors: Vec<(Vec<usize>, Vec<Cyc>)> = Vec::new();
        for a in 0..nvars {
            factors.push((vec![a], vec![Cyc::ONE, omega(phase[a])]));
        }
        for (&(a, b), &k) in mult.iter() {
            scalar.m += k as i32;
            let sign = if k % 2 == 0 { Cyc::ONE } else { omega(4) };
            factors.push((vec![a, b], vec![Cyc::ONE, Cyc::ONE, Cyc::ONE, sign]));
        }

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
                scalar = scalar.mul(Cyc {
                    c: [2, 0, 0, 0],
                    m: 0,
                });
                continue;
            }
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
                let pos: Vec<usize> = vs
                    .iter()
                    .map(|x| full.iter().position(|y| y == x).unwrap())
                    .collect();
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

/// Convert a core gate sequence into a graph-like ZxGraph.
pub fn from_core(n: usize, gates: &[Gate]) -> ZxGraph {
    let n_cx = gates.iter().filter(|g| matches!(g, Gate::Cx(..))).count();
    let n_phase = gates
        .iter()
        .filter(|g| !matches!(g, Gate::Cx(..) | Gate::H(_)))
        .count();
    let mut g = ZxGraph::with_capacity(3 * n + 2 * n_cx + n_phase + 8);
    let mut frontier = Vec::with_capacity(n);
    for _ in 0..n {
        let b = g.add_boundary();
        let v = g.add_z_spider(0);
        g.set_edge(b, v, Some(EdgeType::Normal));
        g.inputs.push(b);
        frontier.push(v);
    }
    let mut pending_h = vec![false; n];

    fn extend(
        g: &mut ZxGraph,
        frontier: &mut [usize],
        pending_h: &mut [bool],
        q: usize,
    ) -> usize {
        let v = g.add_z_spider(0);
        g.set_edge(
            frontier[q],
            v,
            if pending_h[q] {
                Some(EdgeType::Hadamard)
            } else {
                Some(EdgeType::Normal)
            },
        );
        pending_h[q] = false;
        frontier[q] = v;
        v
    }

    fn phase_on(
        g: &mut ZxGraph,
        frontier: &mut [usize],
        pending_h: &mut [bool],
        q: usize,
        k: i64,
    ) {
        let v = if pending_h[q] {
            extend(g, frontier, pending_h, q)
        } else {
            frontier[q]
        };
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
            Gate::X(q) => {
                pending_h[q] = !pending_h[q];
                phase_on(&mut g, &mut frontier, &mut pending_h, q, 4);
                pending_h[q] = !pending_h[q];
            }
            Gate::Cx(a, b) => {
                pending_h[b] = !pending_h[b];
                let va = extend(&mut g, &mut frontier, &mut pending_h, a);
                let vb = extend(&mut g, &mut frontier, &mut pending_h, b);
                g.set_edge(va, vb, Some(EdgeType::Hadamard));
                g.scalar.m -= 1;
                pending_h[b] = !pending_h[b];
            }
        }
    }
    for q in 0..n {
        let b = g.add_boundary();
        g.set_edge(
            frontier[q],
            b,
            if pending_h[q] {
                Some(EdgeType::Hadamard)
            } else {
                Some(EdgeType::Normal)
            },
        );
        g.outputs.push(b);
    }
    g
}

/// Convert a surface gate sequence into a ZxGraph and its lowering phase (mod 16).
pub fn from_surface_with_phase(n: usize, surface: &[Surface]) -> Result<(ZxGraph, i64), String> {
    if let Some(bad) = surface
        .iter()
        .find(|g| matches!(g, Surface::Face(..) | Surface::Rot(_)))
    {
        return Err(format!(
            "zx: {bad:?} is outside the graph-rewriting fragment (Clifford+T only)"
        ));
    }
    let (core, phase16) = holon::qasm::lower(surface);
    Ok((from_core(n, &core), phase16))
}

/// Convert a surface gate sequence into a ZxGraph.
pub fn from_surface(n: usize, surface: &[Surface]) -> Result<ZxGraph, String> {
    let (g, _phase16) = from_surface_with_phase(n, surface)?;
    Ok(g)
}
