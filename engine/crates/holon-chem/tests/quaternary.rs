//! Tests for the Order-4 (O, H, H, H) saturation surface and MBE4 oxygen valence saturation.

use std::f64::consts::PI;
use std::sync::OnceLock;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::atom_energy;
use holon_chem::quaternary::{
    de4_ohhh, de4_ohhh_cart, Mbe4, PairCurve, D_HH_EQ, G2_DEFICIT, R_CUT_4BODY, R_W_EQ, TH_W_DEG,
};
use holon_chem::trimer::TrimerTable;
use holon_chem::water::{self, WaterTable};

fn water_table() -> &'static WaterTable {
    static T: OnceLock<WaterTable> = OnceLock::new();
    T.get_or_init(|| {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(manifest_dir).join("tests/data/s2/s2_water_table.txt");
        let src = std::fs::read_to_string(&path).expect("the committed (O,H,H) table is present");
        water::from_text(&src).expect("the committed water table parses")
    })
}

fn h3_table() -> &'static TrimerTable {
    static T: OnceLock<TrimerTable> = OnceLock::new();
    T.get_or_init(TrimerTable::empty)
}

#[test]
fn s3_permutational_symmetry_is_bit_exact_across_all_six_permutations() {
    let r1 = 1.94357;
    let r2 = 2.15000;
    let r3 = 1.82000;
    let r12 = 2.90318;
    let r23 = 3.10000;
    let r31 = 2.65000;

    let base = de4_ohhh(r1, r2, r3, r12, r23, r31);
    assert!(base > 0.0, "de4 must be positive in bonding region");

    // All 6 elements of S_3:
    let p_123 = de4_ohhh(r1, r2, r3, r12, r23, r31);
    let p_213 = de4_ohhh(r2, r1, r3, r12, r31, r23);
    let p_132 = de4_ohhh(r1, r3, r2, r31, r23, r12);
    let p_321 = de4_ohhh(r3, r2, r1, r23, r12, r31);
    let p_231 = de4_ohhh(r2, r3, r1, r23, r31, r12);
    let p_312 = de4_ohhh(r3, r1, r2, r31, r12, r23);

    assert_eq!(base.to_bits(), p_123.to_bits(), "S3 (1,2,3) failed");
    assert_eq!(base.to_bits(), p_213.to_bits(), "S3 (2,1,3) failed");
    assert_eq!(base.to_bits(), p_132.to_bits(), "S3 (1,3,2) failed");
    assert_eq!(base.to_bits(), p_321.to_bits(), "S3 (3,2,1) failed");
    assert_eq!(base.to_bits(), p_231.to_bits(), "S3 (2,3,1) failed");
    assert_eq!(base.to_bits(), p_312.to_bits(), "S3 (3,1,2) failed");
}

#[test]
fn s3_cartesian_permutational_symmetry_holds() {
    let o = [0.0, 0.0, 0.0];
    let h1 = [0.0, 1.45, 1.30];
    let h2 = [0.0, -1.45, 1.30];
    let h3 = [1.50, 0.0, -0.80];

    let base = de4_ohhh_cart(o, h1, h2, h3);
    assert!(base > 0.0, "de4 must be positive in bonding region");

    let p_123 = de4_ohhh_cart(o, h1, h2, h3);
    let p_213 = de4_ohhh_cart(o, h2, h1, h3);
    let p_132 = de4_ohhh_cart(o, h1, h3, h2);
    let p_321 = de4_ohhh_cart(o, h3, h2, h1);
    let p_231 = de4_ohhh_cart(o, h2, h3, h1);
    let p_312 = de4_ohhh_cart(o, h3, h1, h2);

    assert_eq!(base.to_bits(), p_123.to_bits());
    assert_eq!(base.to_bits(), p_213.to_bits());
    assert_eq!(base.to_bits(), p_132.to_bits());
    assert_eq!(base.to_bits(), p_321.to_bits());
    assert_eq!(base.to_bits(), p_231.to_bits());
    assert_eq!(base.to_bits(), p_312.to_bits());
}

#[test]
fn asymptotic_decoupling_when_any_hydrogen_dissociates() {
    let r1 = R_W_EQ;
    let r2 = R_W_EQ;
    let r12 = D_HH_EQ;

    for r_far in [R_CUT_4BODY, 7.0, 10.0, 25.0] {
        let r23 = (r2 * r2 + r_far * r_far).sqrt();
        let r31 = (r1 * r1 + r_far * r_far).sqrt();
        let val = de4_ohhh(r1, r2, r_far, r12, r23, r31);
        assert_eq!(
            val, 0.0,
            "de4 must be exactly 0.0 past cutoff {R_CUT_4BODY}, got {val} at r3={r_far}"
        );
    }
}

#[test]
fn g2_deficit_magnitude_sized_correctly() {
    let r1 = R_W_EQ;
    let r2 = R_W_EQ;
    let r3 = R_W_EQ;
    let r12 = D_HH_EQ;
    let r23 = D_HH_EQ;
    let r31 = D_HH_EQ;

    let val = de4_ohhh(r1, r2, r3, r12, r23, r31);
    let diff = (val - G2_DEFICIT).abs();
    assert!(
        diff < 1e-4,
        "de4 at symmetric equilibrium must match G2 deficit {G2_DEFICIT} Ha, got {val} Ha (diff {diff:.2e})"
    );
}

#[test]
fn mbe4_enforces_oxygen_valence_saturation_along_g2_channel() {
    let t_water = water_table();
    let t_h3 = h3_table();
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let (oh, hh) = (
        PairCurve::sample(OXYGEN, HYDROGEN),
        PairCurve::sample(HYDROGEN, HYDROGEN),
    );
    let mbe4 = Mbe4::new(t_water, t_h3, &oh, &hh, e_o, e_h);

    // Relaxed water in the XY plane:
    let r_w = R_W_EQ;
    let th_w = TH_W_DEG * PI / 180.0;
    let o = [0.0, 0.0, 0.0];
    let h1 = [r_w * (th_w / 2.0).cos(), r_w * (th_w / 2.0).sin(), 0.0];
    let h2 = [r_w * (th_w / 2.0).cos(), -r_w * (th_w / 2.0).sin(), 0.0];

    // C2 axis approach (the G2 probe channel away from the two hydrogens):
    println!("Testing third hydrogen approach along C2 lone-pair axis:");
    for r in [1.5, 1.8, 1.94357, 2.1, 2.5, 3.0, 4.0, 5.0] {
        let h3 = [-r, 0.0, 0.0];
        let (binding_mbe3, binding_mbe4) = mbe4.third_hydrogen_binding(o, h1, h2, h3);
        println!(
            "  r_OH3 = {r:.3} bohr: MBE3 binding = {binding_mbe3:+.4} Ha, MBE4 binding = {binding_mbe4:+.4} Ha"
        );
        // In the bonding region (r <= 3.0 bohr), binding must be strictly repulsive (<= 0.0 Ha).
        if r <= 3.0 {
            assert!(
                binding_mbe4 <= 0.0,
                "MBE4 must repel third hydrogen along C2 axis at r={r}, got binding {binding_mbe4:+.4} Ha"
            );
        } else {
            // Asymptotic tail: binding must be shallower than 0.005 Ha (>>5x shallower than O-H bond of 0.163 Ha)
            assert!(
                binding_mbe4 <= 0.005,
                "MBE4 asymptotic binding at r={r} must satisfy G2 margin, got {binding_mbe4:+.4} Ha"
            );
        }
    }
}
