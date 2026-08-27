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

pub const N_MAX_STATEVECTOR: usize = 24;

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
}

// ---------------------------------------------------------------- parsing

pub fn parse(src: &str) -> Result<Circuit, String> {
    let mut n_qubits = 0usize;
    let mut n_clbits = 0usize;
    let mut gates = Vec::new();
    let mut measures: Vec<(usize, usize)> = Vec::new();
    let idx = |tok: &str, name: &str| -> Result<usize, String> {
        let open = tok.find('[').ok_or_else(|| format!("bad operand {tok}"))?;
        let close = tok.find(']').ok_or_else(|| format!("bad operand {tok}"))?;
        if !tok.starts_with(name) {
            return Err(format!("expected register {name} in {tok}"));
        }
        tok[open + 1..close].parse().map_err(|_| format!("bad index {tok}"))
    };
    for raw in src.lines() {
        let line = raw.split("//").next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        for stmt in line.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty()
                || stmt.starts_with("OPENQASM")
                || stmt.starts_with("include")
            {
                continue;
            }
            if let Some(rest) = stmt.strip_prefix("qreg ") {
                // qreg q[n]; the bracket holds the SIZE
                n_qubits = rest.trim()[rest.trim().find('[').unwrap() + 1
                    ..rest.trim().find(']').unwrap()]
                    .parse()
                    .map_err(|_| "bad qreg".to_string())?;
                continue;
            }
            if let Some(rest) = stmt.strip_prefix("creg ") {
                n_clbits = rest.trim()[rest.trim().find('[').unwrap() + 1
                    ..rest.trim().find(']').unwrap()]
                    .parse()
                    .map_err(|_| "bad creg".to_string())?;
                continue;
            }
            if let Some(rest) = stmt.strip_prefix("measure ") {
                let parts: Vec<&str> = rest.split("->").collect();
                if parts.len() != 2 {
                    return Err(format!("bad measure: {stmt}"));
                }
                measures.push((idx(parts[0].trim(), "q")?, idx(parts[1].trim(), "c")?));
                continue;
            }
            let (op, args) = stmt
                .split_once(' ')
                .ok_or_else(|| format!("bad statement: {stmt}"))?;
            if !measures.is_empty() {
                return Err("measurements must be terminal in this subset".into());
            }
            let a: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
            let g = match (op.trim(), a.len()) {
                ("x", 1) => Gate::X(idx(a[0], "q")?),
                ("z", 1) => Gate::Z(idx(a[0], "q")?),
                ("h", 1) => Gate::H(idx(a[0], "q")?),
                ("s", 1) => Gate::S(idx(a[0], "q")?),
                ("sdg", 1) => Gate::Sdg(idx(a[0], "q")?),
                ("t", 1) => Gate::T(idx(a[0], "q")?),
                ("tdg", 1) => Gate::Tdg(idx(a[0], "q")?),
                ("cx", 2) => Gate::Cx(idx(a[0], "q")?, idx(a[1], "q")?),
                ("ccx", 3) => {
                    Gate::Ccx(idx(a[0], "q")?, idx(a[1], "q")?, idx(a[2], "q")?)
                }
                _ => return Err(format!("unsupported gate: {stmt}")),
            };
            gates.push(g);
        }
    }
    if n_qubits == 0 {
        return Err("no qreg".into());
    }
    Ok(Circuit { n_qubits, n_clbits, gates, measures })
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
    if c.n_qubits <= N_MAX_STATEVECTOR {
        return Ok(Tier::Statevector);
    }
    Err(format!(
        "REFUSED by the wall: circuit is non-Clifford (T/Tdg or Ccx present) at \
         n = {} > {} qubits. The tableau view is not Closed under non-Clifford \
         motions (lean/CIRISHolon/Stabilizer.lean: tableau_not_closed_under_rotation), \
         and the statevector carrier costs 2^n. The stabilizer-rank tier that \
         would price this by its T-count is owed, not pretended.",
        c.n_qubits, N_MAX_STATEVECTOR
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
#[derive(Clone)]
pub struct Tableau {
    n: usize,
    x: Vec<Vec<bool>>,
    z: Vec<Vec<bool>>,
    r: Vec<bool>,
    m: Mutation,
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

    fn g(x1: bool, z1: bool, x2: bool, z2: bool) -> i32 {
        match (x1, z1) {
            (false, false) => 0,
            (true, true) => (z2 as i32) - (x2 as i32),
            (true, false) => (z2 as i32) * (2 * (x2 as i32) - 1),
            (false, true) => (x2 as i32) * (1 - 2 * (z2 as i32)),
        }
    }

    fn rowsum(&mut self, h: usize, i: usize) {
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
    assert!(n <= N_MAX_STATEVECTOR, "router guarantees the cap");
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

pub fn run(c: &Circuit, tier: Tier, m: Mutation) -> BTreeMap<String, f64> {
    match tier {
        Tier::Classical => run_classical(c, m),
        Tier::Tableau => run_tableau(c, m),
        Tier::Statevector => run_statevector(c),
    }
}
