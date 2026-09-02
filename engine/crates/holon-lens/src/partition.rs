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
//!
//! There is no scene-size cap here. `Mask` used to be a `u16` and `Labels` a `Vec<u8>`
//! because the v1 trajectory word held sixteen atoms; the carrier outgrew that (v2 carries
//! 92,682 atoms as a sparse pair list) and a reader that still stopped at sixteen was the
//! cap surviving in the instrument. The v1 FORMAT keeps its sixteen — that is a property
//! of a file layout, refused by name at the writer — and nothing in memory does.

use crate::traj::{pair_from_index, BondSet};

/// A set of atoms over ARENA indices, any size.
///
/// Trailing zero words are trimmed after every construction, so two masks over the same
/// atoms compare equal (and hash equal) whatever scene size built them. The derived order
/// is lexicographic on the words — a DETERMINISTIC order for report listings, not the
/// order of least member; [`blocks`] states its own order explicitly.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Default)]
pub struct Mask {
    words: Vec<u64>,
}

impl Mask {
    pub fn empty() -> Self {
        Self::default()
    }

    /// The mask whose bits are `bits`. Fixtures and tests state small blocks this way
    /// (`Mask::from_bits(0b0111)` is atoms 0, 1, 2).
    pub fn from_bits(bits: u128) -> Self {
        let mut m = Self {
            words: vec![bits as u64, (bits >> 64) as u64],
        };
        m.trim();
        m
    }

    pub fn singleton(i: usize) -> Self {
        let mut m = Self::empty();
        m.insert(i);
        m
    }

    /// Every atom of an `n`-atom scene.
    pub fn all(n: usize) -> Self {
        let mut m = Self::empty();
        for i in 0..n {
            m.insert(i);
        }
        m
    }

    pub fn from_members<I: IntoIterator<Item = usize>>(members: I) -> Self {
        let mut m = Self::empty();
        for i in members {
            m.insert(i);
        }
        m
    }

    pub fn insert(&mut self, i: usize) {
        let w = i / 64;
        if w >= self.words.len() {
            self.words.resize(w + 1, 0);
        }
        self.words[w] |= 1u64 << (i % 64);
    }

    #[inline]
    pub fn contains(&self, i: usize) -> bool {
        self.words
            .get(i / 64)
            .map_or(false, |w| w >> (i % 64) & 1 == 1)
    }

    pub fn popcount(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// The members in ascending arena order.
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.words.iter().enumerate().flat_map(|(wi, &w)| {
            let mut rest = w;
            std::iter::from_fn(move || {
                if rest == 0 {
                    None
                } else {
                    let b = rest.trailing_zeros() as usize;
                    rest &= rest - 1;
                    Some(wi * 64 + b)
                }
            })
        })
    }

    pub fn least(&self) -> Option<usize> {
        self.iter().next()
    }

    /// An injective byte encoding for readings that carry masks: the word count, then the
    /// words little-endian. Two distinct masks never share bytes, and a sequence of encoded
    /// masks parses back unambiguously because each carries its own length.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + 8 * self.words.len());
        out.push(u8::try_from(self.words.len()).expect("a mask over fewer than 16,320 atoms"));
        for w in &self.words {
            out.extend_from_slice(&w.to_le_bytes());
        }
        out
    }

    pub fn intersects(&self, other: &Mask) -> bool {
        self.words
            .iter()
            .zip(other.words.iter())
            .any(|(a, b)| a & b != 0)
    }

    fn trim(&mut self) {
        while self.words.last() == Some(&0) {
            self.words.pop();
        }
    }
}

impl From<u128> for Mask {
    fn from(bits: u128) -> Self {
        Self::from_bits(bits)
    }
}

/// A partition, canonically labelled: entry `i` is the SMALLEST arena index in `i`'s
/// component. Canonical labelling is what makes two partitions comparable by value.
pub type Labels = Vec<u32>;

/// Union-find over the frame's bonded edges, path-halved, then canonicalised.
///
/// The union-find is the same algorithm `Sim::cluster_roots` runs; it is re-run here
/// rather than dumped because the ROOT an unordered union-find lands on is an artefact of
/// merge order, and the census compares partitions across frames. Canonicalising to the
/// minimum member makes the comparison a fact about the partition rather than about the
/// order the edges arrived in.
pub fn labels_from_bonds(n: usize, bonded: &BondSet) -> Labels {
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    for k in bonded.iter() {
        let (i, j) = pair_from_index(n, k as usize);
        let (a, b) = (find(&mut parent, i), find(&mut parent, j));
        if a != b {
            parent[a] = b;
        }
    }
    // Canonical label = the least arena index in the component.
    let mut least = vec![u32::MAX; n];
    let roots: Vec<usize> = (0..n).map(|i| find(&mut parent, i)).collect();
    for i in 0..n {
        let r = roots[i];
        if (i as u32) < least[r] {
            least[r] = i as u32;
        }
    }
    (0..n).map(|i| least[roots[i]]).collect()
}

/// The blocks of a partition, as masks, in ascending order of least member.
///
/// One pass: a block's lead is the first index carrying its label, so walking `i` upward
/// creates the blocks in exactly that order.
pub fn blocks(labels: &Labels) -> Vec<Mask> {
    let n = labels.len();
    let mut slot = vec![usize::MAX; n];
    let mut out: Vec<Mask> = Vec::new();
    for (i, &l) in labels.iter().enumerate() {
        let lead = l as usize;
        if slot[lead] == usize::MAX {
            slot[lead] = out.len();
            out.push(Mask::empty());
        }
        out[slot[lead]].insert(i);
    }
    out
}

/// A partition as bytes, for the interner that keys readings in the closure leg
/// (`network::Keyer`). Injective: the labels are written verbatim.
pub fn label_bytes(labels: &Labels) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 * labels.len());
    for &l in labels {
        out.extend_from_slice(&l.to_le_bytes());
    }
    out
}

/// The nuclear composition of a block, as `(Z, count)` pairs sorted by `Z`.
pub fn composition(m: &Mask, z: &[u32]) -> Vec<(u32, usize)> {
    let mut out: Vec<(u32, usize)> = Vec::new();
    for i in m.iter() {
        let zi = z[i];
        match out.iter_mut().find(|(zz, _)| *zz == zi) {
            Some(slot) => slot.1 += 1,
            None => out.push((zi, 1)),
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
pub fn formula(m: &Mask, z: &[u32]) -> String {
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

pub fn popcount(m: &Mask) -> usize {
    m.popcount()
}

/// Every `size`-subset of the atoms whose nuclear composition equals `target`, as masks
/// over arena indices — the pool a control rate is measured over.
///
/// Enumerated by composition rather than by walking `1..2^n`: the pool is a product of
/// binomials, `prod_Z C(n_Z, k_Z)`, and that is what [`pool_size`] counts before anything
/// is allocated. `limit` bounds the enumeration; past it the pool is SAMPLED (see
/// [`pool_sample`]) and the caller says so.
pub fn pool_exact(z: &[u32], target: &[(u32, usize)]) -> Vec<Mask> {
    let groups: Vec<(Vec<usize>, usize)> = target
        .iter()
        .map(|&(zz, k)| {
            (
                (0..z.len()).filter(|&i| z[i] == zz).collect::<Vec<usize>>(),
                k,
            )
        })
        .collect();
    let mut out = Vec::new();
    let mut chosen: Vec<usize> = Vec::new();
    fn rec(
        groups: &[(Vec<usize>, usize)],
        g: usize,
        chosen: &mut Vec<usize>,
        out: &mut Vec<Mask>,
    ) {
        if g == groups.len() {
            out.push(Mask::from_members(chosen.iter().copied()));
            return;
        }
        let (members, k) = &groups[g];
        let mut idx: Vec<usize> = (0..*k).collect();
        if *k > members.len() {
            return;
        }
        loop {
            let base = chosen.len();
            for &p in &idx {
                chosen.push(members[p]);
            }
            rec(groups, g + 1, chosen, out);
            chosen.truncate(base);
            // next combination
            let mut i = *k;
            loop {
                if i == 0 {
                    return;
                }
                i -= 1;
                if idx[i] < members.len() - (*k - i) {
                    idx[i] += 1;
                    for j in (i + 1)..*k {
                        idx[j] = idx[j - 1] + 1;
                    }
                    break;
                }
            }
        }
    }
    rec(&groups, 0, &mut chosen, &mut out);
    out
}

/// `prod_Z C(n_Z, k_Z)`: how many masks [`pool_exact`] would produce, computed before any
/// of them is, saturating at `u64::MAX`.
pub fn pool_size(z: &[u32], target: &[(u32, usize)]) -> u64 {
    let mut total: u64 = 1;
    for &(zz, k) in target {
        let n = z.iter().filter(|&&x| x == zz).count();
        total = total.saturating_mul(binomial(n, k));
    }
    total
}

/// `count` distinct uniform draws from the composition pool without enumerating it —
/// each draw picks `k_Z` of the `n_Z` atoms of each species by partial Fisher–Yates.
/// Deterministic in `seed`. Returns fewer than `count` only if the pool is smaller.
pub fn pool_sample(z: &[u32], target: &[(u32, usize)], count: usize, seed: u64) -> Vec<Mask> {
    let groups: Vec<(Vec<usize>, usize)> = target
        .iter()
        .map(|&(zz, k)| ((0..z.len()).filter(|&i| z[i] == zz).collect(), k))
        .collect();
    let size = pool_size(z, target);
    let want = (count as u64).min(size) as usize;
    let mut seen = std::collections::HashSet::with_capacity(want);
    let mut out = Vec::with_capacity(want);
    let mut s = seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut next = move || {
        s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut x = s;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    };
    let mut guard = 0usize;
    while out.len() < want && guard < 64 * want + 64 {
        guard += 1;
        let mut m = Mask::empty();
        for (members, k) in &groups {
            let mut pool = members.clone();
            for t in 0..*k {
                let r = t + (next() % (pool.len() - t) as u64) as usize;
                pool.swap(t, r);
                m.insert(pool[t]);
            }
        }
        if seen.insert(m.clone()) {
            out.push(m);
        }
    }
    out
}

pub fn binomial(n: usize, k: usize) -> u64 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut r: u64 = 1;
    for i in 0..k {
        r = r.saturating_mul((n - i) as u64) / (i as u64 + 1);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits(b: u128) -> BondSet {
        BondSet::from_bits(b)
    }
    fn m(b: u128) -> Mask {
        Mask::from_bits(b)
    }

    // Three atoms; bit 0 = (0,1), bit 1 = (0,2), bit 2 = (1,2).
    #[test]
    fn no_bonds_is_all_singletons() {
        let l = labels_from_bonds(3, &bits(0));
        assert_eq!(l, vec![0, 1, 2]);
        assert_eq!(blocks(&l), vec![m(0b001), m(0b010), m(0b100)]);
    }

    #[test]
    fn one_edge_merges_exactly_two() {
        let l = labels_from_bonds(3, &bits(0b001)); // (0,1)
        assert_eq!(l, vec![0, 0, 2]);
        assert_eq!(blocks(&l), vec![m(0b011), m(0b100)]);
    }

    #[test]
    fn transitive_closure_is_one_block() {
        // (0,1) and (1,2) but NOT (0,2): still one component of three.
        let l = labels_from_bonds(3, &bits(0b101));
        assert_eq!(l, vec![0, 0, 0]);
        assert_eq!(blocks(&l), vec![m(0b111)]);
    }

    /// The canonical label must not depend on the order edges were merged in.
    ///
    /// This is the property the whole census leans on: the same partition arrived at two
    /// ways must compare EQUAL, or the closure leg would report witness pairs that are
    /// artefacts of union-find bookkeeping.
    #[test]
    fn canonical_labels_are_merge_order_independent() {
        let a = labels_from_bonds(4, &bits(0b000_011)); // (0,1),(0,2)
        let b = labels_from_bonds(4, &bits(0b001_001)); // (0,1),(1,2)
        assert_eq!(a, b);
        assert_eq!(label_bytes(&a), label_bytes(&b));
        assert_eq!(blocks(&a), vec![m(0b0111), m(0b1000)]);
    }

    #[test]
    fn label_bytes_separate_distinct_partitions() {
        let all_free = labels_from_bonds(4, &bits(0));
        let one_pair = labels_from_bonds(4, &bits(0b000_001));
        assert_ne!(label_bytes(&all_free), label_bytes(&one_pair));
    }

    #[test]
    fn formula_matches_the_quench_notation() {
        let z = [8, 1, 1, 1];
        assert_eq!(formula(&m(0b0111), &z), "OH2");
        assert_eq!(formula(&m(0b0011), &z), "OH");
        assert_eq!(formula(&m(0b0110), &z), "H2");
        assert_eq!(formula(&m(0b1110), &z), "H3");
        assert_eq!(formula(&m(0b0001), &z), "O");
        assert_eq!(composition(&m(0b0111), &z), vec![(1, 2), (8, 1)]);
    }

    /// The reader's cap is gone: a partition over more atoms than any word holds.
    #[test]
    fn a_partition_over_two_hundred_atoms_is_read_exactly() {
        let n = 200usize;
        // Bond (i, i+1) for every even i: one hundred pairs.
        let mut bs = BondSet::empty();
        for i in (0..n).step_by(2) {
            bs.insert(crate::traj::pair_index(n, i, i + 1) as u32);
        }
        let l = labels_from_bonds(n, &bs);
        let b = blocks(&l);
        assert_eq!(b.len(), 100);
        for (k, blk) in b.iter().enumerate() {
            assert_eq!(blk.popcount(), 2);
            assert_eq!(blk.least(), Some(2 * k));
            assert!(blk.contains(2 * k + 1));
        }
        let mut wide = Mask::empty();
        wide.insert(199);
        assert!(wide.contains(199) && !wide.contains(198));
        assert_eq!(wide.iter().collect::<Vec<_>>(), vec![199]);
    }

    #[test]
    fn masks_compare_by_content_not_by_word_count() {
        let a = Mask::from_bits(0b101);
        let mut b = Mask::empty();
        b.insert(200);
        b.insert(0);
        b.insert(2);
        // b has a high bit then loses nothing; a has the same low bits with fewer words
        let c = Mask::from_members([0usize, 2]);
        assert_eq!(a, c);
        assert_ne!(a, b);
        use std::collections::HashSet;
        let s: HashSet<Mask> = [a.clone(), c].into_iter().collect();
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn the_composition_pool_is_counted_then_enumerated_then_sampled() {
        // 4 O and 8 H: (1 O, 2 H) blocks number 4 * C(8,2) = 112, the census's own figure.
        let z: Vec<u32> = (0..12).map(|i| if i % 3 == 0 { 8 } else { 1 }).collect();
        let target = vec![(1u32, 2usize), (8u32, 1usize)];
        assert_eq!(pool_size(&z, &target), 112);
        let exact = pool_exact(&z, &target);
        assert_eq!(exact.len(), 112);
        let distinct: std::collections::HashSet<Mask> = exact.iter().cloned().collect();
        assert_eq!(distinct.len(), 112);
        for m in &exact {
            assert_eq!(composition(m, &z), target);
        }
        let sample = pool_sample(&z, &target, 50, 7);
        assert_eq!(sample.len(), 50);
        let ds: std::collections::HashSet<Mask> = sample.iter().cloned().collect();
        assert_eq!(ds.len(), 50);
        for m in &sample {
            assert!(distinct.contains(m), "a sample must be a member of the pool");
        }
        assert_eq!(pool_sample(&z, &target, 500, 7).len(), 112, "a sample never exceeds the pool");
    }
}
