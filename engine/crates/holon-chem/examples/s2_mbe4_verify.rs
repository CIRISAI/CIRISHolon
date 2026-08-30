//! VERIFICATION of `src/quaternary.rs`'s four-body surface against this lane's full-CI
//! referee numbers.
//!
//! Assigned because G2 is this lane's gate and the (O, H, H, H) full CI is this lane's
//! measurement. Three questions, in the order they have to be asked.
//!
//! # 1. The sign
//!
//! `quaternary.rs` carries `G2_DEFICIT = +0.183` and this lane's record says `-0.183`.
//! That is a CONVENTION difference and not an error, and the check is one line of algebra
//! made explicit here rather than asserted:
//!
//! ```text
//! binding(X)  = [E(H2O) + E(H)] - E(X)          positive when bound
//! dE4_binding = binding_FCI - binding_MBE3      = -0.183   (this lane's earlier wording)
//! dE4_energy  = E_FCI - E_MBE3                  = +0.183   (quaternary.rs's constant)
//! ```
//!
//! Both are printed below from one computation so the identity is visible rather than
//! argued. What is wrong is this lane's own earlier wording, which stated a number in a
//! convention it did not name.
//!
//! # 2. Does it reproduce full CI, or only change the sign?
//!
//! The assignment's wording: the artifact must flip sign AND land within a stated
//! tolerance of full CI, not merely become repulsive. So the residual measured here is
//!
//! ```text
//! residual = [E_MBE3 + dE4_quaternary] - E_FCI
//! ```
//!
//! over the SAME staked geometries gate G2 scores — eight directions by five radii around
//! relaxed water — so the comparison is against the referee numbers already in the record.
//! A four-body term that worked would drive this to the interpolation scale T1 measures,
//! about 1e-4 Ha. `dE4_true = E_FCI - E_MBE3` is printed beside it, because the ratio of
//! the two is what says whether the form is right or only the magnitude at one point.
//!
//! # 3. The far field
//!
//! `R_CUT_4BODY = 6.0` bohr with a switch from 3.5. This lane's three-body table needed
//! `R_HI = 15` because the tail is ALGEBRAIC (`R^-5`, quadrupolar) rather than
//! exponential, and a truncation is only as good as the worst thing it zeroes. So the true
//! `dE4` is measured on the shells their cut discards.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_mbe4_verify
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point, solve_geometry};
use holon_chem::quaternary::de4_ohhh_cart;
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
    let e_water_fci = solve_geometry(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![c3(o), c3(h1), c3(h2)],
    )
    .e
    .v;

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
    println!("     dE4_energy  = FCI-MBE3 = {:+.6} Ha   <- quaternary.rs's convention", ef - em);
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
    println!("     quaternary.rs G2_DEFICIT = {:+.6}", holon_chem::quaternary::G2_DEFICIT);
    println!("   VERDICT: convention, not error. The lane's own doc is what needs the fix.");

    // ------------------------------------------- 2. does it reproduce full CI?
    println!("\n## 2 — MBE3 + their dE4 against full CI, on gate G2's own staked geometries");
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
        "   {:>4} {:>5} {:>13} {:>13} {:>13} {:>13}",
        "dir", "r", "dE4 true", "dE4 theirs", "residual", "|res|/|true|"
    );
    let (mut worst, mut n, mut n_signflip, mut n_overshoot) = (0.0f64, 0usize, 0usize, 0usize);
    let mut sum_true = 0.0f64;
    for (di, d) in dirs.iter().enumerate() {
        let nn = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        for &r in &radii {
            let p = [r * d[0] / nn, r * d[1] / nn, r * d[2] / nn];
            if dist(o, p) < 0.9 || dist(h1, p) < 0.9 || dist(h2, p) < 0.9 {
                continue;
            }
            let (ef, em) = (fci(p), mbe3(p));
            let t = ef - em;
            let g = de4_ohhh_cart(o, h1, h2, p);
            let res = (em + g) - ef;
            n += 1;
            sum_true += t.abs();
            if t * g < 0.0 {
                n_signflip += 1;
            }
            if g.abs() > 2.0 * t.abs() && t.abs() > 1e-6 {
                n_overshoot += 1;
            }
            if res.abs() > worst {
                worst = res.abs();
            }
            println!(
                "   {di:>4} {r:>5.1} {t:>13.6} {g:>13.6} {res:>13.6} {:>13.3}",
                if t.abs() > 1e-9 { res.abs() / t.abs() } else { f64::NAN }
            );
        }
    }
    println!(
        "\n   over {n} geometries: worst |residual| = {worst:.6} Ha, mean |dE4_true| = {:.6} Ha",
        sum_true / n as f64
    );
    println!(
        "   their term has the WRONG SIGN at {n_signflip} of {n}, and overshoots by more \
         than 2x at {n_overshoot}"
    );
    println!(
        "   T1's interpolation scale, for reference: 2.47e-4 Ha. A four-body term that \
         worked\n   would drive the residual there."
    );

    // ---------------------------------------------------------------- 3. the far field
    println!("\n## 3 — the far field their R_CUT_4BODY = 6.0 discards");
    println!("   {:>6} {:>14} {:>14}", "O-H3", "dE4 true", "dE4 theirs");
    for r in [4.0f64, 5.0, 5.9, 6.1, 7.0, 9.0, 12.0] {
        let p = [-r, 0.0, 0.0];
        let (ef, em) = (fci(p), mbe3(p));
        let g = de4_ohhh_cart(o, h1, h2, p);
        println!("   {r:>6.1} {:>14.3e} {:>14.3e}", ef - em, g);
    }
}
