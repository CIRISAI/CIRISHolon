use q_seam::chart::Chart;
use q_seam::hubbard::Hubbard;
use q_seam::lanczos::ground_state;
use q_seam::observables::ExactObservables;
fn main() {
    println!("{:>3} {:>6} {:>9} {:>10} {:>10} {:>10} {:>10} {:>9} {:>9}",
        "N","U","guess","E_MF/N","E_ex/N","dE(var)","max|m_MF|","idem","iters");
    for &n in &[4usize, 8] {
        for &u in &[0.0, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0] {
            let h = Hubbard::new(n, 1.0, u);
            let g = ground_state(&h).unwrap();
            let o = ExactObservables::measure(&h, &g.vector);
            let c = Chart::best(n, 1.0, u).unwrap();
            let mm = c.magnetization().iter().map(|x| x.abs()).fold(0.0, f64::max);
            let var = c.energy - g.energy;
            println!("{n:>3} {u:>6} {:>9?} {:>10.6} {:>10.6} {:>10.3e} {mm:>10.6} {:>9.2e} {:>9}",
                c.guess, c.energy_per_site(), g.energy/n as f64, var, c.idempotency, c.iterations);
            let _ = o;
        }
    }
}
