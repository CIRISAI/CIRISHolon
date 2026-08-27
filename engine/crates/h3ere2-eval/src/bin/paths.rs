//! Inspect the paths the engine produces, before any generation happens.
//! Guards prereg failure-mode 3 (degenerate paths): if real and scrambled couplings
//! give near-identical orders, arms B and C converge trivially.
use ciris_sim_core::data::COUPLING;
use h3ere2_eval::{blocks, path, scramble};
use std::collections::HashSet;

fn fmt(p: &[path::Arrival]) -> String {
    p.iter().map(|a| format!("{}{}", a.kind, if a.seeded { "*" } else { "" }))
        .collect::<Vec<_>>().join(" > ")
}

fn main() {
    let pp = path::PathParams::default();
    println!("REAL COUPLING (* = seeded by the encoder)");
    let mut real = Vec::new();
    for s in blocks::SURFACES {
        let seeds = blocks::members(s).unwrap();
        let p = path::relax(&path::structure_for(&COUPLING), &seeds, &pp);
        println!("  {:<9} {}", s, fmt(&p));
        real.push(p.iter().map(|a| a.index).collect::<Vec<_>>());
    }

    println!("\nSCRAMBLES (seeds 0..9), surface = Facts");
    let seeds = blocks::members("Facts").unwrap();
    let mut orders = HashSet::new();
    for sd in 0..10u64 {
        let p = path::relax(&path::structure_for(&scramble::scramble(sd)), &seeds, &pp);
        println!("  seed {sd}: {}", fmt(&p));
        orders.insert(p.iter().map(|a| a.index).collect::<Vec<_>>());
    }
    println!("\ndistinct scramble orders: {}/10", orders.len());
    println!("scramble orders equal to real: {}",
             orders.iter().filter(|o| **o == real[0]).count());

    // path diversity across the four surfaces under real coupling
    let d: HashSet<_> = real.iter().cloned().collect();
    println!("distinct real paths across the 4 surfaces: {}/4", d.len());
}
