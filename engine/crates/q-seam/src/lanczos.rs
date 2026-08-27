//! Ground state by Lanczos with full reorthogonalization.
//!
//! Every knob here is pinned by amendment A1/P1 of `Q_SEAM_PREREG.md`, because replay discipline
//! applies to instruments and a random start makes the residual ladder non-replayable:
//!
//! * start vector from **SplitMix64**, seed [`START_SEED`], entries uniform on `[-1, 1]`,
//!   normalized, over the basis in canonical ascending order;
//! * a fixed all-ones start is **rejected** — it is a symmetry eigenvector and can be exactly
//!   orthogonal to a ground state of the opposite parity under global spin flip;
//! * full reorthogonalization against every stored Krylov vector;
//! * the **orthogonality guard** `|<v0|psi0>| >= 1e-8` is reported and enforced;
//! * on a failed residual gate or a failed guard: **one** deterministic restart at `seed + 1`,
//!   reported; a second failure VOIDs the configuration. No adaptive retries.

use crate::dense::jacobi;
use crate::hubbard::Hubbard;

/// A1/P1's pinned start seed. (Corrected from the prereg's first printing, which used a
/// non-hexadecimal digit — see `Q_SEAM_PREREG.md` A1/T.)
pub const START_SEED: u64 = 0x515F_5EA0_0000_0001;

/// Iteration cap. A1/P1 pins 500; the ceiling that actually binds is memory, since full
/// reorthogonalization stores every Krylov vector (400 x 63 504 x 8 B ~ 203 MB at N = 10).
pub const MAX_ITERS: usize = 400;

/// The G-E4a residual gate, relative to `max(1, |E|)`.
pub const RESIDUAL_GATE: f64 = 1e-12;

/// The A1/P1 orthogonality guard on the start vector's overlap with the ground state.
pub const OVERLAP_GUARD: f64 = 1e-8;

/// **Convergence-check cadence. STAKED at 10.**
///
/// The first printing of this module rebuilt and re-diagonalized the full `m x m` tridiagonal on
/// EVERY iteration, so the cumulative cost grew like `iters^4` — invisible while `N <= 10` kept the
/// absolute cost small through Q5, Q7 and Q7b, and dominant once Q8's `N = 12` (dim 853 776) made
/// it bind. Found by the q8-mps campaign's `probe_n12.rs`.
///
/// The cadence is a **pure cost change**: at a checkpoint the loop replays the convergence test at
/// every size not yet tested and stops at the EARLIEST size that would have terminated the original
/// per-iteration loop, so it exits at the same size with the same Ritz pair. Nothing about the
/// mathematics moves — same recurrence, same double reorthogonalization, same tolerances, same
/// `MAX_ITERS`. Setting this to 1 recovers the original cost shape exactly, which is the
/// mutation check.
pub const CHECK_EVERY: usize = 10;

#[derive(Clone, Debug)]
pub struct GroundState {
    pub energy: f64,
    pub vector: Vec<f64>,
    /// `||H v - E v|| / (||v|| max(1, |E|))` — the gate quantity, measured not assumed.
    pub residual: f64,
    /// Second Ritz value: the in-sector gap for G-E6. An estimate, and labelled as one.
    pub first_excited: f64,
    pub overlap_start: f64,
    pub iterations: usize,
    pub restarts: usize,
    pub seed: u64,
}

struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform on `[-1, 1]`.
    fn next_signed(&mut self) -> f64 {
        let bits = self.next_u64() >> 11; // 53 significant bits
        (bits as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    }
}

fn seeded_start(dim: usize, seed: u64) -> Vec<f64> {
    let mut rng = SplitMix64(seed);
    let mut v: Vec<f64> = (0..dim).map(|_| rng.next_signed()).collect();
    let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    for x in v.iter_mut() {
        *x /= norm;
    }
    v
}

/// One Lanczos run at a given seed. `None` means the residual gate or the guard failed.
fn run_once(h: &Hubbard, seed: u64) -> Option<GroundState> {
    let dim = h.dim();
    let v0 = seeded_start(dim, seed);

    let mut basis: Vec<Vec<f64>> = Vec::with_capacity(MAX_ITERS);
    let mut alpha: Vec<f64> = Vec::new();
    // Every beta ever produced, so the convergence test can be REPLAYED at any earlier size.
    // That replay is what makes the cadence a pure cost change (see CHECK_EVERY).
    let mut bs: Vec<f64> = Vec::new();

    basis.push(v0.clone());
    let mut w = vec![0.0; dim];
    let mut iterations = 0;
    let mut scanned = 0usize; // sizes whose convergence test has already been evaluated

    let mut best: Option<(f64, f64, Vec<f64>)> = None; // (E0, E1, ritz vector)

    for j in 0..MAX_ITERS {
        iterations = j + 1;
        h.apply(&basis[j], &mut w);

        let a = dot(&w, &basis[j]);
        alpha.push(a);
        for k in 0..dim {
            w[k] -= a * basis[j][k];
        }
        if j > 0 {
            let b = bs[j - 1];
            for k in 0..dim {
                w[k] -= b * basis[j - 1][k];
            }
        }
        // Full reorthogonalization, twice — one pass is not enough once the basis is large.
        for _ in 0..2 {
            for u in basis.iter() {
                let c = dot(&w, u);
                for k in 0..dim {
                    w[k] -= c * u[k];
                }
            }
        }

        let b = dot(&w, &w).sqrt();
        bs.push(b);
        let size = alpha.len();

        // Only ever build and diagonalize the tridiagonal at a CHECKPOINT. Breakdown and the
        // final iteration force one, so no termination condition can be missed.
        let due = size % CHECK_EVERY == 0 || b <= 1e-13 || j == MAX_ITERS - 1;
        if due {
            // ONE solve at the checkpoint. Only if it passes do we backtrack through the block to
            // find the EARLIEST size that would have terminated the original per-iteration loop,
            // so the exit size and Ritz pair are the ones the original produced.
            let mut stop_at: Option<usize> = None;
            if let Some(t) = test_size(&alpha, &bs, size) {
                let _ = t;
                let lo = scanned + 1;
                let mut earliest = size;
                for m in lo..size {
                    if test_size(&alpha, &bs, m).is_some() {
                        earliest = m;
                        break;
                    }
                }
                stop_at = Some(earliest);
            }
            scanned = size;

            if let Some(m) = stop_at {
                let (e0, e1, vec0) = test_size(&alpha, &bs, m)
                    .expect("the size that just passed must still pass");
                let mut ritz = vec![0.0; dim];
                for (k, uk) in basis.iter().take(m).enumerate() {
                    let c = vec0[k];
                    for i in 0..dim {
                        ritz[i] += c * uk[i];
                    }
                }
                let norm = dot(&ritz, &ritz).sqrt();
                for x in ritz.iter_mut() {
                    *x /= norm;
                }
                // Report the size the loop actually stopped at, not the checkpoint that
                // triggered the scan, so `iterations` means the same thing it always did.
                iterations = m;
                best = Some((e0, e1, ritz));
                break;
            }
        }

        for k in 0..dim {
            w[k] /= b;
        }
        basis.push(w.clone());
    }

    let (energy, first_excited, vector) = best?;

    let mut hv = vec![0.0; dim];
    h.apply(&vector, &mut hv);
    let resid = hv
        .iter()
        .zip(vector.iter())
        .map(|(a, b)| (a - energy * b) * (a - energy * b))
        .sum::<f64>()
        .sqrt()
        / energy.abs().max(1.0);

    let overlap_start = dot(&v0, &vector).abs();

    if resid > RESIDUAL_GATE || overlap_start < OVERLAP_GUARD {
        return None;
    }

    Some(GroundState {
        energy,
        vector,
        residual: resid,
        first_excited,
        overlap_start,
        iterations,
        restarts: 0,
        seed,
    })
}

/// The pinned policy: one run, then exactly one deterministic restart at `seed + 1`.
pub fn ground_state(h: &Hubbard) -> Option<GroundState> {
    if let Some(g) = run_once(h, START_SEED) {
        return Some(g);
    }
    let mut g = run_once(h, START_SEED + 1)?;
    g.restarts = 1;
    Some(g)
}

/// The original per-iteration convergence test, factored so it can be evaluated at ANY size.
/// Returns `(E0, E1, lowest Ritz vector)` when the loop would have terminated at this size.
fn test_size(alpha: &[f64], bs: &[f64], m: usize) -> Option<(f64, f64, Vec<f64>)> {
    let mut tri = vec![0.0; m * m];
    for i in 0..m {
        tri[i * m + i] = alpha[i];
        if i + 1 < m {
            tri[i * m + i + 1] = bs[i];
            tri[(i + 1) * m + i] = bs[i];
        }
    }
    let eig = jacobi(tri, m);
    let e0 = eig.values[0];
    let e1 = if m > 1 { eig.values[1] } else { f64::INFINITY };
    let bm = bs[m - 1];
    let est = bm * eig.vectors[0][m - 1].abs();
    let breakdown = bm <= 1e-13;
    if est <= 0.1 * RESIDUAL_GATE * e0.abs().max(1.0) || breakdown || m == MAX_ITERS {
        Some((e0, e1, eig.vectors[0].clone()))
    } else {
        None
    }
}

#[inline]
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}
