use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::ooh::{de3_point, OohMeta, OohTable, node_index, NR, NU, N_NODES};
use holon_chem::pair::{atom_energy, pair_point};

#[test]
fn ooh_exchange_symmetry_is_bit_exact() {
    let mut table = OohTable::empty();
    table.begin();

    // Fill table with symmetric mock values or sampled points
    for i in 0..NR {
        for j in 0..NR {
            for k in 0..NU {
                let val = (i as f64 * j as f64).sin() * (k as f64 + 1.0).sqrt();
                table.knot(node_index(i, j, k), val);
            }
        }
    }
    table.finish(OohMeta::empty());

    let (r1, r2, r3) = (1.85, 3.40, 2.40);
    let (v_orig, g_orig) = table.eval(r1, r2, r3);
    let (v_swap, g_swap) = table.eval(r2, r1, r3);

    // Value must be bit-for-bit identical
    assert_eq!(v_orig.to_bits(), v_swap.to_bits());
    // Gradients for r1 and r2 must swap, r3 gradient must be bit-identical
    assert_eq!(g_orig[0].to_bits(), g_swap[1].to_bits());
    assert_eq!(g_orig[1].to_bits(), g_swap[0].to_bits());
    assert_eq!(g_orig[2].to_bits(), g_swap[2].to_bits());
}

#[test]
fn ooh_fci_point_is_saturating_and_bound() {
    // Equilibrium OOH geometry: r_OO = 2.40, r_OH1 = 1.85, theta = 105 deg
    let (roh1, roo) = (1.85, 2.40);
    let theta_rad = 105.0f64.to_radians();
    let dx = roo - roh1 * theta_rad.cos();
    let dy = -roh1 * theta_rad.sin();
    let roh2 = (dx * dx + dy * dy).sqrt();

    let de3 = de3_point(roh1, roh2, roo);
    // Many-body 3-body term must be repulsive (valence saturation > 0)
    assert!(de3 > 0.0, "dE3(OOH) must be repulsive, got {:.6}", de3);
    assert!(de3 < 0.25, "dE3(OOH) must be bounded, got {:.6}", de3);
}

#[test]
fn ooh_gradient_tracks_finite_difference() {
    let mut table = OohTable::empty();
    table.begin();
    for i in 0..NR {
        for j in 0..NR {
            for k in 0..NU {
                let tau_i = i as f64 / (NR - 1) as f64;
                let tau_j = j as f64 / (NR - 1) as f64;
                let c_k = k as f64 / (NU - 1) as f64;
                let val = (tau_i + 0.5).ln() * (tau_j + 0.5).ln() * (c_k + 0.1).sin();
                table.knot(node_index(i, j, k), val);
            }
        }
    }
    table.finish(OohMeta::empty());

    let (r1, r2, r3) = (2.0, 3.0, 2.5);
    let (v0, grad) = table.eval(r1, r2, r3);

    let eps = 1e-5;
    let (v_dr1, _) = table.eval(r1 + eps, r2, r3);
    let fd_dr1 = (v_dr1 - v0) / eps;
    assert!((grad[0] - fd_dr1).abs() < 1e-4, "grad[0] = {}, fd = {}", grad[0], fd_dr1);

    let (v_dr2, _) = table.eval(r1, r2 + eps, r3);
    let fd_dr2 = (v_dr2 - v0) / eps;
    assert!((grad[1] - fd_dr2).abs() < 1e-4, "grad[1] = {}, fd = {}", grad[1], fd_dr2);

    let (v_dr3, _) = table.eval(r1, r2, r3 + eps);
    let fd_dr3 = (v_dr3 - v0) / eps;
    assert!((grad[2] - fd_dr3).abs() < 1e-4, "grad[2] = {}, fd = {}", grad[2], fd_dr3);
}
