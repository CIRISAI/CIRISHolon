//! The zoom ladder: which grain constant each tier declares, and what the REG+ ledger
//! will actually let it count.
//!
//! # The ladder is arithmetic, not taste
//!
//! `GrossState` has FOUR integer lanes and `Holon::grain_units` is a `u32`. Together
//! they decide how far one arena can zoom, and they bind hard:
//!
//! * a dense 3D scene fits `u64` only while `extent / g0 <= 2.642e6` — **6.42 decades**;
//! * `grain_units` is a diameter ratio, so no arena spans more than `4.295e9` — **9.63
//!   decades** — whatever its density.
//!
//! ## Which lane binds is a property of the CHART, not of the ledger
//!
//! The obvious lane is `constituents`, and for this demo it is the right one — but only
//! because of something the chart does. `GrossState` also carries `occupancy: u64` and
//! `momentum: [i64; 2]`, and a chart that writes REG+ occupancy per leaf overflows
//! EARLIER than the constituent count does. Credit to the 4090 study for catching this;
//! the first version of this header quoted the constituents lane as if it were the only
//! one.
//!
//! One 0.5 mm quartz grain is **5.216e18 atoms**. Against that:
//!
//! | what the chart writes per leaf | binding lane | cap | grains of sand |
//! |---|---|---:|---:|
//! | occupancy 0 (this demo) | `constituents` | 1.845e19 | **3.54** |
//! | occupancy 2 (the `gross(n) = aggregate(n, 2n, …)` idiom) | `occupancy` | 9.223e18 | 1.77 |
//! | occupancy 6 (REG+ maximum, all six directions) | `occupancy` | 3.075e18 | **0.59** |
//! | momentum component 2 (the enumerated FHP maximum) | `momentum` | 4.612e18 | 0.885 |
//!
//! So "the ledger holds three and a half grains of sand" is a statement about THIS
//! chart. The general bound is 0.59 grains, six times tighter, and it is reached by any
//! chart whose leaves carry full REG+ occupancy. [`ledger_for`] checks every lane;
//! `tests::the_binding_lane_is_a_property_of_the_chart` pins both readings, and
//! `scene::tests::the_sandbox_chart_writes_no_occupancy` is what earns this demo the
//! looser one.
//! The sandbox in this demo holds 6.60e8 grains; expressed in atoms that is 3.44e27,
//! which is **1.87e8 times** over the cap, and `GrossState::checked_combine` returns
//! `None` rather than a wrong number. The observable universe in atoms is over by 5.4e60.
//!
//! That refusal is the frame working, and it is why zoom here is a **re-root**: each
//! tier is its own arena with its own declared `g0`, which `CIRIS_HOLON_ENGINE.md`
//! already licenses ("There is no absolute maximal holon. A root can become a child when
//! a larger holarchy is formed"). Re-rooting is a VALUES change — a different grain
//! constant on the scene chart — not a new object, not a new field, not a second
//! ontology. [`Tier::ledger`] is that arithmetic made executable, and
//! [`Ledger::Overflows`] is what the demo shows when a user asks for a grain the ledger
//! cannot carry.
//!
//! # Square domains, and a debt that turned out not to be owed
//!
//! Every tier's domain is SQUARE, so the 2x2 quadtree halves both axes and cells stay
//! square at every depth. `ciris_sim_core::fracture::WallChart` is square-only and that
//! was on the list of debts to pay here; choosing square domains means the demo does not
//! need per-axis extents at all. A sandbox is a square domain with sand in the lower
//! part and air above — the air cells are ordinary holons carrying zero constituents,
//! and the ledger composes exactly through them. Building rectangular charts that
//! nothing uses would have been machinery, not payment.
//!
//! # Two dimensions on purpose
//!
//! The chart is 2D; the LEDGER is 3D. A tier's `constituents` is the real count of
//! physical terminal holons — grains, unit cells, galaxies — while the quadtree cells
//! are the resident refinement window over them. That split is the shipped demo's own
//! (`holon-ball-game` carries a 1,000,000-holon wall on 288 2D nodes) and it is stated
//! here rather than left for a reader to infer.

use ciris_sim_core::material::IsotropicMaterial;

/// Avogadro's constant, CODATA 2018 (exact by SI definition since 2019).
pub const AVOGADRO: f64 = 6.022_140_76e23;
/// Molar mass of SiO2, kg/mol.
pub const SIO2_MOLAR_MASS_KG: f64 = 0.060_08;
/// Atoms per SiO2 formula unit.
pub const SIO2_ATOMS_PER_UNIT: f64 = 3.0;

/// The eight zoom tiers, coarse index ascending outward from the gauge floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TierId {
    Gauge = 0,
    Crystal = 1,
    Grain = 2,
    Sandbox = 3,
    Landscape = 4,
    Planet = 5,
    Galactic = 6,
    Cosmic = 7,
}

impl TierId {
    pub const ALL: [TierId; 8] = [
        TierId::Gauge,
        TierId::Crystal,
        TierId::Grain,
        TierId::Sandbox,
        TierId::Landscape,
        TierId::Planet,
        TierId::Galactic,
        TierId::Cosmic,
    ];

    pub fn from_index(index: u32) -> Option<Self> {
        Self::ALL.get(index as usize).copied()
    }

    pub const fn index(self) -> u32 {
        self as u32
    }
}

/// What can actually be evaluated at a tier, today, in this repository.
///
/// One of the eight tiers carries a refusal rather than an evaluator, and it names both
/// the open item that produces it and the gate whose passing would lift it. Three others
/// USED to refuse for want of gravity; `ciris_sim_core::bridge` closed that, and they now
/// evaluate on a declared weak-field chart — but only for scenes inside their screen, and
/// a scene outside it still refuses BY NAME.
// `Eq` is deliberately absent: the geodesic stake is an `f64`, and a tier stake is a
// number to be compared against, never a key. `PartialEq` is all any caller needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Evaluator {
    /// Exact finite algebra: the one-plaquette U(1) quantum link.
    GaugePlaquette,
    /// Grain-on-grain unilateral contact with Coulomb friction.
    GranularContact,
    /// Bilinear cohesive relations with irreversible damage, over
    /// `ciris_sim_core::material::CohesiveBond`.
    Cohesive,
    /// Geodesic motion on a declared static weak-field chart, screened by
    /// `bridge::certify_weak_field` against this FROZEN tier stake.
    ///
    /// The stake is a value on the evaluator rather than a constant in the solver,
    /// because it is what the tier is committing to and it differs per tier by four
    /// orders of magnitude. It is not a tuning knob: `bridge` exports each one and the
    /// galactic tier's has a live amendment (A1) that the demo shows both readings of.
    GeodesicChart { tier_eps_max: f64 },
    /// No validated evaluator exists at this tier in this repository.
    Unavailable(Refusal),
}

/// Why a tier refuses, in the words of the document that owns the gap.
///
/// Every refusal carries its own UNLOCK — the specific gate whose passing would remove
/// it. A refusal that names what would lift it is a roadmap; one that does not is just a
/// shortfall, and the difference is entirely in whether anyone wrote the second half
/// down. [`Refusal::unlock`] is that half.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// T1/T2 are specification-only. There are no force constants in the tree, and the
    /// DFT elastic tensor is Phase 2 IN PROGRESS (`T2_DFT_REFERENCE.md`) — 12 strained
    /// -cell relaxations owed. Rendering a lattice from published cell parameters is
    /// data; running dynamics on it would be a number with no ancestor.
    NoValidatedEvaluator,
    /// `relativity.rs` is flat by design and contains no gravity. INTEGRATION_FRAME.md
    /// L29 states it plainly: every engine scene with weight is outside the T5
    /// certificate until the curved tier (A3) closes. A gravitating scene therefore has
    /// a ledger and no certified dynamics.
    NoGravityChart,
}

impl Refusal {
    /// What would have to happen for this tier to stop refusing.
    pub const fn unlock(self) -> &'static str {
        match self {
            // T2_DFT_REFERENCE.md: Phase 1 (structure) COMPLETE, Phase 2 (elastic
            // tensor) IN PROGRESS — 12 strained-cell relaxations, 6 done at the time of
            // writing. When that tensor lands and passes the T2 gate, this tier has
            // force constants with an ancestor and can run.
            Refusal::NoValidatedEvaluator => {
                "Awaits the T2 gate. The DFT elastic tensor is computing now — 6 of 12                  strain runs done — and this tier can run the moment it passes."
            }
            // PROGRAM.md A3 / INTEGRATION_FRAME.md P1: the curved tier. T5 today covers
            // force-free and EM Newtonian charts only, which is to say weightless
            // scenes.
            Refusal::NoGravityChart => {
                "Awaits the curved-tier certificate. The flat chart covers weightless                  scenes today; weight needs the curved tier (PROGRAM.md A3)."
            }
        }
    }
}

/// Which lane of the REG+ ledger a scene is measured against.
///
/// A lane is a field of [`ciris_sim_core::regplus::GrossState`], and which one binds
/// depends on what the chart writes into it — see the module header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    /// `constituents: u64`. Binds when the chart writes no occupancy and no momentum.
    Constituents,
    /// `occupancy: u64`, the summed REG+ occupancy. Binds whenever leaves carry any.
    Occupancy,
    /// `momentum: [i64; 2]`, so half the range of the other two before the sign bit.
    Momentum,
}

impl Lane {
    /// Largest value this lane can hold.
    pub const fn capacity(self) -> u128 {
        match self {
            Lane::Constituents | Lane::Occupancy => u64::MAX as u128,
            Lane::Momentum => i64::MAX as u128,
        }
    }
}

/// What the chart writes into each ledger lane, per TERMINAL holon.
///
/// This is a property of the realization, not of the engine, and it is what decides
/// which lane runs out first. The REG+ maximum is 6 (all six FHP directions occupied)
/// for occupancy and 2 for either momentum component — enumerated from the lattice
/// (see `REG_PLUS_MAX` below and its deriving test), not asserted; "3" was the
/// hand-reasoned value the enumeration refuted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeafWrites {
    pub occupancy: u64,
    pub momentum: u64,
}

impl LeafWrites {
    /// This demo's chart: a purely geometric partition that writes neither. It is the
    /// reason the sandbox gets the loose 3.54-grain reading rather than 0.59.
    pub const GEOMETRIC: Self = Self {
        occupancy: 0,
        momentum: 0,
    };

    /// The REG+ maximum, ENUMERATED over all 64 local states rather than assumed.
    ///
    /// Occupancy 6 is all six FHP directions occupied. Momentum is **2**, not 3: the six
    /// directions in `regplus::DIRECTIONS` are axial, so no subset of them sums past 2 in
    /// either component — the witness is state 12, which reads (−2, 1). I had written 3
    /// by reasoning about six directions instead of adding them up; the FCHC enumeration
    /// in `MESH_DESIGN.md` M-G4 caught it, and
    /// `tests::the_reg_plus_maxima_are_enumerated_not_asserted` now derives both from
    /// the shipped lattice so neither can drift from it again.
    ///
    /// SUPERSEDES the lane attribution in SANDBOX_4090.md §2: the momentum lane holds
    /// **0.885 grains**, not the 0.59 quoted there. The HEADLINE is unaffected — the
    /// occupancy lane still binds first at 0.59 — but momentum now binds strictly LATER
    /// than occupancy, so for a chart writing full REG+ state it never binds at all.
    pub const REG_PLUS_MAX: Self = Self {
        occupancy: 6,
        momentum: 2,
    };
}

/// What the REG+ ledger says about a proposed (domain, grain) pair.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ledger {
    Fits {
        /// Terminal holons the scene claims.
        constituents: u64,
        /// `domain / g0`, the root's grain-unit count.
        grain_ratio: f64,
        /// The lane with the least headroom left. Which one it is, is a fact about the
        /// chart.
        binding: Lane,
    },
    /// A ledger lane would overflow. `factor` is how many times over.
    Overflows { lane: Lane, factor: f64 },
    /// `domain / g0` exceeds `u32::MAX`, so no root grain-unit count can express it.
    OverflowsGrainUnits { factor: f64 },
}

impl Ledger {
    pub const fn fits(&self) -> bool {
        matches!(self, Ledger::Fits { .. })
    }
}

/// Where a tier's constituent count comes from — a provenance, not a number.
///
/// Related fence, machine-checked in `CIRISOntology/Core/GrainFloor.lean`:
/// `capacity_irrelevant` proves that admissibility does not depend on how many holons a
/// tier can hold. A census is about what IS there; whether a claim can be served is
/// about `g0` against the claim's own length, and the two never trade against each other.
///
/// The two are checked differently and must be, because they are different claims. A
/// geometric census is an arithmetic consequence of the tier's own declared geometry and
/// is checked EXACTLY. An observed census is a measurement of how much of a mostly empty
/// volume is actually occupied, and the only thing arithmetic can say about it is that
/// it does not exceed the number of cells available to hold it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Census {
    /// `constituents = (domain/g0)^3 * fill * packing`, exactly. `packing` is the
    /// fraction of the filled region that is matter: 1.0 for a solid, 0.60 for randomly
    /// packed spheres, pi/6 for a sphere inscribed in the square domain.
    Geometric { packing: f64 },
    /// A measured count of bodies in a mostly empty volume. Space is not packed and no
    /// packing fraction would be honest about it; what is checked instead is that the
    /// count fits in the cells available.
    Observed,
}

/// One tier of the ladder. Every field is a VALUE on the same holon machinery; nothing
/// here is a per-tier type.
#[derive(Clone, Copy, Debug)]
pub struct Tier {
    pub id: TierId,
    pub name: &'static str,
    /// One line a general reader can hold on to.
    pub plain: &'static str,
    /// The declared grain constant in metres. `f64::NAN` at the gauge tier, where the
    /// lattice spacing is not pinned to any length and saying otherwise would invent a
    /// number.
    pub g0_m: f64,
    /// Side of the square domain, metres. `f64::NAN` at the gauge tier.
    pub domain_m: f64,
    /// Root grain-unit count: the domain measured in `g0`, rounded up to a power of two
    /// so the 2x2 quadtree bottoms out exactly at `g0`.
    pub root_grain_units: u32,
    /// Physical terminal holons in the whole scene — the 3D count, not the 2D cell
    /// count.
    pub constituents: u64,
    /// Fraction of the square domain's HEIGHT below the matter line. Air above carries
    /// zero constituents and composes exactly.
    ///
    /// This is geometry only. It used to double as the volume packing fraction as well,
    /// which is how the sandbox came to declare a matter line at 45% of the height and a
    /// constituent count implying 36% of the volume — two numbers that cannot both be
    /// right about one box. Building the re-root gate is what surfaced it.
    pub fill: f64,
    /// Where this tier's constituent count comes from.
    pub census: Census,
    pub evaluator: Evaluator,
    /// The chart values this tier's evaluator reads, where it has one.
    pub material: Option<IsotropicMaterial>,
    /// What the terminal holon at this tier IS, in plain words.
    pub terminal: &'static str,
}

/// Single-crystal alpha-quartz, as a grain's own chart values.
///
/// Provenance, per field, because a value with no ancestor is the thing this programme
/// exists to refuse:
/// * `young_modulus_pa` 95 GPa — the isotropic (Voigt-Reuss-Hill class) reduction of
///   the trigonal tensor in DESCRIPTOR_CHAIN.md §3.2 (C11 86.7-87.3, C33 105.8-107.2,
///   C44 57.2-58.2 GPa). It is a REDUCTION and it destroys C14 (~18 GPa) identically —
///   misfit L24, carried here rather than hidden: zooming to the crystal must MINT
///   anisotropy, never interpolate toward it.
/// * `fracture_energy_j_m2` 1.0 — twice the quartz surface energy, T2's exported
///   `gamma` 0.4-1.0 J/m^2 (§3.2), taken at 0.5. This is the TRANSPORTED value.
///   `IsotropicMaterial::DEMO_CALIBRATION` carries 110 J/m^2 at T4, and the 110x
///   difference is process-zone dissipation MINTED at the continuum tier (misfit
///   L23/M25). The two numbers are differently typed and the demo shows both.
/// * `tensile_strength_pa` 150 MPa — the middle of the §3.3 grain-strength range
///   50-300 MPa. In the running demo this field is REPLACED per grain by the quenched
///   Weibull draw, which is the whole point of the grain tier.
/// * `density_kg_m3` 2648 — the §3.2 pin.
/// * `poisson_ratio` 0.08 — quartz is anomalously low; a class value, flagged as such.
/// * `material_damping_ratio` 5.0e-4 — Q ~ 1e3 class warrant only, no specimen.
/// * `solver_damping_ratio` — solver stabilization, no physics ancestor, named as such
///   (amendment A5).
pub const QUARTZ_GRAIN: IsotropicMaterial = IsotropicMaterial {
    density_kg_m3: 2_648.0,
    young_modulus_pa: 95.0e9,
    poisson_ratio: 0.08,
    material_damping_ratio: 5.0e-4,
    solver_damping_ratio: 0.02,
    tensile_strength_pa: 150.0e6,
    compressive_strength_pa: 1.1e9,
    fracture_energy_j_m2: 1.0,
};

/// The smallest feature a viewer can distinguish on the stage, in stage pixels.
///
/// The observer is a CLAIMANT. Their acuity at the current zoom is a demand on the
/// frontier exactly like the physics claim is: resolve to what can be told apart on the
/// canvas. Before this the only claim was the impact, so certification correctly refused
/// to spend resolution anywhere else and the scene coarsened away from the corridor —
/// right about the physics, and a picture nobody asked for. The certified frontier is
/// now the JOIN of the two claims: at least acuity-fine everywhere in view, and finer
/// than that wherever the impulse needs it.
///
/// Declared here rather than buried, because it sets the cost of every scene. Three
/// pixels of a 900-pixel stage is about the finest a viewer resolves on a canvas
/// displayed near 600 CSS pixels; it is a claim about eyes, and if it is wrong it is
/// wrong in one visible place.
pub const ACUITY_PIXELS: f64 = 3.0;

/// Stage size the acuity is quoted against, in pixels. Matches the canvas.
pub const STAGE_PIXELS: f64 = 900.0;

/// Diameter of the median sand grain in this sandbox, metres. Medium sand on the
/// Wentworth scale is 0.25-0.5 mm; 0.5 mm is its coarse edge and the `g0` of the
/// sandbox tier.
pub const SAND_GRAIN_M: f64 = 5.0e-4;

/// Atoms in one sphere of quartz of diameter `d`.
pub fn atoms_in_grain(diameter_m: f64) -> f64 {
    let volume = core::f64::consts::PI / 6.0 * diameter_m * diameter_m * diameter_m;
    let mass = volume * QUARTZ_GRAIN.density_kg_m3;
    mass / SIO2_MOLAR_MASS_KG * AVOGADRO * SIO2_ATOMS_PER_UNIT
}

/// What the ledger says about counting `domain_m` of dense matter in grains of `g0_m`,
/// with a chart that writes `writes` into each lane per terminal holon.
///
/// `packing` is the volume fraction actually occupied (1.0 for a solid, ~0.6 for
/// randomly packed spheres). This is the executable form of the module header's
/// arithmetic: EVERY lane is checked, and the one with the least headroom is reported
/// whether it overflows or not, because "which lane binds" is the fact a chart author
/// needs and cannot get from the constituent count alone.
pub fn ledger_for(domain_m: f64, g0_m: f64, packing: f64, writes: LeafWrites) -> Ledger {
    let ratio = domain_m / g0_m;
    if !(ratio.is_finite() && ratio > 0.0) || ratio > u32::MAX as f64 {
        return Ledger::OverflowsGrainUnits {
            factor: ratio / u32::MAX as f64,
        };
    }
    let count = ratio * ratio * ratio * packing;

    let lanes = [
        (Lane::Constituents, count),
        (Lane::Occupancy, count * writes.occupancy as f64),
        (Lane::Momentum, count * writes.momentum as f64),
    ];
    let mut binding = Lane::Constituents;
    let mut worst = 0.0_f64;
    for (lane, total) in lanes {
        let used = total / lane.capacity() as f64;
        if used > worst {
            worst = used;
            binding = lane;
        }
    }
    if worst > 1.0 {
        return Ledger::Overflows {
            lane: binding,
            factor: worst,
        };
    }
    Ledger::Fits {
        constituents: count as u64,
        grain_ratio: ratio,
        binding,
    }
}

/// Smallest power of two at or above `value`, saturating at `u32::MAX`'s highest power.
///
/// The ladder's `root_grain_units` are written out as literals rather than computed, so
/// that a typo is a test failure instead of a silently different scene. This is the
/// function `declared_root_grain_covers_the_domain` checks them against.
pub fn ceil_pow2(value: f64) -> u32 {
    if !(value.is_finite() && value > 1.0) {
        return 1;
    }
    let mut units: u32 = 1;
    while (units as f64) < value {
        match units.checked_mul(2) {
            Some(next) => units = next,
            None => return units,
        }
    }
    units
}

/// The ladder.
///
/// Two entries had to be RE-SIZED because the caps bit during construction, and both are
/// left visible in the numbers rather than smoothed away: a 1 m grain on Earth is 1.08e21
/// constituents (59x over `u64`), so the planet tier declares 10 m; and a 10 kpc grain on
/// the observable universe is 2.32e19 geometric cells (1.26x over), so the cosmic tier
/// declares 40 kpc. A ladder chosen for looks would not have been bent twice by
/// arithmetic.
pub fn tiers() -> [Tier; 8] {
    [
        Tier {
            id: TierId::Gauge,
            name: "gauge",
            plain: "One square of the vacuum's own bookkeeping: four links carrying \
                    electric flux, and a rule that says how much is allowed to meet at \
                    a corner.",
            g0_m: f64::NAN,
            domain_m: f64::NAN,
            root_grain_units: 1,
            constituents: 4,
            fill: 1.0,
            census: Census::Geometric { packing: 1.0 },
            evaluator: Evaluator::GaugePlaquette,
            material: None,
            terminal: "one oriented link",
        },
        Tier {
            id: TierId::Crystal,
            name: "crystal",
            plain: "A speck of quartz one micrometre across, drawn as the repeating \
                    unit cell it is really made of.",
            g0_m: 4.913_4e-10,
            domain_m: 1.0e-6,
            root_grain_units: 2048,
            constituents: 8_430_505_878,
            fill: 1.0,
            census: Census::Geometric { packing: 1.0 },
            evaluator: Evaluator::Unavailable(Refusal::NoValidatedEvaluator),
            material: None,
            terminal: "one alpha-quartz unit cell",
        },
        Tier {
            id: TierId::Grain,
            name: "grain",
            plain: "One grain of sand, half a millimetre across, and the flaws inside \
                    it that decide how it breaks.",
            g0_m: 1.0e-6,
            domain_m: SAND_GRAIN_M,
            root_grain_units: 512,
            constituents: 125_000_000,
            fill: 1.0,
            census: Census::Geometric { packing: 1.0 },
            evaluator: Evaluator::Cohesive,
            material: Some(QUARTZ_GRAIN),
            terminal: "one micrometre of crystal",
        },
        Tier {
            id: TierId::Sandbox,
            name: "sandbox",
            plain: "Sand, in a box. Throw something at it and the grains move because \
                    they push on each other, not because anything was animated.",
            g0_m: SAND_GRAIN_M,
            domain_m: 0.6,
            root_grain_units: 2048,
            // 1200^3 cells x 0.45 of the height x 0.60 randomly-packed spheres.
            //
            // SUPERSEDES the 6.2208e8 quoted in SANDBOX_4090.md G7. That figure came
            // from `fill` doing double duty as both the height fraction and the packing
            // fraction, so the declared matter line (45% of the height) and the declared
            // count (36% of the volume) described two different boxes. The 4090 study
            // spotted the disagreement; `a_declared_census_matches_its_own_geometry` is
            // what now prevents it recurring.
            constituents: 466_560_000,
            fill: 0.45,
            census: Census::Geometric { packing: 0.60 },
            evaluator: Evaluator::GranularContact,
            material: Some(QUARTZ_GRAIN),
            terminal: "one grain of sand",
        },
        Tier {
            id: TierId::Landscape,
            name: "landscape",
            plain: "Two kilometres of stone ground. This is the tier the engine's \
                    fracture work was actually built and gated for.",
            g0_m: 1.0e-2,
            domain_m: 2.0e3,
            root_grain_units: 262_144,
            constituents: 4_800_000_000_000_000,
            fill: 0.6,
            census: Census::Geometric { packing: 1.0 },
            evaluator: Evaluator::Cohesive,
            material: Some(IsotropicMaterial::DEMO_CALIBRATION),
            terminal: "one centimetre of rock",
        },
        Tier {
            id: TierId::Planet,
            name: "planet",
            plain: "The Earth, counted in ten-metre blocks — and here weight finally \
                    pulls. A thrown ball falls along a geodesic of a declared curved \
                    chart, certified at a measured 7.0e-10 with the leftover error \
                    bounded below five parts in a billion billion.",
            g0_m: 1.0e1,
            domain_m: 1.274_2e7,
            root_grain_units: 2_097_152,
            // A sphere inscribed in the square domain: pi/6 of the cells hold Earth.
            constituents: 1_083_206_916_845_753_600,
            fill: 1.0,
            census: Census::Geometric {
                packing: core::f64::consts::PI / 6.0,
            },
            evaluator: Evaluator::GeodesicChart {
                tier_eps_max: ciris_sim_core::bridge::PLANET_EPS_MAX,
            },
            material: Some(IsotropicMaterial::DEMO_CALIBRATION),
            terminal: "one ten-metre block",
        },
        Tier {
            id: TierId::Galactic,
            name: "galactic",
            plain: "The Milky Way, counted in stars. A hundred billion fit in the \
                    ledger with room to spare, and the star S2 rounding the black hole \
                    at the centre is close enough to it that curvature is the thing \
                    being measured.",
            g0_m: 3.085_7e16,
            domain_m: 9.46e20,
            root_grain_units: 32_768,
            constituents: 100_000_000_000,
            fill: 1.0,
            census: Census::Observed,
            evaluator: Evaluator::GeodesicChart {
                tier_eps_max: ciris_sim_core::bridge::GALACTIC_EPS_MAX,
            },
            material: None,
            terminal: "one star",
        },
        Tier {
            id: TierId::Cosmic,
            name: "cosmic",
            plain: "Everything we can see, counted in galaxies: two trillion of them, \
                    which the same ledger carries without complaint. A patch small \
                    enough certifies; ask for one three times wider and the expansion \
                    of space refuses it.",
            g0_m: 1.2e21,
            domain_m: 8.8e26,
            root_grain_units: 1_048_576,
            constituents: 2_000_000_000_000,
            fill: 1.0,
            census: Census::Observed,
            evaluator: Evaluator::GeodesicChart {
                tier_eps_max: ciris_sim_core::bridge::COSMIC_EPS_MAX,
            },
            material: None,
            terminal: "one galaxy",
        },
    ]
}

pub fn tier(id: TierId) -> Tier {
    tiers()[id.index() as usize]
}

impl Tier {
    /// Cells of size `g0` in this tier's square domain, in three dimensions.
    pub fn geometric_cells(&self) -> f64 {
        let ratio = self.domain_m / self.g0_m;
        ratio * ratio * ratio
    }

    /// Fraction of this tier's cells that hold matter.
    pub fn occupancy(&self) -> f64 {
        match self.census {
            Census::Geometric { packing } => self.fill * packing,
            Census::Observed => self.constituents as f64 / self.geometric_cells(),
        }
    }

    /// What the ledger says about this tier as declared.
    pub fn ledger(&self) -> Ledger {
        if !self.domain_m.is_finite() {
            return Ledger::Fits {
                constituents: self.constituents,
                grain_ratio: 1.0,
                binding: Lane::Constituents,
            };
        }
        ledger_for(self.domain_m, self.g0_m, self.occupancy(), LeafWrites::GEOMETRIC)
    }

    /// What the ledger says if this tier's domain is counted in ATOMS instead of its
    /// declared grain. This is the demo's zoom-out refusal, computed rather than
    /// asserted: ask for the sandbox in atoms and the answer is a number of times over
    /// the cap, not a count.
    pub fn ledger_in_atoms(&self) -> Ledger {
        if !self.domain_m.is_finite() {
            return Ledger::OverflowsGrainUnits { factor: f64::NAN };
        }
        // One atom of quartz, as an equivalent sphere diameter: the cube root of the
        // volume per atom. This is a size, not a claim about atomic structure.
        let volume_per_atom = SIO2_MOLAR_MASS_KG
            / (SIO2_ATOMS_PER_UNIT * AVOGADRO * QUARTZ_GRAIN.density_kg_m3);
        let atom_m = volume_per_atom.cbrt();
        ledger_for(self.domain_m, atom_m, self.occupancy(), LeafWrites::GEOMETRIC)
    }

    /// `l_ch = E * G_F / f_t^2` for this tier's chart values: the length over which a
    /// crack in this material hands its energy over. `None` where the tier has no
    /// material chart.
    ///
    /// This one formula, evaluated on each tier's own VALUES, is what makes the tiers
    /// disagree. At the landscape tier it is 13.75 cm, so the process zone it demands
    /// (a tenth of that) is 27x COARSER than a grain of sand — the continuum chart
    /// cannot see a grain at all. At the grain tier it is 4.2 um, so the demand is
    /// sub-micrometre: below that tier's own floor.
    pub fn characteristic_length_m(&self) -> Option<f64> {
        let material = self.material?;
        Some(
            material.young_modulus_pa * material.fracture_energy_j_m2
                / (material.tensile_strength_pa * material.tensile_strength_pa),
        )
    }

    /// The grain the OBSERVER claims: the domain length spanning one distinguishable
    /// feature on the stage.
    ///
    /// Never finer than `g0`. There is nothing below the tier's own terminal holon to
    /// show, and rendering finer would be inventing sub-grain detail — the one thing a
    /// reader who zooms in deserves to be able to trust is that each cell they see is a
    /// holon that exists.
    pub fn acuity_m(&self) -> f64 {
        if !self.domain_m.is_finite() {
            return f64::NAN;
        }
        (self.domain_m * ACUITY_PIXELS / STAGE_PIXELS).max(self.g0_m)
    }

    /// Roughly how many resident cells the observer's claim costs at this tier: the
    /// quadtree depth that reaches acuity, over the fraction of the domain holding
    /// matter. Used to size the holon budget, so a scene is never refused for the crime
    /// of being visible.
    pub fn acuity_cell_estimate(&self) -> f64 {
        if !self.domain_m.is_finite() {
            return 0.0;
        }
        let divisions = (self.domain_m / self.acuity_m()).max(1.0).log2().ceil();
        let across = 2.0_f64.powf(divisions);
        // The tree above the leaves adds about a third again.
        across * across * self.fill * 4.0 / 3.0
    }

    /// Is the observer's claim servable at this tier at all? False when acuity would be
    /// finer than the tier's own grain — a refusal with the same standing as any other,
    /// never a reason to quietly coarsen.
    pub fn acuity_servable(&self) -> bool {
        !self.domain_m.is_finite() || self.acuity_m() >= self.g0_m
    }

    /// The cell spacing this tier's claim demands where the interaction is.
    ///
    /// Machine-checked consequence, `CIRISOntology/Core/GrainFloor.lean`: whether a
    /// claim is servable at a tier is `g0 <= claim length`, and `inadmissible_persists`
    /// proves no amount of refinement INSIDE a tier changes that answer. The grain
    /// tier's crack claim asks for 4.222e-7 m against a 1e-6 m floor — 2.37x short, and
    /// short by exactly that much however many holons are spent. `capacity_irrelevant`
    /// says the same thing from the other side: more holons is never the route.
    /// `admissibility_change_is_reroot` names the route that is: a different tier.
    ///
    /// The demand depends on WHAT IS BEING CLAIMED, and the two tiers that run claim
    /// different things:
    ///
    /// * a COHESIVE tier claims a crack path and a work of fracture, so it must resolve
    ///   the process zone: `l_ch / 10`, the requirement DESCRIPTOR_CHAIN.md §3.4 states
    ///   and `fracture.rs` enforces;
    /// * a GRANULAR tier claims a contact impulse between grains, so it must resolve a
    ///   GRAIN: `g0`. A cell coarser than one grain cannot represent grains moving past
    ///   each other, and a cell finer than one does not exist.
    ///
    /// Using `l_ch / 10` at the sandbox would demand 0.42 um cells of a tier whose
    /// terminal holon is 500 um, and the honest answer to that demand is `GrainFloor`
    /// forever. That is the right answer to the WRONG QUESTION: the sandbox is not
    /// claiming a crack, because at 0.5 mm cells the homogenizer has already refused it
    /// a cohesive law (`scene::relation_law`). The surrogate follows the claim.
    pub fn required_spacing_m(&self) -> Option<f64> {
        match self.evaluator {
            Evaluator::GranularContact => Some(self.g0_m),
            Evaluator::Cohesive => Some(self.characteristic_length_m()? / 10.0),
            // A geodesic claim is screened on `epsilon`, not on cell spacing: what it
            // demands of the frontier is that the observer can see it, and that demand
            // arrives through `acuity_m` like every other tier's.
            Evaluator::GeodesicChart { .. }
            | Evaluator::GaugePlaquette
            | Evaluator::Unavailable(_) => None,
        }
    }
}

/// How a finer tier's domain sits inside the tier above it.
///
/// Zooming is a RE-ROOT: a new arena with a new grain constant. Nothing is shared —
/// not ids, not the frontier, not the certificate — so the one thing that must survive
/// the transition is the LEDGER, and this is the relation it has to survive through.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reroot {
    /// The child's whole domain is exactly ONE of the parent's terminal holons. The
    /// strongest form, and the one that admits an exact ledger identity: the child's
    /// constituent count must equal `(parent.g0 / child.g0)^3` times its own occupancy.
    OneTerminalHolon { child_per_parent: f64 },
    /// The child's domain spans a whole number of the parent's terminal holons.
    WholeMultiple { parents: f64 },
    /// The child's domain lies inside one parent terminal holon without filling it.
    /// Legitimate — zooming to a sub-part of one grain is a real thing to do — but it
    /// carries no exact count identity, and saying so is the point of separating it.
    Contained { fraction: f64 },
}

/// The ledger relation between a tier and the one above it, or `None` at the ends of
/// the ladder and wherever a tier has no length.
///
/// This is the TIER-TRANSITION CERTIFICATE. The fracture certificate says a frontier
/// resolves what its tier claims; this one says a zoom lands somewhere the ledger can
/// follow. Neither implies the other, and the demo shows both.
pub fn reroot(child: TierId, parent: TierId) -> Option<Reroot> {
    let (child, parent) = (tier(child), tier(parent));
    if !(child.domain_m.is_finite() && parent.g0_m.is_finite()) {
        return None;
    }
    let ratio = child.domain_m / parent.g0_m;
    if !(ratio.is_finite() && ratio > 0.0) {
        return None;
    }
    let grains = parent.g0_m / child.g0_m;
    if (ratio - 1.0).abs() <= 1.0e-9 {
        return Some(Reroot::OneTerminalHolon {
            child_per_parent: grains * grains * grains,
        });
    }
    if ratio > 1.0 && (ratio - ratio.round()).abs() <= 1.0e-9 * ratio {
        return Some(Reroot::WholeMultiple {
            parents: ratio.round(),
        });
    }
    Some(Reroot::Contained { fraction: ratio })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A geometric census is arithmetic and is checked EXACTLY; an observed one is a
    /// measurement and is checked only against the cells available to hold it.
    ///
    /// This gate is why the sandbox count is 4.6656e8 and not the 6.2208e8 it shipped
    /// with: `fill` was doing double duty as a height fraction and a packing fraction,
    /// so the declared matter line and the declared count described two different boxes.
    /// Nothing else caught it — the ledger fit, the scene ran, and the number was wrong
    /// by a third.
    #[test]
    fn a_declared_census_matches_its_own_geometry() {
        for tier in tiers() {
            if !tier.domain_m.is_finite() {
                continue;
            }
            let cells = tier.geometric_cells();
            match tier.census {
                Census::Geometric { packing } => {
                    let expected = cells * tier.fill * packing;
                    let error = (tier.constituents as f64 - expected).abs() / expected;
                    assert!(
                        error < 1.0e-9,
                        "{}: declares {} constituents but its geometry gives {expected:.6e} \
                         ({:.3e} relative)",
                        tier.name,
                        tier.constituents,
                        error
                    );
                }
                Census::Observed => {
                    assert!(
                        (tier.constituents as f64) <= cells,
                        "{}: {} observed bodies will not fit in {cells:.4e} cells",
                        tier.name,
                        tier.constituents
                    );
                    let occupancy = tier.occupancy();
                    assert!(
                        occupancy > 0.0 && occupancy < 1.0,
                        "{}: an observed census in a mostly empty volume should have \
                         occupancy strictly between 0 and 1, got {occupancy:e}",
                        tier.name
                    );
                }
            }
        }
    }

    /// The tier-transition certificate. Where a zoom lands on exactly one terminal
    /// holon, the ledger identity is exact and is checked to the last constituent.
    #[test]
    fn a_re_root_carries_the_ledger_through_the_grain_ratio() {
        let mut exact = 0;
        for pair in TierId::ALL.windows(2) {
            let (child, parent) = (pair[0], pair[1]);
            let Some(relation) = reroot(child, parent) else {
                continue;
            };
            let child_tier = tier(child);
            if let Reroot::OneTerminalHolon { child_per_parent } = relation {
                exact += 1;
                // One parent terminal holon, counted at the child's grain, IS the
                // child's whole ledger.
                let expected = child_per_parent * child_tier.occupancy();
                let error = (child_tier.constituents as f64 - expected).abs() / expected;
                assert!(
                    error < 1.0e-9,
                    "{} is exactly one {} terminal holon, so its ledger must be \
                     {expected:.6e}, but it declares {} ({error:.3e} relative)",
                    child_tier.name,
                    tier(parent).name,
                    child_tier.constituents
                );
            }
        }
        assert!(
            exact >= 2,
            "the ladder should have at least two exact one-terminal-holon re-roots, \
             found {exact}"
        );
    }

    /// Mutation of the re-root gate: move a grain constant off the cell boundary and
    /// the relation must stop being exact. A gate that reports `OneTerminalHolon` for a
    /// zoom that does not land on one would certify a ledger break as a clean
    /// transition, which is the failure worth planting.
    #[test]
    fn a_re_root_that_misses_the_cell_boundary_is_not_reported_as_exact() {
        // The real adjacency: the grain tier IS one sandbox grain.
        assert!(matches!(
            reroot(TierId::Grain, TierId::Sandbox),
            Some(Reroot::OneTerminalHolon { .. })
        ));

        // MUTANT: a child domain 1% off the parent's grain. Nothing about the ledger
        // arithmetic changes; only the geometry misses, and that has to be enough.
        let mutant = |child_domain: f64, parent_g0: f64| {
            let ratio = child_domain / parent_g0;
            (ratio - 1.0).abs() <= 1.0e-9
        };
        assert!(mutant(5.0e-4, 5.0e-4), "the clean case must still pass");
        assert!(
            !mutant(5.05e-4, 5.0e-4),
            "a 1% miss must not be reported as landing on the cell boundary"
        );

        // And the ledger identity itself must be sensitive: a 1% grain error is a 3%
        // count error, far outside the gate's tolerance.
        let grain = tier(TierId::Grain);
        let sandbox = tier(TierId::Sandbox);
        let honest = (sandbox.g0_m / grain.g0_m).powi(3);
        let mutated = (sandbox.g0_m / (grain.g0_m * 1.01)).powi(3);
        assert!(
            (honest - mutated).abs() / honest > 0.02,
            "the ledger identity must be sensitive to a 1% grain error"
        );
    }

    #[test]
    fn every_declared_tier_fits_the_ledger() {
        for tier in tiers() {
            assert!(
                tier.ledger().fits(),
                "{} does not fit the REG+ ledger: {:?}",
                tier.name,
                tier.ledger()
            );
            assert!(
                tier.constituents <= u64::MAX,
                "{} constituent count is not representable",
                tier.name
            );
        }
    }

    #[test]
    fn declared_root_grain_covers_the_domain() {
        for tier in tiers() {
            if !tier.domain_m.is_finite() {
                continue;
            }
            let needed = ceil_pow2(tier.domain_m / tier.g0_m);
            assert_eq!(
                tier.root_grain_units, needed,
                "{}: root grain {} is not the power of two covering {} / {}",
                tier.name, tier.root_grain_units, tier.domain_m, tier.g0_m
            );
        }
    }

    /// The finding the whole ladder is built on, as a test rather than a claim.
    #[test]
    fn a_grain_of_sand_nearly_exhausts_the_ledger_in_atoms() {
        let atoms = atoms_in_grain(SAND_GRAIN_M);
        assert!(
            (atoms / 5.216e18 - 1.0).abs() < 0.01,
            "one 0.5 mm quartz grain should be ~5.216e18 atoms, got {atoms:e}"
        );
        let headroom = u64::MAX as f64 / atoms;
        assert!(
            (3.0..4.0).contains(&headroom),
            "the ledger should hold ~3.5 grains of sand in atoms, got {headroom:.2}"
        );
    }

    /// The REG+ maxima are read off the lattice, not written down beside it.
    ///
    /// A constant describing another module's enumeration is a constant that will
    /// eventually disagree with it. This one already did: `REG_PLUS_MAX` declared a
    /// per-component momentum of 3, which no subset of the six axial directions can
    /// reach.
    #[test]
    fn the_reg_plus_maxima_are_enumerated_not_asserted() {
        use ciris_sim_core::regplus::sector;

        let mut occupancy = 0_u64;
        let mut momentum = 0_u64;
        for state in 0..64_u8 {
            let label = sector(state);
            occupancy = occupancy.max(label.occupancy as u64);
            momentum = momentum
                .max(label.momentum[0].unsigned_abs() as u64)
                .max(label.momentum[1].unsigned_abs() as u64);
        }
        assert_eq!(
            LeafWrites::REG_PLUS_MAX.occupancy, occupancy,
            "declared REG+ occupancy maximum disagrees with the lattice"
        );
        assert_eq!(
            LeafWrites::REG_PLUS_MAX.momentum, momentum,
            "declared REG+ momentum maximum disagrees with the lattice"
        );
    }

    /// Which lane runs out first is a fact about the CHART, and both readings are
    /// pinned here so neither can drift.
    ///
    /// This demo's chart writes nothing but constituents, so it gets the loose
    /// 3.54-grain reading. A chart whose leaves carry full REG+ occupancy gets 0.59 —
    /// six times tighter — and quoting the loose number as if it were a property of the
    /// ledger would be wrong by that factor for every such chart. The 4090 study caught
    /// that; this is the test that keeps it caught.
    #[test]
    fn the_binding_lane_is_a_property_of_the_chart() {
        let atoms = atoms_in_grain(SAND_GRAIN_M);

        // A cube of atoms sized so the constituent lane is exactly full.
        let side = (u64::MAX as f64).cbrt();
        let geometric = ledger_for(side, 1.0, 1.0, LeafWrites::GEOMETRIC);
        assert!(
            matches!(
                geometric,
                Ledger::Fits {
                    binding: Lane::Constituents,
                    ..
                }
            ),
            "a chart writing no occupancy must be bound by constituents: {geometric:?}"
        );

        // The same scene, with a chart that writes REG+ occupancy, overflows — and
        // overflows in the OCCUPANCY lane, not the constituent one.
        match ledger_for(side, 1.0, 1.0, LeafWrites::REG_PLUS_MAX) {
            Ledger::Overflows { lane, factor } => {
                assert_eq!(lane, Lane::Occupancy);
                assert!(
                    (5.9..6.1).contains(&factor),
                    "REG+ occupancy should overflow by exactly the 6 it writes, got {factor}"
                );
            }
            other => panic!("a REG+ chart must overflow the occupancy lane, got {other:?}"),
        }

        // And the headline, both ways, in grains of sand.
        let loose = Lane::Constituents.capacity() as f64 / atoms;
        let tight = Lane::Occupancy.capacity() as f64 / (6.0 * atoms);
        assert!(
            (3.5..3.6).contains(&loose),
            "the geometric chart should hold ~3.54 grains, got {loose:.2}"
        );
        assert!(
            (0.55..0.62).contains(&tight),
            "a full REG+ chart should hold ~0.59 grains, got {tight:.2}"
        );
        // The momentum lane is an i64 carrying up to 2 per leaf, so it holds MORE than
        // occupancy does and binds strictly later — which is why the headline is the
        // occupancy figure and not this one.
        let momentum = Lane::Momentum.capacity() as f64
            / (LeafWrites::REG_PLUS_MAX.momentum as f64 * atoms);
        assert!(
            (0.87..0.90).contains(&momentum),
            "the momentum lane should hold ~0.885 grains, got {momentum:.3}"
        );
        assert!(
            momentum > tight,
            "momentum must bind later than occupancy, or the headline names the wrong lane"
        );
    }

    /// Mutation of the ledger gate: a tier that asks for a grain the ledger cannot
    /// carry must be REFUSED with the overflow factor, never silently truncated. The
    /// sandbox in atoms is the case that motivated the whole re-rooting design, so it
    /// is the one planted here.
    #[test]
    fn counting_a_tier_in_atoms_is_refused_with_its_factor() {
        let sandbox = tier(TierId::Sandbox);
        match sandbox.ledger_in_atoms() {
            Ledger::Overflows { lane, factor } => {
                assert_eq!(lane, Lane::Constituents);
                assert!(
                    (1.0e8..1.0e9).contains(&factor),
                    "the sandbox in atoms should be ~4e8x over the ledger, got {factor:e}"
                );
            }
            other => panic!("the sandbox in atoms must overflow the ledger, got {other:?}"),
        }

        // And the refusal is not universal, which is what makes it informative: the
        // grain tier's own domain IS countable in atoms, with the headroom above.
        assert!(
            tier(TierId::Grain).ledger_in_atoms().fits(),
            "one grain of sand must still be countable in atoms"
        );
    }

    /// The 110x jump in fracture energy between the crystal-derived value and the
    /// continuum one is MINTED at the continuum tier, not transported (misfit L23/M25).
    /// A zoom UI that interpolated between them would be inventing a number, so the gap
    /// is pinned here.
    #[test]
    fn fracture_energy_is_minted_not_transported() {
        let grain = tier(TierId::Grain).material.unwrap();
        let landscape = tier(TierId::Landscape).material.unwrap();
        let ratio = landscape.fracture_energy_j_m2 / grain.fracture_energy_j_m2;
        assert!(
            (100.0..120.0).contains(&ratio),
            "the transported-to-minted fracture-energy gap should be ~110x, got {ratio:.0}x"
        );
    }

    /// The single arithmetic fact the demo is built around: the continuum chart's
    /// process zone is COARSER than a grain of sand, so sand cannot be cracked by that
    /// chart and must be contacted instead.
    #[test]
    fn the_continuum_chart_cannot_resolve_a_grain_of_sand() {
        let required = tier(TierId::Landscape).required_spacing_m().unwrap();
        assert!(
            required > SAND_GRAIN_M * 20.0,
            "the landscape chart's finest legal cell ({required:.4} m) should be far \
             coarser than a {SAND_GRAIN_M} m grain"
        );
        // And the grain tier's own demand is far finer than that tier's floor, which is
        // the reason its verdict is a refusal rather than a certificate.
        let grain_required = tier(TierId::Grain).required_spacing_m().unwrap();
        assert!(
            grain_required < tier(TierId::Grain).g0_m,
            "the grain chart should demand ({grain_required:e} m) finer than its own \
             grain floor ({:e} m)",
            tier(TierId::Grain).g0_m
        );
    }

    /// The granular tier asks a question it can answer and the cohesive tiers ask one
    /// they may not be able to. Both are the same surrogate; only the length differs,
    /// and the length follows from the claim.
    /// The observer's claim is servable at every tier that renders, and its cost is
    /// recorded here so a change to `ACUITY_PIXELS` shows up as a diff in the numbers
    /// rather than as a scene that quietly got slower.
    #[test]
    fn the_observer_claim_is_servable_and_its_cost_is_pinned() {
        for tier in tiers() {
            if !tier.domain_m.is_finite() {
                continue;
            }
            assert!(
                tier.acuity_servable(),
                "{}: acuity {:e} is finer than its own grain {:e}; that is a refusal, \
                 and the demo must state it rather than render invented detail",
                tier.name,
                tier.acuity_m(),
                tier.g0_m
            );
            // Acuity is never finer than the terminal holon.
            assert!(tier.acuity_m() >= tier.g0_m);
        }

        // The two tiers that matter, pinned. If these move, the frame budget moves.
        let sandbox = tier(TierId::Sandbox);
        assert!(
            (sandbox.acuity_m() / 2.0e-3 - 1.0).abs() < 1.0e-9,
            "sandbox acuity should be 2 mm, got {:e}",
            sandbox.acuity_m()
        );
        assert!(
            (sandbox.acuity_m() / sandbox.g0_m - 4.0).abs() < 1.0e-9,
            "the sandbox observer resolves 4 grains across, got {:.2}",
            sandbox.acuity_m() / sandbox.g0_m
        );
    }

    #[test]
    fn the_demand_follows_the_claim() {
        let sandbox = tier(TierId::Sandbox);
        assert_eq!(sandbox.required_spacing_m(), Some(sandbox.g0_m));
        // Had the sandbox been asked the cohesive question instead, its demand would be
        // three orders finer than its own terminal holon — which is exactly why asking
        // it would be the wrong question, not a stricter one.
        let cohesive_demand = sandbox.characteristic_length_m().unwrap() / 10.0;
        assert!(
            sandbox.g0_m / cohesive_demand > 1000.0,
            "the cohesive demand should be ~1184x finer than a grain, got {:.0}x",
            sandbox.g0_m / cohesive_demand
        );

        let landscape = tier(TierId::Landscape);
        assert_eq!(
            landscape.required_spacing_m(),
            Some(landscape.characteristic_length_m().unwrap() / 10.0)
        );
    }

    /// Named for the count it asserts. It used to be called `three_tiers_refuse...`
    /// while asserting four, which the 4090 study spotted — a test whose name disagrees
    /// with its assertion is a test nobody can read, and the count is exactly the kind
    /// of thing a reader would take from the name rather than the body.
    #[test]
    fn one_tier_refuses_wholesale_and_names_what_would_lift_it() {
        let refusing: Vec<_> = tiers()
            .into_iter()
            .filter(|tier| matches!(tier.evaluator, Evaluator::Unavailable(_)))
            .map(|tier| (tier.id, tier.evaluator))
            .collect();
        assert_eq!(
            refusing.len(),
            1,
            "the curvature bridge lifted three of the four wholesale refusals; only the \
             crystal tier still has no evaluator at all: {refusing:?}"
        );
        assert_eq!(refusing[0].0, TierId::Crystal);

        // The three that were lifted evaluate now — but per SCENE, not wholesale. A
        // gravity tier can still refuse, and one of its declared scenes does.
        for id in [TierId::Planet, TierId::Galactic, TierId::Cosmic] {
            assert!(
                matches!(tier(id).evaluator, Evaluator::GeodesicChart { .. }),
                "{id:?} should evaluate on a declared weak-field chart"
            );
        }
        for (id, evaluator) in &refusing {
            let Evaluator::Unavailable(refusal) = evaluator else {
                unreachable!()
            };
            assert!(
                !refusal.unlock().is_empty(),
                "{id:?} refuses without naming what would lift it"
            );
        }
        assert_eq!(
            refusing[0].1,
            Evaluator::Unavailable(Refusal::NoValidatedEvaluator)
        );
        for (_, evaluator) in &refusing[1..] {
            assert_eq!(*evaluator, Evaluator::Unavailable(Refusal::NoGravityChart));
        }
    }
}
