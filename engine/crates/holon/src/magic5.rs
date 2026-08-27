//! MAGIC5-FROM-CAT — the PARTIAL stabilizer decomposition, which reaches
//! α = log₂(3)/4 ≈ 0.3963 **concretely at finite T-count** instead of only
//! asymptotically.
//!
//! # Credit, generously and first
//!
//! The rule is **Kissinger, van de Wetering and Vilmart**, *Classical
//! simulation with partial and graphical stabiliser decompositions*, TQC 2022
//! (arXiv:2202.09202), §4.2 — their `Magic5FromCat`. It stands on
//! **Qassim, Pashayan and Gosset** (arXiv:2106.07740), whose cat-state
//! decompositions it is built from, and on **Bravyi and Gosset** (2016), whose
//! magic-state gadget and block-decomposition framing this crate's
//! [`crate::magic::BgSource`] already uses. The reference implementation we
//! read is **quizx** (`src/decompose.rs`: `replace_magic5_0/1/2`,
//! `BssWithCatsDriver`), Apache-2.0, by the same authors — compatible with this
//! crate's AGPL-3.0, and their `Scalar4` is literally this crate's
//! [`crate::ledger::Cyc`], so the port loses no exactness.
//!
//! Ours is the translation into the affine phase-polynomial engine, the
//! deterministic mixed-radix branch index the mesh shards on, and the exactness
//! gates below. The mathematics is theirs.
//!
//! # The rule, in one line
//!
//! Whenever five magic states remain, **three terms** suffice, and each term
//! keeps **one** magic state. Five in, one back: four consumed per three
//! terms, so `N(t) = 3·N(t−4)` and `α = log₂(3)/4`.
//!
//! This is a RECURSIVE PARTIAL rule, not a fixed block table. A branch is a
//! child holon: the decomposition interface is recursive because the object is.
//!
//! # The identity, DERIVED HERE and never trusted to memory
//!
//! Write `|A⟩ = (|0⟩ + ω|1⟩)/√2` (ω = e^{iπ/4}), so
//! `|A^{⊗5}⟩ = 2^{−5/2} Σ_x ω^{|x|}|x⟩`. Define three five-qubit states, each
//! a CLIFFORD frame applied to `|A⟩` on wire 0 and `|0⟩` on wires 1–4:
//!
//! ```text
//! Φ₀ = [Z(0); CX(0→1..4)]                                   · (|A⟩ ⊗ |0000⟩)
//! Φ₁ = [S†(0); H(0); CX(0→1..4); H(0..4)]                   · (|A⟩ ⊗ |0000⟩)
//! Φ₂ = [S†(0); Z(0); H(1..4); CX(1..4→0); CZ(all 10 pairs)] · (|A⟩ ⊗ |0000⟩)
//! ```
//!
//! Then, exactly in `Z[ω]·2^{−m/2}`:
//!
//! ```text
//! |A^{⊗5}⟩ = ½·Φ₀ + ((−1+i)/2)·Φ₁ + ((−1−i)/2)·Φ₂
//! ```
//!
//! Every Φ is a Clifford image of `|A⟩ ⊗ |0000⟩`, so the retained magic state
//! is a genuine `|A⟩` on a wire the plan names — which is what makes the rule
//! recursive rather than a one-shot block. The three frames are the quizx
//! terms read back into linear algebra: Φ₀ is `replace_magic5_0` (the fused
//! spider at phase −3π/4, i.e. the GHZ with a T-phase), Φ₁ is
//! `replace_magic5_1` (the arity-6 phase gadget), Φ₂ is `replace_magic5_2`
//! (the complete graph plus the two internal spiders). The coefficients are
//! NOT quizx's `Scalar4` constants — those carry ZX normalisation conventions
//! this engine does not share — they are solved here against the six
//! Hamming-weight equations and then re-derived on every construction.
//!
//! The five-qubit identity above is over-determined: the three shapes depend on
//! `x` only through `|x|`, giving six equations in three unknowns. It closes on
//! all six. That is a derivation, and [`magic5_is_exact`] is the machine's
//! check of it, at all 32 basis states, in the exact ring, with no tolerance —
//! called by [`Magic5Source::new`] on every construction so a wrong constant
//! cannot ship. [`register_is_exact`] is the stronger one: it re-derives the
//! WHOLE recursive plan against `|A^{⊗t}⟩` at every basis state.
//!
//! # What is NOT claimed
//!
//! α is an ASYMPTOTIC exponent. What this source realises at a finite `t` is
//! `log₂(3^{⌊(t−1)/4⌋} · 2^{⌈tail/2⌉})/t`, which approaches 0.3963 FROM ABOVE
//! and is never equal to it. [`Magic5Source::realized_exponent`] reports the
//! realized number; the asymptote is [`MAGIC5_ALPHA`] and is never quoted as a
//! measurement. The tail (`t mod 4` magic states, plus the one kept) falls back
//! to this crate's own verified pair blocks, exponent 0.5 — so short circuits
//! are exactly the old path, not a regression.
//!
//! No stabilizer-rank claim about `|A^{⊗5}⟩` is made or implied: the three
//! terms are NOT stabilizer states. Each carries one magic state, which is the
//! entire point and the reason the recursion pays.

use crate::affine::{cyc_eq, omega_pow, Affine, Gate};
use crate::ledger::Cyc;
use crate::magic::{a1_terms, a2_terms, Circuit, Mutation, StabTerm};
use crate::merge::fold;
use crate::prune::push_cz;
use crate::BranchSource;

/// Magic states consumed and re-emitted per application: five in, one back.
pub const MAGIC5_WIDTH: usize = 5;
/// Terms per application. This is the `3` in `N(t) = 3·N(t−4)`.
pub const MAGIC5_RANK: usize = 3;
/// Net magic states removed per application: `MAGIC5_WIDTH − 1`.
pub const MAGIC5_CONSUMED: usize = MAGIC5_WIDTH - 1;

/// The ASYMPTOTIC exponent `log₂(3)/4`. Never quote this as a measurement of a
/// finite run — [`Magic5Source::realized_exponent`] is the measured one, and it
/// is strictly larger at every finite `t`.
pub const MAGIC5_ALPHA: f64 = 0.396_240_625_181_25;

/// The three exact coefficients: `½`, `(−1+i)/2`, `(−1−i)/2` (`i = ω²`).
///
/// Solved from the six Hamming-weight equations, not copied. They are checked
/// against the engine on every [`Magic5Source`] construction.
pub fn magic5_coeffs() -> [Cyc; MAGIC5_RANK] {
    [
        Cyc { c: [1, 0, 0, 0], m: 2 },
        Cyc { c: [-1, 0, 1, 0], m: 2 },
        Cyc { c: [-1, 0, -1, 0], m: 2 },
    ]
}

/// The Clifford frame of term `j` on the five magic wires `q`.
///
/// `q[0]` is the wire that KEEPS its magic state; `q[1..5]` enter as `|0⟩` and
/// leave carrying no magic. Gates are returned in APPLICATION order.
pub fn magic5_frame(j: usize, q: &[usize]) -> Vec<Gate> {
    assert_eq!(q.len(), MAGIC5_WIDTH, "magic5 frame width");
    let mut g: Vec<Gate> = Vec::new();
    match j {
        // Φ₀ — the fused spider: Z on the keeper, then fan it out.
        0 => {
            g.push(Gate::Z(q[0]));
            for &t in &q[1..] {
                g.push(Gate::Cx(q[0], t));
            }
        }
        // Φ₁ — the arity-6 phase gadget: the keeper is rotated into the
        // Y–Z edge state H S†|A⟩, fanned out, and read in the X basis.
        1 => {
            g.push(Gate::Sdg(q[0]));
            g.push(Gate::H(q[0]));
            for &t in &q[1..] {
                g.push(Gate::Cx(q[0], t));
            }
            for &t in q {
                g.push(Gate::H(t));
            }
        }
        // Φ₂ — the complete graph: the keeper becomes Z S†|A⟩ and is written
        // onto the PARITY of the five wires, then every pair is CZ'd.
        2 => {
            g.push(Gate::Sdg(q[0]));
            g.push(Gate::Z(q[0]));
            for &t in &q[1..] {
                g.push(Gate::H(t));
            }
            for &c in &q[1..] {
                g.push(Gate::Cx(c, q[0]));
            }
            for a in 0..MAGIC5_WIDTH {
                for b in a + 1..MAGIC5_WIDTH {
                    push_cz(&mut g, q[a], q[b]);
                }
            }
        }
        _ => panic!("magic5 term index {j} out of range (rank {MAGIC5_RANK})"),
    }
    g
}

/// THE GATE on the five-qubit identity: `Σ_j c_j Φ_j(x)` must equal
/// `ω^{|x|}·2^{−5/2}` at EVERY one of the 32 basis states, in the exact ring.
/// No tolerance, no sampling.
///
/// `|A⟩` on the keeper wire is installed as its two affine pieces (this
/// crate's own verified [`a1_terms`]), so the check exercises the SAME loader
/// the source uses. It runs on a CLEAN engine deliberately: it is a check of
/// the table, and must not be turned into a check of a planted mutation — a
/// mutated engine has to be caught by conformance against the naive sum, which
/// is where the gauge lives.
pub fn magic5_is_exact() -> bool {
    let coeffs = magic5_coeffs();
    let q: Vec<usize> = (0..MAGIC5_WIDTH).collect();
    let frames: Vec<Vec<Gate>> = (0..MAGIC5_RANK).map(|j| magic5_frame(j, &q)).collect();
    let pieces = a1_terms();
    for x in 0u32..(1u32 << MAGIC5_WIDTH) {
        let y: Vec<bool> = (0..MAGIC5_WIDTH).map(|i| (x >> i) & 1 == 1).collect();
        let mut parts: Vec<Cyc> = Vec::with_capacity(MAGIC5_RANK * pieces.len());
        for (j, frame) in frames.iter().enumerate() {
            for piece in &pieces {
                let mut st = Affine::new(MAGIC5_WIDTH);
                st.attach(&[0], &piece.cols, piece.h, &piece.d, &piece.j);
                for g in frame {
                    st.apply(*g);
                }
                parts.push(coeffs[j].mul(piece.coeff).mul(st.amplitude(&y)));
            }
        }
        // Accumulation is THE ONE MERGE LAW, so the audit is the same number
        // in any order — which is also the mesh's warrant for sharding it.
        let acc = fold(parts);
        let w = omega_pow((x.count_ones() % 8) as u8);
        if !cyc_eq(acc, Cyc { c: w.c, m: w.m + MAGIC5_WIDTH as i32 }) {
            return false;
        }
    }
    true
}

// --------------------------------------------------------------------- plan

/// How `t` magic states are cut into rounds of the partial rule plus a tail.
///
/// The rule fires while five or more magic states are live; each round names
/// its five wires and KEEPS the first, which re-enters the pool. Below five
/// the plan falls back to this crate's verified pair/singleton blocks — so a
/// short circuit is byte-for-byte the old path.
///
/// The schedule is BRANCH-INDEPENDENT: all three terms keep the same wire, so
/// which wires a later round owns does not depend on earlier digits. That is
/// what keeps the branch space a deterministic mixed-radix index the mesh can
/// shard by index alone.
#[derive(Clone, Debug)]
pub struct Magic5Plan {
    /// Index of the first magic wire.
    pub base: usize,
    /// Number of magic states.
    pub t: usize,
    /// Per round: the five magic wires, `[0]` being the keeper.
    pub rounds: Vec<[usize; MAGIC5_WIDTH]>,
    /// The ≤ 4 magic wires left when no round can fire, with their verified
    /// product tables ([`a2_terms`] / [`a1_terms`]).
    pub tail: Vec<(Vec<usize>, Vec<StabTerm>)>,
}

impl Magic5Plan {
    pub fn new(base: usize, t: usize) -> Self {
        let mut live: Vec<usize> = (base..base + t).collect();
        let mut rounds: Vec<[usize; MAGIC5_WIDTH]> = Vec::new();
        while live.len() >= MAGIC5_WIDTH {
            let mut grp = [0usize; MAGIC5_WIDTH];
            grp.copy_from_slice(&live[..MAGIC5_WIDTH]);
            rounds.push(grp);
            // Four consumed, `live[0]` kept — the whole rule, as bookkeeping.
            live.drain(1..MAGIC5_WIDTH);
        }
        let mut tail: Vec<(Vec<usize>, Vec<StabTerm>)> = Vec::new();
        let mut i = 0usize;
        while live.len() - i >= 2 {
            tail.push((vec![live[i], live[i + 1]], a2_terms()));
            i += 2;
        }
        while i < live.len() {
            tail.push((vec![live[i]], a1_terms()));
            i += 1;
        }
        Magic5Plan { base, t, rounds, tail }
    }

    /// The mixed-radix shape of the branch index: one radix-`3` digit per
    /// round, then the tail blocks' ranks. Rounds are the LOW digits.
    pub fn radices(&self) -> Vec<u64> {
        let mut r: Vec<u64> = vec![MAGIC5_RANK as u64; self.rounds.len()];
        r.extend(self.tail.iter().map(|(_, ts)| ts.len() as u64));
        r
    }

    /// `Π radices`, refusing rather than wrapping — a branch count that
    /// silently overflowed would index a different space than it names.
    pub fn n_branches(&self) -> u64 {
        self.radices().iter().try_fold(1u64, |a, &b| a.checked_mul(b)).expect(
            "magic5: branch count exceeds u64 — refusing rather than wrapping the index space",
        )
    }

    /// Write branch `b`'s magic-register state into `st` — which must be a
    /// fresh `|0…0⟩` affine state at least `base + t` wide — and return the
    /// branch's exact coefficient.
    ///
    /// Ordering is the recursion's, and it is not a detail: round `r`'s frame
    /// WRAPS every later round (they share the keeper wire), so the frames
    /// compose outward and the LAST round is applied to the base state FIRST.
    pub fn load(&self, st: &mut Affine, b: u64) -> Cyc {
        let coeffs = magic5_coeffs();
        let mut rest = b;
        let mut digits: Vec<usize> = Vec::with_capacity(self.rounds.len());
        let mut coeff = Cyc::ONE;
        for _ in &self.rounds {
            let d = (rest % MAGIC5_RANK as u64) as usize;
            rest /= MAGIC5_RANK as u64;
            digits.push(d);
            coeff = coeff.mul(coeffs[d]);
        }
        for (qs, terms) in &self.tail {
            let d = (rest % terms.len() as u64) as usize;
            rest /= terms.len() as u64;
            let term = &terms[d];
            assert_eq!(qs.len(), term.nq, "magic5 tail: qubit count vs term width");
            st.attach(qs, &term.cols, term.h, &term.d, &term.j);
            coeff = coeff.mul(term.coeff);
        }
        for (r, grp) in self.rounds.iter().enumerate().rev() {
            for g in magic5_frame(digits[r], grp) {
                st.apply(g);
            }
        }
        coeff
    }
}

/// THE STRONG GATE: the whole RECURSIVE plan, re-derived against
/// `|A^{⊗t}⟩ = 2^{−t/2} Σ_x ω^{|x|}|x⟩` at every one of the `2^t` basis
/// states, in the exact ring. This is what says the recursion composes — the
/// five-qubit identity only says one round does.
pub fn register_is_exact(t: usize) -> bool {
    assert!(t <= 16, "register_is_exact is a 2^t audit; {t} is not an audit");
    let plan = Magic5Plan::new(0, t);
    let nb = plan.n_branches();
    for x in 0u32..(1u32 << t) {
        let y: Vec<bool> = (0..t).map(|i| (x >> i) & 1 == 1).collect();
        let parts: Vec<Cyc> = (0..nb)
            .map(|b| {
                let mut st = Affine::new(t);
                let c = plan.load(&mut st, b);
                c.mul(st.amplitude(&y))
            })
            .collect();
        let acc = fold(parts);
        let w = omega_pow((x.count_ones() % 8) as u8);
        if !cyc_eq(acc, Cyc { c: w.c, m: w.m + t as i32 }) {
            return false;
        }
    }
    true
}

// ------------------------------------------------------------------- source

/// Cache ceiling: above this many branches the source recomputes per query
/// instead of holding every end-state. The honest failure is slow, never wrong.
const CACHE_MAX: u64 = 1 << 18;

/// The Magic5FromCat branch source.
///
/// The gadget is the one [`crate::magic::BgSource`] already uses and credits
/// (magic-state injection, Bravyi–Gosset): with ancilla `a` in `|A⟩`,
/// `CX(q→a)` then POST-SELECTING `a = 0` leaves `T|ψ⟩/√2` on `q`, so
/// `⟨y|U|0^n⟩ = 2^{t/2}·⟨y,0^t| C |0^n, A^t⟩`, exact in the ring (`m = −t`).
/// `T†` is the same gadget followed by a Clifford `S†`.
///
/// What is NEW is the magic register: instead of a product of fixed blocks,
/// `|A^{⊗t}⟩` is decomposed by the RECURSIVE partial rule, and the Clifford
/// frames the rule emits are prepended to the circuit.
pub struct Magic5Source {
    n: usize,
    t: usize,
    n_ext: usize,
    ext_gates: Vec<Gate>,
    plan: Magic5Plan,
    mutation: Mutation,
    cache: Vec<(Cyc, Affine)>,
}

impl Magic5Source {
    pub fn new(c: &Circuit) -> Self {
        Self::with_mutation(c, Mutation::default())
    }

    pub fn with_mutation(c: &Circuit, mutation: Mutation) -> Self {
        let t = c.t_count();
        let n = c.n_qubits;
        // The identity is re-derived at every construction. It is cheap (32
        // exact amplitude sums on five qubits) and it is the whole warrant.
        assert!(magic5_is_exact(), "the Magic5FromCat identity is NOT exact");
        assert!(
            crate::magic::decomposition_is_exact(&a2_terms(), 2),
            "the |A^2⟩ tail split is NOT exact"
        );
        assert!(
            crate::magic::decomposition_is_exact(&a1_terms(), 1),
            "the |A⟩ tail split is NOT exact"
        );

        let mut ext_gates = Vec::with_capacity(c.gates.len() + t);
        let mut ti = 0usize;
        for g in &c.gates {
            match *g {
                Gate::T(q) => {
                    ext_gates.push(Gate::Cx(q, n + ti));
                    ti += 1;
                }
                Gate::Tdg(q) => {
                    ext_gates.push(Gate::Cx(q, n + ti));
                    ti += 1;
                    ext_gates.push(Gate::Sdg(q));
                }
                g => ext_gates.push(g),
            }
        }

        let mut s = Magic5Source {
            n,
            t,
            n_ext: n + t,
            ext_gates,
            plan: Magic5Plan::new(n, t),
            mutation,
            cache: Vec::new(),
        };
        let nb = s.n_branches();
        if nb <= CACHE_MAX {
            s.cache = (0..nb).map(|b| s.run_branch(b)).collect();
        }
        s
    }

    /// The exponent this source ACTUALLY realises at this `t`:
    /// `log₂(branches)/t`. It approaches [`MAGIC5_ALPHA`] from above and never
    /// reaches it; report this number, not the asymptote.
    pub fn realized_exponent(&self) -> f64 {
        if self.t == 0 {
            return 0.0;
        }
        (self.n_branches() as f64).log2() / self.t as f64
    }

    /// Rounds of the partial rule this circuit's T-count buys.
    pub fn rounds(&self) -> usize {
        self.plan.rounds.len()
    }

    pub fn plan(&self) -> &Magic5Plan {
        &self.plan
    }

    fn run_branch(&self, branch: u64) -> (Cyc, Affine) {
        let mut st = Affine::with_mutations(self.n_ext, self.mutation.into());
        // 2^{t/2} from the post-selected gadgets, exact.
        let mut coeff = Cyc { c: [1, 0, 0, 0], m: -(self.t as i32) };
        coeff = coeff.mul(self.plan.load(&mut st, branch));
        for g in &self.ext_gates {
            st.apply(*g);
        }
        (coeff, st)
    }

    fn amp_ext(&self, coeff: Cyc, st: &Affine, y: &[bool]) -> Cyc {
        let mut y_ext = vec![false; self.n_ext];
        y_ext[..self.n].copy_from_slice(y);
        let a = st.amplitude(&y_ext);
        if crate::affine::cyc_is_zero(a) {
            Cyc::ZERO
        } else {
            coeff.mul(a)
        }
    }
}

impl BranchSource for Magic5Source {
    fn n_branches(&self) -> u64 {
        self.plan.n_branches()
    }

    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        assert!(branch < self.n_branches(), "branch index out of range");
        assert_eq!(y.len(), self.n, "amplitude query width vs circuit qubits");
        if let Some((coeff, st)) = self.cache.get(branch as usize) {
            return self.amp_ext(*coeff, st, y);
        }
        let (coeff, st) = self.run_branch(branch);
        self.amp_ext(coeff, &st, y)
    }

    fn n_qubits(&self) -> usize {
        self.n
    }
}

/// The branch count this plan realises at `t`, written INDEPENDENTLY of
/// [`Magic5Plan`] so a test using it is a check and not an echo:
/// `⌊(t−1)/4⌋` rounds of three, then `2^{⌈tail/2⌉}` for the `t − 4·rounds`
/// magic states left.
pub fn expected_branches(t: usize) -> u64 {
    let rounds = if t == 0 { 0 } else { (t - 1) / MAGIC5_CONSUMED };
    let tail = t - MAGIC5_CONSUMED * rounds;
    (MAGIC5_RANK as u64).pow(rounds as u32) * (1u64 << tail.div_ceil(2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_identity_is_exact() {
        assert!(magic5_is_exact());
    }

    #[test]
    fn the_recursion_composes() {
        for t in 0..=9usize {
            assert!(register_is_exact(t), "register decomposition wrong at t={t}");
        }
    }

    #[test]
    fn branch_counts_follow_the_rule() {
        for t in 0..=40usize {
            let plan = Magic5Plan::new(0, t);
            assert_eq!(plan.n_branches(), expected_branches(t), "branch count at t={t}");
        }
        // The headline arithmetic, spelled out: 3^15 · 4 against 2^32.
        assert_eq!(expected_branches(64), 57_395_628);
    }
}
