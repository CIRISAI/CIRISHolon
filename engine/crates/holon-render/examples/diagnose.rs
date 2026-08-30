//! Scratch probe: measures the three quantities the gates disagreed about, so the
//! thresholds are set from measurement rather than from guesswork.
use holon_render::sim::{Boundary, Sim, M_H};
use holon_render::table::PotentialTable;

fn table() -> PotentialTable {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/viewer/h2_potential.json"
    ))
    .unwrap();
    let mut t = PotentialTable::empty();
    holon_render::json::load_into(&mut t, &src).unwrap();
    t
}

/// Build a Morse table directly at spacing `h` to see how the consistency residual
/// scales with the grid: truncation should fall like h^2, a genuine inconsistency would not.
fn morse_residual(h: f64) -> (f64, f64) {
    const D_E: f64 = 0.174490;
    const R_E: f64 = 1.40112;
    const A: f64 = 1.0288330;
    let mut t = PotentialTable::empty();
    let n = ((12.0 - 0.4) / h) as usize + 1;
    t.begin(n);
    for i in 0..n {
        let r = 0.4 + h * i as f64;
        let x = A * (r - R_E);
        let e = -1.0 + D_E * ((-2.0 * x).exp() - 2.0 * (-x).exp());
        let f = 2.0 * A * D_E * ((-2.0 * x).exp() - (-x).exp());
        t.knot(i, r, e, f);
    }
    t.finish(R_E, D_E, -1.0);
    (t.residual, t.residual_alt)
}

fn main() {
    let t = table();
    println!("== 1. consistency residual vs grid spacing (uniform Morse grids) ==");
    for h in [0.20, 0.10, 0.05, 0.025, 0.0125] {
        let (r, a) = morse_residual(h);
        println!("  h = {h:<7} residual = {r:.4e}  alt = {a:.4e}");
    }
    println!(
        "  shipped (non-uniform) grid: residual = {:.4e} alt = {:.4e}",
        t.residual, t.residual_alt
    );

    println!("\n== 2. interpolant reproduces its knots? ==");
    let mut worst_v: f64 = 0.0;
    let mut worst_d: f64 = 0.0;
    for i in 0..t.knots() {
        let (v, d, _) = t.eval(t.knot_r(i));
        worst_v = worst_v.max((v - t.knot_u(i)).abs());
        worst_d = worst_d.max((d - t.knot_d(i)).abs());
    }
    println!(
        "  max |E_interp - E_knot| = {worst_v:.3e}   max |dE_interp - dE_knot| = {worst_d:.3e}"
    );

    println!("\n== 3. force vs -dU/dR, UNCLAMPED relative error ==");
    let h = 1e-6;
    let mut rows: Vec<(f64, f64, f64, f64)> = Vec::new();
    let mut r = 0.25;
    while r < 15.0 {
        let numeric = -(t.u(r + h) - t.u(r - h)) / (2.0 * h);
        let analytic = t.force(r);
        let rel = if analytic.abs() > 0.0 {
            (numeric - analytic).abs() / analytic.abs()
        } else {
            0.0
        };
        rows.push((r, analytic, numeric - analytic, rel));
        r += 0.01;
    }
    rows.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
    for (r, f, absd, rel) in rows.iter().take(6) {
        println!("  R = {r:<7.2} F = {f:>12.4e}  absdiff = {absd:>11.3e}  rel = {rel:.3e}");
    }
    // Where is the force big enough to matter dynamically?
    let big: Vec<_> = rows.iter().filter(|x| x.1.abs() > 1e-7).collect();
    let worst_big = big.iter().map(|x| x.3).fold(0.0f64, f64::max);
    println!("  worst rel where |F| > 1e-7: {worst_big:.3e}");

    println!("\n== 4. bond script exploration ==");
    for v0 in [0.0015, 0.002, 0.003] {
        let src = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/viewer/h2_potential.json"
        ))
        .unwrap();
        let mut s = Sim::empty();
        holon_render::json::load_into(s.table_mut(), &src).unwrap();
        s.boundary = Boundary::Open;
        s.reset(2);
        let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
        s.set_position(0, cx - 4.0, cy);
        s.set_position(1, cx + 4.0, cy);
        s.set_velocity(0, v0, 0.0);
        s.set_velocity(1, -v0, 0.0);
        s.rebase();
        s.refresh_pairs();
        let e_rel_start = s.pairs[0].e_rel;
        // approach to closest
        let mut last = f64::INFINITY;
        let mut n = 0;
        loop {
            s.step();
            s.refresh_pairs();
            if s.pairs[0].r > last {
                break;
            }
            last = s.pairs[0].r;
            n += 1;
            if n > 200_000 {
                break;
            }
        }
        s.grab(0);
        // hold, watching E_rel
        let mut min_e = f64::INFINITY;
        let mut fired_at = None;
        for k in 0..30_000 {
            s.step();
            s.refresh_pairs();
            if s.pairs[0].e_rel < min_e {
                min_e = s.pairs[0].e_rel;
            }
            if s.pairs[0].e_rel < 0.0 && fired_at.is_none() {
                fired_at = Some(k);
            }
        }
        println!(
            "  v0 = {v0}: E_rel(start) = {e_rel_start:.4e}, closest R = {last:.4}, \
             min E_rel while held = {min_e:.4e}, first negative at step {fired_at:?}"
        );
    }
    let _ = M_H;
    alias_probe();
}

/// Does boundary-only gate sampling alias against the vibration?
#[allow(dead_code)]
fn alias_probe() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/viewer/h2_potential.json"
    ))
    .unwrap();
    println!("\n== 5. grain-boundary sampling vs the vibration period ==");
    for substeps in [16u32, 32, 48, 61, 63, 64, 65, 96, 128] {
        let mut s = Sim::empty();
        holon_render::json::load_into(s.table_mut(), &src).unwrap();
        s.adopt_table_timescale();
        s.boundary = Boundary::Open;
        s.reset(2);
        let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
        let delta = 0.02_f64;
        let r0 = s.table().r_e + delta;
        s.set_position(0, cx - 0.5 * r0, cy);
        s.set_position(1, cx + 0.5 * r0, cy);
        s.set_velocity(0, 0.0, 0.0);
        s.set_velocity(1, 0.0, 0.0);
        s.rebase();
        let k = s.table().curvature(s.table().r_e);
        let mu = 0.5 * M_H;
        let omega = (k / mu).sqrt();
        let e0 = 0.5 * k * delta * delta;
        let predicted = e0 * (omega * s.dt()).powi(2) / 4.0;
        // true peak, sampled every substep
        let mut true_peak = 0.0f64;
        let frames = 20_000 / substeps as usize;
        for _ in 0..frames {
            for _ in 0..substeps {
                s.step();
                true_peak = true_peak.max(s.drift());
            }
            s.close_grain();
        }
        println!(
            "  substeps/frame = {substeps:<4} dt = {:.4}  boundary-sampled = {:.3e}  true = {:.3e}  \
             boundary/true = {:.4}  true/predicted = {:.4}",
            s.dt(),
            s.drift_peak,
            true_peak,
            s.drift_peak / true_peak,
            true_peak / predicted
        );
    }
}
