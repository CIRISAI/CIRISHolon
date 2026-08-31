//! ozone_seam_scan.rs — Seam scanner across the ring-closure and reactive channels in Ozone (O, O, O).
//!
//! Evaluates dE3, 2nd and 3rd divided differences in c = sqrt(1 - u), and tests for
//! lower-envelope electronic state crossings between the cyclic D3h ring minimum (theta ~ 60 deg)
//! and open C2v ground state (theta ~ 116.8 deg) via warm-start cross-checks.

use holon_chem::dual::D2;
use holon_chem::elements::OXYGEN;
use holon_chem::fci::{ci_ints, davidson_eigh_from, Order, DAVIDSON_MAX_ITER, DAVIDSON_REQUESTED_TOLERANCE};
use holon_chem::ozone::{de3_with, C_HI, C_LO};
use holon_chem::pair::{atom_energy, geometry_problem, pair_point};
use std::sync::atomic::Ordering;
use std::time::Instant;

fn geom(s1: f64, s2: f64, c: f64) -> Vec<[D2; 3]> {
    let u = (1.0 - c * c).clamp(-1.0, 1.0);
    let s = (1.0 - u * u).max(0.0).sqrt();
    vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
        [D2::c(s1), D2::c(0.0), D2::c(0.0)],
        [D2::c(s2 * u), D2::c(s2 * s), D2::c(0.0)],
    ]
}

fn solve_fci(s1: f64, s2: f64, c: f64, start: Option<&[f64]>) -> (f64, Vec<f64>, usize, f64) {
    let (space, mo, nuc) = geometry_problem(&[OXYGEN, OXYGEN, OXYGEN], geom(s1, s2, c));
    let ci0 = ci_ints(&mo, Order::Value);
    let diag = space.diagonal(&ci0);
    let (e, v, iters, resid, _exit) = davidson_eigh_from(
        &space,
        &ci0,
        &diag,
        DAVIDSON_REQUESTED_TOLERANCE,
        DAVIDSON_MAX_ITER.load(Ordering::Relaxed),
        start,
    );
    (e + nuc.v, v, iters, resid)
}

fn scan_slice(name: &str, s1: f64, s2: f64, n_points: usize) {
    println!("\n================================================================================");
    println!("OZONE SEAM SCAN: {} (s1 = {:.3} bohr, s2 = {:.3} bohr, {} points)", name, s1, s2, n_points);
    println!("================================================================================");

    let e_o = atom_energy(OXYGEN);
    let e_s1 = pair_point(OXYGEN, OXYGEN, s1).e;
    let e_s2 = pair_point(OXYGEN, OXYGEN, s2).e;

    let h = (C_HI - C_LO) / (n_points - 1) as f64;
    let cs: Vec<f64> = (0..n_points).map(|i| C_LO + h * i as f64).collect();

    println!("   {:>8} {:>8} {:>8} {:>15} {:>12} {:>12} {:>10}", "c", "theta", "s3", "dE3 (Ha)", "d2", "d3", "dE(warm)");

    let t0 = Instant::now();
    let mut vals = Vec::with_capacity(n_points);
    let mut warm_diffs = Vec::with_capacity(n_points);

    // Initial carrier vector from first node
    let (_, mut carrier, _, _) = solve_fci(s1, s2, cs[0], None);

    for (i, &c) in cs.iter().enumerate() {
        let u = (1.0 - c * c).clamp(-1.0, 1.0);
        let theta = u.acos().to_degrees();
        let s3 = (s1 * s1 + s2 * s2 - 2.0 * s1 * s2 * u).max(0.0).sqrt();
        let e_s3 = pair_point(OXYGEN, OXYGEN, s3).e;

        let (e_cold, v_cold, _iters, _resid) = solve_fci(s1, s2, c, None);
        let (e_warm, v_warm, _, _) = solve_fci(s1, s2, c, Some(&carrier));

        let de3 = de3_with(s1, s2, s3, e_o, e_s1, e_s2, e_s3);
        vals.push(de3);

        let d_warm_cold = e_warm - e_cold;
        warm_diffs.push(d_warm_cold);

        carrier = if e_warm <= e_cold { v_warm } else { v_cold };

        if i % 5 == 0 || i == n_points - 1 {
            println!("   {:>8.4} {:>8.2}° {:>8.4} {:>15.8} {:>12} {:>12} {:>10.2e}", c, theta, s3, de3, "-", "-", d_warm_cold);
        }
    }

    // Compute divided differences
    let mut max_d3 = 0.0f64;
    let mut max_d3_c = 0.0f64;
    for i in 2..(n_points - 2) {
        let d3 = (vals[i + 2] - 2.0 * vals[i + 1] + 2.0 * vals[i - 1] - vals[i - 2]) / (2.0 * h * h * h);
        if d3.abs() > max_d3 {
            max_d3 = d3.abs();
            max_d3_c = cs[i];
        }
    }

    let elapsed = t0.elapsed();
    let min_warm_diff = warm_diffs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_warm_diff = warm_diffs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    println!("--------------------------------------------------------------------------------");
    println!("Scan results for {}:", name);
    println!("  Points scanned       = {}", n_points);
    println!("  Time elapsed         = {:.2?}", elapsed);
    println!("  Max |d3|             = {:.4e} (at c = {:.4})", max_d3, max_d3_c);
    println!("  Warm - Cold diff min = {:+.3e} Ha", min_warm_diff);
    println!("  Warm - Cold diff max = {:+.3e} Ha", max_warm_diff);
    println!("  Electronic stability = {}", if min_warm_diff >= -1e-8 { "STABLE VARIATIONAL GROUND STATE" } else { "BRANCH FLIP DETECTED" });
}

fn main() {
    println!("================================================================================");
    println!("OZONE (O, O, O) ELECTRONIC SEAM SCANNER — D3h / C2v REACTIVE CHANNELS");
    println!("================================================================================");

    // Slice 1: Near-equilibrium ozone (s1 = s2 = 2.41 bohr), sweeping theta from 20 deg to 180 deg
    // Encompasses the cyclic D3h minimum (theta = 60 deg) and open C2v minimum (theta = 116.8 deg)
    scan_slice("Slice 1: Cyclic D3h <-> Open C2v Bend (s1 = 2.41 bohr, s2 = 2.41 bohr)", 2.41, 2.41, 25);

    // Slice 2: Compressed ring region (s1 = 2.10 bohr, s2 = 2.10 bohr)
    scan_slice("Slice 2: Compressed Ring Channel (s1 = 2.10 bohr, s2 = 2.10 bohr)", 2.10, 2.10, 25);

    // Slice 3: Asymmetric dissociation path (s1 = 2.28 bohr, s2 = 3.20 bohr)
    scan_slice("Slice 3: Asymmetric Reactive Channel (s1 = 2.28 bohr, s2 = 3.20 bohr)", 2.28, 3.20, 25);
}
