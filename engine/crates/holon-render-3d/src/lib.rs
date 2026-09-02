//! # holon-render-3d
//!
//! The atom world in three dimensions: hydrogen atoms in a box under the exact STO-3G
//! pair potential, integrated symplectically, with every energy and momentum flow on a
//! gated ledger — drawn with Bevy 0.19 and pushed around with a finger.
//!
//! ## What this crate is not
//!
//! It is not a second physics engine. `holon-render` owns the potential table, the
//! velocity-Verlet integrator, the ledger and its two gates, the bond predicate and the
//! composite-holon census, and every one of those is used here unchanged — the same
//! rlib the canvas shell compiles to wasm. This crate owns a camera, some spheres, some
//! cylinders, an overlay and a finger. Where a number appears on the overlay it was read
//! from the `Sim`, never recomputed, so the two shells cannot disagree.
//!
//! ## What "3D" changed in the physics, and what it did not
//!
//! The lift lives in `holon-render`'s `sim.rs`, which now carries three components per
//! atom and six faces on the box. Almost nothing needed re-deriving, because almost
//! nothing was ever two-dimensional: the curve, the force law, the bond predicate, the
//! outer turning point, the drift bound and all three clocks are functions of the scalar
//! separation alone. Exactly two things were dimension-dependent and both are named at
//! their definitions — the equipartition denominator behind the temperature reading (2
//! translational degrees of freedom per atom against 3), and the opening arrangement of
//! the atoms (a ring becomes a sphere). The 2D scene is now the exact mid-plane slice of
//! this one, which is why the canvas shell and all 40 of its gate tests were untouched
//! by the change.
//!
//! ## Build shape
//!
//! `render` carries everything GPU-facing; `headless` links none of it and is what CI
//! and the gate tests run. See `Cargo.toml` for the feature table and `build-web.sh` for
//! the two wasm artifacts.

#![forbid(unsafe_code)]

// Re-export the physics unchanged, so a consumer of this crate reaches the certified
// core through it rather than through a copy of it.
pub use holon_render;

/// The frame buffer: the one thing the drawing layer consumes (Route B). Not behind
/// `render`, because the WORKBENCH producer fills it without any of the render machinery
/// and a headless consumer should be able to name the type.
pub mod frame;
pub mod world;

#[cfg(feature = "render")]
mod bonds;
#[cfg(feature = "render")]
mod hud;
#[cfg(feature = "render")]
mod lighting;
#[cfg(feature = "render")]
mod pick;
#[cfg(feature = "render")]
mod render;
#[cfg(feature = "render")]
mod scene;

/// Atoms the world opens with. Two, because the two-atom scene is the one the whole
/// thing exists to show: they cannot bond, however hard you push them.
pub const OPENING_ATOMS: usize = 2;

/// Build and run the application.
///
/// With `render` (native or wasm) this opens a window and draws the box. In a `headless`
/// build it spins up a `MinimalPlugins` app holding the same world with no GPU — the
/// shape CI links.
#[cfg(feature = "render")]
pub fn run() {
    render::run_app();
}

/// Headless entry point: `MinimalPlugins`, the world, no rendering.
#[cfg(not(feature = "render"))]
pub fn run() {
    use bevy::app::{PluginGroup, ScheduleRunnerPlugin};
    use bevy::prelude::App;
    use bevy::MinimalPlugins;

    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_once()))
        .insert_resource(world::AtomWorld::new(OPENING_ATOMS))
        .run();
}
