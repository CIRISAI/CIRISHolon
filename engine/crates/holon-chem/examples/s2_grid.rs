//! Grid sizing for the SATURATION-2 (O, H, H) table: build the finest candidate once per
//! stretch, then read the held-out interpolation error of every coarser grid inside it.
//!
//! The coarse grids are SUBSETS of the fine one (node `i` of the 25-grid is node `2i` of
//! the 49-grid), so one electronic-structure pass serves a whole column of the sweep. The
//! interpolant is `trimer::cr_weights`, the SAME Catmull-Rom scheme the shipped table
//! uses, rather than a second implementation of it.
//!
//! # The coordinates under test
//!
//! `x = min(O-H)`, `y = max(O-H)`, `c = sqrt(1 - cos theta_HOH)`. Sorting the two O-H
//! sides is exact in floating point, so the H <-> H symmetry is bit-level; oxygen is
//! never sorted into either axis, which is what makes this table heteronuclear rather
//! than a relabelled H3. Every point of the box is a realisable triangle.
//!
//! `R_HI = 14` is `s2_domain`'s measured answer, not a taste: the worst `|dE3|` anywhere
//! on the shell `max(O-H) = b` is 3.2e-5 Ha at `b = 13` and 9.7e-6 at `b = 14`, so 14 is
//! the first integer shell inside the prereg's 1e-5 truncation stake. `C_LO = 0.05` is
//! the same file's second sweep: the closed-angle corner stays smooth and SATURATES down
//! to `c = 0.01` (two hydrogens 0.013 bohr apart) rather than diverging, because the
//! `1/z` nuclear repulsion cancels between `E(OHH)` and `E(HH)` by construction.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_grid [threads]
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};
use holon_chem::trimer::cr_weights;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

const R_LO: f64 = 0.7;
const R_HI: f64 = 14.0;
const C_LO: f64 = 0.05;
const C_HI: f64 = std::f64::consts::SQRT_2;

/// The finest candidate. Coarse grids are its subsets: `(NR_FINE - 1)` and
/// `(NU_FINE - 1)` are chosen so the interesting sizes divide them exactly.
const NR_FINE: usize = 49;
const NU_FINE: usize = 25;
const NR_TRY: [usize; 5] = [49, 25, 17, 13, 9];
const NU_TRY: [usize; 4] = [25, 13, 9, 7];

/// The staked draw for the held-out set. "SATURAT2" in ASCII.
const SEED: u64 = 0x5341_5455_5241_5432;
const N_HELD: usize = 384;

fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

fn r_of_tau(a: f64, tau: f64) -> f64 {
    if a == 0.0 {
        return R_LO + (R_HI - R_LO) * tau;
    }
    R_LO + (R_HI - R_LO) * ((a * tau).exp() - 1.0) / (a.exp() - 1.0)
}

fn tau_of_r(a: f64, r: f64) -> f64 {
    if a == 0.0 {
        return (r - R_LO) / (R_HI - R_LO);
    }
    (1.0 + (r - R_LO) * (a.exp() - 1.0) / (R_HI - R_LO)).ln() / a
}

fn c3(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

/// `dE3` at O-H sides `(x, y)` and `u = cos(theta_HOH)`, with the two O-H pair energies
/// supplied by the caller (they are cached over the node values).
fn de3(x: f64, y: f64, u: f64, e_o: f64, e_h: f64, e_ox: f64, e_oy: f64) -> f64 {
    let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
    let s = (1.0 - u * u).max(0.0).sqrt();
    let e3 = solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![c3(0.0, 0.0, 0.0), c3(x, 0.0, 0.0), c3(y * u, y * s, 0.0)],
    )
    .e
    .v;
    e3 + e_o + 2.0 * e_h - e_ox - e_oy - pair_point(HYDROGEN, HYDROGEN, z).e
}

/// Tri-cubic Catmull-Rom read of a coarse subgrid of the fine table.
#[allow(clippy::too_many_arguments)]
fn eval(
    fine: &[f64],
    nr: usize,
    nu: usize,
    a: f64,
    x: f64,
    y: f64,
    c: f64,
) -> f64 {
    let sr = (NR_FINE - 1) / (nr - 1);
    let su = (NU_FINE - 1) / (nu - 1);
    let tx = (tau_of_r(a, x) * (nr - 1) as f64).clamp(0.0, (nr - 1) as f64);
    let ty = (tau_of_r(a, y) * (nr - 1) as f64).clamp(0.0, (nr - 1) as f64);
    let tc = ((c - C_LO) / (C_HI - C_LO) * (nu - 1) as f64).clamp(0.0, (nu - 1) as f64);
    let (bx, wx, _) = cr_weights(nr, tx);
    let (by, wy, _) = cr_weights(nr, ty);
    let (bc, wc, _) = cr_weights(nu, tc);
    let mut acc = 0.0;
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                let fi = (bx + i) * sr;
                let fj = (by + j) * sr;
                let fk = (bc + k) * su;
                acc += wx[i] * wy[j] * wc[k] * fine[(fi * NR_FINE + fj) * NU_FINE + fk];
            }
        }
    }
    acc
}

fn main() {
    let threads: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    println!("# E(O) = {e_o:.12}  E(H) = {e_h:.12}  threads = {threads}");
    println!("# box: x, y in [{R_LO}, {R_HI}] bohr, c in [{C_LO}, {C_HI:.6}]");
    println!("# fine grid {NR_FINE} x {NR_FINE} x {NU_FINE}, held-out {N_HELD} points, seed {SEED:#x}\n");

    // The held-out set, drawn ONCE from the staked seed and shared across every stretch
    // and every grid size, so the columns of the sweep differ only in the grid.
    let mut st = SEED;
    let mut held: Vec<(f64, f64, f64)> = Vec::with_capacity(N_HELD);
    while held.len() < N_HELD {
        // Uniform in tau at the reference stretch so the draw is not concentrated in the
        // long tail, and rejected if it lands within a twentieth of a cell of a node of
        // ANY candidate grid — "none on nodes", including the coarse ones.
        let (t1, t2, t3) = (lcg(&mut st), lcg(&mut st), lcg(&mut st));
        let near_node = |t: f64, n: usize| {
            let f = t * (n - 1) as f64;
            (f - f.round()).abs() < 0.05
        };
        if NR_TRY.iter().any(|&n| near_node(t1, n) || near_node(t2, n))
            || NU_TRY.iter().any(|&n| near_node(t3, n))
        {
            continue;
        }
        let (x, y) = (r_of_tau(3.0, t1), r_of_tau(3.0, t2));
        let (x, y) = if x <= y { (x, y) } else { (y, x) };
        held.push((x, y, C_LO + (C_HI - C_LO) * t3));
    }

    for a in [2.0f64, 3.0, 4.0] {
        let t0 = Instant::now();
        let rs: Vec<f64> = (0..NR_FINE)
            .map(|i| r_of_tau(a, i as f64 / (NR_FINE - 1) as f64))
            .collect();
        let e_oh: Vec<f64> = rs.iter().map(|&r| pair_point(OXYGEN, HYDROGEN, r).e).collect();

        // Build the fine table: only `i <= j` is solved, the mirror node takes the SAME
        // float rather than a second rounding of the same number.
        let fine: Vec<Mutex<f64>> = (0..NR_FINE * NR_FINE * NU_FINE).map(|_| Mutex::new(0.0)).collect();
        let rows: Vec<usize> = (0..NR_FINE).collect();
        let next = AtomicUsize::new(0);
        std::thread::scope(|sc| {
            for _ in 0..threads {
                sc.spawn(|| loop {
                    let t = next.fetch_add(1, Ordering::SeqCst);
                    if t >= rows.len() {
                        break;
                    }
                    let i = rows[t];
                    for j in i..NR_FINE {
                        for k in 0..NU_FINE {
                            let cc = C_LO + (C_HI - C_LO) * k as f64 / (NU_FINE - 1) as f64;
                            let u = 1.0 - cc * cc;
                            let d = de3(rs[i], rs[j], u, e_o, e_h, e_oh[i], e_oh[j]);
                            *fine[(i * NR_FINE + j) * NU_FINE + k].lock().unwrap() = d;
                            *fine[(j * NR_FINE + i) * NU_FINE + k].lock().unwrap() = d;
                        }
                    }
                });
            }
        });
        let fine: Vec<f64> = fine.into_iter().map(|m| m.into_inner().unwrap()).collect();
        let peak = fine.iter().fold(0.0f64, |m, v| m.max(v.abs()));
        println!(
            "## stretch a = {a}   fine table built in {:.1} s, peak |dE3| = {peak:.4e} Ha",
            t0.elapsed().as_secs_f64()
        );
        println!(
            "   first three side nodes: {:.4} {:.4} {:.4} bohr  (spacing {:.4} at the compact end)",
            rs[0],
            rs[1],
            rs[2],
            rs[1] - rs[0]
        );

        // The truth at the held-out points, computed once per stretch (the draw is in tau
        // at a fixed reference stretch, so the geometries themselves do not move; this is
        // recomputed only because the pair-energy cache is keyed to the node list).
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
                    let u = 1.0 - c * c;
                    let d = de3(
                        x,
                        y,
                        u,
                        e_o,
                        e_h,
                        pair_point(OXYGEN, HYDROGEN, x).e,
                        pair_point(OXYGEN, HYDROGEN, y).e,
                    );
                    *truth[t].lock().unwrap() = d;
                });
            }
        });
        let truth: Vec<f64> = truth.into_iter().map(|m| m.into_inner().unwrap()).collect();

        print!("   nr \\ nu ");
        for nu in NU_TRY {
            print!("{nu:>12}");
        }
        println!("      solves(sorted half)");
        for nr in NR_TRY {
            print!("   {nr:>6}  ");
            for nu in NU_TRY {
                let mut worst = 0.0f64;
                for (t, &(x, y, c)) in held.iter().enumerate() {
                    let got = eval(&fine, nr, nu, a, x, y, c);
                    worst = worst.max((got - truth[t]).abs());
                }
                print!("{worst:>12.3e}");
            }
            println!("      {}", nr * (nr + 1) / 2);
        }
        println!();
    }
}
