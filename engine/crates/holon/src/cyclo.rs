//! THE COMPLETE EXACT-RING TOWER — one recursive ring object, not a zoo.
//!
//! The classification (Kronecker–Weber, applied): a diagonal gate
//! `diag(1, z)` is exactly representable iff `z` is an algebraic number in an
//! abelian extension of ℚ — equivalently iff `z ∈ ℚ(ζ_n)` for some `n`. So
//! "which rotations can this engine do exactly" has a complete answer, and
//! the wild gate families sort into exactly four rungs:
//!
//! | family | phase | lands in |
//! |---|---|---|
//! | Clifford+T (T, S, Z, CCZ…) | ζ8 | `Z[ζ8]` = `ledger::Cyc` ✓ |
//! | face / T-type magic (arccos(1/√3)) | NOT a root of unity, but in ℚ(ζ24) | `Z[ζ8][√3]` = `face::R3` ✓ |
//! | qutrit magic ζ3, and every 24th root | ζ3, ζ12, ζ24 | ALSO `face::R3` — because ζ3 = (−1+i√3)/2 and the ring already carries i, √3, ½ ✓ |
//! | the Clifford HIERARCHY: rz(π/8), rz(π/16), … | ζ16, ζ32, ζ64 | **this module** |
//!
//! The one rung genuinely outside the tower is `ζ9` (qutrit Strange-state
//! magic), which needs a cubic rather than a 2-power extension; it is named
//! here and not pretended.
//!
//! This module supplies the 2-power tower `Z[ζ_{2^k}]` for any k, because
//! qiskit-shaped circuits use rz(π/8) and finer constantly. Representation:
//! coefficients over the basis `1, ζ, …, ζ^{d−1}` with `d = 2^{k−1}` and the
//! single reduction `ζ^d = −1` (the cyclotomic polynomial for a 2-power is
//! `x^d + 1`), times a global `√2` exponent — the same shape `Cyc` uses, so
//! `Cyc` IS the `k = 3` instance and this generalizes rather than replaces.

use crate::ledger::Cyc;

/// An element of `Z[ζ_{2^k}]`: coefficients over `1, ζ, …, ζ^{d−1}`,
/// `d = 2^{k−1}`, value `(Σ cᵢ ζ^i) · 2^{−m/2}`. Reduction is `ζ^d = −1`.
#[derive(Clone, Debug, PartialEq)]
pub struct Cyclo {
    /// `k` in `ζ = ζ_{2^k}`; the degree is `d = 2^{k−1}`.
    pub k: u32,
    pub c: Vec<i128>,
    pub m: i32,
}

fn refuse_overflow() -> ! {
    panic!(
        "Cyclo: coefficient overflow — the i128 exactness envelope is exceeded; \
         refusing rather than wrapping (route to the residue carrier)"
    )
}

#[inline]
fn cadd(a: i128, b: i128) -> i128 {
    a.checked_add(b).unwrap_or_else(|| refuse_overflow())
}
#[inline]
fn csub(a: i128, b: i128) -> i128 {
    a.checked_sub(b).unwrap_or_else(|| refuse_overflow())
}
#[inline]
fn cmul(a: i128, b: i128) -> i128 {
    a.checked_mul(b).unwrap_or_else(|| refuse_overflow())
}

impl Cyclo {
    pub fn degree(k: u32) -> usize {
        assert!(k >= 2, "the 2-power tower starts at ζ4 = i");
        1usize << (k - 1)
    }

    pub fn zero(k: u32) -> Cyclo {
        Cyclo { k, c: vec![0; Self::degree(k)], m: 0 }
    }

    pub fn one(k: u32) -> Cyclo {
        let mut z = Self::zero(k);
        z.c[0] = 1;
        z
    }

    /// `ζ_{2^k}^j` — the exact primitive root's power.
    pub fn zeta_pow(k: u32, j: i64) -> Cyclo {
        let d = Self::degree(k) as i64;
        let j = j.rem_euclid(2 * d);
        let mut z = Self::zero(k);
        if j < d {
            z.c[j as usize] = 1;
        } else {
            z.c[(j - d) as usize] = -1; // ζ^d = −1
        }
        z
    }

    /// Embed a base-ring value: `Cyc` lives at `k = 3` and embeds into any
    /// `k' ≥ 3` by `ζ8 = ζ_{2^k'}^{2^{k'−3}}` — the tower's inclusion map,
    /// which is what makes this a generalization and not a parallel ring.
    pub fn from_cyc(x: Cyc, k: u32) -> Cyclo {
        assert!(k >= 3, "Cyc embeds only from ζ8 upward");
        let step = 1i64 << (k - 3);
        let mut out = Self::zero(k);
        for (i, &ci) in x.c.iter().enumerate() {
            if ci != 0 {
                let t = Self::zeta_pow(k, step * i as i64);
                for (o, tv) in out.c.iter_mut().zip(t.c.iter()) {
                    *o = cadd(*o, cmul(ci, *tv));
                }
            }
        }
        out.m = x.m;
        out
    }

    fn align(&self, other: &Cyclo) -> (Cyclo, Cyclo) {
        assert_eq!(self.k, other.k, "ring degrees must match");
        let t = self.m.max(other.m);
        (self.clone().raise_to(t), other.clone().raise_to(t))
    }

    /// Multiply by `√2^Δ` exactly. `√2 = ζ8 − ζ8³`, and in the 2-power tower
    /// `ζ8 = ζ^{2^{k−3}}`, so this is a lattice rotation — no rounding.
    fn raise_to(mut self, target: i32) -> Cyclo {
        let mut delta = target - self.m;
        if delta <= 0 {
            return self;
        }
        // even part: multiply by 2 per two halves
        while delta >= 2 {
            for x in self.c.iter_mut() {
                *x = cmul(*x, 2);
            }
            self.m += 2;
            delta -= 2;
        }
        if delta == 1 {
            let s2 = Self::sqrt2(self.k);
            self = self.mul(&s2);
            self.m += 1;
        }
        self
    }

    /// `√2` in this ring (requires k ≥ 3).
    pub fn sqrt2(k: u32) -> Cyclo {
        assert!(k >= 3, "√2 needs ζ8 or finer");
        let step = 1i64 << (k - 3);
        let a = Self::zeta_pow(k, step);
        let b = Self::zeta_pow(k, 3 * step);
        a.sub(&b)
    }

    pub fn add(&self, o: &Cyclo) -> Cyclo {
        let (x, y) = self.align(o);
        let c = x.c.iter().zip(y.c.iter()).map(|(a, b)| cadd(*a, *b)).collect();
        Cyclo { k: self.k, c, m: x.m }.normalize()
    }

    pub fn sub(&self, o: &Cyclo) -> Cyclo {
        let (x, y) = self.align(o);
        let c = x.c.iter().zip(y.c.iter()).map(|(a, b)| csub(*a, *b)).collect();
        Cyclo { k: self.k, c, m: x.m }.normalize()
    }

    pub fn mul(&self, o: &Cyclo) -> Cyclo {
        assert_eq!(self.k, o.k, "ring degrees must match");
        let d = self.c.len();
        let mut acc = vec![0i128; 2 * d];
        for (i, &a) in self.c.iter().enumerate() {
            if a == 0 {
                continue;
            }
            for (j, &b) in o.c.iter().enumerate() {
                if b != 0 {
                    acc[i + j] = cadd(acc[i + j], cmul(a, b));
                }
            }
        }
        let mut c = vec![0i128; d];
        for i in 0..d {
            c[i] = csub(acc[i], acc[i + d]); // ζ^d = −1
        }
        Cyclo { k: self.k, c, m: self.m + o.m }.normalize()
    }

    /// Complex conjugate: `ζ̄ = ζ^{−1} = −ζ^{d−1}`.
    pub fn conj(&self) -> Cyclo {
        let d = self.c.len();
        let mut out = Self::zero(self.k);
        out.m = self.m;
        for (i, &ci) in self.c.iter().enumerate() {
            if ci == 0 {
                continue;
            }
            let t = Self::zeta_pow(self.k, -(i as i64));
            for (o, tv) in out.c.iter_mut().zip(t.c.iter()) {
                *o = cadd(*o, cmul(ci, *tv));
            }
        }
        out
    }

    fn normalize(mut self) -> Cyclo {
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

    pub fn is_zero(&self) -> bool {
        self.c.iter().all(|&x| x == 0)
    }

    /// Numeric value — display and oracle only.
    pub fn to_complex(&self) -> (f64, f64) {
        let d = self.c.len() as f64;
        let (mut re, mut im) = (0.0, 0.0);
        for (i, &ci) in self.c.iter().enumerate() {
            let ang = std::f64::consts::PI * i as f64 / d;
            re += ci as f64 * ang.cos();
            im += ci as f64 * ang.sin();
        }
        let s = 2f64.powf(-(self.m as f64) / 2.0);
        (re * s, im * s)
    }
}

/// The tower's classification, as data: which ring a rotation angle needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RingFor {
    /// `Z[ζ_{2^k}]` — every `rz(2πj/2^k)`; k=3 is Clifford+T.
    TwoPower(u32),
    /// `Z[ζ8][√3]` — face/T-type magic, ζ3, every 24th root.
    Sqrt3,
    /// Outside the implemented tower: named, never pretended.
    Unimplemented(&'static str),
}

/// Classify `θ`: the smallest implemented ring carrying `e^{iθ}` exactly.
pub fn ring_for(theta: f64) -> RingFor {
    const TF: f64 = 0.955_316_618_124_509_2; // arccos(1/√3)
    if (theta.abs() - TF).abs() < 1e-12 {
        return RingFor::Sqrt3;
    }
    for k in 2..=10u32 {
        let d = (1u64 << k) as f64;
        let j = theta / (2.0 * std::f64::consts::PI / d);
        if (j - j.round()).abs() < 1e-12 {
            return RingFor::TwoPower(k);
        }
    }
    // ζ3 family (and ζ6, ζ12, ζ24) — all inside R3
    for n in [3u32, 6, 12, 24] {
        let j = theta / (2.0 * std::f64::consts::PI / n as f64);
        if (j - j.round()).abs() < 1e-12 {
            return RingFor::Sqrt3;
        }
    }
    RingFor::Unimplemented(
        "not in the implemented tower (2-power cyclotomic, or Z[ζ8][√3]); \
         ζ9-class qutrit magic and generic angles need synthesis or a new rung",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(x: (f64, f64), y: (f64, f64), tag: &str) {
        assert!((x.0 - y.0).abs() < 1e-9 && (x.1 - y.1).abs() < 1e-9, "{tag}: {x:?} vs {y:?}");
    }

    #[test]
    fn zeta_powers_are_exact_at_every_rung() {
        for k in 3..=6u32 {
            let d = Cyclo::degree(k) as i64;
            for j in 0..(2 * d) {
                let z = Cyclo::zeta_pow(k, j);
                let ang = std::f64::consts::PI * j as f64 / d as f64;
                close(z.to_complex(), (ang.cos(), ang.sin()), &format!("k={k} j={j}"));
            }
            // ζ^{2d} = 1
            assert_eq!(Cyclo::zeta_pow(k, 2 * d), Cyclo::one(k));
        }
    }

    #[test]
    fn sqrt2_squares_to_two_at_every_rung() {
        for k in 3..=6u32 {
            let s = Cyclo::sqrt2(k);
            let sq = s.mul(&s);
            close(sq.to_complex(), (2.0, 0.0), &format!("k={k}"));
        }
    }

    /// The tower's inclusion: `Cyc` (ζ8) embeds into every finer rung with
    /// its value preserved — this is what makes it one ring, not a zoo.
    #[test]
    fn cyc_embeds_into_the_tower() {
        let x = Cyc { c: [3, -1, 4, -1], m: 3 };
        let (xr, xi) = x.to_complex();
        for k in 3..=6u32 {
            close(Cyclo::from_cyc(x, k).to_complex(), (xr, xi), &format!("embed k={k}"));
        }
        // and arithmetic commutes with the embedding
        let y = Cyc { c: [-2, 7, 0, 5], m: 1 };
        for k in 3..=5u32 {
            let lhs = Cyclo::from_cyc(x.mul(y), k);
            let rhs = Cyclo::from_cyc(x, k).mul(&Cyclo::from_cyc(y, k));
            close(lhs.to_complex(), rhs.to_complex(), &format!("mul commutes k={k}"));
        }
    }

    /// The classification table, checked against the actual angles.
    #[test]
    fn classification_matches_the_wild_families() {
        use std::f64::consts::PI;
        assert_eq!(ring_for(PI / 4.0), RingFor::TwoPower(3)); // T
        assert_eq!(ring_for(PI / 8.0), RingFor::TwoPower(4)); // √T, ζ16
        assert_eq!(ring_for(PI / 16.0), RingFor::TwoPower(5)); // ζ32
        assert_eq!(ring_for(0.955_316_618_124_509_2), RingFor::Sqrt3); // face
        assert_eq!(ring_for(2.0 * PI / 3.0), RingFor::Sqrt3); // qutrit ζ3
        assert!(matches!(ring_for(0.3), RingFor::Unimplemented(_)));
        // the named gap: ζ9
        assert!(matches!(ring_for(2.0 * PI / 9.0), RingFor::Unimplemented(_)));
    }

    #[test]
    #[should_panic(expected = "exactness envelope")]
    fn overflow_refuses_like_the_base_ring() {
        let mut big = Cyclo::one(4);
        big.c[0] = i128::MAX;
        let _ = big.add(&big);
    }
}
