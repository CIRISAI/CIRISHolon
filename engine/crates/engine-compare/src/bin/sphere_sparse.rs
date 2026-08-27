//! Apples-to-apples benchmark of PR #7's sparse kernel against Rapier 0.35.2 f64.
//!
//! The benchmark has two deliberately separate contracts:
//!
//! 1. **Harmonic accuracy and cost.** Both engines receive the same sphere-distributed
//!    point masses, sparse edge list, stiffnesses, zero rest lengths, initial positions,
//!    zero velocities, timestep, and simulated duration. With no contacts or boundary,
//!    this is a linear ODE. Both outputs are scored against its generalized-eigenmode
//!    closed form, so neither engine is the reference.
//! 2. **Rejected rendered-scene diagnostic.** Both engines receive the same nonzero rest
//!    lengths, particle radii, restitution, counter-shear velocity field, timestep,
//!    duration, and post-step sphere projection. The trajectories diverge, so its raw
//!    timings are diagnostic only and no ratio is emitted. See `contact_matched` for the
//!    solution-gated contact benchmark.

use ciris_sim_core::dynamics::State;
use ciris_sim_core::linalg::{jacobi_eigen, Eigen};
use ciris_sim_core::sparse::{resolve_sphere_contacts, ContactParams, Edge, SparseSystem};
use rapier3d_f64::prelude::*;
use std::cmp::Ordering;
use std::hint::black_box;
use std::time::Instant;

const N: usize = 96;
const E: usize = 270;
const SPHERE_RADIUS: f64 = 1.0;
const PARTICLE_RADIUS: f64 = 0.13;
const STIFFNESS: f64 = 7.5;
const REST_SCALE: f64 = 0.985;
const RESTITUTION: f64 = 0.96;
const CONTACT_DT: f64 = 0.003;
const CONTACT_STEPS: usize = 1_432;
const T_SIM: f64 = 2.0;
const RAPIER_SUBSTEPS: usize = 4;
const REPS: usize = 9;
const ACCURACY_CHECKPOINTS: usize = 24;

#[derive(Clone, Copy)]
struct CandidateEdge {
    i: usize,
    j: usize,
    distance: f64,
}

#[derive(Clone)]
struct ExactSolution {
    modes: Eigen<N>,
    sqrt_mass: [f64; N],
    initial: [[f64; 3]; N],
}

impl ExactSolution {
    /// Exact solution of `M x'' = -L x` from rest. In mass-normalized coordinates,
    /// `y = sqrt(M)x`, the symmetric operator is `M^-1/2 L M^-1/2`.
    fn new(system: &SparseSystem<N, E>, initial: [[f64; 3]; N]) -> Self {
        let mut normalized = [[0.0; N]; N];
        let mut sqrt_mass = [0.0; N];
        for (slot, mass) in sqrt_mass.iter_mut().zip(&system.mass) {
            *slot = mass.sqrt();
        }
        for edge in system.edges {
            let k = edge.stiffness;
            normalized[edge.i][edge.i] += k / system.mass[edge.i];
            normalized[edge.j][edge.j] += k / system.mass[edge.j];
            let off = -k / (sqrt_mass[edge.i] * sqrt_mass[edge.j]);
            normalized[edge.i][edge.j] += off;
            normalized[edge.j][edge.i] += off;
        }
        let modes = jacobi_eigen(&normalized);
        assert!(
            modes.converged,
            "exact-reference eigensolve did not converge"
        );
        Self {
            modes,
            sqrt_mass,
            initial,
        }
    }

    fn positions(&self, time: f64) -> Box<[[f64; 3]; N]> {
        let mut out = Box::new([[0.0; 3]; N]);
        for axis in 0..3 {
            for mode in 0..N {
                let mut projection = 0.0;
                for i in 0..N {
                    projection +=
                        self.modes.vectors[mode][i] * self.sqrt_mass[i] * self.initial[i][axis];
                }
                let frequency = self.modes.values[mode].max(0.0).sqrt();
                let amplitude = projection * (frequency * time).cos();
                for i in 0..N {
                    out[i][axis] += amplitude * self.modes.vectors[mode][i] / self.sqrt_mass[i];
                }
            }
        }
        out
    }
}

#[derive(Clone, Copy)]
struct RunPoint {
    dt: f64,
    steps: usize,
    sparse_error: f64,
    sparse_ms: f64,
    rapier_error: f64,
    rapier_ms: f64,
}

#[derive(Clone, Copy)]
struct Timing {
    min_ms: f64,
    median_ms: f64,
    max_ms: f64,
}

impl Timing {
    fn measure(mut run: impl FnMut()) -> Self {
        run();
        let mut samples = [0.0; REPS];
        for sample in &mut samples {
            let start = Instant::now();
            run();
            *sample = start.elapsed().as_secs_f64() * 1.0e3;
        }
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        Self {
            min_ms: samples[0],
            median_ms: samples[REPS / 2],
            max_ms: samples[REPS - 1],
        }
    }
}

struct DisjointSet {
    parent: [usize; N],
    rank: [u8; N],
}

impl DisjointSet {
    fn new() -> Self {
        let mut parent = [0; N];
        for (i, slot) in parent.iter_mut().enumerate() {
            *slot = i;
        }
        Self {
            parent,
            rank: [0; N],
        }
    }

    fn find(&mut self, node: usize) -> usize {
        if self.parent[node] != node {
            self.parent[node] = self.find(self.parent[node]);
        }
        self.parent[node]
    }

    fn union(&mut self, a: usize, b: usize) -> bool {
        let mut root_a = self.find(a);
        let mut root_b = self.find(b);
        if root_a == root_b {
            return false;
        }
        if self.rank[root_a] < self.rank[root_b] {
            std::mem::swap(&mut root_a, &mut root_b);
        }
        self.parent[root_b] = root_a;
        if self.rank[root_a] == self.rank[root_b] {
            self.rank[root_a] += 1;
        }
        true
    }
}

fn fibonacci_sphere() -> [[f64; 3]; N] {
    let mut points = [[0.0; 3]; N];
    let golden_angle = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    for (i, point) in points.iter_mut().enumerate() {
        let y = 1.0 - 2.0 * (i as f64 + 0.5) / N as f64;
        let ring_radius = (1.0 - y * y).sqrt();
        let phi = golden_angle * i as f64;
        *point = [ring_radius * phi.cos(), y, ring_radius * phi.sin()];
    }
    points
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Same deterministic mesh construction as the rendered demo: local MST, then the
/// shortest unused chords until the fixed capacity is full.
fn mesh_edges(points: &[[f64; 3]; N], rest_scale: f64) -> [Edge; E] {
    let mut candidates = Vec::with_capacity(N * (N - 1) / 2);
    for i in 0..N {
        for j in (i + 1)..N {
            candidates.push(CandidateEdge {
                i,
                j,
                distance: distance(points[i], points[j]),
            });
        }
    }
    candidates.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.i.cmp(&b.i))
            .then_with(|| a.j.cmp(&b.j))
    });

    let mut chosen = Vec::with_capacity(E);
    let mut selected = [[false; N]; N];
    let mut sets = DisjointSet::new();
    for candidate in &candidates {
        if sets.union(candidate.i, candidate.j) {
            chosen.push(*candidate);
            selected[candidate.i][candidate.j] = true;
            if chosen.len() == N - 1 {
                break;
            }
        }
    }
    for candidate in candidates {
        if chosen.len() == E {
            break;
        }
        if !selected[candidate.i][candidate.j] {
            chosen.push(candidate);
            selected[candidate.i][candidate.j] = true;
        }
    }

    let mut edges = [Edge::ZERO; E];
    for (slot, candidate) in edges.iter_mut().zip(chosen) {
        *slot = Edge::new(
            candidate.i,
            candidate.j,
            STIFFNESS,
            candidate.distance * rest_scale,
        );
    }
    edges
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn tangent_component(vector: [f64; 3], normal: [f64; 3]) -> [f64; 3] {
    let radial = dot(vector, normal) / dot(normal, normal);
    [
        vector[0] - radial * normal[0],
        vector[1] - radial * normal[1],
        vector[2] - radial * normal[2],
    ]
}

fn counter_shear_state(points: [[f64; 3]; N]) -> State<N> {
    let mut state = State::at_rest(points);
    for i in 0..N {
        let p = state.pos[i];
        let azimuthal = cross([0.0, 1.0, 0.0], p);
        let phi = p[2].atan2(p[0]);
        let direction = if p[1] >= 0.0 { 1.0 } else { -1.0 };
        let raw = [
            0.72 * direction * azimuthal[0],
            0.08 * (3.0 * phi).sin(),
            0.72 * direction * azimuthal[2],
        ];
        state.vel[i] = tangent_component(raw, p);
    }
    state
}

fn constrain_core_to_sphere(state: &mut State<N>) {
    for i in 0..N {
        let p = state.pos[i];
        let radius = dot(p, p).sqrt();
        let projected = [
            SPHERE_RADIUS * p[0] / radius,
            SPHERE_RADIUS * p[1] / radius,
            SPHERE_RADIUS * p[2] / radius,
        ];
        state.pos[i] = projected;
        state.vel[i] = tangent_component(state.vel[i], projected);
    }
}

struct RapierRun {
    world: PhysicsWorld,
    bodies: Vec<RigidBodyHandle>,
}

fn build_rapier(
    system: &SparseSystem<N, E>,
    initial: &State<N>,
    dt: f64,
    contacts: bool,
) -> RapierRun {
    let mut world = PhysicsWorld::new();
    world.gravity = Vec3::ZERO;
    world.integration_parameters.dt = dt;
    world.integration_parameters.num_solver_iterations = RAPIER_SUBSTEPS;

    let mut bodies = Vec::with_capacity(N);
    for i in 0..N {
        let p = initial.pos[i];
        let v = initial.vel[i];
        let handle = world.insert_body(
            RigidBodyBuilder::dynamic()
                .translation(Vec3::new(p[0], p[1], p[2]))
                .linvel(Vec3::new(v[0], v[1], v[2]))
                .additional_mass(system.mass[i])
                .lock_rotations()
                .can_sleep(false),
        );
        if contacts {
            world.insert_collider(
                ColliderBuilder::ball(PARTICLE_RADIUS)
                    .density(0.0)
                    .friction(0.0)
                    .restitution(RESTITUTION),
                Some(handle),
            );
        }
        bodies.push(handle);
    }
    for edge in system.edges {
        world.insert_impulse_joint(
            bodies[edge.i],
            bodies[edge.j],
            SpringJointBuilder::new(edge.rest_length, edge.stiffness, 0.0)
                .spring_model(MotorModel::ForceBased)
                .contacts_enabled(true),
        );
    }
    RapierRun { world, bodies }
}

fn constrain_rapier_to_sphere(run: &mut RapierRun) {
    for &handle in &run.bodies {
        let body = &mut run.world.bodies[handle];
        let p = body.translation();
        let radius = p.length();
        let projected = p * (SPHERE_RADIUS / radius);
        let velocity = body.linvel();
        let tangent = velocity - projected * velocity.dot(projected);
        body.set_translation(projected, true);
        body.set_linvel(tangent, true);
    }
}

fn rapier_positions(run: &RapierRun) -> Box<[[f64; 3]; N]> {
    let mut out = Box::new([[0.0; 3]; N]);
    for (i, &handle) in run.bodies.iter().enumerate() {
        let p = run.world.bodies[handle].translation();
        out[i] = [p.x, p.y, p.z];
    }
    out
}

fn linf(actual: &[[f64; 3]; N], expected: &[[f64; 3]; N]) -> f64 {
    let mut worst = 0.0_f64;
    for i in 0..N {
        for axis in 0..3 {
            worst = worst.max((actual[i][axis] - expected[i][axis]).abs());
        }
    }
    worst
}

fn sparse_accuracy(
    system: &SparseSystem<N, E>,
    exact: &ExactSolution,
    dt: f64,
    steps: usize,
) -> f64 {
    let mut state = State::at_rest(exact.initial);
    let every = (steps / ACCURACY_CHECKPOINTS).max(1);
    let mut worst = 0.0_f64;
    for step in 1..=steps {
        system.step(&mut state, dt);
        if step % every == 0 || step == steps {
            worst = worst.max(linf(&state.pos, &exact.positions(step as f64 * dt)));
        }
    }
    worst
}

fn rapier_accuracy(
    system: &SparseSystem<N, E>,
    exact: &ExactSolution,
    dt: f64,
    steps: usize,
) -> f64 {
    let initial = State::at_rest(exact.initial);
    let mut run = build_rapier(system, &initial, dt, false);
    let every = (steps / ACCURACY_CHECKPOINTS).max(1);
    let mut worst = 0.0_f64;
    for step in 1..=steps {
        run.world.step();
        if step % every == 0 || step == steps {
            worst = worst.max(linf(
                &rapier_positions(&run),
                &exact.positions(step as f64 * dt),
            ));
        }
    }
    worst
}

fn time_sparse_harmonic(
    system: &SparseSystem<N, E>,
    initial: [[f64; 3]; N],
    dt: f64,
    steps: usize,
) -> Timing {
    Timing::measure(|| {
        let mut state = State::at_rest(initial);
        for _ in 0..steps {
            system.step(&mut state, dt);
        }
        black_box(state.pos[0]);
    })
}

/// `Timing::measure` owns the outer clock, so Rapier construction must not be inside
/// its closure for a stepping-only result. This variant records samples explicitly.
fn time_rapier_steps(
    system: &SparseSystem<N, E>,
    initial: State<N>,
    dt: f64,
    steps: usize,
    contacts: bool,
    project: bool,
) -> Timing {
    let mut samples = [0.0; REPS];
    for sample_index in 0..=REPS {
        let mut run = build_rapier(system, &initial, dt, contacts);
        let start = Instant::now();
        for _ in 0..steps {
            run.world.step();
            if project {
                constrain_rapier_to_sphere(&mut run);
            }
        }
        let elapsed = start.elapsed().as_secs_f64() * 1.0e3;
        black_box(run.world.bodies.len());
        if sample_index > 0 {
            samples[sample_index - 1] = elapsed;
        }
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    Timing {
        min_ms: samples[0],
        median_ms: samples[REPS / 2],
        max_ms: samples[REPS - 1],
    }
}

fn time_sparse_contacts(
    system: &SparseSystem<N, E>,
    initial: State<N>,
    contacts: &ContactParams,
) -> Timing {
    Timing::measure(|| {
        let mut state = initial;
        for _ in 0..CONTACT_STEPS {
            system.step(&mut state, CONTACT_DT);
            constrain_core_to_sphere(&mut state);
            resolve_sphere_contacts(&mut state, &system.mass, contacts);
            constrain_core_to_sphere(&mut state);
        }
        black_box(state.pos[0]);
    })
}

fn contact_signature_core(
    system: &SparseSystem<N, E>,
    initial: State<N>,
    contacts: &ContactParams,
) -> (usize, f64, f64) {
    let mut state = initial;
    let mut contact_events = 0;
    for _ in 0..CONTACT_STEPS {
        system.step(&mut state, CONTACT_DT);
        constrain_core_to_sphere(&mut state);
        contact_events += resolve_sphere_contacts(&mut state, &system.mass, contacts);
        constrain_core_to_sphere(&mut state);
    }
    let mut radius_error = 0.0_f64;
    let mut mean_speed = 0.0;
    for i in 0..N {
        radius_error = radius_error.max((dot(state.pos[i], state.pos[i]).sqrt() - 1.0).abs());
        mean_speed += dot(state.vel[i], state.vel[i]).sqrt() / N as f64;
    }
    (contact_events, radius_error, mean_speed)
}

fn contact_signature_rapier(system: &SparseSystem<N, E>, initial: State<N>) -> (usize, f64, f64) {
    let mut run = build_rapier(system, &initial, CONTACT_DT, true);
    let mut contact_events = 0;
    for _ in 0..CONTACT_STEPS {
        run.world.step();
        contact_events += run
            .world
            .contact_pairs()
            .filter(|pair| pair.has_any_active_contact())
            .count();
        constrain_rapier_to_sphere(&mut run);
    }
    let mut radius_error = 0.0_f64;
    let mut mean_speed = 0.0;
    for &handle in &run.bodies {
        let body = &run.world.bodies[handle];
        radius_error = radius_error.max((body.translation().length() - 1.0).abs());
        mean_speed += body.linvel().length() / N as f64;
    }
    (contact_events, radius_error, mean_speed)
}

fn setup_timings(edges: [Edge; E], points: [[f64; 3]; N]) -> (Timing, Timing) {
    let mut sparse_samples = [0.0; REPS];
    for sample_index in 0..=REPS {
        let start = Instant::now();
        let system = SparseSystem::<N, E>::from_edges(edges);
        black_box(&system);
        let elapsed = start.elapsed().as_secs_f64() * 1.0e3;
        if sample_index > 0 {
            sparse_samples[sample_index - 1] = elapsed;
        }
    }
    sparse_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let sparse = Timing {
        min_ms: sparse_samples[0],
        median_ms: sparse_samples[REPS / 2],
        max_ms: sparse_samples[REPS - 1],
    };

    let system = SparseSystem::<N, E>::from_edges(edges);
    let initial = State::at_rest(points);
    let mut rapier_samples = [0.0; REPS];
    for sample_index in 0..=REPS {
        let start = Instant::now();
        let run = build_rapier(&system, &initial, CONTACT_DT, false);
        black_box(run.bodies.len());
        let elapsed = start.elapsed().as_secs_f64() * 1.0e3;
        if sample_index > 0 {
            rapier_samples[sample_index - 1] = elapsed;
        }
    }
    rapier_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let rapier = Timing {
        min_ms: rapier_samples[0],
        median_ms: rapier_samples[REPS / 2],
        max_ms: rapier_samples[REPS - 1],
    };
    (sparse, rapier)
}

fn print_matched_targets(points: &[RunPoint]) {
    println!("\nMeasured matched-accuracy minima (no extrapolation):");
    println!(
        "{:>12} {:>12} {:>12} {:>12}",
        "target Linf", "sparse ms", "rapier ms", "R/S"
    );
    // Decade targets are fixed independently of the measured values. The step ladder
    // extends on both sides far enough that the minimum is an observed point, not a
    // boundary artifact.
    for target in [1.0e-3, 1.0e-4, 1.0e-5] {
        let sparse = points
            .iter()
            .filter(|point| point.sparse_error <= target)
            .min_by(|a, b| {
                a.sparse_ms
                    .partial_cmp(&b.sparse_ms)
                    .unwrap_or(Ordering::Equal)
            });
        let rapier = points
            .iter()
            .filter(|point| point.rapier_error <= target)
            .min_by(|a, b| {
                a.rapier_ms
                    .partial_cmp(&b.rapier_ms)
                    .unwrap_or(Ordering::Equal)
            });
        match (sparse, rapier) {
            (Some(sparse), Some(rapier)) => println!(
                "{:>12.1e} {:>12.3} {:>12.3} {:>11.2}x",
                target,
                sparse.sparse_ms,
                rapier.rapier_ms,
                rapier.rapier_ms / sparse.sparse_ms,
            ),
            _ => println!(
                "{:>12.1e} {:>12} {:>12} {:>12}",
                target, "n/a", "n/a", "n/a"
            ),
        }
    }
}

fn observed_order(points: &[RunPoint], sparse: bool) -> f64 {
    let mut total = 0.0;
    for pair in points.windows(2) {
        let coarse = if sparse {
            pair[0].sparse_error
        } else {
            pair[0].rapier_error
        };
        let fine = if sparse {
            pair[1].sparse_error
        } else {
            pair[1].rapier_error
        };
        total += (coarse / fine).log2();
    }
    total / (points.len() - 1) as f64
}

fn main() {
    let points = fibonacci_sphere();
    let harmonic_edges = mesh_edges(&points, 0.0);
    let harmonic_system = SparseSystem::<N, E>::from_edges(harmonic_edges);
    let exact = ExactSolution::new(&harmonic_system, points);

    println!("PR #7 sparse sphere vs Rapier 0.35.2 f64");
    println!("host: Apple Silicon ({})", std::env::consts::ARCH);
    println!(
        "scene: N={N}, E={E}, k={STIFFNESS}, weighted-degree masses, Rapier substeps={RAPIER_SUBSTEPS}"
    );
    println!("timings: release+LTO, {REPS} samples after warmup; median [min, max]\n");

    println!("PART A — harmonic, independently exact, no contacts or boundary");
    println!(
        "fixed simulated time T={T_SIM}; trajectory Linf over {ACCURACY_CHECKPOINTS} checkpoints"
    );
    println!(
        "{:>8} {:>7} {:>13} {:>12} {:>13} {:>12} {:>9}",
        "dt", "steps", "sparse Linf", "sparse ms", "rapier Linf", "rapier ms", "R/S"
    );
    let mut points_measured = Vec::new();
    for steps in [8, 16, 32, 64, 128, 256, 512, 1_024, 2_048, 4_096] {
        let dt = T_SIM / steps as f64;
        let sparse_error = sparse_accuracy(&harmonic_system, &exact, dt, steps);
        let rapier_error = rapier_accuracy(&harmonic_system, &exact, dt, steps);
        let sparse_timing = time_sparse_harmonic(&harmonic_system, points, dt, steps);
        let rapier_timing = time_rapier_steps(
            &harmonic_system,
            State::at_rest(points),
            dt,
            steps,
            false,
            false,
        );
        let point = RunPoint {
            dt,
            steps,
            sparse_error,
            sparse_ms: sparse_timing.median_ms,
            rapier_error,
            rapier_ms: rapier_timing.median_ms,
        };
        println!(
            "{:>8.6} {:>7} {:>13.4e} {:>12.3} {:>13.4e} {:>12.3} {:>8.2}x",
            point.dt,
            point.steps,
            point.sparse_error,
            point.sparse_ms,
            point.rapier_error,
            point.rapier_ms,
            point.rapier_ms / point.sparse_ms,
        );
        points_measured.push(point);
    }
    println!(
        "observed convergence order: sparse {:.3}, Rapier {:.3}",
        observed_order(&points_measured, true),
        observed_order(&points_measured, false),
    );
    print_matched_targets(&points_measured);

    let (sparse_setup, rapier_setup) = setup_timings(harmonic_edges, points);
    println!("\nSetup, same precomputed edge list (median [min, max]):");
    println!(
        "  sparse {:8.4} ms [{:.4}, {:.4}]    Rapier {:8.4} ms [{:.4}, {:.4}]    R/S {:.1}x",
        sparse_setup.median_ms,
        sparse_setup.min_ms,
        sparse_setup.max_ms,
        rapier_setup.median_ms,
        rapier_setup.min_ms,
        rapier_setup.max_ms,
        rapier_setup.median_ms / sparse_setup.median_ms,
    );
    println!(
        "  SparseSystem size: {:.2} KiB (fixed arrays; no allocation)",
        std::mem::size_of::<SparseSystem<N, E>>() as f64 / 1024.0
    );

    println!("\nPART B — REJECTED rendered counter-shear timing diagnostic");
    println!(
        "dt={CONTACT_DT}, steps={CONTACT_STEPS}, T={:.3}, rest_scale={REST_SCALE}, radius={PARTICLE_RADIUS}, e={RESTITUTION}",
        CONTACT_DT * CONTACT_STEPS as f64,
    );
    println!("Both: same graph/masses/state and the same post-step sphere projector.");
    println!("The trajectories fail the 99.9% solution gate; no timing ratio is admissible.\n");

    let contact_edges = mesh_edges(&points, REST_SCALE);
    let contact_system = SparseSystem::<N, E>::from_edges(contact_edges);
    let contact_initial = counter_shear_state(points);
    let contacts = ContactParams {
        radius: PARTICLE_RADIUS,
        restitution: RESTITUTION,
        correction: 0.92,
    };
    let sparse_contact_time = time_sparse_contacts(&contact_system, contact_initial, &contacts);
    let rapier_contact_time = time_rapier_steps(
        &contact_system,
        contact_initial,
        CONTACT_DT,
        CONTACT_STEPS,
        true,
        true,
    );
    let sparse_signature = contact_signature_core(&contact_system, contact_initial, &contacts);
    let rapier_signature = contact_signature_rapier(&contact_system, contact_initial);
    println!(
        "  sparse {:8.3} ms [{:.3}, {:.3}] = {:8.1} ns/step",
        sparse_contact_time.median_ms,
        sparse_contact_time.min_ms,
        sparse_contact_time.max_ms,
        sparse_contact_time.median_ms * 1.0e6 / CONTACT_STEPS as f64,
    );
    println!(
        "  Rapier {:8.3} ms [{:.3}, {:.3}] = {:8.1} ns/step",
        rapier_contact_time.median_ms,
        rapier_contact_time.min_ms,
        rapier_contact_time.max_ms,
        rapier_contact_time.median_ms * 1.0e6 / CONTACT_STEPS as f64,
    );
    println!("  ratio WITHHELD (solutions are not equivalent)");
    println!("\nUntimed end-state signatures (diagnostics, not equivalence gates):");
    println!(
        "  sparse contacts={} radius_err={:.2e} mean_speed={:.4}",
        sparse_signature.0, sparse_signature.1, sparse_signature.2
    );
    println!(
        "  Rapier contacts={} radius_err={:.2e} mean_speed={:.4}",
        rapier_signature.0, rapier_signature.1, rapier_signature.2
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_solution_reconstructs_initial_state() {
        let points = fibonacci_sphere();
        let system = SparseSystem::<N, E>::from_edges(mesh_edges(&points, 0.0));
        let exact = ExactSolution::new(&system, points);
        assert!(linf(&exact.positions(0.0), &points) < 1.0e-11);
    }

    #[test]
    fn both_harmonic_solvers_converge_toward_the_exact_solution() {
        let points = fibonacci_sphere();
        let system = SparseSystem::<N, E>::from_edges(mesh_edges(&points, 0.0));
        let exact = ExactSolution::new(&system, points);
        let sparse_coarse = sparse_accuracy(&system, &exact, 0.04, 50);
        let sparse_fine = sparse_accuracy(&system, &exact, 0.02, 100);
        let rapier_coarse = rapier_accuracy(&system, &exact, 0.04, 50);
        let rapier_fine = rapier_accuracy(&system, &exact, 0.02, 100);
        assert!(sparse_fine < sparse_coarse);
        assert!(rapier_fine < rapier_coarse);
        assert!((3.8..4.2).contains(&(sparse_coarse / sparse_fine)));
        assert!((1.8..2.2).contains(&(rapier_coarse / rapier_fine)));
    }
}
