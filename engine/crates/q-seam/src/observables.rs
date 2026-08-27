//! Exact observables read off the ground state — the truth side of every comparison.
//!
//! The six observables of `Q_SEAM_PREREG.md` §2.1 all descend from two objects: the spin-resolved
//! one-body density matrix `ρ^σ_ij = ⟨c†_iσ c_jσ⟩`, and the per-site double occupancy
//! `⟨n_i↑ n_i↓⟩`. Site occupations are its diagonal, bond orders its first off-diagonal, and the
//! Boolean defect `D_bool` its eigenvalue spectrum — which is the ModeChart fence as a number:
//! the chart says every natural occupation is 0 or 1, and this says how wrong that is.
//!
//! `⟨S²⟩` is computed independently by applying `S₊` into the neighbouring spin sector, because
//! Lieb's theorem (`S = 0` at half filling on a bipartite lattice) is the statement that convicts
//! a symmetry-broken chart, and a theorem that load-bearing is measured rather than assumed.

use crate::dense::jacobi;
use crate::hubbard::{hop_sign, Hubbard, SpinBasis};

#[derive(Clone, Debug)]
pub struct ExactObservables {
    /// `rdm[0]` is spin up, `rdm[1]` spin down; each row-major `N × N`.
    pub rdm: [Vec<f64>; 2],
    /// `⟨n_iσ⟩` per site, per spin.
    pub occupation: [Vec<f64>; 2],
    /// `m_i = ⟨n_i↑ − n_i↓⟩`. Exactly zero by Lieb; measured anyway.
    pub magnetization: Vec<f64>,
    /// `n_i = ⟨n_i↑ + n_i↓⟩`. Exactly one by particle–hole; measured anyway.
    pub density: Vec<f64>,
    /// `⟨n_i↑ n_i↓⟩` per site.
    pub double_occ: Vec<f64>,
    /// Mean double occupancy per site — observable O2.
    pub double_occ_mean: f64,
    /// `b_i = Σ_σ ⟨c†_iσ c_{i+1,σ} + h.c.⟩` on each of the `N−1` bonds — observable O5.
    pub bond: Vec<f64>,
    /// Natural occupations of both spin blocks, ascending.
    pub natural_occ: Vec<f64>,
    /// `D_bool = max_p min(n_p, 1 − n_p)` — observable O6, the ModeChart fence.
    pub d_bool: f64,
    pub s_squared: f64,
}

impl ExactObservables {
    pub fn measure(h: &Hubbard, psi: &[f64]) -> Self {
        let sites = h.sites;
        let n = h.n_conf();

        let mut rdm_up = vec![0.0; sites * sites];
        let mut rdm_dn = vec![0.0; sites * sites];

        // ρ^σ_ij = ⟨c†_iσ c_jσ⟩. The opposite spin is a spectator, so the sum factorizes over
        // its configuration index.
        for i in 0..sites {
            for j in 0..sites {
                let (mut acc_up, mut acc_dn) = (0.0, 0.0);
                for (ic, &mask) in h.basis.masks.iter().enumerate() {
                    if let Some((new_mask, sign)) = hop_sign(mask, i, j) {
                        let ic2 = h.basis.index(new_mask);
                        for other in 0..n {
                            acc_up += sign * psi[ic2 * n + other] * psi[ic * n + other];
                            acc_dn += sign * psi[other * n + ic2] * psi[other * n + ic];
                        }
                    }
                }
                rdm_up[i * sites + j] = acc_up;
                rdm_dn[i * sites + j] = acc_dn;
            }
        }

        let occ_up: Vec<f64> = (0..sites).map(|i| rdm_up[i * sites + i]).collect();
        let occ_dn: Vec<f64> = (0..sites).map(|i| rdm_dn[i * sites + i]).collect();
        let magnetization = (0..sites).map(|i| occ_up[i] - occ_dn[i]).collect::<Vec<_>>();
        let density = (0..sites).map(|i| occ_up[i] + occ_dn[i]).collect::<Vec<_>>();

        let mut double_occ = vec![0.0; sites];
        for (iu, &mu) in h.basis.masks.iter().enumerate() {
            for (id, &md) in h.basis.masks.iter().enumerate() {
                let w = psi[iu * n + id] * psi[iu * n + id];
                let both = mu & md;
                for i in 0..sites {
                    if both & (1 << i) != 0 {
                        double_occ[i] += w;
                    }
                }
            }
        }
        let double_occ_mean = double_occ.iter().sum::<f64>() / sites as f64;

        let bond = (0..sites - 1)
            .map(|i| 2.0 * (rdm_up[i * sites + i + 1] + rdm_dn[i * sites + i + 1]))
            .collect::<Vec<_>>();

        let mut natural_occ = Vec::with_capacity(2 * sites);
        for block in [&rdm_up, &rdm_dn] {
            natural_occ.extend(jacobi(block.clone(), sites).values);
        }
        let d_bool = natural_occ
            .iter()
            .map(|&x| x.min(1.0 - x))
            .fold(f64::NEG_INFINITY, f64::max);

        let s_squared = s_squared(h, psi);

        Self {
            rdm: [rdm_up, rdm_dn],
            occupation: [occ_up, occ_dn],
            magnetization,
            density,
            double_occ,
            double_occ_mean,
            bond,
            natural_occ,
            d_bool,
            s_squared,
        }
    }

    /// Energy per site in units of `t`, observable O1.
    pub fn energy_per_site(energy: f64, sites: usize, t: f64) -> f64 {
        energy / (sites as f64 * t)
    }
}

/// `⟨S²⟩ = ‖S₊ψ‖²` in the `S_z = 0` sector, where `S² = S₋S₊ + S_z² + S_z` and `S_z ψ = 0`.
///
/// `S₊ = Σ_i c†_{i↑} c_{i↓}` leaves the sector, so the target lives in `(N/2+1, N/2−1)` and needs
/// its own basis. The Jordan–Wigner string for the down operator runs past every up orbital in
/// the (all up, then all down) ordering; that contributes a constant parity here, but it is
/// carried explicitly rather than dropped as "a global phase I checked in my head".
fn s_squared(h: &Hubbard, psi: &[f64]) -> f64 {
    let sites = h.sites;
    let k = sites / 2;
    if k + 1 > sites {
        return 0.0;
    }
    let up_target = SpinBasis::new(sites, k + 1);
    let dn_target = SpinBasis::new(sites, k - 1);
    let n = h.n_conf();
    let (nu, nd) = (up_target.len(), dn_target.len());
    let mut out = vec![0.0; nu * nd];

    for (iu, &mu) in h.basis.masks.iter().enumerate() {
        // Parity of the whole up block, passed by the down annihilator.
        let block_parity = if mu.count_ones() % 2 == 0 { 1.0 } else { -1.0 };
        for (id, &md) in h.basis.masks.iter().enumerate() {
            let amp = psi[iu * n + id];
            if amp == 0.0 {
                continue;
            }
            for i in 0..sites {
                if md & (1 << i) == 0 || mu & (1 << i) != 0 {
                    continue;
                }
                let s_dn = if (md & ((1u16 << i) - 1)).count_ones() % 2 == 0 { 1.0 } else { -1.0 };
                let s_up = if (mu & ((1u16 << i) - 1)).count_ones() % 2 == 0 { 1.0 } else { -1.0 };
                let new_dn = md ^ (1 << i);
                let new_up = mu | (1 << i);
                out[up_target.index(new_up) * nd + dn_target.index(new_dn)] +=
                    block_parity * s_dn * s_up * amp;
            }
        }
    }
    out.iter().map(|x| x * x).sum()
}
