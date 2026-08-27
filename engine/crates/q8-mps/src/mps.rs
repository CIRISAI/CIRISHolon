//! MPS tensors, MPO environments, and the matrix-free two-site effective Hamiltonian —
//! `Q8_MPS_PREREG.md` §2. `chi` here is one bond's declared LEDGER: `dmrg.rs` is where it gets
//! enforced (truncation, discarded weight, the typed refusal); this module only builds and
//! contracts, it never truncates.

use crate::mpo;

/// One MPS site tensor, physical dimension 2 (occupied/empty), flat `[s][l][r]` layout.
#[derive(Clone)]
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
