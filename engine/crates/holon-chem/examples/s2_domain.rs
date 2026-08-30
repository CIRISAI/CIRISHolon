//! SATURATION-2 domain design: where the (O, H, H) surface may be truncated, and where
//! the closed-angle corner stops being a surface at all.
//!
//! # What the first sweep is, and why the prereg's one-line rule needed measuring
//!
//! SATURATION-1's AMENDMENT A1 truncates on the SECOND-SMALLEST side, because `dE3`
//! vanishes exactly when some atom is far from BOTH of the others — which is the
//! statement that two of the three sides are long, i.e. that `s2` is long. That reasoning
//! is about geometry, not about species, so it transfers to this triple unchanged.
//!
//! What does NOT transfer is the shape of the table's box. Here the two axes are the two
//! O-H sides (the pair the H <-> H symmetry may exchange) and the H-H side is the third
//! coordinate, so the box `x, y <= R_HI` bounds BOTH O-H sides and leaves the H-H side
//! free. A point just outside that box has `y > R_HI`, and the question this sweep asks
//! is the honest one: over the whole shell `max(O-H) = b`, how big does `dE3` actually
//! get? Whatever that number is, it is what zeroing the surface at `b` costs.
//!
//! The sweep reaches the CLOSED-angle end of the third coordinate on purpose. `s2_design`
//! stopped at `c = 0.30` and reported its worst there, at the edge of its own grid, which
//! is a reading of the grid rather than of the surface.
//!
//! # The second sweep
//!
//! At `theta -> 0` with the two O-H sides equal, the hydrogens meet: the basis goes
//! linearly dependent and there is no surface. The `1/z` nuclear repulsion cancels
//! between `E(OHH)` and `E(HH)` by construction, so what is being located is where the
//! ELECTRONIC part stops being computable, and the fence `C_LO` is put above it.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_domain [threads]
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

const PI: f64 = std::f64::consts::PI;
const SQRT2: f64 = std::f64::consts::SQRT_2;

/// The smallest O-H side any of this sweep considers, matching the prereg's staked
/// domain floor.
const R_LO: f64 = 0.9;

fn c3(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

/// `dE3` at the triangle with O-H sides `x` and `y` and `u = cos(theta_HOH)`.
///
/// Every pair energy goes through the SAME general N-centre route as the triple, so the
/// difference below is the three-body term and not a difference between two solvers.
fn de3(x: f64, y: f64, u: f64, e_o: f64, e_h: f64, e_ox: f64, e_oy: f64) -> (f64, f64) {
    let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
    let s = (1.0 - u * u).max(0.0).sqrt();
    let e3 = solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![c3(0.0, 0.0, 0.0), c3(x, 0.0, 0.0), c3(y * u, y * s, 0.0)],
    )
    .e
    .v;
    (
        e3 + e_o + 2.0 * e_h - e_ox - e_oy - pair_point(HYDROGEN, HYDROGEN, z).e,
        z,
    )
}

/// The worst reading on one shell, and the geometry that carried it.
#[derive(Clone, Copy, Default)]
struct Worst {
    d: f64,
    x: f64,
    theta: f64,
    z: f64,
    /// The second-smallest of the three sides at the worst point — the quantity
    /// AMENDMENT A1 truncates on, reported so the box rule can be read against it.
    s2: f64,
}

fn main() {
    let threads: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(6);
    let t0 = Instant::now();
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    println!("# E(O) = {e_o:.12} Ha   E(H) = {e_h:.12} Ha   threads = {threads}");

    // ------------------------------------------------------------------ the shell sweep
    const NX: usize = 21;
    const NC: usize = 29;
    /// Closed-angle floor of the sweep. Below this the two hydrogens are inside a bohr of
    /// each other on the diagonal and the second sweep is what covers that corner.
    const C_MIN: f64 = 0.05;
    let bs: Vec<f64> = vec![
        4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
    ];
    println!(
        "\n## the truncation shell: worst |dE3| anywhere with max(O-H) = b\n\
         ##   x swept {NX} ways over [{R_LO}, b], cos(theta) swept {NC} ways over \
         c = sqrt(1-u) in [{C_MIN}, {SQRT2:.4}]\n"
    );

    let jobs: Vec<(usize, usize)> = (0..bs.len())
        .flat_map(|i| (0..NX).map(move |j| (i, j)))
        .collect();
    let next = AtomicUsize::new(0);
    let acc: Vec<Mutex<Worst>> = bs.iter().map(|_| Mutex::new(Worst::default())).collect();
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let t = next.fetch_add(1, Ordering::SeqCst);
                if t >= jobs.len() {
                    break;
                }
                let (i, j) = jobs[t];
                let b = bs[i];
                let x = R_LO + (b - R_LO) * j as f64 / (NX - 1) as f64;
                let e_ox = pair_point(OXYGEN, HYDROGEN, x).e;
                let e_oy = pair_point(OXYGEN, HYDROGEN, b).e;
                let mut local = Worst::default();
                for k in 0..NC {
                    let cc = C_MIN + (SQRT2 - C_MIN) * k as f64 / (NC - 1) as f64;
                    let u: f64 = 1.0 - cc * cc;
                    let (d, z) = de3(x, b, u, e_o, e_h, e_ox, e_oy);
                    if d.abs() > local.d.abs() {
                        let mut s = [x, b, z];
                        s.sort_by(|p, q| p.partial_cmp(q).unwrap());
                        local = Worst {
                            d,
                            x,
                            theta: u.clamp(-1.0, 1.0).acos() * 180.0 / PI,
                            z,
                            s2: s[1],
                        };
                    }
                }
                let mut slot = acc[i].lock().unwrap();
                if local.d.abs() > slot.d.abs() {
                    *slot = local;
                }
            });
        }
    });
    println!("   b      worst |dE3|    at x     theta      z(H-H)   s2 at that point");
    for (i, b) in bs.iter().enumerate() {
        let w = *acc[i].lock().unwrap();
        println!(
            "   {b:5.1}  {:.4e}     {:5.2}   {:6.1}    {:6.2}   {:6.2}",
            w.d.abs(),
            w.x,
            w.theta,
            w.z,
            w.s2
        );
    }

    // ------------------------------------------------------------- the closed-angle fence
    println!("\n## the closed-angle fence: the two hydrogens meeting on the diagonal x = y\n");
    for r in [0.9f64, 1.4, 1.94, 3.0] {
        let e_ox = pair_point(OXYGEN, HYDROGEN, r).e;
        println!("   x = y = {r:.2} bohr");
        for cc in [0.40f64, 0.30, 0.20, 0.15, 0.10, 0.07, 0.05, 0.03, 0.02, 0.01] {
            let u = 1.0 - cc * cc;
            let z = (2.0 * r * r * (1.0 - u)).max(0.0).sqrt();
            let (d, _) = de3(r, r, u, e_o, e_h, e_ox, e_ox);
            println!(
                "      c = {cc:5.3}  theta = {:6.2} deg  z = {z:.4}  dE3 = {d:+.6e}",
                u.clamp(-1.0, 1.0).acos() * 180.0 / PI
            );
        }
    }

    println!("\n# total {:.1} s", t0.elapsed().as_secs_f64());
}
