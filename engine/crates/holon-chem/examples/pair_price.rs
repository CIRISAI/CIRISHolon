//! Price one pair curve on THIS machine: `pair_price <A> <B> [knots]`.
//!
//! The provenance instrument for `holon_render::bank::predicted_load_seconds`: the split
//! the browser enforces is a prediction, and a prediction is re-measured on the engine
//! that ships, never carried over from the solver it was fitted on.
use holon_chem::elements::by_symbol;
use holon_chem::pair::{automatic_route, generate_pair_table};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let a = by_symbol(&args[1]).expect("species A");
    let b = by_symbol(&args[2]).expect("species B");
    let n: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(8);
    let f = automatic_route(a, b);
    let t = Instant::now();
    let pt = generate_pair_table(a, b, n);
    let s = t.elapsed().as_secs_f64();
    println!(
        "{}{} n_orb {} n_det {} knots {} total {:.3} s per_knot {:.1} ms R_e {:?}",
        a.symbol, b.symbol, f.n_orb(), f.n_det(), n, s, 1e3 * s / n as f64,
        pt.meta.well.map(|w| (w.r_e, w.d_e))
    );
}
