//! SATURATION-2 feasibility probe: what an (O, H, H) table costs, and what it holds.
//!
//! Reads, before any gate is written, the three numbers the campaign's design depends on:
//! the determinant count and wall-clock cost of one water point through the general
//! N-centre route, the size of `dE3(O, H, H)` at chemically interesting geometries, and
//! how fast it dies as the two O-H sides are stretched. The prereg's grid and its
//! truncation radius are chosen from these; nothing here is a gate.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_probe
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};
use std::time::Instant;

fn c3(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

/// Total energy of one (O, H, H) geometry: oxygen at the origin, the two hydrogens at
/// `r1` and `r2` with the angle `theta` (radians) between them at oxygen.
fn water_energy(r1: f64, r2: f64, theta: f64) -> f64 {
    solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![
            c3(0.0, 0.0, 0.0),
            c3(r1, 0.0, 0.0),
            c3(r2 * theta.cos(), r2 * theta.sin(), 0.0),
        ],
    )
    .e
    .v
}

fn main() {
    let t0 = Instant::now();
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    println!("# atoms: E(O) = {e_o:.12} Ha, E(H) = {e_h:.12} Ha  [{:?}]", t0.elapsed());

    let probe = solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![c3(0.0, 0.0, 0.0), c3(1.8, 0.0, 0.0), c3(-0.45, 1.74, 0.0)],
    );
    println!(
        "# one water point: n_basis = {}, n_det = {}, route = {}, residual = {:.2e}",
        probe.n_basis,
        probe.n_det,
        probe.route.label(),
        probe.residual
    );

    let t = Instant::now();
    const REPS: usize = 20;
    for i in 0..REPS {
        let _ = water_energy(1.8 + 0.001 * i as f64, 1.8, 1.82);
    }
    let per_water = t.elapsed().as_secs_f64() / REPS as f64;
    println!("# water solve: {:.1} ms each", per_water * 1e3);

    let t = Instant::now();
    for i in 0..REPS {
        let _ = pair_point(OXYGEN, HYDROGEN, 1.8 + 0.001 * i as f64).e;
    }
    let per_oh = t.elapsed().as_secs_f64() / REPS as f64;
    println!("# O-H pair solve: {:.1} ms each", per_oh * 1e3);

    let t = Instant::now();
    for i in 0..REPS {
        let _ = pair_point(HYDROGEN, HYDROGEN, 1.4 + 0.001 * i as f64).e;
    }
    println!(
        "# H-H pair solve (general route): {:.2} ms each",
        t.elapsed().as_secs_f64() / REPS as f64 * 1e3
    );
    let t = Instant::now();
    for i in 0..REPS {
        let _ = holon_chem::trimer::pair_energy(1.4 + 0.001 * i as f64);
    }
    println!(
        "# H-H pair solve (s-only fast path): {:.4} ms each",
        t.elapsed().as_secs_f64() / REPS as f64 * 1e3
    );

    // dE3 on a coarse map. The pair energies are the same model's, through the same
    // general route, so the differences below are the three-body term and nothing else.
    let de3 = |r1: f64, r2: f64, theta: f64| -> f64 {
        let z = (r1 * r1 + r2 * r2 - 2.0 * r1 * r2 * theta.cos()).sqrt();
        water_energy(r1, r2, theta) + e_o + 2.0 * e_h
            - pair_point(OXYGEN, HYDROGEN, r1).e
            - pair_point(OXYGEN, HYDROGEN, r2).e
            - pair_point(HYDROGEN, HYDROGEN, z).e
    };

    println!("\n# dE3(O,H,H), hartree — the angle sweep at r1 = r2 = 1.81 bohr");
    for deg in [30.0, 60.0, 90.0, 104.5, 120.0, 150.0, 180.0] {
        let th = deg * std::f64::consts::PI / 180.0;
        println!("  theta = {deg:6.1} deg   dE3 = {:+.6e}", de3(1.81, 1.81, th));
    }

    println!("\n# dE3 tail: one O-H side stretched, the other at 1.81, theta = 104.5 deg");
    let th = 104.5 * std::f64::consts::PI / 180.0;
    for r2 in [2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0] {
        println!("  r2 = {r2:4.1}   dE3 = {:+.6e}", de3(1.81, r2, th));
    }

    println!("\n# dE3 tail: BOTH O-H sides stretched together, theta = 104.5 deg");
    for r in [2.5, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0] {
        println!("  r1 = r2 = {r:4.1}   dE3 = {:+.6e}", de3(r, r, th));
    }

    println!("\n# dE3 at the compact corner (both sides short, angle closed)");
    for r in [0.7, 0.9, 1.2, 1.5] {
        for deg in [40.0, 90.0, 180.0] {
            let th = deg * std::f64::consts::PI / 180.0;
            println!("  r = {r:4.1}  theta = {deg:5.1}   dE3 = {:+.6e}", de3(r, r, th));
        }
    }
}
