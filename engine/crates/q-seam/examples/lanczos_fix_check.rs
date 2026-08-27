// Does the cadence change COST and not RESULTS? Bit-level comparison against pinned values
// captured from the pre-fix build, plus timing.
use q_seam::hubbard::Hubbard;
use q_seam::lanczos::{ground_state, CHECK_EVERY};
use std::time::Instant;
fn main() {
    println!("CHECK_EVERY = {CHECK_EVERY}");
    println!("{:>3} {:>5} {:>7} {:>24} {:>7} {:>11} {:>11} {:>10}",
        "N","U","dim","energy (all 17 digits)","iters","residual","overlap","secs");
    for &(n, u) in &[(8usize,0.0),(8,16.0),(10,0.0),(10,4.0),(10,16.0)] {
        let t0 = Instant::now();
        let h = Hubbard::new(n, 1.0, u);
        let g = ground_state(&h).unwrap();
        println!("{n:>3} {u:>5} {:>7} {:>24.17} {:>7} {:>11.4e} {:>11.4e} {:>10.3?}",
            h.dim(), g.energy, g.iterations, g.residual, g.overlap_start, t0.elapsed());
    }
    // Q7b family too, since that is the live reference shape.
    for &(v, u) in &[(4.0f64, 1.0f64), (8.0, 16.0)] {
        let t0 = Instant::now();
        let h = Hubbard::with_potential(10, 1.0, u, &q_seam::q7b_box(v));
        let g = ground_state(&h).unwrap();
        println!("box V={v} U={u}: E = {:.17}  iters={} secs={:.3?}", g.energy, g.iterations, t0.elapsed());
    }
}
