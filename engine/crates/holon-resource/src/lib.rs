//! # holon-resource
//!
//! **An allocation is a child holon: probed at birth, ledgered while it lives, released when the
//! need ends, and reclaimed if the rent stops.**
//!
//! Design: `engine/RESOURCE_DESIGN.md`, ADMITTED for implementation 2026-08-30. This crate is
//! the first increment of it, and it is built PLANTS FIRST (D13): a rule that has never fired
//! has never been demonstrated to gate.
//!
//! ## What is here, and what is deliberately not
//!
//! | in | why |
//! |---|---|
//! | [`ledger`] — integer receipts and the exact identity | `opened == released + convicted + live`, so a leak is a PROOF |
//! | [`lease`] — the child-holon tree, depth cap, leaf-to-root release | D7, D9 |
//! | [`probe`] — attempt the thing, never ask the holder | D1, D2, D4 |
//! | [`reaper`] — three rungs, the third being the reaper probing itself | D10 |
//! | [`tier`] — arithmetic precision as a leased resource, and the overflow rule | D3b |
//!
//! NOT here, and each absence is a decision: no CUDA (the GPU owner supplies its VRAM probe),
//! no thread pool (the pool owner supplies its worker probe), no dispatch registry yet, and no
//! arithmetic-tier probe — the solver owns its own floors, and `holon-chem`'s
//! `Scalar::expansion_floor()` is already the declared boundary this design would lease against.
//! The crate has ZERO dependencies and cannot acquire one: it sits under everything.
//!
//! ## The two rules this design turned out to share
//!
//! **D0** says a device class belongs to the ARTIFACT rather than the schedule, because a faster
//! arithmetic route that changes bits changes what was produced (SATURATION-3 G2: the GPU and
//! CPU sigmas agree to 3.033e-15 and differ bitwise on 91.0% of entries). **D3b** says an
//! arithmetic tier is a lease with a QUANTITATIVE boundary — `f64` guarantees residuals down to
//! its expansion floor and no further, and a request past it is an overflow that leases the next
//! rung rather than a constant to edit.
//!
//! They are one rule: **anything that reaches the numbers belongs to the artifact; anything that
//! reaches only the wall-clock belongs to the schedule.** That is also G1's own resolution —
//! chunking is definition, assignment is run — one level up.
//!
//! ## What a lease is NOT
//!
//! It does not reserve. A lease is a **receipt for rent paid** (D3): the probe buys validity
//! *now* and nothing after; every USE is itself a probe, so the write that fails is the
//! authoritative reading and the response is refuse-and-release rather than retry-forever; the
//! lease carries a declared horizon past which it is stale by definition; and the only thing
//! guaranteed forever is the ledger entry.
//!
//! The disk-full window of 2026-08-30 is why that is the shape: a probe passed and was false
//! milliseconds later, and the answer is not a sharper probe but a shorter claim about what the
//! probe means.

pub mod ledger;
pub mod lease;
pub mod probe;
pub mod reaper;
pub mod tier;

pub use ledger::{Ledger, Receipt, ReceiptError};
pub use lease::{Arena, Lease, LeaseError, LeaseId, LeaseState, MAX_DEPTH};
pub use probe::{AttemptProbe, LivenessProbe, Probe, ProbeVerdict, Response, ResourceKind, ScriptedProbe};
pub use reaper::{ReapEvidence, ReapVerdict, Reaper, ReaperWorld, ScriptedWorld};
pub use tier::{Boundary, Ladder, Provenance, Routing, Rung};
