//! Tier 1 on the universal object: a Pauli row IS two planes and a sign;
//! the tableau is rows of them. Aaronson–Gottesman semantics (credited),
//! Stim-style packed layout (credited): the sign arithmetic of row products
//! runs on POPCOUNT MASKS — the mod-4 g-sum computed word-parallel, which is
//! the 64× on the measurement path that the unpacked Vec<bool> tier pays.

use crate::plane::BitPlane;

#[derive(Clone)]
pub struct PauliRow {
    pub x: BitPlane,
    pub z: BitPlane,
    /// Sign exponent mod 4 (physical rows stay mod 2; intermediates mod 4).
    pub r: u8,
}

impl PauliRow {
    pub fn identity(n: usize) -> Self {
        PauliRow { x: BitPlane::zeros(n), z: BitPlane::zeros(n), r: 0 }
    }

    /// Aaronson–Gottesman rowsum semantics: self := self + other, with the
    /// g-sum's FIRST slot holding the SOURCE row (o) — Pauli phases
    /// anticommute, so the role order is load-bearing and matches the
    /// certified unpacked reference. Word-parallel via popcount masks:
    ///   +1: (~x1&z1&x2&~z2) | (x1&z1&~x2&z2) | (x1&~z1&x2&z2)
    ///   −1: (~x1&z1&x2&z2)  | (x1&z1&x2&~z2) | (x1&~z1&~x2&z2)
    /// with (x1,z1) = o (source), (x2,z2) = self (target).
    pub fn mul_assign(&mut self, o: &PauliRow) {
        let mut plus = 0u32;
        let mut minus = 0u32;
        for i in 0..self.x.words.len() {
            let (x1, z1) = (o.x.words[i], o.z.words[i]);
            let (x2, z2) = (self.x.words[i], self.z.words[i]);
            let p = (!x1 & z1 & x2 & !z2) | (x1 & z1 & !x2 & z2) | (x1 & !z1 & x2 & z2);
            let m = (!x1 & z1 & x2 & z2) | (x1 & z1 & x2 & !z2) | (x1 & !z1 & !x2 & z2);
            plus += p.count_ones();
            minus += m.count_ones();
        }
        let g = (plus as i64 - minus as i64).rem_euclid(4) as u8;
        self.r = (self.r + o.r + g) % 4;
        self.x.xor_assign(&o.x);
        self.z.xor_assign(&o.z);
    }
}

pub struct PackedTableau {
    pub n: usize,
    /// rows 0..n destabilizers, n..2n stabilizers.
    pub rows: Vec<PauliRow>,
}

impl PackedTableau {
    pub fn new(n: usize) -> Self {
        let mut rows = Vec::with_capacity(2 * n);
        for i in 0..n {
            let mut r = PauliRow::identity(n);
            r.x.set(i, true);
            rows.push(r);
        }
        for i in 0..n {
            let mut r = PauliRow::identity(n);
            r.z.set(i, true);
            rows.push(r);
        }
        PackedTableau { n, rows }
    }

    pub fn h(&mut self, q: usize) {
        for row in &mut self.rows {
            let (xb, zb) = (row.x.get(q), row.z.get(q));
            if xb && zb {
                row.r = (row.r + 2) % 4;
            }
            row.x.set(q, zb);
            row.z.set(q, xb);
        }
    }

    pub fn s(&mut self, q: usize) {
        for row in &mut self.rows {
            let (xb, zb) = (row.x.get(q), row.z.get(q));
            if xb && zb {
                row.r = (row.r + 2) % 4;
            }
            if xb {
                row.z.flip(q);
            }
        }
    }

    pub fn sdg(&mut self, q: usize) {
        self.s(q);
        self.s(q);
        self.s(q);
    }

    pub fn x_gate(&mut self, q: usize) {
        for row in &mut self.rows {
            if row.z.get(q) {
                row.r = (row.r + 2) % 4;
            }
        }
    }

    pub fn z_gate(&mut self, q: usize) {
        for row in &mut self.rows {
            if row.x.get(q) {
                row.r = (row.r + 2) % 4;
            }
        }
    }

    pub fn cx(&mut self, c: usize, t: usize) {
        for row in &mut self.rows {
            let (xc, zc) = (row.x.get(c), row.z.get(c));
            let (xt, zt) = (row.x.get(t), row.z.get(t));
            if xc && zt && (xt == zc) {
                row.r = (row.r + 2) % 4;
            }
            if xc {
                row.x.flip(t);
            }
            if zt {
                row.z.flip(c);
            }
        }
    }

    /// Measure qubit q. Deterministic → Some(bit); random → None (collapse
    /// with a chosen outcome via `collapse`).
    pub fn measure_peek(&self, q: usize) -> Option<bool> {
        for p in self.n..2 * self.n {
            if self.rows[p].x.get(q) {
                return None;
            }
        }
        let mut scratch = PauliRow::identity(self.n);
        for i in 0..self.n {
            if self.rows[i].x.get(q) {
                let stab = self.rows[i + self.n].clone();
                scratch.mul_assign(&stab);
            }
        }
        Some(scratch.r % 4 == 2)
    }

    pub fn collapse(&mut self, q: usize, outcome: bool) {
        let p = (self.n..2 * self.n)
            .find(|&p| self.rows[p].x.get(q))
            .expect("collapse requires a random measurement");
        let pivot = self.rows[p].clone();
        for i in 0..2 * self.n {
            if i != p && self.rows[i].x.get(q) {
                self.rows[i].mul_assign(&pivot);
            }
        }
        self.rows[p - self.n] = pivot;
        let mut fresh = PauliRow::identity(self.n);
        fresh.z.set(q, true);
        fresh.r = if outcome { 2 } else { 0 };
        self.rows[p] = fresh;
    }
}
