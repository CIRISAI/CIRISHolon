//! Tier 1 on the universal object: a Pauli row IS two planes and a sign;
//! the tableau is rows of them. Aaronson–Gottesman semantics (credited),
//! Stim-style packed layout (credited): the sign arithmetic of row products
//! runs on POPCOUNT MASKS — the mod-4 g-sum computed word-parallel, which is
//! the 64× on the measurement path that the unpacked Vec<bool> tier pays.

use crate::plane::BitPlane;

#[derive(Clone)]
#[derive(Debug)]
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

#[derive(Clone, Debug)]
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

impl PackedTableau {
    /// Terminal computational-basis sample of ALL qubits in ONE canonical
    /// pass — replacing n independent peek/collapse cascades (each up to n
    /// rowsums) with a single Gaussian elimination of the stabilizer half.
    ///
    /// Semantics, pinned: the returned outcome is the unique valid sample
    /// with every FREE bit false in the canonical frame (free = X-pivot
    /// columns of the RREF). Deterministic marginals are forced, so any
    /// qubit whose `measure_peek` is `Some(b)` reads `b` here; the sample as
    /// a whole satisfies every stabilizer parity constraint (the conformance
    /// gate replays it through the sequential reference).
    ///
    /// Why the solve is always well-posed: a nontrivial pure-Z combination
    /// supported only on pivot columns would anticommute with the pivot row
    /// of any column it touches (RREF clears pivot columns elsewhere), and
    /// stabilizers commute — so the pure-Z constraints restricted to
    /// non-pivot columns have full rank, and "free bits on pivot columns,
    /// solve the rest" always has exactly one answer.
    pub fn sample_all(&self) -> Vec<bool> {
        let n = self.n;
        let mut stab: Vec<PauliRow> = self.rows[n..2 * n].to_vec();

        // RREF on the X part.
        let mut is_pivot_col = vec![false; n];
        let mut next_row = 0usize;
        for q in 0..n {
            if let Some(pr) = (next_row..n).find(|&r| stab[r].x.get(q)) {
                stab.swap(next_row, pr);
                let pivot = stab[next_row].clone();
                for r in 0..n {
                    if r != next_row && stab[r].x.get(q) {
                        stab[r].mul_assign(&pivot);
                    }
                }
                is_pivot_col[q] = true;
                next_row += 1;
            }
        }
        let k = next_row; // rows k..n are pure-Z: the parity constraints

        // Solve the parity system on non-pivot coordinates (pivot bits are
        // the free ones, chosen false, so they contribute nothing).
        let mut cons: Vec<(crate::plane::BitPlane, bool)> = stab[k..]
            .iter()
            .map(|g| {
                debug_assert!(g.x.words.iter().all(|&w| w == 0), "pure-Z row expected");
                let mut zp = g.z.clone();
                for q in 0..n {
                    if is_pivot_col[q] {
                        zp.set(q, false);
                    }
                }
                (zp, g.r % 4 == 2)
            })
            .collect();

        let mut y = vec![false; n];
        let mut used = vec![false; cons.len()];
        for q in (0..n).filter(|&q| !is_pivot_col[q]) {
            let ci = (0..cons.len())
                .find(|&ci| !used[ci] && cons[ci].0.get(q))
                .expect("full-rank parity system (see doc comment)");
            used[ci] = true;
            let (sup, rhs) = (cons[ci].0.clone(), cons[ci].1);
            for cj in 0..cons.len() {
                if cj != ci && cons[cj].0.get(q) {
                    cons[cj].0.xor_assign(&sup);
                    cons[cj].1 ^= rhs;
                }
            }
            let _ = (q, rhs);
        }
        // After the full Jordan pass each used constraint retains exactly its
        // pinning column; read the settled values.
        for (ci, (sup, rhs)) in cons.iter().enumerate() {
            if used[ci] {
                for q in 0..n {
                    if !is_pivot_col[q] && sup.get(q) {
                        y[q] = *rhs;
                    }
                }
            }
        }
        y
    }
}

#[cfg(test)]
mod sample_conformance {
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

    /// The gate: the one-pass sample must (a) agree with every deterministic
    /// marginal, and (b) be accepted bit-for-bit by the sequential
    /// peek/collapse reference when its outcomes are forced to the sample.
    #[test]
    fn sample_all_replays_through_reference() {
        let mut rng = Rng(0xBEEF_CAFE);
        for n in [3usize, 8, 24, 61] {
            for _trial in 0..4 {
                let mut t = PackedTableau::new(n);
                for _ in 0..12 * n {
                    let q = rng.below(n);
                    let mut q2 = rng.below(n);
                    while q2 == q {
                        q2 = rng.below(n);
                    }
                    match rng.below(5) {
                        0 => t.h(q),
                        1 => t.s(q),
                        2 => t.x_gate(q),
                        3 => t.cx(q, q2),
                        _ => t.z_gate(q),
                    }
                }
                let y = t.sample_all();
                let mut replay = PackedTableau { n, rows: t.rows.clone() };
                for q in 0..n {
                    match replay.measure_peek(q) {
                        Some(b) => assert_eq!(y[q], b, "n={n} q={q}: deterministic marginal"),
                        None => replay.collapse(q, y[q]),
                    }
                }
            }
        }
    }
}

impl PackedTableau {
    /// THE TIER'S CANONICAL FORM, made explicit.
    ///
    /// TIERS.md's per-tier analysis names the universal need: every tier has
    /// a PRESENTATION (an ordering or gauge) and a CONTENT (the invariant),
    /// and the tiers we do well on are exactly the ones whose canonicalizer
    /// exists. For Clifford that canonicalizer is the tableau itself — the
    /// stabilizer GROUP is the invariant and the gate order is presentation
    /// — but "the tableau is canonical" was implicit until now, which meant
    /// two tableaux for the same state could compare unequal.
    ///
    /// This makes it explicit: reduce the stabilizer half to row-reduced
    /// echelon form over F₂ (X block then Z block), fixing signs along the
    /// way. Two states are then EQUAL as states iff their canonical forms
    /// are equal bit-for-bit — the same contract `Affine::canon_key` gives
    /// the branch layer, and the reason dedup can collapse a branch space.
    pub fn canonicalize(&mut self) {
        let n = self.n;
        let mut pivot = 0usize;
        // X block, then Z block: 2n columns over the n stabilizer rows.
        for col in 0..2 * n {
            let get = |r: &PauliRow, c: usize| {
                if c < n {
                    r.x.get(c)
                } else {
                    r.z.get(c - n)
                }
            };
            let Some(p) = (pivot..n).find(|&i| get(&self.rows[n + i], col)) else {
                continue;
            };
            // Every operation on the stabilizer half must be mirrored on the
            // destabilizer half by its SYMPLECTIC PARTNER operation, or the
            // pairing `measure_peek` relies on is destroyed: a swap pairs
            // with a swap, and `stab[i] *= stab[pivot]` pairs with
            // `destab[pivot] *= destab[i]` (note the reversed indices).
            self.rows.swap(n + pivot, n + p);
            self.rows.swap(pivot, p);
            for i in 0..n {
                if i != pivot && get(&self.rows[n + i], col) {
                    let src = self.rows[n + pivot].clone();
                    self.rows[n + i].mul_assign(&src);
                    let dsrc = self.rows[i].clone();
                    self.rows[pivot].mul_assign(&dsrc);
                }
            }
            pivot += 1;
            if pivot == n {
                break;
            }
        }
    }

    /// A canonical byte key: equal keys ⟺ equal stabilizer states. The
    /// caller must have canonicalized first (this hashes the form, it does
    /// not impose it) — the split is deliberate, so a caller cannot pay for
    /// canonicalization twice by accident.
    pub fn canon_key(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.n * (2 * self.n / 8 + 2));
        for r in &self.rows[self.n..] {
            for w in &r.x.words {
                out.extend_from_slice(&w.to_le_bytes());
            }
            for w in &r.z.words {
                out.extend_from_slice(&w.to_le_bytes());
            }
            out.push(r.r % 4);
        }
        out
    }
}

#[cfg(test)]
mod canonical_tests {
    use super::*;

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            self.0 >> 11
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
    }

    /// THE CONTRACT: the same state reached by DIFFERENT gate orders must
    /// canonicalize to the same key — presentation stripped, content kept.
    #[test]
    fn different_presentations_same_canonical_form() {
        let n = 4;
        // Two orderings of commuting operations that reach the same state.
        let mut a = PackedTableau::new(n);
        a.h(0);
        a.h(2);
        a.cx(0, 1);
        a.cx(2, 3);
        let mut b = PackedTableau::new(n);
        b.h(2);
        b.cx(2, 3);
        b.h(0);
        b.cx(0, 1);
        a.canonicalize();
        b.canonicalize();
        assert_eq!(a.canon_key(), b.canon_key(), "commuting reorder changed the key");
    }

    /// Canonicalization must be IDEMPOTENT and must not change the state:
    /// every measurement outcome is preserved.
    #[test]
    fn canonicalization_is_idempotent_and_state_preserving() {
        let mut rng = Rng(0xCA_11);
        for _ in 0..20 {
            let n = 5;
            let mut t = PackedTableau::new(n);
            for _ in 0..40 {
                let q = rng.below(n);
                let mut q2 = rng.below(n);
                while q2 == q {
                    q2 = rng.below(n);
                }
                match rng.below(4) {
                    0 => t.h(q),
                    1 => t.s(q),
                    2 => t.cx(q, q2),
                    _ => t.z_gate(q),
                }
            }
            let before: Vec<Option<bool>> = (0..n).map(|q| t.measure_peek(q)).collect();
            t.canonicalize();
            let k1 = t.canon_key();
            let after: Vec<Option<bool>> = (0..n).map(|q| t.measure_peek(q)).collect();
            assert_eq!(before, after, "canonicalization changed the state");
            t.canonicalize();
            assert_eq!(k1, t.canon_key(), "canonicalization is not idempotent");
        }
    }

    /// Distinct states must NOT collide — a canonical form that maps
    /// everything to one key would pass the tests above and be useless.
    #[test]
    fn distinct_states_have_distinct_keys() {
        let n = 3;
        let mut keys = std::collections::HashSet::new();
        for bits in 0..8u32 {
            let mut t = PackedTableau::new(n);
            for q in 0..n {
                if bits >> q & 1 == 1 {
                    t.x_gate(q);
                }
            }
            t.canonicalize();
            assert!(keys.insert(t.canon_key()), "distinct basis states collided");
        }
    }
}
