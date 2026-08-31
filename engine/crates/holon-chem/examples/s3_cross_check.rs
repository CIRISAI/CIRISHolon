//! Is the kink near collinear a STATE CROSSING, or is the solver picking the wrong root?
//!
//! `s3_angle_slice` finds a corner in `dE3` at `theta ~ 174.9 deg` on the slice that
//! carries the shipped table's worst held-out error: steeply rising curvature below it,
//! flat and smooth above it. Two readings produce that shape and they call for opposite
//! responses.
//!
//! * A real crossing of two states of different spatial symmetry. The ground state is the
//!   LOWER ENVELOPE, the corner is physical, and no interpolant on any coordinate removes
//!   it — the successor has to place a grid line on it or accept the error.
//! * Davidson converging onto the UPPER branch on one side of a near-degeneracy. Then the
//!   corner is ours, the energies past it are wrong, and the fix is the solver.
//!
//! The discriminator is a warm start across the corner: take the converged eigenvector
//! from a geometry on one branch and start the neighbouring solve from it. If the
//! cold-start answer is already the lowest, warm starting from either side cannot beat it.
//! If a warm start finds a LOWER energy at the same geometry, the cold solve was on the
//! wrong root and the variational bound says so without appeal.
//!
//! ```text
//! cargo run --release -p holon-chem --example s3_cross_check
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::fci::{ci_ints, davidson_eigh_from, Order, DAVIDSON_MAX_ITER, DAVIDSON_REQUESTED_TOLERANCE};
use holon_chem::pair::geometry_problem;
use std::sync::atomic::Ordering;

/// The default slice and window straddle the corner `s3_angle_slice` located at
/// c ~ 1.41278 on (1.766, 2.576). The second seam, at c ~ 0.436 on (2.621, 2.703), is
/// reached through the arguments — one instrument, both seams, no second transcription.
const X_DEFAULT: f64 = 1.766;
const Y_DEFAULT: f64 = 2.576;
const LO_DEFAULT: f64 = 1.4125;
const HI_DEFAULT: f64 = 1.4135;

fn geom(x: f64, y: f64, c: f64) -> Vec<[D2; 3]> {
    let u = 1.0 - c * c;
    let s = (1.0 - u * u).max(0.0).sqrt();
    vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
        [D2::c(x), D2::c(0.0), D2::c(0.0)],
        [D2::c(y * u), D2::c(y * s), D2::c(0.0)],
    ]
}

fn solve(x: f64, y: f64, c: f64, start: Option<&[f64]>) -> (f64, Vec<f64>, f64, usize, &'static str) {
    let (space, mo, nuc) = geometry_problem(&[OXYGEN, HYDROGEN, HYDROGEN], geom(x, y, c));
    let ci0 = ci_ints(&mo, Order::Value);
    let diag = space.diagonal(&ci0);
    let min_diag = diag.iter().copied().fold(f64::INFINITY, f64::min);
    let (e, v, iters, _resid, exit) = davidson_eigh_from(
        &space,
        &ci0,
        &diag,
        DAVIDSON_REQUESTED_TOLERANCE,
        DAVIDSON_MAX_ITER.load(Ordering::Relaxed),
        start,
    );
    (e + nuc.v, v, min_diag - e, iters, exit.label())
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let x: f64 = a.first().and_then(|s| s.parse().ok()).unwrap_or(X_DEFAULT);
    let y: f64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(Y_DEFAULT);
    let lo: f64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(LO_DEFAULT);
    let hi: f64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(HI_DEFAULT);
    let n: usize = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(9);
    let cs: Vec<f64> = (0..n).map(|i| lo + (hi - lo) * i as f64 / (n - 1) as f64).collect();

    println!("# the corner on the slice x = {x}, y = {y}, c in [{lo}, {hi}]");
    println!("# E is the TOTAL (O,H,H) energy; margin is min_i H_ii - E (electronic)\n");
    println!("   {:>10} {:>9} {:>17} {:>17} {:>12} {:>10}", "c", "theta", "E cold", "E warm-from-below", "dE warm-cold", "margin");

    // One converged vector from BELOW the corner, carried up through every geometry.
    let (_, mut carrier, _, _, _) = solve(x, y, cs[0], None);
    let mut any_lower = false;
    for &c in cs.iter() {
        let (e_cold, v_cold, margin, _iters, _exit) = solve(x, y, c, None);
        let (e_warm, v_warm, _, _, _) = solve(x, y, c, Some(&carrier));
        let d = e_warm - e_cold;
        if d < -1e-9 {
            any_lower = true;
        }
        let theta = (1.0f64 - c * c).clamp(-1.0, 1.0).acos().to_degrees();
        println!("   {c:>10.6} {theta:>9.4} {e_cold:>17.9} {e_warm:>17.9} {d:>12.3e} {margin:>10.5}");
        // Carry the BETTER of the two forward, so the walk never loses a branch it found.
        carrier = if e_warm <= e_cold { v_warm } else { v_cold };
    }
    println!();
    if any_lower {
        println!("   A warm start found a LOWER energy than the cold solve at the same geometry.");
        println!("   The corner is the SOLVER's: cold Davidson is on the upper branch somewhere.");
    } else {
        println!("   No warm start beat its cold solve anywhere across the corner. The cold");
        println!("   answers are the lower envelope, so the corner is a real crossing of two");
        println!("   states and belongs to the surface, not to the eigensolver.");
    }
}
