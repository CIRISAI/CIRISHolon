//! Expectation values from a converged MPS — `Q8_MPS_PREREG.md` §4 (G2's density/double-
//! occupancy), §3 (G6's `m_i`, G0-2's standing sector-lock anchor including `<(Sz_tot)^2>`).
//!
//! Deliberately NOT the Hamiltonian's `D_BOND=7`-channel machinery (`mps::grow_left`/
//! `grow_right`): every observable here is a plain identity-everywhere overlap with at most two
//! local operators inserted, a strictly simpler contraction (bond dimension 1 throughout, no
//! MPO channel sum) that does not need to share code with `mps.rs`'s Hamiltonian environments.

use crate::mps::{self, TensorSite};
use crate::ops::{Op2, N2};

/// `<psi|O_{j1} O_{j2} ... |psi>` (RAW, not normalized by `<psi|psi>` — callers divide once at
/// the end), one local operator per listed site, identity everywhere else. A single
/// left-to-right pass, bond dimension 1 (this is NOT the Hamiltonian environment).
pub fn expectation(tensors: &[TensorSite], inserts: &[(usize, Op2)]) -> f64 {
    let mut lo = vec![1.0f64];
    let mut chi = 1usize;

    for (j, a) in tensors.iter().enumerate() {
        let op = inserts.iter().find(|(k, _)| *k == j).map(|(_, o)| o);
        let new_chi = a.chi_r;
        let mut new_lo = vec![0.0; new_chi * new_chi];

        for s in 0..2 {
            for sp in 0..2 {
                let ov = match op {
                    None => {
                        if s == sp {
                            1.0
                        } else {
                            continue;
                        }
                    }
                    Some(o) => o[s][sp],
                };
                if ov == 0.0 {
                    continue;
                }
                for l in 0..chi {
                    for r in 0..new_chi {
                        let av = a.get(s, l, r);
                        if av == 0.0 {
                            continue;
                        }
                        let ov_av = ov * av;
                        for lp in 0..chi {
                            let lov = lo[l * chi + lp];
                            if lov == 0.0 {
                                continue;
                            }
                            let w = ov_av * lov;
                            let dst = r * new_chi;
                            for rp in 0..new_chi {
                                new_lo[dst + rp] += w * a.get(sp, lp, rp);
                            }
                        }
                    }
                }
            }
        }
        lo = new_lo;
        chi = new_chi;
    }
    lo[0]
}

pub fn norm_squared(tensors: &[TensorSite]) -> f64 {
    expectation(tensors, &[])
}

/// `n_i = <n_{i,up} + n_{i,down}>` per chain site, 0-indexed.
pub fn occupation_profile(tensors: &[TensorSite], sites: usize) -> Vec<f64> {
    let norm = norm_squared(tensors);
    (0..sites)
        .map(|cs| {
            (expectation(tensors, &[(2 * cs, N2)]) + expectation(tensors, &[(2 * cs + 1, N2)]))
                / norm
        })
        .collect()
}

/// `m_i = <n_{i,up} - n_{i,down}>` per chain site — theorem-pinned to exactly 0 (G6).
pub fn magnetization_profile(tensors: &[TensorSite], sites: usize) -> Vec<f64> {
    let norm = norm_squared(tensors);
    (0..sites)
        .map(|cs| {
            (expectation(tensors, &[(2 * cs, N2)]) - expectation(tensors, &[(2 * cs + 1, N2)]))
                / norm
        })
        .collect()
}

/// `d_i = <n_{i,up} n_{i,down}>` per chain site — a genuine two-site simultaneous insertion.
pub fn double_occupancy_profile(tensors: &[TensorSite], sites: usize) -> Vec<f64> {
    let norm = norm_squared(tensors);
    (0..sites)
        .map(|cs| expectation(tensors, &[(2 * cs, N2), (2 * cs + 1, N2)]) / norm)
        .collect()
}

pub fn total_number(tensors: &[TensorSite]) -> f64 {
    let norm = norm_squared(tensors);
    let l = tensors.len();
    (0..l).map(|j| expectation(tensors, &[(j, N2)])).sum::<f64>() / norm
}

/// `Sz_tot = N_up - N_down` (unhalved, matching `m_i`'s own convention in this codebase —
/// `q-seam`'s `magnetization` field is `n_up-n_down`, not divided by 2).
pub fn total_sz(tensors: &[TensorSite]) -> f64 {
    let norm = norm_squared(tensors);
    let l = tensors.len();
    (0..l)
        .map(|j| {
            let sign = if mps::is_up_orbital(j) { 1.0 } else { -1.0 };
            sign * expectation(tensors, &[(j, N2)])
        })
        .sum::<f64>()
        / norm
}

/// `<(Sz_tot)^2>` — the G0-2 loophole-closer (`Q8_MPS_PREREG.md` §3): `<Sz_tot>=0` alone is
/// satisfied by an `Sz=+1`/`Sz=-1` mixture, this is not. Decomposed as
/// `<Sz_tot^2> = N_tot + 2 sum_{i<j} sign_i sign_j <n_i n_j>` (diagonal terms use `n_i^2=n_i`,
/// `sign_i^2=1`), `O(L^2)` pair correlators.
pub fn total_sz_squared(tensors: &[TensorSite]) -> f64 {
    let norm = norm_squared(tensors);
    let l = tensors.len();
    let n_tot: f64 = (0..l).map(|j| expectation(tensors, &[(j, N2)])).sum();

    let mut cross = 0.0;
    for i in 0..l {
        let si = if mps::is_up_orbital(i) { 1.0 } else { -1.0 };
        for j in (i + 1)..l {
            let sj = if mps::is_up_orbital(j) { 1.0 } else { -1.0 };
            cross += si * sj * expectation(tensors, &[(i, N2), (j, N2)]);
        }
    }
    (n_tot + 2.0 * cross) / norm
}
