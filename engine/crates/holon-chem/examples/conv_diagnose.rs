//! Which of Davidson's three exits did each heavy atom take?
//!
//! ```text
//! cargo run --release -p holon-chem --example conv_diagnose
//! ```
//!
//! # Why this exists
//!
//! The R1 record refuses a row when `sol.residual > pair::CONVERGED_RESIDUAL` (1e-10). That
//! bar's own doc says it is "an order above davidson's own 1e-11 target, so an ordinary
//! solve clears it and a solve that gave up does not" -- so a refusal is supposed to mean
//! the solve gave up and a published row is supposed to mean it did not.
//!
//! Reading the record against the solver says otherwise. `solve_determinant` calls
//! `davidson(.., 1e-11, ..)`, and EVERY multi-determinant heavy atom comes back above
//! 1e-11 -- the published ones (Br 3.23e-11, Ge 8.85e-11, As 9.80e-11) as well as the
//! refused ones (Sb 1.07e-10, Sn 2.06e-10, Zn 2.72e-10). If that reading is right then
//! nothing in the record converged, the bar sorts rather than separates, and the line
//! between published and refused falls between arsenic and antimony at a factor of 1.1.
//!
//! # The distinction this measures, which the record cannot make
//!
//! `davidson_eigh` has THREE exits and the record keeps only the residual:
//!
//! * `resid < tol` -- converged;
//! * `iter + 1 == max_iter` -- the iteration cap;
//! * no expansion vector survives orthogonalisation -- SUBSPACE STAGNATION.
//!
//! A residual above tol is consistent with the last two, and they are different facts: a
//! cap says "not given long enough", stagnation says "the subspace stopped being able to
//! improve", and only one of them is fixed by patience. `Solution::davidson_iters` tells
//! them apart and no consumer reads it. So this prints it.
//!
//! # What this does NOT do
//!
//! It does not adjust `DAVIDSON_MAX_ITER`, which is `#[doc(hidden)]` and reserved for
//! `tests/front_door.rs`. It reads it, to classify. A diagnostic that changed the thing it
//! measures would be measuring itself.

use holon_chem::dual::D2;
use holon_chem::elements::by_z;
use holon_chem::fci::{solve_determinant, DAVIDSON_MAX_ITER};
use holon_chem::pair::{geometry_problem, CONVERGED_RESIDUAL};

/// Davidson's own target inside `solve_determinant`. Not a constant this crate exports --
/// it is a literal at the call site -- so it is repeated here and its provenance named.
/// If the call site moves, this diagnostic is reading a stale number and says so wrongly.
const DAVIDSON_TARGET: f64 = 1e-11;

/// The atoms to ask, and why each one is in the list.
///
/// Both sides of the bar, deliberately: the four rows the convergence verdict REFUSES and
/// five it PUBLISHES. A diagnostic run only on the refused rows could not tell whether the
/// exit it found was a property of failing or a property of the whole class -- which is the
/// question actually being asked.
///
/// # The light controls are not padding
///
/// If the hypothesis is right, NO heavy atom classifies as CONVERGED -- and then the
/// classifier's `CONVERGED` branch is never exercised and the run cannot distinguish "the
/// heavy class uniformly fails to converge" from "this diagnostic cannot recognise
/// convergence". So three light atoms are included whose spaces are small enough that they
/// must reach the target. If they do not, the instrument is what is broken, and the heavy
/// readings mean nothing. That check has to be IN the run rather than assumed about it.
const ASK: [(u32, &str); 12] = [
    (3, "light control: must CONVERGE"),
    (6, "light control: must CONVERGE"),
    (10, "light control: must CONVERGE"),
    (30, "refused: residual 2.72e-10"),
    (50, "refused: residual 2.06e-10"),
    (51, "refused: residual 1.07e-10"),
    (52, "refused: residual 1.07e-10"),
    (34, "published: residual 5.16e-11"),
    (33, "published: residual 9.80e-11"),
    (32, "published: residual 8.85e-11"),
    (31, "published: residual 7.04e-11"),
    (53, "published: residual 2.47e-11"),
];

fn main() {
    let cap = DAVIDSON_MAX_ITER.load(std::sync::atomic::Ordering::Relaxed);
    println!("# Davidson exit taken, per atom. cap = {cap}, target = {DAVIDSON_TARGET:.0e}, bar = {CONVERGED_RESIDUAL:.0e}");
    println!("# exit CONVERGED: resid < target | CAP: iters == cap | STAGNATED: neither");
    println!(
        "{:>3} {:>3} {:>10} {:>22} {:>11} {:>7} {:>11} {:>8}",
        "Z", "sym", "n_det", "E (hartree)", "resid", "iters", "exit", "vs bar"
    );

    /// The three light atoms whose job is to prove the CONVERGED branch is reachable.
    const CONTROLS: [u32; 3] = [3, 6, 10];
    let mut controls_converged = 0usize;
    let mut heavy_converged = 0usize;
    for (z, why) in ASK {
        let sp = by_z(z).unwrap();
        let (space, mo, _) =
            geometry_problem(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        let sol = solve_determinant(&space, &mo);

        let exit = if sol.residual < DAVIDSON_TARGET {
            if CONTROLS.contains(&z) {
                controls_converged += 1;
            } else {
                heavy_converged += 1;
            }
            "CONVERGED"
        } else if sol.davidson_iters >= cap {
            "CAP"
        } else {
            "STAGNATED"
        };
        println!(
            "{:>3} {:>3} {:>10} {:>22.12} {:>11.2e} {:>7} {:>11} {:>8}   # {}",
            sp.z,
            sp.symbol,
            space.n_det,
            sol.e.v,
            sol.residual,
            sol.davidson_iters,
            exit,
            if sol.residual <= CONVERGED_RESIDUAL { "pass" } else { "REFUSED" },
            why
        );
    }

    println!("#");
    if controls_converged < CONTROLS.len() {
        println!(
            "# INSTRUMENT FAILURE, not a finding: only {controls_converged} of \
             {} light controls reached the target. This run cannot tell 'the heavy class \
             does not converge' from 'this classifier cannot see convergence', and the \
             heavy readings above are not evidence of anything.",
            CONTROLS.len()
        );
    } else {
        println!(
            "# All {} light controls reached the target, so the CONVERGED branch is live \
             and a heavy atom missing it is a fact about the atom, not about this \
             classifier. Heavy atoms that reached it: {heavy_converged}.",
            CONTROLS.len()
        );
    }
    println!(
        "# Read the `exit` column against the `vs bar` column. If they disagree -- if rows \
         that PASS the bar took the same exit as rows the bar REFUSES -- then the bar is \
         sorting a class that uniformly failed to converge, rather than separating the \
         converged from the rest, and the record has to say so."
    );
}
