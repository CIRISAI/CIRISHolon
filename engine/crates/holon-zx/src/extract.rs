//! Circuit extraction from simplified ZX diagrams.
//!
//! Implements frontier-based circuit extraction over GF(2) (Backens et al. 2021)
//! to synthesize an optimized gate sequence (Clifford+T) in the Surface alphabet.

use crate::graph::{bits_of, EdgeType, ZxGraph};
use holon::ledger::Cyc;
use holon::qasm::Surface;
use std::collections::{HashSet, VecDeque};

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
        BitMat {
            rows,
            cols,
            words,
            d: vec![0u64; rows * words],
        }
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
        self.d[r * self.words..(r + 1) * self.words]
            .iter()
            .map(|w| w.count_ones())
            .sum()
    }

    fn row_bits(&self, r: usize) -> Vec<usize> {
        bits_of(&self.d[r * self.words..(r + 1) * self.words])
    }
}

fn gauss_jordan(m: &mut BitMat, proxy: &mut BitMat) {
    let mut pivot = 0;
    for col in 0..m.cols {
        if pivot == m.rows {
            break;
        }
        let Some(r) = (pivot..m.rows).find(|&r| m.get(r, col)) else {
            continue;
        };
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

#[derive(Clone, Debug)]
pub struct Extraction {
    pub gates: Vec<Surface>,
    pub scalar: Cyc,
}

impl ZxGraph {
    fn is_boundary_pauli(&self, v: usize) -> bool {
        matches!(self.phase[v].rem_euclid(8), 0 | 4)
            && self.neighbours(v).into_iter().any(|n| self.is_boundary(n))
    }

    fn check_boundary_pivot(&self, v0: usize, v1: usize) -> bool {
        v0 != v1
            && self.alive[v0]
            && self.alive[v1]
            && !self.is_boundary(v0)
            && !self.is_boundary(v1)
            && self.has_edge(v0, v1)
            && self.is_h(v0, v1)
            && self.is_boundary_pauli(v0)
    }

    /// Extract a quantum circuit from the graph-like ZX diagram.
    pub fn extract(&mut self) -> Result<Extraction, String> {
        let nq = self.outputs.len();
        if nq == 0 {
            return Err("zx: cannot extract a circuit from a diagram with no outputs".into());
        }
        let mut circuit: VecDeque<Surface> = VecDeque::new();

        let mut gadgets: HashSet<usize> = HashSet::new();
        for v in 0..self.phase.len() {
            if self.alive[v] && !self.is_boundary(v) && self.degree(v) == 1 {
                let hub = self.neighbours(v)[0];
                if !self.is_boundary(hub) {
                    gadgets.insert(hub);
                }
            }
        }

        let mut frontier: Vec<(usize, usize)> = Vec::new();
        let fence = 64 * self.n_spiders() + 1024;
        for _ in 0..fence {
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
                    self.set_edge(o, v, Some(EdgeType::Normal));
                }
                if self.is_boundary(v) {
                    continue;
                }
                frontier.push((q, v));
                let p = self.phase[v].rem_euclid(8);
                if p != 0 {
                    circuit.push_front(Surface::DiagPow(p, q));
                    self.phase[v] = 0;
                }
            }

            for i in 0..frontier.len() {
                let (q, v) = frontier[i];
                for n in self.neighbours(v) {
                    if n == self.outputs[q] {
                        continue;
                    }
                    if self.is_boundary(n) {
                        if !self.inputs.contains(&n) {
                            return Err(format!("zx: spider {v} joins two outputs"));
                        }
                        if self.degree(v) > 2 {
                            self.unfuse_boundary(v, n);
                        }
                    } else if let Some(&(r, _)) = frontier.iter().find(|&&(_, w)| w == n) {
                        self.set_edge(v, n, None);
                        self.scalar.m += 1;
                        circuit.push_front(Surface::Cz(q, r));
                    }
                }
            }

            if frontier.is_empty() {
                self.permutation_to_gates(&mut circuit)?;
                return Ok(Extraction {
                    gates: circuit.into_iter().collect(),
                    scalar: self.scalar,
                });
            }

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

            if self.consume_frontier(&frontier) {
                continue;
            }
            self.frontier_gauss(&frontier, &mut circuit)?;
            if self.consume_frontier(&frontier) {
                continue;
            }
            return Err("zx: no extractable vertex — diagram has no gflow".into());
        }
        Err("zx: extraction hit its iteration fence".into())
    }

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

    fn frontier_gauss(
        &mut self,
        frontier: &[(usize, usize)],
        circuit: &mut VecDeque<Surface>,
    ) -> Result<(), String> {
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
            return Err("zx: no frontier vertex can be isolated — diagram has no gflow".into());
        };
        let recipe = ops.row_bits(row);
        if recipe.len() < 2 {
            return Err("zx: frontier elimination made no progress".into());
        }
        let control = recipe[0];
        let (cq, cv) = frontier[control];
        for &i in &recipe[1..] {
            let (tq, tv) = frontier[i];
            let mask = self.had[tv].clone();
            let size = mask.iter().map(|w| w.count_ones()).sum::<u32>() as i32;
            let destroyed = self.xor_neighbourhood(cv, &mask) as i32;
            self.scalar.m += 2 * destroyed - size;
            circuit.push_front(Surface::Cx(cq, tq));
        }
        Ok(())
    }

    fn permutation_to_gates(&self, circuit: &mut VecDeque<Surface>) -> Result<(), String> {
        let nq = self.outputs.len();
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
                return Err("zx: two outputs share an input — diagram is not unitary".into());
            }
            src[i] = j;
        }
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

/// Extract a simplified circuit from surface gates.
pub fn extract_circuit(n: usize, surface: &[Surface]) -> Result<Extraction, String> {
    let mut g = crate::graph::from_surface(n, surface)?;
    g.full_reduce();
    g.extract()
}
