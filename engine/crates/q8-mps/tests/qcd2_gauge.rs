//! GF2a gate G0's engine half: the accumulator MPO is the same operator as the
//! determinant solver's integral tensor — dense spectrum against the FCI referee at N = 4
//! (numbers from `holon-chem`'s `examples/qcd2` on 2026-09-02, plant (i) passed there), and
//! the sweep reaching them.

use q8_mps::qcd2::Qcd2;

/// FCI referees at x = 4, N = 4: E0(B=0) with 6 quarks, E0(B=1) with 9 quarks.
const REFEREE: [(i32, usize, f64); 2] = [(0, 6, -24.5391166860), (1, 9, -17.5847761876)];

/// Lowest eigenvalue of the dense MPO restricted to `n_q` occupied modes, by power
/// iteration on the shifted operator (dim 924 and 220 here).
fn lowest_in_sector(h: &[f64], dim_full: usize, n_q: usize, diagonal_shift: f64) -> f64 {
    let states: Vec<usize> = (0..dim_full).filter(|b| b.count_ones() as usize == n_q).collect();
    let dim = states.len();
    let mut sub = vec![0.0; dim * dim];
    for (i, &bi) in states.iter().enumerate() {
        for (j, &bj) in states.iter().enumerate() {
            sub[i * dim + j] = h[bi * dim_full + bj] + if i == j { diagonal_shift } else { 0.0 };
        }
    }
    // symmetry check: the MPO must be Hermitian to roundoff
    let mut asym = 0.0f64;
    for i in 0..dim {
        for j in 0..dim {
            asym = asym.max((sub[i * dim + j] - sub[j * dim + i]).abs());
        }
    }
    assert!(asym <= 1e-12, "the dense MPO is not symmetric: {asym:e}");
    let shift = sub.iter().map(|v| v.abs()).sum::<f64>() / dim as f64 * 4.0 + 1.0;
    let mut v = vec![1.0 / (dim as f64).sqrt(); dim];
    let mut lam = 0.0;
    for _ in 0..50000 {
        let mut w = vec![0.0; dim];
        for i in 0..dim {
            let mut acc = 0.0;
            for j in 0..dim {
                acc += (if i == j { shift } else { 0.0 } - sub[i * dim + j]) * v[j];
            }
            w[i] = acc;
        }
        let norm = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        for x in w.iter_mut() {
            *x /= norm;
        }
        let new_lam = shift - norm;
        let delta = (new_lam - lam).abs();
        lam = new_lam;
        v = w;
        if delta < 1e-13 {
            break;
        }
    }
    lam
}

#[test]
fn the_accumulator_mpo_is_the_fci_tensor_at_four_sites() {
    let q = Qcd2::new(4, 4.0);
    let dim = 1usize << q.sites();
    for (b, n_q, e_ref) in REFEREE {
        let dense = q.mpo(n_q).dense();
        // the MPO carries the penalty without its constant; inside the sector it is exactly
        // −λ n_q² on the diagonal, put back BEFORE the power iteration (a constant of that size
        // would flatten its convergence rate to nothing)
        let e = lowest_in_sector(&dense, dim, n_q, q.lam * (n_q as f64) * (n_q as f64));
        assert!((e - e_ref).abs() <= 1e-8, "B={b}: dense MPO {e:.10} vs FCI {e_ref:.10}");
    }
}

#[test]
fn the_sweep_reaches_the_fci_ground_state_at_four_sites() {
    let q = Qcd2::new(4, 4.0);
    for (b, _, e_ref) in REFEREE {
        let res = q.ground_energy(b, 64, 120, 1e-12).expect("sweep");
        assert!((res.energy - e_ref).abs() <= 1e-8, "B={b}: engine {:.10} vs FCI {e_ref:.10}", res.energy);
        // the local residual is absolute in an operator whose norm carries the penalty scale
        assert!(res.worst_lanczos_residual <= 1e-9 * (1.0 + q.lam * 81.0), "residual {:e}", res.worst_lanczos_residual);
    }
}
