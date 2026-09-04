//! Massless one-flavour QCD₂ in axial gauge as an ACCUMULATOR MPO — the GF2a instrument
//! on the engine's DMRG (`conformance/crystal/GF2A_QCD2_PREREG.md`).
//!
//! The Hamiltonian is `holon_chem::qcd2`'s (same W-units, same orbital order `3k + c`):
//!
//! ```text
//! W = x Σ_k Σ_c (c†_{k,c} c_{k+1,c} + h.c.)
//!   + Σ_k w(k) Cas_k                              w(k) = N − 1 − k
//!   + Σ_{k<k'} w(k') [ Σ_{cc'} B^{(k)}_{cc'} B^{(k')}_{c'c}  −  ⅓ n_k n_{k'} ]
//! Cas_k = (4/3) Σ_c n_{k,c} − (4/3) Σ_{c<c'} n_{k,c} n_{k,c'}     (diagonal; 4/3, 4/3, 0 for 1, 2, 3 quarks)
//! B^{(k)}_{cc'} = c†_{k,c} c_{k,c'},   n_k = Σ_c n_{k,c}
//! ```
//!
//! (the cross term is `Σ_a q^a_k q^a_{k'}` through the SU(3) Fierz identity, written in the
//! REAL bilinear basis so no complex generator appears). The coefficient of every cross
//! term depends only on the LATER group, so — exactly as the Schwinger MPO's one Coulomb
//! accumulator — each bilinear needs ONE accumulator channel: nine `A_{cc'}` for the
//! bilinears, one for `n`, plus the within-group open/close states of the two-site
//! bilinears, the three-sites-apart hopping strings, and the diagonal Casimir pair channel.
//! Forty-two channels (`channels()`), independent of `N`; a generic integral builder would need `O(N)`.
//!
//! Jordan–Wigner over the `3N` colour modes in orbital order. For `i < j`:
//! `c†_i c_j = σ⁺_i Z_{i+1} … Z_{j−1} σ⁻_j` and `c†_j c_i = σ⁻_i Z_{i+1} … Z_{j−1} σ⁺_j`
//! (the boundary `Z`s absorb into the ladder operators; no residual sign). Basis: `s = 1`
//! occupied. Engine convention: `MpoSite::get(cl, cr, s = ket, sp = bra)`.
//!
//! Credits: Banks–Kogut–Susskind and Hamer et al. (staggered form); Atas et al. 2023 and
//! Farrell et al. 2023 (axial-gauge QCD₂ for quantum simulation); White 1992 (DMRG).

use crate::dmrg::{dmrg_sweep, DmrgConfig, DmrgResult, Refusal, RefusalPolicy};
use crate::symmetric::{dmrg_sweep_sym, labels_of_product, random_start, Labels, Sector, SymConfig, SymRefusal, ZERO_CHARGE};
use crate::mpo::{Mpo, MpoSite};
use crate::mps::TensorSite;

pub const COLOURS: usize = 3;
const CF: f64 = 4.0 / 3.0;

type Block = [[f64; 2]; 2];
/// `[bra][ket]` blocks.
const ID: Block = [[1.0, 0.0], [0.0, 1.0]];
const NUM: Block = [[0.0, 0.0], [0.0, 1.0]];
const ZJW: Block = [[1.0, 0.0], [0.0, -1.0]];
/// `σ⁺ = c†`: `⟨1|σ⁺|0⟩ = 1`.
const SPLUS: Block = [[0.0, 0.0], [1.0, 0.0]];
/// `σ⁻ = c`: `⟨0|σ⁻|1⟩ = 1`.
const SMINUS: Block = [[0.0, 1.0], [0.0, 0.0]];

fn scaled(b: Block, c: f64) -> Block {
    [[b[0][0] * c, b[0][1] * c], [b[1][0] * c, b[1][1] * c]]
}

/// The channel alphabet. `Acc*` channels persist across groups; `Open*`/`Close*` live
/// inside one group; hopping strings run exactly three sites.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Ch {
    Start,
    Done,
    /// `c†` placed at colour `c`, waiting for `c` three sites on (Z in between).
    HopPlus(usize),
    /// `c` placed at colour `c`, waiting for `c†` three sites on.
    HopMinus(usize),
    /// Casimir pair term: an `n` placed in this group, waiting for a later colour's `n`.
    CasPair,
    /// Global quark-number accumulator for the sector penalty `λ (N̂ − n_q)²`: closable at
    /// any later site (a two-site Lanczos leaks into a lower sector by roundoff and the
    /// leak grows; the penalty is what the Python driver carried for the same reason).
    AccTot,
    /// `n` accumulated inside the CURRENT group (cannot close yet).
    AccNOpen,
    /// `Σ_{k<k'} n_k`, closable.
    AccN,
    /// `B_{cc'}` opened at the lower colour of `(c, c')` inside this group.
    Open(usize, usize),
    /// `B_{cc'}` completed inside the current group (cannot close yet).
    AccBOpen(usize, usize),
    /// `Σ_{k<k'} B^{(k)}_{cc'}`, closable.
    AccB(usize, usize),
    /// closing `B^{(k')}_{c'c}` against `AccB(c, c')`: first factor placed, second pending.
    Close(usize, usize),
}

fn channels() -> Vec<Ch> {
    let mut v = vec![Ch::Start, Ch::Done, Ch::CasPair, Ch::AccTot, Ch::AccNOpen, Ch::AccN];
    for c in 0..COLOURS {
        v.push(Ch::HopPlus(c));
        v.push(Ch::HopMinus(c));
    }
    for c in 0..COLOURS {
        for cp in 0..COLOURS {
            v.push(Ch::AccBOpen(c, cp));
            v.push(Ch::AccB(c, cp));
            if c != cp {
                v.push(Ch::Open(c, cp));
                v.push(Ch::Close(c, cp));
            }
        }
    }
    v
}

/// The chain.
#[derive(Clone, Debug)]
pub struct Qcd2 {
    pub n: usize,
    pub x: f64,
    /// The sector-penalty strength: `4 (x + 1)`, which exceeds every sector energy gap this
    /// campaign meets by an order (a wrong-sector state costs `λ · 9` for one baryon) while
    /// keeping the effective Hamiltonian's norm, and with it the Lanczos residual scale, small.
    pub lam: f64,
}

impl Qcd2 {
    pub fn new(n: usize, x: f64) -> Self {
        assert!(n >= 2 && n % 2 == 0, "an even chain of at least two sites, got {n}");
        Self { n, x, lam: 4.0 * (x + 1.0) }
    }

    pub fn sites(&self) -> usize {
        COLOURS * self.n
    }

    /// Quarks in baryon-number sector `b`: the sea `3N/2` plus three per baryon.
    pub fn quarks(&self, b: i32) -> usize {
        (COLOURS * self.n / 2) as usize + 3 * b as usize
    }

    /// The transition list at JW site `j`: `(from, to, block, coefficient)`, including the
    /// sector penalty `λ (N̂ − n_q)²` as `λ N̂ − 2 λ n_q N̂ + 2 λ Σ_{i<j} n_i n_j` (the constant
    /// `λ n_q²` is added back by [`Qcd2::ground_energy`]).
    fn transitions(&self, j: usize, n_q: usize) -> Vec<(Ch, Ch, Block, f64)> {
        let n = self.n;
        let k = j / COLOURS;
        let c = j % COLOURS;
        let w = (n - 1 - k) as f64; // coefficient carried by this group when it is the LATER one
        let last_colour = c == COLOURS - 1;
        let mut t: Vec<(Ch, Ch, Block, f64)> = Vec::new();
        t.push((Ch::Start, Ch::Start, ID, 1.0));
        t.push((Ch::Done, Ch::Done, ID, 1.0));
        // ---- sector penalty
        t.push((Ch::Start, Ch::Done, NUM, self.lam * (1.0 - 2.0 * n_q as f64)));
        t.push((Ch::Start, Ch::AccTot, NUM, 1.0));
        t.push((Ch::AccTot, Ch::AccTot, ID, 1.0));
        t.push((Ch::AccTot, Ch::Done, NUM, 2.0 * self.lam));
        // ---- one-body Casimir (4/3) w(k) n_c
        if k + 2 <= n {
            t.push((Ch::Start, Ch::Done, NUM, CF * w));
        }
        // ---- Casimir pair term −(4/3) w(k) Σ_{c<c'} n_c n_c' inside the group
        if k + 2 <= n {
            if !last_colour {
                t.push((Ch::Start, Ch::CasPair, NUM, 1.0));
            }
            if c > 0 {
                t.push((Ch::CasPair, Ch::Done, NUM, -CF * w));
            }
            if c > 0 && !last_colour {
                t.push((Ch::CasPair, Ch::CasPair, ID, 1.0));
            }
        }
        // ---- hopping x (c†_{k,c} c_{k+1,c} + h.c.), strings of exactly three sites
        if k + 1 < n {
            t.push((Ch::Start, Ch::HopPlus(c), SPLUS, 1.0));
            t.push((Ch::Start, Ch::HopMinus(c), SMINUS, 1.0));
        }
        for cc in 0..COLOURS {
            if cc == c {
                // the string opened three sites ago at this colour closes here
                if k >= 1 {
                    t.push((Ch::HopPlus(cc), Ch::Done, SMINUS, self.x));
                    t.push((Ch::HopMinus(cc), Ch::Done, SPLUS, self.x));
                }
            } else {
                t.push((Ch::HopPlus(cc), Ch::HopPlus(cc), ZJW, 1.0));
                t.push((Ch::HopMinus(cc), Ch::HopMinus(cc), ZJW, 1.0));
            }
        }
        // ---- the n accumulator: −⅓ w(k') n_k n_k' for k < k'
        if !last_colour {
            t.push((Ch::Start, Ch::AccNOpen, NUM, 1.0));
            t.push((Ch::AccNOpen, Ch::AccNOpen, ID, 1.0));
        } else {
            t.push((Ch::Start, Ch::AccN, NUM, 1.0));
            t.push((Ch::AccNOpen, Ch::AccN, ID, 1.0));
        }
        t.push((Ch::AccN, Ch::AccN, ID, 1.0));
        if k + 2 <= n {
            t.push((Ch::AccN, Ch::Done, NUM, -w / 3.0));
        }
        // ---- the bilinear accumulators: w(k') Σ_{cc'} B^{(k)}_{cc'} B^{(k')}_{c'c}
        for a in 0..COLOURS {
            for b in 0..COLOURS {
                // persistence
                t.push((Ch::AccB(a, b), Ch::AccB(a, b), ID, 1.0));
                if !last_colour {
                    t.push((Ch::AccBOpen(a, b), Ch::AccBOpen(a, b), ID, 1.0));
                } else {
                    t.push((Ch::AccBOpen(a, b), Ch::AccB(a, b), ID, 1.0));
                }
                if a == b {
                    // B_{aa} = n_a: open as accumulation at colour a
                    if c == a {
                        let target = if last_colour { Ch::AccB(a, b) } else { Ch::AccBOpen(a, b) };
                        t.push((Ch::Start, target, NUM, 1.0));
                        // close: w(k') n_a at colour a of a later group
                        if k + 2 <= n {
                            t.push((Ch::AccB(a, b), Ch::Done, NUM, w));
                        }
                    }
                } else {
                    // B_{ab} = c†_a c_b spans colours lo..hi within a group
                    let (lo, hi) = (a.min(b), a.max(b));
                    let open_op = if a < b { SPLUS } else { SMINUS }; // the operator at the lower colour
                    let close_op = if a < b { SMINUS } else { SPLUS }; // at the higher colour
                    if c == lo {
                        t.push((Ch::Start, Ch::Open(a, b), open_op, 1.0));
                    }
                    if c > lo && c < hi {
                        t.push((Ch::Open(a, b), Ch::Open(a, b), ZJW, 1.0));
                    }
                    if c == hi {
                        let target = if last_colour { Ch::AccB(a, b) } else { Ch::AccBOpen(a, b) };
                        t.push((Ch::Open(a, b), target, close_op, 1.0));
                    }
                    // closing against the accumulator: B^{(k')}_{ba} = c†_b c_a, lower colour first
                    if k + 2 <= n {
                        let (clo, chi) = (a.min(b), a.max(b));
                        let cl_open = if b < a { SPLUS } else { SMINUS };
                        let cl_close = if b < a { SMINUS } else { SPLUS };
                        if c == clo {
                            t.push((Ch::AccB(a, b), Ch::Close(a, b), cl_open, 1.0));
                        }
                        if c > clo && c < chi {
                            t.push((Ch::Close(a, b), Ch::Close(a, b), ZJW, 1.0));
                        }
                        if c == chi {
                            t.push((Ch::Close(a, b), Ch::Done, cl_close, w));
                        }
                    }
                }
            }
        }
        t
    }

    /// The MPO for sector `n_q` (the penalty pins it): first site `1×D` (Start row),
    /// interior `D×D`, last site `D×1` (Done column).
    pub fn mpo(&self, n_q: usize) -> Mpo {
        let chs = channels();
        let d = chs.len();
        // one lookup, built once, instead of a linear scan per transition per site
        let lookup: std::collections::HashMap<Ch, usize> = chs.iter().enumerate().map(|(i, &c)| (c, i)).collect();
        let idx = |ch: Ch| *lookup.get(&ch).expect("channel");
        let l = self.sites();
        let mut sites = Vec::with_capacity(l);
        for j in 0..l {
            let (d_l, d_r) = (if j == 0 { 1 } else { d }, if j == l - 1 { 1 } else { d });
            let mut site = MpoSite::zeros(d_l, d_r);
            for (from, to, block, coeff) in self.transitions(j, n_q) {
                if j == 0 && from != Ch::Start {
                    continue;
                }
                if j == l - 1 && to != Ch::Done {
                    continue;
                }
                let cl = if j == 0 { 0 } else { idx(from) };
                let cr = if j == l - 1 { 0 } else { idx(to) };
                let b = scaled(block, coeff);
                for bra in 0..2 {
                    for ket in 0..2 {
                        if b[bra][ket] != 0.0 {
                            let v = site.get(cl, cr, ket, bra) + b[bra][ket];
                            site.set(cl, cr, ket, bra, v);
                        }
                    }
                }
            }
            sites.push(site);
        }
        Mpo { sites }
    }

    /// A `χ = 1` product start with `n_q` quarks: the sea sites (odd groups) filled first,
    /// then extra quarks on the even groups from the left — every mode of a group filled
    /// together, so the start is colour-neutral group by group where it can be.
    pub fn product_start(&self, n_q: usize) -> Vec<TensorSite> {
        let l = self.sites();
        let mut occ = vec![false; l];
        let mut left = n_q;
        for k in (0..self.n).filter(|k| k % 2 == 1).chain((0..self.n).filter(|k| k % 2 == 0)) {
            for c in 0..COLOURS {
                if left > 0 {
                    occ[COLOURS * k + c] = true;
                    left -= 1;
                }
            }
        }
        assert_eq!(left, 0, "{n_q} quarks do not fit {l} modes");
        occ.iter()
            .map(|&o| {
                let mut t = TensorSite::zeros(1, 1);
                t.set(usize::from(o), 0, 0, 1.0);
                t
            })
            .collect()
    }

    /// The colour sector of `n_q` quarks for the SYMMETRIC sweep (amendment A1): site `j` is
    /// colour `j mod 3`, and the boundary carries equal counts — the Cartan-neutral block,
    /// exact for every sector this instrument runs (`holon-chem::qcd2`, the same lanes).
    pub fn sector(&self, n_q: usize) -> Result<Sector, SymRefusal> {
        let mut total = ZERO_CHARGE;
        if n_q % COLOURS != 0 {
            return Err(SymRefusal::StartOutsideSector { start: [n_q as i32, 0, 0, 0], total });
        }
        for c in 0..COLOURS {
            total[c] = (n_q / COLOURS) as i32;
        }
        let site_charge = (0..self.sites())
            .map(|j| {
                let mut e = ZERO_CHARGE;
                e[j % COLOURS] = 1;
                e
            })
            .collect();
        Ok(Sector { site_charge, total })
    }

    /// The symmetric sweep (E7) from a product start with the given occupations, on the
    /// UNPENALISED Hamiltonian (`λ = 0`), returning the result and the bond labels so a
    /// χ-ladder continues from them. `ignore_labels` is plant (iv)'s mutant.
    pub fn ground_energy_sym_from(
        &self,
        occ: &[bool],
        n_q: usize,
        cfg: &SymConfig,
        from: Option<(Vec<TensorSite>, Labels)>,
    ) -> Result<(DmrgResult, Labels), SymRefusal> {
        let sector = self.sector(n_q)?;
        let unpenalised = Qcd2 { n: self.n, x: self.x, lam: 0.0 };
        let (tensors, labels) = match from {
            Some(s) => s,
            None => {
                let tensors: Vec<TensorSite> = occ
                    .iter()
                    .map(|&o| {
                        let mut t = TensorSite::zeros(1, 1);
                        t.set(usize::from(o), 0, 0, 1.0);
                        t
                    })
                    .collect();
                let labels = labels_of_product(occ, &sector);
                (tensors, labels)
            }
        };
        dmrg_sweep_sym(&unpenalised.mpo(n_q), tensors, labels, &sector, cfg)
    }

    /// The symmetric sweep of baryon-number sector `b` from the seeded random labelled start
    /// (`symmetric::random_start`, 256 labels, seed 7 — every reachable sector for N ≤ 10; the
    /// product start is a fixed point of the labelled two-site update on this chain, see
    /// `random_start`'s header).
    pub fn ground_energy_sym(&self, b: i32, chi: usize, max_sweeps: usize, ignore_labels: bool) -> Result<(DmrgResult, Labels), SymRefusal> {
        let n_q = self.quarks(b);
        let sector = self.sector(n_q)?;
        let start = random_start(&sector, 256, 7);
        let mut cfg = SymConfig::amendment(chi, max_sweeps);
        cfg.ignore_labels = ignore_labels;
        self.ground_energy_sym_from(&[], n_q, &cfg, Some(start))
    }

    /// Ground state of baryon-number sector `b` on the RETIRED penalised arm (kept as the
    /// amendment's evidence and for plant (iv)'s comparison); the penalty's constant `λ n_q²`
    /// is added back so `energy` is the sector's own `W`.
    pub fn ground_energy(&self, b: i32, chi: usize, max_sweeps: usize, sweep_tol: f64) -> Result<DmrgResult, Refusal> {
        let n_q = self.quarks(b);
        let config = DmrgConfig { chi_max: chi, max_sweeps, sweep_tol, policy: RefusalPolicy::Silent };
        let mut r = dmrg_sweep(&self.mpo(n_q), self.product_start(n_q), &config)?;
        r.energy += self.lam * (n_q as f64) * (n_q as f64);
        for e in r.energy_history.iter_mut() {
            *e += self.lam * (n_q as f64) * (n_q as f64);
        }
        Ok(r)
    }
}


// ------------------------------------------------------------------ the χ-ladder as a library (A2)

/// One sector's χ-ladder, checkpointed and resumable: what the host driver
/// (`examples/qcd2_dmrg.rs`) and the device driver (`holon-gpu/examples/qcd2_sym_device.rs`)
/// both run, so there is one ladder and two executors.
#[derive(Clone, Debug)]
pub struct LadderOpts {
    pub n: usize,
    pub x: f64,
    pub b: i32,
    pub chis: Vec<usize>,
    pub sweeps: usize,
    pub mixing: f64,
    pub reseed: bool,
    pub mutant: bool,
    /// Compute the exact variance of every rung's final state (priced; a refusal is printed).
    pub variance: bool,
    /// Where the rung rows, the per-sweep state and each rung's final state live. With it a
    /// killed run resumes from its last completed sweep; without it nothing is written.
    pub ckpt_dir: Option<std::path::PathBuf>,
    pub seed: u64,
    pub label_cap: usize,
}

impl LadderOpts {
    pub fn tag(&self) -> String {
        format!("x{}_N{}_B{}{}", self.x, self.n, self.b, if self.mutant { "_mutant" } else { "" })
    }
}

/// Run the ladder; returns the JSON document for the sector (every rung, completed earlier
/// or now). `backend` is the device, or `None` for the host loops.
pub fn run_sym_ladder(o: &LadderOpts, backend: Option<std::sync::Arc<dyn crate::blocks::TwoSiteBackend>>) -> String {
    use crate::symmetric::{random_start, reseed_labels, SweepCheckpoint, SymConfig};
    use std::time::Instant;
    let t0 = Instant::now();
    let q = Qcd2::new(o.n, o.x);
    let n_q = q.quarks(o.b);
    let sector = q.sector(n_q).expect("a Cartan-neutral sector");
    let unpen = Qcd2 { n: o.n, x: o.x, lam: 0.0 };
    let mpo = unpen.mpo(n_q);
    let tag = o.tag();
    let class = if backend.is_some() { "device" } else { "host" };
    // rows already completed, from the checkpoint directory
    let rows_path = o.ckpt_dir.as_ref().map(|d| d.join(format!("{tag}.rungs.jsonl")));
    let mut rows: Vec<String> = rows_path
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|t| t.lines().filter(|l| !l.trim().is_empty()).map(String::from).collect())
        .unwrap_or_default();
    let done_chi = |rows: &[String]| -> Vec<usize> {
        rows.iter()
            .filter_map(|r| r.split("\"chi\":").nth(1).and_then(|s| s.split(',').next()).and_then(|s| s.trim().parse().ok()))
            .collect()
    };
    let completed = done_chi(&rows);
    let mut state: Option<(Vec<crate::mps::TensorSite>, crate::symmetric::Labels)> = None;
    let mut prev_chi: Option<usize> = None;
    for &chi in &o.chis {
        if completed.contains(&chi) {
            prev_chi = Some(chi);
            continue;
        }
        let t1 = Instant::now();
        let mut cfg = SymConfig::amendment(chi, o.sweeps);
        cfg.ignore_labels = o.mutant;
        cfg.mixing = o.mixing;
        cfg.backend = backend.clone();
        let state_path = o.ckpt_dir.as_ref().map(|d| d.join(format!("{tag}_chi{chi}.state")));
        cfg.checkpoint = state_path.clone();
        // the start of this rung: a checkpoint of THIS rung, else the previous rung's final
        // state (re-seeded if asked), else the seeded random labelled start
        let resume = state_path.as_ref().filter(|p| p.exists()).and_then(|p| SweepCheckpoint::load(p).ok());
        let resumed_from = resume.as_ref().map_or(0, |c| c.sweeps_done);
        let (start_t, start_l, restored) = if let Some(c) = &resume {
            (c.tensors.clone(), c.labels.clone(), 0usize)
        } else if let Some((t, l)) = state.take() {
            let (mut t, mut l) = (t, l);
            let restored = if o.reseed { reseed_labels(&mut t, &mut l, &sector, o.label_cap, 1e-3, o.seed + prev_chi.unwrap_or(0) as u64) } else { 0 };
            (t, l, restored)
        } else if let Some(pc) = prev_chi {
            // resuming after the previous rung completed in an earlier invocation
            let done = o.ckpt_dir.as_ref().unwrap().join(format!("{tag}_chi{pc}.done.state"));
            let c = SweepCheckpoint::load(&done).unwrap_or_else(|e| panic!("rung chi {pc} is recorded complete but its final state {} is unreadable: {e}", done.display()));
            let (mut t, mut l) = (c.tensors, c.labels);
            let restored = if o.reseed { reseed_labels(&mut t, &mut l, &sector, o.label_cap, 1e-3, o.seed + pc as u64) } else { 0 };
            (t, l, restored)
        } else {
            let (t, l) = random_start(&sector, o.label_cap, o.seed);
            (t, l, 0)
        };
        match crate::symmetric::dmrg_sweep_sym_resume(&mpo, start_t, start_l, &sector, &cfg, resume) {
            Ok((r, labels)) => {
                let max_dw = r.discarded_weight.iter().cloned().fold(0.0f64, f64::max);
                // THE ERROR BAR. The exact variance is priced and refuses above its lease
                // (18.5 GB at chi=512, 74 GB at 1024), and every rung of the volume ladder
                // sits above that — so the two-site variance is computed on EVERY rung and
                // the exact one beside it wherever it fits. Their ratio on the rungs where
                // both exist is the calibration (measured 0.46-0.73 at N=8; the two-site one
                // is an approximation on this long-range H, exact only for nearest-neighbour
                // interactions, and is reported as `variance_2s`, never as `variance`).
                let var = if o.variance {
                    let mut t = r.tensors.clone();
                    let n2 = crate::observables::norm_squared(&t);
                    if n2 > 0.0 {
                        let f = 1.0 / n2.sqrt();
                        for v in t[0].data.iter_mut() { *v *= f; }
                    }
                    let exact = match crate::variance::energy_variance(&t, &mpo) {
                        Ok((_, _, v)) => format!(",\"variance\":{v:.6e}"),
                        Err(e) => format!(",\"variance_refused\":\"{e}\""),
                    };
                    let t0 = std::time::Instant::now();
                    let (d2s, one, two) = crate::variance2::two_site_variance(&t, &mpo);
                    format!("{exact},\"variance_2s\":{d2s:.6e},\"variance_2s_1s\":{one:.3e},\"variance_2s_2s\":{two:.3e},\"variance_2s_seconds\":{:.1}", t0.elapsed().as_secs_f64())
                } else {
                    String::new()
                };
                let row = format!(
                    "{{\"chi\":{chi},\"energy\":{:.12},\"sweeps\":{},\"lanczos_iterations\":{},\"converged\":{},\"exit\":\"{}\",\"worst_residual\":{:.3e},\"max_discarded\":{:.3e},\"max_bond\":{},\"seconds\":{:.1},\"class\":\"{class}\",\"mixing\":{:.1e},\"reseeded\":{restored},\"resumed_from_sweep\":{resumed_from}{var}}}",
                    r.energy, r.sweeps_used, r.lanczos_iterations_total, r.converged,
                    if r.converged { "converged" } else { "sweep_cap" },
                    r.worst_lanczos_residual, max_dw, r.bond_dims.iter().cloned().max().unwrap_or(0), t1.elapsed().as_secs_f64(), o.mixing
                );
                if let Some(d) = &o.ckpt_dir {
                    let done = SweepCheckpoint { tensors: r.tensors.clone(), labels: labels.clone(), sweeps_done: r.sweeps_used, prev_energy: f64::INFINITY, last_energy: r.energy, energy_history: r.energy_history.clone(), discarded: r.discarded_weight.clone(), spectrum_floor: r.spectrum_floor.clone(), bond_energy: r.bond_energy.clone(), bond_energy_prev: Vec::new(), bond_energy_delta: r.bond_energy_delta.clone(), site_delta: r.site_delta.clone(), block_mass: r.block_mass.clone(), worst_resid: r.worst_lanczos_residual, iters_total: r.lanczos_iterations_total, bonds_skipped: r.bonds_skipped, converged: r.converged };
                    done.save(&d.join(format!("{tag}_chi{chi}.done.state"))).expect("the rung's final state is written");
                    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(rows_path.as_ref().unwrap()).expect("the rung rows file");
                    use std::io::Write;
                    writeln!(f, "{row}").expect("a rung row is appended");
                    let _ = std::fs::remove_file(d.join(format!("{tag}_chi{chi}.state")));
                }
                rows.push(row);
                state = Some((r.tensors, labels));
                prev_chi = Some(chi);
            }
            Err(e) => {
                rows.push(format!("{{\"chi\":{chi},\"refused\":\"{e}\"}}"));
                break;
            }
        }
    }
    format!(
        "{{\"n\":{},\"x\":{},\"b\":{},\"n_q\":{n_q},\"arm\":\"{}\",\"class\":\"{class}\",\"threads\":{},\"rungs\":[{}],\"seconds\":{:.1}}}",
        o.n, o.x, o.b, if o.mutant { "mutant-labels-ignored" } else { "symmetric-a2" }, crate::mps::threads(), rows.join(","), t0.elapsed().as_secs_f64()
    )
}
