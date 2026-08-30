//! R1, this lane's half: the atoms from potassium to xenon.
//!
//! # The split
//!
//! The referee lane holds Z = 1..18 at fifty digits (see the ELEMENTS-1 referee and the
//! R1 gate over eighteen atoms). This file holds Z = 19..54, and does not re-test theirs.
//!
//! # What is gated here and what is not
//!
//! Two things, both of which are affordable and neither of which is an energy compared
//! against a table this crate does not own:
//!
//! * the ground-state SPIN MULTIPLICITY of every heavy atom cheap enough to solve, DERIVED
//!   from `<S^2>` on the converged vector, against the periodic table's own value;
//! * the dual-route agreement, `sigma` against `sigma_reference`, on the same atoms.
//!
//! The energies themselves are in the banked record `engine/output/elements3/atoms.log`,
//! with their routes and their refusals. They are not gated against an external reference
//! because no fifty-digit referee reaches them yet: the ELEMENTS-1 referee's `CART` table
//! has no `l = 2` entry, its `_self_overlap` uses the `l = 1` formula for every non-zero
//! `l`, and its `STO3G_SHELLS` stops at neon. All nine of the atoms here that fall inside
//! R1's stated 3e4-determinant referee threshold carry d shells, so that build is owed
//! before the referee leg of R1 can exist. Saying so is better than gating against
//! something weaker and calling it done.
//!
//! # Why the multiplicity is the check worth having
//!
//! It is the strongest statement available for free. Nothing in this crate is told which
//! term symbol an atom has; the declared inputs are `Z`, the masses and the basis, and the
//! solver works in the MINIMAL `S_z` sector precisely so that a wrong guess about spin
//! cannot be baked in -- that sector contains every state of every multiplicity. The
//! multiplicity therefore comes OUT. That it comes out matching Hund's rules across two
//! rows nobody tuned is evidence about the model rather than about the arithmetic.

use holon_chem::dual::D2;
use holon_chem::elements::by_z;
use holon_chem::fci::{ci_ints, s_squared, solve_determinant, Order};
use holon_chem::pair::{electron_counts, geometry_problem};

/// `(Z, 2S+1)` for the heavy atoms cheap enough to solve in a test.
///
/// The multiplicities are the periodic table's, not this crate's: LABELLED CONTEXT, and the
/// only external numbers in this file. Every one is a small integer that a general reader
/// can check -- germanium's ground state is `3P`, arsenic's is `4S`, krypton's is `1S`.
///
/// The list is the p-block of both new rows plus the two nobles, chosen by DETERMINANT
/// COUNT rather than by which ones pass: everything at or under about 2.4e4 determinants,
/// which is what keeps this gate seconds rather than minutes. The heavier open shells
/// (potassium at 2.0e5, gallium at 1.2e5, indium at 1.0e6) are in the banked record and are
/// left out here for cost, not for result.
const EXPECTED_MULTIPLICITY: [(u32, u32); 9] = [
    (32, 3), // Ge  3P
    (33, 4), // As  4S
    (34, 3), // Se  3P
    (35, 2), // Br  2P
    (36, 1), // Kr  1S
    (51, 4), // Sb  4S
    (52, 3), // Te  3P
    (53, 2), // I   2P
    (54, 1), // Xe  1S
];

/// Both rows' p-blocks reproduce Hund's rules, with nothing about spin supplied.
#[test]
fn the_heavy_ground_multiplicities_come_out_rather_than_in() {
    for (z, want) in EXPECTED_MULTIPLICITY {
        let sp = by_z(z).unwrap();
        let (n_elec, na, nb) = electron_counts(&[sp]);
        // The carrier: the solver is in the MINIMAL Sz sector, which contains states of
        // every multiplicity. If it were in a sector chosen to match the answer, the
        // agreement below would be an assumption rather than a result.
        assert_eq!(
            na - nb,
            (n_elec % 2) as usize,
            "{} is not being solved in the minimal S_z sector, so its multiplicity would \
             be an input rather than an output",
            sp.symbol
        );

        let (space, mo, _) = geometry_problem(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        let sol = solve_determinant(&space, &mo);
        let s2 = s_squared(&space, &sol.vector);
        // S(S+1) = <S^2>  =>  2S+1 = sqrt(1 + 4 <S^2>).
        let mult = (1.0 + 4.0 * s2).sqrt();
        assert!(
            (mult - want as f64).abs() < 1e-6,
            "{}: <S^2> = {s2:.9} gives 2S+1 = {mult:.6}, and the periodic table says {want}. \
             The multiplicity is DERIVED from the converged vector in the minimal S_z \
             sector, so a disagreement is a statement about the model or the solver, not a \
             mislabelled input.",
            sp.symbol
        );
        println!(
            "R1: {:>2} {:>2}  {:>6} dets  E = {:>18.9} Ha  <S^2> = {s2:.6}  2S+1 = {mult:.3} \
             (table: {want})",
            z,
            sp.symbol,
            space.n_det,
            sol.e.v
        );
    }
}

/// The dual route, on the same atoms: two expressions of `H c` that share no loop.
///
/// `sigma` is the Knowles-Handy string factorisation and `sigma_reference` enumerates the
/// connected determinants and applies the Slater-Condon rules pair by pair. A matrix-VECTOR
/// comparison rather than an eigenvalue one, because eigenvalues are blind to the failure
/// this crate actually had: an interleaved spin-orbital ordering gives the same Hamiltonian
/// conjugated by a diagonal matrix of signs -- identical spectrum, different matrix.
#[test]
fn the_heavy_atoms_agree_between_two_independent_sigma_routes() {
    /// Determinant count past which the reference route is not bought here.
    ///
    /// `sigma_reference` is `O(N_det^2)` in Slater-Condon evaluations, so germanium's
    /// 23,409 determinants alone is half a billion of them and most of a minute. The
    /// remaining eight still span both new rows and every shell structure in them, which is
    /// what this route is for -- it checks the string factorisation, and the factorisation
    /// does not know how many determinants it is being run on.
    const MAX_N_DET: usize = 10_000;

    let mut worst = 0.0f64;
    let mut worst_sym = "";
    let mut checked = 0usize;
    for (z, _) in EXPECTED_MULTIPLICITY {
        let sp = by_z(z).unwrap();
        let (space, mo, _) = geometry_problem(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        if space.n_det > MAX_N_DET {
            continue;
        }
        checked += 1;
        let ci = ci_ints(&mo, Order::Value);
        let c = probe(space.n_det, 0x9E37_79B9_7F4A_7C15 ^ z as u64);
        let (mut x, mut y) = (vec![0.0; space.n_det], vec![0.0; space.n_det]);
        space.sigma(&ci, &c, &mut x);
        space.sigma_reference(&ci, &c, &mut y);
        let num = x
            .iter()
            .zip(y.iter())
            .map(|(p, q)| (p - q).abs())
            .fold(0.0, f64::max);
        let den = x.iter().map(|p| p.abs()).fold(0.0, f64::max).max(1e-300);
        let rel = num / den;
        // The carrier: the probe must actually excite the operator, or agreement is a
        // statement about two zeros.
        assert!(
            den > 1e-6,
            "{}'s sigma is essentially zero on the probe ({den:.3e}); the routes would agree \
             for the wrong reason",
            sp.symbol
        );
        if rel > worst {
            worst = rel;
            worst_sym = sp.symbol;
        }
    }
    assert!(
        checked >= 6,
        "only {checked} atoms were inside the reference route's budget; the comparison has \
         to span both new rows or it is testing one shell structure"
    );
    assert!(
        worst < 1e-12,
        "the two sigma routes disagree by {worst:.3e} relative, worst at {worst_sym}. They \
         share the integrals and the rules and nothing else, so a disagreement is in the \
         string factorisation."
    );
    println!(
        "R1: two sigma routes agree to {worst:.2e} relative over {checked} heavy atoms \
         (worst {worst_sym})"
    );
}

/// A deterministic pseudo-random probe. Deterministic so a failure reproduces;
/// pseudo-random so the comparison is not made on a vector both routes treat alike.
fn probe(n: usize, seed: u64) -> Vec<f64> {
    let mut s = seed | 1;
    (0..n)
        .map(|_| {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((s >> 33) as f64 / (1u64 << 30) as f64) - 1.0
        })
        .collect()
}
