use q_seam::hubbard::{dimer_double_occupancy, Hubbard};
use q_seam::lanczos::ground_state;
use q_seam::observables::ExactObservables;
fn main() {
    println!("{:>3} {:>5} {:>11} {:>11} {:>11} {:>11} {:>10} {:>10} {:>10}",
        "N","U","max|n-1|","max|m|","max|nup-.5|","refl","<S2>","d/site","D_bool");
    for &n in &[2usize, 4, 6, 8] {
        for &u in &[0.0, 1.0, 4.0, 16.0] {
            let h = Hubbard::new(n, 1.0, u);
            let g = ground_state(&h).unwrap();
            let o = ExactObservables::measure(&h, &g.vector);
            let dn1 = o.density.iter().map(|x| (x-1.0).abs()).fold(0.0, f64::max);
            let dm  = o.magnetization.iter().map(|x| x.abs()).fold(0.0, f64::max);
            let dh  = o.occupation[0].iter().chain(o.occupation[1].iter()).map(|x| (x-0.5).abs()).fold(0.0, f64::max);
            let refl = (0..n).map(|i| (o.density[i]-o.density[n-1-i]).abs()).fold(0.0, f64::max);
            let extra = if n==2 { format!(" [analytic d/site {:.6}]", dimer_double_occupancy(1.0,u)/2.0) } else { String::new() };
            println!("{n:>3} {u:>5} {dn1:>11.3e} {dm:>11.3e} {dh:>11.3e} {refl:>11.3e} {:>10.3e} {:>10.6} {:>10.6}{extra}",
                o.s_squared, o.double_occ_mean, o.d_bool);
        }
    }
}
