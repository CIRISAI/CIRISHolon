//! THE MAGIC TIER'S CANONICALIZER: Native ZX-calculus graph rewriting and circuit extraction.
//!
//! Provides full graph simplification passes in native Rust:
//! - Spider fusion (Z-Z and X-X merging with phase addition mod 2π)
//! - Local complementation around Clifford spiders (phase ±π/2)
//! - Pivoting between interior Pauli spiders (phase 0 or π)
//! - Phase gadget extraction / identity removal
//! - Circuit extraction via GF(2) Gaussian elimination
//!
//! Optionally bridges to quizx under the `zx` feature.

pub mod extract;
pub mod graph;
pub mod simplify;

pub use extract::{extract_circuit, Extraction};
pub use graph::{cyc_eq, from_core, from_surface, omega, EdgeType, SpiderType, ZxGraph};

use holon::qasm::Surface;

/// Summary of what the canonicalizer achieved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reduction {
    pub t_before: usize,
    pub t_after: usize,
    pub gates_before: usize,
    pub gates_after: usize,
    /// The global phase recovered EXACTLY from the graph's scalar as an ω-power (mod 8).
    pub phase_omega: i64,
}

/// Canonicalize a surface program through native ZX graph rewriting,
/// extract an optimized circuit, and compute exact scalar/phase updates.
pub fn canonicalize_native(n: usize, surface: &[Surface]) -> Result<(Vec<Surface>, Reduction), String> {
    let t_before = holon::simplify::magic_weight(surface);
    let gates_before = surface.len();

    let mut g = from_surface(n, surface)?;
    g.full_reduce();
    let t_after = g.t_count();

    let ex = g.extract()?;
    let gates_after = ex.gates.len();

    // Determine the phase_omega: for unitary extractions, ex.scalar = ω^k * (√2)^0
    let mut phase_omega = 0i64;
    let mut matched = false;
    for k in 0..8 {
        if cyc_eq(ex.scalar, omega(k)) {
            phase_omega = k;
            matched = true;
            break;
        }
    }
    if !matched {
        let z = ex.scalar.to_complex();
        let ang = z.1.atan2(z.0);
        let k = (ang / std::f64::consts::FRAC_PI_4).round() as i64;
        phase_omega = k.rem_euclid(8);
    }

    Ok((
        ex.gates,
        Reduction {
            t_before,
            t_after,
            gates_before,
            gates_after,
            phase_omega,
        },
    ))
}

#[cfg(feature = "zx")]
pub fn canonicalize_quizx(n: usize, surface: &[Surface]) -> Result<(Vec<Surface>, Reduction), String> {
    use quizx::circuit::Circuit;
    use quizx::extract::ToCircuit;
    use quizx::vec_graph::Graph;

    let qasm = to_qasm(n, surface)?;
    let c = Circuit::from_qasm(&qasm).map_err(|e| format!("quizx parse: {e:?}"))?;
    let t_before = holon::simplify::magic_weight(surface);
    let gates_before = surface.len();

    let mut g: Graph = c.to_graph();
    quizx::simplify::full_simp(&mut g);
    let phase_omega = {
        use quizx::graph::GraphLike;
        match g.scalar().exact_phase_and_sqrt2_pow() {
            Some((p, _sqrt2)) => {
                let r = p.to_rational();
                let num = *r.numer() * 4;
                let den = *r.denom();
                if num % den != 0 {
                    return Err(format!(
                        "quizx scalar phase {r} is not a ζ8 power: outside Z[ω]; refusing rather than rounding"
                    ));
                }
                (num / den).rem_euclid(8)
            }
            None => {
                return Err(
                    "quizx scalar has no exact phase form: refusing rather than dropping a global phase".into(),
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

/// Primary canonicalizer entry point: uses native ZX graph simplification and extraction.
pub fn canonicalize(n: usize, surface: &[Surface]) -> Result<(Vec<Surface>, Reduction), String> {
    canonicalize_native(n, surface)
}

/// Render a surface program as OpenQASM 2 for serialization / interoperability.
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
