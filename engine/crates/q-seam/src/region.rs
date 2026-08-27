//! Per-region certification: `Q7_SEAM_PREREG.md` §6–§8 with amendment A1(Q7).
//!
//! Everything here is per **region-instance** — one contiguous block of two sites at one
//! `(N, U, a)`. The deliverable is the **refusal map**: a Certified/Refused label per region, which
//! is the crystal tier's seam policy in operational form (where mean-field suffices, light it;
//! where correlation bites, refuse and refine).

use crate::chart::Chart;
use crate::dense::jacobi;
use crate::observables::ExactObservables;
use crate::{KAPPA, TAU};

/// Q7's per-region tolerances, in observable order R1…R5.
/// R1–R4 are **carried unchanged from Q5** (no re-tuning); R5 is the one new stake, at the same
/// scale as Q5's global `D_bool` tolerance.
pub const TAU_R: [f64; 5] = [TAU[2], TAU[1], TAU[3], TAU[4], 0.05];

/// Blocks of two sites (`Q7_SEAM_PREREG.md` §3): the smallest size whose restricted density matrix
/// has a non-trivial spectrum.
pub fn regions(sites: usize) -> Vec<[usize; 2]> {
    (0..sites / 2).map(|k| [2 * k, 2 * k + 1]).collect()
}

/// Boolean defect of a 1-RDM **restricted to a block**.
///
/// A sub-block of an idempotent matrix is NOT idempotent, so the chart makes a real, varying,
/// falsifiable prediction here — which is exactly why the per-region wrongness-meter is admissible
/// where A1/H2 ruled the global one (a structural zero) out.
pub fn block_dbool(rdm: &[Vec<f64>; 2], sites: usize, block: &[usize; 2]) -> f64 {
    let mut worst = 0.0f64;
    for spin in 0..2 {
        let m: Vec<f64> = (0..2)
            .flat_map(|i| (0..2).map(move |j| (i, j)))
            .map(|(i, j)| rdm[spin][block[i] * sites + block[j]])
            .collect();
        for x in jacobi(m, 2).values {
            worst = worst.max(x.min(1.0 - x));
        }
    }
    worst
}

#[derive(Clone, Debug)]
pub struct RegionInstance {
    pub sites: usize,
    pub u: f64,
    pub a: f64,
    pub index: usize,
    pub block: [usize; 2],
    /// R1…R5 normalized by their tolerances.
    pub normalized: [f64; 5],
    /// `E_r` — the max of the five. The truth-side per-region error.
    pub e_r: f64,
    /// D1's quantity: the chart's magnetization, whose exact value is pinned to zero (§2.2).
    pub break_spin: f64,
    /// D1b's quantity: the chart's density reflection asymmetry, pinned to zero (§2.3).
    pub break_refl: f64,
    /// D2's quantity: the local energy estimate `σ²/Δ` (§7).
    pub self_audit: f64,
    /// D4's quantity: `max_i min(n^MF_i, |n^MF_i - 2|)` - distance from a determinate filling.
    pub density_extremity: f64,
    /// Reported per region as a secondary diagnostic; cannot change any determination.
    pub dbool_exact: f64,
    pub dbool_chart: f64,
}

impl RegionInstance {
    pub fn honest(&self) -> bool {
        self.e_r <= 1.0
    }

    pub fn measure(
        sites: usize,
        u: f64,
        a: f64,
        index: usize,
        block: [usize; 2],
        exact: &ExactObservables,
        chart: &Chart,
    ) -> Self {
        let cd = chart.density();
        let cm = chart.magnetization();
        let cb = chart.bond();
        let sigma = chart.sigma();
        let refl = chart.reflection_asymmetry();
        let gap = chart.gap();

        let dbool_exact = block_dbool(&exact.rdm, sites, &block);
        let chart_rdm = [chart.rdm[0].clone(), chart.rdm[1].clone()];
        let dbool_chart = block_dbool(&chart_rdm, sites, &block);

        let m = |f: &dyn Fn(usize) -> f64| block.iter().map(|&i| f(i)).fold(0.0, f64::max);

        let err = [
            // R1 density
            m(&|i| (cd[i] - exact.density[i]).abs()),
            // R2 double occupancy
            m(&|i| (chart.occupation[0][i] * chart.occupation[1][i] - exact.double_occ[i]).abs()),
            // R3 magnetization — exact value pinned to ZERO by §2.2, so this is pure chart data.
            m(&|i| cm[i].abs()),
            // R4 the intra-block bond (block = {2k, 2k+1}, so bond index 2k)
            (cb[block[0]] - exact.bond[block[0]]).abs(),
            // R5 block Boolean defect
            (dbool_chart - dbool_exact).abs(),
        ];
        let mut normalized = [0.0; 5];
        for k in 0..5 {
            normalized[k] = err[k] / TAU_R[k];
        }
        let e_r = normalized.iter().copied().fold(0.0, f64::max);

        Self {
            sites,
            u,
            a,
            index,
            block,
            normalized,
            e_r,
            break_spin: m(&|i| cm[i].abs()),
            break_refl: m(&|i| refl[i]),
            self_audit: m(&|i| sigma[i] * sigma[i]) / gap.max(1e-12),
            density_extremity: m(&|i| cd[i].min((cd[i] - 2.0).abs())),
            dbool_exact,
            dbool_chart,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cand {
    /// The theorem-pinned spin anchor (PRIMARY).
    D1,
    /// The theorem-pinned reflection anchor (PRIMARY, A1(Q7)/OPT).
    D1b,
    /// The self-residual — the staked expected-failure control.
    D2,
    /// `D1 ∧ D1b ∧ D2`. No new constant.
    D3,
    /// The density heuristic (Q7b §4) — chart data, but NO theorem behind it.
    D4,
    /// `D4 ∧ D1b` (A1(Q7b)/P-D4-D1b-COMPLEMENT). No new constant.
    D5,
    N1,
    N2,
}

impl Cand {
    pub fn label(self) -> &'static str {
        match self {
            Cand::D1 => "D1 spin anchor",
            Cand::D1b => "D1b reflection anchor",
            Cand::D2 => "D2 self-residual",
            Cand::D3 => "D3 = D1 AND D1b AND D2",
            Cand::D4 => "D4 density heuristic",
            Cand::D5 => "D5 = D4 AND D1b",
            Cand::N1 => "N1 certify-everywhere",
            Cand::N2 => "N2 refuse-everywhere",
        }
    }

    pub fn certifies(self, r: &RegionInstance) -> bool {
        match self {
            Cand::D1 => r.break_spin <= KAPPA * TAU[3],
            Cand::D1b => r.break_refl <= KAPPA * TAU[2],
            Cand::D2 => r.self_audit <= KAPPA * TAU[0],
            Cand::D3 => {
                Cand::D1.certifies(r) && Cand::D1b.certifies(r) && Cand::D2.certifies(r)
            }
            // D4: certify iff the chart's local density is within 0.25 of a determinate
            // filling. STAKED at 0.25. Density extremity is a SUFFICIENT route to local
            // determinacy, not the criterion - U -> 0 is another route D4 cannot see, which is
            // the derived reason P-D4-COVERAGE expects it to fail clause 3.
            Cand::D4 => r.density_extremity <= 0.25,
            Cand::D5 => Cand::D4.certifies(r) && Cand::D1b.certifies(r),
            Cand::N1 => true,
            Cand::N2 => false,
        }
    }
}

/// One `(N, U, a)` configuration's worth of regions.
pub struct Configuration {
    pub sites: usize,
    pub u: f64,
    pub a: f64,
    pub regions: Vec<RegionInstance>,
}

impl Configuration {
    /// A1(Q7)/P2, pinned WITH A MARGIN: `min_r E_r ≤ 0.5` AND `max_r E_r ≥ 2.0`. The margin stops
    /// G7-FIT passing on two regions straddling the tolerance by numerical noise.
    pub fn spatially_split(&self) -> bool {
        let lo = self.regions.iter().map(|r| r.e_r).fold(f64::INFINITY, f64::min);
        let hi = self.regions.iter().map(|r| r.e_r).fold(0.0, f64::max);
        lo <= 0.5 && hi >= 2.0
    }

    /// The region containing the chain centre — the PLANT's location at `U/t = 16`.
    pub fn centre_index(&self) -> usize {
        self.regions.len() / 2 - if self.regions.len() % 2 == 0 { 1 } else { 0 }
    }
}

#[derive(Clone, Debug)]
pub struct Score {
    pub label: String,
    pub tp: usize,
    pub fp: usize,
    pub fn_: usize,
    pub tn: usize,
    pub coverage: f64,
    pub u_zero_certified: bool,
    pub plant_refused: bool,
    /// A1(Q7)/P2 clause 5: split configurations on which the criterion certifies an honest region
    /// AND refuses a wrong one.
    pub discriminating: usize,
    pub false_positives: Vec<(usize, f64, f64, usize, f64)>,
}

impl Score {
    /// The five clauses of §8.1.
    pub fn passes(&self) -> bool {
        self.fp == 0
            && self.coverage >= 0.5
            && self.u_zero_certified
            && self.plant_refused
            && self.discriminating >= 5
    }

    pub fn clause_report(&self) -> String {
        format!(
            "FP={} cov={:.3} U0={} plant={} discrim={} -> {}",
            self.fp,
            self.coverage,
            if self.u_zero_certified { "ok" } else { "FAIL" },
            if self.plant_refused { "refused" } else { "CERTIFIED" },
            self.discriminating,
            if self.passes() { "PASS" } else { "fail" }
        )
    }
}

pub fn score<F: Fn(&RegionInstance) -> bool>(
    label: &str,
    configs: &[Configuration],
    certify: F,
) -> Score {
    let (mut tp, mut fp, mut fn_, mut tn) = (0, 0, 0, 0);
    let mut u_zero_certified = true;
    let mut plant_refused = true;
    let mut discriminating = 0usize;
    let mut false_positives = Vec::new();

    for c in configs {
        let centre = c.centre_index();
        let (mut cert_honest, mut ref_wrong) = (false, false);
        for r in &c.regions {
            let certified = certify(r);
            match (certified, r.honest()) {
                (true, true) => {
                    tp += 1;
                    cert_honest = true;
                }
                (true, false) => {
                    fp += 1;
                    let worst = r
                        .normalized
                        .iter()
                        .enumerate()
                        .max_by(|x, y| x.1.partial_cmp(y.1).unwrap())
                        .map(|(k, _)| k)
                        .unwrap_or(0);
                    false_positives.push((r.sites, r.u, r.a, worst, r.e_r));
                }
                (false, true) => fn_ += 1,
                (false, false) => {
                    tn += 1;
                    ref_wrong = true;
                }
            }
            if r.u == 0.0 && !certified {
                u_zero_certified = false;
            }
            if r.u == crate::PLANT_U && r.index == centre && certified {
                plant_refused = false;
            }
        }
        if c.spatially_split() && cert_honest && ref_wrong {
            discriminating += 1;
        }
    }

    let coverage = if tp + fn_ == 0 { 0.0 } else { tp as f64 / (tp + fn_) as f64 };
    Score {
        label: label.to_string(),
        tp,
        fp,
        fn_,
        tn,
        coverage,
        u_zero_certified,
        plant_refused,
        discriminating,
        false_positives,
    }
}

/// N3 — best GLOBAL cutoff `U ≤ u*`, one parameter, fitted post hoc to maximise coverage subject
/// to `FP = 0` and the plant refused.
pub fn best_global(configs: &[Configuration], grid: &[f64]) -> Option<(f64, Score)> {
    let mut cands: Vec<f64> = grid.to_vec();
    cands.push(-1.0);
    let mut best: Option<(f64, Score)> = None;
    for &ustar in &cands {
        let s = score("N3", configs, |r| r.u <= ustar);
        if s.fp == 0 && s.plant_refused {
            let better = match &best {
                None => true,
                Some((bu, bs)) => s.coverage > bs.coverage || (s.coverage == bs.coverage && ustar < *bu),
            };
            if better {
                best = Some((ustar, s));
            }
        }
    }
    best
}

/// N4 — best PER-REGION cutoff `U ≤ u*(r)`, **fitted jointly across all `a`** (A1(Q7)/P3).
///
/// One parameter per `(N, region index)`; the parameter count does not scale with the `a`-axis,
/// which is the line separating a baseline from the oracle. The fit is exact rather than a search:
/// FP is per-region-instance and coverage is a sum over them, so the optimum separates by region.
pub fn best_per_region(configs: &[Configuration], grid: &[f64]) -> Option<(Vec<(usize, usize, f64)>, Score)> {
    let mut keys: Vec<(usize, usize)> = Vec::new();
    for c in configs {
        for r in &c.regions {
            if !keys.contains(&(r.sites, r.index)) {
                keys.push((r.sites, r.index));
            }
        }
    }
    let mut thresholds = Vec::new();
    for &(n, idx) in &keys {
        let mut best_u = -1.0f64;
        for &ustar in grid.iter() {
            let ok = configs.iter().all(|c| {
                let centre = c.centre_index();
                c.regions.iter().all(|r| {
                    if r.sites != n || r.index != idx || r.u > ustar {
                        return true;
                    }
                    r.honest() && !(r.u == crate::PLANT_U && r.index == centre)
                })
            });
            if ok && ustar > best_u {
                best_u = ustar;
            }
        }
        thresholds.push((n, idx, best_u));
    }
    let lookup = thresholds.clone();
    let s = score("N4", configs, |r| {
        lookup
            .iter()
            .find(|(n, i, _)| *n == r.sites && *i == r.index)
            .map(|(_, _, u)| r.u <= *u)
            .unwrap_or(false)
    });
    if s.fp == 0 && s.plant_refused {
        Some((thresholds, s))
    } else {
        None
    }
}

/// N5 — best trap-aware cutoff `U ≤ α + β·d(r)`, two parameters, `d` the region's distance from
/// the chain centre in units of half the chain.
pub fn best_scaling(configs: &[Configuration]) -> Option<(f64, f64, Score)> {
    let dist = |r: &RegionInstance| {
        let c = (r.sites as f64 - 1.0) / 2.0;
        ((r.block[0] as f64 + r.block[1] as f64) / 2.0 - c).abs() / (r.sites as f64 / 2.0)
    };
    let mut best: Option<(f64, f64, Score)> = None;
    for ai in -2..=68 {
        let alpha = ai as f64 * 0.25;
        for bi in -40..=80 {
            let beta = bi as f64 * 0.5;
            let s = score("N5", configs, |r| r.u <= alpha + beta * dist(r));
            if s.fp == 0 && s.plant_refused {
                let better = match &best {
                    None => true,
                    Some((_, _, bs)) => s.coverage > bs.coverage,
                };
                if better {
                    best = Some((alpha, beta, s));
                }
            }
        }
    }
    best
}
