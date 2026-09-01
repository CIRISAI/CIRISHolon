// SATURATION-3 G2, the device-capability probe.
//
// Before writing a sigma kernel it is worth knowing what the device can do in the
// precision the tables actually need. The referee gate is 1e-12 and the tables are f64,
// so FP32 throughput is irrelevant here however large it is: on consumer Ada, FP64 runs
// at 1/64 the FP32 rate, and if that ceiling is below the CPU arm the kernel cannot win
// no matter how well it is written.
//
// Three numbers, each one a thing the sigma kernel would be bounded by:
//   1. FP64 FMA throughput  — the arithmetic ceiling
//   2. FP64 DAXPY bandwidth — the memory ceiling (the sigma inner loop is a streaming
//                             multiply-add over contiguous doubles, so this is the
//                             realistic bound, not the FMA one)
//   3. PCIe round trip for one CI vector (1.58 MiB) — what each matvec would pay if the
//                             vector crossed the bus per call

#include <cstdio>
#include <cuda_runtime.h>

#define CK(x) do { cudaError_t e = (x); if (e != cudaSuccess) { \
    printf("CUDA error %s at line %d\n", cudaGetErrorString(e), __LINE__); return 1; } } while(0)

// Arithmetic-bound: a long dependent-free FMA chain in registers.
__global__ void fma64(double *out, double a, int iters) {
    double x0 = threadIdx.x * 1e-6, x1 = x0 + 1.0, x2 = x0 + 2.0, x3 = x0 + 3.0;
    for (int i = 0; i < iters; ++i) {
        x0 = fma(x0, a, 1.0); x1 = fma(x1, a, 1.0);
        x2 = fma(x2, a, 1.0); x3 = fma(x3, a, 1.0);
    }
    if (threadIdx.x == 1 << 30) out[0] = x0 + x1 + x2 + x3; // never taken; keeps it live
}

// Memory-bound: y += a*x over contiguous doubles — the shape of sigma's inner loop.
__global__ void daxpy(double *y, const double *x, double a, long n) {
    long i = blockIdx.x * (long)blockDim.x + threadIdx.x;
    long stride = (long)gridDim.x * blockDim.x;
    for (; i < n; i += stride) y[i] = fma(a, x[i], y[i]);
}

int main() {
    cudaDeviceProp p;
    CK(cudaGetDeviceProperties(&p, 0));
    printf("device            %s  (sm_%d%d, %d SMs, %.1f GB)\n",
           p.name, p.major, p.minor, p.multiProcessorCount,
           p.totalGlobalMem / 1e9);
    printf("mem clock         %.2f GHz, bus %d bit -> peak BW %.1f GB/s\n",
           p.memoryClockRate / 1e6, p.memoryBusWidth,
           2.0 * p.memoryClockRate * 1e3 * (p.memoryBusWidth / 8) / 1e9);

    // ---- 1. FP64 FMA throughput
    {
        int blocks = p.multiProcessorCount * 32, threads = 256, iters = 20000;
        double *d; CK(cudaMalloc(&d, 8));
        fma64<<<blocks, threads>>>(d, 0.9999999, 100); CK(cudaDeviceSynchronize());
        cudaEvent_t a, b; cudaEventCreate(&a); cudaEventCreate(&b);
        CK(cudaEventRecord(a));
        fma64<<<blocks, threads>>>(d, 0.9999999, iters);
        CK(cudaEventRecord(b)); CK(cudaEventSynchronize(b));
        float ms; cudaEventElapsedTime(&ms, a, b);
        // 4 FMAs per iteration per thread, 2 flops per FMA
        double flops = 2.0 * 4.0 * (double)iters * blocks * threads;
        printf("FP64 FMA          %.1f GFLOP/s  (%.3f ms)\n", flops / (ms * 1e6), ms);
        cudaFree(d);
    }

    // ---- 2. FP64 DAXPY (streaming)
    {
        long n = 64L * 1024 * 1024; // 512 MiB per array
        double *x, *y;
        CK(cudaMalloc(&x, n * 8)); CK(cudaMalloc(&y, n * 8));
        CK(cudaMemset(x, 0, n * 8)); CK(cudaMemset(y, 0, n * 8));
        daxpy<<<1024, 256>>>(y, x, 1.5, n); CK(cudaDeviceSynchronize());
        cudaEvent_t a, b; cudaEventCreate(&a); cudaEventCreate(&b);
        CK(cudaEventRecord(a));
        for (int r = 0; r < 5; ++r) daxpy<<<1024, 256>>>(y, x, 1.5, n);
        CK(cudaEventRecord(b)); CK(cudaEventSynchronize(b));
        float ms; cudaEventElapsedTime(&ms, a, b); ms /= 5.0f;
        double bytes = 3.0 * n * 8.0; // read x, read y, write y
        printf("FP64 DAXPY        %.1f GB/s, %.1f GFLOP/s  (%.3f ms)\n",
               bytes / (ms * 1e6), 2.0 * n / (ms * 1e6), ms);
        cudaFree(x); cudaFree(y);
    }

    // ---- 3. PCIe round trip for one CI vector
    {
        size_t bytes = 207025 * 8; // the (O,O,O) CI vector
        double *h, *d;
        CK(cudaMallocHost(&h, bytes)); CK(cudaMalloc(&d, bytes));
        for (int r = 0; r < 3; ++r) {
            CK(cudaMemcpy(d, h, bytes, cudaMemcpyHostToDevice));
            CK(cudaMemcpy(h, d, bytes, cudaMemcpyDeviceToHost));
        }
        cudaEvent_t a, b; cudaEventCreate(&a); cudaEventCreate(&b);
        CK(cudaEventRecord(a));
        for (int r = 0; r < 50; ++r) {
            CK(cudaMemcpy(d, h, bytes, cudaMemcpyHostToDevice));
            CK(cudaMemcpy(h, d, bytes, cudaMemcpyDeviceToHost));
        }
        CK(cudaEventRecord(b)); CK(cudaEventSynchronize(b));
        float ms; cudaEventElapsedTime(&ms, a, b); ms /= 50.0f;
        printf("PCIe H2D+D2H      %.4f ms per 1.58 MiB round trip (%.1f GB/s effective)\n",
               ms, 2.0 * bytes / (ms * 1e6));
        cudaFreeHost(h); cudaFree(d);
    }

    // ---- 4. The pinned-vs-pageable question, because the table path's vectors live in
    //         ordinary Vec<f64> and would be pageable unless deliberately staged.
    {
        size_t bytes = 207025 * 8;
        double *h = (double *)malloc(bytes); double *d;
        CK(cudaMalloc(&d, bytes));
        memset(h, 0, bytes);
        for (int r = 0; r < 3; ++r) CK(cudaMemcpy(d, h, bytes, cudaMemcpyHostToDevice));
        cudaEvent_t a, b; cudaEventCreate(&a); cudaEventCreate(&b);
        CK(cudaEventRecord(a));
        for (int r = 0; r < 50; ++r) CK(cudaMemcpy(d, h, bytes, cudaMemcpyHostToDevice));
        CK(cudaEventRecord(b)); CK(cudaEventSynchronize(b));
        float ms; cudaEventElapsedTime(&ms, a, b); ms /= 50.0f;
        printf("PCIe pageable H2D %.4f ms per 1.58 MiB (%.1f GB/s)\n",
               ms, bytes / (ms * 1e6));
        free(h); cudaFree(d);
    }
    return 0;
}
