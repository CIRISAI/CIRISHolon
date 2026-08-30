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
use std::io::Write;
use std::time::Instant;

/// Print a line and FLUSH it.
///
/// Rust's stdout is line-buffered on a terminal and BLOCK-buffered on a pipe or a file.
/// This harness is meant to be run detached with its output redirected, and a run that
/// buffers is a run whose progress cannot be watched and whose partial results are lost if
/// it is killed — which is exactly what happened to its first invocation. Every line here
/// goes through this.
macro_rules! say {
    ($($t:tt)*) => {{
        println!($($t)*);
        let _ = std::io::stdout().flush();
    }};
}

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

/// The ladder actually used, so a cost survey can walk it without editing the file.
/// `D1_CHI=4,8,16` overrides; absent, [`CHI_LADDER`] stands. The value in force is printed
/// in every header, because a harness whose parameters can be overridden and are not echoed
/// is a harness that means two different things in two different runs.
fn chi_ladder() -> Vec<usize> {
    match std::env::var("D1_CHI") {
        Ok(v) => v.split(',').filter_map(|t| t.trim().parse().ok()).collect(),
        Err(_) => CHI_LADDER.to_vec(),
    }
}

/// Sweep budget in force, `D1_SWEEPS` overriding [`DMRG_SWEEPS`]. Echoed, same rule.
fn sweeps() -> usize {
    std::env::var("D1_SWEEPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DMRG_SWEEPS)
}

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
        "cost" => cost(a, b, &name),
        "overlap" => {
            let n: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(16);
            overlap(a, b, &name, n);
        }
        "curve" => {
            let n: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(24);
            curve(a, b, &name, n);
        }
        other => panic!("unknown mode {other}; expected cost | probe | overlap | curve"),
    }
}

/// Where the DMRG side's time actually goes: MPO construction, then one sweep.
///
/// Split out because the two scale differently and the schedule depends on which
/// dominates. The MPO for a general two-body electronic Hamiltonian is built here from a
/// raw list of `O(n_orb^4)` operator strings; the sweep cost is set by the MPO's bond
/// dimension and by chi. Guessing which one is the wall would have set the whole D1
/// schedule on a guess.
fn cost(a: Species, b: Species, name: &str) {
    let (space, mo, _nuc) = problem_at(a, b, 3.0);
    say!(
        "{name}  n_orb {}  sites {}  n_det {}",
        mo.n,
        2 * mo.n,
        space.n_det
    );
    let h: Vec<f64> = mo.h.iter().map(|d| d.v).collect();
    let g: Vec<f64> = mo.g.iter().map(|d| d.v).collect();
    let t = Instant::now();
    let mpo = q8_mps::mpo::Mpo::from_electronic_integrals(mo.n, &h, &g);
    say!("  MPO build           {:.2} s", t.elapsed().as_secs_f64());
    let bd = mpo.bond_dims();
    say!(
        "  MPO bond dims       max {}  mean {:.0}",
        bd.iter().copied().max().unwrap_or(0),
        bd.iter().sum::<usize>() as f64 / bd.len().max(1) as f64
    );
    for &chi in chi_ladder().iter() {
        let init = q8_mps::mps::initial_state_hf(mo.n, space.alpha.n_elec, space.beta.n_elec);
        let cfg = q8_mps::dmrg::DmrgConfig {
            chi_max: chi,
            max_sweeps: 1,
            sweep_tol: DMRG_TOL,
            policy: q8_mps::dmrg::RefusalPolicy::Silent,
        };
        let t1 = Instant::now();
        match q8_mps::dmrg::dmrg_sweep(&mpo, init, &cfg) {
            Ok(r) => say!(
                "  one sweep chi={chi:<4} {:.2} s   E = {:.12}",
                t1.elapsed().as_secs_f64(),
                r.energy
            ),
            Err(e) => say!("  one sweep chi={chi:<4} REFUSED after {:.2} s: {e:?}", t1.elapsed().as_secs_f64()),
        }
    }
}

/// One geometry, both routes, timed. What the schedule for the full grid is derived from.
fn probe(a: Species, b: Species, name: &str) {
    let r = 3.0;
    let t_asm = Instant::now();
    let (space, mo, nuc) = problem_at(a, b, r);
    say!("  assembly {:.2} s", t_asm.elapsed().as_secs_f64());
    say!(
        "{name}  n_basis {}  n_det {}  n_alpha {}  n_beta {}",
        mo.n, space.n_det, space.alpha.n_elec, space.beta.n_elec
    );

    let t0 = Instant::now();
    let ex = solve_determinant(&space, &mo);
    let t_exact = t0.elapsed().as_secs_f64();
    say!(
        "  determinant  E = {:.15}  residual {:.2e}  iters {}  {:.2} s",
        (ex.e + nuc).v,
        ex.residual,
        ex.davidson_iters,
        t_exact
    );

    say!("  ladder {:?}  sweeps {}  tol {DMRG_TOL:.0e}", chi_ladder(), sweeps());
    for &chi in chi_ladder().iter() {
        say!("  DMRG chi={chi} starting...");
        let t1 = Instant::now();
        let d = solve_mps_with(&space, &mo, chi, sweeps(), DMRG_TOL);
        let t = t1.elapsed().as_secs_f64();
        say!(
            "  DMRG chi={chi:<4}  E = {:.15}  dE_vs_exact {:+.3e}  dw/res {:.2e}  {:.2} s",
            (d.e + nuc).v,
            (d.e + nuc).v - (ex.e + nuc).v,
            d.residual,
            t
        );
    }
}

/// D1's overlap comparison on one species' declared grid.
///
/// # The MPO is built ONCE per geometry
///
/// Measured on this machine (`output/mixtures1/mpo_cost_*.log`): at six orbitals the
/// electronic MPO costs 528 seconds to construct and 0.03 seconds to sweep. Construction
/// is the entire budget. `solve_mps` rebuilds the MPO on every call, so walking a
/// four-rung chi ladder through it would pay that 528 seconds four times over for a
/// quantity — the bond dimension's effect on the energy — that does not depend on it. So
/// the ladder is walked against one MPO here.
fn overlap(a: Species, b: Species, name: &str, n_points: usize) {
    let t_start = Instant::now();
    let e_asymptote = holon_chem::pair::atom_energy(a) + holon_chem::pair::atom_energy(b);
    let (r_min, r_max) = exact_range(a, b, e_asymptote);
    let ladder = chi_ladder();
    say!("# D1 overlap: {name}");
    say!("#   grid rule      uniform in R^-1/4 (table::grid_point), {n_points} knots");
    say!("#   range (exact)  [{r_min:.9}, {r_max:.9}] bohr");
    say!("#   asymptote      {e_asymptote:.15} hartree");
    say!("#   stake          |E_dmrg - E_fci| <= {D1_STAKE:.0e} Ha at chi = {}", ladder[ladder.len() - 1]);
    say!("#   ladder {ladder:?}  sweeps {}  tol {DMRG_TOL:.0e}", sweeps());
    let mut header = String::from("R_bohr\tE_fci\tres_fci\tn_det");
    for chi in ladder.iter() {
        header.push_str(&format!("\tE_chi{chi}\td_chi{chi}"));
    }
    say!("{header}");

    let mut worst = 0.0f64;
    let mut worst_r = 0.0f64;
    for i in 0..n_points {
        let r = grid_point(r_min, r_max, n_points, i);
        let (space, mo, nuc) = problem_at(a, b, r);
        let ex = solve_determinant(&space, &mo);
        let e_fci = (ex.e + nuc).v;

        let h: Vec<f64> = mo.h.iter().map(|d| d.v).collect();
        let g: Vec<f64> = mo.g.iter().map(|d| d.v).collect();
        let t_mpo = Instant::now();
        let mpo = q8_mps::mpo::Mpo::from_electronic_integrals(mo.n, &h, &g);
        let mpo_s = t_mpo.elapsed().as_secs_f64();

        let mut row = format!(
            "{r:.9}\t{e_fci:.15}\t{:.3e}\t{}",
            ex.residual, space.n_det
        );
        let mut last_delta = f64::INFINITY;
        for &chi in ladder.iter() {
            let init = q8_mps::mps::initial_state_hf(mo.n, space.alpha.n_elec, space.beta.n_elec);
            let cfg = q8_mps::dmrg::DmrgConfig {
                chi_max: chi,
                max_sweeps: sweeps(),
                sweep_tol: DMRG_TOL,
                policy: q8_mps::dmrg::RefusalPolicy::Silent,
            };
            match q8_mps::dmrg::dmrg_sweep(&mpo, init, &cfg) {
                Ok(res) => {
                    let e = res.energy + nuc.v;
                    last_delta = e - e_fci;
                    row.push_str(&format!("\t{e:.15}\t{:+.3e}", last_delta));
                }
                Err(e) => {
                    row.push_str(&format!("\tREFUSED\t{e:?}"));
                    last_delta = f64::INFINITY;
                }
            }
        }
        row.push_str(&format!("\tmpo_s {mpo_s:.1}"));
        say!("{row}");
        if last_delta.abs() > worst {
            worst = last_delta.abs();
            worst_r = r;
        }
    }
    let verdict = if worst <= D1_STAKE { "PASS" } else { "FAIL" };
    say!("# WORST |delta| at largest chi = {worst:.6e} Ha at R = {worst_r:.6} bohr   stake {D1_STAKE:.0e}   {verdict}");
    say!("# elapsed {:.1} s", t_start.elapsed().as_secs_f64());
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
    say!("# D1 DMRG-only curve: {name}");
    say!("#   route          DMRG (MPS), chi ladder {CHI_LADDER:?}, sweeps {DMRG_SWEEPS}, tol {DMRG_TOL:.0e}");
    say!("#   grid rule      uniform in R^-1/4 (table::grid_point), {n_knots} knots");
    say!("#   range          [{r_min:.9}, {r_max:.9}] bohr   (DMRG-derived; no exact route exists here)");
    say!("#   asymptote      {e_asymptote:.15} hartree   (DMRG atoms)");
    say!("#   uncertainty    max(|E(chi_prev) - E(chi_max)|, discarded weight) per knot");
    say!("R_bohr\tE_dmrg\tu_conv\tdw\tE_chi16\tE_chi32\tE_chi64\tE_chi128\tn_det");

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
        say!("\t{}", space.n_det);
    }
    say!("# WORST convergence-derived uncertainty = {worst_u:.6e} Ha");
    say!("# elapsed {:.1} s", t_start.elapsed().as_secs_f64());
}
