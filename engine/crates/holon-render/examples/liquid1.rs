//! LIQUID-1 (`conformance/water_observatory/LIQUID1_PREREG.md`): the periodic liquid — 128
//! waters at the state point, the seam law under the lattice sum, and the first three
//! readings of water asked whether they are water's.
//!
//! ```text
//! cargo run --release -p holon-render --example liquid1 -- [--dry] [OUT_DIR]
//! ```
//!
//! The order below is the freeze's order and it is load-bearing. `door.json` is written
//! first, because a cell the image rule refuses has no arm. `expectation.json` is written
//! BEFORE ANY FRAME (§1, M-EMPTY-SECTOR): the box's cross-unit energy per water, its two
//! parts, and which of hold / no expectation / break the `−2 kT` and `−kT` bands put it in;
//! fewer than 128 units at the start is VOID and the arm does not run. `price.json` is
//! written after the first 100 frames and BEFORE the counted ones (§2 L2,
//! M-CHEAPER-THAN-ITS-PRICE), so the cost model cannot be fitted to the answer.
//!
//! **CONDITIONED (the freeze's first paragraph).** This runs a COUNTED arm only on a seam
//! law that passes `SeamModel::bounded`. `--dry` runs the instruments on a REFUSED law and
//! says so in every file it writes: `door.json` carries `law_refused_for_counted_arm: true`
//! and no reading in `arm.json` is a reading of anything. It is an instrument check.

use holon_lens::lens::{diffusion, hbonds, hbonds_periodic, rdf_oo, LensRefusal};
use holon_lens::traj::{BondSet, Frame, Header, Trajectory, AU_TIME_FS};
use holon_render::channel::Row;
use holon_render::seam::SeamModel;
use holon_render::sim::{Boundary, Sim, INTRA_UNIT_REACH, SEAM_REACH_BUDGET};
use holon_render::waterbox::{bohr2_per_fs_to_cm2_per_s, first_peak, liquid_box};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::{scene, K_B};

// ------------------------------------------------------------------ the freeze's numbers

/// §0: the state point. Two numbers CHOSEN and declared, the way a temperature is.
const DENSITY_G_CM3: f64 = 0.997;
const TEMPERATURE_K: f64 = 293.0;
/// §0: one seed, declared (M-FIXED-POINT-TRAJECTORY).
const SEED: u64 = 0x4c49_5155_4944;
/// LIQUID-1 Amendment 2: the C² switch on every seam term, `r_on = r_cut − 2`; chosen from the
/// half-edge 14.797 bohr with 0.8 bohr to spare. `0.0` would be no switch (every record before
/// the amendment).
const SEAM_CUTOFF_BOHR: f64 = 14.0;
/// Amendment 2's stake on the truncation: the energy the switch removes on the start box,
/// per water, must be under a hundredth of kT.
const TRUNCATION_STAKE_PER_WATER: f64 = 1.0e-5;
const N_CELLS: usize = 4;
const N_WATERS: usize = 128;

/// §0: the arm. 2,000 settling frames, then 100,000 counted, one integrator step per frame.
const SETTLE: usize = 2_000;
const COUNT: usize = 100_000;
/// `--dry`: the instruments exercised end to end on 200 counted frames. NOT the arm.
const DRY_COUNT: usize = 200;

/// A DIAGNOSTIC SCREEN (`--screen <label>`): the freeze's arm with one or more knobs turned —
/// temperature, a charge scale, a declared C₆, the settling count, the transfer term's
/// angular exponents. Nothing a screen writes is a reading: every file carries `dry: true`
/// and `screen.json` names the knobs. Its use is to choose which hypotheses enter LIQUID-2's
/// freeze as pre-committed branches, not to read water.
struct Screen {
    label: String,
    temp: f64,
    qscale: f64,
    c6: Option<f64>,
    settle: usize,
    mct: Option<u8>,
    kct: Option<u8>,
    lambda_deg: Option<f64>,
    pctscale: f64,
    pscale: f64,
    hbangle: Option<f64>,
}
static SCREEN: std::sync::OnceLock<Screen> = std::sync::OnceLock::new();
fn temp_k() -> f64 {
    SCREEN.get().map(|x| x.temp).unwrap_or(TEMPERATURE_K)
}
fn settle() -> usize {
    SCREEN.get().map(|x| x.settle).unwrap_or(SETTLE)
}
/// §2 L2: the price is measured on the first 100 frames and written before the counted ones.
const PRICE_FRAMES: usize = 100;
/// §0: the readouts every 100 frames; the units and `pbc_ok` every frame.
const READOUT_EVERY: usize = 100;
/// §0: the radial distribution's declared bin.
const RDF_DR: f64 = 0.1;

// ---- the three KILL bands. From EXPERIMENT, declared as kills wherever they appear (§0).
/// R1: water's first oxygen–oxygen peak, position, bohr — 2.65–2.95 Å (Soper 2000; Skinner
/// et al. 2013).
const KILL_R1_POS: (f64, f64) = (5.0, 5.6);
/// R1: its height.
const KILL_R1_HEIGHT: (f64, f64) = (2.0, 4.0);
/// R2: hydrogen bonds per molecule.
const KILL_R2: (f64, f64) = (3.0, 4.0);
/// R3: self-diffusion, cm²/s, within a factor of 3 (Krynicki–Green–Sawyer 1978; Mills 1973).
const KILL_R3_CENTRE: f64 = 2.3e-5;
const KILL_R3_FACTOR: f64 = 3.0;

/// §5's carrier: the first shell, 3.2 Å in bohr — converted here from the freeze's angstroms
/// through the builder's own bohr, never typed as a second number.
fn first_shell_bohr() -> f64 {
    3.2 / holon_render::waterbox::BOHR_ANGSTROM
}
/// §5: the carrier's floor. Below this the plants act on a sector with nothing in it.
const CARRIER_FLOOR: f64 = 0.05;
/// §5: both plants must move their reading by more than this fraction.
const PLANT_MISS: f64 = 0.20;

/// THE ENGINE'S TIME UNIT is spent inside the lens (`mean_lag_fs` multiplies the frames' own
/// elapsed ATOMIC time by `holon_lens::traj::AU_TIME_FS`, `holon-lens/src/traj.rs:52`), so what
/// this runner adds is the LENGTH. Both live in [`holon_render::waterbox::bohr2_per_fs_to_cm2_per_s`]
/// so the arm and `tests/liquid.rs` convert through ONE statement; the engine's own
/// `AU_TIME_S` (`holon-render/src/sim.rs:91`) is checked against the lens's constant at run
/// time rather than trusted, and both are written into `arm.json`.

// ------------------------------------------------------------------------ JSON, by hand

/// A number, or `null` when it is not finite. NEVER a `+` sign: JSON has no leading plus and
/// `python3 -m json.tool` is the gate (M-FORMAT-FLOOR).
fn n(x: f64) -> String {
    if x.is_finite() {
        format!("{x:.9e}")
    } else {
        "null".to_string()
    }
}

fn s(t: &str) -> String {
    let mut out = String::with_capacity(t.len() + 2);
    out.push('"');
    for c in t.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn opt(t: &Option<String>) -> String {
    match t {
        Some(x) => s(x),
        None => "null".to_string(),
    }
}

fn arr(v: &[f64]) -> String {
    let parts: Vec<String> = v.iter().map(|&x| n(x)).collect();
    format!("[{}]", parts.join(", "))
}

fn write(out: &Path, name: &str, body: String) {
    fs::create_dir_all(out).expect("the output directory");
    fs::write(out.join(name), body).unwrap_or_else(|e| panic!("{name}: {e}"));
}

// ------------------------------------------------------------------------- the seam law

struct Law {
    model: SeamModel,
    source: String,
    /// `Some` when `SeamModel::bounded` names a fall: the law may not carry a COUNTED arm.
    refusal: Option<String>,
}

fn json_num(t: &str, key: &str) -> f64 {
    t.split(&format!("\"{key}\": "))
        .nth(1)
        .and_then(|x| x.split(|c| c == ',' || c == '\n' || c == '}').next())
        .and_then(|x| x.trim().parse::<f64>().ok())
        .unwrap_or(f64::NAN)
}

/// The harvested law, newest record first — `examples/field3_hbonds.rs`'s loader, with the
/// search widened from one directory to the observatory's campaign directories because
/// LIQUID-1's own output directory holds no harvest of its own.
///
/// The FIELD-9 G-B0 boundedness walk is run exactly as that loader runs it, with the
/// record's own `r_min` per class where it carries them. Its verdict is RETURNED rather than
/// thrown: a counted arm refuses on it (the freeze's condition), and `--dry` proceeds past it
/// with the refusal recorded in every file.
fn load_law(out: &Path) -> Result<Law, String> {
    let root = out.parent().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let mut cands: Vec<PathBuf> = Vec::new();
    for f in ["wall_ct2.json", "wall_ct.json", "wall9.json", "wall8.json", "wall7.json", "wall6.json", "wall5.json", "wall4.json", "wall.json"] {
        cands.push(out.join(f));
    }
    for (dir, f) in [
        ("ct2", "wall_ct2.json"),
        ("ct1", "wall_ct.json"),
        ("field9", "wall9.json"),
        ("field8", "wall8.json"),
        ("field7", "wall7.json"),
        ("field6", "wall6.json"),
        ("field5", "wall5.json"),
        ("field4", "wall4.json"),
        ("field3", "wall.json"),
    ] {
        cands.push(root.join(dir).join(f));
        cands.push(PathBuf::from("../conformance/water_observatory").join(dir).join(f));
        cands.push(PathBuf::from("conformance/water_observatory").join(dir).join(f));
    }
    let p = cands.into_iter().find(|p| p.exists()).ok_or_else(|| "no harvested seam law anywhere on the search path".to_string())?;
    let t = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    let (a, b) = (json_num(&t, "a"), json_num(&t, "b"));
    if !a.is_finite() || !b.is_finite() {
        return Err(format!("{}: no \"a\"/\"b\" in the file", p.display()));
    }
    if a == 0.0 {
        return Err(format!("{}: a = 0 — the seam rule with no wall is a legitimate state and is NOT this arm", p.display()));
    }
    let g = |k: &str| {
        let v = json_num(&t, k);
        if v.is_finite() {
            v
        } else {
            0.0
        }
    };
    let model = SeamModel {
        a,
        b,
        p: g("p"),
        c: g("c"),
        c6: g("c6"),
        a_oh: g("a_oh"),
        b_oh: g("b_oh"),
        a_hh: g("a_hh"),
        b_hh: g("b_hh"),
        p_hh: g("p_hh"),
        c_hh: g("c_hh"),
        p_ct: g("p_ct"),
        c_ct: g("c_ct"),
        m_ct: g("m_ct") as u8,
        k_ct: g("k_ct") as u8,
        lambda_ct: g("lambda_ct"),
        ..SeamModel::NO_WALL
    };
    let q_h = holon_render::field::water_charge_at_pin();
    let rmin = |k: &str, default: f64| {
        let v = json_num(&t, k);
        if v.is_finite() && v > 0.0 {
            v
        } else {
            default
        }
    };
    let r_min = [rmin("r_min_oo", 4.724), rmin("r_min_oh", 2.78), rmin("r_min_hh", 3.5)];
    let kt = K_B * temp_k();
    let refusal = model
        .bounded(q_h, r_min, kt)
        .map(|why| format!("{}: the harvested law is NOT BOUNDED below its fit range — {why} (FIELD-9 G-B0)", p.display()));
    Ok(Law { model, source: p.display().to_string(), refusal })
}

// --------------------------------------------------------------------------- the readouts

/// The mean radial distribution over the sampled frames, and the raw-difference PLANT beside
/// it, accumulated in one place so the two can never be taken over different frames.
struct RdfAccum {
    r: Vec<f64>,
    g_sum: Vec<f64>,
    /// PLANT (i): the same lens with the box's periodicity removed. `rdf_oo` at a cell of
    /// `PLANT_CELL` bohr takes every difference RAW (`round(d/L)` is exactly zero for every
    /// separation this box can hold), and the reading is then renormalised from that cell's
    /// density to the real one — a scalar rescale, so the plant is the histogram's change and
    /// nothing else.
    g_raw_sum: Vec<f64>,
    frames: usize,
    rho_o: f64,
    n_o: usize,
}

/// The cell the plant's raw differences are taken in: large enough that no minimum image
/// reduction fires, small enough that its density stays a normal double.
const PLANT_CELL: f64 = 1.0e6;

impl RdfAccum {
    fn new() -> Self {
        RdfAccum { r: Vec::new(), g_sum: Vec::new(), g_raw_sum: Vec::new(), frames: 0, rho_o: 0.0, n_o: 0 }
    }

    fn push(&mut self, pos: &[[f64; 3]], z: &[u32], l: f64) -> Result<(), LensRefusal> {
        let r_max = 0.5 * l;
        let true_rdf = rdf_oo(pos, z, [l, l, l], RDF_DR, r_max)?;
        let raw = rdf_oo(pos, z, [PLANT_CELL; 3], RDF_DR, r_max)?;
        let rescale = raw.rho_o / true_rdf.rho_o;
        if self.frames == 0 {
            self.r = true_rdf.r.clone();
            self.g_sum = vec![0.0; true_rdf.g.len()];
            self.g_raw_sum = vec![0.0; true_rdf.g.len()];
            self.rho_o = true_rdf.rho_o;
            self.n_o = true_rdf.n_o;
        }
        for k in 0..self.g_sum.len() {
            self.g_sum[k] += true_rdf.g[k];
            self.g_raw_sum[k] += raw.g[k] * rescale;
        }
        self.frames += 1;
        Ok(())
    }

    fn mean(&self) -> (Vec<f64>, Vec<f64>) {
        let f = self.frames.max(1) as f64;
        (self.g_sum.iter().map(|x| x / f).collect(), self.g_raw_sum.iter().map(|x| x / f).collect())
    }
}

fn band(x: f64, lo: f64, hi: f64) -> bool {
    x >= lo && x <= hi
}

// -------------------------------------------------------------------------------- the arm

struct Void {
    frame: usize,
    phase: &'static str,
    why: String,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let val = |k: &str| -> Option<String> { args.iter().position(|a| a == k).and_then(|i| args.get(i + 1).cloned()) };
    let screen_label = val("--screen");
    if let Some(label) = screen_label.clone() {
        let sc = Screen {
            label,
            temp: val("--temp").and_then(|v| v.parse().ok()).unwrap_or(TEMPERATURE_K),
            qscale: val("--qscale").and_then(|v| v.parse().ok()).unwrap_or(1.0),
            c6: val("--c6").and_then(|v| v.parse().ok()),
            settle: val("--settle").and_then(|v| v.parse().ok()).unwrap_or(SETTLE),
            mct: val("--mct").and_then(|v| v.parse().ok()),
            kct: val("--kct").and_then(|v| v.parse().ok()),
            lambda_deg: val("--lambda").and_then(|v| v.parse().ok()),
            pctscale: val("--pctscale").and_then(|v| v.parse().ok()).unwrap_or(1.0),
            pscale: val("--pscale").and_then(|v| v.parse().ok()).unwrap_or(1.0),
            hbangle: val("--hbangle").and_then(|v| v.parse().ok()),
        };
        SCREEN.set(sc).ok();
    }
    let dry = args.iter().any(|a| a == "--dry") || screen_label.is_some();
    // the out directory: the first argument that is neither a flag nor a flag's value
    let flag_values: Vec<usize> = args.iter().enumerate().filter(|(_, a)| a.starts_with("--") && *a != "--dry").map(|(i, _)| i + 1).collect();
    let out = PathBuf::from(
        args.iter()
            .enumerate()
            .find(|(i, a)| !a.starts_with("--") && !flag_values.contains(i))
            .map(|(_, a)| a.clone())
            .unwrap_or_else(|| "../conformance/water_observatory/liquid1".to_string()),
    );

    let law = match load_law(&out) {
        Ok(l) => l,
        Err(why) => {
            eprintln!("REFUSED: {why}. No seam law, no arm — nothing was written.");
            std::process::exit(2);
        }
    };
    eprintln!(
        "seam law: {} (a {:.6e}, b {:.6}, P {:.6e}, c {:.6}, C6 {:.6e}, A_OH {:.6e}, b_OH {:.6}, A_HH {:.6e}, b_HH {:.6}, P_HH {:.6e}, c_HH {:.6}, P_CT {:.6e}, c_CT {:.6})",
        law.source, law.model.a, law.model.b, law.model.p, law.model.c, law.model.c6, law.model.a_oh, law.model.b_oh, law.model.a_hh, law.model.b_hh, law.model.p_hh, law.model.c_hh, law.model.p_ct, law.model.c_ct
    );
    if let Some(why) = &law.refusal {
        if !dry {
            eprintln!("REFUSED: {why}");
            eprintln!("LIQUID-1 is CONDITIONED on the first seam law whose boundedness gate admits it; this one does not. Nothing was written.");
            std::process::exit(2);
        }
        eprintln!("WARNING: this law is REFUSED for a COUNTED arm — {why}");
        eprintln!("WARNING: --dry proceeds anyway. Nothing this run writes is a reading of anything; it is an instrument check.");
    }
    let count = if screen_label.is_some() { val("--count").and_then(|v| v.parse().ok()).unwrap_or(10_000) } else if dry { DRY_COUNT } else { COUNT };
    if dry {
        eprintln!("DRY RUN: {} settling frames and {count} counted frames (the freeze's arm is {SETTLE} and {COUNT}).", settle());
    }

    run(&out, &law, count, dry);
}

fn run(out: &Path, law: &Law, count: usize, dry: bool) {
    let t_start = Instant::now();
    let settle_n = settle();
    let temp_k_v = temp_k();
    if let Some(sc) = SCREEN.get() {
        write(out, "screen.json", format!("{{\n  \"DIAGNOSTIC\": \"a screen, not a reading; the knobs below are declared, not derived\",\n  \"label\": {}, \"temperature_k\": {}, \"charge_scale\": {}, \"c6_declared\": {}, \"settle_frames\": {}, \"counted_frames\": {}, \"m_ct_override\": {}, \"k_ct_override\": {}, \"lambda_deg_override\": {}, \"p_ct_scale\": {}, \"p_ho_scale\": {}, \"hbond_angle_readout_deg\": {}\n}}\n", s(&sc.label), n(sc.temp), n(sc.qscale), sc.c6.map(n).unwrap_or_else(|| "null".to_string()), sc.settle, count, sc.mct.map(|m| m.to_string()).unwrap_or_else(|| "null".to_string()), sc.kct.map(|m| m.to_string()).unwrap_or_else(|| "null".to_string()), sc.lambda_deg.map(n).unwrap_or_else(|| "null".to_string()), n(sc.pctscale), n(sc.pscale), sc.hbangle.map(n).unwrap_or_else(|| "null".to_string())));
        eprintln!("SCREEN {}: T {} K, charge x{}, C6 {:?}, settle {}, count {}, m_ct {:?} — a DIAGNOSTIC, not a reading", sc.label, sc.temp, sc.qscale, sc.c6, sc.settle, count, sc.mct);
    }
    let (species, pos, l) = liquid_box(N_CELLS, DENSITY_G_CM3, SEED);
    let cell = [l, l, l];
    eprintln!("box: {} atoms, {N_WATERS} waters, edge {l:.6} bohr at {DENSITY_G_CM3} g/cm³, seed {SEED:#x}", species.len());

    let mut sim: Box<Sim> = scene(&species, &pos, l, temp_k_v);
    let tables_reach = sim.legality_radius();
    let q_override = SCREEN.get().and_then(|x| if x.qscale != 1.0 { Some(holon_render::field::water_charge_at_pin() * x.qscale) } else { None });
    sim.set_field(true, q_override).expect("the open box admits the field");
    // LIQUID-1 Amendment 2: the law with the declared switch; the unswitched reach is
    // reported beside the switched one
    let mut model = SeamModel { r_cut: SEAM_CUTOFF_BOHR, ..law.model };
    if let Some(x) = SCREEN.get() {
        if let Some(c6) = x.c6 { model.c6 = c6; }
        if let Some(m) = x.mct { model.m_ct = m; model.k_ct = 0; }
        if let Some(k) = x.kct { model.k_ct = k; }
        if let Some(lam) = x.lambda_deg { model.lambda_ct = lam.to_radians(); }
        model.p_ct *= x.pctscale;
        model.p *= x.pscale;
    }
    sim.set_seam(Some(model)).expect("no acuity frame is installed");

    // ---------------------------------------------------------------- door.json (§2 L0)
    let seam_reach_unswitched = law.model.reach(SEAM_REACH_BUDGET);
    let seam_reach = model.reach(SEAM_REACH_BUDGET);
    let legality = sim.legality_radius();
    let units_reading = sim.units_reading();
    // the truncation's price (Amendment 2 §3): the energy the switch removes on the start box,
    // summed over every cross-unit pair with r > r_on under the minimum image, by class
    let r_on = SEAM_CUTOFF_BOHR - 2.0;
    let z: Vec<u32> = species.iter().map(|sp| sp.z).collect();
    let mut truncation_tail = 0.0f64;
    let mut tail_pairs = 0usize;
    for i in 0..pos.len() {
        let ui = units_reading[i];
        if ui == holon_render::seam::FREE || (z[i] != 8 && z[i] != 1) {
            continue;
        }
        for j in (i + 1)..pos.len() {
            let uj = units_reading[j];
            if uj == holon_render::seam::FREE || uj == ui || (z[j] != 8 && z[j] != 1) {
                continue;
            }
            let mut d2 = 0.0;
            for c in 0..3 {
                let mut d = pos[i][c] - pos[j][c];
                d -= l * (d / l).round();
                d2 += d * d;
            }
            let r = d2.sqrt();
            if r <= r_on {
                continue;
            }
            let (sw, _) = model.switch(r);
            let u = match (z[i], z[j]) {
                (8, 8) => law.model.wall(r) + law.model.dispersion(r),
                (1, 1) => law.model.contact_hh(r) + law.model.wall_hh(r),
                _ => law.model.penetration(r) + law.model.wall_oh(r) + law.model.charge_transfer(r),
            };
            truncation_tail += u.abs() * (1.0 - sw);
            tail_pairs += 1;
        }
    }
    let truncation_per_water = truncation_tail / N_WATERS as f64;
    let truncation_ok = truncation_per_water <= TRUNCATION_STAKE_PER_WATER;
    eprintln!("L0 door (Amendment 2): seam switch r_cut {SEAM_CUTOFF_BOHR:.2} bohr (r_on {r_on:.2}); unswitched reach {seam_reach_unswitched:.4}, switched reach {seam_reach:.4}; truncation tail {truncation_tail:.4e} Ha over {tail_pairs} pairs beyond r_on = {truncation_per_water:.4e} per water against the stake {TRUNCATION_STAKE_PER_WATER:e} → {}", if truncation_ok { "within" } else { "VOID" });
    let n_atoms = sim.n;
    let units_at_door = units_reading[..n_atoms].iter().enumerate().filter(|(i, &u)| u == *i as u32).count();
    let free_atoms = units_reading[..n_atoms].iter().filter(|&&u| u == holon_render::seam::FREE).count();
    let half_edge = 0.5 * l;
    let refusal = sim.set_boundary(Boundary::Periodic).err();
    let admitted = refusal.is_none();
    let refusal_text = refusal.map(|r| r.to_string());
    eprintln!(
        "L0 door: tables' reach {tables_reach:.4}, seam terms' reach {seam_reach:.4}, legality radius {legality:.4} (= max({INTRA_UNIT_REACH}, {seam_reach:.4})), half-edge {half_edge:.4} → {}",
        if admitted { "ADMITTED" } else { "REFUSED" }
    );
    eprintln!("L0 door: {units_at_door} units, {free_atoms} free atoms");
    write(
        out,
        "door.json",
        format!(
            "{{\n  \"law_source\": {}, \"law_refused_for_counted_arm\": {}, \"law_refusal\": {},\n  \"dry\": {},\n  \"seam_terms_reach_bohr\": {}, \"seam_terms_reach_unswitched_bohr\": {}, \"seam_switch_r_cut_bohr\": {}, \"seam_switch_r_on_bohr\": {}, \"truncation_tail_hartree\": {}, \"truncation_tail_pairs\": {}, \"truncation_tail_per_water\": {}, \"truncation_stake_per_water\": {}, \"truncation_ok\": {}, \"seam_reach_budget_hartree\": {}, \"intra_unit_reach_bohr\": {},\n  \"legality_radius_bohr\": {}, \"tables_reach_bohr\": {}, \"half_edge_bohr\": {}, \"cell_edge_bohr\": {},\n  \"units\": {}, \"staked_units\": {}, \"free_atoms\": {}, \"atoms\": {},\n  \"admitted\": {}, \"refusal\": {},\n  \"legality_is_seam_rule_to_the_bit\": {}\n}}\n",
            s(&law.source),
            law.refusal.is_some(),
            opt(&law.refusal),
            dry,
            n(seam_reach),
            n(seam_reach_unswitched),
            n(SEAM_CUTOFF_BOHR),
            n(r_on),
            n(truncation_tail),
            tail_pairs,
            n(truncation_per_water),
            n(TRUNCATION_STAKE_PER_WATER),
            truncation_ok,
            n(SEAM_REACH_BUDGET),
            n(INTRA_UNIT_REACH),
            n(legality),
            n(tables_reach),
            n(half_edge),
            n(l),
            units_at_door,
            N_WATERS,
            free_atoms,
            n_atoms,
            admitted,
            opt(&refusal_text),
            legality.to_bits() == INTRA_UNIT_REACH.max(seam_reach).to_bits()
        ),
    );
    if !admitted {
        let why = refusal_text.unwrap_or_default();
        write(out, "arm.void", format!("VOID at the door: {why}\n"));
        eprintln!("VOID at the door: {why}");
        return;
    }
    // LIQUID-1 Amendment 2 §3: the truncation's price is a stake, not a note
    if !truncation_ok {
        let why = format!("the seam switch at r_cut {SEAM_CUTOFF_BOHR:.2} bohr removes {truncation_per_water:.4e} hartree per water on the start box, over the stake {TRUNCATION_STAKE_PER_WATER:e}");
        eprintln!("VOID at the door: {why}");
        write(out, "arm.void", format!("VOID at the door: {why}\n"));
        return;
    }
    // The boundary switch is the last act of the SETUP, not an intervention inside the run:
    // the ledger's origin is the wrapped scene. `scene` rebased the open box; this rebases
    // the box the arm actually runs in, so `drift_peak` measures the dynamics and not the
    // wrap.
    sim.rebase();

    // -------------------------------------------------- expectation.json (§1), no frame yet
    sim.compute_forces();
    let e_field = sim.row(Row::Field);
    let e_seam = sim.row(Row::Seam);
    let cross_unit = e_field + e_seam;
    let per_water = cross_unit / N_WATERS as f64;
    let units_start = sim.seam_work.units;
    let kt = K_B * temp_k();
    let void_empty = units_start < N_WATERS as u64;
    let expectation = if void_empty {
        "VOID"
    } else if per_water <= -2.0 * kt {
        "hold"
    } else if per_water > -kt {
        "break"
    } else {
        "no expectation"
    };
    let sentence = if void_empty {
        format!("{units_start} units at the start, under the staked {N_WATERS}: the arm is VOID before it runs (M-EMPTY-SECTOR)")
    } else {
        format!(
            "the box's cross-unit energy is {per_water:.6e} hartree per water against kT = {kt:.6e}; under −2kT the liquid is expected to HOLD together, between −2kT and −kT there is NO EXPECTATION, above −kT it is expected to EVAPORATE into the cell — this box: {expectation}"
        )
    };
    eprintln!("§1 {sentence}");
    write(
        out,
        "expectation.json",
        format!(
            "{{\n  \"written_before_any_frame\": true, \"dry\": {dry}, \"law_refused_for_counted_arm\": {},\n  \"units\": {units_start}, \"staked_units\": {N_WATERS}, \"void\": {void_empty},\n  \"cross_unit_energy_hartree\": {}, \"field_part_hartree\": {}, \"seam_part_hartree\": {},\n  \"per_water_hartree\": {}, \"kT_293_hartree\": {}, \"minus_2kT\": {}, \"minus_kT\": {},\n  \"expectation\": {},\n  \"sentence\": {}\n}}\n",
            law.refusal.is_some(),
            n(cross_unit),
            n(e_field),
            n(e_seam),
            n(per_water),
            n(kt),
            n(-2.0 * kt),
            n(-kt),
            s(expectation),
            s(&sentence)
        ),
    );
    if void_empty {
        write(out, "arm.void", format!("{sentence}\n"));
        return;
    }

    // ------------------------------------------------------------------------- the arm
    let z: Vec<u32> = (0..n_atoms).map(|i| sim.atoms[i].species.z).collect();
    let oxy: Vec<usize> = (0..n_atoms).filter(|&i| z[i] == 8).collect();
    let dt_at_placement = sim.dt();
    let mut void: Option<Void> = None;

    // the unwrapping accumulator: the minimum-image displacement of every oxygen, summed
    // EVERY frame, so a molecule that crosses a face is followed rather than teleported
    let mut prev: Vec<[f64; 3]> = oxy.iter().map(|&i| [sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z]).collect();
    let mut unwrapped: Vec<[f64; 3]> = prev.clone();

    let mut frames_run = 0usize;

    let read_pos = |sim: &Sim| -> Vec<[f64; 3]> { (0..sim.n).map(|i| [sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z]).collect() };

    // one frame, with L0's per-pass check and the unwrap accumulation
    macro_rules! one_frame {
        ($phase:expr, $k:expr) => {{
            sim.step_frame(1);
            frames_run += 1;
            let now: Vec<[f64; 3]> = oxy.iter().map(|&i| [sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z]).collect();
            for a in 0..oxy.len() {
                for c in 0..3 {
                    let mut d = now[a][c] - prev[a][c];
                    d -= l * (d / l).round();
                    unwrapped[a][c] += d;
                }
            }
            prev = now;
            if void.is_none() {
                if !sim.pbc_ok() {
                    let (reach, half) = sim.pbc_margin();
                    void = Some(Void {
                        frame: $k,
                        phase: $phase,
                        why: format!("pbc_ok is false: the force law's reach is {reach:.4} bohr against a half-edge of {half:.4} — a unit has dissolved and the box is judged by the full law again"),
                    });
                } else if sim.seam_work.units != N_WATERS as u64 {
                    void = Some(Void {
                        frame: $k,
                        phase: $phase,
                        why: format!("the unit count is {} and the freeze stakes {N_WATERS} EXACTLY", sim.seam_work.units),
                    });
                }
            }
        }};
    }

    // ---- the price: the first 100 frames, written BEFORE the counted ones (§2 L2)
    let t_price = Instant::now();
    for k in 0..PRICE_FRAMES.min(settle_n) {
        one_frame!("settle", k);
        if void.is_some() {
            break;
        }
    }
    let price_seconds = t_price.elapsed().as_secs_f64() / PRICE_FRAMES.min(settle_n) as f64;
    let price_k_vectors = sim.field_work.k_vectors;
    let price_real_pairs = sim.field_work.pairs;
    let expected_passes = settle_n + count;
    eprintln!(
        "L2 price: {price_seconds:.6} s per force pass at {n_atoms} atoms with the lattice sum ({price_k_vectors} wave-vectors, {price_real_pairs} real pairs); {expected_passes} passes ⇒ {:.1} s expected",
        price_seconds * expected_passes as f64
    );
    write(
        out,
        "price.json",
        format!(
            "{{\n  \"written_before_the_counted_frames\": true, \"dry\": {dry},\n  \"atoms\": {n_atoms}, \"waters\": {N_WATERS},\n  \"measured_on_frames\": {},\n  \"seconds_per_force_pass\": {}, \"k_vectors\": {price_k_vectors}, \"real_pairs\": {price_real_pairs},\n  \"expected_passes\": {expected_passes}, \"expected_seconds\": {},\n  \"refused_under\": {}, \"refused_over\": {},\n  \"note\": {}\n}}\n",
            PRICE_FRAMES.min(settle_n),
            n(price_seconds),
            n(price_seconds * expected_passes as f64),
            n(0.1 * price_seconds * expected_passes as f64),
            n(10.0 * price_seconds * expected_passes as f64),
            s("the price is the whole integrator step, which at these sizes is the force pass; an arm returning under a tenth of it is refused (M-CHEAPER-THAN-ITS-PRICE)")
        ),
    );

    // ---- the rest of the settling
    if void.is_none() {
        for k in PRICE_FRAMES.min(settle_n)..settle_n {
            one_frame!("settle", k);
            if void.is_some() {
                break;
            }
        }
    }

    // ---- the counted frames
    let mut rdf = RdfAccum::new();
    let mut hb_sum = 0.0f64;
    let mut hb_open_sum = 0.0f64;
    let mut temp_sum = 0.0f64;
    let mut readouts = 0usize;
    let mut carrier_crossing = 0u64;
    let mut carrier_pairs = 0u64;
    let mut traj_frames: Vec<Frame> = Vec::new();
    // DRY ONLY: a second trajectory sampled every frame, so the diffusion path is exercised
    // end to end on a 200-frame check. It is NOT a reading and is labelled so wherever it
    // appears.
    let mut dry_frames: Vec<Frame> = Vec::new();
    let mut rdf_refusal: Option<String> = None;
    let mut hb_refusal: Option<String> = None;
    let first_shell = first_shell_bohr();

    if void.is_none() {
        for k in 0..count {
            one_frame!("counted", k);
            temp_sum += sim.temperature();
            if void.is_some() {
                break;
            }
            if dry {
                dry_frames.push(Frame {
                    index: k as u64,
                    time: sim.time,
                    temperature: sim.temperature(),
                    bonds: BondSet::empty(),
                    pos: unwrapped.clone(),
                    vel: vec![[0.0; 3]; oxy.len()],
                });
            }
            if (k + 1) % READOUT_EVERY != 0 {
                continue;
            }
            let p = read_pos(&sim);
            match holon_lens::lens::hbonds_periodic_with(&p, &z, cell, SCREEN.get().and_then(|x| x.hbangle).unwrap_or(holon_lens::lens::HB_ANGLE_DEG)) {
                Ok(v) => hb_sum += v.len() as f64,
                Err(e) => hb_refusal = Some(format!("{}: {}", e.lens, e.reason)),
            }
            // PLANT (ii): the OPEN-box lens on the same wrapped positions
            match hbonds(&p, &z) {
                Ok(v) => hb_open_sum += v.len() as f64,
                Err(e) => hb_refusal = Some(format!("{}: {}", e.lens, e.reason)),
            }
            if let Err(e) = rdf.push(&p, &z, l) {
                rdf_refusal = Some(format!("{}: {}", e.lens, e.reason));
            }
            // the carrier (§5): O–O pairs inside the first shell whose minimum-image vector
            // crosses a face
            for (ii, &i) in oxy.iter().enumerate() {
                for &j in oxy[ii + 1..].iter() {
                    let mut d2 = 0.0;
                    let mut crossed = false;
                    for c in 0..3 {
                        let raw = p[j][c] - p[i][c];
                        let shift = (raw / l).round();
                        if shift != 0.0 {
                            crossed = true;
                        }
                        let d = raw - l * shift;
                        d2 += d * d;
                    }
                    if d2 < first_shell * first_shell {
                        carrier_pairs += 1;
                        if crossed {
                            carrier_crossing += 1;
                        }
                    }
                }
            }
            traj_frames.push(Frame {
                index: k as u64,
                time: sim.time,
                temperature: sim.temperature(),
                bonds: BondSet::empty(),
                pos: unwrapped.clone(),
                vel: vec![[0.0; 3]; oxy.len()],
            });
            readouts += 1;
            if readouts % 20 == 0 || k + 1 == count {
                eprintln!(
                    "  frame {:>7}: T {:6.1} K, units {}, H-bonds/molecule {:.3}, drift {:.2e}, {} readouts",
                    k + 1,
                    sim.temperature(),
                    sim.seam_work.units,
                    hb_sum / readouts as f64 / N_WATERS as f64,
                    sim.drift_peak,
                    readouts
                );
            }
        }
    }

    let counted_frames = if let Some(v) = &void {
        if v.phase == "counted" {
            v.frame
        } else {
            0
        }
    } else {
        count
    };
    let wall_seconds = t_start.elapsed().as_secs_f64();

    // ------------------------------------------------------------------------ the readings
    let (g_mean, g_raw) = rdf.mean();
    let peak = if readouts > 0 { first_peak(&rdf.r, &g_mean) } else { None };
    let peak_raw = if readouts > 0 { first_peak(&rdf.r, &g_raw) } else { None };
    let r2 = if readouts > 0 { hb_sum / readouts as f64 / N_WATERS as f64 } else { f64::NAN };
    let r2_open = if readouts > 0 { hb_open_sum / readouts as f64 / N_WATERS as f64 } else { f64::NAN };
    let carrier = if carrier_pairs > 0 { carrier_crossing as f64 / carrier_pairs as f64 } else { f64::NAN };

    // R1
    let (r1_pos, r1_height) = peak.unwrap_or((f64::NAN, f64::NAN));
    let r1_branch = if !r1_pos.is_finite() {
        "c"
    } else if !band(r1_pos, KILL_R1_POS.0, KILL_R1_POS.1) {
        "c"
    } else if band(r1_height, KILL_R1_HEIGHT.0, KILL_R1_HEIGHT.1) {
        "a"
    } else {
        "b"
    };
    // R2
    let r2_branch = if !r2.is_finite() {
        "VOID"
    } else if band(r2, KILL_R2.0, KILL_R2.1) {
        "a"
    } else if r2 < KILL_R2.0 {
        "b"
    } else {
        "c"
    };
    // R3
    let conv = bohr2_per_fs_to_cm2_per_s();
    let au_s_from_lens = AU_TIME_FS * 1.0e-15;
    let time_units_agree = ((au_s_from_lens - holon_render::sim::AU_TIME_S) / holon_render::sim::AU_TIME_S).abs() < 1.0e-9;
    let build_traj = |frames: Vec<Frame>| -> Trajectory {
        let nf = frames.len();
        Trajectory {
            header: Header {
                seed: SEED,
                n_atoms: oxy.len(),
                dims: 3,
                substeps: 1,
                n_frames: nf,
                dt: dt_at_placement,
                box_w: l,
                box_h: l,
                box_d: l,
                z: vec![8; oxy.len()],
            },
            frames,
        }
    };
    let read_d = |frames: Vec<Frame>| -> (Option<f64>, Option<String>, usize) {
        let nf = frames.len();
        let max_lag = (nf / 4).max(2);
        if nf < 8 {
            return (None, Some(format!("{nf} sampled frames is too few for a lag window (the lens needs at least three lags under max_lag)")), max_lag);
        }
        let t = build_traj(frames);
        match diffusion(&t, max_lag) {
            Ok(d) => (Some(d * conv), None, max_lag),
            Err(e) => (None, Some(format!("{} refuses (gate: {}): {}", e.lens, e.gate, e.reason)), max_lag),
        }
    };
    let (r3, r3_refusal, r3_max_lag) = read_d(traj_frames.clone());
    let (dry_d, dry_refusal, dry_max_lag) = if dry { read_d(dry_frames) } else { (None, None, 0) };
    let r3_branch = match r3 {
        None => "VOID",
        Some(d) if d >= KILL_R3_CENTRE / KILL_R3_FACTOR && d <= KILL_R3_CENTRE * KILL_R3_FACTOR => "a",
        Some(d) if d > KILL_R3_CENTRE * KILL_R3_FACTOR => "b",
        Some(_) => "c",
    };

    // L1
    let transition = sim.work.field.abs().max(sim.work.seam.abs());
    let drift_bar = if transition > 0.0 { 0.1 * transition } else { 1e-5 };
    let columns_ok = sim.work_columns_ok();
    let drift_ok = sim.drift_peak <= drift_bar;
    let momentum_ok = sim.momentum_residual() <= sim.momentum_bound();
    // L2
    let expected = price_seconds * expected_passes as f64;
    let l2_ok = wall_seconds >= 0.1 * expected && wall_seconds <= 10.0 * expected;
    // the plants
    let plant_i_move = match (peak, peak_raw) {
        (Some((_, h)), Some((_, hr))) if h != 0.0 => (hr - h).abs() / h,
        (Some(_), None) => f64::INFINITY,
        _ => f64::NAN,
    };
    let plant_ii_move = if r2 > 0.0 { (r2 - r2_open) / r2 } else { f64::NAN };
    let carrier_ok = carrier.is_finite() && carrier >= CARRIER_FLOOR;
    let plant_i_fires = plant_i_move.is_finite() && plant_i_move > PLANT_MISS || plant_i_move.is_infinite();
    let plant_ii_fires = plant_ii_move.is_finite() && plant_ii_move > PLANT_MISS;

    eprintln!("R1 first O–O peak: r {r1_pos:.3} bohr, g {r1_height:.3} → branch ({r1_branch})  [KILL: position {:?} bohr, height {:?}]", KILL_R1_POS, KILL_R1_HEIGHT);
    eprintln!("R2 H-bonds/molecule: {r2:.4} → branch ({r2_branch})  [KILL: {:?}]", KILL_R2);
    match (&r3, &r3_refusal) {
        (Some(d), _) => eprintln!("R3 self-diffusion: {d:.4e} cm²/s → branch ({r3_branch})  [KILL: within a factor of {KILL_R3_FACTOR} of {KILL_R3_CENTRE:.1e}]"),
        (None, Some(w)) => eprintln!("R3 self-diffusion: VOID — {w}"),
        (None, None) => eprintln!("R3 self-diffusion: VOID"),
    }
    eprintln!("L1 books: columns {columns_ok}, drift peak {:.3e} against {drift_bar:.3e} ({drift_ok}), momentum {:.2e} / bound {:.2e} ({momentum_ok})", sim.drift_peak, sim.momentum_residual(), sim.momentum_bound());
    eprintln!("L2 price: {wall_seconds:.1} s against {expected:.1} s expected ({l2_ok})");
    eprintln!("carrier: {:.4} of first-shell O–O pairs cross a face (floor {CARRIER_FLOOR}) — {}", carrier, if carrier_ok { "nonzero" } else { "UNDER THE FLOOR" });
    eprintln!("plant (i) raw-difference g_OO: first peak {:?} vs {:?}, moved {:.3} (needs > {PLANT_MISS})", peak_raw, peak, plant_i_move);
    eprintln!("plant (ii) open-box H-bonds: {r2_open:.4} against {r2:.4}, fell {:.3} (needs > {PLANT_MISS})", plant_ii_move);
    if let Some(v) = &void {
        eprintln!("VOID at {} frame {}: {}", v.phase, v.frame, v.why);
    }

    // ------------------------------------------------------------------------ the records
    write(
        out,
        "rdf.json",
        format!(
            "{{\n  \"dry\": {dry}, \"law_refused_for_counted_arm\": {},\n  \"dr_bohr\": {}, \"r_max_bohr\": {}, \"frames\": {}, \"n_o\": {}, \"rho_o_bohr3\": {},\n  \"r\": {},\n  \"g\": {},\n  \"g_plant_raw_differences\": {},\n  \"plant_cell_bohr\": {}\n}}\n",
            law.refusal.is_some(),
            n(RDF_DR),
            n(0.5 * l),
            rdf.frames,
            rdf.n_o,
            n(rdf.rho_o),
            arr(&rdf.r),
            arr(&g_mean),
            arr(&g_raw),
            n(PLANT_CELL)
        ),
    );

    let void_json = match &void {
        Some(v) => format!("{{\"void\": true, \"phase\": {}, \"frame\": {}, \"why\": {}}}", s(v.phase), v.frame, s(&v.why)),
        None => "{\"void\": false}".to_string(),
    };
    write(
        out,
        "arm.json",
        format!(
            "{{\n\
             \x20 \"dry\": {dry}, \"law_source\": {}, \"law_refused_for_counted_arm\": {}, \"law_refusal\": {},\n\
             \x20 \"is_a_reading\": {},\n\
             \x20 \"settle_frames\": {settle_n}, \"counted_frames_staked\": {count}, \"counted_frames_run\": {counted_frames}, \"frames_run_total\": {frames_run}, \"readouts\": {readouts},\n\
             \x20 \"cell_edge_bohr\": {}, \"waters\": {N_WATERS}, \"atoms\": {n_atoms}, \"density_g_cm3\": {}, \"target_temperature_k\": {}, \"mean_temperature_k\": {},\n\
             \x20 \"l0\": {},\n\
             \x20 \"r1\": {{\"first_peak_position_bohr\": {}, \"first_peak_height\": {}, \"bin_bohr\": {}, \"kill_position_bohr\": [{}, {}], \"kill_height\": [{}, {}], \"branch\": {}}},\n\
             \x20 \"r2\": {{\"hbonds_per_molecule\": {}, \"kill\": [{}, {}], \"branch\": {}, \"refusal\": {}}},\n\
             \x20 \"r3\": {{\"diffusion_cm2_per_s\": {}, \"kill_centre_cm2_per_s\": {}, \"kill_factor\": {}, \"branch\": {}, \"refusal\": {}, \"max_lag_frames\": {}, \"sampled_frames\": {}, \"bohr2_per_fs_to_cm2_per_s\": {}, \"au_time_s_engine\": {}, \"au_time_fs_lens\": {}, \"time_units_agree\": {}}},\n\
             \x20 \"l1\": {{\"columns_ok\": {}, \"w_ext\": {}, \"work_hand\": {}, \"work_thermostat\": {}, \"work_field\": {}, \"work_seam\": {}, \"drift_peak\": {}, \"drift_bar\": {}, \"drift_ok\": {}, \"momentum_residual\": {}, \"momentum_bound\": {}, \"momentum_ok\": {}, \"k_vectors\": {}, \"real_pairs\": {}, \"seam_pairs_dropped_total\": {}, \"seam_triples_dropped_total\": {}, \"seam_oo_pairs\": {}, \"seam_ho_pairs\": {}, \"seam_transitions\": {}, \"field_transitions\": {}}},\n\
             \x20 \"l2\": {{\"wall_seconds\": {}, \"seconds_per_pass\": {}, \"expected_passes\": {}, \"expected_seconds\": {}, \"within_0.1x_to_10x\": {}}},\n\
             \x20 \"carrier\": {{\"first_shell_bohr\": {}, \"first_shell_angstrom\": {}, \"pairs\": {}, \"crossing_a_face\": {}, \"fraction\": {}, \"floor\": {}, \"nonzero\": {}}},\n\
             \x20 \"plants\": {{\"i_rdf_raw_differences\": {{\"peak_height\": {}, \"peak_position_bohr\": {}, \"fractional_move\": {}, \"needs\": {}, \"fires\": {}}}, \"ii_open_box_hbonds\": {{\"per_molecule\": {}, \"fractional_fall\": {}, \"needs\": {}, \"fires\": {}}}}},\n\
             \x20 \"rdf_refusal\": {}, \"dry_diffusion_path_check\": {{\"cm2_per_s\": {}, \"refusal\": {}, \"max_lag_frames\": {}, \"note\": {}}}\n\
             }}\n",
            s(&law.source),
            law.refusal.is_some(),
            opt(&law.refusal),
            !dry && law.refusal.is_none() && void.is_none(),
            n(l),
            n(DENSITY_G_CM3),
            n(temp_k_v),
            n(if counted_frames > 0 { temp_sum / counted_frames as f64 } else { f64::NAN }),
            void_json,
            n(r1_pos),
            n(r1_height),
            n(RDF_DR),
            n(KILL_R1_POS.0),
            n(KILL_R1_POS.1),
            n(KILL_R1_HEIGHT.0),
            n(KILL_R1_HEIGHT.1),
            s(r1_branch),
            n(r2),
            n(KILL_R2.0),
            n(KILL_R2.1),
            s(r2_branch),
            opt(&hb_refusal),
            match r3 {
                Some(d) => n(d),
                None => "null".to_string(),
            },
            n(KILL_R3_CENTRE),
            n(KILL_R3_FACTOR),
            s(r3_branch),
            opt(&r3_refusal),
            r3_max_lag,
            traj_frames.len(),
            n(conv),
            n(holon_render::sim::AU_TIME_S),
            n(AU_TIME_FS),
            time_units_agree,
            columns_ok,
            n(sim.w_ext),
            n(sim.work.hand),
            n(sim.work.thermostat),
            n(sim.work.field),
            n(sim.work.seam),
            n(sim.drift_peak),
            n(drift_bar),
            drift_ok,
            n(sim.momentum_residual()),
            n(sim.momentum_bound()),
            momentum_ok,
            sim.field_work.k_vectors,
            sim.field_work.pairs,
            sim.seam_work.pairs_dropped_total,
            sim.seam_work.triples_dropped_total,
            sim.seam_work.oo_pairs,
            sim.seam_work.ho_pairs,
            sim.seam_work.transitions,
            sim.field_work.transitions,
            n(wall_seconds),
            n(price_seconds),
            expected_passes,
            n(expected),
            l2_ok,
            n(first_shell),
            n(3.2),
            carrier_pairs,
            carrier_crossing,
            n(carrier),
            n(CARRIER_FLOOR),
            carrier_ok,
            match peak_raw {
                Some((_, h)) => n(h),
                None => "null".to_string(),
            },
            match peak_raw {
                Some((r, _)) => n(r),
                None => "null".to_string(),
            },
            n(plant_i_move),
            n(PLANT_MISS),
            plant_i_fires,
            n(r2_open),
            n(plant_ii_move),
            n(PLANT_MISS),
            plant_ii_fires,
            opt(&rdf_refusal),
            match dry_d {
                Some(d) => n(d),
                None => "null".to_string(),
            },
            opt(&dry_refusal),
            dry_max_lag,
            s("DRY ONLY: the oxygens sampled every frame instead of every hundredth, so the trajectory-and-diffusion path is exercised end to end on a short run. Not a reading.")
        ),
    );

    match &void {
        Some(v) => {
            write(out, "arm.void", format!("VOID at {} frame {}: {}\n", v.phase, v.frame, v.why));
            eprintln!("wrote arm.void");
        }
        None => {
            let note = if dry {
                "DRY: the instruments ran end to end. No reading here is a reading of anything (the law is refused for a counted arm and the frame counts are not the freeze's)."
            } else {
                "the counted arm ran to its staked frame count."
            };
            write(out, "arm.done", format!("{note}\n"));
            eprintln!("wrote arm.done");
        }
    }
    // The carrier is ASSERTED (§5, M-PLANT-SECTOR): plants on a sector with nothing in it
    // are not plants. It is asserted AFTER the record is written, so the record survives it.
    if !dry {
        assert!(carrier_ok, "the plants' carrier is {carrier:.4}, under the staked floor of {CARRIER_FLOOR}: the sector the plants act on is empty and neither plant is a plant");
    } else if !carrier_ok {
        eprintln!("WARNING (dry): the carrier reads {carrier:.4}, under the floor {CARRIER_FLOOR} — on the counted arm this is an assertion failure.");
    }
    println!("done");
}
