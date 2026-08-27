//! DIAGNOSTIC ONLY — not a gate, not staked. `Q9`'s two-solve-rule probe: does chi-warm-starting
//! (converge at small chi, zero-pad to a larger chi, sweep again, instead of restarting from the
//! pinned product state at each chi) escape the stagnation the cold-start chi ladder showed at
//! N=8, U=16 (the worst stuck config: chi=32 converged to dE=8.59e-7, then chi=64/128/256 —
//! COLD-started fresh from the product state each time — sat at dE~1.5e-2 to 7.4e-3 despite eps
//! shrinking to ~1e-17, per `output/q8_mps/g4_certificate.KILLED_RUN.log`)?
//!
//! Run BEFORE freezing Q9's prereg, per the two-solve rule — probe the remedy before staking on
//! it working.

use q8_mps::dmrg::{self, Params, RefusalPolicy};
use std::time::Instant;

fn main() {
    let sites = 8;
    let u = 16.0;
    let t = 1.0;

    let h = q_seam::hubbard::Hubbard::new(sites, t, u);
    let exact = q_seam::lanczos::ground_state(&h).expect("q-seam Lanczos failed");
    println!("N={sites} U={u}: exact E={:.10}", exact.energy);

    println!("\n=== COLD-START chi ladder (for reference; already measured in g4's killed run) ===");
    println!("chi=32:  dE=8.59e-7  (converged)");
    println!("chi=64:  dE=1.49e-2  (NOT converged, eps=4.27e-12)");
    println!("chi=128: dE=1.52e-2  (NOT converged, eps=2.59e-17)");
    println!("chi=256: dE=7.40e-3  (NOT converged, eps~0)");

    println!("\n=== WARM-START chi ladder: converge small, zero-pad, sweep again ===");
    let ladder = [32usize, 64, 128, 256];
    let mu = u / 2.0;
    let n_target = sites as f64;

    let mut tensors = q8_mps::mps::initial_state(sites);
    for &chi in &ladder {
        let t0 = Instant::now();
        let padded = q8_mps::mps::pad_to_chi(&tensors, chi);
        let p = Params { sites, t, u, chi_max: chi, max_sweeps: 20, sweep_tol: 1e-10 };
        let outcome = dmrg::run_from(&p, RefusalPolicy::Typed, padded);
        match outcome {
            Ok(r) => {
                let unshifted = r.energy_shifted + mu * n_target;
                let d_energy = (unshifted - exact.energy).abs();
                let eps: f64 = r.discarded_weight.iter().sum();
                println!(
                    "chi={chi:3}: dE={d_energy:e}  eps={eps:e}  sweeps={} converged={}  ({:.1?})",
                    r.sweeps_used, r.converged, t0.elapsed()
                );
                tensors = r.tensors;
            }
            Err(refusal) => {
                println!(
                    "chi={chi:3}: REFUSED bond={} weight={:e}  ({:.1?})",
                    refusal.bond, refusal.weight, t0.elapsed()
                );
                break;
            }
        }
    }
}
