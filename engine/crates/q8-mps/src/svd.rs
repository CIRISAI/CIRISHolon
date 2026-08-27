//! A heap-allocated one-sided (Hestenes) Jacobi SVD for general real matrices.
//!
//! `Q8_MPS_PREREG.md` §2.1 stakes this before anything downstream runs (G-SVD1/2/3): a bond's
//! truncation is only as honest as the SVD that produces it. The rotation itself is the same
//! math `q-seam`'s `dense.rs::jacobi` uses for a symmetric eigenproblem, applied instead to the
//! 2x2 Gram submatrix of a column pair — diagonalizing `[[a.a, a.b],[a.b, b.b]]` zeroes the
//! cross term, which is exactly "the two columns become orthogonal after rotation."  Unlike a
//! symmetric eigenproblem, an SVD must judge that orthogonality RELATIVE to each column's norm:
//! DMRG routinely carries Schmidt values across many decades, so an absolutely tiny Gram cross
//! term can still mean two normalized singular vectors are nearly parallel.  Wide matrices are
//! transposed before iteration so the one-sided (column) method always works on at most `m`
//! columns in `R^m`; the result is transposed back without changing the decomposition.

/// `u[i]`/`v[i]` are the i-th left/right singular vectors, `s[i]` descending, `k = min(m,n)`
/// triples — the economy SVD, which is what a bond truncation ever needs (`chi <= min(m,n)`
/// always).
///
/// Degenerate (near-zero) singular directions still get a genuine orthonormal `u[i]`, completed
/// by Gram-Schmidt against the standard basis rather than reported as the zero vector — a
/// rank-deficient reshape is the norm at a chain boundary or from a product-state seed, not the
/// exception, so `U^T U = I` must hold there too (G-SVD2 is staked without a rank carve-out).
pub struct Svd {
    pub u: Vec<Vec<f64>>,
    pub s: Vec<f64>,
    pub v: Vec<Vec<f64>>,
    pub sweeps: usize,
    pub converged: bool,
}

const MAX_SWEEPS: usize = 100;
/// Maximum pairwise correlation between retained working columns.  This directly bounds the
/// off-diagonal defect in `U^T U`; an absolute Gram tolerance cannot do that when singular values
/// span the strong-coupling DMRG spectrum.
const RELATIVE_ORTHOGONALITY_TOL: f64 = 1e-13;
/// Singular values at or below this fraction of the Frobenius norm are degenerate and their
/// `u` column is Gram-Schmidt completed rather than trusted from the raw (near-zero-norm)
/// rotated column.
const DEGENERATE_FLOOR_REL: f64 = 1e-12;

#[inline]
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[inline]
fn norm(a: &[f64]) -> f64 {
    dot(a, a).sqrt()
}

/// Rotate the length-`p<q` pair of vectors `cols[p], cols[q]` by the Givens pair `(c,s)`:
/// `cols[p]' = c·cols[p] - s·cols[q]`, `cols[q]' = s·cols[p] + c·cols[q]`. Shared by both the
/// working-matrix rotation and the accumulated right-rotation `vrot`.
#[inline]
fn rotate_pair(cols: &mut [Vec<f64>], p: usize, q: usize, c: f64, s: f64) {
    debug_assert!(p < q);
    let (left, right) = cols.split_at_mut(q);
    let cp = &mut left[p];
    let cq = &mut right[0];
    for (akp, akq) in cp.iter_mut().zip(cq.iter_mut()) {
        let (a, b) = (*akp, *akq);
        *akp = c * a - s * b;
        *akq = s * a + c * b;
    }
}

/// `a` is row-major `m x n`.
pub fn jacobi_svd(a: &[f64], m: usize, n: usize) -> Svd {
    assert_eq!(a.len(), m * n, "row-major m x n buffer expected");
    assert!(m > 0 && n > 0);

    // One-sided Jacobi orthogonalizes columns.  For a wide matrix, solve A^T = U_t S V_t^T
    // instead and return A = V_t S U_t^T.  Besides halving the working column count for the
    // DMRG-wide case, this makes the original left singular vectors accumulated rotations rather
    // than normalized near-null columns.
    if m < n {
        let mut transposed = vec![0.0; m * n];
        for i in 0..m {
            for j in 0..n {
                transposed[j * m + i] = a[i * n + j];
            }
        }
        let svd_t = jacobi_svd(&transposed, n, m);
        return Svd {
            u: svd_t.v,
            s: svd_t.s,
            v: svd_t.u,
            sweeps: svd_t.sweeps,
            converged: svd_t.converged,
        };
    }

    // Working copy as columns (each length m), so a rotation of columns p,q is a rotation of
    // two length-m vectors — the natural shape for the dot products the algorithm needs.
    let mut cols: Vec<Vec<f64>> = (0..n).map(|j| (0..m).map(|i| a[i * n + j]).collect()).collect();
    // Accumulated right-rotation, n x n, starts at the identity.
    let mut vrot: Vec<Vec<f64>> = (0..n)
        .map(|j| {
            let mut e = vec![0.0; n];
            e[j] = 1.0;
            e
        })
        .collect();

    let fro = norm(a).max(1.0);
    let active_floor_sq = (DEGENERATE_FLOOR_REL * fro).powi(2);
    let mut sweeps = 0;
    let mut converged = false;

    for sweep in 0..MAX_SWEEPS {
        sweeps = sweep + 1;

        // The normalized Gram off-diagonal is the canonical-basis invariant the caller needs.
        // Ignore genuinely degenerate columns here: they are completed explicitly below and do
        // not carry content at this numerical floor.
        let mut worst_correlation = 0.0f64;
        for p in 0..n {
            for q in (p + 1)..n {
                let alpha = dot(&cols[p], &cols[p]);
                let beta = dot(&cols[q], &cols[q]);
                if alpha > active_floor_sq && beta > active_floor_sq {
                    let correlation =
                        dot(&cols[p], &cols[q]).abs() / (alpha * beta).sqrt();
                    worst_correlation = worst_correlation.max(correlation);
                }
            }
        }
        if worst_correlation <= RELATIVE_ORTHOGONALITY_TOL {
            converged = true;
            break;
        }

        for p in 0..n {
            for q in (p + 1)..n {
                let alpha = dot(&cols[p], &cols[p]);
                let beta = dot(&cols[q], &cols[q]);
                let gamma = dot(&cols[p], &cols[q]);
                if gamma == 0.0 {
                    continue;
                }
                // Diagonalize the 2x2 Gram submatrix [[alpha,gamma],[gamma,beta]] — identical
                // stable-rotation formula to `dense.rs::jacobi`, applied to a Gram pair instead
                // of a general symmetric-matrix pair.
                let zeta = (beta - alpha) / (2.0 * gamma);
                let t = if zeta >= 0.0 {
                    1.0 / (zeta + (zeta * zeta + 1.0).sqrt())
                } else {
                    -1.0 / (-zeta + (zeta * zeta + 1.0).sqrt())
                };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;

                rotate_pair(&mut cols, p, q, c, s);
                rotate_pair(&mut vrot, p, q, c, s);
            }
        }
    }

    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| norm(&cols[j]).partial_cmp(&norm(&cols[i])).unwrap());

    let k = m.min(n);
    let floor = DEGENERATE_FLOOR_REL * fro;

    let mut s = Vec::with_capacity(k);
    let mut u: Vec<Vec<f64>> = Vec::with_capacity(k);
    let mut v: Vec<Vec<f64>> = Vec::with_capacity(k);
    let mut degenerate_slots = Vec::new();

    for &j in order.iter().take(k) {
        let sigma = norm(&cols[j]);
        s.push(sigma);
        v.push(vrot[j].clone());
        if sigma > floor {
            u.push(cols[j].iter().map(|x| x / sigma).collect());
        } else {
            degenerate_slots.push(u.len());
            u.push(vec![0.0; m]); // placeholder, completed below
        }
    }

    if !degenerate_slots.is_empty() {
        let mut basis: Vec<Vec<f64>> = u
            .iter()
            .enumerate()
            .filter(|(i, _)| !degenerate_slots.contains(i))
            .map(|(_, row)| row.clone())
            .collect();
        let mut filled = 0;
        for e in 0..m {
            if filled == degenerate_slots.len() {
                break;
            }
            let mut w = vec![0.0; m];
            w[e] = 1.0;
            // Two passes protect the completion from the same scale separation that motivated
            // the relative Jacobi criterion.
            for _ in 0..2 {
                for b in &basis {
                    let c = dot(&w, b);
                    for (wt, bt) in w.iter_mut().zip(b.iter()) {
                        *wt -= c * bt;
                    }
                }
            }
            let wn = norm(&w);
            if wn > 1e-8 {
                for wt in w.iter_mut() {
                    *wt /= wn;
                }
                u[degenerate_slots[filled]] = w.clone();
                basis.push(w);
                filled += 1;
            }
        }
        debug_assert_eq!(filled, degenerate_slots.len(), "R^m ran out of directions to complete U");
    }

    Svd { u, s, v, sweeps, converged }
}

/// Row-major `m x n` reconstruction `Σ_i s_i · u_i ⊗ v_i`, for the G-SVD1 gate.
pub fn reconstruct(svd: &Svd, m: usize, n: usize) -> Vec<f64> {
    let mut out = vec![0.0; m * n];
    for (idx, &sigma) in svd.s.iter().enumerate() {
        if sigma == 0.0 {
            continue;
        }
        let u = &svd.u[idx];
        let v = &svd.v[idx];
        for i in 0..m {
            let ui = sigma * u[i];
            for j in 0..n {
                out[i * n + j] += ui * v[j];
            }
        }
    }
    out
}
