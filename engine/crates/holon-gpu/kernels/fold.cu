// The branch fold on the device.
//
// ONE KERNEL SHAPE, and it is an instance of `holon::merge`'s one law: an
// associative-commutative fold of an exact ledger. The ledger is
// `holon::ledger::Cyc` — integer Z[omega] coefficients at a COMMON exponent
// (the host aligns the batch before upload), so every accumulation in here is
// an integer add. Integer add in two's complement is associative and
// commutative WITHOUT QUALIFICATION — including on wraparound — so the value
// this kernel returns does not depend on which thread saw which branch, on the
// warp-shuffle tree's shape, or on the block/grid configuration. That is the
// determinism argument, and it is not a floating-point tolerance story.
//
// PER-BRANCH MATH (the port of `holon::prune::Affine::amplitude`):
//   y in F2^n is the basis state; the branch's affine state is x = R u + h with
//   R full column rank (n x k). Solve R u = y + h over F2 by Gauss-Jordan on the
//   augmented rows; if inconsistent the branch contributes exactly zero.
//   Then  ip   = sum_{a : u_a} d[a]           (mod 4)
//         sign = parity of #{a<b : u_a u_b J[a][b]}
//   and the amplitude is  weight * gamma * i^ip * (-1)^sign.
//
// The host folds `weight * gamma` into a single ring element BASE_b, because
// multiplying a Cyc by i^ip and by -1 is exactly a ROTATION of its four
// coefficients (omega^4 = -1, omega^8 = 1), and rotation commutes with the
// ring's `normalize`. So the device never multiplies in the ring at all: it
// computes r = (2*ip + 4*sign) mod 8 and rotates. This is not an approximation
// of `amplitude_of` — it is equal to it as a STRUCT, per branch, and
// `rotation_codes` exists so the test suite can check that branch by branch
// rather than only checking the sum (a sum can agree while two errors cancel).
//
// LAYOUT: every per-branch array is strided by the batch size B (element i of
// branch b at index i*B + b) so that a warp's loads are contiguous.
//
// CAPS: n <= 64 rows and k <= 63 columns (bit 63 of each augmented row carries
// the right-hand side). The host refuses batches that exceed either rather than
// truncating. NCAP is a compile-time specialization of the augmented-row scratch
// array, which lives in LOCAL memory: n <= 32 gets a 256-byte-per-thread kernel
// and everything else a 512-byte one, because that array is the occupancy
// limiter and halving it is worth two entry points.

typedef unsigned long long u64;

#define R_ZERO      8u   // branch contributes exactly zero (off-coset or dead)
#define R_RANK_BAD  9u   // R had dependent columns: the Affine invariant broke

// ---------------------------------------------------------------- i128 shuffle
//
// __shfl_down_sync has no 128-bit overload; move it as four 32-bit words. The
// value is reassembled bit-exactly, so this is data movement and not arithmetic.
__device__ __forceinline__ __int128 shfl_down_i128(__int128 v, int delta) {
    unsigned __int128 uv = (unsigned __int128)v;
    unsigned int w0 = (unsigned int)(uv);
    unsigned int w1 = (unsigned int)(uv >> 32);
    unsigned int w2 = (unsigned int)(uv >> 64);
    unsigned int w3 = (unsigned int)(uv >> 96);
    w0 = __shfl_down_sync(0xffffffffu, w0, delta);
    w1 = __shfl_down_sync(0xffffffffu, w1, delta);
    w2 = __shfl_down_sync(0xffffffffu, w2, delta);
    w3 = __shfl_down_sync(0xffffffffu, w3, delta);
    uv = ((unsigned __int128)w3 << 96) | ((unsigned __int128)w2 << 64)
       | ((unsigned __int128)w1 << 32) | (unsigned __int128)w0;
    return (__int128)uv;
}

// ------------------------------------------------------------ the per-branch solve
//
// Returns the rotation code in 0..7, or R_ZERO / R_RANK_BAD. This is the whole
// port; both entry points below are thin shells around it.
template <int NCAP>
__device__ __forceinline__ unsigned int branch_code(
    const u64* __restrict__ rrow,
    const u64* __restrict__ hbits,
    const u64* __restrict__ jrow,
    const u64* __restrict__ dpack,
    const unsigned int* __restrict__ kk,
    u64 ybits, unsigned int n, unsigned int B, unsigned int b)
{
    unsigned int k = kk[b];
    if (k == 0xFFFFFFFFu) return R_ZERO;          // state flagged zero on the host

    // --- build the augmented rows: R row in bits 0..k-1, rhs in bit 63.
    u64 rhs = ybits ^ hbits[b];
    u64 aug[NCAP];
    for (unsigned int row = 0; row < n; ++row) {
        aug[row] = rrow[row * B + b] | (((rhs >> row) & 1ull) << 63);
    }

    // --- Gauss-Jordan. R has full column rank by the Affine invariant, so the
    //     pivot for column c lands on row c and rr == k at the end. The CPU
    //     asserts that; a kernel cannot, so it reports instead.
    unsigned int rr = 0;
    for (unsigned int c = 0; c < k; ++c) {
        unsigned int p = rr;
        while (p < n && !((aug[p] >> c) & 1ull)) ++p;
        if (p >= n) return R_RANK_BAD;
        u64 t = aug[p]; aug[p] = aug[rr]; aug[rr] = t;
        u64 piv = aug[rr];
        for (unsigned int p2 = 0; p2 < n; ++p2) {
            if (p2 != rr && ((aug[p2] >> c) & 1ull)) aug[p2] ^= piv;
        }
        ++rr;
    }

    // --- consistency: every row past the rank must carry a zero rhs, else y is
    //     off the coset and the branch contributes exactly zero.
    for (unsigned int row = rr; row < n; ++row) {
        if (aug[row] >> 63) return R_ZERO;
    }

    // --- read u off the pivots (the pivot of column c is row c).
    u64 u = 0;
    for (unsigned int c = 0; c < k; ++c) u |= ((aug[c] >> 63) & 1ull) << c;

    // --- the phase polynomial.
    u64 dlo = dpack[b];
    u64 dhi = dpack[B + b];
    unsigned int ip = 0, sgn = 0;
    u64 uu = u;
    while (uu) {
        int a = __ffsll((long long)uu) - 1;
        uu &= uu - 1;
        u64 dw = (a < 32) ? dlo : dhi;
        ip += (unsigned int)((dw >> ((a & 31) * 2)) & 3ull);
        u64 above = (a == 63) ? 0ull : (~0ull << (a + 1));
        sgn ^= (unsigned int)(__popcll(jrow[(unsigned int)a * B + b] & u & above) & 1);
    }
    return ((2u * (ip & 3u)) + 4u * sgn) & 7u;
}

// ---------------------------------------------------------------- the fold
//
// out: 4 lanes x 2 limbs per BLOCK, strided by gridDim.x:
//      out[(2*lane + limb) * gridDim.x + blockIdx.x]
// plus one more stride, out[8*gridDim.x + blockIdx.x], carrying that block's
// "a branch reported R_RANK_BAD" flag. Per-block and not a single sticky word
// on purpose: every slot is WRITTEN by every launch, so the host never has to
// zero the buffer between folds and the whole result comes back in one copy.
// A silent wrong answer is the thing that flag exists to stop.
// The host sums the block partials in block order. It could equally sum them in
// any other order; see the header.
template <int NCAP>
__device__ __forceinline__ void fold_body(
    const u64* __restrict__ rrow, const u64* __restrict__ hbits,
    const u64* __restrict__ jrow, const u64* __restrict__ dpack,
    const u64* __restrict__ base, const unsigned int* __restrict__ kk,
    u64* __restrict__ out,
    u64 ybits, unsigned int n, unsigned int B)
{
    unsigned int bad = 0;
    // Four SCALAR accumulators, never an array: a dynamically indexed local
    // array would be spilled to local memory by the compiler, and these are the
    // hottest values in the kernel.
    __int128 acc0 = 0, acc1 = 0, acc2 = 0, acc3 = 0;

    for (unsigned int b = blockIdx.x * blockDim.x + threadIdx.x;
         b < B; b += gridDim.x * blockDim.x)
    {
        unsigned int r = branch_code<NCAP>(rrow, hbits, jrow, dpack, kk, ybits, n, B, b);
        if (r == R_RANK_BAD) { bad = 1u; continue; }
        if (r == R_ZERO) continue;

        __int128 a0 = (__int128)(((unsigned __int128)base[1 * B + b] << 64)
                                | (unsigned __int128)base[0 * B + b]);
        __int128 a1 = (__int128)(((unsigned __int128)base[3 * B + b] << 64)
                                | (unsigned __int128)base[2 * B + b]);
        __int128 a2 = (__int128)(((unsigned __int128)base[5 * B + b] << 64)
                                | (unsigned __int128)base[4 * B + b]);
        __int128 a3 = (__int128)(((unsigned __int128)base[7 * B + b] << 64)
                                | (unsigned __int128)base[6 * B + b]);

        // Multiplication by omega is the single step (a0,a1,a2,a3) ->
        // (-a3,a0,a1,a2), because omega^4 = -1. Rotating r times is
        // multiplication by omega^r, which is i^ip * (-1)^sign. Register moves
        // only, at most seven of them.
        for (unsigned int t = 0; t < r; ++t) {
            __int128 tmp = -a3; a3 = a2; a2 = a1; a1 = a0; a0 = tmp;
        }
        acc0 += a0; acc1 += a1; acc2 += a2; acc3 += a3;
    }

    // --- warp reduce, then block reduce. 32 warps is the hardware maximum per
    //     block, so the shared array is sized, not dynamic. The host requires
    //     blockDim.x % 32 == 0 so the full shuffle mask is honest.
    #pragma unroll
    for (int d = 16; d > 0; d >>= 1) {
        acc0 += shfl_down_i128(acc0, d);
        acc1 += shfl_down_i128(acc1, d);
        acc2 += shfl_down_i128(acc2, d);
        acc3 += shfl_down_i128(acc3, d);
    }
    __shared__ __int128 s[4][32];
    __shared__ unsigned int s_bad;
    int lane = threadIdx.x & 31;
    int warp = threadIdx.x >> 5;
    int nwarps = (blockDim.x + 31) >> 5;
    if (threadIdx.x == 0) s_bad = 0;
    __syncthreads();
    if (bad) s_bad = 1u;
    if (lane == 0) {
        s[0][warp] = acc0; s[1][warp] = acc1; s[2][warp] = acc2; s[3][warp] = acc3;
    }
    __syncthreads();

    if (threadIdx.x == 0) {
        #pragma unroll
        for (int p = 0; p < 4; ++p) {
            __int128 t = 0;
            for (int w = 0; w < nwarps; ++w) t += s[p][w];
            unsigned __int128 uv = (unsigned __int128)t;
            out[(2 * p) * gridDim.x + blockIdx.x] = (u64)uv;
            out[(2 * p + 1) * gridDim.x + blockIdx.x] = (u64)(uv >> 64);
        }
        out[8 * gridDim.x + blockIdx.x] = (u64)s_bad;
    }
}

#define FOLD_ENTRY(NAME, NCAP)                                                 \
extern "C" __global__ void NAME(                                               \
    const u64* __restrict__ rrow, const u64* __restrict__ hbits,               \
    const u64* __restrict__ jrow, const u64* __restrict__ dpack,               \
    const u64* __restrict__ base, const unsigned int* __restrict__ kk,         \
    u64* __restrict__ out,                                                     \
    unsigned long long ybits, unsigned int n, unsigned int B)                  \
{ fold_body<NCAP>(rrow, hbits, jrow, dpack, base, kk, out, ybits, n, B); }

FOLD_ENTRY(fold_affine_n32, 32)
FOLD_ENTRY(fold_affine_n64, 64)

#define CODES_ENTRY(NAME, NCAP)                                                \
extern "C" __global__ void NAME(                                               \
    const u64* __restrict__ rrow, const u64* __restrict__ hbits,               \
    const u64* __restrict__ jrow, const u64* __restrict__ dpack,               \
    const unsigned int* __restrict__ kk, unsigned char* __restrict__ out_r,    \
    unsigned long long ybits, unsigned int n, unsigned int B)                  \
{                                                                              \
    for (unsigned int b = blockIdx.x * blockDim.x + threadIdx.x;               \
         b < B; b += gridDim.x * blockDim.x)                                   \
        out_r[b] = (unsigned char)branch_code<NCAP>(                           \
            rrow, hbits, jrow, dpack, kk, ybits, n, B, b);                     \
}

CODES_ENTRY(rotation_codes_n32, 32)
CODES_ENTRY(rotation_codes_n64, 64)
