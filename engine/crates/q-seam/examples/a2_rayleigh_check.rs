use q_seam::dense::jacobi;
use q_seam::hubbard::{free_chain_energy_per_site, Hubbard};
use q_seam::lanczos::ground_state;

fn main() {
    println!("{:>3} {:>5} {:>6} {:>12} {:>12} {:>12}", "N","U","dim","rel(RQ,lanc)","rel(dense,l)","RQ vs exact");
    for &n in &[2usize, 4, 6] {
        for &u in &[0.0, 0.125, 1.0, 4.0, 16.0] {
            let h = Hubbard::new(n, 1.0, u);
            let d = h.dim();
            let eig = jacobi(h.to_dense(), d);
            let v = &eig.vectors[0];
            let mut hv = vec![0.0; d];
            h.apply(v, &mut hv);
            let num: f64 = hv.iter().zip(v).map(|(a,b)| a*b).sum();
            let den: f64 = v.iter().map(|x| x*x).sum();
            let rq = num/den;
            let g = ground_state(&h).unwrap();
            let rel_rq = (rq - g.energy).abs()/g.energy.abs().max(1.0);
            let rel_d  = (eig.values[0] - g.energy).abs()/g.energy.abs().max(1.0);
            let ex = if u==0.0 { (rq - free_chain_energy_per_site(n)*n as f64).abs()/(free_chain_energy_per_site(n)*n as f64).abs() }
                     else if n==2 { (rq - q_seam::hubbard::dimer_energy(1.0,u)).abs()/q_seam::hubbard::dimer_energy(1.0,u).abs() }
                     else { f64::NAN };
            println!("{n:>3} {u:>5} {d:>6} {rel_rq:>12.3e} {rel_d:>12.3e} {ex:>12.3e}");
        }
    }
}
