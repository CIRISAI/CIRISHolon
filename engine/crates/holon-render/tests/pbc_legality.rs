//! THE LEGALITY RADIUS: every interaction channel's reach, not the decomposition's.
//!
//! `Sim::pbc_ok` guards the minimum-image convention — past half the shortest edge an atom
//! sits inside the reach of two images of the same partner, the reduction picks one, and the
//! missing one is a force that silently is not there. It asked `Sim::list_cutoff` for the
//! radius, and **that is a different question**: `list_cutoff` is the radius the CELL
//! DECOMPOSITION must cover, and when no pair truncation is declared the pair loop runs the
//! complete `N²/2` sum and consults no cell list at all, so `list_cutoff` correctly reads
//! **zero**. The guard then compared zero to the half-edge and passed everything.
//!
//! Measured by the cryo campaign before this file existed: pair-only periodic boxes of
//! half-edge 8.000, 4.500 and 1.900 bohr all admitted against an H–H table reaching 10.240.
//!
//! So the fix is not to `list_cutoff`, which is right for what it is for. Minimum-image
//! validity depends on how far the FORCE LAW reaches, and the two numbers differ exactly
//! when the pair sector runs complete.
//!
//! One planted violation per channel, each with the box just above the floor ADMITTED and
//! just below REFUSED — a refusal that fires everywhere is not a criterion, and a channel
//! that never binds is not covered.

use holon_chem::elements::HYDROGEN;
use holon_chem::pair::generate_pair_table;
use holon_render::bank::Host;
use holon_render::longrange::{CurveTail, FarSector};
use holon_render::sim::{Boundary, Dims, Sim};
use holon_render::{load_pair_table, TABLE_OK};

/// The band each channel is probed at: a box at `1.02 × 2·reach` must be admitted and one at
/// `0.98 × 2·reach` must be refused. Tight enough that a channel missing from the radius
/// cannot hide behind another channel's slack.
const OVER: f64 = 1.02;
const UNDER: f64 = 0.98;

fn periodic_scene(edge: f64) -> Box<Sim> {
    let mut sim = Box::new(Sim::empty());
    sim.dims = Dims::Two;
    sim.boundary = Boundary::Periodic;
    sim.width = edge;
    sim.height = edge;
    sim.depth = edge;
    sim
}

fn synthetic_tail(p: f64, r_max: f64) -> CurveTail {
    let n = 40usize;
    let r: Vec<f64> = (0..n)
        .map(|k| 4.0 + (r_max - 4.0) * k as f64 / (n - 1) as f64)
        .collect();
    let u: Vec<f64> = r.iter().map(|x| -x.powf(-p)).collect();
    CurveTail {
        hi_b: p / r_max,
        r,
        u,
        solver_exit: "Converged",
        solver_budget_iterations: 5000,
        uncertainty_hartree: 1.0e-11,
    }
}

/// Assert a channel BINDS: admitted just above its floor, refused just below.
fn assert_channel_binds(name: &str, reach: f64, build: impl Fn(f64) -> Box<Sim>) {
    let wide = build(2.0 * reach * OVER);
    assert!(
        wide.pbc_ok(),
        "{name}: a box at {OVER}x the floor was REFUSED — the radius is larger than \
         this channel's reach ({reach}), so the probe is not measuring this channel. \
         pbc_margin = {:?}",
        wide.pbc_margin()
    );
    let tight = build(2.0 * reach * UNDER);
    assert!(
        !tight.pbc_ok(),
        "{name}: a box at {UNDER}x the floor was ADMITTED — this channel's reach ({reach}) \
         is MISSING from the legality radius. pbc_margin = {:?}",
        tight.pbc_margin()
    );
}

// ------------------------------------------------- channel 1: the pair table, untruncated

#[test]
fn the_untruncated_pair_table_binds() {
    // THE DEFECT. With no truncation declared the pair loop is the complete sum over every
    // separation, evaluated at the MINIMUM IMAGE — so the table's own support is the reach,
    // and a box narrower than twice it double-counts nothing and single-counts what should
    // have been two images.
    let pt = generate_pair_table(HYDROGEN, HYDROGEN, 96);
    let probe = {
        let mut s = periodic_scene(100.0);
        assert_eq!(load_pair_table(&mut s, &pt, Host::Native), TABLE_OK);
        s.reset(4);
        s
    };
    let slot = probe.bank.slot_of_z(1, 1).expect("H-H registered");
    let r_max = probe.bank.table_slot(slot).r_max();
    assert!(r_max > 1.0, "the H-H table has no support to speak of");

    assert_channel_binds("untruncated pair table", r_max, |edge| {
        let mut s = periodic_scene(edge);
        assert_eq!(load_pair_table(&mut s, &pt, Host::Native), TABLE_OK);
        s.reset(4);
        s
    });
}

// --------------------------------------------------- channel 2: the declared pair truncation

#[test]
fn a_declared_pair_truncation_binds_at_its_own_cutoff() {
    // A declared switch is EXACTLY zero past `r_cut`, so the reach shrinks to it — and a
    // truncation must be able to make a box legal that the full table's support would not.
    let pt = generate_pair_table(HYDROGEN, HYDROGEN, 96);
    let mut wide = periodic_scene(400.0);
    assert_eq!(load_pair_table(&mut wide, &pt, Host::Native), TABLE_OK);
    wide.reset(4);
    assert!(wide.set_pair_cutoff(1.0e-12), "a cutoff derives at 1e-12 Ha");
    let (r_in, r_cut) = wide.pair_switch().expect("a switch is declared");
    assert!(r_cut > r_in);

    assert_channel_binds("declared pair truncation", r_cut, |edge| {
        // Declared on a box that can honour it, THEN the box is shrunk. `set_pair_cutoff`
        // refuses a cutoff the box cannot carry, so declaring it on a small box would test
        // that refusal instead of this radius — and shrinking afterwards is the real hazard
        // anyway, which is what `scale_box`'s door exists for.
        let mut s = periodic_scene(400.0);
        assert_eq!(load_pair_table(&mut s, &pt, Host::Native), TABLE_OK);
        s.reset(4);
        assert!(s.set_pair_cutoff(1.0e-12), "the cutoff derives on the wide box");
        assert_eq!(s.pair_switch(), Some((r_in, r_cut)));
        s.width = edge;
        s.height = edge;
        s.depth = edge;
        s
    });
}

// ------------------------------------------------------------- channel 3: the three-body sector

#[test]
fn the_three_body_sector_binds() {
    // The flag is flipped rather than a surface generated, which is the pattern
    // `tests/saturation.rs` already uses: `three_body_cutoff` reads `.loaded`, and this test
    // asks only `pbc_ok`, which touches no table.
    assert_channel_binds("three-body sector", holon_chem::trimer::R_HI, |edge| {
        let mut s = periodic_scene(edge);
        s.reset(4);
        s.trimer.loaded = true;
        s
    });
}

// ------------------------------------------------------------ channel 4: the far sector's near half

#[test]
fn the_far_sectors_near_radius_binds() {
    // A declared far sector hands the NEAR sector everything up to `R_s`, and the near
    // sector reduces to the minimum image — so `R_s` is a reach like any other. The far
    // sector's own `min_edge >= 2 R_s` check stays per-pass and is not this gate: one
    // guards the door, the other guards a call that never went through it.
    let r_s = 20.0f64;
    assert_channel_binds("far sector near radius", r_s, |edge| {
        let mut s = periodic_scene(edge);
        s.reset(4);
        let far = FarSector::build(&[Some(synthetic_tail(6.0, r_s))], r_s, 1.0e-9, Dims::Two)
            .expect("the far sector builds");
        s.far = Some(Box::new(far));
        s
    });
}

// --------------------------------------------------------------- the radius is a MAX, not a pick

#[test]
fn the_legality_radius_is_the_max_over_channels_not_any_one_of_them() {
    // Two channels present at once must give the LARGER floor. A radius that returned the
    // three-body number alone would pass every one of the per-channel tests above and still
    // admit a box the pair table cannot honour — which is the shape of the original defect
    // one level up.
    let pt = generate_pair_table(HYDROGEN, HYDROGEN, 96);
    let probe = {
        let mut s = periodic_scene(100.0);
        assert_eq!(load_pair_table(&mut s, &pt, Host::Native), TABLE_OK);
        s.reset(4);
        s
    };
    let slot = probe.bank.slot_of_z(1, 1).expect("H-H registered");
    let r_max = probe.bank.table_slot(slot).r_max();
    let three = holon_chem::trimer::R_HI;
    let bigger = r_max.max(three);
    let smaller = r_max.min(three);
    assert!(bigger > smaller, "the two channels must differ for this to test anything");

    // A box legal for the SMALLER channel and illegal for the larger must be refused. Its
    // half-edge is the MIDPOINT of the two reaches, which is the only construction that is
    // between them for any pair of them — a fixed multiple of the smaller is not, and the
    // first version of this test used `2 * smaller * 1.5` and shot past the larger floor.
    // The precondition below is what said so.
    let edge = smaller + bigger; // half-edge = (smaller + bigger) / 2
    assert!(
        0.5 * edge >= smaller && 0.5 * edge < bigger,
        "the probe box must sit between the two floors: half-edge {} against {smaller} \
         and {bigger}",
        0.5 * edge
    );
    let mut s = periodic_scene(edge);
    assert_eq!(load_pair_table(&mut s, &pt, Host::Native), TABLE_OK);
    s.reset(4);
    s.trimer.loaded = true;
    assert!(
        !s.pbc_ok(),
        "a box clearing only the smaller channel was admitted; the radius is not a max. \
         pbc_margin = {:?}, reaches: pair {r_max}, three-body {three}",
        s.pbc_margin()
    );
}

// ------------------------------------------------------- the door inherits whatever this returns

#[test]
fn scale_box_refuses_a_shrink_that_breaks_the_pair_tables_reach() {
    // `ScaleRefusal::BreaksPeriodicImages` reads `pbc_margin`, so it inherited the hole: with
    // the radius at zero the door never refused a shrink on a pair-only scene, whatever the
    // box became. The door is not re-implemented here — this asserts it now binds.
    let pt = generate_pair_table(HYDROGEN, HYDROGEN, 96);
    let mut s = periodic_scene(100.0);
    assert_eq!(load_pair_table(&mut s, &pt, Host::Native), TABLE_OK);
    s.reset(4);
    let slot = s.bank.slot_of_z(1, 1).expect("H-H registered");
    let r_max = s.bank.table_slot(slot).r_max();

    // A shrink that keeps the half-edge above the table's reach is admitted...
    assert!(s.pbc_ok(), "the wide box starts legal");
    assert!(s.scale_box(0.5).is_ok(), "50 bohr still clears {r_max}");
    // ...and one that takes it under is refused, leaving the scene where it was.
    let before = (s.width, s.height);
    assert!(
        s.scale_box(0.2).is_err(),
        "a shrink to a half-edge of {} was admitted against a reach of {r_max}",
        0.5 * s.width * 0.2
    );
    assert_eq!((s.width, s.height), before, "a refused move mutated the box");
}

// ------------------------------------------------------------------- the boundary door

/// The switch door: every route INTO a wrapping box now refuses an illegal one.
///
/// `scale_box` guards the box changing under a wrapping scene and `set_pair_cutoff` guards a
/// truncation declared on one, but neither covers the scene that simply *becomes* periodic —
/// and that is the route the ABI's mode 2 takes. A door on each is what makes the illegal
/// state unreachable except by writing the public field, which tests do and which `pbc_ok`
/// covers by telling the truth per pass.
#[test]
fn the_boundary_door_refuses_a_switch_into_an_illegal_periodic_state() {
    use holon_render::sim::BoundaryRefusal;
    let pt = generate_pair_table(HYDROGEN, HYDROGEN, 96);
    let reach = {
        let mut s = periodic_scene(100.0);
        assert_eq!(load_pair_table(&mut s, &pt, Host::Native), TABLE_OK);
        s.reset(4);
        s.legality_radius()
    };
    assert!(reach > 1.0, "the probe has no reach to speak of");

    // A box that cannot honour the reach: WALLS is fine, and the switch to periodic is not.
    let mut tight = periodic_scene(2.0 * reach * UNDER);
    tight.boundary = Boundary::Walls;
    assert_eq!(load_pair_table(&mut tight, &pt, Host::Native), TABLE_OK);
    tight.reset(4);
    assert!(tight.pbc_ok(), "a walled box is vacuously legal");
    match tight.set_boundary(Boundary::Periodic) {
        Err(BoundaryRefusal::BreaksPeriodicImages { reach: r, half_edge }) => {
            assert!((r - reach).abs() < 1.0e-12, "the refusal quoted the wrong reach");
            assert!(r > half_edge, "the refusal fired with the reach INSIDE the half-edge");
        }
        Ok(()) => panic!("the door admitted a switch into an illegal periodic state"),
    }
    // AND IT LEFT THE SCENE ALONE. A door that refuses but mutates has not refused.
    assert_eq!(tight.boundary, Boundary::Walls);

    // A box that can honour it is admitted, so the door is a criterion and not a wall.
    let mut wide = periodic_scene(2.0 * reach * OVER);
    wide.boundary = Boundary::Walls;
    assert_eq!(load_pair_table(&mut wide, &pt, Host::Native), TABLE_OK);
    wide.reset(4);
    assert!(wide.set_boundary(Boundary::Periodic).is_ok());
    assert_eq!(wide.boundary, Boundary::Periodic);
    assert!(wide.pbc_ok(), "the door admitted a state its own guard calls illegal");

    // Leaving a wrapping boundary is ALWAYS admitted: a door that refused the exit would
    // trap a scene in the state it was complaining about.
    let mut trapped = periodic_scene(2.0 * reach * UNDER);
    assert_eq!(load_pair_table(&mut trapped, &pt, Host::Native), TABLE_OK);
    trapped.reset(4);
    assert!(!trapped.pbc_ok(), "the field poke should have produced an illegal scene");
    assert!(trapped.set_boundary(Boundary::Walls).is_ok());
    assert!(trapped.set_boundary(Boundary::Open).is_ok());
}

#[test]
fn the_abi_mode_two_route_goes_through_the_door() {
    // The ABI is the route a host actually takes, and it assigned unconditionally before
    // this. Checked through the exported symbol rather than through `Sim`, because a door
    // the library honours and the ABI walks around is not a door.
    use holon_render::{
        holon_box_scale, holon_reset, holon_set_boundary, holon_table_generate,
        BOUNDARY_REFUSED, TABLE_OK,
    };
    holon_reset(4);
    holon_set_boundary(0);
    // A CURVE FIRST. The ABI's scene starts with the species registered and no table loaded
    // — `PairBank::hydrogen_seeded` seeds `z[0] = 1` and nothing else — so its reach is a
    // legitimate ZERO and mode 2 is legal at any box size. The first version of this test
    // asserted a refusal on that scene and was wrong to: a scene with no interactions in it
    // has nothing the minimum image can get wrong.
    assert_eq!(holon_table_generate(1.0, 10.0, 32), TABLE_OK, "the ABI's curve door works");
    // Shrunk through the ABI's own box door, so the sequence is one a host could perform:
    // the scale door guards only WRAPPING boundaries, so shrinking a walled box is admitted
    // and the illegal state is then one mode-2 call away. That call has to refuse.
    for _ in 0..3 {
        holon_box_scale(0.5);
    }
    let refused = holon_set_boundary(2);
    let ok_open = holon_set_boundary(1);
    let ok_walls = holon_set_boundary(0);
    assert_eq!(ok_open, 0, "leaving a wrapping boundary must always be admitted");
    assert_eq!(ok_walls, 0, "switching to walls must always be admitted");
    assert!(
        refused >= BOUNDARY_REFUSED,
        "ABI mode 2 admitted a box shrunk to 1/8 its default against a 10-bohr curve; \
         it returned {refused}"
    );
}
