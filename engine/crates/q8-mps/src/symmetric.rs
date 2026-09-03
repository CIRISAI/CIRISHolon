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
    /// An environment was not block-diagonal in the bond's labels: the operator does not
    /// conserve them (`blocks::NotBlockDiagonal`).
    NotBlockDiagonal { bond: usize, why: String },
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
            SymRefusal::NotBlockDiagonal { bond, why } => write!(f, "bond {bond}: {why}"),
        }
    }
}

#[derive(Clone)]
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
    /// E14 item 2: skip the local eigensolve on a bond whose two tensors did not move in the
    /// previous sweep (relative change within `sqrt(rtol)`), and only re-canonicalise it.
    ///
    /// OFF, AND MEASURED TO STAY OFF WHERE TRUNCATION IS REAL. On N = 6, B = 0 at χ = 64 the
    /// skipping arm's energy oscillates 3.7e-6 above the full arm's with a three-sweep period
    /// (`tests/qcd2_gauge.rs`): a frozen middle bond drifts out of the truncation fixed point
    /// its neighbours keep moving under, and when its motion finally re-solves it, the split
    /// truncates it afresh and the state pays that cost again. On a sector the ladder solves
    /// exactly the option never fires, because the amendment's test converges in the minimum
    /// sweeps. So under this convergence test skipping buys nothing safe, and it is kept as
    /// the instrument that measured that, not as a speed-up. Three criteria were tried and
    /// are recorded at the site: discarded weight (never fires on truncation), local energy
    /// (second order in the motion: skipped bonds that were still turning), tensor motion
    /// (this one — first order, and still perturbs the fixed point).
    pub skip_unmoved: bool,
    /// E14 item 3, the SUBSPACE EXPANSION Q10_PREREG.md §4 names (White 2005's density-matrix
    /// perturbation, in the labelled two-site update): the reduced density matrix whose
    /// eigenvectors become the new bond basis is `ρ + α·P`, where `P` is the unit-trace sum of
    /// the environment-and-MPO images of the two-site state, blockwise by charge. The state
    /// kept is still the PROJECTION of ψ — the perturbation chooses the basis, it injects
    /// nothing — so a block that ψ has no weight in can be opened by the Hamiltonian's own
    /// direction into it, which is exactly what a warm ladder inheriting a truncated label
    /// set cannot do by itself (label re-seeding alone left x = 4, B = 1 at 2.65e-5, 60× the
    /// cold start's 4.3e-7). `0.0` is the plain blockwise SVD path, bit for bit.
    pub mixing: f64,
    /// E14 item 5b: where the two-site operator runs. `None` is this crate's host loops
    /// (`BlockPlan::apply`); `holon-gpu` supplies the device. The backend is asked once per
    /// local eigensolve for a matvec closure over the plan, so its uploads happen once per
    /// bond and the hundreds of Lanczos matvecs move only ψ.
    pub backend: Option<std::sync::Arc<dyn crate::blocks::TwoSiteBackend>>,
}

impl std::fmt::Debug for SymConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymConfig")
            .field("chi_max", &self.chi_max)
            .field("max_sweeps", &self.max_sweeps)
            .field("rtol", &self.rtol)
            .field("max_discarded", &self.max_discarded)
            .field("min_sweeps", &self.min_sweeps)
            .field("ignore_labels", &self.ignore_labels)
            .field("skip_unmoved", &self.skip_unmoved)
            .field("mixing", &self.mixing)
            .field("backend", &self.backend.as_ref().map(|_| "device"))
            .finish()
    }
}

impl SymConfig {
    /// The amendment's stated convergence test at bond dimension `chi_max`.
    pub fn amendment(chi_max: usize, max_sweeps: usize) -> SymConfig {
        SymConfig { chi_max, max_sweeps, rtol: 1e-10, max_discarded: 1e-8, min_sweeps: 4, ignore_labels: false, skip_unmoved: false, mixing: 0.0, backend: None }
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
    let labels = reachable_labels(sector, chi0);
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

/// Every charge label a bond can carry in this sector — the counts a left part can hold,
/// intersected with what the right part must still supply — up to `chi0` of them per bond,
/// closest to the proportional fill first. The label set `random_start` seeds and the set
/// `reseed_labels` restores.
pub fn reachable_labels(sector: &Sector, chi0: usize) -> Labels {
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
    labels
}

/// LABEL RE-SEEDING — E14 item 3, the cure for the mechanism E7 measured.
///
/// A labelled two-site update can only produce labels that combine its two neighbouring
/// bonds' existing ones, so a charge block truncated away at a low rung of a χ-ladder can
/// never return once both its neighbours lack the labels that would rebuild it (§A1.8.2).
/// The warm ladder 64 → 128 → 256 inherited exactly that: at x = 4, B = 1 it read 6.6e-5
/// above the exact referee where a COLD start at the same χ = 256 read 4.3e-7 — 156× — and
/// was cheaper (`GF2A_QCD2_RESULTS.md`, the cold diagnostic). Growth by zero-padding is what
/// `Q10_PREREG.md` §4 bans; this is the labelled form of the growth rule it names instead:
/// new bond directions that are non-null and physically reachable.
///
/// Every reachable label absent from a bond is restored as ONE new bond state, carrying
/// label-consistent entries of size `eps` relative to the adjacent tensor's largest entry
/// (deterministic pseudo-random from `seed`), on both tensors the bond joins. The sweep
/// then decides what the block is worth; it can only do so if the block exists. Returns
/// how many labels were restored. `chi0` bounds the reachable set as in `random_start`.
pub fn reseed_labels(tensors: &mut [TensorSite], labels: &mut Labels, sector: &Sector, chi0: usize, eps: f64, seed: u64) -> usize {
    let l = sector.site_charge.len();
    assert_eq!(tensors.len(), l);
    assert_eq!(labels.len(), l + 1);
    let reach = reachable_labels(sector, chi0);
    let mut st = seed;
    let mut rnd = || {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((st >> 11) as f64) / ((1u64 << 53) as f64) - 0.5
    };
    let mut restored = 0usize;
    for j in 1..l {
        let missing: Vec<Charge> = reach[j].iter().copied().filter(|q| !labels[j].contains(q)).collect();
        if missing.is_empty() {
            continue;
        }
        let k = missing.len();
        // the tensor left of bond j grows its right index; the tensor right of it, its left
        let (lt, rt) = (&tensors[j - 1], &tensors[j]);
        let scale_l = lt.data.iter().fold(0.0f64, |m, v| m.max(v.abs())).max(f64::MIN_POSITIVE);
        let scale_r = rt.data.iter().fold(0.0f64, |m, v| m.max(v.abs())).max(f64::MIN_POSITIVE);
        let mut new_l = TensorSite::zeros(lt.chi_l, lt.chi_r + k);
        for s in 0..2 {
            for a in 0..lt.chi_l {
                for b in 0..lt.chi_r {
                    new_l.set(s, a, b, lt.get(s, a, b));
                }
            }
        }
        let mut new_r = TensorSite::zeros(rt.chi_l + k, rt.chi_r);
        for s in 0..2 {
            for a in 0..rt.chi_l {
                for b in 0..rt.chi_r {
                    new_r.set(s, a, b, rt.get(s, a, b));
                }
            }
        }
        for (m, &q) in missing.iter().enumerate() {
            let idx = lt.chi_r + m;
            for (li, &ql) in labels[j - 1].iter().enumerate() {
                for a in 0..2 {
                    let after = if a == 1 { charge_add(ql, sector.site_charge[j - 1]) } else { ql };
                    if after == q {
                        new_l.set(a, li, idx, eps * scale_l * rnd());
                    }
                }
            }
            for (ri, &qr) in labels[j + 1].iter().enumerate() {
                for b in 0..2 {
                    let after = if b == 1 { charge_add(q, sector.site_charge[j]) } else { q };
                    if after == qr {
                        new_r.set(b, idx, ri, eps * scale_r * rnd());
                    }
                }
            }
        }
        tensors[j - 1] = new_l;
        tensors[j] = new_r;
        labels[j].extend(missing);
        restored += k;
    }
    restored
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
) -> Option<(TensorSite, TensorSite, f64, f64, Vec<Charge>, Vec<(Charge, f64)>)> {
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
    // BLOCK-RESCUING truncation. A charge block dropped from a bond can never return once both
    // neighbours lack it (the labelled update only combines existing labels), so after the
    // largest `chi_max` singular values are kept, any block that carries weight above
    // BLOCK_FLOOR of the largest and has no kept state is RESCUED with its largest state, and
    // the bond exceeds `chi_max` by the rescued count. Measured 2026-09-03, both ways: a
    // chi=32 rung truncating purely by size dropped sectors at N=8 and every higher rung then
    // converged (discarded 4e-21) inside the wrong label set (x=4 B=1: −39.4956 against the
    // exact −47.9965); and reserving one state per block BEFORE the size-ordered budget spent
    // all of chi=64 on 64 blocks and worsened N=6 B=0 from 1.2e-3 to 3.8e-2.
    const BLOCK_FLOOR: f64 = 1e-12;
    let mut keep = vec![false; triples.len()];
    for k in keep.iter_mut().take(chi_max.min(triples.len())) {
        *k = true;
    }
    let s_top = triples[0].0;
    let mut have: Vec<Charge> = triples.iter().zip(&keep).filter(|(_, k)| **k).map(|(t, _)| t.1).collect();
    for (i, t) in triples.iter().enumerate() {
        if !keep[i] && t.0 * t.0 >= BLOCK_FLOOR * s_top * s_top && !have.contains(&t.1) {
            keep[i] = true;
            have.push(t.1);
        }
    }
    let kept: Vec<&(f64, Charge, Vec<f64>, Vec<f64>)> = triples.iter().zip(&keep).filter(|(_, k)| **k).map(|(t, _)| t).collect();
    let discarded: f64 = triples.iter().zip(&keep).filter(|(_, k)| !**k).map(|(t, _)| t.0 * t.0).sum();
    let chi_new = kept.len().max(1);
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
    // the kept singular mass per charge block: which labels carry the state
    let mut mass: Vec<(Charge, f64)> = Vec::new();
    for (s, c, _, _) in kept.iter() {
        match mass.iter_mut().find(|m| m.0 == *c) {
            Some(m) => m.1 += s * s,
            None => mass.push((*c, s * s)),
        }
    }
    Some((a_left, a_right, discarded, floor, new_q, mass))
}

/// The environment-and-MPO images of the two-site state on the LEFT (rows `(l', s)`, one
/// image per mid channel `c2`): `Φ_{c2}[(l',s),(b,r)] = Σ_{c1,l,a} L[c1][l',l]·W1[c1,c2,s,a]·Ψ[(l,a),(b,r)]`.
fn left_images(psi: &[f64], chi_l: usize, chi_r: usize, left: &Env, w1: &crate::mpo::MpoSite, live_l: &[usize]) -> Vec<Vec<f64>> {
    let (m, n) = (2 * chi_l, 2 * chi_r);
    let mut phis: Vec<Vec<f64>> = vec![Vec::new(); w1.d_r];
    for &c1 in live_l {
        // t[(l',a),(b,r)] = Σ_l L[c1][l',l] Ψ[(l,a),(b,r)]
        let lmat = &left[c1];
        let mut t = vec![0.0; m * n];
        for lp in 0..chi_l {
            for l in 0..chi_l {
                let lv = lmat[lp * chi_l + l];
                if lv == 0.0 {
                    continue;
                }
                for a in 0..2 {
                    let src = (l * 2 + a) * n;
                    let dst = (lp * 2 + a) * n;
                    for k in 0..n {
                        t[dst + k] += lv * psi[src + k];
                    }
                }
            }
        }
        for c2 in 0..w1.d_r {
            for sb in 0..2 {
                for a in 0..2 {
                    let wv = w1.get(c1, c2, sb, a);
                    if wv == 0.0 {
                        continue;
                    }
                    if phis[c2].is_empty() {
                        phis[c2] = vec![0.0; m * n];
                    }
                    let phi = &mut phis[c2];
                    for lp in 0..chi_l {
                        let src = (lp * 2 + a) * n;
                        let dst = (lp * 2 + sb) * n;
                        for k in 0..n {
                            phi[dst + k] += wv * t[src + k];
                        }
                    }
                }
            }
        }
    }
    phis.into_iter().filter(|p| !p.is_empty()).collect()
}

/// The mirror on the RIGHT, returned TRANSPOSED so the column space reads as rows `(t, r')`:
/// `Φ'_{c1'}[(t,r'),(l,a)] = Σ_{c2,b,r} R[c2][r',r]·W2[c1',c2,t,b]·Ψ[(l,a),(b,r)]`.
fn right_images_t(psi: &[f64], chi_l: usize, chi_r: usize, right: &Env, w2: &crate::mpo::MpoSite, live_r: &[usize]) -> Vec<Vec<f64>> {
    let (m, n) = (2 * chi_l, 2 * chi_r);
    let mut phis: Vec<Vec<f64>> = vec![Vec::new(); w2.d_l];
    for &c2 in live_r {
        // t[(b,r'),(l,a)] = Σ_r R[c2][r',r] Ψ[(l,a),(b,r)]  (stored transposed: n x m)
        let rmat = &right[c2];
        let mut t = vec![0.0; n * m];
        for rp in 0..chi_r {
            for r in 0..chi_r {
                let rv = rmat[rp * chi_r + r];
                if rv == 0.0 {
                    continue;
                }
                for b in 0..2 {
                    for row in 0..m {
                        t[(b * chi_r + rp) * m + row] += rv * psi[row * n + b * chi_r + r];
                    }
                }
            }
        }
        for c1p in 0..w2.d_l {
            for tb in 0..2 {
                for b in 0..2 {
                    let wv = w2.get(c1p, c2, tb, b);
                    if wv == 0.0 {
                        continue;
                    }
                    if phis[c1p].is_empty() {
                        phis[c1p] = vec![0.0; n * m];
                    }
                    let phi = &mut phis[c1p];
                    for rp in 0..chi_r {
                        let src = (b * chi_r + rp) * m;
                        let dst = (tb * chi_r + rp) * m;
                        for k in 0..m {
                            phi[dst + k] += wv * t[src + k];
                        }
                    }
                }
            }
        }
    }
    phis.into_iter().filter(|p| !p.is_empty()).collect()
}

/// The perturbed blockwise split. `psi_rows` is `m x n` row-major with rows in the space that
/// receives the new basis (the left `(l,a)` space moving right, the right `(b,r)` space —
/// transposed — moving left); `row_charge[i]` labels row `i`; `images` are the perturbation's
/// images in the same layout. Returns, in kept order: `(charge, eigenvector over rows, the
/// projected other-side row = uᵀΨ)`, the discarded weight of `psi` outside the kept space,
/// the kept-spectrum floor, and the per-block kept mass of `psi`.
#[allow(clippy::type_complexity)]
fn mixed_row_split(
    psi_rows: &[f64],
    m: usize,
    n: usize,
    row_charge: &[Charge],
    images: &[Vec<f64>],
    alpha: f64,
    chi_max: usize,
) -> Option<(Vec<(Charge, Vec<f64>, Vec<f64>)>, f64, f64, Vec<(Charge, f64)>)> {
    // the perturbation's total trace, so `alpha` is a FRACTION of the state's own weight
    let mut ptrace = 0.0;
    for im in images {
        ptrace += im.iter().map(|v| v * v).sum::<f64>();
    }
    let pscale = if ptrace > 0.0 { alpha / ptrace } else { 0.0 };
    let mut charges: Vec<Charge> = row_charge.to_vec();
    charges.sort();
    charges.dedup();
    // (eigenvalue, charge, eigenvector over ALL rows)
    let mut triples: Vec<(f64, Charge, Vec<f64>)> = Vec::new();
    for &c in &charges {
        if c == NO_CHARGE {
            continue;
        }
        let rows: Vec<usize> = (0..m).filter(|&i| row_charge[i] == c).collect();
        let k = rows.len();
        // rho'_R = Psi_R Psi_R^T + pscale * sum_im Im_R Im_R^T
        let mut rho = vec![0.0; k * k];
        for (i, &ri) in rows.iter().enumerate() {
            for (j, &rj) in rows.iter().enumerate().take(i + 1) {
                let mut acc = 0.0;
                let (a, b) = (&psi_rows[ri * n..ri * n + n], &psi_rows[rj * n..rj * n + n]);
                for x in 0..n {
                    acc += a[x] * b[x];
                }
                let mut pert = 0.0;
                for im in images {
                    let (a, b) = (&im[ri * n..ri * n + n], &im[rj * n..rj * n + n]);
                    for x in 0..n {
                        pert += a[x] * b[x];
                    }
                }
                let v = acc + pscale * pert;
                rho[i * k + j] = v;
                rho[j * k + i] = v;
            }
        }
        if rho.iter().all(|v| *v == 0.0) {
            continue;
        }
        // symmetric PSD: its SVD is its eigendecomposition
        let svd = crate::svd::jacobi_svd(&rho, k, k);
        if !svd.converged {
            return None;
        }
        for (e, &lam) in svd.s.iter().enumerate() {
            if lam <= 0.0 {
                continue;
            }
            let mut u_full = vec![0.0; m];
            for (i, &ri) in rows.iter().enumerate() {
                u_full[ri] = svd.u[e][i];
            }
            triples.push((lam, c, u_full));
        }
    }
    if triples.is_empty() {
        return None;
    }
    triples.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    const BLOCK_FLOOR: f64 = 1e-12;
    let mut keep = vec![false; triples.len()];
    for kf in keep.iter_mut().take(chi_max.min(triples.len())) {
        *kf = true;
    }
    let top = triples[0].0;
    let mut have: Vec<Charge> = triples.iter().zip(&keep).filter(|(_, k)| **k).map(|(t, _)| t.1).collect();
    for (i, t) in triples.iter().enumerate() {
        if !keep[i] && t.0 >= BLOCK_FLOOR * top && !have.contains(&t.1) {
            keep[i] = true;
            have.push(t.1);
        }
    }
    let mut kept: Vec<(Charge, Vec<f64>, Vec<f64>)> = Vec::new();
    let mut kept_weight = 0.0;
    let mut mass: Vec<(Charge, f64)> = Vec::new();
    let (mut lmax, mut lmin) = (0.0f64, f64::INFINITY);
    for (t, &k) in triples.iter().zip(&keep) {
        if !k {
            continue;
        }
        let (lam, c, u) = t;
        lmax = lmax.max(*lam);
        lmin = lmin.min(*lam);
        // the projected other side: row_i = u^T Psi
        let mut proj = vec![0.0; n];
        for (ri, &uv) in u.iter().enumerate() {
            if uv == 0.0 {
                continue;
            }
            let row = &psi_rows[ri * n..ri * n + n];
            for x in 0..n {
                proj[x] += uv * row[x];
            }
        }
        let w: f64 = proj.iter().map(|v| v * v).sum();
        kept_weight += w;
        match mass.iter_mut().find(|q| q.0 == *c) {
            Some(q) => q.1 += w,
            None => mass.push((*c, w)),
        }
        kept.push((*c, u.clone(), proj));
    }
    let total: f64 = psi_rows.iter().map(|v| v * v).sum();
    let discarded = (total - kept_weight).max(0.0);
    let floor = if lmax > 0.0 { lmin / lmax } else { 0.0 };
    Some((kept, discarded, floor, mass))
}

/// The blockwise split WITH White's perturbation (`SymConfig::mixing > 0`): the same
/// return as [`split_two_site_sym`], the new basis chosen from `ρ + α·P`.
#[allow(clippy::too_many_arguments)]
pub fn split_two_site_sym_mixed(
    psi: &[f64],
    q_l: &[Charge],
    q_r: &[Charge],
    e1: Charge,
    e2: Charge,
    chi_max: usize,
    absorb_s_left: bool,
    alpha: f64,
    left: &Env,
    right: &Env,
    w1: &crate::mpo::MpoSite,
    w2: &crate::mpo::MpoSite,
    live_l: &[usize],
    live_r: &[usize],
) -> Option<(TensorSite, TensorSite, f64, f64, Vec<Charge>, Vec<(Charge, f64)>)> {
    let (chi_l, chi_r) = (q_l.len(), q_r.len());
    let (m, n) = (2 * chi_l, 2 * chi_r);
    if !absorb_s_left {
        // moving RIGHT: the left (l,a) space receives the new basis
        let row_charge: Vec<Charge> = (0..m).map(|i| if i % 2 == 1 { charge_add(q_l[i / 2], e1) } else { q_l[i / 2] }).collect();
        let images = left_images(psi, chi_l, chi_r, left, w1, live_l);
        let (kept, dw, floor, mass) = mixed_row_split(psi, m, n, &row_charge, &images, alpha, chi_max)?;
        let chi_new = kept.len();
        let mut a_left = TensorSite::zeros(chi_l, chi_new);
        let mut a_right = TensorSite::zeros(chi_new, chi_r);
        let mut new_q = Vec::with_capacity(chi_new);
        for (i, (c, u, proj)) in kept.iter().enumerate() {
            for l in 0..chi_l {
                for a in 0..2 {
                    a_left.set(a, l, i, u[l * 2 + a]);
                }
            }
            for b in 0..2 {
                for r in 0..chi_r {
                    a_right.set(b, i, r, proj[b * chi_r + r]);
                }
            }
            new_q.push(*c);
        }
        Some((a_left, a_right, dw, floor, new_q, mass))
    } else {
        // moving LEFT: the right (b,r) space receives the new basis; work on Ψ transposed
        let mut psi_t = vec![0.0; n * m];
        for row in 0..m {
            for col in 0..n {
                psi_t[col * m + row] = psi[row * n + col];
            }
        }
        let col_charge: Vec<Charge> = (0..n).map(|j| { let (b, r) = (j / chi_r, j % chi_r); if b == 1 { charge_sub(q_r[r], e2) } else { q_r[r] } }).collect();
        let images = right_images_t(psi, chi_l, chi_r, right, w2, live_r);
        let (kept, dw, floor, mass) = mixed_row_split(&psi_t, n, m, &col_charge, &images, alpha, chi_max)?;
        let chi_new = kept.len();
        let mut a_left = TensorSite::zeros(chi_l, chi_new);
        let mut a_right = TensorSite::zeros(chi_new, chi_r);
        let mut new_q = Vec::with_capacity(chi_new);
        for (i, (c, v, proj)) in kept.iter().enumerate() {
            for b in 0..2 {
                for r in 0..chi_r {
                    a_right.set(b, i, r, v[b * chi_r + r]);
                }
            }
            for l in 0..chi_l {
                for a in 0..2 {
                    a_left.set(a, l, i, proj[l * 2 + a]);
                }
            }
            new_q.push(*c);
        }
        Some((a_left, a_right, dw, floor, new_q, mass))
    }
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
    skip: bool,
) -> Result<(f64, f64, f64, f64, usize, Vec<Charge>, Vec<(Charge, f64)>), SymRefusal> {
    let (w1, w2) = (&mpo.sites[j], &mpo.sites[j + 1]);
    let chi_l = tensors[j].chi_l;
    let chi_r = tensors[j + 1].chi_r;
    debug_assert_eq!(q_l.len(), chi_l);
    debug_assert_eq!(q_r.len(), chi_r);
    let mid = tensors[j].chi_r;
    let dim = chi_l * 2 * 2 * chi_r;
    let t_seed = std::time::Instant::now();
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
    TIMING.with(|t| t.borrow_mut().seed += t_seed.elapsed().as_secs_f64());
    let t_plan = std::time::Instant::now();
    let (live_l, live_r) = (mps::live_channels(left_env), mps::live_channels(right_env));
    // THE BLOCK-SPARSE OPERATOR (E14 item 1, `blocks.rs`): the labels say which entries of
    // the two-site tensor can be nonzero, and the environments are block-diagonal in them,
    // so the eigensolve contracts the live blocks only — bit-identical to the dense operator,
    // gated in `tests/qcd2_gauge.rs`. The mutant keeps the dense path: it has no labels to plan by.
    let plan = if cfg.ignore_labels {
        None
    } else {
        Some(crate::blocks::BlockPlan::build(q_l, q_r, e1, e2, left_env, right_env).map_err(|e| SymRefusal::NotBlockDiagonal { bond: j, why: e.to_string() })?)
    };
    let device = match (&plan, &cfg.backend) {
        (Some(p), Some(b)) => Some(b.matvec(p, w1, w2)),
        _ => None,
    };
    let apply = |psi: &[f64]| {
            let mut h = match (&device, &plan) {
                (Some(f), _) => f(psi),
                (None, Some(p)) => p.apply(left_env, w1, w2, right_env, psi),
                (None, None) => mps::apply_effective_h_mpo_live(left_env, w1, w2, right_env, psi, chi_l, chi_r, &live_l, &live_r),
            };
            if !cfg.ignore_labels {
                for (v, &ok) in h.iter_mut().zip(&msk) {
                    if !ok {
                        *v = 0.0;
                    }
                }
            }
            h
        };
    // A SKIPPED bond keeps its state and is only re-canonicalised: the two-site tensor is the
    // seed normalised, its energy one matvec, no Lanczos. The residual reported is the
    // seed's own, so a skipped bond that was NOT converged shows up as such.
    TIMING.with(|t| t.borrow_mut().plan += t_plan.elapsed().as_secs_f64());
    let t_lan = std::time::Instant::now();
    let gs = if skip {
        let nrm = seed.iter().map(|v| v * v).sum::<f64>().sqrt();
        let v: Vec<f64> = seed.iter().map(|x| x / nrm).collect();
        let hv = apply(&v);
        let e: f64 = v.iter().zip(&hv).map(|(a, b)| a * b).sum();
        let resid = hv.iter().zip(&v).map(|(h, x)| (h - e * x) * (h - e * x)).sum::<f64>().sqrt();
        lanczos::TwoSiteGroundState { energy: e, vector: v, residual: resid, iterations: 0 }
    } else {
        lanczos::ground_state(apply, &seed, dim).ok_or(SymRefusal::LanczosFailed { bond: j })?
    };
    TIMING.with(|t| t.borrow_mut().lanczos += t_lan.elapsed().as_secs_f64());
    let t_split = std::time::Instant::now();
    if cfg.ignore_labels {
        let (a_left, a_right, dw, floor) = mps::split_two_site(&gs.vector, chi_l, chi_r, cfg.chi_max, absorb_s_left);
        let chi_new = a_left.chi_r;
        tensors[j] = a_left;
        tensors[j + 1] = a_right;
        return Ok((gs.energy, dw, gs.residual, floor, gs.iterations, vec![NO_CHARGE; chi_new], Vec::new()));
    }
    // A SKIPPED bond is re-canonicalised, never re-truncated: block rescue lets a bond exceed
    // `chi_max`, so a two-site tensor built from two such sites has rank up to `mid` and a
    // split at `chi_max` would cut it — silently, since the skipped bond keeps its previous
    // truncation record. Measured before this line existed: the skipping arm's energy
    // oscillated by 3.7e-6 with a three-sweep period on N = 6, B = 0 at χ = 64. Splitting at
    // `max(chi_max, mid)` keeps the whole rank and the re-canonicalisation is exact.
    let chi_split = if skip { cfg.chi_max.max(mid) } else { cfg.chi_max };
    let (a_left, a_right, dw, floor, new_q, mass) = if cfg.mixing > 0.0 && !skip {
        split_two_site_sym_mixed(&gs.vector, q_l, q_r, e1, e2, chi_split, absorb_s_left, cfg.mixing, left_env, right_env, w1, w2, &live_l, &live_r)
            .ok_or(SymRefusal::EmptyCut { bond: j })?
    } else {
        split_two_site_sym(&gs.vector, q_l, q_r, e1, e2, chi_split, absorb_s_left).ok_or(SymRefusal::EmptyCut { bond: j })?
    };
    tensors[j] = a_left;
    tensors[j + 1] = a_right;
    TIMING.with(|t| t.borrow_mut().split += t_split.elapsed().as_secs_f64());
    Ok((gs.energy, dw, gs.residual, floor, gs.iterations, new_q, mass))
}

/// Stage timing of the labelled sweep, thread-local, printed per sweep when `Q8_TIMING` is
/// set: where a sweep's seconds go (the seed, the block plan, the eigensolve, the split, the
/// environment growth), so a speedup is attributed to the stage it belongs to and a stage
/// that quietly dominates is named. The instrument that found the χ = 64 rung's time was
/// NOT in the eigensolve after the block-sparse operator landed.
#[derive(Default, Clone, Copy)]
pub struct StageSeconds {
    pub seed: f64,
    pub plan: f64,
    pub lanczos: f64,
    pub split: f64,
    pub envs: f64,
}

thread_local! {
    static TIMING: std::cell::RefCell<StageSeconds> = const { std::cell::RefCell::new(StageSeconds { seed: 0.0, plan: 0.0, lanczos: 0.0, split: 0.0, envs: 0.0 }) };
}

/// Take and reset the accumulated stage seconds of this thread.
pub fn take_stage_seconds() -> StageSeconds {
    TIMING.with(|t| std::mem::take(&mut *t.borrow_mut()))
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
    // E14 item 2: what moved. Per bond the local energy of this sweep and the last, per site
    // the tensor at the start of the sweep, per bond the kept block masses of the last split.
    let mut bond_energy = vec![f64::NAN; l.saturating_sub(1)];
    let mut bond_energy_prev = vec![f64::NAN; l.saturating_sub(1)];
    let mut bond_energy_delta = vec![f64::INFINITY; l.saturating_sub(1)];
    let mut site_delta = vec![f64::NAN; l];
    let mut block_mass: Vec<Vec<(Charge, f64)>> = vec![Vec::new(); l.saturating_sub(1)];
    let mut bonds_skipped = 0usize;
    let mut right_envs = all_right_envs(&tensors, mpo);
    // THE MIXING SCHEDULE. White's perturbation chooses the kept basis while the state is
    // still moving and must be OFF at the end, or the basis it leaves behind is a perturbed
    // one — measured: constant α = 1e-4 left N = 6, B = 2 at χ = 64 1.7e-8 above a referee
    // the plain split reaches to 1e-11. The rule: mix while the last sweep's energy change
    // was above 100·rtol·|E|, run unmixed once it is not, and declare convergence only on an
    // UNMIXED sweep. `cfg.mixing` is the ceiling; `sweep_cfg` is what this sweep runs with.
    let mut sweep_cfg = cfg.clone();
    let mut mixed_last;
    for sweep in 0..cfg.max_sweeps {
        sweeps_used = sweep + 1;
        let still_moving = prev_energy.is_infinite() || (last_energy - prev_energy).abs() > 100.0 * cfg.rtol * last_energy.abs().max(1.0);
        sweep_cfg.mixing = if cfg.mixing > 0.0 && still_moving { cfg.mixing } else { 0.0 };
        mixed_last = sweep_cfg.mixing > 0.0;
        let snapshot: Vec<TensorSite> = tensors.clone();
        // A bond is skippable when its two TENSORS did not move in the previous sweep — the
        // relative Frobenius change of both sites inside `sqrt(rtol)`. Two earlier criteria
        // were measured wrong on N = 6, B = 0 at χ = 64: requiring the discarded weight under
        // `max_discarded` never fires on a truncated sector (the weight is truncation, not
        // motion), and requiring the LOCAL ENERGY change inside `rtol` skipped the five middle
        // bonds from sweep three on while the state kept rotating under them — the energy is
        // stationary to second order in a state change, so it reads "stopped" on a bond that
        // is still turning, and the skipping arm parked 2e-6 above the full arm. The tensors'
        // own motion is first order and is what `site_delta` records. A tensor that changed
        // SHAPE reads `NAN` and is never skippable; the first sweep skips nothing.
        // The two END bonds are never skipped: the sweep's energy is reported from the last
        // bond visited, and a reported energy must be a MINIMISED one, or the history mixes
        // the state's energy before and after its last optimisation (measured as a clean
        // 3.7e-6 oscillation with a three-sweep period when bond 0 was skippable).
        let thr = cfg.rtol.sqrt();
        let nb = l.saturating_sub(1);
        let skippable: Vec<bool> = (0..nb)
            .map(|j| cfg.skip_unmoved && j != 0 && j + 1 != nb && site_delta[j] <= thr && site_delta[j + 1] <= thr)
            .collect();
        let mut left_envs: Vec<Env> = Vec::with_capacity(l);
        left_envs.push(mps::trivial_left_env_mpo(mpo.sites[0].d_l));
        for j in 0..(l - 1) {
            let (q_l, q_r) = (labels[j].clone(), labels[j + 2].clone());
            let skip = skippable[j];
            let (e, dw, resid, sf, it, new_q, mass) = update(&mut tensors, &q_l, &q_r, sector, mpo, j, &sweep_cfg, false, &left_envs[j], &right_envs[j + 2], skip)?;
            labels[j + 1] = new_q;
            last_energy = e;
            bond_energy[j] = e;
            // a SKIPPED bond keeps its last real truncation record: re-splitting a two-site
            // tensor of rank ≤ χ discards nothing, and a zero written here would let the
            // convergence test's discarded-weight leg pass on a bond that was never re-solved
            // (measured: the first skipping run "converged" in 6 sweeps on a sector whose
            // middle bonds truncate at 1e-6)
            if skip {
                bonds_skipped += 1;
            } else {
                discarded[j] = dw;
                spectrum_floor[j] = sf;
                block_mass[j] = mass;
            }
            worst_resid = worst_resid.max(resid);
            iters_total += it;
            let t_env = std::time::Instant::now();
            let grown = mps::grow_left_mpo(&left_envs[j], &mpo.sites[j], &tensors[j]);
            left_envs.push(grown);
            TIMING.with(|t| t.borrow_mut().envs += t_env.elapsed().as_secs_f64());
        }
        for j in (0..(l - 1)).rev() {
            let (q_l, q_r) = (labels[j].clone(), labels[j + 2].clone());
            let skip = skippable[j];
            let (e, dw, resid, sf, it, new_q, mass) = update(&mut tensors, &q_l, &q_r, sector, mpo, j, &sweep_cfg, true, &left_envs[j], &right_envs[j + 2], skip)?;
            labels[j + 1] = new_q;
            last_energy = e;
            bond_energy[j] = e;
            // a SKIPPED bond keeps its last real truncation record: re-splitting a two-site
            // tensor of rank ≤ χ discards nothing, and a zero written here would let the
            // convergence test's discarded-weight leg pass on a bond that was never re-solved
            // (measured: the first skipping run "converged" in 6 sweeps on a sector whose
            // middle bonds truncate at 1e-6)
            if skip {
                bonds_skipped += 1;
            } else {
                discarded[j] = dw;
                spectrum_floor[j] = sf;
                block_mass[j] = mass;
            }
            worst_resid = worst_resid.max(resid);
            iters_total += it;
            let t_env = std::time::Instant::now();
            right_envs[j + 1] = mps::grow_right_mpo(&right_envs[j + 2], &mpo.sites[j + 1], &tensors[j + 1]);
            TIMING.with(|t| t.borrow_mut().envs += t_env.elapsed().as_secs_f64());
        }
        if std::env::var_os("Q8_TIMING").is_some() {
            let st = take_stage_seconds();
            eprintln!(
                "q8 sweep {} chi_max {} mix {:.0e}: seed {:.2}s plan {:.2}s lanczos {:.2}s split {:.2}s envs {:.2}s | E {:.10} skipped {}",
                sweeps_used, cfg.chi_max, sweep_cfg.mixing, st.seed, st.plan, st.lanczos, st.split, st.envs, last_energy, bonds_skipped
            );
        }
        for j in 0..l.saturating_sub(1) {
            bond_energy_delta[j] = if bond_energy_prev[j].is_nan() { f64::INFINITY } else { (bond_energy[j] - bond_energy_prev[j]).abs() };
            bond_energy_prev[j] = bond_energy[j];
        }
        for (j, (old, new)) in snapshot.iter().zip(&tensors).enumerate() {
            site_delta[j] = if old.chi_l == new.chi_l && old.chi_r == new.chi_r {
                let num: f64 = old.data.iter().zip(&new.data).map(|(a, b)| (a - b) * (a - b)).sum::<f64>().sqrt();
                let den: f64 = old.data.iter().map(|a| a * a).sum::<f64>().sqrt();
                if den > 0.0 { num / den } else { f64::NAN }
            } else {
                f64::NAN
            };
        }
        energy_history.push(last_energy);
        let max_dw = discarded.iter().cloned().fold(0.0f64, f64::max);
        let de = (last_energy - prev_energy).abs();
        if !mixed_last && sweeps_used >= cfg.min_sweeps && de <= cfg.rtol * last_energy.abs().max(1.0) && max_dw <= cfg.max_discarded {
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
            bond_energy,
            bond_energy_delta,
            site_delta,
            block_mass,
            bonds_skipped,
        },
        labels,
    ))
}
