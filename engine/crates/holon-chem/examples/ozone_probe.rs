use holon_chem::dual::D2;
use holon_chem::elements::OXYGEN;
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};
use std::time::Instant;

fn main() {
    println!("=== Ozone (O3) 3-Body Electronic Structure Probe ===");
    let e_o = atom_energy(OXYGEN);
    println!("E(O) = {:.12} Ha", e_o);

    // Open C2v ozone ground-state geometry:
    // s1 = s2 ~ 2.41 bohr (~1.278 A)
    // theta ~ 116.8 deg
    let s1 = 2.41;
    let s2 = 2.41;
    let theta_deg = 116.8f64;
    let theta = theta_deg.to_radians();

    let species = vec![OXYGEN, OXYGEN, OXYGEN];
    let centers = vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
        [D2::c(s1), D2::c(0.0), D2::c(0.0)],
        [D2::c(s2 * theta.cos()), D2::c(s2 * theta.sin()), D2::c(0.0)],
    ];

    let t0 = Instant::now();
    let sol = solve_geometry(&species, centers);
    let elapsed = t0.elapsed();

    println!("Total E(O3) = {:.12} Ha", sol.e.v);
    println!("N_basis = {}, N_det = {}", sol.n_basis, sol.n_det);
    println!("Davidson iters = {}, residual = {:.3e}", sol.davidson_iters, sol.residual);
    println!("Solve time = {:.2?}", elapsed);

    let s3 = (s1 * s1 + s2 * s2 - 2.0 * s1 * s2 * theta.cos()).sqrt();
    let v2_s1 = pair_point(OXYGEN, OXYGEN, s1).e - 2.0 * e_o;
    let v2_s2 = pair_point(OXYGEN, OXYGEN, s2).e - 2.0 * e_o;
    let v2_s3 = pair_point(OXYGEN, OXYGEN, s3).e - 2.0 * e_o;

    let v_tot_3 = sol.e.v - 3.0 * e_o;
    let de3 = v_tot_3 - (v2_s1 + v2_s2 + v2_s3);

    println!("s1 = {:.4} bohr, s2 = {:.4} bohr, s3 = {:.4} bohr, theta = {:.1}°", s1, s2, s3, theta_deg);
    println!("V_tot(3) = {:.12} Ha", v_tot_3);
    println!("V2(s1)   = {:.12} Ha", v2_s1);
    println!("V2(s2)   = {:.12} Ha", v2_s2);
    println!("V2(s3)   = {:.12} Ha", v2_s3);
    println!("dE3(O3)  = {:.12} Ha", de3);
}
