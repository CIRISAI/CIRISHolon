//! The LEDGER: what the dynamics pays to keep the representation honest.
//! At tier 1 it is a sign (mod 4, inside each Pauli row). At tier 2 it is the
//! exact cyclotomic ring Z[ω]·2^{−m/2} over branch lists. At the ℝ tiers it
//! is budgets and rent accounting. Same slot, richer ring — the deformation
//! is priced by the walls (DATA_OBJECT.md).

/// Exact cyclotomic scalar (c0 + c1ω + c2ω² + c3ω³)·2^{−m/2}, ω = e^{iπ/4}.
/// Transplanted from holon-qasm's certified magic tier (QASM-2, five of five)
/// so THE holon crate owns its own ledger ring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cyc {
    pub c: [i128; 4],
    pub m: i32,
}

impl Cyc {
    pub const ONE: Cyc = Cyc { c: [1, 0, 0, 0], m: 0 };
    pub const ZERO: Cyc = Cyc { c: [0, 0, 0, 0], m: 0 };

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

    pub fn mul(self, o: Cyc) -> Cyc {
        let mut acc = [0i128; 8];
        for a in 0..4 {
            for b in 0..4 {
                acc[a + b] += self.c[a] * o.c[b];
            }
        }
        let mut out = [0i128; 4];
        for a in 0..4 {
            out[a] = acc[a] - acc[a + 4];
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
        for x in &mut a.c {
            *x <<= delta / 2;
        }
        if delta % 2 == 1 {
            let t = a.c;
            let mut acc = [0i128; 8];
            for p in 0..4 {
                acc[p + 1] += t[p];
                acc[p + 3] -= t[p];
            }
            for p in 0..4 {
                a.c[p] = acc[p] - acc[p + 4];
            }
        }
        a.m = b.m;
        Cyc {
            c: [a.c[0] + b.c[0], a.c[1] + b.c[1], a.c[2] + b.c[2], a.c[3] + b.c[3]],
            m: a.m,
        }
        .normalize()
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
