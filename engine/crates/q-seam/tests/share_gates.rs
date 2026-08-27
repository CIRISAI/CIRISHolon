//! `Q_SEAM_PREREG.md` §6.2 — the derived plumb line, as a regression gate.
//!
//! `I_C^(3)` is EXACTLY zero on this family, derived before the instrument existed: complement
//! symmetry forces the three fields of the pairwise maxent family to vanish, leaving 3 couplings
//! to match exactly 3 pair marginals. Parameter count equals constraint count, so `Q = P`.
//! A failure here means the estimator is broken and Q6 is VOID, not falsified.

use q_seam::hubbard::Hubbard;
use q_seam::lanczos::ground_state;
use q_seam::share::measure;

#[test]
fn g_q6_plumb_line_ic3_is_machine_zero() {
    for &n in &[4usize, 6] {
        for &u in &[0.0, 0.5, 2.0, 16.0] {
            let h = Hubbard::new(n, 1.0, u);
            let g = ground_state(&h).unwrap();
            let r = measure(&h, &g.vector);
            assert!(
                r.ic3_max <= 1e-12,
                "N={n} U={u}: |I_C^(3)| = {:e} — the estimator is broken, Q6 would be VOID",
                r.ic3_max
            );
            assert_eq!(r.failed, 0, "N={n} U={u}: {} quadruples reached neither solver", r.failed);
            assert!(
                r.worst_crosscheck <= q_seam::share::CROSSCHECK_GATE,
                "N={n} U={u}: Newton and IPF disagree by {:e} — the boundary-drift lesson firing",
                r.worst_crosscheck
            );
        }
    }
}

/// And the k=4 counterpart of the same derivation: 7 free parameters against 6 couplings leaves
/// one residual degree of freedom, so `B4` must be able to be NONZERO — otherwise the plumb line
/// above would pass vacuously on an estimator that returns zero for everything.
#[test]
fn b4_is_not_identically_zero() {
    let h = Hubbard::new(6, 1.0, 4.0);
    let g = ground_state(&h).unwrap();
    let r = measure(&h, &g.vector);
    assert!(r.b4_max > 1e-6, "B4 reads {:e}; the k=4 residual dof is not being seen", r.b4_max);
}
