//! LIQUID-2 (`conformance/water_observatory/LIQUID2_PREREG.md`): the periodic liquid on
//! CT-3's served law, with the spanning-cluster fraction as a readout and the three
//! corrections LIQUID-1 owed.
//!
//! ```text
//! cargo run --release -p holon-render --example liquid2 -- screen [DIR]
//! cargo run --release -p holon-render --example liquid2 -- gate   [DIR]
//! cargo run --release -p holon-render --example liquid2 -- run    [DIR]
//! cargo run --release -p holon-render --example liquid2 -- read   [DIR]
//! ```
//!
//! This runner is written ON the harness (`holon-campaign`) and on the shared closure type
//! (`holon-closure`, through `holon_lens::closure`). It does not re-implement a gate, a
//! plant, a price, a stake or a record writer; `examples/liquid1.rs`, which built all five
//! by hand, is untouched and is the reference this was read against.
//!
//! # What is new here, and why
//!
//! 1. **CT-3's law.** `ct3/wall_ct3.json` with the transfer term served as the TABLE
//!    (`ct3/ct_table.json`, `CtMode::Table`), which is the term LIQUID-1 did not have.
//! 2. **The spanning-cluster fraction.** `holon_lens::closure::hbond_phase_checked`: the
//!    H-bond graph over the oxygens with the INTEGER wrap count on every edge, and spanning
//!    decided as a WINDING of the torus, the same rule the lattice uses.
//! 3. **The three corrections LIQUID-1 owed**, carried as design and not as prose:
//!    * R2's band restated in the lens's own each-bond-once convention;
//!    * R3's window PRICED from LIQUID-1's own measured MSD exponent, with the step chosen
//!      by a labelled screen and the arm length, the readout stride and the lag window all
//!      derived from that exponent rather than declared;
//!    * L1's drift bar as a FRACTION of the thermostat's posted work, the fraction read off
//!      LIQUID-1's own arm, never a fallback constant.
//!
//! Every number that is not a kill from experiment is either measured here or read out of a
//! record this engine wrote, with its citation kept (`holon_campaign::stake::ReadInput`).

use holon_campaign::{
    is_done, num, read_input_after, read_record, Gate, Plant, Price, Priced, ReadInput, Record,
    RecordWriter, Report, Stake,
};
use holon_closure::Phase;
use holon_lens::closure::hbond_phase_checked;
use holon_lens::lens::{diffusion, hbonds, hbonds_periodic, rdf_oo, LensRefusal};
use holon_lens::traj::{BondSet, Frame, Header, Trajectory, AU_TIME_FS};
use holon_render::channel::Row;
use holon_render::seam::{CtLoad, CtTable, SeamModel, CT_DIM};
use holon_render::sim::{Boundary, Sim, INTRA_UNIT_REACH, SEAM_REACH_BUDGET};
use holon_render::waterbox::{bohr2_per_fs_to_cm2_per_s, first_peak, liquid_box, BOHR_ANGSTROM};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::{scene, K_B};

// ------------------------------------------------------------------ the state point (LIQUID-1's)

/// The state point, CHOSEN and declared, the way a temperature is - LIQUID-1's, unchanged,
/// so the two arms are the same box asked the same question under a different law.
const DENSITY_G_CM3: f64 = 0.997;
const TEMPERATURE_K: f64 = 293.0;
/// THREE SEEDS, declared (M-FIXED-POINT-TRAJECTORY). The first is LIQUID-1's own, so one
/// arm of this campaign is the same trajectory that campaign ran, under a different law; the
/// other two are it plus one and plus two, which is a declaration and not a choice. Every
/// readout is reported with its spread across the three, and a band is met only if it
/// contains the SEED MEAN with that spread printed beside it.
const SEEDS: [u64; 3] = [0x4c49_5155_4944, 0x4c49_5155_4945, 0x4c49_5155_4946];
const N_CELLS: usize = 4;
const N_WATERS: usize = 128;

/// THE SIZE CHECK's boxes, as body-centred lattices of `n^3` cells carrying `2 n^3` waters:
/// 54, 250 and 432 against the campaign's 128. Each one's door is run and its force pass is
/// priced; nothing about the campaign's own box moves.
pub const SIZE_CELLS: [usize; 3] = [3, 5, 6];
const _: [usize; 3] = SIZE_CELLS;
/// Frames the size check times a force pass over. Short on purpose: it is a price, not an arm.
const SIZE_PRICE_FRAMES: usize = 20;

// ---- the equilibration criterion, and every number in it read off LIQUID-1's own arm
/// The settling readout cadence, in frames. At the pre-committed step this is `52.126` fs,
/// which is LIQUID-1's own 2,000-frame block to four decimal places - the criterion below was
/// VALIDATED on that record's blocks and runs here at the same physical cadence.
const SETTLE_READOUT: usize = 250;
/// Samples taken inside each settling block (LIQUID-1's blocks held 20).
const SETTLE_SAMPLES: usize = 10;
/// The window, in settling blocks. Validated on LIQUID-1's series: a 5-block window first
/// fires at its frame 26,000 = 678 fs, against a measured settling of 469 fs - late enough
/// not to fire early and not so late that it costs an arm.
const SETTLE_WINDOW: usize = 5;
/// The settling may not be counted as done before LIQUID-1's own MEASURED settling time in
/// physical units, converted at this arm's step. That number is DERIVED in `design` from
/// `liquid1/arm.log` and is deliberately not typed here.
/// The cap. Reaching it VOIDS the arm with the settling series recorded: a box that will not
/// settle is a finding, not something to count anyway.
const SETTLE_CAP: usize = 40_000;

/// LIQUID-1 Amendment 2's C2 switch on every seam term, `r_on = r_cut - 2`, chosen from the
/// half-edge 14.797 bohr with 0.8 bohr to spare. Unchanged here.
const SEAM_CUTOFF_BOHR: f64 = 14.0;
/// Amendment 2's stake on the truncation, per water, unchanged.
const TRUNCATION_STAKE_PER_WATER: f64 = 1.0e-5;

/// THE STEP, in multiples of the tables' own step, PRE-COMMITTED BY THE FREEZE and chosen by
/// the labelled screen of section 2 (the largest whose measured drift stays under the bar).
const STEP_MULT: f64 = 8.0;

/// The price is measured on the first 100 frames and written before the counted ones
/// (M-CHEAPER-THAN-ITS-PRICE).
const PRICE_FRAMES: usize = 100;
/// The frames the plants' carrier is measured over, after the settling and before any
/// counted frame.
const CARRIER_FRAMES: usize = 100;
/// The radial distribution's declared bin, LIQUID-1's.
const RDF_DR: f64 = 0.1;

/// The screen's own arm: equal for every step, so the drift comparison is at fixed frame
/// count and the step is the only thing that moved.
const SCREEN_SETTLE: usize = 1_000;
const SCREEN_COUNT: usize = 2_000;
/// The steps the screen sweeps, in multiples of the TABLES' own step (the one in force
/// under the exactness hold, which is LIQUID-1's `1.077481` au and not `dt_reference`).
/// Driven from the command line (`--step`), one arm per process, so the sweep runs in
/// parallel on its own cores; this is the list the freeze reports.
pub const SCREEN_STEPS: [f64; 4] = [1.0, 2.0, 4.0, 8.0];
const _: [f64; 4] = SCREEN_STEPS;

// ---- THE KILL BANDS. From EXPERIMENT, named as kills wherever they appear.
/// R1: water's first oxygen-oxygen peak, position, bohr (Soper 2000; Skinner et al. 2013).
const KILL_R1_POS: (f64, f64) = (5.0, 5.6);
/// R1: its height.
const KILL_R1_HEIGHT: (f64, f64) = (2.0, 4.0);
/// R2: hydrogen bonds per molecule, EXPERIMENT'S OWN both-ends number.
const KILL_R2_BOTH_ENDS: f64 = 3.5;
/// R2: the freeze's band in the both-ends convention (LIQUID-1's letter, kept for the record).
const KILL_R2_BOTH_ENDS_BAND: (f64, f64) = (3.0, 4.0);
/// R3: self-diffusion, cm^2/s (Krynicki-Green-Sawyer 1978; Mills 1973), within a factor of 3.
const KILL_R3_CENTRE: f64 = 2.3e-5;
const KILL_R3_FACTOR: f64 = 3.0;

/// The first shell, 3.2 Angstrom in bohr - converted through the builder's own bohr and
/// never typed as a second number.
fn first_shell_bohr() -> f64 {
    3.2 / BOHR_ANGSTROM
}
/// The plants' carrier floor. LIQUID-1's.
const CARRIER_FLOOR: f64 = 0.05;
/// Plants (i) and (ii) must move their reading by more than this fraction. LIQUID-1's.
const PLANT_MISS: f64 = 0.20;
/// Plant (iii): the spanning fraction must move by more than this when the wrap counts are
/// zeroed. Half of the whole range a fraction has.
const PLANT_SPAN_MISS: f64 = 0.50;

/// The angular grid the SERVED boundedness walk uses, CT-3's own.
const SERVED_GRID: usize = 41;

/// The margin the wall-saturation cap is kept under: the top fitted lag's MSD at
/// EXPERIMENT's own diffusion constant must be under `(L/4)^2` divided by this.
const WALL_CAP_MARGIN: f64 = 1.5;
/// The freeze's letter on the arm: the counted physical time is at least this many times the
/// crossover the measured exponent implies.
const CROSSOVER_MULTIPLE: f64 = 4.0;
/// R3 is VOID BY PRICE if the counted arm would cost more than this many times LIQUID-1's own
/// arm - the upper edge of the band LIQUID-1 priced itself in.
const PRICE_CEILING_MULTIPLE: f64 = 10.0;

// ------------------------------------------------------------------------ paths and reading

fn observatory(out: &Path) -> PathBuf {
    // the campaign directory's parent is the observatory; a runner invoked from anywhere
    // still resolves the sibling campaigns' records, and the path is printed in every
    // citation (M-STALE-INSTRUMENT).
    let up = out.parent().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    for c in [
        up.clone(),
        PathBuf::from("../conformance/water_observatory"),
        PathBuf::from("conformance/water_observatory"),
    ] {
        if c.join("ct3").join("wall_ct3.json").exists() {
            return c;
        }
    }
    up
}

fn must(r: Result<ReadInput, holon_campaign::ReadRefusal>) -> ReadInput {
    match r {
        Ok(v) => v,
        Err(e) => panic!("a stake cannot be derived: {e}"),
    }
}

/// LIQUID-1's MSD exponent. The lens wrote it into its own refusal sentence rather than into
/// a numeric field, so it is PARSED out of that sentence and carried with the same citation
/// every other read input carries. It is a number this engine wrote, not a number typed in.
fn liquid1_alpha(path: &str) -> ReadInput {
    let t = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let i = t.find("tau^").unwrap_or_else(|| panic!("{path}: no \"tau^\" in r3's refusal"));
    let rest = &t[i + 4..];
    let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(rest.len());
    let value = rest[..end]
        .parse::<f64>()
        .unwrap_or_else(|_| panic!("{path}: cannot read the exponent out of {:?}", &rest[..end]));
    ReadInput {
        path: path.to_string(),
        keys: vec!["\"r3\"".to_string()],
        field: "refusal (the MSD exponent, parsed from the lens's own sentence)".to_string(),
        value,
    }
}

// ------------------------------------------------------------------------- the seam law

/// Which law a phase runs. `Ct3Table` is the freeze's; the other two exist ONLY for the
/// labelled screen, which may turn any knob, and neither writes anything a gate reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Variant {
    /// CT-3's whole served law, channel 6 as the table. The freeze's law.
    Ct3Table,
    /// CT-3's law with channel 6 switched OFF entirely. The control that asks whether the
    /// transfer term is where a drift comes from.
    Ct3NoTransfer,
    /// CT-2's law, LIQUID-1's own. The control that asks whether THIS PIPELINE reproduces
    /// LIQUID-1's books on LIQUID-1's law.
    Ct2Family,
}

impl Variant {
    fn parse(s: &str) -> Variant {
        match s {
            "ct3" => Variant::Ct3Table,
            "ct3-noct" => Variant::Ct3NoTransfer,
            "ct2" => Variant::Ct2Family,
            other => panic!("unknown law variant {other:?}: ct3 | ct3-noct | ct2"),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Variant::Ct3Table => "ct3",
            Variant::Ct3NoTransfer => "ct3-noct",
            Variant::Ct2Family => "ct2",
        }
    }
}

struct Law {
    model: SeamModel,
    table: CtTable,
    variant: Variant,
    source: String,
    table_source: String,
    c0: f64,
    q_h: f64,
    kt: f64,
    r_min: [f64; 3],
    /// `Some` when the SERVED boundedness walk names a fall: no counted arm on this law.
    refusal: Option<String>,
}

fn json_num(t: &str, key: &str) -> f64 {
    t.split(&format!("\"{key}\": "))
        .nth(1)
        .and_then(|x| x.split(|c| c == ',' || c == '\n' || c == '}').next())
        .and_then(|x| x.trim().parse::<f64>().ok())
        .unwrap_or(f64::NAN)
}

/// The transfer table, built from `ct3/ct_table.json`'s own sites.
///
/// The record carries one entry per SITE - the freeze's pole rule has already merged the four
/// duplicate pairs of the acceptor's azimuth - so the loader pushes what the record holds and
/// merges nothing of its own. `c0` is the exponent the table divides out, the record's.
fn load_table(p: &Path) -> (CtTable, usize) {
    let t = std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let c0 = json_num(&t, "c0_per_bohr");
    let body = t.split("\"sites\": [").nth(1).unwrap_or_else(|| panic!("{}: no sites", p.display()));
    let mut knots: Vec<([f64; CT_DIM], f64)> = Vec::new();
    for chunk in body.split("{\"site\":").skip(1) {
        let y = [
            json_num(chunk, "r"),
            json_num(chunk, "cos_theta_d"),
            json_num(chunk, "u_dot_b"),
            json_num(chunk, "q"),
        ];
        let v = json_num(chunk, "e_ct");
        if y.iter().all(|x| x.is_finite()) && v.is_finite() {
            knots.push((y, v));
        }
    }
    let mut table = CtTable::empty();
    assert!(
        table.begin(knots.len(), c0),
        "the table refused {} knots at c0 = {c0} ({:?})",
        knots.len(),
        table.status
    );
    for (i, (y, v)) in knots.iter().enumerate() {
        assert!(table.knot(i, *y, *v), "knot {i} refused");
    }
    let status = table.finish();
    assert_eq!(status, CtLoad::Ok, "the transfer table did not load: {status:?}");
    (table, knots.len())
}

fn load_law(obs: &Path) -> Law {
    load_variant(obs, Variant::Ct3Table)
}

fn load_variant(obs: &Path, variant: Variant) -> Law {
    let p = match variant {
        Variant::Ct2Family => obs.join("ct2").join("wall_ct2.json"),
        _ => obs.join("ct3").join("wall_ct3.json"),
    };
    let tp = obs.join("ct3").join("ct_table.json");
    let t = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let g = |k: &str| {
        let v = json_num(&t, k);
        if v.is_finite() {
            v
        } else {
            0.0
        }
    };
    let (table, knots) = load_table(&tp);
    let model = SeamModel {
        a: g("a"),
        b: g("b"),
        p: g("p"),
        c: g("c"),
        c6: g("c6"),
        a_oh: g("a_oh"),
        b_oh: g("b_oh"),
        a_hh: g("a_hh"),
        b_hh: g("b_hh"),
        p_hh: g("p_hh"),
        c_hh: g("c_hh"),
        // the transfer row is the TABLE's; these are held at an exact zero by the record
        // itself so that nothing serves twice
        p_ct: g("p_ct"),
        c_ct: g("c_ct"),
        m_ct: g("m_ct") as u8,
        k_ct: g("k_ct") as u8,
        lambda_ct: g("lambda_ct"),
        r_cut: SEAM_CUTOFF_BOHR,
        ct_table_on: variant == Variant::Ct3Table,
    };
    // the two screen controls, each a DECLARED knob and nothing else
    let model = match variant {
        // channel 6 off entirely: no table, no pair term, no family
        Variant::Ct3NoTransfer => SeamModel { p_ct: 0.0, c_ct: 0.0, m_ct: 0, k_ct: 0, lambda_ct: 0.0, ..model },
        _ => model,
    };
    let q_h = holon_render::field::water_charge_at_pin();
    let kt = K_B * TEMPERATURE_K;
    let r_min = [g("r_min_oo"), g("r_min_oh"), g("r_min_hh")];
    eprintln!(
        "law {} ({} knots at c0 = {:.6}); deepest shape {:.6}, worst knot miss {:.3e}",
        p.display(),
        knots,
        table.c0,
        table.deepest_shape,
        table.worst_knot_miss
    );
    // THE SERVED BOUNDEDNESS WALK, CT-3's G-B0W: the whole law - the table plus the re-fit
    // contacts plus the three walls plus the charges - with the transfer row the interpolant
    // ITSELF at each walk radius, minimised over every angular geometry a real contact can
    // present. EVERY class and BOTH legs are named by the walk (M-FIRST-VIOLATION-ONLY).
    // The radii repeat along the walk, so the served minimum is memoised rather than
    // recomputed; the memo is keyed on the radius's bits and changes no value.
    let memo: std::cell::RefCell<BTreeMap<u64, f64>> = std::cell::RefCell::new(BTreeMap::new());
    let served = |r: f64| -> f64 {
        let k = r.to_bits();
        if let Some(v) = memo.borrow().get(&k) {
            return *v;
        }
        let v = table.deepest_served(r, SERVED_GRID).0;
        memo.borrow_mut().insert(k, v);
        v
    };
    let refusal = match variant {
        Variant::Ct3Table => model.bounded_ct(q_h, r_min, kt, &served),
        // the controls walk their own transfer row, which is the model's own function
        _ => model.bounded(q_h, r_min, kt),
    }
    .map(|why| {
        format!(
            "{}: the SERVED law is NOT BOUNDED - {why} (CT-3 G-B0W, the walk on the law as served)",
            p.display()
        )
    });
    Law {
        model,
        table,
        variant,
        source: p.display().to_string(),
        table_source: tp.display().to_string(),
        c0: g("c0_per_bohr"),
        q_h,
        kt,
        r_min,
        refusal,
    }
}

// ------------------------------------------------------------------------ the box

/// The box, the field, the seam law and the step. One builder for every phase, so a screen
/// and the counted arm differ in exactly the knobs the screen declares.
///
/// # THE STEP GOES THROUGH THE ENGINE'S OWN TOGGLE, and that is not a detail
///
/// `adopt_table_timescale` derives `dt_reference` from the O-H curve inside `scene`, and
/// `Timescale::hold_exactness` REWRITES `dt` from `dt_reference` every time the curvature
/// envelope widens - which `Sim::close_grain` makes happen on any frame where a pair reaches
/// an energy the envelope has not covered. Assigning `timescale.dt` directly therefore holds
/// for a few frames and is then silently undone, and a step sweep built that way would read
/// four arms at one step and call them four. The engine has ONE sanctioned way to run above
/// the reference step and it is rung (ii): `allow_dt_growth`, the explicit toggle, under
/// which `hold_exactness` returns at once and `set_dt_multiplier` sets
/// `dt = dt_reference * m` for good.
///
/// At `m = 1.0` the toggle is left OFF, so that arm is LIQUID-1's own integrator setting
/// exactly - the exactness hold still refines `dt` if the envelope stiffens. Every larger
/// step is rung (ii) by construction, the accuracy target is deliberately exceeded, and what
/// the freeze then measures is the drift the arm actually produced against a bar read off
/// LIQUID-1's own arm. That is the whole point of the sweep.
fn build(law: &Law, step_mult: f64, seed: u64) -> (Box<Sim>, Vec<[f64; 3]>, f64, f64, f64) {
    build_sized(law, step_mult, seed, N_CELLS)
}

fn build_sized(law: &Law, step_mult: f64, seed: u64, cells: usize) -> (Box<Sim>, Vec<[f64; 3]>, f64, f64, f64) {
    let (species, pos, l) = liquid_box(cells, DENSITY_G_CM3, seed);
    let mut sim: Box<Sim> = scene(&species, &pos, l, TEMPERATURE_K);
    let tables_reach = sim.legality_radius();
    sim.set_field(true, None).expect("the open box admits the field");
    if law.variant == Variant::Ct3Table {
        sim.ct_table = law.table.clone();
    }
    sim.set_seam(Some(law.model)).expect("no acuity frame is installed");
    // THE TABLES' STEP is the step IN FORCE, not `dt_reference`. On this box the exactness
    // hold has already refined the reference by a factor of four before any frame runs
    // (`dt_reference` 4.309924 au, `dt` 1.077481 au), and 1.077481 au is the step LIQUID-1
    // ran at. A sweep in units of `dt_reference` would be a sweep in units nothing else in
    // this programme speaks, so the multiplier handed to the engine is scaled back.
    let dt_ref = sim.timescale.dt_reference;
    let dt_tables = sim.dt();
    if step_mult != 1.0 {
        sim.timescale.allow_dt_growth = true;
        sim.timescale.set_dt_multiplier(step_mult * dt_tables / dt_ref);
        let want = dt_tables * step_mult;
        assert!(
            (sim.dt() - want).abs() <= 1e-12 * want,
            "the step did not take: {} against {want}",
            sim.dt()
        );
    }
    (sim, pos, l, tables_reach, dt_tables)
}

// ------------------------------------------------------------------------ the readouts

const PLANT_CELL: f64 = 1.0e6;

struct RdfAccum {
    r: Vec<f64>,
    g_sum: Vec<f64>,
    g_raw_sum: Vec<f64>,
    frames: usize,
    rho_o: f64,
    n_o: usize,
}

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

/// THE SPANNING-CLUSTER READOUT, and its plant beside it so the two can never be taken over
/// different frames. The plant is the SAME graph with every wrap count zeroed: an edge that
/// crossed a face is then an edge inside the box, and a winding cannot be exhibited.
struct PhaseAccum {
    frames: usize,
    spanning: usize,
    largest_sum: f64,
    degree_sum: f64,
    edges_sum: f64,
    crossing_edges: u64,
    all_edges: u64,
    /// The plant: the same frames with the displacements zeroed.
    spanning_local: usize,
    refusal: Option<String>,
    disagreement: usize,
}

impl PhaseAccum {
    fn new() -> Self {
        PhaseAccum {
            frames: 0,
            spanning: 0,
            largest_sum: 0.0,
            degree_sum: 0.0,
            edges_sum: 0.0,
            crossing_edges: 0,
            all_edges: 0,
            spanning_local: 0,
            refusal: None,
            disagreement: 0,
        }
    }
    fn push(&mut self, pos: &[[f64; 3]], z: &[u32], cell: [f64; 3]) {
        match hbond_phase_checked(pos, z, cell) {
            Ok((p, largest_domain)) => {
                if p.largest != largest_domain {
                    self.disagreement += 1;
                }
                self.frames += 1;
                if p.largest_spans {
                    self.spanning += 1;
                }
                self.largest_sum += p.largest_fraction;
                self.degree_sum += p.degree;
                self.edges_sum += p.edges as f64;
                // the plant, and the carrier it acts on, on the same frame
                let bonds = match hbonds_periodic(pos, z, cell) {
                    Ok(b) => b,
                    Err(e) => {
                        self.refusal = Some(format!("{}: {}", e.lens, e.reason));
                        return;
                    }
                };
                let oxy = holon_lens::closure::oxygen_index(z);
                let edges = holon_lens::closure::hbond_edges(&bonds, pos, &oxy, cell);
                let mut local = edges.clone();
                for e in local.iter_mut() {
                    e.displacement = [0; 3];
                }
                for e in edges.iter() {
                    self.all_edges += 1;
                    if e.displacement.iter().any(|&d| d != 0) {
                        self.crossing_edges += 1;
                    }
                }
                let q: Phase<3> = holon_closure::phase(oxy.len(), &local);
                if q.largest_spans {
                    self.spanning_local += 1;
                }
            }
            Err(e) => self.refusal = Some(format!("{}: {}", e.lens, e.reason)),
        }
    }
    fn spanning_fraction(&self) -> f64 {
        if self.frames == 0 {
            f64::NAN
        } else {
            self.spanning as f64 / self.frames as f64
        }
    }
    fn spanning_fraction_local(&self) -> f64 {
        if self.frames == 0 {
            f64::NAN
        } else {
            self.spanning_local as f64 / self.frames as f64
        }
    }
    fn largest_fraction(&self) -> f64 {
        if self.frames == 0 {
            f64::NAN
        } else {
            self.largest_sum / self.frames as f64
        }
    }
    fn degree(&self) -> f64 {
        if self.frames == 0 {
            f64::NAN
        } else {
            self.degree_sum / self.frames as f64
        }
    }
    fn crossing_fraction(&self) -> f64 {
        if self.all_edges == 0 {
            f64::NAN
        } else {
            self.crossing_edges as f64 / self.all_edges as f64
        }
    }
    /// `blind` omits every field from which the BOND COUNT could be recovered — the degree,
    /// the edge count and the edge total are `2 x` R2's own reading — so the gate phase can
    /// take plant (iii)'s carrier, which needs this graph, without the campaign having seen
    /// R2 before its arm. The counted arm writes the full form.
    fn json(&self, blind: bool) -> String {
        if blind {
            return format!(
                "{{\"blind\": true, \"why\": \"the degree and the edge counts are R2's own reading and are withheld from the gate phase so that R2 stays a forward prediction; the counted arm writes them\", \"frames\": {}, \"spanning_frames\": {}, \"spanning_fraction\": {}, \"largest_component_fraction\": {}, \"union_find_disagreements_with_largest_domain\": {}, \"edges_crossing_a_face_fraction\": {}, \"plant_spanning_fraction_wraps_zeroed\": {}, \"refusal\": {}}}",
                self.frames,
                self.spanning,
                num(self.spanning_fraction()),
                num(self.largest_fraction()),
                self.disagreement,
                num(self.crossing_fraction()),
                num(self.spanning_fraction_local()),
                match &self.refusal { Some(w) => format!("{w:?}"), None => "null".to_string() }
            );
        }
        format!(
            "{{\"blind\": false, \"frames\": {}, \"spanning_frames\": {}, \"spanning_fraction\": {}, \
             \"largest_component_fraction\": {}, \"mean_degree_both_ends\": {}, \
             \"mean_edges\": {}, \"union_find_disagreements_with_largest_domain\": {}, \
             \"edges_total\": {}, \"edges_crossing_a_face\": {}, \"edge_crossing_fraction\": {}, \
             \"plant_spanning_fraction_wraps_zeroed\": {}, \"refusal\": {}}}",
            self.frames,
            self.spanning,
            num(self.spanning_fraction()),
            num(self.largest_fraction()),
            num(self.degree()),
            num(if self.frames == 0 { f64::NAN } else { self.edges_sum / self.frames as f64 }),
            self.disagreement,
            self.all_edges,
            self.crossing_edges,
            num(self.crossing_fraction()),
            num(self.spanning_fraction_local()),
            match &self.refusal {
                Some(w) => format!("{w:?}"),
                None => "null".to_string(),
            }
        )
    }
}

fn band(x: f64, lo: f64, hi: f64) -> bool {
    x >= lo && x <= hi
}

/// The lens's own lag ladder, reproduced so the window can be designed against it:
/// `msd_exponent` walks `lag = 2` then `ceil(1.5 lag)` while `lag <= max_lag`.
fn lag_ladder(max_lag: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut lag = 2usize;
    while lag <= max_lag {
        out.push(lag);
        lag = (lag as f64 * 1.5).ceil() as usize;
    }
    out
}

// ------------------------------------------------- LIQUID-1's own settling, read off its log

/// LIQUID-1's temperature and bond series, read out of `liquid1/arm.log`.
///
/// The log is the only place that series exists — `arm.json` carries the whole-arm means and
/// not the trajectory of them — so it is parsed here, with the same citation discipline every
/// other read input gets: the path, the field, and the number. Nothing is fitted: an
/// exponential fitted to this series runs to its grid edges in both parameters, so the
/// campaign reports what it MEASURED (the plateau, its scatter, and the first frame from which
/// every later reading stays inside it) and not a relaxation constant the data will not carry.
struct L1Series {
    /// The bond count per BLOCK, recovered from the running mean.
    blocks: Vec<(usize, f64)>,
    t_plateau: f64,
    t_plateau_sd: f64,
    bond_plateau: f64,
    bond_plateau_sd: f64,
    /// The first frame from which every LATER reading stays inside the plateau's own 3 sd.
    t_settled_at: usize,
    bond_settled_at: usize,
    /// The first frame at which THIS FREEZE'S criterion fires, run on LIQUID-1's own blocks.
    criterion_fires_at: usize,
    /// The whole-arm bond mean LIQUID-1 reported, and how far under its own plateau it sits.
    reported_bond: f64,
    plateau_shortfall: f64,
}

fn mean_sd(v: &[f64]) -> (f64, f64) {
    let n = v.len();
    if n == 0 {
        return (f64::NAN, f64::NAN);
    }
    let m = v.iter().sum::<f64>() / n as f64;
    if n < 2 {
        return (m, 0.0);
    }
    (m, (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1) as f64).sqrt())
}

/// The window test this freeze pre-commits, as one function, so the criterion that runs on
/// LIQUID-1's blocks and the criterion that runs on LIQUID-2's arm are the SAME code.
///
/// `temps` are every temperature sample in the window and every one must be inside the band.
/// `blocks` are the window's per-block means of the SETTLING VARIABLE and their least-squares
/// trend across the window must be under the window's own scatter: the variable has stopped
/// moving within its own noise.
///
/// **THE SETTLING VARIABLE IS NOT A READOUT ANY STAKE READS.** An earlier version of this
/// criterion used the bond count, which is exactly what R2 reads and what S's graph is built
/// from, so equilibrating on it would have made R2 a quantity the gate had already seen. The
/// variable is the CROSS-UNIT potential energy (`Row::Field + Row::Seam`, the same quantity
/// section 1's expectation is written in): it is the thermodynamically slow part of the
/// energy, the intramolecular vibration is excluded from it by construction, and R1's
/// histogram, R2's census, R3's displacement and S's graph none of them read it.
fn settled_window(temps: &[f64], blocks: &[f64], band: f64) -> bool {
    if temps.is_empty() || blocks.len() < 2 {
        return false;
    }
    if temps.iter().any(|x| (x - TEMPERATURE_K).abs() > band) {
        return false;
    }
    let w = blocks.len();
    let (my, sd) = mean_sd(blocks);
    let mx = (w - 1) as f64 / 2.0;
    let num: f64 = blocks.iter().enumerate().map(|(i, b)| (i as f64 - mx) * (b - my)).sum();
    let den: f64 = (0..w).map(|i| (i as f64 - mx) * (i as f64 - mx)).sum();
    if den == 0.0 {
        return false;
    }
    (num / den).abs() * (w - 1) as f64 <= sd
}

fn read_l1_series(path: &str, readout_stride: f64, band: f64, reported: f64) -> L1Series {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let cut = |s: &str, a: &str, b: &str| -> Option<f64> {
        let i = s.find(a)? + a.len();
        let rest = &s[i..];
        let j = rest.find(b)?;
        rest[..j].trim().parse::<f64>().ok()
    };
    let mut rows: Vec<(usize, f64, f64)> = Vec::new();
    for line in text.lines() {
        if !line.contains("frame") || !line.contains(" K, units ") {
            continue;
        }
        let f = cut(line, "frame", ":").map(|x| x as usize);
        let tt = cut(line, ": T", " K,");
        let bb = cut(line, "H-bonds/molecule", ",");
        if let (Some(f), Some(tt), Some(bb)) = (f, tt, bb) {
            rows.push((f, tt, bb));
        }
    }
    assert!(rows.len() > 10, "{path}: only {} logged readouts", rows.len());
    // the logged bond number is a RUNNING mean over readouts; recover the per-block mean
    let mut blocks: Vec<(usize, f64)> = Vec::new();
    let (mut pn, mut pm) = (0.0f64, 0.0f64);
    for &(f, _, m) in rows.iter() {
        let n = f as f64 / readout_stride;
        blocks.push((f, (n * m - pn * pm) / (n - pn)));
        pn = n;
        pm = m;
    }
    // the plateau is the SECOND HALF of the counted arm, a declared cut and not a fitted one
    let last = rows.last().unwrap().0;
    let half = last / 2;
    let (tp, tsd) = mean_sd(&rows.iter().filter(|r| r.0 >= half).map(|r| r.1).collect::<Vec<_>>());
    let (bp, bsd) = mean_sd(&blocks.iter().filter(|r| r.0 >= half).map(|r| r.1).collect::<Vec<_>>());
    let settled_from = |v: &[(usize, f64)], mu: f64, sd: f64| -> usize {
        for k in 0..v.len() {
            if v[k..].iter().all(|(_, y)| (y - mu).abs() <= 3.0 * sd) {
                return v[k].0;
            }
        }
        v.last().map(|x| x.0).unwrap_or(0)
    };
    let t_settled = settled_from(&rows.iter().map(|r| (r.0, r.1)).collect::<Vec<_>>(), tp, tsd);
    let bond_settled = settled_from(&blocks, bp, bsd);
    // this freeze's own criterion, run on LIQUID-1's blocks
    let mut fires = blocks.last().map(|x| x.0).unwrap_or(0);
    for k in (SETTLE_WINDOW - 1)..blocks.len() {
        let temps: Vec<f64> = rows[k + 1 - SETTLE_WINDOW..=k].iter().map(|r| r.1).collect();
        // LIQUID-1's log carries NO energy column, so what can be validated on it is the
        // TEMPERATURE leg alone. That is enough to place the criterion: the legs are a
        // CONJUNCTION, so a criterion whose temperature leg first holds at frame F cannot fire
        // before F, whatever its second leg is. The second leg is validated on this campaign's
        // own arm instead, and where it fires is recorded there.
        let flat: Vec<f64> = vec![0.0; SETTLE_WINDOW];
        if settled_window(&temps, &flat, band) {
            fires = blocks[k].0;
            break;
        }
    }
    L1Series {
        blocks,
        t_plateau: tp,
        t_plateau_sd: tsd,
        bond_plateau: bp,
        bond_plateau_sd: bsd,
        t_settled_at: t_settled,
        bond_settled_at: bond_settled,
        criterion_fires_at: fires,
        reported_bond: reported,
        plateau_shortfall: (bp - reported) / bp,
    }
}

/// The settling, live: block means of the cross-unit potential energy and every temperature
/// sample, with the window test above deciding when the counted frames may begin. The bond
/// count is neither sampled nor recorded here, so R2 stays a forward prediction.
struct Settler {
    band: f64,
    floor_frames: usize,
    temps: Vec<f64>,
    blocks: Vec<f64>,
    /// the samples of the block being filled
    cur_t: Vec<f64>,
    cur_b: Vec<f64>,
    series: Vec<(usize, f64, f64)>,
}

impl Settler {
    fn new(band: f64, floor_frames: usize) -> Settler {
        Settler { band, floor_frames, temps: Vec::new(), blocks: Vec::new(), cur_t: Vec::new(), cur_b: Vec::new(), series: Vec::new() }
    }
    fn sample(&mut self, t: f64, b: f64) {
        self.cur_t.push(t);
        self.cur_b.push(b);
    }
    /// Close the block at `frame`. Returns whether the box is settled.
    fn close_block(&mut self, frame: usize) -> bool {
        if self.cur_b.is_empty() {
            return false;
        }
        let (tm, _) = mean_sd(&self.cur_t);
        let (bm, _) = mean_sd(&self.cur_b);
        self.temps.extend(self.cur_t.drain(..));
        self.blocks.push(bm);
        self.cur_b.clear();
        self.series.push((frame, tm, bm));
        self.settled(frame)
    }
    fn settled(&self, frames_done: usize) -> bool {
        if frames_done < self.floor_frames || self.blocks.len() < SETTLE_WINDOW {
            return false;
        }
        let nb = self.blocks.len();
        let bs = &self.blocks[nb - SETTLE_WINDOW..];
        let nt = self.temps.len();
        let take = (SETTLE_WINDOW * SETTLE_SAMPLES).min(nt);
        let ts = &self.temps[nt - take..];
        settled_window(ts, bs, self.band)
    }
    fn json(&self) -> String {
        format!(
            "[{}]",
            self.series
                .iter()
                .map(|(f, t, u)| format!("{{\"frame\": {f}, \"temperature_k\": {}, \"cross_unit_potential_per_water_hartree\": {}}}", num(*t), num(*u)))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// THE SETTLING, RUN. One routine for the gate and for every seed's arm, so the criterion
/// that prices the settling and the criterion that ends it are the same code on the same
/// samples. Returns the frames used, the settler (which carries the series) and whether the
/// cap was reached.
fn settle(
    sim: &mut Sim,
    z: &[u32],
    cell: [f64; 3],
    d: &Design,
    already_done: usize,
) -> (usize, Settler, bool) {
    let _ = (z, cell);
    let mut s = Settler::new(d.temp_band_k, d.settle_floor_frames);
    let every = (SETTLE_READOUT / SETTLE_SAMPLES).max(1);
    let mut frame = already_done;
    loop {
        for k in 0..SETTLE_READOUT {
            sim.step_frame(1);
            frame += 1;
            if (k + 1) % every == 0 {
                // the cross-unit potential energy per water: the settling variable, and NOT a
                // quantity any stake reads
                let u = (sim.row(Row::Field) + sim.row(Row::Seam)) / N_WATERS as f64;
                s.sample(sim.temperature(), u);
            }
        }
        let done = s.close_block(frame);
        if let Some((_, tm, bm)) = s.series.last() {
            if s.series.len() % 4 == 0 || done {
                eprintln!(
                    "  settling block {:>4} at frame {frame:>6}: T {tm:6.1} K, cross-unit U {bm:.6e} Ha per water{}",
                    s.series.len(),
                    if done { "  -> SETTLED" } else { "" }
                );
            }
        }
        if done {
            return (frame, s, false);
        }
        if frame >= SETTLE_CAP {
            return (frame, s, true);
        }
    }
}

// ------------------------------------------- the argmin handover counter (the drift diagnosis)

/// THE SUSPECT, INSTRUMENTED. CT-3 serves channel 6 ONCE per unordered pair of units, at that
/// pair's SHORTEST cross-unit H...O contact. An argmin is discontinuous where it ties: when the
/// shortest contact hands over from one hydrogen to another, the served energy jumps by the
/// difference of the table at the two coordinate points, evaluated at one geometry.
///
/// CT-3 measured that jump on ONE dimer over a full turn of the donor and read `1.418e-5`
/// hartree, small against its own refusal floor. This counter asks the question the liquid
/// asks instead: how many handovers happen per frame in a 128-water box, and do their jumps
/// account for the measured drift?
///
/// It is a DIAGNOSTIC and it changes no force: the arm it runs on integrates exactly the same
/// trajectory with it as without it (the counter reads positions and never writes one).
struct Handovers {
    /// The chosen hydrogen per unordered oxygen pair on the previous frame; `u32::MAX` for a
    /// pair with no served contact.
    prev: Vec<u32>,
    frames: usize,
    events: u64,
    /// The sum of `|jump|` over every handover: the total energy the discontinuity moved.
    abs_sum: f64,
    /// The signed sum: a random walk, and the quantity a drift would actually accumulate.
    signed_sum: f64,
    /// The largest single jump seen.
    worst: f64,
    /// The running signed sum's own extremum, which is what a drift PEAK would be if the
    /// handovers were the whole of it.
    signed_peak: f64,
    pairs_served: u64,
}

impl Handovers {
    fn new(n_ox: usize) -> Handovers {
        Handovers {
            prev: vec![u32::MAX; n_ox * n_ox],
            frames: 0,
            events: 0,
            abs_sum: 0.0,
            signed_sum: 0.0,
            worst: 0.0,
            signed_peak: 0.0,
            pairs_served: 0,
        }
    }

    /// One frame. `unit_h[o]` are the two hydrogens of the unit named by oxygen `o`.
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        pos: &[[f64; 3]],
        oxy: &[usize],
        unit_h: &[[usize; 2]],
        l: f64,
        model: &SeamModel,
        table: &CtTable,
    ) {
        let n = oxy.len();
        let mind = |a: [f64; 3], b: [f64; 3]| -> ([f64; 3], f64) {
            let mut d = [0.0f64; 3];
            let mut d2 = 0.0;
            for c in 0..3 {
                let mut x = b[c] - a[c];
                x -= l * (x / l).round();
                d[c] = x;
                d2 += x * x;
            }
            (d, d2.sqrt())
        };
        // the served energy of ONE named contact, at this frame's geometry
        let served = |hi: usize, oa: usize, od: usize| -> f64 {
            let pa = pos[oa];
            let rel = |k: usize| mind(pa, pos[k]).0;
            let (_, r) = mind(pa, pos[hi]);
            let (sw, _) = model.switch(r);
            if sw == 0.0 {
                return 0.0;
            }
            let [a1, a2] = unit_h[oa];
            sw * table.serve(rel(hi), [0.0; 3], rel(od), rel(a1), rel(a2)).0
        };
        for ia in 0..n {
            let ua = oxy[ia];
            for ib in (ia + 1)..n {
                let ub = oxy[ib];
                // the argmin, exactly as `Sim::accumulate_seam` takes it
                let mut rmin = f64::INFINITY;
                let (mut hi, mut oa, mut od) = (unit_h[ua][0], ub, ua);
                for (dn, ac) in [(ua, ub), (ub, ua)] {
                    for &h in unit_h[dn].iter() {
                        let (_, d) = mind(pos[ac], pos[h]);
                        if d < rmin {
                            rmin = d;
                            hi = h;
                            oa = ac;
                            od = dn;
                        }
                    }
                }
                let (sw, _) = model.switch(rmin);
                let key = ia * n + ib;
                let now = if sw == 0.0 { u32::MAX } else { hi as u32 };
                if now != u32::MAX {
                    self.pairs_served += 1;
                }
                let was = self.prev[key];
                if was != u32::MAX && now != u32::MAX && was != now {
                    // the handover's jump, both contacts read at THIS frame's geometry
                    let old_h = was as usize;
                    // the old contact's own direction: the hydrogen names its donor unit
                    let old_od = if unit_h[ua].contains(&old_h) { ua } else { ub };
                    let old_oa = if old_od == ua { ub } else { ua };
                    let jump = served(hi, oa, od) - served(old_h, old_oa, old_od);
                    self.events += 1;
                    self.abs_sum += jump.abs();
                    self.signed_sum += jump;
                    if jump.abs() > self.worst {
                        self.worst = jump.abs();
                    }
                    if self.signed_sum.abs() > self.signed_peak {
                        self.signed_peak = self.signed_sum.abs();
                    }
                }
                self.prev[key] = now;
            }
        }
        self.frames += 1;
    }

    fn per_frame(&self) -> f64 {
        if self.frames == 0 {
            f64::NAN
        } else {
            self.events as f64 / self.frames as f64
        }
    }
    fn mean_jump(&self) -> f64 {
        if self.events == 0 {
            f64::NAN
        } else {
            self.abs_sum / self.events as f64
        }
    }
    /// The random-walk estimate: `sqrt(N) * <|jump|>`, what an unbiased sum of `N` independent
    /// jumps of that size would reach.
    fn random_walk(&self) -> f64 {
        (self.events as f64).sqrt() * self.mean_jump()
    }
    fn json(&self) -> String {
        format!(
            "{{\"frames\": {}, \"handovers\": {}, \"handovers_per_frame\": {}, \
             \"pair_contacts_served_per_frame\": {}, \"mean_absolute_jump_hartree\": {}, \
             \"worst_jump_hartree\": {}, \"sum_of_absolute_jumps_hartree\": {}, \
             \"signed_sum_hartree\": {}, \"signed_running_peak_hartree\": {}, \
             \"random_walk_estimate_hartree\": {}, \
             \"ct3_g_a0_single_dimer_jump_hartree\": 1.418000000e-5}}",
            self.frames,
            self.events,
            num(self.per_frame()),
            num(if self.frames == 0 { f64::NAN } else { self.pairs_served as f64 / self.frames as f64 }),
            num(self.mean_jump()),
            num(self.worst),
            num(self.abs_sum),
            num(self.signed_sum),
            num(self.signed_peak),
            num(self.random_walk())
        )
    }
}

// --------------------------------------------------------------- the derived design

/// Everything the freeze derives from LIQUID-1's own record and this engine's own step.
struct Design {
    inputs: Vec<ReadInput>,
    /// LIQUID-1's arm: its exponent, its lag window and the crossover they imply.
    alpha: f64,
    t1_l1_fs: f64,
    t2_l1_fs: f64,
    crossover_fs: f64,
    /// This arm: the step, the stride, the lag window, the frames.
    dt_tables_au: f64,
    step_au: f64,
    step_fs: f64,
    stride: usize,
    max_lag: usize,
    sampled: usize,
    count: usize,
    t1_fs: f64,
    t2_fs: f64,
    physical_fs: f64,
    /// The wall-saturation check, at EXPERIMENT's own diffusion constant.
    wall_cap_bohr2: f64,
    msd_at_top_bohr2: f64,
    /// The drift bar, as a fraction of the thermostat's posted work.
    drift_fraction: f64,
    /// The price ceiling, and LIQUID-1's own arm beside it.
    liquid1_seconds: f64,
    liquid1_seconds_per_pass: f64,
    price_ceiling_seconds: f64,
    /// LIQUID-1's bond count, cited beside R2's restated band.
    liquid1_hbonds: f64,
    /// The equilibration criterion, every number of it off LIQUID-1's own arm.
    temp_band_k: f64,
    equipartition_sigma_k: f64,
    settle_floor_frames: usize,
    l1: L1Series,
    /// The Erdos-Renyi giant component at LIQUID-1's own measured degree.
    er_degree: f64,
    er_giant: f64,
}

/// The Erdos-Renyi giant component fraction at mean degree `z`: the fixed point of
/// `s = 1 - exp(-z s)`, iterated from 1. Zero for `z <= 1`, where there is no giant.
fn er_giant(z: f64) -> f64 {
    if !(z > 1.0) {
        return 0.0;
    }
    let mut s = 1.0f64;
    for _ in 0..10_000 {
        let t = 1.0 - (-z * s).exp();
        if (t - s).abs() < 1e-15 {
            return t;
        }
        s = t;
    }
    s
}

fn design(obs: &Path, dt_tables: f64, l: f64) -> Design {
    let arm = obs.join("liquid1").join("arm.json");
    let a = arm.display().to_string();
    let r = |keys: &[&str], f: &str| must(read_input_after(&a, keys, f));
    let alpha_in = liquid1_alpha(&a);
    let max_lag_l1 = r(&["\"r3\""], "max_lag_frames");
    let counted_l1 = r(&[], "counted_frames_run");
    let readouts_l1 = r(&[], "readouts");
    let drift_in = r(&["\"l1\""], "drift_peak");
    let therm_in = r(&["\"l1\""], "work_thermostat");
    let secs_in = r(&["\"l2\""], "wall_seconds");
    let per_pass_in = r(&["\"l2\""], "seconds_per_pass");
    let hb_in = r(&["\"r2\""], "hbonds_per_molecule");

    // LIQUID-1's own lag window, in femtoseconds. Its readout stride is its counted frames
    // over its readouts; its step is the tables' step, which this engine derives again from
    // the same curve and the same box (`adopt_table_timescale`).
    let stride_l1 = (counted_l1.value / readouts_l1.value).round();
    let ladder_l1 = lag_ladder(max_lag_l1.value as usize);
    let low_l1 = *ladder_l1.first().unwrap_or(&2) as f64;
    let high_l1 = *ladder_l1.last().unwrap_or(&2) as f64;
    let t1_l1 = low_l1 * stride_l1 * dt_tables * AU_TIME_FS;
    let t2_l1 = high_l1 * stride_l1 * dt_tables * AU_TIME_FS;
    // THE CROSSOVER THE EXPONENT IMPLIES. Two powers, ballistic below and diffusive above,
    // joined continuously: MSD ~ t^2 for t < t_c and ~ t_c t for t > t_c. A log-log slope
    // over [t1, t2] is then alpha = 1 + ln(t_c/t1)/ln(t2/t1) exactly, so
    // t_c = t1 * (t2/t1)^(alpha-1). At alpha = 1 it returns t1 and at alpha = 2 it returns
    // t2, which are the two facts a crossover estimate has to get right.
    let alpha = alpha_in.value;
    let t_c = t1_l1 * (t2_l1 / t1_l1).powf(alpha - 1.0);

    // THIS ARM. The step is the freeze's pre-committed multiple of the tables' step.
    let step_au = dt_tables * STEP_MULT;
    let step_fs = step_au * AU_TIME_FS;
    // (i) the lowest fitted lag sits AT OR ABOVE the crossover, which puts the whole fit
    // window on the diffusive side and makes the model's own effective exponent exactly 1.
    let stride = (t_c / (2.0 * step_fs)).ceil().max(1.0) as usize;
    // (ii) the top fitted lag is the largest on the lens's own ladder whose MSD at
    // EXPERIMENT's diffusion constant stays under the lens's wall-saturation cap with a
    // stated margin. The cap is the lens's, `(L_min/4)^2`.
    let cap = (l / 4.0).powi(2);
    let d_exp_bohr2_per_fs = KILL_R3_CENTRE / bohr2_per_fs_to_cm2_per_s();
    let t2_max = cap / WALL_CAP_MARGIN / (6.0 * d_exp_bohr2_per_fs);
    let lag_max_allowed = t2_max / (stride as f64 * step_fs);
    let max_lag = *lag_ladder(1_000_000)
        .iter()
        .filter(|&&lg| (lg as f64) <= lag_max_allowed)
        .last()
        .unwrap_or(&2);
    // (iii) the lens's caller convention, LIQUID-1's: `max_lag = sampled/4`.
    let sampled = 4 * max_lag;
    let count = sampled * stride;
    let ladder = lag_ladder(max_lag);
    let t1 = *ladder.first().unwrap_or(&2) as f64 * stride as f64 * step_fs;
    let t2 = *ladder.last().unwrap_or(&2) as f64 * stride as f64 * step_fs;
    let physical = count as f64 * step_fs;
    let msd_top = 6.0 * d_exp_bohr2_per_fs * t2;

    // ---- the equilibration criterion (the addendum's item 1), derived and cross-checked
    //
    // The BAND is three times the scatter of LIQUID-1's own temperature readout over the
    // second half of its counted arm - the box's own measured fluctuation, not a chosen
    // tolerance - and it is cross-checked against the analytic one: for `3N - 3` degrees of
    // freedom the canonical temperature fluctuates by `T sqrt(2/(3N-3))`, which at 384 atoms
    // and 293 K is 12.22 K against the measured 7.01 K per reading. The measured scatter is
    // SMALLER than the analytic one, as a thermostatted box's must be, so the band is stated
    // in what was measured and the analytic value is reported beside it as the check that the
    // scatter is the box's fluctuation and not a residual drift.
    let arm_log = obs.join("liquid1").join("arm.log").display().to_string();
    let ndof = (3 * 3 * N_WATERS - 3) as f64;
    let equipartition = TEMPERATURE_K * (2.0 / ndof).sqrt();
    // a first pass to get the plateau scatter, then the band, then the criterion on it
    let probe = read_l1_series(&arm_log, stride_l1, f64::INFINITY, hb_in.value);
    let band = 3.0 * probe.t_plateau_sd;
    let l1 = read_l1_series(&arm_log, stride_l1, band, hb_in.value);
    // THE FLOOR is the PHYSICAL-TIME conversion, and the frame conversion is recorded beside
    // it without being used. Both were considered and the choice is measured, not assumed.
    //
    // Two clocks run here. The network's own rearrangement is a process in physical time, so
    // LIQUID-1's measured settling converts at the step ratio: 18,000 of its frames is 469.1
    // fs is 2,250 of this arm's. The THERMOSTAT is not: it is applied once per step, so its
    // relaxation is counted in FRAMES and a larger step buys physical time without buying any
    // faster cooling — measured on this campaign's own instrumented settling, where the box is
    // still at 347 K at frame 1,100 while LIQUID-1 at the same PHYSICAL time was already at
    // 315 K.
    //
    // The floor was briefly written as the larger of the two, on the premise that the
    // criterion might fire before the thermostat had finished. THAT PREMISE IS DEAD and the
    // reversal is recorded here rather than quietly applied: the criterion's own temperature
    // leg IS the thermostat test, and on the gate's run it fired at 11,850 frames — 5.3 times
    // LIQUID-1's physical settling and two thirds of its settling in frames — with both legs
    // satisfied. A floor of 18,000 would have overridden a measurement with a guard, which is
    // the opposite of what a floor is for. The frame conversion is written to the record so a
    // reader can see both numbers.
    let l1_settle_fs = l1.bond_settled_at as f64 * dt_tables * AU_TIME_FS;
    let settle_floor = (l1_settle_fs / step_fs).ceil() as usize;

    let z = 2.0 * hb_in.value;
    Design {
        inputs: vec![
            alpha_in,
            max_lag_l1,
            counted_l1,
            readouts_l1,
            drift_in.clone(),
            therm_in.clone(),
            secs_in.clone(),
            per_pass_in.clone(),
            hb_in.clone(),
        ],
        alpha,
        t1_l1_fs: t1_l1,
        t2_l1_fs: t2_l1,
        crossover_fs: t_c,
        dt_tables_au: dt_tables,
        step_au,
        step_fs,
        stride,
        max_lag,
        sampled,
        count,
        t1_fs: t1,
        t2_fs: t2,
        physical_fs: physical,
        wall_cap_bohr2: cap,
        msd_at_top_bohr2: msd_top,
        drift_fraction: drift_in.value.abs() / therm_in.value.abs(),
        liquid1_seconds: secs_in.value,
        liquid1_seconds_per_pass: per_pass_in.value,
        price_ceiling_seconds: PRICE_CEILING_MULTIPLE * secs_in.value,
        liquid1_hbonds: hb_in.value,
        temp_band_k: band,
        equipartition_sigma_k: equipartition,
        settle_floor_frames: settle_floor,
        l1,
        er_degree: z,
        er_giant: er_giant(z),
    }
}

impl Design {
    fn drift_bar(&self, work_thermostat: f64) -> f64 {
        self.drift_fraction * work_thermostat.abs()
    }
    fn stakes(&self) -> Vec<Stake> {
        let i = &self.inputs;
        vec![
            Stake::derived(
                "L1 drift bar as a fraction of the thermostat's posted work",
                self.drift_fraction,
                &i[4..6],
                "LIQUID-1's own arm: drift_peak / |work_thermostat| = 1.076398662e-5 / 7.669546662e-1. \
                 The bar on any arm is this fraction times THAT arm's own posted thermostat work, so \
                 LIQUID-1 sits exactly on the bar and LIQUID-2 is asked to be no worse per unit of \
                 work pumped. FENCE: drift_peak is an extremum and the thermostat column accumulates, \
                 so this ratio loosens with arm length; the screen's arms are all the same length for \
                 that reason and the counted arm's length is reported beside it.",
            ),
            Stake::derived(
                "R3 crossover implied by LIQUID-1's measured MSD exponent, fs",
                self.crossover_fs,
                &i[0..4],
                "two powers joined continuously (ballistic t^2 below t_c, diffusive t_c*t above): a \
                 log-log slope over [t1, t2] is alpha = 1 + ln(t_c/t1)/ln(t2/t1), so \
                 t_c = t1 * (t2/t1)^(alpha-1). LIQUID-1's window is its own lag ladder \
                 (2 .. 210 of its readout stride = counted_frames_run / readouts) times the tables' \
                 step measured again here.",
            ),
            Stake::derived(
                "R3 price ceiling, seconds",
                self.price_ceiling_seconds,
                &i[6..7],
                "ten times LIQUID-1's own counted arm (l2.wall_seconds), which is the upper edge of \
                 the band that arm priced itself in. A counted arm over this is VOID BY PRICE and R3 \
                 with it.",
            ),
            Stake::derived(
                "S: the Erdos-Renyi giant component at LIQUID-1's own measured degree",
                self.er_giant,
                &i[8..9],
                "mean degree z = 2 * hbonds_per_molecule (the lens counts each bond once; a degree \
                 counts it at both ends), then the fixed point of s = 1 - exp(-z s). This is what a \
                 RANDOM graph of the same density would give, and it is the reference the measured \
                 largest component is read against.",
            ),
        ]
    }
    fn json(&self) -> String {
        format!(
            "{{\"liquid1_alpha\": {}, \"liquid1_t1_fs\": {}, \"liquid1_t2_fs\": {}, \
             \"crossover_fs\": {}, \"dt_reference_au\": {}, \"step_multiple\": {}, \"step_au\": {}, \
             \"step_fs\": {}, \"readout_stride_frames\": {}, \"max_lag_readout_frames\": {}, \
             \"sampled_frames\": {}, \"counted_frames\": {}, \"lowest_fitted_lag_fs\": {}, \
             \"top_fitted_lag_fs\": {}, \"counted_physical_fs\": {}, \
             \"counted_physical_over_4_crossover\": {}, \"wall_cap_bohr2\": {}, \
             \"msd_at_top_lag_at_experiment_d_bohr2\": {}, \"wall_cap_margin\": {}, \
             \"drift_fraction\": {}, \"liquid1_wall_seconds\": {}, \
             \"liquid1_seconds_per_pass\": {}, \"price_ceiling_seconds\": {}, \
             \"liquid1_hbonds_per_molecule\": {}, \"er_degree\": {}, \"er_giant\": {}, \
             \"settling\": {{\"temperature_band_k\": {}, \"band_rule\": \"3 x the scatter of LIQUID-1's own temperature readout over the second half of its counted arm\", \"liquid1_plateau_temperature_k\": {}, \"liquid1_plateau_temperature_sd_k\": {}, \"equipartition_sigma_k\": {}, \"equipartition_dof\": {}, \"liquid1_plateau_hbonds\": {}, \"liquid1_plateau_hbonds_sd\": {}, \"liquid1_reported_hbonds\": {}, \"liquid1_reported_under_its_own_plateau\": {}, \"liquid1_temperature_settled_at_frame\": {}, \"liquid1_hbonds_settled_at_frame\": {}, \"liquid1_hbonds_settled_fs\": {}, \"liquid1_settled_only_frames\": 2000, \"this_criterion_on_liquid1_fires_at_frame\": {}, \"this_criterion_on_liquid1_fires_at_fs\": {}, \"readout_frames\": {}, \"samples_per_block\": {}, \"window_blocks\": {}, \"block_fs\": {}, \"floor_frames\": {}, \"floor_from_physical_time\": {}, \"floor_from_the_thermostats_frame_clock\": {}, \"floor_rule\": \"the larger of the two: the network rearranges in physical time and the thermostat is applied once per step\", \"cap_frames\": {}}}}}",
            num(self.alpha),
            num(self.t1_l1_fs),
            num(self.t2_l1_fs),
            num(self.crossover_fs),
            num(self.dt_tables_au),
            num(STEP_MULT),
            num(self.step_au),
            num(self.step_fs),
            self.stride,
            self.max_lag,
            self.sampled,
            self.count,
            num(self.t1_fs),
            num(self.t2_fs),
            num(self.physical_fs),
            num(self.physical_fs / (CROSSOVER_MULTIPLE * self.crossover_fs)),
            num(self.wall_cap_bohr2),
            num(self.msd_at_top_bohr2),
            num(WALL_CAP_MARGIN),
            num(self.drift_fraction),
            num(self.liquid1_seconds),
            num(self.liquid1_seconds_per_pass),
            num(self.price_ceiling_seconds),
            num(self.liquid1_hbonds),
            num(self.er_degree),
            num(self.er_giant),
            num(self.temp_band_k),
            num(self.l1.t_plateau),
            num(self.l1.t_plateau_sd),
            num(self.equipartition_sigma_k),
            3 * 3 * N_WATERS - 3,
            num(self.l1.bond_plateau),
            num(self.l1.bond_plateau_sd),
            num(self.l1.reported_bond),
            num(self.l1.plateau_shortfall),
            self.l1.t_settled_at,
            self.l1.bond_settled_at,
            num(self.l1.bond_settled_at as f64 * self.dt_tables_au * AU_TIME_FS),
            self.l1.criterion_fires_at,
            num(self.l1.criterion_fires_at as f64 * self.dt_tables_au * AU_TIME_FS),
            SETTLE_READOUT,
            SETTLE_SAMPLES,
            SETTLE_WINDOW,
            num(SETTLE_READOUT as f64 * self.step_fs),
            self.settle_floor_frames,
            (self.l1.bond_settled_at as f64 * self.dt_tables_au * AU_TIME_FS / self.step_fs).ceil() as usize,
            self.l1.bond_settled_at,
            SETTLE_CAP
        )
    }
    fn print(&self) {
        println!("the design, derived before any counted frame");
        println!("  LIQUID-1: MSD exponent {:.4} over [{:.4}, {:.1}] fs", self.alpha, self.t1_l1_fs, self.t2_l1_fs);
        println!("  crossover implied      {:.2} fs", self.crossover_fs);
        println!("  tables' step           {:.6} au = {:.6} fs", self.dt_tables_au, self.dt_tables_au * AU_TIME_FS);
        println!("  this arm's step        {:.6} au = {:.6} fs  ({}x)", self.step_au, self.step_fs, STEP_MULT);
        println!("  readout stride         {} frames -> lowest fitted lag {:.2} fs (>= crossover {:.2})", self.stride, self.t1_fs, self.crossover_fs);
        println!("  max_lag                {} readout frames -> top fitted lag {:.1} fs", self.max_lag, self.t2_fs);
        println!("  wall cap               MSD at the top lag at experiment's D = {:.2} bohr^2 against the lens's cap {:.2} / {:.1}", self.msd_at_top_bohr2, self.wall_cap_bohr2, WALL_CAP_MARGIN);
        println!("  sampled frames         {}", self.sampled);
        println!("  counted frames         {} -> {:.1} fs = {:.2} ps ({:.1}x the 4-crossover floor)", self.count, self.physical_fs, self.physical_fs / 1000.0, self.physical_fs / (CROSSOVER_MULTIPLE * self.crossover_fs));
        println!("  drift bar fraction     {:.6e} of the thermostat's posted work", self.drift_fraction);
        println!("  price ceiling          {:.1} s (10x LIQUID-1's {:.1} s)", self.price_ceiling_seconds, self.liquid1_seconds);
        println!("  ER giant at z = {:.4}  {:.4}", self.er_degree, self.er_giant);
        println!("  settling, from LIQUID-1's own log");
        println!("    plateau (2nd half)   T {:.2} +- {:.2} K, bonds {:.4} +- {:.4}", self.l1.t_plateau, self.l1.t_plateau_sd, self.l1.bond_plateau, self.l1.bond_plateau_sd);
        println!("    band                 +- {:.2} K (3 sd measured); equipartition sigma {:.2} K at {} dof", self.temp_band_k, self.equipartition_sigma_k, 3 * 3 * N_WATERS - 3);
        println!("    LIQUID-1 settled at  T frame {}, bonds frame {} = {:.1} fs; it counted from frame 2000 = {:.1} fs", self.l1.t_settled_at, self.l1.bond_settled_at, self.l1.bond_settled_at as f64 * self.dt_tables_au * AU_TIME_FS, 2000.0 * self.dt_tables_au * AU_TIME_FS);
        println!("    its reported bonds   {:.4} against its own plateau {:.4}: {:.2} % low", self.l1.reported_bond, self.l1.bond_plateau, 100.0 * self.l1.plateau_shortfall);
        println!("    this criterion on LIQUID-1 fires at frame {} = {:.1} fs", self.l1.criterion_fires_at, self.l1.criterion_fires_at as f64 * self.dt_tables_au * AU_TIME_FS);
        println!("    here: block {} frames = {:.2} fs, {} samples, window {} blocks, floor {} frames (physical-time conversion {}, thermostat frame clock {}), cap {}", SETTLE_READOUT, SETTLE_READOUT as f64 * self.step_fs, SETTLE_SAMPLES, SETTLE_WINDOW, self.settle_floor_frames, (self.l1.bond_settled_at as f64 * self.dt_tables_au * AU_TIME_FS / self.step_fs).ceil() as usize, self.l1.bond_settled_at, SETTLE_CAP);
        println!("    blocks logged by LIQUID-1: {}", self.l1.blocks.len());
    }
}

// ------------------------------------------------------------------------ the door

struct Door {
    admitted: bool,
    refusal: Option<String>,
    json: String,
    truncation_per_water: f64,
}

fn door(law: &Law, sim: &mut Sim, pos: &[[f64; 3]], l: f64, tables_reach: f64, staked_units: usize) -> Door {
    let seam_reach_unswitched = law.model.reach_unswitched(SEAM_REACH_BUDGET);
    let seam_reach = law.model.reach(SEAM_REACH_BUDGET);
    let table_reach = law.table.reach(SEAM_REACH_BUDGET);
    let legality = sim.legality_radius();
    let units_reading = sim.units_reading();
    let n_atoms = sim.n;
    let z: Vec<u32> = (0..n_atoms).map(|i| sim.atoms[i].species.z).collect();

    // THE TRUNCATION'S PRICE (Amendment 2 section 3), with CT-3's transfer row taken the way
    // the engine serves it: the pair classes are summed over cross-unit ATOM pairs, and the
    // transfer term is summed ONCE per unordered pair of UNITS at that pair's own contact,
    // because that is the serving rule the table has.
    let r_on = SEAM_CUTOFF_BOHR - 2.0;
    let mut tail = 0.0f64;
    let mut tail_pairs = 0usize;
    let mind = |a: [f64; 3], b: [f64; 3]| -> f64 {
        let mut d2 = 0.0;
        for c in 0..3 {
            let mut d = a[c] - b[c];
            d -= l * (d / l).round();
            d2 += d * d;
        }
        d2.sqrt()
    };
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
            let r = mind(pos[i], pos[j]);
            if r <= r_on {
                continue;
            }
            let (sw, _) = law.model.switch(r);
            let u = match (z[i], z[j]) {
                (8, 8) => law.model.wall(r) + law.model.dispersion(r),
                (1, 1) => law.model.contact_hh(r) + law.model.wall_hh(r),
                _ => law.model.penetration(r) + law.model.wall_oh(r),
            };
            tail += u.abs() * (1.0 - sw);
            tail_pairs += 1;
        }
    }
    // the table's own tail, at the deepest reading it can return at that contact
    let oxy: Vec<usize> = (0..n_atoms).filter(|&i| z[*&i] == 8 && units_reading[*&i] == *&i as u32).collect();
    let mut table_tail = 0.0f64;
    let mut table_tail_pairs = 0usize;
    for (ia, &a) in oxy.iter().enumerate() {
        for &b in oxy.iter().skip(ia + 1) {
            let mut rmin = f64::INFINITY;
            for (dn, ac) in [(a, b), (b, a)] {
                for h in 0..n_atoms {
                    if z[h] == 1 && units_reading[h] == dn as u32 {
                        let d = mind(pos[h], pos[ac]);
                        if d < rmin {
                            rmin = d;
                        }
                    }
                }
            }
            if !(rmin > r_on) || !rmin.is_finite() {
                continue;
            }
            let (sw, _) = law.model.switch(rmin);
            table_tail += law.table.deepest_served(rmin, 5).0.abs() * (1.0 - sw);
            table_tail_pairs += 1;
        }
    }
    let truncation_tail = tail + table_tail;
    let truncation_per_water = truncation_tail / staked_units as f64;
    let truncation_ok = truncation_per_water <= TRUNCATION_STAKE_PER_WATER;

    let units_at_door = units_reading[..n_atoms].iter().enumerate().filter(|(i, &u)| u == *i as u32).count();
    let free_atoms = units_reading[..n_atoms].iter().filter(|&&u| u == holon_render::seam::FREE).count();
    let half_edge = 0.5 * l;
    let boundary_refusal = sim.set_boundary(Boundary::Periodic).err().map(|r| r.to_string());
    let admitted_boundary = boundary_refusal.is_none();

    let mut why: Option<String> = None;
    if let Some(w) = &law.refusal {
        why = Some(w.clone());
    } else if let Some(w) = &boundary_refusal {
        why = Some(format!("the image rule refuses the cell: {w}"));
    } else if !truncation_ok {
        why = Some(format!(
            "the seam switch at r_cut {SEAM_CUTOFF_BOHR:.2} bohr removes {truncation_per_water:.4e} hartree per water on the start box, over the stake {TRUNCATION_STAKE_PER_WATER:e}"
        ));
    } else if units_at_door != staked_units {
        why = Some(format!("{units_at_door} units at the door and the freeze stakes {staked_units} EXACTLY"));
    }

    let json = format!(
        "{{\"phase\": \"door\", \"dry\": false, \"law_source\": {:?}, \"table_source\": {:?}, \"table_knots\": {}, \"table_c0_per_bohr\": {}, \
         \"table_deepest_shape\": {}, \"table_worst_knot_miss\": {}, \"ct_mode\": {:?}, \
         \"served_boundedness_walk\": {}, \"served_grid_points_per_axis\": {}, \
         \"seam_terms_reach_bohr\": {}, \"seam_terms_reach_unswitched_bohr\": {}, \
         \"table_reach_bohr\": {}, \"seam_switch_r_cut_bohr\": {}, \"seam_switch_r_on_bohr\": {}, \
         \"truncation_tail_hartree\": {}, \"truncation_tail_pair_classes\": {}, \
         \"truncation_tail_table_hartree\": {}, \"truncation_tail_table_pairs\": {}, \
         \"truncation_tail_per_water\": {}, \"truncation_stake_per_water\": {}, \"truncation_ok\": {}, \
         \"seam_reach_budget_hartree\": {}, \"intra_unit_reach_bohr\": {}, \"legality_radius_bohr\": {}, \
         \"tables_reach_bohr\": {}, \"half_edge_bohr\": {}, \"cell_edge_bohr\": {}, \"units\": {}, \
         \"staked_units\": {}, \"free_atoms\": {}, \"atoms\": {}, \"boundary_admitted\": {}, \
         \"boundary_refusal\": {}, \"admitted\": {}, \"refusal\": {}, \
         \"legality_is_seam_rule_to_the_bit\": {}, \"q_h\": {}, \"kt\": {}, \"r_min\": [{}, {}, {}]}}",
        law.source,
        law.table_source,
        law.table.knots(),
        num(law.c0),
        num(law.table.deepest_shape),
        num(law.table.worst_knot_miss),
        format!("{:?}", law.model.ct_mode()),
        match &law.refusal {
            Some(w) => format!("{w:?}"),
            None => "null".to_string(),
        },
        SERVED_GRID,
        num(seam_reach),
        num(seam_reach_unswitched),
        num(table_reach),
        num(SEAM_CUTOFF_BOHR),
        num(r_on),
        num(tail),
        tail_pairs,
        num(table_tail),
        table_tail_pairs,
        num(truncation_per_water),
        num(TRUNCATION_STAKE_PER_WATER),
        truncation_ok,
        num(SEAM_REACH_BUDGET),
        num(INTRA_UNIT_REACH),
        num(legality),
        num(tables_reach),
        num(half_edge),
        num(l),
        units_at_door,
        staked_units,
        free_atoms,
        n_atoms,
        admitted_boundary,
        match &boundary_refusal {
            Some(w) => format!("{w:?}"),
            None => "null".to_string(),
        },
        why.is_none(),
        match &why {
            Some(w) => format!("{w:?}"),
            None => "null".to_string(),
        },
        legality.to_bits() == INTRA_UNIT_REACH.max(seam_reach.max(table_reach).min(SEAM_CUTOFF_BOHR)).to_bits(),
        num(law.q_h),
        num(law.kt),
        num(law.r_min[0]),
        num(law.r_min[1]),
        num(law.r_min[2])
    );
    eprintln!(
        "L0 door: unswitched reach {seam_reach_unswitched:.4}, table reach {table_reach:.4}, switched {seam_reach:.4}, legality {legality:.4}, half-edge {half_edge:.4}; truncation {truncation_per_water:.4e} per water against {TRUNCATION_STAKE_PER_WATER:e}; {units_at_door} units, {free_atoms} free -> {}",
        if why.is_none() { "ADMITTED" } else { "REFUSED" }
    );
    Door { admitted: why.is_none(), refusal: why, json, truncation_per_water }
}

// ------------------------------------------------------------------------ the plants

struct Plants {
    i: Plant,
    ii: Plant,
    iii: Plant,
}

/// THE PRE-CHECK AS A GATE. At the gate phase a plant has not been run, and `Plant::gate`
/// correctly calls that VOID; what the gate phase asks is the other question -- whether the
/// plant COULD fire, which is [`Plant::pre_check`]: is the carrier nonzero in the sector the
/// plant acts on, and can the observable move as far as the stake by the freeze's own
/// arithmetic. Both are answered with no measurement at all (FLUID-1's correction).
fn precheck_gate(p: &Plant) -> Gate {
    let g = Gate::new(format!("plant pre-check {}", p.name)).work(1).detail(format!(
        "sector {:?}; carrier {:.6e} (floor {:.6e}); stake {:.6e}; analytic reach {:.6e} - {}",
        p.sector, p.carrier, p.carrier_floor, p.stake.value, p.reach, p.reach_arithmetic
    ));
    match p.pre_check() {
        None => g
            .leg_at("the carrier is nonzero in the sector the plant acts on", true, p.carrier)
            .leg_at("the stake is reachable by the freeze's own arithmetic", true, p.reach),
        Some(v) => g.void(format!("{v:?}")),
    }
}

fn plants(carrier: f64, edge_carrier: f64) -> Plants {
    let stake_move = Stake::derived(
        "plants (i) and (ii): the reading must move by a fifth",
        PLANT_MISS,
        &[],
        "the freeze's own number, section 5, LIQUID-1's unchanged",
    );
    let stake_span = Stake::derived(
        "plant (iii): the spanning fraction must move by half its whole range",
        PLANT_SPAN_MISS,
        &[],
        "a fraction's range is [0, 1] and half of it is 0.5; the freeze's own number",
    );
    let sector = "the first-shell O-O pairs whose minimum-image vector crosses a face";
    let reach_why = "removing the minimum image moves EXACTLY the crossing pairs out of the shell, \
                     so the first peak's fractional change is at most the crossing fraction itself - \
                     the carrier is the reach";
    Plants {
        i: Plant::new(
            "the radial distribution without the minimum image",
            sector,
            carrier,
            CARRIER_FLOOR,
            stake_move.clone(),
            carrier,
            reach_why,
        ),
        ii: Plant::new(
            "the bond count without the minimum image",
            sector,
            carrier,
            CARRIER_FLOOR,
            stake_move,
            carrier,
            reach_why,
        ),
        iii: Plant::new(
            "the spanning cluster with the wrap counts zeroed",
            "the H-bond edges whose wrap count is not zero",
            edge_carrier,
            CARRIER_FLOOR,
            stake_span,
            1.0,
            "a winding is exhibited only by an edge with a nonzero wrap count; zeroing every wrap \
             count makes a winding unconstructible, so the spanning fraction can move by its whole \
             range, 1.0",
        ),
    }
}

// ------------------------------------------------------------------------ the carrier

/// The carrier, on whatever frames the caller has: the fraction of first-shell O-O pairs
/// whose minimum-image vector crosses a face.
struct Carrier {
    pairs: u64,
    crossing: u64,
}

impl Carrier {
    fn new() -> Self {
        Carrier { pairs: 0, crossing: 0 }
    }
    fn push(&mut self, p: &[[f64; 3]], oxy: &[usize], l: f64) {
        let first_shell = first_shell_bohr();
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
                    self.pairs += 1;
                    if crossed {
                        self.crossing += 1;
                    }
                }
            }
        }
    }
    fn fraction(&self) -> f64 {
        if self.pairs == 0 {
            f64::NAN
        } else {
            self.crossing as f64 / self.pairs as f64
        }
    }
}

// ------------------------------------------------------------------------ the phases

fn read_pos(sim: &Sim) -> Vec<[f64; 3]> {
    (0..sim.n).map(|i| [sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z]).collect()
}

fn gate_phase(obs: &Path, out: &Path) {
    let w = RecordWriter::new(out);
    let mut report = Report::new();
    let law = load_law(obs);
    let (mut sim, pos, l, tables_reach, dt_tables) = build(&law, STEP_MULT, SEEDS[0]);
    let d = design(obs, dt_tables, l);
    d.print();

    // ---- L0: the door
    let dr = door(&law, &mut sim, &pos, l, tables_reach, N_WATERS);
    let g_l0 = Gate::new("L0")
        .work(1)
        .detail("the box is legal under the seam-aware image rule with CT-3's table served, and the switch's tail is priced")
        .leg("the served boundedness walk returns None (CT-3 G-B0W)", law.refusal.is_none())
        .leg("the image rule admits the cell", dr.admitted || law.refusal.is_some())
        .leg_at("the truncation tail per water is under its stake", dr.truncation_per_water <= TRUNCATION_STAKE_PER_WATER, dr.truncation_per_water);
    report.gate(g_l0);
    w.write_text("door.json", &format!("{}\n", dr.json)).expect("door.json writes");
    if !dr.admitted {
        w.write_text("gate.void", "{\"void\": true}\n").ok();
        eprintln!("VOID at the door: {}", dr.refusal.unwrap_or_default());
        return;
    }
    sim.rebase();

    // ---- the expectation, before any frame (M-EMPTY-SECTOR)
    sim.compute_forces();
    let e_field = sim.row(Row::Field);
    let e_seam = sim.row(Row::Seam);
    let cross_unit = e_field + e_seam;
    let per_water = cross_unit / N_WATERS as f64;
    let units_start = sim.seam_work.units;
    let kt = law.kt;
    let void_empty = units_start < N_WATERS as u64;
    let reading = if void_empty {
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
            "the box's cross-unit energy is {per_water:.6e} hartree per water against kT = {kt:.6e}; LIQUID-1's three-band rule reads {reading}. THE RULE IS REPORTED, NOT GATED: LIQUID-1's own arm read the same rule above the minus-kT line, wrote break, and then HELD its 128 units for 100,000 frames, so the rule's break branch is known to be wrong for this box and this law family. What still VOIDS is the EMPTY branch alone: fewer than 128 units at the start."
        )
    };
    println!("expectation: {sentence}");
    let exp = Record::new("expectation")
        .flag("written_before_any_frame", true)
        .int("units", units_start as i64)
        .int("staked_units", N_WATERS as i64)
        .flag("void", void_empty)
        .number("cross_unit_energy_hartree", cross_unit)
        .number("field_part_hartree", e_field)
        .number("seam_part_hartree", e_seam)
        .number("per_water_hartree", per_water)
        .number("kT_hartree", kt)
        .number("minus_2kT", -2.0 * kt)
        .number("minus_kT", -kt)
        .text("liquid1_rule_reading", reading)
        .flag("liquid1_rule_is_gated", false)
        .text("sentence", &sentence);
    w.write("expectation.json", &exp).expect("expectation.json writes");
    if void_empty {
        w.write_text("gate.void", "{\"void\": true, \"why\": \"fewer units than staked at the start\"}\n").ok();
        return;
    }

    // ---- the price, on the first frames, before anything is counted
    let t_price = Instant::now();
    for _ in 0..PRICE_FRAMES {
        sim.step_frame(1);
    }
    let price_secs = t_price.elapsed().as_secs_f64();
    let price = Price::measure(
        "the counted arm at 384 atoms with the lattice sum and CT-3's table",
        PRICE_FRAMES as u64,
        price_secs,
        (d.settle_floor_frames + d.count) as u64,
        (0.1, 10.0),
    )
    .expect("a price needs a denominator");
    println!("{}", price.print());
    let per_pass = price.seconds_per_step;
    let table_cost_ratio = per_pass / d.liquid1_seconds_per_pass;
    println!(
        "  the table term's measured cost: {:.6} s per pass against LIQUID-1's {:.6} = {:.4}x",
        per_pass, d.liquid1_seconds_per_pass, table_cost_ratio
    );
    let priced: Priced = price.clone().write(&w, "price.json").expect("the price writes before anything is counted");
    let _ = priced;

    // ---- the plants' carrier, measured on the SETTLED box and before any counted frame.
    //
    // It is measured after the settling and not on the price frames, and that is a finding
    // rather than a convenience: the bcc START puts its nearest oxygens 6.41 bohr apart and
    // the first shell is 6.047, so the sector both plants act on is EXACTLY EMPTY at frame
    // zero and the pre-check reads `CarrierEmpty` on a box that will carry a quarter of its
    // first-shell pairs across a face within the settling. The freeze measures the carrier
    // where the plants act, which is the settled liquid, and the counted arm measures it
    // again over its own readout frames (M-PLANT-SECTOR, M-EMPTY-SECTOR).
    let z: Vec<u32> = (0..sim.n).map(|i| sim.atoms[i].species.z).collect();
    let oxy: Vec<usize> = (0..sim.n).filter(|&i| z[i] == 8).collect();
    let cell = [l, l, l];
    // THE SETTLING, BY THE MEASURED CRITERION and not by a frame count. The gate runs it once
    // so the campaign knows what its settling costs before any seed is released.
    let (settle_used, settler, settle_capped) = settle(&mut sim, &z, cell, &d, PRICE_FRAMES);
    println!(
        "settling: {settle_used} frames = {:.1} fs ({}), {} blocks",
        settle_used as f64 * d.step_fs,
        if settle_capped { "CAPPED - the box did not settle" } else { "settled by the criterion" },
        settler.series.len()
    );
    let mut carrier = Carrier::new();
    let mut ph = PhaseAccum::new();
    for _ in 0..CARRIER_FRAMES {
        sim.step_frame(1);
        let p = read_pos(&sim);
        carrier.push(&p, &oxy, l);
        ph.push(&p, &z, cell);
    }
    let pl = plants(carrier.fraction(), ph.crossing_fraction());
    for p in [&pl.i, &pl.ii, &pl.iii] {
        println!("{}", p.print());
        report.gate(precheck_gate(p));
    }

    // ---- R3: live, or VOID BY PRICE
    let arm_seconds = per_pass * (settle_used + d.count) as f64 * SEEDS.len() as f64;
    let r3_live = arm_seconds <= d.price_ceiling_seconds;
    let g_r3 = Gate::new("R3price")
        .work(1)
        .detail(format!(
            "the counted campaign at the pre-committed step, {} seeds, costs {arm_seconds:.1} s against the ceiling {:.1} s (ten times LIQUID-1's own arm); the window is [{:.2}, {:.1}] fs with the crossover at {:.2} fs",
            SEEDS.len(),
            d.price_ceiling_seconds, d.t1_fs, d.t2_fs, d.crossover_fs
        ))
        .leg_at("the counted campaign, all seeds, is inside the price ceiling", r3_live, arm_seconds)
        .leg_at("the lowest fitted lag is at or above the crossover", d.t1_fs >= d.crossover_fs, d.t1_fs / d.crossover_fs)
        .leg_at("the counted physical time is at least four crossovers", d.physical_fs >= CROSSOVER_MULTIPLE * d.crossover_fs, d.physical_fs / (CROSSOVER_MULTIPLE * d.crossover_fs))
        .leg_at("the top fitted lag's MSD at experiment's D is under the lens's wall cap", d.msd_at_top_bohr2 * WALL_CAP_MARGIN <= d.wall_cap_bohr2, d.msd_at_top_bohr2 / d.wall_cap_bohr2);
    report.gate(g_r3);

    let mut rec = Record::new("gate")
        .raw("design", d.json())
        .raw("gates", report.json())
        .raw("phase_on_the_settled_box_blind", ph.json(true))
        .number("carrier_first_shell_crossing_fraction", carrier.fraction())
        .int("carrier_pairs", carrier.pairs as i64)
        .int("carrier_crossing", carrier.crossing as i64)
        .number("carrier_floor", CARRIER_FLOOR)
        .number("seconds_per_pass", per_pass)
        .number("liquid1_seconds_per_pass", d.liquid1_seconds_per_pass)
        .number("table_term_cost_ratio", table_cost_ratio)
        .number("arm_seconds_projected", arm_seconds)
        .flag("r3_live", r3_live)
        .int("settle_frames_used", settle_used as i64)
        .flag("settling_capped", settle_capped)
        .raw("settling_series", settler.json())
        .int("settle_floor_frames", d.settle_floor_frames as i64)
        .int("settle_cap_frames", SETTLE_CAP as i64)
        .int("seeds", SEEDS.len() as i64)
        .int("counted_frames", d.count as i64)
        .int("readout_stride_frames", d.stride as i64)
        .int("max_lag_readout_frames", d.max_lag as i64)
        .plant(&pl.i)
        .plant(&pl.ii)
        .plant(&pl.iii)
        .flag("admits", report.admits());
    for s in d.stakes() {
        rec = rec.stake(&s);
    }
    for s in kill_stakes() {
        rec = rec.stake(&s);
    }
    w.write("gate.json", &rec).expect("gate.json writes");
    for line in report.refusals() {
        println!("REFUSED {line}");
    }
    if report.admits() {
        w.done("gate.done", &format!("the door admitted; the campaign is priced at {arm_seconds:.1} s over {} seeds x {} counted frames, settling {settle_used} frames", SEEDS.len(), d.count))
            .expect("gate.done writes");
    }
}

/// The kill bands, typed in because they ARE the kills and come from experiment.
fn kill_stakes() -> Vec<Stake> {
    vec![
        Stake::typed_kill_from_experiment(
            "R1 first O-O peak position, bohr, low",
            KILL_R1_POS.0,
            "2.65 Angstrom; Soper 2000, Skinner et al. 2013",
        ),
        Stake::typed_kill_from_experiment(
            "R1 first O-O peak position, bohr, high",
            KILL_R1_POS.1,
            "2.95 Angstrom; Soper 2000, Skinner et al. 2013",
        ),
        Stake::typed_kill_from_experiment(
            "R1 first O-O peak height, low",
            KILL_R1_HEIGHT.0,
            "Soper 2000, Skinner et al. 2013",
        ),
        Stake::typed_kill_from_experiment(
            "R1 first O-O peak height, high",
            KILL_R1_HEIGHT.1,
            "Soper 2000, Skinner et al. 2013",
        ),
        Stake::typed_kill_from_experiment(
            "R2 hydrogen bonds per molecule, experiment's own both-ends count",
            KILL_R2_BOTH_ENDS,
            "3.5 per molecule counting each bond at its donor AND its acceptor; the lens counts each \
             bond ONCE, so the band divides by two: [3.0, 4.0] both-ends becomes [1.5, 2.0] on the lens",
        ),
        Stake::typed_kill_from_experiment(
            "R3 self-diffusion centre, cm^2/s",
            KILL_R3_CENTRE,
            "Krynicki, Green and Sawyer 1978; Mills 1973",
        ),
    ]
}

fn r2_band() -> (f64, f64) {
    (KILL_R2_BOTH_ENDS_BAND.0 / 2.0, KILL_R2_BOTH_ENDS_BAND.1 / 2.0)
}

// ------------------------------------------------------------------------ the screen

fn screen_phase(obs: &Path, out: &Path, step: f64, variant: Variant, label: Option<String>, count_handovers: bool) {
    let law = load_variant(obs, variant);
    let dir = out.join("screen");
    let label = label.unwrap_or_else(|| {
        format!("x{}_{}{}", step as u64, variant.name(), if count_handovers { "_handovers" } else { "" })
    });
    let w = RecordWriter::screen(&dir, &label);
    let (mut sim, pos, l, tables_reach, dt_tables) = build(&law, step, SEEDS[0]);
    let dr = door(&law, &mut sim, &pos, l, tables_reach, N_WATERS);
    if !dr.admitted {
        w.write_text(
            &format!("{label}.json"),
            &format!(
                "{{\n  \"phase\": \"screen\",\n  \"dry\": true,\n  \"screen\": {label:?},\n  \"DIAGNOSTIC\": \"a screen, not a reading\",\n  \"step_multiple_of_the_tables_step\": {},\n  \"law\": {:?},\n  \"door_admitted\": false,\n  \"door_refusal\": {:?}\n}}\n",
                num(step),
                variant.name(),
                dr.refusal.unwrap_or_default()
            ),
        )
        .expect("the screen writes");
        return;
    }
    sim.rebase();
    let t0 = Instant::now();
    let mut void: Option<String> = None;
    for k in 0..SCREEN_SETTLE {
        sim.step_frame(1);
        if void.is_none() && (!sim.pbc_ok() || sim.seam_work.units != N_WATERS as u64) {
            void = Some(format!("settle frame {k}: units {}, pbc_ok {}", sim.seam_work.units, sim.pbc_ok()));
        }
    }
    // the ledger's origin is the settled box, so the drift the bar is read against is the
    // counted part's and not the settling transient's
    sim.rebase();
    // the handover counter reads positions and writes none: the trajectory is identical
    let z: Vec<u32> = (0..sim.n).map(|i| sim.atoms[i].species.z).collect();
    let oxy: Vec<usize> = (0..sim.n).filter(|&i| z[i] == 8).collect();
    let units = sim.units_reading();
    let mut unit_h = vec![[usize::MAX; 2]; sim.n];
    for &o in oxy.iter() {
        let hs: Vec<usize> = (0..sim.n).filter(|&i| z[i] == 1 && units[i] == o as u32).collect();
        if hs.len() == 2 {
            unit_h[o] = [hs[0], hs[1]];
        }
    }
    let mut hand = Handovers::new(oxy.len());
    let served_model = SeamModel { r_cut: SEAM_CUTOFF_BOHR, ..law.model };
    let t_count = Instant::now();
    let mut temp_sum = 0.0;
    for k in 0..SCREEN_COUNT {
        sim.step_frame(1);
        temp_sum += sim.temperature();
        if count_handovers && law.variant == Variant::Ct3Table {
            let p: Vec<[f64; 3]> = (0..sim.n).map(|i| [sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z]).collect();
            hand.push(&p, &oxy, &unit_h, l, &served_model, &law.table);
        }
        if void.is_none() && (!sim.pbc_ok() || sim.seam_work.units != N_WATERS as u64) {
            void = Some(format!("counted frame {k}: units {}, pbc_ok {}", sim.seam_work.units, sim.pbc_ok()));
        }
    }
    let secs = t_count.elapsed().as_secs_f64();
    if count_handovers {
        println!(
            "  handovers: {} events over {} frames = {:.1} per frame; mean |jump| {:.4e} Ha, worst {:.4e}; sum |jump| {:.4e}; signed sum {:.4e}, running peak {:.4e}; random walk {:.4e}; DRIFT PEAK {:.4e}",
            hand.events, hand.frames, hand.per_frame(), hand.mean_jump(), hand.worst,
            hand.abs_sum, hand.signed_sum, hand.signed_peak, hand.random_walk(), sim.drift_peak
        );
    }
    let d = design(obs, dt_tables, l);
    let work_thermostat = sim.work.thermostat;
    let bar = d.drift_bar(work_thermostat);
    let ratio = sim.drift_peak / work_thermostat.abs();
    let step_au = dt_tables * step;
    println!(
        "screen {label}: law {}, dt {:.6} au ({:.6} fs) = {} x the tables step, drift peak {:.6e}, thermostat {:.6e} Ha, ratio {:.6e} against the bar fraction {:.6e} -> {}, {:.4} s/pass, T {:.1} K{}",
        variant.name(),
        step_au,
        step_au * AU_TIME_FS,
        step,
        sim.drift_peak,
        work_thermostat,
        ratio,
        d.drift_fraction,
        if ratio <= d.drift_fraction { "UNDER" } else { "OVER" },
        secs / SCREEN_COUNT as f64,
        temp_sum / SCREEN_COUNT as f64,
        match &void {
            Some(v) => format!(", VOID: {v}"),
            None => String::new(),
        }
    );
    let rec = Record::new("screen")
        .text("knob", "the integration step in multiples of the TABLES step, and the law")
        .text("law", variant.name())
        .text("law_source", &law.source)
        .number("step_multiple_of_the_tables_step", step)
        .number("dt_reference_au", sim.timescale.dt_reference)
        .number("dt_tables_au", dt_tables)
        .number("step_au", step_au)
        .number("step_fs", step_au * AU_TIME_FS)
        .number("dt_in_force_at_the_end_au", sim.dt())
        .flag("the_step_held_to_the_bit", (sim.dt() - step_au).abs() <= 1e-12 * step_au.abs())
        .flag("rung_two_dt_growth", step != 1.0)
        .int("settle_frames", SCREEN_SETTLE as i64)
        .int("counted_frames", SCREEN_COUNT as i64)
        .number("counted_physical_fs", SCREEN_COUNT as f64 * step_au * AU_TIME_FS)
        .number("drift_peak", sim.drift_peak)
        .number("work_thermostat_hartree", work_thermostat)
        .number("drift_over_thermostat_work", ratio)
        .number("drift_bar_fraction_from_liquid1", d.drift_fraction)
        .number("drift_bar_hartree", bar)
        .flag("under_the_bar", ratio <= d.drift_fraction)
        .flag("columns_ok", sim.work_columns_ok())
        .number("momentum_residual", sim.momentum_residual())
        .number("momentum_bound", sim.momentum_bound())
        .number("seconds_per_pass", secs / SCREEN_COUNT as f64)
        .number("counted_seconds", secs)
        .number("total_seconds", t0.elapsed().as_secs_f64())
        .number("mean_temperature_k", temp_sum / SCREEN_COUNT as f64)
        .int("units_at_end", sim.seam_work.units as i64)
        .number("seam_ho_pairs", sim.seam_work.ho_pairs as f64)
        .flag("handovers_counted", count_handovers)
        .raw("argmin_handovers", if count_handovers { hand.json() } else { "null".to_string() })
        .number("drift_over_signed_handover_peak", if count_handovers && hand.signed_peak > 0.0 { sim.drift_peak / hand.signed_peak } else { f64::NAN })
        .number("drift_over_summed_absolute_jumps", if count_handovers && hand.abs_sum > 0.0 { sim.drift_peak / hand.abs_sum } else { f64::NAN })
        .text("void", &void.clone().unwrap_or_else(|| "none".to_string()));
    w.write(&format!("{label}.json"), &rec).expect("the screen writes");
    w.done(&format!("{label}.done"), "a screen, not a reading").expect("the marker writes");
}

fn size_phase(obs: &Path, out: &Path, cells: usize) {
    // ONE BOX PER INVOCATION, on purpose. The largest box in the set allocates the periodic
    // three-body buffer at 1,296 atoms and a host that cannot hold it kills the process; run
    // as one process per size, that is a MEASURED fence on the largest box and the smaller
    // ones' records survive it. Run in one process, it would lose all four.
    let w = RecordWriter::new(out.join("size"));
    let law = load_law(obs);
    let waters = 2 * cells * cells * cells;
    let (mut sim, pos, l, tables_reach, _dt) = build_sized(&law, STEP_MULT, SEEDS[0], cells);
    let dr = door(&law, &mut sim, &pos, l, tables_reach, waters);
    let half = 0.5 * l;
    let legality = sim.legality_radius();
    let (secs_per_pass, priced_on) = if dr.admitted {
        sim.rebase();
        let t0 = Instant::now();
        for _ in 0..SIZE_PRICE_FRAMES {
            sim.step_frame(1);
        }
        (t0.elapsed().as_secs_f64() / SIZE_PRICE_FRAMES as f64, SIZE_PRICE_FRAMES)
    } else {
        (f64::NAN, 0)
    };
    println!(
        "size {waters} waters ({cells}^3 bcc): edge {l:.4} bohr, half-edge {half:.4} against legality {legality:.4} -> {}{}",
        if dr.admitted { "ADMITTED" } else { "REFUSED" },
        if priced_on > 0 { format!(", {secs_per_pass:.4} s per pass") } else { String::new() }
    );
    let rec = Record::new("size")
        .text(
            "rule",
            "the door and the force pass at 2 n^3 waters on an n^3 body-centred lattice at the \
             state point's density. A box is available to this campaign only if its door admits \
             it AND a counted arm at it fits the price ceiling; both halves are measured here.",
        )
        .int("cells", cells as i64)
        .int("waters", waters as i64)
        .int("atoms", 3 * waters as i64)
        .number("cell_edge_bohr", l)
        .number("half_edge_bohr", half)
        .number("legality_radius_bohr", legality)
        .flag("door_admitted", dr.admitted)
        .text("door_refusal", dr.refusal.as_deref().unwrap_or("none"))
        .number("seconds_per_pass", secs_per_pass)
        .int("priced_on_frames", priced_on as i64)
        .raw("door", dr.json.clone());
    w.write(&format!("cells{cells}.json"), &rec).expect("the size record writes");
    w.done(&format!("cells{cells}.done"), "one box: its door and its force pass, no arm").expect("the marker writes");
}

// ------------------------------------------------------- the cost probe (GANTT2 step 2)

/// A field of `/proc/self/status`, in kibibytes. `VmHWM` is the kernel's own peak resident
/// set for this process and is MONOTONE for the process's life, so a stage's reading is the
/// peak the process had reached BY the end of that stage and the DIFFERENCE between two
/// consecutive readings is the peak that stage created. That monotonicity is exactly why the
/// probe runs ONE BOX AND ONE ARM PER INVOCATION: two arms in one process would hand the
/// second one the first one's high-water mark and read it as its own.
fn proc_status_kib(key: &str) -> f64 {
    let t = match std::fs::read_to_string("/proc/self/status") {
        Ok(t) => t,
        Err(_) => return f64::NAN,
    };
    for line in t.lines() {
        if let Some(rest) = line.strip_prefix(key) {
            let digits: String = rest.chars().filter(|c| c.is_ascii_digit()).collect();
            return digits.parse::<f64>().unwrap_or(f64::NAN);
        }
    }
    f64::NAN
}

fn hwm_gib() -> f64 {
    proc_status_kib("VmHWM:") / (1024.0 * 1024.0)
}

fn rss_gib() -> f64 {
    proc_status_kib("VmRSS:") / (1024.0 * 1024.0)
}

/// The stage table: what each step of the construction cost in seconds, and where the
/// process's peak resident set stood when it finished.
struct Stages {
    rows: Vec<(String, f64, f64, f64)>,
}

impl Stages {
    fn new() -> Stages {
        let mut s = Stages { rows: Vec::new() };
        s.rows.push(("start".to_string(), 0.0, hwm_gib(), rss_gib()));
        s
    }
    fn mark<T>(&mut self, name: &str, f: impl FnOnce() -> T) -> T {
        let t0 = Instant::now();
        let v = f();
        let secs = t0.elapsed().as_secs_f64();
        self.rows.push((name.to_string(), secs, hwm_gib(), rss_gib()));
        v
    }
    fn peak(&self) -> f64 {
        self.rows.last().map(|r| r.2).unwrap_or(f64::NAN)
    }
    fn json(&self) -> String {
        let mut out = String::from("[");
        for (i, (name, secs, hwm, rss)) in self.rows.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            let prev = if i == 0 { *hwm } else { self.rows[i - 1].2 };
            out.push_str(&format!(
                "{{\"stage\": \"{name}\", \"seconds\": {}, \"peak_rss_gib\": {}, \"peak_rss_created_gib\": {}, \"rss_gib\": {}}}",
                num(*secs),
                num(*hwm),
                num(*hwm - prev),
                num(*rss)
            ));
        }
        out.push(']');
        out
    }
}

/// The two ways a scene of KNOWN coordinates can be built.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Arm {
    /// `quartet::scene`'s path as the second review found it: `reset(n)` first, which lays a
    /// PLACEHOLDER configuration (a ring or a Fibonacci shell of radius 6 bohr) and evaluates
    /// the forces on it, and only then installs the real species and coordinates.
    Placeholder,
    /// The geometry first: storage, species, coordinates, and then ONE force pass, on the
    /// configuration the caller actually asked for. Expressed here in the engine's existing
    /// public API so that the arm can be measured BEFORE it is packaged as `Sim::reset_with`.
    GeometryFirst,
}

/// `quartet::scene` and `field2_scenes::scene` unrolled, with a timer and a `/proc` reading
/// between the steps, and with the opening move switchable between the two arms.
///
/// The duplication is what makes this an instrument: the production constructor is one call,
/// and a probe that could only call it could not say which of its steps carries the cost, nor
/// price the alternative on the same binary. `cost_phase` therefore holds this against the
/// production `scene()` on a small box first (`instrument_residual`) and refuses to report if
/// they disagree on a single bit — a stage table taken on a scene that is not the campaign's
/// scene would be a measurement of this function.
fn cost_scene(
    st: &mut Stages,
    arm: Arm,
    species: &[holon_chem::elements::Species],
    pos: &[[f64; 3]],
    box_edge: f64,
    temp: f64,
) -> Box<Sim> {
    use holon_render::bank::Host;
    use holon_render::sim::Dims;
    use holon_render::{load_pair_table, TABLE_OK};
    let n = species.len();
    let b = st.mark("tables", field2_scenes::quartet::banked);
    let mut s = st.mark("bank_load", || {
        let mut s = Box::new(Sim::empty());
        assert_eq!(load_pair_table(&mut s, &b.hh, Host::Native), TABLE_OK);
        assert_eq!(load_pair_table(&mut s, &b.oh, Host::Native), TABLE_OK);
        assert_eq!(load_pair_table(&mut s, &b.oo, Host::Native), TABLE_OK);
        s.trimer = (*b.trimer).clone();
        s.water = (*b.water).clone();
        s
    });
    match arm {
        Arm::Placeholder => {
            // `reset` lays the placeholder AND evaluates the forces on it (`zero_ledger`).
            st.mark("reset_placeholder", || s.reset(n));
            // The force half of that call, on the same configuration, isolated: `rebase` IS
            // `zero_ledger`, so this is the second half of `reset` run again by itself.
            st.mark("placeholder_force", || s.rebase());
            st.mark("install_geometry", || {
                for (i, sp) in species.iter().enumerate() {
                    assert!(s.set_species(i, *sp));
                }
                for (i, c) in pos.iter().enumerate() {
                    s.atoms[i].x = c[0];
                    s.atoms[i].y = c[1];
                    s.atoms[i].z = c[2];
                    s.atoms[i].vx = 0.0;
                    s.atoms[i].vy = 0.0;
                    s.atoms[i].vz = 0.0;
                }
            });
        }
        Arm::GeometryFirst => {
            st.mark("reset_with", || assert!(s.reset_with(species, pos), "the bank refused a species"));
        }
    }
    s.many_body_order = 0;
    // ---- `field2_scenes::scene`'s tail, verbatim
    st.mark("scene_tail", || {
        s.dims = Dims::Three;
        s.boundary = Boundary::Open;
        s.width = box_edge;
        s.height = box_edge;
        s.depth = box_edge;
        let mut state: u64 = 0x4649_454c_3200;
        let mut lcg = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((state >> 11) as f64) / ((1u64 << 53) as f64)
        };
        let (mut px, mut py, mut pz) = (0.0, 0.0, 0.0);
        for i in 0..n {
            let m = s.atoms[i].mass();
            let scale = (K_B * temp / m).sqrt();
            s.atoms[i].vx = scale * (2.0 * lcg() - 1.0) * 1.7;
            s.atoms[i].vy = scale * (2.0 * lcg() - 1.0) * 1.7;
            s.atoms[i].vz = scale * (2.0 * lcg() - 1.0) * 1.7;
            px += m * s.atoms[i].vx;
            py += m * s.atoms[i].vy;
            pz += m * s.atoms[i].vz;
        }
        let mtot: f64 = (0..n).map(|i| s.atoms[i].mass()).sum();
        for i in 0..n {
            s.atoms[i].vx -= px / mtot;
            s.atoms[i].vy -= py / mtot;
            s.atoms[i].vz -= pz / mtot;
        }
        s.sync_species();
        s.adopt_table_timescale();
        s.thermostat_on = true;
        s.target_temperature = temp;
        s.rebase();
    });
    s
}

/// Every float two scenes have to agree on for the arms to be interchangeable: the ledger's
/// own baselines, the step, the energy, and each atom's coordinates, velocity and force.
fn scene_bits(s: &Sim) -> Vec<u64> {
    let mut v = vec![
        s.n as u64,
        s.energy().to_bits(),
        s.ledger().to_bits(),
        s.dt().to_bits(),
        s.l0.to_bits(),
        s.e_ref.to_bits(),
        s.p0.0.to_bits(),
        s.p0.1.to_bits(),
        s.p0.2.to_bits(),
        s.l0_ang.0.to_bits(),
        s.l0_ang.1.to_bits(),
        s.l0_ang.2.to_bits(),
    ];
    for i in 0..s.n {
        let a = &s.atoms[i];
        let (fx, fy, fz) = s.internal_force(i);
        for x in [a.x, a.y, a.z, a.vx, a.vy, a.vz, fx, fy, fz] {
            v.push(x.to_bits());
        }
    }
    v
}

/// THE INSTRUMENT'S OWN GATE, and the acceptance evidence in one reading.
///
/// Three scenes on the smallest box in the size set, compared BIT FOR BIT: the placeholder
/// arm (the constructor as the second review found it — `reset(n)` first, the coordinates
/// after), the geometry-first arm (`Sim::reset_with`), and the production `scene()`. The
/// first pair is the change under test; the second pair is the claim that the probe measures
/// the campaign's own constructor and not a copy of it that has drifted. Returns the number
/// of disagreeing floats in each pair.
fn instrument_residual(cells: usize) -> (usize, usize) {
    let (species, pos, l) = liquid_box(cells, DENSITY_G_CM3, SEEDS[0]);
    let mut st = Stages::new();
    let old = scene_bits(&cost_scene(&mut st, Arm::Placeholder, &species, &pos, l, TEMPERATURE_K));
    let new = scene_bits(&cost_scene(&mut st, Arm::GeometryFirst, &species, &pos, l, TEMPERATURE_K));
    let prod = scene_bits(&scene(&species, &pos, l, TEMPERATURE_K));
    let count = |a: &[u64], b: &[u64]| -> usize {
        if a.len() != b.len() {
            return a.len().max(b.len());
        }
        a.iter().zip(b.iter()).filter(|(x, y)| x != y).count()
    };
    (count(&old, &new), count(&new, &prod))
}

/// THE COST PROBE. One box, one arm, one process (see `proc_status_kib`).
///
/// It prices the three things the second review named: the `reset` call that lays a
/// placeholder configuration before the real coordinates are known, the force evaluation
/// inside it, and `fenced_triples`. Everything it reports is a measurement; nothing here
/// enters a gate or a claim.
fn cost_phase(obs: &Path, out: &Path, cells: usize, arm: Arm, label: &str) {
    let w = RecordWriter::new(out.join("size"));
    let law = load_law(obs);
    let waters = 2 * cells * cells * cells;
    // The instrument's own gate first, on the smallest box, so its cost is charged to the
    // process's baseline rather than to a stage.
    let (arm_residual, prod_residual) = instrument_residual(3);
    assert_eq!(prod_residual, 0, "the cost probe's unrolled constructor is not `scene()` bit for bit");
    assert_eq!(arm_residual, 0, "the two arms do not build the same scene bit for bit");

    let (species, pos, l) = liquid_box(cells, DENSITY_G_CM3, SEEDS[0]);
    let baseline = hwm_gib();
    let mut st = Stages::new();
    let mut sim = cost_scene(&mut st, arm, &species, &pos, l, TEMPERATURE_K);
    let tables_reach = sim.legality_radius();
    st.mark("field_and_seam", || {
        sim.set_field(true, None).expect("the open box admits the field");
        if law.variant == Variant::Ct3Table {
            sim.ct_table = law.table.clone();
        }
        sim.set_seam(Some(law.model)).expect("no acuity frame is installed");
    });
    let dt_ref = sim.timescale.dt_reference;
    let dt_tables = sim.dt();
    if STEP_MULT != 1.0 {
        sim.timescale.allow_dt_growth = true;
        sim.timescale.set_dt_multiplier(STEP_MULT * dt_tables / dt_ref);
    }
    let dt_au = sim.dt();
    // The door's own move, and the reason it is here: `size_phase` prices its force pass
    // AFTER `door` has switched the box to `Boundary::Periodic`, so a pass priced under the
    // open boundary the constructor leaves behind would not be the campaign's pass.
    let periodic = st.mark("boundary_periodic", || sim.set_boundary(Boundary::Periodic).is_ok());
    assert!(periodic, "the image rule refused this box; price it at a size the door admits");

    // The fence, both ways, on the same scene: the census that is now on the force path and
    // the enumeration it replaced, kept as `fenced_triples_enumerated`. One call is the whole
    // count, so each is repeated and the per-call time reported.
    const FENCE_CALLS: usize = 5;
    let mut fenced = 0u64;
    let fence_secs = {
        let t0 = Instant::now();
        for _ in 0..FENCE_CALLS {
            fenced = sim.fenced_triples();
        }
        t0.elapsed().as_secs_f64() / FENCE_CALLS as f64
    };
    st.rows.push(("fenced_triples_census".to_string(), fence_secs, hwm_gib(), rss_gib()));
    let mut fenced_ref = 0u64;
    let fence_ref_secs = {
        let t0 = Instant::now();
        for _ in 0..FENCE_CALLS {
            fenced_ref = sim.fenced_triples_enumerated();
        }
        t0.elapsed().as_secs_f64() / FENCE_CALLS as f64
    };
    st.rows.push(("fenced_triples_enumerated".to_string(), fence_ref_secs, hwm_gib(), rss_gib()));
    assert_eq!(fenced, fenced_ref, "the census and the enumeration disagree on this box");

    sim.rebase();
    // THE L0 SCENE'S OWN NUMBERS, at the point `gate_phase` reads them (`rebase`, then one
    // force pass, before any frame): the cross-unit rows this campaign's expectation record
    // is written from. Reported so the construction change can be held against the banked
    // `gate_provisional_8x/expectation.json` at the bit, not only against a test's tolerance.
    sim.compute_forces();
    let e_field = sim.row(Row::Field);
    let e_seam = sim.row(Row::Seam);
    let units_start = sim.seam_work.units;
    let cross_unit = e_field + e_seam;

    let t0 = Instant::now();
    for _ in 0..SIZE_PRICE_FRAMES {
        sim.step_frame(1);
    }
    let secs_per_pass = t0.elapsed().as_secs_f64() / SIZE_PRICE_FRAMES as f64;
    st.rows.push(("force_pass".to_string(), secs_per_pass, hwm_gib(), rss_gib()));
    let ps_per_pass = dt_au * AU_TIME_FS / 1000.0;
    let core_seconds_per_ps = secs_per_pass / ps_per_pass;
    // ARITHMETIC, NOT A MEASUREMENT, and labelled as such in the record: the fence is
    // evaluated once per `accumulate_three_body` and `accumulate_three_body` once per force
    // pass, so a pass carrying the enumeration instead of the census costs the pass measured
    // here plus the difference between the two fence timings, all three of them measured.
    let secs_per_pass_enumerated = secs_per_pass + (fence_ref_secs - fence_secs);
    let core_seconds_per_ps_enumerated = secs_per_pass_enumerated / ps_per_pass;

    println!(
        "cost {waters} waters, arm {label}: peak {:.4} GiB, fence {fenced} in {fence_secs:.6} s \
         (enumerated {fence_ref_secs:.6} s), force pass {secs_per_pass:.4} s, \
         {core_seconds_per_ps:.1} core-s per ps",
        st.peak()
    );
    for (name, secs, hwm, rss) in st.rows.iter() {
        println!("  {name:<20} {secs:>10.4} s   peak {hwm:>8.4} GiB   rss {rss:>8.4} GiB");
    }

    let rec = Record::new("cost")
        .text(
            "rule",
            "peak resident set (/proc/self/status VmHWM, monotone) and seconds around every step \
             of the scene construction, around the placeholder force evaluation inside `reset`, \
             and around `fenced_triples`, for one box and one arm per process. A measurement \
             only: nothing here enters a gate or a claim.",
        )
        .text("arm", label)
        .int("cells", cells as i64)
        .int("waters", waters as i64)
        .int("atoms", 3 * waters as i64)
        .number("cell_edge_bohr", l)
        .number("legality_radius_bohr", tables_reach)
        .int("arm_disagreements", arm_residual as i64)
        .int("production_disagreements", prod_residual as i64)
        .number("baseline_peak_rss_gib", baseline)
        .number("peak_rss_gib", st.peak())
        .number("fenced_triples_seconds", fence_secs)
        .number("fenced_triples_enumerated_seconds", fence_ref_secs)
        .int("fenced_triples", fenced as i64)
        .int("fenced_triples_enumerated", fenced_ref as i64)
        .int("fenced_triples_calls", FENCE_CALLS as i64)
        .number("seconds_per_pass", secs_per_pass)
        .number("seconds_per_pass_with_enumerated_fence", secs_per_pass_enumerated)
        .text(
            "seconds_per_pass_with_enumerated_fence_is",
            "arithmetic on three measured numbers, not a fourth measurement: the fence is \
             evaluated once per force pass, so the pass as it stood before the census costs \
             `seconds_per_pass + (fenced_triples_enumerated_seconds - fenced_triples_seconds)`.",
        )
        .int("priced_on_frames", SIZE_PRICE_FRAMES as i64)
        .int("units_at_start", units_start as i64)
        .number("field_part_hartree", e_field)
        .number("seam_part_hartree", e_seam)
        .number("cross_unit_energy_hartree", cross_unit)
        .number("per_water_hartree", cross_unit / waters as f64)
        .int("force_workers", sim.workers() as i64)
        .number("dt_au", dt_au)
        .number("picoseconds_per_pass", ps_per_pass)
        .number("core_seconds_per_picosecond", core_seconds_per_ps)
        .number("core_seconds_per_picosecond_with_enumerated_fence", core_seconds_per_ps_enumerated)
        .raw("stages", st.json());
    w.write(&format!("cost_{label}_cells{cells}.json"), &rec).expect("the cost record writes");
}

/// Gather the per-box cost records of one arm into the reading the build order asks for.
/// A gather, not a measurement: every number in it was taken by a `cost` process of its own,
/// and this only puts the sizes side by side.
fn cost_merge(out: &Path, arm_label: &str, name: &str) {
    let w = RecordWriter::new(out.join("size"));
    let dir = out.join("size");
    let mut boxes = String::from("[");
    let mut found = 0usize;
    for cells in [4usize, 5, 6] {
        let p = dir.join(format!("cost_{arm_label}_cells{cells}.json"));
        let Ok(t) = std::fs::read_to_string(&p) else { continue };
        if found > 0 {
            boxes.push_str(", ");
        }
        boxes.push_str(t.trim());
        found += 1;
    }
    boxes.push(']');
    assert!(found > 0, "no cost_{arm_label}_cells*.json under {}", dir.display());
    let rec = Record::new("cost")
        .text(
            "rule",
            "the per-box cost records of one arm, gathered. Every number was taken by its own \
             process (VmHWM is monotone, so two boxes in one process would read the first \
             one's peak as the second one's); this record only puts them side by side.",
        )
        .text("arm", arm_label)
        .int("boxes", found as i64)
        .raw("sizes", boxes);
    w.write(name, &rec).expect("the merged cost record writes");
    println!("{name}: {found} boxes of arm {arm_label}");
}

// ------------------------------------------------------------------------ the counted arm

fn run_phase(obs: &Path, out: &Path, seed_index: usize) {
    if !is_done(out, "gate.done") {
        eprintln!("REFUSED: gate.done is absent. The counted arm runs only behind the gate phase.");
        std::process::exit(2);
    }
    let seed = *SEEDS.get(seed_index).unwrap_or_else(|| panic!("seed index {seed_index} against {} declared seeds", SEEDS.len()));
    let out = &out.join(format!("seed{seed_index}"));
    let w = RecordWriter::new(out);
    let law = load_law(obs);
    if let Some(why) = &law.refusal {
        eprintln!("REFUSED: {why}");
        std::process::exit(2);
    }
    eprintln!("seed {seed_index} of {}: {seed:#x}", SEEDS.len());
    let (mut sim, pos, l, tables_reach, dt_tables) = build(&law, STEP_MULT, seed);
    let d = design(obs, dt_tables, l);
    d.print();
    let dr = door(&law, &mut sim, &pos, l, tables_reach, N_WATERS);
    if !dr.admitted {
        w.write_text("arm.void", &format!("{{\"void\": true, \"why\": {:?}}}\n", dr.refusal.unwrap_or_default())).ok();
        return;
    }
    sim.rebase();
    let t_start = Instant::now();

    let n_atoms = sim.n;
    let z: Vec<u32> = (0..n_atoms).map(|i| sim.atoms[i].species.z).collect();
    let oxy: Vec<usize> = (0..n_atoms).filter(|&i| z[i] == 8).collect();
    let cell = [l, l, l];
    let dt_at_placement = sim.dt();

    let mut prev: Vec<[f64; 3]> = oxy.iter().map(|&i| [sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z]).collect();
    let mut unwrapped: Vec<[f64; 3]> = prev.clone();
    let mut void: Option<(usize, &'static str, String)> = None;
    let mut frames_run = 0usize;

    macro_rules! one_frame {
        ($phase:expr, $k:expr) => {{
            sim.step_frame(1);
            frames_run += 1;
            let now: Vec<[f64; 3]> = oxy.iter().map(|&i| [sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z]).collect();
            for a in 0..oxy.len() {
                for c in 0..3 {
                    let mut dd = now[a][c] - prev[a][c];
                    dd -= l * (dd / l).round();
                    unwrapped[a][c] += dd;
                }
            }
            prev = now;
            if void.is_none() {
                if !sim.pbc_ok() {
                    let (reach, half) = sim.pbc_margin();
                    void = Some(($k, $phase, format!("pbc_ok is false: the force law's reach is {reach:.4} bohr against a half-edge of {half:.4}")));
                } else if sim.seam_work.units != N_WATERS as u64 {
                    void = Some(($k, $phase, format!("the unit count is {} and the freeze stakes {N_WATERS} EXACTLY", sim.seam_work.units)));
                }
            }
        }};
    }

    // the price, measured again on this phase's own first frames and written before the
    // counted ones
    let t_price = Instant::now();
    for k in 0..PRICE_FRAMES {
        one_frame!("settle", k);
        if void.is_some() {
            break;
        }
    }
    let price = Price::measure(
        "the counted arm at 384 atoms with the lattice sum and CT-3's table",
        PRICE_FRAMES as u64,
        t_price.elapsed().as_secs_f64(),
        (d.settle_floor_frames + d.count) as u64,
        (0.1, 10.0),
    )
    .expect("a price needs a denominator");
    println!("{}", price.print());
    let priced = price.write(&w, "price.json").expect("the price writes");

    // THE SETTLING, by the measured criterion. `one_frame!`'s per-pass L0 check does not run
    // inside it, so the unit count and `pbc_ok` are checked once when it returns; a box that
    // dissolves while settling is caught there and the arm is VOID before a frame is counted.
    let (settle_used, settler, settle_capped) = settle(&mut sim, &z, cell, &d, PRICE_FRAMES);
    frames_run += settle_used - PRICE_FRAMES;
    if void.is_none() && (!sim.pbc_ok() || sim.seam_work.units != N_WATERS as u64) {
        void = Some((settle_used, "settle", format!("after settling: units {}, pbc_ok {}", sim.seam_work.units, sim.pbc_ok())));
    }
    if settle_capped && void.is_none() {
        void = Some((settle_used, "settle", format!("the settling criterion did not fire inside its cap of {SETTLE_CAP} frames; the box did not settle and nothing here is counted")));
    }
    println!(
        "settling: {settle_used} frames = {:.1} fs ({}); {} blocks",
        settle_used as f64 * d.step_fs,
        if settle_capped { "CAPPED - VOID" } else { "settled by the criterion" },
        settler.series.len()
    );
    // the unwrap accumulator restarts from the settled box: the counted trajectory is the
    // counted trajectory
    prev = oxy.iter().map(|&i| [sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z]).collect();
    unwrapped = prev.clone();
    // the ledger's origin is the settled box: the drift the bar is read against is the
    // COUNTED arm's, not the settling transient's
    sim.rebase();

    let mut rdf = RdfAccum::new();
    let mut ph = PhaseAccum::new();
    let mut carrier = Carrier::new();
    let mut hb_sum = 0.0f64;
    let mut hb_open_sum = 0.0f64;
    let mut temp_sum = 0.0f64;
    let mut readouts = 0usize;
    let mut traj_frames: Vec<Frame> = Vec::new();
    let mut rdf_refusal: Option<String> = None;
    let mut hb_refusal: Option<String> = None;

    if void.is_none() {
        for k in 0..d.count {
            one_frame!("counted", k);
            temp_sum += sim.temperature();
            if void.is_some() {
                break;
            }
            if (k + 1) % d.stride != 0 {
                continue;
            }
            let p = read_pos(&sim);
            match hbonds_periodic(&p, &z, cell) {
                Ok(v) => hb_sum += v.len() as f64,
                Err(e) => hb_refusal = Some(format!("{}: {}", e.lens, e.reason)),
            }
            match hbonds(&p, &z) {
                Ok(v) => hb_open_sum += v.len() as f64,
                Err(e) => hb_refusal = Some(format!("{}: {}", e.lens, e.reason)),
            }
            if let Err(e) = rdf.push(&p, &z, l) {
                rdf_refusal = Some(format!("{}: {}", e.lens, e.reason));
            }
            ph.push(&p, &z, cell);
            carrier.push(&p, &oxy, l);
            traj_frames.push(Frame {
                index: k as u64,
                time: sim.time,
                temperature: sim.temperature(),
                bonds: BondSet::empty(),
                pos: unwrapped.clone(),
                vel: vec![[0.0; 3]; oxy.len()],
            });
            readouts += 1;
            if readouts % 20 == 0 || k + 1 == d.count {
                eprintln!(
                    "  frame {:>8}: T {:6.1} K, units {}, bonds/molecule {:.3}, spanning {:.3}, largest {:.3}, drift {:.2e}, {readouts} readouts",
                    k + 1,
                    sim.temperature(),
                    sim.seam_work.units,
                    hb_sum / readouts as f64 / N_WATERS as f64,
                    ph.spanning_fraction(),
                    ph.largest_fraction(),
                    sim.drift_peak
                );
            }
        }
    }
    let counted = if let Some((k, p, _)) = &void {
        if *p == "counted" {
            *k
        } else {
            0
        }
    } else {
        d.count
    };
    let wall = t_start.elapsed().as_secs_f64();
    let (g_mean, g_raw) = rdf.mean();

    w.write_text(
        "rdf.json",
        &format!(
            "{{\n  \"phase\": \"run\", \"dry\": false,\n  \"dr_bohr\": {}, \"r_max_bohr\": {}, \"frames\": {}, \"n_o\": {}, \"rho_o_bohr3\": {},\n  \"r\": [{}],\n  \"g\": [{}],\n  \"g_plant_raw_differences\": [{}],\n  \"plant_cell_bohr\": {}\n}}\n",
            num(RDF_DR),
            num(0.5 * l),
            rdf.frames,
            rdf.n_o,
            num(rdf.rho_o),
            rdf.r.iter().map(|&x| num(x)).collect::<Vec<_>>().join(", "),
            g_mean.iter().map(|&x| num(x)).collect::<Vec<_>>().join(", "),
            g_raw.iter().map(|&x| num(x)).collect::<Vec<_>>().join(", "),
            num(PLANT_CELL)
        ),
    )
    .expect("rdf.json writes");

    // the readings
    let peak = if readouts > 0 { first_peak(&rdf.r, &g_mean) } else { None };
    let peak_raw = if readouts > 0 { first_peak(&rdf.r, &g_raw) } else { None };
    let r2 = if readouts > 0 { hb_sum / readouts as f64 / N_WATERS as f64 } else { f64::NAN };
    let r2_open = if readouts > 0 { hb_open_sum / readouts as f64 / N_WATERS as f64 } else { f64::NAN };

    let conv = bohr2_per_fs_to_cm2_per_s();
    let traj = Trajectory {
        header: Header {
            seed,
            n_atoms: oxy.len(),
            dims: 3,
            substeps: 1,
            n_frames: traj_frames.len(),
            dt: dt_at_placement,
            box_w: l,
            box_h: l,
            box_d: l,
            z: vec![8; oxy.len()],
        },
        frames: traj_frames,
    };
    let max_lag = (traj.frames.len() / 4).max(2);
    let (r3, r3_refusal) = match diffusion(&traj, max_lag) {
        Ok(dd) => (Some(dd * conv), None),
        Err(e) => (None, Some(format!("{} refuses (gate: {}): {}", e.lens, e.gate, e.reason))),
    };

    let drift_bar = d.drift_bar(sim.work.thermostat);
    let arm = Record::new("run")
        .flag("is_a_reading", void.is_none())
        .text("law_source", &law.source)
        .text("table_source", &law.table_source)
        .raw("design", d.json())
        .int("seed_index", seed_index as i64)
        .text("seed", &format!("{seed:#x}"))
        .int("settle_frames_used", settle_used as i64)
        .flag("settling_capped", settle_capped)
        .int("settle_floor_frames", d.settle_floor_frames as i64)
        .int("settle_cap_frames", SETTLE_CAP as i64)
        .number("settle_fs", settle_used as f64 * d.step_fs)
        .raw("settling_series", settler.json())
        .int("counted_frames_staked", d.count as i64)
        .int("counted_frames_run", counted as i64)
        .int("frames_run_total", frames_run as i64)
        .int("readouts", readouts as i64)
        .int("readout_stride_frames", d.stride as i64)
        .number("dt_reference_au", d.dt_tables_au)
        .number("dt_in_force_at_the_end_au", sim.dt())
        .flag("the_step_held_to_the_bit", sim.dt().to_bits() == d.step_au.to_bits())
        .number("cell_edge_bohr", l)
        .int("waters", N_WATERS as i64)
        .int("atoms", n_atoms as i64)
        .number("density_g_cm3", DENSITY_G_CM3)
        .number("target_temperature_k", TEMPERATURE_K)
        .number("mean_temperature_k", if counted > 0 { temp_sum / counted as f64 } else { f64::NAN })
        .raw(
            "l0",
            match &void {
                Some((k, p, why)) => format!("{{\"void\": true, \"phase\": {p:?}, \"frame\": {k}, \"why\": {why:?}}}"),
                None => "{\"void\": false}".to_string(),
            },
        )
        .raw(
            "r1",
            format!(
                "{{\"first_peak_position_bohr\": {}, \"first_peak_height\": {}, \"bin_bohr\": {}, \"kill_position_bohr\": [{}, {}], \"kill_height\": [{}, {}]}}",
                num(peak.map(|x| x.0).unwrap_or(f64::NAN)),
                num(peak.map(|x| x.1).unwrap_or(f64::NAN)),
                num(RDF_DR),
                num(KILL_R1_POS.0),
                num(KILL_R1_POS.1),
                num(KILL_R1_HEIGHT.0),
                num(KILL_R1_HEIGHT.1)
            ),
        )
        .raw(
            "r2",
            format!(
                "{{\"hbonds_per_molecule_lens\": {}, \"hbonds_per_molecule_both_ends\": {}, \"kill_lens\": [{}, {}], \"kill_both_ends\": [{}, {}], \"experiment_both_ends\": {}, \"liquid1_lens\": {}, \"refusal\": {}}}",
                num(r2),
                num(2.0 * r2),
                num(r2_band().0),
                num(r2_band().1),
                num(KILL_R2_BOTH_ENDS_BAND.0),
                num(KILL_R2_BOTH_ENDS_BAND.1),
                num(KILL_R2_BOTH_ENDS),
                num(d.liquid1_hbonds),
                match &hb_refusal {
                    Some(x) => format!("{x:?}"),
                    None => "null".to_string(),
                }
            ),
        )
        .raw(
            "r3",
            format!(
                "{{\"diffusion_cm2_per_s\": {}, \"kill_centre_cm2_per_s\": {}, \"kill_factor\": {}, \"refusal\": {}, \"max_lag_frames\": {}, \"sampled_frames\": {}, \"bohr2_per_fs_to_cm2_per_s\": {}, \"au_time_s_engine\": {}, \"au_time_fs_lens\": {}, \"time_units_agree\": {}}}",
                match r3 {
                    Some(x) => num(x),
                    None => "null".to_string(),
                },
                num(KILL_R3_CENTRE),
                num(KILL_R3_FACTOR),
                match &r3_refusal {
                    Some(x) => format!("{x:?}"),
                    None => "null".to_string(),
                },
                max_lag,
                traj.frames.len(),
                num(conv),
                num(holon_render::sim::AU_TIME_S),
                num(AU_TIME_FS),
                ((AU_TIME_FS * 1.0e-15 - holon_render::sim::AU_TIME_S) / holon_render::sim::AU_TIME_S).abs() < 1.0e-9
            ),
        )
        .raw("s_phase", ph.json(false))
        .raw(
            "l1",
            format!(
                "{{\"columns_ok\": {}, \"w_ext\": {}, \"work_hand\": {}, \"work_thermostat\": {}, \"work_field\": {}, \"work_seam\": {}, \"drift_peak\": {}, \"drift_bar\": {}, \"drift_bar_fraction\": {}, \"drift_over_thermostat_work\": {}, \"drift_ok\": {}, \"momentum_residual\": {}, \"momentum_bound\": {}, \"momentum_ok\": {}, \"k_vectors\": {}, \"real_pairs\": {}, \"seam_oo_pairs\": {}, \"seam_ho_pairs\": {}}}",
                sim.work_columns_ok(),
                num(sim.w_ext),
                num(sim.work.hand),
                num(sim.work.thermostat),
                num(sim.work.field),
                num(sim.work.seam),
                num(sim.drift_peak),
                num(drift_bar),
                num(d.drift_fraction),
                num(sim.drift_peak / sim.work.thermostat.abs()),
                sim.drift_peak <= drift_bar,
                num(sim.momentum_residual()),
                num(sim.momentum_bound()),
                sim.momentum_residual() <= sim.momentum_bound(),
                sim.field_work.k_vectors,
                sim.field_work.pairs,
                sim.seam_work.oo_pairs,
                sim.seam_work.ho_pairs
            ),
        )
        .raw("l2", priced.check(wall).json())
        .raw(
            "carrier",
            format!(
                "{{\"first_shell_bohr\": {}, \"first_shell_angstrom\": {}, \"pairs\": {}, \"crossing_a_face\": {}, \"fraction\": {}, \"floor\": {}}}",
                num(first_shell_bohr()),
                num(3.2),
                carrier.pairs,
                carrier.crossing,
                num(carrier.fraction()),
                num(CARRIER_FLOOR)
            ),
        )
        .raw(
            "plants",
            format!(
                "{{\"i_rdf_raw_differences\": {{\"peak_height\": {}, \"peak_position_bohr\": {}}}, \"ii_open_box_hbonds\": {{\"per_molecule\": {}}}, \"iii_spanning_wraps_zeroed\": {{\"spanning_fraction\": {}}}}}",
                num(peak_raw.map(|x| x.1).unwrap_or(f64::NAN)),
                num(peak_raw.map(|x| x.0).unwrap_or(f64::NAN)),
                num(r2_open),
                num(ph.spanning_fraction_local())
            ),
        )
        .text("rdf_refusal", &rdf_refusal.unwrap_or_else(|| "none".to_string()))
        .number("wall_seconds", wall);
    w.write("arm.json", &arm).expect("arm.json writes");
    match &void {
        Some((k, p, why)) => {
            w.write_text("arm.void", &format!("{{\"void\": true, \"phase\": {p:?}, \"frame\": {k}, \"why\": {why:?}}}\n")).ok();
        }
        None => {
            w.done("run.done", &format!("{} counted frames at {:.6} au per step", d.count, dt_at_placement)).expect("run.done writes");
        }
    }
    println!("wall {wall:.1} s over {frames_run} frames");
}

// ------------------------------------------------------------------------ the read

/// One quantity across the seeds: the mean that the band is read against, and the spread that
/// is printed beside it. A band is met only if it contains the MEAN.
struct Across {
    name: String,
    values: Vec<f64>,
}

impl Across {
    fn mean(&self) -> f64 {
        let v: Vec<f64> = self.values.iter().copied().filter(|x| x.is_finite()).collect();
        if v.is_empty() {
            return f64::NAN;
        }
        v.iter().sum::<f64>() / v.len() as f64
    }
    fn sd(&self) -> f64 {
        let v: Vec<f64> = self.values.iter().copied().filter(|x| x.is_finite()).collect();
        if v.len() < 2 {
            return f64::NAN;
        }
        let m = v.iter().sum::<f64>() / v.len() as f64;
        (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
    }
    fn lo(&self) -> f64 {
        self.values.iter().copied().filter(|x| x.is_finite()).fold(f64::INFINITY, f64::min)
    }
    fn hi(&self) -> f64 {
        self.values.iter().copied().filter(|x| x.is_finite()).fold(f64::NEG_INFINITY, f64::max)
    }
    fn n(&self) -> usize {
        self.values.iter().filter(|x| x.is_finite()).count()
    }
    fn json(&self) -> String {
        format!(
            "{{\"seeds\": {}, \"mean\": {}, \"sd\": {}, \"min\": {}, \"max\": {}, \"spread\": {}, \"per_seed\": [{}]}}",
            self.n(),
            num(self.mean()),
            num(self.sd()),
            num(self.lo()),
            num(self.hi()),
            num(self.hi() - self.lo()),
            self.values.iter().map(|&x| num(x)).collect::<Vec<_>>().join(", ")
        )
    }
    fn line(&self) -> String {
        format!(
            "{:<38} mean {:>12.6e}   sd {:>11.6e}   spread {:>11.6e}   over {} seeds",
            self.name,
            self.mean(),
            self.sd(),
            self.hi() - self.lo(),
            self.n()
        )
    }
}

fn read_phase(out: &Path) {
    // every seed that has an arm, refusing to count a screen's or a dry record
    let mut arms: Vec<(usize, String)> = Vec::new();
    for (i, _) in SEEDS.iter().enumerate() {
        let dir = out.join(format!("seed{i}"));
        let path = dir.join("arm.json");
        if !path.exists() {
            println!("seed {i}: no arm.json - not run");
            continue;
        }
        if !is_done(&dir, "run.done") {
            println!("seed {i}: run.done is ABSENT - the arm did not finish and is not counted");
            continue;
        }
        let reading = read_record(&path).unwrap_or_else(|e| panic!("{e}"));
        let counted = reading.count().unwrap_or_else(|e| panic!("{e}"));
        if counted.text.contains("\"void\": true") {
            println!("seed {i}: the arm is VOID (arm.json's l0 block says which frame) - not counted");
            continue;
        }
        arms.push((i, counted.path.clone()));
    }
    assert!(!arms.is_empty(), "no seed produced a countable arm");
    if arms.len() < SEEDS.len() {
        println!(
            "WARNING: {} of {} seeds counted. The spread below is over {} arms and the freeze stakes at least two.",
            arms.len(),
            SEEDS.len(),
            arms.len()
        );
    }
    let gather = |name: &str, keys: &[&str], field: &str| -> Across {
        Across {
            name: name.to_string(),
            values: arms
                .iter()
                .map(|(_, p)| read_input_after(p, keys, field).map(|x| x.value).unwrap_or(f64::NAN))
                .collect(),
        }
    };
    let r1_pos = gather("R1 first O-O peak, bohr", &["\"r1\""], "first_peak_position_bohr");
    let r1_h = gather("R1 first O-O peak height", &["\"r1\""], "first_peak_height");
    let r2 = gather("R2 bonds per molecule, lens", &["\"r2\""], "hbonds_per_molecule_lens");
    let r3 = gather("R3 diffusion, cm2/s", &["\"r3\""], "diffusion_cm2_per_s");
    let span = gather("S spanning fraction", &["\"s_phase\""], "spanning_fraction");
    let largest = gather("S largest component fraction", &["\"s_phase\""], "largest_component_fraction");
    let degree = gather("S mean degree, both ends", &["\"s_phase\""], "mean_degree_both_ends");
    let drift = gather("L1 drift over thermostat work", &["\"l1\""], "drift_over_thermostat_work");
    let settle = gather("settling used, frames", &[], "settle_frames_used");
    let er = read_input_after(&arms[0].1, &["\"design\""], "er_giant").map(|x| x.value).unwrap_or(f64::NAN);

    for a in [&r1_pos, &r1_h, &r2, &r3, &span, &largest, &degree, &drift, &settle] {
        println!("  {}", a.line());
    }

    // the branches, on the SEED MEAN, with the spread printed beside it
    let (lo, hi) = r2_band();
    let r1_branch = if !band(r1_pos.mean(), KILL_R1_POS.0, KILL_R1_POS.1) {
        'c'
    } else if band(r1_h.mean(), KILL_R1_HEIGHT.0, KILL_R1_HEIGHT.1) {
        'a'
    } else {
        'b'
    };
    let r2_branch = if band(r2.mean(), lo, hi) {
        'a'
    } else if r2.mean() < lo {
        'b'
    } else {
        'c'
    };
    let s_branch = if largest.mean() < er {
        'c'
    } else if span.mean() >= 0.5 {
        'a'
    } else {
        'b'
    };
    let (lo3, hi3) = (KILL_R3_CENTRE / KILL_R3_FACTOR, KILL_R3_CENTRE * KILL_R3_FACTOR);
    let r3_live = r3.n() > 0 && r3.mean().is_finite();
    let r3_branch = if !r3_live {
        'v'
    } else if band(r3.mean(), lo3, hi3) {
        'a'
    } else if r3.mean() > hi3 {
        'b'
    } else {
        'c'
    };

    let mut report = Report::new();
    report.gate(
        Gate::new("R1")
            .work(arms.len() as u64)
            .branch(r1_branch)
            .detail(format!(
                "the first oxygen-oxygen peak against experiment's band, on the seed mean; position spread {:.6e} bohr, height spread {:.6e} over {} seeds",
                r1_pos.hi() - r1_pos.lo(),
                r1_h.hi() - r1_h.lo(),
                r1_pos.n()
            ))
            .leg_at("position in band", band(r1_pos.mean(), KILL_R1_POS.0, KILL_R1_POS.1), r1_pos.mean())
            .leg_at("height in band", band(r1_h.mean(), KILL_R1_HEIGHT.0, KILL_R1_HEIGHT.1), r1_h.mean()),
    );
    report.gate(
        Gate::new("R2")
            .work(arms.len() as u64)
            .branch(r2_branch)
            .detail(format!(
                "bonds per molecule ON THE LENS'S OWN each-bond-once count, band [{lo}, {hi}] (experiment's 3.5 both-ends is 1.75 here); LIQUID-1 read 1.184343750e0 on a box it settled for 52 fs; seed spread {:.6e} over {} seeds",
                r2.hi() - r2.lo(),
                r2.n()
            ))
            .leg_at("the seed mean is in band", band(r2.mean(), lo, hi), r2.mean()),
    );
    let g3 = Gate::new("R3").work(arms.len() as u64).detail(format!(
        "self-diffusion against a factor of {KILL_R3_FACTOR} of {KILL_R3_CENTRE:e} cm^2/s on the window the freeze priced; seed spread {:.6e}",
        r3.hi() - r3.lo()
    ));
    report.gate(if r3_live {
        g3.branch(r3_branch).leg_at("the seed mean is in band", band(r3.mean(), lo3, hi3), r3.mean())
    } else {
        g3.void("the lens refused on every seed, or the arm was VOID BY PRICE; each seed's r3 block carries which")
    });
    report.gate(
        Gate::new("S")
            .work(arms.len() as u64)
            .branch(s_branch)
            .detail(format!(
                "the spanning cluster: (a) a majority of sampled frames wind AND the largest component is at least the Erdos-Renyi giant {er:.6} at LIQUID-1's own degree; (b) giant but not winding; (c) no giant. Spreads: spanning {:.6e}, largest {:.6e}",
                span.hi() - span.lo(),
                largest.hi() - largest.lo()
            ))
            .leg_at("the largest component is at least the ER giant", largest.mean() >= er, largest.mean())
            .leg_at("a majority of sampled frames wind", span.mean() >= 0.5, span.mean()),
    );
    println!("R1 ({r1_branch})  R2 ({r2_branch})  R3 ({r3_branch})  S ({s_branch})   on {} seeds", arms.len());

    RecordWriter::new(out)
        .write(
            "read.json",
            &Record::new("read")
                .int("seeds_counted", arms.len() as i64)
                .int("seeds_declared", SEEDS.len() as i64)
                .text("uncertainty_rule", "every readout is the mean across the counted seeds; the spread is min, max, range and the sample standard deviation; a band is met only if it contains the MEAN")
                .raw("gates", report.json())
                .raw("r1_position_bohr", r1_pos.json())
                .raw("r1_height", r1_h.json())
                .raw("r2_hbonds_per_molecule_lens", r2.json())
                .raw("r3_diffusion_cm2_per_s", r3.json())
                .raw("s_spanning_fraction", span.json())
                .raw("s_largest_component_fraction", largest.json())
                .raw("s_mean_degree_both_ends", degree.json())
                .raw("l1_drift_over_thermostat_work", drift.json())
                .raw("settle_frames_used", settle.json())
                .number("s_er_giant", er),
        )
        .expect("read.json writes");
}

// ------------------------------------------------------------------------------- main

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let phase = args.first().cloned().unwrap_or_else(|| "gate".to_string());
    let val = |k: &str| -> Option<String> { args.iter().position(|a| a == k).and_then(|i| args.get(i + 1).cloned()) };
    let flags: Vec<usize> = args.iter().enumerate().filter(|(_, a)| a.starts_with("--")).map(|(i, _)| i + 1).collect();
    let out = PathBuf::from(
        args.iter()
            .enumerate()
            .skip(1)
            .find(|(i, a)| !a.starts_with("--") && !flags.contains(i))
            .map(|(_, a)| a.clone())
            .unwrap_or_else(|| "../conformance/water_observatory/liquid2".to_string()),
    );
    let obs = observatory(&out);
    eprintln!("phase {phase}, out {}, observatory {}", out.display(), obs.display());
    match phase.as_str() {
        "screen" => screen_phase(
            &obs,
            &out,
            val("--step").and_then(|v| v.parse().ok()).unwrap_or(1.0),
            val("--law").map(|v| Variant::parse(&v)).unwrap_or(Variant::Ct3Table),
            val("--label"),
            args.iter().any(|a| a == "--handovers"),
        ),
        "gate" => gate_phase(&obs, &out),
        "size" => size_phase(&obs, &out, val("--cells").and_then(|v| v.parse().ok()).unwrap_or(N_CELLS)),
        "cost" => {
            let placeholder = args.iter().any(|a| a == "--placeholder");
            let (arm, label) = if placeholder { (Arm::Placeholder, "placeholder") } else { (Arm::GeometryFirst, "geometry_first") };
            match val("--merge") {
                Some(name) => cost_merge(&out, label, &name),
                None => cost_phase(&obs, &out, val("--cells").and_then(|v| v.parse().ok()).unwrap_or(N_CELLS), arm, label),
            }
        }
        "run" => run_phase(&obs, &out, val("--seed").and_then(|v| v.parse().ok()).unwrap_or(0)),
        "read" => read_phase(&out),
        other => panic!("unknown phase {other:?}: screen | size | cost | gate | run | read"),
    }
}
