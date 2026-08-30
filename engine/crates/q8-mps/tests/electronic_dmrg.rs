//! Verification of electronic Hamiltonian MPO mapping and DMRG eigensolver.

use q8_mps::dmrg::solve_electronic_ground_state;
use q8_mps::eigen::jacobi_eigen;
use q8_mps::mpo::{dense_from_mpo, Mpo};
use q8_mps::observables;

/// Independently construct the full 2^(2*n_orb) x 2^(2*n_orb) Hamiltonian matrix
/// directly from creation/annihilation fermionic operators in Jordan-Wigner representation.
fn independent_electronic_dense(n_orb: usize, h: &[f64], g: &[f64]) -> Vec<f64> {
    let l = 2 * n_orb;
    let dim = 1usize << l;
    let mut mat = vec![0.0f64; dim * dim];

    // Helper: apply c_i to state index, returning Option<(sign, new_index)>
    let apply_c = |idx: usize, i: usize| -> Option<(f64, usize)> {
        if (idx & (1 << i)) == 0 {
            return None;
        }
        let count = (idx & ((1 << i) - 1)).count_ones();
        let sign = if count % 2 == 0 { 1.0 } else { -1.0 };
        Some((sign, idx ^ (1 << i)))
    };

    // Helper: apply c_i^dagger to state index
    let apply_cd = |idx: usize, i: usize| -> Option<(f64, usize)> {
        if (idx & (1 << i)) != 0 {
            return None;
        }
        let count = (idx & ((1 << i) - 1)).count_ones();
        let sign = if count % 2 == 0 { 1.0 } else { -1.0 };
        Some((sign, idx | (1 << i)))
    };

    // 1-body terms: sum_{pq, sigma} h_{pq} c_{p,sigma}^dagger c_{q,sigma}
    for p in 0..n_orb {
        for q in 0..n_orb {
            let hpq = h[p * n_orb + q];
            if hpq == 0.0 {
                continue;
            }
            for sigma in 0..2 {
                let i = 2 * p + sigma;
                let j = 2 * q + sigma;
                for col in 0..dim {
                    if let Some((s1, idx1)) = apply_c(col, j) {
                        if let Some((s2, row)) = apply_cd(idx1, i) {
                            mat[row * dim + col] += hpq * s1 * s2;
                        }
                    }
                }
            }
        }
    }

    // 2-body terms: 1/2 sum_{pqrs, sigma tau} g_{pqrs} c_{p,sigma}^dagger c_{r,tau}^dagger c_{s,tau} c_{q,sigma}
    for p in 0..n_orb {
        for q in 0..n_orb {
            for r in 0..n_orb {
                for s in 0..n_orb {
                    let gpqrs = g[(p * n_orb + q) * n_orb * n_orb + (r * n_orb + s)];
                    if gpqrs == 0.0 {
                        continue;
                    }
                    let coeff = 0.5 * gpqrs;
                    for sigma in 0..2 {
                        for tau in 0..2 {
                            let i = 2 * p + sigma;
                            let j = 2 * q + sigma;
                            let k = 2 * r + tau;
                            let l_spin = 2 * s + tau;
                            if i == k || l_spin == j {
                                continue;
                            }
                            for col in 0..dim {
                                if let Some((s1, idx1)) = apply_c(col, j) {
                                    if let Some((s2, idx2)) = apply_c(idx1, l_spin) {
                                        if let Some((s3, idx3)) = apply_cd(idx2, k) {
                                            if let Some((s4, row)) = apply_cd(idx3, i) {
                                                mat[row * dim + col] +=
                                                    coeff * s1 * s2 * s3 * s4;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    mat
}

#[test]
fn test_mpo_electronic_vs_independent_dense() {
    let n_orb = 2;
    // 2-orbital model (e.g. H2-like)
    let h = vec![
        -1.2, 0.3,
        0.3, -0.8,
    ];
    let mut g = vec![0.0f64; 16];
    // On-site Coulomb
    g[(0 * 2 + 0) * 4 + (0 * 2 + 0)] = 0.7;
    g[(1 * 2 + 1) * 4 + (1 * 2 + 1)] = 0.6;
    // Inter-orbital Coulomb & exchange
    g[(0 * 2 + 0) * 4 + (1 * 2 + 1)] = 0.4;
    g[(1 * 2 + 1) * 4 + (0 * 2 + 0)] = 0.4;
    g[(0 * 2 + 1) * 4 + (1 * 2 + 0)] = 0.15;
    g[(1 * 2 + 0) * 4 + (0 * 2 + 1)] = 0.15;

    let mpo = Mpo::from_electronic_integrals(n_orb, &h, &g);
    let mpo_dense = mpo.dense();
    let exact_dense = independent_electronic_dense(n_orb, &h, &g);

    assert_eq!(mpo_dense.len(), exact_dense.len());
    let mut max_diff = 0.0f64;
    for (a, b) in mpo_dense.iter().zip(exact_dense.iter()) {
        max_diff = max_diff.max((a - b).abs());
    }
    assert!(
        max_diff < 1e-13,
        "MPO dense matrix differs from independent build by {max_diff:e}"
    );
}

#[test]
fn test_dmrg_electronic_ground_state_vs_exact() {
    let n_orb = 3;
    let n_alpha = 1;
    let n_beta = 1;

    let mut h = vec![0.0f64; n_orb * n_orb];
    h[0 * 3 + 0] = -1.5;
    h[1 * 3 + 1] = -0.8;
    h[2 * 3 + 2] = -0.2;
    h[0 * 3 + 1] = -0.4;
    h[1 * 3 + 0] = -0.4;
    h[1 * 3 + 2] = -0.3;
    h[2 * 3 + 1] = -0.3;

    let mut g = vec![0.0f64; n_orb * n_orb * n_orb * n_orb];
    for p in 0..n_orb {
        g[(p * n_orb + p) * n_orb * n_orb + (p * n_orb + p)] = 0.6 + 0.1 * p as f64;
        for q in (p + 1)..n_orb {
            g[(p * n_orb + p) * n_orb * n_orb + (q * n_orb + q)] = 0.3;
            g[(q * n_orb + q) * n_orb * n_orb + (p * n_orb + p)] = 0.3;
            g[(p * n_orb + q) * n_orb * n_orb + (q * n_orb + p)] = 0.1;
            g[(q * n_orb + p) * n_orb * n_orb + (p * n_orb + q)] = 0.1;
        }
    }

    // Exact diagonalization in target sector (n_alpha=1, n_beta=1)
    let exact_mat = independent_electronic_dense(n_orb, &h, &g);
    let dim = 1usize << (2 * n_orb);
    let eig = jacobi_eigen(exact_mat, dim);
    assert!(eig.converged);

    // Find lowest eigenvalue in the correct particle/spin sector
    let mut sector_min_e = f64::INFINITY;
    for (val, vec) in eig.values.iter().zip(eig.vectors.iter()) {
        let mut na = 0.0;
        let mut nb = 0.0;
        for (col, &coeff) in vec.iter().enumerate() {
            let p = coeff * coeff;
            let mut ca = 0;
            let mut cb = 0;
            for j in 0..(2 * n_orb) {
                if (col & (1 << j)) != 0 {
                    if j % 2 == 0 {
                        ca += 1;
                    } else {
                        cb += 1;
                    }
                }
            }
            na += p * ca as f64;
            nb += p * cb as f64;
        }
        if (na - n_alpha as f64).abs() < 1e-6 && (nb - n_beta as f64).abs() < 1e-6 {
            sector_min_e = sector_min_e.min(*val);
        }
    }

    // DMRG solve
    let dmrg_res = solve_electronic_ground_state(
        n_orb, n_alpha, n_beta, &h, &g, 32, 20, 1e-9,
    )
    .expect("DMRG solve failed");

    assert!(dmrg_res.converged);
    let e_diff = (dmrg_res.energy - sector_min_e).abs();
    assert!(
        e_diff < 1e-7,
        "DMRG energy {} differs from exact sector ground state {} by {e_diff:e}",
        dmrg_res.energy,
        sector_min_e
    );

    // Verify particle number conservation in MPS
    let n_tot = observables::total_number(&dmrg_res.tensors);
    assert!(
        (n_tot - (n_alpha + n_beta) as f64).abs() < 1e-8,
        "MPS total electron count {n_tot} != expected {}",
        n_alpha + n_beta
    );
}

#[test]
fn test_mpo_hubbard_constructor_equivalence() {
    let sites = 2;
    let t = 1.0;
    let u = 4.0;
    let mu = 2.0;

    let mpo = Mpo::from_hubbard(sites, t, u, mu);
    let mpo_dense = mpo.dense();
    let legacy_dense = dense_from_mpo(sites, t, u, mu);

    assert_eq!(mpo_dense.len(), legacy_dense.len());
    let mut max_diff = 0.0f64;
    for (a, b) in mpo_dense.iter().zip(legacy_dense.iter()) {
        max_diff = max_diff.max((a - b).abs());
    }
    assert!(
        max_diff < 1e-12,
        "Hubbard MPO dense matrix differs from legacy by {max_diff:e}"
    );
}
