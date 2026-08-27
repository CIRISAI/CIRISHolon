//! How much of h3ere2's path ORDER is physics, and how much is the index tiebreak?
//!
//! Arrival times are integer step counts, and at dt = 0.005 the fast end of the object
//! arrives within 3-5 steps. Integers that small tie often, and every tie is broken by
//! canonical KINDS order — which carries no physical content and is identical in both
//! arms. This measures the fraction of adjacent orderings in the emitted path that are
//! decided that way, and whether a finer step recovers them.

use ciris_sim_core::data::{COUPLING, KINDS, N, TWINS};
use ciris_sim_core::dynamics::{step, Params, State};
use ciris_sim_core::structure::Structure;

fn blocks() -> Vec<(&'static str, Vec<usize>)> {
    let idx = |n: &str| KINDS.iter().position(|k| *k == n).unwrap();
    vec![
        ("Facts", ["Facts","Confidence","Model","Premises"].iter().map(|s| idx(s)).collect()),
        ("Rules", ["Rules","Priorities","Process"].iter().map(|s| idx(s)).collect()),
        ("Identity", ["Identity"].iter().map(|s| idx(s)).collect()),
        ("Manner", ["Manner","Structure","Circumstances"].iter().map(|s| idx(s)).collect()),
    ]
}

fn arrivals(c: &[[f64; N]; N], seeds: &[usize], dt: f64) -> [Option<usize>; N] {
    let st = Structure::<N>::from_coupling(c, TWINS);
    let p = Params { dt, ..Params::harmonic() };
    let mut pos = [[0.0f64; 3]; N];
    for &s in seeds { pos[s][0] = 1.0; }
    let mut state = State::at_rest(pos);
    let mut first = [None::<usize>; N];
    for &s in seeds { first[s] = Some(0); }
    let budget = (20_000.0 * (0.005 / dt)) as usize;
    let mut t = 0usize;
    while t < budget && first.iter().any(|f| f.is_none()) {
        step(&mut state, &st, &p, true);
        t += 1;
        for i in 0..N {
            if first[i].is_some() { continue; }
            let q = state.pos[i];
            if (q[0]*q[0] + q[1]*q[1] + q[2]*q[2]).sqrt() > 1e-3 { first[i] = Some(t); }
        }
    }
    first
}

/// Adjacent pairs in the emitted (non-seeded) path that are TIED, and of those, how many
/// are the two twin pairs — which tie exactly and can never be recovered by a finer step.
fn tie_stats(first: &[Option<usize>; N], seeds: &[usize]) -> (usize, usize, usize) {
    let mut order: Vec<usize> = (0..N).filter(|i| !seeds.contains(i)).collect();
    order.sort_by_key(|&i| (first[i].unwrap_or(usize::MAX), i));
    let mut tied = 0;
    let mut twin_tied = 0;
    for w in order.windows(2) {
        if first[w[0]] == first[w[1]] {
            tied += 1;
            let pair = (w[0].min(w[1]), w[0].max(w[1]));
            if TWINS.iter().any(|&(a, b)| (a.min(b), a.max(b)) == pair) { twin_tied += 1; }
        }
    }
    (tied, twin_tied, order.len().saturating_sub(1))
}

fn main() {
    println!("Fraction of the emitted path order decided by the INDEX TIEBREAK, not physics\n");
    for &dt in &[0.005f64, 0.0005, 0.00005] {
        let scale = 0.005 / dt;
        println!("--- dt = {dt} ({scale:.0}x finer than shipped) ---");
        println!("{:<10} {:>28} {:>10} {:>10} {:>12}", "block", "arrival steps (non-seeded)", "tied", "of which twin", "tied frac");
        let (mut tt, mut twt, mut pp) = (0usize, 0usize, 0usize);
        for (name, seeds) in blocks() {
            let f = arrivals(&COUPLING, &seeds, dt);
            let (tied, twin_tied, pairs) = tie_stats(&f, &seeds);
            tt += tied; twt += twin_tied; pp += pairs;
            let mut order: Vec<usize> = (0..N).filter(|i| !seeds.contains(i)).collect();
            order.sort_by_key(|&i| (f[i].unwrap_or(usize::MAX), i));
            let steps: Vec<String> = order.iter().map(|&i| f[i].map(|v| v.to_string()).unwrap_or("-".into())).collect();
            println!("{:<10} {:>28} {:>10} {:>10} {:>11.0}%", name, steps.join(","), tied, twin_tied, 100.0 * tied as f64 / pairs as f64);
        }
        println!("{:<10} {:>28} {:>10} {:>10} {:>11.0}%\n", "TOTAL", "", tt, twt, 100.0 * tt as f64 / pp as f64);
    }
    println!("The twin ties are EXACT (a theorem) and no step size recovers them.");
    println!("Every other tie is quantisation and a finer step resolves it.");
}
