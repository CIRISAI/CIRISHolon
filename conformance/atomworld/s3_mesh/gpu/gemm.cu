// SATURATION-3 G2: the OPTIMISTIC CEILING for a GPU sigma at the (O,O,O) scale.
//
// The mixed alpha-beta block of `sigma_direct` dominates the kernel. For each alpha
// string ja it is, exactly, a small dense GEMM:
//
//     vrow[48 x 455] = G_sub[48 x 225] * t[225 x 455]
//
// where 48 = single excitations per alpha string (12 occupied x 3 virtual + 12
// diagonal), 225 = n_orb^2, 455 = beta strings. Batched over the 455 alpha strings that
// is 2 * 48 * 225 * 455 * 455 = 4.47 GFLOP per sigma call.
//
// This probe runs that batch through cuBLAS. The point is that cuBLAS is an UPPER BOUND
// on any sigma kernel I would write by hand: it does not pay for the t-build scatter,
// the final scatter, the same-spin blocks, or any host transfer, and it is better tuned
// than anything hand-rolled here. If the upper bound loses to the CPU mesh, the refusal
// is settled and no amount of kernel engineering reopens it. If it wins, the margin it
// wins by is the budget a real kernel has to fit inside.
//
// The CPU arm it is being compared against is MEASURED, not extrapolated:
// examples/s3_sigma_cost.rs reports 17.2 sigma/s aggregate over 32 threads on this
// machine, which is 81 GFLOP/s. That is the number to beat.

#include <cstdio>
#include <cstdlib>
#include <cuda_runtime.h>
#include <cublas_v2.h>

#define CK(x) do { cudaError_t e = (x); if (e != cudaSuccess) { \
    printf("CUDA error %s at line %d\n", cudaGetErrorString(e), __LINE__); exit(1); } } while(0)
#define CB(x) do { cublasStatus_t s = (x); if (s != CUBLAS_STATUS_SUCCESS) { \
    printf("cuBLAS error %d at line %d\n", (int)s, __LINE__); exit(1); } } while(0)

int main(int argc, char **argv) {
    const int M = 48;    // single excitations per alpha string
    const int K = 225;   // n_orb^2
    const int N = 455;   // beta strings
    const int B = 455;   // alpha strings (the batch)

    // How many sigma calls to time. Each is one full batched GEMM.
    int reps = (argc > 1) ? atoi(argv[1]) : 20;

    double gflop_per_sigma = 2.0 * M * (double)K * N * B / 1e9;
    printf("shape             (%d x %d) * (%d x %d), batch %d\n", M, K, K, N, B);
    printf("flops per sigma   %.3f GFLOP\n", gflop_per_sigma);

    size_t szA = (size_t)M * K * B, szB = (size_t)K * N * B, szC = (size_t)M * N * B;
    printf("device operands   A %.1f MB, B %.1f MB, C %.1f MB (total %.1f MB)\n",
           szA * 8 / 1e6, szB * 8 / 1e6, szC * 8 / 1e6, (szA + szB + szC) * 8 / 1e6);

    double *dA, *dB, *dC;
    CK(cudaMalloc(&dA, szA * 8));
    CK(cudaMalloc(&dB, szB * 8));
    CK(cudaMalloc(&dC, szC * 8));

    // Fill with something nonzero: an all-zero operand would let nothing be skipped in
    // a dense GEMM, but it also makes a wrong result invisible, so the checksum below
    // needs real values.
    {
        double *h = (double *)malloc(szB * 8);
        for (size_t i = 0; i < szB; ++i) h[i] = 1e-3 * (double)((i * 2654435761u) % 1000) - 0.5;
        CK(cudaMemcpy(dB, h, szB * 8, cudaMemcpyHostToDevice));
        for (size_t i = 0; i < szA; ++i) h[i] = 1e-3 * (double)((i * 40503u) % 997) - 0.5;
        CK(cudaMemcpy(dA, h, szA * 8, cudaMemcpyHostToDevice));
        free(h);
    }

    cublasHandle_t hb; CB(cublasCreate(&hb));
    const double alpha = 1.0, beta = 0.0;

    // Column-major cuBLAS computing C = A * B with A m x k, B k x n, C m x n.
    auto run = [&]() {
        CB(cublasDgemmStridedBatched(
            hb, CUBLAS_OP_N, CUBLAS_OP_N,
            M, N, K,
            &alpha,
            dA, M, (long long)M * K,
            dB, K, (long long)K * N,
            &beta,
            dC, M, (long long)M * N,
            B));
    };

    run(); CK(cudaDeviceSynchronize());

    // M-VACUOUS-SUCCESS: prove the GEMM produced something before timing it.
    {
        double *h = (double *)malloc(szC * 8);
        CK(cudaMemcpy(h, dC, szC * 8, cudaMemcpyDeviceToHost));
        double sum = 0.0; size_t nz = 0;
        for (size_t i = 0; i < szC; ++i) { sum += h[i]; if (h[i] != 0.0) nz++; }
        printf("result check      %zu / %zu nonzero, checksum %.6e\n", nz, szC, sum);
        if (nz == 0) { printf("REFUSED: the GEMM produced nothing to time\n"); return 1; }
        free(h);
    }

    cudaEvent_t a, b; cudaEventCreate(&a); cudaEventCreate(&b);
    CK(cudaEventRecord(a));
    for (int r = 0; r < reps; ++r) run();
    CK(cudaEventRecord(b)); CK(cudaEventSynchronize(b));
    float ms; cudaEventElapsedTime(&ms, a, b); ms /= reps;

    double gflops = gflop_per_sigma / (ms / 1e3);
    printf("\n--- GPU ceiling (cuBLAS batched DGEMM, no scatter, no transfer) ---\n");
    printf("per sigma         %.3f ms\n", ms);
    printf("throughput        %.1f GFLOP/s FP64\n", gflops);
    printf("sigma/s           %.1f\n", 1e3 / ms);
    printf("vs CPU mesh       %.2fx  (CPU arm: 17.2 sigma/s aggregate, 32 threads)\n",
           (1e3 / ms) / 17.2);

    // The same batch as ONE big GEMM, for reference: it is the shape cuBLAS is happiest
    // with, and the gap between the two is the price of the batching itself.
    CK(cudaEventRecord(a));
    for (int r = 0; r < reps; ++r) {
        CB(cublasDgemm(hb, CUBLAS_OP_N, CUBLAS_OP_N,
                       M, N * B, K, &alpha, dA, M, dB, K, &beta, dC, M));
    }
    CK(cudaEventRecord(b)); CK(cudaEventSynchronize(b));
    cudaEventElapsedTime(&ms, a, b); ms /= reps;
    printf("\n(one large GEMM of the same volume: %.3f ms, %.1f GFLOP/s -- the batching \n"
           " overhead is the difference)\n", ms, gflop_per_sigma / (ms / 1e3));

    cublasDestroy(hb);
    cudaFree(dA); cudaFree(dB); cudaFree(dC);
    return 0;
}
