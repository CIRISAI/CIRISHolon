//! Massless one-flavour QCD in 1+1 dimensions, axial gauge, as ELECTRONIC-STRUCTURE
//! INTEGRALS — the GF2a instrument (`conformance/crystal/GF2A_QCD2_PREREG.md`).
//!
//! In one space dimension with open boundaries Gauss's law eliminates the SU(3) gauge
//! field exactly; the surviving Hamiltonian is fermions with a non-local colour-Coulomb
//! term, and with the SU(3) Fierz identity that term is a closed-form two-body tensor over
//! (site × colour) orbitals. So the object is the one `fci::solve_determinant` and
//! `q8-mps::Mpo::from_electronic_integrals` already take — the fold below the atom applied
//! to the solver itself: Z prices, Z never branches, and neither does the gauge group.
//!
//! ```text
//! W = x Σ_n Σ_c (ψ†_{n,c} ψ_{n+1,c} + h.c.) + Σ_{n<N−1} Σ_a (Σ_{k≤n} q^a_k)²
//! ```
//!
//! reduces, in the solver's convention `H = Σ h_pq E_pq + ½ Σ (pq|rs)(E_pq E_rs − δ_qr E_ps)`, to
//!
//! * `(pq|rs) = 2 w_{kk'} F_{cc'dd'}` for `p=(k,c) q=(k,c') r=(k',d) s=(k',d')`, with
//!   `w_{kk'} = N−1−max(k,k')` and `F_{cc'dd'} = ½ δ_{cd'} δ_{c'd} − ⅙ δ_{cc'} δ_{dd'}`;
//! * `h_{(k,c),(k,c)} = (4/3)(N−1−k)` — the Casimir of one quark per link to its right;
//! * `h_{(k,c),(k+1,c)} = h_{(k+1,c),(k,c)} = x`.
//!
//! Orbital `p = 3k + c`. One string carries every quark (no spin, one flavour); the Dirac
//! sea is `n_q = 3N/2`, baryon number `B = (n_q − 3N/2)/3`.
//!
//! Credits: Banks–Kogut–Susskind, Hamer et al. (staggered form); Atas et al. 2023 and
//! Farrell et al. 2023 (the axial-gauge 1+1D QCD Hamiltonian for quantum simulation);
//! 't Hooft 1974 (context only).

use crate::dual::D2;
use crate::fci::{solve_determinant, FciSpace, MoIntegrals, Solution};

/// Planted defects for the freeze's plants, never a physics option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mutation {
    None,
    /// Plant (ii): the Fierz trace subtraction `−⅙ δ δ` is dropped.
    FierzTraceOff,
}

pub const COLOURS: usize = 3;
/// The quark Casimir `C_F = (N_c² − 1) / (2 N_c)` for SU(3).
pub const C_F: f64 = 4.0 / 3.0;

/// `Σ_a T^a_{cc'} T^a_{dd'}` for SU(3), by the Fierz identity.
pub fn fierz(c: usize, cp: usize, d: usize, dp: usize, mutation: Mutation) -> f64 {
    let exchange = if c == dp && cp == d { 0.5 } else { 0.0 };
    let trace = if c == cp && d == dp { 1.0 / 6.0 } else { 0.0 };
    match mutation {
        Mutation::None => exchange - trace,
        Mutation::FierzTraceOff => exchange,
    }
}

/// The chain.
#[derive(Clone, Debug)]
pub struct Qcd2 {
    pub n: usize,
    pub x: f64,
    pub mutation: Mutation,
}

impl Qcd2 {
    pub fn new(n: usize, x: f64) -> Self {
        assert!(n >= 2 && n % 2 == 0, "an even chain of at least two sites, got {n}");
        Self { n, x, mutation: Mutation::None }
    }

    pub fn with_mutation(mut self, m: Mutation) -> Self {
        self.mutation = m;
        self
    }

    pub fn n_orb(&self) -> usize {
        COLOURS * self.n
    }

    /// The quark count of baryon-number sector `b` (the sea plus three per baryon).
    pub fn quarks(&self, b: i32) -> usize {
        let sea = COLOURS * self.n / 2;
        (sea as i64 + 3 * i64::from(b)) as usize
    }

    /// `(h, g)` as plain reals: `h` row-major `n_orb²`, `g` chemist `(pq|rs)` at
    /// `[(p·n+q)·n² + (r·n+s)]`.
    pub fn integrals_f64(&self) -> (Vec<f64>, Vec<f64>) {
        let n = self.n;
        let m = self.n_orb();
        let mut h = vec![0.0; m * m];
        let mut g = vec![0.0; m * m * m * m];
        for k in 0..n {
            for c in 0..COLOURS {
                let p = COLOURS * k + c;
                if k + 1 < n {
                    let q = COLOURS * (k + 1) + c;
                    h[p * m + q] += self.x;
                    h[q * m + p] += self.x;
                }
                // the one-body Casimir per link the quark sits to the left of
                if k + 2 <= n {
                    let w = (n - 1 - k) as f64;
                    let self_term: f64 = (0..COLOURS).map(|cp| fierz(c, cp, cp, c, self.mutation)).sum();
                    h[p * m + p] += w * self_term;
                }
            }
        }
        for k in 0..n {
            for kp in 0..n {
                let w = (n - 1 - k.max(kp)) as f64;
                if w == 0.0 {
                    continue;
                }
                for c in 0..COLOURS {
                    for cp in 0..COLOURS {
                        for d in 0..COLOURS {
                            for dp in 0..COLOURS {
                                let f = fierz(c, cp, d, dp, self.mutation);
                                if f == 0.0 {
                                    continue;
                                }
                                let (p, q, r, s) = (COLOURS * k + c, COLOURS * k + cp, COLOURS * kp + d, COLOURS * kp + dp);
                                g[(p * m + q) * m * m + (r * m + s)] += 2.0 * w * f;
                            }
                        }
                    }
                }
            }
        }
        (h, g)
    }

    /// The same integrals as the solver's dual-number `MoIntegrals` (constants: no
    /// geometric derivative is asked for).
    pub fn integrals(&self) -> MoIntegrals {
        let (h, g) = self.integrals_f64();
        MoIntegrals {
            n: self.n_orb(),
            h: h.into_iter().map(D2::c).collect(),
            g: g.into_iter().map(D2::c).collect(),
        }
    }

    /// The determinant space of baryon-number sector `b`.
    pub fn space(&self, b: i32) -> FciSpace {
        FciSpace::new(self.n_orb(), self.quarks(b), 0)
    }

    /// The exact ground state of sector `b` on the determinant route.
    pub fn ground(&self, b: i32) -> Solution {
        solve_determinant(&self.space(b), &self.integrals())
    }

    /// The baryon mass in units of `g`, the Schwinger normalisation: `(E₀(1) − E₀(0)) / (2√x)`.
    pub fn baryon_mass(e0: f64, e1: f64, x: f64) -> f64 {
        (e1 - e0) / (2.0 * x.sqrt())
    }
}
