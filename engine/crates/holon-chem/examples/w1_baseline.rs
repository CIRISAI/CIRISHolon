//! Regenerate the W1 bit-identity baseline: every pre-ELEMENTS-3 species, as raw f64 bits.
//!
//! # What this is for
//!
//! ELEMENTS-3's gate W1 widens the FCI string masks from `u32` to `u64` so that heavy
//! dimers (Xe2 is 58 spatial orbitals) can be represented at all. The claim the gate makes
//! is that the widening costs NOTHING: every species the crate could already do comes back
//! unchanged. "Unchanged" here means bit-identical, not "agrees to 1e-12" — a widening
//! that perturbed a sum would be a change to the model, and a tolerance would hide it.
//!
//! So the baseline is captured as the IEEE-754 bit patterns of the energies and both
//! derivatives, in hex. Two f64 values with the same bits are the same number; two that
//! print the same to seventeen digits need not be.
//!
//! # Why an example that regenerates, rather than a table someone typed
//!
//! The same reason the grid rule travels with the file it describes: a pinned artifact
//! nobody can re-derive is a promise rather than a record. Run
//!
//! ```text
//! cargo run --release -p holon-chem --example w1_baseline > tests/data/w1_baseline.txt
//! ```
//!
//! to produce it. `tests/fci.rs`'s W1 gate reads that file and compares. Regenerating it
//! to make a failing gate pass would of course defeat it, which is why the baseline is
//! committed BEFORE the widening and the commit that widens does not touch it.

use holon_chem::dual::D2;
use holon_chem::elements::{ALL_ELEMENTS, by_symbol};
use holon_chem::pair::{electron_counts, pair_point, solve_geometry};

/// The banked ELEMENTS-1 pairs, plus the second-row closed shells that exercise the
/// widest determinant strings the crate could build before W1.
const PAIRS: &[(&str, &str)] = &[
    ("H", "H"),
    ("He", "He"),
    ("Li", "H"),
    ("H", "F"),
    ("F", "F"),
    ("Ne", "Ne"),
    ("Na", "H"),
    ("Cl", "H"),
    ("Ar", "Ar"),
    ("Cl", "Cl"),
];

/// Separations, in bohr. Fixed literals: the point is reproducibility, so the grid is
/// declared here and not derived from a curve whose range could itself move.
const SEPARATIONS: &[f64] = &[1.4, 2.0, 3.0, 5.0];

fn main() {
    println!("# W1 bit-identity baseline: f64 bit patterns, captured before the u32->u64 mask widening.");
    println!("# atom: symbol n_basis n_det E_bits dE_bits d2E_bits");
    println!("# pair: a/b r n_elec na/nb E_bits dE_bits d2E_bits");

    for sp in ALL_ELEMENTS {
        let s = solve_geometry(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        println!(
            "atom {} {} {} {:016x} {:016x} {:016x}",
            sp.symbol,
            s.n_basis,
            s.n_det,
            s.e.v.to_bits(),
            s.e.d.to_bits(),
            s.e.e.to_bits()
        );
    }

    for &(a, b) in PAIRS {
        let (sa, sb) = (by_symbol(a).unwrap(), by_symbol(b).unwrap());
        // Electron count and the S_z sector are properties of the pair, not of the
        // separation, so they are reported once rather than re-derived per knot.
        let (n_elec, na, nb) = electron_counts(&[sa, sb]);
        for &r in SEPARATIONS {
            let p = pair_point(sa, sb, r);
            println!(
                "pair {a}/{b} {r:.4} {n_elec} {na}/{nb} {:016x} {:016x} {:016x}",
                p.e.to_bits(),
                (-p.f).to_bits(),
                p.e2.to_bits()
            );
        }
    }
}
