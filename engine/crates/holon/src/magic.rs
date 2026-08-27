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

// ---------------------------------------------------------------- ring helpers
//
// `ledger::Cyc` carries mul/add/to_complex; the powers of ω and i, and an
// equality that is honest across denominators, live here.

/// i^k = ω^{2k}.
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

/// ω^k, ω = e^{iπ/4}, ω⁴ = −1.
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

pub fn is_zero(a: Cyc) -> bool {
    a.c.iter().all(|&x| x == 0)
}

fn neg(a: Cyc) -> Cyc {
    a.mul(Cyc { c: [-1, 0, 0, 0], m: 0 })
}

/// Exact equality ACROSS denominators: {1,ω,ω²,ω³} is a Z-basis, so at a
/// common m the coefficient vector is unique — and `add` aligns denominators
/// before adding. Comparing the structs directly would call 1 and (ω−ω³)/√2
/// different, which they are not.
pub fn cyc_eq(a: Cyc, b: Cyc) -> bool {
    is_zero(a.add(neg(b)))
}

// ---------------------------------------------------------------- circuits

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gate {
    X(usize),
    Z(usize),
    H(usize),
    S(usize),
    Sdg(usize),
    Cx(usize, usize),
    T(usize),
    Tdg(usize),
}

impl Gate {
    pub fn is_t(&self) -> bool {
        matches!(self, Gate::T(_) | Gate::Tdg(_))
    }
}

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
/// not a conformance harness. Both are ported from the certified reference.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Mutation {
    /// Drop the S gate's pairwise J flips.
    pub drop_s_cross: bool,
    /// Use the wrong odd-δ Gauss-sum phase (drops the 1+ structure).
    pub wrong_gauss: bool,
}

// ---------------------------------------------------------------- affine state

/// A phase-tracked stabilizer state in affine form: x = R u ⊕ h.
#[derive(Clone)]
pub struct Affine {
    n: usize,
    /// R: n rows × k columns.
    r: Vec<Vec<bool>>,
    h: Vec<bool>,
    /// d_a mod 4 (the i-power linear part), one per column.
    d: Vec<u8>,
    /// J_{ab}, symmetric, diagonal unused: (−1)^{J u_a u_b}.
    j: Vec<Vec<bool>>,
    gamma: Cyc,
    zero: bool,
    mutation: Mutation,
}

impl Affine {
    pub fn new(n: usize) -> Self {
        Affine::with_mutation(n, Mutation::default())
    }

    pub fn with_mutation(n: usize, mutation: Mutation) -> Self {
        Affine {
            n,
            r: vec![Vec::new(); n],
            h: vec![false; n],
            d: Vec::new(),
            j: Vec::new(),
            gamma: Cyc::ONE,
            zero: false,
            mutation,
        }
    }

    pub fn n_qubits(&self) -> usize {
        self.n
    }

    fn k(&self) -> usize {
        self.d.len()
    }

    /// Number of free affine coordinates — the state's support is 2^k wide.
    pub fn rank(&self) -> usize {
        self.k()
    }

    pub fn is_zero(&self) -> bool {
        self.zero
    }

    /// u_a := u_a ⊕ u_b (fold a's dependence into b).
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

    /// Remove column a with u_a pinned to val.
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

    /// Sum out phase-only column a (its R column is all-zero):
    /// Σ_w i^{δw} (−1)^{w·Λ}, Λ = Σ_{b∈L} u_b, L = the J-neighbours of a.
    fn gauss_sum_out(&mut self, a: usize) {
        let delta = self.d[a];
        let l: Vec<usize> = (0..self.k()).filter(|&b| b != a && self.j[a][b]).collect();
        match delta % 4 {
            0 | 2 => {
                let eps = delta == 2; // the constraint Λ ≡ eps
                if l.is_empty() {
                    if eps {
                        self.zero = true;
                        self.remove_col(a);
                        return;
                    }
                    self.gamma.m -= 2; // ×2
                    self.remove_col(a);
                } else {
                    // impose Σ_L u = eps: fold L onto c = l[0], then pin it.
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
                let phase = if self.mutation.wrong_gauss {
                    i_pow(delta) // PLANTED WRONG: drops the 1+ structure
                } else {
                    Cyc::ONE.add(i_pow(delta))
                };
                self.gamma = self.gamma.mul(phase);
                // (−i^δ)^{Λ mod 2} with Λ = Σ_L u: the XOR expansion — the
                // identity (1+i^δ(−1)^Λ) = (1+i^δ)(−i^δ)^Λ holds only for
                // Λ ∈ {0,1}, and Λ here is an integer sum, so the factorised
                // form needs i^{(δ+2)(⊕_L u)}: per-variable phases PLUS
                // pairwise (−1)^{u_a u_b} flips across L (measured upstream as
                // a sign error on amp(1,1) of h,cx,h,s,h — the entangled HSH
                // sandwich).
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
        // i^{x_q}: γ·i^h, d_a += 1+2h for a ∈ A, J_ab ^= 1 for a<b ∈ A.
        let a_set: Vec<usize> = (0..self.k()).filter(|&a| self.r[q][a]).collect();
        if self.h[q] {
            self.gamma = self.gamma.mul(i_pow(1));
        }
        let bump = if self.h[q] { 3 } else { 1 };
        for &a in &a_set {
            self.d[a] = (self.d[a] + bump) % 4;
        }
        if !self.mutation.drop_s_cross {
            for i in 0..a_set.len() {
                for jj in i + 1..a_set.len() {
                    let (a, b) = (a_set[i], a_set[jj]);
                    self.j[a][b] = !self.j[a][b];
                    self.j[b][a] = self.j[a][b];
                }
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
                self.gauss_sum_out(a);
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

    pub fn apply(&mut self, g: Gate) {
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
            Gate::T(_) | Gate::Tdg(_) => panic!("magic tier branches must be Clifford"),
        }
    }

    /// Tensor a stabilizer term onto the named qubits. Used to load the magic
    /// register at branch construction; the qubits must be untouched (|0⟩,
    /// no columns) or the affine invariant is not this method's to keep.
    pub fn attach(&mut self, qubits: &[usize], term: &StabTerm) {
        assert_eq!(qubits.len(), term.nq, "attach: qubit count vs term width");
        let base = self.k();
        for (ci, &mask) in term.cols.iter().enumerate() {
            let v = self.k();
            for row in 0..self.n {
                self.r[row].push(false);
            }
            self.d.push(term.d[ci]);
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
        let kt = term.cols.len();
        for a in 0..kt {
            for b in a + 1..kt {
                if term.j[a][b] {
                    self.j[base + a][base + b] = true;
                    self.j[base + b][base + a] = true;
                }
            }
        }
        for (bi, &q) in qubits.iter().enumerate() {
            if (term.h >> bi) & 1 == 1 {
                self.h[q] = !self.h[q];
            }
        }
    }

    /// Exact amplitude of basis state y (bit i = qubit i).
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
                return Cyc::ZERO; // y is not in the affine subspace
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
        let mut st = Affine::with_mutation(self.n, self.mutation);
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
        let mut st = Affine::with_mutation(self.n_ext, self.mutation);
        // 2^{t/2} from the post-selected gadgets, exact.
        let mut coeff = Cyc { c: [1, 0, 0, 0], m: -(self.t as i32) };
        let mut rest = branch;
        for (qs, terms) in &self.blocks {
            let digit = (rest % terms.len() as u64) as usize;
            rest /= terms.len() as u64;
            let term = &terms[digit];
            st.attach(qs, term);
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
