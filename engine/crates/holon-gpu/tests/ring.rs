//! `ring::align_to` is a TRANSCRIPTION of the alignment branch inside
//! `Cyc::add` — the only piece of the ledger this crate could not reach through
//! the public surface. A transcription is a place where two copies can drift, so
//! it gets pinned against the original rather than trusted.
//!
//! No GPU needed; this suite runs anywhere.

use holon::ledger::Cyc;
use holon::merge::MergeLedger;
use holon_gpu::ring;

fn sample(seed: u64) -> Cyc {
    let mut x = seed | 1;
    let mut nxt = || {
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    let mut c = [0i128; 4];
    for v in c.iter_mut() {
        *v = (nxt() % 41) as i128 - 20;
    }
    if c.iter().all(|v| *v == 0) {
        c[0] = 1;
    }
    ring::normalize(Cyc { c, m: (nxt() % 9) as i32 })
}

#[test]
fn align_matches_the_ledgers_own_addition() {
    // `x + 0@m` is the ledger aligning `x` up to `m` and adding nothing — except
    // that `Cyc::add` short-circuits on a zero operand, so the zero has to carry
    // weight. Add a KNOWN element at the target exponent instead, and subtract
    // what it contributes; whatever is left is the alignment, and it must equal
    // `align_to` coefficient for coefficient.
    for seed in 1..400u64 {
        let x = sample(seed);
        for gap in 0..7i32 {
            let m = x.m + gap;
            let pad = Cyc { c: [1, 0, 0, 0], m };
            let via_ledger = x.merge(pad);
            let mine = ring::align_to(x, m);
            let expect = Cyc { c: [mine[0] + 1, mine[1], mine[2], mine[3]], m };
            assert_eq!(
                via_ledger,
                ring::normalize(expect),
                "seed {seed}, gap {gap}: align_to disagrees with Cyc::add's own alignment"
            );
        }
    }
}

#[test]
fn align_preserves_the_value() {
    for seed in 1..200u64 {
        let x = sample(seed);
        for gap in 0..7i32 {
            let back = ring::from_lanes(ring::align_to(x, x.m + gap), x.m + gap);
            // Equal as a NUMBER: post the credit against the debit and check it
            // clears, which is `prune::cyc_eq`'s test and is faithful where the
            // derived PartialEq is not (sqrt(2) has two normalized faces).
            let diff = back.merge(Cyc { c: [-x.c[0], -x.c[1], -x.c[2], -x.c[3]], m: x.m });
            assert!(diff.c.iter().all(|&v| v == 0), "seed {seed}, gap {gap}: value moved");
        }
    }
}

#[test]
fn rot_is_multiplication_by_omega_and_normalization_survives_it() {
    // The device never multiplies in the ring; it rotates. That is only sound if
    // rotation IS multiplication by omega^r and leaves `m` where it was.
    let i = Cyc { c: [0, 0, 1, 0], m: 0 }; // omega^2 = i
    let w = Cyc { c: [0, 1, 0, 0], m: 0 }; // omega
    for seed in 1..300u64 {
        let x = sample(seed);
        let mut by_mul = x;
        for r in 0..8u8 {
            assert_eq!(
                ring::rot(x, r),
                by_mul,
                "seed {seed}, r = {r}: rot is not multiplication by omega^r"
            );
            assert_eq!(ring::rot(x, r).m, x.m, "seed {seed}, r = {r}: rotation moved m");
            by_mul = by_mul.mul(w);
        }
        // and the two the kernel actually uses
        assert_eq!(ring::rot(x, 2), x.mul(i));
        assert_eq!(ring::rot(x, 4), x.mul(i).mul(i));
    }
}

#[test]
fn rot_wraps_at_eight() {
    for seed in 1..50u64 {
        let x = sample(seed);
        assert_eq!(ring::rot(x, 8), x);
        assert_eq!(ring::rot(x, 9), ring::rot(x, 1));
    }
}
