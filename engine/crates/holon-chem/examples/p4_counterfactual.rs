//! Does decontracting 4p flip the high-spin ground state back? A computable counterfactual.
//!
//! # The claim being tested, and why it needs testing rather than repeating
//!
//! Zinc comes out a quintet and gallium a quartet where the periodic table says singlet
//! and doublet, and `zn_diagnose` established that both are genuine ground states of this
//! model rather than a Davidson that stepped over the answer. A mechanism was offered for
//! WHY, and it is comfortable enough to be worth distrusting:
//!
//! > STO-3G gives 4p the SAME exponents as 4s. That makes 4p artificially compact, and
//! > therefore artificially low in energy, and Hund exchange then over-stabilises a
//! > high-spin `4s(1) 4p(3)` configuration over the closed `4s(2)`.
//!
//! It is a story, not a measurement, and a story that explains a result is not evidence
//! for itself. This is its discriminator.
//!
//! # The counterfactual, and the PRE-REGISTERED prediction
//!
//! Give 4p its own exponents instead of its partner's: scale the 4p shell's three
//! exponents by `lambda < 1`, which makes that shell strictly more diffuse and therefore
//! strictly higher in energy, and leave every other shell exactly as declared. Nothing
//! else in the basis moves.
//!
//! **Predicted BEFORE running, and this file was committed with the prediction in it:** if
//! the mechanism is the cause, then as `lambda` falls the promotion into 4p stops paying
//! for itself and the ground state reverts — gallium from `2S+1 = 4` to `2`, zinc from `5`
//! to `1`. If the multiplicity does NOT revert at any `lambda` down to the point where 4p
//! is no longer meaningfully occupied, the shared-exponent story is wrong and the
//! high-spin result has some other cause.
//!
//! # What this is NOT
//!
//! Not a correction, and not a proposal to change the basis. STO-3G is the declared model
//! and its 4p exponents are part of the declaration; a `lambda != 1` basis is a DIFFERENT
//! model, computed here only to ask which of its features carries the effect. Every number
//! it produces is labelled counterfactual and none of it enters a gate.
//!
//! Usage: `cargo run --release -p holon-chem --example p4_counterfactual [symbol...]`

use holon_chem::dual::D2;
use holon_chem::elements::{by_symbol, ShellKind, Species};
use holon_chem::fci::{
    cholesky_orthonormaliser, s_squared, solve_determinant, transform, FciSpace,
};
use holon_chem::md::{ao_integrals, Basis};
use holon_chem::pair::electron_counts;

/// Scale factors on the 4p exponents. 1.0 is the declared basis and is the control: it must
/// reproduce the high-spin reading, or the counterfactual machinery is not computing the
/// same thing the registry does.
const LAMBDAS: &[f64] = &[1.0, 0.7, 0.5, 0.35, 0.25, 0.15];

fn main() {
    let syms: Vec<String> = {
        let a: Vec<String> = std::env::args().skip(1).collect();
        if a.is_empty() {
            vec!["Ga".into()]
        } else {
            a
        }
    };

    for sym in syms {
        let sp = by_symbol(&sym).unwrap_or_else(|| panic!("unknown element {sym}"));
        let (_, na, nb) = electron_counts(&[sp]);
        println!("== {sym} (Z = {}), {na}/{nb} electrons ==", sp.z);
        println!("{:>8} {:>20} {:>10} {:>8}", "lambda", "E (hartree)", "<S^2>", "2S+1");
        for &lam in LAMBDAS {
            let basis = scaled_4p_basis(sp, lam);
            let n = basis.n;
            let ao = ao_integrals(&basis);
            // The Cholesky orthonormaliser WITHOUT the SCF rotation. Full CI is invariant
            // under any unitary transformation of the orbitals it is full in, so the
            // energy and <S^2> are identical either way; only Davidson's conditioning
            // suffers. That is a cost, not a bias, and it keeps this example off the
            // crate's private SCF path.
            let x = cholesky_orthonormaliser(&ao.s, n).expect("overlap positive definite");
            let mo = transform(&ao, &x, n);
            let space = FciSpace::new(n, na, nb);
            let sol = solve_determinant(&space, &mo);
            let s2 = s_squared(&space, &sol.vector);
            let mult = (1.0 + 4.0 * s2).sqrt();
            println!(
                "{lam:>8.2} {:>20.9} {s2:>10.6} {mult:>8.3}",
                sol.e.v + basis.nuclear_repulsion().v
            );
        }
        println!();
    }
}

/// The element's declared basis with ONLY the 4p shell's exponents scaled.
///
/// Every other shell is passed through exactly as the registry declares it, including the
/// 4s shell whose exponents 4p currently shares — so `lambda` is precisely the amount by
/// which the sharing is broken, and nothing else differs.
fn scaled_4p_basis(sp: Species, lambda: f64) -> Basis {
    let mut decls = Vec::new();
    let mut found = false;
    for sh in sp.shells {
        let mut alpha = sh.alpha;
        if sh.kind == ShellKind::P4 {
            for a in alpha.iter_mut() {
                *a *= lambda;
            }
            found = true;
        }
        decls.push((0usize, sh.kind.l(), alpha, sh.coeff));
    }
    assert!(
        found,
        "{} has no 4p shell, so scaling one is not a counterfactual about anything",
        sp.symbol
    );
    Basis::assemble(
        vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]],
        vec![sp.z as f64],
        &decls,
    )
}
