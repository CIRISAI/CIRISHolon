//! Q10 §9 pre-freeze probes 1 and 2, in one instrument.
//!
//! Probe 1 — THE FENCE MUST VARY across the intended sweep. `Q10_PREREG.md` §9.1: if the fence
//! does not vary, the family does not pose the question and Q10 is VOID-not-killed. Q7 VOIDed
//! at its family gate because nobody measured the spread first.
//!
//! Probe 2 — THE ANCHORS MUST BE VIOLABLE: at least one configuration where an anchor fires.
//! An anchor that cannot fail on this family proves nothing here.
//!
//! Reports the readings; it adjudicates nothing. Usage: `q10_probe <N> <U> <chi>`.
use q8_mps::dmrg::{self, Params, RefusalPolicy};
use q8_mps::observables;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let sites: usize = a.get(1).map(|s| s.parse().unwrap()).unwrap_or(8);
    let u: f64 = a.get(2).map(|s| s.parse().unwrap()).unwrap_or(16.0);
    let chi: usize = a.get(3).map(|s| s.parse().unwrap()).unwrap_or(256);

    let p = Params { sites, t: 1.0, u, chi_max: chi, max_sweeps: 20, sweep_tol: 1e-10 };
    let r = match dmrg::run(&p, RefusalPolicy::Typed) {
        Ok(r) => r,
        Err(e) => {
            println!("N={sites} U={u} chi={chi} REFUSED bond={} weight={:e}", e.bond, e.weight);
            return;
        }
    };

    // THE FENCE (§3a): min over bonds of the kept-spectrum floor.
    let fence = r.spectrum_floor.iter().copied().fold(f64::INFINITY, f64::min);
    let fence_max = r.spectrum_floor.iter().copied().fold(0.0f64, f64::max);

    // THE FOUR ANCHORS (§3b), each as its own distance from the theorem value.
    //
    // CONVENTIONS TAKEN FROM `tests/full_grid_gates.rs`, NOT GUESSED. `p.sites` counts CHAIN
    // SITES and the MPS carries `2*sites` spin-orbital tensors. Particle-hole pins <n_jσ> to 1/2
    // PER SPIN-ORBITAL and must be read that way — `occupation_profile` sums both spins per
    // chain site and pins to 1, and the harness records that an earlier version of this very
    // check applied the 1/2 to the per-site total. A first draft of this probe repeated that
    // error and also passed `sites/2` to `magnetization_profile`, which takes chain sites.
    let norm = observables::norm_squared(&r.tensors);
    let ph = (0..2 * sites)
        .map(|j| (observables::expectation(&r.tensors, &[(j, q8_mps::ops::N2)]) / norm - 0.5).abs())
        .fold(0.0f64, f64::max);
    let m = observables::magnetization_profile(&r.tensors, sites);
    let mag = m.iter().map(|x| x.abs()).fold(0.0f64, f64::max);
    let sz = observables::total_sz(&r.tensors).abs();
    let sz2 = observables::total_sz_squared(&r.tensors).abs();
    let ntot = (observables::total_number(&r.tensors) - sites as f64).abs();

    println!(
        "N={sites} U={u} chi={chi} sweeps={}/{} converged={} \
         FENCE_min={fence:.6e} FENCE_max={fence_max:.6e} \
         anchor_ph={ph:.6e} anchor_mag={mag:.6e} anchor_sz={sz:.6e} anchor_sz2={sz2:.6e} anchor_N={ntot:.6e}",
        r.sweeps_used, p.max_sweeps, r.converged
    );
}
