//! R1: every atom's ground state, the route that produced it, and an honest refusal where
//! no route reaches.
//!
//! # What "dual route" can and cannot mean here
//!
//! R1 asks for two independent routes at working precision. The crate has three ways to
//! apply the Hamiltonian -- the Knowles-Handy string factorisation, an independent
//! Slater-Condon enumeration, and a raw ladder-operator construction -- and the second is a
//! genuine second route: it shares no loop, no intermediate and no factorisation with the
//! first, so agreement is evidence about the factorisation rather than about one shared
//! rewriting. It costs `O(N_det^2)`, which is what bounds where it can be run.
//!
//! So this reports, per atom, which of those was affordable, and refuses rather than
//! guesses where none was. A refusal names the reason and the size that caused it.
//!
//! Usage: `cargo run --release -p holon-chem --example elements3_atoms [max_n_det] [max_dual_n_det]`

use holon_chem::dual::D2;
use holon_chem::elements::ALL_ELEMENTS;
use holon_chem::fci::{ci_ints, s_squared, solve_determinant, Order};
use holon_chem::pair::{electron_counts, geometry_problem, CONVERGED_RESIDUAL};

/// Determinant count this record is willing to SPEND on one atom. DECLARED, and a budget
/// rather than a limit: Davidson holds a subspace of order twenty vectors, so a million
/// determinants is about 160 MB and a run that finishes, while ten million is hours on a
/// shared machine. Atoms above it are reported as over budget, which is a different fact
/// from having no route, and the record says which.
const DEFAULT_MAX_N_DET: usize = 1_200_000;

/// Where the PRODUCTION path gives out, as opposed to this record's budget.
///
/// `fci::solve` sends anything past `MPS_ROUTE_THRESHOLD` determinants to the MPS/DMRG
/// route, and that route is measured to reach six orbitals (`pair::MPS_MAX_ORBITALS`: LiH
/// at six took 528 s to build its MPO and HCl at ten never finished). Every atom here is
/// well past six orbitals, so for anything over the threshold the production entry point
/// has NO route, and this record reaches it only through `solve_determinant`, which has no
/// threshold. That deserves a column rather than a footnote.
const PRODUCTION_LIMIT: usize = holon_chem::fci::MPS_ROUTE_THRESHOLD;

/// Determinant count past which the `O(N^2)` reference route is not affordable. DECLARED.
const DEFAULT_MAX_DUAL_N_DET: usize = 30_000;

fn main() {
    let mut a = std::env::args().skip(1);
    let max_n_det: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_MAX_N_DET);
    let max_dual: usize = a
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_DUAL_N_DET);
    println!("# max_n_det = {max_n_det} (budget), max_dual_n_det = {max_dual}");
    println!("# route 'det (forced)': the space is past fci::MPS_ROUTE_THRESHOLD ({PRODUCTION_LIMIT}),");
    println!("# so the PRODUCTION entry point fci::solve would have sent it to DMRG -- which is");
    println!("# measured to reach six orbitals and cannot do any atom here. Reached instead via");
    println!("# solve_determinant, which has no threshold. 'OVER BUDGET' is this record's");
    println!("# spending cap, NOT a statement that no route exists.");
    println!(
        "# A row reading NOT CONVERGED hit the Davidson iteration cap: its energy and its"
    );
    println!(
        "# multiplicity are both meaningless and it is refused rather than reported. The bar"
    );
    println!("# is pair::CONVERGED_RESIDUAL ({CONVERGED_RESIDUAL:.0e}), and 2S+1 must be an integer.");
    println!(
        "{:>3} {:>3} {:>5} {:>14} {:>22} {:>10} {:>6} {:>11} {:>13}",
        "Z", "sym", "nbas", "n_det", "E (hartree)", "resid", "2S+1", "dual route", "route"
    );

    for sp in ALL_ELEMENTS {
        let (_, na, nb) = electron_counts(&[sp]);
        let n_basis = sp.n_basis();
        let n_det = choose(n_basis, na).saturating_mul(choose(n_basis, nb));
        if n_det > max_n_det {
            println!(
                "{:>3} {:>3} {:>5} {:>14} {:>22} {:>10} {:>6} {:>11} {:>13}",
                sp.z, sp.symbol, n_basis, n_det, "OVER BUDGET", "-", "-", "-", "none"
            );
            continue;
        }

        let (space, mo, _nuc) = geometry_problem(
            &[sp],
            vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]],
        );
        let sol = solve_determinant(&space, &mo);

        // THE VERDICT, not just the number. `CONVERGED_RESIDUAL` is the crate's declared
        // bar and its doc comment describes precisely the failure this line prevents: a
        // solve that hit its iteration cap emitted "looking perfectly healthy, carrying a
        // wrong energy, with the evidence sitting in a field no consumer is required to
        // read." This record printed the residual and checked nothing, and indium came back
        // at 3.98e-1 -- nine orders above the bar -- in a row otherwise indistinguishable
        // from a measurement.
        let converged = sol.residual <= CONVERGED_RESIDUAL;

        // <S^2> is DERIVED from the converged vector, never assumed from a term symbol.
        let s2 = s_squared(&space, &sol.vector);
        // S(S+1) = s2  =>  S = (-1 + sqrt(1 + 4 s2)) / 2, and 2S+1 follows.
        let s = 0.5 * ((1.0 + 4.0 * s2).max(0.0).sqrt() - 1.0);
        let mult = 2.0 * s + 1.0;
        // A SECOND, independent tell, and it costs nothing. The Hamiltonian is spin-free,
        // so a converged eigenvector is a spin eigenstate and 2S+1 is an INTEGER. Indium
        // returned 4.216, which is not a multiplicity at all -- it says "not an eigenstate"
        // without reference to any residual, and would catch the same failure for a reader
        // who never looks at the residual column.
        let integral = (mult - mult.round()).abs() < 1e-6;

        // The second route: apply H to a fixed probe vector both ways and compare. This is
        // a matrix-VECTOR comparison, which sees a disagreement an eigenvalue cannot --
        // two Hamiltonians differing by a diagonal sign conjugation have identical spectra.
        let dual = if space.n_det <= max_dual {
            let ci = ci_ints(&mo, Order::Value);
            let c = probe(space.n_det, 0x9E37_79B9_7F4A_7C15);
            let mut x = vec![0.0; space.n_det];
            let mut y = vec![0.0; space.n_det];
            space.sigma(&ci, &c, &mut x);
            space.sigma_reference(&ci, &c, &mut y);
            let num: f64 = x
                .iter()
                .zip(y.iter())
                .map(|(p, q)| (p - q).abs())
                .fold(0.0, f64::max);
            let den: f64 = x.iter().map(|p| p.abs()).fold(0.0, f64::max).max(1e-300);
            format!("{:.2e}", num / den)
        } else {
            "too large".to_string()
        };

        if !converged || !integral {
            // Refused rather than printed as a row. A non-converged energy in a column of
            // converged ones is a dead result presenting as a live one.
            println!(
                "{:>3} {:>3} {:>5} {:>14} {:>22} {:>10.2e} {:>6.3} {:>11} {:>13}",
                sp.z,
                sp.symbol,
                n_basis,
                space.n_det,
                "NOT CONVERGED",
                sol.residual,
                mult,
                if integral { "-" } else { "2S+1 not integral" },
                "refused"
            );
            continue;
        }
        println!(
            "{:>3} {:>3} {:>5} {:>14} {:>22.12} {:>10.2e} {:>6.3} {:>11} {:>13}",
            sp.z,
            sp.symbol,
            n_basis,
            space.n_det,
            sol.e.v,
            sol.residual,
            mult,
            dual,
            if space.n_det > PRODUCTION_LIMIT {
                "det (forced)"
            } else {
                sol.route.label().split(',').next().unwrap()
            }
        );
    }
}

fn choose(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    (0..k).fold(1usize, |acc, i| {
        acc.saturating_mul(n - i) / (i + 1)
    })
}

/// A deterministic pseudo-random probe. Deterministic so a failure reproduces;
/// pseudo-random so the comparison is not made on a vector both routes happen to treat
/// alike -- a unit vector tests one column.
fn probe(n: usize, seed: u64) -> Vec<f64> {
    let mut s = seed | 1;
    (0..n)
        .map(|_| {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((s >> 33) as f64 / (1u64 << 30) as f64) - 1.0
        })
        .collect()
}
