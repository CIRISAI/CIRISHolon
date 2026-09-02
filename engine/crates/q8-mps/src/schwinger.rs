//! The massless Schwinger model (QED₂) in the Hamer–Kogut spin form, with STATIC unit
//! charges entering as the Gauss law's background flux — the SCHWINGER-4 Hamiltonian
//! (conformance/crystal/SCHWINGER4_PREREG.md, node GF0) on the engine's own DMRG.
//!
//! ```text
//! W = x Σ_n (σ⁺_n σ⁻_{n+1} + h.c.) + Σ_{n<N−1} (L_n + ε_n)² + λ Q²
//! L_n = Σ_{k≤n} q_k,   q_k = ½(σᶻ_k + (−1)^k),   ε_n = Σ_{k≤n} Q^ext_k,   Q = Σ_k q_k
//! ```
//!
//! `x = 1/(g a)²`; `λ = 20(x + 1)` is the total-charge penalty the mirrored Python driver
//! carries (the MPS roams charge sectors and the global minimum lives outside `Q = 0`,
//! measured there). Expanding the square, the static charges add ONE site-diagonal term
//! `c_k q_k` with `c_k = 2 Σ_{m=k}^{N−2} ε_m`, plus the constant `Σ ε_n²`.
//!
//! THIS MODULE IS THE SAME TENSOR as the Python instrument
//! (`conformance/crystal/instrument/dmrg_schwinger.py`, SCHWINGER-3, bytes pinned, and its
//! `schwinger4.py` extension): six channels, identical entries, identical penalty, so the two
//! drivers can be compared at the χ-premise band and the exact referee (SCHWINGER-1's ED) grades
//! both. Basis convention, stated: physical index `s = 0 ↔ σᶻ = +1` (the Python driver's index 0,
//! `np.diag([1, −1])`), `s = 1 ↔ σᶻ = −1`. The engine's `MpoSite::get(cl, cr, s, sp)` reads
//! `s` as the KET index and `sp` as the BRA index (see `mps::grow_left_mpo`), so each 2×2 block is
//! the transpose of the Python `w[cl, cr, bra, ket]` block.
//!
//! Credits: Banks–Kogut–Susskind and Hamer et al. for the spin formulation; White 1992 for
//! DMRG; Bañuls–Cichy–Jansen–Cirac for the MPS-Schwinger tradition; Schwinger 1962 for the
//! continuum referee.

use crate::dmrg::{dmrg_sweep, DmrgConfig, DmrgResult, Refusal, RefusalPolicy};
use crate::mpo::{Mpo, MpoSite};
use crate::mps::TensorSite;

/// The interior bond dimension: start, after σ⁺, after σ⁻, Coulomb accumulator, charge
/// accumulator, done — the Python driver's channel order.
pub const CHANNELS: usize = 6;

/// Planted defects for the campaign's plants, never a physics option.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mutation {
    None,
    /// Plant (ii): the dynamical Coulomb channel and its diagonal are zeroed, leaving free
    /// staggered fermions in the static site potential.
    CoulombOff,
}

/// One Schwinger chain with its static charges.
#[derive(Clone, Debug)]
pub struct Schwinger {
    pub n: usize,
    pub x: f64,
    pub lam: f64,
    /// `(site, charge)`; charges are ±1 (any integer is accepted).
    pub charges: Vec<(usize, i32)>,
    pub mutation: Mutation,
}

type Block = [[f64; 2]; 2];

const I2: Block = [[1.0, 0.0], [0.0, 1.0]];
/// σ⁺ in the Python convention: `⟨0|σ⁺|1⟩ = 1` (raises σᶻ = −1 to +1).
const SP: Block = [[0.0, 1.0], [0.0, 0.0]];
const SM: Block = [[0.0, 0.0], [1.0, 0.0]];

fn scale(b: Block, c: f64) -> Block {
    [[b[0][0] * c, b[0][1] * c], [b[1][0] * c, b[1][1] * c]]
}

fn add(a: Block, b: Block) -> Block {
    [[a[0][0] + b[0][0], a[0][1] + b[0][1]], [a[1][0] + b[1][0], a[1][1] + b[1][1]]]
}

fn mul(a: Block, b: Block) -> Block {
    [
        [a[0][0] * b[0][0] + a[0][1] * b[1][0], a[0][0] * b[0][1] + a[0][1] * b[1][1]],
        [a[1][0] * b[0][0] + a[1][1] * b[1][0], a[1][0] * b[0][1] + a[1][1] * b[1][1]],
    ]
}

impl Schwinger {
    pub fn new(n: usize, x: f64, charges: Vec<(usize, i32)>) -> Self {
        assert!(n >= 4 && n % 2 == 0, "an even chain of at least four sites, got {n}");
        for &(s, _) in &charges {
            assert!(s < n, "static charge at site {s} outside the chain of {n}");
        }
        Self { n, x, lam: 20.0 * (x + 1.0), charges, mutation: Mutation::None }
    }

    pub fn with_mutation(mut self, m: Mutation) -> Self {
        self.mutation = m;
        self
    }

    /// The site charge operator `q_l = ½(σᶻ + (−1)^l)` in the stated basis.
    pub fn site_charge(&self, l: usize) -> Block {
        let stag = if l % 2 == 0 { 1.0 } else { -1.0 };
        [[0.5 * (1.0 + stag), 0.0], [0.0, 0.5 * (-1.0 + stag)]]
    }

    /// `ε_k = Σ_{j≤k} Q^ext_j` on link `k` (the last entry is never read).
    pub fn background_flux(&self) -> Vec<f64> {
        (0..self.n)
            .map(|k| self.charges.iter().filter(|&&(s, _)| s <= k).map(|&(_, q)| q as f64).sum())
            .collect()
    }

    /// `c_k = 2 Σ_{m=k}^{N−2} ε_m`: the one diagonal entry the static charges add.
    pub fn site_potential(&self) -> Vec<f64> {
        let eps = self.background_flux();
        let mut c = vec![0.0; self.n];
        let mut tail = 0.0;
        for k in (0..self.n).rev() {
            if k + 2 <= self.n {
                tail += eps[k];
            }
            c[k] = 2.0 * tail;
        }
        c
    }

    /// `Σ_{n<N−1} ε_n²`, so that `ground_energy` reports the true `W`.
    pub fn constant(&self) -> f64 {
        let eps = self.background_flux();
        eps[..self.n - 1].iter().map(|e| e * e).sum()
    }

    /// The Python driver's `W[l]` as `[channel_in][channel_out] -> 2×2 block` in the
    /// `[bra][ket]` orientation; transposed into the engine's `(ket, bra)` when written.
    fn w_full(&self, l: usize) -> [[Block; CHANNELS]; CHANNELS] {
        let a = (self.n - 1 - l) as f64;
        let q = self.site_charge(l);
        let qq = mul(q, q);
        let c = self.site_potential()[l];
        let z = [[0.0; 2]; 2];
        let mut w = [[z; CHANNELS]; CHANNELS];
        w[0][0] = I2;
        w[0][1] = SP;
        w[1][5] = scale(SM, self.x);
        w[0][2] = SM;
        w[2][5] = scale(SP, self.x);
        let mut diag = scale(qq, self.lam);
        if self.mutation != Mutation::CoulombOff {
            w[0][3] = q;
            w[3][3] = I2;
            w[3][5] = scale(q, 2.0 * a);
            diag = add(diag, scale(qq, a));
        }
        w[0][4] = q;
        w[4][4] = I2;
        w[4][5] = scale(q, 2.0 * self.lam);
        w[0][5] = add(diag, scale(q, c));
        w[5][5] = I2;
        w
    }

    /// The MPO: first site `1×6` (the start row), interior `6×6`, last site `6×1` (the done
    /// column), so the engine's trivial boundary environments — which select channel 0 on both
    /// ends — select START on the left and DONE on the right.
    pub fn mpo(&self) -> Mpo {
        let n = self.n;
        let mut sites = Vec::with_capacity(n);
        for l in 0..n {
            let w = self.w_full(l);
            let (d_l, d_r) = (
                if l == 0 { 1 } else { CHANNELS },
                if l == n - 1 { 1 } else { CHANNELS },
            );
            let mut site = MpoSite::zeros(d_l, d_r);
            for cl in 0..d_l {
                let from = if l == 0 { 0 } else { cl };
                for cr in 0..d_r {
                    let to = if l == n - 1 { CHANNELS - 1 } else { cr };
                    let b = w[from][to];
                    for bra in 0..2 {
                        for ket in 0..2 {
                            if b[bra][ket] != 0.0 {
                                site.set(cl, cr, ket, bra, b[bra][ket]);
                            }
                        }
                    }
                }
            }
            sites.push(site);
        }
        Mpo { sites }
    }

    /// The strong-coupling vacuum as a `χ = 1` product state: every site charge zero, i.e.
    /// even sites `σᶻ = −1` (`s = 1`) and odd sites `σᶻ = +1` (`s = 0`) — the Dirac sea, and
    /// the fixed initial state the engine's sweep starts from (no RNG anywhere).
    pub fn vacuum(&self) -> Vec<TensorSite> {
        (0..self.n)
            .map(|l| {
                let mut t = TensorSite::zeros(1, 1);
                t.set(if l % 2 == 0 { 1 } else { 0 }, 0, 0, 1.0);
                t
            })
            .collect()
    }

    /// The ground-state energy `W` (with the static constant added back) and the sweep record.
    pub fn ground_energy(&self, chi: usize, max_sweeps: usize, sweep_tol: f64) -> Result<(f64, DmrgResult), Refusal> {
        let config = DmrgConfig { chi_max: chi, max_sweeps, sweep_tol, policy: RefusalPolicy::Silent };
        let res = dmrg_sweep(&self.mpo(), self.vacuum(), &config)?;
        Ok((res.energy + self.constant(), res))
    }
}
