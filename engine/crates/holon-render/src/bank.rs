//! THE PAIR-TABLE BANK: one curve per unordered species pair, and the provenance that
//! says what each curve is worth.
//!
//! # What this replaces, and what it must not cost
//!
//! The sandbox held ONE [`PotentialTable`]. Atoms already carried a species and a mass —
//! so a scene could be drawn with a chlorine and a hydrogen in it, and both pairs, and the
//! H-H pair too, were served the same curve. The bank removes that: the force loop, the
//! bond criterion, the drift bound and the ledger each dispatch on the pair in front of
//! them.
//!
//! MIXTURES-1's gate B1 is the fence around the change: an all-hydrogen scene through the
//! bank must be BIT-FOR-BIT the scene the single table produced. That is why the bank is
//! a lookup returning `&PotentialTable` and not a new evaluator — every float still comes
//! out of the same Hermite coefficients through the same `eval`, in the same order. A bank
//! holding one table is that table.
//!
//! # Why the cap is three species, and how the number was arrived at
//!
//! Each table is a fixed array of [`crate::table::MAX_KNOTS`] knots — four `f64` columns,
//! 32,872 bytes — which is what keeps the core allocation-free and the wasm free of an
//! allocator. The bank is `S*(S+1)/2` of them, and a `Sim` is constructed BY VALUE by
//! every test, example and shell in this workspace.
//!
//! The cap is therefore a STACK budget, and it was measured, twice, by overflowing it:
//!
//! | `MAX_SPECIES` | tables | bank | `Sim` | result |
//! |---|---|---|---|---|
//! | 6 | 21 | 690 KB | 825 KB | release suite aborts: `stack overflow` on the first fixture |
//! | 4 | 10 | 321 KB | 452 KB | release passes; DEBUG aborts in `saturation.rs` |
//! | 3 | 6 | 193 KB | 324 KB | both profiles pass |
//!
//! A debug build reserves every temporary in the frame up front and does not elide the
//! copy through `Box::new`, so a fixture chain that constructs a `Sim` at two nesting
//! levels holds four of them at once. That is the wall the four-species bank hit, and it
//! is why the number here is not the one the release suite would have licensed —
//! `cargo test` is required green in BOTH profiles, so the binding constraint is the
//! worse one.
//!
//! Three species is enough for this campaign's product (8 H + 8 Cl is two) with one to
//! spare, and it is a DECLARED limit rather than a silent one. Lifting it means giving the
//! bank its own storage strategy — a heap-backed bank for the native shells, or a smaller
//! per-slot knot cap — and that is a successor with a stated cost, not a constant to
//! raise. `examples/sizes.rs` prints the budget so the cost of moving either constant
//! shows up as a number rather than as an unexplained abort in an unrelated fixture.
//!
//! Past the cap the bank REFUSES a new species rather than reusing a slot. The
//! alternative — serving some other pair's curve because this one has nowhere to live — is
//! exactly the defect plant (i) exists to catch, and a cap that silently degrades into it
//! would be the same bug with a capacity limit in front of it.
//!
//! # Provenance is machine-readable here and human-readable in the viewer
//!
//! The engine keeps the parts a GATE can act on: which solver produced the curve, how many
//! determinants that solver faced, the declared uncertainty, and whether the loader
//! claimed the curve was exact. The producer's name and the grid rule are strings, they
//! come off a JSON file the host already parsed, and the host displays them — putting them
//! in here would mean an allocator in the physics core to hold text nothing computes with.
//!
//! What the engine will not do is accept a curve that does not say what it is. See
//! [`TableProvenance::admit`].

use crate::table::PotentialTable;

/// How many distinct species one scene may hold. See the module header for how this was
/// measured, and why it is a refusal rather than a wrap-around.
pub const MAX_SPECIES: usize = 3;

/// Unordered pairs over [`MAX_SPECIES`], including the homonuclear diagonal.
pub const MAX_TABLES: usize = MAX_SPECIES * (MAX_SPECIES + 1) / 2;

/// Determinant count at or above which a pair must arrive as a shipped, referee-pinned
/// table rather than being solved in the browser at load.
///
/// This is the criterion the freeze names, and it is necessary but NOT sufficient — see
/// [`IN_BROWSER_BASIS_LIMIT`], which was added after the cost was measured and the
/// determinant count turned out not to be the driver.
pub const IN_BROWSER_DET_LIMIT: u64 = 1024;

/// Basis-function count above which a pair must arrive as a shipped table, whatever its
/// determinant count.
///
/// # The freeze's criterion, and what measuring it showed
///
/// MIXTURES-1 says "light pairs (declared determinant count below a stated threshold) are
/// solved in-browser". Measured on the campaign machine, 24 knots, release, the cost is
/// not a function of the determinant count:
///
/// | pair | `n_basis` | `n_det` | curve |
/// |---|---|---|---|
/// | H-H   | 2  | 4      | 0.22 s |
/// | H-He  | 2  | 2      | 0.15 s |
/// | He-He | 2  | 1      | 0.11 s |
/// | H-Li  | 6  | 225    | 3.19 s |
/// | H-Cl  | 10 | 100    | 9.97 s |
/// | Li-Li | 10 | 14,400 | 40.73 s |
/// | Cl-Cl | 18 | 324    | 95.95 s |
///
/// Cl-Cl has 324 determinants — fewer than lithium hydride's neighbours and forty times
/// fewer than Li2 — and costs the most of any of them, because the integral transform is
/// a high power of the BASIS SIZE and runs before any determinant is enumerated. A split
/// on `n_det` alone would have sent Cl2 to the browser and hung the page for a minute and
/// a half.
///
/// So both are declared and BOTH must pass. Six admits H2, He2, H-He and every
/// first-row-with-hydrogen pair at a few seconds; ten and up is shipped. The threshold is
/// where it is because 3.19 s is a load a page can absorb once and 9.97 s is not.
pub const IN_BROWSER_BASIS_LIMIT: u64 = 6;

/// Which solver produced a curve. Mirrors `holon_chem::fci::SolverRoute`, restated here
/// because a table can also arrive from a FILE, where the route is a declaration the file
/// makes rather than a fact this process observed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Route {
    /// Nothing was declared. The default, and never admissible: a curve that does not say
    /// what produced it cannot be graded, and "unstated" must not read as "fine".
    Undeclared,
    /// Determinant CI, exact in the declared model up to its reported residual.
    Determinant,
    /// MPS/DMRG: variational inside a bond-dimension budget. Admission is gate D1's.
    Dmrg,
}

impl Route {
    pub fn label(self) -> &'static str {
        match self {
            Route::Undeclared => "undeclared",
            Route::Determinant => "FCI (determinant)",
            Route::Dmrg => "DMRG",
        }
    }
}

/// Where the numbers came from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    /// Solved by this process at load, from Z and the declared basis.
    Solved,
    /// Parsed from a shipped table the referee lane pinned.
    Shipped,
}

/// Why the provenance gate refused a curve. One variant per reason, because a single
/// boolean tells a user that something is wrong and not which thing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    /// The curve declares no solver route.
    RouteUndeclared,
    /// A DMRG curve was presented as exact in the model. It is not, at any bond dimension.
    DmrgClaimedExact,
    /// A DMRG curve arrived while gate D1's validation is not recorded.
    DmrgUnvalidated,
    /// A shipped table declared no uncertainty, or declared zero. An absent bound must
    /// never read as a perfect one.
    UncertaintyMissing,
    /// A DMRG curve declared no uncertainty. Same rule, stated separately because a DMRG
    /// curve's uncertainty is its convergence and it can never be absent.
    DmrgUncertaintyMissing,
    /// A curve small enough to solve in the browser arrived as a shipped file, or one too
    /// large to solve in the browser was solved anyway. Only ever raised for
    /// [`Host::Browser`]; see [`TableProvenance::admit`].
    SplitViolated,
    /// The slot's interpolator does not hold a usable curve, whatever its provenance says.
    CurveNotLoaded,
}

impl Refusal {
    pub fn plain(self) -> &'static str {
        match self {
            Refusal::RouteUndeclared => {
                "this curve does not say which solver produced it, so nothing can grade it"
            }
            Refusal::DmrgClaimedExact => {
                "this curve was produced by DMRG and presented as exact in the model; DMRG \
                 gives a variational upper bound inside a bond-dimension budget, never an \
                 exact value"
            }
            Refusal::DmrgUnvalidated => {
                "this curve was produced by DMRG, and gate D1's validation of the DMRG \
                 bridge is not recorded; the bridge is not admitted"
            }
            Refusal::UncertaintyMissing => {
                "this shipped table declares no uncertainty; an absent bound must not read \
                 as a zero one"
            }
            Refusal::DmrgUncertaintyMissing => {
                "this DMRG curve declares no convergence-derived uncertainty"
            }
            Refusal::SplitViolated => {
                "this curve is on the wrong side of the declared in-browser determinant \
                 limit for the way it arrived"
            }
            Refusal::CurveNotLoaded => {
                "there is no usable curve in this slot; its provenance describes nothing"
            }
        }
    }
}

/// Where the sandbox is running, for the one rule that depends on it.
///
/// # Why the browser split is not a universal law
///
/// "Light pairs are solved at load, heavy pairs are shipped" is a statement about a page's
/// load budget, not about chemistry. A native shell has no such budget: the 3D viewer
/// solves N2 (14,400 determinants) at preset load and always has. Making the split a
/// universal admission rule would have refused four of that shell's existing presets, and
/// a gate that has to be loosened the moment it meets working code was the wrong gate.
///
/// So the host is a parameter, it is declared at the call site, and the two call sites are
/// the wasm ABI (browser) and the native shells (native).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Host {
    /// A page with a load budget. The split is enforced.
    Browser,
    /// A native process. The split does not apply; every other rule still does.
    Native,
}

/// What gate D1 measured, recorded so the provenance gate can act on it.
///
/// # Why this is a record and not a flag
///
/// The freeze says DMRG-only curves may enter the sandbox "only then" — after the overlap
/// species agree at 1e-8 Ha. A boolean saying "D1 passed" is a claim with nothing behind
/// it; this carries the measurement, so the gate that admits a DMRG curve and the number
/// that earned the admission are the same object. Before D1 runs, [`D1Admission::NONE`]
/// is what the crate holds and every DMRG curve is refused.
#[derive(Clone, Copy, Debug)]
pub struct D1Admission {
    /// Whether the overlap comparison was run AND met its stake.
    pub validated: bool,
    /// The worst |E_dmrg - E_fci| measured over the staked overlap species' grids, hartree.
    pub worst_overlap_ha: f64,
    /// The stake the measurement was graded against, hartree.
    pub stake_ha: f64,
    /// How many overlap species were compared. The freeze stakes two.
    pub overlap_species: usize,
}

impl D1Admission {
    /// The state before the gate has run: nothing measured, nothing admitted.
    pub const NONE: D1Admission = D1Admission {
        validated: false,
        worst_overlap_ha: f64::INFINITY,
        stake_ha: 1e-8,
        overlap_species: 0,
    };

    /// Whether this record actually licenses a DMRG curve. Re-derived from the numbers
    /// rather than trusting `validated` alone, so a record whose flag and whose
    /// measurement disagree admits nothing.
    pub fn admits(&self) -> bool {
        self.validated
            && self.overlap_species >= 2
            && self.worst_overlap_ha.is_finite()
            && self.worst_overlap_ha <= self.stake_ha
    }
}

/// THE CRATE'S D1 RECORD.
///
/// `NONE` until the D1 engine half has run and its result is committed. While it is
/// `NONE`, every DMRG-labelled curve is refused by [`TableProvenance::admit`] — which is
/// the freeze's "only then", enforced rather than remembered.
pub const D1_RECORD: D1Admission = D1Admission::NONE;

/// What a curve says about itself.
#[derive(Clone, Copy, Debug)]
pub struct TableProvenance {
    pub route: Route,
    pub source: Source,
    /// Determinant count the solver faced.
    pub n_det: u64,
    /// Contracted basis functions the solve carried. With `n_det`, decides the browser
    /// split — see [`IN_BROWSER_BASIS_LIMIT`] for why one of them is not enough.
    pub n_basis: u64,
    /// Declared absolute uncertainty on the energy column, hartree. Zero means NOT
    /// DECLARED and is refused for anything but a solved determinant curve, whose
    /// uncertainty is its own reported residual.
    pub uncertainty_ha: f64,
    /// What the LOADER claims — as distinct from what the route implies. The two being
    /// separate is the whole of plant (iii): a DMRG curve arriving with this set is a
    /// curve presented as exact, and the gate must refuse it.
    pub claimed_exact: bool,
}

impl TableProvenance {
    /// An empty slot: nothing loaded, nothing claimed.
    pub const UNKNOWN: TableProvenance = TableProvenance {
        route: Route::Undeclared,
        source: Source::Solved,
        n_det: 0,
        n_basis: 0,
        uncertainty_ha: 0.0,
        claimed_exact: false,
    };

    /// A curve this process solved on the determinant route.
    pub fn solved_exact(n_det: u64, n_basis: u64, residual_ha: f64) -> Self {
        Self {
            route: Route::Determinant,
            source: Source::Solved,
            n_det,
            n_basis,
            uncertainty_ha: residual_ha,
            claimed_exact: true,
        }
    }

    /// Whether this curve is too expensive to solve at page load.
    ///
    /// EITHER threshold is enough to make it heavy. See [`IN_BROWSER_BASIS_LIMIT`].
    pub fn is_heavy(&self) -> bool {
        self.n_det >= IN_BROWSER_DET_LIMIT || self.n_basis > IN_BROWSER_BASIS_LIMIT
    }

    /// THE PROVENANCE GATE. `Ok(())` admits the curve; `Err` says which rule it broke.
    ///
    /// Every branch here has a demonstrated failing case in `tests/mixtures.rs` — standing
    /// question 4, and the reason plant (iii) exists: a refusal nobody has watched fire is
    /// indistinguishable from a refusal that cannot.
    pub fn admit(&self, d1: &D1Admission, host: Host) -> Result<(), Refusal> {
        match self.route {
            Route::Undeclared => return Err(Refusal::RouteUndeclared),
            Route::Dmrg => {
                // Order matters: the false CLAIM is a worse fault than the missing
                // admission, so it is reported first. A curve that lies about what it is
                // would otherwise be reported as merely unvalidated, and someone would go
                // and validate it.
                if self.claimed_exact {
                    return Err(Refusal::DmrgClaimedExact);
                }
                if !(self.uncertainty_ha > 0.0) {
                    return Err(Refusal::DmrgUncertaintyMissing);
                }
                if !d1.admits() {
                    return Err(Refusal::DmrgUnvalidated);
                }
            }
            Route::Determinant => {}
        }
        if self.source == Source::Shipped && !(self.uncertainty_ha > 0.0) {
            return Err(Refusal::UncertaintyMissing);
        }
        // The browser split, both directions, and only in the browser. A heavy pair
        // solved at page load would be a page that does not load; a light pair shipped as
        // a file would be a number trusted where the sandbox could have computed it, which
        // is the property the whole app is built on.
        if host == Host::Native {
            return Ok(());
        }
        let heavy = self.is_heavy();
        match (self.source, heavy) {
            (Source::Solved, true) => Err(Refusal::SplitViolated),
            (Source::Shipped, false) => Err(Refusal::SplitViolated),
            _ => Ok(()),
        }
    }
}

/// One curve per unordered species pair, plus what each curve says about itself.
pub struct PairBank {
    tables: [PotentialTable; MAX_TABLES],
    provenance: [TableProvenance; MAX_TABLES],
    /// Nuclear charges present, in registration order. `z[i]` is species index `i`.
    z: [u32; MAX_SPECIES],
    n_species: usize,
    /// Slots that have been filled. A slot can be registered (its species are known) and
    /// still empty (its curve has not been loaded).
    filled: [bool; MAX_TABLES],
}

/// Upper-triangular packing including the diagonal: species indices `(i, j)` with
/// `i <= j` map to one slot, so `(H, Cl)` and `(Cl, H)` are the same curve rather than two
/// things to keep in step.
pub const fn slot_index(i: usize, j: usize) -> usize {
    let (lo, hi) = if i <= j { (i, j) } else { (j, i) };
    // Rows of length MAX_SPECIES, MAX_SPECIES-1, ... : offset of row `lo` is
    // lo*MAX_SPECIES - lo*(lo-1)/2.
    lo * MAX_SPECIES - (lo * lo - lo) / 2 + (hi - lo)
}

const EMPTY_TABLE: PotentialTable = PotentialTable::empty();
const EMPTY_PROV: TableProvenance = TableProvenance::UNKNOWN;

impl PairBank {
    pub const fn empty() -> Self {
        Self {
            tables: [EMPTY_TABLE; MAX_TABLES],
            provenance: [EMPTY_PROV; MAX_TABLES],
            z: [0; MAX_SPECIES],
            n_species: 0,
            filled: [false; MAX_TABLES],
        }
    }

    /// An empty bank with HYDROGEN already registered as species 0, so slot 0 is the H-H
    /// pair.
    ///
    /// This is what `Sim::empty` starts from, and it is what makes the bank's arrival
    /// invisible to everything that came before it: `Atom::default` is hydrogen, the
    /// single-table sandbox was a hydrogen sandbox, and every existing test and example
    /// loads its curve through the legacy door into slot 0. Seeding the species the whole
    /// crate already defaults to keeps that door pointing at the same curve it always
    /// pointed at.
    pub const fn hydrogen_seeded() -> Self {
        let mut b = Self::empty();
        b.z[0] = 1;
        b.n_species = 1;
        b
    }

    pub fn species_count(&self) -> usize {
        self.n_species
    }

    pub fn species_z(&self, i: usize) -> u32 {
        if i < self.n_species {
            self.z[i]
        } else {
            0
        }
    }

    /// The species index for a nuclear charge, or `None` if the scene has not registered
    /// it.
    pub fn index_of(&self, z: u32) -> Option<usize> {
        self.z[..self.n_species].iter().position(|&x| x == z)
    }

    /// Register a species, returning its index. Idempotent; `None` past [`MAX_SPECIES`].
    pub fn register(&mut self, z: u32) -> Option<usize> {
        if let Some(i) = self.index_of(z) {
            return Some(i);
        }
        if self.n_species >= MAX_SPECIES {
            return None;
        }
        let i = self.n_species;
        self.z[i] = z;
        self.n_species += 1;
        Some(i)
    }

    /// Forget every curve and return to the hydrogen-seeded state.
    ///
    /// Used when a scene is rebuilt from scratch, so a species that has left the scene
    /// cannot keep a slot warm — which matters because the cap is three, and a shell that
    /// cycled through presets without clearing would be full after the third one.
    ///
    /// Re-seeds hydrogen rather than emptying the species list, for the same reason
    /// [`PairBank::hydrogen_seeded`] exists: slot 0 is the H-H pair everywhere in this
    /// crate, and a `clear` that left slot 0 belonging to nobody would break the legacy
    /// single-curve door the moment it was used after one.
    pub fn clear(&mut self) {
        self.n_species = 1;
        self.z = [0; MAX_SPECIES];
        self.z[0] = 1;
        for s in 0..MAX_TABLES {
            self.tables[s] = PotentialTable::empty();
            self.provenance[s] = TableProvenance::UNKNOWN;
            self.filled[s] = false;
        }
    }

    /// The slot serving two REGISTERED species indices.
    pub fn slot(&self, i: usize, j: usize) -> usize {
        slot_index(i, j)
    }

    /// The slot serving two nuclear charges, if both are registered.
    pub fn slot_of_z(&self, za: u32, zb: u32) -> Option<usize> {
        Some(slot_index(self.index_of(za)?, self.index_of(zb)?))
    }

    /// The curve for two REGISTERED species indices.
    ///
    /// Takes indices rather than charges because the force loop resolves each atom's index
    /// ONCE per force evaluation and then reads pairs off it; resolving a charge inside the
    /// inner loop would be the same lookup done `N^2/2` times.
    pub fn table_at(&self, i: usize, j: usize) -> &PotentialTable {
        &self.tables[slot_index(i, j)]
    }

    pub fn table_slot(&self, s: usize) -> &PotentialTable {
        &self.tables[s]
    }

    pub fn table_slot_mut(&mut self, s: usize) -> &mut PotentialTable {
        &mut self.tables[s]
    }

    pub fn provenance_at(&self, i: usize, j: usize) -> &TableProvenance {
        &self.provenance[slot_index(i, j)]
    }

    pub fn provenance_slot(&self, s: usize) -> &TableProvenance {
        &self.provenance[s]
    }

    /// Whether this slot holds a curve the force loop can evaluate.
    ///
    /// PHYSICS READINESS, deliberately not provenance. The two questions are separate and
    /// were separated on purpose: an interpolator either has knots or it does not, and
    /// that is what the integrator needs to know. Whether the curve is ALLOWED to be here
    /// is [`PairBank::provenance_admitted`], and it is enforced at the doors — a curve the
    /// gate refuses is evicted by [`PairBank::commit`], so it never reaches this question.
    pub fn is_filled(&self, s: usize) -> bool {
        self.tables[s].is_loaded()
    }

    /// Whether every filled slot's provenance was ADMITTED by the gate.
    ///
    /// False also when a curve was written straight into a slot through
    /// [`PairBank::table_slot_mut`] without anything being declared about it — the legacy
    /// door the older tests and examples use. That reads as `Route::Undeclared`, which is
    /// exactly what it is, and it is reported rather than assumed benign.
    pub fn provenance_admitted(&self, d1: &D1Admission, host: Host) -> bool {
        (0..MAX_TABLES)
            .filter(|&s| self.is_filled(s))
            .all(|s| self.provenance[s].admit(d1, host).is_ok())
    }

    /// The first filled slot whose provenance the gate refuses, and why.
    pub fn first_refusal(&self, d1: &D1Admission, host: Host) -> Option<(usize, Refusal)> {
        (0..MAX_TABLES)
            .filter(|&s| self.is_filled(s))
            .find_map(|s| self.provenance[s].admit(d1, host).err().map(|r| (s, r)))
    }

    /// Record a freshly loaded slot's provenance, running the gate over it first.
    ///
    /// # The refusal EVICTS
    ///
    /// A gate that returns an error and leaves the curve in the slot is a gate the force
    /// loop can walk straight past — the caller has to remember to undo the load, and the
    /// one time it does not, a refused curve is silently exerting forces. So a refusal
    /// clears the slot here, in the same call that decides it. What the caller does with
    /// the returned `Refusal` is display it; what it cannot do is accidentally keep the
    /// curve.
    ///
    /// This is the door every provenance-carrying loader goes through. A raw write through
    /// [`PairBank::table_slot_mut`] does not, and leaves `Route::Undeclared` behind, which
    /// [`PairBank::provenance_admitted`] reports.
    pub fn commit(
        &mut self,
        s: usize,
        prov: TableProvenance,
        d1: &D1Admission,
        host: Host,
    ) -> Result<(), Refusal> {
        if let Err(e) = prov.admit(d1, host) {
            self.evict(s);
            return Err(e);
        }
        if !self.tables[s].is_loaded() {
            self.evict(s);
            return Err(Refusal::CurveNotLoaded);
        }
        self.provenance[s] = prov;
        self.filled[s] = true;
        Ok(())
    }

    /// Drop a slot's curve. Used by the plants and by a species change that invalidates a
    /// pair.
    pub fn evict(&mut self, s: usize) {
        self.tables[s] = PotentialTable::empty();
        self.provenance[s] = TableProvenance::UNKNOWN;
        self.filled[s] = false;
    }

    /// The FIRST filled slot, which for a single-species scene is that scene's only curve.
    ///
    /// This is what the single-curve parts of the ABI read — `holon_table_r_e`,
    /// `holon_curve_u`, the banner's residual — and it is what keeps an all-hydrogen scene
    /// reporting exactly what it reported before the bank existed. It is deliberately NOT
    /// used by anything dynamical: the force loop, the bond criterion and the envelope all
    /// dispatch per pair, because "the primary curve" is not a physical quantity.
    pub fn primary(&self) -> &PotentialTable {
        for s in 0..MAX_TABLES {
            if self.filled[s] {
                return &self.tables[s];
            }
        }
        &self.tables[0]
    }

    pub fn primary_slot(&self) -> usize {
        for s in 0..MAX_TABLES {
            if self.filled[s] {
                return s;
            }
        }
        0
    }

    /// Every slot the currently registered species can reach, filled or not.
    pub fn active_slots(&self) -> impl Iterator<Item = usize> + '_ {
        let n = self.n_species;
        (0..n).flat_map(move |i| (i..n).map(move |j| slot_index(i, j)))
    }

    /// How many curves are loaded.
    pub fn filled_count(&self) -> usize {
        (0..MAX_TABLES).filter(|&s| self.is_filled(s)).count()
    }
}
