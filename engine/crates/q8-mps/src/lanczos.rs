//! Matrix-free Lanczos for the two-site effective Hamiltonian (`mps::apply_effective_h`).
//!
//! Same shape as `q-seam`'s `lanczos.rs` (full reorthogonalization, a tridiagonal Ritz problem
//! solved by a small dense eigensolver, a true-residual check before accepting) but the start
//! vector is the CURRENT two-site tensor, never random — the commission's "fixed initial state,
//! no RNG" applies to every local solve, not just the chain's initial product state, and a
//! warm start from the state the outer sweep already has is also the standard, faster choice.

use crate::eigen::jacobi_eigen;

pub const MAX_ITERS: usize = 80;
pub const RESIDUAL_GATE: f64 = 1e-10;

pub struct TwoSiteGroundState {
    pub energy: f64,
    pub vector: Vec<f64>,
    pub residual: f64,
    pub iterations: usize,
}

#[inline]
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// `apply` is `mps::apply_effective_h` partially applied over `(left,w1,w2,right,chi_l,chi_r)`;
/// passed as a closure so this module stays independent of `mps.rs`'s tensor layout.
pub fn ground_state<F: Fn(&[f64]) -> Vec<f64>>(
    apply: F,
    seed: &[f64],
    dim: usize,
) -> Option<TwoSiteGroundState> {
    let seed_norm = dot(seed, seed).sqrt();
    assert!(seed_norm > 0.0, "Lanczos seed vector is exactly zero");
    let v0: Vec<f64> = seed.iter().map(|x| x / seed_norm).collect();

    let max_iters = MAX_ITERS.min(dim);
    let mut basis: Vec<Vec<f64>> = Vec::with_capacity(max_iters);
    let mut alpha: Vec<f64> = Vec::new();
    let mut beta: Vec<f64> = Vec::new();

    basis.push(v0);
    let mut iterations = 0;
    let mut best: Option<(f64, Vec<f64>)> = None;

    for j in 0..max_iters {
        iterations = j + 1;
        let mut w = apply(&basis[j]);

        let a = dot(&w, &basis[j]);
        alpha.push(a);
        for k in 0..dim {
            w[k] -= a * basis[j][k];
        }
        if j > 0 {
            let b = beta[j - 1];
            for k in 0..dim {
                w[k] -= b * basis[j - 1][k];
            }
        }
        for _ in 0..2 {
            for u in basis.iter() {
                let c = dot(&w, u);
                for k in 0..dim {
                    w[k] -= c * u[k];
                }
            }
        }

        let b = dot(&w, &w).sqrt();

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
        let e0 = eig.values[0];

        let est = b * eig.vectors[0][m - 1].abs();
        let breakdown = b <= 1e-13;

        if est <= 0.1 * RESIDUAL_GATE * e0.abs().max(1.0) || breakdown || j == max_iters - 1 {
            let mut ritz = vec![0.0; dim];
            for (k, uk) in basis.iter().enumerate() {
                let c = eig.vectors[0][k];
                for i in 0..dim {
                    ritz[i] += c * uk[i];
                }
            }
            let norm = dot(&ritz, &ritz).sqrt();
            for x in ritz.iter_mut() {
                *x /= norm;
            }
            best = Some((e0, ritz));
            break;
        }

        beta.push(b);
        for k in 0..dim {
            w[k] /= b;
        }
        basis.push(w);
    }

    let (energy, vector) = best?;
    let hv = apply(&vector);
    let resid = hv
        .iter()
        .zip(vector.iter())
        .map(|(a, b)| (a - energy * b) * (a - energy * b))
        .sum::<f64>()
        .sqrt()
        / energy.abs().max(1.0);

    Some(TwoSiteGroundState { energy, vector, residual: resid, iterations })
}
