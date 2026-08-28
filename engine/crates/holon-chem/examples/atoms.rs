//! Atomic ground-state energies for the first row, both routes, with timings.
use holon_chem::elements::FIRST_ROW;
use holon_chem::dual::D2;
use holon_chem::pair::{electron_counts, solve_geometry};
use std::time::Instant;

fn main() {
    println!("{:>3} {:>3} {:>5} {:>5} {:>6} {:>22} {:>6} {:>5} {:>10} {:>10} {:>6}",
             "sym","Z","nbas","ndet","na/nb","E_FCI (hartree)","dav","cg","resid","cg_resid","ms");
    for sp in FIRST_ROW {
        let t0 = Instant::now();
        let s = solve_geometry(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let (_, na, nb) = electron_counts(&[sp]);
        println!("{:>3} {:>3} {:>5} {:>5} {:>5} {:>22.12} {:>6} {:>5} {:>10.2e} {:>10.2e} {:>6.1}",
                 sp.symbol, sp.z, s.n_basis, s.n_det, format!("{na}/{nb}"), s.e.v,
                 s.davidson_iters, s.cg_iters, s.residual, s.cg_residual, ms);
    }
}
