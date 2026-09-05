//! FIELD-3 part D (`conformance/water_observatory/FIELD3_PREREG.md`): the hydrogen bond
//! re-asked from FIELD-2's own bonded starts, under the SEAM law — the closure assignment,
//! the closure surfaces confined within the closure, the field, and the harvested wall.
//!
//! ```text
//! cargo run --release -p holon-render --example field3_hbonds -- [OUT_DIR]
//! cargo run --release -p holon-render --example field3_hbonds -- probe [OUT_DIR]
//! cargo run --release -p holon-render --example field3_hbonds -- smoke <DIR> <A> <B>
//! ```
//!
//! §1's expectation is written FIRST (`expectation.json`), with the EMPTY branch that VOIDs
//! a start before its arms run (M-EMPTY-SECTOR): fewer units than staked, or a binding under
//! `1e-4` hartree in magnitude. Then S3's arms: dimer and cyclic tetramer (the ring), OFF
//! (FIELD-2's bare law, same seed) and SEAM, at 293 K and 150 K, counted by the rung-1 lens.
//! The OFF arms are checked against FIELD-2's banked numbers — M-FIXED-POINT-TRAJECTORY, the
//! freeze's identity check: an OFF arm that is not FIELD-2's arm VOIDs the comparison.
//!
//! The wall's coefficients are READ, never chosen: `wall.json` from the harvest
//! (`examples/field3_harvest.rs`). No harvest, no arms.
use holon_chem::elements::Species;
use holon_render::seam::{water_units, SeamModel};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

// The scenes, seeds and the lens count live in ONE place since FIELD-3 (verbatim move).
#[path = "../tests/common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::*;

/// §1's floor: a binding under this in magnitude is an EMPTY reading and VOIDs the start.
const BINDING_FLOOR: f64 = 1e-4;
/// The separation the binding is read against, per unit index, along x.
const SEPARATION_BOHR: f64 = 40.0;
/// FIELD-2's arms are banked to six decimals on `f` and `nbar` and two on `t_mean`; the
/// identity check is EXACT at the banked precision — half a unit in the last printed place.
const F2_TOL_FRAC: f64 = 5e-7;
const F2_TOL_TEMP: f64 = 5e-3;

fn json_num(t: &str, key: &str) -> f64 {
    t.split(&format!("\"{key}\": ")).nth(1).and_then(|x| x.split(|c| c == ',' || c == '\n' || c == '}').next()).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::NAN)
}

/// One object out of a flat JSON map of objects, by key.
fn json_entry<'a>(t: &'a str, name: &str) -> Option<&'a str> {
    t.split(&format!("\"{name}\": {{")).nth(1).and_then(|x| x.split('}').next())
}

/// FIELD-2's banked arms, wherever this runner's output directory puts them.
fn field2_arms(out: &Path) -> Option<String> {
    let mut cands: Vec<PathBuf> = Vec::new();
    if let Some(p) = out.parent() {
        cands.push(p.join("field2").join("arms.json"));
    }
    cands.push(PathBuf::from("../conformance/water_observatory/field2/arms.json"));
    cands.push(PathBuf::from("conformance/water_observatory/field2/arms.json"));
    cands.into_iter().find_map(|p| fs::read_to_string(p).ok())
}

/// The harvested terms, or the refusal. FIELD-4's `wall4.json` (wall, penetration,
/// dispersion) is preferred when present, else FIELD-3's `wall.json` (wall only). `a == 0.0`
/// is the harvest not having landed — the seam rule with no wall is a legitimate state for
/// gate G-B4 and is NOT these arms.
fn wall_from(out: &Path) -> Result<SeamModel, String> {
    let p4 = out.join("wall4.json");
    let p = if p4.exists() { p4 } else { out.join("wall.json") };
    let t = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    let (a, b) = (json_num(&t, "a"), json_num(&t, "b"));
    if !a.is_finite() || !b.is_finite() {
        return Err(format!("{}: no \"a\"/\"b\" in the file", p.display()));
    }
    if a == 0.0 {
        return Err(format!("{}: a = 0", p.display()));
    }
    let opt = |k: &str| { let v = json_num(&t, k); if v.is_finite() { v } else { 0.0 } };
    Ok(SeamModel { a, b, p: opt("p"), c: opt("c"), c6: opt("c6") })
}

struct Start {
    name: &'static str,
    f2_name: &'static str,
    staked_units: u64,
    box_edge: f64,
    species: Vec<Species>,
    pos: Vec<[f64; 3]>,
}

/// §1: the SEAM law's binding at the start, `E(start) − E(separated)`, with `E = e_field +
/// e_seam` — by gate G-B4 the closure rows contribute exactly nothing across the seam, so
/// those two rows are the whole of it.
struct Expect {
    units: u64,
    hb0: usize,
    e_start: f64,
    e_sep: f64,
    field_part: f64,
    wall_part: f64,
    binding: f64,
    void: bool,
    reason: String,
    e293: String,
    e150: String,
}

fn expectation(st: &Start, model: SeamModel) -> Expect {
    let t0 = Instant::now();
    let mut s = scene(&st.species, &st.pos, st.box_edge, 293.0);
    let hb0 = hbonds_now(&s);
    let t_scene = t0.elapsed().as_secs_f64();
    s.set_field(true, None).expect("open box admits the field");
    let t_field = t0.elapsed().as_secs_f64();
    s.set_seam(Some(model)).expect("no acuity frame");
    let t_seam = t0.elapsed().as_secs_f64();
    s.refresh_pairs();
    let t_pairs = t0.elapsed().as_secs_f64();
    s.compute_forces();
    let t_forces = t0.elapsed().as_secs_f64();
    let units = s.seam_work.units;
    let (field_start, wall_start) = (s.e_field, s.e_seam);

    // Separated: each unit, in its oxygen's index order, translated by 40·k bohr along x.
    // Intra-unit distances are untouched, so the assignment the binding is read against is
    // the start's own and no membership transition is posted by the move.
    let z: Vec<u32> = (0..s.n).map(|i| s.atoms[i].species.z).collect();
    let list = water_units(&s.unit_of, &z);
    let saved: Vec<f64> = (0..s.n).map(|i| s.atoms[i].x).collect();
    for (k, (o, hs)) in list.iter().enumerate() {
        let d = SEPARATION_BOHR * k as f64;
        s.atoms[*o].x += d;
        s.atoms[hs[0]].x += d;
        s.atoms[hs[1]].x += d;
    }
    s.compute_forces();
    let t_sep = t0.elapsed().as_secs_f64();
    let (field_sep, wall_sep) = (s.e_field, s.e_seam);
    for i in 0..s.n {
        s.atoms[i].x = saved[i];
    }
    s.compute_forces();
    eprintln!("  [{}] scene {t_scene:.2}s  set_field {:.2}s  set_seam {:.2}s  refresh_pairs {:.2}s  forces {:.2}s  separated {:.2}s  restore {:.2}s",
        st.name, t_field - t_scene, t_seam - t_field, t_pairs - t_seam, t_forces - t_pairs, t_sep - t_forces, t0.elapsed().as_secs_f64() - t_sep);

    let e_start = field_start + wall_start;
    let e_sep = field_sep + wall_sep;
    let binding = e_start - e_sep;
    // the EMPTY branch, ahead of every other reading
    let short = units < st.staked_units;
    let empty = binding.abs() < BINDING_FLOOR;
    let void = short || empty;
    let reason = if short && empty {
        format!("{} units < {} staked, and |binding| < {BINDING_FLOOR:.0e}", units, st.staked_units)
    } else if short {
        format!("{} units < {} staked", units, st.staked_units)
    } else if empty {
        format!("|binding| {:.3e} < {BINDING_FLOOR:.0e}", binding.abs())
    } else {
        "the start is not empty".to_string()
    };
    let expect = |kt: f64| {
        if void {
            "VOID".to_string()
        } else if binding <= -2.0 * kt {
            "hold".to_string()
        } else if binding > -kt {
            "break".to_string()
        } else {
            "no expectation".to_string()
        }
    };
    Expect {
        units,
        hb0,
        e_start,
        e_sep,
        field_part: field_start - field_sep,
        wall_part: wall_start - wall_sep,
        binding,
        void,
        reason,
        e293: expect(K_B * 293.0),
        e150: expect(K_B * 150.0),
    }
}

fn expect_json(e: &Expect, staked: u64) -> String {
    format!("{{\"staked_units\": {staked}, \"units\": {}, \"hbonds_at_start\": {}, \"e_start\": {:.6e}, \"e_separated\": {:.6e}, \"binding\": {:.6e}, \"field_part\": {:.6e}, \"wall_part\": {:.6e}, \"void\": {}, \"reason\": \"{}\", \"expect_293\": \"{}\", \"expect_150\": \"{}\"}}",
        e.units, e.hb0, e.e_start, e.e_sep, e.binding, e.field_part, e.wall_part, e.void, e.reason, e.e293, e.e150)
}

struct Arm {
    f: f64,
    nbar: f64,
    t_mean: f64,
    e_field_final: f64,
    e_seam_final: f64,
    work_field: f64,
    work_seam: f64,
    field_transitions: u64,
    seam_transitions: u64,
    drift_peak: f64,
    columns_ok: bool,
    momentum_ok: bool,
    units_at_start: u64,
    pairs_dropped: u64,
    triples_dropped: u64,
    /// running totals over every force pass since the seam was switched on — the counts the
    /// vacuity check reads (a last pass can be dissociated; a life cannot hide)
    pairs_dropped_total: u64,
    triples_dropped_total: u64,
    hb0: usize,
    seconds: f64,
}

/// One arm: `seam` is `None` for the OFF arm — the bare law, FIELD-2's arm, field and seam
/// both off — and `Some(model)` for the SEAM arm, field on and the seam rule with the
/// harvested wall. `settle` and `count` are parameters so the smoke mode can run short;
/// the freeze's arms are `SETTLE` and `COUNT`.
fn run_arm(st: &Start, temp: f64, seam: Option<SeamModel>, settle: usize, count: usize) -> Arm {
    let t0 = Instant::now();
    let mut s = scene(&st.species, &st.pos, st.box_edge, temp);
    let hb0 = hbonds_now(&s);
    if let Some(model) = seam {
        s.set_field(true, None).expect("open box admits the field");
        s.set_seam(Some(model)).expect("no acuity frame");
    }
    let units_at_start = s.seam_work.units;
    for _ in 0..settle {
        s.step_frame(1);
    }
    let (mut with, mut total, mut t_sum) = (0usize, 0usize, 0.0f64);
    for _ in 0..count {
        s.step_frame(1);
        let k = hbonds_now(&s);
        if k > 0 {
            with += 1;
        }
        total += k;
        t_sum += s.temperature();
    }
    Arm {
        f: with as f64 / count as f64,
        nbar: total as f64 / count as f64,
        t_mean: t_sum / count as f64,
        e_field_final: s.e_field,
        e_seam_final: s.e_seam,
        work_field: s.work.field,
        work_seam: s.work.seam,
        field_transitions: s.field_work.transitions,
        seam_transitions: s.seam_work.transitions,
        drift_peak: s.drift_peak,
        columns_ok: s.work_columns_ok(),
        momentum_ok: s.momentum_residual() <= s.momentum_bound(),
        units_at_start,
        pairs_dropped: s.seam_work.pairs_dropped,
        triples_dropped: s.seam_work.triples_dropped,
        pairs_dropped_total: s.seam_work.pairs_dropped_total,
        triples_dropped_total: s.seam_work.triples_dropped_total,
        hb0,
        seconds: t0.elapsed().as_secs_f64(),
    }
}

fn arm_json(a: &Arm, extra: &str) -> String {
    format!("{{\"f\": {:.6}, \"nbar\": {:.6}, \"t_mean\": {:.2}, \"e_field_final\": {:.6e}, \"e_seam_final\": {:.6e}, \"work_field\": {:.6e}, \"work_seam\": {:.6e}, \"field_transitions\": {}, \"seam_transitions\": {}, \"drift_peak\": {:.3e}, \"columns_ok\": {}, \"momentum_ok\": {}, \"units_at_start\": {}, \"pairs_dropped\": {}, \"triples_dropped\": {}, \"pairs_dropped_total\": {}, \"triples_dropped_total\": {}, \"hbonds_at_start\": {}, \"seconds\": {:.1}{extra}}}",
        a.f, a.nbar, a.t_mean, a.e_field_final, a.e_seam_final, a.work_field, a.work_seam, a.field_transitions, a.seam_transitions, a.drift_peak, a.columns_ok, a.momentum_ok, a.units_at_start, a.pairs_dropped, a.triples_dropped, a.pairs_dropped_total, a.triples_dropped_total, a.hb0, a.seconds)
}

/// M-FIXED-POINT-TRAJECTORY: the OFF arm must BE FIELD-2's arm. `f`, `nbar` and `t_mean`
/// against the banked entry, at the precision the bank carries.
fn check_field2(a: &Arm, banked: Option<&str>, applicable: bool) -> (bool, String) {
    if !applicable {
        return (false, "not applicable (short frame counts)".to_string());
    }
    let Some(t) = banked else {
        return (false, "no reference (field2/arms.json not found)".to_string());
    };
    let (f, nbar, tm) = (json_num(t, "f"), json_num(t, "nbar"), json_num(t, "t_mean"));
    let ok = (a.f - f).abs() <= F2_TOL_FRAC && (a.nbar - nbar).abs() <= F2_TOL_FRAC && (a.t_mean - tm).abs() <= F2_TOL_TEMP;
    if ok {
        (true, format!("match (f {f:.6}, nbar {nbar:.6}, T {tm:.2})"))
    } else {
        (false, format!("MISMATCH: banked f {f:.6} nbar {nbar:.6} T {tm:.2}; ran f {:.6} nbar {:.6} T {:.2}", a.f, a.nbar, a.t_mean))
    }
}

/// The dynamics' own diagnostic (S3 branch (c)): the dimer's first 400 frames under the
/// seam law, mirroring FIELD-2's probe with the seam's counters beside the rows.
fn probe(model: SeamModel) {
    let (sp, pos) = dimer_positions();
    let mut s = scene(&sp, &pos, 30.0, 293.0);
    s.set_field(true, None).expect("open box admits the field");
    s.set_seam(Some(model)).expect("no acuity frame");
    eprintln!("-- dimer SEAM (a {:.6e}, b {:.6}): dt {:.3}, n {}", model.a, model.b, s.dt(), s.n);
    for k in 0..=400 {
        if k % 25 == 0 {
            let oo = ((s.atoms[0].x - s.atoms[3].x).powi(2) + (s.atoms[0].y - s.atoms[3].y).powi(2) + (s.atoms[0].z - s.atoms[3].z).powi(2)).sqrt();
            let oh = ((s.atoms[3].x - s.atoms[1].x).powi(2) + (s.atoms[3].y - s.atoms[1].y).powi(2) + (s.atoms[3].z - s.atoms[1].z).powi(2)).sqrt();
            eprintln!("  frame {k:4}: O-O {oo:6.2}  Oacc···Hdon {oh:6.2}  T {:6.0} K  e_pair {:+.4}  e_three {:+.4}  e_field {:+.2e}  e_seam {:+.2e}  hb {}  units {}  dropped {}/{}",
                s.temperature(), s.e_pair, s.e_three, s.e_field, s.e_seam, hbonds_now(&s), s.seam_work.units, s.seam_work.pairs_dropped, s.seam_work.triples_dropped);
        }
        s.step_frame(1);
    }
}

fn starts() -> [Start; 2] {
    let (dsp, dpos) = dimer_positions();
    let (tsp, tpos) = ring_positions();
    [
        Start { name: "dimer", f2_name: "dimer", staked_units: 2, box_edge: 30.0, species: dsp, pos: dpos },
        Start { name: "ring", f2_name: "tetramer", staked_units: 4, box_edge: 34.0, species: tsp, pos: tpos },
    ]
}

fn run(out: &Path, model: SeamModel, settle: usize, count: usize) {
    fs::create_dir_all(out).expect("out");
    let sts = starts();

    // ---- §1: the expectation, written before any arm runs
    let kt293 = K_B * 293.0;
    let kt150 = K_B * 150.0;
    let exps: Vec<Expect> = sts.iter().map(|st| expectation(st, model)).collect();
    let blocks: Vec<String> = sts.iter().zip(exps.iter()).map(|(st, e)| format!("  \"{}\": {}", st.name, expect_json(e, st.staked_units))).collect();
    fs::write(out.join("expectation.json"), format!(
        "{{\n  \"wall\": {{\"a\": {:.9e}, \"b\": {:.9}}},\n  \"kT_293\": {kt293:.6e}, \"kT_150\": {kt150:.6e}, \"binding_floor\": {BINDING_FLOOR:.1e},\n  \"settle\": {settle}, \"count\": {count},\n{}\n}}\n",
        model.a, model.b, blocks.join(",\n"))).expect("expectation.json");
    for (st, e) in sts.iter().zip(exps.iter()) {
        eprintln!("§1 {}: units {} (staked {}), binding {:+.4e} Ha = field {:+.4e} + wall {:+.4e}; kT293 {kt293:.2e} → {} / 150 K {}{}",
            st.name, e.units, st.staked_units, e.binding, e.field_part, e.wall_part, e.e293, e.e150,
            if e.void { format!("  [VOID: {}]", e.reason) } else { String::new() });
    }

    // ---- S3: the arms
    let banked = field2_arms(out);
    let applicable = settle == SETTLE && count == COUNT;
    if banked.is_none() {
        eprintln!("WARNING: FIELD-2's arms.json was not found; the OFF arms cannot be checked against it");
    }
    let mut lines = Vec::new();
    let mut voided = Vec::new();
    let mut mismatches = 0usize;
    for (st, e) in sts.iter().zip(exps.iter()) {
        if e.void {
            eprintln!("{}: VOID by §1 ({}) — its arms do not run", st.name, e.reason);
            voided.push(format!("\"{}\"", st.name));
            continue;
        }
        for temp in [293.0, 150.0] {
            for on in [false, true] {
                let a = run_arm(st, temp, if on { Some(model) } else { None }, settle, count);
                let extra = if on {
                    String::new()
                } else {
                    let key = format!("{}_{temp:.0}_off", st.f2_name);
                    let (ok, note) = check_field2(&a, banked.as_deref().and_then(|t| json_entry(t, &key)), applicable);
                    if !ok && applicable {
                        mismatches += 1;
                        eprintln!("  !! M-FIXED-POINT-TRAJECTORY: {} {temp:.0} K OFF is not FIELD-2's arm — {note}", st.name);
                    }
                    format!(", \"reproduces_field2\": {ok}, \"field2_check\": \"{note}\"")
                };
                eprintln!("{} {temp:.0} K {}: f {:.4}, n̄ {:.3}, T {:.0} K, e_field {:+.2e}, e_seam {:+.2e}, units {}, dropped {}/{}, drift {:.1e}, cols {}, mom {}  ({:.1}s)",
                    st.name, if on { "SEAM" } else { "OFF " }, a.f, a.nbar, a.t_mean, a.e_field_final, a.e_seam_final, a.units_at_start, a.pairs_dropped, a.triples_dropped, a.drift_peak, a.columns_ok, a.momentum_ok, a.seconds);
                lines.push(format!("  \"{}_{temp:.0}_{}\": {}", st.name, if on { "seam" } else { "off" }, arm_json(&a, &extra)));
            }
        }
    }
    lines.push(format!("  \"voided\": [{}]", voided.join(", ")));
    fs::write(out.join("arms.json"), format!("{{\n{}\n}}\n", lines.join(",\n"))).expect("arms.json");
    if mismatches > 0 {
        eprintln!("{mismatches} OFF arm(s) did not reproduce FIELD-2 — S3's comparison is VOID for those arms");
    }
    println!("done");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("probe") => {
            let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/field3".to_string()));
            match wall_from(&out) {
                Ok(model) => probe(model),
                Err(why) => {
                    eprintln!("REFUSED: the wall has not been harvested ({why}). Run `field3_harvest fit` first.");
                    std::process::exit(2);
                }
            }
        }
        Some("smoke") => {
            let out = PathBuf::from(args.get(2).cloned().expect("smoke needs an output directory"));
            let a: f64 = args.get(3).expect("smoke needs A").parse().expect("A");
            let b: f64 = args.get(4).expect("smoke needs b").parse().expect("b");
            eprintln!("SMOKE: coefficients from the command line (a {a:.6e}, b {b:.6}), 20 settling and 50 counted frames — NOT the freeze's arms");
            run(&out, SeamModel::wall_only(a, b), 20, 50);
        }
        _ => {
            let out = PathBuf::from(args.get(1).cloned().unwrap_or_else(|| "../conformance/water_observatory/field3".to_string()));
            match wall_from(&out) {
                Ok(model) => {
                    eprintln!("wall: a {:.9e}, b {:.9} (from {})", model.a, model.b, out.join("wall.json").display());
                    run(&out, model, SETTLE, COUNT);
                }
                Err(why) => {
                    eprintln!("REFUSED: the harvest has not landed ({why}). FIELD-3 part C writes `wall.json`; no wall, no arms — nothing was written.");
                    std::process::exit(2);
                }
            }
        }
    }
}
