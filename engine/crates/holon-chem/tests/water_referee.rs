//! GATE R1 — the (O, H, H) surface against a 50-digit referee, geometry by geometry.
//!
//! # What makes it a gate rather than a comparison
//!
//! The referee (`conformance/atomworld/saturation2_referee.py`, mpmath at 60 working
//! digits reported to 50) shares no line of code with this crate: different language,
//! different arithmetic, different integrals, different eigensolver, and its own dual
//! CI route checking itself before it writes anything down. It shares the MODEL — `Z`,
//! the STO-3G contraction, the minimal-|Sz| sector — and nothing else. So agreement here
//! is two independent implementations of one model landing on the same numbers, which is
//! the only kind of check that catches a transcription error in a formula: a single
//! implementation's self-tests all inherit its mistakes.
//!
//! # The staked set, and why it is result-blind
//!
//! The referee's geometries are a function of the DECLARED domain constants and a fixed
//! integer ladder — a six-rung geometric ladder of O-H sides from the staked floor 0.9 to
//! the truncation radius 14, crossed with itself under `x <= y`, crossed with four staked
//! angles (the closed fence, the collinear edge, and the two third-points between). 84
//! geometries against the prereg's `>= 48`. Nothing in the set consults an energy, a
//! minimum, a bond length or an angle, and the rule is written in the referee's own
//! header so it can be re-derived rather than trusted.
//!
//! # Three things this gate deliberately does
//!
//! * It pins the referee BY DIGEST. A gate that grades against a file it does not
//!   identify is a gate that can be satisfied by editing the file.
//! * It compares in exact decimal (`common::decimal_minus_f64`), so the referee's digits
//!   are not rounded to f64 before the subtraction meant to measure them.
//! * It checks the pinned residual in BOTH directions: too large fails as a regression,
//!   and far too small fails as a stale pin, because a bound left a decade looser than
//!   reality has stopped being evidence about anything.

// The shared referee helpers serve several gates; this one does not need all of them.
#[allow(dead_code)]
mod common;

use common::{decimal_minus_f64, string_array, string_scalar};
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::fnv1a32;
use holon_chem::pair::{atom_energy, pair_point};
use holon_chem::water::{hh_side, ohh_energy};
use holon_chem::{
    WATER_R1_MEASURED_E, WATER_R1_STAKE_E, WATER_REFEREE_DIGEST, WATER_REFEREE_GEOMETRIES,
};
use std::collections::HashMap;
use std::path::PathBuf;

fn referee_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/s2/water_referee.json")
}

fn referee_bytes() -> Vec<u8> {
    std::fs::read(referee_path()).expect("the pinned (O,H,H) referee is present")
}

fn referee_text() -> String {
    String::from_utf8(referee_bytes()).expect("referee file is UTF-8")
}

/// Parse a column of decimals into f64, for the geometry inputs only. The geometry is an
/// INPUT to both implementations and is exact in f64 by construction (the ladder rounds
/// to two decimals and the angles to the declared constants), so parsing it is not the
/// precision loss `decimal_minus_f64` exists to avoid — that applies to the ENERGIES,
/// which are never parsed here.
fn floats(src: &str, key: &str) -> Vec<f64> {
    string_array(src, key)
        .iter()
        .map(|s| s.parse::<f64>().expect("a decimal"))
        .collect()
}

#[test]
fn referee_is_the_pinned_file() {
    let got = fnv1a32(&referee_bytes());
    assert_eq!(
        got, WATER_REFEREE_DIGEST,
        "the pinned (O,H,H) referee has changed (digest {got:#010x}, pinned \
         {WATER_REFEREE_DIGEST:#010x}). If that was deliberate, re-derive the residual \
         constant against the new file rather than re-pinning the digest alone."
    );
    let src = referee_text();
    assert_eq!(
        string_array(&src, "col_x").len(),
        WATER_REFEREE_GEOMETRIES,
        "the referee's geometry count moved"
    );
    assert!(
        WATER_REFEREE_GEOMETRIES >= 48,
        "the prereg stakes at least 48 geometries; this file carries \
         {WATER_REFEREE_GEOMETRIES}"
    );
}

#[test]
fn the_staked_set_spans_the_domain_it_claims_to() {
    // The prereg requires the set to span "compact, bent, linear, stretched,
    // near-boundary". Those labels are computed by the referee from the declared
    // constants and the geometry alone; this asserts each one is actually present, so a
    // ladder change that quietly emptied a family would fail rather than pass narrower.
    let src = referee_text();
    let fam = string_array(&src, "col_family");
    assert_eq!(fam.len(), WATER_REFEREE_GEOMETRIES);
    for want in ["compact", "bent", "linear", "stretched", "near-boundary", "closed"] {
        let n = fam.iter().filter(|f| f.contains(want)).count();
        assert!(n > 0, "the staked set contains no {want} geometry");
    }
}

#[test]
fn the_spin_audit_asserts_where_resolved_and_reports_where_degenerate() {
    // M-PARITY-PROTECT, in the prereg's own words: "asserted where resolved, reported
    // where degenerate".
    //
    // The multiplicity is MEASURED by the referee from <S^2> of its converged vector,
    // and whether it MEANS anything is measured too: the referee solves the Sz = 1
    // sector as well, where the lowest state is by construction the lowest triplet, so
    // the difference is the exact singlet-triplet gap. A resolved gap makes the singlet
    // label a fact about the state; a zero gap makes it a fact about which degenerate
    // component the eigensolver happened to return, and asserting on that would be
    // asserting on the solver.
    //
    // Both cases occur here and both are load-bearing: a bonded geometry is a resolved
    // singlet, and a geometry with one hydrogen at the far edge of the domain is a
    // dissociated OH + H whose singlet and triplet are exactly degenerate. A gate that
    // demanded "singlet everywhere" would fire on correct physics.
    let src = referee_text();
    let two_s = string_array(&src, "col_two_S_OHH");
    let resolved = string_array(&src, "col_spin_resolved");
    let gap = string_array(&src, "col_spin_gap");
    let n = WATER_REFEREE_GEOMETRIES;
    assert_eq!(two_s.len(), n);
    assert_eq!(resolved.len(), n);
    assert_eq!(gap.len(), n);

    let mut n_resolved = 0usize;
    let mut wrong: Vec<usize> = Vec::new();
    for i in 0..n {
        if resolved[i] == "1" {
            n_resolved += 1;
            if two_s[i] != "0" {
                wrong.push(i);
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "the referee measured a non-singlet ground state at geometries {wrong:?}, where the \
         singlet-triplet gap says the state is RESOLVED — the two implementations would then \
         be solving different states and every energy in this file would be meaningless"
    );
    // Both halves of the audit have to be non-empty, or the conditional is decoration:
    // an all-resolved set would never exercise the degenerate branch, and an
    // all-degenerate set would mean the gate asserts nothing at all.
    assert!(
        n_resolved > 0 && n_resolved < n,
        "the spin audit's two branches are not both exercised: {n_resolved} of {n} \
         geometries are spin-resolved"
    );
}

#[test]
fn the_referee_says_where_it_could_not_check_itself() {
    // The referee's own independence check re-solves each geometry in a randomly rotated
    // orbital basis. It cannot run everywhere, and the reason is physical: at a
    // DISSOCIATED geometry the ground state is near-degenerate — oxygen's 3P times two
    // hydrogen doublets — so the Temple bound has no gap to certify against and the
    // rotated route does not converge. Measured, it was still running after twenty minutes
    // on a geometry route A finishes in thirty-two seconds.
    //
    // The referee therefore DECLARES route B unavailable wherever route A's own gap says
    // it cannot be certified, decided before route B is paid for and recorded per
    // geometry. This gate reads that record rather than letting it sit in a field nobody
    // is required to look at: a referee that says which geometries it could not
    // double-check is worth more than one that quietly checked some of them.
    let src = referee_text();
    let dual = string_array(&src, "col_dual_route");
    assert_eq!(dual.len(), WATER_REFEREE_GEOMETRIES);
    let n_dual = dual.iter().filter(|d| d.as_str() == "1").count();
    println!(
        "R1: {n_dual} of {WATER_REFEREE_GEOMETRIES} staked geometries carry the referee's \
         second CI route; {} are single-route, where the ground state is degenerate",
        WATER_REFEREE_GEOMETRIES - n_dual
    );
    // Both branches have to be non-empty or the record is decoration: an all-dual set
    // would mean the skip never fired, and an all-single set would mean the referee never
    // checked itself at all.
    assert!(
        n_dual > WATER_REFEREE_GEOMETRIES / 2,
        "only {n_dual} of {WATER_REFEREE_GEOMETRIES} geometries were double-checked by the \
         referee's second route; that is not an independence check, it is a single \
         implementation with a note"
    );
}

#[test]
fn engine_matches_the_referee_at_every_staked_geometry() {
    let src = referee_text();
    let xs = floats(&src, "col_x");
    let ys = floats(&src, "col_y");
    let us = floats(&src, "col_u");
    let e_ohh = string_array(&src, "col_E_OHH");
    let e_oh_x = string_array(&src, "col_E_OH_x");
    let e_oh_y = string_array(&src, "col_E_OH_y");
    let e_hh_z = string_array(&src, "col_E_HH_z");
    let de3 = string_array(&src, "col_dE3");
    let n = WATER_REFEREE_GEOMETRIES;
    for (name, len) in [
        ("col_y", ys.len()),
        ("col_u", us.len()),
        ("col_E_OHH", e_ohh.len()),
        ("col_dE3", de3.len()),
    ] {
        assert_eq!(len, n, "{name} has the wrong length");
    }

    // The two atom energies are the zero of the whole expansion, so an error in either
    // shifts every dE3 by a constant and nothing downstream would notice.
    let eo = atom_energy(OXYGEN);
    let eh = atom_energy(HYDROGEN);
    let d_eo = decimal_minus_f64(&string_scalar(&src, "col_E_O"), eo).abs();
    let d_eh = decimal_minus_f64(&string_scalar(&src, "col_E_H"), eh).abs();
    assert!(
        d_eo <= WATER_R1_STAKE_E && d_eh <= WATER_R1_STAKE_E,
        "the reference atoms disagree: E(O) by {d_eo:.3e}, E(H) by {d_eh:.3e} hartree"
    );

    // The side ladder takes only six distinct values, so the O-H pair curve is solved
    // once per value rather than once per geometry — 84 solves down to 6.
    let mut oh: HashMap<u64, f64> = HashMap::new();
    let mut oh_at = |r: f64| -> f64 {
        *oh.entry(r.to_bits())
            .or_insert_with(|| pair_point(OXYGEN, HYDROGEN, r).e)
    };

    let mut worst = 0.0f64;
    let mut worst_at = 0usize;
    let mut worst_col = "";
    for i in 0..n {
        let (x, y, u) = (xs[i], ys[i], us[i]);
        let z = hh_side(x, y, u);
        let mine_ohh = ohh_energy(x, y, u);
        let mine_x = oh_at(x);
        let mine_y = oh_at(y);
        let mine_z = pair_point(HYDROGEN, HYDROGEN, z).e;
        let mine_de3 = mine_ohh + eo + 2.0 * eh - mine_x - mine_y - mine_z;
        for (col, refd, mine) in [
            ("E_OHH", &e_ohh[i], mine_ohh),
            ("E_OH_x", &e_oh_x[i], mine_x),
            ("E_OH_y", &e_oh_y[i], mine_y),
            ("E_HH_z", &e_hh_z[i], mine_z),
            ("dE3", &de3[i], mine_de3),
        ] {
            let d = decimal_minus_f64(refd, mine).abs();
            if d > worst {
                worst = d;
                worst_at = i;
                worst_col = col;
            }
        }
    }

    println!(
        "R1: worst engine-vs-referee disagreement {worst:.4e} Ha over \
         {WATER_REFEREE_GEOMETRIES} staked geometries x 5 columns (worst on {worst_col}, \
         geometry {worst_at}: x = {}, y = {}, u = {}); stake {WATER_R1_STAKE_E:.0e}",
        xs[worst_at], ys[worst_at], us[worst_at]
    );
    assert!(
        worst <= WATER_R1_STAKE_E,
        "R1 FIRED: worst disagreement {worst:.3e} hartree (column {worst_col}, geometry \
         {worst_at}: x = {}, y = {}, u = {}) exceeds the staked {WATER_R1_STAKE_E:.0e}",
        xs[worst_at],
        ys[worst_at],
        us[worst_at]
    );
    assert!(
        worst <= WATER_R1_MEASURED_E,
        "the measured R1 residual has regressed to {worst:.3e} hartree (column \
         {worst_col}, geometry {worst_at}); the pin is {WATER_R1_MEASURED_E:.3e}"
    );
    assert!(
        worst > WATER_R1_MEASURED_E / 100.0,
        "the pinned R1 residual {WATER_R1_MEASURED_E:.3e} is more than two decades looser \
         than the measured {worst:.3e}; a bound that far from reality has stopped being \
         evidence. Re-pin it."
    );
}
