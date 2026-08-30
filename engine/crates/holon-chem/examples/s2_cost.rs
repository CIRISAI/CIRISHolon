//! Where the 33 ms of one water point actually goes.
//!
//! The table needs VALUES only — the interpolant supplies every derivative the dynamics
//! reads, which is the same design decision `trimer.rs` records — so any time spent
//! propagating second-order dual numbers through the integrals is time the table build
//! does not need to pay. This splits the cost so the design choice is made on a
//! measurement rather than on a guess.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_cost
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::fci::{solve, FciSpace};
use holon_chem::pair::{electron_counts, geometry_problem, solve_geometry};
use std::time::Instant;

fn centers(r1: f64, r2: f64, theta: f64) -> Vec<[D2; 3]> {
    vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
        [D2::c(r1), D2::c(0.0), D2::c(0.0)],
        [
            D2::c(r2 * theta.cos()),
            D2::c(r2 * theta.sin()),
            D2::c(0.0),
        ],
    ]
}

fn main() {
    const REPS: usize = 20;
    let sp = [OXYGEN, HYDROGEN, HYDROGEN];
    let (_, na, nb) = electron_counts(&sp);

    let t = Instant::now();
    for i in 0..REPS {
        let _ = solve_geometry(&sp, centers(1.81 + 1e-3 * i as f64, 1.81, 1.82));
    }
    let whole = t.elapsed().as_secs_f64() / REPS as f64;

    let t = Instant::now();
    let mut probs = Vec::new();
    for i in 0..REPS {
        probs.push(geometry_problem(&sp, centers(1.81 + 1e-3 * i as f64, 1.81, 1.82)));
    }
    let assemble = t.elapsed().as_secs_f64() / REPS as f64;

    let t = Instant::now();
    for (space, mo, _) in &probs {
        let _ = solve(space, mo);
    }
    let ci = t.elapsed().as_secs_f64() / REPS as f64;

    let space = FciSpace::new(7, na, nb);
    println!("# (O,H,H): n_orb 7, n_alpha {na}, n_beta {nb}, n_det {}", space.n_det);
    println!("# whole solve_geometry : {:8.2} ms", whole * 1e3);
    println!("#   assemble + transform: {:8.2} ms   ({:4.1}%)", assemble * 1e3, 100.0 * assemble / whole);
    println!("#   CI eigensolve       : {:8.2} ms   ({:4.1}%)", ci * 1e3, 100.0 * ci / whole);
}
