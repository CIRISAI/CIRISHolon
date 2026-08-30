use holon_chem::elements::{by_symbol, HYDROGEN};
use holon_chem::pair::generate_pair_table;
use std::time::Instant;
fn main() {
    let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(24);
    for sym in ["H", "He", "Li", "Cl"] {
        let b = by_symbol(sym).unwrap();
        let f = holon_chem::pair::feasibility(HYDROGEN, b);
        let t = Instant::now();
        let pt = generate_pair_table(HYDROGEN, b, n);
        println!("H{sym:<3} n_orb {:>2} n_det {:>6}  {n} knots  {:>7.2} s   R_e {:?}",
            f.n_orb(), f.n_det(), t.elapsed().as_secs_f64(),
            pt.meta.well.map(|w| (w.r_e, w.d_e)));
    }
    for sym in ["He", "Li", "Cl"] {
        let b = by_symbol(sym).unwrap();
        let f = holon_chem::pair::feasibility(b, b);
        if f.is_infeasible() { println!("{sym}2 INFEASIBLE"); continue; }
        let t = Instant::now();
        let pt = generate_pair_table(b, b, n);
        println!("{sym}2  n_orb {:>2} n_det {:>6}  {n} knots  {:>7.2} s   R_e {:?}",
            f.n_orb(), f.n_det(), t.elapsed().as_secs_f64(),
            pt.meta.well.map(|w| (w.r_e, w.d_e)));
    }
}
