//! Exact apportionment by largest remainder.
//!
//! `apportion(total, weights)` returns one count per weight, summing to `total` EXACTLY.
//! It is the constructive content of the section law: a parent's count can always be handed
//! to its children with nothing created and nothing lost, whatever the weights.
//!
//! Rules, all deterministic:
//! * each child first gets `floor(total * w / sum)`;
//! * the units left over go one each to the largest remainders;
//! * a tie in remainder goes to the lower index;
//! * a child of weight zero gets zero, unless every weight is zero, in which case the
//!   weights are treated as equal (the parent's count has to go somewhere);
//! * no children and a positive total is the one case with no answer: the result is empty
//!   and the caller's section check will report it.

/// Split `total` over `weights`, exactly.
pub fn apportion(total: u64, weights: &[u64]) -> Vec<u64> {
    let n = weights.len();
    if n == 0 {
        return Vec::new();
    }
    let sum: u128 = weights.iter().map(|&w| w as u128).sum();
    if sum == 0 {
        return apportion(total, &vec![1u64; n]);
    }
    let mut out = Vec::with_capacity(n);
    let mut rem: Vec<(u128, usize)> = Vec::with_capacity(n);
    let mut given: u64 = 0;
    for (i, &w) in weights.iter().enumerate() {
        let p = total as u128 * w as u128;
        let q = (p / sum) as u64;
        out.push(q);
        given += q;
        rem.push((p % sum, i));
    }
    let mut left = total - given;
    rem.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    for &(_, i) in rem.iter() {
        if left == 0 {
            break;
        }
        out[i] += 1;
        left -= 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::Stream;

    #[test]
    fn sums_exactly_for_every_draw() {
        let mut s = Stream::new(7);
        for _ in 0..20_000 {
            let n = 1 + s.below(25) as usize;
            let w: Vec<u64> = (0..n).map(|_| if s.below(4) == 0 { 0 } else { s.below(1_000_000) }).collect();
            let total = s.below(10_000_000);
            let a = apportion(total, &w);
            assert_eq!(a.iter().sum::<u64>(), total);
        }
    }

    #[test]
    fn a_zero_weight_gets_zero_unless_all_are_zero() {
        let a = apportion(1_000, &[0, 3, 0, 7]);
        assert_eq!(a, vec![0, 300, 0, 700]);
        let b = apportion(10, &[0, 0, 0]);
        assert_eq!(b.iter().sum::<u64>(), 10);
        assert_eq!(b, vec![4, 3, 3]);
    }

    #[test]
    fn each_child_is_within_one_of_its_exact_share() {
        let mut s = Stream::new(11);
        for _ in 0..5_000 {
            let n = 1 + s.below(12) as usize;
            let w: Vec<u64> = (0..n).map(|_| 1 + s.below(1_000)).collect();
            let total = s.below(100_000);
            let sum: u128 = w.iter().map(|&x| x as u128).sum();
            for (i, &c) in apportion(total, &w).iter().enumerate() {
                let floor = (total as u128 * w[i] as u128 / sum) as u64;
                assert!(c == floor || c == floor + 1);
            }
        }
    }

    #[test]
    fn huge_values_do_not_overflow() {
        let a = apportion(u64::MAX, &[u64::MAX, u64::MAX, 1]);
        assert_eq!(a.iter().map(|&x| x as u128).sum::<u128>(), u64::MAX as u128);
    }

    #[test]
    fn an_empty_parent_gives_empty_children() {
        assert_eq!(apportion(0, &[5, 1, 9]), vec![0, 0, 0]);
    }
}
