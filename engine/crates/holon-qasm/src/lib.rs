//! holon-qasm — the stratified QASM simulator.
//!
//! Every circuit is routed to the cheapest tier whose VIEW of the dynamics is
//! Closed (lean/CIRISHolon/Object.lean, Stabilizer.lean):
//!
//! * CLASSICAL — gates {x, cx, ccx}: the diagonal retract. Basis states
//!   evolve as bits; conformance IS the retract test (`lift_commutes`).
//! * TABLEAU — Clifford gates {h, s, sdg, z, x, cx}: the stabilizer view is
//!   Closed under Clifford motions (`tableau_closed_under_hadamard`'s n-qubit
//!   engineering face, Aaronson–Gottesman 2004 for the algorithm, credited).
//!   Cost O(n^2) per gate — 200-qubit Clifford circuits run where a
//!   statevector cannot exist.
//! * STATEVECTOR — anything else, refused above N_MAX qubits BY NAME: past
//!   the tableau wall (`tableau_not_closed_under_rotation`) the carrier costs
//!   2^n, and pretending otherwise is what this engine never does.
//!
//! Scope, stated: OpenQASM 2.0 subset — one qreg, one creg, gates
//! x z h s sdg cx ccx t tdg, terminal measurements only. Exact output
//! distributions (no sampling). Planted-mutation hooks (`Mutation`) exist so
//! the conformance harness can prove it would catch a wrong implementation.

use std::collections::BTreeMap;

pub mod magic;
pub mod qasm;

pub use qasm::{are_tableaux_equivalent, canonicalize_circuit, parse_qasm};

pub const N_MAX_STATEVECTOR: usize = 24;

/// The magic tier's branch budget. Clifford+T with t T-gates is a sum of 2^t
/// phase-tracked stabilizer branches, so t IS the price. 2^24 = 16,777,216.
pub const T_MAX_MAGIC: usize = 24;

/// The T-count at or below which `route` PREFERS magic to the carrier. A
/// preference, not a wall: `T_MAX_MAGIC` is the wall, and `--tier magic`
/// reaches up to it.
pub const T_ROUTE_MAGIC: usize = 12;

pub fn t_count(c: &Circuit) -> usize {
    c.gates.iter().filter(|g| matches!(g, Gate::T(_) | Gate::Tdg(_))).count()
}

/// `2^k`, and its decimal value while one fits.
fn pow2(k: usize) -> String {
    if k < 64 {
        format!("2^{k} = {}", 1u64 << k)
    } else {
        format!("2^{k}")
    }
}

/// The AMPLITUDE wall: 2^t stabilizer branches, poly(n) work each, no 2^n
/// anywhere — so the T-count alone prices this path.
pub fn check_t_budget(c: &Circuit) -> Result<(), String> {
    let t = t_count(c);
    if t <= T_MAX_MAGIC {
        return Ok(());
    }
    Err(format!(
        "REFUSED by the magic wall: T-count {t} > {T_MAX_MAGIC}, the magic \
         tier's branch budget. The tableau view is not Closed under \
         non-Clifford motions (lean/CIRISHolon/Stabilizer.lean: \
         tableau_not_closed_under_rotation), so Clifford+T is priced as a sum \
         of 2^t phase-tracked stabilizer branches: this call would have \
         enumerated {} of them at n = {}. Raising the budget is a flag away — \
         pretending the cost is not there never is.",
        pow2(t),
        c.n_qubits
    ))
}

/// The 2^n wall. Shared by the carrier and by the magic tier's DISTRIBUTION
/// path, which accumulates into a 2^n array of exact `Cyc` amplitudes — 80
/// bytes each against the carrier's 16, so letting it share the carrier's n
/// is the generous reading of that budget, not a loose one. `carrier` names
/// which array is being priced, so the refusal says whose budget blew.
pub fn check_n_budget(c: &Circuit, carrier: &str) -> Result<(), String> {
    let n = c.n_qubits;
    if n <= N_MAX_STATEVECTOR {
        return Ok(());
    }
    Err(format!(
        "REFUSED by the carrier wall: n = {n} > {N_MAX_STATEVECTOR} qubits, \
         and {carrier}, which is {} entries. The AMPLITUDE path (`amp`) is \
         priced at 2^t · poly(n) with no 2^n anywhere and is open here at \
         T-count {}. Raising the budget is a flag away — pretending the cost \
         is not there never is.",
        pow2(n),
        t_count(c)
    ))
}

/// The DISTRIBUTION wall: the magic tier's branch sum pays BOTH budgets, 2^t
/// branches accumulated into 2^n exact amplitudes.
pub fn check_magic_distribution_budget(c: &Circuit) -> Result<(), String> {
    check_t_budget(c)?;
    check_n_budget(c, "the magic tier's distribution path accumulates 2^n exact amplitudes")
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Gate {
    X(usize),
    Z(usize),
    H(usize),
    S(usize),
    Sdg(usize),
    Cx(usize, usize),
    Ccx(usize, usize, usize),
    T(usize),
    Tdg(usize),
}

#[derive(Clone, Debug)]
pub struct Circuit {
    pub n_qubits: usize,
    pub n_clbits: usize,
    pub gates: Vec<Gate>,
    /// (qubit, clbit), all measurements terminal.
    pub measures: Vec<(usize, usize)>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tier {
    Classical,
    Tableau,
    Magic,
    Statevector,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mutation {
    None,
    /// Drop the S gate's phase update in the tableau (a wrong Clifford).
    TableauSPhase,
    /// Drop the CNOT phase term in the tableau.
    TableauCxPhase,
    /// Classical tier applies CX with control/target swapped.
    ClassicalCxSwap,
    /// Magic tier drops the S gate's pairwise J flips.
    MagicSCross,
    /// Magic tier uses the wrong odd-delta Gauss-sum phase.
    MagicGauss,
}

// ---------------------------------------------------------------- parsing

pub fn parse(src: &str) -> Result<Circuit, String> {
    qasm::parse_qasm(src)
}

// ---------------------------------------------------------------- routing

/// The router: the cheapest tier whose view is Closed for this gate set.
/// Returns the tier or the REFUSAL, by name.
pub fn route(c: &Circuit) -> Result<Tier, String> {
    let mut clifford_only = true;
    let mut classical_only = true;
    for g in &c.gates {
        match g {
            Gate::X(_) | Gate::Cx(_, _) => {}
            Gate::Ccx(_, _, _) => clifford_only = false,
            Gate::Z(_) | Gate::H(_) | Gate::S(_) | Gate::Sdg(_) => {
                classical_only = false
            }
            Gate::T(_) | Gate::Tdg(_) => {
                classical_only = false;
                clifford_only = false;
            }
        }
    }
    if classical_only {
        return Ok(Tier::Classical);
    }
    if clifford_only {
        return Ok(Tier::Tableau);
    }
    let t = t_count(c);
    // `route` picks a tier for the DISTRIBUTION path, which accumulates into a
    // 2^n array on the magic tier as much as on the carrier. Sending a wide
    // circuit to magic on its T-count alone walks past the n wall the carrier
    // arm below enforces.
    if t <= T_ROUTE_MAGIC
        && !c.gates.iter().any(|g| matches!(g, Gate::Ccx(..)))
        && c.n_qubits <= N_MAX_STATEVECTOR
    {
        return Ok(Tier::Magic);
    }
    if c.n_qubits <= N_MAX_STATEVECTOR {
        return Ok(Tier::Statevector);
    }
    Err(format!(
        "REFUSED by the wall: non-Clifford circuit (T-count {}) at n = {} > {} \
         qubits. The tableau view is not Closed under non-Clifford motions \
         (lean/CIRISHolon/Stabilizer.lean: tableau_not_closed_under_rotation), \
         so no poly tier is open; the statevector carrier costs 2^n, and the \
         magic tier's DISTRIBUTION path costs 2^t branches into a 2^n \
         accumulator, so neither fits at this n. The AMPLITUDE path (`amp`) is \
         priced at 2^t · poly(n) with no 2^n and is open here while the T-count \
         stays within {}. Raising a budget is a flag away — pretending the cost \
         is not there never is.",
        t, c.n_qubits, N_MAX_STATEVECTOR, T_MAX_MAGIC
    ))
}

// ---------------------------------------------------------------- classical

pub fn run_classical(c: &Circuit, m: Mutation) -> BTreeMap<String, f64> {
    let mut bits = vec![false; c.n_qubits];
    for g in &c.gates {
        match *g {
            Gate::X(a) => bits[a] = !bits[a],
            Gate::Cx(a, b) => {
                let (ctl, tgt) =
                    if m == Mutation::ClassicalCxSwap { (b, a) } else { (a, b) };
                if bits[ctl] {
                    bits[tgt] = !bits[tgt];
                }
            }
            Gate::Ccx(a, b, t) => {
                if bits[a] && bits[b] {
                    bits[t] = !bits[t];
                }
            }
            _ => unreachable!("router guarantees classical gate set"),
        }
    }
    let mut key = vec![b'0'; c.n_clbits];
    for &(q, cl) in &c.measures {
        if bits[q] {
            key[c.n_clbits - 1 - cl] = b'1';
        }
    }
    let mut out = BTreeMap::new();
    out.insert(String::from_utf8(key).unwrap(), 1.0);
    out
}

// ---------------------------------------------------------------- tableau

/// Aaronson–Gottesman CHP tableau: rows 0..n destabilizers, n..2n stabilizers.
#[derive(Clone, Debug, PartialEq)]
pub struct Tableau {
    pub n: usize,
    pub x: Vec<Vec<bool>>,
    pub z: Vec<Vec<bool>>,
    pub r: Vec<bool>,
    pub m: Mutation,
}

impl Tableau {
    pub fn new(n: usize, m: Mutation) -> Self {
        let mut x = vec![vec![false; n]; 2 * n];
        let mut z = vec![vec![false; n]; 2 * n];
        for i in 0..n {
            x[i][i] = true;
            z[n + i][i] = true;
        }
        Tableau { n, x, z, r: vec![false; 2 * n], m }
    }

    pub fn g(x1: bool, z1: bool, x2: bool, z2: bool) -> i32 {
        match (x1, z1) {
            (false, false) => 0,
            (true, true) => (z2 as i32) - (x2 as i32),
            (true, false) => (z2 as i32) * (2 * (x2 as i32) - 1),
            (false, true) => (x2 as i32) * (1 - 2 * (z2 as i32)),
        }
    }

    pub fn rowsum(&mut self, h: usize, i: usize) {
        let mut s = 2 * (self.r[h] as i32) + 2 * (self.r[i] as i32);
        for j in 0..self.n {
            s += Self::g(self.x[i][j], self.z[i][j], self.x[h][j], self.z[h][j]);
        }
        let (xi, xh) = (self.x[i].clone(), self.x[h].clone());
        let (zi, zh) = (self.z[i].clone(), self.z[h].clone());
        for j in 0..self.n {
            self.x[h][j] = xi[j] ^ xh[j];
            self.z[h][j] = zi[j] ^ zh[j];
        }
        self.r[h] = (s.rem_euclid(4)) == 2;
    }

    pub fn apply(&mut self, g: Gate) {
        match g {
            Gate::H(a) => {
                for i in 0..2 * self.n {
                    self.r[i] ^= self.x[i][a] & self.z[i][a];
                    let t = self.x[i][a];
                    self.x[i][a] = self.z[i][a];
                    self.z[i][a] = t;
                }
            }
            Gate::S(a) => {
                for i in 0..2 * self.n {
                    if self.m != Mutation::TableauSPhase {
                        self.r[i] ^= self.x[i][a] & self.z[i][a];
                    }
                    self.z[i][a] ^= self.x[i][a];
                }
            }
            Gate::Sdg(a) => {
                self.apply(Gate::S(a));
                self.apply(Gate::S(a));
                self.apply(Gate::S(a));
            }
            Gate::Z(a) => {
                for i in 0..2 * self.n {
                    self.r[i] ^= self.x[i][a];
                }
            }
            Gate::X(a) => {
                for i in 0..2 * self.n {
                    self.r[i] ^= self.z[i][a];
                }
            }
            Gate::Cx(c, t) => {
                for i in 0..2 * self.n {
                    if self.m != Mutation::TableauCxPhase {
                        self.r[i] ^=
                            self.x[i][c] & self.z[i][t] & (self.x[i][t] ^ self.z[i][c] ^ true);
                    }
                    self.x[i][t] ^= self.x[i][c];
                    self.z[i][c] ^= self.z[i][t];
                }
            }
            _ => unreachable!("router guarantees Clifford gate set"),
        }
    }

    /// Measure qubit a. Deterministic → Some(outcome); random → None (caller
    /// branches on both outcomes with `collapse`).
    pub fn measure_peek(&self, a: usize) -> Option<bool> {
        for p in self.n..2 * self.n {
            if self.x[p][a] {
                return None;
            }
        }
        // determinate: accumulate destabilizer contributions in a scratch row
        let mut scratch = self.clone();
        let sc = 2 * self.n; // virtual scratch index handled inline below
        let _ = sc;
        let mut sx = vec![false; self.n];
        let mut sz = vec![false; self.n];
        let mut sr = false;
        for i in 0..self.n {
            if self.x[i][a] {
                // rowsum(scratch, i + n) inlined
                let mut s = 2 * (sr as i32) + 2 * (self.r[i + self.n] as i32);
                for j in 0..self.n {
                    s += Self::g(
                        self.x[i + self.n][j],
                        self.z[i + self.n][j],
                        sx[j],
                        sz[j],
                    );
                }
                for j in 0..self.n {
                    sx[j] ^= self.x[i + self.n][j];
                    sz[j] ^= self.z[i + self.n][j];
                }
                sr = (s.rem_euclid(4)) == 2;
            }
        }
        let _ = &mut scratch;
        Some(sr)
    }

    /// Collapse a RANDOM measurement of qubit a to the given outcome.
    pub fn collapse(&mut self, a: usize, outcome: bool) {
        let p = (self.n..2 * self.n)
            .find(|&p| self.x[p][a])
            .expect("collapse requires a random measurement");
        for i in 0..2 * self.n {
            if i != p && self.x[i][a] {
                self.rowsum(i, p);
            }
        }
        let (xp, zp, rp) = (self.x[p].clone(), self.z[p].clone(), self.r[p]);
        self.x[p - self.n] = xp;
        self.z[p - self.n] = zp;
        self.r[p - self.n] = rp;
        for j in 0..self.n {
            self.x[p][j] = false;
            self.z[p][j] = false;
        }
        self.z[p][a] = true;
        self.r[p] = outcome;
    }
}

pub fn run_tableau(c: &Circuit, m: Mutation) -> BTreeMap<String, f64> {
    let mut t = Tableau::new(c.n_qubits, m);
    for g in &c.gates {
        t.apply(*g);
    }
    let mut out = BTreeMap::new();
    let mut stack: Vec<(Tableau, usize, Vec<bool>, f64)> =
        vec![(t, 0, vec![false; c.n_clbits], 1.0)];
    while let Some((tab, mi, bits, w)) = stack.pop() {
        if mi == c.measures.len() {
            let key: String = (0..c.n_clbits)
                .rev()
                .map(|i| if bits[i] { '1' } else { '0' })
                .collect();
            *out.entry(key).or_insert(0.0) += w;
            continue;
        }
        let (q, cl) = c.measures[mi];
        match tab.measure_peek(q) {
            Some(o) => {
                let mut b = bits;
                b[cl] = o;
                stack.push((tab, mi + 1, b, w));
            }
            None => {
                for outcome in [false, true] {
                    let mut t2 = tab.clone();
                    t2.collapse(q, outcome);
                    let mut b = bits.clone();
                    b[cl] = outcome;
                    stack.push((t2, mi + 1, b, w * 0.5));
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------- statevector

pub fn run_statevector(c: &Circuit) -> BTreeMap<String, f64> {
    let n = c.n_qubits;
    // `--tier statevector` and every library caller reach here without the
    // router, so the wall is enforced here and not merely upstream of here.
    if let Err(reason) =
        check_n_budget(c, "the statevector carrier stores 2^n amplitudes")
    {
        panic!("{reason}");
    }
    let dim = 1usize << n;
    let mut re = vec![0.0f64; dim];
    let mut im = vec![0.0f64; dim];
    re[0] = 1.0;
    let inv_sqrt2 = 1.0 / (2.0f64).sqrt();
    let c8 = (std::f64::consts::FRAC_PI_4).cos();
    let s8 = (std::f64::consts::FRAC_PI_4).sin();
    for g in &c.gates {
        match *g {
            Gate::X(a) => {
                let bit = 1usize << a;
                for i in 0..dim {
                    if i & bit == 0 {
                        re.swap(i, i | bit);
                        im.swap(i, i | bit);
                    }
                }
            }
            Gate::Z(a) => {
                let bit = 1usize << a;
                for i in 0..dim {
                    if i & bit != 0 {
                        re[i] = -re[i];
                        im[i] = -im[i];
                    }
                }
            }
            Gate::H(a) => {
                let bit = 1usize << a;
                for i in 0..dim {
                    if i & bit == 0 {
                        let (r0, i0, r1, i1) = (re[i], im[i], re[i | bit], im[i | bit]);
                        re[i] = (r0 + r1) * inv_sqrt2;
                        im[i] = (i0 + i1) * inv_sqrt2;
                        re[i | bit] = (r0 - r1) * inv_sqrt2;
                        im[i | bit] = (i0 - i1) * inv_sqrt2;
                    }
                }
            }
            Gate::S(a) | Gate::Sdg(a) => {
                let bit = 1usize << a;
                let sgn = if matches!(g, Gate::S(_)) { 1.0 } else { -1.0 };
                for i in 0..dim {
                    if i & bit != 0 {
                        let (r, q) = (re[i], im[i]);
                        re[i] = -sgn * q;
                        im[i] = sgn * r;
                    }
                }
            }
            Gate::T(a) | Gate::Tdg(a) => {
                let bit = 1usize << a;
                let sgn = if matches!(g, Gate::T(_)) { 1.0 } else { -1.0 };
                for i in 0..dim {
                    if i & bit != 0 {
                        let (r, q) = (re[i], im[i]);
                        re[i] = r * c8 - sgn * q * s8;
                        im[i] = q * c8 + sgn * r * s8;
                    }
                }
            }
            Gate::Cx(cq, t) => {
                let (cb, tb) = (1usize << cq, 1usize << t);
                for i in 0..dim {
                    if i & cb != 0 && i & tb == 0 {
                        re.swap(i, i | tb);
                        im.swap(i, i | tb);
                    }
                }
            }
            Gate::Ccx(a, b, t) => {
                let (ab, bb, tb) = (1usize << a, 1usize << b, 1usize << t);
                for i in 0..dim {
                    if i & ab != 0 && i & bb != 0 && i & tb == 0 {
                        re.swap(i, i | tb);
                        im.swap(i, i | tb);
                    }
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    for i in 0..dim {
        let p = re[i] * re[i] + im[i] * im[i];
        if p < 1e-14 {
            continue;
        }
        let mut bits = vec![false; c.n_clbits];
        for &(q, cl) in &c.measures {
            bits[cl] = i & (1usize << q) != 0;
        }
        let key: String = (0..c.n_clbits)
            .rev()
            .map(|j| if bits[j] { '1' } else { '0' })
            .collect();
        *out.entry(key).or_insert(0.0) += p;
    }
    out
}

/// One deterministic shot (random outcomes collapsed to 0): the timing path.
/// Distribution mode enumerates measurement branches and is exponential in
/// the number of RANDOM outcomes — using it to benchmark the tableau would
/// rebuild the exponential wall inside the poly tier (measured, 2026-08-27).
pub fn run_tableau_sample(c: &Circuit, m: Mutation) -> String {
    let mut t = Tableau::new(c.n_qubits, m);
    for g in &c.gates {
        t.apply(*g);
    }
    let mut bits = vec![false; c.n_clbits];
    for &(q, cl) in &c.measures {
        match t.measure_peek(q) {
            Some(o) => bits[cl] = o,
            None => {
                t.collapse(q, false);
                bits[cl] = false;
            }
        }
    }
    (0..c.n_clbits).rev().map(|i| if bits[i] { '1' } else { '0' }).collect()
}

/// `run`, with the tier's budget checked FIRST so a caller that named its own
/// tier gets the router's named refusal instead of the router's silence. The
/// CLI's `--tier` override goes through here; the panics inside `run` are the
/// backstop for library callers that do not.
pub fn try_run(
    c: &Circuit,
    tier: Tier,
    m: Mutation,
) -> Result<BTreeMap<String, f64>, String> {
    match tier {
        Tier::Magic => check_magic_distribution_budget(c)?,
        Tier::Statevector => {
            check_n_budget(c, "the statevector carrier stores 2^n amplitudes")?
        }
        Tier::Classical | Tier::Tableau => {}
    }
    Ok(run(c, tier, m))
}

pub fn run(c: &Circuit, tier: Tier, m: Mutation) -> BTreeMap<String, f64> {
    match tier {
        Tier::Classical => run_classical(c, m),
        Tier::Tableau => run_tableau(c, m),
        Tier::Magic => magic::run_magic(
            c,
            m == Mutation::MagicSCross,
            m == Mutation::MagicGauss,
        ),
        Tier::Statevector => run_statevector(c),
    }
}
