//! Relationships: the population law on pairs.
//!
//! A relationship is a count on an ordered pair of objects. The coarse relationship between
//! two parents is the sum of the fine relationships between their children ([`quotient`]).
//! The tests are `ScenePopulation`'s and `Ladder`'s edge theorems run on random graphs.

/// A fine edge list: `(from, to, count)` over objects `0..n`.
pub type Edges = Vec<(usize, usize, u64)>;

/// The coarse edge table: entry `[c][d]` is the sum of the fine counts from a child of `c`
/// to a child of `d`. `parent_of[x]` is the parent of object `x`; parents are `0..n_coarse`.
/// The diagonal holds the relationships internal to each parent.
pub fn quotient(parent_of: &[usize], n_coarse: usize, edges: &[(usize, usize, u64)]) -> Vec<Vec<u64>> {
    let mut out = vec![vec![0u64; n_coarse]; n_coarse];
    for &(x, y, w) in edges {
        out[parent_of[x]][parent_of[y]] += w;
    }
    out
}

/// The same table as an edge list, for reading coarsely a second time.
pub fn as_edges(table: &[Vec<u64>]) -> Edges {
    let mut out = Vec::new();
    for (c, row) in table.iter().enumerate() {
        for (d, &w) in row.iter().enumerate() {
            if w > 0 {
                out.push((c, d, w));
            }
        }
    }
    out
}

/// What leaves each object, to anything.
pub fn outflow(n: usize, edges: &[(usize, usize, u64)]) -> Vec<u64> {
    let mut out = vec![0u64; n];
    for &(x, _, w) in edges {
        out[x] += w;
    }
    out
}

/// A parent's reading of a per-object count: the sum over its children.
pub fn pop(parent_of: &[usize], n_coarse: usize, count: &[u64]) -> Vec<u64> {
    let mut out = vec![0u64; n_coarse];
    for (x, &c) in count.iter().enumerate() {
        out[parent_of[x]] += c;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::Stream;

    fn random_world(seed: u64) -> (Vec<usize>, Vec<usize>, Edges) {
        // 60 objects in 12 districts in 3 regions, 400 random relationships
        let mut s = Stream::new(seed);
        let v: Vec<usize> = (0..60).map(|_| s.below(12) as usize).collect();
        let u: Vec<usize> = (0..12).map(|_| s.below(3) as usize).collect();
        let e: Edges = (0..400).map(|_| (s.below(60) as usize, s.below(60) as usize, 1 + s.below(5))).collect();
        (v, u, e)
    }

    #[test]
    fn nothing_is_created_or_lost() {
        for seed in 0..50 {
            let (v, _, e) = random_world(seed);
            let fine: u64 = e.iter().map(|x| x.2).sum();
            let coarse: u64 = quotient(&v, 12, &e).iter().flatten().sum();
            assert_eq!(fine, coarse);
        }
    }

    #[test]
    fn relationships_aggregate_through_every_level() {
        for seed in 0..50 {
            let (v, u, e) = random_world(seed);
            let two_steps = quotient(&u, 3, &as_edges(&quotient(&v, 12, &e)));
            let direct: Vec<usize> = v.iter().map(|&c| u[c]).collect();
            assert_eq!(two_steps, quotient(&direct, 3, &e));
        }
    }

    #[test]
    fn outflow_commutes_with_reading_coarsely() {
        for seed in 0..50 {
            let (v, _, e) = random_world(seed);
            let coarse_then_out: Vec<u64> = quotient(&v, 12, &e).iter().map(|row| row.iter().sum()).collect();
            let out_then_coarse = pop(&v, 12, &outflow(60, &e));
            assert_eq!(coarse_then_out, out_then_coarse);
        }
    }

    #[test]
    fn an_internal_relationship_is_read_on_its_parents_diagonal() {
        let v = [0usize, 0, 1, 1];
        let t = quotient(&v, 2, &[(0, 1, 1)]);
        assert_eq!(t, vec![vec![1, 0], vec![0, 0]]);
    }

    #[test]
    fn the_coarse_edge_does_not_fix_the_wiring() {
        // ScenePopulation.edge_law_does_not_fix_wiring: two cells of two objects each
        let v = [0usize, 0, 1, 1];
        let a: Edges = vec![(0, 2, 1), (1, 3, 1)];
        let b: Edges = vec![(0, 2, 2)];
        assert_eq!(quotient(&v, 2, &a), quotient(&v, 2, &b));
        assert_eq!(outflow(4, &a)[1], 1);
        assert_eq!(outflow(4, &b)[1], 0);
    }
}
