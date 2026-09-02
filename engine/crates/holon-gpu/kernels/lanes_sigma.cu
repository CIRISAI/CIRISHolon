// The k-lane sigma kernel: `sigma = H c` for a product of occupation strings, one per conserved
// integer lane, on the device — the transliteration of `holon_chem::lanes::sigma_det`, loop for
// loop, in the same order.
//
// ============================ WHY THIS FILE IS A COPY AND MUST STAY ONE =====================
//
// The host body is the specification. Every loop here nests as it nests there, every product is
// written in the same left-to-right order, and this file is compiled with `-fmad=false`
// (build.rs) so the compiler cannot contract `acc += x * y` into a fused multiply-add the host
// never performs. Under those two conditions one CPU shard, any number of CPU shards, and this
// kernel produce the SAME BITS — a claim the gates in `tests/lanes_sigma.rs` measure by
// `to_bits`, not a nicety. Change the host body and this file together or the gate fires.
//
// ============================ WHY THERE IS NO ATOMIC AND NO REDUCTION =======================
//
// One thread owns one output determinant and walks its own singles lists inward; every read of
// `c` is a gather through the inverse excitation map. Nothing is scattered, so nothing collides
// and no completion order can reach the answer. The grid is stride-loop shaped: the launch
// geometry is a scheduling choice, not part of the artifact.
//
// Tables (all offsets in ELEMENTS, see `LaneTables`): per lane `l` — orbital count, singles per
// string, determinant stride, offsets into the singles and lookup arrays, offset into `h`, and
// the range of pairs whose first lane is `l`. `singles_*` carry the transposed operator index
// (`<K|E_tp|J> = sign`), `at_*` answer `E_rs|I> = sign |J>` with `J = -1` for none, and the pair
// entries carry `sr`, already transposed for the lookup on the second lane.

extern "C" __global__ void holon_lanes_sigma(
    double *sigma, const double *c, long n_det, int n_lanes, int diag,
    const int *lane_n, const int *lane_ns, const long *lane_stride,
    const long *lane_off_singles, const long *lane_off_at, const int *lane_off_h,
    const int *lane_pair_ptr,
    const int *singles_tp, const double *singles_sign, const int *singles_j,
    const double *at_sign, const int *at_j, const double *h,
    const int *pair_m, const double *pair_half, const int *pair_row_off, const int *pair_ent_off,
    const int *row_ptr, const int *ent_sr, const double *ent_coef) {
    long gstride = (long)gridDim.x * blockDim.x;
    for (long k = blockIdx.x * (long)blockDim.x + threadIdx.x; k < n_det; k += gstride) {
        long idx[8];
        long rem = k;
        for (int l = 0; l < n_lanes; ++l) {
            idx[l] = rem / lane_stride[l];
            rem -= idx[l] * lane_stride[l];
        }
        double acc = 0.0;
        for (int l = 0; l < n_lanes; ++l) {
            long ns = lane_ns[l];
            long s_off = lane_off_singles[l] + idx[l] * ns;
            long h_off = lane_off_h[l];
            long stride_l = lane_stride[l];
            for (long e = s_off; e < s_off + ns; ++e) {
                long tp = singles_tp[e];
                double s = singles_sign[e];
                long j = singles_j[e];
                long kj = k + (j - idx[l]) * stride_l;
                if (!diag || kj == k) {
                    double cv = diag ? 1.0 : c[kj];
                    acc += h[h_off + tp] * s * cv;
                }
                for (int p = lane_pair_ptr[l]; p < lane_pair_ptr[l + 1]; ++p) {
                    int m = pair_m[p];
                    double half = pair_half[p];
                    long nm2 = (long)lane_n[m] * (long)lane_n[m];
                    long str_m, base, refidx;
                    if (m == l) {
                        str_m = j;
                        base = k;
                        refidx = idx[l];
                    } else {
                        str_m = idx[m];
                        base = kj;
                        refidx = idx[m];
                    }
                    long at_off = lane_off_at[m] + str_m * nm2;
                    long stride_m = lane_stride[m];
                    long row = (long)pair_row_off[p] + tp;
                    long ent = pair_ent_off[p];
                    for (int e2 = row_ptr[row]; e2 < row_ptr[row + 1]; ++e2) {
                        long a = at_off + ent_sr[ent + e2];
                        long jm = at_j[a];
                        if (jm < 0) {
                            continue;
                        }
                        double s1 = at_sign[a];
                        long kj2 = base + (jm - refidx) * stride_m;
                        if (!diag || kj2 == k) {
                            double cv = diag ? 1.0 : c[kj2];
                            acc += half * ent_coef[ent + e2] * s * s1 * cv;
                        }
                    }
                }
            }
        }
        sigma[k] = acc;
    }
}
