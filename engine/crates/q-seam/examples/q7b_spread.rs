//! THE MANDATORY SPREAD PRE-CHECK (house rule, after Q5 and Q7).
//!
//! Before any Q7b prereg exists: does the per-region honesty spread at fixed (N, U) exceed 4x?
//! The staked-margin arithmetic needs min E_r <= 0.5 and max E_r >= 2.0, so 4x is the floor.
//! If the step family fails this, it is rejected BEFORE a prereg is written.

use q_seam::chart::Chart;
use q_seam::hubbard::Hubbard;
use q_seam::lanczos::ground_state;
use q_seam::observables::ExactObservables;
use q_seam::region::{regions, RegionInstance};

/// Symmetric three-level box at N=10, targeting block densities [0, 2, 1, 2, 0]:
/// outer blocks pushed empty, next blocks deep wells pulled to double occupancy, centre at n=1.
/// The only reflection-symmetric half-filled pattern (with [2,0,1,0,2]) that contains all three
/// regimes at all -- N=8 admits none, which is arithmetic, not preference.
fn step10(v: f64) -> Vec<f64> { return q_seam::q7b_box(v); }
#[allow(dead_code)]
fn _unused(v: f64) -> Vec<f64> {
    vec![v, v, -v, -v, 0.0, 0.0, -v, -v, v, v]
}

fn main() {
    println!("{:>5} {:>5} {:>9} {:>9} {:>7}  {:<28} {:<26}", "V", "U", "min E_r", "max E_r", "SPREAD", "per-region E_r", "exact block density");
    let mut best = (0.0f64, 0.0, 0.0);
    for &v in &[2.0f64, 4.0, 8.0, 16.0] {
        for &u in &[1.0f64, 2.0, 4.0, 8.0] {
            let pot = step10(v);
            let h = Hubbard::with_potential(10, 1.0, u, &pot);
            let g = match ground_state(&h) { Some(g) => g, None => { println!("{v:>5} {u:>5}  lanczos gate failed"); continue; } };
            let gap = g.first_excited - g.energy;
            let o = ExactObservables::measure(&h, &g.vector);
            let c = match Chart::best_with(10, 1.0, u, &pot) { Some(c) => c, None => { println!("{v:>5} {u:>5}  SCF failed"); continue; } };
            let rs: Vec<RegionInstance> = regions(10).into_iter().enumerate()
                .map(|(k, b)| RegionInstance::measure(10, u, v, k, b, &o, &c)).collect();
            let e: Vec<f64> = rs.iter().map(|r| r.e_r).collect();
            let (lo, hi) = (e.iter().cloned().fold(f64::INFINITY, f64::min), e.iter().cloned().fold(0.0, f64::max));
            let spread = if lo > 0.0 { hi / lo } else { f64::INFINITY };
            if spread > best.2 && hi > 1.0 { best = (v, u, spread); }
            let es: Vec<String> = e.iter().map(|x| format!("{x:.2}")).collect();
            let dens: Vec<String> = regions(10).iter()
                .map(|b| format!("{:.2}", (o.density[b[0]] + o.density[b[1]]) / 2.0)).collect();
            println!("{v:>5} {u:>5} {lo:>9.3} {hi:>9.3} {spread:>7.2}  {:<28} {:<26} gap={gap:.3}",
                es.join(" "), dens.join(" "));
        }
    }
    println!("\nBEST: V={} U={} spread={:.2}x   (house rule needs > 4x, else the family is REJECTED)",
        best.0, best.1, best.2);
}
