//! The phase: a bond graph over closure ids, its largest component, and whether it winds.
//!
//! # What it is
//!
//! `GANTT2.md`: *a tier's phase is the grammar's fixed point; a spanning bond graph is a
//! liquid*. This module is the graph half of that sentence and stops there.
//!
//! # Spanning, defined — and it is a WINDING
//!
//! A component spans when it carries a NON-CONTRACTIBLE cycle: walking its bonds around a
//! loop returns to the starting node with a net displacement that is not zero. That is
//! measured with a union–find carrying, per node, its displacement relative to its
//! component's root; an edge closing a cycle whose two endpoints disagree exhibits the
//! winding.
//!
//! The rule and this implementation are `holon-lattice::orientation::bond_graph`'s, lifted
//! to an arbitrary node set and an arbitrary number of axes. The weaker "touches both edges
//! of the box" reading is NOT here: it is not invariant under translating the
//! configuration, and a gate that moves when the scene is shifted is not a gate. That is
//! `orientation.rs`'s own finding (`spanning_is_a_winding_and_a_ring_exhibits_one`) and this
//! module inherits it rather than re-deciding it.
//!
//! # The displacement is an INTEGER, on a continuous carrier too
//!
//! On a lattice the displacement is the link's own lattice vector and integer by
//! construction. On a continuous periodic box it is the WRAP COUNT: for an edge `a → b`,
//! `n = round((x_b − x_a)/L)` per axis. Around any cycle the raw differences telescope to
//! zero exactly, so the summed minimum-image displacement is `−L` times the summed wrap
//! count — the cycle winds iff the integer sum is nonzero. Using the floating-point
//! minimum-image vectors directly would make winding an equality test on rounding error;
//! this is why the type takes `i32` and not `f64`.
//!
//! # What it refuses
//!
//! **It does not name the phase.** `spanning ⇒ liquid` is a reading a tier makes with its
//! own threshold, on its own carrier, against its own reference. A function returning the
//! word "liquid" would be that threshold hidden inside a library, and the fluid tier has
//! already measured what happens when a bond graph never spans (FLUID-1 branch (c)).

/// One edge of the bond graph: two closure ids and the integer displacement from the first
/// to the second, in units of the carrier's period per axis (see the module header).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhaseEdge<const D: usize> {
    pub a: u32,
    pub b: u32,
    pub displacement: [i32; D],
}

impl<const D: usize> PhaseEdge<D> {
    /// An edge inside the box: no axis is crossed.
    pub fn local(a: u32, b: u32) -> PhaseEdge<D> {
        PhaseEdge { a, b, displacement: [0; D] }
    }
}

/// The bond graph's reading.
#[derive(Clone, Debug, PartialEq)]
pub struct Phase<const D: usize> {
    /// How many closures were offered as nodes. Every one counts towards the fraction,
    /// including the isolated ones — a spanning cluster is a fraction OF THE CARRIER.
    pub nodes: usize,
    /// How many edges were walked.
    pub edges: usize,
    /// The size of the largest connected component, in nodes.
    pub largest: usize,
    /// `largest / nodes`; `NaN` on an empty node set, which is not a fraction of anything.
    pub largest_fraction: f64,
    /// Whether the largest component winds on each axis.
    pub largest_winds: [bool; D],
    /// Whether the largest component winds on any axis.
    pub largest_spans: bool,
    /// Whether ANY component winds. Reported beside, since the largest is the one a freeze
    /// asks for and a smaller spanning component is a different fact.
    pub any_spans: bool,
    /// Mean degree `2E/N` — the "bonds per node" convention that counts each bond at both
    /// ends. `NaN` on an empty node set.
    pub degree: f64,
}

fn find(parent: &mut [u32], pot: &mut [Vec<i32>], d: usize, x: u32) -> (u32, Vec<i32>) {
    let mut root = x;
    let mut acc = vec![0i32; d];
    while parent[root as usize] != root {
        for k in 0..d {
            acc[k] += pot[root as usize][k];
        }
        root = parent[root as usize];
    }
    let mut cur = x;
    let mut cacc = acc.clone();
    while parent[cur as usize] != cur {
        let next = parent[cur as usize];
        let p = pot[cur as usize].clone();
        parent[cur as usize] = root;
        pot[cur as usize] = cacc.clone();
        for k in 0..d {
            cacc[k] -= p[k];
        }
        cur = next;
    }
    (root, acc)
}

/// The phase reading of a bond graph over `nodes` closures.
///
/// Node ids are `0..nodes`; an edge naming an id outside that range is SKIPPED and counted
/// out of `edges`, because a graph over ids the caller did not declare is a different graph
/// and silently growing the node set would move the fraction.
pub fn phase<const D: usize>(nodes: usize, edges: &[PhaseEdge<D>]) -> Phase<D> {
    let mut parent: Vec<u32> = (0..nodes as u32).collect();
    let mut pot: Vec<Vec<i32>> = vec![vec![0i32; D]; nodes];
    let inside: Vec<&PhaseEdge<D>> =
        edges.iter().filter(|e| (e.a as usize) < nodes && (e.b as usize) < nodes).collect();

    for e in &inside {
        let (ru, au) = find(&mut parent, &mut pot, D, e.a);
        let (rv, av) = find(&mut parent, &mut pot, D, e.b);
        if ru != rv {
            parent[rv as usize] = ru;
            let mut p = vec![0i32; D];
            for k in 0..D {
                p[k] = e.displacement[k] + au[k] - av[k];
            }
            pot[rv as usize] = p;
        }
    }

    let mut size = vec![0u32; nodes];
    for i in 0..nodes {
        let (r, _) = find(&mut parent, &mut pot, D, i as u32);
        size[r as usize] += 1;
    }
    let mut best_root = 0u32;
    let mut best = 0u32;
    for (r, &sz) in size.iter().enumerate() {
        if sz > best {
            best = sz;
            best_root = r as u32;
        }
    }

    // Second pass for the windings, once every root is final.
    let mut winds_root: Vec<[bool; D]> = vec![[false; D]; nodes];
    let mut any = false;
    for e in &inside {
        let (ru, au) = find(&mut parent, &mut pot, D, e.a);
        let (_rv, av) = find(&mut parent, &mut pot, D, e.b);
        let mut w = [0i32; D];
        let mut nonzero = false;
        for k in 0..D {
            w[k] = au[k] + e.displacement[k] - av[k];
            nonzero |= w[k] != 0;
        }
        if nonzero {
            any = true;
            for k in 0..D {
                winds_root[ru as usize][k] |= w[k] != 0;
            }
        }
    }

    let largest_winds = if nodes == 0 { [false; D] } else { winds_root[best_root as usize] };
    let largest_spans = largest_winds.iter().any(|&x| x);
    let (fraction, degree) = if nodes == 0 {
        (f64::NAN, f64::NAN)
    } else {
        (best as f64 / nodes as f64, 2.0 * inside.len() as f64 / nodes as f64)
    };

    Phase {
        nodes,
        edges: inside.len(),
        largest: best as usize,
        largest_fraction: fraction,
        largest_winds,
        largest_spans,
        any_spans: any,
        degree,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The rule `holon-lattice::orientation`'s test states, on the generic graph: a ring
    /// closed AROUND the torus winds; an open chain of the same length does not.
    #[test]
    fn spanning_is_a_winding_and_a_ring_exhibits_one() {
        // eight nodes in a row on an 8-long torus; the eighth links back to the first
        // across the seam, displacement +1 box.
        let mut e: Vec<PhaseEdge<2>> = (0..7).map(|i| PhaseEdge::local(i, i + 1)).collect();
        e.push(PhaseEdge { a: 7, b: 0, displacement: [1, 0] });
        let p = phase(8, &e);
        assert_eq!(p.largest, 8);
        assert_eq!(p.largest_fraction, 1.0);
        assert!(p.largest_spans, "a closed ring around the torus does not read as spanning");
        assert_eq!(p.largest_winds, [true, false]);
        assert!(p.any_spans);

        let open: Vec<PhaseEdge<2>> = (0..7).map(|i| PhaseEdge::local(i, i + 1)).collect();
        let q = phase(8, &open);
        assert_eq!(q.largest, 8);
        assert!(!q.largest_spans, "an open chain read as spanning");
        assert!(!q.any_spans);
    }

    /// The winding must not depend on how the union-find happened to root the component.
    #[test]
    fn the_winding_is_invariant_under_the_order_the_edges_are_walked() {
        let mut e: Vec<PhaseEdge<2>> = (0..7).map(|i| PhaseEdge::local(i, i + 1)).collect();
        e.push(PhaseEdge { a: 7, b: 0, displacement: [1, 0] });
        let forward = phase(8, &e);
        e.reverse();
        let backward = phase(8, &e);
        assert_eq!(forward.largest_winds, backward.largest_winds);
        assert_eq!(forward.largest, backward.largest);
        assert_eq!(forward.any_spans, backward.any_spans);
    }

    /// A smaller component may wind while the largest does not: two different facts, both
    /// reported.
    #[test]
    fn any_spans_is_reported_beside_the_largest_and_they_can_disagree() {
        // component A: nodes 0..3, a wound triangle. component B: nodes 4..8, a chain.
        let e: Vec<PhaseEdge<1>> = vec![
            PhaseEdge::local(0, 1),
            PhaseEdge::local(1, 2),
            PhaseEdge { a: 2, b: 0, displacement: [1] },
            PhaseEdge::local(4, 5),
            PhaseEdge::local(5, 6),
            PhaseEdge::local(6, 7),
            PhaseEdge::local(7, 8),
        ];
        let p = phase(9, &e);
        assert_eq!(p.largest, 5, "the chain is the larger component");
        assert!(!p.largest_spans);
        assert!(p.any_spans, "the smaller component's winding is reported and not lost");
    }

    #[test]
    fn an_isolated_node_still_counts_towards_the_fraction() {
        let e: Vec<PhaseEdge<2>> = vec![PhaseEdge::local(0, 1)];
        let p = phase(4, &e);
        assert_eq!(p.largest, 2);
        assert_eq!(p.largest_fraction, 0.5);
        assert_eq!(p.degree, 0.5);
    }

    #[test]
    fn an_edge_naming_an_undeclared_node_is_skipped_and_not_counted() {
        let e: Vec<PhaseEdge<2>> = vec![PhaseEdge::local(0, 1), PhaseEdge::local(1, 9)];
        let p = phase(3, &e);
        assert_eq!(p.edges, 1);
        assert_eq!(p.nodes, 3);
        assert_eq!(p.largest, 2);
    }

    #[test]
    fn an_empty_node_set_has_no_fraction_rather_than_a_zero_one() {
        let p = phase::<2>(0, &[]);
        assert!(p.largest_fraction.is_nan());
        assert!(p.degree.is_nan());
        assert_eq!(p.largest, 0);
        assert!(!p.largest_spans);
    }

    /// Three axes, for the continuous carrier the lens reads.
    #[test]
    fn a_three_axis_graph_winds_on_the_axis_it_crosses() {
        let e: Vec<PhaseEdge<3>> = vec![
            PhaseEdge::local(0, 1),
            PhaseEdge::local(1, 2),
            PhaseEdge { a: 2, b: 0, displacement: [0, 0, -1] },
        ];
        let p = phase(3, &e);
        assert_eq!(p.largest_winds, [false, false, true]);
        assert!(p.largest_spans);
    }
}
