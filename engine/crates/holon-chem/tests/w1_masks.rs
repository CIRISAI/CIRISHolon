//! W1: the FCI string masks are sixty-four bits wide, and the widening cost nothing.
//!
//! # What the gate claims
//!
//! ELEMENTS-3 needs determinant spaces that could not previously be WRITTEN DOWN. A string
//! mask carried one bit per spatial orbital in a `u32`, which capped the machinery at
//! thirty-two orbitals; xenon is 29 contracted STO-3G functions with six Cartesian
//! components per d shell, so Xe2 is 58 orbitals and was unrepresentable. W1 widens the
//! mask to `u64` and claims that every species the crate could already do is UNAFFECTED.
//!
//! # Why bit-identity and not a tolerance
//!
//! Because a tolerance wide enough to be comfortable is wide enough to hide the failure
//! this gate is for. The widening reorders nothing and rounds nothing, so the correct
//! prediction is not "agrees to 1e-12" but "is the same f64". Anything less than exact
//! equality would mean the widening had changed the model, and a gate that accepted it
//! would be measuring its own tolerance rather than the code.
//!
//! The baseline in `tests/data/w1_baseline.txt` was generated and committed BEFORE the
//! widening, by `examples/w1_baseline.rs`. This gate does not contain a second copy of the
//! species list: it reads the baseline's own rows and recomputes each one, so the file
//! drives the test and the two cannot drift apart.
//!
//! # The plant (M-PLANT-OBS, M-PLANT-SECTOR)
//!
//! A regression that watched only species below the old cap would pass forever without
//! ever testing the sector the widening changed. So the plant reintroduces the defect
//! behind the widened path -- [`FciSpace::with_mask_width`] builds the space a mask of a
//! given width could actually address -- and the gate requires it to FIRE above thirty-two
//! orbitals and stay SILENT at or below. The carrier is asserted nonzero before the plant
//! is scored, per M-PLANT-SECTOR: a plant on an empty sector proves nothing.

use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::{solve_determinant, FciSpace, Strings, MAX_ORB};
use holon_chem::pair::{electron_counts, geometry_problem, pair_point, solve_geometry};

/// The mask width the crate had through ELEMENTS-1 and MIXTURES-1, and the plant's setting.
const OLD_MASK_WIDTH: usize = 32;

fn baseline_text() -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/w1_baseline.txt");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("cannot read {}: {e}", p.display()))
}

#[test]
fn the_widening_reproduces_every_banked_species_bit_for_bit() {
    let text = baseline_text();
    let mut atoms = 0usize;
    let mut pairs = 0usize;

    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        match f[0] {
            // atom SYMBOL n_basis n_det E dE d2E
            "atom" => {
                let sp = by_symbol(f[1]).unwrap_or_else(|| panic!("unknown symbol {}", f[1]));
                let s = solve_geometry(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
                let (n_basis, n_det): (usize, usize) = (f[2].parse().unwrap(), f[3].parse().unwrap());
                assert_eq!(
                    (s.n_basis, s.n_det),
                    (n_basis, n_det),
                    "{}: the determinant space itself changed shape under the widening",
                    f[1]
                );
                for (col, got, want) in [
                    ("E", s.e.v, f[4]),
                    ("dE", s.e.d, f[5]),
                    ("d2E", s.e.e, f[6]),
                ] {
                    let want = u64::from_str_radix(want, 16).unwrap();
                    let sym = f[1];
                    assert_eq!(
                        got.to_bits(),
                        want,
                        "{sym} {col}: {got:.17e} (bits {:016x}) is not the banked bit \
                         pattern (bits {want:016x}). The mask widening was supposed to \
                         change which spaces can be ADDRESSED and nothing about what they \
                         evaluate to.",
                        got.to_bits()
                    );
                }
                atoms += 1;
            }
            // pair A/B r n_elec na/nb E dE d2E
            "pair" => {
                let (a, b) = f[1].split_once('/').expect("pair row is A/B");
                let (sa, sb) = (by_symbol(a).unwrap(), by_symbol(b).unwrap());
                let r: f64 = f[2].parse().unwrap();
                let p = pair_point(sa, sb, r);
                let (n_elec, na, nb) = electron_counts(&[sa, sb]);
                assert_eq!(
                    (n_elec.to_string(), format!("{na}/{nb}")),
                    (f[3].to_string(), f[4].to_string()),
                    "{a}/{b}: electron count or S_z sector moved"
                );
                for (col, got, want) in
                    [("E", p.e, f[5]), ("dE", -p.f, f[6]), ("d2E", p.e2, f[7])]
                {
                    let want = u64::from_str_radix(want, 16).unwrap();
                    assert_eq!(
                        got.to_bits(),
                        want,
                        "{a}/{b} at R = {r} {col}: {got:.17e} (bits {:016x}) is not the \
                         banked bit pattern (bits {want:016x})",
                        got.to_bits()
                    );
                }
                pairs += 1;
            }
            other => panic!("unknown baseline row kind {other:?}"),
        }
    }

    // The file drives the test, so an EMPTY or truncated file would otherwise pass
    // silently while checking nothing -- the same shape as a plant on an empty sector.
    assert!(
        atoms >= 18 && pairs >= 40,
        "the baseline covers only {atoms} atoms and {pairs} pair points; it is supposed to \
         carry every species the crate could do before the widening (H..Ar and the banked \
         pairs), and a gate driven by a file that lost its rows checks nothing"
    );
    println!("W1: {atoms} atoms and {pairs} pair points reproduce bit-for-bit");
}

/// The widened bound is the one the heavy dimers actually need.
#[test]
fn the_mask_admits_the_species_elements_three_needs() {
    assert_eq!(MAX_ORB, 64, "W1 widened the mask to sixty-four spatial orbitals");
    // Xe is 29 contracted functions (1s, 2sp, 3spd, 4spd, 5sp with six Cartesian d
    // components), so the heaviest dimer in scope is 58 orbitals. Stated as the inequality
    // the gate needs rather than as a number to be trusted.
    assert!(
        58 <= MAX_ORB,
        "Xe2 is 58 spatial orbitals and must be representable for ELEMENTS-3 to be possible"
    );
    assert!(
        58 > OLD_MASK_WIDTH,
        "if the heaviest species in scope fit the OLD mask, W1 would be unmotivated"
    );
}

/// Plant A, at the level the widening acts on: the string enumeration is exactly what the
/// mask width can hold, and the two widths part company precisely above thirty-two.
#[test]
fn the_string_enumeration_is_bounded_by_the_mask_width() {
    // Silent below: at or under the old width the parameter is inert, for every occupancy.
    for n_orb in [8usize, 20, 32] {
        for n_elec in [1usize, 2, 3] {
            let wide = Strings::with_mask_width(n_orb, n_elec, MAX_ORB);
            let narrow = Strings::with_mask_width(n_orb, n_elec, OLD_MASK_WIDTH);
            assert_eq!(
                wide.masks, narrow.masks,
                "at {n_orb} orbitals a 32-bit mask holds every string a 64-bit mask does, \
                 so the plant must be inert here"
            );
        }
    }

    // Fires above: the narrow mask cannot address the high orbitals, so it enumerates a
    // strictly smaller space -- silently, which is the whole defect.
    for (n_orb, n_elec) in [(36usize, 1usize), (36, 2), (58, 1), (64, 2)] {
        let wide = Strings::with_mask_width(n_orb, n_elec, MAX_ORB);
        let narrow = Strings::with_mask_width(n_orb, n_elec, OLD_MASK_WIDTH);
        let expect_wide = binomial(n_orb, n_elec);
        let expect_narrow = binomial(OLD_MASK_WIDTH, n_elec);
        assert_eq!(wide.len(), expect_wide, "C({n_orb},{n_elec}) strings expected");
        assert_eq!(
            narrow.len(),
            expect_narrow,
            "a {OLD_MASK_WIDTH}-bit mask can only hold strings inside its low {OLD_MASK_WIDTH} orbitals"
        );
        assert!(
            wide.len() > narrow.len(),
            "the plant must lose strings at {n_orb} orbitals or it is not testing the \
             sector the widening changed"
        );
    }
}

fn binomial(n: usize, k: usize) -> usize {
    (0..k).fold(1usize, |acc, i| acc * (n - i) / (i + 1))
}

/// Plant B, at the level the GATE reads: a 32-bit truncation behind the widened path moves
/// an energy above thirty-two orbitals and leaves it bit-identical below.
///
/// # Why the electron count is one per spin and not the species'
///
/// The plant is about the MASK, not about chemistry. A real >32-orbital neutral species
/// fills so many orbitals that a 32-bit mask cannot hold a single one of its strings, so
/// the truncated space is EMPTY -- the plant would fire, but by collapse rather than by a
/// wrong number, and a gate that only ever sees a collapse has not shown that a narrow
/// mask can return a plausible wrong answer. One electron per spin in the same 36-orbital
/// integrals keeps BOTH spaces non-empty, so the plant fires the way the real defect did:
/// a smaller space, a finite energy, and nothing to say it was the wrong problem.
#[test]
fn the_mask_plant_fires_above_thirty_two_orbitals_and_is_silent_below() {
    // --- above: four argon centres, 36 spatial orbitals.
    let ar = by_symbol("Ar").unwrap();
    let (space, mo, _nuc) = geometry_problem(
        &[ar, ar, ar, ar],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(6.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(6.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::var(6.0)],
        ],
    );
    let n_orb = space.n_orb;
    assert!(
        n_orb > OLD_MASK_WIDTH,
        "the plant needs a space the old mask could not address; this one is {n_orb} orbitals"
    );

    let wide = FciSpace::with_mask_width(n_orb, 1, 1, MAX_ORB);
    let narrow = FciSpace::with_mask_width(n_orb, 1, 1, OLD_MASK_WIDTH);

    // M-PLANT-SECTOR: the carrier is the divergence above bit 31, asserted NONZERO before
    // the plant is scored. Both spaces must be non-empty, and they must differ.
    assert_eq!(wide.n_det, n_orb * n_orb, "one electron per spin over {n_orb} orbitals");
    assert_eq!(narrow.n_det, OLD_MASK_WIDTH * OLD_MASK_WIDTH);
    assert!(
        narrow.n_det > 0 && wide.n_det > narrow.n_det,
        "carrier is empty: the plant has nothing to act on"
    );

    let e_wide = solve_determinant(&wide, &mo).e.v;
    let e_narrow = solve_determinant(&narrow, &mo).e.v;
    assert!(
        e_wide.is_finite() && e_narrow.is_finite(),
        "both sides of the plant must return a number; a NaN is not the failure being planted"
    );
    let gap = (e_wide - e_narrow).abs();
    assert!(
        gap > 1e-6,
        "PLANT MISSED: a 32-bit mask over {n_orb} orbitals returned {e_narrow:.12} against \
         the widened path's {e_wide:.12} -- a gap of {gap:.3e}. The plant is supposed to \
         drop orbitals 32..{n_orb} from the space entirely, so an agreement here means the \
         regression is not watching the sector the widening changed."
    );
    println!(
        "W1 plant fires: {n_orb} orbitals, {} dets vs {} dets, dE = {gap:.6e} hartree",
        wide.n_det, narrow.n_det
    );

    // --- below: two argon centres, 18 orbitals, same construction throughout.
    let (space2, mo2, _) = geometry_problem(
        &[ar, ar],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::var(6.0)],
        ],
    );
    let m = space2.n_orb;
    assert!(m <= OLD_MASK_WIDTH, "the control must sit inside the old mask");
    let w2 = solve_determinant(&FciSpace::with_mask_width(m, 1, 1, MAX_ORB), &mo2).e.v;
    let n2 = solve_determinant(&FciSpace::with_mask_width(m, 1, 1, OLD_MASK_WIDTH), &mo2).e.v;
    assert_eq!(
        w2.to_bits(),
        n2.to_bits(),
        "PLANT LEAKED: at {m} orbitals the mask width is supposed to be inert, but the \
         two widths gave {w2:.17e} and {n2:.17e}. A plant that fires below its sector \
         would make the gate above meaningless."
    );
    println!("W1 plant silent below: {m} orbitals, bit-identical across both widths");
}
