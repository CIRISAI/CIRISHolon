//! Report, without asserting, how far this build has drifted from W1's frozen baseline.
//!
//! ```text
//! cargo run --release -p holon-chem --example w1_divergence
//! ```
//!
//! # Why this exists rather than a re-bank
//!
//! `tests/data/w1_baseline.txt` is a CONTROL, not a bank. Its entire value is being the
//! snapshot taken BEFORE the u32->u64 mask widening, and re-banking it would convert
//! evidence into wallpaper -- the gate would then prove nothing about W1 ever again. So the
//! W1 gate retires as DISCHARGED (it verified W1's bit-identity while the arithmetic regime
//! it was born under still held), the baseline stays byte-frozen, and the divergence that
//! has since opened is DOCUMENTED beside it rather than erased by regenerating it.
//!
//! This tool produces that documentation. It recomputes every baseline row and reports the
//! ULP gap per species. It asserts NOTHING, on purpose: a report that can fail is a gate,
//! and the ruling was that the gate should stop.
//!
//! # What the numbers mean, and what they do not
//!
//! A non-zero gap here says this build and the pre-widening control disagree. It says
//! NOTHING about which change opened the gap -- the control predates every numeric change
//! since, not only W1. The measured cause of the present divergence is `4884704`, the
//! sigma-kernel summation reorder (ascending `kl` order became first-touch order over a
//! sparse set, which reassociates the addends), bracketed one commit wide by
//! mixtures-engine.

use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::pair::{electron_counts, pair_point, solve_geometry};

fn ulps(got: f64, want_bits: u64) -> i64 {
    (got.to_bits() as i64 - want_bits as i64).abs()
}

fn main() {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/w1_baseline.txt");
    let text = std::fs::read_to_string(&p).expect("baseline readable");

    println!("# W1 baseline divergence, measured against the FROZEN pre-widening control.");
    println!("# A non-zero gap does NOT implicate W1: the control predates every numeric");
    println!("# change since. Measured cause of the present divergence: 4884704.");
    println!("{:<14} {:>10} {:>12} {:>12} {:>12}", "row", "n_det", "ULP(E)", "ULP(dE)", "ULP(d2E)");

    let (mut moved, mut still, mut worst, mut worst_row) = (0usize, 0usize, 0i64, String::new());

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        let hx = |s: &str| u64::from_str_radix(s, 16).expect("hex");

        let (label, n_det, gaps) = match f[0] {
            "atom" => {
                let sp = by_symbol(f[1]).expect("known symbol");
                let s = solve_geometry(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
                (
                    f[1].to_string(),
                    s.n_det,
                    [
                        ulps(s.e.v, hx(f[4])),
                        ulps(s.e.d, hx(f[5])),
                        ulps(s.e.e, hx(f[6])),
                    ],
                )
            }
            "pair" => {
                let (a, b) = f[1].split_once('/').expect("A/B");
                let (sa, sb) = (by_symbol(a).unwrap(), by_symbol(b).unwrap());
                let r: f64 = f[2].parse().unwrap();
                let pt = pair_point(sa, sb, r);
                let (_n, na, nb) = electron_counts(&[sa, sb]);
                let _ = (na, nb);
                (
                    format!("{a}/{b}@{r}"),
                    0,
                    [
                        ulps(pt.e, hx(f[5])),
                        ulps(-pt.f, hx(f[6])),
                        ulps(pt.e2, hx(f[7])),
                    ],
                )
            }
            other => panic!("unknown row kind {other:?}"),
        };

        let any = gaps.iter().any(|g| *g != 0);
        if any {
            moved += 1;
        } else {
            still += 1;
        }
        for g in gaps {
            if g > worst {
                worst = g;
                worst_row = label.clone();
            }
        }
        println!(
            "{:<14} {:>10} {:>12} {:>12} {:>12}{}",
            label, n_det, gaps[0], gaps[1], gaps[2],
            if any { "   MOVED" } else { "" }
        );
    }

    println!("#");
    println!("# {moved} rows moved, {still} unchanged. Worst gap {worst} ULP, on {worst_row}.");
    println!(
        "# Movement is NOT monotone in determinant count: this lane's dimer record has \
         chlorine (9 determinants) moving while bromine (18) does not, both one-hole \
         systems of the same shape. Enumerate moved/not-moved by measurement, never by a \
         size rule."
    );
}
