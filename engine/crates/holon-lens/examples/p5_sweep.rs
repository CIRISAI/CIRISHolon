//! P-5's measurement, not just its assertion: the false-crystal rate of the ICE
//! criterion on synthetic dilute gases, with the diagnostics that say WHY it fires.
//!
//! Run: `cargo run -p holon-lens --example p5_sweep [n_draws]`

use holon_lens::classifier::{self, Phase};
use holon_lens::synthetic::{self, Spec};

fn main() {
    let n: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);
    let (mut ice, mut fired, mut vapor) = (0usize, 0usize, 0usize);
    let mut worst: Vec<(f64, f64, usize, u64)> = Vec::new();
    for s in 0..n {
        let mut sp = Spec::quench_like(300, vec![8, 8, 8, 8, 1, 1, 1, 1, 1, 1, 1, 1]);
        sp.seed = 1000 + s;
        let t = synthetic::vapor(sp);
        let r = classifier::classify(&t);
        if r.verdict.phase() == Some(Phase::Ice) {
            ice += 1;
        }
        if r.verdict.phase() == Some(Phase::Vapor) {
            vapor += 1;
        }
        if r.ice_criterion_fired {
            fired += 1;
        }
        worst.push((r.order, r.mobility, r.interior_atoms, 1000 + s));
    }
    worst.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    println!("# P-5 sweep: {n} synthetic dilute-gas trajectories, 12 atoms, 300 frames");
    println!("# verdict VAPOR            : {vapor}/{n}");
    println!("# verdict ICE              : {ice}/{n}   (staked: 0)");
    println!("# ICE CRITERION fired      : {fired}/{n}   (staked: 0 -- the unconditional reading)");
    let rate = fired as f64 / n as f64;
    let bound = if fired == 0 {
        1.0 - 0.05f64.powf(1.0 / n as f64)
    } else {
        f64::NAN
    };
    println!("# measured false-crystal rate: {rate:.4}");
    if fired == 0 {
        println!("# Clopper-Pearson 95% upper bound at 0/{n}: {bound:.4}");
    }
    println!("# highest five order readings (order, mobility, interior_ATOMS, seed):");
    for w in worst.iter().take(5) {
        println!("#   order {:.4}  mobility {:.4}  interior_atoms {:>3}  seed {}", w.0, w.1, w.2, w.3);
    }
}
