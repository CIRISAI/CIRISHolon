//! EXACT sampling from a magic-tier branch sum — bitstrings, not amplitudes.
//!
//! A magic-tier state is |ψ⟩ = Σ_b c_b |φ_b⟩ over stabilizer branches with
//! exact Z[ω] coefficients (`ledger::Cyc`). Drawing a shot means drawing
//! x ~ |⟨x|ψ⟩|², and THAT is the known-hard half of every stabilizer-rank
//! method: the amplitude of ONE basis state costs 2^t, but the NORM of the
//! state — which every conditional probability is a ratio of — has no such
//! cheap form. Production tools (Bravyi–Gosset–Howard–Kliuchnikov–Mengoni–
//! Mishmash, Simulation of quantum circuits by low-rank stabilizer
//! decompositions, 2019; the `stim`/`qiskit` extended-stabilizer backends)
//! therefore estimate the norm by random sampling of stabilizer inner
//! products and pay an ε-error. This module does NOT. It computes every
//! conditional probability exactly and pays the honest price for it.
//!
//! ## The merge law
//!
//! Every accumulation here — the Gram sum, the branch fold, the brute-force
//! reference — is an instance of `holon::merge::MergeLedger` and goes through
//! `merge::fold`. There is no bespoke summation in this module. That is what
//! makes the quadratic Gram sum shardable: the terms are independent ledger
//! entries, so any ordering or distribution of the fold lands in the same
//! place (`gram_fold_is_shard_invariant` exercises exactly that on the
//! sampler's own accumulation). The Gauss prefactor `1 + i^δ` used to be
//! carried here as a deliberate EXCEPTION — a closed-form phase value rather
//! than an accumulation. It is no longer: the Gauss sum now lives in the one
//! shared engine (`crate::affine`), which posts it through the merge law like
//! every other addition, so this module has no exception left to declare.
//!
//! ## What is exact here
//!
//! Every probability on the conditional chain is an exact element of
//! Z[ω]·2^{−m/2}. No floating point enters the *probabilities* at any point,
//! and the sampler's own branch decision is an exact integer comparison
//! (`cyc_real_cmp`) against a dyadic deviate — see "the one approximation"
//! below. The identity P(prefix) = P(prefix·0) + P(prefix·1) is asserted as
//! an EXACT ring identity at every node, not as a tolerance.
//!
//! ## The one approximation, named
//!
//! The uniform deviate is a 32-bit dyadic r = R/2³². The comparison
//! r·P(prefix) < P(prefix·0) is then exact integer arithmetic. So the
//! sampler is a perfectly faithful sampler of a distribution that differs
//! from the true one by the deviate's own quantisation (≤ 2⁻³² per node),
//! which is what "finite randomness" costs any sampler. `approx_compares()`
//! counts any comparison that had to fall back to f64 because an i128
//! product overflowed; a run with a zero count used no floating point in
//! any decision, and the tests assert zero.
//!
//! ## The cost, stated plainly
//!
//! P(x₁..x_k = prefix) = ⟨ψ|Π|ψ⟩ = Σ_{b,b'} c̄_b c_{b'} ⟨φ_b|Π|φ_{b'}⟩ is a
//! GRAM sum: **quadratic in the branch count**. With B = 2^t branches:
//!
//! | quantity | cost |
//! |---|---|
//! | one pairwise overlap ⟨φ|φ'⟩ | O(n·k² + k³), k ≤ n ⇒ O(n³) |
//! | one conditional step P(prefix·v) | B(B+1)/2 overlaps ⇒ O(B²n³) |
//! | one fresh bitstring | n steps ⇒ O(B²n⁴) |
//! | one amplitude ⟨y|ψ⟩ (for contrast) | O(B·n³) |
//!
//! So exact sampling costs 4^t where exact amplitude evaluation costs 2^t.
//! That squaring is the price of the exact route and it is not recoverable
//! by better bookkeeping — it is the Gram matrix. `Sampler` amortises it
//! across shots by caching the conditional tree, so S shots on n qubits
//! touch at most min(2^{n+1}−1, S·n + 1) nodes rather than S·n; the 20k-shot
//! tests pay for ≤ 27 nodes, not 100k.
//!
//! Measured (release, `tests/sample.rs::cost_curve`, one core), build =
//! constructing the sampler, shots = 1000 draws after it:
//!
//! | n | t | branches | overlaps | build | 1000 shots |
//! |---|---|---|---|---|---|
//! | 4 | 4 | 16 | 1_360 | 0.13 ms | 3.4 ms |
//! | 4 | 6 | 64 | 27_040 | 6.5 ms | 31 ms |
//! | 4 | 8 | 256 | 1_019_776 | 128 ms | 1.1 s |
//! | 6 | 8 | 256 | 1_812_352 | 98 ms | 2.4 s |
//! | 8 | 8 | 256 | 2_467_200 | 111 ms | 4.3 s |
//! | 10 | 6 | 64 | 546_016 | 21 ms | 1.9 s |
//!
//! **The honest working scope: t ≤ 8 is comfortable, t ≈ 10 is the edge, and
//! n is the cheap axis.** That is a much smaller T-budget than the certified
//! amplitude route's 24, and it is the exact-sampling price, not a defect.
//! Nothing here degrades gracefully into an approximation when the budget
//! runs out — it just gets slow, which is the intended failure mode.
//!
//! ## The exact overlap — the crux, and how it is done
//!
//! Branch states are kept in affine (phase-polynomial) form, credited to
//! Dehaene–De Moor 2003 and Van den Nest 2010 as in the certified engine
//! `holon-qasm::magic`: amplitude(x) = γ · i^{Σ d_a u_a} · (−1)^{Σ_{a<b} J_ab
//! u_a u_b} on the affine subspace x = R u ⊕ h, with R's columns INDEPENDENT
//! (so u ↔ x is a bijection on the support — the overlap derivation needs
//! this and `AffineState::amplitude` asserts it).
//!
//! Then, for φ = (R,h,d,J,γ) and φ' = (R',h',d',J',γ'):
//!
//! ⟨φ|φ'⟩ = Σ_x amp_φ(x)‾ · amp_φ'(x)
//!        = γ̄γ' · Σ_{(u,u') : R u ⊕ R' u' = h ⊕ h'} i^{−d·u + d'·u'} (−1)^{Q(u)+Q'(u')}
//!
//! — a quadratic-form Gauss sum over F₂ restricted to the solution set of a
//! linear system. It is evaluated in three moves, all exact:
//!
//! 1. **Eliminate the constraint.** The system R u ⊕ R' u' = h ⊕ h' is
//!    dispatched by CHANGE OF VARIABLE, not by enumeration: for a row with a
//!    leading 1 in column a, `fold(a,b)` (u_a ← u_a ⊕ u_b) clears every other
//!    entry in that row, leaving u_a = rhs, and `pin_remove` substitutes it.
//!    `fold`/`pin_remove` transform R and the phase polynomial in lockstep,
//!    so the sum is preserved exactly. An inconsistent row ⇒ the affine
//!    subspaces are disjoint ⇒ the overlap is exactly zero.
//! 2. **Gauss-sum the free variables**, one at a time, by `gauss_sum_out`:
//!    Σ_{u_a} i^{δ u_a}(−1)^{u_a Λ} with Λ = Σ_{b ∈ L} u_b, L the J-neighbours
//!    of a. δ even ⇒ 2·[Λ ≡ δ/2], a CONSTRAINT, imposed by the same
//!    fold/pin machinery. δ odd ⇒ (1 + i^δ(−1)^Λ) = (1+i^δ)·i^{(δ+2)(⊕_L u)},
//!    and the XOR expansion ⊕_L u = Σu_b − 2Σ_{b<b'}u_b u_b' (+4·… ≡ 0 mod 4)
//!    is EXACT as per-variable d-bumps plus pairwise J-flips.
//! 3. **Return γ.** With every variable summed out, γ IS the overlap.
//!
//! Cost O(k³) with k = k_φ + k_φ' ≤ 2n. No 2^n anywhere. The brute-force
//! reference `overlap_bruteforce` (Σ over all 2^n basis states) exists only
//! to referee it, and the tests hold them to EXACT ring equality.
//!
//! The `wrong_gauss` gauge plants the classic error in move 2 (dropping the
//! `1 +` from (1+i^δ), keeping only the leading phase); `tests/sample.rs`
//! requires it to fire against brute force. A silent phase error in a Gauss
//! sum is the single most likely way this file could be wrong while looking
//! right, so it is gauged rather than trusted.

use crate::ledger::Cyc;
use crate::merge::{self, MergeLedger};
use crate::BranchSource;
use std::cmp::Ordering;
use std::collections::BTreeMap;

// ------------------------------------------------------------------ scalars
//
// `Cyc` lives in `ledger` and carries mul/add/to_complex; the ring helpers the
// engine shares (`cyc_i_pow`, `cyc_conj`, `cyc_is_zero`, `cyc_neg`, `cyc_sub`,
// `cyc_eq`) are re-exported from `crate::affine` just below. What is left here
// is what only the SAMPLER needs: the exact real comparison its draw decides
// on, and the two exact scalings that feed it.

pub use crate::affine::cyc_eq;

/// Multiply by an ordinary integer (exact, denominator untouched). PANICS on
/// i128 overflow rather than wrapping — a wrapped coefficient would be a
/// wrong answer that looks like a right one, and release builds do not trap.
pub fn cyc_scale_int(a: Cyc, k: i128) -> Cyc {
    cyc_scale_int_checked(a, k).expect("cyc_scale_int: i128 overflow")
}

/// `cyc_scale_int` for callers that have a fallback.
pub fn cyc_scale_int_checked(a: Cyc, k: i128) -> Option<Cyc> {
    let mut c = [0i128; 4];
    for i in 0..4 {
        c[i] = a.c[i].checked_mul(k)?;
    }
    Some(Cyc { c, m: a.m })
}

/// Multiply by 2^(b/2) exactly, by moving the denominator rather than the
/// coefficients — so the numbers do not grow at all. (The scale is 2^{−m/2}.)
pub fn cyc_shift_half_powers(a: Cyc, b: i32) -> Cyc {
    Cyc { c: a.c, m: a.m - b }
}

/// A REAL ring element is exactly (a0 + a1·√2)·2^{−m/2}: the imaginary part
/// c1/√2 + c2 + c3/√2 vanishes over ℤ only when c2 = 0 and c1 = −c3, and the
/// representation in the basis {1,ω,ω²,ω³} is unique at fixed m. Returns None
/// for a genuinely complex element — a probability that lands here is a bug,
/// not a rounding artefact, which is why the caller unwraps loudly.
fn real_parts(a: Cyc) -> Option<(i128, i128, i32)> {
    if a.c[2] != 0 || a.c[1] != -a.c[3] {
        return None;
    }
    Some((a.c[0], a.c[1], a.m))
}

/// Raise the exponent by one, exactly: (a0+a1√2)·2^{−m/2} = (2a1+a0√2)·2^{−(m+1)/2}.
/// None on overflow — the caller falls back rather than wrapping.
fn raise_m(p: (i128, i128, i32)) -> Option<(i128, i128, i32)> {
    Some((p.1.checked_mul(2)?, p.0, p.2 + 1))
}

/// The f64 fallback, used only when an i128 step would overflow. Every call
/// is counted by `Sampler::approx_compares`, so a run that reports zero never
/// touched this path.
fn approx_cmp(a: Cyc, b: Cyc) -> Ordering {
    let (ar, _) = a.to_complex();
    let (br, _) = b.to_complex();
    ar.partial_cmp(&br).unwrap_or(Ordering::Equal)
}

/// Exact comparison of two REAL ring elements. `overflowed` is set (rather
/// than the result silently degraded) if the i128 square test cannot be done
/// in integers; the sampler counts those, and a zero count is the receipt
/// that no decision touched floating point.
pub fn cyc_real_cmp(a: Cyc, b: Cyc, overflowed: &mut bool) -> Ordering {
    let mut pa = real_parts(a).expect("cyc_real_cmp: left operand is not real");
    let mut pb = real_parts(b).expect("cyc_real_cmp: right operand is not real");
    while pa.2 < pb.2 {
        match raise_m(pa) {
            Some(next) => pa = next,
            None => {
                *overflowed = true;
                return approx_cmp(a, b);
            }
        }
    }
    while pb.2 < pa.2 {
        match raise_m(pb) {
            Some(next) => pb = next,
            None => {
                *overflowed = true;
                return approx_cmp(a, b);
            }
        }
    }
    let d0 = pa.0 - pb.0;
    let d1 = pa.1 - pb.1;
    // sign(d0 + d1·√2)
    if d1 == 0 {
        return d0.cmp(&0);
    }
    if d0 == 0 {
        return d1.cmp(&0);
    }
    if d0 > 0 && d1 > 0 {
        return Ordering::Greater;
    }
    if d0 < 0 && d1 < 0 {
        return Ordering::Less;
    }
    // Opposite signs: compare d0² against 2·d1².
    match (d0.checked_mul(d0), d1.checked_mul(d1).and_then(|s| s.checked_mul(2))) {
        (Some(sq0), Some(sq1)) => {
            let bigger_is_d0 = sq0 > sq1;
            if sq0 == sq1 {
                // |d0| = |d1|√2 is impossible over ℤ unless both are 0.
                unreachable!("d0² = 2d1² has no nonzero integer solution");
            }
            if d0 > 0 {
                if bigger_is_d0 { Ordering::Greater } else { Ordering::Less }
            } else if bigger_is_d0 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        _ => {
            *overflowed = true;
            approx_cmp(a, b)
        }
    }
}

/// |z|² of a ring element as an exact REAL ring element.
pub fn cyc_abs_sq(a: Cyc) -> Cyc {
    cyc_conj(a).mul(a)
}

// ------------------------------------------------------------------- engine
//
// The affine stabilizer engine — the state `(R, h, d, J, γ)`, the Clifford
// updates, the projector, and the exact pairwise overlap — lives in
// `crate::affine`, the ONE port of `holon-qasm::magic::Affine` in this crate.
// `project` and `overlap` were this lane's contribution to that union; the
// derivation in the module header above is their warrant.
//
// Re-exported under the names this module and its referees were written
// against: `AffineState` is `affine::Affine`, and `Clif` is the Clifford-only
// alphabet (no `T`, so `apply` cannot be handed one).

pub use crate::affine::{
    cyc_conj, cyc_is_zero, cyc_neg, cyc_sub, i_pow as cyc_i_pow, overlap, overlap_bruteforce,
    overlap_gauged, Affine as AffineState, Clif, GaussStats, Rng,
};

// -------------------------------------------------------------- magic state

/// |ψ⟩ = Σ_b c_b |φ_b⟩ — the magic-tier state as an explicit branch list.
///
/// Branches are NOT assumed orthogonal (they are not), which is precisely why
/// the norm is a Gram sum rather than Σ|c_b|².
#[derive(Clone, Debug)]
pub struct MagicState {
    n: usize,
    branches: Vec<(Cyc, AffineState)>,
}

impl MagicState {
    pub fn new(n: usize) -> Self {
        MagicState { n, branches: vec![(Cyc::ONE, AffineState::new(n))] }
    }

    pub fn n_qubits(&self) -> usize {
        self.n
    }

    pub fn branches(&self) -> &[(Cyc, AffineState)] {
        &self.branches
    }

    pub fn apply(&mut self, g: Clif) {
        for (_, st) in &mut self.branches {
            st.apply(g);
        }
    }

    /// T = ((1+ω)/2)·I + ((1−ω)/2)·Z: the branch count doubles, exactly.
    pub fn t(&mut self, q: usize) {
        self.branch(q, Cyc { c: [1, 1, 0, 0], m: 2 }, Cyc { c: [1, -1, 0, 0], m: 2 });
    }

    /// T† with ω ↦ ω⁻¹ = −ω³.
    pub fn tdg(&mut self, q: usize) {
        self.branch(q, Cyc { c: [1, 0, 0, -1], m: 2 }, Cyc { c: [1, 0, 0, 1], m: 2 });
    }

    fn branch(&mut self, q: usize, ci: Cyc, cz: Cyc) {
        let mut out = Vec::with_capacity(self.branches.len() * 2);
        for (c, st) in self.branches.drain(..) {
            let mut zst = st.clone();
            zst.z(q);
            out.push((c.mul(ci), st));
            out.push((c.mul(cz), zst));
        }
        self.branches = out;
    }

    /// Drop annihilated branches. Exact — a zero branch contributes nothing
    /// to any amplitude or overlap.
    pub fn prune(&mut self) {
        self.branches.retain(|(c, st)| !st.is_zero() && !cyc_is_zero(*c));
    }

    /// ⟨y|ψ⟩, exact. O(B·n³).
    pub fn amplitude(&self, y: &[bool]) -> Cyc {
        merge::fold(self.branches.iter().map(|(c, st)| c.mul(st.amplitude(y))))
    }

    /// Π_{x_q = v}|ψ⟩ — the un-normalised projected state.
    pub fn projected(&self, q: usize, v: bool) -> MagicState {
        let mut out = MagicState { n: self.n, branches: Vec::with_capacity(self.branches.len()) };
        for (c, st) in &self.branches {
            let p = st.projected(q, v);
            if !p.is_zero() {
                out.branches.push((*c, p));
            }
        }
        out
    }

    /// ⟨ψ|ψ⟩ = Σ_{b,b'} c̄_b c_{b'} ⟨φ_b|φ_{b'}⟩ — exact, and REAL by
    /// construction here: the off-diagonal is accumulated as t + t̄, so the
    /// result cannot pick up an imaginary part from arithmetic. (That the
    /// Gram matrix really is Hermitian is a separate claim, tested
    /// separately rather than assumed by this shortcut.)
    ///
    /// Cost: B(B+1)/2 overlaps. This is THE quadratic-in-branches term.
    pub fn norm_sq(&self) -> Cyc {
        let b = self.branches.len();
        merge::fold((0..b).flat_map(move |i| {
            let (ci, si) = &self.branches[i];
            std::iter::once(cyc_abs_sq(*ci).mul(overlap(si, si))).chain(
                (i + 1..b).flat_map(move |jx| {
                    let (cj, sj) = &self.branches[jx];
                    let t = cyc_conj(*ci).mul(*cj).mul(overlap(si, sj));
                    [t, cyc_conj(t)]
                }),
            )
        }))
    }

    /// Rebuild from an explicit branch list — the constructor the shard test
    /// needs to re-order a fold and check it lands in the same place.
    pub fn from_branches(n: usize, branches: Vec<(Cyc, AffineState)>) -> Self {
        MagicState { n, branches }
    }

    /// The number of pairwise overlaps one `norm_sq` costs.
    pub fn norm_sq_cost(&self) -> u64 {
        let b = self.branches.len() as u64;
        b * (b + 1) / 2
    }
}

impl BranchSource for MagicState {
    fn n_branches(&self) -> u64 {
        self.branches.len() as u64
    }
    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        let (c, st) = &self.branches[branch as usize];
        c.mul(st.amplitude(y))
    }
    fn n_qubits(&self) -> usize {
        self.n
    }
}

const DYADIC_BITS: u32 = 32;

// ---------------------------------------------------------------- sampler

struct Node {
    state: MagicState,
    prob: Cyc,
}

/// Exact qubit-by-qubit conditional sampler.
///
/// Walks x₁, x₂, … drawing each bit from the exact conditional
/// P(x_q = v | prefix) = ⟨ψ|Π_{prefix·v}|ψ⟩ / ⟨ψ|Π_prefix|ψ⟩. Conditional
/// probabilities are cached per prefix, so shots that share a prefix share
/// its Gram work; the cache holds at most min(2^{n+1}−1, S·n+1) nodes for S
/// shots, and each node holds a projected branch list.
pub struct Sampler {
    n: usize,
    cache: BTreeMap<Vec<bool>, Node>,
    overlaps: u64,
    approx_compares: u64,
}

impl Sampler {
    pub fn new(state: MagicState) -> Self {
        let n = state.n_qubits();
        let mut cache = BTreeMap::new();
        let overlaps = state.norm_sq_cost();
        let prob = state.norm_sq();
        cache.insert(Vec::new(), Node { state, prob });
        Sampler { n, cache, overlaps, approx_compares: 0 }
    }

    /// Total ⟨ψ|ψ⟩, exact. 1 for a state built by unitaries from |0…0⟩ —
    /// worth checking, since it is an arithmetic invariant, not a tolerance.
    pub fn total_weight(&self) -> Cyc {
        self.cache[&Vec::new()].prob
    }

    /// Pairwise stabilizer overlaps computed so far — the honest cost meter.
    pub fn overlaps(&self) -> u64 {
        self.overlaps
    }

    /// Comparisons that had to fall back to f64 because an i128 product
    /// overflowed. Zero means every sampling decision was exact integer
    /// arithmetic.
    pub fn approx_compares(&self) -> u64 {
        self.approx_compares
    }

    pub fn cache_nodes(&self) -> usize {
        self.cache.len()
    }

    /// Ensure the node for `prefix` exists; returns its exact weight
    /// ⟨ψ|Π_prefix|ψ⟩ (NOT divided by the total).
    fn ensure(&mut self, prefix: &[bool]) -> Cyc {
        if let Some(nd) = self.cache.get(prefix) {
            return nd.prob;
        }
        let q = prefix.len() - 1;
        let head = &prefix[..q];
        let v = prefix[q];
        self.ensure(head);
        let child = self.cache[head].state.projected(q, v);
        self.overlaps += child.norm_sq_cost();
        let prob = child.norm_sq();
        self.cache.insert(prefix.to_vec(), Node { state: child, prob });
        prob
    }

    /// Exact ⟨ψ|Π_prefix|ψ⟩ for any prefix (the un-normalised weight; divide
    /// by `total_weight` for a probability).
    pub fn prefix_weight(&mut self, prefix: &[bool]) -> Cyc {
        assert!(prefix.len() <= self.n);
        self.ensure(prefix)
    }

    /// Exact |⟨x|ψ⟩|² for a full bitstring, via the conditional chain.
    pub fn exact_prob(&mut self, x: &[bool]) -> Cyc {
        assert_eq!(x.len(), self.n);
        self.ensure(x)
    }

    /// One shot. Deterministic given the rng state.
    pub fn sample_one(&mut self, rng: &mut Rng) -> Vec<bool> {
        let mut prefix: Vec<bool> = Vec::with_capacity(self.n);
        let mut here = self.ensure(&prefix);
        for q in 0..self.n {
            prefix.push(false);
            let p0 = self.ensure(&prefix);
            prefix[q] = true;
            let p1 = self.ensure(&prefix);
            // EXACT ring identity, not a tolerance: the projector resolution
            // Π_prefix = Π_{prefix·0} + Π_{prefix·1}.
            assert!(
                cyc_eq(p0.merge(p1), here),
                "conditional split is not exact at qubit {q}"
            );
            // Draw: bit = 1 iff r ≥ p0/here, i.e. R·here ≥ 2^32·p0.
            let r = rng.next_dyadic32() as i128;
            // 2^DYADIC_BITS is applied by MOVING THE DENOMINATOR (the scale is
            // 2^{−m/2}), so that side's coefficients do not grow at all; only
            // the R multiply can overflow, and it falls back rather than wraps.
            let rhs = cyc_shift_half_powers(p0, 2 * DYADIC_BITS as i32);
            let mut over = false;
            let bit = match cyc_scale_int_checked(here, r) {
                Some(lhs) => cyc_real_cmp(lhs, rhs, &mut over) != Ordering::Less,
                None => {
                    // The R multiply would wrap. Decide in f64 on the RATIO
                    // instead, and count it — a wrapped integer would be a
                    // silently wrong draw, an f64 ratio is merely a rounded one.
                    over = true;
                    let scale = (1u64 << DYADIC_BITS) as f64;
                    (r as f64 / scale) * here.to_complex().0 >= p0.to_complex().0
                }
            };
            if over {
                self.approx_compares += 1;
            }
            prefix[q] = bit;
            here = if bit { p1 } else { p0 };
        }
        prefix
    }

    /// `shots` shots, seeded. Deterministic: same seed, same multiset.
    pub fn sample(&mut self, shots: usize, seed: u64) -> Vec<Vec<bool>> {
        let mut rng = Rng::new(seed);
        (0..shots).map(|_| self.sample_one(&mut rng)).collect()
    }

    /// `shots` shots as counts, keyed MSB-first over qubits (qubit n−1 is the
    /// leftmost character) — the convention the QASM tiers report in.
    pub fn sample_counts(&mut self, shots: usize, seed: u64) -> BTreeMap<String, u64> {
        let mut out = BTreeMap::new();
        for x in self.sample(shots, seed) {
            *out.entry(bitstring_key(&x)).or_insert(0) += 1;
        }
        out
    }
}

/// MSB-first key over qubits: qubit n−1 leftmost.
pub fn bitstring_key(x: &[bool]) -> String {
    (0..x.len()).rev().map(|i| if x[i] { '1' } else { '0' }).collect()
}
