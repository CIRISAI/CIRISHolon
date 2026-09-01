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
use holon_chem::elements::{
    Species, BERYLLIUM, BORON, CARBON, FLUORINE, GERMANIUM, HELIUM, HYDROGEN, LITHIUM, NEON,
    NITROGEN, OXYGEN,
};
use holon_chem::fci::{
    ci_ints, cholesky_orthonormaliser, davidson_eigh, dense_hamiltonian_ladder, jacobi_eigh,
    s_squared, sigma_direct, solve, solve_mps, transform, FciSpace, Order,
};
use holon_chem::md::ao_integrals;
use holon_chem::pair::{build_basis, electron_counts, geometry_problem, pair_point};

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

/// THE SPIN SECTOR, MEASURED. The converged state must be a spin eigenstate, and the
/// RIGHT one.
///
/// # Why an assumption was not enough here
///
/// Every diatomic and atom in this crate is solved in the MINIMAL `S_z` sector, on the
/// argument that a multiplet of total spin `S` has a component in every sector with
/// `|S_z| <= S`, so the smallest one contains every state whatever its spin. That argument
/// is sound and it is about the SPACE. It says nothing about which state the solver
/// returned — and `H` commutes with `S^2`, so a Krylov space started from one determinant
/// can converge cleanly, with a tiny residual and two independent sigma routes agreeing,
/// to the lowest state of the WRONG multiplet.
///
/// That is not hypothetical: this crate did it. Under a rotated orbital basis, carbon came
/// back converged 0.07 hartree above its ground state, and `davidson`'s generic start
/// vector was added to break the sector. But a perturbation is a mitigation — nothing
/// checked that it worked, and a future edit could remove it silently. So the sector is
/// measured now, and CARBON IS THE DISCRIMINATING CASE: its `S_z = 0` sector holds both
/// the singlet and the `S_z = 0` component of the triplet, so a solver in the wrong one
/// reads 0 where the truth is 2.
///
/// The check also earns something the crate did not have: the multiplicity as a DERIVED
/// quantity. Nothing here declares that nitrogen is a quartet — it comes out of the
/// converged vector.
#[test]
fn the_converged_state_is_in_the_right_spin_sector() {
    use holon_chem::elements::{BERYLLIUM, BORON, HELIUM, NEON};
    use holon_chem::fci::{multiplicity, s_squared};

    // Expected S(S+1) for each ground state. The ATOMS are the periodic table's own
    // multiplicities and the DIATOMICS staked here are all closed-shell singlets; neither
    // is computed by this crate, which is what makes them a check on it.
    let expected: Vec<(&str, Vec<Species>, Option<f64>, f64)> = vec![
        ("H  doublet", vec![HYDROGEN], None, 0.75),
        ("He singlet", vec![HELIUM], None, 0.0),
        ("Li doublet", vec![LITHIUM], None, 0.75),
        ("Be singlet", vec![BERYLLIUM], None, 0.0),
        ("B  doublet", vec![BORON], None, 0.75),
        ("C  TRIPLET", vec![CARBON], None, 2.0),
        ("N  quartet", vec![NITROGEN], None, 3.75),
        ("O  triplet", vec![OXYGEN], None, 2.0),
        ("F  doublet", vec![FLUORINE], None, 0.75),
        ("Ne singlet", vec![NEON], None, 0.0),
        ("H2 singlet", vec![HYDROGEN, HYDROGEN], Some(1.4), 0.0),
        ("LiH singlet", vec![LITHIUM, HYDROGEN], Some(3.0), 0.0),
        ("HF singlet", vec![HYDROGEN, FLUORINE], Some(1.8), 0.0),
        ("F2 singlet", vec![FLUORINE, FLUORINE], Some(2.6), 0.0),
    ];

    for (label, species, r, want) in expected {
        let basis = build_basis(&species, centers(r));
        let n = basis.n;
        let ao = ao_integrals(&basis);
        let x = cholesky_orthonormaliser(&ao.s, n).unwrap();
        let (n_elec, na, nb) = electron_counts(&species);
        let space = FciSpace::new(n, na, nb);
        let mo = transform(&ao, &x, n);
        let sol = solve(&space, &mo);
        assert!(sol.residual < 1e-8, "{label}: not converged ({:.3e})", sol.residual);

        // The vector Davidson returns is normalised; assert it rather than trusting it,
        // because <S^2> is a ratio to the norm and a drifted norm would fake a multiplet.
        let norm_sq: f64 = sol.vector.iter().map(|x| x * x).sum();
        assert!(
            (norm_sq - 1.0).abs() < 1e-10,
            "{label}: the CI vector is not normalised (|c|^2 = {norm_sq})"
        );

        let s_sq = s_squared(&space, &sol.vector);
        let (s, mult) = multiplicity(s_sq, n_elec, 1e-6).unwrap_or_else(|| {
            panic!(
                "{label}: <S^2> = {s_sq} is not S(S+1) for any half-integer S — the \
                 converged state is not a spin eigenstate, which is a broken solve rather \
                 than a chemistry result"
            )
        });
        println!("  {label:>11}: <S^2> = {s_sq:.12}  ->  S = {s}, multiplicity {mult}");
        assert!(
            (s_sq - want).abs() < 1e-6,
            "{label}: <S^2> = {s_sq:.9}, wanted {want} (multiplicity {mult}). The solve \
             converged in the wrong spin sector."
        );
    }
}

/// The spin check must be able to FAIL, or it is decoration — and the probe has to be
/// DERIVED rather than guessed.
///
/// # Two things this catches
///
/// A `s_squared` that returned the floor `S_z(S_z + 1)` unconditionally would pass every
/// row of the test above except carbon's, and one discriminating row is thin cover for a
/// whole gate. So a state that is provably NOT a spin eigenstate is fed in and must read
/// above the floor.
///
/// The first version of this probe took determinant index 0 and asserted a nonzero
/// reading. It failed — correctly. Index 0 of carbon's `S_z = 0` space is the CLOSED-SHELL
/// determinant, which is a perfectly good singlet, and `<S^2> = 0` was the right answer to
/// a question that tested nothing. Picking an index and assuming what it holds is the same
/// error as planting a defect without checking it is observable. So the determinant is now
/// derived from its occupation, and the expected value is derived too: for a single
/// determinant `|| S_+ D ||^2` counts the orbitals holding a beta electron and no alpha
/// one, so
///
/// ```text
/// <S^2> = (# beta-only orbitals) + S_z(S_z + 1)
/// ```
///
/// which is an exact integer this test can demand rather than a bound it can hope for.
#[test]
fn the_spin_check_rejects_a_state_that_is_not_an_eigenstate() {
    use holon_chem::fci::{multiplicity, s_squared};

    // Carbon: 6 electrons, 5 orbitals, S_z = 0. Floor 0, ground state 2.
    let basis = build_basis(&[CARBON], centers(None));
    let n = basis.n;
    let (n_elec, na, nb) = electron_counts(&[CARBON]);
    let space = FciSpace::new(n, na, nb);
    assert_eq!(space.alpha.n_elec, space.beta.n_elec, "S_z must be 0 for this probe");
    let nb_len = space.beta.len();

    // Find an OPEN-SHELL determinant: alpha and beta occupations that differ. Searched
    // rather than indexed, so the test cannot be silently testing the wrong object.
    let mut probes = Vec::new();
    for (ia, &ma) in space.alpha.masks.iter().enumerate() {
        for (ib, &mb) in space.beta.masks.iter().enumerate() {
            let beta_only = (mb & !ma).count_ones();
            if beta_only > 0 {
                probes.push((ia * nb_len + ib, beta_only));
            }
        }
    }
    assert!(
        probes.len() > 10,
        "carbon's S_z = 0 space should be full of open-shell determinants; found {}",
        probes.len()
    );

    let mut checked = 0usize;
    for &(det, beta_only) in probes.iter().take(24) {
        let mut single = vec![0.0f64; space.n_det];
        single[det] = 1.0;
        let s_sq = s_squared(&space, &single);
        let want = beta_only as f64; // + S_z(S_z+1), which is 0 here
        assert!(
            (s_sq - want).abs() < 1e-12,
            "a determinant with {beta_only} beta-only orbitals must read <S^2> = {want}, \
             got {s_sq}"
        );
        assert!(
            s_sq > 1e-6,
            "s_squared returned the S_z floor for a determinant that is not a spin \
             eigenstate; the check cannot distinguish sectors and every other assertion \
             about it is empty"
        );
        // The reader, on the two cases these determinants actually produce. One beta-only
        // orbital gives <S^2> = 1, which is not S(S+1) for any half-integer S and must be
        // refused; two gives exactly 2, a triplet, which carbon's six electrons can have.
        let read = multiplicity(s_sq, n_elec, 1e-9);
        match beta_only {
            1 => assert!(
                read.is_none(),
                "<S^2> = 1 is not S(S+1) for any half-integer S and must not read back as \
                 a multiplicity"
            ),
            2 => assert_eq!(
                read.map(|x| x.1),
                Some(3),
                "<S^2> = 2 is a triplet, and six electrons can have one"
            ),
            _ => {}
        }
        checked += 1;
    }
    println!("  {checked} open-shell determinants: <S^2> = (# beta-only orbitals), exactly");

    // The multiplicity reader, on values that are and are not S(S+1).
    // Valid readings, each with an electron count of the parity it requires.
    assert_eq!(multiplicity(0.0, 6, 1e-9).map(|x| x.1), Some(1), "singlet, even count");
    assert_eq!(multiplicity(0.75, 7, 1e-9).map(|x| x.1), Some(2), "doublet, odd count");
    assert_eq!(multiplicity(2.0, 6, 1e-9).map(|x| x.1), Some(3), "triplet, even count");
    assert_eq!(multiplicity(3.75, 7, 1e-9).map(|x| x.1), Some(4), "quartet, odd count");

    // Not S(S+1) for any half-integer S. F2's far tail reads exactly 1.0 here, which is
    // why the referee lane's doublet defect could not reproduce in this crate.
    assert!(multiplicity(1.0, 6, 1e-9).is_none(), "1.0 is not S(S+1) for any half-integer S");
    assert!(multiplicity(3.0, 6, 1e-9).is_none(), "3.0 is not S(S+1) either");

    // THE PARITY CONDITION, which is the one that is easy to leave out. Each of these IS a
    // spin eigenvalue and is still impossible for an electron count of that parity.
    assert!(
        multiplicity(0.75, 18, 1e-9).is_none(),
        "a doublet is impossible for eighteen electrons, however clean the value looks — \
         this is the reading that passed the sibling lane's agreement test"
    );
    assert!(multiplicity(0.0, 7, 1e-9).is_none(), "a singlet is impossible for seven electrons");
    assert!(multiplicity(2.0, 9, 1e-9).is_none(), "a triplet is impossible for nine electrons");
    assert!(multiplicity(3.75, 8, 1e-9).is_none(), "a quartet is impossible for eight electrons");
}

/// The multiplicity of the ground state along a whole CURVE, settled by DENSE
/// diagonalisation, and the separation at which it changes.
///
/// # The defect in this test's first version, which is the point of its second
///
/// It swept `<S^2>` along each curve and, where the `S_z = 0` and `S_z = 1` sector floors
/// coincided, declared the state "degenerate, spin unresolved". That inference is WRONG,
/// and wrong in this campaign's recurring way: it used a quantity that cannot tell two
/// causes apart, and then named one of them.
///
/// `E_min(S_z = 1) - E_min(S_z = 0)` is not a singlet–triplet gap. A spin-`S` multiplet
/// appears in every sector with `|S_z| <= S`, so a TRIPLET ground state sits at the floor
/// of both sectors and the difference is zero — not because the two states are degenerate,
/// but because they are the same state. The difference is therefore a detector for "is the
/// ground state a singlet", nothing more.
///
/// F2 is where that mattered. Its ground state genuinely CHANGES MULTIPLICITY: singlet
/// near equilibrium, triplet from about 4.7 bohr outward, because at long range the
/// two-centre exchange favours high-spin coupling between the two open-shell fluorines
/// while the bonding term favours the singlet at short range. The first version read the
/// triplet region as "unresolved" and asserted `<S^2> = 0` for F2 everywhere — an
/// assumption that is simply false, and which passed only because its grid stepped over
/// the crossing. The sibling `elements-referee` lane found the crossing independently, by
/// dense diagonalisation, and its own emitter had the mirror-image bug: a guard refusing
/// any species whose spin was not constant, which would have rejected F2 for being right.
///
/// # What this version does instead
///
/// Every species here is at most 225 determinants, so the question is settled rather than
/// inferred: the `S_z = 0` block is built from raw ladder operators and diagonalised
/// densely, `<S^2>` is taken of every eigenvector, and the ground multiplicity is read off
/// directly. No subspace method is involved, so nothing can be trapped in a spin sector.
///
/// Asserted: the iterative solve agrees with the dense ground energy at every separation
/// (a dual-route check along a whole curve, which the single-geometry test cannot give);
/// the dense ground state is a spin eigenstate; and each species is a SINGLET at its
/// equilibrium, which is the region every reported `R_e` and `D_e` comes from. The
/// crossing separation is REPORTED as the derived quantity it is, not asserted.
#[test]
fn the_ground_multiplicity_along_the_curve_is_settled_densely() {
    use holon_chem::fci::{dense_hamiltonian_ladder, multiplicity, s_squared};

    let sweeps: Vec<(&str, Species, Species, f64, Vec<f64>)> = vec![
        // H2 runs to 12 deliberately. The referee lane's beyond-grid probe measures H2 as
        // resolved out to 10 bohr and DEGENERATE BY 12 — a claim boundary two bohr past
        // where its staked grid stops. This sweep ended at 9 and so could not have seen
        // that: "resolved everywhere" would have been a fact about the sweep. Extended so
        // the two lanes' boundaries are comparable rather than each bounded by its own
        // design choice.
        ("H2", HYDROGEN, HYDROGEN, 1.4, vec![0.8, 1.4, 2.0, 3.0, 4.5, 6.0, 9.0, 10.0, 12.0]),
        ("LiH", LITHIUM, HYDROGEN, 2.92, vec![2.0, 2.92, 4.0, 6.0, 9.0, 14.0, 16.8]),
        ("HF", HYDROGEN, FLUORINE, 1.88, vec![1.2, 1.88, 2.6, 3.6, 5.0, 8.0, 9.7, 12.0]),
        // Dense through the crossing the referee lane located at 4.7277 bohr.
        ("F2", FLUORINE, FLUORINE, 2.62, vec![2.0, 2.62, 3.5, 4.4, 4.6, 4.8, 5.5, 8.0, 9.5, 12.0]),
    ];

    for (label, a, b, r_e, rs) in sweeps {
        let mut trail: Vec<(f64, usize)> = Vec::new();
        for &r in rs.iter() {
            let species = [a, b];
            let basis = build_basis(&species, centers(Some(r)));
            let n = basis.n;
            let ao = ao_integrals(&basis);
            let x = cholesky_orthonormaliser(&ao.s, n).unwrap();
            let mo = transform(&ao, &x, n);
            let (n_elec, na, nb) = electron_counts(&species);
            let space = FciSpace::new(n, na, nb);
            assert!(
                space.n_det <= 256,
                "{label}: {} determinants is too many to settle densely",
                space.n_det
            );

            // The whole block, from raw ladder operators. No subspace method anywhere, so
            // no spin sector can trap it.
            let ci = ci_ints(&mo, Order::Value);
            let dense = dense_hamiltonian_ladder(&space, &ci, n);
            let (evals, evecs) = jacobi_eigh(&dense, space.n_det);
            let ground: Vec<f64> = (0..space.n_det)
                .map(|row| evecs[row * space.n_det])
                .collect();
            let s_sq = s_squared(&space, &ground);

            // Whether the ground LEVEL is degenerate is a fact the dense spectrum already
            // holds, so it is read rather than inferred. A Hamiltonian commuting with S^2
            // has spin-eigenstate eigenvectors ONLY where its levels are simple: inside a
            // degenerate level any basis is an eigenbasis, and Jacobi returns an arbitrary
            // one, which need not be a spin eigenstate. F2 at 8 bohr reads
            // <S^2> = 1.99991 for exactly that reason -- 99.995% triplet with a trace of
            // singlet mixed in by the rotation order, not a defect and not a measurement.
            const LEVEL_SIMPLE: f64 = 1e-9;
            let level_gap = evals[1] - evals[0];
            let mult = if level_gap > LEVEL_SIMPLE {
                let (_, m) = multiplicity(s_sq, n_elec, 1e-6).unwrap_or_else(|| {
                    panic!(
                        "{label} at R = {r}: the DENSE ground eigenvector has \
                         <S^2> = {s_sq}, which is not S(S+1) for any half-integer S, and \
                         the level is SIMPLE (gap to the next state {level_gap:.3e}). A \
                         simple eigenvector of a Hamiltonian that commutes with S^2 must \
                         be a spin eigenstate."
                    )
                });
                m
            } else {
                // A degenerate level does not automatically defeat the question. Any basis
                // of it is an eigenbasis, so no single vector's <S^2> means anything by
                // itself — but if EVERY vector in the level reports the same multiplicity,
                // the level is spin-pure and the multiplicity is resolved regardless. That
                // is not a corner case here: F2 has degenerate pi orbitals, so its triplet
                // region is spatially two-fold and every vector in it reads exactly 2.
                // Only where the level actually spans different multiplicities -- F2's
                // four-fold level at 8 bohr, where singlet and triplet have come together
                // -- is the question genuinely unanswerable.
                assert!(
                    (0.0..=2.0 + 1e-6).contains(&s_sq),
                    "{label} at R = {r}: <S^2> = {s_sq} lies outside the singlet-triplet \
                     manifold the degenerate level spans"
                );
                let order = evals
                    .iter()
                    .take_while(|e| **e - evals[0] <= LEVEL_SIMPLE)
                    .count();
                let mults: Vec<Option<usize>> = (0..order)
                    .map(|k| {
                        let v: Vec<f64> = (0..space.n_det)
                            .map(|row| evecs[row * space.n_det + k])
                            .collect();
                        multiplicity(s_squared(&space, &v), n_elec, 1e-6).map(|x| x.1)
                    })
                    .collect();
                let agreed = mults[0].filter(|m| mults.iter().all(|x| x == &Some(*m)));
                match agreed {
                    Some(m) => {
                        println!(
                            "    {label} R = {r}: ground level {order}-fold degenerate \
                             (gap {level_gap:.2e}) but SPIN-PURE -- every vector reads \
                             multiplicity {m}, so it is resolved"
                        );
                        m
                    }
                    None => {
                        println!(
                            "    {label} R = {r}: ground level {order}-fold degenerate \
                             (gap {level_gap:.2e}) and spans multiplicities {mults:?} -- \
                             not resolved, not asserted"
                        );
                        0
                    }
                }
            };

            // The iterative solve must land on the same energy. This is the dual-route
            // check the single-geometry test makes, made along the whole curve — including
            // the tail, where it is hardest.
            let iterative = solve(&space, &mo);
            let delta = (iterative.e.v - evals[0]).abs();
            assert!(
                delta < 1e-9,
                "{label} at R = {r}: Davidson gives {:.12} against the dense {:.12}, a \
                 difference of {delta:.3e}. The iterative solve is not on the ground state.",
                iterative.e.v,
                evals[0]
            );
            trail.push((r, mult));
        }

        // Every species reported here is a singlet at equilibrium, and that is the region
        // every published R_e and D_e comes from.
        let at_eq = trail
            .iter()
            .find(|(r, _)| (*r - r_e).abs() < 1e-9)
            .unwrap_or_else(|| panic!("{label}: the sweep does not visit its equilibrium"));
        assert_eq!(
            at_eq.1, 1,
            "{label} at its equilibrium R = {}: ground multiplicity {} — every reported \
             well depth for this species assumes a singlet there",
            at_eq.0, at_eq.1
        );

        // Crossings are read only across separations where the multiplicity was actually
        // resolved; an unresolved point is a gap in the record, not a change in it.
        let resolved: Vec<(f64, usize)> = trail.iter().copied().filter(|(_, m)| *m > 0).collect();
        assert!(
            resolved.len() >= 3,
            "{label}: only {} separations resolved a multiplicity; too few to be a test",
            resolved.len()
        );
        let changes: Vec<String> = resolved
            .windows(2)
            .filter(|w| w[0].1 != w[1].1)
            .map(|w| format!("{} -> {} between {} and {} bohr", w[0].1, w[1].1, w[0].0, w[1].0))
            .collect();
        println!(
            "  {label:>4}: multiplicity along the curve {:?}{}",
            trail
                .iter()
                .map(|(r, m)| if *m > 0 { format!("{r}:{m}") } else { format!("{r}:?") })
                .collect::<Vec<_>>(),
            if changes.is_empty() {
                " — constant".to_string()
            } else {
                format!(" — CHANGES {}", changes.join(", "))
            }
        );
    }
}

#[test]
fn test_mps_solver_vs_fci_ground_state() {
    // H2 at R=1.4 bohr
    let basis = build_basis(&[HYDROGEN, HYDROGEN], centers(Some(1.4)));
    let n = basis.n;
    let ao = ao_integrals(&basis);
    let x = cholesky_orthonormaliser(&ao.s, n).expect("overlap not positive definite");
    let mo = transform(&ao, &x, n);
    let space = FciSpace::new(n, 1, 1);

    let sol_fci = solve(&space, &mo);
    let sol_mps = solve_mps(&space, &mo, 32);

    let e_diff = (sol_fci.e.v - sol_mps.e.v).abs();
    assert!(
        e_diff < 1e-6,
        "MPS energy {} differs from FCI energy {} by {e_diff:e}",
        sol_mps.e.v,
        sol_fci.e.v
    );

    let d_diff = (sol_fci.e.d - sol_mps.e.d).abs();
    assert!(
        d_diff < 1e-4,
        "MPS force {} differs from FCI force {} by {d_diff:e}",
        sol_mps.e.d,
        sol_fci.e.d
    );
}

#[test]
fn test_solve_routes_large_spaces_to_mps() {
    // 10 spatial orbitals, 5 alpha + 5 beta electrons:
    // (10 choose 5)^2 = 252^2 = 63,504 determinants (> 50,000)
    let n_orb = 10;
    let n_alpha = 5;
    let n_beta = 5;
    let space = FciSpace::new(n_orb, n_alpha, n_beta);
    assert_eq!(space.n_det, 63504);
    assert!(space.n_det > 50_000);

    // Dummy MO integrals for the 10-orbital space
    let mut h = vec![D2::c(0.0); n_orb * n_orb];
    for p in 0..n_orb {
        h[p * n_orb + p] = D2::new(-2.0 - 0.1 * p as f64, 0.05, 0.01);
    }
    let mut g = vec![D2::c(0.0); n_orb * n_orb * n_orb * n_orb];
    for p in 0..n_orb {
        g[(p * n_orb + p) * n_orb * n_orb + (p * n_orb + p)] = D2::c(0.5);
    }
    let mo = holon_chem::fci::MoIntegrals { n: n_orb, h, g };

    // solve() should automatically route to solve_mps() rather than attempting 63,504-dim Davidson
    let sol = solve(&space, &mo);
    assert!(sol.e.v.is_finite());
}

#[test]
fn test_sigma_direct_n2_and_atomic_ground_states() {
    let species_list: [(Species, f64); 10] = [
        (HYDROGEN, -0.46658184955727546),
        (HELIUM, -2.807783957539975),
        (LITHIUM, -7.315836576618031),
        (BERYLLIUM, -14.403655041094132),
        (BORON, -24.18926497514655),
        (CARBON, -37.21873352468449),
        (NITROGEN, -53.719010183796),
        (OXYGEN, -73.80415024579707),
        (FLUORINE, -97.98650500018774),
        (NEON, -126.60452505299895),
    ];

    for &(sp, ref_val) in species_list.iter() {
        let (space, mo, _) = geometry_problem(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        let ci0 = ci_ints(&mo, Order::Value);
        let diag = space.diagonal(&ci0);
        let (e, vec, _iters, resid) = davidson_eigh(&space, &ci0, &diag, 1e-12, 1200);
        assert!(resid < 1e-10, "{}: Davidson did not reach precision (resid = {resid:.3e})", sp.symbol);

        // Also test sigma_direct against sigma
        let mut sig1 = vec![0.0f64; space.n_det];
        let mut sig2 = vec![0.0f64; space.n_det];
        sigma_direct(&space, &ci0, &vec, &mut sig1);
        space.sigma(&ci0, &vec, &mut sig2);
        for (s1, s2) in sig1.iter().zip(sig2.iter()) {
            assert!((s1 - s2).abs() < 1e-13);
        }

        let diff = (e - ref_val).abs();
        println!(
            "  Atom {:>2}: E = {:>16.12} Ha (ref {:>16.12}), diff = {diff:.3e}, resid = {resid:.3e}, dets = {}",
            sp.symbol, e, ref_val, space.n_det
        );
        assert!(
            diff < 1e-12,
            "Atom {} ground state eigenvalue differs from referee by {diff:.3e} (limit 1e-12)",
            sp.symbol
        );
    }

    // Now test N_2 (14,400 determinants)
    let (n2_space, n2_mo, _) = geometry_problem(
        &[NITROGEN, NITROGEN],
        centers(Some(2.074)), // equilibrium separation ~ 2.074 bohr
    );
    assert_eq!(n2_space.n_det, 14400);
    let ci0 = ci_ints(&n2_mo, Order::Value);
    let diag = n2_space.diagonal(&ci0);
    let (e_n2, vec_n2, _iters, resid_n2) = davidson_eigh(&n2_space, &ci0, &diag, 1e-12, 1200);
    assert!(resid_n2 < 1e-10, "N_2 Davidson did not converge: resid = {resid_n2:.3e}");

    let mut sig_n2 = vec![0.0f64; 14400];
    sigma_direct(&n2_space, &ci0, &vec_n2, &mut sig_n2);
    let h_exp: f64 = vec_n2.iter().zip(sig_n2.iter()).map(|(a, b)| a * b).sum();
    let eig_diff = (h_exp - e_n2).abs();
    println!(
        "  N_2 (14,400 det): E = {:.12} Ha, <v|H|v> = {:.12}, diff = {eig_diff:.3e}, resid = {resid_n2:.3e}",
        e_n2, h_exp
    );
    assert!(
        eig_diff < 1e-12,
        "N_2 Rayleigh quotient <v|H|v> differs from eigenvalue by {eig_diff:.3e} (limit 1e-12)"
    );
}

#[test]
fn test_sigma_direct_germanium_ground_state() {
    let sp = GERMANIUM;
    assert_eq!(sp.z, 32);
    let (n_elec, na, nb) = electron_counts(&[sp]);
    assert_eq!(n_elec, 32);
    assert_eq!(na, 16);
    assert_eq!(nb, 16);

    let (space, mo, _) = geometry_problem(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
    assert_eq!(
        space.n_det, 23409,
        "Germanium STO-3G FCI space must have exactly 23,409 determinants (found {})",
        space.n_det
    );

    let ci0 = ci_ints(&mo, Order::Value);
    let diag = space.diagonal(&ci0);
    let (e, vec, iters, resid) = davidson_eigh(&space, &ci0, &diag, 1e-11, 1200);

    // Verify matrix-vector multiplication with sigma_direct
    let mut sig = vec![0.0f64; space.n_det];
    sigma_direct(&space, &ci0, &vec, &mut sig);
    let rayleigh: f64 = vec.iter().zip(sig.iter()).map(|(a, b)| a * b).sum();
    let rayleigh_diff = (rayleigh - e).abs();

    let s2 = s_squared(&space, &vec);
    let mult = (1.0 + 4.0 * s2).sqrt();

    println!(
        "  Germanium (Z=32, {} det): E = {:.12} Ha, resid = {:.3e}, iters = {}, <S^2> = {:.6}, 2S+1 = {:.4} (diff {:.3e})",
        space.n_det, e, resid, iters, s2, mult, rayleigh_diff
    );

    assert!(resid < 1e-10, "Germanium solve did not converge: residual {resid:.3e}");
    assert!(rayleigh_diff < 1e-10, "Germanium Rayleigh quotient differs from eigenvalue by {rayleigh_diff:.3e}");
    assert!(rayleigh_diff / e.abs() < 1e-14, "Germanium relative Rayleigh diff is {:.3e}", rayleigh_diff / e.abs());
    assert!((mult - 3.0).abs() < 1e-4, "Germanium ground state must be 3P (multiplicity 3), got {mult:.4}");
}


// ------------------------------------------------------------------ why a solve stopped

/// A small many-determinant problem: nitrogen's atom, 5 basis functions and enough
/// determinants that Davidson genuinely iterates rather than finishing immediately.
fn h2_like_problem() -> (FciSpace, holon_chem::fci::CiInts, Vec<f64>) {
    let (space, mo, _) = geometry_problem(
        &[holon_chem::elements::NITROGEN],
        vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]],
    );
    let ci0 = ci_ints(&mo, Order::Value);
    let diag = space.diagonal(&ci0);
    (space, ci0, diag)
}

/// A one-determinant problem: argon's atom, a closed shell.
fn one_determinant_problem() -> (FciSpace, holon_chem::fci::CiInts, Vec<f64>) {
    let (space, mo, _) = geometry_problem(
        &[holon_chem::elements::ARGON],
        vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]],
    );
    let ci0 = ci_ints(&mo, Order::Value);
    let diag = space.diagonal(&ci0);
    (space, ci0, diag)
}

/// The three exits are DISTINGUISHABLE, and the residual alone cannot tell them apart.
///
/// # The defect this addition answers
///
/// `davidson_eigh` had three exits and reported one number. A solve that reached tolerance
/// and a solve that ran out of iterations returned through the same branch, so every record
/// downstream had to infer "did this finish?" from "where did it stop?" — which is not the
/// same question. A campaign lost thirteen heavy atoms to that: at a 1e-10 publication bar,
/// solves that had given up sorted in alongside solves that had converged.
///
/// # What writing this test found
///
/// One system, three exits, differing only in the tolerance and the cap. The middle case is
/// not a contrivance — **nitrogen's atom does not reach 1e-11 on this engine.** It stops at
/// 8.1e-11 after eleven iterations because the Krylov subspace stops growing: no new
/// direction survives orthogonalisation against the basis. That is `Stagnated`, and it is a
/// property of the space and the start vector rather than of the budget, so more iterations
/// cannot fix it.
///
/// It also sits UNDER a 1e-10 publication bar while being OVER its own 1e-11 tolerance,
/// which is exactly the confusion the enum exists to remove: a bar on the residual sorts it
/// with the converged solves, and only the exit reason says it gave up.
#[test]
fn the_three_exits_are_distinguishable() {
    use holon_chem::fci::{davidson_eigh_with_exit, SolveExit};

    let (space, ci0, diag) = h2_like_problem();
    assert!(
        space.n_det > 1,
        "this test needs a space Davidson actually iterates on"
    );

    // CONVERGED: a tolerance this system can reach.
    let (e_c, _, it_c, rd_c, ex_c) = davidson_eigh_with_exit(&space, &ci0, &diag, 1e-9, 400);
    println!("tol 1e-9,  cap 400: E = {e_c:.12}  resid = {rd_c:.3e}  iters = {it_c}  exit = {}", ex_c.label());
    assert_eq!(ex_c, SolveExit::Converged, "a reachable tolerance did not report Converged");
    assert!(ex_c.is_converged());

    // STAGNATED: a tolerance it cannot reach, stopped by the subspace rather than the cap.
    let (_, _, it_s, rd_s, ex_s) = davidson_eigh_with_exit(&space, &ci0, &diag, 1e-11, 400);
    println!("tol 1e-11, cap 400: resid = {rd_s:.3e}  iters = {it_s}  exit = {}", ex_s.label());
    assert_eq!(
        ex_s,
        SolveExit::Stagnated,
        "nitrogen's atom used to stop at 8.1e-11 by subspace stagnation with 400 iterations \
         available; it now reports {ex_s:?}. If the solver improved, this test should be \
         re-pointed at a system that still stagnates rather than deleted — the exit is the \
         thing under test, not nitrogen."
    );
    assert!(it_s < 400, "it reported Stagnated but used its whole budget, which is a cap");
    assert!(!ex_s.is_converged());

    // ITERATION CAP: the same reachable tolerance, starved of iterations.
    let (_, _, it_i, rd_i, ex_i) = davidson_eigh_with_exit(&space, &ci0, &diag, 1e-9, 1);
    println!("tol 1e-9,  cap 1:   resid = {rd_i:.3e}  iters = {it_i}  exit = {}", ex_i.label());
    assert_eq!(ex_i, SolveExit::IterationCap, "a one-iteration solve reported {ex_i:?}");
    assert_eq!(it_i, 1);
    assert!(!ex_i.is_converged());

    // THE POINT, and it is sharper than "hard to separate": the converged solve and the
    // stagnated one stop at the SAME PLACE. Same system, same eleven iterations, same
    // 8.105e-11 residual — the only difference is what was asked of them. NO threshold on
    // the residual can distinguish these two, at any value, because there is no difference
    // in the residual to threshold.
    println!(
        "  converged at {rd_c:.3e}, STAGNATED at {rd_s:.3e} — the SAME residual. No bar on \
         the residual can separate them at any value; only the exit reason does."
    );
    assert_eq!(
        rd_c.to_bits(),
        rd_s.to_bits(),
        "the two solves no longer stop at the identical residual ({rd_c:.6e} vs \
         {rd_s:.6e}), so this test has weakened from 'no threshold can separate them' to \
         'a threshold would have to be well chosen'. Still worth having, but re-read the \
         claim above before trusting it."
    );
}

/// A one-determinant space reports `Trivial`, not `Converged`.
///
/// It is exact by construction with no iteration performed, and calling that "converged"
/// would claim an iterative result where none happened. `is_converged()` is still true,
/// because for a consumer asking "can I trust this number" the answer is yes.
#[test]
fn a_one_determinant_space_reports_trivial() {
    use holon_chem::elements::ARGON;
    use holon_chem::fci::{davidson_eigh_with_exit, SolveExit};
    use holon_chem::pair::automatic_route;

    // Ar2 is one determinant: a closed shell at both centres.
    assert_eq!(
        automatic_route(ARGON, ARGON).n_det(),
        1,
        "Ar2 is no longer a one-determinant space; pick another for this test"
    );

    let (space, ci0, diag) = one_determinant_problem();
    let (_, _, iters, resid, exit) = davidson_eigh_with_exit(&space, &ci0, &diag, 1e-11, 400);
    println!("one determinant: iters = {iters}  resid = {resid:.3e}  exit = {}", exit.label());
    assert_eq!(exit, SolveExit::Trivial);
    assert_eq!(iters, 0, "a trivial space performed iterations");
    assert!(exit.is_converged(), "a trivial space's answer is exact and should read as usable");
}

// ============================================================ the variational guard
//
// A residual is small for ANY eigenvector, so no residual threshold can detect a solve that
// converged cleanly onto the wrong one. The `saturation3-mesh` lane measured exactly that
// on this crate: a deliberately wrong warm start landing 7.47 hartree above the ground
// state, reporting a residual of 5.98e-11 against the correct solve's 5.24e-11 and an
// IDENTICAL exit reason. Both of the record's existing discriminators were blind to it.
//
// `Solution::variational_margin` is the cheap bound that is not: `E_0 <= min_i H_ii`,
// rigorously, because a single determinant is itself a normalised trial vector. These two
// tests are the halves that make it trustworthy — it FIRES on the defect, and it is SILENT
// on a correct solve. A guard demonstrated only in one direction is half a guard.

/// An (H, H, Cl) problem at 605 determinants, at a named geometry.
fn hhcl_problem(hh: f64, hcl: f64, deg: f64) -> (holon_chem::fci::FciSpace, holon_chem::fci::MoIntegrals) {
    use holon_chem::dual::D2;
    use holon_chem::elements::{CHLORINE, HYDROGEN};
    let c = |x: f64, y: f64| [D2::c(x), D2::c(y), D2::c(0.0)];
    let th = deg * core::f64::consts::PI / 180.0;
    let (space, mo, _) = holon_chem::pair::geometry_problem(
        &[HYDROGEN, HYDROGEN, CHLORINE],
        vec![c(0.0, 0.0), c(hh, 0.0), c(hcl * th.cos(), hcl * th.sin())],
    );
    (space, mo)
}

/// SATURATION-3's G0 compact staked geometry: H-H and H-Cl at 0.75 of their located
/// equilibria. Robust — Davidson recovers the ground state here from ANY start tried.
fn hhcl_g0_compact() -> (holon_chem::fci::FciSpace, holon_chem::fci::MoIntegrals) {
    hhcl_problem(1.0415, 1.9027, 60.0)
}

/// A geometry where Davidson DOES converge to the wrong eigenvector from a bad start —
/// one of sixteen found in a sixty-geometry scan (`examples/s3_wrongstate_hunt.rs`).
/// Named separately from the compact one because the plant needs a non-empty sector and
/// the guard's silence needs a correct solve, and those are different geometries.
///
/// # RE-STAKED, and the reason is the point
///
/// This read `(1.50, 1.90, 90.0)` and was staked under a solver budget of 4000. When the
/// budget was raised to 5000 the plant VOIDed: at that geometry the wrong start now has
/// enough iterations to escape the trap and it converged to within 1.023e-12 Ha of the true
/// ground state, so there was no wrong-eigenvector failure left to catch.
///
/// **A wrong-eigenvector trap is BUDGET-DEPENDENT.** The sector shrinks as the budget grows,
/// which is obvious in hindsight and was not staked for. Re-hunted at the new budget
/// (`examples/s3_wrongstate_hunt`): 16 of 60 geometries still trap the worst-diagonal start,
/// the guard catches all 16, and this is the nearest neighbour of the old stake that still
/// fires — 8.0179 Ha above the ground state.
///
/// So the VOID branch below is not a formality and must never be softened into a skip: it
/// is the only thing that stopped a budget change from turning this test green on nothing.
/// If the budget moves again, expect to re-stake again, and re-hunt rather than guess.
fn hhcl_wrong_state_prone() -> (holon_chem::fci::FciSpace, holon_chem::fci::MoIntegrals) {
    // Staked under DAVIDSON_DEFAULT_BUDGET = 5000; see above.
    hhcl_problem(1.50, 1.50, 90.0)
}

#[test]
fn the_variational_guard_is_silent_on_a_correct_solve() {
    let (space, mo) = hhcl_g0_compact();
    let sol = holon_chem::fci::solve_determinant(&space, &mo);
    let margin = sol
        .variational_margin
        .expect("the determinant route computes the diagonal and must report the margin");
    assert!(
        margin > 0.0,
        "the guard FIRED on a correct solve: energy {:.9} sits {:.3e} ABOVE min_i H_ii. A \
         guard that voids good nodes is worse than no guard.",
        sol.e.v,
        -margin
    );
    // Not merely positive: far enough above zero that it cannot drift into firing. The
    // residual these solves live at is ~1e-10, so a margin at 1e-2 is eight orders clear.
    assert!(
        margin > 1e-3,
        "the margin is only {margin:.3e} Ha. That is close enough to the residual scale \
         that ordinary numerical variation could cross it, and a guard whose threshold \
         floats in the noise is a source of false VOIDs rather than a check."
    );
    println!(
        "guard silent: E = {:.9}, margin below min_i H_ii = {margin:.6} Ha",
        sol.e.v
    );
}

#[test]
fn the_variational_guard_fires_on_a_solve_that_converged_to_the_wrong_state() {
    let (space, mo) = hhcl_wrong_state_prone();
    let good = holon_chem::fci::solve_determinant(&space, &mo);

    // THE PLANT, and it had to be RE-STAKED once.
    //
    // The first version used a random start at G0's COMPACT geometry, and VOIDed on an
    // empty sector — the test said so rather than passing. A deterministic worst-case start
    // (the highest-diagonal determinant) voided there too. Davidson is simply robust at
    // that geometry: it recovers the ground state from any start tried.
    //
    // So the sector was SEARCHED rather than assumed empty. `examples/s3_wrongstate_hunt.rs`
    // scans sixty (H,H,Cl) geometries and finds the failure at SIXTEEN of them — 27% — with
    // energies 7.3 to 8.1 hartree above the ground state, matching the 7.47 Ha
    // `saturation3-mesh` measured independently. The guard catches all sixteen. The
    // conclusion that matters for a 34,500-node table is that the wrong-eigenvector failure
    // is GEOMETRY-DEPENDENT and common, not rare and not universal.
    //
    // So the plant is now deterministic and aimed: start from the single determinant with
    // the LARGEST diagonal element. It is a legitimate normalised trial vector, it is as
    // far from the ground state as a single determinant gets, and it needs no seed. If
    // Davidson converges anywhere near it, the energy is above `min_i H_ii` by
    // construction and the guard must fire.
    let ci = holon_chem::fci::ci_ints(&mo, holon_chem::fci::Order::Value);
    let diag = space.diagonal(&ci);
    let (hi_i, hi_v) = diag
        .iter()
        .enumerate()
        .fold((0usize, f64::NEG_INFINITY), |acc, (i, &v)| {
            if v > acc.1 {
                (i, v)
            } else {
                acc
            }
        });
    let lo_v = diag.iter().copied().fold(f64::INFINITY, f64::min);
    let mut wrong = vec![0.0f64; space.n_det];
    wrong[hi_i] = 1.0;

    // M-PLANT-OBS: the carrier is asserted before the plant is scored. A start whose own
    // expectation value already sits below `min_i H_ii` could not produce an above-bound
    // answer and would prove nothing.
    assert!(
        hi_v > lo_v + 1.0,
        "the plant's carrier is empty: the highest diagonal ({hi_v:.4}) is only \
         {:.3e} above the lowest ({lo_v:.4}), so no start can put the solve meaningfully \
         above the bound",
        hi_v - lo_v
    );
    let overlap = good.vector[hi_i].abs();
    assert!(
        overlap < 0.2,
        "the plant's carrier is empty: the planted determinant carries {overlap:.3} of the \
         true ground state, which is a warm start rather than a wrong one"
    );

    let planted = holon_chem::fci::solve_determinant_from(&space, &mo, Some(&wrong));
    let margin = planted.variational_margin.expect("determinant route");
    println!(
        "plant: E = {:.6} against a true {:.6} ({:.3} Ha above); residual {:.3e} against \
         {:.3e}; exit {} against {}; margin {:.4}",
        planted.e.v,
        good.e.v,
        planted.e.v - good.e.v,
        planted.residual,
        good.residual,
        planted.exit.label(),
        good.exit.label(),
        margin
    );

    // The two claims that make this the RIGHT guard, in order.
    //
    // First: the existing discriminators are blind. If the plant were catchable by residual
    // or exit reason there would be no case for a new field, and this assertion is what
    // stops the guard being justified by a defect that something else already caught.
    if planted.e.v > good.e.v + 1e-3 {
        assert!(
            planted.residual < 1e-8 && planted.exit == good.exit,
            "the plant was caught by the EXISTING record (residual {:.3e}, exit {}), so the \
             variational margin is not what is needed here",
            planted.residual,
            planted.exit.label()
        );
        // Second: the guard sees it.
        assert!(
            margin < 0.0,
            "plant MISSED: the solve landed {:.3} Ha above the ground state and the \
             variational margin is still {margin:.4}, so the guard does not catch the one \
             failure it exists for",
            planted.e.v - good.e.v
        );
    } else {
        // The plant did not fire. That is a VOID, not a pass: it means this start was not
        // wrong enough on this system, and the guard is untested rather than validated.
        panic!(
            "PLANT VOID (empty sector): the wrong start converged to within {:.3e} Ha of the \
             true ground state, so there is no wrong-eigenvector failure here to catch and \
             this test validates nothing. Re-stake the plant.",
            (planted.e.v - good.e.v).abs()
        );
    }
}
