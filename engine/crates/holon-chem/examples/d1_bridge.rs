//! MIXTURES-1 gate D1, engine half: the DMRG bridge earns its admission, or does not.
//!
//! ```text
//! cargo run -p holon-chem --release --example d1_bridge -- probe   <PAIR>
//! cargo run -p holon-chem --release --example d1_bridge -- overlap <PAIR> [n_points]
//! cargo run -p holon-chem --release --example d1_bridge -- curve   <PAIR> [n_knots]
//! ```
//!
//! # What D1 stakes
//!
//! > q8-mps ground energies match exact FCI on at least two overlap species where BOTH
//! > are feasible (staked: S2 and SiO) at <= 1e-8 Ha across their grids; only then may
//! > DMRG-only curves (staked: Si2, Na2) enter the sandbox, each labelled DMRG with its
//! > own convergence-derived uncertainty, never presented as exact.
//!
//! # The comparison is of SOLVERS, not of pipelines
//!
//! Both routes are driven from ONE call to [`geometry_problem`]: one basis assembly, one
//! orthogonalisation, one orbital rotation, one integral transform, and then two
//! eigensolvers over the identical `MoIntegrals`. Assembling twice would put every
//! upstream stage inside the residual and report integral noise as bridge error.
//!
//! # Why the exact side calls `solve_determinant` and never `solve`
//!
//! [`holon_chem::fci::solve`] switches to DMRG past
//! [`holon_chem::fci::MPS_ROUTE_THRESHOLD`] determinants. SiO is 132,496 determinants —
//! past it. A validation whose REFERENCE is silently the thing under test measures the
//! solver against itself and passes at 0.0 no matter how wrong both are. That is not a
//! hypothetical: it is what this harness would have done had it used the ordinary entry
//! point, and it is the same shape as the provenance hole this campaign is fixing
//! downstream.
//!
//! # The grid is the engine's own declared rule, not a choice made here
//!
//! Range from [`derive_range`] (inner end where the repulsion reaches the declared
//! `WALL_CEILING` above the asymptote, outer end where the interaction falls inside the
//! declared `TAIL_TOLERANCE`), knots from [`grid_point`] (uniform in `R^-1/4`, the
//! spacing that equidistributes the cubic Hermite error). Both are the rules every other
//! curve in this crate is built on, so the grid cannot have been picked to flatter the
//! bridge — and the range search runs on the DETERMINANT route for exactly the reason
//! above.

use holon_chem::dual::D2;
use holon_chem::elements::{by_symbol, Species};
use holon_chem::fci::{solve_determinant, solve_mps_with, FciSpace, MoIntegrals, Solution};
use holon_chem::pair::{derive_range, geometry_problem, WALL_CEILING, TAIL_TOLERANCE, R_SEARCH_MAX};
use holon_chem::table::grid_point;
use std::time::Instant;

/// The bond dimensions the DMRG side is run at, in order.
///
/// Two purposes, and they are separate. The comparison itself uses the LARGEST. The
/// smaller ones are what makes the uncertainty CONVERGENCE-DERIVED rather than declared:
/// D1 requires a DMRG-only curve to carry "its own convergence-derived uncertainty", and
/// the honest measure of that is how far the energy is still moving as the budget grows —
/// `|E(chi_k) - E(chi_max)|` at the last step — taken together with the discarded weight
/// the solver reports. A single run at one bond dimension can report a discarded weight
/// but cannot report whether it has stopped moving.
const CHI_LADDER: [usize; 4] = [16, 32, 64, 128];

/// The staked agreement, hartree. From the freeze; not a tolerance that moves.
const D1_STAKE: f64 = 1e-8;

/// Sweep budget and per-sweep tolerance for the DMRG side. Declared here rather than
/// taken from `solve_mps`'s defaults so the run is reproducible from this file alone.
const DMRG_SWEEPS: usize = 40;
const DMRG_TOL: f64 = 1e-11;

fn pair_of(name: &str) -> (Species, Species) {
    // Split a formula like "SiO" / "S2" / "Na2" into its two species.
    if let Some(stem) = name.strip_suffix('2') {
        let sp = by_symbol(stem).unwrap_or_else(|| panic!("unknown element {stem}"));
        return (sp, sp);
    }
    // Two symbols, the second starting at the second uppercase letter.
    let split = name
        .char_indices()
        .skip(1)
        .find(|(_, c)| c.is_uppercase())
        .map(|(i, _)| i)
        .unwrap_or_else(|| panic!("cannot split {name} into two element symbols"));
    let a = by_symbol(&name[..split]).unwrap_or_else(|| panic!("unknown element {}", &name[..split]));
    let b = by_symbol(&name[split..]).unwrap_or_else(|| panic!("unknown element {}", &name[split..]));
    (a, b)
}

fn problem_at(a: Species, b: Species, r: f64) -> (FciSpace, MoIntegrals, D2) {
    geometry_problem(
        &[a, b],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::c(r)],
        ],
    )
}

/// Exact-in-model total energy at one separation, on the determinant route only.
fn exact_total(a: Species, b: Species, r: f64) -> (f64, Solution, usize) {
    let (space, mo, nuc) = problem_at(a, b, r);
    let sol = solve_determinant(&space, &mo);
    let n_det = space.n_det;
    ((sol.e + nuc).v, sol, n_det)
}

/// The range search, run entirely on the determinant route.
///
/// `derive_range` calls `solve_geometry`, which auto-routes — so on SiO the ENGINE's own
/// range search goes through DMRG. For the gate's declared grid that is not acceptable
/// (the grid would depend on the thing being validated), so the search is repeated here
/// against the exact route. The two are printed together: a disagreement is itself a
/// reading about the bridge, at the coarsest possible resolution, and it is reported
/// rather than resolved.
fn exact_range(a: Species, b: Species, e_asymptote: f64) -> (f64, f64) {
    let u = |r: f64| exact_total(a, b, r).0 - e_asymptote;
    let mut inner_hi = 2.0f64;
    while u(inner_hi) > WALL_CEILING && inner_hi < 6.0 {
        inner_hi += 0.5;
    }
    let mut inner_lo = 0.2f64;
    if u(inner_lo) <= WALL_CEILING {
        return (inner_lo, R_SEARCH_MAX.min(12.0));
    }
    let mut lo = inner_lo;
    let mut hi = inner_hi;
    let flo = u(lo) - WALL_CEILING;
    if !(flo.is_finite()) || flo * (u(hi) - WALL_CEILING) > 0.0 {
        return (inner_lo, R_SEARCH_MAX.min(12.0));
    }
    for _ in 0..24 {
        let mid = 0.5 * (lo + hi);
        if (u(mid) - WALL_CEILING) * flo > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    inner_lo = 0.5 * (lo + hi);

    let mut r_max = R_SEARCH_MAX;
    let mut probe = R_SEARCH_MAX;
    while probe > inner_lo * 2.0 {
        if u(probe).abs() >= TAIL_TOLERANCE {
            break;
        }
        r_max = probe;
        probe -= 0.5;
    }
    (inner_lo, r_max)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("probe");
    let name = args.get(2).cloned().unwrap_or_else(|| "S2".to_string());
    let (a, b) = pair_of(&name);

    match mode {
        "probe" => probe(a, b, &name),
        "overlap" => {
            let n: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(16);
            overlap(a, b, &name, n);
        }
        "curve" => {
            let n: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(24);
            curve(a, b, &name, n);
        }
        other => panic!("unknown mode {other}; expected probe | overlap | curve"),
    }
}

/// One geometry, both routes, timed. What the schedule for the full grid is derived from.
fn probe(a: Species, b: Species, name: &str) {
    let r = 3.0;
    let (space, mo, nuc) = problem_at(a, b, r);
    println!(
        "{name}  n_basis {}  n_det {}  n_alpha {}  n_beta {}",
        mo.n, space.n_det, space.alpha.n_elec, space.beta.n_elec
    );

    let t0 = Instant::now();
    let ex = solve_determinant(&space, &mo);
    let t_exact = t0.elapsed().as_secs_f64();
    println!(
        "  determinant  E = {:.15}  residual {:.2e}  iters {}  {:.2} s",
        (ex.e + nuc).v,
        ex.residual,
        ex.davidson_iters,
        t_exact
    );

    for &chi in CHI_LADDER.iter() {
        let t1 = Instant::now();
        let d = solve_mps_with(&space, &mo, chi, DMRG_SWEEPS, DMRG_TOL);
        let t = t1.elapsed().as_secs_f64();
        println!(
            "  DMRG chi={chi:<4}  E = {:.15}  dE_vs_exact {:+.3e}  dw/res {:.2e}  {:.2} s",
            (d.e + nuc).v,
            (d.e + nuc).v - (ex.e + nuc).v,
            d.residual,
            t
        );
    }
}

/// D1's overlap comparison on one species' declared grid.
fn overlap(a: Species, b: Species, name: &str, n_points: usize) {
    let t_start = Instant::now();
    let e_asymptote = exact_total(a, b, 60.0).0.min(f64::INFINITY);
    // The asymptote the engine itself uses is the sum of two ISOLATED atom energies, not
    // a far-separation dimer point. Use that one, so the grid rule here is the engine's.
    let e_asymptote = {
        let ea = holon_chem::pair::atom_energy(a);
        let eb = holon_chem::pair::atom_energy(b);
        let _ = e_asymptote;
        ea + eb
    };
    let (r_min, r_max) = exact_range(a, b, e_asymptote);
    let (er_min, er_max) = derive_range(a, b, e_asymptote);
    println!("# D1 overlap: {name}");
    println!("#   grid rule      uniform in R^-1/4 (table::grid_point), {n_points} knots");
    println!("#   range (exact)  [{r_min:.9}, {r_max:.9}] bohr");
    println!("#   range (engine) [{er_min:.9}, {er_max:.9}] bohr   <- engine's auto-routed search");
    println!("#   asymptote      {e_asymptote:.15} hartree");
    println!("#   stake          |E_dmrg - E_fci| <= {D1_STAKE:.0e} Ha at chi = {}", CHI_LADDER[CHI_LADDER.len() - 1]);
    println!("#   sweeps {DMRG_SWEEPS}  tol {DMRG_TOL:.0e}");
    println!("R_bohr\tE_fci\tE_dmrg\tdelta\tdw_dmrg\tres_fci\tn_det");

    let chi = CHI_LADDER[CHI_LADDER.len() - 1];
    let mut worst = 0.0f64;
    let mut worst_r = 0.0f64;
    for i in 0..n_points {
        let r = grid_point(r_min, r_max, n_points, i);
        let (space, mo, nuc) = problem_at(a, b, r);
        let ex = solve_determinant(&space, &mo);
        let dm = solve_mps_with(&space, &mo, chi, DMRG_SWEEPS, DMRG_TOL);
        let e_fci = (ex.e + nuc).v;
        let e_dmrg = (dm.e + nuc).v;
        let d = (e_dmrg - e_fci).abs();
        if d > worst {
            worst = d;
            worst_r = r;
        }
        println!(
            "{r:.9}\t{e_fci:.15}\t{e_dmrg:.15}\t{:+.6e}\t{:.3e}\t{:.3e}\t{}",
            e_dmrg - e_fci,
            dm.residual,
            ex.residual,
            space.n_det
        );
    }
    let verdict = if worst <= D1_STAKE { "PASS" } else { "FAIL" };
    println!("# WORST |delta| = {worst:.6e} Ha at R = {worst_r:.6} bohr   stake {D1_STAKE:.0e}   {verdict}");
    println!("# elapsed {:.1} s", t_start.elapsed().as_secs_f64());
}

/// A DMRG-only curve with a convergence-derived uncertainty per knot.
fn curve(a: Species, b: Species, name: &str, n_knots: usize) {
    let t_start = Instant::now();
    let e_asymptote = holon_chem::pair::atom_energy(a) + holon_chem::pair::atom_energy(b);
    // The range search cannot run on the determinant route here — that is the whole point
    // of a DMRG-only species — so it runs on the engine's own auto-routing search, which
    // for these species IS the DMRG route. Stated rather than hidden: the grid of a
    // DMRG-only curve is itself DMRG-derived.
    let (r_min, r_max) = derive_range(a, b, e_asymptote);
    println!("# D1 DMRG-only curve: {name}");
    println!("#   route          DMRG (MPS), chi ladder {CHI_LADDER:?}, sweeps {DMRG_SWEEPS}, tol {DMRG_TOL:.0e}");
    println!("#   grid rule      uniform in R^-1/4 (table::grid_point), {n_knots} knots");
    println!("#   range          [{r_min:.9}, {r_max:.9}] bohr   (DMRG-derived; no exact route exists here)");
    println!("#   asymptote      {e_asymptote:.15} hartree   (DMRG atoms)");
    println!("#   uncertainty    max(|E(chi_prev) - E(chi_max)|, discarded weight) per knot");
    println!("R_bohr\tE_dmrg\tu_conv\tdw\tE_chi16\tE_chi32\tE_chi64\tE_chi128\tn_det");

    let mut worst_u = 0.0f64;
    for i in 0..n_knots {
        let r = grid_point(r_min, r_max, n_knots, i);
        let (space, mo, nuc) = problem_at(a, b, r);
        let mut es = Vec::new();
        let mut dw = 0.0f64;
        for &chi in CHI_LADDER.iter() {
            let d = solve_mps_with(&space, &mo, chi, DMRG_SWEEPS, DMRG_TOL);
            es.push((d.e + nuc).v);
            dw = d.residual;
        }
        let e_final = *es.last().unwrap();
        let step = (es[es.len() - 2] - e_final).abs();
        let u = step.max(dw);
        worst_u = worst_u.max(u);
        print!("{r:.9}\t{e_final:.15}\t{u:.6e}\t{dw:.3e}");
        for e in es.iter() {
            print!("\t{e:.15}");
        }
        println!("\t{}", space.n_det);
    }
    println!("# WORST convergence-derived uncertainty = {worst_u:.6e} Ha");
    println!("# elapsed {:.1} s", t_start.elapsed().as_secs_f64());
}
