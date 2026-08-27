use q_seam::audit::{Mp2Audit, Stability};
use q_seam::chart::Chart;
use q_seam::hubbard::{free_chain_gap, Hubbard};
use q_seam::lanczos::ground_state;
use q_seam::observables::ExactObservables;
fn main() {
    println!("{:>3} {:>6} | {:>9} {:>9} | {:>9} {:>9} | {:>8} {:>8} {:>5} | {:>7}",
        "N","U","E2/site","dE_true","Dhat","D_true","lam_min","lam_proj","null","Delta");
    for &n in &[4usize, 8] {
        for &u in &[0.0, 0.25, 0.5, 0.75, 1.0, 1.5, 2.0, 4.0, 16.0] {
            let h = Hubbard::new(n, 1.0, u);
            let g = ground_state(&h).unwrap();
            let o = ExactObservables::measure(&h, &g.vector);
            let c = Chart::best(n, 1.0, u).unwrap();
            let a = Mp2Audit::of(&c);
            let s = Stability::of(&c);
            let de = (c.energy - g.energy)/n as f64;
            println!("{n:>3} {u:>6} | {:>9.5} {de:>9.5} | {:>9.5} {:>9.5} | {:>8.4} {:>8.4} {:>5} | {:>7.4}",
                a.energy_per_site, a.d_bool, o.d_bool, s.lambda_min, s.lambda_min_projected, s.null_modes, free_chain_gap(n));
        }
    }
}
