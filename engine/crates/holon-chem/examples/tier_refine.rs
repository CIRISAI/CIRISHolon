//! The overflow tier, exercised: solve at f64, refine on double-double, report both.
//!
//! ```text
//! cargo run --release -p holon-chem --example tier_refine -- 52 51 [Z...]
//! ```
//!
//! This is the instrument the 2026-08-30 ruling called for: a solve that stagnates at
//! the f64 expansion floor has EXHAUSTED ITS TIER, and the answer is not a moved
//! constant — it is the next tier of arithmetic, warm-started from the f64 vector so
//! double-double cost pays only for the last mile. One solver body serves both tiers
//! (`tier::davidson_eigh_from_t`); this binary is the f64→Dd ladder.
//!
//! Calibration discipline: `DD_EXPANSION_FLOOR` is provisional until the SMALL
//! formerly-refused atoms (Te 729 determinants, Sb 9,477) show where DD residuals
//! actually pin. Run those first; read the floor; only then spend hours on Zn/Sn/In.
//!
//! What the columns mean: `f64 resid` is where the old tier stopped and `f64 exit` why;
//! `dd resid` is where the new tier stopped; `delta` is the energy the refinement moved —
//! the part of the eigenvalue f64 could never resolve. A delta far below `f64 resid`
//! squared over the gap is the expected shape (the Ritz value is quadratically accurate);
//! a large delta would be a FINDING about the f64 record, printed rather than absorbed.

use holon_chem::dual::D2;
use holon_chem::elements::by_z;
use holon_chem::fci::{ci_ints, solve_determinant, Order};
use holon_chem::pair::geometry_problem;
use holon_chem::tier::refine_determinant_dd;

fn main() {
    let argv: Vec<u32> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let ask: Vec<u32> = if argv.is_empty() { vec![9, 10, 52, 51] } else { argv };

    println!("# f64 tier -> double-double tier, warm-started. One solver body, two scalars.");
    println!(
        "{:>3} {:>3} {:>9} | {:>11} {:>10} {:>5} | {:>11} {:>10} {:>5} | {:>12}",
        "Z", "sym", "n_det", "f64 resid", "f64 exit", "it", "dd resid", "dd exit", "it", "delta (Ha)"
    );

    for z in ask {
        let sp = by_z(z).expect("unknown Z");
        let (space, mo, _) =
            geometry_problem(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        let t0 = std::time::Instant::now();
        let sol = solve_determinant(&space, &mo);
        let t_f64 = t0.elapsed().as_secs_f64();

        let ci0 = ci_ints(&mo, Order::Value);
        let diag = space.diagonal(&ci0);
        let t1 = std::time::Instant::now();
        let r = refine_determinant_dd(&space, &ci0, &diag, sol.e.v, &sol.vector, holon_chem::tier::DD_REQUESTED_TOLERANCE, 4000);
        let t_dd = t1.elapsed().as_secs_f64();

        println!(
            "{:>3} {:>3} {:>9} | {:>11.2e} {:>10} {:>5} | {:>11.2e} {:>10} {:>5} | {:>12.3e}   # e_dd = {:.20e} ; f64 {:.1}s dd {:.1}s",
            sp.z,
            sp.symbol,
            space.n_det,
            sol.residual,
            match sol.exit {
                holon_chem::fci::SolveExit::Converged => "CONV",
                holon_chem::fci::SolveExit::IterationCap => "CAP",
                holon_chem::fci::SolveExit::Stagnated => "STAG",
                holon_chem::fci::SolveExit::Trivial => "TRIV",
            },
            sol.davidson_iters,
            r.residual,
            match r.exit {
                holon_chem::fci::SolveExit::Converged => "CONV",
                holon_chem::fci::SolveExit::IterationCap => "CAP",
                holon_chem::fci::SolveExit::Stagnated => "STAG",
                holon_chem::fci::SolveExit::Trivial => "TRIV",
            },
            r.iters,
            r.delta_vs_f64,
            r.e_f64,
            t_f64,
            t_dd,
        );
    }
    println!("#");
    println!(
        "# dd exit CONV means the tier delivered {} — the residual the f64 floor refused.",
        holon_chem::tier::DD_REQUESTED_TOLERANCE
    );
    println!("# dd exit STAG at a residual ABOVE that is the DD floor calibration reading.");
}
