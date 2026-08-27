//! Regression for the Q8 failure localized by the independent claim-transport comparison.
//!
//! The two-site reshape in a strong-coupling sweep has Schmidt values separated by many
//! decades.  Every retained tensor still has to provide an orthonormal block basis: otherwise
//! `apply_effective_h` is a generalized eigenproblem, while the local Lanczos routine (correctly,
//! for canonical DMRG) solves the ordinary one.

use q8_mps::dmrg::{self, Params, RefusalPolicy};

#[test]
fn strong_coupling_sweep_preserves_both_canonical_bases() {
    let params = Params {
        sites: 8,
        t: 1.0,
        u: 16.0,
        chi_max: 32,
        max_sweeps: 2,
        sweep_tol: 0.0,
    };
    let result = dmrg::run(&params, RefusalPolicy::Typed).expect("unexpected ledger refusal");

    let tolerance = 1e-10;
    assert!(
        result.worst_left_canonical_defect <= tolerance,
        "left block basis lost canonical form: defect={:e}",
        result.worst_left_canonical_defect
    );
    assert!(
        result.worst_right_canonical_defect <= tolerance,
        "right block basis lost canonical form: defect={:e}",
        result.worst_right_canonical_defect
    );
}
