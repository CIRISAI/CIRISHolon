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
