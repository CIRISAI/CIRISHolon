//! q8-mps — MPS/DMRG ground-state engine with LEDGERED BONDS.
//!
//! Prereg: `sim_engine/Q8_MPS_PREREG.md` (frozen + amendment). Nothing lands here
//! that is not staked there first. Build order is the staked order: G-SVD, then
//! G0/G1, before anything downstream.

pub mod dmrg;
pub mod eigen;
pub mod lanczos;
pub mod mpo;
pub mod mps;
pub mod observables;
pub mod ops;
pub mod svd;
