//! The generated world as engine types.
//!
//! A holon is an object with a POPULATION (counts of what it holds, by kind), a view upward
//! (its parent reads that population) and a generator downward (its children, from its
//! population and a seed). This crate is the executable face of three Lean files:
//!
//! | module | theorem it executes |
//! |---|---|
//! | [`apportion`] | the section law is satisfiable exactly: `SceneCert.Generator.aggregates` |
//! | [`seed`] | random access: a child's seed is a function of its parent's and its index |
//! | [`tree`] | `ScenePopulation.pop_comp`, `Ladder.zoom_consistent`, `Ladder.vacuum_generates_vacuum` |
//! | [`edges`] | `ScenePopulation.edgeView_comp`, `edgeView_total`, `Ladder.outflow_commutes` |
//! | [`verdict`] | `SceneCert.verdict_certified_iff`, `unresolved_ignores_match` |
//! | [`rhythm`] | `SceneRhythm.Continuity`, `continuity_coarse`, `source_is_the_defect` |
//! | [`world`] | the call of `FSD_WORLD_MODEL.md` section 10: one verdict per requested row, always; and the first generator with a reference behind it |
//! | [`dist`] | the cost of a coupling in one dimension, exact: `SceneCert.cost` |
//! | [`sums`] | the checker for sums a consumer emits, one JSON object per line |
//!
//! Integers only. No float appears in this crate, so a native build and a wasm build agree
//! bit for bit by construction and not by test.
//!
//! What this crate does NOT do: decide whether a generated scene matches reality. That is the
//! statistical half of the certificate and needs a reference sample; [`dist`] is the
//! arithmetic it will use and [`verdict`] is the order in which it is consulted.

pub mod apportion;
pub mod dist;
pub mod edges;
pub mod household;
pub mod rhythm;
pub mod seed;
pub mod sums;
pub mod tree;
pub mod verdict;
pub mod world;
