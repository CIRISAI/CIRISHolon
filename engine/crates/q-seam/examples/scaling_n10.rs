use q_seam::hubbard::Hubbard;
use q_seam::lanczos::ground_state;
use std::time::Instant;
fn main() {
    for &u in &[0.0f64, 4.0, 16.0] {
        let t0 = Instant::now();
        let h = Hubbard::new(10, 1.0, u);
        let build = t0.elapsed();
        let t1 = Instant::now();
        let g = ground_state(&h).unwrap();
        println!("N=10 U={u:>4}: dim={} build={:?} solve={:?} iters={} resid={:.3e} overlap={:.3e} E={:.12} gapEst={:.6}",
            h.dim(), build, t1.elapsed(), g.iterations, g.residual, g.overlap_start, g.energy, g.first_excited-g.energy);
    }
}
