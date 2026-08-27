//! Q10 §9 probe 4 / §0's named discriminator: SAME configuration, SAME chi, DIFFERENT INITIAL
//! STATE. "Trapping moves or vanishes; conditioning persists."
//!
//! §0 is already discharged on the conditioning branch (the SVD's absolute orthogonality
//! tolerance, repaired at 4bcf0d2), so this probe is no longer what decides the branch. It is
//! run because §9.4 requires it before freeze, and because discharging §0 by mechanism did NOT
//! positively exclude trapping — this is what does that, or fails to.
//!
//! CRITICALLY: every start below is a bond-dimension-1 PRODUCT state whose bonds grow through
//! ordinary two-site updates. Nothing here routes through `pad_to_chi`, whose zero-padding would
//! manufacture the rank-deficient bonds the conditioning branch blames (§0's own fence).
//!
//! All three starts sit in the same sector: N = sites particles, Sz = 0.
use q8_mps::dmrg::{self, Params, RefusalPolicy};
use q8_mps::mps::TensorSite;
use q8_mps::observables;

/// Product state from a per-spin-orbital occupation rule. Bond dimension 1 by construction.
fn product_state(sites: usize, occupied: impl Fn(usize, bool) -> bool) -> Vec<TensorSite> {
    (0..2 * sites)
        .map(|j| {
            let mut t = TensorSite::zeros(1, 1);
            t.set(if occupied(j / 2, j % 2 == 0) { 1 } else { 0 }, 0, 0, 1.0);
            t
        })
        .collect()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let sites: usize = a.get(1).map(|s| s.parse().unwrap()).unwrap_or(8);
    let u: f64 = a.get(2).map(|s| s.parse().unwrap()).unwrap_or(16.0);
    let chi: usize = a.get(3).map(|s| s.parse().unwrap()).unwrap_or(256);
    let p = Params { sites, t: 1.0, u, chi_max: chi, max_sweeps: 20, sweep_tol: 1e-10 };

    // neel: up on even chain sites, down on odd — the engine's own default start.
    // anti-neel: the opposite sublattice. Same sector, different basin if one exists.
    // doublon-hole: even sites doubly occupied, odd sites empty. Maximally unlike both.
    let starts: Vec<(&str, Vec<TensorSite>)> = vec![
        ("neel", product_state(sites, |cs, up| if up { cs % 2 == 0 } else { cs % 2 == 1 })),
        ("anti-neel", product_state(sites, |cs, up| if up { cs % 2 == 1 } else { cs % 2 == 0 })),
        ("doublon-hole", product_state(sites, |cs, _| cs % 2 == 0)),
    ];

    println!("=== N={sites} U={u} chi={chi} — §0 discriminator, {} starts ===", starts.len());
    let mut energies: Vec<(String, f64)> = Vec::new();
    for (name, t0) in starts {
        match dmrg::run_from(&p, RefusalPolicy::Typed, t0) {
            Ok(r) => {
                let n = observables::total_number(&r.tensors);
                let sz = observables::total_sz(&r.tensors);
                println!(
                    "{name:14} E={:.15} sweeps={}/{} converged={} N={n:.6} Sz={sz:.3e}",
                    r.energy_shifted, r.sweeps_used, p.max_sweeps, r.converged
                );
                energies.push((name.to_string(), r.energy_shifted));
            }
            Err(e) => println!("{name:14} REFUSED bond={} weight={:e}", e.bond, e.weight),
        }
    }
    if energies.len() > 1 {
        let lo = energies.iter().map(|x| x.1).fold(f64::INFINITY, f64::min);
        let hi = energies.iter().map(|x| x.1).fold(f64::NEG_INFINITY, f64::max);
        println!("SPREAD across starts = {:.6e}  (trapping => spread; conditioning => none)", hi - lo);
    }
}
