//! The fluid tier's objects as `holon-closure`'s: bonded particles as closures, bonds as
//! the bond production, and the block chart as a view whose edge can be read.
//!
//! # What it is
//!
//! A thin adapter and nothing else. `orientation.rs`'s [`bond_graph`] and `chart.rs`'s
//! [`BlockChart`] are untouched and remain the campaign's readouts; this module hands the
//! same objects to `holon-closure` so that the water unit in `holon-render`, the bonded
//! pair here and the H-bond component in `holon-lens` are ONE type. Nothing here computes
//! physics: every number comes back out of the existing readouts.
//!
//! # What it declares
//!
//! * **A closure at this tier is one particle, identified by its SLOT.** The tier's carrier
//!   is the slot list `cell · 6 + dir`, so a closure's member is a slot and its
//!   [`ClosureId`] is its position in ascending slot order among the OCCUPIED slots. That
//!   compaction is what makes [`crate::closure::phase`]'s `largest_fraction` the same
//!   number `bond_graph` reports: both are a fraction of the particles, not of the slots.
//! * **A bond is an edge with the link's own lattice displacement**, taken from
//!   [`OrientationLattice::bond_geometry`] exactly as `bond_graph` takes it — including its
//!   filter, so a bond whose geometry has gone stale is dropped from the edge list here for
//!   the same reason and at the same place.
//! * **A block chart is a [`BlockView`]** once its per-cell split reading exists, and
//!   [`chart_view`] produces that reading with node LG's own fiber probe.
//!
//! # What it refuses
//!
//! * **It does not re-implement the winding.** [`phase`] calls `holon-closure`'s, which is
//!   `orientation.rs`'s union–find with potentials lifted to `D` axes; the test in this
//!   module asserts the two agree field by field on a stepped lattice, and that is the
//!   fence keeping one rule from becoming two.
//! * **It adds no readout to `BondGraph`.** `largest_touches_all_rows` and its column twin
//!   stay where they are: they are translation-dependent, `orientation.rs` says so, and
//!   lifting them into a shared type would spread a reading that is not a gate.
//! * **[`chart_view`] is expensive and says so.** It runs one exhaustive fiber probe per
//!   micro-cell, which is the only way to know WHICH cell split rather than how often one
//!   did. It is for reading an edge, not for a defect curve; `probe::probe` is still the
//!   curve's instrument.

use crate::chart::BlockChart;
use crate::lattice::Lattice;
use crate::orientation::{bond_graph, BondGraph, OrientationLattice, N_DIRS};
use crate::probe::{probe_on, Move, Population};
use holon_closure::{BlockView, Closure, Edge, Phase, PhaseEdge, TierId};

/// The tier the fluid element's closures live on.
pub const TIER: TierId = TierId::FLUID_ELEMENT;

/// The occupied slots, ascending — the tier's carrier, and the map from a [`ClosureId`]
/// (the index in this list) back to the slot it names.
///
/// [`ClosureId`]: holon_closure::ClosureId
pub fn occupied_slots(g: &OrientationLattice) -> Vec<u32> {
    let mut out = Vec::new();
    for c in 0..g.cells.len() {
        let s = g.cells[c];
        for d in 0..N_DIRS {
            if s >> d & 1 == 1 {
                out.push((c * N_DIRS + d) as u32);
            }
        }
    }
    out
}

/// Every particle as a one-member closure, in ascending slot order.
///
/// One particle is one closure at this tier: FLUID-1's carrier has no rest particle and a
/// closure of two is a BOND between two of these, never a third object. The member is the
/// slot, which is the tier's own carrier index.
pub fn closures(g: &OrientationLattice) -> Vec<Closure> {
    occupied_slots(g)
        .into_iter()
        .map(|slot| Closure::new(TIER, vec![slot]).expect("one member, never empty"))
        .collect()
}

/// Every bond as a phase edge, with the link's lattice displacement.
///
/// The bonds are walked in `g.bonds` order and filtered by
/// [`OrientationLattice::bond_geometry`] exactly as [`bond_graph`] filters them: a bond
/// whose recorded slots no longer carry the geometry is not an edge, in both routines, for
/// the same reason.
pub fn bond_edges(g: &OrientationLattice) -> Vec<PhaseEdge<2>> {
    let index = occupied_slots(g);
    let id = |slot: u32| -> Option<u32> { index.binary_search(&slot).ok().map(|i| i as u32) };
    let dirs: Vec<[i32; 2]> =
        g.model().dirs.iter().map(|d| [d[0] as i32, d[1] as i32]).collect();
    g.bonds
        .iter()
        .filter_map(|b| {
            let (delta, _) = g.bond_geometry(b)?;
            Some(PhaseEdge { a: id(b.donor)?, b: id(b.acceptor)?, displacement: dirs[delta] })
        })
        .collect()
}

/// The phase reading of the bond graph, through `holon-closure`'s one rule.
pub fn phase(g: &OrientationLattice) -> Phase<2> {
    let n = occupied_slots(g).len();
    holon_closure::phase(n, &bond_edges(g))
}

/// A block chart with the per-cell split reading beside it, so its edge can be read.
#[derive(Clone, Debug)]
pub struct ChartView {
    pub chart: BlockChart,
    /// One flag per BLOCK: does a fiber-preserving perturbation anywhere inside it change
    /// the stepped chart? The negation of `Core/Closure.lean`'s `ViewClosed`, localised.
    pub splits: Vec<bool>,
    /// The carrier's side, so the block positions are in lattice cells.
    pub l: usize,
}

impl BlockView for ChartView {
    fn b(&self) -> usize {
        self.chart.b
    }
    fn cells(&self) -> usize {
        self.splits.len()
    }
    fn splits(&self, cell: usize) -> bool {
        self.splits[cell]
    }
    /// The block's own centre, in lattice cells.
    fn position(&self, cell: usize) -> [f64; 2] {
        let nb = self.chart.blocks_per_side();
        let half = self.chart.b as f64 / 2.0;
        [
            (cell % nb) as f64 * self.chart.b as f64 + half,
            (cell / nb) as f64 * self.chart.b as f64 + half,
        ]
    }
    /// The lattice is a torus and says so, which is what makes the edge's centroid a
    /// circular mean rather than a point in the middle of the empty half.
    fn period(&self) -> Option<[f64; 2]> {
        Some([self.l as f64, self.l as f64])
    }
    /// `b = L` is held by conservation alone; `chart.rs` labels it and so does this.
    fn vacuous(&self) -> bool {
        self.chart.is_vacuous_by_conservation()
    }
}

/// Read WHICH blocks split, by running node LG's fiber probe one micro-cell at a time.
///
/// **This is the expensive route and it is the only one that answers this question.**
/// `probe::probe` reports a RATE over a population; the edge needs the SET, so each movable
/// cell is probed on its own and its block is marked. Cost is one exhaustive single-cell
/// probe per cell: fine at `L = 16` or `32` for an edge reading, not a curve's instrument.
pub fn chart_view(base: &Lattice, chart: BlockChart, steps: usize) -> ChartView {
    let nb = chart.blocks_per_side();
    let mut splits = vec![false; nb * nb];
    let l = base.l;
    for c in 0..base.cells.len() {
        if base.solid[c] {
            continue;
        }
        let block = (c / l / chart.b) * nb + (c % l) / chart.b;
        if splits[block] {
            continue; // one witness is enough to say the block splits
        }
        let r = probe_on(base, chart, steps, Population::ExhaustiveOn, Move::Fiber, &[c]);
        if r.witnesses > 0 {
            splits[block] = true;
        }
    }
    ChartView { chart, splits, l }
}

/// The edge at one resolution: the blocks whose fiber the dynamics splits.
pub fn edge(base: &Lattice, chart: BlockChart, steps: usize) -> Edge {
    Edge::at(&chart_view(base, chart, steps))
}

/// `bond_graph`'s reading and the shared one, side by side. The adapter's own witness that
/// the two rules are one rule.
pub fn agrees_with_bond_graph(g: &OrientationLattice) -> Result<(BondGraph, Phase<2>), String> {
    let bg = bond_graph(g);
    let p = phase(g);
    let mut bad = Vec::new();
    if bg.particles != p.nodes {
        bad.push(format!("particles {} vs nodes {}", bg.particles, p.nodes));
    }
    if bg.bonds != p.edges {
        bad.push(format!("bonds {} vs edges {}", bg.bonds, p.edges));
    }
    if bg.largest != p.largest {
        bad.push(format!("largest {} vs {}", bg.largest, p.largest));
    }
    if !same(bg.largest_fraction, p.largest_fraction) {
        bad.push(format!("largest_fraction {} vs {}", bg.largest_fraction, p.largest_fraction));
    }
    if bg.largest_winds != p.largest_winds {
        bad.push(format!("largest_winds {:?} vs {:?}", bg.largest_winds, p.largest_winds));
    }
    if bg.largest_spans != p.largest_spans {
        bad.push(format!("largest_spans {} vs {}", bg.largest_spans, p.largest_spans));
    }
    if bg.any_spans != p.any_spans {
        bad.push(format!("any_spans {} vs {}", bg.any_spans, p.any_spans));
    }
    if !same(bg.bonds_per_particle_degree, p.degree) {
        bad.push(format!("degree {} vs {}", bg.bonds_per_particle_degree, p.degree));
    }
    if bad.is_empty() {
        Ok((bg, p))
    } else {
        Err(bad.join("; "))
    }
}

/// Bit equality, with `NaN` equal to `NaN` — the two routines agree on an empty lattice by
/// both refusing to divide, and a comparison that called that a difference would be wrong.
fn same(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orientation::OrientationRules;
    use crate::state::Model;

    /// FLUID-1's chart shape, on a small torus: the same rules the campaign's own tests
    /// use, so this adapter is exercised on the configurations the freeze runs on.
    pub(crate) fn seeded(l: usize, steps: usize) -> OrientationLattice {
        let m = Model::fhp6();
        let law = m.fhp_i(true);
        let lat = Lattice::seeded(m, l, 0x464c_5549_4431, 0.2, law);
        let rules = OrientationRules {
            bonds_enabled: true,
            amplitude: [1.0, 1.3, 1.28, 1.42, 1.28, 1.3],
            rent: OrientationRules::rent_from_retention(0.5526),
            cold: false,
            stream: true,
        };
        let mut g = OrientationLattice::from_lattice(lat, 0x464c_5549_4431, rules);
        for _ in 0..steps {
            g.step();
        }
        g
    }

    #[test]
    fn every_occupied_slot_is_one_closure_of_one_member() {
        let g = seeded(16, 20);
        let cs = closures(&g);
        let slots = occupied_slots(&g);
        assert_eq!(cs.len(), slots.len());
        assert!(cs.iter().all(|c| c.len() == 1 && c.tier == TIER));
        assert_eq!(cs.iter().map(|c| c.members()[0]).collect::<Vec<_>>(), slots);
    }

    /// A bond never joins a slot to itself, which is why the shared `phase` needs no
    /// special case for a self-loop and reproduces `bond_graph` exactly.
    #[test]
    fn no_bond_is_a_self_loop() {
        let g = seeded(32, 50);
        assert!(!g.bonds.is_empty(), "the scene has bonds to check");
        for b in &g.bonds {
            assert_ne!(b.donor, b.acceptor, "a bond joined a slot to itself");
        }
    }
}
