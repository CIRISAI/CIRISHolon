//! G1a/G1b — `Q8_MPS_PREREG.md` §3. Nothing in G2+ runs before these pass.

mod common;

use common::{independent_dense_h, max_abs_diff, sector_of, U_GRID};
use q8_mps::mpo::dense_from_mpo;

const RECON_TOL: f64 = 1e-13;
const SPECTRUM_TOL: f64 = 1e-10;
const SECTOR_TOL: f64 = 1e-9;

#[test]
fn g1a_mpo_vs_independent_build() {
    for &sites in &[2usize, 4] {
        for &u in &U_GRID {
            let from_mpo = dense_from_mpo(sites, 1.0, u, 0.0);
            let independent = independent_dense_h(sites, 1.0, u, 0.0);
            let err = max_abs_diff(&from_mpo, &independent);
            assert!(
                err <= RECON_TOL,
                "G1a: N={sites} U={u}: max|ΔH| = {err:e} > {RECON_TOL:e}"
            );
        }
    }
}

#[test]
fn g1b_sector_projected_spectrum_vs_q_seam() {
    for &sites in &[2usize, 4] {
        for &u in &U_GRID {
            let dense = dense_from_mpo(sites, 1.0, u, 0.0);
            let dim = 1usize << (2 * sites);
            let eig = q_seam::dense::jacobi(dense, dim);
            assert!(eig.converged, "N={sites} U={u}: full-Fock dense diag did not converge");

            let mut mine: Vec<f64> = eig
                .values
                .iter()
                .zip(eig.vectors.iter())
                .filter_map(|(&e, v)| {
                    let (nup, ndn) = sector_of(v, sites);
                    let target = sites as f64 / 2.0;
                    if (nup - target).abs() <= SECTOR_TOL && (ndn - target).abs() <= SECTOR_TOL {
                        Some(e)
                    } else {
                        None
                    }
                })
                .collect();
            mine.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let h = q_seam::hubbard::Hubbard::new(sites, 1.0, u);
            let dense_seam = h.to_dense();
            let eig_seam = q_seam::dense::jacobi(dense_seam, h.dim());
            assert!(eig_seam.converged, "N={sites} U={u}: q-seam dense diag did not converge");
            let mut theirs = eig_seam.values.clone();
            theirs.sort_by(|a, b| a.partial_cmp(b).unwrap());

            assert_eq!(
                mine.len(),
                theirs.len(),
                "N={sites} U={u}: sector dimension mismatch, mine={} q-seam={}",
                mine.len(),
                theirs.len()
            );
            let err = max_abs_diff(&mine, &theirs);
            assert!(
                err <= SPECTRUM_TOL,
                "G1b: N={sites} U={u}: max|Δspectrum| = {err:e} > {SPECTRUM_TOL:e}"
            );
        }
    }
}
