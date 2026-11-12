//! Households in a unit: whole households whose sizes sum to the unit's people, for every
//! seed. The first generator this crate ships for a consumer to call (HOUSEGEN-1,
//! `conformance/scene/HOUSEGEN1_PREREG.md`).
//!
//! The section law here is `ScenePopulation.pop_total` with the households as the children
//! and their sizes as the populations. The draw rescales nothing: sizes come one at a time
//! from a row of shares, and only the last household is touched to make the sum exact.

use crate::seed::{mix, Stream};

/// Pick an index from shares in any common unit (parts per million here).
fn pick(st: &mut Stream, shares: &[u64], total: u64) -> usize {
    let mut u = st.below(total);
    for (k, &w) in shares.iter().enumerate() {
        if u < w {
            return k;
        }
        u -= w;
    }
    shares.len() - 1
}

/// Draw household sizes summing to `people`.
///
/// `shares[k]` is the share of households of size `sizes[k]`. When a drawn size `s` exceeds
/// the `r` people left, a household of `r` is kept with probability `r/s`; otherwise the
/// `r` people join one earlier household. A unit with no earlier household keeps `r`.
pub fn draw(seed: u64, people: u64, shares: &[u64], sizes: &[u8]) -> Vec<u8> {
    assert_eq!(shares.len(), sizes.len(), "one size per share");
    assert!(sizes.iter().all(|&s| s > 0), "no empty household");
    let total: u64 = shares.iter().sum();
    assert!(total > 0, "a row with no households");
    let mut st = Stream::new(seed);
    let mut out: Vec<u8> = Vec::new();
    let mut left = people;
    while left > 0 {
        let s = sizes[pick(&mut st, shares, total)] as u64;
        if s <= left {
            out.push(s as u8);
            left -= s;
        } else {
            let r = left;
            if out.is_empty() || st.below(s) < r {
                out.push(r as u8);
            } else {
                let j = st.below(out.len() as u64) as usize;
                out[j] += r as u8;
            }
            left = 0;
        }
    }
    out
}

/// The draft's draw, kept as a PLANT: the last household simply takes what is left. It
/// obeys the section law and is biased toward small households (HOUSEGEN-1's review).
pub fn draw_truncating(seed: u64, people: u64, shares: &[u64], sizes: &[u8]) -> Vec<u8> {
    let total: u64 = shares.iter().sum();
    let mut st = Stream::new(seed);
    let mut out: Vec<u8> = Vec::new();
    let mut left = people;
    while left > 0 {
        let s = (sizes[pick(&mut st, shares, total)] as u64).min(left);
        out.push(s as u8);
        left -= s;
    }
    out
}

/// One number for 1,000 fixed calls, to compare builds (native against wasm32).
pub fn digest(shares: &[u64], sizes: &[u8]) -> u64 {
    let mut h = 0u64;
    for i in 0..1000u64 {
        let v = draw(mix(i), (i * 37) % 2001, shares, sizes);
        h = mix(h ^ v.len() as u64);
        for s in v {
            h = mix(h ^ s as u64);
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROW: [u64; 6] = [434_010, 305_330, 124_260, 91_074, 29_674, 15_652];
    const SIZES: [u8; 6] = [1, 2, 3, 4, 5, 7];

    fn sum(v: &[u8]) -> u64 {
        v.iter().map(|&s| s as u64).sum()
    }

    /// G0: the section law, 100,000 calls, people from 0 to 2,000.
    #[test]
    fn sizes_sum_to_people_for_every_seed() {
        let mut st = Stream::new(20261111);
        for i in 0..100_000u64 {
            let people = st.below(2001);
            let v = draw(mix(i), people, &ROW, &SIZES);
            assert_eq!(sum(&v), people);
            assert!(v.iter().all(|&s| s > 0));
        }
    }

    /// P-ZERO.
    #[test]
    fn nobody_gives_no_household() {
        assert!(draw(7, 0, &ROW, &SIZES).is_empty());
    }

    /// P-ONES.
    #[test]
    fn a_row_of_ones_gives_one_household_each() {
        for n in [1u64, 2, 17, 300] {
            let v = draw(n, n, &[1_000_000], &[1]);
            assert_eq!(v.len() as u64, n);
            assert!(v.iter().all(|&s| s == 1));
        }
    }

    /// P-FOURS: units of 10 from a row of fours give two or three households, and both occur.
    #[test]
    fn a_row_of_fours_in_units_of_ten() {
        let (mut two, mut three) = (0, 0);
        for seed in 0..1000u64 {
            let v = draw(mix(seed), 10, &[1_000_000], &[4]);
            assert_eq!(sum(&v), 10);
            match v.len() {
                2 => two += 1,
                3 => {
                    three += 1;
                    assert_eq!(v, vec![4, 4, 2]);
                }
                n => panic!("{n} households"),
            }
        }
        assert!(two > 300 && three > 300, "{two} {three}");
    }

    /// P-SEED.
    #[test]
    fn one_seed_one_scene() {
        assert_eq!(draw(99, 300, &ROW, &SIZES), draw(99, 300, &ROW, &SIZES));
        let differ = (0..1000u64).filter(|&i| draw(mix(i), 300, &ROW, &SIZES) != draw(mix(i + 5000), 300, &ROW, &SIZES)).count();
        assert!(differ >= 990, "{differ}");
    }

    /// P-BROKEN: without the last step the law fails, so the test above is not vacuous.
    #[test]
    fn without_the_last_step_the_law_fails() {
        fn broken(seed: u64, people: u64) -> Vec<u8> {
            let total: u64 = ROW.iter().sum();
            let mut st = Stream::new(seed);
            let (mut out, mut left) = (Vec::new(), people);
            while left > 0 {
                let s = SIZES[pick(&mut st, &ROW, total)] as u64;
                out.push(s as u8);
                left = left.saturating_sub(s);
            }
            out
        }
        let bad = (0..1000u64).filter(|&i| sum(&broken(mix(i), 150)) != 150).count();
        assert!(bad > 100, "{bad}");
    }

    /// The plant draw also obeys the law; it is its bias that S2's scoring must see.
    #[test]
    fn the_truncating_plant_obeys_the_law_and_adds_households() {
        let (mut a, mut b) = (0usize, 0usize);
        for i in 0..5000u64 {
            let t = draw_truncating(mix(i), 8, &ROW, &SIZES);
            assert_eq!(sum(&t), 8);
            a += t.len();
            b += draw(mix(i), 8, &ROW, &SIZES).len();
        }
        assert!(a > b, "{a} {b}");
    }

    #[test]
    fn the_digest_is_stable() {
        assert_eq!(digest(&ROW, &SIZES), digest(&ROW, &SIZES));
    }
}
