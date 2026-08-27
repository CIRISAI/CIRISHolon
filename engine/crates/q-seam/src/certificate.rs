//! The certificate: the four staked criteria, the four mutants, and the joint gate.
//!
//! `Q_SEAM_PREREG.md` §4–§5 with amendment A1. Every threshold here is frozen in that document
//! and nothing in this module may adjust one. The classification is deliberately unforgiving in
//! one direction: a **false positive** — certifying a configuration where the chart is out of
//! tolerance — is the certificate lying, and it is fatal on the spot.

use crate::audit::{Mp2Audit, Stability};
use crate::chart::Chart;
use crate::observables::ExactObservables;
use crate::{KAPPA, PLANT_U, TAU};

/// One configuration of the sweep, fully measured.
#[derive(Clone, Debug)]
pub struct Configuration {
    pub sites: usize,
    pub u: f64,
    /// Signed errors in observable order (chart − exact).
    pub error: [f64; 6],
    /// `|error_o| / τ_o`, in units of tolerances.
    pub normalized: [f64; 6],
    /// `E_tot` — the max over all six. Q5's error number.
    pub e_tot: f64,
    /// `E5` — the max over the five genuine chart predictions, `D_bool` excluded.
    /// Q6's error number, per amendment A1/H2.
    pub e5: f64,
    pub audit: Mp2Audit,
    pub stability: Stability,
    /// The chart's symmetry breaking: `(spin, particle-hole, reflection)`.
    pub breaking: (f64, f64, f64),
    pub exact_energy: f64,
    pub chart_energy: f64,
    pub d_bool_exact: f64,
    pub residual: f64,
    pub in_sector_gap: f64,
}

impl Configuration {
    pub fn assemble(
        sites: usize,
        u: f64,
        exact: &ExactObservables,
        exact_energy: f64,
        residual: f64,
        in_sector_gap: f64,
        chart: &Chart,
    ) -> Self {
        let e_exact = ExactObservables::energy_per_site(exact_energy, sites, chart.t);
        let chart_density = chart.density();
        let chart_mag = chart.magnetization();
        let chart_bond = chart.bond();

        let error = [
            chart.energy_per_site() - e_exact,
            chart.double_occ_mean() - exact.double_occ_mean,
            (0..sites)
                .map(|i| chart_density[i] - exact.density[i])
                .fold(0.0, |a: f64, b| if b.abs() > a.abs() { b } else { a }),
            (0..sites)
                .map(|i| chart_mag[i] - exact.magnetization[i])
                .fold(0.0, |a: f64, b| if b.abs() > a.abs() { b } else { a }),
            (0..sites - 1)
                .map(|i| chart_bond[i] - exact.bond[i])
                .fold(0.0, |a: f64, b| if b.abs() > a.abs() { b } else { a }),
            // The chart's D_bool is exactly zero by idempotency, so its error IS the exact value.
            -exact.d_bool,
        ];

        let mut normalized = [0.0; 6];
        for k in 0..6 {
            normalized[k] = error[k].abs() / TAU[k];
        }
        let e_tot = normalized.iter().copied().fold(0.0, f64::max);
        let e5 = normalized[..5].iter().copied().fold(0.0, f64::max);

        Self {
            sites,
            u,
            error,
            normalized,
            e_tot,
            e5,
            audit: Mp2Audit::of(chart),
            stability: Stability::of(chart),
            breaking: chart.symmetry_breaking(),
            exact_energy,
            chart_energy: chart.energy,
            d_bool_exact: exact.d_bool,
            residual,
            in_sector_gap,
        }
    }

    /// Chart-honest under Q5's six-observable definition.
    pub fn honest6(&self) -> bool {
        self.e_tot <= 1.0
    }

    /// Chart-honest under Q6's five-observable definition (A1/H2).
    pub fn honest5(&self) -> bool {
        self.e5 <= 1.0
    }

    pub fn is_plant(&self) -> bool {
        self.u == PLANT_U
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Criterion {
    /// The chart's MP2 self-audit at `κ = 0.5`.
    C1,
    /// The stability margin against half its own `U = 0` value.
    C2,
    /// The theorem-pinned observables (A1/H1).
    C3,
    /// `C1 ∧ C3` — no new constant (A1).
    C4,
    /// Certify everywhere.
    M1,
    /// Refuse everywhere.
    M2,
}

impl Criterion {
    pub fn label(self) -> &'static str {
        match self {
            Criterion::C1 => "C1 self-audit",
            Criterion::C2 => "C2 stability",
            Criterion::C3 => "C3 theorem-pinned",
            Criterion::C4 => "C4 = C1 AND C3",
            Criterion::M1 => "M1 certify-everywhere",
            Criterion::M2 => "M2 refuse-everywhere",
        }
    }

    /// `lambda_zero` is `λ_min(N, U = 0)` for this configuration's N — C2's self-normalizer.
    pub fn certifies(self, c: &Configuration, lambda_zero: f64) -> bool {
        match self {
            Criterion::C1 => c
                .audit
                .as_vector()
                .iter()
                .zip(TAU.iter())
                .all(|(est, tau)| *est <= KAPPA * tau),
            Criterion::C2 => c.stability.lambda_min >= 0.5 * lambda_zero,
            Criterion::C3 => {
                let (spin, ph, refl) = c.breaking;
                spin <= KAPPA * TAU[3] && ph <= KAPPA * TAU[2] && refl <= KAPPA * TAU[2]
            }
            Criterion::C4 => {
                Criterion::C1.certifies(c, lambda_zero) && Criterion::C3.certifies(c, lambda_zero)
            }
            Criterion::M1 => true,
            Criterion::M2 => false,
        }
    }
}

/// The four clauses of the joint gate, scored.
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
    /// The largest `U/t` certified at each N, in `SWEEP_SITES` order — P-Q5-N's quantity.
    pub certified_boundary: Vec<f64>,
    /// Configurations the criterion certified while the chart was out of tolerance, as
    /// `(N, U, worst observable index, E_tot)`. Named, never just counted.
    pub false_positives: Vec<(usize, f64, usize, f64)>,
}

impl Score {
    /// The joint gate: `FP = 0` AND `coverage ≥ 0.5` AND every `U = 0` certified AND the plant
    /// refused at every N.
    pub fn passes(&self) -> bool {
        self.fp == 0 && self.coverage >= 0.5 && self.u_zero_certified && self.plant_refused
    }

    pub fn clause_report(&self) -> String {
        format!(
            "FP={} cov={:.3} U0={} plant={} -> {}",
            self.fp,
            self.coverage,
            if self.u_zero_certified { "ok" } else { "FAIL" },
            if self.plant_refused { "refused" } else { "CERTIFIED" },
            if self.passes() { "PASS" } else { "fail" }
        )
    }
}

/// Score any certify-predicate over the sweep.
pub fn score<F: Fn(&Configuration) -> bool>(
    label: &str,
    configs: &[Configuration],
    certify: F,
) -> Score {
    let (mut tp, mut fp, mut fn_, mut tn) = (0, 0, 0, 0);
    let mut u_zero_certified = true;
    let mut plant_refused = true;
    let mut false_positives = Vec::new();
    let mut boundary: std::collections::BTreeMap<usize, f64> = std::collections::BTreeMap::new();

    for c in configs {
        let certified = certify(c);
        match (certified, c.honest6()) {
            (true, true) => tp += 1,
            (true, false) => {
                fp += 1;
                let worst = c
                    .normalized
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(k, _)| k)
                    .unwrap_or(0);
                false_positives.push((c.sites, c.u, worst, c.e_tot));
            }
            (false, true) => fn_ += 1,
            (false, false) => tn += 1,
        }
        if c.u == 0.0 && !certified {
            u_zero_certified = false;
        }
        if c.is_plant() && certified {
            plant_refused = false;
        }
        if certified {
            let e = boundary.entry(c.sites).or_insert(0.0);
            if c.u > *e {
                *e = c.u;
            }
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
        certified_boundary: crate::SWEEP_SITES
            .iter()
            .map(|n| boundary.get(n).copied().unwrap_or(-1.0))
            .collect(),
        false_positives,
    }
}

/// M3 — the best FIXED cutoff `U/t ≤ u*`, `u*` chosen POST HOC to maximize coverage subject to
/// `FP = 0` and the plant refused at all five N (pin A1/P3). Tie-break: the most conservative
/// admissible cutoff. `None` means M3 is INFEASIBLE, which is itself informative.
pub fn best_fixed_cutoff(configs: &[Configuration]) -> Option<(f64, Score)> {
    let mut candidates: Vec<f64> = crate::SWEEP_U.to_vec();
    candidates.push(-1.0);
    let mut best: Option<(f64, Score)> = None;
    for &ustar in &candidates {
        let s = score("M3", configs, |c| c.u <= ustar);
        if s.fp == 0 && s.plant_refused {
            let better = match &best {
                None => true,
                Some((bu, bs)) => {
                    s.coverage > bs.coverage || (s.coverage == bs.coverage && ustar < *bu)
                }
            };
            if better {
                best = Some((ustar, s));
            }
        }
    }
    best
}

/// M4 — the best two-parameter cutoff `U/t ≤ a + b/(N+1)`, `(a, b)` fitted post hoc under the
/// same constraints. The hardest baseline the certificate has to beat, and the one that can
/// imitate a criterion whose boundary moves with N.
pub fn best_scaling_cutoff(configs: &[Configuration]) -> Option<(f64, f64, Score)> {
    let mut best: Option<(f64, f64, Score)> = None;
    // A grid dense enough to find any admissible cutoff the sweep can distinguish.
    for ai in -1..=64 {
        let a = ai as f64 * 0.25;
        for bi in -32..=64 {
            let b = bi as f64 * 0.5;
            let s = score("M4", configs, |c| c.u <= a + b / (c.sites as f64 + 1.0));
            if s.fp == 0 && s.plant_refused {
                let better = match &best {
                    None => true,
                    Some((_, _, bs)) => s.coverage > bs.coverage,
                };
                if better {
                    best = Some((a, b, s));
                }
            }
        }
    }
    best
}
