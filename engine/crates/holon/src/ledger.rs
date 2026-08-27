//! The LEDGER: what the dynamics pays to keep the representation honest.
//! At tier 1 it is a sign (mod 4, inside each Pauli row). At tier 2 it is the
//! exact cyclotomic ring Z[ω]·2^{−m/2} over branch lists. At the ℝ tiers it
//! is budgets and rent accounting. Same slot, richer ring — the deformation
//! is priced by the walls (DATA_OBJECT.md).

/// Exact cyclotomic scalar (c0 + c1ω + c2ω² + c3ω³)·2^{−m/2}, ω = e^{iπ/4}.
/// Transplanted from holon-qasm's certified magic tier (QASM-2, five of five)
/// so THE holon crate owns its own ledger ring.
///
/// THE EXACTNESS ENVELOPE, ENFORCED: every coefficient must fit in i128, and
/// the ring REFUSES (panics) on overflow rather than wrapping — a wrapped
/// coefficient would be a silently wrong exact value, which is the one output
/// this engine may never produce. The envelope is real, not hypothetical:
/// Quist–Coopmans–Laarman (arXiv:2602.17775) bound Clifford+T amplitude
/// coefficients by 2^{O(n+t)}, so fixed width WILL be exceeded at some
/// circuit size; when it is, the answer is a refusal, never a number.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cyc {
    pub c: [i128; 4],
    pub m: i32,
}

impl Cyc {
    pub const ONE: Cyc = Cyc { c: [1, 0, 0, 0], m: 0 };
    pub const ZERO: Cyc = Cyc { c: [0, 0, 0, 0], m: 0 };

    #[cfg(test)]
    pub(crate) fn normalize_pub(self) -> Cyc {
        self.normalize()
    }

    fn normalize(mut self) -> Cyc {
        while self.m >= 2
            && self.c.iter().all(|&x| x % 2 == 0)
            && self.c.iter().any(|&x| x != 0)
        {
            for x in &mut self.c {
                *x /= 2;
            }
            self.m -= 2;
        }
        if self.c.iter().all(|&x| x == 0) {
            self.m = 0;
        }
        self
    }

    #[inline]
    fn refuse() -> ! {
        panic!(
            "Cyc: coefficient overflow — the i128 exactness envelope is exceeded; \
             refusing rather than wrapping (a wrapped coefficient is a silently \
             wrong exact value)"
        )
    }

    #[inline]
    fn cadd(a: i128, b: i128) -> i128 {
        match a.checked_add(b) {
            Some(v) => v,
            None => Self::refuse(),
        }
    }

    #[inline]
    fn cmul(a: i128, b: i128) -> i128 {
        match a.checked_mul(b) {
            Some(v) => v,
            None => Self::refuse(),
        }
    }

    #[inline]
    fn csub(a: i128, b: i128) -> i128 {
        match a.checked_sub(b) {
            Some(v) => v,
            None => Self::refuse(),
        }
    }

    pub fn mul(self, o: Cyc) -> Cyc {
        let mut acc = [0i128; 8];
        for a in 0..4 {
            for b in 0..4 {
                acc[a + b] = Self::cadd(acc[a + b], Self::cmul(self.c[a], o.c[b]));
            }
        }
        let mut out = [0i128; 4];
        for a in 0..4 {
            out[a] = Self::csub(acc[a], acc[a + 4]);
        }
        Cyc { c: out, m: self.m + o.m }.normalize()
    }

    /// One-shot denominator alignment then add (a loop of √2-multiplies
    /// fights normalize() and cycles — the measured hang, QASM-2 dev record).
    pub fn add(self, o: Cyc) -> Cyc {
        if self.c.iter().all(|&x| x == 0) {
            return o;
        }
        if o.c.iter().all(|&x| x == 0) {
            return self;
        }
        let (mut a, mut b) = (self, o);
        if a.m > b.m {
            std::mem::swap(&mut a, &mut b);
        }
        let delta = (b.m - a.m) as u32;
        let half = delta / 2;
        if half >= 127 {
            Self::refuse();
        }
        let pow = 1i128 << half;
        for x in &mut a.c {
            *x = Self::cmul(*x, pow);
        }
        if delta % 2 == 1 {
            let t = a.c;
            let mut acc = [0i128; 8];
            for p in 0..4 {
                acc[p + 1] = Self::cadd(acc[p + 1], t[p]);
                acc[p + 3] = Self::csub(acc[p + 3], t[p]);
            }
            for p in 0..4 {
                a.c[p] = Self::csub(acc[p], acc[p + 4]);
            }
        }
        a.m = b.m;
        Cyc {
            c: [
                Self::cadd(a.c[0], b.c[0]),
                Self::cadd(a.c[1], b.c[1]),
                Self::cadd(a.c[2], b.c[2]),
                Self::cadd(a.c[3], b.c[3]),
            ],
            m: a.m,
        }
        .normalize()
    }

    /// `self · i^k`, exactly — and BIT-IDENTICALLY to `self.mul(i_pow(k))`.
    ///
    /// `i^k = ω^{2k}` is a UNIT with a single ±1 coefficient, so multiplying
    /// by it is a permutation of the coefficient vector with signs: the
    /// convolution `Cyc::mul` performs has exactly one contributing term per
    /// output slot, `out[(a+e) mod 4] = ±c[a]` with the sign negative iff
    /// `a + e ≥ 4` (that is `ω⁴ = −1`, which is the only reduction there is).
    /// The same `normalize` runs afterwards, so this is not an approximation
    /// of the general multiply and not a fast path with different rounding —
    /// there is no rounding. `unit_multiply_is_the_general_multiply` pins the
    /// identity on random inputs across every `k` and both parities of `m`.
    ///
    /// It is here rather than in a consumer because the ring is the ring's
    /// business, and because both the per-branch engine (`affine::Affine`'s
    /// phase updates) and the branch-sliced one (`sliced`, once per lane per
    /// phase update) spend most of their ring time in exactly this shape.
    pub fn mul_i_pow(self, k: u8) -> Cyc {
        let e = (2 * (k % 4)) as usize; // i = ω²
        let mut out = [0i128; 4];
        for a in 0..4 {
            let t = (a + e) % 8;
            if t >= 4 {
                out[t - 4] = -self.c[a];
            } else {
                out[t] = self.c[a];
            }
        }
        Cyc { c: out, m: self.m }.normalize()
    }

    pub fn to_complex(self) -> (f64, f64) {
        let s = std::f64::consts::FRAC_1_SQRT_2;
        let w = [(1.0, 0.0), (s, s), (0.0, 1.0), (-s, s)];
        let (mut re, mut im) = (0.0, 0.0);
        for k in 0..4 {
            re += self.c[k] as f64 * w[k].0;
            im += self.c[k] as f64 * w[k].1;
        }
        let scale = (2.0f64).powf(-(self.m as f64) / 2.0);
        (re * scale, im * scale)
    }
}

#[cfg(test)]
mod envelope_tests {
    use super::Cyc;

    // The envelope must REFUSE, never wrap: a wrapped coefficient is a
    // silently wrong exact value. These tests pin the refusal.
    #[test]
    #[should_panic(expected = "exactness envelope")]
    fn mul_overflow_refuses() {
        let big = Cyc { c: [i128::MAX / 2, 0, 0, 0], m: 0 };
        let _ = big.mul(Cyc { c: [3, 0, 0, 0], m: 0 });
    }

    #[test]
    #[should_panic(expected = "exactness envelope")]
    fn add_overflow_refuses() {
        let big = Cyc { c: [i128::MAX, 0, 0, 0], m: 0 };
        let _ = big.add(big);
    }

    #[test]
    #[should_panic(expected = "exactness envelope")]
    fn alignment_overflow_refuses() {
        // Denominator alignment multiplies by 2^{delta/2}; a huge exponent
        // gap must refuse, not shift bits off the top.
        let a = Cyc { c: [i128::MAX / 2, 0, 0, 0], m: 0 };
        let b = Cyc { c: [1, 0, 0, 0], m: 40 };
        let _ = a.add(b);
    }

    /// The unit multiply is the general multiply, bit for bit — coefficient
    /// vector and denominator exponent alike, for every power of i, on random
    /// coefficients at both parities of `m`.
    #[test]
    fn unit_multiply_is_the_general_multiply() {
        fn i_pow(k: u8) -> Cyc {
            let mut c = [0i128; 4];
            match k % 4 {
                0 => c[0] = 1,
                1 => c[2] = 1,
                2 => c[0] = -1,
                _ => c[2] = -1,
            }
            Cyc { c, m: 0 }
        }
        let mut seed = 0xdead_beef_1234_5678u64;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as i128 - (1 << 30)
        };
        for _ in 0..2000 {
            for m in [-3i32, -2, -1, 0, 1, 2, 3, 8, 9] {
                let x = Cyc { c: [next(), next(), next(), next()], m };
                for k in 0..8u8 {
                    assert_eq!(
                        x.mul_i_pow(k),
                        x.mul(i_pow(k)),
                        "x = {x:?}, k = {k}"
                    );
                }
            }
        }
        // Including the degenerate operands, where `normalize` has opinions.
        for m in [0i32, 1, 2, 5] {
            for k in 0..4u8 {
                let z = Cyc { c: [0, 0, 0, 0], m };
                assert_eq!(z.mul_i_pow(k), z.mul(i_pow(k)));
                let e = Cyc { c: [4, 4, -8, 0], m };
                assert_eq!(e.mul_i_pow(k), e.mul(i_pow(k)));
            }
        }
    }

    // In-envelope arithmetic is bit-for-bit what it was before the guards.
    #[test]
    fn in_envelope_unchanged() {
        let x = Cyc { c: [3, -1, 4, -1], m: 3 };
        let y = Cyc { c: [-2, 7, 0, 5], m: 1 };
        assert_eq!(x.mul(y), Cyc { c: [6, 3, -10, 45], m: 4 });
        let s = x.add(y);
        assert_eq!(s.m, 3);
    }
}
