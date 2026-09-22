//! SECTOR — the cheap part of a Clifford+T circuit, LOCATED BY SEARCH.
//!
//! QVM-ACUITY-1's S1 (`conformance/qasm/QVM_ACUITY1_PREREG.md`): *the cheap
//! sector is FOUND, not named.* Two searches run here and neither asks a gate
//! what it is called.
//!
//! # 1. The closure test
//!
//! A gate is CHEAP iff the stabilizer (tableau) view stays Closed across it —
//! the property `lean/CIRISHolon/Stabilizer.lean` states as
//! `tableau_closed_under_hadamard` and denies as
//! `tableau_not_closed_under_rotation`. The test is run on the gate's
//! *unitary*, which is read out of the engine itself
//! ([`unitary_of`]: column `j` is the engine's dense action on `|j⟩`), and it
//! is the definition of Clifford, not a spelling of it:
//!
//! > for every Pauli generator `P` on the gate's support, expand `U P U†` in
//! > the Hermitian Pauli basis `{I,X,Y,Z}^{⊗k}`. If ONE coefficient is
//! > nonzero and it is a phase the engine already tracks (`±1`, the tableau's
//! > sign bit; `±i`, the ledger's `i`-power), the view is CLOSED across the
//! > gate and the surviving coefficient IS the tableau's update rule for it.
//! > If two or more coefficients are nonzero, the Pauli SPLITS — that is the
//! > `+1` the object charges for, and the gate is HARD.
//!
//! So `H`, `S`, `S†`, `X`, `Z`, `CX` come back cheap because their images are
//! single Paulis; `T` comes back hard with a WITNESS —
//! `T X T† = (X + Y)/√2`, two terms of weight `0.7071` — and it would come
//! back hard spelled `rz(π/4)`, `diag(1, ω)`, or as an 8×8 block on three
//! wires, because the test never reads the name. A Clifford dressed in a
//! global phase comes back cheap: a global phase cancels in `U P U†`, which
//! is exactly the sense in which the engine "already tracks" it.
//! [`closure_of_unitary`] is public so that this claim can be put to gates
//! this crate's `Gate` enum cannot even spell.
//!
//! # 2. The backward light cone
//!
//! An observable reads some qubits and not others. Walking the circuit
//! BACKWARD from the end, a gate whose support is disjoint from the set of
//! qubits that can still reach the observable acts entirely inside the part
//! of the register that is traced out (a marginal) or is simply never read,
//! so it cancels against its own adjoint and is REMOVABLE — provably, not
//! probably. A gate that touches the cone is kept and ENLARGES the cone by
//! its own support.
//!
//! Stated plainly, because it decides what the campaign can report: the
//! backward cone of an AMPLITUDE `⟨y|C|0⟩` is the whole register, always.
//! The bra reads all `n` bits, so no gate is ever outside it and
//! `removed = 0` for every amplitude instance. Removal bites on the MARGINAL,
//! where everything off the declared qubit set is traced out. This is a
//! property of the observables the prereg declares, not a weakness of the
//! search; the numbers are reported per instance either way.

use crate::affine::Gate;
use crate::ledger::Cyc;
use crate::magic::Circuit;
use crate::BranchSource;

/// A complex number in the FLOAT lane. The exact path is [`Cyc`]; floats
/// appear here only in the referee and in the closure test's linear algebra,
/// both of which are classifications, never value paths.
pub type C = (f64, f64);

#[inline]
fn cadd(a: C, b: C) -> C {
    (a.0 + b.0, a.1 + b.1)
}
#[inline]
fn cmul(a: C, b: C) -> C {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}
#[inline]
fn cconj(a: C) -> C {
    (a.0, -a.1)
}
#[inline]
fn cabs(a: C) -> f64 {
    (a.0 * a.0 + a.1 * a.1).sqrt()
}

/// The tolerance at which a Pauli coefficient counts as nonzero. The
/// coefficients of a genuine Clifford image are exactly `0` or exactly `±1`
/// up to the rounding of a handful of `1/√2` multiplies, and the smallest
/// coefficient a SPLIT can produce is `1/√2`; the two are separated by nine
/// orders of magnitude, so this number is not a knob.
pub const CLOSURE_TOL: f64 = 1e-9;

// ---------------------------------------------------------------------------
// THE OBSERVABLE
// ---------------------------------------------------------------------------

/// What the sector is located AGAINST. The prereg declares exactly two.
#[derive(Clone, Debug, PartialEq)]
pub enum Observable {
    /// `⟨y|C|0^n⟩` for a declared bitstring `y` (`y[q]` is qubit `q`'s bit).
    Amplitude(Vec<bool>),
    /// `p(bits on qubits)` — the marginal probability of a declared pattern
    /// on a declared qubit set, everything else traced out.
    Marginal { qubits: Vec<usize>, bits: Vec<bool> },
}

impl Observable {
    /// The qubits the observable READS — the seed of the backward light cone.
    pub fn support(&self, n: usize) -> Vec<usize> {
        match self {
            // Every bit of y is read. See the module header.
            Observable::Amplitude(_) => (0..n).collect(),
            Observable::Marginal { qubits, .. } => {
                let mut q = qubits.clone();
                q.sort_unstable();
                q.dedup();
                q
            }
        }
    }

    /// The widest qubit index the observable itself names, plus one.
    pub fn min_width(&self) -> usize {
        match self {
            Observable::Amplitude(y) => y.len(),
            Observable::Marginal { qubits, .. } => {
                qubits.iter().copied().max().map_or(0, |q| q + 1)
            }
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Observable::Amplitude(y) => {
                let s: String = y.iter().map(|&b| if b { '1' } else { '0' }).collect();
                format!("amp[{s}]")
            }
            Observable::Marginal { qubits, bits } => {
                let q: Vec<String> = qubits.iter().map(|q| q.to_string()).collect();
                let s: String = bits.iter().map(|&b| if b { '1' } else { '0' }).collect();
                format!("marginal[{}={}]", q.join(","), s)
            }
        }
    }
}

/// The observable's VALUE: an exact amplitude or a probability. Kept as one
/// type so the driver, the plants and S1 compare like with like.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ObsValue {
    Amp(C),
    Prob(f64),
}

impl ObsValue {
    /// `|a − b|`, the prereg's `10⁻¹²` distance. Comparing an amplitude with
    /// a probability is a category error and says so.
    pub fn abs_diff(self, other: ObsValue) -> f64 {
        match (self, other) {
            (ObsValue::Amp(a), ObsValue::Amp(b)) => cabs((a.0 - b.0, a.1 - b.1)),
            (ObsValue::Prob(a), ObsValue::Prob(b)) => (a - b).abs(),
            _ => panic!("sector: an amplitude and a probability are not comparable"),
        }
    }

    pub fn as_json(self) -> String {
        match self {
            ObsValue::Amp((re, im)) => format!("{{\"re\": {re:.17e}, \"im\": {im:.17e}}}"),
            ObsValue::Prob(p) => format!("{p:.17e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// THE CLOSURE TEST
// ---------------------------------------------------------------------------

/// One Pauli generator's image under conjugation by a gate: `P ↦ i^phase ·
/// X^x Z^z` in the Hermitian basis (`x & z` on a wire means `Y`). This is
/// what a tableau row becomes, so it IS the tableau's update rule for the
/// gate — found, not tabulated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PauliImage {
    /// X-part, bit `j` = local wire `j` of the gate's support.
    pub x: u32,
    /// Z-part, same indexing.
    pub z: u32,
    /// The tracked phase as an `i`-power in `0..4` (`0` = `+1`, `2` = `−1`).
    pub phase: u8,
}

/// What the closure test found across one gate.
#[derive(Clone, Debug, PartialEq)]
pub enum Closure {
    /// The stabilizer view is CLOSED: every generator's image is one Pauli
    /// times a tracked phase. `images` is `[X_0, Z_0, X_1, Z_1, …]` over the
    /// gate's own support.
    Closed { images: Vec<PauliImage> },
    /// NOT closed. `generator` names the Pauli that split, `terms` counts the
    /// Paulis it split into and `weights` gives their magnitudes — the `+1`
    /// the object charges for, with its receipt.
    Split { generator: String, terms: usize, weights: Vec<f64> },
}

impl Closure {
    pub fn is_closed(&self) -> bool {
        matches!(self, Closure::Closed { .. })
    }

    pub fn witness(&self) -> String {
        match self {
            Closure::Closed { images } => format!("closed, {} generator images", images.len()),
            Closure::Split { generator, terms, weights } => {
                let w: Vec<String> = weights.iter().map(|w| format!("{w:.4}")).collect();
                format!("{generator} splits into {terms} Paulis, weights [{}]", w.join(", "))
            }
        }
    }
}

/// `⟨r|P(x,z)|c⟩` for the Hermitian `k`-qubit Pauli word: `[r = c⊕x] ·
/// i^{|x&z|} · (−1)^{|c&z|}` (so `x = z = 1` on a wire is `Y = iXZ`).
fn pauli_entry(x: u32, z: u32, r: usize, c: usize) -> C {
    if r != (c ^ x as usize) {
        return (0.0, 0.0);
    }
    let i_pow = (x & z).count_ones() % 4;
    let sign = if ((c as u32) & z).count_ones() % 2 == 1 { -1.0 } else { 1.0 };
    match i_pow {
        0 => (sign, 0.0),
        1 => (0.0, sign),
        2 => (-sign, 0.0),
        _ => (0.0, -sign),
    }
}

/// `U P U†` for the Pauli word `(px, pz)`, dense on `2^k`.
fn conjugate(k: usize, u: &[C], px: u32, pz: u32) -> Vec<C> {
    let dim = 1usize << k;
    let mut m = vec![(0.0f64, 0.0f64); dim * dim];
    for a in 0..dim {
        for b in 0..dim {
            let mut acc = (0.0, 0.0);
            for d in 0..dim {
                // P has one nonzero per column: row d⊕px of column d.
                let p = pauli_entry(px, pz, d ^ px as usize, d);
                acc = cadd(acc, cmul(cmul(u[a * dim + (d ^ px as usize)], p), cconj(u[b * dim + d])));
            }
            m[a * dim + b] = acc;
        }
    }
    m
}

/// The Pauli-basis expansion of `m`: every `(x, z, coefficient)` whose
/// coefficient survives [`CLOSURE_TOL`]. `c_Q = tr(Q M) / 2^k`, and the
/// Hermitian Pauli words are orthogonal under that pairing.
fn pauli_expand(k: usize, m: &[C]) -> Vec<(u32, u32, C)> {
    let dim = 1usize << k;
    let mut live = Vec::new();
    for qx in 0..dim as u32 {
        for qz in 0..dim as u32 {
            let mut tr = (0.0, 0.0);
            for c in 0..dim {
                let qe = pauli_entry(qx, qz, c ^ qx as usize, c);
                tr = cadd(tr, cmul(qe, m[c * dim + (c ^ qx as usize)]));
            }
            let coeff = (tr.0 / dim as f64, tr.1 / dim as f64);
            if cabs(coeff) > CLOSURE_TOL {
                live.push((qx, qz, coeff));
            }
        }
    }
    live
}

/// The phases the engine already tracks, as an `i`-power: `±1` is the
/// tableau's sign bit, `±i` the ledger's `i`-power. Anything else is not a
/// phase this representation carries.
fn tracked_phase(c: C) -> Option<u8> {
    for (p, v) in [(0u8, (1.0, 0.0)), (1, (0.0, 1.0)), (2, (-1.0, 0.0)), (3, (0.0, -1.0))] {
        if cabs((c.0 - v.0, c.1 - v.1)) < CLOSURE_TOL {
            return Some(p);
        }
    }
    None
}

/// THE CLOSURE TEST, on a `k`-qubit unitary given row-major as `2^k × 2^k`.
///
/// Public, and taking a MATRIX rather than a `Gate`, so that the claim "this
/// is a search, not a gate table" can be put to gates the `Gate` enum cannot
/// spell: a `CZ`, a `√Y`, a Toffoli, a `T` wearing a global phase.
pub fn closure_of_unitary(k: usize, u: &[C]) -> Closure {
    let dim = 1usize << k;
    assert_eq!(u.len(), dim * dim, "closure test: matrix is not 2^k × 2^k");
    let mut images = Vec::with_capacity(2 * k);
    for j in 0..k {
        for (label, (px, pz)) in [("X", (1u32 << j, 0u32)), ("Z", (0u32, 1u32 << j))] {
            let m = conjugate(k, u, px, pz);
            let live = pauli_expand(k, &m);
            match (live.len(), live.first().and_then(|t| tracked_phase(t.2))) {
                (1, Some(phase)) => {
                    images.push(PauliImage { x: live[0].0, z: live[0].1, phase });
                }
                _ => {
                    let mut weights: Vec<f64> = live.iter().map(|t| cabs(t.2)).collect();
                    weights.sort_by(|a, b| b.partial_cmp(a).unwrap());
                    return Closure::Split {
                        generator: format!("{label}_{j}"),
                        terms: live.len(),
                        weights,
                    };
                }
            }
        }
    }
    Closure::Closed { images }
}

/// The qubits a gate touches — read off the gate's DATA. (Which wires it uses
/// is not a classification; whether the view stays closed is, and that is
/// [`closure_of_unitary`]'s business, not this function's.)
pub fn support(g: Gate) -> Vec<usize> {
    match g {
        Gate::X(q) | Gate::Z(q) | Gate::S(q) | Gate::Sdg(q) | Gate::H(q) | Gate::T(q)
        | Gate::Tdg(q) => vec![q],
        Gate::Cx(c, t) => vec![c, t],
    }
}

/// Re-index a gate onto local wires `0..k` of its own support.
fn localize(g: Gate) -> Gate {
    let s = support(g);
    let at = |q: usize| s.iter().position(|&x| x == q).unwrap();
    match g {
        Gate::X(q) => Gate::X(at(q)),
        Gate::Z(q) => Gate::Z(at(q)),
        Gate::S(q) => Gate::S(at(q)),
        Gate::Sdg(q) => Gate::Sdg(at(q)),
        Gate::H(q) => Gate::H(at(q)),
        Gate::T(q) => Gate::T(at(q)),
        Gate::Tdg(q) => Gate::Tdg(at(q)),
        Gate::Cx(c, t) => Gate::Cx(at(c), at(t)),
    }
}

/// The gate's unitary on its own support, READ OUT OF THE ENGINE: column `j`
/// is the engine's dense action on the basis state `|j⟩`. Nothing about the
/// gate's identity is consulted; if the engine's action on a gate changed,
/// this matrix would change with it and so would the closure verdict.
pub fn unitary_of(g: Gate) -> (usize, Vec<C>) {
    let k = support(g).len();
    let local = localize(g);
    let dim = 1usize << k;
    let mut u = vec![(0.0f64, 0.0f64); dim * dim];
    for j in 0..dim {
        let mut st = vec![(0.0f64, 0.0f64); dim];
        st[j] = (1.0, 0.0);
        referee::apply_gate(&mut st, k, local);
        for (i, amp) in st.iter().enumerate() {
            u[i * dim + j] = *amp;
        }
    }
    (k, u)
}

/// The closure test for one gate of the engine's alphabet.
pub fn closure_of(g: Gate) -> Closure {
    let (k, u) = unitary_of(g);
    closure_of_unitary(k, &u)
}

// ---------------------------------------------------------------------------
// THE SECTOR
// ---------------------------------------------------------------------------

/// What the search found.
#[derive(Clone, Debug)]
pub struct Sector {
    /// The gates across which the tableau view stays Closed, in circuit order.
    pub clifford: Vec<Gate>,
    /// The non-Clifford gates the observable DEPENDS ON, with their positions.
    pub hard: Vec<(usize, Gate)>,
    /// The non-Clifford gates OUTSIDE the observable's backward light cone —
    /// provably irrelevant, dropped before the price is stated.
    pub removed: Vec<(usize, Gate)>,
    /// `hard.len()` — the T-count the price is computed from.
    pub t_eff: usize,
    /// The qubits the observable depends on, at the START of the circuit.
    pub light_cone: Vec<usize>,
    /// Per position: is this gate inside the observable's backward cone? The
    /// light cone's own record, kept so the stronger reduction (drop EVERY
    /// out-of-cone gate, Clifford ones too) is available and testable.
    pub keep: Vec<bool>,
    /// The width the search ran at.
    pub n_qubits: usize,
}

impl Sector {
    /// The line the driver prints.
    pub fn line(&self) -> String {
        let q: Vec<String> = self.light_cone.iter().map(|q| q.to_string()).collect();
        format!(
            "sector: clifford={} hard={} removed={} light_cone=[{}]",
            self.clifford.len(),
            self.t_eff,
            self.removed.len(),
            q.join(",")
        )
    }

    /// The price, BEFORE anything runs: `N_pred = expected_branches(t_eff)`.
    pub fn price(&self) -> u64 {
        crate::magic5::expected_branches(self.t_eff)
    }

    /// S1's object: the circuit with every REMOVED gate dropped and nothing
    /// else touched.
    pub fn drop_removed(&self, gates: &[Gate]) -> Vec<Gate> {
        let drop: std::collections::BTreeSet<usize> =
            self.removed.iter().map(|(i, _)| *i).collect();
        gates
            .iter()
            .enumerate()
            .filter(|(i, _)| !drop.contains(i))
            .map(|(_, g)| *g)
            .collect()
    }

    /// The stronger reduction the same cone licenses: every gate outside it,
    /// Clifford ones included. Not what S1 stakes — S1 stakes the T-gates —
    /// but the same theorem, and the tests put it to the same referee.
    pub fn light_cone_circuit(&self, gates: &[Gate]) -> Vec<Gate> {
        gates
            .iter()
            .enumerate()
            .filter(|(i, _)| self.keep[*i])
            .map(|(_, g)| *g)
            .collect()
    }

    /// S1's object as a `Circuit` the branch sources take.
    pub fn reduced_circuit(&self, gates: &[Gate]) -> Circuit {
        Circuit { n_qubits: self.n_qubits, gates: self.drop_removed(gates) }
    }
}

/// The width the search runs at: the widest wire anything names, plus one.
pub fn width(circuit: &[Gate], observable: &Observable) -> usize {
    let from_gates = circuit.iter().flat_map(|g| support(*g)).max().map_or(0, |q| q + 1);
    from_gates.max(observable.min_width())
}

/// LOCATE. Two searches, in this order: the closure test over every gate,
/// then the backward light cone of the observable.
pub fn locate(circuit: &[Gate], observable: &Observable) -> Sector {
    let n = width(circuit, observable);

    // --- search one: where does the tableau view stay Closed? ---
    let mut clifford: Vec<Gate> = Vec::new();
    let mut is_hard = vec![false; circuit.len()];
    for (i, g) in circuit.iter().enumerate() {
        if closure_of(*g).is_closed() {
            clifford.push(*g);
        } else {
            is_hard[i] = true;
        }
    }

    // --- search two: the observable's BACKWARD light cone ---
    let mut cone = vec![false; n];
    for q in observable.support(n) {
        cone[q] = true;
    }
    let mut keep = vec![false; circuit.len()];
    for i in (0..circuit.len()).rev() {
        let s = support(circuit[i]);
        if s.iter().any(|&q| cone[q]) {
            for q in s {
                cone[q] = true;
            }
            keep[i] = true;
        }
    }

    let mut hard = Vec::new();
    let mut removed = Vec::new();
    for (i, g) in circuit.iter().enumerate() {
        if !is_hard[i] {
            continue;
        }
        if keep[i] {
            hard.push((i, *g));
        } else {
            removed.push((i, *g));
        }
    }
    let light_cone: Vec<usize> = (0..n).filter(|&q| cone[q]).collect();
    let t_eff = hard.len();
    Sector { clifford, hard, removed, t_eff, light_cone, keep, n_qubits: n }
}

// ---------------------------------------------------------------------------
// THE REFEREE — the dense 2^n carrier, for n ≤ 20
// ---------------------------------------------------------------------------

/// The exact statevector, the prereg's referee wherever it can exist. It
/// shares nothing with the located sector: no tableau, no branch index, no
/// light cone — `2^n` complex amplitudes and one matrix per gate.
pub mod referee {
    use super::{cmul, Gate, ObsValue, Observable, C};

    /// The engine's dense action of one gate on a `2^n` amplitude array.
    /// This is also the DEFINITION the closure test reads its matrices from.
    pub fn apply_gate(st: &mut [C], n: usize, g: Gate) {
        let dim = 1usize << n;
        debug_assert_eq!(st.len(), dim);
        let s = std::f64::consts::FRAC_1_SQRT_2;
        match g {
            Gate::X(q) => {
                let b = 1usize << q;
                for i in 0..dim {
                    if i & b == 0 {
                        st.swap(i, i | b);
                    }
                }
            }
            Gate::Z(q) => {
                let b = 1usize << q;
                for (i, a) in st.iter_mut().enumerate() {
                    if i & b != 0 {
                        *a = (-a.0, -a.1);
                    }
                }
            }
            Gate::S(q) | Gate::Sdg(q) => {
                let b = 1usize << q;
                let sgn = if matches!(g, Gate::S(_)) { 1.0 } else { -1.0 };
                for (i, a) in st.iter_mut().enumerate() {
                    if i & b != 0 {
                        *a = cmul(*a, (0.0, sgn));
                    }
                }
            }
            Gate::T(q) | Gate::Tdg(q) => {
                let b = 1usize << q;
                let sgn = if matches!(g, Gate::T(_)) { 1.0 } else { -1.0 };
                for (i, a) in st.iter_mut().enumerate() {
                    if i & b != 0 {
                        *a = cmul(*a, (s, sgn * s));
                    }
                }
            }
            Gate::H(q) => {
                let b = 1usize << q;
                for i in 0..dim {
                    if i & b == 0 {
                        let (a, c) = (st[i], st[i | b]);
                        st[i] = ((a.0 + c.0) * s, (a.1 + c.1) * s);
                        st[i | b] = ((a.0 - c.0) * s, (a.1 - c.1) * s);
                    }
                }
            }
            Gate::Cx(c, t) => {
                let (cb, tb) = (1usize << c, 1usize << t);
                for i in 0..dim {
                    if i & cb != 0 && i & tb == 0 {
                        st.swap(i, i | tb);
                    }
                }
            }
        }
    }

    /// `C|0^n⟩`, dense and exact up to f64.
    pub fn statevector(n: usize, gates: &[Gate]) -> Vec<C> {
        let mut st = vec![(0.0f64, 0.0f64); 1usize << n];
        st[0] = (1.0, 0.0);
        for g in gates {
            apply_gate(&mut st, n, *g);
        }
        st
    }

    pub fn index_of(y: &[bool]) -> usize {
        y.iter().enumerate().filter(|(_, &b)| b).map(|(q, _)| 1usize << q).sum()
    }

    pub fn amplitude(sv: &[C], y: &[bool]) -> C {
        sv[index_of(y)]
    }

    pub fn marginal(sv: &[C], qubits: &[usize], bits: &[bool]) -> f64 {
        let mut p = 0.0;
        for (i, a) in sv.iter().enumerate() {
            if qubits.iter().zip(bits).all(|(&q, &b)| (i >> q & 1 == 1) == b) {
                p += a.0 * a.0 + a.1 * a.1;
            }
        }
        p
    }

    /// The observable, read off the referee's carrier.
    pub fn observable(sv: &[C], obs: &Observable) -> ObsValue {
        match obs {
            Observable::Amplitude(y) => ObsValue::Amp(amplitude(sv, y)),
            Observable::Marginal { qubits, bits } => ObsValue::Prob(marginal(sv, qubits, bits)),
        }
    }

    /// The referee's whole job in one call.
    pub fn value(n: usize, gates: &[Gate], obs: &Observable) -> ObsValue {
        observable(&statevector(n, gates), obs)
    }
}

// ---------------------------------------------------------------------------
// THE EXACT LANE — a Cyc's modulus squared, without leaving the integers
// ---------------------------------------------------------------------------

/// `|a|² = (int + sqrt2·√2) / 2^m`, EXACTLY, in integers.
///
/// With `a = c₀ + c₁ω + c₂ω² + c₃ω³` and `ω = e^{iπ/4}`,
/// `|a|² · 2^m = Σcⱼ² + √2·(c₀c₁ − c₀c₃ + c₁c₂ + c₂c₃)`. A single stabilizer
/// branch's amplitude has `|a|² = 2^{−k}`, so the `√2` part comes back `0`
/// there and the probability is a dyadic rational an `f64` carries to the bit
/// — which is what the plants compare against the tableau tier.
pub fn cyc_norm2(a: Cyc) -> (i128, i128, i32) {
    let c = a.c;
    let int = c[0] * c[0] + c[1] * c[1] + c[2] * c[2] + c[3] * c[3];
    let sqrt2 = c[0] * c[1] - c[0] * c[3] + c[1] * c[2] + c[2] * c[3];
    (int, sqrt2, a.m)
}

/// `|a|²` as an `f64`, when it is dyadic — `None` when the `√2` part survives
/// (which no single stabilizer amplitude can produce, so `None` is itself a
/// finding).
pub fn cyc_prob_exact(a: Cyc) -> Option<f64> {
    let (int, sqrt2, m) = cyc_norm2(a);
    if sqrt2 != 0 {
        return None;
    }
    Some(int as f64 * (2.0f64).powi(-m))
}

// ---------------------------------------------------------------------------
// THE HARD-PART SEAM
// ---------------------------------------------------------------------------

/// The hard part's answer, with its certificate. Mirrors the shape
/// `holon::acuity::Budgeted` is being built to (QVM-ACUITY-1's other half):
/// a value, the bound on everything not evaluated, and the two counts the
/// price is checked against.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Budgeted {
    pub value_f64: C,
    pub remainder: f64,
    pub evaluated: u64,
    pub total: u64,
}

/// THE SEAM. `holon::acuity::budgeted_amplitude(src, y, eps, shards)` is
/// being built in parallel with exactly this shape; until it lands in this
/// worktree the driver runs on [`FullSum`], which ignores `eps`, evaluates
/// every branch and certifies a remainder of `0` — the honest fallback, since
/// a sum with nothing left out has nothing left to bound. Swapping the real
/// backend in is one `impl` of this trait.
pub trait AcuityBackend {
    /// What the driver prints in `backend`, so a result can never be read as
    /// the budgeted sum's when it was the full one's.
    fn name(&self) -> &'static str;
    fn budgeted_amplitude(
        &self,
        src: &dyn BranchSource,
        y: &[bool],
        eps: f64,
        shards: usize,
    ) -> Budgeted;
}

/// `&dyn BranchSource` as a `BranchSource`, so the mesh's generic fold takes
/// a trait object. (`BranchSource: Sync`, so this is `Sync` too.)
struct DynSrc<'a>(&'a dyn BranchSource);

impl BranchSource for DynSrc<'_> {
    fn n_branches(&self) -> u64 {
        self.0.n_branches()
    }
    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        self.0.amplitude_of(branch, y)
    }
    fn n_qubits(&self) -> usize {
        self.0.n_qubits()
    }
}

/// THE FULL-SUM FALLBACK — every branch, exact `Z[ω]`, `eps` ignored,
/// remainder `0`. This is also the prereg's second referee (§1: "the FULL
/// exact branch sum, all branches, exact `Z[ω]` arithmetic"), so it is worth
/// having whatever the other half lands.
pub struct FullSum;

impl AcuityBackend for FullSum {
    fn name(&self) -> &'static str {
        "full-sum-fallback"
    }
    fn budgeted_amplitude(
        &self,
        src: &dyn BranchSource,
        y: &[bool],
        _eps: f64,
        shards: usize,
    ) -> Budgeted {
        let n = src.n_branches();
        let exact = crate::mesh::fold_amplitude(&DynSrc(src), y, shards);
        Budgeted { value_f64: exact.to_complex(), remainder: 0.0, evaluated: n, total: n }
    }
}

/// The exact `Z[ω]` full sum, for the callers that want the ring value rather
/// than its float image (the plants compare probabilities to the bit).
pub fn full_sum_exact(src: &dyn BranchSource, y: &[bool], shards: usize) -> Cyc {
    crate::mesh::fold_amplitude(&DynSrc(src), y, shards)
}

// ---------------------------------------------------------------------------
// THE INSTANCES — the prereg's grid, generated once and generated here
// ---------------------------------------------------------------------------

/// splitmix64 — three lines, no dependency, and the same stream on every
/// machine, which is what makes a seed in the results table mean something.
pub struct Rng(pub u64);

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// The prereg's instance: `20n` Clifford gates drawn uniformly from
/// `{x, z, h, s, sdg, cx}` on uniform wires, with `t` `T`/`T†` gates spliced
/// in at `t` distinct random positions among them
/// (`conformance/qasm/battlerig.py::gen_fixed_t`, the same construction,
/// ported so the Rust side and the sweep agree on what an instance IS).
pub fn random_instance(n: usize, t: usize, seed: u64) -> Circuit {
    random_instance_depth(n, t, seed, 20 * n)
}

/// The same family at a DECLARED Clifford depth. `20n` is the prereg's; the
/// shallower depths exist because of what the deep ones turned out to be —
/// see `tests/qvm_acuity_cheap.rs`, where a 4-qubit marginal of a `20n`-deep
/// random circuit is measured to be exactly `1/16` (maximally mixed), and so
/// cannot move when anything is dropped.
pub fn random_instance_depth(n: usize, t: usize, seed: u64, depth: usize) -> Circuit {
    assert!(n >= 2, "the instance family needs at least two wires for cx");
    let mut rng = Rng(seed ^ 0x5165_7561_6369_7479);
    let mut gates: Vec<Gate> = Vec::with_capacity(depth + t);
    for _ in 0..depth {
        let q = rng.below(n);
        let g = match rng.below(6) {
            0 => Gate::X(q),
            1 => Gate::Z(q),
            2 => Gate::H(q),
            3 => Gate::S(q),
            4 => Gate::Sdg(q),
            _ => {
                let mut r = rng.below(n);
                while r == q {
                    r = rng.below(n);
                }
                Gate::Cx(q, r)
            }
        };
        gates.push(g);
    }
    // t DISTINCT insertion positions, inserted from the back so the earlier
    // ones keep their meaning.
    let mut pos: Vec<usize> = Vec::with_capacity(t);
    while pos.len() < t {
        let p = rng.below(gates.len() + 1);
        if !pos.contains(&p) {
            pos.push(p);
        }
    }
    pos.sort_unstable();
    for p in pos.into_iter().rev() {
        let q = rng.below(n);
        let g = if rng.below(2) == 0 { Gate::T(q) } else { Gate::Tdg(q) };
        gates.insert(p, g);
    }
    Circuit { n_qubits: n, gates }
}

/// A DECLARED `y` that is not vacuous: the bitstring the engine's tableau
/// tier samples from the circuit's CLIFFORD SKELETON (every gate the closure
/// test found cheap, applied to a `PackedTableau`; random outcomes collapsed
/// to `0`).
///
/// The rule matters. `y = 0^n` looks like the neutral declaration and is not:
/// a random Clifford circuit's state is supported on a COSET, so `⟨0^n|C|0⟩`
/// is exactly zero for most instances and every comparison against it is
/// `0 = 0` — M-VACUOUS-SUCCESS with a clean green tick. This rule declares
/// `y` from the circuit alone (no referee, no branch sum, poly time) and puts
/// it inside the skeleton's support, where the amplitude has somewhere to be.
pub fn skeleton_bitstring(circuit: &[Gate], n: usize) -> Vec<bool> {
    let mut tab = crate::tableau::PackedTableau::new(n);
    for g in circuit {
        if !closure_of(*g).is_closed() {
            continue;
        }
        match *g {
            Gate::H(q) => tab.h(q),
            Gate::S(q) => tab.s(q),
            Gate::Sdg(q) => tab.sdg(q),
            Gate::X(q) => tab.x_gate(q),
            Gate::Z(q) => tab.z_gate(q),
            Gate::Cx(c, t) => tab.cx(c, t),
            Gate::T(_) | Gate::Tdg(_) => unreachable!("the closure test passed a rotation"),
        }
    }
    tab.sample_all()
}

/// The other DECLARED `y`, for the instances where a referee exists: the
/// bitstring of largest `|⟨y|C|0⟩|`, ties broken by the smallest index.
///
/// Measured on the prereg's own grid: [`skeleton_bitstring`] is cheap and
/// poly-time but it does NOT guarantee a nonzero amplitude — on
/// `random_instance(12, 8, 1)` the skeleton's sample is `0^12` and the full
/// circuit's amplitude there is EXACTLY zero (the branches cancel in the
/// ring). A stake settled at `0 = 0` is M-VACUOUS-SUCCESS, so every instance
/// with a referee declares `y` by this rule instead, and the rule is stated
/// before the run rather than chosen after it.
pub fn argmax_bitstring(n: usize, gates: &[Gate]) -> Vec<bool> {
    let sv = referee::statevector(n, gates);
    let mut best = 0usize;
    let mut best_p = -1.0f64;
    for (i, a) in sv.iter().enumerate() {
        let p = a.0 * a.0 + a.1 * a.1;
        if p > best_p + 1e-18 {
            best_p = p;
            best = i;
        }
    }
    (0..n).map(|q| best >> q & 1 == 1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_closure_test_finds_the_clifford_alphabet_closed() {
        for g in [
            Gate::X(0),
            Gate::Z(0),
            Gate::H(0),
            Gate::S(0),
            Gate::Sdg(0),
            Gate::Cx(0, 1),
            Gate::Cx(1, 0),
        ] {
            let c = closure_of(g);
            assert!(c.is_closed(), "{g:?} should be cheap: {}", c.witness());
        }
    }

    #[test]
    fn the_closure_test_finds_the_rotation_open() {
        for g in [Gate::T(0), Gate::Tdg(0)] {
            match closure_of(g) {
                Closure::Split { terms, weights, generator } => {
                    assert_eq!(generator, "X_0", "T splits X first");
                    assert_eq!(terms, 2, "T splits X into exactly two Paulis");
                    for w in weights {
                        assert!((w - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12);
                    }
                }
                c => panic!("{g:?} must not be closed: {c:?}"),
            }
        }
    }

    #[test]
    fn the_light_cone_of_an_amplitude_is_everything() {
        let c = random_instance(6, 4, 7);
        let s = locate(&c.gates, &Observable::Amplitude(vec![false; 6]));
        assert_eq!(s.removed.len(), 0);
        assert_eq!(s.light_cone, (0..6).collect::<Vec<_>>());
        assert_eq!(s.t_eff, 4);
    }

    #[test]
    fn the_price_is_the_registers() {
        let c = random_instance(6, 8, 11);
        let s = locate(&c.gates, &Observable::Amplitude(vec![false; 6]));
        assert_eq!(s.t_eff, 8);
        assert_eq!(s.price(), crate::magic5::expected_branches(8));
        assert_eq!(s.clifford.len(), c.gates.len() - 8);
    }
}
