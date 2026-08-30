//! E1, E2 and F1: the nobles close themselves, the halide column reproduces a trend of
//! nature, and the model's edge against nature is measured rather than denied.
//!
//! # Why these three sit in one file
//!
//! They share an instrument and they are staked against each other. E1 says the closed
//! shells do not bind; E2 says the open ones do, in a particular ORDER; F1 measures how the
//! error against nature moves down the same column E2 walks.
//!
//! # What is model and what is world
//!
//! Everything computed here is IN-MODEL: nonrelativistic Schrodinger, STO-3G, full CI in
//! that basis. The experimental numbers in [`EXPERIMENTAL_D_E`] are LABELLED CONTEXT and
//! never enter a computation -- they appear in exactly one gate, F1, whose whole subject is
//! the size of the gap between the two. Nothing is fitted to them and nothing is corrected
//! by them.
//!
//! # Why the record is banked, and what keeps that honest
//!
//! A curve's cost is dominated by the ERI assembly, quartic in the CARTESIAN basis size and
//! paid once per geometry, and a curve is about a hundred geometries. Xe2 is 58 Cartesian
//! functions -- 11.3 million integrals per geometry, an hour per curve. A gate that costs an
//! hour stops being run, so `examples/elements3_dimers.rs` banks the record and these gates
//! read it.
//!
//! That would be a promise rather than a measurement if nothing checked it, so
//! [`the_banked_record_still_describes_this_engine`] recomputes the cheapest species live
//! and requires it to reproduce its banked row BIT-FOR-BIT. If the engine has moved, the
//! cheap row says so, and the bank is trusted only for the rows too expensive to check.

use holon_chem::elements::by_symbol;
use holon_chem::pair::{atom_energy, generate_pair_table};

const HARTREE_PER_EV: f64 = 1.0 / 27.211386245988;

/// Experimental equilibrium dissociation energies, eV. LABELLED CONTEXT, NOT model output.
///
/// K. P. Huber and G. Herzberg, *Molecular Spectra and Molecular Structure IV: Constants of
/// Diatomic Molecules* (Van Nostrand Reinhold, 1979). `D_e`, not `D_0`: the well depth from
/// the bottom of the potential, which is what a Born-Oppenheimer curve computes, with no
/// zero-point energy in it. Comparing a computed `D_e` against a measured `D_0` would be
/// the commonest way to make this gate say something untrue -- they differ by the ZPE,
/// about 0.18 eV for HCl, which is five times the margin the middle row below turns on.
///
/// # Their precision, and what it can and cannot support
///
/// HCl's value was checked against the NIST Chemistry WebBook's spectroscopic constants for
/// the X state (omega_e = 2990.9463 cm^-1, omega_e x_e = 52.8186 cm^-1, from Rank et al.
/// 1962/1965): a ZPE of 0.18377 eV on D_0 = 4.4336 eV gives D_e = 4.617, against the 4.618
/// used here. HBr and HI were NOT independently re-derived, so treat all three as carrying
/// a few times 0.01 eV.
///
/// That uncertainty is small against the FALL these numbers are used to establish -- the
/// deficit moves about 1 eV across the column -- and it is NOT small against HBr's deficit,
/// which lands within about 0.03 eV of zero. So F1 below asserts the fall, and asserts the
/// two OUTER signs, and deliberately says nothing about the sign at bromine: where the
/// crossing happens is inside the input's own error bar, and a gate that claimed to locate
/// it would be reporting the uncertainty of a table it did not measure.
const EXPERIMENTAL_D_E: [(&str, f64); 3] = [("H/Cl", 4.618), ("H/Br", 3.922), ("H/I", 3.198)];

// ------------------------------------------------------------------ the banked record

#[derive(Debug, Clone)]
struct Atom {
    n_basis: usize,
    n_det: usize,
    e: f64,
}

#[derive(Debug, Clone)]
struct Pair {
    n_basis: usize,
    n_det: usize,
    knots: usize,
    r_min: f64,
    r_max: f64,
    bound: bool,
    r_e: f64,
    d_e: f64,
    k_e: f64,
    route: String,
}

struct Record {
    atoms: Vec<(String, Atom)>,
    pairs: Vec<(String, Pair)>,
}

impl Record {
    fn atom(&self, s: &str) -> &Atom {
        &self
            .atoms
            .iter()
            .find(|(k, _)| k == s)
            .unwrap_or_else(|| panic!("the banked record has no atom {s}"))
            .1
    }
    fn pair(&self, s: &str) -> &Pair {
        &self
            .pairs
            .iter()
            .find(|(k, _)| k == s)
            .unwrap_or_else(|| panic!("the banked record has no pair {s}"))
            .1
    }
}

fn bits(s: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(s, 16).expect("hex bit pattern"))
}

fn record() -> Record {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/elements3_dimers.txt");
    let text = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()));
    let (mut atoms, mut pairs) = (Vec::new(), Vec::new());
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        match f[0] {
            "knots" => {}
            "atom" => atoms.push((
                f[1].to_string(),
                Atom {
                    n_basis: f[2].parse().unwrap(),
                    n_det: f[3].parse().unwrap(),
                    e: bits(f[4]),
                },
            )),
            "pair" => pairs.push((
                f[1].to_string(),
                Pair {
                    n_basis: f[2].parse().unwrap(),
                    n_det: f[3].parse().unwrap(),
                    knots: f[4].parse().unwrap(),
                    r_min: f[5].parse().unwrap(),
                    r_max: f[6].parse().unwrap(),
                    bound: f[7] == "1",
                    r_e: if f[7] == "1" { bits(f[8]) } else { f64::NAN },
                    d_e: if f[7] == "1" { bits(f[9]) } else { f64::NAN },
                    k_e: if f[7] == "1" { bits(f[10]) } else { f64::NAN },
                    route: f[11].to_string(),
                },
            )),
            other => panic!("unknown record row {other:?}"),
        }
    }
    assert!(
        atoms.len() >= 6 && pairs.len() >= 6,
        "the banked record carries {} atoms and {} pairs; it is supposed to carry six of \
         each, and a gate driven by a truncated file checks nothing",
        atoms.len(),
        pairs.len()
    );
    Record { atoms, pairs }
}

/// The bank is a measurement of THIS engine, and stays one.
///
/// Recomputes the cheapest staked species and requires bit-identity with its banked row.
/// HCl is ten basis functions and a hundred determinants -- affordable every run -- and it
/// travels the same assembly, orthonormalisation, rotation and solve as Xe2 does. It cannot
/// prove Xe2's number is right, but it can prove the machine that produced it has not moved,
/// which is the difference between a banked measurement and a remembered one.
#[test]
fn the_banked_record_still_describes_this_engine() {
    let rec = record();
    let banked = rec.pair("H/Cl");
    let t = generate_pair_table(
        by_symbol("H").unwrap(),
        by_symbol("Cl").unwrap(),
        banked.knots,
    );
    let w = t.meta.well.expect("HCl binds");
    assert_eq!(
        (t.meta.n_basis, t.meta.n_det),
        (banked.n_basis, banked.n_det),
        "HCl's problem changed shape since the record was banked"
    );
    for (name, got, want) in [
        ("R_e", w.r_e, banked.r_e),
        ("D_e", w.d_e, banked.d_e),
        ("k_e", w.k_e, banked.k_e),
    ] {
        assert_eq!(
            got.to_bits(),
            want.to_bits(),
            "HCl's {name} is {got:.17e} and the banked record says {want:.17e}. The bank is \
             a measurement of this engine; if the engine has moved, every heavier row in the \
             record is stale too and the record must be regenerated, not adjusted."
        );
    }
    // The atomic references the well depths are measured against must also still hold.
    for sym in ["H", "Cl", "Br", "I", "Kr", "Xe"] {
        let a = rec.atom(sym);
        let e = atom_energy(by_symbol(sym).unwrap());
        assert_eq!(
            e.to_bits(),
            a.e.to_bits(),
            "{sym}'s atomic energy moved from the banked {:.17e} to {e:.17e}; every D_e in \
             the record is measured against these",
            a.e
        );
    }
    println!("bank check: HCl reproduces bit-for-bit, six atomic references unchanged");
}

/// E1, first half: krypton and xenon are closed shells in this model, exactly.
///
/// Asserted as a determinant COUNT, because that is the exact statement: one determinant
/// means every orbital the basis provides is doubly occupied, with no room for a single
/// excitation anywhere. It is also the carrier for the second half -- a pair of closed
/// shells is the reason to expect no bond, so the closure has to be established before the
/// absence of a well means anything.
#[test]
fn e1_the_heavy_nobles_are_closed_shells_exactly() {
    let rec = record();
    for (sym, n_basis) in [("Kr", 18usize), ("Xe", 27)] {
        let a = rec.atom(sym);
        assert_eq!(a.n_basis, n_basis, "{sym}'s minimal basis size");
        assert_eq!(
            a.n_det, 1,
            "{sym} is {} determinants, not one. Under six Cartesian d components it is not a \
             closed shell at all, and that is the observable which says the sixth component \
             does not belong in a minimal basis (see md::SPHERICAL_D).",
            a.n_det
        );
        assert!(a.e.is_finite(), "{sym}'s energy is not finite");
        println!("E1: {sym} is {n_basis} functions, 1 determinant, E = {:.9} Ha", a.e);
    }
    // The carrier for the count: the neighbouring halogens are NOT one determinant, so
    // "one" is a fact about closed shells rather than about the whole heavy registry.
    for sym in ["Br", "I"] {
        assert!(
            rec.atom(sym).n_det > 1,
            "{sym} is one determinant too, so the closure claim distinguishes nothing"
        );
    }
}

/// E1, second half: neither noble dimer has a well deeper than the schema's threshold.
///
/// A staked NEGATIVE. The threshold is the crate's own `WELL_MIN_DEPTH`, the same one under
/// which He2 and Ne2 report "repulsive only", so this asks the schema's question rather than
/// a new one chosen to be easy to pass.
#[test]
fn e1_the_noble_dimers_do_not_bind() {
    let rec = record();
    for sym in ["Kr", "Xe"] {
        let key = format!("{sym}/{sym}");
        let p = rec.pair(&key);
        assert!(
            !p.bound,
            "{sym}2 reports a well of depth {:.3e} hartree at R_e = {:.4} bohr. E1 stakes \
             that it has none deeper than the schema's 1e-4. Branch (b): investigate, do \
             not massage -- a bound noble dimer in a minimal basis with no dispersion in it \
             would be an artifact, and the first place to look is the basis-set \
             superposition error the counterpoise correction exists for.",
            p.d_e,
            p.r_e
        );
        // "No well" must be a statement about a sampled curve, not about an empty table.
        assert!(
            p.knots >= 8 && p.r_max > p.r_min,
            "{sym}2's curve is {} knots over [{}, {}]; the absence of a well means nothing \
             on a degenerate grid",
            p.knots,
            p.r_min,
            p.r_max
        );
        assert_eq!(
            p.n_det, 1,
            "{sym}2 is {} determinants; two closed shells brought together are still one",
            p.n_det
        );
        println!(
            "E1: {sym}2 repulsive over R = {:.2}..{:.1} bohr, {} knots, no well",
            p.r_min, p.r_max, p.knots
        );
    }
}

/// E2: the halide-hydride column trend, reproduced in kind.
///
/// The ORDERING is the stake and the numbers are the product. Nature's `D_e` falls from HCl
/// to HBr to HI; the claim is that a nonrelativistic minimal-basis full CI, given nothing
/// but `Z`, the masses and the basis, falls the same way.
#[test]
fn e2_the_hydrogen_halide_well_depths_fall_down_the_column() {
    let rec = record();
    let mut depths = Vec::new();
    for (key, _) in EXPERIMENTAL_D_E {
        let p = rec.pair(key);
        assert!(
            p.bound,
            "{key} has no well at all in this model, so the ordering E2 stakes cannot be \
             evaluated. Branch (b): report and investigate."
        );
        assert_eq!(
            p.route, "determinant",
            "{key} is one of E2's staked EXACT dimers and must not have travelled the DMRG \
             route; it reports {}",
            p.route
        );
        println!(
            "E2: {key}  R_e = {:.6} bohr  D_e = {:.6} Ha ({:.4} eV)  k_e = {:.5}  {} dets",
            p.r_e,
            p.d_e,
            p.d_e / HARTREE_PER_EV,
            p.k_e,
            p.n_det
        );
        depths.push((key, p.d_e));
    }
    for w in depths.windows(2) {
        assert!(
            w[0].1 > w[1].1,
            "E2 INVERSION: {} binds at {:.6} Ha and {} at {:.6} Ha, so the in-model well \
             depth does NOT fall down the halide column as staked. Branch (b): report it \
             and investigate. Do not reorder the stake.",
            w[0].0,
            w[0].1,
            w[1].0,
            w[1].1
        );
    }
}

/// E2's other staked exact dimer: Br2 binds, on the determinant route.
#[test]
fn e2_the_bromine_dimer_binds_exactly() {
    let rec = record();
    let p = rec.pair("Br/Br");
    assert_eq!(p.route, "determinant", "Br2 is a staked EXACT dimer");
    assert!(
        p.bound,
        "Br2 has no well in this model; branch (b), report and investigate"
    );
    println!(
        "E2: Br2  R_e = {:.6} bohr  D_e = {:.6} Ha ({:.4} eV)  {} dets, {} basis functions",
        p.r_e,
        p.d_e,
        p.d_e / HARTREE_PER_EV,
        p.n_det,
        p.n_basis
    );
}

/// F1: the relativistic fence, measured -- and the stake FIRED.
///
/// # The stake, and what happened to it
///
/// F1 stakes that the in-model deficit against experimental `D_e` GROWS down the halide
/// column, reasoning that relativity and core correlation are genuinely absent from this
/// model and are missed more as the halogen gets heavier. The prereg makes it explicitly
/// two-sided: if the gap does not grow, the fence claim as stated dies and is investigated.
///
/// It does not grow. Measured, experiment minus model, hartree: HCl +0.0214, HBr -0.0013,
/// HI -0.0149 -- that is +0.58, -0.03 and -0.40 eV. The deficit falls monotonically across
/// about one electronvolt, and it changes sign: the model underbinds HCl and overbinds HI.
/// **The fence claim as stated is dead**, and it is kept here marked dead rather than
/// restated in a direction that would fit.
///
/// The fall and the two outer signs are far outside the experimental input's uncertainty.
/// WHERE the crossing happens is not: HBr's deficit is 0.03 eV, which is the same size as
/// the error on the number it is measured against, so nothing here locates the crossing and
/// the gate below does not assert bromine's sign.
///
/// # What this gate is now
///
/// A regression on the measured direction, not a restatement of the stake. It asserts what
/// was found, so that if the number ever moves back, that is a change this suite reports
/// rather than absorbs.
///
/// # The investigation, which is not optional
///
/// A fired stake is a question, and the obvious competing cause is not relativity at all. A
/// dissociation energy computed as `E(A) + E(B) - E(AB)` in a finite basis carries basis-set
/// superposition error: at equilibrium each atom borrows its partner's basis functions, so
/// the molecule sits in a larger effective basis than the atoms do and the well comes out
/// too deep. The borrowing scales with what the partner brings -- chlorine offers nine
/// functions, iodine twenty-seven -- so BSSE grows down precisely the column F1 walks, and
/// pushes the deficit the OPPOSITE way from the missing relativity.
///
/// If that is the whole story then F1's PHYSICS may be right while its OBSERVABLE was
/// contaminated. That is a different claim from the one staked and does not inherit its
/// standing: `examples/elements3_counterpoise.rs` computes the corrected deficit, and until
/// it reports, the only thing established here is that the stake as written is dead.
#[test]
fn f1_the_fence_stake_fired_the_deficit_falls_down_the_column() {
    let rec = record();
    let mut gaps = Vec::new();
    for (key, ev) in EXPERIMENTAL_D_E {
        let p = rec.pair(key);
        let exp_ha = ev * HARTREE_PER_EV;
        let gap = exp_ha - p.d_e;
        println!(
            "F1: {key}  model {:.6} Ha ({:.4} eV)  experiment {exp_ha:.6} Ha ({ev:.3} eV)  \
             deficit {gap:+.6} Ha ({:+.4} eV, {:+.1}% of experiment)",
            p.d_e,
            p.d_e / HARTREE_PER_EV,
            gap / HARTREE_PER_EV,
            100.0 * gap / exp_ha
        );
        gaps.push((key, gap));
    }

    for w in gaps.windows(2) {
        assert!(
            w[1].1 < w[0].1,
            "the deficit no longer falls from {} ({:+.6} Ha) to {} ({:+.6} Ha). F1's stake \
             was that it GROWS, and that stake fired on the measurement recorded in this \
             test's documentation; if it now grows, this measurement or that one is wrong \
             and the difference has to be found before either is believed.",
            w[0].0,
            w[0].1,
            w[1].0,
            w[1].1
        );
    }
    // The sign change is the sharp part of the finding, so it is asserted rather than left
    // in prose: the model crosses from underbinding to overbinding inside this one column.
    assert!(
        gaps[0].1 > 0.0,
        "HCl no longer underbinds ({:+.6} Ha); the recorded finding was a sign CHANGE across \
         the column, which needs both signs present",
        gaps[0].1
    );
    assert!(
        gaps[2].1 < 0.0,
        "HI no longer overbinds ({:+.6} Ha); see above",
        gaps[2].1
    );
    println!(
        "F1 FIRED (recorded, not restated): deficit {:+.6} -> {:+.6} -> {:+.6} Ha down HCl, \
         HBr, HI -- it falls, and changes sign. The staked GROWTH is dead.",
        gaps[0].1, gaps[1].1, gaps[2].1
    );
}
