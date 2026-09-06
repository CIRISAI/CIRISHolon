//! DIAGNOSTIC, not a campaign artifact: watch the closure reading on the 128-water box frame
//! by frame under the CT-2 law with the seam switch, and name the first hydrogen whose
//! ownership flips, with its own-oxygen and foreign-oxygen distances at that frame. Written
//! because the counted arm and the dry run (on a DIFFERENT law) both voided at settling
//! frame 82 — a coincidence that points at the start box, not at either law.
#[path = "../tests/common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::scene;
use holon_render::seam::{SeamModel, FREE};
use holon_render::sim::Boundary;
use holon_render::waterbox::liquid_box;
use std::fs;

fn json_num(t: &str, key: &str) -> f64 {
    t.split(&format!("\"{key}\": ")).nth(1).and_then(|x| x.split(|c: char| c == ',' || c == '}' || c == '\n').next()).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(0.0)
}

fn main() {
    let t = fs::read_to_string("../conformance/water_observatory/ct2/wall_ct2.json").expect("wall_ct2.json");
    let g = |k: &str| json_num(&t, k);
    let model = SeamModel {
        a: g("a"), b: g("b"), p: g("p"), c: g("c"), c6: g("c6"), a_oh: g("a_oh"), b_oh: g("b_oh"), a_hh: g("a_hh"), b_hh: g("b_hh"),
        p_hh: g("p_hh"), c_hh: g("c_hh"), p_ct: g("p_ct"), c_ct: g("c_ct"), m_ct: g("m_ct") as u8, k_ct: g("k_ct") as u8, lambda_ct: g("lambda_ct"), r_cut: 14.0,
    };
    let (species, pos, l) = liquid_box(4, 0.997, 0x4c49_5155_4944);
    let z: Vec<u32> = species.iter().map(|s| s.z).collect();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let has = |k: &str| args.iter().any(|a| a == k);
    let mut sim = scene(&species, &pos, l, 293.0);
    if !has("nofield") { sim.set_field(true, None).unwrap(); }
    let model = if has("unswitched") { SeamModel { r_cut: 0.0, ..model } } else { model };
    if has("nowall") { sim.set_seam(Some(SeamModel::NO_WALL)).unwrap(); } else if !has("noseam") { sim.set_seam(Some(model)).unwrap(); }
    if has("force_periodic") {
        // bypass the door deliberately (a diagnostic): the public field, as the door's doc allows
        sim.boundary = Boundary::Periodic;
    } else if !has("open") { sim.set_boundary(Boundary::Periodic).unwrap(); } else { sim.width = 4.0 * l; sim.height = 4.0 * l; sim.depth = 4.0 * l; }
    if has("nothermo") { sim.thermostat_on = false; }
    sim.compute_forces();
    sim.rebase();
    println!("config: field {} seam {} boundary {} thermostat {}; e_kin {:.4e} e_field {:.4e} e_seam {:.4e}", !has("nofield"), !has("noseam"), if has("open") { "open" } else { "periodic" }, sim.thermostat_on, sim.row(holon_render::channel::Row::Kin), sim.e_field, sim.e_seam);
    let n = sim.n;
    let mi = |sim: &holon_render::sim::Sim, i: usize, j: usize| -> f64 {
        let mut d2 = 0.0;
        for (a, b) in [(sim.atoms[i].x, sim.atoms[j].x), (sim.atoms[i].y, sim.atoms[j].y), (sim.atoms[i].z, sim.atoms[j].z)] {
            let mut d = a - b;
            d -= l * (d / l).round();
            d2 += d * d;
        }
        d2.sqrt()
    };
    let stats = |sim: &holon_render::sim::Sim, units: &[u32]| -> (f64, f64, f64, usize, usize, f64, usize) {
        // (max own O–H, shortest cross H···O, its H, its O, max H speed, the H)
        let (mut max_own, mut min_cross, mut hc, mut oc, mut vmax, mut hv) = (0.0f64, f64::INFINITY, 0, 0, 0.0f64, 0);
        for h in 0..n {
            if z[h] != 1 { continue; }
            let v = (sim.atoms[h].vx.powi(2) + sim.atoms[h].vy.powi(2) + sim.atoms[h].vz.powi(2)).sqrt();
            if v > vmax { vmax = v; hv = h; }
            let own = units[h];
            if own != FREE { max_own = max_own.max(mi(sim, h, own as usize)); }
            for o in 0..n {
                if z[o] != 8 || o as u32 == own { continue; }
                let d = mi(sim, h, o);
                if d < min_cross { min_cross = d; hc = h; oc = o; }
            }
        }
        (max_own, min_cross, vmax, hc, oc, sim.temperature(), hv)
    };
    let mut prev = sim.units_reading();
    let (mo, mc, vm, hc, oc, temp, hv) = stats(&sim, &prev);
    println!("frame 0: dt {:.4e} au, T {temp:.1} K, max own O–H {mo:.3} bohr, shortest cross H···O {mc:.3} bohr (H {hc}, O {oc}), max H speed {vm:.3e} (H {hv})", sim.dt());
    for frame in 1..=300 {
        sim.step();
        let u = sim.units_reading();
        let (mo, mc, vm, hc, oc, temp, hv) = stats(&sim, &u);
        if frame % 10 == 0 {
            println!("frame {frame}: T {temp:.1} K, drift {:.3e}, w_ext {:.3e} (thermo {:.3e}), max own O–H {mo:.3}, shortest cross H···O {mc:.3} (H {hc}, O {oc}), max H speed {vm:.3e} (H {hv}), free {}", sim.drift(), sim.w_ext, sim.work.thermostat, u[..n].iter().filter(|&&x| x == FREE).count());
        }
        if u != prev {
            println!("FLIP at frame {frame}:");
            for i in 0..n {
                if u[i] != prev[i] {
                    let (o_old, o_new) = (prev[i], u[i]);
                    let d_old = if o_old != FREE { mi(&sim, i, o_old as usize) } else { f64::NAN };
                    let d_new = if o_new != FREE { mi(&sim, i, o_new as usize) } else { f64::NAN };
                    let v = (sim.atoms[i].vx.powi(2) + sim.atoms[i].vy.powi(2) + sim.atoms[i].vz.powi(2)).sqrt();
                    println!("  atom {i} (Z {}): unit {o_old} → {o_new}; d(old O) {d_old:.3}, d(new O) {d_new:.3} bohr; speed {v:.3e}", z[i]);
                }
            }
            break;
        }
        prev = u;
    }
}
