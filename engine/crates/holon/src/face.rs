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
            Surface::Rot(_) => panic!(
                "amplitude_face carries only ring-exact rotations; a generic \
                 angle must go to amplitude_poly (symbolic carrier)"
            ),
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

// ---------------------------------------------------------------------------
// GENERIC ANGLES, UNLOCKED — the symbolic carrier.
//
// The observation that opens it: for a circuit whose non-Clifford content is
// f rotations `diag(1, z)` all at the SAME angle, the exact amplitude is a
// POLYNOMIAL in z with Z[ω] coefficients:
//
//     ⟨y|C|0⟩ = Σ_{j=0}^{f} A_j · z^j,      A_j ∈ Z[ω],
//
// because each rotation contributes `((1+z)/2)·I + ((1−z)/2)·Z` and the
// branch sum collects powers of z. The polynomial is ANGLE-FREE: computing
// it once answers EVERY angle, exactly for angles in the ring tower and to
// full float precision for the rest. That is the honest sense in which
// generic angles are unlocked — not "we can carry e^{iθ} exactly for
// transcendental θ" (nothing can), but "the θ-dependence is factored out
// into an exact object, so no approximation ever enters the circuit
// evaluation; it enters only when you choose a numeric θ, at the very end."
//
// Cost is unchanged (2^f branches before dedup) but the RESULT is strictly
// stronger: one run prices a whole angle family, which is exactly what a
// calibration sweep, a basis-choice campaign, or a hardware-comparison needs.
// ---------------------------------------------------------------------------

/// An exact polynomial `Σ A_j z^j` with `Z[ω]` coefficients — the amplitude
/// of a circuit as a function of its (single) rotation phase.
///
/// This is a `MergeLedger` (see `merge.rs`), so symbolic amplitudes fold
/// through the ONE merge law: mesh sharding, exact dedup, and the
/// certificate machinery all apply to generic-angle work unchanged.
#[derive(Clone, Debug, PartialEq)]
pub struct Poly {
    pub coeffs: Vec<Cyc>,
}

impl Poly {
    pub fn zero() -> Poly {
        Poly { coeffs: vec![] }
    }

    pub fn constant(c: Cyc) -> Poly {
        Poly { coeffs: vec![c] }
    }

    pub fn is_zero(&self) -> bool {
        self.coeffs.iter().all(|c| crate::affine::cyc_is_zero(*c))
    }

    pub fn degree(&self) -> usize {
        self.coeffs.len().saturating_sub(1)
    }

    pub fn add(&self, o: &Poly) -> Poly {
        let n = self.coeffs.len().max(o.coeffs.len());
        let mut c = Vec::with_capacity(n);
        for i in 0..n {
            let a = self.coeffs.get(i).copied().unwrap_or(Cyc::ZERO);
            let b = o.coeffs.get(i).copied().unwrap_or(Cyc::ZERO);
            c.push(a.add(b));
        }
        Poly { coeffs: c }
    }

    pub fn scale(&self, s: Cyc) -> Poly {
        Poly { coeffs: self.coeffs.iter().map(|c| c.mul(s)).collect() }
    }

    /// Multiply by `(a + b·z)` — the only multiply the expansion needs.
    pub fn mul_linear(&self, a: Cyc, b: Cyc) -> Poly {
        let mut out = vec![Cyc::ZERO; self.coeffs.len() + 1];
        for (i, &c) in self.coeffs.iter().enumerate() {
            out[i] = out[i].add(c.mul(a));
            out[i + 1] = out[i + 1].add(c.mul(b));
        }
        Poly { coeffs: out }
    }

    /// Evaluate at an exact ring element (face phase, any ζ_n, …).
    pub fn eval_r3(&self, z: R3) -> R3 {
        let mut acc = R3::ZERO;
        let mut zp = R3::ONE;
        for &c in &self.coeffs {
            acc = acc.add(R3::from_cyc(c).mul(zp));
            zp = zp.mul(z);
        }
        acc
    }

    /// Evaluate at ANY angle — the only place a float ever appears, and it
    /// appears AFTER all exact work is finished.
    pub fn eval_angle(&self, theta: f64) -> (f64, f64) {
        let (mut re, mut im) = (0.0, 0.0);
        for (j, &c) in self.coeffs.iter().enumerate() {
            let (cr, ci) = c.to_complex();
            let (zr, zi) = ((theta * j as f64).cos(), (theta * j as f64).sin());
            re += cr * zr - ci * zi;
            im += cr * zi + ci * zr;
        }
        (re, im)
    }
}

/// One symbolic branch: an exact polynomial weight and an affine state.
#[derive(Clone)]
struct PolyBranch {
    weight: Poly,
    state: Affine,
}

/// THE GENERIC-ANGLE EVALUATOR: the amplitude of `surface` at `y` as an
/// exact polynomial in the rotation phase `z`, with every `Rot` surface gate
/// treated as `diag(1,z)` for the SAME symbolic z. `T`/`Tdg` are lowered
/// normally (they are Clifford+T, in-ring); only the generic rotations
/// become symbolic. Halves of the ½ per rotation ride in the coefficients.
pub fn amplitude_poly(n: usize, surface: &[Surface], y: &[bool], merge_every: usize) -> (Poly, usize) {
    let half = Cyc { c: [1, 0, 0, 0], m: 2 }; // ½
    let mut branches = vec![PolyBranch { weight: Poly::constant(Cyc::ONE), state: Affine::new(n) }];
    let mut since = 0usize;
    let mut peak = 1usize;

    for &g in surface {
        let rot_q = match g {
            Surface::Face(_, q) => Some(q),
            Surface::Rot(q) => Some(q),
            _ => None,
        };
        match rot_q {
            Some(q) => {
                // diag(1,z) = ((1+z)/2) I + ((1−z)/2) Z
                let mut next = Vec::with_capacity(branches.len() * 2);
                for br in branches.drain(..) {
                    let mut zb = br.clone();
                    zb.state.apply(Gate::Z(q));
                    // (1 − z)/2 : a = ½, b = −½
                    zb.weight = zb.weight.mul_linear(half, half.mul(Cyc { c: [-1, 0, 0, 0], m: 0 }));
                    let mut ib = br;
                    ib.weight = ib.weight.mul_linear(half, half);
                    next.push(ib);
                    next.push(zb);
                }
                branches = next;
                peak = peak.max(branches.len());
                since += 1;
                if since >= merge_every {
                    branches = merge_poly(branches);
                    since = 0;
                }
            }
            None => {
                let (core, phase16) = crate::qasm::lower(&[g]);
                let wphase = phase16.rem_euclid(16) / 2;
                for br in branches.iter_mut() {
                    for cg in &core {
                        br.state.apply(*cg);
                    }
                    for _ in 0..wphase {
                        br.weight = br.weight.scale(crate::affine::omega_pow(1));
                    }
                }
            }
        }
    }
    branches = merge_poly(branches);
    let mut acc = Poly::zero();
    for br in &branches {
        let a = br.state.amplitude(y);
        if !crate::affine::cyc_is_zero(a) {
            acc = acc.add(&br.weight.scale(a));
        }
    }
    (acc, peak)
}

fn merge_poly(mut branches: Vec<PolyBranch>) -> Vec<PolyBranch> {
    let mut kept: Vec<PolyBranch> = Vec::with_capacity(branches.len());
    let mut keys: Vec<Vec<u8>> = Vec::new();
    for mut br in branches.drain(..) {
        let g = br.state.canonicalize();
        if br.state.is_zero() {
            continue;
        }
        br.weight = br.weight.scale(g);
        if br.weight.is_zero() {
            continue;
        }
        let key = br.state.canon_key();
        match keys.iter().position(|k| *k == key) {
            Some(i) => {
                kept[i].weight = kept[i].weight.add(&br.weight);
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
mod poly_tests {
    use super::*;
    use crate::qasm::Surface::*;

    /// The polynomial evaluated at the FACE phase must equal the face
    /// engine's own exact answer — two independent routes to one value.
    #[test]
    fn polynomial_at_the_face_phase_matches_the_face_engine() {
        let progs: Vec<Vec<Surface>> = vec![
            vec![H(0), Face(1, 0)],
            vec![H(0), H(1), Cx(0, 1), Face(1, 0), Face(1, 1), H(0)],
            vec![H(0), Cx(0, 1), Face(1, 1), S(0), Face(1, 0), H(1)],
        ];
        for (t, prog) in progs.iter().enumerate() {
            let n = 2.min(prog.iter().filter_map(|g| match g {
                Cx(a, b) => Some(a.max(b) + 1),
                H(q) | S(q) | Face(_, q) => Some(q + 1),
                _ => None,
            }).max().unwrap_or(1)).max(1);
            for basis in 0..(1u32 << n) {
                let y: Vec<bool> = (0..n).map(|q| basis >> q & 1 == 1).collect();
                let (poly, _) = amplitude_poly(n, prog, &y, 2);
                let (face, _) = amplitude_face(n, prog, &y, 2);
                let pv = poly.eval_r3(face_phase()).to_complex();
                let fv = face.to_complex();
                assert!(
                    (pv.0 - fv.0).abs() < 1e-12 && (pv.1 - fv.1).abs() < 1e-12,
                    "trial {t} basis {basis}: poly {pv:?} vs face engine {fv:?}"
                );
            }
        }
    }

    /// GENERIC ANGLES: the same polynomial, evaluated at angles in NO ring,
    /// must equal a direct dense simulation at that angle — so one exact
    /// object prices the whole family.
    #[test]
    fn one_polynomial_prices_every_angle() {
        let n = 2;
        let prog = vec![H(0), H(1), Cx(0, 1), Rot(0), Rot(1), H(0)];
        for theta in [0.3f64, 1.0, 2.7, -0.9, 0.955_316_618_124_509_2] {
            // dense reference at this angle
            let mut v = vec![(0.0f64, 0.0f64); 1 << n];
            v[0] = (1.0, 0.0);
            let apply_h = |v: &mut Vec<(f64, f64)>, q: usize| {
                let s = 1.0 / 2f64.sqrt();
                for i in 0..v.len() {
                    if i >> q & 1 == 0 {
                        let j = i | (1 << q);
                        let (a, b) = (v[i], v[j]);
                        v[i] = ((a.0 + b.0) * s, (a.1 + b.1) * s);
                        v[j] = ((a.0 - b.0) * s, (a.1 - b.1) * s);
                    }
                }
            };
            for &g in &prog {
                match g {
                    H(q) => apply_h(&mut v, q),
                    Cx(a, b) => {
                        for i in 0..v.len() {
                            if i >> a & 1 == 1 && i >> b & 1 == 0 {
                                v.swap(i, i | (1 << b));
                            }
                        }
                    }
                    Rot(q) => {
                        let (zr, zi) = (theta.cos(), theta.sin());
                        for i in 0..v.len() {
                            if i >> q & 1 == 1 {
                                let (a, b) = v[i];
                                v[i] = (a * zr - b * zi, a * zi + b * zr);
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
            for basis in 0..(1u32 << n) {
                let y: Vec<bool> = (0..n).map(|q| basis >> q & 1 == 1).collect();
                let (poly, _) = amplitude_poly(n, &prog, &y, 2);
                let got = poly.eval_angle(theta);
                let want = v[basis as usize];
                assert!(
                    (got.0 - want.0).abs() < 1e-10 && (got.1 - want.1).abs() < 1e-10,
                    "theta {theta} basis {basis}: {got:?} vs dense {want:?}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// MULTI-VIEW: what the symbolic carrier is FOR.
//
// The polynomial factors the angle out of the exact work, so one branch
// evaluation answers a whole family of questions. Two families matter:
//
//   * MULTI-ANGLE — one circuit, many θ (calibration sweeps, basis-choice
//     campaigns, hardware comparisons). Cost: one exact run + O(deg) per
//     angle.
//   * MULTI-VIEW — one circuit, many basis states y (probability tables,
//     marginals, sampling). The branch set is shared; only the final
//     per-branch amplitude query varies.
//
// Both fold through `merge::MergeLedger` (Poly is an instance), so the mesh
// shards them with the same law and the same certificates as every other
// tier — no second mechanism.
// ---------------------------------------------------------------------------

/// The branch set of a symbolic program, retained so many views and many
/// angles can be answered without re-expanding. This is the object a
/// multi-view consumer should hold.
pub struct SymbolicSum {
    branches: Vec<PolyBranch>,
    n: usize,
}

impl SymbolicSum {
    /// Expand once (with the exact canonical merge every `merge_every`
    /// rotations); afterwards every query is cheap.
    pub fn build(n: usize, surface: &[Surface], merge_every: usize) -> SymbolicSum {
        let (branches, _) = expand_poly(n, surface, merge_every);
        SymbolicSum { branches, n }
    }

    pub fn n_branches(&self) -> usize {
        self.branches.len()
    }

    /// One VIEW: the exact amplitude polynomial at basis state `y`.
    pub fn view(&self, y: &[bool]) -> Poly {
        assert_eq!(y.len(), self.n, "view width must equal the qubit count");
        let mut acc = Poly::zero();
        for br in &self.branches {
            let a = br.state.amplitude(y);
            if !crate::affine::cyc_is_zero(a) {
                acc = acc.add(&br.weight.scale(a));
            }
        }
        acc
    }

    /// MANY VIEWS at once — the branch set is shared; folding uses the one
    /// merge law (`Poly: MergeLedger`), so this is shardable unchanged.
    pub fn views(&self, ys: &[Vec<bool>]) -> Vec<Poly> {
        ys.iter().map(|y| self.view(y)).collect()
    }

    /// MANY ANGLES for one view: the exact polynomial, priced at each θ.
    pub fn angles(&self, y: &[bool], thetas: &[f64]) -> Vec<(f64, f64)> {
        let p = self.view(y);
        thetas.iter().map(|&t| p.eval_angle(t)).collect()
    }

    /// The full probability table over all `2^n` basis states at one angle —
    /// the multi-view payoff in its most useful form. Refuses above 20
    /// qubits rather than silently allocating (a refusal is a result).
    pub fn probability_table(&self, theta: f64) -> Vec<f64> {
        assert!(self.n <= 20, "probability_table refuses above 20 qubits: 2^n table");
        (0..1u32 << self.n)
            .map(|b| {
                let y: Vec<bool> = (0..self.n).map(|q| b >> q & 1 == 1).collect();
                let (re, im) = self.view(&y).eval_angle(theta);
                re * re + im * im
            })
            .collect()
    }
}

/// Shared expansion used by both the one-shot and the multi-view paths.
fn expand_poly(n: usize, surface: &[Surface], merge_every: usize) -> (Vec<PolyBranch>, usize) {
    let half = Cyc { c: [1, 0, 0, 0], m: 2 };
    let mut branches = vec![PolyBranch { weight: Poly::constant(Cyc::ONE), state: Affine::new(n) }];
    let mut since = 0usize;
    let mut peak = 1usize;
    for &g in surface {
        let rot_q = match g {
            Surface::Face(_, q) | Surface::Rot(q) => Some(q),
            _ => None,
        };
        match rot_q {
            Some(q) => {
                let mut next = Vec::with_capacity(branches.len() * 2);
                for br in branches.drain(..) {
                    let mut zb = br.clone();
                    zb.state.apply(Gate::Z(q));
                    zb.weight = zb.weight.mul_linear(half, half.mul(Cyc { c: [-1, 0, 0, 0], m: 0 }));
                    let mut ib = br;
                    ib.weight = ib.weight.mul_linear(half, half);
                    next.push(ib);
                    next.push(zb);
                }
                branches = next;
                peak = peak.max(branches.len());
                since += 1;
                if since >= merge_every {
                    branches = merge_poly(branches);
                    since = 0;
                }
            }
            None => {
                let (core, phase16) = crate::qasm::lower(&[g]);
                let wphase = phase16.rem_euclid(16) / 2;
                for br in branches.iter_mut() {
                    for cg in &core {
                        br.state.apply(*cg);
                    }
                    for _ in 0..wphase {
                        br.weight = br.weight.scale(crate::affine::omega_pow(1));
                    }
                }
            }
        }
    }
    (merge_poly(branches), peak)
}

#[cfg(test)]
mod multiview_tests {
    use super::*;
    use crate::merge::{fold, MergeLedger};
    use crate::qasm::Surface::*;

    fn prog() -> Vec<Surface> {
        vec![H(0), H(1), Cx(0, 1), Rot(0), S(1), Rot(1), H(0), Face(1, 1)]
    }

    /// Multi-view must agree with the one-shot path on every view, and the
    /// SAME object must serve every angle.
    #[test]
    fn multiview_agrees_with_one_shot_everywhere() {
        let n = 2;
        let p = prog();
        let sum = SymbolicSum::build(n, &p, 2);
        for b in 0..4u32 {
            let y: Vec<bool> = (0..n).map(|q| b >> q & 1 == 1).collect();
            let one = amplitude_poly(n, &p, &y, 2).0;
            let many = sum.view(&y);
            assert_eq!(one, many, "view {b}: multi-view diverges from one-shot");
        }
        // one object, many angles
        let y = vec![false, false];
        let ts = [0.1f64, 0.9553166181245092, 2.0];
        let got = sum.angles(&y, &ts);
        for (i, &t) in ts.iter().enumerate() {
            let want = amplitude_poly(n, &p, &y, 2).0.eval_angle(t);
            assert!((got[i].0 - want.0).abs() < 1e-12 && (got[i].1 - want.1).abs() < 1e-12);
        }
    }

    /// The probability table must sum to 1 at EVERY angle — the multi-view
    /// object's own normalization certificate.
    #[test]
    fn probability_table_normalizes_at_every_angle() {
        let sum = SymbolicSum::build(2, &prog(), 1);
        for theta in [0.0f64, 0.3, 0.9553166181245092, 1.7, -2.2] {
            let t: f64 = sum.probability_table(theta).iter().sum();
            assert!((t - 1.0).abs() < 1e-10, "theta {theta}: table sums to {t}");
        }
    }

    /// Poly is a lawful MergeLedger: identity and associativity, so the mesh
    /// shards symbolic work with the one law.
    #[test]
    fn poly_is_a_lawful_ledger() {
        let a = Poly { coeffs: vec![Cyc::ONE, Cyc { c: [0, 1, 0, 0], m: 0 }] };
        let b = Poly { coeffs: vec![Cyc { c: [2, 0, 0, 0], m: 1 }] };
        let c = Poly { coeffs: vec![Cyc::ZERO, Cyc::ZERO, Cyc::ONE] };
        assert_eq!(a.clone().merge(Poly::empty()), a);
        assert_eq!(Poly::empty().merge(a.clone()), a);
        let l = a.clone().merge(b.clone()).merge(c.clone());
        let r = a.clone().merge(b.clone().merge(c.clone()));
        assert_eq!(l, r, "associativity");
        assert_eq!(fold(vec![a, b, c]), l, "fold agrees with the law");
    }
}

// ---------------------------------------------------------------------------
// FACTORING OUT THE HARD PART — what CAN be removed from the exponential,
// and the exact statement of what cannot.
//
// The general exponential cannot be removed: exact Clifford+rotation
// amplitudes are #P-hard, so any claim to a general polynomial algorithm
// would collapse the polynomial hierarchy. What CAN be factored out is
// everything in the expansion that is not actually exponential, and there
// are exactly two such parts:
//
// 1. THE RANK, NOT THE COUNT. Push each rotation's Z to the end of the
//    circuit through the remaining Clifford: `Z_i ↦ P_i`, a Pauli. Then the
//    branch for subset `S` is `C·∏_{i∈S} P_i`, and a product of Paulis is a
//    phase times the single Pauli given by the F₂ XOR of their labels. So
//    the 2^f branches take only `2^r` DISTINCT values, where `r` is the F₂
//    rank of `{P_i}` — and `r ≤ min(f, 2n)` always. Measured collapse:
//    f=70 rotations on n=4 qubits is rank 8, i.e. 2^8 instead of 2^70.
//    (`rotation_rank` computes it; the engine's canonical merge realizes
//    it — this is WHY the dedup collapses these expansions, stated as a
//    theorem instead of observed as luck.)
//
// 2. THE CLIFFORD SKELETON. At `z ∈ {1, i, −1, −i}` the rotation
//    `diag(1,z)` is `{I, S, Z, S†}` — CLIFFORD. So the amplitude at those
//    four points is computable in POLYNOMIAL TIME by the tableau, with no
//    branching at all. That gives four exact values of an object whose full
//    computation is exponential: a poly-time CERTIFICATE on the expensive
//    result, and an exact determination whenever the degree is ≤ 3.
//
// The honest boundary: for a circuit where the rotations act on many
// independent qubits (the IBM tracker's 70-on-70 is exactly this), `r = f`
// and no collapse exists. That case is hard because it IS hard.
// ---------------------------------------------------------------------------

/// The F₂ rank of the rotation set — the TRUE exponent of the expansion.
/// Each rotation's Z, conjugated to the circuit's end, is a Pauli with a
/// `2n`-bit F₂ label; the rank of those labels bounds the number of
/// distinct branches by `2^rank`.
pub fn rotation_rank(n: usize, surface: &[Surface]) -> usize {
    // Conjugate each rotation's Z through the SUFFIX Clifford by running a
    // tableau forward from that point; the resulting Pauli's (x|z) bits are
    // the row.
    let mut rows: Vec<Vec<u64>> = Vec::new();
    let words = (2 * n).div_ceil(64);
    for (i, g) in surface.iter().enumerate() {
        let q = match g {
            Surface::Face(_, q) | Surface::Rot(q) => *q,
            _ => continue,
        };
        // Track Z_q through the remaining Clifford gates via the tableau's
        // own conjugation, using a single-row Pauli frame.
        let mut x = vec![false; n];
        let mut z = vec![false; n];
        z[q] = true;
        for later in &surface[i + 1..] {
            match later {
                Surface::H(t) => {
                    let (a, b) = (x[*t], z[*t]);
                    x[*t] = b;
                    z[*t] = a;
                }
                Surface::S(t) | Surface::Sdg(t) => {
                    if x[*t] {
                        z[*t] = !z[*t];
                    }
                }
                Surface::Cx(c, t) => {
                    if x[*c] {
                        x[*t] = !x[*t];
                    }
                    if z[*t] {
                        z[*c] = !z[*c];
                    }
                }
                Surface::Cz(a, b) => {
                    if x[*a] {
                        z[*b] = !z[*b];
                    }
                    if x[*b] {
                        z[*a] = !z[*a];
                    }
                }
                _ => {}
            }
        }
        let mut row = vec![0u64; words];
        for k in 0..n {
            if x[k] {
                row[k >> 6] |= 1 << (k & 63);
            }
            if z[k] {
                let b = n + k;
                row[b >> 6] |= 1 << (b & 63);
            }
        }
        rows.push(row);
    }
    // F₂ Gaussian elimination
    let mut rank = 0usize;
    for col in 0..2 * n {
        let (w, b) = (col >> 6, col & 63);
        if let Some(p) = (rank..rows.len()).find(|&i| rows[i][w] >> b & 1 == 1) {
            rows.swap(rank, p);
            let pivot = rows[rank].clone();
            for i in 0..rows.len() {
                if i != rank && rows[i][w] >> b & 1 == 1 {
                    for (t, s) in rows[i].iter_mut().zip(pivot.iter()) {
                        *t ^= *s;
                    }
                }
            }
            rank += 1;
        }
    }
    rank
}

/// The four CLIFFORD points of the polynomial, each computed in polynomial
/// time with no branching: `z ∈ {1, i, −1, −i}` makes every rotation
/// `{I, S, Z, S†}`, so the whole circuit is Clifford.
///
/// Returns `[(z, ⟨y|C|0⟩)]` — exact `Cyc` values, no expansion anywhere.
pub fn clifford_points(n: usize, surface: &[Surface], y: &[bool]) -> [(Cyc, Cyc); 4] {
    let subs: [(Cyc, Option<Gate>); 4] = [
        (Cyc::ONE, None),                                    // z = 1  → I
        (crate::affine::omega_pow(2), Some(Gate::S(0))),     // z = i  → S
        (Cyc { c: [-1, 0, 0, 0], m: 0 }, Some(Gate::Z(0))),  // z = −1 → Z
        (crate::affine::omega_pow(6), Some(Gate::Sdg(0))),   // z = −i → S†
    ];
    let mut out = [(Cyc::ZERO, Cyc::ZERO); 4];
    for (k, (z, sub)) in subs.iter().enumerate() {
        let mut st = Affine::new(n);
        let mut scalar = Cyc::ONE;
        for &g in surface {
            match g {
                Surface::Face(_, q) | Surface::Rot(q) => {
                    match sub {
                        Some(Gate::S(_)) => st.apply(Gate::S(q)),
                        Some(Gate::Sdg(_)) => st.apply(Gate::Sdg(q)),
                        Some(Gate::Z(_)) => st.apply(Gate::Z(q)),
                        Some(_) | None => {}
                    }
                }
                other => {
                    let (core, phase16) = crate::qasm::lower(&[other]);
                    for cg in &core {
                        st.apply(*cg);
                    }
                    for _ in 0..(phase16.rem_euclid(16) / 2) {
                        scalar = scalar.mul(crate::affine::omega_pow(1));
                    }
                }
            }
        }
        let g = st.canonicalize();
        let amp = st.amplitude(y);
        out[k] = (*z, scalar.mul(g).mul(amp));
    }
    out
}

/// THE CERTIFICATE: an expensively-computed polynomial must reproduce all
/// four poly-time Clifford points. Cheap to check, impossible to fake.
pub fn certify_at_clifford_points(
    n: usize,
    surface: &[Surface],
    y: &[bool],
    poly: &Poly,
) -> Result<(), String> {
    for (z, want) in clifford_points(n, surface, y) {
        let got = poly.eval_r3(R3::from_cyc(z));
        let w = R3::from_cyc(want);
        let (gr, gi) = got.to_complex();
        let (wr, wi) = w.to_complex();
        if (gr - wr).abs() > 1e-9 || (gi - wi).abs() > 1e-9 {
            return Err(format!(
                "Clifford-point certificate FAILED at z={:?}: polynomial gives {:?}, \
                 the poly-time tableau gives {:?}",
                z.to_complex(),
                (gr, gi),
                (wr, wi)
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod factoring_tests {
    use super::*;
    use crate::qasm::Surface::*;

    /// The rank bounds the expansion, and the engine's merge REALIZES it:
    /// many rotations on few qubits must collapse to at most 2^rank.
    #[test]
    fn rank_bounds_the_realized_branch_count() {
        // 8 rotations on 2 qubits: rank ≤ 4, so ≤ 16 branches survive
        let mut prog = vec![H(0), H(1), Cx(0, 1)];
        for i in 0..8 {
            prog.push(Rot(i % 2));
            prog.push(if i % 2 == 0 { S(0) } else { H(1) });
        }
        let r = rotation_rank(2, &prog);
        assert!(r <= 4, "rank on 2 qubits cannot exceed 2n = 4, got {r}");
        let sum_branches = {
            let (branches, _) = expand_poly(2, &prog, 1);
            branches.len()
        };
        assert!(
            sum_branches <= 1 << r,
            "realized branches {sum_branches} must be ≤ 2^rank = {}",
            1 << r
        );
    }

    /// The four Clifford points are computed WITHOUT branching and must
    /// agree with the exponential polynomial — a poly-time certificate on
    /// an exponential object.
    #[test]
    fn clifford_points_certify_the_polynomial() {
        let progs: Vec<(usize, Vec<Surface>)> = vec![
            (2, vec![H(0), H(1), Cx(0, 1), Rot(0), S(1), Rot(1), H(0)]),
            (2, vec![H(0), Rot(0), Cx(0, 1), Rot(1), Face(1, 0), H(1)]),
            (3, vec![H(0), H(2), Cz(0, 1), Rot(2), Cx(2, 1), Rot(0), Rot(1), H(2)]),
        ];
        for (n, prog) in progs {
            for b in 0..(1u32 << n) {
                let y: Vec<bool> = (0..n).map(|q| b >> q & 1 == 1).collect();
                let (poly, _) = amplitude_poly(n, &prog, &y, 2);
                certify_at_clifford_points(n, &prog, &y, &poly)
                    .unwrap_or_else(|e| panic!("n={n} basis={b}: {e}"));
            }
        }
    }

    /// A degree-≤3 polynomial is DETERMINED by the four Clifford points, so
    /// for such circuits the exponential is not merely certified but
    /// avoidable: the poly-time skeleton is the whole answer.
    #[test]
    fn low_degree_is_fully_determined_by_the_skeleton() {
        let n = 2;
        let prog = vec![H(0), Rot(0), Cx(0, 1), Rot(1), H(1)];
        for b in 0..4u32 {
            let y: Vec<bool> = (0..n).map(|q| b >> q & 1 == 1).collect();
            let (poly, _) = amplitude_poly(n, &prog, &y, 1);
            assert!(poly.degree() <= 3, "degree {} exceeds the skeleton", poly.degree());
            certify_at_clifford_points(n, &prog, &y, &poly).unwrap();
        }
    }
}
