//! Does h3ere2's integrator stay stable on the SCRAMBLED couplings?
//! Verlet needs dt < 2/sqrt(lambda_max); harmonic() fixes dt = 0.005. If a scramble
//! raised lambda_max enough, arm B would blow up and every kind would cross the
//! threshold at once — a degenerate path, not a different one.
use ciris_sim_core::data::{COUPLING, N, TWINS};
use ciris_sim_core::structure::Structure;

fn scramble(seed: u64) -> [[f64; N]; N] {
    struct Rng(u64);
    impl Rng {
        fn next_u64(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            z ^ (z >> 31)
        }
        fn below(&mut self, n: u64) -> u64 {
            let zone = u64::MAX - (u64::MAX % n);
            loop { let r = self.next_u64(); if r < zone { return r % n; } }
        }
    }
    let mut vals: Vec<f64> = Vec::new();
    for i in 0..N { for j in (i + 1)..N { vals.push(COUPLING[i][j]); } }
    let mut rng = Rng(seed.wrapping_add(0x9E3779B97F4A7C15));
    for k in (1..vals.len()).rev() { let s = rng.below((k + 1) as u64) as usize; vals.swap(k, s); }
    let mut out = [[0.0f64; N]; N];
    let mut it = vals.into_iter();
    for i in 0..N { for j in (i + 1)..N { let v = it.next().unwrap(); out[i][j] = v; out[j][i] = v; } }
    out
}

fn main() {
    let dt = 0.005f64; // Params::harmonic()
    println!("Verlet stability margin at dt = {dt} (needs dt < 2/sqrt(lambda_max))\n");
    println!("{:<22} {:>12} {:>12} {:>12} {:>10}", "coupling", "lambda_max", "dt_max", "margin", "converged");
    let show = |name: &str, c: &[[f64; N]; N]| {
        let st = Structure::<N>::from_coupling(c, TWINS);
        // the dynamics runs on the SYMMETRISED matrix, so that is the spectrum that matters
        let sym = Structure::<N>::from_coupling(&st.coupling_sym, TWINS);
        let lmax = sym.lambda_max();
        let dtmax = 2.0 / lmax.sqrt();
        println!("{:<22} {:>12.4} {:>12.4} {:>11.1}x {:>10}", name, lmax, dtmax, dtmax / dt, sym.spectrum_converged);
    };
    show("real COUPLING", &COUPLING);
    for s in 0..10u64 { show(&format!("scramble {s}"), &scramble(s)); }
    connectivity();
    println!("\nmargin = how many times larger the stable step could be. All >> 1 means the");
    println!("integrator is nowhere near its limit and arm B is not blowing up.");
}


/// Does every kind actually ARRIVE under every scramble? If a scramble disconnected a
/// node it would never cross the threshold, the path would be short, and arm B would
/// differ from arm C in path LENGTH as well as in content — a systematic difference the
/// renderer could key on that has nothing to do with the structure being real.
fn connectivity() {
    use ciris_sim_core::dynamics::{step, Params, State};
    use ciris_sim_core::data::KINDS;
    let idx = |n: &str| KINDS.iter().position(|k| *k == n).unwrap();
    let blocks: Vec<(&str, Vec<usize>)> = vec![
        ("Facts", ["Facts","Confidence","Model","Premises"].iter().map(|s| idx(s)).collect()),
        ("Rules", ["Rules","Priorities","Process"].iter().map(|s| idx(s)).collect()),
        ("Identity", ["Identity"].iter().map(|s| idx(s)).collect()),
        ("Manner", ["Manner","Structure","Circumstances"].iter().map(|s| idx(s)).collect()),
    ];
    println!("\nEvery kind arrives? (threshold 1e-3, budget 20000 steps, as h3ere2 runs it)");
    println!("{:<12} {:>10} {:>14} {:>16}", "coupling", "block", "arrived/11", "fiedler(sym)");
    let mut bad = 0;
    let run = |c: &[[f64; N]; N], seeds: &[usize]| -> (usize, usize) {
        let st = Structure::<N>::from_coupling(c, TWINS);
        let p = Params::harmonic();
        let mut pos = [[0.0f64; 3]; N];
        for &s in seeds { pos[s][0] = 1.0; }
        let mut state = State::at_rest(pos);
        let mut first = [None::<usize>; N];
        for &s in seeds { first[s] = Some(0); }
        let mut t = 0usize;
        while t < 20_000 && first.iter().any(|f| f.is_none()) {
            step(&mut state, &st, &p, true);
            t += 1;
            for i in 0..N {
                if first[i].is_some() { continue; }
                let q = state.pos[i];
                if (q[0]*q[0] + q[1]*q[1] + q[2]*q[2]).sqrt() > 1e-3 { first[i] = Some(t); }
            }
        }
        (first.iter().filter(|f| f.is_some()).count(), t)
    };
    for (label, c) in std::iter::once(("real".to_string(), COUPLING))
        .chain((0..10u64).map(|s| (format!("scramble {s}"), scramble(s))))
    {
        let st = Structure::<N>::from_coupling(&c, TWINS);
        let sym = Structure::<N>::from_coupling(&st.coupling_sym, TWINS);
        for (bn, seeds) in &blocks {
            let (n_arr, _) = run(&c, seeds);
            if n_arr < N { bad += 1; }
            if n_arr < N || label == "real" {
                println!("{:<12} {:>10} {:>14} {:>16.5}", label, bn, format!("{n_arr}/{}", N), sym.fiedler());
            }
        }
    }
    println!("blocks with an unreached kind: {bad} (out of {} block x coupling combinations)", 4 * 11);
    if bad == 0 { println!("=> every path has the full 11 entries; arm B and arm C cannot differ in LENGTH."); }
}
