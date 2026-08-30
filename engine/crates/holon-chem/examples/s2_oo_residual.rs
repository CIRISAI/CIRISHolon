//! WHERE the O-O curve fails to converge, and whether P1's reading depends on it.
//!
//! P1's mixed and oxygen arms both load an O-O curve whose `worst_residual` is 1.3e-4 Ha,
//! against `pair::CONVERGED_RESIDUAL = 1e-10` — the bar that crate documents as "the worst
//! Davidson residual a generated curve may carry and still be called converged". The
//! harness PRINTED that number and did not act on it, which is the defect shape
//! `CONVERGED_RESIDUAL`'s own doc comment describes: a curve that hit its iteration cap
//! emitted looking perfectly healthy, with the evidence in a field no consumer is required
//! to read.
//!
//! So: where is it? A residual in the DISSOCIATION tail is the near-degeneracy this lane
//! already met in the referee (O's 3P times O's 3P), and it moves nothing the bond
//! criterion reads. A residual in the WELL would put P1's oxygen aggregation in question.
//! This separates the two rather than assuming which it is.

use holon_chem::elements::OXYGEN;
use holon_chem::pair::{solve_geometry, CONVERGED_RESIDUAL};
use holon_chem::dual::D2;

fn main() {
    println!("# CONVERGED_RESIDUAL = {CONVERGED_RESIDUAL:.0e} Ha");
    println!("   {:>6} {:>18} {:>12} {:>8}", "r", "E(O2)", "residual", "over?");
    let mut worst = (0.0f64, 0.0f64);
    for i in 0..28 {
        let r = 1.6 + (9.0 - 1.6) * i as f64 / 27.0;
        let s = solve_geometry(
            &[OXYGEN, OXYGEN],
            vec![
                [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                [D2::c(0.0), D2::c(0.0), D2::c(r)],
            ],
        );
        if s.residual > worst.0 {
            worst = (s.residual, r);
        }
        println!(
            "   {r:>6.2} {:>18.9} {:>12.2e} {:>8}",
            s.e.v,
            s.residual,
            if s.residual > CONVERGED_RESIDUAL { "YES" } else { "" }
        );
    }
    println!("\n   worst residual {:.2e} at r = {:.2} bohr", worst.0, worst.1);
    println!(
        "   O2's own well is near 2.44 bohr (R_e from the 96-knot curve), so a residual\n   \
         living past 5 bohr is the dissociation near-degeneracy and not the bond."
    );
}
