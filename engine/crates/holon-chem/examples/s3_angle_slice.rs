//! WHY the held-out maximum sits where it sits: is the surface smooth there at all?
//!
//! # The obligation this discharges
//!
//! SATURATION-3's freeze makes the angle-axis convergence anomaly a design-time
//! obligation — "explain it or show the chosen coordinate beats it before committing any
//! grid". SATURATION-2 published it as "the angle axis converges at about 5x per doubling
//! where a C1 cubic should give 16x". Two of those numbers are now measured rather than
//! asserted, by `s2_build --gauge`:
//!
//! * 16x was wrong. Catmull-Rom takes its node slopes from centred differences, so it is
//!   third order, not fourth. Gauged on two planted analytic functions with the SAME eval
//!   on the SAME subgrids at the SAME draw, this interpolator delivers 9.3x to 10.9x per
//!   doubling of the angle axis — between `h^3` and `h^4`, and nowhere near 16x.
//! * The end intervals are not the cause. On planted data the "all points" and "no end
//!   interval" columns are IDENTICAL to every digit, so the one-sided slope at the c
//!   boundaries costs nothing on smooth data. On the REAL surface they differ by 3x on the
//!   coarse grids, so end intervals do carry the coarse-grid maximum — but at the shipped
//!   65 x 49 the worst point is at c = 1.373, in the SECOND-to-last interval, and the two
//!   columns coincide.
//!
//! What is left is the surface. At the shipped size BOTH axes stall at the same value
//! (7.68e-4) at the same point, (x, y, c) = (1.766, 2.576, 1.373): refining the radial
//! axis 33 -> 65 buys 1.38x and refining the angle axis 25 -> 49 buys 1.35x, where the
//! planted functions buy 9x on the same rungs. A feature that survives refinement in both
//! directions is not a resolution problem; it is a feature the interpolant cannot
//! represent. This walks the surface through that point to find out what it is.
//!
//! `d2` and `d3` are centred divided differences on the scan's own spacing. A smooth
//! surface gives a `d3` that is bounded and slowly varying; a cusp or a state crossing
//! gives one that blows up like `1/h^2` at one sample.
//!
//! ```text
//! cargo run --release -p holon-chem --example s3_angle_slice -- [n] [c_lo] [c_hi]
//! ```

use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point};
use holon_chem::water::{de3_with, C_HI, C_LO};

/// The held-out maximum's geometry, from `s2_build`'s tableau — the default slice. The
/// second floor the `--gauge` masks expose sits at (2.621, 2.703, 0.461), so the sides are
/// arguments rather than constants: one instrument, two seams.
const X_DEFAULT: f64 = 1.766;
const Y_DEFAULT: f64 = 2.576;
const C_STAR: f64 = 1.373;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let n: usize = a.first().and_then(|s| s.parse().ok()).unwrap_or(201);
    let lo: f64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(C_LO);
    let hi: f64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(C_HI);
    let x: f64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(X_DEFAULT);
    let y: f64 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(Y_DEFAULT);

    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let e_ox = pair_point(OXYGEN, HYDROGEN, x).e;
    let e_oy = pair_point(OXYGEN, HYDROGEN, y).e;

    println!("# dE3 along the angle axis at x = {x}, y = {y}");
    println!("# c in [{lo:.6}, {hi:.6}], {n} points; the shipped 49-node grid has spacing {:.6}", (C_HI - C_LO) / 48.0);
    println!("# theta = acos(1 - c^2); z is the H-H side, which is what E(HH) is read at");
    println!("# d2, d3: centred divided differences in c on this scan's own spacing\n");

    let h = (hi - lo) / (n - 1) as f64;
    let cs: Vec<f64> = (0..n).map(|i| lo + h * i as f64).collect();
    let v: Vec<f64> = cs
        .iter()
        .map(|&c| de3_with(x, y, 1.0 - c * c, e_o, e_h, e_ox, e_oy))
        .collect();

    println!("   {:>10} {:>9} {:>9} {:>16} {:>13} {:>13}", "c", "theta", "z", "dE3", "d2", "d3");
    let mut worst_d3 = (0.0f64, 0.0f64);
    for i in 0..n {
        let c = cs[i];
        let u = 1.0 - c * c;
        let theta = u.clamp(-1.0, 1.0).acos().to_degrees();
        let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
        let (d2, d3) = if i >= 2 && i + 2 < n {
            (
                (v[i + 1] - 2.0 * v[i] + v[i - 1]) / (h * h),
                (v[i + 2] - 2.0 * v[i + 1] + 2.0 * v[i - 1] - v[i - 2]) / (2.0 * h * h * h),
            )
        } else {
            (f64::NAN, f64::NAN)
        };
        if d3.is_finite() && d3.abs() > worst_d3.0 {
            worst_d3 = (d3.abs(), c);
        }
        println!("   {c:>10.6} {theta:>9.4} {z:>9.5} {:>16.9} {d2:>13.4e} {d3:>13.4e}", v[i]);
    }
    println!("\n   largest |d3| {:.4e} at c = {:.6}", worst_d3.0, worst_d3.1);
    println!("   the held-out maximum sits at c = {C_STAR}");
    println!(
        "   A smooth surface gives a bounded, slowly varying d3. One sample orders of\n   \
         magnitude above its neighbours is a kink, and no amount of grid buys it back."
    );
}
