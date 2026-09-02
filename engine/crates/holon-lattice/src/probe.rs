//! Leg B, probed BY CONSTRUCTION rather than by trajectory coincidence.
//!
//! The closure census's observed-fiber pairing collects frames `(s,t)` with `v_s = v_t` and
//! asks whether the next frames still agree. On a moving lattice gas the coarse view
//! essentially never repeats between frames, so that pairing would return "no witness found"
//! — a vacuous pass. M-FIXED-POINT-TRAJECTORY says the same thing from the other side and
//! instructs staking closure over CONFIGURATIONS instead of over one orbit. So the fiber is
//! built directly:
//!
//! > given `x`, replace one cell's state by its cyclic successor within its own `(N,P)`
//! > fiber. The label is unchanged, so `v_b(y) = v_b(x)` for **every** `b` at once.
//!
//! One identical perturbation therefore serves the whole chart family, and no confound
//! enters the defect curve from the probe changing with `b`.
//!
//! Both controls are required and neither is optional. The NEGATIVE control is `y = x`,
//! which must give a witness rate of exactly zero: a probe that fires on nothing is
//! measuring itself. The POSITIVE control moves the cell into a DIFFERENT fiber, which
//! changes the chart by construction and must fire (M-PLANT-OBS, M-BASE-RATE-OMITTED).

use crate::chart::{BlockChart, Field};
use crate::lattice::Lattice;

/// What a probe perturbs a cell into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    /// The fiber move: same `(N,P)`, different local state. The measurement.
    Fiber,
    /// No change at all. Negative control; must never produce a witness.
    None,
    /// Into a different fiber, so the chart moves by construction. Positive control.
    CrossFiber,
}

/// Which perturbations the probe ranges over. **The two answer different questions and the
/// prereg's G7 is staked on the first**, because G7 requires agreement with the frozen
/// reference and the reference's population is every `(position, movable state)` pair.
#[derive(Clone, Copy, Debug)]
pub enum Population {
    /// Every non-solid cell × every state a fiber move can act on, each planted into a copy
    /// of the base configuration. This is a statement about the CHART and the LATTICE, and
    /// it is exact: no sampling noise, so `EXACT` in G7 is meetable.
    Exhaustive,
    /// The given cells × every movable state. Used for the inhomogeneity discharge, where
    /// the reading must cover only the blocks that do not touch the wall — blocks that DO
    /// touch it are reported separately and never averaged into the curve.
    ExhaustiveOn,
    /// The lattice's OWN movable cells, **each exactly once**, keeping their own seeded
    /// states. This is the as-configured reading. It is exact for the configuration it
    /// runs on, and it differs from `Exhaustive` only in which positions the seed happened
    /// to put a movable state at — so it is compared to the derived law within a band whose
    /// `n` is the number of those cells.
    ///
    /// **Two sampler defects were found here and both are the reason this variant draws
    /// nothing.** The first picked a random index and scanned FORWARD to the next movable
    /// cell, which over-samples cells following a run of unmovable ones and so biases the
    /// POSITION within the block — the one quantity the defect law is about. It read 0.7025
    /// against a derived 0.75, and exceeded the bound at another size: a sampler defect
    /// wearing a physics result's clothes. The second drew 20,000 times WITH REPLACEMENT
    /// from ~300 distinct movable cells and quoted a binomial band on 20,000, which
    /// understates the real uncertainty by the ratio of those two numbers and turned a
    /// 0.65-sigma agreement into a 5-sigma disagreement. An enumerated count is not an
    /// effective count; here the effective count is the number of distinct cells, so the
    /// variant enumerates them and the band is computed on that.
    AsConfigured,
}

/// One probe campaign's reading at a fixed `(chart, steps)`.
#[derive(Clone, Debug)]
pub struct Reading {
    pub b: usize,
    pub steps: usize,
    pub probes: u64,
    pub witnesses: u64,
    /// The first witness pair found, kept so the results document can EXHIBIT one rather
    /// than assert that one exists.
    pub exhibit: Option<Witness>,
}

impl Reading {
    pub fn rate(&self) -> f64 {
        if self.probes == 0 {
            f64::NAN
        } else {
            self.witnesses as f64 / self.probes as f64
        }
    }

    /// One binomial standard error on the rate, over the probes ACTUALLY DISTINCT — the
    /// band an `AsConfigured` reading is compared within, and which an `Exhaustive` reading
    /// does not need because it has no sampling in it at all.
    pub fn stderr(&self) -> f64 {
        if self.probes == 0 {
            return f64::NAN;
        }
        let p = self.rate();
        (p * (1.0 - p) / self.probes as f64).sqrt()
    }
}

/// An exhibited pair of micro-states agreeing under the chart and disagreeing after the
/// motion — the `¬Closed` equivalence, witnessed rather than argued.
#[derive(Clone, Debug)]
pub struct Witness {
    pub cell: usize,
    pub state_before: u8,
    pub state_after: u8,
    pub agreed_view: Field,
    pub stepped_x: Field,
    pub stepped_y: Field,
}

/// Probe `v_b` for fiber invariance, advancing each perturbed pair `steps` applications of
/// the tier's own motion.
///
/// `x` and `y` differ in exactly one cell and nothing else, so the perturbation is the only
/// input that moves.
pub fn probe(
    base: &Lattice,
    chart: BlockChart,
    steps: usize,
    population: Population,
    mv: Move,
) -> Reading {
    probe_on(base, chart, steps, population, mv, &[])
}

/// As [`probe`], with the explicit cell list [`Population::ExhaustiveOn`] ranges over.
pub fn probe_on(
    base: &Lattice,
    chart: BlockChart,
    steps: usize,
    population: Population,
    mv: Move,
    only: &[usize],
) -> Reading {
    let movable = base.model.movable();
    let cells: Vec<usize> = (0..base.cells.len()).filter(|&c| !base.solid[c]).collect();
    let seeded_movable: Vec<usize> =
        cells.iter().copied().filter(|&c| movable.contains(&base.cells[c])).collect();

    let mut r = Reading { b: chart.b, steps, probes: 0, witnesses: 0, exhibit: None };

    let plan: Vec<(usize, u8)> = match population {
        Population::Exhaustive => {
            cells.iter().flat_map(|&c| movable.iter().map(move |&s| (c, s))).collect()
        }
        Population::ExhaustiveOn => {
            only.iter().flat_map(|&c| movable.iter().map(move |&s| (c, s))).collect()
        }
        Population::AsConfigured => {
            seeded_movable.iter().map(|&c| (c, base.cells[c])).collect()
        }
    };

    // Buffers, allocated once for the whole campaign rather than per probe. An earlier
    // version cloned the whole `Lattice` twice per probe, which copies the neighbour table
    // — 98 KB at L = 64 — 160,000 times to move one byte.
    let ncells = base.cells.len();
    let (mut x, mut y) = (vec![0u8; ncells], vec![0u8; ncells]);
    let (mut sx, mut sy) = (vec![0u8; ncells], vec![0u8; ncells]);

    for (cell, before) in plan {
        let after = match mv {
            Move::Fiber => base.model.fiber_successor(before).expect("state was chosen movable"),
            Move::None => before,
            Move::CrossFiber => base.model.other_fiber_state(before).expect("more than one fiber"),
        };

        x.copy_from_slice(&base.cells);
        y.copy_from_slice(&base.cells);
        x[cell] = before;
        y[cell] = after;

        let agreed = chart.apply(&base.model, &x);
        let agreed_y = chart.apply(&base.model, &y);
        match mv {
            // The measurement and the negative control BOTH leave the chart fixed; that is
            // what makes them a fiber. If this ever fails the probe is not probing a fiber.
            Move::Fiber | Move::None => assert_eq!(agreed, agreed_y, "the move left the fiber"),
            Move::CrossFiber => {
                assert_ne!(agreed, agreed_y, "the cross-fiber control did not move the chart")
            }
        }

        for k in 0..steps {
            base.advance(&mut x, &mut sx, k as u64);
            base.advance(&mut y, &mut sy, k as u64);
            core::mem::swap(&mut x, &mut sx);
            core::mem::swap(&mut y, &mut sy);
        }
        let (vx, vy) = (chart.apply(&base.model, &x), chart.apply(&base.model, &y));
        r.probes += 1;
        if vx != vy {
            r.witnesses += 1;
            if r.exhibit.is_none() {
                r.exhibit = Some(Witness {
                    cell,
                    state_before: before,
                    state_after: after,
                    agreed_view: agreed,
                    stepped_x: vx,
                    stepped_y: vy,
                });
            }
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Model;

    fn base(l: usize) -> Lattice {
        let m = Model::fhp6();
        let c = m.fhp_i(true);
        Lattice::seeded(m, l, 0xF1BE, 0.35, c)
    }

    /// G7: the measured curve at one step IS the derived closed form, at every `b`, EXACTLY.
    /// Exhaustive, because that is the reference's population and the only one on which an
    /// exact equality is meetable.
    #[test]
    fn the_one_step_defect_is_the_blocks_boundary_fraction() {
        let g = base(16);
        for b in [1usize, 2, 4, 8, 16] {
            let chart = BlockChart::new(b, 16).unwrap();
            let r = probe(&g, chart, 1, Population::Exhaustive, Move::Fiber);
            assert_eq!(r.probes, 16 * 16 * 20, "b={b}: the probe did not cover its population");
            let predicted = BlockChart::predicted_witness_rate(b, 16);
            assert!(
                (r.rate() - predicted).abs() < 1e-12,
                "b={b}: measured {} vs derived {predicted}",
                r.rate()
            );
        }
    }

    /// The as-configured reading agrees with the derivation within its OWN band — the band
    /// computed on the DISTINCT cells it read, which is the whole lesson of this variant's
    /// second defect. Reported with a band and never as an exact equality.
    #[test]
    fn the_as_configured_reading_agrees_within_its_own_band() {
        let g = base(32);
        for b in [2usize, 4, 8, 16] {
            let chart = BlockChart::new(b, 32).unwrap();
            let r = probe(&g, chart, 1, Population::AsConfigured, Move::Fiber);
            let predicted = BlockChart::predicted_witness_rate(b, 32);
            assert!(r.probes > 100, "b={b}: only {} movable cells to read", r.probes);
            // A saturated estimate has zero binomial variance, so the band needs a floor:
            // 1/n is the usual rule-of-three-shaped stand-in for "no counts in the tail".
            // Without it a reading that is EXACTLY right fails for having been too certain.
            let band = 4.0 * r.stderr().max(1.0 / r.probes as f64);
            assert!(
                (r.rate() - predicted).abs() <= band,
                "b={b}: as-configured {} +- {} over {} distinct cells vs derived {predicted}",
                r.rate(), band / 4.0, r.probes
            );
        }
    }

    /// V3: the exhaustive rate may never EXCEED the geometric bound. Doing so would mean
    /// information moved further than one cell in one step — an instrument defect, not a
    /// physics result, and it VOIDs rather than kills.
    #[test]
    fn the_defect_never_exceeds_its_geometric_bound() {
        let g = base(16);
        for b in [1usize, 2, 4, 8, 16] {
            let chart = BlockChart::new(b, 16).unwrap();
            let r = probe(&g, chart, 1, Population::Exhaustive, Move::Fiber);
            assert!(r.rate() <= BlockChart::predicted_witness_rate(b, 16) + 1e-12, "b={b}");
        }
    }

    /// G10, both sides. The probe must be shown able to return 0 and 1 on one instrument.
    #[test]
    fn the_probe_is_gauged_in_both_directions() {
        let g = base(16);
        let neg = probe(&g, BlockChart::new(4, 16).unwrap(), 1, Population::Exhaustive, Move::None);
        assert_eq!(neg.witnesses, 0, "the probe fired on an unperturbed pair");
        assert!(neg.probes > 0);
        let pos =
            probe(&g, BlockChart::new(1, 16).unwrap(), 1, Population::Exhaustive, Move::CrossFiber);
        assert_eq!(pos.rate(), 1.0, "the probe did not fire on a chart-moving perturbation");
    }

    /// G8: a witness pair is EXHIBITED, in the exact `¬Closed` sense — two micro-states
    /// agreeing under the chart whose images disagree.
    #[test]
    fn a_witness_pair_is_exhibited_not_asserted() {
        let g = base(16);
        let r = probe(&g, BlockChart::new(4, 16).unwrap(), 1, Population::Exhaustive, Move::Fiber);
        let w = r.exhibit.expect("no witness pair exhibited at b=4");
        assert_ne!(w.state_before, w.state_after);
        assert_eq!(g.model.label(w.state_before), g.model.label(w.state_after));
        assert_ne!(w.stepped_x, w.stepped_y);
        // The disagreement is a REDISTRIBUTION, never a violation: the two stepped views
        // differ block by block and agree on every total. That is what makes the witness a
        // statement about the CHART rather than about conservation.
        let sum = |f: &Field| {
            f.iter().fold([0i64; 3], |mut a, v| {
                for k in 0..3 {
                    a[k] += v[k];
                }
                a
            })
        };
        assert_eq!(sum(&w.stepped_x), sum(&w.stepped_y), "the witness broke a conservation law");
    }

    /// The global chart returns exactly zero — and it is the VACUOUS end, not a success.
    #[test]
    fn the_global_chart_closes_and_is_labelled_vacuous() {
        let g = base(16);
        let chart = BlockChart::new(16, 16).unwrap();
        assert!(chart.is_vacuous_by_conservation());
        let r = probe(&g, chart, 8, Population::Exhaustive, Move::Fiber);
        assert_eq!(r.witnesses, 0);
        assert!(r.probes > 0, "the vacuous end passed on zero work");
    }

    /// The defect grows with the number of steps as the light cone crosses the block, which
    /// is what distinguishes a boundary effect from a chart that is actually closed.
    #[test]
    fn the_defect_grows_as_the_light_cone_crosses_the_block() {
        let g = base(32);
        let chart = BlockChart::new(16, 32).unwrap();
        let rates: Vec<f64> = [1usize, 2, 4, 8, 16]
            .iter()
            .map(|&k| {
                probe(&g, chart, k, Population::AsConfigured, Move::Fiber).rate()
            })
            .collect();
        for w in rates.windows(2) {
            assert!(w[1] >= w[0] - 1e-12, "the defect fell as steps grew: {rates:?}");
        }
        assert!(*rates.last().unwrap() > 0.9, "the light cone never crossed: {rates:?}");
    }
}
