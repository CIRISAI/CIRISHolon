//! E1/E2/F1: the emergent nobles, the halide-hydride column trend, and the relativistic fence.
//!
//! Emits the banked record `tests/data/elements3_dimers.txt` on stdout, and a human table on
//! stderr. `tests/elements3_dimers.rs` reads the record.
//!
//! # Why the heavy curves are banked rather than recomputed by the gate
//!
//! The cost of a curve is dominated by the ERI assembly, which is quartic in the CARTESIAN
//! basis size and is paid once per geometry -- and a curve is about a hundred geometries,
//! because `derive_range` walks and bisects for the wall before a single knot is computed
//! and `locate_well` bisects again for the minimum. Xe2 is 58 Cartesian functions: 11.3
//! million integrals per geometry, an hour or so per curve.
//!
//! A gate that costs an hour is a gate that stops being run, so the expensive curves are
//! banked here with their provenance and the gate reads them. What keeps that honest is
//! that the gate ALSO recomputes the cheapest species live and requires it to reproduce its
//! banked row bit-for-bit: if the engine has moved, the cheap row says so, and the bank is
//! only trusted for the rows it was too expensive to check.
//!
//! Usage:
//! ```text
//! cargo run --release -p holon-chem --example elements3_dimers > \
//!     crates/holon-chem/tests/data/elements3_dimers.txt
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::FciSpace;
use holon_chem::pair::{atom_energy, build_basis, electron_counts, generate_pair_table};

/// Knots per curve. The range is DERIVED by `pair::derive_range` and the knots are PLACED
/// by `table::grid_point`; this is only how many of them there are.
const KNOTS: usize = 24;

/// The staked pairs: E2's three hydrogen halides and Br2, then E1's two noble negatives.
const PAIRS: &[(&str, &str)] = &[
    ("H", "Cl"),
    ("H", "Br"),
    ("H", "I"),
    ("Br", "Br"),
    ("Kr", "Kr"),
    ("Xe", "Xe"),
];

const ATOMS: &[&str] = &["H", "Cl", "Br", "I", "Kr", "Xe"];

fn main() {
    println!("# ELEMENTS-3 dimer record: E1 negatives, E2 ordering, F1 gauge input.");
    println!("# Produced by examples/elements3_dimers.rs. Regenerate rather than edit.");
    println!("# atom: SYM n_basis n_det E_bits");
    println!("# pair: A/B n_basis n_det knots r_min r_max bound r_e_bits d_e_bits k_e_bits route");
    println!("knots {KNOTS}");

    for sym in ATOMS {
        let sp = by_symbol(sym).unwrap();
        let e = atom_energy(sp);
        // The determinant count is COMPUTED from the space the solver actually builds, not
        // written down. Kr and Xe are genuinely one determinant and the rest are not, so a
        // constant here would have been a fabricated column that happened to be right twice.
        let basis = build_basis(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        let (_, na, nb) = electron_counts(&[sp]);
        let n_det = FciSpace::new(basis.n, na, nb).n_det;
        eprintln!("{sym:>3} atom E = {e:.12}  ({n_det} dets)");
        println!("atom {sym} {} {n_det} {:016x}", basis.n, e.to_bits());
    }

    for &(a, b) in PAIRS {
        let (sa, sb) = (by_symbol(a).unwrap(), by_symbol(b).unwrap());
        let t = generate_pair_table(sa, sb, KNOTS);
        let m = &t.meta;
        let (r_min, r_max) = (t.r[0], t.r[t.r.len() - 1]);
        match m.well {
            Some(w) => {
                eprintln!(
                    "{a}{b}: R_e {:.6} bohr  D_e {:.6} Ha  k_e {:.5}  {} dets  {}",
                    w.r_e, w.d_e, w.k_e, m.n_det, m.route.label()
                );
                println!(
                    "pair {a}/{b} {} {} {} {:.12} {:.12} 1 {:016x} {:016x} {:016x} {}",
                    m.n_basis,
                    m.n_det,
                    t.r.len(),
                    r_min,
                    r_max,
                    w.r_e.to_bits(),
                    w.d_e.to_bits(),
                    w.k_e.to_bits(),
                    route_tag(m.route)
                );
            }
            None => {
                eprintln!(
                    "{a}{b}: NO WELL over R = {r_min:.3}..{r_max:.3} bohr, {} dets  {}",
                    m.n_det,
                    m.route.label()
                );
                println!(
                    "pair {a}/{b} {} {} {} {:.12} {:.12} 0 - - - {}",
                    m.n_basis,
                    m.n_det,
                    t.r.len(),
                    r_min,
                    r_max,
                    route_tag(m.route)
                );
            }
        }
    }
}

fn route_tag(r: holon_chem::fci::SolverRoute) -> &'static str {
    match r {
        holon_chem::fci::SolverRoute::Determinant => "determinant",
        holon_chem::fci::SolverRoute::Dmrg => "dmrg",
    }
}
