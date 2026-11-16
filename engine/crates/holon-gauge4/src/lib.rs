//! Quenched SU(3) lattice gauge theory in 3+1 dimensions.
//!
//! A STANDARD computation, done here by this repository's own code so that the ladder's
//! hadron tier can show a gluon field it computed rather than a drawing. Nothing in the
//! method is new, and every piece is credited where it is implemented:
//!
//! * the Wilson plaquette action on a periodic four-dimensional lattice ([`Lattice`]);
//! * the Cabibbo-Marinari heat bath through SU(2) subgroups, sampled by Kennedy and
//!   Pendleton's method, after Creutz, with overrelaxation (`update`);
//! * Lüscher's Wilson flow with his third-order Runge-Kutta integrator (`flow`);
//! * the clover action density, the clover topological charge density and the Polyakov
//!   loop (`observe`): gauge-invariant quantities only;
//! * a random gauge transformation ([`Lattice::gauge_transform`]), for the gate that says
//!   the observables do not change under one;
//! * the stored record a page reads ([`Record`]).
//!
//! QUENCHED: there are no quarks in the weight, so no sea quarks, and a configuration is one
//! sample of a Euclidean ensemble, not a moment in anything's life.
//!
//! THE SCALE. At `beta = 6.0` the lattice spacing is 0.0931(4) fm: `r0/a = 5.368` from the
//! interpolations of M. Guagnelli, R. Sommer and H. Wittig, Nucl. Phys. B 535, 389 (1998),
//! arXiv:hep-lat/9806005, eq. 2.18, and S. Necco and R. Sommer, Nucl. Phys. B 622, 328
//! (2002), arXiv:hep-lat/0108008, eq. 2.6 (they agree to four figures), with `r0 = 0.5 fm`
//! (R. Sommer, Nucl. Phys. B 411, 839 (1994), arXiv:hep-lat/9310022). The scale is theirs.
//!
//! Zero dependencies, std only; deterministic from its seeds on one thread.
pub mod flow;
pub mod lattice;
pub mod observe;
pub mod record;
pub mod rng;
pub mod su3;
pub mod update;

pub use lattice::{random_su3, Lattice};
pub use observe::PLANES;
pub use record::{Record, Snapshot};
pub use rng::Rng;
pub use su3::{C64, Su3};

/// The lattice spacing at `beta = 6.0`, in femtometres (see the crate's notes).
pub const A_FM_BETA_6: f64 = 0.0931;

/// The mean plaquette at `beta = 6.0` in a large volume: 0.5936846(39) on 32^4, from
/// G. Boyd, J. Engels, F. Karsch, E. Laermann, C. Legeland, M. Lütgemeier and B. Petersson,
/// "Thermodynamics of SU(3) lattice gauge theory", Nucl. Phys. B 469, 419 (1996),
/// arXiv:hep-lat/9602007, doi:10.1016/0550-3213(96)00170-8, table 5 (their `1 - P`); their
/// 16^4 value is 0.593678(24), table 4.
pub const PLAQUETTE_BETA_6_BOYD: (f64, f64) = (0.593_684_6, 0.000_003_9);
