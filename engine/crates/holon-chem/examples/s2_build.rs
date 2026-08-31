//! Build the fine (O, H, H) node set ONCE, then read every candidate table out of it.
//!
//! # Why the sizing sweep and the production build are the same run
//!
//! `examples/s2_grid.rs` chose the stretch by building one fine table per candidate and
//! reading coarser subgrids inside it: node `i` of the 33-grid is node `2i` of the
//! 65-grid, so one electronic-structure pass serves a whole column. The shipped table is
//! itself one of those subgrids. So there is no reason to pay for the sizing and then pay
//! again for the build — this run does the fine pass, reports the held-out error of every
//! subgrid WITH the geometry that carried it, and writes the fine node set to a cache the
//! emit step reads. Choosing a size afterwards costs nothing.
//!
//! At about 53 ms per node this is tens of minutes of arithmetic, so it is meant to be
//! run detached with a done-marker. `--emit NR NU` re-reads the cache and writes the
//! committed artifact for that subgrid without solving anything.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_build -- [threads]
//! cargo run --release -p holon-chem --example s2_build -- --emit 65 33 [out_path]
//! ```

use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point};
use holon_chem::trimer::cr_weights;
use holon_chem::water::{self, de3_with};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// The fine node set. `NR_FINE - 1` and `NU_FINE - 1` are chosen so the candidate sizes
/// divide them exactly and every candidate is a genuine SUBSET rather than a
/// re-interpolation.
const NR_FINE: usize = 65;
const NU_FINE: usize = 49;
const NR_TRY: [usize; 4] = [65, 33, 17, 9];
const NU_TRY: [usize; 4] = [49, 25, 13, 9];

/// The stretch `examples/s2_grid.rs` measured: at 49 x 49 x 25 the held-out maximum is
/// 7.84e-4 Ha at `a = 2`, 6.72e-4 at `a = 3`, 8.00e-4 at `a = 4` — a shallow minimum at 3.
const A: f64 = 3.0;

const R_LO: f64 = water::R_LO;
const R_HI: f64 = water::R_HI;
const C_LO: f64 = water::C_LO;
const C_HI: f64 = water::C_HI;

/// The staked draw for the held-out set. "SATURAT2" in ASCII.
const SEED: u64 = 0x5341_5455_5241_5432;
const N_HELD: usize = 384;

fn cache_path() -> String {
    format!(
        "{}/../../../conformance/atomworld/s2_runs/s2_fine_{NR_FINE}x{NU_FINE}.txt",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

fn r_of_tau(tau: f64) -> f64 {
    R_LO + (R_HI - R_LO) * ((A * tau).exp() - 1.0) / (A.exp() - 1.0)
}

fn tau_of_r(r: f64) -> f64 {
    (1.0 + (r - R_LO) * (A.exp() - 1.0) / (R_HI - R_LO)).ln() / A
}

fn fine_index(i: usize, j: usize, k: usize) -> usize {
    (i * NR_FINE + j) * NU_FINE + k
}

/// Catmull-Rom read of the `(nr, nu)` subgrid of the fine node set.
fn eval(fine: &[f64], nr: usize, nu: usize, x: f64, y: f64, c: f64) -> f64 {
    let sr = (NR_FINE - 1) / (nr - 1);
    let su = (NU_FINE - 1) / (nu - 1);
    let tx = (tau_of_r(x) * (nr - 1) as f64).clamp(0.0, (nr - 1) as f64);
    let ty = (tau_of_r(y) * (nr - 1) as f64).clamp(0.0, (nr - 1) as f64);
    let tc = ((c - C_LO) / (C_HI - C_LO) * (nu - 1) as f64).clamp(0.0, (nu - 1) as f64);
    let (bx, wx, _) = cr_weights(nr, tx);
    let (by, wy, _) = cr_weights(nr, ty);
    let (bc, wc, _) = cr_weights(nu, tc);
    let mut acc = 0.0;
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                acc += wx[i]
                    * wy[j]
                    * wc[k]
                    * fine[fine_index((bx + i) * sr, (by + j) * sr, (bc + k) * su)];
            }
        }
    }
    acc
}

fn held_out() -> Vec<(f64, f64, f64)> {
    let mut st = SEED;
    let mut held = Vec::with_capacity(N_HELD);
    while held.len() < N_HELD {
        let (t1, t2, t3) = (lcg(&mut st), lcg(&mut st), lcg(&mut st));
        // "None on nodes", and on no CANDIDATE grid's nodes either: a draw that could
        // land on a node would be measuring the coarse grids where they are exact.
        let near = |t: f64, n: usize| {
            let f = t * (n - 1) as f64;
            (f - f.round()).abs() < 0.05
        };
        if NR_TRY.iter().any(|&n| near(t1, n) || near(t2, n)) || NU_TRY.iter().any(|&n| near(t3, n))
        {
            continue;
        }
        let (x, y) = (r_of_tau(t1), r_of_tau(t2));
        let (x, y) = if x <= y { (x, y) } else { (y, x) };
        held.push((x, y, C_LO + (C_HI - C_LO) * t3));
    }
    held
}

fn read_cache() -> Option<Vec<f64>> {
    let text = std::fs::read_to_string(cache_path()).ok()?;
    let mut v = Vec::with_capacity(NR_FINE * NR_FINE * NU_FINE);
    for line in text.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        v.push(f64::from_bits(u64::from_str_radix(line, 16).ok()?));
    }
    if v.len() == NR_FINE * NR_FINE * NU_FINE {
        Some(v)
    } else {
        None
    }
}


/// The interpolation variables the grid is uniform in: `eval` walks `tau` and a
/// normalised `c`, so a gauge function must be smooth in THOSE and not in `(x, y, c)`,
/// or the stretch is being graded along with the interpolant.
fn planted(name: &str, tx: f64, ty: f64, ch: f64) -> f64 {
    match name {
        // Smooth, non-polynomial, and of the surface's own scale (peak |dE3| is 0.76 Ha).
        // Nothing here can be reproduced exactly by a cubic, so the measured rate is the
        // scheme's and not an artifact of the test function being too easy.
        "exp-cos" => 0.75 * (-2.0 * tx).exp() * (-2.0 * ty).exp() * (std::f64::consts::PI * ch).cos(),
        // A bump with real curvature ACROSS the angle axis, which is the axis under
        // investigation: if the shortfall were the c direction specifically, this is the
        // function that would show it on a surface with no physics in it at all.
        "c-bump" => {
            let g = (-((ch - 0.5) / 0.15).powi(2)).exp();
            0.75 * g * (1.0 + 0.3 * (3.0 * tx).sin() * (2.0 * ty).cos())
        }
        _ => unreachable!(),
    }
}

/// THE GAUGE: what rate does this interpolator, on THIS grid family, deliver on a function
/// with no physics in it?
///
/// # Why this exists
///
/// SATURATION-2 published that the shipped table's angle axis "converges at 5x per
/// doubling where a C1 cubic should give 16x", and SATURATION-3's freeze makes explaining
/// that a design-time obligation. 16x is `h^4`, and it was asserted rather than measured.
/// Catmull-Rom takes its node slopes from centred differences, whose `O(h^2)` error enters
/// the Hermite form multiplied by one factor of `h` — so the scheme is third order, `8x`,
/// and Keys' cubic convolution result for `a = -1/2` says the same. A gauge settles it
/// without an argument: plant a function whose truth is analytic, interpolate it with the
/// SAME `eval` on the SAME subgrids at the SAME held-out draw, and read the rate off.
///
/// It also separates the two candidate causes of whatever shortfall remains. The
/// tableau's worst held-out point sits at `c = 1.373` (the last c interval on every grid
/// but the finest) or at `c = 0.065` (the first), and `slope_weights` uses a ONE-SIDED
/// three-point slope at both ends. So the columns below report the max over ALL held-out
/// points and the max over points in no END interval, and the difference between those
/// two rates is the end condition's contribution.
///
/// Costs no electronic-structure solves for the planted rows.
fn gauge(threads: usize, e_o: f64, e_h: f64) {
    let held = held_out();
    println!("# GAUGE: the interpolator's own rate, on functions with no physics in them");
    println!("# same eval(), same subgrids, same {N_HELD}-point held-out draw as the tableau");
    println!("# 'interior' excludes any point in the FIRST or LAST interval of the c axis,");
    println!("# which is where slope_weights uses a one-sided three-point slope.\n");

    for name in ["exp-cos", "c-bump"] {
        let mut fine = vec![0.0f64; NR_FINE * NR_FINE * NU_FINE];
        for i in 0..NR_FINE {
            let tx = i as f64 / (NR_FINE - 1) as f64;
            for j in 0..NR_FINE {
                let ty = j as f64 / (NR_FINE - 1) as f64;
                for k in 0..NU_FINE {
                    let ch = k as f64 / (NU_FINE - 1) as f64;
                    fine[fine_index(i, j, k)] = planted(name, tx, ty, ch);
                }
            }
        }
        let truth: Vec<f64> = held
            .iter()
            .map(|&(x, y, c)| {
                planted(name, tau_of_r(x), tau_of_r(y), (c - C_LO) / (C_HI - C_LO))
            })
            .collect();
        println!("## planted `{name}`");
        tableau(&fine, &held, &truth);
    }

    match read_cache() {
        Some(fine) => {
            let truth: Vec<Mutex<f64>> = held.iter().map(|_| Mutex::new(0.0)).collect();
            let next = AtomicUsize::new(0);
            std::thread::scope(|sc| {
                for _ in 0..threads {
                    sc.spawn(|| loop {
                        let t = next.fetch_add(1, Ordering::SeqCst);
                        if t >= held.len() {
                            break;
                        }
                        let (x, y, c) = held[t];
                        *truth[t].lock().unwrap() = de3_with(
                            x,
                            y,
                            1.0 - c * c,
                            e_o,
                            e_h,
                            pair_point(OXYGEN, HYDROGEN, x).e,
                            pair_point(OXYGEN, HYDROGEN, y).e,
                        );
                    });
                }
            });
            let truth: Vec<f64> = truth.into_iter().map(|m| m.into_inner().unwrap()).collect();
            println!("## the real (O, H, H) surface, same columns");
            tableau(&fine, &held, &truth);
        }
        None => println!("## the real surface: SKIPPED, no fine node set cached"),
    }
}

/// One planted or real surface's held-out tableau, with the end-interval split.
fn tableau(fine: &[f64], held: &[(f64, f64, f64)], truth: &[f64]) {
    // Three masks, because they answer three different questions. ALL is the published
    // number. INTERIOR drops points sitting IN an end interval, which is where the
    // one-sided slope lives. CLEAR drops every point whose four-wide Catmull-Rom stencil
    // REACHES the last node — a corner inside the final cell corrupts the interval below
    // it too, and only this mask separates "the grid is too coarse" from "the stencil is
    // reading across something it cannot represent".
    println!(
        "   {:>4} {:>4} {:>12} {:>7} {:>12} {:>7} {:>12} {:>7} {:>6}",
        "nr", "nu", "max all", "rate", "max interior", "rate", "max clear", "rate", "n_end"
    );
    for nr in NR_TRY {
        let mut prev_all: Option<f64> = None;
        let mut prev_int: Option<f64> = None;
        let mut prev_clear: Option<f64> = None;
        // Coarse to fine, so a "rate" is the improvement from the row above.
        let mut nus = NU_TRY;
        nus.reverse();
        for nu in nus {
            let (mut all, mut interior, mut clear, mut n_end) = (0.0f64, 0.0f64, 0.0f64, 0usize);
            let mut clear_at = 0usize;
            for (t, &(x, y, c)) in held.iter().enumerate() {
                let e = (eval(fine, nr, nu, x, y, c) - truth[t]).abs();
                all = all.max(e);
                let tc = (c - C_LO) / (C_HI - C_LO) * (nu - 1) as f64;
                if tc < 1.0 || tc > (nu - 2) as f64 {
                    n_end += 1;
                } else {
                    interior = interior.max(e);
                }
                if tc >= 1.0 && tc <= (nu - 3) as f64 && e > clear {
                    clear = e;
                    clear_at = t;
                }
            }
            let ra = prev_all.map(|p| p / all);
            let ri = prev_int.map(|p| p / interior);
            let rc = prev_clear.map(|p| p / clear);
            println!(
                "   {nr:>4} {nu:>4} {all:>12.4e} {:>7} {interior:>12.4e} {:>7} {clear:>12.4e} {:>7} {n_end:>6}",
                ra.map(|v| format!("{v:.2}x")).unwrap_or_else(|| "-".into()),
                ri.map(|v| format!("{v:.2}x")).unwrap_or_else(|| "-".into()),
                rc.map(|v| format!("{v:.2}x")).unwrap_or_else(|| "-".into()),
            );
            let (cx, cy, cc) = held[clear_at];
            println!("        clear worst at ({cx:.3}, {cy:.3}, {cc:.3})");
            prev_all = Some(all);
            prev_int = Some(interior);
            prev_clear = Some(clear);
        }
        println!();
    }
    println!(
        "   For reference: h^2 is 4x per doubling of (nu-1), h^3 is 8x, h^4 is 16x. The\n   \
         (nu-1) ladder here is 8, 12, 24, 48, so 9->17 is not a doubling and only the\n   \
         13->25 and 25->49 rungs are.\n"
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);

    if args.first().map(String::as_str) == Some("--gauge") {
        let threads: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(6);
        gauge(threads, e_o, e_h);
        return;
    }

    if args.first().map(String::as_str) == Some("--emit") {
        let nr: usize = args[1].parse().expect("NR");
        let nu: usize = args[2].parse().expect("NU");
        assert_eq!(nr, water::NR, "--emit NR must equal water::NR");
        assert_eq!(nu, water::NU, "--emit NU must equal water::NU");
        assert_eq!(A, water::STRETCH_A, "the cache's stretch is not water::STRETCH_A");
        let fine = read_cache().expect("the fine node set is cached");
        let sr = (NR_FINE - 1) / (nr - 1);
        let su = (NU_FINE - 1) / (nu - 1);
        let mut t = water::WaterTable::empty();
        t.begin();
        let mut peak = 0.0f64;
        for i in 0..nr {
            for j in 0..nr {
                for k in 0..nu {
                    let v = fine[fine_index(i * sr, j * sr, k * su)];
                    peak = peak.max(v.abs());
                    assert!(t.knot(water::node_index(i, j, k), v));
                }
            }
        }
        let meta = water::WaterMeta {
            e_o_atom: e_o,
            e_h_atom: e_h,
            peak,
            solves: NR_FINE * (NR_FINE + 1) / 2 * NU_FINE,
            ..water::WaterMeta::empty()
        };
        assert!(t.finish(meta), "the emitted table did not close");
        let out = args
            .get(3)
            .cloned()
            .unwrap_or_else(|| format!("{}/tests/data/s2/s2_water_table.txt", env!("CARGO_MANIFEST_DIR")));
        let text = water::to_text(&t);
        std::fs::write(&out, &text).unwrap_or_else(|e| panic!("cannot write {out}: {e}"));
        println!(
            "wrote {out} ({} bytes)\n  peak |dE3|             = {peak:.6e} Ha\n  \
             curvature_envelope     = {:.6e} Ha/bohr^2\n  curvature_per_gradient = {:.6e} /bohr\n  \
             sort_kink              = {:.3e} Ha/bohr",
            text.len(),
            t.curvature_envelope,
            t.curvature_per_gradient,
            t.sort_kink
        );
        return;
    }

    let threads: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(6);
    let t0 = Instant::now();
    println!("# fine {NR_FINE} x {NR_FINE} x {NU_FINE}, stretch a = {A}");
    println!("# box x, y in [{R_LO}, {R_HI}], c in [{C_LO}, {C_HI:.6}]");
    println!("# E(O) = {e_o:.17}  E(H) = {e_h:.17}  threads = {threads}");

    let fine = match read_cache() {
        Some(v) => {
            println!("# fine node set read from cache, no solves paid");
            v
        }
        None => {
            let rs: Vec<f64> = (0..NR_FINE)
                .map(|i| r_of_tau(i as f64 / (NR_FINE - 1) as f64))
                .collect();
            let e_oh: Vec<f64> = rs
                .iter()
                .map(|&r| pair_point(OXYGEN, HYDROGEN, r).e)
                .collect();
            let vals: Vec<Mutex<f64>> = (0..NR_FINE * NR_FINE * NU_FINE)
                .map(|_| Mutex::new(0.0))
                .collect();
            let next = AtomicUsize::new(0);
            let done = AtomicUsize::new(0);
            std::thread::scope(|sc| {
                for _ in 0..threads {
                    sc.spawn(|| loop {
                        let i = next.fetch_add(1, Ordering::SeqCst);
                        if i >= NR_FINE {
                            break;
                        }
                        for j in i..NR_FINE {
                            for k in 0..NU_FINE {
                                let c = C_LO + (C_HI - C_LO) * k as f64 / (NU_FINE - 1) as f64;
                                let u = 1.0 - c * c;
                                let d = de3_with(rs[i], rs[j], u, e_o, e_h, e_oh[i], e_oh[j]);
                                *vals[fine_index(i, j, k)].lock().unwrap() = d;
                                *vals[fine_index(j, i, k)].lock().unwrap() = d;
                            }
                        }
                        let n = done.fetch_add(1, Ordering::SeqCst) + 1;
                        println!(
                            "  row {i:3} done ({n}/{NR_FINE}, {:.0} s)",
                            t0.elapsed().as_secs_f64()
                        );
                    });
                }
            });
            let v: Vec<f64> = vals.into_iter().map(|m| m.into_inner().unwrap()).collect();
            let mut text = String::with_capacity(v.len() * 17 + 256);
            text.push_str(&format!(
                "# fine (O,H,H) node set: NR={NR_FINE} NU={NU_FINE} A={A} R_LO={R_LO} \
                 R_HI={R_HI} C_LO={C_LO} C_HI={C_HI}\n"
            ));
            text.push_str(&format!("# E_O={:016x} E_H={:016x}\n", e_o.to_bits(), e_h.to_bits()));
            for x in &v {
                text.push_str(&format!("{:016x}\n", x.to_bits()));
            }
            std::fs::write(cache_path(), &text).expect("cache written");
            println!("# fine node set built in {:.0} s", t0.elapsed().as_secs_f64());
            v
        }
    };

    let peak = fine.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    println!("# peak |dE3| on a node = {peak:.6e} Ha\n");

    // The truth at the held-out draw, computed fresh — never read off the fine grid,
    // which would be grading the interpolant against itself.
    let held = held_out();
    let truth: Vec<Mutex<f64>> = held.iter().map(|_| Mutex::new(0.0)).collect();
    let next = AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let t = next.fetch_add(1, Ordering::SeqCst);
                if t >= held.len() {
                    break;
                }
                let (x, y, c) = held[t];
                *truth[t].lock().unwrap() = de3_with(
                    x,
                    y,
                    1.0 - c * c,
                    e_o,
                    e_h,
                    pair_point(OXYGEN, HYDROGEN, x).e,
                    pair_point(OXYGEN, HYDROGEN, y).e,
                );
            });
        }
    });
    let truth: Vec<f64> = truth.into_iter().map(|m| m.into_inner().unwrap()).collect();

    println!(
        "{:>4} {:>4} {:>12} {:>12} {:>10} {:>34} {:>10}",
        "nr", "nu", "max |dV|", "rms |dV|", "solves", "worst at (x, y, c)", "true dE3"
    );
    for nr in NR_TRY {
        for nu in NU_TRY {
            let (mut worst, mut at, mut sq) = (0.0f64, 0usize, 0.0f64);
            for (t, &(x, y, c)) in held.iter().enumerate() {
                let e = (eval(&fine, nr, nu, x, y, c) - truth[t]).abs();
                sq += e * e;
                if e > worst {
                    worst = e;
                    at = t;
                }
            }
            let (x, y, c) = held[at];
            println!(
                "{nr:>4} {nu:>4} {worst:>12.3e} {:>12.3e} {:>10} {:>34} {:>10.3e}",
                (sq / held.len() as f64).sqrt(),
                nr * (nr + 1) / 2 * nu,
                format!("({x:.3}, {y:.3}, {c:.3})"),
                truth[at]
            );
        }
    }
    println!("\n# total {:.0} s", t0.elapsed().as_secs_f64());
}
