//! THE HEITLER–LONDON STATE (FIELD-5, `conformance/water_observatory/FIELD5_PREREG.md`):
//! exchange in the embedding.
//!
//! EMBED-2's Coulomb-only frozen-density embedding has no exchange between its fragments,
//! and FIELD-4 measured what that costs at contact: the fragments polarise into each other
//! unopposed and the "field" over-binds by 27 mHa at 2.5 Å. This module puts the Pauli
//! term where it belongs — in the referee. The antisymmetrised product of the two monomers'
//! EXACT wavefunctions is built on the seam programme's own determinant solver:
//!
//! 1. each monomer's full CI in its own basis (`solve_embedded`): its energy `E_X0`, its
//!    orbitals `C_X` and its CI vector;
//! 2. the dimer's AO basis, the concatenation the supermolecule uses; the block-diagonal
//!    orbital matrix `C = diag(C_A, C_B)` — orthonormal within each block, NOT across them —
//!    symmetrically orthogonalised across the fragments, `C' = C (CᵀSC)^{−1/2}` (Löwdin
//!    1950), so the dimer's determinants over `C'` are orthonormal;
//! 3. the dimer's molecular integrals over `C'` and its determinant space over
//!    `n_A + n_B` orbitals — the SAME space the exact dimer is solved in, in a different
//!    orthonormal basis (a full CI is invariant under that change: gate G-H0);
//! 4. the Heitler–London vector: on every dimer determinant whose α-string and β-string
//!    each put exactly the monomers' electron counts on the monomers' own orbitals, the
//!    product of the two monomer CI coefficients; zero elsewhere (charge transfer is
//!    excluded by construction). With orbital-ordered strings and all α before all β the
//!    determinant phases are a constant sign across the product, which an expectation
//!    value does not see;
//! 5. `E_HL = ⟨Ψ|H|Ψ⟩/⟨Ψ|Ψ⟩ + E_nuc` by ONE Hamiltonian application (`sigma`);
//! 6. `E_es`, the classical interaction of the two ISOLATED monomer densities (EMBED-2's
//!    `classical_interaction`), and
//!
//! ```text
//!     E_exch = E_HL − E_A0 − E_B0 − E_es
//! ```
//!
//! the first-order exchange in the orthogonalised-orbital convention (it differs from
//! SAPT's `E^{(10)}_exch` at order `S⁴`; stated, not corrected). It is the referee a wall is
//! harvested from: with the Pauli term in it, it cannot collapse.
//!
//! Prior art: Heitler and London 1927; Löwdin 1950; Jeziorski, Moszynski and Szalewicz
//! 1994 for the names of the pieces. No number here comes from outside this crate.

use crate::density_embed::classical_interaction;
use crate::dual::D2;
use crate::embed::{ao_density, build_basis_embedded, rdm1, solve_embedded, EmbeddedSolve, Fragment};
use crate::fci::{ci_ints, solve_determinant, transform, FciSpace, MoIntegrals, Order, Solution};
use crate::md::ao_integrals;
use crate::pair::electron_counts;
use std::time::Instant;

/// The freeze's plant (ii): the block-diagonal orbitals used as if they were orthonormal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HlPlant {
    #[default]
    None,
    SkipOrthogonalisation,
}

/// One Heitler–London reading.
#[derive(Clone, Debug)]
pub struct HlReading {
    /// `⟨Ψ_HL|H|Ψ_HL⟩ + E_nuc`.
    pub e_hl: f64,
    pub e_a0: f64,
    pub e_b0: f64,
    /// The classical interaction of the isolated monomer densities.
    pub e_es: f64,
    /// `e_hl − e_a0 − e_b0 − e_es`.
    pub e_exch: f64,
    /// The product vector's norm before normalisation (1 when the monomer vectors are unit).
    pub norm: f64,
    /// Determinants the product vector is nonzero on.
    pub nonzero_dets: usize,
    pub n_det: usize,
    pub n_det_a: usize,
    pub n_det_b: usize,
    /// The largest cross-fragment element of `CᵀSC` — the overlap the orthogonalisation removes.
    pub s_cross_max: f64,
    /// Wall seconds of the one `sigma`.
    pub sigma_seconds: f64,
}

/// Cyclic Jacobi eigendecomposition of a symmetric `n × n` matrix (row-major). Returns the
/// eigenvalues and the eigenvectors as columns of a row-major matrix (`v[i*n+k]` is
/// component `i` of eigenvector `k`). Small matrices only (the orbital count).
fn jacobi_eigen(a: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut m = a.to_vec();
    let mut v = vec![0.0f64; n * n];
    for i in 0..n {
        v[i * n + i] = 1.0;
    }
    for _sweep in 0..100 {
        let mut off = 0.0f64;
        for p in 0..n {
            for q in (p + 1)..n {
                off += m[p * n + q] * m[p * n + q];
            }
        }
        if off < 1e-30 {
            break;
        }
        for p in 0..n {
            for q in (p + 1)..n {
                let apq = m[p * n + q];
                if apq.abs() < 1e-300 {
                    continue;
                }
                let app = m[p * n + p];
                let aqq = m[q * n + q];
                let theta = (aqq - app) / (2.0 * apq);
                let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
                let t = if theta == 0.0 { 1.0 } else { t };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for k in 0..n {
                    let mkp = m[k * n + p];
                    let mkq = m[k * n + q];
                    m[k * n + p] = c * mkp - s * mkq;
                    m[k * n + q] = s * mkp + c * mkq;
                }
                for k in 0..n {
                    let mpk = m[p * n + k];
                    let mqk = m[q * n + k];
                    m[p * n + k] = c * mpk - s * mqk;
                    m[q * n + k] = s * mpk + c * mqk;
                }
                for k in 0..n {
                    let vkp = v[k * n + p];
                    let vkq = v[k * n + q];
                    v[k * n + p] = c * vkp - s * vkq;
                    v[k * n + q] = s * vkp + c * vkq;
                }
            }
        }
    }
    let w: Vec<f64> = (0..n).map(|i| m[i * n + i]).collect();
    (w, v)
}

/// `M^{−1/2}` for a symmetric positive-definite `M` (row-major `n × n`).
fn inverse_sqrt(m: &[f64], n: usize) -> Vec<f64> {
    let (w, v) = jacobi_eigen(m, n);
    let mut out = vec![0.0f64; n * n];
    for i in 0..n {
        for j in 0..n {
            let mut acc = 0.0;
            for k in 0..n {
                assert!(w[k] > 0.0, "the fragment overlap is not positive definite (eigenvalue {})", w[k]);
                acc += v[i * n + k] * v[j * n + k] / w[k].sqrt();
            }
            out[i * n + j] = acc;
        }
    }
    out
}

/// `M^{1/2}` for a symmetric positive-definite `M` (row-major `n × n`).
fn sqrt_sym(m: &[f64], n: usize) -> Vec<f64> {
    let (w, v) = jacobi_eigen(m, n);
    let mut out = vec![0.0f64; n * n];
    for i in 0..n {
        for j in 0..n {
            let mut acc = 0.0;
            for k in 0..n {
                assert!(w[k] > 0.0, "the fragment overlap is not positive definite (eigenvalue {})", w[k]);
                acc += v[i * n + k] * v[j * n + k] * w[k].sqrt();
            }
            out[i * n + j] = acc;
        }
    }
    out
}

/// The determinant of a small dense matrix (row-major `k × k`) by LU with partial pivoting.
fn det_small(a: &mut [f64], k: usize) -> f64 {
    let mut det = 1.0f64;
    for col in 0..k {
        let mut piv = col;
        for r in (col + 1)..k {
            if a[r * k + col].abs() > a[piv * k + col].abs() {
                piv = r;
            }
        }
        if a[piv * k + col] == 0.0 {
            return 0.0;
        }
        if piv != col {
            for c in 0..k {
                a.swap(piv * k + c, col * k + c);
            }
            det = -det;
        }
        let d = a[col * k + col];
        det *= d;
        for r in (col + 1)..k {
            let f = a[r * k + col] / d;
            if f != 0.0 {
                for c in col..k {
                    a[r * k + c] -= f * a[col * k + c];
                }
            }
        }
    }
    det
}

/// The shared setup: the monomers solved, the dimer basis and integrals over the
/// (orthogonalised) block-diagonal orbitals, the dimer space.
struct Setup {
    sa: EmbeddedSolve,
    sb: EmbeddedSolve,
    species: Vec<crate::elements::Species>,
    n: usize,
    n_a: usize,
    mo: MoIntegrals,
    space: FciSpace,
    e_nuc: f64,
    s_cross_max: f64,
    /// `CᵀSC` for the block-diagonal orbitals — the fragment overlap the undeformed state needs.
    m: Vec<f64>,
}

fn setup(a: &Fragment, b: &Fragment, plant: HlPlant) -> Setup {
    let sa = solve_embedded(&a.species, &a.centers, &[]);
    let sb = solve_embedded(&b.species, &b.centers, &[]);
    let (n_a, n_b) = (sa.basis.n, sb.basis.n);
    let mut species = a.species.clone();
    species.extend_from_slice(&b.species);
    let mut centers = a.centers.clone();
    centers.extend_from_slice(&b.centers);
    let basis = build_basis_embedded(&species, &centers, &[]);
    let n = basis.n;
    assert_eq!(n, n_a + n_b, "the dimer basis is the concatenation of the monomers'");
    let ao = ao_integrals(&basis);
    // C = diag(C_A, C_B), AO-major as the engine stores orbitals (`c[i*n+p]`)
    let mut c = vec![0.0f64; n * n];
    for i in 0..n_a {
        for p in 0..n_a {
            c[i * n + p] = sa.gp.orbitals[i * n_a + p];
        }
    }
    for i in 0..n_b {
        for p in 0..n_b {
            c[(n_a + i) * n + (n_a + p)] = sb.gp.orbitals[i * n_b + p];
        }
    }
    // M = Cᵀ S C
    let s: Vec<f64> = ao.s.iter().map(|d| d.v).collect();
    let mut sc = vec![0.0f64; n * n];
    for i in 0..n {
        for q in 0..n {
            let mut acc = 0.0;
            for j in 0..n {
                acc += s[i * n + j] * c[j * n + q];
            }
            sc[i * n + q] = acc;
        }
    }
    let mut m = vec![0.0f64; n * n];
    for p in 0..n {
        for q in 0..n {
            let mut acc = 0.0;
            for i in 0..n {
                acc += c[i * n + p] * sc[i * n + q];
            }
            m[p * n + q] = acc;
        }
    }
    let mut s_cross_max = 0.0f64;
    for p in 0..n_a {
        for q in n_a..n {
            s_cross_max = s_cross_max.max(m[p * n + q].abs());
        }
    }
    let m_kept = m.clone();
    let c_prime = match plant {
        HlPlant::SkipOrthogonalisation => c,
        HlPlant::None => {
            let x = inverse_sqrt(&m, n);
            let mut cp = vec![0.0f64; n * n];
            for i in 0..n {
                for q in 0..n {
                    let mut acc = 0.0;
                    for p in 0..n {
                        acc += c[i * n + p] * x[p * n + q];
                    }
                    cp[i * n + q] = acc;
                }
            }
            cp
        }
    };
    let c_d2: Vec<D2> = c_prime.iter().map(|&x| D2::c(x)).collect();
    let mo = transform(&ao, &c_d2, n);
    let (_, n_alpha, n_beta) = electron_counts(&species);
    let space = FciSpace::new(n, n_alpha, n_beta);
    let e_nuc = basis.nuclear_repulsion().v;
    Setup { sa, sb, species, n, n_a, mo, space, e_nuc, s_cross_max, m: m_kept }
}

/// THE UNDEFORMED HEITLER–LONDON STATE (FIELD-6): the antisymmetrised product of the two
/// monomers' exact wavefunctions in their OWN orbitals, expressed in the orthonormalised
/// basis `C'` and evaluated there. FIELD-5 measured what the orthogonalised product costs:
/// `C' = C·M^{−1/2}` deforms each monomer's wavefunction at order `S²`, and that deformation
/// penalty, not Pauli exchange, was most of the 40 mHa it read at the hydrogen-bond minimum.
///
/// Here the original orbitals are written in the orthonormal ones, `C = C'·T` with
/// `T = M^{1/2}`, so a determinant of original orbitals occupying the set `P` expands as
/// `Σ_Q det(T[Q, P]) |Q⟩` over the equal-sized sets `Q` of orthonormal orbitals. Per spin
/// lane the product of a monomer-A string and a monomer-B string occupies `P = P_A ∪ P_B`
/// (A's orbitals index below B's, so the ascending order is the product order), and the
/// dimer vector is the contraction
///
/// ```text
///     v[Q_α, Q_β] = Σ  c_A(P_A^α, P_A^β) · c_B(P_B^α, P_B^β) · det T[Q_α, P_A^α ∪ P_B^α] · det T[Q_β, P_A^β ∪ P_B^β]
/// ```
///
/// — `T_α (1001 × 441) · C_prod (441 × 441) · T_βᵀ (441 × 1001)` on the water dimer, a few
/// hundred million flops. `E_HL = ⟨v|H|v⟩/⟨v|v⟩ + E_nuc` by one `sigma`; the norm `⟨v|v⟩` is
/// the non-orthogonal product's own overlap and is REPORTED, not assumed to be 1.
pub fn heitler_london_undeformed(a: &Fragment, b: &Fragment) -> HlReading {
    let st = setup(a, b, HlPlant::None);
    let n = st.n;
    let n_a = st.n_a;
    let t = sqrt_sym(&st.m, n);
    let (spa, spb) = (&st.sa.gp.space, &st.sb.gp.space);
    let space = &st.space;
    // per lane: the expansion matrix of every (A-string, B-string) product in the dimer's strings
    let lane = |dimer: &crate::fci::Strings, sa_l: &crate::fci::Strings, sb_l: &crate::fci::Strings| -> (Vec<f64>, usize, usize) {
        let (na, nb) = (sa_l.masks.len(), sb_l.masks.len());
        let k = dimer.n_elec;
        let nq = dimer.masks.len();
        let mut out = vec![0.0f64; nq * na * nb];
        let mut buf = vec![0.0f64; k * k];
        for (iq, &mq) in dimer.masks.iter().enumerate() {
            let rows: Vec<usize> = (0..n).filter(|&o| mq >> o & 1 == 1).collect();
            for (ja, &ma) in sa_l.masks.iter().enumerate() {
                for (jb, &mb) in sb_l.masks.iter().enumerate() {
                    let pm: u64 = ma | (mb << n_a);
                    let cols: Vec<usize> = (0..n).filter(|&o| pm >> o & 1 == 1).collect();
                    debug_assert_eq!(cols.len(), k);
                    for (r, &qr) in rows.iter().enumerate() {
                        for (c, &pc) in cols.iter().enumerate() {
                            buf[r * k + c] = t[qr * n + pc];
                        }
                    }
                    out[(iq * na + ja) * nb + jb] = det_small(&mut buf, k);
                }
            }
        }
        (out, na, nb)
    };
    let (ta, na_a, nb_a) = lane(space.alpha(), spa.alpha(), spb.alpha());
    let (tb, na_b, nb_b) = lane(space.beta(), spa.beta(), spb.beta());
    let (nqa, nqb) = (space.alpha().masks.len(), space.beta().masks.len());
    // C_prod[(ja, jb), (ka, kb)] = c_A[ja, ka] · c_B[jb, kb]
    let pa_n = na_a * nb_a;
    let pb_n = na_b * nb_b;
    let mut cprod = vec![0.0f64; pa_n * pb_n];
    for ja in 0..na_a {
        for jb in 0..nb_a {
            for ka in 0..na_b {
                for kb in 0..nb_b {
                    let ca = st.sa.sol.vector[ja * spa.strides[0] + ka * spa.strides[1]];
                    let cb = st.sb.sol.vector[jb * spb.strides[0] + kb * spb.strides[1]];
                    cprod[(ja * nb_a + jb) * pb_n + (ka * nb_b + kb)] = ca * cb;
                }
            }
        }
    }
    // W = T_α · C_prod  (nqa × pb_n)
    let mut w = vec![0.0f64; nqa * pb_n];
    for iq in 0..nqa {
        for j in 0..pa_n {
            let x = ta[iq * pa_n + j];
            if x == 0.0 {
                continue;
            }
            let row = &cprod[j * pb_n..(j + 1) * pb_n];
            let wrow = &mut w[iq * pb_n..(iq + 1) * pb_n];
            for (wv, cv) in wrow.iter_mut().zip(row.iter()) {
                *wv += x * cv;
            }
        }
    }
    // v = W · T_βᵀ  (nqa × nqb)
    let mut v = vec![0.0f64; space.n_det];
    let mut nonzero = 0usize;
    for iq in 0..nqa {
        let wrow = &w[iq * pb_n..(iq + 1) * pb_n];
        for jq in 0..nqb {
            let trow = &tb[jq * pb_n..(jq + 1) * pb_n];
            let mut acc = 0.0;
            for (x, y) in wrow.iter().zip(trow.iter()) {
                acc += x * y;
            }
            if acc != 0.0 {
                nonzero += 1;
            }
            v[iq * space.strides[0] + jq * space.strides[1]] = acc;
        }
    }
    let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    assert!(norm > 0.0, "the undeformed product state is empty");
    let ci = ci_ints(&st.mo, Order::Value);
    let mut hv = vec![0.0f64; space.n_det];
    let t0 = Instant::now();
    space.sigma(&ci, &v, &mut hv);
    let sigma_seconds = t0.elapsed().as_secs_f64();
    let vhv: f64 = v.iter().zip(hv.iter()).map(|(x, y)| x * y).sum();
    let e_hl = vhv / (norm * norm) + st.e_nuc;
    let pa = ao_density(&rdm1(spa, &st.sa.sol.vector), &st.sa.gp.orbitals, st.sa.basis.n);
    let pb = ao_density(&rdm1(spb, &st.sb.sol.vector), &st.sb.gp.orbitals, st.sb.basis.n);
    let e_es = classical_interaction(a, &pa, b, &pb);
    let _ = &st.species;
    HlReading {
        e_hl,
        e_a0: st.sa.e_total,
        e_b0: st.sb.e_total,
        e_es,
        e_exch: e_hl - st.sa.e_total - st.sb.e_total - e_es,
        norm,
        nonzero_dets: nonzero,
        n_det: space.n_det,
        n_det_a: spa.n_det,
        n_det_b: spb.n_det,
        s_cross_max: st.s_cross_max,
        sigma_seconds,
    }
}

/// The Heitler–London reading for two fragments (§0 of the freeze).
pub fn heitler_london(a: &Fragment, b: &Fragment, plant: HlPlant) -> HlReading {
    let st = setup(a, b, plant);
    let (spa, spb) = (&st.sa.gp.space, &st.sb.gp.space);
    let n_a = st.n_a;
    let mask_a: u64 = (1u64 << n_a) - 1;
    let space = &st.space;
    let mut v = vec![0.0f64; space.n_det];
    let mut nonzero = 0usize;
    let (ea_a, ea_b) = (spa.alpha().n_elec, spb.alpha().n_elec);
    let (eb_a, eb_b) = (spa.beta().n_elec, spb.beta().n_elec);
    for (ia, &ma) in space.alpha().masks.iter().enumerate() {
        let (ma_a, ma_b) = (ma & mask_a, ma >> n_a);
        if ma_a.count_ones() as usize != ea_a || ma_b.count_ones() as usize != ea_b {
            continue;
        }
        let (Some(ja), Some(ka)) = (spa.alpha().index_of(ma_a), spb.alpha().index_of(ma_b)) else {
            continue;
        };
        for (ib, &mb) in space.beta().masks.iter().enumerate() {
            let (mb_a, mb_b) = (mb & mask_a, mb >> n_a);
            if mb_a.count_ones() as usize != eb_a || mb_b.count_ones() as usize != eb_b {
                continue;
            }
            let (Some(jb), Some(kb)) = (spa.beta().index_of(mb_a), spb.beta().index_of(mb_b)) else {
                continue;
            };
            let ca = st.sa.sol.vector[ja * spa.strides[0] + jb * spa.strides[1]];
            let cb = st.sb.sol.vector[ka * spb.strides[0] + kb * spb.strides[1]];
            let x = ca * cb;
            if x != 0.0 {
                nonzero += 1;
            }
            v[ia * space.strides[0] + ib * space.strides[1]] = x;
        }
    }
    let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    assert!(norm > 0.0, "the product state is empty: the monomer electron counts do not fit the dimer's strings");
    let ci = ci_ints(&st.mo, Order::Value);
    let mut hv = vec![0.0f64; space.n_det];
    let t0 = Instant::now();
    space.sigma(&ci, &v, &mut hv);
    let sigma_seconds = t0.elapsed().as_secs_f64();
    let vhv: f64 = v.iter().zip(hv.iter()).map(|(x, y)| x * y).sum();
    let e_elec = vhv / (norm * norm);
    let e_hl = e_elec + st.e_nuc;
    // the isolated densities and their classical interaction
    let pa = ao_density(&rdm1(spa, &st.sa.sol.vector), &st.sa.gp.orbitals, st.sa.basis.n);
    let pb = ao_density(&rdm1(spb, &st.sb.sol.vector), &st.sb.gp.orbitals, st.sb.basis.n);
    let e_es = classical_interaction(a, &pa, b, &pb);
    let e_exch = e_hl - st.sa.e_total - st.sb.e_total - e_es;
    let _ = st.n;
    let _ = &st.species;
    HlReading {
        e_hl,
        e_a0: st.sa.e_total,
        e_b0: st.sb.e_total,
        e_es,
        e_exch,
        norm,
        nonzero_dets: nonzero,
        n_det: space.n_det,
        n_det_a: spa.n_det,
        n_det_b: spb.n_det,
        s_cross_max: st.s_cross_max,
        sigma_seconds,
    }
}

/// The dimer's FULL CI in the same orthogonalised orbital basis (gate G-H0): a full CI
/// energy is invariant under an orthonormal change of orbitals, so this must reproduce the
/// supermolecule's record; under `HlPlant::SkipOrthogonalisation` it must not.
pub fn fci_in_hl_basis(a: &Fragment, b: &Fragment, plant: HlPlant) -> (f64, Solution) {
    let st = setup(a, b, plant);
    let sol = solve_determinant(&st.space, &st.mo);
    (sol.e.v + st.e_nuc, sol)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::by_symbol;
    use crate::embed::supermolecule;

    fn h2(z: f64) -> Fragment {
        let h = by_symbol("H").unwrap();
        Fragment::new(vec![h, h], vec![[0.0, 0.0, z], [0.0, 0.0, z + 1.4]], vec![-1.0, 1.0])
    }

    #[test]
    fn the_product_state_is_normalised_counts_its_determinants_and_vanishes_at_distance() {
        let (a, b) = (h2(0.0), h2(4.0));
        let r = heitler_london(&a, &b, HlPlant::None);
        assert!((r.norm - 1.0).abs() < 1e-12, "norm {}", r.norm);
        assert_eq!(r.nonzero_dets, r.n_det_a * r.n_det_b);
        assert!(r.e_exch > 0.0, "exchange at 4 bohr is repulsive: {}", r.e_exch);
        assert!(r.s_cross_max > 1e-3);
        let far = heitler_london(&a, &h2(40.0), HlPlant::None);
        assert!((far.e_hl - far.e_a0 - far.e_b0).abs() < 1e-8, "the 40-bohr limit: {}", far.e_hl - far.e_a0 - far.e_b0);
        assert!(far.e_exch.abs() < 1e-8);
    }

    #[test]
    fn the_full_ci_in_the_orthogonalised_basis_is_the_supermolecule_and_the_plant_is_not() {
        let (a, b) = (h2(0.0), h2(3.0));
        let record = supermolecule(&a, &b).e_total;
        let (e, sol) = fci_in_hl_basis(&a, &b, HlPlant::None);
        assert!((e - record).abs() < 1e-9, "invariance: {e} vs {record} (residual {})", sol.residual);
        let (e_plant, _) = fci_in_hl_basis(&a, &b, HlPlant::SkipOrthogonalisation);
        assert!((e_plant - record).abs() > 1e-3, "the plant did not fire: {e_plant} vs {record}");
        // the Heitler–London energy lies above the full CI (a variational trial state)
        let r = heitler_london(&a, &b, HlPlant::None);
        assert!(r.e_hl >= record - 1e-10, "HL {} below the exact {}", r.e_hl, record);
        // the UNDEFORMED product: between the exact and the orthogonalised (the deformation
        // penalty is what the orthogonalised state pays on top), with the product's own norm
        let u = heitler_london_undeformed(&a, &b);
        assert!(u.e_hl >= record - 1e-10, "undeformed HL {} below the exact {}", u.e_hl, record);
        assert!(u.e_hl <= r.e_hl + 1e-10, "undeformed HL {} above the orthogonalised {}", u.e_hl, r.e_hl);
        assert!(u.norm > 0.0 && u.norm <= 1.0 + 1e-12, "the non-orthogonal product's overlap {}", u.norm);
        assert!(u.e_exch > 0.0);
        let far = heitler_london_undeformed(&a, &h2(40.0));
        assert!((far.norm - 1.0).abs() < 1e-8 && far.e_exch.abs() < 1e-8, "undeformed at 40 bohr: norm {}, exch {}", far.norm, far.e_exch);
        eprintln!("H2·H2 at 3 bohr: exact {record:.9}, undeformed HL {:.9} (exch {:+.3e}, norm {:.6}), orthogonalised HL {:.9} (exch {:+.3e})", u.e_hl, u.e_exch, u.norm, r.e_hl, r.e_exch);
    }
}
