//! What the EXACT side of gates D1 and R2 costs, on the species `solve` routes away.
//!
//! ```text
//! cargo run --release -p holon-chem --example fci_cost -- [PAIR ...]
//! ```
//!
//! The lead's correction to the referee brief matters here: D1's FCI side is engine f64 at
//! a 1e-8 comparison, NOT the 50-digit referee. So "can we have exact SiO" is a question
//! about this crate's Davidson at 132,496 determinants, not about thirty mpmath hours.
//!
//! One geometry per pair, on `solve_determinant` explicitly — `pair_point` would go through
//! `fci::solve` and route SiO to a DMRG builder that does not return.
use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::solve_determinant;
use holon_chem::pair::{automatic_route, geometry_problem};
use std::io::Write;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let names: Vec<String> = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        ["HCl", "ClF", "Cl2", "NaH", "S2", "SiO"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    };
    println!("pair\tn_orb\tn_det\tR\tE_hartree\tresidual\tassemble_s\tsolve_s");
    let _ = std::io::stdout().flush();
    for name in names {
        let (a, b) = if let Some(stem) = name.strip_suffix('2') {
            let sp = by_symbol(stem).unwrap();
            (sp, sp)
        } else {
            let at = name
                .char_indices()
                .skip(1)
                .find(|(_, c)| c.is_uppercase())
                .map(|(i, _)| i)
                .unwrap();
            (by_symbol(&name[..at]).unwrap(), by_symbol(&name[at..]).unwrap())
        };
        let f = automatic_route(a, b);
        let r = 3.0;
        let t0 = Instant::now();
        let (space, mo, nuc) = geometry_problem(
            &[a, b],
            vec![
                [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                [D2::c(0.0), D2::c(0.0), D2::c(r)],
            ],
        );
        let asm = t0.elapsed().as_secs_f64();
        let t1 = Instant::now();
        let sol = solve_determinant(&space, &mo);
        println!(
            "{name}\t{}\t{}\t{r}\t{:.12}\t{:.3e}\t{asm:.1}\t{:.1}",
            f.n_orb(),
            space.n_det,
            (sol.e + nuc).v,
            sol.residual,
            t1.elapsed().as_secs_f64()
        );
        let _ = std::io::stdout().flush();
    }
}
