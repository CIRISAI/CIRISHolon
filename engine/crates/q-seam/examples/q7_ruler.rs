// G7-E7 / G7-E10: the free-fermion ruler against the many-body Lanczos, at every a.
use q_seam::hubbard::{free_chain_energy_per_site, free_reference, Hubbard};
use q_seam::lanczos::ground_state;
fn main() {
    println!("{:>3} {:>5} {:>16} {:>16} {:>11} {:>8} {:>28}", "N","a","E_free","E_lanczos","rel","gap","density profile");
    for &n in &[8usize, 10] {
        for &a in &[0.0f64, 0.5, 1.0, 2.0, 4.0, 8.0] {
            let v = Hubbard::trap(n, a);
            let fr = free_reference(n, 1.0, &v);
            let h = Hubbard::with_potential(n, 1.0, 0.0, &v);
            let g = ground_state(&h).unwrap();
            let rel = (fr.energy - g.energy).abs() / g.energy.abs().max(1.0);
            let prof: Vec<String> = fr.density.iter().map(|x| format!("{x:.2}")).collect();
            println!("{n:>3} {a:>5} {:>16.10} {:>16.10} {rel:>11.2e} {:>8.4} {:>28}",
                fr.energy, g.energy, fr.gap, prof.join(" "));
        }
    }
    // G7-E10: at a=0 the ruler must reproduce Q5's verified closed form.
    for &n in &[8usize, 10] {
        let fr = free_reference(n, 1.0, &vec![0.0; n]);
        let q5 = free_chain_energy_per_site(n) * n as f64;
        println!("G7-E10 N={n}: ruler {:.12} vs Q5 closed form {:.12} rel {:.2e}",
            fr.energy, q5, (fr.energy - q5).abs() / q5.abs());
    }
}
