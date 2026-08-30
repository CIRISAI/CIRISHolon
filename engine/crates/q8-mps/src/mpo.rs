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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Channel {
    Start,
    Finish,
    LeftCd(usize),
    LeftCm(usize),
    LeftN(usize),
    LeftCdCd(usize, usize),
    LeftCmCm(usize, usize),
    LeftPair(usize, usize),    // CD at x1, CM at x2
    LeftPairRev(usize, usize), // CM at x1, CD at x2
    RightCd(usize),
    RightCm(usize),
}

impl Channel {
    #[inline]
    pub fn idle_op(&self) -> Op2 {
        match self {
            Channel::Start | Channel::Finish => I2,
            Channel::LeftCd(_) | Channel::LeftCm(_) => Z2,
            Channel::LeftN(_)
            | Channel::LeftCdCd(_, _)
            | Channel::LeftCmCm(_, _)
            | Channel::LeftPair(_, _)
            | Channel::LeftPairRev(_, _) => I2,
            Channel::RightCd(_) | Channel::RightCm(_) => Z2,
        }
    }
}

#[inline]
fn canonicalize_and_add_term(
    builder: &mut MpoBuilder,
    mut factors: [(usize, bool); 4],
    len: usize,
    mut coeff: f64,
) {
    if coeff.abs() < 1e-15 || len == 0 {
        return;
    }

    // Sort factors into ascending site order by adjacent transpositions
    let n = len;
    let mut i = 0;
    while i < n {
        let mut swapped = false;
        for j in 0..(n - 1) {
            if factors[j].0 > factors[j + 1].0 {
                factors.swap(j, j + 1);
                coeff = -coeff;
                swapped = true;
            } else if factors[j].0 == factors[j + 1].0 {
                // If same site: ensure (true) is before (false)
                // (false, true) -> c_s c_s^\dagger = 1 - c_s^\dagger c_s
                if !factors[j].1 && factors[j + 1].1 {
                    // Split: coeff * (1 - c^\dagger c)
                    // Part 1: coeff * 1 (remove both factors)
                    let mut rem = factors;
                    for k in (j + 2)..n {
                        rem[k - 2] = rem[k];
                    }
                    canonicalize_and_add_term(builder, rem, n - 2, coeff);

                    // Part 2: -coeff * c^\dagger c
                    factors[j].1 = true;
                    factors[j + 1].1 = false;
                    coeff = -coeff;
                    swapped = true;
                }
            }
        }
        if !swapped {
            break;
        }
        i += 1;
    }

    // Check for duplicate creation or annihilation at the same site (e.g. c_s^\dagger c_s^\dagger = 0)
    for j in 0..(n.saturating_sub(1)) {
        if factors[j].0 == factors[j + 1].0 && factors[j].1 == factors[j + 1].1 {
            return;
        }
    }

    builder.add_sorted_term_factors(&factors[..n], coeff);
}

/// A direct Finite-State Machine / Auto-MPO builder for fermionic and electronic Hamiltonians.
pub struct MpoBuilder {
    pub l: usize,
    structural_transitions: Vec<std::collections::BTreeMap<(Channel, Channel), Op2>>,
    coupling_transitions: Vec<std::collections::BTreeMap<(Channel, Channel), Op2>>,
    active_channels: Vec<std::collections::BTreeSet<Channel>>,
}

impl MpoBuilder {
    pub fn new(l: usize) -> Self {
        let mut active = vec![std::collections::BTreeSet::new(); l + 1];
        if l > 0 {
            active[0].insert(Channel::Start);
            for b in 1..l {
                active[b].insert(Channel::Start);
                active[b].insert(Channel::Finish);
            }
            active[l].insert(Channel::Finish);
        }
        Self {
            l,
            structural_transitions: vec![std::collections::BTreeMap::new(); l],
            coupling_transitions: vec![std::collections::BTreeMap::new(); l],
            active_channels: active,
        }
    }

    fn mark_active(&mut self, ch: Channel, from_bond: usize, to_bond: usize) {
        for b in from_bond..=to_bond {
            self.active_channels[b].insert(ch);
        }
    }

    fn set_structural(&mut self, site: usize, from: Channel, to: Channel, op: Op2) {
        self.structural_transitions[site].insert((from, to), op);
    }

    fn add_coupling(&mut self, site: usize, from: Channel, to: Channel, op: Op2, coeff: f64) {
        let entry = self.coupling_transitions[site]
            .entry((from, to))
            .or_insert([[0.0; 2]; 2]);
        for s in 0..2 {
            for sp in 0..2 {
                entry[s][sp] += coeff * op[s][sp];
            }
        }
    }

    pub fn add_term_factors(&mut self, factors: &[(usize, bool)], coeff: f64) {
        assert!(factors.len() <= 4, "Up to 4 fermionic factors supported");
        let mut arr = [(0, false); 4];
        let len = factors.len();
        arr[..len].copy_from_slice(factors);
        canonicalize_and_add_term(self, arr, len, coeff);
    }

    pub fn add_sorted_term_factors(&mut self, factors: &[(usize, bool)], coeff: f64) {
        if coeff.abs() < 1e-15 {
            return;
        }
        if factors.is_empty() {
            if self.l > 0 {
                self.add_coupling(0, Channel::Start, Channel::Finish, I2, coeff);
            }
            return;
        }

        let mut sites: Vec<usize> = factors.iter().map(|&(s, _)| s).collect();
        sites.dedup();

        let k = sites.len();
        if k == 1 {
            let x1 = sites[0];
            let op1 = term_operator_at_site(factors, x1);
            self.add_coupling(x1, Channel::Start, Channel::Finish, op1, coeff);
        } else if k == 2 {
            let (x1, x2) = (sites[0], sites[1]);
            let op1 = term_operator_at_site(factors, x1);
            let op2 = term_operator_at_site(factors, x2);
            let count_x1 = factors.iter().filter(|&&(s, _)| s == x1).count();
            let c1 = if count_x1 == 1 {
                let (_, is_dagger) = factors.iter().find(|&&(s, _)| s == x1).unwrap();
                if *is_dagger {
                    Channel::LeftCd(x1)
                } else {
                    Channel::LeftCm(x1)
                }
            } else {
                Channel::LeftN(x1)
            };
            self.set_structural(x1, Channel::Start, c1, op1);
            self.mark_active(c1, x1 + 1, x2);
            self.add_coupling(x2, c1, Channel::Finish, op2, coeff);
        } else if k == 3 {
            let (x1, x2, x3) = (sites[0], sites[1], sites[2]);
            let op1 = term_operator_at_site(factors, x1);
            let op2 = term_operator_at_site(factors, x2);
            let op3 = term_operator_at_site(factors, x3);

            let count_x1 = factors.iter().filter(|&&(s, _)| s == x1).count();
            let count_x2 = factors.iter().filter(|&&(s, _)| s == x2).count();

            if count_x1 == 2 {
                let c1 = Channel::LeftN(x1);
                let (_, d3) = factors.iter().find(|&&(s, _)| s == x3).unwrap();
                let c2 = if *d3 {
                    Channel::RightCd(x3)
                } else {
                    Channel::RightCm(x3)
                };
                self.set_structural(x1, Channel::Start, c1, op1);
                self.mark_active(c1, x1 + 1, x2);
                self.add_coupling(x2, c1, c2, op2, coeff);
                self.mark_active(c2, x2 + 1, x3);
                self.set_structural(x3, c2, Channel::Finish, op3);
            } else if count_x2 == 2 {
                let (_, d1) = factors.iter().find(|&&(s, _)| s == x1).unwrap();
                let c1 = if *d1 {
                    Channel::LeftCd(x1)
                } else {
                    Channel::LeftCm(x1)
                };
                let (_, d3) = factors.iter().find(|&&(s, _)| s == x3).unwrap();
                let c2 = if *d3 {
                    Channel::RightCd(x3)
                } else {
                    Channel::RightCm(x3)
                };
                self.set_structural(x1, Channel::Start, c1, op1);
                self.mark_active(c1, x1 + 1, x2);
                self.add_coupling(x2, c1, c2, op2, coeff);
                self.mark_active(c2, x2 + 1, x3);
                self.set_structural(x3, c2, Channel::Finish, op3);
            } else {
                let (_, d1) = factors.iter().find(|&&(s, _)| s == x1).unwrap();
                let (_, d2) = factors.iter().find(|&&(s, _)| s == x2).unwrap();
                let c1 = if *d1 {
                    Channel::LeftCd(x1)
                } else {
                    Channel::LeftCm(x1)
                };
                let c2 = match (*d1, *d2) {
                    (true, true) => Channel::LeftCdCd(x1, x2),
                    (false, false) => Channel::LeftCmCm(x1, x2),
                    (true, false) => Channel::LeftPair(x1, x2),
                    (false, true) => Channel::LeftPairRev(x1, x2),
                };
                self.set_structural(x1, Channel::Start, c1, op1);
                self.mark_active(c1, x1 + 1, x2);
                self.set_structural(x2, c1, c2, op2);
                self.mark_active(c2, x2 + 1, x3);
                self.add_coupling(x3, c2, Channel::Finish, op3, coeff);
            }
        } else if k == 4 {
            let (x1, x2, x3, x4) = (sites[0], sites[1], sites[2], sites[3]);
            let op1 = term_operator_at_site(factors, x1);
            let op2 = term_operator_at_site(factors, x2);
            let op3 = term_operator_at_site(factors, x3);
            let op4 = term_operator_at_site(factors, x4);

            let (_, d1) = factors.iter().find(|&&(s, _)| s == x1).unwrap();
            let (_, d2) = factors.iter().find(|&&(s, _)| s == x2).unwrap();
            let (_, d4) = factors.iter().find(|&&(s, _)| s == x4).unwrap();

            let c1 = if *d1 {
                Channel::LeftCd(x1)
            } else {
                Channel::LeftCm(x1)
            };
            let c2 = match (*d1, *d2) {
                (true, true) => Channel::LeftCdCd(x1, x2),
                (false, false) => Channel::LeftCmCm(x1, x2),
                (true, false) => Channel::LeftPair(x1, x2),
                (false, true) => Channel::LeftPairRev(x1, x2),
            };
            let c3 = if *d4 {
                Channel::RightCd(x4)
            } else {
                Channel::RightCm(x4)
            };

            self.set_structural(x1, Channel::Start, c1, op1);
            self.mark_active(c1, x1 + 1, x2);
            self.set_structural(x2, c1, c2, op2);
            self.mark_active(c2, x2 + 1, x3);
            self.add_coupling(x3, c2, c3, op3, coeff);
            self.mark_active(c3, x3 + 1, x4);
            self.set_structural(x4, c3, Channel::Finish, op4);
        }
    }

    pub fn build(self) -> Mpo {
        if self.l == 0 {
            return Mpo { sites: vec![] };
        }

        let mut bond_index_maps = Vec::with_capacity(self.l + 1);
        let mut bond_dims = Vec::with_capacity(self.l + 1);

        for b in 0..=self.l {
            let mut map = std::collections::BTreeMap::new();
            if b == 0 {
                map.insert(Channel::Start, 0);
            } else if b == self.l {
                map.insert(Channel::Finish, 0);
            } else {
                map.insert(Channel::Start, 0);
                map.insert(Channel::Finish, 1);
                for &ch in &self.active_channels[b] {
                    if ch != Channel::Start && ch != Channel::Finish {
                        let next_idx = map.len();
                        map.insert(ch, next_idx);
                    }
                }
            }
            bond_dims.push(map.len());
            bond_index_maps.push(map);
        }

        let mut mpo_sites = Vec::with_capacity(self.l);

        for m in 0..self.l {
            let d_l = bond_dims[m];
            let d_r = bond_dims[m + 1];
            let mut site_tensor = MpoSite::zeros(d_l, d_r);

            let map_l = &bond_index_maps[m];
            let map_r = &bond_index_maps[m + 1];

            // 1. Idle self-transitions
            for (&ch, &idx_l) in map_l {
                if let Some(&idx_r) = map_r.get(&ch) {
                    let op = ch.idle_op();
                    for s in 0..2 {
                        for sp in 0..2 {
                            let val = site_tensor.get(idx_l, idx_r, s, sp) + op[s][sp];
                            site_tensor.set(idx_l, idx_r, s, sp, val);
                        }
                    }
                }
            }

            // 2. Structural transitions at site m
            for (&(ch_from, ch_to), &op) in &self.structural_transitions[m] {
                if let (Some(&idx_l), Some(&idx_r)) = (map_l.get(&ch_from), map_r.get(&ch_to)) {
                    for s in 0..2 {
                        for sp in 0..2 {
                            let val = site_tensor.get(idx_l, idx_r, s, sp) + op[s][sp];
                            site_tensor.set(idx_l, idx_r, s, sp, val);
                        }
                    }
                }
            }

            // 3. Coupling transitions at site m
            for (&(ch_from, ch_to), &op_sum) in &self.coupling_transitions[m] {
                if let (Some(&idx_l), Some(&idx_r)) = (map_l.get(&ch_from), map_r.get(&ch_to)) {
                    for s in 0..2 {
                        for sp in 0..2 {
                            let val = site_tensor.get(idx_l, idx_r, s, sp) + op_sum[s][sp];
                            site_tensor.set(idx_l, idx_r, s, sp, val);
                        }
                    }
                }
            }

            mpo_sites.push(site_tensor);
        }

        Mpo { sites: mpo_sites }
    }
}

impl Mpo {
    /// Construct a 1D Matrix Product Operator from 1-electron and 2-electron molecular orbital integrals.
    /// `h` is `K x K` (1-electron integrals), `g` is `K^4` (chemist notation `(pq|rs)` indexed `[(p*K+q)*K^2 + r*K+s]`).
    /// The resulting MPO has `2*K` sites in interleaved Jordan–Wigner ordering: `2p` = `(p,up)`, `2p+1` = `(p,down)`.
    pub fn from_electronic_integrals(n_orb: usize, h: &[f64], g: &[f64]) -> Self {
        let l = 2 * n_orb;
        let mut builder = MpoBuilder::new(l);

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
                    builder.add_term_factors(&factors, hpq);
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
                                builder.add_term_factors(&factors, coeff);
                            }
                        }
                    }
                }
            }
        }

        builder.build()
    }

    /// Construct an MPO for the 1D Hubbard model on `sites` chain sites (2*sites spin-orbitals).
    pub fn from_hubbard(sites: usize, t: f64, u: f64, mu: f64) -> Self {
        let l = 2 * sites;
        let mut builder = MpoBuilder::new(l);

        // On-site chemical potential: -mu * sum_i n_i
        if mu.abs() > 1e-15 {
            for i in 0..l {
                let factors = [(i, true), (i, false)];
                builder.add_term_factors(&factors, -mu);
            }
        }

        // Hopping: -t sum_{<cs, cs'>, sigma} (c_{cs, sigma}^\dagger c_{cs', sigma} + h.c.)
        if t.abs() > 1e-15 {
            for cs in 0..(sites - 1) {
                for sigma in 0..2 {
                    let i = 2 * cs + sigma;
                    let j = 2 * (cs + 1) + sigma;
                    // Forward hop
                    let fwd = [(i, true), (j, false)];
                    builder.add_term_factors(&fwd, -t);
                    // Backward hop
                    let bwd = [(j, true), (i, false)];
                    builder.add_term_factors(&bwd, -t);
                }
            }
        }

        // On-site interaction: U sum_{cs} n_{cs, up} n_{cs, down}
        if u.abs() > 1e-15 {
            for cs in 0..sites {
                let i = 2 * cs;
                let j = 2 * cs + 1;
                let factors = [(i, true), (i, false), (j, true), (j, false)];
                builder.add_term_factors(&factors, u);
            }
        }

        builder.build()
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

