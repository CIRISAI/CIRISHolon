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
    by_symbol, by_z, sz2_sector, Shell, ShellKind, Species, ALL_ELEMENTS, C_1S, C_2P, C_2S,
    C_3D, C_3P, C_3P_HEAVY, C_3S, C_3S_HEAVY, C_4D, C_4P, C_4P_HEAVY, C_4S, C_4S_HEAVY, C_5P,
    C_5S, MAX_Z, M_E_PER_U, SECOND_ROW,
};
use holon_chem::md::Basis;
use holon_chem::pair::{atom_energy, solve_basis};
use holon_chem::sto3g::{H_COEFFS, H_EXPONENTS};

/// The coefficient triple every element with this shell must carry.
///
/// # Why this takes `Z` and not only the shell kind
///
/// Because "one triple per shell type" is FALSE, and a gate written to it would fire
/// honestly on correct data. STO-3G fitted 3s, 3p, 4s and 4p twice each -- once for the
/// row where the shell is valence, again for the rows below where it has become core --
/// and the tabulation carries both. The universality that actually holds is per FIT, so
/// the split points are named here as the facts they are.
fn expected_coeff(z: u32, kind: ShellKind) -> [f64; 3] {
    match kind {
        ShellKind::S1 => C_1S,
        ShellKind::S2 => C_2S,
        ShellKind::P2 => C_2P,
        // 3s and 3p: the Na..Ca fit, then the Sc..Xe fit.
        ShellKind::S3 if z <= 20 => C_3S,
        ShellKind::S3 => C_3S_HEAVY,
        ShellKind::P3 if z <= 20 => C_3P,
        ShellKind::P3 => C_3P_HEAVY,
        ShellKind::D3 => C_3D,
        // 4s and 4p: the K..Sr fit, then the Y..Xe fit.
        ShellKind::S4 if z <= 38 => C_4S,
        ShellKind::S4 => C_4S_HEAVY,
        ShellKind::P4 if z <= 38 => C_4P,
        ShellKind::P4 => C_4P_HEAVY,
        ShellKind::D4 => C_4D,
        ShellKind::S5 => C_5S,
        ShellKind::P5 => C_5P,
    }
}

#[test]
fn the_contraction_coefficients_are_universal() {
    let mut checked = 0usize;
    for sp in ALL_ELEMENTS {
        for sh in sp.shells {
            assert_eq!(
                sh.coeff,
                expected_coeff(sp.z, sh.kind),
                "{}'s {:?} contraction is not the STO-3G triple for its fit family; the \
                 coefficients are shared across every element carrying that fit by \
                 construction, so a difference here is a typo rather than a basis",
                sp.symbol,
                sh.kind
            );
            checked += 1;
        }
    }
    // A registry that lost its elements would pass the loop above by doing nothing.
    assert!(
        checked >= 200,
        "only {checked} shells were checked; the registry is supposed to carry 54 elements"
    );
    println!("coefficient universality: {checked} shells over {} elements", ALL_ELEMENTS.len());
}

/// Every s shell and its p partner of the same principal number share ONE exponent set.
///
/// Generalized from the first two rows to all four: this is STO-3G's "sp shell", and it
/// holds wherever both shells are declared. It is free evidence because nothing in the
/// engine uses it -- a mistyped exponent in the wrong column breaks it and nothing else.
#[test]
fn the_sp_shell_shares_its_exponents() {
    let pairs = [
        (ShellKind::S2, ShellKind::P2),
        (ShellKind::S3, ShellKind::P3),
        (ShellKind::S4, ShellKind::P4),
        (ShellKind::S5, ShellKind::P5),
    ];
    let mut checked = 0usize;
    for sp in ALL_ELEMENTS {
        for (sk, pk) in pairs {
            let (Some(s), Some(p)) = (
                sp.shells.iter().find(|x| x.kind == sk),
                sp.shells.iter().find(|x| x.kind == pk),
            ) else {
                continue;
            };
            assert_eq!(
                s.alpha, p.alpha,
                "{}'s {sk:?} and {pk:?} exponents differ; STO-3G declares them as one sp \
                 shell and they are one set",
                sp.symbol
            );
            checked += 1;
        }
    }
    assert!(checked >= 100, "only {checked} sp pairs found; the registry looks truncated");
    println!("sp exponent sharing: {checked} pairs");
}

/// From scandium down, some rows put the d function on the SAME exponent set as an sp
/// shell rather than giving it its own. Which rows do that is a fact of the tabulation, so
/// the gate reports the partition rather than asserting one.
#[test]
fn the_d_shells_report_whether_they_share_an_sp_set() {
    let mut shared = 0usize;
    let mut own = 0usize;
    for sp in ALL_ELEMENTS {
        for (dk, sk) in [(ShellKind::D3, ShellKind::S3), (ShellKind::D4, ShellKind::S4)] {
            let (Some(d), Some(s)) = (
                sp.shells.iter().find(|x| x.kind == dk),
                sp.shells.iter().find(|x| x.kind == sk),
            ) else {
                continue;
            };
            if d.alpha == s.alpha {
                shared += 1;
            } else {
                own += 1;
            }
        }
    }
    // Both cases must actually occur, or the registry has collapsed one into the other.
    assert!(
        shared > 0 && own > 0,
        "d shells: {shared} share an sp exponent set and {own} carry their own. Both \
         patterns are in the tabulation, so a run with either at zero means the \
         transcription flattened a distinction the source makes"
    );
    println!("d shells: {shared} share an sp set, {own} carry their own");
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

/// The five places where the declared mass FALLS as `Z` rises, by `Z` of the lighter one.
///
/// # Why this list exists rather than a monotonic assertion
///
/// Mass is not monotone in `Z` and the gate said it was, because through argon it happens
/// to be. The declared mass here is the MOST ABUNDANT ISOTOPE's, and an element whose
/// abundance peaks on a neutron-poor isotope can weigh less than its lighter neighbour:
///
/// * `40`Ar -> `39`K, `59`Co -> `58`Ni and `130`Te -> `127`I are the three inversions the
///   periodic table is known for, and they survive under this convention;
/// * `80`Se -> `79`Br and `98`Mo -> `97`Tc appear ON TOP of those three because
///   most-abundant-isotope mass is not standard atomic weight -- selenium's abundance
///   peaks at 80 (49.6%) while bromine's peaks at 79 (50.7%);
/// * technetium is the additional oddity that it has NO stable isotope, so its declared
///   `97`Tc is a representative choice rather than an abundance at all.
///
/// Naming them is what keeps the check useful: everywhere else mass still rises, so a
/// swapped row is still caught, and the exceptions are recorded as facts about isotopes
/// rather than quietly weakened into "mass mostly rises".
const MASS_INVERSIONS: [u32; 5] = [19, 28, 35, 43, 53];

#[test]
fn exponents_rise_with_z_and_masses_do_too() {
    // The 1s exponent scales roughly as Z^2 because a 1s orbital contracts onto the
    // nucleus. Neither this nor the mass ordering is used by the engine; both catch a
    // swapped row.
    let mut last_alpha = 0.0f64;
    let mut last_mass = 0.0f64;
    let mut seen_inversions = Vec::new();
    for sp in ALL_ELEMENTS {
        let a = sp.shells[0].alpha[0];
        assert!(
            a > last_alpha,
            "{}'s leading 1s exponent {a} does not exceed the previous element's {last_alpha}",
            sp.symbol
        );
        if MASS_INVERSIONS.contains(&sp.z) {
            assert!(
                sp.mass_u < last_mass,
                "{} is recorded as a mass inversion but its declared mass {} is not below \
                 the previous element's {last_mass}; the exception list has gone stale",
                sp.symbol,
                sp.mass_u
            );
            seen_inversions.push(sp.z);
        } else {
            assert!(
                sp.mass_u > last_mass,
                "{}'s mass {} does not exceed the previous element's {last_mass}, and it is \
                 not one of the five recorded isotope inversions",
                sp.symbol,
                sp.mass_u
            );
        }
        last_alpha = a;
        last_mass = sp.mass_u;
    }
    assert_eq!(
        seen_inversions,
        MASS_INVERSIONS.to_vec(),
        "every recorded inversion must actually occur, or the list is protecting nothing"
    );
    assert!((by_z(1).unwrap().mass_me() / M_E_PER_U - 1.00782503207).abs() < 1e-12);
}

#[test]
fn the_registry_refuses_what_it_does_not_have() {
    assert!(by_z(0).is_none());
    assert_eq!(MAX_Z, 54, "the registry stops at xenon; the next shell needs f functions");
    assert!(
        by_z(MAX_Z + 1).is_none(),
        "caesium needs a 6s shell and no f-capable integral path exists, so it must be \
         REFUSED rather than silently answered with something else"
    );
    for sp in ALL_ELEMENTS {
        assert_eq!(by_symbol(sp.symbol).unwrap().z, sp.z);
        assert_eq!(by_z(sp.z).unwrap().symbol, sp.symbol);
    }
    assert!(by_symbol("Cs").is_none());
    for n in 0..20u32 {
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
    for z in 11..=18 {
        assert_eq!(by_z(z).unwrap().n_basis(), 9, "Z = {z} should carry 1s, 2s, 2p, 3s and 3p");
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
    for z in [2u32, 9, 10, 18] {
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

/// Verify atomic ground states across the second row.
#[test]
fn second_row_atomic_ground_states() {
    for sp in SECOND_ROW {
        let e = atom_energy(sp);
        assert!(
            e.is_finite() && e < 0.0,
            "{}'s atomic energy came back {e}",
            sp.symbol
        );
        println!("  {} atomic energy: {e:.12} hartree", sp.symbol);
    }
}

/// Noble gas unbinding for Argon (Ar2 dimer in minimal basis STO-3G).
#[test]
fn argon_dimer_refuses_to_bind() {
    let ar = by_symbol("Ar").unwrap();
    let table = holon_chem::pair::generate_pair_table(ar, ar, 16);
    let deepest = table.e.iter().cloned().fold(f64::INFINITY, f64::min);
    let depth = table.meta.e_asymptote - deepest;
    println!(
        "  Ar2: {} knots over [{:.3}, {:.3}] bohr; deepest point {:+.4e} hartree relative to asymptote; well = {:?}",
        table.r.len(),
        table.meta.r_min,
        table.meta.r_max,
        depth,
        table.meta.well.map(|w| w.d_e)
    );
    assert!(
        table.meta.well.is_none(),
        "Ar2 reported a well of depth {:?} hartree. In STO-3G minimal basis, noble gas dimers are purely repulsive.",
        table.meta.well.map(|w| w.d_e)
    );
    assert!(
        depth <= holon_chem::pair::WELL_MIN_DEPTH,
        "Ar2 deepest point {depth:.4e} is below asymptote"
    );
    for (i, &f) in table.f.iter().enumerate() {
        assert!(
            f >= -1e-9,
            "Ar2 pulls inward at R = {} (F = {f:.3e}); curve must be monotonically repulsive",
            table.r[i]
        );
    }
}

/// Half a unit in the last place the DECLARATION can carry, for one exponent.
///
/// # Why this is derived per value and not one constant
///
/// Two limits apply and the honest bound is the coarser. The declaration carries eight
/// decimal places, so a small exponent is rounded at `0.5e-8`. The TABULATION it comes
/// from carries ten significant digits, so a large exponent -- xenon's leading 1s is
/// 6264.58 -- is only determined to `0.5e-6`, and the trailing decimals of its declaration
/// are padding rather than information. A single constant would either be too tight on the
/// heavy rows (firing on correct data) or too loose on the light ones (missing typos).
fn declaration_bound(v: f64) -> f64 {
    let sig10 = 0.5 * 10f64.powi(v.abs().log10().floor() as i32 - 9);
    sig10.max(0.5e-8)
}

/// The relative bound a RATIO of two declared exponents inherits from them.
fn ratio_bound(a: f64, b: f64) -> f64 {
    declaration_bound(a) / a + declaration_bound(b) / b
}

/// Worst exponent-ratio deviation over every fit family, in units of each element's own
/// derived rounding bound, with the element and family that produced it.
///
/// # Why the families are read off the coefficients
///
/// STO-3G rescales ONE least-squares fit per element, and a rescaling cannot change the
/// ratio of two exponents within a shell. So the ratio is a constant of the FIT, and the
/// coefficient triple is what identifies the fit -- which makes it the grouping key. That
/// has a second effect worth having: a shell wired to the wrong coefficient constant lands
/// in the wrong family and its ratio then misses that family's constant by orders of
/// magnitude, so this gate covers the wiring as well as the digits.
fn worst_ratio_deviation(species: &[Species]) -> (f64, String) {
    // Group (element, shell) by the declared coefficient triple, compared by bits so that
    // grouping never depends on a tolerance.
    let mut families: Vec<([u64; 3], Vec<(&'static str, [f64; 3])>)> = Vec::new();
    for sp in species {
        for sh in sp.shells {
            let key = [
                sh.coeff[0].to_bits(),
                sh.coeff[1].to_bits(),
                sh.coeff[2].to_bits(),
            ];
            match families.iter_mut().find(|(k, _)| *k == key) {
                Some((_, v)) => v.push((sp.symbol, sh.alpha)),
                None => families.push((key, vec![(sp.symbol, sh.alpha)])),
            }
        }
    }

    let mut worst = 0.0f64;
    let mut where_ = String::from("(nothing compared)");
    for (_, members) in families.iter() {
        if members.len() < 2 {
            continue;
        }
        for (label, i, j) in [("alpha_0/alpha_1", 0usize, 1usize), ("alpha_1/alpha_2", 1, 2)] {
            // Judge against the BEST-DETERMINED member -- the one whose own declaration
            // pins its ratio most tightly -- rather than against a mean, which a single
            // bad entry would drag toward itself.
            let rows: Vec<(&str, f64, f64)> = members
                .iter()
                .map(|(sym, a)| (*sym, a[i] / a[j], ratio_bound(a[i], a[j])))
                .collect();
            let reference = rows
                .iter()
                .min_by(|x, y| x.2.partial_cmp(&y.2).unwrap())
                .copied()
                .unwrap();
            for &(symbol, ratio, bound) in rows.iter() {
                if symbol == reference.0 {
                    continue;
                }
                let n = ((ratio - reference.1).abs() / reference.1) / (bound + reference.2);
                if n > worst {
                    worst = n;
                    where_ = format!(
                        "{symbol}'s {label} against {} (ratio {ratio:.12} vs {:.12})",
                        reference.0, reference.1
                    );
                }
            }
        }
    }
    (worst, where_)
}

/// How many times its own derived rounding bound an element may miss its family by.
///
/// Calibrated, not guessed: over the whole table the worst legitimate deviation is 0.81x,
/// so 4x is roughly a factor of five of headroom, while the smallest defect this is aimed
/// at -- one digit changed in the last place of one exponent -- moves a ratio by about 2x
/// its bound and anything earlier in the number moves it by orders of magnitude.
const RATIO_MARGIN: f64 = 4.0;

/// The exponent ratios within a shell are element-independent, to the precision the
/// declaration allows, across EVERY fit family in the table. This is the check that caught
/// the oxygen defect, generalized from the first two rows to all four.
#[test]
fn the_exponent_ratios_are_element_independent() {
    let (worst, where_) = worst_ratio_deviation(&ALL_ELEMENTS);
    println!("exponent ratios: worst {worst:.3}x its own rounding bound, at {where_}");
    assert!(
        worst < RATIO_MARGIN,
        "an exponent ratio misses its family's constant by {worst:.2}x the rounding its \
         own declaration allows, at {where_}. STO-3G rescales ONE fit per element, so this \
         ratio is a constant across every element sharing that fit: an outlier this far \
         out is a transcription error in one exponent, not rounding."
    );
}

/// PLANT (i): a single-digit typo in a staked new element's exponent fires the ratio band.
///
/// # What the plant sweeps, and the floor it establishes
///
/// A transcription error is one wrong digit somewhere in a number, so the plant does not
/// pick one perturbation: it walks EVERY decimal position of every one of xenon's 33
/// declared exponents and adds one unit there, which is what mistyping that digit does.
///
/// The gate cannot resolve all of them, and the honest thing is to say where the boundary
/// is rather than to choose a mutation that clears it. A ratio band judges deviations
/// against the rounding the declaration itself carries, so a change of ONE unit in the
/// last determined place is, by construction, the same size as the noise the bound is made
/// of -- no ratio-based gate can see it, and one that claimed to would be reporting its own
/// rounding. What this test therefore requires is that there is no gap ABOVE that floor:
/// every mutation at least ten times the declaration's bound must fire, with none slipping
/// through, and the count that falls below is reported rather than hidden.
///
/// # Carrier, per M-PLANT-SECTOR
///
/// The carrier is the band deviation, asserted LARGE before the plant is scored: the clean
/// table sits well under 1x its bound, so the gate demonstrably has room to move and a
/// firing is evidence about the plant rather than about a gate already at its threshold.
#[test]
fn plant_a_single_digit_typo_in_a_new_element_fires_the_ratio_band() {
    let (clean, _) = worst_ratio_deviation(&ALL_ELEMENTS);
    assert!(
        clean < 1.0,
        "carrier check: the unmutated table already sits at {clean:.2}x its bound, so a \
         plant firing here would not be evidence about the plant"
    );

    /// How far above the declaration's own rounding bound a perturbation must be before
    /// the gate is REQUIRED to see it: two decades.
    ///
    /// This number was measured, not chosen. A ratio band judges a perturbation of one
    /// exponent against a bound that both exponents contribute to, and the smaller of a
    /// pair carries much the larger relative bound -- so a typo in the LEADING exponent of
    /// a shell is diluted by its partner's coarser rounding. Sweeping every position of
    /// every xenon exponent, mutations two decades or more above the bound all fire, and
    /// the decade between one and two is a marginal band where firing depends on which
    /// partner the ratio happens to use. Both counts are reported below rather than
    /// tuned away.
    const RESOLVABLE: f64 = 100.0;
    /// The lower edge of the marginal band: below this the perturbation is the same size
    /// as the noise the bound is made of, and no ratio gate can see it.
    const FLOOR: f64 = 10.0;

    // Xenon: the heaviest element in the registry, the one a transcription error is most
    // likely to reach and least likely to be noticed in.
    let xe = by_z(54).unwrap();
    let mut resolvable_total = 0usize;
    let mut resolvable_missed = Vec::new();
    let mut marginal_total = 0usize;
    let mut marginal_fired = 0usize;
    let mut below_floor = 0usize;
    let mut quietest_resolvable = f64::INFINITY;

    for (si, sh) in xe.shells.iter().enumerate() {
        for ai in 0..3 {
            let base = sh.alpha[ai];
            let bound = declaration_bound(base);
            // Decimal positions from the leading digit down to the last declared place.
            let top = base.abs().log10().floor() as i32;
            for k in (-8..=top).rev() {
                let delta = 10f64.powi(k);
                if delta >= base {
                    continue; // changing the leading digit is not a typo, it is a different number
                }
                let mut alpha = sh.alpha;
                alpha[ai] = base + delta;

                let mut shells: Vec<Shell> = xe.shells.to_vec();
                shells[si] = Shell { alpha, ..*sh };
                let mutated = Species {
                    shells: Box::leak(shells.into_boxed_slice()),
                    ..xe
                };
                let mut table: Vec<Species> = ALL_ELEMENTS.to_vec();
                table[53] = mutated;
                let (worst, _) = worst_ratio_deviation(&table);

                if delta >= RESOLVABLE * bound {
                    resolvable_total += 1;
                    quietest_resolvable = quietest_resolvable.min(worst);
                    if worst < RATIO_MARGIN {
                        resolvable_missed.push(format!(
                            "{:?}[{ai}] {base} + 1e{k} reached only {worst:.2}x",
                            sh.kind
                        ));
                    }
                } else if delta >= FLOOR * bound {
                    marginal_total += 1;
                    if worst >= RATIO_MARGIN {
                        marginal_fired += 1;
                    }
                } else {
                    below_floor += 1;
                }
            }
        }
    }

    assert!(
        resolvable_missed.is_empty(),
        "PLANT MISSED: {} of {resolvable_total} resolvable single-digit mutations did not \
         fire the ratio band (it fires at {RATIO_MARGIN:.0}x). A transcription gate a \
         one-digit error can walk past is the oxygen defect with more elements. First few: \
         {:?}",
        resolvable_missed.len(),
        &resolvable_missed[..resolvable_missed.len().min(4)]
    );
    assert!(
        resolvable_total >= 100,
        "only {resolvable_total} mutations were above the gate's resolution; the plant has \
         to exercise the sector broadly or it is testing one lucky digit"
    );
    println!(
        "plant (i): {resolvable_total} single-digit mutations at or above {RESOLVABLE:.0}x \
         the declaration's own rounding -- ALL fire, quietest {quietest_resolvable:.0}x \
         against a {RATIO_MARGIN:.0}x threshold. Marginal band ({FLOOR:.0}x..{RESOLVABLE:.0}x): \
         {marginal_fired} of {marginal_total} fire. Below {FLOOR:.0}x: {below_floor} \
         mutations, invisible to any ratio gate by construction. Clean table {clean:.2}x."
    );
}

/// The oxygen defect itself, replayed on the element it happened to.
///
/// One digit of oxygen's leading 1s exponent was transcribed as `130.70932000` for
/// `130.70932140`. This is the shape the whole T1 apparatus exists for, so it is planted
/// explicitly rather than left to the sweep: a gate that catches every synthetic mutation
/// but not the one real defect in this crate's history would be measuring the wrong thing.
#[test]
fn plant_the_historical_oxygen_defect_fires_the_ratio_band() {
    let o = by_z(8).unwrap();
    let si = o
        .shells
        .iter()
        .position(|x| x.kind == ShellKind::S1)
        .unwrap();
    let mut shells: Vec<Shell> = o.shells.to_vec();
    let mut alpha = shells[si].alpha;
    assert_eq!(alpha[0], 130.70932140, "the defect's starting value has moved");
    alpha[0] = 130.70932000;
    shells[si] = Shell { alpha, ..shells[si] };
    let mutated = Species {
        shells: Box::leak(shells.into_boxed_slice()),
        ..o
    };
    let mut table: Vec<Species> = ALL_ELEMENTS.to_vec();
    table[7] = mutated;

    let (worst, where_) = worst_ratio_deviation(&table);
    assert!(
        worst >= RATIO_MARGIN,
        "PLANT MISSED: the historical oxygen transcription defect reached only {worst:.2}x \
         its rounding bound, under the {RATIO_MARGIN:.0}x the gate fires at"
    );
    println!("plant (oxygen, historical): fires at {worst:.1}x, at {where_}");
}

/// Every declared number is the pinned tabulation's, and every tabulated shell is declared.
///
/// # What this adds that the structural gates cannot
///
/// The ratio band and the universality check are STRUCTURAL: they test the declaration
/// against itself, which is what makes them independent evidence but also what limits them.
/// They would pass on a table that was internally consistent and wholly invented, and they
/// cannot see an error that moves a whole shell coherently.
///
/// So the digits are also checked directly against `tests/data/sto3g_tabulation.txt`, which
/// is the Basis Set Exchange's STO-3G flattened to one line per contraction. That file
/// deliberately carries NO principal quantum number: it is `Z`, angular momentum,
/// coefficients and exponents, at the source's own precision. Withholding the `(n, l)`
/// label is the point -- this gate checks the digits without inheriting the generator's
/// opinion about which shell is which, and the labelling is checked separately and
/// structurally by the ratio families.
///
/// The bijection matters as much as the values: a shell dropped from the registry or one
/// invented in it would leave the numbers correct and the model wrong.
#[test]
fn every_declared_number_is_the_pinned_tabulations() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/sto3g_tabulation.txt");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

    // (z, l, coeffs, exponents), at the source's precision.
    let mut rows: Vec<(u32, u8, [f64; 3], [f64; 3], bool)> = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let v: Vec<f64> = line
            .split_whitespace()
            .map(|x| x.parse().expect("tabulation row is numeric"))
            .collect();
        assert_eq!(v.len(), 8, "tabulation row must be Z l c0 c1 c2 a0 a1 a2: {line}");
        rows.push((
            v[0] as u32,
            v[1] as u8,
            [v[2], v[3], v[4]],
            [v[5], v[6], v[7]],
            false,
        ));
    }
    assert!(
        rows.len() >= 400,
        "the pinned tabulation carries only {} rows; a gate driven by a file that lost its \
         contents checks nothing",
        rows.len()
    );

    // Half a unit in the last place of an eight-decimal declaration. A rounding TIE may
    // legitimately fall either way, so the bound is inclusive of exactly half a ulp; a
    // one-digit error in the last declared place is a whole ulp and clears it.
    const COEFF_BOUND: f64 = 0.5e-8 + 1e-13;

    for sp in ALL_ELEMENTS {
        for sh in sp.shells {
            let l = sh.kind.l();
            // Match on Z, angular momentum, and the nearest leading exponent -- never on
            // the principal quantum number, which is exactly the label under test.
            let hit = rows
                .iter_mut()
                .filter(|r| r.0 == sp.z && r.1 == l && !r.4)
                .min_by(|a, b| {
                    (a.3[0] - sh.alpha[0])
                        .abs()
                        .partial_cmp(&(b.3[0] - sh.alpha[0]).abs())
                        .unwrap()
                })
                .unwrap_or_else(|| {
                    panic!(
                        "{} declares a {:?} shell with no matching l = {l} contraction in \
                         the pinned tabulation",
                        sp.symbol, sh.kind
                    )
                });
            hit.4 = true;

            for i in 0..3 {
                let bound = declaration_bound(hit.3[i]) + 1e-13;
                let d = (sh.alpha[i] - hit.3[i]).abs();
                assert!(
                    d <= bound,
                    "{}'s {:?} exponent {i} is {} but the pinned tabulation says {} \
                     (differs by {d:.3e}, and the declaration's own rounding allows only \
                     {bound:.3e})",
                    sp.symbol,
                    sh.kind,
                    sh.alpha[i],
                    hit.3[i]
                );
                let dc = (sh.coeff[i] - hit.2[i]).abs();
                assert!(
                    dc <= COEFF_BOUND,
                    "{}'s {:?} coefficient {i} is {} but the pinned tabulation says {} \
                     (differs by {dc:.3e})",
                    sp.symbol,
                    sh.kind,
                    sh.coeff[i],
                    hit.2[i]
                );
            }
        }
    }

    let unused: Vec<String> = rows
        .iter()
        .filter(|r| !r.4)
        .map(|r| format!("Z={} l={} alpha0={}", r.0, r.1, r.3[0]))
        .collect();
    assert!(
        unused.is_empty(),
        "{} contractions in the pinned tabulation are not declared by any element: {:?}. \
         A dropped shell leaves every remaining number right and the model wrong.",
        unused.len(),
        &unused[..unused.len().min(6)]
    );
    println!(
        "tabulation pin: {} contractions, every declared number matched and every \
         tabulated shell claimed",
        rows.len()
    );
}

/// Oxygen specifically, pinned by the value the ratio test selects.
#[test]
fn oxygens_leading_exponent_is_the_published_one() {
    let o = by_z(8).unwrap();
    let s1 = o.shells.iter().find(|s| s.kind == ShellKind::S1).unwrap();
    assert_eq!(
        s1.alpha[0], 130.70932140,
        "oxygen's leading 1s exponent was once transcribed as 130.70932000, which moved \
         the oxygen atom by 6.3e-9 hartree — 63x the referee gate's stake"
    );
}
