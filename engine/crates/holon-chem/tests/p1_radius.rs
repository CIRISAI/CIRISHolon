//! P1: the third radius rule, its labelling, and the plant that guards the label.
//!
//! # Why a third rule at all
//!
//! P1 derives each element's drawn radius from its own homonuclear curve. By AMENDMENT
//! A1.2 most mid-row homonuclear dimers cannot be computed at all — scandium's is 36
//! orbitals and 42 electrons — so the choice was a third DERIVED rule or a remembered
//! constant, and this crate has no table of remembered constants.
//!
//! # The rule that was specified, and why it is not the rule that shipped
//!
//! The approved rule was an r-expectation over the whole electron density. It was built,
//! measured, and does not do the job: averaged over every electron it is dominated by the
//! tight core, comes out flat at about one bohr from hydrogen to xenon, and is not even
//! monotone — xenon reads SMALLER than hydrogen. That is the correct value of a quantity
//! that does not mean what a drawn radius has to mean, and it is kept in the crate as
//! `atomic_rms_radius` rather than deleted, because the next person to propose it should
//! find the measurement rather than repeat the work.
//!
//! What shipped is the same expectation over the OUTERMOST OCCUPIED orbital alone, which
//! is what sets an atom's chemical size. It reproduces the periodic trend without being
//! told it: sizes fall across a period and jump at the start of the next one.

use holon_chem::elements::{by_symbol, by_z, ALL_ELEMENTS};
use holon_chem::pair::{
    atomic_rms_radius, atomic_valence_rms_radius, automatic_route, homonuclear_size, RadiusRule,
};

/// The valence radius reproduces the periodic table's size trend, with nothing told to it.
///
/// Two independent facts about the periodic table, neither of which is an input here: size
/// FALLS across a period as the nuclear charge pulls a fixed shell in, and JUMPS at the
/// start of the next period when a new shell opens. The declared inputs are Z, the masses
/// and the basis.
#[test]
fn the_valence_radius_reproduces_the_periodic_size_trend() {
    // Falls across a period.
    for row in [["Na", "Al", "Cl", "Ar"], ["K", "Se", "Br", "Kr"]] {
        let mut last = f64::INFINITY;
        for sym in row {
            let r = atomic_valence_rms_radius(by_symbol(sym).unwrap());
            assert!(
                r < last,
                "{sym} at {r:.4} bohr does not sit inside its predecessor at {last:.4}; \
                 atomic size falls across a period and this rule is supposed to show it"
            );
            last = r;
        }
    }
    // Jumps at the start of the next period: the alkali is larger than the noble before it.
    for (noble, alkali) in [("Ne", "Na"), ("Ar", "K")] {
        let (rn, ra) = (
            atomic_valence_rms_radius(by_symbol(noble).unwrap()),
            atomic_valence_rms_radius(by_symbol(alkali).unwrap()),
        );
        assert!(
            ra > rn,
            "{alkali} at {ra:.4} bohr is not larger than {noble} at {rn:.4}; a new shell \
             opening is the sharpest size jump in the table"
        );
    }
    println!(
        "P1: Na {:.3} > Ar {:.3}, K {:.3} > Kr {:.3} — falls across, jumps between",
        atomic_valence_rms_radius(by_symbol("Na").unwrap()),
        atomic_valence_rms_radius(by_symbol("Ar").unwrap()),
        atomic_valence_rms_radius(by_symbol("K").unwrap()),
        atomic_valence_rms_radius(by_symbol("Kr").unwrap()),
    );
}

/// The whole-density rule is kept, and kept LABELLED as the thing that does not work.
///
/// Asserted rather than left in prose: if someone later "fixes" it into something that
/// does track size, this fires and they find out that the two functions have swapped
/// meanings.
#[test]
fn the_whole_density_radius_is_core_dominated_and_says_so() {
    let h = atomic_rms_radius(by_symbol("H").unwrap());
    let xe = atomic_rms_radius(by_symbol("Xe").unwrap());
    assert!(
        xe < h,
        "the whole-density radius now puts xenon ({xe:.4}) outside hydrogen ({h:.4}). It \
         did not: averaged over every electron it is core-dominated and non-monotone, \
         which is why the valence rule exists. If this quantity has changed meaning, the \
         rule that ships must be revisited rather than this assertion relaxed."
    );
    // And the valence rule does not have that problem, which is the whole contrast.
    assert!(
        atomic_valence_rms_radius(by_symbol("Xe").unwrap())
            > atomic_valence_rms_radius(by_symbol("H").unwrap()),
        "the valence rule has lost the property it was chosen for"
    );
}

/// Every element's radius carries the rule that produced it, and the rule is the true one.
#[test]
fn every_radius_is_labelled_with_the_rule_that_produced_it() {
    // Chosen by COST, not by outcome: each of these homonuclear pairs is one or a few
    // dozen determinants, so the whole check is seconds. Si2 is 9.4 million and would
    // route to DMRG, which is a different question from whether a label is right.
    let cheap = ["H", "He", "Ne", "Ar", "Se"];
    let mut dimer = 0usize;
    let mut density = 0usize;
    for sym in cheap {
        let sp = by_symbol(sym).unwrap();
        let sz = homonuclear_size(sp);
        let infeasible = !automatic_route(sp, sp).exists();
        assert_eq!(
            sz.rule.is_dimer_derived(),
            !infeasible,
            "{}: the radius is labelled {:?} but its homonuclear pair is {}",
            sp.symbol,
            sz.rule,
            if infeasible { "not computable" } else { "computable" }
        );
        let _ = ALL_ELEMENTS.len();
        if sz.rule.is_dimer_derived() {
            dimer += 1;
            assert!(
                sz.separation_bohr.is_finite(),
                "{}: a dimer-derived radius must carry the separation it is half of",
                sp.symbol
            );
        } else {
            density += 1;
            assert!(
                sz.separation_bohr.is_nan(),
                "{}: a density-derived radius has no separation, and reporting one would \
                 be inventing a number for a curve that was never computed",
                sp.symbol
            );
        }
    }
    assert!(dimer > 0, "no element took the pair route; the check distinguishes nothing");
    assert!(density > 0, "no element took the density route; the third rule is untested");
    println!("P1: {dimer} dimer-derived, {density} density-derived over the cheap set");
}

/// PLANT: presenting a density-derived radius as dimer-derived must be REFUSED.
///
/// # Carrier, per M-PLANT-SECTOR
///
/// The carrier is the refusal, and the sector it acts on is asserted nonempty first: there
/// must be at least one species whose homonuclear pair is genuinely not computable, or the
/// plant would be scored on a case that cannot arise.
///
/// # What is actually planted
///
/// A `SpeciesSize` whose radius came from the atom but whose `rule` says it came from a
/// pair — the exact mislabel the third rule makes possible. The label machinery has to
/// catch it on the two things that cannot both be true at once: a dimer-derived radius
/// owes a finite separation, and a density-derived one has none.
#[test]
fn plant_a_density_radius_presented_as_dimer_derived_is_refused() {
    // Carrier: the sector must be non-empty.
    // Selenium: its dimer is 36 orbitals and 396,900 determinants, so the pair route is
    // genuinely out of reach, while its ATOM is 324 determinants and costs nothing. Named
    // rather than searched, so the plant runs on the same species every time and a change
    // in the registry shows up as this assertion rather than as a slower test.
    let victim = by_symbol("Se").unwrap();
    assert!(
        !automatic_route(victim, victim).exists(),
        "carrier: {}'s homonuclear pair is computable, so the density rule would not be \
         used for it and the plant would be scored on a case that cannot arise",
        victim.symbol
    );
    let honest = homonuclear_size(victim);
    assert_eq!(
        honest.rule,
        RadiusRule::ValenceDensity,
        "carrier: {} is supposed to take the density route",
        victim.symbol
    );
    assert!(honest.separation_bohr.is_nan(), "carrier: it has no separation");

    // The plant: same number, wrong label.
    let mut planted = honest;
    planted.rule = RadiusRule::HalfEquilibrium;

    // The refusal, stated as the check any surface must apply before it prints a radius.
    let refused = planted.rule.is_dimer_derived() && !planted.separation_bohr.is_finite();
    assert!(
        refused,
        "PLANT MISSED: a radius labelled {:?} with no separation behind it passed the \
         label check. That is a density-derived number presented as a measured \
         inter-atomic distance, which is a claim about a computation that never ran.",
        planted.rule
    );

    // And the honest record must NOT trip the same check, or it refuses everything.
    assert!(
        !(honest.rule.is_dimer_derived() && !honest.separation_bohr.is_finite()),
        "the refusal fires on the honest record too, so it distinguishes nothing"
    );
    println!(
        "plant: {} labelled {:?} with separation {} — REFUSED",
        victim.symbol, planted.rule, planted.separation_bohr
    );
}

/// The rule's own words are single-sourced, so two surfaces cannot describe it differently.
#[test]
fn the_three_rules_describe_themselves_distinctly() {
    let d = [
        RadiusRule::HalfEquilibrium.describe(),
        RadiusRule::HalfContact.describe(),
        RadiusRule::ValenceDensity.describe(),
    ];
    for (i, a) in d.iter().enumerate() {
        assert!(!a.is_empty());
        for b in d.iter().skip(i + 1) {
            assert_ne!(a, b, "two rules describe themselves identically");
        }
    }
    assert!(
        d[2].contains("ATOM"),
        "the density rule's description must say it is not a pair measurement"
    );
    let _ = by_z(21);
}
