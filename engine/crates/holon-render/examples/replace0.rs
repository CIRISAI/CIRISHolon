//! REPLACE-0 (GANTT2, the fourth review): the first executable holon RUN — rigid water driven
//! through `holon-runtime` against the flexible reference, from ONE branch point, under ONE
//! law (CT-3's served law, the blend at its derived beta, LIQUID-2's own), in one legal box.
//!
//! ```text
//! cargo run --release -p holon-render --example replace0 -- stiffness [DIR]
//! cargo run --release -p holon-render --example replace0 -- run [DIR] --frames N [--settle M] [--readouts R] [--refine] [--reuse] [--match-3n]
//! ```
//!
//! Three things are measured and nothing is assumed:
//!
//! 1. **The contact stiffness the rigid clock needs**, off the served law itself: for a
//!    sample of units, the net force under a rigid translation of the whole unit and the
//!    torque under a rigid rotation about its own centre, differenced; the intramolecular
//!    curvature never enters because the unit moves as one. The envelope is the largest
//!    reading, and `holon-runtime`'s clock takes it under the fine clock's own hold.
//! 2. **The replacement error**: both arms run NVE from the same checkpoint for the same
//!    physical time; the O–O peak, the bond count, the cross-unit potential per water, the
//!    temperature and each arm's own energy excursion are read at the same physical times.
//!    What projection discarded at the branch point is recorded, not netted out.
//! 3. **The cost**, as force passes and seconds per picosecond, both arms, so the speedup is
//!    the step ratio net of every overhead the rigid arm pays (projection, reconstruction,
//!    torque accumulation, the rigid integrator).
//!
//! `--refine` is the demonstration the fourth review re-sized: a unit is heated, the
//! validity rule trips, its first shell is REFINED to fine stepping while the rest are held
//! rigid by projection, and the box re-coarsens after a hysteresis. The cost is then
//! governed by the fraction of physical time spent refined, and the record says so.
//!
//! What this runner does NOT claim: that rigid water is statistically equivalent to the
//! flexible model. Its `Validity` stays `InvariantsOnly`; promoting it is a freeze's
//! decision on these numbers, not this file's.

use holon_campaign::{inefficiency, num, read_input_after, Gate, Record, RecordWriter, Report};
use holon_lens::lens::{hbonds_periodic, rdf_oo};
use holon_render::channel::Row;
use holon_render::checkpoint::Checkpoint;
use holon_render::rigid_adapter::{fine_of, mean_monomer_geometry, project_all, reference_body, reference_body_from, site_forces_of, unit_members, write_back, UnitMembers};
use holon_render::field::{WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD};
use holon_render::seam::{CtLoad, CtServe, CtTable, SeamModel, CT_DIM};
use holon_render::sim::{Boundary, Sim};
use holon_render::thermostat::ThermostatKind;
use holon_render::waterbox::{first_peak, liquid_box};
use holon_runtime::clock::StiffnessEnvelope;
use holon_runtime::rigid::{add, dot, normalize, quat_mul, scale, sub, Body, RigidWater, SiteForces};
use holon_runtime::{Discarded, Operator, Validity};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::{scene, K_B};

// ------------------------------------------------------------------ the state point (LIQUID-2's)

const N_CELLS: usize = 4;
const N_WATERS: usize = 2 * N_CELLS * N_CELLS * N_CELLS;
const DENSITY_G_CM3: f64 = 0.997;
const TEMPERATURE_K: f64 = 293.0;
/// The ASCII of "REPLACE0": disjoint from every LIQUID and PILOT seed by its letters.
const SEED: u64 = 0x5245_504c_4143_4530;
const SEAM_CUTOFF_BOHR: f64 = 14.0;
const RDF_DR: f64 = 0.1;
const AU_TIME_FS: f64 = 0.024_188_843_265_857;

/// The stiffness sample: every `STIFFNESS_EVERY`-th unit in oxygen order.
const STIFFNESS_EVERY: usize = 16;
/// The rigid displacement and rotation the stiffness is differenced over.
const STIFFNESS_H_BOHR: f64 = 2.0e-3;
const STIFFNESS_DELTA_RAD: f64 = 2.0e-3;

/// The refinement demonstration's DECLARED rule: a unit whose rigid kinetic energy exceeds
/// `HOT` times its equipartition share is outside the supported domain and is refined with
/// every unit whose oxygen is within `BUFFER_BOHR`; a refined unit re-coarsens when its
/// projected kinetic energy has stayed under `HOT / 2` of that share for `HYSTERESIS`
/// consecutive readouts. The disturbance SETS the chosen unit's kinetic energy to
/// `DISTURB_SHARE` times its equipartition share, in the rule's own units, so it trips the
/// rule by construction and by a declared margin rather than by luck of the unit's momentum.
const HOT: f64 = 8.0;
const BUFFER_BOHR: f64 = 6.5;
const HYSTERESIS: usize = 3;
const DISTURB_SHARE: f64 = 12.0;

// ------------------------------------------------------------------ the law (LIQUID-2's)

struct Law {
    model: SeamModel,
    table: CtTable,
    beta: f64,
    law_source: String,
    table_source: String,
    beta_source: String,
}

fn json_num(t: &str, key: &str) -> f64 {
    t.split(&format!("\"{key}\": "))
        .nth(1)
        .and_then(|x| x.split(|c| c == ',' || c == '\n' || c == '}').next())
        .and_then(|x| x.trim().parse::<f64>().ok())
        .unwrap_or(f64::NAN)
}

fn load_table(p: &Path) -> CtTable {
    let t = std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let c0 = json_num(&t, "c0_per_bohr");
    let body = t.split("\"sites\": [").nth(1).unwrap_or_else(|| panic!("{}: no sites", p.display()));
    let mut knots: Vec<([f64; CT_DIM], f64)> = Vec::new();
    for chunk in body.split("{\"site\":").skip(1) {
        let y = [json_num(chunk, "r"), json_num(chunk, "cos_theta_d"), json_num(chunk, "u_dot_b"), json_num(chunk, "q")];
        let v = json_num(chunk, "e_ct");
        if y.iter().all(|x| x.is_finite()) && v.is_finite() {
            knots.push((y, v));
        }
    }
    let mut table = CtTable::empty();
    assert!(table.begin(knots.len(), c0), "the table refused {} knots at c0 = {c0}", knots.len());
    for (i, (y, v)) in knots.iter().enumerate() {
        assert!(table.knot(i, *y, *v), "knot {i} refused");
    }
    assert_eq!(table.finish(), CtLoad::Ok, "the transfer table did not load");
    table
}

/// CT-3's served law as LIQUID-2 loads it: the wall from `ct3/wall_ct3.json`, the table from
/// `ct3/ct_table.json`, the blend's beta READ from `ct3/smooth/beta.json`.
fn load_law(obs: &Path) -> Law {
    let p = obs.join("ct3").join("wall_ct3.json");
    let tp = obs.join("ct3").join("ct_table.json");
    let bp = obs.join("ct3").join("smooth").join("beta.json");
    let t = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let g = |k: &str| {
        let v = json_num(&t, k);
        if v.is_finite() {
            v
        } else {
            0.0
        }
    };
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
        p_ct: g("p_ct"),
        c_ct: g("c_ct"),
        m_ct: g("m_ct") as u8,
        k_ct: g("k_ct") as u8,
        lambda_ct: g("lambda_ct"),
        r_cut: SEAM_CUTOFF_BOHR,
        ct_table_on: true,
    };
    let beta = read_input_after(&bp.display().to_string(), &[], "beta_per_bohr").unwrap_or_else(|e| panic!("beta: {e}")).value;
    Law {
        model,
        table: load_table(&tp),
        beta,
        law_source: p.display().to_string(),
        table_source: tp.display().to_string(),
        beta_source: bp.display().to_string(),
    }
}

fn observatory(out: &Path) -> PathBuf {
    let up = out.parent().map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    for c in [up.clone(), PathBuf::from("../conformance/water_observatory"), PathBuf::from("conformance/water_observatory")] {
        if c.join("ct3").join("wall_ct3.json").exists() {
            return c;
        }
    }
    up
}

/// LIQUID-2's box under LIQUID-2's configuration: the blend at the derived beta, the tables'
/// step (the engine's own hold, `allow_dt_growth` never touched), the stochastic thermostat
/// for the settling, the periodic boundary.
fn build(law: &Law) -> (Box<Sim>, f64) {
    let (sp, pos, l) = liquid_box(N_CELLS, DENSITY_G_CM3, SEED);
    let mut sim = scene(&sp, &pos, l, TEMPERATURE_K);
    sim.set_field(true, None).expect("the open box admits the field");
    let mut table = law.table.clone();
    assert!(table.set_blend(law.beta), "the table refused the derived beta");
    sim.ct_table = table;
    sim.set_seam(Some(law.model)).expect("no acuity frame is installed");
    sim.set_thermostat_kind(ThermostatKind::StochasticRescaling, SEED);
    sim.set_boundary(Boundary::Periodic).expect("the periodic box");
    sim.thermostat_on = true;
    (sim, l)
}

// ------------------------------------------------------------------ readings

fn read_pos(sim: &Sim) -> Vec<[f64; 3]> {
    (0..sim.n).map(|i| [sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z]).collect()
}

/// The potential: every ledger row but the kinetic one. Exact whether or not `e_kin` is
/// current, because it is the same two accessors on both arms.
fn potential(sim: &Sim) -> f64 {
    sim.energy() - sim.row(Row::Kin)
}

#[derive(Clone, Copy, Debug)]
struct Obs {
    t_fs: f64,
    temperature_k: f64,
    energy: f64,
    cross_unit_per_water: f64,
    bonds_per_water: f64,
    peak_bohr: f64,
    /// THE ENERGY LEDGER'S ADJUSTMENT, hartree: what a declared disturbance injected is
    /// subtracted, what projection discarded (internal kinetic energy; the potential change at
    /// a coarsening, measured with one pass) is added back, so `energy + ledger_adjust` is the
    /// quantity an NVE arm must conserve. Zero on the flexible arm and on a rigid arm with no
    /// refinement.
    ledger_adjust: f64,
    /// THE RIGID MODES' OWN TEMPERATURE: on the rigid arm the arm's temperature itself; on the
    /// flexible arm every unit projected at the readout and its six retained modes read - the
    /// like-for-like comparator, because a flexible box whose vibrations have not equilibrated
    /// carries its 3N temperature as the average of a hot intermolecular bath and cold
    /// stretches (run 3: 440 K against 319 K), and a rigid replacement inherits the bath.
    rigid_mode_temperature_k: f64,
    /// The vibrational kinetic energy per water in kT at 293 K (the internal kinetic energy the
    /// projection discards): the flexible model's mode split, in one number. Zero on the
    /// rigid arm.
    vibrational_kinetic_per_water_kt: f64,
}

impl Obs {
    fn accounted(&self) -> f64 {
        self.energy + self.ledger_adjust
    }
    fn json(&self) -> String {
        format!(
            "{{\"t_fs\": {}, \"temperature_k\": {}, \"rigid_mode_temperature_k\": {}, \"vibrational_kinetic_per_water_kt\": {}, \"energy_hartree\": {}, \"ledger_adjust_hartree\": {}, \"energy_accounted_hartree\": {}, \"cross_unit_per_water_hartree\": {}, \"bonds_per_water\": {}, \"oo_first_peak_bohr\": {}}}",
            num(self.t_fs),
            num(self.temperature_k),
            num(self.rigid_mode_temperature_k),
            num(self.vibrational_kinetic_per_water_kt),
            num(self.energy),
            num(self.ledger_adjust),
            num(self.accounted()),
            num(self.cross_unit_per_water),
            num(self.bonds_per_water),
            num(self.peak_bohr)
        )
    }
}

fn observe(sim: &Sim, z: &[u32], l: f64, t_fs: f64, temperature_k: f64, energy: f64, ledger_adjust: f64, rigid_mode_temperature_k: f64, vibrational_kinetic_per_water_kt: f64) -> Obs {
    let p = read_pos(sim);
    let cell = [l, l, l];
    let bonds = hbonds_periodic(&p, z, cell).map(|v| v.len() as f64 / N_WATERS as f64).unwrap_or(f64::NAN);
    let peak = rdf_oo(&p, z, cell, RDF_DR, 0.5 * l).ok().and_then(|r| first_peak(&r.r, &r.g)).map(|(r, _)| r).unwrap_or(f64::NAN);
    Obs {
        t_fs,
        temperature_k,
        energy,
        cross_unit_per_water: (sim.row(Row::Field) + sim.row(Row::Seam)) / N_WATERS as f64,
        bonds_per_water: bonds,
        peak_bohr: peak,
        ledger_adjust,
        rigid_mode_temperature_k,
        vibrational_kinetic_per_water_kt,
    }
}

/// The fine box's mode split at this instant: every unit projected, its six retained modes'
/// temperature and the vibrational kinetic energy per water in kT. Projection reads; it
/// writes nothing back.
fn mode_split(sim: &Sim, units: &[UnitMembers], body: &Body) -> (f64, f64) {
    let mut ke_rigid = 0.0;
    let mut ke_vib = 0.0;
    for m in units {
        let (w, d) = RigidWater::project(&fine_of(sim, m, body)).expect("a unit projects at a readout");
        ke_rigid += w.kinetic();
        ke_vib += d.internal_kinetic;
    }
    let n = units.len().max(1) as f64;
    (2.0 * ke_rigid / (6.0 * n * K_B), ke_vib / n / (K_B * TEMPERATURE_K))
}

fn series_json(s: &[Obs]) -> String {
    format!("[{}]", s.iter().map(|o| o.json()).collect::<Vec<_>>().join(", "))
}

/// The second half of a series, summarised with its autocorrelation-aware error.
fn half_summary(s: &[Obs], f: impl Fn(&Obs) -> f64) -> (f64, f64, f64) {
    let tail: Vec<f64> = s[s.len() / 2..].iter().map(f).filter(|x| x.is_finite()).collect();
    if tail.is_empty() {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    let i = inefficiency(&tail);
    (i.mean, i.sem, i.g)
}

// ------------------------------------------------------------------ the stiffness

struct Stiffness {
    k_translation_max: f64,
    kappa_rotation_max: f64,
    /// The envelope handed to the clock: `max(k_T, kappa / lever_max^2)`, so `derive`'s
    /// `sqrt(k rho^2 / I_min)` is never below the measured rotational frequency.
    k_envelope: f64,
    samples: Vec<(usize, &'static str, usize, f64)>,
    units_sampled: usize,
    passes: u64,
    seconds: f64,
}

impl Stiffness {
    fn json(&self) -> String {
        format!(
            "{{\"k_translation_max_hartree_per_bohr2\": {}, \"kappa_rotation_max_hartree_per_rad2\": {}, \"k_envelope_hartree_per_bohr2\": {}, \"units_sampled\": {}, \"every\": {}, \"h_bohr\": {}, \"delta_rad\": {}, \"passes\": {}, \"seconds\": {}, \"rule\": \"for every sampled unit, the net force under a rigid translation of the WHOLE unit and the torque under a rigid rotation about its own centre, centrally differenced; the intramolecular curvature never enters because the unit moves as one; the envelope is the largest reading and the clock takes it under the fine clock's own hold\", \"samples\": [{}]}}",
            num(self.k_translation_max),
            num(self.kappa_rotation_max),
            num(self.k_envelope),
            self.units_sampled,
            STIFFNESS_EVERY,
            num(STIFFNESS_H_BOHR),
            num(STIFFNESS_DELTA_RAD),
            self.passes,
            num(self.seconds),
            self.samples
                .iter()
                .map(|(u, kind, axis, k)| format!("{{\"unit\": {u}, \"kind\": \"{kind}\", \"axis\": {axis}, \"value\": {}}}", num(*k)))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn measure_stiffness(sim: &mut Sim, units: &[UnitMembers], body: &Body) -> Stiffness {
    let t0 = Instant::now();
    let mut passes = 0u64;
    let mut samples = Vec::new();
    let mut k_t = 0.0f64;
    let mut kappa = 0.0f64;
    let mut sampled = 0;
    for (u, m) in units.iter().enumerate().step_by(STIFFNESS_EVERY) {
        sampled += 1;
        let fine0 = fine_of(sim, m, body);
        let (w, _) = RigidWater::project(&fine0).expect("a unit projects");
        let mut force_torque_at = |b: &RigidWater| -> ([f64; 3], [f64; 3]) {
            write_back(sim, m, &b.reconstruct());
            sim.compute_forces();
            passes += 1;
            b.accumulate(&site_forces_of(sim, m))
        };
        for axis in 0..3 {
            let mut plus = w;
            plus.com[axis] += STIFFNESS_H_BOHR;
            let mut minus = w;
            minus.com[axis] -= STIFFNESS_H_BOHR;
            let (fp, _) = force_torque_at(&plus);
            let (fm, _) = force_torque_at(&minus);
            let k = -(fp[axis] - fm[axis]) / (2.0 * STIFFNESS_H_BOHR);
            samples.push((u, "translation", axis, k));
            k_t = k_t.max(k);
        }
        for axis in 0..3 {
            let rotated = |d: f64| {
                let (hs, hc) = (0.5 * d).sin_cos();
                let mut qn = [hc, 0.0, 0.0, 0.0];
                qn[1 + axis] = hs;
                let mut b = w;
                b.q = quat_mul(&qn, &w.q);
                normalize(&mut b.q);
                b
            };
            let (_, tp) = force_torque_at(&rotated(STIFFNESS_DELTA_RAD));
            let (_, tm) = force_torque_at(&rotated(-STIFFNESS_DELTA_RAD));
            let k = -(tp[axis] - tm[axis]) / (2.0 * STIFFNESS_DELTA_RAD);
            samples.push((u, "rotation", axis, k));
            kappa = kappa.max(k);
        }
        // the unit put back exactly as it was read
        write_back(sim, m, &fine0);
    }
    sim.compute_forces();
    passes += 1;
    let k_envelope = k_t.max(kappa / (body.lever_max * body.lever_max));
    Stiffness { k_translation_max: k_t, kappa_rotation_max: kappa, k_envelope, samples, units_sampled: sampled, passes, seconds: t0.elapsed().as_secs_f64() }
}

// ------------------------------------------------------------------ the arms

struct ArmResult {
    series: Vec<Obs>,
    passes: u64,
    seconds: f64,
    physical_fs: f64,
    steps: u64,
    dt_au: f64,
}

impl ArmResult {
    fn core_seconds_per_ps(&self) -> f64 {
        self.seconds / (self.physical_fs / 1000.0)
    }
    fn json(&self, label: &str) -> String {
        let (e0, e_peak, e_end) = energy_excursion(&self.series);
        let (_, a_peak, a_end) = energy_excursion_accounted(&self.series);
        format!(
            "{{\"arm\": \"{label}\", \"steps\": {}, \"dt_au\": {}, \"dt_fs\": {}, \"physical_fs\": {}, \"force_passes\": {}, \"seconds\": {}, \"core_seconds_per_ps\": {}, \"energy_start_hartree\": {}, \"energy_excursion_peak_per_water_hartree\": {}, \"energy_drift_end_per_water_hartree\": {}, \"energy_accounted_excursion_peak_per_water_hartree\": {}, \"energy_accounted_drift_end_per_water_hartree\": {}, \"series\": {}}}",
            self.steps,
            num(self.dt_au),
            num(self.dt_au * AU_TIME_FS),
            num(self.physical_fs),
            self.passes,
            num(self.seconds),
            num(self.core_seconds_per_ps()),
            num(e0),
            num(e_peak / N_WATERS as f64),
            num(e_end / N_WATERS as f64),
            num(a_peak / N_WATERS as f64),
            num(a_end / N_WATERS as f64),
            series_json(&self.series)
        )
    }
}

fn energy_excursion(s: &[Obs]) -> (f64, f64, f64) {
    excursion_of(s, |o| o.energy)
}

fn energy_excursion_accounted(s: &[Obs]) -> (f64, f64, f64) {
    excursion_of(s, |o| o.accounted())
}

fn excursion_of(s: &[Obs], f: impl Fn(&Obs) -> f64) -> (f64, f64, f64) {
    if s.is_empty() {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    let e0 = f(&s[0]);
    let peak = s.iter().map(|o| (f(o) - e0).abs()).fold(0.0, f64::max);
    (e0, peak, f(&s[s.len() - 1]) - e0)
}

/// THE FLEXIBLE REFERENCE: the engine's own integrator at the tables' step, NVE.
fn run_flexible(sim: &mut Sim, z: &[u32], l: f64, frames: usize, stride: usize, units: &[UnitMembers], body: &Body) -> ArmResult {
    let t0 = Instant::now();
    let dt = sim.dt();
    let mut series = Vec::new();
    sim.compute_forces();
    let (tr, vib) = mode_split(sim, units, body);
    series.push(observe(sim, z, l, 0.0, sim.temperature(), sim.energy(), 0.0, tr, vib));
    for k in 0..frames {
        sim.step_frame(1);
        if (k + 1) % stride == 0 {
            let (tr, vib) = mode_split(sim, units, body);
            series.push(observe(sim, z, l, (k + 1) as f64 * dt * AU_TIME_FS, sim.temperature(), sim.energy(), 0.0, tr, vib));
            let o = series.last().unwrap();
            eprintln!("  flexible frame {:>7}: T {:6.1} K (rigid modes {:6.1} K, vibrations {:.2} kT/water), E {:.6} Ha, bonds {:.3}, peak {:.2}", k + 1, o.temperature_k, o.rigid_mode_temperature_k, o.vibrational_kinetic_per_water_kt, o.energy, o.bonds_per_water, o.peak_bohr);
        }
    }
    ArmResult { series, passes: frames as u64 + 1, seconds: t0.elapsed().as_secs_f64(), physical_fs: frames as f64 * dt * AU_TIME_FS, steps: frames as u64, dt_au: dt }
}

/// One rigid unit in flight, with the last force the law left on its sites.
struct Flying {
    m: UnitMembers,
    w: RigidWater,
    f: SiteForces,
}

fn rigid_kinetic(bodies: &[Flying]) -> f64 {
    bodies.iter().map(|b| b.w.kinetic()).sum()
}

/// The rigid box's temperature on its RETAINED degrees of freedom: six per unit.
fn rigid_temperature(bodies: &[Flying]) -> f64 {
    2.0 * rigid_kinetic(bodies) / (6.0 * bodies.len() as f64 * K_B)
}

/// One velocity-Verlet step of every body under the law: half-kick on the stored force, free
/// flight, ONE force pass at the new sites, half-kick. Exactly one pass per step.
fn rigid_step(sim: &mut Sim, bodies: &mut [Flying], dt: f64) {
    for b in bodies.iter_mut() {
        b.w.kick(0.5 * dt, &b.f);
        b.w.free_flight(dt);
        write_back(sim, &b.m, &b.w.reconstruct());
    }
    sim.compute_forces();
    for b in bodies.iter_mut() {
        b.f = site_forces_of(sim, &b.m);
        b.w.kick(0.5 * dt, &b.f);
    }
}

struct RefineEvent {
    t_fs: f64,
    what: String,
    units: usize,
    discarded_deformation_rms: f64,
    discarded_internal_kinetic: f64,
}

struct RigidRun {
    result: ArmResult,
    discarded_at_branch: (f64, f64),
    clock_dt_au: f64,
    clock_omega_dt: f64,
    clock_omega_translation: f64,
    clock_omega_rotation: f64,
    refine_events: Vec<RefineEvent>,
    fine_frames_while_refined: u64,
    physical_fs_refined: f64,
    hot_unit: Option<usize>,
    /// The rigid modes' temperature AS PROJECTED from the fine state, before any matching:
    /// the diagnostic that says whether the fine box was equipartitioned between its
    /// vibrations and its rigid modes (the first counted run: 446 K against the box's 319 K -
    /// the vibrations were cold, the projection kept their angular momentum as rotation).
    t_projected_k: f64,
    /// The temperature the rigid momenta were rescaled to (the fine box's own 3N reading at
    /// the branch), and the kinetic energy that rescaling removed. A DECLARED step: the
    /// coarse state is initialised at the fine state's temperature on its retained modes.
    t_matched_k: f64,
    kinetic_removed_by_matching: f64,
}

/// THE RIGID ARM, and with `refine` the demonstration.
fn run_rigid(sim: &mut Sim, z: &[u32], l: f64, body: &Body, k_envelope: f64, frames: usize, stride: usize, refine: bool, match_k: Option<f64>) -> RigidRun {
    let t0 = Instant::now();
    let dt_f = sim.dt();
    let (projected, other) = project_all(sim, body).expect("every unit projects at the branch point");
    assert!(other.is_empty(), "units that are not water at the branch point: {other:?}");
    let n = projected.len();
    let def_rms = (projected.iter().map(|(_, _, d)| d.deformation_rms * d.deformation_rms).sum::<f64>() / n as f64).sqrt();
    let ke_int: f64 = projected.iter().map(|(_, _, d)| d.internal_kinetic).sum();
    let env = StiffnessEnvelope { k_max: k_envelope };
    let clock = projected[0].1.clock(&env);
    // the readout period is the flexible arm's, and the rigid step is the largest step the
    // clock admits that divides it exactly, so both arms are read at the same physical times
    let period = stride as f64 * dt_f;
    let k_r = (period / clock.dt).ceil().max(1.0) as u64;
    let dt_r = period / k_r as f64;
    let readouts = frames / stride;
    eprintln!(
        "rigid clock: omega_T {:.3e}, omega_R {:.3e} au^-1; dt {:.4} au admitted ({:.2}x the fine {:.4}); {k_r} steps per readout at dt {:.4} au",
        clock.omega_translation, clock.omega_rotation, clock.dt, clock.dt / dt_f, dt_f, dt_r
    );
    let mut bodies: Vec<Flying> = Vec::with_capacity(n);
    for (m, w, _) in projected {
        bodies.push(Flying { m, w, f: SiteForces::default() });
    }
    // TEMPERATURE MATCHING, declared and recorded: the rigid modes as projected carry
    // whatever the fine state's velocities put on them - including the angular momentum of
    // its vibrations - so their temperature is read first, then every momentum is rescaled
    // to the fine box's own temperature and the kinetic energy removed is written down.
    let t_projected = rigid_temperature(&bodies);
    let ke_before = rigid_kinetic(&bodies);
    let (t_matched, kinetic_removed) = match match_k {
        Some(t) => {
            let s = (t / t_projected).sqrt();
            for b in bodies.iter_mut() {
                b.w.p = scale(b.w.p, s);
                b.w.l_body = scale(b.w.l_body, s);
            }
            let removed = ke_before - rigid_kinetic(&bodies);
            eprintln!("  rigid modes as projected: {t_projected:.1} K; rescaled to {t:.1} K, removing {removed:.3e} Ha");
            (t, removed)
        }
        None => {
            eprintln!("  rigid modes as projected: {t_projected:.1} K; kept - the rigid arm inherits the fine box's intermolecular bath as it is");
            (t_projected, 0.0)
        }
    };
    let match_k = t_matched;
    for b in bodies.iter() {
        write_back(sim, &b.m, &b.w.reconstruct());
    }
    sim.compute_forces();
    let mut passes = 1u64;
    for b in bodies.iter_mut() {
        b.f = site_forces_of(sim, &b.m);
    }
    let mut series = Vec::new();
    let e = rigid_kinetic(&bodies) + potential(sim);
    series.push(observe(sim, z, l, 0.0, rigid_temperature(&bodies), e, 0.0, rigid_temperature(&bodies), 0.0));
    // THE LEDGER: injected by the declared disturbance, discarded by projection
    let mut injected = 0.0f64;
    let mut discarded_cum = 0.0f64;
    eprintln!("  rigid branch: T {:.1} K on 6 dof/unit, E {:.6} Ha; discarded {:.3e} bohr rms, {:.3e} Ha internal KE", series[0].temperature_k, e, def_rms, ke_int);

    // the demonstration's state
    let share = 3.0 * K_B * TEMPERATURE_K;
    let mut refined: Vec<usize> = Vec::new();
    let mut quiet: Vec<usize> = vec![0; n];
    let mut events = Vec::new();
    let mut fine_frames = 0u64;
    let mut fs_refined = 0.0;
    let hot_unit = if refine {
        // the unit nearest the box centre
        let c = 0.5 * l;
        Some(
            (0..n)
                .min_by(|&a, &b| {
                    let da = dot(sub(bodies[a].w.com, [c; 3]), sub(bodies[a].w.com, [c; 3]));
                    let db = dot(sub(bodies[b].w.com, [c; 3]), sub(bodies[b].w.com, [c; 3]));
                    da.partial_cmp(&db).unwrap()
                })
                .unwrap(),
        )
    } else {
        None
    };
    let disturb_at = readouts / 4;
    let mut steps = 0u64;
    // the physical clock, advanced by every rigid step and every fine frame
    let mut t_au = 0.0f64;

    for r in 0..readouts {
        let t_start_fs = r as f64 * period * AU_TIME_FS;
        if refine && r == disturb_at {
            let u = hot_unit.unwrap();
            let before = bodies[u].w.kinetic();
            let s = (DISTURB_SHARE * share / before.max(1e-300)).sqrt();
            bodies[u].w.p = scale(bodies[u].w.p, s);
            bodies[u].w.l_body = scale(bodies[u].w.l_body, s);
            injected += bodies[u].w.kinetic() - before;
            events.push(RefineEvent { t_fs: t_start_fs, what: format!("disturbance: unit {u}'s kinetic energy set to {DISTURB_SHARE} times its equipartition share (from {:.2e} Ha)", before), units: 1, discarded_deformation_rms: 0.0, discarded_internal_kinetic: 0.0 });
            eprintln!("  DISTURBANCE at {t_start_fs:.1} fs: unit {u} kinetic {:.3e} Ha against the share {:.3e}", bodies[u].w.kinetic(), share);
        }
        if refined.is_empty() {
            // THE RIGID STEP, and the validity rule checked on every step
            for _ in 0..k_r {
                rigid_step(sim, &mut bodies, dt_r);
                passes += 1;
                steps += 1;
                t_au += dt_r;
                if refine {
                    let hot: Vec<usize> = (0..n).filter(|&u| bodies[u].w.kinetic() > HOT * share).collect();
                    if !hot.is_empty() {
                        // REFINE: the hot units and their first shell go fine; the sites are
                        // already written back, so the fine state IS the rigid one at this instant
                        let mut set: Vec<usize> = Vec::new();
                        for &h in &hot {
                            for u in 0..n {
                                let d = sub(bodies[u].w.com, bodies[h].w.com);
                                let d = [d[0] - l * (d[0] / l).round(), d[1] - l * (d[1] / l).round(), d[2] - l * (d[2] / l).round()];
                                if dot(d, d).sqrt() <= BUFFER_BOHR && !set.contains(&u) {
                                    set.push(u);
                                }
                            }
                        }
                        set.sort_unstable();
                        for &u in &set {
                            quiet[u] = 0;
                        }
                        let t_fs = t_au * AU_TIME_FS;
                        eprintln!("  REFINE at {t_fs:.1} fs: {} hot, {} units to fine stepping (buffer {BUFFER_BOHR} bohr)", hot.len(), set.len());
                        events.push(RefineEvent { t_fs, what: format!("refine: {} hot unit(s), first shell of {}", hot.len(), set.len()), units: set.len(), discarded_deformation_rms: 0.0, discarded_internal_kinetic: 0.0 });
                        refined = set;
                        break;
                    }
                }
            }
            if refined.is_empty() {
                let e = rigid_kinetic(&bodies) + potential(sim);
                series.push(observe(sim, z, l, (r + 1) as f64 * period * AU_TIME_FS, rigid_temperature(&bodies), e, discarded_cum - injected, rigid_temperature(&bodies), 0.0));
                let o = series.last().unwrap();
                eprintln!("  rigid readout {:>4} at {:8.1} fs: T {:6.1} K, E {:.6} Ha, bonds {:.3}, peak {:.2}", r + 1, o.t_fs, o.temperature_k, o.energy, o.bonds_per_water, o.peak_bohr);
                continue;
            }
        }
        // FINE STEPPING while any unit is refined: the engine integrates every atom at the
        // fine step; after each frame the units NOT refined are projected back onto their
        // rigid state and reconstructed (held rigid by projection, what that discards
        // counted), the refined ones are left as the engine leaves them.
        let frames_this = (period / dt_f).round() as usize;
        let mut held_def = 0.0f64;
        let mut held_ke = 0.0f64;
        for _ in 0..frames_this {
            sim.step_frame(1);
            passes += 1;
            fine_frames += 1;
            t_au += dt_f;
            for u in 0..n {
                if refined.contains(&u) {
                    continue;
                }
                let fine = fine_of(sim, &bodies[u].m, body);
                let (w, d) = RigidWater::project(&fine).expect("a held unit projects");
                held_def += d.deformation_rms;
                held_ke += d.internal_kinetic;
                discarded_cum += d.internal_kinetic;
                bodies[u].w = w;
                write_back(sim, &bodies[u].m, &w.reconstruct());
            }
        }
        fs_refined += period * AU_TIME_FS;
        // the refined units' own trial projection decides re-coarsening
        let mut coarsen: Vec<usize> = Vec::new();
        for &u in &refined {
            let (w, _) = RigidWater::project(&fine_of(sim, &bodies[u].m, body)).expect("projects");
            if w.kinetic() < 0.5 * HOT * share {
                quiet[u] += 1;
            } else {
                quiet[u] = 0;
            }
            if quiet[u] >= HYSTERESIS {
                coarsen.push(u);
            }
        }
        // the forces at the readout: one pass so every held body's stored force is current
        sim.compute_forces();
        passes += 1;
        for u in 0..n {
            if !refined.contains(&u) {
                bodies[u].f = site_forces_of(sim, &bodies[u].m);
            }
        }
        // the readout: retained KE on the held bodies plus the engine's on the refined atoms
        let mut ke = 0.0;
        for u in 0..n {
            if refined.contains(&u) {
                for &i in [bodies[u].m.o, bodies[u].m.h[0], bodies[u].m.h[1]].iter() {
                    let a = &sim.atoms[i];
                    ke += 0.5 * a.mass() * (a.vx * a.vx + a.vy * a.vy + a.vz * a.vz);
                }
            } else {
                ke += bodies[u].w.kinetic();
            }
        }
        let dof = 6.0 * (n - refined.len()) as f64 + 9.0 * refined.len() as f64;
        let t_fs = (r + 1) as f64 * period * AU_TIME_FS;
        // the rigid-mode temperature over ALL units at this readout: the held ones as they are,
        // the refined ones projected (a reading, not a write)
        let mut ke_modes = 0.0;
        for u in 0..n {
            if refined.contains(&u) {
                let (w, _) = RigidWater::project(&fine_of(sim, &bodies[u].m, body)).expect("projects");
                ke_modes += w.kinetic();
            } else {
                ke_modes += bodies[u].w.kinetic();
            }
        }
        series.push(observe(sim, z, l, t_fs, 2.0 * ke / (dof * K_B), ke + potential(sim), discarded_cum - injected, 2.0 * ke_modes / (6.0 * n as f64 * K_B), 0.0));
        let o = series.last().unwrap();
        eprintln!("  FINE  readout {:>4} at {:8.1} fs: T {:6.1} K, E {:.6} Ha, bonds {:.3}, peak {:.2}; {} refined, held units discarded {:.2e} bohr / {:.2e} Ha per frame", r + 1, o.t_fs, o.temperature_k, o.energy, o.bonds_per_water, o.peak_bohr, refined.len(), held_def / frames_this as f64 / (n - refined.len()).max(1) as f64, held_ke / frames_this as f64);
        if !coarsen.is_empty() {
            let mut def = 0.0;
            let mut kei = 0.0;
            let u_before = potential(sim);
            for &u in &coarsen {
                let (w, d) = RigidWater::project(&fine_of(sim, &bodies[u].m, body)).expect("projects");
                def += d.deformation_rms;
                kei += d.internal_kinetic;
                bodies[u].w = w;
                write_back(sim, &bodies[u].m, &w.reconstruct());
            }
            refined.retain(|u| !coarsen.contains(u));
            // ONE pass at the re-projected sites: the potential the coarsening moved is
            // measured and ledgered with the kinetic energy it dropped, and every body's
            // stored force is current for whatever steps next
            sim.compute_forces();
            passes += 1;
            let du = potential(sim) - u_before;
            discarded_cum += kei - du;
            for u in 0..n {
                if !refined.contains(&u) {
                    bodies[u].f = site_forces_of(sim, &bodies[u].m);
                }
            }
            eprintln!("  COARSEN at {t_fs:.1} fs: {} unit(s) back to rigid ({} still refined); discarded {:.3e} bohr rms, {:.3e} Ha kinetic, potential moved {:+.3e} Ha", coarsen.len(), refined.len(), def / coarsen.len() as f64, kei, du);
            events.push(RefineEvent { t_fs, what: format!("coarsen: {} unit(s), {} still refined; potential moved {:.3e} Ha (signed)", coarsen.len(), refined.len(), du), units: coarsen.len(), discarded_deformation_rms: def / coarsen.len() as f64, discarded_internal_kinetic: kei });
        }
    }
    let physical_fs = readouts as f64 * period * AU_TIME_FS;
    RigidRun {
        result: ArmResult { series, passes, seconds: t0.elapsed().as_secs_f64(), physical_fs, steps, dt_au: dt_r },
        discarded_at_branch: (def_rms, ke_int),
        clock_dt_au: clock.dt,
        clock_omega_dt: clock.omega_dt,
        clock_omega_translation: clock.omega_translation,
        clock_omega_rotation: clock.omega_rotation,
        refine_events: events,
        fine_frames_while_refined: fine_frames,
        physical_fs_refined: fs_refined,
        hot_unit,
        t_projected_k: t_projected,
        t_matched_k: match_k,
        kinetic_removed_by_matching: kinetic_removed,
    }
}

// ------------------------------------------------------------------ the reference bundle

/// The flexible arm's series as text, one readout per line, with a header naming the frames
/// and stride it was taken at so a run at another length cannot reuse it.
fn write_flexible_series(path: &Path, f: &ArmResult, frames: usize, stride: usize) {
    let mut t = format!("# flexible2 {frames} {stride} {} {} {} {} {}\n", f.passes, f.seconds, f.physical_fs, f.steps, f.dt_au);
    for o in &f.series {
        t.push_str(&format!("{} {} {} {} {} {} {} {}\n", o.t_fs, o.temperature_k, o.energy, o.cross_unit_per_water, o.bonds_per_water, o.peak_bohr, o.rigid_mode_temperature_k, o.vibrational_kinetic_per_water_kt));
    }
    std::fs::write(path, t).expect("flexible.series writes");
}

fn read_flexible_series(path: &Path, frames: usize, stride: usize) -> Option<ArmResult> {
    let t = std::fs::read_to_string(path).ok()?;
    let mut lines = t.lines();
    let head: Vec<&str> = lines.next()?.split_whitespace().collect();
    if head.len() != 9 || head[1] != "flexible2" || head[2].parse::<usize>().ok()? != frames || head[3].parse::<usize>().ok()? != stride {
        return None;
    }
    let mut series = Vec::new();
    for line in lines {
        let v: Vec<f64> = line.split_whitespace().filter_map(|x| x.parse().ok()).collect();
        if v.len() == 8 {
            series.push(Obs { t_fs: v[0], temperature_k: v[1], energy: v[2], cross_unit_per_water: v[3], bonds_per_water: v[4], peak_bohr: v[5], ledger_adjust: 0.0, rigid_mode_temperature_k: v[6], vibrational_kinetic_per_water_kt: v[7] });
        }
    }
    Some(ArmResult { series, passes: head[4].parse().ok()?, seconds: head[5].parse().ok()?, physical_fs: head[6].parse().ok()?, steps: head[7].parse().ok()?, dt_au: head[8].parse().ok()? })
}

// ------------------------------------------------------------------ the phases

fn settle(sim: &mut Sim, frames: usize) {
    for k in 0..frames {
        sim.step_frame(1);
        if (k + 1) % 500 == 0 || k + 1 == frames {
            eprintln!("  settling frame {:>6}: T {:6.1} K, cross-unit U {:.6e} Ha/water", k + 1, sim.temperature(), (sim.row(Row::Field) + sim.row(Row::Seam)) / N_WATERS as f64);
        }
    }
}

fn stiffness_phase(obs: &Path, out: &Path, settle_frames: usize) {
    let w = RecordWriter::new(out);
    let law = load_law(obs);
    let (mut sim, _l) = build(&law);
    settle(&mut sim, settle_frames);
    sim.thermostat_on = false;
    sim.compute_forces();
    let body = reference_body().expect("the pinned monomer is principal");
    let (units, other) = unit_members(&sim.units_reading());
    assert!(other.is_empty());
    let s = measure_stiffness(&mut sim, &units, &body);
    println!("stiffness: k_T max {:.4e} Ha/bohr^2, kappa max {:.4e} Ha/rad^2, envelope {:.4e} Ha/bohr^2 over {} units, {} passes, {:.1} s", s.k_translation_max, s.kappa_rotation_max, s.k_envelope, s.units_sampled, s.passes, s.seconds);
    let clock = RigidWater { body, com: [0.0; 3], q: [1.0, 0.0, 0.0, 0.0], p: [0.0; 3], l_body: [0.0; 3] }.clock(&StiffnessEnvelope { k_max: s.k_envelope });
    println!("the clock on it: omega_T {:.3e}, omega_R {:.3e}; dt {:.4} au = {:.4} fs against the fine {:.4} au ({:.2}x); fine omega_env*dt {:.4}", clock.omega_translation, clock.omega_rotation, clock.dt, clock.dt * AU_TIME_FS, sim.dt(), clock.dt / sim.dt(), sim.timescale.omega_dt());
    let rec = Record::new("stiffness")
        .int("settle_frames", settle_frames as i64)
        .raw("stiffness", s.json())
        .number("fine_dt_au", sim.dt())
        .number("fine_omega_dt", sim.timescale.omega_dt())
        .number("rigid_dt_au", clock.dt)
        .number("rigid_over_fine", clock.dt / sim.dt())
        .number("rigid_omega_dt", clock.omega_dt)
        .text("law_source", &law.law_source)
        .text("table_source", &law.table_source)
        .text("beta_source", &law.beta_source);
    w.write("stiffness.json", &rec).expect("stiffness.json writes");
    w.done("stiffness.done", "the contact stiffness measured off the served law under rigid motions").expect("done");
}

fn run_phase(obs: &Path, out: &Path, frames: usize, settle_frames: usize, readouts: usize, refine: bool, reuse: bool, match_3n: bool) {
    let w = RecordWriter::new(out);
    let law = load_law(obs);
    let (mut sim, l) = build(&law);
    let z: Vec<u32> = (0..sim.n).map(|i| sim.atoms[i].species.z).collect();
    let mut report = Report::new();

    // CONFIG, read back from the objects (M-VALIDATED-NOT-WIRED's rule)
    let g_config = Gate::new("CONFIG")
        .work(4)
        .detail(format!("serve_mode {:?}, beta {} ({}), step {} au (the engine's hold, allow_dt_growth {}), boundary {:?}", sim.ct_table.serve_mode(), num(sim.ct_table.beta()), law.beta_source, num(sim.dt()), sim.timescale.allow_dt_growth, sim.boundary))
        .leg("the table's serve_mode() reads the blend", sim.ct_table.serve_mode() == CtServe::Blend)
        .leg_at("the table's beta() is the derived beta, EXACT", sim.ct_table.beta().to_bits() == law.beta.to_bits(), sim.ct_table.beta())
        .leg("the engine's own hold is in force (allow_dt_growth off)", !sim.timescale.allow_dt_growth)
        .leg("the box is periodic", sim.boundary == Boundary::Periodic);
    report.gate(g_config);

    // THE REFERENCE BUNDLE (WP1): the branch checkpoint and the flexible arm's series are
    // written beside the record and REUSED by a later run of the rigid arms, so an operator
    // change does not pay for the settling and the reference again. A reused flexible series
    // is the same trajectory to the bit (the checkpoint restores it), and the record says
    // which it was.
    let ck_path = out.join("branch.ckpt");
    let reuse = reuse && ck_path.exists();
    let branch = if reuse {
        let bytes = std::fs::read(&ck_path).expect("branch.ckpt reads");
        let ck = Checkpoint { bytes };
        sim.restore(&ck).expect("the branch checkpoint restores into the same box");
        sim.thermostat_on = false;
        sim.compute_forces();
        eprintln!("branch point REUSED from {} (digest {:#x})", ck_path.display(), ck.digest());
        ck
    } else {
        eprintln!("settling {settle_frames} frames under the stochastic thermostat, then NVE from the branch point");
        settle(&mut sim, settle_frames);
        sim.thermostat_on = false;
        sim.compute_forces();
        let ck = sim.checkpoint();
        std::fs::write(&ck_path, &ck.bytes).expect("branch.ckpt writes");
        eprintln!("branch point WRITTEN to {} (digest {:#x})", ck_path.display(), ck.digest());
        ck
    };
    let t_branch = sim.temperature();
    // THE LIFT'S REFERENCE GEOMETRY IS THE LIQUID'S OWN MEAN AT THE BRANCH, measured and
    // recorded beside the pin it is not. (The first matched run carried the pin and released
    // ~0.7 kT per water on the snap.)
    let (units_at_branch, _) = unit_members(&sim.units_reading());
    let geom = mean_monomer_geometry(&sim, &units_at_branch);
    let body = reference_body_from(geom.r_oh_bohr, geom.theta_rad).expect("the mean monomer is principal");
    eprintln!(
        "reference geometry at the branch: O-H {:.5} +/- {:.5} bohr, H-O-H {:.5} +/- {:.5} rad over {} units (the pin: {:.5} bohr, {:.5} rad)",
        geom.r_oh_bohr, geom.r_oh_sd_bohr, geom.theta_rad, geom.theta_sd_rad, geom.units, WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD
    );

    // the stiffness, measured on the branch state unless a record already exists
    let stiff_path = out.join("stiffness.json");
    let k_envelope = if stiff_path.exists() {
        let v = read_input_after(&stiff_path.display().to_string(), &["\"stiffness\""], "k_envelope_hartree_per_bohr2").expect("stiffness.json carries the envelope").value;
        eprintln!("stiffness envelope READ from {}: {v:.4e} Ha/bohr^2", stiff_path.display());
        v
    } else {
        let (units, _) = unit_members(&sim.units_reading());
        let s = measure_stiffness(&mut sim, &units, &body);
        eprintln!("stiffness envelope MEASURED on the branch state: {:.4e} Ha/bohr^2 ({} passes, {:.1} s)", s.k_envelope, s.passes, s.seconds);
        w.write("stiffness.json", &Record::new("stiffness").int("settle_frames", settle_frames as i64).raw("stiffness", s.json()).number("fine_dt_au", sim.dt())).expect("writes");
        sim.restore(&branch).expect("the branch restores");
        s.k_envelope
    };

    let stride = (frames / readouts).max(1);
    let frames = stride * (frames / stride);
    let series_path = out.join("flexible.series");
    let flex = match (reuse, read_flexible_series(&series_path, frames, stride)) {
        (true, Some(f)) => {
            eprintln!("the flexible arm REUSED from {} ({} readouts)", series_path.display(), f.series.len());
            f
        }
        _ => {
            eprintln!("the flexible arm: {frames} frames at the tables' step, a readout every {stride}");
            sim.restore(&branch).expect("the branch restores");
            sim.thermostat_on = false;
            let f = run_flexible(&mut sim, &z, l, frames, stride, &units_at_branch, &body);
            write_flexible_series(&series_path, &f, frames, stride);
            f
        }
    };
    eprintln!("the rigid arm{}", if refine { " with the refinement demonstration" } else { "" });
    sim.restore(&branch).expect("the branch restores");
    sim.thermostat_on = false;
    let match_k = if match_3n { Some(t_branch) } else { None };
    let rigid = run_rigid(&mut sim, &z, l, &body, k_envelope, frames, stride, false, match_k);
    let demo = if refine {
        eprintln!("the rigid arm WITH the refinement demonstration, from the same branch point");
        sim.restore(&branch).expect("the branch restores");
        sim.thermostat_on = false;
        Some(run_rigid(&mut sim, &z, l, &body, k_envelope, frames, stride, true, match_k))
    } else {
        None
    };

    // the readings, second half of each series, with the differences
    let sum = |s: &[Obs]| {
        (
            half_summary(s, |o| o.temperature_k),
            half_summary(s, |o| o.cross_unit_per_water),
            half_summary(s, |o| o.bonds_per_water),
            half_summary(s, |o| o.peak_bohr),
        )
    };
    let (ft, fu, fb, fp) = sum(&flex.series);
    let (rt, ru, rb, rp) = sum(&rigid.result.series);
    let ftr = half_summary(&flex.series, |o| o.rigid_mode_temperature_k);
    let rtr = half_summary(&rigid.result.series, |o| o.rigid_mode_temperature_k);
    let fvib = half_summary(&flex.series, |o| o.vibrational_kinetic_per_water_kt);
    let kt = K_B * TEMPERATURE_K;
    let (_, fe_peak, _) = energy_excursion(&flex.series);
    let (_, re_peak, _) = energy_excursion(&rigid.result.series);
    let bar = 0.1 * kt;
    let speedup = flex.core_seconds_per_ps() / rigid.result.core_seconds_per_ps();

    let g_clock = Gate::new("CLOCK")
        .work(3)
        .detail(format!("the rigid clock from the MEASURED envelope {k_envelope:.4e} Ha/bohr^2: dt {:.4} au against the fine {:.4} au, omega*dt {:.4} under the hold's 2pi/64 = {:.4}", rigid.clock_dt_au, flex.dt_au, rigid.clock_omega_dt, core::f64::consts::TAU / 64.0))
        .leg_at("the rigid step is above the fine step", rigid.clock_dt_au > flex.dt_au, rigid.clock_dt_au / flex.dt_au)
        .leg_at("the rigid step meets the fine clock's own accuracy target", rigid.clock_omega_dt <= core::f64::consts::TAU / 64.0 * (1.0 + 1e-12), rigid.clock_omega_dt)
        .leg("the envelope was measured, not typed", k_envelope.is_finite() && k_envelope > 0.0);
    report.gate(g_clock);
    let g_cost = Gate::new("COST")
        .work(2)
        .detail(format!("flexible {} passes in {:.1} s = {:.0} core-s/ps; rigid {} passes in {:.1} s = {:.0} core-s/ps (every overhead inside); speedup {speedup:.2}x", flex.passes, flex.seconds, flex.core_seconds_per_ps(), rigid.result.passes, rigid.result.seconds, rigid.result.core_seconds_per_ps()))
        .leg_at("fewer force passes per picosecond", rigid.result.passes < flex.passes, rigid.result.passes as f64 / flex.passes as f64)
        .leg_at("lower total cost per picosecond, every overhead counted - the KILL: a speedup under 1", speedup > 1.0, speedup);
    report.gate(g_cost);
    let g_nve = Gate::new("NVE")
        .work(2)
        .detail(format!("each arm's peak energy excursion per water over the run against a tenth of kT ({:.3e} Ha): flexible {:.3e}, rigid {:.3e}", 0.1 * kt, fe_peak / N_WATERS as f64, re_peak / N_WATERS as f64))
        .leg_at("the flexible arm holds its energy to a tenth of kT per water", fe_peak / N_WATERS as f64 <= 0.1 * kt, fe_peak / N_WATERS as f64 / kt)
        .leg_at("the rigid arm holds its energy to a tenth of kT per water", re_peak / N_WATERS as f64 <= 0.1 * kt, re_peak / N_WATERS as f64 / kt);
    report.gate(g_nve);
    if let Some(d) = &demo {
        let (_, raw_peak, _) = energy_excursion(&d.result.series);
        let (_, acc_peak, _) = energy_excursion_accounted(&d.result.series);
        let g_refine = Gate::new("REFINE")
            .work(3)
            .detail(format!(
                "the demonstration: {} events, {} fine frames while refined, {:.1} of {:.1} fs refined ({:.1} %); {} passes in {:.1} s = {:.0} core-s/ps against the flexible {:.0}; energy excursion per water raw {:.3e}, ACCOUNTED (injected subtracted, discards added back) {:.3e} against the bar {:.3e}",
                d.refine_events.len(), d.fine_frames_while_refined, d.physical_fs_refined, d.result.physical_fs, 100.0 * d.physical_fs_refined / d.result.physical_fs,
                d.result.passes, d.result.seconds, d.result.core_seconds_per_ps(), flex.core_seconds_per_ps(), raw_peak / N_WATERS as f64, acc_peak / N_WATERS as f64, bar
            ))
            .leg_at("the rule tripped and the box refined, then re-coarsened at least once", d.refine_events.iter().any(|e| e.what.starts_with("coarsen")), d.refine_events.len() as f64)
            .leg_at("fewer force passes than the flexible arm while carrying the refined region", d.result.passes < flex.passes, d.result.passes as f64 / flex.passes as f64)
            .leg_at("the ACCOUNTED energy holds to a tenth of kT per water", acc_peak / N_WATERS as f64 <= bar, acc_peak / N_WATERS as f64 / kt);
        report.gate(g_refine);
    }

    println!("REPLACEMENT ERROR (second half, rigid - flexible; each with its own SEM):");
    println!("  temperature      {:8.2} K   - {:8.2} K   = {:+.2} K  (sem {:.2} / {:.2}; the arms count 6 and 9 dof per unit - NOT the comparator)", rt.0, ft.0, rt.0 - ft.0, rt.1, ft.1);
    println!("  RIGID-MODE T     {:8.2} K   - {:8.2} K   = {:+.2} K  (sem {:.2} / {:.2}; the flexible arm's units projected at every readout - the comparator; its vibrations hold {:.2} kT/water of kinetic energy against 1.5 at equipartition)", rtr.0, ftr.0, rtr.0 - ftr.0, rtr.1, ftr.1, fvib.0);
    println!("  cross-unit U     {:.6e} - {:.6e} = {:+.3e} Ha/water = {:+.3} kT (sem {:.1e} / {:.1e})", ru.0, fu.0, ru.0 - fu.0, (ru.0 - fu.0) / kt, ru.1, fu.1);
    println!("  bonds per water  {:.4} - {:.4} = {:+.4} (sem {:.4} / {:.4})", rb.0, fb.0, rb.0 - fb.0, rb.1, fb.1);
    println!("  O-O first peak   {:.3} - {:.3} = {:+.3} bohr (sem {:.3} / {:.3})", rp.0, fp.0, rp.0 - fp.0, rp.1, fp.1);

    let validity = Validity::invariants_only("rigid-water", 1);
    let events = demo
        .as_ref()
        .map(|d| d.refine_events.as_slice())
        .unwrap_or(&[])
        .iter()
        .map(|e| format!("{{\"t_fs\": {}, \"what\": {:?}, \"units\": {}, \"discarded_deformation_rms_bohr\": {}, \"discarded_internal_kinetic_hartree\": {}}}", num(e.t_fs), e.what, e.units, num(e.discarded_deformation_rms), num(e.discarded_internal_kinetic)))
        .collect::<Vec<_>>()
        .join(", ");
    let triple = |name: &str, f: (f64, f64, f64), r: (f64, f64, f64)| {
        format!("\"{name}\": {{\"flexible\": {}, \"flexible_sem\": {}, \"flexible_g\": {}, \"rigid\": {}, \"rigid_sem\": {}, \"rigid_g\": {}, \"rigid_minus_flexible\": {}}}", num(f.0), num(f.1), num(f.2), num(r.0), num(r.1), num(r.2), num(r.0 - f.0))
    };
    let rec = Record::new("run")
        .flag("is_a_reading", true)
        .text("law_source", &law.law_source)
        .text("table_source", &law.table_source)
        .text("beta_source", &law.beta_source)
        .text("seed", &format!("{SEED:#x}"))
        .int("waters", N_WATERS as i64)
        .number("cell_edge_bohr", l)
        .number("density_g_cm3", DENSITY_G_CM3)
        .number("settle_temperature_k", TEMPERATURE_K)
        .int("settle_frames", settle_frames as i64)
        .int("frames", frames as i64)
        .int("readout_stride_frames", stride as i64)
        .number("physical_fs", flex.physical_fs)
        .number("stiffness_envelope_hartree_per_bohr2", k_envelope)
        .number("rigid_clock_dt_au", rigid.clock_dt_au)
        .number("rigid_clock_omega_dt", rigid.clock_omega_dt)
        .number("rigid_clock_omega_translation", rigid.clock_omega_translation)
        .number("rigid_clock_omega_rotation", rigid.clock_omega_rotation)
        .number("discarded_at_branch_deformation_rms_bohr", rigid.discarded_at_branch.0)
        .number("discarded_at_branch_internal_kinetic_hartree", rigid.discarded_at_branch.1)
        .number("discarded_at_branch_internal_kinetic_per_water_kt", rigid.discarded_at_branch.1 / N_WATERS as f64 / kt)
        .number("branch_temperature_k_3n", t_branch)
        .raw("reference_geometry", format!("{{\"rule\": \"the liquid's own mean monomer at the branch point, measured through the box's minimum image over every water unit; the lift's body is built from it, not from the gas-phase pin\", \"r_oh_bohr\": {}, \"r_oh_sd_bohr\": {}, \"theta_rad\": {}, \"theta_sd_rad\": {}, \"units\": {}, \"pin_r_oh_bohr\": {}, \"pin_theta_rad\": {}}}", num(geom.r_oh_bohr), num(geom.r_oh_sd_bohr), num(geom.theta_rad), num(geom.theta_sd_rad), geom.units, num(WATER_PIN_R_BOHR), num(WATER_PIN_THETA_RAD)))
        .number("rigid_temperature_as_projected_k", rigid.t_projected_k)
        .number("rigid_temperature_matched_k", rigid.t_matched_k)
        .number("kinetic_removed_by_matching_hartree", rigid.kinetic_removed_by_matching)
        .flag("matched_to_3n", match_3n)
        .text("temperature_matching_rule", "DEFAULT, no rescaling: the rigid modes are read as projected and KEPT, because the fine box's intermolecular bath is what a rigid replacement inherits and its 3N temperature averages that bath with vibrations that have not equilibrated (runs 2 and 3: rescaling to the 3N reading relaxed back within one readout with energy conserved). --match-3n rescales to the 3N reading and records the kinetic energy removed, which is NOT part of the arm's energy ledger")
        .flag("branch_reused", reuse)
        .raw("flexible", flex.json("flexible"))
        .raw("rigid", rigid.result.json("rigid"))
        .number("speedup_core_seconds_per_ps", speedup)
        .raw("rigid_refine", demo.as_ref().map(|d| d.result.json("rigid_refine")).unwrap_or_else(|| "null".to_string()))
        .number("speedup_refine_core_seconds_per_ps", demo.as_ref().map(|d| flex.core_seconds_per_ps() / d.result.core_seconds_per_ps()).unwrap_or(f64::NAN))
        .raw("replacement_error_second_half", format!("{{{}, {}, {}, {}, {}, \"flexible_vibrational_kinetic_per_water_kt\": {}, \"comparator\": \"rigid_mode_temperature_k: the flexible arm's units projected at every readout and their six retained modes read, against the rigid arm's own; temperature_k is the 3N reading on the flexible arm and the 6-dof reading on the rigid arm and compares different things\"}}", triple("temperature_k", ft, rt), triple("rigid_mode_temperature_k", ftr, rtr), triple("cross_unit_per_water_hartree", fu, ru), triple("bonds_per_water", fb, rb), triple("oo_first_peak_bohr", fp, rp), num(fvib.0)))
        .flag("refine", refine)
        .raw("refine_events", format!("[{events}]"))
        .int("fine_frames_while_refined", demo.as_ref().map(|d| d.fine_frames_while_refined as i64).unwrap_or(0))
        .number("physical_fs_refined", demo.as_ref().map(|d| d.physical_fs_refined).unwrap_or(0.0))
        .int("hot_unit", demo.as_ref().and_then(|d| d.hot_unit).map(|u| u as i64).unwrap_or(-1))
        .text("held_reconstruction_potential_change", "UNMEASURED: a held unit's per-frame reconstruction moves its sites by ~1e-6 bohr and its potential by an amount no pass was spent to read; the kinetic part is ledgered")
        .raw("validity", validity.json())
        .raw("gates", report.json())
        .flag("admits", report.admits());
    let name = "run.json";
    w.write(name, &rec).expect("the run record writes");
    for line in report.refusals() {
        println!("REFUSED {line}");
    }
    if report.admits() {
        w.done(&name.replace(".json", ".done"), &format!("{frames} frames both arms from one branch point; speedup {speedup:.2}x")).expect("done");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let phase = args.first().cloned().unwrap_or_else(|| "run".to_string());
    let val = |k: &str| -> Option<String> { args.iter().position(|a| a == k).and_then(|i| args.get(i + 1).cloned()) };
    let flags: Vec<usize> = args.iter().enumerate().filter(|(_, a)| a.starts_with("--")).map(|(i, _)| i + 1).collect();
    let out = PathBuf::from(
        args.iter()
            .enumerate()
            .skip(1)
            .find(|(i, a)| !a.starts_with("--") && !flags.contains(i))
            .map(|(_, a)| a.clone())
            .unwrap_or_else(|| "../conformance/water_observatory/replace0".to_string()),
    );
    let obs = observatory(&out);
    eprintln!("phase {phase}, out {}, observatory {}", out.display(), obs.display());
    let settle_frames = val("--settle").and_then(|v| v.parse().ok()).unwrap_or(2_000);
    match phase.as_str() {
        "stiffness" => stiffness_phase(&obs, &out, settle_frames),
        "run" => run_phase(
            &obs,
            &out,
            val("--frames").and_then(|v| v.parse().ok()).unwrap_or(4_000),
            settle_frames,
            val("--readouts").and_then(|v| v.parse().ok()).unwrap_or(40),
            args.iter().any(|a| a == "--refine"),
            args.iter().any(|a| a == "--reuse"),
            args.iter().any(|a| a == "--match-3n"),
        ),
        other => panic!("unknown phase {other:?}: stiffness | run"),
    }
    let _ = (add, Discarded::default());
}
