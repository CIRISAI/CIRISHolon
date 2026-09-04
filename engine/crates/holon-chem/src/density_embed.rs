//! The field as the partners' own densities — EMBED-2
//! (`conformance/water_observatory/EMBED2_PREREG.md`).
//!
//! A fragment `A` in the field of partners `{B}` with frozen embedded densities `P_B`:
//!
//! ```text
//! h_A^emb  = h_A + Σ_B [ V_nuc(Z_b) + J[P_B] ],   J[P_B]_μν = Σ_{λσ∈B} P_B,λσ (μν|λσ)
//! E_A[{B}] = E_elec(h_A^emb) + E_nn(A) + Σ_B E_nn(A, Z_b) − Σ_B Σ_{a∈A} Z_a V_B^el(R_a)
//! ```
//!
//! The partners' nuclei are external centres (EMBED-1's door); their electrons enter as
//! the Coulomb matrix on the fragment's basis (`pair::geometry_problem_with_potential`)
//! and as the potential at the fragment's nuclei (EMBED-1's unit attraction). Every
//! partner–partner term is excluded by construction. Coulomb only — no Pauli term: the
//! Coulomb part of Wesolowski–Warshel's frozen-density embedding, credited in the freeze.

use crate::elements::Species;
use crate::embed::{ao_density, build_basis_embedded, rdm1, unit_attraction, External, Fragment};
use crate::fci::{solve_determinant, Solution};
use crate::md::{ao_integrals, Basis};
use crate::pair::{geometry_problem_with_potential, GeometryProblem};

/// A partner: its fragment and its frozen density in its own AO basis.
pub struct Partner<'a> {
    pub frag: &'a Fragment,
    pub p: &'a [f64],
    /// Whether the partner's NUCLEI enter as external centres (always, except PLANT (ii) of
    /// EMBED-3, which drops one partner's nuclei while its density stays).
    pub nuclei: bool,
}

impl<'a> Partner<'a> {
    pub fn new(frag: &'a Fragment, p: &'a [f64]) -> Partner<'a> {
        Partner { frag, p, nuclei: true }
    }
}

/// `J[P_B]` on `a`'s functions: the combined basis `A ∪ B` assembled once, its ERIs by the
/// existing `ao_integrals`, the `(μν|λσ)` block with `μν` on A and `λσ` on B contracted
/// with `P_B`. Returned `n_A × n_A`, row-major.
pub fn coulomb_from_partner(a: &Fragment, b: &Fragment, p_b: &[f64]) -> Vec<f64> {
    let mut species = a.species.clone();
    species.extend_from_slice(&b.species);
    let mut centers = a.centers.clone();
    centers.extend_from_slice(&b.centers);
    let basis = build_basis_embedded(&species, &centers, &[]);
    let ao = ao_integrals(&basis);
    let n = ao.n;
    let na = build_basis_embedded(&a.species, &a.centers, &[]).n;
    let nb = n - na;
    assert_eq!(p_b.len(), nb * nb, "partner density must be nb × nb");
    let mut j = vec![0.0f64; na * na];
    for mu in 0..na {
        for nu in 0..na {
            let mut acc = 0.0;
            for l in 0..nb {
                for s in 0..nb {
                    acc += p_b[l * nb + s] * ao.g(mu, nu, na + l, na + s).v;
                }
            }
            j[mu * na + nu] = acc;
        }
    }
    j
}

/// `V_B^el(R)`: the partner density's potential at a point, by the unit-charge attraction
/// on B's basis contracted with `P_B` (positive for a positive density).
pub fn partner_potential_at(b: &Fragment, p_b: &[f64], r: [f64; 3]) -> f64 {
    let att = unit_attraction(&b.species, &b.centers, r);
    let n = (p_b.len() as f64).sqrt().round() as usize;
    let mut v = 0.0;
    for i in 0..n {
        for k in 0..n {
            v += p_b[i * n + k] * att[k * n + i];
        }
    }
    v
}

/// One fragment solved in its partners' densities and nuclei.
pub struct DensitySolve {
    pub basis: Basis,
    pub gp: GeometryProblem,
    pub sol: Solution,
    pub gamma: Vec<f64>,
    /// The fragment's own embedded density, AO basis.
    pub p: Vec<f64>,
    /// `Σ_μν P_A J_μν` — the electron–electron part of the interaction with the partners.
    pub e_ee: f64,
    /// `Σ_B Σ_a Z_a V_B^el(R_a)` — the fragment's nuclei in the partners' electron clouds.
    pub e_ne: f64,
    pub e_total: f64,
}

/// `plant_j_sign` is PLANT (i) (J enters attractive); `plant_drop_nuclei` is PLANT (ii)
/// (the partners' nuclei omitted while J stays).
pub fn solve_in_densities_with(frag: &Fragment, partners: &[Partner], plant_j_sign: bool, plant_drop_nuclei: bool) -> DensitySolve {
    // the partners' nuclei as external centres
    let mut ext = Vec::new();
    if !plant_drop_nuclei {
        for pt in partners {
            if !pt.nuclei {
                continue;
            }
            for (c, sp) in pt.frag.centers.iter().zip(pt.frag.species.iter()) {
                ext.push(External { r: *c, q: sp.z as f64 });
            }
        }
    }
    let basis = build_basis_embedded(&frag.species, &frag.centers, &ext);
    let n = basis.n;
    // J from every partner's density, summed
    let mut j = vec![0.0f64; n * n];
    for pt in partners {
        let jb = coulomb_from_partner(frag, pt.frag, pt.p);
        for k in 0..n * n {
            j[k] += if plant_j_sign { -jb[k] } else { jb[k] };
        }
    }
    let gp = geometry_problem_with_potential(&basis, &frag.species, if partners.is_empty() { None } else { Some(&j) });
    let sol = solve_determinant(&gp.space, &gp.mo);
    let gamma = rdm1(&gp.space, &sol.vector);
    let p = ao_density(&gamma, &gp.orbitals, n);
    let mut e_ee = 0.0;
    for mu in 0..n {
        for nu in 0..n {
            e_ee += p[mu * n + nu] * j[nu * n + mu];
        }
    }
    // the fragment's nuclei in the partners' electron clouds
    let mut e_ne = 0.0;
    for pt in partners {
        for (c, sp) in frag.centers.iter().zip(frag.species.iter()) {
            e_ne += sp.z as f64 * partner_potential_at(pt.frag, pt.p, *c);
        }
    }
    // external self-energy (partner nuclei with partner nuclei) removed, as in EMBED-1
    let self_e = crate::embed::external_self_energy(&ext);
    let e_nuc = gp.e_nuc.v - self_e;
    let e_total = sol.e.v + e_nuc - e_ne;
    DensitySolve { basis, gp, sol, gamma, p, e_ee, e_ne, e_total }
}

pub fn solve_in_densities(frag: &Fragment, partners: &[Partner]) -> DensitySolve {
    solve_in_densities_with(frag, partners, false, false)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DensityStart {
    /// Every partner density zero (and no partner nuclei) at the first sweep.
    Zero,
    /// Each fragment's isolated density at the first sweep.
    Isolated,
}

pub struct DensityFixedPoint {
    pub converged: bool,
    pub sweeps: usize,
    pub last_delta: f64,
    pub densities: Vec<Vec<f64>>,
}

fn partners_except<'a>(frags: &'a [Fragment], dens: &'a [Vec<f64>], except: &[usize]) -> Vec<Partner<'a>> {
    partners_except_flagged(frags, dens, except, None)
}

/// As [`partners_except`], with one partner's nuclei dropped (PLANT (ii) of EMBED-3).
pub fn partners_except_flagged<'a>(frags: &'a [Fragment], dens: &'a [Vec<f64>], except: &[usize], drop_nuclei_of: Option<usize>) -> Vec<Partner<'a>> {
    frags
        .iter()
        .enumerate()
        .filter(|(j, _)| !except.contains(j))
        .map(|(j, f)| Partner { frag: f, p: &dens[j], nuclei: drop_nuclei_of != Some(j) })
        .collect()
}

/// The density-embedded pairwise sum over a SUBSET of the fragments, every solve in the
/// field of ALL the others — inside and outside the subset (EMBED-3's `ρPA_ABC[ρ_D]`).
/// `drop_nuclei_of` is PLANT (ii). With `subset` = everything this is [`rho_pa`].
pub fn rho_pa_subset(frags: &[Fragment], dens: &[Vec<f64>], subset: &[usize], drop_nuclei_of: Option<usize>) -> RhoPa {
    let n = subset.len();
    let e_mono: Vec<f64> = subset
        .iter()
        .map(|&i| solve_in_densities(&frags[i], &partners_except_flagged(frags, dens, &[i], drop_nuclei_of)).e_total)
        .collect();
    let mut e_dimer = Vec::new();
    for a in 0..n {
        for b in (a + 1)..n {
            let (i, j) = (subset[a], subset[b]);
            let (sp, ce) = crate::seam::joined(&frags[i], &frags[j]);
            let mut w = frags[i].weights.clone();
            w.extend_from_slice(&frags[j].weights);
            let dimer = Fragment::new(sp, ce, w);
            let pts = partners_except_flagged(frags, dens, &[i, j], drop_nuclei_of);
            e_dimer.push((i, j, solve_in_densities(&dimer, &pts).e_total));
        }
    }
    let total = e_dimer.iter().map(|d| d.2).sum::<f64>() - (n as f64 - 2.0) * e_mono.iter().sum::<f64>();
    RhoPa { e_mono, e_dimer, total }
}

/// The exact supermolecule of a SUBSET of the fragments solved in the field of all the others.
pub fn subset_in_field(frags: &[Fragment], dens: &[Vec<f64>], subset: &[usize], drop_nuclei_of: Option<usize>) -> DensitySolve {
    let mut species = Vec::new();
    let mut centers = Vec::new();
    let mut weights = Vec::new();
    for &i in subset {
        species.extend_from_slice(&frags[i].species);
        centers.extend_from_slice(&frags[i].centers);
        weights.extend_from_slice(&frags[i].weights);
    }
    let joined = Fragment::new(species, centers, weights);
    solve_in_densities(&joined, &partners_except_flagged(frags, dens, subset, drop_nuclei_of))
}

/// The classical electrostatic interaction of two frozen fragments (densities and nuclei):
/// `nn(A,B) − Σ_a Z_a V_B^el(R_a) − Σ_b Z_b V_A^el(R_b) + tr(P_A J[P_B])`. Symmetric under
/// `A ↔ B` to roundoff — a test on the ERI-block extraction (EMBED-3 G4).
pub fn classical_interaction(a: &Fragment, p_a: &[f64], b: &Fragment, p_b: &[f64]) -> f64 {
    let nn = nn_between(a, b);
    let mut ne = 0.0;
    for (c, sp) in a.centers.iter().zip(a.species.iter()) {
        ne += sp.z as f64 * partner_potential_at(b, p_b, *c);
    }
    for (c, sp) in b.centers.iter().zip(b.species.iter()) {
        ne += sp.z as f64 * partner_potential_at(a, p_a, *c);
    }
    let j = coulomb_from_partner(a, b, p_b);
    let n = (p_a.len() as f64).sqrt().round() as usize;
    let mut ee = 0.0;
    for mu in 0..n {
        for nu in 0..n {
            ee += p_a[mu * n + nu] * j[nu * n + mu];
        }
    }
    nn - ne + ee
}

/// The fixed point of the densities: each fragment solved in the others' current densities,
/// Gauss–Seidel, until the largest density-matrix change is below `1e-9`; at most 100 sweeps.
pub fn embed_densities(frags: &[Fragment], start: DensityStart) -> DensityFixedPoint {
    let n = frags.len();
    let mut dens: Vec<Vec<f64>> = match start {
        DensityStart::Zero => frags.iter().map(|f| vec![0.0; build_basis_embedded(&f.species, &f.centers, &[]).n.pow(2)]).collect(),
        DensityStart::Isolated => frags.iter().map(|f| solve_in_densities(f, &[]).p).collect(),
    };
    // a zero density with partner nuclei present would be a bare nucleus: the Zero start
    // therefore begins with NO partners at all (isolated), and both starts differ only in
    // whether the first sweep's partners are isolated densities or the empty field
    let mut first = matches!(start, DensityStart::Zero);
    let mut sweeps = 0;
    let mut converged = false;
    let mut last_delta = f64::INFINITY;
    loop {
        sweeps += 1;
        let mut delta = 0.0f64;
        for i in 0..n {
            let ds = if first {
                solve_in_densities(&frags[i], &[])
            } else {
                let pts = partners_except(frags, &dens, &[i]);
                solve_in_densities(&frags[i], &pts)
            };
            let d = ds.p.iter().zip(dens[i].iter()).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
            delta = delta.max(d);
            dens[i] = ds.p;
        }
        first = false;
        last_delta = delta;
        if delta < 1e-9 && sweeps > 1 {
            converged = true;
            break;
        }
        if sweeps >= 100 {
            break;
        }
    }
    DensityFixedPoint { converged, sweeps, last_delta, densities: dens }
}

pub struct RhoPa {
    pub e_mono: Vec<f64>,
    pub e_dimer: Vec<(usize, usize, f64)>,
    pub total: f64,
}

/// `Σ_{i<j} E_ij[ρ of the rest] − (N−2) Σ_i E_i[ρ of the others]`, with the plants passed through.
pub fn rho_pa_with(frags: &[Fragment], dens: &[Vec<f64>], plant_j_sign: bool, plant_drop_nuclei: bool) -> RhoPa {
    let n = frags.len();
    let e_mono: Vec<f64> = (0..n)
        .map(|i| solve_in_densities_with(&frags[i], &partners_except(frags, dens, &[i]), plant_j_sign, plant_drop_nuclei).e_total)
        .collect();
    let mut e_dimer = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let (sp, ce) = crate::seam::joined(&frags[i], &frags[j]);
            let dimer = Fragment::new(sp, ce, {
                let mut w = frags[i].weights.clone();
                w.extend_from_slice(&frags[j].weights);
                w
            });
            let pts = partners_except(frags, dens, &[i, j]);
            e_dimer.push((i, j, solve_in_densities_with(&dimer, &pts, plant_j_sign, plant_drop_nuclei).e_total));
        }
    }
    let total = e_dimer.iter().map(|d| d.2).sum::<f64>() - (n as f64 - 2.0) * e_mono.iter().sum::<f64>();
    RhoPa { e_mono, e_dimer, total }
}

pub fn rho_pa(frags: &[Fragment], dens: &[Vec<f64>]) -> RhoPa {
    rho_pa_with(frags, dens, false, false)
}

/// The species' nuclear repulsion between two fragments, for plant (ii)'s carrier.
pub fn nn_between(a: &Fragment, b: &Fragment) -> f64 {
    let mut acc = 0.0;
    for (ca, sa) in a.centers.iter().zip(a.species.iter()) {
        for (cb, sb) in b.centers.iter().zip(b.species.iter()) {
            let d = ((ca[0] - cb[0]).powi(2) + (ca[1] - cb[1]).powi(2) + (ca[2] - cb[2]).powi(2)).sqrt();
            acc += (sa.z as f64) * (sb.z as f64) / d;
        }
    }
    acc
}

#[allow(dead_code)]
fn _species_used(_: Species) {}
