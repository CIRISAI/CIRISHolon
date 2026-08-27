//! PART F — the f32 caveat, measured rather than guessed.
//!
//! The main comparison runs Rapier in f64 (`rapier3d-f64`) because a matched-ACCURACY
//! claim against a f64 engine cannot be made against a f32 competitor: f32 has ~7
//! decimal digits, so it cannot reach the tighter targets at any step size. But f32 is
//! Rapier's default and what a game actually ships, and it enables SIMD paths f64 does
//! not have. So: how much throughput does the f64 requirement cost Rapier?
//!
//! Throughput only. No accuracy claim is made for the f32 build, because there is no
//! honest one to make against an f64 reference.

#[path = "../scene.rs"]
mod scene;
use scene::Scene;
use std::time::Instant;

fn time_f64(s: &Scene, dt: f64, steps: usize, iters: usize) -> f64 {
    use rapier3d_f64::prelude::*;
    let mut best = f64::INFINITY;
    for _ in 0..5 {
        let mut w = PhysicsWorld::new();
        w.gravity = Vec3::new(0.0, 0.0, 0.0);
        w.integration_parameters.dt = dt;
        w.integration_parameters.num_solver_iterations = iters;
        let b: Vec<_> = (0..s.n)
            .map(|i| {
                let p = s.pos0[i];
                w.insert_body(
                    RigidBodyBuilder::dynamic()
                        .translation(Vec3::new(p[0], p[1], p[2]))
                        .additional_mass(1.0)
                        .lock_rotations()
                        .can_sleep(false),
                )
            })
            .collect();
        for &(i, j, k) in &s.edges {
            w.insert_impulse_joint(
                b[i],
                b[j],
                SpringJointBuilder::new(0.0, k, 0.0).spring_model(MotorModel::ForceBased),
            );
        }
        let t0 = Instant::now();
        for _ in 0..steps {
            w.step();
        }
        best = best.min(t0.elapsed().as_secs_f64() * 1e9 / steps as f64);
    }
    best
}

fn time_f32(s: &Scene, dt: f32, steps: usize, iters: usize) -> f64 {
    use rapier3d::prelude::*;
    let mut best = f64::INFINITY;
    for _ in 0..5 {
        let mut w = PhysicsWorld::new();
        w.gravity = Vec3::new(0.0, 0.0, 0.0);
        w.integration_parameters.dt = dt;
        w.integration_parameters.num_solver_iterations = iters;
        let b: Vec<_> = (0..s.n)
            .map(|i| {
                let p = s.pos0[i];
                w.insert_body(
                    RigidBodyBuilder::dynamic()
                        .translation(Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32))
                        .additional_mass(1.0)
                        .lock_rotations()
                        .can_sleep(false),
                )
            })
            .collect();
        for &(i, j, k) in &s.edges {
            w.insert_impulse_joint(
                b[i],
                b[j],
                SpringJointBuilder::new(0.0, k as f32, 0.0).spring_model(MotorModel::ForceBased),
            );
        }
        let t0 = Instant::now();
        for _ in 0..steps {
            w.step();
        }
        best = best.min(t0.elapsed().as_secs_f64() * 1e9 / steps as f64);
    }
    best
}

fn main() {
    println!("PART F — what the f64 requirement costs Rapier (throughput only)");
    println!("Same scene, same dt, same 4 solver iterations. Best of 5.\n");
    println!("{:<24} {:>6} {:>8} {:>14} {:>14} {:>10}", "scene", "N", "edges", "rapier f64", "rapier f32", "f32 gain");
    println!("{}", "-".repeat(80));
    for s in [
        Scene::k11(),
        Scene::complete(64),
        Scene::complete(256),
        Scene::lattice(4),
        Scene::lattice(8),
    ] {
        let steps = (2_000_000 / (s.n * s.n).max(1)).clamp(30, 3000);
        let dt = 1e-2;
        let a = time_f64(&s, dt, steps, 4);
        let b = time_f32(&s, dt as f32, steps, 4);
        println!(
            "{:<24} {:>6} {:>8} {:>11.1} ns {:>11.1} ns {:>9.2}x",
            s.name, s.n, s.edges.len(), a, b, a / b
        );
    }
    iteration_invariance();
    println!("\nEvery Rapier figure in the main comparison is the f64 column. Divide the");
    println!("reported ratios by the f32 gain to get what a shipping f32 Rapier would show");
    println!("on THROUGHPUT — but note it could not have met the matched-accuracy targets");
    println!("at all, which is why the main table is f64.");
}


/// Does the main comparison's choice of `num_solver_iterations = 4` disadvantage Rapier?
///
/// In Rapier 0.35 that parameter is SUBSTEPPING: cost per step is linear in it, and the
/// error is set by the substep size dt/iters. So cost ~ iters/dt and error ~ dt/iters,
/// and accuracy-per-unit-compute should be INVARIANT under the choice. If that holds,
/// picking 4 (Rapier's own default) neither helps nor hurts it, and the equal-compute
/// comparison in PART D is robust to the knob. Measured, not assumed.
fn iteration_invariance() {
    use ciris_sim_core::structure::{Structure, NO_TWINS};
    use rapier3d_f64::prelude::*;

    let sc = Scene::lattice(4);
    // Exact reference from the modal solution, as in the main comparison.
    let flat = sc.coupling();
    let mut c = Box::new([[0.0f64; 64]; 64]);
    for i in 0..64 { for j in 0..64 { c[i][j] = flat[i * 64 + j]; } }
    let mut st = Box::new(Structure::<64>::zeroed());
    st.init_from_coupling(&c, NO_TWINS);
    let mut x0 = [[0.0f64; 3]; 64];
    for i in 0..64 { x0[i] = sc.pos0[i]; }

    let exact_at = |t: f64| -> Vec<[f64; 3]> {
        let mut out = vec![[0.0f64; 3]; 64];
        for a in 0..3 {
            for m in 0..64 {
                let mut proj = 0.0;
                for i in 0..64 { proj += st.eigenvectors[m][i] * x0[i][a]; }
                let w = st.eigenvalues[m].max(0.0).sqrt();
                let cc = proj * (w * t).cos();
                for i in 0..64 { out[i][a] += cc * st.eigenvectors[m][i]; }
            }
        }
        out
    };

    println!("\n--- is `num_solver_iterations` a fair knob? (lattice 4^3, T = 10) ---");
    println!("cost ~ iters, error ~ 1/iters => accuracy-per-compute should be flat.");
    println!("{:>7} {:>9} {:>12} {:>12} {:>14}", "iters", "steps", "wall ms", "Linf", "Linf x wall");
    let t_sim = 10.0f64;
    for &(iters, steps) in &[(1usize, 4096usize), (2, 2048), (4, 1024), (8, 512), (16, 256)] {
        let dt = t_sim / steps as f64;
        let mut w = PhysicsWorld::new();
        w.gravity = Vec3::new(0.0, 0.0, 0.0);
        w.integration_parameters.dt = dt;
        w.integration_parameters.num_solver_iterations = iters;
        let b: Vec<_> = (0..sc.n).map(|i| {
            let p = sc.pos0[i];
            w.insert_body(RigidBodyBuilder::dynamic()
                .translation(Vec3::new(p[0], p[1], p[2]))
                .additional_mass(1.0).lock_rotations().can_sleep(false))
        }).collect();
        for &(i, j, k) in &sc.edges {
            w.insert_impulse_joint(b[i], b[j],
                SpringJointBuilder::new(0.0, k, 0.0).spring_model(MotorModel::ForceBased));
        }
        let every = (steps / 24).max(1);
        let mut worst = 0.0f64;
        let t0 = Instant::now();
        for i in 1..=steps {
            w.step();
            if i % every == 0 || i == steps {
                let e = exact_at(i as f64 * dt);
                for k in 0..64 {
                    let t = w.bodies[b[k]].translation();
                    let p = [t.x, t.y, t.z];
                    for a in 0..3 { worst = worst.max((p[a] - e[k][a]).abs()); }
                }
            }
        }
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        println!("{:>7} {:>9} {:>12.1} {:>12.3e} {:>14.3e}", iters, steps, ms, worst, worst * ms);
    }
    println!("A flat last column means the knob trades cost against accuracy at a constant");
    println!("rate, so the main comparison's choice of 4 (Rapier's own default) is neutral.");
}
