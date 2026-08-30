//! SATURATION-2: the model's own full-FCI water optimum — the reference gate G1 scores
//! against.
//!
//! Gate G1 asks whether the MBE3 optimum reproduces the model's own optimum, so the
//! model's own optimum has to exist as a number before the gate can be written. It is
//! located here by Newton on the EXACT first and second derivatives the dual-number
//! route already carries — in the symmetric stretch and in the angle — with the
//! antisymmetric direction's curvature REPORTED, so "a minimum" is a reading rather than
//! an assumption. Nature's 104.5 degrees and 0.957 angstrom are printed beside the
//! answer as LABELLED CONTEXT and nothing is ever compared against them.
//!
//! The truncation radius and the closed-angle fence are measured separately, in
//! `examples/s2_domain.rs`.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_design
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, solve_geometry};

const PI: f64 = std::f64::consts::PI;

/// Bohr per angstrom. Used ONLY to print nature's bond length in the model's units as
/// labelled context; no computed quantity passes through it.
const BOHR_PER_ANGSTROM: f64 = 1.8897261246257702;

/// `cos` of a dual number, via the second-order chain rule `compose` already provides.
fn dcos(t: D2) -> D2 {
    t.compose(t.v.cos(), -t.v.sin(), -t.v.cos())
}
fn dsin(t: D2) -> D2 {
    t.compose(t.v.sin(), t.v.cos(), -t.v.sin())
}

/// Total energy of the (O, H, H) system with oxygen at the origin and the two hydrogens
/// at `r1`, `r2` and the angle `theta` at oxygen, all as dual numbers so ONE of them may
/// carry the derivative.
fn water(r1: D2, r2: D2, theta: D2) -> D2 {
    let half = theta * 0.5;
    let (c, s) = (dcos(half), dsin(half));
    solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [r1 * c, r1 * s, D2::c(0.0)],
            [r2 * c, -(r2 * s), D2::c(0.0)],
        ],
    )
    .e
}

/// One Newton step on a scalar coordinate: `x - f'/f''`, damped so a step never leaves
/// the physical region.
fn newton(x: f64, e: D2, max_step: f64) -> f64 {
    let step = if e.e.abs() > 1e-12 { -e.d / e.e } else { 0.0 };
    x + step.clamp(-max_step, max_step)
}

fn part_a() -> (f64, f64, f64) {
    println!("## the model's own full-FCI water optimum (G1's reference)\n");
    let (mut r, mut th) = (1.8f64, 1.8f64);
    for it in 0..12 {
        let er = water(D2::var(r), D2::var(r), D2::c(th));
        r = newton(r, er, 0.15);
        let et = water(D2::c(r), D2::c(r), D2::var(th));
        th = newton(th, et, 0.20);
        if it >= 8 {
            println!(
                "   iter {it:2}  r = {r:.12}  theta = {:.10} deg   |dE/dr| = {:.2e}  |dE/dtheta| = {:.2e}",
                th * 180.0 / PI,
                er.d.abs(),
                et.d.abs()
            );
        }
    }
    // Final readings at the located point, each derivative exact.
    let er = water(D2::var(r), D2::var(r), D2::c(th));
    let et = water(D2::c(r), D2::c(r), D2::var(th));
    // The ANTISYMMETRIC direction: r1 = r + s, r2 = r - s. Its first derivative is zero by
    // the H <-> H exchange symmetry; its second derivative is what says "minimum" rather
    // than "saddle", and it is REPORTED, not assumed.
    let s = D2::var(0.0);
    let ea = water(D2::c(r) + s, D2::c(r) - s, D2::c(th));

    println!("\n   E(H2O)            = {:.12} Ha", er.v);
    println!("   r_OH              = {:.10} bohr", r);
    println!("   theta_HOH         = {:.8} deg", th * 180.0 / PI);
    println!("   dE/dr             = {:+.3e} Ha/bohr      (stationary)", er.d);
    println!("   dE/dtheta         = {:+.3e} Ha/rad       (stationary)", et.d);
    println!("   d2E/dr2 (sym)     = {:+.6e} Ha/bohr^2", er.e);
    println!("   d2E/dtheta2       = {:+.6e} Ha/rad^2", et.e);
    println!("   d2E/ds2 (antisym) = {:+.6e} Ha/bohr^2   [> 0 => a minimum, not a saddle]", ea.e);
    println!("   dE/ds  (antisym)  = {:+.3e} Ha/bohr      [zero by H<->H exchange]", ea.d);

    // The linear geometry, relaxed in r at theta = 180 deg: how much the bend is worth.
    let mut rl = r;
    for _ in 0..10 {
        let e = water(D2::var(rl), D2::var(rl), D2::c(PI));
        rl = newton(rl, e, 0.15);
    }
    let el = water(D2::var(rl), D2::var(rl), D2::c(PI));
    println!(
        "\n   relaxed LINEAR (theta = 180): r = {rl:.8} bohr, E = {:.12} Ha",
        el.v
    );
    println!(
        "   bending is worth  {:.6} Ha = {:.2} kcal/mol-equivalent in model units",
        el.v - er.v,
        (el.v - er.v) * 627.5
    );
    println!(
        "\n   [LABELLED CONTEXT, never compared against: nature's water is 104.5 deg and\n    0.957 angstrom = {:.4} bohr. STO-3G's in-model answer is the claim.]",
        0.957 * BOHR_PER_ANGSTROM
    );
    (r, th, er.v)
}

fn main() {
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    println!("# E(O) = {e_o:.12} Ha   E(H) = {e_h:.12} Ha\n");
    part_a();
}
