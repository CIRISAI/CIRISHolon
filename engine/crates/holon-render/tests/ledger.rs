//! Headless gates. Native, not wasm: the physics core is an rlib, so the browser build
//! and this test run the same code with no shim between them.
//!
//! One gate per conservation law, never combined. A single "is the simulation OK"
//! assertion can be green while energy is right and momentum is 5x wrong, so energy and
//! momentum are asserted separately, against separately derived bounds, and the
//! measured-over-bound ratio is printed so a passing gate still shows its margin.

use holon_render::sim::{Boundary, Sim, K_SPRING, M_H};
use holon_render::table::PotentialTable;

fn potential_source() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/viewer/h2_potential.json");
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("cannot read {path}: {e}. Run: cargo run -p holon-render --example make_placeholder")
    })
}

fn loaded_table() -> PotentialTable {
    let mut t = PotentialTable::empty();
    holon_render::json::load_into(&mut t, &potential_source()).expect("table loads");
    t
}

fn loaded_sim() -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(s.table_mut(), &potential_source()).expect("table loads");
    // Every clock is derived from the curve, so adopting the table is what sets dt.
    s.adopt_table_timescale();
    s
}

/// Run `frames` grain boundaries of `substeps` each.
///
/// Tests are written in FRAMES rather than substeps because the frame is the closure
/// point: it is where the gates are evaluated and where the composite layer runs, so a
/// test counting substeps would be counting something the architecture does not schedule
/// on. dt itself is derived and can be refined by the curvature envelope, so a fixed
/// substep count is not a fixed duration either.
fn run(s: &mut Sim, frames: usize, substeps: u32) {
    for _ in 0..frames {
        s.step_frame(substeps);
    }
}

// ------------------------------------------------------------------ the table

#[test]
fn table_load_and_sign_convention() {
    let t = loaded_table();
    assert!(t.is_loaded(), "status {:?}", t.status);
    assert!(t.knots() >= 50, "knots = {}", t.knots());
    // The assumed convention (dE/dR = -F) must be the one that fits. If these two ever
    // swap, the supplied file means the other thing and the whole curve is mirrored.
    println!(
        "residual (dE/dR=-F) = {:.3e}   residual_alt (dE/dR=+F) = {:.3e}",
        t.residual, t.residual_alt
    );
    // The absolute threshold is loose ON PURPOSE, and the looseness is measured rather
    // than guessed. The statistic compares each interval's secant with the mean of its
    // endpoint derivatives, and those differ by -(h^2/12) times the third derivative
    // even for a perfectly consistent table: it has a truncation FLOOR set by the grid,
    // not by the data. Measured on uniform Morse grids (examples/diagnose.rs), the
    // residual falls as exactly h^2 -- 2.27e-2, 5.66e-3, 1.41e-3, 3.53e-4, 8.84e-5 for
    // h = 0.2, 0.1, 0.05, 0.025, 0.0125 -- so 1.4e-3 on the shipped 0.05-bohr grid is
    // the grid's own resolution and nothing else. 0.05 clears that floor by 35x while
    // still catching a few-percent systematic inconsistency in a supplied file.
    assert!(
        t.residual < 5e-2,
        "supplied derivatives disagree with supplied values well beyond the grid's own \
         truncation floor: {:.3e}",
        t.residual
    );
    // The discriminating statistic. Negating the derivatives turns (secant - mean_d)
    // into (secant + mean_d) ~ 2*secant, so `residual_alt` sits at ~2.0 for ANY
    // consistent table; the gap between the two is what identifies the convention.
    assert!(
        t.residual_alt > 20.0 * t.residual,
        "the two sign hypotheses are not distinguishable ({:.3e} vs {:.3e}): the file \
         cannot certify its own convention",
        t.residual,
        t.residual_alt
    );
}

#[test]
fn hermite_reproduces_its_knots_and_is_c1_across_them() {
    let t = loaded_table();
    // Hermite interpolation is DEFINED by reproducing value and derivative at both ends
    // of every interval. Checking that directly is the C1 statement: each interval is a
    // polynomial, and neighbouring intervals agree on value and slope where they meet.
    let mut worst_v: f64 = 0.0;
    let mut worst_d: f64 = 0.0;
    for i in 0..t.knots() {
        let (v, d, _) = t.eval(t.knot_r(i));
        worst_v = worst_v.max((v - t.knot_u(i)).abs());
        worst_d = worst_d.max((d - t.knot_d(i)).abs());
    }
    println!("at knots: max |U - U_knot| = {worst_v:.3e}   max |dU - dU_knot| = {worst_d:.3e}");
    assert!(
        worst_v < 1e-15,
        "interpolant misses its own knot values: {worst_v:.3e}"
    );
    assert!(
        worst_d < 1e-14,
        "interpolant misses its own knot slopes: {worst_d:.3e}"
    );

    // And approaching each knot from the LEFT (the previous interval's polynomial) lands
    // on the same value and slope, which is the half a one-sided evaluation cannot see.
    let eps = 1e-9;
    let mut worst_lv: f64 = 0.0;
    let mut worst_ld: f64 = 0.0;
    for i in 1..t.knots() {
        let r = t.knot_r(i);
        let (v, d, _) = t.eval(r - eps);
        worst_lv = worst_lv.max((v - t.knot_u(i)).abs() - eps * d.abs());
        worst_ld = worst_ld.max((d - t.knot_d(i)).abs());
    }
    println!("from the left: excess value gap = {worst_lv:.3e}   slope gap = {worst_ld:.3e}");
    assert!(
        worst_lv < 1e-12,
        "value is discontinuous at a knot: {worst_lv:.3e}"
    );
    assert!(
        worst_ld < 1e-6,
        "slope is discontinuous at a knot: {worst_ld:.3e}"
    );
    // NOT asserted, and worth being explicit about: the second derivative DOES jump at
    // the knots. "Piecewise cubic Hermite" is C1, not C2, and the forces only need C1.
}

#[test]
fn force_is_exactly_minus_the_gradient_of_the_summed_energy() {
    // THE precondition for the ledger to close. The integrator sums `u` and pushes with
    // `force`; if those two are not derivative-related the energy gate is measuring an
    // inconsistency, not an integration error. Checked across the wall, the well and
    // both extrapolated regions.
    let t = loaded_table();
    let h = 1e-6;
    let mut worst: f64 = 0.0;
    let mut worst_r = 0.0;
    // Offset off the knot lattice (knots sit on multiples of 0.05, 0.10 and 0.25). A
    // central difference STRADDLING a knot measures the second-derivative jump that
    // C1-not-C2 interpolation is entitled to have -- 9.1e-7 at the R = 0.40 seam,
    // measured -- which is a property of the stencil, not of the force.
    let mut r = 0.253;
    while r < 15.0 {
        let numeric = -(t.u(r + h) - t.u(r - h)) / (2.0 * h);
        let analytic = t.force(r);
        // Absolute floor in the denominator: the force passes through zero at R_e, where
        // a pure relative error is meaningless however exact the agreement.
        let rel = (numeric - analytic).abs() / (analytic.abs() + 1e-3);
        if rel > worst {
            worst = rel;
            worst_r = r;
        }
        r += 0.01;
    }
    println!("worst |F + dU/dR| / (|F| + 1e-3) = {worst:.3e} at R = {worst_r:.3} bohr");
    assert!(
        worst < 1e-6,
        "force is not the gradient: {worst:.3e} at R={worst_r}"
    );
}

#[test]
fn placeholder_reproduces_its_own_constants() {
    // Gauge the ruler before trusting a reading off it: the curve must put its minimum
    // where it says R_e is, and put it D_e below the asymptote.
    let t = loaded_table();
    let u_min = t.u(t.r_e);
    println!("U(R_e) = {:.6} Eh   -D_e = {:.6} Eh", u_min, -t.d_e);
    assert!(
        (u_min + t.d_e).abs() < 1e-6,
        "well depth off: {u_min} vs {}",
        -t.d_e
    );
    assert!(t.force(t.r_e).abs() < 1e-6, "R_e is not a stationary point");
    assert!(t.force(0.9) > 0.0, "the wall is not repulsive");
    assert!(t.force(2.5) < 0.0, "the tail is not attractive");
}

// ------------------------------------------------------------------ NVE gates

/// The staked initial condition for the 10k-step NVE run. Frozen here so the run is
/// reproducible: no RNG anywhere in this crate.
fn staked_nve() -> Box<Sim> {
    let mut s = loaded_sim();
    s.boundary = Boundary::Open;
    s.reset(2);
    let cx = 0.5 * s.width;
    let cy = 0.5 * s.height;
    // A bound, vibrating pair at R = 2.2 bohr, with the centre of mass drifting so the
    // momentum gate has something non-zero to conserve.
    s.set_position(0, cx - 1.1, cy);
    s.set_position(1, cx + 1.1, cy);
    s.set_velocity(0, 0.002, 0.001);
    s.set_velocity(1, -0.002, 0.001);
    s.rebase();
    s
}

#[test]
fn nve_energy_gate() {
    let mut s = staked_nve();
    run(&mut s, 156, 64);

    let bound = s.drift_bound();
    let ratio = s.drift_peak / bound;
    println!(
        "NVE 156 frames x 64: |dE|_peak = {:.6e} Eh   bound = {:.6e} Eh   ratio = {:.4}",
        s.drift_peak, bound, ratio
    );
    println!(
        "  E_kin = {:.9} E_pair = {:.9} E_wall = {:.9} W_ext = {:.9}",
        s.e_kin, s.e_pair, s.e_wall, s.w_ext
    );
    // Pure NVE: nothing may have entered the ledger from outside, exactly zero.
    assert_eq!(s.w_ext, 0.0, "NVE run injected external work");
    assert_eq!(s.e_wall, 0.0, "walls are off but carry energy");
    assert!(
        s.energy_gate(),
        "drift {:.3e} exceeds bound {:.3e}",
        s.drift_peak,
        bound
    );
}

#[test]
fn nve_momentum_gate() {
    let mut s = staked_nve();
    let p0 = s.momentum();
    run(&mut s, 156, 64);
    let p = s.momentum();
    let bound = s.momentum_bound();
    println!(
        "NVE 156 frames x 64: |dP|_peak = {:.6e}   bound = {:.6e}   ratio = {:.4}",
        s.momentum_residual_peak,
        bound,
        s.momentum_residual_peak / bound
    );
    println!(
        "  P0 = ({:.6e}, {:.6e}, {:.6e})  P = ({:.6e}, {:.6e}, {:.6e})",
        p0.0, p0.1, p0.2, p.0, p.1, p.2
    );
    // Walls off and no spring: the external impulse is not merely small, it is absent.
    // The third component is asserted with the other two: this scene is the mid-plane
    // slice of the 3D box, and a non-zero z impulse would mean the slice had leaked.
    assert_eq!(
        s.j_ext,
        (0.0, 0.0, 0.0),
        "no external force acted but impulse accrued"
    );
    assert!(
        s.momentum_gate(),
        "momentum residual {:.3e} exceeds roundoff bound {:.3e}",
        s.momentum_residual_peak,
        bound
    );
}

#[test]
fn drift_obeys_the_harmonic_law_it_was_derived_from() {
    // The bound in `drift_bound` claims |dE|/E_0 = (omega dt)^2 / 4 exactly for a
    // harmonic oscillator. Test that claim against the actual curve, at an amplitude
    // small enough for the well to be harmonic, so the bound is validated rather than
    // merely believed. (Gauge the ruler with a planted value before staking a band.)
    let mut s = loaded_sim();
    s.boundary = Boundary::Open;
    s.reset(2);
    let cx = 0.5 * s.width;
    let cy = 0.5 * s.height;
    let delta = 0.02_f64;
    let r0 = s.table().r_e + delta;
    s.set_position(0, cx - 0.5 * r0, cy);
    s.set_position(1, cx + 0.5 * r0, cy);
    s.set_velocity(0, 0.0, 0.0);
    s.set_velocity(1, 0.0, 0.0);
    s.rebase();

    let k = s.table().curvature(s.table().r_e);
    let mu = 0.5 * M_H;
    let omega = (k / mu).sqrt();
    // Starting from rest at maximum displacement, so E_0 is the turning-point energy.
    let e0 = 0.5 * k * delta * delta;
    let predicted = e0 * (omega * s.dt()).powi(2) / 4.0;

    run(&mut s, 312, 64);

    let ratio = s.drift_peak / predicted;
    println!(
        "harmonic law: omega = {:.6e} (={:.1} cm^-1), omega*dt = {:.4}",
        omega,
        omega / 4.556335e-6,
        omega * s.dt()
    );
    println!(
        "  measured |dE|_peak = {:.6e} Eh   predicted (omega dt)^2/4 * E_0 = {:.6e} Eh   ratio = {:.4}",
        s.drift_peak, predicted, ratio
    );
    assert!(
        (0.8..1.25).contains(&ratio),
        "the derived law does not describe the measured drift: ratio {ratio:.4}"
    );
}

// ------------------------------------------------------------------ bonds

/// The approach speed shared by the control and the intervention below, in bohr per
/// atomic time unit. Chosen so the pair starts honestly UNBOUND (relative energy above
/// the asymptote) and still reaches the repulsive wall.
const APPROACH_V: f64 = 0.002;

#[test]
fn two_atoms_alone_can_never_bond() {
    // The control, and half the point of the app. Two atoms approaching from outside the
    // well have E_rel >= 0, NVE conserves it, so no matter how hard they are pushed they
    // climb the wall and come back out. If this ever fires, the bond predicate is
    // reading something other than the curve.
    let mut s = loaded_sim();
    s.boundary = Boundary::Open;
    s.reset(2);
    let cx = 0.5 * s.width;
    let cy = 0.5 * s.height;
    s.set_position(0, cx - 4.0, cy);
    s.set_position(1, cx + 4.0, cy);
    s.set_velocity(0, APPROACH_V, 0.0);
    s.set_velocity(1, -APPROACH_V, 0.0);
    s.rebase();

    let mut min_r = f64::INFINITY;
    let mut ever_bonded = false;
    let mut min_e_rel = f64::INFINITY;
    for _ in 0..5_000 {
        s.step_frame(8);
        min_r = min_r.min(s.pairs[0].r);
        min_e_rel = min_e_rel.min(s.pairs[0].e_rel);
        if s.pairs[0].bonded {
            ever_bonded = true;
        }
    }
    println!(
        "head-on approach: closest R = {:.4} bohr, min E_rel = {:.6e} Eh, bonded ever = {ever_bonded}",
        min_r, min_e_rel
    );
    assert!(
        min_r < 1.2,
        "the atoms never actually got close (R_min = {min_r})"
    );
    assert!(
        !ever_bonded,
        "two isolated atoms bonded with no energy removed"
    );
    assert!(
        min_e_rel >= 0.0,
        "relative energy went below the asymptote in pure NVE"
    );
}

#[test]
fn scripted_push_forms_a_bond_and_the_ledger_stays_closed() {
    // The intervention run. IDENTICAL initial condition to the control above, so the
    // only difference between "never bonds" and "bonds" is the user's hand -- which is
    // the claim the app exists to make.
    //
    // The script: at closest approach the user grabs one atom (the anchor lands on it,
    // so the spring enters at zero extension and injects nothing), holds it while the
    // pair tries to fly apart -- the spring loads up at the expense of the atoms'
    // motion -- and RELEASES the instant the pair's relative energy first falls below
    // the dissociation asymptote. Release carries the stored spring energy out of the
    // scene with the hand, and from that moment the pair is genuinely isolated, so its
    // now-negative relative energy is conserved and the bond is permanent.
    //
    // The release condition is the bond criterion itself, not a tuned step count: the
    // script asks the physics when to let go.
    let mut s = loaded_sim();
    s.boundary = Boundary::Open;
    s.reset(2);
    let cx = 0.5 * s.width;
    let cy = 0.5 * s.height;
    s.set_position(0, cx - 4.0, cy);
    s.set_position(1, cx + 4.0, cy);
    s.set_velocity(0, APPROACH_V, 0.0);
    s.set_velocity(1, -APPROACH_V, 0.0);
    s.rebase();
    s.refresh_pairs();
    let e_rel_start = s.pairs[0].e_rel;
    assert!(
        e_rel_start > 0.0,
        "the pair was already bound before the push"
    );

    // 1. approach until the separation stops shrinking
    let mut last_r = f64::INFINITY;
    let mut approach_steps = 0;
    loop {
        s.step_frame(8);
        let r = s.pairs[0].r;
        if r > last_r {
            break;
        }
        last_r = r;
        approach_steps += 1;
        assert!(approach_steps < 200_000, "never reached closest approach");
    }
    let r_closest = last_r;

    // 2. grab: the spring must enter the ledger for free
    let w_before_grab = s.w_ext;
    s.grab(0);
    assert_eq!(s.w_ext, w_before_grab, "the grab itself injected work");
    assert_eq!(
        s.e_spring, 0.0,
        "the spring did not enter at zero extension"
    );

    // 3. hold until the pair is bound, then let go
    const MAX_HOLD: usize = 20_000;
    let mut hold_steps = 0;
    while s.pairs[0].e_rel >= 0.0 {
        s.step_frame(8);
        hold_steps += 1;
        assert!(
            hold_steps < MAX_HOLD,
            "held for {MAX_HOLD} steps and the pair never became bound"
        );
    }
    let stored = s.e_spring;
    let e_rel_at_release = s.pairs[0].e_rel;
    s.release();
    s.refresh_pairs();
    let removed = -s.w_ext;

    println!(
        "E_rel(start) = {e_rel_start:.4e} Eh; closest approach R = {r_closest:.4} bohr after \
         {approach_steps} steps; held {hold_steps} steps; spring stored {stored:.6} Eh; \
         E_rel at release = {e_rel_at_release:.4e} Eh; removed from scene {removed:.6} Eh"
    );

    // 4. run long enough that a transient cannot masquerade as a bond
    let mut bonded_all_the_way = true;
    let mut max_r_after: f64 = 0.0;
    // Instantaneous overshoot: how far outside its OWN turning point, evaluated at the
    // same instant, the pair ever gets. Comparing the trajectory maximum against a
    // turning point computed at some other instant would compare two different states.
    let mut worst_overshoot: f64 = f64::NEG_INFINITY;
    for _ in 0..5_000 {
        s.step_frame(8);
        if !s.pairs[0].bonded {
            bonded_all_the_way = false;
        }
        max_r_after = max_r_after.max(s.pairs[0].r);
        worst_overshoot = worst_overshoot.max(s.pairs[0].r - s.pairs[0].r_outer);
    }

    println!(
        "after 5k more frames: E_rel = {:.6e} Eh, R = {:.4} bohr (max {:.4}), R_outer = {:.4} bohr, bonded = {}",
        s.pairs[0].e_rel, s.pairs[0].r, max_r_after, s.pairs[0].r_outer, s.pairs[0].bonded
    );
    println!(
        "ledger: E = {:.9} W_ext = {:.9} L = {:.9} L0 = {:.9} drift_peak = {:.3e} bound = {:.3e} ratio = {:.4}",
        s.energy(),
        s.w_ext,
        s.ledger(),
        s.l0,
        s.drift_peak,
        s.drift_bound(),
        s.drift_peak / s.drift_bound()
    );

    assert!(
        removed > 0.0,
        "no energy was removed, so nothing could have bound"
    );
    assert!(s.pairs[0].bonded, "the bond predicate did not fire");
    assert!(
        s.pairs[0].e_rel < 0.0,
        "E_rel = {:.3e} is not below the asymptote",
        s.pairs[0].e_rel
    );
    assert!(
        bonded_all_the_way,
        "the bond flickered: it is a transient, not a bond"
    );
    assert_eq!(s.bonded_count(), 1);
    // The documented redundancy, measured. `refresh_pairs` claims that for an isolated
    // pair the turning-point condition is IMPLIED by the energy condition, because any
    // state the pair occupies has U_eff(R) <= E_rel with the radial kinetic energy as
    // the slack. If that is right, the pair can never be found outside its own
    // instantaneous turning point, and the overshoot is bounded by the turning-point
    // solve's own resolution rather than by anything physical.
    println!("worst instantaneous overshoot beyond R_outer = {worst_overshoot:.3e} bohr");
    assert!(
        worst_overshoot <= 1e-9,
        "the bonded pair was found outside its own turning point by {worst_overshoot:.3e} bohr"
    );
    // The gate that makes the whole thing honest: the user's intervention is ON the
    // ledger, so the energy gate closes across a grab, a hold and a release.
    assert!(
        s.energy_gate(),
        "drift {:.3e} exceeds bound {:.3e} across the intervention",
        s.drift_peak,
        s.drift_bound()
    );
    // And the momentum ledger holds across it too: the spring's impulse is accounted,
    // not excused.
    assert!(
        s.momentum_gate(),
        "momentum residual {:.3e} exceeds bound {:.3e}",
        s.momentum_residual_peak,
        s.momentum_bound()
    );
}

#[test]
fn dragging_the_anchor_keeps_the_ledger_closed() {
    // The interactive path proper: the anchor MOVES, which is the only way the user's
    // hand can do work. Every move posts its own dU to W_ext, so E - W_ext must not
    // budge even while the pointer is dragging an atom across the box.
    let mut s = loaded_sim();
    s.boundary = Boundary::Walls;
    s.reset(3);
    s.rebase();
    s.grab(0);
    for k in 0..2_000 {
        // A scripted drag. The anchor moves ONCE per frame and is then held constant
        // across that frame's substeps -- the zero-order hold that `move_anchor`
        // documents as the interaction model.
        let theta = k as f64 * 0.008;
        s.move_anchor(
            0.5 * s.width + 6.0 * theta.cos(),
            0.5 * s.height + 4.0 * theta.sin(),
        );
        s.step_frame(8);
    }
    s.release();
    s.refresh_pairs();
    println!(
        "drag: W_ext = {:.6} Eh, E = {:.6} Eh, drift_peak = {:.3e}, bound = {:.3e}, ratio = {:.4}",
        s.w_ext,
        s.energy(),
        s.drift_peak,
        s.drift_bound(),
        s.drift_peak / s.drift_bound()
    );
    assert!(s.w_ext != 0.0, "a drag that did no work is not a drag");
    assert!(
        s.energy_gate(),
        "drift {:.3e} exceeds bound {:.3e} during a drag",
        s.drift_peak,
        s.drift_bound()
    );
    assert!(s.momentum_gate(), "momentum ledger broke during a drag");
}

#[test]
fn the_thermostat_is_on_the_ledger_too() {
    // A thermostat is an energy source. If it were left off the ledger the energy gate
    // would fail the moment anyone switched it on, and the temptation would be to widen
    // the gate. It is posted to W_ext instead.
    let mut s = loaded_sim();
    s.boundary = Boundary::Walls;
    s.reset(6);
    s.set_velocity(0, 0.003, 0.001);
    s.rebase();
    s.thermostat_on = true;
    s.target_temperature = 600.0;
    run(&mut s, 312, 64);
    println!(
        "thermostat: T = {:.1} K, W_ext = {:.6e} Eh, drift_peak = {:.3e}, bound = {:.3e}",
        s.temperature(),
        s.w_ext,
        s.drift_peak,
        s.drift_bound()
    );
    assert!(s.w_ext != 0.0, "the thermostat moved no energy");
    assert!(s.energy_gate(), "the thermostat broke the energy ledger");
    assert!(
        s.momentum_gate(),
        "the thermostat broke the momentum ledger"
    );
}

#[test]
fn spring_stiffness_is_the_declared_stage_value() {
    // Guards against the stage constants drifting away from what the comments and the
    // viewer's readouts claim they are.
    assert_eq!(K_SPRING, 0.05);
    assert_eq!(M_H, 1837.152);
}
