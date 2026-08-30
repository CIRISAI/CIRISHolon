//! MIXTURES-1: the pair-table bank's gates, and the plants that show they can fire.
//!
//! Contract: `conformance/atomworld/MIXTURES1_PREREG.md`. This file carries the engine
//! halves of **B1** (the bank is exact where the single table was), **C1** (conservation
//! in a mixed box), and the three plants — the swapped table, the wrong mass, and the DMRG
//! label — each of which is DEMONSTRATED FIRING rather than asserted to be watching.
//!
//! # The standing rule this file exists to honour
//!
//! Every new gate must demonstrate a failing case before it is trusted. A refusal nobody
//! has watched fire is indistinguishable from a refusal that cannot, and this repository
//! has shipped one of those. So each provenance rule below has a test that breaks exactly
//! that rule and requires the specific refusal, and each plant asserts its CARRIER is
//! nonzero in the sector it acts on before it scores the plant — a plant on an empty
//! sector proves nothing and, per M-PLANT-SECTOR, VOIDs.
//!
//! # Why the fixtures are H, He and Li rather than H and Cl
//!
//! The campaign's product is an 8 H + 8 Cl gas, and chlorine is not a test-suite species:
//! measured on the campaign machine at 24 knots, the Cl-Cl curve costs 96 s in release and
//! the H-Cl curve 10 s, against 0.22 s for H-H and 3.2 s for H-Li — and the suite must be
//! green in the DEBUG profile too, where those become minutes. The physics under test here
//! is "does each pair get its own curve, its own reduced mass, and its own criterion", and
//! that is a question about the bank, not about chlorine. The staked H + Cl instances of
//! plants (i) and (ii) run in the P1 harness, where the chlorine curves are being paid for
//! anyway.
//!
//! Helium and lithium each make a different fixture sharp, and both are needed:
//!
//! * HELIUM does not bind hydrogen at ANY separation in this model — `locate_well` returns
//!   `None` for H-He, because the closed shell refuses and nothing in the code knows that
//!   helium is noble. So H-H and H-He disagree about the SIGN of the interaction and about
//!   the bonded verdict at the same separation, which is the sharpest statement B1's mixed
//!   fixture can make: one table cannot produce both readings.
//! * LITHIUM binds hydrogen, at 2.924 bohr against H2's 1.389. That gives plant (i) the
//!   carrier the freeze names — an equilibrium separation that MOVES when the wrong curve
//!   is served — which a pair with no well cannot supply.

#[path = "common/b1_dump.rs"]
mod b1_dump;

use holon_chem::elements::{Species, HELIUM, HYDROGEN, LITHIUM};
use holon_chem::pair::PairTable;
use holon_render::bank::{
    D1Admission, Host, Refusal, Route, Source, TableProvenance, D1_RECORD, MAX_SPECIES,
    MAX_TABLES, RESOLVABLE_UNCERTAINTY,
};
use holon_render::sim::{Boundary, Dims, Sim, MAX_ATOMS};
use holon_render::{load_pair_table, TABLE_OK};
use std::sync::OnceLock;

/// Knots per fixture curve. Small on purpose: the interpolant's accuracy is not what these
/// gates measure, and every knot is a full CI solve at both ends of the debug profile.
const FIXTURE_KNOTS: usize = 24;

/// Per-atom radial closing speed for a placed pair, bohr per atomic time unit.
///
/// Chosen so the relative kinetic energy is a few hundredths of a hartree: enough to lift
/// a pair strictly off its own turning point (see [`place_pair`]) and small against H2's
/// 0.204 Ha well, so a pair inside that well stays inside it. Nothing downstream is
/// sensitive to the value; the fixtures assert what they need from the readings rather
/// than from this number.
const CLOSING_SPEED: f64 = 0.005;

// ------------------------------------------------------------------ the curve fixtures
//
// Generated ONCE per test binary. A `PairTable` is a heap object, so caching it costs a
// pointer and saves tens of seconds per test in debug.

fn curve(a: Species, b: Species) -> &'static PairTable {
    macro_rules! cached {
        ($cell:ident, $x:expr, $y:expr) => {{
            static $cell: OnceLock<Box<PairTable>> = OnceLock::new();
            $cell.get_or_init(|| {
                Box::new(holon_chem::pair::generate_pair_table($x, $y, FIXTURE_KNOTS))
            })
        }};
    }
    let (lo, hi) = if a.z <= b.z { (a, b) } else { (b, a) };
    match (lo.z, hi.z) {
        (1, 1) => cached!(HH, HYDROGEN, HYDROGEN),
        (1, 2) => cached!(HHE, HYDROGEN, HELIUM),
        (1, 3) => cached!(HLI, HYDROGEN, LITHIUM),
        (2, 2) => cached!(HEHE, HELIUM, HELIUM),
        _ => panic!(
            "no fixture curve for {}{} — add one deliberately and price it first",
            lo.symbol, hi.symbol
        ),
    }
}

/// A scene holding `species`, with every pair type they form banked.
///
/// Atoms are placed by the caller afterwards; this only fills the bank and sets `n`.
fn scene(species: &[Species]) -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    s.boundary = Boundary::Open;
    s.dims = Dims::Two;
    s.reset(species.len());
    for (i, sp) in species.iter().enumerate() {
        assert!(s.set_species(i, *sp), "the bank refused species {}", sp.symbol);
    }
    // Every pair type the atoms form, and nothing else: a species with one atom in the
    // scene forms no homonuclear pair, so its diagonal curve is never generated. That is
    // what keeps Li out of Li2's 14,400-determinant solve.
    let mut done: Vec<(u32, u32)> = Vec::new();
    for i in 0..species.len() {
        for j in (i + 1)..species.len() {
            let (a, b) = (species[i], species[j]);
            let key = if a.z <= b.z { (a.z, b.z) } else { (b.z, a.z) };
            if done.contains(&key) {
                continue;
            }
            done.push(key);
            let status = load_pair_table(&mut s, curve(a, b), Host::Native);
            assert_eq!(
                status, TABLE_OK,
                "{}{} did not load into the bank (status {status})",
                a.symbol, b.symbol
            );
        }
    }
    s
}

fn place(s: &mut Sim, i: usize, x: f64, y: f64) {
    let cy = 0.5 * s.height;
    s.set_position(i, x, cy + y);
    s.set_velocity(i, 0.0, 0.0);
}

/// Place atoms `i` and `j` a distance `r` apart along x, closing on each other at `v` per
/// atom.
///
/// # Why the pair is never placed AT REST
///
/// A pair at rest has `E_rel = U(R)` exactly, which puts it exactly ON its own outer
/// classical turning point — and the bond criterion is `E_rel < 0 AND R < R_outer`,
/// strictly. So a pair at rest inside its own well reads NOT BONDED, by rounding, at every
/// separation. `Sim::reset` says this in its own comments and derives an opening speed to
/// avoid it; a fixture that ignored it would be testing the boundary case rather than the
/// physics, and the first version of this file did exactly that and read `bonded = false`
/// on a hydrogen molecule sitting in its own well.
///
/// A RADIAL closing velocity is used rather than a transverse one because transverse
/// motion carries angular momentum, which raises the effective potential and moves the
/// turning point for a reason that has nothing to do with the curve under test.
fn place_pair(s: &mut Sim, i: usize, j: usize, x: f64, r: f64, v: f64) {
    let cy = 0.5 * s.height;
    s.set_position(i, x, cy);
    s.set_position(j, x + r, cy);
    s.set_velocity(i, v, 0.0);
    s.set_velocity(j, -v, 0.0);
}

// ================================================================== B1: the regression

/// B1's first half: an all-hydrogen scene through the bank is BIT-FOR-BIT the scene the
/// single table produced.
///
/// The reference was produced by `examples/b1_reference.rs` run in a git worktree at the
/// commit before the bank landed — 693 lines of raw `f64` bit patterns covering three
/// scenes, every frame of each, and the full final state. This re-derives it through the
/// bank and requires equality.
///
/// A failure here is not "the bank is slightly different". Every quantity in the file is
/// a bit pattern, so any difference at all means the banked physics is not the banked
/// physics — which is the one thing B1 does not allow the bank to cost.
#[test]
fn b1_all_hydrogen_is_bit_identical_to_the_single_table() {
    let expected = include_str!("data/b1_hydrogen_reference.txt");
    let got = b1_dump::dump_all();
    if got != expected {
        // Report the FIRST divergence rather than 693 lines of diff: the first one is the
        // one with a cause, and everything after it is that cause propagating.
        for (n, (a, b)) in got.lines().zip(expected.lines()).enumerate() {
            if a != b {
                panic!(
                    "B1 FAILED at line {}:\n  bank     {a}\n  reference {b}\n\
                     The all-hydrogen scene is no longer bit-for-bit what the single table \
                     produced. Re-run examples/b1_reference.rs and compare against \
                     tests/data/b1_hydrogen_reference.txt before changing anything.",
                    n + 1
                );
            }
        }
        panic!(
            "B1 FAILED on length: {} lines from the bank against {} in the reference",
            got.lines().count(),
            expected.lines().count()
        );
    }
}

// ================================================================== B1: the mixed fixture

/// B1's second half: every pair's bonded/`e_rel` reading uses THAT PAIR's own curve,
/// asserted on a fixture where the two criteria provably differ.
///
/// # The fixture, and why it is hydrogen against helium
///
/// The carrier is stated and checked before anything is concluded from it: in this model
/// H2 BINDS and H-He does not bind AT ANY SEPARATION — helium's closed shell refuses, and
/// `locate_well` reports `None` for it rather than anything having to know that helium is
/// noble. Measured on the generated curves: at 1.6 bohr `u(H-H) = -1.957e-1` and
/// `u(H-He) = +1.994e-1`, opposite in sign and comparable in size.
///
/// So the two pairs disagree about the SIGN of the interaction and about the bonded
/// verdict, at the same separation, in the same scene. One table cannot produce both
/// readings whatever its knots are, which is the sharpest form this fixture can take.
///
/// The first version of this test used lithium and asserted H-Li was repulsive at 2.0
/// bohr. It is not: H-Li crosses zero at about 1.87 and its minimum is at 2.924, so 2.0 is
/// already inside the well. The premise was wrong, the assertion caught it, and the fixture
/// was moved onto measured ground rather than onto a different guess.
#[test]
fn b1_each_pair_reads_its_own_curve() {
    const R: f64 = 1.6;

    // THE CARRIER, measured from the curves themselves rather than assumed.
    let hh_at_r = interp_u(curve(HYDROGEN, HYDROGEN), R);
    let hhe_at_r = interp_u(curve(HYDROGEN, HELIUM), R);
    println!(
        "B1 fixture carrier at R = {R} bohr: u(H-H) = {hh_at_r:+.6e}  u(H-He) = {hhe_at_r:+.6e}"
    );
    assert!(
        hh_at_r < 0.0,
        "the fixture's premise is gone: H-H is not attractive at {R} bohr ({hh_at_r:+.6e})"
    );
    assert!(
        hhe_at_r > 0.0,
        "the fixture's premise is gone: H-He is not repulsive at {R} bohr ({hhe_at_r:+.6e})"
    );
    assert!(
        curve(HYDROGEN, HELIUM).meta.well.is_none(),
        "the fixture's premise is gone: H-He has acquired a well, so the two criteria no \
         longer differ in kind"
    );

    // Four atoms in two well-separated pairs, each at R, each closing radially so that
    // neither sits exactly on its own turning point. See `place_pair`.
    let mut s = scene(&[HYDROGEN, HYDROGEN, HYDROGEN, HELIUM]);
    place_pair(&mut s, 0, 1, 4.0, R, CLOSING_SPEED);
    place_pair(&mut s, 2, 3, 30.0, R, CLOSING_SPEED);
    s.rebase();

    let hh = find_pair(&s, 0, 1);
    let hhe = find_pair(&s, 2, 3);
    println!(
        "  H-H  pair: r = {:.4}  e_rel = {:+.6e}  bonded = {}\n  \
           H-He pair: r = {:.4}  e_rel = {:+.6e}  bonded = {}",
        hh.r, hh.e_rel, hh.bonded, hhe.r, hhe.e_rel, hhe.bonded
    );

    assert!(
        hh.bonded,
        "the H-H pair inside its own well does not read BONDED: e_rel = {:+.6e}, \
         r = {:.4}, r_outer = {:.4}",
        hh.e_rel, hh.r, hh.r_outer
    );
    assert!(
        !hhe.bonded,
        "the H-He pair reads BONDED, and in this model helium binds to nothing — it is \
         being served hydrogen's curve. e_rel = {:+.6e}, r = {:.4}, r_outer = {:.4}",
        hhe.e_rel, hhe.r, hhe.r_outer
    );
    // And each POTENTIAL term is its own curve's, to the interpolator's own accuracy.
    // `e_rel` also carries the pair's relative kinetic energy, which differs between the
    // two pairs because their reduced masses differ — so the potential is recovered by
    // subtracting the kinetic term rather than compared to `e_rel` directly.
    let ke = |mi: f64, mj: f64| {
        let mu = (mi * mj) / (mi + mj);
        0.5 * mu * (2.0 * CLOSING_SPEED) * (2.0 * CLOSING_SPEED)
    };
    let m_h = holon_render::sim::M_H;
    let u_hh_read = hh.e_rel - ke(m_h, m_h);
    let u_hhe_read = hhe.e_rel - ke(m_h, HELIUM.mass_me());
    assert!(
        (u_hh_read - hh_at_r).abs() < 1e-12,
        "the H-H pair's potential {u_hh_read:.9e} is not its own curve's u {hh_at_r:.9e}"
    );
    assert!(
        (u_hhe_read - hhe_at_r).abs() < 1e-12,
        "the H-He pair's potential {u_hhe_read:.9e} is not its own curve's u {hhe_at_r:.9e}"
    );
    // The two potentials differ in SIGN, which is the statement a single table cannot make.
    assert!(
        u_hh_read * u_hhe_read < 0.0,
        "the two pairs agree in sign, so this fixture cannot tell a bank from a table"
    );
}

/// The bank's own bookkeeping: a mixed scene has one slot per pair type, they are
/// distinct, and each holds the curve it should.
#[test]
fn b1_the_bank_holds_one_curve_per_pair_type() {
    let s = scene(&[HYDROGEN, HYDROGEN, HYDROGEN, LITHIUM]);
    let hh = s.bank.slot_of_z(1, 1).expect("H-H is not registered");
    let hli = s.bank.slot_of_z(1, 3).expect("H-Li is not registered");
    assert_ne!(hh, hli, "H-H and H-Li resolved to the SAME slot");
    assert_eq!(
        s.bank.slot_of_z(3, 1),
        Some(hli),
        "the pair key is not unordered: (Li,H) is a different slot from (H,Li)"
    );
    assert!(s.bank.is_filled(hh) && s.bank.is_filled(hli));
    // The Li-Li slot exists (Li is registered) and is EMPTY, because no pair of atoms in
    // this scene forms it. That is the distinction between registered and active, and it
    // is what keeps a 14,400-determinant solve out of a scene with one lithium in it.
    let lili = s.bank.slot_of_z(3, 3).expect("Li is not registered");
    assert!(
        !s.bank.is_filled(lili),
        "the Li-Li curve was generated for a scene containing one lithium atom"
    );
    assert!(
        s.pairs_ready(),
        "the scene reports it is missing a curve, but every ACTIVE pair has one"
    );
    let r_e_hh = s.bank.table_slot(hh).r_e;
    let r_e_hli = s.bank.table_slot(hli).r_e;
    println!("  slot {hh} (H-H) R_e = {r_e_hh:.6}   slot {hli} (H-Li) R_e = {r_e_hli:.6}");
    assert!(
        (r_e_hh - 1.3886).abs() < 1e-3 && (r_e_hli - 2.9244).abs() < 1e-3,
        "the slots do not hold the curves they are supposed to: R_e = {r_e_hh} and {r_e_hli}"
    );
}

/// The packing from a species pair to a slot is a bijection on the upper triangle. A
/// collision here would serve two pair types one curve, silently.
#[test]
fn the_slot_packing_is_a_bijection() {
    let mut seen = vec![false; MAX_TABLES];
    for i in 0..MAX_SPECIES {
        for j in i..MAX_SPECIES {
            let s = holon_render::bank::slot_index(i, j);
            assert!(s < MAX_TABLES, "slot ({i},{j}) = {s} is past MAX_TABLES");
            assert!(!seen[s], "slot ({i},{j}) collides with an earlier pair at {s}");
            seen[s] = true;
            assert_eq!(
                s,
                holon_render::bank::slot_index(j, i),
                "({i},{j}) and ({j},{i}) are different slots; the key is not unordered"
            );
        }
    }
    assert!(seen.iter().all(|&x| x), "the packing does not cover every slot");
}

/// Past the declared cap the bank REFUSES rather than reusing a slot.
///
/// The failing case matters more than the passing one: a bank that wrapped around would
/// serve the fourth species some other pair's curve, which is plant (i)'s defect arriving
/// through the front door.
#[test]
fn the_bank_refuses_a_species_past_its_cap() {
    let mut s = Box::new(Sim::empty());
    s.reset(MAX_SPECIES + 1);
    // Hydrogen is seeded as species 0, so MAX_SPECIES - 1 more fit.
    let extras: Vec<Species> = holon_chem::elements::ALL_ELEMENTS
        .iter()
        .copied()
        .filter(|sp| sp.z != 1)
        .take(MAX_SPECIES)
        .collect();
    for (k, sp) in extras.iter().enumerate() {
        let ok = s.set_species(k + 1, *sp);
        if k + 1 < MAX_SPECIES {
            assert!(ok, "the bank refused {} at index {}", sp.symbol, k + 1);
        } else {
            assert!(
                !ok,
                "the bank ACCEPTED {} as species {} with a cap of {MAX_SPECIES}",
                sp.symbol,
                k + 2
            );
            // And the refusal left the atom alone rather than half-applying.
            assert_ne!(
                s.atoms[k + 1].species.z,
                sp.z,
                "the species was applied even though the bank refused it"
            );
        }
    }
    assert_eq!(s.bank.species_count(), MAX_SPECIES);
}

// ================================================================== plant (i)

/// PLANT (i) — THE SWAPPED TABLE. Serving pair (A,B) the (A,A) curve must move a staked
/// mixed dimer's `R_e` beyond referee tolerance by orders.
///
/// Two carriers, both asserted nonzero before the plant is scored, because the defect has
/// two visible faces and a plant that moved only one would leave the other untested:
///
/// * the EQUILIBRIUM, which is what the freeze names: H2 sits at 1.389 bohr and H-Li at
///   2.924, so serving the wrong curve moves a lithium hydride's bond length by 1.5 bohr;
/// * the ENERGY at the correct equilibrium, which is what the referee grades: the two
///   curves differ there by about 4e-2 Ha, some eight orders above the referee's 1e-10.
#[test]
fn plant_i_the_swapped_table_is_caught() {
    const REFEREE_TOLERANCE: f64 = 1e-10;

    let hli = curve(HYDROGEN, LITHIUM);
    let hh = curve(HYDROGEN, HYDROGEN);
    let r_e_correct = hli.meta.well.expect("H-Li has no well").r_e;
    let r_e_swapped = hh.meta.well.expect("H2 has no well").r_e;

    // CARRIER 1: the equilibrium separation moves.
    let d_r_e = (r_e_swapped - r_e_correct).abs();
    // CARRIER 2: the energy at the correct equilibrium moves.
    let u_correct = interp_u(hli, r_e_correct);
    let u_swapped = interp_u(hh, r_e_correct);
    let d_u = (u_swapped - u_correct).abs();
    println!(
        "plant (i) carriers: R_e {r_e_correct:.6} -> {r_e_swapped:.6} bohr (shift \
         {d_r_e:.6}), u(R_e) {u_correct:+.6e} -> {u_swapped:+.6e} Ha (shift {d_u:.3e} = \
         10^{:.1} x the referee tolerance)",
        (d_u / REFEREE_TOLERANCE).log10()
    );
    assert!(
        d_r_e > 1.0,
        "PLANT (i) VOID: the two curves' minima are {d_r_e:.3e} bohr apart, so the sector \
         the plant acts on is empty"
    );
    assert!(
        d_u > REFEREE_TOLERANCE * 1e6,
        "PLANT (i) VOID: the two curves agree at R_e to {d_u:.3e} Ha"
    );

    // The unplanted reading, with the pair sitting at its own equilibrium.
    let mut s = scene(&[HYDROGEN, LITHIUM]);
    place_pair(&mut s, 0, 1, 10.0, r_e_correct, CLOSING_SPEED);
    s.rebase();
    let before = find_pair(&s, 0, 1);
    let slot = s.bank.slot_of_z(1, 3).unwrap();
    let r_e_before = s.bank.table_slot(slot).r_e;

    // THE PLANT: overwrite the H-Li slot with the H-H curve. The defect in its purest
    // form — the dispatch is correct, the slot is correct, and the CURVE IN IT is another
    // pair's.
    swap_curve_into(&mut s, slot, hh);
    s.refresh_pairs();
    let after = find_pair(&s, 0, 1);
    let r_e_after = s.bank.table_slot(slot).r_e;

    println!(
        "  unplanted: R_e = {r_e_before:.6}  e_rel = {:+.6e}  bonded = {}\n  \
           planted:   R_e = {r_e_after:.6}  e_rel = {:+.6e}  bonded = {}",
        before.e_rel, before.bonded, after.e_rel, after.bonded
    );
    let moved_r_e = (r_e_after - r_e_before).abs();
    let moved_u = (after.e_rel - before.e_rel).abs();
    assert!(
        moved_r_e > 1.0,
        "PLANT (i) MISSED: the served curve's equilibrium moved by only {moved_r_e:.3e} bohr"
    );
    assert!(
        moved_u > REFEREE_TOLERANCE * 1e6,
        "PLANT (i) MISSED: swapping the curve moved the pair's energy by only \
         {moved_u:.3e} Ha. A plant that does not move the observable is not a test."
    );
    println!(
        "  PLANT (i) CAUGHT on the H-Li dimer: R_e moved {moved_r_e:.4} bohr and the pair \
         energy moved {moved_u:.3e} Ha, {:.1} orders above the referee's \
         {REFEREE_TOLERANCE:.0e}.",
        (moved_u / REFEREE_TOLERANCE).log10()
    );
}

/// PLANT (i), second instance: the same swap on a pair that does not bind at all, where
/// the consequence is not a shifted number but a WRONG VERDICT.
///
/// H-He binds at no separation in this model. Serving it hydrogen's curve makes a helium
/// atom form a bond, which is the failure a user would actually see, and it is a different
/// observable from the `R_e` shift above — hence a second instance rather than a second
/// assertion in the first.
#[test]
fn plant_i_the_swapped_table_invents_a_bond() {
    const R: f64 = 1.6;
    let hhe = curve(HYDROGEN, HELIUM);
    let hh = curve(HYDROGEN, HYDROGEN);
    // CARRIER: the pair genuinely does not bind, so a BONDED reading afterwards is created
    // by the plant and not by the fixture.
    assert!(
        hhe.meta.well.is_none(),
        "PLANT (i) VOID: H-He has a well, so an invented bond is not distinguishable from \
         a real one"
    );

    let mut s = scene(&[HYDROGEN, HELIUM]);
    place_pair(&mut s, 0, 1, 10.0, R, CLOSING_SPEED);
    s.rebase();
    let before = find_pair(&s, 0, 1);
    assert!(
        !before.bonded,
        "PLANT (i) VOID: the unplanted H-He pair already reads BONDED"
    );

    let slot = s.bank.slot_of_z(1, 2).unwrap();
    swap_curve_into(&mut s, slot, hh);
    s.refresh_pairs();
    let after = find_pair(&s, 0, 1);
    println!(
        "plant (i) verdict: H-He at {R} bohr reads bonded = {} on its own curve and \
         bonded = {} on hydrogen's",
        before.bonded, after.bonded
    );
    assert!(
        after.bonded,
        "PLANT (i) MISSED: serving the H-H curve to a helium pair did not invent a bond"
    );
    println!("  PLANT (i) CAUGHT: the swap invented a bond helium cannot have.");
}

// ================================================================== plant (ii)

/// PLANT (ii) — THE MASS PLANT. Running one species at another's mass must shift the mixed
/// scene's DERIVED timescale by the mass ratio's square root.
///
/// The expected shift is COMPUTED from the two reduced masses, never written down: `dt` is
/// `2*pi/(64*omega)` with `omega = sqrt(k_e/mu)`, so `dt` scales as `sqrt(mu)` and the
/// prediction is `sqrt(mu_planted/mu_correct)` with both masses read off the species. A
/// literal here would be a test of arithmetic done once by hand.
///
/// The scene is one hydrogen and one lithium, so the only active pair is H-Li and its
/// reduced mass is the one the clock is derived from. In a scene where H-H were also
/// active, H-H would set the timestep (it is by far the fastest mode) and lithium's mass
/// would not appear in the answer at all — which is correct physics and a plant on an
/// empty sector, so the fixture is built to avoid it.
#[test]
fn plant_ii_the_wrong_mass_moves_the_clock() {
    let mut s = scene(&[HYDROGEN, LITHIUM]);
    place(&mut s, 0, 10.0, 0.0);
    place(&mut s, 1, 10.0 + 2.9244, 0.0);
    s.adopt_table_timescale();
    let dt_correct = s.dt();
    let mu_correct = s.timescale.mu;

    // THE PLANT: lithium, at hydrogen's mass. Same Z, so the same curve out of the same
    // slot — the electronic structure is untouched and only the inertia is wrong, which is
    // exactly the defect a mass table typo would produce.
    let mut light_lithium = LITHIUM;
    light_lithium.mass_u = HYDROGEN.mass_u;
    assert!(s.set_species(1, light_lithium));
    s.adopt_table_timescale();
    let dt_planted = s.dt();
    let mu_planted = s.timescale.mu;

    let predicted = (mu_planted / mu_correct).sqrt();
    let measured = dt_planted / dt_correct;
    println!(
        "plant (ii): mu {mu_correct:.6} -> {mu_planted:.6} m_e,  \
         dt {dt_correct:.9} -> {dt_planted:.9} a.u.\n  \
         predicted dt ratio sqrt(mu'/mu) = {predicted:.12}   measured = {measured:.12}"
    );

    // CARRIER: the shift has to be large enough to be a reading rather than roundoff.
    assert!(
        (predicted - 1.0).abs() > 0.1,
        "PLANT (ii) VOID: the two reduced masses differ by only {:.3}%, so there is no \
         shift to detect",
        100.0 * (predicted - 1.0).abs()
    );
    assert!(
        (measured - predicted).abs() < 1e-12,
        "PLANT (ii) MISSED: the derived timestep moved by {measured:.12} where the mass \
         ratio's square root is {predicted:.12}. The clock is not being derived from the \
         active pair's own reduced mass."
    );
    println!(
        "  PLANT (ii) CAUGHT: the derived timestep moved by exactly the mass ratio's square \
         root, to {:.1e} relative.",
        (measured - predicted).abs()
    );
}

// ================================================================== plant (iii)

/// PLANT (iii) — THE DMRG LABEL. A DMRG curve presented as exact must be REFUSED, and the
/// refusal is demonstrated firing.
#[test]
fn plant_iii_a_dmrg_curve_presented_as_exact_is_refused() {
    let planted = TableProvenance {
        route: Route::Dmrg,
        source: Source::Shipped,
        n_det: 132_496, // SiO, the freeze's own overlap species
        n_basis: 14,
        uncertainty_ha: 1e-9,
        claimed_exact: true, // THE PLANT
        // This plant is about a different field; no well is declared, so the
        // uncertainty-versus-well leg has nothing to weigh.
        well_depth_ha: 0.0,
    };
    // Even with D1 fully discharged, the false claim is refused: an admitted bridge makes
    // a DMRG curve usable, it does not make it exact.
    let discharged = D1Admission {
        validated: true,
        worst_overlap_ha: 1e-12,
        stake_ha: 1e-8,
        overlap_species: 2,
    };
    assert_eq!(
        planted.admit(&discharged, Host::Browser),
        Err(Refusal::DmrgClaimedExact),
        "PLANT (iii) MISSED: a DMRG curve presenting itself as exact in the model was \
         admitted"
    );
    println!("plant (iii) CAUGHT: {}", Refusal::DmrgClaimedExact.plain());

    // The same curve WITHOUT the false claim is admitted once D1 is discharged, so the
    // refusal above is about the claim and not about DMRG in general. Without this the
    // test would pass just as well against a gate that refuses everything.
    let honest = TableProvenance {
        claimed_exact: false,
        ..planted
    };
    assert_eq!(
        honest.admit(&discharged, Host::Browser),
        Ok(()),
        "the gate refuses an honestly-labelled DMRG curve even with D1 discharged, so the \
         plant above proved nothing about the label"
    );
}

/// PLANT (iii), THROUGH THE PRODUCTION PATH — a real `PairTable` carrying a DMRG route,
/// pushed through `load_pair_table`, refused, and EVICTED.
///
/// # Why the unit test above is not enough
///
/// `plant_iii_a_dmrg_curve_presented_as_exact_is_refused` calls `TableProvenance::admit`
/// directly. That proves the RULE is right and proves nothing about whether anything calls
/// it. The path a DMRG curve would actually take into the sandbox is
/// `generate_pair_table` -> `PairMeta { route, .. }` -> `load_pair_table` -> `commit`, and
/// the defect this campaign found was precisely a rule that was correct and a path that
/// did not consult it. Standing question 1: the gate has to be connected where the code
/// runs.
///
/// # The plant is a RELABEL, and that is not a shortcut
///
/// No pair on this engine can produce a genuinely DMRG-routed table: `solve` switches
/// route above 50,000 determinants, and every pair with a determinant count that large has
/// an orbital count the MPO builder cannot be driven at (see `MPS_MAX_ORBITALS` — 528 s at
/// six orbitals, and it does not finish at ten). So the honest way to exercise the path is
/// to take a real curve off the real generator and change the one field the gate reads —
/// which is a plant in the strict sense: the defect is introduced, and the question is
/// whether the machinery notices.
///
/// # WHICH refusal fires here, and why it is not the one I first expected
///
/// I predicted `DmrgClaimedExact` and got `DmrgUnvalidated`. The prediction was wrong for
/// a good reason: `load_pair_table` DERIVES `claimed_exact` from the route
/// (`claimed_exact: pt.meta.route.is_exact_in_model()`), so through this door the claim and
/// the route CANNOT DISAGREE. "A DMRG curve presented as exact" is not a state a
/// `PairTable` can reach here — it is structurally unrepresentable, which is a stronger
/// guarantee than a check.
///
/// The defect IS reachable through the other door, where a shipped file declares
/// `exact_in_model` independently of `solver_route`, and there it is refused as
/// `DmrgClaimedExact`. That case is exercised against the real browser artifact and the
/// real shipped JSON in `viewer/smoke.mjs`, which relabels `docs/atoms/tables/Cl2.json` as
/// DMRG and watches the refusal fire and the slot evict.
///
/// So plant (iii) has two demonstrations because the sandbox has two doors, and they fail
/// differently by construction. Both are asserted rather than one being assumed to cover
/// the other.
#[test]
fn plant_iii_a_dmrg_labelled_curve_is_refused_by_the_production_loader() {
    let mut s = scene(&[HYDROGEN, LITHIUM]);
    let slot = s.bank.slot_of_z(1, 3).unwrap();
    assert!(
        s.bank.is_filled(slot),
        "the fixture did not load H-Li through the production loader"
    );
    // CARRIER: the unplanted curve went in on the determinant route, so the relabel below
    // changes something. A curve that was already DMRG would make this plant a no-op.
    assert_eq!(
        curve(HYDROGEN, LITHIUM).meta.route,
        holon_chem::fci::SolverRoute::Determinant,
        "PLANT (iii) VOID: the H-Li curve is already DMRG-routed, so relabelling it plants \
         nothing"
    );

    // THE PLANT: the same curve, relabelled DMRG, still claiming exactness — which is what
    // `PairMeta` does for a determinant route, and what a mislabelled DMRG curve would
    // therefore carry.
    let mut planted = curve(HYDROGEN, LITHIUM).clone();
    planted.meta.route = holon_chem::fci::SolverRoute::Dmrg;
    let status = load_pair_table(&mut s, &planted, Host::Native);
    println!(
        "plant (iii) through load_pair_table: status {status} (expected {}, \
         PROVENANCE_REFUSED + DmrgUnvalidated — see this test's header for why it is not \
         DmrgClaimedExact)",
        holon_render::refusal_code(Refusal::DmrgUnvalidated)
    );
    assert_eq!(
        status,
        holon_render::refusal_code(Refusal::DmrgUnvalidated),
        "PLANT (iii) MISSED: a DMRG-labelled curve went through the production loader and \
         was not refused. The rule is right and nothing consulted it."
    );
    assert!(
        !s.bank.is_filled(slot),
        "PLANT (iii) MISSED: the curve was refused and is STILL IN THE BANK, so the force \
         loop would evaluate it"
    );
    assert!(
        !s.pairs_ready(),
        "the scene reports ready with the refused pair's slot empty"
    );

    // THE POSITIVE CONTROL. Without it this test passes just as well against a loader that
    // refuses everything: the unplanted curve must still go in.
    let clean = curve(HYDROGEN, LITHIUM);
    assert_eq!(
        load_pair_table(&mut s, clean, Host::Native),
        TABLE_OK,
        "the loader refused the unplanted determinant curve, so the refusal above says \
         nothing about the label"
    );
    assert!(s.bank.is_filled(slot) && s.pairs_ready());
    println!("  PLANT (iii) CAUGHT through the production loader, and the clean curve still loads.");
}

/// The provenance strings that reach on-disk artifacts carry no accidental whitespace.
///
/// A referee-pinnable file is a thing other people diff. Runs of spaces from a wrapped
/// string literal are invisible in the source, land in the JSON, and are then annoying to
/// change because changing them moves the digest. This caught exactly that: both
/// `PAIR_PROVENANCE_DMRG` and the emitter's `grid_rule` had picked up six- and
/// fourteen-space runs from `\`-continued literals, and `grid_rule`'s had already shipped.
#[test]
fn provenance_strings_carry_no_accidental_whitespace() {
    for (name, text) in [
        ("PAIR_PROVENANCE", holon_chem::pair::PAIR_PROVENANCE),
        ("PAIR_PROVENANCE_DMRG", holon_chem::pair::PAIR_PROVENANCE_DMRG),
    ] {
        assert!(
            !text.contains("  "),
            "{name} contains a run of spaces and will land in on-disk JSON: {text:?}"
        );
        assert!(
            !text.contains('\n') && !text.contains('\t'),
            "{name} contains a newline or tab: {text:?}"
        );
    }
}

/// Every other provenance refusal, each fired by a curve that breaks exactly that rule.
///
/// The positive control is first and is not decoration: a gate that refuses everything
/// passes every negative test in this function.
#[test]
fn every_provenance_refusal_has_a_demonstrated_failing_case() {
    let d1_none = D1Admission::NONE;
    let d1_ok = D1Admission {
        validated: true,
        worst_overlap_ha: 1e-12,
        stake_ha: 1e-8,
        overlap_species: 2,
    };

    // POSITIVE CONTROL: a light, solved, determinant curve is admitted.
    let good = TableProvenance::solved_exact(4, 2, 0.0);
    assert_eq!(good.admit(&d1_none, Host::Browser), Ok(()));

    // Undeclared route.
    let mut p = good;
    p.route = Route::Undeclared;
    assert_eq!(
        p.admit(&d1_none, Host::Browser),
        Err(Refusal::RouteUndeclared)
    );

    // DMRG with no D1 record: the freeze's "only then", enforced.
    let dmrg = TableProvenance {
        route: Route::Dmrg,
        source: Source::Shipped,
        n_det: 132_496,
        n_basis: 14,
        uncertainty_ha: 1e-9,
        claimed_exact: false,
        // This plant is about a different field; no well is declared, so the
        // uncertainty-versus-well leg has nothing to weigh.
        well_depth_ha: 0.0,
    };
    assert_eq!(
        dmrg.admit(&d1_none, Host::Browser),
        Err(Refusal::DmrgUnvalidated)
    );
    assert_eq!(dmrg.admit(&d1_ok, Host::Browser), Ok(()));

    // DMRG with no uncertainty.
    let dmrg_bare = TableProvenance {
        uncertainty_ha: 0.0,
        ..dmrg
    };
    assert_eq!(
        dmrg_bare.admit(&d1_ok, Host::Browser),
        Err(Refusal::DmrgUncertaintyMissing)
    );

    // A shipped table with no uncertainty: an absent bound must not read as a zero one.
    let shipped_bare = TableProvenance {
        route: Route::Determinant,
        source: Source::Shipped,
        n_det: 132_496,
        n_basis: 14,
        uncertainty_ha: 0.0,
        claimed_exact: true,
        // This plant is about a different field; no well is declared, so the
        // uncertainty-versus-well leg has nothing to weigh.
        well_depth_ha: 0.0,
    };
    assert_eq!(
        shipped_bare.admit(&d1_none, Host::Browser),
        Err(Refusal::UncertaintyMissing)
    );

    // The browser split, both directions, and only in the browser.
    let heavy_solved = TableProvenance::solved_exact(132_496, 14, 1e-11);
    assert_eq!(
        heavy_solved.admit(&d1_none, Host::Browser),
        Err(Refusal::SplitViolated)
    );
    assert_eq!(
        heavy_solved.admit(&d1_none, Host::Native),
        Ok(()),
        "the split fired on a native host, where there is no page load budget to protect"
    );
    let light_shipped = TableProvenance {
        route: Route::Determinant,
        source: Source::Shipped,
        n_det: 4,
        n_basis: 2,
        uncertainty_ha: 1e-14,
        claimed_exact: true,
        // This plant is about a different field; no well is declared, so the
        // uncertainty-versus-well leg has nothing to weigh.
        well_depth_ha: 0.0,
    };
    assert_eq!(
        light_shipped.admit(&d1_none, Host::Browser),
        Err(Refusal::SplitViolated)
    );

    // THE MEASURED HALF OF THE SPLIT. Cl2 has 324 determinants -- comfortably under
    // IN_BROWSER_DET_LIMIT -- and 18 basis functions, and costs 96 s to solve. A split on
    // the determinant count alone, which is the criterion the freeze names, would have
    // sent it to the browser and hung the page for a minute and a half.
    let cl2_solved = TableProvenance::solved_exact(324, 18, 1e-11);
    assert!(
        cl2_solved.n_det < holon_render::bank::IN_BROWSER_DET_LIMIT,
        "the Cl2 case no longer exercises the basis half of the split: its determinant \
         count now trips the determinant half on its own"
    );
    assert_eq!(
        cl2_solved.admit(&d1_none, Host::Browser),
        Err(Refusal::SplitViolated),
        "a curve under the determinant limit but over the measured basis limit was \
         admitted for in-browser solving"
    );

    // A D1 record whose flag and whose measurement disagree admits nothing.
    let liar = D1Admission {
        validated: true,
        worst_overlap_ha: 1e-3,
        stake_ha: 1e-8,
        overlap_species: 2,
    };
    assert!(!liar.admits(), "a D1 record missing its own stake still admits");
    assert_eq!(
        dmrg.admit(&liar, Host::Browser),
        Err(Refusal::DmrgUnvalidated)
    );
    let too_few = D1Admission {
        overlap_species: 1,
        ..d1_ok
    };
    assert!(
        !too_few.admits(),
        "a D1 record with one overlap species admits, where the freeze stakes two"
    );

    // ---- EXHAUSTIVE BY CONSTRUCTION -------------------------------------------------
    //
    // Everything above is a hand-written list, and a hand-written list cannot enforce the
    // word EVERY in this test's name: plant (iv) added two variants and this test went on
    // passing without touching either. So the sweep below routes through a match that the
    // compiler requires to be total — a new `Refusal` stops the crate building until
    // someone says how it is demonstrated, which is the property the name claims.
    enum Demo {
        /// The provenance, and the record and host to grade it against.
        Admit(TableProvenance, D1Admission, Host),
        /// Not reachable from `admit` at all: `CurveNotLoaded` is `commit`'s own refusal,
        /// raised after admission when the slot holds no interpolant. Exercised below.
        CommitOnly,
    }
    let demo_for = |r: Refusal| -> Demo {
        match r {
            Refusal::RouteUndeclared => Demo::Admit(p, d1_none, Host::Browser),
            Refusal::DmrgClaimedExact => Demo::Admit(
                TableProvenance { claimed_exact: true, ..dmrg },
                d1_ok,
                Host::Browser,
            ),
            Refusal::DmrgUnvalidated => Demo::Admit(dmrg, d1_none, Host::Browser),
            Refusal::UncertaintyMissing => Demo::Admit(shipped_bare, d1_none, Host::Browser),
            Refusal::DmrgUncertaintyMissing => Demo::Admit(dmrg_bare, d1_ok, Host::Browser),
            Refusal::SplitViolated => Demo::Admit(light_shipped, d1_none, Host::Browser),
            Refusal::CurveNotLoaded => Demo::CommitOnly,
            Refusal::UncertaintyExceedsResolution => Demo::Admit(
                TableProvenance { uncertainty_ha: 1e-3, ..light_shipped },
                d1_none,
                Host::Native,
            ),
            Refusal::UncertaintyExceedsWell => Demo::Admit(
                // The narrow window named on the variant: a file declaring a well
                // SHALLOWER than the schema's own threshold, so the resolution leg stays
                // silent and this one is the only thing that can speak.
                TableProvenance {
                    uncertainty_ha: 1e-5,
                    well_depth_ha: 5e-6,
                    ..light_shipped
                },
                d1_none,
                Host::Native,
            ),
        }
    };
    let all = [
        Refusal::RouteUndeclared,
        Refusal::DmrgClaimedExact,
        Refusal::DmrgUnvalidated,
        Refusal::UncertaintyMissing,
        Refusal::DmrgUncertaintyMissing,
        Refusal::SplitViolated,
        Refusal::CurveNotLoaded,
        Refusal::UncertaintyExceedsResolution,
        Refusal::UncertaintyExceedsWell,
    ];
    for r in all {
        match demo_for(r) {
            Demo::Admit(prov, d1, host) => assert_eq!(
                prov.admit(&d1, host),
                Err(r),
                "{r:?} has no demonstrated failing case: its fixture produced something                  else, so the refusal is unwatched and may be unreachable"
            ),
            Demo::CommitOnly => {
                let mut sc = scene(&[HYDROGEN]);
                let slot = sc.bank.slot_of_z(1, 1).unwrap();
                sc.bank.evict(slot);
                assert_eq!(
                    sc.bank.commit(slot, good, &d1_none, Host::Browser),
                    Err(Refusal::CurveNotLoaded),
                    "an empty slot accepted a provenance, so a curve's description can                      outlive the curve"
                );
            }
        }
    }
    // ...and the list itself is complete. `demo_for` is total by the compiler; this says
    // the ARRAY names every variant, which the compiler cannot check.
    let mut seen = all.to_vec();
    seen.dedup();
    assert_eq!(seen.len(), all.len(), "a refusal is listed twice in the sweep");
    for r in all {
        assert!(
            all.iter().filter(|x| **x == r).count() == 1,
            "{r:?} is not listed exactly once"
        );
    }
    assert_eq!(
        holon_render::refusal_code(all[all.len() - 1]) - holon_render::PROVENANCE_REFUSED,
        (all.len() - 1) as u32,
        "the sweep's last variant does not carry the last refusal code, so the array has          fallen out of step with `refusal_code` and a variant is missing from one of them"
    );
    println!("  all {} provenance refusals demonstrated firing.", all.len());
}

/// The refusal EVICTS. A gate that reports a problem and leaves the curve in the slot is a
/// gate the force loop walks straight past.
#[test]
fn a_refused_curve_does_not_stay_in_the_bank() {
    let mut s = scene(&[HYDROGEN, LITHIUM]);
    let slot = s.bank.slot_of_z(1, 3).unwrap();
    assert!(s.bank.is_filled(slot), "the fixture did not load H-Li");

    let bad = TableProvenance {
        route: Route::Dmrg,
        source: Source::Shipped,
        n_det: 132_496,
        n_basis: 14,
        uncertainty_ha: 1e-9,
        claimed_exact: true,
        // This plant is about a different field; no well is declared, so the
        // uncertainty-versus-well leg has nothing to weigh.
        well_depth_ha: 0.0,
    };
    let err = s
        .bank
        .commit(slot, bad, &D1_RECORD, Host::Browser)
        .expect_err("the gate admitted a DMRG curve presented as exact");
    assert_eq!(err, Refusal::DmrgClaimedExact);
    assert!(
        !s.bank.is_filled(slot),
        "the refused curve is still in the bank and the force loop would evaluate it"
    );
    assert!(
        !s.pairs_ready(),
        "the scene reports itself ready with a pair whose curve was just refused"
    );
}

/// The crate's committed D1 record, read as a fact rather than as an intention.
///
/// While gate D1's engine half is undischarged this record admits nothing, and that is the
/// freeze's "only then" being enforced rather than remembered. When D1 is discharged this
/// test's expectation changes with the record, deliberately: it is the one place where
/// "has the bridge been validated" is written down for the gate to read.
#[test]
fn the_committed_d1_record_says_what_it_measured() {
    println!(
        "D1 record: validated = {}  worst overlap = {:.3e} Ha  stake = {:.0e} Ha  species = {}",
        D1_RECORD.validated, D1_RECORD.worst_overlap_ha, D1_RECORD.stake_ha,
        D1_RECORD.overlap_species
    );
    if D1_RECORD.admits() {
        assert!(D1_RECORD.worst_overlap_ha <= D1_RECORD.stake_ha);
        assert!(D1_RECORD.overlap_species >= 2);
    } else {
        let dmrg = TableProvenance {
            route: Route::Dmrg,
            source: Source::Shipped,
            n_det: 132_496,
            n_basis: 14,
            uncertainty_ha: 1e-9,
            claimed_exact: false,
            // This plant is about a different field; no well is declared, so the
            // uncertainty-versus-well leg has nothing to weigh.
            well_depth_ha: 0.0,
        };
        assert_eq!(
            dmrg.admit(&D1_RECORD, Host::Browser),
            Err(Refusal::DmrgUnvalidated),
            "the D1 record admits nothing, yet a DMRG curve was let into the bank"
        );
    }
}

// ================================================================== C1

/// C1 — energy in a mixed box: the drift stays inside the derived bound, with the
/// curvature envelope taken over ALL active tables and per-species masses everywhere.
///
/// One gate per conservation law: this one reads the energy ledger and nothing else.
#[test]
fn c1_energy_gate_in_a_mixed_box() {
    let mut s = mixed_box();
    for _ in 0..200 {
        s.step_frame(64);
    }
    let bound = s.drift_bound();
    println!(
        "C1 energy (mixed H,H,Li; 200 x 64): |dE|_peak = {:.6e} Eh   bound = {:.6e} Eh   \
         ratio = {:.4}\n  dt = {:.6e}  k_env = {:.6e}  omega_env = {:.6e}  mu = {:.2}  mu_min = {:.2}",
        s.drift_peak,
        bound,
        s.drift_peak / bound,
        s.dt(),
        s.timescale.k_env,
        s.timescale.omega_env,
        s.timescale.mu,
        s.timescale.mu_min
    );
    assert!(
        s.energy_gate(),
        "C1 FAILED: peak drift {:.6e} Eh exceeds the derived bound {bound:.6e} Eh",
        s.drift_peak
    );
    assert!(
        s.drift_peak > 0.0,
        "C1 VOID: the peak drift is exactly zero, so the gate measured nothing"
    );
}

/// C1 — momentum in a mixed box. A separate gate, because one gate per conservation law:
/// an energy gate reading green while the impulse accounting is wrong is a thing that has
/// happened here.
#[test]
fn c1_momentum_gate_in_a_mixed_box() {
    let mut s = mixed_box();
    for _ in 0..200 {
        s.step_frame(64);
    }
    let bound = s.momentum_bound();
    println!(
        "C1 momentum (mixed H,H,Li; 200 x 64): |dP|_peak = {:.6e}   bound = {:.6e}   \
         ratio = {:.4}",
        s.momentum_residual_peak,
        bound,
        s.momentum_residual_peak / bound
    );
    assert!(
        s.momentum_gate(),
        "C1 FAILED: peak momentum residual {:.6e} exceeds the roundoff bound {bound:.6e}",
        s.momentum_residual_peak
    );
}

/// The envelope the drift bound is built from is the MAXIMUM over the active tables, not
/// whichever one the timescale happened to be derived from.
///
/// Demonstrated by construction: the H-Li curve's envelope is measured on its own, the
/// H-H curve's likewise, and the scene's `k_env` is required to be the larger. A bank that
/// bounded a mixed scene by one curve would read the smaller of the two whenever that
/// curve was the one the clock came from.
#[test]
fn the_envelope_covers_every_active_table() {
    let mut s = mixed_box();
    for _ in 0..20 {
        s.step_frame(64);
    }
    let e = s.timescale.e_rel_max;
    let hh = s.bank.slot_of_z(1, 1).unwrap();
    let hli = s.bank.slot_of_z(1, 3).unwrap();
    let k_hh = s.bank.table_slot(hh).curvature_envelope(e).0;
    let k_hli = s.bank.table_slot(hli).curvature_envelope(e).0;
    println!(
        "envelope at E_rel_max = {e:+.6e}:  k(H-H) = {k_hh:.6e}  k(H-Li) = {k_hli:.6e}  \
         k_env = {:.6e}",
        s.timescale.k_env
    );
    // CARRIER: the two curves must actually differ in stiffness, or the max is not a test.
    let spread = (k_hh - k_hli).abs() / k_hh.max(k_hli).max(1e-300);
    assert!(
        spread > 0.05,
        "VOID: the two active curves have the same reachable curvature to {:.2}%, so \
         taking a maximum over them proves nothing",
        100.0 * spread
    );
    assert!(
        (s.timescale.k_env - k_hh.max(k_hli)).abs() <= 1e-12 * k_hh.max(k_hli),
        "k_env = {:.9e} is not the maximum over the active tables ({k_hh:.9e}, {k_hli:.9e})",
        s.timescale.k_env
    );
}

/// The timestep comes from the FASTEST MODE, not the stiffest curve.
///
/// In the mixed fixture the H-Li curve and the H-H curve have different stiffnesses and
/// very different reduced masses, so `argmax k_e` and `argmax sqrt(k_e/mu)` are questions
/// with different answers. This asserts the clock answers the second one — which is the
/// one `dt` has to resolve — and prints both so a change of mind is visible rather than
/// silent.
#[test]
fn the_clock_is_the_fastest_mode_not_the_stiffest_curve() {
    let s = mixed_box();
    let hh = s.bank.slot_of_z(1, 1).unwrap();
    let hli = s.bank.slot_of_z(1, 3).unwrap();
    let m_h = holon_render::sim::M_H;
    let m_li = LITHIUM.mass_me();
    let mu_hh = (m_h * m_h) / (m_h + m_h);
    let mu_hli = (m_h * m_li) / (m_h + m_li);
    let k_hh = {
        let t = s.bank.table_slot(hh);
        t.curvature(t.r_e).abs()
    };
    let k_hli = {
        let t = s.bank.table_slot(hli);
        t.curvature(t.r_e).abs()
    };
    let w_hh = (k_hh / mu_hh).sqrt();
    let w_hli = (k_hli / mu_hli).sqrt();
    println!(
        "H-H : k_e = {k_hh:.6e}  mu = {mu_hh:.2}  omega = {w_hh:.6e}\n\
         H-Li: k_e = {k_hli:.6e}  mu = {mu_hli:.2}  omega = {w_hli:.6e}\n\
         scene omega_e = {:.6e}  mu = {:.2}",
        s.timescale.omega_e, s.timescale.mu
    );
    let fastest = w_hh.max(w_hli);
    assert!(
        (s.timescale.omega_e - fastest).abs() <= 1e-12 * fastest,
        "the clock was derived from omega = {:.9e}, not from the fastest active mode \
         {fastest:.9e}",
        s.timescale.omega_e
    );
    assert!(
        s.timescale.mu_min <= s.timescale.mu,
        "mu_min ({}) exceeds mu ({}); the envelope would be narrower than the clock's",
        s.timescale.mu_min,
        s.timescale.mu
    );
}

/// A scene missing one of its pair curves does not step.
///
/// The failing case for `pairs_ready`. The old question — "is THE table loaded" — answers
/// yes here, because two of the three curves are present.
#[test]
fn a_scene_missing_a_curve_refuses_to_step() {
    let mut s = mixed_box();
    assert!(s.pairs_ready());
    let hli = s.bank.slot_of_z(1, 3).unwrap();
    s.bank.evict(hli);
    assert!(
        !s.pairs_ready(),
        "the scene reports ready with the H-Li curve evicted"
    );
    assert!(
        s.table().is_loaded(),
        "the fixture no longer exercises the case: the PRIMARY curve is gone too, so the \
         old single-table check would also have refused and this proves nothing"
    );
    let before = (s.time, s.steps);
    s.step_frame(8);
    assert_eq!(
        (s.time, s.steps),
        before,
        "the scene advanced with a pair that has no curve"
    );
}

// ------------------------------------------------------------------ helpers

/// THE MIXED FIXTURE: two hydrogens and a lithium in a walled box, opened so that every
/// pair type is engaged and nothing starts inside a repulsive wall.
fn mixed_box() -> Box<Sim> {
    let mut s = scene(&[HYDROGEN, HYDROGEN, LITHIUM]);
    s.boundary = Boundary::Walls;
    s.dims = Dims::Two;
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    // A triangle whose sides sit near each pair's own minimum, with a small circulation so
    // the run sweeps the curves rather than resting on one point of them.
    s.set_position(0, cx - 1.6, cy - 1.0);
    s.set_position(1, cx + 1.6, cy - 1.0);
    s.set_position(2, cx, cy + 1.9);
    s.set_velocity(0, 0.0012, 0.0004);
    s.set_velocity(1, -0.0012, 0.0004);
    s.set_velocity(2, 0.0003, -0.0009);
    s.rebase();
    s
}

/// THE PLANT's mechanism: write `pt`'s knots into `slot`, whatever pair that slot serves.
///
/// Deliberately goes through the raw interpolator door rather than through
/// `load_pair_table`, which reads the species off the curve's own metadata and would put
/// it in the RIGHT slot — that correctness is exactly what plant (i) exists to check, so
/// the plant has to reach past it.
fn swap_curve_into(s: &mut Sim, slot: usize, pt: &PairTable) {
    let n = pt.r.len();
    let t = s.bank.table_slot_mut(slot);
    assert!(t.begin(n));
    for i in 0..n {
        assert!(t.knot(i, pt.r[i], pt.e[i], pt.f[i]));
    }
    let (r_e, d_e) = match pt.meta.well {
        Some(w) => (w.r_e, w.d_e),
        None => (0.0, 0.0),
    };
    assert_eq!(
        t.finish(r_e, d_e, pt.meta.e_asymptote),
        holon_render::table::LoadStatus::Ok,
        "the planted curve did not load"
    );
}

/// The pair reading for atoms `(i, j)`, in either order.
fn find_pair(s: &Sim, i: usize, j: usize) -> holon_render::sim::PairReading {
    *s.pairs[..s.pair_count]
        .iter()
        .find(|p| (p.i == i && p.j == j) || (p.i == j && p.j == i))
        .unwrap_or_else(|| panic!("no pair reading for atoms ({i}, {j})"))
}

/// A curve's asymptote-zeroed energy at `r`, read from the generated table by the same
/// cubic Hermite the sandbox uses.
///
/// Built through a scratch `PotentialTable` rather than by interpolating the columns here,
/// so the number this test compares against is produced by the interpolator under test and
/// not by a second one written for the occasion.
fn interp_u(pt: &PairTable, r: f64) -> f64 {
    let mut s = Box::new(Sim::empty());
    assert_eq!(
        load_pair_table(&mut s, pt, Host::Native),
        TABLE_OK,
        "the fixture curve did not load"
    );
    let slot = s.bank.slot_of_z(pt.meta.z_a, pt.meta.z_b).unwrap();
    s.bank.table_slot(slot).u(r)
}

/// Guard on the atom cap the fixtures assume.
#[test]
fn the_fixtures_fit_the_scene() {
    assert!(MAX_ATOMS >= 4, "the B1 fixture places four atoms");
    assert!(
        MAX_SPECIES >= 2,
        "a mixture needs at least two species; the cap is {MAX_SPECIES}"
    );
}

// ================================================================== plant (iv)
//
// THE UNCERTAINTY NOBODY WEIGHED.
//
// The provenance gate has asked since it was written whether a shipped table DECLARES an
// uncertainty. It never asked how big it was. A file could state a whole hartree — sixteen
// times Cl2's entire well depth — and be admitted, loaded, and integrated, because
// `uncertainty_ha > 0.0` was the whole test.
//
// It surfaced from the other side. The shipped `Cl2.json` carried `"converged": false` in
// its own metadata from the moment it was first emitted (commit 1a13c49) through two
// regenerations and three commits, and nothing on this side of the door read the field at
// any magnitude — `converged` appears nowhere in `holon-render`, not in the bank, not in
// the JSON reader, not in the viewer. That curve turned out to be FINE: its worst residual
// is 1.0005e-10, and under the tier ruling of 2026-08-30 that is as converged as f64
// arithmetic gets. The defect was never the file. It was that nothing here could have told
// the difference, which is the same guard-not-connected shape `PairCache::get`'s header
// congratulates itself on having closed — closed on one door of two.
//
// So the gate reads the NUMBER, not the producer's flag, for the reason `claimed_exact` is
// kept apart from `route`: a file's self-assessment is exactly the thing a file can get
// wrong, and a bar that lives in the other crate can move under this one without a compile
// error. Both legs below are DERIVED — the schema's own well-resolution, and the curve's
// own declared depth — so neither is a number this door chose.

/// PLANT (iv) — an uncertainty too large to resolve any well, refused, with the boundary
/// pinned and the derivation of the bar asserted rather than described.
#[test]
fn plant_iv_an_unresolvable_uncertainty_is_refused() {
    // THE BAR IS DERIVED. If someone gives this door its own number, this fires.
    assert_eq!(
        RESOLVABLE_UNCERTAINTY, holon_chem::pair::WELL_MIN_DEPTH,
        "the door's resolution bar has stopped being the schema's own well threshold; one \
         of them was edited alone, and a shipped curve can now claim a well the loader \
         cannot resolve (or be refused for a well it can)"
    );

    let d1 = D1Admission::NONE;
    // A heavy shipped determinant curve — Cl2's real shape, so nothing else in the gate
    // fires and the leg under test is the only thing that can speak.
    let base = TableProvenance {
        route: Route::Determinant,
        source: Source::Shipped,
        n_det: 324,
        n_basis: 18,
        uncertainty_ha: 1.0005e-10, // the shipped Cl2 curve's real worst residual
        claimed_exact: true,
        well_depth_ha: 0.064_577_384_997_733_13, // and its real well depth
    };

    // CARRIER. The real curve is admitted, so a refusal below is the plant and not the
    // fixture. Cl2's uncertainty is 6.5e8 times smaller than its own well.
    assert_eq!(
        base.admit(&d1, Host::Browser),
        Ok(()),
        "PLANT (iv) VOID: the unplanted Cl2 provenance is already refused, so the plants \
         below demonstrate nothing"
    );

    // LEG ONE: cannot resolve the shallowest well the schema recognises.
    let unresolvable = TableProvenance {
        uncertainty_ha: 1e-3,
        ..base
    };
    assert_eq!(
        unresolvable.admit(&d1, Host::Browser),
        Err(Refusal::UncertaintyExceedsResolution),
        "PLANT (iv) MISSED: a curve declaring a millihartree of error was admitted. It \
         cannot tell a bond from no bond, and the force loop would integrate it anyway."
    );

    // THE BOUNDARY BELONGS TO THE REFUSAL. Exactly at the bar is not below it.
    assert_eq!(
        TableProvenance { uncertainty_ha: RESOLVABLE_UNCERTAINTY, ..base }
            .admit(&d1, Host::Browser),
        Err(Refusal::UncertaintyExceedsResolution),
        "a curve whose uncertainty is EXACTLY the schema's well threshold was admitted; \
         the comparison has drifted to `<=` and the boundary case is the one that will \
         arrive"
    );
    assert_eq!(
        TableProvenance {
            uncertainty_ha: RESOLVABLE_UNCERTAINTY * (1.0 - 1e-12),
            ..base
        }
        .admit(&d1, Host::Browser),
        Ok(()),
        "a curve just INSIDE the bar was refused; the leg is firing on the wrong side"
    );

    // LEG TWO: inside the schema's resolution, but swallowed by its own well. This is a
    // different curve from leg one's and the reason the two refusals are named apart — a
    // reader told only 'uncertainty too large' would check it against 1e-4 and find it
    // comfortably smaller.
    let swallowed = TableProvenance {
        uncertainty_ha: 1e-5,
        well_depth_ha: 5e-6,
        ..base
    };
    assert!(
        swallowed.uncertainty_ha < RESOLVABLE_UNCERTAINTY,
        "PLANT (iv) VOID: leg two's fixture also trips leg one, so it cannot show that \
         the second leg is reachable at all"
    );
    assert_eq!(
        swallowed.admit(&d1, Host::Browser),
        Err(Refusal::UncertaintyExceedsWell),
        "PLANT (iv) MISSED: a curve whose error bar is twice the well it reports was \
         admitted as a bound pair"
    );

    // A curve that declares NO well is not refused for failing to resolve one. An unbound
    // pair is a real answer in this schema — He2 and Ne2 are the point — and a gate that
    // demanded a well would refuse the honest negative.
    assert_eq!(
        TableProvenance { uncertainty_ha: 1e-5, well_depth_ha: 0.0, ..base }
            .admit(&d1, Host::Browser),
        Ok(()),
        "an unbound curve was refused for having no well to resolve; He2 and Ne2 are \
         supposed to say exactly that"
    );

    println!(
        "  PLANT (iv) CAUGHT on both legs. Bar {RESOLVABLE_UNCERTAINTY:.0e} Ha, derived \
         from the schema's well threshold; the real Cl2 curve clears it by 6.5e8."
    );
}

/// PLANT (iv) THROUGH THE PRODUCTION LOADER — the same defect on the path the sandbox
/// actually uses, because a rule demonstrated only against a struct literal is a rule
/// nothing in the product has to obey.
#[test]
fn plant_iv_the_production_loader_refuses_a_curve_swallowed_by_its_own_error_bar() {
    let mut s = scene(&[HYDROGEN, LITHIUM]);
    let slot = s.bank.slot_of_z(1, 3).unwrap();
    assert!(
        s.bank.is_filled(slot),
        "the fixture did not load H-Li through the production loader"
    );

    let clean = curve(HYDROGEN, LITHIUM);
    let well = clean
        .meta
        .well
        .expect("PLANT (iv) VOID: the H-Li fixture curve declares no well, so an \
                uncertainty cannot swallow one");
    // CARRIER: the unplanted curve's residual is far inside its own well, so the refusal
    // below is the plant.
    assert!(
        clean.meta.worst_residual < well.d_e,
        "PLANT (iv) VOID: the unplanted H-Li curve is ALREADY swallowed by its own error \
         bar (residual {:.3e}, well {:.3e})",
        clean.meta.worst_residual,
        well.d_e
    );

    // WHICH LEG THIS DOOR CAN REACH, established before the plant rather than discovered
    // by it. `locate_well` refuses to call anything shallower than WELL_MIN_DEPTH a well,
    // and the door's bar IS that constant — so any engine-produced curve whose error bar
    // covers its own well has already exceeded the resolution bar, and leg one speaks
    // first. Leg two's window is the shipped-file door, where D_e is a declaration.
    assert!(
        well.d_e > RESOLVABLE_UNCERTAINTY,
        "the schema's well threshold and the door's resolution bar have come unpinned \
         (well {:.3e} against bar {RESOLVABLE_UNCERTAINTY:.0e}): UncertaintyExceedsWell is \
         now reachable through THIS door too, and this test no longer describes the gate",
        well.d_e
    );

    // THE PLANT: the same curve, same energies, reporting an error bar that covers its
    // own bond. Nothing about the numbers changes — only what the curve says it knows,
    // which is exactly what a shipped file is free to get wrong.
    let mut planted = clean.clone();
    planted.meta.worst_residual = well.d_e * 2.0;
    let status = load_pair_table(&mut s, &planted, Host::Native);
    println!(
        "plant (iv) through load_pair_table: status {status} (expected {}), planted \
         residual {:.3e} against a well of {:.3e} and a bar of {RESOLVABLE_UNCERTAINTY:.0e}",
        holon_render::refusal_code(Refusal::UncertaintyExceedsResolution),
        planted.meta.worst_residual,
        well.d_e
    );
    assert_eq!(
        status,
        holon_render::refusal_code(Refusal::UncertaintyExceedsResolution),
        "PLANT (iv) MISSED: a curve whose declared uncertainty is twice its own well depth \
         went through the production loader unrefused"
    );
    assert!(
        !s.bank.is_filled(slot),
        "PLANT (iv) MISSED: the curve was refused and is STILL IN THE BANK, so the force \
         loop would evaluate it"
    );
    assert!(!s.pairs_ready());

    // NATIVE, deliberately. The browser split is not what refused this one — the same
    // plant must fail on a host where no split rule applies, or the gate above is only
    // the split wearing a new name. (This call is already Host::Native; the assertion
    // exists to say that it MUST be, so a later edit to Host::Browser is a visible
    // weakening rather than a silent one.)
    assert_eq!(
        load_pair_table(&mut s, &planted, Host::Browser),
        holon_render::refusal_code(Refusal::UncertaintyExceedsResolution),
        "the resolution leg gives different answers on the two hosts, so it is the split \
         rule wearing a new name rather than a property of the curve"
    );

    // THE POSITIVE CONTROL. Without it this passes against a loader that refuses
    // everything.
    assert_eq!(
        load_pair_table(&mut s, clean, Host::Native),
        TABLE_OK,
        "the loader refused the unplanted curve, so the refusals above say nothing about \
         the uncertainty"
    );
    assert!(s.bank.is_filled(slot) && s.pairs_ready());
    println!("  PLANT (iv) CAUGHT through the production loader, and the clean curve still loads.");
}
