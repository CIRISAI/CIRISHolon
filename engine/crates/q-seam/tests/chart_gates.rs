//! `Q_SEAM_PREREG.md` §3 — the chart's own gates, G-C1 to G-C3.

use q_seam::chart::Chart;
use q_seam::hubbard::{free_chain_energy_per_site, Hubbard};
use q_seam::lanczos::ground_state;

/// G-C1 — SCF convergence at `1e-12`, at every configuration of the pinned sweep.
#[test]
fn g_c1_scf_converges_everywhere() {
    for &n in &q_seam::SWEEP_SITES {
        for &u in &q_seam::SWEEP_U {
            let c = Chart::best(n, 1.0, u);
            assert!(c.is_some(), "N={n} U={u}: no guess converged; configuration would be VOID");
        }
    }
}

/// G-C2 — the chart's density matrix is idempotent: Booleanity CHECKED, not assumed.
///
/// This is `Core/ModeChart.lean`'s `meanOcc_boolean_of_pure` as an executable assertion: the
/// chart's mode occupations really are `{0,1}`, which is exactly why it cannot see its own
/// fractionality.
#[test]
fn g_c2_chart_density_matrix_is_idempotent() {
    for &n in &q_seam::SWEEP_SITES {
        for &u in &q_seam::SWEEP_U {
            let c = Chart::best(n, 1.0, u).unwrap();
            assert!(
                c.idempotency <= 1e-12,
                "N={n} U={u}: ||rho^2 - rho||_max = {:e}",
                c.idempotency
            );
        }
    }
}

/// G-C3 — the variational gate. Mean field is an upper bound, so `E_UHF >= E_exact` always.
/// A violation is a bug, and this is the cheapest place to catch one.
#[test]
fn g_c3_chart_is_variational() {
    for &n in &[2usize, 4, 6, 8] {
        for &u in &q_seam::SWEEP_U {
            let h = Hubbard::new(n, 1.0, u);
            let g = ground_state(&h).unwrap();
            let c = Chart::best(n, 1.0, u).unwrap();
            assert!(
                c.energy - g.energy >= -1e-10,
                "N={n} U={u}: E_UHF {} is BELOW E_exact {} by {:e}",
                c.energy,
                g.energy,
                g.energy - c.energy
            );
        }
    }
}

/// The chart is EXACT at `U = 0` — the premise of joint-gate clause (3), and the reason a
/// criterion that refuses `U = 0` is broken rather than conservative.
#[test]
fn the_chart_is_exact_at_u_zero() {
    for &n in &q_seam::SWEEP_SITES {
        let c = Chart::best(n, 1.0, 0.0).unwrap();
        let predicted = free_chain_energy_per_site(n);
        assert!(
            (c.energy_per_site() - predicted).abs() <= 1e-12,
            "N={n}: chart E/N = {} vs exact {predicted}",
            c.energy_per_site()
        );
        let (spin, ph, refl) = c.symmetry_breaking();
        assert!(spin <= 1e-12 && ph <= 1e-12 && refl <= 1e-12,
            "N={n}: chart breaks a symmetry at U=0 (spin {spin:e}, ph {ph:e}, refl {refl:e})");
    }
}

/// The stability Hessian validates itself at `U = 0`: its lowest eigenvalue must be the
/// particle–hole excitation gap `Δ(N)` up to one overall factor, and that factor must be the
/// same at every N. C2 is self-normalized against its own `U = 0` value, so the factor cancels
/// in the criterion — but it must exist, or the Hessian is not the object it claims to be.
#[test]
fn stability_hessian_reproduces_the_free_gap() {
    use q_seam::audit::Stability;
    use q_seam::hubbard::free_chain_gap;
    let mut ratios = Vec::new();
    for &n in &[2usize, 4, 6, 8] {
        let c = Chart::best(n, 1.0, 0.0).unwrap();
        let s = Stability::of(&c);
        assert_eq!(s.null_modes, 0, "N={n}: the symmetric chart has no broken symmetry, so no null mode");
        ratios.push(s.lambda_min / free_chain_gap(n));
    }
    // Tolerance from the finite-difference budget, not from the observed spread: a central
    // second difference at h = 1e-4 carries O(h^2) truncation plus O(eps/h^2) roundoff, and the
    // N = 8 Hessian accumulates that across 5050 entries before its eigenvalues are taken. 1e-5
    // still pins "the factor is exactly 2" to five digits, which is what this check is for.
    let first = ratios[0];
    for (k, r) in ratios.iter().enumerate() {
        assert!(
            (r - first).abs() <= 1e-5,
            "the lambda_min/Delta ratio is not constant across N: {ratios:?} (index {k})"
        );
    }
    assert!((first - 2.0).abs() <= 1e-5, "expected a factor of exactly 2, got {first}");
}

/// A1/H1, as an executable statement rather than an argument: on the broken branch the chart's
/// own MP2 audit reports health while the true error is large. A self-consistent lie audits
/// clean, and this is the fact criterion C3 exists to cover.
#[test]
fn the_broken_branch_audits_clean_while_lying() {
    use q_seam::audit::Mp2Audit;
    use q_seam::observables::ExactObservables;
    let n = 8;
    let h = Hubbard::new(n, 1.0, q_seam::PLANT_U);
    let g = ground_state(&h).unwrap();
    let o = ExactObservables::measure(&h, &g.vector);
    let c = Chart::best(n, 1.0, q_seam::PLANT_U).unwrap();
    let a = Mp2Audit::of(&c);

    let true_energy_err = (c.energy - g.energy).abs() / n as f64;
    assert!(
        a.energy_per_site < 0.1 * true_energy_err,
        "the plant's self-audit should be an order of magnitude too small: audit {} vs true {}",
        a.energy_per_site,
        true_energy_err
    );
    assert!(
        a.d_bool < 0.01 * o.d_bool,
        "the plant's D_bool audit should be two orders too small: audit {} vs true {}",
        a.d_bool,
        o.d_bool
    );
    // And the audit is inside kappa*tau on every observable, i.e. C1 certifies the plant.
    let kt: Vec<f64> = q_seam::TAU.iter().map(|t| q_seam::KAPPA * t).collect();
    for (k, (est, lim)) in a.as_vector().iter().zip(kt.iter()).enumerate() {
        assert!(est <= lim, "observable {k}: audit {est} exceeds kappa*tau {lim}");
    }
}
