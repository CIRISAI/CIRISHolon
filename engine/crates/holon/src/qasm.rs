//! THE FRONT-END: an OpenQASM-superset surface, lowered onto the certified
//! eight-gate core {X, Z, S, S†, H, CX, T, T†} — holonically.
//!
//! Three commitments, each enforced by structure rather than discipline:
//!
//! 1. **Lowerings are TRANSPORTS, and rules are DATA.** Surface→core is a
//!    claim-transport square (Transport.lean's shape); each rule is a
//!    declarative rewrite `surface gate ↦ word of surface gates × exact
//!    scalar`, applied by ONE recursive rewriter until only core remains.
//!    The certificate of each square is a per-rule conformance test against
//!    an independent dense exact-ring oracle (`tests/qasm_oracle.rs`) —
//!    rule-level, so a program-level test failure can never hide which
//!    square broke.
//! 2. **Recursion is native.** ccz lowers to H·CCX·H where CCX is itself a
//!    surface gate; the rewriter recurses. Rules compose; nothing inlines.
//! 3. **Scalars are LEDGER, not circuit.** The global phase accumulates as
//!    an exact ζ16 exponent on the Program — first-class data the amplitude
//!    consumer multiplies back (ω-part exactly in-ring; the odd ζ16
//!    residual is declared, outside Z[ζ8], never silently dropped).
//!
//! Refusals (a refusal is a result): rz/p at non-π/4 multiples — the exact
//! ring cannot carry e^{iθ} for generic θ; named routes: a native
//! basis-change campaign (CAMPAIGNS.md #2) or Ross–Selinger synthesis under
//! an accuracy-degrading Policy. reset / mid-circuit measurement / classical
//! control — the adaptive rung of the TIERS front-end debt, named.

use crate::affine::Gate;

/// The SURFACE alphabet: the superset quizx and qiskit/OpenQASM circuits
/// actually use. Core gates are surface gates too (the identity transport).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Surface {
    // core (lower to themselves)
    X(usize),
    Z(usize),
    S(usize),
    Sdg(usize),
    H(usize),
    Cx(usize, usize),
    T(usize),
    Tdg(usize),
    // superset
    Y(usize),
    Sx(usize),
    Sxdg(usize),
    Cz(usize, usize),
    Swap(usize, usize),
    Ccx(usize, usize, usize),
    Ccz(usize, usize, usize),
    /// diag(1, ω^k) — p/u1 at a π/4 multiple, no scalar.
    DiagPow(i64, usize),
    /// rz(kπ/4) = ζ16^{−k} · diag(1, ω^k) — carries its scalar.
    RzPow(i64, usize),
    /// rz(±θ_F), θ_F = arccos(1/√3): EXACT in Z[ω][√3] (`face.rs`), not in
    /// Z[ω]. Kept as a surface gate — it is not lowerable to the core
    /// alphabet, so it is a REFUSAL for core consumers and a first-class
    /// gate for the face engine. Sign is the rotation's sign.
    Face(i8, usize),
    /// A GENERIC diagonal rotation `diag(1, z)` with `z = e^{iθ}` left
    /// SYMBOLIC — the angle never enters the exact computation. Consumed by
    /// `face::amplitude_poly`, which returns the amplitude as an exact
    /// polynomial in z; the numeric angle is applied last, or never.
    Rot(usize),
}

/// One rewrite step: a surface gate is either CORE (transport ends) or a
/// WORD of surface gates times an exact ζ16 scalar (transport recurses).
/// This function IS the rule table; adding a gate is adding a row, and its
/// certificate is one oracle test.
pub fn rule(g: Surface) -> Result<Gate, (Vec<Surface>, i64)> {
    use Surface::*;
    match g {
        X(q) => Ok(Gate::X(q)),
        Z(q) => Ok(Gate::Z(q)),
        S(q) => Ok(Gate::S(q)),
        Sdg(q) => Ok(Gate::Sdg(q)),
        H(q) => Ok(Gate::H(q)),
        Cx(a, b) => Ok(Gate::Cx(a, b)),
        T(q) => Ok(Gate::T(q)),
        Tdg(q) => Ok(Gate::Tdg(q)),

        Y(q) => Err((vec![Z(q), X(q)], 4)), // Y = i·X·Z, i = ζ16⁴
        Sx(q) => Err((vec![H(q), S(q), H(q)], 0)),
        Sxdg(q) => Err((vec![H(q), Sdg(q), H(q)], 0)),
        Cz(a, b) => Err((vec![H(b), Cx(a, b), H(b)], 0)),
        Swap(a, b) => Err((vec![Cx(a, b), Cx(b, a), Cx(a, b)], 0)),
        // the standard 7-T Toffoli network
        Ccx(a, b, c) => Err((
            vec![
                H(c),
                Cx(b, c),
                Tdg(c),
                Cx(a, c),
                T(c),
                Cx(b, c),
                Tdg(c),
                Cx(a, c),
                T(b),
                T(c),
                H(c),
                Cx(a, b),
                T(a),
                Tdg(b),
                Cx(a, b),
            ],
            0,
        )),
        // native recursion: ccz is DEFINED through the surface ccx
        Ccz(a, b, c) => Err((vec![H(c), Ccx(a, b, c), H(c)], 0)),
        DiagPow(k, q) => Err((diag_word(k, q), 0)),
        RzPow(k, q) => Err((vec![DiagPow(k, q)], -k)),
        // Not lowerable: the face phase lives one quadratic extension out,
        // and a generic rotation lives outside every ring. `lower` refuses
        // both; the face/poly engines consume them directly.
        Face(..) | Rot(_) => Err((vec![], 0)),
    }
}

fn diag_word(k: i64, q: usize) -> Vec<Surface> {
    use Surface::*;
    match k.rem_euclid(8) {
        0 => vec![],
        1 => vec![T(q)],
        2 => vec![S(q)],
        3 => vec![S(q), T(q)],
        4 => vec![Z(q)],
        5 => vec![Z(q), T(q)],
        6 => vec![Sdg(q)],
        7 => vec![Tdg(q)],
        _ => unreachable!(),
    }
}

/// THE ONE REWRITER: recurse every surface gate through `rule` until core,
/// accumulating the exact scalar in the ledger slot. Termination: every
/// rule's word is strictly closer to core (checked by the depth fence).
pub fn lower(surface: &[Surface]) -> (Vec<Gate>, i64) {
    let mut core = Vec::new();
    let mut phase_16: i64 = 0;
    fn go(g: Surface, core: &mut Vec<Gate>, phase: &mut i64, depth: u32) {
        assert!(depth < 8, "lowering must terminate: rule cycle detected");
        assert!(
            !matches!(g, Surface::Face(..) | Surface::Rot(_)),
            "Face/Rot gates are not lowerable to the core alphabet — route \
             the program to the face engine (face.rs: amplitude_face for the \
             √3 ring, amplitude_poly for symbolic generic angles)"
        );
        match rule(g) {
            Ok(cg) => core.push(cg),
            Err((word, p)) => {
                *phase += p;
                for w in word {
                    go(w, core, phase, depth + 1);
                }
            }
        }
    }
    for &g in surface {
        go(g, &mut core, &mut phase_16, 0);
    }
    (core, phase_16)
}

/// A parsed program: lowered core gates plus the LEDGER scalar and the
/// declared measurements. `phase_omega` is the exactly-foldable part of the
/// global phase (a power of ω, in-ring); `residual_zeta16` (0 or 1) is the
/// part Z[ζ8] cannot carry — declared, never dropped; probabilities are
/// unaffected either way.
#[derive(Clone, Debug)]
pub struct Program {
    pub n_qubits: usize,
    pub gates: Vec<Gate>,
    pub measured: Vec<usize>,
    pub phase_omega: u8,
    pub residual_zeta16: u8,
}

#[derive(Debug, PartialEq)]
pub struct Refusal {
    pub line: usize,
    pub reason: String,
}

fn refuse(line: usize, reason: impl Into<String>) -> Refusal {
    Refusal { line, reason: reason.into() }
}

/// θ as an exact π/4 multiple. The check is on the parsed float; the
/// lowering then uses the exact integer, so no approximation enters the
/// value path. A near-miss beyond 4 ULP refuses.
fn pi4_multiple(theta: f64) -> Option<i64> {
    let k = theta / std::f64::consts::FRAC_PI_4;
    let kr = k.round();
    if (k - kr).abs() < 4.0 * f64::EPSILON * kr.abs().max(1.0) {
        Some(kr as i64)
    } else {
        None
    }
}

/// θ = ±arccos(1/√3) — the face angle, recognized EXACTLY (the value is
/// then used symbolically; the float only classifies).
fn face_angle(theta: f64) -> Option<i8> {
    const TF: f64 = 0.955_316_618_124_509_2;
    if (theta - TF).abs() < 1e-12 {
        Some(1)
    } else if (theta + TF).abs() < 1e-12 {
        Some(-1)
    } else {
        None
    }
}

fn angle(line: usize, p: Option<&str>) -> Result<f64, Refusal> {
    let p = p.ok_or_else(|| refuse(line, "missing parameter"))?;
    let s = p.replace("pi", &std::f64::consts::PI.to_string());
    let num = |x: &str| x.trim().parse::<f64>().map_err(|_| refuse(line, "bad angle"));
    if let Some((x, y)) = s.split_once('/') {
        Ok(num(x)? / num(y)?)
    } else if let Some((x, y)) = s.split_once('*') {
        Ok(num(x)? * num(y)?)
    } else {
        num(&s)
    }
}

/// Text → surface alphabet. Parsing and lowering are SEPARATE transports:
/// adapters (quizx, qiskit) may construct `Vec<Surface>` directly and skip
/// the text entirely.
pub fn parse_surface(src: &str) -> Result<(usize, Vec<Surface>, Vec<usize>), Refusal> {
    let mut n = 0usize;
    let mut out = Vec::new();
    let mut measured = Vec::new();
    for (ln, raw) in src.lines().enumerate() {
        let line = ln + 1;
        let stmt = raw.split("//").next().unwrap_or("").trim();
        if stmt.is_empty()
            || stmt.starts_with("OPENQASM")
            || stmt.starts_with("include")
            || stmt.starts_with("creg")
            || stmt.starts_with("barrier")
        {
            continue;
        }
        let stmt = stmt.trim_end_matches(';');
        if let Some(rest) = stmt.strip_prefix("qreg") {
            n += rest
                .split(['[', ']'])
                .nth(1)
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or_else(|| refuse(line, "unparsable qreg"))?;
            continue;
        }
        if stmt.starts_with("measure") {
            if let Some(q) = stmt.split(['[', ']']).nth(1).and_then(|s| s.parse().ok()) {
                measured.push(q);
            }
            continue;
        }
        let (head, args) = stmt.split_once(' ').ok_or_else(|| refuse(line, "bare token"))?;
        let qs: Vec<usize> = args
            .split(',')
            .filter_map(|a| a.split(['[', ']']).nth(1))
            .filter_map(|s| s.parse().ok())
            .collect();
        let (name, param) = match head.split_once('(') {
            Some((g, p)) => (g, Some(p.trim_end_matches(')'))),
            None => (head, None),
        };
        use Surface::*;
        let g = match (name, qs.as_slice()) {
            ("x", [q]) => X(*q),
            ("y", [q]) => Y(*q),
            ("z", [q]) => Z(*q),
            ("h", [q]) => H(*q),
            ("s", [q]) => S(*q),
            ("sdg", [q]) => Sdg(*q),
            ("t", [q]) => T(*q),
            ("tdg", [q]) => Tdg(*q),
            ("sx", [q]) => Sx(*q),
            ("sxdg", [q]) => Sxdg(*q),
            ("cx" | "CX", [a, b]) => Cx(*a, *b),
            ("cz", [a, b]) => Cz(*a, *b),
            ("swap", [a, b]) => Swap(*a, *b),
            ("ccx", [a, b, c]) => Ccx(*a, *b, *c),
            ("ccz", [a, b, c]) => Ccz(*a, *b, *c),
            ("id" | "u0", _) => continue,
            ("rz", [q]) => match pi4_multiple(angle(line, param)?) {
                Some(k) => RzPow(k, *q),
                None if face_angle(angle(line, param)?).is_some() => {
                    Face(face_angle(angle(line, param)?).unwrap(), *q)
                }
                // A generic angle is not refused any more: it becomes a
                // SYMBOLIC rotation, and `amplitude_poly` returns the exact
                // polynomial in z = e^{iθ}. The angle is applied last.
                None => Rot(*q),
            },
            ("p" | "u1", [q]) => match pi4_multiple(angle(line, param)?) {
                Some(k) => DiagPow(k, *q),
                None => return Err(refuse(line, "u1 at a non-π/4 multiple; same routes as rz")),
            },
            ("reset", _) => {
                return Err(refuse(
                    line,
                    "reset: the adaptive rung (TIERS front-end debt), named, not smuggled",
                ))
            }
            _ => {
                return Err(refuse(
                    line,
                    format!("unsupported gate '{name}' with {} qubit(s)", qs.len()),
                ))
            }
        };
        out.push(g);
    }
    Ok((n, out, measured))
}

/// Text → lowered Program: parse, then the one rewriter, scalar to ledger.
pub fn parse(src: &str) -> Result<Program, Refusal> {
    let (n_qubits, surface, measured) = parse_surface(src)?;
    let faces = surface.iter().filter(|g| matches!(g, Surface::Face(..))).count();
    let rots = surface.iter().filter(|g| matches!(g, Surface::Rot(_))).count();
    if faces + rots > 0 {
        return Err(refuse(
            0,
            format!(
                "program carries {faces} face rotation(s) and {rots} generic rotation(s): not \
                 lowerable to the core alphabet, but BOTH are carried exactly elsewhere — the \
                 face engine (face::amplitude_face, exact in Z[ω][√3]) and the symbolic \
                 carrier (face::amplitude_poly, exact as a polynomial in z = e^{{iθ}}, angle \
                 applied last). Route via `parse_surface`; core consumers refuse by design"
            ),
        ));
    }
    let (gates, phase_16) = lower(&surface);
    let p16 = phase_16.rem_euclid(16);
    Ok(Program {
        n_qubits,
        gates,
        measured,
        phase_omega: (p16 / 2) as u8,
        residual_zeta16: (p16 % 2) as u8,
    })
}
