//! Tier 1, TRANSPOSED AND FLATTENED: the column-major tableau at the
//! hardware roofline — the full mechanical cash-in of the stim comparison.
//!
//! Layout (credited: Gidney, Quantum 5, 497 (2021)): for each qubit, the X
//! and Z bits of all 2n rows as contiguous words; every unitary gate is
//! ~2n/64 word operations with sign updates as word-parallel masks. This
//! revision removes the per-gate constant that dominated at small n: the
//! columns live in ONE flat allocation per plane (no nested-Vec pointer
//! chase), kernels take raw column slices with the bounds check hoisted to
//! a single split, and the loops are stride-1 over contiguous words so the
//! compiler's autovectorizer gets exactly the shape it wants.
//!
//! Division of labour, unchanged: this engine carries the GATE path; the
//! measurement/rowsum path stays on the certified row-major reference via
//! `to_packed()` (one transpose, amortized over whole-circuit application).
//! The conformance gate requires bit-identical tableaux against
//! `PackedTableau` on random circuits — the reference remains the authority;
//! this file is an accelerator with a proof obligation, not a second truth.

use crate::plane::BitPlane;
use crate::tableau::{PackedTableau, PauliRow};

pub struct ColTableau {
    pub n: usize,
    /// 2n rows: 0..n destabilizers, n..2n stabilizers.
    pub nrows: usize,
    words: usize,
    /// Flat X plane: column q occupies words [q*words, (q+1)*words).
    x: Vec<u64>,
    /// Flat Z plane, same geometry.
    z: Vec<u64>,
    /// Sign bit per row (physical rows carry ±1 only; mod-4 intermediates
    /// live in the reference's rowsum path).
    r: Vec<u64>,
}

impl ColTableau {
    pub fn new(n: usize) -> Self {
        let nrows = 2 * n;
        let words = nrows.div_ceil(64);
        let mut t = ColTableau {
            n,
            nrows,
            words,
            x: vec![0u64; n * words],
            z: vec![0u64; n * words],
            r: vec![0u64; words],
        };
        for i in 0..n {
            t.x[i * words + (i >> 6)] |= 1 << (i & 63); // destabilizer i = X_i
            let s = n + i;
            t.z[i * words + (s >> 6)] |= 1 << (s & 63); // stabilizer i = Z_i
        }
        t
    }

    #[inline(always)]
    fn col<'a>(plane: &'a mut [u64], q: usize, words: usize) -> &'a mut [u64] {
        &mut plane[q * words..(q + 1) * words]
    }

    /// H(q): r ^= x&z; swap(x, z) — one pass, stride-1.
    #[inline]
    pub fn h(&mut self, q: usize) {
        let x = Self::col(&mut self.x, q, self.words);
        let z = Self::col(&mut self.z, q, self.words);
        for ((xr, zr), rr) in x.iter_mut().zip(z.iter_mut()).zip(self.r.iter_mut()) {
            *rr ^= *xr & *zr;
            std::mem::swap(xr, zr);
        }
    }

    /// S(q): r ^= x&z; z ^= x.
    #[inline]
    pub fn s(&mut self, q: usize) {
        let x = Self::col(&mut self.x, q, self.words);
        let z = Self::col(&mut self.z, q, self.words);
        for ((xr, zr), rr) in x.iter().zip(z.iter_mut()).zip(self.r.iter_mut()) {
            *rr ^= *xr & *zr;
            *zr ^= *xr;
        }
    }

    /// S†(q): direct one-pass form (S³ folded): r ^= x & ~z; z ^= x.
    /// Derivation: applying the S update three times sends z→z^x and
    /// accumulates r ^= (x&z) ^ (x&(z^x)) ^ (x&z) = x&(z^x) = x&~z on the
    /// x-support. Conformance-gated against the reference like every gate.
    #[inline]
    pub fn sdg(&mut self, q: usize) {
        let x = Self::col(&mut self.x, q, self.words);
        let z = Self::col(&mut self.z, q, self.words);
        for ((xr, zr), rr) in x.iter().zip(z.iter_mut()).zip(self.r.iter_mut()) {
            *rr ^= *xr & !*zr;
            *zr ^= *xr;
        }
    }

    /// X(q): r ^= z.
    #[inline]
    pub fn x_gate(&mut self, q: usize) {
        let z = Self::col(&mut self.z, q, self.words);
        for (rr, zr) in self.r.iter_mut().zip(z.iter()) {
            *rr ^= *zr;
        }
    }

    /// Z(q): r ^= x.
    #[inline]
    pub fn z_gate(&mut self, q: usize) {
        let x = Self::col(&mut self.x, q, self.words);
        for (rr, xr) in self.r.iter_mut().zip(x.iter()) {
            *rr ^= *xr;
        }
    }

    /// CX(c,t): r ^= x_c & z_t & ~(x_t ^ z_c); x_t ^= x_c; z_c ^= z_t.
    /// Two disjoint columns per plane: split the flat buffer once, no
    /// per-word bounds checks, one fused stride-1 pass.
    #[inline]
    pub fn cx(&mut self, c: usize, t: usize) {
        assert_ne!(c, t, "cx: control equals target");
        let w = self.words;
        let (xc, xt) = Self::two_cols(&mut self.x, c, t, w);
        let (zc, zt) = Self::two_cols(&mut self.z, c, t, w);
        for i in 0..w {
            let (xcw, zcw) = (xc[i], zc[i]);
            let (xtw, ztw) = (xt[i], zt[i]);
            self.r[i] ^= xcw & ztw & !(xtw ^ zcw);
            xt[i] = xtw ^ xcw;
            zc[i] = zcw ^ ztw;
        }
    }

    /// Disjoint mutable views of two columns in one flat plane.
    #[inline(always)]
    fn two_cols<'a>(
        plane: &'a mut [u64],
        a: usize,
        b: usize,
        words: usize,
    ) -> (&'a mut [u64], &'a mut [u64]) {
        if a < b {
            let (lo, hi) = plane.split_at_mut(b * words);
            (&mut lo[a * words..(a + 1) * words], &mut hi[..words])
        } else {
            let (lo, hi) = plane.split_at_mut(a * words);
            let (bcol, acol) = (&mut lo[b * words..(b + 1) * words], &mut hi[..words]);
            (acol, bcol)
        }
    }

    /// Transpose back to the certified row-major reference (measurement,
    /// rowsum, audit). Signs: bit b ↦ r = 2b (physical rows are ±1).
    ///
    /// Word-parallel: 64×64 bit blocks through the in-register transpose
    /// (Hacker's Delight §7-3), never a bit alone — the same discipline as
    /// the gate path. The per-bit version this replaces was 87–100% of
    /// whole-circuit wall time at n ≥ 64.
    pub fn to_packed(&self) -> PackedTableau {
        let nq_words = self.n.div_ceil(64);
        let mut rows: Vec<PauliRow> = (0..self.nrows)
            .map(|_| PauliRow {
                x: BitPlane::zeros(self.n),
                z: BitPlane::zeros(self.n),
                r: 0,
            })
            .collect();
        let mut bx = [0u64; 64];
        let mut bz = [0u64; 64];
        for qb in 0..nq_words {
            for rb in 0..self.words {
                // The in-register routine transposes across the ANTI-diagonal
                // ((r,c) -> (63-c, 63-r)); reversing the slot index at gather
                // and scatter turns it into the plain transpose for free.
                for i in 0..64 {
                    let q = qb * 64 + i;
                    if q < self.n {
                        bx[63 - i] = self.x[q * self.words + rb];
                        bz[63 - i] = self.z[q * self.words + rb];
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
                    if row < self.nrows {
                        rows[row].x.words[qb] = bx[63 - j];
                        rows[row].z.words[qb] = bz[63 - j];
                    }
                }
            }
        }
        for (row, pr) in rows.iter_mut().enumerate() {
            pr.r = ((self.r[row >> 6] >> (row & 63) & 1) as u8) * 2;
        }
        PackedTableau { n: self.n, rows }
    }
}

/// In-register 64×64 bit-matrix transpose (Hacker's Delight §7-3): after the
/// call, bit i of `a[j]` is what bit j of `a[i]` was.
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
