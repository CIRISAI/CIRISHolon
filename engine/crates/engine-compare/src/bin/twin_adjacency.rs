//! Is the twin adjacency in h3ere2's paths a THEOREM, a conditional, or an artefact?
//!
//! `inference-scout` observed that Structure and Circumstances — the twin pair (9,6) —
//! arrive adjacent in every path it generated. This decides what that is.
//!
//! The hypothesis under test: `path::relax` steps with `symmetrised = true`, so the
//! dynamics runs on the Z2xZ2 GROUP AVERAGE of whatever coupling it is handed, and that
//! matrix is exactly twin-invariant by construction. If the initial condition is also
//! twin-invariant, then the whole trajectory is, so the two twins move IDENTICALLY and
//! arrive on the same step — adjacency in the sorted path is then just the index
//! tiebreak separating an exact tie.
//!
//! Which makes the real question: are the seedings twin-invariant? They are block
//! seedings, and the blocks come from `Site.block` in the Lean, whose automorphism group
//! IS the twin group — so every block must be a union of twin orbits. Both twin pairs do
//! in fact sit inside one block each (Rules holds 0 and 7; Manner holds 9 and 6).
//!
//! Four arms, designed so each one can falsify the reading:
//!   1. real coupling, symmetrised, block seeds   -> ties predicted
//!   2. real coupling, MEASURED (not symmetrised) -> ties predicted to BREAK
//!   3. real coupling, symmetrised, twin-SPLIT seeds -> ties predicted to BREAK
//!   4. scrambled couplings, symmetrised          -> ties predicted to HOLD anyway

use ciris_sim_core::data::{COUPLING, KINDS, N, TWINS};
use ciris_sim_core::dynamics::{step, Params, State};
use ciris_sim_core::structure::Structure;

const AMPLITUDE: f64 = 1.0;
const THRESHOLD: f64 = 1e-3;
const MAX_STEPS: usize = 20_000;

/// h3ere2's four surface blocks, as engine indices (from `blocks::BLOCK_MEMBERS`).
fn blocks() -> Vec<(&'static str, Vec<usize>)> {
    let idx = |n: &str| KINDS.iter().position(|k| *k == n).unwrap();
    vec![
        ("Facts",    ["Facts", "Confidence", "Model", "Premises"].iter().map(|s| idx(s)).collect()),
        ("Rules",    ["Rules", "Priorities", "Process"].iter().map(|s| idx(s)).collect()),
        ("Identity", ["Identity"].iter().map(|s| idx(s)).collect()),
        ("Manner",   ["Manner", "Structure", "Circumstances"].iter().map(|s| idx(s)).collect()),
    ]
}

struct Run {
    arrival: [Option<usize>; N],
    /// Max |x_a(t) - x_b(t)| over the whole run, per twin pair.
    twin_traj_gap: [f64; 2],
    /// Whether that gap was exactly 0.0 at every step.
    twin_bit_identical: [bool; 2],
}

/// h3ere2's `path::relax`, reproduced, with `symmetrised` exposed as a knob.
fn relax(coupling: &[[f64; N]; N], seeds: &[usize], symmetrised: bool) -> Run {
    let st = Structure::<N>::from_coupling(coupling, TWINS);
    let params = Params::harmonic();

    let mut pos = [[0.0f64; 3]; N];
    for &s in seeds {
        pos[s][0] = AMPLITUDE;
    }
    let mut state = State::at_rest(pos);

    let mut first: [Option<usize>; N] = [None; N];
    for &s in seeds {
        first[s] = Some(0);
    }
    let mut gap = [0.0f64; 2];
    let mut identical = [true; 2];

    let mut t = 0usize;
    while t < MAX_STEPS && first.iter().any(|f| f.is_none()) {
        step(&mut state, &st, &params, symmetrised);
        t += 1;
        for (k, &(a, b)) in TWINS.iter().enumerate() {
            for axis in 0..3 {
                let d = (state.pos[a][axis] - state.pos[b][axis]).abs();
                if d > gap[k] {
                    gap[k] = d;
                }
                if state.pos[a][axis] != state.pos[b][axis] {
                    identical[k] = false;
                }
            }
        }
        for i in 0..N {
            if first[i].is_some() {
                continue;
            }
            let p = state.pos[i];
            let d = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            if d > THRESHOLD {
                first[i] = Some(t);
            }
        }
    }
    Run { arrival: first, twin_traj_gap: gap, twin_bit_identical: identical }
}

fn report(label: &str, r: &Run) {
    let tie = |a: usize, b: usize| match (r.arrival[a], r.arrival[b]) {
        (Some(x), Some(y)) if x == y => format!("TIED at {x}"),
        (x, y) => format!("SPLIT {x:?} vs {y:?}"),
    };
    println!(
        "  {:<34} twin(0,7) Pri/Prc: {:<20} twin(9,6) Str/Cir: {:<20}",
        label,
        tie(TWINS[0].0, TWINS[0].1),
        tie(TWINS[1].0, TWINS[1].1)
    );
    println!(
        "  {:<34} trajectory gap: {:.3e} / {:.3e}   bit-identical: {} / {}",
        "",
        r.twin_traj_gap[0], r.twin_traj_gap[1],
        r.twin_bit_identical[0], r.twin_bit_identical[1]
    );
}

fn main() {
    println!("TWIN ADJACENCY — theorem, conditional, or artefact?\n");
    println!("twins: {:?} = ({}, {}) and ({}, {})", TWINS,
        KINDS[TWINS[0].0], KINDS[TWINS[0].1], KINDS[TWINS[1].0], KINDS[TWINS[1].1]);

    println!("\n=== ARM 1: real coupling, SYMMETRISED (what h3ere2 actually runs) ===");
    println!("prediction: both twin pairs tie exactly, in every block");
    for (name, seeds) in blocks() {
        report(&format!("block {name} {seeds:?}"), &relax(&COUPLING, &seeds, true));
    }

    println!("\n=== ARM 2: real coupling, MEASURED (symmetrised = false) ===");
    println!("prediction: ties BREAK — the measured coupling is not twin-symmetric");
    for (name, seeds) in blocks() {
        report(&format!("block {name} {seeds:?}"), &relax(&COUPLING, &seeds, false));
    }

    println!("\n=== ARM 3: symmetrised, but seeds SPLIT a twin pair ===");
    println!("prediction: the split pair breaks; the intact pair still ties");
    for (label, seeds) in [
        ("seed Structure only [9]", vec![9usize]),
        ("seed Circumstances only [6]", vec![6usize]),
        ("seed Priorities only [0]", vec![0usize]),
        ("Manner block minus Circumstances [2,9]", vec![2usize, 9]),
    ] {
        report(label, &relax(&COUPLING, &seeds, true));
    }

    println!("\n=== ARM 4: SCRAMBLED couplings, symmetrised (h3ere2's placebo arm) ===");
    println!("prediction: ties hold anyway — the group average symmetrises ANY input");
    for seed in 0..4u64 {
        let sc = scramble(seed);
        let seeds = blocks()[3].1.clone(); // Manner
        report(&format!("scramble seed {seed}, Manner block"), &relax(&sc, &seeds, true));
    }

    println!("\n=== the arrival orders h3ere2 actually feeds the model ===");
    for (name, seeds) in blocks() {
        let r = relax(&COUPLING, &seeds, true);
        let mut order: Vec<usize> = (0..N).collect();
        order.sort_by_key(|&i| (r.arrival[i].unwrap_or(usize::MAX), i));
        let rest: Vec<String> = order.iter().filter(|i| !seeds.contains(i))
            .map(|&i| format!("{}@{}", KINDS[i], r.arrival[i].map(|v| v.to_string()).unwrap_or("never".into())))
            .collect();
        println!("  {:<9} -> {}", name, rest.join("  "));
    }
}

/// h3ere2's scramble, reproduced (SplitMix64 + Fisher-Yates over the 55 upper entries).
fn scramble(seed: u64) -> [[f64; N]; N] {
    struct Rng(u64);
    impl Rng {
        fn next_u64(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            z ^ (z >> 31)
        }
        fn below(&mut self, n: u64) -> u64 {
            let zone = u64::MAX - (u64::MAX % n);
            loop { let r = self.next_u64(); if r < zone { return r % n; } }
        }
    }
    let mut vals: Vec<f64> = Vec::new();
    for i in 0..N { for j in (i + 1)..N { vals.push(COUPLING[i][j]); } }
    let mut rng = Rng(seed.wrapping_add(0x9E3779B97F4A7C15));
    for k in (1..vals.len()).rev() {
        let s = rng.below((k + 1) as u64) as usize;
        vals.swap(k, s);
    }
    let mut out = [[0.0f64; N]; N];
    let mut it = vals.into_iter();
    for i in 0..N {
        for j in (i + 1)..N {
            let v = it.next().unwrap();
            out[i][j] = v;
            out[j][i] = v;
        }
    }
    out
}
