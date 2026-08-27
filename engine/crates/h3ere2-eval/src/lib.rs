//! h3ere2 response-quality experiment: the SLM perceives and articulates, the physics
//! engine reasons. Built to `scratchpad/h3ere2_eval/PREREG.md` (commit 1dac9a0), which
//! is binding and was sealed before this crate existed.
//!
//! Excluded from the workspace on purpose: it depends on `ciris-nl`'s native inference
//! feature, and an in-workspace dependency on a C++ build script could unify into
//! `ciris-sim-core`'s graph and falsify the isolation gates.
//!
//! **BUILD FROM INSIDE THIS DIRECTORY.** The exclusion is implemented by this crate
//! carrying its own `[workspace]` table, which makes it a separate workspace root — so
//! from `sim_engine/` the obvious invocation
//!
//! ```text
//! cargo build -p h3ere2-eval        # WRONG: "did not match any packages"
//! ```
//!
//! resolves to NOTHING. It is not an error about a broken crate; it is a package-selection
//! miss, and a silent no-op is worse than a failure because it reads as "nothing to build".
//! The correct invocations are
//!
//! ```text
//! cd sim_engine/crates/h3ere2-eval && cargo build --release
//! cargo build --release --manifest-path sim_engine/crates/h3ere2-eval/Cargo.toml
//! ```
//!
//! Consequence worth stating once: `ci-gates.sh` covers `ciris-nl --features native`, but
//! nothing in the workspace reaches THIS crate, so `bin/generate` breaking is invisible to
//! the board. It broke exactly that way once — see the note on `Session::generate` below.
//!
//! `bin/generate` needs two things from `ciris-nl` that the label bridge does not:
//! `chat::system_turn` and `Session::generate` (free-text, unconstrained). Both were once
//! carried as uncommitted working-tree edits and were lost; the crate then sat committed
//! but uncompilable. If `generate` stops building, check those two first.

pub mod blocks;
pub mod path;
pub mod scramble;
