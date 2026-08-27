//! Validates `q8_mps::observables` against brute-force dense expectation values at N=4
//! (generous chi=16 = the exact natural cap at the middle bond) before trusting it at N=8-12.
//! Not a staked prereg gate by itself — the staked gates (G2, G6, G0-2) reuse this same module.

mod common;

use common::{dense_observables, max_abs_diff, sector_of, U_GRID};
use q8_mps::dmrg::{self, Params};
use q8_mps::mpo::dense_from_mpo;
use q8_mps::observables;

const TOL: f64 = 1e-8;

#[test]
fn mps_observables_match_dense_at_n4() {
    let sites = 4;
    for &u in &U_GRID {
        // Bare-H sector-restricted ground state (brute force, mu=0 so no shift/unshift needed).
        let dense = dense_from_mpo(sites, 1.0, u, 0.0);
        let dim = 1usize << (2 * sites);
        let eig = q_seam::dense::jacobi(dense, dim);
        assert!(eig.converged, "U={u}: dense diag did not converge");

        let target = sites as f64 / 2.0;
        let (best_idx, _) = eig
            .values
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                let (nup, ndn) = sector_of(&eig.vectors[*i], sites);
                (nup - target).abs() <= 1e-9 && (ndn - target).abs() <= 1e-9
            })
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .expect("no sector-matching eigenvector found");
        let exact_v = &eig.vectors[best_idx];
        let exact = dense_observables(exact_v, sites);

        // MPS at generous chi (16 = natural cap at N=4's middle bond, so this should be
        // essentially exact, not merely close).
        let p = Params { sites, t: 1.0, u, chi_max: 16, max_sweeps: 20, sweep_tol: 1e-13 };
        let r = dmrg::run(&p, dmrg::RefusalPolicy::Typed).expect("refused unexpectedly");

        let mps_occ = observables::occupation_profile(&r.tensors, sites);
        let mps_mag = observables::magnetization_profile(&r.tensors, sites);
        let mps_docc = observables::double_occupancy_profile(&r.tensors, sites);
        let mps_ntot = observables::total_number(&r.tensors);
        let mps_sz = observables::total_sz(&r.tensors);
        let mps_sz_sq = observables::total_sz_squared(&r.tensors);

        let occ_err = max_abs_diff(&mps_occ, &exact.occupation);
        let mag_err = max_abs_diff(&mps_mag, &exact.magnetization);
        let docc_err = max_abs_diff(&mps_docc, &exact.double_occ);
        let ntot_err = (mps_ntot - exact.n_tot).abs();
        let sz_err = (mps_sz - exact.sz).abs();
        let sz_sq_err = (mps_sz_sq - exact.sz_sq).abs();

        assert!(occ_err <= TOL, "U={u}: occupation err {occ_err:e}");
        assert!(mag_err <= TOL, "U={u}: magnetization err {mag_err:e}");
        assert!(docc_err <= TOL, "U={u}: double-occ err {docc_err:e}");
        assert!(ntot_err <= TOL, "U={u}: N_tot err {ntot_err:e} (mps={mps_ntot}, exact={})", exact.n_tot);
        assert!(sz_err <= TOL, "U={u}: Sz err {sz_err:e} (mps={mps_sz}, exact={})", exact.sz);
        assert!(
            sz_sq_err <= TOL,
            "U={u}: Sz^2 err {sz_sq_err:e} (mps={mps_sz_sq}, exact={})",
            exact.sz_sq
        );
    }
}
