//! Which third coordinate the (O, H, H) table should use: the cosine `u` itself, or
//! `c = sqrt(1 - u)`.
//!
//! # Why this is not settled by copying SATURATION-1
//!
//! `trimer.rs` uses `c` and records the measurement that chose it: at `x = y` the third
//! side is `z = x sqrt(2) c` exactly, so a uniform `c` grid is a uniform `z` grid there,
//! and `c` beat raw `u` by 5x on held-out error. That argument transfers. What does NOT
//! transfer is the domain: a SORTED hydrogen triple has `u <= 1/2`, so `c >= 0.707` and
//! H3's grid never goes near `c = 0`. Here the two axes are the two O-H sides and the
//! angle is measured at oxygen, so `u = 1` — both hydrogens on one ray from the oxygen —
//! is inside the domain, and it is not an exotic corner: it is EXACTLY the geometry of a
//! hydrogen molecule approaching an oxygen atom head-on, which is the reaction this
//! campaign exists to watch.
//!
//! At `u = 1` the map is singular in `c`. `dE3` is analytic in `u`, so its `c`-derivative
//! must vanish proportionally to `c` there, and the chain rule to the sides needs
//! `dF/du = -F_c / (2c)` — a `0/0` at the collinear point, and a `1/c` amplification of
//! whatever interpolation error stands nearby. In `u` there is no such factor at all.
//!
//! So the choice is a trade — `c`'s resolution against `u`'s regularity — and it is
//! decided here on BOTH numbers that matter: the held-out error of the VALUE, and the
//! held-out error of `dF/du`, which is what the sandbox's force loop actually reads.
//!
//! # The instrument
//!
//! The third coordinate is separable in a tensor-product interpolant, so it is measured
//! one-dimensionally at fixed `(x, y)`: exact node values at each candidate grid, exact
//! held-out values and exact `dE3/du` at a staked draw, and the same Catmull-Rom weights
//! the shipped table uses. Exact means exact — `dE3/du` comes from the dual-number route
//! carrying `u` as its variable, never from a finite difference of the thing under test.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_third [threads]
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};
use holon_chem::trimer::cr_weights;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

const C_LO: f64 = 0.05;
const C_HI: f64 = std::f64::consts::SQRT_2;
/// The `u` the fence stands for: `u = 1 - C_LO^2`, the closed end.
const U_HI: f64 = 1.0 - C_LO * C_LO;
const U_LO: f64 = -1.0;

/// Node counts under test. Each divides `NQ_FINE - 1`, so every coarse grid is a SUBSET
/// of the fine one and one exact pass serves the whole column.
const NQ_FINE: usize = 97;
const NQ_TRY: [usize; 5] = [97, 49, 25, 13, 9];

const SEED: u64 = 0x5341_5455_5241_5433;
const N_HELD: usize = 192;

/// How far the derivative draw stays off the collinear end `u = -1`. See `de3_du`: the
/// instrument's dual number cannot form `0 * infinity` there. Stated as a fence rather
/// than hidden, and it costs the derivative column the last 0.1% of the `u` range.
const U_FENCE: f64 = 2.0e-3;

/// The `(x, y)` pairs the third coordinate is measured at. Staked as a spanning set of
/// the side box — the compact diagonal, an asymmetric compact pair, the near-equilibrium
/// scale, and a stretched pair — not chosen from any result.
const XY: [(f64, f64); 5] = [
    (0.9, 0.9),
    (0.9, 2.0),
    (1.8, 1.8),
    (1.8, 3.5),
    (4.0, 7.0),
];

fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

/// `dE3` and, when `want_d`, its EXACT `d/du` at fixed O-H sides `(x, y)`.
///
/// Both the triple's energy and the H-H pair term move with `u`: the pair term through
/// `z(u)`, whose derivative is supplied by the pair route's own `dE/dr`. The two O-H pair
/// terms and the three atom energies are constants here and drop out of the derivative.
///
/// # Why the derivative is optional, and where it does not exist
///
/// The second hydrogen is placed at `(y u, y sqrt(1 - u^2))`, so its PERPENDICULAR
/// coordinate has an infinite `u`-derivative at `u = -1`, the collinear H-O-H end. The
/// surface itself is perfectly smooth there — the energy depends on that perpendicular
/// displacement quadratically, by symmetry, so `dE3/du` is finite — but the product is
/// `0 * infinity` and a dual number cannot form it. That is a property of THIS
/// INSTRUMENT and not of the table: the table's own chain rule runs `du/dz = -z/(x y)`,
/// which at `u = -1` is `-(x + y)/(x y)` and entirely finite.
///
/// So node VALUES are taken value-only (the node set includes `u = -1`), and the
/// derivative column is measured on a held-out draw fenced away from that end.
fn de3_du(
    x: f64,
    y: f64,
    u: f64,
    e_o: f64,
    e_h: f64,
    e_ox: f64,
    e_oy: f64,
    want_d: bool,
) -> (f64, f64) {
    // At `u = -1` exactly, `1 - u^2` is an exact zero and `D2::sqrt` forms `0/0` in the
    // derivative slots — a NaN even when the INPUT derivative is zero, which then
    // poisons the geometry and comes back as "the overlap matrix is not positive
    // definite". The value-only path therefore takes the sine in plain f64, where
    // `sqrt(0)` is 0 and nothing is divided by it.
    let (ud, sn) = if want_d {
        let ud = D2::var(u);
        (ud, (1.0 - ud * ud).sqrt())
    } else {
        (D2::c(u), D2::c((1.0 - u * u).max(0.0).sqrt()))
    };
    let sol = solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(x), D2::c(0.0), D2::c(0.0)],
            [ud * y, sn * y, D2::c(0.0)],
        ],
    )
    .e;
    let z2 = x * x + y * y - 2.0 * x * y * u;
    let z = z2.max(0.0).sqrt();
    let hh = pair_point(HYDROGEN, HYDROGEN, z);
    let d = sol.v + e_o + 2.0 * e_h - e_ox - e_oy - hh.e;
    if !want_d {
        return (d, 0.0);
    }
    // dz/du = -x y / z; and `hh.f` is the FORCE, -dE/dr, so dE_HH/dz = -hh.f.
    let dd = sol.d - (-hh.f) * (-x * y / z);
    (d, dd)
}

/// Catmull-Rom read of a coarse subgrid, returning the value and `dF/dq` where `q` is the
/// grid's own coordinate.
fn read(fine: &[f64], nq: usize, q: f64, q_lo: f64, q_hi: f64) -> (f64, f64) {
    let step = (NQ_FINE - 1) / (nq - 1);
    let t = ((q - q_lo) / (q_hi - q_lo) * (nq - 1) as f64).clamp(0.0, (nq - 1) as f64);
    let (b, w, dw) = cr_weights(nq, t);
    let mut v = 0.0;
    let mut dv = 0.0;
    for i in 0..4 {
        let f = fine[(b + i) * step];
        v += w[i] * f;
        dv += dw[i] * f;
    }
    // Index-space derivative to `q`-space.
    (v, dv * (nq - 1) as f64 / (q_hi - q_lo))
}

fn main() {
    let threads: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(6);
    let t0 = Instant::now();
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    println!("# E(O) = {e_o:.12}  E(H) = {e_h:.12}  threads = {threads}");
    println!("# third coordinate on [{U_LO}, {U_HI}] in u, [{C_LO}, {C_HI:.6}] in c");
    println!("# fine {NQ_FINE} nodes per candidate, {N_HELD} held-out per (x, y)\n");

    // The held-out draw, in u, once, shared by both coordinates and every node count so
    // the columns differ only in the grid.
    let mut st = SEED;
    let mut held: Vec<f64> = Vec::with_capacity(N_HELD);
    while held.len() < N_HELD {
        let t = lcg(&mut st);
        let u = (U_LO + U_FENCE) + (U_HI - U_LO - U_FENCE) * t;
        let c = (1.0 - u).sqrt();
        let near = |q: f64, lo: f64, hi: f64, n: usize| {
            let f = (q - lo) / (hi - lo) * (n - 1) as f64;
            (f - f.round()).abs() < 0.05
        };
        if NQ_TRY
            .iter()
            .any(|&n| near(u, U_LO, U_HI, n) || near(c, C_LO, C_HI, n))
        {
            continue;
        }
        held.push(u);
    }

    println!(
        "{:>10} {:>5} {:>4} {:>12} {:>12} {:>12} {:>12}",
        "(x, y)", "coord", "nq", "max |dV|", "rms |dV|", "max |dF/du|", "rms |dF/du|"
    );
    for (x, y) in XY {
        let e_ox = pair_point(OXYGEN, HYDROGEN, x).e;
        let e_oy = pair_point(OXYGEN, HYDROGEN, y).e;

        // Exact node values on both candidate grids, and the exact held-out truth.
        let mut jobs: Vec<(usize, f64)> = Vec::new();
        for i in 0..NQ_FINE {
            let t = i as f64 / (NQ_FINE - 1) as f64;
            jobs.push((0, U_LO + (U_HI - U_LO) * t)); // u-grid node
            let c = C_LO + (C_HI - C_LO) * t;
            jobs.push((1, 1.0 - c * c)); // c-grid node
        }
        for &u in &held {
            jobs.push((2, u));
        }
        let out: Vec<Mutex<(f64, f64)>> = jobs.iter().map(|_| Mutex::new((0.0, 0.0))).collect();
        let next = AtomicUsize::new(0);
        std::thread::scope(|sc| {
            for _ in 0..threads {
                sc.spawn(|| loop {
                    let t = next.fetch_add(1, Ordering::SeqCst);
                    if t >= jobs.len() {
                        break;
                    }
                    let want_d = jobs[t].0 == 2;
                    *out[t].lock().unwrap() =
                        de3_du(x, y, jobs[t].1, e_o, e_h, e_ox, e_oy, want_d);
                });
            }
        });
        let out: Vec<(f64, f64)> = out.into_iter().map(|m| m.into_inner().unwrap()).collect();
        let u_nodes: Vec<f64> = (0..NQ_FINE).map(|i| out[2 * i].0).collect();
        let c_nodes: Vec<f64> = (0..NQ_FINE).map(|i| out[2 * i + 1].0).collect();
        let truth: Vec<(f64, f64)> = out[2 * NQ_FINE..].to_vec();

        for (name, nodes, lo, hi) in [
            ("u", &u_nodes, U_LO, U_HI),
            ("c", &c_nodes, C_LO, C_HI),
        ] {
            for nq in NQ_TRY {
                let (mut mv, mut md) = (0.0f64, 0.0f64);
                let (mut sv, mut sd) = (0.0f64, 0.0f64);
                for (t, &u) in held.iter().enumerate() {
                    let (q, dq_du) = if name == "u" {
                        (u, 1.0)
                    } else {
                        let c = (1.0 - u).sqrt();
                        (c, -0.5 / c)
                    };
                    let (v, dv_dq) = read(nodes, nq, q, lo, hi);
                    let ev = (v - truth[t].0).abs();
                    let ed = (dv_dq * dq_du - truth[t].1).abs();
                    mv = mv.max(ev);
                    md = md.max(ed);
                    sv += ev * ev;
                    sd += ed * ed;
                }
                let n = held.len() as f64;
                println!(
                    "{:>10} {name:>5} {nq:>4} {mv:>12.3e} {:>12.3e} {md:>12.3e} {:>12.3e}",
                    format!("({x}, {y})"),
                    (sv / n).sqrt(),
                    (sd / n).sqrt()
                );
            }
        }
        println!();
    }
    println!("# total {:.1} s", t0.elapsed().as_secs_f64());
}
