//! The trimer door, watched refusing.
//!
//! Every leg of `TrimerProvenance::admit` gets a planted defect that fires it, and each
//! plant is one field away from an artifact that IS admitted — so a leg passing here is a
//! leg that discriminates, rather than one that happens to be true of everything.
//!
//! The reason for that shape is the pair door's own history: `converged: false` rode a
//! shipped table through three commits and every gate passed it, not because any gate was
//! lax but because no gate on this side read convergence at any magnitude. A refusal
//! nobody has watched fire is indistinguishable from a refusal that cannot.
//!
//! The positive control is the point of the file as much as the plants are. If
//! `admitted()` ever stops being admitted, every plant below still "passes" — they would
//! all be refused for the wrong reason — so the first test asserts the control is clean
//! and the plants each assert their SPECIFIC refusal rather than merely `is_err()`.

use holon_render::bank::Route;
use holon_render::trimer_bank::{
    AxisRule, SeamRecord, TrimerBank, TrimerProvenance, TrimerRefusal,
    RESOLVABLE_TRIMER_UNCERTAINTY,
};
use holon_chem::trimer::N_NODES;

/// The largest `|dE3|` the synthetic surface below carries. Chosen well under
/// `RESOLVABLE_TRIMER_UNCERTAINTY` in one plant and well over it in the control, so the
/// resolution leg and the feature leg can be fired independently of each other.
const CONTROL_PEAK: f64 = 1.0e-4;

/// A surface whose nodes are all present and finite. The VALUES are synthetic — this file
/// tests the door, not the chemistry — but they are smooth and small, because `finish`
/// measures a curvature envelope from them and a pathological surface would be testing
/// the envelope measurement instead.
fn filled_bank() -> TrimerBank {
    let mut b = TrimerBank::empty();
    b.begin();
    for i in 0..N_NODES {
        let t = i as f64 / N_NODES as f64;
        b.knot(i, CONTROL_PEAK * (1.0 - t) * (1.0 - t));
    }
    b
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
        axis_rule: AxisRule::TauStretchH3,
        region: [3, 3, 13],
        cited_curves: 2,
        declares_converged: false,
        void_count: 0,
        void_named: 0,
        seam: SeamRecord::AcceptedFloor,
        digest: 0x1504_da0f_dead_beef,
        }
}

fn meta() -> holon_chem::trimer::TrimerMeta {
    holon_chem::trimer::TrimerMeta {
        n_nodes: N_NODES,
        nr: holon_chem::trimer::NR,
        nu: holon_chem::trimer::NU,
        r_lo: holon_chem::trimer::R_LO,
        r_hi: holon_chem::trimer::R_HI,
        e_h_atom: holon_chem::trimer::atom_energy(),
        peak: CONTROL_PEAK,
        solves: 0,
    }
}

/// Push the control surface with `prov` and report what the door said.
fn door(prov: TrimerProvenance) -> Result<usize, TrimerRefusal> {
    filled_bank().finish(meta(), prov)
}

// ------------------------------------------------------------------ the positive control

#[test]
fn the_control_artifact_is_admitted() {
    let mut b = filled_bank();
    let r = b.finish(meta(), admitted());
    assert!(
        r.is_ok(),
        "the control artifact was refused ({:?}); every plant below would then pass for \
         the wrong reason",
        r.unwrap_err()
    );
    assert_eq!(b.len(), 1);
    assert!(b.last_refusal.is_none());
    // And it is the surface it says it is, findable under any ordering of its triple.
    assert!(b.find([1, 1, 17]).is_some());
    assert!(b.find([17, 1, 1]).is_some(), "the triple key is not unordered");
    assert!(b.find([1, 1, 1]).is_none(), "a triple that was never loaded was found");
}

// ------------------------------------------------------------------ one plant per leg

#[test]
fn plant_route_undeclared() {
    let mut p = admitted();
    p.route = Route::Undeclared;
    assert_eq!(door(p), Err(TrimerRefusal::RouteUndeclared));
}

#[test]
fn plant_grid_rule_unsupported() {
    // The leg that stops a stretched surface being interpolated on uniform axes. It is
    // the one refusal here that is about ARITHMETIC rather than about a claim: the others
    // refuse an artifact that says too little, this one refuses one this build cannot
    // reproduce the spacing of.
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
    // Also tested AT the bound, because a `<` and a `<=` differ exactly there and the
    // boundary is where a threshold is worth pinning.
    let mut p = admitted();
    p.uncertainty_ha = 1.0;
    assert_eq!(door(p), Err(TrimerRefusal::UncertaintyExceedsResolution));

    let mut at_bound = admitted();
    at_bound.uncertainty_ha = RESOLVABLE_TRIMER_UNCERTAINTY;
    assert_eq!(
        door(at_bound),
        Err(TrimerRefusal::UncertaintyExceedsResolution),
        "an uncertainty exactly AT the resolution bound was admitted; the door must \
         refuse on the boundary, not just past it"
    );
}

#[test]
fn plant_uncertainty_exceeds_feature() {
    // Distinct from the leg above, and the plant has to prove it is: this uncertainty is
    // comfortably INSIDE the schema's resolution and still larger than the feature this
    // particular surface found. A door with only the absolute leg would admit it.
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
    // the fault is the field ARRIVING, whatever it says. Both values are planted, because
    // "converged: false" is the one that actually rode through the pair door for three
    // commits and a gate that only refused `true` would have missed it again.
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
    // artifact declaring the floor it accepts instead must load — a door that demanded
    // the locus would refuse every table the campaign can currently produce, and a gate
    // that has to be loosened the moment it meets working code is the wrong gate.
    for ok in [SeamRecord::Locus, SeamRecord::AcceptedFloor] {
        let mut p = admitted();
        p.seam = ok;
        assert!(door(p).is_ok(), "{ok:?} was refused");
    }
}

#[test]
fn plant_digest_missing() {
    let mut p = admitted();
    p.digest = 0;
    assert_eq!(door(p), Err(TrimerRefusal::DigestMissing));
}

#[test]
fn plant_surface_not_loaded() {
    // Nodes short of the grid. `TrimerTable::finish` reports the table unloaded and the
    // door refuses on that, rather than admitting a provenance that describes a surface
    // with holes in it.
    let mut b = TrimerBank::empty();
    b.begin();
    for i in 0..(N_NODES - 1) {
        b.knot(i, 1.0e-6);
    }
    assert_eq!(b.finish(meta(), admitted()), Err(TrimerRefusal::SurfaceNotLoaded));
    assert!(b.is_empty(), "a refused artifact left a surface behind");
}

#[test]
fn a_finish_with_no_begin_is_refused() {
    let mut b = TrimerBank::empty();
    assert_eq!(b.finish(meta(), admitted()), Err(TrimerRefusal::SurfaceNotLoaded));
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
    // other assertion in this test.
    let mut h3 = admitted();
    h3.z = [1, 1, 1];
    let mut bh = filled_bank();
    assert!(bh.finish(meta(), h3).is_ok());
    b = bh;
    assert!(
        !b.any_heteronuclear(),
        "a shipped H3 surface lifted the fence; the fence is about heteronuclear \
         coverage, not about provenance"
    );

    // And one non-hydrogen centre lifts it.
    let mut bx = filled_bank();
    assert!(bx.finish(meta(), admitted()).is_ok());
    assert!(bx.any_heteronuclear(), "an (H,H,Cl) surface did not lift the fence");
}

#[test]
fn a_refused_artifact_leaves_the_fence_up() {
    // The property that makes the fence worth anything: the only way to lift it is to get
    // past the door. A bad artifact must not lift it as a side effect of having been
    // offered.
    let mut b = filled_bank();
    let mut bad = admitted();
    bad.seam = SeamRecord::Absent;
    assert!(b.finish(meta(), bad).is_err());
    assert!(!b.any_heteronuclear(), "a REFUSED artifact lifted the fence");
    assert!(b.is_empty(), "a refused artifact was stored anyway");
    assert_eq!(b.last_refusal, Some(TrimerRefusal::SeamRecordMissing));
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
    // A reason nobody can read is a boolean with extra steps. Cheap to assert, and it
    // catches a variant added without its sentence.
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
    ];
    let mut seen: Vec<&str> = Vec::new();
    for r in all {
        let p = r.plain();
        assert!(p.len() > 30, "{r:?} has no real explanation");
        assert!(!seen.contains(&p), "{r:?} shares its sentence with another refusal");
        seen.push(p);
    }
}
