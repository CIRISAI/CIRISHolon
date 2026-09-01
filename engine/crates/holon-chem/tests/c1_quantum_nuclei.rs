//! C1's REAL GATE: quantum nuclei on the banked H-H curve.
//!
//! Staked in `conformance/water_observatory/C1_GATE_PREREG.md` before it ran. This file
//! is the CI-affordable half — the plants, the refusals, the classical limit and the
//! bead-forgetting square all run here in seconds — plus the pinned campaign numbers,
//! which are checked TWO-SIDED against a reference recomputed live on every run.
//!
//! The long sampling that produced the pins is `examples/c1_campaign.rs`; a pin that
//! drifts from the live reference fails here whether the drift is up or down.

use holon_chem::elements::Species;
use holon_chem::h2::{equilibrium, h2_point};
use holon_chem::rpmd::*;
use holon_chem::tower::{ClassicalState, RingPolymerState};

fn mu_h2() -> f64 {
    Vib1D::reduced_mass_me(Species::HYDROGEN.mass_u, Species::HYDROGEN.mass_u)
}

fn omega_harm() -> f64 {
    let (r_e, _, _) = equilibrium();
    (h2_point(r_e).e2 / mu_h2()).sqrt()
}

// ---------------------------------------------------------------- the plants

/// P1: the referee on a HARMONIC plant, whose spectrum is `omega (n + 1/2)` exactly.
///
/// This plant is deliberately ZERO in the anharmonic sector — it certifies the kinetic
/// operator, the box and the grid, and it certifies NOTHING about the quantity the
/// zero-point-energy gate reads. `test_referee_reproduces_the_morse_plant` is the one
/// that acts in that sector.
#[test]
fn test_referee_reproduces_the_harmonic_plant() {
    let (r_e, _, v_min) = equilibrium();
    let mu = mu_h2();
    let w = omega_harm();
    let pes = HarmonicPes { k: h2_point(r_e).e2, r0: r_e, v0: v_min };
    let sys = Vib1D { mu, pes: &pes, name: "harmonic" };
    let r = dvr_reference(&sys, r_e - 3.0, r_e + 3.0, f64::NEG_INFINITY, 401, 3, 1e-8)
        .expect("the harmonic plant must certify the referee");
    assert!(r.solves >= 4, "the referee must do its work, not report it: {} solves", r.solves);
    assert!(r.potential_calls > 1000, "work count: {}", r.potential_calls);
    for n in 0..3 {
        let exact = v_min + w * (n as f64 + 0.5);
        assert!(
            (r.levels[n] - exact).abs() < 1e-12,
            "harmonic level {n}: DVR {} vs exact {} (d {:.3e})",
            r.levels[n], exact, r.levels[n] - exact
        );
    }
}

/// P2: the referee on a MORSE plant — anharmonic, exact, and NONZERO IN THE SECTOR THE
/// GATE READS (M-PLANT-SECTOR). Its zero-point energy sits 1.4% below the harmonic one,
/// which is 3.5x the band `G1` allows, so a referee blind to anharmonicity fails here.
#[test]
fn test_referee_reproduces_the_morse_plant() {
    let (r_e, d_e, v_min) = equilibrium();
    let mu = mu_h2();
    let w = omega_harm();
    let a = w * (mu / (2.0 * d_e)).sqrt();
    let pes = MorsePes { d_e, a, r_e, v0: v_min };
    let sys = Vib1D { mu, pes: &pes, name: "morse" };
    let r = dvr_reference(&sys, r_e - 1.0, r_e + 4.0, f64::NEG_INFINITY, 401, 3, 1e-8)
        .expect("the Morse plant must certify the referee");
    let exact = morse_levels(&pes, mu, 3);
    for n in 0..3 {
        assert!(
            (r.levels[n] - v_min - exact[n]).abs() < 1e-12,
            "Morse level {n}: DVR {} vs exact {} (d {:.3e})",
            r.levels[n], v_min + exact[n], r.levels[n] - v_min - exact[n]
        );
    }
    // The planted displacement, and the check that it is observable at all.
    let planted = exact[0] - 0.5 * w;
    assert!(planted < 0.0, "Morse anharmonicity must lower the zero point: {planted:.3e}");
    assert!(
        planted.abs() / (0.5 * w) > 3.0 * 0.004,
        "the plant must exceed 3x G1's 0.40% band or it certifies nothing: {:.4}%",
        100.0 * planted.abs() / (0.5 * w)
    );
}

/// P3: the sampler against the EXACT `P`-bead ring-polymer energy of a harmonic
/// oscillator — the plant that is nonzero in the BEAD-NUMBER sector, which is the axis
/// `G3`'s convergence ladder is a claim about.
#[test]
fn test_pimd_reproduces_the_exact_ring_polymer_energy() {
    let (r_e, _, v_min) = equilibrium();
    let mu = mu_h2();
    let w = omega_harm();
    let k = h2_point(r_e).e2;
    let beta = 1.0 / (K_B_HARTREE_PER_KELVIN * 300.0);
    for &p in &[1usize, 8, 32] {
        let cfg = PimdConfig {
            p,
            temperature_k: 300.0,
            dt: 4.0,
            gamma_centroid: w,
            steps_equil: 20_000,
            steps_sample: 200_000,
            seed: 0xC1_00_17,
        };
        let rep = run_pimd_chains(mu, "harmonic", &cfg, 4, r_e, &|| {
            Box::new(HarmonicPes { k, r0: r_e, v0: v_min })
        });
        let exact = v_min + harmonic_ring_energy(w, beta, p);
        let d = rep.e_virial - exact;
        assert!(
            d.abs() < 6.0 * rep.e_virial_err.max(2e-6),
            "P={p}: E_cv {} vs exact E_P {} (d {:+.3e}, err {:.3e})",
            rep.e_virial, exact, d, rep.e_virial_err
        );
        // The two estimators of one quantity must agree; neither can check itself.
        let g = (rep.e_primitive - rep.e_virial).abs();
        let tol = 6.0 * (rep.e_virial_err.powi(2) + rep.e_primitive_err.powi(2)).sqrt();
        assert!(g < tol.max(2e-5), "P={p}: primitive-virial gap {g:.3e} > {tol:.3e}");
        assert_eq!(rep.chains, 4);
        assert!(rep.samples >= 800_000, "work count: {} samples", rep.samples);
    }
}

/// `P = 1` is `kT` and `P -> inf` is the quantum thermal energy. The two limits of the
/// closed form the ladder is graded against, checked as arithmetic.
#[test]
fn test_harmonic_ring_energy_limits() {
    let beta = 1.0 / (K_B_HARTREE_PER_KELVIN * 300.0);
    let w = omega_harm();
    assert!((harmonic_ring_energy(w, beta, 1) - 1.0 / beta).abs() < 1e-15);
    let quantum = harmonic_exact_energy(w, beta);
    let big = harmonic_ring_energy(w, beta, 8192);
    assert!(
        (big - quantum).abs() / quantum < 1e-5,
        "P=8192 {big:.12} vs quantum {quantum:.12}"
    );
    // The P^-2 law, with the coefficient this test CORRECTED. The module first documented
    // `E_P = omega/2 - beta^2 omega^3 / (48 P^2)`; the closed form's own deficit is
    // exactly 3x that, and this line is what said so — the ratio
    // (E_inf - E_P) / (beta^2 omega^3 / (16 P^2)) converges to 1.0000 by P = 4096 and
    // reads 0.9984 at P = 512. The 1/48 came from expanding arcsinh in a form of the
    // partition function that treats the ring frequency as independent of beta, which it
    // is not; the sum in `harmonic_ring_energy` is derived by differentiating the exact
    // Gaussian path integral and is the one that gives kT at P = 1.
    for &p in &[512usize, 1024, 2048] {
        let deficit = harmonic_exact_energy(w, beta) - harmonic_ring_energy(w, beta, p);
        let predicted = beta * beta * w * w * w / (16.0 * (p * p) as f64);
        assert!(
            (deficit / predicted - 1.0).abs() < 0.01,
            "P={p}: deficit {deficit:.6e} vs P^-2 law {predicted:.6e} (ratio {:.5})",
            deficit / predicted
        );
    }
    // And the classical limit, which is what convicts the wrong closed form: any formula
    // for E_P that does not give exactly kT at P = 1 is not the energy of this ensemble.
    assert!((harmonic_ring_energy(w, beta, 1) * beta - 1.0).abs() < 1e-14);
}

// ---------------------------------------------------------------- the refusals

/// The referee REFUSES a box that does not hold the state it was asked for, and the
/// refusal names WHICH check fired (M-EXIT-DISCRIMINATOR). Proven in both directions:
/// the same system in an adequate box is admitted by the same call.
#[test]
fn test_referee_refuses_a_box_that_does_not_hold_the_state() {
    let (r_e, _, v_min) = equilibrium();
    let mu = mu_h2();
    let pes = HarmonicPes { k: h2_point(r_e).e2, r0: r_e, v0: v_min };
    let sys = Vib1D { mu, pes: &pes, name: "harmonic" };
    // A box clipped hard against the sixth level's turning points.
    let bad = dvr_reference(&sys, r_e - 0.55, r_e + 0.55, f64::NEG_INFINITY, 201, 6, 1e-9);
    match bad {
        // EITHER convergence axis is an honest refusal here, and the reason is worth
        // stating: in a sinc DVR the box truncation IS the boundary condition, so a box
        // that does not hold the state also moves under a grid halving. The two axes are
        // NOT independent when the state leaks, and this test measured that rather than
        // assuming it — the first version demanded a BOX refusal and got a GRID one
        // (1.155e-3). What the referee must do is refuse, name an axis, and carry its
        // numbers; which of the two coupled axes it names is not a fact about the
        // instrument's health.
        Err(RefereeRefusal::BoxNotConverged { shift, tolerance })
        | Err(RefereeRefusal::GridNotConverged { shift, tolerance }) => {
            assert!(shift > tolerance, "the refusal must carry its own numbers");
        }
        Err(other) => panic!("expected a convergence refusal, got {other}"),
        Ok(r) => panic!("a 1.1-bohr box must not certify 6 levels: E0 {}", r.levels[0]),
    }
    // The SAME call in an adequate box is admitted, so the refusal is discriminating and
    // not a solver that always says no.
    assert!(dvr_reference(&sys, r_e - 3.0, r_e + 3.0, f64::NEG_INFINITY, 401, 6, 1e-8).is_ok());
}

/// The `r_floor` fence: widening a box is never allowed to leave the surface's domain.
/// The banked table's inner knot is that floor, and a run that extrapolates says so.
#[test]
fn test_banked_surface_reports_its_excursions() {
    let banked = BankedPes::h2(1024);
    assert_eq!(banked.excursions(), 0);
    let (lo, hi) = banked_range();
    let _ = banked.eval(lo * 0.5);
    assert_eq!(banked.excursions(), 1, "an out-of-range evaluation must be counted");
    let _ = banked.eval(hi * 2.0);
    assert_eq!(banked.excursions(), 2);
    banked.reset_excursions();
    let _ = banked.eval(0.5 * (lo + hi));
    assert_eq!(banked.excursions(), 0, "an in-range evaluation must NOT be counted");
}

// ---------------------------------------------------------------- the interpolant

/// G2: the banked interpolant is not the answer. Its departure from the solver it was
/// built from is measured in the gate's own currency and must be far inside G1's band.
#[test]
fn test_banked_interpolant_tracks_the_solver() {
    let (r_e, _, v_min) = equilibrium();
    let banked = BankedPes::h2(4096);
    let (de, df) = banked.table().hermite_error(4);
    assert!(de < 1e-8, "interpolant energy error {de:.3e} Ha");
    assert!(df < 1e-7, "interpolant force error {df:.3e} Ha/bohr");
    let (r_i, v_i) = banked.minimum();
    assert!((r_i - r_e).abs() < 1e-5, "interpolant minimum moved {:.3e} bohr", r_i - r_e);
    assert!((v_i - v_min).abs() < 1e-9, "interpolant minimum moved {:.3e} Ha", v_i - v_min);
    // The interpolant's derivative is the derivative OF THE INTERPOLANT, so the force the
    // sampler feels is the exact gradient of the energy it accumulates.
    let h = 1e-5;
    for i in 0..20 {
        let r = 0.7 + 0.35 * i as f64;
        let (_, dv) = banked.eval(r);
        let fd = (banked.eval(r + h).0 - banked.eval(r - h).0) / (2.0 * h);
        assert!((dv - fd).abs() < 1e-6, "at R={r}: analytic {dv:.9} vs finite difference {fd:.9}");
    }
}

// ---------------------------------------------------------------- the classical limit

fn launch() -> ClassicalState {
    let (r_e, _, _) = equilibrium();
    ClassicalState {
        positions: vec![[0.0, 0.0, 0.0], [0.0, 0.0, r_e + 0.35]],
        velocities: vec![[0.0, 0.0, -3.0e-4], [1.0e-4, 0.0, 3.0e-4]],
        masses: vec![Species::HYDROGEN.mass_u, Species::HYDROGEN.mass_u],
    }
}

/// Prereg gate (c): at `P = 1` the ring polymer IS the classical trajectory.
///
/// M-FIXED-POINT-TRAJECTORY: the launch is DISPLACED with nonzero velocity and the path
/// length is asserted, so the square cannot be closed by nothing happening. The mutation
/// control at `P = 2` must FAIL the same comparison, or the gate is blind.
#[test]
fn test_p1_ring_dynamics_is_the_classical_trajectory() {
    let banked = BankedPes::h2(2048);
    let beta = 1.0 / (K_B_HARTREE_PER_KELVIN * 300.0);
    let dt = 4.0;
    let mut cl = launch();
    let mut rp = RingPolymerState {
        beads_pos: vec![launch().positions],
        beads_vel: vec![launch().velocities],
        masses: launch().masses,
    };
    let e0 = classical_energy_3d(&cl, &banked);
    let mut path = 0.0f64;
    let mut worst_pos = 0.0f64;
    let mut worst_vel = 0.0f64;
    let (mut half_a, mut half_b) = (0.0f64, 0.0f64);
    let steps = 2000;
    for st in 0..steps {
        let before = cl.positions[1][2];
        classical_step_3d(&mut cl, dt, &banked);
        ring_step_3d(&mut rp, dt, beta, &banked);
        path += (cl.positions[1][2] - before).abs();
        let de = (classical_energy_3d(&cl, &banked) - e0).abs();
        if st < steps / 2 {
            half_a = half_a.max(de);
        } else {
            half_b = half_b.max(de);
        }
        for i in 0..2 {
            for a in 0..3 {
                worst_pos = worst_pos.max((rp.beads_pos[0][i][a] - cl.positions[i][a]).abs());
                worst_vel = worst_vel.max((rp.beads_vel[0][i][a] - cl.velocities[i][a]).abs());
            }
        }
    }
    assert!(path > 0.1, "the trajectory must MOVE or the gate is vacuous: path {path:.6} bohr");
    // BIT-IDENTICAL, not "close": at P = 1 the ring-polymer step and velocity Verlet are
    // the same arithmetic in the same order, and the free-ring-polymer propagator is
    // written on velocities so no mass round trip can break the identity. It did break it
    // once — 1.05e-11 bohr over 5000 steps — and the fix was to remove the round trip
    // rather than to widen this line.
    assert_eq!(worst_pos, 0.0, "P=1 must be bit-identical to the classical trajectory");
    assert_eq!(worst_vel, 0.0, "P=1 velocities must be bit-identical");
    // Velocity Verlet's energy error is BOUNDED and oscillatory, not secular. Asserting a
    // small end-of-run drift would pass on an integrator whose error happened to be near a
    // zero crossing; comparing the two halves is what tests the shape.
    assert!(half_a < 1e-4, "energy error {half_a:.3e} Ha is larger than the scheme allows");
    assert!(
        half_b < 1.5 * half_a,
        "energy error is growing, so it is secular: {half_a:.3e} then {half_b:.3e}"
    );

    // THE MUTATION CONTROL. A ring with two beads that are actually apart must NOT track
    // the classical trajectory; if it did, this test would pass on an integrator that had
    // silently thrown the ring away.
    let mut rp2 = RingPolymerState {
        beads_pos: vec![launch().positions, launch().positions],
        beads_vel: vec![launch().velocities, launch().velocities],
        masses: launch().masses,
    };
    rp2.beads_pos[0][1][2] += 0.05;
    rp2.beads_pos[1][1][2] -= 0.05;
    let mut cl2 = launch();
    let mut worst2 = 0.0f64;
    for _ in 0..2000 {
        classical_step_3d(&mut cl2, dt, &banked);
        ring_step_3d(&mut rp2, dt, beta, &banked);
        let c = centroid_state(&rp2);
        for i in 0..2 {
            for a in 0..3 {
                worst2 = worst2.max((c.positions[i][a] - cl2.positions[i][a]).abs());
            }
        }
    }
    assert!(
        worst2 > 1e-6,
        "the P=2 control must SEPARATE from the classical trajectory: {worst2:.3e} bohr"
    );
}

/// Prereg gate (d) / G5: the bead-forgetting square, with its budget.
///
/// `Object.lean`'s `Closed` at `P = 1`, its negation above — and the mechanism named:
/// the centroid feels the bead-AVERAGED force while the classical chart feels the force
/// AT the centroid, so the defect is carried by `force_gap` and grows with the ring.
#[test]
fn test_bead_forgetting_square_and_its_budget() {
    let (r_e, _, _) = equilibrium();
    let banked = BankedPes::h2(2048);
    let beta = 1.0 / (K_B_HARTREE_PER_KELVIN * 300.0);
    let dt = 4.0;

    // (i) P = 1: the square is EXACT. The chart is Closed.
    let one = RingPolymerState {
        beads_pos: vec![launch().positions],
        beads_vel: vec![launch().velocities],
        masses: launch().masses,
    };
    let b1 = commuting_budget(&one, dt, beta, &banked);
    assert!(b1.defect_pos < 1e-15, "P=1 defect_pos {:.3e}", b1.defect_pos);
    assert!(b1.defect_vel < 1e-15, "P=1 defect_vel {:.3e}", b1.defect_vel);
    assert!(b1.force_gap < 1e-18, "P=1 force_gap {:.3e}", b1.force_gap);

    // (ii) A ring that is actually a ring is NOT closed, and the defect grows with it.
    let mut prev = 0.0f64;
    let mut rgs = Vec::new();
    let mut gaps = Vec::new();
    for &spread in &[0.02f64, 0.04, 0.08, 0.16] {
        let st = spread_ring(8, r_e, spread);
        let b = commuting_budget(&st, dt, beta, &banked);
        assert!(b.defect_pos > 1e-12, "spread {spread}: defect_pos {:.3e}", b.defect_pos);
        assert!(b.defect_pos > prev, "the defect must grow with the ring");
        prev = b.defect_pos;
        rgs.push(b.radius_of_gyration.ln());
        gaps.push(b.force_gap.ln());
    }
    // (iii) The mechanism: force_gap ~ R_g^2, because the leading term of
    // <F(q_k)> - F(q_c) is (1/2) F''(q_c) <(q_k - q_c)^2>.
    let n = rgs.len() as f64;
    let mx = rgs.iter().sum::<f64>() / n;
    let my = gaps.iter().sum::<f64>() / n;
    let num: f64 = rgs.iter().zip(&gaps).map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = rgs.iter().map(|x| (x - mx) * (x - mx)).sum();
    let slope = num / den;
    assert!(
        (1.6..=2.4).contains(&slope),
        "force_gap must scale as R_g^2; fitted exponent {slope:.4}"
    );

    // The dt^2 law of the defect itself.
    let st = spread_ring(8, r_e, 0.08);
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for &d in &[8.0f64, 4.0, 2.0, 1.0] {
        let b = commuting_budget(&st, d, beta, &banked);
        xs.push(d.ln());
        ys.push(b.defect_pos.ln());
    }
    let n = xs.len() as f64;
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let num: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum();
    let slope = num / den;
    assert!(
        (1.7..=2.3).contains(&slope),
        "defect_pos must scale as dt^2; fitted exponent {slope:.4}"
    );
}

/// A ring whose beads are spread along the bond by a stated amount, centred on `r_e`.
fn spread_ring(p: usize, r_e: f64, spread: f64) -> RingPolymerState {
    let mut beads_pos = vec![vec![[0.0f64; 3]; 2]; p];
    let beads_vel = vec![vec![[0.0f64; 3]; 2]; p];
    for k in 0..p {
        let phase = 2.0 * core::f64::consts::PI * (k as f64) / (p as f64);
        beads_pos[k][0] = [0.0, 0.0, 0.0];
        beads_pos[k][1] = [0.0, 0.0, r_e + spread * phase.cos()];
    }
    RingPolymerState {
        beads_pos,
        beads_vel,
        masses: vec![Species::HYDROGEN.mass_u, Species::HYDROGEN.mass_u],
    }
}

/// The normal-mode transform is orthogonal and its frequencies are the free
/// ring-polymer ones. Both are used by every sampled number in this file.
#[test]
fn test_normal_modes_are_orthogonal() {
    let beta = 1.0 / (K_B_HARTREE_PER_KELVIN * 300.0);
    for &p in &[1usize, 2, 3, 8, 16, 64] {
        let nm = NormalModes::new(p, beta);
        for a in 0..p {
            for b in 0..p {
                let s: f64 = (0..p).map(|j| nm.c[j * p + a] * nm.c[j * p + b]).sum();
                let want = if a == b { 1.0 } else { 0.0 };
                assert!((s - want).abs() < 1e-12, "P={p}: C^T C [{a},{b}] = {s}");
            }
        }
        assert!(nm.omega[0] == 0.0, "the centroid mode must be free");
        for k in 1..p {
            let want = 2.0 * (p as f64 / beta) * ((k as f64) * core::f64::consts::PI / p as f64).sin();
            assert!((nm.omega[k] - want).abs() < 1e-15);
        }
    }
}

// ---------------------------------------------------------------- the banked campaign

/// The campaign's spectral references, PINNED — and checked against a reference this test
/// recomputes from scratch on every run, at a cheaper grid, so the pin cannot rot into a
/// number nothing regenerates.
///
/// `engine/output/c1/dvr.log`, box [0.50, 9.00] bohr, n = 601, 6 levels, banked at 4096
/// knots. Both isotopes' references agreed with the ones taken on the EXACT solver to
/// every printed digit, which is G2.
const CAMPAIGN_ZPE_H2: f64 = 0.011288114850;
const CAMPAIGN_ZPE_D2: f64 = 0.008006844017;

#[test]
fn test_campaign_references_regenerate() {
    // Deliberately CHEAPER than the campaign on every axis — 1024 knots against 4096, a
    // 301-point grid against 601, one level against six, a tighter box — so that agreeing
    // to 1e-9 says the pin is a property of the model and not of the campaign's settings.
    let banked = BankedPes::h2(1024);
    let (_, v_min) = banked.minimum();
    for (name, mass, pinned) in [
        ("H2", Species::HYDROGEN.mass_u, CAMPAIGN_ZPE_H2),
        ("D2", MASS_U_DEUTERIUM, CAMPAIGN_ZPE_D2),
    ] {
        let mu = Vib1D::reduced_mass_me(mass, mass);
        let sys = Vib1D { mu, pes: &banked, name: "iso" };
        let (floor, _) = banked_range();
        let r = dvr_reference(&sys, 0.60, 6.50, floor, 301, 1, 1e-8)
            .expect("the campaign's reference must regenerate on a cheaper grid");
        let zpe = r.zpe(v_min);
        assert!(
            (zpe - pinned).abs() < 1e-9,
            "{name}: recomputed ZPE {zpe:.12} vs pinned {pinned:.12} (d {:.3e})",
            zpe - pinned
        );
        assert_eq!(banked.excursions(), 0, "the reference left the table's domain");
    }
}

/// THE ISOTOPE SHIFT, and the direction the frozen prereg got backwards.
///
/// `C1_GATE_PREREG.md` G4 staked that `ZPE(D2)/ZPE(H2)` sits BELOW the harmonic ratio
/// `sqrt(mu_H2/mu_D2)`. It sits ABOVE, and one line says why: `omega_e ~ mu^-1/2` while
/// `omega_e x_e ~ mu^-1`, so the FRACTIONAL anharmonic deficit `omega_e x_e / 2 omega_e`
/// scales as `mu^-1/2` and is therefore SMALLER for the heavier isotope. D2 sits closer to
/// its own harmonic value than H2 does to its, so the ratio is pushed up.
///
/// This test asserts the corrected statement AND the scaling that forces it, so the
/// derivation is checked and not just its conclusion. The fired clause stays in the
/// prereg, marked dead.
#[test]
fn test_isotope_shift_direction_and_its_scaling() {
    let mu_h = mu_h2();
    let mu_d = Vib1D::reduced_mass_me(MASS_U_DEUTERIUM, MASS_U_DEUTERIUM);
    let harmonic_ratio = (mu_h / mu_d).sqrt();
    let measured_ratio = CAMPAIGN_ZPE_D2 / CAMPAIGN_ZPE_H2;
    assert!(
        measured_ratio > harmonic_ratio,
        "the anharmonic ZPE ratio must sit ABOVE the harmonic one: {measured_ratio:.7} vs {harmonic_ratio:.7}"
    );
    // And by the amount the scaling predicts. The fractional deficits are
    //   1 - ZPE / (omega_harm / 2)
    // for each isotope, and their ratio must be sqrt(mu_H2 / mu_D2) — the SAME factor as
    // the harmonic ratio itself, which is what makes this a check and not a restatement.
    let w_h = omega_harm();
    let w_d = w_h * harmonic_ratio;
    let def_h = 1.0 - CAMPAIGN_ZPE_H2 / (0.5 * w_h);
    let def_d = 1.0 - CAMPAIGN_ZPE_D2 / (0.5 * w_d);
    assert!(def_h > 0.0 && def_d > 0.0, "both deficits must be positive: {def_h} {def_d}");
    let scaling = def_d / def_h;
    assert!(
        (scaling / harmonic_ratio - 1.0).abs() < 0.02,
        "fractional-deficit ratio {scaling:.6} should be sqrt(mu_H2/mu_D2) = {harmonic_ratio:.6}"
    );
}
