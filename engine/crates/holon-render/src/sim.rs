//! Hydrogen atoms in a box, integrated symplectically, with every energy and momentum
//! flow written to a ledger.
//!
//! Units are Hartree atomic units throughout: length in bohr, energy in hartree, mass
//! in electron masses, time in hbar/E_h (24.189 as). Nothing is converted for display
//! except in the viewer, so no unit constant is ever applied twice.
//!
//! # Two dimensions and three, in ONE integrator
//!
//! The state carries three components per atom and the box has three pairs of faces.
//! The 2D scene is not a separate code path: it is the exact z = depth/2 SLICE of the
//! same 3D world. Every atom starts on that plane with `vz = 0`; the pair force along z
//! is `(slope/r) * dz` with `dz = 0`, the z faces are never reached, so `az` is
//! identically `0.0` and the plane is invariant — not approximately, exactly, because a
//! float times zero is zero and adding zero to a finite float changes no bit. Every
//! sum that grew a third term grew it in the order `(xx + yy) + zz`, so the 2D
//! arithmetic is bit-for-bit what it was before the lift. That is what lets the canvas
//! shell, the browser ABI and the existing gate tests carry over untouched.
//!
//! Exactly two things are genuinely dimension-dependent, and both are named rather than
//! inferred: the equipartition denominator in [`Sim::temperature`] (2 translational
//! degrees of freedom per atom against 3), and the opening scene in [`Sim::reset`].
//! Both read [`Sim::dims`]. Everything else — the curve, the force law, the bond
//! predicate, the turning point, the drift bound, the clocks — is RADIAL, a function of
//! the scalar separation alone, and so carries into 3D with nothing to re-derive.

use crate::bank::{PairBank, MAX_SPECIES};
use crate::clock::Timescale;
use crate::holon::HolonLayer;
use crate::table::PotentialTable;
use holon_chem::trimer::TrimerTable;
use holon_chem::water::WaterTable;

/// Mass of a protium ATOM (proton + electron) in electron masses:
/// 1.00782503207 u x 1822.888486 m_e/u. The atom, not the proton — the pair curve is
/// Born-Oppenheimer, so the electrons ride with the nuclei and their mass belongs here.
pub const M_H: f64 = 1837.152;

/// The proton, for reference: 1836.152673 m_e.
///
/// The brief specifies the reduced mass of two PROTONS for the timescale derivation, and
/// this crate uses the reduced mass of two ATOMS instead. The reason is the one stated
/// above — the curve is Born-Oppenheimer, so the electrons ride with the nuclei and their
/// inertia belongs in the moving mass — and the cost of the choice is 1 electron mass in
/// 1837, i.e. 0.054% on mu and 0.027% on every frequency derived from it. Recorded here
/// rather than silently resolved, because it is a deliberate departure from the brief and
/// it should be visible to whoever checks the numbers.
pub const M_PROTON: f64 = 1836.152673;

/// The DEFAULT SCENE SIZE, and nothing else.
///
/// This used to be `MAX_ATOMS`, a hard capacity: every per-atom array in [`Sim`] was
/// `[T; 16]` and `reset` clamped to it. That cap is gone — the state is heap-backed and
/// sized by the scene (T3). What survives is the number the viewer opens with and the
/// number the device-calibration burst times, which is a CHOICE about a scene, not a
/// statement about what the engine can hold.
///
/// M-DEVICE-CLASS: the calibration burst is a fixed scene on purpose. A burst whose size
/// varied with the scene would report a rate that could not be compared between devices.
pub const DEFAULT_SCENE_ATOMS: usize = 16;

/// The number of unordered pairs in a scene of `n` atoms.
///
/// A function where there was a constant. `MAX_PAIRS` was `MAX_ATOMS * (MAX_ATOMS - 1)/2`
/// and sized a fixed array; at the scales T3 exists for that array is both wrong and
/// enormous (N = 10⁴ is 5·10⁷ entries), so the pair sector is heap-backed and — past the
/// cell-list work — cutoff-local rather than complete. This helper is what the callers
/// that genuinely want the COMPLETE count (a ceiling, a cost model, a calibration figure)
/// call, so that the arithmetic lives in one place instead of being re-derived.
#[inline]
pub const fn complete_pairs(n: usize) -> usize {
    n * n.saturating_sub(1) / 2
}

/// Boltzmann's constant in hartree per kelvin.
pub const K_B: f64 = 3.166811563e-6;

/// Wall stiffness, hartree/bohr^2. A5 stage value: it is scene furniture, not physics
/// from any table, and is named as such here rather than hidden in the force loop.
pub const K_WALL: f64 = 0.5;
/// User-spring stiffness, hartree/bohr^2. Also a stage value. Finite on purpose: a
/// finite spring cannot push two atoms arbitrarily far up the repulsive wall, which is
/// the honest behaviour rather than a cheat that lets the pointer overpower the curve.
pub const K_SPRING: f64 = 0.05;

/// Standard gravity, m/s^2 (CGPM 1901, exact by definition).
pub const G_SI: f64 = 9.80665;
/// The bohr in metres (CODATA 2018).
pub const BOHR_M: f64 = 0.529177210903e-10;
/// One atomic unit of time in seconds, `hbar / E_h` (CODATA 2018).
pub const AU_TIME_S: f64 = 2.4188843265857e-17;

/// ONE G, in atomic units of acceleration (bohr per atomic time unit squared).
///
/// `a_au = a_SI * t_au^2 / a_0` -- a unit conversion and nothing else, which is why it is
/// a `const` expression over three named constants rather than a number typed in. It
/// works out to about 1.08e-22, and the smallness is the POINT rather than a reason to
/// round it away: FSD-W1 WB-2.4 puts gravity forward as the workbench's cleanest
/// tier-separation exhibit -- one field, correctly invisible at 1 nm, sovereign over a
/// kilometre of water. `tests/gravity.rs` MEASURES the ratio to kT rather than asserting
/// the adjective, and the FSD's own staked figure is checked there against it.
///
/// NOT a fitted parameter and NOT a force field, so WB-5.1 is untouched: that clause bans
/// an empirical potential BETWEEN PARTICLES. This is a uniform external field with one
/// defined constant, in the same category as the box the walls make.
pub const G_EARTH_AU: f64 = G_SI * AU_TIME_S * AU_TIME_S / BOHR_M;

/// Why a scene will not take a gravitational field.
///
/// One variant, and it is a statement about geometry rather than about policy.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GravityRefusal {
    /// A PERIODIC box has no bottom. `m g y` is a linear potential and the wrap makes it
    /// discontinuous: an atom that leaves the top face re-enters at the bottom with its
    /// potential energy changed by `m g H` and nothing having done that work, so the
    /// balance gate opens by exactly that jump on every crossing. The field is not
    /// well-posed on a torus, and this refuses rather than reporting an ever-growing
    /// "drift" that is really a coordinate artifact. Conservation is chart-relative and
    /// this chart has no bottom to fall toward.
    PeriodicBox,
}

impl GravityRefusal {
    /// Why, in the words the viewer shows.
    pub fn plain(self) -> &'static str {
        match self {
            GravityRefusal::PeriodicBox =>
                "a periodic box has no bottom: m*g*y jumps by m*g*H at the wrap, so the \
                 energy ledger cannot close over it. Use walls or an open box.",
        }
    }
}

/// Distance beyond which the outer-turning-point search gives up and reports infinity.
const TURNING_POINT_CAP: f64 = 200.0;

/// The (O,H,H,H) four-body sector's outer radius, bohr: past this every O-H distance puts
/// the quadruple outside the switch and the term is an exact zero without a solve.
pub const DE4_R_CUT: f64 = 6.0;
/// The four-body switch's inner edge, bohr: inside this the term is at full weight.
pub const DE4_R_IN: f64 = 5.0;

/// Width of the pair truncation's switch window, bohr.
///
/// A DECLARED width, and the one number in the truncation that is a choice rather than a
/// reading. The inner edge is derived from the curve and the energy budget
/// ([`Sim::derive_pair_cutoff`]); the window then has to be wide enough that the switch's
/// own curvature — which enters the drift bound through `S''·U` — is small against the
/// curve's, and narrow enough not to push the cell size up for nothing. Two bohr is about
/// four times the H-H well width, so `S''·U` at the inner edge is the budget over four
/// square bohr, i.e. under the budget itself.
pub const PAIR_SWITCH_WIDTH: f64 = 2.0;

/// Safety factor on the derived drift bound. The (omega*dt)^2/4 result below is EXACT
/// for a harmonic oscillator and leading-order in dt^2 for anything else; anharmonicity
/// enters at the same order with a coefficient set by U''' and the amplitude, so a
/// fixed multiple is the honest way to admit "leading order, not a theorem here". The
/// measured-over-bound ratio is reported so the margin is visible rather than absorbed.
pub const DRIFT_SAFETY: f64 = 4.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Boundary {
    /// Soft quadratic walls on every face of the box: four sides in 2D, six in 3D.
    Walls,
    /// No walls at all. Translation invariance is exact, so total momentum is conserved
    /// to roundoff and the momentum gate has nothing to subtract.
    Open,
    /// THE PERIODIC BOX: every separation is taken under the minimum-image convention,
    /// and an atom leaving one face re-enters through the opposite one.
    ///
    /// Like [`Boundary::Open`] this does no work and delivers no impulse — the box has no
    /// walls to push against — so BOTH conservation gates apply in their strict form: the
    /// energy ledger closes and the momentum residual is roundoff only. That is the point
    /// of using it for bulk: walls are a boundary artifact, and the periodic box is how a
    /// finite scene stops pretending it has edges.
    ///
    /// Correctness condition, enforced by [`Sim::pbc_ok`] rather than assumed: every
    /// interaction cutoff must be at most HALF the shortest box edge. Past that an atom
    /// interacts with two images of the same partner and the minimum image is no longer
    /// the only image.
    Periodic,
}

impl Boundary {
    /// Does this box push back?
    ///
    /// TWO boundaries are wall-less and they are wall-less for different physical reasons:
    /// [`Boundary::Open`] has no container at all, [`Boundary::Periodic`] has one that
    /// closes on itself. The code reason is the same one, so it is asked once here rather
    /// than twice at the call site — which is where PLANT P-2 found it, at 1.7e4 hartree,
    /// after `Periodic` was added and `wall_energy_force`'s single `== Open` test silently
    /// kept applying walls to a box that has none. (Folded rather than registered: see
    /// `conformance/water_observatory/DRY_RESIDUALS.md`, R-1.)
    #[inline]
    pub fn has_walls(self) -> bool {
        matches!(self, Boundary::Walls)
    }

    /// Does an atom leaving one face re-enter through the opposite one?
    #[inline]
    pub fn wraps(self) -> bool {
        matches!(self, Boundary::Periodic)
    }
}

/// How many spatial dimensions the SCENE uses. The integrator always carries three
/// components; this says how many of them the scene is allowed to move in, and it is
/// read by exactly the two places where the answer differs (see the module header).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dims {
    /// The z = depth/2 plane. The default, and what the canvas shell draws.
    Two,
    /// The full box.
    Three,
}

impl Dims {
    /// Translational degrees of freedom per atom — the equipartition denominator.
    pub fn dof(self) -> f64 {
        match self {
            Dims::Two => 2.0,
            Dims::Three => 3.0,
        }
    }
}

use holon_chem::elements::{Species, HYDROGEN};

#[derive(Clone, Copy, Debug)]
pub struct Atom {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub species: Species,
}

impl Default for Atom {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            species: HYDROGEN,
        }
    }
}

impl Atom {
    #[inline]
    pub fn mass(&self) -> f64 {
        if self.species.z == 1 {
            M_H
        } else {
            self.species.mass_me()
        }
    }

    /// The drawn radius. `homonuclear_radius` is measured for ten species and `None` for
    /// the other forty-four in the registry, so this makes the fallback a DECLARED choice
    /// ([`holon_chem::elements::UNDECLARED_RADIUS`]) instead of the hydrogen value the
    /// old signature returned silently for every element past neon.
    #[inline]
    pub fn radius(&self) -> f64 {
        self.species
            .homonuclear_radius()
            .unwrap_or(holon_chem::elements::UNDECLARED_RADIUS)
    }
}

/// One pair's bond reading, computed from the table alone.
#[derive(Clone, Copy, Default)]
pub struct PairReading {
    pub i: usize,
    pub j: usize,
    pub r: f64,
    /// Relative energy in the pair's own centre-of-mass frame, asymptote-zeroed.
    pub e_rel: f64,
    /// Outer classical turning point of the effective radial potential at `e_rel`.
    pub r_outer: f64,
    pub bonded: bool,
}

impl PairReading {
    /// Bond-sector energy for this pair's ledger row: pair potential plus pair-frame
    /// kinetic energy.
    ///
    /// Numerically this IS `e_rel` — the same quantity in a second role. It is named
    /// separately because the roles are different (one is the bond criterion's input, the
    /// other is a composite holon's ledger row) and defined once because two definitions
    /// of one number is how they drift apart.
    pub fn e_bond(&self) -> f64 {
        self.e_rel
    }
}

/// ONE PAIR's evaluated contribution, before anything is added up.
///
/// The force loop used to evaluate and accumulate in the same statement. Splitting them is
/// what makes the loop shardable: EVALUATION is a pure function of the state (hundreds of
/// flops per pair, and all of the cost), while ACCUMULATION is a sum whose ORDER is part of
/// the answer. So evaluation goes wide and accumulation stays in one canonical pass — and
/// the threaded run is then bit-for-bit the serial one, rather than merely close to it.
#[derive(Clone, Copy, Debug, Default)]
pub struct PairTerm {
    pub fx: f64,
    pub fy: f64,
    pub fz: f64,
    pub value: f64,
    pub curv: f64,
    /// `r · dU/dr` — this pair's contribution to the internal virial.
    ///
    /// Carried on the term rather than recomputed from the force, because recomputing it
    /// would need the separation again and the two derivations would then be two places for
    /// the sign to be wrong. The pressure is `(2K − Σ virial) / 3V`, so a sign error here is
    /// a barostat that expands under compression.
    pub virial: f64,
}

/// ONE TRIPLE's evaluated contribution. Same split, same reason.
#[derive(Clone, Copy, Debug, Default)]
pub struct TripleTerm {
    /// The three atoms, in the order the surface's own argument convention wants them.
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub v: f64,
    /// `dE/dr` along `ab`, `ac`, `bc`.
    pub g: [f64; 3],
    /// The three side lengths, in the same order.
    pub r: [f64; 3],
    /// The per-triple stiffness the drift bound is built from.
    pub kt: f64,
    /// False when the triple had no server or evaluated to an exact zero — kept as a slot
    /// rather than compacted away, so a chunk's output length is a function of its input
    /// length alone and never of what the physics happened to say.
    pub live: bool,
}

/// WHO EVALUATES THE TERMS.
///
/// The seam the multithreading goes through, and the reason it is a trait rather than a
/// thread pool: `holon-render` is linked into the browser artifact, and
/// `wasm32-unknown-unknown` has no `std::thread`. The engine therefore owns the WORK
/// (`Sim::eval_pair_chunk`, `Sim::eval_triple_chunk` — pure, `&self`, no allocation) and
/// something else owns the WORKERS. `holon-tables` made the same split for the same reason
/// one level down, and says so in its manifest.
///
/// An implementation may run the chunks in any order and on any number of threads. It may
/// NOT change the chunking, because the accumulation pass walks the terms in index order
/// and that order is the answer.
pub trait ForceExecutor {
    /// Evaluate `terms`, whose entry `k` belongs to `sim.neighbours().pairs[k]`.
    fn eval_pairs(&self, sim: &Sim, terms: &mut [PairTerm], chunk: usize);
    /// Evaluate `terms`, whose entry `k` belongs to `sim.triples()[k]`.
    fn eval_triples(&self, sim: &Sim, terms: &mut [TripleTerm], chunk: usize);
    /// How many workers this executor actually has. Reported for the log; nothing branches
    /// on it, because a result that depended on the worker count would be the defect the
    /// whole design exists to prevent.
    fn workers(&self) -> usize {
        1
    }
}

/// The executor that is always available: this thread, in order.
///
/// It is not a fallback or a degraded mode — it is the REFERENCE. `tests/t3_parallel.rs`
/// holds every other executor against it and requires bit-identical output, which is the
/// same shape `holon-mesh` uses for its shards and `holon-tables` for its table workers.
#[derive(Clone, Copy, Debug, Default)]
pub struct SerialExecutor;

impl ForceExecutor for SerialExecutor {
    fn eval_pairs(&self, sim: &Sim, terms: &mut [PairTerm], chunk: usize) {
        let chunk = chunk.max(1);
        for (ci, part) in terms.chunks_mut(chunk).enumerate() {
            sim.eval_pair_chunk(ci * chunk, part);
        }
    }
    fn eval_triples(&self, sim: &Sim, terms: &mut [TripleTerm], chunk: usize) {
        let chunk = chunk.max(1);
        for (ci, part) in terms.chunks_mut(chunk).enumerate() {
            sim.eval_triple_chunk(ci * chunk, part);
        }
    }
}

/// Terms per chunk. A COST parameter and never a correctness one: the accumulation walks
/// the terms in index order whatever the chunking, so changing this changes how the work is
/// handed out and not one bit of the answer. Sized so a chunk is a few hundred microseconds
/// of interpolant evaluation — big enough that the handover is noise, small enough that a
/// straggler cannot hold the frame.
pub const FORCE_CHUNK: usize = 1024;

/// THE EXTERNAL-WORK RECEIPT COLUMNS (FSD-W1 WB-4.3).
///
/// One column per thing that can reach into a closed scene and move its energy. They sum
/// to [`Sim::w_ext`], and the sum is CHECKED ([`Sim::work_columns_ok`]) rather than
/// assumed — a column that stops being posted would otherwise leave the balance gate
/// green and the attribution silently wrong, which is the vacuous-success shape
/// (M-VACUOUS-SUCCESS).
///
/// The columns are separate because they answer different questions and fail differently:
/// the hand is driven by a person and can inject arbitrarily much; the thermostat is a
/// controller with a target; the barostat moves the box rather than the atoms. A run that
/// took 3 mEh from the hand and gave 3 mEh back to the thermostat has a total of zero and
/// two large receipts, and those are not the same run as one nobody touched.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ExternalWork {
    /// THE HAND: anchor motion at fixed atom position (`dU` of the spring term, which is
    /// exactly the work the user's hand did) and the spring energy that leaves with the
    /// hand on release.
    pub hand: f64,
    /// The thermostat's velocity rescaling.
    pub thermostat: f64,
    /// The barostat's box work: the potential energy change produced by moving the box
    /// walls at fixed scaled coordinates, plus the kinetic change from rescaling.
    pub barostat: f64,
}

impl ExternalWork {
    /// The columns' sum, in a FIXED order so the total is reproducible.
    #[inline]
    pub fn total(&self) -> f64 {
        (self.hand + self.thermostat) + self.barostat
    }

    #[inline]
    pub const fn zero() -> Self {
        Self {
            hand: 0.0,
            thermostat: 0.0,
            barostat: 0.0,
        }
    }

    /// The largest column magnitude — the scale a roundoff bound on the sum is taken
    /// against.
    #[inline]
    pub fn scale(&self) -> f64 {
        self.hand
            .abs()
            .max(self.thermostat.abs())
            .max(self.barostat.abs())
    }
}

pub struct Sim {
    /// THE PAIR-TABLE BANK: one curve per unordered species pair. See `bank.rs`.
    ///
    /// Replaces the single `table` this struct used to hold. Read it through
    /// [`Sim::table`] for the single-curve views that have always existed (the banner, the
    /// curve plot, `r_e`/`d_e`), and through [`Sim::table_for`] for anything dynamical —
    /// the force loop, the bond criterion, the envelope. The distinction is not
    /// stylistic: "the curve" is a display convenience in a mixed scene and a physical
    /// quantity in a pure one, and the second kind of reader must never get the first
    /// kind of answer.
    pub bank: PairBank,
    /// The three-body surface. Empty until [`crate::generate_trimer_table`] fills it, and
    /// an empty one contributes an EXACT zero to every term below — so a scene that never
    /// asks for it is bit-for-bit the scene this file simulated before the term existed.
    pub trimer: TrimerTable,
    /// The HETERONUCLEAR three-body surface, (O, H, H). Empty until
    /// [`crate::generate_water_table`] fills it, and an empty one contributes an EXACT
    /// zero exactly as an empty [`Sim::trimer`] does.
    ///
    /// Heap-backed where `trimer` is a fixed array, and that is a size decision rather
    /// than a style one: at 65 x 65 x 33 nodes this surface is 1.1 MB, against the whole
    /// bank's 193 KB, and `crate::bank`'s `MAX_SPECIES` cap records what happens when a
    /// `Sim` constructed by value in a nested fixture outgrows the stack. So the nodes
    /// live behind a pointer and a `Sim` grows by three words.
    pub water: WaterTable,
    /// The HETERONUCLEAR (O, O, H) three-body surface.
    pub ooh: holon_chem::ooh::OohTable,
    /// The HOMONUCLEAR (O, O, O) Ozone three-body surface.
    pub ozone: holon_chem::ozone::OzoneTable,
    /// SHIPPED heteronuclear three-body surfaces, and the door they came through.
    ///
    /// Distinct from [`Sim::trimer`] and [`Sim::water`] because it is neither generated
    /// here nor a single fixed system: it is a bank of artifacts the mesh computed, each
    /// carrying its own provenance, each admitted or refused by
    /// [`crate::trimer_bank::TrimerProvenance::admit`]. Empty until one is loaded, and an
    /// empty bank contributes an EXACT zero exactly as the two above do.
    pub trimers: crate::trimer_bank::TrimerBank,
    /// Triples the three-body sector REFUSED for want of a table: (O, O, H) and (O, O, O),
    /// which SATURATION-2 does not tabulate. Counted rather than ignored, because the
    /// prereg requires the fence's incidence in the quench runs to be reported, and a
    /// truncation nobody counts is a truncation nobody can weigh.
    pub fence_untabulated: u64,
    /// THE SCENE. Heap-backed and sized by [`Sim::reset`]; `atoms.len() == n` is an
    /// invariant every mutator maintains and [`Sim::storage_ok`] states.
    ///
    /// This was `[Atom; 16]`. The cap is gone, and with it the reason the whole engine
    /// could not be pointed at a bulk scene. Everything per-atom below moved the same way,
    /// and each was allocated with `vec![...]` rather than `Box::new([...; N])` on purpose:
    /// the array form BUILDS THE ARRAY ON THE STACK and then moves it, so a boxed
    /// `[Atom; 100_000]` overflows the stack before the heap ever sees it.
    pub atoms: Vec<Atom>,
    pub n: usize,
    pub boundary: Boundary,
    pub width: f64,
    pub height: f64,
    /// The box's z extent. Unreachable in [`Dims::Two`], where every atom sits on the
    /// mid-plane and no force can move it off — kept anyway, because the mid-plane is
    /// defined as `depth / 2` and a scene that flips to [`Dims::Three`] must find a box
    /// already centred on it rather than one that starts at a face.
    pub depth: f64,
    /// The walls act on atom centres, inset by the drawn radius so the picture and the
    /// physics agree about where the edge is.
    pub wall_inset: f64,
    /// Which dimensions the scene moves in. See the module header.
    pub dims: Dims,

    // --- accelerations, kept split so the momentum ledger can name what is external ---
    a_pair: Vec<(f64, f64, f64)>,
    a_ext: Vec<(f64, f64, f64)>,

    /// THE CELL LIST: the scene bucketed by position so the interaction loops are
    /// cutoff-local. Rebuilt by [`Sim::compute_forces`]; see `crate::cells`.
    /// Scratch: each atom's bank slot, rewritten from the atoms on every force
    /// evaluation. See [`Sim::refresh_slots`].
    slots: Vec<usize>,
    /// THE DECLARED PAIR TRUNCATION: `(r_in, r_cut)` of the C² switch, or `None` for the
    /// complete pair sum.
    ///
    /// `None` is the default and it is not laziness: the pair curve's tail is an
    /// exponential, never an exact zero, so any pair cutoff DROPS ENERGY. A scene that
    /// wants `O(N)` pairs says so, and gets told what it paid ([`Sim::truncation_floor`]).
    /// Set through [`Sim::set_pair_cutoff`], which refuses a cutoff the periodic box
    /// cannot honour.
    pub(crate) pair_switch: Option<(f64, f64)>,
    /// The per-pair energy the declared truncation drops, hartree — the bound the switch
    /// window was DERIVED from, not a description of it. Zero when there is no
    /// truncation.
    pub(crate) pair_floor: f64,
    /// WHO EVALUATES. `None` is [`SerialExecutor`] — the reference — and is what
    /// `Sim::empty` starts with, because `empty` is a `const fn` and a box is not.
    executor: Option<Box<dyn ForceExecutor + Send + Sync>>,
    /// Evaluated terms, held across calls so a force evaluation allocates nothing once the
    /// scene has settled at a size.
    pair_terms: Vec<PairTerm>,
    triple_terms: Vec<TripleTerm>,
    /// Scratch for the many-body enumerations, held so a force evaluation allocates
    /// nothing once the scene has settled at a size. Each is cleared before use and is
    /// never read across a call.
    triple_scratch: Vec<[usize; 3]>,
    k_atom_scratch: Vec<f64>,
    quad_force_scratch: Vec<(f64, f64, f64)>,
    pub(crate) cells: crate::cells::CellList,
    /// The neighbour pairs the cell list produced this force evaluation, and the distance
    /// alongside each — computed once and read by the pair loop, the triple loop, the
    /// quadruple loop and the bond reading rather than four times.
    pub(crate) neighbours: crate::cells::Neighbours,

    // --- the user's spring ---
    pub grabbed: Option<usize>,
    pub anchor: (f64, f64, f64),

    // --- thermostat (off by default) ---
    pub thermostat_on: bool,
    pub target_temperature: f64,
    pub thermostat_tau: f64,

    // --- THE LEDGER ---
    pub e_kin: f64,
    pub e_pair: f64,
    /// The many-body sector: the sum of the tabulated three-body term over every triple
    /// inside the table's domain. Its OWN ledger row, never folded into `e_pair` — one
    /// reader per term, because a combined number cannot say which sector moved.
    pub e_three: f64,
    /// The four-body sector: exact ab-initio (O,H,H,H) valence term.
    pub e_four: f64,
    /// THE LONG-RANGE SECTOR (GANTT node B2): the pair tail past `R_s`, summed to a
    /// declared budget and, in a wrapping box, over image shells.
    ///
    /// Its OWN ledger row, following the four-body sector's pattern for the reason that
    /// sector gives: one reader per term, because a combined number cannot say which
    /// sector moved. Exactly 0.0 when no far sector is declared, so every scene that
    /// existed before B2 is bit-unchanged — an exact zero added to a finite float changes
    /// no bit, and the replay fingerprints stay valid.
    pub e_far: f64,
    /// The far sector itself, absent until one is declared. Boxed because it carries a
    /// tail model per bank slot and an image lattice, and a `Sim` that never uses one
    /// should not grow by them.
    pub far: Option<Box<crate::longrange::FarSector>>,
    /// What the last force pass's far sector computed — channels, counts, virial and the
    /// residual bound. Read by B2's gates; carries no state the physics depends on.
    pub far_reading: crate::longrange::FarReading,
    /// Total angular momentum at reset, and the peak residual since. `L` is conserved only
    /// where the box permits it (see [`Sim::angular_gate`]), which is why this is tracked
    /// beside the momentum ledger rather than folded into it.
    pub l0_ang: (f64, f64, f64),
    pub angular_residual_peak: f64,
    /// THE INTERNAL VIRIAL, `Σ r · dU/dr` over every interacting pair, triple side and
    /// four-body radial coordinate.
    ///
    /// The quantity the pressure is built from and the reason a barostat can exist. It is
    /// accumulated by the force loop, where the slopes already are, rather than by a second
    /// pass — a second pass would be a second reading of the same configuration and the two
    /// would be free to disagree.
    ///
    /// Walls and the user's spring are NOT in it. They are the container and the hand, not
    /// the substance, and a virial that included them would report the box pushing on
    /// itself as pressure.
    pub w_virial: f64,
    /// Whether the ab-initio 4-body (O,H,H,H) valence term is active.
    pub de4_enabled: bool,
    /// Counter of compact (O,H,H,H) encounters actually evaluated by the ab-initio solver.
    pub de4_eval_count: u64,
    pub de4_last_pos: Vec<[f64; 3]>,
    pub de4_cached_forces: Vec<(f64, f64, f64)>,
    pub de4_cached_energy: f64,
    /// The four-body sector's virial, cached alongside its energy and forces. Cached for
    /// the same reason and reused on the same condition — a cache that carried the energy
    /// and recomputed the virial would be two answers about one configuration.
    pub de4_cached_virial: f64,
    pub de4_cached_valid: bool,
    /// Per-hub warm start: the converged CI vector of each oxygen's last four-body
    /// solve. Consecutive recomputes of a barely-moved quadruple start Davidson from
    /// the answer instead of from cold.
    pub de4_ci: Vec<(usize, Vec<f64>)>,
    pub e_wall: f64,
    pub e_spring: f64,
    /// The uniform gravitational field as an ACCELERATION VECTOR, atomic units. Zero
    /// unless a caller sets it, so every scene that existed before gravity did is
    /// bit-unchanged -- which is what keeps the standing replay fingerprints valid.
    ///
    /// A VECTOR rather than a magnitude, per FSD-W2 WB-2.4c, and the reason is the tilted
    /// bucket: the field lives in the WORLD, so when the shell rotates the BOX the field's
    /// direction changes in box coordinates and the water sloshes. A scalar "down" cannot
    /// express that, because it hard-codes the one direction the box is not allowed to
    /// leave. Straight down at one G is `(0, -G_EARTH_AU, 0)`.
    ///
    /// A SETTING, not a derived quantity. That is why this is the field the checkpoint
    /// carries and `e_grav` is not: `checkpoint.rs` stores state and RECOMPUTES energies,
    /// so storing `e_grav` would be storing the same fact twice and inviting the copies
    /// to disagree. See [`Sim::set_gravity`] for the one boundary that refuses it.
    pub g_vec: (f64, f64, f64),
    /// The field's potential energy, `sum_i m_i g y_i`, hartree.
    ///
    /// The zero is the box's LOWER FACE (`y = 0`), which is the force law's own zero
    /// rather than a convenience: a potential measured from a different origin than the
    /// force that integrates it reads in the balance gate as an unexplained loss. Derived
    /// from the positions by `compute_forces`, never stored.
    pub e_grav: f64,
    /// Every joule the outside world put in: anchor motion, spring teardown on release,
    /// thermostat rescaling, and the barostat's box work. The intervention is a term in
    /// the ledger, never outside it.
    ///
    /// This is the TOTAL. Which intervention moved it is [`Sim::work`], and the two are
    /// tied together by [`Sim::work_columns_ok`] rather than by trust — see that gate.
    pub w_ext: f64,
    /// WB-4.3 — THE RECEIPT COLUMNS. `w_ext` says the ledger closed; this says who paid.
    ///
    /// A single total closes the balance gate and answers no question: a run that gained
    /// energy from the hand and lost the same amount to the thermostat reads identically
    /// to a run nobody touched. The hand is a moving boundary condition the user drives,
    /// so it gets its own column and the balance gate still closes over it — conservation
    /// is chart-relative and the hand is part of the chart.
    pub work: ExternalWork,
    /// The ledger's invariant at reset. `ledger() - w_ext` must equal this forever.
    pub l0: f64,
    /// Total momentum at reset, and the external impulse since.
    pub p0: (f64, f64, f64),
    pub j_ext: (f64, f64, f64),

    pub time: f64,
    pub steps: u64,

    // --- running maxima that define the drift bound (set by the trajectory, not by hand) ---
    k_pair_max: f64,
    /// Largest PER-ATOM summed three-body stiffness the force loop has actually
    /// evaluated. See [`Sim::k_three`] for the derivation. LIVE, like `k_pair_max`, and
    /// for the same reason: a static envelope taken from the table alone cannot know which
    /// triples the trajectory brings together, nor how many of them.
    k_three_max: f64,
    wall_engaged: bool,
    spring_engaged: bool,
    /// Largest energy scale the ledger has held; the bound's amplitude factor.
    pub e_ref: f64,
    pub drift_peak: f64,
    pub momentum_residual_peak: f64,

    /// Bond readings, one per neighbour pair. Heap-backed and CUTOFF-LOCAL: see
    /// [`Sim::refresh_pairs`] for why dropping the distant pairs cannot change a bond.
    pub pairs: Vec<PairReading>,
    pub pair_count: usize,

    /// THE BAROSTAT, absent until one is enabled. Boxed because a barostat carries two
    /// Nosé-Hoover chains and a `Sim` that never uses one should not grow by them.
    pub barostat: Option<Box<crate::barostat::Barostat>>,
    /// The three clocks and the degradation contract.
    pub timescale: Timescale,
    /// The composite-holon layer. Runs at grain boundaries only.
    pub holons: HolonLayer,
    /// Grain boundaries closed since reset. The holon layer's clock.
    pub frame: u64,
    /// Largest pair relative energy seen since reset — what the curvature envelope, and
    /// therefore the drift bound, is derived from.
    pub e_rel_max: f64,
}

impl Sim {
    pub const fn empty() -> Self {
        Self {
            bank: PairBank::hydrogen_seeded(),
            trimer: TrimerTable::empty(),
            water: WaterTable::empty(),
            ooh: holon_chem::ooh::OohTable::empty(),
            ozone: holon_chem::ozone::OzoneTable::empty(),
            trimers: crate::trimer_bank::TrimerBank::empty(),
            fence_untabulated: 0,
            // Empty, not sixteen-of-something. `Vec::new` allocates nothing, so
            // `Sim::empty()` stays a `const fn` and stays free; `reset` sizes the scene.
            atoms: Vec::new(),
            n: 0,
            boundary: Boundary::Walls,
            width: 40.0,
            height: 24.0,
            depth: 24.0,
            wall_inset: 0.6,
            dims: Dims::Two,
            a_pair: Vec::new(),
            a_ext: Vec::new(),
            slots: Vec::new(),
            pair_switch: None,
            pair_floor: 0.0,
            executor: None,
            pair_terms: Vec::new(),
            triple_terms: Vec::new(),
            triple_scratch: Vec::new(),
            k_atom_scratch: Vec::new(),
            quad_force_scratch: Vec::new(),
            cells: crate::cells::CellList::empty(),
            neighbours: crate::cells::Neighbours::empty(),
            grabbed: None,
            anchor: (0.0, 0.0, 0.0),
            thermostat_on: false,
            target_temperature: 300.0,
            thermostat_tau: 2000.0,
            e_kin: 0.0,
            e_pair: 0.0,
            e_three: 0.0,
            e_four: 0.0,
            w_virial: 0.0,
            de4_enabled: false,
            de4_eval_count: 0,
            de4_last_pos: Vec::new(),
            de4_cached_forces: Vec::new(),
            de4_cached_energy: 0.0,
            de4_cached_virial: 0.0,
            de4_cached_valid: false,
            de4_ci: Vec::new(),
            e_wall: 0.0,
            e_spring: 0.0,
            g_vec: (0.0, 0.0, 0.0),
            e_grav: 0.0,
            w_ext: 0.0,
            work: ExternalWork::zero(),
            l0: 0.0,
            p0: (0.0, 0.0, 0.0),
            j_ext: (0.0, 0.0, 0.0),
            time: 0.0,
            steps: 0,
            k_pair_max: 0.0,
            k_three_max: 0.0,
            wall_engaged: false,
            spring_engaged: false,
            e_ref: 0.0,
            drift_peak: 0.0,
            momentum_residual_peak: 0.0,
            pairs: Vec::new(),
            pair_count: 0,
            e_far: 0.0,
            far: None,
            far_reading: crate::longrange::FarReading {
                energy: 0.0,
                virial: 0.0,
                contributions: 0,
                image_contributions: 0,
                channel_s: 0.0,
                channel_t: 0.0,
                crossings: 0,
                residual_bound: 0.0,
                plant_carrier: 0.0,
                box_illegal: false,
                shells_unresolved: false,
                shells: 0,
            },
            l0_ang: (0.0, 0.0, 0.0),
            angular_residual_peak: 0.0,
            barostat: None,
            timescale: Timescale::empty(),
            holons: HolonLayer::empty(),
            frame: 0,
            e_rel_max: f64::NEG_INFINITY,
        }
    }

    /// The integration step in force. Derived from the curve by `Timescale`, never a
    /// constant in this file.
    pub fn dt(&self) -> f64 {
        self.timescale.dt
    }

    // ------------------------------------------------------------ the bank, read

    /// THE SINGLE-CURVE VIEW: the first loaded curve in the bank.
    ///
    /// For a pure scene this IS the scene's curve, which is what keeps every reading the
    /// sandbox has ever shown — `r_e`, `d_e`, the asymptote, the plotted curve, the
    /// residual on the banner — the same number it was before the bank existed.
    ///
    /// For a MIXED scene it is one of several, and nothing dynamical may read it. The
    /// force loop, the bond criterion, the drift bound and the timescale all go through
    /// [`Sim::table_for`] or iterate the active slots instead. A mixed scene's viewer says
    /// which pair this curve belongs to rather than implying there is only one.
    pub fn table(&self) -> &PotentialTable {
        self.bank.primary()
    }

    /// The LEGACY DOOR: slot 0, which is the H-H pair.
    ///
    /// `Sim::empty` seeds hydrogen as species 0, so slot 0 is the pair the single-table
    /// sandbox always simulated, and every existing caller that loads "the table" —
    /// `json::load_into`, the ABI's knot pusher, the tests' fixtures — keeps loading the
    /// curve it was loading. A write through here declares no provenance, which
    /// [`Sim::provenance_ok`] reports as `Route::Undeclared` rather than treating as fine.
    pub fn table_mut(&mut self) -> &mut PotentialTable {
        self.bank.table_slot_mut(0)
    }

    /// The curve for the pair of atoms `i` and `j`.
    ///
    /// The lookup is by SPECIES SLOT, resolved once per force evaluation into
    /// [`Sim::species_slots`] rather than per pair, because the inner loop runs `N^2/2`
    /// times and the species list does not change inside it.
    pub fn table_for(&self, slots: &[usize], i: usize, j: usize) -> &PotentialTable {
        self.bank.table_at(slots[i], slots[j])
    }

    /// Each atom's index into the bank's species list.
    ///
    /// Computed fresh on every force evaluation rather than cached on the atom. The cache
    /// would be one more thing that can be stale, and a stale species index does not read
    /// as an error — it reads as the wrong curve, quietly, which is precisely the defect
    /// plant (i) fires on. At `N <= 16` atoms over `<= 6` species this is at most 96
    /// integer compares against a force loop that evaluates cubic Hermite interpolants.
    ///
    /// An atom whose species is not registered maps to slot 0. That case cannot reach the
    /// force loop: [`Sim::pairs_ready`] refuses to step a scene with an unregistered
    /// species, because slot 0 would be some other pair's curve.
    pub fn species_slots(&self) -> Vec<usize> {
        let mut out = vec![0usize; self.n];
        for i in 0..self.n {
            out[i] = self.bank.index_of(self.atoms[i].species.z).unwrap_or(0);
        }
        out
    }

    /// [`Sim::species_slots`] into the reusable scratch buffer.
    ///
    /// Same computation, same freshness — it is recomputed from the atoms on every force
    /// evaluation, so it cannot go stale — and no allocation per evaluation. The buffer is
    /// scratch and is never read except immediately after this writes it.
    fn refresh_slots(&mut self) {
        self.slots.clear();
        self.slots.reserve(self.n);
        for i in 0..self.n {
            let s = self.bank.index_of(self.atoms[i].species.z).unwrap_or(0);
            self.slots.push(s);
        }
    }

    /// Register every species the scene currently holds. `false` if the scene needs more
    /// distinct species than the bank can hold — a REFUSAL, never a silent reuse.
    pub fn sync_species(&mut self) -> bool {
        for i in 0..self.n {
            if self.bank.register(self.atoms[i].species.z).is_none() {
                return false;
            }
        }
        true
    }

    /// The slots this scene's ATOMS actually use, deduplicated.
    ///
    /// Derived from the atoms rather than from the bank's registration list, and that is
    /// the load-bearing part: a species that has been registered but has no atom in the
    /// scene contributes no pair, and a bound taken over its curve would be a bound for a
    /// collision that cannot happen. "Active" is a fact about the scene.
    pub fn active_slots(&self) -> ([usize; crate::bank::MAX_TABLES], usize) {
        let mut out = [0usize; crate::bank::MAX_TABLES];
        let mut n = 0usize;
        let slots = self.species_slots();
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                let s = self.bank.slot(slots[i], slots[j]);
                if !out[..n].contains(&s) {
                    out[n] = s;
                    n += 1;
                }
            }
        }
        // A one-atom scene has no pair and therefore no active curve. A scene with no
        // atoms likewise. Both are handled by `n == 0` at every call site rather than by
        // inventing a curve neither of them uses.
        (out, n)
    }

    /// Whether every pair the scene contains has a curve to be evaluated on.
    ///
    /// This is the bank's version of `table.is_loaded()`, and it replaces it in
    /// [`Sim::step`]. The old test asked whether THE table was loaded; in a mixed scene
    /// the question is whether EVERY active pair's table is, and a scene missing one would
    /// otherwise integrate the pairs it can and silently apply no force to the rest.
    pub fn pairs_ready(&self) -> bool {
        for i in 0..self.n {
            if self.bank.index_of(self.atoms[i].species.z).is_none() {
                return false;
            }
        }
        let (slots, n) = self.active_slots();
        if n == 0 {
            // No pairs: nothing to evaluate, and the single-atom scene is not "not ready".
            // It is still gated on a loaded primary curve, exactly as it was, so a scene
            // with no curve at all does not start stepping.
            return self.table().is_loaded();
        }
        slots[..n].iter().all(|&s| self.bank.is_filled(s))
    }

    /// Whether every loaded curve's provenance was admitted by the gate.
    pub fn provenance_ok(&self, host: crate::bank::Host) -> bool {
        self.bank.provenance_admitted(&crate::bank::D1_RECORD, host)
    }

    /// The first provenance refusal in the bank, if there is one.
    pub fn provenance_refusal(
        &self,
        host: crate::bank::Host,
    ) -> Option<(usize, crate::bank::Refusal)> {
        self.bank.first_refusal(&crate::bank::D1_RECORD, host)
    }

    /// The deepest well among the curves this scene actually uses, hartree.
    ///
    /// The amplitude factor in the drift bound and the bond-depth scale the holon layer
    /// reads. `table().d_e` served both when there was one curve; in a mixed scene the
    /// bound must cover the deepest well any active pair can fall into, so it is a MAX
    /// over the active slots. With one active slot it is that slot's `d_e`, bit for bit.
    pub fn active_d_e(&self) -> f64 {
        let (slots, n) = self.active_slots();
        if n == 0 {
            return self.table().d_e;
        }
        let mut d = 0.0f64;
        for &s in slots[..n].iter() {
            let v = self.bank.table_slot(s).d_e.abs();
            if v > d {
                d = v;
            }
        }
        d
    }

    /// Re-derive every clock from the curves the scene actually uses. Call after loading.
    ///
    /// # The criterion is the FASTEST MODE, not the stiffest curve
    ///
    /// `dt` exists to resolve a vibration, and a vibration's frequency is
    /// `sqrt(k_e / mu)` — so the pair that constrains the timestep is the one maximising
    /// THAT, not the one with the largest `k_e`. The two differ in a mixed scene by
    /// exactly the mass ratio: a Cl-Cl bond is stiffer than an H-H bond and oscillates far
    /// more slowly, because chlorine is 35 times heavier. Picking on stiffness alone would
    /// hand a hydrogen-bearing scene chlorine's clock and under-resolve the fastest thing
    /// in the box.
    ///
    /// It is also what makes plant (ii) fire: run chlorine at hydrogen's mass and every
    /// `mu` containing a chlorine drops by the mass ratio, so the derived `dt` moves by
    /// its square root — a quantity computed here, not asserted anywhere.
    ///
    /// With ONE active pair this reduces to what it always was: `mu` is that pair's
    /// reduced mass, computed by the same `(mi*mj)/(mi+mj)` in the same order, and the
    /// curve is that pair's curve. A pure-hydrogen scene therefore gets the identical
    /// float.
    pub fn adopt_table_timescale(&mut self) {
        let species = self.species_slots();

        // The reduced mass of every ACTIVE pair type, alongside its slot. Pair types, not
        // pairs: every H-Cl pair in the box has the same reduced mass and the same curve.
        let mut best: Option<(usize, f64, f64)> = None; // (slot, mu, omega^2)
        let mut mu_min = f64::INFINITY;
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                let mi = self.atoms[i].mass();
                let mj = self.atoms[j].mass();
                let mu = (mi * mj) / (mi + mj);
                if mu < mu_min {
                    mu_min = mu;
                }
                let slot = self.bank.slot(species[i], species[j]);
                let t = self.bank.table_slot(slot);
                if !t.is_loaded() {
                    continue;
                }
                let k_e = t.curvature(t.r_e).abs();
                let omega_sq = k_e / mu;
                if best.map_or(true, |(_, _, w)| omega_sq > w) {
                    best = Some((slot, mu, omega_sq));
                }
            }
        }

        let (slot, mu) = match best {
            Some((slot, mu, _)) => (slot, mu),
            // No loaded active pair: fall back to the primary curve and the two-body
            // reduced mass the single-table sandbox used, so a scene that has not been
            // populated yet behaves exactly as it did.
            None => {
                let mu = if self.n >= 2 {
                    let m0 = self.atoms[0].mass();
                    let m1 = self.atoms[1].mass();
                    (m0 * m1) / (m0 + m1)
                } else {
                    0.5 * M_H
                };
                (self.bank.primary_slot(), mu)
            }
        };
        if !mu_min.is_finite() {
            mu_min = mu;
        }

        // Field-level borrow split: `from_table` needs the timescale mutably and one of
        // the bank's curves immutably, and those are disjoint fields of `self`.
        let Sim { bank, timescale, .. } = self;
        timescale.from_table(bank.table_slot(slot), mu);
        // `from_table` seeds the envelope from ONE curve. The envelope over ALL active
        // curves — what the freeze asks the drift bound to be built from — is taken by
        // `refresh_envelope` below.
        timescale.mu_min = mu_min;
        self.refresh_envelope();
    }

    /// Set the species for atom `i`, registering it with the bank.
    ///
    /// Returns `false` if the bank is full — a REFUSAL, and the species is not applied.
    /// Silently accepting a seventh species would leave an atom resolving to slot 0 and
    /// being served hydrogen's curve, which is plant (i)'s defect arriving through the
    /// front door.
    pub fn set_species(&mut self, i: usize, species: Species) -> bool {
        if i >= self.n {
            return false;
        }
        if self.bank.register(species.z).is_none() {
            return false;
        }
        self.atoms[i].species = species;
        true
    }

    /// The scene's MODE-ENERGY scale: the amplitude factor the drift bound needs.
    ///
    /// This is deliberately NOT `energy()`. The harmonic derivation bounds the total
    /// error by the sum over modes of each mode's own energy, and `energy()` is the
    /// SIGNED total, in which kinetic energy and (negative) bond potential cancel. In a
    /// scene with bonds they cancel almost exactly — which is precisely the situation the
    /// gate is meant to police — so the signed total tracks the CONSERVED quantity while
    /// the oscillation amplitudes it is supposed to stand for grow underneath it.
    ///
    /// Measured on the field-report repro (examples/gate_repro.rs, N = 11 walls on):
    /// `|E| = 0.49` against modes carrying 5.3 Eh, and up to 37x apart on the
    /// configuration that actually breached. Summing magnitudes is positive-definite, so
    /// no cancellation is possible.
    ///
    /// It is an OVER-estimate by construction: a pair resting at the bottom of its well
    /// carries no vibrational energy but contributes `D_e` here. That slack is bounded
    /// (one `D_e` per bonded pair) and it errs toward a wider bound, which is the safe
    /// direction for a term that multiplies a bound.
    pub fn mode_energy(&self) -> f64 {
        self.e_kin
            + self.e_pair.abs()
            + self.e_three.abs()
            + self.e_four.abs()
            + self.e_far.abs()
            + self.e_wall
            + self.e_spring
            // `.abs()` where `e_wall` and `e_spring` are added bare, and not by oversight:
            // those two are non-negative by construction, while `e_grav` goes negative for
            // an atom that has fallen below y = 0 through an open boundary. This sum has
            // to stay positive-definite -- it multiplies a bound, and a term allowed to
            // cancel against the others would narrow the bound exactly when a scene is
            // doing something extreme.
            + self.e_grav.abs()
    }

    /// Set the uniform gravitational field as a WORLD-FRAME acceleration vector, atomic
    /// units. Straight down at one G is `(0.0, -G_EARTH_AU, 0.0)`.
    ///
    /// This is the door WB-2.4c asks for and the scalar one below now delegates to it, so
    /// there is exactly one statement of what the field does to a scene. The shell holds
    /// this vector pointing world-down and rotates the BOX around it; the field's
    /// direction in box coordinates then changes, which is what a tilted bucket is.
    ///
    /// REFUSES on a wrapping box, per component. See [`GravityRefusal::PeriodicBox`]: a
    /// linear potential is not well-posed on a torus. This engine's `Boundary::Periodic`
    /// wraps ALL THREE axes, so the per-axis rule collapses to "any nonzero field is
    /// refused" -- stated rather than silently simplified, because the day an axis-wise
    /// periodic boundary exists this is the line that has to learn about it.
    ///
    /// Re-bases nothing, for the reason the scalar door did not: adding or turning a
    /// potential term moves the total energy, so a caller changing the field mid-run must
    /// `rebase()` or the drift is measured against an origin taken before this field
    /// existed and reads a JUMP no integrator produced.
    pub fn set_gravity_vec(&mut self, gx: f64, gy: f64, gz: f64) -> Result<(), GravityRefusal> {
        if !(gx.is_finite() && gy.is_finite() && gz.is_finite()) {
            return Ok(());
        }
        let nonzero = gx != 0.0 || gy != 0.0 || gz != 0.0;
        if nonzero && self.boundary.wraps() {
            return Err(GravityRefusal::PeriodicBox);
        }
        self.g_vec = (gx, gy, gz);
        self.compute_forces();
        self.accumulate_energy();
        Ok(())
    }

    /// Set the field as a DOWNWARD MAGNITUDE, atomic units -- the pre-WB-2.4c door, kept
    /// because "1 G downward" is what a viewer's slider means and re-deriving the vector
    /// at every call site is how the two spellings drift apart.
    ///
    /// Delegates. `set_gravity(g)` is exactly `set_gravity_vec(0.0, -g, 0.0)`, and
    /// `tests/gravity.rs` asserts the two produce bit-identical scenes rather than
    /// trusting the delegation.
    pub fn set_gravity(&mut self, g_au: f64) -> Result<(), GravityRefusal> {
        self.set_gravity_vec(0.0, -g_au, 0.0)
    }

    /// The field vector currently set, atomic units.
    pub fn gravity_vec(&self) -> (f64, f64, f64) {
        self.g_vec
    }

    /// The field's MAGNITUDE, atomic units. Zero when there is none.
    ///
    /// Deliberately a magnitude and not the old signed scalar: with a vector field there
    /// is no privileged axis to take a component along, and returning `-g_vec.1` would be
    /// right only while the field points down, which is precisely the assumption WB-2.4c
    /// removes.
    pub fn gravity(&self) -> f64 {
        let (x, y, z) = self.g_vec;
        (x * x + y * y + z * z).sqrt()
    }

    /// The INTERNAL force on atom `i`, hartree/bohr: the pair loop's contribution plus the
    /// triple loop's, which are the two that cancel from the momentum sum. Exposed so a
    /// gate can check that the force the integrator pushes with is minus the gradient of
    /// the energy the ledger sums — the precondition that makes an energy gate a
    /// measurement of integration error rather than of an inconsistency.
    pub fn internal_force(&self, i: usize) -> (f64, f64, f64) {
        if i < self.n {
            self.a_pair[i]
        } else {
            (0.0, 0.0, 0.0)
        }
    }

    /// The internal and external accelerations on atom `i`, for an integrator that lives
    /// outside this file. Split exactly as the force loop keeps them, because the momentum
    /// ledger's whole distinction is which of the two cancels.
    #[inline]
    pub(crate) fn a_pair_at(&self, i: usize) -> (f64, f64, f64) {
        self.a_pair[i]
    }

    #[inline]
    pub(crate) fn a_ext_at(&self, i: usize) -> (f64, f64, f64) {
        self.a_ext[i]
    }

    /// Largest pair curvature the force loop has actually evaluated since reset. Exposed
    /// so the attribution probe can separate the two halves of the drift-bound fix.
    pub fn k_pair_max(&self) -> f64 {
        self.k_pair_max
    }

    /// The three-body stiffness the drift bound uses, hartree/bohr^2: the largest
    /// PER-ATOM total the force loop has evaluated since reset.
    ///
    /// # The derivation
    ///
    /// The bound needs `|d2E/dx_i^2|` — the stiffest curvature one atom's displacement can
    /// meet. For a single triple, with `E = F(s_a, s_b, s_c)` a function of the three
    /// sides,
    ///
    /// ```text
    /// d2F/dx_i^2 = sum_{a,b} F_ab (ds_a/dx_i)(ds_b/dx_i) + sum_a F_a (d2 s_a/dx_i^2)
    /// ```
    ///
    /// Atom `i` touches exactly TWO of the three sides, `|ds_a/dx_i| <= 1` because each is
    /// a component of a unit vector, and `||d2 s_a/dx_i^2|| <= 2/s_a` for a distance. So
    /// per triple
    ///
    /// ```text
    /// |d2F/dx_i^2| <= 4 G2 + 2 sum_a |F_a| / s_a
    /// ```
    ///
    /// with the second sum taken over ALL THREE sides rather than the two at `i`, which
    /// only widens it. `G2` is the table's own second-derivative envelope, and it is taken
    /// as the SMALLER of the table's absolute cap and its local one,
    /// `curvature_per_gradient * max_a |F_a|` — both measured from the interpolant and
    /// widened. Taking the local form is what keeps the bound a reading of the
    /// configuration: a dispersed scene has tiny three-body gradients, so a tiny
    /// three-body stiffness, where the absolute cap alone would quote the compact corner's
    /// number forever.
    ///
    /// Curvatures ADD over the triples an atom belongs to, so the force loop accumulates
    /// the per-triple bound into a per-atom total and keeps the largest. Bounding instead
    /// by `C(n-1, 2)` times the worst single triple — every triple simultaneously at the
    /// worst geometry — is valid and was the first form written here; it is a factor of
    /// tens looser on any scene that is not a single compact droplet, which is a bound
    /// that cannot fail rather than a bound that says anything.
    ///
    /// Zero when no table is loaded or the scene has fewer than three atoms, so the pair
    /// bound is returned unchanged — adding an exact zero to a finite float changes no bit.
    pub fn k_three(&self) -> f64 {
        if (!self.trimer.loaded && !self.water.loaded && !self.ooh.loaded && !self.ozone.loaded && self.trimers.is_empty()) || self.n < 3 {
            return 0.0;
        }
        self.k_three_max
    }

    /// The same number, exposed under the `_max` name for the attribution probe, so it can
    /// separate the two halves of the drift bound the way it separates `k_pair_max`.
    pub fn k_three_max(&self) -> f64 {
        self.k_three_max
    }

    /// Total energy currently held by the scene.
    pub fn energy(&self) -> f64 {
        self.e_kin + self.e_pair + self.e_three + self.e_four + self.e_far + self.e_wall + self.e_spring + self.e_grav
    }

    /// The conserved quantity. `E - W_ext` is constant for an exact integrator, with or
    /// without the user's hand in the box.
    pub fn ledger(&self) -> f64 {
        self.energy() - self.w_ext
    }

    pub fn drift(&self) -> f64 {
        (self.ledger() - self.l0).abs()
    }

    /// The energy-drift bound, DERIVED rather than tuned.
    ///
    /// Velocity Verlet applied to a one-dimensional harmonic oscillator of angular
    /// frequency omega is a linear symplectic map, and it conserves EXACTLY the
    /// quadratic form
    ///
    /// ```text
    /// H~ = 1/2 v^2 + 1/2 omega^2 (1 - omega^2 dt^2 / 4) x^2
    /// ```
    ///
    /// (verified numerically against the step map before this bound was written down:
    /// the softening sits on the STIFFNESS, not on the kinetic term). The true energy
    /// is therefore E = H~ + (omega^4 dt^2 / 8) x^2, which oscillates as x^2 sweeps
    /// [0, x_max^2] and does NOT drift secularly — the whole point of a symplectic
    /// integrator, and the reason the 10k-step test asserts the same bound as a 10-step
    /// one would. Substituting x_max^2 = 2 H~ / (omega^2 (1 - omega^2 dt^2 / 4)) and
    /// E_0 = H~ / (1 - omega^2 dt^2 / 4) (the energy at the turning point) gives the
    /// peak-to-peak swing
    ///
    /// ```text
    /// |dE| / E_0 = (omega dt)^2 / 4      (exact; tight, not conservative)
    /// ```
    ///
    /// and the map is unstable for omega dt >= 2, where H~ stops being positive definite.
    ///
    /// Carrying that to this scene: `omega` is `Timescale::omega_env`, the frequency of
    /// the stiffest curvature a pair can REACH at the largest relative energy seen so far
    /// (on the relative coordinate, hence the reduced mass m/2), widened by the wall and
    /// spring stiffnesses once those have actually engaged; and `E_0` is the largest
    /// energy scale the ledger has held.
    ///
    /// Reaching rather than visiting is fence 3, and it is the whole difference between a
    /// bound that survives a collision and one that does not. A bound built from the
    /// curvature the trajectory HAS sampled reads green right up to the encounter that
    /// violates it, because the stiff part of the curve has not been touched yet. The
    /// envelope asks instead what the pair could reach on the energy it already has, so
    /// the number is valid THROUGH the collision rather than up to it.
    ///
    /// Nothing here is cached: `dt` and `omega_env` are read live on every call, so a
    /// changed timestep cannot leave a stale bound behind — there is no stored bound to
    /// go stale.
    pub fn drift_bound(&self) -> f64 {
        // Reachable curvature (the envelope, from the largest pair energy seen) OR
        // VISITED curvature (the running max the force loop has actually evaluated),
        // whichever is larger. The envelope is normally the bigger of the two, but it is
        // refreshed from pair energies sampled at grain BOUNDARIES, so a brief excursion
        // between two boundaries can be stiffer than anything the envelope knows about.
        // `k_pair_max` costs nothing — the force loop already computes every curvature it
        // maximises over — and it closes that gap.
        let mu = if self.n >= 2 {
            let mut min_mu = f64::INFINITY;
            for i in 0..self.n {
                for j in (i + 1)..self.n {
                    let mi = self.atoms[i].mass();
                    let mj = self.atoms[j].mass();
                    let pmu = (mi * mj) / (mi + mj);
                    if pmu < min_mu {
                        min_mu = pmu;
                    }
                }
            }
            if min_mu.is_finite() {
                min_mu
            } else {
                0.5 * M_H
            }
        } else {
            0.5 * M_H
        };
        // The three-body stiffness is ADDED to the pair envelope rather than maxed with
        // it: both potentials act on the same coordinate, so their curvatures add, and a
        // max would understate the sum. With no table loaded `k_three()` is an exact zero
        // and the sum is bit-for-bit the pair bound this line computed before.
        let k = self.timescale.k_env.max(self.k_pair_max) + self.k_three();
        let mut omega_sq: f64 = k / mu;
        if self.wall_engaged {
            let min_m = (0..self.n)
                .map(|i| self.atoms[i].mass())
                .fold(M_H, f64::min);
            omega_sq = omega_sq.max(K_WALL / min_m);
        }
        if self.spring_engaged {
            let m_grab = self.grabbed.map(|g| self.atoms[g].mass()).unwrap_or(M_H);
            omega_sq = omega_sq.max(K_SPRING / m_grab);
        }
        let e_ref = self.e_ref.max(self.active_d_e());
        let dt = self.dt();
        DRIFT_SAFETY * 0.25 * omega_sq * dt * dt * e_ref
    }

    pub fn energy_gate(&self) -> bool {
        self.drift_peak <= self.drift_bound()
    }

    /// THE ATTRIBUTION GATE (WB-4.3): the receipt columns sum to the total.
    ///
    /// One gate per conserved quantity is the discipline, and this is the second half of
    /// the energy one. `energy_gate` says the ledger CLOSED; this says the ledger's
    /// account of WHO MOVED IT is complete. They fail independently: a column that stops
    /// being posted leaves `w_ext` right and the attribution wrong, and no drift appears,
    /// because the total was never the thing that broke.
    ///
    /// The tolerance is roundoff on the sum — both sides accumulate the same increments,
    /// so they differ only in the order they were added, which is a few ulp of the largest
    /// column times the number of postings. A discrepancy above that is a missing column,
    /// not arithmetic.
    pub fn work_columns_ok(&self) -> bool {
        let residual = (self.w_ext - self.work.total()).abs();
        residual <= self.work_columns_bound()
    }

    /// The roundoff bound [`Sim::work_columns_ok`] compares against.
    pub fn work_columns_bound(&self) -> f64 {
        let scale = self.work.scale().max(self.w_ext.abs());
        // Each posting commits one addition to each side; `steps` bounds how many there
        // can have been, and 8 ulp per posting is the same worst-case accounting
        // `momentum_bound` uses.
        (8.0 * (self.steps.max(1) as f64) * f64::EPSILON * scale).max(f64::MIN_POSITIVE)
    }

    /// Momentum residual: `|P(t) - P(0) - J_ext(t)|`.
    ///
    /// Pairwise forces are applied as equal and opposite to the two partners, so they
    /// cancel from the total in exact arithmetic; walls and the spring do not, and their
    /// impulse is accumulated as it enters the velocities. What is left is floating-point
    /// cancellation error only.
    pub fn momentum_residual(&self) -> f64 {
        let (px, py, pz) = self.momentum();
        let dx = px - self.p0.0 - self.j_ext.0;
        let dy = py - self.p0.1 - self.j_ext.1;
        let dz = pz - self.p0.2 - self.j_ext.2;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Roundoff bound for the momentum ledger. Each step commits O(N) floating-point
    /// additions into the momentum sum, each carrying at most one unit in the last place
    /// of the running magnitude; accumulating those worst-case (rather than as the
    /// random walk they actually are) gives `8 * steps * eps * p_scale`.
    pub fn momentum_bound(&self) -> f64 {
        let mut p_scale: f64 = 0.0;
        for i in 0..self.n {
            let a = &self.atoms[i];
            p_scale += a.mass() * (a.vx * a.vx + a.vy * a.vy + a.vz * a.vz).sqrt();
        }
        let p_scale = p_scale.max(1e-12);
        8.0 * (self.steps.max(1) as f64) * f64::EPSILON * p_scale
    }

    pub fn momentum_gate(&self) -> bool {
        self.momentum_residual_peak <= self.momentum_bound()
    }

    pub fn momentum(&self) -> (f64, f64, f64) {
        let mut px = 0.0;
        let mut py = 0.0;
        let mut pz = 0.0;
        for i in 0..self.n {
            let m = self.atoms[i].mass();
            px += m * self.atoms[i].vx;
            py += m * self.atoms[i].vy;
            pz += m * self.atoms[i].vz;
        }
        (px, py, pz)
    }

    /// TOTAL ANGULAR MOMENTUM about the box origin, `Σ m_i r_i × v_i`.
    ///
    /// About the ORIGIN and not the centre of mass, deliberately: the origin is the force
    /// law's own reference — it is where `e_grav`'s potential is measured from and what
    /// `scale_box` scales toward — and a moment taken about a different point than the
    /// forces that change it reads as an unexplained torque.
    pub fn angular_momentum(&self) -> (f64, f64, f64) {
        let mut lx = 0.0;
        let mut ly = 0.0;
        let mut lz = 0.0;
        for i in 0..self.n {
            let a = &self.atoms[i];
            let m = a.mass();
            lx += m * (a.y * a.vz - a.z * a.vy);
            ly += m * (a.z * a.vx - a.x * a.vz);
            lz += m * (a.x * a.vy - a.y * a.vx);
        }
        (lx, ly, lz)
    }

    /// Whether this box conserves angular momentum at all.
    ///
    /// Four things break rotational symmetry and each breaks it for its own reason: soft
    /// walls torque whatever touches them, a uniform field picks out a direction, the
    /// user's spring is an external anchor, and the controllers move energy on a schedule
    /// the symmetry knows nothing about. A PERIODIC box is excluded too, and that one is
    /// the easy one to get wrong: it does no work and delivers no impulse, so it looks
    /// exactly like the open box to the energy and momentum ledgers — but its image
    /// lattice is not isotropic, and a rotated configuration is a different scene.
    /// `Boundary::Open` is the only boundary here that is rotationally symmetric.
    ///
    /// Stated as a precondition rather than assumed, because a gate that reads a conserved
    /// quantity where it is not conserved is measuring the boundary and calling it the
    /// engine.
    pub fn angular_conserved(&self) -> bool {
        matches!(self.boundary, Boundary::Open)
            && self.g_vec == (0.0, 0.0, 0.0)
            && self.grabbed.is_none()
            && !self.thermostat_on
            && !self.barostat_on()
    }

    /// `|L(t) − L(0)|`. Meaningful only where [`Sim::angular_conserved`] holds.
    pub fn angular_residual(&self) -> f64 {
        let (lx, ly, lz) = self.angular_momentum();
        let dx = lx - self.l0_ang.0;
        let dy = ly - self.l0_ang.1;
        let dz = lz - self.l0_ang.2;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Roundoff bound for the angular ledger, built the way `momentum_bound` is: each step
    /// commits O(N) additions into the sum, each worth at most one unit in the last place
    /// of the running magnitude, accumulated worst-case rather than as the random walk it
    /// actually is.
    ///
    /// The scale carries a LEVER ARM as well as a momentum, because `L` is a moment: the
    /// same velocity error at twice the radius is twice the angular error, and a bound
    /// built from `p_scale` alone would tighten as the scene expands, which is backwards.
    pub fn angular_bound(&self) -> f64 {
        let mut l_scale: f64 = 0.0;
        for i in 0..self.n {
            let a = &self.atoms[i];
            let r = (a.x * a.x + a.y * a.y + a.z * a.z).sqrt();
            let v = (a.vx * a.vx + a.vy * a.vy + a.vz * a.vz).sqrt();
            l_scale += a.mass() * r * v;
        }
        let l_scale = l_scale.max(1e-12);
        8.0 * (self.steps.max(1) as f64) * f64::EPSILON * l_scale
    }

    /// THE ANGULAR-MOMENTUM GATE (node B2, G6), independent of the energy and momentum ones
    /// by construction.
    ///
    /// The independence is the whole reason it exists. A pairwise force that is equal and
    /// opposite but NOT central conserves linear momentum exactly and destroys angular
    /// momentum: `momentum_gate` cannot see that, and a green `momentum_gate` read as
    /// covering it is one gate vouching for a conserved quantity it never constrained.
    /// Plant P3 is the demonstration — it fires this gate while the momentum gate stays
    /// green.
    ///
    /// Returns `None` where the box does not conserve `L`, so a caller has to handle
    /// "not applicable" rather than receiving a `true` it can mistake for a pass.
    pub fn angular_gate(&self) -> Option<bool> {
        if !self.angular_conserved() {
            return None;
        }
        Some(self.angular_residual_peak <= self.angular_bound())
    }

    /// The box volume, bohr³.
    #[inline]
    pub fn volume(&self) -> f64 {
        self.width * self.height * self.depth
    }

    /// THE INSTANTANEOUS PRESSURE by the virial theorem, hartree per bohr³.
    ///
    /// ```text
    /// P = (2K − Σ r·dU/dr) / (3V)
    /// ```
    ///
    /// The kinetic half is the ideal-gas term and the virial half is the interactions'.
    /// A repulsive contact has `dU/dr < 0`, so it ADDS to the pressure; an attractive tail
    /// has `dU/dr > 0` and subtracts. That sign is the one thing worth checking by hand
    /// before believing any barostat, and `tests/t3_barostat.rs` checks it on an ideal gas
    /// (where the virial is exactly zero and `P V = N k T` must come out) before it checks
    /// anything else.
    ///
    /// MEANINGFUL ONLY IN A PERIODIC BOX. With walls, the container carries part of the
    /// momentum flux and the virial above is missing it, so the number would be a pressure
    /// with a term left out. `Sim::pressure_defined` says so rather than leaving the caller
    /// to find out.
    pub fn pressure(&self) -> f64 {
        let v = self.volume();
        if !(v > 0.0) {
            return 0.0;
        }
        (2.0 * self.e_kin - self.w_virial) / (3.0 * v)
    }

    /// Whether [`Sim::pressure`] is a pressure. False under walls, where the container
    /// carries flux the internal virial does not see.
    pub fn pressure_defined(&self) -> bool {
        !self.boundary.has_walls()
    }

    /// Kinetic temperature by equipartition: `E_kin = (dof/2) N k_B T`.
    ///
    /// DIMENSION-DEPENDENT, and one of only two places in this file that is. The
    /// degrees of freedom are the scene's, not the state vector's: a 2D scene has two
    /// per atom even though the integrator carries three components, because the third
    /// is frozen at zero and a frozen coordinate holds no thermal energy. At `dof = 2`
    /// the factor `0.5 * dof` is exactly `1.0`, so the 2D reading is the same float it
    /// has always been.
    pub fn temperature(&self) -> f64 {
        if self.n == 0 {
            return 0.0;
        }
        self.e_kin / (0.5 * self.dims.dof() * self.n as f64 * K_B)
    }

    /// Size every per-atom buffer to `n` atoms, preserving the atoms that survive.
    ///
    /// The one place `n` is written. Growing appends default (hydrogen, at the origin,
    /// at rest) atoms, which `reset` then places; shrinking truncates. Every buffer is
    /// resized here so that `atoms.len() == a_pair.len() == ... == n` is an invariant
    /// with ONE maintainer instead of five, and [`Sim::storage_ok`] states it as a fact
    /// a gate can check.
    ///
    /// M-CHEAPER-THAN-ITS-PRICE: `Vec::resize` reuses the allocation when the capacity is
    /// already there, so a scene that is reset repeatedly at one size allocates once.
    pub fn resize_storage(&mut self, n: usize) {
        self.n = n;
        self.atoms.resize(n, Atom::default());
        self.a_pair.resize(n, (0.0, 0.0, 0.0));
        self.a_ext.resize(n, (0.0, 0.0, 0.0));
        self.de4_last_pos.resize(n, [0.0; 3]);
        self.de4_cached_forces.resize(n, (0.0, 0.0, 0.0));
        self.de4_cached_valid = false;
        self.de4_ci.clear();
        self.slots.resize(n, 0);
        // The pair sector is cutoff-local and is rebuilt from the cell list; it carries no
        // per-atom entry to preserve, so it is cleared rather than resized.
        self.pairs.clear();
        self.pair_count = 0;
    }

    /// The storage invariant, as a checkable fact rather than a comment.
    ///
    /// Every per-atom buffer holds exactly `n` entries. A gate calls this because the
    /// failure mode of heap-backed state is a buffer that was grown in one place and not
    /// another, and that reads as a wrong force rather than as an error.
    pub fn storage_ok(&self) -> bool {
        self.atoms.len() == self.n
            && self.a_pair.len() == self.n
            && self.a_ext.len() == self.n
            && self.de4_last_pos.len() == self.n
            && self.de4_cached_forces.len() == self.n
            && self.slots.len() == self.n
    }

    // ------------------------------------------------------- the box, and what wraps

    /// The box as the separation arithmetic sees it. ONE constructor, so no loop can
    /// disagree with another about whether the world wraps.
    #[inline]
    pub fn geom(&self) -> crate::cells::BoxGeom {
        crate::cells::BoxGeom::new(self.width, self.height, self.depth, self.boundary.wraps())
    }

    /// Whether the periodic box can honour every cutoff the scene needs.
    ///
    /// The minimum-image convention is only the minimum image while the largest cutoff is
    /// at most HALF the shortest box edge. Past that an atom is inside the cutoff of two
    /// images of the same partner, the reduction picks one, and the missing one is a force
    /// that silently is not there. Stated as a gate rather than as a comment, because the
    /// symptom is a wrong number and not an error.
    ///
    /// Vacuously true when the boundary does not wrap: there are no images to confuse.
    pub fn pbc_ok(&self) -> bool {
        if !self.boundary.wraps() {
            return true;
        }
        let cut = self.list_cutoff();
        cut.is_finite() && cut <= 0.5 * self.geom().min_edge()
    }

    /// The half-edge the cutoffs must fit inside, and the cutoff that is actually asked
    /// for — the two numbers behind [`Sim::pbc_ok`], so a refusal can say by how much.
    pub fn pbc_margin(&self) -> (f64, f64) {
        (self.list_cutoff(), 0.5 * self.geom().min_edge())
    }

    // ------------------------------------------------------------------- the cutoffs

    /// The three-body sector's radius: the largest side length any LOADED surface's domain
    /// admits.
    ///
    /// Not a tuning parameter and not a truncation — every one of these surfaces returns
    /// an EXACT zero outside its domain, so a triple with no vertex holding two sides
    /// inside this radius contributes exactly nothing. The enumeration skipping it
    /// computes the same number for less.
    ///
    /// The domains gate on the two sides meeting at ONE vertex (water on the two O-H,
    /// (O,O,H) on the two H-O, the homonuclear surfaces on the two shortest, which share a
    /// vertex because any two sides of a triangle do). That common shape is what makes a
    /// single hub-centred radius correct for all of them.
    pub fn three_body_cutoff(&self) -> f64 {
        let mut c = 0.0f64;
        if self.trimer.loaded || !self.trimers.is_empty() {
            c = c.max(holon_chem::trimer::R_HI);
        }
        if self.water.loaded {
            c = c.max(holon_chem::water::R_HI);
        }
        if self.ooh.loaded {
            c = c.max(holon_chem::ooh::R_HI);
        }
        if self.ozone.loaded {
            c = c.max(holon_chem::ozone::R_HI);
        }
        c
    }

    /// The four-body sector's radius: the (O,H,H,H) switch's own `R_CUT`, or zero when the
    /// sector is off. Also exact — the switch is identically zero past it.
    pub fn four_body_cutoff(&self) -> f64 {
        if self.de4_enabled {
            DE4_R_CUT
        } else {
            0.0
        }
    }

    /// The radius the cell list is built at: the largest any sector needs.
    ///
    /// One decomposition serves every loop. Building one list per sector would be three
    /// passes over the scene to answer one question about it.
    pub fn list_cutoff(&self) -> f64 {
        let many = self.three_body_cutoff().max(self.four_body_cutoff());
        // THE B2 SEAM. A declared far sector hands the near sector everything up to `R_s`,
        // so the decomposition must reach that far or the split has a hole in it — which is
        // exactly the defect B1b measured, where the list radius was set by a THREE-BODY
        // table while the pair curve reached 5 bohr further.
        let many = match &self.far {
            Some(f) => many.max(f.r_s()),
            None => many,
        };
        match self.pair_switch {
            Some((_, r_cut)) => many.max(r_cut),
            None => many,
        }
    }

    /// The per-pair energy the declared truncation drops, hartree. Zero when the pair sum
    /// is complete.
    pub fn truncation_floor(&self) -> f64 {
        self.pair_floor
    }

    /// The declared truncation window, `(r_in, r_cut)`.
    pub fn pair_switch(&self) -> Option<(f64, f64)> {
        self.pair_switch
    }

    /// DERIVE a pair cutoff from the curves themselves at a declared energy budget.
    ///
    /// `floor` is the largest energy any single truncated pair may lose, hartree. The
    /// window's inner edge is the radius at which every active curve is already under it,
    /// found by bisection on the curve's own tail; the outer edge is one switch width
    /// further out. So the number is READ OFF the potential rather than chosen: change the
    /// budget and the radius moves, change the curve and the radius moves.
    ///
    /// `None` when no curve is loaded, or when even the last knot is above the budget (a
    /// budget that no truncation can meet is refused rather than rounded up to one).
    pub fn derive_pair_cutoff(&self, floor: f64) -> Option<(f64, f64)> {
        if !(floor > 0.0) {
            return None;
        }
        let (slots, ns) = self.active_slots();
        let mut r_in = 0.0f64;
        let mut any = false;
        for &s in slots[..ns].iter() {
            let t = self.bank.table_slot(s);
            if !t.is_loaded() {
                continue;
            }
            any = true;
            let base = t.r_max();
            if t.u(base).abs() <= floor {
                // Already under budget at the last knot: the tail is all that is left.
                r_in = r_in.max(base);
                continue;
            }
            // The tail is a decaying exponential past the last knot, so `|u|` is monotone
            // there and bisection is exact to the bracket. Walk out in doublings until the
            // budget is met, then halve in.
            let mut hi = base + 1.0;
            let mut guard = 0;
            while t.u(hi).abs() > floor && guard < 64 {
                hi = base + (hi - base) * 2.0;
                guard += 1;
            }
            if t.u(hi).abs() > floor {
                return None;
            }
            let mut lo = base;
            for _ in 0..80 {
                let mid = 0.5 * (lo + hi);
                if t.u(mid).abs() > floor {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            r_in = r_in.max(hi);
        }
        if !any {
            return None;
        }
        Some((r_in, r_in + PAIR_SWITCH_WIDTH))
    }

    /// Declare a pair truncation at `floor` hartree per pair. `false` if none can be
    /// derived, or if the periodic box is too small to honour the resulting cutoff.
    ///
    /// The refusal is the point: a periodic box narrower than twice the cutoff cannot
    /// carry the minimum image, and quietly shrinking the cutoff to fit would replace a
    /// declared truncation budget with an undeclared one.
    pub fn set_pair_cutoff(&mut self, floor: f64) -> bool {
        let Some((r_in, r_cut)) = self.derive_pair_cutoff(floor) else {
            return false;
        };
        let before = self.pair_switch;
        let before_floor = self.pair_floor;
        self.pair_switch = Some((r_in, r_cut));
        self.pair_floor = floor;
        if !self.pbc_ok() {
            self.pair_switch = before;
            self.pair_floor = before_floor;
            return false;
        }
        self.compute_forces();
        self.accumulate_energy();
        true
    }

    /// Return to the complete pair sum. The scene stops being `O(N)` in the pair sector
    /// and stops truncating; both halves of that trade are the caller's to make.
    pub fn clear_pair_cutoff(&mut self) {
        self.pair_switch = None;
        self.pair_floor = 0.0;
        self.compute_forces();
        self.accumulate_energy();
    }

    /// Rebuild the cell decomposition and the neighbour list for the current positions.
    fn rebuild_neighbours(&mut self) {
        let geom = self.geom();
        let cut = self.list_cutoff();
        // No sector has a finite radius (no three-body table, no four-body, no declared
        // pair cutoff): there is nothing for a cell list to accelerate, so it is not
        // built. `Neighbours` stays empty and the complete pair loop runs, which is
        // exactly the pre-T3 engine.
        if !(cut > 0.0) || !cut.is_finite() {
            self.cells.rebuild(&[], geom, f64::INFINITY);
            self.neighbours.pairs.clear();
            self.neighbours.start.clear();
            self.neighbours.start.resize(self.n + 1, 0);
            self.neighbours.cutoff = f64::INFINITY;
            self.neighbours.complete = false;
            self.neighbours.route = crate::cells::Route::Complete;
            return;
        }
        let mut cells = core::mem::take(&mut self.cells);
        let mut nb = core::mem::take(&mut self.neighbours);
        cells.rebuild(&self.atoms, geom, cut);
        cells.build_neighbours(&self.atoms, &mut nb);
        self.cells = cells;
        self.neighbours = nb;
    }

    /// Whether the last force pass's far sector actually summed, or refused.
    ///
    /// It can refuse for a reason that arose AFTER the sector was admitted: `Sim::scale_box`
    /// shrinks the box affinely and nothing re-checks any legality condition afterwards —
    /// not the far sector's `min_edge >= 2 R_s`, and not [`Sim::pbc_ok`] either, which is
    /// consulted only by [`Sim::set_pair_cutoff`]. So this is asked per pass rather than
    /// once at admission.
    pub fn far_ok(&self) -> bool {
        self.far.is_none()
            || (!self.far_reading.box_illegal && !self.far_reading.shells_unresolved)
    }

    /// Which enumeration the last force evaluation ran. Reported so a caller can see
    /// whether the decomposition actually engaged rather than assuming it did.
    pub fn route(&self) -> crate::cells::Route {
        self.cells.route()
    }

    /// Demand the COMPLETE enumeration whatever the geometry admits — the reference route
    /// the local one is audited against. See [`crate::cells::RoutePolicy`].
    pub fn force_complete_route(&mut self) {
        self.cells.set_policy(crate::cells::RoutePolicy::Complete);
        self.recompute();
    }

    /// Let the geometry choose again.
    pub fn auto_route(&mut self) {
        self.cells.set_policy(crate::cells::RoutePolicy::Auto);
        self.recompute();
    }

    /// Install a truncation window WITHOUT re-deriving it — the restore path, where the
    /// window is being read back rather than computed.
    ///
    /// Separate from [`Sim::set_pair_cutoff`] on purpose: that one DERIVES the window from
    /// the curves and refuses one the box cannot hold, which is exactly right when a caller
    /// declares a budget and exactly wrong when a checkpoint is restoring a window that was
    /// already derived and already accepted. A restore that silently re-derived would put
    /// the scene on a different truncation than the one the run used.
    pub(crate) fn set_pair_switch_raw(&mut self, switch: Option<(f64, f64)>, floor: f64) {
        self.pair_switch = switch;
        self.pair_floor = floor;
    }

    /// The envelope state the drift bound is built from, as raw words for the checkpoint.
    ///
    /// These are HISTORY, not configuration: `k_pair_max` and `k_three_max` are the
    /// stiffest curvatures the trajectory has actually met, and the two `engaged` flags say
    /// whether the wall and the spring have ever acted. A restore that dropped them would
    /// hand the continuation a NARROWER bound than the run had earned, and the energy gate
    /// would start reporting a breach that the original run had already accounted for.
    pub(crate) fn envelope_state(&self) -> [f64; crate::checkpoint::ENVELOPE_WORDS] {
        [
            self.k_pair_max,
            self.k_three_max,
            if self.wall_engaged { 1.0 } else { 0.0 },
            if self.spring_engaged { 1.0 } else { 0.0 },
        ]
    }

    pub(crate) fn set_envelope_state(&mut self, w: [f64; crate::checkpoint::ENVELOPE_WORDS]) {
        self.k_pair_max = w[0];
        self.k_three_max = w[1];
        self.wall_engaged = w[2] != 0.0;
        self.spring_engaged = w[3] != 0.0;
    }

    /// The clock's state as raw words. `dt` is in here rather than re-derived because a
    /// re-derivation reads the envelope, and the envelope is history — see above.
    pub(crate) fn clock_state(&self) -> [f64; crate::checkpoint::CLOCK_WORDS] {
        let t = &self.timescale;
        [
            t.mu,
            t.mu_min,
            t.omega_e,
            t.period,
            t.dt_reference,
            t.dt,
            t.k_env,
            t.omega_env,
            t.r_inner,
            t.e_rel_max,
            t.sim_speed_fs_per_wallsec,
            t.accumulator(),
            t.substeps_per_second,
            t.dilation,
        ]
    }

    pub(crate) fn set_clock_state(&mut self, w: [f64; crate::checkpoint::CLOCK_WORDS]) {
        let t = &mut self.timescale;
        t.mu = w[0];
        t.mu_min = w[1];
        t.omega_e = w[2];
        t.period = w[3];
        t.dt_reference = w[4];
        t.dt = w[5];
        t.k_env = w[6];
        t.omega_env = w[7];
        t.r_inner = w[8];
        t.e_rel_max = w[9];
        t.sim_speed_fs_per_wallsec = w[10];
        t.set_accumulator(w[11]);
        t.substeps_per_second = w[12];
        t.dilation = w[13];
    }

    /// Recompute the forces and the energy from the current state, leaving the ledger's
    /// origin alone.
    ///
    /// `rebase` does this and also re-zeroes the ledger, which is the wrong thing after a
    /// change that is not meant to be a new initial condition.
    pub fn recompute(&mut self) {
        self.compute_forces();
        self.accumulate_energy();
    }

    /// The neighbour pairs of the last force evaluation.
    pub fn neighbours(&self) -> &crate::cells::Neighbours {
        &self.neighbours
    }

    /// PLANT P-2 — THE PERIODIC TRANSLATION GATE.
    ///
    /// Translate every atom by one box vector, recompute the energy, and report how far it
    /// moved. Under a correct minimum-image convention the answer is EXACTLY zero: the
    /// periodic box has no origin, so where the scene sits inside it is not a physical
    /// fact and cannot reach a number. Under walls or an open box it is large, because
    /// those boundaries DO have an origin — which is what makes this a gate rather than a
    /// tautology (M-VACUOUS-SUCCESS: a check that passes in every configuration has
    /// checked nothing).
    ///
    /// # The float precondition, checked rather than assumed
    ///
    /// "Bit-identical" is a claim about arithmetic, and translating by `L` is only exact
    /// when `x + L` is representable — otherwise the atom is not one box away, it is one
    /// box away plus a rounding error, and the energies differ by what that error is
    /// worth. So the shift is verified (`(x + L) - L == x`) on every atom before the
    /// energies are compared, and the residual is `f64::NAN` when it fails. A gate that
    /// silently accepted an inexact shift would be reporting the rounding of its own
    /// harness as a property of the engine.
    ///
    /// The scene is restored exactly: positions are written back from the saved copy, not
    /// un-translated.
    pub fn pbc_translation_residual(&mut self) -> f64 {
        if self.n == 0 {
            return 0.0;
        }
        self.compute_forces();
        self.accumulate_energy();
        let before = self.energy();
        let saved: Vec<Atom> = self.atoms.clone();
        let (lx, ly, lz) = (self.width, self.height, self.depth);
        let mut exact = true;
        for i in 0..self.n {
            let (x, y, z) = (self.atoms[i].x, self.atoms[i].y, self.atoms[i].z);
            let (sx, sy, sz) = (x + lx, y + ly, z + lz);
            if (sx - lx) != x || (sy - ly) != y || (sz - lz) != z {
                exact = false;
            }
            self.atoms[i].x = sx;
            self.atoms[i].y = sy;
            self.atoms[i].z = sz;
        }
        self.compute_forces();
        self.accumulate_energy();
        let after = self.energy();
        self.atoms.copy_from_slice(&saved);
        self.compute_forces();
        self.accumulate_energy();
        if !exact {
            return f64::NAN;
        }
        (after - before).abs()
    }

    /// Place `n` atoms and zero the ledger. Deterministic: no RNG, so a reported run can
    /// be re-run byte-for-byte.
    pub fn reset(&mut self, n: usize) {
        self.resize_storage(n);
        // Register whatever species the scene is carrying before anything asks the bank
        // for a slot. An unregistered species resolves to slot 0, which is some OTHER
        // pair's curve, so this has to happen first rather than at the first lookup.
        self.sync_species();
        self.grabbed = None;
        self.thermostat_on = false;
        let cx = 0.5 * self.width;
        let cy = 0.5 * self.height;
        // The mid-plane. In `Dims::Two` this is the plane the whole scene lives on and
        // never leaves; in `Dims::Three` it is just the box's centre.
        let cz = 0.5 * self.depth;
        let three = self.dims == Dims::Three;
        for i in 0..self.n {
            let a = &mut self.atoms[i];
            a.z = cz;
            a.vz = 0.0;
            if self.n <= 2 {
                // The headline scene: two atoms drifting slowly TOWARD each other. They
                // will collide, climb the repulsive wall, and separate again without
                // ever sticking, which is the lesson the app is built around.
                //
                // The inward speed is not decoration. Two atoms placed at rest at any
                // finite separation are ALREADY bound by the criterion in
                // `refresh_pairs` — their relative energy is U(R) < 0, and classically
                // they would fall together and never escape — so a scene that opened at
                // rest would open reading BONDED and teach the opposite of the point.
                // 0.0004 bohr per atomic time unit puts the relative energy at
                // +2.4e-4 Eh, honestly above the asymptote, on the placeholder curve.
                let sign = if i == 0 { -1.0 } else { 1.0 };
                a.x = cx + sign * 5.0;
                a.y = cy;
                a.vx = -sign * 0.0004;
                a.vy = 0.0;
            } else if three {
                // The 3D counterpart of the ring: a deterministic Fibonacci SPHERE.
                // No RNG, so a reported run re-runs byte-for-byte, and near-uniform
                // spacing so no pair opens inside the repulsive wall. Velocities are
                // assigned by the expansion pass below the loop — a shell at REST
                // opens with every pair already reading BONDED (E_rel = U(R) < 0 at
                // any finite separation, and exactly ON its turning point besides),
                // which a field report rightly called out: an opener that hands out
                // bonds nobody paid for contradicts the capture plant's own lesson.
                //
                // 6 bohr is the ring's radius kept: at N = 16 the nearest-neighbour
                // spacing is ~3.4 bohr, comfortably outside the wall and inside the well.
                let n = self.n as f64;
                let golden = core::f64::consts::PI * (3.0 - 5.0f64.sqrt());
                let w = 1.0 - 2.0 * (i as f64 + 0.5) / n;
                let rho = (1.0 - w * w).max(0.0).sqrt();
                let phi = (i as f64) * golden;
                let radius = 6.0;
                a.x = cx + radius * rho * phi.cos();
                a.y = cy + radius * rho * phi.sin();
                a.z = cz + radius * w;
                a.vx = 0.0;
                a.vy = 0.0;
            } else {
                // A deterministic ring; velocities come from the expansion pass below.
                let theta = (i as f64) * core::f64::consts::TAU / (self.n as f64);
                let radius = 6.0;
                a.x = cx + radius * theta.cos();
                a.y = cy + radius * theta.sin();
                a.vx = 0.0;
                a.vy = 0.0;
            }
        }
        // THE OPENER HANDS OUT NO BONDS. For n > 2 the scene opens in uniform
        // (Hubble-style) expansion about the centre: v_i = v * (x_i - c) / R, so
        // every pairwise separation grows in proportion and pair (i, j)'s relative
        // speed is v * d_ij / R. Unboundness for every pair needs
        //     0.5 * mu * (v * d / R)^2 > |U(d)|,
        // and v is DERIVED from that inequality by scanning the actual opening
        // pairs against the loaded curve — worst pair wins, margin 1.5 on the
        // speed. No fitted constant, no distance cutoff, deterministic; and being
        // strictly unbound also clears the measure-zero at-rest boundary where a
        // pair sits exactly ON its outer turning point and the strict criterion
        // falls by solver rounding. Bonds then cost what they always cost: energy,
        // paid out through a third body, the spring, or the thermostat.
        //
        // The two-atom headline scene keeps its own deliberate approach and is
        // untouched; with no curve loaded there is no U to clear and the scene
        // stays at rest (there are no forces either).
        if self.n > 2 && self.pairs_ready() {
            let shell_r = 6.0; // both openers place atoms on a 6-bohr shell
            let species = self.species_slots();
            let mut v2_needed = 0.0f64;
            for i in 0..self.n {
                for j in (i + 1)..self.n {
                    let dx = self.atoms[j].x - self.atoms[i].x;
                    let dy = self.atoms[j].y - self.atoms[i].y;
                    let dz = self.atoms[j].z - self.atoms[i].z;
                    let d2 = dx * dx + dy * dy + dz * dz;
                    let d = d2.sqrt().max(1e-9);
                    // The pair's OWN well and the pair's OWN reduced mass. The uniform
                    // expansion speed below is then whatever clears the worst of them, so
                    // a hydrogen in a chlorine gas is not handed an escape speed derived
                    // from a well it is not in. For a pure-hydrogen scene `mu` here is
                    // `(M_H*M_H)/(M_H+M_H)`, which is bit-for-bit the `0.5 * M_H` this
                    // line used to read — checked, not assumed.
                    let mi = self.atoms[i].mass();
                    let mj = self.atoms[j].mass();
                    let mu = (mi * mj) / (mi + mj);
                    let u = self.bank.table_at(species[i], species[j]).u(d);
                    if u < 0.0 {
                        v2_needed = v2_needed.max(2.0 * (-u) * shell_r * shell_r / (mu * d2));
                    }
                }
            }
            let v = 1.5 * v2_needed.sqrt();
            for i in 0..self.n {
                let a = &mut self.atoms[i];
                a.vx = v * (a.x - cx) / shell_r;
                a.vy = v * (a.y - cy) / shell_r;
                a.vz = v * (a.z - cz) / shell_r;
            }
        }
        self.zero_ledger();
    }

    fn zero_ledger(&mut self) {
        self.w_ext = 0.0;
        self.work = ExternalWork::zero();
        self.j_ext = (0.0, 0.0, 0.0);
        self.time = 0.0;
        self.steps = 0;
        self.frame = 0;
        self.k_pair_max = 0.0;
        self.k_three_max = 0.0;
        self.wall_engaged = false;
        self.spring_engaged = false;
        self.e_ref = 0.0;
        self.drift_peak = 0.0;
        self.momentum_residual_peak = 0.0;
        self.angular_residual_peak = 0.0;
        self.holons.reset();
        self.compute_forces();
        self.accumulate_energy();
        self.l0 = self.ledger();
        self.p0 = self.momentum();
        self.l0_ang = self.angular_momentum();
        self.e_ref = self.mode_energy().max(self.active_d_e());
        self.refresh_pairs();
        // Seed the curvature envelope from the pair energies this scene actually starts
        // with, not from zero: a scene of loosely bound pairs cannot reach the wall, and
        // an envelope seeded at U = 0 would quote a bound for a collision that the
        // energy forbids.
        self.e_rel_max = f64::NEG_INFINITY;
        self.timescale.e_rel_max = f64::NEG_INFINITY;
        self.timescale.k_env = 0.0;
        self.refresh_envelope();
    }

    /// Widen the curvature envelope to cover the largest pair energy seen so far, over
    /// EVERY curve the scene's atoms can meet each other on.
    ///
    /// The freeze's C1 asks for exactly this — "the curvature envelope taken over ALL
    /// active tables" — and the reason is that the bound has to cover the stiffest
    /// encounter the scene permits, which in a mixed box need not be on the curve that set
    /// the timestep. An unloaded slot is skipped rather than contributing a zero: a zero
    /// from an empty interpolator is not a statement that the pair is soft.
    fn refresh_envelope(&mut self) {
        let mut e_max = self.e_rel_max;
        for p in &self.pairs[..self.pair_count] {
            if p.e_rel > e_max {
                e_max = p.e_rel;
            }
        }
        if self.pair_count == 0 && !e_max.is_finite() {
            e_max = 0.0;
        }
        self.e_rel_max = e_max;
        let (slots, n) = self.active_slots();
        let Sim { bank, timescale, .. } = self;
        if n == 0 {
            let t = bank.primary();
            timescale.refresh_envelope(t, e_max);
            return;
        }
        timescale.refresh_envelope_over(e_max, |e| {
            let mut k = 0.0f64;
            let mut r_inner = f64::INFINITY;
            for &s in slots[..n].iter() {
                let t = bank.table_slot(s);
                if !t.is_loaded() {
                    continue;
                }
                let (kk, rr) = t.curvature_envelope(e);
                if kk > k {
                    k = kk;
                }
                if rr < r_inner {
                    r_inner = rr;
                }
            }
            if !r_inner.is_finite() {
                r_inner = 0.0;
            }
            (k, r_inner)
        });
    }

    /// Forget every curve AND return every atom to hydrogen.
    ///
    /// `PairBank::clear` alone leaves the atoms carrying species the bank has just
    /// forgotten, and a scene in that state stops dead: `pairs_ready` refuses it, because
    /// an unregistered species resolves to slot 0 and slot 0 is some other pair's curve.
    /// Refusing is right — silently serving the wrong curve is the defect plant (i) is
    /// about — but a host that called `clear` and then wondered why nothing moved would be
    /// debugging a consistency it never agreed to maintain.
    ///
    /// So the scene-level operation does both halves. Callers that want only the bank half
    /// can still reach `sim.bank.clear()`.
    pub fn clear_bank(&mut self) {
        self.bank.clear();
        for i in 0..self.n {
            self.atoms[i].species = HYDROGEN;
        }
    }

    /// Re-take the curvature envelope at a given energy after it has been reset.
    ///
    /// The exactness-hold toggle clears `k_env` and `e_rel_max` and then needs the
    /// envelope rebuilt at the energy the scene had reached. Exposed rather than
    /// duplicated at the call site, because the "max over all active tables" rule has to
    /// live in exactly one place.
    pub fn reseed_envelope(&mut self, e_rel_max: f64) {
        self.e_rel_max = e_rel_max;
        self.timescale.e_rel_max = f64::NEG_INFINITY;
        self.timescale.k_env = 0.0;
        self.refresh_envelope();
    }

    /// ONE GRAIN BOUNDARY: the closure-aligned checkpoint where every coarse view is
    /// refreshed at once.
    ///
    /// Order matters and is fixed: pair readings first (they are what everything above
    /// reads), then the envelope (so the bound covers what just happened), then the
    /// global ledger gates, then the composite-holon layer. Each stage reads the stage
    /// below and writes nothing dynamical.
    pub fn close_grain(&mut self) {
        self.frame += 1;
        self.refresh_pairs();
        self.refresh_envelope();

        let e_now = self.mode_energy();
        if e_now > self.e_ref {
            self.e_ref = e_now;
        }
        // The momentum residual is sampled HERE and not per substep, and the asymmetry
        // with the energy drift above is deliberate. That residual is a floating-point
        // random walk, not an oscillation, so it has no period to alias against and a
        // boundary sample of it is a fair one. It also costs O(N) per evaluation rather
        // than the drift's handful of flops, so per-substep sampling would buy nothing
        // and charge for it.
        let m = self.momentum_residual();
        if m > self.momentum_residual_peak {
            self.momentum_residual_peak = m;
        }
        // The angular residual is sampled here for the same reason and at the same price.
        // It is tracked in EVERY box, not only the ones that conserve it: the peak is a
        // reading, and `angular_gate` is what decides whether the reading means anything.
        // Deciding at the accumulation site would leave a stale peak behind whenever a
        // scene changed boundary, which is a number that is right for a box that no longer
        // exists.
        let l = self.angular_residual();
        if l > self.angular_residual_peak {
            self.angular_residual_peak = l;
        }

        // The composite layer sees a state nothing above it has modified.
        let count = self.pair_count;
        let frame = self.frame;
        let time = self.time;
        let d_e = self.active_d_e();
        let n = self.n;
        let (pairs, holons) = (&self.pairs, &mut self.holons);
        holons.step_boundary(&pairs[..count], n, frame, time, d_e);
    }

    /// Advance `substeps` fixed steps and close the grain once at the end.
    pub fn step_frame(&mut self, substeps: u32) {
        for _ in 0..substeps {
            self.step();
        }
        self.close_grain();
    }

    /// Set an atom's in-plane velocity, leaving `vz` alone. On the mid-plane `vz` is
    /// zero and stays zero, which is what keeps a scripted 2D scene two-dimensional.
    pub fn set_velocity(&mut self, i: usize, vx: f64, vy: f64) {
        if i < self.n {
            self.atoms[i].vx = vx;
            self.atoms[i].vy = vy;
        }
    }

    pub fn set_velocity_3d(&mut self, i: usize, vx: f64, vy: f64, vz: f64) {
        if i < self.n {
            self.atoms[i].vx = vx;
            self.atoms[i].vy = vy;
            self.atoms[i].vz = vz;
        }
    }

    /// Set an atom's in-plane position, leaving `z` alone — same reasoning as
    /// [`Sim::set_velocity`].
    pub fn set_position(&mut self, i: usize, x: f64, y: f64) {
        if i < self.n {
            self.atoms[i].x = x;
            self.atoms[i].y = y;
        }
    }

    pub fn set_position_3d(&mut self, i: usize, x: f64, y: f64, z: f64) {
        if i < self.n {
            self.atoms[i].x = x;
            self.atoms[i].y = y;
            self.atoms[i].z = z;
        }
    }

    /// Freeze the current state as the ledger's origin. Call after any scripted setup
    /// that is meant to be the initial condition rather than an intervention.
    pub fn rebase(&mut self) {
        self.zero_ledger();
    }

    // ---------------------------------------------------------------- forces

    /// The soft quadratic box: `U = K_WALL * d^2 / 2` per face the atom has passed.
    ///
    /// The z faces are applied UNCONDITIONALLY, with no `dims` branch, and that is the
    /// lift's load-bearing simplification rather than an oversight. A 2D scene sits at
    /// `z = depth/2`, which is inside `[inset, depth - inset]` for any box deeper than
    /// twice the inset, so neither z branch is taken, `u` and `fz` keep the exact zeros
    /// they were initialised with, and `touched` is decided by x and y alone. The 2D
    /// wall energy is therefore the same float it was before the box grew a lid — and
    /// the box needs no mode flag to know which world it is in.
    fn wall_energy_force(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64, f64, bool) {
        // A box with no walls has no wall term. See `Boundary::has_walls` for what PLANT
        // P-2 found when this was two separate equality tests instead of one predicate.
        if !self.boundary.has_walls() {
            return (0.0, 0.0, 0.0, 0.0, false);
        }
        let lo = self.wall_inset;
        let hi_x = self.width - self.wall_inset;
        let hi_y = self.height - self.wall_inset;
        let hi_z = self.depth - self.wall_inset;
        let mut u = 0.0;
        let mut fx = 0.0;
        let mut fy = 0.0;
        let mut fz = 0.0;
        let mut touched = false;
        if x < lo {
            let d = lo - x;
            u += 0.5 * K_WALL * d * d;
            fx += K_WALL * d;
            touched = true;
        } else if x > hi_x {
            let d = x - hi_x;
            u += 0.5 * K_WALL * d * d;
            fx -= K_WALL * d;
            touched = true;
        }
        if y < lo {
            let d = lo - y;
            u += 0.5 * K_WALL * d * d;
            fy += K_WALL * d;
            touched = true;
        } else if y > hi_y {
            let d = y - hi_y;
            u += 0.5 * K_WALL * d * d;
            fy -= K_WALL * d;
            touched = true;
        }
        if z < lo {
            let d = lo - z;
            u += 0.5 * K_WALL * d * d;
            fz += K_WALL * d;
            touched = true;
        } else if z > hi_z {
            let d = z - hi_z;
            u += 0.5 * K_WALL * d * d;
            fz -= K_WALL * d;
            touched = true;
        }
        (u, fx, fy, fz, touched)
    }

    /// Recompute `a_pair` and `a_ext` from the current positions, and refresh the
    /// potential terms of the ledger. Split so the momentum ledger can tell the
    /// internal forces (which cancel) from the external ones (which do not).
    /// THE PAIR CURVE, evaluated — value, slope and curvature — with the declared
    /// truncation applied if there is one.
    ///
    /// Pure and `&self`, which is what lets it be called from a worker thread. It is the
    /// ONLY place the switch is applied to a pair, so the complete route and the
    /// neighbour-list route cannot disagree about what a truncated pair is worth.
    #[inline]
    fn pair_eval(&self, i: usize, j: usize, r: f64) -> (f64, f64, f64) {
        // THE BANK DISPATCH. One lookup per pair, by species slot.
        let (mut value, mut slope, mut curv) =
            self.bank.table_at(self.slots[i], self.slots[j]).eval(r);
        // THE DECLARED TRUNCATION, when there is one. `S(r)·U(r)` is a genuine potential,
        // so the force below is still minus a gradient and the energy ledger still closes
        // exactly — a hard cutoff would leave a step in the energy at the crossing and
        // turn the drift gate into a detector of its own truncation.
        if let Some((r_in, r_cut)) = self.pair_switch {
            if r > r_in {
                let (sw, ds, dds) = crate::cells::switch_c2(r, r_in, r_cut);
                let (u, du, ddu) = (value, slope, curv);
                value = sw * u;
                slope = sw * du + ds * u;
                curv = sw * ddu + 2.0 * ds * du + dds * u;
            }
        }
        (value, slope, curv)
    }

    /// ONE PAIR's evaluated term. Pure; see [`PairTerm`].
    #[inline]
    pub fn pair_term(&self, p: &crate::cells::NeighbourPair) -> PairTerm {
        let (value, slope, curv) = self.pair_eval(p.i as usize, p.j as usize, p.r);
        // F = -dE/dR along the separation; positive slope pulls the pair together.
        let f_over_r = slope / p.r;
        PairTerm {
            fx: f_over_r * p.dx,
            fy: f_over_r * p.dy,
            fz: f_over_r * p.dz,
            value,
            curv,
            virial: p.r * slope,
        }
    }

    /// Evaluate the pair terms `base .. base + out.len()`. The unit of work a
    /// [`ForceExecutor`] hands to a worker.
    pub fn eval_pair_chunk(&self, base: usize, out: &mut [PairTerm]) {
        let pairs = &self.neighbours.pairs;
        for (k, slot) in out.iter_mut().enumerate() {
            *slot = self.pair_term(&pairs[base + k]);
        }
    }

    /// The triples the current configuration admits, as built by the last force
    /// evaluation. Entry `k` is what `eval_triple_chunk` evaluates into term `k`.
    pub fn triples(&self) -> &[[usize; 3]] {
        &self.triple_scratch
    }

    /// Evaluate the triple terms `base .. base + out.len()`.
    pub fn eval_triple_chunk(&self, base: usize, out: &mut [TripleTerm]) {
        for (k, slot) in out.iter_mut().enumerate() {
            *slot = self.triple_term(self.triple_scratch[base + k]);
        }
    }

    /// Install an executor for the force evaluation. `None` restores [`SerialExecutor`].
    ///
    /// Changing this must not change a number. That is not a hope: `tests/t3_parallel.rs`
    /// runs one configuration under one worker and under many and compares the bits.
    pub fn set_executor(&mut self, exec: Option<Box<dyn ForceExecutor + Send + Sync>>) {
        self.executor = exec;
    }

    /// How many workers the installed executor reports.
    pub fn workers(&self) -> usize {
        self.executor.as_ref().map(|e| e.workers()).unwrap_or(1)
    }

    /// Evaluate `terms` through the installed executor, or serially when there is none.
    ///
    /// The executor is moved out for the call because it needs `&Sim` while `&mut Sim` is
    /// held — and taking it out is honest about that rather than reaching for a cell. It
    /// is put back before the function returns on every path.
    fn dispatch_pairs(&mut self, mut terms: Vec<PairTerm>) -> Vec<PairTerm> {
        match self.executor.take() {
            None => SerialExecutor.eval_pairs(self, &mut terms, FORCE_CHUNK),
            Some(e) => {
                e.eval_pairs(self, &mut terms, FORCE_CHUNK);
                self.executor = Some(e);
            }
        }
        terms
    }

    fn dispatch_triples(&mut self, mut terms: Vec<TripleTerm>) -> Vec<TripleTerm> {
        match self.executor.take() {
            None => SerialExecutor.eval_triples(self, &mut terms, FORCE_CHUNK),
            Some(e) => {
                e.eval_triples(self, &mut terms, FORCE_CHUNK);
                self.executor = Some(e);
            }
        }
        terms
    }

    /// ONE PAIR, evaluated and accumulated in one statement — the COMPLETE route, where
    /// there is no term list to shard because there is no cutoff to bound it.
    ///
    /// Shares [`Sim::pair_eval`] with the sharded route, so the two cannot drift apart in
    /// what a pair is worth; what differs is only whether the sum is walked immediately or
    /// after a wide evaluation pass.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn accumulate_pair(
        &mut self,
        i: usize,
        j: usize,
        dx: f64,
        dy: f64,
        dz: f64,
        r: f64,
        e_pair: &mut f64,
        k_pair_max: &mut f64,
        virial: &mut f64,
    ) {
        let (value, slope, curv) = self.pair_eval(i, j, r);
        *e_pair += value;
        *virial += r * slope;
        let f_over_r = slope / r;
        let fx = f_over_r * dx;
        let fy = f_over_r * dy;
        let fz = f_over_r * dz;
        // Newton's third law, applied as one computed value with opposite signs: this is
        // what makes the pair contribution cancel from the momentum sum.
        self.a_pair[i].0 += fx;
        self.a_pair[i].1 += fy;
        self.a_pair[i].2 += fz;
        self.a_pair[j].0 -= fx;
        self.a_pair[j].1 -= fy;
        self.a_pair[j].2 -= fz;
        let ac = curv.abs();
        if ac > *k_pair_max {
            *k_pair_max = ac;
        }
    }

    /// Recompute the whole force ledger at the CURRENT positions, without stepping.
    /// Public because the conservation gates need forces at a fixed geometry — a
    /// gradient check that has to integrate to see a force is measuring the integrator.
    pub fn compute_forces(&mut self) {
        for i in 0..self.n {
            self.a_pair[i] = (0.0, 0.0, 0.0);
            self.a_ext[i] = (0.0, 0.0, 0.0);
        }
        // Each atom's species slot, resolved ONCE. See `Sim::species_slots`.
        self.refresh_slots();
        // THE CELL LIST, rebuilt for this configuration. Sized to the largest cutoff any
        // sector needs, so one decomposition serves the pair, triple and quadruple loops
        // rather than three.
        self.rebuild_neighbours();

        let mut e_pair = 0.0;
        let mut k_pair_max = self.k_pair_max;
        let mut virial = 0.0f64;
        let geom = self.geom();
        let switch = self.pair_switch;

        // The pair sector takes one of two routes, and which one is a DECLARED property
        // of the scene rather than a size heuristic:
        //
        //   * no pair cutoff declared -> the complete `N²/2` sum, exactly the loop this
        //     engine has always run, with the same arithmetic in the same order. The pair
        //     curve has no radius past which it is zero, so a complete sum is the only
        //     one that is not an approximation, and a scene that has not declared a
        //     truncation budget does not get truncated behind its back;
        //   * a pair cutoff declared -> the neighbour list, which is O(N) with density.
        //     The truncation is switched (C², so the result is still a potential and the
        //     energy gate stays exact) and its size is reported by `truncation_floor`.
        if switch.is_none() {
            for i in 0..self.n {
                let a = (self.atoms[i].x, self.atoms[i].y, self.atoms[i].z);
                for j in (i + 1)..self.n {
                    let b = (self.atoms[j].x, self.atoms[j].y, self.atoms[j].z);
                    // ONE minimum-image implementation, called here. In an open or walled
                    // box this is `b - a` and nothing else, so the float is unchanged.
                    let (dx, dy, dz) = geom.delta(a, b);
                    // `(xx + yy) + zz`, in that order: on the mid-plane `zz` is an exact
                    // zero and adding it changes no bit of the 2D result.
                    let r2 = dx * dx + dy * dy + dz * dz;
                    // Two atoms at exactly the same point have no defined direction; the
                    // repulsive wall makes this unreachable dynamically, and the guard
                    // keeps it from being a NaN source if a caller places them there.
                    let r = r2.sqrt().max(1e-9);
                    self.accumulate_pair(
                        i,
                        j,
                        dx,
                        dy,
                        dz,
                        r,
                        &mut e_pair,
                        &mut k_pair_max,
                        &mut virial,
                    );
                }
            }
        } else {
            // EVALUATE WIDE, ACCUMULATE NARROW. The evaluation is where the cost is and it
            // is pure, so it goes through the executor; the accumulation walks the terms in
            // index order, which is the canonical order the complete loop produced, so the
            // sum is the same float however many workers there were.
            let mut terms = core::mem::take(&mut self.pair_terms);
            terms.clear();
            terms.resize(self.neighbours.pairs.len(), PairTerm::default());
            terms = self.dispatch_pairs(terms);
            let nb = core::mem::take(&mut self.neighbours);
            for (t, p) in terms.iter().zip(nb.pairs.iter()) {
                let (i, j) = (p.i as usize, p.j as usize);
                e_pair += t.value;
                virial += t.virial;
                self.a_pair[i].0 += t.fx;
                self.a_pair[i].1 += t.fy;
                self.a_pair[i].2 += t.fz;
                self.a_pair[j].0 -= t.fx;
                self.a_pair[j].1 -= t.fy;
                self.a_pair[j].2 -= t.fz;
                let ac = t.curv.abs();
                if ac > k_pair_max {
                    k_pair_max = ac;
                }
            }
            self.neighbours = nb;
            self.pair_terms = terms;
        }
        self.k_pair_max = k_pair_max;
        self.e_pair = e_pair;
        self.w_virial = virial;

        self.accumulate_three_body();
        self.accumulate_four_body();
        self.accumulate_far();

        let mut e_wall = 0.0;
        for i in 0..self.n {
            let (u, fx, fy, fz, touched) =
                self.wall_energy_force(self.atoms[i].x, self.atoms[i].y, self.atoms[i].z);
            e_wall += u;
            self.a_ext[i].0 += fx;
            self.a_ext[i].1 += fy;
            self.a_ext[i].2 += fz;
            if touched {
                self.wall_engaged = true;
            }
        }
        self.e_wall = e_wall;

        // THE UNIFORM FIELD (WB-2.4). Deliberately OUTSIDE the wall loop above, which
        // returns early for both wall-less boundaries through `Boundary::has_walls()`:
        // gravity acts in an open box and walls do not, and folding the two together is
        // how PLANT P-2 came to hand a periodic box a set of walls.
        //
        // It lands in `a_ext` with the walls and the hand, so the momentum ledger already
        // knows what to do with it -- `step` accumulates `0.5 * dt * a_ext` into `j_ext`
        // on both half-kicks, so the impulse this field delivers is booked without a line
        // of new accounting. It posts NOTHING to `w_ext` or to the receipt columns: a
        // uniform field is conservative, its energy is `e_grav` below, and a `work.gravity`
        // column would be double-counting the same joules the potential already holds.
        self.e_grav = 0.0;
        let (gx, gy, gz) = self.g_vec;
        if gx != 0.0 || gy != 0.0 || gz != 0.0 {
            for i in 0..self.n {
                let m = self.atoms[i].mass();
                let a = &self.atoms[i];
                // Force is `m g`, potential is `-m (g . r)`. The potential's zero is the
                // box ORIGIN, which is this force law's own zero -- the same choice the
                // scalar door made when it measured `y` from the lower face, now stated
                // in a form that does not privilege an axis. For `g = (0, -g, 0)` this
                // reduces to `+m g y` exactly, which is what the scalar-equivalence test
                // checks bit for bit.
                self.a_ext[i].0 += m * gx;
                self.a_ext[i].1 += m * gy;
                self.a_ext[i].2 += m * gz;
                self.e_grav -= m * (gx * a.x + gy * a.y + gz * a.z);
            }
        }

        self.e_spring = 0.0;
        if let Some(g) = self.grabbed {
            if g < self.n {
                let dx = self.atoms[g].x - self.anchor.0;
                let dy = self.atoms[g].y - self.anchor.1;
                let dz = self.atoms[g].z - self.anchor.2;
                self.e_spring = 0.5 * K_SPRING * (dx * dx + dy * dy + dz * dz);
                self.a_ext[g].0 += -K_SPRING * dx;
                self.a_ext[g].1 += -K_SPRING * dy;
                self.a_ext[g].2 += -K_SPRING * dz;
                self.spring_engaged = true;
            }
        }
    }

    /// THE LONG-RANGE SECTOR (node B2): the pair tail past `R_s`.
    ///
    /// Follows the four-body sector's pattern exactly — its own energy row, its virial
    /// accumulated where the slopes already are, forces into `a_pair` because the term is a
    /// conservative pairwise interaction and cancels from the momentum sum, and NOTHING
    /// posted to `w_ext`: a conservative term's energy is held by its potential, and a
    /// receipt column for it would book the same hartrees twice.
    ///
    /// Exactly nothing happens when no far sector is declared, and `e_far` stays an exact
    /// 0.0 — which is what keeps every pre-B2 replay fingerprint valid.
    fn accumulate_far(&mut self) {
        self.e_far = 0.0;
        let Some(mut far) = self.far.take() else {
            return;
        };
        if self.n == 0 {
            self.far = Some(far);
            return;
        }
        let geom = self.geom();
        let pos: Vec<(f64, f64, f64)> = self.atoms[..self.n]
            .iter()
            .map(|a| (a.x, a.y, a.z))
            .collect();
        // The support each TABLE actually carries, read from the bank rather than
        // remembered, so G1's channel split is drawn at the radius the committed table has.
        let r_max_by_slot: Vec<f64> = (0..crate::bank::MAX_TABLES)
            .map(|s| {
                let tbl = self.bank.table_slot(s);
                if tbl.is_loaded() {
                    tbl.r_max()
                } else {
                    0.0
                }
            })
            .collect();
        // THE NEAR SECTOR'S DECLARED TRUNCATION, handed over on every pass rather than
        // captured once. The far sector supplies what the near one did NOT, so it has to
        // know which of the pair sector's two routes ran — and a scene may declare or clear
        // a truncation between passes.
        far.set_switch(self.pair_switch);
        let mut forces = core::mem::take(&mut self.a_pair);
        let reading = far.accumulate(&pos, &self.slots[..self.n], geom, &mut forces, &r_max_by_slot);
        self.a_pair = forces;
        // A far sector that refused this pass contributes NOTHING, and the reading says so
        // rather than the energy quietly reading zero. `Sim::far_ok` is the door a caller
        // must go through before believing `e_far`; a gate that reads the energy without
        // asking has been handed a refusal wearing a number's clothes.
        self.e_far = reading.energy;
        self.w_virial += reading.virial;
        self.far_reading = reading;
        self.far = Some(far);
    }

    /// THE MANY-BODY SECTOR: the tabulated three-body term over every triple, and the
    /// forces it exerts.
    ///
    /// Nothing here is a new constant. The value comes from the interpolant, the three
    /// side-derivatives come from differentiating that same interpolant analytically, and
    /// the force on each atom is assembled from them by the chain rule
    /// `dE/dx_i = sum_a (dE/ds_a)(ds_a/dx_i)`, where `ds_a/dx_i` is a unit vector along
    /// the side. Each side contributes to its TWO atoms as one computed value with
    /// opposite signs — exactly the shape the pair loop uses — so the triple's total force
    /// is zero by construction and the momentum ledger has nothing new to subtract.
    ///
    /// The accelerations go into `a_pair`, which holds INTERNAL forces (those that cancel
    /// from the momentum sum) as opposed to `a_ext` (walls, spring, thermostat). The
    /// energy is kept in its own ledger row.
    ///
    /// A triple whose middle side is past the table's domain returns an exact zero and
    /// costs one comparison; in a dispersed gas that is almost every triple, which is what
    /// keeps the N^3 loop from being the whole budget when there is nothing to compute.
    fn accumulate_three_body(&mut self) {
        self.e_three = 0.0;
        self.fence_untabulated = self.fenced_triples();
        if (!self.trimer.loaded && !self.water.loaded && self.trimers.is_empty()) || self.n < 3 {
            return;
        }
        let cut3 = self.three_body_cutoff();
        if !(cut3 > 0.0) {
            return;
        }

        // THE TRIPLE ENUMERATION, cutoff-local and canonical.
        //
        // Every one of these surfaces is an exact zero unless TWO SIDES MEETING AT ONE
        // VERTEX are both inside its domain (see `Sim::three_body_cutoff`). So the triples
        // that can contribute are exactly the ones some vertex sees both partners of, and
        // the neighbour list already holds, per atom, the partners it sees.
        //
        // A triple can have more than one such vertex, so it would be enumerated more than
        // once. The canonical rule: emit at hub `h` when the opposite side `|jk|` is
        // longer than the cutoff (then `h` is the ONLY qualifying vertex), and otherwise
        // only when `h` is the smallest of the three indices. Every triple is emitted
        // exactly once, and which vertex emitted it is a function of the configuration
        // rather than of the traversal.
        //
        // The list is then SORTED into ascending `(a, b, c)` before evaluation. That is
        // not tidiness: `e_three` and the force accumulations are floating-point sums, so
        // the order is part of the answer, and the sorted order is the one the complete
        // `i < j < k` loop produced. A cutoff-local run therefore agrees with a complete
        // run bit-for-bit on the terms they share.
        let nb = core::mem::take(&mut self.neighbours);
        let mut triples = core::mem::take(&mut self.triple_scratch);
        triples.clear();
        for h in 0..self.n {
            let (mine, radii) = nb.adj_of(h);
            for a in 0..mine.len() {
                if radii[a] > cut3 {
                    continue;
                }
                for b in (a + 1)..mine.len() {
                    if radii[b] > cut3 {
                        continue;
                    }
                    let (j, k) = (mine[a] as usize, mine[b] as usize);
                    // Does a lower-indexed vertex also qualify as a hub? It does exactly
                    // when the opposite side is itself inside the cutoff — then all three
                    // vertices see both their partners and the triple would be emitted
                    // three times.
                    let opposite_local = nb.separation(j, k).map(|r| r <= cut3).unwrap_or(false);
                    if opposite_local && !(h < j && h < k) {
                        continue;
                    }
                    let mut t = [h, j, k];
                    t.sort_unstable();
                    triples.push(t);
                }
            }
        }
        triples.sort_unstable();
        self.neighbours = nb;

        self.triple_scratch = triples;

        // EVALUATE WIDE, ACCUMULATE NARROW — the same split the pair sector uses, and for
        // the same reason: the interpolant evaluations are the cost and are pure, the sums
        // are the answer and are ordered.
        let mut terms = core::mem::take(&mut self.triple_terms);
        terms.clear();
        terms.resize(self.triple_scratch.len(), TripleTerm::default());
        terms = self.dispatch_triples(terms);

        let mut e_three = 0.0;
        // Per-atom stiffness totals: curvatures ADD over the triples an atom is in.
        let mut k_atom = core::mem::take(&mut self.k_atom_scratch);
        k_atom.clear();
        k_atom.resize(self.n, 0.0);
        for t in terms.iter() {
            if !t.live {
                continue;
            }
            let (a, b, c) = (t.a as usize, t.b as usize, t.c as usize);
            e_three += t.v;
            // The three-body virial, by the same `Σ s · dE/ds` the pair sector uses: each
            // side push IS a pair force with `g` in the slope's role, so the side sum is
            // the triple's contribution and nothing new has to be derived.
            self.w_virial += t.g[0] * t.r[0] + t.g[1] * t.r[1] + t.g[2] * t.r[2];
            self.push_side(a, b, t.g[0], t.r[0]);
            self.push_side(a, c, t.g[1], t.r[1]);
            self.push_side(b, c, t.g[2], t.r[2]);
            k_atom[a] += t.kt;
            k_atom[b] += t.kt;
            k_atom[c] += t.kt;
        }
        self.e_three = e_three;
        for k in k_atom.iter() {
            if *k > self.k_three_max {
                self.k_three_max = *k;
            }
        }
        self.k_atom_scratch = k_atom;
        self.triple_terms = terms;
    }

    /// ONE TRIPLE, evaluated. Pure and `&self`; see [`TripleTerm`].
    ///
    /// THE COMPOSITION DISPATCH lives here, and `Sim::served` states the same dispatch for
    /// the fence count, so the two cannot disagree about what is served. The hardcoded
    /// composition branches are DRY-residual R-4 — see
    /// `conformance/water_observatory/DRY_RESIDUALS.md`, which also names what would
    /// discharge them (emitting the four in-memory surfaces through the provenanced bank
    /// door the shipped ones already use).
    pub fn triple_term(&self, t: [usize; 3]) -> TripleTerm {
        let [i, j, k] = t;
        // Sides, minimum-imaged, in the same `sqrt().max(1e-9)` form every other loop uses.
        // Computed here rather than read from the neighbour list because the side opposite
        // the hub may be longer than the list's cutoff.
        let geom = self.geom();
        let pi = (self.atoms[i].x, self.atoms[i].y, self.atoms[i].z);
        let pj = (self.atoms[j].x, self.atoms[j].y, self.atoms[j].z);
        let pk = (self.atoms[k].x, self.atoms[k].y, self.atoms[k].z);
        let sep = |a: (f64, f64, f64), b: (f64, f64, f64)| -> f64 {
            let (dx, dy, dz) = geom.delta(a, b);
            (dx * dx + dy * dy + dz * dz).sqrt().max(1e-9)
        };
        let d_ij = sep(pi, pj);
        let d_ik = sep(pi, pk);
        let d_jk = sep(pj, pk);
        let d = |a: usize, b: usize| -> f64 {
            if (a == i && b == j) || (a == j && b == i) {
                d_ij
            } else if (a == i && b == k) || (a == k && b == i) {
                d_ik
            } else {
                d_jk
            }
        };

        let (za, zb, zc) = (
            self.atoms[i].species.z as u8,
            self.atoms[j].species.z as u8,
            self.atoms[k].species.z as u8,
        );
        let (a, b, c, v, g, env_abs, env_per_grad) = if self
            .trimers
            .find([za, zb, zc])
            .is_some()
        {
            // A SHIPPED surface exists for this composition and is deliberately NOT
            // evaluated. It is fenced and counted, which is what the fence counter is for.
            //
            // This branch used to call `TrimerTable::eval` on it. That was wrong in two
            // independent ways, both found by putting a REAL artifact next to the code
            // rather than the schema's example:
            //
            //   1. GRID RULE. `TrimerTable` is this build's 33x33x13 grid with
            //      `r_of_tau`'s STRETCH_A = 2.0 spacing. `s3_tables` emits UNIFORM-LINEAR
            //      spacing on an arbitrary grid -- the first real artifact is 4x4x2.
            //      Interpolating uniform data on stretched axes is smooth, plausible, and
            //      wrong everywhere except the boundary, which is the exact failure
            //      `load_water_table` refuses by construction.
            //   2. COORDINATES. `eval` takes three SIDE LENGTHS; the artifact's axes are
            //      (x, y, u) with `u` an angle-like coordinate. Even on a matching grid
            //      these would not be the same quantities.
            //
            // So a shipped surface is admitted, stored and READABLE -- the fence in
            // `holon_trimer_h_only` lifts off it -- and it is not integrated until an
            // evaluator exists for the geometry the artifact actually ships. Fencing costs
            // a counted truncation; the alternative costs a wrong force nobody would see.
            //
            // A DEFAULT TripleTerm is `live: false`, which is this function's own way of
            // saying "no server for this triple" -- the same exit the untabulated
            // compositions take below. `served` is the other half: it must NOT report a
            // shipped surface as served, or the census would book these as covered
            // rather than fenced and the truncation would stop being counted.
            return TripleTerm::default();
        } else {
            let n_o = (za == 8) as u32 + (zb == 8) as u32 + (zc == 8) as u32;
            let n_h = (za == 1) as u32 + (zb == 1) as u32 + (zc == 1) as u32;
            if n_h == 3 && self.trimer.loaded {
                let (rab, rac, rbc) = (d_ij, d_ik, d_jk);
                let (val, grad) = self.trimer.eval([rab, rac, rbc]);
                (
                    i,
                    j,
                    k,
                    val,
                    grad,
                    self.trimer.curvature_envelope,
                    self.trimer.curvature_per_gradient,
                )
            } else if n_o == 1 && n_h == 2 && self.water.loaded {
                let (sa, sb, sc) = if za == 8 {
                    (i, j, k)
                } else if zb == 8 {
                    (j, i, k)
                } else {
                    (k, i, j)
                };
                let (rab, rac, rbc) = (d(sa, sb), d(sa, sc), d(sb, sc));
                let (val, grad) = self.water.eval(rab, rac, rbc);
                (
                    sa,
                    sb,
                    sc,
                    val,
                    grad,
                    self.water.curvature_envelope,
                    self.water.curvature_per_gradient,
                )
            } else if n_o == 2 && n_h == 1 && self.ooh.loaded {
                let (sa, sb, sc) = if za == 1 {
                    (i, j, k)
                } else if zb == 1 {
                    (j, i, k)
                } else {
                    (k, i, j)
                };
                let (rab, rac, rbc) = (d(sa, sb), d(sa, sc), d(sb, sc));
                let (val, grad) = self.ooh.eval(rab, rac, rbc);
                (
                    sa,
                    sb,
                    sc,
                    val,
                    grad,
                    self.ooh.curvature_envelope,
                    self.ooh.curvature_per_gradient,
                )
            } else if n_o == 3 && self.ozone.loaded {
                let (rab, rac, rbc) = (d_ij, d_ik, d_jk);
                let (val, grad) = self.ozone.eval(rab, rac, rbc);
                (
                    i,
                    j,
                    k,
                    val,
                    grad,
                    self.ozone.curvature_envelope,
                    self.ozone.curvature_per_gradient,
                )
            } else {
                return TripleTerm::default();
            }
        };

        if v == 0.0 && g[0] == 0.0 && g[1] == 0.0 && g[2] == 0.0 {
            return TripleTerm::default();
        }
        let (rab, rac, rbc) = (d(a, b), d(a, c), d(b, c));
        // The per-triple stiffness the drift bound is built from; the derivation is in
        // `Sim::k_three`.
        let gmax = g[0].abs().max(g[1].abs()).max(g[2].abs());
        let g2 = env_abs.min(env_per_grad * gmax);
        let kt = 4.0 * g2 + 2.0 * (g[0].abs() / rab + g[1].abs() / rac + g[2].abs() / rbc);
        TripleTerm {
            a: a as u32,
            b: b as u32,
            c: c as u32,
            v,
            g,
            r: [rab, rac, rbc],
            kt,
            live: true,
        }
    }

    /// THE FENCE, counted combinatorially instead of by enumeration.
    ///
    /// A triple is fenced when its COMPOSITION has no server — the shipped bank does not
    /// carry it, and it is not one of the four in-memory surfaces, or the surface it would
    /// need is not loaded. That is a fact about which nuclei are in the scene and which
    /// tables are open, and it does not depend on where the atoms are: the old enumeration
    /// counted a triple whether its atoms were 2 bohr or 200 apart.
    ///
    /// So it is computed from the SPECIES CENSUS, in `O(species³)` rather than `O(N³)`,
    /// and it yields the identical number the complete triple loop yielded. That identity
    /// is what let the enumeration become cutoff-local without moving the fence incidence
    /// the prereg pins ("the four OOO triples stay HONESTLY FENCED at exactly 4/seed").
    ///
    /// M-VACUOUS-SUCCESS: `tests/pbc.rs::the_fence_count_survives_going_local` holds the
    /// two counts against each other on a scene that has both fenced and served triples.
    pub fn fenced_triples(&self) -> u64 {
        if self.n < 3 {
            return 0;
        }
        if !self.trimer.loaded && !self.water.loaded && self.trimers.is_empty() {
            // The pre-T3 loop returned before counting anything in this case, and the
            // fence is a reading of that loop. Preserved deliberately: a scene carrying
            // only an (O,O,H) or ozone surface reports no fence today, and changing that
            // here would move a campaign number for a reason that has nothing to do with
            // T3. Entered in the DRY-residual register as R-3.
            return 0;
        }
        // The census: how many atoms of each nuclear charge.
        let mut zs: Vec<(u8, u64)> = Vec::new();
        for i in 0..self.n {
            let z = self.atoms[i].species.z as u8;
            match zs.iter_mut().find(|(k, _)| *k == z) {
                Some(e) => e.1 += 1,
                None => zs.push((z, 1)),
            }
        }
        zs.sort_unstable();
        let mut fenced = 0u64;
        for (ia, &(za, na)) in zs.iter().enumerate() {
            for (ib, &(zb, nb)) in zs.iter().enumerate().skip(ia) {
                for &(zc, nc) in zs.iter().skip(ib) {
                    if self.served([za, zb, zc]) {
                        continue;
                    }
                    // Unordered multiset count: the number of ways to pick this
                    // composition out of the census.
                    let count = if za == zb && zb == zc {
                        na * na.saturating_sub(1) * na.saturating_sub(2) / 6
                    } else if za == zb {
                        na * na.saturating_sub(1) / 2 * nc
                    } else if zb == zc {
                        na * (nb * nb.saturating_sub(1) / 2)
                    } else {
                        na * nb * nc
                    };
                    fenced += count;
                }
            }
        }
        fenced
    }

    /// Whether a composition has a three-body surface to be evaluated on. The dispatch of
    /// `accumulate_three_body`, stated once so the fence count and the force loop cannot
    /// disagree about what is served.
    fn served(&self, z: [u8; 3]) -> bool {
        // A SHIPPED SURFACE IS NOT SERVED. It is admitted, stored and readable, and the
        // three-body dispatch deliberately fences it (see `triple_term`) until an
        // evaluator exists for the geometry `s3_tables` emits. Reporting it as served here
        // would book every such triple as covered, and the truncation the fence creates
        // would vanish from the census -- which is the one thing the census exists to
        // prevent. This goes back to `true` in the same change that lands the evaluator.
        let n_o = z.iter().filter(|&&v| v == 8).count();
        let n_h = z.iter().filter(|&&v| v == 1).count();
        (n_h == 3 && self.trimer.loaded)
            || (n_o == 1 && n_h == 2 && self.water.loaded)
            || (n_o == 2 && n_h == 1 && self.ooh.loaded)
            || (n_o == 3 && self.ozone.loaded)
    }

    /// THE 4-BODY VALENCE SECTOR: Exact ab-initio (O,H,H,H) dE4 evaluation for compact quadruples.
    ///
    /// Cutoff-gated: Quadruples with any O-H distance >= R_CUT (6.0 bohr) evaluate to zero
    /// without invoking the electronic structure solver.
    /// When compact (all 3 O-H distances < 6.0 bohr), the C^2 switching function smoothly blends
    /// from 1.0 (at <= 5.0 bohr) to 0.0 (at 6.0 bohr), and forces are computed via central
    /// finite difference (h = 1e-4 bohr) on the quadruple's Cartesian coordinates.
    fn accumulate_four_body(&mut self) {
        self.e_four = 0.0;
        if !self.de4_enabled || self.n < 4 || !self.water.loaded || !self.trimer.loaded {
            return;
        }

        // Fast displacement-based reuse across micro-substeps:
        if self.de4_cached_valid {
            let mut max_disp_sq = 0.0f64;
            for i in 0..self.n {
                let dx = self.atoms[i].x - self.de4_last_pos[i][0];
                let dy = self.atoms[i].y - self.de4_last_pos[i][1];
                let dz = self.atoms[i].z - self.de4_last_pos[i][2];
                let d2 = dx * dx + dy * dy + dz * dz;
                if d2 > max_disp_sq {
                    max_disp_sq = d2;
                }
            }
            // If all atoms moved < 0.08 bohr (6.4e-3 bohr^2), reuse cached forces and energy:
            if max_disp_sq < 0.0064 {
                self.e_four = self.de4_cached_energy;
                self.w_virial += self.de4_cached_virial;
                for i in 0..self.n {
                    self.a_pair[i].0 += self.de4_cached_forces[i].0;
                    self.a_pair[i].1 += self.de4_cached_forces[i].1;
                    self.a_pair[i].2 += self.de4_cached_forces[i].2;
                }
                return;
            }
        }

        const R_CUT: f64 = DE4_R_CUT;
        const R_IN: f64 = DE4_R_IN;

        let mut e_four = 0.0;
        let mut quad_virial = 0.0f64;
        let mut total_forces = core::mem::take(&mut self.quad_force_scratch);
        total_forces.clear();
        total_forces.resize(self.n, (0.0f64, 0.0f64, 0.0f64));

        // THE QUADRUPLE ENUMERATION, cutoff-local.
        //
        // The sector is (O,H,H,H) with the switch on the three O-H distances, so a
        // quadruple contributes only when all three hydrogens are inside `R_CUT` of the
        // SAME oxygen. That oxygen is the hub, every quadruple has exactly one of them,
        // and the neighbour list already holds each atom's partners inside the cutoff — so
        // the enumeration is over each oxygen's own hydrogens rather than over `N⁴/24`
        // quadruples of which all but a handful are empty.
        //
        // The complete loop this replaces ran `h1` over the whole scene and `h2`, `h3`
        // above it, so its order was ascending `(o, h1, h2, h3)`. The hub's adjacency is
        // ascending too, so the triples of hydrogens come out in that same order and the
        // floating-point sum is unchanged.
        let geom = self.geom();
        let species = self.species_slots();
        let nb = core::mem::take(&mut self.neighbours);
        let mut hs: Vec<usize> = Vec::new();
        for o in 0..self.n {
            if self.atoms[o].species.z != 8 {
                continue;
            }
            let po = [self.atoms[o].x, self.atoms[o].y, self.atoms[o].z];
            hs.clear();
            let (mine, radii) = nb.adj_of(o);
            for k in 0..mine.len() {
                let h = mine[k] as usize;
                if self.atoms[h].species.z == 1 && radii[k] < R_CUT {
                    hs.push(h);
                }
            }
            if hs.len() < 3 {
                continue;
            }

            for a in 0..hs.len() {
                let h1 = hs[a];
                // Positions are taken as the MINIMUM IMAGE about the oxygen, so a
                // quadruple that straddles a periodic face is still a compact quadruple
                // and not four atoms strung across the box. Under walls or an open box
                // `delta` is the raw difference and these are the atoms' own coordinates.
                let p1 = image_about(geom, po, self.atoms[h1]);
                let r1 = dist(po, p1);
                if r1 >= R_CUT {
                    continue;
                }
                for b in (a + 1)..hs.len() {
                    let h2 = hs[b];
                    let p2 = image_about(geom, po, self.atoms[h2]);
                    let r2 = dist(po, p2);
                    if r2 >= R_CUT {
                        continue;
                    }
                    for c in (b + 1)..hs.len() {
                        let h3 = hs[c];
                        let p3 = image_about(geom, po, self.atoms[h3]);
                        let r3 = dist(po, p3);
                        if r3 >= R_CUT {
                            continue;
                        }

                        // Compact encounter under R_CUT = 6.0 bohr!
                        let r_max = r1.max(r2).max(r3);
                        let (sw, dsw, _) = crate::cells::switch_c2(r_max, R_IN, R_CUT);
                        if sw <= 0.0 {
                            continue;
                        }

                        self.de4_eval_count += 1;

                        // EXACT four-body force. Nine seeded dual solves give the exact
                        // Cartesian gradient of E_FCI(OH3) — `ohhh_fci_grad` imposes the
                        // oxygen row by translation invariance, so the FCI force sum is
                        // zero to the last bit — and the MBE3 half is assembled from THE
                        // SAME curves the pair and triple sectors apply, so the four-body
                        // term subtracts exactly what the rest of the ledger adds and its
                        // gradient is pairwise by construction. This replaced a scheme
                        // that took 36 value-only solves per recompute (4 of them
                        // physics: the others re-solved two isolated atoms and six pair
                        // diatomics that are constants and loaded tables) for HALF a
                        // gradient — the radial projection — with O(h) forward-difference
                        // error: every tangential component, including every H-H force
                        // inside the correction, never reached the trajectory. No
                        // finite-difference step remains, and no mass appears anywhere:
                        // `total_forces` holds FORCE, and the integrator divides once.
                        let centers4 = [po, p1, p2, p3];
                        let warm = self
                            .de4_ci
                            .iter()
                            .find(|(hub, _)| *hub == o)
                            .map(|(_, v)| v.clone());
                        let fci =
                            holon_chem::quaternary::ohhh_fci_grad(&centers4, warm.as_deref());
                        match self.de4_ci.iter_mut().find(|(hub, _)| *hub == o) {
                            Some(slot) => slot.1 = fci.ci,
                            None => self.de4_ci.push((o, fci.ci)),
                        }

                        // MBE3 value and gradient from the loaded curves, all pairwise.
                        let e_o = holon_chem::quaternary::atom_energy_o();
                        let e_h = holon_chem::quaternary::atom_energy_h();
                        let r12 = dist(p1, p2);
                        let r23 = dist(p2, p3);
                        let r31 = dist(p3, p1);
                        let mut e_mbe3 = e_o + 3.0 * e_h;
                        let mut gm = [[0.0f64; 3]; 4]; // grad E_MBE3, local slots [O,H1,H2,H3]

                        // The six pair terms, from the bank's own Hermite curves
                        // (value, slope) — the render table's zero IS the dissociated
                        // asymptote, which is exactly the `pair - atoms` quantity the
                        // MBE3 definition subtracts.
                        {
                            let pl = [po, p1, p2, p3];
                            let pair_list: [(usize, usize, f64); 6] = [
                                (0, 1, r1),
                                (0, 2, r2),
                                (0, 3, r3),
                                (1, 2, r12),
                                (2, 3, r23),
                                (3, 1, r31),
                            ];
                            let gidx = [o, h1, h2, h3];
                            for &(a, b, r) in &pair_list {
                                let t =
                                    self.bank.table_at(species[gidx[a]], species[gidx[b]]);
                                let (v, dv, _) = t.eval(r.max(1e-12));
                                e_mbe3 += v;
                                add_pair_grad(&mut gm, &pl, a, b, r, dv);
                            }
                        }

                        // The four triple terms, from the same tables the triple sector
                        // serves, each with its analytic gradient in its three distances.
                        {
                            let pl = [po, p1, p2, p3];
                            let (v, g) = self.water.eval(r1, r2, r12);
                            e_mbe3 += v;
                            add_pair_grad(&mut gm, &pl, 0, 1, r1, g[0]);
                            add_pair_grad(&mut gm, &pl, 0, 2, r2, g[1]);
                            add_pair_grad(&mut gm, &pl, 1, 2, r12, g[2]);
                            let (v, g) = self.water.eval(r2, r3, r23);
                            e_mbe3 += v;
                            add_pair_grad(&mut gm, &pl, 0, 2, r2, g[0]);
                            add_pair_grad(&mut gm, &pl, 0, 3, r3, g[1]);
                            add_pair_grad(&mut gm, &pl, 2, 3, r23, g[2]);
                            let (v, g) = self.water.eval(r3, r1, r31);
                            e_mbe3 += v;
                            add_pair_grad(&mut gm, &pl, 0, 3, r3, g[0]);
                            add_pair_grad(&mut gm, &pl, 0, 1, r1, g[1]);
                            add_pair_grad(&mut gm, &pl, 3, 1, r31, g[2]);
                            let (v, g) = self.trimer.eval([r12, r23, r31]);
                            e_mbe3 += v;
                            add_pair_grad(&mut gm, &pl, 1, 2, r12, g[0]);
                            add_pair_grad(&mut gm, &pl, 2, 3, r23, g[1]);
                            add_pair_grad(&mut gm, &pl, 3, 1, r31, g[2]);
                        }

                        let de4 = fci.e - e_mbe3;
                        let u_four = sw * de4;
                        e_four += u_four;

                        // F = -grad(sw * de4) = -sw*(grad E_FCI - grad E_MBE3)
                        //     - de4 * dsw * grad r_max, the last pairwise on the
                        // argmax O-H bond.
                        let mut fl = [[0.0f64; 3]; 4];
                        for a in 0..4 {
                            for x in 0..3 {
                                fl[a][x] = -sw * (fci.grad[a][x] - gm[a][x]);
                            }
                        }
                        {
                            let (amax, ra, pa) = if r1 >= r2 && r1 >= r3 {
                                (1usize, r1, p1)
                            } else if r2 >= r3 {
                                (2usize, r2, p2)
                            } else {
                                (3usize, r3, p3)
                            };
                            let c = de4 * dsw;
                            let rr = ra.max(1e-12);
                            for x in 0..3 {
                                let u = (pa[x] - po[x]) / rr;
                                fl[amax][x] -= c * u;
                                fl[0][x] += c * u;
                            }
                        }

                        // The four-body virial: -sum p . F over the quadruple's IMAGED
                        // positions. The force sum is exactly zero, so the origin drops
                        // out, and for the pairwise decomposition this is the same
                        // sum r . dU/dr the other sectors accumulate.
                        let pl = [po, p1, p2, p3];
                        for a in 0..4 {
                            quad_virial -=
                                pl[a][0] * fl[a][0] + pl[a][1] * fl[a][1] + pl[a][2] * fl[a][2];
                        }

                        let gidx = [o, h1, h2, h3];
                        for a in 0..4 {
                            total_forces[gidx[a]].0 += fl[a][0];
                            total_forces[gidx[a]].1 += fl[a][1];
                            total_forces[gidx[a]].2 += fl[a][2];
                        }
                    }
                }
            }
        }
        self.neighbours = nb;

        // Apply forces to a_pair and update cache:
        for i in 0..self.n {
            self.a_pair[i].0 += total_forces[i].0;
            self.a_pair[i].1 += total_forces[i].1;
            self.a_pair[i].2 += total_forces[i].2;
            self.de4_last_pos[i] = [self.atoms[i].x, self.atoms[i].y, self.atoms[i].z];
        }
        self.de4_cached_forces.clear();
        self.de4_cached_forces.extend_from_slice(&total_forces);
        self.quad_force_scratch = total_forces;
        self.de4_cached_energy = e_four;
        self.de4_cached_virial = quad_virial;
        self.de4_cached_valid = true;
        self.e_four = e_four;
        self.w_virial += quad_virial;
    }
}

/// An atom's position as the MINIMUM IMAGE of `about` — the coordinates the many-body
/// solvers must be handed under periodic boundaries.
///
/// A quadruple that straddles a box face has atoms whose stored coordinates are a box
/// apart; handing those to an electronic-structure solver would ask it about a molecule
/// stretched across the universe. The image is taken about the hub so the cluster is
/// compact and its internal geometry is the one the cutoff admitted.
#[inline]
fn image_about(geom: crate::cells::BoxGeom, about: [f64; 3], a: Atom) -> [f64; 3] {
    let (dx, dy, dz) = geom.delta((about[0], about[1], about[2]), (a.x, a.y, a.z));
    [about[0] + dx, about[1] + dy, about[2] + dz]
}

/// One pairwise share of a scalar potential's gradient: `dv` is dV/dr for the pair
/// (a, b) at separation `r`; the contribution is `dv` along the unit vector from a to b,
/// equal and opposite — the same convention `push_side` carries, on imaged coordinates.
#[inline]
fn add_pair_grad(g: &mut [[f64; 3]; 4], p: &[[f64; 3]; 4], a: usize, b: usize, r: f64, dv: f64) {
    let rr = r.max(1e-12);
    for x in 0..3 {
        let u = (p[b][x] - p[a][x]) / rr;
        g[b][x] += dv * u;
        g[a][x] -= dv * u;
    }
}

#[inline]
fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[inline]
// KEPT THOUGH UNCALLED: this is the half of the fenced shipped-surface branch
// that was RIGHT — a surface declares its species in its own order, and the
// force loop's i, j, k need permuting to match. The evaluator that is owed will
// need it. Deleting it would throw away the correct part of a branch removed for
// its incorrect part.
#[allow(dead_code)]
fn match_triple_slots(
    i: usize, zi: u8,
    j: usize, zj: u8,
    k: usize, zk: u8,
    target: [u8; 3],
) -> Option<(usize, usize, usize)> {
    let perms = [
        (i, zi, j, zj, k, zk),
        (i, zi, k, zk, j, zj),
        (j, zj, i, zi, k, zk),
        (j, zj, k, zk, i, zi),
        (k, zk, i, zi, j, zj),
        (k, zk, j, zj, i, zi),
    ];
    for (a, za, b, zb, c, zc) in perms {
        if za == target[0] && zb == target[1] && zc == target[2] {
            return Some((a, b, c));
        }
    }
    None
}

impl Sim {
    /// One side's share of a triple's force, applied equal and opposite. `g` is
    /// `dE/dr_ab`, the same convention the pair loop's `slope` carries, so the sign logic
    /// is the one line it already is there and not a second one to keep true.
    #[inline]
    fn push_side(&mut self, a: usize, b: usize, g: f64, r: f64) {
        let f_over_r = g / r;
        let fx = f_over_r * (self.atoms[b].x - self.atoms[a].x);
        let fy = f_over_r * (self.atoms[b].y - self.atoms[a].y);
        let fz = f_over_r * (self.atoms[b].z - self.atoms[a].z);
        self.a_pair[a].0 += fx;
        self.a_pair[a].1 += fy;
        self.a_pair[a].2 += fz;
        self.a_pair[b].0 -= fx;
        self.a_pair[b].1 -= fy;
        self.a_pair[b].2 -= fz;
    }

    pub(crate) fn accumulate_energy(&mut self) {
        let mut e_kin = 0.0;
        for i in 0..self.n {
            let a = &self.atoms[i];
            e_kin += 0.5 * a.mass() * (a.vx * a.vx + a.vy * a.vy + a.vz * a.vz);
        }
        self.e_kin = e_kin;
    }

    // ---------------------------------------------------------------- stepping

    /// One velocity-Verlet step.
    ///
    /// The external impulse is accumulated from exactly the same half-kick terms that
    /// enter the velocities, so the momentum ledger is not an independent estimate of
    /// the impulse — it is the impulse.
    pub fn step(&mut self) {
        if self.n == 0 || !self.pairs_ready() {
            return;
        }
        // TWO INTEGRATORS, and this is the only place that chooses between them.
        //
        // They are not one integrator with a flag: NVE/NVT is velocity Verlet on the
        // physical Hamiltonian, NPT is the MTK Trotter factorization on an extended one
        // whose box is a degree of freedom. Writing the second as a special case of the
        // first is how a barostat becomes a rescale hack. `tests/t3_barostat.rs` gates the
        // relation that DOES hold between them: at infinite barostat mass with the chains
        // idle, NPT reproduces NVE.
        if self.barostat_on() {
            self.step_npt();
            return;
        }
        let dt = self.dt();

        let mut jx = 0.0;
        let mut jy = 0.0;
        let mut jz = 0.0;
        for i in 0..self.n {
            let (px, py, pz) = self.a_pair[i];
            let (ex, ey, ez) = self.a_ext[i];
            let half = 0.5 * dt / self.atoms[i].mass();
            self.atoms[i].vx += half * (px + ex);
            self.atoms[i].vy += half * (py + ey);
            self.atoms[i].vz += half * (pz + ez);
            jx += 0.5 * dt * ex;
            jy += 0.5 * dt * ey;
            jz += 0.5 * dt * ez;
        }

        for i in 0..self.n {
            self.atoms[i].x += dt * self.atoms[i].vx;
            self.atoms[i].y += dt * self.atoms[i].vy;
            self.atoms[i].z += dt * self.atoms[i].vz;
        }
        // THE WRAP. An atom that left one face re-enters through the opposite one. It is
        // done here, on the drift, and nowhere else — a coordinate is either canonical or
        // it is not, and two places that fold it is two places that can disagree about
        // whether it has been folded.
        //
        // It does NO work and delivers NO impulse: the wrap changes a coordinate by
        // exactly one box vector, and under the minimum-image convention every separation
        // in the scene is unchanged by that (`pbc_translation_residual` is the gate that
        // says so). Velocities are untouched. So neither ledger has anything to post,
        // which is why there is no `w_ext` line here and why its absence is a statement
        // rather than an omission.
        if self.boundary.wraps() {
            let geom = self.geom();
            for i in 0..self.n {
                let (x, y, z) = geom.wrap((self.atoms[i].x, self.atoms[i].y, self.atoms[i].z));
                self.atoms[i].x = x;
                self.atoms[i].y = y;
                self.atoms[i].z = z;
            }
        }

        self.compute_forces();

        for i in 0..self.n {
            let (px, py, pz) = self.a_pair[i];
            let (ex, ey, ez) = self.a_ext[i];
            let half = 0.5 * dt / self.atoms[i].mass();
            self.atoms[i].vx += half * (px + ex);
            self.atoms[i].vy += half * (py + ey);
            self.atoms[i].vz += half * (pz + ez);
            jx += 0.5 * dt * ex;
            jy += 0.5 * dt * ey;
            jz += 0.5 * dt * ez;
        }
        self.j_ext.0 += jx;
        self.j_ext.1 += jy;
        self.j_ext.2 += jz;

        self.accumulate_energy();

        if self.thermostat_on {
            self.apply_thermostat();
        }

        self.time += dt;
        self.steps += 1;

        // The energy drift EXTREMUM is tracked per substep; the energy GATE is still
        // evaluated at grain boundaries (`close_grain`), which is what closure-aligned
        // scheduling asks for. Splitting the two is not a liberty, it is a measured
        // necessity: the drift is a bounded OSCILLATION at the vibrational frequency, and
        // sampling it only at boundaries is stroboscopic. With dt = period/64, a frame of
        // 64 substeps is exactly one vibration, so every boundary lands at the same phase
        // and the gate reads a fixed point of the cycle rather than its peak. Measured
        // (examples/diagnose.rs, probe 5): at 64 and 128 substeps per frame the boundary
        // sample is 0.1110 of the true peak; at 16, 32, 48, 61, 63, 65 and 96 it is
        // 1.0000. A gate that goes blind exactly when the frame divides the period evenly
        // is a gate that fails on the tidy configuration and passes on the ragged one.
        //
        // The cost is about seven flops: `energy()` is four adds over terms this step has
        // already updated, and the rest is a subtraction and a compare.
        let d = self.drift();
        if d > self.drift_peak {
            self.drift_peak = d;
        }
        // The amplitude factor is tracked here, not only at boundaries, for the same
        // reason and at the same price: a collision that peaks between two boundaries
        // raises the mode energy the bound has to cover, and a boundary sample of it
        // would miss exactly the events that matter.
        let m = self.mode_energy();
        if m > self.e_ref {
            self.e_ref = m;
        }
    }

    /// Berendsen velocity rescaling. Whatever kinetic energy it adds or removes is
    /// posted to `w_ext` in the same breath, so a thermostatted run is still a closed
    /// ledger rather than an excused one.
    ///
    /// The rescaling also changes the total momentum (it multiplies every velocity),
    /// and that change is posted to `j_ext` for the same reason.
    fn apply_thermostat(&mut self) {
        let t_now = self.temperature();
        if t_now <= 0.0 {
            return;
        }
        let ratio = self.target_temperature / t_now;
        let lambda_sq = 1.0 + (self.dt() / self.thermostat_tau) * (ratio - 1.0);
        if lambda_sq <= 0.0 {
            return;
        }
        let lambda: f64 = lambda_sq.sqrt();
        let before = self.e_kin;
        let (pbx, pby, pbz) = self.momentum();
        for i in 0..self.n {
            self.atoms[i].vx *= lambda;
            self.atoms[i].vy *= lambda;
            self.atoms[i].vz *= lambda;
        }
        self.accumulate_energy();
        self.w_ext += self.e_kin - before;
        self.work.thermostat += self.e_kin - before;
        let (pax, pay, paz) = self.momentum();
        self.j_ext.0 += pax - pbx;
        self.j_ext.1 += pay - pby;
        self.j_ext.2 += paz - pbz;
    }

    // ---------------------------------------------------------------- the hand

    /// Grab atom `i`. The anchor starts ON the atom, so the spring enters the ledger at
    /// zero extension and the grab itself injects nothing.
    pub fn grab(&mut self, i: usize) {
        if i >= self.n {
            return;
        }
        self.grabbed = Some(i);
        self.anchor = (self.atoms[i].x, self.atoms[i].y, self.atoms[i].z);
        self.spring_engaged = true;
        self.compute_forces();
    }

    /// Move the anchor. The spring is a term in the Hamiltonian with a time-dependent
    /// parameter; moving that parameter at fixed atom position changes the stored spring
    /// energy by exactly `dU`, and `dU` IS the work the user's hand did. Posting it here
    /// is what keeps `E - W_ext` constant through a drag, with no path integral to
    /// approximate and no second-order error of its own.
    ///
    /// The 2D form holds the anchor's z, which on the mid-plane is the atom's own z, so
    /// `dz` stays an exact zero and the work posted is the float it always was.
    pub fn move_anchor(&mut self, x: f64, y: f64) {
        self.move_anchor_3d(x, y, self.anchor.2);
    }

    /// [`Sim::move_anchor`] with the third component. The work accounting is identical —
    /// it is `dU` of one spring term either way.
    pub fn move_anchor_3d(&mut self, x: f64, y: f64, z: f64) {
        let Some(g) = self.grabbed else { return };
        if g >= self.n {
            return;
        }
        let before = self.e_spring;
        self.anchor = (x, y, z);
        let dx = self.atoms[g].x - x;
        let dy = self.atoms[g].y - y;
        let dz = self.atoms[g].z - z;
        let after = 0.5 * K_SPRING * (dx * dx + dy * dy + dz * dz);
        self.w_ext += after - before;
        // WB-4.3: the same increment, into the hand's own receipt column. Posted from the
        // one computed value rather than recomputed, so the column cannot drift from the
        // total it is part of.
        self.work.hand += after - before;
        self.compute_forces();
    }

    /// Release. The energy still stored in the spring leaves the scene with the hand, so
    /// it is subtracted from `w_ext` — otherwise release would look like a free energy
    /// gain of exactly the stored amount.
    pub fn release(&mut self) {
        if self.grabbed.is_none() {
            return;
        }
        self.w_ext -= self.e_spring;
        self.work.hand -= self.e_spring;
        self.grabbed = None;
        self.compute_forces();
    }

    // ---------------------------------------------------------------- bonds

    /// Bond readings for every pair, from the table alone.
    ///
    /// A pair is BONDED when
    ///   (1) its relative energy is below the dissociation asymptote, and
    ///   (2) its separation is inside the outer classical turning point at that energy.
    ///
    /// Both come from the curve: (1) is `E_rel < E_asymptote`, which in the
    /// asymptote-zeroed convention is `E_rel < 0`; (2) solves `U_eff(R) = E_rel` on the
    /// same interpolant. There is no distance cutoff and no fitted threshold anywhere.
    ///
    /// Worth being straight about: for an ISOLATED pair, (2) is implied by (1). Any
    /// state the pair actually occupies satisfies `U_eff(R) <= E_rel` by construction
    /// (the leftover is the radial kinetic energy, which cannot be negative), so R is
    /// always inside the turning point. Condition (2) is therefore a redundancy check
    /// here rather than a second independent criterion — it can only fire if the
    /// turning-point solve and the energy disagree, which would mean the interpolant is
    /// not single-valued in the way the search assumes. It is kept because it is the
    /// stated criterion, because `r_outer` is worth displaying as the bond's reach, and
    /// because the redundancy is a live check on the table rather than a dead one.
    ///
    /// The consequence of (1) that the demo exists to show: two atoms alone, approaching
    /// from outside the well, ALWAYS have `E_rel >= 0` and can never bond, no matter how
    /// hard they are pushed together. Forming H2 requires taking energy out — a third
    /// atom to carry it away, a thermostat, or the user's own spring braking one of them
    /// — and the ledger says exactly how much left.
    pub fn refresh_pairs(&mut self) {
        let mut k = 0usize;
        // Every reading below — `e_rel`, `r_outer`, and therefore `bonded` — comes from
        // THE PAIR'S OWN CURVE. This is B1's second half: a mixed scene where the H-H and
        // X-X criteria differ must show them differing, and it does because `u` and
        // `outer_turning_point` are asked of the table this pair is served by.
        let species = self.species_slots();
        let geom = self.geom();
        // WHICH PAIRS ARE READ. With no declared truncation, every pair — the complete
        // reading this has always been. With one, the neighbour pairs only, and the pairs
        // that are dropped provably cannot be bonded: past the switch's outer edge `u` is
        // exactly zero, so `e_rel` is the relative kinetic energy, which is never
        // negative, and the criterion is `e_rel < 0`. The bond reading therefore does not
        // change; the count of PAIRS EXAMINED does, and `pair_count` says so.
        let listed: Option<Vec<(u32, u32)>> = if self.pair_switch.is_some() {
            Some(self.neighbours.pairs.iter().map(|p| (p.i, p.j)).collect())
        } else {
            None
        };
        let total = match &listed {
            Some(v) => v.len(),
            None => complete_pairs(self.n),
        };
        self.pairs.clear();
        self.pairs.reserve(total);
        let iter: Vec<(usize, usize)> = match &listed {
            Some(v) => v.iter().map(|&(a, b)| (a as usize, b as usize)).collect(),
            None => {
                let mut out = Vec::with_capacity(total);
                for i in 0..self.n {
                    for j in (i + 1)..self.n {
                        out.push((i, j));
                    }
                }
                out
            }
        };
        {
            for (i, j) in iter {
                let (dx, dy, dz) = geom.delta(
                    (self.atoms[i].x, self.atoms[i].y, self.atoms[i].z),
                    (self.atoms[j].x, self.atoms[j].y, self.atoms[j].z),
                );
                let r = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-9);
                let vx = self.atoms[j].vx - self.atoms[i].vx;
                let vy = self.atoms[j].vy - self.atoms[i].vy;
                let vz = self.atoms[j].vz - self.atoms[i].vz;
                let mi = self.atoms[i].mass();
                let mj = self.atoms[j].mass();
                let mu = (mi * mj) / (mi + mj);
                let ke_rel = 0.5 * mu * (vx * vx + vy * vy + vz * vz);
                let table = self.bank.table_at(species[i], species[j]);
                let u = table.u(r);
                let e_rel = ke_rel + u;
                // |L|^2 of the relative motion, for the centrifugal term. In 3D the
                // relative motion of an isolated pair is planar but the plane is not the
                // scene's, so the full cross product is needed — and it costs the 2D case
                // nothing, because on the mid-plane `dz` and `vz` are exact zeros, the
                // two transverse components are exactly `0.0`, and `l_sq` reduces to the
                // `L_z^2` this line used to compute, bit for bit.
                let lx = mu * (dy * vz - dz * vy);
                let ly = mu * (dz * vx - dx * vz);
                let lz = mu * (dx * vy - dy * vx);
                let l_sq = lx * lx + ly * ly + lz * lz;
                let r_outer = table.outer_turning_point(e_rel, l_sq, mu, r, TURNING_POINT_CAP);
                self.pairs.push(PairReading {
                    i,
                    j,
                    r,
                    e_rel,
                    r_outer,
                    bonded: e_rel < 0.0 && r < r_outer,
                });
                k += 1;
            }
        }
        self.pair_count = k;
    }

    pub fn bonded_count(&self) -> usize {
        self.pairs[..self.pair_count]
            .iter()
            .filter(|p| p.bonded)
            .count()
    }

    /// The CLUSTER reading: connected components of the bonded-pair graph.
    ///
    /// `bonded_count()` counts PAIRS, and the pair criterion is deliberately two-body:
    /// a pair reads BONDED when that pair, considered alone, is a bound system. Both
    /// facts are correct and together they mislead — 16 atoms collapsed into one cold
    /// droplet read 120 BONDED, because every one of the C(16,2) pairs genuinely is
    /// mutually bound (delete the other fourteen atoms and any pair you kept would stay
    /// bound; the 12-bohr tail of the well is still ~6e-6 Ha deep, so a cold pair at any
    /// separation the box allows has `E_rel < 0`). A field screenshot asking "16 atoms,
    /// 120 bonds?" is what surfaced the mismatch between the number and the noun.
    ///
    /// The chemically meaningful headline object is the component, not the edge: that
    /// droplet is ONE cluster of 16 atoms. This reading introduces no new criterion —
    /// the edge set is exactly the pairs already reading `bonded`, so it cannot disagree
    /// with the pair layer, and there is still no distance cutoff and no fitted
    /// threshold anywhere. Union-find with path halving; components of one atom are
    /// free atoms, not clusters.
    ///
    /// Returns `(clusters, atoms_in_clusters)`. Distinct from the census's MOLECULE
    /// count on purpose: a cluster is a statement about boundness, a molecule row is a
    /// statement about closure, and how far those disagree (the droplet: one cluster,
    /// few or no closed pair-composites, rejections climbing) is the boundness-vs-
    /// closure fence made visible.
    pub fn cluster_count(&self) -> (usize, usize) {
        let size = self.cluster_sizes();
        let clusters = size.iter().filter(|&&s| s >= 2).count();
        let atoms = size.iter().filter(|&&s| s >= 2).sum();
        (clusters, atoms)
    }

    /// The component SIZES behind [`Sim::cluster_count`], indexed by the component's root
    /// atom: entry `i` is the number of atoms in the component rooted at `i`, and zero for
    /// an atom that is not a root. Entries of 1 are free atoms; entries of 2 or more are
    /// clusters.
    ///
    /// Split out rather than duplicated so the quench's histogram and the headline count
    /// read ONE union-find over ONE edge set. Two implementations of a cluster reading is
    /// how the two of them come to disagree.
    pub fn cluster_sizes(&self) -> Vec<usize> {
        let roots = self.cluster_roots();
        let mut size = vec![0usize; self.n];
        for i in 0..self.n {
            size[roots[i]] += 1;
        }
        size
    }

    /// Each atom's component root, by union-find over the bonded-pair edge set.
    ///
    /// The single implementation everything else here is built from. `cluster_sizes` and
    /// [`Sim::cluster_species_counts`] are two READINGS of this one partition, not two
    /// partitions — which is what stops a size histogram and a composition histogram from
    /// disagreeing about how many molecules there are.
    fn cluster_roots(&self) -> Vec<usize> {
        let mut parent: Vec<usize> = (0..self.n).collect();
        fn find(parent: &mut [usize], mut i: usize) -> usize {
            while parent[i] != i {
                parent[i] = parent[parent[i]]; // path halving
                i = parent[i];
            }
            i
        }
        for p in self.pairs[..self.pair_count].iter().filter(|p| p.bonded) {
            let (a, b) = (find(&mut parent, p.i), find(&mut parent, p.j));
            if a != b {
                parent[a] = b;
            }
        }
        let mut roots = vec![0usize; self.n];
        for (i, r) in roots.iter_mut().enumerate() {
            *r = find(&mut parent, i);
        }
        roots
    }

    /// THE COMPOSITION READING: how many atoms of each nuclear charge each component holds.
    ///
    /// Entry `i` is `[(Z, count); ...]` for the component rooted at atom `i`, with unused
    /// entries carrying `Z = 0`. Empty for an atom that is not a root.
    ///
    /// This is what makes a MOLECULE reading possible in a mixed box: a component of two
    /// atoms is a dimer, and whether it is H2, HCl or Cl2 is a fact about which nuclei are
    /// in it. `cluster_sizes` alone cannot tell those three apart, and the whole of gate
    /// P1 is the difference between them.
    ///
    /// Keyed by nuclear charge rather than by the bank's species index deliberately: the
    /// species index is an artefact of registration order and would make a run's output
    /// depend on which atom happened to be placed first.
    pub fn cluster_species_counts(&self) -> Vec<[(u32, usize); MAX_SPECIES]> {
        let roots = self.cluster_roots();
        let mut out = vec![[(0u32, 0usize); MAX_SPECIES]; self.n];
        for i in 0..self.n {
            let z = self.atoms[i].species.z;
            let row = &mut out[roots[i]];
            match row.iter_mut().find(|(rz, _)| *rz == z || *rz == 0) {
                Some(slot) => {
                    slot.0 = z;
                    slot.1 += 1;
                }
                // Unreachable while the bank caps species at MAX_SPECIES and every atom's
                // species is registered, which `set_species` enforces. Dropped rather than
                // panicking in the physics core; the count would be visibly short.
                None => {}
            }
        }
        out
    }
}
