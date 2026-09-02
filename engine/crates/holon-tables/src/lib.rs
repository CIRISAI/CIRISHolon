//! # holon-tables
//!
//! **SATURATION-3 G1: a potential-energy table generated across workers, bit-identical
//! whatever the worker count, with corruption convicted by an exact digest.**
//!
//! The physics lives in `holon-chem`. This crate does not solve anything; it decides WHICH
//! node is solved WHERE, in WHAT ORDER, and from WHAT START — and then proves that none of
//! those decisions reached the numbers.
//!
//! ## The claim, and why it is not free
//!
//! `lean/CIRISHolon/MergeLaw.lean` proves `shardedFold_invariant`: every sharding of a run
//! folds to the run's own sum. That theorem is about an **additive commutative monoid**,
//! and it is exact. A table of `f64` energies is neither — floating point addition is not
//! associative, and a table is not a sum in the first place; it is a map from node to
//! value.
//!
//! So the merge law is not inherited here by assertion. It is instantiated twice, at the
//! two places where it actually applies, and the parts it does NOT cover are named:
//!
//! | what | how it is carried |
//! |---|---|
//! | the table's CONTENTS | not a fold at all — a disjoint union of independent node solves. Shard-invariance is a statement that no node's value depends on its shard, and that is [`grid`]'s job, by construction. |
//! | the table's CERTIFICATE | a genuine fold in a genuine additive commutative monoid: `(Z/2^64)^4` under wrapping addition, which is associative and commutative unconditionally. `shardedFold_invariant` and `digest_convicts` apply to it literally. See [`digest`]. |
//!
//! Anything claiming the Lean covers the f64 table itself would be laundering a theorem
//! about integers into a claim about floats. It does not, and this crate says so where
//! someone would otherwise assume it.
//!
//! ## The thing that made this hard, measured rather than assumed
//!
//! The generator warm-starts each node's Davidson from a neighbour's converged vector.
//! The obvious design is for a worker to start from whatever it solved last. That design
//! is WRONG, and `holon-chem/examples/s3_warm_probe.rs` is the measurement that says so:
//! on `(H,H,Cl)`, warm and cold solves of the same geometry agree to between `3.4e-13` and
//! `4.3e-12` hartree and **never** bit-for-bit, 0 of 5 pairs. A warm start moves the last
//! bits of the answer.
//!
//! If the warm-start source were "whatever this worker solved last", every node's value
//! would therefore be a function of the worker count, and the table would silently differ
//! between a 1-worker and a 32-worker run. That is exactly what G1 forbids.
//!
//! The fix is [`grid::TableGrid`]'s region decomposition: **the partition into regions and
//! the traversal within a region are canonical functions of the grid, fixed before any
//! worker exists.** A region is self-contained — its seed is cold, every later node warm-
//! starts from its canonical predecessor inside the same region — so which worker runs a
//! region, and in what order regions are handed out, cannot reach the numbers. Worker count
//! buys parallelism and nothing else.
//!
//! The cost is honest and is the quantity the locality sweep should be sized against: one
//! cold seed per region, so the cold fraction is `1 / region_volume`.
//!
//! ## The other thing that made this hard
//!
//! A wrong warm start does not announce itself. Measured on the same path: a random start
//! vector converged onto an eigenvector **7.47 hartree above the ground state** while
//! reporting a residual of `5.98e-11` against the correct solve's `5.24e-11`, and the
//! identical exit reason. Neither the residual nor the exit reason can separate them,
//! because a residual is small for ANY eigenvector and this one had genuinely converged —
//! to the wrong state.
//!
//! The guard is [`node::void_reason`]'s variational check: `E <= min_i H_ii`. A single
//! determinant is itself a trial vector, so the ground state is bounded above by every
//! diagonal element; `diag` is already computed for the preconditioner, so the check is
//! free. It fires on that plant by 7.4 hartree and passes both good solves by 5.4e-2.
//!
//! ## What can fail
//!
//! A gate that cannot fail proves nothing, and `holon-mesh`'s header names the specific
//! trap this crate would otherwise fall into: reordering a merge over exact lanes produces
//! the IDENTICAL result, so "reorder and assert the answer moved" cannot pass against a
//! correct implementation. The mutation set is therefore split in the same way
//! ([`mutation::Mutation`]):
//!
//! * [`mutation::Mutation::ReverseRegionOrder`] and dynamic worker scheduling must **not**
//!   move the table — they are the reorderings the design is supposed to absorb;
//! * [`mutation::Mutation::WorkerLocalWarmStart`] **must** move it — it is the design
//!   defect the region decomposition exists to prevent, and if it does not fire then the
//!   warm-start measurement above was wrong and the decomposition is unnecessary;
//! * [`mutation::Mutation::CorruptNode`] must be **convicted by the digest** with zero
//!   false positives on clean runs (plant (iv));
//! * [`mutation::Mutation::WrongWarmStart`] must **VOID its node** rather than write a
//!   silently different entry (plant (iii)).
//!
//! Only the set together proves anything. Each half alone is satisfiable by a broken
//! implementation.

//! ## The fold (WB-8.7)
//!
//! There is exactly ONE tabulation pipeline, and it is dimension- and composition-generic.
//! [`grid::NdGrid`] is the node set over any number of axes; [`surface::Surface`] is the one
//! seam where a composition says what its coordinates mean; [`generate::generate_surface_leased`]
//! is the leased generator, and every discipline in this crate's header lives in it once.
//!
//! The 3-axis trimer path is not a second implementation of any of that. [`generate`],
//! [`generate::generate_with_progress`] and [`generate::generate_leased`] build a
//! [`surface::TrimerSurface`] and an `NdGrid` and come straight back. The warrant is
//! bit-identity, and it is a test rather than an argument: `tests/nd_bit_identity.rs`
//! asserts, node by node and bit by bit, that `NdGrid::from_table_grid` agrees with the
//! [`grid::TableGrid`] it folds on `n_nodes`, `node_id`, `coords`, `geometry`, `region_of`,
//! `region_nodes` and the whole partition.

pub mod checkpoint;
pub mod digest;
pub mod generate;
pub mod grid;
pub mod mesh_reaper;
pub mod mutation;
pub mod node;
pub mod ohhh;
pub mod surface;
pub mod worker;

pub use digest::Digest;
pub use generate::{
    generate, generate_surface, generate_surface_leased, generate_surface_with_progress,
    GenOutcome, GenSpec, SurfaceSpec, WarmPolicy,
};
pub use grid::{Axis, AxisMap, NdGrid, NodeId, RegionId, Serpentine, TableGrid};
pub use mutation::Mutation;
pub use node::{NodeRecord, NodeStatus, VoidReason};
pub use surface::{DistanceTetramer, Realised, Surface, TrimerSurface};
pub use mesh_reaper::MeshWorld;
pub use worker::WorkerProbe;
