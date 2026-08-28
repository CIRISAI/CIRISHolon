//! R1: the dual-route gate on the determinant FCI, plus the two invariances the whole
//! construction rests on.
//!
//! # Three routes, and what each one can see that the others cannot
//!
//! * [`FciSpace::sigma`] is production: the Knowles–Handy string factorisation.
//! * [`FciSpace::sigma_reference`] enumerates connected determinants and applies the
//!   Slater–Condon rules pair by pair. It shares no loop, no intermediate and no
//!   factorisation with the first, so agreement is evidence about the factorisation — and
//!   it runs at FULL SIZE, which an eigenvalue comparison against a dense route cannot.
//! * [`dense_hamiltonian_ladder`] builds the matrix from raw creation and annihilation
//!   operators, applying every fermionic sign one at a time, with no Slater–Condon rule
//!   anywhere. That is what checks the RULES the other two share. It costs `N_det * n^4`
//!   and forms a dense matrix, so it runs only on the small spaces.
//!
//! The gate is that all three agree, and the three together close the loop: a mistyped
//! Slater–Condon rule would be invisible to the first two and a broken factorisation
//! would be invisible to the third.
//!
//! # Why a MATRIX-VECTOR comparison and not only eigenvalues
//!
//! Because eigenvalues are blind to the failure this code actually had. The string
//! formalism orders spin orbitals in alpha and beta blocks; an interleaved ordering gives
//! the same Hamiltonian conjugated by a diagonal matrix of signs — identical spectrum,
//! different matrix. Every eigenvalue check passed while `sigma` and `sigma_reference`
//! disagreed on `H c` by 50%. See the convention note at the top of `fci.rs`.

use holon_chem::dual::D2;
use holon_chem::elements::{Species, CARBON, FLUORINE, HYDROGEN, LITHIUM, NITROGEN, OXYGEN};
use holon_chem::fci::{
    ci_ints, cholesky_orthonormaliser, dense_hamiltonian_ladder, jacobi_eigh, solve, transform,
    FciSpace, Order,
};
use holon_chem::md::ao_integrals;
use holon_chem::pair::{build_basis, electron_counts, pair_point};

/// A deterministic pseudo-random vector. Deterministic so a failure reproduces;
/// pseudo-random so the comparison is not accidentally made on a vector both routes
/// happen to handle the same way (a unit vector, for instance, tests one column).
fn probe_vector(n: usize, seed: u64) -> Vec<f64> {
    let mut s = seed | 1;
    (0..n)
        .map(|_| {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((s >> 33) as f64 / (1u64 << 30) as f64) - 1.0
        })
        .collect()
}

/// A deterministic orthogonal matrix, by Gram–Schmidt on a pseudo-random one.
fn orthogonal(n: usize, seed: u64) -> Vec<f64> {
    let mut m = probe_vector(n * n, seed);
    for p in 0..n {
        for q in 0..p {
            let d: f64 = (0..n).map(|i| m[i * n + p] * m[i * n + q]).sum();
            for i in 0..n {
                m[i * n + p] -= d * m[i * n + q];
            }
        }
        let norm: f64 = (0..n).map(|i| m[i * n + p] * m[i * n + p]).sum::<f64>().sqrt();
        for i in 0..n {
            m[i * n + p] /= norm;
        }
    }
    m
}

struct Case {
    label: &'static str,
    species: Vec<Species>,
    r: Option<f64>,
}

fn cases() -> Vec<Case> {
    vec![
        Case { label: "H atom", species: vec![HYDROGEN], r: None },
        Case { label: "C atom", species: vec![CARBON], r: None },
        Case { label: "N atom", species: vec![NITROGEN], r: None },
        Case { label: "O atom", species: vec![OXYGEN], r: None },
        Case { label: "H2", species: vec![HYDROGEN, HYDROGEN], r: Some(1.4) },
        Case { label: "LiH", species: vec![LITHIUM, HYDROGEN], r: Some(3.0) },
        Case { label: "HF", species: vec![HYDROGEN, FLUORINE], r: Some(1.8) },
        Case { label: "F2", species: vec![FLUORINE, FLUORINE], r: Some(2.6) },
    ]
}

fn centers(r: Option<f64>) -> Vec<[D2; 3]> {
    match r {
        None => vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]],
        Some(r) => vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::var(r)],
        ],
    }
}

#[test]
fn the_orthonormaliser_orthonormalises() {
    for case in cases() {
        let basis = build_basis(&case.species, centers(case.r));
        let n = basis.n;
        let ao = ao_integrals(&basis);
        let x = cholesky_orthonormaliser(&ao.s, n).expect("overlap not positive definite");
        let mut worst = 0.0f64;
        for p in 0..n {
            for q in 0..n {
                let mut acc = 0.0f64;
                for i in 0..n {
                    for j in 0..n {
                        acc += x[i * n + p].v * ao.s[i * n + j].v * x[j * n + q].v;
                    }
                }
                worst = worst.max((acc - if p == q { 1.0 } else { 0.0 }).abs());
            }
        }
        assert!(
            worst < 1e-12,
            "{}: X^T S X departs from the identity by {worst:.3e}",
            case.label
        );
    }
}

/// R1, leg one: the two sigma routes on the same vector, at FULL SIZE.
#[test]
fn the_two_sigma_routes_agree_on_a_general_vector() {
    for case in cases() {
        let basis = build_basis(&case.species, centers(case.r));
        let n = basis.n;
        let ao = ao_integrals(&basis);
        let x = cholesky_orthonormaliser(&ao.s, n).unwrap();
        let (_, na, nb) = electron_counts(&case.species);
        let space = FciSpace::new(n, na, nb);
        let mo = transform(&ao, &x, n);
        let ci = ci_ints(&mo, Order::Value);
        let v = probe_vector(space.n_det, 0x5eed);
        let mut a = vec![0.0; space.n_det];
        let mut b = vec![0.0; space.n_det];
        space.sigma(&ci, &v, &mut a);
        space.sigma_reference(&ci, &v, &mut b);
        let worst = a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).fold(0.0f64, f64::max);
        let scale = a.iter().map(|x| x.abs()).fold(0.0f64, f64::max);
        println!(
            "  {:>7} ({:>5} det): |sigma - sigma_ref| = {worst:.3e}, scale {scale:.3e}",
            case.label, space.n_det
        );
        assert!(
            worst <= 1e-11 * scale.max(1.0),
            "{}: the string factorisation and the Slater-Condon enumeration disagree on \
             H c by {worst:.3e} against a scale of {scale:.3e}",
            case.label
        );
    }
}

/// R1, leg two: the ladder-operator route, which uses no Slater–Condon rule at all.
#[test]
fn the_ladder_route_reproduces_the_iterative_ground_state() {
    for case in cases() {
        let basis = build_basis(&case.species, centers(case.r));
        let n = basis.n;
        let ao = ao_integrals(&basis);
        let x = cholesky_orthonormaliser(&ao.s, n).unwrap();
        let (_, na, nb) = electron_counts(&case.species);
        let space = FciSpace::new(n, na, nb);
        if space.n_det > 400 {
            // The dense route is O(N_det^2) in memory and O(N_det n^4) in time. Above
            // this it is not a checker, it is a wait — and the full-size leg above is
            // what covers the large spaces.
            println!("  {:>7}: {} determinants, dense route skipped", case.label, space.n_det);
            continue;
        }
        let mo = transform(&ao, &x, n);
        let ci = ci_ints(&mo, Order::Value);
        let dense = dense_hamiltonian_ladder(&space, &ci, n);
        let (evals, _) = jacobi_eigh(&dense, space.n_det);
        let iterative = solve(&space, &mo);
        let delta = (evals[0] - iterative.e.v).abs();
        println!(
            "  {:>7} ({:>5} det): ladder {:.12}, Davidson {:.12}, delta {delta:.3e}",
            case.label, space.n_det, evals[0], iterative.e.v
        );
        assert!(
            delta < 1e-9,
            "{}: the raw ladder-operator Hamiltonian's lowest eigenvalue differs from the \
             iterative solve by {delta:.3e}",
            case.label
        );
        // The ladder matrix must also be symmetric — it is built column by column from
        // operator strings, so symmetry is a consequence rather than an assumption.
        let mut asym = 0.0f64;
        for i in 0..space.n_det {
            for j in 0..space.n_det {
                asym = asym.max((dense[i * space.n_det + j] - dense[j * space.n_det + i]).abs());
            }
        }
        assert!(asym < 1e-12, "{}: the ladder Hamiltonian is not symmetric ({asym:.3e})", case.label);
    }
}

/// The invariance the whole differentiation strategy rests on.
///
/// Full CI does not care which orthonormal orbitals span its space, so the energy must be
/// the same through the raw Gram–Schmidt basis and through an arbitrary rotation of it.
/// That is what licenses the constant `U` in `pair.rs` to carry no derivative — if this
/// failed, every force and curvature this crate produces would be wrong.
#[test]
fn the_energy_does_not_depend_on_the_orbital_basis() {
    for case in cases() {
        let basis = build_basis(&case.species, centers(case.r));
        let n = basis.n;
        let ao = ao_integrals(&basis);
        let x = cholesky_orthonormaliser(&ao.s, n).unwrap();
        let (_, na, nb) = electron_counts(&case.species);
        let space = FciSpace::new(n, na, nb);

        let mut energies = Vec::new();
        for seed in [0u64, 0xabcd, 0x1234_5678] {
            let mut c = vec![D2::c(0.0); n * n];
            if seed == 0 {
                c.copy_from_slice(&x);
            } else {
                let u = orthogonal(n, seed);
                for i in 0..n {
                    for p in 0..n {
                        let mut acc = D2::c(0.0);
                        for m in 0..n {
                            acc = acc + x[i * n + m] * u[m * n + p];
                        }
                        c[i * n + p] = acc;
                    }
                }
            }
            let mo = transform(&ao, &c, n);
            let sol = solve(&space, &mo);
            assert!(
                sol.residual < 1e-8,
                "{}: the solve under rotation {seed:#x} did not converge (residual \
                 {:.3e}); the invariance cannot be read off an unconverged eigenvalue",
                case.label,
                sol.residual
            );
            energies.push(sol.e.v);
        }
        let spread = energies
            .iter()
            .map(|e| (e - energies[0]).abs())
            .fold(0.0f64, f64::max);
        println!("  {:>7}: rotation spread {spread:.3e} hartree", case.label);
        assert!(
            spread < 1e-9,
            "{}: rotating the orbitals moved the full-CI energy by {spread:.3e} hartree, \
             which it cannot do — either the CI space is not full or the transformation \
             is wrong",
            case.label
        );
    }
}

/// The analytic derivatives against a fourth-order finite difference of the ENERGY.
///
/// The finite difference is the reference here and it is the crude one: its own truncation
/// error sets the agreement, which is why the tolerances below are far looser than the
/// analytic values deserve. What this catches is a wrong derivative, not a last-bit one —
/// and a wrong derivative is what a response equation with a sign error produces.
#[test]
fn the_analytic_derivatives_match_a_finite_difference() {
    let h = 1e-3;
    for (label, a, b, r) in [
        ("H2", HYDROGEN, HYDROGEN, 1.2),
        ("LiH", LITHIUM, HYDROGEN, 2.9),
        ("HF", HYDROGEN, FLUORINE, 1.9),
        ("F2", FLUORINE, FLUORINE, 2.7),
    ] {
        let p = pair_point(a, b, r);
        let e = |x: f64| pair_point(a, b, x).e;
        let (m2, m1, p1, p2) = (e(r - 2.0 * h), e(r - h), e(r + h), e(r + 2.0 * h));
        let d1 = (m2 - 8.0 * m1 + 8.0 * p1 - p2) / (12.0 * h);
        let d2 = (-m2 + 16.0 * m1 - 30.0 * p.e + 16.0 * p1 - p2) / (12.0 * h * h);
        println!(
            "  {label}: F {:.9} vs {:.9} (|d| {:.2e}); E'' {:.9} vs {:.9} (|d| {:.2e})",
            p.f, -d1, (p.f + d1).abs(), p.e2, d2, (p.e2 - d2).abs()
        );
        assert!(
            (p.f + d1).abs() < 1e-8,
            "{label}: the analytic force and the finite difference differ by {:.3e}",
            (p.f + d1).abs()
        );
        assert!(
            (p.e2 - d2).abs() < 1e-5,
            "{label}: the analytic curvature and the finite difference differ by {:.3e}",
            (p.e2 - d2).abs()
        );
    }
}

/// The diagonal the preconditioner uses must be the Hamiltonian's actual diagonal.
///
/// # Why this is worth a gate even though a wrong diagonal cannot change the answer
///
/// It cannot: Davidson's eigenvalues come from the real `sigma`, and a bad preconditioner
/// only costs iterations. That is precisely why it needs its own test — a wrong diagonal
/// is INVISIBLE in every energy this crate reports, and would show up only as a solve
/// that is mysteriously slow, or as a response equation whose preconditioner points the
/// wrong way. The check is exact: the `i`th diagonal is `sigma` applied to the `i`th unit
/// vector, read at `i`.
#[test]
fn the_stored_diagonal_is_the_hamiltonians_own() {
    for case in cases() {
        let basis = build_basis(&case.species, centers(case.r));
        let n = basis.n;
        let ao = ao_integrals(&basis);
        let x = cholesky_orthonormaliser(&ao.s, n).unwrap();
        let (_, na, nb) = electron_counts(&case.species);
        let space = FciSpace::new(n, na, nb);
        let mo = transform(&ao, &x, n);
        let ci = ci_ints(&mo, Order::Value);
        let diag = space.diagonal(&ci);
        assert_eq!(diag.len(), space.n_det);

        // Every determinant for the small spaces; a deterministic spread for the large
        // ones, because 14 400 sigma calls is a wait rather than a test.
        let probes: Vec<usize> = if space.n_det <= 64 {
            (0..space.n_det).collect()
        } else {
            (0..32).map(|k| k * space.n_det / 32).collect()
        };
        let mut unit = vec![0.0f64; space.n_det];
        let mut out = vec![0.0f64; space.n_det];
        let mut worst = 0.0f64;
        for &i in probes.iter() {
            unit[i] = 1.0;
            space.sigma(&ci, &unit, &mut out);
            worst = worst.max((out[i] - diag[i]).abs());
            unit[i] = 0.0;
        }
        let scale = diag.iter().map(|d| d.abs()).fold(0.0f64, f64::max);
        println!(
            "  {:>7}: worst |diag - (H e_i)_i| = {worst:.3e} against a scale of {scale:.3e}",
            case.label
        );
        assert!(
            worst <= 1e-10 * scale.max(1.0),
            "{}: the stored diagonal differs from the Hamiltonian's by {worst:.3e}",
            case.label
        );
    }
}
