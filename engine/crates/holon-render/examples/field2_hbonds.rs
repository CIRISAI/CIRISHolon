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
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::embed::{fragment_charges, monomer, water_centers, ChargeModel, Fragment};
use holon_render::field::{FieldPlant, WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD};
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[path = "../tests/common/quartet.rs"]
mod quartet;

const SETTLE: usize = 2000;
const COUNT: usize = 20000;
const R_OO: f64 = 5.5;
const K_B: f64 = 3.166811563e-6; // hartree per kelvin

fn unit(v: [f64; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / n, v[1] / n, v[2] / n]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// A water at `o` donating its first hydrogen along `dir`, the second hydrogen in the plane
/// spanned by `dir` and `side` at the pin angle.
fn water_at(o: [f64; 3], dir: [f64; 3], side: [f64; 3]) -> [[f64; 3]; 3] {
    let d = unit(dir);
    let n = unit(cross(cross(d, side), d)); // side made perpendicular to d
    let (r, t) = (WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD);
    let h1 = [o[0] + r * d[0], o[1] + r * d[1], o[2] + r * d[2]];
    let (c, s) = (t.cos(), t.sin());
    let h2 = [o[0] + r * (c * d[0] + s * n[0]), o[1] + r * (c * d[1] + s * n[1]), o[2] + r * (c * d[2] + s * n[2])];
    [o, h1, h2]
}

fn dimer_positions() -> (Vec<holon_chem::elements::Species>, Vec<[f64; 3]>) {
    let c = 15.0;
    let donor = water_at([c, c, c], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    // the acceptor: its C2 axis on the O···O axis, hydrogens on the far side
    let mono = water_centers(WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD);
    let acc: Vec<[f64; 3]> = mono.iter().map(|m| [c + m[0], c + m[1], c + R_OO + m[2]]).collect();
    let mut species = Vec::new();
    let mut pos = Vec::new();
    for (k, p) in donor.iter().chain(acc.iter()).enumerate() {
        species.push(if k % 3 == 0 { OXYGEN } else { HYDROGEN });
        pos.push(*p);
    }
    (species, pos)
}

fn ring_positions() -> (Vec<holon_chem::elements::Species>, Vec<[f64; 3]>) {
    let c = 17.0;
    let h = 0.5 * R_OO;
    let corners = [[c - h, c - h, c], [c + h, c - h, c], [c + h, c + h, c], [c - h, c + h, c]];
    let mut species = Vec::new();
    let mut pos = Vec::new();
    for k in 0..4 {
        let o = corners[k];
        let next = corners[(k + 1) % 4];
        let dir = [next[0] - o[0], next[1] - o[1], next[2] - o[2]];
        let side = if k % 2 == 0 { [0.0, 0.0, 1.0] } else { [0.0, 0.0, -1.0] };
        for (m, p) in water_at(o, dir, side).iter().enumerate() {
            species.push(if m == 0 { OXYGEN } else { HYDROGEN });
            pos.push(*p);
        }
    }
    (species, pos)
}

/// FIELD-1's parallel-dipole square, for plant (ii).
fn square_positions() -> (Vec<holon_chem::elements::Species>, Vec<[f64; 3]>) {
    let mono = water_centers(WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD);
    let oxygens = [[13.0, 13.0, 17.0], [20.0, 13.0, 17.0], [13.0, 20.0, 17.0], [20.0, 20.0, 17.0]];
    let mut species = Vec::new();
    let mut pos = Vec::new();
    for (k, o) in oxygens.iter().enumerate() {
        let flip = if k % 2 == 0 { 1.0 } else { -1.0 };
        for (m, c) in mono.iter().enumerate() {
            species.push(if m == 0 { OXYGEN } else { HYDROGEN });
            pos.push([o[0] + flip * c[0], o[1] + c[2], o[2] + c[1]]);
        }
    }
    (species, pos)
}

fn scene(species: &[holon_chem::elements::Species], pos: &[[f64; 3]], box_edge: f64, temp: f64) -> Box<Sim> {
    let mut s = quartet::scene(species, pos, false);
    s.dims = Dims::Three;
    s.boundary = Boundary::Open;
    s.width = box_edge;
    s.height = box_edge;
    s.depth = box_edge;
    let mut st: u64 = 0x4649_454c_3200;
    let mut lcg = || {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((st >> 11) as f64) / ((1u64 << 53) as f64)
    };
    let n = s.n;
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
    s
}

fn hbonds_now(s: &Sim) -> usize {
    let z: Vec<u32> = (0..s.n).map(|i| s.atoms[i].species.z).collect();
    let pos: Vec<[f64; 3]> = (0..s.n).map(|i| [s.atoms[i].x, s.atoms[i].y, s.atoms[i].z]).collect();
    holon_lens::lens::hbonds(&pos, &z).map(|v| v.len()).unwrap_or(0)
}

/// M1: the field's binding at the start — E_field(start) − E_field(separated), the molecules
/// moved 40 bohr apart along x with their charge assignment kept.
fn binding_at_start(species: &[holon_chem::elements::Species], pos: &[[f64; 3]], box_edge: f64) -> (f64, f64, usize) {
    let mut s = scene(species, pos, box_edge, 293.0);
    s.set_field(true, None).unwrap();
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

fn main() {
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
