//! ZX-calculus graph rewriting passes.
//!
//! Implements graph simplification passes based on Duncan–Kissinger–Perdrix–van de Wetering
//! (Quantum 4, 279 (2020)) and Kissinger–van de Wetering (arXiv:1903.10477):
//!
//! 1. **Spider Fusion**: Merging same-type spiders (Z-Z and X-X) connected by Normal edges,
//!    adding their phases modulo 2π (mod 8).
//! 2. **Local Complementation**: Eliminating Clifford spiders (phase ±π/2) and inverting
//!    the Hadamard graph of their neighbourhoods.
//! 3. **Pivoting**: Eliminating pairs of adjacent interior Pauli spiders (phase 0 or π) and
//!    complemented bipartite subgraph connections.
//! 4. **Phase Gadget Extraction & Fusion**: Unfusing non-Pauli phases into gadgets,
//!    fusing gadgets on identical supports, and removing identity/scalar spiders.
//!
//! Passes compose in hierarchical drivers:
//! `interior_clifford_simp` ⊂ `clifford_simp` ⊂ `full_reduce` (`full_simp`).

use crate::graph::{bits_of, omega, set_bit, EdgeType, ZxGraph};
use holon::ledger::Cyc;
use std::collections::HashMap;

impl ZxGraph {
    // ---------------------------------------------------------------- SPIDER FUSION

    /// Check if two vertices can be merged via spider fusion.
    /// Both must be active non-boundary spiders of the SAME type connected by a Normal edge.
    pub fn check_spider_fusion(&self, u: usize, v: usize) -> bool {
        u != v
            && self.alive[u]
            && self.alive[v]
            && !self.is_boundary(u)
            && !self.is_boundary(v)
            && self.types[u] == self.types[v]
            && self.has_edge(u, v)
            && !self.is_h(u, v)
    }

    /// SPIDER FUSION: Merge two same-type spiders connected by a Normal (plain) edge.
    /// The resulting spider retains `u`'s identity with phase (phase_u + phase_v) mod 8 (mod 2π).
    /// Vertex `v` is absorbed into `u` and removed.
    pub fn spider_fusion(&mut self, u: usize, v: usize) {
        debug_assert!(
            self.check_spider_fusion(u, v),
            "spider fusion precondition failed"
        );
        for w in self.neighbours(v) {
            if w != u {
                let h = self.is_h(v, w);
                self.add_edge_smart(u, w, h);
            }
        }
        self.phase[u] = (self.phase[u] + self.phase[v]).rem_euclid(8);
        self.remove(v);
    }

    /// Apply spider fusion repeatedly until fixpoint.
    pub fn spider_simp(&mut self) -> bool {
        let mut any = false;
        loop {
            let mut matched = false;
            for u in 0..self.phase.len() {
                if !self.alive[u] || self.is_boundary(u) {
                    continue;
                }
                let hit = self
                    .neighbours(u)
                    .into_iter()
                    .find(|&v| self.check_spider_fusion(u, v));
                if let Some(v) = hit {
                    self.spider_fusion(u, v);
                    matched = true;
                }
            }
            if !matched {
                return any;
            }
            any = true;
        }
    }

    // ----------------------------------------------------------- IDENTITY REMOVAL

    pub fn check_remove_id(&self, v: usize) -> bool {
        self.alive[v]
            && !self.is_boundary(v)
            && self.phase[v].rem_euclid(8) == 0
            && self.degree(v) == 2
    }

    /// IDENTITY REMOVAL: A phase-0 spider of degree 2 is a wire.
    /// Its two incident edges compose by XOR parity (Normal ⊕ Normal = Normal, etc.).
    pub fn remove_id(&mut self, v: usize) {
        let nbrs = self.neighbours(v);
        let (a, b) = (nbrs[0], nbrs[1]);
        let e = self.is_h(v, a) ^ self.is_h(v, b);
        self.remove(v);
        self.add_edge_smart(a, b, e);
    }

    /// Apply identity removal repeatedly until fixpoint.
    pub fn id_simp(&mut self) -> bool {
        let mut any = false;
        loop {
            let mut matched = false;
            for v in 0..self.phase.len() {
                if self.check_remove_id(v) {
                    self.remove_id(v);
                    matched = true;
                }
            }
            if !matched {
                return any;
            }
            any = true;
        }
    }

    // ---------------------------------------------------- LOCAL COMPLEMENTATION

    fn all_h_to_spider(&self, v: usize) -> bool {
        self.neighbours(v)
            .into_iter()
            .all(|u| !self.is_boundary(u) && self.is_h(v, u))
    }

    pub fn check_local_comp(&self, v: usize) -> bool {
        self.alive[v]
            && !self.is_boundary(v)
            && matches!(self.phase[v].rem_euclid(8), 2 | 6)
            && self.all_h_to_spider(v)
    }

    /// LOCAL COMPLEMENTATION around a Clifford spider (phase ±π/2, i.e., 2 or 6 mod 8).
    /// Toggles the Hadamard graph on the neighbourhood N(v), shifts neighbour phases by -p,
    /// removes v, and updates the exact scalar.
    pub fn local_comp(&mut self, v: usize) {
        let p = self.phase[v].rem_euclid(8);
        let nbrs = self.neighbours(v);
        let mask = self.adj[v].clone();
        let mut destroyed = 0u32;
        for &u in &nbrs {
            let mut m = mask.clone();
            set_bit(&mut m, u, false);
            destroyed += self.xor_row_h(u, &m);
        }
        self.scalar.m += destroyed as i32;
        for &u in &nbrs {
            self.phase[u] = (self.phase[u] - p).rem_euclid(8);
        }
        self.remove(v);
        let x = nbrs.len() as i32;
        self.scalar.m -= (x - 1) * (x - 2) / 2;
        self.scalar = self.scalar.mul(omega(if p == 2 { 1 } else { -1 }));
    }

    /// Apply local complementation to all eligible Clifford spiders until fixpoint.
    pub fn local_comp_simp(&mut self) -> bool {
        let mut any = false;
        loop {
            let mut matched = false;
            for v in 0..self.phase.len() {
                if self.check_local_comp(v) {
                    self.local_comp(v);
                    matched = true;
                }
            }
            if !matched {
                return any;
            }
            any = true;
        }
    }

    // ------------------------------------------------------------------ PIVOTING

    pub fn check_pivot(&self, u: usize, v: usize) -> bool {
        u != v
            && self.alive[u]
            && self.alive[v]
            && !self.is_boundary(u)
            && !self.is_boundary(v)
            && self.has_edge(u, v)
            && self.is_h(u, v)
            && matches!(self.phase[u].rem_euclid(8), 0 | 4)
            && matches!(self.phase[v].rem_euclid(8), 0 | 4)
            && self.all_h_to_spider(u)
            && self.all_h_to_spider(v)
    }

    /// PIVOTING between adjacent interior Pauli spiders (phase 0 or π, i.e., 0 or 4 mod 8).
    /// Eliminates both spiders, toggles bipartite connections across partition sets A, B, C,
    /// adds π to C, and tracks exact scalar updates.
    pub fn pivot(&mut self, v0: usize, v1: usize) {
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
        self.scalar.m += cn;
        self.scalar.m += cn * (cn - 1);
        self.scalar.m -= (d0 - 2) * (d1 - 2);
        if p0 == 4 && p1 == 4 {
            self.scalar = self.scalar.mul(omega(4));
        }
        self.remove(v0);
        self.remove(v1);
    }

    /// Apply interior Pauli pivoting repeatedly until fixpoint.
    pub fn pivot_simp(&mut self) -> bool {
        let mut any = false;
        loop {
            let mut matched = false;
            for u in 0..self.phase.len() {
                if !self.alive[u] {
                    continue;
                }
                let hit = self
                    .neighbours(u)
                    .into_iter()
                    .find(|&v| self.check_pivot(u, v));
                if let Some(v) = hit {
                    self.pivot(u, v);
                    matched = true;
                }
            }
            if !matched {
                return any;
            }
            any = true;
        }
    }

    // ---------------------------------------------------- GENERIC PIVOT / GADGETS

    /// Unfuse a non-Pauli phase into a phase gadget (phase carrier pendant to a phase-0 hub).
    pub fn unfuse_gadget(&mut self, v: usize) {
        let p = self.phase[v].rem_euclid(8);
        if p == 0 || p == 4 {
            return;
        }
        let hub = self.add_z_spider(0);
        let carrier = self.add_z_spider(p);
        self.phase[v] = 0;
        self.set_edge(v, hub, Some(EdgeType::Hadamard));
        self.set_edge(hub, carrier, Some(EdgeType::Hadamard));
    }

    /// Push a boundary vertex one step away by inserting an identity spider.
    pub fn unfuse_boundary(&mut self, v: usize, b: usize) {
        if !self.is_boundary(b) || !self.has_edge(v, b) {
            return;
        }
        let et = self.is_h(v, b);
        let w = self.add_z_spider(0);
        self.set_edge(v, b, None);
        self.set_edge(v, w, Some(EdgeType::Hadamard));
        self.set_edge(
            w,
            b,
            if !et {
                Some(EdgeType::Hadamard)
            } else {
                Some(EdgeType::Normal)
            },
        );
    }

    fn gen_pivot_shape(&self, v: usize) -> bool {
        self.neighbours(v)
            .into_iter()
            .all(|w| self.is_boundary(w) || self.is_h(v, w))
    }

    fn is_interior_pauli(&self, v: usize) -> bool {
        matches!(self.phase[v].rem_euclid(8), 0 | 4)
            && self
                .neighbours(v)
                .into_iter()
                .all(|n| !self.is_boundary(n) && self.degree(n) > 1)
    }

    pub fn check_gen_pivot_reduce(&self, v0: usize, v1: usize) -> bool {
        v0 != v1
            && self.alive[v0]
            && self.alive[v1]
            && !self.is_boundary(v0)
            && !self.is_boundary(v1)
            && self.has_edge(v0, v1)
            && self.is_h(v0, v1)
            && self.gen_pivot_shape(v0)
            && self.gen_pivot_shape(v1)
            && (self.is_interior_pauli(v0) || self.is_interior_pauli(v1))
    }

    /// Generic pivot: repairs non-Pauli phases and boundaries, then pivots.
    pub fn gen_pivot(&mut self, v0: usize, v1: usize) {
        for &v in &[v0, v1] {
            let nbrs = self.neighbours(v);
            self.unfuse_gadget(v);
            for b in nbrs {
                self.unfuse_boundary(v, b);
            }
        }
        self.pivot(v0, v1);
    }

    pub fn gen_pivot_simp(&mut self) -> bool {
        let mut any = false;
        loop {
            let mut matched = false;
            for u in 0..self.phase.len() {
                if !self.alive[u] {
                    continue;
                }
                let hit = self
                    .neighbours(u)
                    .into_iter()
                    .find(|&v| self.check_gen_pivot_reduce(u, v));
                if let Some(v) = hit {
                    self.gen_pivot(u, v);
                    matched = true;
                }
            }
            if !matched {
                return any;
            }
            any = true;
        }
    }

    // ------------------------------------------------------------- GADGET FUSION

    /// Map phase gadgets by their support vertices.
    pub fn gadget_map(&self) -> HashMap<Vec<usize>, Vec<(usize, usize)>> {
        let mut map: HashMap<Vec<usize>, Vec<(usize, usize)>> = HashMap::new();
        for v in 0..self.phase.len() {
            if !self.alive[v] || self.is_boundary(v) || self.degree(v) != 1 {
                continue;
            }
            let hub = self.neighbours(v)[0];
            if self.is_boundary(hub) || self.phase[hub].rem_euclid(8) != 0 {
                continue;
            }
            if self.neighbours(hub).into_iter().any(|n| self.is_boundary(n)) {
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

    /// GADGET FUSION: Fuses phase gadgets with the same support into a single gadget,
    /// adding carrier phases together and reducing T-count.
    pub fn fuse_gadgets(&mut self) -> bool {
        let map = self.gadget_map();
        let mut fused = false;
        for (support, gs) in map {
            if gs.len() < 2 {
                continue;
            }
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

    /// π-copy on gadget carrier to clear π phase off gadget hub.
    pub fn pi_copy(&mut self, v: usize) {
        let p = self.phase[v].rem_euclid(8);
        self.scalar = self.scalar.mul(omega(p));
        self.phase[v] = (-p).rem_euclid(8);
        for u in self.neighbours(v) {
            self.phase[u] = (self.phase[u] + 4).rem_euclid(8);
        }
    }

    /// Clear π phases from gadget hubs.
    pub fn remove_gadget_pi(&mut self) -> bool {
        let mut hubs: HashMap<usize, usize> = HashMap::new();
        for v in 0..self.phase.len() {
            if !self.alive[v] || self.is_boundary(v) || self.degree(v) != 1 {
                continue;
            }
            let hub = self.neighbours(v)[0];
            if !self.is_boundary(hub) && self.is_h(v, hub) && self.phase[hub].rem_euclid(8) == 4 {
                hubs.insert(hub, v);
            }
        }
        let matched = !hubs.is_empty();
        for (_, carrier) in hubs {
            self.pi_copy(carrier);
        }
        matched
    }

    // ------------------------------------------------------------- SCALAR RULES

    pub fn remove_single(&mut self, v: usize) {
        let s = Cyc::ONE.add(omega(self.phase[v]));
        self.scalar = self.scalar.mul(s);
        self.remove(v);
    }

    pub fn remove_pair(&mut self, u: usize, v: usize) {
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

    pub fn scalar_simp(&mut self) -> bool {
        let mut any = false;
        for v in 0..self.phase.len() {
            if self.alive[v] && !self.is_boundary(v) && self.degree(v) == 0 {
                self.remove_single(v);
                any = true;
            }
        }
        for u in 0..self.phase.len() {
            if !self.alive[u] || self.is_boundary(u) || self.degree(u) != 1 {
                continue;
            }
            let v = self.neighbours(u)[0];
            if v > u && !self.is_boundary(v) && self.degree(v) == 1 {
                self.remove_pair(u, v);
                any = true;
            }
        }
        any
    }

    // ------------------------------------------------------- DRIVER COMPOSITION

    /// Interior Clifford simplification: fusion, identity removal, local complementation,
    /// pivoting, and scalar reduction to fixpoint.
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

    /// Full Clifford simplification including boundary-repairing generic pivots.
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

    /// Full reduction: Clifford simplification + gadget fusion + gadget π removal,
    /// iterated to fixpoint. This achieves maximum T-count reduction.
    pub fn full_reduce(&mut self) {
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
        debug_assert!(settled, "full_reduce iteration fence hit");
        self.compact();
    }

    /// Alias for full_reduce matching published nomenclature.
    pub fn full_simp(&mut self) {
        self.full_reduce();
    }
}
