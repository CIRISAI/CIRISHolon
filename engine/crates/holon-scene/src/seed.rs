//! Seeds with random access.
//!
//! A child's seed is a function of its parent's seed and its own index, so any holon is
//! reached from its chain of ancestors alone, without generating a sibling. A boundary
//! shared by two holons is seeded from the PAIR, symmetrically, so both sides draw the same
//! thing without either generating the other.
//!
//! The mixer is SplitMix64's finaliser: wrapping integer arithmetic only, so the stream is
//! identical on every platform.

const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// One round of the SplitMix64 finaliser.
pub fn mix(z: u64) -> u64 {
    let mut z = z.wrapping_add(GAMMA);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The seed of child `index` of a holon whose seed is `parent`.
pub fn child_seed(parent: u64, index: u64) -> u64 {
    mix(parent ^ mix(index.wrapping_add(0xD1B5_4A32_D192_ED03)))
}

/// The seed of what two holons share. `pair_seed(a, b) == pair_seed(b, a)`.
pub fn pair_seed(a: u64, b: u64) -> u64 {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    mix(mix(lo) ^ hi.rotate_left(32))
}

/// A stream of draws from one seed.
#[derive(Clone, Debug)]
pub struct Stream(u64);

impl Stream {
    pub fn new(seed: u64) -> Self {
        Stream(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        let out = mix(self.0);
        self.0 = self.0.wrapping_add(GAMMA);
        out
    }
    /// A draw in `0..n`, unbiased, by rejection. `n` must be positive.
    pub fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "below(0)");
        let lim = u64::MAX - (u64::MAX % n);
        loop {
            let x = self.next_u64();
            if x < lim {
                return x % n;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_vectors() {
        // Pinned so a port (the twin's JavaScript, a shader) can check itself against these.
        assert_eq!(mix(0), 0xE220_A839_7B1D_CDAF);
        assert_eq!(mix(1), 0x910A_2DEC_8902_5CC1);
        let mut s = Stream::new(0);
        assert_eq!(s.next_u64(), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn pair_seed_is_symmetric_and_separates() {
        assert_eq!(pair_seed(3, 9), pair_seed(9, 3));
        assert_ne!(pair_seed(3, 9), pair_seed(3, 10));
        assert_ne!(pair_seed(3, 9), pair_seed(9, 9));
    }

    #[test]
    fn child_seeds_differ_by_index_and_parent() {
        let mut seen = std::collections::HashSet::new();
        for p in 0..200u64 {
            for i in 0..50u64 {
                assert!(seen.insert(child_seed(p, i)));
            }
        }
    }

    #[test]
    fn below_is_in_range_and_roughly_flat() {
        let mut s = Stream::new(42);
        let mut c = [0u32; 7];
        for _ in 0..70_000 {
            c[s.below(7) as usize] += 1;
        }
        for &k in &c {
            assert!((9_500..10_500).contains(&k), "{c:?}");
        }
    }
}
