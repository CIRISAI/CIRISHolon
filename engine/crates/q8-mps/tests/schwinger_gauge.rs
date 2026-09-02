//! The Schwinger MPO on the engine, gauged three ways before any staked point runs:
//! an INDEPENDENT dense build of the Hamiltonian (basis states, no MPO) against the MPO's
//! own dense contraction, spectrum to spectrum; the engine's DMRG against that dense
//! spectrum on a chain small enough to diagonalise; and the SCHWINGER-4 freeze's plant (i)
//! referees at N = 12 (SCHWINGER-1's exact diagonalisation with the same static term).

use q8_mps::eigen::jacobi_eigen;
use q8_mps::schwinger::{Mutation, Schwinger};

/// The Hamiltonian written out over basis states — no MPO anywhere. Basis index `b` has
/// bit `k` set iff site `k` carries `σᶻ = +1`; `s = 0` in the MPO's convention is `σᶻ = +1`,
/// and `Mpo::dense` orders its basis with site 0 as the MOST significant digit, so the two
/// builds are compared by spectrum, never by matrix entry.
fn independent_dense(s: &Schwinger) -> Vec<f64> {
    let n = s.n;
    let dim = 1usize << n;
    let eps = s.background_flux();
    let mut h = vec![0.0; dim * dim];
    for b in 0..dim {
        let q = |k: usize| -> f64 {
            let sz = if (b >> k) & 1 == 1 { 1.0 } else { -1.0 };
            let stag = if k % 2 == 0 { 1.0 } else { -1.0 };
            0.5 * (sz + stag)
        };
        let mut diag = 0.0;
        let mut acc = 0.0;
        for k in 0..n - 1 {
            acc += q(k);
            let l = if s.mutation == Mutation::CoulombOff { 0.0 } else { acc + eps[k] };
            diag += l * l;
        }
        if s.mutation == Mutation::CoulombOff {
            let c = s.site_potential();
            for k in 0..n {
                diag += c[k] * q(k);
            }
            diag += s.constant();
        }
        let total: f64 = (0..n).map(q).sum();
        diag += s.lam * total * total;
        h[b * dim + b] = diag;
        for k in 0..n - 1 {
            if ((b >> k) & 1) != ((b >> (k + 1)) & 1) {
                let b2 = b ^ (1 << k) ^ (1 << (k + 1));
                h[b2 * dim + b] += s.x;
            }
        }
    }
    h
}

fn spectrum(h: Vec<f64>, dim: usize) -> Vec<f64> {
    let e = jacobi_eigen(h, dim);
    let mut v = e.values.clone();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v
}

#[test]
fn the_mpo_is_the_hamiltonian_spectrum_to_spectrum() {
    for (n, charges, mutation) in [
        (4usize, vec![], Mutation::None),
        (6, vec![(1, 1), (3, -1)], Mutation::None),
        (6, vec![(0, 1), (2, -1), (3, 1), (5, -1)], Mutation::None),
        (6, vec![(1, 1), (3, -1)], Mutation::CoulombOff),
    ] {
        let s = Schwinger::new(n, 4.0, charges.clone()).with_mutation(mutation);
        let dim = 1usize << n;
        let a = spectrum(s.mpo().dense(), dim);
        let mut b = spectrum(independent_dense(&s), dim);
        // the MPO carries the static constant outside the tensor; put it back
        for v in b.iter_mut() {
            *v -= s.constant();
        }
        let worst = a.iter().zip(&b).map(|(p, q)| (p - q).abs()).fold(0.0f64, f64::max);
        assert!(worst <= 1e-9, "N={n} charges={charges:?} {mutation:?}: spectra differ by {worst:e}");
    }
}

#[test]
fn the_engine_sweep_reaches_the_dense_ground_state() {
    let s = Schwinger::new(6, 4.0, vec![(1, 1), (3, -1)]);
    let dim = 64;
    let exact = spectrum(independent_dense(&s), dim)[0];
    let (e, res) = s.ground_energy(16, 40, 1e-12).expect("sweep");
    assert!((e - exact).abs() <= 1e-9, "engine {e} vs dense {exact}: {:e}", (e - exact).abs());
    assert!(res.worst_lanczos_residual <= 1e-9);
}

/// Plant (i) of the freeze on THIS driver: SCHWINGER-1's ED with the static term at N = 12,
/// x = 4 (numbers printed by `schwinger4.py gauge`, 2026-09-02), at the 1e-6 bar.
#[test]
fn plant_i_the_exact_referee_at_twelve_sites() {
    for (charges, e_ed) in [
        (vec![], -26.1743157050),
        (vec![(4, 1), (6, -1)], -24.3677609251),
        (vec![(4, 1), (6, -1), (8, 1), (10, -1)], -22.5337644964),
    ] {
        let s = Schwinger::new(12, 4.0, charges.clone());
        let (e, _) = s.ground_energy(48, 60, 1e-11).expect("sweep");
        assert!((e - e_ed).abs() <= 1e-6, "charges {charges:?}: engine {e:.10} vs ED {e_ed:.10}");
    }
}
