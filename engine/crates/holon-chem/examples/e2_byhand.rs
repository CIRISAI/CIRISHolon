//! E2 and R2's by-hand pairs: a well located on the DETERMINANT route explicitly.
//!
//! ```text
//! cargo run --release -p holon-chem --example e2_byhand -- [PAIR ...]
//! ```
//!
//! # Why this exists rather than a flag on `generate_pair_table`
//!
//! `pair::feasibility` reports SiO as having no AUTOMATIC route, and
//! `generate_pair_table` refuses it — correctly, because `fci::solve` would send its
//! 132,496 determinants to an MPO builder that does not return at fourteen orbitals.
//!
//! But the determinant route reaches it easily: measured on this box, one SiO geometry is
//! **33.9 s** in f64 with a Davidson residual of 9.2e-11. So "no automatic route" was
//! being reported downstream as "owed", and it is not owed — it is a solve nobody had
//! asked for correctly.
//!
//! The right long-run fix is a route policy on `generate_pair_table`. That function and
//! `fci::solve_basis` are being edited by two other lanes right now (W1's mask widening,
//! ELEMENTS-3's radius rule), so this takes the same two-stage discipline `locate_well`
//! uses — bracket by scan, bisect on the gradient, finish with Newton — against
//! `solve_determinant` directly, and leaves the shared signatures alone.

use holon_chem::dual::D2;
use holon_chem::elements::{by_symbol, Species};
use holon_chem::fci::solve_determinant;
use holon_chem::pair::{atom_energy, feasibility, geometry_problem, WELL_MIN_DEPTH};
use std::io::Write;
use std::time::Instant;

/// Total energy and its two exact derivatives at one separation, determinant route only.
fn point(a: Species, b: Species, r: f64) -> (f64, f64, f64) {
    let (space, mo, nuc) = geometry_problem(
        &[a, b],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::var(r)],
        ],
    );
    let e = solve_determinant(&space, &mo).e + nuc;
    (e.v, e.d, e.e)
}

fn split(name: &str) -> (Species, Species) {
    if let Some(stem) = name.strip_suffix('2') {
        let sp = by_symbol(stem).unwrap();
        return (sp, sp);
    }
    let at = name
        .char_indices()
        .skip(1)
        .find(|(_, c)| c.is_uppercase())
        .map(|(i, _)| i)
        .unwrap();
    (by_symbol(&name[..at]).unwrap(), by_symbol(&name[at..]).unwrap())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let names: Vec<String> = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        vec!["SiO".to_string()]
    };
    println!("# E2/R2 by-hand pairs, determinant route (fci::solve_determinant), f64");
    println!("pair\tn_orb\tn_det\tE_asymptote\tR_e\tD_e\tk_e\tpoints\tseconds");
    let _ = std::io::stdout().flush();

    for name in names {
        let (a, b) = split(&name);
        let f = feasibility(a, b);
        let t0 = Instant::now();
        let mut evals = 0usize;

        // The asymptote is two isolated atoms at the same level of theory, exactly as
        // `generate_pair_table` computes it.
        let e_asymptote = atom_energy(a) + atom_energy(b);

        // Stage 1: a coarse scan for the bracket. Deliberately coarse — each evaluation is
        // tens of seconds, and the refinement below is what sets the precision.
        let (mut best_r, mut best_e) = (0.0f64, f64::INFINITY);
        let mut r = 1.6f64;
        while r <= 6.01 {
            let (e, _, _) = point(a, b, r);
            evals += 1;
            if e < best_e {
                best_e = e;
                best_r = r;
            }
            r += 0.2;
        }
        if e_asymptote - best_e <= WELL_MIN_DEPTH {
            println!("{name}\t{}\t{}\tUNBOUND (no minimum deeper than {WELL_MIN_DEPTH:.0e} Ha)",
                f.n_orb(), f.n_det());
            continue;
        }

        // Stage 2: bisect the GRADIENT, so the answer is the root of dE/dR and not the
        // smallest sampled point.
        let (mut lo, mut hi) = (best_r - 0.2, best_r + 0.2);
        for _ in 0..14 {
            let mid = 0.5 * (lo + hi);
            let (_, d, _) = point(a, b, mid);
            evals += 1;
            if d < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
            if hi - lo < 1e-6 {
                break;
            }
        }
        // Stage 3: Newton with the exact second derivative, which lands on the last bits.
        let mut x = 0.5 * (lo + hi);
        for _ in 0..4 {
            let (_, d, e2) = point(a, b, x);
            evals += 1;
            if e2 == 0.0 {
                break;
            }
            let step = d / e2;
            x -= step;
            if step.abs() <= 1e-12 * x.abs() {
                break;
            }
        }
        let (e_min, _, k_e) = point(a, b, x);
        evals += 1;
        println!(
            "{name}\t{}\t{}\t{e_asymptote:.12}\t{x:.9}\t{:.9}\t{k_e:.9}\t{evals}\t{:.1}",
            f.n_orb(),
            f.n_det(),
            e_asymptote - e_min,
            t0.elapsed().as_secs_f64()
        );
        let _ = std::io::stdout().flush();
    }
}
