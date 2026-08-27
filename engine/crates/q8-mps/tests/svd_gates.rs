//! G-SVD1/2/3 — `Q8_MPS_PREREG.md` §2.1. Nothing downstream of the SVD runs before these pass.

use q8_mps::svd::{jacobi_svd, reconstruct};

/// `SVD_FIXTURE_SEED = 1`, pinned in the prereg. Test-only: production DMRG uses no RNG at all
/// (fixed initial state, per the commission), this generator exists solely to build fixture
/// matrices deterministically and replayably.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_signed(&mut self) -> f64 {
        let bits = self.next_u64() >> 11;
        (bits as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    }
}

const SVD_FIXTURE_SEED: u64 = 1;

fn seeded_matrix(m: usize, n: usize, seed: u64) -> Vec<f64> {
    let mut rng = SplitMix64(seed);
    (0..m * n).map(|_| rng.next_signed()).collect()
}

fn seeded_orthonormal_columns(rows: usize, cols: usize, seed: u64) -> Vec<Vec<f64>> {
    assert!(cols <= rows);
    let mut rng = SplitMix64(seed);
    let mut basis: Vec<Vec<f64>> = Vec::with_capacity(cols);
    for _ in 0..cols {
        let mut column: Vec<f64> = (0..rows).map(|_| rng.next_signed()).collect();
        for _ in 0..2 {
            for previous in &basis {
                let overlap: f64 = column.iter().zip(previous).map(|(x, y)| x * y).sum();
                for (x, y) in column.iter_mut().zip(previous) {
                    *x -= overlap * y;
                }
            }
        }
        let norm = column.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!(norm > 1e-12);
        for x in &mut column {
            *x /= norm;
        }
        basis.push(column);
    }
    basis
}

fn matrix_with_spectrum(m: usize, n: usize, singular_values: &[f64], seed: u64) -> Vec<f64> {
    assert_eq!(singular_values.len(), m.min(n));
    let u = seeded_orthonormal_columns(m, singular_values.len(), seed);
    let v = seeded_orthonormal_columns(n, singular_values.len(), seed + 1);
    let mut a = vec![0.0; m * n];
    for (k, &sigma) in singular_values.iter().enumerate() {
        for i in 0..m {
            for j in 0..n {
                a[i * n + j] += sigma * u[k][i] * v[k][j];
            }
        }
    }
    a
}

/// `m x n` row-major times `n x p` row-major.
fn matmul(a: &[f64], m: usize, n: usize, b: &[f64], p: usize) -> Vec<f64> {
    let mut out = vec![0.0; m * p];
    for i in 0..m {
        for k in 0..n {
            let aik = a[i * n + k];
            if aik == 0.0 {
                continue;
            }
            for j in 0..p {
                out[i * p + j] += aik * b[k * p + j];
            }
        }
    }
    out
}

fn max_abs_diff(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max)
}

/// `max|MᵀM - I|` for the `k` columns of `cols` (each length `len`), i.e. orthogonality of the
/// singular-vector set returned by `jacobi_svd`.
fn max_gram_deviation(cols: &[Vec<f64>]) -> f64 {
    let k = cols.len();
    let mut worst = 0.0f64;
    for i in 0..k {
        for j in 0..k {
            let dot: f64 = cols[i].iter().zip(&cols[j]).map(|(x, y)| x * y).sum();
            let target = if i == j { 1.0 } else { 0.0 };
            worst = worst.max((dot - target).abs());
        }
    }
    worst
}

const RECON_TOL: f64 = 1e-13;
const ORTHO_TOL: f64 = 1e-13;

fn check_fixture(name: &str, a: &[f64], m: usize, n: usize) {
    let svd = jacobi_svd(a, m, n);
    assert!(svd.converged, "{name}: SVD did not converge within the sweep cap");

    let recon = reconstruct(&svd, m, n);
    let recon_err = max_abs_diff(a, &recon);
    assert!(recon_err <= RECON_TOL, "{name}: reconstruction error {recon_err:e} > {RECON_TOL:e}");

    let u_err = max_gram_deviation(&svd.u);
    assert!(u_err <= ORTHO_TOL, "{name}: max|UᵀU-I| = {u_err:e} > {ORTHO_TOL:e}");

    let v_err = max_gram_deviation(&svd.v);
    assert!(v_err <= ORTHO_TOL, "{name}: max|VᵀV-I| = {v_err:e} > {ORTHO_TOL:e}");

    // Singular values are non-negative and descending — a property of the routine, not a
    // separate stake, but a cheap sanity check worth failing loudly on.
    for w in svd.s.windows(2) {
        assert!(w[0] + 1e-14 >= w[1], "{name}: singular values not descending: {:?}", svd.s);
    }
    for &sigma in &svd.s {
        assert!(sigma >= 0.0, "{name}: negative singular value {sigma}");
    }
}

#[test]
fn g_svd1_2_identity_3x3() {
    let a: Vec<f64> = (0..9).map(|i| if i % 4 == 0 { 1.0 } else { 0.0 }).collect();
    check_fixture("3x3 identity", &a, 3, 3);
}

#[test]
fn g_svd1_2_random_5x5() {
    let a = seeded_matrix(5, 5, SVD_FIXTURE_SEED);
    check_fixture("5x5 random", &a, 5, 5);
}

#[test]
fn g_svd1_2_random_20x20() {
    let a = seeded_matrix(20, 20, SVD_FIXTURE_SEED);
    check_fixture("20x20 random", &a, 20, 20);
}

#[test]
fn g_svd1_2_random_wide_32x64() {
    // An actual DMRG split shape at chi=32.  One-sided Jacobi acts on columns, so the wide
    // input exercises the transpose route: solve the smaller tall problem, then swap its
    // left/right singular vectors back without changing the decomposition.
    let a = seeded_matrix(32, 64, SVD_FIXTURE_SEED);
    check_fixture("32x64 random wide", &a, 32, 64);
}

#[test]
fn g_svd1_2_rank_deficient_wide_32x64() {
    // DMRG encounters this shape while a product-state bond is growing: the declared wide
    // reshape has rank far below its economy dimension.  It exercises both transpose routing
    // and the degenerate-direction completion used to preserve canonical form.
    let left = seeded_matrix(32, 12, SVD_FIXTURE_SEED);
    let right = seeded_matrix(12, 64, SVD_FIXTURE_SEED + 1);
    let a = matmul(&left, 32, 12, &right, 64);
    check_fixture("32x64 rank-12 wide", &a, 32, 64);
}

#[test]
fn g_svd1_2_dmrg_scale_separated_16x8() {
    // The failing sweep carries Schmidt values across many decades.  An absolute Gram-off
    // convergence test can declare arithmetic-floor stagnation while the small, still-retained
    // columns have O(1) *relative* overlaps; normalizing those columns then destroys the
    // left-canonical basis used by the next local solve.
    let singular_values = [1.0, 0.2, 0.04, 0.008, 0.0016, 3.2e-4, 6.4e-6, 1.28e-8];
    let a = matrix_with_spectrum(16, 8, &singular_values, SVD_FIXTURE_SEED);
    check_fixture("16x8 DMRG-scale-separated", &a, 16, 8);
}

#[test]
fn g_svd1_2_rank_deficient_10x10() {
    // 10x6 times 6x10: rank <= 6 by construction, a genuine SAME degenerate direction the
    // Gram-Schmidt completion in `jacobi_svd` exists for.
    let left = seeded_matrix(10, 6, SVD_FIXTURE_SEED);
    let right = seeded_matrix(6, 10, SVD_FIXTURE_SEED + 1);
    let a = matmul(&left, 10, 6, &right, 10);

    let svd = jacobi_svd(&a, 10, 10);
    let low_count = svd.s.iter().filter(|&&s| s < 1e-9).count();
    assert!(low_count >= 4, "fixture is not actually rank-deficient: singular values {:?}", svd.s);

    check_fixture("10x10 rank-6", &a, 10, 10);
}

#[test]
fn g_svd3_diagonal_spectrum_exact() {
    // diag(3,1,4,1,5,9): singular values must reproduce |diag| sorted descending.
    let diag = [3.0, 1.0, 4.0, 1.0, 5.0, 9.0];
    let n = diag.len();
    let mut a = vec![0.0; n * n];
    for (i, &d) in diag.iter().enumerate() {
        a[i * n + i] = d;
    }
    let svd = jacobi_svd(&a, n, n);

    let mut expected: Vec<f64> = diag.iter().map(|d| d.abs()).collect();
    expected.sort_by(|a, b| b.partial_cmp(a).unwrap());

    let err = max_abs_diff(&svd.s, &expected);
    assert!(err <= 1e-14, "G-SVD3: spectrum error {err:e} > 1e-14 ({:?} vs {:?})", svd.s, expected);
}
