//! G2 FIRED. Branch (b): investigate, never massage.
//!
//! Gate G2 stakes that a third hydrogen brought to relaxed in-model water refuses — its
//! best binding anywhere shallower than water's own second O-H bond by at least 5x. It
//! measured **1.76x**: 0.0925 Ha against 0.1631 Ha. That is a reading, and the question it
//! raises has exactly two answers, which this file separates:
//!
//! 1. **The MODEL binds a third hydrogen.** STO-3G full CI would then say neutral H3O is
//!    bound, and G2's premise is wrong for this model rather than the table being wrong.
//! 2. **The MBE3 TRUNCATION over-binds it.** A fourth atom introduces a four-body term
//!    that a three-body expansion does not have, and if that term is large and repulsive
//!    the truncation is unfit for the four-atom case — which is a finding about the
//!    expansion, reportable and separable from everything G1 and T1 establish.
//!
//! The discriminator is direct and cheap: (O, H, H, H) is 8 orbitals and 11 electrons,
//! 1568 determinants in the minimal-|Sz| sector, so the FULL CI energy at the geometry the
//! MBE3 scan calls deepest can simply be computed and the two compared.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_g2_probe [threads]
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};
use holon_chem::trimer;
use holon_chem::water::{self, hh_side};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

const PI: f64 = std::f64::consts::PI;

/// The model's own relaxed water, from `examples/s2_design.rs`.
const R_W: f64 = 1.9435740105;
const TH_W_DEG: f64 = 96.75788837;

fn c3(p: [f64; 3]) -> [D2; 3] {
    [D2::c(p[0]), D2::c(p[1]), D2::c(p[2])]
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn main() {
    let threads: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(6);
    // this producer owns the machine and runs its own pool: split the cores between
    // the pool and the lane kernel beneath it, or the two multiply (scheduling only)
    holon_chem::lanes::set_lane_threads_for_pool(threads);
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/s2/s2_water_table.txt"),
    )
    .expect("the committed (O,H,H) table");
    let t = water::from_text(&src).expect("it parses");
    let h3 = trimer::generate().expect("the H3 table");
    println!("# E(O) = {e_o:.12}  E(H) = {e_h:.12}");

    let th_w = TH_W_DEG * PI / 180.0;
    let o = [0.0, 0.0, 0.0];
    let h1 = [R_W * (th_w / 2.0).cos(), R_W * (th_w / 2.0).sin(), 0.0];
    let h2 = [R_W * (th_w / 2.0).cos(), -R_W * (th_w / 2.0).sin(), 0.0];
    let d_h1h2 = dist(h1, h2);

    // --------------------------------------------------- where the MBE3 scan is deepest
    let mbe3_binding = |p: [f64; 3]| -> f64 {
        let (a, b, c) = (dist(o, p), dist(h1, p), dist(h2, p));
        if a < 0.9 || b < 0.9 || c < 0.9 {
            return f64::NEG_INFINITY;
        }
        let pairs = (pair_point(OXYGEN, HYDROGEN, a).e - e_o - e_h)
            + (pair_point(HYDROGEN, HYDROGEN, b).e - 2.0 * e_h)
            + (pair_point(HYDROGEN, HYDROGEN, c).e - 2.0 * e_h);
        let (ta, _) = t.eval(R_W, a, b);
        let (tb, _) = t.eval(R_W, a, c);
        let (tc, _) = h3.eval([d_h1h2, b, c]);
        -(pairs + ta + tb + tc)
    };

    let (nr, nt, np) = (25, 25, 25);
    let jobs: Vec<[f64; 3]> = (0..nr)
        .flat_map(|i| {
            let rr = 1.0 + (6.0 - 1.0) * i as f64 / (nr - 1) as f64;
            (0..nt).flat_map(move |j| {
                let a = PI * j as f64 / (nt - 1) as f64;
                (0..np).map(move |k| {
                    let b = 2.0 * PI * k as f64 / np as f64;
                    [rr * a.sin() * b.cos(), rr * a.sin() * b.sin(), rr * a.cos()]
                })
            })
        })
        .collect();
    let best = Mutex::new((f64::NEG_INFINITY, [0.0f64; 3]));
    let next = AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let n = next.fetch_add(1, Ordering::SeqCst);
                if n >= jobs.len() {
                    break;
                }
                let v = mbe3_binding(jobs[n]);
                let mut b = best.lock().unwrap();
                if v > b.0 {
                    *b = (v, jobs[n]);
                }
            });
        }
    });
    let (deep, at) = best.into_inner().unwrap();

    let (a, b, c) = (dist(o, at), dist(h1, at), dist(h2, at));
    println!("\n## the MBE3 scan's deepest third-hydrogen binding");
    println!("   binding    = {deep:.6} Ha");
    println!("   O-H3       = {a:.4} bohr");
    println!("   H1-H3      = {b:.4} bohr");
    println!("   H2-H3      = {c:.4} bohr");
    println!("   (relaxed water is r_OH = {R_W:.4}, H-H = {d_h1h2:.4})");

    // ------------------------------------------------------- what the FULL CI says there
    let e_h3o = solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN],
        vec![c3(o), c3(h1), c3(h2), c3(at)],
    );
    let e_water = solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![c3(o), c3(h1), c3(h2)],
    );
    let true_binding = (e_water.e.v + e_h) - e_h3o.e.v;
    println!("\n## what the model itself says at that geometry");
    println!(
        "   E(OHHH)    = {:.9} Ha   ({} determinants, {} basis, residual {:.1e})",
        e_h3o.e.v, e_h3o.n_det, e_h3o.n_basis, e_h3o.residual
    );
    println!("   E(OHH)     = {:.9} Ha   ({} determinants)", e_water.e.v, e_water.n_det);
    println!("   FULL-CI binding of the third H = {true_binding:+.6} Ha");
    println!("   MBE3 binding at the same point = {deep:+.6} Ha");
    println!("   the four-body term dE4         = {:+.6} Ha", true_binding - deep);

    // ------------------------------------------- and along the approach, not only at one point
    println!("\n## the same comparison along the O-H3 approach, third H on the C2 axis");
    println!(
        "   {:>7} {:>14} {:>14} {:>14}",
        "O-H3", "MBE3 binding", "full-CI", "dE4"
    );
    let axis = [1.0f64, 0.0, 0.0]; // the water's own C2 axis, away from the two hydrogens
    for r in [1.2f64, 1.5, 1.8, 2.1, 2.5, 3.0, 4.0, 5.0] {
        let p = [-axis[0] * r, 0.0, 0.0];
        let m = mbe3_binding(p);
        let e4 = solve_geometry(
            &[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN],
            vec![c3(o), c3(h1), c3(h2), c3(p)],
        )
        .e
        .v;
        let tb = (e_water.e.v + e_h) - e4;
        println!("   {r:>7.2} {m:>14.6} {tb:>14.6} {:>14.6}", tb - m);
    }

    println!("\n## the CONTROL: water's own second O-H bond, both ways");
    let e_oh = {
        let mut r = 1.8f64;
        for _ in 0..80 {
            let p = pair_point(OXYGEN, HYDROGEN, r);
            r += p.f / p.e2.max(1e-6);
            if p.f.abs() < 1e-13 {
                break;
            }
        }
        println!("   relaxed OH: r = {r:.6} bohr");
        pair_point(OXYGEN, HYDROGEN, r).e
    };
    println!(
        "   full-CI second O-H bond = {:+.6} Ha",
        (e_oh + e_h) - e_water.e.v
    );
    let z = hh_side(R_W, R_W, th_w.cos());
    let mbe3_water = e_o + 2.0 * e_h
        + (pair_point(OXYGEN, HYDROGEN, R_W).e - e_o - e_h) * 2.0
        + (pair_point(HYDROGEN, HYDROGEN, z).e - 2.0 * e_h)
        + t.eval(R_W, R_W, z).0;
    println!(
        "   MBE3    second O-H bond = {:+.6} Ha   (MBE3 water is {:.3e} Ha off full CI)",
        (e_oh + e_h) - mbe3_water,
        (mbe3_water - e_water.e.v).abs()
    );
}
