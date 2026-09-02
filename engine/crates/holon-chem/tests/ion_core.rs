//! GANTT node C: H3O+ and OH− EXIST, certified, and everything unstated is refused.
//!
//! # Every number here is MEASURED, and none is quoted
//!
//! There is no literature energy in this file and no experimental proton affinity. The
//! model is STO-3G full CI, the geometries are DECLARED INPUTS staked below, and what the
//! tests certify is that the charged solves run, converge, and stand in the variational
//! relations to their neutral fragments that this model actually produces. Whether those
//! relations match nature is a different question and this file does not touch it — the
//! crate header's "exact-in-model, never compared to experiment" discipline applies here
//! exactly as it does to H2.
//!
//! # What the two variational gates are, and why they are the load-bearing ones
//!
//! A charged solve is easy to get plausibly wrong: a bad electron count, an `S_z` sector
//! that seats the wrong number of alphas, an anion whose extra electron went nowhere. None
//! of those shows up as a crash and all of them produce a number of about the right size.
//! What DOES catch them is a relation between two solves that share everything except the
//! charge:
//!
//! * **Proton affinity.** `E(H3O+) < E(H2O)`, taking `E(H+) = 0` by convention — a bare
//!   proton has no electrons and, alone, no nuclear repulsion, so its energy in this model
//!   is exactly zero and the fragment sum reduces to water's own energy.
//! * **Electron affinity.** `E(OH−) < E(OH)`, at the SAME staked separation, so the
//!   comparison is vertical: one solve differs from the other only by the electron that
//!   was added.
//!
//! **If either inequality fails, that is the RESULT** — a measured fact about this model,
//! not a bug to be tuned away. Nothing in this file adjusts a geometry, a sector or a basis
//! to make a gate pass.
//!
//! # ONE OF THE TWO FIRED, and this is the headline
//!
//! * Proton affinity **PASSES**: `E(H2O) - E(H3O+) = +0.379432332077 Ha` at the staked
//!   geometries.
//! * Electron affinity **FIRES**: `E(OH) - E(OH-) = -0.305545907904 Ha` at `r = 1.83` bohr.
//!   OH− sits 0.3055 hartree ABOVE neutral OH in this model.
//!
//! The fired reading is kept, marked, and PINNED two-sided rather than reversed into a
//! green assertion — see `the_electron_affinity_gate_fired_oh_minus_sits_above_neutral_oh`,
//! which also carries the discriminator that attributes it to the DECLARED BASIS (STO-3G
//! has no diffuse functions) rather than to the charged seam. `ION_STAKING.md` records what
//! that constrains downstream.
//!
//! # The geometries are STAKED, not optimised
//!
//! Written down before any of these solves was run, from the ordinary shapes of these
//! species. No geometry optimisation happens anywhere in this file, so every energy here
//! is an energy AT A POINT and an upper bound on that species' minimum in this model. That
//! matters for how the gates may be read, and the note on each gate says how.

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::fci::{SolveExit, SolverRoute};
use holon_chem::ions::{solve_geometry_charged, spin_partition, ChargeRefusal};
use holon_chem::pair::{solve_geometry, PointSolution};

// ------------------------------------------------------------------ the staked geometry
//
// Bohr and degrees. DECLARED INPUTS: ordinary shapes for these species, written down in
// advance, never fitted and never relaxed.

/// O–H separation in H3O+.
const R_OH_H3O: f64 = 1.85;
/// H–O–H angle in H3O+, degrees. The pyramid's polar angle is DERIVED from it below
/// rather than stated separately, so the two cannot drift apart.
const ANGLE_H3O_DEG: f64 = 113.0;
/// O–H separation in H2O.
const R_OH_H2O: f64 = 1.81;
/// H–O–H angle in H2O, degrees.
const ANGLE_H2O_DEG: f64 = 104.5;
/// O–H separation used for BOTH OH and OH−, so the electron-affinity gate is vertical.
const R_OH_DIATOMIC: f64 = 1.83;
/// H–H separation for the neutrality regression on H2.
const R_HH: f64 = 1.4;

fn c3(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

/// C3v H3O+: oxygen at the origin, three hydrogens on a cone about `+z`.
///
/// The cone's polar angle `t` is derived from the staked H–O–H angle: two unit vectors at
/// polar angle `t` and azimuths differing by 120° have dot product
/// `cos^2 t - (1/2) sin^2 t = (3 cos^2 t - 1) / 2`, so `cos^2 t = (2 cos(HOH) + 1) / 3`.
fn h3o_plus_centers() -> Vec<[D2; 3]> {
    let cos_hoh = ANGLE_H3O_DEG.to_radians().cos();
    let cos_t = ((2.0 * cos_hoh + 1.0) / 3.0).sqrt();
    let sin_t = (1.0 - cos_t * cos_t).sqrt();
    let mut centers = vec![c3(0.0, 0.0, 0.0)];
    for k in 0..3 {
        let phi = 2.0 * std::f64::consts::PI * (k as f64) / 3.0;
        centers.push(c3(
            R_OH_H3O * sin_t * phi.cos(),
            R_OH_H3O * sin_t * phi.sin(),
            R_OH_H3O * cos_t,
        ));
    }
    centers
}

fn h2o_centers() -> Vec<[D2; 3]> {
    let a = ANGLE_H2O_DEG.to_radians();
    vec![
        c3(0.0, 0.0, 0.0),
        c3(R_OH_H2O, 0.0, 0.0),
        c3(R_OH_H2O * a.cos(), R_OH_H2O * a.sin(), 0.0),
    ]
}

fn oh_centers() -> Vec<[D2; 3]> {
    vec![c3(0.0, 0.0, 0.0), c3(R_OH_DIATOMIC, 0.0, 0.0)]
}

/// Print everything the solve reported, and require the three things that make a number
/// usable: the SCF converged, the CI exit says it converged (or the space was trivial),
/// and — when the route computed it — the variational margin is non-negative.
fn certify(label: &str, sol: &PointSolution) -> f64 {
    println!(
        "  {label:<10} E = {:.12} Ha   n_basis {:>2}  n_det {:>6}  route {:?}  exit {:?}  \
         davidson {:>3}  residual {:.2e}  s_min {:.3e}  scf {}",
        sol.e.v,
        sol.n_basis,
        sol.n_det,
        sol.route,
        sol.exit,
        sol.davidson_iters,
        sol.residual,
        sol.s_min_eigenvalue,
        sol.scf_converged,
    );
    assert!(sol.scf_converged, "{label}: the orbital rotation did not converge");
    assert!(
        matches!(sol.exit, SolveExit::Converged | SolveExit::Trivial),
        "{label}: the CI solve exited {:?}, which is not a converged answer",
        sol.exit
    );
    assert!(
        sol.e.v.is_finite(),
        "{label}: energy is not finite ({})",
        sol.e.v
    );
    sol.e.v
}

// ----------------------------------------------------------------------- (a) H3O+ EXISTS

#[test]
fn h3o_plus_solves_at_the_staked_pyramid() {
    println!("H3O+ : charge +1, 8 protons + 3 = 11 nuclear charge, 10 electrons");
    println!(
        "  staked: r(O-H) = {R_OH_H3O} bohr, angle(H-O-H) = {ANGLE_H3O_DEG} deg, C3v"
    );
    let sol = solve_geometry_charged(
        &[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN],
        h3o_plus_centers(),
        1,
    )
    .expect("H3O+ is a stateable charge on this species list");
    certify("H3O+", &sol);

    // The electron count is the whole point of the seam, so it is checked rather than
    // assumed: 11 nuclear charge minus 1 is 10, and the parity rule seats 5 and 5.
    let (na, nb) = spin_partition(10);
    assert_eq!((na, nb), (5, 5), "10 electrons must seat as a singlet");
    println!("  sector: n_alpha {na}, n_beta {nb} (even count -> singlet, by the stated rule)");
}

// ------------------------------------------------------------------------ (b) OH− EXISTS

#[test]
fn oh_minus_solves_at_the_staked_separation() {
    println!("OH-  : charge -1, 9 nuclear charge, 10 electrons");
    println!("  staked: r(O-H) = {R_OH_DIATOMIC} bohr");
    let sol = solve_geometry_charged(&[OXYGEN, HYDROGEN], oh_centers(), -1)
        .expect("OH- is a stateable charge on this species list");
    certify("OH-", &sol);

    let (na, nb) = spin_partition(10);
    assert_eq!((na, nb), (5, 5), "10 electrons must seat as a singlet");
    println!("  sector: n_alpha {na}, n_beta {nb} (even count -> singlet, by the stated rule)");

    // And the neutral it is compared against seats 5/4 by the odd branch of the same rule.
    assert_eq!(spin_partition(9), (5, 4), "9 electrons must seat as a doublet");
}

// -------------------------------------------------- (c) the two load-bearing variational gates

/// The MEASURED proton affinity at the staked geometries, hartree: `E(H2O) - E(H3O+)`.
/// POSITIVE — the gate passes — and pinned so that a passing gate is also a pinned number.
const PROTON_AFFINITY_MEASURED: f64 = 0.379432332077;

#[test]
fn proton_affinity_is_positive_in_this_model() {
    println!("GATE: E(H3O+) < E(H2O) + E(H+), with E(H+) = 0 by convention");
    println!(
        "  E(H+) = 0 is not an approximation here: a bare proton has zero electrons and, \n\
         \x20 alone, zero nuclear repulsion, so its energy in this model is exactly zero."
    );
    let e_h3o = certify(
        "H3O+",
        &solve_geometry_charged(
            &[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN],
            h3o_plus_centers(),
            1,
        )
        .expect("H3O+ solves"),
    );
    let e_h2o = certify("H2O", &solve_geometry(&[OXYGEN, HYDROGEN, HYDROGEN], h2o_centers()));

    let affinity = e_h2o - e_h3o;
    println!(
        "  proton affinity = E(H2O) - E(H3O+) = {:.12} - {:.12} = {:+.12} Ha",
        e_h2o, e_h3o, affinity
    );
    println!(
        "  READ IT AS: both energies are AT STAKED POINTS, not at relaxed minima, so this \n\
         \x20 is the proton affinity of THESE TWO GEOMETRIES in THIS model. It is not a \n\
         \x20 bound on the relaxed one in either direction."
    );
    assert!(
        affinity > 0.0,
        "MEASURED MODEL FACT, reported as the result: at the staked geometries this model \
         puts H3O+ ABOVE H2O by {:.12} hartree — E(H3O+) = {:.12}, E(H2O) = {:.12}. \
         Nothing here has been tuned to avoid this; the gate is stated so that if the model \
         says it, the model gets to say it.",
        -affinity,
        e_h3o,
        e_h2o
    );
    // The gate PASSED, so the number is pinned too. A gate that only ever checks a sign
    // cannot notice the day the seam starts producing a plausible wrong magnitude.
    assert!(
        (affinity - PROTON_AFFINITY_MEASURED).abs() < AFFINITY_PIN_TOLERANCE,
        "the pinned proton affinity moved: measured {affinity:+.12}, pinned \
         {PROTON_AFFINITY_MEASURED:+.12} Ha"
    );
}

/// The MEASURED electron affinity of OH in this model, hartree: `E(OH) - E(OH-)`.
///
/// **NEGATIVE. The staked gate `E(OH-) < E(OH)` FIRED**, and this constant is the fired
/// reading kept in the record rather than an assertion that was quietly reversed. It is
/// pinned two-sided below so the number cannot drift without the suite saying so.
const OH_ELECTRON_AFFINITY_MEASURED: f64 = -0.305545907904;

/// The same reading for the simplest possible anion, `E(H) - E(H-)`, hartree — the
/// discriminator described on the test below. Also negative.
const H_ELECTRON_AFFINITY_MEASURED: f64 = -0.308024094363;

/// How far either pinned affinity may move before this test calls it a change. Both solves
/// exit at Davidson residuals near 7e-11, so a shift at 1e-8 is a change in the model or the
/// code, not in the arithmetic.
const AFFINITY_PIN_TOLERANCE: f64 = 1e-8;

/// **THE GATE FIRED.** `E(OH-) < E(OH)` is false in this model, by 0.3055 hartree.
///
/// This is reported as the result, which is what the brief asked for: nothing here was
/// tuned to make the inequality hold, no geometry was relaxed to chase it and the basis was
/// not changed. The staked criterion is still stated in the test — it has to be, or the
/// fired reading has nothing to be a reading OF — and what the assertions now enforce is
/// the MEASUREMENT: the sign, and the magnitude, two-sided.
///
/// # The discriminator, because a fired gate is a place to look and not yet a verdict
///
/// "OH- came out above OH" has two very different causes and they must not be conflated:
///
/// 1. **The declared basis.** STO-3G carries no diffuse functions, so an extra electron has
///    nowhere loosely bound to go and pays repulsion it cannot recover. If this is the
///    cause, EVERY anion in this basis reads the same way.
/// 2. **The charged seam.** A wrong electron count or a wrong `S_z` sector for the anion
///    would also put it too high, and would be a defect in the module under test.
///
/// The test separates them. It measures the same quantity on H-/H — a system with ONE
/// orbital, where the anion's CI space is a single determinant and there is nothing for a
/// sector rule to get wrong — and gets the same sign and nearly the same magnitude. And
/// `proton_affinity_is_positive_in_this_model` drives the identical seam for a CATION and
/// comes back correctly signed and large. A defect in the seam would have to be one that
/// spares cations, spares a one-determinant anion, and hits only OH-.
///
/// # What would discharge it
///
/// A basis with diffuse functions, which this crate does not have and which is not this
/// node's work. Entered in `conformance/water_observatory/ION_STAKING.md` with its
/// receipt-gate: the reading is that the model's ANION energies are not to be used as
/// affinities, and that is a live constraint on every downstream ion table.
#[test]
fn the_electron_affinity_gate_fired_oh_minus_sits_above_neutral_oh() {
    println!("STAKED GATE: E(OH-) < E(OH), both at r = {R_OH_DIATOMIC} bohr (VERTICAL)");
    let e_anion = certify(
        "OH-",
        &solve_geometry_charged(&[OXYGEN, HYDROGEN], oh_centers(), -1).expect("OH- solves"),
    );
    let e_neutral = certify("OH", &solve_geometry(&[OXYGEN, HYDROGEN], oh_centers()));

    let affinity = e_neutral - e_anion;
    println!(
        "  electron affinity = E(OH) - E(OH-) = {:.12} - {:.12} = {:+.12} Ha",
        e_neutral, e_anion, affinity
    );
    println!("  >>> THE GATE FIRED: the affinity is NEGATIVE. Reported as the result.");

    // The discriminator: the same measurement where the anion's CI space is ONE
    // determinant, so no sector rule and no electron count can be responsible for it.
    let origin = || vec![c3(0.0, 0.0, 0.0)];
    let e_h_anion = certify(
        "H-",
        &solve_geometry_charged(&[HYDROGEN], origin(), -1).expect("H- solves"),
    );
    let e_h = certify("H", &solve_geometry(&[HYDROGEN], origin()));
    let h_affinity = e_h - e_h_anion;
    println!(
        "  control: E(H) - E(H-) = {:.12} - {:.12} = {:+.12} Ha  (same sign, one determinant)",
        e_h, e_h_anion, h_affinity
    );
    println!(
        "  READ IT AS: the minimal basis has no diffuse function for the extra electron, \n\
         \x20 and the one-orbital control fires the same way. The cause is the DECLARED \n\
         \x20 BASIS, not the charged seam — whose cation gate passes on the same code path."
    );

    assert!(
        affinity < 0.0,
        "the fired reading has changed SIGN: E(OH) - E(OH-) is now {affinity:+.12} Ha. \
         That is a new result and must be re-read, not absorbed by this pin."
    );
    assert!(
        (affinity - OH_ELECTRON_AFFINITY_MEASURED).abs() < AFFINITY_PIN_TOLERANCE,
        "the pinned OH affinity moved: measured {affinity:+.12}, pinned \
         {OH_ELECTRON_AFFINITY_MEASURED:+.12} Ha"
    );
    assert!(
        h_affinity < 0.0,
        "the one-determinant control no longer fires: E(H) - E(H-) = {h_affinity:+.12} Ha. \
         The discriminator that attributes the OH- reading to the BASIS rather than to the \
         charged seam has stopped holding, and the attribution must be re-made."
    );
    assert!(
        (h_affinity - H_ELECTRON_AFFINITY_MEASURED).abs() < AFFINITY_PIN_TOLERANCE,
        "the pinned H affinity moved: measured {h_affinity:+.12}, pinned \
         {H_ELECTRON_AFFINITY_MEASURED:+.12} Ha"
    );
}

// --------------------------------------------------------- (d) the neutrality regression

/// `charge == 0` must be the SAME CALL, not a similar one — asserted on the raw bits.
///
/// A tolerance here would pass against a re-implementation of the neutral path that agreed
/// to fifteen digits and then diverged the first time either side changed. Bit equality is
/// the only assertion that says "this is not a second copy".
#[test]
fn charge_zero_is_bit_identical_to_the_neutral_path() {
    let cases: Vec<(&str, Vec<_>, Vec<[D2; 3]>)> = vec![
        (
            "H2",
            vec![HYDROGEN, HYDROGEN],
            vec![c3(0.0, 0.0, 0.0), c3(R_HH, 0.0, 0.0)],
        ),
        ("H2O", vec![OXYGEN, HYDROGEN, HYDROGEN], h2o_centers()),
    ];
    for (label, species, centers) in cases {
        let neutral = solve_geometry(&species, centers.clone());
        let charged = solve_geometry_charged(&species, centers, 0)
            .expect("charge 0 is always stateable on a non-empty species list");
        println!(
            "  {label:<4} neutral {:.17e}  bits {:#018x}   charged(0) {:.17e}  bits {:#018x}",
            neutral.e.v,
            neutral.e.v.to_bits(),
            charged.e.v,
            charged.e.v.to_bits()
        );
        assert_eq!(
            neutral.e.v.to_bits(),
            charged.e.v.to_bits(),
            "{label}: solve_geometry_charged(.., 0) is not bit-identical to solve_geometry. \
             The charged entry point has become a SECOND implementation of the neutral path."
        );
        // The derivative slots and the diagnostics too: an energy that matched while the
        // reported determinant count differed would mean two different CI spaces landed on
        // the same number, which is not the claim.
        assert_eq!(
            neutral.e.d.to_bits(),
            charged.e.d.to_bits(),
            "{label}: first derivative slot differs"
        );
        assert_eq!(
            neutral.e.e.to_bits(),
            charged.e.e.to_bits(),
            "{label}: second derivative slot differs"
        );
        assert_eq!(neutral.n_det, charged.n_det, "{label}: determinant count differs");
        assert_eq!(neutral.n_basis, charged.n_basis, "{label}: basis size differs");
        assert_eq!(
            neutral.davidson_iters, charged.davidson_iters,
            "{label}: iteration count differs — the two paths did not run the same solve"
        );
        assert_eq!(neutral.route, charged.route, "{label}: solver route differs");
        assert!(matches!(neutral.route, SolverRoute::Determinant));
    }
}

// ------------------------------------------------------------------ (e) the refusal plants

#[test]
fn a_charge_beyond_the_nuclear_charge_refuses_as_negative_electrons() {
    // Sanity first, so the refusal below cannot be passing because the whole call path is
    // broken: the same species at a stateable charge comes back Ok.
    assert!(
        solve_geometry_charged(&[HYDROGEN], vec![c3(0.0, 0.0, 0.0)], 0).is_ok(),
        "the neutral hydrogen atom must solve, or this plant proves nothing"
    );

    let refused = solve_geometry_charged(&[HYDROGEN], vec![c3(0.0, 0.0, 0.0)], 2);
    println!("  H with charge +2 -> {refused:?}", refused = refused.as_ref().err());
    assert_eq!(
        refused.err(),
        Some(ChargeRefusal::NegativeElectrons {
            total_z: 1,
            charge: 2,
            would_be_electrons: -1,
        }),
        "a charge past the nuclear charge must refuse as NegativeElectrons"
    );

    // The PARTITION contract: a large positive charge is the cation half and must not come
    // back as ChargeTooLarge. Placing `|charge| > sum(Z)` first would make it, and would
    // silently retire the NegativeElectrons variant.
    let big = solve_geometry_charged(&[OXYGEN], vec![c3(0.0, 0.0, 0.0)], 9);
    println!("  O with charge +9 -> {:?}", big.as_ref().err());
    assert!(
        matches!(big.err(), Some(ChargeRefusal::NegativeElectrons { .. })),
        "the cation half of |charge| > sum(Z) must name NegativeElectrons, not ChargeTooLarge"
    );
}

#[test]
fn an_anion_beyond_the_nuclear_charge_refuses_as_charge_too_large() {
    // Sanity: H- IS stateable — two electrons in one orbital — so the refusal below is
    // about the magnitude of the charge and not about anions in general.
    let h_minus = solve_geometry_charged(&[HYDROGEN], vec![c3(0.0, 0.0, 0.0)], -1)
        .expect("H- has 2 electrons in 1 orbital and is stateable");
    println!(
        "  H- (stateable): E = {:.12} Ha, n_det {}, n_basis {}",
        h_minus.e.v, h_minus.n_det, h_minus.n_basis
    );

    let refused = solve_geometry_charged(&[HYDROGEN], vec![c3(0.0, 0.0, 0.0)], -2);
    println!("  H with charge -2 -> {:?}", refused.as_ref().err());
    assert_eq!(
        refused.err(),
        Some(ChargeRefusal::ChargeTooLarge { total_z: 1, charge: -2 }),
        "an anion carrying more excess electrons than the system has protons must refuse"
    );
}

#[test]
fn a_sector_the_basis_cannot_seat_refuses_as_unstated_spin_sector() {
    // Sanity: O2- IS stateable — 10 electrons in 5 orbitals is exactly full.
    let o2m = solve_geometry_charged(&[OXYGEN], vec![c3(0.0, 0.0, 0.0)], -2)
        .expect("10 electrons in 5 STO-3G orbitals is exactly seatable");
    println!(
        "  O2- (stateable, the basis is exactly full): E = {:.12} Ha, n_det {}, n_basis {}",
        o2m.e.v, o2m.n_det, o2m.n_basis
    );

    // One electron more. |charge| = 3 <= Z = 8, so this is NOT ChargeTooLarge; the
    // arithmetic is fine and it is the SECTOR that does not exist in this basis.
    let refused = solve_geometry_charged(&[OXYGEN], vec![c3(0.0, 0.0, 0.0)], -3);
    println!("  O with charge -3 -> {:?}", refused.as_ref().err());
    assert_eq!(
        refused.err(),
        Some(ChargeRefusal::UnstatedSpinSector {
            n_electrons: 11,
            n_orbitals: 5,
            n_alpha: 6,
        }),
        "a parity sector the declared basis cannot seat must refuse rather than build a \
         zero-determinant CI space"
    );
}
