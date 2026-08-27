//! EXACT branch pruning for magic-tier stabilizer branch sums.
//!
//! The magic tier writes a Clifford+T circuit as Σ_b c_b |φ_b⟩ over 2^t affine
//! stabilizer branches (`holon-qasm`'s certified `magic` module is the
//! reference; the affine form is Dehaene–De Moor 2003 / Van den Nest 2010,
//! credited there). Two branches can be removed from that sum WITHOUT
//! approximation:
//!
//! 1. ANNIHILATED BRANCH — the branch state is the zero vector (a Gauss sum
//!    over an inconsistent constraint). The reference already skips these.
//! 2. DUPLICATE STATE — two branches whose affine states are THE SAME state up
//!    to a global scalar. Then c₁|φ₁⟩ + c₂|φ₂⟩ = (c₁ + c₂λ)|φ₁⟩ exactly, one
//!    branch disappears, and nothing is lost.
//!
//! Both are optimizations only an EXACT ledger can take. In floating point "the
//! same state up to a scalar" is a tolerance question, and the merged weight
//! would carry the tolerance forward into every later merge. Here the test is a
//! byte comparison of a canonical form over F₂ and Z/4, and the merged weight is
//! an addition in Z[ω]·2^{−m/2} — so a merged sum is not an approximation of the
//! naive sum, it IS the naive sum.
//!
//! # The canonical form (why the fingerprint is sound AND complete)
//!
//! A branch state is `amp(x) = γ · i^{Σ d_a u_a} · (−1)^{Σ_{a<b} J_ab u_a u_b}`
//! on `x = R u ⊕ h`, zero off the coset. The parametrization has exactly two
//! gauge freedoms, and each is killed by a normal form:
//!
//! * COLUMN BASIS `u ↦ M u`, `M ∈ GL(k,2)`. Killed by putting `R` in reduced
//!   column echelon form: RCEF is unique for a given column space, and since `R`
//!   has full column rank, `R M = R` forces `M = I`.
//! * COSET ORIGIN `h ↦ h ⊕ R u₀`. Killed by flipping pivot variables until `h`
//!   is zero on every pivot row; any nonzero element of `col(R)` is nonzero on
//!   some pivot row, so that representative is unique.
//!
//! With both fixed, `(d, J)` is forced by the state itself: `g(e_a) = i^{d_a}`
//! pins `d`, and `g(e_a ⊕ e_b) = i^{d_a + d_b}(−1)^{J_ab}` pins `J`. So
//!
//! > two branches carry the same state up to a global scalar **iff** their
//! > canonical `(R, h, d, J)` tuples are equal.
//!
//! The fingerprint is a hash of that tuple — sound because equality is then
//! re-checked on the full canonical bytes (the hash is never trusted), complete
//! because the tuple is a true normal form.
//!
//! # No division: the scalar ratio is extracted, not computed
//!
//! The brief's "phase ratio" `λ = γ₂/γ₁` never has to be divided out. `γ` is a
//! global factor, so canonicalization STRIPS it: the state is normalized to
//! `γ = 1` and the extracted `γ` is multiplied into the branch weight. Merging
//! is then `c₁.merge(c₂ · λ)` — a single exact ledger post — and `λ` is present
//! implicitly and exactly as `γ₂/γ₁`. This also means a merge can produce an
//! EXACT ZERO weight (`w₁ + w₂ = 0`), which drops another branch; in floating
//! point that cancellation reads as ~1e-16 and the branch survives forever.
//!
//! # One merge law, no bespoke accumulation
//!
//! Every accumulation in this module is `merge::MergeLedger` on `Cyc` — the
//! duplicate-branch coefficient merge, the branch fold behind `amplitude` (via
//! `merge::fold`), the odd-δ Gauss sum's `Σ_w i^{δw}`, and the credit-against-
//! debit posting that `cyc_eq` uses to decide exact equality. There is no
//! second addition path here.
//!
//! What that buys is not tidiness: the law is associative and commutative, so
//! the branch fold can be sharded across the mesh in any order without
//! coordination. `branch_fold_is_shardable` exercises that on real pruned branch
//! lists (forward, reversed, two-shard) rather than inferring it from the trait
//! bound.
//!
//! # What the merge actually finds: the Pauli-orbit ceiling
//!
//! This module was written expecting duplicates to be rare COINCIDENCES, so that
//! random Clifford+T would prune near 0% and only structured circuits would pay.
//! The measurement (`tests/prune.rs::measure_prune_rates`) says otherwise, and
//! the reason is structural, not lucky:
//!
//! Branch `b` differs from branch `b'` only by the `Z`'s the T-expansion
//! inserted. Push those `Z`'s forward through the remaining Clifford gates and
//! each becomes a PAULI, so every branch state is `P_b|ψ⟩` for ONE common
//! stabilizer state `|ψ⟩`. Two Paulis give the same state up to a scalar exactly
//! when they differ by an element of `|ψ⟩`'s stabilizer group, and `b ↦ P_b` mod
//! phase is a homomorphism out of `F₂^t`. So the branch set is a Pauli orbit:
//!
//! > **the merged branch count never exceeds `2^min(t, n)`, whatever the
//! > T-count** — checked on every circuit in `pauli_orbit_bound_holds`.
//!
//! That ceiling, not coincidence, is what the merge is discovering, and it is
//! why the measured rates are large everywhere rather than only on structured
//! input. The honest reading: this optimization buys a lot when `t > n` and
//! nothing at all when `t ≤ n`, which is exactly the regime where `2^t` was
//! affordable anyway. It moves the magic tier's wall from the T-count to the
//! QUBIT count; it does not remove a wall.
//!
//! # What is NOT claimed
//!
//! This is a state-equality merge. It is NOT the Bravyi–Gosset stabilizer-rank
//! decomposition (2^{~0.48t}), which finds a smaller SPANNING set rather than
//! collapsing coincidences, and which beats this one whenever `t < ~2n`. That
//! remains the named next improvement.
//!
//! The annihilated-branch drop (optimization 1) fired ZERO times across every
//! measured class, and cannot fire in this setting: a branch is a Clifford orbit
//! of `|0…0⟩` followed by Paulis, hence always a normalized state. The
//! reference's `zero` guard is defensive, not load-bearing, for branch sums that
//! start from a basis state. It is kept for the same reason.
//!
//! Zero runtime dependencies (`std` only).

use crate::ledger::Cyc;
use crate::merge::{fold, MergeLedger};
use std::collections::HashMap;

// ------------------------------------------------------------------ ring helpers
//
// `ledger::Cyc` carries mul/add/to_complex. Pruning needs three more, and they
// live here rather than in the ledger because they are this module's business.

/// `i^k` as a ring element (`i = ω²`).
pub fn i_pow(k: u8) -> Cyc {
    let mut c = [0i128; 4];
    match k % 4 {
        0 => c[0] = 1,
        1 => c[2] = 1,
        2 => c[0] = -1,
        _ => c[2] = -1,
    }
    Cyc { c, m: 0 }
}

pub fn cyc_is_zero(x: Cyc) -> bool {
    x.c.iter().all(|&v| v == 0)
}

pub fn cyc_neg(x: Cyc) -> Cyc {
    Cyc { c: [-x.c[0], -x.c[1], -x.c[2], -x.c[3]], m: x.m }
}

/// EXACT equality. `Cyc`'s derived `PartialEq` is NOT a value equality: `√2` has
/// two fixed points of `normalize()` (`{c:[2,0,0,0], m:1}` and
/// `{c:[0,1,0,-1], m:0}`) and the derive calls them different. Posting the
/// credit against the debit and testing for `MergeLedger::empty` goes through
/// the ledger's one-shot denominator alignment, which is faithful.
pub fn cyc_eq(a: Cyc, b: Cyc) -> bool {
    cyc_is_zero(a.merge(cyc_neg(b)))
}

// ------------------------------------------------------------------ gates

/// The magic-tier gate set. Declared here rather than imported because
/// `holon-qasm` is a DEV-dependency of this crate: `src/` cannot see it, and the
/// isolation gate that keeps it that way is load-bearing (workspace manifest).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gate {
    X(usize),
    Z(usize),
    S(usize),
    Sdg(usize),
    H(usize),
    Cx(usize, usize),
    T(usize),
    Tdg(usize),
}

impl Gate {
    pub fn is_t(self) -> bool {
        matches!(self, Gate::T(_) | Gate::Tdg(_))
    }
}

/// `CZ(c,t)` as `H(t) · CX(c,t) · H(t)`, appended to `out`. Convenience for the
/// structured circuit classes; CZ is not primitive in the reference gate set.
pub fn push_cz(out: &mut Vec<Gate>, c: usize, t: usize) {
    out.push(Gate::H(t));
    out.push(Gate::Cx(c, t));
    out.push(Gate::H(t));
}

// ------------------------------------------------------------------ mutations

/// Planted defects, so the conformance harness can prove it would catch a wrong
/// implementation (the `Mutation` discipline of `holon-qasm`). Default = clean.
#[derive(Clone, Copy, Debug, Default)]
pub struct Mutations {
    /// `flip()` drops the `γ ·= i^{d_p}` factor: the extracted global scalar is
    /// wrong, so the merged weight uses a WRONG PHASE RATIO. Invisible to the
    /// canonical-form check and to the amplitude cross-check (both states are
    /// genuinely equal); only conformance against the naive sum can see it.
    pub flip_drops_gamma: bool,
    /// The merge adds `w₂ · i` instead of `w₂`: a wrong phase ratio applied at
    /// the merge itself, so it fires only on circuits where a merge happens.
    pub merge_phase: bool,
    /// The canonical key omits `h`, so states on DIFFERENT cosets look equal.
    /// The amplitude cross-check is the only remaining guard and must fire —
    /// this is the "never trust the hash alone" defect.
    pub key_ignores_h: bool,
    /// Skip the exact amplitude cross-check on a key match. Only useful paired
    /// with `key_ignores_h`, to let the bad merge through to conformance.
    pub skip_verify: bool,
}

// ------------------------------------------------------------------ affine state

/// One stabilizer branch in affine form. Ported from the certified reference
/// (`holon-qasm::magic::Affine`) — the Clifford updates are kept semantically
/// identical on purpose, including the two repairs its comments record (the
/// odd-δ Gauss sum's pairwise J flips, and the post-H column-dependence fold).
/// The additions here are `flip`, `rcef`, `canonicalize` and `canon_key`.
///
/// The port keeps the reference's indexing idioms (`for c in 0..k` over parallel
/// `Vec`s) even where clippy would rather see an iterator, so that a diff against
/// `magic.rs` shows only the intended differences. The reference carries the same
/// warnings.
#[derive(Clone)]
pub struct Affine {
    n: usize,
    /// `R`: n rows × k columns, `x = R u ⊕ h`.
    r: Vec<Vec<bool>>,
    h: Vec<bool>,
    /// `d_a mod 4` (i-power linear part), one per column.
    d: Vec<u8>,
    /// `J_{ab}` (symmetric, diagonal unused): `(−1)^{J u_a u_b}`.
    j: Vec<Vec<bool>>,
    gamma: Cyc,
    zero: bool,
    mutations: Mutations,
}

impl Affine {
    pub fn new(n: usize) -> Self {
        Affine {
            n,
            r: vec![Vec::new(); n],
            h: vec![false; n],
            d: Vec::new(),
            j: Vec::new(),
            gamma: Cyc::ONE,
            zero: false,
            mutations: Mutations::default(),
        }
    }

    pub fn with_mutations(n: usize, mutations: Mutations) -> Self {
        let mut a = Self::new(n);
        a.mutations = mutations;
        a
    }

    pub fn n_qubits(&self) -> usize {
        self.n
    }

    /// Number of free parameters `u` — the affine subspace's dimension.
    pub fn k(&self) -> usize {
        self.d.len()
    }

    pub fn is_zero(&self) -> bool {
        self.zero
    }

    pub fn gamma(&self) -> Cyc {
        self.gamma
    }

    // ---------------------------------------------------------- gauge moves

    /// `u_a := u_a ⊕ u_b` — column op `col_b ^= col_a`, with the phase
    /// polynomial carried along.
    ///
    /// Not to be confused with `merge::fold`: this one is an F₂ elementary
    /// column operation on the affine parametrization and carries no ledger.
    /// Call sites read `self.fold(..)`; the ledger fold is the free function.
    fn fold(&mut self, a: usize, b: usize) {
        debug_assert_ne!(a, b);
        for row in 0..self.n {
            let ra = self.r[row][a];
            self.r[row][b] ^= ra;
        }
        let da = self.d[a];
        let jab_old = self.j[a][b];
        let ja_row: Vec<bool> = self.j[a].clone();
        self.d[b] = (self.d[b] + da) % 4;
        self.j[a][b] ^= da & 1 == 1;
        self.j[b][a] = self.j[a][b];
        for c in 0..self.k() {
            if c != a && c != b && ja_row[c] {
                self.j[b][c] = !self.j[b][c];
                self.j[c][b] = self.j[b][c];
            }
        }
        self.d[b] = (self.d[b] + if jab_old { 2 } else { 0 }) % 4;
    }

    /// `u_p := 1 ⊕ u_p` — move the coset origin along column p.
    ///
    /// `Σ_a d_a u_a` picks up `d_p − d_p u'_p`, so a global `i^{d_p}` comes out
    /// and `d_p ↦ −d_p`. The J-terms `J_ap u_a (1 ⊕ u'_p)` split (mod 2) into
    /// the same J-term plus a LINEAR `J_ap u'_a`, so every J-neighbour of p gets
    /// `d += 2`; J itself is unchanged. (Same bookkeeping `pin_remove` does for
    /// the value it pins — this keeps the column.)
    fn flip(&mut self, p: usize) {
        if !self.mutations.flip_drops_gamma {
            self.gamma = self.gamma.mul(i_pow(self.d[p]));
        }
        for a in 0..self.k() {
            if a != p && self.j[p][a] {
                self.d[a] = (self.d[a] + 2) % 4;
            }
        }
        self.d[p] = (4 - self.d[p] % 4) % 4;
        for row in 0..self.n {
            if self.r[row][p] {
                self.h[row] = !self.h[row];
            }
        }
    }

    fn swap_cols(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        for row in 0..self.n {
            self.r[row].swap(a, b);
        }
        self.d.swap(a, b);
        self.j.swap(a, b);
        for jr in &mut self.j {
            jr.swap(a, b);
        }
    }

    /// Remove column a with `u_a` pinned to `val`.
    fn pin_remove(&mut self, a: usize, val: bool) {
        if val {
            for row in 0..self.n {
                if self.r[row][a] {
                    self.h[row] = !self.h[row];
                }
            }
            self.gamma = self.gamma.mul(i_pow(self.d[a]));
            for c in 0..self.k() {
                if c != a && self.j[a][c] {
                    self.d[c] = (self.d[c] + 2) % 4;
                }
            }
        }
        self.remove_col(a);
    }

    fn remove_col(&mut self, a: usize) {
        for row in 0..self.n {
            self.r[row].remove(a);
        }
        self.d.remove(a);
        self.j.remove(a);
        for jr in &mut self.j {
            jr.remove(a);
        }
    }

    /// Sum out a phase-only column (its R column is all-zero):
    /// `Σ_w i^{δw} (−1)^{w·Λ}`, `Λ = Σ_{b∈L} u_b`, `L` = J-neighbours of a.
    fn gauss_sum_out(&mut self, a: usize) {
        let delta = self.d[a];
        let l: Vec<usize> = (0..self.k()).filter(|&b| b != a && self.j[a][b]).collect();
        match delta % 4 {
            0 | 2 => {
                let eps = delta == 2; // constraint Λ ≡ eps
                if l.is_empty() {
                    if eps {
                        self.zero = true;
                        self.remove_col(a);
                        return;
                    }
                    self.gamma.m -= 2; // ×2
                    self.remove_col(a);
                } else {
                    let c = l[0];
                    for &b in &l[1..] {
                        self.fold(c, b);
                    }
                    self.gamma.m -= 2;
                    self.remove_col(a);
                    let c_adj = if c > a { c - 1 } else { c };
                    self.pin_remove(c_adj, eps);
                }
            }
            _ => {
                // δ odd: ×(1+i^δ), each b∈L gets d_b += δ+2, PLUS pairwise
                // (−1)^{u_b1 u_b2} flips across L — the factorized form needs
                // i^{(δ+2)(⊕_L u)}, and Λ is an integer sum, not a bit.
                // Σ_w i^{δw} over w ∈ {0,1} is itself a branch sum, so it is
                // the one merge law too, not an ad-hoc ring add.
                let phase = Cyc::ONE.merge(i_pow(delta));
                self.gamma = self.gamma.mul(phase);
                for &b in &l {
                    self.d[b] = (self.d[b] + delta + 2) % 4;
                }
                for i1 in 0..l.len() {
                    for i2 in i1 + 1..l.len() {
                        let (b1, b2) = (l[i1], l[i2]);
                        self.j[b1][b2] = !self.j[b1][b2];
                        self.j[b2][b1] = self.j[b1][b2];
                    }
                }
                self.remove_col(a);
            }
        }
    }

    // ---------------------------------------------------------- Clifford gates

    pub fn x(&mut self, q: usize) {
        self.h[q] = !self.h[q];
    }

    pub fn z(&mut self, q: usize) {
        if self.h[q] {
            self.gamma = self.gamma.mul(i_pow(2));
        }
        for a in 0..self.k() {
            if self.r[q][a] {
                self.d[a] = (self.d[a] + 2) % 4;
            }
        }
    }

    pub fn s(&mut self, q: usize) {
        let a_set: Vec<usize> = (0..self.k()).filter(|&a| self.r[q][a]).collect();
        if self.h[q] {
            self.gamma = self.gamma.mul(i_pow(1));
        }
        let bump = if self.h[q] { 3 } else { 1 };
        for &a in &a_set {
            self.d[a] = (self.d[a] + bump) % 4;
        }
        for i in 0..a_set.len() {
            for jj in i + 1..a_set.len() {
                let (a, b) = (a_set[i], a_set[jj]);
                self.j[a][b] = !self.j[a][b];
                self.j[b][a] = self.j[a][b];
            }
        }
    }

    pub fn cx(&mut self, c: usize, t: usize) {
        for a in 0..self.k() {
            let rc = self.r[c][a];
            self.r[t][a] ^= rc;
        }
        let hc = self.h[c];
        self.h[t] ^= hc;
    }

    pub fn h_gate(&mut self, q: usize) {
        let support: Vec<usize> = (0..self.k()).filter(|&a| self.r[q][a]).collect();
        let a_star = if support.is_empty() {
            None
        } else {
            let a = support[0];
            for &b in &support[1..] {
                self.fold(a, b);
            }
            Some(a)
        };
        let v = self.k();
        for row in 0..self.n {
            self.r[row].push(false);
        }
        self.d.push(if self.h[q] { 2 } else { 0 });
        for jr in &mut self.j {
            jr.push(false);
        }
        self.j.push(vec![false; v + 1]);
        if let Some(a) = a_star {
            self.j[a][v] = true;
            self.j[v][a] = true;
        }
        for a in 0..self.k() {
            self.r[q][a] = false;
        }
        self.r[q][v] = true;
        self.h[q] = false;
        self.gamma.m += 1;
        // Clearing row q can break column independence; the amplitude query
        // REQUIRES independent columns.
        if let Some(a) = a_star {
            if !(0..self.n).all(|row| !self.r[row][a]) {
                if let Some(subset) = self.dependent_subset(a) {
                    for b in subset {
                        self.fold(b, a);
                    }
                }
            }
            if (0..self.n).all(|row| !self.r[row][a]) {
                self.gauss_sum_out(a);
            }
        }
    }

    /// If column a is an XOR of other columns, return that subset.
    fn dependent_subset(&self, a: usize) -> Option<Vec<usize>> {
        let k = self.k();
        let others: Vec<usize> = (0..k).filter(|&b| b != a).collect();
        let mut rows: Vec<(Vec<bool>, bool)> = (0..self.n)
            .map(|r| (others.iter().map(|&b| self.r[r][b]).collect(), self.r[r][a]))
            .collect();
        let m = others.len();
        let mut piv = vec![usize::MAX; m];
        let mut rr = 0;
        for col in 0..m {
            if let Some(p) = (rr..self.n).find(|&p| rows[p].0[col]) {
                rows.swap(rr, p);
                for p2 in 0..self.n {
                    if p2 != rr && rows[p2].0[col] {
                        let src = rows[rr].clone();
                        rows[p2].0.iter_mut().zip(&src.0).for_each(|(x, y)| *x ^= *y);
                        rows[p2].1 ^= src.1;
                    }
                }
                piv[col] = rr;
                rr += 1;
            }
        }
        if rows[rr..].iter().any(|r| r.1) {
            return None; // independent
        }
        let mut subset = Vec::new();
        for col in 0..m {
            if piv[col] != usize::MAX && rows[piv[col]].1 {
                subset.push(others[col]);
            }
        }
        Some(subset)
    }

    /// Apply a Clifford gate. T/Tdg are the driver's business (they branch).
    pub fn apply_clifford(&mut self, g: Gate) {
        if self.zero {
            return;
        }
        match g {
            Gate::X(q) => self.x(q),
            Gate::Z(q) => self.z(q),
            Gate::S(q) => self.s(q),
            Gate::Sdg(q) => {
                self.s(q);
                self.s(q);
                self.s(q);
            }
            Gate::H(q) => self.h_gate(q),
            Gate::Cx(c, t) => self.cx(c, t),
            Gate::T(_) | Gate::Tdg(_) => panic!("T is not Clifford: the driver branches on it"),
        }
    }

    /// Exact amplitude of basis state `y` (bit i = qubit i).
    pub fn amplitude(&self, y: &[bool]) -> Cyc {
        if self.zero {
            return Cyc::ZERO;
        }
        let k = self.k();
        let mut aug: Vec<(Vec<bool>, bool)> = (0..self.n)
            .map(|row| (self.r[row].clone(), y[row] ^ self.h[row]))
            .collect();
        let mut u = vec![false; k];
        let mut pivot_row = vec![usize::MAX; k];
        let mut rr = 0;
        for col in 0..k {
            if let Some(p) = (rr..self.n).find(|&p| aug[p].0[col]) {
                aug.swap(rr, p);
                for p2 in 0..self.n {
                    if p2 != rr && aug[p2].0[col] {
                        let (head, tail) = if p2 < rr {
                            let (a, b) = aug.split_at_mut(rr);
                            (&mut a[p2], &mut b[0])
                        } else {
                            let (a, b) = aug.split_at_mut(p2);
                            (&mut b[0], &mut a[rr])
                        };
                        for cc in 0..k {
                            head.0[cc] ^= tail.0[cc];
                        }
                        head.1 ^= tail.1;
                    }
                }
                pivot_row[col] = rr;
                rr += 1;
            }
        }
        for row in rr..self.n {
            if aug[row].1 {
                return Cyc::ZERO; // y not in the affine subspace
            }
        }
        assert!(
            (0..k).all(|col| pivot_row[col] != usize::MAX),
            "affine invariant broken: R has dependent columns (rank < k)"
        );
        for col in 0..k {
            u[col] = aug[pivot_row[col]].1;
        }
        let mut ip: u8 = 0;
        let mut sign = false;
        for a in 0..k {
            if u[a] {
                ip = (ip + self.d[a]) % 4;
                for b in a + 1..k {
                    if u[b] && self.j[a][b] {
                        sign = !sign;
                    }
                }
            }
        }
        let mut amp = self.gamma.mul(i_pow(ip));
        if sign {
            amp = amp.mul(i_pow(2));
        }
        amp
    }

    /// The basis state at parameter `u`: `y = R u ⊕ h`.
    fn point(&self, u: &[bool]) -> Vec<bool> {
        let mut y = self.h.clone();
        for (a, &ua) in u.iter().enumerate() {
            if ua {
                for row in 0..self.n {
                    y[row] ^= self.r[row][a];
                }
            }
        }
        y
    }

    // ---------------------------------------------------------- canonical form

    /// Reduced column echelon form of `R`, phase polynomial carried along.
    /// Returns the (pivot row, pivot column) pairs, pivot columns in order.
    fn rcef(&mut self) -> Vec<(usize, usize)> {
        let mut pivots = Vec::new();
        let mut p = 0usize;
        for row in 0..self.n {
            if p >= self.k() {
                break;
            }
            let Some(c) = (p..self.k()).find(|&c| self.r[row][c]) else {
                continue;
            };
            self.swap_cols(c, p);
            for c2 in 0..self.k() {
                if c2 != p && self.r[row][c2] {
                    self.fold(p, c2); // col_{c2} ^= col_p
                }
            }
            pivots.push((row, p));
            p += 1;
        }
        pivots
    }

    /// Put the state in canonical form and RETURN the stripped global scalar.
    ///
    /// Afterwards `gamma == Cyc::ONE`, `R` is RCEF with full column rank, `h` is
    /// zero on every pivot row, and `(d, J)` is forced. Two states are equal up
    /// to a global scalar iff their canonical forms agree — so the caller
    /// multiplies the returned scalar into the branch weight and compares the
    /// rest byte for byte. A zero-flagged state returns `Cyc::ZERO`.
    pub fn canonicalize(&mut self) -> Cyc {
        loop {
            if self.zero {
                self.gamma = Cyc::ONE;
                return Cyc::ZERO;
            }
            let pivots = self.rcef();
            // Any column left all-zero is dependent (rank < k): it is a
            // phase-only variable, so sum it out and start over. k strictly
            // decreases, so this terminates.
            if let Some(c) = (0..self.k()).find(|&c| (0..self.n).all(|row| !self.r[row][c])) {
                self.gauss_sum_out(c);
                continue;
            }
            for &(row, col) in &pivots {
                if self.h[row] {
                    self.flip(col);
                }
            }
            break;
        }
        let g = self.gamma;
        self.gamma = Cyc::ONE;
        g
    }

    /// Byte encoding of the canonical form. Equal keys ⇔ equal states up to a
    /// global scalar — PROVIDED `canonicalize` ran first (debug-asserted).
    pub fn canon_key(&self) -> Vec<u8> {
        debug_assert!(
            self.zero || cyc_eq(self.gamma, Cyc::ONE),
            "canon_key on a non-canonical state"
        );
        let k = self.k();
        let mut out = Vec::with_capacity(8 + self.n * (k / 8 + 1) + k + k * k / 8 + 2);
        out.push(self.zero as u8);
        out.extend_from_slice(&(self.n as u32).to_le_bytes());
        out.extend_from_slice(&(k as u32).to_le_bytes());
        let mut bits = BitWriter::new(&mut out);
        for row in 0..self.n {
            for a in 0..k {
                bits.push(self.r[row][a]);
            }
        }
        if !self.mutations.key_ignores_h {
            for row in 0..self.n {
                bits.push(self.h[row]);
            }
        }
        for a in 0..k {
            for b in a + 1..k {
                bits.push(self.j[a][b]);
            }
        }
        bits.finish();
        out.extend_from_slice(&self.d);
        out
    }

    /// FNV-1a of the canonical key. A hash, and treated as one: a bucket hit is
    /// only ever a hint, never a decision.
    pub fn fingerprint(&self) -> u64 {
        let mut hsh: u64 = 0xcbf2_9ce4_8422_2325;
        for b in self.canon_key() {
            hsh ^= b as u64;
            hsh = hsh.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hsh
    }

    /// The independent cross-check: do these two canonical states give the SAME
    /// exact amplitude at real basis states, through the same `amplitude` solver
    /// the branch sum uses? Runs the determining set `{0} ∪ {e_a} ∪ {e_a ⊕ e_b}`
    /// (capped at `budget` points), plus two points off the coset where both
    /// must read exactly zero.
    pub fn amplitudes_agree(&self, other: &Affine, budget: usize) -> bool {
        if self.n != other.n {
            return false;
        }
        let k = self.k();
        let mut checked = 0usize;
        let probe = |u: &[bool]| -> bool {
            let y = self.point(u);
            cyc_eq(self.amplitude(&y), other.amplitude(&y))
        };
        if !probe(&vec![false; k]) {
            return false;
        }
        checked += 1;
        for a in 0..k {
            if checked >= budget {
                break;
            }
            let mut u = vec![false; k];
            u[a] = true;
            if !probe(&u) {
                return false;
            }
            checked += 1;
        }
        'pairs: for a in 0..k {
            for b in a + 1..k {
                if checked >= budget {
                    break 'pairs;
                }
                let mut u = vec![false; k];
                u[a] = true;
                u[b] = true;
                if !probe(&u) {
                    return false;
                }
                checked += 1;
            }
        }
        // Off-coset probes: flip a non-pivot bit of a coset point. If the two
        // states have different supports (the `key_ignores_h` defect), one of
        // these — or the on-coset probes above — must disagree.
        let base = self.point(&vec![false; k]);
        for q in 0..self.n {
            let mut y = base.clone();
            y[q] = !y[q];
            if !cyc_eq(self.amplitude(&y), other.amplitude(&y)) {
                return false;
            }
        }
        true
    }
}

struct BitWriter<'a> {
    out: &'a mut Vec<u8>,
    cur: u8,
    n: u8,
}

impl<'a> BitWriter<'a> {
    fn new(out: &'a mut Vec<u8>) -> Self {
        BitWriter { out, cur: 0, n: 0 }
    }
    fn push(&mut self, b: bool) {
        if b {
            self.cur |= 1 << self.n;
        }
        self.n += 1;
        if self.n == 8 {
            self.out.push(self.cur);
            self.cur = 0;
            self.n = 0;
        }
    }
    fn finish(self) {
        if self.n > 0 {
            self.out.push(self.cur);
        }
    }
}

// ------------------------------------------------------------------ driver

#[derive(Clone, Copy, Debug)]
pub struct PruneConfig {
    /// T-gates per block: duplicates are merged at each block boundary.
    /// `1` merges after every T (the aggressive default); `t` merges once at the
    /// end, which is the naive sum plus one pass.
    pub merge_every: usize,
    /// Exact amplitude cross-checks per candidate merge. `usize::MAX` runs the
    /// whole determining set.
    pub verify_points: usize,
    /// Refuse to grow the working set past this — a magic tier that silently
    /// blows up is worse than one that stops.
    pub max_working_set: usize,
    /// Canonicalize and drop annihilated branches, but never merge duplicates.
    /// The in-module control for a prune rate.
    pub disable_merge: bool,
    pub mutations: Mutations,
}

impl Default for PruneConfig {
    fn default() -> Self {
        PruneConfig {
            merge_every: 1,
            verify_points: 64,
            max_working_set: 1 << 22,
            disable_merge: false,
            mutations: Mutations::default(),
        }
    }
}

/// What the pruning actually achieved. Every field is a count, not a rate; the
/// rates are derived so the raw numbers stay auditable.
#[derive(Clone, Debug, Default)]
pub struct PruneStats {
    pub t_count: usize,
    /// `2^t`, saturating (the naive branch count).
    pub naive_branches: u128,
    /// Branches surviving the final merge.
    pub final_branches: usize,
    /// Largest working set held at any moment.
    pub peak_working_set: usize,
    /// Duplicate states folded into an existing branch.
    pub merged_duplicates: usize,
    /// Branches dropped because the state was annihilated (`zero`).
    pub zero_states_dropped: usize,
    /// Branches dropped because a merge produced an EXACT zero weight. Only an
    /// exact ledger can see these.
    pub exact_cancellations: usize,
    /// Fingerprint bucket hits rejected by the full-key comparison. Should be 0;
    /// a nonzero count means the hash is doing work it must not be trusted for.
    pub hash_collisions_rejected: usize,
    /// Key matches rejected by the exact amplitude cross-check. Should be 0 on a
    /// clean build; nonzero means the canonical form is unsound (or planted).
    pub verify_rejections: usize,
    /// `(t-gates consumed, branches before merge, branches after)` per block.
    pub blocks: Vec<(usize, usize, usize)>,
}

impl PruneStats {
    /// Surviving fraction of the naive `2^t`. 1.0 = nothing pruned.
    pub fn survival(&self) -> f64 {
        if self.naive_branches == 0 {
            return 1.0;
        }
        self.final_branches as f64 / self.naive_branches as f64
    }
    /// `log2(2^t / final)` — the T-gates the pruning gave back.
    pub fn t_saved(&self) -> f64 {
        if self.final_branches == 0 {
            return self.t_count as f64;
        }
        self.t_count as f64 - (self.final_branches as f64).log2()
    }
}

#[derive(Clone)]
pub struct Branch {
    /// `c_b · γ_b` — the T-expansion coefficient with the state's global scalar
    /// already folded in, which is what makes merging a plain addition.
    pub weight: Cyc,
    pub state: Affine,
}

/// The pruned branch sum: `Σ_b weight_b · |φ̂_b⟩` with every `|φ̂_b⟩` canonical,
/// distinct, and nonzero. Exactly equal to the naive sum.
pub struct PrunedSum {
    pub n: usize,
    pub branches: Vec<Branch>,
    pub stats: PruneStats,
}

impl PrunedSum {
    /// Exact amplitude of one basis state: `final_branches · poly(n)` work.
    ///
    /// The branch fold is `merge::fold`, so it is associative and commutative by
    /// the one law and therefore shardable across the mesh without coordination
    /// — which is the warrant `BranchSource`'s doc comment claims for it.
    pub fn amplitude(&self, y: &[bool]) -> Cyc {
        fold(self.branches.iter().filter_map(|b| {
            let a = b.state.amplitude(y);
            if cyc_is_zero(a) {
                None
            } else {
                Some(b.weight.mul(a))
            }
        }))
    }

    /// Full exact state vector, index `i` = the basis state with bit q = i>>q&1.
    /// Costs `2^n` by construction — for conformance on small n, not for use.
    pub fn state_vector(&self) -> Vec<Cyc> {
        let dim = 1usize << self.n;
        let mut out = vec![Cyc::ZERO; dim];
        let mut y = vec![false; self.n];
        for (idx, slot) in out.iter_mut().enumerate() {
            for (q, yq) in y.iter_mut().enumerate() {
                *yq = idx >> q & 1 == 1;
            }
            *slot = self.amplitude(&y);
        }
        out
    }
}

impl crate::BranchSource for PrunedSum {
    fn n_branches(&self) -> u64 {
        self.branches.len() as u64
    }
    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        let b = &self.branches[branch as usize];
        b.weight.mul(b.state.amplitude(y))
    }
    fn n_qubits(&self) -> usize {
        self.n
    }
}

/// `2^t`, saturating. `t` past 127 is not a number anyone is going to reach.
pub fn naive_branch_count(gates: &[Gate]) -> u128 {
    let t = gates.iter().filter(|g| g.is_t()).count();
    if t >= 127 {
        u128::MAX
    } else {
        1u128 << t
    }
}

/// `T = ((1+ω)/2) I + ((1−ω)/2) Z`; `T† ` with `ω ↦ ω⁻¹ = −ω³`.
fn t_coeffs(dagger: bool) -> (Cyc, Cyc) {
    if !dagger {
        (Cyc { c: [1, 1, 0, 0], m: 2 }, Cyc { c: [1, -1, 0, 0], m: 2 })
    } else {
        (Cyc { c: [1, 0, 0, -1], m: 2 }, Cyc { c: [1, 0, 0, 1], m: 2 })
    }
}

/// Canonicalize, drop annihilated branches, merge duplicates, drop exact
/// cancellations. Every surviving branch is canonical and pairwise distinct.
fn merge_block(branches: &mut Vec<Branch>, cfg: &PruneConfig, stats: &mut PruneStats) {
    let before = branches.len();
    let mut kept: Vec<Branch> = Vec::with_capacity(before);
    let mut keys: Vec<Vec<u8>> = Vec::with_capacity(before);
    let mut buckets: HashMap<u64, Vec<usize>> = HashMap::new();

    for mut b in branches.drain(..) {
        let g = b.state.canonicalize();
        if b.state.is_zero() {
            stats.zero_states_dropped += 1;
            continue;
        }
        b.weight = b.weight.mul(g);
        if cyc_is_zero(b.weight) {
            // A T-expansion coefficient is never zero and γ is never zero, so
            // this only happens if a caller handed us a zero weight.
            stats.exact_cancellations += 1;
            continue;
        }
        if cfg.disable_merge {
            kept.push(b);
            continue;
        }
        let key = b.state.canon_key();
        let fp = {
            let mut hsh: u64 = 0xcbf2_9ce4_8422_2325;
            for byte in &key {
                hsh ^= *byte as u64;
                hsh = hsh.wrapping_mul(0x0000_0100_0000_01b3);
            }
            hsh
        };
        let mut target = None;
        for &idx in buckets.get(&fp).map(|v| v.as_slice()).unwrap_or(&[]) {
            if keys[idx] != key {
                // The hash agreed and the states did not. This is exactly the
                // case the brief refuses to trust a hash for.
                stats.hash_collisions_rejected += 1;
                continue;
            }
            if !cfg.mutations.skip_verify
                && !kept[idx].state.amplitudes_agree(&b.state, cfg.verify_points)
            {
                stats.verify_rejections += 1;
                continue;
            }
            target = Some(idx);
            break;
        }
        match target {
            Some(idx) => {
                // THE duplicate-branch coefficient merge, and it is the one
                // merge law: `c₁.merge(c₂ · λ)`. The phase ratio λ = γ₂/γ₁ is
                // already inside `b.weight` — canonicalization multiplied the
                // stripped γ in — so the ledger op is all that is left.
                let posted = if cfg.mutations.merge_phase {
                    b.weight.mul(i_pow(1))
                } else {
                    b.weight
                };
                kept[idx].weight = kept[idx].weight.merge(posted);
                stats.merged_duplicates += 1;
            }
            None => {
                buckets.entry(fp).or_default().push(kept.len());
                keys.push(key);
                kept.push(b);
            }
        }
    }

    // Exact cancellation: a merged weight that is exactly zero removes the
    // branch outright. Floating point reads these as ~1e-16 and keeps them.
    let mut out = Vec::with_capacity(kept.len());
    for b in kept {
        if cyc_is_zero(b.weight) {
            stats.exact_cancellations += 1;
        } else {
            out.push(b);
        }
    }
    stats.blocks.push((stats.blocks.len(), before, out.len()));
    *branches = out;
}

/// Run the circuit as a PRUNED branch sum.
///
/// Breadth-first over T-choices in blocks of `cfg.merge_every`, merging at each
/// block boundary. This is the same enumeration a depth-first walk performs, but
/// the working set is bounded by `merged · 2^block` instead of the recursion's
/// full subtree — with `merge_every = 1` the working set never exceeds twice the
/// merged count, which is the whole point.
pub fn run_pruned(n: usize, gates: &[Gate], cfg: &PruneConfig) -> PrunedSum {
    let mut stats = PruneStats {
        t_count: gates.iter().filter(|g| g.is_t()).count(),
        naive_branches: naive_branch_count(gates),
        ..PruneStats::default()
    };

    let mut branches = vec![Branch {
        weight: Cyc::ONE,
        state: Affine::with_mutations(n, cfg.mutations),
    }];
    let block = cfg.merge_every.max(1);
    let mut since_merge = 0usize;

    for &g in gates {
        match g {
            Gate::T(q) | Gate::Tdg(q) => {
                let (ci, cz) = t_coeffs(matches!(g, Gate::Tdg(_)));
                assert!(
                    branches.len() * 2 <= cfg.max_working_set,
                    "pruned working set would exceed max_working_set ({}) at T #{}",
                    cfg.max_working_set,
                    since_merge
                );
                let mut next = Vec::with_capacity(branches.len() * 2);
                for b in branches.drain(..) {
                    let mut zb = b.clone();
                    zb.weight = zb.weight.mul(cz);
                    zb.state.z(q);
                    let mut ib = b;
                    ib.weight = ib.weight.mul(ci);
                    next.push(ib);
                    next.push(zb);
                }
                branches = next;
                stats.peak_working_set = stats.peak_working_set.max(branches.len());
                since_merge += 1;
                if since_merge >= block {
                    merge_block(&mut branches, cfg, &mut stats);
                    since_merge = 0;
                }
            }
            other => {
                for b in &mut branches {
                    b.state.apply_clifford(other);
                }
            }
        }
    }
    merge_block(&mut branches, cfg, &mut stats);
    stats.peak_working_set = stats.peak_working_set.max(branches.len());
    stats.final_branches = branches.len();
    PrunedSum { n, branches, stats }
}

/// The unpruned sum: the same code path with merging switched off, so the only
/// branches it loses are the annihilated ones the reference already skips. This
/// is the in-module control a prune rate is quoted against; `holon-qasm::magic`
/// is the INDEPENDENT referee, and conformance is stated against that.
pub fn run_naive(n: usize, gates: &[Gate]) -> PrunedSum {
    let cfg = PruneConfig { disable_merge: true, ..PruneConfig::default() };
    run_pruned(n, gates, &cfg)
}
