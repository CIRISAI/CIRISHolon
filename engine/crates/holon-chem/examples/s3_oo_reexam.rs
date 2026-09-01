//! RE-EXAMINATION: SATURATION-2's P1 disclosed the O-O curve's non-convergence as
//! "harmless dissociation-tail stagnation". The exit reason did not exist when that was
//! written, and it says something else.
//!
//! # What was published, and what is now known to be wrong about it
//!
//! `SATURATION2_RESULTS.md`'s DISCLOSED section says the O-O curve's `worst_residual` is
//! 1.3e-4 Ha, locates every offending knot at `r >= 4.34` bohr, and concludes that the
//! unconverged region is "the dissociation near-degeneracy" and "flat as well as distant",
//! so nothing P1 reads depends on it. Two things in that paragraph are now suspect:
//!
//! 1. **"Stagnation" was an inference, not a reading.** `SolveExit` did not exist when
//!    that ran; the characterisation came from the residual alone. The 2026-08-30 bar
//!    measurement (`s3_bar_margin`) shows O-O exiting `IterationCap`, which is a solve
//!    that GAVE UP at 1200 iterations, not one that reached the f64 tier's floor. Every
//!    other multi-determinant curve in the crate stops at the floor with `Stagnated`; O-O
//!    is the one that does not, and it was described as if it were one of them.
//! 2. **"The bond criterion does not read it" is too strong as written.** `Sim`'s
//!    criterion is `e_rel = ke_rel + u(r) < 0 && r < r_outer`, and `u` is the knot energy
//!    minus the asymptote — so a tail knot in error by `d` shifts `u` there by `d`, and
//!    `outer_turning_point` solves for a crossing in exactly that region. The tail is
//!    read. Whether it MATTERS is a quantitative question, and this asks it.
//!
//! # What this measures
//!
//! The same 96-knot O-O grid `waterquench.rs` loads, solved twice:
//!
//! * PRODUCTION — `DAVIDSON_MAX_ITER` at its shipped 1200, which is what P1 ran.
//! * REFERENCE — the identical path with the cap raised. Same start vector, same
//!   tolerance, same everything else, so the only difference is how long the solve is
//!   allowed to keep going.
//!
//! `dE = E_prod - E_ref` is then a LOWER bound on production's error at that knot (both
//! are Ritz values, so both are upper bounds on the true energy and the smaller one is the
//! better one). Where the reference exits `Converged`, `dE` is the error to within the
//! reference's own `resid^2/gap`, which is negligible. Where the reference ALSO fails, the
//! knot is reported as unresolved rather than scored — a reference that gave up cannot
//! referee a production run that gave up.
//!
//! The scoring scale is the quench's own: `kT` at P1's initial 3000 K and target 300 K.
//! An energy error far below `kT` cannot change which pairs read `e_rel < 0`, because the
//! kinetic term it competes with is drawn from that distribution.
//!
//! ```text
//! cargo run --release -p holon-chem --example s3_oo_reexam -- [ref_cap] [baseline_budget]
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::OXYGEN;
use holon_chem::fci::{
    ci_ints, Order, SolveExit, DAVIDSON_MAX_ITER,
    DAVIDSON_REQUESTED_TOLERANCE,
};
use holon_chem::pair::{atom_energy, derive_range, geometry_problem, solve_geometry, CONVERGED_RESIDUAL};
use holon_chem::table::grid_point;
use std::sync::atomic::Ordering;
use std::time::Instant;

/// `waterquench.rs`'s `CURVE_KNOTS`. The grid P1 actually loaded, not a probe of my own.
const N_KNOTS: usize = 96;

/// Boltzmann's constant in hartree per kelvin.
const KB_HA: f64 = 3.166_811_563e-6;

/// P1's frozen thermostat schedule, for scale only.
const T_INIT: f64 = 3000.0;
const T_TARGET: f64 = 300.0;

fn geom(r: f64) -> Vec<[D2; 3]> {
    vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
        [D2::c(0.0), D2::c(0.0), D2::var(r)],
    ]
}

/// One knot, solved at whatever the global cap currently says.
struct Knot {
    r: f64,
    e: f64,
    f: f64,
    resid: f64,
    iters: usize,
    exit: SolveExit,
}

fn solve_knot(r: f64) -> Knot {
    let s = solve_geometry(&[OXYGEN, OXYGEN], geom(r));
    Knot {
        r,
        e: s.e.v,
        f: -s.e.d,
        resid: s.residual,
        iters: s.davidson_iters,
        exit: s.exit,
    }
}

/// The variational margin at this knot: `min_i H_ii - E`. Negative is a proof of a wrong
/// answer; positive is necessary and not sufficient. Computed here rather than read off
/// `solve_geometry`, which does not carry it out through `PointSolution`.
fn margin(r: f64, e_total: f64) -> f64 {
    let (space, mo, nuc) = geometry_problem(&[OXYGEN, OXYGEN], geom(r));
    let ci0 = ci_ints(&mo, Order::Value);
    let diag = space.diagonal(&ci0);
    let min_diag = diag.iter().copied().fold(f64::INFINITY, f64::min);
    // `diag` is electronic; `e_total` carries nuclear repulsion. Put them on one footing.
    min_diag - (e_total - nuc.v)
}

fn main() {
    let ref_cap: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(20_000);
    // The BASELINE budget, optional second argument. Defaulting to the process default
    // makes this the re-examination it was written as; naming a lower one turns the same
    // instrument into a blast-radius report for a budget change, which is what the
    // 4000 -> 5000 ruling needs and is the same measurement either way.
    let prod_cap: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| DAVIDSON_MAX_ITER.load(Ordering::Relaxed));
    DAVIDSON_MAX_ITER.store(prod_cap, Ordering::Relaxed);

    let e_asym = 2.0 * atom_energy(OXYGEN);
    let (r_min, r_max) = derive_range(OXYGEN, OXYGEN, e_asym);

    println!("# O-O re-examination — the curve SATURATION-2's P1 loaded, scored at the derived bar");
    println!("# grid: derive_range(O,O) = [{r_min:.4}, {r_max:.4}] bohr, {N_KNOTS} knots, generate_pair_table's own placement");
    println!("# production cap {prod_cap}, reference cap {ref_cap}, tolerance {DAVIDSON_REQUESTED_TOLERANCE:.0e} (asked = DAVIDSON_EXPANSION_FLOOR)");
    println!("# bar: CONVERGED_RESIDUAL = {CONVERGED_RESIDUAL:.0e} Ha (10x the floor, derived)");
    println!("# E_asymptote = 2 E(O) = {e_asym:.9} Ha; u(r) = E(r) - E_asymptote is what Sim reads");
    println!("# kT = {:.3e} Ha at {T_INIT} K, {:.3e} Ha at {T_TARGET} K", KB_HA * T_INIT, KB_HA * T_TARGET);
    println!();

    // --- PHASE A: production, exactly as shipped -----------------------------------
    let t0 = Instant::now();
    let mut prod: Vec<Knot> = Vec::with_capacity(N_KNOTS);
    for i in 0..N_KNOTS {
        prod.push(solve_knot(grid_point(r_min, r_max, N_KNOTS, i)));
    }
    println!("# phase A (production) done in {:.1} s", t0.elapsed().as_secs_f64());

    let n_conv = prod.iter().filter(|k| k.exit == SolveExit::Converged).count();
    let n_cap = prod.iter().filter(|k| k.exit == SolveExit::IterationCap).count();
    let n_stag = prod.iter().filter(|k| k.exit == SolveExit::Stagnated).count();
    println!("# exits: {n_conv} converged, {n_stag} stagnated, {n_cap} ITERATION CAP, of {N_KNOTS}");
    let worst = prod
        .iter()
        .max_by(|a, b| a.resid.partial_cmp(&b.resid).unwrap())
        .unwrap();
    println!("# worst residual {:.3e} Ha at r = {:.4} bohr ({})", worst.resid, worst.r, worst.exit.label());
    println!();

    // --- PHASE B: reference, cap raised, everything else identical -------------------
    DAVIDSON_MAX_ITER.store(ref_cap, Ordering::Relaxed);
    let t1 = Instant::now();
    let mut refr: Vec<Knot> = Vec::with_capacity(N_KNOTS);
    for k in prod.iter() {
        // Only knots production did not finish need a reference. A knot that exited
        // `Converged` under the shipped cap is already at the asked tolerance and a longer
        // run cannot move it: the loop returns on the same test.
        if k.exit == SolveExit::Converged {
            refr.push(Knot { r: k.r, e: k.e, f: k.f, resid: k.resid, iters: k.iters, exit: k.exit });
        } else {
            refr.push(solve_knot(k.r));
        }
    }
    DAVIDSON_MAX_ITER.store(prod_cap, Ordering::Relaxed);
    println!("# phase B (reference) done in {:.1} s", t1.elapsed().as_secs_f64());
    println!();

    // --- THE TABLE ------------------------------------------------------------------
    println!(
        "   {:>2} {:>8} {:>15} {:>10} {:>6} {:>13} | {:>11} {:>6} {:>13} | {:>11} {:>11}",
        "i", "r", "E_prod", "resid", "iters", "exit", "resid_ref", "iters", "exit_ref", "dE", "dF"
    );
    let mut worst_de = (0.0f64, 0.0f64);
    let mut worst_df = (0.0f64, 0.0f64);
    let mut unresolved = 0usize;
    for (i, (p, q)) in prod.iter().zip(refr.iter()).enumerate() {
        let de = p.e - q.e;
        let df = p.f - q.f;
        let same = p.exit == SolveExit::Converged;
        if !same {
            if q.exit == SolveExit::Converged {
                if de.abs() > worst_de.0 {
                    worst_de = (de.abs(), p.r);
                }
                if df.abs() > worst_df.0 {
                    worst_df = (df.abs(), p.r);
                }
            } else {
                unresolved += 1;
            }
        }
        println!(
            "   {i:>2} {:>8.4} {:>15.9} {:>10.2e} {:>6} {:>13} | {:>11} {:>6} {:>13} | {:>11} {:>11}",
            p.r,
            p.e,
            p.resid,
            p.iters,
            p.exit.label(),
            if same { "-".to_string() } else { format!("{:.2e}", q.resid) },
            if same { "-".to_string() } else { q.iters.to_string() },
            if same { "-" } else { q.exit.label() },
            if same { "-".to_string() } else { format!("{de:.3e}") },
            if same { "-".to_string() } else { format!("{df:.3e}") },
        );
    }
    println!();

    // --- WHERE THE WELL IS, AND WHERE THE FAILURES ARE -------------------------------
    let r_e = prod
        .iter()
        .min_by(|a, b| a.e.partial_cmp(&b.e).unwrap())
        .unwrap();
    println!("   R_e (grid minimum)          {:.4} bohr, E = {:.9}, D_e = {:.6} Ha", r_e.r, r_e.e, e_asym - r_e.e);
    let first_bad = prod.iter().find(|k| k.exit != SolveExit::Converged);
    match first_bad {
        Some(k) => println!("   first non-converged knot    {:.4} bohr ({})", k.r, k.exit.label()),
        None => println!("   first non-converged knot    NONE — every knot converged"),
    }
    let bad_in_well = prod
        .iter()
        .filter(|k| k.exit != SolveExit::Converged && k.r <= r_e.r)
        .count();
    println!("   non-converged knots at r <= R_e   {bad_in_well}");
    println!("   knots where BOTH runs failed      {unresolved} (unresolved: not scored)");
    println!();

    // --- THE SCORE -------------------------------------------------------------------
    println!("   worst |dE| over scored knots  {:.3e} Ha at r = {:.4} bohr", worst_de.0, worst_de.1);
    println!("   worst |dF|                    {:.3e} Ha/bohr at r = {:.4} bohr", worst_df.0, worst_df.1);
    println!(
        "   as a fraction of kT(300 K)    {:.3e}      of kT(3000 K)  {:.3e}",
        worst_de.0 / (KB_HA * T_TARGET),
        worst_de.0 / (KB_HA * T_INIT)
    );
    println!("   as a fraction of D_e          {:.3e}", worst_de.0 / (e_asym - r_e.e));
    println!();
    println!("   The variational margin at the worst production knot, as a second opinion:");
    println!("   min_i H_ii - E = {:.6} Ha (positive is necessary, not sufficient)", margin(worst.r, worst.e));
}
