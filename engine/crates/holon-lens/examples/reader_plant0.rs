//! READER-PLANT-0 — the record's own relaxation reader, fed planted modes at the banked
//! signal-to-noise.
//!
//! UNBUILT as of 2026-11-23: written under a no-build day; the first `cargo run` of this
//! example is the first time it compiled. It was written from the source of
//! `holon_lens::field::{fit_relaxation, align_cycles, Relaxation, Aligned}` as read at
//! `8b9464d`, and calls those functions unchanged. It reads no trajectory and no banked
//! series; every input is a constant below, cited in
//! `conformance/water_observatory/READER_PLANT0_PREREG_DRAFT.md`.
//!
//! What it prints is graded against `conformance/water_observatory/reader_plant0/
//! port_prediction.out`, the output of a python port committed BEFORE this was built. The
//! arms, forms, noise models and the two paths are the port's, one for one; the random
//! streams are not (numpy's generator there, an LCG with Box–Muller here), so agreement is
//! statistical and the draft states the binomial tolerance.
//!
//! Run (when building is allowed):
//!   cargo run -p holon-lens --release --example reader_plant0
use holon_lens::field::{align_cycles, fit_relaxation, Relaxation};

const DT: f64 = 10.0082e-15;
const RELAX: usize = 314;
const CYCLES: usize = 12;
const TRIALS: usize = 2000;
const G_U: f64 = 2.0e12;
const W_U: f64 = 8.8e12;
const L1: f64 = 5.5e12;
const L2: f64 = 1.04e13;
const L_SINGLE: f64 = 1.0e13;
/// The equilibrium law: the fit to the banked density autocorrelation (see the port's header).
const G_E: f64 = 5.0e12;
const W_E: f64 = 5.9e12;
const SNR_LOW: [f64; 3] = [6.4, 6.8, 9.4];
const SNR_HIGH: [f64; 3] = [29.4, 38.4, 76.1];
const SUB: usize = 16;

/// The crate's own generator (the LCG its unit tests use), with Box–Muller on top.
struct Rng { s: u64, spare: Option<f64> }
impl Rng {
    fn new(seed: u64) -> Rng { Rng { s: seed, spare: None } }
    fn unit(&mut self) -> f64 {
        self.s = self.s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (((self.s >> 11) as f64) + 0.5) / ((1u64 << 53) as f64)   // in (0, 1): never 0, so ln is safe
    }
    fn normal(&mut self) -> f64 {
        if let Some(z) = self.spare.take() { return z; }
        let (u1, u2) = (self.unit(), self.unit());
        let r = (-2.0 * u1.ln()).sqrt();
        let a = 2.0 * std::f64::consts::PI * u2;
        self.spare = Some(r * a.sin());
        r * a.cos()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Form { U, Uc, E, O2, O1, N }
impl Form {
    fn name(self) -> &'static str { match self { Form::U => "U ", Form::Uc => "Uc", Form::E => "E ", Form::O2 => "O2", Form::O1 => "O1", Form::N => "N " } }
    /// One cycle of the mode after a unit kick.
    fn series(self) -> Vec<f64> {
        (0..RELAX).map(|i| {
            let t = i as f64 * DT;
            match self {
                Form::U => (-G_U * t).exp() * ((W_U * t).cos() - G_U / W_U * (W_U * t).sin()),
                Form::Uc => (-G_U * t).exp() * (W_U * t).cos(),
                Form::E => (-G_E * t).exp() * ((W_E * t).cos() - G_E / W_E * (W_E * t).sin()),
                Form::O2 => (L2 * (-L2 * t).exp() - L1 * (-L1 * t).exp()) / (L2 - L1),
                Form::O1 => (-L_SINGLE * t).exp(),
                Form::N => 0.0,
            }
        }).collect()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Noise { W, C, F }
impl Noise { fn name(self) -> &'static str { match self { Noise::W => "W", Noise::C => "C", Noise::F => "F" } } }

fn white(rng: &mut Rng, n: usize) -> Vec<f64> { (0..n).map(|_| rng.normal()).collect() }

/// A unit-variance first-order process of correlation time `tau`.
fn ar1(rng: &mut Rng, n: usize, tau: f64) -> Vec<f64> {
    let a = (-DT / tau).exp();
    let b = (1.0 - a * a).sqrt();
    let mut x = Vec::with_capacity(n);
    let mut prev = rng.normal();
    x.push(prev);
    for _ in 1..n { prev = a * prev + b * rng.normal(); x.push(prev); }
    x
}

/// The velocity of `x'' + two_gamma x' + w0sq x = white force`, sampled every `DT`
/// (semi-implicit Euler, `SUB` substeps): the thermal noise a mode of that law carries.
/// Unnormalised; the same scheme as the port's `osc_velocity`.
fn osc_velocity(rng: &mut Rng, n: usize, two_gamma: f64, w0sq: f64) -> Vec<f64> {
    let h = DT / SUB as f64;
    let sh = h.sqrt();
    let (mut x, mut v) = (0.0f64, 0.0f64);
    let burn = 400 * SUB;
    let mut out = Vec::with_capacity(n);
    for k in 0..(burn + n * SUB) {
        v += h * (-two_gamma * v - w0sq * x) + sh * rng.normal();
        x += h * v;
        if k >= burn && (k - burn) % SUB == 0 { out.push(v); }
    }
    out
}

fn sd(v: &[f64]) -> f64 {
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64).sqrt()
}

/// The oscillator law a form's own thermal noise follows, if it has one.
fn osc_law(form: Form) -> Option<(f64, f64)> {
    match form {
        Form::U | Form::Uc => Some((2.0 * G_U, W_U * W_U + G_U * G_U)),
        Form::E => Some((2.0 * G_E, W_E * W_E + G_E * G_E)),
        Form::O2 => Some((L1 + L2, L1 * L2)),
        _ => None,
    }
}

/// Unit-SD noise of `n` readouts for one trial.
fn noise(rng: &mut Rng, kind: Noise, n: usize, form: Form, osc_sd: f64) -> Vec<f64> {
    match kind {
        Noise::W => white(rng, n),
        Noise::C => ar1(rng, n, 50e-15),
        Noise::F => match osc_law(form) {
            Some((tg, w0)) => osc_velocity(rng, n, tg, w0).into_iter().map(|v| v / osc_sd).collect(),
            None => if form == Form::O1 { ar1(rng, n, 1.0 / L_SINGLE) } else { white(rng, n) },
        },
    }
}

#[derive(Default)]
struct Tally { over: usize, under: usize, refused: usize, pts: Vec<usize>, rates: Vec<f64> }
impl Tally {
    fn add(&mut self, r: &Relaxation) {
        match r {
            Relaxation::Overdamped { lambda, points, .. } => { self.over += 1; self.pts.push(*points); self.rates.push(*lambda); }
            Relaxation::Underdamped { .. } => self.under += 1,
            Relaxation::Refused { .. } => self.refused += 1,
        }
    }
    fn n(&self) -> f64 { (self.over + self.under + self.refused).max(1) as f64 }
    fn p_over(&self) -> f64 { self.over as f64 / self.n() }
    fn p_under(&self) -> f64 { self.under as f64 / self.n() }
    fn p_ref(&self) -> f64 { self.refused as f64 / self.n() }
    fn median_pts(&self) -> usize { if self.pts.is_empty() { return 0; } let mut p = self.pts.clone(); p.sort_unstable(); p[p.len() / 2] }
    fn median_rate(&self) -> f64 { median(&self.rates) }
}

fn median(v: &[f64]) -> f64 {
    let mut w: Vec<f64> = v.iter().cloned().filter(|x| x.is_finite()).collect();
    if w.is_empty() { return f64::NAN; }
    w.sort_by(|a, b| a.partial_cmp(b).unwrap());
    w[w.len() / 2]
}

/// Path A: one cycle, the noise passed to the reader as known (the reviewer's port).
fn path_a(rng: &mut Rng, form: Form, kind: Noise, snr: f64, osc_sd: f64) -> Tally {
    let sig = form.series();
    let mut t = Tally::default();
    for _ in 0..TRIALS {
        let nz = noise(rng, kind, RELAX, form, osc_sd);
        let y: Vec<f64> = sig.iter().zip(&nz).map(|(s, e)| s + e / snr).collect();
        t.add(&fit_relaxation(&y, DT, 1.0 / snr));
    }
    t
}

/// Path B: the reader as it ran RESPONSE-1 — a pre-kick row, twelve cycles of alternating
/// sign, `align_cycles`, and `fit_relaxation` on the aligned mean with the reader's own
/// tail noise. The per-cycle noise is `sqrt(12)/snr` for a unit kick, so the EXPECTED
/// aligned SNR is `snr`. Also returns the median realised SNR and the per-cycle tally
/// (the first 200 trials' cycles, each fitted with the per-cycle noise, as `rung2` does).
fn path_b(rng: &mut Rng, form: Form, kind: Noise, snr: f64, osc_sd: f64) -> (Tally, f64, Tally) {
    let sig = form.series();
    let sc = (CYCLES as f64).sqrt() / snr;
    let (mut t, mut per_cycle) = (Tally::default(), Tally::default());
    let mut realised = Vec::with_capacity(TRIALS);
    for trial in 0..TRIALS {
        let nz = noise(rng, kind, 1 + CYCLES * RELAX, form, osc_sd);
        let mut s: Vec<f64> = nz.iter().map(|e| e * sc).collect();
        for c in 0..CYCLES {
            let sign = if c % 2 == 0 { 1.0 } else { -1.0 };
            for i in 0..RELAX { s[1 + c * RELAX + i] += sign * sig[i]; }
        }
        let al = align_cycles(&s, CYCLES, RELAX, 1);
        t.add(&fit_relaxation(&al.mean, DT, al.noise));
        realised.push(al.mean[0].abs() / al.noise);
        if trial < 200 { for cyc in &al.per_cycle { per_cycle.add(&fit_relaxation(cyc, DT, al.noise_per_cycle)); } }
    }
    (t, median(&realised), per_cycle)
}

fn label(r: &Relaxation) -> String {
    match r {
        Relaxation::Overdamped { points, .. } => format!("OVER/{points}"),
        Relaxation::Underdamped { points, .. } => format!("UNDER/{points}"),
        Relaxation::Refused { .. } => "REF/0".to_string(),
    }
}

fn main() {
    println!("READER-PLANT-0 on the real reader; trials per cell {TRIALS}");
    let all: Vec<f64> = SNR_LOW.iter().chain(SNR_HIGH.iter()).cloned().collect();

    println!("\n[0] noise-free landings (the label a clean series gets when the stated noise is 1/SNR)");
    for form in [Form::U, Form::Uc, Form::E, Form::O2, Form::O1] {
        let y = form.series();
        let row: Vec<String> = all.iter().map(|s| format!("SNR {s}: {}", label(&fit_relaxation(&y, DT, 1.0 / s)))).collect();
        println!("  {}: {}", form.name(), row.join("  "));
    }

    // the oscillator noise's stationary SD, from one long fixed-seed run per law
    let mut cal = Rng::new(0x5245_4144_4552_3030);
    let sd_u = sd(&osc_velocity(&mut cal, 64 * 4000, 2.0 * G_U, W_U * W_U + G_U * G_U));
    let sd_o = sd(&osc_velocity(&mut cal, 64 * 4000, L1 + L2, L1 * L2));
    let sd_e = sd(&osc_velocity(&mut cal, 64 * 4000, 2.0 * G_E, W_E * W_E + G_E * G_E));
    let osc_sd = |form: Form| -> f64 { match form { Form::U | Form::Uc => sd_u, Form::E => sd_e, Form::O2 => sd_o, _ => 1.0 } };

    let mut rng = Rng::new(0x5245_4144_4552_3031);
    println!("\n[2] path A (one 314-readout series, the noise passed to the reader as known): form noise SNR | p_over p_under p_ref | median pts | median rate /s");
    for kind in [Noise::W, Noise::C] {
        for &snr in &all {
            let t = path_a(&mut rng, Form::U, kind, snr, osc_sd(Form::U));
            println!("  U  {} {snr:5.1} | {:.3} {:.3} {:.3} | {:3} | {:.3e}", kind.name(), t.p_over(), t.p_under(), t.p_ref(), t.median_pts(), t.median_rate());
        }
    }

    println!("\n[3] path B (twelve cycles aligned, the reader's own tail noise): form noise SNR | p_over p_under p_ref | median pts | median rate /s | median realised SNR | per-cycle p_over");
    // p_over and p_under per (form, noise, snr), in the order printed, for [4] and [5]
    let forms = [Form::U, Form::Uc, Form::E, Form::O2, Form::O1];
    let kinds = [Noise::W, Noise::C, Noise::F];
    let mut p_over = vec![vec![vec![0.0f64; all.len()]; kinds.len()]; forms.len()];
    let mut p_under = vec![vec![vec![0.0f64; all.len()]; kinds.len()]; forms.len()];
    for (fi, &form) in forms.iter().enumerate() {
        for (ki, &kind) in kinds.iter().enumerate() {
            for (si, &snr) in all.iter().enumerate() {
                let (t, realised, pc) = path_b(&mut rng, form, kind, snr, osc_sd(form));
                p_over[fi][ki][si] = t.p_over();
                p_under[fi][ki][si] = t.p_under();
                println!("  {} {} {snr:5.1} | {:.3} {:.3} {:.3} | {:3} | {:.3e} | {realised:5.1} | {:.3}", form.name(), kind.name(), t.p_over(), t.p_under(), t.p_ref(), t.median_pts(), t.median_rate(), pc.p_over());
            }
        }
    }
    for &snr in &SNR_LOW {
        let (t, realised, pc) = path_b(&mut rng, Form::N, Noise::W, snr, 1.0);
        println!("  N  W {snr:5.1} | {:.3} {:.3} {:.3} | {:3} | {:.3e} | {realised:5.1} | {:.3}", t.p_over(), t.p_under(), t.p_ref(), t.median_pts(), t.median_rate(), pc.p_over());
    }

    println!("\n[4] the graded quantity J = p_over(6.4) x p_over(6.8) x p_over(9.4), path B");
    for (fi, &form) in forms.iter().enumerate() {
        let row: Vec<String> = kinds.iter().enumerate().map(|(ki, k)| format!("{}: {:.3}", k.name(), p_over[fi][ki][0] * p_over[fi][ki][1] * p_over[fi][ki][2])).collect();
        println!("  {}: {}", form.name(), row.join("  "));
    }
    println!("\n[5] K = p_under(29.4) x p_under(38.4) x p_under(76.1), path B");
    for (fi, &form) in forms.iter().enumerate() {
        let row: Vec<String> = kinds.iter().enumerate().map(|(ki, k)| format!("{}: {:.3}", k.name(), p_under[fi][ki][3] * p_under[fi][ki][4] * p_under[fi][ki][5])).collect();
        println!("  {}: {}", form.name(), row.join("  "));
    }
    println!("done.");
}
