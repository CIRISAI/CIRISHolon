//! # holon-lattice — the fluid tier the programme was founded on, running.
//!
//! `CIRISOntology/Core/Lattice.lean` proves the FHP-6 local state space has 64 occupancies
//! carved by the conserved label `(N, P)` into 53 sectors with dimension histogram
//! 44 / 7 / 2. `ciris-sim-core::regplus` carries that object into the runtime. **Neither
//! carries a motion**, and both say so in their own headers: `regplus`'s is "it does not
//! invent a collision law: transitions are supplied elsewhere", and the Lean's is that REG+
//! collisions are "DEFINED as unitaries block-diagonal in these fibers — by construction,
//! not discovery". This crate supplies the motion, and it is the whole of what was missing.
//!
//! ## The first law
//!
//! **This tier is NOT a view of the molecular dynamics** and is never composed with it
//! through `Closed`'s composition theorem. It is its own object with its own state space,
//! its own clock, and its own warrant. The molecular-to-lattice seam is a separate claim
//! that takes no status from anything computed here. Adopted verbatim from rung 2's flag;
//! `conformance/mesh/LG_PREREG.md` §0 is the binding statement.
//!
//! ## What the census turned out to be
//!
//! Not a control — a **classification**. A sector-preserving collision permutes within
//! `(N,P)` fibers and can do nothing else, so `sector_dims` states exactly where a collision
//! law may act: the identity on all 44 fibers of dimension 1, and the whole space of REG+
//! collision laws on FHP-6 is `S₃ × (S₂)⁷ × S₃`, of order 4608 ([`state::COLLISION_GROUP_ORDER`]).
//! FHP-I is one named element of it: the 3-cycle on `{9,18,36}` — which is the Lean's own
//! `three_route_sector` — and the swap on `{21,42}`.
//!
//! ## What is certified, and what is not
//!
//! Certified: exact integer conservation per law, a bijective motion, the census
//! reproduction, and the closure reading along the chart scale `b`. **Not claimed:** the
//! Navier–Stokes limit. This crate measures the *necessary* lattice condition — fourth-rank
//! isotropy of the direction set ([`isotropy`]) — and stops there. Viscosity, semi-detailed
//! balance and the `g(ρ) ≠ 1` Galilean defect are unmeasured, and LG_PREREG §3 names them as
//! the exit.
//!
//! ## Prior art, credited
//!
//! * Frisch, Hasslacher & Pomeau, Phys. Rev. Lett. **56** (1986) 1505 — FHP-6, and the
//!   hexagonal lattice's fourth-order isotropy, which is the whole warrant of the founding
//!   64-state object.
//! * Hardy, Pomeau & de Pazzis (1973) — HPP-4. Its spurious per-line momentum invariant is
//!   this crate's positive control for a NON-vacuous exact closure, and is historically why
//!   FHP exists at all.
//! * d'Humières, Lallemand & Frisch, Europhys. Lett. **2** (1986) 291 — FCHC-24, enumerated
//!   already in `holon-mesh::fchc` and not run here.

pub mod chart;
pub mod isotropy;
pub mod lattice;
pub mod probe;
pub mod state;

pub use chart::{BlockChart, Field};
pub use lattice::{Lattice, Ledger};
pub use state::{Model, COLLISION_GROUP_ORDER};
