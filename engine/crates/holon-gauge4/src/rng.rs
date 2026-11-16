//! A seeded stream, the same on every machine: xoshiro256++ (Blackman and Vigna 2021,
//! doi:10.1145/3460772), its state filled by splitmix64.
#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        let mut z = seed;
        let mut next = || {
            z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut x = z;
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            x ^ (x >> 31)
        };
        Rng { s: [next(), next(), next(), next()] }
    }
    #[inline]
    pub fn u64(&mut self) -> u64 {
        let r = self.s[0].wrapping_add(self.s[3]).rotate_left(23).wrapping_add(self.s[0]);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        r
    }
    /// Uniform in [0, 1).
    #[inline]
    pub fn f(&mut self) -> f64 {
        (self.u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    /// Uniform in (0, 1]: safe under a logarithm.
    #[inline]
    pub fn open(&mut self) -> f64 {
        1.0 - self.f()
    }
    /// A standard normal deviate (Box-Muller).
    pub fn normal(&mut self) -> f64 {
        (-2.0 * self.open().ln()).sqrt() * (std::f64::consts::TAU * self.f()).cos()
    }
}
