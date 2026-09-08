//! THE FENCE COUNT IS A CENSUS, AND THE ENUMERATION IS ITS REFEREE.
//!
//! `Sim::fenced_triples` reports how many atom triples no three-body surface serves. Under
//! the seam it used to answer by walking every `a < b < c` — a cubic count of something that
//! is not a force, and 0.23 s of every 0.34 s force pass at 750 atoms. It is now arithmetic
//! on the species-and-unit census, and the enumeration is kept as
//! `Sim::fenced_triples_enumerated` so this file can hold one against the other rather than
//! against a restatement of itself.
//!
//! The scenes are chosen so every branch of the drop rule is exercised and so the equality is
//! not vacuous (M-VACUOUS-SUCCESS): a scene with FREE atoms, a scene whose intra-unit triples
//! are themselves unserved, the `TriplesAcross` diagnostic control that drops nothing, the
//! seam off, no surfaces at all, and the campaign's own 128-water periodic box.

use holon_chem::elements::{HYDROGEN, OXYGEN, Species};
use holon_render::seam::{SeamModel, SeamPlant, FREE};
use holon_render::sim::Sim;
use holon_render::waterbox::liquid_box;

#[path = "common/field2_scenes.rs"]
#[allow(dead_code)]
mod field2_scenes;
use field2_scenes::{dimer_positions, ring_positions, scene, water_at};

/// DECLARED coefficients. The fence is a fact about MEMBERSHIP and about which surfaces are
/// loaded, never about the wall's strength, so any model that switches the seam on will do —
/// and saying so here is cheaper than reading a harvest this file does not depend on.
fn model() -> SeamModel {
    SeamModel { a: 0.5, b: 1.2, p: 0.02, c: 1.5, c6: 10.0, r_cut: 6.0, ..SeamModel::NO_WALL }
}

fn seamed(species: &[Species], pos: &[[f64; 3]], edge: f64) -> Box<Sim> {
    let mut s = scene(species, pos, edge, 293.0);
    s.set_field(true, None).expect("the open box admits the field");
    s.set_seam(Some(model())).expect("no acuity frame is installed");
    s
}

fn free_atoms(s: &Sim) -> usize {
    (0..s.n).filter(|&i| s.unit_of[i] == FREE).count()
}

/// Two waters, and beside them an oxygen carrying ONE hydrogen — which is not a unit, so both
/// of those atoms read FREE. That is the branch the census counts by complement.
fn two_waters_and_a_stray() -> (Vec<Species>, Vec<[f64; 3]>, f64) {
    let (mut species, mut pos) = dimer_positions();
    // far enough that the stray pair is nobody's unit and near enough to stay in the box
    species.push(OXYGEN);
    pos.push([25.0, 15.0, 15.0]);
    species.push(HYDROGEN);
    pos.push([26.8, 15.0, 15.0]);
    (species, pos, 40.0)
}

/// Four waters on a ring, plus a lone hydrogen well away from every oxygen: one FREE atom
/// against four intact units.
fn ring_and_a_lone_hydrogen() -> (Vec<Species>, Vec<[f64; 3]>, f64) {
    let (mut species, mut pos) = ring_positions();
    species.push(HYDROGEN);
    pos.push([32.0, 32.0, 32.0]);
    (species, pos, 44.0)
}

/// A third arrangement with two strays of the same element, so the census's `za == zb` and
/// `zb == zc` branches both carry a count.
fn waters_and_two_stray_oxygens() -> (Vec<Species>, Vec<[f64; 3]>, f64) {
    let mut species = Vec::new();
    let mut pos = Vec::new();
    for (k, o) in [[12.0, 12.0, 12.0], [12.0, 12.0, 18.0]].iter().enumerate() {
        let side = if k == 0 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
        for (m, p) in water_at(*o, [0.0, 0.0, 1.0], side).iter().enumerate() {
            species.push(if m == 0 { OXYGEN } else { HYDROGEN });
            pos.push(*p);
        }
    }
    species.push(OXYGEN);
    pos.push([26.0, 12.0, 12.0]);
    species.push(OXYGEN);
    pos.push([26.0, 12.0, 20.0]);
    (species, pos, 40.0)
}

/// Every scene this file holds the two counts against each other on, with what each is for.
fn scenes() -> Vec<(String, Box<Sim>)> {
    let mut out: Vec<(String, Box<Sim>)> = Vec::new();

    let (sp, p) = dimer_positions();
    out.push(("dimer, seam on".to_string(), seamed(&sp, &p, 40.0)));

    let (sp, p) = ring_positions();
    out.push(("cyclic tetramer, seam on".to_string(), seamed(&sp, &p, 44.0)));

    let (sp, p, e) = two_waters_and_a_stray();
    out.push(("two waters and a stray O-H, seam on".to_string(), seamed(&sp, &p, e)));

    let (sp, p, e) = ring_and_a_lone_hydrogen();
    out.push(("tetramer and a lone hydrogen, seam on".to_string(), seamed(&sp, &p, e)));

    let (sp, p, e) = waters_and_two_stray_oxygens();
    out.push(("two waters and two stray oxygens, seam on".to_string(), seamed(&sp, &p, e)));

    // The intra-unit triple made unserved: with the (O,H,H) surface gone, each unit's own
    // O-H-H is fenced, which is the ONLY way the census's within-unit term is non-zero on a
    // water scene (a unit is one oxygen and exactly two hydrogens, so it has exactly one
    // triple). Emptied AFTER the scene is built, so the assignment is a real one.
    let (sp, p) = ring_positions();
    let mut s = seamed(&sp, &p, 44.0);
    s.water = holon_chem::water::WaterTable::empty();
    out.push(("tetramer with the (O,H,H) surface unloaded".to_string(), s));

    // The diagnostic control: plant (ii) serves the surfaces ACROSS the seam, so nothing is
    // dropped and the census is the plain one.
    let (sp, p, e) = ring_and_a_lone_hydrogen();
    let mut s = scene(&sp, &p, e, 293.0);
    s.set_field(true, None).unwrap();
    s.seam_plant = SeamPlant::TriplesAcross;
    s.set_seam(Some(model())).unwrap();
    out.push(("tetramer and a lone hydrogen, plant TriplesAcross".to_string(), s));

    // The seam off, with the field on: no drop rule at all.
    let (sp, p, e) = ring_and_a_lone_hydrogen();
    let mut s = scene(&sp, &p, e, 293.0);
    s.set_field(true, None).unwrap();
    out.push(("tetramer and a lone hydrogen, seam off".to_string(), s));

    // No surface at all: the pre-T3 loop counted nothing and the fence is a reading of that
    // loop, so both answers are zero (DRY-residual register R-3).
    let (sp, p) = ring_positions();
    let mut s = seamed(&sp, &p, 44.0);
    s.water = holon_chem::water::WaterTable::empty();
    s.trimer = holon_chem::trimer::TrimerTable::empty();
    out.push(("tetramer with no three-body surface loaded".to_string(), s));

    out
}

#[test]
fn the_fence_census_is_the_enumeration() {
    let mut nonzero = 0usize;
    let mut with_free = 0usize;
    let mut with_units = 0usize;
    for (name, s) in scenes() {
        let census = s.fenced_triples();
        let referee = s.fenced_triples_enumerated();
        assert_eq!(
            census, referee,
            "{name}: the census reads {census} fenced triples where the enumeration reads {referee}. \
             The enumeration is the definition; the census has to reproduce it exactly, including \
             its handling of free atoms and of the seam's dropped cross-unit triples."
        );
        if census > 0 {
            nonzero += 1;
        }
        if free_atoms(&s) > 0 {
            with_free += 1;
        }
        if s.seam_work.units > 0 {
            with_units += 1;
        }
    }
    // M-VACUOUS-SUCCESS: an equality that only ever compared 0 with 0 would prove nothing.
    assert!(nonzero >= 3, "only {nonzero} scenes fenced anything; the equality is close to vacuous");
    assert!(with_free >= 3, "only {with_free} scenes had a FREE atom, and the complement term is untested");
    assert!(with_units >= 5, "only {with_units} scenes had units, and the drop rule is untested");
}

/// The within-unit term, isolated: four waters with no (O,H,H) surface fence exactly their
/// own four triples and nothing else, because every other triple in that scene crosses a unit
/// and the seam drops it.
#[test]
fn the_within_unit_term_is_one_triple_per_unit() {
    let (sp, p) = ring_positions();
    let mut s = seamed(&sp, &p, 44.0);
    assert_eq!(s.seam_work.units, 4, "the tetramer did not read as four units");
    assert_eq!(s.fenced_triples(), 0, "with the (O,H,H) surface loaded every intra-unit triple is served");
    s.water = holon_chem::water::WaterTable::empty();
    assert_eq!(s.fenced_triples(), 4, "four units, one (O,H,H) triple each, no surface to serve them");
    assert_eq!(s.fenced_triples(), s.fenced_triples_enumerated());
}

/// The campaign's own box: 128 waters, periodic, the seam on. This is the scene the cost was
/// measured on, and the one the census had to be right about before it could replace the
/// enumeration on a force pass.
#[test]
fn the_census_is_the_enumeration_on_the_water_box() {
    let (species, pos, l) = liquid_box(4, 0.997, 0x4c49_5155_4944);
    assert_eq!(species.len(), 384, "the 4^3 body-centred box carries 128 waters");
    let mut s = seamed(&species, &pos, l);
    assert_eq!(s.seam_work.units, 128, "the box did not read as 128 units");
    assert_eq!(s.fenced_triples(), s.fenced_triples_enumerated());
    // and again with the (O,H,H) surface gone, so the number under test is not zero
    s.water = holon_chem::water::WaterTable::empty();
    assert_eq!(s.fenced_triples(), 128, "128 units, one unserved (O,H,H) triple each");
    assert_eq!(s.fenced_triples(), s.fenced_triples_enumerated());
}
