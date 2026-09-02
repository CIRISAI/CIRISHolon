use holon_lattice::{state::Model, Lattice};
fn main() {
    let m = Model::fhp6();
    let c = m.fhp_i(true);
    let mut g = Lattice::seeded(m, 256, 1, 0.35, c);
    let t = std::time::Instant::now();
    for _ in 0..200 { g.step(); }
    let e = t.elapsed().as_secs_f64();
    let cu = 200.0 * 256.0 * 256.0;
    println!("{:.3} s for {:.3e} cell-updates -> {:.2} ns/cu", e, cu, e / cu * 1e9);
    println!("20000 steps at L=256 would be {:.1} s", e / 200.0 * 20000.0);
}
