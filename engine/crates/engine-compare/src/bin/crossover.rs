//! Where does Rapier overtake us on sparse scenes? Measured, not projected.
//!
//! FSD §10 expects an all-pairs integrator to lose to a general engine on large sparse
//! scenes, and with §11's profile-class reduction retracted there is nothing to offset
//! it. This finds the actual crossover on a 3D spring lattice.
//!
//! THROUGHPUT ONLY. The harmonic step (`rest_scale = 0`) reads only the coupling: the
//! metric is multiplied by `rest_scale` and the resulting `rest == 0.0` branch never
//! divides by it, so a structure carrying the coupling alone steps identically to a
//! fully-derived one. That lets the sweep skip the O(N^3) eigensolve, which by N = 512
//! already costs ~50 s. Setup cost is reported separately (main comparison, PART E) and
//! is NOT hidden by this shortcut — it is measured there and it is brutal.
//!
//! No accuracy is claimed here: without the eigensolve there is no closed-form
//! reference, and a speed number at unstated accuracy is not a benchmark. This table
//! answers one narrow question — per-step cost versus N — and nothing else.

#[path = "../scene.rs"]
mod scene;

use ciris_sim_core::dynamics::{step, Params, State};
use ciris_sim_core::structure::Structure;
use rapier3d_f64::prelude::*;
use scene::Scene;
use std::time::Instant;

/// A structure carrying ONLY the coupling. Valid for harmonic stepping (see module doc);
/// invalid for anything that reads the metric, the spectrum or the sectors.
fn coupling_only<const N: usize>(sc: &Scene) -> Box<Structure<N>> {
    let flat = sc.coupling();
    let mut st = Box::new(Structure::<N>::zeroed());
    for i in 0..N {
        for j in 0..N {
            st.coupling[i][j] = flat[i * N + j];
            st.coupling_sym[i][j] = flat[i * N + j];
        }
    }
    st
}

fn time_ours<const N: usize>(sc: &Scene, steps: usize) -> f64 {
    let st = coupling_only::<N>(sc);
    let mut pos = Box::new([[0.0f64; 3]; N]);
    for i in 0..N {
        pos[i] = sc.pos0[i];
    }
    let p = Params { dt: 1e-3, damping: 1.0, repulsion: 0.0, softening: 0.1, centering: 0.0, rest_scale: 0.0 };
    let mut best = f64::INFINITY;
    for _ in 0..3 {
        let mut s = Box::new(State::<N> { pos: *pos, vel: [[0.0; 3]; N] });
        let t0 = Instant::now();
        for _ in 0..steps {
            step(&mut s, &st, &p, false);
        }
        best = best.min(t0.elapsed().as_secs_f64() * 1e9 / steps as f64);
        std::hint::black_box(&s.pos[0][0]);
    }
    best
}

fn time_rapier(sc: &Scene, steps: usize) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..3 {
        let mut w = PhysicsWorld::new();
        w.gravity = Vec3::new(0.0, 0.0, 0.0);
        w.integration_parameters.dt = 1e-3;
        w.integration_parameters.num_solver_iterations = 4;
        let b: Vec<_> = (0..sc.n)
            .map(|i| {
                let p = sc.pos0[i];
                w.insert_body(
                    RigidBodyBuilder::dynamic()
                        .translation(Vec3::new(p[0], p[1], p[2]))
                        .additional_mass(1.0)
                        .lock_rotations()
                        .can_sleep(false),
                )
            })
            .collect();
        for &(i, j, k) in &sc.edges {
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
        std::hint::black_box(w.bodies.len());
    }
    best
}

macro_rules! row {
    ($s:expr; $($n:literal),*) => {{
        let sc = Scene::lattice($s);
        let steps = (40_000_000 / (sc.n * sc.n).max(1)).clamp(20, 2000);
        let (o, r) = match sc.n {
            $($n => (time_ours::<$n>(&sc, steps), time_rapier(&sc, steps)),)*
            other => panic!("N = {other} not instantiated"),
        };
        let mem = 8.0 * (8.0 * (sc.n as f64) * (sc.n as f64) + 3.0 * sc.n as f64) + 72.0;
        println!(
            "{:>4} {:>7} {:>8} {:>8.4} {:>13.1} {:>14.1} {:>10.2}x {:>12.1}",
            $s, sc.n, sc.edges.len(), sc.density(), o, r, r / o, mem / 1e6
        );
    }};
}

fn main() {
    std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024 * 1024)
        .spawn(|| {
            println!("Sparse-lattice crossover — per-step cost only, harmonic regime, f64 both sides");
            println!("host: {}, best of 3\n", std::env::consts::ARCH);
            println!(
                "{:>4} {:>7} {:>8} {:>8} {:>13} {:>14} {:>11} {:>12}",
                "side", "N", "edges", "density", "ours ns/step", "rapier ns/step", "ratio", "ours mem MB"
            );
            println!("{}", "-".repeat(92));
            row!(3;  27);
            row!(4;  64);
            row!(5;  125);
            row!(6;  216);
            row!(7;  343);
            row!(8;  512);
            row!(9;  729);
            row!(10; 1000);
            row!(11; 1331);
            row!(12; 1728);
            row!(13; 2197);
            row!(14; 2744);
            println!("\nratio > 1 means we are faster. The trend is the point, not any single row.");
            println!("Our per-step cost is O(N^2) regardless of edge count; Rapier's is O(N + E),");
            println!("and on a lattice E grows like N, so the ratio must fall like 1/N and cross 1.");
            println!("\nMemory is the harder wall: a Structure<N> holds EIGHT dense N x N f64");
            println!("matrices. Projected 1.07 GB at N = 4096 and 4.1 GB at N = 8000 — a scene a");
            println!("general engine holds in a few MB. That is a structural limit, not a slowdown.");
        })
        .unwrap()
        .join()
        .unwrap();
}
