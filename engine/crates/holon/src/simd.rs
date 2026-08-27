//! The fused rowsum kernel — the n³ inner loop of stabilizer elimination,
//! in three variants: explicit AVX2 machine code (x86-64, runtime-detected),
//! WASM SIMD128 (browser tier, fixed-width, fully deterministic — the
//! relaxed-SIMD extension is deliberately NOT used: its lane semantics are
//! implementation-defined, and this engine's product is determinism), and a
//! portable scalar fallback. All three compute bit-identical results; the
//! conformance gates run through whichever the platform dispatches.
//!
//! The operation (Aaronson–Gottesman rowsum, credited): given source row
//! (xs, zs) and target row (xt, zt), accumulate the phase-function masks
//!   +1: (~xs&zs&xt&~zt) | (xs&zs&~xt&zt) | (xs&~zs&xt&zt)
//!   −1: (~xs&zs&xt&zt)  | (xs&zs&xt&~zt) | (xs&~zs&~xt&zt)
//! as popcounts, and fold xt ^= xs, zt ^= zs — one pass, no second read.

/// Fused rowsum over word slices: returns (plus, minus) popcounts and
/// updates the target in place. Dispatches to the widest available kernel.
#[inline]
pub fn fused_rowsum(xt: &mut [u64], zt: &mut [u64], xs: &[u64], zs: &[u64]) -> (u32, u32) {
    debug_assert!(xt.len() == zt.len() && xt.len() == xs.len() && xt.len() == zs.len());
    #[cfg(target_arch = "x86_64")]
    {
        if avx2_available() {
            // SAFETY: guarded by runtime CPUID detection.
            return unsafe { fused_rowsum_avx2(xt, zt, xs, zs) };
        }
    }
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        return fused_rowsum_wasm128(xt, zt, xs, zs);
    }
    #[allow(unreachable_code)]
    fused_rowsum_scalar(xt, zt, xs, zs)
}

/// Portable fallback: scalar POPCNT is one instruction per word on every
/// modern target; the loop shape is stride-1 so LLVM vectorizes the mask
/// algebra even here.
pub fn fused_rowsum_scalar(
    xt: &mut [u64],
    zt: &mut [u64],
    xs: &[u64],
    zs: &[u64],
) -> (u32, u32) {
    let (mut plus, mut minus) = (0u32, 0u32);
    for i in 0..xt.len() {
        let (x1, z1) = (xs[i], zs[i]);
        let (x2, z2) = (xt[i], zt[i]);
        let p = (!x1 & z1 & x2 & !z2) | (x1 & z1 & !x2 & z2) | (x1 & !z1 & x2 & z2);
        let m = (!x1 & z1 & x2 & z2) | (x1 & z1 & x2 & !z2) | (x1 & !z1 & !x2 & z2);
        plus += p.count_ones();
        minus += m.count_ones();
        xt[i] = x2 ^ x1;
        zt[i] = z2 ^ z1;
    }
    (plus, minus)
}

#[cfg(target_arch = "x86_64")]
fn avx2_available() -> bool {
    use std::sync::OnceLock;
    static AVX2: OnceLock<bool> = OnceLock::new();
    *AVX2.get_or_init(|| std::arch::is_x86_feature_detected!("avx2"))
}

/// AVX2: 4 words per vector op; popcount via the PSHUFB nibble table
/// (the classic Muła kernel) accumulated with SAD against zero.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fused_rowsum_avx2(
    xt: &mut [u64],
    zt: &mut [u64],
    xs: &[u64],
    zs: &[u64],
) -> (u32, u32) {
    use std::arch::x86_64::*;
    let n = xt.len();
    let chunks = n / 4;
    let mut accp = _mm256_setzero_si256();
    let mut accm = _mm256_setzero_si256();
    let lut = _mm256_setr_epi8(
        0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2,
        3, 3, 4,
    );
    let low = _mm256_set1_epi8(0x0f);
    let zero = _mm256_setzero_si256();
    #[inline(always)]
    unsafe fn popacc(v: __m256i, lut: __m256i, low: __m256i, zero: __m256i, acc: &mut __m256i) {
        let lo = _mm256_and_si256(v, low);
        let hi = _mm256_and_si256(_mm256_srli_epi32(v, 4), low);
        let cnt = _mm256_add_epi8(_mm256_shuffle_epi8(lut, lo), _mm256_shuffle_epi8(lut, hi));
        *acc = _mm256_add_epi64(*acc, _mm256_sad_epu8(cnt, zero));
    }
    for c in 0..chunks {
        let i = c * 4;
        let x1 = _mm256_loadu_si256(xs.as_ptr().add(i) as *const __m256i);
        let z1 = _mm256_loadu_si256(zs.as_ptr().add(i) as *const __m256i);
        let x2 = _mm256_loadu_si256(xt.as_ptr().add(i) as *const __m256i);
        let z2 = _mm256_loadu_si256(zt.as_ptr().add(i) as *const __m256i);
        let nx1 = _mm256_andnot_si256(x1, _mm256_set1_epi64x(-1));
        let nz1 = _mm256_andnot_si256(z1, _mm256_set1_epi64x(-1));
        let nx2 = _mm256_andnot_si256(x2, _mm256_set1_epi64x(-1));
        let nz2 = _mm256_andnot_si256(z2, _mm256_set1_epi64x(-1));
        let p = _mm256_or_si256(
            _mm256_or_si256(
                _mm256_and_si256(_mm256_and_si256(nx1, z1), _mm256_and_si256(x2, nz2)),
                _mm256_and_si256(_mm256_and_si256(x1, z1), _mm256_and_si256(nx2, z2)),
            ),
            _mm256_and_si256(_mm256_and_si256(x1, nz1), _mm256_and_si256(x2, z2)),
        );
        let m = _mm256_or_si256(
            _mm256_or_si256(
                _mm256_and_si256(_mm256_and_si256(nx1, z1), _mm256_and_si256(x2, z2)),
                _mm256_and_si256(_mm256_and_si256(x1, z1), _mm256_and_si256(x2, nz2)),
            ),
            _mm256_and_si256(_mm256_and_si256(x1, nz1), _mm256_and_si256(nx2, z2)),
        );
        popacc(p, lut, low, zero, &mut accp);
        popacc(m, lut, low, zero, &mut accm);
        _mm256_storeu_si256(
            xt.as_mut_ptr().add(i) as *mut __m256i,
            _mm256_xor_si256(x2, x1),
        );
        _mm256_storeu_si256(
            zt.as_mut_ptr().add(i) as *mut __m256i,
            _mm256_xor_si256(z2, z1),
        );
    }
    let mut lanes_p = [0u64; 4];
    let mut lanes_m = [0u64; 4];
    _mm256_storeu_si256(lanes_p.as_mut_ptr() as *mut __m256i, accp);
    _mm256_storeu_si256(lanes_m.as_mut_ptr() as *mut __m256i, accm);
    let mut plus = (lanes_p[0] + lanes_p[1] + lanes_p[2] + lanes_p[3]) as u32;
    let mut minus = (lanes_m[0] + lanes_m[1] + lanes_m[2] + lanes_m[3]) as u32;
    // Tail words, scalar.
    let (tp, tm) = fused_rowsum_scalar(
        &mut xt[chunks * 4..],
        &mut zt[chunks * 4..],
        &xs[chunks * 4..],
        &zs[chunks * 4..],
    );
    plus += tp;
    minus += tm;
    (plus, minus)
}

/// WASM SIMD128: 2 words per vector op, popcount via i8x16.popcnt (part of
/// the standardized fixed-width SIMD — deterministic across engines).
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
fn fused_rowsum_wasm128(xt: &mut [u64], zt: &mut [u64], xs: &[u64], zs: &[u64]) -> (u32, u32) {
    use std::arch::wasm32::*;
    let n = xt.len();
    let chunks = n / 2;
    let (mut plus, mut minus) = (0u32, 0u32);
    for c in 0..chunks {
        let i = c * 2;
        unsafe {
            let x1 = v128_load(xs.as_ptr().add(i) as *const v128);
            let z1 = v128_load(zs.as_ptr().add(i) as *const v128);
            let x2 = v128_load(xt.as_ptr().add(i) as *const v128);
            let z2 = v128_load(zt.as_ptr().add(i) as *const v128);
            let p = v128_or(
                v128_or(
                    v128_and(v128_andnot(z1, x1), v128_and(x2, v128_not(z2))),
                    v128_and(v128_and(x1, z1), v128_andnot(z2, x2)),
                ),
                v128_and(v128_andnot(x1, z1), v128_and(x2, z2)),
            );
            let m = v128_or(
                v128_or(
                    v128_and(v128_andnot(z1, x1), v128_and(x2, z2)),
                    v128_and(v128_and(x1, z1), v128_and(x2, v128_not(z2))),
                ),
                v128_and(v128_andnot(x1, z1), v128_andnot(z2, x2)),
            );
            // NOTE on operand order: wasm v128_andnot(a, b) = a & !b.
            let pc = i8x16_popcnt(p);
            let mc = i8x16_popcnt(m);
            let ps = u16x8_extadd_pairwise_u8x16(pc);
            let ms = u16x8_extadd_pairwise_u8x16(mc);
            let ps = u32x4_extadd_pairwise_u16x8(ps);
            let ms = u32x4_extadd_pairwise_u16x8(ms);
            plus += (u32x4_extract_lane::<0>(ps)
                + u32x4_extract_lane::<1>(ps)
                + u32x4_extract_lane::<2>(ps)
                + u32x4_extract_lane::<3>(ps)) as u32;
            minus += (u32x4_extract_lane::<0>(ms)
                + u32x4_extract_lane::<1>(ms)
                + u32x4_extract_lane::<2>(ms)
                + u32x4_extract_lane::<3>(ms)) as u32;
            v128_store(xt.as_mut_ptr().add(i) as *mut v128, v128_xor(x2, x1));
            v128_store(zt.as_mut_ptr().add(i) as *mut v128, v128_xor(z2, z1));
        }
    }
    let (tp, tm) = fused_rowsum_scalar(
        &mut xt[chunks * 2..],
        &mut zt[chunks * 2..],
        &xs[chunks * 2..],
        &zs[chunks * 2..],
    );
    (plus + tp, minus + tm)
}

#[cfg(test)]
mod kernel_conformance {
    use super::*;

    /// The dispatched kernel must agree with the scalar reference on random
    /// data, bit for bit — including the phase counts.
    #[test]
    fn dispatched_matches_scalar() {
        let mut seed = 0x1234_5678_9abc_def0u64;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            seed
        };
        for len in [1usize, 3, 4, 7, 8, 33, 64, 129] {
            let xs: Vec<u64> = (0..len).map(|_| next()).collect();
            let zs: Vec<u64> = (0..len).map(|_| next()).collect();
            let xt0: Vec<u64> = (0..len).map(|_| next()).collect();
            let zt0: Vec<u64> = (0..len).map(|_| next()).collect();
            let (mut xa, mut za) = (xt0.clone(), zt0.clone());
            let (mut xb, mut zb) = (xt0.clone(), zt0.clone());
            let a = fused_rowsum(&mut xa, &mut za, &xs, &zs);
            let b = fused_rowsum_scalar(&mut xb, &mut zb, &xs, &zs);
            assert_eq!(a, b, "len={len}: phase counts differ");
            assert_eq!(xa, xb, "len={len}: X words differ");
            assert_eq!(za, zb, "len={len}: Z words differ");
        }
    }
}
