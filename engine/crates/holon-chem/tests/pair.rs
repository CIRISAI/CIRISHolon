//! The pair-curve gates: the H2 regression through the new code path (R2), the emergent
//! negatives (E1), and the sandbox contract (E3).
//!
//! The heavy half of ELEMENTS-1 — Li2, N2 and CO, at 14 400 determinants each — is not
//! here. Those curves take minutes and a test suite that takes minutes stops being run;
//! they live in `examples/campaign.rs`, whose output is the campaign's reported product.
//! What IS here is every gate that can be checked on the cheap species, and the negatives
//! are among them: He2 and Ne2 are one determinant each, which is exactly why they cannot
//! bind and exactly why they cost nothing to check.

// `common` is shared with `tests/referee.rs`, which uses helpers this file does not.
#[allow(dead_code)]
mod common;

use common::{decimal_minus_f64, string_array, string_scalar};
use holon_chem::elements::{FLUORINE, HELIUM, HYDROGEN, LITHIUM, NEON};
use holon_chem::fnv1a32;
use holon_chem::pair::{
    atom_energy, declared_colour, generate_pair_table, pair_point, PairCache, WELL_MIN_DEPTH,
};

/// R2, the H2 half: the general first-row path against the banked closed form, at every
/// separation of the pinned 50-digit referee.
///
/// # Why against `h2.rs` and not directly against the referee file
///
/// `tests/referee.rs` already grades `h2.rs` against the referee in exact decimal and
/// pins the file by digest. Grading the general path against `h2.rs` therefore inherits
/// that gate and adds the one thing it cannot say: that TWO independent implementations
/// inside this crate — a symmetry-determined 2x2 closed form and a general determinant
/// FCI over a Cholesky-orthonormalised basis — land on the same curve. They share the
/// six declared decimals of the hydrogen contraction and nothing else.
#[test]
fn the_general_path_reproduces_the_banked_h2_curve() {
    let src = common::referee_text();
    let rs: Vec<f64> = string_array(&src, "R_grid_bohr")
        .iter()
        .map(|s| s.parse::<f64>().unwrap())
        .collect();
    assert_eq!(rs.len(), holon_chem::REFEREE_GRID_POINTS);

    let (mut we, mut wf, mut w2) = (0.0f64, 0.0f64, 0.0f64);
    let (mut re, mut rf) = (0.0f64, 0.0f64);
    for &r in rs.iter() {
        let banked = holon_chem::h2_point(r);
        let general = pair_point(HYDROGEN, HYDROGEN, r);
        if (banked.e - general.e).abs() > we {
            we = (banked.e - general.e).abs();
            re = r;
        }
        if (banked.f - general.f).abs() > wf {
            wf = (banked.f - general.f).abs();
            rf = r;
        }
        w2 = w2.max((banked.e2 - general.e2).abs());
    }
    println!("general vs banked H2 over {} separations:", rs.len());
    println!("  max |dE|  = {we:.4e} hartree      at R = {re}");
    println!("  max |dF|  = {wf:.4e} hartree/bohr at R = {rf}");
    println!("  max |dE2| = {w2:.4e} hartree/bohr^2");

    // ELEMENTS-1 stakes R2 at 1e-10 hartree pointwise, looser than H2's own 1e-12 because
    // p-integral conditioning is harder. H2 has no p integrals, so it should and does
    // land far inside that; the assertion is at the H2 stake rather than the ELEMENTS-1
    // one, because loosening a bound the code already meets is how a regression hides.
    assert!(
        we <= holon_chem::REFEREE_STAKE_E,
        "the general path departs from the banked H2 curve by {we:.4e} hartree at R = {re}"
    );
    assert!(wf < 1e-11, "force residual {wf:.4e} hartree/bohr");
    assert!(w2 < 1e-9, "curvature residual {w2:.4e} hartree/bohr^2");

    // The whole-curve scalars, which are roots and differences rather than values and so
    // fail differently.
    let table = generate_pair_table(HYDROGEN, HYDROGEN, 32);
    let well = table.meta.well.expect("H2 binds in this model");
    let d_re = decimal_minus_f64(&string_scalar(&src, "R_e"), well.r_e).abs();
    let d_de = decimal_minus_f64(&string_scalar(&src, "D_e"), well.d_e).abs();
    let d_as = decimal_minus_f64(&string_scalar(&src, "E_asymptote"), table.meta.e_asymptote).abs();
    println!("  |dR_e| = {d_re:.3e}   |dD_e| = {d_de:.3e}   |dE_asym| = {d_as:.3e}");
    assert!(d_re < 1e-9, "R_e residual {d_re:.3e} bohr");
    assert!(d_de < 1e-12, "D_e residual {d_de:.3e} hartree");
    assert!(d_as < 1e-14, "asymptote residual {d_as:.3e} hartree");
}

/// E1: THE EMERGENT NEGATIVES. In this model helium and neon refuse to bind, and nothing
/// in the code tells them to.
///
/// # What the gate is on, and what it is deliberately not on
///
/// The stake is the IN-MODEL well depth, which is the quantity the model controls
/// (M-NULL-MISSTAKE). It is not a claim about nature: the real He2 dimer IS bound, by
/// about 10^-7 hartree, entirely through dispersion — a correlation effect between
/// virtual orbitals a minimal basis does not contain. The model excludes that physics by
/// construction, and this test asserts what the model says rather than what is true.
#[test]
fn e1_the_closed_shells_refuse_to_bind() {
    for (label, a, b) in [("He2", HELIUM, HELIUM), ("Ne2", NEON, NEON)] {
        let table = generate_pair_table(a, b, 40);
        let deepest = table.e.iter().cloned().fold(f64::INFINITY, f64::min);
        let depth = table.meta.e_asymptote - deepest;
        println!(
            "  {label}: {} knots over [{:.3}, {:.3}] bohr; deepest point sits {:+.4e} \
             hartree relative to the asymptote (a well would be positive); well = {:?}",
            table.r.len(),
            table.meta.r_min,
            table.meta.r_max,
            depth,
            table.meta.well.map(|w| w.d_e)
        );
        assert!(
            table.meta.well.is_none(),
            "BRANCH (b) FIRED: {label} reported a well of depth {:?} hartree. Either the \
             model or the code is wrong; find which.",
            table.meta.well.map(|w| w.d_e)
        );
        assert!(
            depth <= WELL_MIN_DEPTH,
            "BRANCH (b) FIRED: {label}'s deepest sampled point is {depth:.4e} hartree \
             below its asymptote, past the staked {WELL_MIN_DEPTH:.0e}"
        );
        // The force must be repulsive everywhere, not merely shallow. A curve that dipped
        // and recovered inside one grid interval would pass the depth test on the knots
        // and still be a well.
        for (i, &f) in table.f.iter().enumerate() {
            assert!(
                f >= -1e-9,
                "{label} pulls inward at R = {} (F = {f:.3e}); the curve is not \
                 monotonically repulsive",
                table.r[i]
            );
        }
    }
}

/// The other side of E1: the code that reports "no well" is the same code that finds one.
///
/// Without this, the negatives could be a bug — a well-finder that never fires would pass
/// E1 on every pair including the bound ones. So the same call is made on a pair that
/// must bind, and it must.
#[test]
fn the_well_finder_is_not_simply_silent() {
    let table = generate_pair_table(HYDROGEN, FLUORINE, 32);
    let well = table
        .meta
        .well
        .expect("HF binds in this model; a well-finder that cannot see it cannot be trusted to report its absence");
    println!(
        "  HF: R_e = {:.6} bohr, D_e = {:.6} hartree, k_e = {:.6}",
        well.r_e, well.d_e, well.k_e
    );
    assert!(well.d_e > WELL_MIN_DEPTH * 100.0);
    assert!(well.k_e > 0.0, "a minimum must have positive curvature");
    // R_e is the ROOT of dE/dR, so the force there is zero to working precision — not
    // merely small compared to the well.
    let at = pair_point(HYDROGEN, FLUORINE, well.r_e);
    assert!(
        at.f.abs() < 1e-9,
        "the reported R_e carries a force of {:.3e} hartree/bohr",
        at.f
    );
}

/// E3: THE SANDBOX CONTRACT. Every pair emits the same four columns and the same scalars,
/// bound or not, and the unbound ones say so in the schema.
#[test]
fn e3_every_pair_emits_the_renderer_contract() {
    for (label, a, b, expect_bound) in [
        ("H2", HYDROGEN, HYDROGEN, true),
        ("LiH", LITHIUM, HYDROGEN, true),
        ("He2", HELIUM, HELIUM, false),
        ("HeH", HELIUM, HYDROGEN, false),
    ] {
        let t = generate_pair_table(a, b, 24);
        let n = t.r.len();
        assert_eq!(n, 24, "{label}: knot count");
        assert_eq!(t.e.len(), n);
        assert_eq!(t.f.len(), n);
        assert_eq!(t.e2.len(), n);
        for i in 0..n {
            assert!(t.r[i].is_finite() && t.r[i] > 0.0, "{label}: R[{i}]");
            assert!(t.e[i].is_finite(), "{label}: E[{i}]");
            assert!(t.f[i].is_finite(), "{label}: F[{i}]");
            assert!(t.e2[i].is_finite(), "{label}: E2[{i}]");
            if i > 0 {
                assert!(t.r[i] > t.r[i - 1], "{label}: the grid is not increasing at {i}");
            }
        }
        assert_eq!(t.meta.well.is_some(), expect_bound, "{label}: boundness");
        // Species metadata, which is the extension the multi-element sandbox needs.
        assert_eq!(t.meta.z_a, a.z, "{label}: the table's first species");
        assert_eq!(t.meta.z_b, b.z, "{label}: the table's second species");
        assert_eq!(t.meta.symbol_a, a.symbol);
        assert_eq!(t.meta.symbol_b, b.symbol);
        assert!(t.meta.reduced_mass > 0.0);
        assert_eq!(t.meta.n_electrons, (a.z + b.z) as usize);
        assert!(t.meta.n_det >= 1);
        // The asymptote is the sum of two independently computed atoms, and the outermost
        // knot must be near it — that is the whole meaning of the column's zero.
        let tail = (t.e[n - 1] - t.meta.e_asymptote).abs();
        assert!(
            tail < 1e-6,
            "{label}: the outermost knot is {tail:.3e} hartree from the asymptote"
        );
        let expect = atom_energy(a) + atom_energy(b);
        assert!((t.meta.e_asymptote - expect).abs() < 1e-14);

        // The JSON contract. Parsed by pattern rather than by a parser, for the reason
        // `common/mod.rs` gives: the file is a fixture this crate owns.
        let json = t.to_json();
        for key in [
            "\"provenance\"", "\"species\"", "\"E_asymptote\"", "\"bound\"", "\"R_e\"",
            "\"D_e\"", "\"R_grid_bohr\"", "\"E_hartree\"", "\"F_hartree_per_bohr\"",
            "\"E2_hartree_per_bohr2\"", "\"n_grid\"", "\"convergence\"",
        ] {
            assert!(json.contains(key), "{label}: the emitted table has no {key}");
        }
        assert!(json.contains(if expect_bound { "\"bound\": true" } else { "\"bound\": false" }));
        if !expect_bound {
            // Null rather than zero: a zero could be read as a measurement of no binding
            // energy, and this is the absence of a measurement.
            assert!(json.contains("\"R_e\": null"), "{label}: unbound R_e must be null");
            assert!(json.contains("\"D_e\": null"), "{label}: unbound D_e must be null");
        }
        assert!(!json.contains("NaN") && !json.contains("inf"), "{label}: non-JSON number emitted");
        println!(
            "  {label}: {n} knots, {} determinants, bound = {}, {} bytes of JSON, {:.0} ms",
            t.meta.n_det,
            t.meta.well.is_some(),
            json.len(),
            t.meta.generation_ms
        );
    }
}

/// The lazy cache computes once and then does not.
#[test]
fn the_pair_cache_is_lazy_and_unordered() {
    let mut cache = PairCache::new(16);
    let first_ms = {
        let t = cache.get(HYDROGEN, HELIUM);
        t.meta.generation_ms
    };
    assert_eq!(cache.misses, 1);
    assert_eq!(cache.hits, 0);
    // The reversed pair is the SAME pair. Two entries would be two things to keep in step.
    let again = cache.get(HELIUM, HYDROGEN);
    assert_eq!(again.meta.generation_ms, first_ms);
    assert_eq!(cache.len(), 1, "(H, He) and (He, H) must be one entry");
    assert_eq!(cache.hits, 1);
    cache.get(HYDROGEN, HYDROGEN);
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.misses, 2);
    assert!(cache.total_ms > 0.0);
}

/// The palette's colour rule, checked to be what it says it is.
///
/// The rule claims two things and this checks both. MONOTONE in Z, so that neighbours in
/// the row are neighbours on screen — measured as warmth, red minus blue, which is the
/// axis the hue ramp walks. And DISTINGUISHABLE, because a monotone ramp that saturates
/// would put two elements at the same colour and a scene could not be read; the ramp does
/// flatten in warmth at the top of the row (Z = 9 and Z = 10 tie there), so the
/// separation is asserted on the whole colour rather than on warmth alone.
#[test]
fn the_colour_rule_is_declared_monotone_and_distinguishable() {
    let rgb = |z: u32| -> (i32, i32, i32) {
        let c = declared_colour(z);
        assert_eq!(c.len(), 7, "colour must be #rrggbb, got {c}");
        assert!(c.starts_with('#'));
        let n = u32::from_str_radix(&c[1..], 16).expect("hex");
        (((n >> 16) & 255) as i32, ((n >> 8) & 255) as i32, (n & 255) as i32)
    };
    let warmth = |z: u32| { let (r, _, b) = rgb(z); r - b };
    for z in 2..=10u32 {
        assert!(
            warmth(z) >= warmth(z - 1),
            "the declared ramp reverses between Z = {} ({}) and Z = {z} ({})",
            z - 1,
            declared_colour(z - 1),
            declared_colour(z)
        );
        let (r0, g0, b0) = rgb(z - 1);
        let (r1, g1, b1) = rgb(z);
        let sep = (r1 - r0).abs() + (g1 - g0).abs() + (b1 - b0).abs();
        assert!(
            sep >= 20,
            "Z = {} and Z = {z} are only {sep} apart in RGB ({} vs {}); two elements would \
             read as the same atom",
            z - 1,
            declared_colour(z - 1),
            declared_colour(z)
        );
    }
    assert!(
        warmth(10) - warmth(1) > 100,
        "the ramp does not actually traverse the row"
    );
}

// ------------------------------------------------------------------ the referee gate
//
// IGNORED UNTIL THE REFEREE LANDS. The sibling lane (`elements-referee`) is building the
// 50-digit mpmath implementation of this same model. When its output is committed to
// `tests/data/elements1/`, DELETE the `#[ignore]` below and re-pin ELEMENTS1_REFEREE_DIGEST
// from the failure message the digest check prints. Do not do one without the other: a
// gate that grades against a file it does not identify can be satisfied by editing the
// file, which is the whole reason the H2 gate pins its referee by digest.
//
// THE SCHEMA THIS TEST EXPECTS, one file per pair, numbers as decimal STRINGS so the 50
// digits survive JSON:
//
//   tests/data/elements1/<A><B>.json
//     "model":                 "<A><B>/STO-3G/FCI"
//     "R_grid_bohr":           ["1.0", ...]            (decimal strings)
//     "E_hartree":             ["-1.13...", ...]
//     "F_hartree_per_bohr":    [...]                   (the FORCE, -dE/dR)
//     "E2_hartree_per_bohr2":  [...]
//     "E_asymptote":           "..."                   (scalar string)
//     "R_e", "D_e":            "..." or the string "unbound"
//
//   tests/data/elements1/atoms.json
//     "symbols":               ["H", "He", ...]
//     "E_hartree":             [...]                   (one per symbol, Z ascending)
//
// Strings rather than JSON numbers throughout, because a JSON number is an f64 the moment
// anything reads it and the point of a 50-digit referee is the digits past the 17th.

/// FNV-1a of the concatenated referee files, in the order listed below. Zero until the
/// files exist; the digest check prints the value to pin.
pub const ELEMENTS1_REFEREE_DIGEST: u32 = 0;

/// The staked separation-wise agreement for the first row, hartree.
///
/// Looser than H2's 1e-12 by the freeze's own reasoning: p-function integrals go through
/// the Hermite `R` tensor, whose recursion accumulates cancellation that s-only closed
/// forms do not have. The MEASURED residual is the reportable product and becomes the
/// successor's stake.
pub const ELEMENTS1_STAKE_E: f64 = 1e-10;

#[test]
#[ignore = "waiting on the elements-referee lane's 50-digit output; see the note above"]
fn r2_the_first_row_matches_the_fifty_digit_referee() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/elements1");
    let pairs = ["H2", "LiH", "Li2", "HF", "N2", "F2", "CO", "He2", "Ne2"];

    let mut all = Vec::new();
    for name in pairs.iter() {
        all.extend(std::fs::read(dir.join(format!("{name}.json"))).unwrap_or_else(|e| {
            panic!("referee file for {name} is missing ({e}); un-ignore this test only \
                    once the whole set is committed")
        }));
    }
    all.extend(std::fs::read(dir.join("atoms.json")).expect("atoms.json"));
    let digest = fnv1a32(&all);
    assert_eq!(
        digest, ELEMENTS1_REFEREE_DIGEST,
        "the ELEMENTS-1 referee set has changed (digest {digest:#010x}). If that was \
         deliberate, re-derive the residuals against the new files rather than re-pinning \
         the digest alone."
    );

    let species = |sym: &str| holon_chem::elements::by_symbol(sym).expect("first-row symbol");
    let mut worst_overall = 0.0f64;
    for name in pairs.iter() {
        let src = std::fs::read_to_string(dir.join(format!("{name}.json"))).unwrap();
        let (a, b) = split_pair(name);
        let rs = string_array(&src, "R_grid_bohr");
        let es = string_array(&src, "E_hartree");
        assert_eq!(rs.len(), es.len(), "{name}: column length mismatch");
        let mut worst = 0.0f64;
        let mut at = 0usize;
        for (i, (rstr, estr)) in rs.iter().zip(es.iter()).enumerate() {
            let r = rstr.parse::<f64>().unwrap();
            let mine = pair_point(species(&a), species(&b), r).e;
            let d = decimal_minus_f64(estr, mine).abs();
            if d > worst {
                worst = d;
                at = i;
            }
        }
        println!("  {name}: max |dE| = {worst:.4e} hartree at R = {}", rs[at]);
        assert!(
            worst <= ELEMENTS1_STAKE_E,
            "THE STAKE FIRED: {name} disagrees with the 50-digit referee by {worst:.4e} \
             hartree at R = {}, past the staked {ELEMENTS1_STAKE_E:.0e}",
            rs[at]
        );
        worst_overall = worst_overall.max(worst);
    }
    println!("worst over the whole first row: {worst_overall:.4e} hartree");
}

/// `"LiH"` to `("Li", "H")`. Two-letter symbols start with the only capital.
fn split_pair(name: &str) -> (String, String) {
    let cut = 1 + name[1..].find(char::is_uppercase).expect("two symbols");
    (name[..cut].to_string(), name[cut..].to_string())
}
