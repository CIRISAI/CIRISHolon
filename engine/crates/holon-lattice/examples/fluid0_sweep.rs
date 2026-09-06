//! FLUID-0's runner: the gates, the census of all 4,608 REG+ collision laws, and the read.
//!
//! `conformance/mesh/FLUID0_PREREG.md` is the freeze and `FLUID0_AMENDMENT_1` set A is the
//! instrument. Every number the instrument uses lives in ONE block below and nowhere else, and
//! every number this prints is printed (M-PRESENTATION-VERDICT). Usage:
//!
//! ```text
//! fluid0_sweep gate  [out_dir]   G0-G3, the two plants, price.json, gate.json
//! fluid0_sweep sweep [out_dir] [threads]  the census -> census.jsonl (RESUMABLE, PARALLEL
//!                                         across laws), sweep.done
//! fluid0_sweep read  [out_dir]   C and S -> census_summary.json
//! fluid0_sweep diag  [out_dir]   NOT in the freeze: the FROZEN instrument's noise floor
//! ```
//!
//! `out_dir` defaults to `../conformance/mesh/fluid0`, i.e. run from `engine/`. The record of
//! the instrument AS FROZEN is `gate_as_frozen.json` / `gate_as_frozen.log`, kept beside the
//! amended one; `diag` still runs at the frozen parameters and is what convicted them.

// Waived, each for one reason: `write_with_newline` (the JSON is assembled line by line and
// the trailing newline is part of the line), `needless_range_loop` (`t` is a time coordinate),
// and `manual_is_multiple_of` (the progress modulus predates that method's stabilisation).
#![allow(clippy::write_with_newline, clippy::needless_range_loop, clippy::manual_is_multiple_of)]

use holon_lattice::lattice::ColourRule;
use holon_lattice::state::Model;
use holon_lattice::transport::{
    self, boltzmann_nu_fhp_i, colour_ensemble, colour_series, fit, law_digest, shear_ensemble,
    shear_series, wavenumber, Exit, FitRule, LedgerAudit, TransportReading, COLUMN_TRANSVERSE,
    K_AGREEMENT_TOLERANCE, MIN_POINTS, MIN_R2, MONOTONE_TOLERANCE_FRACTION,
};
use std::fmt::Write as _;
use std::io::Write as _;
use std::time::Instant;

// ═════════════════════ FLUID0_AMENDMENT_1, SET A — THE ONE DECLARED BLOCK ═════════════════
// Every line names the MEASUREMENT that fixed it. The measurements are `fluid0_probe`'s and
// are reproduced by `fluid0_probe 8 all`; nothing below is typed from outside this campaign
// except the credited Boltzmann comparison and §1's two experimental water numbers, each
// named as such where it appears.

/// Lattice size. The SMALLEST at which both readouts pass the freeze's own 10 % two-wavevector
/// test: measured gaps are shear 2.0 % and tracer 8.8 % here, against tracer 86.4 % at L = 128,
/// where the tracer mode's whole lifetime at `k_index = 2` is shorter than one mean free time.
const L: usize = 256;
/// Density, unchanged from the freeze.
const D: f64 = 0.2;
/// Seeds averaged BEFORE the fit. Measured `R²` on FHP-I at L = 256: 0.9972 / 0.9974 / 0.9999 /
/// 0.9996 on the four readings, with close-SNR 24.5 to 53.6, twice the measured floor of 12.
const SEEDS: usize = 2;
/// Shear amplitude. At 0.10 one seed already clears `R² = 0.99`, where 0.05 needs four.
const A_SHEAR: f64 = 0.10;
/// Plant (ii) is unchanged in substance: the two amplitudes are still 0.05 and 0.10, and it is
/// now the 0.05 reading that must agree with the instrument's 0.10 within 5 %.
const A_SHEAR_PLANT: f64 = 0.05;
/// Colour amplitude. Colour never enters the occupation dynamics, so a colour wave of ANY
/// amplitude is still linear response; measured `D` 5.476 / 5.356 / 5.332 at 0.05 / 0.25 / 1.0,
/// with the amplitude linear to 4 % over that factor of twenty.
const A_COLOUR: f64 = 1.0;
const K1: usize = 1;
const K2: usize = 2;
/// The declared seed. Member `s` of an ensemble runs at `transport::ensemble_seed(SEED, s)`.
const SEED: u64 = 0x464c_5549_4430;
/// The kinetic clock, `τ = 2D/v²` with `v = 1` link per step and the campaign's own measured
/// tracer `D ≈ 5.7`: the mean free time in the tier's own units.
const TAU_STEPS: f64 = 11.4;
/// THE WINDOW-START RULE: open at this many `τ`. Measured flat from `1τ` to `16τ` — shear
/// 0.5444 to 0.5442, tracer 4.770 to 4.776 — so four is four times clear of the transient and
/// four times clear of the mode.
const TRANSIENT_TAUS: f64 = 4.0;
/// THE WINDOW-END RULE, first half: close at this fraction of the amplitude where it opened.
const WINDOW_END_FRACTION_A: f64 = 0.2;
/// THE WINDOW-END RULE, second half: or at this multiple of the ensemble noise `σ/√S`,
/// whichever is HIGHER. Measured: `R²` fell under 0.99 at close-SNR 8.2 and held above it at
/// every close-SNR from 11.9 up.
const NOISE_FLOOR_SNR: f64 = 12.0;
/// The equilibrium noise `σ`, measured as the sd ACROSS 64 seeds at step 100 with the wave
/// amplitude set to zero: `(L, shear, colour)`. A time RMS would be one sample, because the
/// mode's own correlation time is longer than any window.
const SIGMA_TABLE: [(usize, f64, f64); 3] =
    [(64, 33.9905, 21.7525), (128, 72.3969, 45.4090), (256, 114.4808, 89.8527)];
/// Step caps by `k_index`, shear then colour. A run reaching its cap without a fit is a
/// REFUSAL by name, not a zero. Measured: 27 of 48 sampled laws refuse here, and some do not
/// relax even at twice this cap, so they refuse at any cap.
const CAP_SHEAR: [usize; 2] = [12_000, 3_000];
const CAP_COLOUR: [usize; 2] = [1_500, 400];
/// The census.
const N_LAWS: usize = 4_608;
/// Plant (i)'s stake: `D` must move by more than this fraction.
const PLANT_I_MIN_CHANGE: f64 = 0.20;
/// Plant (ii)'s stake: `ν` must agree within this fraction.
const PLANT_II_MAX_CHANGE: f64 = 0.05;
/// Both plants' carriers, asserted nonzero in the sector each acts on.
const CARRIER_FLOOR: f64 = 0.05;
/// The price gate's band, and the laws it is measured on.
const PRICE_LOW: f64 = 0.1;
const PRICE_HIGH: f64 = 10.0;
const PRICE_LAWS: usize = 10;
/// An EVEN stride aliases with the enumeration's odometer — the last fibers vary fastest, so
/// the first ten laws are ten consecutive odometer states with every slow fiber at its identity
/// permutation. 2411 is coprime to 4608 = 2⁹·3², so this stride walks the whole group; the
/// price is reported on BOTH samples and the freeze's first-ten is the gated one.
const PRICE_STRIDE: usize = 2411;
/// Workers for the census, one whole law each. Cores 24-31 only: `taskset -c 24-31` is the
/// binding constraint, never a core count typed here (M-DEVICE-CLASS).
const DEFAULT_THREADS: usize = 8;
/// The 48-law preview's refusal count at ONE seed, from `fluid0_probe 8 3`. Gate C reports
/// the census's refusal rate against it: a census differing by more than a factor of two is
/// read as an instrument change, not a finding (M-BASE-RATE-OMITTED).
const PREVIEW_REFUSALS: (usize, usize) = (27, 48);
/// Water's ratio, the experimental kill of §1: `ν = 1.00e-2 cm²/s` (Kestin, Sokolov & Wakeham
/// 1978), `D = 2.3e-5 cm²/s` (Krynicki, Green & Sawyer 1978; Mills 1973). TYPED FROM EXPERIMENT.
const SC_WATER: f64 = 434.8;
/// S's bands.
const SC_BRANCH_A: f64 = 100.0;
const SC_BRANCH_B: f64 = 10.0;
// ═══════════════════════════ END OF THE DECLARED BLOCK ════════════════════════════════════

/// The window's opening step, from the clock and the rule — never typed.
fn transient_steps() -> usize {
    (TRANSIENT_TAUS * TAU_STEPS).ceil() as usize
}

/// The measured equilibrium `σ` for this lattice size and readout.
fn sigma(l: usize, colour: bool) -> f64 {
    SIGMA_TABLE
        .iter()
        .find(|(ll, _, _)| *ll == l)
        .map(|(_, s, c)| if colour { *c } else { *s })
        .unwrap_or_else(|| panic!("no measured sigma for L={l}: measure it before using it"))
}

/// The step cap for a readout at a wavevector.
fn cap(colour: bool, k_index: usize) -> usize {
    let t = if colour { CAP_COLOUR } else { CAP_SHEAR };
    t[k_index - 1]
}

/// Set A's fit rule, assembled from the declared block and nowhere else.
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

fn audit_json(a: &LedgerAudit) -> String {
    format!(
        "{{\"mass_exact\": {}, \"px_exact\": {}, \"py_exact\": {}, \"red_exact\": {}, \
         \"steps_checked\": {}, \"mass\": {}, \"px\": {}, \"py\": {}, \"red\": {}}}",
        jb(a.mass_exact),
        jb(a.px_exact),
        jb(a.py_exact),
        jb(a.red_exact),
        a.steps_checked,
        a.mass,
        a.px,
        a.py,
        a.red
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
    fn gate(&mut self, id: &str, pass: bool, work: u64, detail: String) {
        if work == 0 {
            println!("  {id:<6} VOID (zero work)  {detail}");
            self.failures.push(format!("{id}: ZERO WORK"));
            self.entries.push(format!(
                "\"{id}\": {{\"verdict\": \"VOID\", \"work\": 0, \"detail\": {:?}}}",
                detail
            ));
            return;
        }
        let v = if pass { "PASS" } else { "FAIL" };
        println!("  {id:<6} {v}  [{work} checks]  {detail}");
        if !pass {
            self.failures.push(format!("{id}: {detail}"));
        }
        self.entries.push(format!(
            "\"{id}\": {{\"verdict\": \"{v}\", \"work\": {work}, \"detail\": {:?}}}",
            detail
        ));
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

// ---------------------------------------------------------------- one law, four runs
struct LawRuns {
    nu1: TransportReading,
    nu2: TransportReading,
    d1: TransportReading,
    d2: TransportReading,
    audits: [LedgerAudit; 4],
}

/// One shear reading under set A.
fn shear(law: &[u8], model: &Model, k_index: usize, a: f64, along_columns: bool, drive: [f64; 2])
    -> (TransportReading, LedgerAudit) {
    shear_ensemble(
        model, law, L, D, a, k_index, cap(false, k_index), SEED, SEEDS, along_columns, drive,
        &rule(L, false, SEEDS),
    )
}

/// One tracer reading under set A.
fn tracer(law: &[u8], model: &Model, k_index: usize, colour_rule: ColourRule)
    -> (TransportReading, LedgerAudit) {
    colour_ensemble(
        model, law, L, D, A_COLOUR, k_index, cap(true, k_index), SEED, SEEDS, colour_rule,
        &rule(L, true, SEEDS),
    )
}

fn run_law(model: &Model, law: &[u8]) -> LawRuns {
    let (nu1, a1) = shear(law, model, K1, A_SHEAR, false, transport::row_drive());
    let (nu2, a2) = shear(law, model, K2, A_SHEAR, false, transport::row_drive());
    let (d1, a3) = tracer(law, model, K1, ColourRule::Blind);
    let (d2, a4) = tracer(law, model, K2, ColourRule::Blind);
    LawRuns { nu1, nu2, d1, d2, audits: [a1, a2, a3, a4] }
}

/// A law's transport status: the exit that speaks for the pair of wavevectors.
fn pair_status(a: &TransportReading, b: &TransportReading) -> Exit {
    match (a.exit, b.exit) {
        (Exit::Fitted, Exit::Fitted) => {
            if within(a.value, b.value, K_AGREEMENT_TOLERANCE) {
                Exit::Fitted
            } else {
                Exit::PreHydrodynamic
            }
        }
        (Exit::Fitted, e) | (e, _) => e,
    }
}

fn census_line(index: usize, law: &[u8], model: &Model, runs: &LawRuns, seconds: f64) -> String {
    let is_fhp_i = law == model.fhp_i(true).as_slice() || law == model.fhp_i(false).as_slice();
    let is_identity = law == model.identity_collision().as_slice();
    let nu_status = pair_status(&runs.nu1, &runs.nu2);
    let d_status = pair_status(&runs.d1, &runs.d2);
    let sc = match (nu_status, d_status, runs.nu1.value, runs.d1.value) {
        (Exit::Fitted, Exit::Fitted, Some(n), Some(dd)) if dd != 0.0 => Some(n / dd),
        _ => None,
    };
    let fired: u64 = runs.nu1.collisions_fired
        + runs.nu2.collisions_fired
        + runs.d1.collisions_fired
        + runs.d2.collisions_fired;
    let ledger_exact = runs.audits.iter().all(|a| a.all_exact());
    format!(
        "{{\"index\": {index}, \"digest\": \"{:016x}\", \"is_fhp_i\": {}, \"is_identity\": {}, \
         \"seeds\": {SEEDS}, \"l\": {L}, \
         \"nu_k1\": {}, \"nu_k2\": {}, \"d_k1\": {}, \"d_k2\": {}, \
         \"nu_k1_exit\": \"{}\", \"nu_k2_exit\": \"{}\", \"d_k1_exit\": \"{}\", \"d_k2_exit\": \"{}\", \
         \"nu_status\": \"{}\", \"d_status\": \"{}\", \
         \"nu_k1_r2\": {}, \"nu_k2_r2\": {}, \"d_k1_r2\": {}, \"d_k2_r2\": {}, \
         \"nu_k1_points\": {}, \"nu_k2_points\": {}, \"d_k1_points\": {}, \"d_k2_points\": {}, \
         \"sc\": {}, \"collisions_fired\": {}, \"ledger_exact\": {}, \"seconds\": {}}}",
        law_digest(law),
        jb(is_fhp_i),
        jb(is_identity),
        jo(runs.nu1.value),
        jo(runs.nu2.value),
        jo(runs.d1.value),
        jo(runs.d2.value),
        runs.nu1.exit.name(),
        runs.nu2.exit.name(),
        runs.d1.exit.name(),
        runs.d2.exit.name(),
        nu_status.name(),
        d_status.name(),
        jf(runs.nu1.r2),
        jf(runs.nu2.r2),
        jf(runs.d1.r2),
        jf(runs.d2.r2),
        runs.nu1.points,
        runs.nu2.points,
        runs.d1.points,
        runs.d2.points,
        jo(sc),
        fired,
        jb(ledger_exact),
        jf(seconds)
    )
}

/// The declared block, as JSON. Written into every record so a reading can never be read
/// without the instrument that made it.
fn params_json() -> String {
    format!(
        "{{\"amendment\": \"FLUID0_AMENDMENT_1 set A\", \"l\": {L}, \"density\": {}, \
         \"seeds\": {SEEDS}, \"a_shear\": {}, \"a_shear_plant\": {}, \"a_colour\": {}, \
         \"k_index\": [{K1}, {K2}], \"k1\": {}, \"k2\": {}, \"seed\": {SEED}, \
         \"tau_steps\": {}, \"tau_rule\": \"tau = 2D/v^2 with v = 1 link per step and the \
         campaign's measured tracer D; the window opens at {TRANSIENT_TAUS} tau\", \
         \"transient_taus\": {}, \"window_start_steps\": {}, \
         \"window_end_fraction\": {}, \"noise_floor_snr\": {}, \
         \"window_end_rule\": \"close at the first step under max(fraction * |M(t0)|, \
         noise_floor_snr * sigma / sqrt(seeds))\", \
         \"sigma_table\": [{}], \"sigma_used_shear\": {}, \"sigma_used_colour\": {}, \
         \"floor_shear\": {}, \"floor_colour\": {}, \
         \"cap_shear\": [{}, {}], \"cap_colour\": [{}, {}], \
         \"min_r2\": {}, \"min_points\": {MIN_POINTS}, \
         \"monotone_tolerance_fraction\": {}, \"k_agreement_tolerance\": {}}}",
        jf(D),
        jf(A_SHEAR),
        jf(A_SHEAR_PLANT),
        jf(A_COLOUR),
        jf(wavenumber(L, K1)),
        jf(wavenumber(L, K2)),
        jf(TAU_STEPS),
        jf(TRANSIENT_TAUS),
        transient_steps(),
        jf(WINDOW_END_FRACTION_A),
        jf(NOISE_FLOOR_SNR),
        SIGMA_TABLE
            .iter()
            .map(|(l, s, c)| format!("{{\"l\": {l}, \"shear\": {}, \"colour\": {}}}", jf(*s), jf(*c)))
            .collect::<Vec<_>>()
            .join(", "),
        jf(sigma(L, false)),
        jf(sigma(L, true)),
        jf(rule(L, false, SEEDS).floor),
        jf(rule(L, true, SEEDS).floor),
        CAP_SHEAR[0],
        CAP_SHEAR[1],
        CAP_COLOUR[0],
        CAP_COLOUR[1],
        jf(MIN_R2),
        jf(MONOTONE_TOLERANCE_FRACTION),
        jf(K_AGREEMENT_TOLERANCE)
    )
}

// ---------------------------------------------------------------- main
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let phase = args.get(1).map(String::as_str).unwrap_or("gate");
    let out_dir = args.get(2).cloned().unwrap_or_else(|| "../conformance/mesh/fluid0".to_string());
    let threads: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_THREADS);
    std::fs::create_dir_all(&out_dir).expect("cannot create out_dir");
    println!("=========================================================================");
    println!("FLUID-0 — the census's transport character");
    println!("freeze:  conformance/mesh/FLUID0_PREREG.md + FLUID0_AMENDMENT_1 set A");
    println!("instr:   holon-lattice::transport   phase: {phase}   out_dir: {out_dir}");
    println!(
        "set A:   L={L} d={D} seeds={SEEDS} A_shear={A_SHEAR} A_colour={A_COLOUR} \
         k_index in {{{K1},{K2}}} seed={SEED:#x}"
    );
    println!(
        "         k(1)={:.9e}  k(2)={:.9e}   tau={TAU_STEPS} steps (2D/v^2, measured D)",
        wavenumber(L, K1),
        wavenumber(L, K2)
    );
    println!(
        "window:  opens at {TRANSIENT_TAUS} tau = {} steps; closes at the first step under \
         max({WINDOW_END_FRACTION_A}|M(t0)|, {NOISE_FLOOR_SNR} sigma/sqrt(S))",
        transient_steps()
    );
    println!(
        "         sigma(L={L}) shear {:.4} colour {:.4} -> floors {:.4} / {:.4}",
        sigma(L, false),
        sigma(L, true),
        rule(L, false, SEEDS).floor,
        rule(L, true, SEEDS).floor
    );
    println!(
        "caps:    shear {:?} colour {:?} by k_index; R2 >= {MIN_R2} on >= {MIN_POINTS} points; \
         k-agreement {K_AGREEMENT_TOLERANCE}",
        CAP_SHEAR, CAP_COLOUR
    );
    println!("=========================================================================");
    match phase {
        "gate" => gate(&out_dir),
        "sweep" => sweep(&out_dir, threads),
        "read" => read(&out_dir),
        "diag" => diag(&out_dir),
        other => {
            eprintln!("unknown phase {other:?}; expected gate | sweep | read | diag");
            std::process::exit(2);
        }
    }
}

// ---------------------------------------------------------------- gate
fn gate(out_dir: &str) {
    let m = Model::fhp6();
    let mut r = Report { failures: Vec::new(), entries: Vec::new() };
    let mut audits: Vec<(String, LedgerAudit)> = Vec::new();
    let mut json = String::new();

    // ------------------------------------------------------------------ G0
    println!("\n--- G0: the instrument on FHP-I (both chiralities) ---");
    let mut chir: Vec<(bool, LawRuns)> = Vec::new();
    for chirality in [true, false] {
        let law = m.fhp_i(chirality);
        println!("  FHP-I chirality={chirality}  digest {:016x}", law_digest(&law));
        let t = Instant::now();
        let runs = run_law(&m, &law);
        println!("    {}", show("nu  rows k=1", &runs.nu1));
        println!("    {}", show("nu  rows k=2", &runs.nu2));
        println!("    {}", show("D   colour k=1", &runs.d1));
        println!("    {}", show("D   colour k=2", &runs.d2));
        println!("    ({:.2} s)", t.elapsed().as_secs_f64());
        for (n, a) in runs.audits.iter().enumerate() {
            audits.push((format!("fhp_i[{chirality}] run {n}"), *a));
        }
        chir.push((chirality, runs));
    }
    let nu_b = boltzmann_nu_fhp_i(D);
    let mut g0_detail = String::new();
    let mut g0 = true;
    for (chirality, runs) in &chir {
        let all_fitted =
            [&runs.nu1, &runs.nu2, &runs.d1, &runs.d2].iter().all(|x| x.exit == Exit::Fitted);
        let nu_agree = within(runs.nu1.value, runs.nu2.value, K_AGREEMENT_TOLERANCE);
        let d_agree = within(runs.d1.value, runs.d2.value, K_AGREEMENT_TOLERANCE);
        g0 &= all_fitted && nu_agree && d_agree;
        let _ = write!(
            g0_detail,
            "chir={chirality}: exits {}/{}/{}/{} fitted={} nu {} vs {} agree={} \
             D {} vs {} agree={}; ",
            runs.nu1.exit.name(),
            runs.nu2.exit.name(),
            runs.d1.exit.name(),
            runs.d2.exit.name(),
            all_fitted,
            jo(runs.nu1.value),
            jo(runs.nu2.value),
            nu_agree,
            jo(runs.d1.value),
            jo(runs.d2.value),
            d_agree
        );
    }
    let cross = within(chir[0].1.nu1.value, chir[1].1.nu1.value, 0.10);
    g0 &= cross;
    let _ = write!(g0_detail, "chiralities agree = {cross}");
    println!(
        "\n  credited comparison, NOT a gate: Boltzmann nu_B({D}) = 1/(12 d (1-d)^3) - 1/8 = {:.6e}",
        nu_b
    );
    for (chirality, runs) in &chir {
        println!(
            "    chirality={chirality}: nu(k=1) {} , D(k=1) {} , Sc {} , nu/nu_B = {}   \
             [Frisch/d'Humieres/Hasslacher/Lallemand/Pomeau/Rivet 1987; Henon 1987 — typed \
             from the literature]",
            jo(runs.nu1.value),
            jo(runs.d1.value),
            jo(match (runs.nu1.value, runs.d1.value) {
                (Some(n), Some(dd)) if dd != 0.0 => Some(n / dd),
                _ => None,
            }),
            jo(runs.nu1.value.map(|v| v / nu_b))
        );
    }
    r.gate("G0", g0, 8, g0_detail);

    // ------------------------------------------------------------------ G1
    println!("\n--- G1: the identity law refuses ---");
    let id = m.identity_collision();
    let id_runs = run_law(&m, &id);
    println!("    {}", show("identity nu rows k=1", &id_runs.nu1));
    println!("    {}", show("identity nu rows k=2", &id_runs.nu2));
    println!("    {}", show("identity D colour k=1", &id_runs.d1));
    println!("    {}", show("identity D colour k=2", &id_runs.d2));
    for (n, a) in id_runs.audits.iter().enumerate() {
        audits.push((format!("identity run {n}"), *a));
    }
    let id_fired: u64 = id_runs.nu1.collisions_fired
        + id_runs.nu2.collisions_fired
        + id_runs.d1.collisions_fired
        + id_runs.d2.collisions_fired;
    let id_all = [&id_runs.nu1, &id_runs.nu2, &id_runs.d1, &id_runs.d2];
    // FLUID0_AMENDMENT_1: "REFUSES by name" is ANY refusal exit (`NoDecay`, `LowR2`,
    // `TooFewPoints`) — the identity's colour wave at k=1 exits `LowR2` because it streams
    // without damping and the fit has nothing exponential to hold. The stricter
    // all-four-NoDecay reading is still computed and reported, so the letter's change stays
    // visible in the record rather than being absorbed.
    let g1_strict = id_all.iter().all(|x| x.exit == Exit::NoDecay) && id_fired == 0;
    let g1_substance = id_all.iter().all(|x| x.exit != Exit::Fitted) && id_fired == 0;
    r.gate(
        "G1",
        g1_substance,
        4,
        format!(
            "exits {}/{}/{}/{}; collisions_fired = {id_fired} (the reason: nothing collides); \
             GATED reading (all four exit SOME refusal, no number entered, zero collisions) = \
             {g1_substance}; the freeze's stricter all-four-NoDecay reading = {g1_strict}",
            id_runs.nu1.exit.name(),
            id_runs.nu2.exit.name(),
            id_runs.d1.exit.name(),
            id_runs.d2.exit.name()
        ),
    );

    // ------------------------------------------------------------------ G3
    println!("\n--- G3: isotropy on FHP-I (the column leg) ---");
    let law = m.fhp_i(true);
    let (col, col_audit) = shear(&law, &m, K1, A_SHEAR, true, COLUMN_TRANSVERSE);
    audits.push(("G3 column transverse".to_string(), col_audit));
    println!("    {}", show("nu  columns k=1 (transverse)", &col));
    let (col_y, col_y_audit) = shear(&law, &m, K1, A_SHEAR, true, [0.0, 1.0]);
    audits.push(("G3 column literal-y".to_string(), col_y_audit));
    println!("    {}", show("nu  columns k=1 (literal y)", &col_y));
    println!(
        "      the literal-y drive is 25 % longitudinal at this leg's wavevector \
         (sqrt3/2, -1/2); reported, never gated"
    );
    let row = &chir[0].1.nu1;
    let g3 = within(col.value, row.value, 0.10);
    r.gate(
        "G3",
        g3,
        1,
        format!(
            "columns {} vs rows {} -> relative {}",
            jo(col.value),
            jo(row.value),
            match (col.value, row.value) {
                (Some(a), Some(b)) => format!("{:.6}", rel(a, b)),
                _ => "undefined (a leg refused)".to_string(),
            }
        ),
    );

    // ------------------------------------------------------------------ plants
    println!("\n--- P(i): the colour-blind rule removed ---");
    let (d_ordinal, ord_audit) = tracer(&law, &m, K1, ColourRule::Ordinal);
    audits.push(("plant (i) ordinal".to_string(), ord_audit));
    println!("    {}", show("D  colour k=1 ORDINAL plant", &d_ordinal));
    let d_carrier = chir[0].1.d1.value;
    let carrier_i_ok = matches!(d_carrier, Some(v) if v >= CARRIER_FLOOR);
    let plant_i_change = match (d_carrier, d_ordinal.value) {
        (Some(a), Some(b)) => Some(rel(a, b)),
        _ => None,
    };
    // FLUID0_AMENDMENT_1: a plant whose readout REFUSES where the blind rule FITTED fires.
    // Two prongs, and the record names which one fired (a two-prong gate that does not say
    // which prong fired is a gate that cannot be audited).
    let blind_fitted = chir[0].1.d1.exit == Exit::Fitted;
    let fires_by_value = matches!(plant_i_change, Some(c) if c > PLANT_I_MIN_CHANGE);
    let fires_by_refusal = blind_fitted && d_ordinal.exit != Exit::Fitted;
    let plant_i = carrier_i_ok && (fires_by_value || fires_by_refusal);
    let prong = match (fires_by_value, fires_by_refusal) {
        (true, true) => "BOTH prongs: the value moved past the stake AND the plant's reading refuses where the blind rule fitted",
        (true, false) => "the VALUE prong: the plant's own reading fitted and moved past the stake",
        (false, true) => "the REFUSAL prong: the plant's reading refuses where the blind rule fitted",
        (false, false) => "NEITHER prong fired",
    };
    // The stake is on the VALUE, and the value exists only when the plant's own reading is
    // Fitted. When the plant moves the physics so hard that its trace stops being one
    // exponential, the decay RATE is still there and is reported beside — never gated on,
    // because a gate that reads a refused fit is reading noise.
    let gamma_ratio = if chir[0].1.d1.gamma.is_finite() && d_ordinal.gamma.is_finite() {
        Some(d_ordinal.gamma / chir[0].1.d1.gamma)
    } else {
        None
    };
    r.gate(
        "P(i)",
        plant_i,
        1,
        format!(
            "carrier D_FHP-I(k=1) = {} (floor {CARRIER_FLOOR}, asserted {carrier_i_ok}); \
             blind {} vs ordinal {} -> change {} (stake > {PLANT_I_MIN_CHANGE}); \
             FIRED BY {prong}; blind exit {} (fitted = {blind_fitted}), plant exit {}, decay \
             rate {} against the blind rule's {}, a factor {}",
            jo(d_carrier),
            jo(d_carrier),
            jo(d_ordinal.value),
            match plant_i_change {
                Some(c) => format!("{:.6}", c),
                None => "undefined (the plant's own reading refused)".to_string(),
            },
            chir[0].1.d1.exit.name(),
            d_ordinal.exit.name(),
            jf(d_ordinal.gamma),
            jf(chir[0].1.d1.gamma),
            match gamma_ratio {
                Some(v) => format!("{:.4}", v),
                None => "undefined".to_string(),
            }
        ),
    );

    println!("\n--- P(ii): linearity, the OTHER shear amplitude ---");
    let (nu_alt, alt_audit) =
        shear(&law, &m, K1, A_SHEAR_PLANT, false, transport::row_drive());
    audits.push((format!("plant (ii) A={A_SHEAR_PLANT}"), alt_audit));
    println!("    {}", show(&format!("nu  rows k=1 A={A_SHEAR_PLANT}"), &nu_alt));
    let nu_carrier = row.value;
    let carrier_ii_ok = matches!(nu_carrier, Some(v) if v >= CARRIER_FLOOR);
    let plant_ii_change = match (nu_carrier, nu_alt.value) {
        (Some(a), Some(b)) => Some(rel(a, b)),
        _ => None,
    };
    let plant_ii = carrier_ii_ok && matches!(plant_ii_change, Some(c) if c <= PLANT_II_MAX_CHANGE);
    r.gate(
        "P(ii)",
        plant_ii,
        1,
        format!(
            "carrier nu_FHP-I(k=1) (floor {CARRIER_FLOOR}, asserted {carrier_ii_ok}); \
             A={A_SHEAR} {} vs A={A_SHEAR_PLANT} {} -> change {} (stake <= {PLANT_II_MAX_CHANGE})",
            jo(nu_carrier),
            jo(nu_alt.value),
            match plant_ii_change {
                Some(c) => format!("{:.6}", c),
                None => "undefined (a leg refused)".to_string(),
            }
        ),
    );

    // ------------------------------------------------------------------ G2
    println!("\n--- G2: the ledger, exact in every run above ---");
    let mut g2 = true;
    let mut steps_checked = 0u64;
    for (tag, a) in &audits {
        g2 &= a.all_exact();
        steps_checked += a.steps_checked;
        if !a.all_exact() {
            println!("    LEAK  {tag}: {}", audit_json(a));
        }
    }
    println!(
        "    {} runs ({SEEDS} members each, every member against its own initial ledger), \
         {} step-checks, mass/Px/Py/red integer-identical at every step: {}",
        audits.len(),
        steps_checked,
        g2
    );
    r.gate(
        "G2",
        g2,
        steps_checked,
        format!("{} runs audited at every step, all four integers exact = {}", audits.len(), g2),
    );

    // ------------------------------------------------------------------ P
    println!("\n--- P: the price ---");
    let laws = m.collision_laws();
    assert_eq!(laws.len(), N_LAWS, "the enumeration is not {N_LAWS} laws");
    let mut price_lines = Vec::new();
    let mut prices = Vec::new();
    // FLUID0_AMENDMENT_1: the GATED sample is the coprime stride, because an even stride —
    // and the first ten laws are the evenest of all — aliases with the enumeration's odometer
    // and samples one corner of the group. The freeze's original first-ten reading is kept
    // beside it, named as the biased one, so the change is visible in the record.
    for (tag, idxs) in [
        (
            "coprime-stride ten (GATED — the amended sample)",
            (0..PRICE_LAWS).map(|i| (i * PRICE_STRIDE) % N_LAWS).collect::<Vec<_>>(),
        ),
        (
            "first-ten (the freeze's original sample, BIASED, reported not gated)",
            (0..PRICE_LAWS).collect::<Vec<_>>(),
        ),
    ] {
        println!("  {tag}:");
        let t0 = Instant::now();
        for &i in &idxs {
            let t = Instant::now();
            let runs = run_law(&m, &laws[i]);
            let s = t.elapsed().as_secs_f64();
            println!(
                "    law {i:<5} digest {:016x}  nu {}/{}  D {}/{}  {:.3} s",
                law_digest(&laws[i]),
                runs.nu1.exit.name(),
                runs.nu2.exit.name(),
                runs.d1.exit.name(),
                runs.d2.exit.name(),
                s
            );
            price_lines.push(format!(
                "{{\"sample\": {:?}, \"index\": {i}, \"digest\": \"{:016x}\", \"seconds\": {}}}",
                tag,
                law_digest(&laws[i]),
                jf(s)
            ));
        }
        let p = t0.elapsed().as_secs_f64() / PRICE_LAWS as f64;
        println!(
            "    price {p:.4} s/law -> census {:.0} s single core = {:.2} h; on 8 cores {:.2} h",
            p * N_LAWS as f64,
            p * N_LAWS as f64 / 3600.0,
            p * N_LAWS as f64 / 3600.0 / 8.0
        );
        prices.push((tag, p));
    }
    let price = prices[0].1;
    let projected = price * N_LAWS as f64;
    let price_json = format!(
        "{{\n  \"params\": {},\n  \"price_sample\": \"coprime-stride ten\",\n  \
         \"price_stride\": {PRICE_STRIDE},\n  \"price_seconds_per_law\": {},\n  \
         \"price_first_ten_biased\": {},\n  \"laws_measured\": {PRICE_LAWS},\n  \
         \"n_laws\": {N_LAWS},\n  \"projected_total_seconds\": {},\n  \
         \"band_low_seconds\": {},\n  \"band_high_seconds\": {},\n  \
         \"note\": \"FLUID0_AMENDMENT_1 gates the price on ten laws at the coprime stride \
         {PRICE_STRIDE}; the freeze's first ten are ten consecutive odometer states with every \
         slow fiber at its identity permutation, so they are the weakest laws in the group and \
         the dearest to run, and that reading is kept beside as price_first_ten_biased\",\n  \
         \"per_law\": [\n    {}\n  ]\n}}\n",
        params_json(),
        jf(price),
        jf(prices[1].1),
        jf(projected),
        jf(PRICE_LOW * projected),
        jf(PRICE_HIGH * projected),
        price_lines.join(",\n    ")
    );
    std::fs::write(format!("{out_dir}/price.json"), &price_json).expect("write price.json");
    println!("    wrote {out_dir}/price.json  (before any census line)");
    r.gate(
        "P",
        price > 0.0,
        2 * PRICE_LAWS as u64,
        format!(
            "{price:.6} s/law on the GATED coprime-stride ten (stride {PRICE_STRIDE}); \
             {:.6} s/law on the freeze's first ten, kept beside as the BIASED sample",
            prices[1].1
        ),
    );

    // ------------------------------------------------------------------ gate.json
    let _ = write!(json, "{{\n  \"phase\": \"gate\",\n  \"params\": {},\n", params_json());
    let _ = write!(json, "  \"verdicts\": {{{}}},\n", r.entries.join(", "));
    let _ = write!(
        json,
        "  \"fhp_i\": [\n    {}\n  ],\n",
        chir.iter()
            .map(|(c, runs)| format!(
                "{{\"chirality\": {}, \"digest\": \"{:016x}\", \"nu_k1\": {}, \"nu_k2\": {}, \
                 \"d_k1\": {}, \"d_k2\": {}, \"sc\": {}}}",
                jb(*c),
                law_digest(&m.fhp_i(*c)),
                reading_json(&runs.nu1),
                reading_json(&runs.nu2),
                reading_json(&runs.d1),
                reading_json(&runs.d2),
                jo(match (runs.nu1.value, runs.d1.value) {
                    (Some(n), Some(dd)) if dd != 0.0 => Some(n / dd),
                    _ => None,
                })
            ))
            .collect::<Vec<_>>()
            .join(",\n    ")
    );
    let _ = write!(
        json,
        "  \"boltzmann\": {{\"note\": \"credited comparison, not a gate\", \
         \"nu_b\": {}, \"ratio_chirality_true\": {}, \"ratio_chirality_false\": {}}},\n",
        jf(nu_b),
        jo(chir[0].1.nu1.value.map(|v| v / nu_b)),
        jo(chir[1].1.nu1.value.map(|v| v / nu_b))
    );
    let _ = write!(
        json,
        "  \"identity\": {{\"nu_k1\": {}, \"nu_k2\": {}, \"d_k1\": {}, \"d_k2\": {}, \
         \"collisions_fired\": {}, \"gated_all_refuse\": {}, \"stricter_all_nodecay\": {}}},\n",
        reading_json(&id_runs.nu1),
        reading_json(&id_runs.nu2),
        reading_json(&id_runs.d1),
        reading_json(&id_runs.d2),
        id_fired,
        jb(g1_substance),
        jb(g1_strict)
    );
    let _ = write!(
        json,
        "  \"isotropy\": {{\"columns_transverse\": {}, \"columns_literal_y\": {}, \"rows\": {}}},\n",
        reading_json(&col),
        reading_json(&col_y),
        reading_json(row)
    );
    let _ = write!(
        json,
        "  \"plants\": {{\"i\": {{\"carrier\": {}, \"carrier_floor\": {}, \"blind\": {}, \
         \"ordinal\": {}, \"change\": {}, \"stake\": {}, \"gamma_ratio\": {}, \"fires_by_value\": {}, \"fires_by_refusal\": {}, \
         \"firing_prong\": {:?}}}, \"ii\": {{\"carrier\": {}, \
         \"carrier_floor\": {}, \"a_instrument\": {}, \"a_plant\": {}, \"change\": {}, \
         \"stake\": {}}}}},\n",
        jo(d_carrier),
        jf(CARRIER_FLOOR),
        reading_json(&chir[0].1.d1),
        reading_json(&d_ordinal),
        jo(plant_i_change),
        jf(PLANT_I_MIN_CHANGE),
        jo(gamma_ratio),
        jb(fires_by_value),
        jb(fires_by_refusal),
        prong,
        jo(nu_carrier),
        jf(CARRIER_FLOOR),
        reading_json(row),
        reading_json(&nu_alt),
        jo(plant_ii_change),
        jf(PLANT_II_MAX_CHANGE)
    );
    let _ = write!(
        json,
        "  \"ledger\": {{\"runs\": {}, \"members_per_run\": {SEEDS}, \"step_checks\": {}, \
         \"all_exact\": {}, \"audits\": [\n    {}\n  ]}},\n",
        audits.len(),
        steps_checked,
        jb(g2),
        audits
            .iter()
            .map(|(t, a)| format!("{{\"run\": {:?}, \"audit\": {}}}", t, audit_json(a)))
            .collect::<Vec<_>>()
            .join(",\n    ")
    );
    let _ = write!(
        json,
        "  \"price\": {{\"sample\": \"coprime-stride ten\", \"stride\": {PRICE_STRIDE}, \
         \"seconds_per_law\": {}, \"projected_total_seconds\": {}, \
         \"projected_hours_8_cores\": {}, \"price_first_ten_biased\": {}}},\n",
        jf(price),
        jf(projected),
        jf(projected / 3600.0 / 8.0),
        jf(prices[1].1)
    );
    let _ = write!(
        json,
        "  \"failures\": [{}]\n}}\n",
        r.failures.iter().map(|f| format!("{f:?}")).collect::<Vec<_>>().join(", ")
    );
    std::fs::write(format!("{out_dir}/gate.json"), &json).expect("write gate.json");
    println!("\n  wrote {out_dir}/gate.json");

    println!("\n=========================================================================");
    if r.failures.is_empty() {
        println!("GATE VERDICT: all gates PASS under FLUID0_AMENDMENT_1 set A.");
    } else {
        println!("GATE VERDICT: {} gate(s) FAILED — reported, not repaired:", r.failures.len());
        for f in &r.failures {
            println!("  - {f}");
        }
    }
    println!("=========================================================================");
}

// ---------------------------------------------------------------- sweep
/// The census: every law of the enumeration, one line each, RESUMABLE and PARALLEL across
/// laws. Each worker runs one whole law single-threaded, so the per-law `seconds` a line
/// carries stays a single-core number and the price gate keeps meaning what it meant.
fn sweep(out_dir: &str, threads: usize) {
    let m = Model::fhp6();
    let laws = m.collision_laws();
    assert_eq!(laws.len(), N_LAWS);
    let path = format!("{out_dir}/census.jsonl");
    let done: std::collections::HashSet<usize> = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        // A line that does not close is a line a kill truncated: it is NOT done, and the law
        // is re-run. `read` drops it the same way, so a torn tail costs one law, never a hole.
        .filter(|l| l.trim_end().ends_with('}'))
        .filter_map(|l| field(l, "index").and_then(|v| v.trim().parse::<usize>().ok()))
        .collect();
    let todo: Vec<usize> = (0..N_LAWS).filter(|i| !done.contains(i)).collect();
    println!(
        "  resuming: {} of {N_LAWS} laws already present in {path}; {} to run on {threads} workers",
        done.len(),
        todo.len()
    );
    let f = std::sync::Mutex::new(
        std::fs::OpenOptions::new().create(true).append(true).open(&path).expect("open census.jsonl"),
    );
    let t0 = Instant::now();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let written = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let n = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if n >= todo.len() {
                    break;
                }
                let i = todo[n];
                let t = Instant::now();
                let runs = run_law(&m, &laws[i]);
                let line = census_line(i, &laws[i], &m, &runs, t.elapsed().as_secs_f64());
                {
                    let mut h = f.lock().unwrap();
                    writeln!(h, "{line}").expect("append census line");
                    h.flush().expect("flush census line");
                }
                let w = written.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                if w % 100 == 0 {
                    let e = t0.elapsed().as_secs_f64();
                    println!(
                        "  progress: {w} written ({} of {N_LAWS} present), {:.1} s wall, \
                         {:.2} s/law wall, {:.1} % done",
                        done.len() + w,
                        e,
                        e / w as f64,
                        100.0 * (done.len() + w) as f64 / N_LAWS as f64
                    );
                    use std::io::Write as _;
                    let _ = std::io::stdout().flush();
                }
            });
        }
    });
    let written = written.load(std::sync::atomic::Ordering::Relaxed);
    let wall = t0.elapsed().as_secs_f64();
    // The price is a statement about the CENSUS, not about one resumed segment or one worker:
    // the total is the sum of the per-law seconds every line carries, so a sweep finished in
    // three pieces on eight workers is priced the same as one finished in one on one
    // (M-CHEAPER-THAN-ITS-PRICE).
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let census_lines: Vec<&str> = text.lines().filter(|l| l.trim_end().ends_with('}')).collect();
    // `+ 0.0` normalises Rust's `-0.0` empty sum, which would print a signed zero.
    let total: f64 = census_lines.iter().filter_map(|l| fnum(l, "seconds")).sum::<f64>() + 0.0;
    let price = std::fs::read_to_string(format!("{out_dir}/price.json"))
        .ok()
        .and_then(|s| field(&s, "price_seconds_per_law").and_then(|v| v.trim().parse::<f64>().ok()));
    let projected = price.map(|p| p * N_LAWS as f64);
    let in_band = match projected {
        Some(p) => total >= PRICE_LOW * p && total <= PRICE_HIGH * p,
        None => false,
    };
    let mut ids: Vec<&str> = census_lines.iter().filter_map(|l| field(l, "index")).collect();
    ids.sort_unstable();
    ids.dedup();
    println!(
        "  census complete: {written} laws written this run on {threads} workers, {} lines \
         present ({} distinct indices), {:.1} s wall this run, {:.1} s summed single-core over \
         all census lines; price check {}",
        census_lines.len(),
        ids.len(),
        wall,
        total,
        match projected {
            Some(p) => format!(
                "projected {:.1} s, band {:.1}..{:.1} s, in band = {in_band}",
                p,
                PRICE_LOW * p,
                PRICE_HIGH * p
            ),
            None => "NO price.json — the price gate cannot be read".to_string(),
        }
    );
    let done_json = format!(
        "{{\n  \"phase\": \"sweep\",\n  \"params\": {},\n  \"n_laws\": {N_LAWS},\n  \
         \"threads\": {threads},\n  \"written_this_run\": {written},\n  \
         \"lines_present\": {},\n  \"distinct_indices\": {},\n  \
         \"complete\": {},\n  \"wall_seconds_this_run\": {},\n  \"total_seconds\": {},\n  \
         \"price_seconds_per_law\": {},\n  \"projected_total_seconds\": {},\n  \
         \"band_low_seconds\": {},\n  \"band_high_seconds\": {},\n  \
         \"price_check_in_band\": {}\n}}\n",
        params_json(),
        census_lines.len(),
        ids.len(),
        jb(ids.len() == N_LAWS),
        jf(wall),
        jf(total),
        jo(price),
        jo(projected),
        jo(projected.map(|p| PRICE_LOW * p)),
        jo(projected.map(|p| PRICE_HIGH * p)),
        jb(in_band)
    );
    std::fs::write(format!("{out_dir}/sweep.done"), done_json).expect("write sweep.done");
    println!("  wrote {out_dir}/sweep.done");
}

// ---------------------------------------------------------------- read
fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{key}\":");
    let i = line.find(&pat)? + pat.len();
    let rest = &line[i..];
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    Some(rest[..end].trim().trim_matches('"'))
}

fn fnum(line: &str, key: &str) -> Option<f64> {
    match field(line, key) {
        Some("null") | None => None,
        Some(v) => v.parse::<f64>().ok(),
    }
}

fn stats(mut v: Vec<f64>) -> (f64, f64, f64, usize) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n == 0 {
        return (f64::NAN, f64::NAN, f64::NAN, 0);
    }
    let med = if n % 2 == 1 { v[n / 2] } else { 0.5 * (v[n / 2 - 1] + v[n / 2]) };
    (v[0], med, v[n - 1], n)
}

fn read(out_dir: &str) {
    let path = format!("{out_dir}/census.jsonl");
    let text = std::fs::read_to_string(&path).expect("read census.jsonl");
    let all = text.lines().filter(|l| !l.trim().is_empty()).count();
    let lines: Vec<&str> = text.lines().filter(|l| l.trim_end().ends_with('}')).collect();
    println!("  {} census lines in {path} ({} torn and dropped)", lines.len(), all - lines.len());
    let mut ids: Vec<&str> = lines.iter().filter_map(|l| field(l, "index")).collect();
    ids.sort_unstable();
    ids.dedup();
    println!("  {} distinct law indices", ids.len());
    let complete = ids.len() == N_LAWS && lines.len() == N_LAWS;
    if !complete {
        println!("  INCOMPLETE: gate C wants all {N_LAWS} lines; the read below is partial.");
    }

    let mut exit_counts: std::collections::BTreeMap<String, usize> = Default::default();
    for l in &lines {
        for key in ["nu_k1_exit", "nu_k2_exit", "d_k1_exit", "d_k2_exit", "nu_status", "d_status"] {
            if let Some(e) = field(l, key) {
                *exit_counts.entry(format!("{key}={e}")).or_default() += 1;
            }
        }
    }
    println!("\n  exit counts (M-BASE-RATE-OMITTED: the refusal rate beside the distribution):");
    for (k, v) in &exit_counts {
        println!("    {k:<34} {v}");
    }

    let fitted: Vec<&&str> = lines
        .iter()
        .filter(|l| field(l, "nu_status") == Some("Fitted") && field(l, "d_status") == Some("Fitted"))
        .collect();
    let pre_hydro = lines
        .iter()
        .filter(|l| {
            field(l, "nu_status") == Some("PreHydrodynamic")
                || field(l, "d_status") == Some("PreHydrodynamic")
        })
        .count();
    let nus: Vec<f64> = fitted.iter().filter_map(|l| fnum(l, "nu_k1")).collect();
    let ds: Vec<f64> = fitted.iter().filter_map(|l| fnum(l, "d_k1")).collect();
    let scs: Vec<f64> = fitted.iter().filter_map(|l| fnum(l, "sc")).collect();
    let (nu_min, nu_med, nu_max, nu_n) = stats(nus);
    let (d_min, d_med, d_max, d_n) = stats(ds);
    let (sc_min, sc_med, sc_max, sc_n) = stats(scs);
    println!(
        "\n  distribution over the {} laws Fitted on BOTH readouts with k-agreement:",
        fitted.len()
    );
    println!("    nu  min {nu_min:.6e}  median {nu_med:.6e}  max {nu_max:.6e}   (n {nu_n})");
    println!("    D   min {d_min:.6e}  median {d_med:.6e}  max {d_max:.6e}   (n {d_n})");
    println!("    Sc  min {sc_min:.6e}  median {sc_med:.6e}  max {sc_max:.6e}   (n {sc_n})");
    println!("    PreHydrodynamic laws: {pre_hydro}");
    println!(
        "    refusal rate: {} of {} readings",
        lines.len() * 4 - 4 * fitted.len(),
        lines.len() * 4
    );
    // Gate C, amended: the census's refusal rate against the preview that priced it.
    let refused_laws = lines.len() - fitted.len();
    let census_rate = refused_laws as f64 / lines.len().max(1) as f64;
    let preview_rate = PREVIEW_REFUSALS.0 as f64 / PREVIEW_REFUSALS.1 as f64;
    let ratio = if preview_rate > 0.0 { census_rate / preview_rate } else { f64::INFINITY };
    let consistent = (0.5..=2.0).contains(&ratio);
    println!(
        "    laws refusing: {refused_laws} of {} = {census_rate:.4}; the 48-law preview at one \
         seed read {} of {} = {preview_rate:.4}; ratio {ratio:.4}, within a factor of two = \
         {consistent}",
        lines.len(),
        PREVIEW_REFUSALS.0,
        PREVIEW_REFUSALS.1
    );
    if !consistent {
        println!(
            "    READ AS AN INSTRUMENT CHANGE, NOT A FINDING: the census's refusal rate is more \
             than a factor of two from its own preview's."
        );
    }

    let fhp_lines: Vec<&&str> =
        lines.iter().filter(|l| field(l, "is_fhp_i") == Some("true")).collect();
    let id_lines: Vec<&&str> =
        lines.iter().filter(|l| field(l, "is_identity") == Some("true")).collect();
    println!("\n  FHP-I lines ({}):", fhp_lines.len());
    for l in &fhp_lines {
        println!("    {l}");
    }
    println!("  identity line(s) ({}):", id_lines.len());
    for l in &id_lines {
        println!("    {l}");
    }

    let best = fitted
        .iter()
        .filter_map(|l| fnum(l, "sc").map(|s| (s, **l)))
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let (branch, branch_text) = match best {
        None => ("void", "no law is Fitted on both readouts: S cannot be read".to_string()),
        Some((s, l)) if s >= SC_BRANCH_A => (
            "a",
            format!(
                "SOME law reaches Sc >= {SC_BRANCH_A}: max Sc = {s:.6e} at index {} digest {} \
                 — a single-species candidate for water's ratio exists",
                field(l, "index").unwrap_or("?"),
                field(l, "digest").unwrap_or("?")
            ),
        ),
        Some((s, l)) if s >= SC_BRANCH_B => (
            "b",
            format!(
                "the largest Sc is in [{SC_BRANCH_B}, {SC_BRANCH_A}): {s:.6e} at index {} digest {}",
                field(l, "index").unwrap_or("?"),
                field(l, "digest").unwrap_or("?")
            ),
        ),
        Some((s, l)) => (
            "c",
            format!(
                "the largest Sc is under {SC_BRANCH_B}: {s:.6e} at index {} digest {} — no \
                 single-species FHP-6 element carries water's two coefficients at once",
                field(l, "index").unwrap_or("?"),
                field(l, "digest").unwrap_or("?")
            ),
        ),
    };
    println!("\n  S branch ({branch}): {branch_text}");
    println!(
        "    water's kill band (experiment, §1): Sc_water ~= {SC_WATER:.1} \
         [Kestin/Sokolov/Wakeham 1978; Krynicki/Green/Sawyer 1978; Mills 1973]"
    );

    let json = format!(
        "{{\n  \"phase\": \"read\",\n  \"params\": {},\n  \"lines\": {},\n  \"complete\": {},\n  \
         \"exit_counts\": {{{}}},\n  \"fitted_both\": {},\n  \"pre_hydrodynamic\": {pre_hydro},\n  \
         \"nu\": {{\"min\": {}, \"median\": {}, \"max\": {}, \"n\": {nu_n}}},\n  \
         \"d\": {{\"min\": {}, \"median\": {}, \"max\": {}, \"n\": {d_n}}},\n  \
         \"sc\": {{\"min\": {}, \"median\": {}, \"max\": {}, \"n\": {sc_n}}},\n  \
         \"refusal\": {{\"laws_refusing\": {refused_laws}, \"rate\": {}, \
         \"preview_refusing\": {}, \"preview_of\": {}, \"preview_rate\": {}, \
         \"ratio\": {}, \"within_factor_two\": {}}},\n  \
         \"sc_water\": {},\n  \"s_branch\": \"{branch}\",\n  \"s_text\": {:?},\n  \
         \"fhp_i_lines\": [{}],\n  \"identity_lines\": [{}]\n}}\n",
        params_json(),
        lines.len(),
        jb(complete),
        exit_counts.iter().map(|(k, v)| format!("{k:?}: {v}")).collect::<Vec<_>>().join(", "),
        fitted.len(),
        jf(nu_min),
        jf(nu_med),
        jf(nu_max),
        jf(d_min),
        jf(d_med),
        jf(d_max),
        jf(sc_min),
        jf(sc_med),
        jf(sc_max),
        jf(census_rate),
        PREVIEW_REFUSALS.0,
        PREVIEW_REFUSALS.1,
        jf(preview_rate),
        jf(ratio),
        jb(consistent),
        jf(SC_WATER),
        branch_text,
        fhp_lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", "),
        id_lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", ")
    );
    std::fs::write(format!("{out_dir}/census_summary.json"), &json).expect("write summary");
    println!("\n  wrote {out_dir}/census_summary.json");
}

// ---------------------------------------------------------------- diag (NOT in the freeze)
/// The FROZEN instrument's own noise floor, measured rather than argued — the run that
/// convicted `FLUID0_PREREG`'s parameters. It keeps ITS OWN constants on purpose: it is the
/// record of what was wrong, so the amendment must not be able to rewrite it.
fn diag(out_dir: &str) {
    const DIAG_L: usize = 64;
    const DIAG_A: f64 = 0.05;
    const DIAG_STEPS: usize = 4_000;
    let m = Model::fhp6();
    let law = m.fhp_i(true);
    println!("\n  DIAGNOSTIC — not a gate, not in the freeze, and at the FROZEN parameters:");
    println!("  L={DIAG_L} A={DIAG_A} steps={DIAG_STEPS}, window opening at L, one seed.");

    let (flat, _, _) =
        shear_series(&m, &law, DIAG_L, D, 0.0, K1, 600, SEED, false, transport::row_drive(), false);
    let noise_rms =
        (flat[DIAG_L..].iter().map(|v| v * v).sum::<f64>() / (flat.len() - DIAG_L) as f64).sqrt();
    let (flat_c, _, _) =
        colour_series(&m, &law, DIAG_L, D, 0.0, K1, 600, SEED, ColourRule::Blind, false);
    let noise_rms_c =
        (flat_c[DIAG_L..].iter().map(|v| v * v).sum::<f64>() / (flat_c.len() - DIAG_L) as f64).sqrt();

    let (sig, fired, _) = shear_series(
        &m, &law, DIAG_L, D, DIAG_A, K1, DIAG_STEPS, SEED, false, transport::row_drive(), false,
    );
    let (sig_c, fired_c, _) =
        colour_series(&m, &law, DIAG_L, D, DIAG_A, K1, DIAG_STEPS, SEED, ColourRule::Blind, false);
    let r_s = fit(&sig, DIAG_L, wavenumber(DIAG_L, K1), fired);
    let r_c = fit(&sig_c, DIAG_L, wavenumber(DIAG_L, K1), fired_c);
    println!("    {}", show("shear  A=0.05 k=1", &r_s));
    println!("    {}", show("colour A=0.05 k=1", &r_c));
    println!(
        "    shear  |M(0)| {:.4}  |M(L)| {:.4}  A=0 RMS {:.4}  -> SNR at L = {:.2}",
        sig[0].abs(),
        sig[DIAG_L].abs(),
        noise_rms,
        sig[DIAG_L].abs() / noise_rms
    );
    println!(
        "    colour |M(0)| {:.4}  |M(L)| {:.4}  A=0 RMS {:.4}  -> SNR at L = {:.2}",
        sig_c[0].abs(),
        sig_c[DIAG_L].abs(),
        noise_rms_c,
        sig_c[DIAG_L].abs() / noise_rms_c
    );
    println!(
        "    the window closed at 0.1|M(L)| = {:.4} (shear) / {:.4} (colour); a threshold under \
         the noise floor is a coin toss, not a decay.",
        0.1 * sig[DIAG_L].abs(),
        0.1 * sig_c[DIAG_L].abs()
    );

    const NS: usize = 256;
    let mut ens_s = vec![0.0f64; 401];
    let mut ens_c = vec![0.0f64; 401];
    for s in 0..NS {
        let seed = transport::ensemble_seed(SEED, s);
        let (a, _, _) = shear_series(
            &m, &law, DIAG_L, D, DIAG_A, K1, 400, seed, false, transport::row_drive(), false,
        );
        let (b, _, _) =
            colour_series(&m, &law, DIAG_L, D, DIAG_A, K1, 400, seed, ColourRule::Blind, false);
        for t in 0..=400 {
            ens_s[t] += a[t] / NS as f64;
            ens_c[t] += b[t] / NS as f64;
        }
    }
    println!("\n    {NS}-seed ensemble mean |M(t)| at the frozen parameters:");
    println!("      t          0       8      16      32      64     128     256     400");
    println!(
        "      shear  {:7.2} {:7.2} {:7.2} {:7.2} {:7.2} {:7.2} {:7.2} {:7.2}",
        ens_s[0].abs(), ens_s[8].abs(), ens_s[16].abs(), ens_s[32].abs(),
        ens_s[64].abs(), ens_s[128].abs(), ens_s[256].abs(), ens_s[400].abs()
    );
    println!(
        "      colour {:7.2} {:7.2} {:7.2} {:7.2} {:7.2} {:7.2} {:7.2} {:7.2}",
        ens_c[0].abs(), ens_c[8].abs(), ens_c[16].abs(), ens_c[32].abs(),
        ens_c[64].abs(), ens_c[128].abs(), ens_c[256].abs(), ens_c[400].abs()
    );
    println!(
        "      the frozen transient is L = {DIAG_L} steps; tau is {TAU_STEPS} steps, so the \
         tracer mode was 99 % gone before the window opened."
    );

    let kk = wavenumber(DIAG_L, K1) * wavenumber(DIAG_L, K1);
    let mut char_lines = Vec::new();
    let (mut nu_char, mut d_char) = (f64::NAN, f64::NAN);
    for (tag, series, w0, w1) in [
        ("shear ", &ens_s, 64usize, 200usize),
        ("shear ", &ens_s, 64, 300),
        ("colour", &ens_c, 4, 20),
        ("colour", &ens_c, 8, 24),
        ("colour", &ens_c, 8, 32),
    ] {
        let (g, r2) = transport::log_slope(series, w0, w1);
        let v = g / kk;
        println!(
            "      {tag} window {w0:>4}..{w1:<4} gamma {g:.6e}  R2 {r2:.6}  value {v:.6e} link^2/step"
        );
        if tag == "shear " && w1 == 300 {
            nu_char = v;
        }
        if tag == "colour" && w0 == 8 && w1 == 24 {
            d_char = v;
        }
        char_lines.push(format!(
            "{{\"readout\": \"{}\", \"window\": [{w0}, {w1}], \"gamma\": {}, \"r2\": {}, \
             \"value\": {}}}",
            tag.trim(),
            jf(g),
            jf(r2),
            jf(v)
        ));
    }
    println!(
        "      -> FHP-I at L={DIAG_L}: nu ~ {nu_char:.4}, D ~ {d_char:.4}, Sc ~ {:.4} \
         (water's {SC_WATER:.0}); nu/nu_B = {:.4}",
        nu_char / d_char,
        nu_char / boltzmann_nu_fhp_i(D)
    );
    let json = format!(
        "{{\n  \"phase\": \"diag\",\n  \"note\": \"NOT in FLUID0_PREREG; the FROZEN \
         instrument's diagnostic, never a gate, kept at its own constants\",\n  \
         \"diag_l\": {DIAG_L},\n  \"diag_a\": {},\n  \"diag_steps\": {DIAG_STEPS},\n  \
         \"shear_noise_rms_a0\": {},\n  \"colour_noise_rms_a0\": {},\n  \"shear\": {},\n  \
         \"colour\": {},\n  \"shear_amp_at_l\": {},\n  \"colour_amp_at_l\": {},\n  \
         \"characterisation\": {{\"seeds\": {NS}, \"nu\": {}, \"d\": {}, \"sc\": {}, \
         \"nu_over_boltzmann\": {}, \"fits\": [\n    {}\n  ]}}\n}}\n",
        jf(DIAG_A),
        jf(noise_rms),
        jf(noise_rms_c),
        reading_json(&r_s),
        reading_json(&r_c),
        jf(sig[DIAG_L].abs()),
        jf(sig_c[DIAG_L].abs()),
        jf(nu_char),
        jf(d_char),
        jf(nu_char / d_char),
        jf(nu_char / boltzmann_nu_fhp_i(D)),
        char_lines.join(",\n    ")
    );
    std::fs::write(format!("{out_dir}/diag.json"), &json).expect("write diag.json");
    println!("\n  wrote {out_dir}/diag.json");
}
