//! The placebo: a coupling matrix with the same weights wired to different pairs.
//!
//! The prereg fixes the construction: permute the OFF-DIAGONAL entries of `COUPLING`,
//! preserving symmetry. That holds the weight multiset exactly — same values, same total
//! weight, same scale — and destroys only which pair carries which weight.
//!
//! A node relabelling would NOT do. Relabelling is a graph isomorphism, so the physics is
//! literally unchanged under it; arm B would tie with arm C by construction and the null
//! would look clean while testing nothing. `tests::scramble_is_not_a_relabelling` holds
//! that line.

use ciris_sim_core::data::{COUPLING, N};

/// Deterministic, dependency-free PRNG (SplitMix64). Seeded so every draw is reproducible
/// and can be quoted in the record.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self { Rng(seed.wrapping_add(0x9E3779B97F4A7C15)) }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    /// Uniform in `0..n`, rejection-sampled so the distribution is exact.
    fn below(&mut self, n: u64) -> u64 {
        let zone = u64::MAX - (u64::MAX % n);
        loop { let r = self.next_u64(); if r < zone { return r % n; } }
    }
}

/// Permute the off-diagonal weights, keeping the matrix symmetric and the diagonal zero.
pub fn scramble(seed: u64) -> [[f64; N]; N] {
    // The 55 upper-triangular entries ARE the free parameters; the lower triangle is
    // their mirror. Permuting the pairs is what preserves symmetry exactly.
    let mut vals: Vec<f64> = Vec::with_capacity(N * (N - 1) / 2);
    for i in 0..N { for j in (i + 1)..N { vals.push(COUPLING[i][j]); } }

    let mut rng = Rng::new(seed);
    for k in (1..vals.len()).rev() {          // Fisher-Yates
        let s = rng.below((k + 1) as u64) as usize;
        vals.swap(k, s);
    }

    let mut out = [[0.0f64; N]; N];
    let mut it = vals.into_iter();
    for i in 0..N {
        for j in (i + 1)..N {
            let v = it.next().unwrap();
            out[i][j] = v;
            out[j][i] = v;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upper(m: &[[f64; N]; N]) -> Vec<f64> {
        let mut v = Vec::new();
        for i in 0..N { for j in (i + 1)..N { v.push(m[i][j]); } }
        v
    }
    fn sorted(mut v: Vec<f64>) -> Vec<f64> {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap()); v
    }

    #[test]
    fn preserves_the_weight_multiset_exactly() {
        for seed in 0..10u64 {
            let s = scramble(seed);
            assert_eq!(sorted(upper(&s)), sorted(upper(&COUPLING)),
                       "seed {seed} changed the multiset");
        }
    }

    #[test]
    fn stays_symmetric_with_zero_diagonal() {
        for seed in 0..10u64 {
            let s = scramble(seed);
            for i in 0..N {
                assert_eq!(s[i][i], 0.0);
                for j in 0..N { assert_eq!(s[i][j], s[j][i], "asymmetry at {i},{j}"); }
            }
        }
    }

    /// The scramble must actually move weight between PAIRS, not merely rename nodes.
    /// A relabelling preserves the sorted degree sequence; a genuine rewiring generally
    /// does not. If this ever passes trivially the placebo has stopped being a placebo.
    #[test]
    fn scramble_is_not_a_relabelling() {
        let deg = |m: &[[f64; N]; N]| {
            sorted((0..N).map(|i| (0..N).map(|j| m[i][j]).sum::<f64>()).collect())
        };
        let real = deg(&COUPLING);
        let moved = (0..10u64).filter(|&s| {
            deg(&scramble(s)).iter().zip(&real).any(|(a, b)| (a - b).abs() > 1e-9)
        }).count();
        assert!(moved >= 9, "only {moved}/10 draws changed the degree sequence; \
                             a permutation that preserves it may be a relabelling");
    }

    #[test]
    fn seeds_are_reproducible_and_distinct() {
        assert_eq!(upper(&scramble(7)), upper(&scramble(7)));
        assert_ne!(upper(&scramble(1)), upper(&scramble(2)));
    }
}
