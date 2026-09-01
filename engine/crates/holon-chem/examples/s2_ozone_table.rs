//! Produce the committed (O, O, O) Ozone three-body table.
//!
//! Evaluates the STO-3G FCI (O,O,O) three-body term over the symmetry-reduced
//! grid (NR=33, NU=25, N_SOLVED=14,025 points) multi-threaded with 32 worker threads,
//! warm-starting along u-rays and checkpointing to an append-only progress log.

use holon_chem::dual::D2;
use holon_chem::elements::OXYGEN;
use holon_chem::fci::{ci_ints, davidson_eigh_from, Order, DAVIDSON_MAX_ITER, DAVIDSON_REQUESTED_TOLERANCE};
use holon_chem::ozone::{
    self, node_index, node_r, node_u, third_side, N_NODES, N_SOLVED, NR, NU,
};
use holon_chem::pair::{atom_energy, geometry_problem, pair_point};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

const PROGRESS_PATH: &str = "output/s2_ozone_table_progress.log";

struct PairCurve {
    lo: f64,
    hi: f64,
    e: Vec<f64>,
    d: Vec<f64>,
}

impl PairCurve {
    fn sample(a: holon_chem::elements::Species, b: holon_chem::elements::Species) -> Self {
        let n = 256;
        let (lo, hi) = (1.0f64, 15.0f64);
        let pairs: Vec<Mutex<(f64, f64)>> = (0..n).map(|_| Mutex::new((0.0, 0.0))).collect();
        let next_idx = AtomicUsize::new(0);

        std::thread::scope(|sc| {
            for _ in 0..32 {
                let pairs_ref = &pairs;
                let next_ref = &next_idx;
                sc.spawn(move || loop {
                    let i = next_ref.fetch_add(1, Ordering::SeqCst);
                    if i >= n {
                        break;
                    }
                    let p = pair_point(a, b, lo + (hi - lo) * i as f64 / (n - 1) as f64);
                    *pairs_ref[i].lock().unwrap() = (p.e, -p.f);
                });
            }
        });

        let mut e = Vec::with_capacity(n);
        let mut d = Vec::with_capacity(n);
        for m in pairs {
            let (ev, dv) = m.into_inner().unwrap();
            e.push(ev);
            d.push(dv);
        }
        Self { lo, hi, e, d }
    }

    fn at(&self, r: f64) -> f64 {
        let n = self.e.len();
        let h = (self.hi - self.lo) / (n - 1) as f64;
        let t = (r - self.lo) / h;
        let k = (t.floor() as usize).min(n - 2);
        let s = (t - k as f64).clamp(0.0, 1.0);
        let s2 = s * s;
        let s3 = s2 * s;
        let h00 = 2.0 * s3 - 3.0 * s2 + 1.0;
        let h10 = s3 - 2.0 * s2 + s;
        let h01 = -2.0 * s3 + 3.0 * s2;
        let h11 = s3 - s2;
        h00 * self.e[k] + h10 * h * self.d[k] + h01 * self.e[k + 1] + h11 * h * self.d[k + 1]
    }
}

fn geom(s1: f64, s2: f64, u: f64) -> Vec<[D2; 3]> {
    let u_clamped = u.clamp(-1.0, 1.0);
    let s = (1.0 - u_clamped * u_clamped).max(0.0).sqrt();
    vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
        [D2::c(s1), D2::c(0.0), D2::c(0.0)],
        [D2::c(s2 * u_clamped), D2::c(s2 * s), D2::c(0.0)],
    ]
}

fn solve_fci(s1: f64, s2: f64, u: f64, start: Option<&[f64]>) -> (f64, Vec<f64>) {
    let (space, mo, nuc) = geometry_problem(&[OXYGEN, OXYGEN, OXYGEN], geom(s1, s2, u));
    let ci0 = ci_ints(&mo, Order::Value);
    let diag = space.diagonal(&ci0);
    let (e, v, _iters, _resid, _exit) = davidson_eigh_from(
        &space,
        &ci0,
        &diag,
        DAVIDSON_REQUESTED_TOLERANCE,
        DAVIDSON_MAX_ITER.load(Ordering::Relaxed),
        start,
    );
    (e + nuc.v, v)
}

fn load_progress() -> HashMap<usize, f64> {
    let mut map = HashMap::new();
    if let Ok(file) = File::open(PROGRESS_PATH) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if let Some(rest) = line.strip_prefix("KNOT:") {
                let mut idx = None;
                let mut val = None;
                for part in rest.split_whitespace() {
                    if let Some(v) = part.strip_prefix("idx=") {
                        idx = v.parse::<usize>().ok();
                    } else if let Some(v) = part.strip_prefix("val=") {
                        val = v.parse::<f64>().ok();
                    }
                }
                if let (Some(i), Some(v)) = (idx, val) {
                    map.insert(i, v);
                }
            }
        }
    }
    map
}

fn main() {
    let mut args = std::env::args().skip(1);
    let threads: usize = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(16));
    let out = args.next().unwrap_or_else(|| {
        format!(
            "{}/tests/data/s2/s2_ozone_table.txt",
            env!("CARGO_MANIFEST_DIR")
        )
    });

    if let Some(parent) = Path::new(PROGRESS_PATH).parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if let Some(parent) = Path::new(&out).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let t0 = Instant::now();
    let e_o = atom_energy(OXYGEN);
    println!("# Sampling O2 pair reference curve...");
    let pair_oo = PairCurve::sample(OXYGEN, OXYGEN);

    let cached = load_progress();
    println!("# grid {}", ozone::grid_line());
    println!("# E(O) = {e_o:.17} Ha");
    println!("# {N_SOLVED} solves ({N_NODES} nodes), {threads} threads");
    println!("# Loaded {} cached knots from {}", cached.len(), PROGRESS_PATH);

    let log_file = Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(PROGRESS_PATH)
            .expect("cannot open progress log"),
    ));

    let vals: Vec<Mutex<f64>> = (0..N_NODES).map(|_| Mutex::new(0.0)).collect();

    // Pre-fill cached knots
    for (&idx, &val) in &cached {
        if idx < N_NODES {
            *vals[idx].lock().unwrap() = val;
        }
    }

    let done = AtomicUsize::new(cached.len());
    let next_row = AtomicUsize::new(0);

    std::thread::scope(|sc| {
        for _ in 0..threads {
            let log_ref = &log_file;
            let cached_ref = &cached;
            let vals_ref = &vals;
            let done_ref = &done;
            let next_ref = &next_row;
            let pair_ref = &pair_oo;

            sc.spawn(move || loop {
                let i = next_ref.fetch_add(1, Ordering::SeqCst);
                if i >= NR {
                    break;
                }
                let s1 = node_r(i);
                let e_s1 = pair_ref.at(s1);

                for j in i..NR {
                    let s2 = node_r(j);
                    let e_s2 = pair_ref.at(s2);

                    let mut carrier: Option<Vec<f64>> = None;

                    for k in 0..NU {
                        let idx = node_index(i, j, k);
                        let idx_mirror = node_index(j, i, k);

                        let d = if let Some(&cached_val) = cached_ref.get(&idx) {
                            cached_val
                        } else {
                            let u = node_u(k);
                            let s3 = third_side(s1, s2, u);
                            let e_s3 = pair_ref.at(s3);

                            let (e_o3, v_o3) = solve_fci(s1, s2, u, carrier.as_deref());
                            carrier = Some(v_o3);

                            let val = e_o3 + 3.0 * e_o - e_s1 - e_s2 - e_s3;

                            // Append to checkpoint log
                            {
                                let mut f = log_ref.lock().unwrap();
                                writeln!(f, "KNOT: idx={} i={} j={} k={} val={:.16e}", idx, i, j, k, val).unwrap();
                                f.flush().unwrap();
                            }

                            let d_cnt = done_ref.fetch_add(1, Ordering::SeqCst) + 1;
                            if d_cnt % 10 == 0 || d_cnt == N_SOLVED {
                                println!(
                                    "  knot {:>5}/{:>5} ({:>5.1}%) done: (i={:>2}, j={:>2}, k={:>2}) -> dE3={:+.8} Ha [{:.1}s elapsed]",
                                    d_cnt, N_SOLVED, (d_cnt as f64 / N_SOLVED as f64) * 100.0, i, j, k, val, t0.elapsed().as_secs_f64()
                                );
                                std::io::stdout().flush().unwrap();
                            }
                            val
                        };

                        *vals_ref[idx].lock().unwrap() = d;
                        if i != j {
                            *vals_ref[idx_mirror].lock().unwrap() = d;
                        }
                    }
                }
            });
        }
    });

    let vals: Vec<f64> = vals.into_iter().map(|m| m.into_inner().unwrap()).collect();

    let mut t = ozone::OzoneTable::empty();
    t.begin();
    let mut peak = 0.0f64;
    for (i, v) in vals.iter().enumerate() {
        peak = peak.max(v.abs());
        assert!(t.knot(i, *v), "node {i} refused: {v}");
    }
    let meta = ozone::OzoneMeta {
        e_o_atom: e_o,
        peak,
        solves: N_SOLVED,
    };
    assert!(t.finish(meta), "the table did not close");

    let text = ozone::to_text(&t);
    std::fs::write(&out, &text).unwrap_or_else(|e| panic!("cannot write {out}: {e}"));
    println!(
        "\nwrote {out} ({} bytes) in {:.1} s\n  peak |dE3| = {peak:.6e} Ha\n  solves = {N_SOLVED}",
        text.len(),
        t0.elapsed().as_secs_f64()
    );

    let back = ozone::from_text(&text).expect("the artifact reloads");
    for i in 0..NR {
        for j in 0..NR {
            for k in 0..NU {
                assert_eq!(
                    t.node(i, j, k).to_bits(),
                    back.node(i, j, k).to_bits(),
                    "node ({i}, {j}, {k}) did not survive the round trip"
                );
            }
        }
    }
    println!("  round trip = bit-identical on all {N_NODES} nodes");
}
