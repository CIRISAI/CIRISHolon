//! CRYO-H-O ARM 1 — POST-DATA CONTROL, added after the G1 sweep was read.
//!
//! **Labelled POST-DATA and it changes no staked criterion.** The G1 sweep returned an
//! exact interaction at 12 bohr of +4.3e-7 Ha in the H orientation while the pair and
//! triple terms there are 6e-11 and 1.2e-10 — four orders smaller. Two things produce
//! that shape and they are opposite findings:
//!
//! * a genuine long-range ELECTROSTATIC tail (H2 has a permanent quadrupole, and a
//!   minimal basis carries one), which would fall as `R^-5` and would be attractive in
//!   T, repulsive in H and L — the classical quadrupole–quadrupole angular pattern;
//! * an instrument FLOOR — the difference of two ~2.27 Ha numbers not resolving below
//!   ~4e-7 — in which case G1's deepest reading (-5.6e-6 Ha) sits only 13x above noise
//!   and the sign census means nothing.
//!
//! This probe separates them the only way that settles it: keep going out. A tail falls;
//! a floor does not. `CONTROL_R` is far enough that no interaction of any kind survives
//! there, so whatever it reads IS the floor.
//!
//! ```text
//! cargo run --release -p holon-chem --example cryo_h2_tail
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::HYDROGEN;
use holon_chem::pair::solve_geometry;

const R_E: f64 = 1.388_694_018_017_776_3;

/// The tail, decade-spanning so a power law has room to show a straight line.
const TAIL: [f64; 9] = [8.0, 10.0, 12.0, 14.0, 16.0, 20.0, 24.0, 30.0, 40.0];
/// Far enough that every mechanism named above is dead: the pair term at 60 bohr is
/// `exp(-2.46 * 60)` and a quadrupole tail is `60^-5`. Anything read here is the
/// instrument.
const CONTROL_R: [f64; 2] = [60.0, 100.0];

fn e(pos: &[[f64; 3]]) -> f64 {
    let sp = vec![HYDROGEN; pos.len()];
    let c: Vec<[D2; 3]> = pos.iter().map(|p| [D2::c(p[0]), D2::c(p[1]), D2::c(p[2])]).collect();
    solve_geometry(&sp, c).e.v
}

fn scene(orient: char, r: f64) -> [[f64; 3]; 4] {
    let h = 0.5 * R_E;
    match orient {
        'H' => [[-h, 0.0, 0.0], [h, 0.0, 0.0], [-h, r, 0.0], [h, r, 0.0]],
        'T' => [[-h, 0.0, 0.0], [h, 0.0, 0.0], [0.0, r - h, 0.0], [0.0, r + h, 0.0]],
        _ => [[-h, 0.0, 0.0], [h, 0.0, 0.0], [r - h, 0.0, 0.0], [r + h, 0.0, 0.0]],
    }
}

fn main() {
    println!("# CRYO-H-O ARM 1 — tail probe and floor control  [POST-DATA, changes no stake]");
    let e_h2 = e(&[[-0.5 * R_E, 0.0, 0.0], [0.5 * R_E, 0.0, 0.0]]);
    println!("# E(H2) = {e_h2:.15} Ha");
    println!("#\n#     R        E_int(H)        E_int(T)        E_int(L)      slope(H) slope(T) slope(L)");

    let mut prev: Option<(f64, [f64; 3])> = None;
    for &r in TAIL.iter() {
        let v: [f64; 3] = ['H', 'T', 'L'].map(|o| e(&scene(o, r)) - 2.0 * e_h2);
        let sl: [f64; 3] = match prev {
            Some((r0, v0)) => core::array::from_fn(|k| {
                ((v[k] / v0[k]).abs()).ln() / (r / r0).ln()
            }),
            None => [f64::NAN; 3],
        };
        println!(
            "  {:6.1}  {:+.8e}  {:+.8e}  {:+.8e}   {:+7.3} {:+7.3} {:+7.3}",
            r, v[0], v[1], v[2], sl[0], sl[1], sl[2]
        );
        prev = Some((r, v));
    }

    println!("#\n# ---- THE FLOOR CONTROL: nothing can interact here ----");
    for &r in CONTROL_R.iter() {
        let v: [f64; 3] = ['H', 'T', 'L'].map(|o| e(&scene(o, r)) - 2.0 * e_h2);
        println!(
            "  {:6.1}  {:+.8e}  {:+.8e}  {:+.8e}   <- floor",
            r, v[0], v[1], v[2]
        );
    }
    println!("#\n# ---- the T well, refined on a 0.25 bohr grid  [POST-DATA] ----");
    let mut best = (f64::NAN, f64::INFINITY);
    let mut r = 6.0f64;
    while r <= 11.0 + 1e-9 {
        let v = e(&scene('T', r)) - 2.0 * e_h2;
        println!("  {:6.2}  {:+.8e}", r, v);
        if v < best.1 {
            best = (r, v);
        }
        r += 0.25;
    }
    println!(
        "# T minimum on the 0.25 grid: {:+.8e} Ha at R = {:.2} bohr  ({:.3} K)",
        best.1, best.0, best.1 * 315_775.0
    );

    println!("#");
    println!("# READ: if the floor rows are orders BELOW the 8-12 bohr rows, the tail is real");
    println!("#       and the slopes above are its exponent. If they are the same size, the");
    println!("#       tail IS the floor and G1's sign census is noise.");
}
