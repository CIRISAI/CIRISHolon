//! Smoke test, not a staked gate: the two-site sweep engine at N=2 (trivially small, chi=4 is
//! already exact — `chi_l*2` never exceeds 4 on a 4-site chain) against q-seam's exact E0. Run
//! before scaling up to the staked N=8,10,12 gates, on the same "gate immediately" discipline
//! that caught the FINISH-channel bug in G0/G1.

use q8_mps::dmrg::{self, Params};

#[test]
fn n2_matches_exact_at_every_u() {
    for &u in &[0.0, 1.0, 4.0, 16.0] {
        let p = Params { sites: 2, t: 1.0, u, chi_max: 4, max_sweeps: 20, sweep_tol: 1e-12 };
        let r = dmrg::run(&p, dmrg::RefusalPolicy::Typed).expect("refused unexpectedly");

        let n_target = p.sites as f64;
        let mu = u / 2.0;
        let unshifted = r.energy_shifted + mu * n_target;

        let h = q_seam::hubbard::Hubbard::new(p.sites, p.t, u);
        let exact = q_seam::lanczos::ground_state(&h).expect("q-seam Lanczos failed");

        let err = (unshifted - exact.energy).abs();
        assert!(
            err <= 1e-8,
            "N=2 U={u}: DMRG {unshifted} vs exact {} (err {err:e}), sweeps={}, converged={}",
            exact.energy,
            r.sweeps_used,
            r.converged
        );
    }
}
