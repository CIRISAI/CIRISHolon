//! Is the integrator healthy, or is the bound wrong? Two decisive checks.
//!
//! (A) SECULARITY: a symplectic integrator's energy error oscillates and does not grow
//!     with time. If drift grows with t, no per-mode bound covers it and the integrator
//!     (or a non-smooth force) is at fault.
//! (B) dt-SCALING: the bound's whole form is `(omega dt)^2/4 * E`. If the measured drift
//!     does not fall as dt^2, the dt^2 form is wrong and widening the amplitude factor
//!     would be hiding a real defect rather than correcting a derivation.

use holon_render::sim::{Boundary, Sim};

fn loaded(n: usize) -> Sim {
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
    s
}

fn main() {
    println!("== (A) secularity: does drift grow with t? (N = 11, walls on, W_ext = 0) ==");
    let mut s = loaded(11);
    let mut marks = Vec::new();
    while s.time < 24_000.0 {
        s.step_frame(64);
        marks.push((s.time, s.drift_peak));
    }
    for target in [1_500.0, 3_000.0, 6_000.0, 12_000.0, 24_000.0] {
        if let Some((t, d)) = marks.iter().find(|(t, _)| *t >= target) {
            println!("  t = {t:>9.1}   drift_peak = {d:.6e}");
        }
    }

    sensitivity();
    println!("\n== (B) dt-scaling: drift should fall as dt^2 ==");
    // Freeze the amplitude and curvature inputs by using ONE scene and only moving dt,
    // via the declared rung-(ii) path so nothing else re-derives underneath.
    let mut prev: Option<f64> = None;
    for mult in [4.0_f64, 2.0, 1.0, 0.5, 0.25] {
        let mut s = loaded(11);
        s.timescale.allow_dt_growth = true;
        s.timescale.set_dt_multiplier(mult);
        let dt = s.dt();
        while s.time < 3_000.0 {
            s.step_frame(64);
        }
        let d = s.drift_peak;
        let ratio = prev.map(|p| p / d).unwrap_or(f64::NAN);
        println!(
            "  dt = {dt:>8.4} a.u. (x{mult:<5})  drift_peak = {d:.6e}   previous/this = {ratio:.3} (expect 4.0)"
        );
        prev = Some(d);
    }
}

/// (C) SENSITIVITY: the corrected bound is wider, so what leak does the gate still catch?
/// A planted, purely non-conservative velocity rescale that is NOT posted to `w_ext` —
/// exactly the class of bug the ledger exists to catch.
#[allow(dead_code)]
fn sensitivity() {
    println!("\n== (C) sensitivity: smallest planted leak the gate still catches ==");
    println!(
        "{:>12} {:>12} {:>12} {:>8} {:>6}",
        "leak/frame", "drift", "bound", "%bound", "gate"
    );
    for leak in [0.0_f64, 1e-9, 1e-8, 1e-7, 1e-6, 1e-5, 1e-4] {
        let mut s = loaded(11);
        let mut frames = 0;
        while s.time < 6_000.0 {
            s.step_frame(64);
            frames += 1;
            if leak > 0.0 {
                // Inject energy without telling the ledger. A real defect, not a knob.
                for i in 0..s.n {
                    let a = s.atoms[i];
                    s.set_velocity(i, a.vx * (1.0 + leak), a.vy * (1.0 + leak));
                }
            }
        }
        let pct = 100.0 * s.drift_peak / s.drift_bound();
        println!(
            "{leak:>12.0e} {:>12.4e} {:>12.4e} {pct:>7.1}% {:>6}  ({frames} frames)",
            s.drift_peak,
            s.drift_bound(),
            if s.energy_gate() { "PASS" } else { "FAIL" }
        );
    }
}
