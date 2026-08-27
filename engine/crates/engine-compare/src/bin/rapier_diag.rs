//! Is Rapier being configured badly? Decompose its per-step cost before reporting any
//! ratio against it. A misconfigured competitor is the easiest way to manufacture a win.
#[path = "../scene.rs"] mod scene;
use rapier3d_f64::prelude::*;
use scene::Scene;
use std::time::Instant;

fn build(s: &Scene, joints: bool, iters: usize, pgs: usize) -> (PhysicsWorld, Vec<RigidBodyHandle>) {
    let mut w = PhysicsWorld::new();
    w.gravity = Vec3::new(0.0, 0.0, 0.0);
    w.integration_parameters.dt = 1e-2;
    w.integration_parameters.num_solver_iterations = iters;
    w.integration_parameters.num_internal_pgs_iterations = pgs;
    let b: Vec<_> = (0..s.n).map(|i| {
        let p = s.pos0[i];
        w.insert_body(RigidBodyBuilder::dynamic()
            .translation(Vec3::new(p[0], p[1], p[2]))
            .additional_mass(1.0).lock_rotations().can_sleep(false))
    }).collect();
    if joints {
        for &(i, j, k) in &s.edges {
            w.insert_impulse_joint(b[i], b[j],
                SpringJointBuilder::new(0.0, k, 0.0).spring_model(MotorModel::ForceBased));
        }
    }
    (w, b)
}

fn time(s: &Scene, joints: bool, iters: usize, pgs: usize, steps: usize) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..5 {
        let (mut w, _b) = build(s, joints, iters, pgs);
        let t0 = Instant::now();
        for _ in 0..steps { w.step(); }
        best = best.min(t0.elapsed().as_secs_f64() * 1e9 / steps as f64);
    }
    best
}

fn main() {
    println!("Rapier per-step cost decomposition (dt = 1e-2, best of 5)\n");
    println!("{:<22} {:>6} {:>7} {:>12} {:>12} {:>12} {:>12}",
        "scene", "N", "edges", "no joints", "1 iter", "4 iters", "16 iters");
    for s in [Scene::k11(), Scene::complete(64), Scene::lattice(4), Scene::lattice(8)] {
        let steps = (2_000_000 / s.n.max(1)).clamp(200, 20000);
        println!("{:<22} {:>6} {:>7} {:>12.1} {:>12.1} {:>12.1} {:>12.1}",
            s.name, s.n, s.edges.len(),
            time(&s, false, 4, 1, steps),
            time(&s, true, 1, 1, steps),
            time(&s, true, 4, 1, steps),
            time(&s, true, 16, 1, steps));
    }

    println!("\nIs `num_solver_iterations` substepping? If so cost should be ~linear in it.");
    let s = Scene::complete(64);
    for it in [1usize, 2, 4, 8, 16, 32] {
        println!("  iters={:<4} {:>10.1} ns/step", it, time(&s, true, it, 1, 2000));
    }

    println!("\nDefault IntegrationParameters (what a normal Rapier user gets):");
    let d = IntegrationParameters::default();
    println!("  num_solver_iterations           = {}", d.num_solver_iterations);
    println!("  num_internal_pgs_iterations     = {}", d.num_internal_pgs_iterations);
    println!("  num_internal_stabilization_iterations = {}", d.num_internal_stabilization_iterations);
    println!("  warmstart_joints                = {}", d.warmstart_joints);
    println!("  dt                              = {}", d.dt);
}
