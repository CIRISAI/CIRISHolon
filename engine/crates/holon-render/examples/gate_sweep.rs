//! Find a configuration that breaches the CURRENT bound, so the permanent test exercises
//! the real failure rather than a scene that happens to stay green.
use holon_render::sim::{Boundary, Sim};

fn loaded(n: usize, radius: f64) -> Sim {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/viewer/h2_potential.json"
    ))
    .unwrap();
    let mut s = Sim::empty();
    holon_render::json::load_into(&mut s.table, &src).unwrap();
    s.adopt_table_timescale();
    s.boundary = Boundary::Walls;
    s.reset(n);
    if radius > 0.0 {
        let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
        for i in 0..s.n {
            let th = (i as f64) * core::f64::consts::TAU / (s.n as f64);
            s.set_position(i, cx + radius * th.cos(), cy + radius * th.sin());
            s.set_velocity(i, 0.0, 0.0);
        }
        s.rebase();
    }
    s
}

fn main() {
    println!(
        "{:>3} {:>7} {:>11} {:>11} {:>8} {:>9} {:>9} {:>7} {:>6} {:>5}",
        "N", "radius", "drift", "bound", "%bound", "e_ref", "modes", "mratio", "r_min", "mols"
    );
    let mut worst = (0usize, 0.0f64, 0.0f64);
    for n in [3usize, 4, 5, 6, 8, 11, 14, 16] {
        for radius in [4.0_f64, 6.0, 8.0, 10.0] {
            let mut s = loaded(n, radius);
            let mut r_min = f64::INFINITY;
            while s.time < 6200.0 {
                s.step_frame(64);
                for i in 0..s.n {
                    for j in (i + 1)..s.n {
                        let dx = s.atoms[j].x - s.atoms[i].x;
                        let dy = s.atoms[j].y - s.atoms[i].y;
                        r_min = r_min.min((dx * dx + dy * dy).sqrt());
                    }
                }
            }
            let pct = 100.0 * s.drift_peak / s.drift_bound();
            let modes = s.e_kin + s.e_pair.abs() + s.e_wall + s.e_spring;
            if pct > worst.1 {
                worst = (n, pct, radius);
            }
            println!(
                "{n:>3} {radius:>7.1} {:>11.4e} {:>11.4e} {pct:>7.1}% {:>9.4} {:>9.4} {:>7.1} {r_min:>6.3} {:>5}",
                s.drift_peak,
                s.drift_bound(),
                s.e_ref,
                modes,
                modes / s.e_ref,
                s.holons.molecule_count()
            );
        }
    }
    println!(
        "\nworst: N = {}, radius = {:.1}, {:.1}% of bound",
        worst.0, worst.2, worst.1
    );
}
