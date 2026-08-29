//! SATURATION-1 gate D1: the quench runner.
//!
//! Sixteen hydrogens in the box, thermostatted down from a hot start, run twice — once
//! with the pairwise force loop alone (the CONTROL, which must reproduce the field
//! droplet) and once with the three-body term ON. The protocol is frozen in
//! `conformance/atomworld/SATURATION1_RESULTS.md` and committed before the MBE3 arm ran;
//! every number it needs is a constant in this file rather than a flag, so a reported run
//! re-runs byte for byte.
//!
//! ```text
//! cargo run --release -p holon-render --example quench -- <arm> [seed ...]
//!   arm = pair | mbe3 | plant3      (plant3 = MBE3 with dE3 zeroed inside a 4-bohr perimeter)
//! ```
//!
//! With no seeds listed it runs the eight staked ones.

use holon_render::sim::{Boundary, Dims, Sim, K_B, M_H, MAX_ATOMS};
use holon_render::{generate_table, generate_trimer_table, TABLE_OK};
use std::time::Instant;

// ------------------------------------------------------------------ THE FROZEN PROTOCOL

/// The eight staked seeds. Written here, not generated: a seed a program chose is a seed
/// nobody staked.
const SEEDS: [u64; 8] = [
    0x0000_0000_5341_5401,
    0x0000_0000_5341_5402,
    0x0000_0000_5341_5403,
    0x0000_0000_5341_5404,
    0x0000_0000_5341_5405,
    0x0000_0000_5341_5406,
    0x0000_0000_5341_5407,
    0x0000_0000_5341_5408,
];

/// The two staked seeds for plant (iii)'s spot check.
const PLANT3_SEEDS: [u64; 2] = [SEEDS[0], SEEDS[1]];

const N_ATOMS: usize = 16;
/// Box, bohr. The sandbox's own default scene, so the quench happens in the box the field
/// report was taken in.
const BOX_W: f64 = 40.0;
const BOX_H: f64 = 24.0;

/// Initial kinetic temperature, kelvin. Hot enough that the opening configuration is a
/// gas rather than a lattice, cold enough that no pair starts up the repulsive wall.
const T_INIT: f64 = 3000.0;
/// Thermostat target, kelvin — the quench's floor.
const T_TARGET: f64 = 300.0;
/// Berendsen coupling time, in atomic time units.
const TAU: f64 = 2000.0;

/// Grain boundaries per run, and substeps per boundary. `dt` is derived from the curve, so
/// the sim time this buys is printed rather than assumed.
const FRAMES: usize = 20000;
const SUBSTEPS: u32 = 64;

/// Jitter on the opening lattice, bohr. Keeps every opening separation outside the
/// repulsive wall while still making eight seeds eight different scenes.
const JITTER: f64 = 0.8;

/// Plant (iii): the perimeter below which the table is zeroed, bohr. The prereg's number.
const PLANT3_PERIMETER: f64 = 4.0;

/// POST-HOC DIAGNOSTIC, and labelled as one. Added after the staked plant (iii) returned
/// cluster readings IDENTICAL to the MBE3 arm on both of its seeds — which is what a plant
/// aimed at a sector the trajectory never enters looks like, the case M-PLANT-SECTOR names
/// as a VOID rather than a miss. The `arm = plant3b` run zeroes a perimeter the dynamics
/// demonstrably DOES visit (see the `min perimeter` column every arm now prints), so the
/// instrument's ability to fire is shown rather than assumed. It is not the staked plant
/// and is never reported as one.
const PLANT3B_PERIMETER: f64 = 9.0;

/// POST-HOC DIAGNOSTIC, the far-field plant with the sign the measurement says it wants.
/// `arm = plant3c` keeps the compact core and zeroes everything ABOVE this perimeter —
/// the shell where a third atom meets an existing bond, which is where the MBE3 dynamics
/// actually lives (its closest domain triple over 40,000 boundaries has perimeter 8.58
/// bohr, and it never once enters the staked plant's 4). Not the staked plant.
const PLANT3C_PERIMETER: f64 = 6.0;

// ------------------------------------------------------------------ deterministic setup

fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

/// Box-Muller from the same stream, so one seed determines the whole scene.
fn gauss(state: &mut u64) -> f64 {
    let u1 = lcg(state).max(1e-12);
    let u2 = lcg(state);
    (-2.0 * u1.ln()).sqrt() * (core::f64::consts::TAU * u2).cos()
}

/// The opening scene: a 4x4 lattice with a seeded jitter, and velocities drawn from a
/// Maxwellian at `T_INIT` with the net momentum removed.
fn place(s: &mut Sim, seed: u64) {
    let mut st = seed;
    s.reset(N_ATOMS);
    let mut vs = [(0.0f64, 0.0f64); MAX_ATOMS];
    let mut px = 0.0;
    let mut py = 0.0;
    let sigma = (K_B * T_INIT / M_H).sqrt();
    for i in 0..N_ATOMS {
        let (col, row) = (i % 4, i / 4);
        let x = BOX_W * (col as f64 + 0.5) / 4.0 + JITTER * (2.0 * lcg(&mut st) - 1.0);
        let y = BOX_H * (row as f64 + 0.5) / 4.0 + JITTER * (2.0 * lcg(&mut st) - 1.0);
        s.set_position(i, x, y);
        let (vx, vy) = (sigma * gauss(&mut st), sigma * gauss(&mut st));
        vs[i] = (vx, vy);
        px += vx;
        py += vy;
    }
    // Remove the net drift: the box has walls, so a drifting scene would just heat itself
    // against them, and the quench would be measuring the walls.
    for i in 0..N_ATOMS {
        s.set_velocity(
            i,
            vs[i].0 - px / N_ATOMS as f64,
            vs[i].1 - py / N_ATOMS as f64,
        );
    }
    s.rebase();
    s.thermostat_on = true;
    s.target_temperature = T_TARGET;
    s.thermostat_tau = TAU;
}

/// THE MEASUREMENT RULE, frozen with the protocol.
///
/// Components of the bonded-pair graph, from `Sim::cluster_sizes` — one union-find over
/// one edge set, the same one the headline `cluster_count` reads. A component of one atom
/// is a FREE ATOM, not a cluster; the modal cluster size is the mode over components of
/// size two or more, ties broken toward the smaller size. Returned as
/// `(largest, modal, n_clusters, n_free, histogram)`.
fn reading(s: &Sim) -> (usize, usize, usize, usize, [usize; MAX_ATOMS + 1]) {
    let sizes = s.cluster_sizes();
    let mut hist = [0usize; MAX_ATOMS + 1];
    for &sz in sizes[..s.n].iter() {
        if sz >= 1 {
            hist[sz] += 1;
        }
    }
    let largest = (2..=MAX_ATOMS).rev().find(|&k| hist[k] > 0).unwrap_or(0);
    let modal = (2..=MAX_ATOMS)
        .max_by_key(|&k| (hist[k], usize::MAX - k))
        .filter(|&k| hist[k] > 0)
        .unwrap_or(0);
    let n_clusters: usize = hist[2..].iter().sum();
    (largest, modal, n_clusters, hist[1], hist)
}

// ------------------------------------------------------------------ the run

struct Outcome {
    seed: u64,
    largest: usize,
    modal: usize,
    clusters: usize,
    free: usize,
    hist: [usize; MAX_ATOMS + 1],
    drift: f64,
    bound: f64,
    momentum: f64,
    momentum_bound: f64,
    temperature: f64,
    e_three: f64,
    /// Smallest triangle PERIMETER any triple reached during the run, bohr, over triples
    /// inside the table's domain. A pure read of the trajectory: it changes nothing.
    min_perimeter: f64,
    /// The separations INSIDE the largest cluster, sorted, bohr. A cluster is a statement
    /// about boundness, not about closure: two H2 molecules whose cross pair happens to
    /// read `bonded` are ONE component of four, and the only way to tell that from a
    /// tetramer is to look at the distances. Printed, never scored.
    largest_bonds: Vec<f64>,
    /// Grain boundaries at which some triple was inside the staked plant's 4-bohr
    /// perimeter, out of `FRAMES`.
    frames_inside_plant: usize,
    seconds: f64,
}

/// The smallest triangle perimeter among triples whose MIDDLE side is inside the table's
/// domain — i.e. among the triples the three-body term is actually being read for.
fn largest_cluster_bonds(s: &Sim) -> Vec<f64> {
    let sizes = s.cluster_sizes();
    let mut root = usize::MAX;
    let mut best = 1usize;
    for (i, &sz) in sizes[..s.n].iter().enumerate() {
        if sz > best {
            best = sz;
            root = i;
        }
    }
    if root == usize::MAX {
        return Vec::new();
    }
    // Re-derive membership from the same edge set the sizes came from.
    let mut member = vec![false; s.n];
    let mut changed = true;
    member[root] = true;
    while changed {
        changed = false;
        for p in s.pairs[..s.pair_count].iter().filter(|p| p.bonded) {
            if member[p.i] != member[p.j] {
                member[p.i] = true;
                member[p.j] = true;
                changed = true;
            }
        }
    }
    let mut out = Vec::new();
    for i in 0..s.n {
        for j in (i + 1)..s.n {
            if member[i] && member[j] {
                let (a, b) = (&s.atoms[i], &s.atoms[j]);
                out.push(((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt());
            }
        }
    }
    out.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    out
}

fn min_domain_perimeter(s: &Sim, r_cut: f64) -> f64 {
    let mut best = f64::INFINITY;
    for i in 0..s.n {
        for j in (i + 1)..s.n {
            for k in (j + 1)..s.n {
                let d = |a: usize, b: usize| {
                    let (p, q) = (&s.atoms[a], &s.atoms[b]);
                    ((p.x - q.x).powi(2) + (p.y - q.y).powi(2) + (p.z - q.z).powi(2)).sqrt()
                };
                let mut sides = [d(i, j), d(i, k), d(j, k)];
                sides.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
                if sides[1] > r_cut {
                    continue;
                }
                let p = sides[0] + sides[1] + sides[2];
                if p < best {
                    best = p;
                }
            }
        }
    }
    best
}

fn run(arm: &str, seed: u64, template: &Sim) -> Outcome {
    let mut s = Box::new(Sim::empty());
    s.dims = Dims::Two;
    s.boundary = Boundary::Walls;
    s.width = BOX_W;
    s.height = BOX_H;
    // The pair curve is re-solved per seed (30 ms) rather than cloned, so this runner
    // needs no new trait on the shared interpolator; the three-body table, which costs
    // seconds, is copied.
    assert_eq!(
        generate_table(&mut s, 0.3, 10.0, 492),
        TABLE_OK,
        "the pair curve did not generate"
    );
    s.trimer = template.trimer.clone();
    if arm == "pair" {
        s.trimer.loaded = false;
    }
    place(&mut s, seed);

    let r_cut = holon_chem::trimer::R_HI;
    let t0 = Instant::now();
    let mut min_perimeter = f64::INFINITY;
    let mut frames_inside_plant = 0usize;
    for _ in 0..FRAMES {
        s.step_frame(SUBSTEPS);
        let p = min_domain_perimeter(&s, r_cut);
        if p < min_perimeter {
            min_perimeter = p;
        }
        if p < PLANT3_PERIMETER {
            frames_inside_plant += 1;
        }
    }
    let seconds = t0.elapsed().as_secs_f64();
    let (largest, modal, clusters, free, hist) = reading(&s);
    Outcome {
        seed,
        largest,
        modal,
        clusters,
        free,
        hist,
        drift: s.drift_peak,
        bound: s.drift_bound(),
        momentum: s.momentum_residual_peak,
        momentum_bound: s.momentum_bound(),
        temperature: s.temperature(),
        e_three: s.e_three,
        min_perimeter,
        largest_bonds: largest_cluster_bonds(&s),
        frames_inside_plant,
        seconds,
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let arm = args.next().unwrap_or_else(|| "pair".to_string());
    let seeds: Vec<u64> = {
        let listed: Vec<u64> = args.filter_map(|a| a.parse().ok()).collect();
        if !listed.is_empty() {
            listed
        } else if arm == "plant3" {
            PLANT3_SEEDS.to_vec()
        } else {
            SEEDS.to_vec()
        }
    };

    let mut template = Box::new(Sim::empty());
    template.dims = Dims::Two;
    let t0 = Instant::now();
    assert_eq!(
        generate_table(&mut template, 0.3, 10.0, 492),
        TABLE_OK,
        "the pair curve did not generate"
    );
    let pair_ms = t0.elapsed().as_secs_f64() * 1e3;
    let t1 = Instant::now();
    assert_eq!(
        generate_trimer_table(&mut template),
        1,
        "the three-body table did not generate"
    );
    let trimer_s = t1.elapsed().as_secs_f64();
    if arm == "plant3" {
        template.trimer.zero_inside_perimeter(PLANT3_PERIMETER);
    }
    if arm == "plant3b" {
        template.trimer.zero_inside_perimeter(PLANT3B_PERIMETER);
    }
    if arm == "plant3c" {
        template.trimer.zero_outside_perimeter(PLANT3C_PERIMETER);
    }

    println!("# SATURATION-1 D1 quench — arm = {arm}");
    println!(
        "# tables: pair curve {} knots in {pair_ms:.0} ms; trimer {} nodes ({} solves) in \
         {trimer_s:.2} s; curvature envelope {:.4} Ha/bohr^2",
        template.table.knots(),
        template.trimer.meta.n_nodes,
        template.trimer.meta.solves,
        template.trimer.curvature_envelope
    );
    println!(
        "# protocol: N = {N_ATOMS}, box {BOX_W} x {BOX_H} bohr, T_init {T_INIT} K, \
         T_target {T_TARGET} K, tau {TAU}, {FRAMES} frames x {SUBSTEPS} substeps, \
         dt = {:.4} a.u. -> {:.1} fs of sim time",
        template.dt(),
        FRAMES as f64 * SUBSTEPS as f64 * template.dt() * 0.024188843265857
    );
    if arm == "plant3" {
        println!("# PLANT (iii): dE3 zeroed inside a {PLANT3_PERIMETER}-bohr perimeter");
    }
    if arm == "plant3c" {
        println!(
            "# PLANT (iii-c), POST-HOC DIAGNOSTIC, not the staked plant: dE3 zeroed \
             OUTSIDE a {PLANT3C_PERIMETER}-bohr perimeter"
        );
    }
    if arm == "plant3b" {
        println!(
            "# PLANT (iii-b), POST-HOC DIAGNOSTIC, not the staked plant: dE3 zeroed inside \
             a {PLANT3B_PERIMETER}-bohr perimeter"
        );
    }
    println!(
        "seed              largest modal clusters free  hist(2..)         T(K)     \
         E_three     drift/bound  dP/bound  minPerim  inPlant       s"
    );

    let mut outcomes = Vec::new();
    for &seed in &seeds {
        let o = run(&arm, seed, &template);
        let hist: Vec<String> = (2..=MAX_ATOMS)
            .filter(|&k| o.hist[k] > 0)
            .map(|k| format!("{}x{}", o.hist[k], k))
            .collect();
        println!(
            "{:#018x} {:>7} {:>5} {:>8} {:>4}  {:<16} {:>7.1} {:>+10.5} {:>10.4} {:>9.4} {:>9.3} {:>8} {:>7.1}",
            o.seed,
            o.largest,
            o.modal,
            o.clusters,
            o.free,
            hist.join(" "),
            o.temperature,
            o.e_three,
            o.drift / o.bound,
            o.momentum / o.momentum_bound,
            o.min_perimeter,
            o.frames_inside_plant,
            o.seconds
        );
        if o.largest >= 3 {
            let d: Vec<String> = o.largest_bonds.iter().map(|x| format!("{x:.2}")).collect();
            println!("    largest cluster's separations, bohr: {}", d.join(" "));
        }
        outcomes.push(o);
    }

    // The two branch criteria, evaluated but never adjusted here: the arm decides which
    // one is the gate and the results doc records the branch.
    let control_met = outcomes.iter().filter(|o| o.largest >= 8).count();
    let branch_a = outcomes
        .iter()
        .filter(|o| o.modal == 2 && o.largest <= 4)
        .count();
    println!(
        "# CONTROL criterion (largest >= 8): {control_met}/{} seeds",
        outcomes.len()
    );
    println!(
        "# BRANCH (a) criterion (modal == 2 and largest <= 4): {branch_a}/{} seeds",
        outcomes.len()
    );
    let worst_e = outcomes
        .iter()
        .map(|o| o.drift / o.bound)
        .fold(0.0f64, f64::max);
    let worst_p = outcomes
        .iter()
        .map(|o| o.momentum / o.momentum_bound)
        .fold(0.0f64, f64::max);
    println!("# worst energy drift / bound = {worst_e:.4}; worst momentum residual / bound = {worst_p:.4}");
    let closest = outcomes
        .iter()
        .map(|o| o.min_perimeter)
        .fold(f64::INFINITY, f64::min);
    let inside: usize = outcomes.iter().map(|o| o.frames_inside_plant).sum();
    println!(
        "# closest approach of any domain triple: perimeter {closest:.3} bohr; \
         boundaries with a triple inside the staked plant's {PLANT3_PERIMETER} bohr: \
         {inside} of {}",
        outcomes.len() * FRAMES
    );
}
