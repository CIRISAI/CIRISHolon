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
use holon_chem::elements::{by_symbol, by_z, ALL_ELEMENTS, ShellKind};
use holon_chem::md::{ao_integrals, spherical_components, SPHERICAL_D};
use holon_chem::pair::{build_basis, electron_counts, solve_basis};
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

/// The registry's own basis-size arithmetic agrees with what the engine actually assembles.
///
/// # Why this gate exists
///
/// `Species::n_basis()` sums `ShellKind::n_functions()`, and `pair::feasibility` uses it to
/// decide -- before computing anything -- whether a species is reachable and by which
/// route. Nothing tied it to `build_basis`. When the projection landed, `n_functions` went
/// on reporting six per d shell for a while: the registry said xenon was 29 functions, the
/// engine built 27, and the two never met. A check that establishes one direction is not a
/// check on the other, so this asserts the equality for EVERY element rather than for the
/// eighteen that happen to have no d shell.
#[test]
fn the_registrys_basis_size_is_the_one_the_engine_assembles() {
    for sp in ALL_ELEMENTS {
        let built = build_basis(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]).n;
        assert_eq!(
            sp.n_basis(),
            built,
            "{} declares {} basis functions and the engine assembles {built}",
            sp.symbol,
            sp.n_basis()
        );
    }
    // And the two spellings of "how many functions is a d shell" agree.
    for kind in [ShellKind::D3, ShellKind::D4] {
        assert_eq!(
            kind.n_functions(),
            spherical_components(kind.l()),
            "{kind:?} disagrees with md::spherical_components"
        );
    }
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


/// The projected basis is a SUBSPACE of the Cartesian one, and the variational principle
/// says so out loud.
///
/// # Why this is the strongest check available without an external table
///
/// Every other gate here is structural: the rows annihilate `r^2`, they are orthonormal in
/// the Cartesian metric, the counts come out right. All of those would still pass if the
/// projection were an orthonormal map onto the WRONG five-dimensional subspace, or onto the
/// right one with a sign error that left the metric intact.
///
/// This one cannot. The five spherical functions span a strict subspace of the six
/// Cartesian ones, so a full CI in the smaller space is variationally ABOVE a full CI in
/// the larger -- necessarily, with no tolerance and no appeal. If the projection corrupted
/// the integrals in any way that mattered, the "smaller" calculation would be free to come
/// out below the larger one, which nothing in physics permits.
///
/// The gap is also reported rather than merely bounded, because its SIZE is the content of
/// the whole change: it is what the spurious `l = 0` function was contributing, and it is
/// what a minimal basis is not supposed to have.
/// # Why krypton and not xenon
///
/// Krypton is the cheapest element that exercises the whole statement: it has a d shell, so
/// the projection is built, and its Cartesian counterpart is 19 functions and 361
/// determinants -- seconds. Xenon would say the same thing about the same code and costs
/// 164,836 determinants to say it, because dropping two spurious functions from 29 turns a
/// closed shell into one with four holes. A suite that takes minutes to re-establish a fact
/// it already has gets run less often, which is a worse outcome than the coverage is worth.
#[test]
fn the_projected_basis_sits_variationally_above_the_cartesian_one() {
    for (z, sym) in [(36u32, "Kr")] {
        let sp = by_z(z).unwrap();
        let (_, na, nb) = electron_counts(&[sp]);
        let centre = vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]];

        let spherical = build_basis(&[sp], centre.clone());
        let n_d = sp.shells.iter().filter(|s| s.kind.l() == 2).count();
        assert!(n_d > 0, "carrier: {sym} must have d shells for this to test anything");
        assert_eq!(
            spherical.n_cart - spherical.n,
            n_d,
            "carrier: the two bases must actually differ in size"
        );
        let e_sph = solve_basis(&spherical, na, nb).e.v;

        // The same basis with the projection switched off: every Cartesian component kept.
        let mut cartesian = build_basis(&[sp], centre);
        cartesian.n = cartesian.n_cart;
        cartesian.sph = None;
        let e_cart = solve_basis(&cartesian, na, nb).e.v;

        assert!(
            e_cart.is_finite() && e_sph.is_finite(),
            "{sym}: one of the two solves did not return a number"
        );
        assert!(
            e_cart <= e_sph,
            "{sym}: the {}-function projected basis gave {e_sph:.12} and the \
             {}-function Cartesian basis gave {e_cart:.12}. The projected space is a strict \
             SUBSPACE, so its variational minimum cannot lie below -- an inversion here \
             means the projection is not an isometry onto a subspace of the original, which \
             no structural check would catch.",
            spherical.n,
            cartesian.n
        );
        println!(
            "{sym}: spherical ({} fn) {e_sph:.9} Ha, Cartesian ({} fn) {e_cart:.9} Ha, \
             the spurious l=0 functions are worth {:.3e} Ha",
            spherical.n,
            cartesian.n,
            e_sph - e_cart
        );
    }
}

/// PLANT: a wrong transform -- the sixth row left in -- must FIRE against the freeze's counts.
///
/// # What is being planted, and against what
///
/// ELEMENTS3_PREREG states five determinant counts as model properties: xenon's atom is ONE
/// determinant, Br2 is ~1.3e3, HBr ~3.6e2, HI ~784, and Xe2 is up to 54 spatial orbitals.
/// Those numbers are the freeze's, written before this lane existed, and they are what
/// AMENDMENT A1.1 identifies the component convention from.
///
/// The plant is the transform that keeps all six Cartesian components -- which is what a
/// projection with the spurious `l = 0` row left in reduces to. The requirement is
/// two-sided and that is the whole point: the correct transform must REPRODUCE every one of
/// the freeze's counts, and the planted one must MISS every one of them. A gate that only
/// checked the first would pass on any convention that happened to be self-consistent.
///
/// # Why the counts are computed and not solved
///
/// Under the plant, Xe2 is 58 orbitals with 108 electrons, which is about 1.8e11
/// determinants -- building that string list would not return. The count is a binomial and
/// is computed as one, which also makes this gate instant.
#[test]
fn plant_a_wrong_transform_fires_against_the_freezes_own_counts() {
    /// The freeze's stated counts. `None` means the freeze stated an orbital count only.
    /// (species, expected n_orb, expected n_det or None)
    const FROZEN: [(&str, &str, usize, Option<u128>); 5] = [
        ("Xe", "", 27, Some(1)),          // "Xe's atom is ONE determinant"
        ("Br", "Br", 36, Some(1296)),     // "Br2 ~1.3e3"
        ("H", "Br", 19, Some(361)),       // "HBr ~3.6e2"
        ("H", "I", 28, Some(784)),        // "HI ~784"
        ("Xe", "Xe", 54, None),           // "up to 54 spatial orbitals for Xe2"
    ];

    let mut fired = 0usize;
    for (a, b, want_orb, want_det) in FROZEN {
        let sa = by_symbol(a).unwrap();
        let species: Vec<_> = if b.is_empty() {
            vec![sa]
        } else {
            vec![sa, by_symbol(b).unwrap()]
        };
        let name = format!("{a}{b}");

        // The declared basis: five components per d shell.
        let n_sph: usize = species.iter().map(|s| s.n_basis()).sum();
        // The plant: the sixth row retained, so one extra function per d shell.
        let n_d: usize = species
            .iter()
            .map(|s| s.shells.iter().filter(|x| x.kind.l() == 2).count())
            .sum();
        let n_cart = n_sph + n_d;

        // Carrier, per M-PLANT-SECTOR: the plant must actually act. A species with no d
        // shell would leave the two identical and the plant would be scored on nothing.
        assert!(
            n_d > 0 && n_cart > n_sph,
            "{name}: the plant has no sector to act on -- no d shells, so retaining the \
             sixth row changes nothing"
        );

        let (n_elec, na, nb) = electron_counts(&species);
        let _ = n_elec;
        assert_eq!(
            n_sph, want_orb,
            "{name} is {n_sph} orbitals under the declared convention and the freeze says \
             {want_orb}"
        );
        assert_ne!(
            n_cart, want_orb,
            "{name}: PLANT MISSED on the orbital count -- the wrong transform gives \
             {n_cart}, the same number the freeze states, so this count does not \
             discriminate the convention"
        );

        if let Some(want) = want_det {
            let got_sph = binom(n_sph as u128, na as u128) * binom(n_sph as u128, nb as u128);
            let got_cart = binom(n_cart as u128, na as u128) * binom(n_cart as u128, nb as u128);
            assert_eq!(
                got_sph, want,
                "{name} is {got_sph} determinants under the declared convention and the \
                 freeze says {want}"
            );
            assert_ne!(
                got_cart, want,
                "{name}: PLANT MISSED on the determinant count"
            );
            println!(
                "plant: {name} declared {n_sph} orb / {got_sph} dets (freeze: {want_orb} / \
                 {want}); wrong transform {n_cart} orb / {got_cart} dets -- FIRES"
            );
        } else {
            println!(
                "plant: {name} declared {n_sph} orb (freeze: {want_orb}); wrong transform \
                 {n_cart} orb -- FIRES"
            );
        }
        fired += 1;
    }
    assert_eq!(
        fired, FROZEN.len(),
        "every one of the freeze's counts has to discriminate the convention, or the \
         amendment's identification rests on fewer numbers than it claims"
    );
}

/// Binomial coefficient in `u128`, exact for everything this file asks of it.
fn binom(n: u128, k: u128) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut acc: u128 = 1;
    for i in 0..k {
        acc = acc * (n - i) / (i + 1);
    }
    acc
}
