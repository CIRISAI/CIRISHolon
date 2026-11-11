//! The match, as exact arithmetic in one dimension.
//!
//! [`w1`] is the Wasserstein-1 distance between two samples of integers, returned as an exact
//! rational `(numerator, denominator)` with denominator `n * m`. It is the cost of the sorted
//! coupling, which in one dimension is the least cost of any coupling (`SceneCert.cost`).
//! [`quantile`] is the lower empirical quantile. [`resolved`] is the resolution floor counted
//! in BLOCKS: a level is judged only when enough independent blocks each hold enough samples
//! beyond it, so repeating a sample cannot deepen what the reference is allowed to say.
//!
//! Values are integers. A consumer with real-valued readings fixes its own unit (millimetres,
//! parts per million) before calling.

/// Exact W1 between two samples, as `(numerator, n * m)`. Both must be non-empty.
pub fn w1(a: &[i64], b: &[i64]) -> (u128, u128) {
    assert!(!a.is_empty() && !b.is_empty(), "w1 of an empty sample");
    let (mut x, mut y) = (a.to_vec(), b.to_vec());
    x.sort_unstable();
    y.sort_unstable();
    let (n, m) = (x.len() as u128, y.len() as u128);
    // integral of |F_a - F_b|, with both CDFs scaled by n*m so every term is an integer
    let (mut i, mut j) = (0usize, 0usize);
    let mut num: u128 = 0;
    let mut prev = x[0].min(y[0]);
    while i < x.len() || j < y.len() {
        let next = match (x.get(i), y.get(j)) {
            (Some(&p), Some(&q)) => p.min(q),
            (Some(&p), None) => p,
            (None, Some(&q)) => q,
            (None, None) => unreachable!(),
        };
        let fa = i as u128 * m;
        let fb = j as u128 * n;
        num += fa.abs_diff(fb) * (next as i128 - prev as i128) as u128;
        prev = next;
        while i < x.len() && x[i] == next { i += 1; }
        while j < y.len() && y[j] == next { j += 1; }
    }
    (num, n * m)
}

/// The lower empirical quantile at `level_ppm` parts per million: the `ceil(p * n)`-th
/// smallest value, and the smallest for `p = 0`.
pub fn quantile(sample: &[i64], level_ppm: u32) -> i64 {
    assert!(!sample.is_empty() && level_ppm <= 1_000_000);
    let mut s = sample.to_vec();
    s.sort_unstable();
    let n = s.len() as u128;
    let k = (level_ppm as u128 * n).div_ceil(1_000_000);
    s[(k.max(1) - 1) as usize]
}

/// The resolution floor, counted in blocks. `blocks[b]` is the number of samples block `b`
/// holds beyond the level. The level is resolved when at least `min_blocks` blocks each hold
/// at least `min_per_block`.
pub fn resolved(beyond_per_block: &[u64], min_blocks: usize, min_per_block: u64) -> bool {
    beyond_per_block.iter().filter(|&&k| k >= min_per_block).count() >= min_blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::Stream;

    fn sample(seed: u64, n: usize) -> Vec<i64> {
        let mut s = Stream::new(seed);
        (0..n).map(|_| s.below(1_000) as i64 - 500).collect()
    }

    #[test]
    fn zero_on_itself_and_on_a_reordering() {
        let a = sample(1, 300);
        let mut b = a.clone();
        b.reverse();
        assert_eq!(w1(&a, &b).0, 0);
    }

    #[test]
    fn a_shift_costs_the_shift() {
        let a = sample(2, 257);
        let b: Vec<i64> = a.iter().map(|x| x + 37).collect();
        let (num, den) = w1(&a, &b);
        assert_eq!(num, 37 * den);
    }

    #[test]
    fn symmetric_and_obeys_the_triangle_with_unequal_sizes() {
        for seed in 0..40 {
            let (a, b, c) = (sample(seed, 40), sample(seed + 100, 55), sample(seed + 200, 70));
            assert_eq!(w1(&a, &b), w1(&b, &a));
            let (ab, ac, cb) = (w1(&a, &b), w1(&a, &c), w1(&c, &b));
            // ab <= ac + cb, cross-multiplied to a common denominator
            let d = ab.1 * ac.1 * cb.1;
            assert!(ab.0 * (d / ab.1) <= ac.0 * (d / ac.1) + cb.0 * (d / cb.1));
        }
    }

    #[test]
    fn equal_sizes_agree_with_the_sorted_pairing() {
        let (mut a, mut b) = (sample(5, 200), sample(6, 200));
        let (num, den) = w1(&a, &b);
        a.sort_unstable();
        b.sort_unstable();
        let paired: u128 = a.iter().zip(&b).map(|(x, y)| x.abs_diff(*y) as u128).sum();
        assert_eq!(num, paired * 200);
        assert_eq!(den, 200 * 200);
    }

    #[test]
    fn the_mean_can_pass_while_the_distribution_refuses() {
        // same mean, different spread: the case a mean test cannot see
        let a = vec![-1i64, 1, -1, 1];
        let b = vec![-10i64, 10, -10, 10];
        assert_eq!(a.iter().sum::<i64>(), b.iter().sum::<i64>());
        let (num, den) = w1(&a, &b);
        assert_eq!(num, 9 * den);
    }

    #[test]
    fn quantiles() {
        let s: Vec<i64> = (1..=100).collect();
        assert_eq!(quantile(&s, 500_000), 50);
        assert_eq!(quantile(&s, 990_000), 99);
        assert_eq!(quantile(&s, 1_000_000), 100);
        assert_eq!(quantile(&s, 0), 1);
    }

    #[test]
    fn repeating_a_sample_does_not_deepen_the_floor() {
        // 20 blocks, 40 samples beyond the level in each: not resolved at 30 blocks
        let once = vec![40u64; 20];
        assert!(!resolved(&once, 30, 30));
        // the same data with every sample repeated ten times: still 20 blocks
        let tenfold: Vec<u64> = once.iter().map(|k| k * 10).collect();
        assert!(!resolved(&tenfold, 30, 30));
        // thirty real blocks resolve it
        assert!(resolved(&vec![40u64; 30], 30, 30));
    }
}
