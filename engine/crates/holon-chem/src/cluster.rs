//! Species-generic cluster machinery — the general shape whose FIRST INSTANCE is
//! `(O, H, H, H)`.
//!
//! # What this module is, and what it is not
//!
//! `quaternary.rs` grew around one cluster. Its species array was a literal
//! `[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN]`, its six pair terms were six hand-written
//! lines, its four triple terms named two specific tables, and its "nine seeded dual
//! solves" was a `for atom in 1..4`. Every one of those is a statement about a
//! four-atom cluster wearing a particular Z-tuple, and NONE of them is a statement about
//! oxygen and hydrogen: the arithmetic is identical for `(O, O, H, H)`, for
//! `(N, H, H, H)`, for anything the pair solver and the three-body surfaces cover.
//!
//! So the machinery is written ONCE here, keyed by the cluster's sorted Z-tuple, and
//! `quaternary.rs` becomes the OHHH instantiation of it. This is the tower's law applied
//! one tier up: **Z prices, Z never branches**. Nothing below chooses a code path by
//! looking at an element; `Z` selects a Species record, a surface family, and a cost.
//!
//! # What this module deliberately does NOT do
//!
//! It does not fit, extrapolate, or invent a surface for a Z-tuple nobody has tabulated.
//! [`SurfaceRegistry::family`] returns `None` for an unregistered class and
//! [`cluster_mbe3_energy`] REFUSES (returns `None`) rather than substituting a zero.
//! A missing three-body surface is a missing measurement, and a silent zero for one is
//! the defect shape this codebase has paid for before.
//!
//! # The bit-identity contract
//!
//! `tests/mbe_generic_identity.rs` holds a frozen verbatim copy of the pre-generic
//! hand-written OHHH path and compares it to this one on staked geometries with
//! `assert_eq!(a.to_bits(), b.to_bits())` — energy, all twelve gradient components, the
//! full CI vector, the iteration count and the residual. That gate is why the summation
//! ORDERS below are written out as constants instead of being left to a convenient
//! iterator: floating-point addition is commutative but not associative, so the order in
//! which six pair terms are accumulated is part of the answer, not a detail of the loop.
//!
//! Three orderings are load-bearing and each is stated where it is used:
//!
//! * [`QUAD_PAIRS`] and [`QUAD_TRIPLES`] — hub-and-cycle, not lexicographic.
//! * the LEXICOGRAPHIC pair order `[d(0,1), d(0,2), d(1,2)]` inside a three-body family.
//! * reference-atom sums GROUPED BY SPECIES, which is what makes a homonuclear pair
//!   subtract `2.0 * e` in one rounding and a heteronuclear pair subtract `e_a` then
//!   `e_b` in two — exactly as the hand-written path wrote them.

use crate::dual::D2;
use crate::elements::Species;
use crate::ooh::OohTable;
use crate::pair::{atom_energy, geometry_problem, pair_point, solve_geometry};
use crate::trimer::TrimerTable;
use crate::water::WaterTable;
use std::collections::BTreeMap;
use std::sync::{OnceLock, RwLock};

/// The floor a centre separation is clamped to, bohr. Two centres at exactly the same
/// point are a caller error, not a physical configuration; this keeps the `1/r` in the
/// nuclear repulsion finite so the refusal happens in the solver's convergence report
/// rather than as a NaN three layers up.
pub const MIN_SEP: f64 = 1e-12;

/// Euclidean distance between two centres, floored at [`MIN_SEP`].
///
/// Argument order is immaterial to the LAST BIT: the differences enter squared, and IEEE
/// negation is exact, so `d(a, b)` and `d(b, a)` are the same `f64`. The generic assembly
/// relies on that — it addresses a pair by sorted slot index while the hand-written path
/// wrote `dist(h3, h1)`.
#[inline]
pub fn center_distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt().max(MIN_SEP)
}

// ---------------------------------------------------------------- the cluster class

/// A cluster's SPECIES CLASS: its multiset of nuclear charges, stored sorted.
///
/// This is the key everything species-dependent is looked up by. It is deliberately a
/// multiset and not a tuple-with-slots: `(O, H, H)` and `(H, O, H)` are the same physical
/// class of three-body surface, and a registry keyed by slot order would hold three
/// copies of one table and let them disagree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClusterClass<const N: usize> {
    zs: [u8; N],
}

impl<const N: usize> ClusterClass<N> {
    /// The class of a Z-tuple given in any order.
    pub fn from_z(mut zs: [u8; N]) -> Self {
        zs.sort_unstable();
        Self { zs }
    }

    /// The class of a species tuple given in any order.
    pub fn of(species: &[Species; N]) -> Self {
        Self::from_z(core::array::from_fn(|i| species[i].z as u8))
    }

    /// The sorted Z-tuple.
    pub fn zs(&self) -> [u8; N] {
        self.zs
    }
}

// ------------------------------------------------------------- the three-body family

/// A tabulated (or computed) THREE-BODY surface, addressed by species class.
///
/// The three existing surface types — `(O, H, H)`, `(O, O, H)` and `(H, H, H)` — each
/// already expose `eval(..) -> (value, [f64; 3])`. This trait is an ADAPTER over them,
/// not a replacement: no implementation below re-derives an interpolant, computes a
/// chain rule, or touches a node. Each states which Z sits at which of its own argument
/// positions, and permutes.
///
/// # The one convention every implementation must meet
///
/// [`SurfaceFamily::eval_lex`] takes the three separations in LEXICOGRAPHIC pair order
/// over the family's own canonical positions — `[d(0,1), d(0,2), d(1,2)]` — and returns
/// the gradient in that same order. Two of the three types already use exactly that
/// order for their own arguments and adapt with an identity; `TrimerTable` uses the
/// cycle order `[d(0,1), d(1,2), d(2,0)]` and swaps two slots, in both directions.
pub trait SurfaceFamily: Send + Sync {
    /// The Z at each of this family's own canonical argument positions, IN ITS OWN
    /// ORDER — `[8, 1, 1]` for water, because `WaterTable::eval`'s first two arguments
    /// are the O-H sides. Not sorted; [`SurfaceFamily::class`] sorts it.
    fn canonical_z(&self) -> [u8; 3];

    /// Value and gradient, distances in lexicographic pair order over the canonical
    /// positions: `[d(0,1), d(0,2), d(1,2)]`.
    fn eval_lex(&self, d: [f64; 3]) -> (f64, [f64; 3]);

    /// The species class this family serves.
    fn class(&self) -> ClusterClass<3> {
        ClusterClass::from_z(self.canonical_z())
    }
}

impl SurfaceFamily for WaterTable {
    fn canonical_z(&self) -> [u8; 3] {
        [8, 1, 1]
    }
    /// `WaterTable::eval(r_oh1, r_oh2, r_hh)` — with O at canonical position 0, those
    /// three arguments ARE `[d(0,1), d(0,2), d(1,2)]`. Identity in both directions.
    fn eval_lex(&self, d: [f64; 3]) -> (f64, [f64; 3]) {
        self.eval(d[0], d[1], d[2])
    }
}

impl SurfaceFamily for OohTable {
    fn canonical_z(&self) -> [u8; 3] {
        [1, 8, 8]
    }
    /// `OohTable::eval(roh1, roh2, roo)` — with H at canonical position 0, the same
    /// identity as water's: the two H-O sides then the O-O side.
    fn eval_lex(&self, d: [f64; 3]) -> (f64, [f64; 3]) {
        self.eval(d[0], d[1], d[2])
    }
}

impl SurfaceFamily for TrimerTable {
    fn canonical_z(&self) -> [u8; 3] {
        [1, 1, 1]
    }
    /// `TrimerTable::eval([r_ab, r_bc, r_ca])` is the CYCLE order, so lexicographic
    /// `[d01, d02, d12]` enters as `[d01, d12, d02]` and the returned gradient's last
    /// two slots come back swapped.
    fn eval_lex(&self, d: [f64; 3]) -> (f64, [f64; 3]) {
        let (v, g) = self.eval([d[0], d[2], d[1]]);
        (v, [g[0], g[2], g[1]])
    }
}

/// The three-body surfaces available to an assembly, addressed by class.
///
/// Fixed capacity and no allocation: a registry is built per evaluation on the table
/// generator's inner loop, and an assembly that reaches for the heap once per cluster is
/// a cost the four-body path does not need to carry.
pub struct SurfaceRegistry<'a> {
    slots: [Option<(ClusterClass<3>, &'a dyn SurfaceFamily)>; SURFACE_REGISTRY_CAPACITY],
    len: usize,
}

/// How many distinct three-body classes one registry may hold. A four-cluster has four
/// triples, so four distinct classes is the ceiling for MBE4; the slack is for a caller
/// that builds one registry and reuses it across cluster shapes.
pub const SURFACE_REGISTRY_CAPACITY: usize = 8;

impl<'a> Default for SurfaceRegistry<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SurfaceRegistry<'a> {
    pub fn new() -> Self {
        Self { slots: [None; SURFACE_REGISTRY_CAPACITY], len: 0 }
    }

    /// Registers a family. Returns `false` — and registers NOTHING — if this class is
    /// already present or the registry is full. A second table for one class is an
    /// ambiguity, not an update: silently taking either one would make the assembly's
    /// answer depend on registration order, which is exactly the kind of dependence a
    /// bit-identity gate cannot see.
    pub fn insert(&mut self, family: &'a dyn SurfaceFamily) -> bool {
        let class = family.class();
        if self.len >= SURFACE_REGISTRY_CAPACITY || self.family(class).is_some() {
            return false;
        }
        self.slots[self.len] = Some((class, family));
        self.len += 1;
        true
    }

    /// Builder form. Panics on a refused insert, because a caller writing a literal
    /// registry has stated a duplicate at the source level and there is nothing to
    /// recover from.
    pub fn with(mut self, family: &'a dyn SurfaceFamily) -> Self {
        assert!(self.insert(family), "duplicate or overfull surface class in registry");
        self
    }

    /// The family serving a class, or `None`.
    pub fn family(&self, class: ClusterClass<3>) -> Option<&'a dyn SurfaceFamily> {
        self.slots[..self.len]
            .iter()
            .flatten()
            .find(|(c, _)| *c == class)
            .map(|(_, f)| *f)
    }

    /// How many families are registered.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether no family is registered.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Assigns three cluster slots to a family's canonical positions: slot `s` takes the
/// FIRST free canonical position whose Z matches. `None` if the multisets differ.
///
/// Determinism matters here even though repeated Z's look interchangeable: the two
/// hydrogens of a water triple are handed to `eval`'s first and second argument in the
/// order the CALLER listed them, and the frozen hand-written path listed them
/// `(O, H3, H1)` for one of its three triples. First-free is what reproduces that.
fn assign_positions(zs: [u8; 3], canonical: [u8; 3]) -> Option<[usize; 3]> {
    let mut used = [false; 3];
    let mut pos = [0usize; 3];
    for s in 0..3 {
        let mut found = None;
        for (p, &cz) in canonical.iter().enumerate() {
            if !used[p] && cz == zs[s] {
                found = Some(p);
                break;
            }
        }
        let p = found?;
        used[p] = true;
        pos[s] = p;
    }
    Some(pos)
}

/// Index of the pair `(a, b)`, `a < b`, in lexicographic order over three positions:
/// `(0,1) -> 0`, `(0,2) -> 1`, `(1,2) -> 2`.
#[inline]
fn lex_pair(a: usize, b: usize) -> usize {
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    match (lo, hi) {
        (0, 1) => 0,
        (0, 2) => 1,
        _ => 2,
    }
}

/// The three-body term for one triple of cluster slots, with its gradient with respect
/// to the triple's OWN three separations in lexicographic order `[d01, d02, d12]`.
///
/// `None` when no registered family serves the triple's class — a refusal, never a zero.
pub fn triple_term(
    species: [Species; 3],
    centers: [&[f64; 3]; 3],
    surfaces: &SurfaceRegistry,
) -> Option<(f64, [f64; 3])> {
    let class = ClusterClass::of(&species);
    let family = surfaces.family(class)?;
    let zs = [species[0].z as u8, species[1].z as u8, species[2].z as u8];
    let pos = assign_positions(zs, family.canonical_z())?;

    let mut d_canon = [0.0f64; 3];
    for &(s, t) in &[(0usize, 1usize), (0, 2), (1, 2)] {
        d_canon[lex_pair(pos[s], pos[t])] = center_distance(centers[s], centers[t]);
    }
    let (v, g_canon) = family.eval_lex(d_canon);

    let mut g = [0.0f64; 3];
    for &(s, t) in &[(0usize, 1usize), (0, 2), (1, 2)] {
        g[lex_pair(s, t)] = g_canon[lex_pair(pos[s], pos[t])];
    }
    Some((v, g))
}

// ------------------------------------------------------- reference-atom bookkeeping

fn atom_cache() -> &'static RwLock<BTreeMap<u32, f64>> {
    static CACHE: OnceLock<RwLock<BTreeMap<u32, f64>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(BTreeMap::new()))
}

/// The isolated-atom energy of a species, computed ONCE per Z per process.
///
/// These are constants of the level of theory. The pre-generic path memoised exactly two
/// of them in two hand-written `OnceLock`s; this is the same memo keyed by Z, so adding
/// a species costs a table entry rather than a function. `atom_energy` is a pure
/// function of the `Species` record, so the memoised value is bit-for-bit the value the
/// two `OnceLock`s held.
pub fn atom_energy_cached(sp: Species) -> f64 {
    if let Some(v) = atom_cache().read().expect("atom cache").get(&sp.z) {
        return *v;
    }
    let v = atom_energy(sp);
    atom_cache().write().expect("atom cache").insert(sp.z, v);
    v
}

/// Distinct species of a slot list as `(first slot index, multiplicity)`, in order of
/// FIRST APPEARANCE.
///
/// The grouping is not a tidiness: it is what fixes the rounding. `E - 2.0 * e` and
/// `(E - e) - e` are different `f64`s, and the hand-written path wrote the first for a
/// homonuclear pair and the second for a heteronuclear one. Grouping by species and
/// subtracting `multiplicity * e` once per group reproduces BOTH from one expression,
/// with no branch on Z.
/// The most distinct species one cluster may carry. A four-cluster can hold four; the
/// bound is stated rather than assumed because overflowing it silently would DROP a
/// reference atom and read as physics — the assembly would be short one atomic energy
/// and the four-body term would absorb it.
const MAX_DISTINCT_SPECIES: usize = 8;

fn species_groups(species: &[Species]) -> ([(usize, usize); MAX_DISTINCT_SPECIES], usize) {
    let mut out = [(0usize, 0usize); MAX_DISTINCT_SPECIES];
    let mut n = 0usize;
    for (i, sp) in species.iter().enumerate() {
        let mut hit = false;
        for g in out[..n].iter_mut() {
            if species[g.0].z == sp.z {
                g.1 += 1;
                hit = true;
                break;
            }
        }
        if !hit {
            assert!(
                n < MAX_DISTINCT_SPECIES,
                "a cluster with more than {MAX_DISTINCT_SPECIES} distinct species: raise \
                 MAX_DISTINCT_SPECIES, do not let a reference atom go unsubtracted"
            );
            out[n] = (i, 1);
            n += 1;
        }
    }
    (out, n)
}

/// The sum of isolated-atom energies over a cluster, grouped by species.
///
/// For `(O, H, H, H)` this is `e_o + 3.0 * e_h`, accumulated in that order — the first
/// group SEEDS the accumulator rather than being added to a `0.0`, because `0.0 + x` is
/// not `x` when `x` is a negative zero and a gate that compares bits would notice.
pub fn cluster_atom_energy(species: &[Species]) -> f64 {
    let (groups, n) = species_groups(species);
    let mut acc = 0.0f64;
    for (k, &(i, m)) in groups[..n].iter().enumerate() {
        let term = m as f64 * atom_energy_cached(species[i]);
        acc = if k == 0 { term } else { acc + term };
    }
    acc
}

/// The two-body EXCESS of one pair: its total energy minus its isolated atoms.
pub fn pair_excess(a: Species, b: Species, r: f64) -> f64 {
    let sp = [a, b];
    let (groups, n) = species_groups(&sp);
    let mut v = pair_point(a, b, r).e;
    for &(i, m) in groups[..n].iter() {
        v -= m as f64 * atom_energy_cached(sp[i]);
    }
    v
}

// ------------------------------------------------------------------ the FCI half

/// A cluster's exact-in-basis energy with its exact Cartesian gradient.
///
/// The FIRST slot of the gradient is not solved for: `grad[0]` is MINUS the sum of the
/// others by construction (translation invariance: `E(x + t) = E(x)` exactly), so the
/// cluster's force sum is zero to the last bit rather than to a tolerance.
pub struct ClusterFciGrad<const N: usize> {
    /// Total energy, hartree (electronic + nuclear repulsion).
    pub e: f64,
    /// `dE/d(position)`, hartree/bohr, per atom in slot order.
    pub grad: [[f64; 3]; N],
    /// The converged value-part CI vector — the warm start for the NEXT solve at a
    /// nearby geometry.
    pub ci: Vec<f64>,
    pub davidson_iters_total: usize,
    pub worst_residual: f64,
}

/// A cluster's exact-in-basis energy, value only.
pub fn cluster_fci_energy<const N: usize>(
    species: &[Species; N],
    centers: &[[f64; 3]; N],
) -> f64 {
    let dual: Vec<[D2; 3]> = (0..N)
        .map(|a| core::array::from_fn(|x| D2::c(centers[a][x])))
        .collect();
    solve_geometry(species, dual).e.v
}

/// `E_FCI` and its exact Cartesian gradient in `3 * (N - 1)` seeded dual solves.
///
/// # Why this shape
///
/// One seeded dual solve per (movable atom, axis) gives the EXACT directional derivative
/// through the same forward-mode machinery `pair_point` has always used. The value slot
/// is identical across all of them, so the first solve's CI vector warm-starts the rest
/// and the caller's cache warm-starts the first. Slot 0 is held fixed and its gradient
/// row is IMPOSED by translation invariance rather than solved for — that is where the
/// exactly-zero force sum comes from, and it is also why the count is `3(N-1)` and not
/// `3N`.
///
/// For `N = 4` this is the nine seeded dual solves the `(O, H, H, H)` path has always
/// run, in the same order, with the same warm-start chain.
pub fn cluster_fci_grad<const N: usize>(
    species: &[Species; N],
    centers: &[[f64; 3]; N],
    warm: Option<&[f64]>,
) -> ClusterFciGrad<N> {
    assert!(N >= 2, "a cluster gradient needs at least two centres, got {N}");
    let mut grad = [[0.0f64; 3]; N];
    let mut e = 0.0f64;
    let mut ci: Vec<f64> = Vec::new();
    let mut iters = 0usize;
    let mut worst = 0.0f64;
    let mut start: Option<Vec<f64>> = warm.map(|w| w.to_vec());
    for atom in 1..N {
        for axis in 0..3usize {
            let dual: Vec<[D2; 3]> = (0..N)
                .map(|a| {
                    core::array::from_fn(|x| {
                        if a == atom && x == axis {
                            D2::var(centers[a][x])
                        } else {
                            D2::c(centers[a][x])
                        }
                    })
                })
                .collect();
            let (space, mo, nuc) = geometry_problem(species, dual);
            let sol = crate::fci::solve_determinant_from(&space, &mo, start.as_deref());
            let tot = sol.e + nuc;
            grad[atom][axis] = tot.d;
            if atom == 1 && axis == 0 {
                e = tot.v;
                ci = sol.vector.clone();
            }
            iters += sol.davidson_iters;
            worst = worst.max(sol.residual);
            start = Some(sol.vector);
        }
    }
    // Translation invariance. The fold SEEDS on slot 1 rather than on a `0.0` so the
    // association is `-(((g1 + g2) + g3) ...)`, left to right — which is what the
    // hand-written three-term expression compiled to, and `0.0 + x` differs from `x`
    // in bits when `x` is a negative zero.
    for x in 0..3 {
        let mut s = grad[1][x];
        for a in 2..N {
            s += grad[a][x];
        }
        grad[0][x] = -s;
    }
    ClusterFciGrad { e, grad, ci, davidson_iters_total: iters, worst_residual: worst }
}

// ------------------------------------------------------------- the MBE assembly

/// The six pairs of a four-cluster, IN THE ORDER THEY ARE SUMMED: the star from slot 0,
/// then the 3-cycle on the remaining slots.
///
/// Hub-and-cycle, not lexicographic — `(2,3)` precedes `(3,1)`. That is a convention
/// about a four-cluster and carries no species information, but it IS part of the
/// answer: reordering a six-term floating-point sum moves the last bits, and the
/// hand-written OHHH path summed in this order.
pub const QUAD_PAIRS: [(usize, usize); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (2, 3), (3, 1)];

/// The four triples of a four-cluster, in summation order: the three containing slot 0,
/// each carrying one edge of the cycle, then the cycle itself. Same convention, same
/// reason, as [`QUAD_PAIRS`].
pub const QUAD_TRIPLES: [(usize, usize, usize); 4] =
    [(0, 1, 2), (0, 2, 3), (0, 3, 1), (1, 2, 3)];

/// `E_MBE3` for a general four-cluster: isolated atoms + six pair excesses + four
/// three-body terms.
///
/// Returns `None` if any triple's species class has no registered surface. A four-body
/// term is a DIFFERENCE of this against the exact energy, so a missing three-body
/// surface silently read as zero would be laundered into the four-body term as physics.
pub fn cluster_mbe3_energy(
    species: &[Species; 4],
    centers: &[[f64; 3]; 4],
    surfaces: &SurfaceRegistry,
) -> Option<f64> {
    let atoms = cluster_atom_energy(species);

    let mut pairs = 0.0f64;
    for (k, &(i, j)) in QUAD_PAIRS.iter().enumerate() {
        let r = center_distance(&centers[i], &centers[j]);
        let term = pair_excess(species[i], species[j], r);
        pairs = if k == 0 { term } else { pairs + term };
    }

    let mut triples = 0.0f64;
    for (k, &(i, j, l)) in QUAD_TRIPLES.iter().enumerate() {
        let (term, _) = triple_term(
            [species[i], species[j], species[l]],
            [&centers[i], &centers[j], &centers[l]],
            surfaces,
        )?;
        triples = if k == 0 { term } else { triples + term };
    }

    Some(atoms + pairs + triples)
}

/// The exact four-body term of a general four-cluster: `E_FCI - E_MBE3`.
pub fn cluster_de4(
    species: &[Species; 4],
    centers: &[[f64; 3]; 4],
    surfaces: &SurfaceRegistry,
) -> Option<f64> {
    let mbe3 = cluster_mbe3_energy(species, centers, surfaces)?;
    Some(cluster_fci_energy(species, centers) - mbe3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{HYDROGEN, OXYGEN};

    #[test]
    fn class_is_a_sorted_multiset() {
        let a = ClusterClass::of(&[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN]);
        let b = ClusterClass::of(&[HYDROGEN, HYDROGEN, OXYGEN, HYDROGEN]);
        assert_eq!(a, b);
        assert_eq!(a.zs(), [1, 1, 1, 8]);
    }

    #[test]
    fn registry_refuses_a_second_table_for_one_class() {
        let w1 = WaterTable::default();
        let w2 = WaterTable::default();
        let mut r = SurfaceRegistry::new();
        assert!(r.insert(&w1));
        assert!(!r.insert(&w2), "a duplicate class must be refused, not overwritten");
        assert_eq!(r.len(), 1);
        assert!(!r.is_empty());
        assert!(r.family(ClusterClass::from_z([1, 1, 8])).is_some());
        assert!(r.family(ClusterClass::from_z([1, 1, 1])).is_none());
    }

    #[test]
    fn positions_are_assigned_first_free_match() {
        // (O, H3, H1) against water's canonical [8, 1, 1]: O to 0, then the hydrogens
        // in the order the caller listed them.
        assert_eq!(assign_positions([8, 1, 1], [8, 1, 1]), Some([0, 1, 2]));
        // (H, O, H) against the same canonical order.
        assert_eq!(assign_positions([1, 8, 1], [8, 1, 1]), Some([1, 0, 2]));
        // A class mismatch is a refusal.
        assert_eq!(assign_positions([1, 1, 1], [8, 1, 1]), None);
    }

    #[test]
    fn an_unregistered_class_is_refused_not_zeroed() {
        let empty = SurfaceRegistry::new();
        let got = triple_term(
            [OXYGEN, HYDROGEN, HYDROGEN],
            [&[0.0, 0.0, 0.0], &[1.8, 0.0, 0.0], &[-0.5, 1.7, 0.0]],
            &empty,
        );
        assert!(got.is_none());
    }

    #[test]
    fn grouping_reproduces_both_reference_forms() {
        // The point of the grouping: one expression, two roundings.
        let (g, n) = species_groups(&[OXYGEN, HYDROGEN]);
        assert_eq!(n, 2);
        assert_eq!((g[0].1, g[1].1), (1, 1));
        let (g, n) = species_groups(&[HYDROGEN, HYDROGEN]);
        assert_eq!(n, 1);
        assert_eq!(g[0].1, 2);
        let (g, n) = species_groups(&[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN]);
        assert_eq!(n, 2);
        assert_eq!((g[0].1, g[1].1), (1, 3));
    }

    #[test]
    fn lexicographic_pair_index_is_a_bijection() {
        let mut seen = [false; 3];
        for &(a, b) in &[(0usize, 1usize), (0, 2), (1, 2)] {
            assert_eq!(lex_pair(a, b), lex_pair(b, a));
            seen[lex_pair(a, b)] = true;
        }
        assert!(seen.iter().all(|s| *s));
    }
}
