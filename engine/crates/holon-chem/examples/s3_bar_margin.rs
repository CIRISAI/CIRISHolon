//! How much margin does `CONVERGED_RESIDUAL` actually leave?
//!
//! The publication bar is 1e-10. The Davidson stops when a Gram-Schmidt'd expansion vector
//! falls below a hardcoded 1e-10 (SATURATION-3 G0's finding: scale-free, dominates every
//! all-electron solve). Those are the SAME NUMBER, so the pass/fail boundary sits exactly
//! where solves stop. This measures the margin per curve to see whether that is a
//! LiH-specific coincidence or a crate-wide tripwire.
use holon_chem::elements::by_symbol;
use holon_chem::pair::{generate_pair_table, CONVERGED_RESIDUAL};
fn main() {
    println!("   {:>8} {:>13} {:>13} {:>9} {:>12}", "pair", "worst resid", "bar", "margin", "exit");
    let mut thin = 0usize;
    for (a, b) in [("H","H"),("Li","H"),("H","F"),("F","F"),("He","He"),("Ne","Ne"),("O","H"),("O","O"),("Cl","H")] {
        let t = generate_pair_table(by_symbol(a).unwrap(), by_symbol(b).unwrap(), 24);
        let m = &t.meta;
        let frac = m.worst_residual / CONVERGED_RESIDUAL;
        if frac > 0.9 { thin += 1; }
        println!(
            "   {:>8} {:>13.4e} {:>13.0e} {:>8.1}% {:>12}",
            format!("{a}-{b}"), m.worst_residual, CONVERGED_RESIDUAL, 100.0 * frac, m.exit.label()
        );
    }
    println!("\n   {thin} of 9 curves sit within 10% of the bar.");
}
