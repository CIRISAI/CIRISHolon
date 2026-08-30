//! How many curves were shipping as converged with a knot that gave up?
//!
//! `PairMeta::converged` tested the residual alone against 1e-10. A Davidson solve that
//! STAGNATES between the 1e-11 tolerance it was asked for and that 1e-10 bar passed. This
//! counts, per pair, how many knots exit each way — so the scope of the change is measured
//! before anything is decided about it.
use holon_chem::elements::by_symbol;
use holon_chem::fci::SolveExit;
use holon_chem::pair::generate_pair_table;
use std::io::Write;

fn main() {
    let names = ["H2", "HHe", "He2", "HLi", "Li2", "HCl", "ClF", "Cl2", "N2"];
    println!("pair\tknots\texit\tworst_resid\tconverged()\tE_bits\tE_value");
    let _ = std::io::stdout().flush();
    for n in names {
        let (a, b) = if let Some(stem) = n.strip_suffix('2') {
            let sp = by_symbol(stem).unwrap();
            (sp, sp)
        } else {
            let at = n.char_indices().skip(1).find(|(_, c)| c.is_uppercase()).map(|(i, _)| i).unwrap();
            (by_symbol(&n[..at]).unwrap(), by_symbol(&n[at..]).unwrap())
        };
        let pt = generate_pair_table(a, b, 24);
        // Every knot's energy as a RAW BIT PATTERN, folded into one digest, plus the
        // last knot's value. A tolerance change that alters WHEN Davidson stops alters the
        // energy it stops at, and only bits can say whether it did.
        let mut digest: u64 = 0xcbf2_9ce4_8422_2325;
        for e in pt.e.iter() {
            for b in e.to_bits().to_le_bytes() {
                digest ^= b as u64;
                digest = digest.wrapping_mul(0x1000_0000_01b3);
            }
        }
        println!(
            "{n}\t{}\t{}\t{:.4e}\t{}\t{digest:016x}\t{:.15}",
            pt.r.len(),
            pt.meta.exit.label(),
            pt.meta.worst_residual,
            pt.meta.converged(),
            pt.e.last().copied().unwrap_or(f64::NAN)
        );
        let _ = std::io::stdout().flush();
        let _ = SolveExit::Converged;
    }
}
