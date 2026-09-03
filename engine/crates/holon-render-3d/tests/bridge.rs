//! THE WASM BOUNDARY'S CONTRACT: a frame arrives whole or not at all.
//!
//! The workbench's producer is JavaScript, and JavaScript is where the off-by-one lives.
//! A producer that skips an atom, declares nine and sends eight, or interleaves two frames
//! would otherwise draw a scene that is NEARLY right — and a nearly-right scene makes
//! physics claims nobody made: a missing atom reads as an atom that left, a truncated bond
//! list reads as a bond that broke. So the boundary declares, checks and refuses, and
//! these are the tests of the refusing.
//!
//! SHARED STATE, AND THE LOCK. The staging area is one global for the page's whole life,
//! which is right for a page and wrong for a test binary: cargo runs tests in parallel
//! threads of ONE process, so two tests would be two producers filling one frame. Every
//! test here takes `BOUNDARY` first and resets. Without it these tests would fail
//! intermittently, which is worse than failing.

use std::sync::{Mutex, MutexGuard};

use holon_render_3d::bridge::{self, OP_GRAB, OP_MOVE_ANCHOR, OP_RELEASE};
use holon_render_3d::hand::HandOp;

static BOUNDARY: Mutex<()> = Mutex::new(());

fn exclusive() -> MutexGuard<'static, ()> {
    let g = BOUNDARY.lock().unwrap_or_else(|e| e.into_inner());
    bridge::reset_for_test();
    g
}

/// Push a well-formed frame of `n` atoms and no bonds.
fn push_atoms(n: usize, frame: u64) -> bool {
    bridge::frame_begin(n, 0);
    for i in 0..n {
        bridge::frame_atom(i, i as f64, 0.0, 0.0, 0.5, 1);
    }
    bridge::frame_commit(None, (0.0, 0.0, 0.0), frame)
}

#[test]
fn a_whole_frame_arrives_whole_and_arrives_once() {
    let _g = exclusive();
    assert!(push_atoms(3, 42), "precondition: the frame was accepted");

    let pulled = bridge::take_committed().expect("a committed frame is there to take");
    assert_eq!(pulled.atoms.len(), 3);
    assert_eq!(pulled.frame, 42);
    assert_eq!(pulled.atoms[2].x, 2.0, "in the order it was pushed");

    assert!(
        bridge::take_committed().is_none(),
        "taking empties — a frame handed out twice is drawn twice, and the second drawing \
         reports a frame number the engine has already left"
    );
    assert_eq!(bridge::refused_frames(), 0);
    assert_eq!(bridge::dropped_frames(), 0);
}

#[test]
fn an_atom_out_of_order_costs_the_whole_frame() {
    let _g = exclusive();
    bridge::frame_begin(3, 0);
    bridge::frame_atom(0, 0.0, 0.0, 0.0, 0.5, 1);
    // The skip: index 2 where index 1 was due. A JS loop with a stale counter does this.
    bridge::frame_atom(2, 2.0, 0.0, 0.0, 0.5, 1);
    bridge::frame_atom(2, 2.0, 0.0, 0.0, 0.5, 1);
    assert!(
        !bridge::frame_commit(None, (0.0, 0.0, 0.0), 1),
        "the frame is refused"
    );
    assert!(
        bridge::take_committed().is_none(),
        "and nothing is offered to the renderer — not a partial scene, not the old one \
         relabelled"
    );
    assert_eq!(bridge::refused_frames(), 1);
}

#[test]
fn a_short_frame_is_refused_rather_than_drawn_small() {
    let _g = exclusive();
    bridge::frame_begin(3, 0);
    bridge::frame_atom(0, 0.0, 0.0, 0.0, 0.5, 1);
    bridge::frame_atom(1, 1.0, 0.0, 0.0, 0.5, 1);
    assert!(
        !bridge::frame_commit(None, (0.0, 0.0, 0.0), 1),
        "two atoms arrived where three were declared"
    );
    assert!(bridge::take_committed().is_none());
    assert_eq!(bridge::refused_frames(), 1);
}

#[test]
fn a_bond_to_an_atom_that_is_not_in_the_frame_is_refused() {
    let _g = exclusive();
    bridge::frame_begin(2, 1);
    bridge::frame_atom(0, 0.0, 0.0, 0.0, 0.5, 1);
    bridge::frame_atom(1, 1.4, 0.0, 0.0, 0.5, 1);
    // Endpoint 5 in a two-atom frame. Clamping it would draw a real rod to the wrong
    // atom, which is a bond claim the engine never made.
    bridge::frame_bond(0, 0, 5, 1.0);
    assert!(!bridge::frame_commit(None, (0.0, 0.0, 0.0), 1));
    assert!(bridge::take_committed().is_none());
    assert_eq!(bridge::refused_frames(), 1);
}

#[test]
fn a_frame_committed_over_an_undrawn_one_wins_and_the_loss_is_counted() {
    let _g = exclusive();
    assert!(push_atoms(2, 10), "precondition: first frame accepted");
    assert!(push_atoms(4, 11), "precondition: second frame accepted");

    assert_eq!(
        bridge::dropped_frames(),
        1,
        "the first was never drawn, and that is a fact worth having: a stale picture and \
         a stopped producer are different faults"
    );
    let pulled = bridge::take_committed().expect("a frame is waiting");
    assert_eq!(
        pulled.frame, 11,
        "the NEW frame wins — a queue would let the picture fall arbitrarily far behind \
         the engine while reporting itself live"
    );
    assert_eq!(pulled.atoms.len(), 4);
}

#[test]
fn the_held_index_is_bounded_by_the_frame_that_carries_it() {
    let _g = exclusive();
    bridge::frame_begin(2, 0);
    bridge::frame_atom(0, 0.0, 0.0, 0.0, 0.5, 1);
    bridge::frame_atom(1, 1.4, 0.0, 0.0, 0.5, 1);
    assert!(bridge::frame_commit(Some(7), (1.0, 2.0, 3.0), 1));
    let pulled = bridge::take_committed().expect("accepted");
    assert_eq!(
        pulled.grabbed, None,
        "an out-of-range held index is dropped, the same way the native producer bounds \
         it against Sim::n"
    );
    assert_eq!(pulled.anchor, (1.0, 2.0, 3.0), "the anchor still crosses");
}

#[test]
fn the_hand_crosses_in_order_and_clears_once() {
    let _g = exclusive();
    bridge::offer_ops(vec![
        HandOp::Grab(2),
        HandOp::MoveAnchor(1.5, -2.5, 3.5),
        HandOp::Release,
    ]);
    assert_eq!(bridge::ops_len(), 3);

    assert_eq!(bridge::op_kind(0), Some(OP_GRAB));
    assert_eq!(bridge::op_arg(0, 0), 2.0);
    assert_eq!(bridge::op_kind(1), Some(OP_MOVE_ANCHOR));
    assert_eq!(bridge::op_arg(1, 0), 1.5);
    assert_eq!(bridge::op_arg(1, 1), -2.5);
    assert_eq!(bridge::op_arg(1, 2), 3.5);
    assert_eq!(bridge::op_kind(2), Some(OP_RELEASE));
    assert_eq!(bridge::op_kind(3), None, "there is no fourth op");

    bridge::ops_clear();
    assert_eq!(
        bridge::ops_len(),
        0,
        "a caller that reads and forgets to clear replays the gesture on every frame, \
         which posts the drag's work again on every frame"
    );
}

#[test]
fn an_argument_an_op_does_not_have_is_nan_and_not_zero() {
    let _g = exclusive();
    bridge::offer_ops(vec![HandOp::Grab(0), HandOp::Release]);
    // Zero is a legal coordinate AND a legal atom index. A caller reading the third
    // argument of a grab must not be handed a plausible number: it would move the anchor
    // to the origin, and the origin is a corner of the box.
    assert!(bridge::op_arg(0, 2).is_nan(), "grab has no third argument");
    assert!(bridge::op_arg(1, 0).is_nan(), "release has no arguments");
    assert!(bridge::op_arg(9, 0).is_nan(), "and there is no op 9");
}

#[test]
fn an_empty_hand_does_not_touch_the_boundary() {
    let _g = exclusive();
    bridge::offer_ops(vec![HandOp::Release]);
    bridge::offer_ops(Vec::new());
    assert_eq!(
        bridge::ops_len(),
        1,
        "an empty push is a no-op, not a clear — the frames between gestures must not \
         eat an op the sink has not carried yet"
    );
}
