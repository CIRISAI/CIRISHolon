//! Synthetic branch batches, for the sizes a real circuit will not reach in a
//! benchmark's patience.
//!
//! The generator is honest about what it is: it builds descriptors that satisfy
//! the `Affine` invariant BY CONSTRUCTION (the k pivot rows carry the identity,
//! so `R` has full column rank without a rank check) rather than states any
//! circuit produced. Everything downstream of the descriptor — the F2 solve, the
//! phase polynomial, the rotation, the reduction — is the same code the real
//! branches go through, which is why a synthetic batch is a fair timing
//! workload and NOT evidence about the physics.
//!
//! The weights are drawn with at least one odd coefficient at a fixed `m`, so
//! the batch is exponent-uniform. That is deliberate and is the case the module
//! header of `gpu` singles out: it is where struct equality with the sequential
//! CPU fold is guaranteed rather than observed, so a determinism failure there
//! is unambiguous.

use holon::ledger::Cyc;

use crate::desc::AffineDesc;

/// xorshift64*, so the crate keeps `holon`'s zero-runtime-dependency posture.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// `count` descriptors on `n` qubits with affine dimension `k`.
///
/// Panics if `k > n` (a full-column-rank `n x k` matrix needs at least k rows)
/// or if either exceeds the device caps.
pub fn batch(n: usize, k: usize, count: usize, seed: u64) -> Vec<AffineDesc> {
    assert!(k <= n, "k = {k} > n = {n}: R cannot have full column rank");
    assert!(n <= crate::desc::N_CAP && k <= crate::desc::K_CAP);
    let mut rng = Rng::new(seed);
    let kmask = if k == 64 { !0u64 } else { (1u64 << k) - 1 };
    let nmask = if n == 64 { !0u64 } else { (1u64 << n) - 1 };

    (0..count)
        .map(|_| {
            // Pivot rows: a random k-subset, carrying the identity. This is what
            // makes full column rank a construction and not a rejection loop.
            let mut rows: Vec<usize> = (0..n).collect();
            for i in (1..n).rev() {
                rows.swap(i, rng.below(i + 1));
            }
            let mut r_rows = vec![0u64; n];
            for row in r_rows.iter_mut() {
                *row = rng.next_u64() & kmask;
            }
            for (c, &p) in rows.iter().take(k).enumerate() {
                r_rows[p] = 1u64 << c;
            }

            let mut j_rows = vec![0u64; k];
            for (a, j) in j_rows.iter_mut().enumerate() {
                let above = if a >= 63 { 0 } else { !0u64 << (a + 1) };
                *j = rng.next_u64() & kmask & above;
            }

            let mut d = [0u64; 2];
            for a in 0..k {
                d[a >> 5] |= (rng.next_u64() & 3) << ((a & 31) * 2);
            }

            // Weight: small coefficients, at least one odd, fixed exponent — so
            // `normalize` leaves `m` alone and the batch stays exponent-uniform.
            let mut c = [0i128; 4];
            for v in c.iter_mut() {
                *v = (rng.next_u64() % 7) as i128 - 3;
            }
            if c.iter().all(|v| v % 2 == 0) {
                c[0] += 1;
            }

            AffineDesc {
                n,
                k,
                zero: false,
                r_rows,
                h: rng.next_u64() & nmask,
                d,
                j_rows,
                base: crate::ring::normalize(Cyc { c, m: 4 }),
            }
        })
        .collect()
}
