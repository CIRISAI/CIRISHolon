//! GF2a's exact referee on the colour lanes, host or device: sector energies, the derived baryon
//! mass and the finite-volume two-baryon energy, with the door's price and the device class.
//!
//!   qcd2_lanes [--device cpu|gpu|both] [--threads T] [--max-sub M] [--reserve-mib R] N X [N X ...]
//!
//! `both` runs every sector on both arms and prints their disagreement (expected: none, to the
//! bit) and the device-resident sigma rate. `--max-sub` is the Davidson subspace bound the
//! door prices (default 48): a space refused at 48 is admitted at 12 with the same driver.
use std::time::Instant;

use holon_chem::budget::{price_determinant_with, DAVIDSON_SUBSPACE_MAX};
use holon_chem::fci::davidson_budget;
use holon_chem::lanes::{lane_threads, solve_lanes_with, LaneSigma, LaneSolution, LaneTables};
use holon_chem::qcd2::Qcd2;
use holon_gpu::{GpuLaneSigma, GpuSigmaProvider};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let mut device = "cpu".to_string();
    let mut threads = lane_threads();
    let mut reserve_mib = 1024u64;
    let mut max_sub = DAVIDSON_SUBSPACE_MAX;
    let mut pairs: Vec<f64> = Vec::new();
    let mut i = 0;
    while i < a.len() {
        match a[i].as_str() {
            "--device" => {
                device = a[i + 1].clone();
                i += 2;
            }
            "--threads" => {
                threads = a[i + 1].parse().expect("--threads T");
                i += 2;
            }
            "--max-sub" => {
                max_sub = a[i + 1].parse().expect("--max-sub M");
                i += 2;
            }
            "--reserve-mib" => {
                reserve_mib = a[i + 1].parse().expect("--reserve-mib R");
                i += 2;
            }
            v => {
                pairs.push(v.parse().expect("N X pairs"));
                i += 1;
            }
        }
    }
    assert!(!pairs.is_empty() && pairs.len() % 2 == 0, "pairs of N X");
    let (cpu, gpu) = match device.as_str() {
        "cpu" => (true, false),
        "gpu" => (false, true),
        "both" => (true, true),
        d => panic!("--device {d}: cpu | gpu | both"),
    };
    let provider = if gpu { Some(GpuSigmaProvider::new(0).expect("no CUDA device (D4: no fallback)")) } else { None };
    println!("# GF2a exact referee on colour lanes (Cartan-neutral block); W-units; M_B/g = (E1-E0)/(2 sqrt x); host shards = {threads}; subspace bound = {max_sub}");
    for pair in pairs.chunks(2) {
        let n = pair[0] as usize;
        let x = pair[1];
        let q = Qcd2::new(n, x);
        let ham = q.lane_hamiltonian();
        let mut es_cpu = Vec::new();
        let mut es_gpu = Vec::new();
        for b in 0..=2 {
            let space = q.lane_space(b);
            let price = price_determinant_with(space.n_det, max_sub);
            let n_q = q.quarks(b);
            // the tables are built ONCE per sector; the device uploads them, the host owns them
            let tables = LaneTables::build(&space, &ham);
            let line = |arm: &str, s: &LaneSolution, secs: f64| {
                println!(
                    "N={n:2} x={x:4.1} B={b} n_q={n_q:2} n_det={:9} price={:.2e}B {arm:3} E0={:+.10}  iters={} resid={:.1e} margin={:+.3e} exit={:?}  {:.1}s",
                    space.n_det, price.bytes as f64, s.energy, s.iters, s.residual, s.variational_margin, s.exit, secs
                );
            };
            let mut dev_op = match &provider {
                Some(p) => match GpuLaneSigma::new(p.context(), &tables, reserve_mib) {
                    Ok(op) => Some(op),
                    Err(e) => {
                        println!("N={n:2} x={x:4.1} B={b} gpu REFUSED: {e:?}");
                        None
                    }
                },
                None => None,
            };
            if cpu {
                let t0 = Instant::now();
                let mut op = LaneSigma { tables, threads };
                let diag = op.diagonal();
                let s = solve_lanes_with(&mut op, &diag, None, davidson_budget(), max_sub);
                line("cpu", &s, t0.elapsed().as_secs_f64());
                es_cpu.push(s.energy);
            }
            if let Some(op) = dev_op.as_mut() {
                let t0 = Instant::now();
                let diag = op.diagonal().expect("diagonal");
                let s = solve_lanes_with(op, &diag, None, davidson_budget(), max_sub);
                line("gpu", &s, t0.elapsed().as_secs_f64());
                let per = op.seconds_per_sigma_resident(5).expect("timing");
                println!("N={n:2} x={x:4.1} B={b} gpu resident sigma {:.4} s ({:.1} sigma/s)", per, 1.0 / per);
                if cpu {
                    let ec = es_cpu[b as usize];
                    println!(
                        "N={n:2} x={x:4.1} B={b} cpu-vs-gpu |dE|={:.3e} same_bits={}",
                        (ec - s.energy).abs(),
                        ec.to_bits() == s.energy.to_bits()
                    );
                }
                es_gpu.push(s.energy);
            }
        }
        for (arm, es) in [("cpu", &es_cpu), ("gpu", &es_gpu)] {
            if es.len() == 3 {
                let m_b = Qcd2::baryon_mass(es[0], es[1], x);
                let u_bb = es[2] - 2.0 * es[1] + es[0];
                println!(
                    "N={n:2} x={x:4.1} {arm}  M_B/g={m_b:.6}  U_BB={u_bb:+.6e}  (E1-E0={:+.6}, E2-E1={:+.6})",
                    es[1] - es[0],
                    es[2] - es[1]
                );
            }
        }
    }
}
