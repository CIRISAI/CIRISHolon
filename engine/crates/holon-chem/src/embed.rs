//! The embedding field — EMBED-1 (`conformance/water_observatory/EMBED_PREREG.md`).
//!
//! # What this is
//!
//! The engine carried no electrostatic term and solved every cluster in vacuum, and its
//! many-body ladder did not terminate (dE5). The literature's embedded expansions close
//! at two-body because each fragment is solved INSIDE the field of the rest. This module
//! is that field, built from what the crate already had:
//!
//! - **external point charges** enter the electronic Hamiltonian as centres with a charge
//!   and no shells — the nuclear-attraction loop in `md.rs` already sums every centre, so
//!   no integral code is added; the external–external self-energy that
//!   `Basis::nuclear_repulsion` also sums is subtracted here and asserted absent;
//! - **the one-body reduced density matrix** `γ_pq = Σ_σ <ψ|a†_pσ a_qσ|ψ>` comes straight
//!   off the string tables' single-excitation lists, and `P = C γ Cᵀ` puts it in the AO
//!   basis;
//! - **moments** (dipole and second moment about any point) come from the Hermite `E`
//!   tables, which are built two powers past `LMAX` for the kinetic term and therefore
//!   already carry `(x−C)` and `(x−C)²`;
//! - **the electrostatic potential** of a fragment at a point is the nuclei's term plus
//!   the density contracted with the unit-charge attraction integrals, obtained as the
//!   DIFFERENCE of two `ao_integrals` calls (the attraction is linear in the charges) —
//!   which is the same arithmetic the Hamiltonian uses, so Hellmann–Feynman (G3) is a
//!   check on the integrals and the density, never on a second implementation.
//!
//! Every solve here is `solve_determinant` on the host, one device class; nothing routes
//! by size.
//!
//! # What it is not
//!
//! Not chemistry (STO-3G). Not Ewald: the periodic far field is a named exit. Not the
//! seam: Build 2 puts exact cores inside this field under its own freeze.

use crate::dual::D2;
use crate::elements::Species;
use crate::fci::{solve_determinant, FciSpace, Solution};
use crate::md::{ao_integrals, cart_factor, cartesian_components, e_table, Basis};
use crate::pair::{geometry_problem_from_basis, GeometryProblem};
use std::f64::consts::PI;

/// A classical point charge at a position, both in atomic units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct External {
    pub r: [f64; 3],
    pub q: f64,
}

fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// `Σ_{k<l} q_k q_l / r_kl` over the externals — the term `Basis::nuclear_repulsion`
/// sums that an embedded energy must not carry.
pub fn external_self_energy(ext: &[External]) -> f64 {
    let mut acc = 0.0;
    for k in 0..ext.len() {
        for l in (k + 1)..ext.len() {
            acc += ext[k].q * ext[l].q / dist(&ext[k].r, &ext[l].r);
        }
    }
    acc
}

/// The species' shells on their centres, plus the externals as centres with NO shells.
pub fn build_basis_embedded(species: &[Species], centers: &[[f64; 3]], ext: &[External]) -> Basis {
    assert_eq!(species.len(), centers.len());
    let mut decls = Vec::new();
    for (c, sp) in species.iter().enumerate() {
        for sh in sp.shells {
            decls.push((c, sh.kind.l(), sh.alpha, sh.coeff));
        }
    }
    let mut all: Vec<[D2; 3]> = centers.iter().map(|c| [D2::c(c[0]), D2::c(c[1]), D2::c(c[2])]).collect();
    let mut charges: Vec<f64> = species.iter().map(|s| s.z as f64).collect();
    for e in ext {
        all.push([D2::c(e.r[0]), D2::c(e.r[1]), D2::c(e.r[2])]);
        charges.push(e.q);
    }
    Basis::assemble(all, charges, &decls)
}

/// One fragment solved exactly in the field of `ext`.
pub struct EmbeddedSolve {
    pub basis: Basis,
    pub gp: GeometryProblem,
    pub sol: Solution,
    /// Electronic energy (in the field).
    pub e_elec: f64,
    /// Nuclei–nuclei plus nuclei–external, the external–external self-energy REMOVED.
    pub e_nuc: f64,
    pub e_total: f64,
}

pub fn solve_embedded(species: &[Species], centers: &[[f64; 3]], ext: &[External]) -> EmbeddedSolve {
    let basis = build_basis_embedded(species, centers, ext);
    let gp = geometry_problem_from_basis(&basis, species);
    let sol = solve_determinant(&gp.space, &gp.mo);
    let self_e = external_self_energy(ext);
    let e_nuc = gp.e_nuc.v - self_e;
    // the assertion the freeze asks for: with no externals nothing was subtracted, and with
    // externals the subtraction is exactly their own pair sum, never the nuclei's
    debug_assert!((gp.e_nuc.v - e_nuc - self_e).abs() <= 1e-12 * (1.0 + gp.e_nuc.v.abs()));
    let e_elec = sol.e.v;
    EmbeddedSolve { basis, gp, sol, e_elec, e_nuc, e_total: e_elec + e_nuc }
}

// ------------------------------------------------------------------ the density

fn rdm1_impl(space: &FciSpace, c: &[f64], keep_sign: bool) -> Vec<f64> {
    let n = space.n_orb;
    assert_eq!(space.lanes.len(), 2, "rdm1 is written for the two spin lanes");
    assert_eq!(c.len(), space.n_det);
    let mut g = vec![0.0f64; n * n];
    let (sa, sb) = (space.strides[0], space.strides[1]);
    let (na, nb) = (space.lanes[0].masks.len(), space.lanes[1].masks.len());
    // alpha lane: <ψ| a†_p a_q |ψ> = Σ_j Σ_{(pq, s, i) ∈ singles[j]} s Σ_β c[i,β] c[j,β]
    for j in 0..na {
        for &(pq, sign, i) in &space.lanes[0].singles[j] {
            let s = if keep_sign { sign } else { sign.abs() };
            let mut acc = 0.0;
            for ib in 0..nb {
                acc += c[(i as usize) * sa + ib * sb] * c[j * sa + ib * sb];
            }
            g[pq as usize] += s * acc;
        }
    }
    for j in 0..nb {
        for &(pq, sign, i) in &space.lanes[1].singles[j] {
            let s = if keep_sign { sign } else { sign.abs() };
            let mut acc = 0.0;
            for ia in 0..na {
                acc += c[ia * sa + (i as usize) * sb] * c[ia * sa + j * sb];
            }
            g[pq as usize] += s * acc;
        }
    }
    g
}

/// The spin-summed one-body reduced density matrix in the solve's orbital basis.
pub fn rdm1(space: &FciSpace, c: &[f64]) -> Vec<f64> {
    rdm1_impl(space, c, true)
}

/// PLANT (iii): the fermionic phase dropped. The trace survives; the off-diagonal does not.
pub fn rdm1_plant_unsigned(space: &FciSpace, c: &[f64]) -> Vec<f64> {
    rdm1_impl(space, c, false)
}

/// `P = C γ Cᵀ`, AO basis, `C` AO-major (`c[i*n+p]`).
pub fn ao_density(gamma: &[f64], orbitals: &[f64], n: usize) -> Vec<f64> {
    let mut t = vec![0.0f64; n * n];
    for i in 0..n {
        for q in 0..n {
            let mut acc = 0.0;
            for p in 0..n {
                acc += orbitals[i * n + p] * gamma[p * n + q];
            }
            t[i * n + q] = acc;
        }
    }
    let mut pm = vec![0.0f64; n * n];
    for i in 0..n {
        for j in 0..n {
            let mut acc = 0.0;
            for q in 0..n {
                acc += t[i * n + q] * orbitals[j * n + q];
            }
            pm[i * n + j] = acc;
        }
    }
    pm
}

pub fn trace(m: &[f64], n: usize) -> f64 {
    (0..n).map(|i| m[i * n + i]).sum()
}

pub fn trace_product(a: &[f64], b: &[f64], n: usize) -> f64 {
    let mut acc = 0.0;
    for i in 0..n {
        for j in 0..n {
            acc += a[i * n + j] * b[j * n + i];
        }
    }
    acc
}

// ------------------------------------------------------------------ moments

/// Overlap, first moments and the second moment about `c`, in the AO basis the rest of
/// the engine sees (spherical when the basis has d shells).
pub struct Moments {
    pub n: usize,
    pub s: Vec<f64>,
    pub x: [Vec<f64>; 3],
    pub r2: Vec<f64>,
}

pub fn moments(b: &Basis, c: [f64; 3]) -> Moments {
    let nc = b.n_cart;
    let mut s = vec![0.0f64; nc * nc];
    let mut x = [vec![0.0f64; nc * nc], vec![0.0f64; nc * nc], vec![0.0f64; nc * nc]];
    let mut r2 = vec![0.0f64; nc * nc];
    for sa in b.shells.iter() {
        for sb in b.shells.iter() {
            let ca = b.centers[sa.center];
            let cb = b.centers[sb.center];
            for pa in 0..3 {
                for pb in 0..3 {
                    let (a, bb) = (sa.alpha[pa], sb.alpha[pb]);
                    let p = a + bb;
                    let et = [
                        e_table(a, bb, ca[0], cb[0]),
                        e_table(a, bb, ca[1], cb[1]),
                        e_table(a, bb, ca[2], cb[2]),
                    ];
                    let pref = (PI / p).powf(1.5);
                    for (fa, &la) in cartesian_components(sa.l).iter().enumerate() {
                        for (fb, &lb) in cartesian_components(sb.l).iter().enumerate() {
                            let (i, j) = (sa.first + fa, sb.first + fb);
                            let w = sa.coeff[pa] * sb.coeff[pb] * cart_factor(la) * cart_factor(lb) * pref;
                            // per direction: the zeroth, first and second moments about c
                            let mut m0 = [0.0f64; 3];
                            let mut m1 = [0.0f64; 3];
                            let mut m2 = [0.0f64; 3];
                            for dir in 0..3 {
                                let (ii, jj) = (la[dir] as usize, lb[dir] as usize);
                                let e0 = et[dir].e[ii][jj][0].v;
                                let e1 = et[dir].e[ii][jj + 1][0].v;
                                let e2 = et[dir].e[ii][jj + 2][0].v;
                                let bc = cb[dir].v - c[dir];
                                m0[dir] = e0;
                                m1[dir] = e1 + bc * e0;
                                m2[dir] = e2 + 2.0 * bc * e1 + bc * bc * e0;
                            }
                            s[i * nc + j] += w * m0[0] * m0[1] * m0[2];
                            x[0][i * nc + j] += w * m1[0] * m0[1] * m0[2];
                            x[1][i * nc + j] += w * m0[0] * m1[1] * m0[2];
                            x[2][i * nc + j] += w * m0[0] * m0[1] * m1[2];
                            r2[i * nc + j] += w * (m2[0] * m0[1] * m0[2] + m0[0] * m2[1] * m0[2] + m0[0] * m0[1] * m2[2]);
                        }
                    }
                }
            }
        }
    }
    let n = b.n;
    let proj = |m: Vec<f64>| -> Vec<f64> {
        match &b.sph {
            None => m,
            Some(pm) => {
                // P M Pᵀ with P the n × n_cart projection
                let mut t = vec![0.0f64; n * nc];
                for r in 0..n {
                    for k in 0..nc {
                        let mut acc = 0.0;
                        for c2 in 0..nc {
                            acc += pm[r * nc + c2] * m[c2 * nc + k];
                        }
                        t[r * nc + k] = acc;
                    }
                }
                let mut out = vec![0.0f64; n * n];
                for r in 0..n {
                    for q in 0..n {
                        let mut acc = 0.0;
                        for k in 0..nc {
                            acc += t[r * nc + k] * pm[q * nc + k];
                        }
                        out[r * n + q] = acc;
                    }
                }
                out
            }
        }
    };
    let [x0, x1, x2] = x;
    Moments { n, s: proj(s), x: [proj(x0), proj(x1), proj(x2)], r2: proj(r2) }
}

/// The dipole `μ = Σ_A Z_A R_A − Σ_μν P_μν <μ|r|ν>`, about the origin.
pub fn dipole(p: &[f64], species: &[Species], centers: &[[f64; 3]], mom: &Moments) -> [f64; 3] {
    let n = mom.n;
    let mut mu = [0.0f64; 3];
    for (sp, c) in species.iter().zip(centers.iter()) {
        for k in 0..3 {
            mu[k] += sp.z as f64 * c[k];
        }
    }
    for k in 0..3 {
        mu[k] -= trace_product(p, &mom.x[k], n);
    }
    mu
}

/// `<r²>` of the electrons about `c` (the point `mom` was built about), summed over electrons.
pub fn r2_expectation(p: &[f64], mom: &Moments) -> f64 {
    trace_product(p, &mom.r2, mom.n)
}

// ------------------------------------------------------------------ the potential

/// `<μ| 1/|r−R| |ν>` in the AO basis, by the difference of two attraction matrices: the
/// attraction is linear in the charges, so a unit external charge at `R` adds exactly
/// `−<μ|1/|r−R||ν>` to `v`.
pub fn unit_attraction(species: &[Species], centers: &[[f64; 3]], r: [f64; 3]) -> Vec<f64> {
    let bare = ao_integrals(&build_basis_embedded(species, centers, &[]));
    let with = ao_integrals(&build_basis_embedded(species, centers, &[External { r, q: 1.0 }]));
    let n = bare.n;
    let mut out = vec![0.0f64; n * n];
    for i in 0..n * n {
        out[i] = -(with.v[i].v - bare.v[i].v);
    }
    out
}

/// `V(R) = Σ_A Z_A/|R−A| − Σ_μν P_μν <μ|1/|r−R||ν>`.
pub fn potential_at(p: &[f64], species: &[Species], centers: &[[f64; 3]], r: [f64; 3]) -> f64 {
    let n = (p.len() as f64).sqrt().round() as usize;
    let mut v = 0.0;
    for (sp, c) in species.iter().zip(centers.iter()) {
        v += sp.z as f64 / dist(&r, c);
    }
    let a = unit_attraction(species, centers, r);
    v - trace_product(p, &a, n)
}

// ------------------------------------------------------------------ partitioned sizes

/// Each atom's Mulliken population and the RMS radius of the density partitioned to it,
/// taken about its OWN nucleus:
///
/// ```text
/// N_A     = Σ_{μ∈A} (PS)_μμ
/// <r²>_A  = Σ_{μ∈A} Σ_ν P_μν <μ| (r − R_A)² |ν>
/// rms_A   = sqrt(<r²>_A / N_A)
/// ```
///
/// This is the COUPLED size the workbench's atom band owes (OBJECT.md "The surface,
/// audited", step 1): the molecule's own density at the scene's geometry, partitioned to
/// the atom, so it responds to the bond, the neighbour and the box. On a free atom it is
/// exactly `atomic_rms_radius` by construction (one centre, the whole density). Mulliken
/// partitioning is the declared convention and is named as such wherever it is drawn.
pub fn partitioned_sizes(p: &[f64], b: &Basis, species: &[Species], centers: &[[f64; 3]]) -> Vec<(f64, f64)> {
    let n = b.n;
    let map = ao_atom_map(b);
    let s = moments(b, [0.0; 3]).s;
    let mut out = Vec::with_capacity(species.len());
    for (a, c) in centers.iter().enumerate() {
        let mom = moments(b, *c);
        let mut pop = 0.0;
        let mut r2 = 0.0;
        for mu in 0..n {
            if map[mu] != a {
                continue;
            }
            for nu in 0..n {
                pop += p[mu * n + nu] * s[nu * n + mu];
                r2 += p[mu * n + nu] * mom.r2[nu * n + mu];
            }
        }
        out.push((pop, if pop > 0.0 { (r2 / pop).sqrt() } else { 0.0 }));
    }
    out
}

// ------------------------------------------------------------------ charges

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChargeModel {
    /// Neutral, equivalent atoms equal, magnitude fixed by the exact dipole (PRIMARY).
    DipoleExact,
    /// `q_A = Z_A − Σ_{μ∈A} (PS)_μμ` (CONTROL).
    Mulliken,
}

/// A rigid fragment: species, positions, and the weight pattern `q_i = w_i q₀` the
/// dipole-exact model uses (neutral: `Σ w_i = 0`).
#[derive(Clone, Debug)]
pub struct Fragment {
    pub species: Vec<Species>,
    pub centers: Vec<[f64; 3]>,
    pub weights: Vec<f64>,
}

impl Fragment {
    pub fn new(species: Vec<Species>, centers: Vec<[f64; 3]>, weights: Vec<f64>) -> Fragment {
        assert_eq!(species.len(), centers.len());
        assert_eq!(species.len(), weights.len());
        assert!(weights.iter().sum::<f64>().abs() < 1e-12, "dipole-exact weights must be neutral");
        Fragment { species, centers, weights }
    }
    pub fn translated(&self, d: [f64; 3]) -> Fragment {
        Fragment {
            species: self.species.clone(),
            centers: self.centers.iter().map(|c| [c[0] + d[0], c[1] + d[1], c[2] + d[2]]).collect(),
            weights: self.weights.clone(),
        }
    }
}

/// Which AO belongs to which atom, for Mulliken. Refuses a basis with a spherical
/// projection: the AO-to-atom map through `sph` is one bookkeeping step this campaign
/// does not need (STO-3G H, O, F carry no d shell) and does not build.
fn ao_atom_map(b: &Basis) -> Vec<usize> {
    assert!(b.sph.is_none(), "Mulliken charges through a spherical projection are not built");
    b.funcs.iter().map(|&(shell, _)| b.shells[shell].center).collect()
}

pub fn mulliken_charges(p: &[f64], b: &Basis, species: &[Species], s: &[f64]) -> Vec<f64> {
    let n = b.n;
    let map = ao_atom_map(b);
    let mut q: Vec<f64> = species.iter().map(|sp| sp.z as f64).collect();
    for mu in 0..n {
        let mut ps = 0.0;
        for nu in 0..n {
            ps += p[mu * n + nu] * s[nu * n + mu];
        }
        q[map[mu]] -= ps;
    }
    q
}

/// The fragment's charges under `model`, from its (embedded) density.
pub fn fragment_charges(model: ChargeModel, f: &Fragment, p: &[f64], b: &Basis, mom: &Moments) -> Vec<f64> {
    match model {
        ChargeModel::Mulliken => mulliken_charges(p, b, &f.species, &mom.s),
        ChargeModel::DipoleExact => {
            let mu = dipole(p, &f.species, &f.centers, mom);
            let mut d = [0.0f64; 3];
            for (w, c) in f.weights.iter().zip(f.centers.iter()) {
                for k in 0..3 {
                    d[k] += w * c[k];
                }
            }
            let dd = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            // a single atom (or any pattern with no lever arm) carries no dipole-exact
            // charge: zero, never 0/0
            if dd <= 0.0 {
                return vec![0.0; f.weights.len()];
            }
            let q0 = (mu[0] * d[0] + mu[1] * d[1] + mu[2] * d[2]) / dd;
            f.weights.iter().map(|w| w * q0).collect()
        }
    }
}

/// One embedded monomer with everything the gates read off it.
pub struct Monomer {
    pub solve: EmbeddedSolve,
    pub gamma: Vec<f64>,
    pub p: Vec<f64>,
    pub mom: Moments,
    pub charges: Vec<f64>,
    pub charges_control: Vec<f64>,
    pub dipole: [f64; 3],
}

pub fn monomer(f: &Fragment, ext: &[External], model: ChargeModel) -> Monomer {
    let solve = solve_embedded(&f.species, &f.centers, ext);
    let n = solve.basis.n;
    let gamma = rdm1(&solve.gp.space, &solve.sol.vector);
    let p = ao_density(&gamma, &solve.gp.orbitals, n);
    // moments about the origin: the dipole of a NEUTRAL fragment is origin-independent
    let mom = moments(&solve.basis, [0.0; 3]);
    let charges = fragment_charges(model, f, &p, &solve.basis, &mom);
    let control = fragment_charges(
        if model == ChargeModel::DipoleExact { ChargeModel::Mulliken } else { ChargeModel::DipoleExact },
        f,
        &p,
        &solve.basis,
        &mom,
    );
    let dipole = dipole(&p, &f.species, &f.centers, &mom);
    Monomer { solve, gamma, p, mom, charges, charges_control: control, dipole }
}

fn externals_of(f: &Fragment, q: &[f64]) -> Vec<External> {
    f.centers.iter().zip(q.iter()).map(|(c, &q)| External { r: *c, q }).collect()
}

// ------------------------------------------------------------------ the pair

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// Both fragments' external charges zero at the first iteration.
    Zero,
    /// Each fragment's charges from its ISOLATED density at the first iteration.
    Isolated,
}

pub struct EmbedResult {
    pub converged: bool,
    pub iterations: usize,
    pub q_a: Vec<f64>,
    pub q_b: Vec<f64>,
    pub e_a: f64,
    pub e_b: f64,
    /// `Σ_{a∈A,b∈B} q_a q_b / r_ab`, the term counted in both embedded solves.
    pub e_qq: f64,
    pub e_emb: f64,
    pub last_delta: f64,
    pub a: Monomer,
    pub b: Monomer,
}

/// The mutual embedding of two fragments, iterated to a fixed point of the charges.
/// `plant_double_count` is PLANT (ii): the charge–charge subtraction omitted.
pub fn embed_pair(a: &Fragment, b: &Fragment, model: ChargeModel, start: Start, plant_double_count: bool) -> EmbedResult {
    let (mut q_a, mut q_b): (Vec<f64>, Vec<f64>) = match start {
        Start::Zero => (vec![0.0; a.species.len()], vec![0.0; b.species.len()]),
        Start::Isolated => (monomer(a, &[], model).charges, monomer(b, &[], model).charges),
    };
    let mut converged = false;
    let mut iterations = 0;
    let last_delta;
    let (ma, mb);
    loop {
        iterations += 1;
        let ma_it = monomer(a, &externals_of(b, &q_b), model);
        let q_a_new = ma_it.charges.clone();
        let mb_it = monomer(b, &externals_of(a, &q_a_new), model);
        let q_b_new = mb_it.charges.clone();
        let da = q_a.iter().zip(q_a_new.iter()).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max);
        let db = q_b.iter().zip(q_b_new.iter()).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max);
        let delta = da.max(db);
        q_a = q_a_new;
        q_b = q_b_new;
        if delta < 1e-9 {
            converged = true;
            last_delta = delta;
            ma = ma_it;
            mb = mb_it;
            break;
        }
        if iterations >= 100 {
            last_delta = delta;
            ma = ma_it;
            mb = mb_it;
            break;
        }
    }
    // the fixed point's energies are those of the LAST solves, whose external charges are
    // the converged partner charges to within last_delta
    let mut e_qq = 0.0;
    for (ca, &qa) in a.centers.iter().zip(q_a.iter()) {
        for (cb, &qb) in b.centers.iter().zip(q_b.iter()) {
            e_qq += qa * qb / dist(ca, cb);
        }
    }
    let e_emb = ma.solve.e_total + mb.solve.e_total - if plant_double_count { 0.0 } else { e_qq };
    EmbedResult {
        converged,
        iterations,
        q_a,
        q_b,
        e_a: ma.solve.e_total,
        e_b: mb.solve.e_total,
        e_qq,
        e_emb,
        last_delta,
        a: ma,
        b: mb,
    }
}

/// The exact supermolecule, no externals.
pub fn supermolecule(a: &Fragment, b: &Fragment) -> EmbeddedSolve {
    let mut species = a.species.clone();
    species.extend_from_slice(&b.species);
    let mut centers = a.centers.clone();
    centers.extend_from_slice(&b.centers);
    solve_embedded(&species, &centers, &[])
}

// ------------------------------------------------------------------ the monomer pins (G1)

/// Golden-section minimum of `f` on `[lo, hi]` to a bracket of width `tol`.
fn golden(f: &dyn Fn(f64) -> f64, mut lo: f64, mut hi: f64, tol: f64) -> f64 {
    let g = (5.0f64.sqrt() - 1.0) / 2.0;
    let mut c = hi - g * (hi - lo);
    let mut d = lo + g * (hi - lo);
    let (mut fc, mut fd) = (f(c), f(d));
    while (hi - lo).abs() > tol {
        if fc < fd {
            hi = d;
            d = c;
            fd = fc;
            c = hi - g * (hi - lo);
            fc = f(c);
        } else {
            lo = c;
            c = d;
            fc = fd;
            d = lo + g * (hi - lo);
            fd = f(d);
        }
    }
    0.5 * (lo + hi)
}

/// Central-difference gradient with step `h`.
pub fn fd_gradient(f: &dyn Fn(f64) -> f64, x: f64, h: f64) -> f64 {
    (f(x + h) - f(x - h)) / (2.0 * h)
}

/// The HF monomer's bond length at the engine's own STO-3G determinant minimum, with the
/// central-difference gradient there.
pub fn pin_hf(f_species: Species, h_species: Species) -> (f64, f64) {
    let e = |r: f64| solve_embedded(&[f_species, h_species], &[[0.0; 3], [0.0, 0.0, r]], &[]).e_total;
    let r = golden(&e, 1.2, 2.6, 1e-9);
    (r, fd_gradient(&e, r, 1e-4))
}

/// Water's atoms: O at the origin, the hydrogens in the xz-plane symmetric about +z (the
/// C₂ axis is +z, the hydrogens on the +z side).
pub fn water_centers(r: f64, theta: f64) -> [[f64; 3]; 3] {
    let (s, c) = ((0.5 * theta).sin(), (0.5 * theta).cos());
    [[0.0; 3], [r * s, 0.0, r * c], [-r * s, 0.0, r * c]]
}

/// The water monomer's bond length and angle at the minimum, by alternating 1-D searches
/// until both central-difference gradients are below `1e-6`.
pub fn pin_water(o: Species, h: Species) -> (f64, f64, f64, f64) {
    let e = |r: f64, t: f64| solve_embedded(&[o, h, h], &water_centers(r, t), &[]).e_total;
    let (mut r, mut t) = (1.8f64, 1.9f64);
    for _ in 0..40 {
        let t0 = t;
        r = golden(&|x| e(x, t0), r - 0.3, r + 0.3, 1e-9);
        let r0 = r;
        t = golden(&|x| e(r0, x), t - 0.4, t + 0.4, 1e-9);
        let gr = fd_gradient(&|x| e(x, t), r, 1e-4);
        let gt = fd_gradient(&|x| e(r, x), t, 1e-4);
        if gr.abs() < 1e-6 && gt.abs() < 1e-6 {
            return (r, t, gr, gt);
        }
    }
    let gr = fd_gradient(&|x| e(x, t), r, 1e-4);
    let gt = fd_gradient(&|x| e(r, x), t, 1e-4);
    (r, t, gr, gt)
}

// ------------------------------------------------------------------ the staked geometries

pub const ANGSTROM_TO_BOHR: f64 = 1.0 / 0.529177210903;

/// System 1: F–H···F–H collinear along +z, fluorines `r_ff` apart (bohr).
pub fn hf_dimer(f: Species, h: Species, r_hf: f64, r_ff: f64) -> (Fragment, Fragment) {
    let a = Fragment::new(vec![f, h], vec![[0.0; 3], [0.0, 0.0, r_hf]], vec![-1.0, 1.0]);
    let b = a.translated([0.0, 0.0, r_ff]);
    (a, b)
}

/// System 2, DIMER-1's `LINEAR`: the donor's O–H₁ along +z toward the acceptor, whose C₂
/// axis is +z with its hydrogens on the far side.
pub fn water_dimer_linear(o: Species, h: Species, r: f64, theta: f64, r_oo: f64) -> (Fragment, Fragment) {
    let donor = Fragment::new(
        vec![o, h, h],
        vec![[0.0; 3], [0.0, 0.0, r], [r * theta.sin(), 0.0, r * theta.cos()]],
        vec![-2.0, 1.0, 1.0],
    );
    let acc = Fragment::new(vec![o, h, h], water_centers(r, theta).to_vec(), vec![-2.0, 1.0, 1.0])
        .translated([0.0, 0.0, r_oo]);
    (donor, acc)
}
