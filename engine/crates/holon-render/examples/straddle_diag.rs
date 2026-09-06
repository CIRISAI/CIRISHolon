//! DIAGNOSTIC: one water molecule, tables only, 29.6-bohr periodic cell. The same molecule
//! with its second hydrogen at x = -1.15 (unwrapped) and at x = L - 1.15 (wrapped) must give
//! identical rows and forces under the minimum image; and one step from rest must not heat.
#[path = "../tests/common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::scene;
use holon_chem::elements::by_symbol;
use holon_chem::embed::water_centers;
use holon_render::channel::Row;
use holon_render::sim::Boundary;

fn report(label: &str, s: &mut holon_render::sim::Sim) {
    s.compute_forces();
    let f: Vec<(f64, f64, f64)> = (0..s.n).map(|i| s.internal_force(i)).collect();
    println!("{label:34}: e_pair {:+.9e} e_three {:+.9e} | F_O ({:+.4e},{:+.4e},{:+.4e}) F_H1 ({:+.4e},{:+.4e},{:+.4e}) F_H2 ({:+.4e},{:+.4e},{:+.4e}) | x: O {:.3} H1 {:.3} H2 {:.3}",
        s.row(Row::Pair), s.row(Row::Three), f[0].0, f[0].1, f[0].2, f[1].0, f[1].1, f[1].2, f[2].0, f[2].1, f[2].2, s.atoms[0].x, s.atoms[1].x, s.atoms[2].x);
}

fn main() {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let l = 29.59363131;
    let c = water_centers(1.9435738400, 1.6887434037);
    let shift = [0.3, 0.5 * l, 0.5 * l];
    let pos: Vec<[f64; 3]> = c.iter().map(|p| [p[0] + shift[0], p[1] + shift[1], p[2] + shift[2]]).collect();
    let mut s = scene(&[o, h, h], &pos, l, 293.0);
    for i in 0..s.n { s.atoms[i].vx = 0.0; s.atoms[i].vy = 0.0; s.atoms[i].vz = 0.0; }
    s.thermostat_on = false;
    s.boundary = Boundary::Periodic;
    report("unwrapped (H2 at x = -1.15)", &mut s);
    s.atoms[2].x += l;
    report("wrapped (H2 at x = L - 1.15)", &mut s);
    // one step from the wrapped state
    s.step();
    println!("after one step from wrapped: T {:.3} K, drift {:.3e}", s.temperature(), s.drift());
    report("after one step", &mut s);
    // and the same from the unwrapped state, without the wrap being possible (open box, big)
    let mut s2 = scene(&[o, h, h], &pos, l, 293.0);
    for i in 0..s2.n { s2.atoms[i].vx = 0.0; s2.atoms[i].vy = 0.0; s2.atoms[i].vz = 0.0; }
    s2.thermostat_on = false;
    s2.width = 4.0 * l; s2.height = 4.0 * l; s2.depth = 4.0 * l;
    s2.step();
    println!("after one step in the open box: T {:.3} K, drift {:.3e}", s2.temperature(), s2.drift());
}
