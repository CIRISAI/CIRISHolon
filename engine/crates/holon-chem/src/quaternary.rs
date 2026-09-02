//! The 4-body Many-Body Expansion (MBE4-1) evaluation machinery for (O, H, H, H).
//!
//! # What this is for
//!
//! Under pure MBE3, a third hydrogen binds to water along the C2v bisector. The true
//! ab-initio 4-body term:
//! ```text
//! dE4(O, H1, H2, H3) = E_FCI(OH3) - E_MBE3(OH3)
//! ```
//! corrects the total energy. The four-body term changes sign across configuration space,
//! reaching +0.2755 Ha and becoming attractive on 11 of 40 held-out geometries.
//!
//! Per the maximal holon ruling, NO fitted or empirical potential is admitted.
//! All values are evaluated directly from the ab-initio electronic Hamiltonian (1,568 determinants)
//! or interpolated from certified tabulated surfaces.
//!
//! # This module is now an INSTANTIATION, not an implementation
//!
//! Every function below is `crate::cluster`'s species-generic machinery applied to the
//! single Z-tuple [`OHHH`]. The arithmetic — six pair excesses in hub-and-cycle order,
//! four three-body terms addressed by sorted Z-triple, `3(N-1)` seeded dual solves with
//! the first slot's gradient row imposed by translation invariance — lives there and is
//! written once. What lives HERE is the tuple, the two tables that serve its two triple
//! classes, and the signatures the existing callers already hold.
//!
//! Every public signature in this module is unchanged, and
//! `tests/mbe_generic_identity.rs` grades the generic path against a frozen verbatim copy
//! of what used to be written out here — energy, all twelve gradient components, the full
//! CI vector, iteration count and residual — with `assert_eq!` on `to_bits()`. The
//! generalisation is required to be free, and the test is the receipt that it was.

use crate::cluster::{
    cluster_de4, cluster_fci_energy, cluster_fci_grad, cluster_mbe3_energy, ClusterFciGrad,
    SurfaceRegistry,
};
use crate::elements::{Species, HYDROGEN, OXYGEN};
use crate::trimer::TrimerTable;
use crate::water::WaterTable;

/// Measured far-field cutoff (bohr) where dE4 decays below T1 interpolation tolerance (~5e-5 Ha).
pub const R_CUT: f64 = 6.0;

/// THE INSTANCE. Everything below is `crate::cluster`'s general four-cluster machinery
/// applied to this one Z-tuple; nothing below re-derives arithmetic. When a second
/// four-cluster is wanted, it is this constant that changes and nothing else.
pub const OHHH: [Species; 4] = [OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN];

/// Evaluates E_FCI for (O, H, H, H) in STO-3G minimal basis (1,568 determinants).
pub fn ohhh_fci_energy(centers: &[[f64; 3]; 4]) -> f64 {
    cluster_fci_energy(&OHHH, centers)
}

/// Evaluates E_MBE3 for the 4-atom system (O, H, H, H): 6 pairs + 4 triples + isolated atoms.
///
/// The two tables ARE the registry: `(O, H, H)` serves the three triples containing the
/// oxygen and `(H, H, H)` the one that does not, and which triple gets which is decided
/// by the triple's sorted Z-tuple rather than by its slot indices. The `expect` cannot
/// fire for this signature — a caller who has both tables in hand has, by construction,
/// registered a family for both of an OHHH cluster's two triple classes.
pub fn ohhh_mbe3_energy(
    centers: &[[f64; 3]; 4],
    water_table: &WaterTable,
    trimer_table: &TrimerTable,
) -> f64 {
    let surfaces = SurfaceRegistry::new().with(water_table).with(trimer_table);
    cluster_mbe3_energy(&OHHH, centers, &surfaces)
        .expect("(O,H,H) and (H,H,H) cover every triple of an OHHH cluster")
}

/// The isolated-atom energies of this model, computed ONCE. They are constants of the
/// level of theory, and the old path re-solved them ab initio on every MBE3 evaluation —
/// two full FCI solves per call buying two numbers that never change.
///
/// Both are now the Z-keyed memo in `crate::cluster` rather than a hand-written
/// `OnceLock` apiece: `atom_energy` is a pure function of the `Species` record, so the
/// value is bit-for-bit what the two statics held, and a third element costs a table
/// entry instead of a third function.
pub fn atom_energy_o() -> f64 {
    crate::cluster::atom_energy_cached(OXYGEN)
}
pub fn atom_energy_h() -> f64 {
    crate::cluster::atom_energy_cached(HYDROGEN)
}

/// The FCI half of the four-body term with its EXACT Cartesian gradient.
///
/// An alias rather than a struct: the object is `crate::cluster::ClusterFciGrad<4>` and
/// the OHHH path is one instance of it. Every field, and every field's meaning, is
/// unchanged — the oxygen row is still minus the sum of the hydrogen rows BY
/// CONSTRUCTION (translation invariance), so the force sum over the quadruple is exactly
/// zero in floating point, not approximately.
pub type OhhhFciGrad = ClusterFciGrad<4>;

/// E_FCI(OH3) and its exact Cartesian gradient in nine seeded dual solves.
///
/// # Why this shape, and what it replaced
///
/// The runtime force path used to take FOUR value-only solves per recompute (base plus a
/// forward difference along each O-H radial), each of which ALSO re-solved two isolated
/// atoms and six pair diatomics that are constants and loaded tables respectively —
/// 36 FCI solves per recompute, 4 of them physics, buying HALF a gradient (the radial
/// projection) with O(h) error and, as landed, a broken momentum ledger.
///
/// This is the compressed object: one seeded dual solve per (hydrogen, axis) gives the
/// EXACT directional derivative through the same forward-mode machinery `pair_point`
/// has always used (the value slot is identical across the nine, so the first solve's
/// CI vector warm-starts the other eight, and the caller's per-hub cache warm-starts
/// the first). The oxygen gradient is imposed by translation invariance rather than
/// solved for: E(x+t) = E(x) exactly, so grad_O = -(grad_H1 + grad_H2 + grad_H3), and
/// the quadruple's force sum is zero to the last bit.
///
/// No finite-difference step, no radial projection, no mass in sight: the caller gets
/// -grad as FORCES and applies them raw.
pub fn ohhh_fci_grad(centers: &[[f64; 3]; 4], warm: Option<&[f64]>) -> OhhhFciGrad {
    cluster_fci_grad(&OHHH, centers, warm)
}

/// Exact ab-initio 4-body term dE4 = E_FCI - E_MBE3 from Cartesian coordinates.
pub fn de4_ohhh_fci(
    centers: &[[f64; 3]; 4],
    water_table: &WaterTable,
    trimer_table: &TrimerTable,
) -> f64 {
    let surfaces = SurfaceRegistry::new().with(water_table).with(trimer_table);
    cluster_de4(&OHHH, centers, &surfaces)
        .expect("(O,H,H) and (H,H,H) cover every triple of an OHHH cluster")
}

/// Six-coordinate S3 permutation sort for internal coordinate evaluation.
pub fn sort_ohhh_internals(
    r1: f64, r2: f64, r3: f64,
    r12: f64, r23: f64, r31: f64,
) -> ([f64; 3], [f64; 3]) {
    let mut roh = [r1, r2, r3];
    roh.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let mut rhh = [r12, r23, r31];
    rhh.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    (roh, rhh)
}
