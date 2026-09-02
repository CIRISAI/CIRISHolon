//! The vector algebra of a solve as ROW PROGRAMS over a SPACE, so one Davidson body runs on
//! host vectors and on device-resident ones — and the REDUCTION LAW that makes their bits agree.
//!
//! # The fold
//!
//! Read sideways, a Davidson iteration is a fixed program over the same few vectors: the `m`
//! basis vectors `B`, their images `HB`, the diagonal, and a handful of scalars from an `m×m`
//! eigenproblem. Every step is either row-local once those scalars are known, or a reduction
//! whose block partials can be formed in the same pass. So the iteration is four transforms
//! and the physics kernel, each transform reading every vector it needs ONCE:
//!
//! | transform | reads | writes | reductions |
//! |---|---|---|---|
//! | [`VectorSpace::ritz`]        | B, HB, diag | x, corr | ‖r‖², Bᵀ·corr |
//! | [`VectorSpace::deflate`]     | B, w        | w       | Bᵀ·w (for the second pass) |
//! | [`VectorSpace::deflate_norm`]| B, w        | w       | ‖w‖² |
//! | [`VectorSpace::gram_row`]    | B, v        | —       | Bᵀ·v |
//!
//! against roughly six `m` separate passes when the same work is written as dots and axpys.
//! Nothing about the arithmetic changes: each element sees the same additions in the same
//! order, so the folded program and the primitive program are the same bits.
//!
//! # The law
//!
//! A dot product is the one operation whose result depends on the order of its additions.
//! Every reduction here uses ONE fixed order, [`blocked_dot`]: consecutive blocks of
//! [`DOT_BLOCK`] elements, each block's partial the serial left-to-right sum of its products,
//! the total the serial left-to-right sum of the partials. A host thread per block range, a
//! device thread per block, or one host thread walking the blocks all perform the same
//! additions in the same order, so the answer is the same bits on all three — MEASURED
//! (`tests/vecspace_law.rs`, `holon-gpu/tests/vecspace.rs`), never assumed. No fused
//! multiply-add anywhere: the host writes `acc + x * y` and the device kernels are compiled
//! with `-fmad=false`, as the lane kernel is.

use crate::lanes::lane_threads;
use crate::scalar::Scalar;

/// The block of the reduction law. Part of the arithmetic regime: changing it changes bits.
pub const DOT_BLOCK: usize = 256;

/// The reduction law on one thread: block partials in order, then the partials in order.
pub fn blocked_dot<T: Scalar>(a: &[T], b: &[T]) -> T {
    assert_eq!(a.len(), b.len());
    let mut total = T::ZERO;
    for (ba, bb) in a.chunks(DOT_BLOCK).zip(b.chunks(DOT_BLOCK)) {
        let mut acc = T::ZERO;
        for (x, y) in ba.iter().zip(bb.iter()) {
            acc = acc + *x * *y;
        }
        total = total + acc;
    }
    total
}

/// The law's second half: the partials of reduction `q` out of `k` laid out block-major
/// (`partials[block * k + q]`), summed in block order into one scalar.
pub fn sum_partials<T: Scalar>(partials: &[T], k: usize, q: usize) -> T {
    let mut total = T::ZERO;
    let mut i = q;
    while i < partials.len() {
        total = total + partials[i];
        i += k;
    }
    total
}

/// A solve's vectors, wherever the space keeps them, and the row programs over them.
///
/// `B` is passed as a slice of vectors; every program walks rows `i` and, within a row, the
/// vectors in index order `j = 0..m`, which is the order the primitive form (`x += y_j b_j`,
/// `j` ascending) accumulates in. Reductions come back as scalars under the law.
pub trait VectorSpace<T: Scalar> {
    type V;
    fn len(&self, v: &Self::V) -> usize;
    fn zeros(&self, n: usize) -> Self::V;
    fn upload(&self, s: &[T]) -> Self::V;
    fn download(&self, v: &Self::V) -> Vec<T>;
    fn copy(&self, v: &Self::V) -> Self::V;
    /// `a · b` under the law.
    fn dot(&self, a: &Self::V, b: &Self::V) -> T;
    /// `x[i] = x[i] * a`.
    fn scale(&self, a: T, x: &mut Self::V);
    /// `y[i] = y[i] + a * x[i]`.
    fn axpy(&self, a: T, x: &Self::V, y: &mut Self::V);

    /// THE RITZ TRANSFORM. For row `i`:
    /// `x_i = Σ_j y_j B_j[i]`, `hx_i = Σ_j y_j HB_j[i]` (terms with `y_j = 0` skipped, `j`
    /// ascending, from zero), `r_i = hx_i + (−θ)·x_i`, `corr_i = r_i / (θ − d_i)` where
    /// `|θ − d_i| > 1e-8` else `r_i`. Writes `x`, `r` and `corr`; returns `(‖r‖², Bᵀ·corr)` under
    /// the law.
    #[allow(clippy::too_many_arguments)]
    fn ritz(&self, basis: &[Self::V], hbasis: &[Self::V], y: &[T], theta: T, diag: &Self::V, x: &mut Self::V, r: &mut Self::V, corr: &mut Self::V) -> (T, Vec<T>);

    /// `w_i = w_i + Σ_j (−p_j)·B_j[i]` (`j` ascending), returning `Bᵀ·w` afterwards under the law.
    fn deflate(&self, basis: &[Self::V], p: &[T], w: &mut Self::V) -> Vec<T>;

    /// `w_i = w_i + Σ_j (−p_j)·B_j[i]` (`j` ascending), returning `‖w‖²` afterwards under the law.
    fn deflate_norm(&self, basis: &[Self::V], p: &[T], w: &mut Self::V) -> T;

    /// `Bᵀ·v` under the law, every `B_j` and `v` read once.
    fn gram_row(&self, basis: &[Self::V], v: &Self::V) -> Vec<T>;

    fn norm(&self, a: &Self::V) -> T {
        self.dot(a, a).sqrt()
    }
}

// ------------------------------------------------------------------ the host space

/// Host vectors; elementwise work and block partials sharded across `threads`, the
/// partials always combined in block order on one thread.
pub struct HostSpace {
    pub threads: usize,
}

impl HostSpace {
    pub fn new() -> HostSpace {
        HostSpace { threads: lane_threads() }
    }

    pub fn with_threads(threads: usize) -> HostSpace {
        HostSpace { threads: threads.max(1) }
    }

    /// Rows worth a thread: below this the spawn costs more than the work. Scheduling only.
    const MIN_PER_THREAD: usize = 1 << 15;

    /// The chunk (a whole number of blocks) each shard walks, and the shard count.
    fn chunking(&self, n: usize) -> (usize, usize) {
        let shards = self.threads.max(1).min(n.div_ceil(Self::MIN_PER_THREAD).max(1));
        #[cfg(target_arch = "wasm32")]
        let shards = 1usize.min(shards);
        let chunk = n.div_ceil(shards).div_ceil(DOT_BLOCK).max(1) * DOT_BLOCK;
        (chunk, shards)
    }

    /// Runs a row program over disjoint chunks: `f(row0, len, partials_out)` where `partials_out`
    /// has one slot per block of the chunk per reduction (`k` reductions, block-major). Returns
    /// the reductions under the law.
    fn run<T: Scalar>(&self, n: usize, k: usize, f: impl Fn(usize, usize, &mut [T]) + Sync) -> Vec<T> {
        let (chunk, shards) = self.chunking(n);
        let nblocks = n.div_ceil(DOT_BLOCK);
        let mut partials = vec![T::ZERO; nblocks * k];
        if shards <= 1 {
            let _ = chunk;
            f(0, n, &mut partials);
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let per = (chunk / DOT_BLOCK) * k;
                std::thread::scope(|sc| {
                    if k == 0 {
                        for s in 0..shards {
                            let row0 = s * chunk;
                            if row0 >= n {
                                break;
                            }
                            let len = chunk.min(n - row0);
                            let f = &f;
                            sc.spawn(move || f(row0, len, &mut []));
                        }
                    } else {
                        for (s, ps) in partials.chunks_mut(per).enumerate() {
                            let row0 = s * chunk;
                            let len = chunk.min(n - row0);
                            let f = &f;
                            sc.spawn(move || f(row0, len, ps));
                        }
                    }
                });
            }
        }
        // the law: each reduction's partials in block order
        (0..k).map(|q| sum_partials(&partials, k, q)).collect()
    }
}

impl Default for HostSpace {
    fn default() -> Self {
        HostSpace::new()
    }
}

/// A raw pointer that may cross threads: the row programs write disjoint rows only, and a
/// closure captures the WHOLE wrapper through the method (not the bare pointer field).
#[derive(Clone, Copy)]
struct Rows<T>(*mut T);
unsafe impl<T> Send for Rows<T> {}
unsafe impl<T> Sync for Rows<T> {}
impl<T> Rows<T> {
    /// SAFETY: the caller hands each shard a disjoint `[row0, row0+len)`.
    #[inline]
    unsafe fn rows<'a>(self, row0: usize, len: usize) -> &'a mut [T] {
        std::slice::from_raw_parts_mut(self.0.add(row0), len)
    }
}

impl<T: Scalar> VectorSpace<T> for HostSpace {
    type V = Vec<T>;
    fn len(&self, v: &Vec<T>) -> usize {
        v.len()
    }
    fn zeros(&self, n: usize) -> Vec<T> {
        vec![T::ZERO; n]
    }
    fn upload(&self, s: &[T]) -> Vec<T> {
        s.to_vec()
    }
    fn download(&self, v: &Vec<T>) -> Vec<T> {
        v.clone()
    }
    fn copy(&self, v: &Vec<T>) -> Vec<T> {
        v.clone()
    }

    fn dot(&self, a: &Vec<T>, b: &Vec<T>) -> T {
        assert_eq!(a.len(), b.len());
        let n = a.len();
        let r = self.run::<T>(n, 1, |row0, len, ps| {
            for (q, (ba, bb)) in a[row0..row0 + len].chunks(DOT_BLOCK).zip(b[row0..row0 + len].chunks(DOT_BLOCK)).enumerate() {
                let mut acc = T::ZERO;
                for (x, y) in ba.iter().zip(bb.iter()) {
                    acc = acc + *x * *y;
                }
                ps[q] = acc;
            }
        });
        r[0]
    }

    fn scale(&self, a: T, x: &mut Vec<T>) {
        let n = x.len();
        let px = Rows(x.as_mut_ptr());
        self.run::<T>(n, 0, |row0, len, _| {
            // SAFETY: rows [row0, row0+len) belong to this shard alone
            let xs = unsafe { px.rows(row0, len) };
            for xi in xs.iter_mut() {
                *xi = *xi * a;
            }
        });
    }

    fn axpy(&self, a: T, x: &Vec<T>, y: &mut Vec<T>) {
        assert_eq!(x.len(), y.len());
        let n = y.len();
        let py = Rows(y.as_mut_ptr());
        self.run::<T>(n, 0, |row0, len, _| {
            let ys = unsafe { py.rows(row0, len) };
            for (yi, xi) in ys.iter_mut().zip(&x[row0..row0 + len]) {
                *yi = *yi + a * *xi;
            }
        });
    }

    fn ritz(&self, basis: &[Vec<T>], hbasis: &[Vec<T>], y: &[T], theta: T, diag: &Vec<T>, x: &mut Vec<T>, r: &mut Vec<T>, corr: &mut Vec<T>) -> (T, Vec<T>) {
        let m = basis.len();
        let n = diag.len();
        let (px, pr, pc) = (Rows(x.as_mut_ptr()), Rows(r.as_mut_ptr()), Rows(corr.as_mut_ptr()));
        let live: Vec<usize> = (0..m).filter(|&j| !y[j].is_zero()).collect();
        let r = self.run::<T>(n, 1 + m, |row0, len, ps| {
            let xs = unsafe { px.rows(row0, len) };
            let rs = unsafe { pr.rows(row0, len) };
            let cs = unsafe { pc.rows(row0, len) };
            for (q, blk) in (0..len).step_by(DOT_BLOCK).enumerate() {
                let end = (blk + DOT_BLOCK).min(len);
                let mut rr = T::ZERO;
                let mut bc = vec![T::ZERO; m];
                for t in blk..end {
                    let i = row0 + t;
                    let mut xi = T::ZERO;
                    let mut hxi = T::ZERO;
                    for &j in &live {
                        xi = xi + y[j] * basis[j][i];
                    }
                    for &j in &live {
                        hxi = hxi + y[j] * hbasis[j][i];
                    }
                    let ri = hxi + (-theta) * xi;
                    let d = theta - diag[i];
                    let ci = if d.abs().to_f64() > 1e-8 { ri / d } else { ri };
                    xs[t] = xi;
                    rs[t] = ri;
                    cs[t] = ci;
                    rr = rr + ri * ri;
                    for j in 0..m {
                        bc[j] = bc[j] + basis[j][i] * ci;
                    }
                }
                ps[q * (1 + m)] = rr;
                ps[q * (1 + m) + 1..q * (1 + m) + 1 + m].copy_from_slice(&bc);
            }
        });
        (r[0], r[1..].to_vec())
    }

    fn deflate(&self, basis: &[Vec<T>], p: &[T], w: &mut Vec<T>) -> Vec<T> {
        let m = basis.len();
        let n = w.len();
        let pw = Rows(w.as_mut_ptr());
        self.run::<T>(n, m, |row0, len, ps| {
            let ws = unsafe { pw.rows(row0, len) };
            for (q, blk) in (0..len).step_by(DOT_BLOCK).enumerate() {
                let end = (blk + DOT_BLOCK).min(len);
                let mut bw = vec![T::ZERO; m];
                for t in blk..end {
                    let i = row0 + t;
                    let mut wi = ws[t];
                    for j in 0..m {
                        wi = wi + (-p[j]) * basis[j][i];
                    }
                    ws[t] = wi;
                    for j in 0..m {
                        bw[j] = bw[j] + basis[j][i] * wi;
                    }
                }
                ps[q * m..q * m + m].copy_from_slice(&bw);
            }
        })
    }

    fn deflate_norm(&self, basis: &[Vec<T>], p: &[T], w: &mut Vec<T>) -> T {
        let m = basis.len();
        let n = w.len();
        let pw = Rows(w.as_mut_ptr());
        let r = self.run::<T>(n, 1, |row0, len, ps| {
            let ws = unsafe { pw.rows(row0, len) };
            for (q, blk) in (0..len).step_by(DOT_BLOCK).enumerate() {
                let end = (blk + DOT_BLOCK).min(len);
                let mut nn = T::ZERO;
                for t in blk..end {
                    let i = row0 + t;
                    let mut wi = ws[t];
                    for j in 0..m {
                        wi = wi + (-p[j]) * basis[j][i];
                    }
                    ws[t] = wi;
                    nn = nn + wi * wi;
                }
                ps[q] = nn;
            }
        });
        r[0]
    }

    fn gram_row(&self, basis: &[Vec<T>], v: &Vec<T>) -> Vec<T> {
        let m = basis.len();
        let n = v.len();
        self.run::<T>(n, m, |row0, len, ps| {
            for (q, blk) in (0..len).step_by(DOT_BLOCK).enumerate() {
                let end = (blk + DOT_BLOCK).min(len);
                let mut bv = vec![T::ZERO; m];
                for t in blk..end {
                    let i = row0 + t;
                    let vi = v[i];
                    for j in 0..m {
                        bv[j] = bv[j] + basis[j][i] * vi;
                    }
                }
                ps[q * m..q * m + m].copy_from_slice(&bv);
            }
        })
    }
}
