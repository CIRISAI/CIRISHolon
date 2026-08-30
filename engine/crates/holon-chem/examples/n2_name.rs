//! Name the sprint team's N2 number, or fail to and say so.
//!
//! Their receipt quotes E = -131.278565811 Ha for "N2". This engine's N2 total energy at
//! 3.0 bohr is -107.546741772 Ha, some 23.73 Ha away, so the two are not the same quantity
//! and the difference has to be named before their code is trusted.
//!
//! The hypothesis under test is the lead's: ELECTRONIC-ONLY at a different geometry. For
//! N2 the nuclear repulsion is 49/R, so if their number is our electronic energy then
//! 49/R = 23.73 and R = 2.065 bohr -- which is within a percent of nitrogen's experimental
//! bond length. This scans R and reports where, if anywhere, the electronic energy matches.
use holon_chem::dual::D2;
use holon_chem::elements::NITROGEN;
use holon_chem::fci::solve_determinant;
use holon_chem::pair::geometry_problem;
use std::io::Write;

const THEIRS: f64 = -131.278565811;

fn main() {
    println!("# their quoted N2 energy: {THEIRS:.9} Ha");
    println!("R_bohr\tE_total\tV_nn\tE_electronic\tE_elec - theirs");
    let _ = std::io::stdout().flush();
    let mut best = (f64::INFINITY, 0.0f64);
    let mut r = 2.0700f64;
    while r <= 2.0805 {
        let (space, mo, nuc) = geometry_problem(
            &[NITROGEN, NITROGEN],
            vec![
                [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                [D2::c(0.0), D2::c(0.0), D2::c(r)],
            ],
        );
        let _ = space;
        let total = (solve_determinant(&space, &mo).e + nuc).v;
        let vnn = nuc.v;
        let elec = total - vnn;
        let d = elec - THEIRS;
        println!("{r:.4}\t{total:.9}\t{vnn:.9}\t{elec:.9}\t{d:+.6e}");
        let _ = std::io::stdout().flush();
        if d.abs() < best.0 {
            best = (d.abs(), r);
        }
        r += 0.0020;
    }
    println!("# closest electronic match: R = {:.4} bohr, |delta| = {:.3e} Ha", best.1, best.0);
    println!("# nuclear repulsion there: {:.6} Ha", 49.0 / best.1);
}
