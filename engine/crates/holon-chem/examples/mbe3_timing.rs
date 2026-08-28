use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::pair::solve_geometry;
use holon_chem::trimer;
use std::time::Instant;

fn general(cs: &[[f64; 3]]) -> f64 {
    let h = by_symbol("H").unwrap();
    let sp = vec![h; cs.len()];
    let d: Vec<[D2; 3]> = cs.iter().map(|c| [D2::c(c[0]), D2::c(c[1]), D2::c(c[2])]).collect();
    solve_geometry(&sp, d).e.v
}

fn main() {
    // agreement
    let mut worst = 0.0f64;
    let mut at = String::new();
    let mut cases: Vec<Vec<[f64; 3]>> = vec![
        vec![[0.0, 0.0, 0.0]],
        vec![[0.0, 0.0, 0.0], [1.4, 0.0, 0.0]],
        vec![[0.0, 0.0, 0.0], [0.9, 0.0, 0.0]],
        vec![[0.0, 0.0, 0.0], [8.0, 0.0, 0.0]],
    ];
    for &(x, y, u) in &[
        (0.9f64, 0.9f64, 0.5f64), (1.4, 1.4, 0.5), (1.4, 2.0, 0.0), (0.9, 8.0, -1.0),
        (3.5, 3.5, -1.0), (2.0, 5.0, 0.3), (7.0, 8.0, -0.9), (0.7, 0.7, -1.0),
        (1.0, 6.0, 0.4), (4.0, 4.0, 0.25),
    ] {
        let s = (1.0 - u * u).max(0.0).sqrt();
        cases.push(vec![[0.0, 0.0, 0.0], [x, 0.0, 0.0], [y * u, y * s, 0.0]]);
    }
    for c in &cases {
        let a = trimer::hydrogen_energy(c);
        let b = general(c);
        let d = (a - b).abs();
        if d > worst {
            worst = d;
            at = format!("{c:?} fast={a:.12} gen={b:.12}");
        }
    }
    println!("fast vs general: max |dE| = {worst:.3e} Ha   at {at}");
    println!("E(H)  fast = {:.12}", trimer::atom_energy());
    println!("E(H2 r_e) fast = {:.12}  h2_point = {:.12}", trimer::pair_energy(1.3886940), holon_chem::h2_point(1.3886940).e);

    // the probe's disclosed priors, recomputed on the fast path
    let e_h = trimer::atom_energy();
    let re = 1.3886940;
    println!("dE3 equilateral(r_e)   = {:+.6}", trimer::de3_sides(re, re, re, e_h));
    println!("dE3 linear (r_e,r_e)   = {:+.6}", trimer::de3_sides(re, re, 2.0 * re, e_h));
    println!("dE3 H2+H at 2.0 bohr   = {:+.6}", trimer::de3_sides(re, 2.0, re + 2.0, e_h));

    // timing
    let n = 2000;
    let t = Instant::now();
    for i in 0..n {
        let x = 1.0 + 0.001 * i as f64;
        std::hint::black_box(trimer::hydrogen_energy(&[[0.0,0.0,0.0],[x,0.0,0.0],[0.4,1.3,0.0]]));
    }
    println!("fast H3   {:8.2} us", t.elapsed().as_secs_f64() * 1e6 / n as f64);
    let t = Instant::now();
    for i in 0..n {
        std::hint::black_box(trimer::pair_energy(1.0 + 0.001 * i as f64));
    }
    println!("fast H2   {:8.2} us", t.elapsed().as_secs_f64() * 1e6 / n as f64);
    let t = Instant::now();
    for i in 0..n {
        std::hint::black_box(trimer::de3_sides(1.4, 1.4 + 0.001 * i as f64, 2.0, e_h));
    }
    println!("fast dE3  {:8.2} us", t.elapsed().as_secs_f64() * 1e6 / n as f64);
}
