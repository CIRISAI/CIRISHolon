//! The two engines, driven over the same scene, plus the EXACT solution both are
//! measured against.
//!
//! ## Why the reference is exact rather than "a very small step"
//!
//! In the harmonic regime (`rest_scale = 0`, FSD §13) the force law is `F = -L x` with
//! `L` the coupling Laplacian, so the system is LINEAR and its solution is closed-form.
//! Released from rest, each Cartesian axis evolves independently as
//!
//! ```text
//!     x_a(t) = sum_m  v_m * (v_m . x_a(0)) * cos(sqrt(lambda_m) t)
//! ```
//!
//! over the Laplacian's eigenpairs. The zero mode needs no special case: `cos(0) = 1`,
//! which is exactly right for a system starting at rest.
//!
//! This matters for fairness. Using either engine's own output at a tiny step as the
//! reference would measure the other engine's distance from a competitor, and would
//! quietly reward whichever integrator the reference was built from. The closed form is
//! a property of the ODE and of neither engine. It also happens to be free: the
//! eigendecomposition is already in `Structure`, computed by the E10 general path.

use ciris_sim_core::dynamics::{step, Params, State};
use ciris_sim_core::structure::{Structure, NO_TWINS};
use rapier3d_f64::prelude::*;
use std::time::Instant;

use crate::scene::Scene;

/// Harmonic regime: `F = -L x` exactly. The regime in which the twin-decoupling
/// theorem holds (FSD §13) and the one in which the closed form above is valid.
pub fn harmonic(dt: f64) -> Params {
    Params { dt, damping: 1.0, repulsion: 0.0, softening: 0.1, centering: 0.0, rest_scale: 0.0 }
}

pub fn build_structure<const N: usize>(scene: &Scene) -> Box<Structure<N>> {
    assert_eq!(scene.n, N);
    let flat = scene.coupling();
    let mut c = Box::new([[0.0f64; N]; N]);
    for i in 0..N {
        for j in 0..N {
            c[i][j] = flat[i * N + j];
        }
    }
    let mut st = Box::new(Structure::<N>::zeroed());
    st.init_from_coupling(&c, NO_TWINS);
    st
}

pub fn initial_state<const N: usize>(scene: &Scene) -> Box<State<N>> {
    let mut pos = Box::new([[0.0f64; 3]; N]);
    for i in 0..N {
        pos[i] = scene.pos0[i];
    }
    Box::new(State { pos: *pos, vel: [[0.0; 3]; N] })
}

/// The closed-form solution at time `t`, from rest.
pub fn exact<const N: usize>(
    st: &Structure<N>,
    x0: &[[f64; 3]; N],
    t: f64,
) -> Box<[[f64; 3]; N]> {
    let mut out = Box::new([[0.0f64; 3]; N]);
    for a in 0..3 {
        // Modal coefficients of the initial displacement along this axis.
        for m in 0..N {
            let mut proj = 0.0f64;
            for i in 0..N {
                proj += st.eigenvectors[m][i] * x0[i][a];
            }
            let w = st.eigenvalues[m].max(0.0).sqrt();
            let c = proj * (w * t).cos();
            for i in 0..N {
                out[i][a] += c * st.eigenvectors[m][i];
            }
        }
    }
    out
}

/// Largest absolute position error against the closed form, sampled at `checkpoints`
/// evenly spaced times through the run. The error in a symplectic integrator grows
/// secularly with phase drift, so the maximum sits near the end; the sampling is there
/// to catch a method whose error does something else.
pub fn accuracy_ours<const N: usize>(
    st: &Structure<N>,
    x0: &[[f64; 3]; N],
    dt: f64,
    steps: usize,
    checkpoints: usize,
) -> f64 {
    // NOTE: `steps * dt` is the SIMULATED TIME, and error grows with it. Callers
    // comparing accuracy across scenes must hold `steps * dt` fixed; an earlier version
    // of this harness chose `steps` for timing stability instead, which made a scene
    // with a short run look accurate and a scene with a long run look inaccurate. That
    // is a property of the run length, not of the engine.
    let p = harmonic(dt);
    let mut s = Box::new(State { pos: *x0, vel: [[0.0; 3]; N] });
    let every = (steps / checkpoints).max(1);
    let mut worst = 0.0f64;
    for i in 1..=steps {
        step(&mut s, st, &p, false);
        if i % every == 0 || i == steps {
            let e = exact(st, x0, i as f64 * dt);
            for k in 0..N {
                for a in 0..3 {
                    let d = (s.pos[k][a] - e[k][a]).abs();
                    if d > worst { worst = d; }
                }
            }
        }
    }
    worst
}

/// Wall-clock nanoseconds per step, best of `reps`. No error computation inside the
/// timed region.
pub fn time_ours<const N: usize>(
    st: &Structure<N>,
    x0: &[[f64; 3]; N],
    dt: f64,
    steps: usize,
    reps: usize,
) -> f64 {
    let p = harmonic(dt);
    let mut best = f64::INFINITY;
    for _ in 0..reps {
        let mut s = Box::new(State { pos: *x0, vel: [[0.0; 3]; N] });
        let t0 = Instant::now();
        for _ in 0..steps {
            step(&mut s, st, &p, false);
        }
        let ns = t0.elapsed().as_secs_f64() * 1e9 / steps as f64;
        std::hint::black_box(&s);
        best = best.min(ns);
    }
    best
}

// ------------------------------------------------------------------- Rapier

/// The same scene in Rapier: `n` dynamic unit point masses, no colliders (so no
/// broad phase and no contacts to solve), zero gravity, rotations locked, sleeping
/// disabled, and one zero-rest-length spring joint per edge at the edge's stiffness.
pub struct RapierScene {
    pub world: PhysicsWorld,
    pub bodies: Vec<RigidBodyHandle>,
}

pub fn build_rapier(scene: &Scene, dt: f64, iters: usize) -> RapierScene {
    let mut world = PhysicsWorld::new();
    world.gravity = Vec3::new(0.0, 0.0, 0.0);
    world.integration_parameters.dt = dt;
    world.integration_parameters.num_solver_iterations = iters;

    let bodies: Vec<_> = (0..scene.n)
        .map(|i| {
            let p = scene.pos0[i];
            world.insert_body(
                RigidBodyBuilder::dynamic()
                    .translation(Vec3::new(p[0], p[1], p[2]))
                    .additional_mass(1.0)
                    .lock_rotations()
                    .can_sleep(false),
            )
        })
        .collect();

    for &(i, j, k) in &scene.edges {
        world.insert_impulse_joint(
            bodies[i],
            bodies[j],
            SpringJointBuilder::new(0.0, k, 0.0).spring_model(MotorModel::ForceBased),
        );
    }
    RapierScene { world, bodies }
}

pub fn rapier_positions(rs: &RapierScene) -> Vec<[f64; 3]> {
    rs.bodies
        .iter()
        .map(|&h| {
            let t = rs.world.bodies[h].translation();
            [t.x, t.y, t.z]
        })
        .collect()
}

pub fn accuracy_rapier<const N: usize>(
    st: &Structure<N>,
    x0: &[[f64; 3]; N],
    scene: &Scene,
    dt: f64,
    steps: usize,
    iters: usize,
    checkpoints: usize,
) -> f64 {
    // Same caveat on `steps * dt` as `accuracy_ours`.
    let mut rs = build_rapier(scene, dt, iters);
    let every = (steps / checkpoints).max(1);
    let mut worst = 0.0f64;
    for i in 1..=steps {
        rs.world.step();
        if i % every == 0 || i == steps {
            let e = exact(st, x0, i as f64 * dt);
            let p = rapier_positions(&rs);
            for k in 0..N {
                for a in 0..3 {
                    let d = (p[k][a] - e[k][a]).abs();
                    if d > worst { worst = d; }
                }
            }
        }
    }
    worst
}

pub fn time_rapier(scene: &Scene, dt: f64, steps: usize, iters: usize, reps: usize) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..reps {
        let mut rs = build_rapier(scene, dt, iters);
        let t0 = Instant::now();
        for _ in 0..steps {
            rs.world.step();
        }
        let ns = t0.elapsed().as_secs_f64() * 1e9 / steps as f64;
        std::hint::black_box(&rs.world.bodies.len());
        best = best.min(ns);
    }
    best
}

/// Setup cost: what it takes to make each engine ready to step. Ours runs an `O(N^3)`
/// eigensolve; Rapier inserts `N` bodies and `E` joints.
pub fn setup_cost_ours<const N: usize>(scene: &Scene, reps: usize) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..reps {
        let t0 = Instant::now();
        let st = build_structure::<N>(scene);
        let ns = t0.elapsed().as_secs_f64() * 1e9;
        std::hint::black_box(&st.eigenvalues[0]);
        best = best.min(ns);
    }
    best
}

pub fn setup_cost_rapier(scene: &Scene, reps: usize) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..reps {
        let t0 = Instant::now();
        let rs = build_rapier(scene, 1e-3, 4);
        let ns = t0.elapsed().as_secs_f64() * 1e9;
        std::hint::black_box(rs.bodies.len());
        best = best.min(ns);
    }
    best
}

// ---------------------------------------------------------------------------
// REMOVED: a fitted-extrapolation route to "cost at matched accuracy".
//
// It fitted `Linf = c * dt^p` on three step sizes per engine and solved for the step
// needed to hit a target, so that Rapier would not have to be run for the ~4e7 steps a
// tight target implies on a dense scene. It produced ratios of 1e8 and above, and they
// were fiction.
//
// The fit was valid on the sparse lattices and worthless on the dense graphs. On
// `complete K64` every step size Rapier could afford gave Linf between 0.52 and 0.87 on
// a scene of unit extent: the error had SATURATED at the size of the signal, so the
// exponent it fitted (0.371, where the true order is 1) described the approach to
// saturation and not a convergence rate. Extrapolating a saturated exponent by four
// decades is how a benchmark manufactures a win.
//
// The check that caught it is worth keeping in mind: predicted-vs-measured on a scene
// cheap enough to verify directly drifted to 2.76x for Rapier while staying at 1.02x
// for us — i.e. the extrapolation was wrong, and wrong in OUR favour.
//
// What replaced it never predicts a point it has not run: see `achieved_*` below, and
// the measured ladder in the report's PART D2.
// ---------------------------------------------------------------------------

// ------------------------------------------- equal-compute accuracy (measured)

/// What one engine achieved on one scene inside a fixed wall-clock budget.
#[derive(Debug, Clone, Copy)]
pub struct Achieved {
    pub ns_per_step: f64,
    pub steps: usize,
    pub dt: f64,
    pub err: f64,
    pub wall_ms: f64,
}

/// Give the engine `budget_ms` of wall clock, let it spend that on as many steps as it
/// can afford over a FIXED simulated time, and report the accuracy it reached.
///
/// Fully measured — no extrapolation. An earlier version of this comparison fitted
/// `err = c * dt^p` on three points and solved for the step needed to hit a target.
/// That fit was valid on the sparse scenes and WORTHLESS on the dense ones, where
/// Rapier's error at every affordable step size was 0.5-0.9 on a scene of unit extent:
/// the error had saturated at the size of the signal, so the fitted exponent (0.37)
/// described noise, and extrapolating it produced ratios of 1e8 and up. Those numbers
/// were discarded. This function cannot make that mistake because it never predicts a
/// point it has not run.
pub fn achieved_ours<const N: usize>(
    st: &Structure<N>,
    x0: &[[f64; 3]; N],
    t_sim: f64,
    budget_ms: f64,
) -> Achieved {
    let ns = time_ours::<N>(st, x0, t_sim / 1024.0, 512, 3);
    let steps = ((budget_ms * 1e6 / ns) as usize).clamp(8, 20_000_000);
    let dt = t_sim / steps as f64;
    let err = accuracy_ours::<N>(st, x0, dt, steps, 24);
    Achieved { ns_per_step: ns, steps, dt, err, wall_ms: ns * steps as f64 / 1e6 }
}

pub fn achieved_rapier<const N: usize>(
    st: &Structure<N>,
    x0: &[[f64; 3]; N],
    scene: &Scene,
    t_sim: f64,
    budget_ms: f64,
    iters: usize,
) -> Achieved {
    let ns = time_rapier(scene, t_sim / 1024.0, 256, iters, 3);
    let steps = ((budget_ms * 1e6 / ns) as usize).clamp(8, 2_000_000);
    let dt = t_sim / steps as f64;
    let err = accuracy_rapier::<N>(st, x0, scene, dt, steps, iters, 24);
    Achieved { ns_per_step: ns, steps, dt, err, wall_ms: ns * steps as f64 / 1e6 }
}
