// The OTHER HALF of the dE3 subtraction ratio, in the SAME units as `s3_pair_cost`.
//
// G0's per-node trimer costs are wall-clock numbers taken on a loaded box. Dividing a
// work-unit pair cost by a wall-clock trimer cost compares two different quantities, which
// is gpu-prod's registry defect in miniature. So this prices the trimer side through the
// SAME `solve_geometry` call, with the same CPU-time and iteration-count instrumentation,
// so the two halves are commensurable and a ratio finally means something.
//
// (O,O,O) is deliberately ABSENT: at 207,025 determinants it crosses MPS_ROUTE_THRESHOLD,
// which routes into an MPO builder that HANGS rather than erroring. It is also the table
// where the subtraction is ~1% by any accounting, so it is not the case the ratio is for.
use holon_chem::dual::D2;
use holon_chem::elements::{by_symbol, Species};
use holon_chem::pair::solve_geometry;
use std::time::Instant;

fn cpu_seconds() -> f64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let tail = match s.rfind(')') { Some(i) => &s[i + 1..], None => return 0.0 };
    let f: Vec<&str> = tail.split_whitespace().collect();
    let ut: f64 = f.get(11).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    let st: f64 = f.get(12).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    (ut + st) / 100.0
}

fn main() {
    let reps: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let sp = |s: &str| by_symbol(s).unwrap_or_else(|| panic!("no species {s}"));
    // apex first, per TrimerSurface::realise: [0,0,0], [x,0,0], [y*u, y*s, 0].
    // Representative interior nodes at the located R_e values, u = cos(theta).
    let cases: Vec<(&str, [Species; 3], f64, f64, f64)> = vec![
        ("(Cl,H,H)", [sp("Cl"), sp("H"), sp("H")], 2.5369, 2.5369, 0.0),
        ("(H,Cl,Cl)", [sp("H"), sp("Cl"), sp("Cl")], 2.5369, 2.5369, 0.0),
        ("(Cl,Cl,Cl)", [sp("Cl"), sp("Cl"), sp("Cl")], 4.0241, 4.0241, 0.5),
        ("(O,O,H)", [sp("O"), sp("O"), sp("H")], 2.4421, 1.9909, 0.0),
    ];
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("# trimer node solves, {reps} reps, WORK UNITS -- same call and units as s3_pair_cost");
    println!("# loadavg at start  {}", la.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
    for (name, spx, x, y, u) in &cases {
        let s = (1.0 - u * u).max(0.0).sqrt();
        let mut walls = Vec::new();
        let mut cpus = Vec::new();
        let mut iters = Vec::new();
        for _ in 0..reps {
            let c0 = cpu_seconds();
            let t = Instant::now();
            let v = solve_geometry(
                spx,
                vec![
                    [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                    [D2::c(*x), D2::c(0.0), D2::c(0.0)],
                    [D2::c(y * u), D2::c(y * s), D2::c(0.0)],
                ],
            );
            walls.push(t.elapsed().as_secs_f64() * 1e3);
            cpus.push((cpu_seconds() - c0) * 1e3);
            iters.push(v.davidson_iters);
        }
        walls.sort_by(|a, b| a.partial_cmp(b).unwrap());
        cpus.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!(
            "{:<12} WALL {:>9.1}-{:>9.1} ms ({:>5.2}x)   CPU {:>8.1}-{:>8.1} ms ({:>5.2}x)   iters {:?}",
            name, walls[0], walls[walls.len() - 1], walls[walls.len() - 1] / walls[0].max(1e-9),
            cpus[0], cpus[cpus.len() - 1], cpus[cpus.len() - 1] / cpus[0].max(1e-9), iters
        );
    }
    let la2 = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("# loadavg at end    {}", la2.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
}
