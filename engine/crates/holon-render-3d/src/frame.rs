//! THE FRAME BUFFER — the one thing the drawing layer consumes.
//!
//! Route B (FSD-W2 item 1, lead's ruling). The workbench page keeps ONE Sim, in the
//! committed cdylib, and Bevy becomes a pure renderer fed a per-frame buffer. The
//! disqualifying outcome is two Sims — one drawn, one instrumented — so the shape is ONE
//! CONSUMER WITH TWO PRODUCERS rather than two owners:
//!
//!   * `atoms3d` fills this from the `AtomWorld` it already owns. Unchanged behaviour, and
//!     that page is outside this campaign's scope — migrating it would be churn with no
//!     receipt, and freezing it would be worse.
//!   * the workbench fills it from the cdylib through JS. Exactly one Sim exists there and
//!     Bevy draws THAT one.
//!
//! Two producers is not two Sims: each page has one, and the renderer never owns either.
//!
//! WHY THIS TYPE AND NOT A `&Sim`. A borrow would make the renderer a Sim consumer again
//! and put the workbench's second copy back. The buffer is the narrow waist — it carries
//! what the drawing layer actually reads and nothing else, which is a short list because
//! `scene.rs` and `bonds.rs` were already only reading positions, species, bond pairs and
//! the held index.

use bevy::ecs::resource::Resource;

/// One atom, as the renderer needs it.
///
/// `radius` is carried rather than derived from `species` because the producer already
/// knows it — `Atom::radius()` on the native side, the palette on the page side — and a
/// renderer recomputing it would be a second statement of a rule it does not own.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameAtom {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub radius: f64,
    /// Atomic number. Zero means "no species", which the renderer draws as its fallback
    /// rather than guessing hydrogen — the atom viewer once read a Z COORDINATE as a
    /// species for months and drew every atom as hydrogen, so an explicit unknown is
    /// cheaper than a plausible default.
    pub z_species: u32,
}

/// One bonded pair, by index into `atoms`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameBond {
    pub i: usize,
    pub j: usize,
    /// How deep in its well this pair sits, 0..1 — the rod's thickness encodes it, so a
    /// pair grazing the tail draws as a hair and one at the bottom of the well draws as
    /// the full rod. No threshold decides what is "really" a bond; the energy does.
    ///
    /// CARRIED rather than derived, because computing it needs the bond energy AND the
    /// active table's well depth — both the producer's, neither the renderer's. This field
    /// is here because the compiler found `bonds.rs` reaching for `s.table().d_e` after I
    /// had told the lead the drawing layer only needed positions and pairs. It needed one
    /// more thing, and the narrow waist is only narrow if it is complete.
    pub depth: f32,
}

/// Everything the drawing layer reads in a frame, and nothing else.
#[derive(Resource, Clone, Debug, Default)]
pub struct FrameBuffer {
    /// Live atoms, in index order. `atoms.len()` IS the live count; there is no separate
    /// `n` to disagree with it.
    pub atoms: Vec<FrameAtom>,
    /// Bonded pairs only. The BOND CRITERION belongs to the engine (`E_rel < 0` and inside
    /// the outer turning point); the producer applies it and the renderer draws the
    /// verdict, so no distance threshold of the renderer's invention exists anywhere.
    pub bonds: Vec<FrameBond>,
    /// The atom the hand is holding, if any.
    pub grabbed: Option<usize>,
    /// Where the hand's anchor is, in the same coordinates as the atoms. Drawn as the
    /// spring's far end, and drawn AT ALL because the spring is a term in the ledger: if a
    /// viewer can watch W_ext move they should be able to see what moved it.
    pub anchor: (f64, f64, f64),
    /// The frame this buffer describes, so a consumer can tell a stale buffer from a
    /// paused one. A renderer that cannot distinguish those draws a frozen scene and
    /// reports it as live.
    pub frame: u64,
}

impl FrameBuffer {
    /// Is atom `i` an endpoint of any bond in this frame?
    ///
    /// Linear in the bond list on purpose: this is called once per atom per frame over a
    /// scene the acuity law keeps small, and a cached set would be a second copy of a fact
    /// the bond list already carries.
    pub fn is_bonded(&self, i: usize) -> bool {
        self.bonds.iter().any(|b| b.i == i || b.j == i)
    }
}
