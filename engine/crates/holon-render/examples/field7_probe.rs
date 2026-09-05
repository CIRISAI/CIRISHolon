//! FIELD-7 diagnostic: the dimer under `wall7.json` over the full arm length — the first
//! frame a cross-unit H···O contact enters the hole the contact term leaves below its data,
//! the ledger's drift at that moment, and the end state.
use holon_render::seam::SeamModel;
#[path = "../tests/common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::*;
fn num(t: &str, k: &str) -> f64 {
    t.split(&format!("\"{k}\": ")).nth(1).and_then(|x| x.split(',').next()).and_then(|x| x.trim().parse().ok()).unwrap_or(0.0)
}
fn main() {
    let t = std::fs::read_to_string("../conformance/water_observatory/field7/wall7.json").expect("wall7.json");
    let m = SeamModel { a: num(&t, "a"), b: num(&t, "b"), p: num(&t, "p"), c: num(&t, "c"), c6: num(&t, "c6"), a_oh: num(&t, "a_oh"), b_oh: num(&t, "b_oh"), a_hh: num(&t, "a_hh"), b_hh: num(&t, "b_hh") };
    for temp in [293.0, 150.0] {
        let (sp, pos) = dimer_positions();
        let mut s = scene(&sp, &pos, 30.0, temp);
        s.set_field(true, None).unwrap();
        s.set_seam(Some(m)).unwrap();
        let l0 = s.ledger();
        let (mut first_hole, mut min_ho, mut min_frame) = (None, f64::INFINITY, 0usize);
        for frame in 0..(SETTLE + COUNT) {
            s.step_frame(1);
            let mut mn = f64::INFINITY;
            for &(h, o) in &[(1usize, 3usize), (2, 3), (4, 0), (5, 0)] {
                let d = ((s.atoms[h].x - s.atoms[o].x).powi(2) + (s.atoms[h].y - s.atoms[o].y).powi(2) + (s.atoms[h].z - s.atoms[o].z).powi(2)).sqrt();
                mn = mn.min(d);
            }
            if mn < min_ho { min_ho = mn; min_frame = frame; }
            if first_hole.is_none() && mn < 2.5 {
                first_hole = Some(frame);
                eprintln!("{temp:.0} K: first cross-unit H···O under 2.5 bohr at frame {frame}: {mn:.3} bohr; T {:.0} K, drift {:+.3e}, e_seam {:+.4e}, e_field {:+.4e}, hb {}", s.temperature(), s.ledger() - l0, s.e_seam, s.e_field, hbonds_now(&s));
            }
        }
        eprintln!("{temp:.0} K end: min cross H···O {min_ho:.3} bohr at frame {min_frame}; final T {:.0} K, drift {:+.3e}, drift_peak {:.2e}, e_seam {:+.4e}, e_field {:+.4e}, units {}, hb {}, O–O {:.2}", s.temperature(), s.ledger() - l0, s.drift_peak, s.e_seam, s.e_field, s.seam_work.units, hbonds_now(&s), ((s.atoms[0].x - s.atoms[3].x).powi(2) + (s.atoms[0].y - s.atoms[3].y).powi(2) + (s.atoms[0].z - s.atoms[3].z).powi(2)).sqrt());
    }
}
