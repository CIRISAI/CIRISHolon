//! THE MEMBERSHIP VIEW: the partition of arena indices induced by the bonded-pair graph.
//!
//! This is `v` in the Object contract's commuting square, and everything the census does
//! is a statement about it. Two rules govern the module:
//!
//! 1. **The edge set is the engine's.** The bits come from the dump, which got them from
//!    `Sim::refresh_pairs`. Nothing here decides what a bond is.
//! 2. **The labels are ARENA indices** (Object rule 6: identity is arena-level and
//!    append-only). A block is a set of arena indices, so two frames agree about a block
//!    only when the same physical nuclei are in it. A spatially-sorted or
//!    component-rooted label would make a block that lost an atom and gained another read
//!    as the same block, which is precisely the confusion the census exists to remove.

/// A set of atoms, as a bitmask over arena indices. The trajectory format caps scenes at
/// 16 atoms, so 16 bits is the whole space.
pub type Mask = u16;

/// A partition, canonically labelled: entry `i` is the SMALLEST arena index in `i`'s
/// component. Canonical labelling is what makes two partitions comparable by value.
pub type Labels = Vec<u8>;

/// Union-find over the frame's bonded edges, path-halved, then canonicalised.
///
/// The union-find is the same algorithm `Sim::cluster_roots` runs; it is re-run here
/// rather than dumped because the ROOT an unordered union-find lands on is an artefact of
/// merge order, and the census compares partitions across frames. Canonicalising to the
/// minimum member makes the comparison a fact about the partition rather than about the
/// order the edges arrived in.
pub fn labels_from_bonds(n: usize, bonded: u128) -> Labels {
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let mut k = 0usize;
    for i in 0..n {
        for j in (i + 1)..n {
            if bonded >> k & 1 == 1 {
                let (a, b) = (find(&mut parent, i), find(&mut parent, j));
                if a != b {
                    parent[a] = b;
                }
            }
            k += 1;
        }
    }
    // Canonical label = the least arena index in the component.
    let mut least = vec![u8::MAX; n];
    let roots: Vec<usize> = (0..n).map(|i| find(&mut parent, i)).collect();
    for i in 0..n {
        let r = roots[i];
        if (i as u8) < least[r] {
            least[r] = i as u8;
        }
    }
    (0..n).map(|i| least[roots[i]]).collect()
}

/// The blocks of a partition, as masks, in ascending order of least member.
pub fn blocks(labels: &Labels) -> Vec<Mask> {
    let n = labels.len();
    let mut out: Vec<Mask> = Vec::new();
    for lead in 0..n {
        if labels[lead] as usize != lead {
            continue; // not the canonical representative of its block
        }
        let mut m: Mask = 0;
        for (i, &l) in labels.iter().enumerate() {
            if l as usize == lead {
                m |= 1 << i;
            }
        }
        out.push(m);
    }
    out
}

/// A partition as a single comparable value.
///
/// Sixteen atoms, four bits of canonical label each, is exactly 64 bits. Used as the
/// reading `v(x)` in the closure leg, where partitions are compared and counted by the
/// million and a `Vec` key would dominate the cost.
pub fn key(labels: &Labels) -> u64 {
    debug_assert!(labels.len() <= 16);
    let mut k = 0u64;
    for (i, &l) in labels.iter().enumerate() {
        debug_assert!(l < 16);
        k |= (l as u64) << (4 * i);
    }
    k
}

/// The nuclear composition of a block, as `(Z, count)` pairs sorted by `Z`.
pub fn composition(m: Mask, z: &[u32]) -> Vec<(u32, usize)> {
    let mut out: Vec<(u32, usize)> = Vec::new();
    for (i, &zi) in z.iter().enumerate() {
        if m >> i & 1 == 1 {
            match out.iter_mut().find(|(zz, _)| *zz == zi) {
                Some(slot) => slot.1 += 1,
                None => out.push((zi, 1)),
            }
        }
    }
    out.sort_unstable();
    out
}

/// A block's formula in the census's own notation: heavier nuclei first, `O`, `OH2`, `H2`.
///
/// The SAME notation `waterquench`'s `fmt_comp` prints, so a census row and a quench row
/// naming the same object read the same. It is deliberately the notation of the thing
/// being criticised: the whole point is to put "the formula says OH2" and "the closure
/// test says transient" in one table, and they cannot be compared if they are spelled
/// differently.
pub fn formula(m: Mask, z: &[u32]) -> String {
    let mut comp = composition(m, z);
    comp.sort_by_key(|(zz, _)| std::cmp::Reverse(*zz));
    let mut s = String::new();
    for (zz, c) in comp {
        s.push_str(symbol(zz));
        if c > 1 {
            s.push_str(&c.to_string());
        }
    }
    if s.is_empty() {
        s.push('-');
    }
    s
}

fn symbol(z: u32) -> &'static str {
    match z {
        1 => "H",
        6 => "C",
        7 => "N",
        8 => "O",
        _ => "X",
    }
}

pub fn popcount(m: Mask) -> usize {
    m.count_ones() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    // Three atoms; bit 0 = (0,1), bit 1 = (0,2), bit 2 = (1,2).
    #[test]
    fn no_bonds_is_all_singletons() {
        let l = labels_from_bonds(3, 0);
        assert_eq!(l, vec![0, 1, 2]);
        assert_eq!(blocks(&l), vec![0b001, 0b010, 0b100]);
    }

    #[test]
    fn one_edge_merges_exactly_two() {
        let l = labels_from_bonds(3, 0b001); // (0,1)
        assert_eq!(l, vec![0, 0, 2]);
        assert_eq!(blocks(&l), vec![0b011, 0b100]);
    }

    #[test]
    fn transitive_closure_is_one_block() {
        // (0,1) and (1,2) but NOT (0,2): still one component of three.
        let l = labels_from_bonds(3, 0b101);
        assert_eq!(l, vec![0, 0, 0]);
        assert_eq!(blocks(&l), vec![0b111]);
    }

    /// The canonical label must not depend on the order edges were merged in.
    ///
    /// This is the property the whole census leans on: the same partition arrived at two
    /// ways must compare EQUAL, or the closure leg would report witness pairs that are
    /// artefacts of union-find bookkeeping.
    #[test]
    fn canonical_labels_are_merge_order_independent() {
        let a = labels_from_bonds(4, 0b000_011); // (0,1),(0,2)
        let b = labels_from_bonds(4, 0b001_001); // (0,1),(1,2)
        assert_eq!(a, b);
        assert_eq!(key(&a), key(&b));
        assert_eq!(blocks(&a), vec![0b0111, 0b1000]);
    }

    #[test]
    fn key_separates_distinct_partitions() {
        let all_free = labels_from_bonds(4, 0);
        let one_pair = labels_from_bonds(4, 0b000_001);
        assert_ne!(key(&all_free), key(&one_pair));
    }

    #[test]
    fn formula_matches_the_quench_notation() {
        let z = [8, 1, 1, 1];
        assert_eq!(formula(0b0111, &z), "OH2");
        assert_eq!(formula(0b0011, &z), "OH");
        assert_eq!(formula(0b0110, &z), "H2");
        assert_eq!(formula(0b1110, &z), "H3");
        assert_eq!(formula(0b0001, &z), "O");
        assert_eq!(composition(0b0111, &z), vec![(1, 2), (8, 1)]);
    }
}
