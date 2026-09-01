// The FCI sigma kernel: `sigma = H c` at the (O,O,O) scale, on the device, with a FIXED
// reduction order.
//
// This is the production form of SATURATION-3 G2's measured kernel
// (`conformance/atomworld/s3_mesh/gpu/sigma.cu`, 65.7 sigma/s against the 32-thread CPU
// mesh's 20.8). Only the two custom kernels live here; the three GEMMs are cuBLAS calls
// made from the host, so that the tuned library does the work it is good at and the host
// keeps the driver logic.
//
// THE CITATION ABOVE WAS WRONG WHEN THIS FILE WAS COMMITTED and is corrected here rather
// than quietly: it read `scratchpad/s3gpu/sigma.cu`, a relative path that reads as the
// repository's scratch directory and named a location the instruments had never been in
// — they lived in a per-session scratchpad and were one cleanup from taking the result's
// reproducibility with them. That is M-STALE-INSTRUMENT's second variant, registered the
// same day by saturation3-mesh, who rescued the files to the path above and re-ran them
// (correctness figures identical to the digit). It is invisible to both automated axes:
// the string is prose-shaped and carries no session key to grep, so it reads as a clean
// typo rather than as a lost instrument. The enforcement is human and it is the rule that
// caught this line: FOLLOW EVERY SCRATCH CITATION FROM OUTSIDE THE SESSION THAT WROTE IT.
//
// ============================ THE ALGEBRA, AND WHY IT IS THREE GEMMS ======================
//
// `sigma_direct` has three blocks. Written out, two of them are dense GEMMs against
// matrices that do not depend on c at all, so they are built ONCE per Hamiltonian:
//
//   beta same-spin :  Sigma  = C * F_b         F_b[jb][ib], (nb x nb)
//   alpha same-spin:  Sigma += F_a^T * C       F_a[ja][ia], (na x na)
//   mixed          :  Sigma[ia][ib] += sum_m sa[ia][m] * D[ja][m][ib]
//                     with D[ja] = A[ja] * T[ja], A (ns_a x n2) also c-independent
//
// Only T depends on c, and T[ja][kl][ib] = sign * c[ja][jb(kl,ib)] is a pure GATHER: for a
// fixed excitation kl and destination ib the source string jb is UNIQUE, because
// a+_p a_q |jb> = s|ib> inverts to |jb> = a+_q a_p |ib>. There is no summation to order.
//
// ============================ WHY THERE IS NOT ONE ATOMIC IN HERE =========================
//
// The determinism gate is the ADOPTION CONDITION, not a nicety: RESOURCE_DESIGN D0 makes
// the device class part of the artifact, and an artifact that is not even reproducible
// against ITSELF has no class to declare. The obvious implementation scatters — build T by
// scattering c through the beta singles, accumulate sigma by scattering D through the alpha
// singles — and both scatters collide, so both would need atomicAdd. Floating-point
// atomicAdd accumulates in COMPLETION ORDER, which is not reproducible, and a table built
// that way would not be bit-identical to itself across two runs on one machine, let alone
// across shard counts.
//
// Both scatters are therefore inverted into GATHERS, which is possible for exactly the
// reason above: the excitation maps are invertible. Every sum in this file is over a fixed
// index range in a fixed order. The host pins cuBLAS to pedantic math and gives it a fixed
// workspace so the library's kernel selection cannot vary with what happens to be free.
// Bit-identity is then a property of the construction — and it is MEASURED by
// `holon_chem::sigma_op::bit_identity_over_runs` on the operator that will actually run,
// rather than assumed from this comment.

extern "C" {

// T[ja][kl][ib] = sign(kl,ib) * c[ja][jb(kl,ib)], a pure gather. Writes are coalesced in
// ib; the read of c is a gather inside one row of nb, which stays in cache.
//
// `src_jb` is -1 where no source string maps to (kl, ib) — those entries are ZEROED rather
// than left, because T is reused across applications and a stale entry from the previous
// vector would be a silent wrong answer that agrees with itself run to run, which is the
// worst kind: it would pass the determinism gate.
__global__ void holon_fci_build_T(double *T, const double *c, const int *src_jb,
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
// that nothing collides and the summation order is fixed by t.
__global__ void holon_fci_gather_mixed(double *sigma, const double *D, const int *at_ja,
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

} // extern "C"
