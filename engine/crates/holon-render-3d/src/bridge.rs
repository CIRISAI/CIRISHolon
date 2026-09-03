//! THE WORKBENCH'S PRODUCER AND SINK, on this side of the wasm boundary.
//!
//! Route B, increment 3. `frame.rs` and `hand.rs` gave the renderer its two waists;
//! atoms3d fills and drains them from the `AtomWorld` it owns. The workbench has no
//! `AtomWorld` and must not acquire one — its `Sim` lives in the committed cdylib, in a
//! DIFFERENT wasm instance, and a second copy here is the disqualifying outcome Route B
//! exists to prevent. So the two waists are reached from JS instead, and this module is
//! the staging area they are reached through:
//!
//! ```text
//!   cdylib.step()  →  JS reads it  →  frame_atom/frame_bond/frame_commit  →  STAGE
//!   STAGE  →  pull_frame  →  FrameBuffer  →  scene.rs / bonds.rs        (the picture)
//!   pick.rs  →  HandIntent  →  push_hand  →  STAGE
//!   STAGE  →  op_kind/op_arg  →  JS  →  cdylib's holon_grab / …          (the ledger)
//! ```
//!
//! WHY A STAGING AREA AND NOT DIRECT EXPORTS. Bevy's `World` is not reachable from an
//! exported function — an export has no `&mut World` to hand it. A global that two
//! systems swap with the resources is the standard shape, and it buys something beyond
//! necessity: the boundary becomes a place where a malformed frame can be REFUSED, which
//! a direct write into the resource would have no opportunity to do.
//!
//! WHY EVERYTHING HERE IS SAFE CODE. `lib.rs` carries `#![forbid(unsafe_code)]`, and that
//! is not negotiable for a crate that draws a certified core. `forbid` rejects
//! `#[no_mangle]` outright, so the EXPORTS live in `src/main.rs` — the artifact's own
//! crate, which carries no such attribute — and they are thin: every one of them calls a
//! safe function here and does nothing else. The ABI is scalars only, no pointers and no
//! buffers, so nothing in the chain needs `unsafe` and no `wasm-bindgen` dependency is
//! added to reach across. It is the same raw `extern "C"` shape the cdylib already uses,
//! which is the house style rather than a coincidence.
//!
//! WHAT IS REFUSED, AND WHY REFUSAL IS THE POINT. A producer that skips an atom, declares
//! nine and sends eight, or interleaves two frames would otherwise draw a scene that is
//! *nearly* right — a missing atom reads as an atom that left, and a truncated bond list
//! reads as a bond that broke. Both are physics claims, made by a JS bug. So a frame is
//! declared before it is filled, each element must arrive at the index the count implies,
//! and a commit whose counts do not match discards the WHOLE frame rather than showing
//! part of one. The refusals are counted, because a silent refusal and a producer that
//! never ran look identical from the page.

use std::sync::Mutex;

use crate::frame::{FrameAtom, FrameBond, FrameBuffer};
use crate::hand::{HandIntent, HandOp};

/// Op kind codes on the wire. Deliberately not the enum's discriminants: those are an
/// implementation detail that a reordering would change silently, and JS would keep
/// grabbing when it meant to release.
pub const OP_GRAB: u32 = 0;
pub const OP_MOVE_ANCHOR: u32 = 1;
pub const OP_RELEASE: u32 = 2;

/// What the boundary is holding between frames.
#[derive(Default)]
struct Stage {
    /// The frame being filled, and the counts it declared.
    building: FrameBuffer,
    want_atoms: usize,
    want_bonds: usize,
    /// Set once the frame under construction has been refused; a refused frame is filled
    /// out to the end and then discarded at commit, rather than aborted midway, so the
    /// producer's own loop never has to know.
    spoiled: bool,
    /// A committed frame waiting for `pull_frame`.
    ready: Option<FrameBuffer>,
    /// Ops waiting for JS to carry them to the cdylib.
    ops: Vec<HandOp>,
    /// Frames committed on top of one nobody drew. Not an error — a producer stepping
    /// faster than the renderer draws is the normal case — but a number worth having,
    /// because "the picture is stale" and "the producer stopped" are different faults.
    dropped: u64,
    /// Frames discarded at commit for not matching what they declared.
    refused: u64,
}

impl Stage {
    const fn new() -> Self {
        Self {
            building: FrameBuffer {
                atoms: Vec::new(),
                bonds: Vec::new(),
                grabbed: None,
                anchor: (0.0, 0.0, 0.0),
                frame: 0,
            },
            want_atoms: 0,
            want_bonds: 0,
            spoiled: false,
            ready: None,
            ops: Vec::new(),
            dropped: 0,
            refused: 0,
        }
    }
}

static STAGE: Mutex<Stage> = Mutex::new(Stage::new());

/// Run `f` on the staging area.
///
/// A poisoned mutex is treated as live state rather than a panic: the only thing that can
/// poison it is a panic inside one of these short functions, and a page that stops drawing
/// because a previous frame was malformed is a worse failure than one that draws the next
/// frame. `PoisonError::into_inner` is the safe accessor for exactly this.
fn with_stage<T>(f: impl FnOnce(&mut Stage) -> T) -> T {
    let mut guard = STAGE.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut guard)
}

// ── the producer's side: JS fills a frame ────────────────────────────────────────

/// Declare a frame: this many atoms, this many bonds, filled from index 0 upward.
///
/// Declaring first is what makes a short frame detectable. A producer that simply pushed
/// until it stopped would deliver a truncated scene indistinguishable from a smaller one.
pub fn frame_begin(n_atoms: usize, n_bonds: usize) {
    with_stage(|s| {
        s.building.atoms.clear();
        s.building.bonds.clear();
        s.building.atoms.reserve(n_atoms);
        s.building.bonds.reserve(n_bonds);
        s.want_atoms = n_atoms;
        s.want_bonds = n_bonds;
        s.spoiled = false;
    });
}

/// Place one atom. `i` must be the index this atom will occupy.
///
/// The index is passed and CHECKED rather than implied by call order, because the two
/// disagree exactly when the producer has a bug, and an off-by-one in a JS loop is
/// otherwise invisible: the scene simply has the wrong atom missing.
pub fn frame_atom(i: usize, x: f64, y: f64, z: f64, radius: f64, z_species: u32) {
    with_stage(|s| {
        if s.spoiled {
            return;
        }
        if i != s.building.atoms.len() || i >= s.want_atoms {
            s.spoiled = true;
            return;
        }
        s.building.atoms.push(FrameAtom {
            x,
            y,
            z,
            radius,
            z_species,
        });
    });
}

/// Place one bonded pair. `k` must be the index this bond will occupy.
pub fn frame_bond(k: usize, i: usize, j: usize, depth: f32) {
    with_stage(|s| {
        if s.spoiled {
            return;
        }
        if k != s.building.bonds.len() || k >= s.want_bonds {
            s.spoiled = true;
            return;
        }
        // An endpoint outside the declared atom count would index nothing. Caught here
        // rather than clamped: a bond to an atom that is not in the frame is a claim
        // about the scene, and a clamp would draw it to the wrong atom.
        if i >= s.want_atoms || j >= s.want_atoms {
            s.spoiled = true;
            return;
        }
        s.building.bonds.push(FrameBond { i, j, depth });
    });
}

/// Finish the frame and offer it to the renderer.
///
/// Returns `true` when the frame was accepted. A frame whose counts do not match what it
/// declared is discarded ENTIRELY — not truncated, not padded — because a partial scene
/// makes a physics claim its producer never made.
pub fn frame_commit(grabbed: Option<usize>, anchor: (f64, f64, f64), frame: u64) -> bool {
    with_stage(|s| {
        let complete =
            s.building.atoms.len() == s.want_atoms && s.building.bonds.len() == s.want_bonds;
        if s.spoiled || !complete {
            s.refused += 1;
            s.building.atoms.clear();
            s.building.bonds.clear();
            s.spoiled = false;
            return false;
        }
        // The held index is bounded against THIS frame's atoms, the same way the native
        // producer bounds it against `Sim::n`.
        s.building.grabbed = grabbed.filter(|&i| i < s.building.atoms.len());
        s.building.anchor = anchor;
        s.building.frame = frame;
        if s.ready.is_some() {
            s.dropped += 1;
        }
        // The NEW frame wins. A queue would let the picture fall arbitrarily far behind
        // the engine while reporting itself live.
        s.ready = Some(std::mem::take(&mut s.building));
        s.want_atoms = 0;
        s.want_bonds = 0;
        true
    })
}

// ── the sink's side: JS carries the hand's ops to the cdylib ─────────────────────

/// How many ops are waiting.
pub fn ops_len() -> usize {
    with_stage(|s| s.ops.len())
}

/// The kind code of op `k`, or `None` if there is no such op.
pub fn op_kind(k: usize) -> Option<u32> {
    with_stage(|s| {
        s.ops.get(k).map(|op| match op {
            HandOp::Grab(_) => OP_GRAB,
            HandOp::MoveAnchor(..) => OP_MOVE_ANCHOR,
            HandOp::Release => OP_RELEASE,
        })
    })
}

/// Argument `which` of op `k`.
///
/// NaN for an argument the op does not have — never 0.0. Zero is a legal coordinate and a
/// legal atom index, so a caller reading the third argument of a grab would be handed a
/// plausible number and would move the anchor to the origin. NaN is the one value that
/// cannot be mistaken for an answer.
pub fn op_arg(k: usize, which: usize) -> f64 {
    with_stage(|s| match (s.ops.get(k), which) {
        (Some(HandOp::Grab(i)), 0) => *i as f64,
        (Some(HandOp::MoveAnchor(x, _, _)), 0) => *x,
        (Some(HandOp::MoveAnchor(_, y, _)), 1) => *y,
        (Some(HandOp::MoveAnchor(_, _, z)), 2) => *z,
        _ => f64::NAN,
    })
}

/// Drop the ops the caller has now carried across.
///
/// SEPARATE from reading them, and that is the one place this boundary differs from
/// `HandIntent::take`. JS cannot take a `Vec`; it reads op 0, then op 1, then clears. So
/// the clear is its own call and the hazard moves: a caller that reads and forgets to
/// clear replays the gesture every frame, which posts the drag's work again and again.
/// The counter below is how that is seen from the outside.
pub fn ops_clear() {
    with_stage(|s| s.ops.clear());
}

/// Frames committed on top of an undrawn one.
pub fn dropped_frames() -> u64 {
    with_stage(|s| s.dropped)
}

/// Frames discarded at commit for not matching what they declared.
pub fn refused_frames() -> u64 {
    with_stage(|s| s.refused)
}

// ── the two systems that swap the staging area with the resources ────────────────

/// Take the committed frame, if there is one, and give it to the renderer.
///
/// When there is none, the previous frame stays on screen UNCHANGED — including its
/// `frame` number, which is what lets a consumer tell a paused engine from a stalled
/// producer. Nothing here invents a frame.
pub fn pull_frame(mut buffer: bevy::prelude::ResMut<FrameBuffer>) {
    if let Some(next) = take_committed() {
        *buffer = next;
    }
}

/// The committed frame, if any, removed from the boundary.
///
/// Split out of the system so the boundary's behaviour can be checked without a `World`.
/// It empties, for the same reason `HandIntent::take` empties: a frame handed out twice
/// would be drawn twice and the second drawing would report a frame number the engine had
/// already left.
pub fn take_committed() -> Option<FrameBuffer> {
    with_stage(|s| s.ready.take())
}

/// Move everything the finger recorded this frame out to the boundary.
///
/// `HandIntent::take` empties, so an op leaves the intent exactly once; it then sits in
/// the staging area until JS clears it. Between those two the op exists in one place at a
/// time, which is the property the ledger needs.
pub fn push_hand(mut hand: bevy::prelude::ResMut<HandIntent>) {
    offer_ops(hand.take());
}

/// Put ops on the boundary for JS to carry across. Split out of the system for the same
/// reason as [`take_committed`].
pub fn offer_ops(ops: Vec<HandOp>) {
    if ops.is_empty() {
        return;
    }
    with_stage(|s| s.ops.extend(ops));
}

/// Reset the boundary. Tests only — the page has one of these for its whole life.
#[doc(hidden)]
pub fn reset_for_test() {
    with_stage(|s| *s = Stage::new());
}
