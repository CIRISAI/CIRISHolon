//! # q-seam — the certified seam
//!
//! An exact quantum reference and a Boolean-occupancy chart over the *same* system, with a
//! refusal criterion staked before this crate existed. The product is not a better solver; it is
//! the machine declining to speak where correlation makes the chart lie.
//!
//! Everything here is governed by `sim_engine/Q_SEAM_PREREG.md`, frozen at commit `3b6eed0` with
//! amendment A1 at `dab10aa`, both before the first line of this crate was written. Thresholds
//! marked STAKED there are constants here; nothing in this crate may adjust one.
//!
//! **Exactness first.** `Q_SEAM_PREREG.md` §3 is a ladder of sixteen gates, and no configuration
//! reaches the chart, the certificate or the share until it has passed them. A gate failure makes
//! a configuration VOID — excluded and reported as excluded — never a refusal and never a datum.

pub mod audit;
pub mod certificate;
pub mod chart;
pub mod dense;
pub mod hubbard;
pub mod lanczos;
pub mod observables;
pub mod region;
pub mod share;

/// Chain lengths of the pinned sweep (`Q_SEAM_PREREG.md` §1).
pub const SWEEP_SITES: [usize; 5] = [2, 4, 6, 8, 10];

/// Interaction strengths of the pinned sweep, in units of `t`.
pub const SWEEP_U: [f64; 14] = [
    0.0, 0.125, 0.25, 0.5, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0,
];

/// The plant: the sweep's far end, which every surviving criterion must refuse.
pub const PLANT_U: f64 = 16.0;

/// Staked tolerances for the six observables (`Q_SEAM_PREREG.md` §2.1), in the order
/// e, d, n_i, m_i, bond, D_bool.
pub const TAU: [f64; 6] = [0.02, 0.02, 0.02, 0.05, 0.02, 0.05];

/// The staked safety factor shared by C1 and C3.
pub const KAPPA: f64 = 0.5;

/// Q7's chain lengths (`Q7_SEAM_PREREG.md` §3). N = 12 is optional (D0-c).
pub const Q7_SITES: [usize; 2] = [8, 10];
/// Q7's interaction grid.
pub const Q7_U: [f64; 7] = [0.0, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0];
/// Q7's trap-depth grid. `a = 0` is the no-spatial-variation control and the Q5 regression tie.
pub const Q7_A: [f64; 6] = [0.0, 0.5, 1.0, 2.0, 4.0, 8.0];

/// Q7b: the symmetric three-level box at N = 10 (`Q7B_SEAM_PREREG.md` §2). N = 8 is IMPOSSIBLE
/// here by arithmetic — with 4 symmetric blocks at half filling the pattern is (A,B,B,A) with
/// A+B=2, so it admits only {0,2} or {1,1} and never all three regimes.
pub const Q7B_SITES: usize = 10;
pub const Q7B_V: [f64; 8] = [1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0];
pub const Q7B_U: [f64; 7] = [0.0, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0];

/// `v = [+V,+V, −V,−V, 0,0, −V,−V, +V,+V]` — outer blocks emptied, deep wells doubly occupied,
/// centre left at n ≈ 1. Block boundaries coincide with region boundaries, so the step is sharp
/// on the region scale. Reflection-symmetric, which is what keeps D1b and the mirror gate alive.
pub fn q7b_box(v: f64) -> Vec<f64> {
    vec![v, v, -v, -v, 0.0, 0.0, -v, -v, v, v]
}
