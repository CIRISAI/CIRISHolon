//! GF2a on the engine's DMRG: one sector ground state per invocation, JSON on stdout.
//!
//!   qcd2_dmrg --n 24 --x 9 --b 1 --chi 64 [--sweeps 120] [--tol 1e-9]
use q8_mps::qcd2::Qcd2;
use std::time::Instant;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let get = |k: &str| a.iter().position(|s| s == k).map(|i| a[i + 1].clone());
    let n: usize = get("--n").expect("--n").parse().unwrap();
    let x: f64 = get("--x").expect("--x").parse().unwrap();
    let b: i32 = get("--b").expect("--b").parse().unwrap();
    let chi: usize = get("--chi").map_or(64, |v| v.parse().unwrap());
    let sweeps: usize = get("--sweeps").map_or(120, |v| v.parse().unwrap());
    let tol: f64 = get("--tol").map_or(1e-9, |v| v.parse().unwrap());
    let q = Qcd2::new(n, x);
    let t0 = Instant::now();
    let r = q.ground_energy(b, chi, sweeps, tol).expect("sweep refused");
    let max_dw = r.discarded_weight.iter().cloned().fold(0.0f64, f64::max);
    println!(
        "{{\"n\":{n},\"x\":{x},\"b\":{b},\"n_q\":{},\"chi\":{chi},\"energy\":{:.12},\"sweeps\":{},\"converged\":{},\"worst_residual\":{:.3e},\"max_discarded\":{:.3e},\"max_bond\":{},\"seconds\":{:.1}}}",
        q.quarks(b), r.energy, r.sweeps_used, r.converged, r.worst_lanczos_residual, max_dw,
        r.bond_dims.iter().cloned().max().unwrap_or(0), t0.elapsed().as_secs_f64()
    );
}
