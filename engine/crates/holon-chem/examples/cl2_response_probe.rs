//! WHERE on the Cl2 curve the response solve fails, on the SHIPPED grid itself.
//!
//! The shipped `Cl2.json` declares `worst_response_residual` = 2.06e-2 against HCl's
//! 9.96e-11 — eight orders apart. That residual belongs to the CG solve for the
//! first-order wavefunction, which feeds the SECOND derivative only
//! (`E'' = <v|H''|v> + 2<v1|H'|v>`); the energy is Davidson's and the force is
//! Hellmann–Feynman, and neither touches it. So the question is whether the CURVATURE
//! column is bad somewhere that matters — the file declares ONE uncertainty and ships
//! three columns.
//!
//! A fourteen-point hand-picked sweep found nothing worse than 9.6e-11, so this walks the
//! shipped grid exactly, which is the only sample that can contain the reported worst.
use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::pair::solve_geometry;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/atoms/tables/Cl2.json".to_string());
    let src = std::fs::read_to_string(&path).expect("cannot read the shipped table");
    let at = src.find("\"R_grid_bohr\"").expect("no R_grid_bohr");
    let open = src[at..].find('[').unwrap() + at + 1;
    let close = src[open..].find(']').unwrap() + open;
    let grid: Vec<f64> = src[open..close]
        .split(',')
        .map(|t| t.trim().parse::<f64>().expect("bad knot"))
        .collect();
    println!("# {} knots from {path}", grid.len());
    println!("{:>5} {:>10}  {:>12}  {:>13}  {:>8}", "i", "R", "cg_resid", "d2E/dR2", "cg_it");

    let cl = by_symbol("Cl").unwrap();
    let mut worst = (0usize, 0.0f64, 0.0f64);
    for (i, r) in grid.iter().copied().enumerate() {
        let sol = solve_geometry(
            &[cl, cl],
            vec![
                [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                [D2::c(0.0), D2::c(0.0), D2::var(r)],
            ],
        );
        if sol.cg_residual > 1e-9 {
            println!(
                "{i:>5} {r:>10.5}  {:>12.4e}  {:>13.6e}  {:>8}   <-- above 1e-9",
                sol.cg_residual, sol.e.e, sol.cg_iters
            );
        }
        if sol.cg_residual > worst.1 {
            worst = (i, sol.cg_residual, r);
        }
        if i % 24 == 0 {
            println!(
                "{i:>5} {r:>10.5}  {:>12.4e}  {:>13.6e}  {:>8}",
                sol.cg_residual, sol.e.e, sol.cg_iters
            );
        }
    }
    println!(
        "\nWORST: knot {} at R = {:.6} bohr, cg_residual {:.6e}",
        worst.0, worst.2, worst.1
    );
    println!("shipped file declares worst_response_residual = 2.0632080166021364e-2");
}
