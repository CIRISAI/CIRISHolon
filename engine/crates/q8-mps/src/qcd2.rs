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

    /// Ground state of baryon-number sector `b`; the penalty's constant `λ n_q²` is added
    /// back so `energy` is the sector's own `W`.
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
