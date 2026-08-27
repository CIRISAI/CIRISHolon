use q_seam::hubbard::Hubbard;
use q_seam::lanczos::ground_state;
use q_seam::share::measure;
use std::time::Instant;
fn main() {
    println!("{:>3} {:>6} {:>12} {:>12} {:>12} {:>7} {:>7} {:>10} {:>10}",
        "N","U","B4_mean","B4_max","|IC3|max","ipfOnly","failed","minCell","secs");
    for &n in &[4usize, 6] {
        for &u in &[0.0, 0.5, 2.0, 16.0] {
            let t0 = Instant::now();
            let h = Hubbard::new(n, 1.0, u);
            let g = ground_state(&h).unwrap();
            let r = measure(&h, &g.vector);
            println!("{n:>3} {u:>6} {:>12.4e} {:>12.4e} {:>12.4e} {:>7} {:>7} {:>10.2e} {:>10.2?}",
                r.b4_mean, r.b4_max, r.ic3_max, r.ipf_only, r.failed, r.min_cell, t0.elapsed());
        }
    }
}
