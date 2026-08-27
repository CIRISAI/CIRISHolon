//! # ciris-sim-core
//!
//! The deterministic physics core for the CIRIS holon. No rendering,
//! no `std`, no non-deterministic iteration — everything must replay bit-identically
//! across `wasm32-unknown-unknown`, `wasm32-wasip1`, and native CI, following the
//! `ciris-game-engine-core` pattern.
//!
//! ## What makes this physics rather than a force layout
//!
//! Every constant is supplied by a theorem or a measurement from CIRISOntology
//! (42 sorry-free Lean modules), not chosen by feel:
//!
//! * springs are the **measured** couplings ([`data::COUPLING`]);
//! * lengths are **resistance distance** on the coupling Laplacian, a proper metric;
//! * the conserved charges are the **twin parities** — the Z2xZ2 character sectors
//!   have dimensions 9/1/1/0, and the two one-dimensional sectors ARE the dark modes;
//! * the twin dark mode is an **exact** eigenvector, annihilated by every other row
//!   (`DarkState.twin_dark_state`, `dark_state_decoupled`) — so the twin probe has a
//!   *proved* null result, and the measured coupling departs from it by a known amount.
//!
//! ## Dense structure vs sparse stepping
//!
//! [`Structure`] is the theorem/measurement object: metric, spectrum, susceptibility
//! and symmetry projectors. It remains dense because those derived quantities are dense.
//! [`sparse`] is the execution object for large spring networks: it stores only the
//! supplied edges and performs velocity-Verlet stepping in `O(E)` with no allocator.
//! Keeping these separate means sparse scenes do not pay an `O(N^3)` eigensolve or
//! `O(N^2)` storage merely to advance the equations of motion.
//!
//! ## Sizes, and where the fast path went (E10)
//!
//! The crate was originally written against one structure: eleven kinds, one coupling,
//! and every derived quantity emitted as a compile-time table. That is what made it
//! fast, and — per FSD §10.1 — what made it unbenchmarkable against a general engine.
//!
//! It is now generic in `const N: usize`, with the eleven-kind case kept as a
//! specialisation:
//!
//! * [`structure::K11`] is a `static` built by a `const fn` from the shipped
//!   [`tables`]. Using the built-in object costs a reference; no linear algebra runs,
//!   exactly as before.
//! * [`Structure::from_coupling`] takes an arbitrary symmetric coupling at any `N` and
//!   COMPUTES the metric, the spectrum, the susceptibility and the character
//!   projectors with [`linalg`] — one `O(N^3)` eigensolve at construction.
//! * [`sparse::SparseSystem`] is the general stepping path when those global derived
//!   quantities are not required. Its construction and force evaluation are `O(E)`.
//!
//! The dense and sparse paths intentionally coexist: the former carries the ontology's
//! global invariants, while the latter removes the scaling bottleneck for ordinary
//! sparse mechanics.
//!
//! [`holon`] is the recursive layer above both paths. Every entity is the same [`holon::Holon`]
//! type, with REG+ gross state, irreducible whole-state, typed realizations, and
//! an adaptive boundary frontier selected by a model-supplied parity certificate.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod data;
pub mod entropy;
pub mod linalg;
pub mod structure;
pub mod tables;
pub mod dynamics;
pub mod gaps;
pub mod field;
pub mod twin_probe;
pub mod sectors;
pub mod sparse;
pub mod relativity;
pub mod curvature;
pub mod bridge;
pub mod holon;
pub mod mechanical;
pub mod material;
pub mod closing;
pub mod homogenization;
pub mod locality;
pub mod quantum_link;
pub mod regplus;
pub mod reversibility;
#[cfg(feature = "alloc")]
pub mod descriptor;
#[cfg(feature = "alloc")]
pub mod fracture;
#[cfg(feature = "alloc")]
pub mod impact;
#[cfg(feature = "alloc")]
pub mod runtime;

pub use data::{ChoiceKind, Disposition, COUPLING, DEPTH, KINDS, N, TWINS};
pub use structure::{Structure, K11};

/// A square `N x N` matrix of `f64`, row-major, at an arbitrary size.
pub type MatN<const N: usize> = [[f64; N]; N];
/// A vector of length `N` over an arbitrary size.
pub type VecN<const N: usize> = [f64; N];

/// A square matrix over the eleven built-in kinds.
pub type Mat = MatN<N>;
/// A vector over the eleven built-in kinds.
pub type Vec11 = VecN<N>;
