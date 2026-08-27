//! DIAGNOSTIC ONLY — not a gate, not staked. Instruments the per-iteration cost of a
//! full-reorthogonalization Lanczos at N=12 (dim 853,776), using ONLY q-seam's PUBLIC
//! `Hubbard::apply` (its matvec). Written to answer: is the currently-running N=12 exact
//! reference (`full_grid_gates`/`g4_certificate`, 85+ minutes silent per team-lead's monitor)
//! bounded-slow, or pathological? Does not touch q-seam's source or the running jobs.

use q_seam::dense::jacobi;
use q_seam::hubbard::Hubbard;
use std::time::Instant;

fn main() {
    let t0 = Instant::now();
    let h = Hubbard::new(12, 1.0, 0.0);
    let dim = h.dim();
    println!("Hubbard::new(12,1.0,0.0): {:.3?}  dim={dim}  n_conf={}", t0.elapsed(), h.n_conf());
    println!("one Krylov vector = {:.2} MB; RSS should track basis_size * that", (dim * 8) as f64 / 1e6);

    // Isolated matvec timing (no reorthogonalization at all).
    let mut x = vec![0.0; dim];
    x[0] = 1.0;
    let mut y = vec![0.0; dim];
    let n_probe = 10;
    let t1 = Instant::now();
    for _ in 0..n_probe {
        h.apply(&x, &mut y);
        std::mem::swap(&mut x, &mut y);
    }
    let matvec_elapsed = t1.elapsed();
    println!(
        "{n_probe} isolated matvecs: {:.3?} total, {:.3?}/matvec",
        matvec_elapsed,
        matvec_elapsed / n_probe
    );

    // A minimal full-reorthogonalization Lanczos replica — same asymptotic cost SHAPE as
    // q-seam's own lanczos.rs (matvec + local orthogonalization + full reorth against every
    // stored vector), instrumented per-phase, capped low. NOT the pinned start seed, NOT meant
    // to converge — a cost probe only.
    let _cap = 30usize;
    let mut v0 = vec![0.0; dim];
    let mut state = 0x12345_6789_ABCDu64;
    for v in v0.iter_mut() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        *v = ((state >> 33) as f64 / (1u64 << 31) as f64) - 1.0;
    }
    let norm: f64 = v0.iter().map(|x| x * x).sum::<f64>().sqrt();
    for v in v0.iter_mut() {
        *v /= norm;
    }

    let mut basis: Vec<Vec<f64>> = vec![v0];
    let mut w = vec![0.0; dim];
    let mut alpha: Vec<f64> = vec![];
    let mut beta: Vec<f64> = vec![];

    println!("\n=== per-iteration cost, faithful replica INCLUDING the per-iteration dense tridiagonal");
    println!("    Ritz solve (q-seam's OWN pattern — lanczos.rs:117-126 rebuilds and re-diagonalizes");
    println!("    the full m x m tridiagonal via q_seam::dense::jacobi EVERY iteration, m growing to");
    println!("    the iteration count) ===");
    let cap2 = 200usize.min(dim);
    let probe_start = Instant::now();
    for j in 0..cap2 {
        h.apply(&basis[j], &mut w);

        let a: f64 = w.iter().zip(&basis[j]).map(|(p, q)| p * q).sum();
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
                let c: f64 = w.iter().zip(u.iter()).map(|(p, q)| p * q).sum();
                for k in 0..dim {
                    w[k] -= c * u[k];
                }
            }
        }

        let t_ritz = Instant::now();
        let m = alpha.len();
        let mut tri = vec![0.0; m * m];
        for i in 0..m {
            tri[i * m + i] = alpha[i];
            if i + 1 < m {
                tri[i * m + i + 1] = beta[i];
                tri[(i + 1) * m + i] = beta[i];
            }
        }
        let eig = jacobi(tri, m);
        let ritz_dt = t_ritz.elapsed();

        let bnorm: f64 = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        if j % 10 == 0 || j == cap2 - 1 {
            println!(
                "iter {j:3}: m={m:3} ritz_solve={:>9.3?} (jacobi sweeps={:3}, converged={})  cumulative={:>9.3?}",
                ritz_dt, eig.sweeps, eig.converged, probe_start.elapsed()
            );
        }

        if bnorm < 1e-13 {
            println!("breakdown at iter {j}");
            break;
        }
        beta.push(bnorm);
        for k in 0..dim {
            w[k] /= bnorm;
        }
        basis.push(w.clone());
    }
    let total = probe_start.elapsed();
    println!("\n{cap2}-iteration faithful-replica probe total: {total:.3?}");
    println!(
        "extrapolated to 400 iterations (q-seam's MAX_ITERS), assuming ritz cost dominates ~iters^4 \
         (cumulative dense diag of a growing matrix): {:.1?}",
        total * (400i64.pow(4) / (cap2 as i64).pow(4)) as u32
    );
    println!(
        "extrapolated to 400 iterations assuming ritz cost is closer to ~iters^3 cumulative (fewer \
         jacobi sweeps at scale): {:.1?}",
        total * (400i64.pow(3) / (cap2 as i64).pow(3)) as u32
    );
}
