use q_seam::dense::jacobi;
use q_seam::hubbard::{free_chain_energy_per_site, Hubbard};
use q_seam::lanczos::ground_state;

fn main() {
    println!("{:>3} {:>5} {:>6} {:>26} {:>26} {:>12} {:>12} {:>12}",
        "N","U","dim","dense","lanczos","rel(d,l)","rel vs exact(d)","rel vs exact(l)");
    for &n in &[2usize, 4, 6] {
        for &u in &[0.0, 1.0, 4.0, 16.0] {
            let h = Hubbard::new(n, 1.0, u);
            let d = h.dim();
            let eig = jacobi(h.to_dense(), d);
            let g = ground_state(&h).unwrap();
            let rel = (eig.values[0] - g.energy).abs() / g.energy.abs().max(1.0);
            let (ed, el) = if u == 0.0 {
                let exact = free_chain_energy_per_site(n) * n as f64;
                (((eig.values[0]-exact).abs()/exact.abs()), ((g.energy-exact).abs()/exact.abs()))
            } else if n == 2 {
                let exact = q_seam::hubbard::dimer_energy(1.0, u);
                (((eig.values[0]-exact).abs()/exact.abs()), ((g.energy-exact).abs()/exact.abs()))
            } else { (f64::NAN, f64::NAN) };
            println!("{n:>3} {u:>5} {d:>6} {:>26.18} {:>26.18} {rel:>12.3e} {ed:>15.3e} {el:>15.3e}",
                eig.values[0], g.energy);
        }
    }
}
