//! The exact reference: the 1D open Hubbard chain at half filling, in the Sz = 0 sector.
//!
//! `Q_SEAM_PREREG.md` §1 pins the family; this module builds it and nothing else. Two structural
//! facts do all the work here and both are exploited deliberately rather than discovered:
//!
//! 1. **The two spin species factorize.** `H = T ⊗ I + I ⊗ T + U · D`, where `T` is the
//!    single-species hopping matrix on `C(N, N/2)` configurations and `D` is diagonal. So the
//!    largest object ever built is `T` at 252 × 252 (N = 10), never the 63 504 × 63 504 matrix.
//!    The matvec touches the full vector but the operator never exists in memory.
//!
//! 2. **Nearest-neighbour hops on a chain carry no Jordan–Wigner sign.** With the orbital
//!    ordering (all up, then all down) a spin-σ hop's JW string counts only same-spin orbitals
//!    strictly between the two sites, and for `|i − j| = 1` there are none. The general signed
//!    path is implemented anyway in [`hop_sign`] and gate G-E3 asserts the two agree — the
//!    simplification is *checked*, not assumed.

/// Maximum chain length this module supports (masks are `u16`).
pub const MAX_SITES: usize = 12;

/// Configurations of one spin species: every `N`-bit mask with exactly `k` bits set, ascending.
///
/// The ascending order is the canonical order referenced by the prereg's Lanczos start-vector
/// pin (A1/P1): the start vector is defined over *this* ordering, so a run is replayable.
#[derive(Clone, Debug)]
pub struct SpinBasis {
    pub sites: usize,
    pub filled: usize,
    pub masks: Vec<u16>,
    /// `index_of[mask]` is the position in `masks`, or `usize::MAX` when the popcount is wrong.
    index_of: Vec<usize>,
}

impl SpinBasis {
    pub fn new(sites: usize, filled: usize) -> Self {
        assert!(sites <= MAX_SITES, "chain longer than the u16 mask allows");
        assert!(filled <= sites);
        let mut masks = Vec::new();
        let mut index_of = vec![usize::MAX; 1usize << sites];
        for m in 0u32..(1u32 << sites) {
            if (m.count_ones() as usize) == filled {
                index_of[m as usize] = masks.len();
                masks.push(m as u16);
            }
        }
        Self { sites, filled, masks, index_of }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.masks.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.masks.is_empty()
    }

    #[inline]
    pub fn index(&self, mask: u16) -> usize {
        self.index_of[mask as usize]
    }
}

/// Fermionic sign of `c†_a c_b` acting on `mask`, by the explicit Jordan–Wigner string.
///
/// Returns `None` when the term annihilates the state. This is the general path; it is never
/// short-circuited for adjacency, which is what makes gate G-E3 a real check.
#[inline]
pub fn hop_sign(mask: u16, a: usize, b: usize) -> Option<(u16, f64)> {
    if mask & (1 << b) == 0 {
        return None;
    }
    let below_b = mask & ((1u16 << b) - 1);
    let s1 = if below_b.count_ones() % 2 == 0 { 1.0 } else { -1.0 };
    let m1 = mask ^ (1 << b);
    if m1 & (1 << a) != 0 {
        return None;
    }
    let below_a = m1 & ((1u16 << a) - 1);
    let s2 = if below_a.count_ones() % 2 == 0 { 1.0 } else { -1.0 };
    Some((m1 | (1 << a), s1 * s2))
}

/// Single-species hopping matrix in compressed sparse row form.
#[derive(Clone, Debug)]
pub struct SparseSym {
    pub dim: usize,
    pub row_start: Vec<usize>,
    pub col: Vec<usize>,
    pub val: Vec<f64>,
}

impl SparseSym {
    /// `-t Σ_i (c†_i c_{i+1} + h.c.)` on one spin species, open chain.
    pub fn hopping(basis: &SpinBasis, t: f64) -> Self {
        let dim = basis.len();
        let mut row_start = Vec::with_capacity(dim + 1);
        let mut col = Vec::new();
        let mut val = Vec::new();
        row_start.push(0);
        for &mask in &basis.masks {
            for i in 0..basis.sites.saturating_sub(1) {
                // Both orientations of the bond; each is generated once from the state it acts on.
                for (a, b) in [(i, i + 1), (i + 1, i)] {
                    if let Some((new_mask, sign)) = hop_sign(mask, a, b) {
                        col.push(basis.index(new_mask));
                        val.push(-t * sign);
                    }
                }
            }
            row_start.push(col.len());
        }
        Self { dim, row_start, col, val }
    }

    /// `max |M − Mᵀ|`, computed exactly. Gate G-E1 requires this to be bitwise zero.
    pub fn asymmetry(&self) -> f64 {
        let mut worst = 0.0f64;
        for r in 0..self.dim {
            for k in self.row_start[r]..self.row_start[r + 1] {
                let c = self.col[k];
                let mut mirrored = 0.0;
                for k2 in self.row_start[c]..self.row_start[c + 1] {
                    if self.col[k2] == r {
                        mirrored = self.val[k2];
                        break;
                    }
                }
                let d = (self.val[k] - mirrored).abs();
                if d > worst {
                    worst = d;
                }
            }
        }
        worst
    }

    pub fn to_dense(&self) -> Vec<f64> {
        let mut m = vec![0.0; self.dim * self.dim];
        for r in 0..self.dim {
            for k in self.row_start[r]..self.row_start[r + 1] {
                m[r * self.dim + self.col[k]] += self.val[k];
            }
        }
        m
    }
}

/// The half-filled Sz = 0 Hubbard chain: everything needed to apply `H` without forming it.
#[derive(Clone, Debug)]
pub struct Hubbard {
    pub sites: usize,
    pub t: f64,
    pub u: f64,
    /// Site potential `v_i` (Q7). All-zero reproduces the Q5 family exactly.
    pub potential: Vec<f64>,
    /// `Σ_i v_i · bit_i(mask)` per single-species configuration — the potential is diagonal and
    /// separable over the two spins, so it costs one `n_conf` vector, never an `n_conf²` one.
    pot_conf: Vec<f64>,
    pub basis: SpinBasis,
    pub hop: SparseSym,
    /// `double_occ[iu * n + id]` = number of doubly occupied sites. Integer-valued by
    /// construction, so the interaction is exact in floating point.
    pub double_occ: Vec<u8>,
}

impl Hubbard {
    /// The Q5 family: no site potential.
    pub fn new(sites: usize, t: f64, u: f64) -> Self {
        Self::with_potential(sites, t, u, &vec![0.0; sites])
    }

    /// Q7's harmonic trap, `Q7_SEAM_PREREG.md` §3: `v_i = a·((i − c)/(N/2))²`, `c = (N+1)/2`,
    /// so `v` runs from 0 at the centre to ≈ a at the edges. Reflection-symmetric by construction,
    /// which is what §2.3's consistency gate reads.
    pub fn trap(sites: usize, a: f64) -> Vec<f64> {
        let c = (sites as f64 + 1.0) / 2.0;
        (1..=sites)
            .map(|i| {
                let x = (i as f64 - c) / (sites as f64 / 2.0);
                a * x * x
            })
            .collect()
    }

    pub fn with_potential(sites: usize, t: f64, u: f64, potential: &[f64]) -> Self {
        assert!(sites % 2 == 0, "half filling needs an even chain");
        assert_eq!(potential.len(), sites);
        let basis = SpinBasis::new(sites, sites / 2);
        let hop = SparseSym::hopping(&basis, t);
        let n = basis.len();
        let pot_conf: Vec<f64> = basis
            .masks
            .iter()
            .map(|&m| (0..sites).filter(|&i| m & (1 << i) != 0).map(|i| potential[i]).sum())
            .collect();
        let mut double_occ = vec![0u8; n * n];
        for (iu, &mu) in basis.masks.iter().enumerate() {
            for (id, &md) in basis.masks.iter().enumerate() {
                double_occ[iu * n + id] = (mu & md).count_ones() as u8;
            }
        }
        Self { sites, t, u, potential: potential.to_vec(), pot_conf, basis, hop, double_occ }
    }

    #[inline]
    pub fn n_conf(&self) -> usize {
        self.basis.len()
    }

    #[inline]
    pub fn dim(&self) -> usize {
        self.n_conf() * self.n_conf()
    }

    /// `y = H x`. The only place the Hamiltonian acts.
    pub fn apply(&self, x: &[f64], y: &mut [f64]) {
        let n = self.n_conf();
        debug_assert_eq!(x.len(), n * n);

        for (idx, slot) in y.iter_mut().enumerate() {
            let (iu, id) = (idx / n, idx % n);
            *slot = (self.u * f64::from(self.double_occ[idx]) + self.pot_conf[iu] + self.pot_conf[id])
                * x[idx];
        }

        // Up-spin hopping: acts on the row index.
        for iu in 0..n {
            for k in self.hop.row_start[iu]..self.hop.row_start[iu + 1] {
                let ju = self.hop.col[k];
                let v = self.hop.val[k];
                let (dst, src) = (iu * n, ju * n);
                for id in 0..n {
                    y[dst + id] += v * x[src + id];
                }
            }
        }

        // Down-spin hopping: acts on the column index.
        for iu in 0..n {
            let base = iu * n;
            for id in 0..n {
                for k in self.hop.row_start[id]..self.hop.row_start[id + 1] {
                    y[base + id] += self.hop.val[k] * x[base + self.hop.col[k]];
                }
            }
        }
    }

    /// Dense `H`, for the independent cross-check at N ≤ 6 (gate G-E4b). Never used at N ≥ 8.
    pub fn to_dense(&self) -> Vec<f64> {
        let d = self.dim();
        let mut h = vec![0.0; d * d];
        let mut x = vec![0.0; d];
        let mut y = vec![0.0; d];
        for j in 0..d {
            x[j] = 1.0;
            self.apply(&x, &mut y);
            for (i, &yi) in y.iter().enumerate() {
                h[i * d + j] = yi;
            }
            x[j] = 0.0;
        }
        h
    }
}

/// `E(U=0)/N/t` and the HOMO–LUMO gap from the prereg's §1.1(i) closed forms.
///
/// These are the analytic values the instrument must reproduce (gate G-E7) — the ruler is
/// gauged before it is used, and at N = 10 this is the *only* validation that reaches
/// dimension 63 504 (amendment A1/F).
pub fn free_chain_energy_per_site(sites: usize) -> f64 {
    let theta = core::f64::consts::PI / (sites as f64 + 1.0);
    let s: f64 = (1..=sites / 2).map(|k| (k as f64 * theta).cos()).sum();
    -4.0 * s / sites as f64
}

pub fn free_chain_gap(sites: usize) -> f64 {
    4.0 * (core::f64::consts::PI / (2.0 * (sites as f64 + 1.0))).sin()
}

/// The Hubbard dimer's exact ground energy, §1.1(ii). Gauges the whole N = 2 column at every U.
pub fn dimer_energy(t: f64, u: f64) -> f64 {
    (u - (u * u + 16.0 * t * t).sqrt()) / 2.0
}

/// The dimer's exact total double occupancy, by Hellmann–Feynman on [`dimer_energy`].
pub fn dimer_double_occupancy(t: f64, u: f64) -> f64 {
    (1.0 - u / (u * u + 16.0 * t * t).sqrt()) / 2.0
}

/// **THE Q7 RULER** (`Q7_SEAM_PREREG.md` §5, gate G7-E7): the exact `U = 0` solution at any site
/// potential.
///
/// Q5 gauged its instrument on a closed form; a trap has none, so the ruler is the exact
/// diagonalization of the `N × N` single-particle Hamiltonian `h_ij = −t·δ_{i,j±1} + v_i·δ_ij`.
/// Filling the lowest `N/2` levels per spin gives the energy, the full density profile and every
/// 1-RDM element **exactly** — not asymptotically. The many-body Lanczos and the chart must both
/// reproduce it to `1e-12`.
pub struct FreeReference {
    pub energy: f64,
    pub density: Vec<f64>,
    /// One spin block of the 1-RDM, row-major `N × N`.
    pub rdm: Vec<f64>,
    pub levels: Vec<f64>,
    /// HOMO–LUMO gap of the single-particle spectrum.
    pub gap: f64,
}

pub fn free_reference(sites: usize, t: f64, potential: &[f64]) -> FreeReference {
    let mut h = vec![0.0; sites * sites];
    for i in 0..sites {
        h[i * sites + i] = potential[i];
        if i + 1 < sites {
            h[i * sites + i + 1] = -t;
            h[(i + 1) * sites + i] = -t;
        }
    }
    let eig = crate::dense::jacobi(h, sites);
    let n_occ = sites / 2;

    let mut rdm = vec![0.0; sites * sites];
    for p in 0..n_occ {
        let v = &eig.vectors[p];
        for i in 0..sites {
            for j in 0..sites {
                rdm[i * sites + j] += v[i] * v[j];
            }
        }
    }
    // Two spin species, each filling the same lowest N/2 levels.
    let energy = 2.0 * eig.values[..n_occ].iter().sum::<f64>();
    let density = (0..sites).map(|i| 2.0 * rdm[i * sites + i]).collect();
    let gap = eig.values[n_occ] - eig.values[n_occ - 1];

    FreeReference { energy, density, rdm, levels: eig.values, gap }
}
