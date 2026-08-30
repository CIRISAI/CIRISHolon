//! F1 branch (b): is the model's error against nature relativity, or is it the basis?
//!
//! # Why this exists
//!
//! F1 stakes that the in-model deficit against experimental `D_e` GROWS down the halide
//! column, on the reasoning that relativity and core correlation are absent and are missed
//! more as the halogen gets heavier. Measured, the signed deficit does the opposite: it
//! falls, and changes sign. The prereg's answer to that is branch (b) -- investigate, do
//! not massage -- and this is the investigation.
//!
//! # The competing cause, and how to separate it
//!
//! A dissociation energy computed as `E(A) + E(B) - E(AB)` in a FINITE basis is
//! contaminated by basis-set superposition error: at the equilibrium separation each atom
//! can borrow its partner's basis functions, so the molecule is described in a larger
//! effective basis than the atoms are and the well comes out too deep. The borrowing scales
//! with how many functions the partner brings -- chlorine offers nine, iodine
//! twenty-seven -- so BSSE grows down exactly the column F1 walks, and it pushes the
//! deficit the OPPOSITE way from the missing relativity.
//!
//! The counterpoise construction (S. F. Boys and F. Bernardi, *Mol. Phys.* **19**, 553
//! (1970)) separates them: recompute each atom in the FULL dimer basis, with the partner's
//! nuclear charge set to zero so its functions are present and its nucleus is not. The
//! difference between the raw and corrected well depths IS the superposition error.
//!
//! # What this is not
//!
//! A DIAGNOSIS, not a correction. Nothing here changes a staked number and nothing here
//! rescues F1 -- the stake as written is dead whatever this says. What it can establish is
//! whether the stake died because its PHYSICS was wrong or because its OBSERVABLE was
//! contaminated, and those are different findings that deserve different words.
//!
//! Equilibrium separations are read from the banked record rather than recomputed, so this
//! diagnoses the same geometries the record reports, at three single-point solves per pair
//! instead of a hundred.

use holon_chem::dual::D2;
use holon_chem::elements::{by_symbol, Species};
use holon_chem::md::Basis;
use holon_chem::pair::{atom_energy, solve_basis};

const HARTREE_PER_EV: f64 = 1.0 / 27.211386245988;

/// Huber and Herzberg (1979), `D_e` in eV. LABELLED CONTEXT, never an input to a solve.
const EXPERIMENTAL_D_E: [(&str, &str, f64); 3] =
    [("H", "Cl", 4.618), ("H", "Br", 3.922), ("H", "I", 3.198)];

/// The dimer basis, with the charges chosen by the caller.
///
/// `ghost` names the centre that keeps its basis functions and loses its nucleus. `None` is
/// the real dimer. A ghost centre contributes no nuclear attraction and no nuclear
/// repulsion, so `solve_basis` returns the atom's own energy computed in the larger basis --
/// which is the whole construction.
fn dimer_basis(a: Species, b: Species, r: f64, ghost: Option<usize>) -> Basis {
    let centers = vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
        [D2::c(0.0), D2::c(0.0), D2::var(r)],
    ];
    let mut decls = Vec::new();
    for (c, sp) in [a, b].iter().enumerate() {
        for sh in sp.shells {
            decls.push((c, sh.kind.l(), sh.alpha, sh.coeff));
        }
    }
    let charges = vec![
        if ghost == Some(0) { 0.0 } else { a.z as f64 },
        if ghost == Some(1) { 0.0 } else { b.z as f64 },
    ];
    Basis::assemble(centers, charges, &decls)
}

/// Alpha and beta counts for an electron total, in the minimal `S_z` sector.
fn split(n: usize) -> (usize, usize) {
    ((n + n % 2) / 2, (n - n % 2) / 2)
}

/// Equilibrium separations, read from the banked record by pair key.
fn banked_r_e(key: &str) -> f64 {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/elements3_dimers.txt");
    let text = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("cannot read {}: {e} -- run elements3_dimers first", p.display()));
    for line in text.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() > 8 && f[0] == "pair" && f[1] == key && f[7] == "1" {
            return f64::from_bits(u64::from_str_radix(f[8], 16).expect("hex"));
        }
    }
    panic!("the banked record has no bound pair {key}");
}

fn main() {
    println!(
        "{:>6} {:>10} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "pair", "R_e", "D_e raw", "D_e cp", "BSSE", "deficit raw", "deficit cp"
    );
    let (mut raw, mut cp) = (Vec::new(), Vec::new());

    for (sa, sb, ev) in EXPERIMENTAL_D_E {
        let (a, b) = (by_symbol(sa).unwrap(), by_symbol(sb).unwrap());
        let r_e = banked_r_e(&format!("{sa}/{sb}"));

        let (na, nb) = split(a.z as usize + b.z as usize);
        let e_ab = solve_basis(&dimer_basis(a, b, r_e, None), na, nb).e.v;
        let (aa, ab) = split(a.z as usize);
        let e_a_ghost = solve_basis(&dimer_basis(a, b, r_e, Some(1)), aa, ab).e.v;
        let (ba, bb) = split(b.z as usize);
        let e_b_ghost = solve_basis(&dimer_basis(a, b, r_e, Some(0)), ba, bb).e.v;

        let d_raw = atom_energy(a) + atom_energy(b) - e_ab;
        let d_cp = e_a_ghost + e_b_ghost - e_ab;
        let bsse = d_raw - d_cp;
        let exp = ev * HARTREE_PER_EV;

        println!(
            "{:>6} {r_e:>10.5} {d_raw:>12.6} {d_cp:>12.6} {bsse:>12.6} {:>+12.6} {:>+12.6}",
            format!("{sa}{sb}"),
            exp - d_raw,
            exp - d_cp
        );
        raw.push(exp - d_raw);
        cp.push(exp - d_cp);
    }

    let grows = |g: &[f64]| g.windows(2).all(|p| p[1] > p[0]);
    println!();
    println!(
        "raw deficit grows down the column:          {:5}   ({:+.6} -> {:+.6} -> {:+.6})",
        grows(&raw),
        raw[0],
        raw[1],
        raw[2]
    );
    println!(
        "counterpoise deficit grows down the column: {:5}   ({:+.6} -> {:+.6} -> {:+.6})",
        grows(&cp),
        cp[0],
        cp[1],
        cp[2]
    );
    println!();
    println!(
        "Reading: if the raw deficit falls and the counterpoise one grows, F1's stake died"
    );
    println!(
        "of a contaminated observable rather than of wrong physics -- a different finding,"
    );
    println!("and one that does NOT inherit the dead stake's standing.");
}
