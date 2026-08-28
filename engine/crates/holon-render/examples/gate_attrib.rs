//! Which of the two changes fixed the breach? Attribute it, do not infer it.
use holon_render::sim::{Boundary, Sim, K_WALL, M_H};

fn scene(n: usize, radius: f64) -> Sim {
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
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    for i in 0..s.n {
        let th = (i as f64) * core::f64::consts::TAU / (s.n as f64);
        s.set_position(i, cx + radius * th.cos(), cy + radius * th.sin());
        s.set_velocity(i, 0.0, 0.0);
    }
    s.rebase();
    s
}

fn main() {
    let mu = 0.5 * M_H;
    for (n, r) in [(14usize, 10.0_f64), (11, 6.0), (5, 6.0)] {
        let mut s = scene(n, r);
        while s.time < 6200.0 {
            s.step_frame(64);
        }
        let k_env = s.timescale.k_env;
        let k_vis = s.k_pair_max();
        let e_ref = s.e_ref;
        let signed = s.energy().abs().max(s.table.d_e.abs());
        let dt = s.dt();

        // The four candidate bounds, same drift in every case.
        let b = |k: f64, e: f64| 4.0 * 0.25 * (k / mu).max(K_WALL / M_H) * dt * dt * e;
        let old = b(k_env, signed); // as shipped: envelope curvature, signed amplitude
        let curv_only = b(k_env.max(k_vis), signed); // curvature fix alone
        let amp_only = b(k_env, e_ref); // amplitude fix alone
        let both = b(k_env.max(k_vis), e_ref);

        println!("N = {n:>2} r = {r:>4.1}  drift = {:.4e}", s.drift_peak);
        println!(
            "   k_env = {k_env:.3}  k_visited = {k_vis:.3}  (max() is a {} here)",
            if k_env >= k_vis { "NO-OP" } else { "change" }
        );
        println!("   amplitude: signed |E| = {signed:.4}   mode-energy e_ref = {e_ref:.4}");
        println!(
            "   bound OLD        = {old:.4e}  -> {:>6.1}%  {}",
            100.0 * s.drift_peak / old,
            if s.drift_peak > old { "BREACH" } else { "ok" }
        );
        println!(
            "   bound curv only  = {curv_only:.4e}  -> {:>6.1}%  {}",
            100.0 * s.drift_peak / curv_only,
            if s.drift_peak > curv_only {
                "BREACH"
            } else {
                "ok"
            }
        );
        println!(
            "   bound amp only   = {amp_only:.4e}  -> {:>6.1}%  {}",
            100.0 * s.drift_peak / amp_only,
            if s.drift_peak > amp_only {
                "BREACH"
            } else {
                "ok"
            }
        );
        println!(
            "   bound BOTH       = {both:.4e}  -> {:>6.1}%  {}\n",
            100.0 * s.drift_peak / both,
            if s.drift_peak > both { "BREACH" } else { "ok" }
        );
    }
}
