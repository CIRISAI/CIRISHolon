//! FLUID-0's instrument: the shear viscosity and the tracer diffusivity of a collision law,
//! each read off the decay of a seeded wave, each REFUSING BY NAME when it cannot fit.
//!
//! `conformance/mesh/FLUID0_PREREG.md` is the freeze and every constant below is its.
//! A reading is a `TransportReading` and its `exit` is the first-class field: `NoDecay`,
//! `TooFewPoints`, `LowR2` and `PreHydrodynamic` are answers, not failures, and a refusal is
//! never reported as a measurement of zero transport (M-NULL-MISSTAKE).
//!
//! # The geometry, once
//!
//! A cell `c` sits at row `j = c % l` and column `i = c / l`, and its Euclidean position is
//! `i·(1,0) + j·(1/2, √3/2)`. So a field modulated by `j` alone varies only along `ŷ`, with
//! wavevector `k = 2π·k_index / (l·√3/2)`; a field modulated by `i` alone varies along
//! `(√3/2, −1/2)`, with the SAME `|k|`, because the perpendicular spacing of the constant-`i`
//! lines is `√3/2` links exactly as the constant-`j` lines' is. The two line families are one
//! 60° rotation apart, which is what makes the column reading an isotropy test of the row
//! reading rather than a second, different experiment.
//!
//! A shear wave must have its momentum TRANSVERSE to `k`. For the rows that is `x̂` (the
//! freeze's `c_dir,x` drive); for the columns it is the 60° rotation of `x̂`, namely
//! `(1/2, √3/2)`. See [`shear_viscosity`] for the one deviation this forces and why.
//!
//! # What the readouts read
//!
//! The row momentum readout is the crate's own [`Lattice::line_momenta_of`], whose first `l`
//! entries are the AXIAL x-momentum summed over each row. Axial `px` equals Euclidean
//! `P_x − P_y/√3`, so it sees the shear mode with a factor `3` per unit drive and the
//! longitudinal mode with a factor the seeding never excites. The column readout is the
//! second `l` entries, the axial y-momentum, which is `2/√3` times Euclidean `P_y`.

// Three lints are waived here, each for one reason and no other:
//  * `too_many_arguments` — the freeze names the run's parameters one by one, and folding them
//    into a struct would put a second spelling of the freeze in the code.
//  * `neg_cmp_op_on_partial_ord` — `!(x > 0.0)` is the NaN-catching form ON PURPOSE: a NaN
//    amplitude, gamma or R² must take the REFUSAL branch, and `x <= 0.0` would pass it through.
//  * `needless_range_loop` — `t` is the fit's x coordinate, not merely an index into a slice.
#![allow(clippy::too_many_arguments, clippy::neg_cmp_op_on_partial_ord, clippy::needless_range_loop)]

use crate::isotropy::embed;
use crate::lattice::{ColourRule, Lattice};
use crate::state::Model;
use core::f64::consts::PI;

/// The fit window closes at the first step whose amplitude is under this fraction of the
/// amplitude at step `L`. `FLUID0_PREREG` §2 (M-FLOOR-UNSTAKED).
pub const WINDOW_END_FRACTION: f64 = 0.1;
/// Monotonicity is read with a tolerance: a single step may rise by this fraction of the
/// window's starting amplitude before the reading is refused `NoDecay`. STATED, not implied —
/// the freeze leaves the number to the instrument and this is it.
pub const MONOTONE_TOLERANCE_FRACTION: f64 = 0.01;
/// The fit's floor: fewer points than this is `TooFewPoints`.
pub const MIN_POINTS: usize = 20;
/// The fit's floor: below this `R²` is `LowR2`, which the freeze folds under refusal.
pub const MIN_R2: f64 = 0.99;
/// Two wavevectors disagreeing by more than this fraction read `PreHydrodynamic`.
pub const K_AGREEMENT_TOLERANCE: f64 = 0.10;

/// Why a reading stopped. M-EXIT-DISCRIMINATOR: every reading carries one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    /// A decaying exponential was fitted at or above the floor. The only exit carrying a value.
    Fitted,
    /// The amplitude did not fall monotonically over the window — the identity law's case.
    NoDecay,
    /// The two wavevectors disagree by more than [`K_AGREEMENT_TOLERANCE`]. Set by the
    /// caller that holds both readings, never by a single run.
    PreHydrodynamic,
    /// Fewer than [`MIN_POINTS`] points in the window.
    TooFewPoints,
    /// The fit's `R²` is under [`MIN_R2`].
    LowR2,
}

impl Exit {
    pub fn name(self) -> &'static str {
        match self {
            Exit::Fitted => "Fitted",
            Exit::NoDecay => "NoDecay",
            Exit::PreHydrodynamic => "PreHydrodynamic",
            Exit::TooFewPoints => "TooFewPoints",
            Exit::LowR2 => "LowR2",
        }
    }
}

/// One transport reading: the number when there is one, and the whole of why when there is not.
#[derive(Clone, Copy, Debug)]
pub struct TransportReading {
    /// `γ / k²` in link² per step — `Some` only when `exit == Fitted`.
    pub value: Option<f64>,
    /// The fitted decay rate, positive for a decaying wave (the negated least-squares slope).
    pub gamma: f64,
    pub k: f64,
    /// `(first step, last step)` of the fit window, inclusive.
    pub window: (usize, usize),
    pub points: usize,
    pub r2: f64,
    pub collisions_fired: u64,
    pub exit: Exit,
    /// `|M|` at the window's first step, and at its last.
    pub start_amplitude: f64,
    pub end_amplitude: f64,
}

impl TransportReading {
    fn refused(exit: Exit, k: f64, window: (usize, usize), points: usize, fired: u64, a0: f64, a1: f64) -> Self {
        Self {
            value: None,
            gamma: f64::NAN,
            k,
            window,
            points,
            r2: f64::NAN,
            collisions_fired: fired,
            exit,
            start_amplitude: a0,
            end_amplitude: a1,
        }
    }
}

/// Every conserved integer of a run, checked at EVERY step. G2's carrier.
#[derive(Clone, Copy, Debug)]
pub struct LedgerAudit {
    pub mass_exact: bool,
    pub px_exact: bool,
    pub py_exact: bool,
    /// The red-particle count. `true` vacuously on a run with no colour plane.
    pub red_exact: bool,
    pub steps_checked: u64,
    pub mass: i64,
    pub px: i64,
    pub py: i64,
    pub red: i64,
}

impl LedgerAudit {
    pub fn all_exact(&self) -> bool {
        self.mass_exact && self.px_exact && self.py_exact && self.red_exact
    }
}

/// FNV-1a 64 over a collision table — the law's identity beside its enumeration index, so a
/// re-ordered enumeration cannot silently rename a law (M-STALE-INSTRUMENT).
pub fn law_digest(law: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &b in law {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// `|k| = 2π·k_index / (l·√3/2)` per link — the row spacing of the hexagonal lattice, and,
/// as the module header derives, the column spacing too.
pub fn wavenumber(l: usize, k_index: usize) -> f64 {
    2.0 * PI * k_index as f64 / (l as f64 * (3.0f64).sqrt() / 2.0)
}

/// The 60° rotation of `x̂`: the transverse direction of a wave modulated along the columns.
pub const COLUMN_TRANSVERSE: [f64; 2] = [0.5, 0.866_025_403_784_438_6];

fn sine_table(l: usize, k_index: usize) -> Vec<f64> {
    let w = 2.0 * PI * k_index as f64 / l as f64;
    (0..l).map(|m| (w * m as f64).sin()).collect()
}

/// The shear readout: the crate's line momenta projected on the sine of the modulated index.
fn project_momenta(lat: &Lattice, cells: &[u8], sine: &[f64], along_columns: bool) -> f64 {
    let lm = lat.line_momenta_of(cells);
    let off = if along_columns { lat.l } else { 0 };
    (0..lat.l).map(|m| lm[off + m] as f64 * sine[m]).sum()
}

/// The tracer readout: the row colour excess `Σ colour − ½·Σ occupancy`, projected on the sine.
fn project_colour_excess(l: usize, cells: &[u8], colour: &[u8], sine: &[f64]) -> f64 {
    let mut rows = vec![0.0f64; l];
    for (c, (&s, &q)) in cells.iter().zip(colour).enumerate() {
        rows[c % l] += q.count_ones() as f64 - 0.5 * s.count_ones() as f64;
    }
    (0..l).map(|m| rows[m] * sine[m]).sum()
}

/// Where the fit's window opens, where it closes, and what floors it must clear.
///
/// `FLUID0_PREREG`'s rule is [`FitRule::frozen`]; `FLUID0_AMENDMENT_1`'s opens at a multiple of
/// the MEASURED kinetic time and closes on a floor set from the MEASURED equilibrium noise, so
/// both live here as data and neither is spelled twice.
#[derive(Clone, Copy, Debug)]
pub struct FitRule {
    /// The step the window opens at.
    pub start: usize,
    /// The window closes at this fraction of the amplitude at `start` …
    pub end_fraction: f64,
    /// … or at this absolute floor, whichever is higher. Zero means no floor.
    pub floor: f64,
    pub monotone_tolerance_fraction: f64,
    pub min_points: usize,
    pub min_r2: f64,
}

impl FitRule {
    /// `FLUID0_PREREG`'s rule: open at the lattice size, close at a tenth, no noise floor.
    pub fn frozen(l: usize) -> Self {
        Self {
            start: l,
            end_fraction: WINDOW_END_FRACTION,
            floor: 0.0,
            monotone_tolerance_fraction: MONOTONE_TOLERANCE_FRACTION,
            min_points: MIN_POINTS,
            min_r2: MIN_R2,
        }
    }
}

/// [`fit_with`] under `FLUID0_PREREG`'s frozen rule. Kept so the frozen-parameter record
/// (`gate_as_frozen.json`) can be reproduced exactly after the amendment lands.
pub fn fit(series: &[f64], l: usize, k: f64, fired: u64) -> TransportReading {
    fit_with(series, &FitRule::frozen(l), k, fired)
}

/// The whole of the fit, and the whole of the refusal, in one place so both readouts obey the
/// same floor. `series[t]` is the projection after `t` steps.
pub fn fit_with(series: &[f64], rule: &FitRule, k: f64, fired: u64) -> TransportReading {
    let l = rule.start;
    if series.len() <= l + 1 {
        return TransportReading::refused(Exit::TooFewPoints, k, (l, l), 0, fired, f64::NAN, f64::NAN);
    }
    let a0 = series[l].abs();
    if !(a0 > 0.0) {
        // An exactly-zero pre-arm amplitude voids the reading rather than measuring zero
        // transport (M-EMPTY-SECTOR).
        return TransportReading::refused(Exit::NoDecay, k, (l, l), 0, fired, a0, a0);
    }
    let threshold = (rule.end_fraction * a0).max(rule.floor);
    let mut end = series.len() - 1;
    for t in (l + 1)..series.len() {
        if series[t].abs() < threshold {
            end = t;
            break;
        }
    }
    let a1 = series[end].abs();
    let points = end - l + 1;

    // NoDecay first: the identity law's answer is about the SHAPE of the trace, and it must
    // not be reported as a short window or a bad fit.
    let tol = rule.monotone_tolerance_fraction * a0;
    for t in l..end {
        if series[t + 1].abs() > series[t].abs() + tol {
            return TransportReading::refused(Exit::NoDecay, k, (l, end), points, fired, a0, a1);
        }
    }
    if points < rule.min_points {
        return TransportReading::refused(Exit::TooFewPoints, k, (l, end), points, fired, a0, a1);
    }
    // A zero inside the window has no logarithm; the trace crossed through nothing, which is
    // not a decay.
    if series[l..=end].iter().any(|v| v.abs() == 0.0) {
        return TransportReading::refused(Exit::NoDecay, k, (l, end), points, fired, a0, a1);
    }

    let n = points as f64;
    let (mut sx, mut sy, mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for t in l..=end {
        let x = t as f64;
        let y = series[t].abs().ln();
        sx += x;
        sy += y;
        sxx += x * x;
        sxy += x * y;
        syy += y * y;
    }
    let den = n * sxx - sx * sx;
    if den == 0.0 {
        return TransportReading::refused(Exit::TooFewPoints, k, (l, end), points, fired, a0, a1);
    }
    let slope = (n * sxy - sx * sy) / den;
    let var_y = syy - sy * sy / n;
    let r2 = if var_y > 0.0 { (slope * (sxy - sx * sy / n)) / var_y } else { 0.0 };
    let gamma = -slope;
    if !(gamma > 0.0) {
        return TransportReading::refused(Exit::NoDecay, k, (l, end), points, fired, a0, a1);
    }
    if !(r2 >= rule.min_r2) {
        let mut r = TransportReading::refused(Exit::LowR2, k, (l, end), points, fired, a0, a1);
        r.gamma = gamma;
        r.r2 = r2;
        return r;
    }
    TransportReading {
        value: Some(gamma / (k * k)),
        gamma,
        k,
        window: (l, end),
        points,
        r2,
        collisions_fired: fired,
        exit: Exit::Fitted,
        start_amplitude: a0,
        end_amplitude: a1,
    }
}

/// The shear viscosity of one collision law, from the decay of a transverse momentum wave.
///
/// `along_columns` selects the isotropy leg: `false` modulates along the rows and drives the
/// momentum along `x̂`, which is the freeze's `c_dir,x` seeding verbatim; `true` modulates
/// along the columns and drives the momentum along [`COLUMN_TRANSVERSE`], the 60° rotation of
/// `x̂`, reading the crate's column momenta.
///
/// **Deviation, declared.** `FLUID0_PREREG` G3 names the column leg "the shear wave of the
/// y-momentum along the columns". Driving the momentum along `ŷ` there is not a shear wave:
/// `ŷ` has a component `−1/2` along that leg's wavevector `(√3/2, −1/2)`, so a quarter of the
/// seeded momentum is longitudinal — a propagating sound mode with its own damping, which
/// contaminates the fit rather than testing isotropy. This function seeds the exact 60°
/// rotation of the row experiment instead, which IS the isotropy test G3 asks for; the
/// literal `ŷ` drive is available through [`shear_viscosity_driven`] and the runner reports it
/// beside, ungated.
pub fn shear_viscosity(
    model: &Model,
    law: &[u8],
    l: usize,
    d: f64,
    a: f64,
    k_index: usize,
    steps: usize,
    seed: u64,
    along_columns: bool,
) -> TransportReading {
    let drive = if along_columns { COLUMN_TRANSVERSE } else { [1.0, 0.0] };
    shear_viscosity_driven(model, law, l, d, a, k_index, steps, seed, along_columns, drive).0
}

/// [`shear_viscosity`] with the momentum drive named, and the per-step ledger audit returned.
#[allow(clippy::too_many_arguments)]
pub fn shear_viscosity_driven(
    model: &Model,
    law: &[u8],
    l: usize,
    d: f64,
    a: f64,
    k_index: usize,
    steps: usize,
    seed: u64,
    along_columns: bool,
    drive: [f64; 2],
) -> (TransportReading, LedgerAudit) {
    let (series, fired, audit) =
        shear_series(model, law, l, d, a, k_index, steps, seed, along_columns, drive, true);
    (fit(&series, l, wavenumber(l, k_index), fired), audit)
}

/// The raw shear trace `M(t)`, `t = 0..`, with the collision count and the ledger audit.
///
/// `early_stop` closes the run at the window's own end — the first step under a tenth of the
/// step-`l` amplitude — which is the same reading, sooner. A caller that wants the whole
/// trace (a noise diagnostic, a seed average) passes `false`.
#[allow(clippy::too_many_arguments)]
pub fn shear_series(
    model: &Model,
    law: &[u8],
    l: usize,
    d: f64,
    a: f64,
    k_index: usize,
    steps: usize,
    seed: u64,
    along_columns: bool,
    drive: [f64; 2],
    early_stop: bool,
) -> (Vec<f64>, u64, LedgerAudit) {
    let lat = Lattice::seeded_shear_drive(
        model.clone(),
        l,
        seed,
        d,
        a,
        k_index,
        law.to_vec(),
        drive,
        along_columns,
    );
    let sine = sine_table(l, k_index);
    let mut cells = lat.cells.clone();
    let mut out = vec![0u8; l * l];
    let l0 = lat.ledger_of(&cells);
    let mut audit = LedgerAudit {
        mass_exact: true,
        px_exact: true,
        py_exact: true,
        red_exact: true,
        steps_checked: 0,
        mass: l0.mass,
        px: l0.momentum[0],
        py: l0.momentum[1],
        red: 0,
    };
    let mut series = Vec::with_capacity(steps + 1);
    series.push(project_momenta(&lat, &cells, &sine, along_columns));
    let mut fired = 0u64;
    let mut threshold = f64::NAN;
    for t in 0..steps {
        let stats = lat.advance(&mut cells, &mut out, t as u64);
        core::mem::swap(&mut cells, &mut out);
        fired += stats.collisions_fired;
        let led = lat.ledger_of(&cells);
        audit.mass_exact &= led.mass == l0.mass;
        audit.px_exact &= led.momentum[0] == l0.momentum[0];
        audit.py_exact &= led.momentum[1] == l0.momentum[1];
        audit.steps_checked += 1;
        series.push(project_momenta(&lat, &cells, &sine, along_columns));
        if t + 1 == l {
            threshold = WINDOW_END_FRACTION * series[l].abs();
        }
        // The window closes at the first step under a tenth of the step-L amplitude, so
        // nothing past it is read; stopping there is the same reading, sooner.
        if early_stop && t + 1 > l && series[t + 1].abs() < threshold {
            break;
        }
    }
    (series, fired, audit)
}

/// The tracer diffusivity of one collision law, from the decay of a colour wave on a fluid at
/// rest. Uniform density `d`, the colour wave of amplitude `a`, and the colour-blind step.
pub fn tracer_diffusion(
    model: &Model,
    law: &[u8],
    l: usize,
    d: f64,
    a: f64,
    k_index: usize,
    steps: usize,
    seed: u64,
) -> TransportReading {
    tracer_diffusion_rule(model, law, l, d, a, k_index, steps, seed, ColourRule::Blind).0
}

/// [`tracer_diffusion`] with the colour rule named (the plant handle) and the ledger audit.
#[allow(clippy::too_many_arguments)]
pub fn tracer_diffusion_rule(
    model: &Model,
    law: &[u8],
    l: usize,
    d: f64,
    a: f64,
    k_index: usize,
    steps: usize,
    seed: u64,
    rule: ColourRule,
) -> (TransportReading, LedgerAudit) {
    let (series, fired, audit) = colour_series(model, law, l, d, a, k_index, steps, seed, rule, true);
    (fit(&series, l, wavenumber(l, k_index), fired), audit)
}

/// The raw colour trace, as [`shear_series`] is for the shear readout.
#[allow(clippy::too_many_arguments)]
pub fn colour_series(
    model: &Model,
    law: &[u8],
    l: usize,
    d: f64,
    a: f64,
    k_index: usize,
    steps: usize,
    seed: u64,
    rule: ColourRule,
    early_stop: bool,
) -> (Vec<f64>, u64, LedgerAudit) {
    let lat = Lattice::seeded(model.clone(), l, seed, d, law.to_vec());
    let sine = sine_table(l, k_index);
    let mut cells = lat.cells.clone();
    let mut colour = lat.seed_colour_wave(seed, a, k_index);
    let mut out = vec![0u8; l * l];
    let mut out_colour = vec![0u8; l * l];
    let l0 = lat.ledger_of(&cells);
    let red0: i64 = colour.iter().map(|&q| q.count_ones() as i64).sum();
    let mut audit = LedgerAudit {
        mass_exact: true,
        px_exact: true,
        py_exact: true,
        red_exact: true,
        steps_checked: 0,
        mass: l0.mass,
        px: l0.momentum[0],
        py: l0.momentum[1],
        red: red0,
    };
    let mut series = Vec::with_capacity(steps + 1);
    series.push(project_colour_excess(l, &cells, &colour, &sine));
    let mut fired = 0u64;
    let mut threshold = f64::NAN;
    for t in 0..steps {
        let stats =
            lat.advance_with_colour_rule(&mut cells, &mut colour, &mut out, &mut out_colour, t as u64, rule);
        core::mem::swap(&mut cells, &mut out);
        core::mem::swap(&mut colour, &mut out_colour);
        fired += stats.collisions_fired;
        let led = lat.ledger_of(&cells);
        let red: i64 = colour.iter().map(|&q| q.count_ones() as i64).sum();
        audit.mass_exact &= led.mass == l0.mass;
        audit.px_exact &= led.momentum[0] == l0.momentum[0];
        audit.py_exact &= led.momentum[1] == l0.momentum[1];
        audit.red_exact &= red == red0;
        audit.steps_checked += 1;
        series.push(project_colour_excess(l, &cells, &colour, &sine));
        if t + 1 == l {
            threshold = WINDOW_END_FRACTION * series[l].abs();
        }
        if early_stop && t + 1 > l && series[t + 1].abs() < threshold {
            break;
        }
    }
    (series, fired, audit)
}

/// Plain least squares of `ln|series[t]|` against `t` over the inclusive window `[w0, w1]`:
/// the decay rate (positive for decay) and `R²`, with NO floors and NO exits.
///
/// This is the arithmetic inside [`fit`], exposed so a diagnostic can characterise a trace
/// whose window the freeze's floors refuse. A value from here is never a reading.
pub fn log_slope(series: &[f64], w0: usize, w1: usize) -> (f64, f64) {
    let n = (w1 - w0 + 1) as f64;
    let (mut sx, mut sy, mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for t in w0..=w1 {
        let x = t as f64;
        let y = series[t].abs().max(1e-300).ln();
        sx += x;
        sy += y;
        sxx += x * x;
        sxy += x * y;
        syy += y * y;
    }
    let slope = (n * sxy - sx * sy) / (n * sxx - sx * sx);
    let var_y = syy - sy * sy / n;
    let r2 = if var_y > 0.0 { slope * (sxy - sx * sy / n) / var_y } else { f64::NAN };
    (-slope, r2)
}

/// The ensemble mean of a readout's trace over `seeds` seeds, `steps + 1` long. The seeds are
/// a counter stream off `seed`, so an ensemble is reproducible from its first member.
pub fn ensemble_shear(
    model: &Model,
    law: &[u8],
    l: usize,
    d: f64,
    a: f64,
    k_index: usize,
    steps: usize,
    seed: u64,
    seeds: usize,
) -> Vec<f64> {
    let mut acc = vec![0.0f64; steps + 1];
    for s in 0..seeds {
        let sd = seed ^ (s as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let (v, _, _) =
            shear_series(model, law, l, d, a, k_index, steps, sd, false, row_drive(), false);
        for (t, x) in v.iter().enumerate() {
            acc[t] += x / seeds as f64;
        }
    }
    acc
}

/// The seed of ensemble member `s`, a counter stream off the declared seed. One spelling.
pub fn ensemble_seed(seed: u64, s: usize) -> u64 {
    seed ^ (s as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

fn project_ensemble(lats: &[Lattice], cells: &[Vec<u8>], sine: &[f64], along_columns: bool) -> f64 {
    let n = lats.len() as f64;
    lats.iter()
        .zip(cells)
        .map(|(x, c)| project_momenta(x, c, sine, along_columns))
        .sum::<f64>()
        / n
}

fn project_colour_ensemble(l: usize, cells: &[Vec<u8>], colour: &[Vec<u8>], sine: &[f64]) -> f64 {
    let n = cells.len() as f64;
    cells
        .iter()
        .zip(colour)
        .map(|(c, q)| project_colour_excess(l, c, q, sine))
        .sum::<f64>()
        / n
}

/// The shear reading of `seeds` members averaged BEFORE the fit, advanced in lockstep so the
/// window can still close on the ensemble's own amplitude and the run stop there.
///
/// `FLUID0_AMENDMENT_1`'s shear readout. Every member is audited against its OWN initial
/// ledger at every step; the reported integers are member zero's.
pub fn shear_ensemble(
    model: &Model,
    law: &[u8],
    l: usize,
    d: f64,
    a: f64,
    k_index: usize,
    cap: usize,
    seed: u64,
    seeds: usize,
    along_columns: bool,
    drive: [f64; 2],
    rule: &FitRule,
) -> (TransportReading, LedgerAudit) {
    assert!(seeds >= 1, "an ensemble needs at least one member");
    let sine = sine_table(l, k_index);
    let lats: Vec<Lattice> = (0..seeds)
        .map(|s| {
            Lattice::seeded_shear_drive(
                model.clone(),
                l,
                ensemble_seed(seed, s),
                d,
                a,
                k_index,
                law.to_vec(),
                drive,
                along_columns,
            )
        })
        .collect();
    let mut cells: Vec<Vec<u8>> = lats.iter().map(|x| x.cells.clone()).collect();
    let mut outs: Vec<Vec<u8>> = (0..seeds).map(|_| vec![0u8; l * l]).collect();
    let l0: Vec<crate::lattice::Ledger> =
        lats.iter().zip(&cells).map(|(x, c)| x.ledger_of(c)).collect();
    let mut audit = LedgerAudit {
        mass_exact: true,
        px_exact: true,
        py_exact: true,
        red_exact: true,
        steps_checked: 0,
        mass: l0[0].mass,
        px: l0[0].momentum[0],
        py: l0[0].momentum[1],
        red: 0,
    };
    let mut series = Vec::with_capacity(cap + 1);
    series.push(project_ensemble(&lats, &cells, &sine, along_columns));
    let mut fired = 0u64;
    let mut threshold = f64::NAN;
    for t in 0..cap {
        for s in 0..seeds {
            let st = lats[s].advance(&mut cells[s], &mut outs[s], t as u64);
            fired += st.collisions_fired;
            core::mem::swap(&mut cells[s], &mut outs[s]);
            let led = lats[s].ledger_of(&cells[s]);
            audit.mass_exact &= led.mass == l0[s].mass;
            audit.px_exact &= led.momentum[0] == l0[s].momentum[0];
            audit.py_exact &= led.momentum[1] == l0[s].momentum[1];
        }
        audit.steps_checked += 1;
        series.push(project_ensemble(&lats, &cells, &sine, along_columns));
        if t + 1 == rule.start {
            threshold = (rule.end_fraction * series[rule.start].abs()).max(rule.floor);
        }
        if t + 1 > rule.start && series[t + 1].abs() < threshold {
            break;
        }
    }
    (fit_with(&series, rule, wavenumber(l, k_index), fired), audit)
}

/// The tracer reading of `seeds` members averaged before the fit, as [`shear_ensemble`] is for
/// the shear. The red count of every member is audited against its own initial count.
pub fn colour_ensemble(
    model: &Model,
    law: &[u8],
    l: usize,
    d: f64,
    a: f64,
    k_index: usize,
    cap: usize,
    seed: u64,
    seeds: usize,
    colour_rule: ColourRule,
    rule: &FitRule,
) -> (TransportReading, LedgerAudit) {
    assert!(seeds >= 1, "an ensemble needs at least one member");
    let sine = sine_table(l, k_index);
    let lats: Vec<Lattice> = (0..seeds)
        .map(|s| Lattice::seeded(model.clone(), l, ensemble_seed(seed, s), d, law.to_vec()))
        .collect();
    let mut cells: Vec<Vec<u8>> = lats.iter().map(|x| x.cells.clone()).collect();
    let mut colour: Vec<Vec<u8>> =
        (0..seeds).map(|s| lats[s].seed_colour_wave(ensemble_seed(seed, s), a, k_index)).collect();
    let mut outs: Vec<Vec<u8>> = (0..seeds).map(|_| vec![0u8; l * l]).collect();
    let mut out_colour: Vec<Vec<u8>> = (0..seeds).map(|_| vec![0u8; l * l]).collect();
    let l0: Vec<crate::lattice::Ledger> =
        lats.iter().zip(&cells).map(|(x, c)| x.ledger_of(c)).collect();
    let red0: Vec<i64> =
        colour.iter().map(|q| q.iter().map(|&b| b.count_ones() as i64).sum()).collect();
    let mut audit = LedgerAudit {
        mass_exact: true,
        px_exact: true,
        py_exact: true,
        red_exact: true,
        steps_checked: 0,
        mass: l0[0].mass,
        px: l0[0].momentum[0],
        py: l0[0].momentum[1],
        red: red0[0],
    };
    let mut series = Vec::with_capacity(cap + 1);
    series.push(project_colour_ensemble(l, &cells, &colour, &sine));
    let mut fired = 0u64;
    let mut threshold = f64::NAN;
    for t in 0..cap {
        for s in 0..seeds {
            let st = lats[s].advance_with_colour_rule(
                &mut cells[s],
                &mut colour[s],
                &mut outs[s],
                &mut out_colour[s],
                t as u64,
                colour_rule,
            );
            fired += st.collisions_fired;
            core::mem::swap(&mut cells[s], &mut outs[s]);
            core::mem::swap(&mut colour[s], &mut out_colour[s]);
            let led = lats[s].ledger_of(&cells[s]);
            let red: i64 = colour[s].iter().map(|&b| b.count_ones() as i64).sum();
            audit.mass_exact &= led.mass == l0[s].mass;
            audit.px_exact &= led.momentum[0] == l0[s].momentum[0];
            audit.py_exact &= led.momentum[1] == l0[s].momentum[1];
            audit.red_exact &= red == red0[s];
        }
        audit.steps_checked += 1;
        series.push(project_colour_ensemble(l, &cells, &colour, &sine));
        if t + 1 == rule.start {
            threshold = (rule.end_fraction * series[rule.start].abs()).max(rule.floor);
        }
        if t + 1 > rule.start && series[t + 1].abs() < threshold {
            break;
        }
    }
    (fit_with(&series, rule, wavenumber(l, k_index), fired), audit)
}

/// Do two wavevectors agree within [`K_AGREEMENT_TOLERANCE`]? `None` unless both are `Fitted`.
pub fn k_agree(a: &TransportReading, b: &TransportReading) -> Option<bool> {
    match (a.value, b.value) {
        (Some(x), Some(y)) if x.abs() > 0.0 || y.abs() > 0.0 => {
            Some((x - y).abs() <= K_AGREEMENT_TOLERANCE * x.abs().max(y.abs()))
        }
        _ => None,
    }
}

/// The Boltzmann kinematic viscosity of FHP-I at density `d` — Frisch, d'Humières,
/// Hasslacher, Lallemand, Pomeau & Rivet 1987, Complex Systems **1**, 649; Hénon 1987.
/// TYPED FROM THE LITERATURE, so it is a credited comparison and never a gate.
pub fn boltzmann_nu_fhp_i(d: f64) -> f64 {
    1.0 / (12.0 * d * (1.0 - d).powi(3)) - 0.125
}

/// The Euclidean drive direction the freeze's row seeding uses, exposed for the runner's
/// record and for the isotropy leg's arithmetic.
pub fn row_drive() -> [f64; 2] {
    [1.0, 0.0]
}

/// `Σ_d (c_d · drive)·c_d` — the momentum density the shear seeding actually produces, per
/// unit amplitude. On the hexagon this is `3·drive` for any drive, which is why the row and
/// column legs are the same experiment rotated.
pub fn drive_response(model: &Model, drive: [f64; 2]) -> [f64; 2] {
    let mut r = [0.0f64; 2];
    for v in embed(model) {
        let p = v[0] * drive[0] + v[1] * drive[1];
        r[0] += p * v[0];
        r[1] += p * v[1];
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    const L: usize = 64;
    const D: f64 = 0.2;
    const A: f64 = 0.05;
    const SEED: u64 = 0x464c_5549_4430;

    /// The hexagon's response to any drive is `3·drive`: the seeded wave is transverse by
    /// construction, on both legs, and the two legs are one rotation apart.
    #[test]
    fn the_shear_seeding_is_transverse_on_both_legs() {
        let m = Model::fhp6();
        for drive in [row_drive(), COLUMN_TRANSVERSE] {
            let r = drive_response(&m, drive);
            assert!((r[0] - 3.0 * drive[0]).abs() < 1e-12, "{r:?} vs {drive:?}");
            assert!((r[1] - 3.0 * drive[1]).abs() < 1e-12, "{r:?} vs {drive:?}");
        }
        // and the column drive is a unit vector 60 degrees from the row drive
        let c = COLUMN_TRANSVERSE;
        assert!((c[0] * c[0] + c[1] * c[1] - 1.0).abs() < 1e-12);
        assert!((c[0] - 0.5).abs() < 1e-12);
    }

    /// The red count is an integer identity through the colour step, on FHP-I in BOTH
    /// chiralities and on ten other census laws. The carrier is asserted: a run whose
    /// collisions never fire conserves colour trivially (M-VACUOUS-SUCCESS).
    #[test]
    fn the_red_count_is_conserved_through_the_colour_step() {
        let m = Model::fhp6();
        let mut laws: Vec<Vec<u8>> = vec![m.fhp_i(true), m.fhp_i(false)];
        let census = m.collision_laws();
        // Ten laws that are NOT the identity, spread through the enumeration.
        let identity = m.identity_collision();
        laws.extend(
            census
                .iter()
                .filter(|c| **c != identity)
                .step_by(419)
                .take(10)
                .cloned(),
        );
        assert_eq!(laws.len(), 12);
        for (n, law) in laws.iter().enumerate() {
            let lat = Lattice::seeded(m.clone(), 32, SEED, 0.3, law.clone());
            let mut cells = lat.cells.clone();
            let mut colour = lat.seed_colour_wave(SEED, 0.4, 1);
            let mut out = vec![0u8; 32 * 32];
            let mut out_colour = vec![0u8; 32 * 32];
            let red0: i64 = colour.iter().map(|&q| q.count_ones() as i64).sum();
            assert!(red0 > 0, "law {n}: no red particle exists to conserve");
            let l0 = lat.ledger_of(&cells);
            let mut fired = 0u64;
            for t in 0..500u64 {
                let s = lat.advance_with_colour(&mut cells, &mut colour, &mut out, &mut out_colour, t);
                core::mem::swap(&mut cells, &mut out);
                core::mem::swap(&mut colour, &mut out_colour);
                fired += s.collisions_fired;
                let red: i64 = colour.iter().map(|&q| q.count_ones() as i64).sum();
                assert_eq!(red, red0, "law {n} lost red at step {t}");
                assert!(
                    colour.iter().zip(&cells).all(|(&q, &s)| q & !s == 0),
                    "law {n}: a colour bit outran its particle at step {t}"
                );
                let led = lat.ledger_of(&cells);
                assert_eq!(led.mass, l0.mass, "law {n} lost mass at step {t}");
                assert_eq!(led.momentum, l0.momentum, "law {n} lost momentum at step {t}");
            }
            assert!(fired > 0, "law {n}: no collision fired, so nothing was conserved");
        }
    }

    /// The plant's rule conserves the red count too — it removes the RANDOMISATION and
    /// nothing else, so a change in `D` cannot be a change in the tracer's mass.
    #[test]
    fn the_ordinal_plant_conserves_the_red_count() {
        let m = Model::fhp6();
        let lat = Lattice::seeded(m.clone(), 32, SEED, 0.3, m.fhp_i(true));
        let mut cells = lat.cells.clone();
        let mut colour = lat.seed_colour_wave(SEED, 0.4, 1);
        let mut out = vec![0u8; 32 * 32];
        let mut out_colour = vec![0u8; 32 * 32];
        let red0: i64 = colour.iter().map(|&q| q.count_ones() as i64).sum();
        for t in 0..300u64 {
            lat.advance_with_colour_rule(
                &mut cells,
                &mut colour,
                &mut out,
                &mut out_colour,
                t,
                ColourRule::Ordinal,
            );
            core::mem::swap(&mut cells, &mut out);
            core::mem::swap(&mut colour, &mut out_colour);
            let red: i64 = colour.iter().map(|&q| q.count_ones() as i64).sum();
            assert_eq!(red, red0, "the ordinal plant lost red at step {t}");
        }
    }

    /// G1's shape, as a unit test: the identity law damps nothing, on either readout, and
    /// the reason is that no collision ever fires.
    #[test]
    fn the_identity_law_refuses_on_both_readouts() {
        let m = Model::fhp6();
        let id = m.identity_collision();
        let s = shear_viscosity(&m, &id, L, D, A, 1, 1_000, SEED, false);
        let c = tracer_diffusion(&m, &id, L, D, A, 1, 1_000, SEED);
        assert_eq!(s.exit, Exit::NoDecay, "shear: {s:?}");
        assert_eq!(c.exit, Exit::NoDecay, "colour: {c:?}");
        assert_eq!(s.collisions_fired, 0);
        assert_eq!(c.collisions_fired, 0);
        assert!(s.value.is_none() && c.value.is_none());
    }

    /// The digest is a function of the table and separates the laws it must.
    #[test]
    fn the_digest_separates_the_laws() {
        let m = Model::fhp6();
        assert_ne!(law_digest(&m.fhp_i(true)), law_digest(&m.fhp_i(false)));
        assert_ne!(law_digest(&m.fhp_i(true)), law_digest(&m.identity_collision()));
        assert_eq!(law_digest(&m.fhp_i(true)), law_digest(&m.fhp_i(true)));
        let laws = m.collision_laws();
        let mut d: Vec<u64> = laws.iter().map(|c| law_digest(c)).collect();
        d.sort_unstable();
        d.dedup();
        assert_eq!(d.len(), laws.len(), "two laws share a digest");
    }

    /// The `k²` law on FHP-I: `ν` read at two wavevectors must agree, because `ν` is the
    /// thing and `k` is the knob. The freeze's own parameters.
    ///
    /// **THIS TEST FAILS TODAY AND IS IGNORED, NOT REPAIRED.** It is `FLUID0_PREREG` G0's
    /// requirement written as an assertion, and the instrument refuses both legs at these
    /// parameters. The measured reason is in the three tests below and in
    /// `fluid0_sweep diag`: at `L = 64, d = 0.2` the mean free path is about ten links, so
    /// `k_index = 2` is not hydrodynamic (ensemble `ν(k=1) = 0.509` against
    /// `ν(k=2) = 0.33..0.37`, a 30 % gap that no amount of averaging closes), the shear
    /// window's own closing threshold sits under the instrument's noise floor, and the
    /// colour mode is 99 % gone before the freeze's transient ends. Un-ignore it when the
    /// freeze's parameters change, and it becomes the check that the change worked.
    #[ignore = "FLUID0 G0 fails at the frozen parameters: k=2 is pre-hydrodynamic at L=64 and \
                the window closes under the noise floor — see the three tests below"]
    #[test]
    fn fhp_i_reads_the_same_viscosity_at_two_wavevectors() {
        let m = Model::fhp6();
        let law = m.fhp_i(true);
        let r1 = shear_viscosity(&m, &law, L, D, A, 1, 4_000, SEED, false);
        let r2 = shear_viscosity(&m, &law, L, D, A, 2, 4_000, SEED, false);
        let (v1, v2) = (
            r1.value.unwrap_or_else(|| panic!("k=1 refused: {r1:?}")),
            r2.value.unwrap_or_else(|| panic!("k=2 refused: {r2:?}")),
        );
        assert!(
            (v1 - v2).abs() <= K_AGREEMENT_TOLERANCE * v1.abs().max(v2.abs()),
            "nu(k=1) {v1} vs nu(k=2) {v2}"
        );
    }

    /// A one-member ensemble IS the single run, on both readouts and to the bit. The
    /// lockstep ensemble is the amended instrument's whole new mechanism, so it is pinned
    /// against the machinery the frozen record was made with rather than trusted.
    #[test]
    fn a_one_member_ensemble_reproduces_the_single_run() {
        let m = Model::fhp6();
        let law = m.fhp_i(true);
        let rule = FitRule::frozen(L);
        let (a, aa) = shear_viscosity_driven(&m, &law, L, D, A, 1, 900, SEED, false, row_drive());
        let (b, ba) =
            shear_ensemble(&m, &law, L, D, A, 1, 900, SEED, 1, false, row_drive(), &rule);
        assert_eq!(a.exit, b.exit, "shear exits differ: {a:?} vs {b:?}");
        assert_eq!(a.window, b.window, "shear windows differ");
        assert_eq!(a.collisions_fired, b.collisions_fired, "shear collision counts differ");
        assert_eq!(a.start_amplitude.to_bits(), b.start_amplitude.to_bits());
        assert_eq!(aa.steps_checked, ba.steps_checked);
        assert!(aa.all_exact() && ba.all_exact());

        let (c, ca) =
            tracer_diffusion_rule(&m, &law, L, D, A, 1, 900, SEED, ColourRule::Blind);
        let (d, da) =
            colour_ensemble(&m, &law, L, D, A, 1, 900, SEED, 1, ColourRule::Blind, &rule);
        assert_eq!(c.exit, d.exit, "colour exits differ: {c:?} vs {d:?}");
        assert_eq!(c.window, d.window, "colour windows differ");
        assert_eq!(c.collisions_fired, d.collisions_fired, "colour collision counts differ");
        assert_eq!(c.start_amplitude.to_bits(), d.start_amplitude.to_bits());
        assert_eq!(ca.red, da.red);
        assert!(ca.all_exact() && da.all_exact());
        assert!(c.collisions_fired > 0, "no collision fired: the comparison is vacuous");
    }

    /// The window rule is DATA, and changing it changes the reading: the amended rule opens
    /// at a different step and closes on a floor, and it must not silently agree with the
    /// frozen one. A rule that made no difference would not be worth an amendment.
    #[test]
    fn the_fit_rule_is_data_and_moves_the_window() {
        let m = Model::fhp6();
        let law = m.fhp_i(true);
        let (series, fired, _) =
            shear_series(&m, &law, L, D, A, 1, 900, SEED, false, row_drive(), false);
        let frozen = fit(&series, L, wavenumber(L, 1), fired);
        let amended = fit_with(
            &series,
            &FitRule { start: 46, end_fraction: 0.2, floor: 12.0 * 33.99, ..FitRule::frozen(L) },
            wavenumber(L, 1),
            fired,
        );
        assert_ne!(frozen.window, amended.window, "the two rules chose the same window");
        assert_eq!(frozen.window.0, L);
        assert_eq!(amended.window.0, 46);
    }

    /// Why the shear leg refuses, measured: the window's own closing threshold — a tenth of
    /// the amplitude at step `L` — sits UNDER the instrument's equilibrium noise, which is
    /// read on the same lattice with the wave amplitude set to zero. The last part of every
    /// fit is therefore a coin toss, and the monotone test fires on it.
    #[test]
    fn the_shear_windows_floor_sits_under_the_instruments_own_noise() {
        let m = Model::fhp6();
        let law = m.fhp_i(true);
        let (flat, _, _) =
            shear_series(&m, &law, L, D, 0.0, 1, 400, SEED, false, row_drive(), false);
        let noise =
            (flat[L..].iter().map(|v| v * v).sum::<f64>() / (flat.len() - L) as f64).sqrt();
        let (sig, _, _) = shear_series(&m, &law, L, D, A, 1, 400, SEED, false, row_drive(), false);
        let threshold = WINDOW_END_FRACTION * sig[L].abs();
        assert!(noise > 0.0, "the A=0 control read exactly zero: the noise floor is not measured");
        assert!(
            threshold < noise,
            "the window now closes ABOVE the noise ({threshold} vs {noise}) — the frozen \
             parameters changed and G0's shear leg may now be readable"
        );
    }

    /// Why the colour leg refuses, measured: the tracer mode's lifetime is far shorter than
    /// the `L`-step transient the freeze opens its window after, so by the time the window
    /// opens the ensemble-mean signal is gone. Read on a seed ensemble, where the answer is
    /// about the MODE and not about one seed's fluctuation.
    #[test]
    fn the_colour_mode_dies_before_the_freezes_transient_ends() {
        let m = Model::fhp6();
        let law = m.fhp_i(true);
        let seeds = 32usize;
        let mut acc = vec![0.0f64; L + 1];
        let mut flat = vec![0.0f64; L + 1];
        for s in 0..seeds {
            let sd = SEED ^ (s as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let (v, fired, _) =
                colour_series(&m, &law, L, D, A, 1, L, sd, ColourRule::Blind, false);
            assert!(fired > 0, "no collision fired: the tracer never mixed");
            // The SAME ensemble with the wave amplitude set to zero: what is left at step L
            // when there is nothing to measure. Its scale is the residual noise floor.
            let (z, _, _) = colour_series(&m, &law, L, D, 0.0, 1, L, sd, ColourRule::Blind, false);
            for t in 0..=L {
                acc[t] += v[t] / seeds as f64;
                flat[t] += z[t] / seeds as f64;
            }
        }
        let residual = (flat[L / 2..].iter().map(|v| v * v).sum::<f64>()
            / (L - L / 2 + 1) as f64)
            .sqrt();
        assert!(acc[0].abs() > 10.0, "the colour wave was never seeded: |M(0)| = {}", acc[0]);
        assert!(residual > 0.0, "the A=0 control read exactly zero: no noise floor is measured");
        assert!(
            acc[L].abs() < 0.10 * acc[0].abs() && acc[L].abs() < 3.0 * residual,
            "the colour mode survived the transient: |M(L)| = {} of |M(0)| = {}, residual \
             noise {} — the frozen parameters changed",
            acc[L].abs(),
            acc[0].abs(),
            residual
        );
    }

    /// The finite-size control the freeze calls `PreHydrodynamic`, on FHP-I itself and with
    /// the noise averaged away: `ν` read at `k_index = 2` is far from `ν` at `k_index = 1`,
    /// because at `L = 64, d = 0.2` the mean free path is about ten links and the `k = 2`
    /// wavelength is 27.7. The `k²` law is not a statistics problem here.
    #[test]
    fn the_two_wavevectors_disagree_on_fhp_i_even_with_the_noise_averaged_away() {
        let m = Model::fhp6();
        let law = m.fhp_i(true);
        let e1 = ensemble_shear(&m, &law, L, D, A, 1, 300, SEED, 32);
        let e2 = ensemble_shear(&m, &law, L, D, A, 2, 200, SEED, 32);
        let (g1, r1) = log_slope(&e1, 64, 300);
        let (g2, r2) = log_slope(&e2, 16, 75);
        let (v1, v2) = (g1 / wavenumber(L, 1).powi(2), g2 / wavenumber(L, 2).powi(2));
        assert!(r1 > 0.9 && r2 > 0.9, "the characterisation fits are not lines: R2 {r1} / {r2}");
        assert!(v1 > 0.4 && v1 < 0.6, "nu(k=1) moved off its measured 0.509: {v1}");
        assert!(
            (v1 - v2).abs() > K_AGREEMENT_TOLERANCE * v1.max(v2),
            "the two wavevectors now AGREE ({v1} vs {v2}) — the frozen parameters changed and \
             G0's finite-size control may now pass"
        );
    }
}
