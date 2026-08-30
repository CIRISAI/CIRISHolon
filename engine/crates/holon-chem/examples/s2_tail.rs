//! Why `dE3(O, H, H)` does not die on the far shell: a decomposition, not a guess.
//!
//! `s2_design`'s shell sweep reported a worst `|dE3|` of 9.6e-4 Ha with the larger O-H
//! side at 9 bohr and the smaller at 5 — a configuration where every atom is far from
//! every other and the three-body term should be at the level of the solver's residual.
//! Something in the decomposition is wrong, and the candidates are separable:
//!
//! 1. the (O,H,H) solve is landing on the wrong root at that geometry;
//! 2. the O-H pair curve is wrong at intermediate separation;
//! 3. the reference atom energies are wrong;
//! 4. the whole thing is orientation-dependent, i.e. the FCI is not rotationally
//!    invariant in this basis the way a full CI over a rotationally closed shell set
//!    must be.
//!
//! Each is READ here rather than argued.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_tail
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point, solve_geometry, PointSolution};

const PI: f64 = std::f64::consts::PI;

fn c3(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

fn ohh(x: f64, y: f64, theta: f64) -> PointSolution {
    solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![
            c3(0.0, 0.0, 0.0),
            c3(x, 0.0, 0.0),
            c3(y * theta.cos(), y * theta.sin(), 0.0),
        ],
    )
}

fn main() {
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let o = solve_geometry(&[OXYGEN], vec![c3(0.0, 0.0, 0.0)]);
    println!(
        "# E(O) = {e_o:.12}  (n_det {}, residual {:.2e}, scf_converged {})",
        o.n_det, o.residual, o.scf_converged
    );
    println!("# E(H) = {e_h:.12}\n");

    println!("## 1 — the O-H pair curve against its own asymptote");
    println!("   r        E_OH            V2 = E_OH - E(O) - E(H)   resid     scf");
    for r in [1.8, 2.5, 3.0, 4.0, 4.95, 5.0, 6.0, 7.0, 9.0, 12.0, 20.0] {
        let s = solve_geometry(&[OXYGEN, HYDROGEN], vec![c3(0.0, 0.0, 0.0), c3(0.0, 0.0, r)]);
        println!(
            "   {r:5.2}   {:.12}   {:+.6e}          {:.1e}   {}",
            s.e.v,
            s.e.v - e_o - e_h,
            s.residual,
            s.scf_converged
        );
    }

    println!("\n## 2 — orientation invariance of the same pair (a full CI must have it)");
    let r = 4.95;
    for (name, c) in [
        ("along z", c3(0.0, 0.0, r)),
        ("along x", c3(r, 0.0, 0.0)),
        ("along y", c3(0.0, r, 0.0)),
        ("in-plane diag", c3(r * 0.6, r * 0.8, 0.0)),
        ("off-axis", c3(r * 0.5774, r * 0.5774, r * 0.5774)),
    ] {
        let s = solve_geometry(&[OXYGEN, HYDROGEN], vec![c3(0.0, 0.0, 0.0), c]);
        println!("   {name:14}  E = {:.12}   resid {:.1e}  scf {}", s.e.v, s.residual, s.scf_converged);
    }

    println!("\n## 3 — the (O,H,H) solve on the far shell, decomposed");
    println!("   x      y     theta    E3               E3-E(O)-2E(H)   V2(x)+V2(y)+V2(z)   dE3        resid    scf");
    for (x, y, deg) in [
        (4.95, 9.0, 24.5),
        (4.95, 9.0, 60.0),
        (4.95, 9.0, 120.0),
        (4.95, 9.0, 180.0),
        (3.0, 9.0, 24.5),
        (2.0, 9.0, 24.5),
        (1.94, 9.0, 96.76),
        (5.0, 5.0, 90.0),
        (6.0, 6.0, 90.0),
        (8.0, 8.0, 90.0),
    ] {
        let th = deg * PI / 180.0;
        let s = ohh(x, y, th);
        let z = (x * x + y * y - 2.0 * x * y * th.cos()).sqrt();
        let v2 = (pair_point(OXYGEN, HYDROGEN, x).e - e_o - e_h)
            + (pair_point(OXYGEN, HYDROGEN, y).e - e_o - e_h)
            + (pair_point(HYDROGEN, HYDROGEN, z).e - 2.0 * e_h);
        let tot = s.e.v - e_o - 2.0 * e_h;
        println!(
            "   {x:4.2}  {y:5.2}  {deg:6.1}  {:.10}  {:+.6e}    {:+.6e}       {:+.4e}  {:.1e}  {}",
            s.e.v,
            tot,
            v2,
            tot - v2,
            s.residual,
            s.scf_converged
        );
    }

    println!("\n## 4 — the fully dispersed limit: does E(OHH) reach E(O) + 2E(H)?");
    for d in [8.0, 10.0, 12.0, 16.0, 20.0, 30.0] {
        let s = ohh(d, d, PI / 2.0);
        println!(
            "   x = y = {d:5.1}, theta = 90   E3 - E(O) - 2E(H) = {:+.6e}   resid {:.1e}  scf {}",
            s.e.v - e_o - 2.0 * e_h,
            s.residual,
            s.scf_converged
        );
    }

    println!("\n## 5 — is the far-shell excess in the TRIPLE or in the PAIR? Two H at right");
    println!("   angles to each other, both far: the pair terms are then near zero and any");
    println!("   residue is the triple's own.");
    for d in [5.0, 6.0, 7.0, 8.0, 10.0, 14.0] {
        let s3 = solve_geometry(
            &[OXYGEN, HYDROGEN, HYDROGEN],
            vec![c3(0.0, 0.0, 0.0), c3(d, 0.0, 0.0), c3(0.0, d, 0.0)],
        );
        let a = solve_geometry(&[OXYGEN, HYDROGEN], vec![c3(0.0, 0.0, 0.0), c3(d, 0.0, 0.0)]);
        let b = solve_geometry(&[OXYGEN, HYDROGEN], vec![c3(0.0, 0.0, 0.0), c3(0.0, d, 0.0)]);
        let hh = solve_geometry(
            &[HYDROGEN, HYDROGEN],
            vec![c3(d, 0.0, 0.0), c3(0.0, d, 0.0)],
        );
        let de3 = s3.e.v + e_o + 2.0 * e_h - a.e.v - b.e.v - hh.e.v;
        println!(
            "   d = {d:5.1}   V2(a) = {:+.4e}  V2(b) = {:+.4e}  V2(HH) = {:+.4e}   dE3 = {:+.6e}",
            a.e.v - e_o - e_h,
            b.e.v - e_o - e_h,
            hh.e.v - 2.0 * e_h,
            de3
        );
    }
}
