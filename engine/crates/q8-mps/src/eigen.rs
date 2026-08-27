//! A heap-allocated cyclic Jacobi eigensolver for real symmetric matrices — own implementation,
//! zero runtime dependencies (q-seam's `dense::jacobi` is dev-dependency-only, unusable from
//! `src/`). Same well-known algorithm q-seam's copy uses; this exists only for the small
//! tridiagonal Ritz sub-problem inside `lanczos.rs`'s two-site solve (typically well under 100
//! rows), never for anything bond-dimension-sized.
//!
//! Convergence is measured against the FULL Frobenius norm with a stagnation break — carrying
//! `q-seam`'s `dense.rs` house lesson forward rather than re-discovering it: a criterion that
//! scales like `sqrt(n)` loosens as the matrix grows.

pub struct DenseEigen {
    /// Ascending.
    pub values: Vec<f64>,
    /// `vectors[i]` is the eigenvector for `values[i]`.
    pub vectors: Vec<Vec<f64>>,
    pub converged: bool,
}

const MAX_SWEEPS: usize = 100;
const TOL: f64 = 1e-16;

/// `a` is row-major `n x n`, consumed by value; must be symmetric.
pub fn jacobi_eigen(mut a: Vec<f64>, n: usize) -> DenseEigen {
    let mut v = vec![0.0; n * n];
    for i in 0..n {
        v[i * n + i] = 1.0;
    }

    let mut converged = false;
    let mut prev_off = f64::INFINITY;
    for _sweep in 0..MAX_SWEEPS {
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
    let vectors = order.iter().map(|&i| (0..n).map(|k| v[k * n + i]).collect()).collect();

    DenseEigen { values, vectors, converged }
}
