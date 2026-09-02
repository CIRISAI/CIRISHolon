//! The solver's algebra, written once: Jacobi and Davidson generic over [`Scalar`]. The sigma
//! lives in `lanes.rs`, generic the same way, and the f64 entry points in `fci.rs` delegate
//! to both. The `Dd` instantiation is the overflow tier for solves that stagnate at the f64
//! expansion floor: the f64 solve does the heavy lifting cold, [`refine_determinant_dd`] pays
//! double-double cost only for the last mile from that warm start.
//!
//! What is NOT here, on purpose: `sigma_reference` (the connected-determinant checker)
//! stays concrete in `fci.rs` — it exists to share no structure with the lane kernel, and
//! genericising it against the same trait would be the first step toward the drift this
//! crate's two-route discipline exists to prevent.

use crate::fci::{CiInts, FciSpace, SolveExit};
use crate::scalar::{Dd, Scalar};

// ------------------------------------------------------------------ generic helpers

/// `a · b` under the reduction law (`vecspace::blocked_dot`): the same bits as every host
/// thread count and as the device.
pub fn dot_t<T: Scalar>(a: &[T], b: &[T]) -> T {
    crate::vecspace::blocked_dot(a, b)
}

pub fn axpy_t<T: Scalar>(a: T, x: &[T], y: &mut [T]) {
    for (yi, xi) in y.iter_mut().zip(x.iter()) {
        *yi = *yi + a * *xi;
    }
}


pub fn norm_t<T: Scalar>(a: &[T]) -> T {
    dot_t(a, a).sqrt()
}


// ------------------------------------------------------------------ jacobi, one body

pub fn jacobi_eigh_t<T: Scalar>(a_in: &[T], n: usize) -> (Vec<T>, Vec<T>) {
    let mut a = a_in.to_vec();
    let mut v = vec![T::ZERO; n * n];
    for i in 0..n {
        v[i * n + i] = T::ONE;
    }
    for _ in 0..100 {
        let mut off = T::ZERO;
        for i in 0..n {
            for j in (i + 1)..n {
                off = off + a[i * n + j] * a[i * n + j];
            }
        }
        if off.to_f64() <= T::jacobi_off_floor() {
            break;
        }
        for p in 0..n {
            for q in (p + 1)..n {
                if a[p * n + q].abs().to_f64() < 1e-300 {
                    continue;
                }
                let theta = (a[q * n + q] - a[p * n + p]) / a[p * n + q].scale(2.0);
                let t = if theta.to_f64() >= 0.0 {
                    T::ONE / (theta + (T::ONE + theta * theta).sqrt())
                } else {
                    -(T::ONE / (-theta + (T::ONE + theta * theta).sqrt()))
                };
                let c = T::ONE / (T::ONE + t * t).sqrt();
                let s = t * c;
                for k in 0..n {
                    let akp = a[k * n + p];
                    let akq = a[k * n + q];
                    a[k * n + p] = c * akp - s * akq;
                    a[k * n + q] = s * akp + c * akq;
                }
                for k in 0..n {
                    let apk = a[p * n + k];
                    let aqk = a[q * n + k];
                    a[p * n + k] = c * apk - s * aqk;
                    a[q * n + k] = s * apk + c * aqk;
                }
                for k in 0..n {
                    let vkp = v[k * n + p];
                    let vkq = v[k * n + q];
                    v[k * n + p] = c * vkp - s * vkq;
                    v[k * n + q] = s * vkp + c * vkq;
                }
            }
        }
    }
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&i, &j| a[i * n + i].partial_cmp(&a[j * n + j]).unwrap());
    let evals: Vec<T> = idx.iter().map(|&i| a[i * n + i]).collect();
    let mut evecs = vec![T::ZERO; n * n];
    for (newc, &oldc) in idx.iter().enumerate() {
        for r in 0..n {
            evecs[r * n + newc] = v[r * n + oldc];
        }
    }
    (evals, evecs)
}

// ------------------------------------------------------------------ davidson, one body

/// The Davidson driver, generic. Transcribed verbatim from the concrete f64 driver —
/// every policy (symmetry-breaking perturbation, thick restart, two-candidate expansion,
/// the degeneracy-safe preconditioner) is unchanged and now exists ONCE. The residual is
/// returned in f64: it is an absolute norm, representable at any tier's floor.
#[allow(clippy::type_complexity)]
pub fn davidson_eigh_from_t<T: Scalar>(
    space: &FciSpace,
    k: &[T],
    g: &[T],
    diag: &[T],
    tol: f64,
    max_iter: usize,
    start_vector: Option<&[T]>,
) -> (T, Vec<T>, usize, f64, SolveExit) {
    // ONE driver body, in `davidson_eigh_from_op`, parameterised by the sigma OPERATOR rather
    // than by the three arguments that name the host one. This is its host instantiation: the
    // operator it builds is the lane kernel on the tables of `(space, k, g)`.
    let mut op = crate::sigma_op::CpuSigma::new(space, k, g);
    davidson_eigh_from_op(&mut op, diag, tol, max_iter, start_vector)
}

/// [`davidson_eigh_from_t`] over an arbitrary sigma OPERATOR — the device-class arm's entry
/// point (RESOURCE_DESIGN D0).
///
/// The driver logic stays host-side and is unchanged: same start-vector policy, same
/// symmetry-breaking perturbation, same thick restart, same two-candidate expansion, same
/// degeneracy-safe preconditioner, same exit reasons. The ONLY thing the operator changes is
/// where `H c` is computed — which is the whole cost, and which is also the whole artifact:
/// the device class travels on the solution because it is the operator's regime.
///
/// # Why the class is not an argument here
///
/// It is a property of the operator, and asking the caller to pass it alongside would create a
/// second place for the same fact to live and a first place for it to disagree. A caller that
/// wants the class asks the operator.
pub fn davidson_eigh_from_op<T: Scalar, S: crate::sigma_op::SigmaOp<T> + ?Sized>(
    op: &mut S,
    diag: &[T],
    tol: f64,
    max_iter: usize,
    start_vector: Option<&[T]>,
) -> (T, Vec<T>, usize, f64, SolveExit) {
    davidson_eigh_from_op_sub(op, diag, tol, max_iter, start_vector, crate::budget::DAVIDSON_SUBSPACE_MAX)
}

/// [`davidson_eigh_from_op`] with the SUBSPACE BOUND stated by the caller.
///
/// The bound is the working set: `2·max_sub` vectors of `n_det` entries, and it is what the
/// admission door prices (`budget::price_determinant_with`). A space the door refuses at 48
/// is admitted at 12 with the same driver, the same policies and the same code — more
/// iterations, each cheaper, and the Ritz values it converges to are the same eigenvalues.
/// Two solves under different bounds are two artifacts (the bound travels on the manifest
/// beside device class and iteration budget), which is why the default is a named constant
/// and not a literal here.
///
/// This is the HOST instantiation of [`davidson_in`]: the vectors live in host memory and
/// the operator is any [`SigmaOp`](crate::sigma_op::SigmaOp).
pub fn davidson_eigh_from_op_sub<T: Scalar, S: crate::sigma_op::SigmaOp<T> + ?Sized>(
    op: &mut S,
    diag: &[T],
    tol: f64,
    max_iter: usize,
    start_vector: Option<&[T]>,
    max_sub: usize,
) -> (T, Vec<T>, usize, f64, SolveExit) {
    let sp = crate::vecspace::HostSpace::new();
    let mut apply = |v: &Vec<T>, w: &mut Vec<T>| op.apply(v, w);
    davidson_in(&sp, &mut apply, diag, tol, max_iter, start_vector, max_sub)
}

/// THE Davidson driver, written once against a [`VectorSpace`](crate::vecspace::VectorSpace):
/// the vectors live wherever the space keeps them (host memory, or the device), `apply` is
/// `w = H v` on the space's vectors, and every reduction follows the space's law.
///
/// Read sideways, one iteration is four row programs and the operator (see `vecspace.rs`):
/// the Gram row of the newest image, the Ritz transform (x, r, corr and their reductions in
/// one pass), and two deflation passes that orthogonalise the candidate against the basis
/// (classical Gram–Schmidt, re-orthogonalised). The policies are the driver's own — the
/// symmetry-breaking perturbation of the start, the lowest-diagonal start, the thick restart
/// carrying the Ritz vector and its projected residual, the two-candidate expansion, the
/// degeneracy-safe preconditioner, the three exit reasons — and they exist here and nowhere
/// else. The residual is returned in f64: it is an absolute norm, representable at any tier.
#[allow(clippy::type_complexity)]
pub fn davidson_in<T: Scalar, S: crate::vecspace::VectorSpace<T>>(
    sp: &S,
    apply: &mut dyn FnMut(&S::V, &mut S::V),
    diag_host: &[T],
    tol: f64,
    max_iter: usize,
    start_vector: Option<&[T]>,
    max_sub: usize,
) -> (T, Vec<T>, usize, f64, SolveExit) {
    let nd = diag_host.len();
    if let Some(v) = start_vector {
        assert_eq!(
            v.len(),
            nd,
            "davidson warm start carries {} entries for a space of {} determinants; that \
             is a start vector from a different geometry's space, not a guess at this one",
            v.len(),
            nd
        );
    }
    if nd == 1 {
        let one = sp.upload(&[T::ONE]);
        let mut s = sp.zeros(1);
        apply(&one, &mut s);
        return (sp.download(&s)[0], vec![T::ONE], 0, 0.0, SolveExit::Trivial);
    }
    let max_sub = max_sub.max(2).min(nd);
    let diag = sp.upload(diag_host);
    let mut basis: Vec<S::V> = Vec::new();
    let mut hbasis: Vec<S::V> = Vec::new();
    // The Gram matrix `<b_i|H|b_j>` is CACHED: a basis vector never changes once pushed, so
    // its row and column are computed once and reused until a restart clears the basis.
    let mut gram: Vec<T> = vec![T::ZERO; max_sub * max_sub];
    let mut gram_m = 0usize;

    let start = diag_host
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, _)| i)
        .unwrap();

    // Symmetry-breaking perturbation, deterministic; see the f64 driver's original
    // comment (carried in fci.rs) for why it is not a hedge.
    let mut v0 = match start_vector {
        Some(w) => w.to_vec(),
        None => vec![T::ZERO; nd],
    };
    let mut seed = 0x9e37_79b9_7f4a_7c15u64;
    for x in v0.iter_mut() {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *x = *x + T::from_f64(1e-3 * (((seed >> 33) as f64 / (1u64 << 30) as f64) - 1.0));
    }
    if start_vector.is_none() {
        v0[start] = v0[start] + T::ONE;
    }
    let normalised = |v: &mut S::V| {
        let n = sp.norm(v);
        if n.to_f64() > 0.0 {
            sp.scale(T::ONE / n, v);
        }
    };
    let mut b0 = sp.upload(&v0);
    normalised(&mut b0);
    basis.push(b0);

    let mut theta = diag_host[start];
    let mut x = sp.copy(&basis[0]);
    let mut r = sp.zeros(nd);
    let mut corr = sp.zeros(nd);
    let mut resid = f64::INFINITY;

    for iter in 0..max_iter {
        while hbasis.len() < basis.len() {
            let mut w = sp.zeros(nd);
            apply(&basis[hbasis.len()], &mut w);
            hbasis.push(w);
        }
        let m = basis.len();
        // new rows: <b_i | hb_j> for every j; new columns: <b_i | hb_j> for the old i
        for i in gram_m..m {
            let row = sp.gram_row(&hbasis[..m], &basis[i]);
            gram[i * max_sub..i * max_sub + m].copy_from_slice(&row);
        }
        if gram_m > 0 {
            for j in gram_m..m {
                let col = sp.gram_row(&basis[..gram_m], &hbasis[j]);
                for (i, v) in col.into_iter().enumerate() {
                    gram[i * max_sub + j] = v;
                }
            }
        }
        gram_m = m;
        let mut sub = vec![T::ZERO; m * m];
        for i in 0..m {
            for j in 0..m {
                sub[i * m + j] = gram[i * max_sub + j];
            }
        }
        for i in 0..m {
            for j in 0..i {
                let a = (sub[i * m + j] + sub[j * m + i]).scale(0.5);
                sub[i * m + j] = a;
                sub[j * m + i] = a;
            }
        }
        let (evals, evecs) = jacobi_eigh_t(&sub, m);
        theta = evals[0];
        let y: Vec<T> = (0..m).map(|i| evecs[i * m]).collect();
        let (rr, bt_corr) = sp.ritz(&basis, &hbasis, &y, theta, &diag, &mut x, &mut r, &mut corr);
        resid = rr.sqrt().to_f64();
        if resid < tol {
            return (theta, sp.download(&x), iter + 1, resid, SolveExit::Converged);
        }
        if iter + 1 == max_iter {
            return (theta, sp.download(&x), iter + 1, resid, SolveExit::IterationCap);
        }
        if basis.len() >= max_sub {
            let mut d = sp.copy(&r);
            let p = sp.dot(&x, &d);
            sp.axpy(-p, &x, &mut d);
            let nd_ = sp.norm(&d).to_f64();
            basis.clear();
            hbasis.clear();
            gram_m = 0;
            let mut bx = sp.copy(&x);
            normalised(&mut bx);
            basis.push(bx);
            if nd_ > T::expansion_floor() {
                sp.scale(T::from_f64(1.0 / nd_), &mut d);
                basis.push(d);
            }
            continue;
        }

        // The corrector first; the raw residual is the second candidate and is built ONLY if
        // the first fails to expand the basis. Each candidate is deflated against the basis
        // twice (classical Gram–Schmidt, re-orthogonalised), every pass reading the basis once.
        let mut added = false;
        let mut first = true;
        loop {
            let (mut w, p1) = if first {
                (sp.copy(&corr), bt_corr.clone())
            } else {
                let w = sp.copy(&r);
                let p = sp.gram_row(&basis, &w);
                (w, p)
            };
            let p2 = sp.deflate(&basis, &p1, &mut w);
            let nn = sp.deflate_norm(&basis, &p2, &mut w);
            let nw = nn.sqrt().to_f64();
            if nw > T::expansion_floor() {
                sp.scale(T::from_f64(1.0 / nw), &mut w);
                basis.push(w);
                added = true;
                break;
            }
            if !first {
                break;
            }
            first = false;
        }
        if !added {
            return (theta, sp.download(&x), iter + 1, resid, SolveExit::Stagnated);
        }
    }
    (theta, sp.download(&x), max_iter, resid, SolveExit::IterationCap)
}

// ------------------------------------------------------------------ the overflow tier

/// What the double-double refinement asks for, and what came back.
#[derive(Debug, Clone)]
pub struct RefinedSolution {
    /// The energy at the DD tier, both limbs.
    pub e: Dd,
    /// `e.to_f64()` — the value f64 consumers read.
    pub e_f64: f64,
    /// What the refinement moved the energy by, relative to the f64 tier's answer:
    /// the part of the eigenvalue f64 could never resolve.
    pub delta_vs_f64: f64,
    /// Final residual norm (absolute, f64-representable at any tier).
    pub residual: f64,
    pub iters: usize,
    pub exit: SolveExit,
}

/// The DEFAULT tolerance for DD refinement — REACHABLE, by measurement, which is the
/// whole point. The first value here was 1e-22 ("two orders above the DD floor"), and
/// the 2026-08-30 deep calibration showed it was the f64 tier's original disease
/// reproduced one rung up: no practical budget reaches it (Sb crawled to 1.99e-15 in
/// 4000 iterations with the eigenvalue identical to 21 digits from iteration ~400;
/// Te to 2.97e-13), so `Converged` could never fire and the exit label would have
/// stopped discriminating — an unreachable ask is a typo with force, wherever it
/// lives. 1e-12 is crossed comfortably by BOTH measured degeneracy classes, sits four
/// orders below any staked chemistry precision, and restores `Converged`'s meaning.
/// A caller wanting deeper passes its own ask through [`refine_determinant_dd`]'s
/// `tol` parameter — per-class, from the lease layer's measured boundaries, which is
/// where depth negotiation belongs.
///
/// CORRECTION from the confirmation run, recorded before anyone leans on the deep
/// numbers: the residual is NOT MONOTONE across thick restarts. Te crossed 5.75e-13 at
/// iteration 15, wandered up to 5.53e-12 by 400, and was back to 2.97e-13 at 4000 — so
/// a budget-CHECKPOINT reading of the residual is not a boundary, and the per-class
/// deep-tail split (Sb 1.99e-15 vs Te 2.97e-13 at 4000) is real data but NOT yet a
/// calibrated class boundary: at the reachable ask both classes converge in ~15
/// iterations, class-independent. Boundaries are measured AT EXIT under an ask that
/// means something, never read off a wandering sequence mid-flight.
pub const DD_REQUESTED_TOLERANCE: f64 = 1e-12;

/// Refine an f64-tier solution on the DOUBLE-DOUBLE tier.
///
/// The overflow move for a solve that stagnated at the f64 expansion floor: promote the
/// integrals losslessly (each f64 becomes `Dd{hi, 0}` — the model is the stored
/// integrals, and "exact in the model" now extends ~14 more digits down), start Davidson
/// from the f64 eigenvector, and pay double-double cost only for the iterations the warm
/// start leaves over. Same driver, same policies, same code — different scalar.
pub fn refine_determinant_dd(
    space: &FciSpace,
    ci: &CiInts,
    diag: &[f64],
    e_f64: f64,
    start: &[f64],
    tol: f64,
    max_iter: usize,
) -> RefinedSolution {
    let kdd: Vec<Dd> = ci.k.iter().map(|&v| Dd::from_f64(v)).collect();
    let gdd: Vec<Dd> = ci.g.iter().map(|&v| Dd::from_f64(v)).collect();
    let ddiag: Vec<Dd> = diag.iter().map(|&v| Dd::from_f64(v)).collect();
    let sdd: Vec<Dd> = start.iter().map(|&v| Dd::from_f64(v)).collect();
    let (e, _x, iters, residual, exit) = davidson_eigh_from_t::<Dd>(
        space,
        &kdd,
        &gdd,
        &ddiag,
        tol,
        max_iter,
        Some(&sdd),
    );
    RefinedSolution {
        e,
        e_f64: e.to_f64(),
        delta_vs_f64: e.to_f64() - e_f64,
        residual,
        iters,
        exit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The generic helpers at T = f64 must be the native operations in the native order.
    #[test]
    fn f64_helpers_bit_match_reference() {
        let a: Vec<f64> = (0..37).map(|i| ((i * 7919 % 101) as f64) * 0.013 - 0.5).collect();
        let b: Vec<f64> = (0..37).map(|i| ((i * 104729 % 97) as f64) * 0.017 - 0.7).collect();
        let d1: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert_eq!(dot_t(&a, &b), d1);
        let mut y1 = b.clone();
        for (yi, xi) in y1.iter_mut().zip(a.iter()) {
            *yi += 0.37 * xi;
        }
        let mut y2 = b.clone();
        axpy_t(0.37, &a, &mut y2);
        assert_eq!(y1, y2);
        assert_eq!(norm_t(&a), a.iter().map(|x| x * x).sum::<f64>().sqrt());
    }

    /// Jacobi at DD on a known 3x3: eigenvalues to ~1e-28.
    #[test]
    fn jacobi_dd_deep_eigenvalues() {
        // A = diag(1,2,3) rotated by a Givens rotation in the (0,1) plane, angle 0.3.
        let (c, s) = (0.3f64.cos(), 0.3f64.sin());
        let mut a = [0.0f64; 9];
        // R diag R^T with diag (1,2,3)
        let d = [1.0, 2.0, 3.0];
        let r = [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]];
        for i in 0..3 {
            for j in 0..3 {
                let mut acc = 0.0;
                for t in 0..3 {
                    acc += r[i][t] * d[t] * r[j][t];
                }
                a[i * 3 + j] = acc;
            }
        }
        let add: Vec<Dd> = a.iter().map(|&v| Dd::from_f64(v)).collect();
        let (evals, _) = jacobi_eigh_t::<Dd>(&add, 3);
        // The f64 inputs carry rounding, so recover only to f64-construction accuracy in
        // ABSOLUTE terms — but the JACOBI process itself must not add error above DD scale:
        // check the eigenvalue sum against the exact trace of the stored matrix.
        let trace = Dd::from_f64(a[0]) + Dd::from_f64(a[4]) + Dd::from_f64(a[8]);
        let sum = evals[0] + evals[1] + evals[2];
        assert!(
            (sum - trace).to_f64().abs() < 1e-28,
            "trace defect {}",
            (sum - trace).to_f64()
        );
        for (got, want) in evals.iter().zip([1.0, 2.0, 3.0]) {
            assert!((got.to_f64() - want).abs() < 1e-13, "{} vs {want}", got.to_f64());
        }
    }
}
