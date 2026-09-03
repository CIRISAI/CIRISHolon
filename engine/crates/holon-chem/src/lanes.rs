//! THE determinant solver: one occupation string per conserved integer lane, generic over the
//! scalar tier, one per-determinant body shared by the host shards and the device kernel.
//!
//! # The holon reading, and why it is the general form
//!
//! `holon-swarm`'s law is that a shard is an arena, a boundary is geometry, and what crosses a
//! boundary is a conserved integer. A many-fermion Hamiltonian with `k` commuting particle-number
//! symmetries has exactly that shape: the determinant space is the PRODUCT of one occupation
//! string per conserved lane, every term of the Hamiltonian moves particles WITHIN lanes (it
//! commutes with each lane's count), and a term touching two lanes is a product of one single
//! excitation in each. So `sigma = H c` is a sum of blocks, every block batched over the lanes it
//! does not touch, and there is no halo: a shard on the slowest lane owns its output rows
//! outright. Chemistry's alpha/beta string factorisation (Knowles–Handy 1984, Olsen et al. 1988)
//! is the `k = 2` instance — [`crate::fci::FciSpace`] IS a two-lane [`LaneSpace`] — and a colour
//! gauge theory in a Cartan-neutral sector is `k = N_c` (`qcd2.rs`).
//!
//! # Convention — operator PRODUCTS, never normal-ordered
//!
//! ```text
//! H = Σ_l Σ_pq h^l_pq E^l_pq
//!   + Σ_{l<m} Σ_{pq,rs} g^{lm}_{pq,rs} E^l_pq E^m_rs
//!   + ½ Σ_l Σ_{pq,rs} g^{ll}_{pq,rs} E^l_pq E^l_rs            E^l_pq = a†_{l,p} a_{l,q}
//! ```
//!
//! with `g^{ll}_{pq,rs} = g^{ll}_{rs,pq}`. The chemistry Hamiltonian
//! `Σ h E + ½ Σ (pq|rs)(E_pq E_rs − δ_qr E_ps)` is this with `h^l = k` (the folded one-body
//! integral `ci_ints` builds) and `g = (pq|rs)` on every lane pair —
//! [`LaneHamiltonian::from_ci_parts`]. `tests/lanes_gauge.rs` asserts the two-lane kernel
//! against an independent dense Slater–Condon referee on random integral sets.
//!
//! # One representation, every tier, both devices
//!
//! [`LaneTables`] is the flat, integer-indexed form of a space and its Hamiltonian, and it is the
//! ONLY thing any kernel reads. [`sigma_det`] is the per-determinant body, generic over the
//! scalar so the double-double refinement tier (`tier::refine_determinant_dd`) runs the same
//! loops; the CPU shards call it directly and `holon-gpu`'s `lanes_sigma.cu` is its `f64`
//! transliteration, loop for loop, in the same order, compiled without fused multiply-add.
//! Bit-identity between one host shard, any number of host shards, and the device is therefore
//! a property of the construction — and it is MEASURED, never assumed (the gates compare by
//! `to_bits`).
//!
//! # The gather, and why there is no atomic and no merge
//!
//! Every sum is written from the OUTPUT determinant inward: the singles list of an output string
//! is walked, and the source string is read through the inverse map (`a†_p a_q |J> = s |K>`
//! inverts to `a†_q a_p |K> = s |J>`, same sign). A shard writes its own rows and reads anywhere.
//! That is `holon-mesh`'s condition for a bit-identical threaded run, and the reason its
//! reordering plant cannot fire here: there is no reduction whose order could move.

use crate::fci::{davidson_budget, CiInts, SolveExit, Strings, DAVIDSON_REQUESTED_TOLERANCE, MAX_ORB};
use crate::scalar::Scalar;
use crate::sigma_op::{DeviceClass, SigmaOp};

/// The most lanes a determinant index is decoded into — a fixed register array in both
/// kernels (`idx[MAX_LANES]` here, `HOLON_MAX_LANES` in `lanes_sigma.cu`, which must match; the
/// device wrapper checks the two at build time), so the bound is refused at construction
/// rather than discovered by a stack overflow in a launch.
pub const MAX_LANES: usize = 8;

// ------------------------------------------------------------------ the space

/// The product of `k` occupation-string spaces. Lane 0 is the slowest index.
///
/// [`crate::fci::FciSpace`] is this type: chemistry's alpha and beta strings are lanes 0 and 1,
/// reached by [`LaneSpace::alpha`] and [`LaneSpace::beta`].
pub struct LaneSpace {
    /// Orbital count of lane 0. Every constructor here builds lanes over ONE orbital set, so
    /// this is the space's orbital count; a caller assembling lanes over different sets through
    /// [`LaneSpace::from_lanes`] reads each lane's own `n_orb`.
    pub n_orb: usize,
    pub lanes: Vec<Strings>,
    /// Determinant stride of each lane: `index = Σ_l I_l · stride_l`.
    pub strides: Vec<usize>,
    pub n_det: usize,
}

impl LaneSpace {
    pub fn from_lanes(lanes: Vec<Strings>) -> LaneSpace {
        assert!(!lanes.is_empty() && lanes.len() <= MAX_LANES, "1..={MAX_LANES} lanes");
        let mut strides = vec![0usize; lanes.len()];
        let mut acc = 1usize;
        for l in (0..lanes.len()).rev() {
            strides[l] = acc;
            acc = acc.checked_mul(lanes[l].len()).expect("determinant count overflows usize");
        }
        LaneSpace { n_orb: lanes[0].n_orb, lanes, strides, n_det: acc }
    }

    /// `k` lanes over the same orbital count, with the given particle numbers.
    pub fn uniform(n_orb: usize, n_per_lane: &[usize]) -> LaneSpace {
        LaneSpace::from_lanes(n_per_lane.iter().map(|&e| Strings::new(n_orb, e)).collect())
    }

    /// The spin space: alpha and beta strings over `n_orb` orbitals.
    pub fn new(n_orb: usize, n_alpha: usize, n_beta: usize) -> LaneSpace {
        LaneSpace::with_mask_width(n_orb, n_alpha, n_beta, MAX_ORB)
    }

    /// The spin space a mask of `mask_width` bits can hold — the W1 plant's entry point; see
    /// [`Strings::with_mask_width`]. With `mask_width >= n_orb` this is exactly [`Self::new`].
    pub fn with_mask_width(n_orb: usize, n_alpha: usize, n_beta: usize, mask_width: usize) -> LaneSpace {
        LaneSpace::from_lanes(vec![
            Strings::with_mask_width(n_orb, n_alpha, mask_width),
            Strings::with_mask_width(n_orb, n_beta, mask_width),
        ])
    }

    pub fn n_lanes(&self) -> usize {
        self.lanes.len()
    }

    /// Lane 0 of a two-lane (spin) space.
    pub fn alpha(&self) -> &Strings {
        assert_eq!(self.lanes.len(), 2, "alpha/beta name the lanes of a two-lane space; this one has {}", self.lanes.len());
        &self.lanes[0]
    }

    /// Lane 1 of a two-lane (spin) space.
    pub fn beta(&self) -> &Strings {
        assert_eq!(self.lanes.len(), 2, "alpha/beta name the lanes of a two-lane space; this one has {}", self.lanes.len());
        &self.lanes[1]
    }

}

// ------------------------------------------------------------------ the Hamiltonian

/// `h^l` dense per lane and `g^{lm}` dense per unordered lane pair, accumulated by the caller
/// and compressed into [`LaneTables`] once. Dense: a lane of `n` orbitals costs `n⁴` per pair
/// here, which is what `ci_ints` already holds for the chemistry tensor.
pub struct LaneHamiltonian<T: Scalar> {
    pub n: Vec<usize>,
    pub h: Vec<Vec<T>>,
    /// `g[pair(l, m)]` for `l ≤ m`, row-major `(pq) × (rs)` with `pq = p·n_l + q`.
    pub g: Vec<Vec<T>>,
}

impl<T: Scalar> LaneHamiltonian<T> {
    pub fn new(n_orb: &[usize]) -> LaneHamiltonian<T> {
        let k = n_orb.len();
        let mut g = Vec::new();
        for l in 0..k {
            for m in l..k {
                g.push(vec![T::ZERO; n_orb[l] * n_orb[l] * n_orb[m] * n_orb[m]]);
            }
        }
        LaneHamiltonian { n: n_orb.to_vec(), h: n_orb.iter().map(|&n| vec![T::ZERO; n * n]).collect(), g }
    }

    /// Index of the unordered pair `(l, m)`, `l ≤ m`, in the order `new` laid them out.
    pub fn pair(&self, l: usize, m: usize) -> usize {
        assert!(l <= m && m < self.n.len(), "pair ({l}, {m}) with l ≤ m < {}", self.n.len());
        let k = self.n.len();
        (0..l).map(|i| k - i).sum::<usize>() + (m - l)
    }

    /// `h^l_pq += v`.
    pub fn one_body(&mut self, l: usize, p: usize, q: usize, v: T) {
        let n = self.n[l];
        self.h[l][p * n + q] = self.h[l][p * n + q] + v;
    }

    /// `g^{lm}_{pq,rs} += v` for `l ≤ m`: the coefficient of `E^l_pq E^m_rs` (with the ½ for
    /// `l = m` applied by the kernel, not here).
    pub fn two_body(&mut self, l: usize, p: usize, q: usize, m: usize, r: usize, s: usize, v: T) {
        let (nl, nm) = (self.n[l], self.n[m]);
        let idx = self.pair(l, m);
        let at = (p * nl + q) * nm * nm + (r * nm + s);
        self.g[idx][at] = self.g[idx][at] + v;
    }

    /// The chemistry Hamiltonian in lane form: `h^l = k` (the folded one-body integral),
    /// `g = (pq|rs)` on every pair, for `lanes` lanes over one orbital set of `n` orbitals. With
    /// two lanes it is the alpha/beta operator, term for term.
    pub fn from_ci_parts(n: usize, k: &[T], g: &[T], lanes: usize) -> LaneHamiltonian<T> {
        assert_eq!(k.len(), n * n, "k is not n²");
        assert_eq!(g.len(), n * n * n * n, "g is not n⁴");
        let mut ham = LaneHamiltonian::new(&vec![n; lanes]);
        for l in 0..lanes {
            ham.h[l].copy_from_slice(k);
            for m in l..lanes {
                let idx = ham.pair(l, m);
                ham.g[idx].copy_from_slice(g);
            }
        }
        ham
    }
}

impl LaneHamiltonian<f64> {
    /// [`Self::from_ci_parts`] on a [`CiInts`].
    pub fn from_ci_ints(ci: &CiInts, lanes: usize) -> LaneHamiltonian<f64> {
        LaneHamiltonian::from_ci_parts(ci.n, &ci.k, &ci.g, lanes)
    }
}

// ------------------------------------------------------------------ the flat tables

/// The flat form every kernel reads. Integer-indexed, with every per-lane array reached through
/// an offset, so a device can hold it as a handful of contiguous buffers.
///
/// Singles entries are stored TRANSPOSED: entry `e` of string `I` in lane `l` carries the
/// operator index `tp = q·n + p` such that `<I| E_tp |J_e> = sign_e` — the gather form — while
/// `at[off + I·n² + rs]` answers `E_rs |I> = sign |J>` (`J = −1` when the excitation vanishes).
/// Pair entries carry `sr`, the transpose of `rs`, because the kernel looks the second
/// operator up from ITS output string.
pub struct LaneTables<T: Scalar> {
    pub n_lanes: usize,
    pub n_det: usize,
    pub lane_n: Vec<i32>,
    pub lane_size: Vec<i32>,
    pub lane_ns: Vec<i32>,
    pub lane_stride: Vec<i64>,
    pub lane_off_singles: Vec<i64>,
    pub lane_off_at: Vec<i64>,
    pub lane_off_h: Vec<i32>,
    /// Pairs whose FIRST lane is `l`: `[lane_pair_ptr[l], lane_pair_ptr[l+1])` into `pair_*`.
    pub lane_pair_ptr: Vec<i32>,
    pub singles_tp: Vec<i32>,
    pub singles_sign: Vec<T>,
    pub singles_j: Vec<i32>,
    pub at_sign: Vec<T>,
    pub at_j: Vec<i32>,
    pub h: Vec<T>,
    pub pair_m: Vec<i32>,
    /// ½ for a same-lane pair, 1 otherwise — the convention's factor, carried as data so the
    /// kernel has one expression.
    pub pair_half: Vec<T>,
    pub pair_row_off: Vec<i32>,
    pub pair_ent_off: Vec<i32>,
    pub row_ptr: Vec<i32>,
    pub ent_sr: Vec<i32>,
    pub ent_coef: Vec<T>,
}

impl<T: Scalar> LaneTables<T> {
    pub fn build(space: &LaneSpace, ham: &LaneHamiltonian<T>) -> LaneTables<T> {
        let k = space.n_lanes();
        assert_eq!(ham.n.len(), k, "Hamiltonian lanes {} vs space lanes {k}", ham.n.len());
        let mut t = LaneTables {
            n_lanes: k,
            n_det: space.n_det,
            lane_n: Vec::new(),
            lane_size: Vec::new(),
            lane_ns: Vec::new(),
            lane_stride: Vec::new(),
            lane_off_singles: Vec::new(),
            lane_off_at: Vec::new(),
            lane_off_h: Vec::new(),
            lane_pair_ptr: vec![0],
            singles_tp: Vec::new(),
            singles_sign: Vec::new(),
            singles_j: Vec::new(),
            at_sign: Vec::new(),
            at_j: Vec::new(),
            h: Vec::new(),
            pair_m: Vec::new(),
            pair_half: Vec::new(),
            pair_row_off: Vec::new(),
            pair_ent_off: Vec::new(),
            row_ptr: Vec::new(),
            ent_sr: Vec::new(),
            ent_coef: Vec::new(),
        };
        for l in 0..k {
            let st = &space.lanes[l];
            let n = st.n_orb;
            assert_eq!(ham.n[l], n, "lane {l}: Hamiltonian over {} orbitals, strings over {n}", ham.n[l]);
            let ns = st.singles.first().map_or(0, |s| s.len());
            for (i, list) in st.singles.iter().enumerate() {
                assert_eq!(list.len(), ns, "lane {l} string {i}: ragged singles ({} vs {ns})", list.len());
            }
            t.lane_n.push(n as i32);
            t.lane_size.push(st.len() as i32);
            t.lane_ns.push(ns as i32);
            t.lane_stride.push(space.strides[l] as i64);
            t.lane_off_singles.push(t.singles_tp.len() as i64);
            t.lane_off_at.push(t.at_j.len() as i64);
            t.lane_off_h.push(t.h.len() as i32);
            t.h.extend_from_slice(&ham.h[l]);
            let n2 = n * n;
            let mut at_sign = vec![T::ZERO; st.len() * n2];
            let mut at_j = vec![-1i32; st.len() * n2];
            for (i, list) in st.singles.iter().enumerate() {
                for &(pq, sign, j) in list {
                    let (p, q) = ((pq as usize) / n, (pq as usize) % n);
                    t.singles_tp.push((q * n + p) as i32);
                    t.singles_sign.push(T::from_f64(sign));
                    t.singles_j.push(j as i32);
                    at_sign[i * n2 + pq as usize] = T::from_f64(sign);
                    at_j[i * n2 + pq as usize] = j as i32;
                }
            }
            t.at_sign.extend_from_slice(&at_sign);
            t.at_j.extend_from_slice(&at_j);
            for m in l..k {
                let nm = ham.n[m];
                let g = &ham.g[ham.pair(l, m)];
                t.pair_m.push(m as i32);
                t.pair_half.push(if m == l { T::from_f64(0.5) } else { T::ONE });
                t.pair_row_off.push(t.row_ptr.len() as i32);
                t.pair_ent_off.push(t.ent_sr.len() as i32);
                let mut ptr = 0i32;
                for pq in 0..n2 {
                    t.row_ptr.push(ptr);
                    for rs in 0..nm * nm {
                        let v = g[pq * nm * nm + rs];
                        if !v.is_zero() {
                            let (r, s) = (rs / nm, rs % nm);
                            t.ent_sr.push((s * nm + r) as i32);
                            t.ent_coef.push(v);
                            ptr += 1;
                        }
                    }
                }
                t.row_ptr.push(ptr);
            }
            t.lane_pair_ptr.push(t.pair_m.len() as i32);
        }
        t
    }

    /// The chemistry tables of a space: `k` lanes over one orbital set with the folded
    /// integrals `(k, g)`.
    pub fn for_ci_parts(space: &LaneSpace, k: &[T], g: &[T]) -> LaneTables<T> {
        LaneTables::build(space, &LaneHamiltonian::from_ci_parts(space.n_orb, k, g, space.n_lanes()))
    }

    /// Bytes the tables occupy, the number a device checks against its free memory before
    /// uploading (the two vectors are added by the caller).
    pub fn bytes(&self) -> u64 {
        let i32s = self.singles_tp.len() + self.singles_j.len() + self.at_j.len() + self.row_ptr.len() + self.ent_sr.len();
        let scalars = self.singles_sign.len() + self.at_sign.len() + self.h.len() + self.ent_coef.len();
        (i32s * 4 + scalars * std::mem::size_of::<T>()) as u64
    }
}

impl LaneTables<f64> {
    /// The chemistry tables of a space on a [`CiInts`].
    pub fn for_ci(space: &LaneSpace, ci: &CiInts) -> LaneTables<f64> {
        LaneTables::for_ci_parts(space, &ci.k, &ci.g)
    }
}

// ------------------------------------------------------------------ the kernel body

/// `sigma[k]` for one output determinant — THE kernel, in the order the device runs it.
///
/// With `DIAG` the same walk keeps only the terms whose source determinant is `k` itself and
/// reads `c` as 1, which is `<k|H|k>`: the preconditioner's diagonal from the same code path
/// as the operator, so the two cannot disagree about what the operator is.
#[inline]
pub fn sigma_det<T: Scalar, const DIAG: bool>(t: &LaneTables<T>, c: &[T], k: usize) -> T {
    let kk = k as i64;
    let mut idx = [0i64; MAX_LANES];
    let mut rem = kk;
    for l in 0..t.n_lanes {
        idx[l] = rem / t.lane_stride[l];
        rem -= idx[l] * t.lane_stride[l];
    }
    let mut acc = T::ZERO;
    for l in 0..t.n_lanes {
        let ns = t.lane_ns[l] as i64;
        let s_off = t.lane_off_singles[l] + idx[l] * ns;
        let h_off = t.lane_off_h[l] as i64;
        let stride_l = t.lane_stride[l];
        // The pair metadata of lane `l` is invariant over its singles: read once, per lane.
        let (p0, p1) = (t.lane_pair_ptr[l] as usize, t.lane_pair_ptr[l + 1] as usize);
        let mut pm = [(0usize, T::ZERO, 0i64, 0i64, 0i64, 0i64, 0i64); MAX_LANES];
        for (q, p) in (p0..p1).enumerate() {
            let m = t.pair_m[p] as usize;
            pm[q] = (
                m,
                t.pair_half[p],
                (t.lane_n[m] as i64) * (t.lane_n[m] as i64),
                t.lane_off_at[m],
                t.lane_stride[m],
                t.pair_row_off[p] as i64,
                t.pair_ent_off[p] as i64,
            );
        }
        for e in s_off..s_off + ns {
            let tp = t.singles_tp[e as usize] as i64;
            let s = t.singles_sign[e as usize];
            let j = t.singles_j[e as usize] as i64;
            let kj = kk + (j - idx[l]) * stride_l;
            if !DIAG || kj == kk {
                let cv = if DIAG { T::ONE } else { c[kj as usize] };
                acc = acc + t.h[(h_off + tp) as usize] * s * cv;
            }
            for &(m, half, nm2, at_off_m, stride_m, row_off, ent) in pm.iter().take(p1 - p0) {
                // On the diagonal a cross-lane term needs BOTH lanes at rest; with lane `l`
                // moved the whole row is provably zero, so it is not walked.
                if DIAG && m != l && kj != kk {
                    continue;
                }
                let (str_m, base, refidx) = if m == l { (j, kk, idx[l]) } else { (idx[m], kj, idx[m]) };
                let at_off = at_off_m + str_m * nm2;
                let row = (row_off + tp) as usize;
                for e2 in t.row_ptr[row]..t.row_ptr[row + 1] {
                    let a = (at_off + t.ent_sr[(ent + e2 as i64) as usize] as i64) as usize;
                    let jm = t.at_j[a] as i64;
                    if jm < 0 {
                        continue;
                    }
                    let s1 = t.at_sign[a];
                    let kj2 = base + (jm - refidx) * stride_m;
                    if !DIAG || kj2 == kk {
                        let cv = if DIAG { T::ONE } else { c[kj2 as usize] };
                        acc = acc + half * t.ent_coef[(ent + e2 as i64) as usize] * s * s1 * cv;
                    }
                }
            }
        }
    }
    acc
}

static LANE_THREADS_OVERRIDE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Set the process's host thread policy for every operator and space built afterwards — the
/// call an orchestrator that already runs `w` workers makes, with `cores / w`, so nested
/// parallelism does not oversubscribe the machine. Scheduling only: no thread count can reach
/// a bit. `0` restores the default (`LANE_THREADS`, else the machine's parallelism).
/// Split this machine between a producer's OWN worker pool and the lane kernel beneath it.
///
/// The nested-parallelism policy, stated once. Every solve shards its rows and its vector
/// algebra across `lane_threads()` threads by default; a producer that runs `workers` solves
/// at a time would multiply the two and oversubscribe the machine. Scheduling only — no
/// thread count can reach a bit (`lanes_gauge.rs` pins one shard against many).
///
/// The precedent is `holon-tables`' surface generator, which has carried this line since the
/// lane kernel landed; it became load-bearing for every producer on 2026-09-02, when
/// [`MIN_ROWS_PER_SHARD`] fell to 128 and pair-sized spaces started sharding at all.
pub fn set_lane_threads_for_pool(workers: usize) {
    set_lane_threads(
        (std::thread::available_parallelism().map_or(1, |n| n.get()) / workers.max(1)).max(1),
    );
}

pub fn set_lane_threads(threads: usize) {
    LANE_THREADS_OVERRIDE.store(threads, std::sync::atomic::Ordering::Relaxed);
}

/// The thread count the host kernels use: the process override, else `LANE_THREADS`, else the
/// machine's parallelism; 1 on wasm, where there are no threads to have.
pub fn lane_threads() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        1
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let o = LANE_THREADS_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed);
        if o > 0 {
            return o;
        }
        std::env::var("LANE_THREADS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&t| t >= 1)
            .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()))
    }
}

/// Rows a shard is worth spawning a thread for. A SCHEDULING choice: the shard count cannot
/// reach the answer (rows are disjoint and each row's order is fixed — `lanes_gauge.rs`
/// pins one shard against many to the bit), so this only decides when the spawn costs more
/// than the rows it would take.
///
/// MEASURED 2026-09-02 (`examples/sigma_price.rs`): at 2,025 determinants, ten orbitals,
/// one apply is 16–19 ms on one thread — a row is ~9 µs of scattered gathers, so 128 rows
/// are a millisecond of work against a ~30 µs spawn, and the (O,O) curve's Davidson, which
/// was paying that apply 462 times a knot, runs on every core the policy allows. The old
/// bound of 2,048 left every pair curve in the workbench's range single-threaded.
const MIN_ROWS_PER_SHARD: usize = 128;

/// `sigma = H c` (or the diagonal, with `DIAG`), sharded on contiguous output rows across up to
/// `threads` scoped threads. Each shard writes only its rows; the per-row order is
/// [`sigma_det`]'s whatever the shard count, so the result is bit-identical across `threads`.
pub fn sigma_rows<T: Scalar, const DIAG: bool>(t: &LaneTables<T>, c: &[T], out: &mut [T], threads: usize) {
    assert_eq!(out.len(), t.n_det);
    if !DIAG {
        assert_eq!(c.len(), t.n_det);
    }
    #[cfg(target_arch = "wasm32")]
    {
        // no threads to have: one shard, the same rows in the same order
        let _ = threads;
        for (k, o) in out.iter_mut().enumerate() {
            *o = sigma_det::<T, DIAG>(t, c, k);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let shards = threads.max(1).min(t.n_det.div_ceil(MIN_ROWS_PER_SHARD).max(1));
        if shards <= 1 {
            for (k, o) in out.iter_mut().enumerate() {
                *o = sigma_det::<T, DIAG>(t, c, k);
            }
            return;
        }
        std::thread::scope(|sc| {
            let chunk = t.n_det.div_ceil(shards).max(1);
            for (ci, slice) in out.chunks_mut(chunk).enumerate() {
                let k0 = ci * chunk;
                sc.spawn(move || {
                    for (i, o) in slice.iter_mut().enumerate() {
                        *o = sigma_det::<T, DIAG>(t, c, k0 + i);
                    }
                });
            }
        });
    }
}

/// The host operator: the tables plus a thread count. Owns its tables so a solve can be handed
/// one and nothing else.
pub struct LaneSigma<T: Scalar> {
    pub tables: LaneTables<T>,
    pub threads: usize,
}

impl<T: Scalar> LaneSigma<T> {
    pub fn new(space: &LaneSpace, ham: &LaneHamiltonian<T>, threads: usize) -> LaneSigma<T> {
        LaneSigma { tables: LaneTables::build(space, ham), threads }
    }

    /// The chemistry operator of a space on the folded integrals `(k, g)`, at the machine's
    /// thread count.
    pub fn for_ci_parts(space: &LaneSpace, k: &[T], g: &[T]) -> LaneSigma<T> {
        LaneSigma { tables: LaneTables::for_ci_parts(space, k, g), threads: lane_threads() }
    }

    /// `<k|H|k>` for every `k`, from the operator's own walk.
    pub fn diagonal(&self) -> Vec<T> {
        let mut d = vec![T::ZERO; self.tables.n_det];
        sigma_rows::<T, true>(&self.tables, &[], &mut d, self.threads);
        d
    }
}

impl LaneSigma<f64> {
    /// The chemistry operator of a space on a [`CiInts`].
    pub fn for_ci(space: &LaneSpace, ci: &CiInts) -> LaneSigma<f64> {
        LaneSigma::for_ci_parts(space, &ci.k, &ci.g)
    }
}

impl<T: Scalar> SigmaOp<T> for LaneSigma<T> {
    fn n_det(&self) -> usize {
        self.tables.n_det
    }
    fn device(&self) -> DeviceClass {
        DeviceClass::Cpu
    }
    fn apply(&mut self, c: &[T], sigma: &mut [T]) {
        sigma_rows::<T, false>(&self.tables, c, sigma, self.threads);
    }
}

// ------------------------------------------------------------------ the solve

/// What a lane solve returns. Not `fci::Solution`: that type carries the pair table's
/// derivative and CG fields, which a lane solve neither has nor pretends to.
#[derive(Clone, Debug)]
pub struct LaneSolution {
    pub energy: f64,
    pub vector: Vec<f64>,
    pub iters: usize,
    pub residual: f64,
    pub exit: SolveExit,
    pub device: DeviceClass,
    /// `min diag − E`: non-negative for the ground state, which the variational principle puts
    /// at or below every diagonal element; the one check that catches a clean convergence onto
    /// the wrong vector (a negative margin is a Ritz value above a determinant's own energy).
    pub variational_margin: f64,
}

/// The ground state through ANY sigma operator on these tables, the vectors on the HOST. The
/// device-resident solve is [`solve_lanes_in`] with a device space; this is the same call with
/// the host space and the operator's `apply`.
pub fn solve_lanes_with(
    op: &mut dyn SigmaOp<f64>,
    diag: &[f64],
    start: Option<&[f64]>,
    budget: usize,
    max_sub: usize,
) -> LaneSolution {
    if let Err(r) = crate::budget::admit(&crate::budget::price_determinant_with(op.n_det(), max_sub)) {
        panic!("solve_lanes: {r}");
    }
    let device = op.device();
    let sp = crate::vecspace::HostSpace::new();
    let mut apply = |c: &Vec<f64>, s: &mut Vec<f64>| op.apply(c, s);
    solve_lanes_in(&sp, &mut apply, diag, start, budget, max_sub, device)
}

/// The ground state on ANY space — host vectors or device-resident ones — through `apply`
/// (`sigma = H c` on that space's vectors). The Davidson body is `tier::davidson_in`, once.
/// The ADMISSION is the caller's: the working set lives in the space's memory (host RAM for
/// [`solve_lanes_with`], VRAM for the device arm), and only the arm knows which door to ask.
/// `device` is the class of the arithmetic that `apply` and the space perform, stamped on the
/// solution because it is the regime the numbers were produced under.
#[allow(clippy::too_many_arguments)]
pub fn solve_lanes_in<S: crate::vecspace::VectorSpace<f64>>(
    sp: &S,
    apply: &mut dyn FnMut(&S::V, &mut S::V),
    diag: &[f64],
    start: Option<&[f64]>,
    budget: usize,
    max_sub: usize,
    device: DeviceClass,
) -> LaneSolution {
    let (e, v, iters, residual, exit) =
        crate::tier::davidson_in(sp, apply, diag, DAVIDSON_REQUESTED_TOLERANCE, budget, start, max_sub);
    let min_diag = diag.iter().copied().fold(f64::INFINITY, f64::min);
    LaneSolution { energy: e, vector: v, iters, residual, exit, device, variational_margin: min_diag - e }
}

/// The host solve: tables, diagonal and Davidson at the default subspace bound, `threads` shards.
pub fn solve_lanes(space: &LaneSpace, ham: &LaneHamiltonian<f64>, threads: usize) -> LaneSolution {
    let mut op = LaneSigma::new(space, ham, threads);
    let diag = op.diagonal();
    solve_lanes_with(&mut op, &diag, None, davidson_budget(), crate::budget::DAVIDSON_SUBSPACE_MAX)
}
