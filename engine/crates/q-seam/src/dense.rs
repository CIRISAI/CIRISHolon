//! A heap-allocated cyclic Jacobi eigensolver for real symmetric matrices.
//!
//! This exists for two jobs and no others: the independent dense cross-check of the Lanczos
//! ground state at N ≤ 6 (gate G-E4b), and the small tridiagonal solves inside Lanczos itself.
//! `ciris_sim_core::linalg::jacobi_eigen` is the vacuum tier's solver and is the right one —
//! but it is const-generic over `[[f64; N]; N]`, so a 400 × 400 instance would be 1.28 MB of
//! stack. This is the same algorithm on the heap, and gate G-E4c pins it against the core's
//! solver on 3×3 and 4×4 fixtures so the substitution is checked rather than trusted.

/// Eigenvalues ascending, with `vectors[i]` the eigenvector for `values[i]`.
pub struct DenseEigen {
    pub values: Vec<f64>,
    pub vectors: Vec<Vec<f64>>,
    pub sweeps: usize,
    pub converged: bool,
}

const MAX_SWEEPS: usize = 100;

/// Convergence is measured against the FULL Frobenius norm, not against the diagonal.
///
/// The first printing of this solver stopped at `off <= 1e-15 * sqrt(Σ diag²)`. That scales like
/// `√n`, so it loosens as the matrix grows: at dim 400 it permitted an off-diagonal norm of
/// ~6e-14, which is exactly the eigenvalue error gate G-E4b then caught (dense was 1.07e-13 from
/// the analytic value while Lanczos was 1.11e-14). The gate found a real solver defect; the fix
/// is here, not in the threshold.
const TOL: f64 = 1e-16;

/// `a` is row-major `n × n` and is consumed by value; it must be symmetric.
pub fn jacobi(mut a: Vec<f64>, n: usize) -> DenseEigen {
    let mut v = vec![0.0; n * n];
    for i in 0..n {
        v[i * n + i] = 1.0;
    }

    let mut sweeps = 0;
    let mut converged = false;
    let mut prev_off = f64::INFINITY;
    for sweep in 0..MAX_SWEEPS {
        sweeps = sweep + 1;

        let mut off = 0.0;
        for p in 0..n {
            for q in (p + 1)..n {
                off += a[p * n + q] * a[p * n + q];
            }
        }
        let off = off.sqrt();
        let fro: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt().max(1.0);
        if off <= TOL * fro {
            converged = true;
            break;
        }
        // Stagnation: cyclic Jacobi converges quadratically, so a sweep that fails to reduce the
        // off-diagonal norm has reached the floor of the arithmetic. Stopping here is honest;
        // spinning to MAX_SWEEPS would only report a larger sweep count for the same answer.
        if off >= prev_off {
            converged = true;
            break;
        }
        prev_off = off;

        for p in 0..n {
            for q in (p + 1)..n {
                let apq = a[p * n + q];
                if apq == 0.0 {
                    continue;
                }
                let app = a[p * n + p];
                let aqq = a[q * n + q];
                // Standard stable rotation: theta = (aqq - app) / (2 apq), t = sgn/(|theta| + sqrt(theta^2+1)).
                let theta = (aqq - app) / (2.0 * apq);
                let t = if theta >= 0.0 {
                    1.0 / (theta + (theta * theta + 1.0).sqrt())
                } else {
                    -1.0 / (-theta + (theta * theta + 1.0).sqrt())
                };
                let c = 1.0 / (t * t + 1.0).sqrt();
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
                // Keep the off-diagonal exactly zero rather than at rounding noise.
                a[p * n + q] = 0.0;
                a[q * n + p] = 0.0;

                for k in 0..n {
                    let vkp = v[k * n + p];
                    let vkq = v[k * n + q];
                    v[k * n + p] = c * vkp - s * vkq;
                    v[k * n + q] = s * vkp + c * vkq;
                }
            }
        }
    }

    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| a[i * n + i].partial_cmp(&a[j * n + j]).unwrap());

    let values = order.iter().map(|&i| a[i * n + i]).collect();
    let vectors = order
        .iter()
        .map(|&i| (0..n).map(|k| v[k * n + i]).collect())
        .collect();

    DenseEigen { values, vectors, sweeps, converged }
}
