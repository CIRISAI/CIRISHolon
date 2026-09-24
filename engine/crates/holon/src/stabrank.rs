//! QVM-RANK-1's instrument: stabilizer states as exact objects, the exact span test, and the
//! two search branches (annealing with exact acceptance; pivot completion, exhaustive over a
//! declared class) for decompositions of `|H⟩^{⊗m}` into few stabilizer states.
//!
//! The prereg is `conformance/rank/QVM_RANK1_PREREG.md`; the field it shoots at is
//! `conformance/rank/STABRANK_STATE.md` (the Unitary Foundation's `stabrank`, whose verifier is
//! the referee, not this file).
//!
//! # The objects
//!
//! A stabilizer state on `n` qubits is held in the flat-and-phase form stabrank's verifier
//! reads (and Dehaene–De Moor / Van den Nest wrote down — credited):
//!
//! ```text
//! s(x0 ⊕ Σ_j y_j w_j) = i^{Σ_j l_j y_j} · (−1)^{Σ_{i<j} Q_ij y_i y_j},   s = 0 off the flat
//! ```
//!
//! UNNORMALISED (entries are units of `ℤ[i]` on the support), so every stabilizer state is a
//! vector over the Gaussian integers. The canonical form [`Stab`] fixes the ray: `w` in reduced
//! row-echelon form with each row's pivot its LOWEST set bit, `x0` with every pivot bit clear,
//! and the phase normalised to `1` at `x0`. Two states are the same ray iff their `Stab`s are
//! equal, which is what the dictionary dedup and the symmetry reduction key on.
//!
//! # The target, and why the ring is `ℤ[i]` plus `√2`
//!
//! `|H⟩^{⊗m} = cos^m(π/8) · v`, `v = (|0⟩ + (√2−1)|1⟩)^{⊗m} = a + √2·b` with `a, b ∈ ℤ^{2^m}`.
//! Because every stabilizer state is a `ℤ[i]` vector, `v ∈ span_ℂ(S)` iff `a, b ∈ span_{ℚ(i)}(S)`
//! (split `c = c₀ + √2 c₁` over the basis `{1, √2}` of `ℚ(ζ₈)/ℚ(i)`). That is the qubit form of
//! stabrank PR 16's Galois constraint, and it makes the exact test integer linear algebra over
//! `ℤ[i]` (fraction-free Bareiss, [`exact_solve`]). Acceptance is then re-done in the engine's
//! ledger ring [`Cyc`] ([`Witness::accept_cyc`]): `Σ_j α_j s_j = D · v` entry by entry, `α_j`
//! in `ℤ[ζ₈]`, `D` a positive integer. Floats prune; they never accept.
//!
//! # The searches
//!
//! * [`anneal`] — a local search over `r`-tuples of stabilizer states (Pauli, Clifford and
//!   single-state moves, Metropolis on the float residual of `span(a, b)` off the tuple's
//!   span), with every sub-threshold tuple handed to the exact test and with the SNAP: every
//!   `(r−2)`-subset of a good tuple is completed exactly by [`complete_pivot`].
//! * [`complete_pivot`] — the complete decision behind "the cheap part first": given `r − 2`
//!   pivot states `P`, every decomposition of rank `r` containing `P` has its two other terms in
//!   `U = span(P, a, b)` (dimension `r`), and every stabilizer state in `U` is found by
//!   enumerating its values on `r` rows where `U` is invertible (values in `c·{0, ±1, ±i}`,
//!   `(5^r − 1)/4` assignments), each candidate confirmed exactly.

// The ring types spell their operations `add`/`mul`/… as `Cyc` does (by-value, checked, no
// operator overloading that could hide a refusal), and the dense kernels index several
// parallel arrays by one loop variable, which is clearer than zipped iterators here.
#![allow(clippy::should_implement_trait, clippy::needless_range_loop)]

use crate::ledger::Cyc;
use crate::tableau::PackedTableau;

// ============================================================== Gaussian integers ====

/// A Gaussian integer with the ledger's envelope discipline: checked `i128`, refusal on
/// overflow (a wrapped coefficient is a silently wrong exact value).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Gi {
    pub re: i128,
    pub im: i128,
}

#[inline]
fn ck(v: Option<i128>) -> i128 {
    match v {
        Some(x) => x,
        None => panic!("Gi: i128 envelope exceeded; refusing rather than wrapping"),
    }
}

impl Gi {
    pub const ZERO: Gi = Gi { re: 0, im: 0 };
    pub const ONE: Gi = Gi { re: 1, im: 0 };

    pub fn new(re: i128, im: i128) -> Gi {
        Gi { re, im }
    }
    /// `i^e`.
    pub fn unit(e: u8) -> Gi {
        match e & 3 {
            0 => Gi::new(1, 0),
            1 => Gi::new(0, 1),
            2 => Gi::new(-1, 0),
            _ => Gi::new(0, -1),
        }
    }
    pub fn is_zero(self) -> bool {
        self.re == 0 && self.im == 0
    }
    pub fn add(self, o: Gi) -> Gi {
        Gi::new(ck(self.re.checked_add(o.re)), ck(self.im.checked_add(o.im)))
    }
    pub fn sub(self, o: Gi) -> Gi {
        Gi::new(ck(self.re.checked_sub(o.re)), ck(self.im.checked_sub(o.im)))
    }
    pub fn neg(self) -> Gi {
        Gi::new(-self.re, -self.im)
    }
    pub fn conj(self) -> Gi {
        Gi::new(self.re, -self.im)
    }
    pub fn mul(self, o: Gi) -> Gi {
        let rr = ck(self.re.checked_mul(o.re));
        let ii = ck(self.im.checked_mul(o.im));
        let ri = ck(self.re.checked_mul(o.im));
        let ir = ck(self.im.checked_mul(o.re));
        Gi::new(ck(rr.checked_sub(ii)), ck(ri.checked_add(ir)))
    }
    pub fn scale(self, s: i128) -> Gi {
        Gi::new(ck(self.re.checked_mul(s)), ck(self.im.checked_mul(s)))
    }
    /// `self · i^e`, exact.
    pub fn mul_unit(self, e: u8) -> Gi {
        match e & 3 {
            0 => self,
            1 => Gi::new(-self.im, self.re),
            2 => Gi::new(-self.re, -self.im),
            _ => Gi::new(self.im, -self.re),
        }
    }
    pub fn norm(self) -> i128 {
        ck(ck(self.re.checked_mul(self.re)).checked_add(ck(self.im.checked_mul(self.im))))
    }
    /// `self / d` when `d` divides `self` in `ℤ[i]`, else `None`.
    pub fn div_exact(self, d: Gi) -> Option<Gi> {
        let nd = d.norm();
        assert!(nd != 0, "Gi::div_exact by zero");
        let p = self.mul(d.conj());
        if p.re % nd != 0 || p.im % nd != 0 {
            return None;
        }
        Some(Gi::new(p.re / nd, p.im / nd))
    }
    /// The ledger ring's face of this number: `re + im·ω²`.
    pub fn to_cyc(self) -> Cyc {
        Cyc { c: [self.re, 0, self.im, 0], m: 0 }
    }
    pub fn to_c64(self) -> C64 {
        C64 { re: self.re as f64, im: self.im as f64 }
    }
}

fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

// ================================================================= float complex ====

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct C64 {
    pub re: f64,
    pub im: f64,
}

impl C64 {
    #[inline]
    pub fn add(self, o: C64) -> C64 {
        C64 { re: self.re + o.re, im: self.im + o.im }
    }
    #[inline]
    pub fn sub(self, o: C64) -> C64 {
        C64 { re: self.re - o.re, im: self.im - o.im }
    }
    #[inline]
    pub fn mul(self, o: C64) -> C64 {
        C64 { re: self.re * o.re - self.im * o.im, im: self.re * o.im + self.im * o.re }
    }
    #[inline]
    pub fn conj(self) -> C64 {
        C64 { re: self.re, im: -self.im }
    }
    #[inline]
    pub fn norm2(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
    #[inline]
    pub fn scale(self, s: f64) -> C64 {
        C64 { re: self.re * s, im: self.im * s }
    }
    pub fn div(self, o: C64) -> C64 {
        let d = o.norm2();
        let p = self.mul(o.conj());
        C64 { re: p.re / d, im: p.im / d }
    }
}

const UNITS: [C64; 4] = [
    C64 { re: 1.0, im: 0.0 },
    C64 { re: 0.0, im: 1.0 },
    C64 { re: -1.0, im: 0.0 },
    C64 { re: 0.0, im: -1.0 },
];

// ===================================================================== the target ====

/// `v = (|0⟩ + (√2−1)|1⟩)^{⊗n} = a + √2·b`, returned as `(a, b)` over the integers.
/// Entry `x` is `(√2−1)^{|x|}`; bit `j` of `x` is qubit `j`.
pub fn target_ab(n: usize) -> (Vec<i128>, Vec<i128>) {
    // (√2−1)^w = A_w + B_w √2
    let mut pw = vec![(1i128, 0i128)];
    for w in 1..=n {
        let (a, b) = pw[w - 1];
        // (a + b√2)(−1 + √2) = (−a + 2b) + (a − b)√2
        pw.push((2 * b - a, a - b));
    }
    let size = 1usize << n;
    let mut av = vec![0i128; size];
    let mut bv = vec![0i128; size];
    for x in 0..size {
        let (a, b) = pw[(x as u32).count_ones() as usize];
        av[x] = a;
        bv[x] = b;
    }
    (av, bv)
}

/// `√2` in the ledger ring: `ω − ω³`.
pub fn cyc_sqrt2() -> Cyc {
    Cyc { c: [0, 1, 0, -1], m: 0 }
}

/// The rescaled target `v` in the ledger ring, entry by entry.
pub fn target_cyc(n: usize) -> Vec<Cyc> {
    let (a, b) = target_ab(n);
    a.iter()
        .zip(&b)
        .map(|(&ai, &bi)| Cyc { c: [ai, 0, 0, 0], m: 0 }.add(cyc_sqrt2().mul(Cyc { c: [bi, 0, 0, 0], m: 0 })))
        .collect()
}

/// Normalised float targets: `v/‖v‖` (`galois = false`) or its Galois image
/// `σ(v)/‖σ(v)‖ ∝ |H^⊥⟩^{⊗n}` (`galois = true`).
pub fn target_f64(n: usize, galois: bool) -> Vec<C64> {
    let t = if galois { -(1.0 + 2f64.sqrt()) } else { 2f64.sqrt() - 1.0 };
    let size = 1usize << n;
    let mut v: Vec<C64> = (0..size)
        .map(|x| C64 { re: t.powi((x as u32).count_ones() as i32), im: 0.0 })
        .collect();
    let nr = v.iter().map(|c| c.norm2()).sum::<f64>().sqrt();
    for c in &mut v {
        *c = c.scale(1.0 / nr);
    }
    v
}

// ======================================================== the canonical stabilizer ====

/// Canonical flat-and-phase form of one stabilizer ray (module header).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Stab {
    pub n: u8,
    pub k: u8,
    pub x0: u32,
    /// `k` rows, RREF, pivot = lowest set bit, ascending pivots.
    pub w: Vec<u32>,
    /// `l_j ∈ ℤ₄`.
    pub l: Vec<u8>,
    /// `q[i]`: bitmask over `j > i` with `Q_ij = 1`.
    pub q: Vec<u32>,
}

/// Absent entry in a phase vector.
pub const ABSENT: u8 = 4;

/// Insert `v` into an RREF basis (pivot = lowest set bit). Returns false if dependent.
fn rref_insert(basis: &mut Vec<u32>, mut v: u32) -> bool {
    for &b in basis.iter() {
        let p = b.trailing_zeros();
        if (v >> p) & 1 == 1 {
            v ^= b;
        }
    }
    if v == 0 {
        return false;
    }
    let p = v.trailing_zeros();
    for b in basis.iter_mut() {
        if (*b >> p) & 1 == 1 {
            *b ^= v;
        }
    }
    basis.push(v);
    basis.sort_by_key(|b| b.trailing_zeros());
    true
}

fn reduce_by(basis: &[u32], mut v: u32) -> u32 {
    for &b in basis {
        if (v >> b.trailing_zeros()) & 1 == 1 {
            v ^= b;
        }
    }
    v
}

impl Stab {
    pub fn size(&self) -> usize {
        1usize << self.n
    }

    /// The support points in parameter order: `pts[y] = x0 ⊕ Σ_j y_j w_j`.
    pub fn points(&self) -> Vec<u32> {
        let k = self.k as usize;
        let mut pts = vec![0u32; 1 << k];
        pts[0] = self.x0;
        for y in 1..(1usize << k) {
            let j = y.trailing_zeros() as usize;
            pts[y] = pts[y & (y - 1)] ^ self.w[j];
        }
        pts
    }

    /// `(l·y + 2Q(y)) mod 4` at parameter `y`.
    pub fn phase_at(&self, y: u32) -> u8 {
        let mut e: u32 = 0;
        let mut quad: u32 = 0;
        let mut t = y;
        while t != 0 {
            let i = t.trailing_zeros() as usize;
            e += self.l[i] as u32;
            quad += (self.q[i] & y).count_ones();
            t &= t - 1;
        }
        ((e + 2 * quad) & 3) as u8
    }

    /// The phase vector: entry `x` is the exponent `e` of `i^e`, or [`ABSENT`].
    pub fn phases(&self) -> Vec<u8> {
        let mut ph = vec![ABSENT; self.size()];
        for (y, &x) in self.points().iter().enumerate() {
            ph[x as usize] = self.phase_at(y as u32);
        }
        ph
    }

    pub fn gi_vec(&self) -> Vec<Gi> {
        self.phases()
            .iter()
            .map(|&e| if e == ABSENT { Gi::ZERO } else { Gi::unit(e) })
            .collect()
    }

    pub fn cyc_vec(&self) -> Vec<Cyc> {
        self.gi_vec().iter().map(|g| g.to_cyc()).collect()
    }

    /// The NORMALISED float vector (norm 1).
    pub fn c64_vec(&self) -> Vec<C64> {
        phases_to_c64(&self.phases())
    }

    /// Recognise a phase vector (exponents, [`ABSENT`] off the support) as a stabilizer ray.
    /// This is the exact stabilizer-ness test on unit-valued data: the support must be an
    /// affine subspace, and the exponents must be `l·y + 2Q(y) mod 4` in the flat's canonical
    /// parameters. `None` = not a stabilizer state.
    pub fn from_phases(n: usize, ph: &[u8]) -> Option<Stab> {
        assert_eq!(ph.len(), 1usize << n);
        let supp: Vec<u32> = (0..ph.len() as u32).filter(|&x| ph[x as usize] != ABSENT).collect();
        let s = supp.len();
        if s == 0 || !s.is_power_of_two() {
            return None;
        }
        let k = s.trailing_zeros() as usize;
        let s0 = supp[0];
        let mut basis: Vec<u32> = Vec::with_capacity(k);
        for &x in &supp[1..] {
            if basis.len() == k {
                break;
            }
            rref_insert(&mut basis, x ^ s0);
        }
        if basis.len() != k {
            return None;
        }
        let x0 = reduce_by(&basis, s0);
        // every support point must lie on x0 + span (equal cardinality then gives equality)
        for &x in &supp {
            if reduce_by(&basis, x) != x0 {
                return None;
            }
        }
        let t0 = ph[x0 as usize];
        let rel = |x: u32| -> u8 { (ph[x as usize] + 4 - t0) & 3 };
        let l: Vec<u8> = basis.iter().map(|&w| rel(x0 ^ w)).collect();
        let mut q = vec![0u32; k];
        for i in 0..k {
            for j in i + 1..k {
                let d = (rel(x0 ^ basis[i] ^ basis[j]) + 8 - l[i] - l[j]) & 3;
                match d {
                    0 => {}
                    2 => q[i] |= 1 << j,
                    _ => return None,
                }
            }
        }
        let st = Stab { n: n as u8, k: k as u8, x0, w: basis, l, q };
        for (y, &x) in st.points().iter().enumerate() {
            if rel(x) != st.phase_at(y as u32) {
                return None;
            }
        }
        Some(st)
    }

    /// The exact stabilizer-ness test on a `ℤ[i]` vector: nonzero entries must be
    /// `c · i^{t_x}` for one common `c`, then [`Stab::from_phases`].
    pub fn recognize_gi(n: usize, v: &[Gi]) -> Option<Stab> {
        let first = v.iter().position(|g| !g.is_zero())?;
        let c = v[first];
        let nc = c.norm();
        let cc = c.conj();
        let mut ph = vec![ABSENT; v.len()];
        for (x, g) in v.iter().enumerate() {
            if g.is_zero() {
                continue;
            }
            let p = g.mul(cc);
            let e = if p == Gi::new(nc, 0) {
                0
            } else if p == Gi::new(0, nc) {
                1
            } else if p == Gi::new(-nc, 0) {
                2
            } else if p == Gi::new(0, -nc) {
                3
            } else {
                return None;
            };
            ph[x] = e;
        }
        Stab::from_phases(n, &ph)
    }

    /// The exact stabilizer-ness test on a vector over the ledger ring `Cyc` (PR-2's form):
    /// every nonzero entry must equal `c · i^{t}` for the first nonzero `c`, exactly.
    pub fn recognize_cyc(n: usize, v: &[Cyc]) -> Option<Stab> {
        use crate::affine::{cyc_eq, cyc_is_zero};
        let first = v.iter().position(|g| !cyc_is_zero(*g))?;
        let c = v[first];
        let mut ph = vec![ABSENT; v.len()];
        for (x, g) in v.iter().enumerate() {
            if cyc_is_zero(*g) {
                continue;
            }
            let e = (0..4u8).find(|&e| cyc_eq(*g, c.mul_i_pow(e)))?;
            ph[x] = e;
        }
        Stab::from_phases(n, &ph)
    }

    /// Build from stabrank's term fields (`x0`, `W` rows, `Q` upper triangle incl. diagonal,
    /// `l` mod 4), with lists indexed by qubit. `None` if the term is malformed.
    pub fn from_stabrank(n: usize, x0: &[u8], w: &[Vec<u8>], q: &[Vec<u8>], l: &[u8]) -> Option<Stab> {
        let k = w.len();
        let pack = |bits: &[u8]| -> u32 {
            bits.iter().enumerate().fold(0u32, |acc, (c, &b)| acc | (((b & 1) as u32) << c))
        };
        let x0m = pack(x0);
        let wm: Vec<u32> = w.iter().map(|r| pack(r)).collect();
        let mut ph = vec![ABSENT; 1 << n];
        for y in 0..(1u32 << k) {
            let mut x = x0m;
            let mut lin = 0u32;
            let mut quad = 0u32;
            for i in 0..k {
                if (y >> i) & 1 == 1 {
                    x ^= wm[i];
                    lin += l[i] as u32;
                    for j in i..k {
                        if (y >> j) & 1 == 1 && q[i][j] & 1 == 1 {
                            quad += 1;
                        }
                    }
                }
            }
            if ph[x as usize] != ABSENT {
                return None;
            }
            ph[x as usize] = ((lin + 2 * quad) & 3) as u8;
        }
        Stab::from_phases(n, &ph)
    }

    /// stabrank's JSON term (lists indexed by qubit, `Q` strictly upper here, `l` mod 4).
    pub fn to_stabrank_json(&self) -> String {
        let n = self.n as usize;
        let k = self.k as usize;
        let bits = |m: u32| -> String {
            let v: Vec<String> = (0..n).map(|c| ((m >> c) & 1).to_string()).collect();
            format!("[{}]", v.join(", "))
        };
        let rows: Vec<String> = self.w.iter().map(|&r| bits(r)).collect();
        let qrows: Vec<String> = (0..k)
            .map(|i| {
                let v: Vec<String> = (0..k).map(|j| (if j > i { (self.q[i] >> j) & 1 } else { 0 }).to_string()).collect();
                format!("[{}]", v.join(", "))
            })
            .collect();
        let ls: Vec<String> = self.l.iter().map(|x| x.to_string()).collect();
        format!(
            "{{\"k\": {}, \"x0\": {}, \"W\": [{}], \"Q\": [{}], \"l\": [{}]}}",
            k,
            bits(self.x0),
            rows.join(", "),
            qrows.join(", "),
            ls.join(", ")
        )
    }

    // ----------------------------------------------------------- symmetry actions ----

    /// Qubit permutation: old qubit `c` goes to position `perm[c]`.
    pub fn permute(&self, perm: &[u8]) -> Stab {
        let ph = self.phases();
        let mut out = vec![ABSENT; ph.len()];
        for (x, &e) in ph.iter().enumerate() {
            if e != ABSENT {
                out[permute_bits(x as u32, perm) as usize] = e;
            }
        }
        Stab::from_phases(self.n as usize, &out).expect("a permuted stabilizer state is one")
    }

    /// Hadamard on every qubit in `mask`, exactly (unnormalised butterflies over `ℤ[i]`).
    pub fn hadamard(&self, mask: u32) -> Stab {
        let mut v = self.gi_vec();
        hadamard_gi(&mut v, mask);
        Stab::recognize_gi(self.n as usize, &v).expect("a Clifford image of a stabilizer state is one")
    }

    /// Dual distance of the direction space: the minimum weight of a nonzero vector
    /// orthogonal to every row of `w` (`n + 1` when the flat is the whole space).
    pub fn dual_distance(&self) -> u32 {
        let n = self.n as u32;
        let mut best = n + 1;
        for u in 1u32..(1 << n) {
            if self.w.iter().all(|&r| (r & u).count_ones() % 2 == 0) {
                best = best.min(u.count_ones());
            }
        }
        best
    }

    /// Is the term visible (nonzero somewhere) at every point of the qubit subset `mask`?
    /// i.e. does its flat project onto all of `F₂^{|mask|}` along those qubits.
    pub fn full_along(&self, mask: u32) -> bool {
        let j = mask.count_ones();
        let mut seen = std::collections::HashSet::new();
        for x in self.points() {
            seen.insert(x & mask);
            if seen.len() == 1usize << j {
                return true;
            }
        }
        seen.len() == 1usize << j
    }

    // ------------------------------------------------------------------- tableaux ----

    /// The preparation circuit from `|0…0⟩` (gates as `(name, a, b)`): H and `S^{l_j}` on each
    /// pivot, CZ for each `Q_ij`, CX from each pivot to its row's other bits, X on `x0`.
    pub fn prep_circuit(&self) -> Vec<(&'static str, usize, usize)> {
        let mut g = Vec::new();
        let piv: Vec<usize> = self.w.iter().map(|r| r.trailing_zeros() as usize).collect();
        for (j, &p) in piv.iter().enumerate() {
            g.push(("h", p, 0));
            for _ in 0..self.l[j] {
                g.push(("s", p, 0));
            }
        }
        for i in 0..piv.len() {
            for j in i + 1..piv.len() {
                if (self.q[i] >> j) & 1 == 1 {
                    g.push(("cz", piv[i], piv[j]));
                }
            }
        }
        for (j, &p) in piv.iter().enumerate() {
            let mut rest = self.w[j] & !(1 << p);
            while rest != 0 {
                let t = rest.trailing_zeros() as usize;
                g.push(("cx", p, t));
                rest &= rest - 1;
            }
        }
        let mut x0 = self.x0;
        while x0 != 0 {
            let t = x0.trailing_zeros() as usize;
            g.push(("x", t, 0));
            x0 &= x0 - 1;
        }
        g
    }

    /// The state as a tier-1 tableau (`crate::tableau::PackedTableau`), by running
    /// [`Stab::prep_circuit`] on it.
    pub fn to_tableau(&self) -> PackedTableau {
        let mut t = PackedTableau::new(self.n as usize);
        for (name, a, b) in self.prep_circuit() {
            match name {
                "h" => t.h(a),
                "s" => t.s(a),
                "x" => t.x_gate(a),
                "cx" => t.cx(a, b),
                "cz" => {
                    t.h(b);
                    t.cx(a, b);
                    t.h(b);
                }
                _ => unreachable!(),
            }
        }
        t
    }

    /// Back from a tableau: the common `+1` eigenvector of its stabilizer rows, built exactly
    /// as `Π_j (I + g_j)|x⟩` for the first basis state `x` it does not annihilate, then
    /// recognised. The rows are Aaronson–Gottesman: `(x, z) = (1, 1)` is `Y`, sign
    /// `(−1)^{r/2}`.
    pub fn from_tableau(t: &PackedTableau) -> Stab {
        let n = t.n;
        let gens: Vec<(u32, u32, u8)> = (n..2 * n)
            .map(|r| {
                let row = &t.rows[r];
                let mut xm = 0u32;
                let mut zm = 0u32;
                for q in 0..n {
                    if row.x.get(q) {
                        xm |= 1 << q;
                    }
                    if row.z.get(q) {
                        zm |= 1 << q;
                    }
                }
                (xm, zm, row.r)
            })
            .collect();
        for start in 0..(1usize << n) {
            let mut v = vec![Gi::ZERO; 1 << n];
            v[start] = Gi::ONE;
            for &(xm, zm, r) in &gens {
                let pv = apply_pauli_gi(&v, xm, zm, r);
                for (a, b) in v.iter_mut().zip(pv) {
                    *a = a.add(b);
                }
            }
            if v.iter().any(|g| !g.is_zero()) {
                return Stab::recognize_gi(n, &v).expect("a tableau's stabilizer state");
            }
        }
        panic!("from_tableau: every basis state annihilated — not a stabilizer tableau");
    }

    // --------------------------------------------------------------------- random ----

    /// A random stabilizer state: `k` drawn with the dictionary's weights, then a uniform
    /// flat and uniform phases.
    pub fn random(n: usize, rng: &mut Rng) -> Stab {
        let weights: Vec<f64> = (0..=n).map(|k| count_with_dim(n, k) as f64).collect();
        let total: f64 = weights.iter().sum();
        let mut u = rng.f64() * total;
        let mut k = n;
        for (kk, w) in weights.iter().enumerate() {
            if u < *w {
                k = kk;
                break;
            }
            u -= w;
        }
        Stab::random_with_dim(n, k, rng)
    }

    pub fn random_with_dim(n: usize, k: usize, rng: &mut Rng) -> Stab {
        let mut basis = Vec::new();
        while basis.len() < k {
            let v = (rng.next() as u32) & ((1u32 << n) - 1);
            rref_insert(&mut basis, v);
        }
        let x0 = reduce_by(&basis, (rng.next() as u32) & ((1u32 << n) - 1));
        let l: Vec<u8> = (0..k).map(|_| (rng.next() & 3) as u8).collect();
        let q: Vec<u32> = (0..k)
            .map(|i| {
                let hi = if i + 1 >= 32 { 0 } else { (!0u32) << (i + 1) };
                (rng.next() as u32) & hi & ((1u32 << k) - 1)
            })
            .collect();
        Stab { n: n as u8, k: k as u8, x0, w: basis, l, q }
    }
}

pub fn permute_bits(x: u32, perm: &[u8]) -> u32 {
    let mut out = 0u32;
    for (c, &p) in perm.iter().enumerate() {
        if (x >> c) & 1 == 1 {
            out |= 1 << p;
        }
    }
    out
}

/// Unnormalised Hadamard butterflies on the qubits of `mask`.
pub fn hadamard_gi(v: &mut [Gi], mask: u32) {
    let size = v.len();
    let mut m = mask;
    while m != 0 {
        let q = m.trailing_zeros();
        let bit = 1usize << q;
        for x in 0..size {
            if x & bit == 0 {
                let (a, b) = (v[x], v[x | bit]);
                v[x] = a.add(b);
                v[x | bit] = a.sub(b);
            }
        }
        m &= m - 1;
    }
}

/// `(−1)^{r/2} · P v` with `P` the Pauli of `(xm, zm)` (`Y = iXZ` where both set).
fn apply_pauli_gi(v: &[Gi], xm: u32, zm: u32, r: u8) -> Vec<Gi> {
    let ny = (xm & zm).count_ones() as u8;
    let mut out = vec![Gi::ZERO; v.len()];
    for (x, &g) in v.iter().enumerate() {
        if g.is_zero() {
            continue;
        }
        // Z^{zm} then X^{xm}, times i^{ny} for the Y's, times the sign
        let zs = ((x as u32 & zm).count_ones() & 1) as u8 * 2;
        let e = (zs + ny + r) & 3;
        out[x ^ xm as usize] = g.mul_unit(e);
    }
    out
}

pub fn phases_to_c64(ph: &[u8]) -> Vec<C64> {
    let cnt = ph.iter().filter(|&&e| e != ABSENT).count();
    let s = 1.0 / (cnt as f64).sqrt();
    ph.iter()
        .map(|&e| if e == ABSENT { C64::default() } else { UNITS[e as usize].scale(s) })
        .collect()
}

// ============================================================== the dictionary ====

/// Number of `k`-dimensional subspaces of `F₂^n` (Gaussian binomial at q = 2).
pub fn gauss_binom(n: usize, k: usize) -> u128 {
    let mut num: u128 = 1;
    let mut den: u128 = 1;
    for i in 0..k {
        num *= (1u128 << (n - i)) - 1;
        den *= (1u128 << (i + 1)) - 1;
    }
    num / den
}

/// States whose flat has dimension `k`: subspaces × cosets × `4^k · 2^{k(k−1)/2}` phases.
pub fn count_with_dim(n: usize, k: usize) -> u128 {
    gauss_binom(n, k) * (1u128 << (n - k)) * (1u128 << (2 * k)) * (1u128 << (k * (k.saturating_sub(1)) / 2))
}

/// `2^n Π_{j=1}^{n} (2^j + 1)`, the number of `n`-qubit stabilizer states.
pub fn count_states(n: usize) -> u128 {
    (1..=n).fold(1u128 << n, |acc, j| acc * ((1u128 << j) + 1))
}

/// Every RREF (pivot-lowest) basis of every `k`-dimensional subspace of `F₂^n`.
pub fn rref_subspaces(n: usize, k: usize) -> Vec<Vec<u32>> {
    let mut out = Vec::new();
    // choose pivots p_0 < ... < p_{k-1}; row j has bit p_j, free bits at non-pivot positions > p_j
    fn rec(n: usize, k: usize, start: usize, piv: &mut Vec<usize>, out: &mut Vec<Vec<u32>>) {
        if piv.len() == k {
            let pivmask: u32 = piv.iter().fold(0, |a, &p| a | (1 << p));
            let frees: Vec<Vec<usize>> = piv
                .iter()
                .map(|&p| (p + 1..n).filter(|&c| (pivmask >> c) & 1 == 0).collect())
                .collect();
            let total_bits: usize = frees.iter().map(|f| f.len()).sum();
            for combo in 0u64..(1u64 << total_bits) {
                let mut rows = Vec::with_capacity(k);
                let mut off = 0;
                for (j, &p) in piv.iter().enumerate() {
                    let mut r = 1u32 << p;
                    for (t, &c) in frees[j].iter().enumerate() {
                        if (combo >> (off + t)) & 1 == 1 {
                            r |= 1 << c;
                        }
                    }
                    off += frees[j].len();
                    rows.push(r);
                }
                out.push(rows);
            }
            return;
        }
        for p in start..n {
            piv.push(p);
            rec(n, k, p + 1, piv, out);
            piv.pop();
        }
    }
    rec(n, k, 0, &mut Vec::new(), &mut out);
    out
}

/// The full dictionary of `n`-qubit stabilizer rays, canonical, in a fixed order.
pub fn enumerate_all(n: usize) -> Vec<Stab> {
    let mut out = Vec::new();
    for k in 0..=n {
        for w in rref_subspaces(n, k) {
            let pivmask: u32 = w.iter().fold(0, |a, r| a | (1 << r.trailing_zeros()));
            let nonpiv: Vec<u32> = (0..n as u32).filter(|c| (pivmask >> c) & 1 == 0).collect();
            for xc in 0u32..(1 << nonpiv.len()) {
                let x0 = nonpiv.iter().enumerate().fold(0u32, |a, (t, &c)| a | (((xc >> t) & 1) << c));
                let nq = k * k.saturating_sub(1) / 2;
                for lc in 0u32..(1 << (2 * k)) {
                    let l: Vec<u8> = (0..k).map(|j| ((lc >> (2 * j)) & 3) as u8).collect();
                    for qc in 0u64..(1u64 << nq) {
                        let mut q = vec![0u32; k];
                        let mut t = 0;
                        for i in 0..k {
                            for j in i + 1..k {
                                if (qc >> t) & 1 == 1 {
                                    q[i] |= 1 << j;
                                }
                                t += 1;
                            }
                        }
                        out.push(Stab { n: n as u8, k: k as u8, x0, w: w.clone(), l: l.clone(), q });
                    }
                }
            }
        }
    }
    out
}

// ================================================================ symmetry group ====

/// The symmetry group of `|H⟩^{⊗n}` the prereg declares: `S_n × H^{⊗subset}`, as
/// `(perm, hmask)` pairs, acting as "Hadamards on `hmask`, then permute". Order `n!·2^n`.
pub fn symmetry_group(n: usize) -> Vec<(Vec<u8>, u32)> {
    let mut perms = Vec::new();
    let mut p: Vec<u8> = (0..n as u8).collect();
    permutations(&mut p, 0, &mut perms);
    let mut g = Vec::with_capacity(perms.len() << n);
    for pm in perms {
        for h in 0..(1u32 << n) {
            g.push((pm.clone(), h));
        }
    }
    g
}

fn permutations(p: &mut Vec<u8>, i: usize, out: &mut Vec<Vec<u8>>) {
    if i == p.len() {
        out.push(p.clone());
        return;
    }
    for j in i..p.len() {
        p.swap(i, j);
        permutations(p, i + 1, out);
        p.swap(i, j);
    }
}

impl Stab {
    pub fn act(&self, g: &(Vec<u8>, u32)) -> Stab {
        let h = if g.1 == 0 { self.clone() } else { self.hadamard(g.1) };
        h.permute(&g.0)
    }
}

/// `act[g][i]` = index of `g · dict[i]`, over the whole group.
pub fn action_table(dict: &[Stab], group: &[(Vec<u8>, u32)]) -> Vec<Vec<u32>> {
    let index: std::collections::HashMap<&Stab, u32> =
        dict.iter().enumerate().map(|(i, s)| (s, i as u32)).collect();
    group
        .iter()
        .map(|g| dict.iter().map(|s| *index.get(&s.act(g)).expect("closed under the group")).collect())
        .collect()
}

/// Orbits of `k`-subsets of the dictionary under the group, by Burnside over EVERY group
/// element (the brute count the prereg asks the reduced enumeration to meet). `k ≤ 3`.
pub fn burnside_subsets(act: &[Vec<u32>], k: usize) -> u128 {
    let mut total: u128 = 0;
    for perm in act {
        let n = perm.len();
        // cycle type
        let mut seen = vec![false; n];
        let mut c = [0u128; 4];
        for i in 0..n {
            if seen[i] {
                continue;
            }
            let mut len = 0;
            let mut j = i;
            while !seen[j] {
                seen[j] = true;
                j = perm[j] as usize;
                len += 1;
            }
            if len <= 3 {
                c[len] += 1;
            }
        }
        let f = match k {
            1 => c[1],
            2 => c[1] * c[1].saturating_sub(1) / 2 + c[2],
            3 => {
                let c1 = c[1];
                c1 * c1.saturating_sub(1) * c1.saturating_sub(2) / 6 + c1 * c[2] + c[3]
            }
            _ => panic!("burnside_subsets: k <= 3"),
        };
        total += f;
    }
    assert_eq!(total % act.len() as u128, 0, "Burnside: orbit count must be an integer");
    total / act.len() as u128
}

/// Is the sorted index set `t` the lexicographically least member of its orbit?
pub fn is_canonical_set(t: &[u32], act: &[Vec<u32>]) -> bool {
    let mut img = vec![0u32; t.len()];
    for perm in act {
        for (a, &x) in img.iter_mut().zip(t) {
            *a = perm[x as usize];
        }
        img.sort_unstable();
        if img.as_slice() < t {
            return false;
        }
    }
    true
}

/// The symmetry-reduced enumeration: canonical `k`-sets generated orderly (a canonical set's
/// prefix without its largest element is canonical — proven in the results note), so the
/// reps of size `k` are the canonical extensions of the reps of size `k − 1`.
pub fn canonical_sets(act: &[Vec<u32>], n_dict: usize, k: usize) -> Vec<Vec<u32>> {
    let mut level: Vec<Vec<u32>> = vec![vec![]];
    for _ in 0..k {
        let mut next = Vec::new();
        for t in &level {
            let start = t.last().map(|&x| x + 1).unwrap_or(0);
            for x in start..n_dict as u32 {
                let mut c = t.clone();
                c.push(x);
                if is_canonical_set(&c, act) {
                    next.push(c);
                }
            }
        }
        level = next;
    }
    level
}

/// Brute orbit count of `k`-subsets: canonicalise every subset by the full group, count
/// distinct forms. Feasible only for small dictionaries.
pub fn brute_orbit_count(act: &[Vec<u32>], n_dict: usize, k: usize) -> usize {
    let mut forms = std::collections::HashSet::new();
    let mut t: Vec<u32> = (0..k as u32).collect();
    loop {
        let mut best = t.clone();
        let mut img = vec![0u32; k];
        for perm in act {
            for (a, &x) in img.iter_mut().zip(&t) {
                *a = perm[x as usize];
            }
            img.sort_unstable();
            if img < best {
                best.clone_from(&img);
            }
        }
        forms.insert(best);
        // next combination
        let mut i = k;
        loop {
            if i == 0 {
                return forms.len();
            }
            i -= 1;
            if (t[i] as usize) < n_dict - k + i {
                t[i] += 1;
                for j in i + 1..k {
                    t[j] = t[j - 1] + 1;
                }
                break;
            }
        }
    }
}

// =================================================================== the filters ====

/// Board values `χ(|H⟩^{⊗j})` for `j = 0..=6` (STABRANK_STATE.md §1; `χ(H^0) = 1`).
pub const CHI_H: [usize; 7] = [1, 2, 2, 3, 4, 6, 6];

/// The slice-visibility filter's per-term reduction: in a rank-`r` decomposition of
/// `|H⟩^{⊗m}`, a `j`-qubit slice point sees at least `χ(H^{m−j})` terms, so when
/// `χ(H^{m−j}) = r` every term is visible at every `j`-qubit point — full along every
/// `j`-subset. Returns the largest such `j` (0 = the filter says nothing per term).
pub fn full_slice_order(m: usize, r: usize) -> usize {
    let mut best = 0;
    for j in 1..m {
        let rest = m - j;
        if rest < CHI_H.len() && CHI_H[rest] >= r {
            best = j;
        }
    }
    best
}

/// Per-term filter at `(m, r)`: full along every `j`-subset for `j ≤ full_slice_order`
/// (dual distance `≥ j + 1`). At `(7, 6)` this is PR 87's property P.
pub fn passes_term_filter(s: &Stab, m: usize, r: usize) -> bool {
    let j = full_slice_order(m, r);
    j == 0 || s.dual_distance() > j as u32
}

/// The slice-visibility filter on a WHOLE tuple: at every `j`-qubit slice point the number
/// of visible terms is at least `χ(H^{m−j})`.
pub fn passes_tuple_slice_filter(terms: &[Stab], m: usize) -> bool {
    for mask in 1u32..(1 << m) {
        let j = mask.count_ones() as usize;
        if j >= m {
            continue;
        }
        let need = CHI_H.get(m - j).copied().unwrap_or(0);
        if need == 0 {
            continue;
        }
        // for each point p of F_2^mask (as a value on those bits), count visible terms
        let pts_per: Vec<std::collections::HashSet<u32>> =
            terms.iter().map(|s| s.points().iter().map(|&x| x & mask).collect()).collect();
        // enumerate sub-assignments of the mask
        let mut sub = mask;
        loop {
            let vis = pts_per.iter().filter(|set| set.contains(&sub)).count();
            if vis < need {
                return false;
            }
            if sub == 0 {
                break;
            }
            sub = (sub - 1) & mask;
        }
    }
    true
}

/// The Galois tuple filter: a decomposition's span contains `a` and `b` (header). On a
/// `(r−1)`-subset of a rank-`r` decomposition, `rank(subset, a, b) ≤ r`. Exact.
pub fn passes_galois_subset(terms: &[Stab], r: usize) -> bool {
    let n = terms[0].n as usize;
    let (a, b) = target_ab(n);
    let mut cols: Vec<Vec<Gi>> = terms.iter().map(|s| s.gi_vec()).collect();
    cols.push(a.iter().map(|&x| Gi::new(x, 0)).collect());
    cols.push(b.iter().map(|&x| Gi::new(x, 0)).collect());
    bareiss_rank(&cols).0 <= r
}

// ================================================================== exact algebra ====

/// Fraction-free (Bareiss) column-rank over `ℤ[i]`. Returns `(rank, pivot columns, pivot rows)`.
pub fn bareiss_rank(cols: &[Vec<Gi>]) -> (usize, Vec<usize>, Vec<usize>) {
    let nc = cols.len();
    let nr = cols[0].len();
    // row-major copy
    let mut a: Vec<Vec<Gi>> = (0..nr).map(|i| cols.iter().map(|c| c[i]).collect()).collect();
    let mut row_id: Vec<usize> = (0..nr).collect();
    let mut prev = Gi::ONE;
    let mut r = 0usize;
    let mut pc = Vec::new();
    let mut pr = Vec::new();
    for col in 0..nc {
        if r == nr {
            break;
        }
        let Some(p) = (r..nr).find(|&i| !a[i][col].is_zero()) else { continue };
        a.swap(r, p);
        row_id.swap(r, p);
        let piv = a[r][col];
        for i in r + 1..nr {
            let f = a[i][col];
            for j in col + 1..nc {
                let t = piv.mul(a[i][j]).sub(f.mul(a[r][j]));
                a[i][j] = t.div_exact(prev).expect("Bareiss: the division is exact");
            }
            a[i][col] = Gi::ZERO;
        }
        prev = piv;
        pc.push(col);
        pr.push(row_id[r]);
        r += 1;
    }
    (r, pc, pr)
}

/// Determinant of a square `ℤ[i]` matrix (rows), fraction-free.
pub fn bareiss_det(m: &[Vec<Gi>]) -> Gi {
    let n = m.len();
    let mut a: Vec<Vec<Gi>> = m.to_vec();
    let mut prev = Gi::ONE;
    let mut sign = false;
    for col in 0..n {
        let Some(p) = (col..n).find(|&i| !a[i][col].is_zero()) else { return Gi::ZERO };
        if p != col {
            a.swap(col, p);
            sign = !sign;
        }
        let piv = a[col][col];
        for i in col + 1..n {
            let f = a[i][col];
            for j in col + 1..n {
                let t = piv.mul(a[i][j]).sub(f.mul(a[col][j]));
                a[i][j] = t.div_exact(prev).expect("Bareiss det: exact division");
            }
            a[i][col] = Gi::ZERO;
        }
        prev = piv;
    }
    if sign {
        prev.neg()
    } else {
        prev
    }
}

/// An exact decomposition: `Σ_j (αa_j + √2 αb_j) s_j = den · (a + √2 b)` over the unnormalised
/// terms, `αa_j, αb_j ∈ ℤ[i]`, `den` a positive integer.
#[derive(Clone, Debug, PartialEq)]
pub struct Witness {
    pub n: usize,
    pub terms: Vec<Stab>,
    pub alpha_a: Vec<Gi>,
    pub alpha_b: Vec<Gi>,
    pub den: i128,
}

/// The exact test: is `v` in the span of `terms`, and if the terms are independent, the
/// exact coefficients. `None` when `a` or `b` is off the span or the terms are dependent.
pub fn exact_solve(terms: &[Stab]) -> Option<Witness> {
    let n = terms[0].n as usize;
    let (a, b) = target_ab(n);
    let mut cols: Vec<Vec<Gi>> = terms.iter().map(|s| s.gi_vec()).collect();
    let r = cols.len();
    cols.push(a.iter().map(|&x| Gi::new(x, 0)).collect());
    cols.push(b.iter().map(|&x| Gi::new(x, 0)).collect());
    let (rank, pc, pr) = bareiss_rank(&cols);
    if rank != r || pc.iter().any(|&c| c >= r) {
        return None;
    }
    // Cramer on the pivot rows
    let rows = &pr[..r];
    let m: Vec<Vec<Gi>> = rows.iter().map(|&i| (0..r).map(|j| cols[j][i]).collect()).collect();
    let det = bareiss_det(&m);
    if det.is_zero() {
        return None;
    }
    let solve = |rhs: &Vec<Gi>| -> Vec<Gi> {
        (0..r)
            .map(|j| {
                let mj: Vec<Vec<Gi>> = rows
                    .iter()
                    .enumerate()
                    .map(|(ri, &i)| (0..r).map(|c| if c == j { rhs[i] } else { m[ri][c] }).collect())
                    .collect();
                bareiss_det(&mj)
            })
            .collect()
    };
    let na = solve(&cols[r]);
    let nb = solve(&cols[r + 1]);
    // integer denominator: multiply through by conj(det)
    let cd = det.conj();
    let mut alpha_a: Vec<Gi> = na.iter().map(|x| x.mul(cd)).collect();
    let mut alpha_b: Vec<Gi> = nb.iter().map(|x| x.mul(cd)).collect();
    let mut den = det.norm();
    let mut g = den;
    for x in alpha_a.iter().chain(&alpha_b) {
        g = gcd(g, gcd(x.re, x.im));
    }
    if g > 1 {
        den /= g;
        for x in alpha_a.iter_mut().chain(alpha_b.iter_mut()) {
            *x = Gi::new(x.re / g, x.im / g);
        }
    }
    let w = Witness { n, terms: terms.to_vec(), alpha_a, alpha_b, den };
    if w.accept_gi() {
        Some(w)
    } else {
        None
    }
}

impl Witness {
    /// Integer check over every entry: `Σ αa_j s_j = den·a` and `Σ αb_j s_j = den·b`.
    pub fn accept_gi(&self) -> bool {
        let (a, b) = target_ab(self.n);
        let vs: Vec<Vec<Gi>> = self.terms.iter().map(|s| s.gi_vec()).collect();
        for x in 0..(1usize << self.n) {
            let mut sa = Gi::ZERO;
            let mut sb = Gi::ZERO;
            for (j, v) in vs.iter().enumerate() {
                sa = sa.add(self.alpha_a[j].mul(v[x]));
                sb = sb.add(self.alpha_b[j].mul(v[x]));
            }
            if sa != Gi::new(self.den * a[x], 0) || sb != Gi::new(self.den * b[x], 0) {
                return false;
            }
        }
        true
    }

    /// The coefficient `α_j = αa_j + √2 αb_j` in the ledger ring.
    pub fn alpha_cyc(&self, j: usize) -> Cyc {
        self.alpha_a[j].to_cyc().add(cyc_sqrt2().mul(self.alpha_b[j].to_cyc()))
    }

    /// THE ACCEPTANCE (prereg §2): `Σ_j α_j s_j = den · v` in `Cyc`, every entry, exactly.
    pub fn accept_cyc(&self) -> bool {
        self.accept_cyc_with(&(0..self.terms.len()).map(|j| self.alpha_cyc(j)).collect::<Vec<_>>())
    }

    /// The same check with caller-supplied coefficients (PR-1 perturbs one).
    pub fn accept_cyc_with(&self, alpha: &[Cyc]) -> bool {
        use crate::affine::cyc_eq;
        let v = target_cyc(self.n);
        let vs: Vec<Vec<Cyc>> = self.terms.iter().map(|s| s.cyc_vec()).collect();
        let d = Cyc { c: [self.den, 0, 0, 0], m: 0 };
        for x in 0..(1usize << self.n) {
            let mut acc = Cyc::ZERO;
            for (j, col) in vs.iter().enumerate() {
                acc = acc.add(alpha[j].mul(col[x]));
            }
            if !cyc_eq(acc, d.mul(v[x])) {
                return false;
            }
        }
        true
    }

    /// Float residual `‖Σ α_j s_j / den − v‖ / ‖v‖` for given coefficients (PR-1's float arm).
    pub fn float_residual_with(&self, alpha: &[Cyc]) -> f64 {
        let v = target_cyc(self.n);
        let vs: Vec<Vec<Gi>> = self.terms.iter().map(|s| s.gi_vec()).collect();
        let mut num = 0.0;
        let mut den2 = 0.0;
        for x in 0..(1usize << self.n) {
            let mut acc = C64::default();
            for (j, col) in vs.iter().enumerate() {
                let (re, im) = alpha[j].to_complex();
                acc = acc.add(C64 { re, im }.mul(col[x].to_c64()));
            }
            let (tr, ti) = v[x].to_complex();
            let t = C64 { re: tr, im: ti };
            num += acc.scale(1.0 / self.den as f64).sub(t).norm2();
            den2 += t.norm2();
        }
        (num / den2).sqrt()
    }

    /// stabrank's upper-bound submission (STABRANK_STATE.md §4). Coefficient of the
    /// normalised term `2^{−k/2} s_j` for the normalised target `cos^m(π/8) v`:
    /// `cos(π/8)^m · α_j · 2^{k_j/2} / den`.
    pub fn to_stabrank_json(&self, author: &str, method: &str, date: &str, compute: &str) -> String {
        let terms: Vec<String> = self.terms.iter().map(|s| format!("      {}", s.to_stabrank_json())).collect();
        let coeffs: Vec<String> = (0..self.terms.len())
            .map(|j| {
                let a = self.alpha_a[j];
                let b = self.alpha_b[j];
                format!(
                    "      \"cos(pi/8)**{m}*(({ar}) + ({ai})*I + sqrt(2)*(({br}) + ({bi})*I))*2**({k}/2)/{den}\"",
                    m = self.n,
                    ar = a.re,
                    ai = a.im,
                    br = b.re,
                    bi = b.im,
                    k = self.terms[j].k,
                    den = self.den
                )
            })
            .collect();
        format!(
            "{{\n  \"schema_version\": \"0.1\",\n  \"orbit\": \"qubit_H\",\n  \"m\": {m},\n  \"direction\": \"upper\",\n  \"rank\": {r},\n  \"provenance\": {{\n    \"author\": \"{author}\",\n    \"reference\": \"CIRISHolon conformance/rank (QVM-RANK-1)\",\n    \"method\": \"{method}\",\n    \"date\": \"{date}\",\n    \"github\": [],\n    \"compute\": {compute}\n  }},\n  \"witness\": {{\n    \"terms\": [\n{terms}\n    ],\n    \"coeffs\": [\n{coeffs}\n    ]\n  }}\n}}\n",
            m = self.n,
            r = self.terms.len(),
            terms = terms.join(",\n"),
            coeffs = coeffs.join(",\n")
        )
    }
}

// ================================================================ float residual ====

/// Squared residual of the normalised targets off the span of the (normalised) columns,
/// by modified Gram–Schmidt. `targets` are unit vectors, so the value is in `[0, #targets]`.
pub fn float_residual(cols: &[Vec<C64>], targets: &[&[C64]]) -> f64 {
    let n = cols[0].len();
    let mut q: Vec<Vec<C64>> = Vec::with_capacity(cols.len());
    for c in cols {
        let mut v = c.clone();
        for _pass in 0..2 {
            for b in &q {
                let mut d = C64::default();
                for i in 0..n {
                    d = d.add(b[i].conj().mul(v[i]));
                }
                for i in 0..n {
                    v[i] = v[i].sub(b[i].mul(d));
                }
            }
        }
        let nr = v.iter().map(|x| x.norm2()).sum::<f64>().sqrt();
        if nr > 1e-9 {
            for x in &mut v {
                *x = x.scale(1.0 / nr);
            }
            q.push(v);
        }
    }
    let mut total = 0.0;
    for t in targets {
        let mut proj = 0.0;
        for b in &q {
            let mut d = C64::default();
            for i in 0..n {
                d = d.add(b[i].conj().mul(t[i]));
            }
            proj += d.norm2();
        }
        total += (1.0 - proj).max(0.0);
    }
    total
}

// ============================================================== pivot completion ====

/// Every stabilizer state lying in `U = span(pivots, a, b)` and outside `span(pivots)`, found
/// completely (module header) and each confirmed exactly. `dim U` must be `pivots.len() + 2`.
pub fn complete_pivot(pivots: &[Stab]) -> Vec<Stab> {
    let n = pivots[0].n as usize;
    let size = 1usize << n;
    let (a, b) = target_ab(n);
    // basis of U, orthonormalised in floats (pruning only)
    let mut raw: Vec<Vec<C64>> = pivots.iter().map(|s| s.c64_vec()).collect();
    raw.push(a.iter().map(|&x| C64 { re: x as f64, im: 0.0 }).collect());
    raw.push(b.iter().map(|&x| C64 { re: x as f64, im: 0.0 }).collect());
    let d = raw.len();
    let mut q: Vec<Vec<C64>> = Vec::new();
    for c in &raw {
        let mut v = c.clone();
        for _ in 0..2 {
            for bq in &q {
                let mut dd = C64::default();
                for i in 0..size {
                    dd = dd.add(bq[i].conj().mul(v[i]));
                }
                for i in 0..size {
                    v[i] = v[i].sub(bq[i].mul(dd));
                }
            }
        }
        let nr = v.iter().map(|x| x.norm2()).sum::<f64>().sqrt();
        if nr < 1e-7 {
            return Vec::new(); // dependent pivots: U is smaller; the caller's tuple is degenerate
        }
        for x in &mut v {
            *x = x.scale(1.0 / nr);
        }
        q.push(v);
    }
    // choose d rows by greedy pivoting on the row vectors of Q (size × d)
    let mut rows: Vec<usize> = Vec::new();
    let mut m: Vec<Vec<C64>> = (0..size).map(|i| (0..d).map(|j| q[j][i]).collect()).collect();
    for _ in 0..d {
        let (best, _) = m
            .iter()
            .enumerate()
            .filter(|(i, _)| !rows.contains(i))
            .map(|(i, r)| (i, r.iter().map(|x| x.norm2()).sum::<f64>()))
            .fold((usize::MAX, -1.0), |acc, x| if x.1 > acc.1 { x } else { acc });
        rows.push(best);
        let pr = m[best].clone();
        let pn = pr.iter().map(|x| x.norm2()).sum::<f64>();
        for r in m.iter_mut() {
            let mut dd = C64::default();
            for j in 0..d {
                dd = dd.add(pr[j].conj().mul(r[j]));
            }
            let f = dd.scale(1.0 / pn);
            for j in 0..d {
                r[j] = r[j].sub(pr[j].mul(f));
            }
        }
    }
    // Z = Q · (Q_R)^{-1}: every u in U is Z · u|_R
    let qr: Vec<Vec<C64>> = rows.iter().map(|&i| (0..d).map(|j| q[j][i]).collect()).collect();
    let inv = invert(&qr);
    let z: Vec<Vec<C64>> = (0..size)
        .map(|i| {
            (0..d)
                .map(|c| {
                    let mut acc = C64::default();
                    for j in 0..d {
                        acc = acc.add(q[j][i].mul(inv[j][c]));
                    }
                    acc
                })
                .collect()
        })
        .collect();
    let others: Vec<usize> = (0..size).filter(|i| !rows.contains(i)).collect();
    let mut found: Vec<Stab> = Vec::new();
    let mut vals = vec![0u8; d]; // 0..3 = unit exponent, 4 = zero
    // enumerate: first nonzero position p gets exponent 0; after it, 5 options
    for p in 0..d {
        let rest = d - 1 - p;
        let total = 5usize.pow(rest as u32);
        for code in 0..total {
            for v in vals.iter_mut().take(p) {
                *v = ABSENT;
            }
            vals[p] = 0;
            let mut c = code;
            for v in vals.iter_mut().skip(p + 1) {
                *v = (c % 5) as u8;
                c /= 5;
            }
            // reconstruct and test each other row
            let mut ok = true;
            let mut ph = vec![ABSENT; size];
            for (t, &i) in rows.iter().enumerate() {
                ph[i] = vals[t];
            }
            for &i in &others {
                let mut s = C64::default();
                for (t, &e) in vals.iter().enumerate() {
                    if e != ABSENT {
                        s = s.add(z[i][t].mul(UNITS[e as usize]));
                    }
                }
                let n2 = s.norm2();
                if n2 < 1e-6 {
                    continue;
                }
                if (n2 - 1.0).abs() > 1e-4 {
                    ok = false;
                    break;
                }
                let e = if s.re > 0.7 {
                    0
                } else if s.im > 0.7 {
                    1
                } else if s.re < -0.7 {
                    2
                } else if s.im < -0.7 {
                    3
                } else {
                    ok = false;
                    break;
                };
                ph[i] = e;
            }
            if !ok {
                continue;
            }
            let Some(st) = Stab::from_phases(n, &ph) else { continue };
            // exact membership: in U and not in span(pivots)
            let mut cols: Vec<Vec<Gi>> = pivots.iter().map(|s| s.gi_vec()).collect();
            let r0 = bareiss_rank(&cols).0;
            cols.push(st.gi_vec());
            let r1 = bareiss_rank(&cols).0;
            if r1 == r0 {
                continue; // inside span(pivots)
            }
            cols.push(a.iter().map(|&x| Gi::new(x, 0)).collect());
            cols.push(b.iter().map(|&x| Gi::new(x, 0)).collect());
            if bareiss_rank(&cols).0 != d {
                continue; // numerically close, exactly not in U
            }
            if !found.contains(&st) {
                found.push(st);
            }
        }
    }
    found
}

fn invert(m: &[Vec<C64>]) -> Vec<Vec<C64>> {
    let n = m.len();
    let mut a: Vec<Vec<C64>> = m
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let mut row = r.clone();
            row.extend((0..n).map(|j| if i == j { C64 { re: 1.0, im: 0.0 } } else { C64::default() }));
            row
        })
        .collect();
    for col in 0..n {
        let p = (col..n)
            .max_by(|&x, &y| a[x][col].norm2().partial_cmp(&a[y][col].norm2()).unwrap())
            .unwrap();
        a.swap(col, p);
        let piv = a[col][col];
        for j in 0..2 * n {
            a[col][j] = a[col][j].div(piv);
        }
        for i in 0..n {
            if i != col {
                let f = a[i][col];
                for j in 0..2 * n {
                    let t = a[col][j].mul(f);
                    a[i][j] = a[i][j].sub(t);
                }
            }
        }
    }
    a.iter().map(|r| r[n..].to_vec()).collect()
}

/// Every rank-`r` decomposition containing `pivots` (`r − 2` of them): pairs of completions
/// that make the whole tuple independent, each accepted exactly.
pub fn witnesses_from_pivot(pivots: &[Stab]) -> Vec<Witness> {
    let comp = complete_pivot(pivots);
    let mut out = Vec::new();
    for i in 0..comp.len() {
        for j in i + 1..comp.len() {
            let mut t = pivots.to_vec();
            t.push(comp[i].clone());
            t.push(comp[j].clone());
            if let Some(w) = exact_solve(&t) {
                out.push(w);
            }
        }
    }
    out
}

// ========================================================================= rng ====

/// SplitMix64 — deterministic, seedable, dependency-free.
#[derive(Clone, Debug)]
pub struct Rng(pub u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    pub fn f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
}

// ===================================================================== annealing ====

/// One move on a phase vector (a stabilizer state, unnormalised). Returns the new phases.
/// Kinds: 0 Pauli, 1 S, 2 CZ, 3 CX, 4 H, 5 fresh random state, 6 H·S·H-style local swap.
pub fn random_move(ph: &[u8], n: usize, rng: &mut Rng, fresh_filter: &dyn Fn(&Stab) -> bool) -> Vec<u8> {
    let size = ph.len();
    let kind = rng.below(100);
    let mut out = vec![ABSENT; size];
    if kind < 25 {
        // Pauli
        let xm = (rng.next() as usize) & (size - 1);
        let zm = (rng.next() as u32) & ((1u32 << n) - 1);
        for x in 0..size {
            if ph[x] != ABSENT {
                let e = ph[x] + 2 * ((x as u32 & zm).count_ones() as u8 & 1);
                out[x ^ xm] = e & 3;
            }
        }
    } else if kind < 40 {
        let q = rng.below(n);
        for x in 0..size {
            if ph[x] != ABSENT {
                out[x] = (ph[x] + ((x >> q) & 1) as u8) & 3;
            }
        }
    } else if kind < 55 {
        let p = rng.below(n);
        let mut q = rng.below(n);
        if q == p {
            q = (q + 1) % n;
        }
        for x in 0..size {
            if ph[x] != ABSENT {
                out[x] = (ph[x] + 2 * (((x >> p) & (x >> q) & 1) as u8)) & 3;
            }
        }
    } else if kind < 70 {
        let c = rng.below(n);
        let mut t = rng.below(n);
        if t == c {
            t = (t + 1) % n;
        }
        for x in 0..size {
            if ph[x] != ABSENT {
                let y = if (x >> c) & 1 == 1 { x ^ (1 << t) } else { x };
                out[y] = ph[x];
            }
        }
    } else if kind < 92 {
        let q = rng.below(n);
        let mut v: Vec<Gi> = ph.iter().map(|&e| if e == ABSENT { Gi::ZERO } else { Gi::unit(e) }).collect();
        hadamard_gi(&mut v, 1 << q);
        let st = Stab::recognize_gi(n, &v).expect("H image");
        return st.phases();
    } else {
        loop {
            let s = Stab::random(n, rng);
            if fresh_filter(&s) {
                return s.phases();
            }
        }
    }
    out
}

/// Configuration of one annealing run.
#[derive(Clone, Debug)]
pub struct AnnealCfg {
    pub n: usize,
    pub rank: usize,
    pub seed: u64,
    /// Moves per restart.
    pub moves: u64,
    pub t0: f64,
    pub t1: f64,
    /// Include the Galois image in the objective.
    pub galois_objective: bool,
    /// Enforce the per-term slice filter on every term.
    pub term_filter: bool,
    /// Residual below which the exact test runs.
    pub exact_threshold: f64,
    /// Snap (pivot-complete every (r−2)-subset) when the residual is below this.
    pub snap_threshold: f64,
}

#[derive(Clone, Debug, Default)]
pub struct AnnealStats {
    pub moves: u64,
    pub accepted: u64,
    pub exact_tests: u64,
    pub snaps: u64,
    pub restarts: u64,
    pub best: f64,
}

/// Run restarts until `stop()` says so or a witness is found. `seeds` (phase vectors) replace
/// the random start on the first restarts, in order.
pub fn anneal(
    cfg: &AnnealCfg,
    seeds: &[Vec<Vec<u8>>],
    stop: &dyn Fn(&AnnealStats) -> bool,
) -> (Option<Witness>, AnnealStats) {
    let n = cfg.n;
    let r = cfg.rank;
    let mut rng = Rng::new(cfg.seed);
    let tv = target_f64(n, false);
    let tg = target_f64(n, true);
    let targets: Vec<&[C64]> = if cfg.galois_objective { vec![&tv, &tg] } else { vec![&tv] };
    let term_ok = |s: &Stab| -> bool { !cfg.term_filter || passes_term_filter(s, n, r) };
    let mut st = AnnealStats { best: f64::INFINITY, ..Default::default() };
    let mut restart = 0usize;
    loop {
        // start
        let mut cur: Vec<Vec<u8>> = if restart < seeds.len() {
            seeds[restart].clone()
        } else {
            (0..r)
                .map(|_| loop {
                    let s = Stab::random(n, &mut rng);
                    if term_ok(&s) {
                        break s.phases();
                    }
                })
                .collect()
        };
        restart += 1;
        st.restarts += 1;
        let mut cols: Vec<Vec<C64>> = cur.iter().map(|p| phases_to_c64(p)).collect();
        let mut f = float_residual(&cols, &targets);
        let mut last_snap = f64::INFINITY;
        for step in 0..cfg.moves {
            let frac = step as f64 / cfg.moves as f64;
            let temp = cfg.t0 * (cfg.t1 / cfg.t0).powf(frac);
            let j = rng.below(r);
            let np = random_move(&cur[j], n, &mut rng, &term_ok);
            if cfg.term_filter {
                match Stab::from_phases(n, &np) {
                    Some(s) if term_ok(&s) => {}
                    _ => continue,
                }
            }
            let old = std::mem::replace(&mut cols[j], phases_to_c64(&np));
            let nf = float_residual(&cols, &targets);
            st.moves += 1;
            if nf <= f || rng.f64() < ((f - nf) / temp).exp() {
                cur[j] = np;
                f = nf;
                st.accepted += 1;
                if f < st.best {
                    st.best = f;
                }
                if f < cfg.exact_threshold {
                    st.exact_tests += 1;
                    let terms: Vec<Stab> = cur.iter().map(|p| Stab::from_phases(n, p).unwrap()).collect();
                    if let Some(w) = exact_solve(&terms) {
                        return (Some(w), st);
                    }
                }
                if f < cfg.snap_threshold && f < last_snap * 0.999 && r >= 3 {
                    last_snap = f;
                    st.snaps += 1;
                    let terms: Vec<Stab> = cur.iter().map(|p| Stab::from_phases(n, p).unwrap()).collect();
                    if let Some(w) = snap(&terms) {
                        return (Some(w), st);
                    }
                }
            } else {
                cols[j] = old;
            }
            if step % 4096 == 0 && stop(&st) {
                return (None, st);
            }
        }
        if stop(&st) {
            return (None, st);
        }
    }
}

/// Complete every `(r−2)`-subset of `terms` exactly (the prereg's "cheap part first", in its
/// complete form). Returns the first witness.
pub fn snap(terms: &[Stab]) -> Option<Witness> {
    let r = terms.len();
    for i in 0..r {
        for j in i + 1..r {
            let piv: Vec<Stab> = (0..r).filter(|&t| t != i && t != j).map(|t| terms[t].clone()).collect();
            if let Some(w) = witnesses_from_pivot(&piv).into_iter().next() {
                return Some(w);
            }
        }
    }
    None
}

// ================================================================ known witnesses ====

/// A known decomposition read from `conformance/rank/known/*.txt` (exported from stabrank by
/// `conformance/rank/export_known.py`): `(m, rank, source, terms)`.
pub type KnownDec = (usize, usize, String, Vec<Stab>);

/// Parse the flat export. Panics on a malformed line (the file is ours).
pub fn parse_known(text: &str) -> Vec<KnownDec> {
    let mut out: Vec<KnownDec> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("dec ") {
            let f: Vec<&str> = rest.split_whitespace().collect();
            out.push((f[0].parse().unwrap(), f[1].parse().unwrap(), f[2].to_string(), Vec::new()));
            continue;
        }
        let dec = out.last_mut().expect("term before any dec line");
        let m = dec.0;
        let mut k = 0usize;
        let mut x0: Vec<u8> = Vec::new();
        let mut w: Vec<Vec<u8>> = Vec::new();
        let mut q: Vec<Vec<u8>> = Vec::new();
        let mut l: Vec<u8> = Vec::new();
        let bits = |s: &str| -> Vec<u8> { s.bytes().map(|c| c - b'0').collect() };
        for tok in line.split_whitespace() {
            let (key, val) = tok.split_once('=').unwrap();
            match key {
                "k" => k = val.parse().unwrap(),
                "x0" => x0 = bits(val),
                "W" if val != "-" => w = val.split(';').map(bits).collect(),
                "Q" if val != "-" => q = val.split(';').map(bits).collect(),
                "l" if val != "-" => {
                    l = val.split(',').map(|v| v.parse::<i64>().unwrap().rem_euclid(4) as u8).collect()
                }
                _ => {}
            }
        }
        assert_eq!(w.len(), k, "known term: W rows != k");
        if q.is_empty() {
            q = vec![vec![0; k]; k];
        }
        if l.is_empty() {
            l = vec![0; k];
        }
        let s = Stab::from_stabrank(m, &x0, &w, &q, &l).expect("known term is a stabilizer state");
        dec.3.push(s);
    }
    out
}

// ============================================================== the slice lift ====

/// Every distinct vector `μ · P · a` (`P = X^x Z^z`, `μ ∈ {±1, ±i}`), as phase vectors. For a
/// stabilizer state `a` on `n` qubits there are `4 · 2^n` of them.
pub fn pauli_orbit(ph: &[u8]) -> Vec<Vec<u8>> {
    let size = ph.len();
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for x in 0..size {
        for z in 0..size {
            let mut v = vec![ABSENT; size];
            for y in 0..size {
                if ph[y] != ABSENT {
                    v[y ^ x] = (ph[y] + 2 * (((y & z) as u32).count_ones() & 1) as u8) & 3;
                }
            }
            for mu in 0..4u8 {
                let w: Vec<u8> = v.iter().map(|&e| if e == ABSENT { ABSENT } else { (e + mu) & 3 }).collect();
                if seen.insert(w.clone()) {
                    out.push(w);
                }
            }
        }
    }
    out
}

impl Witness {
    /// The float coefficients `c_j = α_j / den` of the rescaled target `v`.
    pub fn coeffs_c64(&self) -> Vec<C64> {
        (0..self.terms.len())
            .map(|j| {
                let (re, im) = self.alpha_cyc(j).to_complex();
                C64 { re: re / self.den as f64, im: im / self.den as f64 }
            })
            .collect()
    }
}

/// Statistics of one lift.
#[derive(Clone, Debug, Default)]
pub struct LiftStats {
    pub candidates_per_term: usize,
    pub left: u64,
    pub right: u64,
    pub window_hits: u64,
    pub float_matches: u64,
    pub exact_lifts: u64,
}

/// The slice lift (stabrank's `slice_lift.py` argument, made complete for all-visible
/// slices): every rank-`r` decomposition of `|H⟩^{⊗(n+1)}` whose terms are all visible on both
/// slices of the new qubit restricts at `x_{n+1} = 0` to a rank-`r` decomposition `Σ c_j a_j`
/// of `|H⟩^{⊗n}` (independent terms, so the `c_j` are forced), and at `x_{n+1} = 1` to
/// `Σ c_j b_j = (√2−1)·v_n` with `b_j = μ_j P_j a_j` (the structure lemma of stabrank PR 87).
/// This finds EVERY such `(b_j)` by meet-in-the-middle over `(4·2^n)^{r/2}` half-sums on a
/// random complex functional, confirms each candidate on a second functional, then exactly.
/// Returns every lift, as exact witnesses on `n + 1` qubits.
pub fn lift(base: &Witness, seed: u64) -> (Vec<Witness>, LiftStats) {
    let n = base.n;
    let size = 1usize << n;
    let r = base.terms.len();
    let c = base.coeffs_c64();
    let t = 2f64.sqrt() - 1.0;
    let mut rng = Rng::new(seed);
    let f: Vec<C64> = (0..size).map(|_| C64 { re: rng.f64() - 0.5, im: rng.f64() - 0.5 }).collect();
    let g: Vec<C64> = (0..size).map(|_| C64 { re: rng.f64() - 0.5, im: rng.f64() - 0.5 }).collect();
    let fun = |w: &[C64], ph: &[u8]| -> C64 {
        let mut acc = C64::default();
        for (y, &e) in ph.iter().enumerate() {
            if e != ABSENT {
                acc = acc.add(w[y].mul(UNITS[e as usize]));
            }
        }
        acc
    };
    let (av, bv) = target_ab(n);
    let vn: Vec<C64> = av.iter().zip(&bv).map(|(&a, &b)| C64 { re: a as f64 + 2f64.sqrt() * b as f64, im: 0.0 }).collect();
    let dot = |w: &[C64]| -> C64 { w.iter().zip(&vn).fold(C64::default(), |acc, (x, y)| acc.add(x.mul(*y))) };
    let tf = dot(&f).scale(t);
    let tg = dot(&g).scale(t);
    let cands: Vec<Vec<Vec<u8>>> = base.terms.iter().map(|s| pauli_orbit(&s.phases())).collect();
    let kf: Vec<Vec<C64>> = (0..r).map(|j| cands[j].iter().map(|p| c[j].mul(fun(&f, p))).collect()).collect();
    let kg: Vec<Vec<C64>> = (0..r).map(|j| cands[j].iter().map(|p| c[j].mul(fun(&g, p))).collect()).collect();
    let mut st = LiftStats { candidates_per_term: cands[0].len(), ..Default::default() };
    let half = r / 2;
    let (lo, hi): (Vec<usize>, Vec<usize>) = ((0..half).collect(), (half..r).collect());
    // mixed-radix enumeration of a half
    let sizes: Vec<usize> = (0..r).map(|j| cands[j].len()).collect();
    let count = |idx: &[usize]| -> usize { idx.iter().map(|&j| sizes[j]).product() };
    let decode = |mut code: usize, idx: &[usize]| -> Vec<usize> {
        idx.iter()
            .map(|&j| {
                let p = code % sizes[j];
                code /= sizes[j];
                p
            })
            .collect()
    };
    let nl = count(&lo);
    let mut left: Vec<(f64, f64, u32)> = Vec::with_capacity(nl);
    for code in 0..nl {
        let ps = decode(code, &lo);
        let mut s = C64::default();
        for (t_, &j) in lo.iter().enumerate() {
            s = s.add(kf[j][ps[t_]]);
        }
        left.push((s.re, s.im, code as u32));
    }
    st.left = nl as u64;
    left.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let tol = 1e-7;
    let nr = count(&hi);
    st.right = nr as u64;
    let mut out = Vec::new();
    for code in 0..nr {
        let ps = decode(code, &hi);
        let mut s = tf;
        for (t_, &j) in hi.iter().enumerate() {
            s = s.sub(kf[j][ps[t_]]);
        }
        let start = left.partition_point(|e| e.0 < s.re - tol);
        let mut i = start;
        while i < left.len() && left[i].0 <= s.re + tol {
            st.window_hits += 1;
            if (left[i].1 - s.im).abs() <= tol {
                let pl = decode(left[i].2 as usize, &lo);
                // second functional
                let mut sg = C64::default();
                for (t_, &j) in lo.iter().enumerate() {
                    sg = sg.add(kg[j][pl[t_]]);
                }
                for (t_, &j) in hi.iter().enumerate() {
                    sg = sg.add(kg[j][ps[t_]]);
                }
                if sg.sub(tg).norm2() < 1e-12 {
                    st.float_matches += 1;
                    // exact: build the lifted terms and solve
                    let mut picks = vec![0usize; r];
                    for (t_, &j) in lo.iter().enumerate() {
                        picks[j] = pl[t_];
                    }
                    for (t_, &j) in hi.iter().enumerate() {
                        picks[j] = ps[t_];
                    }
                    let terms: Vec<Stab> = (0..r)
                        .map(|j| {
                            let mut ph = base.terms[j].phases();
                            ph.extend_from_slice(&cands[j][picks[j]]);
                            Stab::from_phases(n + 1, &ph).expect("|0>a + |1>μPa is a stabilizer state")
                        })
                        .collect();
                    if let Some(w) = exact_solve(&terms) {
                        st.exact_lifts += 1;
                        out.push(w);
                    }
                }
            }
            i += 1;
        }
    }
    (out, st)
}

/// Canonical form of a term set under a group given as `(perm, hmask)` pairs: the least sorted
/// image. Equal keys ⇔ same class.
pub fn canonical_set_key(terms: &[Stab], group: &[(Vec<u8>, u32)]) -> Vec<Stab> {
    // Hadamard images once per (term, mask)
    let n = terms[0].n as usize;
    let masks = 1usize << n;
    let himg: Vec<Vec<Stab>> =
        terms.iter().map(|s| (0..masks as u32).map(|h| if h == 0 { s.clone() } else { s.hadamard(h) }).collect()).collect();
    let mut best: Option<Vec<Stab>> = None;
    for (perm, h) in group {
        let mut img: Vec<Stab> = himg.iter().map(|hs| hs[*h as usize].permute(perm)).collect();
        img.sort();
        if best.as_ref().is_none_or(|b| img < *b) {
            best = Some(img);
        }
    }
    best.unwrap()
}

/// Flat text line for a term (the `known/*.txt` format, lists indexed by qubit).
pub fn term_line(s: &Stab) -> String {
    let n = s.n as usize;
    let k = s.k as usize;
    let bits = |m: u32| -> String { (0..n).map(|c| char::from(b'0' + ((m >> c) & 1) as u8)).collect() };
    if k == 0 {
        return format!("k=0 x0={} W=- Q=- l=-", bits(s.x0));
    }
    let wrows: Vec<String> = s.w.iter().map(|&r| bits(r)).collect();
    let qrows: Vec<String> =
        (0..k).map(|i| (0..k).map(|j| if j > i && (s.q[i] >> j) & 1 == 1 { '1' } else { '0' }).collect()).collect();
    let ls: Vec<String> = s.l.iter().map(|x| x.to_string()).collect();
    format!("k={} x0={} W={} Q={} l={}", k, bits(s.x0), wrows.join(";"), qrows.join(";"), ls.join(","))
}

/// A decomposition in the flat text format, with a `dec` header.
pub fn dec_text(terms: &[Stab], source: &str) -> String {
    let mut s = format!("dec {} {} {}\n", terms[0].n, terms.len(), source);
    for t in terms {
        s.push_str(&term_line(t));
        s.push('\n');
    }
    s
}

// ======================================================= members of V = span(a, b) ====

/// `1 − λ_max(B† P_S B)`: how far the span of `cols` is from containing SOME nonzero vector of
/// `V = span(a, b)` (`B` an orthonormal basis of `V`). Zero iff `span(S) ∩ V ≠ 0`.
pub fn vmember_residual(cols: &[Vec<C64>], bv: &[Vec<C64>; 2]) -> f64 {
    let n = cols[0].len();
    let mut q: Vec<Vec<C64>> = Vec::with_capacity(cols.len());
    for c in cols {
        let mut v = c.clone();
        for _ in 0..2 {
            for b in &q {
                let mut d = C64::default();
                for i in 0..n {
                    d = d.add(b[i].conj().mul(v[i]));
                }
                for i in 0..n {
                    v[i] = v[i].sub(b[i].mul(d));
                }
            }
        }
        let nr = v.iter().map(|x| x.norm2()).sum::<f64>().sqrt();
        if nr > 1e-9 {
            for x in &mut v {
                *x = x.scale(1.0 / nr);
            }
            q.push(v);
        }
    }
    let proj: Vec<[C64; 2]> = q
        .iter()
        .map(|qv| {
            let mut out = [C64::default(); 2];
            for (i, o) in out.iter_mut().enumerate() {
                for x in 0..n {
                    *o = o.add(qv[x].conj().mul(bv[i][x]));
                }
            }
            out
        })
        .collect();
    let mut m = [[C64::default(); 2]; 2];
    for p in &proj {
        for i in 0..2 {
            for j in 0..2 {
                m[i][j] = m[i][j].add(p[i].conj().mul(p[j]));
            }
        }
    }
    let (a, d) = (m[0][0].re, m[1][1].re);
    let b2 = m[0][1].norm2();
    let lmax = 0.5 * (a + d) + (0.25 * (a - d) * (a - d) + b2).sqrt();
    (1.0 - lmax).max(0.0)
}

/// Orthonormal basis of `V = span(a, b)` in floats.
pub fn v_basis(n: usize) -> [Vec<C64>; 2] {
    let (a, b) = target_ab(n);
    let mut e0: Vec<C64> = a.iter().map(|&x| C64 { re: x as f64, im: 0.0 }).collect();
    let n0 = e0.iter().map(|x| x.norm2()).sum::<f64>().sqrt();
    for x in &mut e0 {
        *x = x.scale(1.0 / n0);
    }
    let mut e1: Vec<C64> = b.iter().map(|&x| C64 { re: x as f64, im: 0.0 }).collect();
    let d: f64 = e0.iter().zip(&e1).map(|(p, q)| p.re * q.re).sum();
    for (x, p) in e1.iter_mut().zip(&e0) {
        *x = x.sub(p.scale(d));
    }
    let n1 = e1.iter().map(|x| x.norm2()).sum::<f64>().sqrt();
    for x in &mut e1 {
        *x = x.scale(1.0 / n1);
    }
    [e0, e1]
}

/// Does `span(terms)` meet `V` (exactly), with the terms independent?
pub fn meets_v_exact(terms: &[Stab]) -> bool {
    let n = terms[0].n as usize;
    let (a, b) = target_ab(n);
    let mut cols: Vec<Vec<Gi>> = terms.iter().map(|s| s.gi_vec()).collect();
    let r0 = bareiss_rank(&cols).0;
    cols.push(a.iter().map(|&x| Gi::new(x, 0)).collect());
    cols.push(b.iter().map(|&x| Gi::new(x, 0)).collect());
    r0 == terms.len() && bareiss_rank(&cols).0 < r0 + 2
}

/// Complete every `(r1−1)`-subset of `terms` to the `r1`-tuples whose span meets `V`: the
/// extra state lies in `span(pivots, a, b)`, found completely by [`complete_pivot`].
pub fn vmember_snap(terms: &[Stab]) -> Vec<Vec<Stab>> {
    let r = terms.len();
    let mut out = Vec::new();
    for drop in 0..r {
        let piv: Vec<Stab> = (0..r).filter(|&t| t != drop).map(|t| terms[t].clone()).collect();
        for s in complete_pivot(&piv) {
            let mut t = piv.clone();
            t.push(s);
            if meets_v_exact(&t) {
                t.sort();
                if !out.contains(&t) {
                    out.push(t);
                }
            }
        }
    }
    out
}

/// Anneal `r1`-tuples toward `span ∩ V ≠ 0`; return the exact member tuples the snaps find in
/// the first restart that finds any (or none, at `stop`). Same moves as [`anneal`].
pub fn anneal_vmember(
    n: usize,
    r1: usize,
    seed: u64,
    moves: u64,
    term_ok: &dyn Fn(&Stab) -> bool,
    stop: &dyn Fn(&AnnealStats) -> bool,
) -> (Vec<Vec<Stab>>, AnnealStats) {
    let mut rng = Rng::new(seed);
    let bv = v_basis(n);
    let mut st = AnnealStats { best: f64::INFINITY, ..Default::default() };
    let mut found: Vec<Vec<Stab>> = Vec::new();
    loop {
        st.restarts += 1;
        let mut cur: Vec<Vec<u8>> = (0..r1)
            .map(|_| loop {
                let s = Stab::random(n, &mut rng);
                if term_ok(&s) {
                    break s.phases();
                }
            })
            .collect();
        let mut cols: Vec<Vec<C64>> = cur.iter().map(|p| phases_to_c64(p)).collect();
        let mut f = vmember_residual(&cols, &bv);
        let mut last_snap = f64::INFINITY;
        for step in 0..moves {
            let temp = 0.02 * (0.0005f64 / 0.02).powf(step as f64 / moves as f64);
            let j = rng.below(r1);
            let np = random_move(&cur[j], n, &mut rng, term_ok);
            match Stab::from_phases(n, &np) {
                Some(s) if term_ok(&s) => {}
                _ => continue,
            }
            let old = std::mem::replace(&mut cols[j], phases_to_c64(&np));
            let nf = vmember_residual(&cols, &bv);
            st.moves += 1;
            if nf <= f || rng.f64() < ((f - nf) / temp).exp() {
                cur[j] = np;
                f = nf;
                st.accepted += 1;
                st.best = st.best.min(f);
                if f < 0.08 && f < last_snap * 0.999 {
                    last_snap = f;
                    st.snaps += 1;
                    let terms: Vec<Stab> = cur.iter().map(|p| Stab::from_phases(n, p).unwrap()).collect();
                    for t in vmember_snap(&terms) {
                        if !found.contains(&t) {
                            found.push(t);
                        }
                    }
                    if !found.is_empty() {
                        break;
                    }
                }
            } else {
                cols[j] = old;
            }
            if step % 4096 == 0 && stop(&st) {
                return (found, st);
            }
        }
        if !found.is_empty() || stop(&st) {
            return (found, st);
        }
    }
}

/// The member of `V` an independent tuple meets, as `(α, β)` with `member = α a + β b` (exact
/// Gaussian-integer minors, up to a common scale). Tuples with non-proportional `(α, β)`
/// together span `V`, so their union contains `v`.
pub fn vmember_direction(terms: &[Stab]) -> (Gi, Gi) {
    let n = terms[0].n as usize;
    let (a, b) = target_ab(n);
    let mut cols: Vec<Vec<Gi>> = terms.iter().map(|s| s.gi_vec()).collect();
    cols.push(a.iter().map(|&x| Gi::new(x, 0)).collect());
    cols.push(b.iter().map(|&x| Gi::new(x, 0)).collect());
    let (rank, _pc, pr) = bareiss_rank(&cols);
    let r = terms.len();
    assert_eq!(rank, r + 1, "vmember_direction: the tuple must meet V in a line");
    let rows = &pr[..rank];
    let mut kern = Vec::new();
    for c in 0..r + 2 {
        let m: Vec<Vec<Gi>> =
            rows.iter().map(|&i| (0..r + 2).filter(|&cc| cc != c).map(|cc| cols[cc][i]).collect()).collect();
        let d = bareiss_det(&m);
        kern.push(if c % 2 == 0 { d } else { d.neg() });
    }
    // Σ x_j s_j + x_a a + x_b b = 0  ⇒  member = −(x_a a + x_b b)
    (kern[r].neg(), kern[r + 1].neg())
}
