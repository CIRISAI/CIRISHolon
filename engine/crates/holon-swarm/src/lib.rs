//! # holon-swarm
//!
//! A CPU prototype that de-risks a GPU sharding design for the CIRIS holon engine.
//!
//! N independent holon engines run in parallel on `std::thread`, exchanging **conserved
//! integer** REG+ ledger quantities across shard boundaries, behind a conservation gate
//! whose every leg is mutation-tested.
//!
//! ## The frame constraint, and how sharding satisfies it
//!
//! `sim_engine/INTEGRATION_FRAME.md` rule 1: *all tiers use THE holon* — no per-tier
//! structs, no new fields. Rule 2: *the only difference between tiers is VALUES.* So
//! sharding may not introduce a shard type, a boundary type, or a message type.
//!
//! It does not need to:
//!
//! * a **shard is an arena** — one [`ciris_sim_core::runtime::RuntimeArena`] with its own
//!   declared root and its own `g0` (opening balance);
//! * a **boundary port is a holon** with the `boundary: true` flag the holon already has;
//! * a **boundary relation is a pair of indices**, `(shard, holon)` on each side — values,
//!   exactly as the frame's own descriptor-library decision already established
//!   ("Two arenas ... related through `MaterialBinding` indices is frame-faithful");
//! * an **exchange is a transfer of integer ledger quantity** between two arenas' boundary
//!   holons — [`GrossState`](ciris_sim_core::regplus::GrossState) arithmetic, nothing else.
//!
//! Nothing here is a new entity class. The single piece of state that is *not* inside the
//! arena is the live ledger overlay, and that is forced by the core's API rather than
//! chosen — see [`shard`] for the full statement, and for the gate weakness it creates.
//!
//! ## What is conserved, and how strongly
//!
//! [`GrossState`](ciris_sim_core::regplus::GrossState) is four integers:
//! `constituents: u64`, `occupancy: u64`, `momentum: [i64; 2]`. All four are transferred
//! across boundaries and all four are gated. Conservation is **bit-equality on integers**,
//! not a residual under a tolerance: there is no epsilon anywhere in this crate.
//!
//! ## Quick tour
//!
//! ```no_run
//! use holon_swarm::{FaultInjection, GateLevel, Swarm, SwarmSpec};
//!
//! let spec = SwarmSpec::ring(8, 4096).with_gate(GateLevel::Full);
//! let mut swarm = Swarm::new(&spec).unwrap();
//! let opening = swarm.global_ledger().unwrap();
//! swarm.run_rounds_threaded(100, 8, FaultInjection::None).unwrap();
//! assert_eq!(swarm.global_ledger().unwrap(), opening); // exact, all four lanes
//! ```

pub mod error;
pub mod exchange;
pub mod gate;
pub mod ledger;
pub mod shard;
pub mod swarm;

pub use error::{Side, SwarmError};
pub use exchange::{plan_transfer, BoundaryPair, FaultInjection};
pub use gate::{ConservationGate, GateLevel};
pub use ledger::{LedgerDelta, LANES};
pub use shard::{Shard, ShardLink};
pub use swarm::{RoundOrder, Swarm, SwarmSpec};
