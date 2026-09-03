//! THE 3D HIGH-N CARRIER: the N-ladder that prices it, and the generator that produces it.
//!
//! `CARRIER_V2_PREREG.md` §4 and §5. Two modes, one instrument, because the ladder and the
//! production run must place the SAME scene — a price measured on a different placement
//! than the one that runs is not a price for the run.
//!
//! ## What this pays
//!
//! `CENSUS_RESULTS.md` §14.4: seventeen of eighteen banked trajectories declare `dims = 2`
//! and hold `z` at placement BIT-EXACTLY for 20,000 frames, so the one arm that left the
//! plane was exploring a configuration space the others could not reach and the
//! one-variable design was defeated. A planar configuration under in-plane forces stays
//! planar by symmetry; the fix is not a flag, it is a PLACEMENT that breaks the symmetry
//! at frame zero. `place3d` jitters z exactly as it jitters x and y, and G9 measures that
//! at frame 0 rather than taking it on faith.
//!
//! `RUNG2_RESULTS.md`: twelve atoms in 1.831 x 1.101 nm cannot hold a hundred atoms per
//! cell and still move atoms across cell faces. That needs N in the hundreds at least, and
//! N in the hundreds needs the `O(N)` neighbour route, which `cells.rs` only engages when
//! every axis admits three cells at the declared cutoff. In two dimensions the box was
//! never deep enough and the route never engaged; in three it can. Whether it DOES at the
//! sizes this campaign can afford is the ladder's question, and F1/F2 are staked on it.
//!
//! ## Work units, not wall clock
//!
//! The host is shared and loaded and this node cannot pin cores (`M-PLACEMENT-LOTTERY`,
//! `M-DEVICE-CLASS`, neither discharged). Every price is in work units, counted EXACTLY by
//! an executor that wraps the real one and tallies the terms handed to it — not sampled
//! from the neighbour list at frame boundaries, which would miss every rebuild in between.
//! Wall clock is printed where it is unavoidable and labelled contended.
//!
//! ```text
//! carrier3d --mode=ladder  --out=DIR [--dry-run] [--workers=N]
//! carrier3d --mode=produce --out=DIR --n=402 --frames=20000 [--seeds=0x..,..]
//! ```
//!
//! Exit codes: 0 fine, 2 bad arguments, 3 a path did not resolve, 4 a format refusal,
//! 6 a worker-lease refusal, 7 an envelope refusal.

use holon_chem::elements::{Species, HYDROGEN, OXYGEN};
use holon_chem::pair::generate_pair_table;
use holon_lens::traj::pair_index;
use holon_lens::traj2::{Header2, Ledger, TrajWriter2, CONTENT_FORCES, CONTENT_LEDGER};
use holon_md::WorkerPool;
use holon_render::sim::{
    Boundary, Dims, ForceExecutor, PairTerm, Sim, TripleTerm, K_B,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

// ================================================================= THE FROZEN PROTOCOL

/// ARM A's density, atoms per cubic bohr — `CARRIER_V2_PREREG.md` §4.2's stake, WHICH
/// FIRED.
///
/// Liquid water's atom number density: 0.0334 molecules/A^3 x 3 atoms/molecule at
/// 1 A = 1.8897261 bohr. It is kept, marked, and still reachable through `--rho=`, because
/// `CARRIER_V2_AMENDMENT_1.md`'s A1 needs the placement that failed as its must-fire
/// control. It is NOT the default and must never become one again: see [`LATTICE_SPACING`]
/// for what it did and why.
const RHO_ATOMS_ARM_A_FIRED: f64 = 0.014_860;

/// The ladder's rungs, in ATOMS, each an exact 2:1 stoichiometry.
///
/// `CARRIER_V2_PREREG.md` §4.2's ladder is the brief's 24/48/96/200/400 rounded to whole
/// molecules — 8, 16, 32, 67 and 134 waters. A scene with a spare hydrogen is a different
/// chemistry, and the ladder's whole point is that only N changes across it.
const LADDER: [usize; 5] = [24, 48, 96, 201, 402];

/// Initial kinetic temperature, kelvin. The census protocol's, unchanged: hot enough that
/// the opening configuration is a gas, cold enough that no pair starts up the wall.
const T_INIT: f64 = 3000.0;
/// Thermostat target, kelvin — the quench's floor. The census protocol's.
const T_TARGET: f64 = 300.0;
/// Berendsen coupling time, atomic time units. The census protocol's.
const TAU: f64 = 2000.0;
/// Substeps per grain boundary. The census protocol's.
const SUBSTEPS: u32 = 64;
/// Jitter on the opening lattice, bohr — applied to ALL THREE axes. See `place3d`.
const JITTER: f64 = 0.8;
/// Knots per pair curve. The census protocol's.
const CURVE_KNOTS: usize = 96;
/// The declared pair-truncation budget, hartree per pair. The workspace's floor: every
/// `set_pair_cutoff` call in this tree uses 1e-6 and this campaign does not invent a new
/// one to move a route boundary in its favour.
const PAIR_FLOOR: f64 = 1e-6;

/// The census campaign's eight staked seeds, kept so the arms remain comparable.
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

/// THE CENSUS PROTOCOL'S OWN NEAREST-NEIGHBOUR SPACING, bohr.
///
/// Written as the quotient rather than as 6.9333 so its provenance is visible: the census
/// places 12 atoms on a 4x3 lattice in a 34.6 x 20.8 bohr box, and 20.8/3 is the smaller of
/// its two spacings — the one that decides whether a pair opens inside a well.
///
/// `CARRIER_V2_AMENDMENT_1.md` is why this constant exists at all. The freeze fixed the
/// DENSITY, and the certified protocol's controlled quantity is the SPACING. The two
/// coincide in a fixed dimension and part company across a 2D-to-3D change: at N = 402 the
/// frozen density gave 3.75 bohr between neighbours against the census's 6.93, every atom
/// opened inside its neighbours' wells, and the scene reached 241,001 K by frame 500
/// against a 300 K target. A uniform ATOM lattice at liquid ATOM density is a
/// supersaturated covalent solid, not a liquid.
const LATTICE_SPACING: f64 = 20.8 / 3.0;

/// The cubic lattice side that holds `n` atoms — sites past `n` stay empty.
fn lattice_side(n: usize) -> usize {
    (n as f64).cbrt().ceil() as usize
}

/// The cube edge for `n` atoms.
///
/// `Some(rho)` reproduces ARM A, the fired stake, kept runnable because
/// `CARRIER_V2_AMENDMENT_1.md`'s A1 needs it as a must-fire control: a 5000 K bar that has
/// never been seen to fail is a fence, and the placement that failed it is the only
/// carrier that can show it discriminates.
///
/// `None` is ARM B: the edge is `side x LATTICE_SPACING`, and the density is whatever that
/// implies and is REPORTED rather than declared.
fn box_edge(n: usize, rho: Option<f64>) -> f64 {
    match rho {
        Some(r) => (n as f64 / r).cbrt(),
        None => lattice_side(n) as f64 * LATTICE_SPACING,
    }
}

/// The number density a scene actually has, atoms per bohr^3 — reported, never declared.
fn density_of(n: usize, rho: Option<f64>) -> f64 {
    let e = box_edge(n, rho);
    n as f64 / (e * e * e)
}

/// Oxygens FIRST, then hydrogens — the census protocol's convention, so the lattice site a
/// nucleus occupies is a function of the protocol and not of the seed.
fn species_at(i: usize, n: usize) -> Species {
    if i < n / 3 {
        OXYGEN
    } else {
        HYDROGEN
    }
}

/// THE OPENING SCENE, and the one thing about it that is not the census protocol.
///
/// A cubic lattice with a seeded jitter on ALL THREE AXES, and velocities from a
/// Maxwellian in ALL THREE COMPONENTS, per species because oxygen is sixteen times
/// hydrogen's mass.
///
/// The z jitter is the whole point. `CENSUS_RESULTS.md` §14.4 measured seventeen
/// trajectories holding z bit-exactly for 20,000 frames — not because anything forbade
/// them from leaving, but because they were placed exactly in a plane and the exact
/// gradient of a z-reflection-invariant energy is z-free at a symmetric point. The
/// symmetry has to be broken at placement or it is never broken at all, and no flag
/// anywhere in the engine does that.
fn place3d(s: &mut Sim, n: usize, seed: u64, rho: Option<f64>) {
    let mut st = seed;
    s.reset(n);
    for i in 0..n {
        assert!(
            s.set_species(i, species_at(i, n)),
            "species {i} did not register"
        );
    }
    // DERIVE THE TIMESTEP FROM THE PLACED SCENE, never from the empty box — the defect
    // `waterquench_traj.rs` documents: `load_pair_table` calls this while the box is empty,
    // its loop body never runs, and it falls through to a hydrogen default.
    s.adopt_table_timescale();

    let edge = box_edge(n, rho);
    // Sites beyond `n` stay empty. In arm B the edge is DERIVED from this side and the
    // census's spacing, so the placed nearest-neighbour distance is that spacing exactly
    // (A2's one-variable claim) and the density is a consequence.
    let side = lattice_side(n);
    let mut vel = vec![(0.0f64, 0.0, 0.0); n];
    let (mut px, mut py, mut pz) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let (ix, iy, iz) = (i % side, (i / side) % side, i / (side * side));
        let at = |k: usize, st: &mut u64| {
            edge * (k as f64 + 0.5) / side as f64 + JITTER * (2.0 * lcg(st) - 1.0)
        };
        let x = at(ix, &mut st);
        let y = at(iy, &mut st);
        let z = at(iz, &mut st);
        s.set_position_3d(i, x, y, z);
        let sigma = (K_B * T_INIT / s.atoms[i].mass()).sqrt();
        let v = (
            sigma * gauss(&mut st),
            sigma * gauss(&mut st),
            sigma * gauss(&mut st),
        );
        vel[i] = v;
        // MOMENTUM, not velocity: with two masses in the box those are different sums.
        let m = s.atoms[i].mass();
        px += m * v.0;
        py += m * v.1;
        pz += m * v.2;
    }
    let m_tot: f64 = (0..n).map(|i| s.atoms[i].mass()).sum();
    for i in 0..n {
        s.set_velocity_3d(
            i,
            vel[i].0 - px / m_tot,
            vel[i].1 - py / m_tot,
            vel[i].2 - pz / m_tot,
        );
    }
    s.rebase();
    s.thermostat_on = true;
    s.target_temperature = T_TARGET;
    s.thermostat_tau = TAU;
}

// ==================================================================== the work counter

/// An executor that counts the terms it is handed and then hands them on.
///
/// EXACT, and that is the reason it exists rather than sampling `neighbours().pairs.len()`
/// at grain boundaries: the neighbour list is rebuilt every step and a boundary sample
/// misses the 63 rebuilds in between. It also cannot change an answer — it adds no
/// arithmetic, only a tally, and the inner executor does the same work in the same order.
struct Counting<E> {
    inner: E,
    pair_terms: AtomicU64,
    triple_terms: AtomicU64,
    pair_calls: AtomicU64,
}

impl<E: ForceExecutor> Counting<E> {
    fn new(inner: E) -> Self {
        Self {
            inner,
            pair_terms: AtomicU64::new(0),
            triple_terms: AtomicU64::new(0),
            pair_calls: AtomicU64::new(0),
        }
    }
}

impl<E: ForceExecutor> ForceExecutor for Counting<E> {
    fn eval_pairs(&self, sim: &Sim, terms: &mut [PairTerm], chunk: usize) {
        self.pair_terms.fetch_add(terms.len() as u64, Ordering::Relaxed);
        self.pair_calls.fetch_add(1, Ordering::Relaxed);
        self.inner.eval_pairs(sim, terms, chunk)
    }
    fn eval_triples(&self, sim: &Sim, terms: &mut [TripleTerm], chunk: usize) {
        self.triple_terms
            .fetch_add(terms.len() as u64, Ordering::Relaxed);
        self.inner.eval_triples(sim, terms, chunk)
    }
    fn workers(&self) -> usize {
        self.inner.workers()
    }
}

/// A counter the caller keeps a handle on while the engine owns the executor.
///
/// Same discipline as `holon_md::PoolHandle`, and the same reason: the engine's seam takes
/// an owned box that must survive across `&mut Sim` calls, and the tally is the caller's.
#[derive(Clone, Copy)]
struct CountHandle(*const Counting<holon_render::sim::SerialExecutor>);
// SAFETY: the handle is installed and removed inside one function, the target outlives it,
// and every method it calls takes `&self` and touches only atomics.
unsafe impl Send for CountHandle {}
unsafe impl Sync for CountHandle {}
impl ForceExecutor for CountHandle {
    fn eval_pairs(&self, sim: &Sim, terms: &mut [PairTerm], chunk: usize) {
        unsafe { &*self.0 }.eval_pairs(sim, terms, chunk)
    }
    fn eval_triples(&self, sim: &Sim, terms: &mut [TripleTerm], chunk: usize) {
        unsafe { &*self.0 }.eval_triples(sim, terms, chunk)
    }
    fn workers(&self) -> usize {
        unsafe { &*self.0 }.workers()
    }
}

// ========================================================================== the tables

/// The three curves, generated ONCE.
///
/// Every knot is a full CI solve and O2 is 2025 determinants a point, so regenerating them
/// per rung would put the ladder's whole cost in the tables and price nothing. Generated in
/// `main`, loaded into each rung's fresh `Sim`.
fn make_tables(knots: usize) -> Vec<holon_chem::pair::PairTable> {
    [(HYDROGEN, HYDROGEN), (OXYGEN, HYDROGEN), (OXYGEN, OXYGEN)]
        .into_iter()
        .map(|(a, b)| {
            let t0 = Instant::now();
            let pt = generate_pair_table(a, b, knots);
            // THE DISCLOSURE FIELDS on a solver-derived number, printed whether or not
            // they are comfortable. The census's own log carries
            // "WARNING O-O: worst residual 2.68e-6 exceeds CONVERGED_RESIDUAL 1e-9" and
            // that curve sits under every banked trajectory; a runner that printed only
            // the timing would have produced the same physics with the warning invisible.
            // `converged` is the VERDICT, `worst_residual` the number, and the solver
            // budget and route are part of the artifact's IDENTITY rather than
            // diagnostics -- two curves under different budgets are two artifacts.
            let m = &pt.meta;
            println!(
                "  curve {}-{}: {knots} knots, {}, worst residual {:.2e}, route {:?}, \
budget {}, exit {:?}, {}{:.1} s (contended)",
                m.symbol_a,
                m.symbol_b,
                match m.well {
                    Some(w) => format!("R_e {:.4} bohr D_e {:.6} Ha", w.r_e, w.d_e),
                    None => "no well (repulsive only)".to_string(),
                },
                m.worst_residual,
                m.route,
                m.solver_budget,
                m.exit,
                if m.converged() {
                    ""
                } else {
                    "NOT CONVERGED (worst residual above CONVERGED_RESIDUAL), "
                },
                t0.elapsed().as_secs_f64()
            );
            pt
        })
        .collect()
}

fn load_tables(s: &mut Sim, tables: &[holon_chem::pair::PairTable]) {
    for pt in tables {
        let ok = holon_render::load_pair_table(s, pt, holon_render::bank::Host::Native);
        assert_eq!(
            ok,
            holon_render::TABLE_OK,
            "the (Z{}, Z{}) curve did not load",
            pt.meta.z_a,
            pt.meta.z_b
        );
    }
}

// ============================================================================ the rungs

struct Rung {
    n: usize,
    edge: f64,
    route: &'static str,
    cells: [usize; 3],
    r_cut: f64,
    w_pair: u64,
    w_triple: u64,
    w_de4: u64,
    steps: u64,
    workers: usize,
    seconds: f64,
    dims_at_zero: bool,
}

/// Frames the ladder runs at each rung. Small on purpose: the ladder measures a RATE, and
/// a rate needs enough steps to be stable and not one step more. Reported so the per-step
/// division is auditable.
const LADDER_FRAMES: usize = 4;

fn run_rung(
    n: usize,
    workers: usize,
    tables: &[holon_chem::pair::PairTable],
    rho: Option<f64>,
) -> Result<Rung, String> {
    let edge = box_edge(n, rho);
    let mut s = Box::new(Sim::empty());
    s.boundary = Boundary::Walls;
    s.dims = Dims::Three;
    s.width = edge;
    s.height = edge;
    s.depth = edge;
    load_tables(&mut s, tables);
    place3d(&mut s, n, SEEDS[0], rho);
    if !s.set_pair_cutoff(PAIR_FLOOR) {
        return Err(format!(
            "no pair cutoff could be derived at floor {PAIR_FLOOR:e} for N = {n}"
        ));
    }
    // G9's placement half, measured at frame ZERO before anything integrates. A scene that
    // opens planar is refused here; catching it at the end is catching it too late.
    let z0 = s.atoms[0].z;
    let dims_at_zero = (0..n).any(|i| (s.atoms[i].z - z0).abs() > 1e-9);

    let counter = Counting::new(holon_render::sim::SerialExecutor);
    let handle = CountHandle(&counter);
    let pool = if workers > 1 {
        match WorkerPool::new(workers) {
            Ok(p) => Some(p),
            Err(e) => return Err(format!("worker lease refused: {e:?}")),
        }
    } else {
        None
    };
    let leased = pool.as_ref().map(|p| p.workers()).unwrap_or(1);
    // The counter wraps the SERIAL executor here rather than the pool: the ladder is
    // pricing WORK, and the work is the same term count under any worker count — that is
    // `tests/bit_identity.rs`'s whole guarantee. The pool's own scaling is `examples/
    // scaling.rs`'s question and is not re-answered here under a different name.
    //
    // The pool is RETIRED rather than dropped. Nothing observes the difference — the arena
    // is owned by the pool and dies with it — but the lease discipline is that leases are
    // released, and a lane that drops them because nobody is looking is a lane that has
    // stopped obeying the rule rather than one the rule does not apply to.
    if let Some(p) = pool {
        let _ = p.retire();
    }
    s.set_executor(Some(Box::new(handle)));
    let t0 = Instant::now();
    for _ in 0..LADDER_FRAMES {
        s.step_frame(SUBSTEPS);
    }
    let seconds = t0.elapsed().as_secs_f64();
    s.set_executor(None);

    Ok(Rung {
        n,
        edge,
        route: match s.route() {
            holon_render::cells::Route::Cells => "Cells",
            holon_render::cells::Route::Complete => "Complete",
        },
        cells: s.cells_per_axis(),
        r_cut: s.list_cutoff(),
        w_pair: counter.pair_terms.load(Ordering::Relaxed),
        w_triple: counter.triple_terms.load(Ordering::Relaxed),
        // `de4_eval_count` became `many_body_evals` at 02bc47f, when the sector went
        // any-order: the counter is the same column, counting every many-body term
        // rather than four-body ones alone.
        w_de4: s.many_body_evals,
        steps: LADDER_FRAMES as u64 * SUBSTEPS as u64,
        workers: leased,
        seconds,
        dims_at_zero,
    })
}

/// The log-log slope of `y` against `x` by least squares. Reported with the points it was
/// fitted to, never alone.
/// A1's PAIRED TEMPERATURE PROBE — `CARRIER_V2_AMENDMENT_1.md` §A1.2.2.
///
/// Runs the SAME instrument on the SAME N and seed, differing in one argument: `rho`. Arm A
/// (the fired placement) must EXCEED the bar and arm B must fall below it. Both numbers are
/// printed whatever they say, because a 5000 K bar that has never been seen to fail is a
/// fence, and if both land on the same side the amendment's own rule is that A1 is VOID
/// rather than passed.
fn a1_probe(
    n: usize,
    frames: usize,
    tables: &[holon_chem::pair::PairTable],
    rho: Option<f64>,
) -> Result<(f64, f64, usize), String> {
    let edge = box_edge(n, rho);
    let mut s = Box::new(Sim::empty());
    s.boundary = Boundary::Walls;
    s.dims = Dims::Three;
    s.width = edge;
    s.height = edge;
    s.depth = edge;
    load_tables(&mut s, tables);
    place3d(&mut s, n, SEEDS[0], rho);
    if !s.set_pair_cutoff(PAIR_FLOOR) {
        return Err(format!("no pair cutoff at floor {PAIR_FLOOR:e} for N = {n}"));
    }
    let t_open = s.temperature();
    for _ in 0..frames {
        s.step_frame(SUBSTEPS);
    }
    let bonds = s.pairs[..s.pair_count].iter().filter(|p| p.bonded).count();
    Ok((t_open, s.temperature(), bonds))
}

fn loglog_slope(pts: &[(f64, f64)]) -> f64 {
    let n = pts.len() as f64;
    let (mut sx, mut sy, mut sxx, mut sxy) = (0.0, 0.0, 0.0, 0.0);
    for (x, y) in pts {
        let (lx, ly) = (x.ln(), y.ln());
        sx += lx;
        sy += ly;
        sxx += lx * lx;
        sxy += lx * ly;
    }
    (n * sxy - sx * sy) / (n * sxx - sx * sx)
}

fn ladder(workers: usize, rungs: &[usize], knots: usize, rho: Option<f64>) -> i32 {
    println!("THE N-LADDER — CARRIER_V2_PREREG.md §4");
    println!(
        "{}, pair floor {PAIR_FLOOR:e} Ha, dE4 ON, {LADDER_FRAMES} frames x {SUBSTEPS} \
substeps per rung",
        match rho {
            Some(r) => format!("ARM A (FIRED, kept as A1's control): density {r} atoms/bohr^3 declared"),
            None => format!(
                "ARM B: lattice spacing {LATTICE_SPACING:.4} bohr (= 20.8/3, the census's own)"
            ),
        }
    );
    println!(
        "\n{:>5} {:>8} {:>9} {:>9} {:>8} {:>14} {:>12} {:>8} {:>7} {:>6} {:>9}",
        "N", "edge", "route", "cells", "r_cut", "W_pair/step", "W_trip/step", "W_dE4", "3D@0",
        "lease", "s (cont.)"
    );
    println!("\ngenerating the three pair curves once:");
    let tables = make_tables(knots);
    let mut rows = Vec::new();
    for &n in rungs {
        match run_rung(n, workers, &tables, rho) {
            Ok(r) => {
                println!(
                    "{:>5} {:>8.2} {:>9} {:>3}x{}x{} {:>8.3} {:>14.1} {:>12.1} {:>8} {:>7} {:>6} {:>9.1}",
                    r.n,
                    r.edge,
                    r.route,
                    r.cells[0],
                    r.cells[1],
                    r.cells[2],
                    r.r_cut,
                    r.w_pair as f64 / r.steps as f64,
                    r.w_triple as f64 / r.steps as f64,
                    r.w_de4,
                    if r.dims_at_zero { "yes" } else { "NO" },
                    // THE LEASED count, never the requested one (M-PROBE-THE-RESOURCE).
                    // It enters no number here -- the counter wraps the serial executor and
                    // term counts are worker-invariant -- but the freeze requires it
                    // reported, and the first ladder run collected it and printed nothing.
                    r.workers,
                    r.seconds,
                );
                rows.push(r);
            }
            Err(e) => {
                eprintln!("{n:>5}  REFUSED  {e}");
            }
        }
    }
    println!("\n(the seconds column is CONTENDED — shared, loaded host, no core pinning)");

    if rows.len() < 3 {
        eprintln!("\nVOID — fewer than 3 rungs completed; no slope is quoted (§4.4)");
        return 0;
    }
    let top: Vec<(f64, f64)> = rows
        .iter()
        .filter(|r| r.n >= 96)
        .map(|r| (r.n as f64, r.w_pair as f64 / r.steps as f64))
        .collect();
    let all: Vec<(f64, f64)> = rows
        .iter()
        .map(|r| (r.n as f64, r.w_pair as f64 / r.steps as f64))
        .collect();
    let slope_top = loglog_slope(&top);
    let slope_all = loglog_slope(&all);
    println!("\nF1 — d ln W_pair / d ln N");
    println!("  over the top three rungs (96, 201, 402): {slope_top:.4}");
    println!("  over all {} rungs:                       {slope_all:.4}", all.len());
    println!("  staked: PASS at <= 1.35, and >= 1.80 means the cell route never engaged");
    let f1 = if slope_top <= 1.35 {
        "PASS"
    } else if slope_top >= 1.80 {
        "FAIL — the O(N^2) path under a 3D label"
    } else {
        "NEITHER — between the staked bands, reported as such"
    };
    println!("  F1: {f1}");

    let top_rung = rows.last().unwrap();
    let f2 = top_rung.route == "Cells" && top_rung.cells.iter().all(|c| *c >= 3);
    println!(
        "\nF2 — Route::Cells at N = {} with >= 3 cells per axis: {} ({} {}x{}x{})",
        top_rung.n,
        if f2 { "PASS" } else { "FAIL" },
        top_rung.route,
        top_rung.cells[0],
        top_rung.cells[1],
        top_rung.cells[2]
    );

    // ------------------------------------------------------------------- G8, the scissor
    //
    // Extrapolated from the fit, reported as an extrapolation. RUNG2's successor bar is
    // >= 100 atoms/cell WITH inter-cell transport, and a chart needs at least 2 cells per
    // axis to have faces at all, so the smallest grid that can be asked is 2x2x2 and the
    // smallest N that can answer is 800.
    let (ref_n, ref_w) = *all.last().unwrap();
    println!("\nG8 — THE SCISSOR'S PRICE, extrapolated from the fit above");
    for target in [800.0f64, 6400.0] {
        let w = ref_w * (target / ref_n).powf(slope_top);
        println!(
            "  N = {:>5.0}  ({:>2}x{:>2}x{:>2} cells at 100 atoms/cell)  W_pair/step ~ {:>12.0}  \
             = {:>5.1}x the N = {ref_n:.0} rung",
            target,
            (target / 100.0).cbrt().round() as i64,
            (target / 100.0).cbrt().round() as i64,
            (target / 100.0).cbrt().round() as i64,
            w,
            w / ref_w
        );
    }
    println!(
        "  the extrapolation uses the TOP-RUNG slope {slope_top:.4}; if F1 read NEITHER or\n  \
         FAIL above, this number is an upper bound on the cost and not a forecast"
    );
    0
}

// ======================================================================== the production

fn produce(
    out: &PathBuf,
    n: usize,
    frames: usize,
    seeds: &[u64],
    workers: usize,
    knots: usize,
    rho: Option<f64>,
) -> i32 {
    println!("generating the three pair curves once:");
    let tables = make_tables(knots);
    produce_with(out, n, frames, seeds, workers, knots, rho, &tables)
}

/// The production loop with the curves already in hand, so `--mode=both` pays for them once.
#[allow(clippy::too_many_arguments)]
fn produce_with(
    out: &PathBuf,
    n: usize,
    frames: usize,
    seeds: &[u64],
    workers: usize,
    knots: usize,
    rho: Option<f64>,
    tables: &[holon_chem::pair::PairTable],
) -> i32 {
    if let Err(e) = std::fs::create_dir_all(out) {
        eprintln!("REFUSED  output directory {}: {e}", out.display());
        return 3;
    }
    let edge = box_edge(n, rho);
    for &seed in seeds {
        let mut s = Box::new(Sim::empty());
        s.boundary = Boundary::Walls;
        s.dims = Dims::Three;
        s.width = edge;
        s.height = edge;
        s.depth = edge;
        load_tables(&mut s, &tables);
        place3d(&mut s, n, seed, rho);
        if !s.set_pair_cutoff(PAIR_FLOOR) {
            eprintln!("REFUSED  no pair cutoff at floor {PAIR_FLOOR:e} for N = {n}");
            return 4;
        }
        let z0 = s.atoms[0].z;
        if !(0..n).any(|i| (s.atoms[i].z - z0).abs() > 1e-9) {
            eprintln!("REFUSED  G9: the scene opens PLANAR and would stay planar by symmetry");
            return 4;
        }
        // A NON-PROTOCOL CURVE CANNOT PRODUCE A FILE THAT LOOKS LIKE A PRODUCTION ONE.
        //
        // `--knots` exists so the write path can be exercised without paying the O-O
        // curve, and a smoke artifact that shared a production artifact's name would be
        // one `mv` away from being banked as one. The knot count is part of the curve's
        // identity exactly as the solver budget is, so it goes in the NAME when it is not
        // the protocol's.
        let mut stem = format!("n{n}_seed_{seed:#018x}");
        if knots != CURVE_KNOTS {
            stem = format!("SMOKE-{knots}knots_{stem}");
        }
        if rho.is_some() {
            // ARM A IS A FIRED STAKE. Its trajectories are controls, not carriers, and a
            // control that shares a carrier's filename is one `mv` from being banked as
            // one.
            stem = format!("ARMA-FIRED_{stem}");
        }
        let path = out.join(format!("{stem}.traj"));
        let header = Header2 {
            seed,
            n_atoms: n,
            // Recorded as what this runner ASSERTS. The reader measures it and the manifest
            // carries the measurement; this field is never the answer.
            dims_declared: 3,
            substeps: SUBSTEPS,
            n_frames: frames,
            dt: s.dt(),
            box_w: edge,
            box_h: edge,
            box_d: edge,
            z: (0..n).map(|i| s.atoms[i].species.z).collect(),
            content: CONTENT_FORCES | CONTENT_LEDGER,
        };
        let mut w = match TrajWriter2::create(&path, &header) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("REFUSED  {}: {e}", path.display());
                return e.exit_code();
            }
        };
        let pool = match WorkerPool::new(workers) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("REFUSED  worker lease: {e:?}");
                return 6;
            }
        };
        let leased = pool.workers();
        println!("[{seed:#018x}] N={n} edge={edge:.2} bohr workers={leased} -> {}", path.display());

        let mut pos = vec![[0.0f64; 3]; n];
        let mut vel = vec![[0.0f64; 3]; n];
        let mut frc = vec![[0.0f64; 3]; n];
        let mut bonds: Vec<u32> = Vec::new();
        let t0 = Instant::now();
        let boxed: Box<dyn ForceExecutor + Send + Sync> = Box::new(PoolRef(&pool));
        s.set_executor(Some(boxed));
        for frame in 0..frames {
            s.step_frame(SUBSTEPS);
            for i in 0..n {
                let a = s.atoms[i];
                pos[i] = [a.x, a.y, a.z];
                vel[i] = [a.vx, a.vy, a.vz];
                // THE TOTAL FORCE, hartree/bohr: internal (pair + triple + four-body,
                // which cancels from the momentum sum) plus external (walls and the
                // field, which does not). Recording only the internal half would put a
                // wall-driven atom in the artifact with no force explaining its turn.
                //
                // The per-atom SPLIT is deliberately not carried. It is recoverable in
                // aggregate from the ledger's `j_ext`, and a second per-atom array would
                // double the file for a distinction nothing downstream has asked for.
                let (ix, iy, iz) = s.internal_force(i);
                let (ex, ey, ez) = s.external_force(i);
                frc[i] = [ix + ex, iy + ey, iz + ez];
            }
            bonds.clear();
            for p in s.pairs[..s.pair_count].iter().filter(|p| p.bonded) {
                let (a, b) = if p.i < p.j { (p.i, p.j) } else { (p.j, p.i) };
                bonds.push(pair_index(n, a, b) as u32);
            }
            bonds.sort_unstable();
            let led = Ledger {
                j_ext: [s.j_ext.0, s.j_ext.1, s.j_ext.2],
                w_hand: s.work.hand,
                w_thermostat: s.work.thermostat,
                w_barostat: s.work.barostat,
                total: s.ledger(),
                l0: s.l0,
            };
            if let Err(e) = w.push(
                frame as u64,
                s.time,
                s.temperature(),
                &bonds,
                &pos,
                &vel,
                Some(&frc),
                Some(&led),
            ) {
                eprintln!("REFUSED  frame {frame}: {e}");
                return e.exit_code();
            }
            if (frame + 1) % 500 == 0 || frame + 1 == frames {
                println!(
                    "  frame {:>6}/{} | T {:>5.0} K | bonds {:>5} | drift {:.2e}/{:.2e} | \
                     |p| {:.2e}/{:.2e} | dE4 {:>6} | {:>7.1} s (cont.)",
                    frame + 1,
                    frames,
                    s.temperature(),
                    bonds.len(),
                    s.drift(),
                    s.drift_bound(),
                    s.momentum_residual(),
                    s.momentum_bound(),
                    s.many_body_evals,
                    t0.elapsed().as_secs_f64()
                );
            }
        }
        s.set_executor(None);
        match w.finish() {
            Ok(k) => println!("  {k} frames written"),
            Err(e) => {
                eprintln!("REFUSED  closing {}: {e}", path.display());
                return e.exit_code();
            }
        }
        pool.retire();
    }
    0
}

/// A borrow of a pool for the engine's executor slot — `holon_md::run_frames`'s
/// `PoolHandle` in miniature, needed here because this loop writes a frame between steps
/// and cannot hand the whole run to `run_frames`.
struct PoolRef(*const WorkerPool);
// SAFETY: installed and removed inside `produce`, which owns the pool for the whole time.
unsafe impl Send for PoolRef {}
unsafe impl Sync for PoolRef {}
impl ForceExecutor for PoolRef {
    fn eval_pairs(&self, sim: &Sim, terms: &mut [PairTerm], chunk: usize) {
        unsafe { &*self.0 }.eval_pairs(sim, terms, chunk)
    }
    fn eval_triples(&self, sim: &Sim, terms: &mut [TripleTerm], chunk: usize) {
        unsafe { &*self.0 }.eval_triples(sim, terms, chunk)
    }
    fn workers(&self) -> usize {
        unsafe { &*self.0 }.workers()
    }
}

// =============================================================================== main

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let arg = |k: &str| args.iter().find_map(|a| a.strip_prefix(k));
    let dry = args.iter().any(|a| a == "--dry-run");
    let mode = match arg("--mode=") {
        Some(m) => m.to_string(),
        None => {
            eprintln!("REFUSED  --mode=ladder|produce is required and has no default");
            std::process::exit(2);
        }
    };
    let knots: usize = match arg("--knots=") {
        Some(v) => match v.parse::<usize>() {
            // A curve needs enough knots to have a shape; below this the interpolant is
            // not a potential, and a smoke test on one would exercise a write path under
            // physics nobody would recognise.
            Ok(k) if k >= 8 => k,
            _ => {
                eprintln!("REFUSED  --knots={v} is not an integer >= 8");
                std::process::exit(2);
            }
        },
        None => CURVE_KNOTS,
    };
    // ARM SELECTION, and it has no silent default. `--rho=` reproduces arm A, the FIRED
    // stake, kept runnable because A1's bar needs it as a must-fire control. Absent, the
    // scene is arm B and its density is a reported consequence of the census's spacing.
    let rho: Option<f64> = match arg("--rho=") {
        Some(v) => match v.parse::<f64>() {
            Ok(r) if r > 0.0 && r.is_finite() => Some(r),
            _ => {
                eprintln!("REFUSED  --rho={v} is not a positive finite density");
                std::process::exit(2);
            }
        },
        None => None,
    };
    let workers: usize = match arg("--workers=") {
        Some(v) => match v.parse() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("REFUSED  --workers={v} is not a number");
                std::process::exit(2);
            }
        },
        None => 8,
    };

    if mode == "ladder" {
        // THE RUNG LIST IS PARSED BEFORE THE DRY-RUN BRANCH, NOT AFTER IT.
        //
        // It was after, so `--dry-run --rungs=402,804` printed the DEFAULT ladder and
        // exited 0 — a dry run that describes a different job from the one the same
        // arguments would launch, which is worse than no dry run at all because it is
        // believed. A diagnostic must echo the parameters it was given.
        let rungs: Vec<usize> = match arg("--rungs=") {
            Some(list) => list.split(',').filter_map(|v| v.trim().parse().ok()).collect(),
            None => LADDER.to_vec(),
        };
        if rungs.is_empty() {
            eprintln!("REFUSED  --rungs= parsed to nothing");
            std::process::exit(2);
        }
        if dry {
            println!("DRY RUN — nothing was computed and nothing was written.");
            println!(
                "would run {} rungs, {LADDER_FRAMES} frames x {SUBSTEPS} substeps each:",
                rungs.len()
            );
            for &n in &rungs {
                println!(
                    "  N = {n:>4}  ({:>3} waters)  side {:>2}  edge {:>6.2} bohr  \
spacing {:>5.3}  rho {:.5}  3 cells/axis needs r_cut <= {:.3}",
                    n / 3,
                    lattice_side(n),
                    box_edge(n, rho),
                    box_edge(n, rho) / lattice_side(n) as f64,
                    density_of(n, rho),
                    box_edge(n, rho) / 3.0
                );
            }
            println!("workers requested: {workers} (the LEASED count is what gets reported)");
            std::process::exit(0);
        }
        std::process::exit(ladder(workers, &rungs, knots, rho));
    }

    if mode == "both" || mode == "probe" {
        // ONE PROCESS, ONE SET OF CURVES. The O-O curve costs ~1000 s and A1's whole design
        // is that its two arms differ in ONE argument on ONE instrument; paying the curve
        // twice would also mean two sets of curves, which is a second difference.
        let rungs: Vec<usize> = match arg("--rungs=") {
            Some(list) => list.split(',').filter_map(|v| v.trim().parse().ok()).collect(),
            None => LADDER.to_vec(),
        };
        let probe_n: usize = arg("--probe-n=").and_then(|v| v.parse().ok()).unwrap_or(804);
        let probe_frames: usize =
            arg("--probe-frames=").and_then(|v| v.parse().ok()).unwrap_or(500);
        if dry {
            println!("DRY RUN — nothing was computed and nothing was written.");
            println!("would price rungs {rungs:?} on arm B, then run A1's PAIR at N = {probe_n}");
            println!("  arm B  spacing {:.4} bohr, edge {:.2}", LATTICE_SPACING, box_edge(probe_n, None));
            println!(
                "  arm A  rho {RHO_ATOMS_ARM_A_FIRED}, edge {:.2}, spacing {:.4} (THE FIRED STAKE)",
                box_edge(probe_n, Some(RHO_ATOMS_ARM_A_FIRED)),
                box_edge(probe_n, Some(RHO_ATOMS_ARM_A_FIRED)) / lattice_side(probe_n) as f64
            );
            println!("  {probe_frames} frames x {SUBSTEPS} substeps each, bar 5000 K");
            if mode == "both" {
                let out = arg("--out=").unwrap_or("(REQUIRED)");
                let frames: usize = arg("--frames=").and_then(|v| v.parse().ok()).unwrap_or(20000);
                let nn: usize = arg("--n=").and_then(|v| v.parse().ok()).unwrap_or(804);
                println!("then produce N = {nn}, {frames} frames, into {out}");
            }
            std::process::exit(0);
        }
        println!("generating the three pair curves once:");
        let tables = make_tables(knots);

        if !rungs.is_empty() {
            println!("\n--- ARM B LADDER ---");
            for &n in &rungs {
                match run_rung(n, workers, &tables, None) {
                    Ok(r) => println!(
                        "N {:>5}  edge {:>7.2}  {:>8} {}x{}x{}  r_cut {:>7.3}  \
W_pair/step {:>12.1}  W_dE4 {:>6}  3D@0 {}  lease {}",
                        r.n, r.edge, r.route, r.cells[0], r.cells[1], r.cells[2], r.r_cut,
                        r.w_pair as f64 / r.steps as f64, r.w_de4,
                        if r.dims_at_zero { "yes" } else { "NO" }, r.workers
                    ),
                    Err(e) => eprintln!("N {n:>5}  REFUSED  {e}"),
                }
            }
        }

        println!("\n--- A1: THE PAIRED TEMPERATURE PROBE, bar 5000 K ---");
        println!("both numbers are printed whatever they say; if both land on the SAME side");
        println!("of the bar, A1 is VOID rather than passed (AMENDMENT_1 §A1.2.2)");
        let mut verdicts = Vec::new();
        for (label, r) in [
            ("P-A arm A (FIRED placement, must EXCEED)", Some(RHO_ATOMS_ARM_A_FIRED)),
            ("P-B arm B (census spacing, must fall BELOW)", None),
        ] {
            match a1_probe(probe_n, probe_frames, &tables, r) {
                Ok((t0, t1, bonds)) => {
                    println!(
                        "  {label:<44} N={probe_n} spacing {:>6.4}  T(0) {:>9.0} K  \
T({probe_frames}) {:>10.0} K  bonds {bonds}",
                        box_edge(probe_n, r) / lattice_side(probe_n) as f64,
                        t0,
                        t1
                    );
                    verdicts.push((label, t1));
                }
                Err(e) => eprintln!("  {label}: REFUSED {e}"),
            }
        }
        if verdicts.len() == 2 {
            let (a, b) = (verdicts[0].1, verdicts[1].1);
            let fired = a > 5000.0;
            let quenching = b < 5000.0;
            println!(
                "\n  A1: {}",
                if fired && quenching {
                    "PASS — the control fired and the measurement did not"
                } else if !fired {
                    "VOID — the control did NOT fire, so the bar does not discriminate \
and the A1.1 diagnosis is not supported"
                } else {
                    "FAIL — arm B is also above the bar; the spacing was not the cause"
                }
            );
            println!("  arm A {a:.0} K, arm B {b:.0} K, ratio {:.1}x", a / b.max(1e-9));
        }

        if mode == "probe" {
            std::process::exit(0);
        }
        let Some(out) = arg("--out=") else {
            eprintln!("REFUSED  --out=DIR is required for --mode=both");
            std::process::exit(2);
        };
        let out = PathBuf::from(out);
        let nn: usize = arg("--n=").and_then(|v| v.parse().ok()).unwrap_or(804);
        let frames: usize = arg("--frames=").and_then(|v| v.parse().ok()).unwrap_or(20000);
        let seeds: Vec<u64> = match arg("--seeds=") {
            Some(list) => list
                .split(',')
                .filter_map(|v| u64::from_str_radix(v.trim().trim_start_matches("0x"), 16).ok())
                .collect(),
            None => SEEDS[..1].to_vec(),
        };
        if nn % 3 != 0 {
            eprintln!("REFUSED  N = {nn} is not 3 x whole waters");
            std::process::exit(2);
        }
        println!("\n--- PRODUCTION (arm B) ---");
        std::process::exit(produce_with(&out, nn, frames, &seeds, workers, knots, rho, &tables));
    }

    if mode == "produce" {
        let Some(out) = arg("--out=") else {
            eprintln!("REFUSED  --out=DIR is required for --mode=produce");
            std::process::exit(2);
        };
        let out = PathBuf::from(out);
        let n: usize = arg("--n=").and_then(|v| v.parse().ok()).unwrap_or(402);
        let frames: usize = arg("--frames=").and_then(|v| v.parse().ok()).unwrap_or(20000);
        let seeds: Vec<u64> = match arg("--seeds=") {
            Some(list) => list
                .split(',')
                .filter_map(|s| u64::from_str_radix(s.trim().trim_start_matches("0x"), 16).ok())
                .collect(),
            None => SEEDS[..1].to_vec(),
        };
        if n % 3 != 0 {
            eprintln!("REFUSED  N = {n} is not 3 x whole waters; the ladder's stoichiometry is 2:1");
            std::process::exit(2);
        }
        if dry {
            println!("DRY RUN — nothing was computed and nothing was written.");
            println!("out    {}", out.display());
            println!(
                "N      {n} ({} waters), edge {:.2} bohr, spacing {:.4}, rho {:.5}",
                n / 3,
                box_edge(n, rho),
                box_edge(n, rho) / lattice_side(n) as f64,
                density_of(n, rho)
            );
            println!("frames {frames} x {SUBSTEPS} substeps, dE4 ON, content = forces|ledger");
            println!("seeds  {}", seeds.iter().map(|s| format!("{s:#018x}")).collect::<Vec<_>>().join(" "));
            let bytes = frames as u64
                * (8 + 8 + 8 + 4 + 4 * 2 * n as u64 + 8 * 9 * n as u64);
            println!("size   ~{:.1} MB per trajectory (bonds estimated at 2N)", bytes as f64 / 1e6);
            std::process::exit(0);
        }
        std::process::exit(produce(&out, n, frames, &seeds, workers, knots, rho));
    }

    eprintln!("REFUSED  unknown --mode={mode}; expected ladder, produce, probe or both");
    std::process::exit(2);
}
