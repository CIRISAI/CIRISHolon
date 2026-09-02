//! SATURATION-2 gate P1: the quench runner. Eight hydrogens and four oxygens in a box,
//! thermostatted down from a hot start, and what comes out.
//!
//! Three arms, eight staked seeds each:
//!
//! * `mixed`     — 8 H + 4 O, the product arm. Branch (a) is H2O as the modal
//!                 O-containing molecule with zero free oxygen.
//! * `hydrogen`  — 12 H, the control that has to reproduce SATURATION-1's molecules.
//! * `oxygen`    — 12 O, the control that has to show the O-O curve is live at all. This
//!                 arm is MBE2-ONLY by design — SATURATION-2 does not tabulate (O,O,O) —
//!                 so its bonding is pair bonding and is labelled as such, exactly as the
//!                 freeze's Scope says.
//!
//! Every number the protocol needs is a `const` here rather than a flag, so a reported run
//! re-runs byte for byte.
//!
//! ```text
//! cargo run --release -p holon-render --example waterquench -- <arm> [seed ...]
//!   arm = mixed | hydrogen | oxygen
//! ```
//!
//! With no seeds listed it runs the eight staked ones.

use holon_chem::elements::{Species, HYDROGEN, OXYGEN};
use holon_chem::pair::generate_pair_table;
use holon_render::bank::Host;
use holon_render::sim::{Boundary, Dims, Sim, K_B, DEFAULT_SCENE_ATOMS};
use holon_render::{load_pair_table, TABLE_OK};
use std::time::Instant;

// ================================================================ THE FROZEN PROTOCOL

/// The eight staked seeds. Written here, not generated: a seed a program chose is a seed
/// nobody staked.
const SEEDS: [u64; 8] = [
    0x0000_0000_5341_5421,
    0x0000_0000_5341_5422,
    0x0000_0000_5341_5423,
    0x0000_0000_5341_5424,
    0x0000_0000_5341_5425,
    0x0000_0000_5341_5426,
    0x0000_0000_5341_5427,
    0x0000_0000_5341_5428,
];

/// Atoms per scene, the same in every arm. Holding `N` fixed across the three arms is
/// what makes them comparable: the box, the density, the thermostat's per-atom coupling
/// and the number of triples are then identical, and the arms differ only in WHICH nuclei
/// are in the box.
const N_ATOMS: usize = 12;

/// The mixed arm's composition: 4 oxygens, then 8 hydrogens. The freeze's 8 H + 4 O.
const N_OXYGEN_MIXED: usize = 4;

/// Box, bohr. SATURATION-1 ran sixteen atoms in 40 x 24; this is twelve at the same
/// number density, so the hydrogen control's coalescence is comparable to its own bank's.
const BOX_W: f64 = 34.6;
const BOX_H: f64 = 20.8;

/// Initial kinetic temperature, kelvin. Hot enough that the opening configuration is a
/// gas rather than a lattice, cold enough that no pair starts up the repulsive wall.
const T_INIT: f64 = 3000.0;
/// Thermostat target, kelvin — the quench's floor.
const T_TARGET: f64 = 300.0;
/// Berendsen coupling time, in atomic time units.
const TAU: f64 = 2000.0;

/// Grain boundaries per run, and substeps per boundary. `dt` is derived from the curves,
/// so the sim time this buys is printed rather than assumed.
const FRAMES: usize = 20000;
const SUBSTEPS: u32 = 64;

/// Jitter on the opening lattice, bohr.
const JITTER: f64 = 0.8;

/// Knots per pair curve. Every knot is a full CI solve and O2 is 2025 determinants a
/// point, so this is the one place the protocol pays for the second element: measured at
/// 0.21 s / 0.40 s / 5.11 s at 2.2 / 3.0 / 5.0 bohr, the long range being where the
/// near-degenerate dissociation makes Davidson work.
const CURVE_KNOTS: usize = 96;

// ================================================================ deterministic setup

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

#[derive(Clone, Copy, PartialEq)]
enum Arm {
    Mixed,
    Hydrogen,
    Oxygen,
}

impl Arm {
    fn parse(s: &str) -> Option<Arm> {
        match s {
            "mixed" => Some(Arm::Mixed),
            "hydrogen" => Some(Arm::Hydrogen),
            "oxygen" => Some(Arm::Oxygen),
            _ => None,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Arm::Mixed => "mixed (8 H + 4 O)",
            Arm::Hydrogen => "hydrogen control (12 H)",
            Arm::Oxygen => "oxygen control (12 O)",
        }
    }
    /// Which nucleus sits at each index. Oxygens FIRST in the mixed arm, so the lattice
    /// cell an oxygen occupies is a function of the protocol rather than of the seed.
    fn species(self, i: usize) -> Species {
        match self {
            Arm::Hydrogen => HYDROGEN,
            Arm::Oxygen => OXYGEN,
            Arm::Mixed => {
                if i < N_OXYGEN_MIXED {
                    OXYGEN
                } else {
                    HYDROGEN
                }
            }
        }
    }
}

/// The opening scene: a 4 x 3 lattice with a seeded jitter, and velocities drawn from a
/// Maxwellian at `T_INIT` — PER SPECIES, because a Maxwellian is a distribution over
/// speeds and oxygen is sixteen times hydrogen's mass. Drawing one sigma for the box
/// would open the scene at two different temperatures.
fn place(s: &mut Sim, arm: Arm, seed: u64) {
    let mut st = seed;
    s.reset(N_ATOMS);
    for i in 0..N_ATOMS {
        assert!(s.set_species(i, arm.species(i)), "species {i} did not register");
    }
    // DERIVE THE TIMESTEP FROM THE SCENE, not from the empty box.
    //
    // `adopt_table_timescale` walks the ACTIVE pair types — every (i, j) in the box — and
    // takes the stiffest mode's `omega^2 = k_e / mu`. `load_pair_table` calls it too, but
    // that happens while the box is still empty, so its loop body never runs and it falls
    // through to a default: one curve and a hydrogen reduced mass. Placing the atoms and
    // not re-deriving leaves that default in force.
    //
    // It is not a small effect and it was not visible from the outside. As first run, the
    // three arms took dt = 1.0772 (12 H), 0.8490 (12 O) and 4.3088 (mixed) — the MIXED
    // arm four times the hydrogen one, though it contains the same H-H mode. A protocol
    // whose whole design is "hold N fixed so the arms are comparable" cannot have the
    // arms integrating at different timesteps.
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
        // MOMENTUM, not velocity: the drift that has to be removed is the box's total
        // momentum, and with two masses in the scene those are different sums.
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
    s.target_temperature = T_TARGET;
    s.thermostat_tau = TAU;
}

// ================================================================ THE MEASUREMENT RULE
//
// Frozen with the protocol, and it is a MOLECULE rule rather than a size rule, because
// this campaign's question is which molecules form and `cluster_sizes` cannot tell H2O
// from H3 plus a free O.
//
// Components of the bonded-pair graph, from `Sim::cluster_species_counts` — one union-find
// over one edge set, the same one the headline `cluster_count` reads, read for its
// COMPOSITION. A component of one atom is a FREE ATOM, not a molecule.
//
//   * a molecule is written O_a H_b with a and b its nuclear counts;
//   * "O-containing" means a >= 1;
//   * the MODAL O-containing molecule is the most common composition among them, ties
//     broken toward the one with FEWER hydrogens — the conservative direction for a gate
//     whose branch (a) is "H2O", so a tie can never be resolved in the claim's favour;
//   * "free O" is a component that is exactly one oxygen.

/// One component's composition: (oxygens, hydrogens).
type Comp = (usize, usize);

fn compositions(s: &Sim) -> Vec<Comp> {
    let counts = s.cluster_species_counts();
    let sizes = s.cluster_sizes();
    let mut out = Vec::new();
    for i in 0..s.n {
        if sizes[i] == 0 {
            continue;
        }
        let (mut o, mut h) = (0usize, 0usize);
        for (z, n) in counts[i] {
            match z {
                8 => o += n,
                1 => h += n,
                _ => {}
            }
        }
        out.push((o, h));
    }
    out
}

fn fmt_comp(c: Comp) -> String {
    let mut s = String::new();
    if c.0 > 0 {
        s.push('O');
        if c.0 > 1 {
            s.push_str(&c.0.to_string());
        }
    }
    if c.1 > 0 {
        s.push('H');
        if c.1 > 1 {
            s.push_str(&c.1.to_string());
        }
    }
    if s.is_empty() {
        s.push('-');
    }
    s
}

struct Reading {
    /// Every component with two or more atoms, sorted for a stable print.
    molecules: Vec<Comp>,
    /// The modal O-containing molecule, or `None` if there are none.
    modal_o: Option<Comp>,
    free_o: usize,
    free_h: usize,
    largest: usize,
}

fn reading(s: &Sim) -> Reading {
    let comps = compositions(s);
    let mut molecules: Vec<Comp> = comps.iter().copied().filter(|c| c.0 + c.1 >= 2).collect();
    molecules.sort();
    let free_o = comps.iter().filter(|c| **c == (1, 0)).count();
    let free_h = comps.iter().filter(|c| **c == (0, 1)).count();
    let largest = comps.iter().map(|c| c.0 + c.1).max().unwrap_or(0);
    // The mode over O-containing molecules, ties toward FEWER hydrogens.
    let mut best: Option<(usize, Comp)> = None;
    for c in molecules.iter().copied().filter(|c| c.0 >= 1) {
        let n = molecules.iter().filter(|d| **d == c).count();
        best = match best {
            Some((bn, bc)) if (bn, std::cmp::Reverse(bc.1)) >= (n, std::cmp::Reverse(c.1)) => {
                Some((bn, bc))
            }
            _ => Some((n, c)),
        };
    }
    Reading {
        molecules,
        modal_o: best.map(|(_, c)| c),
        free_o,
        free_h,
        largest,
    }
}

// ================================================================ the run

struct Outcome {
    seed: u64,
    r: Reading,
    drift: f64,
    bound: f64,
    momentum: f64,
    momentum_bound: f64,
    temperature: f64,
    e_three: f64,
    e_many: f64,
    de4_evals: u64,
    /// The (O,O,H) and (O,O,O) fence's incidence: triples the three-body sector refused
    /// for want of a table, at the final force evaluation. The prereg requires it counted.
    fenced: u64,
    /// The timestep the run ACTUALLY used, recorded per seed rather than printed once
    /// from the header. Reading it off the scene before the seeds are placed reports the
    /// empty box's fallback, which is how a four-times-too-coarse timestep can be
    /// reported as the protocol's own number.
    dt: f64,
    seconds: f64,
}

/// One seed, in place. The scene is REUSED rather than cloned: `Sim` is not `Clone` (it
/// carries the bank's fixed tables), and `place` calls `reset` and `rebase`, which is what
/// clears the ledger's peaks between seeds. So a seed's run starts from the same state
/// whichever seed ran before it.
fn run(s: &mut Sim, arm: Arm, seed: u64) -> Outcome {
    let t0 = Instant::now();
    place(s, arm, seed);
    assert!(
        s.pairs_ready(),
        "the bank is missing a curve this arm needs; nothing would move"
    );
    for frame in 0..FRAMES {
        s.step_frame(SUBSTEPS);
        if (frame + 1) % 1000 == 0 || frame + 1 == FRAMES {
            println!(
                "  [{:#018x}] frame {:>5}/{} | T {:>4.0} K | dE4 solves: {:>4} | drift: {:.2e} | time: {:>5.1} s",
                seed,
                frame + 1,
                FRAMES,
                s.temperature(),
                s.many_body_evals,
                s.drift(),
                t0.elapsed().as_secs_f64()
            );
        }
    }
    let dt = s.dt();
    Outcome {
        seed,
        dt,
        r: reading(s),
        drift: s.drift_peak,
        bound: s.drift_bound(),
        momentum: s.momentum_residual_peak,
        momentum_bound: s.momentum_bound(),
        temperature: s.temperature(),
        e_three: s.e_three,
        e_many: s.e_many,
        de4_evals: s.many_body_evals,
        fenced: s.fence_untabulated,
        seconds: t0.elapsed().as_secs_f64(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arm = args
        .first()
        .and_then(|a| Arm::parse(a))
        .expect("arm = mixed | hydrogen | oxygen");
    let seeds: Vec<u64> = if args.len() > 1 {
        args[1..]
            .iter()
            .map(|a| u64::from_str_radix(a.trim_start_matches("0x"), 16).expect("hex seed"))
            .collect()
    } else {
        SEEDS.to_vec()
    };

    // The curves. Which pairs an arm needs is a fact about its composition, and the bank
    // caps at three species, so only what the arm contains is paid for.
    let t0 = Instant::now();
    let mut base = Box::new(Sim::empty());
    base.boundary = Boundary::Walls;
    base.dims = Dims::Two;
    base.width = BOX_W;
    base.height = BOX_H;
    let needed: &[(Species, Species)] = match arm {
        Arm::Hydrogen => &[(HYDROGEN, HYDROGEN)],
        Arm::Oxygen => &[(OXYGEN, OXYGEN)],
        Arm::Mixed => &[(HYDROGEN, HYDROGEN), (OXYGEN, HYDROGEN), (OXYGEN, OXYGEN)],
    };
    for &(a, b) in needed {
        let t = Instant::now();
        let pt = generate_pair_table(a, b, CURVE_KNOTS);
        let well = match pt.meta.well {
            Some(w) => format!("R_e = {:.4} bohr, D_e = {:.6} Ha", w.r_e, w.d_e),
            None => "NO WELL".to_string(),
        };
        assert_eq!(
            load_pair_table(&mut base, &pt, Host::Native),
            TABLE_OK,
            "the {}-{} curve did not load",
            a.symbol,
            b.symbol
        );
        if pt.meta.worst_residual > holon_chem::pair::CONVERGED_RESIDUAL {
            println!(
                "# WARNING {}-{}: worst residual {:.2e} exceeds CONVERGED_RESIDUAL {:.0e}. \
                 Locate it before trusting anything this arm reports; see \
                 examples/s2_oo_residual.rs",
                a.symbol,
                b.symbol,
                pt.meta.worst_residual,
                holon_chem::pair::CONVERGED_RESIDUAL
            );
        }
        println!(
            "# curve {}-{}: {CURVE_KNOTS} knots, {}, worst residual {:.1e}, {:.1} s",
            a.symbol,
            b.symbol,
            well,
            pt.meta.worst_residual,
            t.elapsed().as_secs_f64()
        );
    }

    // The three-body surfaces. H3 is generated; (O,H,H) is the committed artifact; (O,O,H) is generated.
    base.trimer = holon_chem::trimer::generate().expect("the H3 table generates");
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../holon-chem/tests/data/s2/s2_water_table.txt"),
    )
    .expect("the committed (O,H,H) table");
    base.water = holon_chem::water::from_text(&src).expect("it parses");
    base.ooh = holon_chem::ooh::generate().expect("the OOH table generates");
    assert!(base.ooh.loaded, "OOH table must be loaded and ready");
    // (O,O,O) is honestly fenced per FSD section 10 pending table certification
    base.ozone = holon_chem::ozone::OzoneTable::empty();
    base.many_body_order = 4;

    // The timestep is reported from a PLACED scene, because that is where it is derived
    // from; reading `base.dt()` on the empty box reports the fallback rather than the
    // protocol's own number.
    place(&mut base, arm, seeds[0]);
    println!(
        "# arm = {}   seeds = {}   {N_ATOMS} atoms in {BOX_W} x {BOX_H} bohr\n\
         # {FRAMES} boundaries x {SUBSTEPS} substeps, dt = {:.4} a.u. -> {:.2} ps\n\
         # T_init {T_INIT} K, T_target {T_TARGET} K, tau {TAU}\n\
         # setup {:.1} s",
        arm.label(),
        seeds.len(),
        base.dt(),
        base.dt() * (FRAMES * SUBSTEPS as usize) as f64 * 2.4188843265e-5,
        t0.elapsed().as_secs_f64()
    );

    let mut outs = Vec::new();
    for &seed in &seeds {
        let o = run(&mut base, arm, seed);
        println!(
            "seed {:#018x}  dt {:.4}  modal-O {:>4}  free O {}  free H {}  largest {}  \
             molecules [{}]  fenced {}  dE4_evals {}  drift {:.2e}/{:.2e}  |p| {:.2e}/{:.2e}  T {:.0} K  \
             {:.0} s",
            o.seed,
            o.dt,
            o.r.modal_o.map(fmt_comp).unwrap_or_else(|| "-".into()),
            o.r.free_o,
            o.r.free_h,
            o.r.largest,
            o.r
                .molecules
                .iter()
                .map(|c| fmt_comp(*c))
                .collect::<Vec<_>>()
                .join(" "),
            o.fenced,
            o.de4_evals,
            o.drift,
            o.bound,
            o.momentum,
            o.momentum_bound,
            o.temperature,
            o.seconds
        );
        outs.push(o);
    }

    // ------------------------------------------------------------------ the verdict
    println!("\n# ---- {} ----", arm.label());
    let n = outs.len();
    let water = outs
        .iter()
        .filter(|o| o.r.modal_o == Some((1, 2)))
        .count();
    let zero_free_o = outs.iter().filter(|o| o.r.free_o == 0).count();
    let modal_two = outs.iter().filter(|o| o.r.largest >= 2).count();
    println!("# seeds with the modal O-containing molecule = H2O : {water} / {n}");
    println!("# seeds with ZERO free oxygen                      : {zero_free_o} / {n}");
    println!("# seeds with any molecule at all                   : {modal_two} / {n}");
    let worst_drift = outs
        .iter()
        .map(|o| o.drift / o.bound)
        .fold(0.0f64, f64::max);
    let worst_p = outs
        .iter()
        .map(|o| o.momentum / o.momentum_bound)
        .fold(0.0f64, f64::max);
    println!("# worst drift / bound = {worst_drift:.3e}, worst |p| / bound = {worst_p:.3e}");
    println!(
        "# fence incidence (triples refused for want of a table), per seed: {:?}",
        outs.iter().map(|o| o.fenced).collect::<Vec<_>>()
    );
    println!(
        "# dE4 (O,H,H,H) ab-initio solves, per seed: {:?}",
        outs.iter().map(|o| o.de4_evals).collect::<Vec<_>>()
    );
    let mut hist: Vec<(Comp, usize)> = Vec::new();
    for o in &outs {
        for &c in &o.r.molecules {
            match hist.iter_mut().find(|(d, _)| *d == c) {
                Some((_, n)) => *n += 1,
                None => hist.push((c, 1)),
            }
        }
    }
    hist.sort_by_key(|(c, _)| (c.0, c.1));
    println!(
        "# molecule census over all seeds: {}",
        hist.iter()
            .map(|(c, n)| format!("{}x{}", n, fmt_comp(*c)))
            .collect::<Vec<_>>()
            .join("  ")
    );
}
