//! SIMPLIFICATION — the pass the QuiZX head-to-head forced (BENCHMARKS
//! entry thirteen: they beat us 15–8000× because their term count was ONE,
//! their ZX rewriting having deleted the branch sum we were racing to
//! compute faster).
//!
//! Three exact rewrites, in the order they compose, each provable in one
//! line and each applied at the SURFACE level so every downstream tier
//! (core, face, symbolic, mesh) inherits the reduction:
//!
//! 1. **Diagonal-run cancellation.** All diagonal gates commute with each
//!    other, so within a maximal run of them the multiset is all that
//!    matters: `Z·Z = I`, `CZ·CZ = I`, `CCZ·CCZ = I`, and the per-qubit
//!    phase powers (S, T, DiagPow) add mod 8. On the hidden-shift family
//!    that is the whole game — 981 Z gates on 12 qubits collapse to at most
//!    12, 1036 CZ to at most C(12,2).
//! 2. **Involution cancellation.** `H·H = I` and `X·X = I` adjacently.
//! 3. **Magic cancellation, which is the one that matters for cost**: a
//!    CCZ pair inside a diagonal run cancels, removing 14 T-equivalents;
//!    `T·T† = I`; `Face(+1,q)·Face(−1,q) = I`. **This is the only rewrite
//!    that lowers the exponent**, and it is why simplification is a
//!    prerequisite rather than an optimization — every exponent gain the
//!    engine has banked multiplies against the reduced t, not the original.
//!
//! Not implemented, and named rather than implied: full ZX rewriting
//! (spider fusion, local complementation, pivoting) and phase teleportation
//! (Kissinger–van de Wetering, arXiv:1903.10477). Those reach reductions
//! these local rules cannot; this pass is the provable floor, not the
//! ceiling.
//!
//! Correctness is not argued, it is gated: `tests/simplify.rs` requires
//! amplitude equality on every basis state before and after, and the
//! head-to-head reruns on simplified input.

use crate::qasm::Surface;
use std::collections::BTreeMap;

/// Is this gate diagonal in the computational basis? Diagonal gates commute
/// with one another — the fact rewrite 1 rests on.
fn is_diagonal(g: &Surface) -> bool {
    matches!(
        g,
        Surface::Z(_)
            | Surface::S(_)
            | Surface::Sdg(_)
            | Surface::T(_)
            | Surface::Tdg(_)
            | Surface::Cz(_, _)
            | Surface::Ccz(_, _, _)
            | Surface::DiagPow(_, _)
            | Surface::RzPow(_, _)
            | Surface::Face(_, _)
            | Surface::Rot(_)
    )
}

/// Cancel and combine within one maximal diagonal run. Exact.
fn reduce_diagonal_run(run: &[Surface]) -> Vec<Surface> {
    // per-qubit phase power mod 8 (ω units: T=1, S=2, Z=4, S†=6, T†=7)
    let mut phase: BTreeMap<usize, i64> = BTreeMap::new();
    // CZ parity per unordered pair, CCZ parity per unordered triple
    let mut cz: BTreeMap<(usize, usize), bool> = BTreeMap::new();
    let mut ccz: BTreeMap<(usize, usize, usize), bool> = BTreeMap::new();
    // symbolic rotations: net signed count per qubit (Face) and raw count (Rot)
    let mut face: BTreeMap<usize, i64> = BTreeMap::new();
    let mut rot: BTreeMap<usize, usize> = BTreeMap::new();
    // rz's ζ16 scalar is carried by RzPow; keep them as DiagPow + a scalar
    // list so the ledger is unchanged (RzPow's phase is handled by `lower`).
    let mut rz_scalars: Vec<(i64, usize)> = Vec::new();

    for g in run {
        match *g {
            Surface::Z(q) => *phase.entry(q).or_insert(0) += 4,
            Surface::S(q) => *phase.entry(q).or_insert(0) += 2,
            Surface::Sdg(q) => *phase.entry(q).or_insert(0) += 6,
            Surface::T(q) => *phase.entry(q).or_insert(0) += 1,
            Surface::Tdg(q) => *phase.entry(q).or_insert(0) += 7,
            Surface::DiagPow(k, q) => *phase.entry(q).or_insert(0) += k,
            Surface::RzPow(k, q) => {
                *phase.entry(q).or_insert(0) += k;
                rz_scalars.push((k, q));
            }
            Surface::Cz(a, b) => {
                let key = if a <= b { (a, b) } else { (b, a) };
                let e = cz.entry(key).or_insert(false);
                *e = !*e;
            }
            Surface::Ccz(a, b, c) => {
                let mut v = [a, b, c];
                v.sort_unstable();
                let e = ccz.entry((v[0], v[1], v[2])).or_insert(false);
                *e = !*e;
            }
            Surface::Face(s, q) => *face.entry(q).or_insert(0) += s as i64,
            Surface::Rot(q) => *rot.entry(q).or_insert(0) += 1,
            _ => unreachable!("non-diagonal gate in a diagonal run"),
        }
    }

    let mut out = Vec::new();
    // RzPow scalars must survive exactly: re-emit them as RzPow with their
    // own k, and subtract that k from the accumulated phase so the DIAGONAL
    // action is not double-counted. (The scalar is the only reason RzPow is
    // not just DiagPow.)
    for (k, q) in &rz_scalars {
        *phase.entry(*q).or_insert(0) -= k;
        out.push(Surface::RzPow(*k, *q));
    }
    for (q, k) in phase {
        let k = k.rem_euclid(8);
        if k != 0 {
            out.push(Surface::DiagPow(k, q));
        }
    }
    for ((a, b), on) in cz {
        if on {
            out.push(Surface::Cz(a, b));
        }
    }
    for ((a, b, c), on) in ccz {
        if on {
            out.push(Surface::Ccz(a, b, c));
        }
    }
    for (q, net) in face {
        // Face(+1) and Face(−1) cancel exactly; the net count survives.
        for _ in 0..net.abs() {
            out.push(Surface::Face(if net > 0 { 1 } else { -1 }, q));
        }
    }
    for (q, count) in rot {
        // Generic rotations at the SAME symbolic angle do not cancel
        // (z·z ≠ 1 in general); they compose, and the symbolic carrier
        // handles a repeated angle natively, so keep them all.
        for _ in 0..count {
            out.push(Surface::Rot(q));
        }
    }
    out
}

/// Cancel adjacent involutions (`H·H`, `X·X`) — one pass, exact.
fn cancel_involutions(gs: Vec<Surface>) -> Vec<Surface> {
    let mut out: Vec<Surface> = Vec::with_capacity(gs.len());
    for g in gs {
        let cancels = match (out.last(), &g) {
            (Some(Surface::H(a)), Surface::H(b)) => a == b,
            (Some(Surface::X(a)), Surface::X(b)) => a == b,
            (Some(Surface::Cx(a, b)), Surface::Cx(c, d)) => a == c && b == d,
            (Some(Surface::Swap(a, b)), Surface::Swap(c, d)) => a == c && b == d,
            _ => false,
        };
        if cancels {
            out.pop();
        } else {
            out.push(g);
        }
    }
    out
}

/// THE PASS: diagonal-run cancellation and involution cancellation, applied
/// to fixpoint (each pass can expose new adjacencies for the other).
pub fn simplify(surface: &[Surface]) -> Vec<Surface> {
    let mut cur = surface.to_vec();
    for _ in 0..16 {
        let before = cur.len();
        // 1 & 3: diagonal runs
        let mut next: Vec<Surface> = Vec::with_capacity(cur.len());
        let mut run: Vec<Surface> = Vec::new();
        for g in cur.drain(..) {
            if is_diagonal(&g) {
                run.push(g);
            } else {
                if !run.is_empty() {
                    next.extend(reduce_diagonal_run(&run));
                    run.clear();
                }
                next.push(g);
            }
        }
        if !run.is_empty() {
            next.extend(reduce_diagonal_run(&run));
        }
        // 2: involutions
        cur = cancel_involutions(next);
        if cur.len() == before {
            break;
        }
    }
    cur
}

/// The magic count that actually prices a run: CCZ counts 7 T-equivalents
/// (its standard decomposition), Face/Rot/T count 1 each.
pub fn magic_weight(surface: &[Surface]) -> usize {
    surface
        .iter()
        .map(|g| match g {
            Surface::Ccz(..) | Surface::Ccx(..) => 7,
            Surface::T(_) | Surface::Tdg(_) | Surface::Face(..) | Surface::Rot(_) => 1,
            Surface::DiagPow(k, _) | Surface::RzPow(k, _) => {
                if k.rem_euclid(2) == 1 {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        })
        .sum()
}
