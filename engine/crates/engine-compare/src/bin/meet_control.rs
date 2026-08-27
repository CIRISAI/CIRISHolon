//! MEET-2 limiting-case CONTROL (PROGRAM.md Path N-c): Rapier 0.35 f64 corroborates
//! the rigid ballistic segment the T5 Newtonian chart licenses for the exported wall.
//!
//! Rapier is the CONTROL, never the reference: the reference for a uniform-g
//! contact-free rigid body is the ballistic closed form x0 + v0 t + g t^2 / 2.
//! Rapier integrates with symplectic Euler, whose discrete drop differs from the
//! closed form by ~g T dt / 2 — an INTEGRATOR budget, never an ulp bar (the
//! two-runtimes lesson). PASS: |rapier - closed| <= g*T*dt and zero lateral drift,
//! on a weightless drift segment, the demo wall's effective g, and the chart g.

use rapier3d_f64::prelude::*;

const DT: f64 = 1.0 / 600.0;
const STEPS: usize = 600;
const START_Y: f64 = 10_000.0;

fn ballistic_segment(g: f64, v0: f64) -> (f64, f64, f64) {
    let mut world = PhysicsWorld::new();
    world.gravity = Vec3::new(0.0, -g, 0.0);
    world.integration_parameters.dt = DT;
    let body = world.insert_body(
        RigidBodyBuilder::dynamic()
            .translation(Vec3::new(0.0, START_Y, 0.0))
            .linvel(Vec3::new(v0, 0.0, 0.0))
            .additional_mass(1.0)
            .can_sleep(false),
    );
    world.insert_collider(ColliderBuilder::ball(0.05).density(0.0), Some(body));
    for _ in 0..STEPS {
        world.step();
    }
    let t = STEPS as f64 * DT;
    let pos = world.bodies[body].translation();
    (START_Y - pos.y, pos.x - v0 * t, t)
}

fn main() {
    let mut failed = false;
    for (label, g, v0) in [
        ("weightless drift", 0.0, 3.0),
        ("demo effective g ", 1.8 * 0.035, 0.0),
        ("chart g          ", 1.8, 5.0),
    ] {
        let (drop, drift, t) = ballistic_segment(g, v0);
        let closed = 0.5 * g * t * t;
        let budget = (g * t * DT).max(1.0e-12);
        let drop_err = (drop - closed).abs();
        let ok = drop_err <= budget && drift.abs() <= 1.0e-9;
        println!(
            "{label}: drop {drop:.9} vs closed {closed:.9} (err {drop_err:.3e}, budget {budget:.3e}), drift {drift:.3e} -> {}",
            if ok { "PASS" } else { "FAIL" }
        );
        failed |= !ok;
    }
    if failed {
        println!("MEET-2 Rapier control: FAIL");
        std::process::exit(1);
    }
    println!("MEET-2 Rapier control: PASS (Rapier corroborates the ballistic limiting case; the closed form is the reference)");
}
