//! VIEWS — the dictionary QVM-ACUITY-2 searches (`conformance/qasm/QVM_ACUITY2_PREREG.md`).
//!
//! One procedure, given a circuit, an observable and an acuity `ε`, runs every view's CLOSURE
//! TEST, PRICES every closed view before running anything, and SELECTS the cheapest whose
//! certificate reaches `ε`. It is never told which family a circuit came from: [`Query`] can
//! carry a label (PQ-5 reveals it) and nothing in this module reads it.
//!
//! | view | closure test | price (seconds, measured constants) | certificate |
//! |---|---|---|---|
//! | [`View::Tableau`] | [`crate::sector::locate`]: `t_eff = 0` after the light cone | `c_tab · n² · G` | exact (`0`) |
//! | [`View::Sum`] | the located `t_eff` (always closed: the budget reaches any `ε`) | `N(t_eff)·(G+t)(n+t)·c_sb + N·legs·(n+t)²·c_sa` | [`crate::acuity`]'s certified remainder |
//! | [`View::Mps`] | the PROBE ([`probe`]) — discarded weight measured at `χ ∈ {2,4,8,16}` | `ops · (c_m0 + c_m3·χ³)` | `Σ_k √w_k` (see below) |
//! | [`View::Dense`] | `n ≤ 24` | `c_dense · 2^n · G` | exact (`0`) |
//!
//! # The MPS view and its certificate
//!
//! No complex-amplitude MPS with a gate path exists in this workspace (`q8-mps` is real-tensor
//! DMRG/TDVP and `holon` carries no runtime dependencies), so one is written here, as §6 of the
//! prereg allows: a complex MPS kept in MIXED CANONICAL form, one- and two-qubit gates applied
//! gate by gate, non-adjacent `CX` routed by a SWAP network (the qubit→site map is carried, not
//! swapped back), each two-site update split by a one-sided Jacobi SVD and truncated to `χ`.
//!
//! The certificate is the one the canonical form makes TRUE, and it is not the prereg's literal
//! "summed discarded weight". Before a two-site update the orthogonality centre is moved onto the
//! pair, so both environments are isometries and the singular values of the local two-site
//! matrix ARE the state's Schmidt coefficients across that bond. Truncation is then an orthogonal
//! projection `P_k` on the global state and removes a component of norm exactly `√w_k`, where
//! `w_k` is the sum of the dropped singular values squared. With `φ_k` the exact state and `ψ_k`
//! the truncated one after step `k` (every step unitary or a projection, never a renormalisation),
//! `φ_k − ψ_k = U_k(φ_{k−1} − ψ_{k−1}) + (1 − P_k)U_kψ_{k−1}`, so
//! `‖φ − ψ‖ ≤ Σ_k √w_k` — the triangle inequality, nothing else. The prereg's "summed discarded
//! weight `Σ w_k`" is a bound on the SQUARED error (Verstraete–Cirac's `‖φ−ψ‖² ≤ 2Σw` for one
//! canonical sweep) and is NOT a bound on the 2-norm error: one truncation of weight `w = 10⁻⁴`
//! makes an error of `10⁻²`. The observable then inherits: an amplitude `|⟨y|φ⟩ − ⟨y|ψ⟩| ≤ δ`;
//! a marginal `|⟨φ|Π|φ⟩ − ⟨ψ|Π|ψ⟩| ≤ δ(‖φ‖ + ‖ψ‖) ≤ δ(1 + ‖ψ‖)`.
//!
//! Singular values below [`SV_FLOOR`] are dropped even under the cap (a numerically-zero
//! direction carried forward only grows the bond) and their weight is CHARGED to the
//! certificate like any other truncation.

// Flat tensor code indexes several arrays by one loop variable; iterator rewrites obscure it.
#![allow(clippy::needless_range_loop)]

use crate::affine::{Affine, Gate};
use crate::magic::Circuit;
use crate::sector::{self, ObsValue, Observable, Sector};
use std::ops::{Add, AddAssign, Mul, Neg, Sub};
use std::time::{Duration, Instant};

/// The per-view wall cap the prereg sets for the by-hand runs: a view that does not finish in
/// it is recorded as `> 60 s`.
pub const CAP_S: f64 = 60.0;

/// Singular values at or below this (absolute; every state here has norm `≤ 1`) are dropped and
/// their weight charged to the certificate.
pub const SV_FLOOR: f64 = 1e-13;

/// The bond caps the probe measures at, as frozen.
pub const PROBE_CHIS: [usize; 4] = [2, 4, 8, 16];

// ===========================================================================
// complex arithmetic
// ===========================================================================

/// A complex number.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Z {
    pub re: f64,
    pub im: f64,
}

impl Z {
    pub const ZERO: Z = Z { re: 0.0, im: 0.0 };
    pub const ONE: Z = Z { re: 1.0, im: 0.0 };
    #[inline]
    pub const fn new(re: f64, im: f64) -> Z {
        Z { re, im }
    }
    #[inline]
    pub fn conj(self) -> Z {
        Z { re: self.re, im: -self.im }
    }
    #[inline]
    pub fn norm2(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
    #[inline]
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
    #[inline]
    pub fn scale(self, f: f64) -> Z {
        Z { re: self.re * f, im: self.im * f }
    }
}

impl Add for Z {
    type Output = Z;
    #[inline]
    fn add(self, o: Z) -> Z {
        Z { re: self.re + o.re, im: self.im + o.im }
    }
}
impl Sub for Z {
    type Output = Z;
    #[inline]
    fn sub(self, o: Z) -> Z {
        Z { re: self.re - o.re, im: self.im - o.im }
    }
}
impl Mul for Z {
    type Output = Z;
    #[inline]
    fn mul(self, o: Z) -> Z {
        Z { re: self.re * o.re - self.im * o.im, im: self.re * o.im + self.im * o.re }
    }
}
impl Neg for Z {
    type Output = Z;
    #[inline]
    fn neg(self) -> Z {
        Z { re: -self.re, im: -self.im }
    }
}
impl AddAssign for Z {
    #[inline]
    fn add_assign(&mut self, o: Z) {
        self.re += o.re;
        self.im += o.im;
    }
}

/// The 2×2 matrix of a single-qubit gate, row-major; `None` for `CX`.
pub fn one_qubit_matrix(g: Gate) -> Option<[Z; 4]> {
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let (o, z) = (Z::ONE, Z::ZERO);
    Some(match g {
        Gate::X(_) => [z, o, o, z],
        Gate::Z(_) => [o, z, z, -o],
        Gate::S(_) => [o, z, z, Z::new(0.0, 1.0)],
        Gate::Sdg(_) => [o, z, z, Z::new(0.0, -1.0)],
        Gate::T(_) => [o, z, z, Z::new(s, s)],
        Gate::Tdg(_) => [o, z, z, Z::new(s, -s)],
        Gate::H(_) => [Z::new(s, 0.0), Z::new(s, 0.0), Z::new(s, 0.0), Z::new(-s, 0.0)],
        Gate::Cx(..) => return None,
    })
}

fn gate_qubit(g: Gate) -> usize {
    match g {
        Gate::X(q) | Gate::Z(q) | Gate::S(q) | Gate::Sdg(q) | Gate::T(q) | Gate::Tdg(q) | Gate::H(q) => q,
        Gate::Cx(c, _) => c,
    }
}

fn deadline_passed(deadline: Option<Instant>) -> bool {
    deadline.is_some_and(|d| Instant::now() >= d)
}

// ===========================================================================
// the query — what the search is given, and all it is given
// ===========================================================================

/// One instance as the search sees it. `label` exists so PQ-5 can REVEAL the family; no function
/// in this module reads it.
#[derive(Clone, Debug)]
pub struct Query {
    pub n: usize,
    pub gates: Vec<Gate>,
    pub obs: Observable,
    /// The acuity demanded on the observable, absolute (the relative ladder is applied by the
    /// caller: `ε_rel · 2^{−n/2}` for an amplitude, `ε_rel · 2^{−4}` for a 4-qubit marginal).
    pub eps: f64,
    pub label: Option<String>,
}

impl Query {
    pub fn t_count(&self) -> usize {
        self.gates.iter().filter(|g| g.is_t()).count()
    }
    fn is_marginal(&self) -> bool {
        matches!(self.obs, Observable::Marginal { .. })
    }
    /// What a state-norm error `δ` costs on this observable (amplitude `δ`, marginal `≤ 2δ`).
    fn obs_factor(&self) -> f64 {
        if self.is_marginal() {
            2.0
        } else {
            1.0
        }
    }
}

// ===========================================================================
// DENSE — the 2^n vector, written independently of the referee
// ===========================================================================

/// The dense view: every gate as its matrix on a `2^n` complex vector. Written against
/// [`one_qubit_matrix`], sharing no kernel with [`crate::sector::referee`], so S3 on this view
/// is a comparison and not an identity. `None` if the deadline passes.
pub fn dense_run(n: usize, gates: &[Gate], obs: &Observable, deadline: Option<Instant>) -> Option<ObsValue> {
    let dim = 1usize << n;
    let mut st = vec![Z::ZERO; dim];
    st[0] = Z::ONE;
    for g in gates {
        if deadline_passed(deadline) {
            return None;
        }
        match *g {
            Gate::Cx(c, t) => {
                let (cb, tb) = (1usize << c, 1usize << t);
                for i in 0..dim {
                    if i & cb != 0 && i & tb == 0 {
                        st.swap(i, i | tb);
                    }
                }
            }
            g1 => {
                let u = one_qubit_matrix(g1).expect("single-qubit gate");
                let b = 1usize << gate_qubit(g1);
                for i in 0..dim {
                    if i & b == 0 {
                        let (a0, a1) = (st[i], st[i | b]);
                        st[i] = u[0] * a0 + u[1] * a1;
                        st[i | b] = u[2] * a0 + u[3] * a1;
                    }
                }
            }
        }
    }
    Some(match obs {
        Observable::Amplitude(y) => {
            let a = st[sector::referee::index_of(y)];
            ObsValue::Amp((a.re, a.im))
        }
        Observable::Marginal { qubits, bits } => {
            let mut p = 0.0;
            for (i, a) in st.iter().enumerate() {
                if qubits.iter().zip(bits).all(|(&q, &b)| (i >> q & 1 == 1) == b) {
                    p += a.norm2();
                }
            }
            ObsValue::Prob(p)
        }
    })
}

// ===========================================================================
// TABLEAU — the closed view under Clifford motions
// ===========================================================================

/// The tableau view on a located sector with `t_eff = 0`. The amplitude is read from the
/// engine's affine stabilizer state (exact in `Z[ω]`, one branch — the tableau's phase-carrying
/// form); the marginal from the packed tableau by peek/collapse on the light-cone circuit, exact
/// dyadic. Panics if the sector is not closed: running a view whose closure test failed is the
/// caller's error.
pub fn tableau_run(q: &Query, sec: &Sector) -> ObsValue {
    assert_eq!(sec.t_eff, 0, "views: the tableau view is not closed on this query");
    match &q.obs {
        Observable::Amplitude(y) => {
            let mut st = Affine::new(q.n);
            for g in sec.drop_removed(&q.gates) {
                st.apply(g);
            }
            ObsValue::Amp(st.amplitude(y).to_complex())
        }
        Observable::Marginal { qubits, bits } => {
            let mut tab = crate::tableau::PackedTableau::new(q.n);
            for g in sec.light_cone_circuit(&q.gates) {
                match g {
                    Gate::H(a) => tab.h(a),
                    Gate::S(a) => tab.s(a),
                    Gate::Sdg(a) => tab.sdg(a),
                    Gate::X(a) => tab.x_gate(a),
                    Gate::Z(a) => tab.z_gate(a),
                    Gate::Cx(c, t) => tab.cx(c, t),
                    Gate::T(_) | Gate::Tdg(_) => unreachable!("a T inside the cone with t_eff = 0"),
                }
            }
            let mut p = 1.0f64;
            for (&qq, &b) in qubits.iter().zip(bits) {
                match tab.measure_peek(qq) {
                    Some(v) => {
                        if v != b {
                            p = 0.0;
                            break;
                        }
                    }
                    None => {
                        p *= 0.5;
                        tab.collapse(qq, b);
                    }
                }
            }
            ObsValue::Prob(p)
        }
    }
}

// ===========================================================================
// SUM — the budgeted stabilizer-rank sum (holon::acuity)
// ===========================================================================

/// The sum view's answer.
#[derive(Clone, Debug)]
pub struct SumRun {
    pub value: ObsValue,
    pub certificate: f64,
    /// Branch evaluations executed / named, summed over legs.
    pub evaluated: u64,
    pub total: u64,
    pub legs: u64,
    pub wall_source: f64,
    pub wall_fold: f64,
}

/// `2^{|L|−|S|}`: the amplitudes a marginal on `S` inside a light cone `L` is a sum of.
pub fn sum_legs(q: &Query, sec: &Sector) -> u64 {
    match &q.obs {
        Observable::Amplitude(_) => 1,
        Observable::Marginal { qubits, .. } => {
            let free = sec.light_cone.iter().filter(|x| !qubits.contains(x)).count();
            1u64.checked_shl(free as u32).unwrap_or(u64::MAX)
        }
    }
}

/// The budgeted sum at acuity `eps` (`0` = the full sum). A marginal runs each leg at the
/// amplitude acuity `R = ε / (4√legs)`, so `Σ(2|a|R + R²) ≤ 2R√legs + legs·R² ≤ ε`, and reports the
/// propagated bound it actually reached. `None` if the deadline passes between legs.
pub fn sum_run(q: &Query, sec: &Sector, eps: f64, deadline: Option<Instant>) -> Option<SumRun> {
    use crate::acuity::{budgeted_amplitude_with, certified_source_for, BudgetPlan};
    match &q.obs {
        Observable::Amplitude(y) => {
            let reduced = sec.reduced_circuit(&q.gates);
            let t0 = Instant::now();
            let src = certified_source_for(&reduced, y);
            let plan = BudgetPlan::of(&src);
            let wall_source = t0.elapsed().as_secs_f64();
            let t1 = Instant::now();
            let b = budgeted_amplitude_with(&src, &plan, y, eps, 1);
            Some(SumRun {
                value: ObsValue::Amp(b.value_f64),
                certificate: b.remainder,
                evaluated: b.evaluated,
                total: b.total,
                legs: 1,
                wall_source,
                wall_fold: t1.elapsed().as_secs_f64(),
            })
        }
        Observable::Marginal { qubits, bits } => {
            let legs = sum_legs(q, sec);
            let free: Vec<usize> = sec.light_cone.iter().copied().filter(|x| !qubits.contains(x)).collect();
            let reduced = Circuit { n_qubits: sec.n_qubits, gates: sec.light_cone_circuit(&q.gates) };
            let mut y0 = vec![false; sec.n_qubits];
            for (&qq, &b) in qubits.iter().zip(bits) {
                y0[qq] = b;
            }
            let t0 = Instant::now();
            let src = certified_source_for(&reduced, &y0);
            let plan = BudgetPlan::of(&src);
            let wall_source = t0.elapsed().as_secs_f64();
            let t1 = Instant::now();
            let r_leg = eps / (4.0 * (legs as f64).sqrt());
            let (mut p, mut cert, mut ev, mut tot) = (0.0f64, 0.0f64, 0u64, 0u64);
            for m in 0..legs {
                if deadline_passed(deadline) {
                    return None;
                }
                let mut y = y0.clone();
                for (j, &qq) in free.iter().enumerate() {
                    y[qq] = m >> j & 1 == 1;
                }
                let b = budgeted_amplitude_with(&src, &plan, &y, r_leg, 1);
                let a2 = b.value_f64.0 * b.value_f64.0 + b.value_f64.1 * b.value_f64.1;
                p += a2;
                cert = crate::acuity::nudge_up(cert + 2.0 * a2.sqrt() * b.remainder + b.remainder * b.remainder);
                ev += b.evaluated;
                tot += b.total;
            }
            Some(SumRun {
                value: ObsValue::Prob(p),
                certificate: cert,
                evaluated: ev,
                total: tot,
                legs,
                wall_source,
                wall_fold: t1.elapsed().as_secs_f64(),
            })
        }
    }
}

// ===========================================================================
// linear algebra for the MPS: Householder QR and one-sided Jacobi SVD
// ===========================================================================

/// A row-major complex matrix.
#[derive(Clone, Debug)]
pub struct Mat {
    pub m: usize,
    pub n: usize,
    pub a: Vec<Z>,
}

impl Mat {
    pub fn zeros(m: usize, n: usize) -> Mat {
        Mat { m, n, a: vec![Z::ZERO; m * n] }
    }
    #[inline]
    pub fn at(&self, i: usize, j: usize) -> Z {
        self.a[i * self.n + j]
    }
    pub fn adjoint(&self) -> Mat {
        let mut o = Mat::zeros(self.n, self.m);
        for i in 0..self.m {
            for j in 0..self.n {
                o.a[j * self.m + i] = self.a[i * self.n + j].conj();
            }
        }
        o
    }
    pub fn matmul(&self, b: &Mat) -> Mat {
        assert_eq!(self.n, b.m);
        let mut o = Mat::zeros(self.m, b.n);
        for i in 0..self.m {
            for k in 0..self.n {
                let x = self.a[i * self.n + k];
                if x == Z::ZERO {
                    continue;
                }
                let row = &b.a[k * b.n..(k + 1) * b.n];
                let orow = &mut o.a[i * b.n..(i + 1) * b.n];
                for (oj, &bj) in orow.iter_mut().zip(row) {
                    *oj += x * bj;
                }
            }
        }
        o
    }
}

/// Householder QR: `A = Q R`, `Q` is `m × k` with orthonormal columns, `R` is `k × n`,
/// `k = min(m, n)`.
pub fn qr(a: &Mat) -> (Mat, Mat) {
    let (m, n) = (a.m, a.n);
    let k = m.min(n);
    let mut r = a.clone();
    let mut vs: Vec<Vec<Z>> = Vec::with_capacity(k);
    for j in 0..k {
        let norm: f64 = (j..m).map(|i| r.at(i, j).norm2()).sum::<f64>().sqrt();
        if norm == 0.0 {
            vs.push(Vec::new());
            continue;
        }
        let x0 = r.at(j, j);
        let x0a = x0.abs();
        let ph = if x0a < 1e-250 { Z::ONE } else { Z::new(x0.re / x0a, x0.im / x0a) };
        let alpha = -(ph.scale(norm));
        let mut v: Vec<Z> = (j..m).map(|i| r.at(i, j)).collect();
        v[0] = v[0] - alpha;
        let vn: f64 = v.iter().map(|z| z.norm2()).sum::<f64>().sqrt();
        if vn == 0.0 {
            vs.push(Vec::new());
            continue;
        }
        for z in v.iter_mut() {
            *z = Z::new(z.re / vn, z.im / vn);
        }
        // R[j.., j..] -= 2 v (v† R[j.., j..])
        for c in j..n {
            let mut d = Z::ZERO;
            for (ii, vi) in v.iter().enumerate() {
                d += vi.conj() * r.a[(j + ii) * n + c];
            }
            let d2 = d.scale(2.0);
            for (ii, vi) in v.iter().enumerate() {
                let idx = (j + ii) * n + c;
                r.a[idx] = r.a[idx] - *vi * d2;
            }
        }
        vs.push(v);
    }
    let mut q = Mat::zeros(m, k);
    for i in 0..k {
        q.a[i * k + i] = Z::ONE;
    }
    for j in (0..k).rev() {
        let v = &vs[j];
        if v.is_empty() {
            continue;
        }
        for c in 0..k {
            let mut d = Z::ZERO;
            for (ii, vi) in v.iter().enumerate() {
                d += vi.conj() * q.a[(j + ii) * k + c];
            }
            let d2 = d.scale(2.0);
            for (ii, vi) in v.iter().enumerate() {
                let idx = (j + ii) * k + c;
                q.a[idx] = q.a[idx] - *vi * d2;
            }
        }
    }
    let mut rr = Mat::zeros(k, n);
    for i in 0..k {
        for c in i..n {
            rr.a[i * n + c] = r.a[i * n + c];
        }
    }
    (q, rr)
}

/// One-sided (Hestenes) Jacobi SVD, `A = U diag(s) Vh`, singular values descending. Chosen over
/// the Gram-matrix route because it resolves SMALL singular values to absolute accuracy
/// `~ε_mach‖A‖` — the discarded weight is made of exactly those, and a Gram eigensolver would
/// only resolve them to `~√ε_mach`.
pub fn svd(a: &Mat) -> (Mat, Vec<f64>, Mat) {
    if a.m < a.n {
        let (u, s, vh) = svd(&a.adjoint());
        // A† = U S Vh  ⇒  A = Vh† S U†
        return (vh.adjoint(), s, u.adjoint());
    }
    let (m, n) = (a.m, a.n);
    // columns of W, columns of V
    let mut w: Vec<Vec<Z>> = (0..n).map(|j| (0..m).map(|i| a.at(i, j)).collect()).collect();
    let mut v: Vec<Vec<Z>> = (0..n)
        .map(|j| (0..n).map(|i| if i == j { Z::ONE } else { Z::ZERO }).collect())
        .collect();
    let mut norms: Vec<f64> = w.iter().map(|c| c.iter().map(|z| z.norm2()).sum()).collect();
    for _sweep in 0..60 {
        let mut rotated = false;
        for p in 0..n {
            for q in p + 1..n {
                let (alpha, beta) = (norms[p], norms[q]);
                if alpha == 0.0 || beta == 0.0 {
                    continue;
                }
                let mut g = Z::ZERO;
                for i in 0..m {
                    g += w[p][i].conj() * w[q][i];
                }
                let ga = g.abs();
                // subnormal overlaps are zero overlaps: 1/ga would overflow and 0·∞ is NaN
                if ga <= 1e-15 * (alpha * beta).sqrt() || ga < 1e-250 {
                    continue;
                }
                rotated = true;
                let e = Z::new(g.re / ga, -g.im / ga); // e^{-iφ}
                let zeta = (beta - alpha) / (2.0 * ga);
                let t = zeta.signum() / (zeta.abs() + (1.0 + zeta * zeta).sqrt());
                let t = if zeta == 0.0 { 1.0 } else { t };
                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = c * t;
                let (wp, wq) = {
                    let (lo, hi) = w.split_at_mut(q);
                    (&mut lo[p], &mut hi[0])
                };
                for i in 0..m {
                    let a0 = wp[i];
                    let b0 = wq[i] * e;
                    wp[i] = a0.scale(c) - b0.scale(s);
                    wq[i] = a0.scale(s) + b0.scale(c);
                }
                let (vp, vq) = {
                    let (lo, hi) = v.split_at_mut(q);
                    (&mut lo[p], &mut hi[0])
                };
                for i in 0..n {
                    let a0 = vp[i];
                    let b0 = vq[i] * e;
                    vp[i] = a0.scale(c) - b0.scale(s);
                    vq[i] = a0.scale(s) + b0.scale(c);
                }
                norms[p] = wp.iter().map(|z| z.norm2()).sum();
                norms[q] = wq.iter().map(|z| z.norm2()).sum();
            }
        }
        if !rotated {
            break;
        }
    }
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&x, &y| norms[y].total_cmp(&norms[x]));
    let s: Vec<f64> = idx.iter().map(|&j| norms[j].sqrt()).collect();
    let mut u = Mat::zeros(m, n);
    let mut vh = Mat::zeros(n, n);
    for (col, &j) in idx.iter().enumerate() {
        let sj = s[col];
        for i in 0..m {
            u.a[i * n + col] = if sj > 1e-250 { Z::new(w[j][i].re / sj, w[j][i].im / sj) } else { Z::ZERO };
        }
        for i in 0..n {
            vh.a[col * n + i] = v[j][i].conj();
        }
    }
    (u, s, vh)
}

// ===========================================================================
// MPS — the bond-dimension view
// ===========================================================================

/// One routed operation on SITES (the SWAP network already resolved).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    /// A single-qubit gate on a site.
    One { site: usize, g: Gate },
    /// A two-site gate on `(site, site+1)`: `Swap`, or `CX` with the control on the left
    /// (`ctl_left`) or on the right.
    Swap { site: usize },
    Cx { site: usize, ctl_left: bool },
}

impl Op {
    pub fn is_two_site(&self) -> bool {
        !matches!(self, Op::One { .. })
    }
}

/// Route a circuit onto a line: a non-adjacent `CX` moves its control next to its target by
/// adjacent SWAPs, and the qubit→site map is carried forward rather than undone. Returns the ops
/// and, for every original gate, the op index at which it ENDS (the probe's cadence counts
/// circuit gates, not routed ops).
pub fn route(n: usize, gates: &[Gate]) -> (Vec<Op>, Vec<usize>) {
    let mut pos: Vec<usize> = (0..n).collect(); // logical → site
    let mut at: Vec<usize> = (0..n).collect(); // site → logical
    let mut ops = Vec::with_capacity(gates.len() * 2);
    let mut ends = Vec::with_capacity(gates.len());
    for g in gates {
        match *g {
            Gate::Cx(c, t) => {
                while pos[c].abs_diff(pos[t]) > 1 {
                    let (pc, pt) = (pos[c], pos[t]);
                    let (a, b) = if pc < pt { (pc, pc + 1) } else { (pc - 1, pc) };
                    ops.push(Op::Swap { site: a });
                    let (qa, qb) = (at[a], at[b]);
                    at.swap(a, b);
                    pos[qa] = b;
                    pos[qb] = a;
                    let _ = pt;
                }
                let (pc, pt) = (pos[c], pos[t]);
                ops.push(Op::Cx { site: pc.min(pt), ctl_left: pc < pt });
            }
            g1 => ops.push(Op::One { site: pos[gate_qubit(g1)], g: g1 }),
        }
        ends.push(ops.len());
    }
    (ops, ends)
}

/// The number of two-site operations a routed circuit costs — the MPS price's operation count.
pub fn two_site_ops(n: usize, gates: &[Gate]) -> usize {
    route(n, gates).0.iter().filter(|o| o.is_two_site()).count()
}

/// A complex MPS in mixed canonical form. Site tensor `i` is `(dl, 2, dr)`, flat index
/// `(l·2 + s)·dr + r`.
#[derive(Clone, Debug)]
pub struct Mps {
    pub n: usize,
    pub chi: usize,
    pub sites: Vec<Vec<Z>>,
    pub dims: Vec<usize>, // bond dims, len n+1, dims[0] = dims[n] = 1
    pub center: usize,
    /// logical qubit → site, and back.
    pub pos: Vec<usize>,
    pub at: Vec<usize>,
    /// Per truncation: `(bond, w)`, `w` the dropped Schmidt weight.
    pub discarded: Vec<(usize, f64)>,
    /// `Σ √w_k`, rounded up at every step: the certified bound on the state's 2-norm error.
    pub delta: f64,
    pub max_bond: usize,
}

impl Mps {
    /// `|0…0⟩`.
    pub fn zero_state(n: usize, chi: usize) -> Mps {
        Mps {
            n,
            chi,
            sites: (0..n).map(|_| vec![Z::ONE, Z::ZERO]).collect(),
            dims: vec![1; n + 1],
            center: 0,
            pos: (0..n).collect(),
            at: (0..n).collect(),
            discarded: Vec::new(),
            delta: 0.0,
            max_bond: 1,
        }
    }

    fn move_center_to(&mut self, target: usize) {
        while self.center < target {
            let i = self.center;
            let (dl, dr) = (self.dims[i], self.dims[i + 1]);
            let a = Mat { m: dl * 2, n: dr, a: std::mem::take(&mut self.sites[i]) };
            let (q, r) = qr(&a);
            let k = q.n;
            self.sites[i] = q.a;
            let dr2 = self.dims[i + 2];
            let next = Mat { m: dr, n: 2 * dr2, a: std::mem::take(&mut self.sites[i + 1]) };
            self.sites[i + 1] = r.matmul(&next).a;
            self.dims[i + 1] = k;
            self.center += 1;
        }
        while self.center > target {
            let i = self.center;
            let (dl, dr) = (self.dims[i], self.dims[i + 1]);
            let a = Mat { m: dl, n: 2 * dr, a: std::mem::take(&mut self.sites[i]) };
            // A = L Q via A† = Q' R'
            let (qp, rp) = qr(&a.adjoint());
            let k = qp.n;
            self.sites[i] = qp.adjoint().a; // k × 2dr
            let l = rp.adjoint(); // dl × k
            let dl2 = self.dims[i - 1];
            let prev = Mat { m: dl2 * 2, n: dl, a: std::mem::take(&mut self.sites[i - 1]) };
            self.sites[i - 1] = prev.matmul(&l).a;
            self.dims[i] = k;
            self.center -= 1;
        }
    }

    fn apply_one(&mut self, site: usize, u: &[Z; 4]) {
        let (dl, dr) = (self.dims[site], self.dims[site + 1]);
        let t = &mut self.sites[site];
        for l in 0..dl {
            for r in 0..dr {
                let a0 = t[(l * 2) * dr + r];
                let a1 = t[(l * 2 + 1) * dr + r];
                t[(l * 2) * dr + r] = u[0] * a0 + u[1] * a1;
                t[(l * 2 + 1) * dr + r] = u[2] * a0 + u[3] * a1;
            }
        }
    }

    /// A two-site gate on `(i, i+1)` given as a permutation of the local basis `s1·2 + s2`
    /// (SWAP and CX are both permutations): the centre is moved onto `i`, the pair contracted,
    /// permuted, split by SVD, truncated, the centre left on `i+1`.
    fn apply_two_perm(&mut self, i: usize, perm: [usize; 4]) {
        self.move_center_to(i);
        let (dl, dm, dr) = (self.dims[i], self.dims[i + 1], self.dims[i + 2]);
        let a = Mat { m: dl * 2, n: dm, a: std::mem::take(&mut self.sites[i]) };
        let b = Mat { m: dm, n: 2 * dr, a: std::mem::take(&mut self.sites[i + 1]) };
        let th = a.matmul(&b); // (l,s1) × (s2,r)
        // θ'[(l,s1),(s2,r)] = θ[(l,s1'),(s2',r)] where perm maps (s1,s2) ← (s1',s2'):
        // the output basis state perm[x] receives input basis state x.
        let mut tp = Mat::zeros(dl * 2, 2 * dr);
        for l in 0..dl {
            for x in 0..4usize {
                let (s1, s2) = (x >> 1, x & 1);
                let y = perm[x];
                let (o1, o2) = (y >> 1, y & 1);
                for r in 0..dr {
                    tp.a[(l * 2 + o1) * (2 * dr) + o2 * dr + r] = th.a[(l * 2 + s1) * (2 * dr) + s2 * dr + r];
                }
            }
        }
        let (u, s, vh) = svd(&tp);
        assert!(s.iter().all(|x| x.is_finite()), "views: a non-finite singular value would make the certificate a lie");
        let keep_nz = s.iter().take_while(|&&x| x > SV_FLOOR).count();
        let keep = keep_nz.min(self.chi).max(1);
        let w: f64 = s[keep..].iter().map(|x| x * x).sum();
        if w > 0.0 {
            self.discarded.push((i + 1, w));
            self.delta = crate::acuity::nudge_up(self.delta + crate::acuity::nudge_up(w.sqrt()));
        }
        let mut na = Mat::zeros(dl * 2, keep);
        for row in 0..dl * 2 {
            for c in 0..keep {
                na.a[row * keep + c] = u.at(row, c);
            }
        }
        let mut nb = Mat::zeros(keep, 2 * dr);
        for rrow in 0..keep {
            for c in 0..2 * dr {
                nb.a[rrow * 2 * dr + c] = vh.at(rrow, c).scale(s[rrow]);
            }
        }
        self.sites[i] = na.a;
        self.sites[i + 1] = nb.a;
        self.dims[i + 1] = keep;
        self.max_bond = self.max_bond.max(keep);
        self.center = i + 1;
    }

    pub fn apply_op(&mut self, op: Op) {
        match op {
            Op::One { site, g } => {
                let u = one_qubit_matrix(g).expect("single-qubit op");
                self.apply_one(site, &u);
            }
            Op::Swap { site } => {
                self.apply_two_perm(site, [0, 2, 1, 3]);
                let (qa, qb) = (self.at[site], self.at[site + 1]);
                self.at.swap(site, site + 1);
                self.pos[qa] = site + 1;
                self.pos[qb] = site;
            }
            Op::Cx { site, ctl_left } => {
                // basis x = s_left·2 + s_right
                let perm = if ctl_left { [0, 1, 3, 2] } else { [0, 3, 2, 1] };
                self.apply_two_perm(site, perm);
            }
        }
    }

    /// `‖ψ‖`, read off the centre tensor.
    pub fn norm(&self) -> f64 {
        self.sites[self.center].iter().map(|z| z.norm2()).sum::<f64>().sqrt()
    }

    /// `⟨y|ψ⟩`, `y` indexed by LOGICAL qubit.
    pub fn amplitude(&self, y: &[bool]) -> Z {
        let mut v = vec![Z::ONE];
        for i in 0..self.n {
            let (dl, dr) = (self.dims[i], self.dims[i + 1]);
            let s = y[self.at[i]] as usize;
            let t = &self.sites[i];
            let mut nv = vec![Z::ZERO; dr];
            for (l, &vl) in v.iter().enumerate().take(dl) {
                if vl == Z::ZERO {
                    continue;
                }
                for (r, o) in nv.iter_mut().enumerate() {
                    *o += vl * t[(l * 2 + s) * dr + r];
                }
            }
            v = nv;
        }
        v[0]
    }

    /// `⟨ψ|Π|ψ⟩` for `Π` the projector on `bits` at the LOGICAL `qubits`.
    pub fn marginal(&self, qubits: &[usize], bits: &[bool]) -> f64 {
        let mut e = vec![Z::ONE]; // dl × dl, row-major
        for i in 0..self.n {
            let (dl, dr) = (self.dims[i], self.dims[i + 1]);
            let q = self.at[i];
            let allowed: Vec<usize> = match qubits.iter().position(|&x| x == q) {
                Some(j) => vec![bits[j] as usize],
                None => vec![0, 1],
            };
            let t = &self.sites[i];
            let mut ne = vec![Z::ZERO; dr * dr];
            for &s in &allowed {
                // tmp[l', r] = Σ_l E[l', l] A[l, s, r]
                let mut tmp = vec![Z::ZERO; dl * dr];
                for lp in 0..dl {
                    for l in 0..dl {
                        let x = e[lp * dl + l];
                        if x == Z::ZERO {
                            continue;
                        }
                        for r in 0..dr {
                            tmp[lp * dr + r] += x * t[(l * 2 + s) * dr + r];
                        }
                    }
                }
                // ne[r', r] += Σ_l' conj(A[l', s, r']) tmp[l', r]
                for lp in 0..dl {
                    for rp in 0..dr {
                        let x = t[(lp * 2 + s) * dr + rp].conj();
                        if x == Z::ZERO {
                            continue;
                        }
                        for r in 0..dr {
                            ne[rp * dr + r] += x * tmp[lp * dr + r];
                        }
                    }
                }
            }
            e = ne;
        }
        e[0].re
    }

    /// The dense vector (tests only; `n ≤ 16`), indexed by LOGICAL qubit bits.
    pub fn to_dense(&self) -> Vec<Z> {
        let dim = 1usize << self.n;
        (0..dim)
            .map(|x| {
                let y: Vec<bool> = (0..self.n).map(|q| x >> q & 1 == 1).collect();
                self.amplitude(&y)
            })
            .collect()
    }
}

/// How an MPS run stops early.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MpsStop {
    /// Run to the end whatever the certificate says.
    Never,
    /// Stop the moment `obs_factor · δ > limit` — the certificate can only grow, so the run can
    /// no longer reach the acuity.
    CertOver(f64),
    /// The CADENCE rule (Amendment 1): every `window` circuit gates, project the certificate
    /// linearly to the whole circuit and stop if it exceeds `limit`; also stop on `CertOver`.
    Cadence { window: usize, limit: f64 },
}

/// One MPS run.
#[derive(Clone, Debug)]
pub struct MpsRun {
    pub chi: usize,
    pub value: Option<ObsValue>,
    /// On the OBSERVABLE: `δ` for an amplitude, `δ(1+‖ψ‖)` for a marginal.
    pub certificate: f64,
    pub delta: f64,
    pub max_bond: usize,
    pub ops: usize,
    pub gates_done: usize,
    pub stopped: Option<String>,
    /// Discarded weight summed per bond (index = bond `1..n−1`).
    pub per_cut: Vec<f64>,
    pub truncations: usize,
    pub wall: f64,
}

/// Run the MPS view on the first `n_gates` gates of the query at cap `chi`.
pub fn mps_run(q: &Query, chi: usize, n_gates: usize, stop: MpsStop, deadline: Option<Instant>) -> MpsRun {
    let t0 = Instant::now();
    let gates = &q.gates[..n_gates.min(q.gates.len())];
    let (ops, ends) = route(q.n, gates);
    let mut m = Mps::zero_state(q.n, chi);
    let fac = q.obs_factor();
    let total_g = q.gates.len() as f64;
    let mut stopped = None;
    let mut gi = 0usize; // gates completed
    let mut done_ops = 0usize;
    'outer: for (k, op) in ops.iter().enumerate() {
        m.apply_op(*op);
        done_ops = k + 1;
        while gi < ends.len() && ends[gi] <= done_ops {
            gi += 1;
            if let MpsStop::Cadence { window, limit } = stop {
                if gi.is_multiple_of(window) {
                    let proj = fac * m.delta * total_g / gi as f64;
                    if proj > limit {
                        stopped = Some(format!(
                            "cadence: after {gi} gates the projected certificate {proj:.3e} > {limit:.3e}"
                        ));
                        break 'outer;
                    }
                }
            }
        }
        let lim = match stop {
            MpsStop::CertOver(l) => Some(l),
            MpsStop::Cadence { limit, .. } => Some(limit),
            MpsStop::Never => None,
        };
        if let Some(l) = lim {
            if fac * m.delta > l {
                stopped = Some(format!("certificate {:.3e} > {l:.3e} after {} gates", fac * m.delta, gi));
                break;
            }
        }
        if k % 16 == 0 && deadline_passed(deadline) {
            stopped = Some(format!("deadline after {gi} gates"));
            break;
        }
    }
    let mut per_cut = vec![0.0; q.n + 1];
    for &(b, w) in &m.discarded {
        per_cut[b] += w;
    }
    let complete = stopped.is_none() && gi == gates.len();
    let value = if complete {
        Some(match &q.obs {
            Observable::Amplitude(y) => {
                let a = m.amplitude(y);
                ObsValue::Amp((a.re, a.im))
            }
            Observable::Marginal { qubits, bits } => ObsValue::Prob(m.marginal(qubits, bits)),
        })
    } else {
        None
    };
    let certificate = if q.is_marginal() {
        crate::acuity::nudge_up(m.delta * (1.0 + m.norm()))
    } else {
        m.delta
    };
    MpsRun {
        chi,
        value,
        certificate,
        delta: m.delta,
        max_bond: m.max_bond,
        ops: done_ops,
        gates_done: gi,
        stopped,
        per_cut,
        truncations: m.discarded.len(),
        wall: t0.elapsed().as_secs_f64(),
    }
}

// ===========================================================================
// the probe — the MPS view's closure test
// ===========================================================================

/// Which reading of "a PROBE at cadence" the MPS closure test runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeMode {
    /// As frozen (§1's table, and the task's spelling of it): the circuit's first `2n` gates at
    /// each `χ`, the certificate over that prefix projected linearly to the whole circuit.
    Prefix,
    /// Amendment 1: the same measurement repeated every `2n` gates through the whole circuit —
    /// a `χ` is closed only if no checkpoint's projection exceeds the acuity.
    Cadence,
}

impl ProbeMode {
    pub fn name(self) -> &'static str {
        match self {
            ProbeMode::Prefix => "prefix",
            ProbeMode::Cadence => "cadence",
        }
    }
}

/// One `χ`'s probe reading.
#[derive(Clone, Debug)]
pub struct ProbeAt {
    pub chi: usize,
    /// The certificate on the observable over what the probe ran.
    pub measured: f64,
    /// Projected to the whole circuit.
    pub projected: f64,
    pub gates_probed: usize,
    pub max_bond: usize,
    pub per_cut: Vec<f64>,
    pub closed: bool,
}

/// The probe's verdict.
#[derive(Clone, Debug)]
pub struct Probe {
    pub mode: ProbeMode,
    pub at: Vec<ProbeAt>,
    /// The smallest `χ` whose projected certificate is under the acuity, if any.
    pub chi: Option<usize>,
    pub wall: f64,
}

/// The MPS view's closure test. The acuity's share is ALL of `ε` (no other error enters the MPS
/// view's answer).
pub fn probe(q: &Query, mode: ProbeMode, deadline: Option<Instant>) -> Probe {
    let t0 = Instant::now();
    let g = q.gates.len();
    let window = (2 * q.n).max(1);
    let mut at = Vec::new();
    let mut chi = None;
    for &c in &PROBE_CHIS {
        let (run, projected) = match mode {
            ProbeMode::Prefix => {
                let r = mps_run(q, c, window.min(g), MpsStop::Never, deadline);
                let gp = r.gates_done.max(1) as f64;
                let proj = r.certificate * g as f64 / gp;
                (r, proj)
            }
            ProbeMode::Cadence => {
                let r = mps_run(q, c, g, MpsStop::Cadence { window, limit: q.eps }, deadline);
                let proj = if r.stopped.is_some() {
                    // the checkpoint that stopped it, re-projected
                    let gp = r.gates_done.max(1) as f64;
                    (r.certificate * g as f64 / gp).max(r.certificate)
                } else {
                    r.certificate
                };
                let proj = if r.stopped.as_deref().is_some_and(|s| s.starts_with("deadline")) {
                    f64::INFINITY
                } else {
                    proj
                };
                (r, proj)
            }
        };
        let closed = projected <= q.eps;
        at.push(ProbeAt {
            chi: c,
            measured: run.certificate,
            projected,
            gates_probed: run.gates_done,
            max_bond: run.max_bond,
            per_cut: run.per_cut.clone(),
            closed,
        });
        if closed {
            chi = Some(c);
            break;
        }
    }
    Probe { mode, at, chi, wall: t0.elapsed().as_secs_f64() }
}

// ===========================================================================
// prices, and the selection
// ===========================================================================

/// The four views.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum View {
    Tableau,
    Sum,
    Mps,
    Dense,
}

impl View {
    pub const ALL: [View; 4] = [View::Tableau, View::Sum, View::Mps, View::Dense];
    pub fn name(self) -> &'static str {
        match self {
            View::Tableau => "tableau",
            View::Sum => "sum",
            View::Mps => "mps",
            View::Dense => "dense",
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

/// The measured cost constants, seconds per unit of each view's price formula. Calibrated ONCE
/// ([`calibrate`]) and printed.
#[derive(Clone, Copy, Debug)]
pub struct Constants {
    /// per `n² · G`
    pub c_tab: f64,
    /// per branch · `(G+t)(n+t)` (branch construction + certified bound)
    pub c_sb: f64,
    /// per branch · leg · `(n+t)²` (the fold)
    pub c_sa: f64,
    /// per two-site op (overhead)
    pub c_m0: f64,
    /// per two-site op · `χ³`
    pub c_m3: f64,
    /// per `2^n · G`
    pub c_dense: f64,
}

impl Constants {
    pub fn line(&self) -> String {
        format!(
            "calibrate: c_tab={:.3e} s/(n^2 G)  c_sb={:.3e} s/(branch (G+t)(n+t))  c_sa={:.3e} s/(branch leg (n+t)^2)  \
             c_m0={:.3e} s/op  c_m3={:.3e} s/(op chi^3)  c_dense={:.3e} s/(2^n G)",
            self.c_tab, self.c_sb, self.c_sa, self.c_m0, self.c_m3, self.c_dense
        )
    }
    pub fn json(&self) -> String {
        format!(
            "{{\"c_tab\": {:.6e}, \"c_sb\": {:.6e}, \"c_sa\": {:.6e}, \"c_m0\": {:.6e}, \"c_m3\": {:.6e}, \"c_dense\": {:.6e}}}",
            self.c_tab, self.c_sb, self.c_sa, self.c_m0, self.c_m3, self.c_dense
        )
    }
}

/// Median wall of `f` over up to `reps` repetitions (fewer if one already costs `> 0.2 s`).
pub fn timed<T>(reps: usize, mut f: impl FnMut() -> T) -> (T, f64) {
    let mut walls = Vec::new();
    let mut out;
    let t0 = Instant::now();
    out = f();
    walls.push(t0.elapsed().as_secs_f64());
    while walls.len() < reps && walls.iter().sum::<f64>() < 0.2 {
        let t = Instant::now();
        out = f();
        walls.push(t.elapsed().as_secs_f64());
    }
    walls.sort_by(f64::total_cmp);
    (out, walls[walls.len() / 2])
}

/// Measure the constants on fixed calibration circuits that belong to none of the families'
/// instance seeds (seed `0xCA11`).
pub fn calibrate() -> Constants {
    let seed = 0xCA11u64;
    // tableau: Clifford, n = 16, 20n gates
    let c = sector::random_instance(16, 0, seed);
    let y = vec![false; 16];
    let q = Query { n: 16, gates: c.gates.clone(), obs: Observable::Amplitude(y), eps: 0.0, label: None };
    let sec = sector::locate(&q.gates, &q.obs);
    let (_, w) = timed(9, || tableau_run(&q, &sec));
    let c_tab = w / (16.0 * 16.0 * q.gates.len() as f64);
    // sum, construction: an amplitude at n = 16, t = 16, 20n deep (mid-range of family T)
    let c = sector::random_instance(16, 16, seed);
    let q = Query { n: 16, gates: c.gates.clone(), obs: Observable::Amplitude(vec![false; 16]), eps: 0.0, label: None };
    let sec = sector::locate(&q.gates, &q.obs);
    let r = sum_run(&q, &sec, 0.0, None).expect("calibration sum");
    let (n, t, gg) = (16.0, sec.t_eff as f64, q.gates.len() as f64);
    let nb = crate::magic5::expected_branches(sec.t_eff) as f64;
    let c_sb = r.wall_source / (nb * (gg + t) * (n + t));
    // sum, fold: a marginal's legs at n = 12, t = 12, 24 deep
    let c = sector::random_instance_depth(12, 12, seed, 24);
    let obs = Observable::Marginal { qubits: vec![0, 1, 2, 3], bits: vec![false; 4] };
    let q = Query { n: 12, gates: c.gates.clone(), obs, eps: 0.0, label: None };
    let sec = sector::locate(&q.gates, &q.obs);
    let r = sum_run(&q, &sec, 0.0, None).expect("calibration sum");
    let (n, t, gg) = (12.0, sec.t_eff as f64, sec.light_cone_circuit(&q.gates).len() as f64);
    let nb = crate::magic5::expected_branches(sec.t_eff) as f64;
    let _ = gg;
    let c_sa = r.wall_fold / (nb * r.legs as f64 * (n + t) * (n + t));
    // mps: n = 12 brickwork at chi = 2 (overhead) and chi = 16 (the cubic term)
    let b = brickwork(12, 12, 40, seed);
    let q = Query { n: 12, gates: b.gates, obs: Observable::Amplitude(vec![false; 12]), eps: 0.0, label: None };
    let ops = two_site_ops(12, &q.gates) as f64;
    let (_, w2) = timed(5, || mps_run(&q, 2, q.gates.len(), MpsStop::Never, None));
    let (_, w16) = timed(3, || mps_run(&q, 16, q.gates.len(), MpsStop::Never, None));
    let c_m0 = w2 / ops;
    let c_m3 = ((w16 - w2) / (ops * (16f64.powi(3) - 8.0))).max(1e-12);
    // dense: n = 16, 20n gates
    let c = sector::random_instance(16, 8, seed);
    let obs = Observable::Amplitude(vec![false; 16]);
    let (_, wd) = timed(5, || dense_run(16, &c.gates, &obs, None));
    let c_dense = wd / (65536.0 * c.gates.len() as f64);
    Constants { c_tab, c_sb, c_sa, c_m0, c_m3, c_dense }
}

/// One view's closure test and price.
#[derive(Clone, Debug)]
pub struct Priced {
    pub view: View,
    pub closed: bool,
    /// Predicted seconds (`∞` when not closed).
    pub price: f64,
    /// What the closure test found (`t_eff=…`, `chi=…`, …).
    pub detail: String,
}

/// The search's whole output for one query.
#[derive(Clone, Debug)]
pub struct Selection {
    pub priced: Vec<Priced>,
    pub selected: View,
    pub sector_line: String,
    pub t_eff: usize,
    pub probe: Probe,
    pub line: String,
    pub wall_search: f64,
}

impl Selection {
    pub fn price_of(&self, v: View) -> f64 {
        self.priced[v.index()].price
    }
    /// The closed views ordered by price.
    pub fn ranking(&self) -> Vec<View> {
        let mut v: Vec<&Priced> = self.priced.iter().filter(|p| p.closed).collect();
        v.sort_by(|a, b| a.price.total_cmp(&b.price).then(a.view.cmp(&b.view)));
        v.iter().map(|p| p.view).collect()
    }
}

/// The price of the sum view on a located sector (seconds), before anything is built.
pub fn sum_price(q: &Query, sec: &Sector, k: &Constants) -> f64 {
    let nb = crate::magic5::expected_branches(sec.t_eff) as f64;
    let legs = sum_legs(q, sec) as f64;
    let (n, t) = (q.n as f64, sec.t_eff as f64);
    let g = match q.obs {
        Observable::Amplitude(_) => sec.drop_removed(&q.gates).len() as f64,
        Observable::Marginal { .. } => sec.light_cone_circuit(&q.gates).len() as f64,
    };
    k.c_sb * nb * (g + t) * (n + t) + k.c_sa * nb * legs * (n + t) * (n + t)
}

/// The MPS price at bond cap `chi`.
pub fn mps_price(q: &Query, chi: usize, k: &Constants) -> f64 {
    let ops = two_site_ops(q.n, &q.gates) as f64;
    ops * (k.c_m0 + k.c_m3 * (chi as f64).powi(3))
}

/// THE SEARCH. Every closure test, every closed view priced, the cheapest selected. `scale`
/// multiplies each view's predicted price (all `1.0` in the campaign; PQ-4 halves one). Reads
/// `q.n`, `q.gates`, `q.obs`, `q.eps` and nothing else — never `q.label`.
pub fn select_scaled(q: &Query, k: &Constants, mode: ProbeMode, scale: [f64; 4]) -> Selection {
    let t0 = Instant::now();
    let sec = sector::locate(&q.gates, &q.obs);
    let g = q.gates.len() as f64;
    let n = q.n as f64;
    let mut priced = Vec::with_capacity(4);
    // TABLEAU
    let tab_closed = sec.t_eff == 0;
    priced.push(Priced {
        view: View::Tableau,
        closed: tab_closed,
        price: if tab_closed { scale[0] * k.c_tab * n * n * g } else { f64::INFINITY },
        detail: format!("t_eff={} removed={}", sec.t_eff, sec.removed.len()),
    });
    // SUM
    priced.push(Priced {
        view: View::Sum,
        closed: true,
        price: scale[1] * sum_price(q, &sec, k),
        detail: format!(
            "t_eff={} N={} legs={}",
            sec.t_eff,
            crate::magic5::expected_branches(sec.t_eff),
            sum_legs(q, &sec)
        ),
    });
    // MPS
    let deadline = Some(Instant::now() + Duration::from_secs_f64(CAP_S));
    let pr = probe(q, mode, deadline);
    let (mclosed, mprice, mdetail) = match pr.chi {
        Some(c) => (true, scale[2] * mps_price(q, c, k), format!("chi={c} probe={}", mode.name())),
        None => (
            false,
            f64::INFINITY,
            format!(
                "not closed at chi<=16 (probe={}, projected at 16: {:.3e} > eps {:.3e})",
                mode.name(),
                pr.at.last().map_or(f64::NAN, |a| a.projected),
                q.eps
            ),
        ),
    };
    priced.push(Priced { view: View::Mps, closed: mclosed, price: mprice, detail: mdetail });
    // DENSE
    let dclosed = q.n <= 24;
    priced.push(Priced {
        view: View::Dense,
        closed: dclosed,
        price: if dclosed { scale[3] * k.c_dense * (1u64 << q.n) as f64 * g } else { f64::INFINITY },
        detail: format!("n={}", q.n),
    });
    let mut best = View::Dense;
    let mut bp = f64::INFINITY;
    for p in &priced {
        if p.closed && p.price < bp {
            bp = p.price;
            best = p.view;
        }
    }
    let alts: Vec<String> = priced
        .iter()
        .filter(|p| p.view != best)
        .map(|p| {
            if p.closed {
                format!("{}={:.3e}", p.view.name(), p.price)
            } else {
                format!("{}=open", p.view.name())
            }
        })
        .collect();
    let line = format!("select: view={} price={:.3e} alternatives=[{}]", best.name(), bp, alts.join(", "));
    Selection {
        priced,
        selected: best,
        sector_line: sec.line(),
        t_eff: sec.t_eff,
        probe: pr,
        line,
        wall_search: t0.elapsed().as_secs_f64(),
    }
}

/// [`select_scaled`] with every price as measured.
pub fn select(q: &Query, k: &Constants, mode: ProbeMode) -> Selection {
    select_scaled(q, k, mode, [1.0; 4])
}

// ===========================================================================
// running a view
// ===========================================================================

/// One view run: its answer, its certificate on the observable, its wall.
#[derive(Clone, Debug)]
pub struct ViewRun {
    pub view: View,
    pub value: Option<ObsValue>,
    pub certificate: f64,
    pub wall: f64,
    /// `true` if the view produced an answer whose certificate reaches `ε`.
    pub reached: bool,
    pub note: String,
    /// MPS: the `χ` it ran at; SUM: the executed fraction.
    pub chi: Option<usize>,
    pub fraction: Option<f64>,
}

impl ViewRun {
    fn none(view: View, note: String) -> ViewRun {
        ViewRun { view, value: None, certificate: f64::INFINITY, wall: f64::INFINITY, reached: false, note, chi: None, fraction: None }
    }
}

/// Run one view on a query at acuity `eps` (the sum takes `eps`; the MPS takes `chi`). The wall is
/// the median over a few repetitions when a run is short. `stop` governs the MPS only.
pub fn run_view(q: &Query, view: View, eps: f64, chi: Option<usize>, stop: MpsStop) -> ViewRun {
    let deadline = Some(Instant::now() + Duration::from_secs_f64(CAP_S));
    match view {
        View::Tableau => {
            let sec = sector::locate(&q.gates, &q.obs);
            if sec.t_eff != 0 {
                return ViewRun::none(view, format!("not closed (t_eff={})", sec.t_eff));
            }
            let (v, w) = timed(5, || tableau_run(q, &sec));
            ViewRun { view, value: Some(v), certificate: 0.0, wall: w, reached: true, note: String::new(), chi: None, fraction: None }
        }
        View::Sum => {
            let sec = sector::locate(&q.gates, &q.obs);
            if crate::magic5::expected_branches(sec.t_eff) > (1 << 18) {
                return ViewRun::none(view, format!("priced > {CAP_S} s (N={})", crate::magic5::expected_branches(sec.t_eff)));
            }
            let t0 = Instant::now();
            let first = sum_run(q, &sec, eps, deadline);
            let w0 = t0.elapsed().as_secs_f64();
            match first {
                None => ViewRun::none(view, format!("> {CAP_S} s")),
                Some(r) => {
                    let (r, w) = if w0 < 0.05 {
                        let (r2, w) = timed(5, || sum_run(q, &sec, eps, None).expect("no deadline"));
                        (r2, w)
                    } else {
                        (r, w0)
                    };
                    if w > CAP_S {
                        return ViewRun::none(view, format!("> {CAP_S} s ({w:.1} s)"));
                    }
                    let frac = if r.total == 0 { 0.0 } else { r.evaluated as f64 / r.total as f64 };
                    ViewRun {
                        view,
                        value: Some(r.value),
                        certificate: r.certificate,
                        wall: w,
                        reached: r.certificate <= eps,
                        note: format!("k/N={}/{} legs={}", r.evaluated, r.total, r.legs),
                        chi: None,
                        fraction: Some(frac),
                    }
                }
            }
        }
        View::Mps => {
            let c = chi.expect("the MPS view needs a chi");
            let first = mps_run(q, c, q.gates.len(), stop, deadline);
            let r = if first.wall < 0.05 && first.value.is_some() {
                let (r2, w) = timed(5, || mps_run(q, c, q.gates.len(), stop, None));
                MpsRun { wall: w, ..r2 }
            } else {
                first
            };
            let reached = r.value.is_some() && r.certificate <= eps;
            ViewRun {
                view,
                value: r.value,
                certificate: r.certificate,
                wall: if r.value.is_some() { r.wall } else { f64::INFINITY },
                reached,
                note: format!(
                    "chi={} max_bond={} truncations={} ops={}{}",
                    c,
                    r.max_bond,
                    r.truncations,
                    r.ops,
                    r.stopped.map_or(String::new(), |s| format!(" stopped: {s}"))
                ),
                chi: Some(c),
                fraction: None,
            }
        }
        View::Dense => {
            if q.n > 24 {
                return ViewRun::none(view, "n > 24".into());
            }
            let first_t = Instant::now();
            let first = dense_run(q.n, &q.gates, &q.obs, deadline);
            let w0 = first_t.elapsed().as_secs_f64();
            match first {
                None => ViewRun::none(view, format!("> {CAP_S} s")),
                Some(v) => {
                    let (v, w) = if w0 < 0.05 {
                        timed(5, || dense_run(q.n, &q.gates, &q.obs, None).expect("no deadline"))
                    } else {
                        (v, w0)
                    };
                    ViewRun { view, value: Some(v), certificate: 0.0, wall: w, reached: true, note: String::new(), chi: None, fraction: None }
                }
            }
        }
    }
}

/// The MPS view BY HAND: the smallest `χ ∈ {2,4,…,64}` whose certificate reaches `ε`, each attempt
/// stopped the moment its certificate passes `ε`; the wall reported is the successful attempt's.
pub fn mps_by_hand(q: &Query) -> (ViewRun, f64) {
    let t0 = Instant::now();
    let mut last = ViewRun::none(View::Mps, "no chi <= 64 reached eps".into());
    for c in [2usize, 4, 8, 16, 32, 64] {
        if t0.elapsed().as_secs_f64() > CAP_S {
            last = ViewRun::none(View::Mps, format!("search over chi passed {CAP_S} s"));
            break;
        }
        let r = run_view(q, View::Mps, q.eps, Some(c), MpsStop::CertOver(q.eps));
        if r.reached {
            return (r, t0.elapsed().as_secs_f64());
        }
        last = ViewRun { note: format!("{} (last tried)", r.note), ..ViewRun::none(View::Mps, String::new()) };
    }
    (last, t0.elapsed().as_secs_f64())
}

// ===========================================================================
// the instance families (§2) — generated here, labelled for the record only
// ===========================================================================

/// Brickwork on a line: an `H` layer, then `depth` layers each of `CX` on alternating
/// nearest-neighbour pairs followed by one single-qubit gate per qubit; exactly `t` of those
/// single-qubit slots are `T`/`T†` (the rest `H` or `S`). `t ≤ depth · n`.
pub fn brickwork(n: usize, depth: usize, t: usize, seed: u64) -> Circuit {
    assert!(t <= depth * n, "brickwork: t = {t} needs depth·n ≥ t");
    let mut rng = sector::Rng(seed ^ 0x6272_6963_6b77_6f72);
    let slots = depth * n;
    let mut is_t = vec![false; slots];
    let mut placed = 0;
    while placed < t {
        let p = rng.below(slots);
        if !is_t[p] {
            is_t[p] = true;
            placed += 1;
        }
    }
    let mut gates = Vec::new();
    for q in 0..n {
        gates.push(Gate::H(q));
    }
    for d in 0..depth {
        let mut a = d % 2;
        while a + 1 < n {
            gates.push(Gate::Cx(a, a + 1));
            a += 2;
        }
        for q in 0..n {
            let g = if is_t[d * n + q] {
                if rng.below(2) == 0 {
                    Gate::T(q)
                } else {
                    Gate::Tdg(q)
                }
            } else if rng.below(2) == 0 {
                Gate::H(q)
            } else {
                Gate::S(q)
            };
            gates.push(g);
        }
    }
    Circuit { n_qubits: n, gates }
}

/// A family instance, as generated. `family` is carried for the RECORD and is never put into
/// the [`Query`] the search receives.
#[derive(Clone, Debug)]
pub struct Instance {
    pub id: usize,
    /// Index within its family (the generator's `i`).
    pub index: usize,
    pub family: char,
    pub seed: u64,
    pub n: usize,
    pub t: usize,
    pub circuit: Circuit,
    pub observable: Observable,
    pub eps_rel: f64,
    pub eps: f64,
}

impl Instance {
    /// What the search is given: no family.
    pub fn query(&self) -> Query {
        Query { n: self.n, gates: self.circuit.gates.clone(), obs: self.observable.clone(), eps: self.eps, label: None }
    }
    /// PQ-5's other arm: the same query with the family REVEALED.
    pub fn query_labelled(&self) -> Query {
        Query { label: Some(self.family.to_string()), ..self.query() }
    }
}

/// The L-family depth (entangling layers).
pub const L_DEPTH: usize = 6;

/// Generate one family's `count` instances. Instance `i`: `n = [12,16,20][i % 3]` (D: 12), the
/// observable an amplitude for even `i` and the 4-qubit marginal on qubits `0..4` for odd `i`,
/// `ε_rel = 10⁻¹` when `(i/2)` is even and `10⁻²` otherwise. The amplitude's `y` (and the
/// marginal's pattern, its first four bits) is the referee's argmax — QVM-ACUITY-1's rule,
/// declared before any view runs.
pub fn family(fam: char, count: usize, seed: u64) -> Vec<Instance> {
    (0..count).map(|i| family_instance(fam, i, seed)).collect()
}

/// Instance `i` of family `fam` alone (the rule in [`family`]).
pub fn family_instance(fam: char, i: usize, seed: u64) -> Instance {
    {
        let s = seed.wrapping_mul(1_000_003).wrapping_add(i as u64 * 7919 + fam as u64);
        let n = if fam == 'D' { 12 } else { [12, 16, 20][i % 3] };
        let (t, circuit) = match fam {
            'C' => (0, sector::random_instance(n, 0, s)),
            'T' => {
                let t = [12, 16, 20][(i / 3) % 3];
                (t, sector::random_instance(n, t, s))
            }
            'L' => {
                let t = 40 + (s % 9) as usize;
                (t, brickwork(n, L_DEPTH, t, s))
            }
            'D' => {
                let t = 40 + (s % 9) as usize;
                (t, sector::random_instance(n, t, s))
            }
            _ => panic!("views: no family {fam}"),
        };
        let y = sector::argmax_bitstring(n, &circuit.gates);
        let (observable, scale) = if i.is_multiple_of(2) {
            (Observable::Amplitude(y), (2f64).powf(-(n as f64) / 2.0))
        } else {
            (Observable::Marginal { qubits: vec![0, 1, 2, 3], bits: y[..4].to_vec() }, 1.0 / 16.0)
        };
        let eps_rel = if (i / 2).is_multiple_of(2) { 1e-1 } else { 1e-2 };
        Instance { id: 0, index: i, family: fam, seed: s, n, t, circuit, observable, eps_rel, eps: eps_rel * scale }
    }
}

/// All four families, `count` each, SHUFFLED by `seed` (Fisher–Yates on splitmix64) and numbered
/// in the shuffled order.
pub fn campaign(count: usize, seed: u64) -> Vec<Instance> {
    let mut all = Vec::new();
    for f in ['C', 'T', 'L', 'D'] {
        all.extend(family(f, count, seed));
    }
    let mut rng = sector::Rng(seed ^ 0x7368_7566_666c_6521);
    for i in (1..all.len()).rev() {
        let j = rng.below(i + 1);
        all.swap(i, j);
    }
    for (k, inst) in all.iter_mut().enumerate() {
        inst.id = k;
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rand_mat(m: usize, n: usize, seed: u64) -> Mat {
        let mut r = sector::Rng(seed);
        let mut a = Mat::zeros(m, n);
        for z in a.a.iter_mut() {
            *z = Z::new(r.next_u64() as f64 / u64::MAX as f64 - 0.5, r.next_u64() as f64 / u64::MAX as f64 - 0.5);
        }
        a
    }

    fn max_diff(a: &Mat, b: &Mat) -> f64 {
        a.a.iter().zip(&b.a).map(|(x, y)| (*x - *y).abs()).fold(0.0, f64::max)
    }

    #[test]
    fn qr_reconstructs_and_is_orthonormal() {
        for (m, n) in [(6, 3), (3, 6), (4, 4), (8, 2)] {
            let a = rand_mat(m, n, 3 + m as u64 * 10 + n as u64);
            let (q, r) = qr(&a);
            assert!(max_diff(&q.matmul(&r), &a) < 1e-13);
            let qq = q.adjoint().matmul(&q);
            for i in 0..qq.m {
                for j in 0..qq.n {
                    let e = if i == j { 1.0 } else { 0.0 };
                    assert!((qq.at(i, j) - Z::new(e, 0.0)).abs() < 1e-13);
                }
            }
        }
    }

    #[test]
    fn svd_reconstructs() {
        for (m, n) in [(6, 3), (3, 6), (8, 8), (16, 4)] {
            let a = rand_mat(m, n, 99 + m as u64 * 3 + n as u64);
            let (u, s, vh) = svd(&a);
            let mut us = u.clone();
            for i in 0..us.m {
                for j in 0..us.n {
                    us.a[i * us.n + j] = us.a[i * us.n + j].scale(s[j]);
                }
            }
            assert!(max_diff(&us.matmul(&vh), &a) < 1e-12, "svd {m}x{n}");
            for w in s.windows(2) {
                assert!(w[0] >= w[1]);
            }
        }
    }

    #[test]
    fn untruncated_mps_matches_dense_on_a_routed_random_circuit() {
        let c = sector::random_instance_depth(8, 6, 5, 60);
        let q = Query { n: 8, gates: c.gates.clone(), obs: Observable::Amplitude(vec![false; 8]), eps: 0.0, label: None };
        let (ops, _) = route(8, &q.gates);
        let mut m = Mps::zero_state(8, 1 << 8);
        for op in ops {
            m.apply_op(op);
        }
        let sv = sector::referee::statevector(8, &q.gates);
        let d = m.to_dense();
        let err: f64 = sv.iter().zip(&d).map(|(a, b)| (Z::new(a.0, a.1) - *b).abs()).fold(0.0, f64::max);
        assert!(err < 1e-12, "mps vs referee {err}");
        assert!(m.delta < 1e-12);
    }

    #[test]
    fn mps_marginal_matches_dense() {
        let c = brickwork(8, 4, 10, 3);
        let obs = Observable::Marginal { qubits: vec![0, 1, 2, 3], bits: vec![true, false, true, false] };
        let q = Query { n: 8, gates: c.gates.clone(), obs: obs.clone(), eps: 0.0, label: None };
        let r = mps_run(&q, 64, q.gates.len(), MpsStop::Never, None);
        let ref_v = sector::referee::value(8, &c.gates, &obs);
        assert!(r.value.unwrap().abs_diff(ref_v) < 1e-12);
    }

    #[test]
    fn dense_view_matches_referee() {
        let c = sector::random_instance(10, 8, 4);
        let obs = Observable::Amplitude(sector::argmax_bitstring(10, &c.gates));
        let v = dense_run(10, &c.gates, &obs, None).unwrap();
        assert!(v.abs_diff(sector::referee::value(10, &c.gates, &obs)) < 1e-12);
    }
}
