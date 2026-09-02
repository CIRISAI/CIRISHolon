//! THE HAND INTENT — the one thing the interaction layer produces.
//!
//! Route B's second waist, and the mirror of [`crate::frame::FrameBuffer`]. The buffer
//! carries the engine's state INTO the drawing layer; this carries the finger's intent
//! OUT of it. Between them the renderer touches no `Sim` at all, which is what makes
//! "the workbench has exactly one Sim" a property of the wiring rather than a promise
//! about discipline.
//!
//! WHY A RECORD AND NOT A CALL. `pick.rs` used to call `Sim::grab` and
//! `Sim::move_anchor_3d` directly. On atoms3d that is correct — the page owns its `Sim`
//! and the renderer is holding it. On the workbench it is exactly the disqualifying
//! outcome: the drawn `Sim` would be Bevy's and the instrumented one would be the
//! cdylib's, and the hand would move the copy nobody is reading. So the picker now
//! computes geometry and RECORDS what it wants done, and a sink applies it:
//!
//!   * atoms3d's sink is [`HandIntent::apply_to`], one system in `render.rs`, straight
//!     into the `Sim` that page owns.
//!   * the workbench's sink is JS, calling `holon_grab` / `holon_move_anchor_3d` /
//!     `holon_release` on the committed cdylib — the SAME three doors, named in the same
//!     order, on the only `Sim` that page has.
//!
//! The ledger is why this matters more than tidiness. `move_anchor_3d` posts `dU` to
//! `W_ext` and to the hand's own receipt column, and `release` subtracts the stored
//! spring energy. An op applied twice posts the work twice; an op dropped posts none of
//! it and leaves `E - W_ext` visibly broken. So the type owns both halves of "exactly
//! once": recording appends, and [`HandIntent::take`] is the only way to read ops out
//! and it empties the queue.
//!
//! # What is NOT here
//!
//! No energy, no force, no work. This module computes a point and an index. Every number
//! that reaches a ledger is computed inside `Sim`, from these arguments, exactly as it
//! was when `pick.rs` called it directly — the call site moved, the accounting did not.
//!
//! # The index
//!
//! [`HandOp::Grab`] carries an index into the FRAME BUFFER's atom list, because that is
//! what the picker had in hand and what the user actually clicked on. The producer owns
//! the mapping back: today both producers fill the buffer in `Sim` index order, so the
//! two indices are the same number, and `apply_to` re-checks the bound anyway because a
//! buffer is by construction one frame older than the `Sim` it is applied to and the
//! scene can shrink in between.

use bevy::ecs::resource::Resource;
use holon_render::sim::Sim;

/// One thing the hand wants done, in the vocabulary of the three doors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HandOp {
    /// Take hold of the atom at this index in the frame buffer.
    Grab(usize),
    /// Move the spring's far end to this point, in simulation coordinates (bohr).
    MoveAnchor(f64, f64, f64),
    /// Let go.
    Release,
}

/// What the finger did this frame, waiting for a sink.
///
/// Ordered, because grab-then-move and move-then-grab are different histories: the first
/// drags an atom, the second moves an anchor that is holding nothing and then picks up
/// whatever is under it with the spring already stretched — which would inject the
/// stretch as unposted energy. A queue keeps the order the gesture had.
#[derive(Resource, Clone, Debug, Default)]
pub struct HandIntent {
    ops: Vec<HandOp>,
    issued: u64,
    refused: u64,
}

impl HandIntent {
    /// Record an op.
    pub fn record(&mut self, op: HandOp) {
        self.ops.push(op);
        self.issued += 1;
    }

    /// Record a grab of the frame-buffer atom at `i`.
    pub fn grab(&mut self, i: usize) {
        self.record(HandOp::Grab(i));
    }

    /// Record a move of the anchor, in simulation coordinates.
    pub fn move_anchor(&mut self, x: f64, y: f64, z: f64) {
        self.record(HandOp::MoveAnchor(x, y, z));
    }

    /// Record a release.
    pub fn release(&mut self) {
        self.record(HandOp::Release);
    }

    /// Read the queue without consuming it. For a sink that has to translate rather than
    /// apply — the workbench's JS bridge is the case — but NOT a way to apply twice: the
    /// applying sink still has to [`take`](Self::take).
    pub fn peek(&self) -> &[HandOp] {
        &self.ops
    }

    /// Take everything recorded, leaving the queue empty.
    ///
    /// The ONLY way to get ops out for application. A `&self` getter plus a separate
    /// clear would let a sink apply the queue and then fail to clear it, and the symptom
    /// would be work posted every frame from one gesture — a drift in `E - W_ext` that
    /// looks like an integrator bug and is not one.
    pub fn take(&mut self) -> Vec<HandOp> {
        std::mem::take(&mut self.ops)
    }

    /// Ops recorded over this intent's whole life.
    ///
    /// A counter and not a length, so a consumer can tell "nothing happened" from "it
    /// happened and something ate it". `issued - refused` is what reached a `Sim`.
    pub fn issued(&self) -> u64 {
        self.issued
    }

    /// Grabs refused because the index was not in the `Sim` when the op arrived.
    ///
    /// Counted rather than silently skipped. `Sim::grab` already returns without effect
    /// on an out-of-range index, so applying a stale grab is harmless — but a refusal
    /// that leaves no trace is the shape where a picker aiming at the wrong list looks
    /// exactly like a user missing the atom.
    pub fn refused(&self) -> u64 {
        self.refused
    }

    /// Apply everything recorded to `sim`, in order, and empty the queue.
    ///
    /// Returns how many ops actually reached the `Sim`. The three arms are the three
    /// doors and nothing else: this function contains no arithmetic, so the work posted
    /// by a drag is the work `Sim::move_anchor_3d` computes, unchanged and unduplicated.
    pub fn apply_to(&mut self, sim: &mut Sim) -> usize {
        let mut applied = 0usize;
        for op in self.take() {
            match op {
                HandOp::Grab(i) => {
                    if i >= sim.n {
                        self.refused += 1;
                        continue;
                    }
                    sim.grab(i);
                }
                HandOp::MoveAnchor(x, y, z) => sim.move_anchor_3d(x, y, z),
                HandOp::Release => sim.release(),
            }
            applied += 1;
        }
        applied
    }
}
