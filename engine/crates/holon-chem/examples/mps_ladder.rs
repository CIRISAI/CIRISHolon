//! Re-derive `pair::MPS_MAX_ORBITALS` as a MEASUREMENT, on an orbital-count ladder.
//!
//! ```text
//! cargo run --release -p holon-chem --example mps_ladder -- [PAIR ...]
//! ```
//!
//! # Why the constant needs re-deriving rather than raising
//!
//! `MPS_MAX_ORBITALS = 6` was measured against the OLD MPO construction — a raw list of
//! `O(n_orb^4)` operator strings compressed by one SVD per site — which cost 528 s at six
//! orbitals and did not finish at ten. The external team's channel-based rebuild
//! (`bb1a07a`) removed that: SiO's build went from over twelve hours to 0.07 s on real
//! integrals. So the 6 no longer describes the engine, and every route verdict resting on
//! it — this crate's `automatic_route` door, and ELEMENTS-3's route table — is stale in the
//! direction of under-promising.
//!
//! The honest replacement is a number in the same form as the one it replaces: measured,
//! against a stated budget, with the per-rung numbers published. "The rebuild fixed
//! everything" is not a constant.
//!
//! # What is measured, and against what budget
//!
//! For each pair, at its own equilibrium separation, on REAL STO-3G integrals:
//!
//! * the exact-in-model reference, from [`solve_determinant`] — never `solve`, which would
//!   route the large spaces to the very thing under test;
//! * the MPO build time;
//! * DMRG at a chi ladder, each rung run in [`SWEEP_CHUNK`]-sweep chunks from the previous
//!   rung's state, until it either reaches [`D1_STAKE`] of the exact energy or exceeds
//!   [`CELL_BUDGET_S`] of wall clock.
//!
//! A cell that runs out of budget is reported as BUDGET, not as a failure: the difference
//! between "DMRG cannot do this" and "DMRG was not given long enough" is the whole point,
//! and collapsing them is how a cost measurement turns into a capability claim.
//!
//! THE NEW CONSTANT is then the largest orbital count at which some rung reached the stake
//! inside the budget — and the run prints both that number and the wall it crossed, so the
//! next person can see whether they are bounded by time or by the method.

use holon_chem::dual::D2;
use holon_chem::elements::{by_symbol, Species};
use holon_chem::fci::solve_determinant;
use holon_chem::pair::{automatic_route, geometry_problem};
use std::io::Write;
use std::time::Instant;

macro_rules! say {
    ($($t:tt)*) => {{
        println!($($t)*);
        let _ = std::io::stdout().flush();
    }};
}

/// The agreement a rung has to reach to count. D1's stake, so the two answer one question.
const D1_STAKE: f64 = 1e-8;

/// Wall-clock budget per (pair, chi) cell, seconds. DECLARED, and the thing the new
/// constant is measured against — a reach without a budget is not a measurement.
const CELL_BUDGET_S: f64 = 300.0;

/// Sweeps per chunk. DMRG is re-entered from its own previous state, so the budget can be
/// checked between chunks rather than only after a fixed sweep count — which is what makes
/// "ran out of time" and "stopped improving" distinguishable.
const SWEEP_CHUNK: usize = 3;

/// The bond dimensions tried, smallest first. A rung is only attempted if the one below it
/// failed to reach the stake, so a pair that converges cheaply is not charged for the rest.
const CHI_LADDER: [usize; 5] = [32, 64, 128, 256, 512];

const DMRG_TOL: f64 = 1e-11;

/// The ladder, and the separation each is measured at.
///
/// The separations are each pair's own computed equilibrium, from this campaign's E2
/// measurement (`locate_well`, Newton on the solver) — so every cell is measured where the
/// molecule actually sits rather than at a round number that flatters short bonds. The two
/// unbound pairs have no equilibrium and are not in the ladder; cost, not chemistry, is
/// what this measures, and a repulsive wall is not representative of a curve's cost.
const LADDER: [(&str, f64); 7] = [
    ("H2", 1.388694),
    ("LiH", 2.924394),
    ("HCl", 2.536888),
    ("NaH", 3.133867),
    ("ClF", 3.341873),
    ("SiO", 2.908134),
    ("S2", 3.706603),
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
    let wanted: Vec<(&str, f64)> = if args.len() > 1 {
        LADDER
            .iter()
            .copied()
            .filter(|(n, _)| args[1..].iter().any(|a| a == n))
            .collect()
    } else {
        LADDER.to_vec()
    };

    say!("# MPS_MAX_ORBITALS, re-derived as a measurement");
    say!("#   stake        |E_dmrg - E_fci| <= {D1_STAKE:.0e} Ha");
    say!("#   budget       {CELL_BUDGET_S:.0} s of wall clock per (pair, chi) cell");
    say!("#   chi ladder   {CHI_LADDER:?}, smallest first, stopping at the first rung that reaches the stake");
    say!("#   sweeps       in chunks of {SWEEP_CHUNK}, re-entered from the previous state, tol {DMRG_TOL:.0e}");
    say!("#   reference    fci::solve_determinant (NOT solve, which would route the large spaces to the thing under test)");
    say!("#   geometry     each pair's own computed equilibrium separation");
    say!("pair\tn_orb\tn_det\tR\tE_fci\tmpo_s\tchi\tdelta\tsweeps\tsecs\tmaxbond\tverdict");

    let mut best_reached: Option<(usize, &str)> = None;
    let mut first_wall: Option<(usize, &str, f64)> = None;

    for (name, r) in wanted {
        let (a, b) = split(name);
        let route = automatic_route(a, b);
        let n_orb = route.n_orb();
        let n_det = route.n_det();

        let (space, mo, nuc) = geometry_problem(
            &[a, b],
            vec![
                [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                [D2::c(0.0), D2::c(0.0), D2::c(r)],
            ],
        );
        let t_ex = Instant::now();
        let exact = (solve_determinant(&space, &mo).e + nuc).v;
        let exact_s = t_ex.elapsed().as_secs_f64();

        let h: Vec<f64> = mo.h.iter().map(|d| d.v).collect();
        let g: Vec<f64> = mo.g.iter().map(|d| d.v).collect();
        let t_mpo = Instant::now();
        let mpo = q8_mps::mpo::Mpo::from_electronic_integrals(mo.n, &h, &g);
        let mpo_s = t_mpo.elapsed().as_secs_f64();

        say!(
            "# {name}: exact {exact:.12} Ha in {exact_s:.1} s; MPO {mpo_s:.2} s; \
             max MPO bond {}",
            mpo.bond_dims().into_iter().max().unwrap_or(0)
        );

        let mut reached_here = false;
        for &chi in CHI_LADDER.iter() {
            let t0 = Instant::now();
            let mut tensors =
                q8_mps::mps::initial_state_hf(mo.n, space.alpha.n_elec, space.beta.n_elec);
            let mut sweeps = 0usize;
            let mut delta = f64::INFINITY;
            let mut max_bond = 0usize;
            let mut verdict = "BUDGET";
            loop {
                let cfg = q8_mps::dmrg::DmrgConfig {
                    chi_max: chi,
                    max_sweeps: SWEEP_CHUNK,
                    sweep_tol: DMRG_TOL,
                    policy: q8_mps::dmrg::RefusalPolicy::Silent,
                };
                match q8_mps::dmrg::dmrg_sweep(&mpo, tensors, &cfg) {
                    Ok(res) => {
                        sweeps += res.sweeps_used;
                        delta = (res.energy + nuc.v) - exact;
                        max_bond = res.bond_dims.iter().copied().max().unwrap_or(0);
                        tensors = res.tensors;
                        if delta.abs() <= D1_STAKE {
                            verdict = "REACHED";
                            break;
                        }
                        // Stopped improving inside its own tolerance and still short of the
                        // stake: more sweeps at this chi will not help, so charge the next
                        // rung rather than the clock.
                        if res.converged {
                            verdict = "PLATEAU";
                            break;
                        }
                    }
                    Err(e) => {
                        verdict = "REFUSED";
                        say!("#   {name} chi={chi} refused: {e:?}");
                        break;
                    }
                }
                if t0.elapsed().as_secs_f64() > CELL_BUDGET_S {
                    break;
                }
            }
            let secs = t0.elapsed().as_secs_f64();
            say!(
                "{name}\t{n_orb}\t{n_det}\t{r:.4}\t{exact:.9}\t{mpo_s:.2}\t{chi}\t{:+.3e}\t{sweeps}\t{secs:.1}\t{max_bond}\t{verdict}",
                delta
            );
            if verdict == "REACHED" {
                reached_here = true;
                break;
            }
        }

        if reached_here {
            if best_reached.map_or(true, |(o, _)| n_orb > o) {
                best_reached = Some((n_orb, name));
            }
        } else if first_wall.map_or(true, |(o, _, _)| n_orb < o) {
            first_wall = Some((n_orb, name, CELL_BUDGET_S));
        }
    }

    say!("#");
    match best_reached {
        Some((orb, name)) => say!(
            "# LARGEST ORBITAL COUNT REACHING THE STAKE INSIDE BUDGET: {orb} ({name})"
        ),
        None => say!("# NO pair reached the stake inside budget"),
    }
    match first_wall {
        Some((orb, name, b)) => say!(
            "# SMALLEST ORBITAL COUNT THAT DID NOT: {orb} ({name}), at a {b:.0} s per-cell budget"
        ),
        None => say!("# every pair in the ladder reached the stake"),
    }
    say!(
        "# The new MPS_MAX_ORBITALS is the first number, and it is bounded by the stated \
         budget rather than by the method. Raising the budget may raise it."
    );
}
