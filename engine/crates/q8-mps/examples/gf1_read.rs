//! GF1's reader: one MPS in, its magic out as JSON on stdout.
//!
//! ```text
//! gf1_read --ckpt PATH [--sweeps 3] [--box 10]          # a Q8SYMCK1 checkpoint (the crate's own format)
//! gf1_read --x 4 --n 16 --chi 8 [--dmrg-sweeps 60] [--tol 1e-11] [--sweeps 3] [--box 10]
//! ```
//!
//! The crate's only banked-state format is `symmetric::SweepCheckpoint` (`Q8SYMCK1`, written by
//! the QCD₂ runner); `schwinger4.rs` prints its energy and keeps no state, and the
//! `conformance/crystal/ckpt4_*.npz` files are the Python driver's numpy, which this crate does
//! not read (no I/O beyond its own format is added here). With `--ckpt` the checkpoint's tensors
//! are read as they are; otherwise a Schwinger vacuum is generated fresh from `schwinger.rs` at
//! the given `x`, `N`, `χ`, its energy variance reported beside the reading, and the JSON says
//! `"source": "generated"`.
//!
//! Printed: `n`, `chi` (the largest bond), `m2`, `m2_loc` (the local-Clifford minimum after
//! `--sweeps` sweeps — equal to `m2` to rounding, by Clifford invariance; see `magic.rs`),
//! `m2_per_site`, `m2_box` (the mixed-state `M₂` of the first `--box` sites) and the price
//! bound `2^{m2_box}`, plus the wall time of each reading.

use q8_mps::magic::{sre2, sre2_box, sre2_local_min};
use q8_mps::mps::TensorSite;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut ckpt: Option<String> = None;
    let (mut n, mut x, mut chi) = (0usize, 0.0f64, 8usize);
    let (mut dmrg_sweeps, mut tol) = (60usize, 1e-11f64);
    let (mut sweeps, mut box_len) = (3usize, 10usize);
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--ckpt" => { ckpt = Some(args[i + 1].clone()); i += 1; }
            "--n" => { n = args[i + 1].parse().expect("--n"); i += 1; }
            "--x" => { x = args[i + 1].parse().expect("--x"); i += 1; }
            "--chi" => { chi = args[i + 1].parse().expect("--chi"); i += 1; }
            "--dmrg-sweeps" => { dmrg_sweeps = args[i + 1].parse().expect("--dmrg-sweeps"); i += 1; }
            "--tol" => { tol = args[i + 1].parse().expect("--tol"); i += 1; }
            "--sweeps" => { sweeps = args[i + 1].parse().expect("--sweeps"); i += 1; }
            "--box" => { box_len = args[i + 1].parse().expect("--box"); i += 1; }
            other => panic!("unknown argument {other}"),
        }
        i += 1;
    }

    let (tensors, source): (Vec<TensorSite>, String) = match ckpt {
        Some(path) => {
            let c = q8_mps::symmetric::SweepCheckpoint::load(std::path::Path::new(&path))
                .unwrap_or_else(|e| panic!("{path}: {e}"));
            (c.tensors, format!("\"source\":\"ckpt\",\"path\":{path:?},\"sweeps_done\":{},\"energy\":{:.12}", c.sweeps_done, c.last_energy))
        }
        None => {
            assert!(n > 0 && x > 0.0, "--ckpt PATH, or --n and --x for a fresh Schwinger vacuum");
            let s = q8_mps::schwinger::Schwinger::new(n, x, vec![]);
            let t0 = Instant::now();
            let (e, res) = s.ground_energy(chi, dmrg_sweeps, tol).expect("the sweep runs");
            let t_dmrg = t0.elapsed().as_secs_f64();
            let (variance, per_site) = match q8_mps::variance::energy_variance(&res.tensors, &s.mpo()) {
                Ok((_, _, v)) => (format!("{v:.6e}"), format!("{:.6e}", v / n as f64)),
                Err(e) => (format!("\"refused: {e}\""), "null".to_string()),
            };
            let max_dw = res.discarded_weight.iter().cloned().fold(0.0f64, f64::max);
            (
                res.tensors,
                format!(
                    "\"source\":\"generated\",\"x\":{x},\"chi_requested\":{chi},\"energy\":{e:.12},\"variance\":{variance},\"variance_per_site\":{per_site},\"converged\":{},\"dmrg_sweeps\":{},\"max_discarded\":{max_dw:.3e},\"dmrg_seconds\":{t_dmrg:.1}",
                    res.converged,
                    res.sweeps_used
                ),
            )
        }
    };

    let n = tensors.len();
    let chi_max = tensors.iter().map(|t| t.chi_l.max(t.chi_r)).max().unwrap_or(1);
    let box_len = box_len.min(n);

    let t0 = Instant::now();
    let m2 = sre2(&tensors).unwrap_or_else(|e| panic!("sre2: {e}"));
    let t_m2 = t0.elapsed().as_secs_f64();
    let t0 = Instant::now();
    let (m2_loc, frame) = sre2_local_min(&tensors, sweeps).unwrap_or_else(|e| panic!("sre2_local_min: {e}"));
    let t_loc = t0.elapsed().as_secs_f64();
    let t0 = Instant::now();
    let m2_box = sre2_box(&tensors, box_len).unwrap_or_else(|e| panic!("sre2_box: {e}"));
    let t_box = t0.elapsed().as_secs_f64();

    println!(
        "{{{source},\"n\":{n},\"chi\":{chi_max},\"m2\":{m2:.12},\"m2_loc\":{m2_loc:.12},\"m2_loc_sweeps\":{sweeps},\"m2_loc_frame\":{frame:?},\"m2_per_site\":{:.12},\"box\":{box_len},\"m2_box\":{m2_box:.12},\"price_bound_box\":{:.6e},\"seconds_m2\":{t_m2:.2},\"seconds_m2_loc\":{t_loc:.2},\"seconds_m2_box\":{t_box:.2},\"threads\":{}}}",
        m2 / n as f64,
        m2_box.exp2(),
        q8_mps::mps::threads()
    );
}
