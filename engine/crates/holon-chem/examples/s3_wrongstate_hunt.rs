//! WHERE does Davidson converge to the wrong eigenvector? A search, not an assumption.
//!
//! `saturation3-mesh` measured a wrong warm start landing 7.47 Ha above the ground state on
//! (H,H,Cl). At SATURATION-3's G0 compact geometry the same defect does NOT occur: a random
//! start recovers the ground state, and so does the worst possible single-determinant start
//! (the highest diagonal). So the failure is GEOMETRY-DEPENDENT, and a plant staked where it
//! does not occur VOIDs on an empty sector.
//!
//! This scans for the sector. For each geometry it runs the correct solve and then the
//! worst-case start, and reports where the two disagree. What it finds decides two things:
//! whether the guard can be demonstrated firing by this lane, and — more important for a
//! 34,500-node table — HOW COMMON the failure is.

use holon_chem::dual::D2;
use holon_chem::elements::{CHLORINE, HYDROGEN};
use holon_chem::fci::{ci_ints, solve_determinant, solve_determinant_from, Order};
use holon_chem::pair::geometry_problem;

fn c(x: f64, y: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(0.0)]
}

fn main() {
    println!("   {:>6} {:>6} {:>7} {:>14} {:>14} {:>10} {:>8}", "hh", "hcl", "angle", "E(correct)", "E(worst start)", "dE", "guard");
    let mut fired = 0usize;
    let mut n = 0usize;
    for &hh in &[0.8f64, 1.04, 1.5, 2.0, 2.6] {
        for &hcl in &[1.5f64, 1.90, 2.5, 3.2] {
            for &deg in &[40.0f64, 90.0, 150.0] {
                let th = deg * std::f64::consts::PI / 180.0;
                let (space, mo, _) = geometry_problem(
                    &[HYDROGEN, HYDROGEN, CHLORINE],
                    vec![c(0.0, 0.0), c(hh, 0.0), c(hcl * th.cos(), hcl * th.sin())],
                );
                let good = solve_determinant(&space, &mo);
                let ci = ci_ints(&mo, Order::Value);
                let diag = space.diagonal(&ci);
                let hi = diag
                    .iter()
                    .enumerate()
                    .fold((0usize, f64::NEG_INFINITY), |a, (i, &v)| if v > a.1 { (i, v) } else { a })
                    .0;
                let mut start = vec![0.0f64; space.n_det];
                start[hi] = 1.0;
                let bad = solve_determinant_from(&space, &mo, Some(&start));
                let de = bad.e.v - good.e.v;
                let m = bad.variational_margin.unwrap();
                n += 1;
                if de > 1e-3 {
                    fired += 1;
                    println!(
                        "   {hh:>6.2} {hcl:>6.2} {deg:>7.0} {:>14.6} {:>14.6} {de:>10.4} {:>8}",
                        good.e.v,
                        bad.e.v,
                        if m < 0.0 { "CATCHES" } else { "MISSES" }
                    );
                }
            }
        }
    }
    println!("\n   {fired} of {n} geometries converged to a different state from the worst start");
    if fired == 0 {
        println!("   The sector is EMPTY across this scan: at every geometry tried, Davidson");
        println!("   recovered the ground state even from the worst single-determinant start.");
    }
}
