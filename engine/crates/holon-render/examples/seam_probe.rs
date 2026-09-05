//! FIELD-3 diagnostic: where FIELD-1's pair-verdict assignment and the closure reading
//! disagree on the receipt's four-water scene.
use holon_render::sim::Boundary;
#[path = "../tests/common/channel_scenes.rs"]
#[allow(dead_code)]
mod channel_scenes;
fn main() {
    let mut s = channel_scenes::four_waters(Boundary::Walls);
    s.set_field(true, None).unwrap();
    let mut first: Option<usize> = None;
    let mut differing = 0usize;
    for frame in 0..(channel_scenes::WATER_STEPS / channel_scenes::SUBSTEPS as usize) {
        s.step_frame(channel_scenes::SUBSTEPS);
        let old = s.units_by_pair_verdict();
        let new: Vec<u32> = s.unit_of[..s.n].to_vec();
        if old != new {
            differing += 1;
            if first.is_none() {
                first = Some(frame);
                eprintln!("first disagreement at frame {frame} (step {}):", (frame + 1) * channel_scenes::SUBSTEPS as usize);
                for i in 0..s.n {
                    if old[i] != new[i] {
                        eprintln!("  atom {i} z {}: verdict unit {} vs closure unit {}", s.atoms[i].species.z, old[i] as i64, new[i] as i64);
                    }
                }
                for p in s.pairs[..s.pair_count].iter() {
                    let (zi, zj) = (s.atoms[p.i].species.z, s.atoms[p.j].species.z);
                    if (zi == 8 && zj == 1) || (zi == 1 && zj == 8) {
                        if old[p.i] != new[p.i] || old[p.j] != new[p.j] {
                            eprintln!("    pair {}-{}: r {:.3} e_rel {:+.5} r_outer {:.3} bonded {}", p.i, p.j, p.r, p.e_rel, p.r_outer, p.bonded);
                        }
                    }
                }
            }
        }
    }
    eprintln!("frames differing: {differing} of {}; field transitions {}; units now {}", channel_scenes::WATER_STEPS / channel_scenes::SUBSTEPS as usize, s.field_work.transitions, s.seam_work.units);
}
