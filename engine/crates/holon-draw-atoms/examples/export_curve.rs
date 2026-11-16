//! Write the engine's own H2 curve to a file, so that a host can load it without solving it.
//!
//! The curve is `holon-chem`'s STO-3G full-CI H2 (`holon_chem::stream_table`, the generator
//! behind `holon_render::generate_table`), on the grid the atoms page asks for: 492 knots
//! from 0.3 to 10 bohr. Every number is written with the shortest decimal that reads back to
//! the same f64, so the table a host fills from the file is the table the solve fills.
//!
//! ```text
//! cargo run --release -p holon-draw-atoms --example export_curve -- <commit> > crates/holon-draw-atoms/data/h2_sto3g.json
//! ```
//!
//! The fields are those of `docs/atoms/h2_potential.json` (the page's file door reads either),
//! plus the curvature column the generator also supplies.

const R_MIN: f64 = 0.3;
const R_MAX: f64 = 10.0;
const KNOTS: usize = 492;

fn main() {
    let commit = std::env::args().nth(1).unwrap_or_else(|| "unrecorded".to_string());
    let (mut r, mut e, mut f, mut e2) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let meta = holon_chem::stream_table(R_MIN, R_MAX, KNOTS, |_, ri, ei, fi, e2i| {
        r.push(ri);
        e.push(ei);
        f.push(fi);
        e2.push(e2i);
        true
    })
    .expect("the generator accepts this grid");
    // the same knots through the renderer's own door, as a check that this file is that curve
    let mut sim = Box::new(holon_render::sim::Sim::empty());
    assert_eq!(holon_render::generate_table(&mut sim, R_MIN, R_MAX, KNOTS), holon_render::TABLE_OK);
    let t = sim.table_mut();
    assert_eq!(t.knots(), KNOTS);
    for i in 0..KNOTS {
        assert_eq!(t.knot_r(i), r[i]);
    }
    let list = |v: &[f64]| v.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(", ");
    println!("{{");
    println!("  \"provenance\": \"ENGINE-COMPUTED AHEAD OF TIME: H2 in the STO-3G basis, full CI, by holon-chem ({}); {} knots from {} to {} bohr, uniform in R^(-1/4), written by holon-draw-atoms/examples/export_curve.rs at commit {}. R_e, D_e and the asymptote are the generator's own. Exact in the model, not a prediction of experiment.\",", holon_chem::PROVENANCE.replace('"', "'"), KNOTS, R_MIN, R_MAX, commit);
    println!("  \"units\": \"Hartree atomic units: R in bohr, E in hartree, F in hartree/bohr, d2E in hartree/bohr^2. F is the FORCE, so dE/dR = -F.\",");
    println!("  \"R_e\": {:?},", meta.r_e);
    println!("  \"D_e\": {:?},", meta.d_e);
    println!("  \"E_asymptote\": {:?},", meta.e_asymptote);
    println!("  \"R_grid_bohr\": [{}],", list(&r));
    println!("  \"E_hartree\": [{}],", list(&e));
    println!("  \"F_hartree_per_bohr\": [{}],", list(&f));
    println!("  \"d2E_hartree_per_bohr2\": [{}]", list(&e2));
    println!("}}");
}
