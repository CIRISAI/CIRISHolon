//! ROUTE B'S CONTRACT: the frame buffer carries what the renderer used to read.
//!
//! The refactor's claim is that `scene.rs` and `bonds.rs` see exactly what they saw when
//! they borrowed a `&Sim` — and that claim is worth a test rather than a compile. A
//! producer that dropped a field, or bounded an index differently, would leave the crate
//! compiling and the picture wrong, which is the class of defect no type checker sees.
//!
//! These run HEADLESS, which is the point: `world.rs` and `frame.rs` are deliberately not
//! behind `feature = "render"`, so gate 15 — the gate that could not see the render-only
//! modules and therefore missed a two-day compile break — DOES cover Route B's contract.

use holon_render_3d::frame::FrameBuffer;
use holon_render_3d::world::AtomWorld;

/// A world with a real curve and a settled scene.
fn world(n: usize) -> AtomWorld {
    let mut w = AtomWorld::new(n);
    for _ in 0..200 {
        w.advance(1.0 / 60.0);
    }
    w
}

/// A world that has actually FORMED BONDS, which the plain one has not.
///
/// The opener starts at 5,486 K and the scene carries zero bonded pairs for at least the
/// first thousand steps — measured, after two of these tests were found passing over an
/// EMPTY bond list. Bonds cost energy, so the thermostat has to take it out: cooled toward
/// 50 K the scene reaches ~717 K and four bonds by 10,000 steps.
///
/// `bonded_count` below is asserted by every test that walks the bond list. A test whose
/// subject is absent does not pass — it has not run, and saying so is the difference
/// between a green suite and a checked one.
fn bonded_world() -> AtomWorld {
    let mut w = AtomWorld::new(12);
    w.sim.thermostat_on = true;
    w.sim.target_temperature = 50.0;
    while w.sim.steps < 10_000 {
        w.advance(1.0 / 60.0);
    }
    w
}

fn bonded_count(w: &AtomWorld) -> usize {
    w.sim.pairs[..w.sim.pair_count].iter().filter(|p| p.bonded).count()
}

#[test]
fn the_buffer_reproduces_the_sims_atoms_exactly() {
    let w = world(12);
    let mut f = FrameBuffer::default();
    w.fill_frame(&mut f);

    assert_eq!(f.atoms.len(), w.sim.n, "the buffer's length IS the live count");
    for i in 0..w.sim.n {
        let a = &w.sim.atoms[i];
        let b = &f.atoms[i];
        // BIT-for-bit: a producer that rounded or converted would move the picture by an
        // amount no reviewer could see and no gate could name.
        assert_eq!(b.x.to_bits(), a.x.to_bits(), "atom {i} x");
        assert_eq!(b.y.to_bits(), a.y.to_bits(), "atom {i} y");
        assert_eq!(b.z.to_bits(), a.z.to_bits(), "atom {i} z");
        assert_eq!(b.radius.to_bits(), a.radius().to_bits(), "atom {i} radius");
        assert_eq!(b.z_species, a.species.z, "atom {i} species");
    }
}

#[test]
fn the_buffer_carries_exactly_the_bonded_pairs_and_their_depth() {
    let w = bonded_world();
    assert!(
        bonded_count(&w) > 0,
        "PRECONDITION: this test walks the bond list, so a scene with no bonds means it \
         checked nothing. Two earlier versions passed exactly that way."
    );
    let mut f = FrameBuffer::default();
    w.fill_frame(&mut f);

    let s = &w.sim;
    let expected: Vec<(usize, usize)> = s.pairs[..s.pair_count]
        .iter()
        .filter(|p| p.bonded)
        .map(|p| (p.i, p.j))
        .collect();
    let got: Vec<(usize, usize)> = f.bonds.iter().map(|b| (b.i, b.j)).collect();
    assert_eq!(got, expected, "the bond list is the engine's own verdict, in its order");

    // The depth encoding is the renderer's only continuous quantity and it is computed by
    // the producer, which owns both halves of it. A renderer recomputing it would need the
    // active table, which is exactly the Sim access route B removes.
    let d_e = s.table().d_e.max(1e-12);
    for (k, p) in s.pairs[..s.pair_count].iter().filter(|p| p.bonded).enumerate() {
        let want = (-p.e_bond() / d_e).clamp(0.0, 1.0) as f32;
        assert_eq!(f.bonds[k].depth.to_bits(), want.to_bits(), "bond {k} depth");
    }
}

#[test]
fn every_bond_endpoint_indexes_a_carried_atom() {
    // The invariant the renderer relies on to draw a rod at all. `bonds.rs` skips a bond
    // whose endpoints are missing rather than guessing one; this asserts the producer
    // never asks it to.
    let w = bonded_world();
    assert!(bonded_count(&w) > 0, "PRECONDITION: no bonds means nothing was checked");
    let mut f = FrameBuffer::default();
    w.fill_frame(&mut f);
    for b in &f.bonds {
        assert!(b.i < f.atoms.len(), "bond names atom {} of {}", b.i, f.atoms.len());
        assert!(b.j < f.atoms.len(), "bond names atom {} of {}", b.j, f.atoms.len());
    }
}

#[test]
fn the_held_index_is_bounded_by_the_producer_not_the_renderer() {
    // `scene.rs` used to re-check `grabbed` against `n` itself. Two guards that can
    // disagree is one guard too many, so the producer bounds it and the renderer trusts
    // the buffer — which is only safe if the producer actually does it.
    let mut w = world(12);
    let n = w.sim.n;
    w.sim.grabbed = Some(n + 5);
    let mut f = FrameBuffer::default();
    w.fill_frame(&mut f);
    assert_eq!(f.grabbed, None, "an out-of-range held index must not reach the renderer");

    w.sim.grabbed = Some(0);
    w.fill_frame(&mut f);
    assert_eq!(f.grabbed, Some(0), "and a valid one must");
}

#[test]
fn a_refilled_buffer_carries_no_residue_from_the_frame_before() {
    // `fill_frame` reuses the caller's allocation, which is the point of taking `&mut`.
    // Reuse without clearing is how a shrinking scene keeps drawing atoms that are gone —
    // the buffer would report last frame's population as this frame's.
    // `AtomWorld::new(n)` selects a PRESET rather than an atom count — new(4) and new(12)
    // both open Quench16 — so the smaller scene is made with `reset`, which is the door
    // that actually takes a count. My first version of this test used `new(4)` and failed
    // on its own premise rather than on the code; the API said so and I had assumed.
    let big = world(12);
    let mut small = world(12);
    small.reset(4);
    let mut f = FrameBuffer::default();
    big.fill_frame(&mut f);
    let was = f.atoms.len();
    small.fill_frame(&mut f);
    assert!(
        was > f.atoms.len(),
        "the test needs the second scene to be smaller: {was} then {}",
        f.atoms.len()
    );
    assert_eq!(f.atoms.len(), small.sim.n, "stale atoms survived a refill");
    assert!(
        f.bonds.iter().all(|b| b.i < f.atoms.len() && b.j < f.atoms.len()),
        "stale bonds survived a refill and now name atoms that do not exist"
    );
}
