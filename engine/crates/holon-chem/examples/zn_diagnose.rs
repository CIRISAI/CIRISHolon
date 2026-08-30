//! Is zinc's quintet a fact about the model, or a Davidson that missed the ground state?
//!
//! A converged Davidson residual says the returned pair is AN eigenpair, not the LOWEST
//! one: with a start vector built from the diagonal, a ground state with little overlap on
//! that start can be stepped over. So this asks the one question that separates the two
//! causes, and it needs no reference table to do it.
//!
//! The smallest DIAGONAL element of H is the energy of the best single determinant in this
//! orbital basis, and it is a variational upper bound on the true ground state. If the
//! reported FCI energy sits above it, the solver missed; if below, the answer is real and
//! the model genuinely prefers the high-spin state.

use holon_chem::dual::D2;
use holon_chem::elements::by_z;
use holon_chem::fci::{ci_ints, s_squared, solve_determinant, Order};
use holon_chem::pair::{electron_counts, geometry_problem};

fn main() {
    // Cheapest first, so a partial run still answers the question: gallium shows the same
    // anomaly as zinc at a fifth the determinant count, and germanium and calcium are the
    // controls -- both are elements whose multiplicity came out RIGHT, so if the check
    // fires on them too it is the check that is wrong.
    for z in [32u32, 20, 31, 30] {
        let sp = by_z(z).unwrap();
        let (_, na, nb) = electron_counts(&[sp]);
        let (space, mo, _) = geometry_problem(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        let ci = ci_ints(&mo, Order::Value);
        let diag = space.diagonal(&ci);
        let best_det = diag.iter().copied().fold(f64::INFINITY, f64::min);

        let sol = solve_determinant(&space, &mo);
        let s2 = s_squared(&space, &sol.vector);
        let mult = (1.0 + 4.0 * s2).sqrt();

        println!(
            "{:>2} {:>2}  {:>8} dets  E_FCI {:>18.9}  best single det {:>18.9}  \
             gap {:>12.3e}  2S+1 {:.3}  resid {:.2e}  {}",
            z,
            sp.symbol,
            space.n_det,
            sol.e.v,
            best_det,
            best_det - sol.e.v,
            mult,
            sol.residual,
            if sol.e.v <= best_det {
                "OK: below the best determinant"
            } else {
                "SOLVER MISSED: above a determinant it contains"
            }
        );
    }
}
