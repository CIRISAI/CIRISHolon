use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::atom_energy;
use holon_chem::quaternary::{de4_ohhh, de4_ohhh_cart, third_hydrogen_binding, G2_DEFICIT, R_CUT};
use holon_chem::trimer::TrimerTable;
use holon_chem::water::WaterTable;

#[test]
fn s3_permutational_symmetry_is_bit_exact_across_all_six_permutations() {
    let (r1, r2, r3) = (1.94, 2.10, 2.30);
    let (r12, r23, r31) = (2.90, 3.10, 3.30);

    // Permutation 1: (1, 2, 3)
    let v1 = de4_ohhh(r1, r2, r3, r12, r23, r31);

    // Permutation 2: (1, 3, 2)
    let v2 = de4_ohhh(r1, r3, r2, r31, r23, r12);

    // Permutation 3: (2, 1, 3)
    let v3 = de4_ohhh(r2, r1, r3, r12, r31, r23);

    // Permutation 4: (2, 3, 1)
    let v4 = de4_ohhh(r2, r3, r1, r23, r12, r31);

    // Permutation 5: (3, 1, 2)
    let v5 = de4_ohhh(r3, r1, r2, r31, r12, r23);

    // Permutation 6: (3, 2, 1)
    let v6 = de4_ohhh(r3, r2, r1, r23, r31, r12);

    assert_eq!(v1.to_bits(), v2.to_bits());
    assert_eq!(v1.to_bits(), v3.to_bits());
    assert_eq!(v1.to_bits(), v4.to_bits());
    assert_eq!(v1.to_bits(), v5.to_bits());
    assert_eq!(v1.to_bits(), v6.to_bits());
}

#[test]
fn asymptotic_decoupling_when_any_hydrogen_dissociates() {
    let r_eq_oh = 1.94357;
    let r_eq_hh = 2.90318;

    // Inside cutoff
    let v_bound = de4_ohhh(r_eq_oh, r_eq_oh, r_eq_oh, r_eq_hh, r_eq_hh, r_eq_hh);
    assert!(v_bound > 0.0);

    // Hydrogen 3 departs past R_CUT
    let v_far = de4_ohhh(r_eq_oh, r_eq_oh, R_CUT + 0.5, r_eq_hh, R_CUT + 1.0, R_CUT + 1.0);
    assert_eq!(v_far, 0.0, "dE4 must be strictly zero beyond R_CUT");
}

#[test]
fn g2_deficit_magnitude_sized_correctly() {
    let r_eq_oh = 1.94357;
    let r_eq_hh = 2.90318;

    let v_eq = de4_ohhh(r_eq_oh, r_eq_oh, r_eq_oh, r_eq_hh, r_eq_hh, r_eq_hh);
    // At equilibrium, dE4 is within reasonable margin of the staked G2 deficit (+0.183 Ha)
    assert!(v_eq > 0.10 && v_eq < 0.25, "v_eq = {}", v_eq);
}

#[test]
fn s3_cartesian_permutational_symmetry_holds() {
    let o = [0.0, 0.0, 0.0];
    let h1 = [1.8, 0.5, 0.2];
    let h2 = [-1.5, 0.8, -0.3];
    let h3 = [0.2, -1.9, 0.4];

    let v_orig = de4_ohhh_cart(&[o, h1, h2, h3]);
    let v_perm = de4_ohhh_cart(&[o, h3, h1, h2]);

    assert!((v_orig - v_perm).abs() < 1e-14);
}
