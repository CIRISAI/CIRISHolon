//! The price: measured on the first steps, WRITTEN before the counted ones, checked in a
//! band.
//!
//! # What it declares
//!
//! A campaign that does not know what it costs cannot be scheduled and cannot be believed
//! when it says a run finished. LIQUID-1 measures its price on the first hundred frames and
//! writes `price.json` before the counted ones; FLUID-1 does the same on two hundred steps
//! per run and gates the actual against it in a band. Both do it by convention.
//!
//! **Here it is a type.** [`Price::write`] is the only way to obtain a [`Priced`], and
//! [`Priced`] is the token the counted phase takes. A runner that counts before it prices
//! cannot be written: there is nothing to pass. The band is checked afterwards with
//! [`Priced::check`], which returns a [`Gate`] like everything else.
//!
//! # Why the band and not a number
//!
//! A price is a projection from a short measurement and it will not be exact. The band says
//! how wrong it may be before the projection is telling you something — a run four times
//! its own price is not slow, it is a different run. FLUID-1's band is `[0.1, 10]` on the
//! ratio; the band is a parameter here and never a default, because it is a decision.
//!
//! # What it refuses
//!
//! * Counting before pricing: [`Priced`] cannot be constructed except by writing.
//! * A price measured on zero steps, or over zero seconds: [`Price::measure`] returns
//!   `None`, because a rate with a zero denominator is not a rate (M-VACUOUS-SUCCESS wearing
//!   a stopwatch).
//! * Projecting from a measurement made with the wrong thing running: the label is carried
//!   and printed, and a caller that measures a warm-up and projects an arm has to say so in
//!   it.

use crate::gate::Gate;
use crate::record::{Record, RecordWriter, WriteRefusal};

/// What a run is expected to cost, measured on its own first steps.
#[derive(Clone, Debug, PartialEq)]
pub struct Price {
    /// What was measured, in the runner's own words.
    pub label: String,
    /// How many steps the measurement covered.
    pub measured_on: u64,
    /// How long those steps took.
    pub seconds_measured: f64,
    /// The rate: seconds per step.
    pub seconds_per_step: f64,
    /// How many steps the counted phase will take.
    pub steps_projected: u64,
    /// The projection: `seconds_per_step * steps_projected`.
    pub seconds_projected: f64,
    /// The band the actual/projected ratio must land in.
    pub band: (f64, f64),
}

impl Price {
    /// Measure a price. `None` on a zero step count or a non-positive elapsed time: a rate
    /// needs a denominator.
    // The negated comparison is DELIBERATE: a `NaN` elapsed time must be REFUSED, and
    // `NaN <= 0.0` is false while `!(NaN > 0.0)` is true.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn measure(
        label: impl Into<String>,
        measured_on: u64,
        seconds_measured: f64,
        steps_projected: u64,
        band: (f64, f64),
    ) -> Option<Price> {
        if measured_on == 0 || !(seconds_measured > 0.0) || !seconds_measured.is_finite() {
            return None;
        }
        let seconds_per_step = seconds_measured / measured_on as f64;
        Some(Price {
            label: label.into(),
            measured_on,
            seconds_measured,
            seconds_per_step,
            steps_projected,
            seconds_projected: seconds_per_step * steps_projected as f64,
            band,
        })
    }

    pub fn record(&self) -> Record {
        Record::new("price")
            .flag("written_before_the_counted_steps", true)
            .text("label", &self.label)
            .int("measured_on_steps", self.measured_on as i64)
            .number("seconds_measured", self.seconds_measured)
            .number("seconds_per_step", self.seconds_per_step)
            .int("steps_projected", self.steps_projected as i64)
            .number("seconds_projected", self.seconds_projected)
            .number("band_low", self.band.0)
            .number("band_high", self.band.1)
    }

    /// Write the price, and get back the token the counted phase needs.
    ///
    /// This is the ordering rule as a type: a [`Priced`] exists only downstream of a file on
    /// the disk.
    pub fn write(self, w: &RecordWriter, name: &str) -> Result<Priced, WriteRefusal> {
        let record = self.record();
        w.write(name, &record)?;
        Ok(Priced { price: self })
    }

    pub fn print(&self) -> String {
        format!(
            "price [{}]: {:.6e} s over {} steps = {:.6e} s/step; {} steps projected at \
             {:.3} s; band [{}, {}] on actual/projected",
            self.label,
            self.seconds_measured,
            self.measured_on,
            self.seconds_per_step,
            self.steps_projected,
            self.seconds_projected,
            self.band.0,
            self.band.1
        )
    }
}

/// A price that has been written. The counted phase takes one of these.
#[derive(Clone, Debug, PartialEq)]
pub struct Priced {
    price: Price,
}

impl Priced {
    pub fn price(&self) -> &Price {
        &self.price
    }

    /// The ratio of what it actually cost to what it was priced at.
    pub fn ratio(&self, actual_seconds: f64) -> f64 {
        if self.price.seconds_projected > 0.0 {
            actual_seconds / self.price.seconds_projected
        } else {
            f64::NAN
        }
    }

    /// The band check, as a gate.
    pub fn check(&self, actual_seconds: f64) -> Gate {
        let r = self.ratio(actual_seconds);
        let (lo, hi) = self.price.band;
        Gate::new(format!("price {}", self.price.label))
            .work(1)
            .leg_at(
                format!("actual/projected within [{lo}, {hi}]"),
                r.is_finite() && r >= lo && r <= hi,
                r,
            )
            .detail(format!(
                "{:.3} s actual against {:.3} s projected from {} steps at {:.6e} s/step",
                actual_seconds,
                self.price.seconds_projected,
                self.price.measured_on,
                self.price.seconds_per_step
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate::Verdict;
    use crate::record::{read_record, RecordWriter};
    use std::path::PathBuf;

    fn dir(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("holon-campaign-price-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn a_price_needs_a_denominator() {
        assert!(Price::measure("nothing", 0, 1.0, 10, (0.1, 10.0)).is_none());
        assert!(Price::measure("instant", 100, 0.0, 10, (0.1, 10.0)).is_none());
        assert!(Price::measure("real", 200, 2.0, 100_000, (0.1, 10.0)).is_some());
    }

    #[test]
    fn the_price_is_written_before_the_token_exists() {
        let d = dir("write");
        let w = RecordWriter::new(&d);
        let p = Price::measure("the 128-water arm", 100, 23.0, 102_000, (0.1, 10.0)).unwrap();
        assert!((p.seconds_per_step - 0.23).abs() < 1e-12);
        let priced = p.write(&w, "price.json").expect("the price is written");
        assert!(d.join("price.json").exists(), "the file is there before the run is");
        let back = read_record(d.join("price.json")).unwrap();
        assert!(back.count().is_ok());
        assert!(back.text.contains("written_before_the_counted_steps"));
        assert!((priced.price().seconds_projected - 23_460.0).abs() < 1e-6);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn the_band_check_passes_inside_and_fails_outside() {
        let d = dir("band");
        let w = RecordWriter::new(&d);
        let priced = Price::measure("a run", 200, 2.0, 20_000, (0.1, 10.0))
            .unwrap()
            .write(&w, "price.json")
            .unwrap();
        // projected: 0.01 s/step * 20000 = 200 s
        assert_eq!(priced.check(210.0).verdict(), Verdict::Pass);
        assert_eq!(priced.check(2_500.0).verdict(), Verdict::Fail, "12.5x is a different run");
        assert_eq!(priced.check(1.0).verdict(), Verdict::Fail, "0.005x is a different run too");
        let g = priced.check(2_500.0);
        assert_eq!(g.failing_legs().len(), 1);
        assert!((g.failing_legs()[0].value.unwrap() - 12.5).abs() < 1e-9);
        let _ = std::fs::remove_dir_all(&d);
    }
}
