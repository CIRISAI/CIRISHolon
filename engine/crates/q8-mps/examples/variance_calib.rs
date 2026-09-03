//! CALIBRATE the two-site variance against the exact one on the campaign's OWN states —
//! the rungs banked as checkpoints by the A2/A3 exam. Where the exact variance fits its
//! lease both are printed with their ratio; above the lease only the two-site one exists,
//! which is the case the volume ladder lives in.
//!
//!   variance_calib <N> <x> <B> <ckpt-dir> <chi> [chi...]
use q8_mps::qcd2::Qcd2;
use q8_mps::symmetric::SweepCheckpoint;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let n: usize = a[1].parse().unwrap();
    let x: f64 = a[2].parse().unwrap();
    let b: i32 = a[3].parse().unwrap();
    let dir = std::path::PathBuf::from(&a[4]);
    let q = Qcd2::new(n, x);
    let n_q = q.quarks(b);
    let mpo = { let mut u = Qcd2::new(n, x); u.lam = 0.0; u.mpo(n_q) };
    let exact_e = match (x, b) {
        (4.0, 0) => -51.9229999638, (4.0, 1) => -47.9964825669, (4.0, 2) => -36.6401053164,
        (9.0, 0) => -123.0642401146, (9.0, 1) => -113.9136751337, (9.0, 2) => -87.5269948585,
        _ => f64::NAN,
    };
    println!("{:>6} {:>13} {:>13} {:>13} {:>8} {:>10} {:>10}", "chi", "miss", "exact var", "two-site", "ratio", "1s part", "seconds");
    for chi in &a[5..] {
        let p = dir.join(format!("x{}_N{}_B{}_chi{}.done.state", x, n, b, chi));
        let Ok(c) = SweepCheckpoint::load(&p) else { println!("{chi:>6}  (no checkpoint at {})", p.display()); continue };
        let t0 = std::time::Instant::now();
        let (d2s, one, _two) = q8_mps::variance2::two_site_variance(&c.tensors, &mpo);
        let secs = t0.elapsed().as_secs_f64();
        let exact = q8_mps::variance::energy_variance(&c.tensors, &mpo).map(|(_, _, v)| v);
        let (ev, ratio) = match &exact {
            Ok(v) => (format!("{v:.6e}"), if v.abs() > 1e-12 { format!("{:.4}", d2s / v) } else { "—".into() }),
            Err(_) => ("refused".into(), "—".into()),
        };
        println!("{chi:>6} {:>13.3e} {ev:>13} {d2s:>13.6e} {ratio:>8} {one:>10.2e} {secs:>10.1}", c.last_energy - exact_e);
    }
}
