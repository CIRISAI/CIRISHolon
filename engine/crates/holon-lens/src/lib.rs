//! THE LENS LAYER: readings of a trajectory, and the census that decides whether a
//! reading is a molecule.
//!
//! Every module here answers one question about a run that has already happened. None of
//! them decides physics: the bond criterion belongs to `Sim::refresh_pairs` and arrives
//! in the dump as bits, and the lenses that DO compute something new (`q6`, the hexatic,
//! the H-bond census) say so and state their criterion in full.
//!
//! The organising discipline is Object rule 9 — refusal is a feature. A lens asked for a
//! reading its scene cannot carry REFUSES and names the gate whose passing would lift the
//! refusal, rather than returning a number that looks like a measurement. The quench
//! scenes this crate was built for are two-dimensional, so the tetrahedral and `q6`
//! lenses refuse on them by construction; that is the honest reading and not a gap.

pub mod census;
pub mod field;
pub mod field_lg;
pub mod classifier;
pub mod lens;
pub mod partition;
pub mod quenchlog;
pub mod synthetic;
pub mod traj;
