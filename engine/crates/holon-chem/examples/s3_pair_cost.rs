// What does one pair solve cost? The dE3 emitter subtracts pair energies per node, so this
// decides whether the subtraction needs an axis cache or can be paid naively.
//
// REPEATS AND SPREAD, not a single draw. The first version of this instrument quoted one
// wall-clock timing per pair on a box at loadavg ~65 and read H-Cl at 19,970 ms; the next
// run read 216 ms for the same call. A single timing on a contended box is a DRAW, not a
// measurement -- this campaign's own registered lesson, walked into by its own registrant.
//
// VALUE-ONLY solves (constant centres, no dual propagation): the dE3 subtraction needs
// energies, and the emitter is fenced from emitting derivatives of the subtracted quantity
// until `Surface::subtract` can carry them, so there is nothing to spend duals on.
use holon_chem::dual::D2;
use holon_chem::elements::{by_symbol, Species};
use holon_chem::pair::{atom_energy, solve_geometry};
use std::time::Instant;

fn solve_ms(a: Species, b: Species, r: f64) -> (f64, f64) {
    let t = Instant::now();
    let v = solve_geometry(
        &[a, b],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::c(r)],
        ],
    );
    (t.elapsed().as_secs_f64() * 1e3, v.e.v)
}

fn main() {
    let reps: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let sp = |s: &str| by_symbol(s).unwrap_or_else(|| panic!("no species {s}"));
    let pairs: Vec<(Species, Species, f64)> = vec![
        (sp("H"), sp("H"), 1.3887),
        (sp("O"), sp("H"), 1.9909),
        (sp("H"), sp("Cl"), 2.5369),
        (sp("O"), sp("O"), 2.4421),
        (sp("Cl"), sp("Cl"), 4.0241),
    ];
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("# value-only pair solves, {reps} reps each");
    println!("# loadavg at start  {}", la.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
    println!("{:<10} {:>10} {:>10} {:>10} {:>8} {:>16}", "pair", "min ms", "med ms", "max ms", "spread", "dE2 (Ha)");
    for (a, b, r) in &pairs {
        let mut ms: Vec<f64> = Vec::new();
        let mut e = 0.0;
        for _ in 0..reps {
            let (t, ev) = solve_ms(*a, *b, *r);
            ms.push(t);
            e = ev;
        }
        ms.sort_by(|x, y| x.partial_cmp(y).unwrap());
        let de2 = e - atom_energy(*a) - atom_energy(*b);
        println!(
            "{:<10} {:>10.1} {:>10.1} {:>10.1} {:>7.2}x {:>16.8}",
            format!("{}-{}", a.symbol, b.symbol),
            ms[0], ms[ms.len() / 2], ms[ms.len() - 1], ms[ms.len() - 1] / ms[0].max(1e-9), de2
        );
    }
    let la2 = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("# loadavg at end    {}", la2.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
}
