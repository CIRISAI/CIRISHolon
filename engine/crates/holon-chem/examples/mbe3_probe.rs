//! MBE3 FEASIBILITY PROBE — is many-body saturation already computable in-model?
//!
//! The sandbox's force loop is pairwise-additive, so 16 H atoms condense into a
//! noble-gas-style droplet instead of eight H2 molecules. Real hydrogen saturates
//! because valence is a MANY-BODY fact: the three-body term of the many-body
//! expansion (MBE) is repulsive where a third atom approaches an existing bond.
//!
//! This probe asks the machinery we already have (N-center STO-3G FCI via
//! `solve_geometry`) the three questions the SATURATION-1 campaign would stake:
//!
//!   1. Is dE3 (the three-body interaction) REPULSIVE for compact H3?
//!   2. Does dE3 vanish when the third atom leaves (MBE consistency)?
//!   3. Do two separated dimers beat a compact H4 cluster (saturation's product)?
//!
//! Definitions (atomization-consistent):
//!   V_tot(3) = E(H3) - 3 E(H)          V2(rij) = E2(rij) - 2 E(H)
//!   dE3      = V_tot(3) - sum_pairs V2(rij)
use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};

const R_E: f64 = 1.3886940; // the banked H2 equilibrium separation, bohr

fn e_tot(centers: &[[f64; 3]]) -> f64 {
    let h = by_symbol("H").unwrap();
    let species = vec![h; centers.len()];
    let cs: Vec<[D2; 3]> = centers
        .iter()
        .map(|c| [D2::c(c[0]), D2::c(c[1]), D2::c(c[2])])
        .collect();
    solve_geometry(&species, cs).e.v
}

fn v2(r: f64) -> f64 {
    let h = by_symbol("H").unwrap();
    pair_point(h, h, r).e - 2.0 * atom_energy(h)
}

fn d(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn de3(centers: &[[f64; 3]; 3]) -> f64 {
    let h = by_symbol("H").unwrap();
    let v_tot = e_tot(centers) - 3.0 * atom_energy(h);
    let pair_sum = v2(d(&centers[0], &centers[1]))
        + v2(d(&centers[0], &centers[2]))
        + v2(d(&centers[1], &centers[2]));
    v_tot - pair_sum
}

fn main() {
    let h = by_symbol("H").unwrap();
    println!("E(H)  = {:+.9} Ha", atom_energy(h));
    println!("V2(r_e) = {:+.9} Ha (the pair well, sanity: ~ -0.2041)", v2(R_E));
    println!();

    // 1. compact trimers: the saturation sign
    let s = R_E;
    let eq = [
        [0.0, 0.0, 0.0],
        [s, 0.0, 0.0],
        [0.5 * s, s * 0.8660254037844386, 0.0],
    ];
    println!("dE3 equilateral (side r_e)        = {:+.6} Ha", de3(&eq));
    let lin = [[0.0, 0.0, 0.0], [s, 0.0, 0.0], [2.0 * s, 0.0, 0.0]];
    println!("dE3 linear symmetric (r_e, r_e)   = {:+.6} Ha", de3(&lin));
    let approach = [[0.0, 0.0, 0.0], [s, 0.0, 0.0], [s + 2.0, 0.0, 0.0]];
    println!("dE3 H2 + H at 2.0 bohr, collinear = {:+.6} Ha", de3(&approach));

    // 2. MBE consistency: the third atom leaves, dE3 must vanish
    let far = [[0.0, 0.0, 0.0], [s, 0.0, 0.0], [s + 20.0, 0.0, 0.0]];
    println!("dE3 H2 + H at 20 bohr             = {:+.3e} Ha (must -> 0)", de3(&far));
    println!();

    // 3. the product: two dimers vs a compact tetrahedron, both as TOTAL energies
    let e_2h2 = 2.0 * e_tot(&[[0.0, 0.0, 0.0], [R_E, 0.0, 0.0]]);
    let t = R_E / 2.0_f64.sqrt();
    let tet = [
        [t, t, t],
        [t, -t, -t],
        [-t, t, -t],
        [-t, -t, t],
    ];
    let e_tet = e_tot(&tet);
    println!("E(2 x H2 at r_e, separated) = {:+.9} Ha", e_2h2);
    println!("E(H4 tetrahedron, edge r_e) = {:+.9} Ha", e_tet);
    println!(
        "two dimers win by {:+.6} Ha  ({})",
        e_tet - e_2h2,
        if e_tet > e_2h2 { "SATURATION" } else { "no saturation in-model" }
    );
}
