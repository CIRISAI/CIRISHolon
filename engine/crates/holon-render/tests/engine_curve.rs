//! Gates for the ENGINE-COMPUTED curve — the route the browser takes by default.
//!
//! # Why this file exists separately from `ledger.rs` and `amendments.rs`
//!
//! Those two load `viewer/h2_potential.json`, so after this change they exercise the
//! FALLBACK curve rather than the shipped one. That gap is the whole reason for this
//! file: without it, every conservation gate in the crate would be gauged on a curve the
//! viewer no longer uses.
//!
//! Nothing here touches the `extern "C"` globals except the one test that has to. The
//! ABI is a single shared static behind a mutex, so tests that drive it cannot run in
//! parallel with each other; keeping the rest at the `PotentialTable` level makes them
//! independent as well as faster.

use holon_render::sim::{Boundary, Sim};
use holon_render::table::{LoadStatus, PotentialTable};

const R_MIN: f64 = 0.3;
const R_MAX: f64 = 10.0;
const KNOTS: usize = 492;

/// Fill a table straight from the generator, the way `holon_table_generate` does.
fn generated() -> PotentialTable {
    let mut t = PotentialTable::empty();
    assert!(t.begin(KNOTS));
    let meta = holon_chem::stream_table(R_MIN, R_MAX, KNOTS, |i, r, e, f, e2| {
        t.knot(i, r, e, f) && t.knot_curvature(i, e2)
    })
    .expect("the generator produced a curve");
    assert_eq!(
        t.finish(meta.r_e, meta.d_e, meta.e_asymptote),
        LoadStatus::Ok
    );
    t
}

#[test]
fn the_generated_table_certifies_its_own_sign_convention() {
    // The same check the viewer runs on a file it was handed. The generator is not exempt
    // from it: `F = -dE/dR` is a convention the table asserts about its input, and the
    // input now comes from code rather than from a file, which changes who could get it
    // wrong but not whether they could.
    let t = generated();
    println!(
        "generated table: {} knots, residual {:.3e} vs alt {:.3e}, R_e {:.6}, D_e {:.6}",
        t.knots(),
        t.residual,
        t.residual_alt,
        t.r_e,
        t.d_e
    );
    assert!(
        t.residual_alt > 20.0 * t.residual,
        "the generated table cannot certify its sign convention: {:.3e} vs {:.3e}",
        t.residual,
        t.residual_alt
    );
    assert!(t.force(0.5) > 0.0, "the wall is not repulsive");
    assert!(t.force(3.0) < 0.0, "the tail is not attractive");
}

#[test]
fn the_generated_table_carries_its_curvature_column() {
    // The generator supplies d2E/dR2 for free (it is differentiating analytically
    // anyway), and the table's optional column is only useful if it actually arrives.
    let t = generated();
    assert!(t.has_supplied_curvature(), "the curvature column did not arrive");
    // Reported, never enforced: cubic Hermite is C1, so its curvature is discontinuous at
    // knots and disagreement with a supplied column is expected structure. Printing it is
    // the point -- a number here that had grown to O(1) would mean the column and the
    // interpolant had stopped describing the same function.
    println!("d2 mismatch (interpolant vs supplied column): {:.3e}", t.d2_mismatch);
    assert!(t.d2_mismatch < 0.5, "the curvature column and the interpolant have diverged");
}

#[test]
fn the_interpolant_reproduces_the_model_it_was_built_from() {
    // The knots are exact to 2.5e-15 against the referee (holon-chem's own gate). What
    // this checks is the OTHER half: that the curve the integrator actually sees --
    // the Hermite interpolant, evaluated between knots -- still tracks the model.
    let t = generated();
    let asym = t.e_asymptote;
    let mut worst_u: f64 = 0.0;
    let mut worst_f: f64 = 0.0;
    let mut worst_at = 0.0;
    let mut r = R_MIN;
    while r < R_MAX {
        let exact = holon_chem::h2_point(r);
        let du = (t.u(r) - (exact.e - asym)).abs();
        let df = (t.force(r) - exact.f).abs();
        if du > worst_u {
            worst_u = du;
            worst_at = r;
        }
        worst_f = worst_f.max(df);
        r += 0.00131;
    }
    println!("interpolant vs model: max |dU| = {worst_u:.3e} Eh at R = {worst_at:.4}, max |dF| = {worst_f:.3e} Eh/a0");
    assert!(worst_u < 1e-8, "the interpolant is {worst_u:.3e} Eh off the model");
    assert!(worst_f < 1e-5, "the interpolant's force is {worst_f:.3e} off the model");
}

#[test]
fn the_generated_and_file_routes_are_the_same_table() {
    // Two ways to fill one interpolator, so two chances to be wrong. The JSON emitter
    // writes each number with the shortest representation that round-trips, so this is
    // required to be BIT-IDENTICAL rather than close: anything less would mean the
    // fallback silently simulates a slightly different molecule from the default.
    let direct = generated();
    let json = holon_chem::generate_table(R_MIN, R_MAX, KNOTS)
        .expect("table")
        .to_json();
    let mut via_file = PotentialTable::empty();
    let parsed = holon_render::json::load_into(&mut via_file, &json).expect("emitted JSON loads");
    assert_eq!(parsed.provenance, holon_chem::PROVENANCE);

    assert_eq!(direct.knots(), via_file.knots());
    for i in 0..direct.knots() {
        assert_eq!(direct.knot_r(i), via_file.knot_r(i), "R differs at knot {i}");
        assert_eq!(direct.knot_u(i), via_file.knot_u(i), "U differs at knot {i}");
        assert_eq!(direct.knot_d(i), via_file.knot_d(i), "dU/dR differs at knot {i}");
    }
    assert_eq!(direct.r_e, via_file.r_e);
    assert_eq!(direct.d_e, via_file.d_e);
    assert_eq!(direct.e_asymptote, via_file.e_asymptote);
    // The emitter writes the curvature under the referee's spelling. Before `json.rs`
    // learned that name this loaded clean and SILENTLY without the column, which is the
    // failure mode of every optional field: absent and ignored look identical.
    assert!(
        via_file.has_supplied_curvature(),
        "the curvature column did not survive the JSON round trip"
    );
}

#[test]
fn a_bad_grid_is_refused_rather_than_defaulted() {
    let mut t = PotentialTable::empty();
    assert!(t.begin(10));
    assert!(
        holon_chem::stream_table(0.0, 10.0, 10, |i, r, e, f, _| t.knot(i, r, e, f)).is_none(),
        "R = 0 is not a separation"
    );
    assert!(
        holon_chem::stream_table(10.0, 0.3, 10, |i, r, e, f, _| t.knot(i, r, e, f)).is_none(),
        "an inverted range is not a grid"
    );
}

#[test]
fn energy_is_conserved_on_the_curve_the_viewer_actually_uses() {
    // A short NVE run on the GENERATED curve. The existing ledger gates run this on the
    // fallback file, so without this one the shipped physics has no conservation gate at
    // all -- and the two curves are not small perturbations of each other: the real one
    // has a 1/R wall where the placeholder has an exponential.
    let mut s = Sim::empty();
    let mut t = PotentialTable::empty();
    assert!(t.begin(KNOTS));
    let meta = holon_chem::stream_table(R_MIN, R_MAX, KNOTS, |i, r, e, f, e2| {
        t.knot(i, r, e, f) && t.knot_curvature(i, e2)
    })
    .expect("curve");
    assert_eq!(t.finish(meta.r_e, meta.d_e, meta.e_asymptote), LoadStatus::Ok);
    s.table = t;
    s.adopt_table_timescale();

    s.boundary = Boundary::Open;
    s.reset(2);
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    s.set_position(0, cx - 1.1, cy);
    s.set_position(1, cx + 1.1, cy);
    s.set_velocity(0, 0.002, 0.001);
    s.set_velocity(1, -0.002, 0.001);
    s.rebase();

    for _ in 0..400 {
        s.step_frame(16);
    }
    let bound = s.drift_bound();
    println!(
        "generated-curve NVE: drift_peak = {:.4e}, bound = {bound:.4e}, ratio = {:.4}, \
         energy gate = {}",
        s.drift_peak,
        s.drift_peak / bound,
        s.energy_gate()
    );
    assert!(
        s.drift_peak <= bound,
        "drift {:.3e} exceeds the derived bound {bound:.3e} on the generated curve",
        s.drift_peak
    );
    assert!(s.energy_gate(), "the energy gate is red on the generated curve");
}

#[test]
fn the_abi_generate_call_fills_the_shared_table() {
    // The ONE test that drives the extern "C" globals, and it does the whole sequence in
    // one function on purpose: the ABI is a single shared static, so two tests touching
    // it in parallel would be testing each other.
    assert_eq!(
        holon_render::holon_table_generate(R_MIN, R_MAX, KNOTS as u32),
        1,
        "the generator did not fill the table"
    );
    assert_eq!(holon_render::holon_table_knots(), KNOTS as u32);
    assert_eq!(holon_render::holon_table_has_curvature(), 1);
    assert!(holon_render::holon_table_residual_alt() > 20.0 * holon_render::holon_table_residual());
    assert!((holon_render::holon_table_r_e() - 1.388694018017776).abs() < 1e-12);
    assert!(holon_render::holon_table_r_min() == R_MIN && holon_render::holon_table_r_max() == R_MAX);

    // The banner's numbers come through the ABI, so they are gated like any other export.
    assert_eq!(
        holon_render::holon_chem_referee_digest(),
        holon_chem::REFEREE_DIGEST
    );
    assert_eq!(holon_render::holon_chem_referee_points(), 492);
    let residual = holon_render::holon_chem_referee_residual();
    assert!(
        residual > 0.0 && residual < 1e-12,
        "the banner would display a residual of {residual:e}, which is not the staked scale"
    );

    // A refused request must be distinguishable from a table refusal, and must not leave
    // a half-filled table behind claiming to be loaded.
    assert_eq!(
        holon_render::holon_table_generate(10.0, 0.3, 100),
        holon_render::GENERATOR_REFUSED
    );
    assert_ne!(
        holon_render::holon_table_status(),
        1,
        "a refused generation left the table reporting Ok"
    );
}
