// The labelled two-site operator of q8-mps on the device — E14 item 5b.
//
// Four kernels, one per stage, each thread computing ONE output element from 0.0 with the
// dense loop order inside: channels in the dense order, the inner index ascending. They
// transliterate `q8_mps::blocks::CompactPlan::apply_reference` line for line, so the device
// bits are the host bits (compiled with -fmad=false, as every kernel here is). Only ψ goes
// up and Hψ comes down; the three intermediates live in SLOTS on the card — one contiguous
// r block per structurally-live (channel, l, physical pair) — and nothing dense is ever
// allocated or cleared.

extern "C" __global__ void q8_stage1(
    double *t1, const double *psi, int nr, int chi_l, int chi_r, int nrb, int max_rb,
    const int *l_block_of, const int *cut, const int *r_idx, const int *r_off,
    const int *right_map, const double *rtile, const long *rtile_off, const long *t1_off) {
    long total = (long)nr * chi_l * 4 * max_rb;
    long gstride = (long)gridDim.x * blockDim.x;
    for (long k = blockIdx.x * (long)blockDim.x + threadIdx.x; k < total; k += gstride) {
        int row = (int)(k % max_rb);
        long slot = k / max_rb;                 // (ci*chi_l + l)*4 + ab
        int ab = (int)(slot % 4);
        long cl = slot / 4;
        int l = (int)(cl % chi_l);
        int ci = (int)(cl / chi_l);
        long off = t1_off[slot];
        if (off < 0) continue;
        int lb = l_block_of[l];
        int rb_in = cut[lb * 4 + ab];
        int rb_out = right_map[ci * nrb + rb_in];
        int n_out = r_off[rb_out + 1] - r_off[rb_out];
        if (row >= n_out) continue;
        int n_in = r_off[rb_in + 1] - r_off[rb_in];
        const double *tile = rtile + rtile_off[ci * nrb + rb_in] + (long)row * n_in;
        const double *p = psi + ((long)l * 4 + ab) * chi_r;
        const int *rin = r_idx + r_off[rb_in];
        double acc = 0.0;
        for (int q = 0; q < n_in; ++q) acc += tile[q] * p[rin[q]];
        t1[off + row] = acc;
    }
}

extern "C" __global__ void q8_stage2(
    double *t2, const double *t1, int d_mid, int chi_l, int nlb, int max_rb,
    const int *l_block_of, const int *r_off, const int *t2_rb, const long *t2_off,
    const int *contrib_off, const int *contrib_ci, const int *contrib_b, const double *contrib_w,
    const long *t1_off) {
    long total = (long)d_mid * chi_l * 4 * max_rb;
    long gstride = (long)gridDim.x * blockDim.x;
    for (long k = blockIdx.x * (long)blockDim.x + threadIdx.x; k < total; k += gstride) {
        int pos = (int)(k % max_rb);
        long slot = k / max_rb;                 // (c1p*chi_l + l)*4 + at
        int at = (int)(slot % 4);
        long cl = slot / 4;
        int l = (int)(cl % chi_l);
        int c1p = (int)(cl / chi_l);
        long off = t2_off[slot];
        if (off < 0) continue;
        int lb = l_block_of[l];
        int rb = t2_rb[(c1p * nlb + lb) * 4 + at];
        int n = r_off[rb + 1] - r_off[rb];
        if (pos >= n) continue;
        int a = at / 2, t = at % 2;
        int c0 = contrib_off[(c1p * 2 + a) * 2 + t], c1 = contrib_off[(c1p * 2 + a) * 2 + t + 1];
        double acc = 0.0;
        for (int c = c0; c < c1; ++c) {
            int ci = contrib_ci[c], b = contrib_b[c];
            long o1 = t1_off[((long)ci * chi_l + l) * 4 + a * 2 + b];
            if (o1 < 0) continue;
            acc += contrib_w[c] * t1[o1 + pos];
        }
        t2[off + pos] = acc;
    }
}

extern "C" __global__ void q8_stage3(
    double *t3, const double *t2, int d_l, int chi_l, int nlb, int max_rb,
    const int *l_block_of, const int *r_off, const int *t3_rb, const long *t3_off,
    const int *contrib_off, const int *contrib_c1p, const int *contrib_a, const double *contrib_w,
    const long *t2_off) {
    long total = (long)d_l * chi_l * 4 * max_rb;
    long gstride = (long)gridDim.x * blockDim.x;
    for (long k = blockIdx.x * (long)blockDim.x + threadIdx.x; k < total; k += gstride) {
        int pos = (int)(k % max_rb);
        long slot = k / max_rb;                 // (c1*chi_l + l)*4 + st
        int st = (int)(slot % 4);
        long cl = slot / 4;
        int l = (int)(cl % chi_l);
        int c1 = (int)(cl / chi_l);
        long off = t3_off[slot];
        if (off < 0) continue;
        int lb = l_block_of[l];
        int rb = t3_rb[(c1 * nlb + lb) * 4 + st];
        int n = r_off[rb + 1] - r_off[rb];
        if (pos >= n) continue;
        int sb = st / 2, t = st % 2;
        int c0 = contrib_off[(c1 * 2 + sb) * 2 + t], cend = contrib_off[(c1 * 2 + sb) * 2 + t + 1];
        double acc = 0.0;
        for (int c = c0; c < cend; ++c) {
            int c1p = contrib_c1p[c], a = contrib_a[c];
            long o2 = t2_off[((long)c1p * chi_l + l) * 4 + a * 2 + t];
            if (o2 < 0) continue;
            acc += contrib_w[c] * t2[o2 + pos];
        }
        t3[off + pos] = acc;
    }
}

extern "C" __global__ void q8_stage4(
    double *out, const double *t3, int nl, int chi_l, int chi_r, int nlb, int max_rb,
    const int *l_block_of, const int *l_pos_of, const int *l_idx, const int *l_off,
    const int *r_idx, const int *r_off, const int *cut,
    const int *left_chan, const int *left_map, const double *ltile, const long *ltile_off,
    const long *t3_off) {
    long total = (long)chi_l * 4 * max_rb;
    long gstride = (long)gridDim.x * blockDim.x;
    for (long k = blockIdx.x * (long)blockDim.x + threadIdx.x; k < total; k += gstride) {
        int pos = (int)(k % max_rb);
        long ls = k / max_rb;                   // l_out*4 + st
        int st = (int)(ls % 4);
        int l_out = (int)(ls / 4);
        int lb_out = l_block_of[l_out];
        int rb = cut[lb_out * 4 + st];
        if (rb < 0) continue;
        int n = r_off[rb + 1] - r_off[rb];
        if (pos >= n) continue;
        int r_out = r_idx[r_off[rb] + pos];
        int row = l_pos_of[l_out];
        double total_v = 0.0;
        for (int ci = 0; ci < nl; ++ci) {
            int lb_in = left_map[ci * nlb + lb_out];
            if (lb_in < 0) continue;
            int c1 = left_chan[ci];
            int n_in = l_off[lb_in + 1] - l_off[lb_in];
            const double *tile = ltile + ltile_off[ci * nlb + lb_out] + (long)row * n_in;
            const int *lin = l_idx + l_off[lb_in];
            double acc = 0.0;
            for (int q = 0; q < n_in; ++q) {
                long o3 = t3_off[((long)c1 * chi_l + lin[q]) * 4 + st];
                double v = (o3 < 0) ? 0.0 : t3[o3 + pos];
                acc += tile[q] * v;
            }
            total_v += acc;
        }
        out[((long)l_out * 4 + st) * chi_r + r_out] = total_v;
    }
}
