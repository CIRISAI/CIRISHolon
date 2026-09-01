// SATURATION-3 G2: the sparse Hamiltonian matvec at the (O,O,O) scale, on the GPU,
// measured against the CPU on the real table path's own data.
//
// Loads what holon-chem/examples/s3_sigma_export.rs wrote: the real excitation lists, the
// real integrals, a probe vector, and the CPU's own sigma on it. The GPU answer is checked
// against that reference before any timing is reported.
//
// ============================ THE ALGEBRA, AND WHY IT IS THREE GEMMS ======================
//
// `sigma_direct` has three blocks. Written out, two of them are dense GEMMs against
// matrices that do not depend on c at all, so they can be built ONCE per geometry:
//
//   beta same-spin :  Sigma += C * F_b          F_b[jb][ib], (nb x nb)
//   alpha same-spin:  Sigma += F_a^T * C        F_a[ja][ia], (na x na)
//   mixed          :  Sigma[ia][ib] += sum_m sa[ia][m] * D[ja][m][ib]
//                     with D[ja] = A[ja] * T[ja], A (ns_a x n2) also c-independent
//
// Only T depends on c, and T[ja][kl][ib] = sign * c[ja][jb(kl,ib)] is a pure GATHER: for a
// fixed excitation kl and destination ib, the source string jb is UNIQUE, because
// a+_p a_q |jb> = s|ib> inverts to |jb> = a+_q a_p |ib>. So there is no summation to order.
//
// ============================ WHY THERE IS NOT ONE ATOMIC IN HERE =========================
//
// The determinism gate is the adoption condition, not a nicety. The obvious GPU
// implementation scatters — build T by scattering c through the beta singles, and
// accumulate sigma by scattering D through the alpha singles — and both scatters collide,
// so both need atomicAdd. Floating-point atomicAdd accumulates in completion order, which
// is not reproducible, and a table generated that way would not be bit-identical even to
// ITSELF across two runs, let alone across shard counts.
//
// Both scatters are therefore inverted into GATHERS, which is possible for exactly the
// reason above: the excitation maps are invertible. Every sum in this file is over a fixed
// index range in a fixed order, and cuBLAS is deterministic for fixed shapes on fixed
// hardware. Bit-identity run-to-run is then a property of the construction, and it is
// MEASURED below rather than assumed.

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cmath>
#include <vector>
#include <cuda_runtime.h>
#include <cublas_v2.h>

#define CK(x) do { cudaError_t e = (x); if (e != cudaSuccess) { \
    printf("CUDA error %s at %d\n", cudaGetErrorString(e), __LINE__); exit(1);} } while(0)
#define CB(x) do { cublasStatus_t s = (x); if (s != CUBLAS_STATUS_SUCCESS) { \
    printf("cuBLAS error %d at %d\n", (int)s, __LINE__); exit(1);} } while(0)

struct Single { uint32_t pq, dst; double sign; };

// T[ja][kl][ib] = sign(kl,ib) * c[ja][jb(kl,ib)], a pure gather. Writes are coalesced in
// ib; the read of c is a gather inside one 455-element row, which stays in cache.
__global__ void build_T(double *T, const double *c, const int *src_jb,
                        const double *src_sign, int na, int nb, int n2) {
    long total = (long)na * n2 * nb;
    long stride = (long)gridDim.x * blockDim.x;
    for (long idx = blockIdx.x * (long)blockDim.x + threadIdx.x; idx < total; idx += stride) {
        long ib = idx % nb;
        long kl = (idx / nb) % n2;
        long ja = idx / ((long)nb * n2);
        int jb = src_jb[kl * nb + ib];
        T[idx] = (jb >= 0) ? src_sign[kl * nb + ib] * c[ja * (long)nb + jb] : 0.0;
    }
}

// Sigma[ia][ib] += sum_t sign * D[ja][m][ib], gathered over the TRANSPOSED alpha singles so
// that nothing collides and the summation order is fixed.
__global__ void gather_mixed(double *sigma, const double *D, const int *at_ja,
                             const int *at_m, const double *at_sign,
                             int na, int nb, int ns_a) {
    long total = (long)na * nb;
    long stride = (long)gridDim.x * blockDim.x;
    for (long idx = blockIdx.x * (long)blockDim.x + threadIdx.x; idx < total; idx += stride) {
        long ib = idx % nb;
        long ia = idx / nb;
        double acc = 0.0;
        for (int t = 0; t < ns_a; ++t) {
            long e = ia * (long)ns_a + t;
            acc += at_sign[e] * D[((long)at_ja[e] * ns_a + at_m[e]) * nb + ib];
        }
        sigma[idx] += acc;
    }
}

static void read_exact(FILE *f, void *p, size_t n, const char *what) {
    if (fread(p, 1, n, f) != n) { printf("short read on %s\n", what); exit(1); }
}

int main(int argc, char **argv) {
    const char *path = (argc > 1) ? argv[1] : "ooo.bin";
    int reps = (argc > 2) ? atoi(argv[2]) : 20;

    FILE *f = fopen(path, "rb");
    if (!f) { printf("cannot open %s\n", path); return 1; }
    uint64_t hdr[6];
    read_exact(f, hdr, sizeof(hdr), "header");
    const int n = (int)hdr[0], na = (int)hdr[1], nb = (int)hdr[2];
    const int ns_a = (int)hdr[3], ns_b = (int)hdr[4];
    const long n_det = (long)hdr[5];
    const int n2 = n * n;
    printf("loaded            n_orb %d, na %d, nb %d, ns_a %d, ns_b %d, n_det %ld\n",
           n, na, nb, ns_a, ns_b, n_det);

    std::vector<double> k(n2), g((size_t)n2 * n2);
    read_exact(f, k.data(), k.size() * 8, "k");
    read_exact(f, g.data(), g.size() * 8, "g");
    std::vector<Single> as((size_t)na * ns_a), bs((size_t)nb * ns_b);
    read_exact(f, as.data(), as.size() * sizeof(Single), "alpha singles");
    read_exact(f, bs.data(), bs.size() * sizeof(Single), "beta singles");
    std::vector<double> cvec(n_det), sigma_cpu(n_det);
    read_exact(f, cvec.data(), n_det * 8, "c");
    read_exact(f, sigma_cpu.data(), n_det * 8, "sigma_cpu");
    fclose(f);

    // ---------------- host-side precomputation, all c-independent ----------------

    // The beta gather map. Asserted unique: two sources for one (kl, ib) would mean a
    // summation had been silently dropped, and the GPU answer would be wrong in a way the
    // reference check might not localise.
    std::vector<int> src_jb((size_t)n2 * nb, -1);
    std::vector<double> src_sign((size_t)n2 * nb, 0.0);
    for (int jb = 0; jb < nb; ++jb) {
        for (int t = 0; t < ns_b; ++t) {
            const Single &s = bs[(size_t)jb * ns_b + t];
            size_t e = (size_t)s.pq * nb + s.dst;
            if (src_jb[e] != -1) { printf("beta gather map is not injective\n"); return 1; }
            src_jb[e] = jb;
            src_sign[e] = s.sign;
        }
    }

    // A[ja][m][kl] = g[pq(ja,m)][kl] — the rows of g this alpha string's excitations select.
    std::vector<double> A((size_t)na * ns_a * n2);
    for (int ja = 0; ja < na; ++ja)
        for (int m = 0; m < ns_a; ++m)
            memcpy(&A[((size_t)ja * ns_a + m) * n2],
                   &g[(size_t)as[(size_t)ja * ns_a + m].pq * n2], n2 * 8);

    // The transposed alpha singles. Uniform ns_a per destination for the same invertibility
    // reason; asserted, because the flat layout depends on it.
    std::vector<int> at_ja((size_t)na * ns_a), at_m((size_t)na * ns_a), fill(na, 0);
    std::vector<double> at_sign((size_t)na * ns_a);
    for (int ja = 0; ja < na; ++ja) {
        for (int m = 0; m < ns_a; ++m) {
            const Single &s = as[(size_t)ja * ns_a + m];
            int ia = (int)s.dst;
            if (fill[ia] >= ns_a) { printf("alpha transpose overflows at ia=%d\n", ia); return 1; }
            size_t e = (size_t)ia * ns_a + fill[ia]++;
            at_ja[e] = ja; at_m[e] = m; at_sign[e] = s.sign;
        }
    }
    for (int ia = 0; ia < na; ++ia)
        if (fill[ia] != ns_a) { printf("alpha transpose is ragged at ia=%d (%d)\n", ia, fill[ia]); return 1; }

    // The two same-spin matrices, exactly as sigma_direct builds their inner vector `f`.
    std::vector<double> Fb((size_t)nb * nb, 0.0), Fa((size_t)na * na, 0.0);
    for (int jb = 0; jb < nb; ++jb) {
        double *row = &Fb[(size_t)jb * nb];
        for (int t = 0; t < ns_b; ++t) {
            const Single &s1 = bs[(size_t)jb * ns_b + t];
            row[s1.dst] += s1.sign * k[s1.pq];
            for (int t2 = 0; t2 < ns_b; ++t2) {
                const Single &s2 = bs[(size_t)s1.dst * ns_b + t2];
                row[s2.dst] += 0.5 * s1.sign * s2.sign * g[(size_t)s2.pq * n2 + s1.pq];
            }
        }
    }
    for (int ja = 0; ja < na; ++ja) {
        double *row = &Fa[(size_t)ja * na];
        for (int t = 0; t < ns_a; ++t) {
            const Single &s1 = as[(size_t)ja * ns_a + t];
            row[s1.dst] += s1.sign * k[s1.pq];
            for (int t2 = 0; t2 < ns_a; ++t2) {
                const Single &s2 = as[(size_t)s1.dst * ns_a + t2];
                row[s2.dst] += 0.5 * s1.sign * s2.sign * g[(size_t)s2.pq * n2 + s1.pq];
            }
        }
    }

    // ---------------- upload ----------------
    double *dT, *dA, *dD, *dFa, *dFb, *dC, *dSig, *dSrcSign, *dAtSign;
    int *dSrcJb, *dAtJa, *dAtM;
    size_t szT = (size_t)na * n2 * nb, szA = (size_t)na * ns_a * n2, szD = (size_t)na * ns_a * nb;
    printf("device operands   T %.1f MB, A %.1f MB, D %.1f MB, F %.1f MB, total %.1f MB\n",
           szT * 8 / 1e6, szA * 8 / 1e6, szD * 8 / 1e6,
           ((size_t)nb * nb + (size_t)na * na) * 8 / 1e6,
           (szT + szA + szD + (size_t)nb * nb + (size_t)na * na + 2 * n_det) * 8 / 1e6);

    CK(cudaMalloc(&dT, szT * 8));  CK(cudaMalloc(&dA, szA * 8));  CK(cudaMalloc(&dD, szD * 8));
    CK(cudaMalloc(&dFa, (size_t)na * na * 8)); CK(cudaMalloc(&dFb, (size_t)nb * nb * 8));
    CK(cudaMalloc(&dC, n_det * 8)); CK(cudaMalloc(&dSig, n_det * 8));
    CK(cudaMalloc(&dSrcJb, (size_t)n2 * nb * 4)); CK(cudaMalloc(&dSrcSign, (size_t)n2 * nb * 8));
    CK(cudaMalloc(&dAtJa, (size_t)na * ns_a * 4)); CK(cudaMalloc(&dAtM, (size_t)na * ns_a * 4));
    CK(cudaMalloc(&dAtSign, (size_t)na * ns_a * 8));

    CK(cudaMemcpy(dA, A.data(), szA * 8, cudaMemcpyHostToDevice));
    CK(cudaMemcpy(dFa, Fa.data(), (size_t)na * na * 8, cudaMemcpyHostToDevice));
    CK(cudaMemcpy(dFb, Fb.data(), (size_t)nb * nb * 8, cudaMemcpyHostToDevice));
    CK(cudaMemcpy(dSrcJb, src_jb.data(), (size_t)n2 * nb * 4, cudaMemcpyHostToDevice));
    CK(cudaMemcpy(dSrcSign, src_sign.data(), (size_t)n2 * nb * 8, cudaMemcpyHostToDevice));
    CK(cudaMemcpy(dAtJa, at_ja.data(), (size_t)na * ns_a * 4, cudaMemcpyHostToDevice));
    CK(cudaMemcpy(dAtM, at_m.data(), (size_t)na * ns_a * 4, cudaMemcpyHostToDevice));
    CK(cudaMemcpy(dAtSign, at_sign.data(), (size_t)na * ns_a * 8, cudaMemcpyHostToDevice));
    CK(cudaMemcpy(dC, cvec.data(), n_det * 8, cudaMemcpyHostToDevice));

    cublasHandle_t h; CB(cublasCreate(&h));
    const double one = 1.0, zero = 0.0;

    // One sigma, in a FIXED order: beta same-spin, then alpha same-spin, then mixed.
    auto sigma_gpu = [&]() {
        // Sigma = C * F_b        (row-major m=na, k=nb, n=nb)
        CB(cublasDgemm(h, CUBLAS_OP_N, CUBLAS_OP_N, nb, na, nb,
                       &one, dFb, nb, dC, nb, &zero, dSig, nb));
        // Sigma += F_a^T * C     (row-major m=na, k=na, n=nb; F_a transposed)
        CB(cublasDgemm(h, CUBLAS_OP_N, CUBLAS_OP_T, nb, na, na,
                       &one, dC, nb, dFa, na, &one, dSig, nb));
        // T <- gather(c)
        build_T<<<4096, 256>>>(dT, dC, dSrcJb, dSrcSign, na, nb, n2);
        // D[ja] = A[ja] * T[ja]  (row-major m=ns_a, k=n2, n=nb), batched over alpha strings
        CB(cublasDgemmStridedBatched(h, CUBLAS_OP_N, CUBLAS_OP_N, nb, ns_a, n2,
                                     &one, dT, nb, (long long)n2 * nb,
                                     dA, n2, (long long)ns_a * n2,
                                     &zero, dD, nb, (long long)ns_a * nb, na));
        // Sigma += gather(D)
        gather_mixed<<<2048, 256>>>(dSig, dD, dAtJa, dAtM, dAtSign, na, nb, ns_a);
    };

    // ---------------- correctness against the CPU's own answer ----------------
    sigma_gpu(); CK(cudaDeviceSynchronize());
    std::vector<double> got(n_det);
    CK(cudaMemcpy(got.data(), dSig, n_det * 8, cudaMemcpyDeviceToHost));

    double max_abs = 0, max_rel = 0, ref_max = 0;
    long worst = -1;
    for (long i = 0; i < n_det; ++i) ref_max = fmax(ref_max, fabs(sigma_cpu[i]));
    for (long i = 0; i < n_det; ++i) {
        double d = fabs(got[i] - sigma_cpu[i]);
        if (d > max_abs) { max_abs = d; worst = i; }
        max_rel = fmax(max_rel, d / fmax(fabs(sigma_cpu[i]), 1e-300));
    }
    printf("\n--- agreement with the CPU sigma (the real table path's own answer) ---\n");
    printf("max |sigma_gpu - sigma_cpu|   %.6e   (at index %ld)\n", max_abs, worst);
    printf("max |sigma| in the reference  %.6e\n", ref_max);
    printf("relative to that scale        %.3e\n", max_abs / ref_max);
    // How many entries differ BITWISE. This is the number that decides what adoption costs
    // G1: if the GPU sigma is not bit-identical to the CPU's, then a Davidson driven by it
    // follows a different path, and a GPU-built table is a different artifact from a
    // CPU-built one -- so the two may not be mixed inside one table.
    long bitdiff = 0;
    for (long i = 0; i < n_det; ++i)
        if (memcmp(&got[i], &sigma_cpu[i], 8) != 0) bitdiff++;
    printf("entries differing BITWISE     %ld of %ld (%.1f%%)\n",
           bitdiff, n_det, 100.0 * bitdiff / n_det);
    if (!(max_abs / ref_max < 1e-12)) {
        printf("REFUSED: the GPU kernel does not reproduce the CPU sigma. No timing is \n"
               "reported, because a fast wrong answer is not a result.\n");
        return 1;
    }

    // ---------------- determinism: bit-identity run to run ----------------
    std::vector<double> again(n_det);
    int identical = 1;
    for (int r = 0; r < 5 && identical; ++r) {
        sigma_gpu(); CK(cudaDeviceSynchronize());
        CK(cudaMemcpy(again.data(), dSig, n_det * 8, cudaMemcpyDeviceToHost));
        for (long i = 0; i < n_det; ++i)
            if (memcmp(&again[i], &got[i], 8) != 0) { identical = 0; break; }
    }
    printf("\n--- the determinism gate ---\n");
    printf("five repeat runs bit-identical: %s\n", identical ? "YES" : "NO");
    if (!identical) printf("  (adoption would require a fixed reduction order; this does not have one)\n");

    // ---------------- timing ----------------
    cudaEvent_t a, b; cudaEventCreate(&a); cudaEventCreate(&b);
    CK(cudaEventRecord(a));
    for (int r = 0; r < reps; ++r) sigma_gpu();
    CK(cudaEventRecord(b)); CK(cudaEventSynchronize(b));
    float ms; cudaEventElapsedTime(&ms, a, b); ms /= reps;

    // With the host round trip a real Davidson iteration would pay: c up, sigma down.
    CK(cudaEventRecord(a));
    for (int r = 0; r < reps; ++r) {
        CK(cudaMemcpy(dC, cvec.data(), n_det * 8, cudaMemcpyHostToDevice));
        sigma_gpu();
        CK(cudaMemcpy(got.data(), dSig, n_det * 8, cudaMemcpyDeviceToHost));
    }
    CK(cudaEventRecord(b)); CK(cudaEventSynchronize(b));
    float ms_rt; cudaEventElapsedTime(&ms_rt, a, b); ms_rt /= reps;

    double gflop = 2.0 * ns_a * (double)n2 * nb * na / 1e9
                 + 2.0 * (double)na * nb * nb / 1e9
                 + 2.0 * (double)na * na * nb / 1e9;
    printf("\n--- GPU sigma, whole kernel, on the real (O,O,O) problem ---\n");
    printf("flops per sigma       %.3f GFLOP\n", gflop);
    printf("kernel only           %.3f ms  ->  %.1f sigma/s, %.1f GFLOP/s FP64\n",
           ms, 1e3 / ms, gflop / (ms / 1e3));
    printf("with host round trip  %.3f ms  ->  %.1f sigma/s\n", ms_rt, 1e3 / ms_rt);

    cublasDestroy(h);
    return 0;
}
