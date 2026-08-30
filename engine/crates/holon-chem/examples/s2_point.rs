//! One (O, H, H) geometry through the engine, printed at full f64 width — the spot check
//! that the 50-digit referee and this implementation are solving the same model before an
//! hour of referee compute is spent on the whole staked set.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_point -- 1.5 2.0 1.0
//! ```
//!
//! Arguments are `x` (the shorter O-H side), `y` (the longer), and
//! `c = sqrt(1 - cos theta_HOH)`, the table's third coordinate.

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};

fn c3(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

fn main() {
    let a: Vec<f64> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let (x, y, c) = (a[0], a[1], a[2]);
    let u = 1.0 - c * c;
    let sn = (1.0 - u * u).max(0.0).sqrt();
    let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();

    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let w = solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![c3(0.0, 0.0, 0.0), c3(x, 0.0, 0.0), c3(y * u, y * sn, 0.0)],
    );
    let p1 = pair_point(OXYGEN, HYDROGEN, x).e;
    let p2 = pair_point(OXYGEN, HYDROGEN, y).e;
    let p3 = pair_point(HYDROGEN, HYDROGEN, z).e;
    let d = w.e.v + e_o + 2.0 * e_h - p1 - p2 - p3;

    println!("x      = {x:.17}");
    println!("y      = {y:.17}");
    println!("c      = {c:.17}");
    println!("u      = {u:.17}");
    println!("z      = {z:.17}");
    println!("E_O    = {e_o:.17}");
    println!("E_H    = {e_h:.17}");
    println!("E_OHH  = {:.17}   (n_det {}, resid {:.2e})", w.e.v, w.n_det, w.residual);
    println!("E_OH_x = {p1:.17}");
    println!("E_OH_y = {p2:.17}");
    println!("E_HH_z = {p3:.17}");
    println!("dE3    = {d:.17}");
}
