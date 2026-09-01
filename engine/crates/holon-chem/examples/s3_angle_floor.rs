//! DERIVE the angle axis's closed end, per table, from where the basis actually fails.
//!
//! # Why this is not a constant
//!
//! SATURATION-2's water table fences the closed-angle corner at `C_LO = 0.05`, and it
//! needs to: at `theta -> 0` with the two O-H sides EQUAL, the two hydrogens converge on
//! each other, their 1s orbitals approach unit overlap, and the basis goes linearly
//! dependent — there is no surface to tabulate. That number was measured for that table
//! and then inherited by this lane's species-general sweep, which is the defect
//! `saturation3-mesh` found: every row of the first (Cl,H,H) sweep put its worst |dE3| at
//! exactly `theta = 4.05 deg`, which is `arccos(1 - 0.05^2)` — the inherited edge, not a
//! property of the surface.
//!
//! The real constraint is not an angle at all. It is a minimum separation between the two
//! NON-APEX atoms, and the angle it implies depends on where you are in the box:
//!
//! ```text
//! z^2 = x^2 + y^2 - 2 x y u,   and on the diagonal x = y:   z = x * c * sqrt(2)
//! ```
//!
//! so the closest the pair ever comes, for a given `c`, is at the box's inner radial end.
//! The floor is therefore `c_lo = z_min / (x_lo * sqrt(2))` — species-general, because
//! `z_min` is a fact about the two orbitals and `x_lo` is the box the table declares.
//!
//! This measures `z_min` rather than assuming it: walk the two non-apex atoms together on
//! the diagonal and watch the smallest eigenvalue of the AO overlap matrix. `S` is
//! positive definite exactly while the basis is independent, and its smallest eigenvalue
//! is how much room is left.
//!
//! ```text
//! cargo run --release -p holon-chem --example s3_angle_floor -- <APEX> <B> <C> [x_lo]
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{by_symbol, Species};
use holon_chem::fci::jacobi_eigh;
use holon_chem::md::ao_integrals;
use holon_chem::pair::{atom_energy, build_basis, derive_range};

/// The declared floor on the overlap matrix's smallest eigenvalue.
///
/// DECLARED, not measured: `cholesky_orthonormaliser` fails outright below zero, and a
/// basis approaching that is producing energies whose conditioning is worse than the
/// quantity being tabulated. 1e-6 leaves four orders of headroom against an `f64`
/// Cholesky and is where this lane fences rather than where the arithmetic dies.
const S_MIN_FLOOR: f64 = 1e-6;

fn c3(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let sp = |i: usize| -> Species {
        by_symbol(&a[i]).unwrap_or_else(|| panic!("unknown element {}", a[i]))
    };
    let (apex, b1, b2) = (sp(0), sp(1), sp(2));
    let (x_lo_default, _) =
        derive_range(apex, b1, atom_energy(apex) + atom_energy(b1));
    let x_lo: f64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(x_lo_default);

    println!(
        "# ANGLE FLOOR for the ({}, {}, {}) table — apex {}, non-apex pair {}-{}",
        apex.symbol, b1.symbol, b2.symbol, apex.symbol, b1.symbol, b2.symbol
    );
    println!("# x_lo = {x_lo:.4} bohr (pair::derive_range's inner end for the apex pair)");
    println!("# walking the non-apex pair together on the diagonal x = y = x_lo");
    println!("# S_MIN_FLOOR = {S_MIN_FLOOR:.0e} — the declared fence on min eig(S)\n");

    println!("   {:>10} {:>10} {:>14} {:>10}", "z", "c", "min eig(S)", "verdict");
    let mut z_min = f64::NAN;
    // Geometric walk: each rung is 0.8x the last, so the fence is bracketed to 20% and
    // the scan cannot step over a collapse that a linear walk would miss.
    let mut z = 2.0f64;
    for _ in 0..40 {
        // On the diagonal, z = x * c * sqrt(2), so this z corresponds to:
        let c = z / (x_lo * std::f64::consts::SQRT_2);
        if c > std::f64::consts::SQRT_2 {
            z *= 0.8;
            continue;
        }
        let u = 1.0 - c * c;
        let s = (1.0 - u * u).max(0.0).sqrt();
        // BASIS ONLY. The question is whether `S` is still positive definite, which is
        // answered by assembling the overlap matrix — the electronic structure is not
        // consulted and must not be. The first version of this file called
        // `solve_geometry`, which solves the FCI at every rung: forty full solves to ask
        // a linear-algebra question, and worse, the near-singular rungs are exactly where
        // Davidson thrashes, so the instrument was slowest precisely where the answer
        // mattered. An instrument that costs more than its question will not be run often
        // enough to be useful.
        let sol = std::panic::catch_unwind(|| {
            let basis = build_basis(
                &[apex, b1, b2],
                vec![c3(0.0, 0.0, 0.0), c3(x_lo, 0.0, 0.0), c3(x_lo * u, x_lo * s, 0.0)],
            );
            let n = basis.n;
            let ao = ao_integrals(&basis);
            let sv: Vec<f64> = ao.s.iter().map(|d| d.v).collect();
            let (eigs, _) = jacobi_eigh(&sv, n);
            eigs[0]
        });
        match sol {
            Ok(smin) => {
                let ok = smin >= S_MIN_FLOOR;
                println!(
                    "   {z:>10.6} {c:>10.6} {smin:>14.4e} {:>10}",
                    if ok { "ok" } else { "FENCED" }
                );
                if !ok && z_min.is_nan() {
                    z_min = z;
                }
            }
            Err(_) => {
                println!("   {z:>10.6} {c:>10.6} {:>14} {:>10}", "REFUSED", "FENCED");
                if z_min.is_nan() {
                    z_min = z;
                }
            }
        }
        z *= 0.8;
    }

    println!();
    if z_min.is_finite() {
        let c_lo = z_min / (x_lo * std::f64::consts::SQRT_2);
        println!("   z_min  = {z_min:.6} bohr   (first rung at or below the fence)");
        println!("   c_lo   = {c_lo:.6}          = z_min / (x_lo * sqrt(2))");
        println!(
            "   u_hi   = {:.9}      = 1 - c_lo^2, the CLOSED end in the emitter's variable",
            1.0 - c_lo * c_lo
        );
        println!(
            "   theta  = {:.4} deg      for scale only; the fence is a SEPARATION, not an angle",
            (1.0f64 - c_lo * c_lo).clamp(-1.0, 1.0).acos().to_degrees()
        );
    } else {
        println!("   NO FENCE FOUND on this ladder: the basis stayed conditioned to");
        println!("   z = {z:.6} bohr. The closed end is then set by the grid, not by the");
        println!("   basis, and that is a different claim which this instrument does not make.");
    }
}
