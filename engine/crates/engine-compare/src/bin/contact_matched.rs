//! Solution-gated contact benchmark for PR #7's sparse resolver and Rapier 0.35.2 f64.
//!
//! The workload is 48 independent, simultaneous, head-on collisions between equal
//! spheres. Its piecewise-linear hard-sphere trajectory is analytic, so neither engine
//! supplies the reference. A timing ratio is emitted only after all three comparisons
//! (sparse/exact, Rapier/exact, and sparse/Rapier) agree by at least 99.9% in both
//! position and velocity over synchronized trajectory checkpoints.

use ciris_sim_core::dynamics::State;
use ciris_sim_core::sparse::{ContactParams, SparseSystem};
use rapier3d_f64::prelude::*;
use std::cmp::Ordering;
use std::hint::black_box;
use std::time::Instant;

const N: usize = 96;
const PAIRS: usize = N / 2;
const RADIUS: f64 = 0.1;
const DIAMETER: f64 = 2.0 * RADIUS;
const INCIDENT_SPEED: f64 = 0.5;
const CLOSING_SPEED: f64 = 2.0 * INCIDENT_SPEED;
const RESTITUTION: f64 = 0.96;
const IMPACT_TIME: f64 = 0.277;
const T_SIM: f64 = 0.6;
const CHECKPOINTS: usize = 24;
const RAPIER_SOLVER_ITERATIONS: usize = 4;
const REPS: usize = 9;
const MAX_NORMALIZED_ERROR: f64 = 0.001;
const STEP_LADDER: [usize; 7] = [384, 768, 1_536, 3_072, 6_144, 12_288, 24_576];

#[derive(Clone)]
struct Frame {
    pos: [[f64; 3]; N],
    vel: [[f64; 3]; N],
}

#[derive(Clone, Copy)]
struct Agreement {
    position: f64,
    velocity: f64,
}

impl Agreement {
    fn score(self) -> f64 {
        (1.0 - self.position.max(self.velocity)).clamp(0.0, 1.0)
    }

    fn passes(self) -> bool {
        self.position <= MAX_NORMALIZED_ERROR && self.velocity <= MAX_NORMALIZED_ERROR
    }
}

#[derive(Clone, Copy)]
struct Timing {
    min_ms: f64,
    median_ms: f64,
    max_ms: f64,
}

impl Timing {
    fn from_samples(mut samples: [f64; REPS]) -> Self {
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        Self {
            min_ms: samples[0],
            median_ms: samples[REPS / 2],
            max_ms: samples[REPS - 1],
        }
    }

    fn measure(mut run: impl FnMut()) -> Self {
        run();
        let mut samples = [0.0; REPS];
        for sample in &mut samples {
            let start = Instant::now();
            run();
            *sample = start.elapsed().as_secs_f64() * 1.0e3;
        }
        Self::from_samples(samples)
    }
}

fn pair_center(pair: usize) -> [f64; 3] {
    let x = pair % 6;
    let y = (pair / 6) % 4;
    let z = pair / 24;
    [
        (x as f64 - 2.5) * 1.5,
        (y as f64 - 1.5) * 1.5,
        (z as f64 - 0.5) * 1.5,
    ]
}

fn pair_axis(pair: usize) -> usize {
    pair % 3
}

fn exact_state(time: f64) -> State<N> {
    let mut state = State::ZERO;
    let (half_separation, left_velocity) = if time <= IMPACT_TIME {
        (
            RADIUS + INCIDENT_SPEED * (IMPACT_TIME - time),
            INCIDENT_SPEED,
        )
    } else {
        (
            RADIUS + RESTITUTION * INCIDENT_SPEED * (time - IMPACT_TIME),
            -RESTITUTION * INCIDENT_SPEED,
        )
    };

    for pair in 0..PAIRS {
        let left = 2 * pair;
        let right = left + 1;
        let axis = pair_axis(pair);
        let center = pair_center(pair);
        state.pos[left] = center;
        state.pos[right] = center;
        state.pos[left][axis] -= half_separation;
        state.pos[right][axis] += half_separation;
        state.vel[left][axis] = left_velocity;
        state.vel[right][axis] = -left_velocity;
    }
    state
}

fn exact_trace() -> Vec<Frame> {
    (1..=CHECKPOINTS)
        .map(|checkpoint| {
            let state = exact_state(T_SIM * checkpoint as f64 / CHECKPOINTS as f64);
            Frame {
                pos: state.pos,
                vel: state.vel,
            }
        })
        .collect()
}

fn contact_params() -> ContactParams {
    ContactParams {
        radius: RADIUS,
        restitution: RESTITUTION,
        correction: 1.0,
    }
}

fn sparse_trace(steps: usize) -> Vec<Frame> {
    assert_eq!(steps % CHECKPOINTS, 0);
    let dt = T_SIM / steps as f64;
    let every = steps / CHECKPOINTS;
    let system = SparseSystem::<N, 0>::from_edges([]);
    let mut state = exact_state(0.0);
    let contacts = contact_params();
    let mut trace = Vec::with_capacity(CHECKPOINTS);
    for step in 1..=steps {
        system.step_with_contacts(&mut state, dt, &contacts);
        if step % every == 0 {
            trace.push(Frame {
                pos: state.pos,
                vel: state.vel,
            });
        }
    }
    trace
}

struct RapierRun {
    world: PhysicsWorld,
    bodies: Vec<RigidBodyHandle>,
}

fn build_rapier(dt: f64) -> RapierRun {
    let initial = exact_state(0.0);
    let mut world = PhysicsWorld::new();
    world.gravity = Vec3::ZERO;
    world.integration_parameters.dt = dt;
    world.integration_parameters.num_solver_iterations = RAPIER_SOLVER_ITERATIONS;

    // The benchmark contract is an exact hard-sphere surface. Rapier's default 0.02 m
    // speculative-contact margin would deliberately bounce these 0.1 m balls before
    // their surfaces meet, so set that engine tolerance to the contract's zero margin.
    world.integration_parameters.normalized_prediction_distance = 0.0;
    world.integration_parameters.normalized_allowed_linear_error = 0.0;

    let mut bodies = Vec::with_capacity(N);
    for i in 0..N {
        let p = initial.pos[i];
        let v = initial.vel[i];
        let body = world.insert_body(
            RigidBodyBuilder::dynamic()
                .translation(Vec3::new(p[0], p[1], p[2]))
                .linvel(Vec3::new(v[0], v[1], v[2]))
                .additional_mass(1.0)
                .lock_rotations()
                .can_sleep(false),
        );
        world.insert_collider(
            ColliderBuilder::ball(RADIUS)
                .density(0.0)
                .friction(0.0)
                .restitution(RESTITUTION),
            Some(body),
        );
        bodies.push(body);
    }
    RapierRun { world, bodies }
}

fn rapier_frame(run: &RapierRun) -> Frame {
    let mut frame = Frame {
        pos: [[0.0; 3]; N],
        vel: [[0.0; 3]; N],
    };
    for (i, &handle) in run.bodies.iter().enumerate() {
        let body = &run.world.bodies[handle];
        let p = body.translation();
        let v = body.linvel();
        frame.pos[i] = [p.x, p.y, p.z];
        frame.vel[i] = [v.x, v.y, v.z];
    }
    frame
}

fn rapier_trace(steps: usize) -> Vec<Frame> {
    assert_eq!(steps % CHECKPOINTS, 0);
    let every = steps / CHECKPOINTS;
    let mut run = build_rapier(T_SIM / steps as f64);
    let mut trace = Vec::with_capacity(CHECKPOINTS);
    for step in 1..=steps {
        run.world.step();
        if step % every == 0 {
            trace.push(rapier_frame(&run));
        }
    }
    trace
}

fn vector_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn agreement(actual: &[Frame], expected: &[Frame]) -> Agreement {
    assert_eq!(actual.len(), expected.len());
    let mut position = 0.0_f64;
    let mut velocity = 0.0_f64;
    for (a, b) in actual.iter().zip(expected) {
        for i in 0..N {
            position = position.max(vector_distance(a.pos[i], b.pos[i]) / DIAMETER);
            velocity = velocity.max(vector_distance(a.vel[i], b.vel[i]) / CLOSING_SPEED);
        }
    }
    Agreement { position, velocity }
}

fn time_sparse(steps: usize) -> Timing {
    let system = SparseSystem::<N, 0>::from_edges([]);
    let initial = exact_state(0.0);
    let contacts = contact_params();
    let dt = T_SIM / steps as f64;
    Timing::measure(|| {
        let mut state = initial;
        for _ in 0..steps {
            system.step_with_contacts(&mut state, dt, &contacts);
        }
        black_box(state.pos[0]);
        black_box(state.vel[0]);
    })
}

fn time_rapier(steps: usize) -> Timing {
    let dt = T_SIM / steps as f64;
    let mut samples = [0.0; REPS];
    for sample_index in 0..=REPS {
        let mut run = build_rapier(dt);
        let start = Instant::now();
        for _ in 0..steps {
            run.world.step();
        }
        let elapsed = start.elapsed().as_secs_f64() * 1.0e3;
        black_box(rapier_frame(&run));
        if sample_index > 0 {
            samples[sample_index - 1] = elapsed;
        }
    }
    Timing::from_samples(samples)
}

fn percent(value: Agreement) -> f64 {
    100.0 * value.score()
}

fn main() {
    println!("SOLUTION-GATED CONTACT BENCHMARK — sparse PR #7 vs Rapier 0.35.2 f64");
    println!(
        "N={N} (48 isolated pairs), radius={RADIUS}, v={INCIDENT_SPEED}, e={RESTITUTION}, impact={IMPACT_TIME}, T={T_SIM}"
    );
    println!(
        "Gate: worst of position/diameter and velocity/closing-speed <= {:.4} ({:.3}% agreement) over {CHECKPOINTS} checkpoints.",
        MAX_NORMALIZED_ERROR,
        100.0 * (1.0 - MAX_NORMALIZED_ERROR),
    );
    println!("Both engines must pass exact and cross-engine gates before timing.\n");
    println!(" steps       dt       sparse/exact  Rapier/exact  sparse/Rapier  gate");

    let exact = exact_trace();
    let mut selected = None;
    for steps in STEP_LADDER {
        let sparse = sparse_trace(steps);
        let rapier = rapier_trace(steps);
        let sparse_exact = agreement(&sparse, &exact);
        let rapier_exact = agreement(&rapier, &exact);
        let cross = agreement(&sparse, &rapier);
        let passes = sparse_exact.passes() && rapier_exact.passes() && cross.passes();
        println!(
            "{steps:6}  {:9.7}    {:9.5}%    {:9.5}%    {:9.5}%   {}",
            T_SIM / steps as f64,
            percent(sparse_exact),
            percent(rapier_exact),
            percent(cross),
            if passes { "PASS" } else { "reject" },
        );
        if passes {
            selected = Some((steps, sparse_exact, rapier_exact, cross));
            break;
        }
    }

    let Some((steps, sparse_exact, rapier_exact, cross)) = selected else {
        panic!("no tested timestep passed the 99.9% solution-equivalence gate; no timing emitted");
    };

    println!(
        "\nPASS at equal dt={:.9} ({steps} steps).",
        T_SIM / steps as f64
    );
    println!(
        "Worst normalized errors (position, velocity): sparse/exact=({:.3e}, {:.3e}), Rapier/exact=({:.3e}, {:.3e}), cross=({:.3e}, {:.3e})",
        sparse_exact.position,
        sparse_exact.velocity,
        rapier_exact.position,
        rapier_exact.velocity,
        cross.position,
        cross.velocity,
    );
    println!("\nTiming is now admissible (stepping only; 9 samples after warmup):");
    let sparse_time = time_sparse(steps);
    let rapier_time = time_rapier(steps);
    println!(
        "  sparse {:9.3} ms [{:.3}, {:.3}]",
        sparse_time.median_ms, sparse_time.min_ms, sparse_time.max_ms
    );
    println!(
        "  Rapier {:9.3} ms [{:.3}, {:.3}]",
        rapier_time.median_ms, rapier_time.min_ms, rapier_time.max_ms
    );
    println!(
        "  Rapier/sparse at >=99.9% same solution: {:.2}x",
        rapier_time.median_ms / sparse_time.median_ms
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analytic_collision_is_geometrically_continuous() {
        let at_impact = exact_state(IMPACT_TIME);
        for pair in 0..PAIRS {
            let left = at_impact.pos[2 * pair];
            let right = at_impact.pos[2 * pair + 1];
            assert!((vector_distance(left, right) - DIAMETER).abs() < 1.0e-12);
        }
    }

    #[test]
    fn intended_pairs_are_initially_separate_and_cross_pairs_cannot_touch() {
        let initial = exact_state(0.0);
        for i in 0..N {
            for j in (i + 1)..N {
                let distance = vector_distance(initial.pos[i], initial.pos[j]);
                if i / 2 == j / 2 {
                    assert!(distance > DIAMETER);
                } else {
                    assert!(distance > 5.0 * DIAMETER);
                }
            }
        }
    }

    #[test]
    fn both_engines_converge_toward_the_analytic_contact_solution() {
        let exact = exact_trace();
        let sparse_coarse = agreement(&sparse_trace(384), &exact);
        let sparse_fine = agreement(&sparse_trace(768), &exact);
        let rapier_coarse = agreement(&rapier_trace(384), &exact);
        let rapier_fine = agreement(&rapier_trace(768), &exact);
        assert!(sparse_fine.position < sparse_coarse.position);
        assert!(rapier_fine.position < rapier_coarse.position);
        assert!(sparse_fine.velocity <= sparse_coarse.velocity + 1.0e-12);
        assert!(rapier_fine.velocity <= rapier_coarse.velocity + 1.0e-12);
    }
}
