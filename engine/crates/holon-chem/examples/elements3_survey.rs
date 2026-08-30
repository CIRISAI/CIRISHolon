//! Basis size and determinant count for every element the registry now carries.
//!
//! No energies: this reports the SHAPE of each atom's problem, which is what decides the
//! route it can be solved by, and it is the first thing ELEMENTS-3's R1 needs to know.

use holon_chem::elements::{by_symbol, ALL_ELEMENTS};
use holon_chem::pair::{build_basis, electron_counts};
use holon_chem::dual::D2;
use holon_chem::fci::FciSpace;

fn main() {
    println!("{:>3} {:>3} {:>5} {:>7} {:>20}", "Z", "sym", "nbas", "na/nb", "n_det");
    for sp in ALL_ELEMENTS {
        let basis = build_basis(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]);
        let (_, na, nb) = electron_counts(&[sp]);
        let space = FciSpace::new(basis.n, na, nb);
        println!(
            "{:>3} {:>3} {:>5} {:>7} {:>20}",
            sp.z,
            sp.symbol,
            basis.n,
            format!("{na}/{nb}"),
            space.n_det
        );
    }

    println!();
    println!("{:>8} {:>5} {:>7} {:>7} {:>20}", "pair", "nbas", "nelec", "na/nb", "n_det");
    for (a, b) in [
        ("H", "Cl"), ("H", "Br"), ("H", "I"), ("Br", "Br"),
        ("Kr", "Kr"), ("Xe", "Xe"),
    ] {
        let (sa, sb) = (by_symbol(a).unwrap(), by_symbol(b).unwrap());
        let basis = build_basis(
            &[sa, sb],
            vec![
                [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                [D2::c(0.0), D2::c(0.0), D2::var(3.0)],
            ],
        );
        let (n_elec, na, nb) = electron_counts(&[sa, sb]);
        let space = FciSpace::new(basis.n, na, nb);
        println!(
            "{:>8} {:>5} {:>7} {:>7} {:>20}",
            format!("{a}{b}"), basis.n, n_elec, format!("{na}/{nb}"), space.n_det
        );
    }
}
