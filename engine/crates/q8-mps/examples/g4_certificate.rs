//! G4 — the certificate. `Q8_MPS_PREREG.md` §5, re-staked by Amendment 2 (2026-08-24).
//!
//! Chi ladder {16,32,64,128,256} at all 8 (N,U) configurations of the AMENDED grid `N in
//! {8,10}` (`N=12` demoted mid-run on a resource decision — see the amendment). Calibrated on
//! N=8 ALONE (the original design pooled two N's precisely so the fit wasn't a single-N fluke;
//! this is a genuine weakening, not a relabeling), held out on N=10. Reports the fitted power
//! law, R², the held-out log-band, and the theoretical prior on the exponent — as a diagnostic,
//! per the prereg, never as an override of the two staked checks. Prints a clear PASS/FAIL
//! verdict and the policy that follows from it (quote the fit at N~100, or refuse and report
//! raw weight only) — a thinner calibration set firing the refuse-to-quote policy is the policy
//! working correctly, not a failure.
//!
//! Run: `cargo run --release --manifest-path crates/q8-mps/Cargo.toml --example g4_certificate`

use q8_mps::dmrg::{self, Params, RefusalPolicy};
use std::time::Instant;

const CHI_LADDER: [usize; 5] = [16, 32, 64, 128, 256];
const U_GRID: [f64; 4] = [0.0, 1.0, 4.0, 16.0];
const N_GRID: [usize; 2] = [8, 10];

const CALIBRATION_R2_MIN: f64 = 0.85;
const HELD_OUT_LOG_BAND: f64 = 3.0; // factor of 3, log10 space
const FLOOR: f64 = 1e-14; // pairs with epsilon or dE below this are excluded from the log-log fit

struct Point {
    sites: usize,
    u: f64,
    chi: usize,
    epsilon: f64, // accumulated discarded weight, final sweep
    d_energy: f64, // |E_MPS - E_exact|, absolute
}

fn main() {
    let mut points: Vec<Point> = Vec::new();

    for &sites in &N_GRID {
        eprintln!("=== N={sites}: computing q-seam exact reference (dev-dependency, live) ===");
        for &u in &U_GRID {
            let t0 = Instant::now();
            let h = q_seam::hubbard::Hubbard::new(sites, 1.0, u);
            let exact = match q_seam::lanczos::ground_state(&h) {
                Some(g) => g,
                None => {
                    eprintln!("N={sites} U={u}: q-seam Lanczos FAILED — skipping this (N,U)");
                    continue;
                }
            };
            eprintln!("N={sites} U={u}: exact E={:.10} ({:.1?})", exact.energy, t0.elapsed());

            for &chi in &CHI_LADDER {
                let t1 = Instant::now();
                let p = Params { sites, t: 1.0, u, chi_max: chi, max_sweeps: 20, sweep_tol: 1e-10 };
                let r = match dmrg::run(&p, RefusalPolicy::Typed) {
                    Ok(r) => r,
                    Err(refusal) => {
                        eprintln!(
                            "N={sites} U={u} chi={chi}: REFUSED (bond={} weight={:e}) — excluded from the fit",
                            refusal.bond, refusal.weight
                        );
                        continue;
                    }
                };
                let n_target = sites as f64;
                let mu = u / 2.0;
                let unshifted = r.energy_shifted + mu * n_target;
                let d_energy = (unshifted - exact.energy).abs();
                let epsilon = r.discarded_weight.iter().sum::<f64>();

                eprintln!(
                    "N={sites} U={u} chi={chi}: eps={epsilon:e} dE={d_energy:e} sweeps={} converged={} ({:.1?})",
                    r.sweeps_used, r.converged, t1.elapsed()
                );
                points.push(Point { sites, u, chi, epsilon, d_energy });
            }
        }
    }

    eprintln!("\n=== fitting: log(dE) = log(c) + p*log(eps), calibration = N=8 ONLY (Amendment 2) ===");
    let calib: Vec<&Point> = points
        .iter()
        .filter(|pt| pt.sites == 8 && pt.epsilon > FLOOR && pt.d_energy > FLOOR)
        .collect();
    eprintln!("calibration points: {} (of {} total N=8 points, rest floored out)", calib.len(),
        points.iter().filter(|pt| pt.sites == 8).count());

    let xs: Vec<f64> = calib.iter().map(|pt| pt.epsilon.ln()).collect();
    let ys: Vec<f64> = calib.iter().map(|pt| pt.d_energy.ln()).collect();
    let (slope, intercept, r2) = linear_fit(&xs, &ys);
    let c = intercept.exp();

    println!("\n=== G4 CALIBRATION FIT ===");
    println!("dE = {c:e} * eps^{slope:.4}   (R^2 = {r2:.4}, n={})", calib.len());
    println!(
        "theoretical prior: p ~ 1 (first-order in discarded weight) — fitted p={slope:.4}, {}",
        if (slope - 1.0).abs() <= 0.5 { "consistent with the prior" } else { "FAR from the prior — diagnostic, does not override the staked checks" }
    );
    let fit_passes = r2 >= CALIBRATION_R2_MIN;
    println!(
        "STAKED: R^2 >= {CALIBRATION_R2_MIN} -> {}",
        if fit_passes { "PASS" } else { "FAIL" }
    );

    eprintln!("\n=== held-out prediction: N=10 (Amendment 2's re-staked hold-out) ===");
    let held_out: Vec<&Point> = points
        .iter()
        .filter(|pt| pt.sites == 10 && pt.epsilon > FLOOR && pt.d_energy > FLOOR)
        .collect();
    let mut log_ratios: Vec<f64> = held_out
        .iter()
        .map(|pt| {
            let predicted = c * pt.epsilon.powf(slope);
            (predicted / pt.d_energy).log10().abs()
        })
        .collect();
    log_ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_log_ratio = if log_ratios.is_empty() {
        f64::INFINITY
    } else {
        let n = log_ratios.len();
        if n % 2 == 1 {
            log_ratios[n / 2]
        } else {
            (log_ratios[n / 2 - 1] + log_ratios[n / 2]) / 2.0
        }
    };
    let band_log = HELD_OUT_LOG_BAND.log10();
    let held_out_passes = median_log_ratio <= band_log;

    println!("\n=== G4 HELD-OUT (N=10) ===");
    println!(
        "n={} points, median |log10(predicted/actual)| = {median_log_ratio:.4} (band = log10({HELD_OUT_LOG_BAND}) = {band_log:.4})",
        held_out.len()
    );
    println!(
        "STAKED: median <= log10({HELD_OUT_LOG_BAND}) -> {}",
        if held_out_passes { "PASS" } else { "FAIL" }
    );

    println!("\n=== G4 VERDICT ===");
    if fit_passes && held_out_passes {
        println!("PASS — quote dE ~= {c:e} * eps^{slope:.4} at N~100 as an extrapolated estimate, never a bound.");
    } else {
        println!("FAIL — POLICY: refuse to quote a derived error bar at N~100. Report raw accumulated discarded weight only.");
    }

    println!("\n=== raw points ({}) ===", points.len());
    println!("{:>3} {:>6} {:>5} {:>12} {:>12}", "N", "U", "chi", "epsilon", "dE");
    for pt in &points {
        println!("{:>3} {:>6} {:>5} {:>12.4e} {:>12.4e}", pt.sites, pt.u, pt.chi, pt.epsilon, pt.d_energy);
    }
}

/// Ordinary least squares `y = slope*x + intercept`, and R^2.
fn linear_fit(xs: &[f64], ys: &[f64]) -> (f64, f64, f64) {
    let n = xs.len() as f64;
    let xbar = xs.iter().sum::<f64>() / n;
    let ybar = ys.iter().sum::<f64>() / n;
    let sxy: f64 = xs.iter().zip(ys).map(|(x, y)| (x - xbar) * (y - ybar)).sum();
    let sxx: f64 = xs.iter().map(|x| (x - xbar) * (x - xbar)).sum();
    let slope = sxy / sxx;
    let intercept = ybar - slope * xbar;

    let ss_tot: f64 = ys.iter().map(|y| (y - ybar) * (y - ybar)).sum();
    let ss_res: f64 = xs.iter().zip(ys).map(|(x, y)| {
        let pred = slope * x + intercept;
        (y - pred) * (y - pred)
    }).sum();
    let r2 = 1.0 - ss_res / ss_tot;

    (slope, intercept, r2)
}
