//! The closure as an engine type — one object, under one name, for every tier that has one.
//!
//! # What this is
//!
//! `CIRISOntology/Core/Closure.lean` says what an object is: a lossy view `v` of a state
//! space that the dynamics `T` never splits (`ViewClosed v T`, equivalently
//! `viewClosed_iff_never_splits` — two states agreeing under `v` still agree after `T`).
//! Four consequences are machine-checked there and each is load-bearing here: the coarse
//! law is FORCED (`macro_law_forced`, so a closed view has exactly one dynamics and there
//! is nothing to choose), tiers COMPOSE (`viewClosed_comp`), conservation DESCENDS
//! (`conserved_descends`), and closure is a property of the pair `(v, T)` and never of `v`
//! alone.
//!
//! The engine already had that object in four places under four names — a water unit in
//! `holon-render`'s `units_reading`, a bonded pair in `holon-lattice`'s `bond_graph`, an
//! H-bond component in `holon-lens`'s `largest_domain`, a block-chart cell in
//! `holon-lattice`'s `chart`. This crate is the one type they now share, and the three
//! adapters that reach it are thin functions in those crates rather than logic moved out
//! of them: every existing readout is reproduced BIT FOR BIT by its adapter's test, and
//! not one frozen record moves.
//!
//! # What it declares
//!
//! 1. **A closure is its members, its tier, its ledger and its rent** ([`Closure`]). The
//!    members are indices into the tier's OWN carrier and are held ascending and unique,
//!    which is the invariant [`Closure::merge`] and [`Closure::part`] preserve exactly.
//!
//! 2. **The ledger has six rows and they are the six channels of GANTT2's vocabulary
//!    table**, in that order: presence, accommodation, attunement, concert, refusal,
//!    sharing ([`Channel`], [`CHANNELS`]). A row is a value AND a `served` flag
//!    ([`LedgerRow`]), and an unserved row cannot be read as a number — [`LedgerRow::read`]
//!    returns `None` and the stored value is `NaN`, so a channel nobody measured can never
//!    be summed into a total as a zero. That is the whole reason the flag exists: at the
//!    molecular tier accommodation and attunement READ ZERO AT THIS BASIS, which is a
//!    different fact from "not served", and the two must not print the same.
//!
//! 3. **Rent is `Core/Maintenance.lean`'s step, transcribed** ([`Rent`]): `S ↦ S − γS + α`.
//!    Paying `α = γS` holds the entry (`rent_holds`), paying less strictly loses
//!    (`underpaid_shrinks`), paying nothing tends to zero (`unpaid_decays`). `None` where
//!    the tier has no rent rule; a tier that has one names it.
//!
//! 4. **Three productions** ([`Production`]): unit (members → a closure, by a condition on
//!    the carrier), bond (two closures → a pair, by a condition on the ledger), rewrite (a
//!    closure changes composition, by a boundary crossed). A [`Grammar`] is a list of them
//!    with the conditions supplied by the tier.
//!
//! 5. **The edge at a resolution** ([`Edge`], [`BlockView`]): the set of cells whose fiber
//!    the dynamics splits, which is node LG's `closed_iff_fiber_invariant` read as a set
//!    rather than as a rate, and EDGE-2's instrument.
//!
//! 6. **The phase** ([`phase()`]): a bond graph's largest component and whether it winds.
//!
//! # What it refuses
//!
//! * **A closure with no members, or with a member listed twice.** An empty closure is a
//!   reading about nothing and a repeated member double-counts the carrier.
//! * **Merging closures from two tiers, or with a member in common.** Both are category
//!   errors and both are silent if allowed.
//! * **Inheriting a merged closure's ledger or rent from its parts.** [`Closure::merge`]
//!   returns a closure whose ledger is EMPTY and whose rent is `None`, and says so: the
//!   coarse law of a merged view is forced by the view (`macro_law_forced`), it is not the
//!   sum of the parts' laws, and a ledger carried up by addition would be a claim nobody
//!   measured. `merge` and `part` are exact inverses ON THE MEMBER SET and nowhere else,
//!   which is exactly what the test asserts.
//! * **Reading an unserved ledger row as a number** ([`LedgerRow::read`] is `Option`), and
//!   **totalling a ledger without saying how many rows were not served**
//!   ([`Ledger::served_total`] returns both).
//! * **Reporting an empty edge as a zero-width edge.** [`Edge`] carries `empty` and its
//!   width and position are `NaN`/`None` there: a chart at which nothing splits is a
//!   verdict about the chart (M-EMPTY-SECTOR), not a measurement of an edge.
//! * **Reporting a vacuous chart's edge as a result.** [`Edge::vacuous`] carries the
//!   `b = L` label through, exactly as `holon-lattice`'s `BlockChart` does, so a defect
//!   curve cannot quietly end in a success.
//! * **Naming a phase.** [`phase()`] reports the largest component and its windings and
//!   stops. "A spanning bond graph is a liquid" is a reading a tier makes with its own
//!   threshold, and a function that returned the word would be that threshold in hiding.

pub mod closure;
pub mod edge;
pub mod grammar;
pub mod phase;

pub use closure::{
    Channel, Closure, ClosureId, ClosureRefusal, Ledger, LedgerRow, MergeRefusal,
    NeighbourLedger, PartRefusal, Rent, RentVerdict, TierId, CHANNELS,
};
pub use edge::{BlockView, Edge};
pub use grammar::{
    BondReading, Crossing, Grammar, Production, ProductionKind, RewriteReading, UnitReading,
};
pub use phase::{phase, Phase, PhaseEdge};
