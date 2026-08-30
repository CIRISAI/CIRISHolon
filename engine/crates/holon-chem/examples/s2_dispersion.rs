//! Why the (O, H, H) tail is ALGEBRAIC, and why the first explanation for it was wrong.
//!
//! # The prediction, staked before the measurement — and FIRED
//!
//! `examples/s2_domain.rs` found that past `b = 14` the worst point on the truncation
//! shell stops being the near-collinear chain and becomes a stretched hydrogen MOLECULE
//! with the oxygen far away, falling far too slowly for an exponential tail. The
//! explanation on offer was DISPERSION, and it made two predictions that could be wrong:
//!
//! 1. **`dE3` falls as `R^-6`** in the oxygen's distance from the H2's centre of mass — the
//!    dipole-dipole dispersion law. Staked at `-6` in [`PREDICTED_SLOPE`] before this file
//!    was run.
//! 2. **No PAIR term carries any of it.** Hydrogen's STO-3G shell is a single function, so
//!    an isolated H has no spatial virtual orbital to excite into and the O-H pair's full
//!    CI has no dispersion available to it at all.
//!
//! **(1) FIRED.** The measured exponent is `-5.01` collinear and `-4.93` broadside over
//! the full range, and on the clean tail (`R > 13` bohr, where the exponential overlap
//! term has died) the successive two-point exponents are 5.016, 5.012, 5.008, 5.007 —
//! converging on 5, not 6. `R^-5` is the classical QUADRUPOLE-QUADRUPOLE law, not a
//! dispersion law.
//!
//! **(2) held**, and for a reason that also explains (1)'s replacement: an isolated
//! hydrogen atom is spherical in this basis and carries no quadrupole, while the H2 BOND
//! does. So the O-H2 quadrupole interaction has no pair to live in and appears for the
//! first time at order three. Measured, `V2_OH` at 12 bohr is 7.4e-14 while the triple at
//! the same separation holds 3.1e-6.
//!
//! # The discriminator, and what it actually said
//!
//! The quadrupole reading predicts something that can be wrong and is cheap: replace the
//! oxygen with NEON, whose 2p shell is closed and spherical and therefore carries no
//! quadrupole. Part 4 does that, and the answer is sharper than the prediction. Neon has
//! no algebraic tail AT ALL — `dE3` is 3.5e-8 at 8 bohr, 8.0e-12 at 10, and in the f64
//! cancellation floor (1e-13) from 12 bohr out. So removing the open-shell quadrupole
//! removes the whole algebraic sector, which is what the reading requires; and the `R^-6`
//! dispersion channel that should have been left behind is BELOW THE FLOOR in this basis,
//! which means dispersion was never the right story at any separation this campaign can
//! resolve.
//!
//! # What this is NOT
//!
//! Not a claim about nature: a minimal basis is not where multipole moments are quoted
//! from, and no coefficient appears here. What is measured is the SCALING and the SECTOR,
//! both properties of the model. And not a gate — it is the explanation of a gate's shape,
//! which is why it is an example. What it changed is `R_HI`: an algebraic tail is why the
//! truncation radius is 15 bohr and not the 14 an exponential reading would have licensed.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_dispersion
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{Species, HYDROGEN, NEON, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};

/// THE STAKED EXPONENT, kept at its staked value now that it has fired. The measurement
/// that replaced it is in the header; re-pinning this to -5 would delete the evidence
/// that a prediction was made and missed.
const PREDICTED_SLOPE: f64 = -6.0;

fn c3(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

/// `dE3` for oxygen at the origin and a hydrogen molecule of bond length `z` placed with
/// its near hydrogen at `x` along `+x`, collinear.
///
/// The bond length is the molecule's OWN in-model equilibrium, located rather than
/// quoted, and it is held fixed across the sweep: a scan that let it relax would be
/// measuring the relaxation as well as the separation.
fn de3_collinear(x: f64, z: f64, e_o: f64, e_h: f64) -> f64 {
    de3_collinear_with(OXYGEN, x, z, e_o, e_h)
}

/// The same, for an arbitrary heavy partner — the discriminator of part 4.
fn de3_collinear_with(a: Species, x: f64, z: f64, e_a: f64, e_h: f64) -> f64 {
    let y = x + z;
    let e3 = solve_geometry(
        &[a, HYDROGEN, HYDROGEN],
        vec![c3(0.0, 0.0, 0.0), c3(x, 0.0, 0.0), c3(y, 0.0, 0.0)],
    )
    .e
    .v;
    e3 + e_a + 2.0 * e_h
        - pair_point(a, HYDROGEN, x).e
        - pair_point(a, HYDROGEN, y).e
        - pair_point(HYDROGEN, HYDROGEN, z).e
}

/// The same molecule broadside on: the oxygen sits on the H2's perpendicular bisector at
/// distance `d` from its centre.
fn de3_broadside(d: f64, z: f64, e_o: f64, e_h: f64) -> f64 {
    let h = 0.5 * z;
    let r = (d * d + h * h).sqrt();
    let e3 = solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![c3(0.0, 0.0, 0.0), c3(d, h, 0.0), c3(d, -h, 0.0)],
    )
    .e
    .v;
    e3 + e_o + 2.0 * e_h - 2.0 * pair_point(OXYGEN, HYDROGEN, r).e
        - pair_point(HYDROGEN, HYDROGEN, z).e
}

/// Least-squares slope of `log|y|` against `log x`.
fn slope(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    let (lx, ly): (Vec<f64>, Vec<f64>) = (
        xs.iter().map(|v| v.ln()).collect(),
        ys.iter().map(|v| v.abs().ln()).collect(),
    );
    let (mx, my) = (lx.iter().sum::<f64>() / n, ly.iter().sum::<f64>() / n);
    let num: f64 = lx.iter().zip(&ly).map(|(a, b)| (a - mx) * (b - my)).sum();
    let den: f64 = lx.iter().map(|a| (a - mx) * (a - mx)).sum();
    num / den
}

fn main() {
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    println!("# E(O) = {e_o:.12}  E(H) = {e_h:.12}");
    println!("# staked before running: log-log slope = {PREDICTED_SLOPE}\n");

    // The hydrogen molecule's own in-model equilibrium, located rather than quoted.
    let mut z = 1.4f64;
    for _ in 0..60 {
        let p = pair_point(HYDROGEN, HYDROGEN, z);
        // Newton on the force: f = -dE/dr, so a root of f is the minimum.
        z += p.f / p.e2.max(1e-6);
        if p.f.abs() < 1e-13 {
            break;
        }
    }
    println!("# H2's in-model equilibrium: {z:.9} bohr\n");

    println!("## 1 — the scaling, collinear O...H-H");
    println!(
        "   {:>6} {:>10} {:>16} {:>16} {:>14}",
        "x", "R_centre", "dE3", "V2_OH(x)", "dE3 x R^6"
    );
    let mut rs = Vec::new();
    let mut ds = Vec::new();
    for x in [8.0f64, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0] {
        let d = de3_collinear(x, z, e_o, e_h);
        let rc = x + 0.5 * z;
        let v2 = pair_point(OXYGEN, HYDROGEN, x).e - e_o - e_h;
        println!(
            "   {x:>6.1} {rc:>10.3} {d:>16.6e} {v2:>16.3e} {:>14.6e}",
            d * rc.powi(6)
        );
        rs.push(rc);
        ds.push(d);
    }
    let s1 = slope(&rs, &ds);
    println!("\n   measured log-log slope = {s1:.4}   (staked {PREDICTED_SLOPE})");

    println!("\n## 2 — the same molecule broadside on");
    println!("   {:>6} {:>16} {:>14}", "d", "dE3", "dE3 x d^6");
    let mut rs2 = Vec::new();
    let mut ds2 = Vec::new();
    for d in [8.0f64, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0] {
        let v = de3_broadside(d, z, e_o, e_h);
        println!("   {d:>6.1} {v:>16.6e} {:>14.6e}", v * d.powi(6));
        rs2.push(d);
        ds2.push(v);
    }
    let s2 = slope(&rs2, &ds2);
    println!("\n   measured log-log slope = {s2:.4}   (staked {PREDICTED_SLOPE})");
    println!(
        "\n   anisotropy at d = 12: collinear / broadside = {:.3}",
        (de3_collinear(12.0 - 0.5 * z, z, e_o, e_h) / de3_broadside(12.0, z, e_o, e_h)).abs()
    );

    println!("\n## 3 — THE SECTOR: no pair term carries any of it");
    println!(
        "   Hydrogen's STO-3G shell is ONE function, so an isolated H has no spatial\n   \
         virtual orbital and the O-H pair's full CI has no dispersion available to it.\n   \
         The column above is the check: V2_OH at these separations against dE3 at the\n   \
         same ones. A pair term that carried dispersion would fall as R^-6 beside it."
    );
    let mut v2s = Vec::new();
    for &x in &[8.0f64, 10.0, 12.0, 14.0, 16.0] {
        v2s.push(pair_point(OXYGEN, HYDROGEN, x).e - e_o - e_h);
    }
    println!("\n   V2_OH log-log slope = {:.3} (meaningless if the values are at the f64\n   \
         cancellation floor, which is the point: they are)", slope(&[8.0, 10.0, 12.0, 14.0, 16.0], &v2s));
    println!(
        "   largest |V2_OH| over that range = {:.3e} Ha, against |dE3| = {:.3e} Ha",
        v2s.iter().fold(0.0f64, |m, v| m.max(v.abs())),
        ds[0].abs()
    );

    // ------------------------------------------------------------------ the discriminator
    //
    // The staked -6 FIRED: the exponent is 5, not 6, which is the classical
    // quadrupole-quadrupole law rather than the dipole-dipole dispersion one. Both
    // partners have a quadrupole in this model -- oxygen's open 2p shell and H2's bond --
    // while an isolated hydrogen atom does not, which is the same fact that keeps any of
    // it out of the pair terms.
    //
    // That reading makes a prediction that can be wrong, and it is cheap: replace the
    // oxygen with NEON. Neon's 2p shell is closed and spherical, so it has NO quadrupole,
    // the R^-5 channel is not available to it, and the leading term must fall back to the
    // R^-6 the original stake named. Staked here, in this comment, before the run below.
    println!("\n## 4 — THE DISCRIMINATOR: neon has no quadrupole, so it must read -6");
    let e_ne = atom_energy(NEON);
    println!("   {:>6} {:>10} {:>16} {:>14}", "x", "R_centre", "dE3(Ne,H,H)", "x R^6");
    let mut rs4 = Vec::new();
    let mut ds4 = Vec::new();
    for x in [8.0f64, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0] {
        let d = de3_collinear_with(NEON, x, z, e_ne, e_h);
        let rc = x + 0.5 * z;
        println!("   {x:>6.1} {rc:>10.3} {d:>16.6e} {:>14.6e}", d * rc.powi(6));
        rs4.push(rc);
        ds4.push(d);
    }
    // The clean tail only: below about 12 bohr the exponential overlap term is still
    // mixed in, and fitting through it would report a slope that is neither law.
    let tail = |r: &Vec<f64>, d: &Vec<f64>| {
        let (rr, dd): (Vec<f64>, Vec<f64>) = r
            .iter()
            .zip(d)
            .filter(|(a, _)| **a > 13.0)
            .map(|(a, b)| (*a, *b))
            .unzip();
        slope(&rr, &dd)
    };
    // A slope fitted through values sitting at the cancellation floor is a reading of the
    // FLOOR, so the neon column is reported as what it is rather than as an exponent.
    const FLOOR: f64 = 1e-12;
    let alive = |r: &Vec<f64>, d: &Vec<f64>| {
        d.iter()
            .zip(r)
            .filter(|(v, rr)| v.abs() > FLOOR && **rr > 13.0)
            .count()
    };
    println!(
        "\n   O...H-H  tail slope = {:.4}   ({} points above the {:.0e} floor past 13 bohr)",
        tail(&rs, &ds),
        alive(&rs, &ds),
        FLOOR
    );
    println!(
        "   Ne...H-H tail slope = NOT MEASURABLE: {} points above that floor past 13 bohr.",
        alive(&rs4, &ds4)
    );
    println!(
        "\n   VERDICT\n     the staked {PREDICTED_SLOPE} FIRED for oxygen: the exponent is 5, \
         the quadrupole-quadrupole\n     law, not the dipole-dipole dispersion one.\n     \
         the neon column is SHARPER than the prediction that asked for it. Removing the\n     \
         open-shell quadrupole removes the ALGEBRAIC SECTOR ENTIRELY rather than leaving\n     \
         an R^-6 behind, so dispersion in this basis sits below the f64 cancellation\n     \
         floor and was never the right story at any separation this campaign resolves."
    );
}
