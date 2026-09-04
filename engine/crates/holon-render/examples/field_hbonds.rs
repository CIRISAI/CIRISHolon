//! FIELD-1 S1 (`conformance/water_observatory/FIELD_PREREG.md` §1): do hydrogen bonds appear?
//!
//! ```text
//! cargo run --release -p holon-render --example field_hbonds -- [OUT_DIR]
//! ```
//!
//! Two arms of the same four-water scene from the same seed, walls, thermostat at 293 K:
//! field OFF and field ON. 2,000 frames of settling, then 20,000 frames counted — one frame
//! is one integrator step here (`step_frame(1)`), stated so the count means what it says.
//! The fraction of frames carrying at least one inter-molecular hydrogen bond by the rung-1
//! lens's criterion (`holon_lens::lens::hbonds`: r(O···O) < 6.614, r(O···H) < 4.630, donor
//! angle < 30°). Branch (a): ON ≥ 10 × OFF and ON ≥ 0.10.
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::embed::water_centers;
use holon_render::field::{WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD};
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[path = "../tests/common/quartet.rs"]
mod quartet;

const SETTLE: usize = 2000;
const COUNT: usize = 20000;

fn four_waters() -> Box<Sim> {
    let mono = water_centers(WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD);
    let oxygens = [[5.0, 5.0, 5.0], [12.0, 5.0, 5.0], [5.0, 12.0, 5.0], [12.0, 12.0, 5.0]];
    let mut species = Vec::new();
    let mut pos = Vec::new();
    for (k, o) in oxygens.iter().enumerate() {
        let flip = if k % 2 == 0 { 1.0 } else { -1.0 };
        for (m, c) in mono.iter().enumerate() {
            species.push(if m == 0 { OXYGEN } else { HYDROGEN });
            pos.push([o[0] + flip * c[0], o[1] + c[2], o[2] + c[1]]);
        }
    }
    let mut s = quartet::scene(&species, &pos, false);
    s.dims = Dims::Three;
    s.boundary = Boundary::Walls;
    s.width = 17.0;
    s.height = 17.0;
    s.depth = 10.0;
    let mut st: u64 = 0x4649_454c;
    let mut lcg = || {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((st >> 11) as f64) / ((1u64 << 53) as f64)
    };
    let n = s.n;
    let (mut px, mut py, mut pz) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let scale = if s.atoms[i].species.z == 8 { 2.5e-4 } else { 1.0e-3 };
        s.atoms[i].vx = scale * (2.0 * lcg() - 1.0);
        s.atoms[i].vy = scale * (2.0 * lcg() - 1.0);
        s.atoms[i].vz = scale * (2.0 * lcg() - 1.0);
        let m = s.atoms[i].mass();
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
    s.target_temperature = 293.0;
    s.rebase();
    s
}

fn arm(on: bool) -> (f64, f64, f64, u64, f64, f64) {
    let mut s = four_waters();
    if on {
        s.set_field(true, None).expect("walls admit the field");
    }
    for _ in 0..SETTLE {
        s.step_frame(1);
    }
    let z: Vec<u32> = (0..s.n).map(|i| s.atoms[i].species.z).collect();
    let (mut with, mut total_hb, mut t_sum) = (0usize, 0usize, 0.0f64);
    for _ in 0..COUNT {
        s.step_frame(1);
        let pos: Vec<[f64; 3]> = (0..s.n).map(|i| [s.atoms[i].x, s.atoms[i].y, s.atoms[i].z]).collect();
        let hb = holon_lens::lens::hbonds(&pos, &z).expect("oxygen and hydrogen present");
        if !hb.is_empty() {
            with += 1;
        }
        total_hb += hb.len();
        t_sum += s.temperature();
    }
    (with as f64 / COUNT as f64, total_hb as f64 / COUNT as f64, t_sum / COUNT as f64, s.field_work.transitions, s.e_field, s.work.field)
}

fn main() {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "../conformance/water_observatory/field".to_string()));
    fs::create_dir_all(&out).expect("out");
    let t0 = Instant::now();
    let off = arm(false);
    eprintln!("OFF: frames with ≥1 H-bond {:.4}, mean H-bonds/frame {:.4}, mean T {:.1} K  ({:.0}s)", off.0, off.1, off.2, t0.elapsed().as_secs_f64());
    let t1 = Instant::now();
    let on = arm(true);
    eprintln!("ON:  frames with ≥1 H-bond {:.4}, mean H-bonds/frame {:.4}, mean T {:.1} K, transitions {}, e_field {:.3e}, work.field {:.3e}  ({:.0}s)", on.0, on.1, on.2, on.3, on.4, on.5, t1.elapsed().as_secs_f64());
    let branch = if on.0 >= 10.0 * off.0 && on.0 >= 0.10 { "a" } else { "b" };
    fs::write(out.join("s1.json"), format!(
        "{{\n  \"settle_frames\": {SETTLE}, \"count_frames\": {COUNT}, \"frame\": \"one integrator step\",\n  \"off\": {{\"frac_frames_with_hbond\": {:.6}, \"mean_hbonds_per_frame\": {:.6}, \"mean_temperature_k\": {:.3}}},\n  \"on\":  {{\"frac_frames_with_hbond\": {:.6}, \"mean_hbonds_per_frame\": {:.6}, \"mean_temperature_k\": {:.3}, \"transitions\": {}, \"e_field_final\": {:.6e}, \"work_field_final\": {:.6e}}},\n  \"s1_branch\": \"{branch}\"\n}}\n",
        off.0, off.1, off.2, on.0, on.1, on.2, on.3, on.4, on.5)).unwrap();
    println!("S1 branch ({branch}): OFF {:.4} → ON {:.4}", off.0, on.0);
}
