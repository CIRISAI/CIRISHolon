//! CRYO-H-O — the quench ladder. ARM 1 gate G4, ARM 2 gates G7 and G8, plant P4.
//!
//! Frozen by `conformance/atomworld/CRYO_HO_PREREG.md` (2026-09-02, commit fc7b6a0).
//!
//! **This is SATURATION-2's `waterquench` protocol with exactly one thing changed:
//! `T_target` is a ladder instead of a constant.** Every other number below — the atom
//! count, the box, the lattice, the jitter, the opening temperature, the coupling time,
//! the frame count, the substeps, the knot count, the seeds — is copied from
//! `examples/waterquench.rs` unchanged, so a rung at 300 K is directly comparable to
//! SATURATION-2's banked reading and the ladder is the only variable.
//!
//! It is a separate file rather than a flag for the reason `waterquench_traj.rs` gives:
//! `tests/protocol_identity.rs` holds `waterquench.rs` and `waterquench_traj.rs` to byte
//! equality on their frozen block, and a flag added to either would edit a gated file.
//! Nothing here may be presented as SATURATION-2's own output; it is a new campaign
//! reading the same protocol at new temperatures.
//!
//! ```text
//! cargo run --release -p holon-render --example cryo_quench -- hydrogen|oxygen
//! ```

use holon_chem::elements::{Species, HYDROGEN, OXYGEN};
use holon_chem::pair::generate_pair_table;
use holon_lens::classifier::{self, Phase, Verdict};
use holon_lens::traj::{pair_index, Frame, Header, Trajectory};
use holon_render::bank::Host;
use holon_render::sim::{Boundary, Dims, Sim, DEFAULT_SCENE_ATOMS, K_B};
use holon_render::{load_pair_table, TABLE_OK};
use std::time::Instant;

// ============================== SATURATION-2's PROTOCOL, COPIED, ONE FIELD FREED

const N_ATOMS: usize = 12;
const BOX_W: f64 = 34.6;
const BOX_H: f64 = 20.8;
const T_INIT: f64 = 3000.0;
const TAU: f64 = 2000.0;
const FRAMES: usize = 20000;
const SUBSTEPS: u32 = 64;
const JITTER: f64 = 0.8;
const CURVE_KNOTS: usize = 96;

/// THE ONE NEW THING. A logarithmic ladder in steps of ~3.16, chosen as a log ladder and
/// not aimed at any physical transition temperature. 300 K is SATURATION-2's own rung and
/// is the tie to the banked record.
const T_LADDER: [f64; 5] = [300.0, 100.0, 30.0, 10.0, 3.0];

/// The first three of SATURATION-2's eight staked seeds, in its order.
const SEEDS: [u64; 3] = [
    0x0000_0000_5341_5421,
    0x0000_0000_5341_5422,
    0x0000_0000_5341_5423,
];

/// The classifier reads the FINAL QUARTER of the run, so it reads the quenched state and
/// not the quench. Staked in the freeze.
const CLASSIFY_FROM: usize = 15_000;
const CLASSIFY_STRIDE: usize = 25;

/// G4's bar: at most this fraction of atoms may sit in components larger than 2. One H4
/// in twelve atoms is 4/12 = 1/3 of the atoms; the freeze's bar is 1/6, which admits the
/// banked 300 K artifact rate over three seeds without admitting a second H4 in one seed.
const G4_AGG_BAR: f64 = 1.0 / 6.0;
const G4_LARGEST_BAR: usize = 4;
/// G7's bar: the oxygen aggregate.
const G7_LARGEST_BAR: usize = 10;
/// G8's tolerance: `order` may fall by at most this between adjacent rungs as T falls.
const G8_TOL: f64 = 0.05;

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

#[derive(Clone, Copy, PartialEq)]
enum Arm {
    Hydrogen,
    Oxygen,
}

impl Arm {
    fn parse(s: &str) -> Option<Arm> {
        match s {
            "hydrogen" => Some(Arm::Hydrogen),
            "oxygen" => Some(Arm::Oxygen),
            _ => None,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Arm::Hydrogen => "ARM 1 — hydrogen (12 H)",
            Arm::Oxygen => "ARM 2 — oxygen (12 O)",
        }
    }
    fn species(self) -> Species {
        match self {
            Arm::Hydrogen => HYDROGEN,
            Arm::Oxygen => OXYGEN,
        }
    }
}

/// `waterquench::place`, with `T_target` taken as an argument.
fn place(s: &mut Sim, arm: Arm, seed: u64, t_target: f64) {
    let mut st = seed;
    s.reset(N_ATOMS);
    for i in 0..N_ATOMS {
        assert!(s.set_species(i, arm.species()), "species {i} did not register");
    }
    s.adopt_table_timescale();

    let mut vs = [(0.0f64, 0.0f64); DEFAULT_SCENE_ATOMS];
    let (mut px, mut py) = (0.0, 0.0);
    #[allow(clippy::needless_range_loop)]
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
    #[allow(clippy::needless_range_loop)]
    for i in 0..N_ATOMS {
        s.set_velocity(i, vs[i].0 - px / m_tot, vs[i].1 - py / m_tot);
    }
    s.rebase();
    s.thermostat_on = true;
    s.target_temperature = t_target;
    s.thermostat_tau = TAU;
}

fn bond_bits(s: &Sim) -> u128 {
    let n = s.n;
    let mut bits = 0u128;
    for p in s.pairs[..s.pair_count].iter().filter(|p| p.bonded) {
        let (a, b) = if p.i < p.j { (p.i, p.j) } else { (p.j, p.i) };
        bits |= 1u128 << pair_index(n, a, b);
    }
    bits
}

struct Rung {
    t_target: f64,
    seed: u64,
    /// Component sizes at the final grain boundary, sorted descending.
    sizes: Vec<usize>,
    largest: usize,
    /// Fraction of atoms in components strictly larger than 2 — G4's criterion.
    agg_fraction: f64,
    free_atoms: usize,
    fenced: u64,
    drift_ratio: f64,
    p_ratio: f64,
    temperature: f64,
    dt: f64,
    order: f64,
    mobility: f64,
    free_fraction: f64,
    ice_fired: bool,
    verdict: String,
    traj: Trajectory,
}

fn component_sizes(s: &Sim) -> Vec<usize> {
    let sizes = s.cluster_sizes();
    let mut out: Vec<usize> = (0..s.n).map(|i| sizes[i]).filter(|&k| k > 0).collect();
    out.sort_unstable_by(|a, b| b.cmp(a));
    out
}

fn run(s: &mut Sim, arm: Arm, seed: u64, t_target: f64) -> Rung {
    let t0 = Instant::now();
    place(s, arm, seed, t_target);
    assert!(s.pairs_ready(), "the bank is missing a curve this arm needs");

    let header = Header {
        seed,
        n_atoms: s.n,
        dims: 2,
        substeps: SUBSTEPS,
        n_frames: 0,
        dt: s.dt(),
        box_w: BOX_W,
        box_h: BOX_H,
        box_d: s.depth,
        z: (0..s.n).map(|i| s.atoms[i].species.z).collect(),
    };
    let mut frames: Vec<Frame> = Vec::new();

    for frame in 0..FRAMES {
        s.step_frame(SUBSTEPS);
        if frame >= CLASSIFY_FROM && (frame - CLASSIFY_FROM) % CLASSIFY_STRIDE == 0 {
            frames.push(Frame {
                index: frame as u64,
                time: s.time,
                temperature: s.temperature(),
                bonded: bond_bits(s),
                pos: (0..s.n).map(|i| [s.atoms[i].x, s.atoms[i].y, s.atoms[i].z]).collect(),
                vel: (0..s.n).map(|i| [s.atoms[i].vx, s.atoms[i].vy, s.atoms[i].vz]).collect(),
            });
        }
    }

    let mut header = header;
    header.n_frames = frames.len();
    let traj = Trajectory { header, frames };
    let rep = classifier::classify(&traj);

    let sizes = component_sizes(s);
    let largest = sizes.first().copied().unwrap_or(0);
    let in_big: usize = sizes.iter().filter(|&&k| k > 2).sum();
    let rung = Rung {
        t_target,
        seed,
        agg_fraction: in_big as f64 / N_ATOMS as f64,
        free_atoms: sizes.iter().filter(|&&k| k == 1).count(),
        largest,
        sizes,
        fenced: s.fence_untabulated,
        drift_ratio: s.drift_peak / s.drift_bound(),
        p_ratio: s.momentum_residual_peak / s.momentum_bound(),
        temperature: s.temperature(),
        dt: s.dt(),
        order: rep.order,
        mobility: rep.mobility,
        free_fraction: rep.free_fraction,
        ice_fired: rep.ice_criterion_fired,
        verdict: match &rep.verdict {
            Verdict::Phase(Phase::Vapor) => "VAPOR".to_string(),
            Verdict::Phase(Phase::Liquid) => "LIQUID".to_string(),
            Verdict::Phase(Phase::Ice) => "ICE".to_string(),
            Verdict::Refused { gate, reason } => format!("REFUSED[{gate}: {reason}]"),
        },
        traj,
    };
    println!(
        "  T {:>5.0} K  seed {:#018x}  dt {:.4}  sizes {:?}  largest {:>2}  agg {:.3}  free {}  \
         fence {:>3}  drift/b {:.2e}  |p|/b {:.2e}  T_end {:>4.0} K  | order {:.4}  mob {:.4}  \
         freefrac {:.3}  iceFired {}  {}  [{:.0} s]",
        rung.t_target, rung.seed, rung.dt, rung.sizes, rung.largest, rung.agg_fraction,
        rung.free_atoms, rung.fenced, rung.drift_ratio, rung.p_ratio, rung.temperature,
        rung.order, rung.mobility, rung.free_fraction, rung.ice_fired, rung.verdict,
        t0.elapsed().as_secs_f64()
    );
    rung
}

// ------------------------------------------------------------------------ plant P4

/// The scrambled-scene plant. Permutes each frame's positions ACROSS ATOMS, independently
/// per frame, leaving the bonded bitset and each frame's position multiset untouched. The
/// bond graph therefore says exactly what it said before and only the GEOMETRY is
/// destroyed, so a firing is attributable to the order channel and to nothing else.
fn scramble(t: &Trajectory, seed: u64) -> (Trajectory, usize, usize) {
    let n = t.header.n_atoms;
    let mut st = seed;
    let mut frames = t.frames.clone();
    let mut moved = 0usize;
    for f in frames.iter_mut() {
        let before = f.pos.clone();
        // Fisher–Yates from the same stream that seeds every other random thing here.
        for i in (1..n).rev() {
            let j = (lcg(&mut st) * (i + 1) as f64) as usize;
            f.pos.swap(i, j.min(i));
        }
        moved += (0..n).filter(|&k| f.pos[k] != before[k]).count();
    }
    let nf = frames.len();
    (
        Trajectory { header: t.header.clone(), frames },
        nf,
        moved,
    )
}

/// P4b's jitter, as a fraction of the frame's own mean nearest-neighbour distance.
const P4B_JITTER_FRAC: f64 = 0.5;

/// P4b, added POST-DATA after P4 was found VACUOUS, and it does NOT cure P4's void.
///
/// P4 permutes position LABELS across atoms. `psi6` is computed from each atom's nearest
/// neighbours and is therefore a function of the POINT SET, which a relabelling does not
/// touch — so P4's carrier lies exactly in the null space of the statistic it was meant to
/// move, and it could not have fired on any scene whatever. That is M-PLANT-OBS in its
/// pure form: the plant was not re-derived for this instrument, and the freeze's own
/// wording ("leaving the per-frame position multiset untouched") is where the vacuity was
/// written in.
///
/// This one displaces every atom by an independent uniform vector of up to
/// `P4B_JITTER_FRAC` times the frame's mean nearest-neighbour distance. It changes the
/// geometry, which is what the order parameter reads. Its only job is to establish that
/// the order channel is MOVABLE — that the instrument is alive — and a post-data plant can
/// never discharge a pre-registered one.
fn jitter_scene(t: &Trajectory, seed: u64) -> (Trajectory, usize, f64) {
    let n = t.header.n_atoms;
    let mut st = seed;
    let mut frames = t.frames.clone();
    let mut moved = 0usize;
    let mut total_amp = 0.0f64;
    for f in frames.iter_mut() {
        let mut nn_sum = 0.0f64;
        for i in 0..n {
            let mut best = f64::INFINITY;
            for j in 0..n {
                if i == j {
                    continue;
                }
                let (dx, dy) = (f.pos[i][0] - f.pos[j][0], f.pos[i][1] - f.pos[j][1]);
                best = best.min((dx * dx + dy * dy).sqrt());
            }
            nn_sum += best;
        }
        let amp = P4B_JITTER_FRAC * nn_sum / n as f64;
        total_amp += amp;
        for i in 0..n {
            f.pos[i][0] += amp * (2.0 * lcg(&mut st) - 1.0);
            f.pos[i][1] += amp * (2.0 * lcg(&mut st) - 1.0);
            moved += 1;
        }
    }
    let nf = frames.len().max(1);
    (Trajectory { header: t.header.clone(), frames }, moved, total_amp / nf as f64)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arm = args
        .first()
        .and_then(|a| Arm::parse(a))
        .expect("arm = hydrogen | oxygen");

    println!("# CRYO-H-O — the quench ladder: {}", arm.label());
    println!("# prereg conformance/atomworld/CRYO_HO_PREREG.md, frozen fc7b6a0");
    println!("# STANDING FENCES: 2D scene | classical nuclei (no NQE) | STO-3G minimal basis");
    println!(
        "# protocol COPIED from waterquench.rs: {N_ATOMS} atoms, {BOX_W} x {BOX_H} bohr, \
         {FRAMES} x {SUBSTEPS}, T_init {T_INIT} K, tau {TAU}, jitter {JITTER}, {CURVE_KNOTS} knots"
    );
    println!("# THE ONE CHANGE: T_target ladder {T_LADDER:?} K, seeds {SEEDS:#018x?}");
    println!(
        "# classifier window: grain boundaries {}..{} stride {} -> {} frames",
        CLASSIFY_FROM,
        FRAMES,
        CLASSIFY_STRIDE,
        (FRAMES - CLASSIFY_FROM).div_ceil(CLASSIFY_STRIDE)
    );

    let t0 = Instant::now();
    let mut base = Box::new(Sim::empty());
    base.boundary = Boundary::Walls;
    base.dims = Dims::Two;
    base.width = BOX_W;
    base.height = BOX_H;
    let sp = arm.species();
    let pt = generate_pair_table(sp, sp, CURVE_KNOTS);
    let well = match pt.meta.well {
        Some(w) => format!("R_e = {:.4} bohr, D_e = {:.6} Ha", w.r_e, w.d_e),
        None => "NO WELL".to_string(),
    };
    assert_eq!(load_pair_table(&mut base, &pt, Host::Native), TABLE_OK);

    // THE TABLE SET, copied from `waterquench.rs`'s own `main` and not abbreviated.
    //
    // The first run of this file omitted every line below, and the hydrogen arm read a
    // seven-to-twelve-atom aggregate at EVERY rung including 300 K — against
    // SATURATION-2's banked "44 x H2, 2 x H4, largest <= 4 in 8/8" on the SAME seeds and
    // the same protocol. With no H3 surface the scene is MBE2-only, the pair curve is a
    // bonding curve that knows nothing of valence saturation, and twelve hydrogens
    // over-coordinate exactly as twelve oxygens do. The curve-identity check (V1) passed
    // throughout, because the curve was never the thing that was wrong: a check that
    // establishes the CURVE is the banked one establishes nothing about the PROTOCOL.
    base.trimer = holon_chem::trimer::generate().expect("the H3 table generates");
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../holon-chem/tests/data/s2/s2_water_table.txt"),
    )
    .expect("the committed (O,H,H) table");
    base.water = holon_chem::water::from_text(&src).expect("it parses");
    base.ooh = holon_chem::ooh::generate().expect("the OOH table generates");
    assert!(base.ooh.loaded, "OOH table must be loaded and ready");
    base.ozone = holon_chem::ozone::OzoneTable::empty();
    base.de4_enabled = true;
    println!(
        "# table set (copied from waterquench.rs): H3 generated, (O,H,H) committed artifact, \
         (O,O,H) generated, (O,O,O) FENCED empty, de4_enabled = true"
    );
    println!(
        "# V1 curve identity: {}-{} {CURVE_KNOTS} knots, {}, exit {}, budget {}, worst_residual {:.3e}, \
         n_det {}, n_basis {}, device {:?}  [{:.1} s]",
        sp.symbol, sp.symbol, well, pt.meta.exit.label(), pt.meta.solver_budget,
        pt.meta.worst_residual, pt.meta.n_det, pt.meta.n_basis, pt.meta.device,
        t0.elapsed().as_secs_f64()
    );
    println!(
        "# V1 BANKED for comparison: H-H R_e 1.3887 D_e 0.204142 | O-O R_e 2.4421 D_e 0.147621"
    );

    let mut rungs: Vec<Rung> = Vec::new();
    for &t_target in T_LADDER.iter() {
        println!("#");
        for &seed in SEEDS.iter() {
            rungs.push(run(&mut base, arm, seed, t_target));
        }
    }

    // ------------------------------------------------------ V1 leg 2: PROTOCOL identity
    //
    // Added after V1's first leg (the curve) passed on an instrument that was still not
    // the banked protocol. The 300 K rung exists to be compared, so it is compared
    // mechanically rather than by eye.
    println!("\n# ================================ V1 leg 2 — protocol identity at the 300 K rung");
    let at300: Vec<&Rung> = rungs.iter().filter(|r| r.t_target == 300.0).collect();
    let banked_ok = match arm {
        Arm::Hydrogen => at300.iter().all(|r| r.largest <= 4 && r.free_atoms == 0),
        Arm::Oxygen => at300.iter().all(|r| r.largest == 12 && r.free_atoms == 0 && r.fenced == 220),
    };
    println!(
        "# banked at 300 K: hydrogen -> largest <= 4, zero free H | oxygen -> O12, zero free O, fence 220"
    );
    println!(
        "# measured at 300 K: largest {:?}, free {:?}, fence {:?}",
        at300.iter().map(|r| r.largest).collect::<Vec<_>>(),
        at300.iter().map(|r| r.free_atoms).collect::<Vec<_>>(),
        at300.iter().map(|r| r.fenced).collect::<Vec<_>>()
    );
    println!(
        "# V1 leg 2: {}",
        if banked_ok {
            "PASS — this IS the banked protocol; the ladder is the only variable"
        } else {
            "FAIL — the campaign is VOID, this is not the banked instrument"
        }
    );

    // ------------------------------------------------------------------ V2 and V3
    println!("\n# ================================ V2 / V3 — void conditions");
    let v2: Vec<&Rung> = rungs.iter().filter(|r| r.drift_ratio > 1.0 || r.p_ratio > 1.0).collect();
    println!(
        "# V2 ledger: {} of {} rung-seeds VOID (worst drift/bound {:.3e}, worst |p|/bound {:.3e})",
        v2.len(),
        rungs.len(),
        rungs.iter().map(|r| r.drift_ratio).fold(0.0, f64::max),
        rungs.iter().map(|r| r.p_ratio).fold(0.0, f64::max)
    );
    let v3 = rungs.iter().filter(|r| r.verdict.starts_with("REFUSED")).count();
    println!("# V3 classifier refusals: {v3} of {}", rungs.len());

    // ------------------------------------------------------------------ G4 / G7
    let live: Vec<&Rung> = rungs.iter().filter(|r| r.drift_ratio <= 1.0 && r.p_ratio <= 1.0).collect();
    match arm {
        Arm::Hydrogen => {
            println!("\n# ================================ G4 — does any rung condense?");
            println!(
                "# staked: aggregated fraction <= {:.4} AND largest <= {} in every rung-seed",
                G4_AGG_BAR, G4_LARGEST_BAR
            );
            let bad: Vec<&&Rung> = live
                .iter()
                .filter(|r| r.agg_fraction > G4_AGG_BAR + 1e-12 || r.largest > G4_LARGEST_BAR)
                .collect();
            for r in bad.iter() {
                println!(
                    "#   BREACH  T {:>5.0} K seed {:#018x}: agg {:.3}, largest {}",
                    r.t_target, r.seed, r.agg_fraction, r.largest
                );
            }
            println!(
                "# G4: {}  ({} of {} live rung-seeds breach)",
                if bad.is_empty() { "HOLDS — no rung condenses" } else { "KILLED — a rung condensed" },
                bad.len(),
                live.len()
            );
            for &t in T_LADDER.iter() {
                let at: Vec<&&Rung> = live.iter().filter(|r| r.t_target == t).collect();
                let modal = at.iter().map(|r| r.largest).max().unwrap_or(0);
                println!(
                    "#   T {:>5.0} K: largest across seeds {:?}, agg fraction {:?}",
                    t,
                    at.iter().map(|r| r.largest).collect::<Vec<_>>(),
                    at.iter().map(|r| format!("{:.3}", r.agg_fraction)).collect::<Vec<_>>()
                );
                let _ = modal;
            }
        }
        Arm::Oxygen => {
            println!("\n# ================================ G7 — one aggregate at every rung?");
            println!(
                "# staked: largest >= {} in >= 2 of 3 seeds at every rung; fence exactly 220; zero free O",
                G7_LARGEST_BAR
            );
            let mut holds = true;
            for &t in T_LADDER.iter() {
                let at: Vec<&&Rung> = live.iter().filter(|r| r.t_target == t).collect();
                let n_big = at.iter().filter(|r| r.largest >= G7_LARGEST_BAR).count();
                let fence_ok = at.iter().all(|r| r.fenced == 220);
                let free_ok = at.iter().all(|r| r.free_atoms == 0);
                if at.len() == 3 && (n_big < 2 || !fence_ok || !free_ok) {
                    holds = false;
                }
                println!(
                    "#   T {:>5.0} K: largest {:?}  ({}/{} at or above {}), fence {:?}, free O {:?}",
                    t,
                    at.iter().map(|r| r.largest).collect::<Vec<_>>(),
                    n_big,
                    at.len(),
                    G7_LARGEST_BAR,
                    at.iter().map(|r| r.fenced).collect::<Vec<_>>(),
                    at.iter().map(|r| r.free_atoms).collect::<Vec<_>>()
                );
            }
            println!(
                "# G7: {}",
                if holds { "HOLDS — one aggregate, MBE2-only, at every rung" } else { "KILLED" }
            );
        }
    }

    // ------------------------------------------------------------------ G8
    println!("\n# ================================ G8 — is `order` monotone as T falls?");
    println!("# staked: order non-decreasing as T_target falls, tolerance {G8_TOL} per step");
    let mut means: Vec<(f64, f64, usize)> = Vec::new();
    for &t in T_LADDER.iter() {
        let at: Vec<&&Rung> = live.iter().filter(|r| r.t_target == t && !r.verdict.starts_with("REFUSED")).collect();
        if at.is_empty() {
            println!("#   T {t:>5.0} K: no scorable seed");
            continue;
        }
        let m = at.iter().map(|r| r.order).sum::<f64>() / at.len() as f64;
        let mob = at.iter().map(|r| r.mobility).sum::<f64>() / at.len() as f64;
        println!(
            "#   T {:>5.0} K: order {:.4} (seeds {:?})  mobility {:.4}  verdicts {:?}",
            t, m,
            at.iter().map(|r| format!("{:.4}", r.order)).collect::<Vec<_>>(),
            mob,
            at.iter().map(|r| r.verdict.clone()).collect::<Vec<_>>()
        );
        means.push((t, m, at.len()));
    }
    let mut g8 = true;
    for w in means.windows(2) {
        if w[1].1 < w[0].1 - G8_TOL {
            println!(
                "#   BREACH: order falls {:.4} -> {:.4} between {:.0} K and {:.0} K",
                w[0].1, w[1].1, w[0].0, w[1].0
            );
            g8 = false;
        }
    }
    println!(
        "# G8: {}",
        if g8 { "HOLDS — order does not fall as the scene is cooled" } else { "KILLED" }
    );

    // ------------------------------------------------------------------ P4
    println!("\n# ================================ P4 — the scrambled-scene plant (carrier: order channel)");
    let top = live
        .iter()
        .filter(|r| !r.verdict.starts_with("REFUSED"))
        .max_by(|a, b| a.order.partial_cmp(&b.order).unwrap());
    match top {
        None => println!("# P4: NOT RUN — no scorable trajectory"),
        Some(r) => {
            println!(
                "# carrier NONZERO IN the order channel: unscrambled order {:.4} at T {:.0} K, \
                 seed {:#018x} (bar 0.10)",
                r.order, r.t_target, r.seed
            );
            if r.order <= 0.10 {
                println!("# P4: UNOBSERVABLE on this scene — the carrier is below its own bar, reported not passed");
            } else {
                let (sc, nf, moved) = scramble(&r.traj, 0x0000_0000_5341_5421);
                let rep = classifier::classify(&sc);
                let fires = rep.order < classifier::STAKE_ORDER
                    && !matches!(rep.verdict, Verdict::Phase(Phase::Ice));
                println!(
                    "# work count: {nf} frames permuted, {moved} atom positions moved"
                );
                println!(
                    "# planted: order {:.4} (bar {:.2}), verdict {:?}, iceFired {}",
                    rep.order, classifier::STAKE_ORDER, rep.verdict, rep.ice_criterion_fired
                );
                println!(
                    "# P4: {}",
                    if fires { "FIRES" } else { "DID NOT FIRE — the order gate it guards is VOID" }
                );

                let (jt, jmoved, jamp) = jitter_scene(&r.traj, 0x0000_0000_5341_5421);
                let jrep = classifier::classify(&jt);
                println!(
                    "# P4b [POST-DATA; does NOT cure P4's void]: geometry jitter, mean amplitude \
                     {jamp:.3} bohr, {jmoved} atom positions displaced"
                );
                println!(
                    "# P4b planted: order {:.4} (bar {:.2}), verdict {:?}, iceFired {}  ->  {}",
                    jrep.order,
                    classifier::STAKE_ORDER,
                    jrep.verdict,
                    jrep.ice_criterion_fired,
                    if jrep.order < classifier::STAKE_ORDER {
                        "the order channel IS movable; the instrument is alive"
                    } else {
                        "the order channel did not move even here — the instrument is in question"
                    }
                );
            }
        }
    }

    println!(
        "\n# WORK: {} quench runs of {} x {} steps, plus one {}-knot curve. \
         Wall clock is NOT the cost on this box (load average 60 on 32 cores).",
        rungs.len(), FRAMES, SUBSTEPS, CURVE_KNOTS
    );
}
