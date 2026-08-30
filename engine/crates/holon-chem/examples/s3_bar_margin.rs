//! How much margin does `CONVERGED_RESIDUAL` actually leave?
//!
//! The publication bar is 1e-10. The Davidson stops when a Gram-Schmidt'd expansion vector
//! falls below a hardcoded 1e-10 (SATURATION-3 G0's finding: scale-free, dominates every
//! all-electron solve). Those are the SAME NUMBER, so the pass/fail boundary sits exactly
//! where solves stop. This measures the margin per curve to see whether that is a
//! LiH-specific coincidence or a crate-wide tripwire.
//!
//! # Re-run after the ruling (2026-08-30)
//!
//! It was crate-wide, and the answer landed as `179db95`: the edge is named once
//! (`fci::DAVIDSON_EXPANSION_FLOOR`), the requested tolerance equals it by construction,
//! and `pair::CONVERGED_RESIDUAL` is DERIVED a decade above it. Both numbers this file
//! prints therefore come from the constants rather than from a literal, so re-running it
//! is the ruling's own confirmation: the curves that used to stop at 96-100% of the bar
//! should now clear it by an order of magnitude and exit `Converged` rather than
//! `Stagnated`, without any energy having moved. The header above is left as it was
//! written, because it is the reading that produced the ruling.
//!
//! # This table is OPTIMISTIC, and by how much is measured
//!
//! `worst_residual` is a maximum over a curve's knots, so a denser grid can only find a
//! worse one. These curves are 24 knots; the ones the campaign actually ships are 96
//! (`waterquench.rs`) and 192 (`emit_pair_tables.rs`). On the one pair where both reads
//! exist the gap is a factor of eight: O-O reads 1.6353e-5 here and 1.321e-4 on its own
//! 96-knot production grid (`--example s3_oo_reexam`). So a curve passing HERE has not
//! been shown to pass on the grid it ships at, and a lane that needs that answer has to
//! ask at its own knot count.
use holon_chem::elements::by_symbol;
use holon_chem::pair::{generate_pair_table, CONVERGED_RESIDUAL};
fn main() {
    println!("   {:>8} {:>13} {:>13} {:>9} {:>12}", "pair", "worst resid", "bar", "margin", "exit");
    // Two counts, not one. A single "close to the bar" counter read the derived bar's
    // one REFUSAL (O-O, 1.6e5 x the bar) as a thin margin, which is the opposite reading:
    // thin means nearly failed, over means failed. Separated so the line cannot say the
    // wrong one.
    let mut thin = 0usize;
    let mut over = 0usize;
    for (a, b) in [("H","H"),("Li","H"),("H","F"),("F","F"),("He","He"),("Ne","Ne"),("O","H"),("O","O"),("Cl","H")] {
        let t = generate_pair_table(by_symbol(a).unwrap(), by_symbol(b).unwrap(), 24);
        let m = &t.meta;
        let frac = m.worst_residual / CONVERGED_RESIDUAL;
        if frac > 1.0 { over += 1; } else if frac > 0.9 { thin += 1; }
        println!(
            "   {:>8} {:>13.4e} {:>13.0e} {:>8.1}% {:>12}",
            format!("{a}-{b}"), m.worst_residual, CONVERGED_RESIDUAL, 100.0 * frac, m.exit.label()
        );
    }
    println!("\n   {thin} of 9 curves sit within 10% of the bar; {over} of 9 are OVER it.");
}
