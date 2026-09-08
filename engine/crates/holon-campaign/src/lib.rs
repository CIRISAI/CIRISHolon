//! The campaign harness — so a delegate builds only the physics.
//!
//! # What this is
//!
//! Every campaign in this repo does the same five things around its measurement: it gates,
//! it plants, it prices, it stakes, and it writes records somebody else has to be able to
//! read. Each runner has built all five by hand, and the failures have been in the harness
//! and not in the physics: a gate that returned at its first violation and understated a
//! two-class abyss as a one-kT dip (CT-1); a plant whose carrier was exactly zero in the
//! sector it acted on, voiding eight arms (FIELD-2); two plants whose stakes were
//! unreachable by the freeze's own arithmetic before a single step ran (FLUID-1); results
//! cited from paths that resolved only inside the session that wrote them
//! (M-STALE-INSTRUMENT).
//!
//! This crate is those five, with each of those failures made unwritable.
//!
//! # What it declares
//!
//! | | |
//! |---|---|
//! | [`gate::Gate`] | names EVERY failing leg, worst value first; zero work is VOID; a branch is an answer |
//! | [`plant::Plant`] | carrier nonzero IN ITS SECTOR, and the observable's ANALYTIC REACH printed beside the stake — both checked with no measurement at all |
//! | [`price::Price`] | measured on the first steps and WRITTEN before the counted ones: [`price::Priced`] is the token the counted phase takes and only writing makes one |
//! | [`stake::Stake`] | derived from [`stake::ReadInput`]s with its arithmetic printed, or typed and labelled `kill_from_experiment` — and the record writer refuses an unlabelled typed one |
//! | [`record::RecordWriter`] | validates JSON on write, refuses any `{:+` output, stamps `dry` and the screen label, writes `.done` markers |
//! | [`record::Reading`] | reads a record back and REFUSES to count a screen's |
//! | [`uncertainty::inefficiency`] | the statistical inefficiency `g`, the effective sample count and the standard error a CORRELATED series admits — reported beside the seed spread, never instead of it |
//! | [`uncertainty::equilibration_start`] | Chodera 2016's automated equilibration detection: the discard point that maximises the effectively uncorrelated sample count |
//!
//! # What it refuses to be
//!
//! **It is not a physics library and it holds no constant.** Every number in a campaign is
//! the campaign's — its bands, its stakes, its floors, its seeds. What is here is the shape
//! those numbers move in and the checks that shape makes possible. A default band or a
//! default floor in this crate would be a decision taken away from the freeze that has to
//! own it, which is why [`price::Price::measure`] takes its band and [`plant::Plant::new`]
//! takes its floor.
//!
//! **It does not rewrite the existing runners.** `fluid1_lattice.rs`, `liquid1.rs` and
//! `ct2_harvest.rs` are the reference: they built these five by hand, they are what this
//! was read off, and they keep passing untouched. `examples/tiny_campaign.rs` is what a new
//! campaign looks like when it does not have to.
//!
//! # The shape of a runner
//!
//! ```text
//! gate  -> the checks that must hold before anything is counted, plus the plants'
//!          pre-checks, plus the price, written first
//! run   -> the counted steps, which take a `Priced`, ending in a `.done` marker
//! read  -> the readouts, refusing to count anything a screen wrote
//! ```
//!
//! `examples/tiny_campaign.rs` is that, whole, in under 150 lines.

pub mod gate;
pub mod plant;
pub mod price;
pub mod record;
pub mod stake;
/// Autocorrelation-aware uncertainty and Chodera 2016's equilibration rule.
pub mod uncertainty;

pub use gate::{Gate, Leg, Report, Verdict};
pub use plant::{Plant, PlantVerdict};
pub use price::{Price, Priced};
pub use record::{
    find_plus_flag, is_done, num, read_record, validate_json, CountRefusal, JsonRefusal, Reading,
    Record, RecordWriter, WriteRefusal,
};
pub use stake::{read_input, read_input_after, Provenance, ReadInput, ReadRefusal, Stake};
pub use uncertainty::{equilibration_start, inefficiency, Equilibration, Ineff};
