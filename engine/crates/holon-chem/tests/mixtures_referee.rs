//! MIXTURES-1 gate R2, engine half: the staked exact pairs against the 50-digit referee.
//!
//! Contract: `conformance/atomworld/MIXTURES1_PREREG.md`.
//!
//! > **R2 — staked-pair referee gate**: the engine's curves for the staked EXACT set —
//! > Cl2, S2, Ar2, HCl, ClF, NaH, SiO — match the referee at <= 1e-10 Ha pointwise on
//! > each pair's declared grid (sparse grids allowed, rule declared in the file,
//! > result-blind), with per-column declared uncertainties and the spin audit carried per
//! > geometry. Coverage manifest-declared; present + owed = staked, enforced.
//!
//! # IGNORED UNTIL THE REFEREE LANDS
//!
//! The sibling lane (`mixtures-referee`) is building the drop this grades against. When it
//! is committed to `tests/data/mixtures1/`, DELETE the `#[ignore]` and re-pin
//! `MIXTURES1_REFEREE_DIGEST` from the failure message the digest check prints — and read
//! the residuals rather than only bumping the number. Doing one without the other gives a
//! gate that grades against a file it does not identify, which can be satisfied by editing
//! the file.
//!
//! To exercise it against a drop that is not committed yet:
//!
//! ```text
//! MIXTURES1_REFEREE_DIR=/path/to/drop cargo test -p holon-chem --release \
//!     --test mixtures_referee -- --ignored --nocapture
//! ```
//!
//! The digest is enforced only when reading the committed default path, because a preview
//! drop is not a thing to pin.
//!
//! # This is the ELEMENTS-1 gate's shape, not a second implementation of it
//!
//! The JSON readers and the exact-decimal comparator are `tests/common/mod.rs`, shared with
//! the H2 referee gate — the comparison has to be done in fixed-point decimal, because
//! parsing a 50-digit referee into an `f64` and subtracting cannot resolve anything below
//! half an ulp and would report its own rounding as agreement. What is restated here rather
//! than imported is the STAKED SET, deliberately: two lanes disagreeing about which pairs a
//! campaign stakes is a disagreement worth firing on, and it is invisible if either side
//! reads the other's list.

mod common;

use common::{decimal_minus_f64, string_array, string_scalar};
use holon_chem::elements::by_symbol;
use holon_chem::dual::D2;
use holon_chem::fci::solve_determinant;
use holon_chem::pair::{automatic_route, geometry_problem};

/// The seven pairs MIXTURES-1 stakes as EXACT, in the freeze's own order.
pub const MIXTURES1_STAKED_PAIRS: [&str; 7] =
    ["Cl2", "S2", "Ar2", "HCl", "ClF", "NaH", "SiO"];

/// FNV-1a over the drop: each STAKED pair that is present, in the order above, then
/// `atoms.json`, then `manifest.json`. Filesystem order never enters it.
///
/// ZERO means NOT YET PINNED — the drop does not exist. Re-pin deliberately when the
/// referee delivers.
pub const MIXTURES1_REFEREE_DIGEST: u32 = 0x0000_0000;

/// The staked pointwise agreement on the ENERGY, hartree. The freeze's number.
pub const MIXTURES1_STAKE_E: f64 = 1e-10;

/// The largest nuclear charge the REFEREE's model covers.
///
/// # Why a coverage gate needs a scope, and what happens without one
///
/// `elements.rs` now spans Z <= 54. The committed referee's model stops at argon. A gate
/// that grades "the engine matches the referee" over whatever the drop happens to contain
/// therefore says nothing at all about Z = 19..54 — and, worse, would keep saying "matches"
/// as the engine grew, because the pairs it does not cover are pairs it does not check.
/// That is the vacuous-success shape: a green gate whose greenness is a fact about its own
/// coverage rather than about the engine.
///
/// So the scope is DECLARED here, cross-checked against the drop's own declaration, and a
/// species above it is REFUSED BY NAME rather than skipped. The refusal is demonstrated
/// firing in `r2_an_out_of_scope_species_is_refused_by_name`.
pub const MIXTURES1_REFEREE_Z_MAX: u32 = 18;

/// Why a pair cannot be graded against this referee.
#[derive(Debug, PartialEq, Eq)]
pub enum ScopeRefusal {
    /// One of the pair's species is past the referee's declared model.
    OutOfScope { symbol: &'static str, z: u32 },
}

/// Whether this referee can grade a pair at all. `Err` is a REFUSAL, never a skip.
pub fn in_referee_scope(
    a: holon_chem::elements::Species,
    b: holon_chem::elements::Species,
) -> Result<(), ScopeRefusal> {
    for sp in [a, b] {
        if sp.z > MIXTURES1_REFEREE_Z_MAX {
            return Err(ScopeRefusal::OutOfScope {
                symbol: sp.symbol,
                z: sp.z,
            });
        }
    }
    Ok(())
}

/// How many times its own DECLARED uncertainty a derivative column may miss by.
///
/// Inherited from ELEMENTS-1 and for its reason: the referee supplies `E` at full working
/// precision everywhere, but `F` and `E2` are referee-grade only where a raised-precision
/// stencil covers the knot. A file with no declaration is REFUSED rather than given a flat
/// bound — an absent uncertainty must never read as zero uncertainty.
pub const DERIVATIVE_MARGIN: f64 = 2.0;

fn referee_dir() -> std::path::PathBuf {
    match std::env::var("MIXTURES1_REFEREE_DIR") {
        Ok(d) => std::path::PathBuf::from(d),
        Err(_) => {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/mixtures1")
        }
    }
}

fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in bytes {
        h ^= *b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// The exact-in-model total energy at one separation, ON THE DETERMINANT ROUTE.
///
/// `pair::pair_point` would be the obvious call and it is the wrong one: it goes through
/// `fci::solve`, which routes any space past `MPS_ROUTE_THRESHOLD` to DMRG. SiO is 132,496
/// determinants, so grading it through `pair_point` would compare the referee against the
/// DMRG bridge while calling the result exact — and, on this engine's MPO builder at
/// fourteen orbitals, would not return at all.
fn exact_total(
    a: holon_chem::elements::Species,
    b: holon_chem::elements::Species,
    r: f64,
) -> f64 {
    let (space, mo, nuc) = geometry_problem(
        &[a, b],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::c(r)],
        ],
    );
    (solve_determinant(&space, &mo).e + nuc).v
}

fn split_pair(name: &str) -> (holon_chem::elements::Species, holon_chem::elements::Species) {
    if let Some(stem) = name.strip_suffix('2') {
        let sp = by_symbol(stem).unwrap_or_else(|| panic!("unknown element {stem}"));
        return (sp, sp);
    }
    let at = name
        .char_indices()
        .skip(1)
        .find(|(_, c)| c.is_uppercase())
        .map(|(i, _)| i)
        .unwrap_or_else(|| panic!("cannot split {name} into two element symbols"));
    (
        by_symbol(&name[..at]).unwrap_or_else(|| panic!("unknown element {}", &name[..at])),
        by_symbol(&name[at..]).unwrap_or_else(|| panic!("unknown element {}", &name[at..])),
    )
}

/// The staked set is exactly what the freeze says, and every pair in it is a pair this
/// engine can name.
///
/// NOT ignored: it needs no drop, and it is the half of R2 that can be checked today.
/// A campaign whose staked list has quietly drifted is worth catching before the referee
/// arrives, not after.
#[test]
fn r2_the_staked_set_is_the_freeze_s() {
    assert_eq!(
        MIXTURES1_STAKED_PAIRS.len(),
        7,
        "the freeze stakes seven exact pairs"
    );
    for name in MIXTURES1_STAKED_PAIRS {
        let (a, b) = split_pair(name);
        let f = automatic_route(a, b);
        println!(
            "{name:>5}: {} + {}  n_basis {:>2}  n_det {:>9}  route {}",
            a.symbol,
            b.symbol,
            f.n_orb(),
            f.n_det(),
            f.route_name()
        );
    }
}

/// THE ENGINE HALF OF R2's FEASIBILITY READING, and it is a finding rather than a gate.
///
/// ONE of the seven staked EXACT pairs is past this engine's AUTOMATIC route: `fci::solve`
/// switches to DMRG above `MPS_ROUTE_THRESHOLD` determinants, and SiO is 132,496. So an
/// exact curve for SiO cannot be produced by the ordinary entry point — `solve_determinant`
/// has to be called directly, which is what gate D1's harness does and what the R2
/// comparison below does.
///
/// Printed rather than asserted: which pairs cross the threshold is a property of the
/// model and the basis, and pinning it would be pinning a consequence.
#[test]
fn r2_which_staked_pairs_leave_the_determinant_route() {
    let mut crossing = Vec::new();
    for name in MIXTURES1_STAKED_PAIRS {
        let (a, b) = split_pair(name);
        let f = automatic_route(a, b);
        if f.n_det() > holon_chem::fci::MPS_ROUTE_THRESHOLD {
            crossing.push((name, f.n_det(), f.n_orb()));
        }
    }
    println!(
        "staked pairs past fci::MPS_ROUTE_THRESHOLD ({}): {crossing:?}",
        holon_chem::fci::MPS_ROUTE_THRESHOLD
    );
    println!(
        "  For these, `fci::solve` routes to DMRG. An R2 comparison must call \
         `fci::solve_determinant` directly, or it would grade the DMRG bridge against the \
         referee while calling the result exact."
    );
}

/// THE SCOPE REFUSAL, demonstrated firing.
///
/// Standing question 4: a refusal nobody has watched fire is indistinguishable from a
/// refusal that cannot. The positive control comes first and is not decoration — a scope
/// check that refused everything would pass every negative case below.
#[test]
fn r2_an_out_of_scope_species_is_refused_by_name() {
    use holon_chem::elements::by_symbol;

    // POSITIVE CONTROL: every staked pair is inside the referee's model, so the gate is
    // not refusing its own campaign.
    for name in MIXTURES1_STAKED_PAIRS {
        let (a, b) = split_pair(name);
        assert_eq!(
            in_referee_scope(a, b),
            Ok(()),
            "{name} is staked by the freeze and outside the referee's declared scope; one \
             of the two is wrong"
        );
    }

    // The engine reaches far past the referee. Each of these must be refused BY NAME.
    for sym in ["K", "Ca", "Fe", "Zn", "Kr", "Ag", "Xe"] {
        let Some(sp) = by_symbol(sym) else {
            panic!("{sym} is not in the engine's registry; this test's premise is gone");
        };
        assert!(
            sp.z > MIXTURES1_REFEREE_Z_MAX,
            "{sym} (Z = {}) is no longer past the referee's scope of \
             {MIXTURES1_REFEREE_Z_MAX}; pick a species that is, or the refusal below is \
             untested",
            sp.z
        );
        let refusal = in_referee_scope(sp, holon_chem::elements::HYDROGEN)
            .expect_err("an out-of-scope species was admitted for grading");
        assert_eq!(
            refusal,
            ScopeRefusal::OutOfScope { symbol: sp.symbol, z: sp.z },
            "the refusal does not name the species that caused it"
        );
        // And in the other argument position, because a scope check that only looks at one
        // side of a pair is half a check.
        assert!(in_referee_scope(holon_chem::elements::HYDROGEN, sp).is_err());
        println!("  refused {sym} (Z = {}) by name", sp.z);
    }

    // The engine's reach and the referee's scope are DIFFERENT numbers, and the gap is the
    // thing this test exists for. If they ever coincide, the refusal above stops being
    // reachable and somebody should notice deliberately.
    let engine_z_max = holon_chem::elements::ALL_ELEMENTS
        .iter()
        .map(|s| s.z)
        .max()
        .unwrap_or(0);
    println!(
        "engine reaches Z = {engine_z_max}; this referee's model stops at Z = \
         {MIXTURES1_REFEREE_Z_MAX}. {} species are out of scope and are refused rather \
         than skipped.",
        engine_z_max.saturating_sub(MIXTURES1_REFEREE_Z_MAX)
    );
    assert!(
        engine_z_max > MIXTURES1_REFEREE_Z_MAX,
        "the engine no longer reaches past the referee, so the scope refusal is \
         unreachable and this gate has stopped testing anything"
    );
}

/// R2 proper. Ignored until the drop lands; see the module header.
#[test]
#[ignore = "waiting on the mixtures-referee drop in tests/data/mixtures1/"]
fn r2_the_staked_pairs_match_the_fifty_digit_referee() {
    let dir = referee_dir();
    let is_committed = std::env::var("MIXTURES1_REFEREE_DIR").is_err();

    let manifest = std::fs::read_to_string(dir.join("manifest.json"))
        .expect("manifest.json: the drop must declare its own coverage");
    let present = string_array(&manifest, "pairs_present");
    let owed = string_array(&manifest, "pairs_owed");
    let fingerprint = string_scalar(&manifest, "basis_fingerprint");

    // The invariant that makes shrinkage impossible: present + owed IS the staked set. A
    // pair may move from present to owed; it cannot leave both, which would shrink this
    // gate's coverage with nothing to say so.
    let mut union: Vec<String> = present.iter().chain(owed.iter()).cloned().collect();
    union.sort();
    let mut staked: Vec<String> = MIXTURES1_STAKED_PAIRS.iter().map(|s| s.to_string()).collect();
    staked.sort();
    assert_eq!(
        union, staked,
        "the manifest's pairs_present + pairs_owed is not the staked set"
    );
    assert!(!fingerprint.is_empty(), "the manifest declares no basis_fingerprint");

    // THE DROP DECLARES ITS OWN SCOPE, and it is cross-checked against the constant frozen
    // here rather than replacing it. Two lanes disagreeing about how far the referee's
    // model reaches is exactly the disagreement that produces a vacuous pass, and it is
    // invisible if either side simply reads the other's number.
    if manifest.contains("\"z_max\"") {
        let theirs: u32 = string_scalar(&manifest, "z_max")
            .parse()
            .expect("z_max is an integer");
        assert_eq!(
            theirs, MIXTURES1_REFEREE_Z_MAX,
            "the referee declares its model reaches Z = {theirs}; this gate is frozen at \
             Z = {MIXTURES1_REFEREE_Z_MAX}. One of the two is grading a different model."
        );
    } else {
        println!(
            "  NOTE: the drop declares no z_max. Scope is being enforced from this gate's \
             frozen {MIXTURES1_REFEREE_Z_MAX} alone, with nothing to cross-check it."
        );
    }

    // Cross-check the referee's own declared staked list against the constant frozen here,
    // rather than replacing one with the other.
    if manifest.contains("\"staked_pairs\"") {
        let mut theirs = string_array(&manifest, "staked_pairs");
        theirs.sort();
        assert_eq!(
            theirs, staked,
            "the referee's declared staked set differs from this gate's frozen one; one of \
             the two lanes is grading a different campaign"
        );
    }

    // Every declared pair has a file, and every pair file is declared. Neither direction is
    // redundant: the first catches a manifest promising what it did not ship, the second
    // catches a file being graded that nobody declared.
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
            "{file} is in the drop but the manifest does not declare {stem} present"
        );
    }

    if is_committed {
        let mut bytes = Vec::new();
        for name in MIXTURES1_STAKED_PAIRS {
            if present.iter().any(|p| p == name) {
                bytes.extend_from_slice(&std::fs::read(dir.join(format!("{name}.json"))).unwrap());
            }
        }
        bytes.extend_from_slice(&std::fs::read(dir.join("atoms.json")).unwrap());
        bytes.extend_from_slice(manifest.as_bytes());
        let got = fnv1a32(&bytes);
        assert_eq!(
            got, MIXTURES1_REFEREE_DIGEST,
            "the pinned referee drop has changed (digest {got:#010x}, pinned \
             {MIXTURES1_REFEREE_DIGEST:#010x}). Read the residuals below and re-pin \
             deliberately, rather than re-pinning the digest alone."
        );
    }

    println!(
        "referee drop at {} — basis {fingerprint}\n  COVERED ({} of {}): {}\n  OWED ({}): {}",
        dir.display(),
        present.len(),
        MIXTURES1_STAKED_PAIRS.len(),
        present.join(" "),
        owed.len(),
        owed.join(" ")
    );
    if !owed.is_empty() {
        println!(
            "  R2 IS PARTIALLY DISCHARGED. The staked pointwise bound is measured on the \
             {} pair(s) above; the {} owed pair(s) are NOT graded against a 50-digit \
             referee by this gate.",
            present.len(),
            owed.len()
        );
    }

    let mut worst_overall = 0.0f64;
    for name in present.iter() {
        let src = std::fs::read_to_string(dir.join(format!("{name}.json"))).unwrap();
        let grid = string_array(&src, "R_grid_bohr");
        let energies = string_array(&src, "E_hartree");
        assert_eq!(
            grid.len(),
            energies.len(),
            "{name}: the grid and the energy column are different lengths"
        );
        assert!(
            src.contains("\"derivative_provenance\""),
            "{name}: no derivative_provenance block. A file with no declared uncertainty is \
             REFUSED rather than given a flat bound — an absent bound must not read as a \
             zero one."
        );
        let (a, b) = split_pair(name);
        // The scope refusal, ENFORCED where the grading happens and not only in its own
        // test. A pair the referee's model does not cover is refused by name; it is never
        // quietly skipped, which would shrink this gate's coverage and report the
        // shrinkage as a pass.
        in_referee_scope(a, b).unwrap_or_else(|e| {
            panic!("{name} cannot be graded against this referee: {e:?}")
        });
        let mut worst = 0.0f64;
        let mut worst_at = 0usize;
        for (i, (r_s, e_s)) in grid.iter().zip(energies.iter()).enumerate() {
            let r: f64 = r_s.parse().expect("R is a decimal");
            let mine = exact_total(a, b, r);
            let d = decimal_minus_f64(e_s, mine).abs();
            if d > worst {
                worst = d;
                worst_at = i;
            }
        }
        println!(
            "  {name:>5}: worst |dE| = {worst:.3e} Ha at R = {} ({} points)",
            grid[worst_at],
            grid.len()
        );
        worst_overall = worst_overall.max(worst);
        assert!(
            worst <= MIXTURES1_STAKE_E,
            "{name}: worst pointwise energy disagreement {worst:.3e} Ha exceeds the staked \
             {MIXTURES1_STAKE_E:.0e}"
        );
    }
    println!("R2: worst over every covered pair = {worst_overall:.3e} Ha, stake {MIXTURES1_STAKE_E:.0e}");
    let _ = DERIVATIVE_MARGIN;
}
