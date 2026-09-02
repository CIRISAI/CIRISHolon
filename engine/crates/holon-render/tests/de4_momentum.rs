//! THE MOMENTUM AUDIT on the four-body sector: do its forces sum to zero?
//!
//! # Why this gate exists, and why it is FIRST
//!
//! Every completed seed of the dE4 quench arm violated the momentum bound by four to five
//! orders (|p|/bound 9.8e3 to 4.2e5 on 6 of 6) while energy drift stayed IN bound. That is
//! the one-gate-per-conservation-law signature: a channel that is in the force and not in
//! the ledger shows up on exactly one gate, and the gate that stays green tells you nothing
//! about the one that fired.
//!
//! `Sim::momentum_bound` is a pure ROUNDOFF bound — `8 · steps · eps · p_scale` — and it is
//! entitled to be, because every other sector applies its forces as equal and opposite
//! contributions of the SAME bit pattern (`push_side` adds `fx` to one partner and
//! subtracts the identical `fx` from the other), so they cancel exactly rather than
//! approximately. A sector that breaks that cancellation does not drift within the bound;
//! it leaves it by orders.
//!
//! # What is actually checked
//!
//! `a_pair` is FORCE, not acceleration: the integrator divides by mass at the point of use
//! (`half = 0.5 · dt / mass`), and `Sim::internal_force` returns `a_pair` under that name.
//! So the invariant is flatly stated: **the sum of `internal_force` over every atom is
//! exactly zero**, because walls, the spring and the thermostat live in `a_ext` and are the
//! only things allowed to inject net momentum.
//!
//! The control matters as much as the measurement. The same scene with the four-body sector
//! OFF must sum to exactly zero, or the test is measuring the pair and triple sectors and
//! would fire whatever the four-body code did.

use holon_render::sim::Sim;

#[path = "common/quartet.rs"]
mod quartet;
use quartet::quartet;

/// The summed internal force, and the scale to judge it against.
fn net_internal_force(s: &Sim) -> ((f64, f64, f64), f64) {
    let (mut fx, mut fy, mut fz) = (0.0, 0.0, 0.0);
    let mut scale = 0.0f64;
    for i in 0..s.n {
        let (x, y, z) = s.internal_force(i);
        fx += x;
        fy += y;
        fz += z;
        scale = scale.max((x * x + y * y + z * z).sqrt());
    }
    ((fx, fy, fz), scale.max(1e-30))
}

#[test]
fn the_control_sums_to_exactly_zero_without_the_four_body_sector() {
    let mut s = quartet(false);
    assert!(s.pairs_ready(), "the bank is missing a curve this scene needs");
    s.step();
    let ((fx, fy, fz), scale) = net_internal_force(&s);
    let net = (fx * fx + fy * fy + fz * fz).sqrt();
    assert!(scale > 1e-6, "the scene produced no internal force at all, so this proves nothing");
    // NOT an exact zero, and the reason is this test's own arithmetic rather than the
    // sim's. `push_side` cancels exactly IN THE ARRAY; summing that array in index order
    // is a separate floating-point sum with its own roundoff, so the instrument has a
    // floor of a few ulp of the force scale. Measured 3.2e-17 against 9.5e-2, a relative
    // 3.3e-16. The bar is set twelve orders above that and nine below the defect this
    // file was written for, so nothing about the separation is delicate.
    assert!(
        net <= 1e-14 * scale,
        "the pair and triple sectors do not cancel to roundoff: net {net:.6e} against a \
         force scale of {scale:.6e}, a relative {:.3e}. They add and subtract the same bit \
         pattern, so anything above roundoff here means the control itself is broken and \
         the four-body reading below cannot be attributed.",
        net / scale
    );
}

#[test]
fn the_four_body_forces_sum_to_zero_over_the_quartet() {
    let mut s = quartet(true);
    assert!(s.pairs_ready(), "the bank is missing a curve this scene needs");
    s.step();
    let ((fx, fy, fz), scale) = net_internal_force(&s);
    let net = (fx * fx + fy * fy + fz * fz).sqrt();
    assert!(
        scale > 1e-6,
        "the four-body sector contributed no force at all — the switch is off or the tables \
         are unloaded, and this test would pass without exercising anything"
    );
    // Roundoff only. The four-body force is built as equal-and-opposite pairs along each
    // O-H direction, so in exact arithmetic the sum is zero; what is allowed here is the
    // floating-point residue of that construction, not a systematic term.
    assert!(
        net <= 1e-12 * scale,
        "THE FOUR-BODY SECTOR INJECTS NET MOMENTUM: net internal force {net:.6e} against a \
         force scale of {scale:.6e}, a relative {:.3e}. Every force in this array must \
         cancel — walls, spring and thermostat are the only things allowed to change the \
         total, and they live in `a_ext`. This is what put |p|/bound at 9.8e3-4.2e5 on all \
         six banked dE4 seeds while energy stayed in bound.",
        net / scale
    );
}


/// The dE4-only force on each atom: ON minus OFF at identical positions.
fn de4_force(on: &Sim, off: &Sim, i: usize) -> [f64; 3] {
    let (fx, fy, fz) = on.internal_force(i);
    let (gx, gy, gz) = off.internal_force(i);
    [fx - gx, fy - gy, fz - gz]
}

/// The law the OLD scheme broke silently: radial-only finite differences summed to
/// zero (momentum looked fine at the raw-force level) while NOT being -grad U — every
/// tangential component, including every H-H force inside the correction, was missing.
/// This measures force-against-gradient directly: displace all four atoms along a fixed
/// asymmetric direction field, central-difference the four-body energy, and compare
/// with the analytic four-body force projected on the same field.
#[test]
fn the_four_body_force_is_the_gradient_of_the_four_body_energy() {
    let d: [[f64; 3]; 4] = [
        [0.31, -0.12, 0.05],
        [-0.20, 0.44, -0.11],
        [0.07, -0.33, 0.29],
        [-0.18, 0.01, -0.23],
    ];
    let delta = 5e-4;

    let mut on = quartet(true);
    on.compute_forces();
    let mut off = quartet(false);
    off.compute_forces();
    let mut proj = 0.0;
    for i in 0..4 {
        let f = de4_force(&on, &off, i);
        proj += f[0] * d[i][0] + f[1] * d[i][1] + f[2] * d[i][2];
    }

    let e_at = |sign: f64| -> f64 {
        let mut s = quartet(true);
        for i in 0..4 {
            s.atoms[i].x += sign * delta * d[i][0];
            s.atoms[i].y += sign * delta * d[i][1];
            s.atoms[i].z += sign * delta * d[i][2];
        }
        s.compute_forces();
        s.e_many
    };
    let du_dl = (e_at(1.0) - e_at(-1.0)) / (2.0 * delta);

    let scale = du_dl.abs().max(1e-8);
    let mismatch = (proj + du_dl).abs() / scale;
    assert!(
        mismatch < 1e-4,
        "four-body force is not the energy's gradient: F.d = {proj:e}, dU/dl = {du_dl:e}, \
         relative mismatch {mismatch:e} (the radial-only scheme fails this at order one)"
    );
}

/// Rotation invariance: the four-body torque sums to zero. The pairwise MBE3 shares
/// cancel exactly; the FCI gradient's net torque is zero in exact arithmetic and comes
/// back at solver-residual scale here.
#[test]
fn the_four_body_torque_sums_to_zero() {
    let mut on = quartet(true);
    on.compute_forces();
    let mut off = quartet(false);
    off.compute_forces();
    let mut t = [0.0f64; 3];
    let mut scale = 0.0f64;
    for i in 0..4 {
        let f = de4_force(&on, &off, i);
        let r = [on.atoms[i].x, on.atoms[i].y, on.atoms[i].z];
        t[0] += r[1] * f[2] - r[2] * f[1];
        t[1] += r[2] * f[0] - r[0] * f[2];
        t[2] += r[0] * f[1] - r[1] * f[0];
        let rn = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
        let fn_ = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
        scale = scale.max(rn * fn_);
    }
    let tn = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
    assert!(
        tn <= 1e-6 * scale.max(1e-12),
        "four-body torque does not cancel: |tau| = {tn:e} against scale {scale:e}"
    );
}
