//! G0-1 — `Q8_MPS_PREREG.md` §3 / D1. The unrestricted (all-sector) ground state of
//! `H' = H - (U/2)*N_tot` must sit exactly at half filling, `Sz=0` — the fix that makes plain
//! two-site DMRG (no quantum-number blocking) target the right physics at all. Checked at
//! N=2,4 by brute-force full-Fock-space diagonalization before it is trusted at N=8-12.

mod common;

use common::{sector_of, U_GRID};
use q8_mps::mpo::dense_from_mpo;

const SECTOR_TOL: f64 = 1e-9;
const ENERGY_TOL: f64 = 1e-10;

#[test]
fn g0_1_shifted_ground_state_sits_at_half_filling() {
    for &sites in &[2usize, 4] {
        for &u in &U_GRID {
            let mu = u / 2.0;
            let dense = dense_from_mpo(sites, 1.0, u, mu);
            let dim = 1usize << (2 * sites);
            let eig = q_seam::dense::jacobi(dense, dim);
            assert!(eig.converged, "N={sites} U={u}: H' dense diag did not converge");

            // Ascending order (`dense::jacobi`'s convention): index 0 is the GLOBAL minimum
            // over the entire Fock space, not a sector-restricted one — exactly what D1's
            // worked example shows is at risk without the shift.
            let ground_energy_shifted = eig.values[0];
            let ground_vec = &eig.vectors[0];

            let target = sites as f64 / 2.0;
            let (nup, ndn) = sector_of(ground_vec, sites);
            assert!(
                (nup - target).abs() <= SECTOR_TOL && (ndn - target).abs() <= SECTOR_TOL,
                "N={sites} U={u}: unrestricted H' ground state is NOT half-filled — \
                 (N_up,N_down)=({nup},{ndn}), target=({target},{target}). D1's fix failed."
            );

            // Unshift with the INTEGER target, never the measured expectation — the pin from
            // team-lead review (`Q8_MPS_PREREG.md` §2, amendment).
            let n_target = sites as f64;
            let unshifted = ground_energy_shifted + mu * n_target;

            let h = q_seam::hubbard::Hubbard::new(sites, 1.0, u);
            let exact = q_seam::lanczos::ground_state(&h)
                .unwrap_or_else(|| panic!("N={sites} U={u}: q-seam Lanczos failed to converge"));

            let err = (unshifted - exact.energy).abs();
            assert!(
                err <= ENERGY_TOL,
                "N={sites} U={u}: shifted-back energy {unshifted} vs q-seam exact {} \
                 (err {err:e} > {ENERGY_TOL:e})",
                exact.energy
            );
        }
    }
}
