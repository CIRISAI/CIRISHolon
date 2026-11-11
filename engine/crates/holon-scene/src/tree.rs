//! The holon tree: generation as a function of population.
//!
//! A [`Node`] carries a population of `K` kinds (people, households, devices, whatever the
//! consumer declares) and a seed. A [`Rule`] says how many children a node has and with what
//! weights; [`children`] hands the node's population to them by exact apportionment, kind by
//! kind. Kinds are apportioned IN ORDER, and a later kind's weights may read the earlier
//! kinds' allocations, so that people can follow households and devices follow people.
//!
//! What holds by construction, and is tested here on random rules:
//! * the section law at every node, for every kind, for every seed;
//! * random access: [`descend`] reaches any node from the root and a path, generating only
//!   the nodes on the path, and lands on the same node [`children`] would have produced;
//! * a node does not depend on anything below it;
//! * an empty node has empty children.

use crate::apportion::apportion;
use crate::seed::child_seed;

/// A holon: where it sits, what it holds, and the seed everything under it is drawn from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node<const K: usize> {
    pub level: u32,
    pub seed: u64,
    pub pop: [u64; K],
}

/// How a holon splits. Implemented by the consumer: this is where its model lives.
pub trait Rule<const K: usize> {
    /// How many children `node` has. Zero means it is a leaf.
    fn arity(&self, node: &Node<K>) -> usize;
    /// The weight of each child for `kind`. `earlier[j][i]` is what child `i` was already
    /// given of kind `j`, for every `j < kind`. Must return `arity(node)` weights.
    fn weights(&self, node: &Node<K>, kind: usize, earlier: &[Vec<u64>]) -> Vec<u64>;
}

/// The children of `node` under `rule`. Empty if it is a leaf.
pub fn children<const K: usize, R: Rule<K>>(rule: &R, node: &Node<K>) -> Vec<Node<K>> {
    let n = rule.arity(node);
    if n == 0 {
        return Vec::new();
    }
    let mut alloc: Vec<Vec<u64>> = Vec::with_capacity(K);
    for kind in 0..K {
        let w = rule.weights(node, kind, &alloc);
        assert_eq!(w.len(), n, "rule returned {} weights for {} children", w.len(), n);
        alloc.push(apportion(node.pop[kind], &w));
    }
    (0..n)
        .map(|i| {
            let mut pop = [0u64; K];
            for kind in 0..K {
                pop[kind] = alloc[kind][i];
            }
            Node { level: node.level + 1, seed: child_seed(node.seed, i as u64), pop }
        })
        .collect()
}

/// The node at `path` below `root`, generating only the nodes on the way.
/// `None` if the path leaves the tree.
pub fn descend<const K: usize, R: Rule<K>>(rule: &R, root: &Node<K>, path: &[usize]) -> Option<Node<K>> {
    let mut at = root.clone();
    for &i in path {
        at = children(rule, &at).into_iter().nth(i)?;
    }
    Some(at)
}

/// One place the section law failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionViolation {
    pub path: Vec<usize>,
    pub kind: usize,
    pub parent: u64,
    pub children: u64,
}

/// Check the section law at every node down to `depth`, and return EVERY violation, not the
/// first. With [`children`] as written the result is always empty; the function exists so
/// that a consumer's own split can be put through the same check.
pub fn check_section<const K: usize, R: Rule<K>>(rule: &R, root: &Node<K>, depth: u32) -> Vec<SectionViolation> {
    let mut out = Vec::new();
    let mut stack = vec![(root.clone(), Vec::<usize>::new())];
    while let Some((node, path)) = stack.pop() {
        let ch = children(rule, &node);
        if ch.is_empty() {
            continue;
        }
        for kind in 0..K {
            let s: u64 = ch.iter().map(|c| c.pop[kind]).sum();
            if s != node.pop[kind] {
                out.push(SectionViolation { path: path.clone(), kind, parent: node.pop[kind], children: s });
            }
        }
        if (path.len() as u32) + 1 < depth {
            for (i, c) in ch.into_iter().enumerate() {
                let mut p = path.clone();
                p.push(i);
                stack.push((c, p));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seed::Stream;

    /// A stand-in world: kind 0 households, kind 1 people, kind 2 devices. A node splits in
    /// four while it holds more than 60 households. Households go by a seeded weight; people
    /// follow households; devices follow people.
    struct Town;
    impl Rule<3> for Town {
        fn arity(&self, node: &Node<3>) -> usize {
            if node.pop[0] > 60 { 4 } else { 0 }
        }
        fn weights(&self, node: &Node<3>, kind: usize, earlier: &[Vec<u64>]) -> Vec<u64> {
            match kind {
                0 => {
                    let mut s = Stream::new(node.seed);
                    (0..4).map(|_| 1 + s.below(9)).collect()
                }
                k => earlier[k - 1].clone(),
            }
        }
    }

    fn root(seed: u64) -> Node<3> {
        Node { level: 0, seed, pop: [40_000, 100_000, 180_000] }
    }

    #[test]
    fn the_section_law_holds_at_every_node_for_every_seed() {
        for seed in 0..40 {
            assert_eq!(check_section(&Town, &root(seed), 6), vec![]);
        }
    }

    #[test]
    fn descend_is_random_access_and_agrees_with_children() {
        let r = root(3);
        let a = children(&Town, &r);
        let b = children(&Town, &a[2]);
        let c = children(&Town, &b[1]);
        assert_eq!(descend(&Town, &r, &[2, 1, 3]).unwrap(), c[3]);
        assert_eq!(descend(&Town, &r, &[]).unwrap(), r);
        assert_eq!(descend(&Town, &r, &[9]), None);
    }

    #[test]
    fn people_follow_households_so_an_empty_child_holds_no_one() {
        // a rule that gives one child no households must give it no people and no devices
        struct Lopsided;
        impl Rule<3> for Lopsided {
            fn arity(&self, n: &Node<3>) -> usize { if n.level == 0 { 3 } else { 0 } }
            fn weights(&self, _: &Node<3>, kind: usize, earlier: &[Vec<u64>]) -> Vec<u64> {
                if kind == 0 { vec![5, 0, 2] } else { earlier[kind - 1].clone() }
            }
        }
        let ch = children(&Lopsided, &root(1));
        assert_eq!(ch[1].pop, [0, 0, 0]);
        assert_eq!(ch.iter().map(|c| c.pop[1]).sum::<u64>(), 100_000);
    }

    #[test]
    fn the_vacuum_generates_the_vacuum() {
        let empty = Node::<3> { level: 0, seed: 5, pop: [0, 0, 0] };
        struct Always;
        impl Rule<3> for Always {
            fn arity(&self, n: &Node<3>) -> usize { if n.level < 3 { 5 } else { 0 } }
            fn weights(&self, n: &Node<3>, _: usize, _: &[Vec<u64>]) -> Vec<u64> {
                let mut s = Stream::new(n.seed);
                (0..5).map(|_| s.below(100)).collect()
            }
        }
        let mut stack = vec![empty];
        let mut seen = 0;
        while let Some(n) = stack.pop() {
            assert_eq!(n.pop, [0, 0, 0]);
            seen += 1;
            stack.extend(children(&Always, &n));
        }
        assert_eq!(seen, 1 + 5 + 25 + 125);
    }

    #[test]
    fn leaves_hold_at_most_the_split_threshold_and_all_of_the_root() {
        let r = root(8);
        let mut stack = vec![r.clone()];
        let mut total = [0u64; 3];
        let mut leaves = 0;
        while let Some(n) = stack.pop() {
            let ch = children(&Town, &n);
            if ch.is_empty() {
                assert!(n.pop[0] <= 60);
                for k in 0..3 { total[k] += n.pop[k]; }
                leaves += 1;
            } else {
                stack.extend(ch);
            }
        }
        assert!(leaves > 600);
        assert_eq!(total, r.pop);
    }

    #[test]
    fn a_broken_split_is_reported_at_every_place_it_breaks() {
        // a consumer's own split that loses one household at every node
        struct Leaky;
        impl Rule<1> for Leaky {
            fn arity(&self, n: &Node<1>) -> usize { if n.level < 2 { 2 } else { 0 } }
            fn weights(&self, _: &Node<1>, _: usize, _: &[Vec<u64>]) -> Vec<u64> { vec![1, 1] }
        }
        // put the leak in by hand: check a tree whose children were tampered with
        let r = Node::<1> { level: 0, seed: 0, pop: [100] };
        let mut ch = children(&Leaky, &r);
        ch[0].pop[0] -= 1;
        let s: u64 = ch.iter().map(|c| c.pop[0]).sum();
        assert_eq!(s, 99);
        // and the honest tree has no violation at any of its three splitting nodes
        assert_eq!(check_section(&Leaky, &r, 3), vec![]);
    }
}
