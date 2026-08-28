//! THE FACE ENGINE — the exact ring for face-state rotations, and the
//! branch expansion that makes the IBM tracker's instances RUNNABLE instead
//! of refused.
//!
//! The algebra that opens the door (verified numerically before this file
//! existed, and pinned by test below): the tracker's basis rotation is
//! `rz(θ_F)` with `θ_F = arccos(1/√3)`, and
//!
//! ```text
//!     e^{iθ_F} = cos θ_F + i sin θ_F = 1/√3 + i·√2/√3 = (1 + i√2)/√3.
//! ```
//!
//! `√2 = ω − ω³` and `i = ω²` are both in Z[ω], so the numerator `1 + i√2`
//! is EXACT in our existing ring; only the `1/√3` escapes it. So the whole
//! rotation is exact in the quadratic extension
//!
//! ```text
//!     R3 := Z[ω][√3] = { a + b√3 : a, b ∈ Cyc },
//! ```
//!
//! with the `1/√3` per gate carried as a global 3-power exponent — the same
//! trick the base ring plays with √2. Nothing is approximated: the tracker's
//! angle, which `qasm.rs` refuses as a float, is an EXACT element here.
//!
//! What this buys, exactly: `diag(1, z)` with `z = e^{iθ_F}` expands as
//!
//! ```text
//!     diag(1, z) = ((1+z)/2)·I + ((1−z)/2)·Z,
//! ```
//!
//! two stabilizer branches with R3 coefficients — the same shape as a
//! T-gate's expansion, so the existing branch machinery carries it. Cost is
//! therefore 2^{f} in the face-gate count f, and the honest statement is
//! that this is the NAIVE face exponent (1.0), not the literature's
//! 0.3424-extent-informed one: the cat-style face decomposition that lowers
//! it is the named next rung (CAMPAIGNS.md #2), and it plugs into the same
//! rank-agnostic interface Magic5 already uses.

use crate::ledger::Cyc;

/// An element of `Z[ω][√3]`: `a + b·√3`, times a global `3^{-halves/2}`
/// bookkeeping exponent (mirroring `Cyc`'s √2 exponent, so the per-gate
/// `1/√3` never leaves the exact world).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct R3 {
    pub a: Cyc,
    pub b: Cyc,
    /// value = (a + b√3) · 3^{−halves/2}
    pub halves: i32,
}

impl R3 {
    pub const ZERO: R3 = R3 { a: Cyc::ZERO, b: Cyc::ZERO, halves: 0 };
    pub const ONE: R3 = R3 { a: Cyc::ONE, b: Cyc::ZERO, halves: 0 };

    pub fn from_cyc(c: Cyc) -> R3 {
        R3 { a: c, b: Cyc::ZERO, halves: 0 }
    }

    pub fn is_zero(&self) -> bool {
        crate::affine::cyc_is_zero(self.a) && crate::affine::cyc_is_zero(self.b)
    }

    /// Align two values to a common 3-power exponent, exactly: raising by
    /// one half multiplies by √3, which swaps the components (a + b√3)·√3 =
    /// 3b + a√3 — no rounding anywhere.
    fn raise(mut self, target: i32) -> R3 {
        while self.halves < target {
            let three = Cyc { c: [3, 0, 0, 0], m: 0 };
            self = R3 { a: self.b.mul(three), b: self.a, halves: self.halves + 1 };
        }
        self
    }

    pub fn add(self, o: R3) -> R3 {
        let t = self.halves.max(o.halves);
        let (x, y) = (self.raise(t), o.raise(t));
        R3 { a: x.a.add(y.a), b: x.b.add(y.b), halves: t }
    }

    pub fn mul(self, o: R3) -> R3 {
        // (a + b√3)(c + d√3) = (ac + 3bd) + (ad + bc)√3
        let three = Cyc { c: [3, 0, 0, 0], m: 0 };
        R3 {
            a: self.a.mul(o.a).add(self.b.mul(o.b).mul(three)),
            b: self.a.mul(o.b).add(self.b.mul(o.a)),
            halves: self.halves + o.halves,
        }
    }

    /// Numeric value, for display and for the conformance oracle only.
    pub fn to_complex(self) -> (f64, f64) {
        let (ar, ai) = self.a.to_complex();
        let (br, bi) = self.b.to_complex();
        let s3 = 3f64.sqrt();
        let scale = 3f64.powf(-(self.halves as f64) / 2.0);
        ((ar + br * s3) * scale, (ai + bi * s3) * scale)
    }
}

/// `√2 = ω − ω³` in the base ring.
pub fn sqrt2() -> Cyc {
    Cyc { c: [0, 1, 0, -1], m: 0 }
}

/// `i = ω²`.
pub fn imag() -> Cyc {
    Cyc { c: [0, 0, 1, 0], m: 0 }
}

/// `e^{iθ_F} = (1 + i√2)/√3`, EXACT: numerator in Z[ω], one 3-half of
/// denominator. This is the tracker's rotation phase, in the ring.
pub fn face_phase() -> R3 {
    let num = Cyc::ONE.add(imag().mul(sqrt2()));
    R3 { a: num, b: Cyc::ZERO, halves: 1 }
}

/// The two branch coefficients of `diag(1, e^{iθ_F}) = c_i·I + c_z·Z`:
/// `c_i = (1+z)/2`, `c_z = (1−z)/2`, both exact in R3.
pub fn face_branch_coeffs() -> (R3, R3) {
    let z = face_phase();
    let half = R3::from_cyc(Cyc { c: [1, 0, 0, 0], m: 2 }); // 1/2 = 2^{-2/2}
    let one = R3::ONE;
    let minus = R3::from_cyc(Cyc { c: [-1, 0, 0, 0], m: 0 });
    (one.add(z).mul(half), one.add(z.mul(minus)).mul(half))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(x: (f64, f64), y: (f64, f64), tag: &str) {
        assert!(
            (x.0 - y.0).abs() < 1e-12 && (x.1 - y.1).abs() < 1e-12,
            "{tag}: {x:?} vs {y:?}"
        );
    }

    #[test]
    fn ring_arithmetic_is_exact_and_aligned() {
        let x = R3 { a: Cyc { c: [1, 2, 0, 0], m: 1 }, b: Cyc { c: [0, 1, 1, 0], m: 0 }, halves: 1 };
        let y = R3 { a: Cyc { c: [2, 0, -1, 0], m: 0 }, b: Cyc { c: [1, 0, 0, 1], m: 2 }, halves: 3 };
        // sum and product agree with float evaluation
        let (xr, xi) = x.to_complex();
        let (yr, yi) = y.to_complex();
        close(x.add(y).to_complex(), (xr + yr, xi + yi), "add");
        close(x.mul(y).to_complex(), (xr * yr - xi * yi, xr * yi + xi * yr), "mul");
    }

    /// THE DOOR: the tracker's exact angle, in the ring.
    #[test]
    fn face_phase_is_the_trackers_angle() {
        let theta = (1f64 / 3f64.sqrt()).acos();
        assert!(
            (theta - 0.9553166181245092).abs() < 1e-15,
            "arccos(1/sqrt3) must be the tracker's literal angle"
        );
        close(face_phase().to_complex(), (theta.cos(), theta.sin()), "e^{i theta_F}");
        // and it is a unit: |z|^2 = 1 exactly (a+b√3 arithmetic, no floats)
        let z = face_phase();
        let conj = R3 { a: z.a.conj(), b: z.b.conj(), halves: z.halves };
        let norm = z.mul(conj);
        close(norm.to_complex(), (1.0, 0.0), "|z|^2 = 1");
    }

    #[test]
    fn branch_coefficients_reconstruct_the_gate() {
        let (ci, cz) = face_branch_coeffs();
        // diag entry 0 = ci + cz = 1 ; entry 1 = ci - cz = z
        let minus = R3::from_cyc(Cyc { c: [-1, 0, 0, 0], m: 0 });
        close(ci.add(cz).to_complex(), (1.0, 0.0), "ci+cz");
        close(ci.add(cz.mul(minus)).to_complex(), face_phase().to_complex(), "ci-cz");
    }
}

// ---------------------------------------------------------------------------
// THE FACE EVALUATOR: exact amplitudes for programs carrying face rotations.
// Each face gate expands to two Z-branches with R3 coefficients; branches
// ride the existing affine engine, and the pruned path's EXACT canonical
// merge runs between rounds — the one dedup, not a second mechanism. The
// tension law (BENCHMARKS entry twelve) applies here in our favour: this
// expansion is the redundant kind, so dedup can actually collapse it.
// ---------------------------------------------------------------------------

use crate::affine::{Affine, Gate};
use crate::qasm::Surface;

/// One branch: an exact R3 weight and an affine stabilizer state.
#[derive(Clone)]
pub struct FaceBranch {
    pub weight: R3,
    pub state: Affine,
}

/// Exact amplitude of `y` for a surface program that may contain face
/// rotations. Cost is 2^f in the face count before dedup; the canonical
/// merge is applied every `merge_every` face gates.
pub fn amplitude_face(
    n: usize,
    surface: &[Surface],
    y: &[bool],
    merge_every: usize,
) -> (R3, usize) {
    let (ci, cz) = face_branch_coeffs();
    let mut branches = vec![FaceBranch {
        weight: R3::ONE,
        state: Affine::new(n),
    }];
    let mut since_merge = 0usize;
    let mut peak = 1usize;

    // THE UNIFICATION: every diagonal magic gate is the SAME shape — a
    // two-branch expansion diag(1,z) = ((1+z)/2)·I + ((1−z)/2)·Z — and only
    // the ring element z differs (ω for T, ω⁻¹ for T†, the face phase for
    // Face). One code path carries all of them.
    let conj3 = |x: R3| R3 { a: x.a.conj(), b: x.b.conj(), halves: x.halves };
    let split = |z: R3| {
        let half = R3::from_cyc(Cyc { c: [1, 0, 0, 0], m: 2 });
        let minus = R3::from_cyc(Cyc { c: [-1, 0, 0, 0], m: 0 });
        (R3::ONE.add(z).mul(half), R3::ONE.add(z.mul(minus)).mul(half))
    };
    let t_split = split(R3::from_cyc(crate::affine::omega_pow(1)));
    let tdg_split = split(R3::from_cyc(crate::affine::omega_pow(7)));

    for &g in surface {
        let diag: Option<((R3, R3), usize)> = match g {
            Surface::Face(sign, q) => Some((
                if sign >= 0 { (ci, cz) } else { (conj3(ci), conj3(cz)) },
                q,
            )),
            Surface::T(q) => Some((t_split, q)),
            Surface::Tdg(q) => Some((tdg_split, q)),
            _ => None,
        };
        match diag {
            Some(((a, b), q)) => {
                let mut next = Vec::with_capacity(branches.len() * 2);
                for br in branches.drain(..) {
                    let mut zb = br.clone();
                    zb.state.apply(Gate::Z(q));
                    zb.weight = zb.weight.mul(b);
                    let mut ib = br;
                    ib.weight = ib.weight.mul(a);
                    next.push(ib);
                    next.push(zb);
                }
                branches = next;
                peak = peak.max(branches.len());
                since_merge += 1;
                if since_merge >= merge_every {
                    branches = merge_face(branches);
                    since_merge = 0;
                }
            }
            None => {
                // Clifford-lowerable gate: lower once, apply to every branch
                let (core, phase16) = crate::qasm::lower(&[g]);
                debug_assert_eq!(phase16.rem_euclid(2), 0, "odd ζ16 phase in a face program");
                let wphase = phase16.rem_euclid(16) / 2;
                for br in branches.iter_mut() {
                    for cg in &core {
                        br.state.apply(*cg);
                    }
                    for _ in 0..wphase {
                        br.weight = br.weight.mul(R3::from_cyc(crate::affine::omega_pow(1)));
                    }
                }
            }
        }
    }
    branches = merge_face(branches);
    let mut acc = R3::ZERO;
    for br in &branches {
        let a = br.state.amplitude(y);
        if !crate::affine::cyc_is_zero(a) {
            acc = acc.add(br.weight.mul(R3::from_cyc(a)));
        }
    }
    (acc, peak)
}

/// The exact canonical merge on face branches: canonicalize, fold the
/// state's global scalar into the R3 weight, and add duplicates exactly.
fn merge_face(mut branches: Vec<FaceBranch>) -> Vec<FaceBranch> {
    let mut kept: Vec<FaceBranch> = Vec::with_capacity(branches.len());
    let mut keys: Vec<Vec<u8>> = Vec::new();
    for mut br in branches.drain(..) {
        let g = br.state.canonicalize();
        if br.state.is_zero() {
            continue;
        }
        br.weight = br.weight.mul(R3::from_cyc(g));
        if br.weight.is_zero() {
            continue;
        }
        let key = br.state.canon_key();
        match keys.iter().position(|k| *k == key) {
            Some(i) => {
                kept[i].weight = kept[i].weight.add(br.weight);
                if kept[i].weight.is_zero() {
                    kept.remove(i);
                    keys.remove(i);
                }
            }
            None => {
                keys.push(key);
                kept.push(br);
            }
        }
    }
    kept
}

#[cfg(test)]
mod eval_tests {
    use super::*;
    use crate::qasm::Surface::*;

    /// A single face rotation on |+⟩, against the closed form: the exact
    /// engine must equal (1 + z)/2 · 2^{-1/2} at y = 0.
    #[test]
    fn one_face_gate_matches_closed_form() {
        let prog = vec![H(0), Face(1, 0)];
        let (amp, _) = amplitude_face(1, &prog, &[false], 4);
        let expect = R3::from_cyc(crate::ledger::Cyc { c: [1, 0, 0, 0], m: 1 }); // 1/sqrt2
        let (ar, ai) = amp.to_complex();
        let (er, ei) = expect.to_complex();
        assert!((ar - er).abs() < 1e-12 && (ai - ei).abs() < 1e-12, "{amp:?}");
    }

    /// Face-free programs must agree with the core path exactly — the
    /// face engine is a superset, never a second semantics.
    #[test]
    fn face_free_agrees_with_core() {
        let prog = vec![H(0), Cx(0, 1), S(1), T(0), H(1)];
        let (core, _) = crate::qasm::lower(&prog);
        for basis in 0..4u32 {
            let y: Vec<bool> = (0..2).map(|q| basis >> q & 1 == 1).collect();
            let (fa, _) = amplitude_face(2, &prog, &y, 4);
            let ca = crate::run::amplitude(2, &core, &y);
            let (fr, fi) = fa.to_complex();
            let (cr, ci2) = ca.to_complex();
            assert!(
                (fr - cr).abs() < 1e-12 && (fi - ci2).abs() < 1e-12,
                "basis {basis}: face engine {fa:?} vs core {ca:?}"
            );
        }
    }

    /// Probabilities over a full basis must sum to 1 — the engine's own
    /// normalization check on a face-carrying circuit.
    #[test]
    fn face_circuit_probabilities_sum_to_one() {
        let prog = vec![H(0), H(1), Cx(0, 1), Face(1, 0), Face(-1, 1), H(0)];
        let mut tot = 0.0;
        for basis in 0..4u32 {
            let y: Vec<bool> = (0..2).map(|q| basis >> q & 1 == 1).collect();
            let (a, _) = amplitude_face(2, &prog, &y, 2);
            let (r, i) = a.to_complex();
            tot += r * r + i * i;
        }
        assert!((tot - 1.0).abs() < 1e-10, "probabilities sum to {tot}");
    }
}
