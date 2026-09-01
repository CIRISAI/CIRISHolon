//! The trimer door, watched refusing — and watched refusing the REAL artifact.
//!
//! Every leg of `TrimerProvenance::admit` gets a planted defect that fires it, and each
//! plant is one field away from an artifact that IS admitted, so a leg passing here is a
//! leg that discriminates rather than one that happens to be true of everything.
//!
//! The reason for that shape is the pair door's own history: `converged: false` rode a
//! shipped table through three commits and every gate passed it, not because any gate was
//! lax but because no gate on this side read convergence at any magnitude. A refusal
//! nobody has watched fire is indistinguishable from a refusal that cannot.
//!
//! `tests/data/s3_h3_4x4x2.json` is REAL OUTPUT of `s3_tables` at 497cbbd, not a fixture
//! written to match this file's expectations. The tests that read it are the ones that
//! found the three interface gaps reported to `saturation3-mesh`, and they assert the gaps
//! rather than working around them: while the emitter ships no uncertainty, the door
//! refuses its artifact, and that is the designed behaviour rather than a bug in either
//! side.

use holon_render::bank::Route;
use holon_render::trimer_bank::{
    AxisRule, SeamRecord, SurfaceGrid, TrimerBank, TrimerProvenance, TrimerRefusal,
    RESOLVABLE_TRIMER_UNCERTAINTY,
};

/// The largest `|dE3|` the synthetic surface carries.
const CONTROL_PEAK: f64 = 1.0e-4;

/// A small, well-formed grid: uniform-linear on every axis, strictly increasing, with an
/// energy array of exactly `nx * ny * nu`.
fn grid() -> SurfaceGrid {
    let (nx, ny, nu) = (4, 4, 2);
    SurfaceGrid {
        nx,
        ny,
        nu,
        x: (0..nx).map(|i| 1.6 + 0.2 * i as f64).collect(),
        y: (0..ny).map(|i| 1.8 + 0.2 * i as f64).collect(),
        u: (0..nu).map(|i| 0.1 + 0.4 * i as f64).collect(),
        energy: (0..nx * ny * nu)
            .map(|i| CONTROL_PEAK * (i as f64 / (nx * ny * nu) as f64))
            .collect(),
    }
}

/// The provenance an admitted artifact carries: every field declared, the uncertainty an
/// f64-tier residual, the peak the surface's own.
fn admitted() -> TrimerProvenance {
    TrimerProvenance {
        route: Route::Determinant,
        z: [1, 1, 17],
        n_det: 605,
        uncertainty_ha: 1.0e-10,
        peak_ha: CONTROL_PEAK,
        axis_rule: AxisRule::UniformLinear,
        region: [2, 2, 2],
        cited_curves: 2,
        declares_converged: false,
        void_count: 0,
        void_named: 0,
        seam: SeamRecord::AcceptedFloor,
        digest: [0x1504da0f, 1, 2, 3, 4, 5, 6, 7],
    }
}

/// Put the control grid to the door with `prov`.
fn door(prov: TrimerProvenance) -> Result<usize, TrimerRefusal> {
    TrimerBank::empty().commit(grid(), prov)
}

/// Put a MODIFIED grid to the door with a clean provenance.
fn door_grid(g: SurfaceGrid) -> Result<usize, TrimerRefusal> {
    TrimerBank::empty().commit(g, admitted())
}

// ------------------------------------------------------------------ the positive control

#[test]
fn the_control_artifact_is_admitted() {
    let mut b = TrimerBank::empty();
    let r = b.commit(grid(), admitted());
    assert!(
        r.is_ok(),
        "the control artifact was refused ({:?}); every plant below would then pass for \
         the wrong reason",
        r.unwrap_err()
    );
    assert_eq!(b.len(), 1);
    assert!(b.last_refusal.is_none());
    assert!(b.find([1, 1, 17]).is_some());
    assert!(b.find([17, 1, 1]).is_some(), "the triple key is not unordered");
    assert!(b.find([1, 1, 1]).is_none(), "a triple never loaded was found");
}

// ------------------------------------------------- what the artifact SHIPPED

#[test]
fn plant_coordinates_missing() {
    let mut g = grid();
    g.x.clear();
    assert_eq!(door_grid(g), Err(TrimerRefusal::CoordinatesMissing));
}

#[test]
fn plant_coordinate_count_mismatch() {
    // The grid says four x nodes and three arrive. Neither is obviously the right one,
    // which is exactly why this is a refusal rather than a truncation.
    let mut g = grid();
    g.x.pop();
    assert_eq!(door_grid(g), Err(TrimerRefusal::CoordinateCountMismatch));
}

#[test]
fn plant_coordinates_not_monotone() {
    for bad in [f64::NAN, 0.0] {
        let mut g = grid();
        g.y[2] = bad;
        assert_eq!(
            door_grid(g),
            Err(TrimerRefusal::CoordinatesNotMonotone),
            "a {bad} in a coordinate axis was not refused"
        );
    }
    // Equal neighbours are non-monotone too: a zero-width cell has no interior to
    // interpolate in and would divide by its own width.
    let mut g = grid();
    g.x[2] = g.x[1];
    assert_eq!(door_grid(g), Err(TrimerRefusal::CoordinatesNotMonotone));
}

#[test]
fn plant_axis_rule_contradicts_coordinates() {
    // THE LEG THAT MAKES THE DECLARED RULE CHECKABLE. The artifact says its coordinates
    // win where they disagree with its `axis_rule` name; that is only a meaningful
    // promise if a disagreement is caught, because otherwise the name may as well be
    // absent. Here the name says uniform-linear and the coordinates are stretched.
    let mut g = grid();
    g.x = vec![1.6, 1.7, 2.0, 2.6];
    assert_eq!(
        door_grid(g),
        Err(TrimerRefusal::AxisRuleContradictsCoordinates)
    );
}

#[test]
fn plant_angle_cosine_out_of_range() {
    // THE REAL NEAR-MISS, not an invented one. `saturation3-mesh` found while writing the
    // convention down that a neighbouring lane parameterises the same axis as
    // `c = sqrt(1 - cos theta)` over [0.05, 1.4142], while `u` IS the cosine. Handed over
    // in `c` and consumed as `u`, everything past 1.0 has `s = sqrt(1 - u^2)` imaginary,
    // clamped to zero: a silent band of degenerate collinear geometries along the top of
    // the table. The plant uses that lane's actual range rather than a made-up one.
    let mut g = grid();
    g.nu = 3;
    g.u = vec![0.05, 0.7, 1.4142];
    g.energy = vec![CONTROL_PEAK; g.nx * g.ny * g.nu];
    assert_eq!(door_grid(g), Err(TrimerRefusal::AngleCosineOutOfRange));

    // Both ends, because a cosine has two.
    let mut low = grid();
    low.u = vec![-1.5, 0.5];
    assert_eq!(door_grid(low), Err(TrimerRefusal::AngleCosineOutOfRange));

    // And the endpoints themselves are legal: u = -1 is collinear and u = 1 is the
    // degenerate coincident case, both of which a table may legitimately reach. A door
    // that refused its own boundary would refuse a correct artifact.
    let mut edge = grid();
    edge.u = vec![-1.0, 1.0];
    assert!(door_grid(edge).is_ok(), "the legal endpoints of the cosine were refused");
}

#[test]
fn plant_side_length_not_positive() {
    for bad in [0.0, -2.0] {
        let mut g = grid();
        g.x[0] = bad;
        assert_eq!(
            door_grid(g),
            Err(TrimerRefusal::SideLengthNotPositive),
            "an x side of {bad} bohr was not refused"
        );
    }
    // y as well as x — a leg that only checked one axis would pass this file while
    // admitting half the defect.
    let mut g = grid();
    g.y = vec![-3.0, -2.0, -1.0, 1.0];
    assert_eq!(door_grid(g), Err(TrimerRefusal::SideLengthNotPositive));
}

#[test]
fn the_third_side_is_derived_and_the_convention_closes() {
    // Not a door test: a check that the convention I encoded is the one they stated, and
    // that it is self-consistent. Their centres are [0,0,0], [x,0,0], [y*u, y*s, 0] with
    // s = sqrt(1 - u^2); the law of cosines then gives the unstored third side. If those
    // two disagreed, every geometry the evaluator ever builds would be wrong, so it is
    // worth six lines to show they do not.
    for &(x, y, u) in &[(1.6_f64, 1.8_f64, 0.1_f64), (2.2, 2.4, 0.5), (3.0, 1.5, -0.8)] {
        let s = (1.0 - u * u).sqrt();
        let apex = [0.0, 0.0];
        let b = [x, 0.0];
        let c = [y * u, y * s];
        let side_23 = ((b[0] - c[0]).powi(2) + (b[1] - c[1]).powi(2)).sqrt();
        let by_law = (x * x + y * y - 2.0 * x * y * u).sqrt();
        assert!(
            (side_23 - by_law).abs() < 1e-12,
            "the stated centres and the law of cosines disagree at ({x}, {y}, {u}): \
             {side_23} vs {by_law}"
        );
        // And the two stored sides are the ones measured from the apex, not from each
        // other — the thing that would be silently transposed if x and y were swapped.
        let side_1 = ((b[0] - apex[0]).powi(2) + (b[1] - apex[1]).powi(2)).sqrt();
        let side_2 = ((c[0] - apex[0]).powi(2) + (c[1] - apex[1]).powi(2)).sqrt();
        assert!((side_1 - x).abs() < 1e-12 && (side_2 - y).abs() < 1e-12);
    }
}

#[test]
fn plant_energy_count_mismatch() {
    let mut g = grid();
    g.energy.pop();
    assert_eq!(door_grid(g), Err(TrimerRefusal::EnergyCountMismatch));
}

// ------------------------------------------------- what the artifact SAYS

#[test]
fn plant_route_undeclared() {
    let mut p = admitted();
    p.route = Route::Undeclared;
    assert_eq!(door(p), Err(TrimerRefusal::RouteUndeclared));
}

#[test]
fn plant_grid_rule_unsupported() {
    let mut p = admitted();
    p.axis_rule = AxisRule::Undeclared;
    assert_eq!(door(p), Err(TrimerRefusal::GridRuleUnsupported));
}

#[test]
fn plant_region_shape_missing() {
    for axis in 0..3 {
        let mut p = admitted();
        p.region[axis] = 0;
        assert_eq!(
            door(p),
            Err(TrimerRefusal::RegionShapeMissing),
            "a zero on region axis {axis} was not refused"
        );
    }
}

#[test]
fn plant_domain_uncited() {
    let mut p = admitted();
    p.cited_curves = 0;
    assert_eq!(door(p), Err(TrimerRefusal::DomainUncited));
}

#[test]
fn plant_uncertainty_missing() {
    for absent in [0.0, -0.0, f64::NAN] {
        let mut p = admitted();
        p.uncertainty_ha = absent;
        assert_eq!(
            door(p),
            Err(TrimerRefusal::UncertaintyMissing),
            "uncertainty {absent} was not treated as absent"
        );
    }
}

#[test]
fn plant_uncertainty_exceeds_resolution() {
    // A whole hartree is the magnitude the pair door could not refuse until 2026-08-30.
    // Also tested AT the bound, because `<` and `<=` differ exactly there.
    let mut p = admitted();
    p.uncertainty_ha = 1.0;
    assert_eq!(door(p), Err(TrimerRefusal::UncertaintyExceedsResolution));

    let mut at_bound = admitted();
    at_bound.uncertainty_ha = RESOLVABLE_TRIMER_UNCERTAINTY;
    assert_eq!(
        door(at_bound),
        Err(TrimerRefusal::UncertaintyExceedsResolution),
        "an uncertainty exactly AT the resolution bound was admitted"
    );
}

#[test]
fn plant_uncertainty_exceeds_feature() {
    // Distinct from the leg above, and the plant proves it: this uncertainty is
    // comfortably INSIDE the schema's resolution and still larger than the feature this
    // surface found. A door with only the absolute leg would admit it.
    let mut p = admitted();
    p.peak_ha = 1.0e-8;
    p.uncertainty_ha = 1.0e-7;
    assert!(
        p.uncertainty_ha < RESOLVABLE_TRIMER_UNCERTAINTY,
        "the plant must clear the absolute leg or it tests the wrong one"
    );
    assert_eq!(door(p), Err(TrimerRefusal::UncertaintyExceedsFeature));
}

#[test]
fn plant_converged_field_present() {
    // The refusal that runs the other way: this schema HAS no top-level `converged`, so
    // the fault is the field ARRIVING, whatever it says.
    let mut p = admitted();
    p.declares_converged = true;
    assert_eq!(door(p), Err(TrimerRefusal::ConvergedFieldPresent));
}

#[test]
fn plant_voids_counted_not_named() {
    let mut p = admitted();
    p.void_count = 7;
    p.void_named = 0;
    assert_eq!(door(p), Err(TrimerRefusal::VoidsCountedNotNamed));

    // ...and a VOID that IS named passes, so the leg is about naming rather than about
    // VOIDs existing. A door that refused every VOID would push a producer to hide them.
    let mut named = admitted();
    named.void_count = 7;
    named.void_named = 7;
    assert!(door(named).is_ok(), "a named VOID was refused");
}

#[test]
fn plant_seam_record_missing() {
    let mut p = admitted();
    p.seam = SeamRecord::Absent;
    assert_eq!(door(p), Err(TrimerRefusal::SeamRecordMissing));

    // Both honest answers are accepted. The campaign still owes the seam LOCUS, so an
    // artifact declaring the floor it accepts instead must load — a door demanding the
    // locus would refuse every table the campaign can currently produce, and a gate that
    // has to be loosened the moment it meets working code is the wrong gate.
    for ok in [SeamRecord::Locus, SeamRecord::AcceptedFloor] {
        let mut p = admitted();
        p.seam = ok;
        assert!(door(p).is_ok(), "{ok:?} was refused");
    }
}

#[test]
fn plant_digest_missing() {
    let mut p = admitted();
    p.digest = [0; 8];
    assert_eq!(door(p), Err(TrimerRefusal::DigestMissing));
}

#[test]
fn a_finish_with_no_begin_is_refused() {
    let mut b = TrimerBank::empty();
    assert_eq!(b.finish(admitted()), Err(TrimerRefusal::SurfaceNotLoaded));
}

// ------------------------------------------------------------------ the fence

#[test]
fn the_fence_lifts_only_for_a_heteronuclear_surface() {
    // THE DELIVERABLE. `holon_trimer_h_only()` is the negation of this, and both viewers
    // print their disclaimer from it, so this is the assertion that the sentence on the
    // page changes when — and only when — the successor actually lands.
    let mut b = TrimerBank::empty();
    assert!(!b.any_heteronuclear(), "an empty bank lifted the fence");

    // The discriminator: a SHIPPED H3 surface is a legitimate artifact and must NOT lift
    // the fence, because the fence is about heteronuclear coverage rather than about
    // where a surface came from. Without this leg, "any surface loaded" would pass every
    // other assertion here.
    let mut h3 = admitted();
    h3.z = [1, 1, 1];
    assert!(b.commit(grid(), h3).is_ok());
    assert!(
        !b.any_heteronuclear(),
        "a shipped H3 surface lifted the fence; the fence is about heteronuclear \
         coverage, not about provenance"
    );

    // And one non-hydrogen centre lifts it.
    assert!(b.commit(grid(), admitted()).is_ok());
    assert!(b.any_heteronuclear(), "an (H,H,Cl) surface did not lift the fence");
}

#[test]
fn a_refused_artifact_leaves_the_fence_up() {
    // The property that makes the fence worth anything: the only way to lift it is to get
    // past the door.
    let mut b = TrimerBank::empty();
    let mut bad = admitted();
    bad.seam = SeamRecord::Absent;
    assert!(b.commit(grid(), bad).is_err());
    assert!(!b.any_heteronuclear(), "a REFUSED artifact lifted the fence");
    assert!(b.is_empty(), "a refused artifact was stored anyway");
    assert_eq!(b.last_refusal, Some(TrimerRefusal::SeamRecordMissing));
}

// ------------------------------------------------------------------ the real artifact

/// The smallest thing that reads a number out of the real JSON. Not a parser: the file's
/// shape is fixed and this only has to find scalars and flat arrays, and adding a JSON
/// dependency to a crate whose whole size claim is "no dependencies" would cost more than
/// it buys.
fn scalar_after(src: &str, key: &str) -> Option<String> {
    let at = src.find(&format!("\"{key}\":"))? + key.len() + 3;
    let rest = src[at..].trim_start();
    let end = rest
        .find(|c: char| c == ',' || c == '}' || c == ']' || c == '\n')
        .unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

fn array_after(src: &str, key: &str) -> Vec<f64> {
    let Some(at) = src.find(&format!("\"{key}\":")) else {
        return Vec::new();
    };
    let rest = &src[at..];
    let Some(open) = rest.find('[') else {
        return Vec::new();
    };
    let Some(close) = rest[open..].find(']') else {
        return Vec::new();
    };
    rest[open + 1..open + close]
        .split(',')
        .filter_map(|t| t.trim().parse::<f64>().ok())
        .collect()
}

fn real_artifact() -> String {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/s3_h3_4x4x2.json"
    );
    std::fs::read_to_string(path).expect("the committed s3_tables artifact")
}

#[test]
fn the_real_artifact_ships_the_grid_this_door_needs() {
    // The half of the interface that WORKS, asserted so a regression in it is visible
    // separately from the half that does not. These are the fields option (b) added at
    // 497cbbd after the spans-and-counts question, and they are exactly what lets a
    // consumer interpolate a grid whose spacing is not its own.
    let src = real_artifact();
    let x = array_after(&src, "x_nodes");
    let y = array_after(&src, "y_nodes");
    let u = array_after(&src, "u_nodes");
    let e = array_after(&src, "energy_hartree");
    assert_eq!((x.len(), y.len(), u.len()), (4, 4, 2), "coordinates are shipped");
    assert_eq!(e.len(), 32, "energy column is nx*ny*nu");
    assert!(x.windows(2).all(|w| w[1] > w[0]), "x is strictly increasing");
    assert!(
        AxisRule::UniformLinear.matches(&x) && AxisRule::UniformLinear.matches(&y),
        "the declared uniform-linear rule does not match the shipped coordinates"
    );
    assert!(src.contains("\"seams\""), "the seam record is present");
    assert!(src.contains("\"accepted_floor\""), "the seam floor is declared");
    assert!(src.contains("\"region\""), "the region shape is present");
    assert!(src.contains("\"digest\""), "the merge digest is present");
    // The schema's own rule, and the emitter honours it. Stated PRECISELY, because a
    // naive `!src.contains("converged")` fails on a correct artifact: `exit_histogram`
    // carries a `converged` COUNT, and that field is the whole point of the histogram —
    // H3 reads 32 there while the heavy systems will read stagnated everywhere. The
    // refusal is about a TOP-LEVEL boolean, so the assertion has to be about location.
    let histogram = src.find("\"exit_histogram\"").expect("exit_histogram is present");
    let histogram_end = src[histogram..]
        .find('}')
        .map(|e| histogram + e)
        .expect("exit_histogram closes");
    for (at, _) in src.match_indices("\"converged\"") {
        assert!(
            at > histogram && at < histogram_end,
            "a `converged` field appears outside exit_histogram (at byte {at}); this \
             schema has no top-level one on purpose"
        );
    }
}

#[test]
fn the_real_artifact_is_refused_for_want_of_an_uncertainty() {
    // THE GAP, asserted rather than worked around.
    //
    // `s3_tables` at 497cbbd emits no `uncertainty_hartree` and has no flag to supply
    // one, so the weighed-uncertainty leg the lead required the gate to have finds
    // nothing to weigh — and the door refuses. That is the DESIGNED behaviour of a door
    // built to refuse an absent bound rather than read it as a perfect one, and this test
    // pins it so that the day the emitter grows the field, this test fails and says so.
    let src = real_artifact();
    assert!(
        scalar_after(&src, "uncertainty_hartree").is_none(),
        "the emitter now ships an uncertainty; this door can weigh it, and this test \
         should be replaced by one that admits the real artifact"
    );

    let mut b = TrimerBank::empty();
    let x = array_after(&src, "x_nodes");
    let y = array_after(&src, "y_nodes");
    let u = array_after(&src, "u_nodes");
    let g = SurfaceGrid {
        nx: x.len(),
        ny: y.len(),
        nu: u.len(),
        x,
        y,
        u,
        energy: array_after(&src, "energy_hartree"),
    };
    // Everything the artifact DOES declare, read from the file; the two it does not are
    // left at their refusing defaults rather than invented.
    let prov = TrimerProvenance {
        route: Route::Determinant,
        z: [1, 1, 1],
        n_det: 0,
        uncertainty_ha: 0.0, // ABSENT in the artifact
        peak_ha: 0.0,
        axis_rule: AxisRule::UniformLinear,
        region: [2, 2, 2],
        cited_curves: 0, // ABSENT in the artifact
        declares_converged: false,
        void_count: 0,
        void_named: 0,
        seam: SeamRecord::AcceptedFloor,
        digest: [0xe763ba08, 1, 2, 3, 4, 5, 6, 7],
        ..TrimerProvenance::undeclared()
    };
    // The grid legs pass — the artifact's shape and coordinates are sound — and it is
    // refused on the first thing it does not say. `DomainUncited` comes before the
    // uncertainty in the schema's own order, so that is the one that fires.
    assert_eq!(
        b.commit(g, prov),
        Err(TrimerRefusal::DomainUncited),
        "the real artifact was not refused on a missing declaration"
    );
    assert!(b.is_empty());
}

// ------------------------------------------------------------------ the pinning

#[test]
fn plant_resolution_pinning() {
    // The trimer door's absolute bound IS the pair door's, deliberately: the three-body
    // term enters the same ledger as the pair term, so "too small for this app to read"
    // must mean one thing in both doors. Asserted rather than commented, so unpinning
    // them is announced here instead of being discovered as a drift between two gates.
    assert_eq!(
        RESOLVABLE_TRIMER_UNCERTAINTY,
        holon_chem::pair::WELL_MIN_DEPTH,
        "the trimer door's resolution bound has been unpinned from the pair door's"
    );
}

#[test]
fn every_refusal_says_something_specific() {
    use TrimerRefusal::*;
    let all = [
        RouteUndeclared,
        GridRuleUnsupported,
        RegionShapeMissing,
        DomainUncited,
        UncertaintyMissing,
        UncertaintyExceedsResolution,
        UncertaintyExceedsFeature,
        ConvergedFieldPresent,
        VoidsCountedNotNamed,
        SeamRecordMissing,
        DigestMissing,
        SurfaceNotLoaded,
        CoordinatesMissing,
        CoordinateCountMismatch,
        CoordinatesNotMonotone,
        AxisRuleContradictsCoordinates,
        EnergyCountMismatch,
        AngleCosineOutOfRange,
        SideLengthNotPositive,
    ];
    let mut seen: Vec<&str> = Vec::new();
    for r in all {
        let p = r.plain();
        assert!(p.len() > 30, "{r:?} has no real explanation");
        assert!(!seen.contains(&p), "{r:?} shares its sentence with another refusal");
        seen.push(p);
    }
}
