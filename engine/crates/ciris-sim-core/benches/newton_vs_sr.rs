//! The T5 speed clause, reading (a): the licensed Newtonian chart must step >= 2x
//! faster than full SR at EQUAL TOLERANCE on the licensed segment class (force-free +
//! uniform-boost segments). CURVATURE_BRIDGE.md section 5; DESCRIPTOR_CHAIN.md
//! section 3.5. Failure BLOCKS the speed half of the Newton-chart license (the
//! certificate then stands on accuracy alone) — the consequence is stated here so a
//! FAIL line in this report is a decision, not trivia.
//!
//! Protocol: for each tolerance, find the SR step count whose relative error on u^1
//! against the hyperbolic closed form meets it (integrator-tolerance budget, NEVER an
//! ulp bar — the two-fp32-runtimes lesson), then wall-clock both charts.
//!
//! STRUCTURAL HONESTY NOTE: on this segment class the Newtonian trajectory is a
//! polynomial in t, which RK4 integrates exactly, so the Newton side meets ANY
//! tolerance in one step per segment and the equal-tolerance ratio is structurally
//! large (measured ~7.7e2 at 1e-9, ~3.0e3 at 1e-12). The per-step cost ratio at
//! equal step counts is reported beside it and MEASURED at ~10.5x — NOT the ~1.5-2.5x
//! gamma-overhead figure section 3.5 estimates, and the discrepancy is understood
//! rather than smoothed: on the licensed class the Newtonian force is LITERALLY
//! constant (zero arithmetic per evaluation) while the SR pusher rotates 8 data-
//! dependent components, so the asymmetry here is force-evaluation cost, not gamma
//! arithmetic. The 1.5-2.5x figure presumes force-dominated loops (the reading-(b)
//! workload-matched comparison, a field gather against a nontrivial field), which
//! this slice deliberately does not implement. If the per-step ratio ever fell below
//! 1, the license's premise (SR costs more per step) would be dead and this header
//! would have to say so.

use ciris_sim_core::relativity::{
    hyperbolic_worldline, pack, rk4_step, BoostField, Worldline, SPEED_OF_LIGHT_M_S,
};
use std::hint::black_box;
use std::time::Instant;

const C: f64 = SPEED_OF_LIGHT_M_S;
const TAU_TOTAL: f64 = 2.0; // rapidity 2 at alpha = c

fn sr_error(steps: u32) -> f64 {
    let field = BoostField { alpha_m_s2: C, dir: [1.0, 0.0, 0.0] };
    let f = |y: &[f64; 8]| {
        let u = [y[4], y[5], y[6], y[7]];
        let a = field.proper_accel(&u);
        [y[4], y[5], y[6], y[7], a[0], a[1], a[2], a[3]]
    };
    let dtau = TAU_TOTAL / steps as f64;
    let mut y = pack(&Worldline::from_celerity([0.0; 4], [0.0; 3]));
    for _ in 0..steps {
        y = rk4_step(&f, &y, dtau);
    }
    let exact = hyperbolic_worldline(C, TAU_TOTAL);
    ((y[5] - exact.u[1]) / exact.u[1]).abs()
}

fn time_sr(steps: u32, reps: u32) -> f64 {
    let field = BoostField { alpha_m_s2: C, dir: [1.0, 0.0, 0.0] };
    let f = |y: &[f64; 8]| {
        let u = [y[4], y[5], y[6], y[7]];
        let a = field.proper_accel(&u);
        [y[4], y[5], y[6], y[7], a[0], a[1], a[2], a[3]]
    };
    let dtau = TAU_TOTAL / steps as f64;
    let start = Instant::now();
    let mut sink = 0.0;
    for _ in 0..reps {
        let mut y = pack(&Worldline::from_celerity([0.0; 4], [0.0; 3]));
        for _ in 0..steps {
            // black_box on the state each step so BOTH charts execute their
            // arithmetic; without it LLVM folds the Newton side's constant-force
            // loop entirely and the "per-step ratio" measures constant-folding
            // (first run of this bench read 92x where gamma overhead is ~2x).
            y = rk4_step(&f, black_box(&y), dtau);
        }
        sink += y[5];
    }
    let dt = start.elapsed().as_secs_f64();
    assert!(sink.is_finite());
    dt / reps as f64
}

/// Newton chart: RK4 on (x, v), 6 dof, constant acceleration a = alpha.
fn rk4_6<F: Fn(&[f64; 6]) -> [f64; 6]>(f: &F, y: &[f64; 6], h: f64) -> [f64; 6] {
    let k1 = f(y);
    let mut y2 = [0.0; 6];
    for i in 0..6 {
        y2[i] = y[i] + 0.5 * h * k1[i];
    }
    let k2 = f(&y2);
    let mut y3 = [0.0; 6];
    for i in 0..6 {
        y3[i] = y[i] + 0.5 * h * k2[i];
    }
    let k3 = f(&y3);
    let mut y4 = [0.0; 6];
    for i in 0..6 {
        y4[i] = y[i] + h * k3[i];
    }
    let k4 = f(&y4);
    let mut out = [0.0; 6];
    for i in 0..6 {
        out[i] = y[i] + (h / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
    }
    out
}

fn time_newton(steps: u32, reps: u32) -> f64 {
    let f = |y: &[f64; 6]| [y[3], y[4], y[5], C, 0.0, 0.0];
    let dt_step = TAU_TOTAL / steps as f64;
    let start = Instant::now();
    let mut sink = 0.0;
    for _ in 0..reps {
        let mut y = [0.0; 6];
        for _ in 0..steps {
            y = rk4_6(&f, black_box(&y), dt_step);
        }
        sink += y[3];
    }
    let dt = start.elapsed().as_secs_f64();
    assert!(sink.is_finite());
    dt / reps as f64
}

fn median(mut xs: Vec<f64>) -> f64 {
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    xs[xs.len() / 2]
}

fn main() {
    println!("T5 speed clause, reading (a): Newton chart vs own-best SR at equal tolerance");
    println!("segment: uniform boost, rapidity 2; error metric: relative u^1 vs closed form\n");

    for tol in [1.0e-9_f64, 1.0e-12] {
        // Find the SR step count meeting the tolerance (double-and-settle).
        let mut steps = 8_u32;
        while sr_error(steps) > tol {
            steps *= 2;
        }
        let err = sr_error(steps);

        // Newton meets any tolerance at 1 step on this class (polynomial exactness);
        // time it at 1 step per segment, and also both at EQUAL steps for the
        // per-step ratio.
        let reps = 2000;
        let t_sr = median((0..9).map(|_| time_sr(steps, reps)).collect());
        let t_newton = median((0..9).map(|_| time_newton(1, reps)).collect());
        let equal_tol_ratio = t_sr / t_newton;

        let t_sr_eq = median((0..9).map(|_| time_sr(steps, reps)).collect());
        let t_n_eq = median((0..9).map(|_| time_newton(steps, reps)).collect());
        let per_step_ratio = t_sr_eq / t_n_eq;

        let verdict = if equal_tol_ratio >= 2.0 { "PASS" } else { "FAIL (license's speed half blocked)" };
        println!(
            "tol {tol:.0e}: SR needs {steps} steps (err {err:.2e}); \
             equal-tolerance ratio {equal_tol_ratio:.1}x -> {verdict}"
        );
        println!(
            "          per-step cost ratio (8-dof+gamma vs 6-dof, equal steps): \
             {per_step_ratio:.2}x\n"
        );
    }
}
