//! Is the O-O solve DESCENDING when it hits the iteration cap, or stuck?
//!
//! `s3_oo_reexam` scores the cost of the cap by comparing a capped solve against a
//! longer one. That comparison is only meaningful if the longer one is better, and on a
//! near-degenerate dissociation limit it might not be: a solve that is oscillating
//! between two nearly-degenerate Ritz vectors gets no closer with more iterations, and
//! "raise the cap" would then be the wrong prescription however reasonable it sounds.
//!
//! So this traces one knot — the worst on the production grid — across a ladder of caps,
//! and reports the residual and the energy at each. Three readings are possible and they
//! mean different things:
//!
//! * residual falling, energy settling — the cap is simply too low, and the fix is
//!   arithmetic the tier can deliver;
//! * residual flat, energy flat — the solve is at a floor this tier cannot pass, which
//!   is the overflow case: the next arithmetic tier's job, not this constant's;
//! * residual flat, energy MOVING — the solve is not converging to a single state at
//!   all, which would make every number at that knot meaningless rather than imprecise.
//!
//! `dE` is against the deepest run, so it reads as "how much did the shorter run cost".
//!
//! ```text
//! cargo run --release -p holon-chem --example s3_oo_trace -- [knot_index]
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::OXYGEN;
use holon_chem::fci::{ci_ints, davidson_eigh_from, Order, DAVIDSON_REQUESTED_TOLERANCE};
use holon_chem::pair::{atom_energy, derive_range, geometry_problem};
use holon_chem::table::grid_point;

/// The production curve `waterquench.rs` loads, and the index of its worst knot as
/// `s3_oo_reexam` phase A measured it.
const N_KNOTS: usize = 96;
const WORST_KNOT: usize = 45;

/// The exact knot, DERIVED rather than typed. Writing `4.2244` here — the printed value —
/// moved the geometry by 3e-5 bohr, which on this part of the curve is 3.6e-7 hartree of
/// slope: enough that the trace's converged energy and the re-examination's disagreed in
/// the seventh decimal for no reason a reader could see. Two numbers that are supposed to
/// be the same knot must come from the same expression.
fn knot_r(i: usize) -> f64 {
    let (r_min, r_max) = derive_range(OXYGEN, OXYGEN, 2.0 * atom_energy(OXYGEN));
    grid_point(r_min, r_max, N_KNOTS, i)
}

const CAPS: [usize; 8] = [150, 300, 600, 1200, 2400, 4800, 9600, 19200];

fn main() {
    // A KNOT INDEX, not a radius. Passing a typed radius was the trap this file already
    // paid for once: `4.2244` is 3e-5 bohr off knot 45, which is 3.6e-7 hartree of slope,
    // and on a near-degenerate dissociation knot it moved the iteration count from 3930 to
    // past 5000 — enough to read a BUDGET case as a FLOOR case and price a
    // high-precision tier that was never owed. An index cannot be off by 3e-5.
    let idx: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(WORST_KNOT);
    let r = knot_r(idx);

    // ONE assembly, reused by every cap: the ladder must differ in iterations and in
    // nothing else, and re-assembling per rung would put basis arithmetic in the diff.
    let (space, mo, nuc) = geometry_problem(
        &[OXYGEN, OXYGEN],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::var(r)],
        ],
    );
    let ci0 = ci_ints(&mo, Order::Value);
    let diag = space.diagonal(&ci0);
    let min_diag = diag.iter().copied().fold(f64::INFINITY, f64::min);

    // Six decimals, because four is the width `s3_oo_reexam` prints and a reader
    // comparing the two files must be able to see that this is the SAME knot rather than
    // a nearby one — the failure this file already paid for once.
    println!("# O-O convergence trace at knot {idx} of {N_KNOTS}, r = {r:.9} bohr, {} determinants", space.n_det);
    println!("# tolerance {DAVIDSON_REQUESTED_TOLERANCE:.0e}, one assembly shared by every rung");
    println!("# E is TOTAL (electronic + nuclear repulsion {:.9})", nuc.v);
    println!("# variational bound: min_i H_ii = {:.9} electronic\n", min_diag);

    let mut rows: Vec<(usize, usize, f64, f64, &'static str)> = Vec::new();
    for cap in CAPS {
        let (e, _v, iters, resid, exit) =
            davidson_eigh_from(&space, &ci0, &diag, DAVIDSON_REQUESTED_TOLERANCE, cap, None);
        rows.push((cap, iters, resid, e + nuc.v, exit.label()));
    }
    let deepest = rows.last().unwrap().3;

    println!("   {:>7} {:>7} {:>12} {:>18} {:>12} {:>15}", "cap", "iters", "residual", "E", "dE vs deepest", "exit");
    for (cap, iters, resid, e, exit) in rows.iter() {
        println!(
            "   {cap:>7} {iters:>7} {resid:>12.4e} {e:>18.9} {:>12.3e} {exit:>15}",
            e - deepest
        );
    }

    let first = rows[0];
    let last = rows[rows.len() - 1];
    println!(
        "\n   residual over the ladder: {:.3e} -> {:.3e}  ({:.1}x)",
        first.2,
        last.2,
        first.2 / last.2
    );
    println!("   energy over the ladder:   {:.3e} Ha total movement", first.3 - last.3);
}
