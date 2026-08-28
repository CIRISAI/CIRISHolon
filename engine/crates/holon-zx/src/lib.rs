//! THE MAGIC TIER'S CANONICALIZER, COMPOSED.
//!
//! BENCHMARKS entries 13–19 measured the same structural fact from six
//! directions: the Clifford tier has its canonical form (the tableau) and
//! beats stim at every n; the magic tier lacks its own (the ZX graph) and
//! loses to quizx by up to four orders. Entry 17 recorded that our own ZX
//! build got the Clifford half working and the gadget half not — the gap
//! located precisely, at gadgetization.
//!
//! This crate closes that gap by composition rather than by a second
//! attempt: **quizx canonicalizes, holon evaluates exactly.** The division
//! is principled, not expedient — quizx is Clifford+T-only and has no face
//! ring, no symbolic angle, no qudits, no distributed certificates, so it
//! is the canonicalizer and nothing more, while everything downstream of a
//! simplified circuit stays ours.
//!
//! LICENSE: quizx is Apache-2.0, one-way compatible with this project's
//! AGPL-3.0. Credit: Kissinger and van de Wetering, and the graph-theoretic
//! simplification of Duncan–Kissinger–Perdrix–van de Wetering (Quantum 4,
//! 279 (2020)).
//!
//! WHY A SEPARATE CRATE: `holon` has zero external dependencies and ships
//! as a 65 KB wasm module; quizx brings thirteen and 390 KB. The core stays
//! small, and callers who want the canonicalizer opt in here.

use holon::qasm::Surface;

/// What the canonicalizer did — reported, never assumed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reduction {
    pub t_before: usize,
    pub t_after: usize,
    pub gates_before: usize,
    pub gates_after: usize,
    /// The global phase quizx's extraction drops, recovered EXACTLY from
    /// the graph's own scalar, as an ω-power (ζ8 exponent, mod 8). Measured
    /// necessity, not caution: extraction preserves the state up to a
    /// global phase, so without this the composed pipeline preserves
    /// PROBABILITIES and not AMPLITUDES — and amplitudes are the product.
    pub phase_omega: i64,
}

/// Canonicalize a surface program through quizx's graph rewriting, then
/// extract a circuit and return it in OUR alphabet.
///
/// Contract: the returned program is EXACTLY equivalent (ZX rewrites are
/// identities). Verified downstream by `tests/composed.rs`, which requires
/// amplitude equality on every basis state — the same gate our own passes
/// carry, applied to a third-party simplifier.
#[cfg(feature = "zx")]
pub fn canonicalize(n: usize, surface: &[Surface]) -> Result<(Vec<Surface>, Reduction), String> {
    use quizx::circuit::Circuit;
    use quizx::extract::ToCircuit;
    use quizx::vec_graph::Graph;

    let qasm = to_qasm(n, surface)?;
    let c = Circuit::from_qasm(&qasm).map_err(|e| format!("quizx parse: {e:?}"))?;
    let t_before = holon::simplify::magic_weight(surface);
    let gates_before = surface.len();

    let mut g: Graph = c.to_graph();
    quizx::simplify::full_simp(&mut g);
    // Capture the graph's scalar BEFORE extraction: `to_circuit` returns the
    // state up to this phase, and it is available exactly (a ζ8 power plus a
    // √2 power) rather than as a float.
    let phase_omega = {
        use quizx::graph::GraphLike;
        match g.scalar().exact_phase_and_sqrt2_pow() {
            Some((p, _sqrt2)) => {
                // Phase is a rational multiple of π; ω = e^{iπ/4}, so the
                // ω-exponent is 4× that rational, and it must be an integer
                // for the value to live in our ring.
                let r = p.to_rational();
                let num = *r.numer() * 4;
                let den = *r.denom();
                if num % den != 0 {
                    return Err(format!(
                        "quizx scalar phase {r} is not a ζ8 power: outside Z[ω];                          refusing rather than rounding"
                    ));
                }
                (num / den).rem_euclid(8)
            }
            None => {
                return Err(
                    "quizx scalar has no exact phase form: refusing rather than                      dropping a global phase the amplitude depends on"
                        .into(),
                )
            }
        }
    };
    let out = g
        .to_circuit()
        .map_err(|e| format!("quizx extraction failed: {e:?}"))?;

    let (n2, simplified, _) = holon::qasm::parse_surface(&out.to_qasm())
        .map_err(|e| format!("re-parse of the extracted circuit: {}", e.reason))?;
    if n2 != n {
        return Err(format!("extraction changed the qubit count: {n} -> {n2}"));
    }
    let t_after = holon::simplify::magic_weight(&simplified);
    Ok((
        simplified,
        Reduction {
            t_before,
            t_after,
            gates_before,
            gates_after: out.num_gates(),
            phase_omega,
        },
    ))
}

/// Without the `zx` feature this is an explicit no-op: the program is
/// returned unchanged and the reduction reports zero. Downstream code
/// compiles and runs either way — it just does not get the canonicalizer.
#[cfg(not(feature = "zx"))]
pub fn canonicalize(_n: usize, surface: &[Surface]) -> Result<(Vec<Surface>, Reduction), String> {
    let t = holon::simplify::magic_weight(surface);
    Ok((
        surface.to_vec(),
        Reduction {
            t_before: t,
            t_after: t,
            gates_before: surface.len(),
            gates_after: surface.len(),
            phase_omega: 0,
        },
    ))
}

/// Render a surface program as OpenQASM 2 for the bridge. Only the
/// Clifford+T fragment crosses — face and symbolic rotations stay on our
/// side, where quizx has no representation for them anyway.
pub fn to_qasm(n: usize, surface: &[Surface]) -> Result<String, String> {
    let mut s = format!("OPENQASM 2.0;\ninclude \"qelib1.inc\";\nqreg q[{n}];\n");
    for g in surface {
        let line = match *g {
            Surface::X(q) => format!("x q[{q}];\n"),
            Surface::Z(q) => format!("z q[{q}];\n"),
            Surface::H(q) => format!("h q[{q}];\n"),
            Surface::S(q) => format!("s q[{q}];\n"),
            Surface::Sdg(q) => format!("sdg q[{q}];\n"),
            Surface::T(q) => format!("t q[{q}];\n"),
            Surface::Tdg(q) => format!("tdg q[{q}];\n"),
            Surface::Cx(a, b) => format!("cx q[{a}], q[{b}];\n"),
            Surface::Cz(a, b) => format!("cz q[{a}], q[{b}];\n"),
            Surface::Ccz(a, b, c) => format!("ccz q[{a}], q[{b}], q[{c}];\n"),
            Surface::Ccx(a, b, c) => format!("ccx q[{a}], q[{b}], q[{c}];\n"),
            Surface::Swap(a, b) => format!("swap q[{a}], q[{b}];\n"),
            Surface::Sx(q) => format!("sx q[{q}];\n"),
            Surface::Sxdg(q) => format!("sxdg q[{q}];\n"),
            Surface::Y(q) => format!("y q[{q}];\n"),
            Surface::DiagPow(k, q) | Surface::RzPow(k, q) => {
                format!("rz({}*pi/4) q[{q}];\n", k)
            }
            Surface::Face(..) | Surface::Rot(_) => {
                return Err(
                    "face/generic rotations do not cross to quizx: it is Clifford+T only. \
                     Keep them on our side (face::amplitude_face / amplitude_poly)"
                        .into(),
                )
            }
        };
        s.push_str(&line);
    }
    Ok(s)
}
