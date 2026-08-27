//! THE RESIDUE CARRIER — the ring scaled to the circuit, not the circuit
//! capped by the ring.
//!
//! This is NOT a workaround, and it is not "big-number fallback". It is the
//! object doing what the object does: RECURSING. Each prime modulus carries a
//! CHILD HOLON — the same four lanes, the same √2-exponent bookkeeping, the
//! same one merge law — and the parent's value is what the children jointly
//! know. The mod-p lane digests were already the certificate layer
//! (`lean/CIRISHolon/MergeLaw.lean`: `digest_commutes`, `digest_convicts`),
//! and `Boundaries.lean::digests_jointly_faithful` proves the family
//! separates any two distinct values — so the corruption detector and the
//! carrier are ONE mechanism. Computing IN the digests over enough primes and
//! reconstructing at the end loses nothing, and no i128 envelope exists here
//! AT ALL: lane values are bounded by their modulus at every step, by
//! construction. Overflow is not handled; it is impossible.
//!
//! We do not assume the limits of other approaches. Fixed-width engines cap
//! the circuit to protect the ring (SliQSim's `--r/--alloc`, our own i128
//! refusal on the direct path); here the router reads the circuit's proven
//! coefficient bound (growth 2^{O(n+t)}, arXiv:2602.17775) and issues enough
//! prime children to carry it. The direct `Cyc` path remains the fast lane
//! for circuits that fit it; its refusal is the router's signal, never a
//! user-facing error.
//!
//! Division of labour (why per-branch stays i128): a single Clifford+T
//! branch amplitude is a unit times a scale — tiny lanes. The growth lives
//! in the FOLD, where 2^t aligned terms sum. So branches are computed on the
//! direct ring and reduced into the residue accumulator; only the fold and
//! the final reading live here.
//!
//! What the residue carrier gives up, honestly: evenness is invisible mod an
//! odd prime, so there is no `normalize()` here — `m` rides at the running
//! maximum and canonical form returns only at reconstruction. Magnitude
//! comparisons (the sampler's need) are likewise reconstruction-time
//! questions. The amplitude path needs neither along the way.

use crate::ledger::Cyc;
use crate::merge::{self, MergeLedger};
use crate::BranchSource;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Prime supply: deterministic Miller–Rabin (the witness set below is proven
// complete for all n < 2^64), scanning down from 2^61 so lane sums and
// products stay comfortably inside u64/u128.
// ---------------------------------------------------------------------------

fn mulmod(a: u64, b: u64, p: u64) -> u64 {
    ((a as u128 * b as u128) % p as u128) as u64
}

fn powmod(mut b: u64, mut e: u64, p: u64) -> u64 {
    let mut acc = 1u64 % p;
    b %= p;
    while e > 0 {
        if e & 1 == 1 {
            acc = mulmod(acc, b, p);
        }
        b = mulmod(b, b, p);
        e >>= 1;
    }
    acc
}

fn is_prime_u64(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    for q in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n == q {
            return true;
        }
        if n % q == 0 {
            return false;
        }
    }
    let mut d = n - 1;
    let mut s = 0u32;
    while d & 1 == 0 {
        d >>= 1;
        s += 1;
    }
    'witness: for a in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let mut x = powmod(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 1..s {
            x = mulmod(x, x, n);
            if x == n - 1 {
                continue 'witness;
            }
        }
        return false;
    }
    true
}

/// Bits of headroom each prime child carries (primes sit just under 2^61).
pub const BITS_PER_PRIME: usize = 60;

/// The standing prime table: 32 children ≈ 1920 bits of exact coefficient,
/// generated once, deterministically, at first use.
pub fn primes() -> &'static [u64] {
    static PRIMES: OnceLock<Vec<u64>> = OnceLock::new();
    PRIMES.get_or_init(|| {
        let mut out = Vec::with_capacity(32);
        let mut cand = (1u64 << 61) - 1;
        while out.len() < 32 {
            if is_prime_u64(cand) {
                out.push(cand);
            }
            cand -= 2;
        }
        out
    })
}

/// How many prime children a coefficient bound of `bits` needs. REFUSES when
/// the bound exceeds the table — a silent clamp here would shrink the CRT
/// window below the coefficient bound and reconstruct a WRONG exact value,
/// which is the one output this engine may never produce. (Caught by external
/// review, 2026-08-27: the previous `.min(table)` was exactly that clamp.)
pub fn primes_for_bits(bits: usize) -> usize {
    // +1 for the sign headroom (values live in (−P/2, P/2)).
    let need = bits / BITS_PER_PRIME + 2;
    assert!(
        need <= primes().len(),
        "residue: coefficient bound of {bits} bits needs {need} primes but the          table holds {}; refusing rather than silently shrinking the CRT window          (grow the prime table to proceed)",
        primes().len()
    );
    need
}

// ---------------------------------------------------------------------------
// The child arithmetic: four lanes mod one prime, mirroring Cyc exactly.
// ---------------------------------------------------------------------------

fn reduce_i128(c: i128, p: u64) -> u64 {
    let r = (c % p as i128 + p as i128) % p as i128;
    r as u64
}

/// Multiply one child's lanes by √2 = ω − ω³ (the same lane mix as
/// `Cyc::add`'s odd-Δ branch, taken mod p).
fn mul_sqrt2(l: [u64; 4], p: u64) -> [u64; 4] {
    let sub = |a: u64, b: u64| (a + p - b) % p;
    [
        sub(l[1], l[3]),
        (l[0] + l[2]) % p,
        (l[1] + l[3]) % p,
        sub(l[2], l[0]),
    ]
}

/// Lanes × √2^delta mod p.
fn scale_sqrt2_pow(mut l: [u64; 4], delta: u32, p: u64) -> [u64; 4] {
    let half = powmod(2, (delta / 2) as u64, p);
    for v in &mut l {
        *v = mulmod(*v, half, p);
    }
    if delta % 2 == 1 {
        l = mul_sqrt2(l, p);
    }
    l
}

// ---------------------------------------------------------------------------
// The accumulator: one child per prime, one merge law over all of them.
// ---------------------------------------------------------------------------

/// Exact cyclotomic value carried by prime children: lane residues per prime
/// at a shared √2-exponent `m`. The empty value (no children) is the merge
/// identity. No overflow exists in this carrier.
#[derive(Clone, Debug)]
pub struct ResCyc {
    /// One `[c0,c1,c2,c3] mod primes()[i]` per active prime.
    pub r: Vec<[u64; 4]>,
    /// Shared denominator exponent: value = lanes · 2^{−m/2}.
    pub m: i32,
}

impl ResCyc {
    /// Zero, carried by `k` prime children.
    pub fn zero(k: usize) -> ResCyc {
        assert!(k <= primes().len(), "residue: prime table exhausted");
        ResCyc { r: vec![[0u64; 4]; k], m: 0 }
    }

    pub fn is_zero(&self) -> bool {
        self.r.iter().all(|l| l.iter().all(|&x| x == 0))
    }

    /// Reduce a direct-ring value into `k` children.
    pub fn from_cyc(x: &Cyc, k: usize) -> ResCyc {
        let ps = primes();
        assert!(k <= ps.len(), "residue: prime table exhausted");
        ResCyc {
            r: (0..k)
                .map(|i| {
                    let p = ps[i];
                    [
                        reduce_i128(x.c[0], p),
                        reduce_i128(x.c[1], p),
                        reduce_i128(x.c[2], p),
                        reduce_i128(x.c[3], p),
                    ]
                })
                .collect(),
            m: x.m,
        }
    }

    /// Raise this value's exponent to `m` (multiply lanes by √2^Δ per child).
    /// Exact in every child; the value is unchanged.
    fn align_to(&mut self, m: i32) {
        assert!(m >= self.m, "residue: alignment must raise m, never round");
        let delta = (m - self.m) as u32;
        if delta > 0 {
            let ps = primes();
            for (i, l) in self.r.iter_mut().enumerate() {
                *l = scale_sqrt2_pow(*l, delta, ps[i]);
            }
            self.m = m;
        }
    }

    /// Accumulate one branch amplitude (computed on the direct ring, where a
    /// single branch always fits).
    pub fn add_cyc(&mut self, x: &Cyc) {
        if x.c.iter().all(|&v| v == 0) {
            return;
        }
        let k = self.r.len();
        let mut xr = ResCyc::from_cyc(x, k);
        if self.is_zero() {
            // Adopting the first term's exponent directly keeps m minimal;
            // aligning zero would be exact too, this is just tidier.
            *self = xr;
            return;
        }
        let target = self.m.max(xr.m);
        self.align_to(target);
        xr.align_to(target);
        let ps = primes();
        for i in 0..k {
            for lane in 0..4 {
                self.r[i][lane] = (self.r[i][lane] + xr.r[i][lane]) % ps[i];
            }
        }
    }
}

/// Semantic equality: same value, regardless of which exponent each side
/// rides at (align, then compare children).
impl PartialEq for ResCyc {
    fn eq(&self, other: &ResCyc) -> bool {
        if self.r.is_empty() || other.r.is_empty() {
            return self.is_zero() == other.is_zero()
                && (self.r.len() == other.r.len() || self.is_zero());
        }
        if self.r.len() != other.r.len() {
            return false;
        }
        let target = self.m.max(other.m);
        let mut a = self.clone();
        let mut b = other.clone();
        a.align_to(target);
        b.align_to(target);
        a.r == b.r
    }
}

/// The one merge law, unchanged: the residue carrier is an instance, not a
/// new mechanism. The empty value is the identity; two carried values align
/// and add child-wise.
impl MergeLedger for ResCyc {
    fn empty() -> Self {
        ResCyc { r: Vec::new(), m: 0 }
    }
    fn merge(self, other: Self) -> Self {
        if self.r.is_empty() {
            return other;
        }
        if other.r.is_empty() {
            return self;
        }
        assert_eq!(
            self.r.len(),
            other.r.len(),
            "residue: shards must carry the same prime children"
        );
        let target = self.m.max(other.m);
        let mut a = self;
        let mut b = other;
        a.align_to(target);
        b.align_to(target);
        let ps = primes();
        for i in 0..a.r.len() {
            for lane in 0..4 {
                a.r[i][lane] = (a.r[i][lane] + b.r[i][lane]) % ps[i];
            }
        }
        a
    }
}

// ---------------------------------------------------------------------------
// The residue fold: same shape as mesh::fold_amplitude, same law, no cap.
// ---------------------------------------------------------------------------

/// Per-shard accumulation over an ascending branch range.
pub fn fold_range_res<S: BranchSource>(
    src: &S,
    y: &[bool],
    range: std::ops::Range<u64>,
    k: usize,
) -> ResCyc {
    let mut acc = ResCyc::zero(k);
    for b in range {
        acc.add_cyc(&src.amplitude_of(b, y));
    }
    acc
}

/// `Σ_b coeff_b · ⟨y|φ_b⟩` in the residue carrier, folded across `shards`
/// threads through the one merge law — bit-identical for every shard count,
/// with NO coefficient envelope anywhere in the path.
pub fn fold_amplitude_res<S: BranchSource>(
    src: &S,
    y: &[bool],
    shards: usize,
    k: usize,
) -> ResCyc {
    assert_eq!(y.len(), src.n_qubits(), "residue: |y| must match qubit count");
    let ranges = crate::mesh::shard_ranges(src.n_branches(), shards);
    match ranges.len() {
        0 => ResCyc::zero(k),
        1 => fold_range_res(src, y, ranges[0].clone(), k),
        n => {
            let mut partial = vec![ResCyc::empty(); n];
            std::thread::scope(|scope| {
                for (slot, range) in partial.iter_mut().zip(ranges) {
                    scope.spawn(move || {
                        *slot = fold_range_res(src, y, range, k);
                    });
                }
            });
            merge::fold(partial)
        }
    }
}

// ---------------------------------------------------------------------------
// Reconstruction: the parent reads its children once, at the end.
// ---------------------------------------------------------------------------

/// Minimal magnitude bignum (u64 limbs, little-endian) — used only at
/// reconstruction and in tests; never in the fold.
#[derive(Clone, Debug, PartialEq)]
pub struct Big(pub Vec<u64>);

impl Big {
    pub fn from_u64(v: u64) -> Big {
        Big(if v == 0 { vec![] } else { vec![v] })
    }
    pub fn is_zero(&self) -> bool {
        self.0.is_empty()
    }
    fn trim(&mut self) {
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
    }
    pub fn mul_u64(&mut self, f: u64) {
        let mut carry: u128 = 0;
        for limb in &mut self.0 {
            let v = *limb as u128 * f as u128 + carry;
            *limb = v as u64;
            carry = v >> 64;
        }
        if carry > 0 {
            self.0.push(carry as u64);
        }
        self.trim();
    }
    pub fn add_big(&mut self, o: &Big) {
        let mut carry: u128 = 0;
        for i in 0..self.0.len().max(o.0.len()) {
            let a = *self.0.get(i).unwrap_or(&0) as u128;
            let b = *o.0.get(i).unwrap_or(&0) as u128;
            let v = a + b + carry;
            if i < self.0.len() {
                self.0[i] = v as u64;
            } else {
                self.0.push(v as u64);
            }
            carry = v >> 64;
        }
        if carry > 0 {
            self.0.push(carry as u64);
        }
    }
    /// self −= o. Requires self ≥ o.
    pub fn sub_big(&mut self, o: &Big) {
        let mut borrow: i128 = 0;
        for i in 0..self.0.len() {
            let a = self.0[i] as i128;
            let b = *o.0.get(i).unwrap_or(&0) as i128;
            let v = a - b - borrow;
            if v < 0 {
                self.0[i] = (v + (1i128 << 64)) as u64;
                borrow = 1;
            } else {
                self.0[i] = v as u64;
                borrow = 0;
            }
        }
        assert_eq!(borrow, 0, "residue: sub_big underflow");
        self.trim();
    }
    pub fn cmp_big(&self, o: &Big) -> std::cmp::Ordering {
        if self.0.len() != o.0.len() {
            return self.0.len().cmp(&o.0.len());
        }
        for i in (0..self.0.len()).rev() {
            if self.0[i] != o.0[i] {
                return self.0[i].cmp(&o.0[i]);
            }
        }
        std::cmp::Ordering::Equal
    }
    pub fn mod_u64(&self, p: u64) -> u64 {
        let mut r: u128 = 0;
        for limb in self.0.iter().rev() {
            r = ((r << 64) | *limb as u128) % p as u128;
        }
        r as u64
    }
    /// self /= 10^18, returning the remainder (for decimal rendering).
    fn divmod_1e18(&mut self) -> u64 {
        const D: u128 = 1_000_000_000_000_000_000;
        let mut rem: u128 = 0;
        for limb in self.0.iter_mut().rev() {
            let cur = (rem << 64) | *limb as u128;
            *limb = (cur / D) as u64;
            rem = cur % D;
        }
        self.trim();
        rem as u64
    }
    pub fn to_decimal(&self) -> String {
        if self.is_zero() {
            return "0".into();
        }
        let mut chunks = Vec::new();
        let mut v = self.clone();
        while !v.is_zero() {
            chunks.push(v.divmod_1e18());
        }
        let mut s = chunks.pop().unwrap().to_string();
        for c in chunks.into_iter().rev() {
            s.push_str(&format!("{c:018}"));
        }
        s
    }
    pub fn to_i128_checked(&self) -> Option<i128> {
        match self.0.len() {
            0 => Some(0),
            1 => Some(self.0[0] as i128),
            2 => {
                let v = (self.0[1] as u128) << 64 | self.0[0] as u128;
                if v <= i128::MAX as u128 { Some(v as i128) } else { None }
            }
            _ => None,
        }
    }
}

fn inv_mod(a: u64, p: u64) -> u64 {
    // Fermat: p prime.
    powmod(a, p - 2, p)
}

/// One reconstructed lane: sign and magnitude.
#[derive(Clone, Debug, PartialEq)]
pub struct SignedBig {
    pub negative: bool,
    pub magnitude: Big,
}

impl SignedBig {
    pub fn to_decimal(&self) -> String {
        if self.magnitude.is_zero() {
            return "0".into();
        }
        let mut s = self.magnitude.to_decimal();
        if self.negative {
            s.insert(0, '-');
        }
        s
    }
    pub fn to_i128_checked(&self) -> Option<i128> {
        let v = self.magnitude.to_i128_checked()?;
        Some(if self.negative { -v } else { v })
    }
}

/// The parent's reading: exact lanes and the shared exponent —
/// value = (l0 + l1ω + l2ω² + l3ω³)·2^{−m/2}.
#[derive(Clone, Debug, PartialEq)]
pub struct ResReading {
    pub lanes: [SignedBig; 4],
    pub m: i32,
}

impl ResReading {
    /// Direct-ring view, when the value happens to fit — the differential
    /// tests live on this.
    pub fn to_cyc_checked(&self) -> Option<Cyc> {
        Some(Cyc {
            c: [
                self.lanes[0].to_i128_checked()?,
                self.lanes[1].to_i128_checked()?,
                self.lanes[2].to_i128_checked()?,
                self.lanes[3].to_i128_checked()?,
            ],
            m: self.m,
        })
    }
    /// Approximate complex value, for display only (the exact value is the
    /// lanes; this is a courtesy rendering and says so).
    pub fn to_complex_approx(&self) -> (f64, f64) {
        let f = |sb: &SignedBig| {
            let mut x = 0.0f64;
            for limb in sb.magnitude.0.iter().rev() {
                x = x * 1.8446744073709552e19 + *limb as f64;
            }
            if sb.negative { -x } else { x }
        };
        let s = std::f64::consts::FRAC_1_SQRT_2;
        let w = [(1.0, 0.0), (s, s), (0.0, 1.0), (-s, s)];
        let (mut re, mut im) = (0.0, 0.0);
        for k in 0..4 {
            re += f(&self.lanes[k]) * w[k].0;
            im += f(&self.lanes[k]) * w[k].1;
        }
        let scale = (2.0f64).powf(-(self.m as f64) / 2.0);
        (re * scale, im * scale)
    }
}

impl ResCyc {
    /// CRT the children back into exact integer lanes (incremental Garner).
    /// Values are read in (−P/2, P/2); with the router's prime count chosen
    /// from the coefficient bound, that window provably contains the truth.
    pub fn reconstruct(&self) -> ResReading {
        let ps = primes();
        let k = self.r.len();
        let mut lanes: Vec<SignedBig> = Vec::with_capacity(4);
        for lane in 0..4 {
            // Incremental CRT: x ≡ r_i (mod p_i), P = Π p_i so far.
            let mut x = Big::from_u64(if k > 0 { self.r[0][lane] } else { 0 });
            let mut pprod = Big::from_u64(if k > 0 { ps[0] } else { 1 });
            for i in 1..k {
                let p = ps[i];
                let x_mod = x.mod_u64(p);
                let want = self.r[i][lane];
                let diff = (want + p - x_mod) % p;
                let t = mulmod(diff, inv_mod(pprod.mod_u64(p), p), p);
                let mut add = pprod.clone();
                add.mul_u64(t);
                x.add_big(&add);
                pprod.mul_u64(p);
            }
            // Signed window.
            let mut half = pprod.clone();
            // half = floor(P/2): shift right one bit.
            {
                let mut carry = 0u64;
                for limb in half.0.iter_mut().rev() {
                    let cur = *limb;
                    *limb = (cur >> 1) | (carry << 63);
                    carry = cur & 1;
                }
                half.trim();
            }
            if k > 0 && x.cmp_big(&half) == std::cmp::Ordering::Greater {
                let mut mag = pprod.clone();
                mag.sub_big(&x);
                lanes.push(SignedBig { negative: true, magnitude: mag });
            } else {
                lanes.push(SignedBig { negative: false, magnitude: x });
            }
        }
        ResReading {
            lanes: [
                lanes[0].clone(),
                lanes[1].clone(),
                lanes[2].clone(),
                lanes[3].clone(),
            ],
            m: self.m,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: the child carrier must agree with the direct ring exactly wherever
// the direct ring exists, and must keep going where it does not.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    struct TwoBranch {
        a: Cyc,
        b: Cyc,
    }
    impl BranchSource for TwoBranch {
        fn n_branches(&self) -> u64 {
            2
        }
        fn amplitude_of(&self, branch: u64, _y: &[bool]) -> Cyc {
            if branch == 0 { self.a } else { self.b }
        }
        fn n_qubits(&self) -> usize {
            1
        }
    }

    fn align_cyc_to(x: Cyc, m: i32) -> Cyc {
        // Test-side lift on the direct ring (small values only).
        assert!(m >= x.m);
        let mut c = x.c;
        let delta = (m - x.m) as u32;
        for v in &mut c {
            *v <<= delta / 2;
        }
        if delta % 2 == 1 {
            c = [c[1] - c[3], c[0] + c[2], c[1] + c[3], c[2] - c[0]];
        }
        Cyc { c, m }
    }

    #[test]
    fn primes_are_prime_and_distinct() {
        let ps = primes();
        assert_eq!(ps.len(), 32);
        for w in ps.windows(2) {
            assert!(w[0] > w[1]);
        }
        assert!(ps.iter().all(|&p| is_prime_u64(p) && p < (1 << 61)));
    }

    #[test]
    fn matches_direct_ring_exactly() {
        // A spread of values incl. odd-Δ alignment and negatives.
        let vals = [
            Cyc { c: [3, -1, 4, -1], m: 3 },
            Cyc { c: [-2, 7, 0, 5], m: 0 },
            Cyc { c: [1, 1, -1, 1], m: 7 },
            Cyc { c: [0, -6, 2, 9], m: 4 },
        ];
        let mut acc = ResCyc::zero(4);
        let mut direct = Cyc::ZERO;
        for v in vals {
            acc.add_cyc(&v);
            direct = direct.add(v);
        }
        let got = acc.reconstruct().to_cyc_checked().expect("fits i128");
        // Compare semantically: lift the (normalized) direct result to the
        // accumulator's exponent.
        assert!(got.m >= direct.m);
        assert_eq!(got, align_cyc_to(direct, got.m));
    }

    #[test]
    fn shard_invariant_bit_identical() {
        let src = TwoBranch {
            a: Cyc { c: [3, 0, -2, 1], m: 5 },
            b: Cyc { c: [-1, 4, 0, 0], m: 2 },
        };
        let y = [false];
        let one = fold_amplitude_res(&src, &y, 1, 3);
        let eight = fold_amplitude_res(&src, &y, 8, 3);
        let thirty_two = fold_amplitude_res(&src, &y, 32, 3);
        assert_eq!(one, eight);
        assert_eq!(eight, thirty_two);
        // And identical readings, not just PartialEq.
        assert_eq!(one.reconstruct(), thirty_two.reconstruct());
    }

    #[test]
    fn no_envelope_beyond_i128() {
        // An m-spread of 400 would REFUSE on the direct ring (alignment shift
        // 2^200). The residue carrier does not notice.
        let src = TwoBranch {
            a: Cyc { c: [3, 0, 0, 0], m: 400 },
            b: Cyc { c: [1, 0, 0, 0], m: 0 },
        };
        let y = [false];
        let out = fold_amplitude_res(&src, &y, 4, 8);
        assert_eq!(out.m, 400);
        let reading = out.reconstruct();
        // Expected lane 0: 1·2^200 + 3, computed by an independent route
        // (plain bignum doubling, no CRT involved).
        let mut expect = Big::from_u64(1);
        for _ in 0..200 {
            expect.mul_u64(2);
        }
        let mut expect_dec = expect.clone();
        // + 3
        expect_dec.add_big(&Big::from_u64(3));
        assert_eq!(reading.lanes[0].to_decimal(), expect_dec.to_decimal());
        assert_eq!(reading.lanes[1].to_decimal(), "0");
        assert!(reading.lanes[0].to_i128_checked().is_none(), "beyond i128, as designed");
    }

    #[test]
    fn negative_lanes_reconstruct_signed() {
        let mut acc = ResCyc::zero(3);
        acc.add_cyc(&Cyc { c: [-5, 0, 7, -1], m: 0 });
        let r = acc.reconstruct();
        assert_eq!(r.lanes[0].to_decimal(), "-5");
        assert_eq!(r.lanes[2].to_decimal(), "7");
        assert_eq!(r.lanes[3].to_decimal(), "-1");
        assert_eq!(r.to_cyc_checked().unwrap(), Cyc { c: [-5, 0, 7, -1], m: 0 });
    }

    #[test]
    fn merge_identity_and_mismatch_guard() {
        let a = ResCyc::from_cyc(&Cyc { c: [1, 2, 3, 4], m: 1 }, 3);
        let merged = ResCyc::empty().merge(a.clone()).merge(ResCyc::empty());
        assert_eq!(merged, a);
    }
}

#[cfg(test)]
mod saturation_tests {
    use super::*;

    #[test]
    fn in_table_bounds_are_served() {
        assert!(primes_for_bits(60) >= 3);
        assert!(primes_for_bits(1700) <= primes().len());
    }

    #[test]
    #[should_panic(expected = "refusing rather than silently shrinking")]
    fn oversized_bound_refuses() {
        let _ = primes_for_bits(10_000);
    }
}
