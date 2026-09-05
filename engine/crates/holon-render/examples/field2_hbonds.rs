//! FIELD-2 (`conformance/water_observatory/FIELD2_PREREG.md`): does the fixed-charge field HOLD
//! a hydrogen bond from a staked bonded start?
//!
//! ```text
//! cargo run --release -p holon-render --example field2_hbonds -- [OUT_DIR]
//! ```
//!
//! M1 and M2 are written first (`expectation.json`), then the arms: dimer and cyclic
//! tetramer, field OFF and ON, 293 K and 150 K, plus plants (i) and (ii). One frame is one
//! integrator step. Counted by the rung-1 lens (`holon_lens::lens::hbonds`).
//!
//! Diagnostics (`FIELD2_RESULTS.md`): `-- probe` prints the dimer's first 400 frames with
//! the field OFF and ON; `-- refs` prints the reference energies (monomer, dimer at 40
//! bohr, dimer at the staked start), the charge assignment and the triple list at the start.
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::embed::{fragment_charges, monomer, water_centers, ChargeModel, Fragment};
use holon_render::field::{FieldPlant, WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD};
use holon_render::sim::Sim;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

// The scenes, seeds and the lens count live in ONE place since FIELD-3 (verbatim move).
#[path = "../tests/common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::*;

/// M1: the field's binding at the start — E_field(start) − E_field(separated), the molecules
/// moved 40 bohr apart along x with their charge assignment kept.
fn binding_at_start(species: &[holon_chem::elements::Species], pos: &[[f64; 3]], box_edge: f64) -> (f64, f64, usize) {
    let mut s = scene(species, pos, box_edge, 293.0);
    s.set_field(true, None).unwrap();
    // the bond verdicts the assignment reads are written by `refresh_pairs` at grain
    // boundaries and at rebase. This call was added after the arms had run once, on the
    // suspicion that the first expectation read the verdicts before they existed; it moved
    // no number (`expectation.json` bit-identical). M1 is EXACTLY ZERO because the pair
    // verdict bonds the donor hydrogen to the acceptor oxygen at 3.56 bohr and the unit
    // assignment (FIELD_AMENDMENT_1) then finds no water unit at the bonded start — the
    // results document reads this as M-EMPTY-SECTOR.
    s.refresh_pairs();
    s.compute_forces();
    let charge = s.charge.clone();
    let e_start = s.field_energy_of(&charge);
    let hb0 = hbonds_now(&s);
    for i in 0..s.n {
        s.atoms[i].x += 40.0 * (i / 3) as f64;
    }
    let e_sep = s.field_energy_of(&charge);
    (e_start, e_sep, hb0)
}

struct Arm {
    f: f64,
    nbar: f64,
    t_mean: f64,
    e_field_final: f64,
    work_field: f64,
    transitions: u64,
    drift_peak: f64,
    columns_ok: bool,
    momentum_ok: bool,
    hb0: usize,
    seconds: f64,
}

fn run_arm(species: &[holon_chem::elements::Species], pos: &[[f64; 3]], box_edge: f64, temp: f64, on: bool, plant: FieldPlant) -> Arm {
    let t0 = Instant::now();
    let mut s = scene(species, pos, box_edge, temp);
    let hb0 = hbonds_now(&s);
    s.field_plant = plant;
    if on {
        s.set_field(true, None).expect("open box admits the field");
    }
    for _ in 0..SETTLE {
        s.step_frame(1);
    }
    let (mut with, mut total, mut t_sum) = (0usize, 0usize, 0.0f64);
    for _ in 0..COUNT {
        s.step_frame(1);
        let k = hbonds_now(&s);
        if k > 0 {
            with += 1;
        }
        total += k;
        t_sum += s.temperature();
    }
    Arm {
        f: with as f64 / COUNT as f64,
        nbar: total as f64 / COUNT as f64,
        t_mean: t_sum / COUNT as f64,
        e_field_final: s.e_field,
        work_field: s.work.field,
        transitions: s.field_work.transitions,
        drift_peak: s.drift_peak,
        columns_ok: s.work_columns_ok(),
        momentum_ok: s.momentum_residual() <= s.momentum_bound(),
        hb0,
        seconds: t0.elapsed().as_secs_f64(),
    }
}

fn arm_json(a: &Arm) -> String {
    format!("{{\"f\": {:.6}, \"nbar\": {:.6}, \"t_mean\": {:.2}, \"e_field_final\": {:.6e}, \"work_field\": {:.6e}, \"transitions\": {}, \"drift_peak\": {:.3e}, \"columns_ok\": {}, \"momentum_ok\": {}, \"hbonds_at_start\": {}, \"seconds\": {:.1}}}",
        a.f, a.nbar, a.t_mean, a.e_field_final, a.work_field, a.transitions, a.drift_peak, a.columns_ok, a.momentum_ok, a.hb0, a.seconds)
}

fn probe() {
    let (sp, pos) = dimer_positions();
    for on in [false, true] {
        let mut s = scene(&sp, &pos, 30.0, 293.0);
        if on { s.set_field(true, None).unwrap(); }
        eprintln!("-- dimer field {}: dt {:.3}, n {}", if on { "ON" } else { "OFF" }, s.dt(), s.n);
        for k in 0..=400 {
            if k % 25 == 0 {
                let oo = ((s.atoms[0].x - s.atoms[3].x).powi(2) + (s.atoms[0].y - s.atoms[3].y).powi(2) + (s.atoms[0].z - s.atoms[3].z).powi(2)).sqrt();
                let oh = ((s.atoms[3].x - s.atoms[1].x).powi(2) + (s.atoms[3].y - s.atoms[1].y).powi(2) + (s.atoms[3].z - s.atoms[1].z).powi(2)).sqrt();
                let bonded = s.pairs.iter().filter(|p| p.bonded).map(|p| format!("{}-{}", p.i, p.j)).collect::<Vec<_>>().join(",");
                eprintln!("  frame {k:4}: O-O {oo:6.2}  Oacc···Hdon {oh:6.2}  T {:6.0} K  e_pair {:+.4}  e_three {:+.4}  e_field {:+.2e}  hb {}  bonded [{bonded}]",
                    s.temperature(), s.e_pair, s.e_three, s.e_field, hbonds_now(&s));
            }
            s.step_frame(1);
        }
    }
}


/// Reference energies: the monomer at the pin, the dimer separated to 40 bohr, the dimer at
/// its bonded start — and, at the start, the charge assignment and the cross-molecule
/// (O,H,H) triples the water surface is asked to evaluate.
fn refs() {
    let (sp, pos) = dimer_positions();
    let mono_sp: Vec<_> = sp[..3].to_vec();
    let mono_pos: Vec<_> = pos[..3].to_vec();
    let mut m = scene(&mono_sp, &mono_pos, 30.0, 293.0);
    m.refresh_pairs();
    m.compute_forces();
    eprintln!("monomer at pin: e_pair {:+.6}  e_three {:+.6}  total {:+.6}  triples {}", m.e_pair, m.e_three, m.e_pair + m.e_three, m.triples().len());
    let mut far = pos.clone();
    for i in 3..6 { far[i][0] += 40.0; }
    let mut d40 = scene(&sp, &far, 100.0, 293.0);
    d40.refresh_pairs();
    d40.compute_forces();
    eprintln!("dimer at 40 bohr: e_pair {:+.6}  e_three {:+.6}  total {:+.6}  triples {}", d40.e_pair, d40.e_three, d40.e_pair + d40.e_three, d40.triples().len());
    let mut d = scene(&sp, &pos, 30.0, 293.0);
    d.set_field(true, None).unwrap();
    d.refresh_pairs();
    d.compute_forces();
    eprintln!("dimer at start: e_pair {:+.6}  e_three {:+.6}  total {:+.6}  e_field {:+.3e}  triples {} (fenced, untabulated class: {})", d.e_pair, d.e_three, d.e_pair + d.e_three, d.e_field, d.triples().len(), d.fence_untabulated);
    eprintln!("  interaction vs 40 bohr: pair {:+.6}  three {:+.6}  total {:+.6}", d.e_pair - d40.e_pair, d.e_three - d40.e_three, (d.e_pair + d.e_three) - (d40.e_pair + d40.e_three));
    eprintln!("  charges: {:?}", &d.charge[..d.n]);
    for p in d.pairs.iter() {
        if p.bonded {
            eprintln!("  bonded {}-{}: r {:.3}  e_rel {:+.5}  r_outer {:.2}", p.i, p.j, p.r, p.e_rel, p.r_outer);
        }
    }
    for t in d.triples() {
        let cross = (t[0] / 3 != t[1] / 3) || (t[1] / 3 != t[2] / 3);
        let z: Vec<u32> = t.iter().map(|&i| d.atoms[i].species.z).collect();
        let r = |a: usize, b: usize| ((d.atoms[a].x - d.atoms[b].x).powi(2) + (d.atoms[a].y - d.atoms[b].y).powi(2) + (d.atoms[a].z - d.atoms[b].z).powi(2)).sqrt();
        eprintln!("  triple {:?} z {:?} {}: r01 {:.2} r02 {:.2} r12 {:.2}", t, z, if cross { "CROSS" } else { "intra" }, r(t[0], t[1]), r(t[0], t[2]), r(t[1], t[2]));
    }
    // the same three readings with the acceptor's own hydrogens dropped: a lone O at 5.5
    let (sp3, pos3) = (sp[..4].to_vec(), pos[..4].to_vec());
    let mut lone = scene(&sp3, &pos3, 30.0, 293.0);
    lone.refresh_pairs();
    lone.compute_forces();
    eprintln!("monomer + lone O at 5.5: e_pair {:+.6}  e_three {:+.6}  (minus monomer: pair {:+.6} three {:+.6})", lone.e_pair, lone.e_three, lone.e_pair - m.e_pair, lone.e_three - m.e_three);
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("probe") {
        probe();
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("refs") {
        refs();
        return;
    }
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "../conformance/water_observatory/field2".to_string()));
    fs::create_dir_all(&out).expect("out");
    let (dsp, dpos) = dimer_positions();
    let (tsp, tpos) = ring_positions();
    let (qsp, qpos) = square_positions();

    // ---- M1, M2: the expectation, written first
    let (ed_start, ed_sep, dhb) = binding_at_start(&dsp, &dpos, 30.0);
    let (et_start, et_sep, thb) = binding_at_start(&tsp, &tpos, 34.0);
    let (eq_start, eq_sep, qhb) = binding_at_start(&qsp, &qpos, 34.0);
    let kt293 = K_B * 293.0;
    let kt150 = K_B * 150.0;
    let (o, h) = (OXYGEN, HYDROGEN);
    let q_at = |r: f64, t: f64| {
        let f = Fragment::new(vec![o, h, h], water_centers(r, t).to_vec(), vec![-2.0, 1.0, 1.0]);
        let m = monomer(&f, &[], ChargeModel::DipoleExact);
        fragment_charges(ChargeModel::DipoleExact, &f, &m.p, &m.solve.basis, &m.mom)[1]
    };
    let q_pin = q_at(WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD);
    let mut grid = Vec::new();
    let mut worst = 0.0f64;
    for dr in [-0.15, 0.0, 0.15] {
        for dt in [-15.0f64, 0.0, 15.0] {
            let q = q_at(WATER_PIN_R_BOHR + dr, WATER_PIN_THETA_RAD + dt.to_radians());
            worst = worst.max(((q - q_pin) / q_pin).abs());
            grid.push(format!("[{:.3}, {:.1}, {:.9}]", WATER_PIN_R_BOHR + dr, WATER_PIN_THETA_RAD.to_degrees() + dt, q));
        }
    }
    let expect = |bind: f64, kt: f64| if bind <= -2.0 * kt { "hold" } else if bind > -kt { "break" } else { "no expectation" };
    fs::write(out.join("expectation.json"), format!(
        "{{\n  \"kT_293\": {kt293:.6e}, \"kT_150\": {kt150:.6e},\n  \"dimer\":    {{\"hbonds_at_start\": {dhb}, \"e_field_start\": {ed_start:.6e}, \"e_field_separated\": {ed_sep:.6e}, \"binding\": {:.6e}, \"expect_293\": \"{}\", \"expect_150\": \"{}\"}},\n  \"tetramer\": {{\"hbonds_at_start\": {thb}, \"e_field_start\": {et_start:.6e}, \"e_field_separated\": {et_sep:.6e}, \"binding\": {:.6e}, \"expect_293\": \"{}\", \"expect_150\": \"{}\"}},\n  \"square_plant_ii\": {{\"hbonds_at_start\": {qhb}, \"e_field_start\": {eq_start:.6e}, \"e_field_separated\": {eq_sep:.6e}, \"binding\": {:.6e}, \"carrier_vs_ring\": {:.6e}}},\n  \"m2\": {{\"q_pin\": {q_pin:.9}, \"max_rel_dev\": {worst:.4}, \"grid_r_theta_q\": [{}]}}\n}}\n",
        ed_start - ed_sep, expect(ed_start - ed_sep, kt293), expect(ed_start - ed_sep, kt150),
        et_start - et_sep, expect(et_start - et_sep, kt293), expect(et_start - et_sep, kt150),
        eq_start - eq_sep, (eq_start - eq_sep) - (et_start - et_sep), grid.join(", "))).unwrap();
    eprintln!("M1 dimer: binding {:+.4e} Ha (kT293 {:.2e}) → {} / 150 K {}; tetramer: {:+.4e} → {} / {}; square: {:+.4e}; H-bonds at start {dhb} / {thb} / {qhb}",
        ed_start - ed_sep, kt293, expect(ed_start - ed_sep, kt293), expect(ed_start - ed_sep, kt150), et_start - et_sep, expect(et_start - et_sep, kt293), expect(et_start - et_sep, kt150), eq_start - eq_sep);
    eprintln!("M2: q_pin {q_pin:.6}, max relative deviation over the thermal grid {worst:.4}");

    // ---- the arms
    let mut lines = Vec::new();
    for (name, sp, pos, edge) in [("dimer", &dsp, &dpos, 30.0), ("tetramer", &tsp, &tpos, 34.0)] {
        for temp in [293.0, 150.0] {
            for on in [false, true] {
                let a = run_arm(sp, pos, edge, temp, on, FieldPlant::None);
                eprintln!("{name} {temp:.0} K {}: f {:.4}, n̄ {:.3}, T {:.0} K, e_field {:+.2e}, drift {:.1e}, cols {}, mom {}  ({:.0}s)", if on { "ON " } else { "OFF" }, a.f, a.nbar, a.t_mean, a.e_field_final, a.drift_peak, a.columns_ok, a.momentum_ok, a.seconds);
                lines.push(format!("  \"{name}_{temp:.0}_{}\": {}", if on { "on" } else { "off" }, arm_json(&a)));
            }
        }
        let p = run_arm(sp, pos, edge, 293.0, true, FieldPlant::FlipSign);
        eprintln!("{name} 293 K plant(i) sign: f {:.4}  ({:.0}s)", p.f, p.seconds);
        lines.push(format!("  \"{name}_293_plant_i\": {}", arm_json(&p)));
    }
    let q = run_arm(&qsp, &qpos, 34.0, 293.0, true, FieldPlant::None);
    eprintln!("square 293 K plant(ii) start: f {:.4}  ({:.0}s)", q.f, q.seconds);
    lines.push(format!("  \"square_293_plant_ii\": {}", arm_json(&q)));
    fs::write(out.join("arms.json"), format!("{{\n{}\n}}\n", lines.join(",\n"))).unwrap();
    println!("done");
}
