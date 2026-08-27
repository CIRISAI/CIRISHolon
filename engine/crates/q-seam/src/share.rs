//! Q6's instrument: the beyond-pair information of the exact state's Fock-basis distribution.
//!
//! Named honestly once, and never described otherwise: this is a **classical statistic of a
//! quantum state** — the measurement distribution in the occupation basis. It is not a
//! quantum-sector reading.
//!
//! Two quantities, both computed on EXACT marginals (no sampling, so no finite-N floor, no
//! permutation floor for the estimator, no tie fraction, and no `b ≥ 3` binning anywhere — the
//! slots are natively binary):
//!
//! * `B4` — beyond-pair information of a 4-slot marginal: `D(P ‖ Q)` where `Q` is the maximum
//!   entropy distribution matching all 1- and 2-marginals. This is the primary statistic.
//! * `I_C^(3)` — the same object on three slots, which `Q_SEAM_PREREG.md` §6.2 **derives** to be
//!   exactly zero on this family: complement symmetry forces the three fields to vanish, leaving
//!   3 couplings to match 3 pair marginals, so the parameter count equals the constraint count
//!   and `Q = P`. It is the plumb line, and a nonzero reading VOIDs Q6 rather than falsifying it.
//!
//! Solver policy is amendment A1/P2: Newton first, IPF on failure, a quadruple counts as
//! converged iff either meets the `1e-13` marginal-residual gate, and where **both** converge
//! they must agree — the programme has caught IPF one-sidedly overstating the share on
//! near-deterministic states, so that agreement is checked, not assumed.

use crate::hubbard::Hubbard;

/// The marginal-residual gate of A1/P2.
pub const MARGINAL_GATE: f64 = 1e-13;
/// Newton and IPF must agree this closely where both converge.
pub const CROSSCHECK_GATE: f64 = 1e-10;

const NEWTON_ITERS: usize = 200;
const IPF_ITERS: usize = 200_000;

#[derive(Clone, Debug, Default)]
pub struct ShareReport {
    /// Mean `B4` over every quadruple of slots — the primary aggregate.
    pub b4_mean: f64,
    pub b4_max: f64,
    /// Largest `|I_C^(3)|` over every triple — the plumb line, derived to be exactly zero.
    pub ic3_max: f64,
    pub quadruples: usize,
    pub triples: usize,
    /// Quadruples reached only by IPF: the boundary-of-the-exponential-family diagnostic.
    pub ipf_only: usize,
    /// Quadruples where neither solver met the gate. Any of these VOIDs the configuration.
    pub failed: usize,
    /// Worst Newton-vs-IPF disagreement where both converged.
    pub worst_crosscheck: f64,
    /// Smallest cell probability seen, reported as a diagnostic and never used as a filter.
    pub min_cell: f64,
}

impl ShareReport {
    pub fn is_void(&self) -> bool {
        self.failed > 0 || self.worst_crosscheck > CROSSCHECK_GATE
    }
}

/// Accumulate every 3- and 4-slot marginal of `|psi|^2` over the `2N` occupancy slots.
///
/// One pass over the basis, incrementing every tuple's histogram — the FULL grid, because the
/// equally-spaced diagonal is provably blind and exact marginals cost nothing.
pub fn measure(h: &Hubbard, psi: &[f64]) -> ShareReport {
    let slots = 2 * h.sites;
    let n = h.n_conf();

    let quads: Vec<[usize; 4]> = combinations4(slots);
    let tris: Vec<[usize; 3]> = combinations3(slots);
    let mut quad_hist = vec![0.0f64; quads.len() * 16];
    let mut tri_hist = vec![0.0f64; tris.len() * 8];

    for (iu, &mu) in h.basis.masks.iter().enumerate() {
        for (id, &md) in h.basis.masks.iter().enumerate() {
            let amp = psi[iu * n + id];
            let p = amp * amp;
            if p == 0.0 {
                continue;
            }
            let bits = (mu as u32) | ((md as u32) << h.sites);
            for (q, quad) in quads.iter().enumerate() {
                let cell = ((bits >> quad[0]) & 1)
                    | (((bits >> quad[1]) & 1) << 1)
                    | (((bits >> quad[2]) & 1) << 2)
                    | (((bits >> quad[3]) & 1) << 3);
                quad_hist[q * 16 + cell as usize] += p;
            }
            for (t, tri) in tris.iter().enumerate() {
                let cell = ((bits >> tri[0]) & 1)
                    | (((bits >> tri[1]) & 1) << 1)
                    | (((bits >> tri[2]) & 1) << 2);
                tri_hist[t * 8 + cell as usize] += p;
            }
        }
    }

    let mut rep = ShareReport {
        quadruples: quads.len(),
        triples: tris.len(),
        min_cell: 1.0,
        ..Default::default()
    };

    let mut sum_b4 = 0.0;
    for q in 0..quads.len() {
        let p = &quad_hist[q * 16..q * 16 + 16];
        rep.min_cell = rep.min_cell.min(p.iter().copied().fold(1.0, f64::min));
        match beyond_pair::<4, 16>(p) {
            Solved::Both { value, disagreement } => {
                sum_b4 += value;
                rep.b4_max = rep.b4_max.max(value);
                rep.worst_crosscheck = rep.worst_crosscheck.max(disagreement);
            }
            Solved::IpfOnly(value) => {
                sum_b4 += value;
                rep.b4_max = rep.b4_max.max(value);
                rep.ipf_only += 1;
            }
            Solved::Failed => rep.failed += 1,
        }
    }
    rep.b4_mean = sum_b4 / quads.len() as f64;

    for t in 0..tris.len() {
        let p = &tri_hist[t * 8..t * 8 + 8];
        if let Solved::Both { value, .. } | Solved::IpfOnly(value) = beyond_pair::<3, 8>(p) {
            rep.ic3_max = rep.ic3_max.max(value.abs());
        }
    }

    rep
}

enum Solved {
    Both { value: f64, disagreement: f64 },
    IpfOnly(f64),
    Failed,
}

/// `D(P ‖ Q)` where `Q` is the pairwise maximum-entropy distribution matching `P`'s 1- and
/// 2-marginals, on `K` binary slots (`CELLS = 2^K`).
fn beyond_pair<const K: usize, const CELLS: usize>(p: &[f64]) -> Solved {
    let newton = maxent_newton::<K, CELLS>(p);
    let ipf = maxent_ipf::<K, CELLS>(p);
    match (newton, ipf) {
        (Some(qn), Some(qi)) => {
            let vn = divergence(p, &qn);
            let vi = divergence(p, &qi);
            Solved::Both { value: vn, disagreement: (vn - vi).abs() }
        }
        (Some(qn), None) => Solved::Both { value: divergence(p, &qn), disagreement: 0.0 },
        (None, Some(qi)) => Solved::IpfOnly(divergence(p, &qi)),
        (None, None) => Solved::Failed,
    }
}

fn divergence(p: &[f64], q: &[f64]) -> f64 {
    // H(Q) - H(P), accumulated with Kahan summation (near-ceiling numerics lesson).
    let mut sum = 0.0;
    let mut comp = 0.0;
    for (&pi, &qi) in p.iter().zip(q.iter()) {
        if pi <= 0.0 {
            continue;
        }
        let term = pi * ((pi / qi).ln());
        let y = term - comp;
        let t = sum + y;
        comp = (t - sum) - y;
        sum = t;
    }
    sum.max(0.0)
}

/// The 1- and 2-marginal features of a cell: `K` singles then `K(K-1)/2` pairs.
fn features<const K: usize>(cell: usize, out: &mut [f64]) {
    let mut k = 0;
    for a in 0..K {
        out[k] = ((cell >> a) & 1) as f64;
        k += 1;
    }
    for a in 0..K {
        for b in (a + 1)..K {
            out[k] = (((cell >> a) & 1) * ((cell >> b) & 1)) as f64;
            k += 1;
        }
    }
}

fn n_features<const K: usize>() -> usize {
    K + K * (K - 1) / 2
}

fn maxent_newton<const K: usize, const CELLS: usize>(p: &[f64]) -> Option<Vec<f64>> {
    let nf = n_features::<K>();
    let mut feat = vec![0.0; CELLS * nf];
    for cell in 0..CELLS {
        features::<K>(cell, &mut feat[cell * nf..(cell + 1) * nf]);
    }
    let target: Vec<f64> = (0..nf)
        .map(|k| (0..CELLS).map(|c| p[c] * feat[c * nf + k]).sum())
        .collect();

    let mut theta = vec![0.0; nf];
    for _ in 0..NEWTON_ITERS {
        let mut q = vec![0.0; CELLS];
        let mut z = 0.0;
        for cell in 0..CELLS {
            let e: f64 = (0..nf).map(|k| theta[k] * feat[cell * nf + k]).sum();
            q[cell] = e.exp();
            z += q[cell];
        }
        for x in q.iter_mut() {
            *x /= z;
        }

        let mean: Vec<f64> = (0..nf)
            .map(|k| (0..CELLS).map(|c| q[c] * feat[c * nf + k]).sum())
            .collect();
        let grad: Vec<f64> = (0..nf).map(|k| mean[k] - target[k]).collect();
        let resid = grad.iter().fold(0.0f64, |a, b| a.max(b.abs()));
        if resid <= MARGINAL_GATE {
            return Some(q);
        }

        let mut cov = vec![0.0; nf * nf];
        for k in 0..nf {
            for l in 0..nf {
                let mut s = 0.0;
                for c in 0..CELLS {
                    s += q[c] * feat[c * nf + k] * feat[c * nf + l];
                }
                cov[k * nf + l] = s - mean[k] * mean[l];
            }
        }
        let step = solve_spd(&cov, &grad, nf)?;
        // Damped Newton: the exponential family's boundary is where this needs the damping.
        let scale = {
            let m = step.iter().fold(0.0f64, |a, b| a.max(b.abs()));
            if m > 4.0 {
                4.0 / m
            } else {
                1.0
            }
        };
        for k in 0..nf {
            theta[k] -= scale * step[k];
            if !theta[k].is_finite() {
                return None;
            }
        }
    }
    None
}

/// Iterative proportional fitting over the pair marginals. Converges to the CLOSURE of the
/// exponential family, which is exactly where Newton cannot go — the right tool at the boundary.
fn maxent_ipf<const K: usize, const CELLS: usize>(p: &[f64]) -> Option<Vec<f64>> {
    let mut q = vec![1.0 / CELLS as f64; CELLS];
    for it in 0..IPF_ITERS {
        for a in 0..K {
            for b in (a + 1)..K {
                let mut want = [0.0f64; 4];
                let mut have = [0.0f64; 4];
                for cell in 0..CELLS {
                    let s = ((cell >> a) & 1) | (((cell >> b) & 1) << 1);
                    want[s] += p[cell];
                    have[s] += q[cell];
                }
                for cell in 0..CELLS {
                    let s = ((cell >> a) & 1) | (((cell >> b) & 1) << 1);
                    q[cell] = if have[s] > 0.0 { q[cell] * want[s] / have[s] } else { 0.0 };
                }
            }
        }
        if it % 16 == 0 {
            let mut worst = 0.0f64;
            for a in 0..K {
                for b in (a + 1)..K {
                    let mut want = [0.0f64; 4];
                    let mut have = [0.0f64; 4];
                    for cell in 0..CELLS {
                        let s = ((cell >> a) & 1) | (((cell >> b) & 1) << 1);
                        want[s] += p[cell];
                        have[s] += q[cell];
                    }
                    for s in 0..4 {
                        worst = worst.max((want[s] - have[s]).abs());
                    }
                }
            }
            if worst <= MARGINAL_GATE {
                return Some(q);
            }
        }
    }
    None
}

/// Cholesky solve for a symmetric positive-definite system. `None` when the matrix is singular,
/// which is the boundary case IPF is there to cover.
fn solve_spd(a: &[f64], b: &[f64], n: usize) -> Option<Vec<f64>> {
    let mut l = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..=i {
            let mut s = a[i * n + j];
            for k in 0..j {
                s -= l[i * n + k] * l[j * n + k];
            }
            if i == j {
                if s <= 1e-14 {
                    return None;
                }
                l[i * n + j] = s.sqrt();
            } else {
                l[i * n + j] = s / l[j * n + j];
            }
        }
    }
    let mut y = vec![0.0; n];
    for i in 0..n {
        let mut s = b[i];
        for k in 0..i {
            s -= l[i * n + k] * y[k];
        }
        y[i] = s / l[i * n + i];
    }
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut s = y[i];
        for k in (i + 1)..n {
            s -= l[k * n + i] * x[k];
        }
        x[i] = s / l[i * n + i];
    }
    Some(x)
}

fn combinations4(n: usize) -> Vec<[usize; 4]> {
    let mut out = Vec::new();
    for a in 0..n {
        for b in (a + 1)..n {
            for c in (b + 1)..n {
                for d in (c + 1)..n {
                    out.push([a, b, c, d]);
                }
            }
        }
    }
    out
}

fn combinations3(n: usize) -> Vec<[usize; 3]> {
    let mut out = Vec::new();
    for a in 0..n {
        for b in (a + 1)..n {
            for c in (b + 1)..n {
                out.push([a, b, c]);
            }
        }
    }
    out
}
