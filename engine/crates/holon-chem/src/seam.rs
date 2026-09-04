//! The seam — SEAM-1 (`conformance/water_observatory/SEAM_PREREG.md`).
//!
//! Exact cores solved INSIDE the embedding field of `embed.rs`, for any number of
//! fragments: the mutual self-consistent embedding (`embed_many`), the embedded pairwise
//! expansion (`ee_pa` — Dahlke and Truhlar's EE-PA: every monomer in the field of all the
//! others, every dimer in the field of the rest), the bare pairwise expansion (`bare_pa`),
//! and the exact supermolecule of them all. The difference between the exact energy and
//! `bare_pa` is the bare three-body term the dE5 audit found un-terminable; the
//! difference between the exact energy and `ee_pa` is what the field does not carry.
//!
//! Every energy is the fragment system's own energy in its field — electrons and nuclei
//! with the charges, never a charge–charge term (`solve_embedded` subtracts the external
//! self-energy) — so the QM–charge terms cancel pairwise between the dimer and monomer
//! sums and `ee_pa` differs from `bare_pa` by exactly the field's polarisation of each
//! solve. With every charge zero the two are identical (gate G3).

use crate::elements::Species;
use crate::embed::{monomer, solve_embedded, ChargeModel, EmbeddedSolve, External, Fragment, Monomer, Start};

/// The mutual embedding of `N` fragments at its fixed point.
pub struct ManyResult {
    pub converged: bool,
    pub iterations: usize,
    pub last_delta: f64,
    pub charges: Vec<Vec<f64>>,
    /// The last sweep's embedded monomers (their fields within `last_delta` of the fixed point).
    pub monomers: Vec<Monomer>,
}

/// The charges of every fragment except those in `except`, as externals.
pub fn externals_except(frags: &[Fragment], charges: &[Vec<f64>], except: &[usize]) -> Vec<External> {
    let mut out = Vec::new();
    for (j, f) in frags.iter().enumerate() {
        if except.contains(&j) {
            continue;
        }
        for (c, &q) in f.centers.iter().zip(charges[j].iter()) {
            out.push(External { r: *c, q });
        }
    }
    out
}

/// Every fragment solved in the field of all the others' charges, swept Gauss–Seidel
/// (each fragment sees the latest charges of those already updated this sweep), until the
/// largest charge change is below `1e-9`; at most 100 sweeps. `embed_pair` is `N = 2`.
pub fn embed_many(frags: &[Fragment], model: ChargeModel, start: Start) -> ManyResult {
    let n = frags.len();
    let mut q: Vec<Vec<f64>> = match start {
        Start::Zero => frags.iter().map(|f| vec![0.0; f.species.len()]).collect(),
        Start::Isolated => frags.iter().map(|f| monomer(f, &[], model).charges).collect(),
    };
    let mut iterations = 0;
    let mut converged = false;
    let mut last_delta = f64::INFINITY;
    let mut monomers = Vec::new();
    loop {
        iterations += 1;
        let mut delta = 0.0f64;
        let mut swept = Vec::with_capacity(n);
        for i in 0..n {
            let ext = externals_except(frags, &q, &[i]);
            let m = monomer(&frags[i], &ext, model);
            let d = m.charges.iter().zip(q[i].iter()).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
            delta = delta.max(d);
            q[i] = m.charges.clone();
            swept.push(m);
        }
        monomers = swept;
        last_delta = delta;
        if delta < 1e-9 {
            converged = true;
            break;
        }
        if iterations >= 100 {
            break;
        }
    }
    ManyResult { converged, iterations, last_delta, charges: q, monomers }
}

/// Two fragments as one, for a dimer solve.
pub fn joined(a: &Fragment, b: &Fragment) -> (Vec<Species>, Vec<[f64; 3]>) {
    let mut species = a.species.clone();
    species.extend_from_slice(&b.species);
    let mut centers = a.centers.clone();
    centers.extend_from_slice(&b.centers);
    (species, centers)
}

/// One pairwise expansion's record: the monomer energies, the dimer energies, the sum.
pub struct PaResult {
    pub e_mono: Vec<f64>,
    /// `(i, j, E_ij)` for every pair `i < j`.
    pub e_dimer: Vec<(usize, usize, f64)>,
    pub total: f64,
}

/// `Σ_{i<j} E_ij[field of the rest] − (N−2) Σ_i E_i[field of the others]`.
/// With `dimers_in_field == false` this is PLANT (i): the dimers solved in vacuum while
/// the monomers stay embedded.
pub fn ee_pa_with(frags: &[Fragment], charges: &[Vec<f64>], dimers_in_field: bool) -> PaResult {
    let n = frags.len();
    let e_mono: Vec<f64> = (0..n)
        .map(|i| solve_embedded(&frags[i].species, &frags[i].centers, &externals_except(frags, charges, &[i])).e_total)
        .collect();
    let mut e_dimer = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let (sp, ce) = joined(&frags[i], &frags[j]);
            let ext = if dimers_in_field { externals_except(frags, charges, &[i, j]) } else { Vec::new() };
            e_dimer.push((i, j, solve_embedded(&sp, &ce, &ext).e_total));
        }
    }
    let total = e_dimer.iter().map(|d| d.2).sum::<f64>() - (n as f64 - 2.0) * e_mono.iter().sum::<f64>();
    PaResult { e_mono, e_dimer, total }
}

pub fn ee_pa(frags: &[Fragment], charges: &[Vec<f64>]) -> PaResult {
    ee_pa_with(frags, charges, true)
}

/// The bare pairwise expansion: every solve in vacuum.
pub fn bare_pa(frags: &[Fragment]) -> PaResult {
    let zeros: Vec<Vec<f64>> = frags.iter().map(|f| vec![0.0; f.species.len()]).collect();
    // zero charges at extra centres change no integral; this is the identity G3 measures
    let n = frags.len();
    let e_mono: Vec<f64> = (0..n).map(|i| solve_embedded(&frags[i].species, &frags[i].centers, &[]).e_total).collect();
    let mut e_dimer = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let (sp, ce) = joined(&frags[i], &frags[j]);
            e_dimer.push((i, j, solve_embedded(&sp, &ce, &[]).e_total));
        }
    }
    let _ = zeros;
    let total = e_dimer.iter().map(|d| d.2).sum::<f64>() - (n as f64 - 2.0) * e_mono.iter().sum::<f64>();
    PaResult { e_mono, e_dimer, total }
}

/// The exact supermolecule of every fragment, no field.
pub fn supermolecule_all(frags: &[Fragment]) -> EmbeddedSolve {
    let mut species = Vec::new();
    let mut centers = Vec::new();
    for f in frags {
        species.extend_from_slice(&f.species);
        centers.extend_from_slice(&f.centers);
    }
    solve_embedded(&species, &centers, &[])
}

/// SEAM-1's carrier: `n` HF monomers on a line, F–H···F–H···, fluorines `r_ff` apart.
pub fn hf_chain(f: Species, h: Species, r_hf: f64, r_ff: f64, n: usize) -> Vec<Fragment> {
    let a = Fragment::new(vec![f, h], vec![[0.0; 3], [0.0, 0.0, r_hf]], vec![-1.0, 1.0]);
    (0..n).map(|k| a.translated([0.0, 0.0, k as f64 * r_ff])).collect()
}
