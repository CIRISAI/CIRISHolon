//! Does the variational guard `E <= min_i H_ii` have ZERO false positives on the real
//! table path?
//!
//! # Why this check and not the mesh lane's
//!
//! `saturation3-mesh` measured that a deliberately wrong warm start converges cleanly onto
//! the WRONG EIGENVECTOR — 7.47 Ha above the ground state, with a residual (5.98e-11)
//! indistinguishable from the correct solve's (5.24e-11) and an IDENTICAL exit reason. No
//! residual threshold can catch that, because a residual is small for ANY eigenvector. They
//! proposed the guard and demonstrated it FIRING on the plant.
//!
//! A guard that fires on the plant is half a guard. The half that decides whether it can go
//! into a SHARED solver is the other one: does it stay silent on every solve that is
//! correct? A false positive here does not produce a wrong number — it VOIDs a good node,
//! and at (O,O,O)'s 39.8 s a point a guard that voids one node in a thousand is a guard
//! that costs the campaign days and produces a table with holes.
//!
//! So this measures the MARGIN — how far below `min_i H_ii` each correct solve actually
//! sits — across all five of G0's staked combos, which is every triple type the campaign
//! will build.
//!
//! # The bound, and exactly how far it goes
//!
//! For any normalised trial vector `psi`, `E_0 <= <psi|H|psi>`. A single determinant `|i>`
//! is such a vector and gives `H_ii`. So `E_0 <= H_ii` for every `i`, hence
//!
//! ```text
//! E_0 <= min_i H_ii
//! ```
//!
//! rigorously, with no assumption about the solver. `diag` is already computed for
//! Davidson's preconditioner, so the check costs one pass over an array the solve is
//! holding anyway.
//!
//! WHAT IT DOES NOT CATCH, stated because a guard oversold is worse than no guard: it
//! catches a converged-to-an-excited-state result only when that state lies ABOVE
//! `min_i H_ii`. An excited state below that line passes. The guard is NECESSARY, not
//! sufficient — it is a cheap bound, not a proof of groundness, and the `<S^2>` audit and
//! the dual-route agreement remain the things that catch the rest.
//!
//! ```text
//! cargo run --release -p holon-chem --example s3_variational_guard
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{Species, CHLORINE, HYDROGEN, OXYGEN};
use holon_chem::fci::{ci_ints, solve_determinant, Order};
use holon_chem::pair::{geometry_problem, pair_point};

const COMPACT_FRACTION: f64 = 0.75;

fn c3(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

fn locate_r_e(a: Species, b: Species) -> f64 {
    let (lo, hi, n) = (1.0f64, 8.0f64, 29);
    let mut best = (f64::INFINITY, lo);
    for i in 0..n {
        let r = lo + (hi - lo) * i as f64 / (n - 1) as f64;
        let e = pair_point(a, b, r).e;
        if e < best.0 {
            best = (e, r);
        }
    }
    best.1
}

fn triangle(ab: f64, ac: f64, bc: f64) -> Vec<[D2; 3]> {
    let cos = ((ab * ab + ac * ac - bc * bc) / (2.0 * ab * ac)).clamp(-1.0, 1.0);
    let sin = (1.0 - cos * cos).max(0.0).sqrt();
    vec![c3(0.0, 0.0, 0.0), c3(ab, 0.0, 0.0), c3(ac * cos, ac * sin, 0.0)]
}

fn main() {
    println!("# the variational guard E <= min_i H_ii, false-positive check\n");
    println!("# electronic energies throughout: `diag` is the electronic diagonal and");
    println!("# `Solution::e` is the electronic energy, so no nuclear term enters either side.\n");

    let hh = COMPACT_FRACTION * locate_r_e(HYDROGEN, HYDROGEN);
    let hcl = COMPACT_FRACTION * locate_r_e(HYDROGEN, CHLORINE);
    let clcl = COMPACT_FRACTION * locate_r_e(CHLORINE, CHLORINE);
    let oo = COMPACT_FRACTION * locate_r_e(OXYGEN, OXYGEN);
    let oh = COMPACT_FRACTION * locate_r_e(OXYGEN, HYDROGEN);

    let combos: [(&str, [Species; 3], [f64; 3]); 5] = [
        ("(H,H,Cl)", [HYDROGEN, HYDROGEN, CHLORINE], [hh, hcl, hcl]),
        ("(H,Cl,Cl)", [HYDROGEN, CHLORINE, CHLORINE], [hcl, hcl, clcl]),
        ("(Cl,Cl,Cl)", [CHLORINE; 3], [clcl, clcl, clcl]),
        ("(O,O,H)", [OXYGEN, OXYGEN, HYDROGEN], [oo, oh, oh]),
        ("(O,O,O)", [OXYGEN; 3], [oo, oo, oo]),
    ];

    println!(
        "   {:>11} {:>9} {:>16} {:>16} {:>13} {:>8}",
        "combo", "n_det", "E (electronic)", "min_i H_ii", "margin", "guard"
    );
    let mut worst_margin = f64::INFINITY;
    let mut false_positives = 0usize;
    for (name, sp, sides) in combos {
        let (space, mo, _nuc) = geometry_problem(&sp, triangle(sides[0], sides[1], sides[2]));
        let ci = ci_ints(&mo, Order::Value);
        let diag = space.diagonal(&ci);
        let min_diag = diag.iter().cloned().fold(f64::INFINITY, f64::min);
        let sol = solve_determinant(&space, &mo);
        // The guard as it would be applied: a solve whose energy is ABOVE the bound cannot
        // be the ground state and the node VOIDs.
        let margin = min_diag - sol.e.v;
        let passes = sol.e.v <= min_diag;
        if !passes {
            false_positives += 1;
        }
        worst_margin = worst_margin.min(margin);
        println!(
            "   {name:>11} {:>9} {:>16.6} {min_diag:>16.6} {margin:>13.6} {:>8}",
            space.n_det,
            sol.e.v,
            if passes { "pass" } else { "VOID" }
        );
    }

    println!("\n   false positives: {false_positives} of 5");
    println!("   smallest margin below the bound: {worst_margin:.6} Ha");
    if false_positives == 0 {
        println!(
            "\n   The guard is SILENT on every correct solve, with the tightest margin\n   \
             {worst_margin:.4} Ha — many orders above the 1e-11 scale a residual lives at, so\n   \
             it is not a threshold that could drift into firing. Safe for a shared solver."
        );
    } else {
        println!(
            "\n   The guard FIRES on a correct solve. It must NOT go into a shared solver:\n   \
             a guard that voids good nodes at 39.8 s a point costs the campaign days and\n   \
             leaves a table with holes."
        );
    }
}
