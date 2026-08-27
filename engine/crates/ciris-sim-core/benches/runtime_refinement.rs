use std::hint::black_box;
use std::time::{Duration, Instant};

use ciris_sim_core::holon::{
    certify, BoundaryModel, Channels, Decomposition, Evaluation, Frontier, Holon, HolonArena,
    NO_HOLON,
};
use ciris_sim_core::regplus::GrossState;
use ciris_sim_core::runtime::{
    certify_runtime, certify_runtime_in, RuntimeArena, RuntimeArenaBuilder, RuntimeBoundaryModel,
    RuntimeFrontier, RuntimeHolonSpec, NO_RUNTIME_HOLON,
};

const CAP: usize = 7;
const W: usize = 0;
const ITERATIONS: usize = 250_000;
const REPEATS: usize = 9;

fn gross(elements: u64) -> GrossState {
    GrossState::aggregate(elements, 2 * elements, [elements as i64, 0])
}

fn fixed_arena() -> HolonArena<CAP, W> {
    let channels = Channels::REG_PLUS.union(Channels::MECHANICAL);
    let nodes = [
        Holon::new(
            NO_HOLON,
            0,
            4,
            gross(8),
            [],
            channels,
            true,
            Decomposition::Expanded,
        ),
        Holon::new(
            0,
            1,
            2,
            gross(4),
            [],
            channels,
            true,
            Decomposition::Expanded,
        ),
        Holon::new(
            0,
            1,
            2,
            gross(4),
            [],
            channels,
            false,
            Decomposition::Expanded,
        ),
        Holon::new(1, 2, 1, gross(2), [], channels, true, Decomposition::Leaf),
        Holon::new(1, 2, 1, gross(2), [], channels, false, Decomposition::Leaf),
        Holon::new(2, 2, 1, gross(2), [], channels, false, Decomposition::Leaf),
        Holon::new(2, 2, 1, gross(2), [], channels, false, Decomposition::Leaf),
    ];
    HolonArena::from_holons(nodes, CAP, 0).unwrap()
}

fn runtime_arena(fixed: &HolonArena<CAP, W>) -> RuntimeArena {
    let mut builder = RuntimeArenaBuilder::with_capacity(CAP, 0);
    for i in 0..fixed.len() {
        let node = fixed.holon(i).unwrap();
        builder
            .push(RuntimeHolonSpec {
                parent: if node.parent == NO_HOLON {
                    NO_RUNTIME_HOLON
                } else {
                    node.parent as u32
                },
                depth: node.depth,
                grain_units: node.grain_units,
                gross: node.gross,
                whole: &node.whole,
                channels: node.channels,
                boundary: node.boundary,
                decomposition: node.decomposition,
            })
            .unwrap();
    }
    builder.build(0).unwrap()
}

struct FixedModel;

impl BoundaryModel<CAP, W, 1> for FixedModel {
    fn evaluate(&mut self, arena: &HolonArena<CAP, W>, frontier: &Frontier<CAP>) -> Evaluation<1> {
        let grain = frontier.represented_grain(arena, 1) as f64;
        Evaluation {
            observables: [grain],
            macro_error_bound: 0.0002 * grain * grain,
            conservation_residual: 0.0,
        }
    }

    fn refinement_priority(
        &self,
        arena: &HolonArena<CAP, W>,
        _frontier: &Frontier<CAP>,
        node: usize,
    ) -> f64 {
        arena.holon(node).unwrap().grain_units as f64
    }
}

struct RuntimeModel;

impl RuntimeBoundaryModel<1> for RuntimeModel {
    fn evaluate(&mut self, arena: &RuntimeArena, frontier: &RuntimeFrontier) -> Evaluation<1> {
        let grain = frontier.represented_grain(arena, 1) as f64;
        Evaluation {
            observables: [grain],
            macro_error_bound: 0.0002 * grain * grain,
            conservation_residual: 0.0,
        }
    }

    fn refinement_priority(
        &self,
        arena: &RuntimeArena,
        _frontier: &RuntimeFrontier,
        node: usize,
    ) -> f64 {
        arena.holon(node).unwrap().grain_units as f64
    }
}

fn per_iteration(elapsed: Duration) -> f64 {
    elapsed.as_nanos() as f64 / ITERATIONS as f64
}

fn median(samples: &mut [f64; REPEATS]) -> f64 {
    samples.sort_by(f64::total_cmp);
    samples[REPEATS / 2]
}

fn main() {
    let fixed = fixed_arena();
    let runtime = runtime_arena(&fixed);

    let fixed_once = certify(&fixed, &mut FixedModel, 0.001, 1.0e-12);
    let runtime_once = certify_runtime(&runtime, &mut RuntimeModel, 0.001, 1.0e-12);
    assert_eq!(fixed_once.observables, runtime_once.observables);
    assert_eq!(fixed_once.macro_error_bound, runtime_once.macro_error_bound);
    assert_eq!(fixed_once.evaluations, runtime_once.evaluations);

    let mut workspace = RuntimeFrontier::root(&runtime);
    let mut fixed_samples = [0.0; REPEATS];
    let mut reused_samples = [0.0; REPEATS];
    let mut owned_samples = [0.0; REPEATS];
    for repeat in 0..REPEATS {
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let certificate = certify(&fixed, &mut FixedModel, 0.001, 1.0e-12);
            black_box(certificate.observables);
        }
        fixed_samples[repeat] = per_iteration(start.elapsed());

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let certificate =
                certify_runtime_in(&runtime, &mut RuntimeModel, &mut workspace, 0.001, 1.0e-12);
            black_box(certificate.observables);
        }
        reused_samples[repeat] = per_iteration(start.elapsed());

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let certificate = certify_runtime(&runtime, &mut RuntimeModel, 0.001, 1.0e-12);
            black_box(certificate.observables);
        }
        owned_samples[repeat] = per_iteration(start.elapsed());
    }

    let fixed_ns = median(&mut fixed_samples);
    let reused_ns = median(&mut reused_samples);
    let owned_ns = median(&mut owned_samples);
    println!(
        "runtime refinement, median of {REPEATS} x {ITERATIONS} identical two-evaluation certificates"
    );
    println!("const arena:            {fixed_ns:9.2} ns/certificate");
    println!(
        "runtime arena, reused:  {reused_ns:9.2} ns/certificate  ({:.3}x const)",
        reused_ns / fixed_ns
    );
    println!(
        "runtime arena, owned:   {owned_ns:9.2} ns/certificate  ({:.3}x const)",
        owned_ns / fixed_ns
    );
    println!(
        "outputs: bit-identical; runtime node header: {} bytes",
        std::mem::size_of::<ciris_sim_core::runtime::RuntimeHolon>()
    );
}
