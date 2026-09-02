//! ROUTE B'S OTHER CONTRACT: the hand's intent reaches the ledger exactly once.
//!
//! The frame buffer's tests ask whether the renderer still SEES what it saw. These ask
//! the harder half: whether the renderer still DOES what it did, now that it records an
//! op instead of calling a door.
//!
//! Why this needs tests and not a compile. Every op in the queue moves a ledger:
//! `move_anchor_3d` posts `dU` to `W_ext` and to the hand's receipt column, and `release`
//! subtracts the stored spring energy. Apply the queue twice and the work is posted twice
//! — `E - W_ext` drifts, and it drifts in the shape of an integrator bug, which is where
//! the search would go. Apply it out of order and a grab lands after a move, so the
//! anchor is placed on an atom nobody is holding and then the grab resets it, silently
//! swallowing a stretch. Neither failure changes a type.
//!
//! Headless, like the frame-buffer tests and for the same reason: `hand.rs` is not behind
//! `feature = "render"`, so gate 15 covers it.

use holon_render_3d::hand::{HandIntent, HandOp};
use holon_render_3d::world::AtomWorld;

/// A settled two-atom world — the opening scene, the one the hand exists for.
fn world() -> AtomWorld {
    let mut w = AtomWorld::new(2);
    for _ in 0..200 {
        w.advance(1.0 / 60.0);
    }
    w
}

/// Move the anchor a visible distance off the held atom, so the spring is genuinely
/// stretched and the work posted is genuinely nonzero.
fn stretched_anchor(w: &AtomWorld, i: usize) -> (f64, f64, f64) {
    let a = &w.sim.atoms[i];
    (a.x + 0.7, a.y, a.z)
}

#[test]
fn the_queue_keeps_the_order_the_gesture_had() {
    let mut hand = HandIntent::default();
    hand.grab(3);
    hand.move_anchor(1.0, 2.0, 3.0);
    hand.release();
    assert_eq!(
        hand.take(),
        vec![
            HandOp::Grab(3),
            HandOp::MoveAnchor(1.0, 2.0, 3.0),
            HandOp::Release
        ],
        "the sink applies ops in the order it receives them, so the queue must not reorder"
    );
}

#[test]
fn taking_empties_the_queue() {
    let mut hand = HandIntent::default();
    hand.grab(0);
    hand.move_anchor(1.0, 2.0, 3.0);
    assert_eq!(hand.take().len(), 2, "precondition: the ops were recorded");
    assert!(
        hand.take().is_empty(),
        "a second take must return nothing — a `take` that cloned instead of draining \
         would let a sink apply one gesture on every frame after it"
    );
    assert_eq!(hand.issued(), 2, "the counter records the whole life, not the queue");
}

#[test]
fn a_drag_posts_its_work_once_and_only_once() {
    let mut w = world();
    let mut hand = HandIntent::default();
    let (x, y, z) = stretched_anchor(&w, 0);

    hand.grab(0);
    hand.move_anchor(x, y, z);
    assert_eq!(hand.apply_to(&mut w.sim), 2, "both ops reached the Sim");

    // The identity the ledger rests on: the anchor started ON the atom (grab puts it
    // there, at zero extension), so the work the hand has done is exactly the energy now
    // stored in the spring. Not approximately — the same float, computed once.
    assert!(
        w.sim.e_spring > 1.0e-6,
        "precondition: the move actually stretched the spring (got {})",
        w.sim.e_spring
    );
    assert!(
        (w.sim.w_ext - w.sim.e_spring).abs() < 1.0e-12,
        "W_ext {} should be the stored spring energy {}",
        w.sim.w_ext,
        w.sim.e_spring
    );
    assert!(
        (w.sim.work.hand - w.sim.e_spring).abs() < 1.0e-12,
        "the hand's own receipt column must carry the same increment"
    );

    // Now the thing the type is for: applying an emptied queue moves nothing.
    let w_ext_before = w.sim.w_ext;
    let hand_before = w.sim.work.hand;
    assert_eq!(hand.apply_to(&mut w.sim), 0, "nothing left to apply");
    assert_eq!(
        w.sim.w_ext, w_ext_before,
        "a second apply posted work a second time"
    );
    assert_eq!(w.sim.work.hand, hand_before);
}

#[test]
fn release_hands_the_stored_energy_back() {
    let mut w = world();
    let mut hand = HandIntent::default();
    let (x, y, z) = stretched_anchor(&w, 0);

    hand.grab(0);
    hand.move_anchor(x, y, z);
    hand.apply_to(&mut w.sim);
    assert!(w.sim.w_ext > 1.0e-6, "precondition: work was posted");
    assert_eq!(w.sim.grabbed, Some(0), "precondition: the atom is held");

    hand.release();
    hand.apply_to(&mut w.sim);
    assert_eq!(w.sim.grabbed, None, "the hand let go");
    assert!(
        w.sim.w_ext.abs() < 1.0e-12,
        "the spring energy leaves with the hand, so W_ext returns to zero (got {})",
        w.sim.w_ext
    );
}

#[test]
fn order_is_load_bearing_and_the_ledger_says_so() {
    // grab-then-move stretches the spring; move-then-grab does not, because the move
    // finds nothing held and the grab then re-seats the anchor on the atom. Same three
    // numbers, same two ops, different history — and the ledger reads them differently.
    // An unordered container would make these two equal, and this test is the reason the
    // queue is a Vec.
    let (x, y, z) = {
        let w = world();
        stretched_anchor(&w, 0)
    };

    let mut forward = world();
    let mut hand = HandIntent::default();
    hand.grab(0);
    hand.move_anchor(x, y, z);
    hand.apply_to(&mut forward.sim);

    let mut reversed = world();
    let mut hand = HandIntent::default();
    hand.move_anchor(x, y, z);
    hand.grab(0);
    hand.apply_to(&mut reversed.sim);

    assert!(
        forward.sim.w_ext > 1.0e-6,
        "precondition: the forward order posts work"
    );
    assert!(
        reversed.sim.w_ext.abs() < 1.0e-12,
        "the reversed order posts nothing — the move had nothing to move (got {})",
        reversed.sim.w_ext
    );
    assert_ne!(forward.sim.w_ext, reversed.sim.w_ext);
}

#[test]
fn a_stale_grab_is_refused_and_the_refusal_is_counted() {
    let mut w = world();
    let stale = w.sim.n + 5;
    assert!(stale >= w.sim.n, "precondition: the index is out of the scene");

    let mut hand = HandIntent::default();
    hand.grab(stale);
    assert_eq!(hand.apply_to(&mut w.sim), 0, "nothing reached the Sim");
    assert_eq!(w.sim.grabbed, None, "and nothing was picked up");
    assert_eq!(
        hand.refused(),
        1,
        "the refusal is counted — a silently dropped grab looks exactly like a user \
         missing the atom, and those want different fixes"
    );
    assert_eq!(hand.issued(), 1, "issued counts what was asked for, not what landed");
}

/// The interaction and drawing layers hold no `Sim`.
///
/// Mechanical, over the source, because that is the only form this property has: it is a
/// statement about what the code is ALLOWED to reach for, and a passing runtime test
/// cannot see a reach that has not happened yet.
///
/// SCOPE, stated because the exclusions are deliberate and not oversights:
///   * `render.rs` is EXCLUDED. It is atoms3d's producer and sink — it is supposed to
///     hold that page's `AtomWorld`, fill the buffer from it and drain the intent into
///     it. The workbench replaces exactly those two systems.
///   * `hud.rs` is EXCLUDED and OWED. It still reads numbers straight off the `Sim`, and
///     retiring it from the workbench page is the next increment of Route B. Listing it
///     here as an exclusion is the debt, in the file that would otherwise hide it.
///   * This is NOT R7. R7 is page-scoped — "the workbench's drawn scene is fed only by
///     the cdylib's buffer" — and lives in the page's own gate. This is the crate-side
///     precondition that makes R7 statable at all.
#[test]
fn the_drawing_and_interaction_layers_name_no_sim() {
    for (name, src) in [
        ("pick.rs", include_str!("../src/pick.rs")),
        ("scene.rs", include_str!("../src/scene.rs")),
        ("bonds.rs", include_str!("../src/bonds.rs")),
    ] {
        // Comments are where the reasons live, and a rule that forbade the WORD would
        // forbid explaining itself. Strip them, then read the code.
        let code: String = src
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for forbidden in ["AtomWorld", "world.sim", ".sim."] {
            assert!(
                !code.contains(forbidden),
                "{name} names `{forbidden}` outside a comment — Route B's waist is that \
                 the drawing and interaction layers read a FrameBuffer and write a \
                 HandIntent, and nothing else"
            );
        }
    }
}
