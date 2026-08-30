//! The Hubbard MPO in interleaved Jordan–Wigner order (site `2s`=`(s,up)`, `2s+1`=`(s,down)`,
//! 0-indexed), and the dense contraction used to gate it (G1 — `Q8_MPS_PREREG.md` §3, §1's "JW
//! string of length exactly 1" fact).
//!
//! THE DERIVATION (so the channel graph below is checked, not asserted). Standard JW,
//! `c_j = (Z_1..Z_{j-1}) sigma-_j`. For `p<q`, working the string through:
//! `c†_p c_q = sigma+_p . (Z_{p+1}..Z_{q-1}) . sigma-_q`, and for the reverse,
//! `c†_q c_p = sigma-_p . (Z_{p+1}..Z_{q-1}) . sigma+_q` (using `Z.sigma- = sigma-` and
//! `sigma+.Z = sigma+` as MATRIX identities on this file's 2x2 operators — the string factor
//! at the operator's OWN site is trivial, which is why only the sites STRICTLY between carry a
//! `Z`). A real-space nearest-neighbour hop `(s,sigma)-(s+1,sigma)` is JW sites `j, j+2` for
//! EITHER spin (up: `j=2s,q=2s+2`; down: `j=2s+1,q=2s+3`) — always exactly one site strictly
//! between (`j+1`), so every hop in this model is a 3-site window: open, one `Z`, close.
//!
//! THE CHANNEL GRAPH (bond dimension 7), used identically at every site — no per-position
//! boundary tensor is built; the left/right boundary conditions fall out for free from starting
//! the contraction in channel `START` alone and reading off only channel `FINISH` at the end
//! (`dense_from_mpo`).
//!
//! ```text
//! START --I---------------------------> START           (nothing pending)
//! START --(-mu)*n-----------------------> FINISH          (on-site potential, one site)
//! START --(-t)*sigma+-------------------> PEND_CD         (open an up-branch hop)
//! START --(-t)*sigma-------------------> PEND_CM         (open a down-branch hop)
//! PEND_CD --Z----------------------------> STR_CM          (string site of an up-branch hop)
//! PEND_CM --Z----------------------------> STR_CD          (string site of a down-branch hop)
//! STR_CM --sigma-------------------------> FINISH          (close an up-branch hop)
//! STR_CD --sigma+------------------------> FINISH          (close a down-branch hop)
//! START --n (odd/up site only)-----------> PEND_N          (open the on-site interaction)
//! PEND_N --U*n (even/down site only)-----> FINISH          (close the interaction)
//! ```
//!
//! The interaction is the one non-uniform piece: it opens only at an up (even-index, 0-indexed)
//! spin-orbital and closes only at the immediately following down spin-orbital — a real up-down
//! pair on the same chain site, never a down-up pair straddling two different chain sites.

use crate::ops::{kron, CD2, CM2, Op2, I2, N2, Z2};

pub const D_BOND: usize = 7;
pub const START: usize = 0;
const PEND_CD: usize = 1;
const PEND_CM: usize = 2;
const STR_CM: usize = 3;
const STR_CD: usize = 4;
pub const FINISH: usize = 5;
const PEND_N: usize = 6;

type Edge = (usize, usize, Op2, f64);

/// `is_up_orbital`: true at JW site `2s` (0-indexed), i.e. the interaction-opening half of a
/// chain site's pair.
fn bulk_edges(is_up_orbital: bool, t: f64, u: f64, mu: f64) -> Vec<Edge> {
    let mut e = vec![
        (START, START, I2, 1.0),
        // Once a term completes, it must propagate as pure identity for every remaining site —
        // without this self-loop, a term completing before the LAST site (e.g. this chain's
        // up-spin hop, which closes two sites early) is silently dropped on the next site's
        // contraction, while a term that happens to close exactly at the last site survives by
        // accident. Caught by G0-1/G1a: N=2, U=0 read -1.0 (only the down-hop) instead of -2.0.
        (FINISH, FINISH, I2, 1.0),
        (START, FINISH, N2, -mu),
        (START, PEND_CD, CD2, -t),
        (START, PEND_CM, CM2, -t),
        (PEND_CD, STR_CM, Z2, 1.0),
        (PEND_CM, STR_CD, Z2, 1.0),
        (STR_CM, FINISH, CM2, 1.0),
        (STR_CD, FINISH, CD2, 1.0),
    ];
    if is_up_orbital {
        e.push((START, PEND_N, N2, 1.0));
    } else {
        e.push((PEND_N, FINISH, N2, u));
    }
    e
}

/// The per-site local MPO tensor, dense and small (`D_BOND x D_BOND x 2 x 2 = 196` entries,
/// mostly zero): `w[((c*D_BOND+c2)*2+s)*2+sp]` is the matrix element on channel edge `c -> c2`
/// between physical states `s` (row) and `sp` (column). What the sweep engine (`mps.rs`)
/// contracts against; `dense_from_mpo` below stays the independent gate-only path.
pub fn w_dense(is_up_orbital: bool, t: f64, u: f64, mu: f64) -> Vec<f64> {
    let mut w = vec![0.0; D_BOND * D_BOND * 4];
    for (from, to, block, weight) in bulk_edges(is_up_orbital, t, u, mu) {
        for s in 0..2 {
            for sp in 0..2 {
                w[((from * D_BOND + to) * 2 + s) * 2 + sp] += weight * block[s][sp];
            }
        }
    }
    w
}

/// Dense `2^(2*sites) x 2^(2*sites)` contraction of the MPO, row-major. Gate-only (G1): the
/// exponential blow-up is only ever asked for at `sites<=4` (dim<=256).
///
/// `mu=0.0` gives the bare `H`; `mu=U/2` gives the working `H'` of `Q8_MPS_PREREG.md` §2. Basis
/// index bit `q` (0-indexed from the LSB) is the occupation of JW site `q`, checked against
/// `ops::kron`'s doc comment.
pub fn dense_from_mpo(sites: usize, t: f64, u: f64, mu: f64) -> Vec<f64> {
    let l = 2 * sites;
    let mut acc: Vec<Option<Vec<f64>>> = (0..D_BOND).map(|_| None).collect();
    acc[START] = Some(vec![1.0]);
    let mut dim = 1usize;

    for j in 0..l {
        let edges = bulk_edges(j % 2 == 0, t, u, mu);
        let mut new_acc: Vec<Option<Vec<f64>>> = (0..D_BOND).map(|_| None).collect();
        for &(from, to, block, weight) in &edges {
            let Some(inner) = &acc[from] else { continue };
            let scaled: Op2 = [
                [block[0][0] * weight, block[0][1] * weight],
                [block[1][0] * weight, block[1][1] * weight],
            ];
            let contrib = kron(&scaled, inner, dim);
            match &mut new_acc[to] {
                Some(existing) => {
                    for (e, c) in existing.iter_mut().zip(contrib.iter()) {
                        *e += c;
                    }
                }
                None => new_acc[to] = Some(contrib),
            }
        }
        acc = new_acc;
        dim *= 2;
    }

    acc[FINISH].clone().expect("no path reached FINISH — the channel graph is broken")
}

// ------------------------------------------------------------------ General MPO & Electronic Hamiltonian

/// A single site tensor of a Matrix Product Operator (MPO).
/// Left bond dimension `d_l`, right bond dimension `d_r`, physical dimension 2.
#[derive(Clone, Debug)]
pub struct MpoSite {
    pub d_l: usize,
    pub d_r: usize,
    /// Flattened row-major entries `[c_l, c_r, s, sp]` of shape `d_l x d_r x 2 x 2`.
    pub data: Vec<f64>,
}

impl MpoSite {
    pub fn new(d_l: usize, d_r: usize, data: Vec<f64>) -> Self {
        assert_eq!(data.len(), d_l * d_r * 4, "MpoSite data size mismatch");
        Self { d_l, d_r, data }
    }

    pub fn zeros(d_l: usize, d_r: usize) -> Self {
        Self {
            d_l,
            d_r,
            data: vec![0.0; d_l * d_r * 4],
        }
    }

    #[inline]
    pub fn get(&self, cl: usize, cr: usize, s: usize, sp: usize) -> f64 {
        self.data[((cl * self.d_r + cr) * 2 + s) * 2 + sp]
    }

    #[inline]
    pub fn set(&mut self, cl: usize, cr: usize, s: usize, sp: usize, val: f64) {
        self.data[((cl * self.d_r + cr) * 2 + s) * 2 + sp] = val;
    }
}

/// A 1D Matrix Product Operator across an L-site chain.
#[derive(Clone, Debug)]
pub struct Mpo {
    pub sites: Vec<MpoSite>,
}

#[inline]
fn mat_mul_2x2(a: &Op2, b: &Op2) -> Op2 {
    [
        [
            a[0][0] * b[0][0] + a[0][1] * b[1][0],
            a[0][0] * b[0][1] + a[0][1] * b[1][1],
        ],
        [
            a[1][0] * b[0][0] + a[1][1] * b[1][0],
            a[1][0] * b[0][1] + a[1][1] * b[1][1],
        ],
    ]
}

#[inline]
fn jw_factor(i: usize, is_dagger: bool, k: usize) -> Op2 {
    if k < i {
        Z2
    } else if k == i {
        if is_dagger {
            CD2
        } else {
            CM2
        }
    } else {
        I2
    }
}

fn term_operator_at_site(factors: &[(usize, bool)], k: usize) -> Op2 {
    let mut op = I2;
    for &(i, is_dagger) in factors {
        let f = jw_factor(i, is_dagger, k);
        op = mat_mul_2x2(&op, &f);
    }
    op
}

#[derive(Clone)]
struct Term {
    coeff: f64,
    ops: Vec<Op2>,
}

impl Mpo {
    /// Build an MPO from a list of local operator strings and compress it with SVD.
    fn from_terms(l: usize, terms_in: Vec<Term>) -> Self {
        if l == 0 {
            return Mpo { sites: vec![] };
        }

        // Merge identical operator strings
        use std::collections::BTreeMap;
        let mut grouped: BTreeMap<Vec<[i32; 4]>, (f64, Vec<Op2>)> = BTreeMap::new();
        for t in terms_in {
            if t.coeff.abs() < 1e-15 {
                continue;
            }
            let key: Vec<[i32; 4]> = t
                .ops
                .iter()
                .map(|op| {
                    [
                        (op[0][0] * 2.0).round() as i32,
                        (op[0][1] * 2.0).round() as i32,
                        (op[1][0] * 2.0).round() as i32,
                        (op[1][1] * 2.0).round() as i32,
                    ]
                })
                .collect();
            match grouped.get_mut(&key) {
                Some(entry) => entry.0 += t.coeff,
                None => {
                    grouped.insert(key, (t.coeff, t.ops));
                }
            }
        }

        let terms: Vec<Term> = grouped
            .into_values()
            .filter(|(c, _)| c.abs() > 1e-15)
            .map(|(coeff, ops)| Term { coeff, ops })
            .collect();

        let m = terms.len();
        if m == 0 {
            // Trivial zero MPO
            let sites = (0..l).map(|_| MpoSite::zeros(1, 1)).collect();
            return Mpo { sites };
        }

        // 1. Site 0 forward SVD
        let mut x0 = vec![0.0f64; m * 4];
        for (t_idx, t) in terms.iter().enumerate() {
            for s in 0..2 {
                for sp in 0..2 {
                    x0[t_idx * 4 + s * 2 + sp] = t.coeff * t.ops[0][s][sp];
                }
            }
        }

        let svd0 = crate::svd::jacobi_svd(&x0, m, 4);
        let s0_max = svd0.s.first().copied().unwrap_or(0.0).max(1e-15);
        let mut r1 = 0;
        for &s in &svd0.s {
            if s > 1e-13 * s0_max {
                r1 += 1;
            }
        }
        r1 = r1.max(1);

        let mut w0 = MpoSite::zeros(1, r1);
        for alpha in 0..r1 {
            for s in 0..2 {
                for sp in 0..2 {
                    w0.set(0, alpha, s, sp, svd0.v[alpha][s * 2 + sp]);
                }
            }
        }

        let mut c_coeffs: Vec<Vec<f64>> = (0..m)
            .map(|t| (0..r1).map(|alpha| svd0.u[alpha][t] * svd0.s[alpha]).collect())
            .collect();

        let mut mpo_sites = vec![w0];
        let mut r_prev = r1;

        // 2. Intermediate sites 1..L-2
        for k in 1..(l - 1) {
            let cols = 4 * r_prev;
            let mut xk = vec![0.0f64; m * cols];
            for (t_idx, t) in terms.iter().enumerate() {
                for alpha in 0..r_prev {
                    let ca = c_coeffs[t_idx][alpha];
                    if ca == 0.0 {
                        continue;
                    }
                    for s in 0..2 {
                        for sp in 0..2 {
                            let op_val = t.ops[k][s][sp];
                            if op_val == 0.0 {
                                continue;
                            }
                            let col = (alpha * 2 + s) * 2 + sp;
                            xk[t_idx * cols + col] += ca * op_val;
                        }
                    }
                }
            }

            let svd_k = crate::svd::jacobi_svd(&xk, m, cols);
            let sk_max = svd_k.s.first().copied().unwrap_or(0.0).max(1e-15);
            let mut r_next = 0;
            for &s in &svd_k.s {
                if s > 1e-13 * sk_max {
                    r_next += 1;
                }
            }
            r_next = r_next.max(1);

            let mut wk = MpoSite::zeros(r_prev, r_next);
            for alpha in 0..r_prev {
                for beta in 0..r_next {
                    for s in 0..2 {
                        for sp in 0..2 {
                            let col = (alpha * 2 + s) * 2 + sp;
                            wk.set(alpha, beta, s, sp, svd_k.v[beta][col]);
                        }
                    }
                }
            }

            c_coeffs = (0..m)
                .map(|t| (0..r_next).map(|beta| svd_k.u[beta][t] * svd_k.s[beta]).collect())
                .collect();

            mpo_sites.push(wk);
            r_prev = r_next;
        }

        // 3. Site L-1 (last site)
        if l > 1 {
            let mut w_last = MpoSite::zeros(r_prev, 1);
            for alpha in 0..r_prev {
                for s in 0..2 {
                    for sp in 0..2 {
                        let mut sum = 0.0f64;
                        for (t_idx, t) in terms.iter().enumerate() {
                            let ca = c_coeffs[t_idx][alpha];
                            let op_val = t.ops[l - 1][s][sp];
                            sum += ca * op_val;
                        }
                        w_last.set(alpha, 0, s, sp, sum);
                    }
                }
            }
            mpo_sites.push(w_last);
        }

        // 4. Backward SVD compression sweep (sites L-1 down to 1)
        for k in (1..l).rev() {
            let r_k = mpo_sites[k].d_l;
            let r_kp1 = mpo_sites[k].d_r;
            let cols = 4 * r_kp1;
            let mut yk = vec![0.0f64; r_k * cols];
            for alpha in 0..r_k {
                for beta in 0..r_kp1 {
                    for s in 0..2 {
                        for sp in 0..2 {
                            let col = (beta * 2 + s) * 2 + sp;
                            yk[alpha * cols + col] = mpo_sites[k].get(alpha, beta, s, sp);
                        }
                    }
                }
            }

            let svd_b = crate::svd::jacobi_svd(&yk, r_k, cols);
            let sb_max = svd_b.s.first().copied().unwrap_or(0.0).max(1e-15);
            let mut r_new = 0;
            for &s in &svd_b.s {
                if s > 1e-13 * sb_max {
                    r_new += 1;
                }
            }
            r_new = r_new.max(1);

            let mut new_wk = MpoSite::zeros(r_new, r_kp1);
            for alpha_p in 0..r_new {
                for beta in 0..r_kp1 {
                    for s in 0..2 {
                        for sp in 0..2 {
                            let col = (beta * 2 + s) * 2 + sp;
                            new_wk.set(alpha_p, beta, s, sp, svd_b.v[alpha_p][col]);
                        }
                    }
                }
            }
            mpo_sites[k] = new_wk;

            // Contract (U * S) into left neighbour's right bond
            let r_km1 = mpo_sites[k - 1].d_l;
            let mut new_wkm1 = MpoSite::zeros(r_km1, r_new);
            for gamma in 0..r_km1 {
                for alpha_p in 0..r_new {
                    let s_val = svd_b.s[alpha_p];
                    for s in 0..2 {
                        for sp in 0..2 {
                            let mut acc = 0.0f64;
                            for alpha in 0..r_k {
                                let old_val = mpo_sites[k - 1].get(gamma, alpha, s, sp);
                                if old_val != 0.0 {
                                    acc += old_val * svd_b.u[alpha_p][alpha] * s_val;
                                }
                            }
                            new_wkm1.set(gamma, alpha_p, s, sp, acc);
                        }
                    }
                }
            }
            mpo_sites[k - 1] = new_wkm1;
        }

        Mpo { sites: mpo_sites }
    }

    /// Construct a 1D Matrix Product Operator from 1-electron and 2-electron molecular orbital integrals.
    /// `h` is `K x K` (1-electron integrals), `g` is `K^4` (chemist notation `(pq|rs)` indexed `[(p*K+q)*K^2 + r*K+s]`).
    /// The resulting MPO has `2*K` sites in interleaved Jordan–Wigner ordering: `2p` = `(p,up)`, `2p+1` = `(p,down)`.
    pub fn from_electronic_integrals(n_orb: usize, h: &[f64], g: &[f64]) -> Self {
        let l = 2 * n_orb;
        let mut raw_terms = Vec::new();

        // 1-body terms: sum_{pq, sigma} h_{pq} c_{p sigma}^\dagger c_{q sigma}
        for p in 0..n_orb {
            for q in 0..n_orb {
                let hpq = h[p * n_orb + q];
                if hpq.abs() < 1e-15 {
                    continue;
                }
                for sigma in 0..2 {
                    let i = 2 * p + sigma;
                    let j = 2 * q + sigma;
                    let factors = [(i, true), (j, false)];
                    let ops: Vec<Op2> = (0..l).map(|k| term_operator_at_site(&factors, k)).collect();
                    raw_terms.push(Term { coeff: hpq, ops });
                }
            }
        }

        // 2-body terms: 1/2 sum_{pqrs, sigma tau} g_{pqrs} c_{p sigma}^\dagger c_{r tau}^\dagger c_{s tau} c_{q sigma}
        for p in 0..n_orb {
            for q in 0..n_orb {
                for r in 0..n_orb {
                    for s in 0..n_orb {
                        let gpqrs = g[(p * n_orb + q) * n_orb * n_orb + (r * n_orb + s)];
                        if gpqrs.abs() < 1e-15 {
                            continue;
                        }
                        let coeff = 0.5 * gpqrs;
                        for sigma in 0..2 {
                            for tau in 0..2 {
                                let i = 2 * p + sigma;
                                let j = 2 * q + sigma;
                                let k = 2 * r + tau;
                                let l_spin = 2 * s + tau;
                                if i == k || l_spin == j {
                                    continue;
                                }
                                let factors = [(i, true), (k, true), (l_spin, false), (j, false)];
                                let ops: Vec<Op2> =
                                    (0..l).map(|m| term_operator_at_site(&factors, m)).collect();
                                raw_terms.push(Term { coeff, ops });
                            }
                        }
                    }
                }
            }
        }

        Self::from_terms(l, raw_terms)
    }

    /// Construct an MPO for the 1D Hubbard model on `sites` chain sites (2*sites spin-orbitals).
    pub fn from_hubbard(sites: usize, t: f64, u: f64, mu: f64) -> Self {
        let l = 2 * sites;
        let mut raw_terms = Vec::new();

        // On-site chemical potential: -mu * sum_i n_i
        if mu != 0.0 {
            for i in 0..l {
                let factors = [(i, true), (i, false)];
                let ops: Vec<Op2> = (0..l).map(|k| term_operator_at_site(&factors, k)).collect();
                raw_terms.push(Term { coeff: -mu, ops });
            }
        }

        // Hopping: -t sum_{<cs, cs'>, sigma} (c_{cs, sigma}^\dagger c_{cs', sigma} + h.c.)
        for cs in 0..(sites - 1) {
            for sigma in 0..2 {
                let i = 2 * cs + sigma;
                let j = 2 * (cs + 1) + sigma;
                // Forward hop
                let fwd = [(i, true), (j, false)];
                let ops_fwd: Vec<Op2> = (0..l).map(|k| term_operator_at_site(&fwd, k)).collect();
                raw_terms.push(Term { coeff: -t, ops: ops_fwd });
                // Backward hop
                let bwd = [(j, true), (i, false)];
                let ops_bwd: Vec<Op2> = (0..l).map(|k| term_operator_at_site(&bwd, k)).collect();
                raw_terms.push(Term { coeff: -t, ops: ops_bwd });
            }
        }

        // On-site interaction: U sum_{cs} n_{cs, up} n_{cs, down}
        if u != 0.0 {
            for cs in 0..sites {
                let i = 2 * cs;
                let j = 2 * cs + 1;
                let factors = [(i, true), (i, false), (j, true), (j, false)];
                let ops: Vec<Op2> = (0..l).map(|k| term_operator_at_site(&factors, k)).collect();
                raw_terms.push(Term { coeff: u, ops });
            }
        }

        Self::from_terms(l, raw_terms)
    }

    /// Dense `2^L x 2^L` contraction of the MPO, row-major.
    pub fn dense(&self) -> Vec<f64> {
        let l = self.sites.len();
        if l == 0 {
            return vec![1.0];
        }
        let d_r0 = self.sites[0].d_r;
        let mut acc: Vec<Option<Vec<f64>>> = (0..d_r0).map(|_| None).collect();
        for b in 0..d_r0 {
            let mut mat = vec![0.0; 4];
            for s in 0..2 {
                for sp in 0..2 {
                    mat[s * 2 + sp] = self.sites[0].get(0, b, s, sp);
                }
            }
            acc[b] = Some(mat);
        }
        let mut dim = 2usize;

        for j in 1..l {
            let site = &self.sites[j];
            let mut new_acc: Vec<Option<Vec<f64>>> = (0..site.d_r).map(|_| None).collect();
            for a in 0..site.d_l {
                let Some(inner) = &acc[a] else { continue };
                for b in 0..site.d_r {
                    let block: Op2 = [
                        [site.get(a, b, 0, 0), site.get(a, b, 0, 1)],
                        [site.get(a, b, 1, 0), site.get(a, b, 1, 1)],
                    ];
                    let is_zero = block[0][0] == 0.0
                        && block[0][1] == 0.0
                        && block[1][0] == 0.0
                        && block[1][1] == 0.0;
                    if is_zero {
                        continue;
                    }
                    let contrib = kron(&block, inner, dim);
                    match &mut new_acc[b] {
                        Some(existing) => {
                            for (e, c) in existing.iter_mut().zip(contrib.iter()) {
                                *e += c;
                            }
                        }
                        None => new_acc[b] = Some(contrib),
                    }
                }
            }
            acc = new_acc;
            dim *= 2;
        }

        acc[0].clone().unwrap_or_else(|| vec![0.0; dim * dim])
    }

    pub fn len(&self) -> usize {
        self.sites.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sites.is_empty()
    }

    pub fn bond_dims(&self) -> Vec<usize> {
        self.sites.iter().map(|s| s.d_r).collect()
    }

    /// Evaluate `<psi | MPO | psi> / <psi | psi>` on any state given by `TensorSite`s.
    pub fn expectation(&self, tensors: &[crate::mps::TensorSite]) -> f64 {
        let norm = crate::observables::norm_squared(tensors);
        if norm == 0.0 {
            return 0.0;
        }
        let l = self.sites.len();
        assert_eq!(tensors.len(), l);
        let mut left_env = crate::mps::trivial_left_env_mpo(self.sites[0].d_l);
        for j in 0..l {
            left_env = crate::mps::grow_left_mpo(&left_env, &self.sites[j], &tensors[j]);
        }
        left_env[0][0] / norm
    }
}

