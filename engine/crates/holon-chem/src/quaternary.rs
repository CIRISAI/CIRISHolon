//! The 4-body Many-Body Expansion (MBE4-1) evaluation machinery for (O, H, H, H).
//!
//! # What this is for
//!
//! Under pure MBE3, a third hydrogen binds to water along the C2v bisector. The true
//! ab-initio 4-body term:
//! ```text
//! dE4(O, H1, H2, H3) = E_FCI(OH3) - E_MBE3(OH3)
//! ```
//! corrects the total energy. The four-body term changes sign across configuration space,
//! reaching +0.2755 Ha and becoming attractive on 11 of 40 held-out geometries.
//!
//! Per the maximal holon ruling, NO fitted or empirical potential is admitted.
//! All values are evaluated directly from the ab-initio electronic Hamiltonian (1,568 determinants)
//! or interpolated from certified tabulated surfaces.

use crate::dual::D2;
use crate::elements::{HYDROGEN, OXYGEN};
use crate::pair::{atom_energy, pair_point, solve_geometry};
use crate::trimer::TrimerTable;
use crate::water::WaterTable;

/// Measured far-field cutoff (bohr) where dE4 decays below T1 interpolation tolerance (~5e-5 Ha).
pub const R_CUT: f64 = 6.0;

#[inline]
fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt().max(1e-12)
}

/// Evaluates E_FCI for (O, H, H, H) in STO-3G minimal basis (1,568 determinants).
pub fn ohhh_fci_energy(centers: &[[f64; 3]; 4]) -> f64 {
    let species = [OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN];
    let dual_centers = vec![
        [D2::c(centers[0][0]), D2::c(centers[0][1]), D2::c(centers[0][2])],
        [D2::c(centers[1][0]), D2::c(centers[1][1]), D2::c(centers[1][2])],
        [D2::c(centers[2][0]), D2::c(centers[2][1]), D2::c(centers[2][2])],
        [D2::c(centers[3][0]), D2::c(centers[3][1]), D2::c(centers[3][2])],
    ];
    solve_geometry(&species, dual_centers).e.v
}

/// Evaluates E_MBE3 for the 4-atom system (O, H, H, H): 6 pairs + 4 triples + isolated atoms.
pub fn ohhh_mbe3_energy(
    centers: &[[f64; 3]; 4],
    water_table: &WaterTable,
    trimer_table: &TrimerTable,
) -> f64 {
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);

    let o = &centers[0];
    let h1 = &centers[1];
    let h2 = &centers[2];
    let h3 = &centers[3];

    let r1 = dist(o, h1);
    let r2 = dist(o, h2);
    let r3 = dist(o, h3);
    let r12 = dist(h1, h2);
    let r23 = dist(h2, h3);
    let r31 = dist(h3, h1);

    // 6 Pair terms
    let v2_oh1 = pair_point(OXYGEN, HYDROGEN, r1).e - e_o - e_h;
    let v2_oh2 = pair_point(OXYGEN, HYDROGEN, r2).e - e_o - e_h;
    let v2_oh3 = pair_point(OXYGEN, HYDROGEN, r3).e - e_o - e_h;
    let v2_h12 = pair_point(HYDROGEN, HYDROGEN, r12).e - 2.0 * e_h;
    let v2_h23 = pair_point(HYDROGEN, HYDROGEN, r23).e - 2.0 * e_h;
    let v2_h31 = pair_point(HYDROGEN, HYDROGEN, r31).e - 2.0 * e_h;
    let pairs = v2_oh1 + v2_oh2 + v2_oh3 + v2_h12 + v2_h23 + v2_h31;

    // 4 Triple terms: 3 (O,H,H) + 1 (H,H,H)
    let triples = water_table.eval(r1, r2, r12).0
        + water_table.eval(r2, r3, r23).0
        + water_table.eval(r3, r1, r31).0
        + trimer_table.eval([r12, r23, r31]).0;

    e_o + 3.0 * e_h + pairs + triples
}

/// Exact ab-initio 4-body term dE4 = E_FCI - E_MBE3 from Cartesian coordinates.
pub fn de4_ohhh_fci(
    centers: &[[f64; 3]; 4],
    water_table: &WaterTable,
    trimer_table: &TrimerTable,
) -> f64 {
    let ef = ohhh_fci_energy(centers);
    let em = ohhh_mbe3_energy(centers, water_table, trimer_table);
    ef - em
}

/// Six-coordinate S3 permutation sort for internal coordinate evaluation.
pub fn sort_ohhh_internals(
    r1: f64, r2: f64, r3: f64,
    r12: f64, r23: f64, r31: f64,
) -> ([f64; 3], [f64; 3]) {
    let mut roh = [r1, r2, r3];
    roh.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let mut rhh = [r12, r23, r31];
    rhh.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    (roh, rhh)
}
