//! NON-LOCAL SIMPLIFICATION — the phase-polynomial pass.
//!
//! `simplify.rs` cancels what MEETS: gates adjacent in one diagonal run.
//! Its measured ceiling (BENCHMARKS entry fourteen) is that magic weight only
//! drops when CCZ triples happen to repeat inside a single run. This module
//! removes that ceiling for the {CX, CZ, CCZ, Z, S, T, DiagPow} fragment by
//! working in the representation where distance does not exist.
//!
//! The idea (Amy–Maslov–Mosca's phase-polynomial normalization, credited):
//! inside a maximal CNOT+diagonal block, each qubit carries an F₂ LINEAR FORM
//! of the block's input variables, and every diagonal gate contributes a
//! phase `ω^k` on one such form:
//!
//! ```text
//!   Z(q)         →  k=4 on f_q            S(q) → k=2      T(q) → k=1
//!   CZ(a,b)      →  since a·b = (a + b − (a⊕b))/2,
//!                   k=2 on f_a, k=2 on f_b, k=−2 on f_a⊕f_b
//!   CCZ(a,b,c)   →  since a·b·c = (a+b+c − (a⊕b) − (a⊕c) − (b⊕c)
//!                                  + (a⊕b⊕c))/4,
//!                   k=1 on each single, k=−1 on each pair-XOR,
//!                   k=1 on the triple-XOR   (the 7-T decomposition, exactly)
//! ```
//!
//! Two gates contributing to the SAME linear form merge by adding their `k`
//! mod 8 — **regardless of how many gates separated them**. A form whose
//! total `k` is even costs no T; a form whose total is 0 vanishes entirely.
//! That is magic cancellation at a distance, which is precisely what the
//! local pass could not do and what the QuiZX gap measured.
//!
//! Scope, stated: this is exact for CNOT+diagonal blocks. Hadamards end a
//! block (they do not preserve the linear-form representation), so a circuit
//! is processed block by block. Full ZX rewriting reaches further still —
//! spider fusion and pivoting act ACROSS Hadamards — and remains the named
//! next rung; this pass is its provable, implemented floor.

use crate::qasm::Surface;
use std::collections::BTreeMap;

/// An F₂ linear form over the block's input variables, as a bitmask.
type Form = u64;

/// The phase polynomial of one CNOT+diagonal block: `k` (mod 8) per form,
/// plus the CNOT frame the block ends in.
struct Block {
    /// form → accumulated ω-power (mod 8)
    terms: BTreeMap<Form, i64>,
    /// per-qubit linear form at the block's end
    frame: Vec<Form>,
    /// the CNOTs, in order, so the frame can be rebuilt on emission
    cnots: Vec<(usize, usize)>,
}

impl Block {
    fn new(n: usize) -> Block {
        assert!(n <= 64, "phase-polynomial pass handles up to 64 qubits per block");
        Block {
            terms: BTreeMap::new(),
            frame: (0..n).map(|q| 1u64 << q).collect(),
            cnots: Vec::new(),
        }
    }

    fn add(&mut self, f: Form, k: i64) {
        if f == 0 {
            return; // phase on the zero form is a global scalar; tracked below
        }
        let e = self.terms.entry(f).or_insert(0);
        *e = (*e + k).rem_euclid(8);
        if *e == 0 {
            self.terms.remove(&f);
        }
    }
}

/// Number of T-equivalents a block's phase polynomial costs: one per form
/// whose accumulated power is ODD (even powers are Clifford).
fn block_t_count(b: &Block) -> usize {
    b.terms.values().filter(|k| k.rem_euclid(2) == 1).count()
}

/// Apply the phase-polynomial pass to a surface program. Returns the
/// rewritten program. Exact: gated by `tests/phasepoly.rs` on every basis
/// state.
pub fn optimize(n: usize, surface: &[Surface]) -> Vec<Surface> {
    if n > 64 {
        return surface.to_vec(); // out of scope: refuse to transform, never to run
    }
    let mut out: Vec<Surface> = Vec::new();
    let mut block = Block::new(n);
    let mut pending: Vec<Surface> = Vec::new(); // gates the block does not model

    let flush = |block: &mut Block, out: &mut Vec<Surface>| {
        if block.terms.is_empty() && block.cnots.is_empty() {
            return;
        }
        out.extend(emit_block(block));
        *block = Block::new(block.frame.len());
    };

    for &g in surface {
        match g {
            Surface::Cx(a, b) => {
                let fa = block.frame[a];
                block.frame[b] ^= fa;
                block.cnots.push((a, b));
            }
            Surface::Z(q) => block.add(block.frame[q], 4),
            Surface::S(q) => block.add(block.frame[q], 2),
            Surface::Sdg(q) => block.add(block.frame[q], 6),
            Surface::T(q) => block.add(block.frame[q], 1),
            Surface::Tdg(q) => block.add(block.frame[q], 7),
            Surface::DiagPow(k, q) => block.add(block.frame[q], k),
            Surface::Cz(a, b) => {
                let (fa, fb) = (block.frame[a], block.frame[b]);
                block.add(fa, 2);
                block.add(fb, 2);
                block.add(fa ^ fb, -2);
            }
            Surface::Ccz(a, b, c) => {
                let (fa, fb, fc) = (block.frame[a], block.frame[b], block.frame[c]);
                block.add(fa, 1);
                block.add(fb, 1);
                block.add(fc, 1);
                block.add(fa ^ fb, -1);
                block.add(fa ^ fc, -1);
                block.add(fb ^ fc, -1);
                block.add(fa ^ fb ^ fc, 1);
            }
            other => {
                // Not modelled by the block (H, X, Face, Rot, RzPow, …):
                // close the block, emit it, then pass the gate through.
                flush(&mut block, &mut out);
                out.push(other);
                pending.push(other);
            }
        }
    }
    flush(&mut block, &mut out);
    out
}

/// Emit a block: rebuild the CNOT frame, then place each surviving phase
/// term. A term on a single-qubit form is a plain `DiagPow`; a term on a
/// multi-qubit form is realized by a CNOT ladder that computes the parity,
/// the phase, then the ladder's inverse — exact, and the standard
/// construction.
fn emit_block(b: &Block) -> Vec<Surface> {
    let n = b.frame.len();
    let mut out = Vec::new();
    // Rebuild the frame the block ended in (the CNOTs, in order).
    for &(a, c) in &b.cnots {
        out.push(Surface::Cx(a, c));
    }
    // With the frame rebuilt, qubit q holds form b.frame[q]. For each phase
    // term, find a qubit whose form is a subset we can reach: the simplest
    // exact route is to compute the term's parity into a chosen qubit via
    // CNOTs from the block's INPUT basis. Since the frame is invertible we
    // can express any form as an XOR of frame rows; do that by solving.
    let mut rows: Vec<(Form, usize)> = b.frame.iter().copied().zip(0..n).collect();
    for (&form, &k) in &b.terms {
        if k == 0 {
            continue;
        }
        // Solve form = XOR of some frame rows (Gaussian elimination over F₂).
        let sel = solve_xor(&rows, form);
        match sel {
            Some(qs) if !qs.is_empty() => {
                let target = qs[0];
                for &q in &qs[1..] {
                    out.push(Surface::Cx(q, target));
                }
                out.push(Surface::DiagPow(k, target));
                for &q in qs[1..].iter().rev() {
                    out.push(Surface::Cx(q, target));
                }
            }
            _ => {
                // Unreachable for an invertible frame; if it ever happens,
                // refuse loudly rather than silently dropping a phase.
                panic!("phase-polynomial emit: form {form:#x} not in the frame's span");
            }
        }
    }
    rows.clear();
    out
}

/// Solve `form = XOR of chosen rows` over F₂; returns the chosen qubits.
fn solve_xor(rows: &[(Form, usize)], form: Form) -> Option<Vec<usize>> {
    // Gaussian elimination tracking which rows combine into each pivot.
    let mut basis: Vec<(Form, u64)> = Vec::new(); // (value, row-mask)
    for (i, &(r, _)) in rows.iter().enumerate() {
        let (mut v, mut mask) = (r, 1u64 << i);
        for &(bv, bm) in &basis {
            if v ^ bv < v {
                v ^= bv;
                mask ^= bm;
            }
        }
        if v != 0 {
            basis.push((v, mask));
            basis.sort_by(|a, b| b.0.cmp(&a.0));
        }
    }
    let (mut v, mut mask) = (form, 0u64);
    for &(bv, bm) in &basis {
        if v ^ bv < v {
            v ^= bv;
            mask ^= bm;
        }
    }
    if v != 0 {
        return None;
    }
    Some((0..rows.len()).filter(|&i| mask >> i & 1 == 1).map(|i| rows[i].1).collect())
}

/// The T-count a program costs after phase-polynomial normalization —
/// computed WITHOUT emitting, so a caller can price a circuit cheaply.
pub fn normalized_t_count(n: usize, surface: &[Surface]) -> usize {
    if n > 64 {
        return crate::simplify::magic_weight(surface);
    }
    let mut total = 0usize;
    let mut block = Block::new(n);
    for &g in surface {
        match g {
            Surface::Cx(a, b) => {
                let fa = block.frame[a];
                block.frame[b] ^= fa;
            }
            Surface::Z(q) => block.add(block.frame[q], 4),
            Surface::S(q) => block.add(block.frame[q], 2),
            Surface::Sdg(q) => block.add(block.frame[q], 6),
            Surface::T(q) => block.add(block.frame[q], 1),
            Surface::Tdg(q) => block.add(block.frame[q], 7),
            Surface::DiagPow(k, q) => block.add(block.frame[q], k),
            Surface::Cz(a, b) => {
                let (fa, fb) = (block.frame[a], block.frame[b]);
                block.add(fa, 2);
                block.add(fb, 2);
                block.add(fa ^ fb, -2);
            }
            Surface::Ccz(a, b, c) => {
                let (fa, fb, fc) = (block.frame[a], block.frame[b], block.frame[c]);
                block.add(fa, 1);
                block.add(fb, 1);
                block.add(fc, 1);
                block.add(fa ^ fb, -1);
                block.add(fa ^ fc, -1);
                block.add(fb ^ fc, -1);
                block.add(fa ^ fb ^ fc, 1);
            }
            Surface::Face(..) | Surface::Rot(_) | Surface::RzPow(..) => total += 1,
            _ => {
                total += block_t_count(&block);
                block = Block::new(n);
            }
        }
    }
    total + block_t_count(&block)
}
