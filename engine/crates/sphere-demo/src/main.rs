//! Deterministic particles-on-a-sphere demonstration for the sparse mechanics path.
//!
//! The simulation runs natively in Rust and writes a self-describing JSON frame bundle.
//! The adjacent browser viewer consumes that artifact without reimplementing physics.

use ciris_sim_core::dynamics::State;
use ciris_sim_core::sparse::{resolve_sphere_contacts, ContactParams, Edge, SparseSystem};
use serde::Serialize;
use std::cmp::Ordering;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

const N: usize = 96;
const E: usize = 270;
const SPHERE_RADIUS: f64 = 1.0;
const PARTICLE_RADIUS: f64 = 0.13;
const SPRING_STIFFNESS: f64 = 7.5;
const DT: f64 = 0.003;
const DEFAULT_FRAMES: usize = 180;
const DEFAULT_STEPS_PER_FRAME: usize = 8;

#[derive(Clone, Copy)]
enum ScenarioKind {
    CoherentSpin,
    CounterShear,
    StandingWave,
}

impl ScenarioKind {
    const ALL: [ScenarioKind; 3] = [
        ScenarioKind::CoherentSpin,
        ScenarioKind::CounterShear,
        ScenarioKind::StandingWave,
    ];

    fn id(self) -> &'static str {
        match self {
            ScenarioKind::CoherentSpin => "coherent-spin",
            ScenarioKind::CounterShear => "counter-shear",
            ScenarioKind::StandingWave => "standing-wave",
        }
    }

    fn title(self) -> &'static str {
        match self {
            ScenarioKind::CoherentSpin => "Coherent spin",
            ScenarioKind::CounterShear => "Counter-rotating shear",
            ScenarioKind::StandingWave => "Standing surface wave",
        }
    }

    fn description(self) -> &'static str {
        match self {
            ScenarioKind::CoherentSpin => {
                "A near-rigid azimuthal flow checks the spherical constraint under smooth motion."
            }
            ScenarioKind::CounterShear => {
                "The hemispheres counter-rotate, loading the spring mesh and activating contacts near the equator."
            }
            ScenarioKind::StandingWave => {
                "A deterministic three-lobed tangential velocity field launches a surface wave through the mesh."
            }
        }
    }
}

#[derive(Serialize)]
struct Bundle {
    meta: Meta,
    edges: Vec<[usize; 2]>,
    simulations: Vec<Simulation>,
}

#[derive(Serialize)]
struct Meta {
    title: &'static str,
    particle_count: usize,
    edge_count: usize,
    frame_count: usize,
    steps_per_frame: usize,
    dt: f64,
    sphere_radius: f64,
    particle_radius: f64,
    integrator: &'static str,
    spring_kernel: &'static str,
    contact_solver: &'static str,
    surface_constraint: &'static str,
    generator: &'static str,
}

#[derive(Serialize)]
struct Simulation {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    frames: Vec<Frame>,
    summary: Summary,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct Frame {
    step: usize,
    time: f64,
    positions: Vec<[f32; 3]>,
    speeds: Vec<f32>,
    kinetic_energy: f64,
    spring_energy: f64,
    total_energy: f64,
    contact_events: usize,
    max_radius_error: f64,
    mean_speed: f64,
}

#[derive(Serialize)]
struct Summary {
    total_contact_events: usize,
    peak_contact_events_per_frame: usize,
    peak_speed: f64,
    max_radius_error: f64,
    initial_energy: f64,
    final_energy: f64,
}

#[derive(Clone, Copy)]
struct CandidateEdge {
    i: usize,
    j: usize,
    distance: f64,
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

/// Build a connected local mesh: a shortest-edge spanning tree, followed by the next
/// shortest unused chords until the fixed edge capacity is filled.
fn sphere_edges(points: &[[f64; 3]; N]) -> [Edge; E] {
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

    assert_eq!(chosen.len(), E);
    let mut edges = [Edge::ZERO; E];
    for (slot, candidate) in edges.iter_mut().zip(chosen) {
        *slot = Edge::new(
            candidate.i,
            candidate.j,
            SPRING_STIFFNESS,
            candidate.distance * 0.985,
        );
    }
    edges
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn initial_state(points: [[f64; 3]; N], scenario: ScenarioKind) -> State<N> {
    let mut state = State::at_rest(points);
    for i in 0..N {
        let p = state.pos[i];
        let azimuthal = cross([0.0, 1.0, 0.0], p);
        let phi = p[2].atan2(p[0]);
        let raw = match scenario {
            ScenarioKind::CoherentSpin => [
                0.48 * azimuthal[0] + 0.035 * (4.0 * phi).sin() * p[1],
                0.025 * (3.0 * phi).cos(),
                0.48 * azimuthal[2] - 0.035 * (4.0 * phi).sin() * p[0],
            ],
            ScenarioKind::CounterShear => {
                let direction = if p[1] >= 0.0 { 1.0 } else { -1.0 };
                [
                    0.72 * direction * azimuthal[0],
                    0.08 * (3.0 * phi).sin(),
                    0.72 * direction * azimuthal[2],
                ]
            }
            ScenarioKind::StandingWave => {
                let meridional = [-p[1] * p[0], 1.0 - p[1] * p[1], -p[1] * p[2]];
                let amplitude = 0.62 * (3.0 * phi).sin();
                [
                    amplitude * meridional[0],
                    amplitude * meridional[1],
                    amplitude * meridional[2],
                ]
            }
        };
        state.vel[i] = tangent_component(raw, p);
    }
    state
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn tangent_component(vector: [f64; 3], normal: [f64; 3]) -> [f64; 3] {
    let radial = dot(vector, normal) / dot(normal, normal);
    [
        vector[0] - radial * normal[0],
        vector[1] - radial * normal[1],
        vector[2] - radial * normal[2],
    ]
}

/// Geometric projection for the demo's unit-sphere boundary. The sparse core remains
/// boundary-agnostic; this explicitly records the caller-supplied E9 choice.
fn constrain_to_sphere(state: &mut State<N>) {
    for i in 0..N {
        let p = state.pos[i];
        let radius = dot(p, p).sqrt();
        assert!(radius > 1.0e-12, "particle reached the sphere centre");
        let projected = [
            SPHERE_RADIUS * p[0] / radius,
            SPHERE_RADIUS * p[1] / radius,
            SPHERE_RADIUS * p[2] / radius,
        ];
        state.pos[i] = projected;
        state.vel[i] = tangent_component(state.vel[i], projected);
    }
}

fn capture_frame(
    state: &State<N>,
    system: &SparseSystem<N, E>,
    step: usize,
    contact_events: usize,
) -> Frame {
    let mut max_radius_error = 0.0_f64;
    let mut speed_sum = 0.0;
    let mut positions = Vec::with_capacity(N);
    let mut speeds = Vec::with_capacity(N);

    for i in 0..N {
        let p = state.pos[i];
        let v = state.vel[i];
        let radius = dot(p, p).sqrt();
        let speed = dot(v, v).sqrt();
        max_radius_error = max_radius_error.max((radius - SPHERE_RADIUS).abs());
        speed_sum += speed;
        positions.push([p[0] as f32, p[1] as f32, p[2] as f32]);
        speeds.push(speed as f32);
    }

    let kinetic_energy = system.kinetic_energy(state);
    let spring_energy = system.potential_energy(state);
    Frame {
        step,
        time: step as f64 * DT,
        positions,
        speeds,
        kinetic_energy,
        spring_energy,
        total_energy: kinetic_energy + spring_energy,
        contact_events,
        max_radius_error,
        mean_speed: speed_sum / N as f64,
    }
}

fn simulate(
    kind: ScenarioKind,
    system: &SparseSystem<N, E>,
    points: [[f64; 3]; N],
    frame_count: usize,
    steps_per_frame: usize,
) -> Simulation {
    let mut state = initial_state(points, kind);
    let contacts = ContactParams {
        radius: PARTICLE_RADIUS,
        restitution: 0.96,
        correction: 0.92,
    };
    let mut frames = Vec::with_capacity(frame_count);
    let mut total_contact_events = 0;
    frames.push(capture_frame(&state, system, 0, 0));

    for frame_index in 1..frame_count {
        let mut frame_contacts = 0;
        for _ in 0..steps_per_frame {
            system.step(&mut state, DT);
            constrain_to_sphere(&mut state);
            frame_contacts += resolve_sphere_contacts(&mut state, &system.mass, &contacts);
            constrain_to_sphere(&mut state);
        }
        total_contact_events += frame_contacts;
        frames.push(capture_frame(
            &state,
            system,
            frame_index * steps_per_frame,
            frame_contacts,
        ));
    }

    let peak_contact_events_per_frame = frames
        .iter()
        .map(|frame| frame.contact_events)
        .max()
        .unwrap_or(0);
    let peak_speed = frames
        .iter()
        .flat_map(|frame| frame.speeds.iter().copied())
        .fold(0.0_f32, f32::max) as f64;
    let max_radius_error = frames
        .iter()
        .map(|frame| frame.max_radius_error)
        .fold(0.0_f64, f64::max);
    let initial_energy = frames
        .first()
        .map(|frame| frame.total_energy)
        .unwrap_or(0.0);
    let final_energy = frames.last().map(|frame| frame.total_energy).unwrap_or(0.0);

    Simulation {
        id: kind.id(),
        title: kind.title(),
        description: kind.description(),
        frames,
        summary: Summary {
            total_contact_events,
            peak_contact_events_per_frame,
            peak_speed,
            max_radius_error,
            initial_energy,
            final_energy,
        },
    }
}

fn build_bundle(frame_count: usize, steps_per_frame: usize) -> Bundle {
    assert!(frame_count >= 2, "frame count must be at least two");
    assert!(steps_per_frame >= 1, "steps per frame must be positive");
    let points = fibonacci_sphere();
    let edge_array = sphere_edges(&points);
    let system = SparseSystem::<N, E>::from_edges(edge_array);
    let edges = edge_array.iter().map(|edge| [edge.i, edge.j]).collect();
    let simulations = ScenarioKind::ALL
        .into_iter()
        .map(|scenario| simulate(scenario, &system, points, frame_count, steps_per_frame))
        .collect();

    Bundle {
        meta: Meta {
            title: "Particles on a sphere",
            particle_count: N,
            edge_count: E,
            frame_count,
            steps_per_frame,
            dt: DT,
            sphere_radius: SPHERE_RADIUS,
            particle_radius: PARTICLE_RADIUS,
            integrator: "mass-aware velocity Verlet",
            spring_kernel: "fixed-capacity sparse O(E)",
            contact_solver: "deterministic frictionless spheres (e=0.96); O(N^2) pair discovery",
            surface_constraint: "position projection plus tangential velocity projection",
            generator: "sphere-demo (native Rust)",
        },
        edges,
        simulations,
    }
}

fn parse_args() -> (PathBuf, usize, usize) {
    let mut output = PathBuf::from("crates/sphere-demo/viewer/simulation.json");
    let mut frame_count = DEFAULT_FRAMES;
    let mut steps_per_frame = DEFAULT_STEPS_PER_FRAME;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => {
                output = PathBuf::from(args.next().expect("--output requires a path"));
            }
            "--frames" => {
                frame_count = args
                    .next()
                    .expect("--frames requires a value")
                    .parse()
                    .expect("--frames must be an integer");
            }
            "--steps-per-frame" => {
                steps_per_frame = args
                    .next()
                    .expect("--steps-per-frame requires a value")
                    .parse()
                    .expect("--steps-per-frame must be an integer");
            }
            "--help" | "-h" => {
                println!("sphere-demo [--output PATH] [--frames N] [--steps-per-frame N]");
                std::process::exit(0);
            }
            other => panic!("unknown argument: {other}"),
        }
    }
    (output, frame_count, steps_per_frame)
}

fn write_bundle(path: &Path, bundle: &Bundle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let writer = BufWriter::new(File::create(path)?);
    serde_json::to_writer(writer, bundle)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (output, frame_count, steps_per_frame) = parse_args();
    let bundle = build_bundle(frame_count, steps_per_frame);
    write_bundle(&output, &bundle)?;

    println!(
        "wrote {} scenarios × {} frames × {} particles to {}",
        bundle.simulations.len(),
        bundle.meta.frame_count,
        bundle.meta.particle_count,
        output.display()
    );
    for simulation in &bundle.simulations {
        println!(
            "  {:<24} contacts={:<6} peak_speed={:.4} radius_err={:.2e} energy={:.4} -> {:.4}",
            simulation.id,
            simulation.summary.total_contact_events,
            simulation.summary.peak_speed,
            simulation.summary.max_radius_error,
            simulation.summary.initial_energy,
            simulation.summary.final_energy,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fibonacci_points_are_on_the_unit_sphere() {
        for point in fibonacci_sphere() {
            assert!((dot(point, point).sqrt() - 1.0).abs() < 1.0e-12);
        }
    }

    #[test]
    fn sparse_mesh_is_connected_local_and_unique() {
        let points = fibonacci_sphere();
        let edges = sphere_edges(&points);
        let mut sets = DisjointSet::new();
        let mut seen = [[false; N]; N];
        let mut longest = 0.0_f64;
        for edge in edges {
            assert!(edge.i < edge.j);
            assert!(!seen[edge.i][edge.j]);
            seen[edge.i][edge.j] = true;
            sets.union(edge.i, edge.j);
            longest = longest.max(distance(points[edge.i], points[edge.j]));
        }
        let root = sets.find(0);
        for i in 1..N {
            assert_eq!(sets.find(i), root);
        }
        assert!(longest < 0.65, "unexpectedly nonlocal chord: {longest}");
    }

    #[test]
    fn simulation_is_deterministic_and_stays_on_sphere() {
        let first = build_bundle(12, 3);
        let second = build_bundle(12, 3);
        for (a, b) in first.simulations.iter().zip(&second.simulations) {
            assert_eq!(a.frames, b.frames);
            assert!(a.summary.max_radius_error < 1.0e-12);
        }
    }
}
