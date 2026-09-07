//! EDGE-0's runner: the edge exists, or it does not — screened, gated, run and read.
//!
//! `conformance/mesh/EDGE0_PREREG.md` is the freeze. The carrier is `holon_lattice::edge`
//! (the rest particle, the two arms, the wait, gravity as a posted injection) and the
//! instruments are `holon_lattice::surface` (the density histogram, the pressure tensor and
//! Laplace's law, the capillary spectrum, the interface width, the block chart's edge). The
//! harness is `holon-campaign`: this runner is its first adopter from this crate, so the
//! gates name every failing leg, the plants' carriers and analytic reaches are checked with
//! no measurement at all, the price is a token that only writing makes, and the record
//! writer refuses malformed JSON and unlabelled typed stakes.
//!
//! ```text
//! edge0_lattice screen [dir]            (also spelled `--screen`)
//!                                       the density x rent chart, swept coarsely.
//!                                       EVERY FILE IS `dry: true` AND NOTHING IT WRITES
//!                                       ENTERS A GATE OR A CLAIM (GANTT2's screen law).
//! edge0_lattice gate   [dir]            G0 the ledger, G1 the FLUID-1 identity, G2 the five
//!                                       instruments on their known scenes, the two plants
//!                                       pre-checked, price.json -> gate.json
//! edge0_lattice run    [dir] [threads]  the counted runs -> run.json, run.done
//! edge0_lattice read   [dir]            the branches -> read.json
//! ```
//!
//! `out_dir` defaults to `../conformance/mesh/edge0`, i.e. run from `engine/`.
//!
//! # No number from another tier is typed here
//!
//! The bond amplitude table and CT-2's measured retention are READ at run time from the
//! molecular tier's own records with the path and the field printed beside every value, as
//! FLUID-1's runner reads them. FLUID-1's measured cost per step is READ from
//! `mesh/fluid1/price.json`. The only numbers typed from outside this engine are the
//! experimental kills, and each says so where it appears.

// Waived for the reasons FLUID-0's and FLUID-1's runners waive them: the JSON is assembled
// line by line, and the readouts take their parameters positionally by design.
// `!(x > bar)` is deliberate where `x` may be `NaN`: a refused fit falls on the failing side.
#![allow(
    clippy::write_with_newline,
    clippy::needless_range_loop,
    clippy::too_many_arguments,
    clippy::neg_cmp_op_on_partial_ord
)]

use holon_campaign::{
    is_done, read_input, read_input_after, read_record, Gate, Plant, Price, Priced, Record,
    RecordWriter, Report, Stake,
};
use holon_lattice::edge::{
    hex_embed, EdgeCounts, EdgeLattice, EdgeRules, Gravity, MoverRule, N_ORIENT,
};
use holon_lattice::lattice::Lattice;
use holon_lattice::orientation::{OrientationLattice, OrientationRules};
use holon_lattice::state::Model;
use holon_lattice::surface::{
    beyond, bimodality, bimodality_floor, block_density, capillary, cell_position,
    centred_height, column_profiles, density_variance, droplet_lifetime, droplet_scene,
    edge_at, height_field, ideal_gas_kt, interface_width, laplace_sigma, line_fit, pressure,
    pressure_over, profile_j, quantile, slab_scene, two_phase_densities, within,
    DropletLife,
};
use std::fmt::Write as _;
use std::time::Instant;

// ═══════════ THE DECLARED BLOCK — every number the instrument uses, and nowhere else ═══════

// ---- the carrier ----
/// The box. One size for every scene, so the two σ readings and the edge are read on the
/// same carrier at the same resolution and no comparison crosses a size.
const L: usize = 128;
/// The seed FLUID-1 and FLUID-0 declared, kept so the identity control is an IDENTITY.
const SEED: u64 = 0x464c_5549_4430;
/// Members averaged in every reading that has a noise: the two σ readings' own spread over
/// seeds is what the freeze stakes their agreement band from, so it is never one seed.
const SEEDS: usize = 4;

// ---- the chart, PRE-COMMITTED BY THE FREEZE FROM THE SCREEN, BY THE SCREEN'S OWN RULE ----
/// **The rule, stated before the numbers**: the chart is the CLUSTER arm's swept point of
/// largest block-density variance against the one-phase control, among the points whose
/// largest bonded component is under half the particles. The second clause is not decoration:
/// a point where the whole box is one component is a GEL, and a gel has no droplet, so a
/// chart there would be a chart for a different campaign. The dip cannot choose the chart —
/// it never cleared its floor anywhere on the pair arm's sweep and its largest excesses sat
/// where a longer warm-up read inside the floor's spread.
///
/// Applied to `conformance/mesh/edge0/screen.json` the rule selects `d = 0.02`, `f = 0.90`:
/// variance ratio `13.378` against the one-phase control, largest component `0.157`,
/// `1.168` bonds per particle. The PAIR arm at the same chart reads `1.919`, one mode, and
/// `0.506` bonds per particle — it is the freeze's pre-committed control.
const CHART_DENSITY: f64 = 0.02;
/// The bond's depth as the linear-bond RETENTION it is equivalent to, so CT-2's measured
/// `0.5526` is a point ON the sweep. `E₀/kT_rent = ln(f/(1−f)) = 2.1972`.
const CHART_RETENTION: f64 = 0.90;
/// The two phases' own densities, MEASURED at the chart on the counted runs' own box, window
/// and seeds (`conformance/mesh/edge0/screen_chart.json`, the cluster arm's
/// `rho_lo` / `rho_hi`): the mean density of the blocks below and above the midpoint of the
/// sample's `q05` and `q95`. The dense phase holds `0.198` of the box.
///
/// **The droplet and the slab are built at these**, so the interface instruments are read on
/// the carrier's OWN two densities and not on a construction. The same decomposition on the
/// PAIR arm reads `0.01392 / 0.02865` with a dense share of `0.414`, and on the one-phase
/// control `0.01493 / 0.02566` with `0.477` — one phase in both, which is what the
/// instrument's own unit test says a single phase looks like.
const CHART_D_LIQUID: f64 = 0.05905;
const CHART_D_VAPOUR: f64 = 0.01038;

// ---- the density histogram (instrument a) ----
const B_HIST: usize = 8;
const HIST_BINS: usize = 100;
/// Two modes must be this many bins apart to count as two. Eight bins is `0.08` in density.
const HIST_MIN_SEP: usize = 8;

// ---- the windows ----
/// Steps discarded before any reading. The rent clause relaxes in one step and the bond
/// graph in tens; this is two orders clear of both and is measured for stationarity by the
/// interface-width readout, which reports its whole time series.
const WARMUP: usize = 3_000;
/// Frames sampled after the warm-up, and the steps between them.
const FRAMES: usize = 200;
const FRAME_STRIDE: usize = 5;

// ---- Laplace's law (instrument b) ----
/// The droplet radii, in links. The largest is under a quarter of the box so a droplet never
/// sees its own image across the torus.
const DROP_RADII: [f64; 6] = [10.0, 14.0, 18.0, 22.0, 26.0, 30.0];
/// The pressure inside is read within this fraction of the radius, the pressure outside
/// beyond this multiple of it: a shell of the interface itself is in NEITHER reading.
const P_IN_FRACTION: f64 = 0.5;
const P_OUT_FACTOR: f64 = 1.6;
/// Laplace's law is a LINE or it is not: the fit's own `R²` decides, and this is the bar.
const LAPLACE_MIN_R2: f64 = 0.90;

// ---- the capillary spectrum (instrument c) ----
/// The slab occupies `j ∈ [L/4, 3L/4)`, so both interfaces are half a box from each other.
const SLAB_J0: usize = L / 4;
const SLAB_J1: usize = 3 * L / 4;
/// The mode band: the smallest is two wavelengths across the box, the largest is a
/// wavelength of eight cells. A mode of a few cells is not a capillary wave.
const CAP_M_MIN: usize = 2;
const CAP_M_MAX: usize = 16;
/// The height field is searched for its crossing in this window about the slab's upper face.
const HEIGHT_HALF_WINDOW: usize = 16;

// ---- the interface width (instrument d) ----
/// The width is fitted on this window about the upper face and tracked over the whole run.
const WIDTH_HALF_WINDOW: usize = 20;
/// The width is STATIONARY when the least-squares slope of `w(t)` over the sampled frames is
/// under this many cells per thousand steps in absolute value. Growing without bound is the
/// freeze's kill and this is the line it is read against.
const WIDTH_SLOPE_BAR: f64 = 0.05;

// ---- gravity (deliverable 2) ----
const GRAV_DIR: usize = 4;
const GRAV_RATE: f64 = 0.02;
/// The settling readout: the dense phase's centre of mass along gravity, tracked over the
/// run and fitted for a drift.
const GRAV_FRAMES: usize = 200;
const GRAV_STRIDE: usize = 10;

// ---- the droplet's survival (instrument f) ----
/// One bonded droplet of this radius in a background at the sparse phase's own density, per
/// arm, per seed; settled this long so the bond rule can bind it, sampled at this stride, and
/// capped here. A CAP IS A REFUSAL and is reported as one, never as a lifetime.
const DROP_LIFE_R: f64 = 12.0;
const DROP_LIFE_SETTLE: usize = 50;
const DROP_LIFE_STRIDE: usize = 25;
const DROP_LIFE_CAP: usize = 6_000;

// ---- the block chart's edge (instrument e) ----
const W_BLOCKS: [usize; 3] = [4, 8, 16];
/// One step, because the edge is the ONE-step split set — node LG's own reading.
const W_STEPS: usize = 1;
/// The edge is read on a smaller box: `chart_view` runs one exhaustive probe per movable
/// cell and its cost is quadratic in the box.
const L_EDGE: usize = 48;

// ---- branch C's two bars, DERIVED (see EDGE0_PREREG §3) ----
/// The cluster arm's block-density variance ratio must exceed the PAIR arm's at the same
/// chart by this factor. Derived, not typed: over the whole screen the pair rule's ratio never
/// exceeded `2.489` at any density or rent, and a one-phase lattice's is `1` by construction,
/// so a factor of three over the pair arm at the same chart is a move the pair rule was never
/// measured to make anywhere on the chart.
const C_VARIANCE_FACTOR: f64 = 3.0;
/// The cluster arm's block-density SPREAD `q95 − q05` must clear the ONE-PHASE control's by
/// this many standard deviations of the two arms' COMBINED measured spread over seeds. The
/// bar is a measured one and not a typed factor: what a single correlated phase's
/// finite-sample spread does at the same density, seeds, box and window is exactly what the
/// control measures. The band is COMBINED and not the control's alone because the control's
/// own seed spread came back exactly zero on the confirmation pass, and a band of zero voids
/// the branch instead of passing it.
///
/// A SPREAD and not a ratio, because the sparse phase's `q05` was measured EXACTLY ZERO at
/// the chart (its blocks are empty), and a ratio with a zero denominator is not a statistic.
/// Quantiles and not the histogram's modes, because the mode finder's `min_separation` knob
/// can manufacture a second mode at exactly that separation.
const C_SPREAD_SIGMA: f64 = 3.0;

// ---- the gates' own numbers ----
/// G0's ledger sweep: configurations, and steps per configuration, every integer checked at
/// every step.
const G0_STEPS: usize = 500;
/// G1's identity: steps of a BONDED run compared bit for bit against FLUID-1's carrier.
const G1_STEPS: usize = 300;
const G1_L: usize = 32;
/// G2's known-scene tolerances. The first three are EXACT identities on planted data and the
/// tolerance is a floating-point one, not a physical one.
const G2_EXACT_TOL: f64 = 1e-9;
/// The free gas's `c_s²`, which the hexagon forces and the gate MEASURES. A gate on a
/// derived constant, not on a fitted one.
const G2_CS2: f64 = 0.5;
const G2_CS2_TOL: f64 = 0.02;
const G2_KT_STEPS: usize = 300;

// ---- the plants ----
/// Plant (i) doubles the rent; plant (ii) doubles gravity's rate. Both stakes are DERIVED
/// from the freeze's own arithmetic below and neither is typed (FLUID-1's correction: a
/// plant's stake must be derived from the reach of the observable it acts on).
const PLANT_STAKE_FRACTION: f64 = 0.5;
const PLANT_I_CARRIER_FLOOR: f64 = 0.01;
const PLANT_II_CARRIER_FLOOR: f64 = 0.001;
/// Plant (i)'s held-geometry scene.
const PLANT_I_STEPS: usize = 200_000;
/// Plant (ii)'s first-step injection, averaged over this many seeds.
const PLANT_II_SEEDS: usize = 8;

// ---- the price ----
const PRICE_STEPS: u64 = 200;
const PRICE_LOW: f64 = 0.1;
const PRICE_HIGH: f64 = 10.0;

// ---- the screen (a screen is not a reading; every file it writes is `dry`) ----
const SCREEN_L: usize = 64;
/// Long enough for a domain to cross the box several times over: the tracer's diffusion on
/// this family is of order a few links squared per step, so `sqrt(D·t)` is past `L` within a
/// few hundred steps and this is an order clear of that. A separate long-warm-up spot check
/// at the sweep's own best point guards the "it had not coarsened yet" reading.
const SCREEN_WARMUP: usize = 3_000;
const SCREEN_FRAMES: usize = 40;
const SCREEN_STRIDE: usize = 10;
const SCREEN_B: usize = 8;
/// Widened DOWNWARD for the second pass: the cluster rule (2b) is all-or-nothing and its
/// behaviour is set by cluster size against free space, so the dilute end where a component
/// can still move has to be on the grid. The first pass's grid (`0.10 … 0.60`) is a subset of
/// this one at three of its points, so the pair rule's readings there are comparable.
const SCREEN_DENSITIES: [f64; 6] = [0.02, 0.05, 0.10, 0.20, 0.35, 0.50];
/// The rent, written as the linear-bond retention it is equivalent to. The sweep runs from
/// CT-2's own measured `0.5526` out to a bond that essentially never pays rent
/// (`0.999` is `p_break = 1.0e-3`), because the question the screen asks is whether ANY depth
/// of bond makes two phases on this carrier.
const SCREEN_RETENTIONS: [f64; 5] = [0.5526, 0.90, 0.99, 0.999, 0.99999];
/// The long-warm-up spot check at the sweep's best point: the same reading with the warm-up
/// this many times longer, so "no bimodality" cannot be "not yet coarsened".
const SCREEN_LONG_FACTOR: usize = 10;
/// The droplet-survival reading on the screen: one bonded droplet of this radius in a sparse
/// background, settled this long, sampled at this stride, capped here. A cap is a refusal.
const SCREEN_DROP_R: f64 = 6.0;
const SCREEN_DROP_SETTLE: usize = 20;
const SCREEN_DROP_STRIDE: usize = 25;
const SCREEN_DROP_CAP: usize = 3_000;
const SCREEN_DROP_SEEDS: usize = 2;
/// The two mover rules the screen sweeps side by side.
const SCREEN_RULES: [MoverRule; 2] = [MoverRule::Pair, MoverRule::Cluster];
/// A chart whose largest bonded component holds this fraction of the particles or more is a
/// GEL, not a droplet in a vapour, and the screen's selection rule refuses it: EDGE-0 measures
/// an edge, and a system-spanning network has no edge to measure.
const SCREEN_PERCOLATION_BAR: f64 = 0.5;
const SCREEN_SEEDS: usize = 3;

// ---- the records READ at run time; the paths are data, the numbers are never typed ----
const R_CT1_LINEAR: &str = "water_observatory/ct1/sector_linear_R2.9.json";
const R_CT2_LINEAR_EXACT: &str = "water_observatory/ct2/gd0_linear_R2.9.json";
const R_CT2_T60: &str = "water_observatory/ct2/node_tilt_R2.9_t60.json";
const R_CT2_T120: &str = "water_observatory/ct2/node_tilt_R2.9_t120.json";
const R_CT2_T180: &str = "water_observatory/ct2/node_tilt_R2.9_t180.json";
const R_CT2_ARMS: &str = "water_observatory/ct2/arms.json";
const R_FLUID1_PRICE: &str = "mesh/fluid1/price.json";
// ═════════════════════════ END OF THE DECLARED BLOCK ══════════════════════════════════════

// ---------------------------------------------------------------- small helpers

fn jf(x: f64) -> String {
    if x.is_finite() {
        format!("{x:.10}")
    } else {
        "null".to_string()
    }
}

fn jarr(v: &[f64]) -> String {
    format!("[{}]", v.iter().map(|x| jf(*x)).collect::<Vec<_>>().join(", "))
}

fn jarr_u(v: &[u64]) -> String {
    format!("[{}]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

fn sd(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return f64::NAN;
    }
    let m = mean(v);
    (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
}

fn rel(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        f64::NAN
    } else {
        (a - b).abs() / b.abs()
    }
}

fn records_root() -> String {
    for cand in ["../conformance", "conformance", "../../conformance"] {
        if std::path::Path::new(cand).is_dir() {
            return cand.to_string();
        }
    }
    panic!("no conformance/ directory found from the working directory");
}

/// A field read out of a record, with its path and its field printed beside its value.
fn read_num(root: &str, rel_path: &str, keys: &[&str], field: &str) -> (f64, String) {
    let path = format!("{root}/{rel_path}");
    let inp = if keys.is_empty() {
        read_input(&path, field)
    } else {
        read_input_after(&path, keys, field)
    };
    let inp = inp.unwrap_or_else(|e| panic!("reading {path}:{field} — {e}"));
    (inp.value, inp.cite())
}

fn seed_of(member: usize) -> u64 {
    SEED ^ (member as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

// ---------------------------------------------------------------- the bond rule, READ

struct Amp {
    phi_degrees: usize,
    amplitude: f64,
    source: String,
}

/// The amplitude table `A(φ)`, READ from CT-1's and CT-2's records exactly as FLUID-1 reads
/// it: the value at each lattice angle is the measured `E_CT` at the nearest measured tilt,
/// normalised to one at 0°. The 2.9 Å tilt family starts at 60°, so the linear node's `E_CT`
/// is built from CT-2's total minus CT-1's no-transfer total, by the tilt records' own rule.
fn amplitude_table(root: &str) -> [Amp; N_ORIENT] {
    let (full, c_full) = read_num(root, R_CT2_LINEAR_EXACT, &[], "e_full");
    let (noct, c_noct) = read_num(root, R_CT1_LINEAR, &[], "e_noct");
    let e0 = full - noct;
    let (e60, c60) = read_num(root, R_CT2_T60, &[], "e_ct");
    let (e120, c120) = read_num(root, R_CT2_T120, &[], "e_ct");
    let (e180, c180) = read_num(root, R_CT2_T180, &[], "e_ct");
    let a = |e: f64| e / e0;
    [
        Amp { phi_degrees: 0, amplitude: 1.0, source: format!("{c_full} minus {c_noct}") },
        Amp { phi_degrees: 60, amplitude: a(e60), source: c60.clone() },
        Amp { phi_degrees: 120, amplitude: a(e120), source: c120.clone() },
        Amp { phi_degrees: 180, amplitude: a(e180), source: c180 },
        Amp { phi_degrees: 240, amplitude: a(e120), source: format!("{c120} by tilt symmetry") },
        Amp { phi_degrees: 300, amplitude: a(e60), source: format!("{c60} by tilt symmetry") },
    ]
}

fn amplitudes(t: &[Amp; N_ORIENT]) -> [f64; N_ORIENT] {
    let mut out = [0.0; N_ORIENT];
    for k in 0..N_ORIENT {
        out[k] = t[k].amplitude;
    }
    out
}

fn amplitude_table_json(t: &[Amp; N_ORIENT]) -> String {
    let mut s = String::from("[");
    for (k, a) in t.iter().enumerate() {
        if k > 0 {
            s.push_str(", ");
        }
        let _ = write!(
            s,
            "{{\"phi_degrees\": {}, \"phi_index\": {}, \"amplitude\": {}, \"source\": {:?}}}",
            a.phi_degrees,
            k,
            jf(a.amplitude),
            a.source
        );
    }
    s.push(']');
    s
}

/// The chart's carrier: EDGE-0's cluster arm (rule 2b). The pair arm is the same rules with
/// `mover` set back to [`MoverRule::Pair`], which is how the control is a CONFIGURATION and
/// not a second carrier.
fn chart_rules(amp: [f64; N_ORIENT], retention: f64) -> EdgeRules {
    EdgeRules::edge0_cluster(amp, EdgeRules::rent_from_retention(retention))
}

// ---------------------------------------------------------------- scene builders

fn uniform(l: usize, seed: u64, d: f64, rules: EdgeRules) -> EdgeLattice {
    EdgeLattice::seeded(l, seed, d, rules)
}

fn no_bond(mut rules: EdgeRules) -> EdgeRules {
    rules.bonds_enabled = false;
    rules
}

/// Advance a lattice, discarding the counts, and return the wall seconds it took.
fn advance(g: &mut EdgeLattice, steps: usize) -> f64 {
    let t = Instant::now();
    for _ in 0..steps {
        g.step();
    }
    t.elapsed().as_secs_f64()
}

/// Advance a WARM-UP with the per-step ledger audit off, and switch it back on with a fresh
/// baseline afterwards. The conserved integers are conserved from whatever instant the
/// baseline is taken at, so this moves the start of the audited window and weakens nothing;
/// what it buys is the whole census pass per step, which is most of a warm-up's cost. G0
/// audits every step of every configuration in its own right.
fn warm(g: &mut EdgeLattice, steps: usize) -> f64 {
    g.audit_every_step = false;
    let s = advance(g, steps);
    g.audit_every_step = true;
    g.reset_initial();
    s
}

/// A bounded worker pool over an index set, one whole item per worker. Cores 24–31 only:
/// `taskset` is the binding constraint and the count here is never a device claim.
fn parallel_map<T: Send, F>(n: usize, threads: usize, f: F) -> Vec<T>
where
    F: Fn(usize) -> T + Sync,
{
    if threads <= 1 || n <= 1 {
        return (0..n).map(&f).collect();
    }
    let next = std::sync::atomic::AtomicUsize::new(0);
    let slots: Vec<std::sync::Mutex<Option<T>>> =
        (0..n).map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for _ in 0..threads.min(n) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if i >= n {
                    break;
                }
                *slots[i].lock().unwrap() = Some(f(i));
            });
        }
    });
    slots.into_iter().map(|m| m.into_inner().unwrap().unwrap()).collect()
}

// ---------------------------------------------------------------- readouts

/// The coexistence readout on one member: the block-density histogram over frames.
struct Coexistence {
    dip: f64,
    /// Bonds released per bond formed over the sampled window — the churn, which is what
    /// FLUID-1's and rule 2's ceiling is made of.
    release_over_formed: f64,
    /// The largest component the mover pass saw (rule 2b only; zero under rule 2, which has
    /// no component pass at all).
    largest_component: u64,
    /// The block-density variance — the precursor, reported beside the dip and never as a
    /// verdict: a single large domain and two coexisting phases raise it alike.
    variance: f64,
    mode_lo: f64,
    mode_hi: f64,
    /// The sparse and dense phases' own densities, read as quantiles of the block-density
    /// sample. No `min_separation` knob touches these, which is why the freeze's C branch
    /// reads them and not the modes.
    q05: f64,
    q50: f64,
    q95: f64,
    /// The two phases' own densities and the dense phase's share of the box.
    rho_lo: f64,
    rho_hi: f64,
    dense_fraction: f64,
    separated: bool,
    mean_density: f64,
    rest_fraction: f64,
    bonds_per_particle: f64,
    largest_fraction: f64,
    counts: Vec<u64>,
    all_exact: bool,
    failing: Vec<&'static str>,
}

fn coexistence(l: usize, seed: u64, d: f64, rules: EdgeRules, warmup: usize, frames: usize, stride: usize, b: usize) -> Coexistence {
    let mut g = uniform(l, seed, d, rules);
    warm(&mut g, warmup);
    let mut all: Vec<f64> = Vec::with_capacity(frames * (l / b) * (l / b));
    let mut rest = Vec::with_capacity(frames);
    let mut bpp = Vec::with_capacity(frames);
    let mut largest = Vec::with_capacity(frames);
    for _ in 0..frames {
        advance(&mut g, stride);
        all.extend(block_density(&g, b));
        rest.push(g.rest_count() as f64 / g.particles() as f64);
        bpp.push(g.live_bonds() as f64 / g.particles() as f64);
        largest.push(g.phase().largest_fraction);
    }
    let bi = bimodality(&all, HIST_BINS, HIST_MIN_SEP);
    let tot = g.audit.totals;
    Coexistence {
        dip: bi.dip,
        release_over_formed: tot.blocked as f64 / tot.formed.max(1) as f64,
        largest_component: tot.largest_component,
        variance: density_variance(&all),
        q05: quantile(&all, 0.05),
        q50: quantile(&all, 0.50),
        q95: quantile(&all, 0.95),
        rho_lo: two_phase_densities(&all).0,
        rho_hi: two_phase_densities(&all).1,
        dense_fraction: two_phase_densities(&all).2,
        mode_lo: bi.mode_lo,
        mode_hi: bi.mode_hi,
        separated: bi.separated,
        mean_density: bi.mean,
        rest_fraction: mean(&rest),
        bonds_per_particle: mean(&bpp),
        largest_fraction: mean(&largest),
        counts: bi.counts,
        all_exact: g.audit.all_exact(),
        failing: g.audit.failing_legs(),
    }
}

/// One droplet's pressure jump.
struct Drop {
    r: f64,
    p_in: f64,
    p_out: f64,
    dp: f64,
    cells_in: usize,
    cells_out: usize,
    anisotropy_in: f64,
    all_exact: bool,
}

fn droplet(l: usize, seed: u64, r: f64, rules: EdgeRules, warmup: usize, window: usize) -> Drop {
    let embed = hex_embed(&Model::fhp7());
    let centre = (l / 2) * l + l / 2;
    let mut g = droplet_scene(l, seed, r, CHART_D_LIQUID, CHART_D_VAPOUR, rules).with_flux();
    warm(&mut g, warmup);
    g.clear_flux();
    advance(&mut g, window);
    let p_in = pressure_over(&g, within(l, centre, P_IN_FRACTION * r, &embed));
    let p_out = pressure_over(&g, beyond(l, centre, P_OUT_FACTOR * r, &embed));
    Drop {
        r,
        p_in: p_in.scalar(),
        p_out: p_out.scalar(),
        dp: p_in.scalar() - p_out.scalar(),
        cells_in: p_in.cells,
        cells_out: p_out.cells,
        anisotropy_in: p_in.anisotropy(),
        all_exact: g.audit.all_exact(),
    }
}

/// The slab's readouts: the height field per frame, the width per frame, the profile.
struct Slab {
    heights: Vec<Vec<f64>>,
    widths: Vec<f64>,
    centres: Vec<f64>,
    rms: Vec<f64>,
    frames_with_holes: usize,
    profile: Vec<f64>,
    rho_hi: f64,
    rho_lo: f64,
    all_exact: bool,
}

fn slab(l: usize, seed: u64, rules: EdgeRules, warmup: usize, frames: usize, stride: usize) -> Slab {
    let mut g = slab_scene(l, seed, SLAB_J0, SLAB_J1, CHART_D_LIQUID, CHART_D_VAPOUR, rules);
    warm(&mut g, warmup);
    let mut heights = Vec::new();
    let mut widths = Vec::new();
    let mut centres = Vec::new();
    let mut rms = Vec::new();
    let mut holes = 0usize;
    let mut profile_acc = vec![0.0f64; l];
    // The two plateau densities are READ from the profile's own interiors, never typed: the
    // middle of the slab and the middle of the vapour, a quarter of a box from either face.
    let first = profile_j(&g);
    let rho_hi = mean(&first[SLAB_J0 + l / 16..SLAB_J1 - l / 16]);
    let rho_lo = {
        let mut v: Vec<f64> = first[..SLAB_J0.saturating_sub(l / 16)].to_vec();
        v.extend_from_slice(&first[(SLAB_J1 + l / 16).min(l)..]);
        mean(&v)
    };
    let mid = 0.5 * (rho_hi + rho_lo);
    for _ in 0..frames {
        advance(&mut g, stride);
        let p = profile_j(&g);
        for (a, b) in profile_acc.iter_mut().zip(&p) {
            *a += b;
        }
        let f = interface_width(
            &p,
            SLAB_J1 - WIDTH_HALF_WINDOW,
            (SLAB_J1 + WIDTH_HALF_WINDOW).min(l),
            rho_hi,
            rho_lo,
        );
        widths.push(f.width);
        centres.push(f.centre);
        rms.push(f.rms);
        let cols = column_profiles(&g);
        let h = height_field(
            &cols,
            SLAB_J1 - HEIGHT_HALF_WINDOW,
            (SLAB_J1 + HEIGHT_HALF_WINDOW).min(l),
            mid,
            true,
        );
        match centred_height(&h) {
            Some(v) => heights.push(v),
            None => holes += 1,
        }
    }
    profile_acc.iter_mut().for_each(|v| *v /= frames as f64);
    Slab {
        heights,
        widths,
        centres,
        rms,
        frames_with_holes: holes,
        profile: profile_acc,
        rho_hi,
        rho_lo,
        all_exact: g.audit.all_exact(),
    }
}

/// The dense phase's centre of mass along gravity, over time — the settling readout.
struct Settling {
    com: Vec<f64>,
    slope: f64,
    r2: f64,
    injected: [i64; 2],
    counts: EdgeCounts,
    all_exact: bool,
}

fn settling(l: usize, seed: u64, mut rules: EdgeRules, on: bool) -> Settling {
    rules.gravity = if on { Some(Gravity { dir: GRAV_DIR, rate: GRAV_RATE }) } else { None };
    let embed = hex_embed(&Model::fhp7());
    let gdir = embed[GRAV_DIR];
    let norm = (gdir[0] * gdir[0] + gdir[1] * gdir[1]).sqrt();
    let mut g = slab_scene(l, seed, SLAB_J0, SLAB_J1, CHART_D_LIQUID, CHART_D_VAPOUR, rules);
    let mut com = Vec::with_capacity(GRAV_FRAMES);
    for _ in 0..GRAV_FRAMES {
        advance(&mut g, GRAV_STRIDE);
        // The dense phase's centre of mass PROJECTED on gravity, weighted by occupancy. The
        // box wraps, so the projection is taken as a circular mean of the phase along the
        // gravity axis and reported as a displacement from the first frame.
        let (mut sx, mut sy, mut m) = (0.0f64, 0.0f64, 0.0f64);
        for c in 0..g.cells.len() {
            let w = g.cells[c].count_ones() as f64;
            if w == 0.0 {
                continue;
            }
            let p = cell_position(l, c, &embed);
            sx += w * p[0];
            sy += w * p[1];
            m += w;
        }
        com.push(((sx / m) * gdir[0] + (sy / m) * gdir[1]) / norm);
    }
    let t: Vec<f64> = (0..com.len()).map(|k| (k * GRAV_STRIDE) as f64).collect();
    let fit = line_fit(&t, &com);
    Settling {
        com,
        slope: fit.slope,
        r2: fit.r2,
        injected: g.injected,
        counts: g.audit.totals,
        all_exact: g.audit.all_exact(),
    }
}

// ---------------------------------------------------------------- the price

/// The price, from FLUID-1's OWN measured cost per step scaled by this campaign's box and
/// this carrier's measured overhead. Nothing here is typed.
struct PriceInputs {
    fluid1_seconds_per_step: f64,
    fluid1_l: f64,
    per_cell_per_step: f64,
    cite: String,
}

fn price_inputs(root: &str) -> PriceInputs {
    let (s, c1) = read_num(root, R_FLUID1_PRICE, &[], "chart_seconds_per_step");
    let (l, c2) = read_num(root, R_FLUID1_PRICE, &["params"], "l");
    PriceInputs {
        fluid1_seconds_per_step: s,
        fluid1_l: l,
        per_cell_per_step: s / (l * l),
        cite: format!("{c1}; {c2}"),
    }
}

/// Measure this carrier's own cost on the first `PRICE_STEPS` steps of the chart scene, in
/// the SAME MIX the counted runs advance in.
///
/// A counted run is a warm-up with the per-step ledger audit off and a sampling window with
/// it on, and the audited step costs a whole census pass more than the unaudited one. A price
/// measured entirely with the audit on projects a counted run several times over, which is a
/// price that cannot fail — so the measurement uses the run's own audited fraction,
/// `frames·stride / (warmup + frames·stride)`, and says so in its label.
fn measure_price(rules: EdgeRules, steps_projected: u64, label: &str) -> (Price, f64) {
    let sampled = (FRAMES * FRAME_STRIDE) as f64 / (WARMUP + FRAMES * FRAME_STRIDE) as f64;
    let audited = ((PRICE_STEPS as f64 * sampled).round() as usize).max(1);
    let plain = (PRICE_STEPS as usize).saturating_sub(audited);
    let mut g = uniform(L, SEED, CHART_DENSITY, rules);
    g.audit_every_step = false;
    let mut secs = advance(&mut g, plain);
    g.audit_every_step = true;
    g.reset_initial();
    secs += advance(&mut g, audited);
    let p = Price::measure(
        format!("{label} ({plain} unaudited + {audited} audited steps, the run's own mix)"),
        PRICE_STEPS,
        secs,
        steps_projected,
        (PRICE_LOW, PRICE_HIGH),
    )
    .expect("a price needs a denominator");
    let per_cell = secs / PRICE_STEPS as f64 / (L * L) as f64;
    (p, per_cell)
}

/// How many steps the counted phase takes, counted rather than guessed.
fn counted_steps() -> u64 {
    // the cluster arm, the pair CONTROL arm and the one-phase control
    let coex = (WARMUP + FRAMES * FRAME_STRIDE) * SEEDS * 3;
    let droplets = (DROP_LIFE_SETTLE + DROP_LIFE_CAP) * SEEDS * 2;
    let drops = (WARMUP + FRAMES * FRAME_STRIDE) * DROP_RADII.len() * SEEDS;
    let slabs = (WARMUP + FRAMES * FRAME_STRIDE) * SEEDS;
    let grav = GRAV_FRAMES * GRAV_STRIDE * SEEDS * 2; // with gravity and without
    let kt = G2_KT_STEPS + WARMUP;
    // The edge probe is not a stepped scene: it warms up, then restores and single-steps once
    // per movable cell per block size. Counted here in step-equivalents so the price covers
    // it; the restore itself is a state copy the step count does not carry, and the cost is a
    // fraction of a percent of the total either way.
    let edge = WARMUP / 4 + W_BLOCKS.len() * L_EDGE * L_EDGE * (W_STEPS + 1);
    (coex + droplets + drops + slabs + grav + kt + edge) as u64
}

// ---------------------------------------------------------------- the plants

/// Plant (i): the rent DOUBLED. The observable is the linear bond's stationary held fraction
/// on a held geometry, and its analytic reach is exact before anything runs:
/// `|f(E) − f(2E)| / f(E)` with `f(E) = 1/(1 + e^{−E})`.
fn plant_i(rent: f64, retention_cite: &str) -> Plant {
    let f = |e: f64| 1.0 / (1.0 + (-e).exp());
    let one = f(rent);
    let two = f(2.0 * rent);
    let reach = (one - two).abs() / one;
    let stake = Stake::derived(
        "the held fraction moves by half of what doubling the rent can move it",
        PLANT_STAKE_FRACTION * reach,
        &[],
        "stake = 0.5 * |f(E) - f(2E)| / f(E), f(E) = 1/(1+e^-E), computed from the chart's \
         own rent before any step: FLUID-1's plants were typed at 0.20 against a reach of \
         0.085 and both were unreachable by the freeze's own arithmetic",
    );
    Plant::new(
        "the rent doubled",
        "the bond's break probability, whose held fraction is the observable",
        one,
        PLANT_I_CARRIER_FLOOR,
        stake,
        reach,
        format!(
            "f(E) = 1/(1+e^-E) at E = {} gives {}, at 2E gives {}; relative reach {} \
             (the chart's rent is set from {})",
            jf(rent),
            jf(one),
            jf(two),
            jf(reach),
            retention_cite
        ),
    )
}

/// Plant (ii): gravity's rate DOUBLED. The observable is the momentum posted on the FIRST
/// step from a common initial state, and its analytic reach is exactly one in relative terms:
/// each cell fires when its own uniform draw is under the rate, so the cells firing at `r`
/// are a subset of those firing at `2r` and the expected count is exactly proportional to the
/// rate at a fixed state.
fn plant_ii() -> Plant {
    let reach = 1.0;
    let stake = Stake::derived(
        "the posted injection moves by half of what doubling the rate can move it",
        PLANT_STAKE_FRACTION * reach,
        &[],
        "stake = 0.5 * 1.0; the reach is exactly 1.0 because a cell fires when its uniform \
         draw is under the rate, so at a FIXED state the expected firing count -- and hence \
         the expected posted injection -- is exactly proportional to the rate",
    );
    Plant::new(
        "gravity's rate doubled",
        "the per-cell gravity draw, whose posted injection is the observable",
        GRAV_RATE,
        PLANT_II_CARRIER_FLOOR,
        stake,
        reach,
        "a uniform draw under `rate` fires; doubling the rate exactly doubles the expected \
         number of firing cells at a fixed state, so the relative reach is 1.0",
    )
}

/// Plant (i) measured: the held fraction on a held-geometry scene at `E` and at `2E`.
fn measure_plant_i(rules: EdgeRules) -> (f64, f64) {
    let held = |rent: f64| -> f64 {
        let mut r = rules;
        r.rent = rent;
        r.arms = 1;
        r.acceptors = 1;
        r.stream = false;
        let mut g = EdgeLattice::seeded_by(4, 0x5EED, r, |_, _| 0.0);
        let n = g.n;
        let rest = g.rest.expect("the seven-slot chart has a rest slot");
        let a_cell = 0usize;
        let b_cell = g.tables().neighbour_of(a_cell, 0);
        g.cells[a_cell] = 1 << rest;
        g.cells[b_cell] = 1 << rest;
        g.orient[a_cell * n + rest] = 0;
        g.orient[b_cell * n + rest] = 3; // phi = 0, the linear bond
        g.reset_initial();
        let tracked = a_cell * n + rest;
        let mut count = 0u64;
        for _ in 0..PLANT_I_STEPS {
            g.step();
            if g.donor_of[tracked * 2] != u32::MAX {
                count += 1;
            }
        }
        count as f64 / PLANT_I_STEPS as f64
    };
    (held(rules.rent), held(2.0 * rules.rent))
}

/// Plant (ii) measured: the first-step posted injection at `rate` and at `2·rate`, averaged
/// over seeds, from the SAME initial state in each pair.
fn measure_plant_ii(rules: EdgeRules) -> (f64, f64) {
    let run = |rate: f64| -> f64 {
        let mut acc = 0.0;
        for s in 0..PLANT_II_SEEDS {
            let mut r = rules;
            r.gravity = Some(Gravity { dir: GRAV_DIR, rate });
            let mut g = uniform(64, seed_of(s), CHART_DENSITY, r);
            g.step();
            let i = g.injected;
            acc += ((i[0] * i[0] + i[1] * i[1]) as f64).sqrt();
        }
        acc / PLANT_II_SEEDS as f64
    };
    (run(GRAV_RATE), run(2.0 * GRAV_RATE))
}

// ---------------------------------------------------------------- the params block

fn params_json(root: &str, table: &[Amp; N_ORIENT], rent: f64, pi: &PriceInputs) -> String {
    let mut s = String::new();
    let _ = write!(s, "{{\"freeze\": \"conformance/mesh/EDGE0_PREREG.md\"");
    let _ = write!(s, ", \"carrier\": \"holon_lattice::edge (FHP-II rest slot, two donor arms, the wait, gravity posted)\"");
    let _ = write!(s, ", \"instruments\": \"holon_lattice::surface\"");
    let _ = write!(s, ", \"records_root\": {root:?}");
    let _ = write!(s, ", \"l\": {L}, \"seed\": {SEED}, \"seeds\": {SEEDS}");
    let _ = write!(s, ", \"chart_density\": {}", jf(CHART_DENSITY));
    let _ = write!(s, ", \"chart_retention\": {}", jf(CHART_RETENTION));
    let _ = write!(s, ", \"chart_rent_e0_over_kt\": {}", jf(rent));
    let _ = write!(s, ", \"chart_d_liquid\": {}", jf(CHART_D_LIQUID));
    let _ = write!(s, ", \"chart_d_vapour\": {}", jf(CHART_D_VAPOUR));
    let _ = write!(s, ", \"b_hist\": {B_HIST}, \"hist_bins\": {HIST_BINS}, \"hist_min_sep\": {HIST_MIN_SEP}");
    let _ = write!(s, ", \"warmup\": {WARMUP}, \"frames\": {FRAMES}, \"frame_stride\": {FRAME_STRIDE}");
    let _ = write!(s, ", \"drop_radii\": {}", jarr(&DROP_RADII));
    let _ = write!(s, ", \"p_in_fraction\": {}, \"p_out_factor\": {}", jf(P_IN_FRACTION), jf(P_OUT_FACTOR));
    let _ = write!(s, ", \"laplace_min_r2\": {}", jf(LAPLACE_MIN_R2));
    let _ = write!(s, ", \"slab_j0\": {SLAB_J0}, \"slab_j1\": {SLAB_J1}");
    let _ = write!(s, ", \"cap_m_min\": {CAP_M_MIN}, \"cap_m_max\": {CAP_M_MAX}");
    let _ = write!(s, ", \"height_half_window\": {HEIGHT_HALF_WINDOW}, \"width_half_window\": {WIDTH_HALF_WINDOW}");
    let _ = write!(s, ", \"width_slope_bar\": {}", jf(WIDTH_SLOPE_BAR));
    let _ = write!(s, ", \"gravity_dir\": {GRAV_DIR}, \"gravity_rate\": {}", jf(GRAV_RATE));
    let _ = write!(s, ", \"w_blocks\": [4, 8, 16], \"w_steps\": {W_STEPS}, \"l_edge\": {L_EDGE}");
    let _ = write!(s, ", \"amplitude_table\": {}", amplitude_table_json(table));
    let _ = write!(
        s,
        ", \"rent_rule\": \"E0/kT_rent = ln(f/(1-f)); f is EDGE-0's OWN chart parameter, \
         swept on the screen, and CT-2's measured 0.5526 is where the molecular tier sits \
         on that sweep, never the chart itself\""
    );
    let _ = write!(s, ", \"fluid1_price_cite\": {:?}", pi.cite);
    let _ = write!(s, ", \"fluid1_seconds_per_step\": {}", jf(pi.fluid1_seconds_per_step));
    let _ = write!(s, ", \"fluid1_l\": {}", jf(pi.fluid1_l));
    let _ = write!(s, ", \"fluid1_seconds_per_cell_per_step\": {}", jf(pi.per_cell_per_step));
    s.push('}');
    s
}

// ---------------------------------------------------------------- main

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let phase = args.get(1).map(String::as_str).unwrap_or("gate");
    let root = records_root();
    let dir = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| format!("{root}/mesh/edge0"));
    let threads: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(4);
    std::fs::create_dir_all(&dir).expect("the output directory");
    match phase {
        // `--screen` is the same phase under the flag spelling the campaign brief uses.
        "screen" | "--screen" => screen(&dir, &root),
        // The chart's own confirmation pass, still a SCREEN and still dry.
        "screen_chart" | "--screen-chart" => screen_chart(&dir, &root),
        "gate" => gate(&dir, &root),
        "run" => run(&dir, &root, threads),
        "read" => read(&dir),
        other => panic!(
            "unknown phase {other:?}: screen (--screen) | screen_chart | gate | run | read"
        ),
    }
}

// ---------------------------------------------------------------- screen

/// The density × rent chart, swept coarsely to find WHERE coexistence appears on this
/// carrier. **Every file this writes is `dry: true` and nothing it writes enters a gate or a
/// claim** (GANTT2's screen law); the freeze pre-commits the chart FROM the measured
/// bimodality, and the screen is how it knows what to pre-commit.
fn screen(dir: &str, root: &str) {
    let w = RecordWriter::screen(dir, "EDGE-0 density x rent chart screen");
    let table = amplitude_table(root);
    let amp = amplitudes(&table);
    println!(
        "SCREEN  L={SCREEN_L} warmup={SCREEN_WARMUP} frames={SCREEN_FRAMES} b={SCREEN_B} \
         seeds={SCREEN_SEEDS}  (dry: nothing here is a reading)"
    );
    let t0 = Instant::now();

    // The one-phase floor at each density: bonds OFF, everything else the same.
    let mut floors = String::from("[");
    let mut floor_by_density: Vec<(f64, f64, f64, f64, f64)> = Vec::new();
    for (di, &d) in SCREEN_DENSITIES.iter().enumerate() {
        let runs: Vec<Coexistence> = (0..SCREEN_SEEDS)
            .map(|s| {
                coexistence(
                    SCREEN_L,
                    seed_of(s),
                    d,
                    no_bond(chart_rules(amp, 0.5526)),
                    SCREEN_WARMUP,
                    SCREEN_FRAMES,
                    SCREEN_STRIDE,
                    SCREEN_B,
                )
            })
            .collect();
        let _ = di;
        let dips: Vec<f64> = runs.iter().map(|c| c.dip).collect();
        let vars: Vec<f64> = runs.iter().map(|c| c.variance).collect();
        let fq05: Vec<f64> = runs.iter().map(|c| c.q05).collect();
        let fq95: Vec<f64> = runs.iter().map(|c| c.q95).collect();
        let (m, s, mx) = bimodality_floor(&dips);
        floor_by_density.push((d, m, s, mx, mean(&vars)));
        if di > 0 {
            floors.push_str(", ");
        }
        let _ = write!(
            floors,
            "{{\"density\": {}, \"floor_mean\": {}, \"floor_sd\": {}, \"floor_max\": {}, \
             \"dips\": {}, \"block_variance\": {}, \"q05\": {}, \"q95\": {}, \
             \"q95_over_q05\": {}}}",
            jf(d),
            jf(m),
            jf(s),
            jf(mx),
            jarr(&dips),
            jf(mean(&vars)),
            jf(mean(&fq05)),
            jf(mean(&fq95)),
            jf(mean(&fq95) / mean(&fq05))
        );
        println!(
            "  floor  d={d:.2}  dip {m:.4} +/- {s:.4}  max {mx:.4}  block variance {:.3e}  \
             q95/q05 {:.3}",
            mean(&vars),
            mean(&fq95) / mean(&fq05)
        );
    }
    floors.push(']');

    let mut rows = String::from("[");
    let mut first = true;
    let mut best: Option<(f64, f64, f64, f64, f64)> = None;
    let mut best_var: Option<(f64, f64, f64, MoverRule)> = None;
    for &d in SCREEN_DENSITIES.iter() {
      for &mover in SCREEN_RULES.iter() {
        for &f in SCREEN_RETENTIONS.iter() {
            let mut rules = chart_rules(amp, f);
            rules.mover = mover;
            let per_seed: Vec<Coexistence> = (0..SCREEN_SEEDS)
                .map(|s| {
                    coexistence(
                        SCREEN_L,
                        seed_of(s),
                        d,
                        rules,
                        SCREEN_WARMUP,
                        SCREEN_FRAMES,
                        SCREEN_STRIDE,
                        SCREEN_B,
                    )
                })
                .collect();
            let dips: Vec<f64> = per_seed.iter().map(|c| c.dip).collect();
            let lo: Vec<f64> = per_seed.iter().map(|c| c.mode_lo).collect();
            let hi: Vec<f64> = per_seed.iter().map(|c| c.mode_hi).collect();
            let rest: Vec<f64> = per_seed.iter().map(|c| c.rest_fraction).collect();
            let bpp: Vec<f64> = per_seed.iter().map(|c| c.bonds_per_particle).collect();
            let lg: Vec<f64> = per_seed.iter().map(|c| c.largest_fraction).collect();
            let var: Vec<f64> = per_seed.iter().map(|c| c.variance).collect();
            let rel_rate: Vec<f64> = per_seed.iter().map(|c| c.release_over_formed).collect();
            let q05: Vec<f64> = per_seed.iter().map(|c| c.q05).collect();
            let q50: Vec<f64> = per_seed.iter().map(|c| c.q50).collect();
            let q95: Vec<f64> = per_seed.iter().map(|c| c.q95).collect();
            let exact = per_seed.iter().all(|c| c.all_exact);
            let row = floor_by_density.iter().find(|x| x.0 == d);
            let floor = row.map(|x| x.3).unwrap_or(f64::NAN);
            let floor_var = row.map(|x| x.4).unwrap_or(f64::NAN);
            let excess = mean(&dips) - floor;
            let var_ratio = mean(&var) / floor_var;
            if !first {
                rows.push_str(", ");
            }
            first = false;
            let _ = write!(
                rows,
                "{{\"mover\": {:?}, \"density\": {}, \"retention\": {}, \"rent\": {}, \
                 \"dip_mean\": {}, \
                 \"dip_sd\": {}, \"dip_over_floor\": {}, \"mode_lo\": {}, \"mode_hi\": {}, \
                 \"rest_fraction\": {}, \"bonds_per_particle\": {}, \"largest_fraction\": {}, \
                 \"block_variance\": {}, \"variance_over_one_phase\": {}, \
                 \"release_over_formed\": {}, \"largest_component\": {}, \"q05\": {}, \
                 \"q50\": {}, \"q95\": {}, \"q95_over_q05\": {}, \"ledger_exact\": {}}}",
                format!("{mover:?}"),
                jf(d),
                jf(f),
                jf(EdgeRules::rent_from_retention(f)),
                jf(mean(&dips)),
                jf(sd(&dips)),
                jf(excess),
                jf(mean(&lo)),
                jf(mean(&hi)),
                jf(mean(&rest)),
                jf(mean(&bpp)),
                jf(mean(&lg)),
                jf(mean(&var)),
                jf(var_ratio),
                jf(mean(&rel_rate)),
                per_seed.iter().map(|c| c.largest_component).max().unwrap_or(0),
                jf(mean(&q05)),
                jf(mean(&q50)),
                jf(mean(&q95)),
                jf(mean(&q95) / mean(&q05)),
                exact
            );
            println!(
                "  {mover:?} d={d:.2} f={f:.4}  dip {:.4} (floor {floor:.4})  \
                 var/one-phase {var_ratio:.3}  modes {:.3}/{:.3}  rest {:.3}  bonds/p {:.3}  \
                 largest {:.4}  release/formed {:.3}  q05/q50/q95 {:.4}/{:.4}/{:.4}",
                mean(&dips),
                mean(&lo),
                mean(&hi),
                mean(&rest),
                mean(&bpp),
                mean(&lg),
                mean(&rel_rate),
                mean(&q05),
                mean(&q50),
                mean(&q95)
            );
            if best.map(|b| excess > b.4).unwrap_or(true) {
                best = Some((d, f, mean(&lo), mean(&hi), excess));
            }
            // THE SCREEN'S SELECTION RULE, stated here and in EDGE0_PREREG §0: the largest
            // block-density variance ratio among the points whose largest bonded component is
            // UNDER HALF the particles. The constraint is not decoration — a point where the
            // whole box is one component is a gel and has no droplet to have an edge, which
            // is the thing EDGE-0 measures, so a chart there would be a chart for a different
            // campaign.
            let percolated = mean(&lg) >= SCREEN_PERCOLATION_BAR;
            if mover == MoverRule::Cluster
                && !percolated
                && best_var.map(|b: (f64, f64, f64, MoverRule)| var_ratio > b.2).unwrap_or(true)
            {
                best_var = Some((d, f, var_ratio, mover));
            }
        }
      }
    }
    rows.push(']');

    // The droplet's survival: one bonded droplet in a sparse background, per rule, per
    // chart point, and the steps until its largest component falls under half its start. A
    // cap is a REFUSAL and is reported as one, and both absences are named.
    let mut drops = String::from("[");
    let mut first_drop = true;
    for &mover in SCREEN_RULES.iter() {
        for &d in SCREEN_DENSITIES.iter() {
            for &f in SCREEN_RETENTIONS.iter() {
                let mut rules = chart_rules(amp, f);
                rules.mover = mover;
                let lives: Vec<DropletLife> = (0..SCREEN_DROP_SEEDS)
                    .map(|s| {
                        droplet_lifetime(
                            SCREEN_L,
                            seed_of(s),
                            SCREEN_DROP_R,
                            CHART_D_LIQUID,
                            d,
                            rules,
                            SCREEN_DROP_SETTLE,
                            SCREEN_DROP_STRIDE,
                            SCREEN_DROP_CAP,
                        )
                    })
                    .collect();
                let life: Vec<f64> = lives.iter().map(|x| x.lifetime()).collect();
                let survived = lives.iter().filter(|x| x.capped).count();
                let absent = lives.iter().filter(|x| x.empty || x.unbound).count();
                let start: Vec<f64> = lives.iter().map(|x| x.start_largest as f64).collect();
                let died: Vec<f64> = life.iter().copied().filter(|x| x.is_finite()).collect();
                if !first_drop {
                    drops.push_str(", ");
                }
                first_drop = false;
                let _ = write!(
                    drops,
                    "{{\"mover\": {:?}, \"background_density\": {}, \"retention\": {}, \
                     \"start_largest_mean\": {}, \"lifetime_mean_steps\": {}, \
                     \"seeds\": {}, \"survived_to_cap\": {}, \"absent\": {}, \"cap\": {}}}",
                    format!("{mover:?}"),
                    jf(d),
                    jf(f),
                    jf(mean(&start)),
                    if died.is_empty() { "null".to_string() } else { jf(mean(&died)) },
                    SCREEN_DROP_SEEDS,
                    survived,
                    absent,
                    SCREEN_DROP_CAP
                );
                println!(
                    "  droplet {mover:?} bg={d:.2} f={f:.4}: start {:.0}, survived {survived}/{}, \
                     absent {absent}, lifetime {}",
                    mean(&start),
                    SCREEN_DROP_SEEDS,
                    if died.is_empty() {
                        "none died".to_string()
                    } else {
                        format!("{:.0} steps", mean(&died))
                    }
                );
            }
        }
    }
    drops.push(']');

    // The long-warm-up spot check at the sweep's best point: the same reading with the
    // warm-up ten times longer. "No bimodality" and "not yet coarsened" are different
    // readings and this is what tells them apart.
    let mut long_json = String::from("null");
    if let Some((d, f, vr, mover)) = best_var {
        let mut rules = chart_rules(amp, f);
        rules.mover = mover;
        let long: Vec<Coexistence> = (0..SCREEN_SEEDS)
            .map(|s| {
                coexistence(
                    SCREEN_L,
                    seed_of(s),
                    d,
                    rules,
                    SCREEN_WARMUP * SCREEN_LONG_FACTOR,
                    SCREEN_FRAMES,
                    SCREEN_STRIDE,
                    SCREEN_B,
                )
            })
            .collect();
        let dips: Vec<f64> = long.iter().map(|c| c.dip).collect();
        let vars: Vec<f64> = long.iter().map(|c| c.variance).collect();
        let lg: Vec<f64> = long.iter().map(|c| c.largest_fraction).collect();
        long_json = format!(
            "{{\"mover\": {:?}, \"density\": {}, \"retention\": {}, \"warmup\": {}, \
             \"dip_mean\": {}, \
             \"dip_sd\": {}, \"block_variance\": {}, \"largest_fraction\": {}, \
             \"short_warmup_variance_ratio\": {}}}",
            format!("{mover:?}"),
            jf(d),
            jf(f),
            SCREEN_WARMUP * SCREEN_LONG_FACTOR,
            jf(mean(&dips)),
            jf(sd(&dips)),
            jf(mean(&vars)),
            jf(mean(&lg)),
            jf(vr)
        );
        println!(
            "  long warm-up ({}x) at {mover:?} d={d:.2} f={f:.4}: dip {:.4} +/- {:.4}, block \
             variance {:.3e}, largest bonded component {:.4}",
            SCREEN_LONG_FACTOR,
            mean(&dips),
            sd(&dips),
            mean(&vars),
            mean(&lg)
        );
    }

    let secs = t0.elapsed().as_secs_f64();
    let mut rec = Record::new("screen")
        .raw("droplet_survival", drops)
        .raw("long_warmup_spot_check", long_json)
        .text("law", "screen: a labelled sweep; nothing here enters a gate or a claim")
        .int("l", SCREEN_L as i64)
        .int("warmup", SCREEN_WARMUP as i64)
        .int("frames", SCREEN_FRAMES as i64)
        .int("block", SCREEN_B as i64)
        .int("seeds", SCREEN_SEEDS as i64)
        .raw("one_phase_floor", floors)
        .raw("chart", rows)
        .number("seconds", secs);
    if let Some((d, f, vr, mover)) = best_var {
        rec = rec
            .text("best_variance_mover", &format!("{mover:?}"))
            .number("best_variance_density", d)
            .number("best_variance_retention", f)
            .number("best_variance_ratio", vr);
        println!(
            "  the cluster rule's largest block-density variance against the one-phase \
             control: d={d:.2} f={f:.4} ratio {vr:.3}  (a PRECURSOR, never a verdict)"
        );
    }
    if let Some((d, f, lo, hi, ex)) = best {
        rec = rec
            .number("best_density", d)
            .number("best_retention", f)
            .number("best_mode_lo", lo)
            .number("best_mode_hi", hi)
            .number("best_dip_over_floor", ex);
        println!(
            "\n  the sweep's largest excess over the one-phase floor: d={d:.2} f={f:.4} \
             modes {lo:.3}/{hi:.3} excess {ex:+.4}"
        );
    }
    w.write("screen.json", &rec).expect("screen.json writes");
    println!("  screen wrote {dir}/screen.json in {secs:.1} s (dry)");
}

/// One arm's reading on the chart's confirmation pass — named rather than left a tuple, so the
/// three arms are compared field by field and not by position.
struct ArmReading {
    q05: f64,
    q95: f64,
    spread: f64,
    spread_sd: f64,
    variance: f64,
}

/// The chart's own confirmation pass: the three arms at the pre-committed chart ONLY, at the
/// counted runs' own box and window, reporting the block-density QUANTILES the freeze reads
/// the two phase densities off.
///
/// **Still a screen and still dry.** It exists because the full sweep's two "modes" sat
/// exactly at the mode finder's `min_separation`, which is a fact about the constraint and
/// not about the distribution; the quantiles have no such knob. Nothing it writes enters a
/// gate or a claim.
fn screen_chart(dir: &str, root: &str) {
    let w = RecordWriter::screen(dir, "EDGE-0 chart confirmation (quantiles at the chart)");
    let table = amplitude_table(root);
    let amp = amplitudes(&table);
    let rules = chart_rules(amp, CHART_RETENTION);
    let mut pair_rules = rules;
    pair_rules.mover = MoverRule::Pair;
    println!(
        "SCREEN_CHART  L={L} d={CHART_DENSITY} f={CHART_RETENTION} b={B_HIST} seeds={SEEDS} \
         warmup={WARMUP} frames={FRAMES}  (dry: nothing here is a reading)"
    );
    let t0 = Instant::now();
    let arms: [(&str, EdgeRules); 3] =
        [("cluster", rules), ("pair", pair_rules), ("one_phase", no_bond(rules))];
    let mut rows = String::from("[");
    let mut out: Vec<ArmReading> = Vec::new();
    let (mut rho_lo, mut rho_hi, mut dense_frac) = (f64::NAN, f64::NAN, f64::NAN);
    for (k, (name, r)) in arms.iter().enumerate() {
        let runs: Vec<Coexistence> = (0..SEEDS)
            .map(|s| {
                coexistence(L, seed_of(s), CHART_DENSITY, *r, WARMUP, FRAMES, FRAME_STRIDE, B_HIST)
            })
            .collect();
        let g = |f: fn(&Coexistence) -> f64| mean(&runs.iter().map(f).collect::<Vec<_>>());
        let (q05, q50, q95) = (g(|c| c.q05), g(|c| c.q50), g(|c| c.q95));
        let (var, dip) = (g(|c| c.variance), g(|c| c.dip));
        let sp: Vec<f64> = runs.iter().map(|c| c.q95 - c.q05).collect();
        let (bpp, lg) = (g(|c| c.bonds_per_particle), g(|c| c.largest_fraction));
        let sep = runs.iter().filter(|c| c.separated).count();
        let exact = runs.iter().all(|c| c.all_exact);
        if k > 0 {
            rows.push_str(", ");
        }
        let _ = write!(
            rows,
            "{{\"arm\": {:?}, \"q05\": {}, \"q50\": {}, \"q95\": {}, \"q95_over_q05\": {}, \
             \"spread\": {}, \"spread_sd\": {}, \"rho_lo\": {}, \"rho_hi\": {}, \
             \"dense_fraction\": {}, \
             \"block_variance\": {}, \"dip\": {}, \"mode_lo\": {}, \"mode_hi\": {}, \
             \"seeds_separated\": {}, \"bonds_per_particle\": {}, \"largest_fraction\": {}, \
             \"release_over_formed\": {}, \"ledger_exact\": {}}}",
            name,
            jf(q05),
            jf(q50),
            jf(q95),
            jf(q95 / q05),
            jf(mean(&sp)),
            jf(sd(&sp)),
            jf(g(|c| c.rho_lo)),
            jf(g(|c| c.rho_hi)),
            jf(g(|c| c.dense_fraction)),
            jf(var),
            jf(dip),
            jf(g(|c| c.mode_lo)),
            jf(g(|c| c.mode_hi)),
            sep,
            jf(bpp),
            jf(lg),
            jf(g(|c| c.release_over_formed)),
            exact
        );
        println!(
            "  {name:9}  q05 {q05:.5}  q50 {q50:.5}  q95 {q95:.5}  q95/q05 {:.3}  \
             variance {var:.3e}  dip {dip:.4}  separated {sep}/{SEEDS}  bonds/p {bpp:.3}  \
             largest {lg:.4}  spread {:.5} +/- {:.5}  phases {:.5} / {:.5} (dense share {:.3})",
            q95 / q05,
            mean(&sp),
            sd(&sp),
            g(|c| c.rho_lo),
            g(|c| c.rho_hi),
            g(|c| c.dense_fraction)
        );
        if k == 0 {
            rho_lo = g(|c| c.rho_lo);
            rho_hi = g(|c| c.rho_hi);
            dense_frac = g(|c| c.dense_fraction);
        }
        out.push(ArmReading {
            q05,
            q95,
            spread: mean(&sp),
            spread_sd: sd(&sp),
            variance: var,
        });
    }
    rows.push(']');
    let ratio = |k: usize| out[k].q95 / out[k].q05.max(f64::MIN_POSITIVE);
    println!(
        "\n  the two phase densities at the chart: q05 = {:.5} and q95 = {:.5}, spread {:.5}; \
         the pair arm's spread is {:.5} and the one-phase control's {:.5} +/- {:.5}",
        out[0].q05,
        out[0].q95,
        out[0].spread,
        out[1].spread,
        out[2].spread,
        out[2].spread_sd
    );
    let band = 3.0 * (out[0].spread_sd * out[0].spread_sd + out[2].spread_sd * out[2].spread_sd).sqrt();
    println!(
        "  C's third leg, as the counted run will read it: cluster spread - one-phase spread \
         = {:.5} against a band of 3 sigma of their COMBINED seed spread = {:.5} ({:.1}x)",
        out[0].spread - out[2].spread,
        band,
        (out[0].spread - out[2].spread) / band.max(f64::MIN_POSITIVE)
    );
    println!(
        "  block-density variance against the one-phase control: cluster {:.3}, pair {:.3}",
        out[0].variance / out[2].variance,
        out[1].variance / out[2].variance
    );
    let secs = t0.elapsed().as_secs_f64();
    let rec = Record::new("screen_chart")
        .text("law", "screen: a labelled pass; nothing here enters a gate or a claim")
        .int("l", L as i64)
        .number("density", CHART_DENSITY)
        .number("retention", CHART_RETENTION)
        .int("block", B_HIST as i64)
        .int("seeds", SEEDS as i64)
        .int("warmup", WARMUP as i64)
        .int("frames", FRAMES as i64)
        .raw("arms", rows)
        .number("spread_cluster", out[0].spread)
        .number("spread_pair", out[1].spread)
        .number("spread_one_phase", out[2].spread)
        .number("spread_one_phase_sd", out[2].spread_sd)
        .number("spread_cluster_sd", out[0].spread_sd)
        .number("spread_pair_sd", out[1].spread_sd)
        .number("spread_gap", out[0].spread - out[2].spread)
        .number("rho_lo_cluster", rho_lo)
        .number("rho_hi_cluster", rho_hi)
        .number("dense_fraction_cluster", dense_frac)
        .number(
            "spread_band_3sigma_combined",
            3.0 * (out[0].spread_sd * out[0].spread_sd + out[2].spread_sd * out[2].spread_sd).sqrt(),
        )
        .number("q_ratio_cluster", ratio(0))
        .number("q_ratio_pair", ratio(1))
        .number("q_ratio_one_phase", ratio(2))
        .number("variance_ratio_cluster", out[0].variance / out[2].variance)
        .number("variance_ratio_pair", out[1].variance / out[2].variance)
        .number("seconds", secs);
    w.write("screen_chart.json", &rec).expect("screen_chart.json writes");
    println!("  wrote {dir}/screen_chart.json in {secs:.1} s (dry)");
}

// ---------------------------------------------------------------- gate

fn gate(dir: &str, root: &str) {
    let w = RecordWriter::new(dir);
    let mut r = Report::new();
    let table = amplitude_table(root);
    let amp = amplitudes(&table);
    let (retention_read, retention_cite) = read_num(root, R_CT2_ARMS, &["dimer_293_seam"], "f");
    let rent = EdgeRules::rent_from_retention(CHART_RETENTION);
    let rules = chart_rules(amp, CHART_RETENTION);
    let pi = price_inputs(root);
    // Filled by G0 and carried into the record, so the release's prongs are numbers a
    // reader can act on and not only prose in a gate's detail line.
    let g0_totals: String;
    println!("EDGE-0 gate");
    println!("  amplitude table READ: {}", amplitude_table_json(&table));
    println!("  CT-2's retention READ: {retention_read} from {retention_cite}");
    println!("  the chart's rent E0/kT_rent = {rent} at retention {CHART_RETENTION}");

    // ---------------- G0 — the ledger, exact, including the posted injection
    {
        let mut legs: Vec<(String, bool, f64)> = Vec::new();
        let mut work = 0u64;
        let mut totals = EdgeCounts::default();
        let configs: Vec<(&str, EdgeRules)> = vec![
            ("the chart", rules),
            ("the chart with gravity", {
                let mut x = rules;
                x.gravity = Some(Gravity { dir: GRAV_DIR, rate: GRAV_RATE });
                x
            }),
            ("the no-bond control", no_bond(rules)),
            ("the cold limit", {
                let mut x = rules;
                x.cold = true;
                x
            }),
            ("the wait switched off", {
                let mut x = rules;
                x.wait_enabled = false;
                x
            }),
        ];
        for (name, cfg) in &configs {
            for s in 0..2 {
                let mut g = uniform(G1_L * 2, seed_of(s), CHART_DENSITY, *cfg);
                advance(&mut g, G0_STEPS);
                work += g.audit.steps_checked;
                totals.add(&g.audit.totals);
                let bad = g.audit.failing_legs();
                legs.push((
                    format!("{name}, seed {s}: every conserved integer exact at every step"),
                    bad.is_empty(),
                    bad.len() as f64,
                ));
                if !bad.is_empty() {
                    for b in bad {
                        legs.push((format!("{name}, seed {s}: {b}"), false, 1.0));
                    }
                }
            }
        }
        let mut g0 = Gate::new("G0")
            .work(work)
            .detail(format!(
                "mass, Px - posted injection, Py - posted injection, red, the orientation \
                 total and its per-orientation census, and the bond balance formed - \
                 broken_rent - blocked = held, checked at EVERY step of {} configurations x 2 \
                 seeds x {G0_STEPS} steps. Totals, every prong apart because the split IS the \
                 carrier's own diagnosis: {} collisions, {} formed, {} broken by the rent, {} \
                 released (no vacancy {}, already claimed {}, no geometry {}), {} joint moves, \
                 {} anomalous hops, {} waits tried and {} taken, {} rest created and {} \
                 destroyed, gravity {} flips + {} from rest, {} refused; under rule 2b: {} \
                 components walked, {} moved as one, {} refused the drawn move, {} refused the \
                 wait as well, largest {}",
                configs.len(),
                totals.collisions_fired,
                totals.formed,
                totals.broken_rent,
                totals.blocked,
                totals.blocked_no_vacancy,
                totals.blocked_claimed,
                totals.blocked_no_geometry,
                totals.joint_moves,
                totals.anomalous_hops,
                totals.wait_attempts,
                totals.waited,
                totals.rest_created,
                totals.rest_destroyed,
                totals.gravity_flip,
                totals.gravity_rest,
                totals.gravity_refused,
                totals.components,
                totals.component_moves,
                totals.component_move_refused,
                totals.component_wait_refused,
                totals.largest_component
            ));
        for (n, p, v) in legs {
            g0 = g0.leg_at(n, p, v);
        }
        // M-VACUOUS-SUCCESS: a ledger gate that passed on no bonds, no waits and no gravity
        // has not been shown able to fail.
        g0_totals = format!(
            "{{\"collisions_fired\": {}, \"formed\": {}, \"broken_rent\": {}, \
             \"blocked\": {}, \"blocked_no_vacancy\": {}, \"blocked_claimed\": {}, \
             \"blocked_no_geometry\": {}, \"joint_moves\": {}, \"anomalous_hops\": {}, \
             \"wait_attempts\": {}, \"waited\": {}, \"wait_saved_fraction\": {}, \
             \"rest_created\": {}, \"rest_destroyed\": {}, \"gravity_flip\": {}, \
             \"gravity_rest\": {}, \"gravity_refused\": {}, \"components\": {}, \
             \"component_moves\": {}, \"component_move_refused\": {}, \
             \"component_wait_refused\": {}, \"largest_component\": {}, \
             \"released_by_rent_over_released_by_mover\": {}, \"steps_checked\": {}}}",
            totals.collisions_fired,
            totals.formed,
            totals.broken_rent,
            totals.blocked,
            totals.blocked_no_vacancy,
            totals.blocked_claimed,
            totals.blocked_no_geometry,
            totals.joint_moves,
            totals.anomalous_hops,
            totals.wait_attempts,
            totals.waited,
            jf(totals.waited as f64 / (totals.wait_attempts.max(1)) as f64),
            totals.rest_created,
            totals.rest_destroyed,
            totals.gravity_flip,
            totals.gravity_rest,
            totals.gravity_refused,
            totals.components,
            totals.component_moves,
            totals.component_move_refused,
            totals.component_wait_refused,
            totals.largest_component,
            jf(totals.broken_rent as f64 / totals.blocked.max(1) as f64),
            work
        );
        g0 = g0
            .leg_at("bonds formed", totals.formed > 0, totals.formed as f64)
            .leg_at("the wait fired", totals.waited > 0, totals.waited as f64)
            .leg_at("rest particles created", totals.rest_created > 0, totals.rest_created as f64)
            .leg_at(
                "components were walked and moved as one",
                totals.components > 0 && totals.component_moves > 0,
                totals.component_moves as f64,
            )
            .leg_at(
                "and a component larger than a pair was built",
                totals.largest_component > 2,
                totals.largest_component as f64,
            )
            .leg_at("gravity posted", totals.gravity_flip > 0, totals.gravity_flip as f64);
        r.gate(g0);
    }

    // ---------------- G1 — the no-bond / no-gravity control IS FLUID-1's carrier
    {
        let m = Model::fhp6();
        let law = m.fhp_i(true);
        let o_rules = OrientationRules {
            bonds_enabled: true,
            amplitude: amp,
            rent: OrientationRules::rent_from_retention(retention_read),
            cold: false,
            stream: true,
        };
        let mut e_rules = EdgeRules::fluid1(amp, EdgeRules::rent_from_retention(retention_read));
        e_rules.gravity = None;
        let base = Lattice::seeded(m, G1_L, SEED, 0.2, law.clone());
        let colour = base.seed_colour_wave(SEED, 1.0, 1);
        let mut a = OrientationLattice::from_lattice(base.clone(), SEED, o_rules)
            .with_colour(colour.clone());
        let mut b = EdgeLattice::from_lattice(base, vec![law], 0, SEED, e_rules).with_colour(colour);
        let (mut cells, mut orient, mut col, mut bonds) = (true, true, true, true);
        let mut bonded_steps = 0u64;
        for _ in 0..G1_STEPS {
            a.step();
            b.step();
            cells &= a.cells == b.cells;
            orient &= a.orient == b.orient;
            col &= a.colour == b.colour;
            bonds &= a.bonds.len() == b.bonds.len()
                && a.bonds.iter().zip(&b.bonds).all(|(x, y)| (x.donor, x.acceptor) == (y.donor, y.acceptor));
            if !a.bonds.is_empty() {
                bonded_steps += 1;
            }
        }
        let g1 = Gate::new("G1")
            .work(G1_STEPS as u64)
            .detail(format!(
                "with ONE donor arm, ONE acceptor role, no rest slot, the wait off and no \
                 gravity, this carrier and `orientation.rs`'s advance the same state: L={G1_L}, \
                 {G1_STEPS} steps of a BONDED run, FLUID-1's own seed, compared bit for bit. \
                 FLUID-1's own counts: {} formed, {} broken by the rent, {} blocked",
                a.audit.totals.formed, a.audit.totals.broken_rent, a.audit.totals.blocked
            ))
            .leg("the occupation is bit-identical", cells)
            .leg("the orientation plane is bit-identical", orient)
            .leg("the colour plane is bit-identical", col)
            .leg("every bond is the same pair of slots", bonds)
            .leg_at(
                "formed / broken / blocked agree",
                a.audit.totals.formed == b.audit.totals.formed
                    && a.audit.totals.broken_rent == b.audit.totals.broken_rent
                    && a.audit.totals.blocked == b.audit.totals.blocked,
                b.audit.totals.formed as f64,
            )
            .leg_at(
                "the identity was measured on a scene WITH bonds in it",
                bonded_steps > (G1_STEPS as u64) * 4 / 5,
                bonded_steps as f64,
            )
            .leg_at("no rest slot means the wait never fires", b.audit.totals.waited == 0, b.audit.totals.waited as f64)
            .leg("both ledgers exact", a.audit.all_exact() && b.audit.all_exact());
        r.gate(g1);
    }

    // ---------------- G2 — each instrument on a scene whose answer is known
    {
        let embed = hex_embed(&Model::fhp7());
        let free = no_bond(rules);
        let mut work = 0u64;
        let mut g2 = Gate::new("G2").detail(
            "the five instruments on scenes whose answers are known before the instrument \
             sees them: a planted two-mode histogram, one particle's own outer product and \
             the free gas's c_s^2, a planted Laplace line, a height field built FROM a chosen \
             sigma, a planted tanh profile, and an empty lattice's EMPTY edge",
        );

        // (a) the dip on a planted histogram: two modes with an empty valley read exactly 1.
        let two: Vec<f64> = (0..500).map(|k| if k % 2 == 0 { 0.105 } else { 0.605 }).collect();
        let bi = bimodality(&two, HIST_BINS, HIST_MIN_SEP);
        let one: Vec<f64> = (0..500).map(|_| 0.305).collect();
        let uni = bimodality(&one, HIST_BINS, HIST_MIN_SEP);
        let flat: Vec<f64> = (0..1000).map(|k| (k % HIST_BINS) as f64 / HIST_BINS as f64 + 0.005).collect();
        let fl = bimodality(&flat, HIST_BINS, HIST_MIN_SEP);
        work += 3;
        g2 = g2
            .leg_at("(a) two planted modes read a dip of exactly 1", bi.dip == 1.0, bi.dip)
            .leg_at("(a) one planted mode reads UNSEPARATED, not a small dip", !uni.separated, uni.dip)
            .leg_at("(a) a uniform spread reads a dip of exactly 0", fl.dip == 0.0, fl.dip);

        // (b) one particle reads its own outer product; the free gas reads c_s^2 = 1/2; a
        //     planted Laplace line comes back with its slope.
        let mut worst_outer = 0.0f64;
        for d in 0..N_ORIENT {
            let mut g = EdgeLattice::seeded_by(8, 5, free, |_, _| 0.0).with_flux();
            g.cells[20] = 1 << d;
            g.orient[20 * g.n + d] = 0;
            g.reset_initial();
            advance(&mut g, 1);
            let p = pressure_over(&g, |c| c == 20);
            let c = embed[d];
            worst_outer = worst_outer
                .max((p.xx - c[0] * c[0]).abs())
                .max((p.yy - c[1] * c[1]).abs())
                .max((p.xy - c[0] * c[1]).abs());
        }
        let mut gas = uniform(64, SEED, 0.3, free).with_flux();
        advance(&mut gas, 50);
        gas.clear_flux();
        advance(&mut gas, G2_KT_STEPS);
        let kt = ideal_gas_kt(&gas);
        let aniso = pressure(&gas).anisotropy();
        let sigma_planted = 0.0375f64;
        let dp: Vec<f64> = DROP_RADII.iter().map(|r| sigma_planted / r - 0.002).collect();
        let lf = laplace_sigma(&DROP_RADII, &dp);
        let flat_dp: Vec<f64> = DROP_RADII.iter().map(|_| 0.01).collect();
        let lflat = laplace_sigma(&DROP_RADII, &flat_dp);
        work += (N_ORIENT + 3) as u64;
        g2 = g2
            .leg_at("(b) one particle reads c (x) c exactly", worst_outer < G2_EXACT_TOL, worst_outer)
            .leg_at("(b) the free gas reads c_s^2 = 1/2", (kt - G2_CS2).abs() < G2_CS2_TOL, kt)
            .leg_at("(b) the free gas's pressure is isotropic", aniso < 0.05, aniso)
            .leg_at(
                "(b) a planted Laplace line returns its sigma",
                rel(lf.slope, sigma_planted) < G2_EXACT_TOL,
                lf.slope,
            )
            .leg_at("(b) a FLAT jump does not fit a Laplace line", !(lflat.r2 > 0.5), lflat.slope);

        // (c) a height field built FROM a chosen sigma returns that sigma.
        let l_c = 128usize;
        let kt_c = 0.5f64;
        let sig_c = 0.08f64;
        let mut h = vec![0.0f64; l_c];
        for m in 1..l_c / 2 {
            let k = 2.0 * std::f64::consts::PI * m as f64 / l_c as f64;
            let a = 2.0 * (kt_c / (sig_c * l_c as f64 * k * k)).sqrt();
            let phase = 0.7 * m as f64;
            for (x, hv) in h.iter_mut().enumerate() {
                *hv += a * (k * x as f64 + phase).cos();
            }
        }
        let cap = capillary(&[h], kt_c, CAP_M_MIN, CAP_M_MAX);
        work += 1;
        g2 = g2
            .leg_at("(c) a planted capillary spectrum returns its sigma", rel(cap.sigma, sig_c) < 1e-8, cap.sigma)
            .leg_at("(c) and it is a line", (cap.fit.r2 - 1.0).abs() < 1e-8, cap.fit.r2);

        // (d) a planted tanh returns its width and its centre.
        let mut worst_w = 0.0f64;
        let mut worst_c = 0.0f64;
        for (wv, cv) in [(1.5f64, 40.3f64), (4.0, 33.0), (0.8, 45.7)] {
            let (hi, lo) = (0.62f64, 0.08f64);
            let profile: Vec<f64> = (0..64)
                .map(|j| 0.5 * (hi + lo) - 0.5 * (hi - lo) * ((j as f64 - cv) / wv).tanh())
                .collect();
            let f = interface_width(&profile, 20, 62, hi, lo);
            worst_w = worst_w.max((f.width - wv).abs());
            worst_c = worst_c.max((f.centre - cv).abs());
        }
        work += 3;
        g2 = g2
            .leg_at("(d) a planted tanh returns its width", worst_w < 0.05, worst_w)
            .leg_at("(d) and its centre", worst_c < 0.1, worst_c);

        // (e) an empty lattice's edge is EMPTY (M-EMPTY-SECTOR), b = L is vacuous, and a
        //     droplet in vacuum leaves the outside closed.
        let empty = EdgeLattice::seeded_by(16, 3, free, |_, _| 0.0);
        let (e_empty, v_empty) = edge_at(&empty, 4, W_STEPS);
        let seeded = uniform(16, 11, 0.3, free);
        let (e_glob, _) = edge_at(&seeded, 16, W_STEPS);
        let drop = droplet_scene(32, 4, 8.0, 0.6, 0.0, free);
        let (e_drop, v_drop) = edge_at(&drop, 4, W_STEPS);
        let centroid = e_drop.position.unwrap_or([f64::NAN, f64::NAN]);
        work += 3;
        g2 = g2
            .leg_at("(e) an empty lattice's edge is EMPTY, not zero-width", e_empty.empty && e_empty.width.is_nan(), v_empty.probes as f64)
            .leg("(e) the b = L chart is labelled vacuous", e_glob.vacuous)
            .leg_at("(e) a droplet in vacuum leaves the outside closed", e_drop.fraction < 0.7 && !e_drop.empty, e_drop.fraction)
            .leg_at(
                "(e) and its edge sits on the droplet",
                (centroid[0] - 16.0).abs() < 6.0 && (centroid[1] - 16.0).abs() < 6.0,
                centroid[0],
            )
            .leg_at("(e) the droplet scene was actually probed", v_drop.probes > 0, v_drop.probes as f64);

        let g2 = g2.work(work);
        r.gate(g2);
    }

    // ---------------- the plants, PRE-CHECKED before any measurement
    let mut p_i = plant_i(rent, &retention_cite);
    let mut p_ii = plant_ii();
    println!("{}", p_i.print());
    println!("{}", p_ii.print());
    let pre_i = p_i.pre_check();
    let pre_ii = p_ii.pre_check();
    if pre_i.is_none() {
        let (a, b) = measure_plant_i(rules);
        p_i = p_i.measured((a - b).abs() / a);
        println!("  plant (i) measured: held {a:.6} at E, {b:.6} at 2E");
    }
    if pre_ii.is_none() {
        let (a, b) = measure_plant_ii(rules);
        p_ii = p_ii.measured((b - a).abs() / a);
        println!("  plant (ii) measured: injection {a:.4} at rate, {b:.4} at 2*rate");
    }
    r.gate(p_i.gate());
    r.gate(p_ii.gate());

    // ---------------- the price, WRITTEN before anything is counted
    let steps = counted_steps();
    let (price, per_cell) = measure_price(rules, steps, "the EDGE-0 counted runs");
    println!("{}", price.print());
    let overhead = per_cell / pi.per_cell_per_step;
    // The cluster rule's own step, priced apart: component finding is the new cost and a
    // freeze that runs rule 2b must not be priced on rule 2's step.
    let mut pair_rules = rules;
    pair_rules.mover = MoverRule::Pair;
    let (_, per_cell_pair) = measure_price(pair_rules, steps, "the pair-rule control arm");
    let cluster_overhead = per_cell / per_cell_pair;
    println!(
        "  this carrier costs {per_cell:.6e} s per cell per step against FLUID-1's \
         {:.6e} — an overhead of {overhead:.3}x, measured, on {} ({})",
        pi.per_cell_per_step, PRICE_STEPS, pi.cite
    );
    println!(
        "  the cluster rule's step costs {per_cell:.6e} against the pair rule's \
         {per_cell_pair:.6e} on the same scene — component finding is {cluster_overhead:.3}x"
    );
    price.clone().write(&w, "price.json").expect("the price writes before anything is counted");

    let rec = Record::new("gate")
        .raw("params", params_json(root, &table, rent, &pi))
        .raw("gates", r.json())
        .plant(&p_i)
        .plant(&p_ii)
        .number("retention_read", retention_read)
        .text("retention_source", &retention_cite)
        .number("rent_e0_over_kt", rent)
        .number("seconds_per_cell_per_step", per_cell)
        .number("fluid1_seconds_per_cell_per_step", pi.per_cell_per_step)
        .number("carrier_overhead_over_fluid1", overhead)
        .number("seconds_per_cell_per_step_pair_arm", per_cell_pair)
        .number("cluster_over_pair_step_cost", cluster_overhead)
        .text("mover_rule", &format!("{:?}", rules.mover))
        .int("counted_steps_projected", steps as i64)
        .raw("g0_totals", g0_totals)
        .flag("admits", r.admits());
    w.write("gate.json", &rec).expect("gate.json writes");
    for l in r.refusals() {
        println!("REFUSED {l}");
    }
    println!("\n  admits: {}", r.admits());
}

// ---------------------------------------------------------------- run

fn run(dir: &str, root: &str, threads: usize) {
    let w = RecordWriter::new(dir);
    let table = amplitude_table(root);
    let amp = amplitudes(&table);
    let rent = EdgeRules::rent_from_retention(CHART_RETENTION);
    let rules = chart_rules(amp, CHART_RETENTION);
    let pi = price_inputs(root);
    let steps = counted_steps();
    let (price, _) = measure_price(rules, steps, "the EDGE-0 counted runs");
    let priced: Priced = price.write(&w, "price.json").expect("the price writes");
    let t0 = Instant::now();

    // ---- the mechanical kT, MEASURED on this carrier's own free gas.
    let mut gas = uniform(L, SEED, CHART_DENSITY, no_bond(rules)).with_flux();
    warm(&mut gas, WARMUP);
    gas.clear_flux();
    advance(&mut gas, G2_KT_STEPS);
    let kt_mech = ideal_gas_kt(&gas);
    println!("  kT_mech (this carrier's own c_s^2, measured) = {kt_mech:.6}");

    // ---- coexistence at the chart: the CLUSTER arm, the PAIR arm as the pre-committed
    //      control, and the one-phase control (bonds forbidden) that both are measured
    //      against. All three at the same density, the same seeds and the same window.
    let mut pair_rules = rules;
    pair_rules.mover = MoverRule::Pair;
    let chart_runs: Vec<Coexistence> = parallel_map(SEEDS, threads, |s| {
        coexistence(L, seed_of(s), CHART_DENSITY, rules, WARMUP, FRAMES, FRAME_STRIDE, B_HIST)
    });
    let pair_runs: Vec<Coexistence> = parallel_map(SEEDS, threads, |s| {
        coexistence(L, seed_of(s), CHART_DENSITY, pair_rules, WARMUP, FRAMES, FRAME_STRIDE, B_HIST)
    });
    let control_runs: Vec<Coexistence> = parallel_map(SEEDS, threads, |s| {
        coexistence(L, seed_of(s), CHART_DENSITY, no_bond(rules), WARMUP, FRAMES, FRAME_STRIDE, B_HIST)
    });
    let chart_dips: Vec<f64> = chart_runs.iter().map(|c| c.dip).collect();
    let control_dips: Vec<f64> = control_runs.iter().map(|c| c.dip).collect();
    let (floor_mean, floor_sd, floor_max) = bimodality_floor(&control_dips);
    let var_one = mean(&control_runs.iter().map(|c| c.variance).collect::<Vec<_>>());
    let var_cluster = mean(&chart_runs.iter().map(|c| c.variance).collect::<Vec<_>>());
    let var_pair = mean(&pair_runs.iter().map(|c| c.variance).collect::<Vec<_>>());
    let mode_lo = mean(&chart_runs.iter().map(|c| c.mode_lo).collect::<Vec<_>>());
    let mode_hi = mean(&chart_runs.iter().map(|c| c.mode_hi).collect::<Vec<_>>());
    let separated = chart_runs.iter().filter(|c| c.separated).count();
    let qm = |v: &[Coexistence], f: fn(&Coexistence) -> f64| mean(&v.iter().map(f).collect::<Vec<_>>());
    let (q05_c, q50_c, q95_c) =
        (qm(&chart_runs, |c| c.q05), qm(&chart_runs, |c| c.q50), qm(&chart_runs, |c| c.q95));
    let (q05_p, q95_p) = (qm(&pair_runs, |c| c.q05), qm(&pair_runs, |c| c.q95));
    let (q05_o, q95_o) = (qm(&control_runs, |c| c.q05), qm(&control_runs, |c| c.q95));
    // The block-density SPREAD per seed, and the one-phase control's own spread over seeds —
    // which is the bar C's third leg is read against, and is measured rather than typed.
    let spread = |v: &[Coexistence]| -> Vec<f64> { v.iter().map(|c| c.q95 - c.q05).collect() };
    let (sp_c, sp_p, sp_o) = (spread(&chart_runs), spread(&pair_runs), spread(&control_runs));

    // ---- the droplet's survival at the chart, both arms.
    let lives: Vec<DropletLife> = parallel_map(SEEDS * 2, threads, |k| {
        let (arm, s) = (k / SEEDS, k % SEEDS);
        let r = if arm == 0 { rules } else { pair_rules };
        droplet_lifetime(
            L,
            seed_of(s),
            DROP_LIFE_R,
            CHART_D_LIQUID,
            CHART_D_VAPOUR,
            r,
            DROP_LIFE_SETTLE,
            DROP_LIFE_STRIDE,
            DROP_LIFE_CAP,
        )
    });
    let life_of = |arm: usize| -> (f64, usize, usize, f64) {
        let a = &lives[arm * SEEDS..(arm + 1) * SEEDS];
        let died: Vec<f64> = a.iter().map(|x| x.lifetime()).filter(|x| x.is_finite()).collect();
        (
            if died.is_empty() { f64::NAN } else { mean(&died) },
            a.iter().filter(|x| x.capped).count(),
            a.iter().filter(|x| x.empty || x.unbound).count(),
            mean(&a.iter().map(|x| x.start_largest as f64).collect::<Vec<_>>()),
        )
    };
    let (life_c, cap_c, absent_c, start_c) = life_of(0);
    let (life_p, cap_p, absent_p, start_p) = life_of(1);

    // ---- Laplace's law.
    let drops: Vec<Drop> = parallel_map(DROP_RADII.len() * SEEDS, threads, |k| {
        let (ri, s) = (k / SEEDS, k % SEEDS);
        droplet(L, seed_of(s), DROP_RADII[ri], rules, WARMUP, FRAMES * FRAME_STRIDE)
    });
    let mut sigma_l_per_seed = Vec::new();
    for s in 0..SEEDS {
        let radii: Vec<f64> = DROP_RADII.to_vec();
        let dp: Vec<f64> = (0..DROP_RADII.len()).map(|ri| drops[ri * SEEDS + s].dp).collect();
        sigma_l_per_seed.push(laplace_sigma(&radii, &dp));
    }
    let sigma_l: Vec<f64> = sigma_l_per_seed.iter().map(|f| f.slope).collect();
    let r2_l: Vec<f64> = sigma_l_per_seed.iter().map(|f| f.r2).collect();

    // ---- the capillary spectrum and the interface width.
    let slabs: Vec<Slab> = parallel_map(SEEDS, threads, |s| {
        slab(L, seed_of(s), rules, WARMUP, FRAMES, FRAME_STRIDE)
    });
    let mut sigma_c = Vec::new();
    let mut r2_c = Vec::new();
    let mut width_slope = Vec::new();
    let mut width_mean = Vec::new();
    for sl in &slabs {
        if sl.heights.is_empty() {
            sigma_c.push(f64::NAN);
            r2_c.push(f64::NAN);
        } else {
            let c = capillary(&sl.heights, kt_mech, CAP_M_MIN, CAP_M_MAX);
            sigma_c.push(c.sigma);
            r2_c.push(c.fit.r2);
        }
        let t: Vec<f64> = (0..sl.widths.len()).map(|k| (k * FRAME_STRIDE) as f64 / 1000.0).collect();
        width_slope.push(line_fit(&t, &sl.widths).slope);
        width_mean.push(mean(&sl.widths));
    }

    // ---- gravity: the dense phase settling, and the control with gravity off.
    let grav_on: Vec<Settling> =
        parallel_map(SEEDS, threads, |s| settling(L, seed_of(s), rules, true));
    let grav_off: Vec<Settling> =
        parallel_map(SEEDS, threads, |s| settling(L, seed_of(s), rules, false));

    // ---- the block chart's edge, on a droplet, at the three block sizes.
    let mut edges = String::from("[");
    {
        let drop = droplet_scene(L_EDGE, SEED, L_EDGE as f64 / 5.0, CHART_D_LIQUID, CHART_D_VAPOUR, rules);
        let mut g = drop;
        warm(&mut g, WARMUP / 4);
        for (bi, &b) in W_BLOCKS.iter().enumerate() {
            let (e, v) = edge_at(&g, b, W_STEPS);
            if bi > 0 {
                edges.push_str(", ");
            }
            let _ = write!(
                edges,
                "{{\"b\": {b}, \"cells_total\": {}, \"edge_cells\": {}, \"fraction\": {}, \
                 \"width\": {}, \"empty\": {}, \"vacuous\": {}, \"probes\": {}}}",
                e.cells_total,
                e.cells.len(),
                jf(e.fraction),
                jf(e.width),
                e.empty,
                e.vacuous,
                v.probes
            );
        }
    }
    edges.push(']');

    let grav_slope_on: Vec<f64> = grav_on.iter().map(|s| s.slope).collect();
    let grav_slope_off: Vec<f64> = grav_off.iter().map(|s| s.slope).collect();
    let grav_injected: Vec<f64> = grav_on
        .iter()
        .map(|s| ((s.injected[0] * s.injected[0] + s.injected[1] * s.injected[1]) as f64).sqrt())
        .collect();

    let secs = t0.elapsed().as_secs_f64();
    let check = priced.check(secs);
    println!("{}", check.line());

    let mut drops_json = String::from("[");
    for (k, d) in drops.iter().enumerate() {
        if k > 0 {
            drops_json.push_str(", ");
        }
        let _ = write!(
            drops_json,
            "{{\"r\": {}, \"seed\": {}, \"p_in\": {}, \"p_out\": {}, \"dp\": {}, \
             \"cells_in\": {}, \"cells_out\": {}, \"anisotropy_in\": {}, \"ledger_exact\": {}}}",
            jf(d.r),
            k % SEEDS,
            jf(d.p_in),
            jf(d.p_out),
            jf(d.dp),
            d.cells_in,
            d.cells_out,
            jf(d.anisotropy_in),
            d.all_exact
        );
    }
    drops_json.push(']');

    let mut slab_json = String::from("[");
    for (k, s) in slabs.iter().enumerate() {
        if k > 0 {
            slab_json.push_str(", ");
        }
        let _ = write!(
            slab_json,
            "{{\"seed\": {k}, \"rho_hi\": {}, \"rho_lo\": {}, \"widths\": {}, \"centres\": {}, \
             \"fit_rms\": {}, \"frames_with_holes\": {}, \"profile\": {}, \"ledger_exact\": {}}}",
            jf(s.rho_hi),
            jf(s.rho_lo),
            jarr(&s.widths),
            jarr(&s.centres),
            jarr(&s.rms),
            s.frames_with_holes,
            jarr(&s.profile),
            s.all_exact
        );
    }
    slab_json.push(']');

    let mut grav_json = String::from("[");
    for (k, (on, off)) in grav_on.iter().zip(&grav_off).enumerate() {
        if k > 0 {
            grav_json.push_str(", ");
        }
        let _ = write!(
            grav_json,
            "{{\"seed\": {k}, \"slope_on\": {}, \"r2_on\": {}, \"slope_off\": {}, \
             \"r2_off\": {}, \"injected\": [{}, {}], \"gravity_flip\": {}, \
             \"gravity_rest\": {}, \"gravity_refused\": {}, \"com_on\": {}, \
             \"ledger_exact\": {}}}",
            jf(on.slope),
            jf(on.r2),
            jf(off.slope),
            jf(off.r2),
            on.injected[0],
            on.injected[1],
            on.counts.gravity_flip,
            on.counts.gravity_rest,
            on.counts.gravity_refused,
            jarr(&on.com),
            on.all_exact && off.all_exact
        );
    }
    grav_json.push(']');

    let mut pair_json = String::from("[");
    for (k, c) in pair_runs.iter().enumerate() {
        if k > 0 {
            pair_json.push_str(", ");
        }
        let _ = write!(
            pair_json,
            "{{\"seed\": {k}, \"dip\": {}, \"mode_lo\": {}, \"mode_hi\": {}, \
             \"separated\": {}, \"variance\": {}, \"bonds_per_particle\": {}, \
             \"largest_fraction\": {}, \"release_over_formed\": {}, \"ledger_exact\": {}}}",
            jf(c.dip),
            jf(c.mode_lo),
            jf(c.mode_hi),
            c.separated,
            jf(c.variance),
            jf(c.bonds_per_particle),
            jf(c.largest_fraction),
            jf(c.release_over_formed),
            c.all_exact
        );
    }
    pair_json.push(']');

    let mut coex_json = String::from("[");
    for (k, (c, ctl)) in chart_runs.iter().zip(&control_runs).enumerate() {
        if k > 0 {
            coex_json.push_str(", ");
        }
        let _ = write!(
            coex_json,
            "{{\"seed\": {k}, \"dip\": {}, \"mode_lo\": {}, \"mode_hi\": {}, \
             \"separated\": {}, \"mean_density\": {}, \"rest_fraction\": {}, \
             \"bonds_per_particle\": {}, \"largest_fraction\": {}, \"counts\": {}, \
             \"control_dip\": {}, \"control_separated\": {}, \"ledger_exact\": {}, \
             \"failing_legs\": {:?}}}",
            jf(c.dip),
            jf(c.mode_lo),
            jf(c.mode_hi),
            c.separated,
            jf(c.mean_density),
            jf(c.rest_fraction),
            jf(c.bonds_per_particle),
            jf(c.largest_fraction),
            jarr_u(&c.counts),
            jf(ctl.dip),
            ctl.separated,
            c.all_exact && ctl.all_exact,
            [c.failing.join("; "), ctl.failing.join("; ")].join(" | ")
        );
    }
    coex_json.push(']');

    let rec = Record::new("run")
        .raw("params", params_json(root, &table, rent, &pi))
        .number("kt_mech_measured", kt_mech)
        .raw("coexistence", coex_json)
        .number("dip_mean", mean(&chart_dips))
        .number("dip_sd", sd(&chart_dips))
        .number("floor_mean", floor_mean)
        .number("floor_sd", floor_sd)
        .number("floor_max", floor_max)
        .text("chart_mover", &format!("{:?}", rules.mover))
        .number("variance_one_phase", var_one)
        .number("variance_cluster_arm", var_cluster)
        .number("variance_pair_arm", var_pair)
        .number("variance_ratio_cluster_arm", var_cluster / var_one)
        .number("variance_ratio_pair_arm", var_pair / var_one)
        .number("mode_lo", mode_lo)
        .number("mode_hi", mode_hi)
        .number("mode_ratio", mode_hi / mode_lo)
        .number("q05_cluster_arm", q05_c)
        .number("q50_cluster_arm", q50_c)
        .number("q95_cluster_arm", q95_c)
        .number("q_ratio_cluster_arm", q95_c / q05_c)
        .number("q_ratio_pair_arm", q95_p / q05_p)
        .number("q_ratio_one_phase", q95_o / q05_o)
        .number("q05_pair_arm", q05_p)
        .number("q95_pair_arm", q95_p)
        .number("q05_one_phase", q05_o)
        .number("q95_one_phase", q95_o)
        .number("spread_cluster_arm", mean(&sp_c))
        .number("spread_cluster_arm_sd", sd(&sp_c))
        .number("spread_pair_arm", mean(&sp_p))
        .number("spread_pair_arm_sd", sd(&sp_p))
        .number("spread_one_phase", mean(&sp_o))
        .number("spread_one_phase_sd", sd(&sp_o))
        .number("rho_lo_cluster_arm", qm(&chart_runs, |c| c.rho_lo))
        .number("rho_hi_cluster_arm", qm(&chart_runs, |c| c.rho_hi))
        .number("dense_fraction_cluster_arm", qm(&chart_runs, |c| c.dense_fraction))
        .number("rho_lo_pair_arm", qm(&pair_runs, |c| c.rho_lo))
        .number("rho_hi_pair_arm", qm(&pair_runs, |c| c.rho_hi))
        .number("rho_lo_one_phase", qm(&control_runs, |c| c.rho_lo))
        .number("rho_hi_one_phase", qm(&control_runs, |c| c.rho_hi))
        .int("seeds_separated", separated as i64)
        .int("seeds", SEEDS as i64)
        .number("droplet_lifetime_cluster_arm", life_c)
        .int("droplet_survived_cluster_arm", cap_c as i64)
        .int("droplet_absent_cluster_arm", absent_c as i64)
        .number("droplet_start_largest_cluster_arm", start_c)
        .number("droplet_lifetime_pair_arm", life_p)
        .int("droplet_survived_pair_arm", cap_p as i64)
        .int("droplet_absent_pair_arm", absent_p as i64)
        .number("droplet_start_largest_pair_arm", start_p)
        .int("droplet_cap", DROP_LIFE_CAP as i64)
        .raw("pair_arm", pair_json)
        .raw("droplets", drops_json)
        .raw("sigma_laplace", jarr(&sigma_l))
        .raw("sigma_laplace_r2", jarr(&r2_l))
        .number("sigma_laplace_mean", mean(&sigma_l))
        .number("sigma_laplace_sd", sd(&sigma_l))
        .raw("slabs", slab_json)
        .raw("sigma_capillary", jarr(&sigma_c))
        .raw("sigma_capillary_r2", jarr(&r2_c))
        .number("sigma_capillary_mean", mean(&sigma_c))
        .number("sigma_capillary_sd", sd(&sigma_c))
        .raw("width_slope_per_kstep", jarr(&width_slope))
        .number("width_slope_mean", mean(&width_slope))
        .number("width_slope_sd", sd(&width_slope))
        .raw("width_per_seed", jarr(&width_mean))
        .number("width_mean", mean(&width_mean))
        .raw("gravity", grav_json)
        .number("gravity_slope_on_mean", mean(&grav_slope_on))
        .number("gravity_slope_on_sd", sd(&grav_slope_on))
        .number("gravity_slope_off_mean", mean(&grav_slope_off))
        .number("gravity_slope_off_sd", sd(&grav_slope_off))
        .number("gravity_injected_norm", mean(&grav_injected))
        .raw("edge", edges)
        .number("seconds", secs)
        .gate(&check);
    w.write("run.json", &rec).expect("run.json writes");
    w.done("run.done", &format!("{steps} steps projected, {secs:.1} s of wall"))
        .expect("run.done writes");
}

// ---------------------------------------------------------------- read

/// The read's gates go through a report so each prints exactly once, the way the gate phase's
/// do, and so a branch that does not admit is named in `refusals()`.
fn read(dir: &str) {
    if !is_done(dir, "run.done") {
        println!("run.done is ABSENT: the read below is on a run that did not finish.");
    }
    let reading = read_record(format!("{dir}/run.json")).expect("run.json reads");
    let counted = reading.count().unwrap_or_else(|e| panic!("{e}"));
    let f = |k: &str| counted.field(k).unwrap_or_else(|e| panic!("{k}: {e}")).value;
    // A lifetime written as `null` is an ABSENCE by design and not a defect: it means no seed
    // of that arm lost half its cluster, so there is no death to average. It is read as `NaN`
    // and the branch below reads the SURVIVAL COUNT beside it, never the missing number. Used
    // for the two lifetime fields and nowhere else, so a genuinely absent field still panics.
    let f_life = |k: &str| counted.field(k).map(|v| v.value).unwrap_or(f64::NAN);

    let dip = f("dip_mean");
    let floor_max = f("floor_max");
    let sl = f("sigma_laplace_mean");
    let sl_sd = f("sigma_laplace_sd");
    let sc = f("sigma_capillary_mean");
    let sc_sd = f("sigma_capillary_sd");

    // ---- C: coexistence. The EMPTY branch is NAMED: no bimodality at the chart is a
    //      reading about the chart (M-EMPTY-SECTOR), never a zero fed onward.
    let excess = dip - floor_max;
    let ratio_cluster = f("variance_ratio_cluster_arm");
    let ratio_pair = f("variance_ratio_pair_arm");
    let mode_ratio = f("mode_ratio");
    let separated = f("seeds_separated");
    let seeds = f("seeds");
    let sp_c = f("spread_cluster_arm");
    let sp_c_sd = f("spread_cluster_arm_sd");
    let sp_o = f("spread_one_phase");
    let sp_o_sd = f("spread_one_phase_sd");
    // The noise the third leg is read against is the COMBINED measured spread of the two arms
    // over seeds, never one arm's alone: the one-phase control's own seed spread came back
    // EXACTLY ZERO on the confirmation pass (its quantiles land on the same discrete block
    // occupancies every seed), and dividing by it would have turned a real 25-sigma gap into
    // a number with no meaning. A band of exactly zero VOIDS the gate rather than passing it.
    let band_c = C_SPREAD_SIGMA * (sp_c_sd * sp_c_sd + sp_o_sd * sp_o_sd).sqrt();
    let l2 = separated >= seeds;
    let l3 = ratio_cluster > C_VARIANCE_FACTOR * ratio_pair;
    let l4 = band_c > 0.0 && sp_c - sp_o > band_c;
    let c_branch = if l2 && l3 && l4 { 'a' } else { 'b' };
    let mut gc = Gate::new("C")
        .work(3)
        .branch(c_branch)
        .detail(format!(
            "(a) the block-density histogram is bimodal beyond BOTH controls — the one-phase \
             control that says what a correlated single phase reads, and the PAIR ARM at the \
             same chart that says what rule 2's cohesion reads — so the carrier has two \
             phases and the two modes are their densities; (b) it is NOT: the EMPTY branch, a \
             reading about the chart and never a zero, and everything below it is VOID. \
             Variance ratio against the one-phase control: cluster arm {}, pair arm {}. \
             Modes {} and {}, ratio {}. THE DIP IS REPORTED AND NOT GATED, and the freeze says \
             why (§3): where the dense phase is a minority of blocks the valley between the \
             two modes is as full as the smaller of them and the dip reads zero while the two \
             phases are plainly there — the screen measured exactly that at this chart, on all \
             three arms alike",
            jf(ratio_cluster),
            jf(ratio_pair),
            jf(f("mode_lo")),
            jf(f("mode_hi")),
            jf(mode_ratio)
        ))
        .leg_at("every seed's histogram has two separated modes", l2, separated)
        .leg_at(
            "the variance ratio is over three times the pair arm's at the same chart",
            l3,
            ratio_cluster - C_VARIANCE_FACTOR * ratio_pair,
        )
        .leg_at(
            "the block-density spread clears the one-phase control's by three standard \
             deviations of the two arms' COMBINED measured spread over seeds",
            l4,
            sp_c - sp_o - band_c,
        )
        .leg_at("(reported, NOT gated) the dip over the control's maximum", true, excess)
        .leg_at("(reported, NOT gated) the histogram's two modes", true, mode_ratio)
        .leg_at("(reported, NOT gated) the pair arm's own spread", true, f("spread_pair_arm"));
    if !(band_c > 0.0) {
        gc = gc.void(format!(
            "BOTH ARMS' SEED SPREAD IS EXACTLY ZERO ({sp_c_sd:.3e} and {sp_o_sd:.3e}), so the \
             third leg has no noise to be read against and the branch is VOID rather than \
             passed: a control measuring exactly zero voids what it gates (M-EMPTY-SECTOR)"
        ));
    }
    let gc = gc;

    // ---- L: Laplace's law is a line, or it is not.
    let l_branch = if sl > 0.0 { 'a' } else { 'b' };
    let gl = Gate::new("L")
        .work(1)
        .branch(l_branch)
        .detail(format!(
            "(a) the pressure jump is linear in 1/R over {} radii with ONE slope, which is \
             sigma_L; (b) it is not a line, and no surface tension is read from it. \
             sigma_L = {} +/- {} over the seeds",
            DROP_RADII.len(),
            jf(sl),
            jf(sl_sd)
        ))
        .leg_at("the slope is positive (the drop is at higher pressure)", sl > 0.0, sl)
        .leg_at("the slope is resolved against its own spread over seeds", sl.abs() > 2.0 * sl_sd, sl_sd);

    // ---- A: the two sigmas agree within a band MEASURED from their own noise.
    let band = 2.0 * (sl_sd * sl_sd + sc_sd * sc_sd).sqrt();
    let gap = (sl - sc).abs();
    let a_branch = if gap <= band { 'a' } else { 'b' };
    let ga = Gate::new("A")
        .work(1)
        .branch(a_branch)
        .detail(format!(
            "(a) the two independent readings of sigma agree within two standard errors of \
             their COMBINED measured noise; (b) they disagree beyond it. The band is not \
             typed: band = 2*sqrt(sd_L^2 + sd_C^2) = {}. sigma_L = {}, sigma_C = {}. \
             sigma_C carries the kT_mech identification and sigma_L does not, so a \
             disagreement is a statement about the pair and not about either alone",
            jf(band),
            jf(sl),
            jf(sc)
        ))
        .leg_at("the two readings are within the measured band", gap <= band, gap);

    // ---- W: the interface width stationary, or growing without bound.
    let slope = f("width_slope_mean");
    let slope_sd = f("width_slope_sd");
    let w_branch = if slope.abs() < WIDTH_SLOPE_BAR { 'a' } else { 'b' };
    let gw = Gate::new("W")
        .work(1)
        .branch(w_branch)
        .detail(format!(
            "(a) the fitted interface width is stationary over the sampled frames; (b) it \
             grows, which is the freeze's kill. The bar is {} cells per thousand steps",
            jf(WIDTH_SLOPE_BAR)
        ))
        .leg_at("|dw/dt| under the bar", slope.abs() < WIDTH_SLOPE_BAR, slope)
        .leg_at(
            "and the drift is not resolved against its own spread over seeds",
            slope.abs() < 2.0 * slope_sd,
            slope_sd,
        );

    // ---- G: gravity settles the dense phase, or it does not.
    let on = f("gravity_slope_on_mean");
    let on_sd = f("gravity_slope_on_sd");
    let off = f("gravity_slope_off_mean");
    let off_sd = f("gravity_slope_off_sd");
    let injected = f("gravity_injected_norm");
    let moves = on.abs() > 2.0 * on_sd && on.abs() > 3.0 * off.abs().max(off_sd);
    let g_branch = if moves && injected > 0.0 { 'a' } else { 'b' };
    let gg = Gate::new("G1B")
        .work(1)
        .branch(g_branch)
        .detail(format!(
            "(a) with gravity on the dense phase's centre of mass drifts along gravity and \
             the control with gravity off does not; (b) it does not move, or the control \
             moves too. Slopes in links per step, projected on gravity: on {} +/- {}, off \
             {} +/- {}; posted injection {} over the run. Per seed in `gravity`",
            jf(on),
            jf(on_sd),
            jf(off),
            jf(off_sd),
            jf(injected)
        ))
        .leg_at("gravity posted momentum into the ledger", injected > 0.0, injected)
        .leg_at("the dense phase moved, against its own spread", on.abs() > 2.0 * on_sd, on)
        .leg_at("and the no-gravity control did not", on.abs() > 3.0 * off.abs().max(off_sd), off);

    // ---- D: the droplet's lifetime. This branch is NOT void under C (b): a bonded droplet
    //      either keeps half its cluster or it does not, and that is the number the campaign
    //      reads where coexistence is absent.
    let life_c = f_life("droplet_lifetime_cluster_arm");
    let cap_c = f("droplet_survived_cluster_arm");
    let life_p = f_life("droplet_lifetime_pair_arm");
    let cap_p = f("droplet_survived_pair_arm");
    let absent_c = f("droplet_absent_cluster_arm");
    let cap = f("droplet_cap");
    let d_branch = if absent_c >= seeds {
        'c'
    } else if cap_c >= seeds {
        'a'
    } else {
        'b'
    };
    let gd = Gate::new("D1")
        .work(1)
        .branch(d_branch)
        .detail(format!(
            "(a) every seed's droplet keeps half its starting cluster to the cap of {} steps; \
             (b) it loses half, and the lifetime is the number; (c) there was no droplet to \
             lose — EMPTY, named. Cluster arm: {} of {} survived, lifetime {}. Pair arm: {} of \
             {} survived, lifetime {}",
            jf(cap),
            cap_c,
            seeds,
            jf(life_c),
            cap_p,
            seeds,
            jf(life_p)
        ))
        .leg_at("the cluster arm's droplet bound at all", absent_c < seeds, absent_c)
        .leg_at(
            "and it outlives the pair arm's, or both reached the cap",
            cap_c >= cap_p && (cap_c >= seeds || !(life_c < life_p)),
            life_c,
        );

    let mut rr = Report::new();
    for g in [&gc, &gd, &gl, &ga, &gw, &gg] {
        rr.gate(g.clone());
    }
    for l in rr.refusals() {
        println!("REFUSED {l}");
    }
    let rec = Record::new("read")
        .gate(&gd)
        .number("droplet_lifetime_cluster_arm", life_c)
        .number("droplet_lifetime_pair_arm", life_p)
        .number("variance_ratio_cluster_arm", ratio_cluster)
        .number("variance_ratio_pair_arm", ratio_pair)
        .number("mode_ratio", mode_ratio)
        .gate(&gc)
        .gate(&gl)
        .gate(&ga)
        .gate(&gw)
        .gate(&gg)
        .number("dip_mean", dip)
        .number("dip_sd", f("dip_sd"))
        .number("floor_mean", f("floor_mean"))
        .number("floor_sd", f("floor_sd"))
        .number("floor_max", floor_max)
        .number("sigma_laplace", sl)
        .number("sigma_capillary", sc)
        .number("agreement_band", band)
        .number("agreement_gap", gap)
        .number("width_slope_per_kstep", slope);
    RecordWriter::new(dir).write("read.json", &rec).expect("read.json writes");
}
