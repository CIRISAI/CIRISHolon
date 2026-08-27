//! `Q_SEAM_PREREG.md` §3 — the exactness ladder, as executable gates.
//!
//! These run before anything downstream. Each test names the gate it discharges and uses the
//! staked threshold verbatim; none of them may be relaxed to make a later result work.

use q_seam::dense::jacobi;
use q_seam::hubbard::{
    dimer_double_occupancy, dimer_energy, free_chain_energy_per_site, free_chain_gap, hop_sign,
    Hubbard, SpinBasis,
};
use q_seam::lanczos::ground_state;

/// G-E1 — Hamiltonian Hermiticity, bitwise. Staked: exactly 0.0.
#[test]
fn g_e1_hopping_is_bitwise_symmetric() {
    for &n in &[2usize, 4, 6, 8, 10] {
        let h = Hubbard::new(n, 1.0, 3.0);
        let asym = h.hop.asymmetry();
        assert_eq!(asym, 0.0, "N={n}: hopping asymmetry {asym:e} is not bitwise zero");
    }
}

/// G-E2 — every matrix element connects identical `(N_up, N_dn)`. Integer check, exact.
///
/// The sector is conserved by construction here (the basis holds only fixed-popcount masks), so
/// the honest form of this gate is that every hop lands on a mask of the same popcount and that
/// the basis index lookup never falls off the end.
#[test]
fn g_e2_every_hop_conserves_the_sector() {
    for &n in &[2usize, 4, 6, 8, 10] {
        let basis = SpinBasis::new(n, n / 2);
        let mut violations = 0usize;
        for &mask in &basis.masks {
            for i in 0..n - 1 {
                for (a, b) in [(i, i + 1), (i + 1, i)] {
                    if let Some((new_mask, _)) = hop_sign(mask, a, b) {
                        if new_mask.count_ones() != mask.count_ones() {
                            violations += 1;
                        }
                        if basis.index(new_mask) == usize::MAX {
                            violations += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(violations, 0, "N={n}: {violations} sector violations");
    }
}

/// G-E3 — fermionic signs: the general Jordan–Wigner path against the adjacency shortcut.
///
/// The prereg claims nearest-neighbour hops on a chain carry no JW sign. This checks that claim
/// on every state of every chain rather than taking it, which is the whole point of the gate.
#[test]
fn g_e3_nearest_neighbour_hops_carry_no_jw_sign() {
    for &n in &[2usize, 4, 6, 8, 10] {
        let basis = SpinBasis::new(n, n / 2);
        for &mask in &basis.masks {
            for i in 0..n - 1 {
                for (a, b) in [(i, i + 1), (i + 1, i)] {
                    if let Some((_, sign)) = hop_sign(mask, a, b) {
                        assert_eq!(
                            sign, 1.0,
                            "N={n} mask={mask:b}: hop ({a},{b}) carried sign {sign}"
                        );
                    }
                }
            }
        }
        // And a non-adjacent hop must be able to carry -1, or the sign machinery is dead code
        // that would pass the above vacuously.
        if n >= 4 {
            let mut saw_negative = false;
            for &mask in &basis.masks {
                if let Some((_, s)) = hop_sign(mask, 0, 3) {
                    if s < 0.0 {
                        saw_negative = true;
                    }
                }
            }
            assert!(saw_negative, "N={n}: the JW string never fired; the gate is vacuous");
        }
    }
}

/// G-E4c — this crate's dense Jacobi against the vacuum tier's `jacobi_eigen`.
///
/// The substitution (const-generic stack solver → heap solver) is checked, not trusted.
#[test]
fn g_e4c_dense_solver_matches_the_vacuum_tier_solver() {
    use ciris_sim_core::linalg::jacobi_eigen;

    let m3 = [[2.0, -1.0, 0.3], [-1.0, 1.5, 0.7], [0.3, 0.7, -0.5]];
    let core3 = jacobi_eigen(&m3);
    let mine3 = jacobi(m3.iter().flatten().copied().collect(), 3);
    for i in 0..3 {
        let d = (core3.values[i] - mine3.values[i]).abs();
        assert!(d <= 1e-14, "3x3 eigenvalue {i}: |Δ| = {d:e}");
    }

    let m4 = [
        [1.0, 0.4, -0.2, 0.1],
        [0.4, -2.0, 0.6, 0.0],
        [-0.2, 0.6, 3.0, -0.9],
        [0.1, 0.0, -0.9, 0.25],
    ];
    let core4 = jacobi_eigen(&m4);
    let mine4 = jacobi(m4.iter().flatten().copied().collect(), 4);
    for i in 0..4 {
        let d = (core4.values[i] - mine4.values[i]).abs();
        assert!(d <= 1e-14, "4x4 eigenvalue {i}: |Δ| = {d:e}");
    }
}

/// G-E4a — the Lanczos eigen-residual gate, at every N of the sweep and across U.
#[test]
fn g_e4a_lanczos_residual_gate() {
    for &n in &[2usize, 4, 6, 8] {
        for &u in &[0.0, 1.0, 4.0, 16.0] {
            let h = Hubbard::new(n, 1.0, u);
            let g = ground_state(&h).unwrap_or_else(|| panic!("N={n} U={u}: no ground state"));
            assert!(
                g.residual <= 1e-12,
                "N={n} U={u}: residual {:e} exceeds the staked 1e-12",
                g.residual
            );
            assert!(
                g.overlap_start >= 1e-8,
                "N={n} U={u}: start overlap {:e} below the guard",
                g.overlap_start
            );
        }
    }
}

/// G-E4b (AMENDED by A2/T1) — at N ≤ 6, an independent dense diagonalization must agree with
/// Lanczos, compared through the dense eigenvector's **Rayleigh quotient**.
///
/// As frozen this gate demanded 1e-14 on the raw dense eigenvalue and **it fired**: 9.6e-14 at
/// N = 6. The analytic arbiter showed the dense side was at 1.07e-13 (cyclic Jacobi's `O(n)·ε`
/// accumulation at dim 400) against Lanczos's 1.11e-14, so the miss was a mis-staked threshold,
/// not a bad ground state. The Rayleigh quotient is second-order in the eigenvector error and
/// removes that accumulation; the raw comparison is kept below as a reported diagnostic.
#[test]
fn g_e4b_dense_and_lanczos_agree_at_small_n() {
    for &n in &[2usize, 4, 6] {
        for &u in &[0.0, 1.0, 4.0, 16.0] {
            let h = Hubbard::new(n, 1.0, u);
            let d = h.dim();
            let eig = jacobi(h.to_dense(), d);
            let g = ground_state(&h).unwrap();

            let v = &eig.vectors[0];
            let mut hv = vec![0.0; d];
            h.apply(v, &mut hv);
            let num: f64 = hv.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
            let den: f64 = v.iter().map(|x| x * x).sum();
            let rq = num / den;

            let rel = (rq - g.energy).abs() / g.energy.abs().max(1.0);
            let raw = (eig.values[0] - g.energy).abs() / g.energy.abs().max(1.0);
            assert!(
                rel <= 1e-13,
                "N={n} U={u}: Rayleigh quotient {rq} vs Lanczos {} (rel {rel:e}, raw dense {raw:e})",
                g.energy
            );
        }
    }
}

/// G-E6 — the in-sector gap, so that "the ground state" is well defined and the symmetry
/// residuals are meaningful. Staked: `E1 - E0 >= 1e-6 t`.
#[test]
fn g_e6_ground_state_is_non_degenerate_in_sector() {
    for &n in &[2usize, 4, 6] {
        for &u in &[0.0, 1.0, 4.0, 16.0] {
            let h = Hubbard::new(n, 1.0, u);
            let eig = jacobi(h.to_dense(), h.dim());
            let gap = eig.values[1] - eig.values[0];
            assert!(gap >= 1e-6, "N={n} U={u}: in-sector gap {gap:e} below 1e-6 t");
        }
    }
}

/// G-E7 — the `U = 0` column against the §1.1(i) closed form.
///
/// Amendment A1/F: at N = 10 this is the only validation that reaches dimension 63 504; the
/// 3x3/4x4 fixtures of G-E4c cannot.
#[test]
fn g_e7_free_column_matches_the_closed_form() {
    for &n in &[2usize, 4, 6, 8, 10] {
        let h = Hubbard::new(n, 1.0, 0.0);
        let g = ground_state(&h).unwrap();
        let predicted = free_chain_energy_per_site(n) * n as f64;
        let rel = (g.energy - predicted).abs() / predicted.abs();
        assert!(
            rel <= 1e-12,
            "N={n}: E(U=0) = {} vs closed form {predicted} (rel {rel:e})",
            g.energy
        );
    }
}

/// The closed forms themselves, against the prereg's printed six-decimal table. If this fails,
/// the table in the frozen document is wrong and the document is what must be corrected.
#[test]
fn the_prereg_table_is_reproduced() {
    let expected: [(usize, f64, f64); 5] = [
        (2, -1.000000, 2.000000),
        (4, -1.118034, 1.236068),
        (6, -1.164653, 0.890084),
        (8, -1.189693, 0.694593),
        (10, -1.205335, 0.569259),
    ];
    for (n, e_per_site, gap) in expected {
        assert!((free_chain_energy_per_site(n) - e_per_site).abs() < 5e-7, "N={n} energy");
        assert!((free_chain_gap(n) - gap).abs() < 5e-7, "N={n} gap");
    }
}

/// G-E8 — the whole N = 2 column, all 14 U, against the §1.1(ii) analytic dimer.
///
/// Double occupancy is compared against Hellmann–Feynman, so this gates the observable machinery
/// as well as the energy.
#[test]
fn g_e8_dimer_column_matches_the_closed_form() {
    for &u in &q_seam::SWEEP_U {
        let h = Hubbard::new(2, 1.0, u);
        let g = ground_state(&h).unwrap();
        let predicted = dimer_energy(1.0, u);
        let d = (g.energy - predicted).abs();
        assert!(d <= 1e-12, "U={u}: dimer E = {} vs closed form {predicted} (|Δ| {d:e})", g.energy);

        // Double occupancy straight off the wavefunction, against Hellmann–Feynman.
        let n = h.n_conf();
        let mut docc = 0.0;
        for iu in 0..n {
            for id in 0..n {
                let amp = g.vector[iu * n + id];
                docc += amp * amp * f64::from(h.double_occ[iu * n + id]);
            }
        }
        let dd = (docc - dimer_double_occupancy(1.0, u)).abs();
        assert!(dd <= 1e-10, "U={u}: double occupancy {docc} vs analytic (|Δ| {dd:e})");
    }
}

/// G-E5a–d — the symmetry residuals of the exact ground state.
///
/// These are the exactness gates AND the exact side of observables O3/O4: the same theorems that
/// certify the solver are the ones that will convict a symmetry-broken chart. Staked: 1e-11 on
/// the three density residuals, 1e-10 on `⟨S²⟩`.
#[test]
fn g_e5_symmetry_residuals() {
    use q_seam::observables::ExactObservables;
    for &n in &[2usize, 4, 6, 8] {
        for &u in &[0.0, 1.0, 4.0, 16.0] {
            let h = Hubbard::new(n, 1.0, u);
            let g = ground_state(&h).unwrap();
            let o = ExactObservables::measure(&h, &g.vector);

            // G-E5a: particle-hole gives <n_isigma> = 1/2 exactly.
            let ph = o.occupation[0]
                .iter()
                .chain(o.occupation[1].iter())
                .map(|x| (x - 0.5).abs())
                .fold(0.0, f64::max);
            assert!(ph <= 1e-11, "N={n} U={u}: particle-hole residual {ph:e}");

            // G-E5b: spin-flip gives m_i = 0 exactly.
            let sf = o.magnetization.iter().map(|x| x.abs()).fold(0.0, f64::max);
            assert!(sf <= 1e-11, "N={n} U={u}: spin-flip residual {sf:e}");

            // G-E5c: the open chain is reflection symmetric.
            let refl = (0..n)
                .map(|i| (o.density[i] - o.density[n - 1 - i]).abs())
                .fold(0.0, f64::max);
            assert!(refl <= 1e-11, "N={n} U={u}: reflection residual {refl:e}");

            // G-E5d: Lieb's theorem, measured rather than assumed.
            assert!(o.s_squared <= 1e-10, "N={n} U={u}: <S^2> = {:e}", o.s_squared);
        }
    }
}

/// The Boolean defect is exactly zero where the chart is exact, and large at the plant.
///
/// Not a staked gate — a construction check that `D_bool` is measuring what §2.1 says it does.
/// If `D_bool(U=0)` were nonzero the ModeChart fence would be mis-implemented.
#[test]
fn d_bool_is_zero_at_u_zero_and_large_at_the_plant() {
    use q_seam::observables::ExactObservables;
    for &n in &[2usize, 4, 6, 8] {
        let h0 = Hubbard::new(n, 1.0, 0.0);
        let g0 = ground_state(&h0).unwrap();
        let o0 = ExactObservables::measure(&h0, &g0.vector);
        assert!(o0.d_bool <= 1e-10, "N={n}: D_bool(U=0) = {:e}, chart is not exact there", o0.d_bool);

        let hp = Hubbard::new(n, 1.0, q_seam::PLANT_U);
        let gp = ground_state(&hp).unwrap();
        let op = ExactObservables::measure(&hp, &gp.vector);
        assert!(
            op.d_bool > q_seam::TAU[5],
            "N={n}: plant D_bool = {} is inside its own tolerance",
            op.d_bool
        );
    }
}

/// **TIMING GATE** — the cost shape must not come back.
///
/// The re-solve defect (rebuilding and re-diagonalizing the full `m x m` tridiagonal every
/// iteration) was invisible through Q5/Q7/Q7b because `N <= 10` kept the absolute cost small; it
/// became dominant at Q8's `N = 12`. This gate is placed where it actually DISCRIMINATES.
///
/// Measured on this box, same build, same case (N=8, U=16, 180 iterations):
///   CHECK_EVERY = 1 (the original cost shape): 3.779 s
///   CHECK_EVERY = 10 (the fix):                1.326 s
/// Ceiling staked at 2.5 s: the fix clears it with 1.9x headroom and the defect overruns it by 51%.
///
/// NOTE, and it is a deviation from the commissioned case: `N = 10, U = 0` was suggested, but it
/// converges in 72 iterations where the two builds measure 418 ms and 454 ms — it does not
/// discriminate at all, so a gate there would be decorative. The N=10/U=0 timing is asserted too,
/// at a loose ceiling, but it is the N=8/U=16 clause that can actually fail.
#[test]
fn timing_gate_the_resolve_cost_shape_stays_fixed() {
    use std::time::Instant;

    let t0 = Instant::now();
    let h = Hubbard::new(8, 1.0, 16.0);
    let g = ground_state(&h).unwrap();
    let elapsed = t0.elapsed();
    assert_eq!(g.iterations, 180, "the reference stopped at a different size than measured");
    assert!(
        elapsed.as_secs_f64() <= 2.5,
        "N=8 U=16 took {elapsed:?}; the per-iteration re-solve cost shape may have returned \
         (fix measured 1.326 s, defect measured 3.779 s)"
    );

    let t1 = Instant::now();
    let h2 = Hubbard::new(10, 1.0, 0.0);
    let g2 = ground_state(&h2).unwrap();
    let e2 = t1.elapsed();
    assert_eq!(g2.iterations, 72);
    assert!(e2.as_secs_f64() <= 10.0, "N=10 U=0 reference took {e2:?}");
}

/// The cadence is a COST change, not a mathematics change: the loop must stop at exactly the size
/// the original per-iteration test would have stopped at, and return that Ritz pair.
///
/// These stop sizes and energies were captured from a `CHECK_EVERY = 1` build — which IS the
/// original algorithm — and are reproduced bit-for-bit by the shipped `CHECK_EVERY = 10`.
#[test]
fn the_cadence_reproduces_the_original_stop_sizes_and_energies() {
    let cases: [(usize, f64, usize, f64); 5] = [
        (8, 0.0, 56, -9.51754096628741131),
        (8, 16.0, 180, -1.26213613233428346),
        (10, 0.0, 72, -12.05334836666469300),
        (10, 4.0, 109, -5.38061882041472561),
        (10, 16.0, 246, -1.60278502194068184),
    ];
    for (n, u, iters, energy) in cases {
        let h = Hubbard::new(n, 1.0, u);
        let g = ground_state(&h).unwrap();
        assert_eq!(g.iterations, iters, "N={n} U={u}: stop size moved");
        assert_eq!(
            g.energy, energy,
            "N={n} U={u}: energy is not bit-identical to the pre-fix build"
        );
    }
}
