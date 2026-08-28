//! Generate the PLACEHOLDER potential file.
//!
//! This exists so the renderer can be built and gated against the real contract before
//! the exact curve arrives. It writes `viewer/h2_potential.json` in exactly the schema
//! the sibling agent's file will use, so swapping the file in is the whole migration:
//! no code changes, no schema translation, no second code path.
//!
//! The curve is a Morse potential fitted to H2's spectroscopic constants. It has the
//! right well depth, the right equilibrium bond length and the right harmonic
//! frequency, and it is WRONG everywhere those three do not pin it — badly so on the
//! repulsive wall, where the true curve rises like the nuclear repulsion 1/R and Morse
//! merely rises exponentially. It is labelled PLACEHOLDER in the file's own
//! `provenance` field, which the viewer reads and shows as a banner.
//!
//! The absolute energy convention is deliberately NOT asymptote-at-zero: two separated
//! H atoms sit at -1.0 hartree here, as they would in an electronic-structure table.
//! That exercises the zeroing path in `table.rs` rather than leaving it untested until
//! the real file arrives carrying its own reference energy.
//!
//! Run: cargo run -p holon-render --example make_placeholder

use std::fmt::Write as _;

/// Spectroscopic constants for H2 (X 1-Sigma-g+).
const D_E: f64 = 0.174490; // hartree; 4.7477 eV
const R_E: f64 = 1.40112; // bohr
const OMEGA_E_CM: f64 = 4401.21; // cm^-1
const CM_TO_HARTREE: f64 = 4.556335e-6;
/// Two ground-state H atoms. The placeholder's own reference; the real table brings its own.
const E_ASYMPTOTE: f64 = -1.0;

const M_H: f64 = 1837.152;

fn main() {
    let mu = 0.5 * M_H;
    let omega = OMEGA_E_CM * CM_TO_HARTREE;
    // Morse: omega = a * sqrt(2 De / mu)  =>  a = omega * sqrt(mu / (2 De)).
    let a = omega * (mu / (2.0 * D_E)).sqrt();

    // Self-check: the harmonic force constant of this Morse is k = 2 a^2 De, and it must
    // reproduce the frequency it was fitted from.
    let k = 2.0 * a * a * D_E;
    let omega_back = (k / mu).sqrt();
    eprintln!("Morse a       = {a:.6} bohr^-1");
    eprintln!("k = 2a^2De    = {k:.6} Eh/bohr^2");
    eprintln!(
        "omega round-trip: {:.4} cm^-1 (target {OMEGA_E_CM})",
        omega_back / CM_TO_HARTREE
    );

    // Non-uniform grid: dense through the wall and the well, coarse in the tail. Real
    // ab initio grids are non-uniform, so the placeholder exercises that path too.
    let mut grid: Vec<f64> = Vec::new();
    let mut r: f64 = 0.40;
    while r <= 3.0001 {
        grid.push((r * 1e6).round() / 1e6);
        r += 0.05;
    }
    r = 3.10;
    while r <= 6.0001 {
        grid.push((r * 1e6).round() / 1e6);
        r += 0.10;
    }
    r = 6.25;
    while r <= 12.0001 {
        grid.push((r * 1e6).round() / 1e6);
        r += 0.25;
    }

    let energy = |r: f64| {
        let x = a * (r - R_E);
        E_ASYMPTOTE + D_E * ((-2.0 * x).exp() - 2.0 * (-x).exp())
    };
    // F = -dE/dR. Positive = repulsive.
    let force = |r: f64| {
        let x = a * (r - R_E);
        2.0 * a * D_E * ((-2.0 * x).exp() - (-x).exp())
    };

    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(
        out,
        "  \"provenance\": \"PLACEHOLDER - Morse potential fitted to H2 spectroscopic constants (De=0.174490 Eh, Re=1.40112 a0, we=4401.21 cm^-1). NOT an ab initio curve; the repulsive wall is exponential where the true curve is nuclear-repulsion dominated. Replace this file with the exact table - no code change is required.\","
    );
    out.push_str("  \"units\": \"Hartree atomic units: R in bohr, E in hartree, F in hartree/bohr. F is the FORCE, so dE/dR = -F.\",\n");
    let _ = writeln!(out, "  \"R_e\": {R_E},");
    let _ = writeln!(out, "  \"D_e\": {D_E},");
    let _ = writeln!(out, "  \"E_asymptote\": {E_ASYMPTOTE},");

    let fmt_array = |name: &str, values: &[f64], last: bool| {
        let mut s = format!("  \"{name}\": [");
        for (i, v) in values.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            let _ = write!(s, "{v:.10e}");
        }
        s.push(']');
        if !last {
            s.push(',');
        }
        s.push('\n');
        s
    };

    let energies: Vec<f64> = grid.iter().map(|&r| energy(r)).collect();
    let forces: Vec<f64> = grid.iter().map(|&r| force(r)).collect();

    out.push_str(&fmt_array("R_grid_bohr", &grid, false));
    out.push_str(&fmt_array("E_hartree", &energies, false));
    out.push_str(&fmt_array("F_hartree_per_bohr", &forces, true));
    out.push_str("}\n");

    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("viewer");
    std::fs::create_dir_all(&dir).expect("create viewer dir");
    let path = dir.join("h2_potential.json");
    std::fs::write(&path, &out).expect("write placeholder");
    eprintln!("knots         = {}", grid.len());
    eprintln!("wrote {} ({} bytes)", path.display(), out.len());
}
