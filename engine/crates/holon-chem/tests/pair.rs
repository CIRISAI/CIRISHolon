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
// `tests/data/elements1/`, DELETE the `#[ignore]` below and re-pin
// ELEMENTS1_REFEREE_DIGEST from the failure message the digest check prints. Do not do one
// without the other: a gate that grades against a file it does not identify can be
// satisfied by editing the file, which is the whole reason the H2 gate pins its referee by
// digest.
//
// To exercise it against a drop that is not committed yet:
//   ELEMENTS1_REFEREE_DIR=/path/to/drop cargo test -p holon-chem --release --test pair \
//       -- --ignored --nocapture
// The digest is enforced only when reading the committed default path, because a preview
// drop is not a thing to pin.
//
// THE SCHEMA, one file per pair, numbers as decimal STRINGS so the 50 digits survive JSON:
//
//   <PAIR>.json
//     "model":                 "<A><B>/STO-3G/FCI"
//     "R_grid_bohr":           ["1.0", ...]            (plain fixed point, no exponent)
//     "E_hartree":             ["-1.13...", ...]
//     "F_hartree_per_bohr":    [...]                   (the FORCE, -dE/dR)
//     "E2_hartree_per_bohr2":  [...]
//     "E_asymptote":           "..."
//     "R_e", "D_e":            "..." or the string "unbound"
//     "derivative_provenance": {"route": "...",
//                               "E_max_abs_uncertainty_hartree": "...",
//                               "F_max_abs_uncertainty_hartree_per_bohr": "...",
//                               "E2_max_abs_uncertainty_hartree_per_bohr2": "..."}
//
//   atoms.json
//     "symbols":               ["H", "He", ...]        Z ascending
//     "E_hartree":             [...]                   one per symbol

/// The nine pairs ELEMENTS-1 stakes, in the freeze's own order.
///
/// # Why this one list stays hardcoded when everything else is read from the drop
///
/// The referee's `manifest.json` declares which pairs it has delivered and which it still
/// owes, and coverage is read from THAT rather than from a list here — a list of delivered
/// pairs maintained beside the files drifts out of step with them, which is the referee
/// lane's point and it is correct.
///
/// But coverage read entirely from the data has the failure this whole campaign keeps
/// meeting: if a file disappears and the manifest shrinks with it, the gate covers less and
/// says nothing, because the data it is checking against shrank too. So exactly one thing
/// is frozen here — the STAKED SET, which comes from the prereg and cannot legitimately
/// change — and the test asserts `pairs_present + pairs_owed` is exactly this. A pair may
/// move from present to owed; it cannot vanish from both.
pub const ELEMENTS1_STAKED_PAIRS: [&str; 9] = [
    "H2", "LiH", "Li2", "HF", "N2", "F2", "CO", "He2", "Ne2",
];

/// FNV-1a over the drop: each STAKED pair that is present, in the order above, then
/// `atoms.json`, then `manifest.json`. Filesystem order never enters it.
///
/// Re-pin deliberately whenever the referee delivers, and re-read the residuals rather
/// than only bumping the number — that is what the H2 gate's digest exists to force and
/// this one inherits the rule.
pub const ELEMENTS1_REFEREE_DIGEST: u32 = 0xeb8c_4d9a;

/// The staked separation-wise agreement on the ENERGY for the first row, hartree.
///
/// This is R2's stake, not E1's — E1 is the well-depth gate at 1e-4 and has nothing to do
/// with this number. Looser than H2's own 1e-12 by the freeze's reasoning: p-function
/// integrals go through the Hermite `R` tensor, whose recursion accumulates cancellation
/// that s-only closed forms do not have. The MEASURED residual is the reportable product
/// and becomes the successor's stake.
pub const ELEMENTS1_STAKE_E: f64 = 1e-10;

/// How many times its own DECLARED uncertainty a derivative column may miss by.
///
/// # Why the derivative columns are not gated at `ELEMENTS1_STAKE_E`
///
/// Because that would grade the referee's interpolant rather than this engine's
/// arithmetic. The referee supplies `E` at full working precision everywhere, but `F` and
/// `E2` are only referee-grade where a raised-precision stencil covers the knot; elsewhere
/// they come from a local interpolant, which is worst at the outermost knot — exactly
/// where a dissociation tail's `E'` is smallest. Measured on H2 by the referee lane, the
/// interpolant reads `F(10 bohr) = +1.54e-7` where the stencil reads `-3.82e-8`: the WRONG
/// SIGN, and three orders above this gate's energy stake. So each file declares what its
/// own derivative columns are worth and they are graded against that.
///
/// A file with no declaration is REFUSED rather than given the flat bound: an absent
/// uncertainty must never read as zero uncertainty.
pub const DERIVATIVE_MARGIN: f64 = 2.0;

fn referee_dir() -> std::path::PathBuf {
    match std::env::var("ELEMENTS1_REFEREE_DIR") {
        Ok(d) => std::path::PathBuf::from(d),
        Err(_) => std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/elements1"),
    }
}

/// A scalar the referee wrote in ordinary f64 notation (an uncertainty, not a value).
/// Exponent notation is fine here and is not fine in the columns — the exact-decimal
/// comparator handles only plain fixed point, which is why the two are read differently.
fn f64_scalar(src: &str, key: &str) -> f64 {
    string_scalar(src, key)
        .parse::<f64>()
        .unwrap_or_else(|e| panic!("\"{key}\" is not a number: {e}"))
}

#[test]
fn r2_the_first_row_matches_the_fifty_digit_referee() {
    let dir = referee_dir();
    let is_committed = std::env::var("ELEMENTS1_REFEREE_DIR").is_err();

    // Coverage comes from the drop's own manifest, never from a list maintained here.
    let manifest = std::fs::read_to_string(dir.join("manifest.json"))
        .expect("manifest.json: the drop must declare its own coverage");
    let present = string_array(&manifest, "pairs_present");
    let owed = string_array(&manifest, "pairs_owed");
    let fingerprint = string_scalar(&manifest, "basis_fingerprint");

    // The invariant that makes shrinkage impossible: present + owed IS the staked set.
    let mut union: Vec<&str> = present.iter().chain(owed.iter()).map(|s| s.as_str()).collect();
    union.sort_unstable();
    let mut staked: Vec<&str> = ELEMENTS1_STAKED_PAIRS.to_vec();
    staked.sort_unstable();
    assert_eq!(
        union, staked,
        "the manifest's pairs_present + pairs_owed is not the staked set. A pair may move \
         from present to owed, but it cannot leave both lists — that would shrink this \
         gate's coverage with nothing to say so."
    );
    assert!(!fingerprint.is_empty(), "the manifest declares no basis_fingerprint");

    // Every declared pair has a file, and every pair file is declared. Neither direction
    // is redundant: the first catches a manifest that promises what it did not ship, the
    // second catches a file being graded that nobody declared.
    for name in present.iter() {
        assert!(
            dir.join(format!("{name}.json")).exists(),
            "manifest declares {name} present but {name}.json is not in the drop"
        );
    }
    for entry in std::fs::read_dir(&dir).expect("referee directory").flatten() {
        let file = entry.file_name().to_string_lossy().to_string();
        let Some(stem) = file.strip_suffix(".json") else { continue };
        if stem == "atoms" || stem == "manifest" {
            continue;
        }
        assert!(
            present.iter().any(|p| p == stem),
            "{file} is in the drop but the manifest does not declare {stem} present; an \
             undeclared file would be graded with nothing stating what it is"
        );
    }

    println!(
        "referee drop at {} — basis {fingerprint}\n  COVERED ({} of {}): {}\n  OWED  ({}): {}",
        dir.display(),
        present.len(),
        ELEMENTS1_STAKED_PAIRS.len(),
        present.join(" "),
        owed.len(),
        owed.join(" ")
    );
    if !owed.is_empty() {
        println!(
            "  R2 IS PARTIALLY DISCHARGED. The staked pointwise bound is measured on the \
             {} pair(s) above and on all ten atoms; the {} owed pair(s) are NOT yet graded \
             against a 50-digit referee by this gate.",
            present.len(),
            owed.len()
        );
    }

    if is_committed {
        let mut all = Vec::new();
        for name in ELEMENTS1_STAKED_PAIRS.iter() {
            if present.iter().any(|p| p == name) {
                all.extend(std::fs::read(dir.join(format!("{name}.json"))).unwrap());
            }
        }
        all.extend(std::fs::read(dir.join("atoms.json")).unwrap());
        all.extend(std::fs::read(dir.join("manifest.json")).unwrap());
        let digest = fnv1a32(&all);
        assert_eq!(
            digest, ELEMENTS1_REFEREE_DIGEST,
            "the ELEMENTS-1 referee drop has changed (digest {digest:#010x}). If that was \
             deliberate — a delivery, or a re-emission — re-pin to that value AND re-read \
             the residuals below, rather than bumping the number alone."
        );
    } else {
        println!("  UNPINNED drop (ELEMENTS1_REFEREE_DIR set); the digest gate is enforced \
                  only on the committed set");
    }

    // --- the atoms, which every curve's asymptote is built from ---
    let atoms_src = std::fs::read_to_string(dir.join("atoms.json")).expect("atoms.json");
    let symbols = string_array(&atoms_src, "symbols");
    let atom_e = string_array(&atoms_src, "E_hartree");
    assert_eq!(symbols.len(), atom_e.len());
    let mut worst_atom = 0.0f64;
    for (sym, estr) in symbols.iter().zip(atom_e.iter()) {
        let sp = holon_chem::elements::by_symbol(sym).expect("first-row symbol");
        let d = decimal_minus_f64(estr, atom_energy(sp)).abs();
        println!("  atom {sym:>2}: |dE| = {d:.4e} hartree");
        worst_atom = worst_atom.max(d);
    }
    assert!(
        worst_atom <= ELEMENTS1_STAKE_E,
        "THE STAKE FIRED on an atomic energy: {worst_atom:.4e} hartree, past the staked \
         {ELEMENTS1_STAKE_E:.0e}"
    );

    // --- the curves ---
    let species = |sym: &str| holon_chem::elements::by_symbol(sym).expect("first-row symbol");
    let (mut worst_e, mut worst_f, mut worst_e2) = (0.0f64, 0.0f64, 0.0f64);
    for name in present.iter() {
        let src = std::fs::read_to_string(dir.join(format!("{name}.json"))).unwrap();
        let (a, b) = split_pair(name);
        let (a, b) = (species(&a), species(&b));

        // The declared worth of this file's own derivative columns. Absent means refused.
        assert!(
            src.contains("\"derivative_provenance\""),
            "{name}.json carries no derivative_provenance block. Its F and E2 columns \
             cannot be graded: an absent uncertainty is not a zero uncertainty."
        );
        let route = string_scalar(&src, "route");
        let unc_e = f64_scalar(&src, "E_max_abs_uncertainty_hartree");
        let unc_f = f64_scalar(&src, "F_max_abs_uncertainty_hartree_per_bohr");
        let unc_e2 = f64_scalar(&src, "E2_max_abs_uncertainty_hartree_per_bohr2");
        // An EXACT zero is not an uncertainty, it is a missing one wearing a number. It is
        // more dangerous than an absent block because it passes a presence check, and
        // refusing it here is the same rule as refusing the absence.
        //
        // ALL THREE columns, and the third one is the lesson. This check first covered only
        // F and E2, on the reasoning that a numerically differentiated column can never be
        // exact — and the energy column was praised in the same breath for declaring a real
        // 6e-62 bound. It was not doing so everywhere: the referee's He2 and Ne2 declared
        // E uncertainty ZERO, because their eigensolve genuinely is exact for a
        // one-determinant space. But an exact eigensolve of inexact integrals is not an
        // exact energy, and the zero claimed it was. The referee lane found that by
        // applying this rule to a column this rule did not yet cover, which is the whole
        // argument for making the rule uniform rather than reasoning per column about
        // which ones "could" be exact.
        for (col, unc) in [("E", unc_e), ("F", unc_f), ("E2", unc_e2)] {
            assert!(
                unc > 0.0 && unc.is_finite(),
                "{name}.json declares a {col} uncertainty of {unc:?}. An exact zero is not \
                 a bound, it is a missing bound wearing a number: no column here is \
                 computed exactly, and grading against zero would assert the referee is \
                 perfect. Declared route: {route}"
            );
        }

        let rs = string_array(&src, "R_grid_bohr");
        let es = string_array(&src, "E_hartree");
        let fs = string_array(&src, "F_hartree_per_bohr");
        let e2s = string_array(&src, "E2_hartree_per_bohr2");
        assert_eq!(rs.len(), es.len(), "{name}: E column length");
        assert_eq!(rs.len(), fs.len(), "{name}: F column length");
        assert_eq!(rs.len(), e2s.len(), "{name}: E2 column length");

        let (mut we, mut wf, mut w2, mut at) = (0.0f64, 0.0f64, 0.0f64, 0usize);
        // My own curve ON THE REFEREE'S GRID, kept so the boundness cross-check below can
        // be made from it. Re-deriving a table of my own would compare two curves sampled
        // at different separations, and would cost a full generation per pair on top.
        let mut my_r = Vec::with_capacity(rs.len());
        let mut my_e = Vec::with_capacity(rs.len());
        for (i, rstr) in rs.iter().enumerate() {
            let r = rstr.parse::<f64>().unwrap();
            let mine = pair_point(a, b, r);
            let de = decimal_minus_f64(&es[i], mine.e).abs();
            if de > we {
                we = de;
                at = i;
            }
            wf = wf.max(decimal_minus_f64(&fs[i], mine.f).abs());
            w2 = w2.max(decimal_minus_f64(&e2s[i], mine.e2).abs());
            my_r.push(r);
            my_e.push(mine.e);
        }
        // The asymptote, and the well when the referee reports one.
        let d_asym = decimal_minus_f64(
            &string_scalar(&src, "E_asymptote"),
            atom_energy(a) + atom_energy(b),
        )
        .abs();
        let r_e = string_scalar(&src, "R_e");
        println!(
            "  {name:>4}: |dE| {we:.3e} at R = {}  |dF| {wf:.3e} (declared {unc_f:.1e})  \
             |dE2| {w2:.3e} (declared {unc_e2:.1e})  |dAsym| {d_asym:.3e}  R_e = {r_e}",
            rs[at]
        );
        println!("        derivative route: {route}");

        assert!(
            we <= ELEMENTS1_STAKE_E + unc_e,
            "THE STAKE FIRED: {name}'s energy disagrees with the 50-digit referee by \
             {we:.4e} hartree at R = {}, past the staked {ELEMENTS1_STAKE_E:.0e}",
            rs[at]
        );
        assert!(
            d_asym <= ELEMENTS1_STAKE_E,
            "{name}: the dissociation asymptote disagrees by {d_asym:.4e} hartree"
        );
        // Graded against what the file says its own columns are worth, plus this engine's
        // energy stake, because the residual carries both implementations' error.
        let allow_f = DERIVATIVE_MARGIN * unc_f + ELEMENTS1_STAKE_E;
        let allow_e2 = DERIVATIVE_MARGIN * unc_e2 + ELEMENTS1_STAKE_E;
        assert!(
            wf <= allow_f,
            "{name}: the force column disagrees by {wf:.4e} hartree/bohr, past \
             {DERIVATIVE_MARGIN}x the referee's own declared {unc_f:.3e} plus the energy \
             stake. That is this engine's defect, not the interpolant's."
        );
        assert!(
            w2 <= allow_e2,
            "{name}: the curvature column disagrees by {w2:.4e} hartree/bohr^2, past \
             {DERIVATIVE_MARGIN}x the referee's declared {unc_e2:.3e}"
        );
        // R_e is a root and is reported as "unbound" where there is none. The two
        // implementations must AGREE about which pairs bind — that is E1's substance
        // arriving through R2's file.
        let mine_well = holon_chem::pair::locate_well(
            a,
            b,
            &my_r,
            &my_e,
            atom_energy(a) + atom_energy(b),
        );
        assert_eq!(
            r_e == "unbound",
            mine_well.is_none(),
            "{name}: the referee says R_e = {r_e} and this engine says well = {:?}. The \
             two implementations disagree about whether this pair binds.",
            mine_well.map(|w| w.d_e)
        );
        if r_e != "unbound" {
            let w = mine_well.unwrap();
            let d_re = decimal_minus_f64(&r_e, w.r_e).abs();
            let d_de = decimal_minus_f64(&string_scalar(&src, "D_e"), w.d_e).abs();
            println!("        |dR_e| = {d_re:.3e} bohr, |dD_e| = {d_de:.3e} hartree");
            // R_e is a root, so an energy error `dE` displaces it by `sqrt(2 dE / E'')`.
            let displacement = (2.0 * ELEMENTS1_STAKE_E / w.k_e.abs()).sqrt();
            assert!(
                d_re <= 10.0 * displacement,
                "{name}: R_e differs by {d_re:.3e} bohr against a root displacement of \
                 {displacement:.3e} implied by the energy stake and this well's curvature"
            );
            assert!(d_de <= 10.0 * ELEMENTS1_STAKE_E, "{name}: D_e differs by {d_de:.3e}");
        }
        worst_e = worst_e.max(we);
        worst_f = worst_f.max(wf);
        worst_e2 = worst_e2.max(w2);
    }
    println!(
        "worst over the drop: E {worst_e:.4e} hartree, F {worst_f:.4e}, E2 {worst_e2:.4e}; \
         atoms {worst_atom:.4e}"
    );
}

/// A pair's file name to its two element symbols.
///
/// Two forms, because chemistry writes them differently: a homonuclear pair is a FORMULA
/// with a subscript (`"H2"`, `"He2"`, `"Ne2"`) and a heteronuclear one is two symbols run
/// together (`"LiH"`, `"CO"`). Getting this wrong is not subtle — it panics rather than
/// silently grading the wrong molecule — but it does have to handle both.
fn split_pair(name: &str) -> (String, String) {
    if let Some(sym) = name.strip_suffix('2') {
        return (sym.to_string(), sym.to_string());
    }
    let cut = 1 + name[1..]
        .find(char::is_uppercase)
        .unwrap_or_else(|| panic!("{name:?} is neither <symbol>2 nor two run-together symbols"));
    (name[..cut].to_string(), name[cut..].to_string())
}

#[test]
fn pair_file_names_parse_to_their_elements() {
    for (name, a, b) in [
        ("H2", "H", "H"), ("He2", "He", "He"), ("Ne2", "Ne", "Ne"), ("Li2", "Li", "Li"),
        ("N2", "N", "N"), ("F2", "F", "F"), ("LiH", "Li", "H"), ("HF", "H", "F"),
        ("CO", "C", "O"),
    ] {
        assert_eq!(split_pair(name), (a.to_string(), b.to_string()), "{name}");
        assert!(holon_chem::elements::by_symbol(a).is_some(), "{a}");
        assert!(holon_chem::elements::by_symbol(b).is_some(), "{b}");
    }
}
