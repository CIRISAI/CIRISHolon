//! THE UNIFORM FIELD (FSD-W1 WB-2.4), and the four things it has to be true of.
//!
//! WB-2.4 asks for 1 G downward at every scale and calls it "the workbench's cleanest
//! tier-separation exhibit": one field, correctly invisible at 1 nm, sovereign over a
//! kilometre of water. The exhibit is only worth anything if the invisibility is a
//! MEASUREMENT rather than an adjective, so `gravity_is_invisible_at_the_atomic_scale`
//! computes the ratio and checks the FSD's own staked figure against it — and the staked
//! figure is where this file's most interesting result is.
//!
//! The other three are the conservation obligations. Gravity enters `a_ext` beside the
//! walls and the hand, which means the momentum ledger already books its impulse and the
//! energy ledger already holds its potential; these tests are what make that "already"
//! checkable instead of asserted.

use holon_render::sim::{Boundary, Dims, GravityRefusal, Sim, G_EARTH_AU, G_SI, K_B};

/// A walled, gravity-free reference scene with a real curve loaded.
fn scene(n: usize) -> Sim {
    let mut s = Sim::empty();
    holon_render::generate_table(&mut s, 0.6, 12.0, 192);
    s.dims = Dims::Three;
    s.boundary = Boundary::Walls;
    s.reset(n);
    s
}

/// The ledger closes under a field strong enough to move the scene.
///
/// 1 G is far too weak to show up in an f64 ledger over a few hundred femtoseconds — that
/// is the exhibit, not a problem — so the CONSERVATION test runs at a field that actually
/// does work. The physics is linear in `g` and the accounting has no threshold in it, so a
/// test at 1e18 G is a test of the same code path with a signal the gate can see. Running
/// only at 1 G would be a gate that passes because nothing happened.
#[test]
fn energy_gate_closes_under_a_field_that_does_real_work() {
    let mut s = scene(12);
    let e_before = s.energy();
    s.set_gravity(1e18 * G_EARTH_AU).expect("walls accept a field");
    // A new potential term moves the total, so the origin has to move with it.
    s.rebase();
    assert!(
        s.e_grav.abs() > 0.0,
        "a field this strong must hold real potential energy, got {}",
        s.e_grav
    );
    assert!(
        (s.energy() - e_before).abs() > 1e-12,
        "the field must change the total energy, or it is not in the sum"
    );

    for _ in 0..400 {
        s.step_frame(64);
    }

    assert!(
        s.energy_gate(),
        "energy gate open under gravity: drift {:.3e} against bound {:.3e}",
        s.drift(),
        s.drift_bound()
    );
    assert!(
        s.momentum_gate(),
        "momentum gate open under gravity: residual {:.3e} against bound {:.3e}",
        s.momentum_residual(),
        s.momentum_bound()
    );
}

/// A conservative field posts NOTHING to the work receipts.
///
/// The obligation runs the other way from the hand's. The hand is a moving boundary
/// condition and its work IS a receipt; a uniform field is conservative, its energy lives
/// in `e_grav`, and a `work.gravity` column would count the same joules twice — once in
/// the potential and once in the receipt — which the balance gate would then report as a
/// drift equal to the double count.
#[test]
fn gravity_posts_nothing_to_the_work_columns() {
    let mut s = scene(12);
    s.set_gravity(1e18 * G_EARTH_AU).unwrap();
    s.rebase();
    let w0 = s.w_ext;
    let hand0 = s.work.hand;
    let thermo0 = s.work.thermostat;
    let baro0 = s.work.barostat;

    for _ in 0..200 {
        s.step_frame(64);
    }

    assert_eq!(s.w_ext, w0, "gravity moved w_ext; it is conservative and must not");
    assert_eq!(s.work.hand, hand0);
    assert_eq!(s.work.thermostat, thermo0);
    assert_eq!(s.work.barostat, baro0);
    assert!(s.work_columns_ok(), "the receipt columns must still sum to w_ext");
}

/// A periodic box refuses the field, and the refusal is the whole point.
///
/// `m g y` on a torus is discontinuous at the wrap: an atom leaving the top face re-enters
/// at the bottom with its potential changed by `m g H` and nothing having done that work.
/// Serving it would open the balance gate by exactly that jump on every crossing and
/// report the result as integration drift.
#[test]
fn a_periodic_box_refuses_the_field() {
    let mut s = scene(12);
    s.boundary = Boundary::Periodic;
    assert_eq!(
        s.set_gravity(G_EARTH_AU),
        Err(GravityRefusal::PeriodicBox),
        "a periodic box has no bottom and must refuse"
    );
    assert_eq!(s.gravity_vec(), (0.0, 0.0, 0.0), "a refused field must not be stored");

    // Zero is not a field, so it is always accepted: turning gravity OFF on a periodic box
    // must not be an error, or a viewer switching boundary with the control at zero would
    // get a refusal for asking nothing.
    assert!(s.set_gravity(0.0).is_ok());

    // And the walled box accepts it, so the refusal is about the chart rather than a
    // blanket ban. A test that only showed the refusal could not tell those apart.
    s.boundary = Boundary::Walls;
    assert!(s.set_gravity(G_EARTH_AU).is_ok());
    assert_eq!(s.gravity_vec(), (0.0, -G_EARTH_AU, 0.0));
}

/// THE TIER-SEPARATION EXHIBIT, measured — and it does not say what WB-2.4 says.
///
/// FSD-W1 WB-2.4 offers gravity as "one field, silent at the bottom, sovereign at the
/// top": ~1e-13 of kT at 1 nm, the thing that sags droplets at 1 mm, and the whole
/// hydrostatic column at 1 km. Computing the three numbers instead of quoting them turns
/// up two corrections, one arithmetic and one conceptual:
///
///   1. THE 1 nm FIGURE IS 25x OFF. Measured 4.05e-15 of kT for a hydrogen atom raised
///      1 nm, against the staked ~1e-13. The claim's SUBSTANCE survives — the point is
///      that gravity is invisible there, and it is more invisible than advertised — but
///      the exponent is wrong by more than a rounding.
///   2. "SOVEREIGN AT 1 km" IS NOT A PER-PARTICLE STATEMENT, and reading it as one is
///      wrong by five orders of magnitude. A hydrogen atom raised a full kilometre gains
///      0.004 kT: still invisible. The per-atom crossover is the SCALE HEIGHT,
///      kT/(m g), which this test measures at ~246 km for atomic hydrogen — the textbook
///      scale height of an isothermal hydrogen atmosphere at room temperature, which is
///      what makes it a check on the whole unit chain rather than a number. Gravity's
///      sovereignty at the top is COLLECTIVE: the hydrostatic column is ~9.8 MPa at 1 km,
///      about 97 atmospheres, and that is a sum over ~1e28 particles, not an energy any
///      one of them carries.
///
/// So the exhibit is real and it is BETTER than the FSD's phrasing: what changes across
/// the tiers is not the size of the per-particle term (which stays negligible the whole
/// way) but whether the quantity that matters is a per-particle energy or a sum over the
/// column. That is a cleaner statement of tier separation than "small here, big there",
/// and it is the one the workbench should make.
///
/// These are DOCUMENTATION corrections, so this test asserts the measured values and does
/// not fail on the FSD's prose — a red gate is the wrong instrument for a wrong sentence.
#[test]
fn gravity_is_invisible_at_the_atomic_scale() {
    let m_h = holon_render::sim::M_H;
    let bohr_m = holon_render::sim::BOHR_M;
    let kt_293 = K_B * 293.15;

    let u_at = |metres: f64| m_h * G_EARTH_AU * (metres / bohr_m);

    let ratio_nm = u_at(1e-9) / kt_293;
    let ratio_mm = u_at(1e-3) / kt_293;
    let ratio_km = u_at(1e3) / kt_293;
    // kT / (m g), in bohr, converted to metres: the height at which one atom's
    // gravitational energy reaches kT.
    let scale_height_m = (kt_293 / (m_h * G_EARTH_AU)) * bohr_m;

    println!("  kT at 293.15 K            = {kt_293:.4e} Ha");
    println!("  m_H g h / kT at    1 nm   = {ratio_nm:.4e}   (WB-2.4 stakes ~1e-13)");
    println!("  m_H g h / kT at    1 mm   = {ratio_mm:.4e}");
    println!("  m_H g h / kT at    1 km   = {ratio_km:.4e}");
    println!("  per-atom crossover height = {:.1} km  (kT / m_H g)", scale_height_m / 1e3);

    assert!(
        (1e-15..1e-14).contains(&ratio_nm),
        "the 1 nm ratio moved out of its decade: {ratio_nm:.4e}"
    );
    // The per-atom term is STILL below kT at a kilometre. Asserted because it is the
    // counter-intuitive half and the one a reader is most likely to get backwards.
    assert!(
        ratio_km < 1.0,
        "a single atom raised 1 km should still be under kT, got {ratio_km:.4e}"
    );
    // ~246 km for atomic hydrogen at room temperature. The window is wide because the
    // assertion is about the unit chain being right, not about the third digit.
    assert!(
        (2.0e5..3.0e5).contains(&scale_height_m),
        "the hydrogen scale height should be ~246 km; got {:.1} km — the unit \
         conversion chain is wrong somewhere",
        scale_height_m / 1e3
    );

    // The constant is a unit conversion, not a number that was typed in: round-trip it.
    let round_trip = G_EARTH_AU * bohr_m
        / (holon_render::sim::AU_TIME_S * holon_render::sim::AU_TIME_S);
    assert!(
        (round_trip - G_SI).abs() < 1e-9,
        "G_EARTH_AU does not convert back to 9.80665 m/s^2: {round_trip}"
    );
}

/// A scene with no field is BIT-IDENTICAL to one from before the field existed.
///
/// The standing replay fingerprints and every banked B1 reference were taken without
/// gravity, so `g = 0` has to be exactly the old code path — not nearly.
///
/// A CLAIM THIS TEST WAS WRITTEN TO DEFEND, AND WHICH TURNED OUT TO BE FALSE. The first
/// version of this comment said the `if self.g != 0.0` guard in `compute_forces` was
/// load-bearing for bit-identity, on the grounds that an unconditional
/// `a_ext[i].1 -= m * 0.0` is not the identity for a negative-zero acceleration. Planting
/// exactly that mutation (P-G3) did NOT fail this test, and the mutation is right: in
/// IEEE 754 round-to-nearest, `x - (+0.0)` is `x` for every finite `x`, and `-0.0 - 0.0`
/// is `-0.0`. So the guard is a COST optimisation — a gravity-free scene pays nothing per
/// atom per force pass — and not a correctness one, and the plant surviving is the
/// evidence for that rather than a hole in the gate.
///
/// The test is kept, because what it actually pins is the property the fingerprints
/// depend on: a zero field changes no bit of a run. That is worth a gate whether the
/// mechanism protecting it is a branch or the arithmetic itself.
#[test]
fn a_field_of_zero_changes_no_bit() {
    let mut a = scene(12);
    let mut b = scene(12);
    b.set_gravity(0.0).unwrap();

    for _ in 0..50 {
        a.step_frame(64);
        b.step_frame(64);
    }

    assert_eq!(a.energy().to_bits(), b.energy().to_bits());
    assert_eq!(a.ledger().to_bits(), b.ledger().to_bits());
    for i in 0..a.n {
        assert_eq!(a.atoms[i].x.to_bits(), b.atoms[i].x.to_bits(), "atom {i} x");
        assert_eq!(a.atoms[i].y.to_bits(), b.atoms[i].y.to_bits(), "atom {i} y");
        assert_eq!(a.atoms[i].vy.to_bits(), b.atoms[i].vy.to_bits(), "atom {i} vy");
    }
    assert_eq!(a.e_grav, 0.0);
}

/// The field survives a checkpoint round trip.
///
/// `g` is a SETTING and rides in the checkpoint; `e_grav` is derived and does not. A
/// checkpoint that dropped `g` would restore a scene that looks identical, recomputes a
/// self-consistent ledger around the missing field, and falls differently — with every
/// invariant closing over the wrong physics, which is precisely the failure no gate
/// downstream could see.
#[test]
fn a_checkpoint_carries_the_field() {
    let mut s = scene(12);
    s.set_gravity(1e18 * G_EARTH_AU).unwrap();
    s.rebase();
    for _ in 0..20 {
        s.step_frame(64);
    }
    let saved = s.checkpoint();
    let e_grav_before = s.e_grav;

    let mut t = scene(12);
    t.restore(&saved).expect("restore");

    assert_eq!(t.gravity_vec(), s.gravity_vec(), "the field did not survive the round trip");
    assert_eq!(
        t.e_grav.to_bits(),
        e_grav_before.to_bits(),
        "the restored scene recomputed a different potential"
    );
    assert_eq!(t.energy().to_bits(), s.energy().to_bits());
}

// ---------------------------------------------------------------- WB-2.4c: the vector

/// THE SCALAR DOOR IS THE VECTOR DOOR. Not "agrees to a tolerance" — bit for bit.
///
/// `set_gravity(g)` delegates to `set_gravity_vec(0, -g, 0)`, and a delegation nobody
/// checks is two implementations waiting to drift. Every float in the two scenes is
/// compared by BITS, because a difference of one ulp here would mean the reduction
/// `-m (g . r)` to `+m g y` is not exact, and the whole point of routing both through one
/// force loop is that it is.
#[test]
fn the_scalar_door_and_the_vector_door_are_bit_identical() {
    let mut a = scene(12);
    let mut b = scene(12);
    a.set_gravity(1e18 * G_EARTH_AU).unwrap();
    b.set_gravity_vec(0.0, -1e18 * G_EARTH_AU, 0.0).unwrap();
    a.rebase();
    b.rebase();
    assert_eq!(a.e_grav.to_bits(), b.e_grav.to_bits(), "potential differs at t = 0");

    for _ in 0..200 {
        a.step_frame(64);
        b.step_frame(64);
    }
    assert_eq!(a.energy().to_bits(), b.energy().to_bits());
    assert_eq!(a.e_grav.to_bits(), b.e_grav.to_bits());
    for i in 0..a.n {
        assert_eq!(a.atoms[i].y.to_bits(), b.atoms[i].y.to_bits(), "atom {i} y");
        assert_eq!(a.atoms[i].vy.to_bits(), b.atoms[i].vy.to_bits(), "atom {i} vy");
    }
    assert_eq!(a.gravity().to_bits(), b.gravity().to_bits());
}

/// A TILTED FIELD CONSERVES, and it is not secretly the vertical one.
///
/// The tilted bucket is the whole reason WB-2.4c exists, so this runs the field at 45° in
/// the x-y plane and requires two things at once: the ledger still closes (the term is
/// conservative whatever direction it points), and the scene is measurably DIFFERENT from
/// the same magnitude pointing straight down. The second half is the one that matters —
/// a bug that silently projected the vector back onto -y would pass every conservation
/// check ever written and produce a bucket that cannot be tilted.
#[test]
fn a_tilted_field_conserves_and_actually_tilts() {
    let g = 1e18 * G_EARTH_AU;
    let c = core::f64::consts::FRAC_1_SQRT_2;

    let mut tilted = scene(12);
    tilted.set_gravity_vec(g * c, -g * c, 0.0).unwrap();
    tilted.rebase();

    let mut down = scene(12);
    down.set_gravity_vec(0.0, -g, 0.0).unwrap();
    down.rebase();

    for _ in 0..200 {
        tilted.step_frame(64);
        down.step_frame(64);
    }

    assert!(
        tilted.energy_gate(),
        "a tilted field must conserve: drift {:.3e} vs bound {:.3e}",
        tilted.drift(),
        tilted.drift_bound()
    );
    assert!(tilted.momentum_gate(), "momentum gate open under a tilted field");
    assert_eq!(tilted.w_ext, 0.0, "a tilted field is still conservative");

    // The magnitudes match to float precision, so any difference in the trajectory is
    // DIRECTION and nothing else. RELATIVE, not absolute: `sqrt((g/sqrt2)^2 * 2)` is not
    // bit-exactly `g`, and the first version of this line demanded 1e-30 absolute on a
    // quantity of order 1e-4 — an impossible bar that failed on correct arithmetic. The
    // bar that means something is "the same to within float rounding".
    let rel = (tilted.gravity() - down.gravity()).abs() / down.gravity();
    assert!(
        rel < 1e-15,
        "the two fields are not the same strength (relative {rel:.3e}); this test can only \
         attribute a trajectory difference to direction if the magnitudes agree"
    );
    let moved = (0..tilted.n)
        .map(|i| (tilted.atoms[i].x - down.atoms[i].x).abs())
        .fold(0.0f64, f64::max);
    println!("  max |x_tilted - x_down| = {moved:.4e} bohr   (box width {:.1} bohr)", tilted.width);

    // THE BAR IS DERIVED, AND THE FIRST VERSION OF IT WAS USELESS.
    //
    // This started as `moved > 0.0`, which a plant proved cannot fail: with the vector
    // silently projected back onto -y — the exact defect this test exists to catch — the
    // two scenes still separated by 3.4e-12 bohr, because their field MAGNITUDES differ in
    // the last bits (`sqrt(2 (g/sqrt2)^2)` is not bit-exactly `g`) and two hundred frames
    // of a chaotic trajectory amplify that into something merely nonzero. A threshold at
    // zero cannot tell "the direction matters" from "floats diverged".
    //
    // The bar that can: a field whose x-component is ~7.7e-5 a.u. acting for ~1.4e4 a.u.
    // of simulated time drives atoms across the box and into the +x wall, so the two
    // scenes must differ by at least an INTERATOMIC distance. One bohr is that scale, and
    // it sits ten orders above the float-divergence floor and one order below the measured
    // 21.5 bohr — which is the box width, i.e. exactly the pinned-against-the-wall
    // behaviour the physics predicts.
    assert!(
        moved > 1.0,
        "the tilted field moved atoms only {moved:.3e} bohr in x against the vertical \
         field — below an interatomic distance, so the direction is being discarded and \
         this difference is float divergence rather than physics"
    );
}

/// The field's DIRECTION survives a checkpoint, not merely its strength.
///
/// The v2 format stored one number. A v3 file that stored only the magnitude would restore
/// a tilted bucket standing upright — conserving perfectly, gates all green, and wrong.
#[test]
fn a_checkpoint_carries_the_field_direction() {
    let g = 1e18 * G_EARTH_AU;
    let c = core::f64::consts::FRAC_1_SQRT_2;
    let mut s = scene(12);
    s.set_gravity_vec(g * c, -g * c, 0.0).unwrap();
    s.rebase();
    for _ in 0..20 {
        s.step_frame(64);
    }
    let saved = s.checkpoint();

    let mut t = scene(12);
    t.restore(&saved).expect("restore");
    assert_eq!(t.gravity_vec(), s.gravity_vec(), "the direction did not survive");
    assert_eq!(t.e_grav.to_bits(), s.e_grav.to_bits());
    assert_eq!(t.energy().to_bits(), s.energy().to_bits());
}

/// A WRAPPING BOX REFUSES EVERY NONZERO DIRECTION, not just the vertical one.
///
/// The scalar door could only ever offer `-y`, so its refusal test only ever exercised
/// that axis. With a vector the refusal has three ways to be got wrong, and a check on one
/// component would pass while a sideways field sailed through onto a torus.
#[test]
fn a_wrapping_box_refuses_the_field_in_every_direction() {
    let g = G_EARTH_AU;
    for (gx, gy, gz) in [
        (g, 0.0, 0.0),
        (0.0, -g, 0.0),
        (0.0, 0.0, g),
        (g, -g, g),
    ] {
        let mut s = scene(12);
        s.boundary = Boundary::Periodic;
        assert_eq!(
            s.set_gravity_vec(gx, gy, gz),
            Err(GravityRefusal::PeriodicBox),
            "a wrapping box accepted ({gx}, {gy}, {gz})"
        );
        assert_eq!(s.gravity_vec(), (0.0, 0.0, 0.0), "a refused field was stored");
    }
    // And the zero vector is always fine: turning gravity OFF on a torus is not a request
    // for a field, and refusing it would make a viewer's boundary switch an error.
    let mut s = scene(12);
    s.boundary = Boundary::Periodic;
    assert!(s.set_gravity_vec(0.0, 0.0, 0.0).is_ok());
}

/// THE CROSS-TERM: compressing the box under gravity keeps the ledger closed.
///
/// Neither lane wrote this one, and it is exactly where two correct changes can produce a
/// wrong sum. `scale_box` moves every atom affinely and posts `energy_after -
/// energy_before` to both ledger columns; `e_grav` is part of `energy()`, so the work done
/// against the field during a compression is booked automatically. "Automatically" is a
/// claim, and this is the check on it: a hand that squeezed a box under gravity and did
/// not pay for the lift would show up here and nowhere else.
#[test]
fn compressing_the_box_under_gravity_keeps_the_ledger_closed() {
    let mut s = scene(12);
    s.set_gravity_vec(0.0, -1e18 * G_EARTH_AU, 0.0).unwrap();
    s.rebase();
    for _ in 0..50 {
        s.step_frame(64);
    }

    let e_grav_before = s.e_grav;
    let w_before = s.w_ext;
    s.scale_box(0.9).expect("a 10% compression is not a collapse");
    assert!(
        (s.e_grav - e_grav_before).abs() > 0.0,
        "an affine compression under gravity must change the gravitational potential"
    );
    assert!(
        (s.w_ext - w_before).abs() > 0.0,
        "the compression's cost was not posted to the ledger"
    );
    assert!(s.work_columns_ok(), "the receipt columns parted from w_ext");

    for _ in 0..200 {
        s.step_frame(64);
    }
    assert!(
        s.energy_gate(),
        "energy gate open after a compression under gravity: drift {:.3e} vs bound {:.3e}",
        s.drift(),
        s.drift_bound()
    );
    assert!(s.momentum_gate());
}
