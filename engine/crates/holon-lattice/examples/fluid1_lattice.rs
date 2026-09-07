//! FLUID-1's runner: the orientation lattice's gates, runs and read.
//!
//! `conformance/mesh/FLUID1_PREREG.md` is the freeze; the window, floor, size, amplitude and
//! step-cap rules are FLUID-0's under `FLUID0_AMENDMENT_1` set A and are inherited UNCHANGED.
//! The instrument is `holon_lattice::orientation`. Usage:
//!
//! ```text
//! fluid1_lattice gate [out_dir]           G0-G3, the two plants, price.json, gate.json
//! fluid1_lattice run  [out_dir] [threads] the chart runs, the controls, the plants, the
//!                                         block-chart witness rates -> run.json, run.done
//! fluid1_lattice read [out_dir]           B, S, W -> read.json
//! ```
//!
//! `out_dir` defaults to `../conformance/mesh/fluid1`, i.e. run from `engine/`.
//!
//! # No number from the molecular tier is typed here
//!
//! The amplitude table `A(φ)` and the chart's one retention are READ from CT-1's, CT-2's and
//! LIQUID-1's own records at run time, with the path and the field printed beside every value
//! (M-STALE-INSTRUMENT). FLUID-0's FHP-I lines and its measured price are read from
//! `conformance/mesh/fluid0/gate.json` and `price.json` the same way. The ONLY number typed
//! from outside is water's Schmidt number, which is the experimental KILL and is named as such
//! wherever it appears.

// Waived for the same reasons FLUID-0's runner waives them: the JSON is assembled line by
// line, and `t` is a time coordinate.
#![allow(clippy::write_with_newline, clippy::needless_range_loop, clippy::too_many_arguments)]

use holon_lattice::chart::BlockChart;
use holon_lattice::lattice::Lattice;
use holon_lattice::orientation::{
    colour_ensemble_oriented, shear_ensemble_oriented, witness_rate, BondGraph, OrientationAudit,
    OrientationLattice, OrientationRules, WitnessReading, N_DIRS,
};
use holon_lattice::state::Model;
use holon_lattice::transport::{
    self, law_digest, wavenumber, Exit, FitRule, TransportReading, K_AGREEMENT_TOLERANCE,
    MIN_POINTS, MIN_R2, MONOTONE_TOLERANCE_FRACTION,
};
use std::fmt::Write as _;
use std::time::Instant;

// ═══════════ THE DECLARED BLOCK — every number the instrument uses, and nowhere else ═══════

// ---- inherited from FLUID0_AMENDMENT_1 set A, unchanged (FLUID1_PREREG §0) ----
/// The hydrodynamic size. FLUID-0's, and M-VOLUME-SCALE's.
const L: usize = 256;
const D: f64 = 0.2;
/// Members averaged BEFORE the fit — the freeze's "two seeds".
const SEEDS: usize = 2;
const A_SHEAR: f64 = 0.10;
const A_COLOUR: f64 = 1.0;
const K1: usize = 1;
const K2: usize = 2;
/// FLUID-0's declared seed, kept so the no-bond control is an IDENTITY against FLUID-0's own
/// readings rather than an agreement between two different configurations.
const SEED: u64 = 0x464c_5549_4430;
const TAU_STEPS: f64 = 11.4;
const TRANSIENT_TAUS: f64 = 4.0;
const WINDOW_END_FRACTION_A: f64 = 0.2;
const NOISE_FLOOR_SNR: f64 = 12.0;
const SIGMA_TABLE: [(usize, f64, f64); 3] =
    [(64, 33.9905, 21.7525), (128, 72.3969, 45.4090), (256, 114.4808, 89.8527)];
const CAP_SHEAR: [usize; 2] = [12_000, 3_000];
const CAP_COLOUR: [usize; 2] = [1_500, 400];

// ---- FLUID-1's own ----
/// G3's scene length and tolerance (freeze §2 G3).
const G3_STEPS: usize = 100_000;
const G3_TOL: f64 = 0.02;
/// G0's tolerance on the reproduced retention (freeze §2 G0).
const G0_TOL: f64 = 0.05;
/// G1's tolerance against FLUID-0's lines (freeze §2 G1).
const G1_TOL: f64 = 0.05;
/// G2's ledger sweep: runs and steps, every integer checked at every step.
const G2_RUNS: usize = 16;
const G2_STEPS: usize = 500;
/// The bond-count reading: steps discarded, then steps averaged. The rent clause's own
/// relaxation is one step, so the warm-up is two orders clear of it.
const BOND_WARMUP: usize = 200;
const BOND_SAMPLE: usize = 100;
/// The plants' stakes (freeze §5) and their carriers' floor.
const PLANT_I_MIN_CHANGE: f64 = 0.20;
const PLANT_II_MIN_CHANGE: f64 = 0.20;
const PLANT_I_CARRIER_FLOOR: f64 = 0.20;
/// The price: measured on this many steps of every run, before its counted steps.
const PRICE_STEPS: usize = 200;
const PRICE_LOW: f64 = 0.1;
const PRICE_HIGH: f64 = 10.0;
/// Node LG's block sizes (freeze §0, §2 W).
const W_BLOCKS: [usize; 4] = [4, 8, 16, 32];
/// One step, because `W(b)` is the ONE-step defect law of `LG_PREREG` §5.3.
const W_STEPS: usize = 1;
const W_TOL: f64 = 0.10;
/// B's band: within this factor of the molecular box's count, which is READ, not typed.
const B_BAND_FACTOR: f64 = 1.5;
/// S's bands (freeze §2 S).
const SC_BRANCH_A: f64 = 100.0;
const SC_BRANCH_B: f64 = 10.0;
/// Water's ratio. THE ONE NUMBER TYPED FROM OUTSIDE THIS ENGINE, and it is the KILL:
/// `ν = 1.00e-2 cm²/s` (Kestin, Sokolov & Wakeham 1978), `D = 2.3e-5 cm²/s` (Krynicki, Green
/// & Sawyer 1978; Mills 1973), as FLUID-0 §1 cites them.
const SC_WATER: f64 = 434.8;
/// Workers for `run`'s block-chart pass, one whole `b` each. Cores 24-31 only: `taskset` is
/// the binding constraint, never a count typed here (M-DEVICE-CLASS).
const DEFAULT_THREADS: usize = 4;

// ---- the records READ at run time; the paths are data, the numbers are never typed ----
const R_CT1_LINEAR: &str = "water_observatory/ct1/sector_linear_R2.9.json";
const R_CT2_LINEAR_EXACT: &str = "water_observatory/ct2/gd0_linear_R2.9.json";
const R_CT2_T60: &str = "water_observatory/ct2/node_tilt_R2.9_t60.json";
const R_CT2_T120: &str = "water_observatory/ct2/node_tilt_R2.9_t120.json";
const R_CT2_T180: &str = "water_observatory/ct2/node_tilt_R2.9_t180.json";
const R_CT2_ARMS: &str = "water_observatory/ct2/arms.json";
const R_LIQUID1_ARM: &str = "water_observatory/liquid1/arm.json";
const R_FLUID0_GATE: &str = "mesh/fluid0/gate.json";
const R_FLUID0_PRICE: &str = "mesh/fluid0/price.json";
// ═════════════════════════ END OF THE DECLARED BLOCK ══════════════════════════════════════

fn transient_steps() -> usize {
    (TRANSIENT_TAUS * TAU_STEPS).ceil() as usize
}

fn sigma(l: usize, colour: bool) -> f64 {
    SIGMA_TABLE
        .iter()
        .find(|(ll, _, _)| *ll == l)
        .map(|(_, s, c)| if colour { *c } else { *s })
        .unwrap_or_else(|| panic!("no measured sigma for L={l}: measure it before using it"))
}

fn cap(colour: bool, k_index: usize) -> usize {
    let t = if colour { CAP_COLOUR } else { CAP_SHEAR };
    t[k_index - 1]
}

fn rule(l: usize, colour: bool, seeds: usize) -> FitRule {
    FitRule {
        start: transient_steps(),
        end_fraction: WINDOW_END_FRACTION_A,
        floor: NOISE_FLOOR_SNR * sigma(l, colour) / (seeds as f64).sqrt(),
        monotone_tolerance_fraction: MONOTONE_TOLERANCE_FRACTION,
        min_points: MIN_POINTS,
        min_r2: MIN_R2,
    }
}

// ---------------------------------------------------------------- JSON, by hand
fn jf(x: f64) -> String {
    if x.is_finite() {
        format!("{:.9e}", x)
    } else {
        "null".to_string()
    }
}
fn jo(x: Option<f64>) -> String {
    match x {
        Some(v) if v.is_finite() => format!("{:.9e}", v),
        _ => "null".to_string(),
    }
}
fn jb(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}

fn rel(a: f64, b: f64) -> f64 {
    let d = a.abs().max(b.abs());
    if d > 0.0 {
        (a - b).abs() / d
    } else {
        f64::INFINITY
    }
}

fn within(a: Option<f64>, b: Option<f64>, tol: f64) -> bool {
    matches!((a, b), (Some(x), Some(y)) if rel(x, y) <= tol)
}

fn reading_json(r: &TransportReading) -> String {
    format!(
        "{{\"value\": {}, \"gamma\": {}, \"k\": {}, \"window\": [{}, {}], \"points\": {}, \
         \"r2\": {}, \"collisions_fired\": {}, \"exit\": \"{}\", \"start_amplitude\": {}, \
         \"end_amplitude\": {}}}",
        jo(r.value),
        jf(r.gamma),
        jf(r.k),
        r.window.0,
        r.window.1,
        r.points,
        jf(r.r2),
        r.collisions_fired,
        r.exit.name(),
        jf(r.start_amplitude),
        jf(r.end_amplitude)
    )
}

fn audit_json(a: &OrientationAudit) -> String {
    let t = a.totals;
    format!(
        "{{\"all_exact\": {}, \"mass_exact\": {}, \"px_exact\": {}, \"py_exact\": {}, \
         \"red_exact\": {}, \"orientation_total_exact\": {}, \"orientation_census_exact\": {}, \
         \"bond_balance_exact\": {}, \"steps_checked\": {}, \"mass\": {}, \"px\": {}, \
         \"py\": {}, \"red\": {}, \"orientation_total\": {}, \"bonds_final\": {}, \
         \"collisions_fired\": {}, \"formed\": {}, \"broken_rent\": {}, \"blocked\": {}, \
         \"blocked_no_vacancy\": {}, \"blocked_claimed\": {}, \"blocked_no_geometry\": {}, \
         \"joint_moves\": {}, \"anomalous_hops\": {}, \"break_tests\": [{}], \
         \"broken_by_phi\": [{}]}}",
        jb(a.all_exact()),
        jb(a.mass_exact),
        jb(a.px_exact),
        jb(a.py_exact),
        jb(a.red_exact),
        jb(a.orientation_total_exact),
        jb(a.orientation_census_exact),
        jb(a.bond_balance_exact),
        a.steps_checked,
        a.mass,
        a.px,
        a.py,
        a.red,
        a.orientation_total,
        a.bonds_final,
        t.collisions_fired,
        t.formed,
        t.broken_rent,
        t.blocked,
        t.blocked_no_vacancy,
        t.blocked_claimed,
        t.blocked_no_geometry,
        t.joint_moves,
        t.anomalous_hops,
        t.break_tests.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "),
        t.broken_by_phi.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")
    )
}

fn bond_graph_json(b: &BondGraph) -> String {
    format!(
        "{{\"particles\": {}, \"bonds\": {}, \"largest\": {}, \"largest_fraction\": {}, \
         \"largest_winds\": [{}, {}], \"largest_spans\": {}, \"any_spans\": {}, \
         \"largest_touches_all_rows\": {}, \"largest_touches_all_columns\": {}, \
         \"bonds_per_particle_degree\": {}, \"bonds_per_particle_donor\": {}}}",
        b.particles,
        b.bonds,
        b.largest,
        jf(b.largest_fraction),
        jb(b.largest_winds[0]),
        jb(b.largest_winds[1]),
        jb(b.largest_spans),
        jb(b.any_spans),
        jb(b.largest_touches_all_rows),
        jb(b.largest_touches_all_columns),
        jf(b.bonds_per_particle_degree),
        jf(b.bonds_per_particle_donor)
    )
}

fn show(tag: &str, r: &TransportReading) -> String {
    format!(
        "{tag:<34} exit {:<15} value {:<16} gamma {:<14} R2 {:<10} pts {:<5} window {}..{} \
         amp {:.4} -> {:.4} fired {}",
        r.exit.name(),
        match r.value {
            Some(v) => format!("{:.6e}", v),
            None => "-".to_string(),
        },
        if r.gamma.is_finite() { format!("{:.6e}", r.gamma) } else { "-".to_string() },
        if r.r2.is_finite() { format!("{:.6}", r.r2) } else { "-".to_string() },
        r.points,
        r.window.0,
        r.window.1,
        r.start_amplitude,
        r.end_amplitude,
        r.collisions_fired
    )
}

// ---------------------------------------------------------------- the report
struct Report {
    failures: Vec<String>,
    entries: Vec<String>,
}

impl Report {
    /// A gate names EVERY failing leg (M-FIRST-VIOLATION-ONLY): `legs` is the whole list of
    /// (name, passed) and the detail carries the failing names in full.
    fn gate(&mut self, id: &str, legs: &[(String, bool)], work: u64, detail: String) {
        let failing: Vec<&str> =
            legs.iter().filter(|(_, ok)| !ok).map(|(n, _)| n.as_str()).collect();
        let pass = failing.is_empty();
        if work == 0 {
            println!("  {id:<7} VOID (zero work)  {detail}");
            self.failures.push(format!("{id}: ZERO WORK — {detail}"));
            self.entries.push(format!(
                "\"{id}\": {{\"verdict\": \"VOID\", \"work\": 0, \"legs\": [{}], \
                 \"failing_legs\": [{}], \"detail\": {:?}}}",
                legs.iter()
                    .map(|(n, o)| format!("{{\"leg\": {:?}, \"pass\": {}}}", n, jb(*o)))
                    .collect::<Vec<_>>()
                    .join(", "),
                failing.iter().map(|f| format!("{f:?}")).collect::<Vec<_>>().join(", "),
                detail
            ));
            return;
        }
        let v = if pass { "PASS" } else { "FAIL" };
        println!("  {id:<7} {v}  [{work} checks, {} legs]  {detail}", legs.len());
        if !pass {
            println!("          FAILING LEGS ({}): {}", failing.len(), failing.join(" | "));
            self.failures.push(format!("{id}: {} — {detail}", failing.join(" | ")));
        }
        self.entries.push(format!(
            "\"{id}\": {{\"verdict\": \"{v}\", \"work\": {work}, \"legs\": [{}], \
             \"failing_legs\": [{}], \"detail\": {:?}}}",
            legs.iter()
                .map(|(n, o)| format!("{{\"leg\": {:?}, \"pass\": {}}}", n, jb(*o)))
                .collect::<Vec<_>>()
                .join(", "),
            failing.iter().map(|f| format!("{f:?}")).collect::<Vec<_>>().join(", "),
            detail
        ));
    }
}

// ---------------------------------------------------------------- reading the records
/// The conformance root, PROBED rather than assumed, and printed once. A record that cannot be
/// found is a refusal (the instrument stops), never a default.
fn records_root() -> String {
    for c in ["../conformance", "conformance", "../../../conformance", "../../conformance"] {
        if std::path::Path::new(&format!("{c}/{R_CT2_ARMS}")).exists() {
            return c.to_string();
        }
    }
    panic!("no conformance root found: looked for {R_CT2_ARMS} under ../conformance, conformance, ../../../conformance, ../../conformance");
}

/// The first number written under `key`, at or after every literal in `path_keys` has been
/// found in order. Pretty-printed and nested records are read the same way as flat ones, and
/// an absent key panics with the file it was absent from.
fn read_field(root: &str, rel_path: &str, path_keys: &[&str], key: &str) -> f64 {
    let full = format!("{root}/{rel_path}");
    let text = std::fs::read_to_string(&full)
        .unwrap_or_else(|e| panic!("cannot read the record {full}: {e}"));
    let mut at = 0usize;
    for needle in path_keys {
        at = at
            + text[at..]
                .find(needle)
                .unwrap_or_else(|| panic!("{full}: no {needle:?} at or after byte {at}"))
            + needle.len();
    }
    let pat = format!("\"{key}\":");
    let i = at
        + text[at..].find(&pat).unwrap_or_else(|| panic!("{full}: no field {key:?}"))
        + pat.len();
    let rest = &text[i..];
    let end = rest.find([',', '}', '\n']).unwrap_or(rest.len());
    rest[..end]
        .trim()
        .trim_matches('"')
        .parse::<f64>()
        .unwrap_or_else(|e| panic!("{full}: field {key:?} is not a number: {e}"))
}

/// One angle of the amplitude table, with the provenance it was read through.
#[derive(Clone, Debug)]
struct Amp {
    degrees: usize,
    e_ct: f64,
    a: f64,
    source: String,
}

/// `A(φ)` at the six lattice angles, READ from CT-1's and CT-2's records at run time.
///
/// `E_CT` is the tilt records' own quantity, `E_exact(total) − E_noCT(total)` (their field
/// `e_ct_rule`). The `0°` node has no tilt record of its own — the tilt family at 2.9 Å runs
/// from 60° — so its `E_CT` is assembled from the two records that hold its halves BY THE SAME
/// RULE: the exact total from `gd0_linear_R2.9.json` and the no-CT total from CT-1's
/// `sector_linear_R2.9.json`. Both paths and both fields are printed. `240°` and `300°` are
/// `120°` and `60°`: the tilt is a rotation of the acceptor about the link, so `+φ` and `−φ`
/// are the same geometry, and the lattice's `240°` is the map's `120°`.
fn amplitude_table(root: &str) -> [Amp; N_DIRS] {
    let e_exact_0 = read_field(root, R_CT2_LINEAR_EXACT, &[], "e_full");
    let e_noct_0 = read_field(root, R_CT1_LINEAR, &[], "e_noct");
    let ct0 = e_exact_0 - e_noct_0;
    let ct60 = read_field(root, R_CT2_T60, &[], "e_ct");
    let ct120 = read_field(root, R_CT2_T120, &[], "e_ct");
    let ct180 = read_field(root, R_CT2_T180, &[], "e_ct");
    let src0 = format!(
        "{R_CT2_LINEAR_EXACT}:e_full ({e_exact_0}) - {R_CT1_LINEAR}:e_noct ({e_noct_0}), \
         the tilt records' own e_ct_rule"
    );
    let mk = |deg: usize, e: f64, src: String| Amp { degrees: deg, e_ct: e, a: e / ct0, source: src };
    [
        mk(0, ct0, src0),
        mk(60, ct60, format!("{R_CT2_T60}:e_ct")),
        mk(120, ct120, format!("{R_CT2_T120}:e_ct")),
        mk(180, ct180, format!("{R_CT2_T180}:e_ct")),
        mk(240, ct120, format!("{R_CT2_T120}:e_ct (tilt symmetry: 240 deg is the map's 120 deg)")),
        mk(300, ct60, format!("{R_CT2_T60}:e_ct (tilt symmetry: 300 deg is the map's 60 deg)")),
    ]
}

fn amplitudes(t: &[Amp; N_DIRS]) -> [f64; N_DIRS] {
    let mut a = [0.0f64; N_DIRS];
    for k in 0..N_DIRS {
        a[k] = t[k].a;
    }
    a
}

fn amplitude_table_json(t: &[Amp; N_DIRS]) -> String {
    t.iter()
        .map(|x| {
            format!(
                "{{\"phi_degrees\": {}, \"phi_index\": {}, \"e_ct_hartree\": {}, \
                 \"amplitude\": {}, \"source\": {:?}}}",
                x.degrees,
                x.degrees / 60,
                jf(x.e_ct),
                jf(x.a),
                x.source
            )
        })
        .collect::<Vec<_>>()
        .join(",\n    ")
}

/// The chart's ONE number: the 293 K dimer arm's measured retention.
fn read_retention(root: &str) -> f64 {
    read_field(root, R_CT2_ARMS, &["\"dimer_293_seam\""], "f")
}

/// The molecular box's bond count, in the convention its own record quotes it in (each bond
/// once, at its donor).
fn read_box_bonds_per_molecule(root: &str) -> f64 {
    read_field(root, R_LIQUID1_ARM, &["\"r2\""], "hbonds_per_molecule")
}

/// FLUID-0's own FHP-I line for one chirality: `(ν(k1), ν(k2), D(k1), D(k2), Sc)`.
fn read_fluid0_line(root: &str, chirality: bool) -> (f64, f64, f64, f64, f64) {
    let head = format!("\"chirality\": {}", jb(chirality));
    let f = |k: &str| read_field(root, R_FLUID0_GATE, &["\"fhp_i\"", &head, k], "value");
    (
        f("\"nu_k1\""),
        f("\"nu_k2\""),
        f("\"d_k1\""),
        f("\"d_k2\""),
        read_field(root, R_FLUID0_GATE, &["\"fhp_i\"", &head], "sc"),
    )
}

fn read_fluid0_price(root: &str) -> f64 {
    read_field(root, R_FLUID0_PRICE, &[], "price_seconds_per_law")
}

// ---------------------------------------------------------------- the rules, from the chart
fn chart_rules(amp: [f64; N_DIRS], f: f64) -> OrientationRules {
    OrientationRules {
        bonds_enabled: true,
        amplitude: amp,
        rent: OrientationRules::rent_from_retention(f),
        cold: false,
        stream: true,
    }
}

/// One law's four readings under the orientation lattice, with the audits and the step counts.
struct LawRuns {
    nu1: TransportReading,
    nu2: TransportReading,
    d1: TransportReading,
    d2: TransportReading,
    audits: [OrientationAudit; 4],
    seconds: [f64; 4],
    price_seconds_per_member_step: f64,
}

impl LawRuns {
    fn sc(&self) -> Option<f64> {
        match (self.nu1.value, self.d1.value) {
            (Some(n), Some(d)) if d != 0.0 => Some(n / d),
            _ => None,
        }
    }
    fn all_fitted(&self) -> bool {
        [&self.nu1, &self.nu2, &self.d1, &self.d2].iter().all(|x| x.exit == Exit::Fitted)
    }
    fn member_steps(&self) -> u64 {
        self.audits.iter().map(|a| a.steps_checked).sum()
    }
    fn seconds_total(&self) -> f64 {
        self.seconds.iter().sum()
    }
    /// M-CHEAPER-THAN-ITS-PRICE: the run's own measured cost against the price taken on its
    /// first `PRICE_STEPS` steps, in the band `[PRICE_LOW, PRICE_HIGH]`.
    fn price_ratio(&self) -> f64 {
        let expected = self.price_seconds_per_member_step * self.member_steps() as f64;
        if expected > 0.0 {
            self.seconds_total() / expected
        } else {
            f64::INFINITY
        }
    }
}

fn shear_run(
    m: &Model,
    law: &[u8],
    k_index: usize,
    a: f64,
    along_columns: bool,
    drive: [f64; 2],
    rules: &OrientationRules,
) -> (TransportReading, OrientationAudit) {
    shear_ensemble_oriented(
        m,
        law,
        L,
        D,
        a,
        k_index,
        cap(false, k_index),
        SEED,
        SEEDS,
        along_columns,
        drive,
        &rule(L, false, SEEDS),
        rules,
    )
}

fn tracer_run(
    m: &Model,
    law: &[u8],
    k_index: usize,
    rules: &OrientationRules,
) -> (TransportReading, OrientationAudit) {
    colour_ensemble_oriented(
        m,
        law,
        L,
        D,
        A_COLOUR,
        k_index,
        cap(true, k_index),
        SEED,
        SEEDS,
        &rule(L, true, SEEDS),
        rules,
    )
}

/// The price of one member-step under the given rules, measured on `PRICE_STEPS` steps of a
/// `SEEDS`-member ensemble built the way the runs are built. Taken BEFORE the counted steps.
fn measure_price(m: &Model, law: &[u8], rules: &OrientationRules) -> f64 {
    let mut members: Vec<OrientationLattice> = (0..SEEDS)
        .map(|s| {
            let sd = transport::ensemble_seed(SEED, s);
            let lat = Lattice::seeded(m.clone(), L, sd, D, law.to_vec());
            let colour = lat.seed_colour_wave(sd, A_COLOUR, K1);
            OrientationLattice::from_lattice(lat, sd, *rules).with_colour(colour)
        })
        .collect();
    let t = Instant::now();
    for _ in 0..PRICE_STEPS {
        for g in members.iter_mut() {
            g.step();
        }
    }
    t.elapsed().as_secs_f64() / (PRICE_STEPS * SEEDS) as f64
}

/// One law's four readings, priced before they are taken.
fn run_law(m: &Model, law: &[u8], rules: &OrientationRules) -> LawRuns {
    let price = measure_price(m, law, rules);
    let mut seconds = [0.0f64; 4];
    let t = Instant::now();
    let (nu1, a1) = shear_run(m, law, K1, A_SHEAR, false, transport::row_drive(), rules);
    seconds[0] = t.elapsed().as_secs_f64();
    let t = Instant::now();
    let (nu2, a2) = shear_run(m, law, K2, A_SHEAR, false, transport::row_drive(), rules);
    seconds[1] = t.elapsed().as_secs_f64();
    let t = Instant::now();
    let (d1, a3) = tracer_run(m, law, K1, rules);
    seconds[2] = t.elapsed().as_secs_f64();
    let t = Instant::now();
    let (d2, a4) = tracer_run(m, law, K2, rules);
    seconds[3] = t.elapsed().as_secs_f64();
    LawRuns {
        nu1,
        nu2,
        d1,
        d2,
        audits: [a1, a2, a3, a4],
        seconds,
        price_seconds_per_member_step: price,
    }
}

fn law_runs_json(tag: &str, chirality: bool, law: &[u8], r: &LawRuns) -> String {
    format!(
        "{{\"run\": {:?}, \"chirality\": {}, \"digest\": \"{:016x}\", \"nu_k1\": {}, \
         \"nu_k2\": {}, \"d_k1\": {}, \"d_k2\": {}, \"sc\": {}, \"all_fitted\": {}, \
         \"seconds\": [{}], \"seconds_total\": {}, \"member_steps\": {}, \
         \"price_seconds_per_member_step\": {}, \"price_ratio\": {}, \"audits\": [{}]}}",
        tag,
        jb(chirality),
        law_digest(law),
        reading_json(&r.nu1),
        reading_json(&r.nu2),
        reading_json(&r.d1),
        reading_json(&r.d2),
        jo(r.sc()),
        jb(r.all_fitted()),
        r.seconds.iter().map(|s| jf(*s)).collect::<Vec<_>>().join(", "),
        jf(r.seconds_total()),
        r.member_steps(),
        jf(r.price_seconds_per_member_step),
        jf(r.price_ratio()),
        r.audits.iter().map(audit_json).collect::<Vec<_>>().join(", ")
    )
}

// ---------------------------------------------------------------- the held-geometry scene
/// G3's scene and G0's retention leg: ONE link at a fixed geometry, streaming off.
///
/// Returns `(held fraction of THE tracked link, 1/(1 + p_break), break tests at φ, live links
/// in the scene)`. At `φ = 0°` the acceptor's arm points back along the link by the definition
/// of the angle, so the reverse link forms too and the scene carries TWO bonds; the reading is
/// the tracked donor's role, counted once per step, never the bond count.
fn held_fraction_scene(
    phi: usize,
    steps: usize,
    rules: OrientationRules,
) -> (f64, f64, u64, usize) {
    let m = Model::fhp6();
    let law = m.fhp_i(true);
    let l = 8usize;
    let lat = Lattice::seeded(m, l, 0x0, 0.0, law);
    assert!(lat.cells.iter().all(|&s| s == 0), "the held-geometry scene is not empty");
    let mut g =
        OrientationLattice::from_lattice(lat, 0x0, OrientationRules { stream: false, ..rules });
    let delta = 0usize;
    let i = 2 * l + 2;
    let j = g.tables().neighbour_of(i, delta);
    g.cells[i] = 1 << 1;
    g.orient[i * N_DIRS + 1] = delta as u8;
    g.cells[j] = 1 << 4;
    let opp = (delta + 3) % N_DIRS;
    g.orient[j * N_DIRS + 4] = ((opp + phi) % N_DIRS) as u8;
    g.reset_initial();
    let tracked = i * N_DIRS + 1;
    let mut held = 0u64;
    let mut links = 0usize;
    for _ in 0..steps {
        g.step();
        if g.donor_of[tracked] != holon_lattice::orientation::NO_BOND {
            held += 1;
        }
        links = links.max(g.bonds.len());
    }
    (
        held as f64 / steps as f64,
        OrientationRules::retention(g.rules.p_break(phi)),
        g.audit.totals.break_tests[phi],
        links,
    )
}

// ---------------------------------------------------------------- the bond census
/// The bond graph after `BOND_WARMUP` steps, averaged over `BOND_SAMPLE` steps and over the
/// `SEEDS` members: the freeze's bond count and spanning reading, on the chart's own scene.
struct BondCensus {
    degree: f64,
    donor: f64,
    largest_fraction: f64,
    spanning_steps: u64,
    samples: u64,
    formed: u64,
    broken_rent: u64,
    blocked: u64,
    blocked_no_vacancy: u64,
    blocked_claimed: u64,
    break_tests: [u64; N_DIRS],
    broken_by_phi: [u64; N_DIRS],
    last: BondGraph,
    audit_exact: bool,
    seconds: f64,
    price_seconds_per_member_step: f64,
}

impl BondCensus {
    /// The measured `p_break` at the linear bond, from the run's OWN counts — the rent clause
    /// firing inside the full lattice rather than in the two-particle scene.
    fn measured_p_break(&self, phi: usize) -> Option<f64> {
        if self.break_tests[phi] == 0 {
            None
        } else {
            Some(self.broken_by_phi[phi] as f64 / self.break_tests[phi] as f64)
        }
    }
    fn spanning_fraction(&self) -> f64 {
        if self.samples == 0 {
            f64::NAN
        } else {
            self.spanning_steps as f64 / self.samples as f64
        }
    }
}

fn bond_census(m: &Model, law: &[u8], rules: &OrientationRules) -> BondCensus {
    let price = measure_price(m, law, rules);
    let t0 = Instant::now();
    let mut members: Vec<OrientationLattice> = (0..SEEDS)
        .map(|s| {
            let sd = transport::ensemble_seed(SEED, s);
            let lat = Lattice::seeded(m.clone(), L, sd, D, law.to_vec());
            OrientationLattice::from_lattice(lat, sd, *rules)
        })
        .collect();
    for _ in 0..BOND_WARMUP {
        for g in members.iter_mut() {
            g.step();
        }
    }
    for g in members.iter_mut() {
        g.reset_initial();
    }
    let (mut deg, mut don, mut lf) = (0.0f64, 0.0f64, 0.0f64);
    let (mut spanning, mut samples) = (0u64, 0u64);
    let mut last = members[0].bond_graph();
    for _ in 0..BOND_SAMPLE {
        for g in members.iter_mut() {
            g.step();
            let bg = g.bond_graph();
            deg += bg.bonds_per_particle_degree;
            don += bg.bonds_per_particle_donor;
            lf += bg.largest_fraction;
            if bg.largest_spans {
                spanning += 1;
            }
            samples += 1;
            last = bg;
        }
    }
    let mut audit = members[0].audit;
    for g in members.iter().skip(1) {
        audit.merge_public(&g.audit);
    }
    let t = audit.totals;
    BondCensus {
        degree: deg / samples as f64,
        donor: don / samples as f64,
        largest_fraction: lf / samples as f64,
        spanning_steps: spanning,
        samples,
        formed: t.formed,
        broken_rent: t.broken_rent,
        blocked: t.blocked,
        blocked_no_vacancy: t.blocked_no_vacancy,
        blocked_claimed: t.blocked_claimed,
        break_tests: t.break_tests,
        broken_by_phi: t.broken_by_phi,
        last,
        audit_exact: audit.all_exact(),
        seconds: t0.elapsed().as_secs_f64(),
        price_seconds_per_member_step: price,
    }
}

fn bond_census_json(tag: &str, c: &BondCensus) -> String {
    format!(
        "{{\"run\": {:?}, \"warmup_steps\": {BOND_WARMUP}, \"sample_steps\": {BOND_SAMPLE}, \
         \"members\": {SEEDS}, \"samples\": {}, \"bonds_per_particle_degree\": {}, \
         \"bonds_per_particle_donor\": {}, \"largest_fraction\": {}, \"spanning_steps\": {}, \
         \"spanning_fraction\": {}, \"formed\": {}, \"broken_rent\": {}, \"blocked\": {}, \
         \"blocked_no_vacancy\": {}, \"blocked_claimed\": {}, \"break_tests\": [{}], \
         \"broken_by_phi\": [{}], \"measured_p_break_phi0\": {}, \"ledger_exact\": {}, \
         \"seconds\": {}, \"price_seconds_per_member_step\": {}, \"final_graph\": {}}}",
        tag,
        c.samples,
        jf(c.degree),
        jf(c.donor),
        jf(c.largest_fraction),
        c.spanning_steps,
        jf(c.spanning_fraction()),
        c.formed,
        c.broken_rent,
        c.blocked,
        c.blocked_no_vacancy,
        c.blocked_claimed,
        c.break_tests.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "),
        c.broken_by_phi.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "),
        jo(c.measured_p_break(0)),
        jb(c.audit_exact),
        jf(c.seconds),
        jf(c.price_seconds_per_member_step),
        bond_graph_json(&c.last)
    )
}

// ---------------------------------------------------------------- the params block
fn params_json(root: &str, table: &[Amp; N_DIRS], f: f64, rent: f64) -> String {
    format!(
        "{{\"freeze\": \"conformance/mesh/FLUID1_PREREG.md\", \
         \"inherits\": \"FLUID0_AMENDMENT_1 set A (window, floor, size, amplitudes, caps, seed)\", \
         \"records_root\": {:?}, \"l\": {L}, \"density\": {}, \"seeds\": {SEEDS}, \
         \"a_shear\": {}, \"a_colour\": {}, \"k_index\": [{K1}, {K2}], \"k1\": {}, \"k2\": {}, \
         \"seed\": {SEED}, \"tau_steps\": {}, \"transient_taus\": {}, \
         \"window_start_steps\": {}, \"window_end_fraction\": {}, \"noise_floor_snr\": {}, \
         \"floor_shear\": {}, \"floor_colour\": {}, \"cap_shear\": [{}, {}], \
         \"cap_colour\": [{}, {}], \"min_r2\": {}, \"min_points\": {MIN_POINTS}, \
         \"monotone_tolerance_fraction\": {}, \"k_agreement_tolerance\": {}, \
         \"retention_read\": {}, \"retention_source\": \"{R_CT2_ARMS}:dimer_293_seam.f\", \
         \"rent_e0_over_kt\": {}, \"rent_rule\": \"E0/kT = ln(f/(1-f)), the value at which a \
         linear bond's stationary held fraction 1/(1+p_break) is the READ retention\", \
         \"p_break_phi0\": {}, \"amplitude_table\": [\n    {}\n  ], \
         \"g3_steps\": {G3_STEPS}, \"g3_tolerance\": {}, \"g0_tolerance\": {}, \
         \"g1_tolerance\": {}, \"g2_runs\": {G2_RUNS}, \"g2_steps\": {G2_STEPS}, \
         \"bond_warmup\": {BOND_WARMUP}, \"bond_sample\": {BOND_SAMPLE}, \
         \"plant_i_min_change\": {}, \"plant_ii_min_change\": {}, \"price_steps\": {PRICE_STEPS}, \
         \"price_band\": [{}, {}], \"w_blocks\": [{}], \"w_steps\": {W_STEPS}, \
         \"w_tolerance\": {}, \"b_band_factor\": {}, \"sc_branch_a\": {}, \"sc_branch_b\": {}, \
         \"sc_water\": {}, \"sc_water_note\": \"THE KILL, typed from experiment: Kestin/Sokolov/\
Wakeham 1978 and Krynicki/Green/Sawyer 1978, Mills 1973, as FLUID-0 §1 cites them\"}}",
        root,
        jf(D),
        jf(A_SHEAR),
        jf(A_COLOUR),
        jf(wavenumber(L, K1)),
        jf(wavenumber(L, K2)),
        jf(TAU_STEPS),
        jf(TRANSIENT_TAUS),
        transient_steps(),
        jf(WINDOW_END_FRACTION_A),
        jf(NOISE_FLOOR_SNR),
        jf(rule(L, false, SEEDS).floor),
        jf(rule(L, true, SEEDS).floor),
        CAP_SHEAR[0],
        CAP_SHEAR[1],
        CAP_COLOUR[0],
        CAP_COLOUR[1],
        jf(MIN_R2),
        jf(MONOTONE_TOLERANCE_FRACTION),
        jf(K_AGREEMENT_TOLERANCE),
        jf(f),
        jf(rent),
        jf((-rent).exp()),
        amplitude_table_json(table),
        jf(G3_TOL),
        jf(G0_TOL),
        jf(G1_TOL),
        jf(PLANT_I_MIN_CHANGE),
        jf(PLANT_II_MIN_CHANGE),
        jf(PRICE_LOW),
        jf(PRICE_HIGH),
        W_BLOCKS.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(", "),
        jf(W_TOL),
        jf(B_BAND_FACTOR),
        jf(SC_BRANCH_A),
        jf(SC_BRANCH_B),
        jf(SC_WATER)
    )
}

// ---------------------------------------------------------------- main
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let phase = args.get(1).map(String::as_str).unwrap_or("gate");
    let out_dir = args.get(2).cloned().unwrap_or_else(|| "../conformance/mesh/fluid1".to_string());
    let threads: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_THREADS);
    std::fs::create_dir_all(&out_dir).expect("cannot create out_dir");
    let root = records_root();
    let table = amplitude_table(&root);
    let f = read_retention(&root);
    let rent = OrientationRules::rent_from_retention(f);

    println!("=========================================================================");
    println!("FLUID-1 — the orientation lattice: a closure in the carrier");
    println!("freeze:  conformance/mesh/FLUID1_PREREG.md (window/floor/size: FLUID0_AMENDMENT_1 set A)");
    println!("instr:   holon-lattice::orientation   phase: {phase}   out_dir: {out_dir}");
    println!("records: {root}");
    println!(
        "set A:   L={L} d={D} seeds={SEEDS} A_shear={A_SHEAR} A_colour={A_COLOUR} \
         k_index in {{{K1},{K2}}} seed={SEED:#x}"
    );
    println!(
        "window:  opens at {TRANSIENT_TAUS} tau = {} steps; closes at the first step under \
         max({WINDOW_END_FRACTION_A}|M(t0)|, {NOISE_FLOOR_SNR} sigma/sqrt(S))",
        transient_steps()
    );
    println!(
        "         floors shear {:.4} / colour {:.4}; caps shear {:?} colour {:?}",
        rule(L, false, SEEDS).floor,
        rule(L, true, SEEDS).floor,
        CAP_SHEAR,
        CAP_COLOUR
    );
    println!("\nTHE BOND RULE, READ FROM THE MAP (no number below is typed):");
    for a in &table {
        println!(
            "  A({:>3} deg) = {:.9}   E_CT = {:.12} Ha   <- {}",
            a.degrees, a.a, a.e_ct, a.source
        );
    }
    println!(
        "  retention f = {f} <- {R_CT2_ARMS}:dimer_293_seam.f   =>  E0/kT_lat = ln(f/(1-f)) = \
         {rent:.9},  p_break(0 deg) = {:.9}",
        (-rent).exp()
    );
    println!("=========================================================================");

    match phase {
        "gate" => gate(&out_dir, &root, &table, f, rent),
        "run" => run(&out_dir, &root, &table, f, rent, threads),
        "read" => read(&out_dir, &root),
        other => {
            eprintln!("unknown phase {other:?}; expected gate | run | read");
            std::process::exit(2);
        }
    }
}

// ---------------------------------------------------------------- gate
fn gate(out_dir: &str, root: &str, table: &[Amp; N_DIRS], f: f64, rent: f64) {
    let m = Model::fhp6();
    let amp = amplitudes(table);
    let rules = chart_rules(amp, f);
    let mut r = Report { failures: Vec::new(), entries: Vec::new() };
    let mut json = String::new();
    let t_gate = Instant::now();

    // ------------------------------------------------------------------ G3 first: it is
    // cheap, it is the rent clause's own detailed balance, and G0's retention leg reads it.
    println!("\n--- G3: detailed balance of the bond on a held geometry ---");
    let mut g3_legs: Vec<(String, bool)> = Vec::new();
    let mut g3_detail = String::new();
    let mut g3_rows = Vec::new();
    let mut held_phi0 = f64::NAN;
    for (phi, deg) in [(0usize, 0usize), (2usize, 120usize)] {
        let (held, expected, tests, links) = held_fraction_scene(phi, G3_STEPS, rules);
        if phi == 0 {
            held_phi0 = held;
        }
        let d = rel(held, expected);
        let work_ok = tests > 1_000;
        g3_legs.push((format!("phi={deg} deg: the scene exposed a bond to the rule"), work_ok));
        g3_legs.push((format!("phi={deg} deg: held fraction within {G3_TOL} of 1/(1+p_break)"), d <= G3_TOL));
        println!(
            "  phi={deg:>3} deg  held {held:.6}  vs 1/(1+p_break) {expected:.6}  rel {d:.6}  \
             break tests {tests}  live links in the scene {links}"
        );
        let _ = write!(
            g3_detail,
            "phi={deg}: held {held:.6} vs 1/(1+p_break) {expected:.6} rel {d:.6} \
             (tests {tests}, links {links}); "
        );
        g3_rows.push(format!(
            "{{\"phi_degrees\": {deg}, \"phi_index\": {phi}, \"steps\": {G3_STEPS}, \
             \"held_fraction\": {}, \"expected\": {}, \"relative\": {}, \"break_tests\": {tests}, \
             \"live_links\": {links}, \"note\": \"the reading is the TRACKED link's donor role, \
             once per step; at phi=0 the acceptor's arm points back along the link by the \
             definition of the angle, so the reverse link forms too and the scene carries two \
             bonds — reported, never subtracted\"}}",
            jf(held),
            jf(expected),
            jf(d)
        ));
    }
    let _ = write!(g3_detail, "streaming off, {G3_STEPS} steps each");
    r.gate("G3", &g3_legs, 2 * G3_STEPS as u64, g3_detail);

    // ------------------------------------------------------------------ G2 + the chart's
    // own bond census: 16 bonded runs, every integer checked at every step.
    println!("\n--- G2: the ledger, exact at every step ({G2_RUNS} runs x {G2_STEPS} steps, L={L}) ---");
    let t = Instant::now();
    let mut g2_audits: Vec<(String, OrientationAudit)> = Vec::new();
    for run_i in 0..G2_RUNS {
        let chirality = run_i % 2 == 0;
        let law = m.fhp_i(chirality);
        let sd = transport::ensemble_seed(SEED, run_i);
        let lat = Lattice::seeded(m.clone(), L, sd, D, law);
        let colour = lat.seed_colour_wave(sd, A_COLOUR, K1);
        let mut g = OrientationLattice::from_lattice(lat, sd, rules).with_colour(colour);
        for _ in 0..G2_STEPS {
            g.step();
        }
        g2_audits.push((format!("G2 run {run_i} chirality={chirality} seed={sd:#x}"), g.audit));
    }
    let mut g2_legs: Vec<(String, bool)> = Vec::new();
    let mut step_checks = 0u64;
    let mut g2_totals = g2_audits[0].1;
    for (i, (tag, a)) in g2_audits.iter().enumerate() {
        step_checks += a.steps_checked;
        if i > 0 {
            g2_totals.merge_public(a);
        }
        for (name, ok) in [
            ("mass", a.mass_exact),
            ("px", a.px_exact),
            ("py", a.py_exact),
            ("red", a.red_exact),
            ("orientation total", a.orientation_total_exact),
            ("orientation census", a.orientation_census_exact),
            ("bond balance", a.bond_balance_exact),
        ] {
            g2_legs.push((format!("{tag}: {name}"), ok));
        }
    }
    let tt = g2_totals.totals;
    // M-VACUOUS-SUCCESS: a ledger that conserves because nothing happened has not passed.
    g2_legs.push(("work: bonds formed".to_string(), tt.formed > 0));
    g2_legs.push(("work: rent paid".to_string(), tt.broken_rent > 0));
    g2_legs.push(("work: joint moves refused".to_string(), tt.blocked > 0));
    g2_legs.push(("work: collisions fired".to_string(), tt.collisions_fired > 0));
    g2_legs.push(("no bond lost outside the balance".to_string(), tt.blocked_no_geometry == 0));
    println!(
        "  {G2_RUNS} runs, {step_checks} step-checks in {:.1} s; formed {} broken_rent {} \
         blocked {} (no-vacancy {} / already-claimed {} / no-geometry {}) joint {} hops {}",
        t.elapsed().as_secs_f64(),
        tt.formed,
        tt.broken_rent,
        tt.blocked,
        tt.blocked_no_vacancy,
        tt.blocked_claimed,
        tt.blocked_no_geometry,
        tt.joint_moves,
        tt.anomalous_hops
    );
    r.gate(
        "G2",
        &g2_legs,
        step_checks,
        format!(
            "{G2_RUNS} runs x {G2_STEPS} steps at L={L}, mass/Px/Py/red/orientation census/bond \
             balance integer-identical at every step; work: formed {} broken_rent {} blocked {} \
             (no-vacancy {} / already-claimed {}) collisions {}",
            tt.formed, tt.broken_rent, tt.blocked, tt.blocked_no_vacancy, tt.blocked_claimed,
            tt.collisions_fired
        ),
    );

    // ------------------------------------------------------------------ G0
    println!("\n--- G0: the rule is the map ---");
    let mut g0_legs: Vec<(String, bool)> = Vec::new();
    for a in table.iter() {
        println!(
            "  A({:>3} deg) = {:.9}  E_CT = {:.12} Ha   <- {}",
            a.degrees, a.a, a.e_ct, a.source
        );
    }
    g0_legs.push(("A(0 deg) = 1 exactly".to_string(), table[0].a == 1.0));
    g0_legs.push(("A(60 deg) > 1 on the READ value".to_string(), table[1].a > 1.0));
    g0_legs.push(("A(120 deg) > 1 on the READ value".to_string(), table[2].a > 1.0));
    // The chart, and the two readings of the rent clause it must reproduce.
    let p0 = rules.p_break(0);
    let retention_target = OrientationRules::retention(p0);
    let d_scene = rel(held_phi0, f);
    g0_legs.push((
        format!("the held-geometry linear retention is within {G0_TOL} of the READ f"),
        d_scene <= G0_TOL,
    ));
    // The SECOND, independent leg: the rent clause firing inside the full lattice. The chart
    // run's own break tests at phi=0 must reproduce p_break(0) — a statistic the two-particle
    // scene cannot give and the one that says the rule fires at the right rate in the runs.
    let measured_p0 = if g2_totals.totals.break_tests[0] > 0 {
        Some(g2_totals.totals.broken_by_phi[0] as f64 / g2_totals.totals.break_tests[0] as f64)
    } else {
        None
    };
    let d_run = match measured_p0 {
        Some(v) => rel(v, p0),
        None => f64::INFINITY,
    };
    g0_legs.push((
        format!("the G2 runs' own measured p_break(0 deg) is within {G0_TOL} of the chart's"),
        d_run <= G0_TOL,
    ));
    println!(
        "  chart:  f = {f} (READ) -> E0/kT_lat = {rent:.9} -> p_break(0 deg) = {p0:.9}, \
         1/(1+p_break) = {retention_target:.9}"
    );
    println!(
        "  held-geometry linear retention {held_phi0:.6} vs the READ f {f} -> rel {d_scene:.6}"
    );
    println!(
        "  the G2 runs' measured p_break(0 deg) {} vs the chart's {p0:.9} -> rel {d_run:.6} \
         ({} break tests at phi=0 inside the full lattice)",
        jo(measured_p0),
        g2_totals.totals.break_tests[0]
    );
    r.gate(
        "G0",
        &g0_legs,
        (6 + G3_STEPS + g2_totals.totals.break_tests[0] as usize) as u64,
        format!(
            "A = [{}] read from CT-1/CT-2; f = {f} read from {R_CT2_ARMS}:dimer_293_seam.f; \
             E0/kT_lat = {rent:.9}; held-geometry retention {held_phi0:.6} vs f -> rel \
             {d_scene:.6}; run-measured p_break(0) {} vs {p0:.9} -> rel {d_run:.6}",
            table.iter().map(|a| format!("{:.6}", a.a)).collect::<Vec<_>>().join(", "),
            jo(measured_p0)
        ),
    );

    // ------------------------------------------------------------------ G1
    println!("\n--- G1: the no-bond control is FLUID-0 ---");
    let no_bond = OrientationRules { bonds_enabled: false, ..rules };
    let mut g1_legs: Vec<(String, bool)> = Vec::new();
    let mut g1_detail = String::new();
    let mut g1_rows = Vec::new();
    let mut g1_price_ratios = Vec::new();
    for chirality in [true, false] {
        let law = m.fhp_i(chirality);
        let runs = run_law(&m, &law, &no_bond);
        let (f0_nu1, f0_nu2, f0_d1, f0_d2, f0_sc) = read_fluid0_line(root, chirality);
        println!("  chirality={chirality} digest {:016x}", law_digest(&law));
        println!("    {}", show("nu rows k=1", &runs.nu1));
        println!("    {}", show("nu rows k=2", &runs.nu2));
        println!("    {}", show("D  colour k=1", &runs.d1));
        println!("    {}", show("D  colour k=2", &runs.d2));
        let sc = runs.sc();
        for (name, got, want) in [
            ("nu(k=1)", runs.nu1.value, f0_nu1),
            ("nu(k=2)", runs.nu2.value, f0_nu2),
            ("D(k=1)", runs.d1.value, f0_d1),
            ("D(k=2)", runs.d2.value, f0_d2),
            ("Sc", sc, f0_sc),
        ] {
            let ok = within(got, Some(want), G1_TOL);
            g1_legs.push((format!("chirality={chirality}: {name} within {G1_TOL} of FLUID-0's"), ok));
            println!(
                "    {name:<9} {} vs FLUID-0 {want:.9e}  rel {}  <- {R_FLUID0_GATE}",
                jo(got),
                match got {
                    Some(v) => format!("{:.9}", rel(v, want)),
                    None => "undefined (the reading refused)".to_string(),
                }
            );
            let _ = write!(
                g1_detail,
                "chir={chirality} {name} {} vs {want:.6e} rel {}; ",
                jo(got),
                match got {
                    Some(v) => format!("{:.6}", rel(v, want)),
                    None => "undefined".to_string(),
                }
            );
        }
        g1_legs.push((
            format!("chirality={chirality}: all four readouts Fitted"),
            runs.all_fitted(),
        ));
        g1_legs.push((
            format!("chirality={chirality}: no bond formed with bonds forbidden"),
            runs.audits.iter().all(|a| a.totals.formed == 0),
        ));
        g1_price_ratios.push((format!("G1 no-bond chirality={chirality}"), runs.price_ratio()));
        g1_rows.push(format!(
            "{{\"fluid0\": {{\"nu_k1\": {}, \"nu_k2\": {}, \"d_k1\": {}, \"d_k2\": {}, \
             \"sc\": {}, \"source\": \"{R_FLUID0_GATE}\"}}, \"fluid1_no_bond\": {}}}",
            jf(f0_nu1),
            jf(f0_nu2),
            jf(f0_d1),
            jf(f0_d2),
            jf(f0_sc),
            law_runs_json("G1 no-bond control", chirality, &law, &runs)
        ));
    }
    r.gate("G1", &g1_legs, 12, g1_detail);

    // ------------------------------------------------------------------ the plants
    println!("\n--- P(i): the acceptor factor flattened ---");
    let flat = OrientationRules { amplitude: [1.0; N_DIRS], ..rules };
    let law = m.fhp_i(true);
    let chart_census = bond_census(&m, &law, &rules);
    let flat_census = bond_census(&m, &law, &flat);
    let carrier_i = table[2].a - 1.0;
    let carrier_i_ok = carrier_i >= PLANT_I_CARRIER_FLOOR;
    let change_i = rel(chart_census.donor, flat_census.donor);
    let mut pi_legs: Vec<(String, bool)> = Vec::new();
    pi_legs.push((
        format!("carrier A(120 deg) - 1 = {carrier_i:.6} >= {PLANT_I_CARRIER_FLOOR} on the READ table"),
        carrier_i_ok,
    ));
    pi_legs.push((
        format!("the bond count moved by more than {PLANT_I_MIN_CHANGE}"),
        change_i > PLANT_I_MIN_CHANGE,
    ));
    pi_legs.push(("work: the chart run formed bonds".to_string(), chart_census.formed > 0));
    pi_legs.push(("work: the plant run formed bonds".to_string(), flat_census.formed > 0));
    println!(
        "  chart  bonds/particle donor {:.6} degree {:.6} largest {:.6} spanning {}/{}",
        chart_census.donor,
        chart_census.degree,
        chart_census.largest_fraction,
        chart_census.spanning_steps,
        chart_census.samples
    );
    println!(
        "  plant  bonds/particle donor {:.6} degree {:.6} largest {:.6} spanning {}/{}",
        flat_census.donor,
        flat_census.degree,
        flat_census.largest_fraction,
        flat_census.spanning_steps,
        flat_census.samples
    );
    println!("  change {change_i:.6} against the stake {PLANT_I_MIN_CHANGE}; carrier A(120)-1 = {carrier_i:.6}");
    // What the plant's own sector CAN move, from the two READ inputs alone. The amplitude
    // table enters the dynamics only through `p_break(phi) = exp(-A(phi)*E0/kT)`, so the whole
    // table is worth exactly the spread of `1/(1 + p_break)` between its smallest and largest
    // amplitude at this chart. Flattening the table to `A = 1` cannot move the bond count
    // further than that, whatever the lattice does with it.
    let a_max = table.iter().map(|x| x.a).fold(1.0f64, f64::max);
    let r_flat = OrientationRules::retention((-rent).exp());
    let r_max = OrientationRules::retention((-a_max * rent).exp());
    let reach_i = rel(r_flat, r_max);
    println!(
        "  the plant's REACH from the freeze's own two read inputs: A in [1, {a_max:.6}] at          E0/kT_lat = {rent:.6} gives a per-angle retention in [{r_flat:.6}, {r_max:.6}], a          spread of {reach_i:.6}"
    );
    if change_i <= PLANT_I_MIN_CHANGE {
        println!(
            "  THE STAKE IS UNREACHABLE BY THE FREEZE'S OWN ARITHMETIC, not by the measurement:              the amplitude table can move the per-angle retention by at most {reach_i:.6} at this              chart, well under the {PLANT_I_MIN_CHANGE} stake, and the measured bond-count change              {change_i:.6} sits inside that reach. The carrier IS nonzero in the sector the plant              acts on (A(120)-1 = {carrier_i:.6}); it is the OBSERVABLE that cannot respond,              because E0/kT_lat = {rent:.6} is small enough that a 42 % change in A is an 8 %              change in p_break. The instrument reports FAIL and does not move the target."
        );
    }
    r.gate(
        "P(i)",
        &pi_legs,
        chart_census.samples + flat_census.samples,
        format!(
            "carrier A(120 deg)-1 = {carrier_i:.6} (floor {PLANT_I_CARRIER_FLOOR}, asserted \
             {carrier_i_ok}); bonds/particle (each bond once) chart {:.6} vs A=1 plant {:.6} \
             -> change {change_i:.6} (stake > {PLANT_I_MIN_CHANGE}); degree convention \
             {:.6} vs {:.6}; ANALYTIC reach: A in [1, {a_max:.6}] at E0/kT_lat = {rent:.6} moves \
             the per-angle retention only from {r_flat:.6} to {r_max:.6}, a spread of \
             {reach_i:.6}, so the stake is unreachable by the freeze's own arithmetic rather \
             than by the measurement",
            chart_census.donor, flat_census.donor, chart_census.degree, flat_census.degree
        ),
    );

    println!("\n--- P(ii): the rent doubled ---");
    let doubled = OrientationRules { rent: rent * 2.0, ..rules };
    let (held_double, expect_double, tests_double, _) =
        held_fraction_scene(0, G3_STEPS, doubled);
    let change_ii = rel(held_phi0, held_double);
    let carrier_ii_ok = f > 0.0 && f < 1.0;
    let mut pii_legs: Vec<(String, bool)> = Vec::new();
    pii_legs.push((
        format!("carrier: the READ retention {f} is nonzero and under 1"),
        carrier_ii_ok,
    ));
    pii_legs.push((
        "work: the plant scene exposed a bond to the rule".to_string(),
        tests_double > 1_000,
    ));
    pii_legs.push((
        format!("the linear-bond retention moved by more than {PLANT_II_MIN_CHANGE}"),
        change_ii > PLANT_II_MIN_CHANGE,
    ));
    // The plant's effect on the sector it acts on, reported beside and NEVER gated in place of
    // the freeze's own stake: the break probability, and the bond count on the full lattice.
    let doubled_census = bond_census(&m, &law, &doubled);
    let p0_double = doubled.p_break(0);
    let change_p = rel(p0, p0_double);
    let change_bonds = rel(chart_census.donor, doubled_census.donor);
    println!(
        "  chart   E0/kT_lat {rent:.9}  p_break(0) {p0:.9}  retention (measured) {held_phi0:.6}"
    );
    println!(
        "  plant   E0/kT_lat {:.9}  p_break(0) {p0_double:.9}  retention (measured) \
         {held_double:.6}  1/(1+p) {expect_double:.6}",
        doubled.rent
    );
    println!(
        "  retention change {change_ii:.6} against the stake {PLANT_II_MIN_CHANGE}  \
         [break probability change {change_p:.6}; bond count change {change_bonds:.6}, \
         reported, NOT gated]"
    );
    if change_ii <= PLANT_II_MIN_CHANGE {
        println!(
            "  THE STAKE IS UNREACHABLE BY THE FREEZE'S OWN ARITHMETIC, not by the measurement: \
             the retention is the logistic 1/(1+exp(-E)), so doubling E from {rent:.6} moves it \
             from {:.6} to {:.6}, a change of {:.6} — the largest a doubling can produce at this \
             chart. The instrument reports FAIL and does not move the target.",
            OrientationRules::retention(p0),
            OrientationRules::retention(p0_double),
            rel(OrientationRules::retention(p0), OrientationRules::retention(p0_double))
        );
    }
    r.gate(
        "P(ii)",
        &pii_legs,
        (2 * G3_STEPS) as u64,
        format!(
            "carrier: the READ retention {f} in (0,1), asserted {carrier_ii_ok}; E0/kT_lat \
             {rent:.9} -> {:.9}; measured linear retention {held_phi0:.6} -> {held_double:.6}, \
             change {change_ii:.6} (stake > {PLANT_II_MIN_CHANGE}); ANALYTIC bound: the \
             retention is the logistic 1/(1+exp(-E)) and a doubling of E at this chart moves it \
             by exactly {:.6}, so the stake is unreachable by the freeze's own arithmetic \
             rather than by the measurement; reported beside and NOT gated: p_break(0) \
             {p0:.9} -> {p0_double:.9}, change {change_p:.6}; bonds/particle {:.6} -> {:.6}, \
             change {change_bonds:.6}",
            doubled.rent,
            rel(OrientationRules::retention(p0), OrientationRules::retention(p0_double)),
            chart_census.donor,
            doubled_census.donor
        ),
    );

    // ------------------------------------------------------------------ P
    println!("\n--- P: the price ---");
    let fluid0_price = read_fluid0_price(root);
    // The orientation overhead, MEASURED: the same scene under the chart's rules and under
    // FLUID-0's own step, over PRICE_STEPS steps each.
    let plain_price = {
        let sd = transport::ensemble_seed(SEED, 0);
        let lat = Lattice::seeded(m.clone(), L, sd, D, law.clone());
        let mut cells = lat.cells.clone();
        let mut col = lat.seed_colour_wave(sd, A_COLOUR, K1);
        let mut oc = vec![0u8; L * L];
        let mut oq = vec![0u8; L * L];
        for t in 0..20u64 {
            lat.advance_with_colour(&mut cells, &mut col, &mut oc, &mut oq, t);
            std::mem::swap(&mut cells, &mut oc);
            std::mem::swap(&mut col, &mut oq);
        }
        let t0 = Instant::now();
        for t in 20..(20 + PRICE_STEPS as u64) {
            lat.advance_with_colour(&mut cells, &mut col, &mut oc, &mut oq, t);
            std::mem::swap(&mut cells, &mut oc);
            std::mem::swap(&mut col, &mut oq);
        }
        t0.elapsed().as_secs_f64() / PRICE_STEPS as f64
    };
    let chart_price = measure_price(&m, &law, &rules);
    let overhead = chart_price / plain_price;
    let projected_law = fluid0_price * overhead;
    let mut price_rows: Vec<(String, f64)> = Vec::new();
    price_rows.extend(g1_price_ratios);
    price_rows.push(("P(i) chart bond census".to_string(), {
        let e = chart_census.price_seconds_per_member_step
            * (SEEDS * (BOND_WARMUP + BOND_SAMPLE)) as f64;
        chart_census.seconds / e
    }));
    price_rows.push(("P(i) flattened bond census".to_string(), {
        let e = flat_census.price_seconds_per_member_step
            * (SEEDS * (BOND_WARMUP + BOND_SAMPLE)) as f64;
        flat_census.seconds / e
    }));
    price_rows.push(("P(ii) doubled-rent bond census".to_string(), {
        let e = doubled_census.price_seconds_per_member_step
            * (SEEDS * (BOND_WARMUP + BOND_SAMPLE)) as f64;
        doubled_census.seconds / e
    }));
    let mut p_legs: Vec<(String, bool)> = Vec::new();
    p_legs.push(("the measured orientation overhead is positive".to_string(), overhead > 0.0));
    for (tag, ratio) in &price_rows {
        let ok = *ratio >= PRICE_LOW && *ratio <= PRICE_HIGH;
        println!("  {tag:<38} actual/price = {ratio:.4}   band [{PRICE_LOW}, {PRICE_HIGH}]");
        p_legs.push((format!("{tag}: within [{PRICE_LOW}, {PRICE_HIGH}] of its own price"), ok));
    }
    println!(
        "  plain step {plain_price:.6} s   chart step {chart_price:.6} s   \
         MEASURED orientation overhead {overhead:.4}x"
    );
    println!(
        "  FLUID-0's price {fluid0_price:.6} s/law <- {R_FLUID0_PRICE}:price_seconds_per_law; \
         FLUID-1's projected {projected_law:.4} s/law (4 readings, {SEEDS} members)"
    );
    let price_json = format!(
        "{{\n  \"phase\": \"price\",\n  \"params\": {},\n  \
         \"fluid0_seconds_per_law\": {},\n  \"fluid0_source\": \"{R_FLUID0_PRICE}:price_seconds_per_law\",\n  \
         \"plain_seconds_per_step\": {},\n  \"chart_seconds_per_step\": {},\n  \
         \"orientation_overhead\": {},\n  \"projected_seconds_per_law\": {},\n  \
         \"band_low_seconds_per_law\": {},\n  \"band_high_seconds_per_law\": {},\n  \
         \"price_steps\": {PRICE_STEPS},\n  \"per_run\": [\n    {}\n  ]\n}}\n",
        params_json(root, table, f, rent),
        jf(fluid0_price),
        jf(plain_price),
        jf(chart_price),
        jf(overhead),
        jf(projected_law),
        jf(PRICE_LOW * projected_law),
        jf(PRICE_HIGH * projected_law),
        price_rows
            .iter()
            .map(|(t, v)| format!("{{\"run\": {:?}, \"actual_over_price\": {}}}", t, jf(*v)))
            .collect::<Vec<_>>()
            .join(",\n    ")
    );
    std::fs::write(format!("{out_dir}/price.json"), &price_json).expect("write price.json");
    println!("  wrote {out_dir}/price.json  (before any counted run)");
    r.gate(
        "P",
        &p_legs,
        price_rows.len() as u64 + 1,
        format!(
            "plain {plain_price:.6} s/step, chart {chart_price:.6} s/step, MEASURED overhead \
             {overhead:.4}x; FLUID-0 {fluid0_price:.6} s/law -> FLUID-1 {projected_law:.4} s/law"
        ),
    );

    // ------------------------------------------------------------------ amplitude_table.json
    let amp_json = format!(
        "{{\n  \"phase\": \"amplitude_table\",\n  \"note\": \"READ at run time from CT-1's and \
         CT-2's own records; no value here is typed into the crate\",\n  \
         \"e_ct_rule\": \"E_exact(total) - E_noCT(total), the tilt records' own e_ct_rule\",\n  \
         \"records_root\": {:?},\n  \"retention\": {},\n  \
         \"retention_source\": \"{R_CT2_ARMS}:dimer_293_seam.f\",\n  \
         \"rent_e0_over_kt\": {},\n  \"p_break_phi0\": {},\n  \"table\": [\n    {}\n  ]\n}}\n",
        root,
        jf(f),
        jf(rent),
        jf(rules.p_break(0)),
        amplitude_table_json(table)
    );
    std::fs::write(format!("{out_dir}/amplitude_table.json"), &amp_json)
        .expect("write amplitude_table.json");
    println!("\n  wrote {out_dir}/amplitude_table.json");

    // ------------------------------------------------------------------ gate.json
    let _ = write!(json, "{{\n  \"phase\": \"gate\",\n  \"params\": {},\n", params_json(root, table, f, rent));
    let _ = write!(json, "  \"verdicts\": {{{}}},\n", r.entries.join(", "));
    let _ = write!(json, "  \"g3\": [\n    {}\n  ],\n", g3_rows.join(",\n    "));
    let _ = write!(json, "  \"g1\": [\n    {}\n  ],\n", g1_rows.join(",\n    "));
    let _ = write!(
        json,
        "  \"g2\": {{\"runs\": {G2_RUNS}, \"steps\": {G2_STEPS}, \"l\": {L}, \
         \"step_checks\": {step_checks}, \"audits\": [\n    {}\n  ]}},\n",
        g2_audits
            .iter()
            .map(|(t, a)| format!("{{\"run\": {:?}, \"audit\": {}}}", t, audit_json(a)))
            .collect::<Vec<_>>()
            .join(",\n    ")
    );
    let _ = write!(
        json,
        "  \"plants\": {{\"i\": {{\"carrier_a120_minus_1\": {}, \"carrier_floor\": {}, \
         \"stake\": {}, \"change\": {}, \"amplitude_max\": {}, \"retention_at_a1\": {}, \
         \"retention_at_a_max\": {}, \"analytic_reach_of_the_table\": {}, \
         \"chart\": {}, \"plant\": {}}}, \
         \"ii\": {{\"carrier_retention\": {}, \"stake\": {}, \"change\": {}, \
         \"rent_chart\": {}, \"rent_plant\": {}, \"retention_chart\": {}, \
         \"retention_plant\": {}, \"p_break_chart\": {}, \"p_break_plant\": {}, \
         \"p_break_change\": {}, \"bond_count_change\": {}, \
         \"analytic_max_change_of_a_doubling\": {}, \"plant_census\": {}}}}},\n",
        jf(carrier_i),
        jf(PLANT_I_CARRIER_FLOOR),
        jf(PLANT_I_MIN_CHANGE),
        jf(change_i),
        jf(a_max),
        jf(r_flat),
        jf(r_max),
        jf(reach_i),
        bond_census_json("chart", &chart_census),
        bond_census_json("plant (i) A = 1", &flat_census),
        jf(f),
        jf(PLANT_II_MIN_CHANGE),
        jf(change_ii),
        jf(rent),
        jf(doubled.rent),
        jf(held_phi0),
        jf(held_double),
        jf(p0),
        jf(p0_double),
        jf(change_p),
        jf(change_bonds),
        jf(rel(OrientationRules::retention(p0), OrientationRules::retention(p0_double))),
        bond_census_json("plant (ii) rent doubled", &doubled_census)
    );
    let _ = write!(
        json,
        "  \"price\": {{\"plain_seconds_per_step\": {}, \"chart_seconds_per_step\": {}, \
         \"orientation_overhead\": {}, \"fluid0_seconds_per_law\": {}, \
         \"projected_seconds_per_law\": {}, \"gate_seconds\": {}}},\n",
        jf(plain_price),
        jf(chart_price),
        jf(overhead),
        jf(fluid0_price),
        jf(projected_law),
        jf(t_gate.elapsed().as_secs_f64())
    );
    let _ = write!(
        json,
        "  \"failures\": [{}]\n}}\n",
        r.failures.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(", ")
    );
    std::fs::write(format!("{out_dir}/gate.json"), &json).expect("write gate.json");
    println!("  wrote {out_dir}/gate.json");

    println!("\n=========================================================================");
    if r.failures.is_empty() {
        println!("GATE VERDICT: all gates PASS. ({:.1} s)", t_gate.elapsed().as_secs_f64());
    } else {
        println!(
            "GATE VERDICT: {} gate(s) FAILED — reported, not repaired ({:.1} s):",
            r.failures.len(),
            t_gate.elapsed().as_secs_f64()
        );
        for x in &r.failures {
            println!("  - {x}");
        }
    }
    println!("=========================================================================");
}

// ---------------------------------------------------------------- run
fn run(out_dir: &str, root: &str, table: &[Amp; N_DIRS], f: f64, rent: f64, threads: usize) {
    let m = Model::fhp6();
    let amp = amplitudes(table);
    let chart = chart_rules(amp, f);
    let t0 = Instant::now();
    let mut json = String::new();
    let _ = write!(json, "{{\n  \"phase\": \"run\",\n  \"params\": {},\n", params_json(root, table, f, rent));

    // The configurations, each named the way the freeze names it.
    let configs: Vec<(&str, OrientationRules)> = vec![
        ("chart", chart),
        ("cold control (p_break = 0)", OrientationRules { cold: true, ..chart }),
        ("no-bond control", OrientationRules { bonds_enabled: false, ..chart }),
        ("plant (i) A = 1", OrientationRules { amplitude: [1.0; N_DIRS], ..chart }),
        ("plant (ii) rent doubled", OrientationRules { rent: rent * 2.0, ..chart }),
    ];

    // ---- the transport runs: every configuration, both chiralities, two seeds ----
    println!("\n--- the runs: transport at L={L}, {SEEDS} seeds, both chiralities ---");
    let mut rows = Vec::new();
    let mut price_rows = Vec::new();
    for (tag, rules) in &configs {
        for chirality in [true, false] {
            let law = m.fhp_i(chirality);
            println!("  {tag}  chirality={chirality}  digest {:016x}", law_digest(&law));
            let runs = run_law(&m, &law, rules);
            println!("    price {:.9} s/member-step (first {PRICE_STEPS} steps, before the counted steps)",
                runs.price_seconds_per_member_step);
            println!("    {}", show("nu rows k=1", &runs.nu1));
            println!("    {}", show("nu rows k=2", &runs.nu2));
            println!("    {}", show("D  colour k=1", &runs.d1));
            println!("    {}", show("D  colour k=2", &runs.d2));
            println!(
                "    Sc {}   ledger exact {}   actual/price {:.4}   {:.1} s",
                jo(runs.sc()),
                runs.audits.iter().all(|a| a.all_exact()),
                runs.price_ratio(),
                runs.seconds_total()
            );
            price_rows.push(format!(
                "{{\"run\": {:?}, \"chirality\": {}, \"price_seconds_per_member_step\": {}, \
                 \"member_steps\": {}, \"seconds\": {}, \"actual_over_price\": {}}}",
                tag,
                jb(chirality),
                jf(runs.price_seconds_per_member_step),
                runs.member_steps(),
                jf(runs.seconds_total()),
                jf(runs.price_ratio())
            ));
            rows.push(law_runs_json(tag, chirality, &law, &runs));
        }
    }
    let _ = write!(json, "  \"transport\": [\n    {}\n  ],\n", rows.join(",\n    "));

    // ---- the bond censuses: the count, the component, the spanning reading ----
    println!("\n--- the bond censuses (warm-up {BOND_WARMUP}, sample {BOND_SAMPLE}) ---");
    let law = m.fhp_i(true);
    let mut census_rows = Vec::new();
    for (tag, rules) in &configs {
        let c = bond_census(&m, &law, rules);
        println!(
            "  {tag:<28} donor {:.6}  degree {:.6}  largest {:.6}  spanning {}/{}  \
             formed {} broken {} blocked {} ({:.1} s)",
            c.donor,
            c.degree,
            c.largest_fraction,
            c.spanning_steps,
            c.samples,
            c.formed,
            c.broken_rent,
            c.blocked,
            c.seconds
        );
        census_rows.push(bond_census_json(tag, &c));
    }
    let _ = write!(json, "  \"bond_census\": [\n    {}\n  ],\n", census_rows.join(",\n    "));

    // ---- node LG's block-chart witness rates, on the orientation lattice ----
    println!("\n--- W: node LG's block-chart witness rate on the chart's own motion ---");
    let base = {
        let sd = transport::ensemble_seed(SEED, 0);
        let lat = Lattice::seeded(m.clone(), L, sd, D, law.clone());
        let mut g = OrientationLattice::from_lattice(lat, sd, chart);
        for _ in 0..BOND_WARMUP {
            g.step();
        }
        g
    };
    let next = std::sync::atomic::AtomicUsize::new(0);
    let out: std::sync::Mutex<Vec<WitnessReading>> = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|sc| {
        for _ in 0..threads.min(W_BLOCKS.len()) {
            let (next, out, base) = (&next, &out, &base);
            sc.spawn(move || loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if i >= W_BLOCKS.len() {
                    break;
                }
                let b = W_BLOCKS[i];
                let t = Instant::now();
                let w = witness_rate(base, b, W_STEPS);
                println!(
                    "  b={b:<3} probes {} witnesses {} rate {:.6} +/- {:.6} vs W(b) {:.6} \
                     (rel {:.6})  {:.1} s",
                    w.probes,
                    w.witnesses,
                    w.rate(),
                    w.stderr(),
                    w.predicted,
                    rel(w.rate(), w.predicted),
                    t.elapsed().as_secs_f64()
                );
                out.lock().unwrap().push(w);
            });
        }
    });
    let mut results = out.into_inner().unwrap();
    results.sort_by_key(|w| w.b);
    let _ = write!(
        json,
        "  \"witness\": [\n    {}\n  ],\n",
        results
            .iter()
            .map(|w| format!(
                "{{\"b\": {}, \"l\": {L}, \"steps\": {W_STEPS}, \"probes\": {}, \
                 \"witnesses\": {}, \"rate\": {}, \"stderr\": {}, \"predicted\": {}, \
                 \"relative\": {}, \"vacuous_by_conservation\": {}}}",
                w.b,
                w.probes,
                w.witnesses,
                jf(w.rate()),
                jf(w.stderr()),
                jf(w.predicted),
                jf(rel(w.rate(), w.predicted)),
                jb(BlockChart::new(w.b, L).map(|c| c.is_vacuous_by_conservation()).unwrap_or(false))
            ))
            .collect::<Vec<_>>()
            .join(",\n    ")
    );

    let _ = write!(json, "  \"price\": [\n    {}\n  ],\n", price_rows.join(",\n    "));
    let _ = write!(json, "  \"seconds\": {}\n}}\n", jf(t0.elapsed().as_secs_f64()));
    std::fs::write(format!("{out_dir}/run.json"), &json).expect("write run.json");
    std::fs::write(
        format!("{out_dir}/run.done"),
        format!("run complete in {:.1} s\n", t0.elapsed().as_secs_f64()),
    )
    .expect("write run.done");
    println!("\n  wrote {out_dir}/run.json and {out_dir}/run.done ({:.1} s)", t0.elapsed().as_secs_f64());
}

// ---------------------------------------------------------------- read
fn read(out_dir: &str, root: &str) {
    let path = format!("{out_dir}/run.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("read {path}: {e} — `run` must finish before `read`; its marker is run.done")
    });
    let done = std::path::Path::new(&format!("{out_dir}/run.done")).exists();
    if !done {
        println!("  run.done is ABSENT: the read below is on a run that did not finish.");
    }

    // ---- B: the bond count and the phase ----
    let chart = section(&text, "\"bond_census\"", "\"run\": \"chart\"");
    let degree = num(&chart, "bonds_per_particle_degree");
    let donor = num(&chart, "bonds_per_particle_donor");
    let spanning_fraction = num(&chart, "spanning_fraction");
    let largest = num(&chart, "largest_fraction");
    let box_count = read_box_bonds_per_molecule(root);
    let lo = box_count / B_BAND_FACTOR;
    let hi = box_count * B_BAND_FACTOR;
    let spans = spanning_fraction > 0.5;
    let in_band = donor >= lo && donor <= hi;
    let (b_branch, b_text) = if !spans {
        (
            "c",
            format!(
                "the bond graph does not span the torus (a spanning step in {:.4} of the sampled \
                 steps, largest component {largest:.6} of the particles): the closure is local",
                spanning_fraction
            ),
        )
    } else if in_band {
        (
            "a",
            format!(
                "the bond graph spans and {donor:.6} bonds per particle is inside \
                 [{lo:.6}, {hi:.6}], the molecular box's {box_count} within a factor \
                 {B_BAND_FACTOR}"
            ),
        )
    } else {
        (
            "b",
            format!(
                "the bond graph spans but {donor:.6} bonds per particle is outside \
                 [{lo:.6}, {hi:.6}] around the molecular box's {box_count}"
            ),
        )
    };
    println!("\n  B branch ({b_branch}): {b_text}");
    println!(
        "    conventions: {donor:.6} each-bond-once (the convention the box's {box_count} is \
         quoted in, source {R_LIQUID1_ARM}:r2.hbonds_per_molecule) and {degree:.6} degree \
         (the freeze's [0,2] range). The lattice's each-bond-once CEILING is 1.0 — one donor \
         arm per particle — against water's 2 on the lens, so the ceiling fraction is \
         {:.6} against the box's {:.6}.",
        donor,
        box_count / 2.0
    );

    // ---- S: the Schmidt stake ----
    let mut sc_rows = Vec::new();
    for chirality in [true, false] {
        let s = section(
            &text,
            "\"transport\"",
            &format!("\"run\": \"chart\", \"chirality\": {}", jb(chirality)),
        );
        let nu1 = num(&s, "\"nu_k1\": {\"value\"");
        let sc = num(&s, "\"sc\"");
        let fitted = s.contains("\"all_fitted\": true");
        sc_rows.push((chirality, nu1, sc, fitted));
    }
    let best = sc_rows.iter().filter(|(_, _, s, fit)| *fit && s.is_finite()).map(|(_, _, s, _)| *s).fold(f64::NAN, f64::max);
    let (s_branch, s_text) = if !best.is_finite() {
        (
            "void",
            "no chart run is Fitted on both readouts: S cannot be read (a cap is a refusal)"
                .to_string(),
        )
    } else if best >= SC_BRANCH_A {
        (
            "a",
            format!(
                "Sc = {best:.6e} >= {SC_BRANCH_A}: a single-species carrier with bound pairs \
                 reaches water's ratio (the kill band, from experiment: Sc_water ~= {SC_WATER})"
            ),
        )
    } else if best >= SC_BRANCH_B {
        (
            "b",
            format!(
                "Sc = {best:.6e} is in [{SC_BRANCH_B}, {SC_BRANCH_A}): closure moves the ratio \
                 by the two decades FLUID-0 said no single-species rule could, and the rest is \
                 the chart's (Sc_water ~= {SC_WATER}, from experiment)"
            ),
        )
    } else {
        (
            "c",
            format!(
                "Sc = {best:.6e} < {SC_BRANCH_B}: trapping at this bond fraction does not \
                 separate momentum from particles; the element needs a stronger closure than a \
                 pair, or the network's own dynamics (Sc_water ~= {SC_WATER}, from experiment)"
            ),
        )
    };
    println!("\n  S branch ({s_branch}): {s_text}");
    for (chirality, nu, sc, fitted) in &sc_rows {
        println!("    chirality={chirality}: nu(k=1) {nu:.6e}  Sc {sc:.6e}  all four Fitted {fitted}");
    }

    // ---- W: the block chart's defect ----
    let mut ws = Vec::new();
    for b in W_BLOCKS {
        let s = section(&text, "\"witness\"", &format!("\"b\": {b},"));
        ws.push((b, num(&s, "rate"), num(&s, "predicted"), num(&s, "stderr")));
    }
    let all_within = ws.iter().all(|(_, r, p, _)| rel(*r, *p) <= W_TOL);
    let above_small = ws.first().map(|(_, r, p, _)| *r > *p).unwrap_or(false);
    let converging = ws
        .windows(2)
        .all(|w| rel(w[1].1, w[1].2) <= rel(w[0].1, w[0].2) + 1e-12);
    let any_below = ws.iter().any(|(_, r, p, s)| *r < *p - 3.0 * *s);
    let (w_branch, w_text) = if all_within {
        (
            "a",
            format!(
                "the witness rate is within {W_TOL} of W(b) = 1 - (b-2)^2/b^2 at every b in \
                 {W_BLOCKS:?}: the closure inside the block does not change the block's closure"
            ),
        )
    } else if above_small && converging {
        (
            "b",
            "the witness rate is above W(b) at small b and converging: bonds crossing the \
             block face add to the defect by a measured amount"
                .to_string(),
        )
    } else if any_below {
        (
            "c",
            "the witness rate is BELOW W(b) by more than three standard errors: a closure in \
             the carrier makes the block MORE closed than its boundary fraction — a finding \
             about the fluid tier itself, entered as such"
                .to_string(),
        )
    } else {
        (
            "void",
            "the witness rate matches no branch as the freeze letters them: neither within \
             tolerance everywhere, nor above-and-converging, nor below by three standard errors"
                .to_string(),
        )
    };
    println!("\n  W branch ({w_branch}): {w_text}");
    for (b, rate, pred, se) in &ws {
        println!("    b={b:<3} rate {rate:.6} +/- {se:.6}  W(b) {pred:.6}  rel {:.6}", rel(*rate, *pred));
    }

    let json = format!(
        "{{\n  \"phase\": \"read\",\n  \"run_done\": {},\n  \
         \"b\": {{\"branch\": \"{b_branch}\", \"text\": {:?}, \
         \"bonds_per_particle_donor\": {}, \"bonds_per_particle_degree\": {}, \
         \"box_bonds_per_molecule\": {}, \"box_source\": \"{R_LIQUID1_ARM}:r2.hbonds_per_molecule\", \
         \"band\": [{}, {}], \"band_factor\": {}, \"in_band\": {}, \
         \"spanning_fraction\": {}, \"spans\": {}, \"largest_fraction\": {}, \
         \"lattice_each_bond_once_ceiling\": 1.0, \"lens_each_bond_once_ceiling\": 2.0}},\n  \
         \"s\": {{\"branch\": \"{s_branch}\", \"text\": {:?}, \"sc_max\": {}, \
         \"sc_branch_a\": {}, \"sc_branch_b\": {}, \"sc_water\": {}, \
         \"sc_water_note\": \"THE KILL, typed from experiment\", \"per_chirality\": [{}]}},\n  \
         \"w\": {{\"branch\": \"{w_branch}\", \"text\": {:?}, \"tolerance\": {}, \
         \"witness\": \"closed_iff_fiber_invariant (node LG's; the rate is against its law)\", \
         \"rows\": [{}]}}\n}}\n",
        jb(done),
        b_text,
        jf(donor),
        jf(degree),
        jf(box_count),
        jf(lo),
        jf(hi),
        jf(B_BAND_FACTOR),
        jb(in_band),
        jf(spanning_fraction),
        jb(spans),
        jf(largest),
        s_text,
        jf(best),
        jf(SC_BRANCH_A),
        jf(SC_BRANCH_B),
        jf(SC_WATER),
        sc_rows
            .iter()
            .map(|(c, nu, sc, fit)| format!(
                "{{\"chirality\": {}, \"nu_k1\": {}, \"sc\": {}, \"all_fitted\": {}}}",
                jb(*c),
                jf(*nu),
                jf(*sc),
                jb(*fit)
            ))
            .collect::<Vec<_>>()
            .join(", "),
        w_text,
        jf(W_TOL),
        ws.iter()
            .map(|(b, r2, p, s)| format!(
                "{{\"b\": {b}, \"rate\": {}, \"predicted\": {}, \"stderr\": {}, \"relative\": {}}}",
                jf(*r2),
                jf(*p),
                jf(*s),
                jf(rel(*r2, *p))
            ))
            .collect::<Vec<_>>()
            .join(", ")
    );
    std::fs::write(format!("{out_dir}/read.json"), &json).expect("write read.json");
    println!("\n  wrote {out_dir}/read.json");
}

/// The slice of `text` that begins at `anchor` after `head` — the record's own object, read
/// forward. A missing anchor is a refusal, never an empty default.
fn section(text: &str, head: &str, anchor: &str) -> String {
    let h = text.find(head).unwrap_or_else(|| panic!("run.json has no {head}"));
    let a = h + text[h..].find(anchor).unwrap_or_else(|| panic!("run.json has no {anchor} under {head}"));
    let rest = &text[a..];
    let end = rest.len().min(20000);
    rest[..end].to_string()
}

fn num(s: &str, key: &str) -> f64 {
    let pat = if key.starts_with('"') { format!("{key}:") } else { format!("\"{key}\":") };
    let i = s.find(&pat).unwrap_or_else(|| panic!("no field {key} in the section")) + pat.len();
    let rest = &s[i..];
    let end = rest.find([',', '}', '\n']).unwrap_or(rest.len());
    rest[..end].trim().parse::<f64>().unwrap_or(f64::NAN)
}
