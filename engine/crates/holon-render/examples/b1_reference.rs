//! Print MIXTURES-1 gate B1's all-hydrogen reference dump.
//!
//! ```text
//! cargo run -p holon-render --release --example b1_reference > reference.txt
//! ```
//!
//! The dump itself is `tests/common/b1_dump.rs`, shared with `tests/mixtures.rs` so the
//! file this produced and the gate that re-derives it cannot drift apart. This binary
//! exists because the reference had to be produced by code that predates the bank: it was
//! run in a git worktree at the parent commit, and the output committed as
//! `tests/data/b1_hydrogen_reference.txt`.
#[path = "../tests/common/b1_dump.rs"]
mod b1_dump;

fn main() {
    print!("{}", b1_dump::dump_all());
}
