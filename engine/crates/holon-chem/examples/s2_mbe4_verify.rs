//! The four-body reference set: `dE4_true = E_FCI - E_MBE3` at gate G2's forty staked
//! geometries, and the far-field shells beyond them.
//!
//! # What this was, and what it is now
//!
//! This file was written to VERIFY `src/quaternary.rs`, an analytic four-body surface. It
//! did not verify — wrong sign at 11 of 40 geometries, worst residual 0.2755 Ha against a
//! mean `|dE4_true|` of 0.1119 Ha and T1's interpolation scale of 2.47e-4 — and that
//! module has since been removed from the crate. The numbers, the credit and the reasoning
//! are in `conformance/atomworld/SATURATION2_RESULTS.md`; the run that produced them is in
//! git history.
//!
//! What remains here is the half that was never about that module: the exact reference a
//! four-body surface has to be built against. It is kept rather than deleted because it is
//! the successor's instrument, already sized —
//!
//! * `E_MBE3` assembled for a FOUR-atom system: six pair terms, three `(O,H,H)` triples
//!   and one `(H,H,H)`, which is the arithmetic a four-body term is defined as the
//!   remainder of and the easiest thing in this campaign to get subtly wrong;
//! * `E_FCI` for `(O,H,H,H)` — 1568 determinants in the minimal-|Sz| sector;
//! * their difference at the SAME forty geometries gate G2 scores, so a candidate surface
//!   is graded against referee numbers already in the record rather than against a set
//!   chosen after the fact;
//! * and the far field, which is where this lane's own expectation was wrong.
//!
//! # The sign convention, stated because getting it wrong cost a lane a day
//!
//! ```text
//! binding(X)  = [E(H2O) + E(H)] - E(X)        positive when bound
//! dE4_energy  = E_FCI - E_MBE3                the sign a four-body TERM is ADDED with
//! dE4_binding = E_MBE3 - E_FCI                the same number, negated
//! ```
//!
//! `dE4_energy` is used throughout and is what part 1 prints from a single evaluation.
//! This lane once published `-0.183` without naming which of the two it was, and a
//! downstream lane read it backwards.
//!
//! # The far-field finding, which survives as a real input to the successor's domain
//!
//! This lane expected the four-body term to need a domain like its three-body one, whose
//! tail is ALGEBRAIC (`R^-5`, quadrupolar) and forced `R_HI = 15` bohr. **It does not.**
//! Measured in part 3: `dE4_true` is 7.8e-5 Ha at 5.9 bohr, 4.9e-5 at 6.1 and 1.7e-6 by 9.
//! A six-bohr cut on the four-body term costs about 5e-5 Ha, inside T1's own scale. The
//! four-body term is genuinely shorter-ranged than the three-body one, and the `R_CUT =
//! 6.0` bohr the removed module chose was defensible — that much of it was right, and the
//! successor should start from it rather than from this lane's 15.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_mbe4_verify
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};
use holon_chem::trimer;
use holon_chem::water;

const PI: f64 = std::f64::consts::PI;
const R_W: f64 = 1.9435740105;
const TH_W: f64 = 96.75788837;

fn c3(p: [f64; 3]) -> [D2; 3] {
    [D2::c(p[0]), D2::c(p[1]), D2::c(p[2])]
}
fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn main() {
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/s2/s2_water_table.txt"),
    )
    .expect("the committed (O,H,H) table");
    let w = water::from_text(&src).expect("it parses");
    let h3 = trimer::generate().expect("the H3 table");

    let th = TH_W * PI / 180.0;
    let o = [0.0f64, 0.0, 0.0];
    let h1 = [R_W * (th / 2.0).cos(), R_W * (th / 2.0).sin(), 0.0];
    let h2 = [R_W * (th / 2.0).cos(), -R_W * (th / 2.0).sin(), 0.0];
    let d_h1h2 = dist(h1, h2);
    // E_MBE3 of the four-atom system: six pair terms, four triples, no four-body term.
    let mbe3 = |p: [f64; 3]| -> f64 {
        let (a, b, c) = (dist(o, p), dist(h1, p), dist(h2, p));
        let pairs = (pair_point(OXYGEN, HYDROGEN, R_W).e - e_o - e_h) * 2.0
            + (pair_point(OXYGEN, HYDROGEN, a).e - e_o - e_h)
            + (pair_point(HYDROGEN, HYDROGEN, d_h1h2).e - 2.0 * e_h)
            + (pair_point(HYDROGEN, HYDROGEN, b).e - 2.0 * e_h)
            + (pair_point(HYDROGEN, HYDROGEN, c).e - 2.0 * e_h);
        let triples = w.eval(R_W, R_W, d_h1h2).0
            + w.eval(R_W, a, b).0
            + w.eval(R_W, a, c).0
            + h3.eval([d_h1h2, b, c]).0;
        e_o + 3.0 * e_h + pairs + triples
    };
    let fci = |p: [f64; 3]| -> f64 {
        solve_geometry(
            &[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN],
            vec![c3(o), c3(h1), c3(h2), c3(p)],
        )
        .e
        .v
    };

    // ---------------------------------------------------------------- 1. the sign
    println!("## 1 — the sign, from one computation");
    let probe = [-2.25f64, 0.0, 0.0];
    let (ef, em) = (fci(probe), mbe3(probe));
    println!("   at O-H3 = 2.250 bohr on the C2 axis:");
    println!("     E_FCI                 = {ef:.9} Ha");
    println!("     E_MBE3                = {em:.9} Ha");
    println!("     dE4_energy  = FCI-MBE3 = {:+.6} Ha   <- the convention a term is ADDED with", ef - em);
    // binding(X) = [E(H2O) + E(H)] - E(X), so a difference of BINDINGS is
    // [.. - E_FCI] - [.. - E_MBE3] = E_MBE3 - E_FCI: the same magnitude, opposite sign.
    println!(
        "     dE4_binding = MBE3-FCI = {:+.6} Ha   <- this lane's earlier wording",
        em - ef
    );
    println!(
        "     the two sum to {:.1e}, so they differ by a sign and nothing else",
        ((ef - em) + (em - ef)).abs()
    );
    println!("   dE4_energy is the convention used from here on: it is the sign a four-body");
    println!("   term is ADDED with, so a surface printing this number needs no translation.");

    // ------------------------------------------- 2. does it reproduce full CI?
    println!("\n## 2 — the reference set: dE4_true at gate G2's own forty staked geometries");
    let dirs: [[f64; 3]; 8] = [
        [-1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [-0.7071, 0.0, 0.7071],
        [-0.7071, 0.0, -0.7071],
        [0.5774, 0.5774, 0.5774],
    ];
    let radii = [1.4f64, 1.8, 2.2, 2.8, 3.6];
    println!(
        "   {:>4} {:>5} {:>14} {:>14} {:>14}",
        "dir", "r", "E_FCI", "E_MBE3", "dE4_true"
    );
    // A candidate surface has to reproduce the SIGN as well as the magnitude, and the sign
    // is not constant: this counts the geometries where the true term is attractive, which
    // is the property the removed module could not represent.
    let (mut n, mut n_attractive) = (0usize, 0usize);
    let (mut sum_true, mut max_true) = (0.0f64, 0.0f64);
    for (di, d) in dirs.iter().enumerate() {
        let nn = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        for &r in &radii {
            let p = [r * d[0] / nn, r * d[1] / nn, r * d[2] / nn];
            if dist(o, p) < 0.9 || dist(h1, p) < 0.9 || dist(h2, p) < 0.9 {
                continue;
            }
            let (ef, em) = (fci(p), mbe3(p));
            let t = ef - em;
            n += 1;
            sum_true += t.abs();
            max_true = max_true.max(t.abs());
            if t < 0.0 {
                n_attractive += 1;
            }
            println!("   {di:>4} {r:>5.1} {ef:>14.6} {em:>14.6} {t:>14.6}");
        }
    }
    println!(
        "\n   over {n} geometries: mean |dE4_true| = {:.6} Ha, max {max_true:.6} Ha",
        sum_true / n as f64
    );
    println!(
        "   the true term is ATTRACTIVE at {n_attractive} of {n} — it CHANGES SIGN with\n            geometry, which is the property that ended the analytic candidate and which any\n            successor has to carry."
    );
    println!(
        "   the bar: T1's interpolation scale is 2.47e-4 Ha, so a four-body surface that\n            worked would land the residual against this column there."
    );

    // ---------------------------------------------------------------- 3. the far field
    println!("\n## 3 — the far field, and why a six-bohr cut is defensible here");
    println!("   {:>6} {:>14}", "O-H3", "dE4_true");
    for r in [4.0f64, 5.0, 5.9, 6.1, 7.0, 9.0, 12.0] {
        let p = [-r, 0.0, 0.0];
        let (ef, em) = (fci(p), mbe3(p));
        println!("   {r:>6.1} {:>14.3e}", ef - em);
    }
    println!(
        "\n   This lane's THREE-body table needed R_HI = 15 bohr for an algebraic R^-5\n            tail. That does NOT transfer: the four-body term is 4.9e-5 Ha just past 6 bohr\n            and 1.7e-6 by 9, so a six-bohr cut costs about 5e-5 Ha — inside T1's own scale."
    );
}
