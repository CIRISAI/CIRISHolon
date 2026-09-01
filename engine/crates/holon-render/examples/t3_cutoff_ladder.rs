//! THE DERIVED PAIR CUTOFF, MEASURED: what each truncation budget actually costs in
//! radius, and therefore in box size and in cell count.
//!
//! The pair cutoff is not chosen, it is READ OFF the curve at a declared energy budget
//! (`Sim::derive_pair_cutoff`): the inner edge of the switch is the radius at which the
//! curve is already under the budget, and the outer edge is one switch width further.
//! This example prints the ladder so the trade is visible rather than folded into a
//! constant — a tighter budget is a longer cutoff, a longer cutoff is a bigger minimum
//! box (the minimum image needs `2 * r_cut`), and a bigger box at fixed density is more
//! atoms.
//!
//! Run: `cargo run -p holon-render --example t3_cutoff_ladder`
//!
//! Measured on the shipped placeholder H-H curve (`viewer/h2_potential.json`, knots out to
//! 12 bohr, exponential tail beyond):
//!
//! ```text
//! budget     r_in     r_cut    minimum periodic edge
//! 1e-6      13.81     15.81     31.6
//! 1e-8      18.28     20.28     40.6
//! 1e-10     22.76     24.76     49.5
//! 1e-12     27.23     29.23     58.5
//! ```
//!
//! The tail is an exponential, so the radius grows LOGARITHMICALLY in the budget: two more
//! decades of accuracy costs about 4.5 bohr, every time. That is the shape that makes a
//! truncation affordable at all, and it is worth seeing rather than assuming.

use holon_render::sim::Sim;

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/viewer/h2_potential.json");
    let src = std::fs::read_to_string(path).expect("the placeholder curve is shipped");
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(s.table_mut(), &src).expect("table loads");
    s.adopt_table_timescale();
    s.resize_storage(2);
    s.sync_species();

    let t = s.table();
    println!(
        "curve: knots {:.2} .. {:.2} bohr,  D_e {:.6e} Ha at R_e {:.4} bohr",
        t.r_min(),
        t.r_max(),
        t.d_e,
        t.r_e
    );
    println!("\nthe tail, past the last knot:");
    for r in [12.0f64, 14.0, 16.0, 20.0, 25.0, 30.0, 40.0] {
        println!("  |u({r:5.1})| = {:.4e}", t.u(r).abs());
    }

    println!("\nbudget      r_in     r_cut   minimum periodic edge");
    for floor in [1e-6f64, 1e-7, 1e-8, 1e-9, 1e-10, 1e-12, 1e-14] {
        match s.derive_pair_cutoff(floor) {
            None => println!("{floor:8.0e}   (no cutoff meets this budget)"),
            Some((r_in, r_cut)) => println!(
                "{floor:8.0e}  {r_in:8.3}  {r_cut:8.3}   {:8.2}",
                2.0 * r_cut
            ),
        }
    }
}
