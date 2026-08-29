//! THE H3 EXCHANGE BARRIER, IN-MODEL — is nature's 9.6 kcal/mol emergent?
//!
//! The H + H2 exchange reaction's transition state is the collinear symmetric
//! H-H-H configuration. Benchmark surfaces (CCI/BKMP2 class) put the saddle at
//! R = 1.757 bohr with a barrier of 9.61 kcal/mol = 0.01532 Ha over H2 + H.
//! This example asks what OUR three declared inputs produce: scan the collinear
//! symmetric line E(R) = e_tot([0, R, 2R]), find its minimum (the saddle of the
//! full surface), and subtract the asymptote E(H2 at its own r_e) + E(H).
//! Everything exact-in-model STO-3G FCI; no number below is fitted.
use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};

const R_E: f64 = 1.3886940;
const KCAL_PER_HA: f64 = 627.509474;

fn e_tot(centers: &[[f64; 3]]) -> f64 {
    let h = by_symbol("H").unwrap();
    let species = vec![h; centers.len()];
    let cs: Vec<[D2; 3]> = centers
        .iter()
        .map(|c| [D2::c(c[0]), D2::c(c[1]), D2::c(c[2])])
        .collect();
    solve_geometry(&species, cs).e.v
}

fn main() {
    let h = by_symbol("H").unwrap();
    let e_asym = pair_point(h, h, R_E).e + atom_energy(h); // H2 at r_e, plus a far H

    // Coarse-to-fine scan of the collinear symmetric line.
    let (mut best_r, mut best_e) = (0.0, f64::INFINITY);
    let mut scan = |lo: f64, hi: f64, n: usize, best_r: &mut f64, best_e: &mut f64| {
        for i in 0..=n {
            let r = lo + (hi - lo) * i as f64 / n as f64;
            let e = e_tot(&[[0.0, 0.0, 0.0], [r, 0.0, 0.0], [2.0 * r, 0.0, 0.0]]);
            if e < *best_e {
                *best_e = e;
                *best_r = r;
            }
        }
    };
    scan(1.3, 2.4, 44, &mut best_r, &mut best_e);
    scan(best_r - 0.05, best_r + 0.05, 100, &mut best_r, &mut best_e);
    scan(best_r - 0.002, best_r + 0.002, 80, &mut best_r, &mut best_e);

    let barrier = best_e - e_asym;
    println!("asymptote  E(H2 at r_e) + E(H) = {:+.9} Ha", e_asym);
    println!("saddle     R = {:.5} bohr (benchmark 1.757), E = {:+.9} Ha", best_r, best_e);
    println!(
        "barrier    {:+.6} Ha = {:+.3} kcal/mol   (nature/benchmark: 0.01532 Ha = 9.61 kcal/mol)",
        barrier,
        barrier * KCAL_PER_HA
    );
    println!("ratio      in-model / benchmark = {:.2}x", barrier / 0.01532);

    // Topology check: no bound H3 anywhere on the line — the saddle sits ABOVE
    // the asymptote, so exchange has a barrier and H3 is not a molecule.
    println!(
        "topology   saddle above asymptote: {} (H3 is a pass, not a molecule)",
        barrier > 0.0
    );
}
