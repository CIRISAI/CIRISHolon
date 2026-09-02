// The vector space on the device: the row programs of `holon_chem::vecspace`, transliterated,
// under the same reduction law.
//
// THE LAW. Every reduction is a sum of BLOCK PARTIALS: one thread owns one block of
// HOLON_DOT_BLOCK consecutive rows, walks them in order accumulating `acc = acc + x * y` from
// zero, and writes the block's partial; a second kernel sums the partials in block order on
// ONE thread. That is exactly `holon_chem::vecspace::blocked_dot` and exactly what a host thread
// per block range does, so the host and the device produce the same bits. Compiled with
// `-fmad=false` (build.rs) like the lane kernel: no contraction the host never performs.
//
// THE FOLD. A Davidson iteration is four row programs and the physics kernel. Each program
// reads every vector it touches ONCE: `ritz` forms x, r and corr for a row from the m basis
// vectors and their images and accumulates the row's contributions to ‖r‖² and Bᵀ·corr in the
// same pass; `deflate` subtracts the projections and accumulates the next Bᵀ·w; `deflate_norm`
// subtracts and accumulates ‖w‖²; `gram` accumulates Bᵀ·v. The m vectors arrive as an array
// of device pointers. Partials are laid out block-major: partials[block * k + q].
//
// HOLON_DOT_BLOCK must equal `holon_chem::vecspace::DOT_BLOCK`; the host wrapper refuses a
// mismatch at build. HOLON_MAX_M bounds the subspace a kernel can hold in registers; the host
// wrapper refuses a larger one (the Davidson's default bound is 48).

#define HOLON_DOT_BLOCK 256
#define HOLON_MAX_M 48

extern "C" __global__ void holon_vec_partials_dot(const double *a, const double *b, double *partials, long n) {
    long nblocks = (n + HOLON_DOT_BLOCK - 1) / HOLON_DOT_BLOCK;
    long gstride = (long)gridDim.x * blockDim.x;
    for (long blk = blockIdx.x * (long)blockDim.x + threadIdx.x; blk < nblocks; blk += gstride) {
        long i0 = blk * HOLON_DOT_BLOCK;
        long i1 = i0 + HOLON_DOT_BLOCK < n ? i0 + HOLON_DOT_BLOCK : n;
        double acc = 0.0;
        for (long i = i0; i < i1; ++i) {
            acc = acc + a[i] * b[i];
        }
        partials[blk] = acc;
    }
}

// out[q] = Σ_blk partials[blk * k + q], in block order, one thread per reduction q.
extern "C" __global__ void holon_vec_sum_partials(const double *partials, long nblocks, int k, double *out) {
    int q = blockIdx.x * blockDim.x + threadIdx.x;
    if (q < k) {
        double total = 0.0;
        for (long blk = 0; blk < nblocks; ++blk) {
            total = total + partials[blk * k + q];
        }
        out[q] = total;
    }
}

extern "C" __global__ void holon_vec_scale(double a, double *x, long n) {
    long gstride = (long)gridDim.x * blockDim.x;
    for (long i = blockIdx.x * (long)blockDim.x + threadIdx.x; i < n; i += gstride) {
        x[i] = x[i] * a;
    }
}

extern "C" __global__ void holon_vec_axpy(double a, const double *x, double *y, long n) {
    long gstride = (long)gridDim.x * blockDim.x;
    for (long i = blockIdx.x * (long)blockDim.x + threadIdx.x; i < n; i += gstride) {
        y[i] = y[i] + a * x[i];
    }
}

// x, r, corr for every row; partials[blk * (1 + m) + 0] = ‖r‖² of the block,
// partials[blk * (1 + m) + 1 + j] = Σ_block B_j[i] · corr_i.
extern "C" __global__ void holon_vec_ritz(const unsigned long long *bptrs, const unsigned long long *hbptrs, int m,
                                          const double *y, double theta, const double *diag,
                                          double *x, double *r, double *corr, double *partials, long n) {
    long nblocks = (n + HOLON_DOT_BLOCK - 1) / HOLON_DOT_BLOCK;
    long gstride = (long)gridDim.x * blockDim.x;
    for (long blk = blockIdx.x * (long)blockDim.x + threadIdx.x; blk < nblocks; blk += gstride) {
        long i0 = blk * HOLON_DOT_BLOCK;
        long i1 = i0 + HOLON_DOT_BLOCK < n ? i0 + HOLON_DOT_BLOCK : n;
        double rr = 0.0;
        double bc[HOLON_MAX_M];
        for (int j = 0; j < m; ++j) {
            bc[j] = 0.0;
        }
        for (long i = i0; i < i1; ++i) {
            double xi = 0.0;
            double hxi = 0.0;
            for (int j = 0; j < m; ++j) {
                if (y[j] != 0.0) {
                    xi = xi + y[j] * ((const double *)bptrs[j])[i];
                }
            }
            for (int j = 0; j < m; ++j) {
                if (y[j] != 0.0) {
                    hxi = hxi + y[j] * ((const double *)hbptrs[j])[i];
                }
            }
            double ri = hxi + (-theta) * xi;
            double d = theta - diag[i];
            double ci = fabs(d) > 1e-8 ? ri / d : ri;
            x[i] = xi;
            r[i] = ri;
            corr[i] = ci;
            rr = rr + ri * ri;
            for (int j = 0; j < m; ++j) {
                bc[j] = bc[j] + ((const double *)bptrs[j])[i] * ci;
            }
        }
        partials[blk * (1 + m)] = rr;
        for (int j = 0; j < m; ++j) {
            partials[blk * (1 + m) + 1 + j] = bc[j];
        }
    }
}

// w_i = w_i + Σ_j (−p_j) B_j[i]; partials[blk * m + j] = Σ_block B_j[i] · w_i (after).
extern "C" __global__ void holon_vec_deflate(const unsigned long long *bptrs, int m, const double *p,
                                             double *w, double *partials, long n) {
    long nblocks = (n + HOLON_DOT_BLOCK - 1) / HOLON_DOT_BLOCK;
    long gstride = (long)gridDim.x * blockDim.x;
    for (long blk = blockIdx.x * (long)blockDim.x + threadIdx.x; blk < nblocks; blk += gstride) {
        long i0 = blk * HOLON_DOT_BLOCK;
        long i1 = i0 + HOLON_DOT_BLOCK < n ? i0 + HOLON_DOT_BLOCK : n;
        double bw[HOLON_MAX_M];
        for (int j = 0; j < m; ++j) {
            bw[j] = 0.0;
        }
        for (long i = i0; i < i1; ++i) {
            double wi = w[i];
            for (int j = 0; j < m; ++j) {
                wi = wi + (-p[j]) * ((const double *)bptrs[j])[i];
            }
            w[i] = wi;
            for (int j = 0; j < m; ++j) {
                bw[j] = bw[j] + ((const double *)bptrs[j])[i] * wi;
            }
        }
        for (int j = 0; j < m; ++j) {
            partials[blk * m + j] = bw[j];
        }
    }
}

// w_i = w_i + Σ_j (−p_j) B_j[i]; partials[blk] = Σ_block w_i² (after).
extern "C" __global__ void holon_vec_deflate_norm(const unsigned long long *bptrs, int m, const double *p,
                                                  double *w, double *partials, long n) {
    long nblocks = (n + HOLON_DOT_BLOCK - 1) / HOLON_DOT_BLOCK;
    long gstride = (long)gridDim.x * blockDim.x;
    for (long blk = blockIdx.x * (long)blockDim.x + threadIdx.x; blk < nblocks; blk += gstride) {
        long i0 = blk * HOLON_DOT_BLOCK;
        long i1 = i0 + HOLON_DOT_BLOCK < n ? i0 + HOLON_DOT_BLOCK : n;
        double nn = 0.0;
        for (long i = i0; i < i1; ++i) {
            double wi = w[i];
            for (int j = 0; j < m; ++j) {
                wi = wi + (-p[j]) * ((const double *)bptrs[j])[i];
            }
            w[i] = wi;
            nn = nn + wi * wi;
        }
        partials[blk] = nn;
    }
}

// partials[blk * m + j] = Σ_block B_j[i] · v_i.
extern "C" __global__ void holon_vec_gram(const unsigned long long *bptrs, int m, const double *v,
                                          double *partials, long n) {
    long nblocks = (n + HOLON_DOT_BLOCK - 1) / HOLON_DOT_BLOCK;
    long gstride = (long)gridDim.x * blockDim.x;
    for (long blk = blockIdx.x * (long)blockDim.x + threadIdx.x; blk < nblocks; blk += gstride) {
        long i0 = blk * HOLON_DOT_BLOCK;
        long i1 = i0 + HOLON_DOT_BLOCK < n ? i0 + HOLON_DOT_BLOCK : n;
        double bv[HOLON_MAX_M];
        for (int j = 0; j < m; ++j) {
            bv[j] = 0.0;
        }
        for (long i = i0; i < i1; ++i) {
            double vi = v[i];
            for (int j = 0; j < m; ++j) {
                bv[j] = bv[j] + ((const double *)bptrs[j])[i] * vi;
            }
        }
        for (int j = 0; j < m; ++j) {
            partials[blk * m + j] = bv[j];
        }
    }
}
