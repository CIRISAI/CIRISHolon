//! Smoke test, not a staked gate: N=2's smoke test never actually forces truncation (chi_max=4
//! was never below the natural per-bond cap there). This checks N=4 (natural middle-bond cap
//! 2^4=16) with chi_max=4, deliberately truncating hard enough that the (also correct) refusal
//! mechanism fires at this chi for some U — that is G5's job, tested there, not this test's, so
//! this uses `RefusalPolicy::Silent` deliberately to isolate the TRUNCATION MATH: does a
//! truncated run stay at/above the exact floor and land reasonably close, independent of
//! whether the engine would also have refused.

use q8_mps::dmrg::{self, Params, RefusalPolicy};

#[test]
fn n4_truncated_stays_above_exact_and_close() {
    let mut any_truncated = false;
    for &u in &[0.0, 1.0, 4.0, 16.0] {
        let p = Params { sites: 4, t: 1.0, u, chi_max: 4, max_sweeps: 20, sweep_tol: 1e-12 };
        let r = dmrg::run(&p, RefusalPolicy::Silent).expect("Silent policy never refuses");

        let n_target = p.sites as f64;
        let mu = u / 2.0;
        let unshifted = r.energy_shifted + mu * n_target;

        let h = q_seam::hubbard::Hubbard::new(p.sites, p.t, u);
        let exact = q_seam::lanczos::ground_state(&h).expect("q-seam Lanczos failed");

        // Variational floor (G3-secondary's shape, informally): truncated DMRG can only ever
        // read AT or ABOVE the true ground energy, small numerical slack.
        assert!(
            unshifted >= exact.energy - 1e-8,
            "N=4 U={u}: DMRG {unshifted} BELOW exact {} — floor violated",
            exact.energy
        );
        // Some truncation error is expected at chi=4 against a natural cap of 16; sanity-bound
        // it loosely just to catch gross breakage, not to stake a real tolerance (that's G2).
        assert!(
            (unshifted - exact.energy).abs() <= 1.0,
            "N=4 U={u}: DMRG {unshifted} vs exact {} — absurdly far off, real bug",
            exact.energy
        );
        if r.discarded_weight.iter().any(|&w| w > 1e-9) {
            any_truncated = true;
        }
    }
    // Not every U need force truncation at chi=4 (U=0's Slater determinant can genuinely have
    // Schmidt rank <=4 at this small N) — but SOME U must, or the truncation path is untested.
    assert!(any_truncated, "N=4: chi=4 never truncated anything at ANY U — truncation path unexercised");
}
