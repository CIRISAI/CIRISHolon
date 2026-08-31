//! **The strict no-op control for M-MAINTENANCE-LENS row 4** — issue the warm-start command
//! and restore nothing, then check the answer did not move.
//!
//! The re-audit graded this lane's own control as the WEAKER kind: `wrong_start` supplies a
//! random vector, which is a MIS-TARGETED repair, not a no-op. The misfit's obligation asks for
//! the strict one — the command issued, nothing restored — so here it is.
//!
//! # What "restore nothing" means precisely, and it is not the obvious thing
//!
//! Passing the cold solve's *converged* vector would not be a no-op; it would be the strongest
//! possible warm start. The no-op is the vector the COLD path would have built for itself.
//!
//! The cold path builds `v0 = perturbation; v0[argmin diag] += 1.0` and normalises.
//! The warm path builds `v0 = w + perturbation` and normalises, with no `+= 1.0`.
//!
//! So the strict no-op is `w = e_argmin` — the unit vector on the lowest-diagonal determinant.
//! Then warm and cold construct the *same* first basis vector by different routes, and a
//! bit-identical energy is the only acceptable outcome.
//!
//! **If it is NOT bit-identical**, the warm-start machinery perturbs the answer independently of
//! the vector's content — which would mean every warm-started table entry carries a shift the
//! warm start's *content* never justified, and row 4's lens finding would be worse than
//! "control was the weaker kind".

use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::{ci_ints, solve_determinant, solve_determinant_from, Order};
use holon_chem::pair::geometry_problem;

fn at(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

fn triangle(x: f64, y: f64, u: f64) -> Vec<[D2; 3]> {
    let s = (1.0 - u * u).max(0.0).sqrt();
    vec![at(0.0, 0.0, 0.0), at(x, 0.0, 0.0), at(y * u, y * s, 0.0)]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let symbols: Vec<String> = args
        .first()
        .map(|s| s.split(',').map(|t| t.to_string()).collect())
        .unwrap_or_else(|| vec!["H".into(), "H".into(), "Cl".into()]);
    let species: Vec<_> = symbols
        .iter()
        .map(|s| by_symbol(s).unwrap_or_else(|| panic!("unknown species {s}")))
        .collect();

    println!("species {}", symbols.join(","));
    println!("=== strict no-op control: the command issued, nothing restored ===\n");

    // A few geometries, so one lucky agreement is not the whole result.
    let geoms = [
        (2.6_f64, 2.8_f64, 0.30_f64),
        (2.8, 3.0, 0.10),
        (3.0, 3.2, 0.50),
        (2.4, 3.4, -0.20),
    ];

    let mut identical = 0usize;
    let mut worst = 0.0f64;

    println!(
        "{:>22}  {:>24} {:>24} {:>11}  {}",
        "geometry", "E(cold)", "E(no-op warm)", "|dE|", "bits"
    );

    for (x, y, u) in geoms {
        let (space, mo, _) = geometry_problem(&species, triangle(x, y, u));

        // The vector the COLD path would have built for itself: a unit spike on the
        // lowest-diagonal determinant. The perturbation is added by the solver in both routes.
        let ci0 = ci_ints(&mo, Order::Value);
        let diag = space.diagonal(&ci0);
        let argmin = diag
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .expect("a non-empty diagonal");
        let mut noop = vec![0.0f64; space.n_det];
        noop[argmin] = 1.0;

        let cold = solve_determinant(&space, &mo);
        let warm = solve_determinant_from(&space, &mo, Some(&noop));

        let same = cold.e.v.to_bits() == warm.e.v.to_bits();
        let de = (cold.e.v - warm.e.v).abs();
        if same {
            identical += 1;
        }
        worst = worst.max(de);

        println!(
            "{:>22}  {:>24.15} {:>24.15} {:>11.3e}  {}",
            format!("({x:.1},{y:.1},{u:+.2})"),
            cold.e.v,
            warm.e.v,
            de,
            if same { "IDENTICAL" } else { "DIFFER" }
        );
        // M-VACUOUS-SUCCESS: both solves must have done real work, or "identical" is a
        // statement about two things that never ran.
        assert!(cold.davidson_iters > 0 && warm.davidson_iters > 0);
    }

    println!();
    println!("=== VERDICT ===");
    println!("bit-identical on {identical} of {} geometries; worst |dE| {worst:.3e} Ha", geoms.len());
    if identical == geoms.len() {
        println!(
            "STRICT NO-OP PASSES. Issuing the warm-start command while restoring nothing leaves \
             the answer bit-for-bit unchanged, so the machinery contributes no shift of its own \
             and every warm-start difference measured elsewhere is attributable to the vector's \
             CONTENT."
        );
    } else {
        println!(
            "STRICT NO-OP FAILS. The warm-start path moved the answer while restoring nothing, \
             so the shift is a property of the MACHINERY and not of the guess. Every warm-started \
             table entry carries a difference its warm start's content never justified."
        );
    }
}
