//! Curvature at the minimum against knot density: what the derived timestep converges to.
use holon_chem::elements::{HYDROGEN, CHLORINE};
use holon_chem::pair::generate_pair_table;
fn main() {
    for (a, b, name) in [(HYDROGEN, HYDROGEN, "H-H"), (HYDROGEN, CHLORINE, "H-Cl")] {
        println!("{name}:");
        for n in [24usize, 48, 96, 192, 384, 492] {
            let pt = generate_pair_table(a, b, n);
            let w = pt.meta.well.unwrap();
            println!("  {n:>4} knots  R_e {:.9}  D_e {:.9}  k_e {:.9}", w.r_e, w.d_e, w.k_e);
        }
    }
}
