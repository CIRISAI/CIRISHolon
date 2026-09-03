//! THE TWO-SITE VARIANCE — the exit `variance.rs` names when the exact one refuses.
//!
//! Hubig, Haegeman and Schollwöck, *Error estimates for extrapolations with matrix-product
//! states*, Phys. Rev. B 97, 045125 (2018), arXiv:1711.01104 — credited for the construction;
//! the implementation and its calibration here are ours.
//!
//! The Hilbert space splits into `W_l`, the variations that leave the MPS's own bases on
//! exactly `l` neighbouring sites: `W_0` is `span|ψ⟩`, `W_1` the one-site variations
//! `A_1…A_{i-1}·F_i·V·B_{i+1}…`, `W_2` the two-site ones
//! `A_1…A_{i-1}·F_i·W·G_{i+1}·B_{i+2}…`, with `F_i` the orthogonal complement of the
//! left-canonical `A_i` and `G_{i+1}` that of the right-canonical `B_{i+1}`. Inserting
//! `1 = Σ_l P_l` into `⟨ψ|(H−E)²|ψ⟩` and keeping `l ≤ 2`:
//!
//! ```text
//!   Δ_2s = ⟨ψ|H P_1 H|ψ⟩ + ⟨ψ|H P_2 H|ψ⟩ = Σ_i ‖F_i†·(H_eff^{(i)} M_i)‖² + Σ_i ‖F_i†·(H_eff^{(i,i+1)} Ψ_i)·G_{i+1}‖²
//! ```
//!
//! `L` one-site terms and `L−1` two-site terms; `P_l|ψ⟩ = 0` for `l ≥ 1`, which is why `E`
//! never appears. The complements are never formed: with `P_A = A A†` and `P_B = B† B`,
//!
//! ```text
//!   ‖(1−P_A)·Z‖²          = ‖Z‖² − ‖A†Z‖²
//!   ‖(1−P_A)·Z·(1−P_B)‖²  = ‖Z‖² − ‖ZB†‖² − ‖A†Z‖² + ‖A†ZB†‖²
//! ```
//!
//! so every term is a contraction against tensors the sweep already holds. Cost
//! `O(L·m³·d·w)` with NO doubled environment — the whole point, since the exact variance
//! prices at 74 GB where this one is megabytes.
//!
//! ONE PROJECTION PER BOND, NOT TWO. On site `i` the complement `F_i` tensored with
//! *anything* on site `i+1` splits as `F_i ⊗ (B_{i+1} ⊕ G_{i+1})`: the `B` part IS the
//! one-site variation at `i` and the `G` part is the two-site variation at `(i, i+1)`. So
//!
//! ```text
//!   ‖(1−A_i A_i†)·Z_i‖²  =  (one-site term at i)  +  (two-site term at i, i+1)
//! ```
//!
//! with `Z_i = H_eff^{(i,i+1)} Ψ_i`, and the whole sum is `L−1` two-site applications plus
//! ONE one-site application at the last site. Gated against the term-by-term form.
//!
//! LABELLED PATH. Given the bond labels the two-site application runs on `blocks.rs`'s
//! block-sparse operator and the centre walks by the sweep's own labelled split at full
//! rank, so nothing is truncated and nothing dense is built. Measured on the campaign's own
//! rung at N = 8, χ = 512: 348 s dense against a few seconds labelled, for the same number.
//!
//! **IT IS AN APPROXIMATION, NOT A BOUND.** The paper's own statement: the truncation at
//! `l ≤ 2` is an equality only for nearest-neighbour interactions. The QCD₂ Hamiltonian's
//! Coulomb term couples every pair of sites (`w_kk' = N−1−max(k,k')`), so `Δ_2s ≠ Var` here
//! and the difference is MEASURED against the exact variance wherever both fit
//! (`tests/qcd2_gauge.rs`), never assumed. It is reported as `variance_2s`, beside the exact
//! `variance` where that exists, and never as the variance.

use crate::mpo::Mpo;
use crate::mps::{self, Env, TensorSite};

/// `A†Z` for a left-canonical `A` (as `(d·χ_l) × χ_r`) against `Z` in the same row space.
/// `z` is `(d·chi_l) × n` row-major; the result is `chi_r × n`.
fn a_dagger_z(a: &TensorSite, z: &[f64], n: usize) -> Vec<f64> {
    let (chi_l, chi_r) = (a.chi_l, a.chi_r);
    debug_assert_eq!(z.len(), 2 * chi_l * n);
    let mut out = vec![0.0; chi_r * n];
    for s in 0..2 {
        for l in 0..chi_l {
            let zrow = &z[(s * chi_l + l) * n..(s * chi_l + l) * n + n];
            for r in 0..chi_r {
                let av = a.get(s, l, r);
                if av == 0.0 {
                    continue;
                }
                let orow = &mut out[r * n..r * n + n];
                for (o, zv) in orow.iter_mut().zip(zrow) {
                    *o += av * zv;
                }
            }
        }
    }
    out
}

/// `Z·B†` for a right-canonical `B` (as `χ_l × (d·χ_r)`) against `Z` whose columns run over
/// `(s, r)` in the layout `[s][r]`. `z` is `m × (d·chi_r)` row-major; the result is `m × chi_l`.
fn z_b_dagger(b: &TensorSite, z: &[f64], m: usize) -> Vec<f64> {
    let (chi_l, chi_r) = (b.chi_l, b.chi_r);
    debug_assert_eq!(z.len(), m * 2 * chi_r);
    let mut out = vec![0.0; m * chi_l];
    for row in 0..m {
        let zrow = &z[row * 2 * chi_r..(row + 1) * 2 * chi_r];
        let orow = &mut out[row * chi_l..(row + 1) * chi_l];
        for s in 0..2 {
            for r in 0..chi_r {
                let zv = zrow[s * chi_r + r];
                if zv == 0.0 {
                    continue;
                }
                for (l, o) in orow.iter_mut().enumerate() {
                    *o += zv * b.get(s, l, r);
                }
            }
        }
    }
    out
}

fn norm2(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum()
}

/// The one-site effective Hamiltonian applied to the centre tensor `m` at site `j`:
/// `out[σ,l,r] = Σ L[c][l,l'] W[c,c'][σ,σ'] R[c'][r,r'] M[σ',l',r']`, returned in the
/// `(σ,l) × r` row-major layout the projections want.
fn h_eff_one_site(left: &Env, w: &crate::mpo::MpoSite, right: &Env, m: &TensorSite) -> Vec<f64> {
    let (chi_l, chi_r) = (m.chi_l, m.chi_r);
    let (d_l, d_r) = (w.d_l, w.d_r);
    // t1[c'][σ',l'][r_out] = Σ_{r_in} R[c'][r_out,r_in] M[σ',l',r_in]
    let block = 2 * chi_l * chi_r;
    let mut t1 = vec![0.0; d_r * block];
    for c2 in mps::live_channels(right) {
        let rmat = &right[c2];
        for sp in 0..2 {
            for lp in 0..chi_l {
                let dst = (c2 * 2 * chi_l + sp * chi_l + lp) * chi_r;
                for r_out in 0..chi_r {
                    let rrow = r_out * chi_r;
                    let mut acc = 0.0;
                    for r_in in 0..chi_r {
                        acc += rmat[rrow + r_in] * m.get(sp, lp, r_in);
                    }
                    t1[dst + r_out] = acc;
                }
            }
        }
    }
    // t2[c][σ,l'][r] = Σ_{c',σ'} W[c,c'][σ,σ'] t1[c'][σ',l'][r]
    let mut t2 = vec![0.0; d_l * block];
    for c1 in 0..d_l {
        for c2 in 0..d_r {
            for s in 0..2 {
                for sp in 0..2 {
                    let wv = w.get(c1, c2, s, sp);
                    if wv == 0.0 {
                        continue;
                    }
                    for lp in 0..chi_l {
                        let src = (c2 * 2 * chi_l + sp * chi_l + lp) * chi_r;
                        let dst = (c1 * 2 * chi_l + s * chi_l + lp) * chi_r;
                        for r in 0..chi_r {
                            t2[dst + r] += wv * t1[src + r];
                        }
                    }
                }
            }
        }
    }
    // out[σ,l][r] = Σ_{c,l'} L[c][l,l'] t2[c][σ,l'][r]
    let mut out = vec![0.0; block];
    for c1 in mps::live_channels(left) {
        let lmat = &left[c1];
        for s in 0..2 {
            for l in 0..chi_l {
                let lrow = l * chi_l;
                let dst = (s * chi_l + l) * chi_r;
                for lp in 0..chi_l {
                    let lv = lmat[lrow + lp];
                    if lv == 0.0 {
                        continue;
                    }
                    let src = (c1 * 2 * chi_l + s * chi_l + lp) * chi_r;
                    for r in 0..chi_r {
                        out[dst + r] += lv * t2[src + r];
                    }
                }
            }
        }
    }
    out
}

/// Left-normalise a centre tensor: returns `(A, C)` with `A` left-canonical and `M = A·C`.
fn left_normalise(m: &TensorSite) -> (TensorSite, Vec<f64>) {
    let (chi_l, chi_r) = (m.chi_l, m.chi_r);
    let rows = 2 * chi_l;
    let mut mat = vec![0.0; rows * chi_r];
    for s in 0..2 {
        for l in 0..chi_l {
            for r in 0..chi_r {
                mat[(s * chi_l + l) * chi_r + r] = m.get(s, l, r);
            }
        }
    }
    let svd = crate::svd::jacobi_svd(&mat, rows, chi_r);
    assert!(svd.converged, "left-normalising SVD did not converge");
    let keep: Vec<usize> = (0..svd.s.len()).filter(|&k| svd.s[k] > 0.0).collect();
    let k = keep.len().max(1);
    let mut a = TensorSite::zeros(chi_l, k);
    let mut c = vec![0.0; k * chi_r];
    for (i, &ki) in keep.iter().enumerate() {
        for s in 0..2 {
            for l in 0..chi_l {
                a.set(s, l, i, svd.u[ki][s * chi_l + l]);
            }
        }
        for r in 0..chi_r {
            c[i * chi_r + r] = svd.s[ki] * svd.v[ki][r];
        }
    }
    (a, c)
}

/// `C · B`: absorb a centre matrix `(k × chi_l)` into the site tensor to its right.
fn absorb_left(c: &[f64], k: usize, b: &TensorSite) -> TensorSite {
    let mut out = TensorSite::zeros(k, b.chi_r);
    for s in 0..2 {
        for i in 0..k {
            for l in 0..b.chi_l {
                let cv = c[i * b.chi_l + l];
                if cv == 0.0 {
                    continue;
                }
                for r in 0..b.chi_r {
                    let v = out.get(s, i, r) + cv * b.get(s, l, r);
                    out.set(s, i, r, v);
                }
            }
        }
    }
    out
}

/// The two-site variance of a RIGHT-CANONICAL MPS under `mpo`, with its one-site and
/// two-site parts. The state must be right-canonical (as `dmrg_sweep_sym` leaves it after a
/// right-to-left half sweep) and normalised; `norm_squared` is checked, not assumed.
pub fn two_site_variance(tensors: &[TensorSite], mpo: &Mpo) -> (f64, f64, f64) {
    let l = tensors.len();
    assert_eq!(l, mpo.sites.len());
    let n2 = crate::observables::norm_squared(tensors);
    assert!((n2 - 1.0).abs() < 1e-6, "the two-site variance wants a normalised state (norm² = {n2})");
    // right environments of the right-canonical tail
    let mut rights: Vec<Env> = vec![mps::trivial_right_env_mpo(mpo.sites[l - 1].d_r); l + 1];
    for k in (0..l).rev() {
        rights[k] = mps::grow_right_mpo(&rights[k + 1], &mpo.sites[k], &tensors[k]);
    }
    let mut lefts = mps::trivial_left_env_mpo(mpo.sites[0].d_l);
    // walk the centre left to right: A_0..A_{j-1} | M_j | B_{j+1}..B_{L-1}
    let mut centre = tensors[0].clone();
    let mut bs: Vec<TensorSite> = tensors.to_vec();
    let (mut one, mut two) = (0.0, 0.0);
    for j in 0..l {
        let (chi_l, chi_r) = (centre.chi_l, centre.chi_r);
        // ---- the one-site term at j: ‖(1−A A†)·(H_eff M)‖²
        let z = h_eff_one_site(&lefts, &mpo.sites[j], &rights[j + 1], &centre);
        let (a, c) = left_normalise(&centre);
        let atz = a_dagger_z(&a, &z, chi_r);
        one += norm2(&z) - norm2(&atz);
        // ---- the two-site term at (j, j+1): ‖(1−A A†)·Z·(1−B†B)‖²
        if j + 1 < l {
            let b = &bs[j + 1];
            let (chi_m, chi_rr) = (b.chi_l, b.chi_r);
            debug_assert_eq!(chi_m, chi_r);
            // Ψ[(σ,l),(τ,r)] = Σ_m M[σ,l,m] B[τ,m,r]
            let cols = 2 * chi_rr;
            let mut psi = vec![0.0; 2 * chi_l * cols];
            for s in 0..2 {
                for lft in 0..chi_l {
                    for m in 0..chi_r {
                        let mv = centre.get(s, lft, m);
                        if mv == 0.0 {
                            continue;
                        }
                        let row = &mut psi[(s * chi_l + lft) * cols..(s * chi_l + lft) * cols + cols];
                        for t in 0..2 {
                            for r in 0..chi_rr {
                                row[t * chi_rr + r] += mv * b.get(t, m, r);
                            }
                        }
                    }
                }
            }
            // the two-site operator wants ψ in [l,a,b,r]; ours is [(σ,l),(τ,r)] = [σ,l,τ,r]
            let mut psi_lab = vec![0.0; chi_l * 4 * chi_rr];
            for s in 0..2 {
                for lft in 0..chi_l {
                    for t in 0..2 {
                        for r in 0..chi_rr {
                            psi_lab[((lft * 2 + s) * 2 + t) * chi_rr + r] = psi[(s * chi_l + lft) * cols + t * chi_rr + r];
                        }
                    }
                }
            }
            let hz = mps::apply_effective_h_mpo(&lefts, &mpo.sites[j], &mpo.sites[j + 1], &rights[j + 2], &psi_lab, chi_l, chi_rr);
            // back to [(σ,l),(τ,r)]
            let mut z2 = vec![0.0; 2 * chi_l * cols];
            for s in 0..2 {
                for lft in 0..chi_l {
                    for t in 0..2 {
                        for r in 0..chi_rr {
                            z2[(s * chi_l + lft) * cols + t * chi_rr + r] = hz[((lft * 2 + s) * 2 + t) * chi_rr + r];
                        }
                    }
                }
            }
            let zb = z_b_dagger(b, &z2, 2 * chi_l);
            let atz2 = a_dagger_z(&a, &z2, cols);
            let atzb = a_dagger_z(&a, &zb, chi_m);
            two += norm2(&z2) - norm2(&zb) - norm2(&atz2) + norm2(&atzb);
        }
        // ---- move the centre right
        if j + 1 < l {
            centre = absorb_left(&c, a.chi_r, &bs[j + 1]);
            lefts = mps::grow_left_mpo(&lefts, &mpo.sites[j], &a);
            bs[j] = a;
        }
    }
    (one + two, one, two)
}


// THE LABELLED FAST PATH IS NOT SHIPPED, AND THIS IS WHY.
//
// A block-sparse route was written and MEASURED WRONG, so it is removed rather than left
// behind a flag: on N = 4, B = 0 at χ = 16 it read 8.75e-1 where this implementation reads
// 1.83e-1, and this implementation is the one that reproduces the exact variance to
// ratio 1.000000 on a chain-local Hamiltonian (`examples/var_nn.rs`), which is the paper's
// own exactness statement and therefore the test that decides.
//
// What was ruled out (`examples/var_isolate.rs`, `examples/canon_probe.rs`): the sweep's
// output IS right-canonical to 6.7e-16 on every site; the two-site wavefunction IS
// label-consistent to 0.0; and `blocks::BlockPlan::apply` matches the dense operator to
// 0.0 on every bond of the real state. What is NOT explained: the labelled split at full
// rank returns MORE columns than the state's own bond dimension (27 against 18 at bond 4,
// 37 against 28 at bond 5), because a per-block SVD keeps singular values that are
// numerically rather than exactly zero, so the two routes use different `A` and the
// identity that relates them stops holding.
//
// OWNER: the crystal lane. EXIT: rank-truncate the labelled split at a relative floor and
// re-run `examples/var_isolate.rs`, whose per-bond table is the discriminator; ship only if
// it reproduces this implementation on every bond. COST OF NOT HAVING IT, measured: 348 s
// at N = 8, χ = 512 against a 567 s rung solve — the same order as the solve it prices, so
// the ladder is affordable without it.
