//! The H2 potential energy curve, computed rather than tabulated.
//!
//! # What this crate is for
//!
//! `holon-render` used to load its pair potential from a JSON file someone else had
//! produced. That made the atom sandbox a *player* of a curve. This crate makes it a
//! *solver* of one: at page load the browser computes the STO-3G full-CI energy of H2
//! from closed-form Gaussian integrals, differentiates it analytically for the forces,
//! locates its own equilibrium and its own dissociation asymptote, and hands the result
//! to the same interpolator the file used to fill. Nothing about the physics is read in.
//!
//! # The three things it computes, and the one thing it is given
//!
//! Given: six decimal numbers, the STO-3G hydrogen contraction (`sto3g::H_EXPONENTS`
//! and `sto3g::H_COEFFS`). A basis set is a model choice and this crate states its own.
//!
//! Computed: every integral (`sto3g`), the exact-in-basis ground state by two
//! independent CI routes (`h2`), and the curve with its first and second derivatives
//! (`table`). The hydrogen-atom energy, the dissociation asymptote, `R_e` and `D_e` are
//! all results — none of them is quoted from anywhere, and there is no number in this
//! crate that came from a table of physical constants.
//!
//! # The referee gate
//!
//! The claim "the engine computes the same model" is only worth as much as its check.
//! `tests/referee.rs` compares this f64 implementation against a 50-digit mpmath
//! implementation of the same model, point by point, at all 492 separations of the
//! pinned referee curve — and pins the referee itself by digest so the comparison
//! cannot silently start grading against a different file. The two implementations share
//! no code, no language and no arithmetic; they share only the model definition.
//!
//! # What is deliberately NOT claimed
//!
//! STO-3G is a minimal basis and full CI in a minimal basis is not the answer nature
//! gives. Everything here is EXACT-IN-MODEL: the `R_e` and `D_e` below are properties of
//! this model, and this crate never compares them to experiment. The referee gate
//! measures agreement between two implementations of one model, which is a statement
//! about arithmetic, not about hydrogen.

pub mod dual;
pub mod elements;
pub mod fci;
pub mod h2;
pub mod md;
pub mod pair;
pub mod special;
pub mod sto3g;
pub mod table;
pub mod trimer;

pub use h2::{asymptote, equilibrium, h2_energy, h2_point, h_atom_energy, Point};
pub use table::{generate_table, stream_table, Meta, Table, PROVENANCE};

// ---------------------------------------------------------------- the referee pins
//
// These constants are the gate's memory. `tests/referee.rs` enforces every one of them
// on every `cargo test`, and `holon-render` exports the residual through its ABI so the
// viewer's banner states the number rather than the adjective.

/// FNV-1a (32-bit) of `tests/data/referee_h2_sto3g_fci.json`, the pinned 50-digit curve.
///
/// Pinned by DIGEST rather than by shape: a length check or a spot-check of `R_e` would
/// pass against a file whose interior had been edited, and the gate would then be
/// grading against a referee nobody had looked at.
pub const REFEREE_DIGEST: u32 = 0x72ad_8fa3;

/// Number of separations in the pinned referee curve.
pub const REFEREE_GRID_POINTS: usize = 492;

/// THE STAKE: the pointwise agreement this implementation is required to reach, in
/// hartree. Chosen before the comparison was run, at the scale f64 roundoff on an
/// `O(1)` energy makes reachable.
pub const REFEREE_STAKE_E: f64 = 1e-12;

/// The MEASURED worst pointwise disagreement, hartree — the number the viewer's banner
/// displays. Pinned about a factor two above the observed maximum so a libm difference
/// between platforms does not read as a regression; `tests/referee.rs` fails both if the
/// measurement exceeds it and if it is left more than a decade looser than reality.
pub const REFEREE_MEASURED_E: f64 = 5e-15;

/// The same, for the force column, hartree/bohr.
pub const REFEREE_MEASURED_F: f64 = 1e-13;

/// The same, for the curvature column, hartree/bohr^2.
pub const REFEREE_MEASURED_E2: f64 = 1e-12;

/// The same, for the equilibrium separation, bohr. `R_e` is a ROOT rather than a value,
/// so its residual is set by how flat the curve is at the bottom of the well: an energy
/// error `dE` displaces the root by `sqrt(2 dE / E'')`, and `E''(R_e) = 0.477`. That the
/// measured displacement is a couple of ulp instead is the Newton polish landing on the
/// last bit, not the bound being generous.
pub const REFEREE_MEASURED_R_E: f64 = 5e-15;

/// The same, for the well depth, hartree.
pub const REFEREE_MEASURED_D_E: f64 = 5e-15;

/// FNV-1a (32-bit). Small, dependency-free, and adequate for its one job: noticing that
/// the pinned referee file is not the file the pin was taken from. It is NOT a security
/// primitive and is not used as one.
pub fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for &b in bytes {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}
