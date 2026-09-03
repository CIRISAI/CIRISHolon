//! U(1)^k-symmetric two-site DMRG on CHARGE-LABELLED bonds — GF2a amendment A1 §A1.2 (E7).
//!
//! The same integer lanes the exact colour-lane referee runs on, in the MPS: every bond index
//! carries a charge label (the conserved counts to its left), every site declares what an
//! occupied mode adds, and the sweep never leaves the sector the boundary labels fix. Three
//! changes to the dense two-site update and no fourth:
//!
//! 1. no penalty term — the MPO is the Hamiltonian;
//! 2. the local solve is MASKED to the label-consistent entries of the two-site wavefunction
//!    after every matvec, so roundoff cannot leak charge across a sweep;
//! 3. the SVD is BLOCKWISE by the cut bond's charge: rows `(l, a)` and columns `(b, r)` are
//!    grouped by charge, each block decomposed alone, the kept singular values the largest
//!    `χ` across blocks, every kept bond state inheriting its block's label. `χ` counts
//!    in-sector states only.
//!
//! The retired arm (`dmrg::dmrg_sweep` on the penalised MPO) stuck in a penalty-sector
//! metastable state at N = 8 (`GF2A_QCD2_RESULTS.md`). Plant (iv) of the amendment is the
//! `ignore_labels` mutant here: the same sweep with the labels switched off in the mask and
//! the SVD, which is that arm without its penalty and is expected to wander.
//!
//! Convergence is the amendment's (A1.3), gated as stated: energy change `≤ rtol·max(1,|E|)`
//! AND the last sweep's maximum discarded weight `≤ max_discarded` AND at least `min_sweeps`.
//! The labels travel beside the tensors as one vector per bond; `TensorSite` is untouched.

use crate::dmrg::DmrgResult;
use crate::lanczos;
use crate::mpo::Mpo;
use crate::mps::{self, Env, TensorSite};

/// The most conserved counts a label carries; a colour sector uses three.
pub const MAX_CHARGES: usize = 4;
pub type Charge = [i32; MAX_CHARGES];
pub const ZERO_CHARGE: Charge = [0; MAX_CHARGES];
/// A label no physical bond state carries: the mutant's bonds wear it.
pub const NO_CHARGE: Charge = [i32::MIN; MAX_CHARGES];

pub fn charge_add(a: Charge, b: Charge) -> Charge {
    let mut c = a;
    for i in 0..MAX_CHARGES {
        c[i] = a[i].saturating_add(b[i]);
    }
    c
}

pub fn charge_sub(a: Charge, b: Charge) -> Charge {
    let mut c = a;
    for i in 0..MAX_CHARGES {
        c[i] = a[i].saturating_sub(b[i]);
    }
    c
}

/// The sector: what an occupied site adds, and the total the right boundary must carry.
#[derive(Clone, Debug)]
pub struct Sector {
    pub site_charge: Vec<Charge>,
    pub total: Charge,
}

/// The labels of every bond, boundaries included: `labels[j]` has one entry per state of
/// bond `j`; `labels[0] = [ZERO_CHARGE]`, `labels[l] = [total]`.
pub type Labels = Vec<Vec<Charge>>;

#[derive(Debug, Clone, PartialEq)]
pub enum SymRefusal {
    /// The start state's total charge is not the sector's: refused BY NAME, never solved to a
    /// number in the wrong block (amendment plant (v)).
    StartOutsideSector { start: Charge, total: Charge },
    /// A cut bond had no label-consistent block at all.
    EmptyCut { bond: usize },
    /// The local Lanczos did not converge.
    LanczosFailed { bond: usize },
}

impl std::fmt::Display for SymRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymRefusal::StartOutsideSector { start, total } => write!(
                f,
                "the start state carries charge {start:?} and the sector's boundary carries {total:?}: \
                 the symmetric sweep cannot leave its block and will not pretend to"
            ),
            SymRefusal::EmptyCut { bond } => write!(f, "bond {bond}: no label-consistent block at the cut"),
            SymRefusal::LanczosFailed { bond } => write!(f, "bond {bond}: the local Lanczos did not converge"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SymConfig {
    pub chi_max: usize,
    pub max_sweeps: usize,
    /// Relative energy tolerance between successive sweeps (A1.3 (a)).
    pub rtol: f64,
    /// The last sweep's maximum discarded weight must be at most this (A1.3 (b)).
    pub max_discarded: f64,
    /// At least this many sweeps before "converged" can be said (A1.3 (c)).
    pub min_sweeps: usize,
    /// Plant (iv): labels ignored in the mask and the SVD — the retired arm without its penalty.
    pub ignore_labels: bool,
}

impl SymConfig {
    /// The amendment's stated convergence test at bond dimension `chi_max`.
    pub fn amendment(chi_max: usize, max_sweeps: usize) -> SymConfig {
        SymConfig { chi_max, max_sweeps, rtol: 1e-10, max_discarded: 1e-8, min_sweeps: 4, ignore_labels: false }
    }
}

/// Bond labels of a product state: `labels[j]` is the charge left of site `j`.
pub fn labels_of_product(occ: &[bool], sector: &Sector) -> Labels {
    let mut q = Vec::with_capacity(occ.len() + 1);
    let mut acc = ZERO_CHARGE;
    q.push(vec![acc]);
    for (j, &o) in occ.iter().enumerate() {
        if o {
            acc = charge_add(acc, sector.site_charge[j]);
        }
        q.push(vec![acc]);
    }
    q
}

/// A seeded RANDOM labelled start: every bond carries one state per reachable charge (the
/// counts a left part can hold, intersected with what the right part must still supply), up
/// to `chi0` of them, and every label-consistent tensor entry is a deterministic pseudo-random
/// number. `chi0` counts LABELS, and it must cover every sector that carries weight: a
/// labelled two-site update can only produce labels that combine its two neighbouring bonds'
/// existing ones, so a sector absent from both neighbours can never appear later (measured
/// 2026-09-03: eight labels per bond left N = 4 at −23.48 against −24.54 exact, converged and
/// wrong). The reachable count at the middle bond is `(n_c + 1)^3` — 216 at N = 10 — so 256
/// covers every N ≤ 10 exactly; beyond that the labels closest to the proportional fill are
/// kept, which is a stated truncation of far sectors, not an assumption about them. Why not the product start: its energy is exactly zero on full colour-singlet
/// sites, and a labelled two-site block cannot change its own charge while this chain's
/// hopping spans THREE sites, so with a product environment every local update is a fixed
/// point (measured 2026-09-03: the labelled sweep returned 0.000000 at N = 4 from the product
/// start). The retired arm only moved by changing local quark number — the leak its penalty
/// fought. A superposed environment carries the hopping channels, and the sweep proceeds.
pub fn random_start(sector: &Sector, chi0: usize, seed: u64) -> (Vec<TensorSite>, Labels) {
    let l = sector.site_charge.len();
    let k = MAX_CHARGES;
    // capacity of each charge component to the left of bond j and to its right
    let mut left_cap = vec![ZERO_CHARGE; l + 1];
    for j in 0..l {
        left_cap[j + 1] = charge_add(left_cap[j], sector.site_charge[j]);
    }
    let total_cap = left_cap[l];
    let mut labels: Labels = Vec::with_capacity(l + 1);
    for j in 0..=l {
        let right_cap = charge_sub(total_cap, left_cap[j]);
        // ranges per component: max(0, total − right) ..= min(total, left)
        let mut lo = ZERO_CHARGE;
        let mut hi = ZERO_CHARGE;
        for c in 0..k {
            lo[c] = (sector.total[c] - right_cap[c]).max(0);
            hi[c] = sector.total[c].min(left_cap[j][c]);
        }
        let mut set: Vec<Charge> = vec![ZERO_CHARGE];
        for c in 0..k {
            let mut next = Vec::new();
            for q in &set {
                for v in lo[c]..=hi[c] {
                    let mut q2 = *q;
                    q2[c] = v;
                    next.push(q2);
                }
            }
            set = next;
        }
        // deterministic: closest to the proportional fill first, ties by value
        let frac = j as f64 / l.max(1) as f64;
        set.sort_by(|a, b| {
            let da: f64 = (0..k).map(|c| (a[c] as f64 - frac * sector.total[c] as f64).powi(2)).sum();
            let db: f64 = (0..k).map(|c| (b[c] as f64 - frac * sector.total[c] as f64).powi(2)).sum();
            da.partial_cmp(&db).unwrap().then(a.cmp(b))
        });
        set.truncate(chi0.max(1));
        labels.push(set);
    }
    let mut st = seed;
    let mut rnd = || {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((st >> 11) as f64) / ((1u64 << 53) as f64) - 0.5
    };
    let mut tensors = Vec::with_capacity(l);
    for j in 0..l {
        let (ql, qr) = (&labels[j], &labels[j + 1]);
        let mut t = TensorSite::zeros(ql.len(), qr.len());
        for (li, &a_q) in ql.iter().enumerate() {
            for a in 0..2 {
                let q_after = if a == 1 { charge_add(a_q, sector.site_charge[j]) } else { a_q };
                for (ri, &b_q) in qr.iter().enumerate() {
                    if b_q == q_after {
                        t.set(a, li, ri, rnd());
                    }
                }
            }
        }
        tensors.push(t);
    }
    (tensors, labels)
}

/// The occupations of a `χ = 1` product state.
pub fn occupations_of_product(tensors: &[TensorSite]) -> Vec<bool> {
    tensors.iter().map(|t| t.chi_l == 1 && t.chi_r == 1 && t.get(1, 0, 0) != 0.0).collect()
}

/// The label-consistency mask of a two-site wavefunction: `q_l[l] + a·e₁ + b·e₂ == q_r[r]`.
fn mask(q_l: &[Charge], q_r: &[Charge], e1: Charge, e2: Charge) -> Vec<bool> {
    let chi_r = q_r.len();
    let mut m = vec![false; q_l.len() * 4 * chi_r];
    for (l, &ql) in q_l.iter().enumerate() {
        for a in 0..2 {
            let qa = if a == 1 { charge_add(ql, e1) } else { ql };
            for b in 0..2 {
                let qab = if b == 1 { charge_add(qa, e2) } else { qa };
                for (r, &qr) in q_r.iter().enumerate() {
                    m[((l * 2 + a) * 2 + b) * chi_r + r] = qab == qr;
                }
            }
        }
    }
    m
}

/// The blockwise split (A1.2 item 3): the two site tensors, the discarded weight, the
/// kept-spectrum floor, and the new bond's labels. `None` when no label-consistent block
/// carries weight or a block's SVD did not converge.
pub fn split_two_site_sym(
    psi: &[f64],
    q_l: &[Charge],
    q_r: &[Charge],
    e1: Charge,
    e2: Charge,
    chi_max: usize,
    absorb_s_left: bool,
) -> Option<(TensorSite, TensorSite, f64, f64, Vec<Charge>)> {
    let (chi_l, chi_r) = (q_l.len(), q_r.len());
    let mut row_charge: Vec<(Charge, usize)> = Vec::with_capacity(chi_l * 2);
    for (l, &ql) in q_l.iter().enumerate() {
        for a in 0..2 {
            row_charge.push((if a == 1 { charge_add(ql, e1) } else { ql }, l * 2 + a));
        }
    }
    let mut col_charge: Vec<(Charge, usize)> = Vec::with_capacity(2 * chi_r);
    for b in 0..2 {
        for (r, &qr) in q_r.iter().enumerate() {
            col_charge.push((if b == 1 { charge_sub(qr, e2) } else { qr }, b * chi_r + r));
        }
    }
    let mut charges: Vec<Charge> = row_charge.iter().map(|c| c.0).collect();
    charges.sort();
    charges.dedup();
    // (σ, charge, left column over ALL rows, right row over ALL columns)
    let mut triples: Vec<(f64, Charge, Vec<f64>, Vec<f64>)> = Vec::new();
    for &c in &charges {
        if c == NO_CHARGE {
            continue;
        }
        let rows: Vec<usize> = row_charge.iter().filter(|x| x.0 == c).map(|x| x.1).collect();
        let cols: Vec<usize> = col_charge.iter().filter(|x| x.0 == c).map(|x| x.1).collect();
        if rows.is_empty() || cols.is_empty() {
            continue;
        }
        let (m, n) = (rows.len(), cols.len());
        let mut block = vec![0.0; m * n];
        for (i, &ri) in rows.iter().enumerate() {
            for (j, &cj) in cols.iter().enumerate() {
                block[i * n + j] = psi[ri * 2 * chi_r + cj];
            }
        }
        if block.iter().all(|v| *v == 0.0) {
            continue;
        }
        let svd = crate::svd::jacobi_svd(&block, m, n);
        if !svd.converged {
            return None;
        }
        for (k, &s) in svd.s.iter().enumerate() {
            if s == 0.0 {
                continue;
            }
            let mut u_full = vec![0.0; chi_l * 2];
            for (i, &ri) in rows.iter().enumerate() {
                u_full[ri] = svd.u[k][i];
            }
            let mut v_full = vec![0.0; 2 * chi_r];
            for (j, &cj) in cols.iter().enumerate() {
                v_full[cj] = svd.v[k][j];
            }
            triples.push((s, c, u_full, v_full));
        }
    }
    if triples.is_empty() {
        return None;
    }
    triples.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let chi_new = chi_max.min(triples.len()).max(1);
    let discarded: f64 = triples[chi_new..].iter().map(|t| t.0 * t.0).sum();
    let kept = &triples[..chi_new];
    let s_max = kept.iter().map(|t| t.0).fold(0.0f64, f64::max);
    let s_min = kept.iter().map(|t| t.0).fold(f64::INFINITY, f64::min);
    let floor = if s_max > 0.0 { s_min / s_max } else { 0.0 };
    let mut a_left = TensorSite::zeros(chi_l, chi_new);
    let mut a_right = TensorSite::zeros(chi_new, chi_r);
    let mut new_q = Vec::with_capacity(chi_new);
    for (i, (s, c, u, v)) in kept.iter().enumerate() {
        let (fl, fr) = if absorb_s_left { (*s, 1.0) } else { (1.0, *s) };
        for l in 0..chi_l {
            for a in 0..2 {
                a_left.set(a, l, i, u[l * 2 + a] * fl);
            }
        }
        for b in 0..2 {
            for r in 0..chi_r {
                a_right.set(b, i, r, v[b * chi_r + r] * fr);
            }
        }
        new_q.push(*c);
    }
    Some((a_left, a_right, discarded, floor, new_q))
}

/// One two-site update at bond `j`: `(energy, discarded, residual, floor, iterations, new labels)`.
#[allow(clippy::too_many_arguments)]
fn update(
    tensors: &mut [TensorSite],
    q_l: &[Charge],
    q_r: &[Charge],
    sector: &Sector,
    mpo: &Mpo,
    j: usize,
    cfg: &SymConfig,
    absorb_s_left: bool,
    left_env: &Env,
    right_env: &Env,
) -> Result<(f64, f64, f64, f64, usize, Vec<Charge>), SymRefusal> {
    let (w1, w2) = (&mpo.sites[j], &mpo.sites[j + 1]);
    let chi_l = tensors[j].chi_l;
    let chi_r = tensors[j + 1].chi_r;
    debug_assert_eq!(q_l.len(), chi_l);
    debug_assert_eq!(q_r.len(), chi_r);
    let mid = tensors[j].chi_r;
    let dim = chi_l * 2 * 2 * chi_r;
    let mut seed = vec![0.0; dim];
    for lft in 0..chi_l {
        for a in 0..2 {
            for m in 0..mid {
                let av = tensors[j].get(a, lft, m);
                if av == 0.0 {
                    continue;
                }
                let base = (lft * 2 + a) * 2 * chi_r;
                for b in 0..2 {
                    for r in 0..chi_r {
                        seed[base + b * chi_r + r] += av * tensors[j + 1].get(b, m, r);
                    }
                }
            }
        }
    }
    let (e1, e2) = (sector.site_charge[j], sector.site_charge[j + 1]);
    let msk = if cfg.ignore_labels { Vec::new() } else { mask(q_l, q_r, e1, e2) };
    if !cfg.ignore_labels {
        for (v, &ok) in seed.iter_mut().zip(&msk) {
            if !ok {
                *v = 0.0;
            }
        }
        if seed.iter().all(|v| *v == 0.0) {
            return Err(SymRefusal::EmptyCut { bond: j });
        }
    }
    let (live_l, live_r) = (mps::live_channels(left_env), mps::live_channels(right_env));
    let gs = lanczos::ground_state(
        |psi| {
            let mut h = mps::apply_effective_h_mpo_live(left_env, w1, w2, right_env, psi, chi_l, chi_r, &live_l, &live_r);
            if !cfg.ignore_labels {
                for (v, &ok) in h.iter_mut().zip(&msk) {
                    if !ok {
                        *v = 0.0;
                    }
                }
            }
            h
        },
        &seed,
        dim,
    )
    .ok_or(SymRefusal::LanczosFailed { bond: j })?;
    if cfg.ignore_labels {
        let (a_left, a_right, dw, floor) = mps::split_two_site(&gs.vector, chi_l, chi_r, cfg.chi_max, absorb_s_left);
        let chi_new = a_left.chi_r;
        tensors[j] = a_left;
        tensors[j + 1] = a_right;
        return Ok((gs.energy, dw, gs.residual, floor, gs.iterations, vec![NO_CHARGE; chi_new]));
    }
    let (a_left, a_right, dw, floor, new_q) =
        split_two_site_sym(&gs.vector, q_l, q_r, e1, e2, cfg.chi_max, absorb_s_left).ok_or(SymRefusal::EmptyCut { bond: j })?;
    tensors[j] = a_left;
    tensors[j + 1] = a_right;
    Ok((gs.energy, dw, gs.residual, floor, gs.iterations, new_q))
}

fn all_right_envs(tensors: &[TensorSite], mpo: &Mpo) -> Vec<Env> {
    let l = tensors.len();
    let d_last = if l > 0 { mpo.sites[l - 1].d_r } else { 1 };
    let mut envs: Vec<Env> = vec![mps::trivial_right_env_mpo(d_last); l + 1];
    for k in (0..l).rev() {
        envs[k] = mps::grow_right_mpo(&envs[k + 1], &mpo.sites[k], &tensors[k]);
    }
    envs
}

/// The symmetric sweep. Returns the result and the final labels, so a χ-ladder continues
/// from them: raising `chi_max` on the returned state needs no padding — the two-site
/// update grows the bond itself.
pub fn dmrg_sweep_sym(
    mpo: &Mpo,
    initial: Vec<TensorSite>,
    labels: Labels,
    sector: &Sector,
    cfg: &SymConfig,
) -> Result<(DmrgResult, Labels), SymRefusal> {
    let l = mpo.sites.len();
    assert_eq!(initial.len(), l, "tensors length must match MPO length");
    assert_eq!(labels.len(), l + 1, "one label vector per bond, boundaries included");
    assert_eq!(sector.site_charge.len(), l, "one site charge per site");
    let start_total = labels[l][0];
    if !cfg.ignore_labels && start_total != sector.total {
        return Err(SymRefusal::StartOutsideSector { start: start_total, total: sector.total });
    }
    for (j, t) in initial.iter().enumerate() {
        assert_eq!(labels[j].len(), t.chi_l, "bond {j}: labels vs chi_l");
        assert_eq!(labels[j + 1].len(), t.chi_r, "bond {}: labels vs chi_r", j + 1);
    }
    let mut tensors = initial;
    let mut labels = labels;
    let mut prev_energy = f64::INFINITY;
    let mut last_energy = 0.0;
    let mut converged = false;
    let mut sweeps_used = 0;
    let mut discarded = vec![0.0; l.saturating_sub(1)];
    let mut spectrum_floor = vec![0.0; l.saturating_sub(1)];
    let mut energy_history = Vec::with_capacity(cfg.max_sweeps);
    let mut worst_resid = 0.0f64;
    let mut iters_total = 0usize;
    let mut right_envs = all_right_envs(&tensors, mpo);
    for sweep in 0..cfg.max_sweeps {
        sweeps_used = sweep + 1;
        let mut left_envs: Vec<Env> = Vec::with_capacity(l);
        left_envs.push(mps::trivial_left_env_mpo(mpo.sites[0].d_l));
        for j in 0..(l - 1) {
            let (q_l, q_r) = (labels[j].clone(), labels[j + 2].clone());
            let (e, dw, resid, sf, it, new_q) = update(&mut tensors, &q_l, &q_r, sector, mpo, j, cfg, false, &left_envs[j], &right_envs[j + 2])?;
            labels[j + 1] = new_q;
            last_energy = e;
            discarded[j] = dw;
            spectrum_floor[j] = sf;
            worst_resid = worst_resid.max(resid);
            iters_total += it;
            let grown = mps::grow_left_mpo(&left_envs[j], &mpo.sites[j], &tensors[j]);
            left_envs.push(grown);
        }
        for j in (0..(l - 1)).rev() {
            let (q_l, q_r) = (labels[j].clone(), labels[j + 2].clone());
            let (e, dw, resid, sf, it, new_q) = update(&mut tensors, &q_l, &q_r, sector, mpo, j, cfg, true, &left_envs[j], &right_envs[j + 2])?;
            labels[j + 1] = new_q;
            last_energy = e;
            discarded[j] = dw;
            spectrum_floor[j] = sf;
            worst_resid = worst_resid.max(resid);
            iters_total += it;
            right_envs[j + 1] = mps::grow_right_mpo(&right_envs[j + 2], &mpo.sites[j + 1], &tensors[j + 1]);
        }
        energy_history.push(last_energy);
        let max_dw = discarded.iter().cloned().fold(0.0f64, f64::max);
        let de = (last_energy - prev_energy).abs();
        if sweeps_used >= cfg.min_sweeps && de <= cfg.rtol * last_energy.abs().max(1.0) && max_dw <= cfg.max_discarded {
            converged = true;
            break;
        }
        prev_energy = last_energy;
    }
    let occ = crate::observables::occupation_profile(&tensors, l / 2);
    let spin_occ = crate::observables::spin_orbital_occupations(&tensors);
    let bond_dims = tensors[..l.saturating_sub(1)].iter().map(|t| t.chi_r).collect();
    Ok((
        DmrgResult {
            energy: last_energy,
            tensors,
            sweeps_used,
            converged,
            discarded_weight: discarded,
            spectrum_floor,
            energy_history,
            occupation_profile: occ,
            spin_orbital_occupations: spin_occ,
            bond_dims,
            worst_lanczos_residual: worst_resid,
            lanczos_iterations_total: iters_total,
        },
        labels,
    ))
}
