//! CRYO-H-O — POST-DATA diagnostic. Why did neither P4 nor P4b move the order channel?
//!
//! **This changes no staked criterion and cures no void.** P4 was vacuous by construction
//! (it permutes labels, and `psi6` is a function of the point set). P4b displaced the
//! geometry by half the frame's own mean nearest-neighbour distance and the order still
//! read 0.5956, above the 0.45 ICE bar. Two things explain that and they are opposite:
//!
//!  * the jitter was too SMALL, in which case the order channel is fine and P4b was
//!    under-powered;
//!  * the order statistic is DEGENERATE on a twelve-atom molecular scene — too few atoms
//!    ever close a complete first shell, so `order` is a handful of correlated samples
//!    and no perturbation moves it reliably.
//!
//! The second is the classifier's own documented failure mode (`STAKE_MIN_INTERIOR_ATOMS`
//! exists because of it) and it is what the freeze's pre-staked instrument fence predicts.
//! This probe separates them: one quench rung, the classifier's internals printed, and the
//! jitter swept over two decades. If order survives a jitter of several nearest-neighbour
//! distances, the amplitude was never the issue.
//!
//! ```text
//! cargo run --release -p holon-render --example cryo_order_probe
//! ```

use holon_chem::elements::HYDROGEN;
use holon_chem::pair::generate_pair_table;
use holon_lens::classifier;
use holon_lens::traj::{pair_index, BondSet, Frame, Header, Trajectory};
use holon_render::bank::Host;
use holon_render::sim::{Boundary, Dims, Sim, DEFAULT_SCENE_ATOMS, K_B};
use holon_render::{load_pair_table, TABLE_OK};

const N_ATOMS: usize = 12;
const BOX_W: f64 = 34.6;
const BOX_H: f64 = 20.8;
const T_INIT: f64 = 3000.0;
const TAU: f64 = 2000.0;
const FRAMES: usize = 20000;
const SUBSTEPS: u32 = 64;
const JITTER: f64 = 0.8;
const CURVE_KNOTS: usize = 96;
const CLASSIFY_FROM: usize = 15_000;
const CLASSIFY_STRIDE: usize = 25;

/// The rung P4 and P4b were both run on: the highest `order` in the hydrogen ladder.
const PROBE_T: f64 = 3.0;
const PROBE_SEED: u64 = 0x0000_0000_5341_5422;

/// The jitter sweep, as multiples of the frame's own mean nearest-neighbour distance.
/// P4b used 0.5. Two decades either side of it.
const SWEEP: [f64; 7] = [0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0];

fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

fn gauss(state: &mut u64) -> f64 {
    let u1 = lcg(state).max(1e-12);
    let u2 = lcg(state);
    (-2.0 * u1.ln()).sqrt() * (core::f64::consts::TAU * u2).cos()
}

fn bond_bits(s: &Sim) -> BondSet {
    let n = s.n;
    let mut bits = BondSet::empty();
    for p in s.pairs[..s.pair_count].iter().filter(|p| p.bonded) {
        let (a, b) = if p.i < p.j { (p.i, p.j) } else { (p.j, p.i) };
        bits.insert(pair_index(n, a, b) as u32);
    }
    bits
}

fn mean_nn(f: &Frame, n: usize) -> f64 {
    let mut acc = 0.0;
    for i in 0..n {
        let mut best = f64::INFINITY;
        for j in 0..n {
            if i == j {
                continue;
            }
            let (dx, dy) = (f.pos[i][0] - f.pos[j][0], f.pos[i][1] - f.pos[j][1]);
            best = best.min((dx * dx + dy * dy).sqrt());
        }
        acc += best;
    }
    acc / n as f64
}

fn jitter(t: &Trajectory, frac: f64, seed: u64) -> (Trajectory, f64) {
    let n = t.header.n_atoms;
    let mut st = seed;
    let mut frames = t.frames.clone();
    let mut total = 0.0;
    for f in frames.iter_mut() {
        let amp = frac * mean_nn(f, n);
        total += amp;
        for i in 0..n {
            f.pos[i][0] += amp * (2.0 * lcg(&mut st) - 1.0);
            f.pos[i][1] += amp * (2.0 * lcg(&mut st) - 1.0);
        }
    }
    let nf = frames.len().max(1);
    (Trajectory { header: t.header.clone(), frames }, total / nf as f64)
}

fn main() {
    println!("# CRYO-H-O — POST-DATA order-channel diagnostic. Cures no void, changes no stake.");
    println!("# one rung: T_target {PROBE_T} K, seed {PROBE_SEED:#018x} — the rung P4/P4b ran on");

    let mut s = Box::new(Sim::empty());
    s.boundary = Boundary::Walls;
    s.dims = Dims::Two;
    s.width = BOX_W;
    s.height = BOX_H;
    let pt = generate_pair_table(HYDROGEN, HYDROGEN, CURVE_KNOTS);
    assert_eq!(load_pair_table(&mut s, &pt, Host::Native), TABLE_OK);
    s.trimer = holon_chem::trimer::generate().expect("the H3 table generates");
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../holon-chem/tests/data/s2/s2_water_table.txt"),
    )
    .expect("the committed (O,H,H) table");
    s.water = holon_chem::water::from_text(&src).expect("it parses");
    s.ooh = holon_chem::ooh::generate().expect("the OOH table generates");
    s.ozone = holon_chem::ozone::OzoneTable::empty();
    s.de4_enabled = true;

    // The frozen placement, identical to `cryo_quench::place`.
    let mut st = PROBE_SEED;
    s.reset(N_ATOMS);
    for i in 0..N_ATOMS {
        assert!(s.set_species(i, HYDROGEN));
    }
    s.adopt_table_timescale();
    let mut vs = [(0.0f64, 0.0f64); DEFAULT_SCENE_ATOMS];
    let (mut px, mut py) = (0.0, 0.0);
    for i in 0..N_ATOMS {
        let (col, row) = (i % 4, i / 4);
        let x = BOX_W * (col as f64 + 0.5) / 4.0 + JITTER * (2.0 * lcg(&mut st) - 1.0);
        let y = BOX_H * (row as f64 + 0.5) / 3.0 + JITTER * (2.0 * lcg(&mut st) - 1.0);
        s.set_position(i, x, y);
        let sigma = (K_B * T_INIT / s.atoms[i].mass()).sqrt();
        let (vx, vy) = (sigma * gauss(&mut st), sigma * gauss(&mut st));
        vs[i] = (vx, vy);
        px += s.atoms[i].mass() * vx;
        py += s.atoms[i].mass() * vy;
    }
    let m_tot: f64 = (0..N_ATOMS).map(|i| s.atoms[i].mass()).sum();
    for i in 0..N_ATOMS {
        s.set_velocity(i, vs[i].0 - px / m_tot, vs[i].1 - py / m_tot);
    }
    s.rebase();
    s.thermostat_on = true;
    s.target_temperature = PROBE_T;
    s.thermostat_tau = TAU;

    let mut frames: Vec<Frame> = Vec::new();
    for frame in 0..FRAMES {
        s.step_frame(SUBSTEPS);
        if frame >= CLASSIFY_FROM && (frame - CLASSIFY_FROM) % CLASSIFY_STRIDE == 0 {
            frames.push(Frame {
                index: frame as u64,
                time: s.time,
                temperature: s.temperature(),
                bonds: bond_bits(&s),
                pos: (0..s.n).map(|i| [s.atoms[i].x, s.atoms[i].y, s.atoms[i].z]).collect(),
                vel: (0..s.n).map(|i| [s.atoms[i].vx, s.atoms[i].vy, s.atoms[i].vz]).collect(),
            });
        }
    }
    let header = Header {
        seed: PROBE_SEED,
        n_atoms: s.n,
        dims: 2,
        substeps: SUBSTEPS,
        n_frames: frames.len(),
        dt: s.dt(),
        box_w: BOX_W,
        box_h: BOX_H,
        box_d: s.depth,
        z: (0..s.n).map(|i| s.atoms[i].species.z).collect(),
    };
    let traj = Trajectory { header, frames };
    let base = classifier::classify(&traj);
    let nn = traj.frames.iter().map(|f| mean_nn(f, N_ATOMS)).sum::<f64>() / traj.frames.len() as f64;

    println!("#");
    println!("# THE CLASSIFIER'S INTERNALS ON THE UNPERTURBED SCENE");
    println!(
        "#   verdict {:?}   order {:.4}   mobility {:.4}   free_fraction {:.4}   iceFired {}",
        base.verdict, base.order, base.mobility, base.free_fraction, base.ice_criterion_fired
    );
    println!(
        "#   frames_read {}   interior_atoms {}   interior_samples {}   \
         (STAKE_MIN_INTERIOR_ATOMS = {})",
        base.frames_read, base.interior_atoms, base.interior_samples,
        classifier::STAKE_MIN_INTERIOR_ATOMS
    );
    println!("#   mean nearest-neighbour distance over the window: {nn:.4} bohr");
    println!("#");
    println!("# THE JITTER SWEEP. P4b used frac = 0.50.");
    println!("#   frac    amplitude(bohr)   order    mobility   interior_atoms  verdict");
    for &frac in SWEEP.iter() {
        let (jt, amp) = jitter(&traj, frac, 0x0000_0000_5341_5421);
        let r = classifier::classify(&jt);
        println!(
            "  {:6.2}   {:12.4}   {:7.4}  {:9.4}   {:>12}    {:?}",
            frac, amp, r.order, r.mobility, r.interior_atoms, r.verdict
        );
    }
    println!("#");
    println!("# READ: if `order` stays above {:.2} at frac = 5 — a displacement of several", classifier::STAKE_ORDER);
    println!("#       nearest-neighbour distances — then amplitude was never the issue and the");
    println!("#       order statistic is DEGENERATE on this scene, which is what the freeze's");
    println!("#       pre-staked instrument fence says: a twelve-atom molecular gas has no bulk");
    println!("#       for a bond-orientational order parameter to be about.");
}
