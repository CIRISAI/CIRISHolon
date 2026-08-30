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
/// # The controls are not padding, and choosing them corrected the hypothesis
///
/// If nothing classifies as CONVERGED then that branch never runs, and the output cannot
/// distinguish "these solves do not converge" from "this diagnostic cannot recognise
/// convergence". So controls that MUST converge have to be in the list.
///
/// Picking them is what showed the phenomenon is not about heavy atoms at all. The first
/// three chosen -- Li, C, Ne -- were assumed to converge because their spaces are tiny. Two
/// of them do not: lithium is FIFTY determinants and stops at 7.03e-11, carbon at 3.35e-11.
/// Reading the record's whole light block, exactly one atom with more than one determinant
/// reaches the 1e-11 target: fluorine, at 5.68e-14 on five determinants. Everything else
/// from lithium to indium piles up between 1.1e-11 and 1.0e-10 with no dependence on space
/// size across four orders of magnitude.
///
/// So the controls are now fluorine (converges inside the loop) and neon (one determinant,
/// which returns exactly before iterating). Lithium and beryllium are in the list as
/// EVIDENCE rather than as controls: a fifty-determinant space that misses the target
/// cannot be explained by "not given long enough".
const ASK: [(u32, &str); 11] = [
    (9, "CONTROL: the only atom in the record that converges inside the loop"),
    (10, "CONTROL: one determinant, exact without ever iterating"),
    (3, "50 determinants -- and still short of the target"),
    (4, "100 determinants"),
    (19, "published: 9.95e-11, just inside the bar"),
    (32, "published: 8.85e-11"),
    (33, "published: 9.80e-11, just inside the bar"),
    (51, "REFUSED: 1.07e-10, just outside the bar"),
    (52, "REFUSED: 1.07e-10, just outside the bar"),
    (50, "REFUSED: 2.06e-10"),
    (30, "REFUSED: 2.72e-10"),
];

fn main() {
    // Optional Z arguments override the default list, so a single expensive atom can be
    // asked on its own. Indium is the reason this exists: the record calls its status "did
    // not converge at the production cap", which is an INFERENCE from the fact that it runs
    // for the better part of an hour while every other atom returns in seconds. Every atom
    // measured so far stagnated instead, none of them anywhere near the cap, so that
    // inference needs checking rather than repeating -- and asking indium costs a run of
    // its own, which is why it is not in the default list.
    let argv: Vec<u32> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let ask: Vec<(u32, &str)> = if argv.is_empty() {
        ASK.to_vec()
    } else {
        argv.iter()
            .map(|z| (*z, "asked on the command line"))
            .collect()
    };

    let cap = DAVIDSON_MAX_ITER.load(std::sync::atomic::Ordering::Relaxed);
    println!("# Davidson exit taken, per atom. cap = {cap}, target = {DAVIDSON_TARGET:.0e}, bar = {CONVERGED_RESIDUAL:.0e}");
    println!("# exit CONVERGED: resid < target | CAP: iters == cap | STAGNATED: neither");
    println!(
        "{:>3} {:>3} {:>10} {:>22} {:>11} {:>7} {:>11} {:>8}",
        "Z", "sym", "n_det", "E (hartree)", "resid", "iters", "exit", "vs bar"
    );

    /// The three light atoms whose job is to prove the CONVERGED branch is reachable.
    const CONTROLS: [u32; 2] = [9, 10];
    let mut controls_converged = 0usize;
    let mut heavy_converged = 0usize;
    for (z, why) in ask.iter().copied() {
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
    let controls_present = ask.iter().filter(|(z, _)| CONTROLS.contains(z)).count();
    if controls_present == 0 {
        println!(
            "# NO CONTROLS IN THIS RUN: an explicit atom list was given, so the CONVERGED \
             branch was never required to fire and this run cannot vouch for its own \
             classifier. Read its exits against a default run that carries the controls."
        );
    } else if controls_converged < controls_present {
        println!(
            "# INSTRUMENT FAILURE, not a finding: only {controls_converged} of \
             {controls_present} controls reached the target. This run cannot tell 'these \
             solves do not converge' from 'this classifier cannot see convergence', and \
             the readings above are not evidence of anything."
        );
    } else {
        println!(
            "# All {controls_present} controls reached the target, so the CONVERGED branch \
             is live and an atom missing it is a fact about the atom, not about this \
             classifier. Non-control atoms that reached it: {heavy_converged}."
        );
    }
    println!(
        "# Read the `exit` column against the `vs bar` column. If they disagree -- if rows \
         that PASS the bar took the same exit as rows the bar REFUSES -- then the bar is \
         sorting a class that uniformly failed to converge, rather than separating the \
         converged from the rest, and the record has to say so."
    );
}
