//! Benchmark: Transition Metal DMRG Ground States (Sc, Ti, V, Cr, Mn, Fe).
//!
//! Multi-d-orbital models in the Slater-Condon formulation.
//! Demonstrates:
//! - Ground-state energy E_0
//! - Bond dimension convergence (chi = 8, 16, 32, 64)
//! - Local occupation numbers <n_p> and spin polarizations <m_p>
//! - Discarded weight convergence

use q8_mps::dmrg::{dmrg_sweep, DmrgConfig, RefusalPolicy};
use q8_mps::mpo::Mpo;
use q8_mps::mps;
use q8_mps::observables;

fn make_tm_integrals(
    n_d_orb: usize,
    e_d: f64,
    u_val: f64,
    j_val: f64,
) -> (Vec<f64>, Vec<f64>) {
    let mut h = vec![0.0f64; n_d_orb * n_d_orb];
    for p in 0..n_d_orb {
        h[p * n_d_orb + p] = e_d;
    }

    let mut g = vec![0.0f64; n_d_orb * n_d_orb * n_d_orb * n_d_orb];
    for p in 0..n_d_orb {
        g[(p * n_d_orb + p) * n_d_orb * n_d_orb + (p * n_d_orb + p)] = u_val;
        for q in 0..n_d_orb {
            if p != q {
                g[(p * n_d_orb + p) * n_d_orb * n_d_orb + (q * n_d_orb + q)] = u_val - 2.0 * j_val;
                g[(p * n_d_orb + q) * n_d_orb * n_d_orb + (q * n_d_orb + p)] = j_val;
                g[(p * n_d_orb + q) * n_d_orb * n_d_orb + (p * n_d_orb + q)] = j_val;
            }
        }
    }
    (h, g)
}

fn main() {
    println!("================================================================================");
    println!("       TRANSITION METAL DMRG BENCHMARK (Sc, Ti, V, Cr, Mn, Fe)                 ");
    println!("================================================================================");

    let cases = [
        ("Scandium (Sc)", 21, "3d1", 1, 0, -3.5, 4.0, 0.8),
        ("Titanium (Ti)", 22, "3d2", 2, 0, -3.0, 4.0, 0.8),
        ("Vanadium (V)",  23, "3d3", 3, 0, -2.5, 4.5, 0.9),
        ("Chromium (Cr)", 24, "3d5", 5, 0, -2.0, 5.0, 1.0),
        ("Manganese (Mn)",25, "3d5", 5, 0, -2.0, 5.0, 1.0),
        ("Iron (Fe)",     26, "3d6", 5, 1, -2.0, 4.0, 0.8),
    ];

    let n_orb = 5; // 5 d-orbitals = 10 spin-orbitals

    for &(name, z, conf, na, nb, e_d, u_val, j_val) in &cases {
        println!("\n--- {} (Z={}, {}) ---", name, z, conf);
        println!("  Electrons: N_alpha={}, N_beta={}, Total={}", na, nb, na + nb);
        println!("  Parameters: e_d={:.2}, U={:.2}, J={:.2}", e_d, u_val, j_val);

        let (h, g) = make_tm_integrals(n_orb, e_d, u_val, j_val);
        let mpo = Mpo::from_electronic_integrals(n_orb, &h, &g);
        let init_state = mps::initial_state_hf(n_orb, na, nb);

        println!("  MPO bond dimensions: {:?}", mpo.bond_dims());
        println!("  {:>6} | {:>14} | {:>14} | {:>10}", "chi", "Energy (E_0)", "Max Discarded", "Sweeps");
        println!("  {:-<6}-+-{:-<14}-+-{:-<14}-+-{:-<10}", "", "", "", "");

        for &chi in &[8usize, 16, 32, 64] {
            let config = DmrgConfig {
                chi_max: chi,
                max_sweeps: 15,
                sweep_tol: 1e-8,
                policy: RefusalPolicy::Silent,
            };
            let res = dmrg_sweep(&mpo, init_state.clone(), &config).expect("DMRG sweep failed");
            let max_dw = res.discarded_weight.iter().copied().fold(0.0f64, f64::max);
            println!("  {:>6} | {:>14.8} | {:>14.4e} | {:>10}", chi, res.energy, max_dw, res.sweeps_used);

            if chi == 64 {
                let occ = observables::occupation_profile(&res.tensors, n_orb);
                let pol = observables::spin_polarization_profile(&res.tensors, n_orb);
                println!("  Final Local Spatial Occupations <n_p>: {:?}", occ);
                println!("  Final Local Spin Polarization <m_p>: {:?}", pol);
            }
        }
    }
    println!("\n================================================================================");
    println!("Benchmark completed successfully.");
}
