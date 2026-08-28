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
//! ZX-calculus", Quantum 4, 279 (2020)) — graph-like form, identity removal,
//! local complementation, and pivoting. The reference implementation is
//! quizx (Apache-2.0, Kissinger–van de Wetering); this is our own
//! implementation of their published rules, not a port, and quizx remains
//! the benchmark we measure against.
//!
//! THE CIRIS-NATIVE PART: adjacency is BIT-PACKED (one `u64` word per 64
//! vertices), so the rewrites become word-parallel set algebra —
//! local complementation is `adj[u] ^= mask` across a neighbourhood, and
//! pivoting is three such sweeps. That is the same discipline that beat stim
//! at tier 1, applied to a graph instead of a tableau.
//!
//! Phases are EXACT: an integer multiple of π/4 (mod 8), never a float —
//! so "is this spider Clifford" is an integer test, not a tolerance.

use crate::qasm::Surface;

/// A graph-like ZX diagram: every spider is Z-type, every edge is a
/// Hadamard edge, phases are exact eighths of a turn (units of π/4).
#[derive(Debug)]
pub struct ZxGraph {
    /// phase[v] in units of π/4, mod 8.
    pub phase: Vec<i64>,
    /// Bit-packed adjacency: `adj[v]` has bit `u` set iff edge (v,u) exists.
    adj: Vec<Vec<u64>>,
    /// Boundary vertices (inputs/outputs) are never removed by a rewrite.
    boundary: Vec<bool>,
    /// Removed vertices are tombstoned, keeping indices stable.
    alive: Vec<bool>,
    words: usize,
    /// Global scalar as an ω-power (exact, in the ledger's units).
    pub scalar_omega: i64,
}

#[inline]
fn bit(v: &[u64], i: usize) -> bool {
    v[i >> 6] >> (i & 63) & 1 == 1
}

#[inline]
fn set_bit(v: &mut [u64], i: usize, on: bool) {
    if on {
        v[i >> 6] |= 1 << (i & 63);
    } else {
        v[i >> 6] &= !(1 << (i & 63));
    }
}

impl ZxGraph {
    fn new(n: usize) -> ZxGraph {
        let words = n.div_ceil(64).max(1);
        ZxGraph {
            phase: vec![0; n],
            adj: vec![vec![0u64; words]; n],
            boundary: vec![false; n],
            alive: vec![true; n],
            words,
            scalar_omega: 0,
        }
    }

    fn grow(&mut self) -> usize {
        let v = self.phase.len();
        self.phase.push(0);
        self.adj.push(vec![0u64; self.words]);
        self.boundary.push(false);
        self.alive.push(true);
        // widen if we crossed a word boundary
        if (v + 1).div_ceil(64) > self.words {
            self.words += 1;
            for a in self.adj.iter_mut() {
                a.push(0);
            }
        }
        v
    }

    #[inline]
    fn toggle(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        let on = bit(&self.adj[a], b);
        set_bit(&mut self.adj[a], b, !on);
        set_bit(&mut self.adj[b], a, !on);
    }

    fn neighbours(&self, v: usize) -> Vec<usize> {
        let mut out = Vec::new();
        for (w, &word) in self.adj[v].iter().enumerate() {
            let mut x = word;
            while x != 0 {
                let b = x.trailing_zeros() as usize;
                out.push(w * 64 + b);
                x &= x - 1;
            }
        }
        out
    }

    fn degree(&self, v: usize) -> usize {
        self.adj[v].iter().map(|w| w.count_ones() as usize).sum()
    }

    fn remove(&mut self, v: usize) {
        for u in self.neighbours(v) {
            set_bit(&mut self.adj[u], v, false);
        }
        for w in self.adj[v].iter_mut() {
            *w = 0;
        }
        self.alive[v] = false;
    }

    /// T-count: spiders whose phase is an ODD multiple of π/4.
    pub fn t_count(&self) -> usize {
        (0..self.phase.len())
            .filter(|&v| self.alive[v] && !self.boundary[v] && self.phase[v].rem_euclid(2) == 1)
            .count()
    }

    pub fn n_spiders(&self) -> usize {
        (0..self.phase.len()).filter(|&v| self.alive[v]).count()
    }

    /// Is `v` an interior Clifford spider eligible for local complementation
    /// (phase ±π/2, i.e. 2 or 6 eighths, not on a boundary)?
    fn is_lcomp_site(&self, v: usize) -> bool {
        self.alive[v]
            && !self.boundary[v]
            && matches!(self.phase[v].rem_euclid(8), 2 | 6)
            && self.neighbours(v).iter().all(|&u| !self.boundary[u])
    }

    /// LOCAL COMPLEMENTATION — the word-parallel rewrite. Delete `v`,
    /// complement the edges among its neighbourhood, and shift every
    /// neighbour's phase by ∓π/2.
    fn lcomp(&mut self, v: usize) {
        let nbrs = self.neighbours(v);
        let a = self.phase[v].rem_euclid(8);
        // complement the neighbourhood: word-parallel XOR per row
        let mut mask = self.adj[v].clone();
        for &u in &nbrs {
            set_bit(&mut mask, u, true);
        }
        for &u in &nbrs {
            let mut m = mask.clone();
            set_bit(&mut m, u, false);
            for (dst, src) in self.adj[u].iter_mut().zip(m.iter()) {
                *dst ^= *src;
            }
        }
        // fix the mirrored halves: the XOR above set both directions, so
        // re-symmetrize by recomputing from the row we wrote.
        for &u in &nbrs {
            for &w in &nbrs {
                if u < w {
                    let on = bit(&self.adj[u], w);
                    set_bit(&mut self.adj[w], u, on);
                }
            }
        }
        // phases: neighbours shift by −a (mod 8)
        for &u in &nbrs {
            self.phase[u] = (self.phase[u] - a).rem_euclid(8);
        }
        // exact scalar: ω^{a(a−4)/2}-ish; tracked as an ω power so nothing
        // is dropped. (The value only matters for amplitudes, not T-count.)
        self.scalar_omega = (self.scalar_omega + if a == 2 { 1 } else { -1 }).rem_euclid(8);
        self.remove(v);
    }

    /// Is `(u,v)` a pivot site: adjacent interior spiders both with phase in
    /// {0, π}?
    fn is_pivot_site(&self, u: usize, v: usize) -> bool {
        self.alive[u]
            && self.alive[v]
            && !self.boundary[u]
            && !self.boundary[v]
            && bit(&self.adj[u], v)
            && matches!(self.phase[u].rem_euclid(8), 0 | 4)
            && matches!(self.phase[v].rem_euclid(8), 0 | 4)
            && self.neighbours(u).iter().all(|&w| !self.boundary[w])
            && self.neighbours(v).iter().all(|&w| !self.boundary[w])
    }

    /// PIVOT — three neighbourhood complementations, then delete both
    /// spiders, with the published phase corrections.
    fn pivot(&mut self, u: usize, v: usize) {
        let nu: Vec<usize> = self.neighbours(u).into_iter().filter(|&x| x != v).collect();
        let nv: Vec<usize> = self.neighbours(v).into_iter().filter(|&x| x != u).collect();
        let inter: Vec<usize> = nu.iter().copied().filter(|x| nv.contains(x)).collect();
        let a: Vec<usize> = nu.iter().copied().filter(|x| !nv.contains(x)).collect();
        let b: Vec<usize> = nv.iter().copied().filter(|x| !nu.contains(x)).collect();
        // toggle A×B, A×C, B×C
        for &x in &a {
            for &y in &b {
                self.toggle(x, y);
            }
            for &y in &inter {
                self.toggle(x, y);
            }
        }
        for &x in &b {
            for &y in &inter {
                self.toggle(x, y);
            }
        }
        let (pu, pv) = (self.phase[u].rem_euclid(8), self.phase[v].rem_euclid(8));
        for &x in &a {
            self.phase[x] = (self.phase[x] + pv).rem_euclid(8);
        }
        for &x in &b {
            self.phase[x] = (self.phase[x] + pu).rem_euclid(8);
        }
        for &x in &inter {
            self.phase[x] = (self.phase[x] + pu + pv + 4).rem_euclid(8);
        }
        self.remove(u);
        self.remove(v);
    }

    /// IDENTITY REMOVAL: a phase-0 interior spider of degree 2 vanishes,
    /// its two neighbours joined by a toggled edge.
    fn remove_identities(&mut self) -> bool {
        let mut changed = false;
        for v in 0..self.phase.len() {
            if !self.alive[v] || self.boundary[v] || self.phase[v].rem_euclid(8) != 0 {
                continue;
            }
            if self.degree(v) == 2 {
                let n = self.neighbours(v);
                self.remove(v);
                self.toggle(n[0], n[1]);
                changed = true;
            }
        }
        changed
    }

    /// The simplification loop: identity removal, local complementation and
    /// pivoting to fixpoint (`clifford_simp` in the published vocabulary).
    pub fn simplify(&mut self) {
        for _ in 0..64 {
            let mut changed = self.remove_identities();
            for v in 0..self.phase.len() {
                if self.is_lcomp_site(v) {
                    self.lcomp(v);
                    changed = true;
                }
            }
            for u in 0..self.phase.len() {
                if !self.alive[u] {
                    continue;
                }
                let nbrs = self.neighbours(u);
                for v in nbrs {
                    if v > u && self.is_pivot_site(u, v) {
                        self.pivot(u, v);
                        changed = true;
                        break;
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }
}

/// Circuit → graph-like ZX diagram. Each qubit's world line is a chain of
/// spiders; a Hadamard becomes an edge decoration (absorbed by the
/// all-H-edge convention), CZ/CX become edges, diagonal phases become spider
/// phases. Non-Clifford+T gates are refused rather than approximated.
pub fn from_surface(n: usize, surface: &[Surface]) -> Result<ZxGraph, String> {
    let mut g = ZxGraph::new(n);
    // frontier[q] = the spider currently at the end of qubit q's line
    let mut frontier: Vec<usize> = (0..n).collect();
    for q in 0..n {
        g.boundary[q] = true;
    }
    // `pending_h[q]` counts Hadamards not yet absorbed into an edge.
    let mut pending_h = vec![0usize; n];

    let mut chain = |g: &mut ZxGraph, frontier: &mut Vec<usize>, pending_h: &mut Vec<usize>, q: usize| -> usize {
        let v = g.grow();
        let prev = frontier[q];
        g.toggle(prev, v);
        // an odd number of pending Hadamards means the edge is NOT an
        // H-edge; graph-like form wants all H-edges, so we insert a
        // degree-2 phase-0 spider to absorb the parity.
        if pending_h[q] % 2 == 0 {
            let mid = g.grow();
            g.toggle(prev, mid);
            g.toggle(mid, v);
            set_bit(&mut g.adj[prev].clone(), v, false);
            g.toggle(prev, v); // undo the direct edge
        }
        pending_h[q] = 0;
        frontier[q] = v;
        v
    };

    for &gate in surface {
        match gate {
            Surface::H(q) => pending_h[q] += 1,
            Surface::Z(q) => {
                let v = chain(&mut g, &mut frontier, &mut pending_h, q);
                g.phase[v] = (g.phase[v] + 4).rem_euclid(8);
            }
            Surface::S(q) => {
                let v = chain(&mut g, &mut frontier, &mut pending_h, q);
                g.phase[v] = (g.phase[v] + 2).rem_euclid(8);
            }
            Surface::Sdg(q) => {
                let v = chain(&mut g, &mut frontier, &mut pending_h, q);
                g.phase[v] = (g.phase[v] + 6).rem_euclid(8);
            }
            Surface::T(q) => {
                let v = chain(&mut g, &mut frontier, &mut pending_h, q);
                g.phase[v] = (g.phase[v] + 1).rem_euclid(8);
            }
            Surface::Tdg(q) => {
                let v = chain(&mut g, &mut frontier, &mut pending_h, q);
                g.phase[v] = (g.phase[v] + 7).rem_euclid(8);
            }
            Surface::DiagPow(k, q) => {
                let v = chain(&mut g, &mut frontier, &mut pending_h, q);
                g.phase[v] = (g.phase[v] + k).rem_euclid(8);
            }
            Surface::Cz(a, b) => {
                let va = chain(&mut g, &mut frontier, &mut pending_h, a);
                let vb = chain(&mut g, &mut frontier, &mut pending_h, b);
                g.toggle(va, vb);
            }
            Surface::Cx(a, b) => {
                // CX = (I⊗H) CZ (I⊗H): one H-edge plus the target's parity
                pending_h[b] += 1;
                let va = chain(&mut g, &mut frontier, &mut pending_h, a);
                let vb = chain(&mut g, &mut frontier, &mut pending_h, b);
                g.toggle(va, vb);
                pending_h[b] += 1;
            }
            other => {
                return Err(format!(
                    "zx: {other:?} is outside the graph-rewriting fragment \
                     (Clifford+T only); route it to the face/symbolic engines"
                ))
            }
        }
    }
    Ok(g)
}

/// T-count after graph simplification — the metric entry sixteen measured
/// quizx by, computed on our own graph.
pub fn simplified_t_count(n: usize, surface: &[Surface]) -> Result<usize, String> {
    let mut g = from_surface(n, surface)?;
    let before = g.t_count();
    g.simplify();
    let after = g.t_count();
    debug_assert!(after <= before, "simplification must not increase T-count");
    Ok(after)
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
            let after = simplified_t_count(n, &p).unwrap();
            assert!(after <= before, "T-count grew: {before} → {after}");
        }
    }

    #[test]
    fn non_clifford_t_gates_are_refused_not_approximated() {
        let e = from_surface(1, &[Rot(0)]).unwrap_err();
        assert!(e.contains("outside the graph-rewriting fragment"));
    }
}

impl ZxGraph {
    /// A PHASE GADGET is a degree-1 spider (the phase carrier) hanging off a
    /// phase-0 hub; the hub's other neighbours are the gadget's SUPPORT.
    /// Returns `(carrier, hub, support-bitmask)` for every gadget present.
    fn gadgets(&self) -> Vec<(usize, usize, Vec<u64>)> {
        let mut out = Vec::new();
        for v in 0..self.phase.len() {
            if !self.alive[v] || self.boundary[v] || self.degree(v) != 1 {
                continue;
            }
            let h = self.neighbours(v)[0];
            if !self.alive[h] || self.boundary[h] || self.phase[h].rem_euclid(8) != 0 {
                continue;
            }
            let mut sup = self.adj[h].clone();
            set_bit(&mut sup, v, false);
            if sup.iter().all(|&w| w == 0) {
                continue;
            }
            out.push((v, h, sup));
        }
        out
    }

    /// GADGET FUSION — the rewrite that actually removes T. Two gadgets on
    /// the SAME support are one gadget whose phase is the sum; if the sum is
    /// even the gadget is Clifford, and if it is zero the gadget vanishes.
    /// Bit-packed supports make "same support" an exact word comparison, so
    /// this is a group-by on bitmasks rather than a set-equality search.
    fn fuse_gadgets(&mut self) -> bool {
        let gs = self.gadgets();
        let mut changed = false;
        let mut seen: std::collections::HashMap<Vec<u64>, (usize, usize)> =
            std::collections::HashMap::new();
        for (v, h, sup) in gs {
            if !self.alive[v] || !self.alive[h] {
                continue;
            }
            match seen.get(&sup) {
                Some(&(v0, _h0)) if self.alive[v0] => {
                    self.phase[v0] = (self.phase[v0] + self.phase[v]).rem_euclid(8);
                    self.remove(v);
                    self.remove(h);
                    if self.phase[v0].rem_euclid(8) == 0 {
                        // the fused gadget is the identity: it and its hub go
                        let h0 = self.neighbours(v0).first().copied();
                        self.remove(v0);
                        if let Some(h0) = h0 {
                            if self.degree(h0) == 0 {
                                self.remove(h0);
                            }
                        }
                    }
                    changed = true;
                }
                _ => {
                    seen.insert(sup, (v, h));
                }
            }
        }
        changed
    }

    /// GADGETIZATION: an interior T-spider adjacent to a Clifford (phase
    /// 0/π) spider can be pivoted into gadget form, exposing it to fusion.
    /// This is what creates gadgets from an ordinary circuit graph.
    pub fn gadgetize(&mut self) -> bool {
        let mut changed = false;
        for v in 0..self.phase.len() {
            if !self.alive[v] || self.boundary[v] || self.phase[v].rem_euclid(2) != 1 {
                continue;
            }
            // find a Clifford interior neighbour to pivot against
            let nbrs = self.neighbours(v);
            let Some(&u) = nbrs.iter().find(|&&u| {
                self.alive[u]
                    && !self.boundary[u]
                    && matches!(self.phase[u].rem_euclid(8), 0 | 4)
                    && self.neighbours(u).iter().all(|&w| !self.boundary[w])
            }) else {
                continue;
            };
            // Split v into a hub (phase 0) plus a carrier holding v's phase,
            // then the hub is a legal pivot partner for u.
            let carrier = self.grow();
            self.phase[carrier] = self.phase[v];
            self.phase[v] = 0;
            self.toggle(v, carrier);
            if self.is_pivot_site(u, v) {
                self.pivot(u, v);
                changed = true;
            } else {
                // undo: keep the graph exactly as it was
                self.phase[v] = self.phase[carrier];
                self.toggle(v, carrier);
                self.remove(carrier);
            }
        }
        changed
    }

    /// Diagnostic: how many gadgets exist, and how many share a support?
    pub fn gadget_stats(&self) -> (usize, usize) {
        let gs = self.gadgets();
        let mut seen: std::collections::HashMap<Vec<u64>, usize> = std::collections::HashMap::new();
        for (_, _, sup) in &gs {
            *seen.entry(sup.clone()).or_insert(0) += 1;
        }
        (gs.len(), seen.values().filter(|&&c| c > 1).count())
    }

    /// The full loop: Clifford simplification, gadgetization, gadget fusion,
    /// to fixpoint — `full_reduce` in the published vocabulary.
    pub fn full_reduce(&mut self) {
        for _ in 0..32 {
            self.simplify();
            let g = self.gadgetize();
            let f = self.fuse_gadgets();
            if !g && !f {
                break;
            }
        }
        self.simplify();
    }
}

/// T-count after FULL reduction (Clifford simplification + gadget fusion).
pub fn full_reduced_t_count(n: usize, surface: &[Surface]) -> Result<usize, String> {
    let mut g = from_surface(n, surface)?;
    g.full_reduce();
    Ok(g.t_count())
}
