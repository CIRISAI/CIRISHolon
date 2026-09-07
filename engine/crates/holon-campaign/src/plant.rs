//! Plants: a deliberate change that MUST move the reading, checked before it is believed.
//!
//! # What it declares
//!
//! A plant is the positive control. It turns a knob the hypothesis says the observable
//! depends on and requires the observable to move by more than a staked amount; a plant
//! that does not fire convicts the instrument, not the physics.
//!
//! Two things have to be true before a plant means anything, and three freezes have now
//! failed on the second:
//!
//! 1. **The carrier is nonzero IN THE SECTOR THE PLANT ACTS ON** (M-PLANT-SECTOR). FIELD-2
//!    measured its binding at the bonded starts and read exactly `0.0`: no water unit
//!    existed there, so eight arms ran a field that was never on a bonded configuration.
//!    A carrier under its floor VOIDs the plant (M-EMPTY-SECTOR: a measured exact zero is a
//!    verdict about the assignment, never a value an expectation rule may consume).
//!
//! 2. **The observable's ANALYTIC REACH is computed and printed beside the stake.** FLUID-1
//!    staked that its plants would move the bond count by 20 % and read 8 %; the amplitude
//!    table can move the per-angle retention by at most a computable amount at that chart,
//!    and that amount was under the stake. The stake was unreachable by the freeze's own
//!    arithmetic — a design failure, discoverable BEFORE any compute. [`Plant::pre_check`]
//!    is exactly that check, and it needs no measurement to run.
//!
//! # What it refuses
//!
//! * A plant whose carrier is at or under its floor: VOID, named.
//! * A plant whose reach is under its stake: VOID, named "unreachable by the freeze's own
//!   arithmetic", with both numbers. It is not a FAIL, because nothing was measured — the
//!   design is what failed.
//! * A plant that has not run: [`PlantVerdict::NotRun`], never a pass.
//! * A stake with no reach beside it cannot be built: `reach` is not `Option`.

use crate::gate::Gate;
use crate::record::num;
use crate::stake::Stake;

/// A positive control: a knob turned, and how far the observable must move.
#[derive(Clone, Debug, PartialEq)]
pub struct Plant {
    pub name: String,
    /// The sector the plant acts on, named. The carrier below is the carrier IN THIS
    /// SECTOR, not the carrier overall.
    pub sector: String,
    /// The carrier's measured value in that sector.
    pub carrier: f64,
    /// Under this, the sector is empty and the plant acts on nothing.
    pub carrier_floor: f64,
    /// How far the observable must move for the plant to fire.
    pub stake: Stake,
    /// The most the knob CAN move the observable, computed from the freeze's own arithmetic
    /// before any run.
    pub reach: f64,
    /// How that reach was computed, printed beside it.
    pub reach_arithmetic: String,
    /// How far the observable actually moved. `None` before the plant has run.
    pub measured: Option<f64>,
}

/// What a plant says.
#[derive(Clone, Debug, PartialEq)]
pub enum PlantVerdict {
    /// The carrier is at or under its floor: the plant acts on an empty sector.
    CarrierEmpty { carrier: f64, floor: f64 },
    /// The observable cannot move as far as the stake, by the freeze's own arithmetic.
    StakeUnreachable { reach: f64, stake: f64 },
    /// Admissible, and not yet run.
    NotRun,
    /// Ran, and did not move far enough.
    Misses { measured: f64, stake: f64 },
    /// Ran, and fired.
    Fires { measured: f64, stake: f64 },
}

impl Plant {
    pub fn new(
        name: impl Into<String>,
        sector: impl Into<String>,
        carrier: f64,
        carrier_floor: f64,
        stake: Stake,
        reach: f64,
        reach_arithmetic: impl Into<String>,
    ) -> Plant {
        Plant {
            name: name.into(),
            sector: sector.into(),
            carrier,
            carrier_floor,
            stake,
            reach,
            reach_arithmetic: reach_arithmetic.into(),
            measured: None,
        }
    }

    /// Record what the plant actually moved the observable by.
    pub fn measured(mut self, change: f64) -> Plant {
        self.measured = Some(change);
        self
    }

    /// The two checks that need NO measurement, run in the order the misfits were found in.
    ///
    /// This is the whole value of the type: both failures below are visible before a single
    /// step of the arm, and both have cost a freeze.
    // The negated comparison is DELIBERATE and must not become `<=`: a `NaN` carrier has to
    // fall on the REFUSING side, and `NaN <= floor` is false while `!(NaN > floor)` is true.
    // A carrier nobody could measure is an empty sector, not an admissible one.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn pre_check(&self) -> Option<PlantVerdict> {
        if !(self.carrier.abs() > self.carrier_floor) {
            return Some(PlantVerdict::CarrierEmpty {
                carrier: self.carrier,
                floor: self.carrier_floor,
            });
        }
        if self.reach.abs() < self.stake.value.abs() {
            return Some(PlantVerdict::StakeUnreachable {
                reach: self.reach,
                stake: self.stake.value,
            });
        }
        None
    }

    pub fn verdict(&self) -> PlantVerdict {
        if let Some(v) = self.pre_check() {
            return v;
        }
        match self.measured {
            None => PlantVerdict::NotRun,
            Some(m) if m.abs() > self.stake.value.abs() => {
                PlantVerdict::Fires { measured: m, stake: self.stake.value }
            }
            Some(m) => PlantVerdict::Misses { measured: m, stake: self.stake.value },
        }
    }

    /// The plant as a gate, so it enters a report through the same door every other check
    /// does. A pre-check failure is VOID with the arithmetic in the reason; a miss is FAIL.
    pub fn gate(&self) -> Gate {
        let g = Gate::new(format!("plant {}", self.name)).detail(format!(
            "sector {:?}; carrier {:.6e} (floor {:.6e}); stake {:.6e}; analytic reach {:.6e} — {}",
            self.sector, self.carrier, self.carrier_floor, self.stake.value, self.reach,
            self.reach_arithmetic
        ));
        match self.verdict() {
            PlantVerdict::CarrierEmpty { carrier, floor } => g.work(1).void(format!(
                "the carrier in sector {:?} is {carrier:.6e}, at or under its floor {floor:.6e}: \
                 the plant acts on an empty sector and the arms it gates are VOID \
                 (M-PLANT-SECTOR, M-EMPTY-SECTOR)",
                self.sector
            )),
            PlantVerdict::StakeUnreachable { reach, stake } => g.work(1).void(format!(
                "THE STAKE IS UNREACHABLE BY THE FREEZE'S OWN ARITHMETIC, not by the measurement: \
                 the knob can move the observable by at most {reach:.6e}, under the {stake:.6e} \
                 staked. The carrier IS nonzero in the sector the plant acts on; it is the \
                 OBSERVABLE that cannot respond. Reach: {}",
                self.reach_arithmetic
            )),
            PlantVerdict::NotRun => g.void("the plant has not been run"),
            PlantVerdict::Misses { measured, stake } => g
                .work(1)
                .leg_at(format!("{} moves the reading by more than {stake:.6e}", self.name), false, measured),
            PlantVerdict::Fires { measured, stake } => g
                .work(1)
                .leg_at(format!("{} moves the reading by more than {stake:.6e}", self.name), true, measured),
        }
    }

    /// The console block: the carrier, the reach and the stake side by side, which is the
    /// arrangement that makes an unreachable stake visible at a glance.
    pub fn print(&self) -> String {
        let mut s = format!(
            "plant {}\n  sector          {}\n  carrier         {:.6e}  (floor {:.6e})\n  \
             analytic reach  {:.6e}  ({})\n  stake           {:.6e}\n  measured        {}",
            self.name,
            self.sector,
            self.carrier,
            self.carrier_floor,
            self.reach,
            self.reach_arithmetic,
            self.stake.value,
            match self.measured {
                Some(m) => format!("{m:.6e}"),
                None => "not run".to_string(),
            }
        );
        s.push_str(&format!("\n  stake provenance\n    {}", self.stake.print().replace('\n', "\n    ")));
        s.push_str(&format!("\n  verdict         {:?}", self.verdict()));
        s
    }

    pub fn json(&self) -> String {
        format!(
            "{{\"name\": {:?}, \"sector\": {:?}, \"carrier\": {}, \"carrier_floor\": {}, \
             \"analytic_reach\": {}, \"reach_arithmetic\": {:?}, \"measured\": {}, \
             \"stake\": {}, \"verdict\": {:?}}}",
            self.name,
            self.sector,
            num(self.carrier),
            num(self.carrier_floor),
            num(self.reach),
            self.reach_arithmetic,
            self.measured.map(num).unwrap_or_else(|| "null".to_string()),
            self.stake.json(),
            format!("{:?}", self.verdict())
        )
    }

    /// The stake this plant carries, for a record that wants to validate it.
    pub fn stake(&self) -> &Stake {
        &self.stake
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate::Verdict;
    use crate::stake::Stake;

    fn stake(v: f64) -> Stake {
        Stake::derived("the plant's stake", v, &[], "the freeze's own number, section 5")
    }

    /// M-PLANT-SECTOR / M-EMPTY-SECTOR: FIELD-2's shape, caught before any arm runs.
    #[test]
    fn an_empty_carrier_voids_the_plant_before_it_runs() {
        let p = Plant::new("the field at a bonded start", "bonded starts", 0.0, 0.05, stake(0.2), 1.0, "the whole range");
        assert_eq!(p.verdict(), PlantVerdict::CarrierEmpty { carrier: 0.0, floor: 0.05 });
        assert_eq!(p.gate().verdict(), Verdict::Void);
        assert!(p.gate().void_reason().unwrap().contains("empty sector"));
    }

    /// FLUID-1 P(i)/P(ii): the stake is unreachable by the freeze's own arithmetic, and it
    /// is visible with no measurement at all.
    #[test]
    fn a_stake_over_the_analytic_reach_voids_the_plant_with_no_measurement() {
        let p = Plant::new(
            "flatten the amplitude table",
            "the per-angle retention",
            0.42,
            0.20,
            stake(0.20),
            0.081,
            "E0/kT_lat = 0.35 makes a 42 % change in A an 8.1 % change in p_break",
        );
        assert!(p.measured.is_none(), "no measurement was needed");
        assert_eq!(p.verdict(), PlantVerdict::StakeUnreachable { reach: 0.081, stake: 0.20 });
        let g = p.gate();
        assert_eq!(g.verdict(), Verdict::Void);
        assert!(g.void_reason().unwrap().contains("UNREACHABLE BY THE FREEZE'S OWN ARITHMETIC"));
    }

    #[test]
    fn a_reachable_stake_runs_and_then_fires_or_misses() {
        let base = Plant::new("double the rent", "the bond census", 0.55, 0.05, stake(0.20), 0.9, "p_break doubles");
        assert_eq!(base.verdict(), PlantVerdict::NotRun);
        assert_eq!(base.gate().verdict(), Verdict::Void, "not run is never a pass");

        let fires = base.clone().measured(0.31);
        assert_eq!(fires.verdict(), PlantVerdict::Fires { measured: 0.31, stake: 0.20 });
        assert_eq!(fires.gate().verdict(), Verdict::Pass);

        let misses = base.measured(0.08);
        assert_eq!(misses.verdict(), PlantVerdict::Misses { measured: 0.08, stake: 0.20 });
        assert_eq!(misses.gate().verdict(), Verdict::Fail);
        assert_eq!(misses.gate().failing_legs().len(), 1);
    }

    /// The carrier is checked FIRST: an empty sector is not reported as an unreachable
    /// stake, because the two need different repairs.
    #[test]
    fn the_carrier_is_checked_before_the_reach() {
        let p = Plant::new("both wrong", "an empty sector", 0.0, 0.05, stake(1.0), 0.001, "nothing");
        assert!(matches!(p.verdict(), PlantVerdict::CarrierEmpty { .. }));
    }

    #[test]
    fn the_print_puts_the_reach_beside_the_stake() {
        let p = Plant::new("p", "s", 1.0, 0.1, stake(0.2), 0.5, "the arithmetic");
        let text = p.print();
        let reach_line = text.lines().position(|l| l.contains("analytic reach")).unwrap();
        let stake_line = text.lines().position(|l| l.trim_start().starts_with("stake  ")).unwrap();
        assert_eq!(stake_line, reach_line + 1, "the stake sits directly under the reach");
    }
}
