//! The MAGIC tier of THE holon: Clifford+T by exact stabilizer branch sums,
//! with the Bravyi–Gosset stabilizer-rank reduction on blocks of six T gates.
//!
//! The affine stabilizer engine is ported from holon-qasm's certified magic
//! tier (QASM-2 record, five of five against qiskit) onto this crate's own
//! ledger ring: amplitude(x) = γ · i^{Σ d_a u_a} · (−1)^{Σ_{a<b} J_ab u_a u_b}
//! on the affine subspace x = R u ⊕ h, with γ exact in Z[ω]·2^{−m/2}
//! (Dehaene–De Moor 2003; Van den Nest 2010 — credited). No floating point
//! enters the exact path; `to_complex` is the only exit.
//!
//! Every subtlety in the port was bought by a campaign upstream and is marked
//! at its site: the column-dependence repair after a row clear (with its LOUD
//! rank assertion), the odd-δ Gauss-sum XOR expansion, the one-shot
//! denominator alignment in the ring's `add`.
//!
//! Two branch sources, both implementing `crate::BranchSource`:
//!
//! * [`NaiveSource`] — T = ((1+ω)/2)·I + ((1−ω)/2)·Z per gate, 2^t branches.
//!   Semantics identical to the reference; this is the conformance baseline.
//!
//! * [`BgSource`] — the T gates are gadgetised onto a magic register
//!   |A⟩^{⊗t}, A = (|0⟩+ω|1⟩)/√2 (Bravyi–Gosset 2016, credited), and the
//!   register is decomposed BLOCK BY BLOCK by [`block_plan`], so the branch
//!   count is a product of per-block ranks instead of 2^t.
//!
//! WHAT THIS LANE ACHIEVED, stated before any exponent is quoted: rank 2 per
//! TWO T gates — χ(|A^{⊗2}⟩) = 2, derived and verified here — hence 2^{⌈t/2⌉}
//! branches, exponent 0.5. The Bravyi–Gosset rank 7 per SIX (exponent 0.4679)
//! is NOT in this file. It is a numerical-search result; the search run for it
//! here did not find it, on a searcher that also failed its own planted
//! control, so this is a failure to reproduce and NOT evidence against rank 7.
//! [`A6_DATA`] is the swap point, and currently holds the trivial composition
//! of three pair blocks (rank 8 = 2³) — which is why [`block_plan`] declines
//! six-wide blocks entirely. See [`A6_PROVENANCE`].
//!
//! THE DECOMPOSITION IS NOT TRUSTED TO MEMORY. `decomposition_is_exact`
//! re-derives Σ_j c_j φ_j(x) in the exact ring and compares it to ω^{|x|}·2^{−k/2}
//! at every one of the 2^k basis states; the conformance test calls it and
//! `BgSource::new` asserts it on every construction, so a wrong table cannot
//! ship even if someone pastes one in.

use crate::ledger::Cyc;
use crate::BranchSource;

// ---------------------------------------------------------------- the engine
//
// The affine stabilizer engine — the state `(R, h, d, J, γ)`, the Clifford
// updates, the block loader `Affine::attach`, and the ring helpers — lives in
// `crate::affine`, the ONE port of `holon-qasm::magic::Affine` in this crate.
// `attach` was this lane's contribution to that union. Re-exported here under
// the names this module and its referees were written against.

pub use crate::affine::{cyc_eq, cyc_is_zero as is_zero, i_pow, omega_pow, Affine, Gate};

// ---------------------------------------------------------------- circuits

#[derive(Clone, Debug)]
pub struct Circuit {
    pub n_qubits: usize,
    pub gates: Vec<Gate>,
}

impl Circuit {
    pub fn t_count(&self) -> usize {
        self.gates.iter().filter(|g| g.is_t()).count()
    }
}

/// Planted mutations for the gauge: a conformance harness that cannot fail is
/// not a conformance harness. Both are ported from the certified reference,
/// which names them in exactly this pair — kept as its own two-field type so
/// that this tier's gauge vocabulary stays the reference's, and widened to the
/// engine's union on the way in.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Mutation {
    /// Drop the S gate's pairwise J flips.
    pub drop_s_cross: bool,
    /// Use the wrong odd-δ Gauss-sum phase (drops the 1+ structure).
    pub wrong_gauss: bool,
}

impl From<Mutation> for crate::affine::Mutations {
    fn from(m: Mutation) -> Self {
        crate::affine::Mutations {
            drop_s_cross: m.drop_s_cross,
            wrong_gauss: m.wrong_gauss,
            ..crate::affine::Mutations::default()
        }
    }
}

// ------------------------------------------------- stabilizer decompositions

/// One term of a stabilizer decomposition: an exact coefficient times an
/// unnormalised affine state on `nq` qubits (γ = 1, support 2^{cols.len()}).
#[derive(Clone, Debug)]
pub struct StabTerm {
    pub coeff: Cyc,
    pub nq: usize,
    /// Column c as a bitmask over the term's qubits (bit b ⇒ R[b][c] = 1).
    pub cols: Vec<u32>,
    pub h: u32,
    pub d: Vec<u8>,
    /// Symmetric, cols.len() × cols.len(), diagonal unused.
    pub j: Vec<Vec<bool>>,
}

impl StabTerm {
    /// Amplitude of basis state x, computed independently of `Affine` — this
    /// is the audit path, so it solves by enumeration rather than sharing the
    /// engine's elimination.
    pub fn amplitude(&self, x: u32) -> Cyc {
        let k = self.cols.len();
        for u in 0u32..(1u32 << k) {
            let mut xv = self.h;
            for (a, &col) in self.cols.iter().enumerate() {
                if (u >> a) & 1 == 1 {
                    xv ^= col;
                }
            }
            if xv != x {
                continue;
            }
            let mut ip: u8 = 0;
            let mut sign = false;
            for a in 0..k {
                if (u >> a) & 1 == 1 {
                    ip = (ip + self.d[a]) % 4;
                    for b in a + 1..k {
                        if (u >> b) & 1 == 1 && self.j[a][b] {
                            sign = !sign;
                        }
                    }
                }
            }
            let amp = i_pow(ip);
            return if sign { amp.mul(i_pow(2)) } else { amp };
        }
        Cyc::ZERO
    }
}

/// THE GATE on the whole construction: Σ_j c_j φ_j(x) must equal the magic
/// register's amplitude ω^{|x|}·2^{−nq/2} at EVERY one of the 2^{nq} basis
/// states, in the exact ring. No tolerance, no sampling.
pub fn decomposition_is_exact(terms: &[StabTerm], nq: usize) -> bool {
    for x in 0u32..(1u32 << nq) {
        // Accumulation is the ONE merge law (`merge::fold`), not a local sum:
        // the terms fold associatively and commutatively, so the audit is the
        // same number in any order. Out-of-support terms contribute ZERO,
        // which is the ledger's identity.
        let acc = crate::merge::fold(terms.iter().map(|t| t.coeff.mul(t.amplitude(x))));
        let w = omega_pow((x.count_ones() % 8) as u8);
        let want = Cyc { c: w.c, m: w.m + nq as i32 };
        if !cyc_eq(acc, want) {
            return false;
        }
    }
    true
}

/// |A⟩^{⊗2} in TWO stabilizer terms — χ(|A^{⊗2}⟩) = 2, derived here:
///
///   |A⟩⊗|A⟩ = ½·[ (|00⟩ + i|11⟩) + ω·(|01⟩ + |10⟩) ]
///
/// because ω^{|x|} restricted to even parity is i^{|x|/2} (a Clifford phase)
/// and restricted to odd parity is the constant ω. Both sectors are affine,
/// so both terms are stabilizer states. Verified by `decomposition_is_exact`.
pub fn a2_terms() -> Vec<StabTerm> {
    vec![
        // ψ_e = |00⟩ + i|11⟩, coefficient ½.
        StabTerm {
            coeff: Cyc { c: [1, 0, 0, 0], m: 2 },
            nq: 2,
            cols: vec![0b11],
            h: 0,
            d: vec![1],
            j: vec![vec![false]],
        },
        // ψ_o = |01⟩ + |10⟩, coefficient ω/2.
        StabTerm {
            coeff: Cyc { c: [0, 1, 0, 0], m: 2 },
            nq: 2,
            cols: vec![0b11],
            h: 1,
            d: vec![0],
            j: vec![vec![false]],
        },
    ]
}

/// |A⟩ = (|0⟩ + ω|1⟩)/√2 — two terms, the irreducible leftover.
pub fn a1_terms() -> Vec<StabTerm> {
    vec![
        StabTerm {
            coeff: Cyc { c: [1, 0, 0, 0], m: 1 },
            nq: 1,
            cols: vec![],
            h: 0,
            d: vec![],
            j: vec![],
        },
        StabTerm {
            coeff: Cyc { c: [0, 1, 0, 0], m: 1 },
            nq: 1,
            cols: vec![],
            h: 1,
            d: vec![],
            j: vec![],
        },
    ]
}

/// The block width the BG source groups T gates into.
pub const A6_WIDTH: usize = 6;

/// A decomposition term in const-friendly form: fixed-width arrays with an
/// explicit column count, so the table below is data and nothing else.
#[derive(Clone, Copy, Debug)]
pub struct RawTerm {
    /// coeff = (c0 + c1ω + c2ω² + c3ω³)·2^{−m/2}.
    pub coeff_c: [i128; 4],
    pub coeff_m: i32,
    pub k: usize,
    /// Column a as a bitmask over the six block qubits.
    pub cols: [u32; A6_WIDTH],
    pub h: u32,
    pub d: [u8; A6_WIDTH],
    /// Row a of J as a bitmask over columns (symmetric; diagonal unused).
    pub j: [u8; A6_WIDTH],
}

impl RawTerm {
    pub fn expand(&self, nq: usize) -> StabTerm {
        let k = self.k;
        let mut j = vec![vec![false; k]; k];
        for a in 0..k {
            for b in 0..k {
                if (self.j[a] >> b) & 1 == 1 {
                    j[a][b] = true;
                    j[b][a] = true;
                }
            }
        }
        StabTerm {
            coeff: Cyc { c: self.coeff_c, m: self.coeff_m },
            nq,
            cols: self.cols[..k].to_vec(),
            h: self.h,
            d: self.d[..k].to_vec(),
            j,
        }
    }
}

/// The rank ACHIEVED on a block of six T gates — the measured number, not the
/// hoped-for one. See [`A6_PROVENANCE`].
pub const A6_RANK: usize = A6_DATA.len();

/// The decomposition of |A⟩^{⊗6} = 2^{−3} Σ_x ω^{|x|}|x⟩ into [`A6_RANK`]
/// stabilizer terms. Verified exactly by [`decomposition_is_exact`] — which
/// `BgSource::new` calls on every construction, so a wrong table cannot ship.
pub fn a6_terms() -> Vec<StabTerm> {
    A6_DATA.iter().map(|t| t.expand(A6_WIDTH)).collect()
}

/// How the table below was obtained, stated so the exponent is never quoted
/// without it. Replaced only when the table is.
pub const A6_PROVENANCE: &str = A6_PROVENANCE_TEXT;

// --- the table -------------------------------------------------------------
//
// PAIRWISE PRODUCT DECOMPOSITION, rank 8 on six T gates (exponent 1/2).
//
// The identity, derived here and verified exactly below:
//     |A⟩⊗|A⟩ = ½·[ (|00⟩ + i|11⟩) + ω·(|01⟩ + |10⟩) ]
// — the even-parity sector of ω^{|x|} is i^{|x|/2}, a Clifford phase, and the
// odd-parity sector is the constant ω. Both sectors are stabilizer states, so
// χ(|A^{⊗2}⟩) = 2 and three independent pairs give 8 terms on six qubits.
//
// This is NOT the Bravyi–Gosset rank 7. See `A6_PROVENANCE_TEXT`.
const A6_DATA: &[RawTerm] = &[
    raw(&[1, 0, 0, 0], 0, [1, 1, 1]),   // eee : ω⁰
    raw(&[0, 1, 0, 0], 1, [0, 1, 1]),   // oee : ω¹
    raw(&[0, 1, 0, 0], 4, [1, 0, 1]),   // eoe : ω¹
    raw(&[0, 0, 1, 0], 5, [0, 0, 1]),   // ooe : ω²
    raw(&[0, 1, 0, 0], 16, [1, 1, 0]),  // eeo : ω¹
    raw(&[0, 0, 1, 0], 17, [0, 1, 0]),  // oeo : ω²
    raw(&[0, 0, 1, 0], 20, [1, 0, 0]),  // eoo : ω²
    raw(&[0, 0, 0, 1], 21, [0, 0, 0]),  // ooo : ω³
];

/// One pairwise-product term: three independent columns, one per qubit pair,
/// coefficient ω^{#odd}/8.
const fn raw(coeff_c: &[i128; 4], h: u32, d3: [u8; 3]) -> RawTerm {
    RawTerm {
        coeff_c: [coeff_c[0], coeff_c[1], coeff_c[2], coeff_c[3]],
        coeff_m: 6,
        k: 3,
        cols: [0b000011, 0b001100, 0b110000, 0, 0, 0],
        h,
        d: [d3[0], d3[1], d3[2], 0, 0, 0],
        j: [0; A6_WIDTH],
    }
}

const A6_PROVENANCE_TEXT: &str = "\
Pairwise product decomposition: rank 8 per six T gates, rank 2 per two, \
asymptotic exponent 0.5000. DERIVED AND VERIFIED HERE from chi(|A^2>) = 2. \
This is NOT the Bravyi-Gosset rank 7 per six (exponent 0.4679). BG obtained \
theirs by numerical search; the search run in this lane did not find it, and \
the honest reading of that is a failure to reproduce, NOT evidence against \
rank 7 -- which is known to exist. Search record: (1) annealing over 7 \
stabilizer states, plateau ~0.09 residual; (2) alternating maximisation, \
DISCARDED because it failed its own planted control (it could not rediscover \
the known rank 8 from random starts, so its silence at 7 meant nothing); \
(3) subset search over a 1371-atom dictionary -- product states of all 15 \
qubit pairings, the computational basis, and their images under the symmetry \
group <U_0..U_5> with U = (X+Y)/sqrt2, U|A> = |A> -- whose control DOES pass \
(rank 8 found at residual 0, mixing three pairings) and which found nothing \
at rank 7. That last negative is real but narrow: it rules out rank 7 inside \
that dictionary only. Two algebraic negatives were also derived: merging any \
two product terms into one stabilizer state is impossible (their coefficient \
ratio is an ODD power of omega, so the merged phase function is not a \
Clifford phase), and the parity-sector split can only ever give an EVEN rank, \
so BG's 7 must be carried by a sector-mixing state. \
The interface, the gadget reduction and the exactness gate are rank-agnostic: \
verify a rank-7 table, paste it into A6_DATA, and block_plan starts using \
six-wide blocks on its own.";

// ---------------------------------------------------------------- naive source

/// Cache ceiling: above this many branches the source recomputes per query
/// instead of holding every end-state. 2^18 affine states is already a lot of
/// memory; the honest failure is slow, never wrong.
const CACHE_MAX: u64 = 1 << 18;

/// The 2^t branch sum: T = ((1+ω)/2)·I + ((1−ω)/2)·Z, one two-way branch per
/// T gate. Semantics identical to the certified reference.
pub struct NaiveSource {
    n: usize,
    t: usize,
    gates: Vec<Gate>,
    mutation: Mutation,
    cache: Vec<(Cyc, Affine)>,
}

fn t_coeffs(dag: bool) -> (Cyc, Cyc) {
    // T = (1+ω)/2 · I + (1−ω)/2 · Z ; T† with ω → ω⁻¹ = −ω³.
    if !dag {
        (Cyc { c: [1, 1, 0, 0], m: 2 }, Cyc { c: [1, -1, 0, 0], m: 2 })
    } else {
        (Cyc { c: [1, 0, 0, -1], m: 2 }, Cyc { c: [1, 0, 0, 1], m: 2 })
    }
}

impl NaiveSource {
    pub fn new(c: &Circuit) -> Self {
        Self::with_mutation(c, Mutation::default())
    }

    pub fn with_mutation(c: &Circuit, mutation: Mutation) -> Self {
        let t = c.t_count();
        assert!(t < 63, "T-count out of range for a u64 branch index");
        let mut s = NaiveSource {
            n: c.n_qubits,
            t,
            gates: c.gates.clone(),
            mutation,
            cache: Vec::new(),
        };
        let nb = s.n_branches();
        if nb <= CACHE_MAX {
            s.cache = (0..nb).map(|b| s.run_branch(b)).collect();
        }
        s
    }

    fn run_branch(&self, branch: u64) -> (Cyc, Affine) {
        let mut st = Affine::with_mutations(self.n, self.mutation.into());
        let mut coeff = Cyc::ONE;
        let mut ti = 0usize;
        for g in &self.gates {
            match *g {
                Gate::T(q) | Gate::Tdg(q) => {
                    let (ci, cz) = t_coeffs(matches!(*g, Gate::Tdg(_)));
                    let z_branch = (branch >> ti) & 1 == 1;
                    ti += 1;
                    if z_branch {
                        coeff = coeff.mul(cz);
                        st.apply(Gate::Z(q));
                    } else {
                        coeff = coeff.mul(ci);
                    }
                }
                g => st.apply(g),
            }
        }
        (coeff, st)
    }
}

impl BranchSource for NaiveSource {
    fn n_branches(&self) -> u64 {
        1u64 << self.t
    }

    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        assert!(branch < self.n_branches(), "branch index out of range");
        if let Some((coeff, st)) = self.cache.get(branch as usize) {
            let a = st.amplitude(y);
            return if is_zero(a) { Cyc::ZERO } else { coeff.mul(a) };
        }
        let (coeff, st) = self.run_branch(branch);
        let a = st.amplitude(y);
        if is_zero(a) {
            Cyc::ZERO
        } else {
            coeff.mul(a)
        }
    }

    fn n_qubits(&self) -> usize {
        self.n
    }
}

// ------------------------------------------------------------------ BG source

/// How the t magic qubits are cut into blocks, and which verified table each
/// block carries.
///
/// The width is CHOSEN, not assumed: a six-wide block is worth taking only if
/// its table beats three pair blocks, i.e. only if `A6_RANK < 8`. At the rank
/// this lane actually verified (8 = 2³, three independent pairs) it does not,
/// and six-wide blocking would be strictly WORSE — it costs 2^{t mod 6} on the
/// remainder where pairs cost 2^{⌈(t mod 6)/2⌉} (t = 10: 128 branches against
/// 32). So the plan falls through to pairs, and the six-wide branch turns
/// itself on the moment a genuine rank-7 table is verified into `A6_DATA`.
fn block_plan(n: usize, t: usize) -> Vec<(Vec<usize>, Vec<StabTerm>)> {
    let mut blocks: Vec<(Vec<usize>, Vec<StabTerm>)> = Vec::new();
    let mut pos = 0usize;
    // Three pair blocks cost 2³ = 8; take six-wide blocks only if they beat it.
    if A6_RANK < 8 {
        while t - pos >= A6_WIDTH {
            blocks.push(((0..A6_WIDTH).map(|i| n + pos + i).collect(), a6_terms()));
            pos += A6_WIDTH;
        }
    }
    while t - pos >= 2 {
        blocks.push((vec![n + pos, n + pos + 1], a2_terms()));
        pos += 2;
    }
    while pos < t {
        blocks.push((vec![n + pos], a1_terms()));
        pos += 1;
    }
    blocks
}

/// The Bravyi–Gosset source: T gates gadgetised onto a magic register, which
/// is decomposed block by block.
///
/// The gadget (standard, credited to the magic-state-injection literature):
/// with ancilla a in |A⟩ = (|0⟩+ω|1⟩)/√2, CX(q→a) followed by POST-SELECTING
/// a = 0 leaves T|ψ⟩/√2 on q. The ancilla is never touched again, so the
/// post-selection commutes to the end of the circuit and is nothing but an
/// amplitude query at (y, 0^t) — no measurement machinery, no adaptivity.
/// T† is the same gadget followed by a Clifford S†, since S†T = T†.
/// Hence ⟨y|U|0^n⟩ = 2^{t/2} · ⟨y,0^t| C |0^n, A^t⟩, and the 2^{t/2} is exact
/// in the ring (m = −t).
///
/// The register |A⟩^{⊗t} then factorises into blocks by [`block_plan`], each
/// block carrying a table that `decomposition_is_exact` has verified.
pub struct BgSource {
    n: usize,
    t: usize,
    n_ext: usize,
    ext_gates: Vec<Gate>,
    /// Per block: the ancilla qubits it owns and its decomposition.
    blocks: Vec<(Vec<usize>, Vec<StabTerm>)>,
    mutation: Mutation,
    cache: Vec<(Cyc, Affine)>,
}

impl BgSource {
    pub fn new(c: &Circuit) -> Self {
        Self::with_mutation(c, Mutation::default())
    }

    pub fn with_mutation(c: &Circuit, mutation: Mutation) -> Self {
        let t = c.t_count();
        let n = c.n_qubits;
        // The decomposition is re-verified at every construction. It is cheap
        // (2^6 exact amplitude sums) and it is the whole warrant.
        assert!(
            decomposition_is_exact(&a6_terms(), A6_WIDTH),
            "the {A6_WIDTH}-qubit magic decomposition is NOT exact"
        );
        assert!(decomposition_is_exact(&a2_terms(), 2), "the |A^2⟩ split is NOT exact");
        assert!(decomposition_is_exact(&a1_terms(), 1), "the |A⟩ split is NOT exact");

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

        let blocks = block_plan(n, t);

        let mut s = BgSource {
            n,
            t,
            n_ext: n + t,
            ext_gates,
            blocks,
            mutation,
            cache: Vec::new(),
        };
        let nb = s.n_branches();
        if nb <= CACHE_MAX {
            s.cache = (0..nb).map(|b| s.run_branch(b)).collect();
        }
        s
    }

    /// The exponent actually achieved: log2(branches)/t.
    pub fn exponent(&self) -> f64 {
        if self.t == 0 {
            return 0.0;
        }
        (self.n_branches() as f64).log2() / self.t as f64
    }

    fn run_branch(&self, branch: u64) -> (Cyc, Affine) {
        let mut st = Affine::with_mutations(self.n_ext, self.mutation.into());
        // 2^{t/2} from the post-selected gadgets, exact.
        let mut coeff = Cyc { c: [1, 0, 0, 0], m: -(self.t as i32) };
        let mut rest = branch;
        for (qs, terms) in &self.blocks {
            let digit = (rest % terms.len() as u64) as usize;
            rest /= terms.len() as u64;
            let term = &terms[digit];
            assert_eq!(qs.len(), term.nq, "attach: qubit count vs term width");
            st.attach(qs, &term.cols, term.h, &term.d, &term.j);
            coeff = coeff.mul(term.coeff);
        }
        for g in &self.ext_gates {
            st.apply(*g);
        }
        (coeff, st)
    }

    fn amp_ext(&self, coeff: Cyc, st: &Affine, y: &[bool]) -> Cyc {
        let mut y_ext = vec![false; self.n_ext];
        y_ext[..self.n].copy_from_slice(y);
        let a = st.amplitude(&y_ext);
        if is_zero(a) {
            Cyc::ZERO
        } else {
            coeff.mul(a)
        }
    }
}

impl BranchSource for BgSource {
    fn n_branches(&self) -> u64 {
        self.blocks.iter().map(|(_, t)| t.len() as u64).product()
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

// ---------------------------------------------------------------- the fold

/// Fold a branch source into one exact amplitude, through THE ONE MERGE LAW
/// (`merge::fold`, `Cyc: MergeLedger`). Exact Z[ω] sums are associative and
/// commutative, so this is the same number under any sharding or ordering —
/// which is the mesh's warrant, and the reason this is not a local loop.
pub fn amplitude<S: BranchSource + ?Sized>(src: &S, y: &[bool]) -> Cyc {
    crate::merge::fold((0..src.n_branches()).map(|b| src.amplitude_of(b, y)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompositions_are_exact() {
        assert!(decomposition_is_exact(&a1_terms(), 1));
        assert!(decomposition_is_exact(&a6_terms(), A6_WIDTH));
        assert_eq!(a6_terms().len(), A6_RANK);
    }

    #[test]
    fn t_on_plus_matches_by_hand() {
        // H then T on |0⟩: amplitudes (1/√2, ω/√2).
        let c = Circuit { n_qubits: 1, gates: vec![Gate::H(0), Gate::T(0)] };
        for (y, want) in [(false, Cyc { c: [1, 0, 0, 0], m: 1 }), (true, Cyc { c: [0, 1, 0, 0], m: 1 })] {
            let n = amplitude(&NaiveSource::new(&c), &[y]);
            let b = amplitude(&BgSource::new(&c), &[y]);
            assert!(cyc_eq(n, want), "naive {n:?} vs {want:?}");
            assert!(cyc_eq(b, want), "bg {b:?} vs {want:?}");
        }
    }
}
