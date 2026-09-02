//! B2 — THE LONG-RANGE PAIR SUBSYSTEM'S GATE BATTERY.
//!
//! ```text
//!   cargo run --release --example b2_longrange -- \
//!       --arm=engine|frames|refusals [--curves=hh|full] [--steps=N] [--budget=X]
//!       [--root=DIR] [--manifest=FILE] [--stride=N]
//! ```
//!
//! Frozen design: `conformance/water_observatory/B2_PREREG.md`, ADMITTED by
//! `Audit/prereg_audit.py` and committed BEFORE `longrange.rs` existed. Nothing here moves
//! a threshold that freeze staked; where this file disagrees with the freeze, this file is
//! the defect.
//!
//! Three arms, because the gates have three different costs and three different subjects:
//!
//! * `engine` — the conservation battery (G4, G5, G6), the ledger and gradient gates (G7,
//!   G8), the cache seam (G9), image convergence (G10), the work count (G12) and the cost
//!   curve (G13), each with its plant. Subject: the engine.
//! * `frames` — the channel split (G1), the near sector's coverage (G2) and B1b's bill
//!   (G14), on B1b's own trajectories under B1b's own manifest refusal. Subject: the
//!   measurement B2 was fired by.
//! * `refusals` — G11. Every refusal is constructed and observed to fire, because a scope
//!   fence nobody has seen fire is a comment.
//!
//! G3 runs in every arm that loads a curve, because every other gate's meaning depends on
//! which band the tail landed in.

use holon_chem::elements::{Species, HYDROGEN, OXYGEN};
use holon_chem::pair::generate_pair_table;
use holon_lens::traj::Trajectory;
use holon_render::bank::Host;
use holon_render::cells::{switch_c2, BoxGeom};
use holon_render::longrange::{
    CurveTail, FarPlant, FarRefusal, FarSector, TailBand, SHELL_CAP,
};
use holon_render::sim::{Boundary, Dims, Sim, PAIR_SWITCH_WIDTH};
use holon_render::{load_pair_table, TABLE_OK};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ============================================================== THE STAKED CONSTANTS
//
// Quoted from B2_PREREG.md and, where they are inherited, from B1B_PREREG.md. A value here
// that disagrees with a freeze is a defect in this file.

/// B1b's primary cutoff, bohr: `three_body_cutoff() = list_cutoff()` at the commit that
/// produced the parked artifacts. G1 decomposes the discard at exactly this radius.
const C_STAR: f64 = 15.0;

/// The protocol's knot count and box, carried unchanged so nothing about the load path
/// differs from the audit B2 answers.
const CURVE_KNOTS: usize = 96;
const BOX_W: f64 = 34.6;
const BOX_H: f64 = 20.8;

/// B1b's inherited fraction. NOT re-chosen: the one number most obviously tunable to
/// produce an outcome is the one B1b refused to touch, and B2 refuses too.
const NEGLIGIBLE_FRACTION: f64 = 0.10;

/// G1's dominance ratio. Above it one channel is the finding; inside it both are.
const G1_DOMINANCE: f64 = 10.0;

/// G2's relative tolerance on the near sector's reproduction of the complete sum.
const G2_TOL: f64 = 1.0e-12;

/// G8's relative tolerance on the far term's gradient.
const G8_TOL: f64 = 1.0e-7;
/// G8's work count floor: configurations differenced.
const G8_CONFIGS: usize = 100;

/// G7's relative tolerance on the virial against `−dE/dλ`, and its three step sizes.
const G7_TOL: f64 = 1.0e-6;
const G7_STEPS: [f64; 3] = [1.0e-3, 1.0e-4, 1.0e-5];

/// G12's floors.
const G12_MIN_STEPS: u64 = 20_000;
const G12_MIN_FRAMES: usize = 50;

/// G13's sizes and repetitions.
const G13_SIZES: [usize; 5] = [12, 24, 48, 96, 192];
const G13_REPS: usize = 3;
/// Substeps per grain boundary. The peaks the conservation gates read are sampled at
/// boundaries, so a run that never closes one reports zeros.
const GRAIN_SUBSTEPS: u32 = 20;
/// G13 measures until it has burned at least this much CPU per reading, because a 10 ms
/// clock tick against a microsecond kernel reads 0.0 and 0.0 is not a time.
const G13_MIN_CPU_S: f64 = 0.10;

/// The declared per-pair far budget, hartree. A DECLARED INPUT like
/// `Sim::set_pair_cutoff`'s floor, not a threshold: it is printed with every result and
/// `--budget=` overrides it. R2 refuses when the image shells cannot meet it.
const DEFAULT_BUDGET: f64 = 1.0e-9;

/// B1b's per-seed drift peaks `D_s` and bounds `B_s`, mixed class, read from
/// `B1B_RESULTS.md` §2 — themselves read from the committed arm log
/// `conformance/water_observatory/census_traj_arm_fenced.log`. Quoted as `0.10·B_s` and
/// `0.10·D_s`, the form the freeze gates.
const B1B_MIXED: [(u64, f64, f64, f64); 8] = [
    // seed, 0.10*B_s, 0.10*D_s, B1b's measured ratio
    (0x0000000053415421, 2.040e0, 1.500e-5, 0.454),
    (0x0000000053415422, 8.850e-1, 4.800e-6, 1.898),
    (0x0000000053415423, 1.580e-1, 5.620e-6, 1.574),
    (0x0000000053415424, 2.050e-1, 4.610e-6, 2.496),
    (0x0000000053415425, 1.330e0, 2.450e-5, 0.312),
    (0x0000000053415426, 1.220e0, 1.130e-5, 0.982),
    (0x0000000053415427, 8.220e-1, 2.260e-5, 0.443),
    (0x0000000053415428, 1.360e0, 1.280e-5, 0.595),
];

// ====================================================================== the launch header

/// CPU time this process has burned, seconds — utime + stime from `/proc/self/stat`.
///
/// CPU and not wall, because M-PLACEMENT-LOTTERY's fourth instance is exactly a timing
/// table that measured a neighbouring lane's load: descheduling inflates wall clock and
/// does not touch this.
fn cpu_seconds() -> f64 {
    let Ok(s) = std::fs::read_to_string("/proc/self/stat") else {
        return f64::NAN;
    };
    // Fields after the comm field, which may itself contain spaces.
    let Some(rest) = s.rsplit_once(") ") else {
        return f64::NAN;
    };
    let f: Vec<&str> = rest.1.split_whitespace().collect();
    let tick = 100.0; // _SC_CLK_TCK on every Linux this runs on
    let u: f64 = f.get(11).and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let sy: f64 = f.get(12).and_then(|v| v.parse().ok()).unwrap_or(0.0);
    (u + sy) / tick
}

fn cpu_now() -> usize {
    let Ok(s) = std::fs::read_to_string("/proc/self/stat") else {
        return usize::MAX;
    };
    let Some(rest) = s.rsplit_once(") ") else {
        return usize::MAX;
    };
    rest.1
        .split_whitespace()
        .nth(36)
        .and_then(|v| v.parse().ok())
        .unwrap_or(usize::MAX)
}

fn loadavg() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next().and_then(|v| v.parse().ok()))
        .unwrap_or(f64::NAN)
}

/// The clock this core is actually running at, as a fraction of its advertised maximum.
///
/// M-IDLE-CALIBRATED-TIMEOUT's remedy in one line: take the baseline in the regime you will
/// compare against, or state which regime it came from. Half the variation in any timing
/// here is visible in this number rather than inferred.
fn clock_fraction(cpu: usize) -> f64 {
    let cur = std::fs::read_to_string(format!(
        "/sys/devices/system/cpu/cpu{cpu}/cpufreq/scaling_cur_freq"
    ))
    .ok()
    .and_then(|s| s.trim().parse::<f64>().ok());
    let max = std::fs::read_to_string(format!(
        "/sys/devices/system/cpu/cpu{cpu}/cpufreq/cpuinfo_max_freq"
    ))
    .ok()
    .and_then(|s| s.trim().parse::<f64>().ok());
    match (cur, max) {
        (Some(c), Some(m)) if m > 0.0 => c / m,
        _ => f64::NAN,
    }
}

/// Whether this cpu has an SMT sibling — the P/E discriminator on this box, and the reason
/// a citable ratio must declare its core class.
fn core_class(cpu: usize) -> &'static str {
    match std::fs::read_to_string(format!(
        "/sys/devices/system/cpu/cpu{cpu}/topology/thread_siblings_list"
    )) {
        Ok(s) if s.trim().contains(',') || s.trim().contains('-') => "P (SMT sibling present)",
        Ok(_) => "E (no SMT sibling)",
        Err(_) => "unknown",
    }
}

// ====================================================================== the scene

/// A deterministic scene: no RNG crate, no clock, no file. The same call gives the same
/// atoms on any machine, which is what lets G9 compare a rescaled sim against a fresh one
/// and mean anything by "the same scene".
struct SceneCfg {
    n: usize,
    /// Fraction of the atoms that are oxygen; the rest hydrogen.
    oxygen_every: usize,
    boundary: Boundary,
    w: f64,
    h: f64,
    /// Positions and box multiplied by this before anything else. G9's "fresh at the scaled
    /// box" arm is this at `f != 1`; every other caller leaves it at 1.
    prescale: f64,
    /// Spread the atoms through the box's depth instead of pinning them to the mid-plane.
    ///
    /// Only G2 needs this, and it needs it for a reason worth stating: in an open or walled
    /// box `CellList::rebuild` takes its extent from the ATOMS' bounding box, not from the
    /// nominal box. A `Dims::Two` scene sits on one plane, so its z extent is zero, so
    /// `nc[2]` is 1, so the route is COMPLETE however many atoms the scene holds. G2
    /// compares two enumerations and cannot do that on a scene that has only one.
    three_d: bool,
}

/// A 64-bit LCG, written out rather than pulled in: the constants are Knuth's MMIX and the
/// sequence is a property of this file, so a scene is reproducible from the source alone.
fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

fn build_scene(cfg: &SceneCfg, curves: &[(Species, Species)]) -> Box<Sim> {
    let mut sim = Box::new(Sim::empty());
    sim.dims = if cfg.three_d { Dims::Three } else { Dims::Two };
    sim.boundary = cfg.boundary;
    sim.width = cfg.w * cfg.prescale;
    sim.height = cfg.h * cfg.prescale;
    sim.depth = cfg.h * cfg.prescale;
    for (a, b) in curves {
        let pt = generate_pair_table(*a, *b, CURVE_KNOTS);
        assert_eq!(load_pair_table(&mut sim, &pt, Host::Native), TABLE_OK);
    }
    sim.reset(cfg.n);
    // A jittered grid rather than uniform placement: two atoms landing on top of each other
    // is a force spike that would make every conservation reading about the collision
    // instead of about the far sector.
    // A 3D scene gets a CUBIC lattice, not the planar grid with the atoms striped through
    // z: striping puts consecutive atoms tens of bohr apart in z while the grid moves them
    // a few bohr in x, so the nearest actual 3D neighbour is far out in the tail and a gate
    // reading that scene is reading almost nothing.
    let side = if cfg.three_d {
        (cfg.n as f64).cbrt().ceil() as usize
    } else {
        (cfg.n as f64).sqrt().ceil() as usize
    };
    let cols = side;
    let rows = if cfg.three_d { side } else { cfg.n.div_ceil(cols) };
    let mut st = 0x5341_5424u64;
    for i in 0..cfg.n {
        let (cx, cy) = if cfg.three_d {
            (i % side, (i / side) % side)
        } else {
            (i % cols, i / cols)
        };
        let x = (cx as f64 + 0.5) / cols as f64 * cfg.w + (lcg(&mut st) - 0.5) * 0.6;
        let y = (cy as f64 + 0.5) / rows as f64 * cfg.h + (lcg(&mut st) - 0.5) * 0.6;
        let a = &mut sim.atoms[i];
        a.x = x * cfg.prescale;
        a.y = y * cfg.prescale;
        a.z = if cfg.three_d {
            ((i / (side * side)) as f64 + 0.5) / side as f64 * cfg.h * cfg.prescale
        } else {
            0.5 * cfg.h * cfg.prescale
        };
        // Velocities in the plane only, and small: the gates read conservation, not
        // thermodynamics, and a hot scene spends its time in the repulsive wall.
        a.vx = (lcg(&mut st) - 0.5) * 2.0e-4;
        a.vy = (lcg(&mut st) - 0.5) * 2.0e-4;
        a.vz = 0.0;
        let oxy = cfg.oxygen_every > 0 && i % cfg.oxygen_every == 0;
        a.species = if oxy { OXYGEN } else { HYDROGEN };
    }
    assert!(sim.sync_species(), "the bank holds every species this scene needs");
    sim
}

/// Read each loaded curve's knots straight out of the engine's own bank.
///
/// A VIEW of the committed table and not a copy of it: G3 is a measurement OF that table,
/// and a second array of knots is a second thing that can disagree with it. `hi_b` is
/// recomputed here from the public knot accessors by the same expression
/// `Table::build_extrapolations` uses, rather than read through a new accessor added for
/// this instrument.
fn curve_tails(sim: &Sim, meta: &BTreeMap<usize, (&'static str, u64, f64)>) -> Vec<Option<CurveTail>> {
    (0..holon_render::bank::MAX_TABLES)
        .map(|s| {
            let t = sim.bank.table_slot(s);
            if !t.is_loaded() {
                return None;
            }
            let n = t.knots();
            let r: Vec<f64> = (0..n).map(|k| t.knot_r(k)).collect();
            let u: Vec<f64> = (0..n).map(|k| t.knot_u(k)).collect();
            let (an, dn) = (u[n - 1], t.knot_d(n - 1));
            let hi_b = if an.abs() > 0.0 { -dn / an } else { 0.0 };
            let (exit, budget, unc) = meta
                .get(&s)
                .copied()
                .unwrap_or(("Unrecorded", 0, f64::NAN));
            Some(CurveTail {
                r,
                u,
                hi_b,
                solver_exit: exit,
                solver_budget_iterations: budget,
                uncertainty_hartree: unc,
            })
        })
        .collect()
}

/// G3, printed for every loaded curve whether or not the far sector uses it.
fn report_g3(tails: &[Option<CurveTail>]) -> (usize, usize) {
    let (mut adopting, mut fenced) = (0, 0);
    println!("COLUMNS G3 slot r_max u_at_max p_fit fit_residual exp_index knots band");
    for (s, c) in tails.iter().enumerate() {
        let Some(c) = c else { continue };
        let f = c.fit();
        match f.band {
            TailBand::Adopting => adopting += 1,
            TailBand::Fenced => fenced += 1,
        }
        println!(
            "G3 {s} {:.4} {:.6e} {:.4} {:.4e} {:.4} {} {:?}",
            f.r_max, f.u_at_max, f.p_fit, f.residual, f.exp_index, f.knots_fitted, f.band
        );
    }
    println!(
        "# GATE G3: {adopting} curve(s) ADOPTING, {fenced} FENCED. Band [{}, {}] on p_fit, \
         exp_index <= {}x p_fit.",
        holon_render::longrange::P_FIT_LO,
        holon_render::longrange::P_FIT_HI,
        holon_render::longrange::EXP_INDEX_FACTOR
    );
    (adopting, fenced)
}

// ====================================================================== the engine arm

#[allow(clippy::too_many_lines)]
fn arm_engine(curves: &[(Species, Species)], steps: u64, budget: f64) {
    let ox = if curves.len() > 1 { 3 } else { 0 };
    let base = SceneCfg {
        n: 12,
        oxygen_every: ox,
        boundary: Boundary::Open,
        w: BOX_W,
        h: BOX_H,
        prescale: 1.0,
        three_d: false,
    };

    // --- the curves, with their solver certificates. W1's fields are B2's R5 inputs.
    let mut probe = Box::new(Sim::empty());
    probe.dims = Dims::Two;
    probe.width = BOX_W;
    probe.height = BOX_H;
    let mut meta: BTreeMap<usize, (&'static str, u64, f64)> = BTreeMap::new();
    let mut setup_cpu = 0.0;
    for (a, b) in curves {
        let t0 = cpu_seconds();
        let pt = generate_pair_table(*a, *b, CURVE_KNOTS);
        setup_cpu += cpu_seconds() - t0;
        assert_eq!(load_pair_table(&mut probe, &pt, Host::Native), TABLE_OK);
        let slot = probe.bank.slot_of_z(a.z, b.z).expect("the curve registered a slot");
        // The disclosure law: exit status and budget beside the number, always. A capped
        // residual is not monotone in effort, so the residual alone is not a specification.
        let exit: &'static str = match format!("{:?}", pt.meta.exit).as_str() {
            "Converged" => "Converged",
            "IterationCap" => "IterationCap",
            "Stagnated" => "Stagnated",
            _ => "Other",
        };
        meta.insert(slot, (exit, pt.meta.solver_budget as u64, pt.meta.worst_residual));
        println!(
            "# W1 {}-{}: slot {slot} route {:?} exit {exit} n_det {} n_basis {} \
             solver_budget {} worst_residual {:.3e}",
            a.symbol, b.symbol, pt.meta.route, pt.meta.n_det, pt.meta.n_basis,
            pt.meta.solver_budget, pt.meta.worst_residual
        );
    }
    println!("# curve setup CPU: {setup_cpu:.1} s");
    let tails = curve_tails(&probe, &meta);
    let (_adopting, fenced) = report_g3(&tails);

    let r_s = tails
        .iter()
        .flatten()
        .map(|c| c.r_max())
        .fold(0.0f64, f64::max);
    println!("# R_s = {r_s:.4} bohr (the largest loaded curve's support); budget = {budget:.3e} Ha");

    let far = match FarSector::build(&tails, r_s, budget, Dims::Two) {
        Ok(f) => f,
        Err(e) => {
            println!("VOID engine arm — the far sector refused to build: {e}");
            return;
        }
    };
    println!(
        "# far sector: R_s = {:.4}  R_f = {:.4} bohr  fenced = {}",
        far.r_s(),
        far.r_f(),
        far.is_fenced()
    );
    // R4 is a live refusal, not a comment: a fenced sector hands out a bracket.
    if fenced > 0 {
        match far.scalar_ok(-1.0e-6, -1.0e-9) {
            Ok(()) => println!("# GATE R4: NOT ARMED — the sector reports itself unfenced"),
            Err(e) => println!("# R4 (live, on the fenced sector): {e}"),
        }
    }

    // ---------------------------------------------------------------- G2
    gate_g2(&base, curves, &tails, r_s, budget);

    // ---------------------------------------------------------------- G8, then G7
    let g8 = gate_g8(&base, curves, &tails, r_s, budget);
    let g7 = gate_g7(&base, curves, &tails, r_s, budget);

    // ---------------------------------------------------------------- G4, G5, G6, G12
    let run = conservation_run(&base, curves, &tails, r_s, budget, steps, None);
    println!(
        "# GATE G4 energy: {}  drift_peak {:.6e}  bound {:.6e}  ratio {:.4}  \
         work_columns_ok {}",
        pf(run.energy_ok), run.drift_peak, run.drift_bound,
        run.drift_peak / run.drift_bound, run.work_columns_ok
    );
    println!(
        "# GATE G5 momentum: {}  residual_peak {:.6e}  bound {:.6e}  \
         isolated-pair |F_i + F_j| = {:.1e} (EXACT 0 required)",
        pf(run.momentum_ok), run.momentum_peak, run.momentum_bound, run.third_law
    );
    match run.angular_ok {
        Some(ok) => println!(
            "# GATE G6 angular: {}  residual_peak {:.6e}  bound {:.6e}",
            pf(ok), run.angular_peak, run.angular_bound
        ),
        None => println!("# GATE G6 angular: NOT APPLICABLE — this box does not conserve L"),
    }
    println!(
        "# GATE G12 work count: steps {} (floor {G12_MIN_STEPS})  far contributions {}  \
         image contributions {}  R_s crossings {} (floor 1)",
        run.steps, run.contributions, run.image_contributions, run.crossings
    );
    let g12 = run.steps >= G12_MIN_STEPS && run.contributions > 0 && run.crossings >= 1;
    println!("# GATE G12: {}", pf(g12));

    // ---------------------------------------------------------------- the plants
    println!("# --- PLANTS (M-PLANT-OBS: each pre-checked to fire; a 0.0 carrier REFUSES) ---");
    plant_p2(&base, curves, &tails, r_s, budget, steps);
    plant_p3(&base, curves, &tails, r_s, budget, steps);
    plant_p4(&base, curves, &tails, r_s, budget, steps);
    plant_p5(&base, curves, &tails, r_s, budget, g8);
    plant_p6(&base, curves, &tails, r_s, budget);
    plant_p7(&base, curves, &tails, r_s, budget, g7);

    // ---------------------------------------------------------------- G9, G10, P1
    periodic_arm(curves, &tails, r_s, budget);

    // ---------------------------------------------------------------- G13
    gate_g13(curves, &tails, r_s, budget);
}

fn pf(b: bool) -> &'static str {
    if b {
        "PASS"
    } else {
        "FAIL"
    }
}

/// Everything one integration run measured. One run, three conservation gates, because the
/// three laws constrain three quantities of the same trajectory and separating the runs
/// would let them disagree about which trajectory they were reading.
struct RunOut {
    steps: u64,
    drift_peak: f64,
    drift_bound: f64,
    work_columns_ok: bool,
    energy_ok: bool,
    momentum_peak: f64,
    momentum_bound: f64,
    momentum_ok: bool,
    third_law: f64,
    angular_peak: f64,
    angular_bound: f64,
    angular_ok: Option<bool>,
    contributions: u64,
    image_contributions: u64,
    crossings: u64,
}

fn attach_far(
    sim: &mut Sim,
    tails: &[Option<CurveTail>],
    r_s: f64,
    budget: f64,
    plant: Option<FarPlant>,
) {
    let mut far = FarSector::build(tails, r_s, budget, sim.dims).expect("the far sector builds");
    far.plant = plant;
    sim.far = Some(Box::new(far));
    // `rebase` recomputes the forces and re-zeroes the ledger, which is what makes `l0`,
    // `p0` and `l0_ang` the origins of a run that HAS the far sector rather than one that
    // acquired it partway through.
    sim.rebase();
}

fn conservation_run(
    cfg: &SceneCfg,
    curves: &[(Species, Species)],
    tails: &[Option<CurveTail>],
    r_s: f64,
    budget: f64,
    steps: u64,
    plant: Option<FarPlant>,
) -> RunOut {
    let mut sim = build_scene(cfg, curves);
    attach_far(&mut sim, tails, r_s, budget, plant);
    let (mut contributions, mut images, mut crossings) = (0u64, 0u64, 0u64);
    // THROUGH `step_frame`, NOT `step`. `Sim::close_grain` is where the momentum and
    // angular residuals are sampled, and only `step_frame` calls it — a loop over `step`
    // leaves both peaks at exactly 0.0 forever. The first run of this instrument did
    // exactly that, and P2 and P3 REFUSED on a 0.0 carrier rather than being scored
    // against it: an asserted zero has to be a fact about the scene and not about the
    // instrument's coverage.
    let frames = (steps / GRAIN_SUBSTEPS as u64).max(1);
    for _ in 0..frames {
        for _ in 0..GRAIN_SUBSTEPS {
            sim.step();
            let r = sim.far_reading;
            contributions += r.contributions;
            images += r.image_contributions;
            crossings += r.crossings;
        }
        sim.close_grain();
    }
    // The third law on an ISOLATED far pair, checked as an EXACT zero rather than a small
    // one: `+f` and `−f` are one computed value with opposite signs, so their sum is
    // bit-zero or the construction is not what it claims.
    let third = isolated_pair_third_law(tails, r_s, budget);
    RunOut {
        steps: sim.steps,
        drift_peak: sim.drift_peak,
        drift_bound: sim.drift_bound(),
        work_columns_ok: sim.work_columns_ok(),
        energy_ok: sim.energy_gate() && sim.work_columns_ok(),
        momentum_peak: sim.momentum_residual_peak,
        momentum_bound: sim.momentum_bound(),
        momentum_ok: sim.momentum_gate(),
        third_law: third,
        angular_peak: sim.angular_residual_peak,
        angular_bound: sim.angular_bound(),
        angular_ok: sim.angular_gate(),
        contributions,
        image_contributions: images,
        crossings,
    }
}

fn isolated_pair_third_law(tails: &[Option<CurveTail>], r_s: f64, budget: f64) -> f64 {
    let mut far = FarSector::build(tails, r_s, budget, Dims::Two).expect("builds");
    far.plant = None;
    let pos = [(0.0, 0.0, 0.0), (r_s + 1.0, 0.0, 0.0)];
    let slots = [0usize, 0usize];
    let geom = BoxGeom::new(1.0e6, 1.0e6, 1.0e6, false);
    let mut f = [(0.0, 0.0, 0.0); 2];
    let r_max: Vec<f64> = tails
        .iter()
        .map(|c| c.as_ref().map(CurveTail::r_max).unwrap_or(0.0))
        .collect();
    far.accumulate(&pos, &slots, geom, &mut f, &r_max);
    let s = (f[0].0 + f[1].0, f[0].1 + f[1].1, f[0].2 + f[1].2);
    (s.0 * s.0 + s.1 * s.1 + s.2 * s.2).sqrt()
}

// ---------------------------------------------------------------- G2

/// G2's energy half — the near sector's DECOMPOSED enumeration reproduces the complete one.
///
/// Two routes over one configuration, differing only in how the pairs are found: the cell
/// decomposition built at `list_cutoff()` (which a declared far sector forces to at least
/// `R_s`) against the complete `N²/2` sum, with the same declared switch applied by both.
/// A pair the decomposition cannot reach shows up here as an energy difference, which is
/// the defect B1b measured expressed as a number rather than as a count.
///
/// The EXACT half of G2 — zero pairs in `(c*, r_max]` outside the list — is counted on the
/// real frames in the `frames` arm, where `c*` and the trajectories both live.
fn gate_g2(
    _cfg: &SceneCfg,
    curves: &[(Species, Species)],
    tails: &[Option<CurveTail>],
    r_s: f64,
    budget: f64,
) {
    // ITS OWN SCENE, and it needs one. `CellList::rebuild` takes the cell route only at 64
    // atoms or more AND at least 3 cells per axis, so the 12-atom conservation scene runs
    // the COMPLETE sum on both sides and this gate compares a route against itself. That is
    // a vacuous success wearing a green gate; the first run of this instrument printed
    // "route Complete vs Complete  relative 0.000e0" and it meant nothing at all.
    // The box is GROWN until the decomposition engages, and the scale that worked is
    // printed. A fixed multiple was tried first and did not work: the extent is the ATOMS'
    // bounding box and the placement grid insets them, so a box at three cutoffs per axis
    // yields two cells. Searching for the precondition beats guessing a constant that the
    // next change to the switch width would silently invalidate.
    let mut cells_cfg = SceneCfg {
        n: 512,
        oxygen_every: _cfg.oxygen_every,
        boundary: Boundary::Open,
        w: 0.0,
        h: 0.0,
        prescale: 1.0,
        three_d: true,
    };
    let mut local = Box::new(Sim::empty());
    let mut found = 0.0f64;
    for scale in [5.0f64, 7.0, 9.0, 12.0, 16.0, 22.0, 30.0] {
        cells_cfg.w = scale * r_s.max(1.0);
        cells_cfg.h = scale * r_s.max(1.0);
        let mut s = build_scene(&cells_cfg, curves);
        attach_far(&mut s, tails, r_s, budget, None);
        // A declared truncation is what makes the engine take the neighbour route at all;
        // with none declared it runs the complete sum whatever the cell list says, and the
        // two routes would be the same arithmetic rather than two enumerations.
        if !s.set_pair_cutoff(1.0e-12) {
            println!(
                "# GATE G2 (energy half): NOT RUN — no pair cutoff could be derived at 1e-12 Ha"
            );
            return;
        }
        if s.route() == holon_render::cells::Route::Cells {
            local = s;
            found = scale;
            break;
        }
    }
    if found == 0.0 {
        println!(
            "# GATE G2 (energy half): VOID (V2) — the cell decomposition would not engage at \
             any box size tried, so there is one enumeration and nothing to compare it with. \
             `CellList::rebuild` needs 64+ atoms AND 3 cells per axis of the ATOMS' bounding \
             box, which a planar (Dims::Two) open or walled scene can never supply on z."
        );
        return;
    }
    let cfg = &cells_cfg;
    // THE POPULATION THE GATE IS DRAWN FROM, counted independently of the engine. Two
    // enumerations that both find nothing agree perfectly and say nothing, so the count is
    // printed and floored rather than inferred from a nonzero energy.
    let (_, r_cut) = local.pair_switch().unwrap_or((0.0, 0.0));
    let mut in_range = 0usize;
    for i in 0..local.n {
        for j in (i + 1)..local.n {
            let (a, b) = (&local.atoms[i], &local.atoms[j]);
            let (dx, dy, dz) = (b.x - a.x, b.y - a.y, b.z - a.z);
            if (dx * dx + dy * dy + dz * dz).sqrt() <= r_cut {
                in_range += 1;
            }
        }
    }
    println!(
        "# G2 scene: {} atoms, 3D cubic lattice, box {:.1} bohr ({found}x R_s), \
         {in_range} pairs within r_cut = {r_cut:.4} bohr",
        cfg.n, cfg.w
    );
    let cut = local.pair_switch();
    let route_local = local.route();
    let e_local = local.e_pair;

    let mut complete = build_scene(cfg, curves);
    attach_far(&mut complete, tails, r_s, budget, None);
    assert!(complete.set_pair_cutoff(1.0e-12));
    complete.force_complete_route();
    let e_complete = complete.e_pair;

    let rel = (e_local - e_complete).abs() / e_complete.abs().max(1.0e-30);
    // THE PRECONDITION, asserted rather than hoped for. `Sim::route` exists so a caller can
    // see whether the decomposition engaged instead of assuming it did, and a comparison of
    // the complete route against itself is not a comparison.
    let engaged = route_local == holon_render::cells::Route::Cells;
    println!(
        "# GATE G2 (energy half): {}  n {}  box {:.1} bohr  route {route_local:?} vs {:?} \
         (decomposition engaged: {engaged})  pair switch {:?}  list_cutoff {:.4} bohr \
         (>= R_s {r_s:.4})  e_pair {e_local:.12e} vs {e_complete:.12e}  \
         relative {rel:.3e} (<= {G2_TOL:.0e})",
        if !engaged {
            "VOID (V2: the routes were the same route)"
        } else if in_range == 0 {
            "VOID (V2: no pair within the cutoff, so both routes agreed about nothing)"
        } else {
            pf(rel <= G2_TOL)
        },
        cfg.n,
        cfg.w,
        complete.route(),
        cut,
        local.list_cutoff()
    );
}

// ---------------------------------------------------------------- G8

/// G8 — the far force is minus the gradient of the far energy.
///
/// Run on the far sector directly and not through `Sim`, so that a failure names the far
/// term rather than the sum of every term the engine holds.
fn gate_g8(
    cfg: &SceneCfg,
    curves: &[(Species, Species)],
    tails: &[Option<CurveTail>],
    r_s: f64,
    budget: f64,
) -> f64 {
    let sim = build_scene(cfg, curves);
    let mut far = FarSector::build(tails, r_s, budget, Dims::Two).expect("builds");
    let geom = BoxGeom::new(sim.width, sim.height, sim.depth, false);
    let slots: Vec<usize> = (0..sim.n)
        .map(|i| sim.bank.index_of(sim.atoms[i].species.z).unwrap_or(0))
        .collect();
    let r_max: Vec<f64> = tails
        .iter()
        .map(|c| c.as_ref().map(CurveTail::r_max).unwrap_or(0.0))
        .collect();
    let base: Vec<(f64, f64, f64)> = (0..sim.n)
        .map(|i| (sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z))
        .collect();
    let h = 1.0e-5;
    let mut worst = 0.0f64;
    let mut done = 0usize;
    let mut st = 0xB2_0000_0008u64;
    while done < G8_CONFIGS {
        let i = (lcg(&mut st) * sim.n as f64) as usize % sim.n;
        let ax = if lcg(&mut st) < 0.5 { 0 } else { 1 };
        let mut fx = vec![(0.0, 0.0, 0.0); sim.n];
        far.accumulate(&base, &slots, geom, &mut fx, &r_max);
        let analytic = if ax == 0 { fx[i].0 } else { fx[i].1 };
        let mut plus = base.clone();
        let mut minus = base.clone();
        if ax == 0 {
            plus[i].0 += h;
            minus[i].0 -= h;
        } else {
            plus[i].1 += h;
            minus[i].1 -= h;
        }
        let ep = far.energy_at_shells(&plus, &slots, geom, 0);
        let em = far.energy_at_shells(&minus, &slots, geom, 0);
        let numeric = -(ep - em) / (2.0 * h);
        let scale = analytic.abs().max(numeric.abs()).max(1.0e-30);
        worst = worst.max((analytic - numeric).abs() / scale);
        done += 1;
    }
    println!(
        "# GATE G8 gradient: {}  worst relative {worst:.4e} (<= {G8_TOL:.0e})  \
         configurations {done} (floor {G8_CONFIGS})",
        pf(worst <= G8_TOL)
    );
    worst
}

// ---------------------------------------------------------------- G7

/// G7 — the far channel posts to the virial.
///
/// Compared against `dE_pot/dλ` under an AFFINE scaling, which for a pairwise central
/// potential is exactly `Σ r·dU/dr` — the virial's own definition. Going through
/// `Sim::scale_box` rather than scaling by hand is deliberate: the barostat seam is the
/// thing under test, and a hand-rolled scaling would test a second implementation of it.
///
/// The box is OPEN and the field is off, so `energy() − e_kin` contains only terms that
/// scale affinely; walls and the spring are excluded from the virial by construction and
/// would otherwise be an unexplained difference rather than a defect.
fn gate_g7(
    cfg: &SceneCfg,
    curves: &[(Species, Species)],
    tails: &[Option<CurveTail>],
    r_s: f64,
    budget: f64,
) -> f64 {
    let mut base = build_scene(cfg, curves);
    attach_far(&mut base, tails, r_s, budget, None);
    let virial = base.w_virial;
    let mut worst = 0.0f64;
    let mut finest = f64::NAN;
    let mut prev: Option<(f64, f64)> = None;
    for &h in &G7_STEPS {
        let mut sp = build_scene(cfg, curves);
        attach_far(&mut sp, tails, r_s, budget, None);
        sp.scale_box(1.0 + h).expect("the box scales");
        let ep = sp.energy() - sp.e_kin;
        let mut sm = build_scene(cfg, curves);
        attach_far(&mut sm, tails, r_s, budget, None);
        sm.scale_box(1.0 - h).expect("the box scales");
        let em = sm.energy() - sm.e_kin;
        let d = (ep - em) / (2.0 * h);
        let rel = (d - virial).abs() / virial.abs().max(1.0e-30);
        worst = worst.max(rel);
        // Richardson on two central differences: the leading error is O(h²), so the
        // extrapolated value says how much of the discrepancy is the differencing itself
        // rather than a missing virial term.
        let rich = prev.map(|(hp, dp)| (d * hp * hp - dp * h * h) / (hp * hp - h * h));
        // Every step size gets its own PASS/FAIL, because the freeze gates three of them
        // and a max over the three reports the coarsest one's O(h^2) truncation error as
        // though it were a missing virial. The Richardson column is in the freeze for
        // exactly this separation, and it is printed beside each row rather than summarised.
        println!(
            "G7 h {h:.1e}  dE/dlambda {d:.9e}  virial {virial:.9e}  rel {rel:.3e}  {}{}",
            pf(rel <= G7_TOL),
            match rich {
                Some(v) => format!("  richardson {v:.9e}"),
                None => String::new(),
            }
        );
        finest = rel;
        prev = Some((h, d));
    }
    println!(
        "# GATE G7 virial (LEDGER COMPLETENESS, not a conservation law): {}  \
         worst over the three staked steps {worst:.4e}, finest step {finest:.4e} \
         (<= {G7_TOL:.0e} required at every step)",
        pf(worst <= G7_TOL)
    );
    finest
}

// ---------------------------------------------------------------- the plants

fn plant_p2(
    cfg: &SceneCfg,
    curves: &[(Species, Species)],
    tails: &[Option<CurveTail>],
    r_s: f64,
    budget: f64,
    steps: u64,
) {
    let short = steps.min(2000);
    let run = conservation_run(cfg, curves, tails, r_s, budget, short, Some(FarPlant::OneSidedForce));
    let carrier = run.momentum_peak;
    println!(
        "P2 one-sided far force | carrier |P − P0 − J| = {carrier:.6e} (sector: the far pair \
         force) | G5 {} | verdict {}",
        pf(run.momentum_ok),
        if carrier == 0.0 {
            "REFUSED — carrier reads 0.0, the plant is not observable here"
        } else if !run.momentum_ok {
            "FIRED"
        } else {
            "DID NOT FIRE — G5 cannot see this defect"
        }
    );
}

fn plant_p3(
    cfg: &SceneCfg,
    curves: &[(Species, Species)],
    tails: &[Option<CurveTail>],
    r_s: f64,
    budget: f64,
    steps: u64,
) {
    let short = steps.min(2000);
    let run = conservation_run(
        cfg,
        curves,
        tails,
        r_s,
        budget,
        short,
        Some(FarPlant::NonCentralForce),
    );
    let fired = run.angular_ok == Some(false);
    // THE STAKED OBSERVATION, and the reason G5 and G6 are separate rows: a force that is
    // equal and opposite but not central leaves the LINEAR sum exactly zero and destroys
    // the ANGULAR one. A plant that fires both has not demonstrated the independence.
    println!(
        "P3 non-central far force | carrier |L − L0| = {:.6e} (sector: angular) | \
         G6 {} | G5 {} | verdict {}",
        run.angular_peak,
        match run.angular_ok {
            Some(b) => pf(b),
            None => "N/A",
        },
        pf(run.momentum_ok),
        if run.angular_peak == 0.0 {
            "REFUSED — carrier reads 0.0"
        } else if fired && run.momentum_ok {
            "FIRED, and G5 STAYED GREEN — the two gates are independent"
        } else if fired {
            "FIRED but G5 also fired — independence NOT demonstrated"
        } else {
            "DID NOT FIRE"
        }
    );
}

fn plant_p4(
    cfg: &SceneCfg,
    curves: &[(Species, Species)],
    tails: &[Option<CurveTail>],
    r_s: f64,
    budget: f64,
    steps: u64,
) {
    let run = conservation_run(cfg, curves, tails, r_s, budget, steps, Some(FarPlant::ZeroPointStep));
    println!(
        "P4 zero-point STEP at R_s | carrier = the {:.1e} Ha jump per crossing (sector: the \
         energy ledger) | crossings {} | drift_peak {:.6e} vs bound {:.6e} | G4 {} | verdict {}",
        holon_render::longrange::PLANT_STEP_HARTREE,
        run.crossings,
        run.drift_peak,
        run.drift_bound,
        pf(run.energy_ok),
        if run.crossings == 0 {
            "REFUSED — no pair crossed R_s, so the plant is vacuous (M-VACUOUS-SUCCESS)"
        } else if !run.energy_ok {
            "FIRED"
        } else {
            "DID NOT FIRE — G4's derived bound is larger than the planted defect"
        }
    );
    if run.energy_ok {
        // G4'S POWER CERTIFICATE, and it is a measurement OF the gate rather than a retune
        // OF it: the plant's verdict above stands at the staked 1e-6 Ha whatever this
        // sweep says. What the sweep buys is the number a successor needs — the smallest
        // step this gate can resolve on this scene — so that "the plant did not fire" is a
        // quantity instead of a shrug.
        println!(
            "# G4 POWER CERTIFICATE (the staked plant did not fire; this is what the gate \
             CAN see, not a new criterion):"
        );
        let mut first: Option<f64> = None;
        for k in 1..=6 {
            let step = holon_render::longrange::PLANT_STEP_HARTREE * 10f64.powi(k);
            let mut sim = build_scene(cfg, curves);
            let mut far =
                FarSector::build(tails, r_s, budget, sim.dims).expect("the far sector builds");
            far.plant = Some(FarPlant::ZeroPointStep);
            far.plant_step = step;
            sim.far = Some(Box::new(far));
            sim.rebase();
            let frames = (steps / GRAIN_SUBSTEPS as u64).max(1);
            for _ in 0..frames {
                for _ in 0..GRAIN_SUBSTEPS {
                    sim.step();
                }
                sim.close_grain();
            }
            let ok = sim.energy_gate();
            println!(
                "P4-power step {step:.1e} Ha  drift_peak {:.6e}  bound {:.6e}  G4 {}",
                sim.drift_peak,
                sim.drift_bound(),
                pf(ok)
            );
            if !ok && first.is_none() {
                first = Some(step);
            }
        }
        match first {
            Some(s) => println!(
                "# G4 resolves a zero-point step at {s:.1e} Ha and not at the staked \
                 {:.1e} Ha — a factor of {:.0}. V2 applies at the staked plant: G4's arm \
                 is VOID until a successor stakes a plant this gate can see.",
                holon_render::longrange::PLANT_STEP_HARTREE,
                s / holon_render::longrange::PLANT_STEP_HARTREE
            ),
            None => println!(
                "# G4 resolved NO step up to {:.1e} Ha. V2 applies and the gate's power is \
                 not established at any scale this sweep reached.",
                holon_render::longrange::PLANT_STEP_HARTREE * 1.0e6
            ),
        }
    }
}

fn plant_p5(
    cfg: &SceneCfg,
    curves: &[(Species, Species)],
    tails: &[Option<CurveTail>],
    r_s: f64,
    budget: f64,
    clean: f64,
) {
    let sim = build_scene(cfg, curves);
    let mut far = FarSector::build(tails, r_s, budget, Dims::Two).expect("builds");
    far.plant = Some(FarPlant::GradientMismatch);
    let geom = BoxGeom::new(sim.width, sim.height, sim.depth, false);
    let slots: Vec<usize> = (0..sim.n)
        .map(|i| sim.bank.index_of(sim.atoms[i].species.z).unwrap_or(0))
        .collect();
    let r_max: Vec<f64> = tails
        .iter()
        .map(|c| c.as_ref().map(CurveTail::r_max).unwrap_or(0.0))
        .collect();
    let base: Vec<(f64, f64, f64)> = (0..sim.n)
        .map(|i| (sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z))
        .collect();
    let h = 1.0e-5;
    let mut worst = 0.0f64;
    for i in 0..sim.n {
        let mut fx = vec![(0.0, 0.0, 0.0); sim.n];
        far.accumulate(&base, &slots, geom, &mut fx, &r_max);
        let analytic = fx[i].0;
        let (mut p, mut m) = (base.clone(), base.clone());
        p[i].0 += h;
        m[i].0 -= h;
        let numeric = -(far.energy_at_shells(&p, &slots, geom, 0)
            - far.energy_at_shells(&m, &slots, geom, 0))
            / (2.0 * h);
        let scale = analytic.abs().max(numeric.abs()).max(1.0e-30);
        worst = worst.max((analytic - numeric).abs() / scale);
    }
    println!(
        "P5 gradient mismatch (force x{}) | carrier = relative gradient error {worst:.4e} \
         (sector: the far force) | clean run was {clean:.4e} | G8 {} | verdict {}",
        holon_render::longrange::PLANT_FORCE_SCALE,
        pf(worst <= G8_TOL),
        if worst == 0.0 {
            "REFUSED — carrier reads 0.0"
        } else if worst > G8_TOL {
            "FIRED"
        } else {
            "DID NOT FIRE"
        }
    );
}

fn plant_p6(
    cfg: &SceneCfg,
    curves: &[(Species, Species)],
    tails: &[Option<CurveTail>],
    r_s: f64,
    budget: f64,
) {
    let mut clean = build_scene(cfg, curves);
    attach_far(&mut clean, tails, r_s, budget, None);
    let e_clean = clean.e_far;
    let mut cut = build_scene(cfg, curves);
    attach_far(&mut cut, tails, r_s, budget, Some(FarPlant::TruncatedFarSum));
    let e_cut = cut.e_far;
    let carrier = (e_clean - e_cut).abs();
    println!(
        "P6 truncated far sum (R_s + {} bohr) | carrier = the omitted band's energy \
         {carrier:.6e} Ha (sector: the far pair sector) | E_far clean {e_clean:.6e} vs \
         truncated {e_cut:.6e} | verdict {}",
        holon_render::longrange::PLANT_TRUNCATION_BOHR,
        if carrier == 0.0 {
            "REFUSED — carrier reads 0.0, nothing lies in the omitted band"
        } else {
            "FIRED"
        }
    );
}

fn plant_p7(
    cfg: &SceneCfg,
    curves: &[(Species, Species)],
    tails: &[Option<CurveTail>],
    r_s: f64,
    budget: f64,
    clean: f64,
) {
    let mut sim = build_scene(cfg, curves);
    attach_far(&mut sim, tails, r_s, budget, Some(FarPlant::OmittedVirial));
    let virial = sim.w_virial;
    // THE FINEST staked step, not the coarsest. At h = 1e-3 the central difference's own
    // O(h^2) truncation error is the same size as the far sector's whole virial share, so
    // the coarse step cannot separate the planted defect from the differencing — measured,
    // not assumed: the clean run reads 1.35e-4 there and the plant 1.86e-4, a factor of
    // 1.4. At the finest step the clean reading is nine orders below the plant.
    let h = G7_STEPS[G7_STEPS.len() - 1];
    let mut sp = build_scene(cfg, curves);
    attach_far(&mut sp, tails, r_s, budget, Some(FarPlant::OmittedVirial));
    sp.scale_box(1.0 + h).expect("scales");
    let ep = sp.energy() - sp.e_kin;
    let mut sm = build_scene(cfg, curves);
    attach_far(&mut sm, tails, r_s, budget, Some(FarPlant::OmittedVirial));
    sm.scale_box(1.0 - h).expect("scales");
    let em = sm.energy() - sm.e_kin;
    let d = (ep - em) / (2.0 * h);
    let rel = (d - virial).abs() / virial.abs().max(1.0e-30);
    // The other half of the staked observation: a channel can be perfectly conservative and
    // still be missing from the pressure, so G4 and G5 must stay green while G7 fires.
    let run = conservation_run(cfg, curves, tails, r_s, budget, 2000, Some(FarPlant::OmittedVirial));
    println!(
        "P7 omitted virial | carrier = the far sector's Sum r du/dr, {:.6e} Ha (sector: the \
         virial) | G7 relative {rel:.4e} vs clean {clean:.4e} | G7 {} | G4 {} | G5 {} | \
         verdict {}",
        (d - virial).abs(),
        pf(rel <= G7_TOL),
        pf(run.energy_ok),
        pf(run.momentum_ok),
        if (d - virial).abs() == 0.0 {
            "REFUSED — carrier reads 0.0"
        } else if rel > G7_TOL && run.energy_ok && run.momentum_ok {
            "FIRED, and G4/G5 STAYED GREEN — a conservative channel missing from the pressure"
        } else if rel > G7_TOL {
            "FIRED, but a conservation gate moved too"
        } else {
            "DID NOT FIRE"
        }
    );
}

// ---------------------------------------------------------------- G9, G10, P1

/// The periodic arm: image convergence, the barostat cache seam, and P1.
///
/// It runs at the SAME declared budget as the arm it belongs to. A separate, looser
/// periodic budget was staked here first, on the reasoning that a 20.8-bohr box's first
/// image shell sits where an `r^-6` tail is still worth ~5e-7 Ha. The measurement inverted
/// that reasoning: G3 reads `p_fit = 20.7` on this curve, not 6, and at `p ≈ 21` a looser
/// budget COLLAPSES `R_f` onto `R_s` — the far window becomes empty and every gate in this
/// arm scores a zero against a zero. The constant was chosen from an assumed tail and the
/// measured tail contradicted it, which is the whole reason G3 runs before anything uses a
/// tail model.
///
/// A wrapping box is REQUIRED here and not a convenience. In a walled box the image list is
/// empty, so P1's carrier — the difference a stale image lattice makes — is exactly zero and
/// the plant would be scored on a scene it cannot act in. That is M-PLANT-SECTOR, and it is
/// why this arm exists separately from the open-box one.
fn periodic_arm(
    curves: &[(Species, Species)],
    tails: &[Option<CurveTail>],
    r_s: f64,
    budget: f64,
) {
    // THE BOX IS SIZED FROM `R_f`, and that is a correctness condition rather than a
    // convenience. On the census box (34.6 x 20.8) the nearest image sits at 20.8 bohr
    // while a steep tail's `R_f` is 11.2, so EVERY image contribution is an exact zero:
    // G9 would compare 0.0 against 0.0, G10 would converge at one shell with a difference
    // of 0.0, and P1's carrier would read 0.0 in a sector it cannot act in. That is a
    // vacuous success wearing a green gate, which is the failure M-VACUOUS-SUCCESS names,
    // and the first run of this instrument produced all three of them.
    let probe = FarSector::build(tails, r_s, budget, Dims::Two);
    let edge = match &probe {
        Ok(f) => 0.75 * f.r_f(),
        Err(_) => BOX_H,
    };
    let cfg = SceneCfg {
        n: 12,
        oxygen_every: if curves.len() > 1 { 3 } else { 0 },
        boundary: Boundary::Periodic,
        w: edge,
        h: edge,
        prescale: 1.0,
        three_d: false,
    };
    println!("# periodic arm: box {edge:.4} x {edge:.4} bohr, sized at 0.75 R_f so the image sector is nonempty");
    let sim = build_scene(&cfg, curves);
    let geom = BoxGeom::new(sim.width, sim.height, sim.depth, true);
    let slots: Vec<usize> = (0..sim.n)
        .map(|i| sim.bank.index_of(sim.atoms[i].species.z).unwrap_or(0))
        .collect();
    let pos: Vec<(f64, f64, f64)> = (0..sim.n)
        .map(|i| (sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z))
        .collect();

    let mut far = match FarSector::build(tails, r_s, budget, Dims::Two) {
        Ok(f) => f,
        Err(e) => {
            println!("# periodic arm VOID: {e}");
            return;
        }
    };
    // G10 — image convergence, and its uncertainty DISCLOSED rather than discarded.
    match far.resolve_shells(&pos, &slots, geom) {
        Ok((m, diff)) => {
            println!(
                "# GATE G10 image convergence: PASS  shells {m} (cap {SHELL_CAP})  \
                 |E(m+1) − E(m)| = {diff:.6e} Ha  budget {budget:.3e} Ha  \
                 uncertainty_hartree {diff:.6e}"
            );
        }
        Err(e) => {
            println!("# GATE G10 image convergence: FAIL — {e}");
            println!("# R2 fired; the periodic arm is VOID and G9/P1 are not scored.");
            return;
        }
    }
    let shells = far.shells();
    // M-PLANT-SECTOR, mechanized: the sector the arm's gates act in must be nonempty
    // BEFORE any of them is scored.
    let mut probe_f = vec![(0.0, 0.0, 0.0); sim.n];
    let r_max_probe: Vec<f64> = tails
        .iter()
        .map(|c| c.as_ref().map(CurveTail::r_max).unwrap_or(0.0))
        .collect();
    let probe_read = far.accumulate(&pos, &slots, geom, &mut probe_f, &r_max_probe);
    println!(
        "# periodic arm image sector: {} of {} far contributions come from a nonzero image \
         offset; E_far = {:.6e} Ha",
        probe_read.image_contributions, probe_read.contributions, probe_read.energy
    );
    if probe_read.image_contributions == 0 {
        println!(
            "# VOID (V2): the image sector is EMPTY, so G9, G10 and P1 would all be scored \
             on carriers that cannot be nonzero. Not scored."
        );
        return;
    }

    // G9 — nothing box-derived goes stale. A rescaled sim against one built fresh at the
    // scaled box, compared BITWISE within one device class.
    for f in [0.90f64, 1.10f64] {
        let mut scaled = build_scene(&cfg, curves);
        let mut fs = FarSector::build(tails, r_s, budget, Dims::Two).expect("builds");
        let _ = fs.resolve_shells(&pos, &slots, geom);
        scaled.far = Some(Box::new(fs));
        scaled.rebase();
        scaled.scale_box(f).expect("the box scales");

        let fresh_cfg = SceneCfg { prescale: f, ..cfg_clone(&cfg) };
        let mut fresh = build_scene(&fresh_cfg, curves);
        let fgeom = BoxGeom::new(fresh.width, fresh.height, fresh.depth, true);
        let fpos: Vec<(f64, f64, f64)> = (0..fresh.n)
            .map(|i| (fresh.atoms[i].x, fresh.atoms[i].y, fresh.atoms[i].z))
            .collect();
        let mut ffs = FarSector::build(tails, r_s, budget, Dims::Two).expect("builds");
        let _ = ffs.resolve_shells(&fpos, &slots, fgeom);
        fresh.far = Some(Box::new(ffs));
        fresh.rebase();

        let e_ok = scaled.e_far.to_bits() == fresh.e_far.to_bits();
        let v_ok = scaled.w_virial.to_bits() == fresh.w_virial.to_bits();
        let mut f_ok = true;
        for i in 0..scaled.n {
            let (a, b) = (scaled.internal_force(i), fresh.internal_force(i));
            f_ok &= a.0.to_bits() == b.0.to_bits()
                && a.1.to_bits() == b.1.to_bits()
                && a.2.to_bits() == b.2.to_bits();
        }
        println!(
            "# GATE G9 stale-cache f={f:.2}: {}  e_far bit-identical {e_ok}  virial {v_ok}  \
             every force {f_ok}  (scaled {:.12e} vs fresh {:.12e})",
            pf(e_ok && v_ok && f_ok),
            scaled.e_far,
            fresh.e_far
        );
    }

    // P1 — the stale lattice. The carrier is the difference a skipped recomputation makes,
    // and it is nonzero in the image sector because scaling moves every image offset.
    let mut stale = build_scene(&cfg, curves);
    let mut sfar = FarSector::build(tails, r_s, budget, Dims::Two).expect("builds");
    let _ = sfar.resolve_shells(&pos, &slots, geom);
    sfar.plant = Some(FarPlant::StaleLattice);
    stale.far = Some(Box::new(sfar));
    stale.rebase();
    stale.scale_box(0.90).expect("scales");

    let fresh_cfg = SceneCfg { prescale: 0.90, ..cfg_clone(&cfg) };
    let mut fresh = build_scene(&fresh_cfg, curves);
    let fgeom = BoxGeom::new(fresh.width, fresh.height, fresh.depth, true);
    let fpos: Vec<(f64, f64, f64)> = (0..fresh.n)
        .map(|i| (fresh.atoms[i].x, fresh.atoms[i].y, fresh.atoms[i].z))
        .collect();
    let mut ffs = FarSector::build(tails, r_s, budget, Dims::Two).expect("builds");
    let _ = ffs.resolve_shells(&fpos, &slots, fgeom);
    fresh.far = Some(Box::new(ffs));
    fresh.rebase();
    let carrier = (stale.e_far - fresh.e_far).abs();
    println!(
        "P1 stale image lattice (f = 0.90, {shells} shells) | carrier |E_far(stale) − \
         E_far(fresh)| = {carrier:.6e} Ha (sector: the periodic image sector) | verdict {}",
        if carrier == 0.0 {
            "REFUSED — carrier reads 0.0; this box has no images for the plant to act in"
        } else {
            "FIRED — G9 sees it"
        }
    );
}

fn cfg_clone(c: &SceneCfg) -> SceneCfg {
    SceneCfg {
        n: c.n,
        oxygen_every: c.oxygen_every,
        boundary: c.boundary,
        w: c.w,
        h: c.h,
        prescale: c.prescale,
        three_d: c.three_d,
    }
}

// ---------------------------------------------------------------- G13

/// G13 — the cost curve, MEASURED and not gated on a value.
///
/// No prior record carries an N-scaling curve for this engine's pair sector, and a freeze
/// may not gate a number it would have to invent (M-UNTESTED-GAP). What IS gated is the
/// ordering: a cost that comes back non-monotone in N convicts the measurement outright,
/// with no baseline and no second run needed — which is M-PLACEMENT-LOTTERY's fourth
/// instance turned into a check.
fn gate_g13(curves: &[(Species, Species)], tails: &[Option<CurveTail>], r_s: f64, budget: f64) {
    let cpu = cpu_now();
    println!(
        "# G13 conditions: cpu {cpu}  core class {}  clock fraction {:.3}  loadavg {:.2}",
        core_class(cpu),
        clock_fraction(cpu),
        loadavg()
    );
    println!("COLUMNS G13 n cpu_seconds_mean cpu_seconds_min cpu_seconds_max spread_factor");
    let mut rows: Vec<(usize, f64)> = Vec::new();
    let density = 12.0 / (BOX_W * BOX_H);
    for &n in &G13_SIZES {
        // FIXED NUMBER DENSITY: the box grows with N, so what is measured is the sum's
        // scaling and not a scene getting denser.
        let area = n as f64 / density;
        let aspect = BOX_W / BOX_H;
        let h = (area / aspect).sqrt();
        let w = aspect * h;
        let cfg = SceneCfg {
            n,
            oxygen_every: if curves.len() > 1 { 3 } else { 0 },
            boundary: Boundary::Open,
            w,
            h,
            prescale: 1.0,
            three_d: false,
        };
        let sim = build_scene(&cfg, curves);
        let mut far = FarSector::build(tails, r_s, budget, Dims::Two).expect("builds");
        let geom = BoxGeom::new(sim.width, sim.height, sim.depth, false);
        let slots: Vec<usize> = (0..n)
            .map(|i| sim.bank.index_of(sim.atoms[i].species.z).unwrap_or(0))
            .collect();
        let pos: Vec<(f64, f64, f64)> = (0..n)
            .map(|i| (sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z))
            .collect();
        let r_max: Vec<f64> = tails
            .iter()
            .map(|c| c.as_ref().map(CurveTail::r_max).unwrap_or(0.0))
            .collect();
        let mut times = Vec::new();
        for _ in 0..G13_REPS {
            let mut f = vec![(0.0, 0.0, 0.0); n];
            // ADAPTIVE, because the clock tick is 10 ms and one far pass over 12 atoms is
            // microseconds: a fixed repetition count reads 0.0, and 0.0 is not a time. The
            // first run of this instrument returned five sizes of which three read exactly
            // zero and the ordering came back non-monotone, which V6 correctly convicted.
            let mut reps = 1usize;
            let elapsed = loop {
                let t0 = cpu_seconds();
                for _ in 0..reps {
                    far.accumulate(&pos, &slots, geom, &mut f, &r_max);
                }
                let e = cpu_seconds() - t0;
                if e >= G13_MIN_CPU_S || reps >= 1 << 24 {
                    break e / reps as f64;
                }
                reps *= 4;
            };
            times.push(elapsed);
        }
        let mean = times.iter().sum::<f64>() / times.len() as f64;
        let lo = times.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = times.iter().copied().fold(0.0f64, f64::max);
        println!(
            "G13 {n} {mean:.6e} {lo:.6e} {hi:.6e} {:.3}",
            if lo > 0.0 { hi / lo } else { f64::NAN }
        );
        rows.push((n, mean));
    }
    let monotone = rows.windows(2).all(|w| w[1].1 >= w[0].1);
    // Least squares on (ln N, ln t): the exponent, with the caveat that it is a reading and
    // not a gate.
    let (mut sx, mut sy, mut sxx, mut sxy, mut k) = (0.0, 0.0, 0.0, 0.0, 0.0f64);
    for (n, t) in &rows {
        if *t <= 0.0 {
            continue;
        }
        let (x, y) = ((*n as f64).ln(), t.ln());
        sx += x;
        sy += y;
        sxx += x * x;
        sxy += x * y;
        k += 1.0;
    }
    let slope = if k >= 2.0 {
        (k * sxy - sx * sy) / (k * sxx - sx * sx)
    } else {
        f64::NAN
    };
    println!(
        "# GATE G13: exponent {slope:.3} over N = {:?}, {G13_REPS} repetitions each. \
         PRINTED, NOT GATED ON A VALUE. Ordering monotone in N: {monotone} — {}",
        G13_SIZES,
        if monotone {
            "the measurement stands"
        } else {
            "V6: a non-monotone cost convicts the measurement, not the engine"
        }
    );
}

// ====================================================================== the refusals arm

/// G11 — every refusal constructed and observed to fire. A scope fence nobody has seen fire
/// is a comment.
/// What a refusal-probe actually returned, in one line. `FarSector` carries an image
/// lattice and a model per slot and is deliberately not `Debug`; what a G11 row needs is
/// the refusal or the fact that there wasn't one.
fn verdict_of(r: &Result<FarSector, FarRefusal>) -> String {
    match r {
        Ok(_) => "built a sector where a refusal was required".to_string(),
        Err(e) => format!("refused, but with the wrong refusal: {e}"),
    }
}

fn arm_refusals() {
    let mut fired = 0usize;
    let mut total = 0usize;

    // A synthetic power-law curve, so the refusals are exercised without paying for a solve.
    let power = |p: f64, r_max: f64| -> CurveTail {
        let n = 40usize;
        let r: Vec<f64> = (0..n)
            .map(|k| 4.0 + (r_max - 4.0) * k as f64 / (n - 1) as f64)
            .collect();
        let u: Vec<f64> = r.iter().map(|x| -1.0 * x.powf(-p)).collect();
        CurveTail {
            hi_b: p / r_max,
            r,
            u,
            solver_exit: "Converged",
            solver_budget_iterations: 5000,
            uncertainty_hartree: 1.0e-11,
        }
    };

    // R3 — the sub-support refusal. This is the one that would have fired on the exact
    // configuration B1b audited: R_s = 15.0 against the O–O curve's r_max = 20.0.
    total += 1;
    let c = vec![Some(power(6.0, 20.0))];
    match FarSector::build(&c, 15.0, 1.0e-9, Dims::Two) {
        Err(e @ FarRefusal::SubSupport { .. }) => {
            fired += 1;
            println!("R3 FIRED: {e}");
        }
        other => println!("R3 DID NOT FIRE: {}", verdict_of(&other)),
    }

    // R1a — the exponent refusal. p = 1 is the ionic case and it fails in 2D and 3D alike.
    for (d, dims) in [(2usize, Dims::Two), (3usize, Dims::Three)] {
        total += 1;
        let c = vec![Some(power(1.0, 20.0))];
        match FarSector::build(&c, 20.0, 1.0e-9, dims) {
            Err(e @ FarRefusal::ExponentTooShallow { .. }) => {
                fired += 1;
                println!("R1a(d={d}) FIRED: {e}");
            }
            other => println!("R1a(d={d}) DID NOT FIRE: {}", verdict_of(&other)),
        }
    }

    // R1b — the charged-scene refusal.
    total += 1;
    match FarSector::admit_charge(1.0) {
        Err(e @ FarRefusal::ChargedScene { .. }) => {
            fired += 1;
            println!("R1b FIRED: {e}");
        }
        other => println!("R1b DID NOT FIRE: {other:?}"),
    }

    // R5 — the disclosure refusal. A tail parameter without its solve's exit or budget.
    for (label, exit, budget) in [("no exit", "", 5000u64), ("no budget", "Converged", 0u64)] {
        total += 1;
        let mut t = power(6.0, 20.0);
        t.solver_exit = if exit.is_empty() { "" } else { "Converged" };
        t.solver_budget_iterations = budget;
        match FarSector::build(&vec![Some(t)], 20.0, 1.0e-9, Dims::Two) {
            Err(e @ FarRefusal::UndisclosedSolve { .. }) => {
                fired += 1;
                println!("R5 ({label}) FIRED: {e}");
            }
            other => println!("R5 ({label}) DID NOT FIRE: {}", verdict_of(&other)),
        }
    }

    // R4 — the fenced-tail refusal. An exponent outside the band fences the sector, and a
    // caller asking for a scalar gets the bracket.
    total += 1;
    let c = vec![Some(power(12.0, 20.0))];
    match FarSector::build(&c, 20.0, 1.0e-9, Dims::Two) {
        Ok(f) if f.is_fenced() => match f.scalar_ok(-1.0e-6, -1.0e-9) {
            Err(e @ FarRefusal::FencedTailScalar { .. }) => {
                fired += 1;
                println!("R4 FIRED: {e}");
            }
            other => println!("R4 DID NOT FIRE: {other:?}"),
        },
        other => println!("R4 DID NOT FIRE (sector not fenced): {:?}", other.map(|f| f.is_fenced())),
    }

    // R2 — the image-budget refusal, on a box whose shells cannot reach an impossible
    // budget inside the cap.
    total += 1;
    let c = vec![Some(power(3.0, 20.0))];
    let mut f = FarSector::build(&c, 20.0, 1.0e-30, Dims::Two).expect("builds");
    let pos = [(0.0, 0.0, 0.0), (5.0, 0.0, 0.0)];
    let geom = BoxGeom::new(BOX_W, BOX_H, BOX_H, true);
    match f.resolve_shells(&pos, &[0, 0], geom) {
        Err(e @ FarRefusal::ImageBudget { .. }) => {
            fired += 1;
            println!("R2 FIRED: {e}");
        }
        other => println!("R2 DID NOT FIRE: {other:?}"),
    }

    println!("# GATE G11: {fired} of {total} refusals fired. {}", pf(fired == total));
}

// ====================================================================== the frames arm

/// One frame's channel split at `c*`, on the estimator B1b gated.
#[derive(Default, Clone, Copy)]
struct Split {
    /// `E_switch(c*)` — B1b's gated quantity, reproduced here.
    switched: f64,
    /// Channel S: the switched contribution of pairs with `c* < r ≤ r_max`.
    s: f64,
    /// Channel T: the switched contribution of pairs past `r_max`.
    t: f64,
    /// Pairs in `(c*, r_max]` — the sub-support population, and the near sector's job.
    sub_support_pairs: usize,
    /// Pairs in `(c*, r_max]` that a list built at `R_s` would MISS. EXACT 0 required (G2).
    missed_pairs: usize,
    /// WHAT THE NEW SUBSYSTEM ACTUALLY DISCARDS on this frame: the table's own value summed
    /// over pairs past `R_f`, where neither the near sector nor the far sum reaches.
    beyond_rf: f64,
    /// Where the tail model DISAGREES with the extrapolation it replaces, summed over
    /// `(R_s, R_f]`. Not a discard — the far sector carries these pairs — but it is the
    /// model's own uncertainty on them, and G14 adds it because a model that carries a pair
    /// wrongly has not paid for it either.
    model_gap: f64,
}

#[allow(clippy::too_many_arguments)]
fn split_frame(
    pos: &[[f64; 3]],
    zidx: &[usize],
    slot: &[Vec<usize>],
    rmax: &[Vec<f64>],
    bank: &holon_render::bank::PairBank,
    r_s: f64,
    far: &FarSector,
) -> Split {
    let n = pos.len();
    let mut out = Split::default();
    let r_in = C_STAR - PAIR_SWITCH_WIDTH;
    for i in 0..n {
        for j in (i + 1)..n {
            let d = [
                pos[j][0] - pos[i][0],
                pos[j][1] - pos[i][1],
                pos[j][2] - pos[i][2],
            ];
            let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let (a, b) = (zidx[i], zidx[j]);
            let t = bank.table_slot(slot[a][b]);
            let rm = rmax[a][b];
            if r > C_STAR && r <= rm {
                out.sub_support_pairs += 1;
                if r > r_s {
                    out.missed_pairs += 1;
                }
            }
            if r > r_in {
                let (sw, _, _) = switch_c2(r, r_in, C_STAR);
                let removed = (1.0 - sw) * t.u(r);
                out.switched += removed;
                if r <= rm {
                    out.s += removed;
                } else {
                    out.t += removed;
                }
            }
            // WHAT B2 LEAVES BEHIND, per frame. Accumulated here rather than differenced
            // from three maxima afterwards: those maxima fall on different frames, and a
            // max of a part is not a part of a max — the freeze says so about B1b's own
            // numbers and it would be a poor thing to then do it here.
            if r > far.r_f() {
                out.beyond_rf += t.u(r);
            } else if r > r_s {
                if let Some(m) = far.model(slot[a][b]) {
                    out.model_gap += m.eval(r).0 - t.u(r);
                }
            }
        }
    }
    out
}

#[allow(clippy::too_many_lines)]
fn arm_frames(root: &Path, manifest_path: &Path, stride: usize) {
    // The curves, through the same door B1b used.
    let mut sim = Box::new(Sim::empty());
    sim.boundary = Boundary::Walls;
    sim.dims = Dims::Two;
    sim.width = BOX_W;
    sim.height = BOX_H;
    let mut meta: BTreeMap<usize, (&'static str, u64, f64)> = BTreeMap::new();
    let t_hh;
    let mut t_oo = f64::NAN;
    {
        let t0 = cpu_seconds();
        let pt = generate_pair_table(HYDROGEN, HYDROGEN, CURVE_KNOTS);
        t_hh = cpu_seconds() - t0;
        assert_eq!(load_pair_table(&mut sim, &pt, Host::Native), TABLE_OK);
        record_meta(&mut meta, &sim, HYDROGEN, HYDROGEN, &pt);
    }
    for (a, b) in [(OXYGEN, HYDROGEN), (OXYGEN, OXYGEN)] {
        let t0 = cpu_seconds();
        let pt = generate_pair_table(a, b, CURVE_KNOTS);
        let secs = cpu_seconds() - t0;
        if a == OXYGEN && b == OXYGEN {
            t_oo = secs;
        }
        assert_eq!(load_pair_table(&mut sim, &pt, Host::Native), TABLE_OK);
        record_meta(&mut meta, &sim, a, b, &pt);
    }
    // B1b's W2, inherited unchanged. B2 generates no new curves and consumes B1b's, so its
    // price evidence is B1b's price evidence.
    println!(
        "# W2 (inherited from B1b): t(O-O)/t(H-H) = {t_oo:.1}/{t_hh:.1} = {:.1} against \
         floor 100",
        t_oo / t_hh
    );

    let tails = curve_tails(&sim, &meta);
    let (_adopting, fenced) = report_g3(&tails);
    let r_s = tails.iter().flatten().map(|c| c.r_max()).fold(0.0f64, f64::max);
    let far = match FarSector::build(&tails, r_s, DEFAULT_BUDGET, Dims::Two) {
        Ok(f) => f,
        Err(e) => {
            println!("VOID frames arm — the far sector refused to build: {e}");
            return;
        }
    };
    println!(
        "# R_s = {r_s:.4} bohr; c* = {C_STAR} bohr; R_f = {:.4} bohr; budget \
         {DEFAULT_BUDGET:.3e} Ha; fenced = {fenced} curve(s)",
        far.r_f()
    );
    for s in 0..holon_render::bank::MAX_TABLES {
        if let Some(m) = far.model(s) {
            // R5's fields travel with the parameter, every time it is quoted.
            println!(
                "# tail slot {s}: p {:.4}  C_p {:.6e}  solver_exit {}  \
                 solver_budget_iterations {}  uncertainty_hartree {:.3e}",
                m.p, m.c_p, m.solver_exit, m.solver_budget_iterations, m.uncertainty_hartree
            );
        }
    }

    // The manifest refusal, reused verbatim from B1b: the digests are the same file's.
    let Ok(manifest_text) = std::fs::read_to_string(manifest_path) else {
        println!("VOID frames arm — the committed manifest does not open");
        return;
    };
    let mut want: BTreeMap<String, String> = BTreeMap::new();
    for line in manifest_text.lines() {
        let mut it = line.split_whitespace();
        if let (Some(h), Some(p)) = (it.next(), it.next()) {
            want.insert(p.trim_start_matches("./").to_string(), h.to_string());
        }
    }
    println!("# manifest = {} ({} entries)", manifest_path.display(), want.len());

    let dir = root.join("fenced");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "traj"))
                .collect()
        })
        .unwrap_or_default();
    paths.sort();
    println!("# parked .traj in {}: {}", dir.display(), paths.len());

    let mut zs: Vec<u32> = vec![];
    let mut slot: Vec<Vec<usize>> = vec![];
    let mut rmax: Vec<Vec<f64>> = vec![];
    let mut per_seed: Vec<(u64, f64, f64, f64, usize, usize, usize, f64, u64, f64, f64)> =
        Vec::new();
    let mut frames_scored = 0usize;
    let mut worst = (0.0f64, 0u64, 0u64, Split::default());
    let mut worst_s = (0.0f64, 0u64);
    let mut worst_t = (0.0f64, 0u64);
    let mut missed_total = 0usize;
    let mut refusals = 0usize;

    for p in &paths {
        let key = format!(
            "census-traj/fenced/{}",
            p.file_name().unwrap().to_string_lossy()
        );
        // M-PROVENANCE-OVERREACH: the refusal names the FILE it hashed and infers nothing
        // about which run produced it.
        if !want.contains_key(&key) {
            println!("# REFUSED {key}: the manifest does not list this path");
            refusals += 1;
            continue;
        }
        let Ok(traj) = Trajectory::read(p) else {
            println!("# REFUSED {key}: unreadable");
            refusals += 1;
            continue;
        };
        if zs.is_empty() {
            zs = traj.header.z.clone();
            zs.sort_unstable();
            zs.dedup();
            let nz = zs.len();
            slot = vec![vec![0; nz]; nz];
            rmax = vec![vec![0.0; nz]; nz];
            for a in 0..nz {
                for b in 0..nz {
                    let s = sim.bank.slot_of_z(zs[a], zs[b]).expect("a loaded slot");
                    slot[a][b] = s;
                    rmax[a][b] = sim.bank.table_slot(s).r_max();
                }
            }
        }
        let zidx: Vec<usize> = traj
            .header
            .z
            .iter()
            .map(|z| zs.iter().position(|v| v == z).unwrap())
            .collect();
        let seed = traj.header.seed;
        let (mut mx, mut ms, mut mt, mut mframe) = (0.0f64, 0.0f64, 0.0f64, 0u64);
        let (mut sub, mut miss, mut cnt) = (0usize, 0usize, 0usize);
        // The residual is a max over frames of a PER-FRAME quantity, and every frame is in
        // the max whatever else it did (M-MAX-OVER-SUCCESSES).
        let (mut mres, mut mres_frame, mut mbeyond, mut mgap) = (0.0f64, 0u64, 0.0f64, 0.0f64);
        for f in traj.frames.iter().step_by(stride.max(1)) {
            let sp = split_frame(&f.pos, &zidx, &slot, &rmax, &sim.bank, r_s, &far);
            cnt += 1;
            sub += sp.sub_support_pairs;
            miss += sp.missed_pairs;
            if sp.switched.abs() > mx {
                mx = sp.switched.abs();
                mframe = f.index;
                if mx > worst.0 {
                    worst = (mx, seed, f.index, sp);
                }
            }
            if sp.s.abs() > ms {
                ms = sp.s.abs();
                if ms > worst_s.0 {
                    worst_s = (ms, seed);
                }
            }
            if sp.t.abs() > mt {
                mt = sp.t.abs();
                if mt > worst_t.0 {
                    worst_t = (mt, seed);
                }
            }
            let res = sp.beyond_rf.abs() + sp.model_gap.abs();
            if res > mres {
                mres = res;
                mres_frame = f.index;
                mbeyond = sp.beyond_rf;
                mgap = sp.model_gap;
            }
        }
        frames_scored += cnt;
        missed_total += miss;
        per_seed.push((seed, mx, ms, mt, cnt, sub, miss, mres, mres_frame, mbeyond, mgap));
        println!(
            "SEED {seed:#018x} max|E_switch(c*)| {mx:.6e} at frame {mframe}  \
             max|S| {ms:.6e}  max|T| {mt:.6e}  frames {cnt}  sub-support pairs {sub}  \
             missed-by-R_s {miss}"
        );
    }

    if per_seed.is_empty() {
        println!("VOID frames arm — V1: no admitted trajectory.");
        return;
    }

    // ---- G1
    let (ws, wt) = (worst.3.s.abs(), worst.3.t.abs());
    let ratio_st = if wt > 0.0 { ws / wt } else { f64::INFINITY };
    let verdict = if ratio_st >= G1_DOMINANCE {
        "S-DOMINANT (branch a): a RADIUS-BOOKKEEPING defect"
    } else if ratio_st <= 1.0 / G1_DOMINANCE {
        "T-DOMINANT (branch b): the extrapolation form is the finding"
    } else {
        "MIXED (branch c): both channels load-bearing, neither fix alone discharges B1b"
    };
    println!(
        "# GATE G1 channel split at the worst frame (seed {:#018x}, frame {}): \
         E_switch {:.6e}  S {:.6e}  T {:.6e}  signed sum {:.6e}  S/T {ratio_st:.4} \
         (dominance {G1_DOMINANCE}) — {verdict}",
        worst.1, worst.2, worst.3.switched, worst.3.s, worst.3.t, worst.3.s + worst.3.t
    );
    println!(
        "# G1 also, per M-LOOP-BLIND: max|S| over ALL frames {:.6e} (seed {:#018x}); \
         max|T| {:.6e} (seed {:#018x}). The two channels are reported separately because a \
         marginal over separation can hide two contributions of opposite sign.",
        worst_s.0, worst_s.1, worst_t.0, worst_t.1
    );

    // ---- G2
    println!(
        "# GATE G2 near-sector coverage: {}  pairs in (c*, r_max] missed by a list at \
         R_s = {r_s:.4}: {missed_total} (EXACT 0 required)",
        pf(missed_total == 0)
    );

    // ---- G14
    println!(
        "COLUMNS G14 seed max_switch 0.10Bs 0.10Ds b1b_ratio residual_after at_frame \
         beyond_Rf model_gap b2_ratio paid"
    );
    let mut unpaid = 0usize;
    for (seed, mx, _ms, _mt, _, _, _, mres, mres_frame, mbeyond, mgap) in &per_seed {
        let Some(&(_, b10, d10, b1b)) = B1B_MIXED.iter().find(|r| r.0 == *seed) else {
            println!("G14 {seed:#018x} NO STAKED B1b ROW — refusing to grade an unstaked seed");
            unpaid += 1;
            continue;
        };
        // WHAT THE SUBSYSTEM LEAVES BEHIND, and it is a per-frame quantity maximised over
        // frames rather than a difference of three maxima that fall on different frames.
        // Two parts, both counted: pairs past `R_f`, which nothing reaches, and the tail
        // model's disagreement with the extrapolation it replaces on the pairs it does
        // carry — a pair carried wrongly has not been paid for either.
        let ratio = mres / d10;
        let paid = ratio < 1.0;
        if !paid {
            unpaid += 1;
        }
        println!(
            "G14 {seed:#018x} {mx:.6e} {b10:.3e} {d10:.3e} {b1b:.3} {mres:.6e} \
             {mres_frame} {mbeyond:.6e} {mgap:.6e} {ratio:.4} {paid}"
        );
    }
    println!(
        "# GATE G14: {}  {} of {} seeds still over the criterion. B1b's fired G1b stays \
         fired: 3 of 8 seeds, worst 2.496x.",
        pf(unpaid == 0),
        unpaid,
        per_seed.len()
    );

    // ---- G12
    println!(
        "# GATE G12 work count: frames scored {frames_scored} (floor {G12_MIN_FRAMES})  \
         refusals {refusals}  seeds {}  sub-support pairs {}",
        per_seed.len(),
        per_seed.iter().map(|r| r.5).sum::<usize>()
    );
    println!("# GATE G12: {}", pf(frames_scored >= G12_MIN_FRAMES));
    println!(
        "# NEGLIGIBLE_FRACTION inherited unchanged from B1: {NEGLIGIBLE_FRACTION}. Not \
         re-chosen, not widened, not narrowed."
    );
}

fn record_meta(
    meta: &mut BTreeMap<usize, (&'static str, u64, f64)>,
    sim: &Sim,
    a: Species,
    b: Species,
    pt: &holon_chem::pair::PairTable,
) {
    let slot = sim.bank.slot_of_z(a.z, b.z).expect("registered");
    let exit: &'static str = match format!("{:?}", pt.meta.exit).as_str() {
        "Converged" => "Converged",
        "IterationCap" => "IterationCap",
        "Stagnated" => "Stagnated",
        _ => "Other",
    };
    meta.insert(slot, (exit, pt.meta.solver_budget as u64, pt.meta.worst_residual));
    println!(
        "# W1 {}-{}: slot {slot} route {:?} exit {exit} n_det {} n_basis {} \
         solver_budget {} worst_residual {:.3e}",
        a.symbol, b.symbol, pt.meta.route, pt.meta.n_det, pt.meta.n_basis,
        pt.meta.solver_budget, pt.meta.worst_residual
    );
}

// ====================================================================== main

fn arg<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.iter().find_map(|a| a.strip_prefix(key))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arm = arg(&args, "--arm=").unwrap_or("engine").to_string();
    let curveset = arg(&args, "--curves=").unwrap_or("hh").to_string();
    let steps: u64 = arg(&args, "--steps=")
        .and_then(|v| v.parse().ok())
        .unwrap_or(G12_MIN_STEPS);
    let budget: f64 = arg(&args, "--budget=")
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_BUDGET);
    let root = PathBuf::from(
        arg(&args, "--root=").unwrap_or("/home/emoore/holon-artifacts/census-traj"),
    );
    let manifest = PathBuf::from(arg(&args, "--manifest=").unwrap_or(
        "conformance/water_observatory/census_traj_manifest.sha256",
    ));
    let stride: usize = arg(&args, "--stride=")
        .and_then(|v| v.parse().ok())
        .unwrap_or(400);

    let cpu = cpu_now();
    println!("# B2 LONG-RANGE GATE BATTERY — freeze conformance/water_observatory/B2_PREREG.md");
    println!("# instrument = engine/crates/holon-render/examples/b2_longrange.rs");
    println!("# arm = {arm}  curves = {curveset}  steps = {steps}  budget = {budget:.3e} Ha");
    println!(
        "# launch: loadavg {:.2}  cpu {cpu}  core class {}  clock fraction {:.3}",
        loadavg(),
        core_class(cpu),
        clock_fraction(cpu)
    );
    let t_wall = Instant::now();
    let t_cpu = cpu_seconds();

    let curves: Vec<(Species, Species)> = match curveset.as_str() {
        "full" => vec![
            (HYDROGEN, HYDROGEN),
            (OXYGEN, HYDROGEN),
            (OXYGEN, OXYGEN),
        ],
        _ => vec![(HYDROGEN, HYDROGEN)],
    };

    match arm.as_str() {
        "engine" => arm_engine(&curves, steps, budget),
        "frames" => arm_frames(&root, &manifest, stride),
        "refusals" => arm_refusals(),
        other => println!("unknown arm {other}; expected engine|frames|refusals"),
    }

    println!(
        "# done: wall {:.1} s  cpu {:.1} s  loadavg at exit {:.2}  clock fraction at exit {:.3}",
        t_wall.elapsed().as_secs_f64(),
        cpu_seconds() - t_cpu,
        loadavg(),
        clock_fraction(cpu_now())
    );
}
