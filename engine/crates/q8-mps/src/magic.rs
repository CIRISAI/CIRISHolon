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
//! **The non-local minimiser** (`sre2_nonlocal_min`) is Amendment 1 §A2's
//! (`conformance/crystal/GF1_AMENDMENT_1.md`): `M₂` minimised over products of CONTINUOUS
//! single-site real rotations `R_y(θ_j)`, by coordinate descent from the identity frame with
//! every evaluation the exact `sre2`. Building it found four things the amendment has wrong,
//! stated in its `Correction on building` and here. (0) THE ONE THAT MATTERS FOR THE LADDER: on
//! a real state of definite parity, `Π_j Z_j|ψ⟩ = ±|ψ⟩`, the identity frame is every site's own
//! minimum and the descent never moves — `M₂^nl = M₂` by symmetry. Proof: `⟨P⟩ ≠ 0` needs `P` to
//! commute with `Π Z`, i.e. an even count of `X`/`Y`; for any rest-string `P_r`, `X_j P_r` and
//! `Z_j P_r` differ by one in that count, so at most one of `⟨X_j P_r⟩`, `⟨Z_j P_r⟩` is non-zero,
//! and every pair's term below is `¼ r⁴ cos(8θ)` with phase zero. The Jordan–Wigner Schwinger
//! vacuum has definite charge `Σ Z_j`, hence definite parity: the amendment's re-staked S3 will
//! read `M₂^nl = M₂` at every ladder point, by the vacuum's symmetry and not its physics — the
//! prereg's "quantity assumed to vary that a theorem holds fixed", a second time. Measured on
//! the vacuum at `x = 4, N = 12, χ = 6` and on a parity-projected random MPS (six descents from
//! random frames all converge back down toward the identity's value, none below it; a joint
//! two-angle scan on the two-site tilted GHZ finds nothing lower). Nor is the real frame the
//! limitation: for a real state every string with an odd count of `Y` reads zero, so with
//! parity each fibre `(⟨X_j P_r⟩, ⟨Y_j P_r⟩, ⟨Z_j P_r⟩)` has at most one non-zero entry and lies
//! on an axis already, and `Σ_a (Rv)_a⁴ ≤ |v|⁴` with equality only on an axis — the identity is
//! each site's minimum over ALL of `SU(2)`. Jointly, on even-parity real random states at
//! `N = 4`, thousands of random `SU(2)^N` frames and a descent over every angle find nothing
//! below the identity (an unprojected real state is lowered by `0.37` under `R_y` alone). On
//! such a state the local frame has nothing to remove: `M₂^nl = M₂` is the state's property.
//! (1) The per-site landscape is not a function
//! to be searched: with the other sites fixed, `2^{−M₂}(θ_j) = a + r cos(8θ_j − φ)` EXACTLY (each
//! `⟨P⟩` with `P_j ∈ {X, Z}` turns as `cos(2θ − δ)` or `sin(2θ − δ)` and the two fourth powers sum
//! to `¾ + ¼ cos(8θ − 4δ)`), so its minimiser is known in closed form from three readings. The
//! amendment's golden-section search on `[0, π)` is a unimodal method on FOUR periods of that
//! sinusoid (`R_y(π/4) = HZ` is a Clifford, so the period is `π/4`); it is kept only as the
//! referee `sre2_nonlocal_min_golden`. The true cost is `3` evaluations per site per sweep, not
//! `~20`. (2) The amendment's P3′ row, "`M₂` itself is `0.415 N`" for H-type states "rotated by
//! random single-site unitaries", holds for random Cliffords (P3's draw) and not for random
//! angles: `M₂` of `R_y(α)|H⟩` is `−log₂(1 − ¼ cos² 4α)`, zero at `α = π/8`. Under a random
//! continuous frame the product's `M₂` is the sum of those, and only `M₂^nl = 0` survives as
//! the stake. (3) §A2's "a GHZ state's [M₂^nl] is not zero: its magic is in the entanglement"
//! contradicts its own P7 row and the theorem: GHZ is a stabilizer state, `M₂ = 0`, and
//! `M₂^nl ≤ M₂`. What the sentence reaches for is a state whose entanglement is NOT a stabilizer
//! state's — `cos(π/8)|0…0⟩ + sin(π/8)|1…1⟩`, Schmidt coefficients no local frame can change — and
//! that is P7's control.
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
    /// `sre2_nonlocal_min_ordered`: the site order is not a permutation of `0..N`.
    SiteOrder { len: usize, n: usize },
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
            MagicError::SiteOrder { len, n } => {
                write!(f, "a site order of {len} entries is not a permutation of the {n} sites")
            }
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

// ------------------------------------------------------------------ the non-local magic (Amendment 1, §A2)

/// The real single-site rotation `R_y(θ) = [[cos θ, −sin θ], [sin θ, cos θ]]` — `exp(−iθY)`, the
/// Bloch sphere turned about `Y` by `2θ`, so `R_y(π/8)|0⟩` is the H-type state and `R_y(π/4)|0⟩`
/// is `|+⟩` — contracted into a site's physical leg: `A'^s = Σ_{s'} R[s][s'] A^{s'}`. On a real
/// state this is the whole of `SO(2)`, the continuous local frame Amendment 1 §A2 minimises over;
/// `R_y(θ + π) = −R_y(θ)` is the same state, so `[0, π)` is every frame there is.
pub fn apply_ry(site: &TensorSite, theta: f64) -> TensorSite {
    let (c, s) = (theta.cos(), theta.sin());
    let mut out = TensorSite::zeros(site.chi_l, site.chi_r);
    for l in 0..site.chi_l {
        for r in 0..site.chi_r {
            let (a0, a1) = (site.get(0, l, r), site.get(1, l, r));
            out.set(0, l, r, c * a0 - s * a1);
            out.set(1, l, r, s * a0 + c * a1);
        }
    }
    out
}

/// The period of `M₂` in any one site's `R_y` angle, `π/4`: `R_y(π/4) = HZ` sends
/// `X ↦ Z, Z ↦ −X, Y ↦ Y` — a Clifford — and `M₂` is Clifford-invariant. The amendment's `[0, π)`
/// holds four copies of this fundamental domain, and every angle this module reports is reduced
/// into it.
pub const RY_PERIOD: f64 = std::f64::consts::FRAC_PI_4;

/// What the non-local minimiser returns.
#[derive(Debug, Clone, PartialEq)]
pub struct NonlocalMagic {
    /// `M₂^nl`: the minimum of `M₂` over every frame evaluated. The identity frame is evaluated
    /// first, so this never exceeds `m2_identity`.
    pub m2: f64,
    /// `M₂` of the state as given — `sre2`'s own reading, the first evaluation.
    pub m2_identity: f64,
    /// The frame the minimum was found in: `R_y(angles[j])` applied to site `j` AS GIVEN (not
    /// cumulative), each in `[0, RY_PERIOD)` for the exact line search; the golden-section
    /// referee reports the angle it evaluated, in `[0, π)`.
    pub angles: Vec<f64>,
    /// Entry `s` is the running minimum after `s` full sweeps; entry 0 is `m2_identity`.
    pub per_sweep: Vec<f64>,
    /// Calls to `sre2` made, the identity frame's included.
    pub evaluations: usize,
    /// The site order the sweeps ran in.
    pub order: Vec<usize>,
}

/// An angle reduced into `[0, RY_PERIOD)`, and snapped to `0` from within `10⁻¹²` below the
/// period: a rounding-level negative (`−10⁻¹⁶`, the fitted minimiser of a site already at its
/// minimum) would otherwise come back as `π/4 − 10⁻¹⁶` and be reported as a quarter turn. The
/// snap moves the frame by under `10⁻¹²` rad, `10⁻²⁴` in `M₂`, and is applied BEFORE the
/// evaluation, so the reported angle is exactly the evaluated one.
fn reduce_angle(theta: f64) -> f64 {
    let r = theta.rem_euclid(RY_PERIOD);
    if r >= RY_PERIOD - 1e-12 { 0.0 } else { r }
}

#[derive(Clone, Copy)]
enum LineSearch {
    /// The exact line search: two readings beside the current one fit the sinusoid, the third is
    /// at its maximiser. Three evaluations per site per sweep.
    Sinusoid,
    /// Amendment 1 §A2's line search as written: golden section on `[0, π)`, stopping when the
    /// bracket is under `tol` radians.
    Golden { tol: f64 },
}

/// **M₂^nl, the non-local magic** — Amendment 1 §A2: `min_{θ_1..θ_N} M₂( ⊗_j R_y(θ_j) |ψ⟩ )` by
/// coordinate descent from the identity frame, sites in index order, `sweeps` full sweeps, every
/// evaluation the exact [`sre2`] of the rotated chain (the rotation is contracted into one site's
/// physical leg; nothing else is rebuilt). Refuses exactly what `sre2` refuses, the `χ ≤ 11`
/// lease included, before any evaluation.
///
/// **The line search is exact, not golden-section, and this is the finding.** With every other
/// site fixed, each `⟨P⟩` with `P_j ∈ {X, Z}` is `r cos(2θ_j − δ)` or `−r sin(2θ_j − δ)` (the
/// Bloch vector turned in the `XZ` plane) and the two fourth powers sum to
/// `r⁴(¾ + ¼ cos(8θ_j − 4δ))`; strings with `P_j ∈ {I, Y}` do not move. So
///
/// ```text
/// 2^{−M₂(θ_j)} = a + r · cos(8 θ_j − φ),   EXACTLY, for every site of every real MPS
/// ```
///
/// — one sinusoid of period `π/4`, which is also what `R_y(π/4) = HZ` being a Clifford says.
/// Its three unknowns are fixed by three readings — the current frame's, and two more at
/// `θ_j + π/16` and `θ_j + π/8` — and its maximiser `θ* = (φ mod 2π)/8` is then known in closed
/// form. The third evaluation is AT `θ*`, and it is that exact reading, not the fit, that is
/// compared and banked: the site moves only to the strictly lowest of the four frames evaluated
/// (current, two samples, `θ*`), so the running value is monotone and is the minimum over
/// everything evaluated. The amendment's golden-section search on `[0, π)` is a unimodal method
/// on four full periods of a sinusoid: it can settle on a period's boundary rather than its
/// minimum, and measured (`tests/gf1_plants.rs`, the referee) it spends 19 evaluations per line
/// search to land `10⁻⁵` above the exact search at a `10⁻³` bracket and 34 to land beside it at
/// `10⁻⁶`, where three are exact. It is kept as [`sre2_nonlocal_min_golden`], the referee.
///
/// Cost: `1 + 3 N · sweeps` evaluations of `sre2`, each `64 χ⁹` per site over `N` sites —
/// `O(N²)` per sweep. Memory: `sre2`'s own. Deterministic: serial, the same order every time.
/// The descent is coordinate descent and converges linearly where it moves at all: at `N = 12,
/// χ = 6` on a random MPS the sweeps buy `1.28, 1.4 × 10⁻², 7 × 10⁻⁴, 2.3 × 10⁻⁴, …`, so "three
/// sweeps" is not a convergence criterion; read `per_sweep` and stop on it. On a state of
/// definite parity it does not move at all (module doc, finding (0)).
pub fn sre2_nonlocal_min(tensors: &[TensorSite], sweeps: usize) -> Result<NonlocalMagic, MagicError> {
    let order: Vec<usize> = (0..tensors.len()).collect();
    nonlocal_min_impl(tensors, sweeps, &order, LineSearch::Sinusoid)
}

/// [`sre2_nonlocal_min`] with the sites visited in `order`, which must be a permutation of
/// `0..N` (P8's reversed order, or any other); a malformed order is refused by name.
pub fn sre2_nonlocal_min_ordered(
    tensors: &[TensorSite],
    sweeps: usize,
    order: &[usize],
) -> Result<NonlocalMagic, MagicError> {
    nonlocal_min_impl(tensors, sweeps, order, LineSearch::Sinusoid)
}

/// Amendment 1 §A2's line search AS WRITTEN — golden section on `θ_j ∈ [0, π)` until the bracket
/// is under `tol` radians, the site moved only to a strictly lower frame — with the same descent
/// as [`sre2_nonlocal_min`]. Kept as that reader's referee, not as an instrument: see the module
/// doc for why the landscape makes it the wrong tool, and `tests/gf1_plants.rs` for what it reads
/// beside the exact search.
pub fn sre2_nonlocal_min_golden(tensors: &[TensorSite], sweeps: usize, tol: f64) -> Result<NonlocalMagic, MagicError> {
    let order: Vec<usize> = (0..tensors.len()).collect();
    nonlocal_min_impl(tensors, sweeps, &order, LineSearch::Golden { tol })
}

fn nonlocal_min_impl(
    tensors: &[TensorSite],
    sweeps: usize,
    order: &[usize],
    ls: LineSearch,
) -> Result<NonlocalMagic, MagicError> {
    check(tensors)?;
    let (price, lease) = (price_bytes(tensors), lease_bytes());
    if price > lease {
        return Err(MagicError::Price { bytes: price, lease_bytes: lease });
    }
    let n = tensors.len();
    let mut seen = vec![false; n];
    if order.len() != n || !order.iter().all(|&j| j < n && !std::mem::replace(&mut seen[j], true)) {
        return Err(MagicError::SiteOrder { len: order.len(), n });
    }

    let mut work: Vec<TensorSite> = tensors.to_vec();
    let mut evaluations = 0usize;
    // M₂ of the working frame with site j re-rotated to θ (from the site as given).
    let mut eval = |work: &mut [TensorSite], j: usize, theta: f64| -> Result<f64, MagicError> {
        work[j] = apply_ry(&tensors[j], theta);
        evaluations += 1;
        sre2(work)
    };

    let m2_identity = eval(&mut work, 0, 0.0)?;
    let mut cur = m2_identity;
    let mut angles = vec![0.0f64; n];
    let mut per_sweep = vec![cur];
    for _ in 0..sweeps {
        for &j in order {
            let th_c = angles[j];
            // The frames evaluated at this site, the current one first; the strictly lowest wins
            // (a tie keeps the earlier), so the running value never rises.
            let mut cands: Vec<(f64, f64)> = vec![(th_c, cur)];
            match ls {
                LineSearch::Sinusoid => {
                    // 2^{−M₂} = a + r cos(8(θ − θ_c) + u): the current reading is a + r cos u, the
                    // one at +π/16 is a − r sin u, the one at +π/8 is a − r cos u.
                    let th1 = reduce_angle(th_c + RY_PERIOD / 4.0);
                    let th2 = reduce_angle(th_c + RY_PERIOD / 2.0);
                    let m1 = eval(&mut work, j, th1)?;
                    let m2 = eval(&mut work, j, th2)?;
                    let (s0, s1, s2) = ((-cur).exp2(), (-m1).exp2(), (-m2).exp2());
                    let a = 0.5 * (s0 + s2);
                    let (r_cos_u, r_sin_u) = (0.5 * (s0 - s2), a - s1);
                    let u = r_sin_u.atan2(r_cos_u);
                    let th_star = reduce_angle(th_c - u / 8.0);
                    let m_star = eval(&mut work, j, th_star)?;
                    cands.extend([(th1, m1), (th2, m2), (th_star, m_star)]);
                }
                LineSearch::Golden { tol } => {
                    let g = 0.5 * (5.0f64.sqrt() - 1.0);
                    let (mut lo, mut hi) = (0.0f64, std::f64::consts::PI);
                    let (mut c, mut d) = (hi - g * (hi - lo), lo + g * (hi - lo));
                    let (mut fc, mut fd) = (eval(&mut work, j, c)?, eval(&mut work, j, d)?);
                    cands.extend([(c, fc), (d, fd)]);
                    while hi - lo > tol {
                        if fc < fd {
                            hi = d;
                            d = c;
                            fd = fc;
                            c = hi - g * (hi - lo);
                            fc = eval(&mut work, j, c)?;
                            cands.push((c, fc));
                        } else {
                            lo = c;
                            c = d;
                            fc = fd;
                            d = lo + g * (hi - lo);
                            fd = eval(&mut work, j, d)?;
                            cands.push((d, fd));
                        }
                    }
                }
            }
            let (th_new, m_new) = cands.iter().copied().fold(cands[0], |b, c| if c.1 < b.1 { c } else { b });
            angles[j] = th_new;
            cur = m_new;
            work[j] = apply_ry(&tensors[j], th_new);
        }
        per_sweep.push(cur);
    }
    Ok(NonlocalMagic { m2: cur, m2_identity, angles, per_sweep, evaluations, order: order.to_vec() })
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

/// The even-parity part `(1 + Π_j Z_j)|ψ⟩` of a chain as an MPS of doubled bonds (not
/// normalised): two copies of every site tensor, block-diagonal, the second with `Z` absorbed,
/// summed at the ends. A carrier for the parity theorem in the module doc — any real state with
/// `Π_j Z_j|ψ⟩ = ±|ψ⟩`, which a Jordan–Wigner vacuum of definite charge is.
pub fn even_parity_mps(tensors: &[TensorSite]) -> Vec<TensorSite> {
    let n = tensors.len();
    tensors
        .iter()
        .enumerate()
        .map(|(j, t)| {
            let (cl, cr) = (t.chi_l, t.chi_r);
            let (first, last) = (j == 0, j + 1 == n);
            let mut o = TensorSite::zeros(if first { 1 } else { 2 * cl }, if last { 1 } else { 2 * cr });
            for s in 0..2 {
                let z = if s == 0 { 1.0 } else { -1.0 };
                for l in 0..cl {
                    for r in 0..cr {
                        let v = t.get(s, l, r);
                        for copy in 0..2 {
                            let w = if copy == 0 { v } else { z * v };
                            let ll = if first { 0 } else { l + copy * cl };
                            let rr = if last { 0 } else { r + copy * cr };
                            o.set(s, ll, rr, o.get(s, ll, rr) + w);
                        }
                    }
                }
            }
            o
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

    /// `R_y(π/4)` is `HZ`, a Clifford: on a random MPS `M₂` is unchanged by it at any site, and
    /// `R_y(π/8)` (a non-Clifford, the H-type rotation) changes it — so `RY_PERIOD` is `π/4` and
    /// not less.
    #[test]
    fn ry_by_a_quarter_turn_is_the_clifford_hz_and_by_an_eighth_is_not() {
        let t = random_mps(6, 3, 21);
        let m2 = sre2(&t).unwrap();
        for j in [0usize, 2, 5] {
            let mut r = t.clone();
            r[j] = apply_ry(&t[j], RY_PERIOD);
            // Z is index 3 and H index 21 of `clifford_group()`; index 20 is [[h,−h],[h,h]] itself.
            let hz = apply_real_clifford(&apply_real_clifford(&t[j], 3).unwrap(), 21).unwrap();
            let g20 = apply_real_clifford(&t[j], 20).unwrap();
            for ((x, y), z) in r[j].data.iter().zip(&hz.data).zip(&g20.data) {
                let tol = 1e-14 * (1.0 + x.abs());
                assert!((x - y).abs() < tol && (x - z).abs() < tol, "R_y(π/4) is HZ: {x} vs {y} vs {z}");
            }
            let m2_hz = sre2(&r).unwrap();
            assert!((m2 - m2_hz).abs() < 1e-12, "site {j}: {m2} vs {m2_hz}");
            r[j] = apply_ry(&t[j], RY_PERIOD / 2.0);
            assert!((m2 - sre2(&r).unwrap()).abs() > 1e-4, "an eighth turn is not a Clifford");
        }
    }

    /// The landscape theorem the exact line search rests on: `2^{−M₂}(θ_j) = a + r cos(8θ_j − φ)`
    /// with the other sites fixed. Three readings fix the sinusoid; sixteen more at other angles
    /// agree with it to rounding, including at its predicted maximiser.
    #[test]
    fn one_sites_landscape_is_a_single_sinusoid_in_eight_theta() {
        let t = random_mps(7, 4, 22);
        let read = |j: usize, th: f64| -> f64 {
            let mut r = t.clone();
            r[j] = apply_ry(&t[j], th);
            (-sre2(&r).unwrap()).exp2()
        };
        for j in [0usize, 3, 6] {
            let (s0, s1, s2) = (read(j, 0.0), read(j, RY_PERIOD / 4.0), read(j, RY_PERIOD / 2.0));
            let a = 0.5 * (s0 + s2);
            let (rc, rs) = (0.5 * (s0 - s2), a - s1);
            let (r, u) = ((rc * rc + rs * rs).sqrt(), rs.atan2(rc));
            assert!(r > 1e-4, "site {j}: a flat landscape would not test the fit ({r:.3e})");
            for k in 0..16 {
                let th = std::f64::consts::PI * k as f64 / 16.0 + 0.0123;
                let predicted = a + r * (8.0 * th + u).cos();
                let got = read(j, th);
                assert!((got - predicted).abs() < 1e-13, "site {j}, θ = {th}: {got} vs {predicted}");
            }
            let th_star = (-u / 8.0).rem_euclid(RY_PERIOD);
            assert!((read(j, th_star) - (a + r)).abs() < 1e-13, "site {j}: the maximiser");
        }
    }

    /// The parity theorem: on a real state with `Π_j Z_j|ψ⟩ = ±|ψ⟩`, every site's sinusoid
    /// `a + r cos(8θ − φ)` has `φ = 0` — the identity frame is each site's own minimum — so the
    /// descent from the identity never moves and `M₂^nl = M₂`. The carrier is the even-parity
    /// part of a random MPS; the SAME random MPS before projection has generic phases (the
    /// theorem is not vacuous), and its parity expectation is measured, not assumed.
    #[test]
    fn on_a_parity_symmetric_state_the_identity_frame_is_every_sites_minimum() {
        let base = random_mps(5, 2, 77);
        let even = even_parity_mps(&base);
        let z_all: Vec<(usize, Op2)> = (0..5).map(|j| (j, PAULI_REAL[PAULI_Z])).collect();
        let parity = |t: &[TensorSite]| observables::expectation(t, &z_all) / observables::norm_squared(t);
        assert!((parity(&even) - 1.0).abs() < 1e-12, "even parity: {}", parity(&even));
        assert!(parity(&base).abs() < 0.9, "the unprojected state has no definite parity: {}", parity(&base));
        let phase = |t: &[TensorSite], j: usize| -> (f64, f64) {
            let read = |th: f64| {
                let mut r = t.to_vec();
                r[j] = apply_ry(&t[j], th);
                (-sre2(&r).unwrap()).exp2()
            };
            let (s0, s1, s2) = (read(0.0), read(RY_PERIOD / 4.0), read(RY_PERIOD / 2.0));
            let a = 0.5 * (s0 + s2);
            let (rc, rs) = (0.5 * (s0 - s2), a - s1);
            ((rc * rc + rs * rs).sqrt(), rs.atan2(rc))
        };
        let mut generic = 0;
        for j in 0..5 {
            let (r, u) = phase(&even, j);
            assert!(r > 1e-4 && u.abs() < 1e-10, "site {j}: r {r:.3e}, phase {u:.3e}");
            let (_, u_base) = phase(&base, j);
            generic += (u_base.abs() > 1e-2) as usize;
        }
        assert!(generic >= 3, "the unprojected phases are generic: {generic} of 5");
        let m2 = sre2(&even).unwrap();
        let nl = sre2_nonlocal_min(&even, 2).unwrap();
        assert!((nl.m2 - m2).abs() < 1e-12, "the frame removes nothing: {} vs {m2}", nl.m2);
        assert!(nl.angles.iter().all(|&a| a.min(RY_PERIOD - a) < 1e-8), "{:?}", nl.angles);
    }

    /// The identity frame's evaluation IS `sre2`'s reading, `M₂^nl` never exceeds it, a run of
    /// three sweeps is the prefix of a run of four, the count is `1 + 3N·sweeps`, and a
    /// malformed site order is refused by name.
    #[test]
    fn the_nonlocal_minimiser_is_deterministic_monotone_and_refuses_a_bad_order() {
        let t = random_mps(6, 3, 23);
        let m2 = sre2(&t).unwrap();
        let three = sre2_nonlocal_min(&t, 3).unwrap();
        let four = sre2_nonlocal_min(&t, 4).unwrap();
        assert_eq!(three.m2_identity, m2);
        assert_eq!(three.per_sweep[0], m2);
        assert!(three.m2 <= m2);
        assert_eq!(three.per_sweep, four.per_sweep[..4]);
        assert!(three.per_sweep.windows(2).all(|w| w[1] <= w[0]));
        assert_eq!(three.evaluations, 1 + 3 * 6 * 3);
        assert_eq!(four.evaluations, 1 + 3 * 6 * 4);
        assert!(three.angles.iter().all(|&a| (0.0..RY_PERIOD).contains(&a)));
        let again = sre2_nonlocal_min(&t, 3).unwrap();
        assert_eq!(three, again, "deterministic");
        let fwd: Vec<usize> = (0..6).collect();
        assert_eq!(sre2_nonlocal_min_ordered(&t, 3, &fwd).unwrap(), three);
        assert!(matches!(sre2_nonlocal_min_ordered(&t, 1, &[0, 1, 2]), Err(MagicError::SiteOrder { len: 3, n: 6 })));
        assert!(matches!(sre2_nonlocal_min_ordered(&t, 1, &[0, 1, 2, 3, 4, 4]), Err(MagicError::SiteOrder { .. })));
        assert!(matches!(sre2_nonlocal_min_ordered(&t, 1, &[0, 1, 2, 3, 4, 6]), Err(MagicError::SiteOrder { .. })));
        assert!(matches!(sre2_nonlocal_min(&random_mps(16, 12, 1), 1), Err(MagicError::Price { .. })));
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
