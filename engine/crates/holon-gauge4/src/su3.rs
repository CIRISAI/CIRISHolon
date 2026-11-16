//! Complex numbers and 3 by 3 complex matrices, as much as SU(3) needs.
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct C64 {
    pub re: f64,
    pub im: f64,
}

pub const fn c(re: f64, im: f64) -> C64 {
    C64 { re, im }
}

impl C64 {
    pub const ZERO: C64 = c(0.0, 0.0);
    pub const ONE: C64 = c(1.0, 0.0);
    #[inline]
    pub fn conj(self) -> C64 {
        c(self.re, -self.im)
    }
    #[inline]
    pub fn norm2(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
    #[inline]
    pub fn scale(self, k: f64) -> C64 {
        c(self.re * k, self.im * k)
    }
}
impl Add for C64 {
    type Output = C64;
    #[inline]
    fn add(self, o: C64) -> C64 {
        c(self.re + o.re, self.im + o.im)
    }
}
impl Sub for C64 {
    type Output = C64;
    #[inline]
    fn sub(self, o: C64) -> C64 {
        c(self.re - o.re, self.im - o.im)
    }
}
impl Mul for C64 {
    type Output = C64;
    #[inline]
    fn mul(self, o: C64) -> C64 {
        c(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
}
impl Neg for C64 {
    type Output = C64;
    #[inline]
    fn neg(self) -> C64 {
        c(-self.re, -self.im)
    }
}

/// A 3 by 3 complex matrix, row-major. Not necessarily unitary: staples and field strengths
/// are held in the same type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Su3(pub [C64; 9]);

impl Su3 {
    pub const ZERO: Su3 = Su3([C64::ZERO; 9]);
    pub const ONE: Su3 = Su3([C64::ONE, C64::ZERO, C64::ZERO, C64::ZERO, C64::ONE, C64::ZERO, C64::ZERO, C64::ZERO, C64::ONE]);

    #[inline]
    pub fn mul(&self, o: &Su3) -> Su3 {
        let (a, b) = (&self.0, &o.0);
        let mut r = [C64::ZERO; 9];
        for i in 0..3 {
            for j in 0..3 {
                r[3 * i + j] = a[3 * i] * b[j] + a[3 * i + 1] * b[3 + j] + a[3 * i + 2] * b[6 + j];
            }
        }
        Su3(r)
    }
    /// `self * o†`.
    #[inline]
    pub fn mul_dag(&self, o: &Su3) -> Su3 {
        let (a, b) = (&self.0, &o.0);
        let mut r = [C64::ZERO; 9];
        for i in 0..3 {
            for j in 0..3 {
                r[3 * i + j] = a[3 * i] * b[3 * j].conj() + a[3 * i + 1] * b[3 * j + 1].conj() + a[3 * i + 2] * b[3 * j + 2].conj();
            }
        }
        Su3(r)
    }
    /// `self† * o`.
    #[inline]
    pub fn dag_mul(&self, o: &Su3) -> Su3 {
        let (a, b) = (&self.0, &o.0);
        let mut r = [C64::ZERO; 9];
        for i in 0..3 {
            for j in 0..3 {
                r[3 * i + j] = a[i].conj() * b[j] + a[3 + i].conj() * b[3 + j] + a[6 + i].conj() * b[6 + j];
            }
        }
        Su3(r)
    }
    #[inline]
    pub fn dag(&self) -> Su3 {
        let a = &self.0;
        Su3([a[0].conj(), a[3].conj(), a[6].conj(), a[1].conj(), a[4].conj(), a[7].conj(), a[2].conj(), a[5].conj(), a[8].conj()])
    }
    #[inline]
    pub fn add(&self, o: &Su3) -> Su3 {
        let mut r = self.0;
        for (x, y) in r.iter_mut().zip(&o.0) {
            *x = *x + *y;
        }
        Su3(r)
    }
    #[inline]
    pub fn sub(&self, o: &Su3) -> Su3 {
        let mut r = self.0;
        for (x, y) in r.iter_mut().zip(&o.0) {
            *x = *x - *y;
        }
        Su3(r)
    }
    #[inline]
    pub fn scale(&self, k: f64) -> Su3 {
        let mut r = self.0;
        for x in r.iter_mut() {
            *x = x.scale(k);
        }
        Su3(r)
    }
    #[inline]
    pub fn trace(&self) -> C64 {
        self.0[0] + self.0[4] + self.0[8]
    }
    /// The real part of the trace of `self * o`, without forming the product.
    #[inline]
    pub fn re_tr_mul(&self, o: &Su3) -> f64 {
        let (a, b) = (&self.0, &o.0);
        let mut s = 0.0;
        for i in 0..3 {
            for k in 0..3 {
                s += a[3 * i + k].re * b[3 * k + i].re - a[3 * i + k].im * b[3 * k + i].im;
            }
        }
        s
    }
    pub fn det(&self) -> C64 {
        let a = &self.0;
        a[0] * (a[4] * a[8] - a[5] * a[7]) - a[1] * (a[3] * a[8] - a[5] * a[6]) + a[2] * (a[3] * a[7] - a[4] * a[6])
    }
    /// The largest entry of `self† self - 1`, in modulus.
    pub fn unitarity_defect(&self) -> f64 {
        let p = self.dag_mul(self).sub(&Su3::ONE);
        p.0.iter().map(|z| z.norm2().sqrt()).fold(0.0, f64::max)
    }
    /// The largest entry of the difference, in modulus.
    pub fn distance(&self, o: &Su3) -> f64 {
        self.sub(o).0.iter().map(|z| z.norm2().sqrt()).fold(0.0, f64::max)
    }
    /// Back onto SU(3): Gram-Schmidt on the first two rows, the third their conjugated cross
    /// product, which makes the determinant exactly one.
    pub fn reunitarise(&self) -> Su3 {
        let a = &self.0;
        let mut r0 = [a[0], a[1], a[2]];
        let n0 = (r0[0].norm2() + r0[1].norm2() + r0[2].norm2()).sqrt();
        for x in r0.iter_mut() {
            *x = x.scale(1.0 / n0);
        }
        let mut r1 = [a[3], a[4], a[5]];
        let p = r0[0].conj() * r1[0] + r0[1].conj() * r1[1] + r0[2].conj() * r1[2];
        for k in 0..3 {
            r1[k] = r1[k] - p * r0[k];
        }
        let n1 = (r1[0].norm2() + r1[1].norm2() + r1[2].norm2()).sqrt();
        for x in r1.iter_mut() {
            *x = x.scale(1.0 / n1);
        }
        let r2 = [(r0[1] * r1[2] - r0[2] * r1[1]).conj(), (r0[2] * r1[0] - r0[0] * r1[2]).conj(), (r0[0] * r1[1] - r0[1] * r1[0]).conj()];
        Su3([r0[0], r0[1], r0[2], r1[0], r1[1], r1[2], r2[0], r2[1], r2[2]])
    }
    /// The traceless anti-Hermitian part: `(M - M†)/2 - tr(M - M†)/6`.
    #[inline]
    pub fn traceless_antihermitian(&self) -> Su3 {
        let mut r = self.sub(&self.dag()).scale(0.5);
        let t = r.trace().scale(1.0 / 3.0);
        r.0[0] = r.0[0] - t;
        r.0[4] = r.0[4] - t;
        r.0[8] = r.0[8] - t;
        r
    }
    /// `exp(self)` by scaling and squaring a Taylor series (twelve terms after the entries are
    /// brought under 1/16; the remainder is below 1e-19 of the result).
    pub fn exp(&self) -> Su3 {
        let norm = self.0.iter().map(|z| z.norm2()).sum::<f64>().sqrt();
        let mut k = 0;
        let mut s = 1.0;
        while norm * s > 0.0625 {
            s *= 0.5;
            k += 1;
        }
        let x = self.scale(s);
        // Horner: 1 + x (1 + x/2 (1 + x/3 ( ... )))
        let mut r = Su3::ONE;
        for n in (1..=12).rev() {
            r = Su3::ONE.add(&x.mul(&r).scale(1.0 / n as f64));
        }
        for _ in 0..k {
            r = r.mul(&r);
        }
        r
    }
    /// Multiply rows `i` and `j` from the left by the SU(2) matrix of the unit quaternion `q`
    /// (`q0 + i (q1 s1 + q2 s2 + q3 s3)`, the Pauli matrices): the embedding of an SU(2)
    /// subgroup that Cabibbo and Marinari update one at a time.
    #[inline]
    pub fn left_su2(&mut self, i: usize, j: usize, q: [f64; 4]) {
        let (m00, m01, m10, m11) = (c(q[0], q[3]), c(q[2], q[1]), c(-q[2], q[1]), c(q[0], -q[3]));
        for col in 0..3 {
            let (a, b) = (self.0[3 * i + col], self.0[3 * j + col]);
            self.0[3 * i + col] = m00 * a + m01 * b;
            self.0[3 * j + col] = m10 * a + m11 * b;
        }
    }
    /// The quaternion part of the 2 by 2 block on rows and columns `i`, `j`: the block is that
    /// quaternion's matrix plus a part whose trace against every SU(2) matrix is imaginary.
    #[inline]
    pub fn su2_part(&self, i: usize, j: usize) -> [f64; 4] {
        let (a, b, cc, d) = (self.0[3 * i + i], self.0[3 * i + j], self.0[3 * j + i], self.0[3 * j + j]);
        [0.5 * (a.re + d.re), 0.5 * (b.im + cc.im), 0.5 * (b.re - cc.re), 0.5 * (a.im - d.im)]
    }
}

/// The product of two quaternions in the convention of [`Su3::left_su2`].
#[inline]
pub fn quat_mul(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + b[0] * a[1] - (a[2] * b[3] - a[3] * b[2]),
        a[0] * b[2] + b[0] * a[2] - (a[3] * b[1] - a[1] * b[3]),
        a[0] * b[3] + b[0] * a[3] - (a[1] * b[2] - a[2] * b[1]),
    ]
}
