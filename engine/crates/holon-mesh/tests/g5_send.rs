//! **G5, pre-registered before the refactor exists.**
//!
//! `SANDBOX_4090.md` G5: `FractureModel` and `ImpactModel` hold `Rc<RefCell<WallChart>>`, so
//! they are `!Send` and cannot cross a thread — which makes them unusable from a mesh shard.
//! `MESH_DESIGN.md` §7 makes G5 the blocking item for the mesh runtime carrying a real solver.
//!
//! The refactor itself lives entirely inside `ciris-sim-core` (six sites in `fracture.rs`, four
//! in `impact.rs`) and this lane must not touch those files. **What it CAN do is write the gate
//! first**, so the target is a committed expectation rather than something decided after the
//! fact by whatever the refactor happens to produce.
//!
//! # The gate — PRE-REGISTERED, then MET
//!
//! Written before the refactor existed, in both directions:
//!
//! * **before G5**: `cargo test --features g5` **failed to compile**, the compiler naming
//!   `Rc<RefCell<WallChart>>` as the reason. That failure was the before-state, established
//!   by the compiler rather than asserted here.
//! * **after G5**: it compiles and passes.
//!
//! **G5 landed and the target was met with no edit to the assertion.** The `cfg(feature)`
//! gate has therefore been removed: the Send requirement is now unconditional, so a
//! regression that re-introduced shared ownership would break the default build rather than
//! an opt-in one. The feature flag itself is kept as a no-op alias for one release so any
//! script invoking `--features g5` keeps working.
//!
//! # Why there is no runtime "is it Send?" probe here
//!
//! `T: Send` cannot be negated in a bound on stable Rust, so a first version of this file used
//! autoref-based specialisation to surface Send-ness as a runtime `bool`. **It did not work,
//! and it failed in the dangerous direction**: the specialised arm was never reached, the probe
//! returned `false` for every type including `u64`, and the before-state assertion therefore
//! PASSED for entirely the wrong reason. A control test (`u64` must probe as `Send`) is what
//! caught it — the same discipline the mutation suite runs on.
//!
//! The lesson is the one this crate keeps re-learning: an instrument that cannot report both
//! outcomes reports nothing. `assert_send::<T>()` below cannot be vacuous, because it is the
//! compiler's own bound check — if the type is not `Send`, the file does not build.

use ciris_sim_core::fracture::FractureModel;
use ciris_sim_core::impact::ImpactModel;

/// Compiles only for types that are `Send`. Not an assertion about a value — a bound the
/// compiler must discharge, which is why it cannot silently pass.
fn assert_send<T: Send>() {}

/// **The mesh side is already `Send`, so G5 is genuinely the only thing in the way.**
///
/// Asserted rather than assumed: if some part of the mesh were itself `!Send`, G5 could land
/// and the mesh still would not work. Each line below fails to COMPILE if it stops being true.
#[test]
fn the_mesh_side_is_already_send() {
    assert_send::<holon_mesh::MeshSpec>();
    assert_send::<holon_mesh::Grid>();
    assert_send::<holon_mesh::Partition>();
    assert_send::<holon_mesh::Mesh>();
    assert_send::<holon_mesh::MeshShard>();
    assert_send::<holon_mesh::Reference>();
    // The two core types a shard is built from. A shard IS an arena; if the arena were !Send
    // the whole design would be wrong, not merely blocked.
    assert_send::<ciris_sim_core::regplus::GrossState>();
    assert_send::<ciris_sim_core::runtime::RuntimeArena>();
}

/// **The G5 target — pre-registered, and MET.**
///
/// This was `#[cfg(feature = "g5")]` and did not compile; G5 landed and it now passes with
/// the assertion unchanged. Unconditional from here, so losing `Send` again fails the
/// default build.
///
/// What made it possible: the `Rc` went away, not the `RefCell`. `RefCell<T>` is `Send`
/// whenever `T` is — it is `Rc<T>` that is `Send` for no `T` at all — so the fix was to
/// remove the SECOND OWNER rather than the interior mutability, which
/// `RuntimeBoundaryModel::refinement_priority(&self)` still requires.
#[test]
fn after_g5_the_solvers_are_send() {
    assert_send::<FractureModel>();
    assert_send::<ImpactModel>();
}

/// The two types G5 targets are reachable from here, so the gate above is aimed at the real
/// things rather than at names that might have moved. This compiles today: naming a type is
/// not the same as requiring it to be `Send`.
#[test]
fn the_g5_target_types_are_reachable() {
    fn accepts<T>() {}
    accepts::<FractureModel>();
    accepts::<ImpactModel>();
}
