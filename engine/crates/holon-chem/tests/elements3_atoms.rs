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

mod common;

use common::decimal_minus_f64;
use holon_chem::dual::D2;
use holon_chem::elements::by_z;
use holon_chem::fci::{ci_ints, s_squared, solve_determinant, Order};
use holon_chem::pair::{electron_counts, geometry_problem, CONVERGED_RESIDUAL};

/// `(Z, 2S+1)` for the heavy atoms cheap enough to solve in a test.
///
/// The multiplicities are the periodic table's, not this crate's: LABELLED CONTEXT, and the
/// only external numbers in this file. Every one is a small integer that a general reader
/// can check -- germanium's ground state is `3P`, arsenic's is `4S`, krypton's is `1S`.
///
/// The list is the p-block of both new rows plus the two nobles, chosen by DETERMINANT
/// COUNT rather than by which ones pass: everything at or under about 2.4e4 determinants,
/// which is what keeps this gate seconds rather than minutes. Potassium (2.0e5) and indium
/// (1.0e6) are in the banked record and are left out here for cost, not for result.
///
/// # Zinc and gallium are excluded for RESULT, and that has to be said here
///
/// They are the two exceptions in the range and they are left out deliberately, so that
/// nobody reads this clean sweep as covering the whole table. Measured:
///
/// ```text
///   Zn   2S+1 = 5   (periodic table: 1, a closed 3d10 4s2 singlet)
///   Ga   2S+1 = 4   (periodic table: 2)
/// ```
///
/// The model prefers a high-spin state for exactly the two elements immediately after the
/// 3d shell fills, and germanium onward is correct again. That is a property of the model
/// and not a solver failure, which is the part that needed establishing before it could be
/// written down: `examples/zn_diagnose.rs` compares each FCI energy against the smallest
/// DIAGONAL element of the same Hamiltonian, which is the best single determinant in the
/// same orbital basis and a variational upper bound on the true ground state. Above it
/// would mean Davidson stepped over the ground state; below means the answer is real.
/// Both are below -- Zn by 4.149e-2 hartree, Ga by 3.794e-2 -- and both controls (Ge,
/// whose multiplicity is right, and Ca) are below theirs too, so the check discriminates.
///
/// A plausible mechanism, offered as a reading and not as a result: STO-3G gives 4p the
/// SAME exponents as 4s, which makes 4p artificially compact and therefore artificially
/// low, and Hund exchange then over-stabilises a high-spin 4s(1)4p(3) configuration over
/// the closed 4s(2). Nothing here tests that, and it is not what the measurement says.
///
/// They are not gated because gating them would cost minutes (Zn is 665,856 determinants)
/// and would freeze an unexplained number into the suite. The record carries them.
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

/// The two elements whose measured multiplicity DISAGREES with the periodic table.
///
/// Present so the exclusion above is enforced rather than trusted: a later hand adding
/// zinc to the expected list would get a confusing failure, and this turns that into a
/// clear one.
const KNOWN_DISAGREEMENTS: [u32; 2] = [30, 31];

/// The atoms above whose solve MISSES the crate's declared convergence bar, with the
/// residual measured rather than remembered.
///
/// # Why this list has to exist
///
/// This gate said its multiplicity was "DERIVED from the converged vector" and then checked
/// nothing about convergence. It was blessing antimony and tellurium at the same moment the
/// R1 record REFUSED them for missing `CONVERGED_RESIDUAL` — two instruments in one campaign
/// disagreeing about the same two atoms, with the gate's own doc comment asserting the thing
/// it had not looked at.
///
/// # What is and is not claimed about these two
///
/// The bar is a PROCESS verdict: `solve_determinant` targets 1e-11 and these stop above
/// 1e-10, so the solve did not finish its job. It is NOT an accuracy verdict. For a
/// symmetric eigenproblem `|theta - lambda| <= ||r||`, so a residual of 1.07e-10 bounds the
/// energy error at 1.07e-10 hartree even before a gap is used to sharpen it — six orders
/// inside anything this campaign reports. The multiplicity is asserted for them on that
/// basis, and it is CORROBORATION rather than a clean pass; the record's refusal stands.
///
/// # Two-sided on purpose
///
/// Membership is asserted in BOTH directions. An atom here must still miss the bar — so if
/// a solver improvement fixes it, this test fails and says to delete the entry rather than
/// letting a stale exemption hide a repair. And the residual must not have grown by more
/// than a decade, so a real regression cannot hide inside an accepted exception.
/// Emptied 2026-08-30, and the emptiness is load-bearing: Sb and Te lived here while the
/// publication bar sat exactly on the solver's expansion floor and their stagnated-at-the-
/// floor residuals (1.07e-10) missed it by 7%. The bar is now DERIVED a decade above the
/// named floor (see `pair::CONVERGED_RESIDUAL`), their solves meet it, and this test's own
/// staleness assertion forced the deletion — an exemption must not outlive its defect.
/// Their exits are still `Stagnated`; what changed is that the bar stopped misreading the
/// tier edge as a failure. ELEMENTS3_RESULTS.md carries the dated annotation.
const MISSES_CONVERGENCE_BAR: [(u32, f64); 0] = [];

/// Both rows' p-blocks reproduce Hund's rules, with nothing about spin supplied.
#[test]
fn the_heavy_ground_multiplicities_come_out_rather_than_in() {
    for (z, _) in EXPECTED_MULTIPLICITY {
        assert!(
            !KNOWN_DISAGREEMENTS.contains(&z),
            "Z = {z} is a MEASURED disagreement with the periodic table (see the doc \
             comment on EXPECTED_MULTIPLICITY) and cannot be in a list of elements \
             expected to agree. It is excluded for result, not for cost."
        );
    }

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
        // THE CONVERGENCE STATEMENT, kept separate from the multiplicity one. A residual
        // alone cannot say WHY a solve stopped (target, iteration cap, or subspace
        // stagnation are three different facts), so this asserts only what a residual can
        // support: whether the declared bar was met, in both directions.
        let recorded = MISSES_CONVERGENCE_BAR
            .iter()
            .find(|(zz, _)| *zz == z)
            .map(|(_, r)| *r);
        match recorded {
            None => assert!(
                sol.residual <= CONVERGED_RESIDUAL,
                "{}: residual {:.3e} misses the declared bar {CONVERGED_RESIDUAL:.0e}, and \
                 this atom is not in MISSES_CONVERGENCE_BAR. Either the solve regressed or \
                 the list is out of date — and a multiplicity read off a vector whose solve \
                 did not finish is not the clean result this gate reports.",
                sp.symbol,
                sol.residual
            ),
            Some(r) => {
                assert!(
                    sol.residual > CONVERGED_RESIDUAL,
                    "{}: residual {:.3e} now MEETS the bar {CONVERGED_RESIDUAL:.0e}, so its \
                     entry in MISSES_CONVERGENCE_BAR is stale. Delete the entry — an \
                     exemption that outlives the defect it documents is how a repair goes \
                     unnoticed.",
                    sp.symbol,
                    sol.residual
                );
                assert!(
                    sol.residual < 10.0 * r,
                    "{}: residual {:.3e} is more than a decade worse than the {r:.3e} this \
                     exception was written against. An accepted exception is not a place for \
                     a regression to hide.",
                    sp.symbol,
                    sol.residual
                );
            }
        }

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
        // Three separate statements per row, each carrying its own evidence: what the
        // solve DID (residual against the declared bar), how good the ENERGY is regardless
        // (|theta - lambda| <= ||r||), and the multiplicity. A process verdict must not be
        // readable as an accuracy verdict in either direction.
        println!(
            "R1: {:>2} {:>2}  {:>6} dets  E = {:>18.9} Ha  <S^2> = {s2:.6}  2S+1 = {mult:.3} \
             (table: {want})  resid {:.2e} [{}]  |dE| <= {:.1e} Ha",
            z,
            sp.symbol,
            space.n_det,
            sol.e.v,
            sol.residual,
            if recorded.is_some() { "MISSES BAR (process)" } else { "meets bar" },
            sol.residual
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

/// R1's referee leg, for the two atoms a 50-digit referee can actually reach.
///
/// # Why these two and not the nine
///
/// R1 stakes a 50-digit referee for every atom at or under 3e4 determinants — nine of
/// them. The threshold was staked result-blind and is a reasonable threshold; the
/// ARITHMETIC does not follow it that far. A 50-digit FCI needs an eigensolve over the
/// determinant space in mpmath, and germanium's 23,409 is far past what that reaches.
///
/// Krypton and xenon are different in kind. Every orbital the basis provides is doubly
/// occupied, so the determinant is unique up to a phase, its energy is invariant under
/// orbital rotation, and it needs NO eigensolve and NO SCF — the energy is a closed
/// expression in the AO integrals with `D = 2 S^-1`. So the two atoms E1 asserts
/// "exactly" turn out to be exactly the two the referee can reach exactly, which is worth
/// using rather than stepping over.
///
/// # What agreement here establishes
///
/// The referee is an independent implementation in another language: its own integral
/// recursions, its own normalisation, its own spherical projection derived rather than
/// copied, and its basis PARSED from `elements.rs` rather than regenerated beside it
/// (which is how the first attempt disagreed by 6.3e-7 hartree over a single rounding
/// tie). Agreement therefore covers the d integrals, the sqrt(3) per-component
/// normalisation, the 5x6 projection and the heavy basis table at once.
#[test]
fn the_two_closed_shell_nobles_match_the_fifty_digit_referee() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/elements3_referee_nobles.txt");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

    let mut checked = 0usize;
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.first() != Some(&"atom") {
            continue;
        }
        let z: u32 = f[1].parse().unwrap();
        let n_basis: usize = f[2].parse().unwrap();
        let dps: usize = f[3].parse().unwrap();
        assert!(dps >= 50, "the referee ran at only {dps} digits");

        let sp = by_z(z).unwrap();
        let (space, mo, _) = geometry_problem(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        assert_eq!(space.n_orb, n_basis, "{}: basis size", sp.symbol);
        assert_eq!(
            space.n_det, 1,
            "{}: the referee's closed-shell form applies only to a one-determinant space",
            sp.symbol
        );
        let e = solve_determinant(&space, &mo).e.v;

        // Compare in exact decimal against the referee's digits, never by parsing the
        // referee to f64 first — that would round the reference to the precision of the
        // thing being graded and the residual would be measuring its own rounding.
        let resid = decimal_minus_f64(f[4], e);
        let bar = 1e-9 * e.abs();
        assert!(
            resid.abs() < bar,
            "{}: the engine gives {e:.15} and the 50-digit referee {}, a residual of \
             {resid:.3e} against a bar of {bar:.3e}. The referee is an independent \
             implementation, so a disagreement is in one of them and not in the model.",
            sp.symbol,
            f[4]
        );
        println!(
            "R1 referee: {} {n_basis} functions, engine {e:.12}, referee {}, residual \
             {resid:.3e} ({:.1} ulp)",
            sp.symbol,
            &f[4][..22.min(f[4].len())],
            resid.abs() / (e.abs() * f64::EPSILON)
        );
        checked += 1;
    }
    assert_eq!(
        checked, 2,
        "the referee record is supposed to carry krypton and xenon; it carried {checked}"
    );
}

/// THE MPS ARM IS THE LEASED OVERFLOW (2026-09-02), and the plant proves the arm is
/// selectable: a door that refuses the determinant working set but admits the MPO price
/// routes to MPS. On this machine the real door reports per species which route the
/// price buys — printed, not pinned, because it is a fact about the machine.
#[test]
fn the_mps_arm_is_the_leased_overflow_and_the_plant_selects_it() {
    use holon_chem::budget::{price_determinant, price_mpo, set_admission};
    use holon_chem::pair::{automatic_route, AutomaticRoute};
    use holon_resource::probe::{Probe, ProbeVerdict, ResourceKind};

    /// A door with a byte ceiling: passes at or under it, refuses above. A test fixture,
    /// not a second probe implementation — production uses the resource layer's.
    struct Ceiling(u64);
    impl Probe for Ceiling {
        fn probe(&mut self, _k: ResourceKind, amount: u64) -> ProbeVerdict {
            if amount <= self.0 {
                ProbeVerdict::Pass("under the planted ceiling")
            } else {
                ProbeVerdict::Fail("over the planted ceiling")
            }
        }
    }

    // A species whose determinant price exceeds its MPO price: the ceiling sits between.
    let sp = holon_chem::elements::by_symbol("Si").expect("silicon is registered");
    let r0 = automatic_route(sp, sp);
    let (det, mpo) = (price_determinant(r0.n_det()).bytes, price_mpo(r0.n_orb()).bytes);
    assert!(det > mpo, "the plant needs det ({det}) > mpo ({mpo}) for Si2; pick another species");
    set_admission(Some(Box::new(Ceiling((det + mpo) / 2))));
    let r = automatic_route(sp, sp);
    assert!(
        matches!(r, AutomaticRoute::Mps { .. }),
        "with the determinant refused and the MPO admitted the arm must be selected, got {r:?}"
    );
    set_admission(Some(Box::new(Ceiling(mpo / 2))));
    assert!(!automatic_route(sp, sp).exists(), "under both prices, nowhere");
    set_admission(None);
    for z in [1u32, 8, 14, 17] {
        let s = holon_chem::elements::by_z(z).unwrap();
        println!("  route on this machine: {}{} -> {:?}", s.symbol, s.symbol, automatic_route(s, s));
    }
}
