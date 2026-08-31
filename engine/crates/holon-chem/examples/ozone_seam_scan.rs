//! ozone_seam_scan.rs — Multi-threaded Checkpointed Seam Scanner across Ozone (O, O, O) reactive channels.
//!
//! Evaluates ab-initio dE3, 2nd and 3rd divided differences along reaction coordinates,
//! tests for electronic state crossings between cyclic D3h (theta ~ 60 deg) and open C2v (theta ~ 116.8 deg),
//! logs every intermediate point to an append-only checkpoint log, running all 3 slices concurrently in parallel threads.

use holon_chem::dual::D2;
use holon_chem::elements::OXYGEN;
use holon_chem::fci::{ci_ints, davidson_eigh_from, Order, DAVIDSON_MAX_ITER, DAVIDSON_REQUESTED_TOLERANCE};
use holon_chem::ozone::{C_HI, C_LO};
use holon_chem::pair::{atom_energy, geometry_problem, pair_point};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;

const PROGRESS_PATH: &str = "output/ozone_seam_scan_progress.log";

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

#[derive(Clone, Debug)]
struct CachedPoint {
    e_o3: f64,
    de3: f64,
    d_warm: f64,
}

fn load_progress() -> HashMap<(usize, usize), CachedPoint> {
    let mut map = HashMap::new();
    if let Ok(file) = File::open(PROGRESS_PATH) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if let Some(rest) = line.strip_prefix("POINT:") {
                let mut slice_idx = None;
                let mut pt_idx = None;
                let mut e_o3 = None;
                let mut de3 = None;
                let mut d_warm = None;

                for part in rest.split_whitespace() {
                    if let Some(v) = part.strip_prefix("slice=") {
                        slice_idx = v.parse::<usize>().ok();
                    } else if let Some(v) = part.strip_prefix("i=") {
                        pt_idx = v.parse::<usize>().ok();
                    } else if let Some(v) = part.strip_prefix("E_o3=") {
                        e_o3 = v.parse::<f64>().ok();
                    } else if let Some(v) = part.strip_prefix("dE3=") {
                        de3 = v.parse::<f64>().ok();
                    } else if let Some(v) = part.strip_prefix("d_warm=") {
                        d_warm = v.parse::<f64>().ok();
                    }
                }

                if let (Some(s), Some(i), Some(e), Some(d), Some(w)) = (slice_idx, pt_idx, e_o3, de3, d_warm) {
                    map.insert((s, i), CachedPoint { e_o3: e, de3: d, d_warm: w });
                }
            }
        }
    }
    map
}

fn scan_slice(
    slice_idx: usize,
    name: &str,
    s1: f64,
    s2: f64,
    n_points: usize,
    cached: &HashMap<(usize, usize), CachedPoint>,
    log_file: &Arc<Mutex<File>>,
) {
    let e_o = atom_energy(OXYGEN);
    let e_s1 = pair_point(OXYGEN, OXYGEN, s1).e;
    let e_s2 = pair_point(OXYGEN, OXYGEN, s2).e;

    let h = (C_HI - C_LO) / (n_points - 1) as f64;
    let cs: Vec<f64> = (0..n_points).map(|i| C_LO + h * i as f64).collect();

    let mut vals = Vec::with_capacity(n_points);
    let mut warm_diffs = Vec::with_capacity(n_points);
    let mut carrier: Option<Vec<f64>> = None;

    let t_slice = Instant::now();

    for (i, &c) in cs.iter().enumerate() {
        let u = (1.0 - c * c).clamp(-1.0, 1.0);
        let theta = u.acos().to_degrees();
        let s3 = (s1 * s1 + s2 * s2 - 2.0 * s1 * s2 * u).max(0.0).sqrt();
        let e_s3 = pair_point(OXYGEN, OXYGEN, s3).e;

        if let Some(cp) = cached.get(&(slice_idx, i)) {
            println!(
                "  [{}] pt {:>2}/{:>2}: c={:.4} theta={:>6.2}° s3={:.4} -> E={:.8} Ha, dE3={:+.8} Ha (CACHED)",
                name, i + 1, n_points, c, theta, s3, cp.e_o3, cp.de3
            );
            std::io::stdout().flush().unwrap();
            vals.push(cp.de3);
            warm_diffs.push(cp.d_warm);
            continue;
        }

        let t_pt = Instant::now();

        // 1. Cold solve
        let (e_cold, v_cold, _, _) = solve_fci(s1, s2, c, None);

        // 2. Warm solve (if carrier available)
        let (e_best, v_best, d_warm_cold) = match carrier.as_ref() {
            Some(w) => {
                let (e_warm, v_warm, _, _) = solve_fci(s1, s2, c, Some(w));
                let diff = e_warm - e_cold;
                if e_warm <= e_cold {
                    (e_warm, v_warm, diff)
                } else {
                    (e_cold, v_cold, diff)
                }
            }
            None => (e_cold, v_cold, 0.0),
        };

        carrier = Some(v_best);

        let de3 = e_best + 3.0 * e_o - e_s1 - e_s2 - e_s3;
        vals.push(de3);
        warm_diffs.push(d_warm_cold);

        let pt_ms = t_pt.elapsed().as_millis();

        // Thread-safe append to checkpoint log
        {
            let mut f = log_file.lock().unwrap();
            writeln!(
                f,
                "POINT: slice={} i={} c={:.8} theta={:.6} s1={:.6} s2={:.6} s3={:.6} E_o3={:.12} dE3={:.12} d_warm={:.6e} time_ms={}",
                slice_idx, i, c, theta, s1, s2, s3, e_best, de3, d_warm_cold, pt_ms
            ).unwrap();
            f.flush().unwrap();
        }

        println!(
            "  [{}] pt {:>2}/{:>2}: c={:.4} theta={:>6.2}° s3={:.4} -> E={:.8} Ha, dE3={:+.8} Ha (d_warm={:+.2e}, {:.1}s)",
            name, i + 1, n_points, c, theta, s3, e_best, de3, d_warm_cold, t_pt.elapsed().as_secs_f64()
        );
        std::io::stdout().flush().unwrap();
    }

    // 3. Compute divided differences
    let mut max_d3 = 0.0f64;
    let mut max_d3_c = 0.0f64;
    for i in 2..(n_points - 2) {
        let d3 = (vals[i + 2] - 2.0 * vals[i + 1] + 2.0 * vals[i - 1] - vals[i - 2]) / (2.0 * h * h * h);
        if d3.abs() > max_d3 {
            max_d3 = d3.abs();
            max_d3_c = cs[i];
        }
    }

    let min_warm_diff = warm_diffs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_warm_diff = warm_diffs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    println!("--------------------------------------------------------------------------------");
    println!("Summary for Slice {} ({}):", slice_idx, name);
    println!("  Points: {} | Wall time: {:.1?}", n_points, t_slice.elapsed());
    println!("  Max |d3|: {:.4e} (at c = {:.4})", max_d3, max_d3_c);
    println!("  Warm-Cold Delta: min={:+.3e} Ha, max={:+.3e} Ha", min_warm_diff, max_warm_diff);
    println!("  Ground State Stability: {}", if min_warm_diff >= -1e-8 { "VARIATIONALLY STABLE" } else { "BRANCH CROSSING DETECTED" });
    println!("--------------------------------------------------------------------------------");
    std::io::stdout().flush().unwrap();
}

fn main() {
    println!("================================================================================");
    println!("OZONE (O, O, O) ELECTRONIC SEAM SCANNER — PARALLEL CHECKPOINT ENGINE");
    println!("================================================================================");

    if let Some(parent) = Path::new(PROGRESS_PATH).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let cached = load_progress();
    println!("# Loaded {} cached seam points from {}", cached.len(), PROGRESS_PATH);

    let log_file = Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(PROGRESS_PATH)
            .expect("cannot open progress log"),
    ));

    let t0 = Instant::now();

    // Execute all 3 slices concurrently in parallel threads
    std::thread::scope(|sc| {
        let (cached_ref, log_ref) = (&cached, &log_file);

        sc.spawn(move || {
            scan_slice(1, "Cyclic D3h <-> Open C2v Bend", 2.41, 2.41, 17, cached_ref, log_ref);
        });

        sc.spawn(move || {
            scan_slice(2, "Compressed Ring Channel", 2.10, 2.10, 17, cached_ref, log_ref);
        });

        sc.spawn(move || {
            scan_slice(3, "Asymmetric Reactive Channel", 2.28, 3.20, 17, cached_ref, log_ref);
        });
    });

    println!("\n# All 3 ozone seam scan slices completed in {:.1?}.", t0.elapsed());
}
