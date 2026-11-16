//! Thermalise one configuration, measure the plaquette, flow it, and write the record.
//!
//!   cargo run --release -p holon-gauge4 --example export -- [L] [therm] [meas] [seed] [out]
//!
//! Defaults: L = 12 (a 12^4 lattice), 500 thermalisation sweeps, 500 measured sweeps, seed
//! 20261115. A sweep is one heat bath and four overrelaxations. With `out` = `-` nothing is
//! written (a measurement run).
use holon_gauge4::{Lattice, Record, Rng};
use std::time::Instant;

pub const BETA: f64 = 6.0;
pub const N_OR: usize = 4;
/// Flow times kept, in lattice units: `sqrt(8 t)` from 0 to 4.5 spacings.
pub const TIMES: [f64; 11] = [0.0, 0.03, 0.05, 0.09, 0.15, 0.25, 0.40, 0.65, 1.00, 1.60, 2.50];
pub const EPS: f64 = 0.01;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let l: usize = a.first().map(|s| s.parse().unwrap()).unwrap_or(12);
    let therm: usize = a.get(1).map(|s| s.parse().unwrap()).unwrap_or(500);
    let meas: usize = a.get(2).map(|s| s.parse().unwrap()).unwrap_or(500);
    let seed: u64 = a.get(3).map(|s| s.parse().unwrap()).unwrap_or(20261115);
    let out = a.get(4).cloned().unwrap_or_else(|| format!("{}/data/gluon_b600_l{l}.hg4f", env!("CARGO_MANIFEST_DIR")));
    let git = |args: &[&str]| std::process::Command::new("git").args(args).current_dir(env!("CARGO_MANIFEST_DIR")).output().ok().and_then(|o| String::from_utf8(o.stdout).ok()).map(|s| s.trim().to_string()).unwrap_or_default();
    // the repository's HEAD, and whether this crate's own source differs from it
    let dirty = !git(&["status", "--porcelain", "--", "src", "examples", "Cargo.toml"]).is_empty();
    let commit = format!("{}{}", git(&["rev-parse", "--short", "HEAD"]), if dirty { " (with this crate's source not yet committed)" } else { "" });

    let t0 = Instant::now();
    let mut lat = Lattice::hot([l; 4], seed);
    let mut rng = Rng::new(seed ^ 0x5eed_0000_0000_0001);
    for _ in 0..therm {
        lat.sweep(BETA, N_OR, &mut rng);
    }
    let t_therm = t0.elapsed().as_secs_f64();
    let t1 = Instant::now();
    let p: Vec<f64> = (0..meas)
        .map(|_| {
            lat.sweep(BETA, N_OR, &mut rng);
            lat.mean_plaquette()
        })
        .collect();
    let t_meas = t1.elapsed().as_secs_f64();
    let mean = p.iter().sum::<f64>() / p.len().max(1) as f64;
    // the error of the mean from bins of several lengths: it stops growing once a bin is
    // longer than the autocorrelation
    let err = |bin: usize| {
        let b: Vec<f64> = p.chunks_exact(bin).map(|c| c.iter().sum::<f64>() / bin as f64).collect();
        let m = b.iter().sum::<f64>() / b.len() as f64;
        (b.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (b.len() * (b.len() - 1)) as f64).sqrt()
    };
    let errs: Vec<String> = [1usize, 5, 10, 25, 50, 100, 250].iter().filter(|&&b| meas / b >= 8).map(|&b| format!("bin {b}: {:.2e}", err(b))).collect();
    println!("{l}^4 beta {BETA} seed {seed}: {therm} thermalisation sweeps in {t_therm:.1} s, {meas} measured in {t_meas:.1} s");
    println!("mean plaquette {mean:.6}  errors by bin length: {}", errs.join(", "));
    let last = lat.mean_plaquette();
    let (du, dd) = lat.group_defect();
    println!("the kept configuration: plaquette {last:.6}, unitarity defect {du:.1e}, |det - 1| {dd:.1e}");

    let t2 = Instant::now();
    let err50 = if meas >= 400 { err(50) } else { f64::NAN };
    let provenance = format!(
        "holon-gauge4 examples/export.rs at commit {commit}: quenched SU(3), Wilson action, {l}^4 periodic, beta {BETA}, a = 0.0931 fm; hot start seed {seed}, update stream seed {seed} xor 0x5eed000000000001; {therm} thermalisation sweeps then {meas} measured (a sweep: 1 Cabibbo-Marinari/Kennedy-Pendleton heat bath + {N_OR} overrelaxations); mean plaquette over the measured sweeps {mean:.6} +- {err50:.1e} (bins of 50), the kept (last) configuration's {last:.6}; Wilson flow, Luscher RK3, step {EPS}, flow times {TIMES:?}; time slice 0; clover action density (u16), clover topological charge density (i16), Polyakov loop (i16 pairs), each quantised to 16 bits"
    );
    let rec = Record::flowed(&lat, BETA, 0, &TIMES, EPS, provenance);
    let t_flow = t2.elapsed().as_secs_f64();
    println!("flow to t = {} in {t_flow:.1} s", TIMES[TIMES.len() - 1]);
    println!("     t  sqrt(8t)        Q    <E>      t^2<E>   slice max E  slice max |q|");
    for s in &rec.snapshots {
        let amax = s.action.iter().fold(0.0f32, |m, x| m.max(*x));
        let qmax = s.charge.iter().fold(0.0f32, |m, x| m.max(x.abs()));
        println!("{:6.2}  {:7.3}  {:7.3}  {:8.5}  {:7.4}  {:10.5}  {:10.6}", s.t, (8.0 * s.t).sqrt(), s.q_total, s.e_mean, s.t * s.t * s.e_mean, amax, qmax);
    }
    if out != "-" {
        let bytes = rec.encode();
        std::fs::write(&out, &bytes).unwrap();
        println!("wrote {out}: {} bytes", bytes.len());
        println!("{}", rec.provenance);
    }
}
