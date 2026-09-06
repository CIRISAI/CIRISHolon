//! FLUID-0's amendment probe: the numbers an amended instrument has to be built from.
//!
//! NOT a gate, NOT in `FLUID0_PREREG`, and it touches nothing the runner writes. It exists so
//! that `FLUID0_AMENDMENT_1`'s lattice size, seed count, window rules, wavevector set and
//! price are read off measurements instead of typed. Everything it prints is an ensemble mean
//! fitted by plain least squares with no floors — a characterisation, never a reading.
//!
//! Usage: `fluid0_probe [threads] [pass]`, defaults 8 and `all`. Run pinned to cores 24-31.
//! `pass` is `1` (the curves), `2` (the rules), or `all`.

// `t` is a time coordinate in every fit below, not merely an index into a slice.
#![allow(clippy::needless_range_loop)]

use holon_lattice::lattice::ColourRule;
use holon_lattice::state::Model;
use holon_lattice::transport::{colour_series, log_slope, row_drive, shear_series, wavenumber};
use holon_lattice::Lattice;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

const D: f64 = 0.2;
const A_SHEAR: f64 = 0.05;
const SEED: u64 = 0x464c_5549_4430;
/// FLUID-0's diagnostic coefficients, used ONLY to size run lengths, never as a result.
const NU_GUESS: f64 = 0.6;
const D_GUESS: f64 = 5.7;
/// The kinetic clock: `tau = 2D / v²` with `v = 1` link/step, from the MEASURED `D`.
const TAU: f64 = 2.0 * D_GUESS;

fn seed_of(s: usize) -> u64 {
    SEED ^ (s as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

fn parallel<T: Send, F: Fn(usize) -> T + Sync>(n: usize, threads: usize, f: F) -> Vec<T> {
    let next = AtomicUsize::new(0);
    let slots: Vec<std::sync::Mutex<Option<T>>> =
        (0..n).map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= n {
                    break;
                }
                *slots[i].lock().unwrap() = Some(f(i));
            });
        }
    });
    slots.into_iter().map(|m| m.into_inner().unwrap().unwrap()).collect()
}

/// Every member trace of an ensemble, kept apart so sub-ensembles cost nothing extra.
#[allow(clippy::too_many_arguments)]
fn traces(
    model: &Model,
    law: &[u8],
    l: usize,
    a: f64,
    k_index: usize,
    steps: usize,
    seeds: usize,
    colour: bool,
    threads: usize,
) -> Vec<Vec<f64>> {
    parallel(seeds, threads, |s| {
        if colour {
            colour_series(model, law, l, D, a, k_index, steps, seed_of(s), ColourRule::Blind, false).0
        } else {
            shear_series(model, law, l, D, a, k_index, steps, seed_of(s), false, row_drive(), false).0
        }
    })
}

fn mean_of(parts: &[Vec<f64>], take: usize) -> Vec<f64> {
    let n = parts[0].len();
    let mut acc = vec![0.0f64; n];
    for p in parts.iter().take(take) {
        for (t, v) in p.iter().enumerate() {
            acc[t] += v / take as f64;
        }
    }
    acc
}

/// Open at `t0`, close at the last step at or above `frac` of the amplitude there.
fn window_fit(series: &[f64], t0: usize, frac: f64) -> (usize, usize, f64, f64, f64, f64) {
    let a0 = series[t0].abs();
    let mut t1 = series.len() - 1;
    for t in (t0 + 1)..series.len() {
        if series[t].abs() < frac * a0 {
            t1 = t;
            break;
        }
    }
    let (g, r2) = log_slope(series, t0, t1);
    (t0, t1, g, r2, a0, series[t1].abs())
}

fn steps_for(t0: usize, coeff: f64, k: f64, lifetimes: f64) -> usize {
    (t0 as f64 + lifetimes / (coeff * k * k)).ceil() as usize
}

fn main() {
    let mut a = std::env::args().skip(1);
    let threads: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(8);
    let pass = a.next().unwrap_or_else(|| "all".to_string());
    let m = Model::fhp6();
    let law = m.fhp_i(true);
    let t0_rule = (4.0 * TAU).ceil() as usize;
    println!("FLUID-0 AMENDMENT PROBE — characterisation only, never a reading. threads={threads}");
    println!("tau = 2D/v^2 = {TAU:.3} steps (from the measured D); 4 tau = {t0_rule} steps");

    if pass == "1" || pass == "all" {
        pass1(&m, &law, threads, t0_rule);
    }
    if pass == "2" || pass == "all" {
        pass2(&m, &law, threads, t0_rule);
    }
    if pass == "3" || pass == "all" {
        pass3(&m, &law, threads, t0_rule);
    }
}

/// The census's own cost spread: a law's collision rate sets its viscosity, its viscosity sets
/// its mode lifetime, and its mode lifetime sets what it costs. Measured on all 4,608 laws,
/// then priced against eight laws actually run at the amended size.
fn pass3(m: &Model, law: &[u8], threads: usize, t0_rule: usize) {
    println!("\nJ. THE COLLISION RATE OF EVERY LAW (L=64, 200 steps, d={D}) — the cost driver");
    let laws = m.collision_laws();
    let rates = parallel(laws.len(), threads, |i| {
        let lat = Lattice::seeded(m.clone(), 64, SEED, D, laws[i].clone());
        let mut cells = lat.cells.clone();
        let mut out = vec![0u8; 64 * 64];
        let mut fired = 0u64;
        for t in 0..200u64 {
            fired += lat.advance(&mut cells, &mut out, t).collisions_fired;
            core::mem::swap(&mut cells, &mut out);
        }
        fired as f64 / (200.0 * 4096.0)
    });
    let fhp_rate = rates[laws.iter().position(|c| c == law).unwrap()];
    let id = m.identity_collision();
    let id_rate = rates[laws.iter().position(|c| *c == id).unwrap()];
    let mut sorted = rates.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |f: f64| sorted[((sorted.len() - 1) as f64 * f) as usize];
    println!("   min {:.6}  p10 {:.6}  median {:.6}  p90 {:.6}  max {:.6}", sorted[0], q(0.10), q(0.5), q(0.90), sorted[sorted.len() - 1]);
    println!("   FHP-I {fhp_rate:.6}   identity {id_rate:.6}   laws with rate 0: {}", rates.iter().filter(|r| **r == 0.0).count());
    // nu + 1/8 scales as 1 / (collision rate): the propagation term is law-independent.
    let nu_fhp = 0.5447f64;
    let est = |r: f64| if r > 0.0 { (nu_fhp + 0.125) * fhp_rate / r - 0.125 } else { f64::INFINITY };
    println!("   estimated nu (calibrated on FHP-I's measured 0.5447 at k_index=1, L=256):");
    println!("     min {:.4}  p10 {:.4}  median {:.4}  p90 {:.4}  max {:.4}", est(sorted[sorted.len()-1]), est(q(0.90)), est(q(0.5)), est(q(0.10)), est(sorted[1]));

    println!("\nK. EIGHT LAWS RUN AT THE AMENDED SIZE (L=256, 1 seed, A_shear=0.10, A_col=1.0,");
    println!("   k_index in {{1,2}}, window 4 tau .. a fifth, step cap 24000/6000 shear, 3000/800 colour)");
    println!("   law     rate      nu_est   nu_k1     D_k1      shear steps  colour steps  seconds  capped");
    let mut picks: Vec<usize> = Vec::new();
    let order: Vec<usize> = {
        let mut o: Vec<usize> = (0..laws.len()).collect();
        o.sort_by(|a, b| rates[*a].partial_cmp(&rates[*b]).unwrap());
        o
    };
    for f in [0.0f64, 0.05, 0.2, 0.4, 0.6, 0.8, 0.95, 1.0] {
        let i = order[((order.len() - 1) as f64 * f) as usize];
        if !picks.contains(&i) { picks.push(i); }
    }
    if !picks.contains(&laws.iter().position(|c| c == law).unwrap()) {
        picks.push(laws.iter().position(|c| c == law).unwrap());
    }
    let mut total = 0.0f64;
    for &i in &picks {
        let t = Instant::now();
        let mut ssteps = 0usize;
        let mut csteps = 0usize;
        let mut capped = false;
        let mut nu1 = f64::NAN;
        let mut d1 = f64::NAN;
        for (colour, ki, cap) in [(false, 1usize, 24000usize), (false, 2, 6000), (true, 1, 3000), (true, 2, 800)] {
            let k = wavenumber(256, ki);
            let e = mean_of(&traces(m, &laws[i], 256, if colour { 1.0 } else { 0.10 }, ki, cap, 1, colour, threads), 1);
            let (_, w1, g, _, _, _) = window_fit(&e, t0_rule, 0.2);
            if w1 == cap { capped = true; }
            if colour { csteps += w1; if ki == 1 { d1 = g / (k * k); } } else { ssteps += w1; if ki == 1 { nu1 = g / (k * k); } }
        }
        let secs = t.elapsed().as_secs_f64() * threads as f64;
        total += secs;
        println!("   {i:<7} {:<9.6} {:<8.4} {nu1:<9.4} {d1:<9.4} {ssteps:<12} {csteps:<13} {secs:<8.2} {capped}", rates[i], est(rates[i]));
    }
    println!("   mean WALL seconds per law over the {} sampled, at the FULL cap: {:.2}", picks.len(), total / picks.len() as f64 / threads as f64);

    // ---------------------------------------------------------------- L. the census's own cost
    // Cost is modelled from the MEASURED throughput and the MEASURED window end, because the
    // probe has to consume the whole cap to find the window end and the runner does not.
    const NS_SHEAR: f64 = 16.944;
    const NS_COLOUR: f64 = 23.890;
    const CELLS: f64 = 65536.0;
    let caps = [(false, 1usize, 12000usize), (false, 2, 3000), (true, 1, 1500), (true, 2, 400)];
    println!("\nL. THE CENSUS'S OWN COST — {} laws sampled evenly through the enumeration,", 48);
    println!("   L=256, 1 seed, A_shear=0.10, A_col=1.0, k in {{1,2}}, window 4 tau .. a fifth,");
    println!("   caps 12000/3000 shear and 1500/400 colour. Cost MODELLED from the measured");
    println!("   {NS_SHEAR} / {NS_COLOUR} ns per cell-update and the measured window end.");
    // A STRIDE of laws.len()/48 aliases with the enumeration's odometer: the last fibers vary
    // fastest, so an even stride holds them all at their identity permutation and samples one
    // corner of the group. 2411 is coprime to 4608 (= 2^9 * 3^2), so this walks the whole set.
    let sample: Vec<usize> = (0..48).map(|i| (i * 2411) % laws.len()).collect();
    let rows = parallel(sample.len(), threads, |si| {
        let i = sample[si];
        let mut steps = [0usize; 4];
        let mut capped = false;
        let mut vals = [f64::NAN; 4];
        for (n, &(colour, ki, cap)) in caps.iter().enumerate() {
            let k = wavenumber(256, ki);
            let e = mean_of(&traces(m, &laws[i], 256, if colour { 1.0 } else { 0.10 }, ki, cap, 1, colour, 1), 1);
            let (_, w1, g, r2, _, _) = window_fit(&e, t0_rule, 0.2);
            steps[n] = w1;
            capped |= w1 >= cap;
            vals[n] = if r2 >= 0.99 { g / (k * k) } else { f64::NAN };
        }
        let cost = ((steps[0] + steps[1]) as f64 * NS_SHEAR + (steps[2] + steps[3]) as f64 * NS_COLOUR)
            * CELLS * 1e-9;
        (i, rates[i], steps, capped, vals, cost)
    });
    let mut costs: Vec<f64> = rows.iter().map(|r| r.5).collect();
    costs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let capped_n = rows.iter().filter(|r| r.3).count();
    let fitted_n = rows.iter().filter(|r| r.4.iter().all(|v| v.is_finite())).count();
    let mean_cost = costs.iter().sum::<f64>() / costs.len() as f64;
    println!("   laws reaching a cap (a NoDecay refusal): {capped_n} of {}", rows.len());
    println!("   laws with all four readings at R2 >= 0.99: {fitted_n} of {}", rows.len());
    println!("   modelled single-core seconds per law: min {:.2}  median {:.2}  mean {:.2}  max {:.2}",
             costs[0], costs[costs.len() / 2], mean_cost, costs[costs.len() - 1]);
    for s in [1usize, 2, 4] {
        let total = mean_cost * s as f64 * laws.len() as f64;
        println!("   seeds {s}: census {:.0} s single core = {:.2} h; on 8 cores = {:.2} h",
                 total, total / 3600.0, total / 3600.0 / 8.0);
    }
    let sc: Vec<f64> = rows.iter().filter(|r| r.4.iter().all(|v| v.is_finite()))
        .map(|r| r.4[0] / r.4[2]).collect();
    if !sc.is_empty() {
        let mut v = sc.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("   Sc = nu(k1)/D(k1) over the {} sampled laws that fitted: min {:.4}  median {:.4}  max {:.4}",
                 v.len(), v[0], v[v.len() / 2], v[v.len() - 1]);
        println!("   (a 48-law PREVIEW at 1 seed, NOT the census and NOT a reading of S)");
    }
}

fn pass1(m: &Model, law: &[u8], threads: usize, t0_rule: usize) {
    println!("\nA. THROUGHPUT (single core, WITH the per-step ledger audit the runner does)");
    println!("   L     readout   steps  seconds     cell-updates    ns/cell-update");
    for l in [64usize, 128, 256] {
        for colour in [false, true] {
            let steps = 200usize;
            let t = Instant::now();
            if colour {
                colour_series(m, law, l, D, 0.5, 1, steps, SEED, ColourRule::Blind, false);
            } else {
                shear_series(m, law, l, D, A_SHEAR, 1, steps, SEED, false, row_drive(), false);
            }
            let e = t.elapsed().as_secs_f64();
            let cu = (steps * l * l) as f64;
            println!(
                "   {l:<5} {:<9} {steps:<6} {e:<11.4} {cu:<15.3e} {:.3}",
                if colour { "colour" } else { "shear" },
                e / cu * 1e9
            );
        }
    }

    println!("\nB. THE KINETIC CLOCK, measured on FHP-I at d={D}");
    let lat = Lattice::seeded(m.clone(), 128, SEED, D, law.to_vec());
    let mut cells = lat.cells.clone();
    let mut out = vec![0u8; 128 * 128];
    let mut fired = 0u64;
    for t in 0..400u64 {
        fired += lat.advance(&mut cells, &mut out, t).collisions_fired;
        core::mem::swap(&mut cells, &mut out);
    }
    let cn = (128 * 128) as f64;
    println!("   collisions fired per cell-step: {:.6}", fired as f64 / (400.0 * cn));
    println!("   particles per cell:             {:.6}", lat.ledger().mass as f64 / cn);

    println!("\nC. nu(k) AND D(k) ON FHP-I — the curve the 10 % two-wavevector test is read on");
    println!("   window: open at 4 tau = {t0_rule} steps, close at a fifth of the amplitude there");
    println!(
        "   L     k_i  k          seeds  steps   window        gamma        R2        value      \
         amp0      amp1"
    );
    let mut nu_curve: Vec<(usize, usize, f64)> = Vec::new();
    let mut d_curve: Vec<(usize, usize, f64)> = Vec::new();
    for (l, ki, s, colour) in [
        (64usize, 1usize, 64usize, false),
        (64, 2, 64, false),
        (128, 1, 32, false),
        (128, 2, 32, false),
        (128, 3, 32, false),
        (128, 4, 32, false),
        (256, 1, 16, false),
        (256, 2, 16, false),
        (64, 1, 64, true),
        (128, 1, 32, true),
        (128, 2, 32, true),
        (256, 1, 16, true),
        (256, 2, 16, true),
    ] {
        let k = wavenumber(l, ki);
        let coeff = if colour { D_GUESS } else { NU_GUESS };
        let steps = steps_for(t0_rule, coeff, k, 2.5);
        let e = mean_of(&traces(m, law, l, if colour { 1.0 } else { A_SHEAR }, ki, steps, s, colour, threads), s);
        let (w0, w1, g, r2, a0, a1) = window_fit(&e, t0_rule, 0.2);
        let v = g / (k * k);
        println!(
            "   {l:<5} {ki:<4} {k:<10.6} {s:<6} {steps:<7} {w0:>4}..{w1:<7} {g:<12.6e} {r2:<9.5} \
             {v:<10.5} {a0:<9.2} {a1:<9.2}{}",
            if colour { "  [colour, A=1.0]" } else { "" }
        );
        if colour {
            d_curve.push((l, ki, v));
        } else {
            nu_curve.push((l, ki, v));
        }
    }
    println!("\n   the 10 % two-wavevector test, per lattice size:");
    for l in [64usize, 128, 256] {
        for (name, c) in [("nu", &nu_curve), ("D", &d_curve)] {
            let g = |ki: usize| c.iter().find(|(a, b, _)| *a == l && *b == ki).map(|(_, _, v)| *v);
            if let (Some(v1), Some(v2)) = (g(1), g(2)) {
                let rel = (v1 - v2).abs() / v1.abs().max(v2.abs());
                println!(
                    "     L={l:<4} {name:<3} k1 {v1:<10.5} k2 {v2:<10.5} relative {rel:.4}  \
                     within 10 % = {}",
                    rel <= 0.10
                );
            }
        }
    }
}

fn pass2(m: &Model, law: &[u8], threads: usize, t0_rule: usize) {
    // -------------------------------------------------------------------- F. the noise floor
    println!("\nF. EQUILIBRIUM NOISE, measured ACROSS SEEDS (not across time: the mode's own");
    println!("   correlation time is longer than any window, so a time RMS is one sample)");
    println!("   L     readout  sd across 64 seeds at step 100   signal M(0)   SNR(1 seed)");
    let mut noise: Vec<(usize, bool, f64)> = Vec::new();
    for l in [64usize, 128, 256] {
        for colour in [false, true] {
            let parts = traces(m, law, l, 0.0, 1, 120, 64, colour, threads);
            let v: Vec<f64> = parts.iter().map(|p| p[100]).collect();
            let mean = v.iter().sum::<f64>() / 64.0;
            let sd = (v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / 63.0).sqrt();
            let sig = if colour {
                colour_series(m, law, l, D, 1.0, 1, 1, SEED, ColourRule::Blind, false).0[0].abs()
            } else {
                shear_series(m, law, l, D, A_SHEAR, 1, 1, SEED, false, row_drive(), false).0[0].abs()
            };
            println!(
                "   {l:<5} {:<8} {sd:<31.4} {sig:<13.2} {:.3}",
                if colour { "colour" } else { "shear" },
                sig / sd
            );
            noise.push((l, colour, sd));
        }
    }

    // -------------------------------------------------------------------- G. colour is passive
    println!("\nG. THE COLOUR AMPLITUDE IS FREE — colour never enters the occupation dynamics,");
    println!("   so a colour wave of ANY amplitude is still linear response. L=128, k_index=1.");
    println!("   A_col   amp0        D            R2");
    {
        let (l, ki, s) = (128usize, 1usize, 32usize);
        let k = wavenumber(l, ki);
        let steps = steps_for(t0_rule, D_GUESS, k, 2.5);
        for a in [0.05f64, 0.25, 1.0] {
            let e = mean_of(&traces(m, law, l, a, ki, steps, s, true, threads), s);
            let (_, _, g, r2, a0, _) = window_fit(&e, t0_rule, 0.2);
            println!("   {a:<7} {a0:<11.3} {:<12.5} {r2:.5}", g / (k * k));
        }
    }

    // -------------------------------------------------------------------- H. the transient rule
    println!("\nH. THE WINDOW-START RULE — does the value stop moving once the transient is past?");
    println!("   L=256, 16 seeds; the start swept in units of tau = {TAU:.1} steps");
    println!("   readout  k_i  t0     t0/tau  window        value        R2");
    for (colour, ki) in [(false, 1usize), (false, 2), (true, 1), (true, 2)] {
        let l = 256usize;
        let k = wavenumber(l, ki);
        let coeff = if colour { D_GUESS } else { NU_GUESS };
        let steps = steps_for(200, coeff, k, 2.5);
        let parts = traces(m, law, l, if colour { 1.0 } else { A_SHEAR }, ki, steps, 16, colour, threads);
        let e = mean_of(&parts, 16);
        for mult in [1.0f64, 2.0, 4.0, 8.0, 16.0] {
            let t0 = (mult * TAU).ceil() as usize;
            if t0 + 30 >= e.len() {
                continue;
            }
            let (w0, w1, g, r2, _, _) = window_fit(&e, t0, 0.2);
            println!(
                "   {:<8} {ki:<4} {t0:<6} {mult:<7.0} {w0:>4}..{w1:<7} {:<12.5} {r2:.5}",
                if colour { "colour" } else { "shear" },
                g / (k * k)
            );
        }
    }

    // -------------------------------------------------------------------- I. the seed floor
    println!("\nI. THE SEED FLOOR AT L=256 — how many seeds the R2 >= 0.99 floor actually needs");
    println!("   window: open at 4 tau = {t0_rule}, close at a fifth of the amplitude there");
    println!("   readout  A      k_i  seeds  window        value        R2        SNR at close");
    for (colour, a, ki) in [
        (false, 0.05f64, 1usize),
        (false, 0.05, 2),
        (false, 0.10, 1),
        (false, 0.10, 2),
        (true, 1.0, 1),
        (true, 1.0, 2),
    ] {
        let l = 256usize;
        let k = wavenumber(l, ki);
        let coeff = if colour { D_GUESS } else { NU_GUESS };
        let steps = steps_for(t0_rule, coeff, k, 2.5);
        let parts = traces(m, law, l, a, ki, steps, 16, colour, threads);
        let sd = noise.iter().find(|(ll, c, _)| *ll == l && *c == colour).map(|(_, _, s)| *s).unwrap();
        for s in [1usize, 2, 4, 8, 16] {
            let e = mean_of(&parts, s);
            let (w0, w1, g, r2, _, a1) = window_fit(&e, t0_rule, 0.2);
            println!(
                "   {:<8} {a:<6} {ki:<4} {s:<6} {w0:>4}..{w1:<7} {:<12.5} {r2:<9.5} {:.2}",
                if colour { "colour" } else { "shear" },
                g / (k * k),
                a1 / (sd / (s as f64).sqrt())
            );
        }
    }
}
