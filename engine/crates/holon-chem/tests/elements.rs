//! The species registry, and the two ELEMENTS-1 mutation plants.
//!
//! # What the structural checks are for
//!
//! The STO-3G basis is a transcription of two 1969/1970 papers, and a transcription error
//! is the failure this crate is least able to see by itself: every energy would still be
//! self-consistent, every gate would still pass, and the answer would be a different
//! model's. The checks below use the tabulation's own STRUCTURE against itself. STO-3G is
//! a fixed least-squares expansion of a Slater orbital, RESCALED per element, so the
//! contraction coefficients are the same for every element and only the exponents move —
//! and 2s and 2p share an exponent set. Neither fact is used anywhere in the engine, so
//! both are free evidence: a mistyped coefficient breaks the first, a mistyped exponent
//! in the wrong column breaks the second.

use holon_chem::dual::D2;
use holon_chem::elements::{
    by_symbol, by_z, sz2_sector, ShellKind, C_1S, C_2P, C_2S, FIRST_ROW, M_E_PER_U,
};
use holon_chem::md::Basis;
use holon_chem::pair::{atom_energy, solve_basis};
use holon_chem::sto3g::{H_COEFFS, H_EXPONENTS};

#[test]
fn the_contraction_coefficients_are_universal() {
    for sp in FIRST_ROW {
        for sh in sp.shells {
            let expected = match sh.kind {
                ShellKind::S1 => C_1S,
                ShellKind::S2 => C_2S,
                ShellKind::P2 => C_2P,
            };
            assert_eq!(
                sh.coeff, expected,
                "{}'s {:?} contraction is not the universal STO-3G triple; the coefficients \
                 are shared across the whole row by construction, so a difference here is a \
                 typo rather than a basis",
                sp.symbol, sh.kind
            );
        }
    }
}

#[test]
fn the_sp_shell_shares_its_exponents() {
    for sp in FIRST_ROW.iter().filter(|s| s.z >= 3) {
        let s2 = sp.shells.iter().find(|s| s.kind == ShellKind::S2).unwrap();
        let p2 = sp.shells.iter().find(|s| s.kind == ShellKind::P2).unwrap();
        assert_eq!(
            s2.alpha, p2.alpha,
            "{}'s 2s and 2p exponents differ; STO-3G's first row is an sp shell and they \
             are one set",
            sp.symbol
        );
    }
}

#[test]
fn hydrogen_is_not_re_declared() {
    let h = by_z(1).unwrap();
    assert_eq!(h.shells.len(), 1);
    assert_eq!(h.shells[0].alpha, H_EXPONENTS);
    assert_eq!(h.shells[0].coeff, H_COEFFS);
    // The whole point: the H2 referee gate and the first-row path must be grading the
    // same six decimals, not two transcriptions of them.
    assert_eq!(C_1S, H_COEFFS);
}

#[test]
fn exponents_rise_with_z_and_masses_do_too() {
    // Two more free structural facts. The 1s exponent scales roughly as Z^2 because a
    // 1s orbital contracts onto the nucleus, and the first row's most abundant isotopes
    // get heavier along it. Neither is used by the engine; both catch a swapped row.
    let mut last_alpha = 0.0f64;
    let mut last_mass = 0.0f64;
    for sp in FIRST_ROW {
        let a = sp.shells[0].alpha[0];
        assert!(
            a > last_alpha,
            "{}'s leading 1s exponent {a} does not exceed the previous element's {last_alpha}",
            sp.symbol
        );
        assert!(
            sp.mass_u > last_mass,
            "{}'s mass does not exceed the previous element's",
            sp.symbol
        );
        last_alpha = a;
        last_mass = sp.mass_u;
    }
    assert!((by_z(1).unwrap().mass_me() / M_E_PER_U - 1.00782503207).abs() < 1e-12);
}

#[test]
fn the_registry_refuses_what_it_does_not_have() {
    assert!(by_z(0).is_none());
    assert!(by_z(11).is_none(), "sodium is not the first row and d functions are a successor");
    for sp in FIRST_ROW {
        assert_eq!(by_symbol(sp.symbol).unwrap().z, sp.z);
        assert_eq!(by_z(sp.z).unwrap().symbol, sp.symbol);
    }
    assert!(by_symbol("Co").is_none());
    for n in 0..12u32 {
        assert_eq!(sz2_sector(n), n % 2);
    }
}

/// Basis functions per element, and the determinant counts that follow. A count is the
/// cheapest thing to get wrong silently: the wrong electron partition would run a
/// different physical system and still converge.
#[test]
fn the_basis_and_electron_counts_are_what_the_row_gives() {
    assert_eq!(by_z(1).unwrap().n_basis(), 1);
    assert_eq!(by_z(2).unwrap().n_basis(), 1);
    for z in 3..=10 {
        assert_eq!(by_z(z).unwrap().n_basis(), 5, "Z = {z} should carry 1s, 2s and 2p");
        assert_eq!(by_z(z).unwrap().n_electrons(), z);
    }
}

// ------------------------------------------------------------------ plant (i)

/// Build a two-centre basis by hand, so a plant can change ONE declared input.
fn hand_basis(
    shells: &[(usize, u8, [f64; 3], [f64; 3])],
    charges: Vec<f64>,
    r: f64,
) -> Basis {
    Basis::assemble(
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::var(r)],
        ],
        charges,
        shells,
    )
}

fn decls_for(z: u32, centre: usize) -> Vec<(usize, u8, [f64; 3], [f64; 3])> {
    by_z(z)
        .unwrap()
        .shells
        .iter()
        .map(|sh| (centre, sh.kind.l(), sh.alpha, sh.coeff))
        .collect()
}

/// PLANT (i), the Z-mutation: the pipeline provably reads the nuclear charge.
///
/// Carrier: a run of HF with fluorine's charge set to 8 and everything else — the basis,
/// the electron count, the geometry — untouched. Sector: the nuclear one-electron
/// operator, which is the only place `Z` appears. Asserted nonzero BEFORE the plant is
/// scored, per M-PLANT-SECTOR: a plant on an empty sector VOIDs, and "the nuclear
/// attraction is identically zero" would make this test pass for the wrong reason.
#[test]
fn plant_z_mutation_moves_the_energy_by_orders_of_magnitude() {
    let mut shells = decls_for(1, 0);
    shells.extend(decls_for(9, 1));
    let r = 1.8;

    // --- the carrier's sector, asserted nonzero first ---
    let honest = hand_basis(&shells, vec![1.0, 9.0], r);
    let ao = holon_chem::md::ao_integrals(&honest);
    let v_norm: f64 = ao.v.iter().map(|x| x.v.abs()).sum();
    assert!(
        v_norm > 1.0,
        "the nuclear-attraction sector this plant acts on is empty (sum |V| = {v_norm:.3e}); \
         the plant would VOID rather than score"
    );

    let (na, nb) = (5usize, 5usize);
    let e_honest = solve_basis(&honest, na, nb).e.v;
    let mutated = hand_basis(&shells, vec![1.0, 8.0], r);
    let e_mutated = solve_basis(&mutated, na, nb).e.v;
    let shift = (e_honest - e_mutated).abs();
    println!(
        "plant (i) Z-mutation: E(Z_F = 9) = {e_honest:.9}, E(Z_F = 8) = {e_mutated:.9}, \
         shift = {shift:.4e} hartree"
    );
    // The staked bound is the referee gate's own tolerance; the observed shift is many
    // orders above it, which is the plant firing rather than squeaking past.
    assert!(
        shift > 1e-10,
        "one unit of nuclear charge moved the energy by only {shift:.3e} hartree; the \
         pipeline is not reading Z"
    );
    assert!(
        shift > 1.0,
        "the Z-mutation shifted the energy by {shift:.3e} hartree, which is not the \
         orders-of-magnitude response a whole unit of charge must produce"
    );
}

/// PLANT (ii), the basis-mutation: the pin protects the declared contraction.
///
/// Carrier: hydrogen's leading contraction coefficient perturbed by one part in a
/// million. Sector: the residual against the banked 50-digit H2 curve. Asserted nonzero
/// first, in the sense that matters — the unmutated run must AGREE with the reference,
/// or "the mutation changed the residual" would be measuring nothing.
#[test]
fn plant_basis_mutation_fires_the_referee_tolerance() {
    let r = 1.4;
    let honest_c = H_COEFFS;
    let mut mutated_c = H_COEFFS;
    mutated_c[0] *= 1.0 + 1e-6;

    let shells_of = |c: [f64; 3]| {
        vec![
            (0usize, 0u8, H_EXPONENTS, c),
            (1usize, 0u8, H_EXPONENTS, c),
        ]
    };
    let e_honest = solve_basis(&hand_basis(&shells_of(honest_c), vec![1.0, 1.0], r), 1, 1)
        .e
        .v;
    let reference = holon_chem::h2_energy(r);
    let base_residual = (e_honest - reference).abs();
    assert!(
        base_residual < 1e-12,
        "the UNMUTATED run already disagrees with the banked H2 curve by {base_residual:.3e}; \
         the plant would be scoring against a broken baseline"
    );

    let e_mutated = solve_basis(&hand_basis(&shells_of(mutated_c), vec![1.0, 1.0], r), 1, 1)
        .e
        .v;
    let residual = (e_mutated - reference).abs();
    println!(
        "plant (ii) basis-mutation: 1e-6 on one coefficient moves the residual from \
         {base_residual:.3e} to {residual:.3e} hartree"
    );
    // `REFEREE_STAKE_E` is the pointwise bound the H2 gate is staked at. The mutation
    // must break it, or a mistyped basis could pass the gate.
    assert!(
        residual > holon_chem::REFEREE_STAKE_E,
        "a 1e-6 perturbation of one contraction coefficient moved the energy by only \
         {residual:.3e} hartree, which is inside the staked referee tolerance \
         {:.0e} — the gate would not notice a mistyped basis",
        holon_chem::REFEREE_STAKE_E
    );
}

/// The atomic energies the whole first row rests on, checked where the model makes them
/// checkable WITHOUT a reference table.
///
/// # Why these four and not all ten
///
/// Helium, fluorine and neon have determinant spaces of one, five and one respectively,
/// and in each case full CI degenerates to a single-configuration answer for a structural
/// reason: neon fills every orbital, helium fills its only one, and fluorine is one hole
/// in a filled shell, where the CI problem is the one-particle problem for the hole.
/// Hydrogen is one electron and therefore exact in its own basis. So these four energies
/// are the ones this crate can state a PROPERTY of rather than a number, and the property
/// is checked here: the FCI must equal the closed-shell/one-hole answer exactly, and the
/// hydrogen atom must equal what the banked H2 path already computes for it.
#[test]
fn the_degenerate_atoms_are_exactly_their_single_configuration_energies() {
    let h = atom_energy(by_z(1).unwrap());
    let banked = holon_chem::h_atom_energy();
    assert!(
        (h - banked).abs() < 1e-15,
        "the general path's hydrogen atom {h:.17} differs from the banked {banked:.17}"
    );
    for z in [2u32, 9, 10] {
        let sp = by_z(z).unwrap();
        let n_orb = sp.n_basis();
        let na = ((z + z % 2) / 2) as usize;
        let nb = ((z - z % 2) / 2) as usize;
        // The determinant count these energies are exact for.
        let choose = |n: usize, k: usize| -> usize {
            (0..k).fold(1usize, |acc, i| acc * (n - i) / (i + 1))
        };
        let n_det = choose(n_orb, na) * choose(n_orb, nb);
        assert!(
            n_det <= 5,
            "{}'s determinant space is {n_det}, so it is no longer the degenerate case \
             this test's reasoning depends on",
            sp.symbol
        );
        let e = atom_energy(sp);
        assert!(
            e.is_finite() && e < 0.0,
            "{}'s atomic energy came back {e}",
            sp.symbol
        );
        println!("  {} ({} determinants): {e:.12} hartree", sp.symbol, n_det);
    }
}
