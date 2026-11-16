//! The topological charge of a run of configurations as the flow proceeds.
//!
//!   cargo run --release -p holon-gauge4 --example topology -- [L] [therm] [gap] [count] [seed]
//!
//! Prints, for each of `count` configurations `gap` sweeps apart after `therm`, the clover
//! charge at flow times 0, 0.5, 1, 2 and 3 (step 0.02). After enough flow it sits near an
//! integer; how near is what the clover definition leaves at this lattice spacing.
use holon_gauge4::{Lattice, Rng};

fn main() {
    let a: Vec<usize> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let get = |i: usize, d: usize| a.get(i).copied().unwrap_or(d);
    let (l, therm, gap, count, seed) = (get(0, 8), get(1, 200), get(2, 20), get(3, 20), get(4, 11) as u64);
    let mut lat = Lattice::hot([l; 4], seed);
    let mut rng = Rng::new(seed ^ 0x5eed_0000_0000_0001);
    for _ in 0..therm {
        lat.sweep(6.0, 4, &mut rng);
    }
    println!("{l}^4 beta 6.0 seed {seed}: sweeps, plaquette, Q at t = 0, 0.5, 1, 2, 3");
    for k in 0..count {
        let mut w = lat.clone();
        let mut q = vec![w.topological_charge()];
        for (dt, steps) in [(0.5, 25), (0.5, 25), (1.0, 50), (1.0, 50)] {
            w.flow(dt, steps);
            q.push(w.topological_charge());
        }
        println!("{:5}  {:.5}  {}", therm + k * gap, lat.mean_plaquette(), q.iter().map(|x| format!("{x:8.4}")).collect::<Vec<_>>().join(" "));
        for _ in 0..gap {
            lat.sweep(6.0, 4, &mut rng);
        }
    }
}
