//! MPS tensors, MPO environments, and the matrix-free two-site effective Hamiltonian —
//! `Q8_MPS_PREREG.md` §2. `chi` here is one bond's declared LEDGER: `dmrg.rs` is where it gets
//! enforced (truncation, discarded weight, the typed refusal); this module only builds and
//! contracts, it never truncates.

use crate::mpo;

/// One MPS site tensor, physical dimension 2 (occupied/empty), flat `[s][l][r]` layout.
#[derive(Clone, Debug)]
pub struct TensorSite {
    pub chi_l: usize,
    pub chi_r: usize,
    pub data: Vec<f64>,
}

impl TensorSite {
    pub fn zeros(chi_l: usize, chi_r: usize) -> Self {
        Self { chi_l, chi_r, data: vec![0.0; 2 * chi_l * chi_r] }
    }

    #[inline]
    pub fn get(&self, s: usize, l: usize, r: usize) -> f64 {
        self.data[(s * self.chi_l + l) * self.chi_r + r]
    }

    #[inline]
    pub fn set(&mut self, s: usize, l: usize, r: usize, v: f64) {
        self.data[(s * self.chi_l + l) * self.chi_r + r] = v;
    }
}

/// One MPO channel's `chi x chi` block per element, `D_BOND` of them (mostly zero away from the
/// trivial boundary). Kept dense rather than sparse — `D_BOND=7` is small and fixed, and dense
/// arithmetic is far simpler to get right than tracking per-channel sparsity, which is exactly
/// the risk class G1's bug came from.
pub type Env = Vec<Vec<f64>>;

pub fn trivial_left_env() -> Env {
    let mut e: Env = (0..mpo::D_BOND).map(|_| vec![0.0]).collect();
    e[mpo::START][0] = 1.0;
    e
}

pub fn trivial_right_env() -> Env {
    let mut e: Env = (0..mpo::D_BOND).map(|_| vec![0.0]).collect();
    e[mpo::FINISH][0] = 1.0;
    e
}

/// `max|env[channel] - I|` — the canonical-form check. `Left[START]` after absorbing sites
/// `0..j-1` is EXACTLY the left block's overlap matrix `<block|block>` (the only edge with
/// `to=START` in `mpo.rs`'s channel graph is the pure-identity self-loop, so no Hamiltonian
/// content ever reaches this channel); it equals the identity iff those tensors are properly
/// left-canonical. Mirror for `Right[FINISH]`. A correct two-site update with zero discarded
/// weight cannot raise the variational energy — if it does, this is the first thing to check,
/// not assumed clean (team-lead/chief-of-staff-2 finding, N=8 U=16's 2.55e-2 non-monotone rise
/// at `discarded_max=0`).
pub fn identity_defect(env: &Env, channel: usize) -> f64 {
    let n = env[channel].len();
    let chi = (n as f64).sqrt().round() as usize;
    debug_assert_eq!(chi * chi, n, "env[channel] is not a square chi x chi matrix");
    let mut worst = 0.0f64;
    for i in 0..chi {
        for j in 0..chi {
            let target = if i == j { 1.0 } else { 0.0 };
            worst = worst.max((env[channel][i * chi + j] - target).abs());
        }
    }
    worst
}

/// Absorb one more site into a LEFT environment: `new_L[c2][r,r'] = sum_{c1,s,sp,l,l'}
/// A[s][l][r] . L[c1][l,l'] . W[c1][c2][s][sp] . A[sp][l'][r']`, staged to avoid ever forming
/// the full `D^2` object at once.
pub fn grow_left(l_env: &Env, w: &[f64], a: &TensorSite) -> Env {
    let d = mpo::D_BOND;
    let (chi_l, chi_r) = (a.chi_l, a.chi_r);

    // Stage A: tmp_a[c1][s][r][l'] = sum_l A[s][l][r] . L[c1][l][l']
    let mut tmp_a = vec![0.0; d * 2 * chi_r * chi_l];
    for c1 in 0..d {
        let lmat = &l_env[c1];
        for s in 0..2 {
            for l in 0..chi_l {
                let lrow = l * chi_l;
                for r in 0..chi_r {
                    let av = a.get(s, l, r);
                    if av == 0.0 {
                        continue;
                    }
                    let base = ((c1 * 2 + s) * chi_r + r) * chi_l;
                    for lp in 0..chi_l {
                        tmp_a[base + lp] += av * lmat[lrow + lp];
                    }
                }
            }
        }
    }

    // Stage B: tmp_b[c2][sp][r][l'] = sum_{c1,s} tmp_a[c1][s][r][l'] . W[c1][c2][s][sp]
    let mut tmp_b = vec![0.0; d * 2 * chi_r * chi_l];
    let block = chi_r * chi_l;
    for c1 in 0..d {
        for c2 in 0..d {
            for s in 0..2 {
                for sp in 0..2 {
                    let wv = w[((c1 * d + c2) * 2 + s) * 2 + sp];
                    if wv == 0.0 {
                        continue;
                    }
                    let src = (c1 * 2 + s) * block;
                    let dst = (c2 * 2 + sp) * block;
                    for idx in 0..block {
                        tmp_b[dst + idx] += wv * tmp_a[src + idx];
                    }
                }
            }
        }
    }

    // Stage C: new_L[c2][r][r'] = sum_{sp,l'} tmp_b[c2][sp][r][l'] . A[sp][l'][r']
    let mut new_l: Env = (0..d).map(|_| vec![0.0; chi_r * chi_r]).collect();
    for c2 in 0..d {
        for sp in 0..2 {
            for r in 0..chi_r {
                let base = ((c2 * 2 + sp) * chi_r + r) * chi_l;
                for lp in 0..chi_l {
                    let bv = tmp_b[base + lp];
                    if bv == 0.0 {
                        continue;
                    }
                    let row = &mut new_l[c2][r * chi_r..r * chi_r + chi_r];
                    for rp in 0..chi_r {
                        row[rp] += bv * a.get(sp, lp, rp);
                    }
                }
            }
        }
    }
    new_l
}

/// Mirror of `grow_left`, absorbing one more site into a RIGHT environment:
/// `new_R[c1][l,l'] = sum_{c2,s,sp,r,r'} A[s][l][r] . W[c1][c2][s][sp] . R[c2][r][r'] .
/// A[sp][l'][r']`.
pub fn grow_right(r_env: &Env, w: &[f64], a: &TensorSite) -> Env {
    let d = mpo::D_BOND;
    let (chi_l, chi_r) = (a.chi_l, a.chi_r);

    // Stage A: tmp_a[c2][s][l][r'] = sum_r A[s][l][r] . R[c2][r][r']
    let mut tmp_a = vec![0.0; d * 2 * chi_l * chi_r];
    for c2 in 0..d {
        let rmat = &r_env[c2];
        for s in 0..2 {
            for l in 0..chi_l {
                let base = ((c2 * 2 + s) * chi_l + l) * chi_r;
                for r in 0..chi_r {
                    let av = a.get(s, l, r);
                    if av == 0.0 {
                        continue;
                    }
                    let rrow = r * chi_r;
                    for rp in 0..chi_r {
                        tmp_a[base + rp] += av * rmat[rrow + rp];
                    }
                }
            }
        }
    }

    // Stage B: tmp_b[c1][sp][l][r'] = sum_{c2,s} tmp_a[c2][s][l][r'] . W[c1][c2][s][sp]
    let mut tmp_b = vec![0.0; d * 2 * chi_l * chi_r];
    let block = chi_l * chi_r;
    for c1 in 0..d {
        for c2 in 0..d {
            for s in 0..2 {
                for sp in 0..2 {
                    let wv = w[((c1 * d + c2) * 2 + s) * 2 + sp];
                    if wv == 0.0 {
                        continue;
                    }
                    let src = (c2 * 2 + s) * block;
                    let dst = (c1 * 2 + sp) * block;
                    for idx in 0..block {
                        tmp_b[dst + idx] += wv * tmp_a[src + idx];
                    }
                }
            }
        }
    }

    // Stage C: new_R[c1][l][l'] = sum_{sp,r'} tmp_b[c1][sp][l][r'] . A[sp][l'][r']
    let mut new_r: Env = (0..d).map(|_| vec![0.0; chi_l * chi_l]).collect();
    for c1 in 0..d {
        for sp in 0..2 {
            for l in 0..chi_l {
                let base = ((c1 * 2 + sp) * chi_l + l) * chi_r;
                for rp in 0..chi_r {
                    let bv = tmp_b[base + rp];
                    if bv == 0.0 {
                        continue;
                    }
                    let row = &mut new_r[c1][l * chi_l..l * chi_l + chi_l];
                    for lp in 0..chi_l {
                        row[lp] += bv * a.get(sp, lp, rp);
                    }
                }
            }
        }
    }
    new_r
}

/// `H_eff|psi>` for the two active sites `j,j+1`, `psi` flat `[l][a][b][r]`
/// (`a`=site `j`'s physical index, `b`=site `j+1`'s), `w1`/`w2` those sites' dense MPO tensors.
/// Staged right-to-left: contract `R` first, then `W2`, then `W1`, then `L` — every intermediate
/// stays `O(D . chi_l . 4 . chi_r)`, never the full `D^2`-channel object.
pub fn apply_effective_h(
    left: &Env,
    w1: &[f64],
    w2: &[f64],
    right: &Env,
    psi: &[f64],
    chi_l: usize,
    chi_r: usize,
) -> Vec<f64> {
    let d = mpo::D_BOND;

    // Step 1: t1[c2][l_in][a][b][r_out] = sum_{r_in} R[c2][r_out,r_in] . psi[l_in,a,b,r_in]
    let mut t1 = vec![0.0; d * chi_l * 2 * 2 * chi_r];
    for c2 in 0..d {
        let rmat = &right[c2];
        for l_in in 0..chi_l {
            for a in 0..2 {
                for b in 0..2 {
                    let psi_base = ((l_in * 2 + a) * 2 + b) * chi_r;
                    let out_base = (((c2 * chi_l + l_in) * 2 + a) * 2 + b) * chi_r;
                    for r_out in 0..chi_r {
                        let rrow = r_out * chi_r;
                        let mut acc = 0.0;
                        for r_in in 0..chi_r {
                            acc += rmat[rrow + r_in] * psi[psi_base + r_in];
                        }
                        t1[out_base + r_out] = acc;
                    }
                }
            }
        }
    }

    // Step 2: t2[c1'][l_in][a][t][r_out] = sum_{b,c2} t1[c2][l_in][a][b][r_out] . W2[c1',c2,t,b]
    let mut t2 = vec![0.0; d * chi_l * 2 * 2 * chi_r];
    for c1p in 0..d {
        for c2 in 0..d {
            for t in 0..2 {
                for b in 0..2 {
                    let wv = w2[((c1p * d + c2) * 2 + t) * 2 + b];
                    if wv == 0.0 {
                        continue;
                    }
                    for l_in in 0..chi_l {
                        for a in 0..2 {
                            let src = (((c2 * chi_l + l_in) * 2 + a) * 2 + b) * chi_r;
                            let dst = (((c1p * chi_l + l_in) * 2 + a) * 2 + t) * chi_r;
                            for r_out in 0..chi_r {
                                t2[dst + r_out] += wv * t1[src + r_out];
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 3: t3[c1][l_in][s][t][r_out] = sum_{a,c1'} t2[c1'][l_in][a][t][r_out] . W1[c1,c1',s,a]
    let mut t3 = vec![0.0; d * chi_l * 2 * 2 * chi_r];
    for c1 in 0..d {
        for c1p in 0..d {
            for s in 0..2 {
                for a in 0..2 {
                    let wv = w1[((c1 * d + c1p) * 2 + s) * 2 + a];
                    if wv == 0.0 {
                        continue;
                    }
                    for l_in in 0..chi_l {
                        for t in 0..2 {
                            let src = (((c1p * chi_l + l_in) * 2 + a) * 2 + t) * chi_r;
                            let dst = (((c1 * chi_l + l_in) * 2 + s) * 2 + t) * chi_r;
                            for r_out in 0..chi_r {
                                t3[dst + r_out] += wv * t2[src + r_out];
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 4: out[l_out][s][t][r_out] = sum_{c1,l_in} L[c1][l_out,l_in] . t3[c1][l_in][s][t][r_out]
    let mut out = vec![0.0; chi_l * 2 * 2 * chi_r];
    for c1 in 0..d {
        let lmat = &left[c1];
        for l_out in 0..chi_l {
            let lrow = l_out * chi_l;
            for s in 0..2 {
                for t in 0..2 {
                    let out_base = ((l_out * 2 + s) * 2 + t) * chi_r;
                    for r_out in 0..chi_r {
                        let mut acc = 0.0;
                        for l_in in 0..chi_l {
                            let src = (((c1 * chi_l + l_in) * 2 + s) * 2 + t) * chi_r;
                            acc += lmat[lrow + l_in] * t3[src + r_out];
                        }
                        out[out_base + r_out] += acc;
                    }
                }
            }
        }
    }
    out
}

/// The pinned deterministic initial state (`Q8_MPS_PREREG.md` §2): chain site `cs` (0-indexed)
/// carries an up electron if `cs` even, a down electron if `cs` odd — the Néel product state,
/// `chi=1` everywhere.
pub fn initial_state(sites: usize) -> Vec<TensorSite> {
    let l = 2 * sites;
    (0..l)
        .map(|j| {
            let cs = j / 2;
            let is_up_orbital = j % 2 == 0;
            let occupied = if is_up_orbital { cs % 2 == 0 } else { cs % 2 == 1 };
            let mut t = TensorSite::zeros(1, 1);
            t.set(if occupied { 1 } else { 0 }, 0, 0, 1.0);
            t
        })
        .collect()
}

/// Zero-pad every bond of `tensors` up to `min(target_chi, natural_cap)`, `natural_cap` at bond
/// `k` (`k=0..=L`) being `min(2^k, 2^(L-k))` — the SAME physical bound `split_two_site`'s SVD
/// rank falls out of automatically, computed explicitly here since there is no SVD step to fall
/// out of. Represents the EXACT SAME quantum state at a larger declared ledger: the padded
/// entries are zero, not a guess. `Q9`'s chi-warm-start probe/remedy: sweep a converged state at
/// a small `chi`, pad it up, sweep again at the larger `chi_max` instead of restarting from the
/// pinned product state.
pub fn pad_to_chi(tensors: &[TensorSite], target_chi: usize) -> Vec<TensorSite> {
    let l = tensors.len();
    let natural_cap = |k: usize| -> usize { target_chi.min(1usize << k).min(1usize << (l - k)) };

    (0..l)
        .map(|j| {
            let old = &tensors[j];
            let new_chi_l = natural_cap(j);
            let new_chi_r = natural_cap(j + 1);
            let mut nt = TensorSite::zeros(new_chi_l, new_chi_r);
            for s in 0..2 {
                for lidx in 0..old.chi_l.min(new_chi_l) {
                    for ridx in 0..old.chi_r.min(new_chi_r) {
                        nt.set(s, lidx, ridx, old.get(s, lidx, ridx));
                    }
                }
            }
            nt
        })
        .collect()
}

/// `is_up_orbital(j)` for JW site `j`, 0-indexed — `j` even is the up half of chain site `j/2`.
#[inline]
pub fn is_up_orbital(j: usize) -> bool {
    j.is_multiple_of(2)
}

/// Split a two-site ground-state tensor (flat `[l][a][b][r]`, which IS already row-major
/// `(chi_l*2) x (2*chi_r)` — `mps.rs`'s own `[l][a][b][r]` layout collapses to exactly that, no
/// copy needed) into two site tensors via `svd::jacobi_svd`, truncated to the declared ledger
/// `chi_max` — the ONE place a bond's dimension gets enforced. Returns `(left, right,
/// discarded_weight)`; `absorb_s_left` carries the singular values into whichever tensor should
/// hold the orthogonality center next (left when sweeping right-to-left, right when sweeping
/// left-to-right — `dmrg.rs` picks the direction, this function just does what it's told).
pub fn split_two_site(
    psi: &[f64],
    chi_l: usize,
    chi_r: usize,
    chi_max: usize,
    absorb_s_left: bool,
) -> (TensorSite, TensorSite, f64, f64) {
    let m = chi_l * 2;
    let n = 2 * chi_r;
    let svd = crate::svd::jacobi_svd(psi, m, n);
    assert!(
        svd.converged,
        "two-site Jacobi SVD did not reach its relative canonicality tolerance for {m}x{n} reshape"
    );
    let k = svd.s.len();
    let chi_new = chi_max.min(k).max(1);

    let discarded: f64 = svd.s[chi_new..].iter().map(|s| s * s).sum();

    // THE FENCE (Q10 §3a): this bond's own kept-spectrum floor, `s_min / s_max` over the
    // RETAINED singular values. A bond whose smallest KEPT value is still far below its largest
    // has budget to spare; one whose kept spectrum has flattened up against its largest value
    // has none. Computed by explicit min/max rather than from `s[0]` and `s[chi_new-1]`, so it
    // does not silently depend on the SVD's descending-order guarantee.
    //
    // It is NOT an error estimate and must never be reported as one (Q10 §2). It is the chart's
    // own declaration of how close it is to its declared ledger.
    let kept = &svd.s[..chi_new];
    let s_max = kept.iter().copied().fold(0.0f64, f64::max);
    let s_min = kept.iter().copied().fold(f64::INFINITY, f64::min);
    let spectrum_floor = if s_max > 0.0 { s_min / s_max } else { 0.0 };

    let mut a_left = TensorSite::zeros(chi_l, chi_new);
    let mut a_right = TensorSite::zeros(chi_new, chi_r);

    for i in 0..chi_new {
        let (sfac_left, sfac_right) = if absorb_s_left { (svd.s[i], 1.0) } else { (1.0, svd.s[i]) };
        for l in 0..chi_l {
            for a in 0..2 {
                a_left.set(a, l, i, svd.u[i][l * 2 + a] * sfac_left);
            }
        }
        for b in 0..2 {
            for r in 0..chi_r {
                a_right.set(b, i, r, svd.v[i][b * chi_r + r] * sfac_right);
            }
        }
    }

    (a_left, a_right, discarded, spectrum_floor)
}

// ------------------------------------------------------------------ General MPO environment contractions

/// Trivial left environment for an MPO with incoming bond dimension `d_l`.
pub fn trivial_left_env_mpo(d_l: usize) -> Env {
    let mut e: Env = (0..d_l).map(|_| vec![0.0]).collect();
    if d_l > 0 {
        e[0][0] = 1.0;
    }
    e
}

/// Trivial right environment for an MPO with outgoing bond dimension `d_r`.
pub fn trivial_right_env_mpo(d_r: usize) -> Env {
    let mut e: Env = (0..d_r).map(|_| vec![0.0]).collect();
    if d_r > 0 {
        e[0][0] = 1.0;
    }
    e
}

/// Hartree–Fock initial product state (`chi=1`) with `n_alpha` alpha electrons and `n_beta` beta electrons
/// in `n_orb` spatial orbitals (`2 * n_orb` spin-orbitals in interleaved Jordan–Wigner order).
pub fn initial_state_hf(n_orb: usize, n_alpha: usize, n_beta: usize) -> Vec<TensorSite> {
    let l = 2 * n_orb;
    (0..l)
        .map(|j| {
            let p = j / 2;
            let is_up = is_up_orbital(j);
            let occupied = if is_up { p < n_alpha } else { p < n_beta };
            let mut t = TensorSite::zeros(1, 1);
            t.set(if occupied { 1 } else { 0 }, 0, 0, 1.0);
            t
        })
        .collect()
}

/// Absorb one more site into a LEFT environment using an `MpoSite` tensor.
pub fn grow_left_mpo(l_env: &Env, site: &crate::mpo::MpoSite, a: &TensorSite) -> Env {
    let (d_l, d_r) = (site.d_l, site.d_r);
    let (chi_l, chi_r) = (a.chi_l, a.chi_r);

    // Stage A: tmp_a[c1][s][r][lp] = sum_l A[s][l][r] * L[c1][l][lp], live channels only
    let mut tmp_a = vec![0.0; d_l * 2 * chi_r * chi_l];
    for c1 in live_channels(l_env) {
        let lmat = &l_env[c1];
        for s in 0..2 {
            for l in 0..chi_l {
                let lrow = l * chi_l;
                for r in 0..chi_r {
                    let av = a.get(s, l, r);
                    if av == 0.0 {
                        continue;
                    }
                    let base = ((c1 * 2 + s) * chi_r + r) * chi_l;
                    for lp in 0..chi_l {
                        tmp_a[base + lp] += av * lmat[lrow + lp];
                    }
                }
            }
        }
    }

    // Stage B: tmp_b[c2][sp][r][lp] = sum_{c1,s} tmp_a[c1][s][r][lp] * W[c1, c2, s, sp]
    let mut tmp_b = vec![0.0; d_r * 2 * chi_r * chi_l];
    let block = chi_r * chi_l;
    for c1 in 0..d_l {
        for c2 in 0..d_r {
            for s in 0..2 {
                for sp in 0..2 {
                    let wv = site.get(c1, c2, s, sp);
                    if wv == 0.0 {
                        continue;
                    }
                    let src = (c1 * 2 + s) * block;
                    let dst = (c2 * 2 + sp) * block;
                    for idx in 0..block {
                        tmp_b[dst + idx] += wv * tmp_a[src + idx];
                    }
                }
            }
        }
    }

    // Stage C: new_L[c2][r][rp] = sum_{sp,lp} tmp_b[c2][sp][r][lp] * A[sp][lp][rp]
    let mut new_l: Env = (0..d_r).map(|_| vec![0.0; chi_r * chi_r]).collect();
    for c2 in 0..d_r {
        for sp in 0..2 {
            for r in 0..chi_r {
                let base = ((c2 * 2 + sp) * chi_r + r) * chi_l;
                for lp in 0..chi_l {
                    let bv = tmp_b[base + lp];
                    if bv == 0.0 {
                        continue;
                    }
                    let row = &mut new_l[c2][r * chi_r..r * chi_r + chi_r];
                    for rp in 0..chi_r {
                        row[rp] += bv * a.get(sp, lp, rp);
                    }
                }
            }
        }
    }
    new_l
}

/// Absorb one more site into a RIGHT environment using an `MpoSite` tensor.
pub fn grow_right_mpo(r_env: &Env, site: &crate::mpo::MpoSite, a: &TensorSite) -> Env {
    let (d_l, d_r) = (site.d_l, site.d_r);
    let (chi_l, chi_r) = (a.chi_l, a.chi_r);

    // Stage A: tmp_a[c2][s][l][rp] = sum_r A[s][l][r] * R[c2][r][rp], live channels only
    let mut tmp_a = vec![0.0; d_r * 2 * chi_l * chi_r];
    for c2 in live_channels(r_env) {
        let rmat = &r_env[c2];
        for s in 0..2 {
            for l in 0..chi_l {
                let base = ((c2 * 2 + s) * chi_l + l) * chi_r;
                for r in 0..chi_r {
                    let av = a.get(s, l, r);
                    if av == 0.0 {
                        continue;
                    }
                    let rrow = r * chi_r;
                    for rp in 0..chi_r {
                        tmp_a[base + rp] += av * rmat[rrow + rp];
                    }
                }
            }
        }
    }

    // Stage B: tmp_b[c1][sp][l][rp] = sum_{c2,s} tmp_a[c2][s][l][rp] * W[c1, c2, s, sp]
    let mut tmp_b = vec![0.0; d_l * 2 * chi_l * chi_r];
    let block = chi_l * chi_r;
    for c1 in 0..d_l {
        for c2 in 0..d_r {
            for s in 0..2 {
                for sp in 0..2 {
                    let wv = site.get(c1, c2, s, sp);
                    if wv == 0.0 {
                        continue;
                    }
                    let src = (c2 * 2 + s) * block;
                    let dst = (c1 * 2 + sp) * block;
                    for idx in 0..block {
                        tmp_b[dst + idx] += wv * tmp_a[src + idx];
                    }
                }
            }
        }
    }

    // Stage C: new_R[c1][l][lp] = sum_{sp,rp} tmp_b[c1][sp][l][rp] * A[sp][lp][rp]
    let mut new_r: Env = (0..d_l).map(|_| vec![0.0; chi_l * chi_l]).collect();
    for c1 in 0..d_l {
        for sp in 0..2 {
            for l in 0..chi_l {
                let base = ((c1 * 2 + sp) * chi_l + l) * chi_r;
                for rp in 0..chi_r {
                    let bv = tmp_b[base + rp];
                    if bv == 0.0 {
                        continue;
                    }
                    let row = &mut new_r[c1][l * chi_l..l * chi_l + chi_l];
                    for lp in 0..chi_l {
                        row[lp] += bv * a.get(sp, lp, rp);
                    }
                }
            }
        }
    }
    new_r
}

/// Apply effective 2-site Hamiltonian using `MpoSite` tensors.
/// Worker threads for the two heavy contraction steps of [`apply_effective_h_mpo`], from
/// `Q8_THREADS` (default 1). The split is over DISJOINT output regions with the reduction
/// order inside each unchanged, so the result is bit-identical at every thread count —
/// the thread count is not part of the arithmetic regime (M-DEVICE-CLASS).
pub fn threads() -> usize {
    static T: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *T.get_or_init(|| {
        std::env::var("Q8_THREADS").ok().and_then(|v| v.parse().ok()).filter(|&n| n >= 1).unwrap_or(1)
    })
}

/// Which channels of an environment carry anything at all. Most channels of a
/// many-channel MPO (the QCD₂ accumulator MPO has 42) are structurally zero at a given
/// bond; scanning them costs the same as the live ones, so they are skipped by name.
pub fn live_channels(env: &Env) -> Vec<usize> {
    env.iter()
        .enumerate()
        .filter(|(_, m)| m.iter().any(|&v| v != 0.0))
        .map(|(c, _)| c)
        .collect()
}

pub fn apply_effective_h_mpo(
    left: &Env,
    w1: &crate::mpo::MpoSite,
    w2: &crate::mpo::MpoSite,
    right: &Env,
    psi: &[f64],
    chi_l: usize,
    chi_r: usize,
) -> Vec<f64> {
    apply_effective_h_mpo_live(left, w1, w2, right, psi, chi_l, chi_r, &live_channels(left), &live_channels(right))
}

/// [`apply_effective_h_mpo`] with the live-channel lists supplied: the environments are fixed
/// for a whole local eigensolve, so the Lanczos loop computes them once and passes them in.
#[allow(clippy::too_many_arguments)]
pub fn apply_effective_h_mpo_live(
    left: &Env,
    w1: &crate::mpo::MpoSite,
    w2: &crate::mpo::MpoSite,
    right: &Env,
    psi: &[f64],
    chi_l: usize,
    chi_r: usize,
    live_l: &[usize],
    live_r: &[usize],
) -> Vec<f64> {
    let (d_l, d_mid, d_r) = (w1.d_l, w1.d_r, w2.d_r);
    debug_assert_eq!(d_mid, w2.d_l);
    let nthreads = threads();
    // Step 1: t1[c2][l_in][a][b][r_out] = sum_{r_in} R[c2][r_out,r_in] * psi[l_in,a,b,r_in]
    // over the LIVE right channels only, threaded over disjoint channel blocks.
    let mut t1 = vec![0.0; d_r * chi_l * 2 * 2 * chi_r];
    {
        let block = chi_l * 2 * 2 * chi_r;
        let step1 = |c2: usize, t1c: &mut [f64]| {
            let rmat = &right[c2];
            for l_in in 0..chi_l {
                for a in 0..2 {
                    for b in 0..2 {
                        let psi_base = ((l_in * 2 + a) * 2 + b) * chi_r;
                        let out_base = ((l_in * 2 + a) * 2 + b) * chi_r;
                        for r_out in 0..chi_r {
                            let rrow = r_out * chi_r;
                            let mut acc = 0.0;
                            for r_in in 0..chi_r {
                                acc += rmat[rrow + r_in] * psi[psi_base + r_in];
                            }
                            t1c[out_base + r_out] = acc;
                        }
                    }
                }
            }
        };
        let mut blocks: Vec<(usize, &mut [f64])> = t1
            .chunks_mut(block)
            .enumerate()
            .filter(|(c2, _)| live_r.contains(c2))
            .collect();
        if nthreads <= 1 || blocks.len() < 2 {
            for (c2, t1c) in blocks.iter_mut() {
                step1(*c2, t1c);
            }
        } else {
            let per = blocks.len().div_ceil(nthreads).max(1);
            std::thread::scope(|sc| {
                for chunk in blocks.chunks_mut(per) {
                    let step1 = &step1;
                    sc.spawn(move || {
                        for (c2, t1c) in chunk.iter_mut() {
                            step1(*c2, t1c);
                        }
                    });
                }
            });
        }
    }
    // Step 2: t2[c1'][l_in][a][t][r_out] = sum_{b,c2} t1[c2][l_in][a][b][r_out] * W2[c1',c2,t,b]
    let mut t2 = vec![0.0; d_mid * chi_l * 2 * 2 * chi_r];
    for c1p in 0..d_mid {
        for c2 in 0..d_r {
            for t in 0..2 {
                for b in 0..2 {
                    let wv = w2.get(c1p, c2, t, b);
                    if wv == 0.0 {
                        continue;
                    }
                    for l_in in 0..chi_l {
                        for a in 0..2 {
                            let src = (((c2 * chi_l + l_in) * 2 + a) * 2 + b) * chi_r;
                            let dst = (((c1p * chi_l + l_in) * 2 + a) * 2 + t) * chi_r;
                            for r_out in 0..chi_r {
                                t2[dst + r_out] += wv * t1[src + r_out];
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 3: t3[c1][l_in][s][t][r_out] = sum_{a,c1'} t2[c1'][l_in][a][t][r_out] * W1[c1,c1',s,a]
    let mut t3 = vec![0.0; d_l * chi_l * 2 * 2 * chi_r];
    for c1 in 0..d_l {
        for c1p in 0..d_mid {
            for s in 0..2 {
                for a in 0..2 {
                    let wv = w1.get(c1, c1p, s, a);
                    if wv == 0.0 {
                        continue;
                    }
                    for l_in in 0..chi_l {
                        for t in 0..2 {
                            let src = (((c1p * chi_l + l_in) * 2 + a) * 2 + t) * chi_r;
                            let dst = (((c1 * chi_l + l_in) * 2 + s) * 2 + t) * chi_r;
                            for r_out in 0..chi_r {
                                t3[dst + r_out] += wv * t2[src + r_out];
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 4: out[l_out][s][t][r_out] = sum_{c1,l_in} L[c1][l_out,l_in] * t3[c1][l_in][s][t][r_out]
    // over the LIVE left channels, threaded over DISJOINT l_out rows; the c1 and l_in sums
    // keep their serial order inside a row, so every thread count gives the same bits.
    let mut out = vec![0.0; chi_l * 2 * 2 * chi_r];
    let row = 2 * 2 * chi_r;
    let step4 = |l_out: usize, outrow: &mut [f64]| {
        for &c1 in live_l {
            let lmat = &left[c1];
            let lrow = l_out * chi_l;
            for s in 0..2 {
                for t in 0..2 {
                    let out_base = (s * 2 + t) * chi_r;
                    for r_out in 0..chi_r {
                        let mut acc = 0.0;
                        for l_in in 0..chi_l {
                            let src = (((c1 * chi_l + l_in) * 2 + s) * 2 + t) * chi_r;
                            acc += lmat[lrow + l_in] * t3[src + r_out];
                        }
                        outrow[out_base + r_out] += acc;
                    }
                }
            }
        }
    };
    if nthreads <= 1 || chi_l < 2 {
        for (l_out, outrow) in out.chunks_mut(row).enumerate() {
            step4(l_out, outrow);
        }
    } else {
        let per = chi_l.div_ceil(nthreads).max(1);
        std::thread::scope(|sc| {
            for (ci, chunk) in out.chunks_mut(per * row).enumerate() {
                let step4 = &step4;
                sc.spawn(move || {
                    for (i, outrow) in chunk.chunks_mut(row).enumerate() {
                        step4(ci * per + i, outrow);
                    }
                });
            }
        });
    }
    out
}

