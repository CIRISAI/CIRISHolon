//! The solver, written once: sigma, Jacobi and Davidson generic over [`Scalar`].
//!
//! These bodies are VERBATIM transcriptions of the concrete f64 functions that used to
//! live in `fci.rs` — same loop order, same operation shape — and the f64 functions now
//! delegate here. That is the bit-identity contract: monomorphised at `T = f64` this IS
//! the old code (IEEE operations in the same order), and the pinned-digest gates
//! (holon-tables G1, the atoms records) are the proof that holds it. The `Dd`
//! instantiation is the overflow tier for solves that stagnate at the f64 expansion
//! floor: the f64 solve does the heavy lifting cold, [`refine_determinant_dd`] pays
//! double-double cost only for the last mile from that warm start.
//!
//! What is NOT here, on purpose: `sigma_reference` (the connected-determinant checker)
//! stays concrete in `fci.rs` — it exists to share no structure with this factorisation,
//! and genericising it against the same trait would be the first step toward the drift
//! this crate's two-route discipline exists to prevent.

use crate::fci::{CiInts, FciSpace, SolveExit};
use crate::scalar::{Dd, Scalar};

// ------------------------------------------------------------------ generic helpers

pub fn dot_t<T: Scalar>(a: &[T], b: &[T]) -> T {
    let mut acc = T::ZERO;
    for (x, y) in a.iter().zip(b.iter()) {
        acc = acc + *x * *y;
    }
    acc
}

pub fn axpy_t<T: Scalar>(a: T, x: &[T], y: &mut [T]) {
    for (yi, xi) in y.iter_mut().zip(x.iter()) {
        *yi = *yi + a * *xi;
    }
}

pub fn scale_t<T: Scalar>(a: T, x: &mut [T]) {
    for xi in x.iter_mut() {
        *xi = *xi * a;
    }
}

pub fn norm_t<T: Scalar>(a: &[T]) -> T {
    dot_t(a, a).sqrt()
}

pub fn normalised_t<T: Scalar>(a: &[T]) -> Vec<T> {
    let mut v = a.to_vec();
    let n = norm_t(&v);
    if n.to_f64() > 0.0 {
        scale_t(T::ONE / n, &mut v);
    }
    v
}

// ------------------------------------------------------------------ sigma, one body

/// `sigma = H c` by the Knowles–Handy string factorisation, generic over the scalar.
/// The integrals arrive as slices in `T` (the caller promotes once per solve, not per
/// sigma); the excitation lists and their signs stay in their exact integer/f64 form.
pub fn sigma_direct_t<T: Scalar>(space: &FciSpace, k: &[T], g: &[T], c: &[T], sigma: &mut [T]) {
    let n = space.n_orb;
    let n2 = n * n;
    let na = space.alpha.len();
    let nb = space.beta.len();
    assert_eq!(c.len(), space.n_det);
    assert_eq!(sigma.len(), space.n_det);

    for x in sigma.iter_mut() {
        *x = T::ZERO;
    }

    let mut f = vec![T::ZERO; na.max(nb)];

    // 1. Beta same-spin block
    for jb in 0..nb {
        for x in f[..nb].iter_mut() {
            *x = T::ZERO;
        }
        for &(kl, s1, kb) in space.beta.singles[jb].iter() {
            f[kb as usize] = f[kb as usize] + k[kl as usize].scale(s1);
            for &(ij, s2, ib) in space.beta.singles[kb as usize].iter() {
                f[ib as usize] =
                    f[ib as usize] + g[ij as usize * n2 + kl as usize].scale(0.5 * s1 * s2);
            }
        }
        for (ib, &fv) in f[..nb].iter().enumerate() {
            if fv.is_zero() {
                continue;
            }
            for ia in 0..na {
                sigma[ia * nb + ib] = sigma[ia * nb + ib] + fv * c[ia * nb + jb];
            }
        }
    }

    // 2. Alpha same-spin block
    for ja in 0..na {
        for x in f[..na].iter_mut() {
            *x = T::ZERO;
        }
        for &(kl, s1, ka) in space.alpha.singles[ja].iter() {
            f[ka as usize] = f[ka as usize] + k[kl as usize].scale(s1);
            for &(ij, s2, ia) in space.alpha.singles[ka as usize].iter() {
                f[ia as usize] =
                    f[ia as usize] + g[ij as usize * n2 + kl as usize].scale(0.5 * s1 * s2);
            }
        }
        for (ia, &fv) in f[..na].iter().enumerate() {
            if fv.is_zero() {
                continue;
            }
            let (src, dst) = (ja * nb, ia * nb);
            for ib in 0..nb {
                sigma[dst + ib] = sigma[dst + ib] + fv * c[src + ib];
            }
        }
    }

    // 3. Mixed alpha-beta block
    //
    // ### ARITHMETIC-REGIME BOUNDARY: commit 4884704 (2026-08-31 18:02)
    //
    // The `kl` walk below is SPARSE and runs in FIRST-TOUCH order. It used to run over
    // every `kl` ascending (`grow.iter().enumerate()`). `vrow[ib]` accumulates over `kl`,
    // so this reassociates a floating-point sum: the optimisation is a real speedup and
    // changes no physics, and it changes the last bits of almost everything this crate
    // computes.
    //
    // MEASURED, on the shipped pair tables: 186 of 192 HCl energy knots moved, max
    // 3.979e-12 hartree, with the grid 0/192 different — tens of ULPs on a total of order
    // 455 Ha, 25x inside every declared uncertainty and inside R2's 1e-10 stake. No
    // tolerance gate can see it. Every gate asserting IDENTITY can, and three went red on
    // it in three different lanes, each blaming the last thing its own author had changed.
    // Bracketed to this commit against its parent 2b0d154, source-only: no Cargo.toml,
    // Cargo.lock or workspace change, same toolchain and profile.
    //
    // WHICH SYSTEMS MOVE IS NOT A SIZE RULE, and the first version of this note said it
    // was: right mechanism, wrong axis — what governs is whether the sparsity pattern's
    // first-touch order differs from ascending, per system. Measured on the dimer record:
    // Cl (9 determinants) moved, Br (18) did NOT, I (27) moved — the same one-hole
    // `C(n,n)·C(n,n-1)` shape at all three, so the effect is not monotone in determinant
    // count and a threshold on that axis mispredicts in both directions. H2, HHe and He2 are observed not to move, which is the only reason B1's
    // all-hydrogen bit-identity reference survived this commit; that is an observation
    // about those three curves and not a guarantee, so anything extending this walk — or
    // changing the order `touched_kl` is pushed in — should expect B1 and every banked
    // bit pattern to move, and should re-bank them naming the change.
    //
    // The regime boundary is this commit and it is ACCEPTED, not reverted (reverting
    // would split the running ozone tabulation's arithmetic mid-table). Artifacts banked
    // before it are records of the previous regime; see MISFITS.md, M-STALE-INSTRUMENT's
    // artifact-identity rung.
    let mut t = vec![T::ZERO; n2 * nb];
    let mut vrow = vec![T::ZERO; nb];
    let mut touched_kl: Vec<usize> = Vec::with_capacity(n2);
    let mut is_touched = vec![false; n2];

    for ja in 0..na {
        let crow = &c[ja * nb..(ja + 1) * nb];
        touched_kl.clear();

        #[allow(clippy::needless_range_loop)]
        for jb in 0..nb {
            let cv = crow[jb];
            if cv.is_zero() {
                continue;
            }
            for &(kl, s, ib) in space.beta.singles[jb].iter() {
                let kl_u = kl as usize;
                if !is_touched[kl_u] {
                    is_touched[kl_u] = true;
                    touched_kl.push(kl_u);
                }
                t[kl_u * nb + ib as usize] = t[kl_u * nb + ib as usize] + cv.scale(s);
            }
        }

        if touched_kl.is_empty() {
            continue;
        }

        for &(ij, sa, ia) in space.alpha.singles[ja].iter() {
            for x in vrow.iter_mut() {
                *x = T::ZERO;
            }
            let grow = &g[ij as usize * n2..(ij as usize + 1) * n2];
            for &kl in &touched_kl {
                let gv = grow[kl];
                if gv.is_zero() {
                    continue;
                }
                let trow = &t[kl * nb..(kl + 1) * nb];
                for ib in 0..nb {
                    vrow[ib] = vrow[ib] + gv * trow[ib];
                }
            }
            let dst = ia as usize * nb;
            for ib in 0..nb {
                sigma[dst + ib] = sigma[dst + ib] + vrow[ib].scale(sa);
            }
        }

        // Clean up only touched slices of t
        for &kl in &touched_kl {
            is_touched[kl] = false;
            for x in t[kl * nb..(kl + 1) * nb].iter_mut() {
                *x = T::ZERO;
            }
        }
    }
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
    // than by the three arguments that used to name the host one. This is its host
    // instantiation and it is a wrapper: the operator it builds calls the same
    // `sigma_direct_t` this function used to call directly, in the same place, so the path is
    // bit-identical and `sigma_op`'s own test asserts that against `sigma_direct`.
    let mut op = crate::sigma_op::CpuSigma::new(space, k, g);
    davidson_eigh_from_op(&mut op, diag, tol, max_iter, start_vector)
}

/// [`davidson_eigh_from_t`] over an arbitrary sigma OPERATOR — the device-class arm's entry
/// point (RESOURCE_DESIGN D0).
///
/// The driver logic stays host-side and is unchanged: same start-vector policy, same
/// symmetry-breaking perturbation, same thick restart, same two-candidate expansion, same
/// degeneracy-safe preconditioner, same exit reasons. The ONLY thing the operator changes is
/// where `H c` is computed — which is the whole cost, and which is also the whole artifact,
/// because two devices that agree to 3e-15 and differ on 91% of entries bitwise send this
/// driver down two different paths.
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
    let nd = op.n_det();
    assert_eq!(
        diag.len(),
        nd,
        "the preconditioner diagonal carries {} entries for an operator of dimension {nd};          those are two different spaces",
        diag.len()
    );
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
        let mut s = vec![T::ZERO; 1];
        op.apply(&[T::ONE], &mut s);
        return (s[0], vec![T::ONE], 0, 0.0, SolveExit::Trivial);
    }
    let max_sub = 48.min(nd);
    let mut basis: Vec<Vec<T>> = Vec::new();
    let mut hbasis: Vec<Vec<T>> = Vec::new();

    let start = diag
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
    basis.push(normalised_t(&v0));

    let mut theta = diag[start];
    let mut x = basis[0].clone();
    let mut resid = f64::INFINITY;

    for iter in 0..max_iter {
        while hbasis.len() < basis.len() {
            let mut w = vec![T::ZERO; nd];
            op.apply(&basis[hbasis.len()], &mut w);
            hbasis.push(w);
        }
        let m = basis.len();
        let mut sub = vec![T::ZERO; m * m];
        for i in 0..m {
            for j in 0..m {
                sub[i * m + j] = dot_t(&basis[i], &hbasis[j]);
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
        x = vec![T::ZERO; nd];
        for i in 0..m {
            let ci_ = evecs[i * m];
            if !ci_.is_zero() {
                axpy_t(ci_, &basis[i], &mut x);
            }
        }
        let mut hx = vec![T::ZERO; nd];
        for i in 0..m {
            let ci_ = evecs[i * m];
            if !ci_.is_zero() {
                axpy_t(ci_, &hbasis[i], &mut hx);
            }
        }
        let mut r = hx;
        axpy_t(-theta, &x, &mut r);
        resid = norm_t(&r).to_f64();
        if resid < tol {
            return (theta, x, iter + 1, resid, SolveExit::Converged);
        }
        if iter + 1 == max_iter {
            return (theta, x, iter + 1, resid, SolveExit::IterationCap);
        }
        if basis.len() >= max_sub {
            let mut d = r.clone();
            let p = dot_t(&x, &d);
            axpy_t(-p, &x, &mut d);
            let nd_ = norm_t(&d).to_f64();
            basis.clear();
            hbasis.clear();
            basis.push(normalised_t(&x));
            if nd_ > T::expansion_floor() {
                scale_t(T::from_f64(1.0 / nd_), &mut d);
                basis.push(d);
            }
            continue;
        }

        let mut corr = r.clone();
        for i in 0..nd {
            let d = theta - diag[i];
            if d.abs().to_f64() > 1e-8 {
                corr[i] = r[i] / d;
            }
        }
        let mut added = false;
        for cand in [corr, r.clone()] {
            let mut w = cand;
            for b in basis.iter() {
                let p = dot_t(b, &w);
                axpy_t(-p, b, &mut w);
            }
            for b in basis.iter() {
                let p = dot_t(b, &w);
                axpy_t(-p, b, &mut w);
            }
            let nw = norm_t(&w).to_f64();
            if nw > T::expansion_floor() {
                scale_t(T::from_f64(1.0 / nw), &mut w);
                basis.push(w);
                added = true;
                break;
            }
        }
        if !added {
            return (theta, x, iter + 1, resid, SolveExit::Stagnated);
        }
    }
    (theta, x, max_iter, resid, SolveExit::IterationCap)
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
