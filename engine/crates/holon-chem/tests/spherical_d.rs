//! The d shells are five functions, not six, and the sixth was not an `l = 2` function.
//!
//! # What this gate is for
//!
//! ELEMENTS-3 is the first campaign whose elements carry d shells, and the integral
//! recursions are Cartesian: a d shell computes as six components, `xx yy zz xy xz yz`.
//! Those six do not span an `l = 2` space. They span the five real solid harmonics plus
//! `(x^2+y^2+z^2) exp(-a r^2)`, which is spherically symmetric and therefore `l = 0`.
//!
//! In a MINIMAL basis that sixth function is not a refinement, it is a different model.
//! The whole premise of STO-3G is one contracted function per occupied atomic orbital, and
//! a spurious `l = 0` function per d shell breaks it in a way that shows up immediately:
//! krypton stops being a closed shell. With five components krypton is eighteen functions
//! holding thirty-six electrons -- one determinant, exactly, which is what ELEMENTS-3's E1
//! asserts. With six it is nineteen functions with two holes, 361 determinants, and no
//! closed shell anywhere.
//!
//! So the engine projects to the five spherical components, and this file checks the
//! projection is the right one rather than merely a smaller one.

use holon_chem::dual::D2;
use holon_chem::elements::{by_z, ALL_ELEMENTS, ShellKind};
use holon_chem::md::{ao_integrals, spherical_components, SPHERICAL_D};
use holon_chem::pair::{build_basis, electron_counts};
use holon_chem::fci::{jacobi_eigh, FciSpace};

/// The projection annihilates exactly the spherically symmetric direction, and nothing else.
///
/// This is the whole claim in one line of algebra: `x^2 + y^2 + z^2` is the `l = 0`
/// contaminant, so the five rows must kill it, and they must not kill anything independent
/// of it or the projection would be discarding real `l = 2` content.
#[test]
fn the_projection_removes_the_spherically_symmetric_direction_and_only_that() {
    // r^2 in the crate's normalised Cartesian component order [xx, yy, zz, xy, xz, yz].
    let r2 = [1.0, 1.0, 1.0, 0.0, 0.0, 0.0];
    for (i, row) in SPHERICAL_D.iter().enumerate() {
        let dot: f64 = row.iter().zip(r2.iter()).map(|(a, b)| a * b).sum();
        assert!(
            dot.abs() < 1e-15,
            "spherical row {i} has a component {dot:.3e} along x^2+y^2+z^2; the projection \
             is supposed to remove exactly that direction"
        );
    }

    // Rank five: the five rows are independent, so nothing genuine was thrown away with
    // the contaminant. Checked through the Gram determinant in the Cartesian metric.
    let gram = cartesian_gram();
    let mut m = vec![0.0f64; 25];
    for a in 0..5 {
        for b in 0..5 {
            let mut acc = 0.0;
            for i in 0..6 {
                for j in 0..6 {
                    acc += SPHERICAL_D[a][i] * gram[i * 6 + j] * SPHERICAL_D[b][j];
                }
            }
            m[a * 5 + b] = acc;
        }
    }
    // In the Cartesian overlap metric the five spherical functions are ORTHONORMAL, which
    // is the statement that makes them a basis rather than five arbitrary combinations.
    for a in 0..5 {
        for b in 0..5 {
            let want = if a == b { 1.0 } else { 0.0 };
            assert!(
                (m[a * 5 + b] - want).abs() < 1e-14,
                "spherical overlap ({a},{b}) is {} and should be {want}; the projection's \
                 coefficients are not normalised against the Cartesian metric",
                m[a * 5 + b]
            );
        }
    }
}

/// The overlap matrix of the crate's six NORMALISED Cartesian d components.
///
/// Not read from the engine: derived from the primitive integrals, so the test's metric is
/// an independent statement rather than the same code checking itself. With the three
/// squares normalised and the three products carrying `sqrt(3)`, the squares overlap each
/// other at exactly `1/3` and everything else is orthogonal.
fn cartesian_gram() -> Vec<f64> {
    let mut g = vec![0.0f64; 36];
    for i in 0..6 {
        g[i * 6 + i] = 1.0;
    }
    for (i, j) in [(0usize, 1usize), (0, 2), (1, 2)] {
        g[i * 6 + j] = 1.0 / 3.0;
        g[j * 6 + i] = 1.0 / 3.0;
    }
    g
}

/// Every element's basis is one function per occupied orbital, which is what "minimal" means.
#[test]
fn the_basis_is_one_function_per_shell_component() {
    for sp in ALL_ELEMENTS {
        let want: usize = sp
            .shells
            .iter()
            .map(|sh| spherical_components(sh.kind.l()))
            .sum();
        let basis = build_basis(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        assert_eq!(
            basis.n, want,
            "{} assembles {} functions but its shells call for {want}",
            sp.symbol, basis.n
        );
        // d shells are where the two conventions differ, so name the difference.
        let n_d = sp.shells.iter().filter(|s| s.kind.l() == 2).count();
        assert_eq!(
            basis.n_cart - basis.n,
            n_d,
            "{} has {n_d} d shells, so exactly {n_d} Cartesian functions should have been \
             projected away",
            sp.symbol
        );
    }
}

/// E1's premise: the two heavy nobles close, exactly.
///
/// This is the observable that separates the two conventions, so it is asserted as a
/// determinant COUNT rather than as an energy. One determinant means every orbital the
/// basis provides is doubly occupied -- a closed shell with no room for a single
/// excitation, which is what a noble gas in a minimal basis is.
#[test]
fn the_heavy_nobles_are_single_determinant_closed_shells() {
    for (z, n_basis) in [(36u32, 18usize), (54, 27)] {
        let sp = by_z(z).unwrap();
        let basis = build_basis(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        let (n_elec, na, nb) = electron_counts(&[sp]);
        let space = FciSpace::new(basis.n, na, nb);
        assert_eq!(basis.n, n_basis, "{}'s minimal basis size", sp.symbol);
        assert_eq!(na, nb, "{} must be spin-paired", sp.symbol);
        assert_eq!(
            n_elec,
            2 * basis.n,
            "{} has {n_elec} electrons and {} orbitals; a closed shell needs exactly two \
             per orbital",
            sp.symbol,
            basis.n
        );
        assert_eq!(
            space.n_det, 1,
            "{} is not a single determinant in this basis ({} determinants). Under six \
             Cartesian d components it is not, and that is the observable which says the \
             sixth component does not belong in a minimal basis.",
            sp.symbol, space.n_det
        );
    }
}

/// The projected integrals are still a legitimate basis: positive definite overlap, unit
/// diagonal, and Hermitian one-electron operators -- on an element with d shells.
#[test]
fn the_projected_integrals_are_a_well_formed_basis() {
    let xe = by_z(54).unwrap();
    let basis = build_basis(&[xe], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
    let g = ao_integrals(&basis);
    let n = g.n;
    assert!(basis.shells.iter().any(|s| s.l == 2), "carrier: xenon must have d shells");

    for i in 0..n {
        assert!(
            (g.s[i * n + i].v - 1.0).abs() < 1e-12,
            "diagonal overlap at {i} is {}, so a projected function is not normalised",
            g.s[i * n + i].v
        );
        for j in 0..n {
            assert_eq!(
                g.s[i * n + j].v,
                g.s[j * n + i].v,
                "overlap is not exactly symmetric at ({i},{j})"
            );
            assert_eq!(
                g.t[i * n + j].v,
                g.t[j * n + i].v,
                "kinetic energy is not exactly symmetric at ({i},{j})"
            );
        }
    }
    let s_vals: Vec<f64> = g.s.iter().map(|d| d.v).collect();
    let (eigs, _) = jacobi_eigh(&s_vals, n);
    assert!(
        eigs[0] > 0.0,
        "projected overlap has a non-positive eigenvalue {}; the five rows are not \
         independent in this basis",
        eigs[0]
    );
    println!(
        "Xe projected basis: {n} functions, overlap eigenvalues {:.3e}..{:.3e}",
        eigs[0],
        eigs[eigs.len() - 1]
    );
}

/// Nothing below the first d shell moved.
///
/// The projection is built only when a basis has a d shell, so every element through argon
/// takes byte-for-byte the path it took before. Asserted rather than assumed, because it is
/// what keeps every ELEMENTS-1 and MIXTURES-1 result valid across this change.
#[test]
fn elements_without_d_shells_are_untouched() {
    for sp in ALL_ELEMENTS.iter().filter(|s| s.z <= 20) {
        let basis = build_basis(&[*sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        assert!(
            basis.sph.is_none(),
            "{} has no d shell, so no projection should have been built at all",
            sp.symbol
        );
        assert_eq!(basis.n, basis.n_cart);
    }
    // The first element that DOES carry one, so the check above is not vacuous.
    let sc = by_z(21).unwrap();
    assert!(sc.shells.iter().any(|s| s.kind == ShellKind::D3));
    let basis = build_basis(&[sc], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
    assert!(basis.sph.is_some(), "scandium carries a 3d shell and must be projected");
    assert_eq!(basis.n_cart - basis.n, 1);
}
