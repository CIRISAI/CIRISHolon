//! Tier 1, TRANSPOSED: the column-major tableau — the mechanical piece the
//! stim comparison was owed.
//!
//! The row-major `PackedTableau` (the certified reference) pays 2n strided
//! single-bit accesses per gate: bit q of every row, each row its own
//! allocation. Stim's measured lead is exactly this transpose (credited:
//! Gidney, Quantum 5, 497 (2021)): store, for each qubit, the X and Z bits
//! of ALL 2n rows as contiguous words, and every unitary gate becomes ~2n/64
//! word operations with sign updates as word-parallel masks — no bit is
//! touched alone. Same object, same planes-and-a-sign semantics; the layout
//! is the only thing that moves.
//!
//! Division of labour, honestly: this engine carries the GATE path. The
//! measurement/rowsum path stays on the certified row-major reference via
//! `to_packed()` (one transpose, O(n²/64) words, amortized over whole-circuit
//! application). The conformance gate below requires bit-identical tableaux
//! against `PackedTableau` on random circuits — the reference remains the
//! authority; this file is an accelerator with a proof obligation, not a
//! second truth.

use crate::plane::BitPlane;
use crate::tableau::{PackedTableau, PauliRow};

pub struct ColTableau {
    pub n: usize,
    /// 2n rows: 0..n destabilizers, n..2n stabilizers.
    pub nrows: usize,
    words: usize,
    /// Per qubit: the X bits of all rows, packed (row-index bit order).
    pub xcol: Vec<Vec<u64>>,
    /// Per qubit: the Z bits of all rows.
    pub zcol: Vec<Vec<u64>>,
    /// Sign bit per row (physical rows carry ±1 only; the mod-4 intermediates
    /// live in the rowsum path, which is the reference's job).
    pub r: Vec<u64>,
}

impl ColTableau {
    pub fn new(n: usize) -> Self {
        let nrows = 2 * n;
        let words = nrows.div_ceil(64);
        let mut t = ColTableau {
            n,
            nrows,
            words,
            xcol: vec![vec![0u64; words]; n],
            zcol: vec![vec![0u64; words]; n],
            r: vec![0u64; words],
        };
        for i in 0..n {
            t.xcol[i][i >> 6] |= 1 << (i & 63); // destabilizer i = X_i
            let s = n + i;
            t.zcol[i][s >> 6] |= 1 << (s & 63); // stabilizer i = Z_i
        }
        t
    }

    /// H(q): swap the X and Z columns; r ^= x&z (word-parallel).
    pub fn h(&mut self, q: usize) {
        let (x, z) = (&mut self.xcol[q], &mut self.zcol[q]);
        for w in 0..self.words {
            self.r[w] ^= x[w] & z[w];
        }
        std::mem::swap(x, z);
    }

    /// S(q): r ^= x&z; z ^= x.
    pub fn s(&mut self, q: usize) {
        let (x, z) = (&self.xcol[q], &mut self.zcol[q]);
        for w in 0..self.words {
            self.r[w] ^= x[w] & z[w];
            z[w] ^= x[w];
        }
    }

    /// S†(q) = S³(q) — three word-parallel passes; still no bit touched alone.
    pub fn sdg(&mut self, q: usize) {
        self.s(q);
        self.s(q);
        self.s(q);
    }

    /// X(q): r ^= z.
    pub fn x_gate(&mut self, q: usize) {
        for w in 0..self.words {
            self.r[w] ^= self.zcol[q][w];
        }
    }

    /// Z(q): r ^= x.
    pub fn z_gate(&mut self, q: usize) {
        for w in 0..self.words {
            self.r[w] ^= self.xcol[q][w];
        }
    }

    /// CX(c,t): r ^= x_c & z_t & ~(x_t ^ z_c); x_t ^= x_c; z_c ^= z_t.
    pub fn cx(&mut self, c: usize, t: usize) {
        for w in 0..self.words {
            let (xc, zc) = (self.xcol[c][w], self.zcol[c][w]);
            let (xt, zt) = (self.xcol[t][w], self.zcol[t][w]);
            self.r[w] ^= xc & zt & !(xt ^ zc);
            self.xcol[t][w] = xt ^ xc;
            self.zcol[c][w] = zc ^ zt;
        }
    }

    /// Transpose back to the certified row-major reference (for measurement,
    /// rowsum, or audit). Signs: bit b ↦ r = 2b (physical rows are ±1).
    pub fn to_packed(&self) -> PackedTableau {
        let mut rows = Vec::with_capacity(self.nrows);
        for row in 0..self.nrows {
            let (w, b) = (row >> 6, row & 63);
            let mut x = BitPlane::zeros(self.n);
            let mut z = BitPlane::zeros(self.n);
            for q in 0..self.n {
                if self.xcol[q][w] >> b & 1 == 1 {
                    x.set(q, true);
                }
                if self.zcol[q][w] >> b & 1 == 1 {
                    z.set(q, true);
                }
            }
            let r = ((self.r[w] >> b & 1) as u8) * 2;
            rows.push(PauliRow { x, z, r });
        }
        PackedTableau { n: self.n, rows }
    }
}

#[cfg(test)]
mod conformance {
    use super::*;

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 11
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
    }

    /// The proof obligation: after any random Clifford circuit, the
    /// transposed engine and the certified reference hold bit-identical
    /// tableaux — planes and signs both.
    #[test]
    fn bit_identical_to_reference() {
        let mut rng = Rng(0xD00D_F00D);
        for n in [3usize, 17, 64, 130] {
            let mut col = ColTableau::new(n);
            let mut refr = PackedTableau::new(n);
            for _ in 0..600 {
                let q = rng.below(n);
                let mut q2 = rng.below(n);
                while q2 == q {
                    q2 = rng.below(n);
                }
                match rng.below(6) {
                    0 => {
                        col.h(q);
                        refr.h(q);
                    }
                    1 => {
                        col.s(q);
                        refr.s(q);
                    }
                    2 => {
                        col.sdg(q);
                        refr.sdg(q);
                    }
                    3 => {
                        col.x_gate(q);
                        refr.x_gate(q);
                    }
                    4 => {
                        col.z_gate(q);
                        refr.z_gate(q);
                    }
                    _ => {
                        col.cx(q, q2);
                        refr.cx(q, q2);
                    }
                }
            }
            let packed = col.to_packed();
            for (i, (a, b)) in packed.rows.iter().zip(&refr.rows).enumerate() {
                assert_eq!(a.x, b.x, "n={n} row {i}: X planes differ");
                assert_eq!(a.z, b.z, "n={n} row {i}: Z planes differ");
                assert_eq!(a.r, b.r, "n={n} row {i}: signs differ");
            }
        }
    }

    /// Measurement flows through the reference after transpose: peek results
    /// must agree with a reference-run circuit.
    #[test]
    fn measurement_via_reference_agrees() {
        let mut rng = Rng(42);
        let n = 24;
        let mut col = ColTableau::new(n);
        let mut refr = PackedTableau::new(n);
        for _ in 0..300 {
            let q = rng.below(n);
            let mut q2 = rng.below(n);
            while q2 == q {
                q2 = rng.below(n);
            }
            match rng.below(4) {
                0 => {
                    col.h(q);
                    refr.h(q);
                }
                1 => {
                    col.s(q);
                    refr.s(q);
                }
                2 => {
                    col.cx(q, q2);
                    refr.cx(q, q2);
                }
                _ => {
                    col.z_gate(q);
                    refr.z_gate(q);
                }
            }
        }
        let via_col = col.to_packed();
        for q in 0..n {
            assert_eq!(via_col.measure_peek(q), refr.measure_peek(q), "qubit {q}");
        }
    }
}
