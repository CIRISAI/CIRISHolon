//! Example running the SELECTOR-5 landscape sweep in Rust.

use holon::selector::*;

fn main() {
    println!("================================================================================");
    println!("SELECTOR-5 LANDSCAPE SWEEP (RUST ENGINE)");
    println!("================================================================================");

    let mut groups = Vec::new();

    // Cyclic
    for n in 1..=64 {
        groups.push(build_cyclic(n));
    }
    // Dihedral
    for n in 2..=32 {
        groups.push(build_dihedral(n));
    }
    // Dicyclic
    for n in 2..=16 {
        groups.push(build_dicyclic(n));
    }
    // 2T
    groups.push(build_binary_tetrahedral());

    // Frobenius
    for &p in &[3, 5, 7, 11, 13] {
        for k in 2..p {
            if let Some(f) = build_frobenius(p, k) {
                groups.push(f);
            }
        }
    }

    // Delta3n2
    for &n in &[1, 2, 3] {
        if let Some(d) = build_delta_3n2(n) {
            groups.push(d);
        }
    }

    println!("Total Groups Enumerated: {}", groups.len());
    println!();
    println!("| Order | Group Name | Family | Comm Defect | Orient Index | Full Pass? |");
    println!("|---|---|---|---|---|---|");

    let mut passers = 0;
    for g in &groups {
        let pass = g.full_selector_pass();
        if pass {
            passers += 1;
        }
        if pass || g.order <= 24 {
            println!(
                "| {:5} | {:12} | {:16} | {:11.4} | {:12.4} | {:10} |",
                g.order, g.name, g.family, g.commutator_defect(), g.orientation_index(), if pass { "YES" } else { "no" }
            );
        }
    }

    println!();
    println!("Total Passers: {} / {} ({:.2}%)", passers, groups.len(), (passers as f64 / groups.len() as f64) * 100.0);
}
