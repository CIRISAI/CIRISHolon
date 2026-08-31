use holon_chem::quaternary::{de4_ohhh_fci, sort_ohhh_internals};
use holon_chem::trimer;
use holon_chem::water;

const R_W: f64 = 1.9435740105;
const TH_W: f64 = 96.75788837;
const PI: f64 = std::f64::consts::PI;

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[test]
fn s3_permutation_invariance_is_bit_exact() {
    let (r1, r2, r3) = (1.94, 2.10, 2.30);
    let (r12, r23, r31) = (2.90, 3.10, 3.30);

    let (roh1, rhh1) = sort_ohhh_internals(r1, r2, r3, r12, r23, r31);
    let (roh2, rhh2) = sort_ohhh_internals(r2, r3, r1, r23, r31, r12);
    let (roh3, rhh3) = sort_ohhh_internals(r3, r1, r2, r31, r12, r23);
    let (roh4, rhh4) = sort_ohhh_internals(r1, r3, r2, r31, r23, r12);

    assert_eq!(roh1, roh2);
    assert_eq!(roh1, roh3);
    assert_eq!(roh1, roh4);

    assert_eq!(rhh1, rhh2);
    assert_eq!(rhh1, rhh3);
    assert_eq!(rhh1, rhh4);
}

#[test]
fn forty_witnesses_ab_initio_sign_structure_and_bounds() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/s2/s2_water_table.txt"),
    )
    .expect("the committed (O,H,H) table");
    let w = water::from_text(&src).expect("water table parses");
    let h3 = trimer::generate().expect("the H3 table");

    let th = TH_W * PI / 180.0;
    let o = [0.0f64, 0.0, 0.0];
    let h1 = [R_W * (th / 2.0).cos(), R_W * (th / 2.0).sin(), 0.0];
    let h2 = [R_W * (th / 2.0).cos(), -R_W * (th / 2.0).sin(), 0.0];

    let dirs: [[f64; 3]; 8] = [
        [-1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [-0.7071, 0.0, 0.7071],
        [-0.7071, 0.0, -0.7071],
        [0.5774, 0.5774, 0.5774],
    ];
    let radii = [1.4f64, 1.8, 2.2, 2.8, 3.6];

    let mut n_tested = 0usize;
    let mut n_attractive = 0usize;
    let mut max_magnitude = 0.0f64;
    let mut sum_magnitude = 0.0f64;

    for d in dirs.iter() {
        let nn = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        for &r in &radii {
            let p = [r * d[0] / nn, r * d[1] / nn, r * d[2] / nn];
            if dist(o, p) < 0.9 || dist(h1, p) < 0.9 || dist(h2, p) < 0.9 {
                continue;
            }
            let de4 = de4_ohhh_fci(&[o, h1, h2, p], &w, &h3);
            n_tested += 1;
            sum_magnitude += de4.abs();
            max_magnitude = max_magnitude.max(de4.abs());
            if de4 < 0.0 {
                n_attractive += 1;
            }
        }
    }
    let mean_magnitude = sum_magnitude / n_tested as f64;

    assert_eq!(n_tested, 40, "Must test all 40 staked witness geometries");
    // Verify the fundamental sign-changing nature of the true 4-body term
    assert_eq!(n_attractive, 11, "True 4-body term must be attractive on exactly 11/40 geometries");
    assert!((max_magnitude - 0.228401).abs() < 1e-4, "Max magnitude across 40 witnesses is 0.228401 Ha, got {:.6}", max_magnitude);
    assert!((mean_magnitude - 0.111948).abs() < 1e-4, "Mean magnitude across 40 witnesses is 0.111948 Ha, got {:.6}", mean_magnitude);
}

#[test]
fn far_field_decay_validates_six_bohr_cutoff() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/s2/s2_water_table.txt"),
    )
    .expect("the committed (O,H,H) table");
    let w = water::from_text(&src).expect("water table parses");
    let h3 = trimer::generate().expect("the H3 table");

    let th = TH_W * PI / 180.0;
    let o = [0.0f64, 0.0, 0.0];
    let h1 = [R_W * (th / 2.0).cos(), R_W * (th / 2.0).sin(), 0.0];
    let h2 = [R_W * (th / 2.0).cos(), -R_W * (th / 2.0).sin(), 0.0];

    // At R = 6.1 bohr (past R_CUT = 6.0)
    let p_far = [-6.1, 0.0, 0.0];
    let de4_far = de4_ohhh_fci(&[o, h1, h2, p_far], &w, &h3);
    assert!(de4_far.abs() < 1e-4, "Far field at 6.1 bohr must decay, got {:.3e} Ha", de4_far);

    // At R = 9.0 bohr
    let p_very_far = [-9.0, 0.0, 0.0];
    let de4_very_far = de4_ohhh_fci(&[o, h1, h2, p_very_far], &w, &h3);
    assert!(de4_very_far.abs() < 5e-6, "Far field at 9.0 bohr must decay to near zero, got {:.3e} Ha", de4_very_far);
}
