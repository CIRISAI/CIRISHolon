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

pub mod cluster;
pub mod dual;
pub mod elements;
pub mod budget;
pub mod fci;
pub mod h2;
pub mod ion_table;
pub mod ions;
pub mod lanes;
pub mod md;
pub mod ooh;
pub mod ozone;
pub mod pair;
pub mod qcd2;
pub mod quaternary_table;
pub mod rpmd;
pub mod scalar;
pub mod sigma_op;
pub mod special;
pub mod sto3g;
pub mod table;
pub mod tier;
pub mod trimer;
pub mod tower;
pub mod vecspace;
pub mod water;

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

// ------------------------------------------------------- the SATURATION-2 referee pins
//
// The same three things the H2 pins above carry, for the (O, H, H) three-body surface:
// which file the gate grades against, how many geometries it must contain, and what the
// agreement actually measured — the last checked in BOTH directions, so a bound left far
// looser than reality fails as a stale pin rather than passing as a comfortable one.

/// FNV-1a (32-bit) of `tests/data/s2/water_referee.json`, the pinned 50-digit (O, H, H)
/// staked set.
pub const WATER_REFEREE_DIGEST: u32 = 0x6d0ee974;

/// Geometries in that file. The prereg stakes at least 48; the referee's ladder produces
/// 84, and `tests/water_referee.rs` enforces both the count and the floor.
pub const WATER_REFEREE_GEOMETRIES: usize = 84;

/// THE STAKE: gate R1's required agreement, hartree, written in the prereg before any
/// comparison was run.
pub const WATER_R1_STAKE_E: f64 = 1e-10;

/// The MEASURED worst disagreement over the staked set, hartree: 5.921e-12, on the `dE3`
/// column, pinned at twice it so a libm difference between platforms does not read as a
/// regression.
///
/// Three decades wider than the H2 curve's `REFEREE_MEASURED_E`, and that is expected
/// rather than tolerated. `E(H2O)` is about -75 hartree against H2's -1.1, so an f64
/// carries roughly 1e-14 of absolute room before its own rounding — and `dE3` is a
/// difference of FIVE such numbers, four of them near -75, so the cancellation alone
/// costs about a decade on top. 5.9e-12 is what that arithmetic predicts and it is 17x
/// inside the prereg's 1e-10 stake.
pub const WATER_R1_MEASURED_E: f64 = 1.2e-11;

// ------------------------------------------------------------ the SATURATION-2 gate pins
//
// The stakes are the prereg's, written before any of them was measured. The MEASURED
// constants beside them are what the gates actually read, pinned so a regression fails as
// a regression; `tests/water.rs` also refuses a pin left more than two decades looser than
// reality, because a bound that far from the measurement has stopped being evidence.

/// T1's KILL, hartree: the largest held-out interpolation error the (O, H, H) table may
/// carry. Staked in `SATURATION2_PREREG.md`.
pub const WATER_T1_KILL_E: f64 = 1e-3;

/// The MEASURED worst held-out error over the staked 256-point draw, hartree: 2.467e-4,
/// pinned at twice it.
///
/// The gate's draw is deterministic — fixed seed, fixed rejection rule, fixed count — so
/// this number does not drift unless the table or the code does, which is what makes it a
/// pin rather than a tolerance.
///
/// What it is NOT is the surface's worst case. `examples/s2_build.rs` runs an independent
/// 384-point draw over the same domain and reads 7.68e-4 — three times the gate's — and
/// the spread between two honest draws IS the evidence that a maximum over a finite draw
/// understates the supremum it stands for. The same fact cost this campaign its first
/// truncation radius. Both numbers are inside the 1e-3 kill; neither is the supremum.
pub const WATER_T1_MEASURED_E: f64 = 5e-4;

/// T2's KILL, hartree: the domain-boundary systematic, i.e. the largest `|dE3|` anywhere
/// on the shell the surface is truncated at. Staked in `SATURATION2_PREREG.md`.
pub const WATER_T2_KILL_E: f64 = 1e-5;

/// The MEASURED systematic on the truncation shell, hartree.
///
/// `examples/s2_domain.rs` swept twelve shells to choose `R_HI`; this is the chosen one,
/// re-measured inside the suite by a two-stage search that reaches past the table's own
/// closed-angle fence. The first choice of `R_HI` was 14, and re-measuring it at five
/// times the resolution moved it from 9.71e-6 to 1.0091e-5 and across the stake — which
/// is why the gate refines rather than reading one grid.
pub const WATER_T2_MEASURED_E: f64 = 5e-6;

/// G2's STAKE: how much shallower the third hydrogen's best binding must be than water's
/// own second O-H bond. A factor, staked in `SATURATION2_PREREG.md`.
pub const WATER_G2_STAKED_RATIO: f64 = 5.0;
