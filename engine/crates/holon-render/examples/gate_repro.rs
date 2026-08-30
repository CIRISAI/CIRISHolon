//! Repro + instrumentation for the live GATE 1 ENERGY failure.
//!
//! Field report: drift(peak) 5.627e-4 against bound 4.920e-4 (114.4%), N ~ 11, walls on,
//! W_ext = 0, t = 5848 a.u., four bonds formed by three-body collisions, momentum gate at
//! roundoff. The gate failed honestly; this asks which side is wrong.
//!
//! The instrument tracks, per SUBSTEP, the quantities the bound is built from — the true
//! maximum pair curvature and the true maximum pair relative energy — and compares them
//! with what the envelope believes at the grain boundaries where it is refreshed.

use holon_render::sim::{Boundary, Sim, K_WALL, M_H};

fn loaded() -> Sim {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/viewer/h2_potential.json"
    ))
    .unwrap();
    let mut s = Sim::empty();
    holon_render::json::load_into(s.table_mut(), &src).unwrap();
    s.adopt_table_timescale();
    s
}

/// True max |U''| and true max pair relative energy over the CURRENT state.
fn true_maxima(s: &Sim) -> (f64, f64, f64) {
    let mu = 0.5 * M_H;
    let mut k_max: f64 = 0.0;
    let mut e_rel_max = f64::NEG_INFINITY;
    let mut r_min = f64::INFINITY;
    for i in 0..s.n {
        for j in (i + 1)..s.n {
            let dx = s.atoms[j].x - s.atoms[i].x;
            let dy = s.atoms[j].y - s.atoms[i].y;
            let r = (dx * dx + dy * dy).sqrt().max(1e-9);
            k_max = k_max.max(s.table().curvature(r).abs());
            r_min = r_min.min(r);
            let vx = s.atoms[j].vx - s.atoms[i].vx;
            let vy = s.atoms[j].vy - s.atoms[i].vy;
            let e = 0.5 * mu * (vx * vx + vy * vy) + s.table().u(r);
            if e > e_rel_max {
                e_rel_max = e;
            }
        }
    }
    (k_max, e_rel_max, r_min)
}

fn main() {
    let mut s = loaded();
    s.boundary = Boundary::Walls;
    s.reset(11);

    // Running truths, sampled every substep.
    let mut true_k_max: f64 = 0.0;
    let mut true_e_rel_max = f64::NEG_INFINITY;
    let mut true_r_min = f64::INFINITY;
    let mut max_wall_pen: f64 = 0.0;
    let mut first_breach: Option<(f64, f64, f64)> = None;

    println!(
        "{:>9} {:>7} {:>11} {:>11} {:>7}  {:>10} {:>10} {:>6} {:>9} {:>9} {:>6} {:>5}",
        "t",
        "frame",
        "drift",
        "bound",
        "%bound",
        "k_env",
        "k_true",
        "kratio",
        "e_ref",
        "modes",
        "mratio",
        "bond"
    );
    let mut frame = 0u64;
    while s.time < 6200.0 {
        for _ in 0..64 {
            s.step();
            let (k, e, rmin) = true_maxima(&s);
            true_k_max = true_k_max.max(k);
            if e > true_e_rel_max {
                true_e_rel_max = e;
            }
            true_r_min = true_r_min.min(rmin);
            for i in 0..s.n {
                let a = &s.atoms[i];
                let lo = s.wall_inset;
                let (hx, hy) = (s.width - s.wall_inset, s.height - s.wall_inset);
                for d in [lo - a.x, a.x - hx, lo - a.y, a.y - hy] {
                    if d > max_wall_pen {
                        max_wall_pen = d;
                    }
                }
            }
        }
        s.close_grain();
        frame += 1;

        let drift = s.drift_peak;
        let bound = s.drift_bound();
        if drift > bound && first_breach.is_none() {
            first_breach = Some((s.time, drift, bound));
        }
        if frame.is_multiple_of(6) || (drift > bound && frame.is_multiple_of(2)) {
            // The amplitude factor, and the quantity it is SUPPOSED to stand for. The
            // harmonic derivation sums each mode's own energy; `energy()` is the SIGNED
            // total, in which kinetic and (negative) bond potential cancel.
            let modes = s.e_kin + s.e_pair.abs() + s.e_wall + s.e_spring;
            println!(
                "{:>9.1} {:>7} {:>11.4e} {:>11.4e} {:>6.1}%  {:>10.4} {:>10.4} {:>6.2} {:>9.4} {:>9.4} {:>6.1} {:>5}",
                s.time,
                frame,
                drift,
                bound,
                100.0 * drift / bound,
                s.timescale.k_env,
                true_k_max,
                true_k_max / s.timescale.k_env,
                s.e_ref,
                modes,
                modes / s.e_ref,
                s.holons.molecule_count()
            );
        }
    }

    let mu = 0.5 * M_H;
    println!("\n--- verdict inputs ---");
    println!(
        "envelope believes: k_env = {:.4}  (from e_rel_max = {:.4e} sampled AT BOUNDARIES)",
        s.timescale.k_env, s.timescale.e_rel_max
    );
    println!(
        "trajectory reached: k_true = {:.4}  (from e_rel = {:.4e} seen BETWEEN boundaries), r_min = {:.4} bohr",
        true_k_max, true_e_rel_max, true_r_min
    );
    println!(
        "  under-derivation factor on k: {:.3}x  -> on omega: {:.3}x  -> on the bound: {:.3}x",
        true_k_max / s.timescale.k_env,
        (true_k_max / s.timescale.k_env).sqrt(),
        true_k_max / s.timescale.k_env
    );
    println!(
        "wall: max penetration {:.4} bohr, omega_wall = {:.4e}, omega_env = {:.4e} (wall binds: {})",
        max_wall_pen,
        (K_WALL / M_H).sqrt(),
        s.timescale.omega_env,
        (K_WALL / M_H) > s.timescale.omega_env * s.timescale.omega_env
    );
    println!(
        "bound with TRUE curvature would be {:.4e} vs displayed {:.4e}; drift {:.4e}",
        4.0 * 0.25 * (true_k_max / mu) * s.dt() * s.dt() * s.e_ref.max(s.table().d_e.abs()),
        s.drift_bound(),
        s.drift_peak
    );
    let modes = s.e_kin + s.e_pair.abs() + s.e_wall + s.e_spring;
    println!(
        "AMPLITUDE FACTOR: e_ref = {:.4} (max |E_kin + E_pair + ...| over the run, floored at D_e = {:.4})",
        s.e_ref, s.table().d_e
    );
    println!(
        "  but the modes actually carry E_kin {:.4} + |E_pair| {:.4} + E_wall {:.4} = {:.4}  ({:.1}x e_ref)",
        s.e_kin, s.e_pair.abs(), s.e_wall, modes, modes / s.e_ref
    );
    println!(
        "  signed total energy = {:.6} -- kinetic and bond potential CANCEL, which is what e_ref reads",
        s.energy()
    );
    match first_breach {
        Some((t, d, b)) => println!(
            "FIRST BREACH at t = {t:.1}: drift {d:.4e} > bound {b:.4e} ({:.1}%)",
            100.0 * d / b
        ),
        None => println!("no breach in this run"),
    }
    println!(
        "energy gate: {}  momentum gate: {} (residual {:.3e} vs {:.3e})",
        if s.energy_gate() { "PASS" } else { "FAIL" },
        if s.momentum_gate() { "PASS" } else { "FAIL" },
        s.momentum_residual_peak,
        s.momentum_bound()
    );
}
