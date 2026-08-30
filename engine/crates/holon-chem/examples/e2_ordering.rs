//! MIXTURES-1 gate E2, engine half: the emergent chemical contrast.
//!
//! ```text
//! cargo run --release -p holon-chem --example e2_ordering -- [PAIR ...]
//! ```
//!
//! > **E2 — the emergent chemical contrast** (two-branch, structural): in-model `D_e`
//! > ordering N2 > SiO > HCl > ClF > S2 > Cl2 > NaH >> (Ar2, NeAr) in its broad strokes,
//! > numbers reported as the product; any gross inversion is branch (b), reported and
//! > investigated.
//!
//! # What is emergent about it
//!
//! Nothing here is told which pairs bind. `locate_well` looks for a minimum deeper than
//! the declared `WELL_MIN_DEPTH` and reports `None` when there is not one, so Ar2 and NeAr
//! come out unbound because in this model they are — the same code path that produces N2's
//! curve produces theirs. The ordering is a consequence of Z, the masses and the STO-3G
//! contraction, and of nothing else.
//!
//! # The two the ordinary route cannot reach
//!
//! SiO is 132,496 determinants, past `fci::MPS_ROUTE_THRESHOLD`, so `generate_pair_table`
//! refuses it rather than routing it to a DMRG builder that would not return. S2 is 23,409
//! determinants and inside the threshold but expensive. Both are reported as OWED rather
//! than guessed at, and the ordering is scored on its broad strokes over what is measured.

use holon_chem::elements::{by_symbol, Species};
use holon_chem::pair::{feasibility, generate_pair_table};
use std::io::Write;
use std::time::Instant;

/// Knots per curve. `locate_well` refines the minimum by bisection and Newton on the
/// SOLVER rather than on the interpolant, so `R_e` and `D_e` do not depend on this (the
/// P1 protocol's knot sweep measured that); it only has to be dense enough to bracket the
/// minimum.
const KNOTS: usize = 24;

/// The freeze's ordering, in the freeze's own order. The staked hypothesis, written here
/// so the measurement is compared against a list nobody edited after seeing it.
const STAKED_ORDER: [&str; 9] = [
    "N2", "SiO", "HCl", "ClF", "S2", "Cl2", "NaH", "Ar2", "NeAr",
];

fn split(name: &str) -> (Species, Species) {
    if let Some(stem) = name.strip_suffix('2') {
        let sp = by_symbol(stem).unwrap_or_else(|| panic!("unknown element {stem}"));
        return (sp, sp);
    }
    let at = name
        .char_indices()
        .skip(1)
        .find(|(_, c)| c.is_uppercase())
        .map(|(i, _)| i)
        .unwrap_or_else(|| panic!("cannot split {name}"));
    (
        by_symbol(&name[..at]).unwrap(),
        by_symbol(&name[at..]).unwrap(),
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let names: Vec<String> = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        STAKED_ORDER.iter().map(|s| s.to_string()).collect()
    };
    println!("# MIXTURES-1 E2, engine half. Staked order: {STAKED_ORDER:?}");
    println!("# knots {KNOTS}; D_e from locate_well (Newton on the solver, not on the interpolant)");
    println!("pair\tn_basis\tn_det\tD_e_Ha\tR_e_bohr\tseconds");
    let _ = std::io::stdout().flush();

    let mut measured: Vec<(String, f64)> = Vec::new();
    for name in names.iter() {
        let (a, b) = split(name);
        let f = feasibility(a, b);
        if f.is_infeasible() {
            println!(
                "{name}\t{}\t{}\tOWED\tOWED\t-\t# {} — no automatic route; the determinant \
                 route can reach it by hand, at a cost this run does not spend",
                f.n_orb(),
                f.n_det(),
                f.route_name()
            );
            let _ = std::io::stdout().flush();
            continue;
        }
        let t0 = Instant::now();
        let pt = generate_pair_table(a, b, KNOTS);
        let secs = t0.elapsed().as_secs_f64();
        match pt.meta.well {
            Some(w) => {
                println!(
                    "{name}\t{}\t{}\t{:.9}\t{:.6}\t{secs:.1}",
                    pt.meta.n_basis, pt.meta.n_det, w.d_e, w.r_e
                );
                measured.push((name.clone(), w.d_e));
            }
            None => {
                println!(
                    "{name}\t{}\t{}\tUNBOUND\t-\t{secs:.1}\t# no minimum deeper than \
                     WELL_MIN_DEPTH; nothing here knows it is a closed shell",
                    pt.meta.n_basis, pt.meta.n_det
                );
                measured.push((name.clone(), 0.0));
            }
        }
        let _ = std::io::stdout().flush();
    }

    let mut sorted = measured.clone();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    println!("# MEASURED ORDER (deepest first): {:?}", sorted.iter().map(|(n, _)| n).collect::<Vec<_>>());
    let staked_measured: Vec<&String> = STAKED_ORDER
        .iter()
        .filter_map(|s| measured.iter().find(|(n, _)| n == s).map(|(n, _)| n))
        .collect();
    println!("# STAKED ORDER, restricted to what was measured: {staked_measured:?}");
    let agree = sorted
        .iter()
        .map(|(n, _)| n)
        .eq(staked_measured.iter().copied());
    println!(
        "# The measured order {} the staked one on the pairs covered.",
        if agree { "MATCHES" } else { "DIFFERS FROM" }
    );
}
