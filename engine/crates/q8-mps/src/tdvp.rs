//! C2 — REAL-TIME MPS ELECTRONIC DYNAMICS (WB-8.3's third carrier).
//!
//! Single-site TDVP (the Haegeman–Lubich–Oseledets–Vandereycken–Verstraete projector
//! splitting) for an arbitrary `mpo::Mpo`, in complex arithmetic. This is where the
//! tower's horizontal axis stops being ground states and becomes DYNAMICS: `dmrg.rs`
//! minimises `<psi|H|psi>`, this module integrates `i d|psi>/dt = H|psi>` on the MPS
//! manifold.
//!
//! # The integrator, stated once
//!
//! One `step(dt)` is a palindrome — forward half-sweep, backward half-sweep — which is
//! what makes it second order:
//!
//! ```text
//!   forward   j = 0 .. L-1 :  A_j <- exp(-i (dt/2) H1_j) A_j
//!                             QR split, absorb the left env
//!                             C   <- exp(+i (dt/2) H0_j) C        <- BACKWARD on the bond
//!                             C into A_{j+1}
//!   backward  j = L-1 .. 0 :  the same, mirrored
//! ```
//!
//! Each site is evolved `dt/2` in each direction (total `dt`); each BOND is evolved
//! `-dt/2` twice (total `-dt`). The minus sign is not a convention, it is the tangent-space
//! projector `P_T = sum_j P_j - sum_bonds P_bond` — drop it or flip it and the integrator
//! is wrong in a way the energy gate sees immediately, which is why both are planted
//! mutations below.
//!
//! # What is exact and what is not
//!
//! * **The norm is exact.** Every substep applies `exp(-i theta H_eff)` with `H_eff`
//!   Hermitian to the orthogonality centre of a canonical MPS: unitary on the centre,
//!   identity on everything else.
//! * **The energy is exact.** At the centre, `<psi|H|psi> = <A|H_eff|A>`, and
//!   `exp(-i theta H_eff)` preserves that. The QR splits do not change the state at all.
//!   So energy drift is a BUG DETECTOR, not a discretisation error — `G-C2-1` gates it
//!   at `1e-10`, and it is the gate that catches an environment built with the wrong
//!   conjugation.
//! * **At the natural bond cap the trajectory is EXACT, not merely second order.** This
//!   was staked wrong on the first pass and the gate corrected it: the projector-splitting
//!   integrator has the EXACTNESS PROPERTY (Lubich–Oseledets 2014 for matrices;
//!   Lubich–Vandereycken–Walach and Haegeman et al. 2016 for tensor networks) — when the
//!   manifold contains the exact trajectory, the splitting reproduces it with no
//!   step-size error at all. At `chi = natural cap` the manifold IS the whole Hilbert
//!   space, so `G-C2-2a` gates the trajectory against the dense propagator at `1e-11`
//!   INDEPENDENT of `dt`, measured `~3e-13` across a factor of eight in step size.
//! * **Below the cap it is second order**, and that is where the integrator's own order
//!   can be read: on a genuine submanifold the flow is the TDVP flow rather than the
//!   Schrödinger flow, so `G-C2-2b` measures self-convergence against a fine-step
//!   reference. This is also the only gate that separates the symmetric sweep from the
//!   one-directional one — at the cap BOTH are exact, so the palindrome looks free there.
//!
//! # THE FENCE (stated because it is a real limitation, not a caveat)
//!
//! Single-site TDVP **cannot grow a bond dimension**. The tangent space at `A` only
//! contains directions the existing bond can represent, so a rank-deficient start — a
//! product state zero-padded up to `chi` — stays rank-deficient forever and the effective
//! Hamiltonian is singular on the padding. `deterministic_state` therefore builds a
//! FULL-RANK start and `pad_to_chi`-style zero padding is deliberately NOT offered here.
//! Two-site TDVP (which can grow bonds, at the price of a truncation) is the discharge
//! route and is not built: it is a visible fence, in the `Capability::Stub` sense, and
//! `holon-chem`'s C2 carrier names it.
//!
//! Credits: Haegeman et al. PRL 107 070601 (2011) and PRB 94 165116 (2016) for the
//! integrator; Lubich–Oseledets–Vandereycken for the splitting's unconditional stability;
//! Paeckel et al. Ann. Phys. 411 167998 (2019) for the review whose §5 conventions this
//! follows. The exact referee here is dense diagonalisation, in-engine.
//!
//! Zero runtime dependencies, like the rest of the crate: the complex scalar, the
//! Gram–Schmidt QR and the Krylov exponential are all written here.

use crate::eigen::jacobi_eigen;
use crate::mpo::{Mpo, MpoSite};

// ---------------------------------------------------------------- complex scalar

/// A complex number. Written here rather than pulled in: this crate's certificate must
/// not rest on a crate the repository does not audit (`Cargo.toml`'s zero-dependency
/// clause is load-bearing, not tidy).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cx {
    pub re: f64,
    pub im: f64,
}

impl Cx {
    pub const ZERO: Cx = Cx { re: 0.0, im: 0.0 };
    pub const ONE: Cx = Cx { re: 1.0, im: 0.0 };

    #[inline]
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// `exp(i theta)`.
    #[inline]
    pub fn cis(theta: f64) -> Self {
        Self { re: theta.cos(), im: theta.sin() }
    }

    #[inline]
    pub fn conj(self) -> Self {
        Self { re: self.re, im: -self.im }
    }

    #[inline]
    pub fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    #[inline]
    pub fn abs(self) -> f64 {
        self.norm_sq().sqrt()
    }

    #[inline]
    pub fn add(self, o: Self) -> Self {
        Self { re: self.re + o.re, im: self.im + o.im }
    }

    #[inline]
    pub fn sub(self, o: Self) -> Self {
        Self { re: self.re - o.re, im: self.im - o.im }
    }

    #[inline]
    pub fn mul(self, o: Self) -> Self {
        Self {
            re: self.re * o.re - self.im * o.im,
            im: self.re * o.im + self.im * o.re,
        }
    }

    #[inline]
    pub fn scale(self, f: f64) -> Self {
        Self { re: self.re * f, im: self.im * f }
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.re.is_finite() && self.im.is_finite()
    }
}

/// `sum_i conj(a_i) b_i` — the Hermitian inner product, conjugate-linear in the FIRST
/// argument. Getting this backwards is invisible on real data and fatal on complex, so
/// every use in this file goes through this one function.
#[inline]
pub fn cdot(a: &[Cx], b: &[Cx]) -> Cx {
    let mut acc = Cx::ZERO;
    for (x, y) in a.iter().zip(b.iter()) {
        acc = acc.add(x.conj().mul(*y));
    }
    acc
}

#[inline]
pub fn cnorm(a: &[Cx]) -> f64 {
    a.iter().map(|x| x.norm_sq()).sum::<f64>().sqrt()
}

/// `y <- y + alpha * x`.
#[inline]
fn axpy(y: &mut [Cx], alpha: Cx, x: &[Cx]) {
    for (yi, xi) in y.iter_mut().zip(x.iter()) {
        *yi = yi.add(alpha.mul(*xi));
    }
}

// ---------------------------------------------------------------- tensors

/// One complex MPS site tensor, physical dimension 2, flat `[s][l][r]` layout — the same
/// index order as `mps::TensorSite`, so a real ground state converts in place.
#[derive(Clone, Debug)]
pub struct CTensorSite {
    pub chi_l: usize,
    pub chi_r: usize,
    pub data: Vec<Cx>,
}

impl CTensorSite {
    pub fn zeros(chi_l: usize, chi_r: usize) -> Self {
        Self { chi_l, chi_r, data: vec![Cx::ZERO; 2 * chi_l * chi_r] }
    }

    #[inline]
    pub fn get(&self, s: usize, l: usize, r: usize) -> Cx {
        self.data[(s * self.chi_l + l) * self.chi_r + r]
    }

    #[inline]
    pub fn set(&mut self, s: usize, l: usize, r: usize, v: Cx) {
        self.data[(s * self.chi_l + l) * self.chi_r + r] = v;
    }

    /// Lift a real site tensor (e.g. a converged DMRG ground state) into the complex
    /// carrier. This is the C2 half of a picture change: the state is the same state,
    /// re-expressed where a phase can move.
    pub fn from_real(t: &crate::mps::TensorSite) -> Self {
        Self {
            chi_l: t.chi_l,
            chi_r: t.chi_r,
            data: t.data.iter().map(|&x| Cx::new(x, 0.0)).collect(),
        }
    }
}

/// One MPO environment block: `env[channel][a * chi + b]`, `a` the BRA index and `b` the
/// KET index. Each channel block is Hermitian; `H_eff` inherits its Hermiticity from that
/// and from the surrounding tensors being canonical.
pub type CEnv = Vec<Vec<Cx>>;

/// The boundary environment for an MPO bond of dimension `d` (both chain ends have `d = 1`
/// for every MPO this crate builds).
pub fn trivial_env(d: usize) -> CEnv {
    let mut e: CEnv = (0..d).map(|_| vec![Cx::ZERO]).collect();
    if d > 0 {
        e[0][0] = Cx::ONE;
    }
    e
}

// ---------------------------------------------------------------- environments

/// Absorb one site into a LEFT environment:
/// `L'[c2][a'][b'] = sum conj(A[s][a][a']) L[c1][a][b] W[c1,c2,s,sp] A[sp][b][b']`.
pub fn grow_left(l_env: &CEnv, w: &MpoSite, a: &CTensorSite) -> CEnv {
    grow_left_impl(l_env, w, a, true)
}

/// `grow_left` with the bra conjugation switchable. `conjugate = false` is the
/// GATE-ONLY planted defect `Mutation::LeftEnvNoConjugate`: on real tensors it is
/// invisible, on complex ones it silently makes `H_eff` non-Hermitian, and the energy
/// gate is what sees it. Nothing but the mutation path passes `false`.
fn grow_left_impl(l_env: &CEnv, w: &MpoSite, a: &CTensorSite, conjugate: bool) -> CEnv {
    let (d_l, d_r) = (w.d_l, w.d_r);
    let (chi_l, chi_r) = (a.chi_l, a.chi_r);
    debug_assert_eq!(l_env.len(), d_l);

    // Stage A: t1[c1][s][ap][b] = sum_a conj(A[s][a][ap]) L[c1][a][b]
    let mut t1 = vec![Cx::ZERO; d_l * 2 * chi_r * chi_l];
    for c1 in 0..d_l {
        let lm = &l_env[c1];
        for s in 0..2 {
            for a_idx in 0..chi_l {
                for ap in 0..chi_r {
                    let raw = a.get(s, a_idx, ap);
                    let av = if conjugate { raw.conj() } else { raw };
                    if av == Cx::ZERO {
                        continue;
                    }
                    let base = ((c1 * 2 + s) * chi_r + ap) * chi_l;
                    let lrow = a_idx * chi_l;
                    for b in 0..chi_l {
                        t1[base + b] = t1[base + b].add(av.mul(lm[lrow + b]));
                    }
                }
            }
        }
    }

    // Stage B: t2[c2][sp][ap][b] = sum_{c1,s} t1[c1][s][ap][b] W[c1,c2,s,sp]
    let block = chi_r * chi_l;
    let mut t2 = vec![Cx::ZERO; d_r * 2 * block];
    for c1 in 0..d_l {
        for c2 in 0..d_r {
            for s in 0..2 {
                for sp in 0..2 {
                    let wv = w.get(c1, c2, s, sp);
                    if wv == 0.0 {
                        continue;
                    }
                    let src = (c1 * 2 + s) * block;
                    let dst = (c2 * 2 + sp) * block;
                    for idx in 0..block {
                        t2[dst + idx] = t2[dst + idx].add(t1[src + idx].scale(wv));
                    }
                }
            }
        }
    }

    // Stage C: L'[c2][ap][bp] = sum_{sp,b} t2[c2][sp][ap][b] A[sp][b][bp]
    let mut out: CEnv = (0..d_r).map(|_| vec![Cx::ZERO; chi_r * chi_r]).collect();
    for c2 in 0..d_r {
        for sp in 0..2 {
            for ap in 0..chi_r {
                let base = ((c2 * 2 + sp) * chi_r + ap) * chi_l;
                for b in 0..chi_l {
                    let tv = t2[base + b];
                    if tv == Cx::ZERO {
                        continue;
                    }
                    let row = &mut out[c2][ap * chi_r..ap * chi_r + chi_r];
                    for (bp, cell) in row.iter_mut().enumerate() {
                        *cell = cell.add(tv.mul(a.get(sp, b, bp)));
                    }
                }
            }
        }
    }
    out
}

/// Absorb one site into a RIGHT environment:
/// `R'[c1][a][b] = sum conj(A[s][a][a']) W[c1,c2,s,sp] A[sp][b][b'] R[c2][a'][b']`.
pub fn grow_right(r_env: &CEnv, w: &MpoSite, a: &CTensorSite) -> CEnv {
    let (d_l, d_r) = (w.d_l, w.d_r);
    let (chi_l, chi_r) = (a.chi_l, a.chi_r);
    debug_assert_eq!(r_env.len(), d_r);

    // Stage A: t1[c2][sp][b][ap] = sum_{bp} A[sp][b][bp] R[c2][ap][bp]
    let mut t1 = vec![Cx::ZERO; d_r * 2 * chi_l * chi_r];
    for c2 in 0..d_r {
        let rm = &r_env[c2];
        for sp in 0..2 {
            for b in 0..chi_l {
                let base = ((c2 * 2 + sp) * chi_l + b) * chi_r;
                for bp in 0..chi_r {
                    let av = a.get(sp, b, bp);
                    if av == Cx::ZERO {
                        continue;
                    }
                    for ap in 0..chi_r {
                        t1[base + ap] = t1[base + ap].add(av.mul(rm[ap * chi_r + bp]));
                    }
                }
            }
        }
    }

    // Stage B: t2[c1][s][b][ap] = sum_{c2,sp} t1[c2][sp][b][ap] W[c1,c2,s,sp]
    let block = chi_l * chi_r;
    let mut t2 = vec![Cx::ZERO; d_l * 2 * block];
    for c1 in 0..d_l {
        for c2 in 0..d_r {
            for s in 0..2 {
                for sp in 0..2 {
                    let wv = w.get(c1, c2, s, sp);
                    if wv == 0.0 {
                        continue;
                    }
                    let src = (c2 * 2 + sp) * block;
                    let dst = (c1 * 2 + s) * block;
                    for idx in 0..block {
                        t2[dst + idx] = t2[dst + idx].add(t1[src + idx].scale(wv));
                    }
                }
            }
        }
    }

    // Stage C: R'[c1][a][b] = sum_{s,ap} conj(A[s][a][ap]) t2[c1][s][b][ap]
    let mut out: CEnv = (0..d_l).map(|_| vec![Cx::ZERO; chi_l * chi_l]).collect();
    for c1 in 0..d_l {
        for s in 0..2 {
            for a_idx in 0..chi_l {
                for ap in 0..chi_r {
                    let av = a.get(s, a_idx, ap).conj();
                    if av == Cx::ZERO {
                        continue;
                    }
                    let base = ((c1 * 2 + s) * chi_l) * chi_r + ap;
                    for b in 0..chi_l {
                        let tv = t2[base + b * chi_r];
                        out[c1][a_idx * chi_l + b] = out[c1][a_idx * chi_l + b].add(av.mul(tv));
                    }
                }
            }
        }
    }
    out
}

/// The SINGLE-SITE effective Hamiltonian applied to a site tensor `psi` (flat `[s][a][a']`):
/// `out[s][a][a'] = sum L[c1][a][b] W[c1,c2,s,sp] psi[sp][b][b'] R[c2][a'][b']`.
pub fn apply_h1(
    left: &CEnv,
    w: &MpoSite,
    right: &CEnv,
    psi: &[Cx],
    chi_l: usize,
    chi_r: usize,
) -> Vec<Cx> {
    let (d_l, d_r) = (w.d_l, w.d_r);

    // Step 1: t1[c2][sp][b][ap] = sum_{bp} psi[sp][b][bp] R[c2][ap][bp]
    let mut t1 = vec![Cx::ZERO; d_r * 2 * chi_l * chi_r];
    for c2 in 0..d_r {
        let rm = &right[c2];
        for sp in 0..2 {
            for b in 0..chi_l {
                let src = (sp * chi_l + b) * chi_r;
                let dst = ((c2 * 2 + sp) * chi_l + b) * chi_r;
                for ap in 0..chi_r {
                    let rrow = ap * chi_r;
                    let mut acc = Cx::ZERO;
                    for bp in 0..chi_r {
                        acc = acc.add(psi[src + bp].mul(rm[rrow + bp]));
                    }
                    t1[dst + ap] = acc;
                }
            }
        }
    }

    // Step 2: t2[c1][s][b][ap] = sum_{c2,sp} t1[c2][sp][b][ap] W[c1,c2,s,sp]
    let block = chi_l * chi_r;
    let mut t2 = vec![Cx::ZERO; d_l * 2 * block];
    for c1 in 0..d_l {
        for c2 in 0..d_r {
            for s in 0..2 {
                for sp in 0..2 {
                    let wv = w.get(c1, c2, s, sp);
                    if wv == 0.0 {
                        continue;
                    }
                    let src = (c2 * 2 + sp) * block;
                    let dst = (c1 * 2 + s) * block;
                    for idx in 0..block {
                        t2[dst + idx] = t2[dst + idx].add(t1[src + idx].scale(wv));
                    }
                }
            }
        }
    }

    // Step 3: out[s][a][ap] = sum_{c1,b} L[c1][a][b] t2[c1][s][b][ap]
    let mut out = vec![Cx::ZERO; 2 * chi_l * chi_r];
    for c1 in 0..d_l {
        let lm = &left[c1];
        for s in 0..2 {
            for a_idx in 0..chi_l {
                let lrow = a_idx * chi_l;
                let dst = (s * chi_l + a_idx) * chi_r;
                for b in 0..chi_l {
                    let lv = lm[lrow + b];
                    if lv == Cx::ZERO {
                        continue;
                    }
                    let src = ((c1 * 2 + s) * chi_l + b) * chi_r;
                    for ap in 0..chi_r {
                        out[dst + ap] = out[dst + ap].add(lv.mul(t2[src + ap]));
                    }
                }
            }
        }
    }
    out
}

/// The ZERO-SITE effective Hamiltonian applied to a bond tensor `c` (flat `[b][b']`,
/// `cl x cr`): `out[a][a'] = sum_{ch,b,b'} L[ch][a][b] c[b][b'] R[ch][a'][b']`.
///
/// This is the operator the palindrome runs BACKWARD. `L` and `R` must be the two
/// environments meeting at the SAME bond, so their channel dimension agrees.
pub fn apply_h0(left: &CEnv, right: &CEnv, c: &[Cx], cl: usize, cr: usize) -> Vec<Cx> {
    debug_assert_eq!(left.len(), right.len());
    let d = left.len();

    // Step 1: t1[ch][b][ap] = sum_{bp} c[b][bp] R[ch][ap][bp]
    let mut t1 = vec![Cx::ZERO; d * cl * cr];
    for ch in 0..d {
        let rm = &right[ch];
        for b in 0..cl {
            let src = b * cr;
            let dst = (ch * cl + b) * cr;
            for ap in 0..cr {
                let rrow = ap * cr;
                let mut acc = Cx::ZERO;
                for bp in 0..cr {
                    acc = acc.add(c[src + bp].mul(rm[rrow + bp]));
                }
                t1[dst + ap] = acc;
            }
        }
    }

    // Step 2: out[a][ap] = sum_{ch,b} L[ch][a][b] t1[ch][b][ap]
    let mut out = vec![Cx::ZERO; cl * cr];
    for ch in 0..d {
        let lm = &left[ch];
        for a_idx in 0..cl {
            let lrow = a_idx * cl;
            let dst = a_idx * cr;
            for b in 0..cl {
                let lv = lm[lrow + b];
                if lv == Cx::ZERO {
                    continue;
                }
                let src = (ch * cl + b) * cr;
                for ap in 0..cr {
                    out[dst + ap] = out[dst + ap].add(lv.mul(t1[src + ap]));
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------- QR

const QR_RANK_TOL: f64 = 1e-13;

/// Thin QR of a row-major `m x n` complex matrix by modified Gram–Schmidt with one full
/// reorthogonalisation pass (the classical MGS loses orthogonality at
/// `O(eps * cond(A))`; the second pass restores it to `O(eps)`, which is the standard
/// twice-is-enough result and is cheap at these sizes).
///
/// Returns `(q, r, k)`: `q` is `m x k` with `q^H q = I_k`, `r` is `k x n`, and `A = q r`
/// EXACTLY up to round-off. `k` is the numerical rank, so a bond that has genuinely
/// collapsed shrinks rather than carrying a null direction — single-site TDVP can never
/// grow it back, and that is the fence stated in this module's header, not a silent loss.
pub fn thin_qr(a: &[Cx], m: usize, n: usize) -> (Vec<Cx>, Vec<Cx>, usize) {
    let kmax = m.min(n);
    let mut basis: Vec<Vec<Cx>> = Vec::with_capacity(kmax);
    // r is built column by column into a dense kmax x n scratch, trimmed at the end.
    let mut r = vec![Cx::ZERO; kmax * n];

    for j in 0..n {
        let mut v: Vec<Cx> = (0..m).map(|i| a[i * n + j]).collect();
        for _pass in 0..2 {
            for (i, q) in basis.iter().enumerate() {
                let c = cdot(q, &v);
                if c == Cx::ZERO {
                    continue;
                }
                r[i * n + j] = r[i * n + j].add(c);
                axpy(&mut v, c.scale(-1.0), q);
            }
        }
        let nv = cnorm(&v);
        if basis.len() < kmax && nv > QR_RANK_TOL {
            let inv = 1.0 / nv;
            for x in v.iter_mut() {
                *x = x.scale(inv);
            }
            r[basis.len() * n + j] = Cx::new(nv, 0.0);
            basis.push(v);
        }
    }

    let k = basis.len();
    let mut q = vec![Cx::ZERO; m * k];
    for (i, col) in basis.iter().enumerate() {
        for (row, &val) in col.iter().enumerate() {
            q[row * k + i] = val;
        }
    }
    let r_trim = r[..k * n].to_vec();
    (q, r_trim, k)
}

/// Split the orthogonality centre LEFT-canonically: `A = Q C` with `Q^H Q = I`.
/// Returns `(Q, C, k)` with `C` row-major `k x chi_r`.
pub fn split_left(a: &CTensorSite) -> (CTensorSite, Vec<Cx>, usize) {
    // `a.data` is already row-major `(2*chi_l) x chi_r` with row index `s*chi_l + l`.
    let m = 2 * a.chi_l;
    let (q, c, k) = thin_qr(&a.data, m, a.chi_r);
    let out = CTensorSite { chi_l: a.chi_l, chi_r: k, data: q };
    (out, c, k)
}

/// Split the orthogonality centre RIGHT-canonically: `A = C Q` with `Q Q^H = I`.
/// Returns `(C, Q, k)` with `C` row-major `chi_l x k`.
pub fn split_right(a: &CTensorSite) -> (Vec<Cx>, CTensorSite, usize) {
    let (chi_l, chi_r) = (a.chi_l, a.chi_r);
    let cols = 2 * chi_r;
    // N[l][s*chi_r + r] = A[s][l][r]; QR is applied to N^H, and the result transposed
    // back, because one QR routine is easier to keep correct than two.
    let mut nh = vec![Cx::ZERO; cols * chi_l];
    for s in 0..2 {
        for l in 0..chi_l {
            for r in 0..chi_r {
                nh[(s * chi_r + r) * chi_l + l] = a.get(s, l, r).conj();
            }
        }
    }
    let (qt, rt, k) = thin_qr(&nh, cols, chi_l);
    // Q = qt^H : k x cols ; C = rt^H : chi_l x k
    let mut out = CTensorSite::zeros(k, chi_r);
    for s in 0..2 {
        for i in 0..k {
            for r in 0..chi_r {
                out.set(s, i, r, qt[(s * chi_r + r) * k + i].conj());
            }
        }
    }
    let mut c = vec![Cx::ZERO; chi_l * k];
    for i in 0..k {
        for l in 0..chi_l {
            c[l * k + i] = rt[i * chi_l + l].conj();
        }
    }
    (c, out, k)
}

/// `A[j+1] <- C . A[j+1]`, contracting the bond matrix `C` (`k x chi_l`) into the next
/// site's left index.
fn absorb_left(c: &[Cx], k: usize, a: &CTensorSite) -> CTensorSite {
    let mut out = CTensorSite::zeros(k, a.chi_r);
    for s in 0..2 {
        for i in 0..k {
            for b in 0..a.chi_l {
                let cv = c[i * a.chi_l + b];
                if cv == Cx::ZERO {
                    continue;
                }
                for r in 0..a.chi_r {
                    let cur = out.get(s, i, r);
                    out.set(s, i, r, cur.add(cv.mul(a.get(s, b, r))));
                }
            }
        }
    }
    out
}

/// `A[j-1] <- A[j-1] . C`, contracting the bond matrix `C` (`chi_r x k`) into the previous
/// site's right index.
fn absorb_right(a: &CTensorSite, c: &[Cx], k: usize) -> CTensorSite {
    let mut out = CTensorSite::zeros(a.chi_l, k);
    for s in 0..2 {
        for l in 0..a.chi_l {
            for r in 0..a.chi_r {
                let av = a.get(s, l, r);
                if av == Cx::ZERO {
                    continue;
                }
                for i in 0..k {
                    let cur = out.get(s, l, i);
                    out.set(s, l, i, cur.add(av.mul(c[r * k + i])));
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------- Krylov exponential

/// What the Krylov exponential did, reported rather than assumed.
#[derive(Clone, Copy, Debug)]
pub struct KrylovReport {
    pub iterations: usize,
    /// The standard `beta_m |c_m|` a-posteriori estimate at the accepted subspace size.
    pub estimate: f64,
    /// True when the subspace ran out before the estimate met the tolerance.
    pub truncated: bool,
}

/// `exp(-i theta H) v`, by Lanczos on the Krylov space of `apply`.
///
/// `H` is real-symmetric here (an MPO built from real integrals), and `v` is complex, so
/// the Lanczos coefficients are real and the projected problem is a small real symmetric
/// tridiagonal — `eigen::jacobi_eigen` solves it. Full reorthogonalisation, same as
/// `lanczos.rs`: the subspaces are tiny and losing orthogonality here would show up as
/// energy drift, which is the one thing `G-C2-1` must not tolerate.
pub fn expm_krylov<F>(apply: &F, v: &[Cx], theta: f64, m_max: usize, tol: f64) -> (Vec<Cx>, KrylovReport)
where
    F: Fn(&[Cx]) -> Vec<Cx>,
{
    let n = v.len();
    let nrm = cnorm(v);
    if nrm == 0.0 || n == 0 {
        return (v.to_vec(), KrylovReport { iterations: 0, estimate: 0.0, truncated: false });
    }
    let inv = 1.0 / nrm;
    let mut basis: Vec<Vec<Cx>> = vec![v.iter().map(|x| x.scale(inv)).collect()];
    let mut alpha: Vec<f64> = Vec::new();
    let mut beta: Vec<f64> = Vec::new();

    let m_cap = m_max.min(n);
    let mut coeffs: Vec<Cx> = vec![Cx::ONE];
    let mut iterations = 0;
    let mut estimate = 0.0;
    let mut truncated = false;

    for j in 0..m_cap {
        iterations = j + 1;
        let mut w = apply(&basis[j]);
        let a = cdot(&basis[j], &w).re;
        alpha.push(a);
        axpy(&mut w, Cx::new(-a, 0.0), &basis[j]);
        if j > 0 {
            axpy(&mut w, Cx::new(-beta[j - 1], 0.0), &basis[j - 1]);
        }
        for _pass in 0..2 {
            for u in basis.iter() {
                let c = cdot(u, &w);
                axpy(&mut w, c.scale(-1.0), u);
            }
        }
        let b = cnorm(&w);

        // Project: exp(-i theta T) e_1 with T the real symmetric tridiagonal.
        let m = alpha.len();
        let mut tri = vec![0.0; m * m];
        for i in 0..m {
            tri[i * m + i] = alpha[i];
            if i + 1 < m {
                tri[i * m + i + 1] = beta[i];
                tri[(i + 1) * m + i] = beta[i];
            }
        }
        let eig = jacobi_eigen(tri, m);
        coeffs = vec![Cx::ZERO; m];
        for i in 0..m {
            let phase = Cx::cis(-theta * eig.values[i]);
            let first = eig.vectors[i][0];
            if first == 0.0 {
                continue;
            }
            let w_i = phase.scale(first);
            for (k, ck) in coeffs.iter_mut().enumerate() {
                *ck = ck.add(w_i.scale(eig.vectors[i][k]));
            }
        }

        estimate = b * coeffs[m - 1].abs();
        let converged = estimate <= tol;
        let breakdown = b <= 1e-14;
        if converged || breakdown || j == m_cap - 1 {
            truncated = !(converged || breakdown);
            break;
        }
        beta.push(b);
        let binv = 1.0 / b;
        for x in w.iter_mut() {
            *x = x.scale(binv);
        }
        basis.push(w);
    }

    let mut out = vec![Cx::ZERO; n];
    for (k, &ck) in coeffs.iter().enumerate() {
        axpy(&mut out, ck.scale(nrm), &basis[k]);
    }
    (out, KrylovReport { iterations, estimate, truncated })
}

// ---------------------------------------------------------------- the integrator

/// GATE-ONLY planted defects. Each one is a real bug someone writes when implementing
/// this integrator, and each must be caught by a staked gate: a gate that has never
/// failed has never gated (`tests/c2_tdvp_gates.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mutation {
    /// The integrator as specified.
    #[default]
    None,
    /// Skip the zero-site bond evolution entirely — "the projector is just the sum over
    /// sites". Wrong: `P_T = sum_sites - sum_bonds`.
    NoBondBackstep,
    /// Evolve the bond FORWARD instead of backward: the sign of the subtracted term.
    BondBackstepWrongSign,
    /// Forward half-sweep only, at full `dt`. Loses the palindrome, so second order
    /// collapses to first.
    ForwardSweepOnly,
    /// Build the left environment WITHOUT conjugating the bra tensor. Invisible on real
    /// arithmetic — which is exactly why it is planted: it is the defect a real-valued
    /// test suite cannot see, and it makes `H_eff` non-Hermitian so the energy stops
    /// being conserved.
    LeftEnvNoConjugate,
}

/// A single-site TDVP integrator bound to one MPO.
pub struct Tdvp<'a> {
    mpo: &'a Mpo,
    pub tensors: Vec<CTensorSite>,
    /// `left[j]` covers sites `0..j`; `left[0]` is the boundary.
    left: Vec<CEnv>,
    /// `right[j]` covers sites `j..L`; `right[L]` is the boundary.
    right: Vec<CEnv>,
    pub krylov_dim: usize,
    pub krylov_tol: f64,
    pub mutation: Mutation,
    /// Worst Krylov a-posteriori estimate seen so far, so a trajectory can report the
    /// local solver's own accuracy rather than have it assumed.
    pub worst_krylov_estimate: f64,
    /// Number of Krylov solves that hit the subspace cap without converging.
    pub krylov_truncations: usize,
}

impl<'a> Tdvp<'a> {
    /// Build an integrator around a RIGHT-CANONICAL MPS (orthogonality centre at site 0).
    /// `right_canonicalise` produces one; passing anything else is a programming error the
    /// energy gate will find immediately.
    pub fn new(mpo: &'a Mpo, tensors: Vec<CTensorSite>) -> Self {
        let l = mpo.sites.len();
        assert_eq!(tensors.len(), l, "MPS length must match the MPO");
        assert!(l > 0, "an empty MPO has no dynamics");
        let mut right: Vec<CEnv> = vec![Vec::new(); l + 1];
        right[l] = trivial_env(mpo.sites[l - 1].d_r);
        for j in (0..l).rev() {
            let e = grow_right(&right[j + 1], &mpo.sites[j], &tensors[j]);
            right[j] = e;
        }
        let mut left: Vec<CEnv> = vec![Vec::new(); l + 1];
        left[0] = trivial_env(mpo.sites[0].d_l);
        Self {
            mpo,
            tensors,
            left,
            right,
            krylov_dim: 30,
            krylov_tol: 1e-13,
            mutation: Mutation::None,
            worst_krylov_estimate: 0.0,
            krylov_truncations: 0,
        }
    }

    fn expm(&mut self, apply: impl Fn(&[Cx]) -> Vec<Cx>, v: &[Cx], theta: f64) -> Vec<Cx> {
        let (out, rep) = expm_krylov(&apply, v, theta, self.krylov_dim, self.krylov_tol);
        if rep.estimate > self.worst_krylov_estimate {
            self.worst_krylov_estimate = rep.estimate;
        }
        if rep.truncated {
            self.krylov_truncations += 1;
        }
        out
    }

    /// One symmetric second-order step of size `dt`.
    pub fn step(&mut self, dt: f64) {
        let l = self.tensors.len();
        let bond_sign = match self.mutation {
            Mutation::BondBackstepWrongSign => -1.0,
            _ => 1.0,
        };
        let do_bond = self.mutation != Mutation::NoBondBackstep;
        let forward_only = self.mutation == Mutation::ForwardSweepOnly;
        // The palindrome halves `dt`; a forward-only sweep must cover the full step or it
        // is measuring the wrong integrator for the wrong reason.
        let half = if forward_only { dt } else { 0.5 * dt };

        // ---- forward half-sweep, left to right
        for j in 0..l {
            let (chi_l, chi_r) = (self.tensors[j].chi_l, self.tensors[j].chi_r);
            let psi = self.tensors[j].data.clone();
            let evolved = {
                let lenv = self.left[j].clone();
                let renv = self.right[j + 1].clone();
                let w = self.mpo.sites[j].clone();
                self.expm(
                    move |v: &[Cx]| apply_h1(&lenv, &w, &renv, v, chi_l, chi_r),
                    &psi,
                    half,
                )
            };
            self.tensors[j] = CTensorSite { chi_l, chi_r, data: evolved };

            if j + 1 == l {
                break;
            }
            let (q, c, k) = split_left(&self.tensors[j]);
            self.tensors[j] = q;
            let e = grow_left_impl(
                &self.left[j],
                &self.mpo.sites[j],
                &self.tensors[j],
                self.mutation != Mutation::LeftEnvNoConjugate,
            );
            self.left[j + 1] = e;
            let c = if do_bond {
                let lenv = self.left[j + 1].clone();
                let renv = self.right[j + 1].clone();
                self.expm(
                    move |v: &[Cx]| apply_h0(&lenv, &renv, v, k, chi_r),
                    &c,
                    -half * bond_sign,
                )
            } else {
                c
            };
            self.tensors[j + 1] = absorb_left(&c, k, &self.tensors[j + 1]);
        }

        if forward_only {
            // Restore right-canonicality so the next step starts where it must; the state
            // is unchanged by this, only its gauge.
            self.recanonicalise_right();
            return;
        }

        // ---- backward half-sweep, right to left
        for j in (0..l).rev() {
            let (chi_l, chi_r) = (self.tensors[j].chi_l, self.tensors[j].chi_r);
            let psi = self.tensors[j].data.clone();
            let evolved = {
                let lenv = self.left[j].clone();
                let renv = self.right[j + 1].clone();
                let w = self.mpo.sites[j].clone();
                self.expm(
                    move |v: &[Cx]| apply_h1(&lenv, &w, &renv, v, chi_l, chi_r),
                    &psi,
                    half,
                )
            };
            self.tensors[j] = CTensorSite { chi_l, chi_r, data: evolved };

            if j == 0 {
                break;
            }
            let (c, q, k) = split_right(&self.tensors[j]);
            self.tensors[j] = q;
            let e = grow_right(&self.right[j + 1], &self.mpo.sites[j], &self.tensors[j]);
            self.right[j] = e;
            let c = if do_bond {
                let lenv = self.left[j].clone();
                let renv = self.right[j].clone();
                self.expm(
                    move |v: &[Cx]| apply_h0(&lenv, &renv, v, chi_l, k),
                    &c,
                    -half * bond_sign,
                )
            } else {
                c
            };
            self.tensors[j - 1] = absorb_right(&self.tensors[j - 1], &c, k);
        }
    }

    /// Re-establish right-canonical form (centre at site 0) and rebuild the right
    /// environments. The STATE is unchanged; only its gauge is.
    fn recanonicalise_right(&mut self) {
        let l = self.tensors.len();
        for j in (1..l).rev() {
            let (c, q, k) = split_right(&self.tensors[j]);
            self.tensors[j] = q;
            self.tensors[j - 1] = absorb_right(&self.tensors[j - 1], &c, k);
        }
        self.right[l] = trivial_env(self.mpo.sites[l - 1].d_r);
        for j in (0..l).rev() {
            let e = grow_right(&self.right[j + 1], &self.mpo.sites[j], &self.tensors[j]);
            self.right[j] = e;
        }
    }

    /// `<psi|psi>`, contracted from the tensors themselves rather than assumed from the
    /// canonical form.
    pub fn norm_squared(&self) -> f64 {
        norm_squared(&self.tensors)
    }

    /// `<psi|H|psi> / <psi|psi>`, contracted from scratch through the MPO.
    pub fn energy(&self) -> f64 {
        let l = self.tensors.len();
        let mut env = trivial_env(self.mpo.sites[0].d_l);
        for j in 0..l {
            env = grow_left(&env, &self.mpo.sites[j], &self.tensors[j]);
        }
        let n = self.norm_squared();
        if n == 0.0 {
            return 0.0;
        }
        env[0][0].re / n
    }

    /// Contract the MPS to a dense state vector. Basis index bit `q` is the occupation of
    /// JW site `q`, matching `ops::kron`'s convention and therefore `Mpo::dense`'s.
    /// Exponential by construction — gate use only, at `L <= 12`.
    pub fn to_dense(&self) -> Vec<Cx> {
        to_dense(&self.tensors)
    }
}

/// Contract an MPS to a dense state vector (see `Tdvp::to_dense`).
pub fn to_dense(tensors: &[CTensorSite]) -> Vec<Cx> {
    // `state[r * dim + idx]`
    let mut dim = 1usize;
    let mut chi = 1usize;
    let mut state = vec![Cx::ONE];
    for t in tensors {
        assert_eq!(t.chi_l, chi, "MPS bond mismatch in to_dense");
        let new_dim = dim * 2;
        let mut next = vec![Cx::ZERO; t.chi_r * new_dim];
        for s in 0..2 {
            for l in 0..chi {
                for r in 0..t.chi_r {
                    let av = t.get(s, l, r);
                    if av == Cx::ZERO {
                        continue;
                    }
                    for idx in 0..dim {
                        let cur = state[l * dim + idx];
                        if cur == Cx::ZERO {
                            continue;
                        }
                        next[r * new_dim + s * dim + idx] =
                            next[r * new_dim + s * dim + idx].add(av.mul(cur));
                    }
                }
            }
        }
        state = next;
        dim = new_dim;
        chi = t.chi_r;
    }
    state[..dim].to_vec()
}

// ---------------------------------------------------------------- initial states

/// The natural bond cap at bond `k` of an `l`-site chain: `min(chi_max, 2^k, 2^(l-k))`.
/// At `chi_max >= 2^(l/2)` this is the FULL Hilbert space and the TDVP projector is the
/// identity — which is the regime `G-C2-2` measures the integrator's own order in.
pub fn natural_chi(l: usize, k: usize, chi_max: usize) -> usize {
    let left = if k >= 63 { usize::MAX } else { 1usize << k };
    let right = if l - k >= 63 { usize::MAX } else { 1usize << (l - k) };
    chi_max.min(left).min(right)
}

/// A DETERMINISTIC full-rank starting state at the natural cap.
///
/// Not an RNG: a pinned 64-bit LCG, the same discipline `mps::initial_state` follows for
/// the sweep engine ("fixed initial state, no RNG" applies to every gate in this crate).
/// Full rank is a REQUIREMENT, not a preference — see this module's fence: single-site
/// TDVP cannot grow a bond, so a zero-padded product state would stay rank-deficient and
/// the effective Hamiltonian would be singular on the padding.
pub fn deterministic_state(l: usize, chi_max: usize, seed: u64) -> Vec<CTensorSite> {
    let mut s = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let mut next = move || {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        // top 53 bits into [-1, 1)
        ((s >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    };
    let mut out = Vec::with_capacity(l);
    for j in 0..l {
        let cl = natural_chi(l, j, chi_max);
        let cr = natural_chi(l, j + 1, chi_max);
        let mut t = CTensorSite::zeros(cl, cr);
        for cell in t.data.iter_mut() {
            let re = next();
            let im = next();
            *cell = Cx::new(re, im);
        }
        out.push(t);
    }
    right_canonicalise(&mut out);
    out
}

/// Put an MPS into right-canonical form with the orthogonality centre at site 0, and
/// normalise it. The state is unchanged up to that normalisation.
pub fn right_canonicalise(tensors: &mut Vec<CTensorSite>) {
    let l = tensors.len();
    for j in (1..l).rev() {
        let (c, q, k) = split_right(&tensors[j]);
        tensors[j] = q;
        tensors[j - 1] = absorb_right(&tensors[j - 1], &c, k);
    }
    let n = cnorm(&tensors[0].data);
    if n > 0.0 {
        let inv = 1.0 / n;
        for x in tensors[0].data.iter_mut() {
            *x = x.scale(inv);
        }
    }
}

/// `<psi|H|psi> / <psi|psi>` for any complex MPS, canonical or not: both the numerator and
/// the denominator are contracted from the tensors themselves. The free-function form
/// exists so a carrier can read an energy without constructing an integrator.
pub fn expectation(mpo: &Mpo, tensors: &[CTensorSite]) -> f64 {
    assert_eq!(tensors.len(), mpo.sites.len(), "MPS length must match the MPO");
    let mut env = trivial_env(mpo.sites[0].d_l);
    for (j, t) in tensors.iter().enumerate() {
        env = grow_left(&env, &mpo.sites[j], t);
    }
    let num = env[0][0].re;
    let den = norm_squared(tensors);
    if den == 0.0 {
        0.0
    } else {
        num / den
    }
}

/// `<psi|psi>` for any complex MPS.
pub fn norm_squared(tensors: &[CTensorSite]) -> f64 {
    let mut env = vec![Cx::ONE];
    let mut chi = 1usize;
    for t in tensors.iter() {
        let mut next = vec![Cx::ZERO; t.chi_r * t.chi_r];
        for s in 0..2 {
            for a in 0..t.chi_l {
                for b in 0..t.chi_l {
                    let ev = env[a * chi + b];
                    if ev == Cx::ZERO {
                        continue;
                    }
                    for ap in 0..t.chi_r {
                        let ca = t.get(s, a, ap).conj();
                        if ca == Cx::ZERO {
                            continue;
                        }
                        let acc = ev.mul(ca);
                        for bp in 0..t.chi_r {
                            next[ap * t.chi_r + bp] =
                                next[ap * t.chi_r + bp].add(acc.mul(t.get(s, b, bp)));
                        }
                    }
                }
            }
        }
        env = next;
        chi = t.chi_r;
    }
    env[0].re
}

// ---------------------------------------------------------------- the exact referee

/// The exact propagator `exp(-i H t) |psi0>`, by dense diagonalisation.
///
/// Gate-only and exponential in the chain length: this is the REFEREE, and it is a
/// different code path from the MPS one all the way down (`Mpo::dense` builds `H` by
/// Kronecker products, `jacobi_eigen` diagonalises it, and the propagation is a phase per
/// eigenvalue). Nothing it uses is shared with `Tdvp::step`, which is the point.
pub fn exact_propagate(h_dense: &[f64], dim: usize, psi0: &[Cx], t: f64) -> Vec<Cx> {
    let eig = jacobi_eigen(h_dense.to_vec(), dim);
    let mut out = vec![Cx::ZERO; dim];
    for i in 0..dim {
        let v = &eig.vectors[i];
        // <v_i|psi0> with v_i real
        let mut overlap = Cx::ZERO;
        for (k, &vk) in v.iter().enumerate() {
            if vk != 0.0 {
                overlap = overlap.add(psi0[k].scale(vk));
            }
        }
        let amp = Cx::cis(-t * eig.values[i]).mul(overlap);
        for (k, &vk) in v.iter().enumerate() {
            if vk != 0.0 {
                out[k] = out[k].add(amp.scale(vk));
            }
        }
    }
    out
}

/// `1 - |<a|b>|^2 / (<a|a><b|b>)`: the state-distance the trajectory gates use. Zero iff
/// the two states are equal up to a global phase — which is the right invariance for a
/// wavefunction and the wrong one for a bug, since a phase error in the integrator shows
/// up in `<H>` and in every relative amplitude.
pub fn infidelity(a: &[Cx], b: &[Cx]) -> f64 {
    let na = a.iter().map(|x| x.norm_sq()).sum::<f64>();
    let nb = b.iter().map(|x| x.norm_sq()).sum::<f64>();
    if na == 0.0 || nb == 0.0 {
        return 1.0;
    }
    let ov = cdot(a, b).norm_sq();
    (1.0 - ov / (na * nb)).max(0.0)
}

/// The full complex distance, phase INCLUDED: `|| a - b || / || b ||`. `infidelity` is
/// blind to a global phase and this is not, so the two together separate "the state is
/// wrong" from "only the overall phase is wrong".
pub fn relative_distance(a: &[Cx], b: &[Cx]) -> f64 {
    let nb = cnorm(b);
    if nb == 0.0 {
        return f64::INFINITY;
    }
    let diff: Vec<Cx> = a.iter().zip(b.iter()).map(|(x, y)| x.sub(*y)).collect();
    cnorm(&diff) / nb
}
