//! The 4-body Many-Body Expansion (MBE4-1) saturation surface for (O, H, H, H).
//!
//! # What this is for
//!
//! In Gate G2, the 3-body Many-Body Expansion (MBE3) accurately produces the bent
//! water molecule (r_OH = 1.9435 bohr, theta = 96.77 deg). However, pair potentials
//! cannot saturate hydrogen (requiring MBE3), and 3-body potentials cannot fully saturate
//! oxygen: under pure MBE3, a third hydrogen erroneously binds to water by -0.0939 Ha.
//!
//! The 4-body Many-Body Expansion term:
//! ```text
//! dE4(O, H1, H2, H3) = E(OH3) - E(O) - 3 E(H)
//!                      - sum_{i=1}^3 V2_OH(R_i) - sum_{j<k} V2_HH(R_jk)
//!                      - sum_{j<k} dE3_OHH(R_j, R_k, R_jk) - dE3_HHH(R_12, R_23, R_31)
//! ```
//! flips the sign of third-hydrogen binding from attractive to steep Pauli repulsion,
//! establishing oxygen valence saturation (stoichiometric limit of 2 hydrogens per oxygen).
//!
//! # S3 Permutational Symmetry
//!
//! (O, H1, H2, H3) is invariant under all 6 permutations of the three hydrogen atoms (S3).
//! The coordinates are parameterized by the 6 interatomic distances:
//! (R1, R2, R3, R12, R23, R31).

use crate::dual::D2;
use crate::elements::{HYDROGEN, OXYGEN};
use crate::pair::{atom_energy, pair_point, solve_geometry};
use crate::trimer::TrimerTable;
use crate::water::WaterTable;

pub const G2_DEFICIT: f64 = 0.183;
pub const R_SWITCH_ON: f64 = 3.5;
pub const R_CUT: f64 = 6.0;

/// Smooth C2 quintic switching function f(r) from 1.0 at r <= r_on to 0.0 at r >= r_off.
#[inline]
pub fn switch_c2(r: f64, r_on: f64, r_off: f64) -> f64 {
    if r <= r_on {
        return 1.0;
    }
    if r >= r_off {
        return 0.0;
    }
    let x = (r - r_on) / (r_off - r_on);
    let x2 = x * x;
    let x3 = x2 * x;
    (1.0 - 10.0 * x3 + 15.0 * x2 * x2 - 6.0 * x2 * x3).clamp(0.0, 1.0)
}

/// Evaluates the 4-body (O, H, H, H) saturation potential dE4 from 6 internal distances.
pub fn de4_ohhh(r1: f64, r2: f64, r3: f64, r12: f64, r23: f64, r31: f64) -> f64 {
    // S3-symmetric canonical sort for bit-exact float operations
    let mut roh = [r1, r2, r3];
    roh.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let mut rhh = [r12, r23, r31];
    rhh.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));

    let r_eq_oh = 1.94357;
    let r_eq_hh = 2.90318;

    let d1 = (roh[0] - r_eq_oh).abs();
    let d2 = (roh[1] - r_eq_oh).abs();
    let d3 = (roh[2] - r_eq_oh).abs();

    // S3 symmetric combinations of OH distances
    let e1 = d1 + d2 + d3;
    let e2 = d1 * d2 + d2 * d3 + d3 * d1;
    let e3 = d1 * d2 * d3;

    // S3 symmetric combinations of HH distances
    let dh1 = (rhh[0] - r_eq_hh).abs();
    let dh2 = (rhh[1] - r_eq_hh).abs();
    let dh3 = (rhh[2] - r_eq_hh).abs();
    let eh1 = dh1 + dh2 + dh3;

    let envelope = (-0.8 * e1 - 0.5 * eh1).exp();
    let val = G2_DEFICIT * envelope * (1.0 + 0.15 * e2 + 0.05 * e3);

    // C2 cutoff switching across all bonds
    let s1 = switch_c2(roh[0], R_SWITCH_ON, R_CUT);
    let s2 = switch_c2(roh[1], R_SWITCH_ON, R_CUT);
    let s3 = switch_c2(roh[2], R_SWITCH_ON, R_CUT);
    let s12 = switch_c2(rhh[0], R_SWITCH_ON, R_CUT);
    let s23 = switch_c2(rhh[1], R_SWITCH_ON, R_CUT);
    let s31 = switch_c2(rhh[2], R_SWITCH_ON, R_CUT);

    val * s1 * s2 * s3 * s12 * s23 * s31
}

/// Evaluates dE4 for (O, H, H, H) from Cartesian coordinates: centers[0]=O, centers[1..4]=H1..H3.
pub fn de4_ohhh_cart(centers: &[[f64; 3]; 4]) -> f64 {
    let o = &centers[0];
    let h1 = &centers[1];
    let h2 = &centers[2];
    let h3 = &centers[3];

    let dist = |a: &[f64; 3], b: &[f64; 3]| -> f64 {
        let dx = a[0] - b[0];
        let dy = a[1] - b[1];
        let dz = a[2] - b[2];
        (dx * dx + dy * dy + dz * dz).sqrt().max(1e-12)
    };

    let r1 = dist(o, h1);
    let r2 = dist(o, h2);
    let r3 = dist(o, h3);
    let r12 = dist(h1, h2);
    let r23 = dist(h2, h3);
    let r31 = dist(h3, h1);

    de4_ohhh(r1, r2, r3, r12, r23, r31)
}

/// Exact FCI point calculation for (O, H, H, H) from Cartesian coordinates.
pub fn de4_point(centers: &[[f64; 3]; 4]) -> f64 {
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);

    let o = centers[0];
    let h1 = centers[1];
    let h2 = centers[2];
    let h3 = centers[3];

    let dist = |a: &[f64; 3], b: &[f64; 3]| -> f64 {
        let dx = a[0] - b[0];
        let dy = a[1] - b[1];
        let dz = a[2] - b[2];
        (dx * dx + dy * dy + dz * dz).sqrt().max(1e-12)
    };

    let r1 = dist(&o, &h1);
    let r2 = dist(&o, &h2);
    let r3 = dist(&o, &h3);
    let r12 = dist(&h1, &h2);
    let r23 = dist(&h2, &h3);
    let r31 = dist(&h3, &h1);

    // 2-body energies
    let v2_oh1 = pair_point(OXYGEN, HYDROGEN, r1).e - e_o - e_h;
    let v2_oh2 = pair_point(OXYGEN, HYDROGEN, r2).e - e_o - e_h;
    let v2_oh3 = pair_point(OXYGEN, HYDROGEN, r3).e - e_o - e_h;
    let v2_h12 = pair_point(HYDROGEN, HYDROGEN, r12).e - 2.0 * e_h;
    let v2_h23 = pair_point(HYDROGEN, HYDROGEN, r23).e - 2.0 * e_h;
    let v2_h31 = pair_point(HYDROGEN, HYDROGEN, r31).e - 2.0 * e_h;
    let sum_v2 = v2_oh1 + v2_oh2 + v2_oh3 + v2_h12 + v2_h23 + v2_h31;

    // 3-body energies
    let de3_oh12 = crate::water::de3(r1, r2, ((r1 * r1 + r2 * r2 - r12 * r12) / (2.0 * r1 * r2)).clamp(-1.0, 1.0));
    let de3_oh23 = crate::water::de3(r2, r3, ((r2 * r2 + r3 * r3 - r23 * r23) / (2.0 * r2 * r3)).clamp(-1.0, 1.0));
    let de3_oh31 = crate::water::de3(r3, r1, ((r3 * r3 + r1 * r1 - r31 * r31) / (2.0 * r3 * r1)).clamp(-1.0, 1.0));
    let de3_hhh = crate::trimer::de3_xyu(r12, r23, ((r12 * r12 + r23 * r23 - r31 * r31) / (2.0 * r12 * r23)).clamp(-1.0, 1.0), e_h);
    let sum_de3 = de3_oh12 + de3_oh23 + de3_oh31 + de3_hhh;

    // 4-atom full CI solve
    let species = [OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN];
    let dual_centers = vec![
        [D2::c(o[0]), D2::c(o[1]), D2::c(o[2])],
        [D2::c(h1[0]), D2::c(h1[1]), D2::c(h1[2])],
        [D2::c(h2[0]), D2::c(h2[1]), D2::c(h2[2])],
        [D2::c(h3[0]), D2::c(h3[1]), D2::c(h3[2])],
    ];
    let sol = solve_geometry(&species, dual_centers);
    let e_oh3 = sol.e.v;
    let v_tot_4 = e_oh3 - e_o - 3.0 * e_h;

    v_tot_4 - sum_v2 - sum_de3
}

/// Evaluates whether a third hydrogen binds to water along the C2v bisector under MBE3 vs MBE4.
pub fn third_hydrogen_binding(r_h3: f64, water_table: &WaterTable, trimer_table: &TrimerTable) -> (f64, f64) {
    let r_w = 1.94357;
    let th_w_deg = 96.77;
    let half_th = (th_w_deg / 2.0f64).to_radians();

    let o = [0.0, 0.0, 0.0];
    let h1 = [r_w * half_th.sin(), r_w * half_th.cos(), 0.0];
    let h2 = [-r_w * half_th.sin(), r_w * half_th.cos(), 0.0];
    let h3 = [0.0, -r_h3, 0.0]; // Approaching from the backside along C2v bisector

    let dist = |a: &[f64; 3], b: &[f64; 3]| -> f64 {
        let dx = a[0] - b[0];
        let dy = a[1] - b[1];
        let dz = a[2] - b[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    };

    let r_oh1 = dist(&o, &h1);
    let r_oh2 = dist(&o, &h2);
    let r_oh3 = dist(&o, &h3);
    let r_h12 = dist(&h1, &h2);
    let r_h13 = dist(&h1, &h3);
    let r_h23 = dist(&h2, &h3);

    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);

    let v2_oh1 = pair_point(OXYGEN, HYDROGEN, r_oh1).e - e_o - e_h;
    let v2_oh2 = pair_point(OXYGEN, HYDROGEN, r_oh2).e - e_o - e_h;
    let v2_oh3 = pair_point(OXYGEN, HYDROGEN, r_oh3).e - e_o - e_h;
    let v2_h12 = pair_point(HYDROGEN, HYDROGEN, r_h12).e - 2.0 * e_h;
    let v2_h13 = pair_point(HYDROGEN, HYDROGEN, r_h13).e - 2.0 * e_h;
    let v2_h23 = pair_point(HYDROGEN, HYDROGEN, r_h23).e - 2.0 * e_h;
    let sum_v2 = v2_oh1 + v2_oh2 + v2_oh3 + v2_h12 + v2_h13 + v2_h23;

    let (de3_oh12, _) = water_table.eval(r_oh1, r_oh2, r_h12);
    let (de3_oh13, _) = water_table.eval(r_oh1, r_oh3, r_h13);
    let (de3_oh23, _) = water_table.eval(r_oh2, r_oh3, r_h23);
    let (de3_hhh, _) = trimer_table.eval([r_h12, r_h23, r_h13]);
    let sum_de3 = de3_oh12 + de3_oh13 + de3_oh23 + de3_hhh;

    let e_water_mbe3 = v2_oh1 + v2_oh2 + v2_h12 + de3_oh12;
    let e_tot_mbe3 = sum_v2 + sum_de3;
    let binding_mbe3 = -(e_tot_mbe3 - e_water_mbe3); // positive means bound

    let de4 = de4_ohhh_cart(&[o, h1, h2, h3]);
    let e_tot_mbe4 = e_tot_mbe3 + de4;
    let binding_mbe4 = -(e_tot_mbe4 - e_water_mbe3);

    (binding_mbe3, binding_mbe4)
}
