//! Species-generic, ORDER-generic cluster machinery — the many-body expansion of a
//! cluster of any arity, whose first instance was `(O, H, H, H)`.
//!
//! # What this module is, and what it is not
//!
//! `quaternary.rs` grew around one cluster. Its species array was a literal
//! `[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN]`, its six pair terms were six hand-written
//! lines, its four triple terms named two specific tables, and its "nine seeded dual
//! solves" was a `for atom in 1..4`. Every one of those is a statement about a
//! four-atom cluster wearing a particular Z-tuple, and NONE of them is a statement about
//! oxygen and hydrogen — or about FOUR: the arithmetic is identical for `(O, O, H, H)`,
//! for `(N, H, H, H)`, for a five-cluster, for anything the pair solver and the
//! three-body surfaces cover. So the machinery is written ONCE here, keyed by the
//! cluster's sorted Z-tuple and parameterised by its arity, and `quaternary.rs` is gone
//! (its table lives on in `quaternary_table.rs`, with the OHHH tuple as one constant).
//! This is the tower's law applied one tier up: **Z prices, Z never branches** — and
//! neither does the atom count. Nothing below chooses a code path by looking at an
//! element or at `N`; `Z` selects a Species record, a surface family and a cost, and `N`
//! selects how many seeded solves a gradient costs.
//!
//! # The expansion, stated once
//!
//! For a cluster `S` of `N` atoms, `E_MBE_k(S)` is the isolated atoms plus every pair
//! excess plus every tabulated three-body term plus, for `4 <= m <= k`, every `m`-subset's
//! own body term `eps_m(T) = E_FCI(T) - E_MBE_{m-1}(T)` — the connected term, computed
//! recursively and exactly. The cluster's own body term is `eps_N(S) = E_FCI(S) -
//! E_MBE_{N-1}(S)`; at `N = 4` that is the `dE4` every certified table was built from,
//! and the engine's many-body sector evaluates it live with its exact gradient.
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
//! * [`pair_order`] and [`triple_order`] — the star from slot 0, then the cycle, then the
//!   chords; and the triples through slot 0 around the cycle, then the rest. At `N = 4`
//!   these are exactly the banked [`QUAD_PAIRS`] and [`QUAD_TRIPLES`], hub-and-cycle,
//!   pinned by a test; at any other `N` they are the same rule and not a special case.
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

/// A cluster's SPECIES CLASS: its multiset of nuclear charges, stored sorted, of any
/// arity.
///
/// This is the key everything species-dependent is looked up by. It is deliberately a
/// multiset and not a tuple-with-slots: `(O, H, H)` and `(H, O, H)` are the same physical
/// class of three-body surface, and a registry keyed by slot order would hold three
/// copies of one table and let them disagree.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClusterClass {
    zs: Vec<u8>,
}

impl ClusterClass {
    /// The class of a Z-tuple given in any order.
    pub fn from_z(zs: &[u8]) -> Self {
        let mut zs = zs.to_vec();
        zs.sort_unstable();
        Self { zs }
    }

    /// The class of a species tuple given in any order.
    pub fn of(species: &[Species]) -> Self {
        let zs: Vec<u8> = species.iter().map(|s| s.z as u8).collect();
        Self::from_z(&zs)
    }

    /// The sorted Z-tuple.
    pub fn zs(&self) -> &[u8] {
        &self.zs
    }

    /// How many atoms the class describes.
    pub fn arity(&self) -> usize {
        self.zs.len()
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

    /// How far the surface reaches, bohr: past this every term it serves is an exact
    /// zero. The tables' own `R_HI`.
    fn reach(&self) -> f64;

    fn class(&self) -> ClusterClass {
        ClusterClass::from_z(&self.canonical_z())
    }
}

impl SurfaceFamily for WaterTable {
    fn canonical_z(&self) -> [u8; 3] {
        [8, 1, 1]
    }
    fn eval_lex(&self, d: [f64; 3]) -> (f64, [f64; 3]) {
        self.eval(d[0], d[1], d[2])
    }
    fn reach(&self) -> f64 {
        crate::water::R_HI
    }
}

impl SurfaceFamily for OohTable {
    fn canonical_z(&self) -> [u8; 3] {
        [1, 8, 8]
    }
    fn eval_lex(&self, d: [f64; 3]) -> (f64, [f64; 3]) {
        self.eval(d[0], d[1], d[2])
    }
    fn reach(&self) -> f64 {
        crate::ooh::R_HI
    }
}

impl SurfaceFamily for TrimerTable {
    fn canonical_z(&self) -> [u8; 3] {
        [1, 1, 1]
    }
    fn eval_lex(&self, d: [f64; 3]) -> (f64, [f64; 3]) {
        let (v, g) = self.eval([d[0], d[2], d[1]]);
        (v, [g[0], g[2], g[1]])
    }
    fn reach(&self) -> f64 {
        crate::trimer::R_HI
    }
}

/// The three-body surfaces a cluster evaluation may draw on, keyed by class, plus the
/// MEASURED reach of every body order above three that has one.
///
/// Refuses a second family for a class it already holds rather than overwriting: two
/// tables for one class is two answers, and the registry's job is to have one. There is
/// no capacity — the old fixed-slot array was a cap wearing a struct.
///
/// A body reach is a measurement about a CLASS at an ORDER — where `|eps_m|` for that
/// class falls below the tables' tolerance — and it is what a many-body sector's cutoff
/// is derived from. A class with no declared reach at an order is refused at that order
/// by name, never given a radius somebody guessed.
#[derive(Default)]
pub struct SurfaceRegistry<'a> {
    families: Vec<(ClusterClass, &'a dyn SurfaceFamily)>,
    body_reach: Vec<(ClusterClass, f64, &'static str)>,
}

impl<'a> SurfaceRegistry<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a family; `false` if its class is already served (nothing is replaced).
    pub fn insert(&mut self, family: &'a dyn SurfaceFamily) -> bool {
        let class = family.class();
        if self.family(&class).is_some() {
            return false;
        }
        self.families.push((class, family));
        true
    }

    pub fn with(mut self, family: &'a dyn SurfaceFamily) -> Self {
        assert!(self.insert(family), "duplicate surface class in registry");
        self
    }

    /// Declare the measured reach of body order `class.arity()` for `class`, with the
    /// record that measured it.
    pub fn declare_reach(&mut self, class: ClusterClass, reach: f64, provenance: &'static str) -> bool {
        if self.reach_of(&class).is_some() {
            return false;
        }
        self.body_reach.push((class, reach, provenance));
        true
    }

    pub fn family(&self, class: &ClusterClass) -> Option<&'a dyn SurfaceFamily> {
        self.families.iter().find(|(c, _)| c == class).map(|(_, f)| *f)
    }

    /// The declared reach of `class` at its own order, and where it was measured.
    pub fn reach_of(&self, class: &ClusterClass) -> Option<(f64, &'static str)> {
        self.body_reach.iter().find(|(c, _, _)| c == class).map(|(_, r, p)| (*r, *p))
    }

    /// The largest declared reach at body order `order`, or `None` when no class has one:
    /// the radius a many-body sector at that order enumerates to.
    pub fn max_reach_at(&self, order: usize) -> Option<f64> {
        self.body_reach
            .iter()
            .filter(|(c, _, _)| c.arity() == order)
            .map(|(_, r, _)| *r)
            .fold(None, |m, r| Some(m.map_or(r, |x: f64| x.max(r))))
    }

    /// Whether every three-body class inside `class` is served by a registered family —
    /// the condition under which an expansion of `class` can be assembled at all.
    pub fn family_covers(&self, class: &ClusterClass) -> bool {
        let zs = class.zs();
        for (i, j, k) in triple_order(zs.len()) {
            if self.family(&ClusterClass::from_z(&[zs[i], zs[j], zs[k]])).is_none() {
                return false;
            }
        }
        true
    }

    pub fn len(&self) -> usize {
        self.families.len()
    }

    pub fn is_empty(&self) -> bool {
        self.families.is_empty()
    }
}

/// Every body reach this crate has MEASURED, with the record that measured it: the
/// classes an engine's many-body sector may enumerate at their orders. One entry today —
/// the `(O,H,H,H)` far field behind `quaternary_table::R_HI`. A second is a second
/// measurement, never a second guess.
pub fn measured_body_reaches() -> Vec<(ClusterClass, f64, &'static str)> {
    vec![(
        ClusterClass::of(&crate::quaternary_table::OHHH),
        crate::quaternary_table::R_HI,
        crate::quaternary_table::OHHH_REACH_PROVENANCE,
    )]
}

/// Which canonical position each of the three slots of `zs` takes in `canonical`:
/// first free match, in slot order. `None` on a class mismatch.
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
    let family = surfaces.family(&class)?;
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
/// Grouping is what reproduces the two reference-atom forms the hand-written paths
/// used: a homonuclear pair subtracted `2.0 * e` in ONE rounding, a heteronuclear pair
/// subtracted `e_a` and then `e_b` in two. `m as f64 * e` with `m = 1` is `e` exactly,
/// so the heteronuclear case is unchanged by passing through the multiplication.
fn species_groups(species: &[Species]) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for (i, sp) in species.iter().enumerate() {
        match out.iter_mut().find(|g| species[g.0].z == sp.z) {
            Some(g) => g.1 += 1,
            None => out.push((i, 1)),
        }
    }
    out
}

/// The isolated-atom reference of a cluster: each species' energy times its
/// multiplicity, grouped by species in order of first appearance, summed left to right
/// with the first term seeding the sum.
pub fn cluster_atom_energy(species: &[Species]) -> f64 {
    let groups = species_groups(species);
    let mut acc = 0.0f64;
    for (k, &(i, m)) in groups.iter().enumerate() {
        let term = m as f64 * atom_energy_cached(species[i]);
        acc = if k == 0 { term } else { acc + term };
    }
    acc
}

/// `E(pair at r) - E(a) - E(b)`, the pair's excess over its dissociated atoms, solved
/// ab initio at `r`. The reference is grouped by species (see [`species_groups`]).
pub fn pair_excess(a: Species, b: Species, r: f64) -> f64 {
    let sp = [a, b];
    let groups = species_groups(&sp);
    let mut v = pair_point(a, b, r).e;
    for &(i, m) in groups.iter() {
        v -= m as f64 * atom_energy_cached(sp[i]);
    }
    v
}

/// Where a cluster evaluation gets its PAIR excesses: value, and slope when a gradient is
/// wanted. The chemistry crate's own source solves each pair ab initio; the engine's
/// source reads its bank's Hermite curves, whose zero IS the dissociated asymptote, so the
/// two are the same quantity from two ledgers.
pub trait PairSource {
    fn excess(&self, a: Species, b: Species, r: f64) -> f64;
    /// `d/dr` of [`PairSource::excess`].
    fn excess_slope(&self, a: Species, b: Species, r: f64) -> f64;
}

/// Pairs solved ab initio at every separation: the reference source.
pub struct AbInitioPairs;

impl PairSource for AbInitioPairs {
    fn excess(&self, a: Species, b: Species, r: f64) -> f64 {
        pair_excess(a, b, r)
    }
    fn excess_slope(&self, a: Species, b: Species, r: f64) -> f64 {
        // The pair lies along z; the excess's slope is the diatomic's dE/dz on the
        // second centre, the atoms' energies being constants.
        let sol = solve_geometry(
            &[a, b],
            vec![
                [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                [D2::c(0.0), D2::c(0.0), D2::var(r)],
            ],
        );
        sol.e.d
    }
}

// ------------------------------------------------------------------ the FCI half

/// A cluster's exact-in-basis energy with its exact Cartesian gradient.
///
/// The FIRST slot of the gradient is not solved for: `grad[0]` is MINUS the sum of the
/// others by construction (translation invariance: `E(x + t) = E(x)` exactly), so the
/// cluster's force sum is zero to the last bit rather than to a tolerance.
pub struct ClusterFciGrad {
    /// Total energy, hartree (electronic + nuclear repulsion).
    pub e: f64,
    /// `dE/d(position)`, hartree/bohr, per atom in slot order.
    pub grad: Vec<[f64; 3]>,
    /// The converged value-part CI vector — the warm start for the NEXT solve at a
    /// nearby geometry.
    pub ci: Vec<f64>,
    pub davidson_iters_total: usize,
    pub worst_residual: f64,
}

/// A cluster's exact-in-basis energy, value only.
pub fn cluster_fci_energy(species: &[Species], centers: &[[f64; 3]]) -> f64 {
    let dual: Vec<[D2; 3]> = centers
        .iter()
        .map(|c| core::array::from_fn(|x| D2::c(c[x])))
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
pub fn cluster_fci_grad(species: &[Species], centers: &[[f64; 3]], warm: Option<&[f64]>) -> ClusterFciGrad {
    let n = species.len();
    assert_eq!(n, centers.len(), "one centre per species");
    assert!(n >= 2, "a cluster gradient needs at least two centres, got {n}");
    let mut grad = vec![[0.0f64; 3]; n];
    let mut e = 0.0f64;
    let mut ci: Vec<f64> = Vec::new();
    let mut iters = 0usize;
    let mut worst = 0.0f64;
    let mut start: Option<Vec<f64>> = warm.map(|w| w.to_vec());
    for atom in 1..n {
        for axis in 0..3usize {
            let dual: Vec<[D2; 3]> = (0..n)
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
        for a in 2..n {
            s += grad[a][x];
        }
        grad[0][x] = -s;
    }
    ClusterFciGrad { e, grad, ci, davidson_iters_total: iters, worst_residual: worst }
}

// ------------------------------------------------------------- the MBE assembly

/// The six pairs of a four-cluster, IN THE ORDER THEY ARE SUMMED: the star from slot 0,
/// then the 3-cycle on the remaining slots. The `N = 4` instance of [`pair_order`],
/// kept as the banked convention the identity test names.
pub const QUAD_PAIRS: [(usize, usize); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (2, 3), (3, 1)];

/// The four triples of a four-cluster, in summation order: the three containing slot 0,
/// each carrying one edge of the cycle, then the cycle itself. The `N = 4` instance of
/// [`triple_order`].
pub const QUAD_TRIPLES: [(usize, usize, usize); 4] =
    [(0, 1, 2), (0, 2, 3), (0, 3, 1), (1, 2, 3)];

/// The pairs of an `n`-cluster in summation order: the star from slot 0 in slot order,
/// then the CYCLE over the remaining slots `(1,2), (2,3), .., (n-1, 1)`, then the chords
/// among them lexicographically. Hub-and-cycle at `n = 4` — [`QUAD_PAIRS`] exactly — and
/// the same rule at every other `n`. Floating-point addition is not associative, so this
/// order is part of the answer and is stated rather than left to an iterator.
pub fn pair_order(n: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(n * n.saturating_sub(1) / 2);
    for j in 1..n {
        out.push((0, j));
    }
    let m = n.saturating_sub(1);
    if m >= 3 {
        for k in 1..n {
            let next = if k + 1 < n { k + 1 } else { 1 };
            out.push((k, next));
        }
        for a in 1..n {
            for b in (a + 1)..n {
                let cycle_edge = b == a + 1 || (a == 1 && b == n - 1);
                if !cycle_edge {
                    out.push((a, b));
                }
            }
        }
    } else if m == 2 {
        out.push((1, 2));
    }
    out
}

/// The triples of an `n`-cluster in summation order: those through slot 0 taken around
/// the cycle `(0, k, k+1)`, then the remaining triples through slot 0 lexicographically,
/// then the triples without slot 0 lexicographically. [`QUAD_TRIPLES`] at `n = 4`.
pub fn triple_order(n: usize) -> Vec<(usize, usize, usize)> {
    let mut out = Vec::new();
    if n < 3 {
        return out;
    }
    let mut seen = std::collections::HashSet::new();
    if n - 1 >= 3 {
        for k in 1..n {
            let next = if k + 1 < n { k + 1 } else { 1 };
            let t = (0, k, next);
            if seen.insert((k.min(next), k.max(next))) {
                out.push(t);
            }
        }
    }
    for a in 1..n {
        for b in (a + 1)..n {
            if seen.insert((a, b)) {
                out.push((0, a, b));
            }
        }
    }
    for a in 1..n {
        for b in (a + 1)..n {
            for c in (b + 1)..n {
                out.push((a, b, c));
            }
        }
    }
    out
}

/// Every `k`-subset of `0..n`, lexicographically.
pub fn subsets_lex(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    if k > n {
        return out;
    }
    let mut idx: Vec<usize> = (0..k).collect();
    loop {
        out.push(idx.clone());
        let mut i = k;
        loop {
            if i == 0 {
                return out;
            }
            i -= 1;
            if idx[i] < n - (k - i) {
                idx[i] += 1;
                for j in (i + 1)..k {
                    idx[j] = idx[j - 1] + 1;
                }
                break;
            }
        }
    }
}

/// One pair term of an assembly: the slots, the separation, the excess and its slope
/// (`0.0` when no gradient was asked for).
#[derive(Clone, Debug)]
pub struct PairTermOut {
    pub slots: (usize, usize),
    pub r: f64,
    pub v: f64,
    pub dv: f64,
}

/// One three-body term: the slots, the value, and the gradient with respect to the
/// triple's own separations in lexicographic slot order `[(a,b), (a,c), (b,c)]`.
#[derive(Clone, Debug)]
pub struct TripleTermOut {
    pub slots: (usize, usize, usize),
    pub v: f64,
    pub g: [f64; 3],
}

/// One body term of order four or more: the subcluster's slots, its connected term, and
/// (when asked) its Cartesian gradient per slot.
#[derive(Clone, Debug)]
pub struct BodyTermOut {
    pub slots: Vec<usize>,
    pub v: f64,
    pub grad: Vec<[f64; 3]>,
}

/// The terms of `E_MBE_order` for a cluster, in their stated orders, NOT yet summed.
///
/// Two consumers fold these with two banked associations — the chemistry crate's
/// `atoms + (pairs) + (triples) + (higher)` with each group seeded on its first term
/// ([`MbeTerms::energy`]), and the engine's single running sum in term order — so the
/// terms are handed over and the fold is the consumer's convention.
#[derive(Clone, Debug)]
pub struct MbeTerms {
    pub atoms: f64,
    pub pairs: Vec<PairTermOut>,
    pub triples: Vec<TripleTermOut>,
    pub higher: Vec<BodyTermOut>,
}

impl MbeTerms {
    /// The chemistry crate's fold: each group summed left to right seeded on its first
    /// term, then `atoms + pairs + triples (+ higher)`.
    pub fn energy(&self) -> f64 {
        let mut e = self.atoms + seeded_sum(self.pairs.iter().map(|p| p.v));
        if !self.triples.is_empty() {
            e += seeded_sum(self.triples.iter().map(|t| t.v));
        }
        if !self.higher.is_empty() {
            e += seeded_sum(self.higher.iter().map(|h| h.v));
        }
        e
    }

    /// Add this assembly's gradient to `grad` (Cartesian, per slot), every term
    /// distributed pairwise along the unit vector between its two centres — equal and
    /// opposite in the same bits, which is what keeps a cluster's force sum exactly zero.
    ///
    /// Shares are added in term order. A triple's three shares go in its slot order
    /// `(a,b), (a,c), (b,c)` when it contains slot 0 and in CYCLIC order
    /// `(a,b), (b,c), (c,a)` when it does not: the two conventions the banked four-body
    /// sector applied to its hub triples and its cycle triple respectively, stated once.
    pub fn add_gradient(&self, centers: &[[f64; 3]], grad: &mut [[f64; 3]]) {
        for p in &self.pairs {
            add_pair_share(grad, centers, p.slots.0, p.slots.1, p.r, p.dv);
        }
        for t in &self.triples {
            let (a, b, c) = t.slots;
            let d = |x: usize, y: usize| center_distance(&centers[x], &centers[y]);
            if a == 0 || b == 0 || c == 0 {
                add_pair_share(grad, centers, a, b, d(a, b), t.g[0]);
                add_pair_share(grad, centers, a, c, d(a, c), t.g[1]);
                add_pair_share(grad, centers, b, c, d(b, c), t.g[2]);
            } else {
                add_pair_share(grad, centers, a, b, d(a, b), t.g[0]);
                add_pair_share(grad, centers, b, c, d(b, c), t.g[2]);
                add_pair_share(grad, centers, c, a, d(c, a), t.g[1]);
            }
        }
        for h in &self.higher {
            for (k, &slot) in h.slots.iter().enumerate() {
                for x in 0..3 {
                    grad[slot][x] += h.grad[k][x];
                }
            }
        }
    }
}

fn seeded_sum(mut it: impl Iterator<Item = f64>) -> f64 {
    let Some(first) = it.next() else { return 0.0 };
    it.fold(first, |acc, v| acc + v)
}

/// One pairwise share of a scalar potential's gradient: `dv` is dV/dr for the pair
/// `(a, b)` at separation `r`; the contribution is `dv` along the unit vector from `a`
/// to `b`, equal and opposite.
#[inline]
pub fn add_pair_share(g: &mut [[f64; 3]], p: &[[f64; 3]], a: usize, b: usize, r: f64, dv: f64) {
    let rr = r.max(MIN_SEP);
    for x in 0..3 {
        let u = (p[b][x] - p[a][x]) / rr;
        g[b][x] += dv * u;
        g[a][x] -= dv * u;
    }
}

/// The terms of `E_MBE_order` for `species` at `centers`.
///
/// `None` if any triple's class has no registered surface, or any higher subcluster's
/// own expansion refuses: a body term is a DIFFERENCE against the exact energy, so a
/// missing surface silently read as zero would be laundered into the term as physics.
/// `order` is clamped to `N - 1`: the cluster's own body term is not a term of its own
/// expansion.
pub fn mbe_terms(
    order: usize,
    species: &[Species],
    centers: &[[f64; 3]],
    pairs: &dyn PairSource,
    surfaces: &SurfaceRegistry,
    want_grad: bool,
) -> Option<MbeTerms> {
    let n = species.len();
    assert_eq!(n, centers.len(), "one centre per species");
    let order = order.min(n.saturating_sub(1));
    let atoms = cluster_atom_energy(species);
    let mut pair_terms = Vec::new();
    if order >= 2 {
        for (i, j) in pair_order(n) {
            let r = center_distance(&centers[i], &centers[j]);
            let v = pairs.excess(species[i], species[j], r);
            let dv = if want_grad { pairs.excess_slope(species[i], species[j], r) } else { 0.0 };
            pair_terms.push(PairTermOut { slots: (i, j), r, v, dv });
        }
    }
    let mut triple_terms = Vec::new();
    if order >= 3 {
        for (i, j, l) in triple_order(n) {
            let (v, g) = triple_term(
                [species[i], species[j], species[l]],
                [&centers[i], &centers[j], &centers[l]],
                surfaces,
            )?;
            triple_terms.push(TripleTermOut { slots: (i, j, l), v, g });
        }
    }
    let mut higher = Vec::new();
    for m in 4..=order {
        for slots in subsets_lex(n, m) {
            let sub_species: Vec<Species> = slots.iter().map(|&s| species[s]).collect();
            let sub_centers: Vec<[f64; 3]> = slots.iter().map(|&s| centers[s]).collect();
            if want_grad {
                let bt = body_term_grad(&sub_species, &sub_centers, pairs, surfaces, None)?;
                higher.push(BodyTermOut { slots, v: bt.v, grad: bt.grad });
            } else {
                let v = body_term(&sub_species, &sub_centers, pairs, surfaces)?;
                higher.push(BodyTermOut { slots, v, grad: Vec::new() });
            }
        }
    }
    Some(MbeTerms { atoms, pairs: pair_terms, triples: triple_terms, higher })
}

/// `E_MBE_order` of a cluster, value only, in the chemistry crate's fold.
pub fn mbe_energy(
    order: usize,
    species: &[Species],
    centers: &[[f64; 3]],
    pairs: &dyn PairSource,
    surfaces: &SurfaceRegistry,
) -> Option<f64> {
    Some(mbe_terms(order, species, centers, pairs, surfaces, false)?.energy())
}

/// The cluster's own connected term, `eps_N = E_FCI - E_MBE_{N-1}`, value only. At
/// `N = 4` this is `dE4`.
pub fn body_term(
    species: &[Species],
    centers: &[[f64; 3]],
    pairs: &dyn PairSource,
    surfaces: &SurfaceRegistry,
) -> Option<f64> {
    let mbe = mbe_energy(species.len().saturating_sub(1), species, centers, pairs, surfaces)?;
    Some(cluster_fci_energy(species, centers) - mbe)
}

/// A body term with its exact Cartesian gradient: the FCI gradient minus the assembled
/// gradient of the lower-order expansion, per slot.
pub struct BodyTermGrad {
    pub v: f64,
    pub grad: Vec<[f64; 3]>,
    pub fci: ClusterFciGrad,
    pub mbe: MbeTerms,
}

/// `eps_N` with its gradient. `warm` seeds the FCI solve.
pub fn body_term_grad(
    species: &[Species],
    centers: &[[f64; 3]],
    pairs: &dyn PairSource,
    surfaces: &SurfaceRegistry,
    warm: Option<&[f64]>,
) -> Option<BodyTermGrad> {
    let n = species.len();
    let mbe = mbe_terms(n.saturating_sub(1), species, centers, pairs, surfaces, true)?;
    let fci = cluster_fci_grad(species, centers, warm);
    let mut gm = vec![[0.0f64; 3]; n];
    mbe.add_gradient(centers, &mut gm);
    let grad: Vec<[f64; 3]> = (0..n)
        .map(|a| core::array::from_fn(|x| fci.grad[a][x] - gm[a][x]))
        .collect();
    let v = fci.e - mbe.energy();
    Some(BodyTermGrad { v, grad, fci, mbe })
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
        assert!(r.family(&ClusterClass::from_z(&[1, 1, 8])).is_some());
        assert!(r.family(&ClusterClass::from_z(&[1, 1, 1])).is_none());
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
        let g = species_groups(&[OXYGEN, HYDROGEN]);
        let n = g.len();
        assert_eq!(n, 2);
        assert_eq!((g[0].1, g[1].1), (1, 1));
        let g = species_groups(&[HYDROGEN, HYDROGEN]);
        let n = g.len();
        assert_eq!(n, 1);
        assert_eq!(g[0].1, 2);
        let g = species_groups(&[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN]);
        let n = g.len();
        assert_eq!(n, 2);
        assert_eq!((g[0].1, g[1].1), (1, 3));
    }

    /// The stated orders reproduce the banked four-cluster convention exactly.
    #[test]
    fn the_orders_at_four_are_hub_and_cycle() {
        assert_eq!(pair_order(4), QUAD_PAIRS.to_vec());
        assert_eq!(triple_order(4), QUAD_TRIPLES.to_vec());
        // and at five they are the same rule, not a special case
        assert_eq!(
            pair_order(5),
            vec![(0, 1), (0, 2), (0, 3), (0, 4), (1, 2), (2, 3), (3, 4), (4, 1), (1, 3), (2, 4)]
        );
        assert_eq!(
            triple_order(5),
            vec![(0, 1, 2), (0, 2, 3), (0, 3, 4), (0, 4, 1), (0, 1, 3), (0, 2, 4), (1, 2, 3), (1, 2, 4), (1, 3, 4), (2, 3, 4)]
        );
        assert_eq!(pair_order(3), vec![(0, 1), (0, 2), (1, 2)]);
        assert_eq!(triple_order(3), vec![(0, 1, 2)]);
        assert_eq!(subsets_lex(5, 4), vec![vec![0, 1, 2, 3], vec![0, 1, 2, 4], vec![0, 1, 3, 4], vec![0, 2, 3, 4], vec![1, 2, 3, 4]]);
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
