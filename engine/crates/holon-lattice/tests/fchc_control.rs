//! G11's second leg: the FCHC-24 numbers, through the enumerator that already owned them.
//!
//! `holon-lattice`'s own [`Model`] is 2D — its directions are `[i64; 2]` — so it cannot
//! enumerate a 4D mode set, and pretending otherwise would put a third implementation of the
//! census in the tree. `holon-mesh::fchc` is the enumerator `MESH_DESIGN.md` §2.1's numbers
//! were checked with, and it runs the SAME routine over FHP-6 as its own control. This test
//! runs it here so the campaign's G11 has both legs under one invocation, and so the two
//! crates' FHP-6 answers are compared against each other rather than each against itself.
//!
//! `holon-mesh` is a DEV-dependency only: the library's dependency profile stays one crate
//! (`ciris-sim-core` with `alloc`), so nothing here can perturb the core's feature
//! resolution or `ci-gates.sh`'s isolation gates.

use holon_lattice::state::Model;

#[test]
fn the_two_crates_agree_about_fhp_and_holon_mesh_supplies_the_fchc_leg() {
    // --- this crate's census, and holon-mesh's, on the same mode set.
    let (sectors, hist) = Model::fhp6().census();
    assert_eq!((sectors, hist), (53, vec![(1, 44), (2, 7), (3, 2)]));

    let mesh_fhp = holon_mesh::fchc::enumerate(&holon_mesh::fchc::fhp_directions())
        .expect("the FHP-6 mode set is within the enumerator's momentum range");
    assert_eq!(mesh_fhp.local_states, 64);
    assert_eq!(mesh_fhp.sectors, sectors, "the two crates disagree about the FHP-6 census");
    assert_eq!(mesh_fhp.largest_sector, 3);
    assert_eq!(mesh_fhp.singleton_sectors, 44);
    assert_eq!(mesh_fhp.max_occupancy, 6);

    // --- the FCHC-24 leg, unchanged, from the crate that owns it.
    let fchc = holon_mesh::fchc::enumerate(&holon_mesh::fchc::fchc_directions())
        .expect("FCHC-24 is within the enumerator's momentum range");
    assert_eq!(fchc.local_states, 16_777_216, "MESH_DESIGN 2.1's local-state count moved");
    assert_eq!(fchc.sectors, 72_047, "MESH_DESIGN 2.1's sector count moved");
    assert_eq!(fchc.largest_sector, 11_740, "MESH_DESIGN 2.1's largest sector moved");
}

/// P5 for the FCHC leg: perturbing the mode set must move the numbers. An enumerator that
/// returns 72,047 whatever it is fed is not a control.
#[test]
fn the_fchc_enumerator_moves_when_its_mode_set_does() {
    let mut d = holon_mesh::fchc::fchc_directions();
    d.pop();
    let short = holon_mesh::fchc::enumerate(&d).expect("23 directions stay in range");
    assert_ne!(short.sectors, 72_047);
    assert_eq!(short.local_states, 8_388_608);
}
