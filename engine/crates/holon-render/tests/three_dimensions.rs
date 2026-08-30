//! The conservation gates, re-run in three dimensions, plus the two claims the 3D lift
//! rests on.
//!
//! `tests/ledger.rs` and `tests/amendments.rs` state these gates on the mid-plane. This
//! file is deliberately NOT a second implementation of them: it re-runs the same staked
//! scenes with the pair axis tilted out of plane, because the whole argument of the lift
//! is that nothing dimension-dependent survives in the physics — the curve, the force
//! law, the bond predicate, the turning point and the drift bound are all functions of
//! the scalar separation, so a gate that holds in the plane must hold out of it for the
//! same reasons, and a gate that does not has found a real defect.
//!
//! Two claims are load-bearing and are checked directly rather than inferred:
//!
//! * **The mid-plane is exactly invariant.** A 2D scene's `z` and `vz` must stay
//!   bit-identical through walls, collisions and the user's spring. This is what makes
//!   the canvas shell's numbers, and the 40 existing gate tests, still the same numbers.
//! * **The lift is rotationally covariant.** The same scene rotated into a generic plane
//!   must produce the same scalars. This is the check that catches a dimension-dependent
//!   bug the mid-plane test cannot see, because on the mid-plane the new terms are all
//!   exact zeros and a wrong formula multiplied by zero still reads correct.

use holon_render::sim::{Boundary, Dims, Sim, K_B, M_H};

fn potential_source() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/viewer/h2_potential.json");
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("cannot read {path}: {e}. Run: cargo run -p holon-render --example make_placeholder")
    })
}

fn loaded_sim() -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(s.table_mut(), &potential_source()).expect("table loads");
    s.adopt_table_timescale();
    s
}

fn run(s: &mut Sim, frames: usize, substeps: u32) {
    for _ in 0..frames {
        s.step_frame(substeps);
    }
}

/// A rotation that maps the mid-plane onto a generic one: `Rz(1.3) * Rx(0.7)`. Neither
/// angle is a multiple of a right angle, so no component is accidentally preserved and
/// every one of the lifted terms carries a non-zero value.
fn rotate(v: (f64, f64, f64)) -> (f64, f64, f64) {
    let (a, b) = (0.7f64, 1.3f64);
    let (x, y, z) = v;
    // Rx(a)
    let (y, z) = (y * a.cos() - z * a.sin(), y * a.sin() + z * a.cos());
    // Rz(b)
    let (x, y) = (x * b.cos() - y * b.sin(), x * b.sin() + y * b.cos());
    (x, y, z)
}

/// The `staked_nve` scene of `tests/ledger.rs`, tilted out of plane: the same bound
/// vibrating pair at R = 2.2 bohr with the same drifting centre of mass, rotated so that
/// every coordinate, every velocity component and the pair's angular momentum vector all
/// carry all three components.
fn staked_nve_3d() -> Box<Sim> {
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    s.boundary = Boundary::Open;
    s.reset(2);
    let c = (0.5 * s.width, 0.5 * s.height, 0.5 * s.depth);
    // The flat scene's velocities decompose as a centre-of-mass drift of (0, 0.001, 0)
    // plus a relative half of (0.002, 0, 0); each part is rotated as the vector it is.
    let half = rotate((1.1, 0.0, 0.0));
    let v_rel = rotate((0.002, 0.0, 0.0));
    let v_com = rotate((0.0, 0.001, 0.0));
    s.set_position_3d(0, c.0 - half.0, c.1 - half.1, c.2 - half.2);
    s.set_position_3d(1, c.0 + half.0, c.1 + half.1, c.2 + half.2);
    s.set_velocity_3d(0, v_rel.0 + v_com.0, v_rel.1 + v_com.1, v_rel.2 + v_com.2);
    s.set_velocity_3d(1, -v_rel.0 + v_com.0, -v_rel.1 + v_com.1, -v_rel.2 + v_com.2);
    s.rebase();
    s
}

// ------------------------------------------------------------------ the two claims

#[test]
fn the_midplane_is_exactly_invariant() {
    // Claim 1. A 2D scene may not leave the plane — not approximately, exactly. Asserted
    // on the BITS, because "z stayed within 1e-15 of the plane" is a different and much
    // weaker statement, and it is the weaker one that would let the canvas shell's
    // numbers drift out from under it.
    //
    // The scene is chosen to exercise every force that could push in z if any of them
    // had been lifted wrongly: walls on (so the box's x and y faces engage), a close
    // encounter (the repulsive core), and the user's spring dragged around a circle.
    let mut s = loaded_sim();
    s.boundary = Boundary::Walls;
    s.reset(3);
    let cz = 0.5 * s.depth;
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    s.set_position(0, cx - 1.4, cy);
    s.set_position(1, cx + 1.4, cy);
    s.set_position(2, cx, cy + 3.0);
    s.set_velocity(0, 0.004, 0.0);
    s.set_velocity(1, -0.004, 0.0);
    s.set_velocity(2, 0.0, -0.004);
    s.rebase();

    s.grab(2);
    // Radius 12 against a half-height of 12: the anchor circle passes OUTSIDE the y
    // faces, so the spring drags the held atom into the wall and the wall term is
    // actually exercised rather than merely enabled.
    let mut max_wall: f64 = 0.0;
    for k in 0..600 {
        let theta = k as f64 * 0.01;
        s.move_anchor(cx + 12.0 * theta.cos(), cy + 12.0 * theta.sin());
        s.step_frame(8);
        max_wall = max_wall.max(s.e_wall);
        for i in 0..s.n {
            assert_eq!(
                s.atoms[i].z.to_bits(),
                cz.to_bits(),
                "atom {i} left the mid-plane at frame {k}: z = {}",
                s.atoms[i].z
            );
            assert_eq!(
                s.atoms[i].vz.to_bits(),
                0.0f64.to_bits(),
                "atom {i} acquired vz = {} at frame {k}",
                s.atoms[i].vz
            );
        }
    }
    // The z impulse and the z momentum are exactly absent, not merely small.
    assert_eq!(s.j_ext.2, 0.0, "z impulse accrued in a 2D scene");
    assert_eq!(s.momentum().2, 0.0, "z momentum accrued in a 2D scene");
    // And the run was not trivial: the wall and the spring both did work. Without these
    // two the test would pass on a scene where nothing ever pushed, which would say
    // nothing about whether a push can leave the plane.
    assert!(s.w_ext != 0.0, "the spring never did any work");
    assert!(max_wall > 0.0, "the walls never engaged: this tested nothing");
    println!(
        "mid-plane held for 600 frames with walls, a collision and a dragged spring; \
         W_ext = {:.6e} Eh, peak E_wall = {max_wall:.6e} Eh",
        s.w_ext
    );
}

#[test]
fn the_lift_is_rotationally_covariant() {
    // Claim 2. Every scalar the gates read is a rotation invariant, so the same scene in
    // a generic plane must produce the same numbers. This is the test that can SEE a
    // wrong z term: on the mid-plane the new terms are exact zeros, and a wrong formula
    // times zero still reads as correct.
    //
    // Walls off, because a box is not rotation invariant and rotating INTO one would be
    // comparing two different scenes. What is compared is the isolated pair, where the
    // only terms are kinetic and pair energy and both are invariant by construction.
    let mut flat = loaded_sim();
    flat.boundary = Boundary::Open;
    flat.reset(2);
    let cf = (0.5 * flat.width, 0.5 * flat.height, 0.5 * flat.depth);
    flat.set_position_3d(0, cf.0 - 1.1, cf.1, cf.2);
    flat.set_position_3d(1, cf.0 + 1.1, cf.1, cf.2);
    flat.set_velocity_3d(0, 0.002, 0.001, 0.0);
    flat.set_velocity_3d(1, -0.002, 0.001, 0.0);
    flat.rebase();

    let mut tilted = staked_nve_3d();

    let mut worst_r: f64 = 0.0;
    let mut worst_e: f64 = 0.0;
    for _ in 0..200 {
        flat.step_frame(64);
        tilted.step_frame(64);
        let (a, b) = (flat.pairs[0], tilted.pairs[0]);
        worst_r = worst_r.max((a.r - b.r).abs() / a.r.abs().max(1e-30));
        worst_e = worst_e.max((a.e_rel - b.e_rel).abs() / a.e_rel.abs().max(1e-30));
        assert_eq!(a.bonded, b.bonded, "the bond predicate is frame-dependent");
    }

    let d_kin = (flat.e_kin - tilted.e_kin).abs() / flat.e_kin.abs();
    let d_pair = (flat.e_pair - tilted.e_pair).abs() / flat.e_pair.abs();
    let pf = flat.momentum();
    let pt = tilted.momentum();
    let mag = |p: (f64, f64, f64)| (p.0 * p.0 + p.1 * p.1 + p.2 * p.2).sqrt();
    let d_p = (mag(pf) - mag(pt)).abs() / mag(pf);
    println!(
        "covariance over 200 frames x 64: dR/R = {worst_r:.3e}  dE_rel/E_rel = \
         {worst_e:.3e}  dE_kin = {d_kin:.3e}  dE_pair = {d_pair:.3e}  d|P|/|P| = {d_p:.3e}"
    );

    // MEASURED, then staked with margin, rather than guessed. The rotation itself puts a
    // relative perturbation of a few ulp into the initial condition; 12,800 steps of a
    // vibrating pair amplify that by the trajectory's own sensitivity. Measured worst
    // case on this scene: dR/R = 3.6e-11, dE_rel/E_rel = 5.1e-13, dE_kin = 1.0e-10,
    // dE_pair = 3.2e-11, d|P|/|P| = 4.4e-15. 1e-8 clears the largest of those by 100x
    // and sits far below anything a wrong z term could produce — the cheapest such bug,
    // a centrifugal term built from L_z instead of |L|, moves `r_outer` by percent, and
    // an earlier revision of this test's own scene construction (rotating the wrong
    // velocity split) showed up here as dR/R = 0.86.
    const TOL: f64 = 1e-8;
    assert!(worst_r < TOL, "separation is not rotation-invariant: {worst_r:.3e}");
    assert!(worst_e < TOL, "pair energy is not rotation-invariant: {worst_e:.3e}");
    assert!(d_kin < TOL, "kinetic energy is not rotation-invariant: {d_kin:.3e}");
    assert!(d_pair < TOL, "pair potential is not rotation-invariant: {d_pair:.3e}");
    assert!(d_p < TOL, "momentum magnitude is not rotation-invariant: {d_p:.3e}");
}

// ------------------------------------------------------------------ the gates, in 3D

#[test]
fn nve_energy_gate_in_three_dimensions() {
    let mut s = staked_nve_3d();
    run(&mut s, 156, 64);

    let bound = s.drift_bound();
    let ratio = s.drift_peak / bound;
    println!(
        "3D NVE 156 frames x 64: |dE|_peak = {:.6e} Eh   bound = {:.6e} Eh   ratio = {ratio:.4}",
        s.drift_peak, bound
    );
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
fn nve_momentum_gate_in_three_dimensions() {
    let mut s = staked_nve_3d();
    let p0 = s.momentum();
    run(&mut s, 156, 64);
    let p = s.momentum();
    let bound = s.momentum_bound();
    println!(
        "3D NVE momentum: |dP|_peak = {:.6e}  bound = {:.6e}  ratio = {:.4}",
        s.momentum_residual_peak,
        bound,
        s.momentum_residual_peak / bound
    );
    println!(
        "  P0 = ({:.6e}, {:.6e}, {:.6e})  P = ({:.6e}, {:.6e}, {:.6e})",
        p0.0, p0.1, p0.2, p.0, p.1, p.2
    );
    // Every component is genuinely in play — this is the point of tilting the scene.
    assert!(p0.2.abs() > 0.0, "the staked 3D scene has no z momentum to conserve");
    // Walls off and no spring: the external impulse is absent in all three components.
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
fn capture_plant_in_three_dimensions() {
    // FENCE 2, out of plane. The argument is dimension-free — an isolated two-body
    // system with W_ext = 0 conserves its pair energy, so there is no channel to carry
    // the surplus away — but the ensemble is re-run in 3D because the IMPLEMENTATION is
    // not dimension-free: the impact parameter now feeds an angular momentum with three
    // components, and it is the |L|^2 that enters the turning-point solve.
    let mut worst_e_rel = f64::INFINITY;
    let mut closest = f64::INFINITY;
    let mut runs = 0;
    for speed_step in 0..4 {
        for impact_step in 0..4 {
            let v = 0.0015 + 0.0015 * speed_step as f64;
            let b = 0.4 * impact_step as f64;
            let mut s = loaded_sim();
            s.dims = Dims::Three;
            s.boundary = Boundary::Open;
            s.reset(2);
            let c = (0.5 * s.width, 0.5 * s.height, 0.5 * s.depth);
            // Approach along a tilted axis with the impact parameter taken along a
            // second, perpendicular tilted direction, so the encounter plane is generic.
            let axis = rotate((5.0, 0.0, 0.0));
            let off = rotate((0.0, 0.5 * b, 0.0));
            let vel = rotate((v, 0.0, 0.0));
            s.set_position_3d(0, c.0 - axis.0 - off.0, c.1 - axis.1 - off.1, c.2 - axis.2 - off.2);
            s.set_position_3d(1, c.0 + axis.0 + off.0, c.1 + axis.1 + off.1, c.2 + axis.2 + off.2);
            s.set_velocity_3d(0, vel.0, vel.1, vel.2);
            s.set_velocity_3d(1, -vel.0, -vel.1, -vel.2);
            s.rebase();
            runs += 1;

            for _ in 0..2_000 {
                s.step_frame(8);
                closest = closest.min(s.pairs[0].r);
                worst_e_rel = worst_e_rel.min(s.pairs[0].e_rel);
                assert_eq!(s.w_ext, 0.0, "the plant is not isolated: W_ext moved");
                assert_eq!(
                    s.holons.molecule_count(),
                    0,
                    "CAPTURE PLANT VIOLATED in 3D: a molecule formed at v = {v}, b = {b}, \
                     R = {:.4}, E_rel = {:.4e}",
                    s.pairs[0].r,
                    s.pairs[0].e_rel
                );
            }
            assert_eq!(
                s.holons.census.formations, 0,
                "a row was created and destroyed"
            );
        }
    }
    println!(
        "3D capture plant: {runs} isolated encounters, closest approach {closest:.4} bohr, \
         lowest E_rel {worst_e_rel:.4e} Eh, molecules formed: 0"
    );
    assert!(
        closest < 1.2,
        "the ensemble never actually collided (min R = {closest})"
    );
    assert!(
        worst_e_rel >= 0.0,
        "an isolated pair went below the asymptote"
    );
}

#[test]
fn the_box_has_a_lid() {
    // The 2D box's four sides became six faces. The two new ones are the only genuinely
    // new force term in the lift, so they get their own test: an atom fired at the +z
    // face must be turned around, the wall must charge for it, and the ledger must stay
    // closed through the bounce.
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    s.boundary = Boundary::Walls;
    s.reset(2);
    let c = (0.5 * s.width, 0.5 * s.height, 0.5 * s.depth);
    // Far apart, so the pair force is negligible and this is a clean wall test.
    s.set_position_3d(0, c.0 - 8.0, c.1, c.2);
    s.set_position_3d(1, c.0 + 8.0, c.1, c.2 + 6.0);
    s.set_velocity_3d(0, 0.0, 0.0, -0.004);
    s.set_velocity_3d(1, 0.0, 0.0, 0.004);
    s.rebase();

    let mut max_wall: f64 = 0.0;
    let mut min_z = f64::INFINITY;
    let mut max_z = f64::NEG_INFINITY;
    for _ in 0..3_000 {
        s.step_frame(16);
        max_wall = max_wall.max(s.e_wall);
        for i in 0..s.n {
            min_z = min_z.min(s.atoms[i].z);
            max_z = max_z.max(s.atoms[i].z);
        }
    }
    println!(
        "lid test: z visited [{min_z:.3}, {max_z:.3}] in a box of depth {:.1} (inset {:.2}); \
         peak E_wall = {max_wall:.6e} Eh; drift {:.3e} of bound {:.3e}",
        s.depth,
        s.wall_inset,
        s.drift_peak,
        s.drift_bound()
    );
    assert!(max_wall > 0.0, "the z faces never engaged: this tested nothing");
    // Confinement: the soft wall is finite, so the test is that the atoms are turned
    // around well inside the box rather than that they never pass the inset.
    assert!(min_z > 0.0 && max_z < s.depth, "an atom escaped the box in z");
    assert!(min_z < s.wall_inset + 1.0, "the -z face was never approached");
    assert!(
        max_z > s.depth - s.wall_inset - 1.0,
        "the +z face was never approached"
    );
    assert!(
        s.energy_gate(),
        "drift {:.3e} exceeds bound {:.3e} through a z-wall bounce",
        s.drift_peak,
        s.drift_bound()
    );
}

#[test]
fn temperature_counts_the_scene_degrees_of_freedom() {
    // The one reading whose FORMULA is dimension-dependent. Equipartition puts
    // E_kin = (dof/2) N k T, so the same kinetic energy is a lower temperature in 3D:
    // exactly 2/3 of the 2D reading, because the same energy is spread over one more
    // degree of freedom per atom.
    let mut s = loaded_sim();
    s.boundary = Boundary::Open;
    s.reset(2);
    s.set_velocity_3d(0, 0.003, 0.002, 0.001);
    s.set_velocity_3d(1, -0.001, 0.002, -0.003);
    s.rebase();

    let t2 = s.temperature();
    s.dims = Dims::Three;
    let t3 = s.temperature();
    println!("same E_kin = {:.6e} Eh reads {t2:.2} K in 2D and {t3:.2} K in 3D", s.e_kin);

    // Against the closed form, not against each other alone.
    let expect2 = s.e_kin / (s.n as f64 * K_B);
    let expect3 = 2.0 * s.e_kin / (3.0 * s.n as f64 * K_B);
    assert_eq!(t2, expect2, "the 2D reading changed");
    assert!((t3 - expect3).abs() <= 1e-9 * expect3, "the 3D reading is wrong");
    assert!(
        (t3 / t2 - 2.0 / 3.0).abs() < 1e-12,
        "the ratio is not the dof ratio: {}",
        t3 / t2
    );
    // And the mass constant is the atom's, in both.
    assert!((M_H - 1837.152).abs() < 1e-9);
}
