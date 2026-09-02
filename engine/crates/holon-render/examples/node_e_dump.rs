//! NODE E's raw-bit dump of the four staked scenes, run classically.
//!
//! Exists to be built and run at TWO commits: the one before `Sim::step` was factored
//! into its passes, and the one after. If the factoring moved a bit, the two outputs
//! differ and the refactor is not a refactor. The dump itself is
//! `tests/common/node_e_scenes.rs`, shared with `tests/node_e.rs` so the two of them
//! cannot disagree about what they are comparing.
//!
//! Uses public API only, so it builds unchanged at both commits.
//!
//!     cargo run -p holon-render --release --example node_e_dump

#[path = "../tests/common/node_e_scenes.rs"]
mod node_e_scenes;

fn main() {
    print!("{}", node_e_scenes::dump_classical());
    print!("{}", node_e_scenes::count_negative_zeros());
}
