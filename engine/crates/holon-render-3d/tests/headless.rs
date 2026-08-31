//! The gates, run through the shell rather than beside it.
//!
//! `holon-render`'s own suite asserts these properties on a bare `Sim`. This file
//! asserts them on the thing the app actually runs: an `AtomWorld` in a Bevy `App`,
//! advanced by a Bevy system, in the 3D scene the 3D shell opens with. The two are
//! different claims. The first says the physics is right; the second says the shell did
//! not break it — that no scheduling decision, no resource wrapper and no scene
//! construction of this crate's own perturbed a conserved quantity.
//!
//! It runs under `--no-default-features --features headless`, which links NO rendering:
//! no bevy_render, no wgpu, no window, nothing that needs a GPU or a display. A gate is
//! a property of the physics, so the test that asserts it must be runnable where there
//! are no pixels.
//!
//! The wall interval is PINNED rather than read from the real clock. `plan_frame` turns
//! wall time into a substep count, so a test driven by the real clock would take a
//! different number of steps on every run and on every machine, and a gate that passes
//! on one step count and not another would be indistinguishable from a flake.

use bevy::app::{App, Update};
use bevy::ecs::system::ResMut;
use bevy::MinimalPlugins;
use holon_render::sim::{Boundary, Dims, MAX_ATOMS};
use holon_render_3d::world::{AtomWorld, BOX_SIDE};

/// The pinned wall interval, seconds. 60 Hz exactly — a plausible frame, held fixed.
const FIXED_DT: f64 = 1.0 / 60.0;

/// The system under test: exactly what `render.rs`'s `advance_world` does, with the
/// clock pinned.
fn advance(mut world: ResMut<AtomWorld>) {
    world.advance(FIXED_DT);
}

/// A headless app holding `world`, advanced by the system above.
fn app_with(world: AtomWorld) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(world)
        .add_systems(Update, advance);
    app
}

fn run(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

fn world(app: &App) -> &AtomWorld {
    app.world().resource::<AtomWorld>()
}

fn world_mut(app: &mut App) -> &mut AtomWorld {
    app.world_mut().resource_mut::<AtomWorld>().into_inner()
}

/// The box's wall inset — where the wall potential starts, measured from the face.
fn w_inset(app: &App) -> f64 {
    world(app).sim.wall_inset
}

// ------------------------------------------------------------------ the scene

#[test]
fn the_shell_solves_its_own_curve() {
    // The forces come from H2 in the STO-3G basis, solved by full CI at startup, not
    // from a file and not from a fit. If this fails the whole app is drawing something
    // whose forces came from nowhere, and every test below would be measuring that.
    let w = AtomWorld::new(2);
    assert!(
        w.table_ok(),
        "the curve did not load: status {}",
        w.table_status
    );
    assert_eq!(w.sim.table().knots(), 492);
    // The STO-3G/FCI equilibrium, which is NOT the experimental 1.401 bohr — a minimal
    // basis is a real model with a real error, and the number asserted here is the
    // model's own, so a change in the basis shows up as a failure rather than passing
    // by being close to experiment.
    assert!(
        (1.3..1.5).contains(&w.sim.table().r_e),
        "R_e = {} bohr is not near the STO-3G minimum",
        w.sim.table().r_e
    );
    assert!(w.sim.table().d_e > 0.0, "the well has no depth");
    // Every clock is derived from that curve.
    assert!(w.sim.timescale.omega_e > 0.0, "omega_e was never derived");
    let t = &w.sim.timescale;
    assert!(
        (t.dt_reference - t.period / 64.0).abs() < 1e-12,
        "the reference timestep is not period/64"
    );
    // dt in force is the reference OR a refinement of it. Under the exactness hold, a
    // curvature envelope stiffer than the well bottom HALVES dt until the accuracy
    // target is met again — that is the hold doing its job, not a degradation, and it is
    // why this is an inequality rather than an equality.
    println!(
        "dt = {:.5e} against a reference of {:.5e} (refined by {:.0}x); omega_env*dt = {:.5}",
        t.dt,
        t.dt_reference,
        t.dt_reference / t.dt,
        t.omega_dt()
    );
    assert!(
        t.dt > 0.0 && t.dt <= t.dt_reference,
        "dt exceeds its reference"
    );
    assert!(
        t.omega_dt() <= t.accuracy_target() + 1e-12,
        "the exactness hold did not meet its own accuracy target: omega_env*dt = {}",
        t.omega_dt()
    );
}

#[test]
fn the_scene_is_a_three_dimensional_box() {
    let mut w = AtomWorld::new(2);
    assert_eq!(w.sim.dims, Dims::Three);
    assert_eq!(w.sim.width, BOX_SIDE);
    assert_eq!(w.sim.height, BOX_SIDE);
    assert_eq!(w.sim.depth, BOX_SIDE);
    // The opening pair is the headline scene and lies along one axis; more atoms open on
    // a sphere, which is the claim that the scene is genuinely three-dimensional.
    w.reset(MAX_ATOMS);
    let z: Vec<f64> = (0..w.sim.n).map(|i| w.sim.atoms[i].z).collect();
    let spread =
        z.iter().cloned().fold(f64::MIN, f64::max) - z.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        spread > 1.0,
        "the {}-atom opening scene is flat in z (spread {spread})",
        w.sim.n
    );
    // And every atom is inside the box it is drawn in.
    for i in 0..w.sim.n {
        let a = &w.sim.atoms[i];
        for (name, v) in [("x", a.x), ("y", a.y), ("z", a.z)] {
            assert!(
                v > 0.0 && v < BOX_SIDE,
                "atom {i} opens outside the box in {name}: {v}"
            );
        }
    }
}

// ------------------------------------------------------------------ the gates

#[test]
fn both_gates_hold_through_a_headless_run() {
    let mut w = AtomWorld::new(2);
    // Walls off: pure NVE, so the external ledger is not merely small but exactly empty,
    // and the momentum gate has nothing to subtract.
    w.sim.boundary = Boundary::Open;
    // A scene that COLLIDES. The opening drift is watchable rather than quick, and a
    // gate asked about two atoms 10 bohr apart that never reach each other is asked
    // nothing — it reads a drift of 1e-14 and passes on a scene with no dynamics in it.
    // The collision is where the integrator is under load, so that is what is gated.
    let c = 0.5 * BOX_SIDE;
    w.sim.set_position_3d(0, c - 4.0, c - 0.2, c + 0.1);
    w.sim.set_position_3d(1, c + 4.0, c + 0.2, c - 0.1);
    w.sim.set_velocity_3d(0, 0.002, 0.0004, -0.0002);
    w.sim.set_velocity_3d(1, -0.002, 0.0004, -0.0002);
    w.sim.rebase();
    let mut app = app_with(w);
    run(&mut app, 400);

    let w = world(&app);
    let s = &w.sim;
    println!(
        "headless 400 frames: {} steps, t = {:.1} a.u.\n  energy: drift {:.3e} / bound \
         {:.3e} = {:.4}\n  momentum: residual {:.3e} / bound {:.3e} = {:.4}",
        s.steps,
        s.time,
        s.drift_peak,
        s.drift_bound(),
        s.drift_peak / s.drift_bound(),
        s.momentum_residual_peak,
        s.momentum_bound(),
        s.momentum_residual_peak / s.momentum_bound(),
    );
    // The run has to have actually run, and the honest measure of that is the physics'
    // own clock rather than a step count: the gate has to have been asked about at least
    // one complete vibration, which is the motion the whole scene is about.
    assert!(
        s.time > s.timescale.period,
        "the run covered {:.1} a.u., less than one vibration ({:.1})",
        s.time,
        s.timescale.period
    );
    assert_eq!(s.w_ext, 0.0, "nothing touched the scene but W_ext moved");
    assert_eq!(s.e_wall, 0.0, "walls are off but carry energy");
    // The gate must have had something to measure. A drift of exactly zero would mean
    // the trajectory never loaded the integrator, and a bound cleared by a scene with no
    // dynamics in it is not evidence about the integrator.
    assert!(
        s.drift_peak > 0.0,
        "the energy gate passed on a run with no measurable drift at all"
    );
    assert!(
        s.energy_gate(),
        "ENERGY GATE FAILED through the shell: drift {:.3e} exceeds bound {:.3e}",
        s.drift_peak,
        s.drift_bound()
    );
    assert!(
        s.momentum_gate(),
        "MOMENTUM GATE FAILED through the shell: residual {:.3e} exceeds bound {:.3e}",
        s.momentum_residual_peak,
        s.momentum_bound()
    );
}

#[test]
fn the_gates_hold_with_the_walls_on() {
    // The box, actually hit. The wall is a conservative term in the energy ledger and an
    // EXTERNAL force in the momentum one, so a bounce asks the two gates two different
    // questions: energy must stay closed across it, and the impulse it delivers must
    // appear in `J_ext` rather than as a momentum residual.
    //
    // The scene is scripted to reach the faces. The default arrangements do not — a
    // shell of atoms at rest collapses inward under its own attraction and never touches
    // anything, so a test that merely turned the walls on would pass with `E_wall`
    // exactly zero and would be evidence about nothing.
    // TWO atoms, already far apart and diverging. An earlier version of this scene put
    // four atoms 2 bohr apart and fired them outward; they bound into a cluster instead
    // and drifted as one, reaching 7.6 bohr from the centre against a wall at 11.4 — the
    // assertion below caught it, which is the reason the assertion is there. Starting
    // them apart and moving apart makes the trajectory ballistic and the test about
    // walls rather than about whether four atoms happen to bind.
    //
    // One aimed at a -x face and one at the +z face: the z faces are the only genuinely
    // new force term the 3D lift added, so the scene has to reach one.
    let mut w = AtomWorld::new(2);
    let c = 0.5 * BOX_SIDE;
    let v = 0.006;
    w.sim.set_position_3d(0, c - 6.0, c, c);
    w.sim.set_position_3d(1, c, c, c + 6.0);
    w.sim.set_velocity_3d(0, -v, 0.0, 0.0);
    w.sim.set_velocity_3d(1, 0.0, 0.0, v);
    w.sim.rebase();
    let mut app = app_with(w);

    let mut max_wall: f64 = 0.0;
    let mut max_impulse: f64 = 0.0;
    let mut max_z = f64::NEG_INFINITY;
    for _ in 0..2_000 {
        app.update();
        let s = &world(&app).sim;
        max_wall = max_wall.max(s.e_wall);
        let j = s.j_ext;
        max_impulse = max_impulse.max((j.0 * j.0 + j.1 * j.1 + j.2 * j.2).sqrt());
        for i in 0..s.n {
            max_z = max_z.max(s.atoms[i].z);
        }
    }

    let s = &world(&app).sim;
    println!(
        "walls on, N = {}: peak E_wall = {max_wall:.3e}, peak |J_ext| = {max_impulse:.3e}, \
         drift {:.3e} / {:.3e}, momentum {:.3e} / {:.3e}",
        s.n,
        s.drift_peak,
        s.drift_bound(),
        s.momentum_residual_peak,
        s.momentum_bound(),
    );
    // The scene did what the test says it did.
    assert!(
        max_wall > 0.0,
        "no atom ever reached a wall: this tested nothing"
    );
    assert!(
        max_impulse > 0.0,
        "the walls pushed but delivered no external impulse"
    );
    // Specifically the +z face — the pair the lift added. An x-face bounce alone would
    // pass the two assertions above while testing nothing the 2D box did not already do.
    assert!(
        max_z > BOX_SIDE - w_inset(&app),
        "the +z face was never reached (max z = {max_z:.3} in a box of depth {BOX_SIDE})"
    );
    assert!(
        s.energy_gate(),
        "energy gate failed across a wall bounce: drift {:.3e} exceeds bound {:.3e}",
        s.drift_peak,
        s.drift_bound()
    );
    assert!(
        s.momentum_gate(),
        "momentum gate failed across a wall bounce: residual {:.3e} exceeds bound {:.3e}",
        s.momentum_residual_peak,
        s.momentum_bound()
    );
}

#[test]
fn an_isolated_pair_in_this_shell_still_cannot_bond() {
    // The claim the whole app exists to make, asserted on the shell's own scene: two
    // atoms alone cannot bond however they are pushed, because their relative energy
    // starts at or above the dissociation asymptote and the dynamics conserve it.
    //
    // Walls OFF, because a wall is an external force on one atom and therefore a real
    // third-body channel — with the box on, a bond CAN form, and that is physics rather
    // than a defect. Isolation is what the claim is about.
    let mut w = AtomWorld::new(2);
    w.sim.boundary = Boundary::Open;
    // The opening scene drifts together at 0.0004 bohr per atomic time unit, which is
    // deliberately slow enough to watch and far too slow to reach a collision inside a
    // test. The claim is about the ENCOUNTER, so the approach is scripted to produce one.
    let c = 0.5 * BOX_SIDE;
    w.sim.set_position_3d(0, c - 4.0, c, c);
    w.sim.set_position_3d(1, c + 4.0, c, c);
    w.sim.set_velocity_3d(0, 0.002, 0.0, 0.0);
    w.sim.set_velocity_3d(1, -0.002, 0.0, 0.0);
    w.sim.rebase();
    let mut app = app_with(w);

    let mut closest = f64::INFINITY;
    for _ in 0..1_500 {
        app.update();
        let s = &world(&app).sim;
        closest = closest.min(s.pairs[0].r);
        assert_eq!(
            s.holons.molecule_count(),
            0,
            "a molecule formed in an isolated pair: R = {:.4}, E_rel = {:.4e}",
            s.pairs[0].r,
            s.pairs[0].e_rel
        );
        assert!(
            s.pairs[0].e_rel >= 0.0,
            "the pair went below the asymptote unaided: {:.4e}",
            s.pairs[0].e_rel
        );
    }
    println!("isolated pair: closest approach {closest:.4} bohr, molecules formed 0");
    // Inside the well's outer wall: the pair genuinely collided, so "no bond formed" is
    // a statement about an encounter rather than about two atoms that never met.
    assert!(
        closest < 2.0,
        "the pair never actually collided (min R = {closest})"
    );
    assert_eq!(world(&app).sim.holons.census.formations, 0);
}

#[test]
fn the_drag_path_keeps_the_ledger_closed() {
    // The interaction `pick.rs` drives, exercised without a pointer: grab, move the
    // anchor through a three-dimensional path, release. Every joule the hand moves is
    // posted to W_ext as it moves, so `E - W_ext` must stay inside the same derived
    // bound it obeys when nobody is touching anything.
    let mut w = AtomWorld::new(3);
    w.sim.boundary = Boundary::Open;
    w.sim.rebase();
    let mut app = app_with(w);

    world_mut(&mut app).sim.grab(0);
    for k in 0..500 {
        let theta = k as f64 * 0.02;
        let c = 0.5 * BOX_SIDE;
        {
            let w = world_mut(&mut app);
            // A path with all three components moving, so the drag is genuinely 3D.
            w.sim.move_anchor_3d(
                c + 5.0 * theta.cos(),
                c + 5.0 * theta.sin(),
                c + 3.0 * (0.5 * theta).sin(),
            );
        }
        app.update();
    }
    world_mut(&mut app).sim.release();

    let s = &world(&app).sim;
    println!(
        "drag: W_ext = {:+.6e} Eh, E - W_ext = {:+.6e} (origin {:+.6e}), drift {:.3e} / \
         bound {:.3e}",
        s.w_ext,
        s.ledger(),
        s.l0,
        s.drift_peak,
        s.drift_bound()
    );
    assert!(s.w_ext != 0.0, "the drag did no work: it tested nothing");
    // Released: the energy still stored in the spring left with the hand, so no spring
    // term is left holding energy the ledger would have to explain.
    assert_eq!(
        s.e_spring, 0.0,
        "the spring still holds energy after release"
    );
    assert!(
        s.energy_gate(),
        "the hand broke the ledger: drift {:.3e} exceeds bound {:.3e}",
        s.drift_peak,
        s.drift_bound()
    );
}

// ------------------------------------------------------------------ the shell's own parts

#[test]
fn every_drawn_bond_is_a_bonded_pair_and_every_row_is_drawn() {
    // The picture and the census must agree, because they are read from the same place.
    // `bonds.rs` draws a cylinder for pair `k` exactly when `pairs[k].bonded`; the census
    // creates a row for a pair only after the dwell and the closure gate. So every live
    // row must correspond to a pair that is currently drawn — a row over a pair with no
    // cylinder would be a molecule the picture denies.
    let mut w = AtomWorld::new(MAX_ATOMS);
    w.sim.boundary = Boundary::Open;
    w.sim.rebase();
    let mut app = app_with(w);
    run(&mut app, 300);

    let s = &world(&app).sim;
    let drawn = s.pairs[..s.pair_count].iter().filter(|p| p.bonded).count();
    let rows = s.holons.molecule_count();
    println!("drawn bonds {drawn}, live molecule rows {rows}");
    for (_, row) in s.holons.live_rows() {
        let k = row.pair as usize;
        assert!(
            k < s.pair_count,
            "a row points at a pair that does not exist"
        );
        assert!(
            s.pairs[k].bonded,
            "a live molecule row sits on a pair the picture does not draw as bonded"
        );
    }
    // A row claims two atoms, and an atom belongs to at most one composite, so rows can
    // never outnumber the pairs drawn.
    assert!(rows <= drawn, "{rows} rows over {drawn} drawn bonds");
}

#[test]
fn the_calibration_burst_gives_the_scene_back() {
    // The burst runs a different scene (N = MAX_ATOMS, walls off) and must leave no
    // trace of it. If it did, the first frame after load would be measuring the
    // calibration scene rather than the user's.
    let mut w = AtomWorld::new(3);
    let n_before = w.sim.n;
    let boundary_before = w.sim.boundary;
    let positions: Vec<(f64, f64, f64)> = (0..w.sim.n)
        .map(|i| (w.sim.atoms[i].x, w.sim.atoms[i].y, w.sim.atoms[i].z))
        .collect();

    w.calibration_burst(2_000);

    assert_eq!(w.sim.n, n_before, "the burst changed the atom count");
    assert_eq!(
        w.sim.boundary, boundary_before,
        "the burst changed the boundary"
    );
    for (i, p) in positions.iter().enumerate() {
        let a = &w.sim.atoms[i];
        assert_eq!((a.x, a.y, a.z), *p, "atom {i} did not come back");
    }
    // And it left the ledger at its origin, not mid-run.
    assert_eq!(
        w.sim.steps, 0,
        "the restored scene carries the burst's steps"
    );
    assert_eq!(w.sim.w_ext, 0.0);
}

#[test]
fn the_atom_count_is_clamped_to_what_the_device_sustains() {
    let mut w = AtomWorld::new(2);
    // Uncalibrated, the cap is the array bound and nothing else.
    assert_eq!(w.n_max(), MAX_ATOMS as f64);
    w.reset(MAX_ATOMS + 10);
    assert_eq!(w.sim.n, MAX_ATOMS, "the array bound was exceeded");
    w.reset(0);
    assert_eq!(
        w.sim.n, 2,
        "the scene needs at least a pair to be about anything"
    );

    // A device measured as very slow cannot carry MAX_ATOMS, and the clamp says so
    // rather than delivering the shortfall as silent time dilation. 10 substeps/second
    // is chosen far below the crossing point, not near it: `n_max` grows as the square
    // root of throughput, so a device only a little too slow is barely clamped and the
    // test would be measuring the threshold rather than the behaviour.
    w.record_calibration(10.0);
    let cap = w.n_max();
    assert!(
        cap < MAX_ATOMS as f64,
        "a 10 substeps/sec device was not clamped below {MAX_ATOMS} (cap {cap})"
    );
    w.reset(MAX_ATOMS);
    assert!(
        (w.sim.n as f64) <= cap.max(2.0),
        "reset ignored the measured cap: {} atoms against a cap of {cap}",
        w.sim.n
    );
}

#[test]
fn a_clockless_host_is_reported_rather_than_invented() {
    // If the burst cannot be timed, the honest answer is "uncalibrated", which makes
    // `substep_budget` return "as many as asked for". Inventing a throughput would put a
    // fabricated number on the device panel and a fabricated cap on the atom count.
    let mut w = AtomWorld::new(2);
    w.record_calibration(f64::NAN);
    assert_eq!(
        w.calibration,
        holon_render_3d::world::Calibration::Unavailable
    );
    assert!(!w.sim.timescale.calibrated);
    assert_eq!(w.n_max(), MAX_ATOMS as f64);
}


/// STANDING QUESTION 1 — is the thing that passes the thing that RUNS? The MBE3
/// tier existed, was gated, and was verified, while this shell integrated
/// pair-only forces because nothing here called `generate_trimer_table`. This
/// asserts the connection INSIDE the constructor's product, not in a sibling
/// that imports the same module: the world the app actually builds must carry a
/// loaded three-body table, and a compact triple must feel it.
#[test]
fn the_world_the_app_builds_carries_the_three_body_law() {
    let w = AtomWorld::new(3);
    assert!(w.table_ok(), "curve failed to build");
    assert!(
        w.sim.trimer.loaded,
        "the 3D world's Sim has no trimer table: the shell is running pair-only physics"
    );
    // And the term is live, not merely resident: a compact equilateral triple
    // must carry positive three-body energy through the same Sim the app steps.
    let mut s = w.sim;
    let (cx, cy, cz) = (0.5 * s.width, 0.5 * s.height, 0.5 * s.depth);
    let r = 1.4;
    s.set_position_3d(0, cx, cy, cz);
    s.set_position_3d(1, cx + r, cy, cz);
    s.set_position_3d(2, cx + 0.5 * r, cy + r * 0.8660254037844386, cz);
    for i in 0..3 {
        s.set_velocity_3d(i, 0.0, 0.0, 0.0);
    }
    s.step_frame(1);
    assert!(
        s.e_three > 0.1,
        "a compact equilateral trimer must pay the three-body term, read {:.6}",
        s.e_three
    );
}

#[test]
fn species_radii_and_colours_match_palette() {
    use holon_chem::elements::FIRST_ROW;
    for sp in FIRST_ROW.iter() {
        // `homonuclear_radius` is declared for the first row only and returns None past
        // it. FIRST_ROW is exactly the declared set, so an unwrap here is a claim this
        // loop stays inside it — and the message says so rather than panicking blankly.
        let r = sp
            .homonuclear_radius()
            .unwrap_or_else(|| panic!("species {} is in FIRST_ROW but has no declared radius", sp.symbol));
        assert!(
            r > 0.5 && r < 4.0,
            "species {} has unphysical homonuclear radius {:.4}",
            sp.symbol,
            r
        );
        let hex = sp.colour_hex();
        assert!(
            hex.starts_with('#') && hex.len() == 7,
            "species {} has invalid hex colour {}",
            sp.symbol,
            hex
        );
        let (cr, cg, cb) = sp.colour_rgb();
        assert!(
            cr >= 0.0 && cr <= 1.0 && cg >= 0.0 && cg <= 1.0 && cb >= 0.0 && cb <= 1.0,
            "species {} has out-of-range RGB: ({cr}, {cg}, {cb})",
            sp.symbol
        );
    }
}

/// The scene struct stays POINTER-SIZED, so the debug profile can build one.
///
/// `Sim` is 331,656 bytes since the pair bank landed (six potential tables where there
/// was one). `AtomWorld::new_with_preset` builds one, moves it into the struct and
/// returns the struct by value, and the debug profile elides none of those moves — an
/// unboxed field put roughly 2 MB of copies on the stack and
/// `all_presets_load_and_conserve_energy` aborted with a stack overflow. It passed in
/// RELEASE the whole time, which is how it went unnoticed: a suite run in one profile
/// cannot see it.
///
/// This gate is on the SIZE rather than on the overflow, because an overflow aborts the
/// process and takes the rest of the suite with it — a defect that destroys the evidence
/// of its own occurrence. Un-boxing the field fires this instead, with a number.
#[test]
fn the_scene_struct_is_not_carried_on_the_stack() {
    let size = std::mem::size_of::<AtomWorld>();
    let sim = std::mem::size_of::<holon_render::sim::Sim>();
    println!("AtomWorld = {size} bytes, Sim = {sim} bytes (boxed away)");
    assert!(
        sim > 100_000,
        "Sim has shrunk to {sim} bytes; this gate was written when it was 331,656, and \
         its premise should be rechecked rather than the bar quietly passing"
    );
    assert!(
        size < 4_096,
        "AtomWorld is {size} bytes: the Sim is being carried BY VALUE again. In debug \
         that is several megabytes of stack copies through new_with_preset, and the \
         preset test aborts the whole suite with a stack overflow."
    );
}

#[test]
fn all_presets_load_and_conserve_energy() {
    use holon_render_3d::world::Preset;

    // Test representative presets covering homonuclear, heteronuclear, 16-atom quench, and negative controls.
    let test_presets = [Preset::H2, Preset::Quench16, Preset::LiH, Preset::He2];
    for preset in test_presets.iter() {
        println!("Testing preset: {}", preset.name());
        let mut w = AtomWorld::new_with_preset(*preset);
        assert!(
            w.table_ok(),
            "preset {} failed to load potential table: status {}",
            preset.name(),
            w.table_status
        );
        assert_eq!(w.preset, *preset);
        let mut app = app_with(w);
        run(&mut app, 100);

        let w = world(&app);
        let s = &w.sim;
        assert!(
            s.energy_gate(),
            "preset {} failed energy gate: drift {:.3e} > bound {:.3e}",
            preset.name(),
            s.drift_peak,
            s.drift_bound()
        );
        assert!(
            s.momentum_gate(),
            "preset {} failed momentum gate: residual {:.3e} > bound {:.3e}",
            preset.name(),
            s.momentum_residual_peak,
            s.momentum_bound()
        );
    }
}

#[test]
fn closed_shell_negative_controls_refuse_to_bind() {
    use holon_render_3d::world::Preset;

    for preset in [Preset::He2, Preset::Ne2] {
        let mut w = AtomWorld::new_with_preset(preset);
        w.sim.boundary = Boundary::Open;
        let mut app = app_with(w);
        run(&mut app, 300);

        let s = &world(&app).sim;
        assert_eq!(
            s.holons.molecule_count(),
            0,
            "closed-shell control {} bound into a molecule",
            preset.name()
        );
        assert_eq!(
            s.bonded_count(),
            0,
            "closed-shell control {} has bonded pairs",
            preset.name()
        );
    }
}
