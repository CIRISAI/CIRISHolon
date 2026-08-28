//! THE REFEREE GATE: this f64 implementation against a 50-digit one, point by point.
//!
//! # What makes it a gate rather than a comparison
//!
//! The referee (`atom-core`'s `h2_core.py`, mpmath at 60 working digits, reported to 50)
//! shares NOTHING with this crate except the model definition — not a line of code, not
//! a language, not an arithmetic. It computes the same six-decimal STO-3G contraction,
//! the same closed-form Gaussian integrals, the same 2x2 singlet CI, and it checks
//! itself against a second brute-force determinant route at 50 digits before it writes
//! anything down. So agreement here is two independent implementations of one model
//! landing on the same numbers, which is the only kind of check that can catch a
//! transcription error in a formula: a single implementation's self-tests all inherit
//! its mistakes.
//!
//! # Three things this gate deliberately does
//!
//! * It pins the referee BY DIGEST. A gate that grades against a file it does not
//!   identify is a gate that can be satisfied by editing the file.
//! * It compares in exact decimal (see `common::decimal_minus_f64`), so the referee's
//!   digits are not rounded to f64 before the subtraction that is supposed to measure
//!   them.
//! * It checks the pinned residual constants in BOTH directions: too large fails as a
//!   regression, and far too small fails as a stale pin, because a bound left a decade
//!   looser than reality stops being evidence about anything.

mod common;

use common::{decimal_minus_f64, referee_bytes, referee_text, string_array, string_scalar, worst};
use holon_chem::{
    equilibrium, fnv1a32, h2_point, REFEREE_DIGEST, REFEREE_GRID_POINTS, REFEREE_MEASURED_D_E,
    REFEREE_MEASURED_E, REFEREE_MEASURED_E2, REFEREE_MEASURED_F, REFEREE_MEASURED_R_E,
    REFEREE_STAKE_E,
};

#[test]
fn referee_is_the_pinned_file() {
    let bytes = referee_bytes();
    let got = fnv1a32(&bytes);
    assert_eq!(
        got, REFEREE_DIGEST,
        "the pinned referee curve has changed (digest {got:#010x}, pinned {REFEREE_DIGEST:#010x}). \
         If that was deliberate, re-derive the residual constants against the new file rather \
         than re-pinning the digest alone."
    );
    let src = referee_text();
    assert_eq!(
        string_array(&src, "R_grid_bohr").len(),
        REFEREE_GRID_POINTS
    );
}

#[test]
fn engine_curve_matches_the_referee_at_every_separation() {
    let src = referee_text();
    let r_ref = string_array(&src, "R_grid_bohr");
    let e_ref = string_array(&src, "E_hartree");
    let f_ref = string_array(&src, "F_hartree_per_bohr");
    let e2_ref = string_array(&src, "E2_hartree_per_bohr2");
    assert_eq!(r_ref.len(), REFEREE_GRID_POINTS);

    // Evaluate at the referee's own separations, parsed to the nearest f64. The
    // separation itself therefore differs from the referee's by up to half an ulp, which
    // moves E by |dE/dR| * 5.6e-17 <= 6e-16 hartree at the steepest point on the curve.
    // That is a floor under everything below and it is 8x under the pinned bound, so the
    // residuals reported here are dominated by arithmetic rather than by the grid.
    let rs: Vec<f64> = r_ref.iter().map(|s| s.parse::<f64>().unwrap()).collect();
    let pts: Vec<_> = rs.iter().map(|&r| h2_point(r)).collect();

    let (de, i_e) = worst(&e_ref, &pts.iter().map(|p| p.e).collect::<Vec<_>>());
    let (df, i_f) = worst(&f_ref, &pts.iter().map(|p| p.f).collect::<Vec<_>>());
    let (d2, i_2) = worst(&e2_ref, &pts.iter().map(|p| p.e2).collect::<Vec<_>>());

    println!("referee gate over {} separations:", rs.len());
    println!("  max |dE|  = {de:.4e} hartree        at R = {}", rs[i_e]);
    println!("  max |dF|  = {df:.4e} hartree/bohr   at R = {}", rs[i_f]);
    println!("  max |dE2| = {d2:.4e} hartree/bohr^2 at R = {}", rs[i_2]);

    assert!(
        de <= REFEREE_STAKE_E,
        "THE STAKE FIRED: max |dE| = {de:.4e} > {REFEREE_STAKE_E:.0e} hartree at R = {}",
        rs[i_e]
    );
    assert!(
        de <= REFEREE_MEASURED_E,
        "energy residual regressed: {de:.4e} > pinned {REFEREE_MEASURED_E:.1e}"
    );
    assert!(
        df <= REFEREE_MEASURED_F,
        "force residual regressed: {df:.4e} > pinned {REFEREE_MEASURED_F:.1e}"
    );
    assert!(
        d2 <= REFEREE_MEASURED_E2,
        "curvature residual regressed: {d2:.4e} > pinned {REFEREE_MEASURED_E2:.1e}"
    );

    // The other direction. A pin left far looser than the measurement stops being
    // evidence, and the viewer displays this number as the residual.
    assert!(
        REFEREE_MEASURED_E <= 10.0 * de,
        "REFEREE_MEASURED_E is stale: pinned {REFEREE_MEASURED_E:.1e} against a measured \
         {de:.4e}; tighten the pin to what the code actually achieves"
    );
    assert!(
        REFEREE_MEASURED_F <= 10.0 * df,
        "REFEREE_MEASURED_F is stale: pinned {REFEREE_MEASURED_F:.1e} against {df:.4e}"
    );
    assert!(
        REFEREE_MEASURED_E2 <= 10.0 * d2,
        "REFEREE_MEASURED_E2 is stale: pinned {REFEREE_MEASURED_E2:.1e} against {d2:.4e}"
    );
}

#[test]
fn equilibrium_and_asymptote_match_the_referee() {
    let src = referee_text();
    let (r_e, d_e, e_at_r_e) = equilibrium();
    let asym = holon_chem::asymptote();
    let h_atom = holon_chem::h_atom_energy();

    let d_re = decimal_minus_f64(&string_scalar(&src, "R_e"), r_e).abs();
    let d_de = decimal_minus_f64(&string_scalar(&src, "D_e"), d_e).abs();
    let d_ea = decimal_minus_f64(&string_scalar(&src, "E_asymptote"), asym).abs();
    let d_eh = decimal_minus_f64(&string_scalar(&src, "E_H_atom"), h_atom).abs();
    let d_er = decimal_minus_f64(&string_scalar(&src, "E_at_R_e"), e_at_r_e).abs();

    println!("  |dR_e|         = {d_re:.4e} bohr      (mine {r_e:.17})");
    println!("  |dD_e|         = {d_de:.4e} hartree   (mine {d_e:.17})");
    println!("  |dE_asymptote| = {d_ea:.4e} hartree   (mine {asym:.17})");
    println!("  |dE_H_atom|    = {d_eh:.4e} hartree   (mine {h_atom:.17})");
    println!("  |dE(R_e)|      = {d_er:.4e} hartree   (mine {e_at_r_e:.17})");

    assert!(
        d_re <= REFEREE_MEASURED_R_E,
        "R_e residual {d_re:.4e} > pinned {REFEREE_MEASURED_R_E:.1e}"
    );
    assert!(
        REFEREE_MEASURED_R_E <= 10.0 * d_re,
        "REFEREE_MEASURED_R_E is stale: pinned {REFEREE_MEASURED_R_E:.1e} against {d_re:.4e}"
    );
    assert!(
        d_de <= REFEREE_MEASURED_D_E,
        "D_e residual {d_de:.4e} > pinned {REFEREE_MEASURED_D_E:.1e}"
    );
    assert!(
        REFEREE_MEASURED_D_E <= 10.0 * d_de,
        "REFEREE_MEASURED_D_E is stale: pinned {REFEREE_MEASURED_D_E:.1e} against {d_de:.4e}"
    );
    assert!(
        d_ea <= REFEREE_MEASURED_E,
        "asymptote residual {d_ea:.4e} > pinned {REFEREE_MEASURED_E:.1e}"
    );
    assert!(
        d_eh <= REFEREE_MEASURED_E,
        "H-atom residual {d_eh:.4e} > pinned {REFEREE_MEASURED_E:.1e}"
    );
    assert!(
        d_er <= REFEREE_MEASURED_E,
        "E(R_e) residual {d_er:.4e} > pinned {REFEREE_MEASURED_E:.1e}"
    );
}

#[test]
fn the_contraction_is_the_referees_contraction() {
    // The cheapest single number that catches a mistyped exponent or coefficient: the
    // tabulated STO-3G data does not normalise exactly, and how far off it is is a
    // fingerprint of the six decimals rather than of the code that uses them.
    let src = referee_text();
    let raw = holon_chem::sto3g::sto3g_hydrogen().raw_norm;
    let d = decimal_minus_f64(&string_scalar(&src, "contraction_raw_norm"), raw).abs();
    println!("  <chi|chi> before renormalisation = {raw:.17}, |d| = {d:.3e}");
    // Three ulp of 1.0. The referee sums the same nine terms at 60 digits; f64 cannot do
    // better than a few ulp on a nine-term sum, and anything ABOVE this scale would be a
    // different contraction rather than a different rounding.
    assert!(
        d < 5e-16,
        "contraction raw norm differs from the referee by {d:.3e}, which is too large to \
         be rounding: check H_EXPONENTS / H_COEFFS"
    );
}

#[test]
fn the_stake_is_stated_where_the_gate_can_see_it() {
    // A stake nobody can find is not a stake. This fails if someone loosens the
    // contract constant without saying so in the report the brief asks for.
    assert_eq!(
        REFEREE_STAKE_E, 1e-12,
        "the staked pointwise bound was changed; that is a decision to report, not a \
         constant to edit"
    );
}
