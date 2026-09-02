// What does one pair solve cost, and WHY does Cl-Cl's wall time vary 23x?
//
// Priced in WORK UNITS, not seconds. Wall clock on a contended box is a draw; CPU time and
// iteration count are properties of the computation. The discriminator:
//
//   CPU stable while wall varies  -> scheduling/contention. The pair table is fine.
//   CPU varies too                -> the solver takes different PATHS on identical input,
//                                    which is a different and much worse finding.
//   iters vary on identical input -> settles it outright; no wall number means anything.
//
// NOT RUN, and why: the banked BLAS-spin-thread control does not apply here. `holon-chem`
// has NO blas/lapack/ndarray/rayon dependency and no thread spawn in `fci.rs` or `pair.rs` —
// the solve is single-threaded pure Rust, so `OPENBLAS_NUM_THREADS` cannot discriminate
// anything. A control that cannot fire is not a control.
use holon_chem::dual::D2;
use holon_chem::elements::{by_symbol, Species};
use holon_chem::pair::{atom_energy, solve_geometry};
use std::time::Instant;

/// utime + stime for this process, in seconds. /proc/self/stat fields 14 and 15, in ticks.
fn cpu_seconds() -> f64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    // The comm field can contain spaces and parens; everything after the last ')' is safe.
    let tail = match s.rfind(')') {
        Some(i) => &s[i + 1..],
        None => return 0.0,
    };
    let f: Vec<&str> = tail.split_whitespace().collect();
    // After comm and state, field 14 (utime) is index 11 of this tail, 15 (stime) index 12.
    let ut: f64 = f.get(11).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    let st: f64 = f.get(12).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    (ut + st) / 100.0 // USER_HZ, 100 on this kernel
}

fn main() {
    let reps: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let only = std::env::args().nth(2);
    let sp = |s: &str| by_symbol(s).unwrap_or_else(|| panic!("no species {s}"));
    let all: Vec<(Species, Species, f64)> = vec![
        (sp("H"), sp("H"), 1.3887),
        (sp("O"), sp("H"), 1.9909),
        (sp("H"), sp("Cl"), 2.5369),
        (sp("O"), sp("O"), 2.4421),
        (sp("Cl"), sp("Cl"), 4.0241),
    ];
    let pairs: Vec<_> = match &only {
        Some(p) => all.into_iter().filter(|(a, b, _)| format!("{}-{}", a.symbol, b.symbol) == *p).collect(),
        None => all,
    };
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("# value-only pair solves, {reps} reps, priced in WORK UNITS");
    println!("# loadavg at start  {}", la.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
    println!("{:<8} {:>4} {:>10} {:>10} {:>8} {:>7} {:>16}", "pair", "rep", "wall ms", "cpu ms", "wall/cpu", "iters", "E_total");
    for (a, b, r) in &pairs {
        let mut walls: Vec<f64> = Vec::new();
        let mut cpus: Vec<f64> = Vec::new();
        let mut iters: Vec<usize> = Vec::new();
        let mut last_e = 0.0f64;
        for k in 0..reps {
            let c0 = cpu_seconds();
            let t = Instant::now();
            let v = solve_geometry(
                &[*a, *b],
                vec![
                    [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                    [D2::c(0.0), D2::c(0.0), D2::c(*r)],
                ],
            );
            let wall = t.elapsed().as_secs_f64() * 1e3;
            let cpu = (cpu_seconds() - c0) * 1e3;
            println!(
                "{:<8} {:>4} {:>10.1} {:>10.1} {:>8.2} {:>7} {:>16.8}",
                format!("{}-{}", a.symbol, b.symbol), k, wall, cpu,
                wall / cpu.max(1e-9), v.davidson_iters, v.e.v
            );
            walls.push(wall);
            cpus.push(cpu);
            iters.push(v.davidson_iters);
            last_e = v.e.v;
        }
        walls.sort_by(|x, y| x.partial_cmp(y).unwrap());
        cpus.sort_by(|x, y| x.partial_cmp(y).unwrap());
        let de2 = last_e - atom_energy(*a) - atom_energy(*b);
        println!(
            "  -> {}-{}: WALL spread {:.2}x ({:.1}-{:.1} ms), CPU spread {:.2}x ({:.1}-{:.1} ms), iters {:?}, dE2 {:.8}",
            a.symbol, b.symbol,
            walls[walls.len() - 1] / walls[0].max(1e-9), walls[0], walls[walls.len() - 1],
            cpus[cpus.len() - 1] / cpus[0].max(1e-9), cpus[0], cpus[cpus.len() - 1],
            iters, de2
        );
    }
    let la2 = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("# loadavg at end    {}", la2.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
}
