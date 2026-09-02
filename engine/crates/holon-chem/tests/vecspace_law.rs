//! The reduction law and the row programs: one host thread, many host threads, and the
//! primitive form agree to the bit, and the folded Davidson lands where the primitive one does.

use holon_chem::scalar::Scalar;
use holon_chem::vecspace::{blocked_dot, HostSpace, VectorSpace, DOT_BLOCK};

fn lcg(n: usize, mut st: u64) -> Vec<f64> {
    (0..n)
        .map(|_| {
            st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((st >> 11) as f64) / ((1u64 << 53) as f64) - 0.5
        })
        .collect()
}

fn bits_equal(a: &[f64], b: &[f64]) -> usize {
    a.iter().zip(b).filter(|(x, y)| x.to_bits() != y.to_bits()).count()
}

#[test]
fn the_law_is_the_same_bits_on_every_thread_count() {
    for n in [1usize, 255, 256, 257, 10_000, 3 * DOT_BLOCK * 1024 + 17] {
        let (a, b) = (lcg(n, 1), lcg(n, 2));
        let serial = blocked_dot(&a, &b);
        for threads in [1usize, 2, 3, 8, 32] {
            let sp = HostSpace::with_threads(threads);
            let d = VectorSpace::<f64>::dot(&sp, &a, &b);
            assert_eq!(d.to_bits(), serial.to_bits(), "n={n} threads={threads}: {d:.17e} vs {serial:.17e}");
        }
        // and it is a dot product: within rounding of the naive sum
        let naive: f64 = a.iter().zip(&b).map(|(x, y)| x * y).sum();
        assert!((serial - naive).abs() <= 1e-12 * (1.0 + naive.abs()) * (n as f64).sqrt());
    }
}

#[test]
fn the_row_programs_are_the_primitive_form_to_the_bit() {
    let n = 5 * DOT_BLOCK * 64 + 3;
    let m = 5;
    let basis: Vec<Vec<f64>> = (0..m).map(|j| lcg(n, 10 + j as u64)).collect();
    let hbasis: Vec<Vec<f64>> = (0..m).map(|j| lcg(n, 20 + j as u64)).collect();
    let diag = lcg(n, 30);
    let y: Vec<f64> = vec![0.3, -0.2, 0.0, 0.7, 0.1];
    let theta = -1.25f64;
    for threads in [1usize, 4, 32] {
        let sp = HostSpace::with_threads(threads);
        // primitive form, serial, the law's dots
        let mut x = vec![0.0; n];
        let mut hx = vec![0.0; n];
        for j in 0..m {
            if y[j] != 0.0 {
                for i in 0..n {
                    x[i] = x[i] + y[j] * basis[j][i];
                }
            }
        }
        for j in 0..m {
            if y[j] != 0.0 {
                for i in 0..n {
                    hx[i] = hx[i] + y[j] * hbasis[j][i];
                }
            }
        }
        let r: Vec<f64> = (0..n).map(|i| hx[i] + (-theta) * x[i]).collect();
        let corr: Vec<f64> = (0..n)
            .map(|i| {
                let d = theta - diag[i];
                if d.abs() > 1e-8 { r[i] / d } else { r[i] }
            })
            .collect();
        let rr = blocked_dot(&r, &r);
        let bc: Vec<f64> = basis.iter().map(|b| blocked_dot(b, &corr)).collect();
        // the transform
        let (mut fx, mut fr, mut fc) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
        let (frr, fbc) = sp.ritz(&basis, &hbasis, &y, theta, &diag, &mut fx, &mut fr, &mut fc);
        assert_eq!(bits_equal(&x, &fx), 0, "x differs (threads {threads})");
        assert_eq!(bits_equal(&r, &fr), 0, "r differs");
        assert_eq!(bits_equal(&corr, &fc), 0, "corr differs");
        assert_eq!(frr.to_bits(), rr.to_bits(), "‖r‖² differs");
        assert_eq!(bits_equal(&bc, &fbc), 0, "Bᵀcorr differs");
        // deflate: w -= Σ p_j b_j (j ascending), then Bᵀw
        let p: Vec<f64> = fbc.clone();
        let mut w = corr.clone();
        for i in 0..n {
            let mut wi = w[i];
            for j in 0..m {
                wi = wi + (-p[j]) * basis[j][i];
            }
            w[i] = wi;
        }
        let bw: Vec<f64> = basis.iter().map(|b| blocked_dot(b, &w)).collect();
        let mut fw = corr.clone();
        let fbw = sp.deflate(&basis, &p, &mut fw);
        assert_eq!(bits_equal(&w, &fw), 0, "deflate differs");
        assert_eq!(bits_equal(&bw, &fbw), 0, "Bᵀw differs");
        let nn = blocked_dot(&w, &w);
        let mut w2 = corr.clone();
        let fnn = sp.deflate_norm(&basis, &p, &mut w2);
        assert_eq!(fnn.to_bits(), nn.to_bits(), "‖w‖² differs");
        let g = sp.gram_row(&basis, &corr);
        assert_eq!(bits_equal(&bc, &g), 0, "gram row differs");
        let mut s = corr.clone();
        sp.scale(0.5, &mut s);
        let s_ref: Vec<f64> = corr.iter().map(|v| v * 0.5).collect();
        assert_eq!(bits_equal(&s, &s_ref), 0, "scale differs");
        let mut ax = corr.clone();
        sp.axpy(-0.25, &r, &mut ax);
        let ax_ref: Vec<f64> = (0..n).map(|i| corr[i] + (-0.25) * r[i]).collect();
        assert_eq!(bits_equal(&ax, &ax_ref), 0, "axpy differs");
    }
}
