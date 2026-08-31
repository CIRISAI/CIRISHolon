use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};
use std::time::Instant;

fn main() {
    println!("=== OOH 3-Body Electronic Structure Probe ===");
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    println!("E(O) = {:.12} Ha", e_o);
    println!("E(H) = {:.12} Ha", e_h);

    // Equilibrium-like HOO geometry:
    // r_OO ~ 2.4 bohr (~1.27 A)
    // r_OH ~ 1.85 bohr (~0.98 A)
    // theta_OOH ~ 105 deg
    let r_oo = 2.40;
    let r_oh = 1.85;
    let theta_deg = 105.0f64;
    let theta = theta_deg.to_radians();

    let species = vec![OXYGEN, OXYGEN, HYDROGEN];
    let centers = vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],                  // O1
        [D2::c(r_oo), D2::c(0.0), D2::c(0.0)],                 // O2
        [D2::c(r_oh * theta.cos()), D2::c(r_oh * theta.sin()), D2::c(0.0)], // H bonded to O1
    ];

    let t0 = Instant::now();
    let sol = solve_geometry(&species, centers);
    let elapsed = t0.elapsed();

    println!("Total E(OOH) = {:.12} Ha", sol.e.v);
    println!("N_basis = {}, N_det = {}", sol.n_basis, sol.n_det);
    println!("Davidson iters = {}, residual = {:.3e}", sol.davidson_iters, sol.residual);
    println!("Solve time = {:.2?}", elapsed);

    // Compute 2-body energies
    // r(O1, O2) = r_oo
    let v2_oo = pair_point(OXYGEN, OXYGEN, r_oo).e - 2.0 * e_o;
    // r(O1, H) = r_oh
    let v2_oh1 = pair_point(OXYGEN, HYDROGEN, r_oh).e - e_o - e_h;
    // r(O2, H) = sqrt((r_oo - r_oh*cos)^2 + (r_oh*sin)^2)
    let dx = r_oo - r_oh * theta.cos();
    let dy = -r_oh * theta.sin();
    let r_oh2 = (dx * dx + dy * dy).sqrt();
    let v2_oh2 = pair_point(OXYGEN, HYDROGEN, r_oh2).e - e_o - e_h;

    let v_tot_3 = sol.e.v - 2.0 * e_o - e_h;
    let de3 = v_tot_3 - (v2_oo + v2_oh1 + v2_oh2);

    println!("r_OO = {:.4} bohr, r_OH1 = {:.4} bohr, r_OH2 = {:.4} bohr", r_oo, r_oh, r_oh2);
    println!("V_tot(3) = {:.12} Ha", v_tot_3);
    println!("V2(OO)   = {:.12} Ha", v2_oo);
    println!("V2(OH1)  = {:.12} Ha", v2_oh1);
    println!("V2(OH2)  = {:.12} Ha", v2_oh2);
    println!("dE3(OOH) = {:.12} Ha", de3);
}
