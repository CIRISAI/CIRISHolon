//! GF2a on the engine's DMRG: one sector per invocation, the staked χ-ladder in ONE
//! process (χ₁ cold, then padded to χ₂ as a warm start), JSON on stdout.
//!
//!   qcd2_dmrg --n 24 --x 9 --b 1 --chi 40,64 [--sweeps 120] [--rtol 1e-9]
//!   qcd2_dmrg --n 8 --x 4 --b 0 --chi 32,64,128,256 --sym        (E7, amendment A1)
//!   qcd2_dmrg --n 8 --x 4 --b 0 --chi 64 --mutant                (plant iv: labels ignored)
//!
//! The sweep tolerance is RELATIVE: two sweeps run at tolerance zero to learn the energy
//! scale, then the run continues from those tensors with `rtol · max(1, |E|)`. An absolute
//! 1e-9 against a sector energy of 10³ asked for a relative 1e-12, below what a two-site
//! sweep resolves, and ran every point to its sweep cap.
use q8_mps::dmrg::{dmrg_sweep, DmrgConfig, RefusalPolicy};
use q8_mps::mps::pad_to_chi;
use q8_mps::qcd2::Qcd2;
use std::time::Instant;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let get = |k: &str| a.iter().position(|s| s == k).map(|i| a[i + 1].clone());
    let n: usize = get("--n").expect("--n").parse().unwrap();
    let x: f64 = get("--x").expect("--x").parse().unwrap();
    let b: i32 = get("--b").expect("--b").parse().unwrap();
    let chis: Vec<usize> = get("--chi").map_or(vec![40, 64], |v| v.split(',').map(|c| c.parse().unwrap()).collect());
    let sweeps: usize = get("--sweeps").map_or(120, |v| v.parse().unwrap());
    let rtol: f64 = get("--rtol").map_or(1e-9, |v| v.parse().unwrap());
    let sym = a.iter().any(|s| s == "--sym");
    let mutant = a.iter().any(|s| s == "--mutant");
    let q = Qcd2::new(n, x);
    let n_q = q.quarks(b);
    if sym || mutant {
        // E7 (amendment A1): the symmetric sweep on the unpenalised Hamiltonian, the χ-ladder
        // continuing from the previous rung's tensors and labels (no padding: the two-site
        // update grows the bond itself); `--mutant` is plant (iv), labels ignored.
        let t0 = Instant::now();
        let sector = q.sector(n_q).expect("a Cartan-neutral sector");
        let mut state: Option<(Vec<q8_mps::mps::TensorSite>, q8_mps::symmetric::Labels)> = Some(q8_mps::symmetric::random_start(&sector, 256, 7));
        let occ: Vec<bool> = Vec::new();
        let mut rungs = Vec::new();
        for &chi in &chis {
            let t1 = Instant::now();
            let mut cfg = q8_mps::symmetric::SymConfig::amendment(chi, sweeps);
            cfg.ignore_labels = mutant;
            match q.ground_energy_sym_from(&occ, n_q, &cfg, state.take()) {
                Ok((r, labels)) => {
                    let max_dw = r.discarded_weight.iter().cloned().fold(0.0f64, f64::max);
                    rungs.push(format!(
                        "{{\"chi\":{chi},\"energy\":{:.12},\"sweeps\":{},\"lanczos_iterations\":{},\"converged\":{},\"worst_residual\":{:.3e},\"max_discarded\":{:.3e},\"max_bond\":{},\"seconds\":{:.1}}}",
                        r.energy, r.sweeps_used, r.lanczos_iterations_total, r.converged, r.worst_lanczos_residual, max_dw,
                        r.bond_dims.iter().cloned().max().unwrap_or(0), t1.elapsed().as_secs_f64()
                    ));
                    state = Some((r.tensors, labels));
                }
                Err(e) => {
                    rungs.push(format!("{{\"chi\":{chi},\"refused\":\"{e}\"}}"));
                    break;
                }
            }
        }
        println!(
            "{{\"n\":{n},\"x\":{x},\"b\":{b},\"n_q\":{n_q},\"arm\":\"{}\",\"threads\":{},\"rungs\":[{}],\"seconds\":{:.1}}}",
            if mutant { "mutant-labels-ignored" } else { "symmetric" }, q8_mps::mps::threads(), rungs.join(","), t0.elapsed().as_secs_f64()
        );
        return;
    }
    let mpo = q.mpo(n_q);
    let constant = q.lam * (n_q as f64) * (n_q as f64);
    let mut tensors = q.product_start(n_q);
    let mut rungs = Vec::new();
    let t0 = Instant::now();
    for &chi in &chis {
        let t1 = Instant::now();
        tensors = pad_to_chi(&tensors, chi);
        let probe = DmrgConfig { chi_max: chi, max_sweeps: 2, sweep_tol: 0.0, policy: RefusalPolicy::Silent };
        let r0 = dmrg_sweep(&mpo, tensors, &probe).expect("probe sweep refused");
        let tol = rtol * (1.0f64).max((r0.energy + constant).abs());
        let cfg = DmrgConfig { chi_max: chi, max_sweeps: sweeps, sweep_tol: tol, policy: RefusalPolicy::Silent };
        let r = dmrg_sweep(&mpo, r0.tensors, &cfg).expect("sweep refused");
        let max_dw = r.discarded_weight.iter().cloned().fold(0.0f64, f64::max);
        rungs.push(format!(
            "{{\"chi\":{chi},\"energy\":{:.12},\"sweeps\":{},\"lanczos_iterations\":{},\"converged\":{},\"worst_residual\":{:.3e},\"max_discarded\":{:.3e},\"max_bond\":{},\"tol\":{:.3e},\"seconds\":{:.1}}}",
            r.energy + constant, r.sweeps_used + 2, r.lanczos_iterations_total + r0.lanczos_iterations_total, r.converged,
            r.worst_lanczos_residual, max_dw, r.bond_dims.iter().cloned().max().unwrap_or(0), tol, t1.elapsed().as_secs_f64()
        ));
        tensors = r.tensors;
    }
    println!(
        "{{\"n\":{n},\"x\":{x},\"b\":{b},\"n_q\":{n_q},\"threads\":{},\"rungs\":[{}],\"seconds\":{:.1}}}",
        q8_mps::mps::threads(), rungs.join(","), t0.elapsed().as_secs_f64()
    );
}
