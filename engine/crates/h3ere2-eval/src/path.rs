//! The reasoning step: relax the encoder's perturbation through the coupling graph and
//! read off the order in which the disturbance reaches each kind.
//!
//! This is `field::arrival_step` generalised from one source to the encoder's whole block,
//! and it is deliberately the same question that function asks: K11 has no hop distance,
//! so locality here is METRIC — a disturbance reaches strongly-coupled kinds before
//! weakly-coupled ones even though the graph is dense. The arrival order is that metric,
//! made observable.
//!
//! **Harmonic regime only.** `Params::harmonic()` sets `rest_scale = 0`, and FSD §13 is
//! binding on this: the twin-decoupling theorem is about a LINEAR operator and does not
//! survive a spring law with rest lengths. Running this in the default regime would be
//! measuring a different object than the one the theorems are about.

use ciris_sim_core::data::{KINDS, N};
use ciris_sim_core::dynamics::{step, Params, State};
use ciris_sim_core::structure::Structure;

/// Where the disturbance reached, and when.
#[derive(Debug, Clone, PartialEq)]
pub struct Arrival {
    pub kind: &'static str,
    pub index: usize,
    /// `None` if the kind never moved past `threshold` within the budget.
    pub step: Option<usize>,
    /// True for the kinds the encoder seeded.
    pub seeded: bool,
}

#[derive(Debug, Clone)]
pub struct PathParams {
    pub amplitude: f64,
    pub threshold: f64,
    pub max_steps: usize,
    /// Integration step. Deliberately 10x finer than `Params::harmonic()`'s default.
    ///
    /// Arrival times are integer step counts, so a coarse `dt` makes the fast end of the
    /// path arrive within 3-5 steps, where small integers tie constantly. Every tie is
    /// then broken by canonical `KINDS` order -- which is not physics, and is identical
    /// in both arms, so it dilutes the comparison without biasing it. Measured share of
    /// adjacent orderings decided by a tie: 31% at dt=0.005, 21% at dt=0.0005, 21% at
    /// dt=0.00005 (the floor). Costs ~0.2ms per path against 160 tokens of generation.
    pub dt: f64,
}

impl Default for PathParams {
    fn default() -> Self {
        // amplitude 1.0 and threshold 1e-3 mirror the crate's own arrival tests; the
        // budget is generous enough that "never arrived" means the coupling is genuinely
        // weak rather than that we stopped early.
        PathParams { amplitude: 1.0, threshold: 1e-3, max_steps: 200_000, dt: 0.000_5 }
    }
}

/// Inject `amplitude` at every seeded kind, relax, and return every kind ordered by
/// arrival time. Seeded kinds are reported with `step = Some(0)`: they are where the
/// change landed, not somewhere it propagated to.
pub fn structure_for(coupling: &[[f64; N]; N]) -> Structure<N> {
    Structure::<N>::from_coupling(coupling, ciris_sim_core::data::TWINS)
}

/// Convenience wrapper. Prefer `relax` with a cached `Structure`: `from_coupling` runs a
/// full O(N^3) Jacobi eigensolve whose result this module never reads (in the harmonic
/// regime the metric is multiplied by `rest_scale = 0`, and spectrum and sectors are
/// never consulted), so rebuilding it per call is ~15us of pure waste.
pub fn relax_coupling(coupling: &[[f64; N]; N], seeds: &[usize], pp: &PathParams) -> Vec<Arrival> {
    relax(&structure_for(coupling), seeds, pp)
}

pub fn relax(
    st: &Structure<N>,
    seeds: &[usize],
    pp: &PathParams,
) -> Vec<Arrival> {
    let mut params = Params::harmonic();
    params.dt = pp.dt;
    // A `debug_assert` here would be compiled out of the release build we actually run,
    // i.e. a guard that does not guard in the shipped configuration.
    assert_eq!(params.rest_scale, 0.0, "FSD 13: the linear regime is binding here");

    let mut pos = [[0.0f64; 3]; N];
    for &s in seeds { pos[s][0] = pp.amplitude; }
    let mut state = State::at_rest(pos);

    let mut first: [Option<usize>; N] = [None; N];
    for &s in seeds { first[s] = Some(0); }

    let mut t = 0usize;
    while t < pp.max_steps && first.iter().any(|f| f.is_none()) {
        // `false` = the MEASURED coupling; `true` would be the Z2xZ2 group average.
        // The experiment asks whether the real object carries information, and the
        // average discards the measured twin defect (g_DB 2.284 vs 8.617), which is one
        // of the object's better-measured features. h3ere2 needs LINEARITY -- supplied by
        // `Params::harmonic()` -- and never invokes the twin-decoupling theorem, so there
        // is no reason to average. Running arm C on the average would test "does the
        // symmetrised coupling beat scrambled", which is a different question.
        step(&mut state, &st, &params, false);
        t += 1;
        for i in 0..N {
            if first[i].is_some() { continue; }
            let p = state.pos[i];
            let d = libm_sqrt(p[0] * p[0] + p[1] * p[1] + p[2] * p[2]);
            if d > pp.threshold { first[i] = Some(t); }
        }
    }

    let mut out: Vec<Arrival> = (0..N)
        .map(|i| Arrival {
            kind: KINDS[i],
            index: i,
            step: first[i],
            seeded: seeds.contains(&i),
        })
        .collect();
    // Arrived kinds first, in time order; never-arrived last, in canonical order. The
    // index tiebreak keeps the ordering total and deterministic.
    out.sort_by(|a, b| match (a.step, b.step) {
        (Some(x), Some(y)) => x.cmp(&y).then(a.index.cmp(&b.index)),
        (Some(_), None) => core::cmp::Ordering::Less,
        (None, Some(_)) => core::cmp::Ordering::Greater,
        (None, None) => a.index.cmp(&b.index),
    });
    out
}

/// A2 soft seeding: every block is seeded with amplitude proportional to its softmax
/// mass (within-block split uniform, exactly the hard rule's per-member amplitude scaled
/// by the block's mass). `block_mass` is in `blocks::SURFACES` order.
///
/// The `seeded` flag — the PRIMARY ASPECT the renderer names — marks the argmax block,
/// so the rendered prompt format is byte-compatible with the hard encoding. Arrival for
/// every other kind is the first step its position DEVIATES from its initial value by
/// `threshold`. A2 fixed the seeding and said "relax as before" but left the arrival
/// readout for mass-carrying kinds unspecified; deviation-from-initial is the one
/// reading that reduces EXACTLY to `relax` in the one-hot limit (non-primary kinds then
/// start at 0, where deviation and displacement coincide), so soft is a strict
/// generalisation of hard rather than a second instrument.
pub fn relax_soft(st: &Structure<N>, block_mass: &[f64; 4], pp: &PathParams) -> Vec<Arrival> {
    let mut params = Params::harmonic();
    params.dt = pp.dt;
    assert_eq!(params.rest_scale, 0.0, "FSD 13: the linear regime is binding here");

    // per-kind initial amplitude = its block's mass; argmax block is the primary
    let mut w = [0.0f64; N];
    let mut best = 0usize;
    for (bi, (label, _)) in crate::blocks::BLOCK_MEMBERS.iter().enumerate() {
        debug_assert_eq!(*label, crate::blocks::SURFACES[bi]);
        for i in crate::blocks::members(label).expect("block") { w[i] = block_mass[bi]; }
        if block_mass[bi] > block_mass[best] { best = bi; }
    }
    let primary: Vec<usize> = crate::blocks::members(crate::blocks::SURFACES[best]).unwrap();

    let mut pos = [[0.0f64; 3]; N];
    for i in 0..N { pos[i][0] = pp.amplitude * w[i]; }
    let x0: Vec<f64> = (0..N).map(|i| pos[i][0]).collect();
    let mut state = State::at_rest(pos);

    let mut first: [Option<usize>; N] = [None; N];
    for &s in &primary { first[s] = Some(0); }

    let mut t = 0usize;
    while t < pp.max_steps && first.iter().any(|f| f.is_none()) {
        step(&mut state, &st, &params, false);
        t += 1;
        for i in 0..N {
            if first[i].is_some() { continue; }
            let p = state.pos[i];
            let dx = p[0] - x0[i];
            let d = libm_sqrt(dx * dx + p[1] * p[1] + p[2] * p[2]);
            if d > pp.threshold { first[i] = Some(t); }
        }
    }

    let mut out: Vec<Arrival> = (0..N)
        .map(|i| Arrival { kind: KINDS[i], index: i, step: first[i], seeded: primary.contains(&i) })
        .collect();
    out.sort_by(|a, b| match (a.step, b.step) {
        (Some(x), Some(y)) => x.cmp(&y).then(a.index.cmp(&b.index)),
        (Some(_), None) => core::cmp::Ordering::Less,
        (None, Some(_)) => core::cmp::Ordering::Greater,
        (None, None) => a.index.cmp(&b.index),
    });
    out
}

fn libm_sqrt(x: f64) -> f64 { libm::sqrt(x) }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks;
    use ciris_sim_core::data::COUPLING;

    #[test]
    fn seeded_kinds_lead_the_path() {
        let seeds = blocks::members("Rules").unwrap();
        let p = relax_coupling(&COUPLING, &seeds, &PathParams::default());
        for a in p.iter().take(seeds.len()) {
            assert!(a.seeded, "a seeded kind should sort ahead of anything it propagates to");
        }
        assert_eq!(p.len(), N);
    }

    #[test]
    fn different_surfaces_give_different_paths() {
        let mut seen = std::collections::HashSet::new();
        for s in blocks::SURFACES {
            let seeds = blocks::members(s).unwrap();
            let p = relax_coupling(&COUPLING, &seeds, &PathParams::default());
            seen.insert(p.iter().map(|a| a.index).collect::<Vec<_>>());
        }
        assert_eq!(seen.len(), 4, "the four surfaces must not collapse to one path");
    }

    /// The whole experiment rests on the scramble changing the reasoning. If real and
    /// scrambled couplings produced the same order, arms B and C would be identical and
    /// the comparison would be vacuous.
    #[test]
    fn scrambling_changes_the_arrival_order() {
        let seeds = blocks::members("Facts").unwrap();
        let real = relax_coupling(&COUPLING, &seeds, &PathParams::default());
        let differing = (0..10u64).filter(|&s| {
            let sc = crate::scramble::scramble(s);
            relax_coupling(&sc, &seeds, &PathParams::default()).iter().map(|a| a.index).collect::<Vec<_>>()
                != real.iter().map(|a| a.index).collect::<Vec<_>>()
        }).count();
        assert!(differing >= 8, "only {differing}/10 scrambles changed the path; \
                                 arms B and C would be near-identical");
    }

    #[test]
    fn deterministic() {
        let seeds = blocks::members("Manner").unwrap();
        let a = relax_coupling(&COUPLING, &seeds, &PathParams::default());
        let b = relax_coupling(&COUPLING, &seeds, &PathParams::default());
        assert_eq!(a, b);
    }

    /// One-hot soft seeding must give EXACTLY the hard path: same order, same seeded
    /// flags, same arrival steps. This is what makes soft a generalisation of hard
    /// rather than a second instrument.
    #[test]
    fn soft_one_hot_reduces_to_hard() {
        let st = structure_for(&COUPLING);
        let pp = PathParams::default();
        for (bi, s) in blocks::SURFACES.iter().enumerate() {
            let mut m = [0.0f64; 4];
            m[bi] = 1.0;
            let soft = relax_soft(&st, &m, &pp);
            let hard = relax(&st, &blocks::members(s).unwrap(), &pp);
            assert_eq!(soft, hard, "one-hot {s} diverged from hard");
        }
    }

    /// Different distributions must move the path: this is the per-item variation A2
    /// exists to create.
    #[test]
    fn soft_distributions_vary_the_path() {
        let st = structure_for(&COUPLING);
        let pp = PathParams::default();
        let dists: [[f64; 4]; 4] = [
            [0.85, 0.05, 0.05, 0.05],
            [0.55, 0.05, 0.05, 0.35],
            [0.55, 0.35, 0.05, 0.05],
            [0.40, 0.10, 0.10, 0.40],
        ];
        let mut seen = std::collections::HashSet::new();
        for m in dists {
            let p = relax_soft(&st, &m, &pp);
            seen.insert(p.iter().map(|a| (a.index, a.step)).collect::<Vec<_>>());
        }
        assert!(seen.len() >= 3, "only {} distinct paths from 4 distributions", seen.len());
    }

    #[test]
    fn soft_deterministic() {
        let st = structure_for(&COUPLING);
        let m = [0.6, 0.2, 0.1, 0.1];
        let a = relax_soft(&st, &m, &PathParams::default());
        let b = relax_soft(&st, &m, &PathParams::default());
        assert_eq!(a, b);
    }
}
