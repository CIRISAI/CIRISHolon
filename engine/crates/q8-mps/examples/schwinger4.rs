//! SCHWINGER-4 on the engine: one ground state per invocation, JSON on stdout.
//!
//! ```text
//! schwinger4 --n 84 --x 9 --chi 40 --charges 28:1,30:-1,54:1,56:-1 [--coulomb-off] [--sweeps 60] [--tol 1e-9]
//! schwinger4 --gauge
//! ```
//!
//! `--gauge` is plant (i) of the freeze on THIS driver: the three N = 12, x = 4 configurations
//! against the exact referee (SCHWINGER-1's ED, extended by the same static term; the numbers
//! below were printed by `schwinger4.py gauge` and are the ED side of that run, not the DMRG
//! side), at the freeze's 1e-6 bar; exit code 1 on any miss.

use q8_mps::schwinger::{Mutation, Schwinger};
use std::time::Instant;

/// ED referees at N = 12, x = 4 (Q = 0 sector, no penalty): the vacuum, one static pair at
/// (4, 6), two pairs at (4, 6) and (8, 10). Printed by the Python gauge on 2026-09-02.
const REFEREE: [(&str, &[(usize, i32)], f64); 3] = [
    ("E0", &[], -26.1743157050),
    ("E1(p=4,s=2)", &[(4, 1), (6, -1)], -24.3677609251),
    ("E2(d=2)", &[(4, 1), (6, -1), (8, 1), (10, -1)], -22.5337644964),
];

fn parse_charges(s: &str) -> Vec<(usize, i32)> {
    if s.trim().is_empty() {
        return Vec::new();
    }
    s.split(',')
        .map(|t| {
            let (p, q) = t.split_once(':').expect("charges are site:charge pairs");
            (p.trim().parse().expect("site"), q.trim().parse().expect("charge"))
        })
        .collect()
}

fn gauge() -> i32 {
    let mut worst = 0.0f64;
    let mut es = Vec::new();
    for (name, charges, e_ed) in REFEREE {
        let s = Schwinger::new(12, 4.0, charges.to_vec());
        let (e, res) = s.ground_energy(48, 60, 1e-11).expect("sweep");
        let diff = (e - e_ed).abs();
        worst = worst.max(diff);
        es.push(e);
        println!(
            "plant (i) {name:12} ED {e_ed:.10}  engine {e:.10}  |diff| {diff:.2e}  sweeps {} residual {:.1e}",
            res.sweeps_used, res.worst_lanczos_residual
        );
    }
    let carrier = es[1] - es[0];
    let ok = worst <= 1e-6 && carrier > 0.1;
    println!("carrier: E1 - E0 = {carrier:.6}  ({})", if carrier > 0.1 { "nonzero" } else { "VACUOUS" });
    println!("gauge verdict: {} (worst |diff| {worst:.2e} vs 1e-6)", if ok { "PASS" } else { "FAIL" });
    i32::from(!ok)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--gauge") {
        std::process::exit(gauge());
    }
    let mut n = 0usize;
    let mut x = 0.0f64;
    let mut chi = 40usize;
    let mut sweeps = 60usize;
    let mut tol = 1e-9f64;
    let mut charges = Vec::new();
    let mut mutation = Mutation::None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--n" => { n = args[i + 1].parse().expect("--n"); i += 1; }
            "--x" => { x = args[i + 1].parse().expect("--x"); i += 1; }
            "--chi" => { chi = args[i + 1].parse().expect("--chi"); i += 1; }
            "--sweeps" => { sweeps = args[i + 1].parse().expect("--sweeps"); i += 1; }
            "--tol" => { tol = args[i + 1].parse().expect("--tol"); i += 1; }
            "--charges" => { charges = parse_charges(&args[i + 1]); i += 1; }
            "--coulomb-off" => mutation = Mutation::CoulombOff,
            other => panic!("unknown argument {other}"),
        }
        i += 1;
    }
    assert!(n > 0 && x > 0.0, "--n and --x are required");
    let s = Schwinger::new(n, x, charges.clone()).with_mutation(mutation);
    let t0 = Instant::now();
    let (e, res) = s.ground_energy(chi, sweeps, tol).expect("sweep refused");
    let max_dw = res.discarded_weight.iter().cloned().fold(0.0f64, f64::max);
    println!(
        "{{\"n\":{n},\"x\":{x},\"chi\":{chi},\"charges\":\"{}\",\"mutation\":\"{:?}\",\"energy\":{e:.12},\"constant\":{:.6},\"sweeps\":{},\"converged\":{},\"worst_residual\":{:.3e},\"max_discarded\":{:.3e},\"seconds\":{:.1}}}",
        charges.iter().map(|(p, q)| format!("{p}:{q}")).collect::<Vec<_>>().join(","),
        mutation,
        s.constant(),
        res.sweeps_used,
        res.converged,
        res.worst_lanczos_residual,
        max_dw,
        t0.elapsed().as_secs_f64()
    );
}
