//! The ledger ring, from the outside.
//!
//! `holon::ledger::Cyc` keeps `normalize` and the one-shot denominator alignment
//! private, and rightly so — they are the ring's business. This module reaches
//! the two operations the GPU path needs THROUGH the public surface rather than
//! reimplementing them, so there is one definition of each in the tree:
//!
//! * [`normalize`] is `x.mul(Cyc::ONE)`, because `Cyc::mul` normalizes its
//!   result and multiplying by one changes nothing else;
//! * [`align_to`] is the alignment branch of `Cyc::add`, transcribed. This one
//!   IS a transcription and cannot be anything else (the alignment is not
//!   reachable on its own), so it is pinned by
//!   `tests/ring.rs::align_matches_the_ledgers_own_addition`, which drives it
//!   against `Cyc::add` over a sweep of exponent gaps INCLUDING the odd ones
//!   where the sqrt(2) multiply fires.

use holon::ledger::Cyc;

/// The ring's own normal form, obtained through the ring: `mul` normalizes.
#[inline]
pub fn normalize(x: Cyc) -> Cyc {
    x.mul(Cyc::ONE)
}

/// Multiply by `omega^r`. `omega^8 = 1` and `omega^4 = -1`, so this is a
/// rotation of the four coefficients with a sign on wraparound — and because it
/// only permutes and negates, it does not change which coefficients are even,
/// so a normalized input gives a normalized output and `m` is untouched.
///
/// This is the whole reason the device never multiplies in the ring: `i^ip` and
/// `(-1)^sign` are `omega^{2 ip}` and `omega^{4 sign}`.
#[inline]
pub fn rot(x: Cyc, r: u8) -> Cyc {
    let mut c = x.c;
    for _ in 0..(r % 8) {
        c = [-c[3], c[0], c[1], c[2]];
    }
    Cyc { c, m: x.m }
}

/// `x`'s coefficients re-expressed at denominator exponent `m >= x.m`.
///
/// Transcribed from `Cyc::add`: shift by `delta/2`, and when `delta` is odd
/// multiply by `sqrt(2) = omega - omega^3` once. Panics if `m < x.m`, which
/// would be a rounding, not an alignment.
pub fn align_to(x: Cyc, m: i32) -> [i128; 4] {
    assert!(m >= x.m, "align_to: {m} is below the element's own exponent {}", x.m);
    let delta = (m - x.m) as u32;
    let mut c = x.c;
    for v in &mut c {
        *v <<= delta / 2;
    }
    if delta % 2 == 1 {
        let t = c;
        let mut acc = [0i128; 8];
        for p in 0..4 {
            acc[p + 1] += t[p];
            acc[p + 3] -= t[p];
        }
        for p in 0..4 {
            c[p] = acc[p] - acc[p + 4];
        }
    }
    c
}

/// The inverse trip: coefficient lanes at exponent `m`, put back in normal form.
#[inline]
pub fn from_lanes(c: [i128; 4], m: i32) -> Cyc {
    normalize(Cyc { c, m })
}

/// `max |c_i|` as a `u128` — the input to the overflow guard on a batch. `u128`
/// and not `i128` because `|i128::MIN|` does not fit in an `i128`, and a guard
/// that overflows is not a guard.
#[inline]
pub fn magnitude(c: &[i128; 4]) -> u128 {
    c.iter().map(|v| v.unsigned_abs()).max().unwrap_or(0)
}
