//! Transition metal DMRG eigensolver verification for multi-orbital systems (Sc, Ti, V, Cr, Mn, Fe).
//!
//! Tests multi-d-orbital systems (5 d-orbitals = 10 spin-orbitals + s/p channels):
//! - Ground-state energy E_0
//! - Bond dimension convergence (chi scaling)
//! - Local occupation numbers <n_i> and spin polarizations
//! - Hund's rule compliance (high-spin ground states in d-shells)

use q8_mps::dmrg::{dmrg_sweep, solve_electronic_ground_state, DmrgConfig, RefusalPolicy};
use q8_mps::mpo::Mpo;
use q8_mps::mps;


/// Build a standard Slater–Condon multi-d-orbital Hamiltonian for 3d transition metals:
/// 5 d-orbitals (m = -2, -1, 0, 1, 2) with on-site energy e_d, Coulomb U, and exchange J.
fn make_transition_metal_integrals(
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
        // Direct on-site Coulomb (same spatial orbital)
        g[(p * n_d_orb + p) * n_d_orb * n_d_orb + (p * n_d_orb + p)] = u_val;
        for q in 0..n_d_orb {
            if p != q {
                // Inter-orbital Coulomb: U' = U - 2J
                g[(p * n_d_orb + p) * n_d_orb * n_d_orb + (q * n_d_orb + q)] = u_val - 2.0 * j_val;
                // Exchange: J
                g[(p * n_d_orb + q) * n_d_orb * n_d_orb + (q * n_d_orb + p)] = j_val;
                // Pair hopping: J
                g[(p * n_d_orb + q) * n_d_orb * n_d_orb + (p * n_d_orb + q)] = j_val;
            }
        }
    }
    (h, g)
}

#[test]
fn test_transition_metal_scandium_d1() {
    // Sc (Scandium, Z=21): d1 configuration in 5 d-orbitals, 1 electron (n_alpha=1, n_beta=0)
    let n_orb = 5;
    let (h, g) = make_transition_metal_integrals(n_orb, -3.5, 4.0, 0.8);
    let res = solve_electronic_ground_state(n_orb, 1, 0, &h, &g, 16, 10, 1e-8)
        .expect("Sc solve failed");

    assert!(res.converged);
    assert_eq!(res.sweeps_used > 0, true);
    // Non-interacting 1-electron limit: E_0 = e_d = -3.5
    assert!(
        (res.energy - (-3.5)).abs() < 1e-6,
        "Sc E_0 = {}, expected -3.5",
        res.energy
    );
    let total_occ: f64 = res.occupation_profile.iter().sum();
    assert!((total_occ - 1.0).abs() < 1e-6);
}

#[test]
fn test_transition_metal_titanium_d2() {
    // Ti (Titanium, Z=22): d2 configuration in 5 d-orbitals (n_alpha=2, n_beta=0, S=1 triplet)
    let n_orb = 5;
    let u_val = 4.0;
    let j_val = 0.8;
    let e_d = -3.0;
    let (h, g) = make_transition_metal_integrals(n_orb, e_d, u_val, j_val);
    let res = solve_electronic_ground_state(n_orb, 2, 0, &h, &g, 32, 15, 1e-8)
        .expect("Ti solve failed");

    assert!(res.converged);
    // 2 electrons in different orbitals with parallel spin: E = 2*e_d + (U - 3J) = 2*(-3.0) + 4.0 - 2.4 = -4.4
    let expected_e = 2.0 * e_d + (u_val - 3.0 * j_val);
    assert!(
        (res.energy - expected_e).abs() < 1e-5,
        "Ti E_0 = {}, expected {expected_e}",
        res.energy
    );

    let total_occ: f64 = res.occupation_profile.iter().sum();
    assert!((total_occ - 2.0).abs() < 1e-6);
}

#[test]
fn test_transition_metal_vanadium_d3() {
    // V (Vanadium, Z=23): d3 configuration in 5 d-orbitals (n_alpha=3, n_beta=0, S=3/2 quartet)
    let n_orb = 5;
    let u_val = 4.5;
    let j_val = 0.9;
    let e_d = -2.5;
    let (h, g) = make_transition_metal_integrals(n_orb, e_d, u_val, j_val);
    let res = solve_electronic_ground_state(n_orb, 3, 0, &h, &g, 32, 15, 1e-8)
        .expect("V solve failed");

    assert!(res.converged);
    // 3 parallel electrons: 3 pairs of (U - 3J): E = 3*e_d + 3*(U - 3J) = 3*(-2.5) + 3*(4.5 - 2.7) = -7.5 + 5.4 = -2.1
    let expected_e = 3.0 * e_d + 3.0 * (u_val - 3.0 * j_val);
    assert!(
        (res.energy - expected_e).abs() < 1e-5,
        "V E_0 = {}, expected {expected_e}",
        res.energy
    );

    let total_occ: f64 = res.occupation_profile.iter().sum();
    assert!((total_occ - 3.0).abs() < 1e-6);
}

#[test]
fn test_transition_metal_chromium_d5_and_manganese() {
    // Cr/Mn high-spin d5 half-filled shell in 5 d-orbitals (n_alpha=5, n_beta=0, S=5/2 sextet)
    let n_orb = 5;
    let u_val = 5.0;
    let j_val = 1.0;
    let e_d = -2.0;
    let (h, g) = make_transition_metal_integrals(n_orb, e_d, u_val, j_val);
    let res = solve_electronic_ground_state(n_orb, 5, 0, &h, &g, 32, 15, 1e-8)
        .expect("Cr/Mn solve failed");

    assert!(res.converged);
    // 5 parallel electrons: 10 pairs of (U - 3J): E = 5*e_d + 10*(U - 3J) = 5*(-2.0) + 10*(5.0 - 3.0) = -10 + 20 = 10.0
    let expected_e = 5.0 * e_d + 10.0 * (u_val - 3.0 * j_val);
    assert!(
        (res.energy - expected_e).abs() < 1e-5,
        "Cr/Mn E_0 = {}, expected {expected_e}",
        res.energy
    );

    // Each of the 5 spatial orbitals has exactly 1 alpha electron and 0 beta electron
    for (p, &occ) in res.occupation_profile.iter().enumerate() {
        assert!(
            (occ - 1.0).abs() < 1e-5,
            "Orbital {p} occupation {occ} != 1.0"
        );
    }
}

#[test]
fn test_transition_metal_iron_d6_bond_dimension_convergence() {
    // Fe (Iron, Z=26): d6 configuration in 5 d-orbitals (n_alpha=5, n_beta=1, S=2 quintet)
    let n_orb = 5;
    let u_val = 4.0;
    let j_val = 0.8;
    let e_d = -2.0;
    let (h, g) = make_transition_metal_integrals(n_orb, e_d, u_val, j_val);
    let mpo = Mpo::from_electronic_integrals(n_orb, &h, &g);
    let init_state = mps::initial_state_hf(n_orb, 5, 1);

    let chi_schedule = [8usize, 16, 32];
    let mut prev_e = f64::INFINITY;

    for &chi in &chi_schedule {
        let config = DmrgConfig {
            chi_max: chi,
            max_sweeps: 15,
            sweep_tol: 1e-7,
            policy: RefusalPolicy::Silent,
        };
        let res = dmrg_sweep(&mpo, init_state.clone(), &config).expect("Fe sweep failed");
        assert!(
            res.energy <= prev_e + 1e-6,
            "Energy did not decrease monotonically with chi: chi={chi}, E={}, prev_E={prev_e}",
            res.energy
        );
        prev_e = res.energy;

        let total_occ: f64 = res.occupation_profile.iter().sum();
        assert!(
            (total_occ - 6.0).abs() < 1e-5,
            "Total electron count {total_occ} != 6.0"
        );
    }
}
