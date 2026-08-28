//! THE GENERAL CYCLOTOMIC RING `Z[ζ_n]`, for ANY n — the tower's completion.
//!
//! `cyclo.rs` carries the 2-power tower fast (reduction is just `ζ^d = −1`).
//! This module carries EVERY n, including the rungs a power of two cannot
//! reach: `ζ9` (qutrit Strange-state magic — the one gap the tower named),
//! `ζ3`, `ζ5`, `ζ7`, and the mixed orders. Elements are integer polynomials
//! modulo the cyclotomic polynomial `Φ_n`, which is itself computed exactly
//! here by the recursive division
//!
//! ```text
//!     Φ_n(x) = (x^n − 1) / Π_{d | n, d < n} Φ_d(x),
//! ```
//!
//! over ℤ — no tables, no floats, no trusted constants. With `Φ_n` in hand
//! the ring is ordinary polynomial arithmetic mod `Φ_n`, and `deg Φ_n = φ(n)`
//! is the ring's degree.
//!
//! What this completes: with the 2-power tower, `Z[ζ8][√3]`, and this, the
//! classification in TIERS.md has NO unimplemented rung — every phase lying
//! in an abelian extension of ℚ (Kronecker–Weber: every exactly-representable
//! rotation there is) has a ring here that carries it. Generic angles remain
//! handled by the symbolic carrier, which needs no ring at all.

/// Integer polynomial arithmetic, exact.
fn poly_mul(a: &[i128], b: &[i128]) -> Vec<i128> {
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let mut out = vec![0i128; a.len() + b.len() - 1];
    for (i, &x) in a.iter().enumerate() {
        if x == 0 {
            continue;
        }
        for (j, &y) in b.iter().enumerate() {
            out[i + j] = out[i + j]
                .checked_add(x.checked_mul(y).expect("cyclon: overflow"))
                .expect("cyclon: overflow");
        }
    }
    out
}

/// Exact division `a / b` for integer polynomials, when it divides evenly.
fn poly_div_exact(a: &[i128], b: &[i128]) -> Vec<i128> {
    let mut r = a.to_vec();
    let bd = b.len() - 1;
    let lead = *b.last().expect("cyclon: divide by empty");
    assert!(lead == 1 || lead == -1, "cyclotomic divisors are monic");
    if r.len() < b.len() {
        return vec![];
    }
    let mut q = vec![0i128; r.len() - bd];
    for i in (0..q.len()).rev() {
        let c = r[i + bd] / lead;
        q[i] = c;
        if c != 0 {
            for (j, &bj) in b.iter().enumerate() {
                r[i + j] -= c * bj;
            }
        }
    }
    assert!(r.iter().all(|&x| x == 0), "cyclon: division was not exact");
    q
}

/// `Φ_n(x)`, computed exactly by recursive division. Coefficients are
/// little-endian (`c[0] + c[1]x + …`).
pub fn cyclotomic(n: usize) -> Vec<i128> {
    assert!(n >= 1, "Φ_n needs n ≥ 1");
    // x^n − 1
    let mut num = vec![0i128; n + 1];
    num[0] = -1;
    num[n] = 1;
    for d in 1..n {
        if n % d == 0 {
            let phi_d = cyclotomic(d);
            num = poly_div_exact(&num, &phi_d);
        }
    }
    num
}

/// An element of `Z[ζ_n]`: `Σ cᵢ ζ^i` reduced mod `Φ_n`, with a global
/// `√2`-exponent slot so the ring composes with the engine's scaling
/// convention (`value = (Σ cᵢ ζ^i) · 2^{−m/2}`).
#[derive(Clone, Debug, PartialEq)]
pub struct CycloN {
    pub n: usize,
    /// length = φ(n) = deg Φ_n
    pub c: Vec<i128>,
    pub m: i32,
    /// Φ_n, cached with the element so reduction never re-derives it.
    phi: std::rc::Rc<Vec<i128>>,
}

fn ck(a: i128, b: i128, op: fn(i128, i128) -> Option<i128>) -> i128 {
    op(a, b).unwrap_or_else(|| {
        panic!(
            "CycloN: coefficient overflow — the i128 exactness envelope is \
             exceeded; refusing rather than wrapping"
        )
    })
}

impl CycloN {
    /// A fresh ring at order `n`. `Φ_n` is computed once here.
    pub fn ring(n: usize) -> std::rc::Rc<Vec<i128>> {
        std::rc::Rc::new(cyclotomic(n))
    }

    pub fn degree(phi: &[i128]) -> usize {
        phi.len() - 1
    }

    pub fn zero(n: usize, phi: std::rc::Rc<Vec<i128>>) -> CycloN {
        let d = Self::degree(&phi);
        CycloN { n, c: vec![0; d], m: 0, phi }
    }

    pub fn one(n: usize, phi: std::rc::Rc<Vec<i128>>) -> CycloN {
        let mut z = Self::zero(n, phi);
        z.c[0] = 1;
        z
    }

    /// `ζ_n^j`, exactly (reduced into the ring's basis).
    pub fn zeta_pow(n: usize, phi: std::rc::Rc<Vec<i128>>, j: i64) -> CycloN {
        let j = j.rem_euclid(n as i64) as usize;
        let d = Self::degree(&phi);
        let mut raw = vec![0i128; j + 1];
        raw[j] = 1;
        let c = reduce(&raw, &phi, d);
        CycloN { n, c, m: 0, phi }
    }

    pub fn is_zero(&self) -> bool {
        self.c.iter().all(|&x| x == 0)
    }

    fn align(&self, o: &CycloN) -> (CycloN, CycloN) {
        assert_eq!(self.n, o.n, "ring orders must match");
        let t = self.m.max(o.m);
        (self.clone().raise(t), o.clone().raise(t))
    }

    fn raise(mut self, target: i32) -> CycloN {
        let mut d = target - self.m;
        while d >= 2 {
            for x in self.c.iter_mut() {
                *x = ck(*x, 2, i128::checked_mul);
            }
            self.m += 2;
            d -= 2;
        }
        assert!(d == 0, "odd √2 alignment needs the base ring's √2; use n divisible by 8");
        self
    }

    pub fn add(&self, o: &CycloN) -> CycloN {
        let (x, y) = self.align(o);
        let c = x.c.iter().zip(y.c.iter()).map(|(a, b)| ck(*a, *b, i128::checked_add)).collect();
        CycloN { n: self.n, c, m: x.m, phi: self.phi.clone() }.normalize()
    }

    pub fn sub(&self, o: &CycloN) -> CycloN {
        let (x, y) = self.align(o);
        let c = x.c.iter().zip(y.c.iter()).map(|(a, b)| ck(*a, *b, i128::checked_sub)).collect();
        CycloN { n: self.n, c, m: x.m, phi: self.phi.clone() }.normalize()
    }

    pub fn mul(&self, o: &CycloN) -> CycloN {
        assert_eq!(self.n, o.n, "ring orders must match");
        let raw = poly_mul(&self.c, &o.c);
        let d = Self::degree(&self.phi);
        let c = reduce(&raw, &self.phi, d);
        CycloN { n: self.n, c, m: self.m + o.m, phi: self.phi.clone() }.normalize()
    }

    fn normalize(mut self) -> CycloN {
        while self.m >= 2 && self.c.iter().all(|x| x % 2 == 0) && self.c.iter().any(|&x| x != 0) {
            for x in self.c.iter_mut() {
                *x /= 2;
            }
            self.m -= 2;
        }
        if self.c.iter().all(|&x| x == 0) {
            self.m = 0;
        }
        self
    }

    /// Numeric value — display and oracle only.
    pub fn to_complex(&self) -> (f64, f64) {
        let (mut re, mut im) = (0.0, 0.0);
        for (i, &ci) in self.c.iter().enumerate() {
            let ang = 2.0 * std::f64::consts::PI * i as f64 / self.n as f64;
            re += ci as f64 * ang.cos();
            im += ci as f64 * ang.sin();
        }
        let s = 2f64.powf(-(self.m as f64) / 2.0);
        (re * s, im * s)
    }
}

/// Reduce a raw coefficient vector mod `Φ_n` into the degree-`d` basis.
fn reduce(raw: &[i128], phi: &[i128], d: usize) -> Vec<i128> {
    let mut r = raw.to_vec();
    if r.len() < d {
        r.resize(d, 0);
    }
    for i in (d..r.len()).rev() {
        let c = r[i];
        if c == 0 {
            continue;
        }
        r[i] = 0;
        // x^i = x^{i-d} · x^d, and x^d = −(Φ's lower terms)
        for (j, &pj) in phi.iter().take(d).enumerate() {
            r[i - d + j] = ck(r[i - d + j], ck(c, pj, i128::checked_mul), i128::checked_sub);
        }
    }
    r.truncate(d);
    r
}

/// The qutrit magic rung the tower named: `Z[ζ9]`, degree 6, `Φ₉ = x⁶+x³+1`.
pub fn zeta9_ring() -> std::rc::Rc<Vec<i128>> {
    CycloN::ring(9)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(x: (f64, f64), y: (f64, f64), tag: &str) {
        assert!((x.0 - y.0).abs() < 1e-9 && (x.1 - y.1).abs() < 1e-9, "{tag}: {x:?} vs {y:?}");
    }

    /// Φ_n computed from scratch must match the classical values.
    #[test]
    fn cyclotomic_polynomials_are_exact() {
        assert_eq!(cyclotomic(1), vec![-1, 1]); // x − 1
        assert_eq!(cyclotomic(2), vec![1, 1]); // x + 1
        assert_eq!(cyclotomic(3), vec![1, 1, 1]); // x² + x + 1
        assert_eq!(cyclotomic(4), vec![1, 0, 1]); // x² + 1
        assert_eq!(cyclotomic(8), vec![1, 0, 0, 0, 1]); // x⁴ + 1
        // THE QUTRIT RUNG: Φ₉ = x⁶ + x³ + 1
        assert_eq!(cyclotomic(9), vec![1, 0, 0, 1, 0, 0, 1]);
        assert_eq!(cyclotomic(12), vec![1, 0, -1, 0, 1]); // x⁴ − x² + 1
        // degrees are φ(n)
        for (n, phi_n) in [(3usize, 2usize), (8, 4), (9, 6), (12, 4), (24, 8), (5, 4), (7, 6)] {
            assert_eq!(cyclotomic(n).len() - 1, phi_n, "deg Φ_{n}");
        }
    }

    /// ζ9 powers are exact, and ζ9⁹ = 1 in the ring.
    #[test]
    fn zeta9_is_exact_and_closes() {
        let phi = zeta9_ring();
        for j in 0..18i64 {
            let z = CycloN::zeta_pow(9, phi.clone(), j);
            let ang = 2.0 * std::f64::consts::PI * (j.rem_euclid(9)) as f64 / 9.0;
            close(z.to_complex(), (ang.cos(), ang.sin()), &format!("ζ9^{j}"));
        }
        // ζ9³ is a primitive cube root: (ζ9³)³ = 1
        let z3 = CycloN::zeta_pow(9, phi.clone(), 3);
        let cube = z3.mul(&z3).mul(&z3);
        assert_eq!(cube, CycloN::one(9, phi.clone()), "ζ9⁹ = 1 exactly");
        // and ring arithmetic tracks the complex values
        let a = CycloN::zeta_pow(9, phi.clone(), 2);
        let b = CycloN::zeta_pow(9, phi.clone(), 5);
        close(a.mul(&b).to_complex(), CycloN::zeta_pow(9, phi.clone(), 7).to_complex(), "ζ9²·ζ9⁵");
        close(
            a.add(&b).to_complex(),
            {
                let (ar, ai) = a.to_complex();
                let (br, bi) = b.to_complex();
                (ar + br, ai + bi)
            },
            "add",
        );
    }

    /// The general ring reproduces the 2-power tower where they overlap —
    /// one classification, two implementations, same values.
    #[test]
    fn general_ring_agrees_with_the_two_power_tower() {
        for n in [8usize, 16] {
            let phi = CycloN::ring(n);
            for j in 0..(2 * n as i64) {
                let g = CycloN::zeta_pow(n, phi.clone(), j).to_complex();
                let k = (n as f64).log2() as u32 + 1;
                let t = crate::cyclo::Cyclo::zeta_pow(k, 2 * j).to_complex();
                close(g, t, &format!("n={n} j={j}"));
            }
        }
    }

    #[test]
    #[should_panic(expected = "exactness envelope")]
    fn overflow_refuses() {
        let phi = zeta9_ring();
        let mut big = CycloN::one(9, phi);
        big.c[0] = i128::MAX;
        let _ = big.add(&big.clone());
    }
}
