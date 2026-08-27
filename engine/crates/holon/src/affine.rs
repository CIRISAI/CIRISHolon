//! THE AFFINE ENGINE — one port, every consumer.
//!
//! A stabilizer branch in affine (phase-polynomial) form:
//!
//! ```text
//! amplitude(x) = γ · i^{Σ d_a u_a} · (−1)^{Σ_{a<b} J_ab u_a u_b}   on x = R u ⊕ h
//! ```
//!
//! with `R`'s columns INDEPENDENT (so `u ↔ x` is a bijection on the support —
//! every query below needs it, and [`Affine::amplitude`] says so loudly) and
//! `γ` exact in `Z[ω]·2^{−m/2}` ([`crate::ledger::Cyc`]). No floating point
//! enters the exact path; `Cyc::to_complex` is the only exit. The form is
//! Dehaene–De Moor 2003 and Van den Nest 2010 — credited, not ours.
//!
//! # Why this file exists
//!
//! `holon-qasm::magic::Affine` is the certified reference (QASM-2 record, five
//! of five against qiskit), and `holon-qasm` is a DEV-dependency of this crate:
//! `src/` cannot see it, and the isolation gate that keeps it that way is
//! load-bearing (workspace manifest). So the engine has to be transplanted. It
//! was transplanted THREE TIMES, once per lane — pruning, sampling, and the
//! BG-decomposed magic tier — because the lanes ran in parallel and each
//! needed one thing the others did not:
//!
//! | lane | what it added |
//! |---|---|
//! | `prune` | the canonical form: [`Affine::canonicalize`], [`Affine::canon_key`], [`Affine::fingerprint`] |
//! | `sample` | pairwise [`overlap`] and basis-state [`Affine::project`] |
//! | `magic` | [`Affine::attach`], the block-decomposition loader |
//!
//! This module is the UNION of the three, and it is the only copy. The lanes
//! re-export it under the names they were written against, so their call sites
//! and their referees read unchanged.
//!
//! The three copies were diffed method by method before they were merged, and
//! they agreed on every value-producing line. What differed was gauge plumbing
//! (which lane carried which planted mutation), instrumentation ([`GaussStats`]
//! is the sampler's coverage meter), assertion strength, and spelling. Those
//! are reconciled here, each at its site.
//!
//! # The campaign-bought subtleties, kept with their provenance
//!
//! Three lines in this file are not obvious and were each paid for by a
//! measured failure upstream. They carry their receipts:
//!
//! * the odd-δ Gauss sum's XOR expansion — pairwise `J` flips across the
//!   neighbour set, not just per-variable `d` bumps (measured: a sign error on
//!   `amp(1,1)` of `h,cx,h,s,h`, the entangled HSH sandwich);
//! * the column-dependence repair after a row clear (measured: err 0.375 on
//!   `h,h,h,h,cx,h`), and the LOUD rank assertion in [`Affine::amplitude`]
//!   that refuses to answer if the repair ever fails;
//! * the one-shot denominator alignment inside `Cyc::add` (measured: a hang at
//!   Δ ≥ 3 when a loop of √2-multiplies fought `normalize`) — that one lives
//!   in [`crate::ledger`], where the ring does.
//!
//! # One merge law
//!
//! Every accumulation here is `crate::merge::MergeLedger` on `Cyc` — the
//! branch fold behind [`overlap_bruteforce`], the odd-δ Gauss sum's
//! `Σ_w i^{δw}`, and the credit-against-debit posting that [`cyc_eq`] uses to
//! decide exact equality. There is no second addition path in this file.
//!
//! The indexing idioms of the reference (`for c in 0..k` over parallel `Vec`s)
//! are kept even where clippy would rather see an iterator, so that a diff
//! against `holon-qasm::magic` shows only the intended differences.
//!
//! Zero runtime dependencies (`std` only).

use crate::ledger::Cyc;
use crate::merge::MergeLedger;

// ------------------------------------------------------------------ ring helpers
//
// `ledger::Cyc` carries mul/add/to_complex. The powers of ω and i, and an
// equality that is honest across denominators, live here rather than in the
// ledger because they are the engine's business, not the ring's.

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

/// `ω^k`, `ω = e^{iπ/4}`, `ω⁴ = −1`.
pub fn omega_pow(k: u8) -> Cyc {
    let mut c = [0i128; 4];
    let k = k % 8;
    if k < 4 {
        c[k as usize] = 1;
    } else {
        c[(k - 4) as usize] = -1;
    }
    Cyc { c, m: 0 }
}

pub fn cyc_is_zero(a: Cyc) -> bool {
    a.c.iter().all(|&x| x == 0)
}

pub fn cyc_neg(a: Cyc) -> Cyc {
    Cyc { c: [-a.c[0], -a.c[1], -a.c[2], -a.c[3]], m: a.m }
}

/// Complex conjugate: `ω̄ = ω⁻¹ = −ω³`, so `[c0,c1,c2,c3] ↦ [c0,−c3,−c2,−c1]`.
pub fn cyc_conj(a: Cyc) -> Cyc {
    Cyc { c: [a.c[0], -a.c[3], -a.c[2], -a.c[1]], m: a.m }
}

pub fn cyc_sub(a: Cyc, b: Cyc) -> Cyc {
    a.merge(cyc_neg(b))
}

/// EXACT equality ACROSS denominators. `Cyc`'s derived `PartialEq` is NOT a
/// value equality: `√2` has two fixed points of `normalize()`
/// (`{c:[2,0,0,0], m:1}` and `{c:[0,1,0,-1], m:0}`) and the derive calls them
/// different. `{1,ω,ω²,ω³}` is a Z-basis, so at a COMMON `m` the coefficient
/// vector is unique — and posting the credit against the debit goes through
/// the ledger's one-shot denominator alignment, which is faithful.
pub fn cyc_eq(a: Cyc, b: Cyc) -> bool {
    cyc_is_zero(cyc_sub(a, b))
}

// ------------------------------------------------------------------ gates

/// The magic-tier gate set. Declared here rather than imported because
/// `holon-qasm` is a DEV-dependency of this crate: `src/` cannot see it, and
/// the isolation gate that keeps it that way is load-bearing.
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
    pub fn is_t(&self) -> bool {
        matches!(self, Gate::T(_) | Gate::Tdg(_))
    }
}

/// The CLIFFORD-only gate set — the same alphabet with `T`/`T†` removed, so
/// that a consumer which branches on `T` itself (the sampler does) cannot be
/// handed one by accident. `Gate` is the superset; the conversion is total in
/// the direction that is total.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Clif {
    X(usize),
    Z(usize),
    S(usize),
    Sdg(usize),
    H(usize),
    Cx(usize, usize),
}

impl From<Clif> for Gate {
    fn from(c: Clif) -> Gate {
        match c {
            Clif::X(q) => Gate::X(q),
            Clif::Z(q) => Gate::Z(q),
            Clif::S(q) => Gate::S(q),
            Clif::Sdg(q) => Gate::Sdg(q),
            Clif::H(q) => Gate::H(q),
            Clif::Cx(c, t) => Gate::Cx(c, t),
        }
    }
}

// ------------------------------------------------------------------ mutations

/// Planted defects, so a conformance harness can prove it would catch a wrong
/// implementation (the `Mutation` discipline of `holon-qasm`). Default = clean.
///
/// This is the UNION of the three lanes' gauges. A lane sets only the flags it
/// gauges and leaves the rest at `false`, which is exactly the clean engine.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mutations {
    /// Drop the `S` gate's pairwise `J` flips.
    pub drop_s_cross: bool,
    /// Use the wrong odd-δ Gauss-sum phase: `i^δ` instead of `1 + i^δ`, which
    /// drops the `1+` structure. The classic phase slip — it leaves every
    /// magnitude plausible, which is why it is gauged rather than trusted.
    pub wrong_gauss: bool,
    /// [`Affine::flip`] drops the `γ ·= i^{d_p}` factor: the extracted global
    /// scalar is wrong, so a merged weight uses a WRONG PHASE RATIO. Invisible
    /// to the canonical-form check and to the amplitude cross-check (both
    /// states are genuinely equal); only conformance against the naive sum can
    /// see it.
    pub flip_drops_gamma: bool,
    /// [`Affine::canon_key`] omits `h`, so states on DIFFERENT cosets look
    /// equal. The amplitude cross-check is the only remaining guard and must
    /// fire — this is the "never trust the hash alone" defect.
    pub key_ignores_h: bool,
    /// The duplicate-branch merge adds `w₂ · i` instead of `w₂`: a wrong phase
    /// ratio applied at the merge itself, so it fires only on circuits where a
    /// merge happens.
    ///
    /// Read by the DRIVER (`prune::merge_block`), never by the engine — see
    /// the note on `skip_verify`.
    pub merge_phase: bool,
    /// Skip the exact amplitude cross-check on a key match. Only useful paired
    /// with `key_ignores_h`, to let the bad merge through to conformance.
    ///
    /// Read by the DRIVER (`prune::merge_block`), never by the engine. The two
    /// driver flags ride on the state's `Mutations` so that ONE value describes
    /// one gauged run end to end, which is what the gauge matrix reports on.
    pub skip_verify: bool,
}

/// Coverage meter for the Gauss-sum branches.
///
/// "The planted mutation fired" is evidence about the odd-δ Gauss sum only if
/// the odd-δ branch was actually REACHED, and most random state pairs never
/// reach it — they are annihilated by an inconsistent constraint first. So the
/// gauge counts coverage rather than trusting it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GaussStats {
    /// `Σ_{u_a}` with δ odd — the `(1+i^δ)` prefactor, where the mutation lives.
    pub odd_steps: u64,
    /// `Σ_{u_a}` with δ even — a parity CONSTRAINT rather than a phase.
    pub even_steps: u64,
    /// Sums that collapsed to zero (an unsatisfiable parity constraint).
    pub annihilated: u64,
    /// Overlaps that never reached a Gauss sum: disjoint affine subspaces.
    pub inconsistent: u64,
}

// ------------------------------------------------------------------ affine state

/// One stabilizer branch in affine form: `amplitude(x) = γ·i^{d·u}·(−1)^{Q_J(u)}`
/// on `x = R u ⊕ h`, `u ∈ F₂^k`, `R`'s columns independent.
#[derive(Clone, Debug)]
pub struct Affine {
    n: usize,
    /// `R`: n rows × k columns, `x = R u ⊕ h`.
    r: Vec<Vec<bool>>,
    h: Vec<bool>,
    /// `d_a mod 4` (the i-power linear part), one per column.
    d: Vec<u8>,
    /// `J_{ab}` (symmetric, diagonal unused): `(−1)^{J u_a u_b}`.
    j: Vec<Vec<bool>>,
    gamma: Cyc,
    zero: bool,
    mutations: Mutations,
}

impl Affine {
    /// `|0…0⟩` on n qubits.
    pub fn new(n: usize) -> Self {
        Affine::with_mutations(n, Mutations::default())
    }

    pub fn with_mutations(n: usize, mutations: Mutations) -> Self {
        Affine {
            n,
            r: vec![Vec::new(); n],
            h: vec![false; n],
            d: Vec::new(),
            j: Vec::new(),
            gamma: Cyc::ONE,
            zero: false,
            mutations,
        }
    }

    pub fn n_qubits(&self) -> usize {
        self.n
    }

    /// The affine dimension: the number of free parameters `u`, and `2^k` is
    /// the width of the state's support.
    pub fn k(&self) -> usize {
        self.d.len()
    }

    /// True when the branch has been annihilated (an inconsistent projection
    /// or a vanishing Gauss sum). Such a branch contributes exactly nothing.
    pub fn is_zero(&self) -> bool {
        self.zero
    }

    pub fn gamma(&self) -> Cyc {
        self.gamma
    }

    // ---------------------------------------------------------- gauge moves

    /// `u_a := u_a ⊕ u_b` — the F₂ elementary column operation `col_b ^= col_a`,
    /// with the phase polynomial carried along. A bijection of `F₂^k`, so any
    /// sum over `u` is preserved exactly.
    ///
    /// Not to be confused with `merge::fold`: this one carries no ledger.
    fn fold(&mut self, a: usize, b: usize) {
        assert_ne!(a, b);
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
    /// `d += 2`; J itself is unchanged. (Same bookkeeping [`Affine::pin_remove`]
    /// does for the value it pins — this keeps the column.)
    fn flip(&mut self, p: usize) {
        if !self.mutations.flip_drops_gamma {
            self.gamma = self.gamma.mul_i_pow(self.d[p]);
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
            self.gamma = self.gamma.mul_i_pow(self.d[a]);
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

    /// Sum out phase-only column a (its R column is all-zero):
    /// `Σ_w i^{δw} (−1)^{w·Λ}`, `Λ = Σ_{b∈L} u_b`, `L` = the J-neighbours of a.
    ///
    /// `stats` is the sampler's coverage meter; every other caller passes a
    /// throwaway, which costs nothing and keeps ONE Gauss sum in the tree.
    fn gauss_sum_out(&mut self, a: usize, stats: &mut GaussStats) {
        debug_assert!(
            (0..self.n).all(|row| !self.r[row][a]),
            "gauss_sum_out on a column that still carries an x-dependence"
        );
        let delta = self.d[a];
        let l: Vec<usize> = (0..self.k()).filter(|&b| b != a && self.j[a][b]).collect();
        match delta % 4 {
            0 | 2 => {
                // Σ_{u_a} (±1)^{u_a}(−1)^{u_a Λ} = 2·[Λ ≡ eps]: a CONSTRAINT,
                // not a phase.
                stats.even_steps += 1;
                let eps = delta == 2; // the constraint Λ ≡ eps
                if l.is_empty() {
                    if eps {
                        stats.annihilated += 1;
                        self.zero = true;
                        self.remove_col(a);
                        return;
                    }
                    self.gamma.m -= 2; // ×2
                    self.remove_col(a);
                } else {
                    // Impose Σ_L u = eps: fold the rest of L onto c = l[0]
                    // (which makes Λ = u_c exactly), then pin u_c.
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
                // δ odd: ×(1+i^δ), and each b∈L gets d_b += δ+2.
                stats.odd_steps += 1;
                // Σ_w i^{δw} over w ∈ {0,1} is itself a branch sum, so it goes
                // through the one merge law and not an ad-hoc ring add.
                let phase = if self.mutations.wrong_gauss {
                    i_pow(delta) // PLANTED WRONG: drops the 1+ structure
                } else {
                    Cyc::ONE.merge(i_pow(delta))
                };
                self.gamma = self.gamma.mul(phase);
                // (−i^δ)^{Λ mod 2} with Λ = Σ_L u: the XOR expansion — the
                // identity (1+i^δ(−1)^Λ) = (1+i^δ)(−i^δ)^Λ holds only for
                // Λ ∈ {0,1}, and Λ here is an integer sum, so the factorised
                // form needs i^{(δ+2)(⊕_L u)}: per-variable phases PLUS
                // pairwise (−1)^{u_a u_b} flips across L. Concretely
                // ⊕_L u = Σ u_b − 2Σ_{b<b'} u_b u_b' + 4(…) and i^{4(…)} = 1,
                // so the XOR is EXACTLY d-bumps plus pairwise J-flips
                // (measured upstream as a sign error on amp(1,1) of
                // h,cx,h,s,h — the entangled HSH sandwich).
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
            self.gamma = self.gamma.mul_i_pow(2);
        }
        for a in 0..self.k() {
            if self.r[q][a] {
                self.d[a] = (self.d[a] + 2) % 4;
            }
        }
    }

    pub fn s(&mut self, q: usize) {
        // i^{x_q}: γ·i^h, d_a += 1+2h for a ∈ A, J_ab ^= 1 for a<b ∈ A.
        let a_set: Vec<usize> = (0..self.k()).filter(|&a| self.r[q][a]).collect();
        if self.h[q] {
            self.gamma = self.gamma.mul_i_pow(1);
        }
        let bump = if self.h[q] { 3 } else { 1 };
        for &a in &a_set {
            self.d[a] = (self.d[a] + bump) % 4;
        }
        if !self.mutations.drop_s_cross {
            for i in 0..a_set.len() {
                for jj in i + 1..a_set.len() {
                    let (a, b) = (a_set[i], a_set[jj]);
                    self.j[a][b] = !self.j[a][b];
                    self.j[b][a] = self.j[a][b];
                }
            }
        }
    }

    pub fn sdg(&mut self, q: usize) {
        self.s(q);
        self.s(q);
        self.s(q);
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
        // Reduce row q to at most one supporting column a*.
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
        // New variable v; phase (−1)^{(u_{a*} ⊕ h_q)·v}.
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
        // Row-clearing can break column independence: col a* may now equal an
        // XOR of other columns (two columns that differed only at row q
        // collide once the row is cleared — measured upstream as err 0.375 on
        // h,h,h,h,cx,h). If so, fold that subset into a* until it is all-zero,
        // then Gauss-sum it out; the amplitude query REQUIRES independent
        // columns and says so, loudly.
        if let Some(a) = a_star {
            if !(0..self.n).all(|row| !self.r[row][a]) {
                if let Some(subset) = self.dependent_subset(a) {
                    for b in subset {
                        self.fold(b, a);
                    }
                }
            }
            if (0..self.n).all(|row| !self.r[row][a]) {
                self.gauss_sum_out(a, &mut GaussStats::default());
            }
        }
    }

    /// If column a is an XOR of other columns, return that subset.
    fn dependent_subset(&self, a: usize) -> Option<Vec<usize>> {
        let k = self.k();
        let others: Vec<usize> = (0..k).filter(|&b| b != a).collect();
        // Solve [cols others] x = col a over F2.
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

    /// Apply a Clifford gate. `T`/`T†` are the driver's business — they branch,
    /// and a branch is not a state update.
    ///
    /// Generic over the two alphabets: `Gate` (which can name `T`, and is
    /// refused here) and `Clif` (which cannot, so the refusal is unreachable
    /// by construction at those call sites).
    pub fn apply<G: Into<Gate>>(&mut self, g: G) {
        if self.zero {
            return;
        }
        match g.into() {
            Gate::X(q) => self.x(q),
            Gate::Z(q) => self.z(q),
            Gate::S(q) => self.s(q),
            Gate::Sdg(q) => self.sdg(q),
            Gate::H(q) => self.h_gate(q),
            Gate::Cx(c, t) => self.cx(c, t),
            Gate::T(_) | Gate::Tdg(_) => panic!("magic tier branches must be Clifford"),
        }
    }

    /// Tensor a stabilizer block onto the named qubits — the loader a block
    /// decomposition needs to install one of its terms.
    ///
    /// `cols[a]` is column a as a bitmask over `qubits` (bit b ⇒ `R[qubits[b]][a]`),
    /// `h_mask` likewise over `qubits`, `d` and `j` are the block's phase
    /// polynomial. The qubits must be untouched (`|0⟩`, no columns) or the
    /// affine invariant is not this method's to keep.
    pub fn attach(
        &mut self,
        qubits: &[usize],
        cols: &[u32],
        h_mask: u32,
        d: &[u8],
        j: &[Vec<bool>],
    ) {
        let base = self.k();
        for (ci, &mask) in cols.iter().enumerate() {
            let v = self.k();
            for row in 0..self.n {
                self.r[row].push(false);
            }
            self.d.push(d[ci]);
            for jr in &mut self.j {
                jr.push(false);
            }
            self.j.push(vec![false; v + 1]);
            for (bi, &q) in qubits.iter().enumerate() {
                if (mask >> bi) & 1 == 1 {
                    self.r[q][v] = true;
                }
            }
        }
        let kt = cols.len();
        for a in 0..kt {
            for b in a + 1..kt {
                if j[a][b] {
                    self.j[base + a][base + b] = true;
                    self.j[base + b][base + a] = true;
                }
            }
        }
        for (bi, &q) in qubits.iter().enumerate() {
            if (h_mask >> bi) & 1 == 1 {
                self.h[q] = !self.h[q];
            }
        }
    }

    /// Exact amplitude of basis state `y` (bit i = qubit i).
    pub fn amplitude(&self, y: &[bool]) -> Cyc {
        if self.zero {
            return Cyc::ZERO;
        }
        // Solve R u = y ⊕ h.
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
                return Cyc::ZERO; // y is off the affine subspace
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
        let mut amp = self.gamma.mul_i_pow(ip);
        if sign {
            amp = amp.mul_i_pow(2);
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

    // ---------------------------------------------------------- projection

    /// Apply the computational-basis projector `Π_{x_q = v}`.
    ///
    /// An affine state projected on one coordinate is another affine state (or
    /// zero): pinning `x_q` pins one variable, exactly the way a measurement
    /// outcome does. The result is NOT renormalised — its norm is the branch's
    /// contribution to `P(x_q = v)`, which is the whole point.
    pub fn project(&mut self, q: usize, v: bool) {
        if self.zero {
            return;
        }
        let support: Vec<usize> = (0..self.k()).filter(|&a| self.r[q][a]).collect();
        if support.is_empty() {
            // The support already has a definite x_q = h_q.
            if self.h[q] != v {
                self.zero = true;
            }
            return;
        }
        let a = support[0];
        for &b in &support[1..] {
            self.fold(a, b);
        }
        debug_assert!((0..self.k()).all(|c| self.r[q][c] == (c == a)));
        let val = v ^ self.h[q];
        self.pin_remove(a, val);
        debug_assert_eq!(self.h[q], v);
        debug_assert!((0..self.k()).all(|c| !self.r[q][c]));
    }

    pub fn projected(&self, q: usize, v: bool) -> Affine {
        let mut s = self.clone();
        s.project(q, v);
        s
    }

    /// Read `(r, h)` as a linear SYSTEM `r·u = h` rather than a parametrisation,
    /// eliminate it by change of variable, then Gauss-sum every remaining
    /// variable out. Returns `Σ_{u : r u = h} γ·i^{d·u}·(−1)^{Q_J(u)}`.
    ///
    /// [`Affine::fold`] and [`Affine::pin_remove`] happen to be correct for BOTH
    /// readings — a parametrisation's column update and a constraint's column
    /// update are the same F₂ operation — which is why the constraint can be
    /// dispatched with the state machinery instead of a separate solver.
    fn solve_and_sum(mut self, stats: &mut GaussStats) -> Cyc {
        if self.zero {
            return Cyc::ZERO;
        }
        // 1. Eliminate every constraint row.
        loop {
            let mut target: Option<(usize, usize)> = None;
            'scan: for row in 0..self.n {
                for col in 0..self.k() {
                    if self.r[row][col] {
                        target = Some((row, col));
                        break 'scan;
                    }
                }
            }
            let (row, a) = match target {
                Some(t) => t,
                None => break,
            };
            let others: Vec<usize> =
                (0..self.k()).filter(|&b| b != a && self.r[row][b]).collect();
            for b in others {
                self.fold(a, b);
            }
            debug_assert!((0..self.k()).all(|c| self.r[row][c] == (c == a)));
            let val = self.h[row];
            self.pin_remove(a, val);
            debug_assert!(!self.h[row]);
        }
        // 2. A surviving 0 = 1 row means the subspaces are disjoint.
        if self.h.iter().any(|&b| b) {
            stats.inconsistent += 1;
            return Cyc::ZERO;
        }
        // 3. Sum out the free variables.
        while self.k() > 0 {
            self.gauss_sum_out(0, stats);
            if self.zero {
                return Cyc::ZERO;
            }
        }
        self.gamma
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
                self.gauss_sum_out(c, &mut GaussStats::default());
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
    /// global scalar — PROVIDED [`Affine::canonicalize`] ran first
    /// (debug-asserted).
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
        fnv1a(&self.canon_key())
    }

    /// The independent cross-check: do these two canonical states give the SAME
    /// exact amplitude at real basis states, through the same
    /// [`Affine::amplitude`] solver the branch sum uses? Runs the determining
    /// set `{0} ∪ {e_a} ∪ {e_a ⊕ e_b}` (capped at `budget` points), plus two
    /// points off the coset where both must read exactly zero.
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

/// FNV-1a over bytes. The one hash in the engine, so a bucket index and a
/// state fingerprint cannot drift apart.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hsh: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hsh ^= *b as u64;
        hsh = hsh.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hsh
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

// ----------------------------------------------------------------- overlaps

/// Exact `⟨φ|φ'⟩` between two affine states on the same register.
///
/// `O(n·k² + k³)` with `k = k_φ + k_φ' ≤ 2n` — no enumeration of the subspaces.
///
/// `⟨φ|φ'⟩ = Σ_x amp_φ(x)‾ · amp_φ'(x)` is a quadratic-form Gauss sum over F₂
/// restricted to the solution set of `R u ⊕ R' u' = h ⊕ h'`, and it is
/// evaluated in three exact moves: eliminate the constraint by CHANGE OF
/// VARIABLE (`fold`/`pin_remove` transform R and the phase polynomial in
/// lockstep, so the sum is preserved), Gauss-sum the free variables one at a
/// time, and return `γ` — with every variable summed out, `γ` IS the overlap.
pub fn overlap(a: &Affine, b: &Affine) -> Cyc {
    overlap_gauged(a, b, false, &mut GaussStats::default())
}

/// [`overlap`] with the Gauss-sum mutation and the coverage meter exposed, for
/// the gauge. `wrong_gauss` plants the phase error; `stats` records which Gauss
/// branches the call actually reached.
///
/// The mutation is applied to the COMBINED state built here, never to either
/// input — so a gauged overlap cannot perturb the states the sampler holds.
pub fn overlap_gauged(
    a: &Affine,
    b: &Affine,
    wrong_gauss: bool,
    stats: &mut GaussStats,
) -> Cyc {
    assert_eq!(a.n, b.n, "overlap of states on different registers");
    if a.zero || b.zero {
        stats.inconsistent += 1;
        return Cyc::ZERO;
    }
    let (ka, kb) = (a.k(), b.k());
    let k = ka + kb;
    let n = a.n;
    // Constraint: R_a u ⊕ R_b u' = h_a ⊕ h_b (the intersection of the two
    // affine subspaces, parametrised jointly).
    let mut r = vec![vec![false; k]; n];
    let mut h = vec![false; n];
    for row in 0..n {
        r[row][..ka].copy_from_slice(&a.r[row][..ka]);
        r[row][ka..].copy_from_slice(&b.r[row][..kb]);
        h[row] = a.h[row] ^ b.h[row];
    }
    // Phase: conj on the left flips the sign of the i-powers; (−1)^{Q_J} is
    // real, so J is carried over unchanged, block-diagonal in (u, u').
    let mut d = vec![0u8; k];
    for c in 0..ka {
        d[c] = (4 - a.d[c] % 4) % 4;
    }
    d[ka..ka + kb].copy_from_slice(&b.d[..kb]);
    let mut j = vec![vec![false; k]; k];
    for x in 0..ka {
        for y in 0..ka {
            j[x][y] = a.j[x][y];
        }
    }
    for x in 0..kb {
        for y in 0..kb {
            j[ka + x][ka + y] = b.j[x][y];
        }
    }
    let comb = Affine {
        n,
        r,
        h,
        d,
        j,
        gamma: cyc_conj(a.gamma).mul(b.gamma),
        zero: false,
        mutations: Mutations { wrong_gauss, ..Mutations::default() },
    };
    comb.solve_and_sum(stats)
}

/// The referee: `⟨φ|φ'⟩` by summing over all `2^n` basis states. Exact but
/// exponential in the REGISTER, so it exists to test [`overlap`], never to be
/// called by it.
pub fn overlap_bruteforce(a: &Affine, b: &Affine) -> Cyc {
    assert_eq!(a.n, b.n);
    let n = a.n;
    assert!(n <= 20, "brute-force overlap is 2^n by construction");
    crate::merge::fold((0..(1usize << n)).map(|idx| {
        let y: Vec<bool> = (0..n).map(|q| idx >> q & 1 == 1).collect();
        let av = a.amplitude(&y);
        if cyc_is_zero(av) {
            return Cyc::ZERO;
        }
        cyc_conj(av).mul(b.amplitude(&y))
    }))
}

// --------------------------------------------------------------------- rng

/// splitmix64 — a deterministic, seed-reproducible stream. Small on purpose:
/// the crate carries zero runtime dependencies.
#[derive(Clone, Debug)]
pub struct Rng {
    s: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng { s: seed }
    }
    pub fn next_u64(&mut self) -> u64 {
        self.s = self.s.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.s;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// A 32-bit dyadic deviate R, meaning `R/2³² ∈ [0,1)`.
    pub fn next_dyadic32(&mut self) -> u64 {
        self.next_u64() >> 32
    }
    /// A draw in `[0, n)`. Modulo, so it is very slightly biased for `n` that
    /// does not divide `2^64` — fine for picking a qubit or a gate kind, and
    /// NOT fine for a sampling decision, which is why the sampler's own draw
    /// goes through [`Rng::next_dyadic32`] and an exact comparison instead.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}
