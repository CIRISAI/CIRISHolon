//! GF1's instrument — `conformance/crystal/GF1_PREREG.md` §2: the stabilizer 2-Rényi entropy
//! (magic) of a real MPS, EXACT, with the brute-force referee the prereg names and the local
//! Clifford minimiser it stakes.
//!
//! ```text
//! M₂ = −log₂ ( Σ_P ⟨P⟩⁴ / 2^N ),   P over all 4^N Pauli strings   (Leone–Oliviero–Hamma 2022)
//! ```
//!
//! **The Pauli-replica contraction.** `Σ_P ⟨P⟩⁴ = ⟨ψ^{⊗4}| (Σ_a σ_a^{⊗4})^{⊗N} |ψ^{⊗4}⟩` is
//! a product-operator expectation on four replicas of the state (Haug & Piroli 2023; Lami &
//! Collura 2023). Per site the replica transfer is `T_j = Σ_a E_a^{⊗4}` with
//! `E_a = Σ_{ss'} (σ_a)_{ss'} A^s ⊗ A^{s'}` the site's Pauli transfer matrix, one `χ² × χ²` block
//! per Pauli; the replica vector carries EIGHT bond legs (four ket, four bra) — `χ⁸` doubles —
//! and every site costs `64 χ⁹` multiply-adds. Pauli `Y` enters as the real matrix `iY`, whose
//! fourth power is `Y`'s: the whole contraction stays in `f64`, the type `TensorSite` carries.
//!
//! PRICED, NOT ASSUMED — and the prereg's price is wrong. §2 writes `O(N · 4 · χ⁴)` and
//! "feasible at χ ≤ 64": `4 χ⁴` per site is the SIZE of the Pauli MPS (four `χ² × χ²` blocks),
//! not the cost of its fourth-power sum. The norm of the `χ²` Pauli MPS is `Σ_P ⟨P⟩² = 2^N`, the
//! purity, identically; the FOURTH powers need two more copies, `χ⁸` memory and `χ⁹` time, and
//! that is what this module builds. It is exact and admits `χ ≤ 11` under the default lease
//! (`χ⁸ · 32` bytes: 0.5 GB at χ = 8, 3.2 GB at χ = 10, 6.9 GB at χ = 11, 13.8 GB at χ = 12).
//! Above that it refuses
//! by name; the prereg's ladder at `χ = 40, 64` cannot be read by this instrument, and the
//! named exits (a compressed Pauli MPS, Tarabunga–Tirrito–Bañuls–Dalmonte 2024; or perfect
//! Pauli sampling with a statistical bar, Lami–Collura 2023) are cited, not built.
//!
//! **The local Clifford minimiser** (`sre2_local_min`) is built as staked: coordinate descent
//! over the 24 single-qubit Cliffords per site, sites in order, a fixed enumeration. What it
//! measures is stated here rather than discovered later: a Clifford `C` sends every Pauli to a
//! signed Pauli, `C†σ_aC = ±σ_{π(a)}`, so a site's transfer becomes `Σ_a (±E_{π(a)})^{⊗4}` —
//! the SAME sum, the sign killed by the fourth power, the terms merely permuted. `M₂` is
//! invariant under every product of single-site Cliffords (it is invariant under the whole
//! Clifford group), so the staked `M₂^loc` equals `M₂` to the rounding of a re-ordered
//! four-term sum. The minimiser reports that honestly; it does not manufacture a difference.
//!
//! Nothing here touches `dmrg.rs`, `variance.rs` or `schwinger.rs`.

use crate::mps::{self, TensorSite};
use crate::observables;
use crate::ops::Op2;

/// Pauli channel indices, the fixed order every sum in this module runs in.
pub const PAULI_I: usize = 0;
pub const PAULI_X: usize = 1;
pub const PAULI_Y: usize = 2;
pub const PAULI_Z: usize = 3;

/// The four Paulis as REAL `2×2` matrices: `I`, `X`, `iY = [[0,1],[−1,0]]`, `Z`. `iY` stands in
/// for `Y` everywhere a fourth power (or an even power) is taken: `(iY)^{⊗4}` and `Y^{⊗4}` are
/// the same operator, and `⟨P⟩⁴` is unchanged for every string.
pub const PAULI_REAL: [Op2; 4] = [
    [[1.0, 0.0], [0.0, 1.0]],
    [[0.0, 1.0], [1.0, 0.0]],
    [[0.0, 1.0], [-1.0, 0.0]],
    [[1.0, 0.0], [0.0, -1.0]],
];

/// The brute-force referee's ceiling: `4^10 ≈ 10⁶` expectation values.
pub const BRUTE_MAX_SITES: usize = 10;

/// Why a reading was refused rather than produced.
#[derive(Debug, Clone, PartialEq)]
pub enum MagicError {
    /// No sites.
    Empty,
    /// `data.len() ≠ 2·χ_l·χ_r`: the site's local dimension is not 2 (the prereg's Paulis are
    /// qubit Paulis; local dimension above 2 needs generalised Paulis and a second prereg, §7).
    LocalDimension { site: usize, data_len: usize, chi_l: usize, chi_r: usize },
    /// Bond `site → site+1` has two sizes.
    BondMismatch { site: usize, chi_r: usize, next_chi_l: usize },
    /// The chain does not close on trivial bonds at both ends.
    OpenBoundary { chi_l_first: usize, chi_r_last: usize },
    /// `⟨ψ|ψ⟩ = 0`, or the Pauli sum collapsed to zero (a normalised state has `Σ_P⟨P⟩⁴ ≥ 1`).
    ZeroNorm,
    /// The brute-force referee is admitted to `N ≤ 10` only.
    BruteTooLarge { n: usize, max: usize },
    /// The replica vectors would exceed the lease (`Q8_MAGIC_LEASE_BYTES`, default 8 GiB).
    Price { bytes: u64, lease_bytes: u64 },
    /// A Clifford with a complex matrix cannot be applied to a real tensor.
    ComplexClifford { index: usize },
    /// `sre2_box`: the box is longer than the chain.
    BoxTooLong { box_len: usize, n: usize },
}

impl std::fmt::Display for MagicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MagicError::Empty => write!(f, "no sites"),
            MagicError::LocalDimension { site, data_len, chi_l, chi_r } => write!(
                f,
                "site {site}: data length {data_len} is not 2·{chi_l}·{chi_r} — local dimension is not 2, refused"
            ),
            MagicError::BondMismatch { site, chi_r, next_chi_l } => {
                write!(f, "bond {site}: chi_r {chi_r} against the next site's chi_l {next_chi_l}")
            }
            MagicError::OpenBoundary { chi_l_first, chi_r_last } => {
                write!(f, "the chain must close on trivial bonds: chi_l[0] = {chi_l_first}, chi_r[N-1] = {chi_r_last}")
            }
            MagicError::ZeroNorm => write!(f, "the state has zero norm"),
            MagicError::BruteTooLarge { n, max } => {
                write!(f, "brute enumeration of 4^{n} Pauli strings refused: admitted to N ≤ {max}")
            }
            MagicError::Price { bytes, lease_bytes } => write!(
                f,
                "the Pauli-replica contraction would allocate {:.2} GB against a lease of {:.2} GB: refused, not estimated",
                *bytes as f64 / 1e9,
                *lease_bytes as f64 / 1e9
            ),
            MagicError::ComplexClifford { index } => {
                write!(f, "Clifford {index} has a complex matrix and cannot rotate a real tensor")
            }
            MagicError::BoxTooLong { box_len, n } => write!(f, "a box of {box_len} sites on a chain of {n}"),
        }
    }
}

impl std::error::Error for MagicError {}

/// The lease the replica contraction may spend, bytes: `Q8_MAGIC_LEASE_BYTES` or 8 GiB.
pub fn lease_bytes() -> u64 {
    std::env::var("Q8_MAGIC_LEASE_BYTES").ok().and_then(|v| v.parse().ok()).unwrap_or(8u64 << 30)
}

fn check(tensors: &[TensorSite]) -> Result<(), MagicError> {
    if tensors.is_empty() {
        return Err(MagicError::Empty);
    }
    for (j, t) in tensors.iter().enumerate() {
        if t.data.len() != 2 * t.chi_l * t.chi_r || t.chi_l == 0 || t.chi_r == 0 {
            return Err(MagicError::LocalDimension { site: j, data_len: t.data.len(), chi_l: t.chi_l, chi_r: t.chi_r });
        }
        if j + 1 < tensors.len() && t.chi_r != tensors[j + 1].chi_l {
            return Err(MagicError::BondMismatch { site: j, chi_r: t.chi_r, next_chi_l: tensors[j + 1].chi_l });
        }
    }
    let (first, last) = (tensors[0].chi_l, tensors[tensors.len() - 1].chi_r);
    if first != 1 || last != 1 {
        return Err(MagicError::OpenBoundary { chi_l_first: first, chi_r_last: last });
    }
    Ok(())
}

fn chi_max(tensors: &[TensorSite]) -> usize {
    tensors.iter().map(|t| t.chi_l.max(t.chi_r)).max().unwrap_or(1)
}

/// Peak bytes one `sre2` reading allocates: the replica vector, its successor and the two
/// ping-pong pair buffers — four `χ⁸` vectors at the largest bond.
pub fn price_bytes(tensors: &[TensorSite]) -> u64 {
    4 * (chi_max(tensors) as u64).pow(8) * 8
}

/// Peak bytes `sre2_local_min` allocates: `sre2`'s working set plus the stack of right
/// environments (one per bond) and the four per-channel left vectors.
pub fn price_bytes_local_min(tensors: &[TensorSite]) -> u64 {
    let stack: u64 = tensors.iter().map(|t| (t.chi_r as u64).pow(8) * 8).sum();
    price_bytes(tensors) + stack + 4 * (chi_max(tensors) as u64).pow(8) * 8
}

// ------------------------------------------------------------------ the replica contraction

/// A site's Pauli transfer, laid out for the leg routines. `arow[l][s][r] = A^s[l][r]`
/// (the ket leg) and `atil[a][s][l][r] = Σ_{s'} σ_a[s][s'] A^{s'}[l][r]` (the bra leg with the
/// Pauli absorbed), for one direction of contraction: `n_in` legs are consumed, `n_out` produced.
struct SiteTables {
    n_in: usize,
    n_out: usize,
    arow: Vec<f64>,
    atil: [Vec<f64>; 4],
}

impl SiteTables {
    /// `forward`: consume the left legs, produce the right (a left environment growing right).
    /// Otherwise the mirror.
    fn new(t: &TensorSite, forward: bool) -> Self {
        let (n_in, n_out) = if forward { (t.chi_l, t.chi_r) } else { (t.chi_r, t.chi_l) };
        let a = |s: usize, i: usize, o: usize| if forward { t.get(s, i, o) } else { t.get(s, o, i) };
        let mut arow = vec![0.0; n_in * 2 * n_out];
        for i in 0..n_in {
            for s in 0..2 {
                for o in 0..n_out {
                    arow[(i * 2 + s) * n_out + o] = a(s, i, o);
                }
            }
        }
        let atil = std::array::from_fn(|p| {
            let sigma = PAULI_REAL[p];
            let mut v = vec![0.0; 2 * n_in * n_out];
            for s in 0..2 {
                for i in 0..n_in {
                    for o in 0..n_out {
                        let mut acc = 0.0;
                        for sp in 0..2 {
                            let w = sigma[s][sp];
                            if w != 0.0 {
                                acc += w * a(sp, i, o);
                            }
                        }
                        v[(s * n_in + i) * n_out + o] = acc;
                    }
                }
            }
            v
        });
        Self { n_in, n_out, arow, atil }
    }
}

/// Below this many output rows a leg runs on the calling thread: the split's cost would
/// exceed the work.
const PAR_MIN_ROWS: usize = 1 << 12;

/// Output rows a leg kernel handles per pass over the site table (a few KB, cache-resident).
const LEG_BLOCK: usize = 32;

/// Run `f(row_lo, rows_chunk)` over `out` split into contiguous row chunks of `row_len` doubles,
/// on `mps::threads()` threads. Every row is computed by exactly one thread with the same
/// arithmetic in the same order, so the result is bit-identical at every thread count.
fn par_rows<F>(out: &mut [f64], row_len: usize, f: F)
where
    F: Fn(usize, &mut [f64]) + Sync,
{
    let rows = out.len() / row_len;
    let threads = mps::threads().min(rows.max(1));
    if threads <= 1 || rows < PAR_MIN_ROWS {
        f(0, out);
        return;
    }
    let per = rows.div_ceil(threads);
    std::thread::scope(|sc| {
        for (k, chunk) in out.chunks_mut(per * row_len).enumerate() {
            let f = &f;
            sc.spawn(move || f(k * per, chunk));
        }
    });
}

/// One pair of legs converted, for every output row: the pair's ket leg (the OUTERMOST leg of
/// `v`, size `n_in`) is contracted with `A^s` keeping `s` open, then the bra leg (the next leg)
/// and `s` with `Σ_{s'} σ_a[s][s'] A^{s'}`; the new pair `(r, r')` lands innermost:
///
/// ```text
/// out[j][r][r'] (+)= Σ_{l'} Σ_s ( Σ_l v[l][l'][j] · arow[l][s][r] ) · atil_a[s][l'][r']
/// ```
///
/// with `j` over the `rest2` untouched legs. Both legs run per block of `LEG_BLOCK` rows with
/// the intermediate in cache, so the traffic is one read of `v` and one write of `out`. Every
/// output element is summed in one fixed order (`l` inner, then `l'` outer with `s` inside),
/// on whichever thread owns its row — bit-identical at every thread count.
fn pair(v: &[f64], n_in: usize, rest2: usize, tabs: &SiteTables, a: usize, out: &mut [f64], accumulate: bool) {
    let n_out = tabs.n_out;
    let (w, vw) = (n_out * n_out, 2 * n_out);
    debug_assert_eq!(v.len(), n_in * n_in * rest2);
    debug_assert_eq!(out.len(), rest2 * w);
    let (arow, atil) = (&tabs.arow, &tabs.atil[a]);
    par_rows(out, w, |lo, chunk| {
        let mut mid = vec![0.0; n_in * LEG_BLOCK * vw];
        for (bi, block) in chunk.chunks_mut(LEG_BLOCK * w).enumerate() {
            let j0 = lo + bi * LEG_BLOCK;
            let rows = block.len() / w;
            // Leg 1: mid[l'][b][s][r] = Σ_l v[l][l'][j0 + b] · arow[l][s][r].
            let mid = &mut mid[..n_in * rows * vw];
            mid.iter_mut().for_each(|x| *x = 0.0);
            for lp in 0..n_in {
                let midp = &mut mid[lp * rows * vw..(lp + 1) * rows * vw];
                for l in 0..n_in {
                    let trow = &arow[l * vw..(l + 1) * vw];
                    let base = (l * n_in + lp) * rest2 + j0;
                    for (orow, &x) in midp.chunks_exact_mut(vw).zip(&v[base..base + rows]) {
                        if x == 0.0 {
                            continue;
                        }
                        for (o, t) in orow.iter_mut().zip(trow) {
                            *o += x * *t;
                        }
                    }
                }
            }
            // Leg 2: out[b][r][r'] (+)= Σ_{l', s} mid[l'][b][s][r] · atil[s][l'][r'].
            if !accumulate {
                block.iter_mut().for_each(|x| *x = 0.0);
            }
            for lp in 0..n_in {
                let midp = &mid[lp * rows * vw..(lp + 1) * rows * vw];
                for (o, vin) in block.chunks_exact_mut(w).zip(midp.chunks_exact(vw)) {
                    for s in 0..2 {
                        let trow = &atil[(s * n_in + lp) * n_out..(s * n_in + lp + 1) * n_out];
                        for (orow, &x) in o.chunks_exact_mut(n_out).zip(&vin[s * n_out..(s + 1) * n_out]) {
                            if x == 0.0 {
                                continue;
                            }
                            for (oo, t) in orow.iter_mut().zip(trow) {
                                *oo += x * *t;
                            }
                        }
                    }
                }
            }
        }
    });
}

/// Two ping-pong buffers for the pairs between the first and the last, reused across sites so
/// a sweep does not re-fault `χ⁸` pages at every site.
#[derive(Default)]
struct Scratch {
    a: Vec<f64>,
    b: Vec<f64>,
}

/// One site of the `n_pairs`-replica transfer: `out = Σ_{a ∈ channels} E_a^{⊗n_pairs} v`, with
/// `v` on `2·n_pairs` legs of size `n_in` (layout `k₁ b₁ k₂ b₂ …`, leg 1 outermost) and `out` on
/// the same legs of size `n_out`. Each pair is converted by `pair`; the converted pair lands
/// innermost, so after `n_pairs` conversions the layout is the input's again. The channel sum
/// runs in the order `channels` lists — that order, and nothing else, is what a Clifford frame
/// changes.
fn apply_site(v: &[f64], tabs: &SiteTables, channels: &[usize], n_pairs: usize, out: &mut Vec<f64>, sc: &mut Scratch) {
    let (n_in, n_out) = (tabs.n_in, tabs.n_out);
    let n_legs = 2 * n_pairs;
    debug_assert_eq!(v.len(), n_in.pow(n_legs as u32));
    out.clear();
    out.resize(n_out.pow(n_legs as u32), 0.0);
    for (ci, &a) in channels.iter().enumerate() {
        for p in 0..n_pairs {
            let legs_left = n_legs - 2 * p;
            let rest2 = n_in.pow((legs_left - 2) as u32) * n_out.pow((2 * p) as u32);
            let last = p + 1 == n_pairs;
            let (src, dst): (&[f64], &mut Vec<f64>) = match (p == 0, p % 2 == 0) {
                (true, _) => (v, if last { &mut *out } else { &mut sc.a }),
                (false, true) => (&sc.b, if last { &mut *out } else { &mut sc.a }),
                (false, false) => (&sc.a, if last { &mut *out } else { &mut sc.b }),
            };
            if !last {
                dst.clear();
                dst.resize(rest2 * n_out * n_out, 0.0);
            }
            pair(src, n_in, rest2, tabs, a, dst, last && ci > 0);
        }
    }
}

/// A replica vector with its scale carried in `log₂`, so a chain of any length neither
/// overflows nor underflows: the true vector is `v · 2^log2`.
#[derive(Clone)]
struct Scaled {
    v: Vec<f64>,
    log2: f64,
}

impl Scaled {
    fn unit() -> Self {
        Self { v: vec![1.0], log2: 0.0 }
    }

    /// Divide by the largest magnitude and book it; a zero vector is the `ZeroNorm` refusal.
    fn renormalise(&mut self) -> Result<(), MagicError> {
        let m = self.v.iter().fold(0.0f64, |m, x| m.max(x.abs()));
        if m == 0.0 || !m.is_finite() {
            return Err(MagicError::ZeroNorm);
        }
        let inv = 1.0 / m;
        self.v.iter_mut().for_each(|x| *x *= inv);
        self.log2 += m.log2();
        Ok(())
    }
}

/// `Σ a·b` by pairwise summation (blocks of `DOT_LEAF` summed in order, then halved), so the
/// rounding of a `χ⁸`-term overlap is `O(ε log n)` rather than `O(ε √n)` — the sequential sum
/// left the minimiser's reading 8e-13 under `sre2`'s at `χ⁸ = 1.7 × 10⁷` terms. Serial and
/// deterministic.
fn dot(a: &[f64], b: &[f64]) -> f64 {
    const DOT_LEAF: usize = 1024;
    debug_assert_eq!(a.len(), b.len());
    if a.len() <= DOT_LEAF {
        return a.iter().zip(b).map(|(x, y)| x * y).sum();
    }
    let mid = a.len() / 2;
    dot(&a[..mid], &b[..mid]) + dot(&a[mid..], &b[mid..])
}

/// `log₂ Σ_P Π_j ⟨P_j⟩ …` — the `n_pairs`-replica Pauli sum over the strings whose site-`j`
/// Pauli ranges over `channels_at(j)`, RAW (the state's norm is not divided out).
fn log2_replica_sum(
    tensors: &[TensorSite],
    n_pairs: usize,
    channels_at: impl Fn(usize) -> &'static [usize],
) -> Result<f64, MagicError> {
    let mut cur = Scaled::unit();
    let mut next = Vec::new();
    let mut sc = Scratch::default();
    for (j, t) in tensors.iter().enumerate() {
        let tabs = SiteTables::new(t, true);
        apply_site(&cur.v, &tabs, channels_at(j), n_pairs, &mut next, &mut sc);
        std::mem::swap(&mut cur.v, &mut next);
        cur.renormalise()?;
    }
    debug_assert_eq!(cur.v.len(), 1);
    let last = cur.v[0];
    if last <= 0.0 {
        return Err(MagicError::ZeroNorm);
    }
    Ok(cur.log2 + last.log2())
}

const ALL_CHANNELS: [usize; 4] = [PAULI_I, PAULI_X, PAULI_Y, PAULI_Z];
const IDENTITY_ONLY: [usize; 1] = [PAULI_I];

/// `log₂ ⟨ψ|ψ⟩`, by the one-pair identity transfer with the same scaling.
fn log2_norm2(tensors: &[TensorSite]) -> Result<f64, MagicError> {
    log2_replica_sum(tensors, 1, |_| &IDENTITY_ONLY)
}

// ------------------------------------------------------------------ the instrument

/// **M₂**, exact, by the Pauli-replica contraction: `M₂ = N − log₂ Σ_P ⟨P⟩⁴` over all `4^N`
/// Pauli strings of the NORMALISED state (the norm is divided out, so an unnormalised input
/// reads the same). Refuses a chain whose local dimension is not 2, that does not close on
/// trivial bonds, or whose replica vectors would exceed the lease.
pub fn sre2(tensors: &[TensorSite]) -> Result<f64, MagicError> {
    check(tensors)?;
    let (price, lease) = (price_bytes(tensors), lease_bytes());
    if price > lease {
        return Err(MagicError::Price { bytes: price, lease_bytes: lease });
    }
    let n = tensors.len() as f64;
    let l2n2 = log2_norm2(tensors)?;
    let l2s4 = log2_replica_sum(tensors, 4, |_| &ALL_CHANNELS)?;
    Ok(n - l2s4 + 4.0 * l2n2)
}

/// `log₂ Σ_P ⟨P⟩²` over all strings — equals `N + 2 log₂⟨ψ|ψ⟩` for every state (the Pauli basis
/// is orthogonal): the two-replica contraction's own identity check, exposed for the gates.
pub fn log2_pauli_sum2(tensors: &[TensorSite]) -> Result<f64, MagicError> {
    check(tensors)?;
    log2_replica_sum(tensors, 2, |_| &ALL_CHANNELS)
}

/// **M₂ of a box**: the stabilizer 2-Rényi entropy of the reduced state `ρ_L` on the first
/// `box_len` sites, in the mixed-state form of Leone–Oliviero–Hamma,
/// `M̃₂(ρ_L) = −log₂ [ Σ_{P on L} Tr(ρ_L P)⁴ / Σ_{P on L} Tr(ρ_L P)² ]`, which is `M₂` when
/// `box_len = N` and zero on the maximally mixed state. The prereg's `M₂(L)` (§2, "for a box of
/// L sites") does not say which state of the box it means; this is the reading under which the
/// reduced state of the vacuum is what is priced, and it is named as a choice, not the prereg's.
pub fn sre2_box(tensors: &[TensorSite], box_len: usize) -> Result<f64, MagicError> {
    check(tensors)?;
    let n = tensors.len();
    if box_len == 0 || box_len > n {
        return Err(MagicError::BoxTooLong { box_len, n });
    }
    let (price, lease) = (price_bytes(tensors), lease_bytes());
    if price > lease {
        return Err(MagicError::Price { bytes: price, lease_bytes: lease });
    }
    let l2n2 = log2_norm2(tensors)?;
    let inside = move |j: usize| -> &'static [usize] { if j < box_len { &ALL_CHANNELS } else { &IDENTITY_ONLY } };
    let l2s4 = log2_replica_sum(tensors, 4, inside)?;
    let l2s2 = log2_replica_sum(tensors, 2, inside)?;
    Ok(l2s2 + 2.0 * l2n2 - l2s4)
}

/// **The brute-force referee**: every one of the `4^N` Pauli expectations through
/// `observables::expectation`, fourth powers summed with Neumaier compensation. `N ≤ 10`.
pub fn sre2_brute(tensors: &[TensorSite]) -> Result<f64, MagicError> {
    check(tensors)?;
    let n = tensors.len();
    if n > BRUTE_MAX_SITES {
        return Err(MagicError::BruteTooLarge { n, max: BRUTE_MAX_SITES });
    }
    let n2 = observables::norm_squared(tensors);
    if n2 <= 0.0 || !n2.is_finite() {
        return Err(MagicError::ZeroNorm);
    }
    let (mut sum, mut comp) = (0.0f64, 0.0f64);
    let mut inserts: Vec<(usize, Op2)> = Vec::with_capacity(n);
    for code in 0..(1usize << (2 * n)) {
        inserts.clear();
        for j in 0..n {
            let a = (code >> (2 * j)) & 3;
            if a != PAULI_I {
                inserts.push((j, PAULI_REAL[a]));
            }
        }
        let e = observables::expectation(tensors, &inserts) / n2;
        let term = e * e * e * e;
        let t = sum + term;
        comp += if sum.abs() >= term.abs() { (sum - t) + term } else { (term - t) + sum };
        sum = t;
    }
    let total = sum + comp;
    if total <= 0.0 {
        return Err(MagicError::ZeroNorm);
    }
    Ok(n as f64 - total.log2())
}

// ------------------------------------------------------------------ the single-qubit Cliffords

/// One single-qubit Clifford, up to phase, by what it does to the Pauli axes: on the rotated
/// state `C|ψ⟩`, `⟨σ_a⟩ = sign[a] · ⟨σ_{perm[a]}⟩_ψ` for `a ∈ {X, Y, Z} = {0, 1, 2}`. The 24
/// elements are the signed permutations of the three axes with determinant `+1` (the rotation
/// group of the octahedron), and that action is the whole of what `M₂` can see of a Clifford.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clifford1 {
    pub perm: [usize; 3],
    pub sign: [i8; 3],
}

impl Clifford1 {
    /// Real up to phase — an `O(2)` matrix — iff it sends the `Y` axis to `±Y`. Eight of the 24.
    pub fn is_real(&self) -> bool {
        self.perm[1] == 1
    }

    /// The `2×2` real orthogonal matrix `U` with `Uᵀ σ_a U = sign[a] σ_{perm[a]}`, for the eight
    /// real Cliffords; `None` for the sixteen whose matrix is complex (`S`, `HS`, …).
    pub fn real_matrix(&self) -> Option<[[f64; 2]; 2]> {
        if !self.is_real() {
            return None;
        }
        let h = std::f64::consts::FRAC_1_SQRT_2;
        // rotations R(φ) = [[c,−s],[s,c]] send Y → Y; reflections [[c,s],[s,−c]] send Y → −Y.
        // Derived by hand in the doc of `clifford_group` and checked against the tensor
        // conjugation in this module's tests.
        let m = match (self.perm, self.sign) {
            ([0, 1, 2], [1, 1, 1]) => [[1.0, 0.0], [0.0, 1.0]],
            ([2, 1, 0], [1, 1, -1]) => [[h, -h], [h, h]],
            ([0, 1, 2], [-1, 1, -1]) => [[0.0, -1.0], [1.0, 0.0]],
            ([2, 1, 0], [-1, 1, 1]) => [[-h, -h], [h, -h]],
            ([0, 1, 2], [-1, -1, 1]) => [[1.0, 0.0], [0.0, -1.0]],
            ([2, 1, 0], [1, -1, 1]) => [[h, h], [h, -h]],
            ([0, 1, 2], [1, -1, -1]) => [[0.0, 1.0], [1.0, 0.0]],
            ([2, 1, 0], [-1, -1, -1]) => [[-h, h], [h, h]],
            _ => return None,
        };
        Some(m)
    }

    /// The channel order this frame sums in: `channels[a] = perm[a]` on `{X, Y, Z}`, `I ↦ I`.
    fn channels(&self) -> [usize; 4] {
        [PAULI_I, 1 + self.perm[0], 1 + self.perm[1], 1 + self.perm[2]]
    }
}

/// The 24 single-qubit Cliffords in a FIXED enumeration: the six permutations of `(X, Y, Z)`
/// in lexicographic order, each with the four sign patterns of the right parity (an even
/// permutation with an even number of minus signs, an odd one with an odd number), sign
/// patterns in the order `+++, ++−, +−+, +−−, −++, −+−, −−+, −−−`. Index 0 is the identity.
pub fn clifford_group() -> [Clifford1; 24] {
    const PERMS: [[usize; 3]; 6] = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];
    const PARITY: [i8; 6] = [1, -1, -1, 1, 1, -1];
    let mut out = Vec::with_capacity(24);
    for (p, perm) in PERMS.iter().enumerate() {
        for bits in 0..8u8 {
            let sign = [
                if bits & 4 == 0 { 1i8 } else { -1 },
                if bits & 2 == 0 { 1i8 } else { -1 },
                if bits & 1 == 0 { 1i8 } else { -1 },
            ];
            if PARITY[p] * sign[0] * sign[1] * sign[2] == 1 {
                out.push(Clifford1 { perm: *perm, sign });
            }
        }
    }
    out.try_into().expect("24 signed permutations of determinant +1")
}

/// The eight indices into `clifford_group()` whose matrix is real.
pub fn real_clifford_indices() -> Vec<usize> {
    clifford_group().iter().enumerate().filter(|(_, c)| c.is_real()).map(|(k, _)| k).collect()
}

/// Rotate one site by a real Clifford: `A'^s = Σ_{s'} U[s][s'] A^{s'}` — the `2×2` unitary on
/// the physical index, as the prereg's P3 applies it. The sixteen complex Cliffords are refused
/// here (a real tensor cannot carry them); the minimiser sees them through their Pauli action.
pub fn apply_real_clifford(site: &TensorSite, index: usize) -> Result<TensorSite, MagicError> {
    let c = clifford_group()[index];
    let u = c.real_matrix().ok_or(MagicError::ComplexClifford { index })?;
    let mut out = TensorSite::zeros(site.chi_l, site.chi_r);
    for l in 0..site.chi_l {
        for r in 0..site.chi_r {
            let (a0, a1) = (site.get(0, l, r), site.get(1, l, r));
            out.set(0, l, r, u[0][0] * a0 + u[0][1] * a1);
            out.set(1, l, r, u[1][0] * a0 + u[1][1] * a1);
        }
    }
    Ok(out)
}

/// **M₂^loc**: `M₂` minimised over products of single-site Cliffords by coordinate descent from
/// the identity frame — sites in order, the 24 Cliffords of `clifford_group()` in their fixed
/// order at each site, `sweeps` full sweeps, a candidate accepted only when strictly lower than
/// the site's current frame. Returns the minimum found (over every evaluation) and the frame it
/// was found in, one Clifford index per site. `sweeps = 0` is `sre2` in the identity frame.
///
/// Each site's 24 candidates are evaluated from the same four per-channel numbers
/// `w_a = ⟨L_j| E_a^{⊗4} |R_{j+1}⟩` (one site application), because a Clifford only permutes
/// which channel the state calls `X`, `Y`, `Z` and the fourth power forgets the sign — see the
/// module doc: the minimum is `M₂` to rounding, and this function is the measurement of that.
/// Cost: `2N` site applications per sweep; memory: every right environment is kept.
pub fn sre2_local_min(tensors: &[TensorSite], sweeps: usize) -> Result<(f64, Vec<usize>), MagicError> {
    let (trace, frames) = sre2_local_min_trace(tensors, sweeps)?;
    Ok((*trace.last().expect("one entry per sweep, plus the start"), frames))
}

/// [`sre2_local_min`] with its record: entry `s` of the trace is the minimum found after `s`
/// sweeps (entry 0 the identity frame's `M₂`), so one run reads what a further sweep bought.
pub fn sre2_local_min_trace(tensors: &[TensorSite], sweeps: usize) -> Result<(Vec<f64>, Vec<usize>), MagicError> {
    check(tensors)?;
    let (price, lease) = (price_bytes_local_min(tensors), lease_bytes());
    if price > lease {
        return Err(MagicError::Price { bytes: price, lease_bytes: lease });
    }
    let n = tensors.len();
    let group = clifford_group();
    let l2n2 = log2_norm2(tensors)?;
    let m2_of = |log2_sum4: f64| n as f64 - log2_sum4 + 4.0 * l2n2;

    let mut frames = vec![0usize; n];
    let mut best = (sre2(tensors)?, frames.clone());
    let mut trace = vec![best.0];
    let fwd: Vec<SiteTables> = tensors.iter().map(|t| SiteTables::new(t, true)).collect();
    let bwd: Vec<SiteTables> = tensors.iter().map(|t| SiteTables::new(t, false)).collect();
    let mut sc = Scratch::default();

    for _sweep in 0..sweeps {
        // Right environments in the current frames: right[j] is the contraction of sites j..N−1,
        // a vector on the eight legs of bond j; right[N] is the unit.
        let mut right: Vec<Scaled> = Vec::with_capacity(n + 1);
        right.push(Scaled::unit());
        for j in (0..n).rev() {
            let prev = right.last().expect("seeded");
            let mut next = Vec::new();
            apply_site(&prev.v, &bwd[j], &group[frames[j]].channels(), 4, &mut next, &mut sc);
            let mut s = Scaled { v: next, log2: prev.log2 };
            s.renormalise()?;
            right.push(s);
        }
        right.reverse();

        let mut left = Scaled::unit();
        for j in 0..n {
            // The four channel vectors Y_a = E_a^{⊗4} L_j, and their overlaps with R_{j+1}.
            let mut ys: [Vec<f64>; 4] = Default::default();
            let mut w = [0.0f64; 4];
            for a in 0..4 {
                apply_site(&left.v, &fwd[j], &[a], 4, &mut ys[a], &mut sc);
                w[a] = dot(&ys[a], &right[j + 1].v);
            }
            let base = left.log2 + right[j + 1].log2;
            let value_in = |k: usize| -> f64 {
                let ch = group[k].channels();
                let s = ((w[ch[0]] + w[ch[1]]) + w[ch[2]]) + w[ch[3]];
                if s > 0.0 { m2_of(base + s.log2()) } else { f64::INFINITY }
            };
            let mut cur_k = frames[j];
            let mut cur = value_in(cur_k);
            for k in 0..24 {
                let v = value_in(k);
                if v < cur {
                    cur = v;
                    cur_k = k;
                }
            }
            frames[j] = cur_k;
            if cur < best.0 {
                best = (cur, frames.clone());
            }
            // Advance the left environment in the chosen frame's channel order.
            let ch = group[cur_k].channels();
            let mut next = ys[ch[0]].clone();
            for &a in &ch[1..] {
                for (x, y) in next.iter_mut().zip(&ys[a]) {
                    *x += *y;
                }
            }
            left = Scaled { v: next, log2: left.log2 };
            left.renormalise()?;
        }
        trace.push(best.0);
    }
    Ok((trace, best.1))
}

// ------------------------------------------------------------------ carriers for the plants

/// A product state as a `χ = 1` MPS, one `(ψ₀, ψ₁)` per site (not normalised here).
pub fn product_state_mps(local: &[[f64; 2]]) -> Vec<TensorSite> {
    local
        .iter()
        .map(|amp| {
            let mut t = TensorSite::zeros(1, 1);
            t.set(0, 0, 0, amp[0]);
            t.set(1, 0, 0, amp[1]);
            t
        })
        .collect()
}

/// The GHZ state `(|0…0⟩ + |1…1⟩)/√2` as a `χ = 2` MPS (a single site reads `|+⟩`).
pub fn ghz_mps(n: usize) -> Vec<TensorSite> {
    assert!(n >= 1, "a GHZ state needs a site");
    let h = std::f64::consts::FRAC_1_SQRT_2;
    (0..n)
        .map(|j| {
            let (chi_l, chi_r) = (if j == 0 { 1 } else { 2 }, if j + 1 == n { 1 } else { 2 });
            let mut t = TensorSite::zeros(chi_l, chi_r);
            for s in 0..2 {
                let (l, r) = (if j == 0 { 0 } else { s }, if j + 1 == n { 0 } else { s });
                t.set(s, l, r, if j == 0 { h } else { 1.0 });
            }
            t
        })
        .collect()
}

/// A seeded random real MPS, bond dimension `min(2^j, χ, 2^{N−j})` at bond `j`, entries
/// uniform in `[−½, ½)` from the crate's LCG, normalised to `⟨ψ|ψ⟩ = 1` (each site scaled to
/// unit Frobenius norm first, so long chains neither overflow nor underflow).
pub fn random_mps(n: usize, chi: usize, seed: u64) -> Vec<TensorSite> {
    assert!(n >= 1 && chi >= 1);
    let mut st = seed;
    let mut rnd = || {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((st >> 11) as f64) / ((1u64 << 53) as f64) - 0.5
    };
    let bond = |j: usize| -> usize {
        let cap = 62usize;
        let left = if j >= cap { usize::MAX } else { 1usize << j };
        let right = if n - j >= cap { usize::MAX } else { 1usize << (n - j) };
        left.min(right).min(chi)
    };
    let mut tensors: Vec<TensorSite> = (0..n)
        .map(|j| {
            let mut t = TensorSite::zeros(bond(j), bond(j + 1));
            for x in t.data.iter_mut() {
                *x = rnd();
            }
            let f = 1.0 / t.data.iter().map(|x| x * x).sum::<f64>().sqrt();
            t.data.iter_mut().for_each(|x| *x *= f);
            t
        })
        .collect();
    let l2 = log2_norm2(&tensors).expect("a random MPS has a norm");
    let f = (2.0f64).powf(-0.5 * l2);
    tensors[0].data.iter_mut().for_each(|x| *x *= f);
    tensors
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pauli_transfer(t: &TensorSite, a: usize) -> Vec<f64> {
        let (cl, cr) = (t.chi_l, t.chi_r);
        let sigma = PAULI_REAL[a];
        let mut e = vec![0.0; cl * cl * cr * cr];
        for l in 0..cl {
            for lp in 0..cl {
                for r in 0..cr {
                    for rp in 0..cr {
                        let mut acc = 0.0;
                        for s in 0..2 {
                            for sp in 0..2 {
                                acc += sigma[s][sp] * t.get(s, l, r) * t.get(sp, lp, rp);
                            }
                        }
                        e[((l * cl + lp) * cr + r) * cr + rp] = acc;
                    }
                }
            }
        }
        e
    }

    #[test]
    fn the_group_has_24_distinct_elements_eight_of_them_real_and_index_zero_is_the_identity() {
        let g = clifford_group();
        for i in 0..24 {
            for j in (i + 1)..24 {
                assert_ne!(g[i], g[j]);
            }
        }
        assert_eq!(g[0], Clifford1 { perm: [0, 1, 2], sign: [1, 1, 1] });
        assert_eq!(real_clifford_indices().len(), 8);
        assert!(g.iter().all(|c| c.real_matrix().is_some() == c.is_real()));
    }

    /// The Pauli-frame table of every real Clifford is what conjugating the tensor does:
    /// `E_a(U A) = sign[a] · E_{perm[a]}(A)` on a random site, entry by entry.
    #[test]
    fn real_cliffords_act_on_the_pauli_transfer_as_the_table_says() {
        let t = &random_mps(3, 4, 11)[1];
        let g = clifford_group();
        for k in real_clifford_indices() {
            let rot = apply_real_clifford(t, k).unwrap();
            for a in 0..3 {
                let lhs = pauli_transfer(&rot, 1 + a);
                let rhs = pauli_transfer(t, 1 + g[k].perm[a]);
                let sgn = g[k].sign[a] as f64;
                for (x, y) in lhs.iter().zip(&rhs) {
                    assert!((x - sgn * y).abs() < 1e-14, "Clifford {k}, axis {a}: {x} vs {sgn}·{y}");
                }
            }
            let (lhs, rhs) = (pauli_transfer(&rot, 0), pauli_transfer(t, 0));
            for (x, y) in lhs.iter().zip(&rhs) {
                assert!((x - y).abs() < 1e-14);
            }
        }
    }

    /// `Σ_P ⟨P⟩² = 2^N ⟨ψ|ψ⟩²` for every state: the two-replica contraction's identity.
    #[test]
    fn the_two_replica_sum_is_the_purity_identity() {
        for (n, chi, seed) in [(5usize, 3usize, 1u64), (8, 6, 2), (12, 8, 3)] {
            let t = random_mps(n, chi, seed);
            let l2 = log2_pauli_sum2(&t).unwrap();
            let l2n2 = log2_norm2(&t).unwrap();
            assert!((l2 - (n as f64 + 2.0 * l2n2)).abs() < 1e-11, "N={n}: {l2} vs {}", n as f64 + 2.0 * l2n2);
        }
    }

    #[test]
    fn a_wrong_local_dimension_and_an_open_boundary_are_refused() {
        let mut t = random_mps(4, 2, 5);
        t[1].data.push(0.0);
        assert!(matches!(sre2(&t), Err(MagicError::LocalDimension { site: 1, .. })));
        let mut t = random_mps(4, 2, 5);
        let cl = t[3].chi_l;
        t[3].chi_r = 2;
        t[3].data.resize(2 * cl * 2, 0.0);
        assert!(matches!(sre2(&t), Err(MagicError::OpenBoundary { .. })));
        assert!(matches!(sre2_brute(&random_mps(11, 2, 1)), Err(MagicError::BruteTooLarge { n: 11, .. })));
    }

    /// The prereg's largest point, N = 48 at χ = 64, is priced at `64⁸ · 32` bytes — nine
    /// petabytes — and refused by name; χ = 12 is refused under the default lease too.
    #[test]
    fn the_prereg_ladder_at_chi_64_is_refused_by_price_not_attempted() {
        let big = random_mps(48, 64, 1);
        match sre2(&big) {
            Err(MagicError::Price { bytes, lease_bytes }) => {
                assert_eq!(bytes, 4 * 64u64.pow(8) * 8);
                assert!(bytes > lease_bytes);
            }
            other => panic!("expected the price refusal, got {other:?}"),
        }
        assert!(matches!(sre2(&random_mps(16, 12, 1)), Err(MagicError::Price { .. })));
        assert!(matches!(sre2_local_min(&big, 1), Err(MagicError::Price { .. })));
    }

    #[test]
    fn the_box_at_full_length_is_m2_and_a_single_site_of_a_product_state_reads_its_own_magic() {
        let t = random_mps(6, 4, 9);
        let (a, b) = (sre2(&t).unwrap(), sre2_box(&t, 6).unwrap());
        assert!((a - b).abs() < 1e-12, "{a} vs {b}");
        let c = (std::f64::consts::PI / 8.0).cos();
        let s = (std::f64::consts::PI / 8.0).sin();
        let p = product_state_mps(&[[c, s]; 5]);
        let one = sre2_box(&p, 1).unwrap();
        assert!((one - (4.0f64 / 3.0).log2()).abs() < 1e-13, "{one}");
    }
}
