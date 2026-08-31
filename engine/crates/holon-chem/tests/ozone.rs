use holon_chem::ozone::{node_index, OzoneMeta, OzoneTable, NR, NU};

#[test]
fn ozone_s3_exchange_symmetry_is_bit_exact() {
    let mut table = OzoneTable::empty();
    table.begin();

    for i in 0..NR {
        for j in 0..NR {
            for k in 0..NU {
                let val = (i as f64 + 1.0).sqrt() * (j as f64 + 1.0).sqrt() * (k as f64 * 0.1).cos();
                table.knot(node_index(i, j, k), val);
            }
        }
    }
    table.finish(OzoneMeta::empty());

    let (s1, s2, s3) = (2.20, 2.80, 3.10);
    let (v_123, g_123) = table.eval(s1, s2, s3);
    let (v_213, g_213) = table.eval(s2, s1, s3);
    let (v_312, g_312) = table.eval(s3, s1, s2);
    let (v_321, g_321) = table.eval(s3, s2, s1);

    // Value must be bit-for-bit identical under all permutations
    assert_eq!(v_123.to_bits(), v_213.to_bits());
    assert_eq!(v_123.to_bits(), v_312.to_bits());
    assert_eq!(v_123.to_bits(), v_321.to_bits());

    // Gradients must permute with corresponding sides
    assert_eq!(g_123[0].to_bits(), g_213[1].to_bits());
    assert_eq!(g_123[1].to_bits(), g_213[0].to_bits());
    assert_eq!(g_123[2].to_bits(), g_213[2].to_bits());

    assert_eq!(g_123[0].to_bits(), g_312[1].to_bits());
    assert_eq!(g_123[1].to_bits(), g_312[2].to_bits());
    assert_eq!(g_123[2].to_bits(), g_312[0].to_bits());
}

#[test]
fn ozone_gradient_tracks_finite_difference() {
    let mut table = OzoneTable::empty();
    table.begin();
    for i in 0..NR {
        for j in 0..NR {
            for k in 0..NU {
                let tau_i = i as f64 / (NR - 1) as f64;
                let tau_j = j as f64 / (NR - 1) as f64;
                let c_k = k as f64 / (NU - 1) as f64;
                let val = (tau_i * 2.0 + 1.0).exp() * (tau_j * 1.5 + 1.0).exp() * (c_k + 0.2).sin();
                table.knot(node_index(i, j, k), val);
            }
        }
    }
    table.finish(OzoneMeta::empty());

    let (s1, s2, s3) = (2.4, 2.8, 3.2);
    let (v0, grad) = table.eval(s1, s2, s3);

    let eps = 1e-5;
    let (v_ds1, _) = table.eval(s1 + eps, s2, s3);
    let fd_ds1 = (v_ds1 - v0) / eps;
    assert!((grad[0] - fd_ds1).abs() < 1e-4, "grad[0] = {}, fd = {}", grad[0], fd_ds1);

    let (v_ds2, _) = table.eval(s1, s2 + eps, s3);
    let fd_ds2 = (v_ds2 - v0) / eps;
    assert!((grad[1] - fd_ds2).abs() < 1e-4, "grad[1] = {}, fd = {}", grad[1], fd_ds2);

    let (v_ds3, _) = table.eval(s1, s2, s3 + eps);
    let fd_ds3 = (v_ds3 - v0) / eps;
    assert!((grad[2] - fd_ds3).abs() < 1e-4, "grad[2] = {}, fd = {}", grad[2], fd_ds3);
}
