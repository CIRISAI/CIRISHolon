//! One tier, live: certify a frontier, throw something at it, step the result.
//!
//! Everything domain-specific here is a VALUE. The solver does not know which tier it is
//! running: it steps nodes under uniform chart gravity, asks each live relation holon for
//! its axial force, and resolves unilateral contact on the pairs no live relation owns.
//! Sand and stone differ in whether the homogenizer would give their relations a law at
//! their own grain, and that question is asked in `scene::relation_law`, not here.

use ciris_sim_core::bridge::{WeakFieldCertificate, WeakFieldRefusal};
use ciris_sim_core::curvature::{curved_from_celerity, integrate_geodesic};
use ciris_sim_core::holon::CertificationStatus;
use ciris_sim_core::relativity::Worldline;
use ciris_sim_core::homogenization::derive_lattice_elastic_law;
use ciris_sim_core::runtime::RuntimeArena;

use crate::chart::Chart;
use crate::gauge::{GaugeScene, Move};
use crate::incremental::{certify_incremental, Budget, IncrementalError, Settled, Workspace};
use crate::scene::{
    build_nodes, build_relations, mass_per_constituent_kg, relation_law, root_scene, LawRefusal,
    Nodes, QuadrantMaterializer, Relations, ResolutionModel, CHART_GRAVITY_M_S2,
    CONSERVATION_TOLERANCE, OBSERVABLES,
};
use crate::tier::{tier, Evaluator, Tier, TierId};

/// How far the resolution demand relaxes with distance from the throw. 1.0 means a cell
/// may be as coarse as its own distance from the interaction.
///
/// This is the demo's default, not a physical constant, and the viewer exposes it: it
/// trades how much of the scene is resolved against how much simulated time a frame
/// buys, and both sides of that trade are shown.
pub const GRADING: f64 = 2.0;

/// Bonds are formed between cells within this multiple of touching.
pub const RELATION_REACH: f64 = 1.35;

/// Wall-clock shape of one throw EVENT, for the record rather than as an enforced
/// limit: the engine cannot time itself on `wasm32-unknown-unknown`.
///
/// The original figure was 120 ms, written when the only claim was the physics and a
/// throw materialized a few hundred holons. The observer's claim multiplies the resident
/// set by about a thousand, and a throw now measures 180-380 ms through wasm. Raising
/// the declared budget rather than quietly exceeding it: a throw is a one-off event on a
/// click, and 400 ms is still a responsive one. Frames are unaffected and keep their own
/// budget.
pub const EVENT_BUDGET_MS: f64 = 400.0;

/// Headroom over the observer's claim that one throw's physics may spend.
///
/// The ceiling is no longer a flat constant. Pinning the scene at acuity means the
/// resident set is set by the OBSERVER, and a fixed budget below that would refuse every
/// scene for being visible. So the budget is derived: whatever acuity costs at this
/// tier, times this factor for the physics corridor on top. Hitting it is still a
/// reported refusal and never a silent coarsening.
pub const PHYSICS_HEADROOM: f64 = 3.0;

/// Greedy-round ceiling for one throw, likewise declared.
pub const MAX_ROUNDS: usize = 400_000;

/// Default substep work budget per frame, in node-substeps.
///
/// This is a STARTING POINT, not a calibration. It was set from native measurements and
/// then measured again through `wasm32-unknown-unknown`, where the same scene ran 3.6x
/// slower — 30 ms per frame against the 8 ms it cost natively. A constant tuned on one
/// of those two numbers is wrong on the other, and wrong again on a slower laptop.
///
/// So the host adjusts it: it is the side of the boundary that has a clock. See
/// [`Session::set_work_budget`].
pub const SUBSTEP_WORK_BUDGET: usize = 24_000;

/// Impact speed at a tier, m/s.
///
/// Scaled by the square root of the domain so a larger tier is not instantaneous, and
/// declared as a stage value: it changes what you see and nothing that is claimed.
pub fn impact_speed(tier: &Tier, speed_fraction: f64) -> f64 {
    4.0 * tier.domain_m.sqrt() * speed_fraction.clamp(0.05, 1.0)
}

/// Deliberate defects in the impulse accounting, plantable without forking the code
/// path — the `ResidualMode` pattern of `ciris_sim_core::fracture`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImpulseMode {
    /// The net contact impulse: the vector sum, integrated. Conserved.
    #[default]
    Net,
    /// MUTANT, and the bug this actually was: add the SIZES of every simultaneous
    /// contact force instead of their vector sum. A projectile resting against several
    /// cells is pushed by each in directions that largely cancel, so this counts force
    /// that is not there — and it does so by a factor that scales with how many cells
    /// are touched at once, which is why nothing about it looked like a constant.
    SumOfMagnitudes,
}

/// The strain-rate band no simulation crosses, per second.
///
/// Below it, laboratory quasi-static tests; above it, molecular dynamics. Between them
/// is an interpolation, and this demo's throws land in the middle of it
/// (DESCRIPTOR_CHAIN §3.3, misfit L21).
pub const STRAIN_RATE_GAP: (f64, f64) = (1.0e0, 1.0e7);

/// Preview grain, as a fraction of the domain: how coarse the scene is rendered before
/// anything has been thrown at it. A declared RENDERING depth carrying no certificate —
/// see [`Session::settle`].
pub const PREVIEW_FRACTION: f64 = 1.0 / 32.0;

/// Holon ceiling for the preview pass, declared so a resting scene cannot be expensive.
pub const PREVIEW_HOLONS: usize = 2_000;

/// Absolute ceiling on substeps in one frame, whatever the work budget divides out to.
///
/// A budget expressed per node-substep has no answer when almost nothing is awake, and
/// the fixed overhead of a substep — taking the working sets, refreshing the
/// projectile's neighbourhood — is real even when the substep does no physics.
pub const MAX_SUBSTEPS_PER_FRAME: usize = 4_000;

/// The body's position in the scene's own view plane, relative to the scene's centre.
///
/// Ballistic scenes are read in (x, z) about their launch point; orbital ones in (x, y)
/// about the central mass. Both are declared per scene rather than inferred, because a
/// trail in raw chart coordinates is unusable: the planet ball's z is 6.37 MILLION
/// metres, on a view two hundred metres wide.
fn scene_point(scene: &crate::gravity::GravityScene, line: &Worldline) -> [f64; 2] {
    let raw = [
        line.x[1 + scene.plane[0]],
        line.x[1 + scene.plane[1]],
    ];
    [raw[0] - scene.view_center[0], raw[1] - scene.view_center[1]]
}

/// Geodesic RK4 steps per rendered frame, at the scene's own pinned proper-time step.
pub const GEODESIC_STEPS_PER_FRAME: u32 = 8;

/// Longest trail kept for drawing, in points.
pub const GEODESIC_TRAIL_MAX: usize = 900;

/// Substeps a frame is assumed to advance when placing the projectile's release point.
pub const RELEASE_SUBSTEPS: usize = 256;

/// Approach speed, as a fraction of the scene's impact speed, at which a sleeping cell
/// wakes.
///
/// Waking on FORCE cannot work, and the reason is worth stating: a cell resting on
/// another already pushes it with its own weight plus the column above, so any force
/// threshold low enough to catch an impact is also crossed by ordinary support. The
/// first attempt used a twentieth of a cell's weight and woke 107,000 of 118,296 cells
/// for one thrown marble — not a tuning error, a category error.
///
/// Relative SPEED separates the two cleanly: a supporting contact has none, an impact
/// has plenty. It is also the same quantity the sleep test uses, so a cell cannot wake
/// on a criterion it can never satisfy well enough to sleep again.
pub const WAKE_SPEED_FRACTION: f64 = 0.01;

/// Speed below which a cell is a candidate for sleeping, as a fraction of the scene's
/// own impact speed. Scale-free, so it means the same thing at every tier.
pub const SLEEP_SPEED_FRACTION: f64 = 1.0e-4;

/// What sleeping costs the reported impulse, declared.
///
/// A sleeping cell is treated as fixed, so it absorbs a little of the momentum a moving
/// cell would otherwise have carried. Measured against a control with sleeping disabled,
/// on the sandbox tier: **0.94%**. The impulse this demo reports is therefore good to
/// about a percent, and the viewer says so where the number is shown rather than
/// implying the four figures it prints.
///
/// The bound below is 2%, which is above the measurement and not fitted to it. If a
/// change pushes the real cost past this, the right response is to tighten the sleep
/// criterion or to publish a wider figure — not to widen the bound.
pub const SLEEP_IMPULSE_TOLERANCE: f64 = 0.02;

/// Consecutive slow substeps before a cell sleeps. A margin against sleeping something
/// that is merely at the top of its arc.
pub const SLEEP_SUBSTEPS: u16 = 24;

/// Coulomb friction between never-bonded grains.
///
/// 0.65 is `tan(33 deg)`, the angle of repose class value for dry medium sand. It is a
/// CLASS warrant with no specimen behind it and is labelled as such wherever the demo
/// shows it — the same standard `IsotropicMaterial::DEMO_CALIBRATION` is held to.
pub const GRAIN_FRICTION_MU: f64 = 0.65;

/// Largest contact overlap the soft-particle stiffness is allowed to produce, as a
/// fraction of a grain radius, at the scene's own impact speed.
///
/// 10% is the loose end of ordinary discrete-element practice — up to about there the
/// bulk response of a granular assembly is insensitive to the choice, which is the
/// property the method rests on. DECLARED here so the number that follows from it has a
/// criterion rather than a preference behind it, and chosen at the loose end because
/// the stiffness sets the timestep and the timestep sets how much physics a frame buys:
/// at 5% the sandbox advanced 0.9% of real time per frame, and a demo nobody can watch
/// is not more honest for being stiffer.
pub const MAX_OVERLAP_FRACTION: f64 = 0.10;

/// Restitution of a grain-on-grain contact. A pair/velocity/geometry OUTCOME taken per
/// contact, which is why `IsotropicMaterial` deliberately has no such field (A5).
pub const CONTACT_RESTITUTION: f64 = 0.35;

/// What the user is told after a throw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Nothing has been thrown yet at this tier.
    Idle,
    /// The frontier met the tier's own resolution demand and the readout stands.
    Certified,
    /// The demand could not be met at this tier's grain floor. No claim is made.
    GrainFloor,
    /// Refinement ran out before the demand was met — children are not resident and
    /// the generator declined, or the declared budget was reached.
    RefinementUnavailable,
    /// This tier has no validated evaluator in this repository.
    NoEvaluator,
    /// This tier's scene has weight and this engine has no certified gravity chart.
    NoGravityChart,
    /// The declared event budget was exhausted before any verdict was reached. An
    /// error, deliberately not a verdict.
    BudgetExhausted,
    /// The spin-1 truncation: flux is `-1, 0, +1` and the loop is already at `+1`. The
    /// only refusal on the ladder that is not about resolution — the state space ends
    /// here, exactly, rather than the engine being unable to see finely enough.
    FluxCeiling,
    /// The scene fell outside its tier's weak-field screen. Which way it fell is in
    /// [`Session::weak_field`]'s refusal, and every one of them names its unlock.
    WeakFieldRefused,
}

impl Verdict {
    pub const fn code(self) -> u32 {
        match self {
            Verdict::Idle => 0,
            Verdict::Certified => 1,
            Verdict::GrainFloor => 2,
            Verdict::RefinementUnavailable => 3,
            Verdict::NoEvaluator => 4,
            Verdict::NoGravityChart => 5,
            Verdict::BudgetExhausted => 6,
            Verdict::FluxCeiling => 7,
            Verdict::WeakFieldRefused => 8,
        }
    }
}

/// Uniform-grid broadphase over the resident nodes.
///
/// `sparse::resolve_sphere_contacts` in the core is an all-pairs sweep over
/// compile-time-sized arrays, which is right for the scenes it was written for and wrong
/// for a runtime frontier of thousands of cells. This is the runtime-sized companion; it
/// finds exactly the same pairs, and the equality is asserted in `tests::broadphase_
/// finds_every_overlapping_pair` rather than assumed.
#[derive(Debug, Default)]
pub struct Broadphase {
    cell_m: f64,
    columns: usize,
    rows: usize,
    /// Node indices bucketed by cell, as a CSR-style pair of arrays so there is one
    /// allocation rather than one per bucket.
    starts: Vec<u32>,
    items: Vec<u32>,
    counts: Vec<u32>,
    /// Nodes too large for the grid's cell, checked against everything.
    oversized: Vec<u32>,
    small: Vec<u32>,
    node_count: usize,
}

impl Broadphase {
    pub fn rebuild(&mut self, nodes: &Nodes, domain_m: f64, margin_m: f64) {
        // Sizing the grid by the LARGEST node is what a uniform grid normally does, and
        // on a graded quadtree it collapses: one coarse far-field cell can be the whole
        // domain wide, the grid becomes a single bucket, and the "broadphase" is an
        // all-pairs sweep wearing a grid's clothes. That is what it was doing, measured
        // at 30 ms per frame on a 200-node landscape frontier.
        //
        // So the grid is sized by the MEDIAN node instead, and the few nodes too big for
        // it are held aside and checked against everything. On a graded quadtree the
        // sizes are geometric, so that oversized set stays small.
        let mut radii: Vec<f64> = nodes.radius_m.clone();
        radii.sort_by(f64::total_cmp);
        let median = radii.get(radii.len() / 2).copied().unwrap_or(0.0);
        self.cell_m = (2.0 * median + margin_m).max(domain_m / 512.0);
        self.columns = ((domain_m / self.cell_m).ceil() as usize).clamp(1, 512);
        self.rows = self.columns;
        let buckets = self.columns * self.rows;

        self.oversized.clear();
        self.small.clear();
        for i in 0..nodes.len() {
            if 2.0 * nodes.radius_m[i] > self.cell_m {
                self.oversized.push(i as u32);
            } else {
                self.small.push(i as u32);
            }
        }

        self.counts.clear();
        self.counts.resize(buckets + 1, 0);
        for i in self.small.iter().copied() {
            let bucket = self.bucket(nodes.position[i as usize], domain_m);
            self.counts[bucket] += 1;
        }
        self.starts.clear();
        self.starts.resize(buckets + 1, 0);
        let mut running = 0_u32;
        for bucket in 0..=buckets {
            self.starts[bucket] = running;
            running += self.counts[bucket];
        }
        let mut cursor = self.starts.clone();
        self.items.clear();
        self.items.resize(self.small.len(), 0);
        for i in self.small.iter().copied() {
            let bucket = self.bucket(nodes.position[i as usize], domain_m);
            self.items[cursor[bucket] as usize] = i;
            cursor[bucket] += 1;
        }
        self.node_count = nodes.len();
    }

    fn bucket(&self, position: [f64; 2], domain_m: f64) -> usize {
        let column = ((position[0] / domain_m * self.columns as f64) as isize)
            .clamp(0, self.columns as isize - 1) as usize;
        let row = ((position[1] / domain_m * self.rows as f64) as isize)
            .clamp(0, self.rows as isize - 1) as usize;
        row * self.columns + column
    }

    /// Resident cells whose bucket neighbourhood could contain something within
    /// `radius` of `point`, written into a caller-owned buffer.
    ///
    /// Row and column bounds are computed from the POINT, not from a linear bucket index
    /// modulo the width. Deriving them from the index gives an empty column range
    /// whenever the corners fall in different rows — which is most of the time, and
    /// which silently returned nothing: the projectile passed through the sand touching
    /// nothing at all, and the scene stayed asleep while a throw was in flight.
    pub fn near_into(
        &self,
        point: [f64; 2],
        radius: f64,
        domain_m: f64,
        out: &mut Vec<usize>,
    ) {
        out.clear();
        if self.columns == 0 || self.starts.is_empty() || domain_m <= 0.0 {
            return;
        }
        let reach = radius + self.cell_m;
        let axis = |value: f64, count: usize| -> usize {
            ((value / domain_m * count as f64) as isize).clamp(0, count as isize - 1)
                as usize
        };
        let lo_c = axis(point[0] - reach, self.columns);
        let hi_c = axis(point[0] + reach, self.columns);
        let lo_r = axis(point[1] - reach, self.rows);
        let hi_r = axis(point[1] + reach, self.rows);
        for row in lo_r..=hi_r {
            for column in lo_c..=hi_c {
                let bucket = row * self.columns + column;
                if bucket + 1 >= self.starts.len() {
                    continue;
                }
                out.extend(
                    self.items[self.starts[bucket] as usize
                        ..self.starts[bucket + 1] as usize]
                        .iter()
                        .map(|index| *index as usize),
                );
            }
        }
        // The oversized set is never in a bucket, so it is always a candidate.
        out.extend(self.oversized.iter().map(|index| *index as usize));
        out.sort_unstable();
        out.dedup();
    }

    /// Candidate pairs, each once, in ascending `(i, j)` order so replay is
    /// bit-identical whatever the grid does.
    pub fn pairs(&self, out: &mut Vec<(usize, usize)>) {
        out.clear();
        for row in 0..self.rows {
            for column in 0..self.columns {
                let bucket = row * self.columns + column;
                let here = &self.items
                    [self.starts[bucket] as usize..self.starts[bucket + 1] as usize];
                for (offset, i) in here.iter().enumerate() {
                    for j in &here[offset + 1..] {
                        out.push((*i.min(j) as usize, *i.max(j) as usize));
                    }
                    // Half the neighbourhood, so each pair of buckets is visited once.
                    for (dr, dc) in [(0_isize, 1_isize), (1, -1), (1, 0), (1, 1)] {
                        let nr = row as isize + dr;
                        let nc = column as isize + dc;
                        if nr < 0 || nc < 0 || nr >= self.rows as isize || nc >= self.columns as isize
                        {
                            continue;
                        }
                        let neighbour = nr as usize * self.columns + nc as usize;
                        let there = &self.items[self.starts[neighbour] as usize
                            ..self.starts[neighbour + 1] as usize];
                        for j in there {
                            out.push((*i.min(j) as usize, *i.max(j) as usize));
                        }
                    }
                }
            }
        }
        // Everything against the few nodes the grid cannot hold.
        for big in self.oversized.iter().copied() {
            for other in 0..self.node_count as u32 {
                if other == big {
                    continue;
                }
                out.push((big.min(other) as usize, big.max(other) as usize));
            }
        }
        out.sort_unstable();
        out.dedup();
    }
}

/// The projectile. One holon's worth of mass moving through the tier's own metres.
#[derive(Clone, Copy, Debug, Default)]
pub struct Projectile {
    pub position: [f64; 2],
    pub velocity: [f64; 2],
    pub radius_m: f64,
    pub mass_kg: f64,
    pub live: bool,
}

/// One tier's live session.
/// **Where the energy went, in joules, per named channel.**
///
/// The gate this supersedes was `assert!(worst < 1.5 * launch)` — a comparison of the
/// projectile's peak SPEED to its launch speed. That is not an energy check at all: it
/// cannot see energy appearing in the sand and it cannot see energy vanishing, and its
/// 50% band meant a declared dissipation channel and a bug read identically.
///
/// Superseded as the ENERGY check, not deleted: `tests::a_throw_never_gains_energy` is still
/// in this file and still runs, because a speed bound and a finiteness sweep over every node
/// catch a blow-up that a closed balance would not — the failure mode its own header names
/// is a certificate reading fine next to a ball leaving the box at fifty times launch speed.
/// Two instruments, different questions.
///
/// # The channels, and why there are six rather than the five first named
///
/// * `contact_damping_j` — **this IS grain-grain restitution.** `Session::retune` imposes
///   restitution *through* the contact damping that produces it for the pair's reduced
///   mass, "which keeps restitution a per-contact OUTCOME rather than a material field
///   (A5)". There is no separate restitution impulse between grains to instrument, and
///   counting one would double-count this channel. **Charged at TWO sites**: grain-grain
///   pairs and projectile-grain contacts. Only the first was instrumented at first, and the
///   second is where a throw does most of its work — at the landscape tier that single
///   omission was 4.82e7 J, 99% of a residual first reported as unexplained dissipation.
/// * `contact_friction_j` — tangential Coulomb-capped friction at grain contacts.
/// * `bond_friction_j` — the same at live cohesive relations. Zero at the sandbox tier,
///   where the homogenizer refuses a cohesive law outright.
/// * `wall_restitution_j` — **not in the original list, and a real sink.** The box has a
///   bottom and two sides, and each reflects with `CONTACT_RESTITUTION`; the code already
///   says "they take momentum out of the scene and the readout does not claim otherwise".
///   Here it claims otherwise. Charged for cells and, since this pass, for the projectile's
///   own floor bounce — which reads **exactly zero at all three tiers** on the gate's
///   throw, so it is completeness rather than a term that moves a number here.
/// * `sleep_absorbed_j` — a cell that sleeps has its velocity zeroed; that kinetic energy
///   leaves the scene.
/// * `anchor_absorbed_j` — an anchored cell has its velocity zeroed every substep, so any
///   work done on it that substep is absorbed by the anchor.
///
/// # The residual is NAMED, and that is the point
///
/// The integrator is semi-implicit Euler, which is not energy-conserving: it carries its
/// own `O(dt)` error per step. So `E(0) − E(t) − Σ channels` is **not zero even when every
/// channel is instrumented perfectly**, and a two-sided gate demanding zero would read
/// integrator error as a missing channel — a worse instrument than the one-sided one it
/// replaces, because it would fail for something that is not a defect.
///
/// [`Session::energy_residual_j`] is that term, reported rather than hidden, and the number
/// to quote is its size relative to the total dissipated.
///
/// # The balance is CHART-RELATIVE, and the gate refuses where the law is absent
///
/// Energy conservation is not a fact about the world; it is Noether's theorem applied to
/// time-translation symmetry, and this engine ships a chart that lacks it — the cosmic
/// tier's expansion background, where total vacuum energy grows with volume, predicted and
/// observed. So the balance below is a law at the flat Newtonian tiers and **refuses,
/// rather than passing or failing, everywhere else**. See [`BalanceApplicability`], which
/// also keeps static curvature separate from expansion: a static metric has a timelike
/// Killing vector, so a conserved energy exists there and only this instrument is missing.
///
/// # THE VERDICT ON THE FIRST RUN: the engine does not create energy. The ledger did.
///
/// This ledger's first run reported that the sandbox scene **gains 1.14 J of 107 (+1.06%)**
/// over four seconds, and that the landscape tier's named channels explained only **24%** of
/// what it dissipates. It named two open candidates — semi-implicit Euler injecting on stiff
/// contacts, and the `.max(0.0)` clamp on the contact force. **Both candidates are wrong, and
/// neither number was a defect in the engine.** Both were gaps in this instrument, and they
/// are two different ones:
///
/// * The **gain** was the overlap-potential term measuring from `radius[i] + radius[j]` where
///   the force law measures from [`Session::rest_gap`]. The term is summed over the pairs
///   adjacent to AWAKE cells, so the fiction switched on as the throw woke the scene and
///   read as energy appearing. Decomposed: of the +1.1389 J gain, **+1.1534 J is that term**
///   and every other component is negative. See [`Session::total_energy_j`].
/// * The **76% unexplained** was `contact_damping_j` charged at the grain-grain contacts and
///   not at the projectile's, which at the landscape tier is where the energy actually goes.
///
/// With both corrected, every tier DISSIPATES and the two-sided balance closes:
///
/// | tier | dissipated | named channels | explained | residual |
/// |---|---|---|---|---|
/// | Sandbox | +1.3368e-2 J | 1.2514e-2 J | **93.6%** | +6.4% |
/// | Grain | +9.2630e-15 J | 8.8380e-15 J | **95.4%** | +4.6% |
/// | Landscape | +6.3847e7 J | 6.3336e7 J | **99.2%** | +0.8% |
///
/// The residual is now positive at every tier — a LOSS, which is the only sign an
/// uninstrumented sink or an explicit integrator can have here. What is left in it, named so
/// the next reader starts past this point: the projectile's contact potential is not in
/// `E(t)` at all (see [`Session::total_energy_j`]); a projectile contact against an ASLEEP
/// cell applies no force to that cell, so the momentum is dropped rather than delivered
/// (measured at 1.3e-4 J of the sandbox's 1.34e-2 and 8.3e5 J of the landscape's 6.4e7,
/// **overlapping the damping channel and so not separably sized yet**); and semi-implicit
/// Euler's own `O(dt)` error, which was always going to be in here.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DissipationLedger {
    pub contact_damping_j: f64,
    pub contact_friction_j: f64,
    pub bond_friction_j: f64,
    pub wall_restitution_j: f64,
    pub sleep_absorbed_j: f64,
    pub anchor_absorbed_j: f64,
}

impl DissipationLedger {
    /// Every named channel summed.
    pub fn total_j(&self) -> f64 {
        self.contact_damping_j
            + self.contact_friction_j
            + self.bond_friction_j
            + self.wall_restitution_j
            + self.sleep_absorbed_j
            + self.anchor_absorbed_j
    }
}

/// **Whether a two-sided energy balance is a LAW in this scene's chart — the refusal
/// discipline applied to a conservation law.**
///
/// Energy conservation is not a fact about the universe, it is a fact about charts with
/// **time-translation symmetry**: Noether's theorem turns that symmetry into the conserved
/// quantity, and where the symmetry is absent so is the conservation. A balance gate that
/// fires wherever the books do not close would therefore be **enforcing a law the chart does
/// not have**, and would report correct physics as a defect.
///
/// This engine has a chart where it genuinely fails. The cosmic tier carries an expansion
/// background, and in an expanding universe **total vacuum energy grows with the volume** —
/// predicted, and observed. A gate flagging that as non-conservation would be wrong about
/// the physics, not merely conservative.
///
/// So the gate **refuses rather than passing or failing**, exactly as this crate's other
/// screens do. The three refusals are separable and they are not the same case:
///
/// * **Expansion background** — no global timelike Killing vector, so there is no globally
///   conserved energy to balance against. The law is absent.
/// * **Static curvature** — a static metric HAS a timelike Killing vector, so a conserved
///   energy does exist. The law is present; this ledger does not compute its quantity.
///   `total_energy_j` sums the Newtonian `½mv² + mgy`, and the conserved quantity on a
///   static chart is the Killing energy `−g_{μν} ξ^μ p^ν`. Refusing here is a statement
///   about the INSTRUMENT, and conflating it with the expansion case would hide the fact
///   that one of them is fixable and the other is not.
/// * **No mechanical evaluator** — nothing here steps a mechanical scene to balance.
///
/// The discriminator between the first two is not asserted from the tier name: it is
/// [`crate::gravity::GravityScene::certify`], the weak-field screen this crate already
/// ships, whose `WeakFieldRefusal::ExpansionScale` is precisely the finding that the
/// background term `(H L/c)²` has grown until a static chart is no longer licensed. That
/// screen already answers the Killing-vector question in the engine's own arithmetic, and
/// answering it twice, in two places, is how the two answers drift apart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BalanceApplicability {
    /// Flat chart, Newtonian dynamics, static uniform gravity, fixed volume, fixed particle
    /// number. Time-translation symmetry holds, so energy conservation is a theorem of the
    /// continuum equations and **a secular gain is a defect** — integrator, unaccounted
    /// declared channel, or bug.
    Holds,
    /// The expansion background dominates: no timelike Killing vector, no conserved energy.
    /// Non-conservation here is the physics, not a defect.
    RefusedNoTimeTranslationSymmetry,
    /// Static curved chart. A conserved energy EXISTS (timelike Killing vector) and this
    /// ledger computes the Newtonian expression instead of the Killing energy.
    RefusedConservedQuantityNotComputed,
    /// Nothing here steps a mechanical scene.
    RefusedNoMechanicalEvaluator,
}

impl BalanceApplicability {
    /// Is the balance a law here, so that a gain is a defect?
    pub fn holds(self) -> bool {
        matches!(self, Self::Holds)
    }

    /// What this means for a general reader.
    pub fn plain(self) -> &'static str {
        match self {
            Self::Holds => {
                "Energy cannot appear or vanish in this scene, so if the books do not \
                 balance something is wrong with the engine."
            }
            Self::RefusedNoTimeTranslationSymmetry => {
                "Space is expanding here, and in an expanding universe the total energy is \
                 not fixed — it grows with the volume. There is no balance to check, so \
                 this refuses to check one."
            }
            Self::RefusedConservedQuantityNotComputed => {
                "Gravity curves this scene but does not change with time, so a conserved \
                 energy does exist — it is just not the everyday one this ledger adds up. \
                 The law holds; the instrument is the wrong instrument."
            }
            Self::RefusedNoMechanicalEvaluator => {
                "Nothing in this scene moves the way a ball or a grain of sand moves, so \
                 there is no mechanical energy to balance."
            }
        }
    }
}

/// **The sigma-ledger: distinguishability destroyed, in nats.**
///
/// The D-ledger counts joules. Joules cannot see this: a step can conserve energy exactly
/// and still destroy information, and the places it does are where the engine *collapses*
/// state rather than where it dissipates it.
///
/// # The three merge sites, and why the fourth candidate is not one
///
/// `Core/Habit.lean`'s `production_eq_zero_iff_rate_injective` makes production the failure
/// of injectivity, so a merge site is a place where **many inputs give one output**:
///
/// * `sleep_nats` — a cell that sleeps has its velocity set to `[0, 0]`. Every slow
///   velocity maps to the same zero: many-to-one.
/// * `anchor_nats` — an anchored cell is zeroed every substep. Same collapse.
/// * `wall_clamp_nats` — a cell past the floor or a side wall has its **position** set to
///   exactly the wall. An interval of positions maps to one point.
///
/// **The wall's velocity flip is NOT a merge site**, and this is worth stating because it
/// sits two lines from one that is. `v ↦ −v·e` is injective for `e ≠ 0` — it relabels a
/// distinction rather than destroying it — so it dissipates energy (the D-ledger's
/// `wall_restitution_j`) while destroying no information. The two ledgers disagree about
/// that line, and they are both right: **energy loss and information loss are different
/// events**, which is the whole reason for keeping two currencies.
///
/// # Units: nats, and how a collapse is priced
///
/// Nats, because `Core/StochasticHabit.lean` proves `frameEntropy` and Shannon entropy are
/// the same quantity on the uniform-on-fiber state using `Real.log`. Collapsing a discarded
/// value `x` to a target destroys the distinctions `x` could have carried: `|x|/ulp(x)`
/// representable values, so `ln(|x|/ulp(x))` nats. For any normal `f64` that is
/// **≈ 36.0 nats (52 bits)** — the mantissa — so the event count and the nats are near
/// proportional, and both are reported.
///
/// # Sign
///
/// Not clamped at zero. `StochasticHabit`'s reset channel lowers entropy by exactly
/// `log 2`, so **negative production is expected physics for a stochastic step**, not an
/// accounting bug. Nothing here silently takes an absolute value.
///
/// # THE VERDICT: at these three sites the ledger is an EVENT COUNTER in expensive units
///
/// Measured over 240 frames after the same throw, at three tiers spanning 26 orders of
/// magnitude in scene energy:
///
/// | tier | sleep | wall clamp | anchor | total |
/// |---|---|---|---|---|
/// | Sandbox | 202,052 nats / 2,777 ev | 208,871 nats / 5,757 ev | 0 / 0 | 410,923 nats |
/// | Grain | 311,365 nats / 4,278 ev | 0 / 0 | 0 / 0 | 311,365 nats |
/// | Landscape | 3,786 nats / 52 ev | 0 / 0 | 0 / 0 | 3,786 nats |
///
/// **The per-event price is 72.76 / 72.78 / 72.80 nats for a sleep and 36.28 for a wall
/// clamp, and it is the same everywhere because it is a fact about `f64` and not about the
/// scene.** `nats_collapsed` returns `ln(|x|/ulp(x))`, which is the mantissa — ≈36.4 nats —
/// for every normal double there is. So the nats and the events are proportional to within
/// 0.05% across every tier, and the ledger's reading is the event count in disguise.
///
/// **That is the sharpest form of the two-currency claim and also this instrument's limit.**
/// Set the two ledgers side by side on the SAME site, sleep: the D-ledger charges 2.76e-12 J
/// per sleep at the sandbox and 965.9 J per sleep at the landscape — **29 orders of
/// magnitude apart** — while the sigma-ledger charges 72.76 and 72.80 nats. Energy loss and
/// information loss are genuinely different events, and no conversion factor relates them.
/// But it also means this ledger **cannot tell a cheap collapse from an expensive one**: it
/// would read the same whether the engine slept a cell at 1 m/s or at 1e-30 m/s.
///
/// So the readings are a **CEILING on distinguishability destroyed, not a measurement of
/// it.** Pricing a discarded `x` at its full mantissa assumes every representable value
/// below it was equally available, and the states an engine actually visits are not uniform
/// over the mantissa. The near-constant per-event price is the tell: a real entropy would
/// vary with the state; this varies only with the format. Grading a site by how much
/// structure it destroyed needs the reachable set, which this does not have.
///
/// **Three sites identified, two live.** The anchor reads exactly zero at every tier and is
/// dead by construction rather than by scene — an anchored cell's velocity is zeroed
/// *before* force reaches it, so nothing ever accumulates there to collapse. The wall clamp
/// is the sandbox's largest single site (51% of its nats) and is zero at the other two,
/// which IS a scene fact: nothing there reaches a wall.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MergeLedger {
    pub sleep_nats: f64,
    pub anchor_nats: f64,
    pub wall_clamp_nats: f64,
    pub sleep_events: u64,
    pub anchor_events: u64,
    pub wall_clamp_events: u64,
}

impl MergeLedger {
    pub fn total_nats(&self) -> f64 {
        self.sleep_nats + self.anchor_nats + self.wall_clamp_nats
    }

    pub fn total_events(&self) -> u64 {
        self.sleep_events + self.anchor_events + self.wall_clamp_events
    }

    /// The same total in bits, for readers who find "one merge destroys one bit" clearer.
    /// **1 nat = 1/ln 2 = 1.442695… bits.**
    pub fn total_bits(&self) -> f64 {
        self.total_nats() / core::f64::consts::LN_2
    }
}

/// Nats destroyed by discarding `x` — the distinctions it could have carried.
///
/// `|x| / ulp(x)` representable values collapse to one, so the destroyed entropy is the log
/// of that count. Uses `reversibility::next_ulp` rather than a second implementation: it is
/// the same ULP notion the float production floor is measured with, and two detectors for
/// one event is how they drift apart.
fn nats_collapsed(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return 0.0;
    }
    let magnitude = x.abs();
    let ulp = ciris_sim_core::reversibility::next_ulp(magnitude) - magnitude;
    if ulp <= 0.0 {
        return 0.0;
    }
    (magnitude / ulp).ln()
}

pub struct Session {
    pub tier: Tier,
    /// Per-channel dissipation, joules, accumulated since the scene was built.
    ledger: DissipationLedger,
    /// Distinguishability destroyed at the merge sites, nats.
    merge_ledger: MergeLedger,
    /// Total energy at the moment the scene finished being built — the `E(0)` of the
    /// two-sided balance. Recorded once, never updated.
    opening_energy_j: f64,
    arena: RuntimeArena,
    materializer: QuadrantMaterializer,
    model: ResolutionModel,
    workspace: Workspace,
    chart: Chart,
    nodes: Nodes,
    relations: Relations,
    projectile: Projectile,
    verdict: Verdict,
    /// Cohesive law refusal, when the relations are contact-only.
    law_refusal: Option<LawRefusal>,
    contact_stiffness_n_m: f64,
    /// The stiffness the tier's own material values imply, kept alongside the softened
    /// one so the demo can show both.
    physical_stiffness_n_m: f64,
    contact_damping_n_s_m: f64,
    impact_speed_m_s: f64,
    dt_s: f64,
    time_s: f64,
    impulse_n_s: f64,
    /// Net contact impulse VECTOR. Its magnitude is the momentum the contact actually
    /// transferred, and it is what the momentum-conservation gate checks; `impulse_n_s`
    /// is the total pushing done, which is larger whenever the direction changes.
    impulse_vec: [f64; 2],
    peak_contact_n: f64,
    disturbance_m: f64,
    materializations: usize,
    rounds: usize,
    /// The vacuum tier's state. Present on every session and used by one, which is the
    /// same shape the material chart has: a value that most tiers leave alone.
    gauge: GaugeScene,
    /// Which declared scene of a gravity tier is live. Each carries two, because one
    /// number cannot say what the certificate says.
    gravity_scene: usize,
    /// The weak-field certificate for the live gravity scene, when there is one.
    weak_field: Option<WeakFieldCertificate>,
    /// The thrown body's worldline, on shell in the declared chart.
    worldline: Option<Worldline>,
    /// Where it has been, in scene metres, for drawing. Bounded; oldest dropped.
    trail: Vec<[f64; 2]>,
    /// Set when a ballistic body returns to the height it was thrown from.
    landed: bool,
    broadphase: Broadphase,
    pairs: Vec<(usize, usize)>,
    wake: Vec<usize>,
    /// The awake cells, and the candidate pairs and bonds with at least one awake end.
    ///
    /// Skipping asleep work inside loops over every resident cell is not enough: at
    /// 118,296 cells the loop itself is the cost even when every iteration does nothing.
    /// These lists are rebuilt only when the awake set actually changes, which after the
    /// first burst of a throw is rare, so a substep costs what is MOVING rather than
    /// what is resident.
    awake_list: Vec<usize>,
    active_pairs: Vec<(usize, usize)>,
    active_bonds: Vec<usize>,
    /// Set whenever a cell wakes or sleeps; cleared when the lists are rebuilt.
    awake_dirty: bool,
    /// Reused buffer for the resident cells near the projectile, and where the
    /// projectile was when it was built.
    near: Vec<usize>,
    near_anchor: [f64; 2],
    /// Candidate-pair indices touching each node, so the awake working set is built from
    /// the awake nodes rather than by filtering every pair in the scene.
    pair_adjacency: Vec<Vec<u32>>,
    /// Cells that woke since the working set was last rebuilt.
    woke: Vec<usize>,
    sleep_speed_fraction: f64,

    /// Bumped whenever a cell falls asleep, so a host that caches a static rendering of
    /// the resting cells knows when to rebuild it.
    sleep_generation: u64,
    /// Reusable per-substep force accumulator. Allocating one per substep cost more
    /// than the forces did.
    force: Vec<[f64; 2]>,
    /// Node positions at the last broadphase rebuild, and the largest displacement any
    /// node has accumulated since. The pair list is built with a margin and is valid
    /// until something has moved half of it — the standard Verlet-list criterion, with
    /// the criterion checked rather than a fixed rebuild interval guessed at.
    anchor: Vec<[f64; 2]>,
    margin_m: f64,
    /// Where the certified frontier put each node, so the disturbance is measured
    /// against the scene's own rest state.
    rest: Vec<[f64; 2]>,
    work_budget: usize,
    substeps: usize,
    impulse_mode: ImpulseMode,
    /// Physics seconds advanced per second of wall clock, measured. A grain contact is
    /// stiff enough that this is well under one, and the demo says so rather than
    /// letting a viewer assume they are watching real time.
    slow_motion: f64,
}

impl Session {
    pub fn new(id: TierId) -> Self {
        Self::with_grading(id, GRADING)
    }

    /// A session whose resolution demand relaxes with distance at `grading`.
    ///
    /// This is the demo's one interactive knob on the certificate itself: smaller
    /// values demand a fine frontier over a wider area, so the same throw materializes
    /// more holons, costs more to certify, and resolves more. Making it a control is
    /// the point — the certified frontier should be visible as a thing with a price,
    /// not an invisible internal.
    pub fn with_grading(id: TierId, grading: f64) -> Self {
        let mut session = Self::unsettled(id, grading);
        session.settle();
        // E(0) is recorded once the scene has SETTLED, not when it was allocated: settling
        // is scene construction, and energy the settler removes is not dissipation the
        // stepping ledger is accountable for.
        session.opening_energy_j = session.total_energy_j();
        session.ledger = DissipationLedger::default();
        session
    }

    /// A session whose scene has not been certified yet. Only [`Self::with_grading`]
    /// and the tests use this; every session handed to a caller has a resident frontier.
    fn unsettled(id: TierId, grading: f64) -> Self {
        let tier = tier(id);
        let arena = root_scene(&tier).expect("every declared tier builds a valid root");
        let matter_line = tier.fill * tier.domain_m;
        // The demand this tier makes of itself. Where the tier has no material chart
        // there is nothing to resolve, and the domain size is used so the surrogate is
        // satisfied immediately rather than refined forever.
        let required = tier.required_spacing_m().unwrap_or(tier.domain_m);
        // A cell already at or below the strictest possible demand can never need
        // refining wherever the throw lands, so it is settled. See
        // `QuadrantMaterializer::settled_size_m` for why this flag is load-bearing.
        // A cell is settled once no claim can ask more of it: finer than the physics
        // demand AND finer than the observer's.
        let settled = required.min(tier.acuity_m()).min(tier.g0_m);
        Self {
            tier,
            arena,
            materializer: QuadrantMaterializer::new(tier.domain_m, matter_line, settled),
            model: ResolutionModel::new(
                tier.domain_m,
                required,
                tier.acuity_m(),
                grading.clamp(0.05, 8.0),
                [0.5 * tier.domain_m, matter_line],
            ),
            workspace: Workspace::new(),
            chart: Chart::new(tier.domain_m),
            nodes: Nodes::default(),
            relations: Relations::default(),
            projectile: Projectile::default(),
            verdict: Verdict::Idle,
            law_refusal: None,
            contact_stiffness_n_m: 0.0,
            physical_stiffness_n_m: 0.0,
            contact_damping_n_s_m: 0.0,
            impact_speed_m_s: 0.0,
            dt_s: 0.0,
            time_s: 0.0,
            impulse_n_s: 0.0,
            impulse_vec: [0.0; 2],
            peak_contact_n: 0.0,
            disturbance_m: 0.0,
            materializations: 0,
            rounds: 0,
            gauge: GaugeScene::new(),
            gravity_scene: 0,
            ledger: DissipationLedger::default(),
            merge_ledger: MergeLedger::default(),
            opening_energy_j: 0.0,
            weak_field: None,
            worldline: None,
            trail: Vec::new(),
            landed: false,
            broadphase: Broadphase::default(),
            pairs: Vec::new(),
            wake: Vec::new(),
            awake_list: Vec::new(),
            active_pairs: Vec::new(),
            active_bonds: Vec::new(),
            awake_dirty: true,
            near: Vec::new(),
            near_anchor: [f64::INFINITY; 2],
            pair_adjacency: Vec::new(),
            woke: Vec::new(),
            sleep_speed_fraction: SLEEP_SPEED_FRACTION,
            sleep_generation: 0,
            force: Vec::new(),
            anchor: Vec::new(),
            margin_m: 0.0,
            rest: Vec::new(),
            work_budget: SUBSTEP_WORK_BUDGET,
            substeps: 0,
            impulse_mode: ImpulseMode::default(),
            slow_motion: 1.0,
        }
    }

    /// Physics seconds advanced per second of wall clock on the last frame.
    pub fn slow_motion(&self) -> f64 {
        self.slow_motion
    }

    /// Plant a deliberate defect in the impulse accounting. Production callers never
    /// touch this; it exists so the momentum gate can be shown to have teeth.
    pub fn set_impulse_mode(&mut self, mode: ImpulseMode) {
        self.impulse_mode = mode;
    }

    /// Resident-holon ceiling for this tier: what the observer's claim costs, plus
    /// headroom for the physics corridor.
    pub fn model_counters(&self) -> (usize, usize) {
        (self.model.calls, self.model.settles)
    }

    pub fn holon_ceiling(&self) -> usize {
        let acuity_cells = self.tier.acuity_cell_estimate();
        ((acuity_cells * PHYSICS_HEADROOM) as usize).clamp(1_024, 4_000_000)
    }

    /// Cells currently being integrated. Resident and drawn is not the same as awake.
    pub fn awake_count(&self) -> usize {
        self.awake_list.len()
    }

    /// Rebuild the awake working set. Free when nothing changed, `O(resident)` when it
    /// did — and that only happens on a wake or a sleep.
    fn refresh_awake(&mut self) {
        if !self.awake_dirty {
            return;
        }
        self.awake_dirty = false;

        // Build the pair adjacency if the scene changed under us. Doing it here rather
        // than only at certification means every path that produces nodes gets it,
        // instead of the one path that happened to be edited.
        if self.pair_adjacency.len() != self.nodes.len() {
            self.pair_adjacency.clear();
            self.pair_adjacency.resize(self.nodes.len(), Vec::new());
            for (index, (i, j)) in self.pairs.iter().copied().enumerate() {
                self.pair_adjacency[i].push(index as u32);
                self.pair_adjacency[j].push(index as u32);
            }
        }

        // The awake list is rebuilt from ITSELF plus what just woke, minus what just
        // slept — both small — rather than by scanning every resident cell. And the
        // active pairs and bonds come from the awake nodes' own adjacency rather than by
        // filtering every relation in the scene.
        //
        // Scanning was 65,539 nodes and 130,566 bonds EVERY time one cell flickered
        // between waking and sleeping, which at the grain tier is most substeps: 683 ms
        // of one frame with two cells awake. The cost was never the moving cells, it was
        // rebuilding a description of the still ones over and over.
        self.awake_list.retain(|index| self.nodes.awake[*index]);
        for index in self.woke.drain(..) {
            if self.nodes.awake[index] && !self.awake_list.contains(&index) {
                self.awake_list.push(index);
            }
        }
        self.awake_list.sort_unstable();
        self.awake_list.dedup();

        self.active_pairs.clear();
        self.active_bonds.clear();
        for node in self.awake_list.iter().copied() {
            for pair in self.pair_adjacency[node].iter().copied() {
                self.active_pairs.push(self.pairs[pair as usize]);
            }
            for bond in self.relations.touching(node).iter().copied() {
                self.active_bonds.push(bond as usize);
            }
        }
        self.active_pairs.sort_unstable();
        self.active_pairs.dedup();
        self.active_bonds.sort_unstable();
        self.active_bonds.dedup();
    }

    /// Refresh the resident cells near enough to the projectile to matter.
    ///
    /// The projectile has to be tested against ASLEEP cells too — it is what wakes them
    /// — so this cannot use the awake set. It uses the grid instead, into a reused
    /// buffer, which is why a moving projectile does not cost a sweep of the scene.
    /// Refresh only when the projectile has moved far enough to invalidate the list.
    ///
    /// The list is built with a MARGIN, so it stays valid until the projectile has
    /// travelled half of it — the same Verlet criterion the pair list uses, and a check
    /// rather than an interval. Rebuilding every substep was 190 us a substep at the
    /// grain tier and 755 ms of one frame, with two cells awake: the cost was never the
    /// physics, it was asking the grid the same question four thousand times while
    /// nothing moved far enough for the answer to change.
    fn refresh_near(&mut self) {
        let margin = 2.0 * self.projectile.radius_m;
        let dx = self.projectile.position[0] - self.near_anchor[0];
        let dy = self.projectile.position[1] - self.near_anchor[1];
        if !self.near.is_empty() && (dx * dx + dy * dy) * 4.0 < margin * margin {
            return;
        }
        self.near_anchor = self.projectile.position;
        let mut near = core::mem::take(&mut self.near);
        self.broadphase.near_into(
            self.projectile.position,
            self.projectile.radius_m + margin,
            self.tier.domain_m,
            &mut near,
        );
        self.near = near;
    }


    /// **Total mechanical energy of the scene, joules.** Kinetic plus gravitational plus
    /// the contact overlap potential currently stored in live pairs.
    ///
    /// # The overlap term's zero point is the FORCE LAW's, and getting that wrong was the
    /// whole of the reported energy creation
    ///
    /// The first version of this function measured overlap from `radius[i] + radius[j]`. The
    /// contact force does not: it measures from [`Session::rest_gap`], which is
    /// `min(radius[i] + radius[j], |rest[i] − rest[j]|)` — the certified frontier's own
    /// spacing when the lattice is packed tighter than geometric touching, which at the
    /// sandbox tier it is, by 44 um on a 250 um radius.
    ///
    /// So every packed pair sat at zero force and non-zero *potential*: about 6.7e-4 J of
    /// energy the force law does not have. The term is summed over `active_pairs`, which is
    /// derived from the AWAKE set, so as a throw woke the scene from 0 to 19,277 active
    /// pairs the fiction switched on pair by pair and the balance read **+1.15 J of created
    /// energy** — 1.01 of the 1.06% "gain", the rest of it real dissipation with the sign
    /// hidden underneath. Measured against the force law's own zero it is 1.1e-3 J.
    ///
    /// That is also what makes this a state function on a settled scene: at the frontier's
    /// own spacing the resting lattice stores exactly zero, so a pair's contribution does not
    /// depend on whether it happens to be awake.
    ///
    /// Scope, stated: **bond potential is not included**, so this is exact only where the
    /// scene has no live cohesive relations. That is the sandbox tier by construction — the
    /// homogenizer refuses a cohesive law at 0.5 mm cells — and it is why the gate below is
    /// declared there. A bonded tier needs the bond term added before its balance means
    /// anything, and `Relations::refusal` is the field that says which case a scene is in.
    ///
    /// Still missing, and named rather than left for the next reader to rediscover: the
    /// **projectile's own contact potential**. Projectile-cell overlap is not a node-node
    /// pair, so it is not in this sum; it is non-zero only while the projectile is touching
    /// something, and it is part of the residual the gate below reports.
    pub fn total_energy_j(&self) -> f64 {
        let mut e = 0.0;
        for i in 0..self.nodes.len() {
            let m = self.nodes.mass_kg[i];
            let v = self.nodes.velocity[i];
            e += 0.5 * m * (v[0] * v[0] + v[1] * v[1]);
            e += m * CHART_GRAVITY_M_S2 * self.nodes.position[i][1];
        }
        if self.projectile.live {
            let m = self.projectile.mass_kg;
            let v = self.projectile.velocity;
            e += 0.5 * m * (v[0] * v[0] + v[1] * v[1]);
            e += m * CHART_GRAVITY_M_S2 * self.projectile.position[1];
        }
        for (i, j) in self.active_pairs.iter().copied() {
            let delta = [
                self.nodes.position[j][0] - self.nodes.position[i][0],
                self.nodes.position[j][1] - self.nodes.position[i][1],
            ];
            let distance = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt();
            // The force law's zero, not geometric touching. See this function's header.
            let touching = self.rest_gap(i, j);
            if distance < touching {
                let overlap = touching - distance;
                e += 0.5 * self.contact_stiffness_n_m * overlap * overlap;
            }
        }
        e
    }

    /// Dissipation so far, per named channel.
    pub fn energy_ledger(&self) -> DissipationLedger {
        self.ledger
    }

    /// Distinguishability destroyed so far, per merge site, in nats.
    pub fn merge_ledger(&self) -> MergeLedger {
        self.merge_ledger
    }

    /// `E(0)` — the scene's energy when it was built.
    pub fn opening_energy_j(&self) -> f64 {
        self.opening_energy_j
    }

    /// **Is the two-sided balance a law in this scene's chart?** See
    /// [`BalanceApplicability`] for why a conservation gate has to ask.
    ///
    /// The expansion-versus-static-curvature discriminator is delegated to the weak-field
    /// screen rather than decided here from the tier's name, because that screen already
    /// computes `(H L/c)²` against the scene's declared envelope and already refuses BY NAME
    /// when it dominates. Deciding it twice is how two answers drift apart.
    pub fn balance_applicability(&self) -> BalanceApplicability {
        match self.tier.evaluator {
            Evaluator::GranularContact | Evaluator::Cohesive => BalanceApplicability::Holds,
            Evaluator::GeodesicChart { .. } => {
                let expanding = self
                    .gravity_scene()
                    .map(|scene| scene.certify().refusal)
                    .unwrap_or(None)
                    == Some(WeakFieldRefusal::ExpansionScale);
                if expanding {
                    BalanceApplicability::RefusedNoTimeTranslationSymmetry
                } else {
                    BalanceApplicability::RefusedConservedQuantityNotComputed
                }
            }
            Evaluator::GaugePlaquette | Evaluator::Unavailable(_) => {
                BalanceApplicability::RefusedNoMechanicalEvaluator
            }
        }
    }

    /// **The two-sided balance's residual:** `E(0) − E(t) − Σ channels`.
    ///
    /// Not expected to be zero. Semi-implicit Euler is not energy-conserving, so this term
    /// carries the integrator's own error; it is named so that error is not mistaken for a
    /// missing channel. Quote it relative to the dissipated total.
    pub fn energy_residual_j(&self) -> f64 {
        self.opening_energy_j - self.total_energy_j() - self.ledger.total_j()
    }

    /// The residual as a fraction of everything that left the scene — **the number the gate
    /// reports**. Small means the named channels explain the dissipation; large means they
    /// do not, which is exactly what a two-sided gate exists to catch.
    ///
    /// The divide-by-zero guard is **relative to `E(0)`, not an absolute joule count**. It
    /// was `dissipated.abs() < 1e-12` in absolute joules, which is larger than the grain
    /// tier's ENTIRE energy (8.1e-10 J): that tier's real 4.6% residual reported as a clean
    /// 0.000 for the same reason a metre-stick reports a grain of sand as zero long. The
    /// scene sets the scale; the instrument does not get to.
    pub fn energy_residual_fraction(&self) -> f64 {
        let dissipated = self.opening_energy_j - self.total_energy_j();
        if dissipated.abs() <= 1.0e-12 * self.opening_energy_j.abs() {
            return 0.0;
        }
        self.energy_residual_j() / dissipated
    }

    /// Counter that changes whenever a cell falls asleep.
    pub fn sleep_generation(&self) -> u64 {
        self.sleep_generation
    }

    /// Substeps this frame advanced.
    pub fn substeps(&self) -> usize {
        self.substeps
    }

    /// Test hook: the speed below which a cell may sleep, as a fraction of the impact
    /// speed. Setting it to zero disables sleeping, which is how the approximation is
    /// measured against itself.
    pub fn set_sleep_speed_fraction(&mut self, fraction: f64) {
        self.sleep_speed_fraction = fraction.max(0.0);
    }

    /// Set the per-frame solver work budget, in node-substeps.
    ///
    /// The engine has no clock on `wasm32-unknown-unknown` — `std::time::Instant` panics
    /// there — so it cannot tune this itself, and guessing a constant that holds across
    /// native, wasm, and whatever machine a reader opens the page on is not something a
    /// constant can do. The host measures how long a frame actually took and moves this
    /// toward its target. The engine keeps the physics; the host keeps the clock.
    pub fn set_work_budget(&mut self, budget: usize) {
        self.work_budget = budget.clamp(64, 4_000_000);
    }

    pub fn arena(&self) -> &RuntimeArena {
        &self.arena
    }

    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    /// Direct write access to the resident state, for instruments that prepare an
    /// initial condition the scene builder cannot express. Production callers never
    /// touch this — like [`Self::set_impulse_mode`] it exists so a claim about the
    /// solver can be put to a test that needs a hand-placed micro-state.
    ///
    /// The caller owns the invariants: a velocity written onto a cell that is not in
    /// the awake set is never integrated, and the sleep pass will zero it.
    pub fn nodes_mut(&mut self) -> &mut Nodes {
        &mut self.nodes
    }

    pub fn relations(&self) -> &Relations {
        &self.relations
    }

    pub fn projectile(&self) -> &Projectile {
        &self.projectile
    }

    pub fn verdict(&self) -> Verdict {
        self.verdict
    }

    pub fn law_refusal(&self) -> Option<LawRefusal> {
        self.law_refusal
    }

    pub fn chart(&self) -> &Chart {
        &self.chart
    }

    /// The live gravity scene of this tier, if it has one.
    pub fn gravity_scene(&self) -> Option<&'static crate::gravity::GravityScene> {
        crate::gravity::scenes_for(self.tier.id).get(self.gravity_scene)
    }

    pub fn gravity_scene_index(&self) -> usize {
        self.gravity_scene
    }

    /// Select a declared scene. Out-of-range selects the first.
    pub fn set_gravity_scene(&mut self, index: usize) {
        let count = crate::gravity::scenes_for(self.tier.id).len();
        self.gravity_scene = if index < count { index } else { 0 };
    }

    pub fn weak_field(&self) -> Option<&WeakFieldCertificate> {
        self.weak_field.as_ref()
    }

    pub fn weak_field_refusal(&self) -> Option<WeakFieldRefusal> {
        self.weak_field.and_then(|certificate| certificate.refusal)
    }

    /// Has a ballistic body come back to the height it was thrown from?
    pub fn landed(&self) -> bool {
        self.landed
    }

    /// The body's trail in SCENE metres, oldest first.
    pub fn trail(&self) -> &[[f64; 2]] {
        &self.trail
    }

    pub fn gauge(&self) -> &GaugeScene {
        &self.gauge
    }

    pub fn gauge_mut(&mut self) -> &mut GaugeScene {
        &mut self.gauge
    }

    pub fn time_s(&self) -> f64 {
        self.time_s
    }

    /// Total contact impulse MAGNITUDE: how much pushing the throw did. Equals the
    /// momentum transferred for a single straight contact and exceeds it whenever the
    /// direction changes or the projectile strikes more than once.
    pub fn impulse_n_s(&self) -> f64 {
        self.impulse_n_s
    }

    /// Magnitude of the NET contact impulse — the momentum the contact actually
    /// transferred to the scene. This is the conserved quantity, and
    /// `tests::the_contact_impulse_is_the_momentum_the_projectile_gave_up` checks it
    /// against the projectile's own change in momentum.
    pub fn net_impulse_n_s(&self) -> f64 {
        if self.impulse_mode == ImpulseMode::SumOfMagnitudes {
            // The mutant IS the old accounting: one field, the scalar sum, carried
            // straight onto the certificate. Planting it anywhere else would leave the
            // conserved quantity correct and the gate with nothing to catch — which is
            // what the first version of this mutant did, and why it read 1.01x.
            return self.impulse_n_s;
        }
        (self.impulse_vec[0] * self.impulse_vec[0]
            + self.impulse_vec[1] * self.impulse_vec[1])
            .sqrt()
    }

    pub fn peak_contact_n(&self) -> f64 {
        self.peak_contact_n
    }

    pub fn disturbance_m(&self) -> f64 {
        self.disturbance_m
    }

    pub fn materializations(&self) -> usize {
        self.materializations
    }

    pub fn rounds(&self) -> usize {
        self.rounds
    }

    pub fn dt_s(&self) -> f64 {
        self.dt_s
    }

    /// Order-of-magnitude strain rate this throw imposes at the contact: impact speed
    /// over the finest resident cell.
    ///
    /// Worth computing because of where it lands. DESCRIPTOR_CHAIN §3.3 carries a
    /// strain-rate GAP: molecular-dynamics certificates cover 1e7-1e9 per second, lab
    /// quasi-static ones cover 1e-6-1e0, and **1e0 to 1e7 is an interpolation the
    /// consumer must accept** because no simulation crosses it. Both tiers that certify
    /// here land inside that gap. The demo surfaces the pin rather than burying it;
    /// see `STRAIN_RATE_GAP`.
    pub fn strain_rate_per_s(&self) -> f64 {
        let finest = self
            .nodes
            .radius_m
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min)
            * 2.0;
        if !(finest.is_finite() && finest > 0.0) {
            return 0.0;
        }
        self.impact_speed_m_s / finest
    }

    /// Screen the live gravity scene and set up its body.
    ///
    /// The verdict comes straight from `bridge::certify_weak_field` — three outcomes,
    /// all of them honest: certified inside the screen; `GrainFloor` where the curved
    /// and flat charts are indistinguishable at f64 and the FLAT one is licensed; and a
    /// typed refusal that names its own unlock.
    fn certify_geodesic(&mut self, tier_eps_max: f64) {
        let Some(scene) = self.gravity_scene() else {
            self.verdict = Verdict::WeakFieldRefused;
            return;
        };
        let certificate = scene.certify();
        debug_assert!(
            scene.tier_eps_max <= tier_eps_max || tier_eps_max <= scene.tier_eps_max,
            "the scene's stake and the tier's are both declared; neither derives"
        );
        self.weak_field = Some(certificate);
        self.verdict = match certificate.status {
            CertificationStatus::Certified => Verdict::Certified,
            CertificationStatus::GrainFloor => Verdict::GrainFloor,
            CertificationStatus::RefinementUnavailable => Verdict::WeakFieldRefused,
        };

        self.trail.clear();
        self.worldline = None;
        self.landed = false;
        // A refused scene gets no body. Stepping a trajectory the certificate has just
        // declined to vouch for would be drawing a claim that was refused.
        if certificate.status == CertificationStatus::RefinementUnavailable {
            return;
        }
        if let Some((pos, celerity)) = scene.body {
            let chart = scene.chart();
            let line = curved_from_celerity(&chart, [0.0, pos[0], pos[1], pos[2]], celerity);
            self.trail.push(scene_point(scene, &line));
            self.worldline = Some(line);
        }
    }

    /// Advance the body along a geodesic of the declared chart.
    ///
    /// No mass appears anywhere in this call. Universality of free fall is a property of
    /// `geodesic_accel`'s construction, not something re-imposed here, and the bridge's
    /// own gate is what checks it.
    fn step_geodesic(&mut self) {
        let (Some(scene), Some(line)) = (self.gravity_scene(), self.worldline) else {
            return;
        };
        if scene.dtau_s <= 0.0 {
            return;
        }
        // A ballistic scene ENDS when the body comes back to the height it left. There
        // is no ground in a potential — left running, the ball falls toward Earth's
        // centre forever, and after sixteen seconds it is two kilometres below a scene
        // that declared a hundred-metre envelope. The throw is over when it lands, and
        // stopping there is what keeps the motion inside the guarantee it was screened
        // against.
        if self.landed {
            return;
        }
        let chart = scene.chart();
        // A scene whose second view axis is the chart's z is one where "down" means
        // something; an orbital scene in (x, y) has no landing to detect.
        let ballistic = scene.plane[1] == 2;

        // ONE trail point per RK4 step, not per frame.
        //
        // The drawn path has to satisfy the observer's claim like everything else that
        // is shown, and a polyline is only as fine as its samples: the straight line
        // BETWEEN two computed points is interpolation, not computed. Recording once a
        // frame put the planet's samples 2.38 m apart against a 0.67 m acuity — 3.6x too
        // coarse — and the galactic ones 1.6x. Per step they are 0.2 to 0.45x, and
        // `tests::the_drawn_path_is_no_coarser_than_the_observer_can_see` keeps it so.
        let mut line = line;
        for _ in 0..GEODESIC_STEPS_PER_FRAME {
            let next = integrate_geodesic(&chart, &line, scene.dtau_s, 1);
            if ballistic && self.trail.len() > 2 && scene_point(scene, &next)[1] <= 0.0 {
                self.landed = true;
                self.worldline = Some(line);
                return;
            }
            line = next;
            self.time_s += scene.dtau_s;
            if self.trail.len() >= GEODESIC_TRAIL_MAX {
                self.trail.remove(0);
            }
            self.trail.push(scene_point(scene, &line));
        }
        self.worldline = Some(line);
    }

    /// Certify a frontier for a throw aimed at `aim` (fractions of the domain, 0..1) and
    /// launch the projectile.
    ///
    /// Certification is an EVENT. It runs once, here, and never inside the frame loop —
    /// which is why the budget is a ceiling on ROUNDS and HOLONS rather than a time
    /// slice: `std::time::Instant` panics on `wasm32-unknown-unknown`, so the engine
    /// cannot hold itself to a wall-clock budget and does not pretend to. The host times
    /// this call and reports what it cost.
    /// Make the scene resident at PREVIEW grain, with nothing claimed about it.
    ///
    /// This exists because of a bug Eric found in the browser: the canvas was blank
    /// until the first click. Not because rendering was gated on input — the draw loop
    /// ran from the first frame — but because there was NOTHING RESIDENT to draw. No
    /// throw meant no certification, no certification meant no frontier, and a sandbox
    /// full of sand rendered as an empty box.
    ///
    /// The first fix was to run the certifier at rest with the focus at the centre of
    /// the sand. That was wrong twice over. It produced a FINER frontier than a throw
    /// does (116 cells against 110 — a focus buried in the matter has more matter around
    /// it than one at the surface), and more importantly it answered a question nobody
    /// asked: the certificate says whether a frontier resolves what the tier CLAIMS, and
    /// a scene at rest claims nothing. There is no impulse to certify.
    ///
    /// So this is not a certificate and does not pretend to be one. It materializes the
    /// matter uniformly down to [`PREVIEW_FRACTION`] of the domain — a declared
    /// rendering depth, cheap and coarse — and leaves the verdict at [`Verdict::Idle`].
    /// What you see before you throw is the scene's latent structure at a preview grain,
    /// and the throw's fine corridor then reads against it.
    pub fn settle(&mut self) {
        self.certify_at(None, 0.0);
        self.projectile.live = false;
        // A resting scene has served the observer's claim and nothing else, so it holds
        // no verdict about any physics. `Idle` says exactly that — EXCEPT on a gravity
        // tier, where the weak-field screen does not depend on anything being thrown.
        // Whether a declared chart can carry a declared scene is answered by the
        // envelope alone, so that verdict is true at rest and blanking it would hide a
        // certificate the tier has already earned.
        let screened = matches!(self.tier.evaluator, Evaluator::GeodesicChart { .. });
        if !screened && matches!(self.verdict, Verdict::Certified | Verdict::GrainFloor) {
            self.verdict = Verdict::Idle;
        }
    }

    /// Certify a frontier for an interaction focused at `focus`, and build the resident
    /// mechanical state on it. Shared by [`Self::settle`] and [`Self::throw`], which
    /// differ only in where the focus is and whether anything is launched afterwards.
    fn certify_at(&mut self, focus: Option<[f64; 2]>, speed_fraction: f64) -> Option<f64> {
        self.time_s = 0.0;
        self.impulse_n_s = 0.0;
        self.impulse_vec = [0.0; 2];
        self.peak_contact_n = 0.0;
        self.disturbance_m = 0.0;
        self.nodes.clear();
        self.relations = Relations::default();

        // Tiers with no evaluator refuse before anything is materialized. Refusing early
        // is the point: there is no frontier fine enough to rescue a claim that has no
        // way to be evaluated at all. Doing it here rather than in `throw` means zooming
        // to such a tier shows its refusal immediately, instead of looking idle until
        // someone throws something at it to find out.
        match self.tier.evaluator {
            Evaluator::Unavailable(refusal) => {
                self.verdict = match refusal {
                    crate::tier::Refusal::NoValidatedEvaluator => Verdict::NoEvaluator,
                    crate::tier::Refusal::NoGravityChart => Verdict::NoGravityChart,
                };
                self.projectile.live = false;
                return None;
            }
            // The vacuum tier's state is its flux, not a frontier; there is nothing to
            // certify at rest and its verdict is whatever its last move produced.
            Evaluator::GaugePlaquette => return None,
            // A gravity tier is screened, not materialized. Nothing here is a
            // coarse-grained matter field, so the observer's claim has nothing to bind
            // on: what is drawn is one body and the path it takes, and a path is not a
            // partition of anything. Materializing 262,144 cells of Earth-interior to
            // satisfy a demand about matter that is not being shown would be paying for
            // a picture nobody asked for.
            Evaluator::GeodesicChart { tier_eps_max } => {
                self.certify_geodesic(tier_eps_max);
                return None;
            }
            Evaluator::GranularContact | Evaluator::Cohesive => {}
        }

        self.model.set_focus(focus);

        let budget = Budget {
            macro_tolerance: 0.0,
            conservation_tolerance: CONSERVATION_TOLERANCE,
            max_rounds: MAX_ROUNDS,
            max_holons: self.holon_ceiling(),
        };
        let outcome = certify_incremental(
            &mut self.arena,
            &mut self.model,
            &mut self.materializer,
            &mut self.workspace,
            budget,
        );

        let certificate = match outcome {
            Ok(certificate) => certificate,
            Err(IncrementalError::RoundBudgetExhausted) => {
                self.verdict = Verdict::BudgetExhausted;
                self.projectile.live = false;
                return None;
            }
            Err(_) => {
                self.verdict = Verdict::RefinementUnavailable;
                self.projectile.live = false;
                return None;
            }
        };
        self.materializations = certificate.materializations;
        self.rounds = certificate.rounds;
        self.verdict = match certificate.status {
            CertificationStatus::Certified => Verdict::Certified,
            CertificationStatus::GrainFloor => Verdict::GrainFloor,
            CertificationStatus::RefinementUnavailable => Verdict::RefinementUnavailable,
        };

        self.chart.sync(&self.arena);
        let mass_per = mass_per_constituent_kg(&self.tier);
        self.nodes = build_nodes(
            &self.arena,
            &self.chart,
            &certificate.active,
            mass_per,
            self.tier.domain_m,
        );

        // The smallest node sets both the relation law's spacing and the stability
        // limit, so both are read off THE SAME node. Taking the minimum diameter and
        // the minimum mass independently pairs a fine cell's spacing with some other
        // cell's mass, and the derived stiffness is then a number about no node that
        // exists — which is how the landscape tier came to be running a 5.9e13 N/m
        // contact against a 7.6 mm cell.
        let smallest = (0..self.nodes.len()).fold(None::<usize>, |best, i| match best {
            Some(current) if self.nodes.radius_m[current] <= self.nodes.radius_m[i] => {
                Some(current)
            }
            _ => Some(i),
        });
        let (finest, node_mass) = match smallest {
            Some(i) => (2.0 * self.nodes.radius_m[i], self.nodes.mass_kg[i]),
            None => (f64::INFINITY, f64::INFINITY),
        };
        // The relation law is asked for at the FINEST resident spacing, because that is
        // the spacing the relations actually have. A coarser frontier gets a refusal and
        // becomes contact-only, which is the honest reading of a chart that cannot
        // resolve its own process zone.
        let law = if finest.is_finite() && node_mass.is_finite() {
            relation_law(&self.tier, finest, node_mass)
        } else {
            Err(LawRefusal::NoMaterial)
        };
        self.law_refusal = law.err();
        // Candidate pairs from the grid, not an all-pairs sweep. `build_relations` was
        // O(N^2) and invisible at two hundred nodes; pinning the scene at observer
        // acuity took the landscape frontier to 157,804 and the sweep to 2.5e10 pair
        // checks — 33 of the 34 seconds one throw cost. The sandbox hid it entirely,
        // because its cohesive law is refused and the function returns before the loop.
        let margin = self
            .nodes
            .radius_m
            .iter()
            .copied()
            .fold(0.0_f64, f64::max)
            * (RELATION_REACH - 1.0).max(0.0)
            * 2.0;
        self.broadphase
            .rebuild(&self.nodes, self.tier.domain_m, margin);
        let mut pairs = core::mem::take(&mut self.pairs);
        self.broadphase.pairs(&mut pairs);
        self.relations =
            build_relations(&self.nodes, &self.arena, &pairs, law, RELATION_REACH);
        self.pairs = pairs;
        self.anchor.clear();

        self.rest.clear();
        self.rest.extend_from_slice(&self.nodes.position);
        self.anchor.clear();
        self.impact_speed_m_s = impact_speed(&self.tier, speed_fraction);
        self.set_contact_law(finest, node_mass);
        // The STABILITY limit is a different question from the contact law's, and
        // collapsing them was an energy-injecting bug: the fastest oscillator in the
        // scene is set by the LIGHTEST node, which on a graded frontier is often a
        // barely-filled cell several levels coarser than the smallest one. Sizing the
        // step from the smallest cell's mass instead left dt 5.7x above the explicit
        // limit, and the sandbox projectile left the box at twelve times its launch
        // speed.
        let lightest = self
            .nodes
            .mass_kg
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        self.dt_s = self.stable_step(finest, lightest);
        Some(finest)
    }

    /// Certify a frontier for a throw aimed at `aim` (fractions of the domain, 0..1) and
    /// launch the projectile.
    ///
    /// Certification is an EVENT. It runs once, here, and never inside the frame loop —
    /// which is why the budget is a ceiling on ROUNDS and HOLONS rather than a time
    /// slice: `std::time::Instant` panics on `wasm32-unknown-unknown`, so the engine
    /// cannot hold itself to a wall-clock budget and does not pretend to. The host times
    /// this call and reports what it cost.
    /// Launch, and **re-baseline the energy balance**.
    ///
    /// The throw is an energy INJECTION, not dissipation: it puts a live projectile with
    /// kinetic energy into a scene that did not have one. The first run of the two-sided
    /// balance caught exactly this — every tier read a NEGATIVE dissipation, because `E(0)`
    /// had been recorded before the launch and `E(t)` included a projectile that was never
    /// in it. So `E(0)` is re-taken here: the ledger is accountable for the FLIGHT, and the
    /// launch is the initial condition rather than a channel.
    pub fn throw(&mut self, aim_x: f64, aim_y: f64, speed_fraction: f64) {

        if let Evaluator::GaugePlaquette = self.tier.evaluator {
            // The vacuum tier's throw is a magnetic plaquette move, and it either
            // applies or hits the spin-1 truncation — a refusal about the state space
            // ending, not about resolution.
            self.verdict = match self.gauge.raise() {
                Move::Applied => Verdict::Certified,
                Move::FluxCeiling => Verdict::FluxCeiling,
            };
            self.projectile.live = false;
            return;
        }

        let matter_line = self.tier.fill * self.tier.domain_m;
        let focus = [
            aim_x.clamp(0.0, 1.0) * self.tier.domain_m,
            (aim_y.clamp(0.0, 1.0) * self.tier.domain_m).min(matter_line),
        ];
        let Some(finest) = self.certify_at(Some(focus), speed_fraction) else {
            return;
        };
        self.launch(focus, speed_fraction, finest);
        self.opening_energy_j = self.total_energy_j();
        self.ledger = DissipationLedger::default();
        self.merge_ledger = MergeLedger::default();
    }

    /// The contact law, and the one place this demo knowingly softens physics.
    ///
    /// The PHYSICAL stiffness is `derive_lattice_elastic_law`'s homogenized normal
    /// stiffness at this spacing — the tier's own chart values, no tuning. At the
    /// sandbox that is 1.35e7 N/m on a 1.73e-7 kg grain, an oscillator period of
    /// 7.1e-7 s, so an explicit integrator needs a 4.2e-8 s step. Real quartz grains at
    /// real stiffness cannot be explicitly integrated at interactive rates, in a
    /// browser or anywhere else, and no amount of engineering in this crate changes
    /// that.
    ///
    /// So the demo uses the soft-particle stiffness that discrete-element practice has
    /// used since Cundall & Strack (1979): the smallest stiffness for which the maximum
    /// contact overlap at the scene's own impact speed stays under a declared fraction
    /// of a grain radius, `k = m v^2 / (f R)^2` by energy balance. That fraction is
    /// [`MAX_OVERLAP_FRACTION`] and it is DECLARED, not fitted.
    ///
    /// This is solver-owned, exactly like `IsotropicMaterial::solver_damping_ratio`:
    /// it has a criterion rather than a physics-tier ancestor. Both stiffnesses are
    /// kept and both are shown, so a reader can see the factor by which the demo is
    /// softer than the material it names. Nothing certified rides on it — the
    /// certificate is about whether the FRONTIER resolves the tier's own demand, and
    /// the contact law does not enter that question.
    fn set_contact_law(&mut self, spacing_m: f64, node_mass_kg: f64) {
        let Some(material) = self.tier.material else {
            self.contact_stiffness_n_m = 0.0;
            self.contact_damping_n_s_m = 0.0;
            self.physical_stiffness_n_m = 0.0;
            return;
        };
        let physical = derive_lattice_elastic_law(&material, node_mass_kg, spacing_m)
            .map(|elastic| elastic.normal_stiffness_n_m)
            .unwrap_or(0.0);
        self.physical_stiffness_n_m = physical;
        if physical <= 0.0 {
            self.contact_stiffness_n_m = 0.0;
            self.contact_damping_n_s_m = 0.0;
            return;
        }

        let impact = self.impact_speed_m_s.max(1.0e-6);
        let allowed = MAX_OVERLAP_FRACTION * 0.5 * spacing_m;
        let soft = node_mass_kg * impact * impact / (allowed * allowed);
        // Never STIFFER than the material: the criterion may only soften.
        self.contact_stiffness_n_m = soft.min(physical);

        // Restitution is imposed through the contact damping that produces it for this
        // pair's reduced mass, which keeps restitution a per-contact OUTCOME rather
        // than a material field (A5).
        let log_e = CONTACT_RESTITUTION.max(1.0e-6).ln();
        let zeta = -log_e / (log_e * log_e + core::f64::consts::PI.powi(2)).sqrt();
        self.contact_damping_n_s_m =
            2.0 * zeta * (self.contact_stiffness_n_m * 0.5 * node_mass_kg).sqrt();
    }

    pub fn contact_stiffness_n_m(&self) -> f64 {
        self.contact_stiffness_n_m
    }

    pub fn physical_stiffness_n_m(&self) -> f64 {
        self.physical_stiffness_n_m
    }

    /// How much softer than the named material this scene's contact is. 1.0 means the
    /// physical stiffness is in use.
    pub fn softening_factor(&self) -> f64 {
        if self.contact_stiffness_n_m > 0.0 {
            self.physical_stiffness_n_m / self.contact_stiffness_n_m
        } else {
            1.0
        }
    }

    /// Explicit stability limit for the stiffest, lightest resident pair.
    ///
    /// `node_mass_kg` must be the LIGHTEST node in the scene, not the smallest one:
    /// the fastest oscillator is `sqrt(k/m)` and it is mass that sets it.
    fn stable_step(&self, spacing_m: f64, node_mass_kg: f64) -> f64 {
        let stiffness = self
            .relations
            .bonds
            .iter()
            .map(|bond| bond.law.stiffness_n_m)
            .fold(0.0_f64, f64::max)
            .max(self.contact_stiffness_n_m);
        if !(stiffness > 0.0 && node_mass_kg > 0.0) {
            return 1.0e-4;
        }
        // Semi-implicit Euler on a linear spring is stable for dt < 2/omega; a tenth of
        // the PERIOD is dt = 0.1 * 2*pi/omega ~ 0.63/omega, comfortably inside it, and
        // the margin is what absorbs the damping and friction terms the bound does not
        // cover.
        let period = core::f64::consts::TAU * (node_mass_kg / stiffness).sqrt();
        let dt = 0.1 * period;
        // And never step so far that something moving at the impact speed could pass
        // straight through a contact in one step. The tolerance is the same overlap the
        // contact law is declared against, so the two criteria agree instead of one of
        // them being a number chosen to make the other work.
        let travel_limit = MAX_OVERLAP_FRACTION * 0.5 * spacing_m
            / self.impact_speed_m_s.max(1.0e-6);
        dt.min(travel_limit).clamp(1.0e-12, 1.0e-3)
    }

    /// Place the projectile just clear of the matter it is aimed at, moving at the
    /// impact speed.
    ///
    /// The release height is read off the RESIDENT NODES rather than the matter line,
    /// because a coarse partly-filled cell's disc reaches well above the line it
    /// straddles: on the landscape frontier the highest node top is 1350 m while the
    /// matter line is at 1200 m. Releasing at the line therefore starts the projectile
    /// 150 m INSIDE a cell, and a penalty contact resolving that initial condition
    /// produces a force with no physical meaning — the version of this that shipped for
    /// about an hour reported 4.0e8 N on a 0.5 mm grain.
    ///
    /// The throw is delivered AT CONTACT rather than from across the domain. A grain
    /// contact is stiff enough that one frame advances a fraction of a millisecond of
    /// physics, so a projectile released from the far wall is still in flight after a
    /// minute of watching — correct, and indistinguishable from broken. The flight is
    /// not what the certificate is about; the impact is.
    fn launch(&mut self, focus: [f64; 2], speed_fraction: f64, finest_m: f64) {
        let domain = self.tier.domain_m;
        // The projectile is scaled to the TIER — the point of throwing at a zoom level —
        // and the scale is the GEOMETRIC MEAN of the tier's grain and its domain:
        // halfway between the smallest thing that exists here and the whole scene, on
        // the log axis the zoom itself moves along. That gives a 3.5 cm ball in the
        // sandbox, a 45 um particle at the grain, and a 9 m boulder on the landscape.
        //
        // A fixed fraction of the domain does not survive the ladder. At 4% of 2 km the
        // projectile is 160 m across and weighs 5.7 million tonnes, and a throw that
        // heavy is not stopped by anything the scene contains — it registered a 3.3e11 N
        // contact and carried on at its release speed, which is correct arithmetic about
        // an absurd object.
        let radius = (0.5 * (self.tier.g0_m * domain).sqrt()).max(finest_m);
        let density = self
            .tier
            .material
            .map(|material| material.density_kg_m3)
            .unwrap_or(2650.0);
        let mass = core::f64::consts::PI / 6.0 * (2.0 * radius).powi(3) * density;
        let speed = impact_speed(&self.tier, speed_fraction);

        let x = focus[0].clamp(radius, domain - radius);
        // The height at which the projectile would FIRST touch something, descending
        // along this x. Not the highest node top under the aim: a node at the edge of
        // the x-window contributes its full top while contacting it would need the
        // projectile to descend nearly a whole radius further, and releasing at that
        // optimistic height starts the throw already inside nothing and then never
        // arrives. `y_touch = node.y + sqrt(touching^2 - dx^2)` is the exact answer and
        // costs one square root per node, once.
        // Start from nothing, not from the matter line. The line is where matter starts
        // in the LEDGER; the resident nodes are a coarse partition of it and their
        // topmost contact height can be well below it. Taking the max of the two puts
        // the projectile above every node in the scene and it simply falls, which is
        // what the landscape tier did: released at 1200 m with the nearest contact
        // below 1199, it was still descending after sixty frames with a certificate
        // saying everything was fine.
        let mut surface = f64::NEG_INFINITY;
        for i in 0..self.nodes.len() {
            let touching = radius + self.nodes.radius_m[i];
            let dx = self.nodes.position[i][0] - x;
            let span = touching * touching - dx * dx;
            if span <= 0.0 {
                continue;
            }
            surface = surface.max(self.nodes.position[i][1] + span.sqrt());
        }
        if !surface.is_finite() {
            surface = self.tier.fill * domain;
        }
        // `surface` is already the contact height, so the projectile's own radius is
        // not added again.
        let radius_included = 0.0;
        // The gap is TWO FRAMES OF FLIGHT, computed from the step this scene will
        // actually take. A fixed gap in metres or in radii cannot work across ten
        // decades of tier: at the sandbox it would be a stall, and at the landscape —
        // where one frame advances 0.2 ms of physics and the projectile moves 18 mm —
        // an 84 m gap is 220 frames of watching nothing. Two frames is a gap by
        // construction, at every tier.
        // Two frames of flight at a REPRESENTATIVE substep count. Deriving it from the
        // awake set does not work here: nothing is awake at release, so the budget
        // divides by one and the gap comes out most of a domain wide — the throw then
        // spends eight frames falling through empty air before it touches anything.
        let flight = speed * self.dt_s * RELEASE_SUBSTEPS as f64 * 2.0;
        let y = (surface + radius_included + flight).min(domain * 4.0);

        self.projectile = Projectile {
            position: [x, y],
            // A shallow lateral component, so it reads as a throw and not a drop.
            velocity: [0.25 * speed, -speed],
            radius_m: radius,
            mass_kg: mass,
            live: true,
        };
    }

    /// Advance the certified frontier by up to `elapsed_s` of simulated time.
    ///
    /// Frames never certify. This steps the frontier the throw event already certified,
    /// which is what keeps a 60 fps loop and an event-scoped certificate compatible.
    pub fn step(&mut self, elapsed_s: f64) {
        // A gravity tier has no frontier to step: it has ONE body on a geodesic of a
        // declared chart, and the chart is what carries the physics.
        if let Evaluator::GeodesicChart { .. } = self.tier.evaluator {
            self.step_geodesic();
            self.slow_motion = 0.0;
            return;
        }
        if self.dt_s <= 0.0 || self.nodes.is_empty() {
            self.slow_motion = 0.0;
            return;
        }
        self.refresh_awake();
        // Nothing awake and nothing in flight is nothing to do. Without this the budget
        // divides by one, asks for twenty thousand substeps, and spends 1.4 SECONDS of
        // per-substep overhead advancing a scene in which nothing can move.
        if self.awake_list.is_empty() && !self.projectile.live {
            self.substeps = 0;
            self.slow_motion = 0.0;
            return;
        }
        // Once a frame, not once a substep: cells can only have moved by what the last
        // frame stepped them.
        self.refresh_pairs();
        let budget = elapsed_s.clamp(0.0, 1.0 / 20.0);
        // Substeps are budgeted by the AWAKE set, not the resident one, and capped
        // absolutely: a budget expressed per node-substep has no sensible answer when
        // almost nothing is awake, and a substep costs something even when it does no
        // physics.
        let per_frame = (self.work_budget / self.awake_list.len().max(1))
            .clamp(1, MAX_SUBSTEPS_PER_FRAME);
        let wanted = (budget / self.dt_s).floor() as usize;
        let limit = wanted.min(per_frame);
        // Spend the budget as it is consumed, not as it was predicted. The awake set is
        // near zero at the moment of impact and thousands of cells a few substeps later,
        // so a per-frame count computed up front commits to work the frame cannot
        // afford: the grain tier's impact frame ran 4,000 substeps over 2,000 freshly
        // woken cells and took 678 ms. Counting node-substeps as they happen bounds the
        // frame whatever wakes up inside it.
        let mut spent = 0_usize;
        let mut steps = 0_usize;
        while steps < limit && spent <= self.work_budget {
            self.substep(self.dt_s);
            spent += self.awake_list.len().max(1);
            steps += 1;
        }
        self.substeps = steps;
        // What the viewer is actually watching, measured rather than claimed.
        self.slow_motion = if budget > 0.0 {
            steps as f64 * self.dt_s / budget
        } else {
            0.0
        };
    }

    /// Advance one fixed step.
    ///
    /// Everything here iterates a WORKING SET, never the resident set. That distinction
    /// is the whole reason an acuity-pinned scene is affordable: 118,296 cells resident
    /// and a few hundred awake. Skipping asleep cells inside loops over all of them is
    /// not enough — at this size the loop is the cost even when every iteration does
    /// nothing, and a sleeping scene measured ten seconds a frame that way.
    fn substep(&mut self, dt: f64) {
        // Per-substep dissipation, accumulated into locals and folded into the ledger at
        // the end so the borrow of `self` inside the force loops stays immutable.
        let mut damping_j = 0.0_f64;
        let mut contact_friction_j = 0.0_f64;
        let mut bond_friction_j = 0.0_f64;
        let mut wall_j = 0.0_f64;
        let mut sleep_j = 0.0_f64;
        let mut anchor_j = 0.0_f64;
        let mut sleep_nats = 0.0_f64;
        let mut anchor_nats = 0.0_f64;
        let mut wall_nats = 0.0_f64;
        let mut sleep_events = 0_u64;
        let mut anchor_events = 0_u64;
        let mut wall_events = 0_u64;

        let mut wake = core::mem::take(&mut self.wake);
        wake.clear();
        let wake_speed = WAKE_SPEED_FRACTION * self.impact_speed_m_s.max(1.0e-12);

        // Zero only what this substep will write, from the same sets that write it.
        if self.force.len() != self.nodes.len() {
            self.force.clear();
            self.force.resize(self.nodes.len(), [0.0; 2]);
        }
        let mut force = core::mem::take(&mut self.force);
        for i in self.awake_list.iter().copied() {
            force[i] = [0.0; 2];
        }
        for (i, j) in self.active_pairs.iter().copied() {
            force[i] = [0.0; 2];
            force[j] = [0.0; 2];
        }
        for index in self.active_bonds.iter().copied() {
            let [a, b] = self.relations.ends[index];
            force[a] = [0.0; 2];
            force[b] = [0.0; 2];
        }

        for i in self.awake_list.iter().copied() {
            if !self.nodes.anchored[i] {
                force[i][1] -= self.nodes.mass_kg[i] * CHART_GRAVITY_M_S2;
            }
        }

        // Live relation holons own their pairs, including while closed.
        let bonds = core::mem::take(&mut self.active_bonds);
        for index in bonds.iter().copied() {
            let [a, b] = self.relations.ends[index];
            let bond = &mut self.relations.bonds[index];
            if bond.is_broken() {
                continue;
            }
            let delta = [
                self.nodes.position[b][0] - self.nodes.position[a][0],
                self.nodes.position[b][1] - self.nodes.position[a][1],
            ];
            let length = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt();
            if length <= 0.0 {
                continue;
            }
            let normal = [delta[0] / length, delta[1] / length];
            let extension = length - bond.rest_length_m;
            let relative = [
                self.nodes.velocity[b][0] - self.nodes.velocity[a][0],
                self.nodes.velocity[b][1] - self.nodes.velocity[a][1],
            ];
            let closing = relative[0] * normal[0] + relative[1] * normal[1];
            let axial = bond.axial_force(extension, closing);
            force[a][0] += axial * normal[0];
            force[a][1] += axial * normal[1];
            force[b][0] -= axial * normal[0];
            force[b][1] -= axial * normal[1];
            if closing.abs() > wake_speed {
                wake.push(a);
                wake.push(b);
            }

            // Tangential channel: the closed-interface slider, capped at D*mu*|F_n|.
            let tangent = [-normal[1], normal[0]];
            let sliding = relative[0] * tangent[0] + relative[1] * tangent[1];
            let friction = bond.closed_friction_force(axial, sliding);
            if friction > 0.0 {
                bond_friction_j += friction * sliding.abs() * dt;
                let sign = if sliding > 0.0 { 1.0 } else { -1.0 };
                force[a][0] += sign * friction * tangent[0];
                force[a][1] += sign * friction * tangent[1];
                force[b][0] -= sign * friction * tangent[0];
                force[b][1] -= sign * friction * tangent[1];
            }
        }
        self.active_bonds = bonds;

        // Contact on pairs no live relation owns — the jurisdiction rule, executable.
        if self.contact_stiffness_n_m > 0.0 {
            let pairs = core::mem::take(&mut self.active_pairs);
            for (i, j) in pairs.iter().copied() {
                // Reject on the SQUARED distance, before any square root and before
                // asking who owns the pair. Most candidate pairs are not touching, and
                // on this path a `sqrt` cost more than the contact it was rejecting.
                let delta = [
                    self.nodes.position[j][0] - self.nodes.position[i][0],
                    self.nodes.position[j][1] - self.nodes.position[i][1],
                ];
                let square = delta[0] * delta[0] + delta[1] * delta[1];
                let touching = self.rest_gap(i, j);
                if square >= touching * touching || square <= 0.0 {
                    continue;
                }
                if self.relations.owns(i, j) {
                    continue;
                }
                let distance = square.sqrt();
                let normal = [delta[0] / distance, delta[1] / distance];
                let overlap = touching - distance;
                let relative = [
                    self.nodes.velocity[j][0] - self.nodes.velocity[i][0],
                    self.nodes.velocity[j][1] - self.nodes.velocity[i][1],
                ];
                let closing = relative[0] * normal[0] + relative[1] * normal[1];
                let push = (self.contact_stiffness_n_m * overlap
                    - self.contact_damping_n_s_m * closing)
                    .max(0.0);
                if push > 0.0 {
                    // Damping opposes the closing speed, so it dissipates
                    // `c * closing^2` of power for as long as the contact is live. This is
                    // the channel that IS restitution (A5); there is no second one.
                    damping_j += self.contact_damping_n_s_m * closing * closing * dt;
                }
                force[i][0] -= push * normal[0];
                force[i][1] -= push * normal[1];
                force[j][0] += push * normal[0];
                force[j][1] += push * normal[1];
                if -closing > wake_speed {
                    wake.push(i);
                    wake.push(j);
                }

                let tangent = [-normal[1], normal[0]];
                let sliding = relative[0] * tangent[0] + relative[1] * tangent[1];
                let capacity = GRAIN_FRICTION_MU * push;
                let viscous = self.contact_damping_n_s_m * sliding.abs();
                let friction = viscous.min(capacity);
                if friction > 0.0 {
                    let sign = if sliding > 0.0 { 1.0 } else { -1.0 };
                    force[i][0] += sign * friction * tangent[0];
                    force[i][1] += sign * friction * tangent[1];
                    force[j][0] -= sign * friction * tangent[0];
                    force[j][1] -= sign * friction * tangent[1];
                    contact_friction_j += friction * sliding.abs() * dt;
                }
            }
            self.active_pairs = pairs;
        }

        // The projectile against the frontier. It is tested against resident cells in
        // its own neighbourhood, ASLEEP ONES INCLUDED — it is what does the waking.
        let mut projectile_force = [0.0_f64; 2];
        if self.projectile.live && self.contact_stiffness_n_m > 0.0 {
            self.refresh_near();
            let near = core::mem::take(&mut self.near);
            for i in near.iter().copied() {
                let delta = [
                    self.nodes.position[i][0] - self.projectile.position[0],
                    self.nodes.position[i][1] - self.projectile.position[1],
                ];
                let square = delta[0] * delta[0] + delta[1] * delta[1];
                let touching = self.nodes.radius_m[i] + self.projectile.radius_m;
                if square >= touching * touching || square <= 0.0 {
                    continue;
                }
                let distance = square.sqrt();
                let normal = [delta[0] / distance, delta[1] / distance];
                let overlap = touching - distance;
                let relative = [
                    self.nodes.velocity[i][0] - self.projectile.velocity[0],
                    self.nodes.velocity[i][1] - self.projectile.velocity[1],
                ];
                let closing = relative[0] * normal[0] + relative[1] * normal[1];
                let push = (self.contact_stiffness_n_m * overlap
                    - self.contact_damping_n_s_m * closing)
                    .max(0.0);
                // The SAME channel as the grain-grain loop above — contact damping, which
                // IS restitution (A5) — at the contact that does most of the work in a
                // throw. It was in the force and not in the ledger, and at the landscape
                // tier that one omission was 4.82e7 J: 99% of the residual the first run
                // reported as 76% unexplained dissipation.
                if push > 0.0 {
                    damping_j += self.contact_damping_n_s_m * closing * closing * dt;
                }
                if !self.nodes.awake[i] {
                    force[i] = [0.0; 2];
                }
                force[i][0] += push * normal[0];
                force[i][1] += push * normal[1];
                projectile_force[0] -= push * normal[0];
                projectile_force[1] -= push * normal[1];
                self.peak_contact_n = self.peak_contact_n.max(push);
                // Anything the throw touches is awake, whatever the force: this is the
                // disturbance, and it is why the scene has an awake set at all.
                wake.push(i);
                if self.impulse_mode == ImpulseMode::SumOfMagnitudes {
                    self.impulse_n_s += push * dt;
                }
            }
            self.near = near;
        }

        let domain = self.tier.domain_m;
        let sleep_speed = self.sleep_speed_fraction * self.impact_speed_m_s.max(1.0e-12);
        let awake = core::mem::take(&mut self.awake_list);
        for i in awake.iter().copied() {
            if self.nodes.anchored[i] {
                let v = self.nodes.velocity[i];
                anchor_j += 0.5 * self.nodes.mass_kg[i] * (v[0] * v[0] + v[1] * v[1]);
                let collapsed = nats_collapsed(v[0]) + nats_collapsed(v[1]);
                if collapsed > 0.0 {
                    anchor_nats += collapsed;
                    anchor_events += 1;
                }
                self.nodes.velocity[i] = [0.0, 0.0];
                continue;
            }
            let inverse = 1.0 / self.nodes.mass_kg[i];
            self.nodes.velocity[i][0] += force[i][0] * inverse * dt;
            self.nodes.velocity[i][1] += force[i][1] * inverse * dt;
            self.nodes.position[i][0] += self.nodes.velocity[i][0] * dt;
            self.nodes.position[i][1] += self.nodes.velocity[i][1] * dt;

            // The box has a bottom and two sides. They take momentum out of the scene
            // and the readout does not claim otherwise.
            let radius = self.nodes.radius_m[i];
            // Each reflection keeps `e` of the normal speed, so it takes `1 - e^2` of that
            // component's kinetic energy out of the scene. The walls were not in the
            // channel list and are a real sink.
            let wall_loss = 0.5 * self.nodes.mass_kg[i]
                * (1.0 - CONTACT_RESTITUTION * CONTACT_RESTITUTION);
            if self.nodes.position[i][1] < radius {
                wall_nats += nats_collapsed(radius - self.nodes.position[i][1]);
                wall_events += 1;
                self.nodes.position[i][1] = radius;
                let vn = self.nodes.velocity[i][1];
                wall_j += wall_loss * vn * vn;
                self.nodes.velocity[i][1] = -vn * CONTACT_RESTITUTION;
            }
            if self.nodes.position[i][0] < radius {
                wall_nats += nats_collapsed(radius - self.nodes.position[i][0]);
                wall_events += 1;
                self.nodes.position[i][0] = radius;
                let vn = self.nodes.velocity[i][0];
                wall_j += wall_loss * vn * vn;
                self.nodes.velocity[i][0] = -vn * CONTACT_RESTITUTION;
            } else if self.nodes.position[i][0] > domain - radius {
                wall_nats += nats_collapsed(self.nodes.position[i][0] - (domain - radius));
                wall_events += 1;
                self.nodes.position[i][0] = domain - radius;
                let vn = self.nodes.velocity[i][0];
                wall_j += wall_loss * vn * vn;
                self.nodes.velocity[i][0] = -vn * CONTACT_RESTITUTION;
            }

            // A cell slow for long enough goes back to sleep, and its velocity is zeroed
            // rather than left as a residue that would wake it again next substep.
            let speed = (self.nodes.velocity[i][0] * self.nodes.velocity[i][0]
                + self.nodes.velocity[i][1] * self.nodes.velocity[i][1])
                .sqrt();
            if speed < sleep_speed {
                self.nodes.still[i] = self.nodes.still[i].saturating_add(1);
                if self.nodes.still[i] >= SLEEP_SUBSTEPS {
                    self.nodes.awake[i] = false;
                    let v = self.nodes.velocity[i];
                    sleep_j += 0.5 * self.nodes.mass_kg[i] * (v[0] * v[0] + v[1] * v[1]);
                    let collapsed = nats_collapsed(v[0]) + nats_collapsed(v[1]);
                    if collapsed > 0.0 {
                        sleep_nats += collapsed;
                        sleep_events += 1;
                    }
                    self.nodes.velocity[i] = [0.0, 0.0];
                    self.sleep_generation = self.sleep_generation.wrapping_add(1);
                    self.awake_dirty = true;
                }
            } else {
                self.nodes.still[i] = 0;
            }
        }
        self.awake_list = awake;

        for index in wake.drain(..) {
            if !self.nodes.awake[index] {
                self.awake_dirty = true;
                self.nodes.wake(index);
                self.woke.push(index);
            }
        }
        self.wake = wake;
        self.refresh_awake();

        self.ledger.contact_damping_j += damping_j;
        self.ledger.contact_friction_j += contact_friction_j;
        self.ledger.bond_friction_j += bond_friction_j;
        self.ledger.wall_restitution_j += wall_j;
        self.ledger.sleep_absorbed_j += sleep_j;
        self.ledger.anchor_absorbed_j += anchor_j;
        self.merge_ledger.sleep_nats += sleep_nats;
        self.merge_ledger.anchor_nats += anchor_nats;
        self.merge_ledger.wall_clamp_nats += wall_nats;
        self.merge_ledger.sleep_events += sleep_events;
        self.merge_ledger.anchor_events += anchor_events;
        self.merge_ledger.wall_clamp_events += wall_events;

        // The contact impulse is the magnitude of the NET force on the projectile,
        // integrated — not the sum of the individual contact magnitudes. See
        // `tests::the_contact_impulse_is_the_momentum_the_projectile_gave_up`.
        if self.projectile.live && self.impulse_mode == ImpulseMode::Net {
            let jx = projectile_force[0] * dt;
            let jy = projectile_force[1] * dt;
            self.impulse_vec[0] += jx;
            self.impulse_vec[1] += jy;
            self.impulse_n_s += (jx * jx + jy * jy).sqrt();
        }

        // The disturbance the throw caused: how far any resident cell has been moved
        // from where the certified frontier put it. Only awake cells can have moved.
        for i in self.awake_list.iter().copied() {
            let dx = self.nodes.position[i][0] - self.rest[i][0];
            let dy = self.nodes.position[i][1] - self.rest[i][1];
            self.disturbance_m = self.disturbance_m.max((dx * dx + dy * dy).sqrt());
        }

        self.force = force;

        if self.projectile.live {
            self.projectile.velocity[0] += projectile_force[0] / self.projectile.mass_kg * dt;
            self.projectile.velocity[1] += projectile_force[1] / self.projectile.mass_kg * dt;
            self.projectile.velocity[1] -= CHART_GRAVITY_M_S2 * dt;
            self.projectile.position[0] += self.projectile.velocity[0] * dt;
            self.projectile.position[1] += self.projectile.velocity[1] * dt;
            if self.projectile.position[1] < self.projectile.radius_m {
                self.projectile.position[1] = self.projectile.radius_m;
                // Same sink as a cell's floor bounce and the same channel. It reads exactly
                // zero at all three tiers on the gate's throw — the projectile buries itself
                // in the sand or leaves the domain before it ever reaches the floor — so it
                // is completeness rather than a term that moves any number here.
                let vn = self.projectile.velocity[1];
                self.ledger.wall_restitution_j += 0.5
                    * self.projectile.mass_kg
                    * (1.0 - CONTACT_RESTITUTION * CONTACT_RESTITUTION)
                    * vn
                    * vn;
                self.projectile.velocity[1] = -vn * CONTACT_RESTITUTION;
            }
            if self.projectile.position[0] > domain + self.projectile.radius_m {
                self.projectile.live = false;
            }
        }

        self.time_s += dt;
    }

    /// The separation at which pair `(i, j)` begins to resist: geometric touching, or
    /// their separation on the certified frontier if that is closer.
    fn rest_gap(&self, i: usize, j: usize) -> f64 {
        let touching = self.nodes.radius_m[i] + self.nodes.radius_m[j];
        if self.rest.len() != self.nodes.len() {
            return touching;
        }
        let dx = self.rest[i][0] - self.rest[j][0];
        let dy = self.rest[i][1] - self.rest[j][1];
        touching.min((dx * dx + dy * dy).sqrt())
    }

    /// Rebuild the candidate-pair list when something has moved far enough that the
    /// last one could have gone stale.
    ///
    /// The list is built with a MARGIN, so a pair that is not on it cannot come into
    /// contact until some node has travelled half that margin. Checking the largest
    /// displacement against that bound is the standard Verlet-list criterion — a CHECK
    /// rather than a rebuild interval chosen by feel, so no contact can be missed.
    ///
    /// Only AWAKE cells are checked, because only awake cells move. That is what makes
    /// this affordable to run once a frame over a scene of 118,296 resident cells: the
    /// scan is over the hundreds that are moving, not the hundred thousand that are not.
    ///
    /// This nearly went missing. Restructuring the substep around working sets left it
    /// orphaned — the compiler reported it as dead code, which read like tidy-up and was
    /// actually a correctness hole: the pair list would have been built once per throw
    /// and never refreshed, so contacts formed by cells that had moved since would
    /// simply not exist. A warning about an unused method was the only sign.
    fn refresh_pairs(&mut self) {
        let count = self.nodes.len();
        let rebuild = if self.anchor.len() != count {
            true
        } else {
            let mut worst = 0.0_f64;
            for i in self.awake_list.iter().copied() {
                let dx = self.nodes.position[i][0] - self.anchor[i][0];
                let dy = self.nodes.position[i][1] - self.anchor[i][1];
                worst = worst.max(dx * dx + dy * dy);
            }
            worst.sqrt() * 2.0 >= self.margin_m
        };
        if !rebuild {
            return;
        }

        self.anchor.clear();
        self.anchor.extend_from_slice(&self.nodes.position);
        // In METRES. `RELATION_REACH` is a dimensionless multiple of touching, and
        // folding it in here made the margin 0.675 m — on a 0.5 mm domain, which
        // collapsed the grid to a single bucket, turned the broadphase into an all-pairs
        // sweep over 65,539 nodes, and got the test binary killed by the OOM reaper. One
        // radius of slack is the right amount and the right unit.
        self.margin_m = self
            .nodes
            .radius_m
            .iter()
            .fold(0.0_f64, |best, radius| best.max(*radius));
        self.broadphase
            .rebuild(&self.nodes, self.tier.domain_m, self.margin_m);
        let mut pairs = core::mem::take(&mut self.pairs);
        self.broadphase.pairs(&mut pairs);
        self.pairs = pairs;
        // The pair set changed, so everything derived from it has to be rebuilt too.
        self.pair_adjacency.clear();
        self.awake_dirty = true;
        self.refresh_awake();
    }

    /// What the certificate would read out for the frontier as it stands.
    pub fn readout(&self) -> Settled<OBSERVABLES> {
        Settled {
            observables: [
                // The certificate carries the CONSERVED quantity: momentum actually
                // transferred, not total pushing done.
                self.net_impulse_n_s(),
                self.disturbance_m,
                self.relations.cracked() as f64,
            ],
            conservation_residual: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sandbox throw must certify, materialize a bounded number of holons, and do it
    /// inside the declared event budget. This is the demo's flagship interaction and the
    /// budget is the one written down before any of it was built.
    ///
    /// 2026-08-24: the budget check used to assert wall-clock `elapsed < 0.5` s, which is
    /// exactly the thing `certify_at`'s own doc comment says the engine "does not pretend
    /// to" hold itself to — certification's actual budget is a ceiling on ROUNDS and
    /// HOLONS (that is why `std::time::Instant` isn't even available on
    /// `wasm32-unknown-unknown`). The wall-clock version failed once under concurrent CI
    /// load and passed the same tree moments later under a release build with less
    /// contention — a gate whose green depends on how busy the box is that moment is the
    /// same failure shape as a comment claiming coverage a command doesn't deliver: it
    /// cannot tell "the work regressed" apart from "the box was busy". `rounds()` is
    /// deterministic for a fixed throw (measured: identical across three separate runs)
    /// and IS the quantity the budget is actually about, so it replaces the timing
    /// assertion rather than loosening it. The bound (a quarter of [`MAX_ROUNDS`]) is
    /// generous relative to this scene's measured 39,636 rounds (~10% of the ceiling) —
    /// room for legitimate algorithmic drift without going red on it, while still
    /// catching a real blow-up. Wall time is still measured and reported on failure, per
    /// the "host times this call and reports what it cost" contract; it is no longer
    /// asserted on.
    #[test]
    fn a_sandbox_throw_certifies_within_the_declared_budget() {
        let mut session = Session::new(TierId::Sandbox);
        let started = std::time::Instant::now();
        session.throw(0.5, 0.4, 0.6);
        let elapsed = started.elapsed().as_secs_f64();
        assert_eq!(
            session.verdict(),
            Verdict::Certified,
            "sandbox throw verdict: {:?}",
            session.verdict()
        );
        assert!(
            session.arena().len() <= session.holon_ceiling(),
            "resident holons {} exceeded the declared ceiling {}",
            session.arena().len(),
            session.holon_ceiling()
        );
        let round_ceiling = MAX_ROUNDS / 4;
        assert!(
            session.rounds() <= round_ceiling,
            "a throw event spent {} of {} declared rounds (ceiling for this test: {round_ceiling}) \
             — a work regression, not a load artifact: this call took {elapsed:.3} s to get there, \
             but rounds are deterministic and load-independent, unlike wall-clock time",
            session.rounds(),
            MAX_ROUNDS,
        );
        assert!(!session.nodes().is_empty(), "the throw resolved no matter");
        session.arena().validate().expect("ledger still composes");
    }

    /// Sand's relations are never-bonded, and that is derived rather than declared.
    #[test]
    fn the_sandbox_frontier_has_no_cohesive_relations() {
        let mut session = Session::new(TierId::Sandbox);
        session.throw(0.5, 0.4, 0.6);
        assert_eq!(session.law_refusal(), Some(LawRefusal::UnderResolved));
        assert!(session.relations().is_empty());
    }

    /// Stepping must not blow up, and the ledger must survive the whole event.
    #[test]
    fn stepping_a_certified_sandbox_stays_finite() {
        let mut session = Session::new(TierId::Sandbox);
        session.throw(0.5, 0.4, 0.6);
        for _ in 0..120 {
            session.step(1.0 / 60.0);
        }
        for position in &session.nodes().position {
            assert!(
                position[0].is_finite() && position[1].is_finite(),
                "a node left the reals: {position:?}"
            );
        }
        assert!(session.impulse_n_s().is_finite());
        session.arena().validate().expect("ledger still composes");
    }

    /// The contact impulse must be the momentum the projectile actually gave up.
    ///
    /// This gate did not exist, and its absence is why a landscape throw reported ten
    /// times the projectile's momentum for as long as it did: the ENERGY gate passed
    /// throughout, because nothing was gaining energy. The reported figure was summing
    /// the SIZES of every simultaneous contact force instead of their vector sum, and a
    /// projectile resting against several cells at once is pushed by each of them in
    /// directions that largely cancel. The overstatement therefore scaled with how many
    /// cells were touched at once — 1.9x at the grain tier, 1.2x at the sandbox, 5.3x on
    /// the landscape's coarse frontier — which is the tell that it was about contact
    /// COUNT and not about any tier's physics.
    ///
    /// Energy and momentum are independent conservation laws and a solver needs a gate
    /// on each. This is the missing one.
    #[test]
    fn the_contact_impulse_is_the_momentum_the_projectile_gave_up() {
        for id in [TierId::Grain, TierId::Sandbox, TierId::Landscape] {
            let mut session = Session::new(id);
            session.throw(0.5, 0.4, 0.6);
            let mass = session.projectile().mass_kg;
            let before = session.projectile().velocity;
            for _ in 0..240 {
                session.step(1.0 / 60.0);
            }
            let after = session.projectile().velocity;
            // Gravity acted on the projectile for the whole simulated time; the contact
            // is responsible for the rest.
            let dvx = after[0] - before[0];
            let dvy = after[1] - before[1] + CHART_GRAVITY_M_S2 * session.time_s();
            let expected = mass * (dvx * dvx + dvy * dvy).sqrt();
            let net = session.net_impulse_n_s();
            assert!(
                expected > 0.0,
                "{id:?}: nothing was transferred, so this gate proves nothing"
            );
            let error = (net - expected).abs() / expected;
            assert!(
                error < 1.0e-6,
                "{id:?}: reported a net contact impulse of {net:e} N*s while the \
                 projectile's momentum changed by {expected:e} N*s ({error:.3e} relative)"
            );

            // And the total pushing is never LESS than the momentum transferred: it is
            // the same integral without the vector cancellation, so it bounds it above.
            assert!(
                session.impulse_n_s() >= net * (1.0 - 1.0e-9),
                "{id:?}: total impulse {:e} is below the net {net:e}, which is \
                 arithmetically impossible",
                session.impulse_n_s()
            );
        }
    }

    /// The momentum gate must CATCH the accounting it replaced.
    ///
    /// Two things about this mutant are worth recording, because both were wrong first.
    ///
    /// The plant has to corrupt the CERTIFICATE's number. The first version left the
    /// conserved vector correct and only inflated a secondary field, so the gate had
    /// nothing to catch and read 1.01x — a mutant that cannot be caught is not evidence
    /// that the gate works, it is evidence that the plant missed.
    ///
    /// And the bug's SEVERITY collapsed when the scene was pinned at observer acuity.
    /// It overstated by 5.3x on the landscape when that tier's frontier was 200 coarse
    /// cells and one projectile touched many at once. At acuity the landscape's cells
    /// are 7.8 m across against a 4.5 m projectile, so it touches one at a time and the
    /// mutant is INERT there; at the sandbox it is 0.6%. The gate's tolerance is 1e-6,
    /// so 0.6% is still caught by four orders of magnitude — but a test asserting
    /// "overstates severalfold" would now fail for a reason that has nothing to do with
    /// the accounting being right.
    #[test]
    fn the_momentum_gate_catches_the_accounting_it_replaced() {
        let measure = |id: TierId, mode: ImpulseMode| {
            let mut session = Session::new(id);
            session.set_impulse_mode(mode);
            session.throw(0.5, 0.4, 0.6);
            let mass = session.projectile().mass_kg;
            let before = session.projectile().velocity;
            for _ in 0..240 {
                session.step(1.0 / 60.0);
            }
            let after = session.projectile().velocity;
            let dvx = after[0] - before[0];
            let dvy = after[1] - before[1] + CHART_GRAVITY_M_S2 * session.time_s();
            let expected = mass * (dvx * dvx + dvy * dvy).sqrt();
            assert!(expected > 0.0, "{id:?}: nothing was transferred");
            session.net_impulse_n_s() / expected
        };

        // Every tier that runs must conserve momentum exactly under the real accounting.
        for id in [TierId::Grain, TierId::Sandbox, TierId::Landscape] {
            let clean = measure(id, ImpulseMode::Net);
            assert!(
                (clean - 1.0).abs() < 1.0e-6,
                "{id:?}: the unmutated accounting must conserve momentum, got {clean}"
            );
        }

        // And on the tiers where a projectile spans several cells, the mutant must be
        // caught — by a margin the gate can see, not by a margin that looks impressive.
        for id in [TierId::Grain, TierId::Sandbox] {
            let mutant = measure(id, ImpulseMode::SumOfMagnitudes);
            assert!(
                (mutant - 1.0).abs() > 1.0e-3,
                "{id:?}: the mutant read {mutant} against a gate tolerance of 1e-6; it \
                 is not being caught with any margin"
            );
        }
    }

    /// The drawn path is no coarser than the observer can see.
    ///
    /// "Never coarsen" binds on everything SHOWN, and a trajectory is shown. A polyline
    /// is exactly as fine as its samples — the segment between two computed points is
    /// interpolation — so the sample spacing is the thing to check, and it was not
    /// trivially satisfied: sampling once a frame put the planet's points 3.6x acuity
    /// apart and the galactic ones 1.6x. This is why the trail records per RK4 step.
    #[test]
    fn the_drawn_path_is_no_coarser_than_the_observer_can_see() {
        for id in [TierId::Planet, TierId::Galactic] {
            for index in 0..crate::gravity::scenes_for(id).len() {
                let mut session = Session::new(id);
                session.set_gravity_scene(index);
                session.throw(0.5, 0.4, 0.6);
                for _ in 0..40 {
                    session.step(1.0 / 60.0);
                }
                let scene = session.gravity_scene().unwrap();
                let acuity = scene.acuity_m();
                let trail = session.trail();
                if trail.len() < 2 {
                    continue;
                }
                let mut worst = 0.0_f64;
                for pair in trail.windows(2) {
                    let dx = pair[1][0] - pair[0][0];
                    let dy = pair[1][1] - pair[0][1];
                    worst = worst.max((dx * dx + dy * dy).sqrt());
                }
                assert!(
                    worst <= acuity,
                    "{id:?} scene {index} ({}): the path is sampled {worst:e} m apart \
                     against an acuity of {acuity:e} m — the line between samples is \
                     interpolation, and at that spacing a viewer can see it",
                    scene.name
                );
            }
        }
    }

    /// Sleeping is an APPROXIMATION, and this is what bounds it — measured at 0.94% on
    /// the impulse, declared as [`SLEEP_IMPULSE_TOLERANCE`].
    ///
    /// A sleeping cell is treated as fixed, so it can absorb momentum that a moving cell
    /// would have carried. That is only honest if it stays negligible, and the check is
    /// the one that matters: the projectile's momentum ledger must still close exactly
    /// with cells sleeping, because the projectile is never asleep and everything it
    /// touches is woken on contact.
    ///
    /// What sleeping may NOT do is change the answer. So the same throw is run with the
    /// sleep threshold at its shipped value and with sleeping effectively disabled, and
    /// the impulse must agree to within a tolerance far tighter than anything the
    /// certificate claims.
    #[test]
    fn sleeping_does_not_change_what_is_measured() {
        let impulse = |sleep: bool| {
            let mut session = Session::new(TierId::Sandbox);
            if !sleep {
                // Nothing can ever be still enough to sleep.
                session.set_sleep_speed_fraction(0.0);
            }
            session.throw(0.5, 0.4, 0.6);
            for _ in 0..120 {
                session.step(1.0 / 60.0);
            }
            (session.net_impulse_n_s(), session.awake_count())
        };

        let (sleeping, awake_when_sleeping) = impulse(true);
        let (never, awake_when_never) = impulse(false);
        assert!(
            awake_when_never > awake_when_sleeping,
            "disabling sleep left {awake_when_never} awake against {awake_when_sleeping} \
             — the control is not controlling anything"
        );
        let error = (sleeping - never).abs() / never.max(1.0e-30);
        assert!(
            error < SLEEP_IMPULSE_TOLERANCE,
            "sleeping changed the measured impulse by {error:.3e} ({sleeping:e} against \
             {never:e}), past the {SLEEP_IMPULSE_TOLERANCE} this demo declares. Tighten \
             the sleep criterion or publish a wider figure; do not widen the bound."
        );
    }

    /// A penalty contact that gains energy is the failure mode that produced every
    /// wrong number in this module's history: an exploding rest state, a projectile
    /// integrated inside its own force loop, a stability limit read off the wrong mass.
    /// None of them showed up as a crash — they showed up as a certificate saying
    /// everything was fine next to a ball leaving the box at fifty times its launch
    /// speed. So the gate is on the energy, and it runs at both tiers that step.
    #[test]
    fn a_throw_never_gains_energy() {
        for id in [TierId::Sandbox, TierId::Grain, TierId::Landscape] {
            let mut session = Session::new(id);
            session.throw(0.5, 0.4, 0.6);
            let launch = {
                let velocity = session.projectile().velocity;
                (velocity[0] * velocity[0] + velocity[1] * velocity[1]).sqrt()
            };
            assert!(launch > 0.0, "{id:?} launched nothing");
            let mut worst = 0.0_f64;
            for _ in 0..240 {
                session.step(1.0 / 60.0);
                let velocity = session.projectile().velocity;
                worst = worst
                    .max((velocity[0] * velocity[0] + velocity[1] * velocity[1]).sqrt());
                for node in &session.nodes().velocity {
                    assert!(
                        node[0].is_finite() && node[1].is_finite(),
                        "{id:?}: a node velocity left the reals"
                    );
                }
            }
            assert!(
                worst < 1.5 * launch,
                "{id:?}: the projectile reached {worst:.2} m/s having been launched at \
                 {launch:.2} m/s — the contact is injecting energy"
            );
        }
    }

    /// A throw must actually land, at every tier that steps, inside a few seconds of
    /// watching. A correct simulation of a projectile still in flight after a minute is
    /// indistinguishable from a broken one, and this demo had that bug twice.
    #[test]
    fn a_throw_lands() {
        for id in [TierId::Sandbox, TierId::Grain, TierId::Landscape] {
            let mut session = Session::new(id);
            session.throw(0.5, 0.4, 0.6);
            let mut landed = None;
            for frame in 0..180 {
                session.step(1.0 / 60.0);
                if session.impulse_n_s() > 0.0 {
                    landed = Some(frame);
                    break;
                }
            }
            assert!(
                landed.is_some_and(|frame| frame < 120),
                "{id:?}: nothing had been hit after 180 frames"
            );
        }
    }

    /// A scene is resident and certified the moment it exists, before anything is
    /// thrown at it.
    ///
    /// This is the gate on the bug Eric found: the browser canvas was blank until the
    /// first click, because nothing was certified until then and there was nothing
    /// resident to draw. A settled scene has a real, coarse frontier — and a real
    /// verdict, so zooming to a refusing tier shows the refusal immediately rather than
    /// looking idle until someone throws something to find out.
    #[test]
    fn a_scene_is_resident_before_anything_is_thrown() {
        for id in [TierId::Sandbox, TierId::Grain, TierId::Landscape] {
            let session = Session::new(id);
            assert!(
                !session.nodes().is_empty(),
                "{id:?} had nothing resident at rest — the canvas would be blank"
            );
            assert!(session.arena().len() > 1, "{id:?} materialized nothing at rest");
            assert_eq!(
                session.impulse_n_s(),
                0.0,
                "{id:?} claims an impulse with nothing thrown"
            );
            assert!(!session.projectile().live);
            session.arena().validate().expect("the rest ledger composes");
        }

        // A tier with no evaluator states that at rest, rather than reading idle.
        assert_eq!(Session::new(TierId::Crystal).verdict(), Verdict::NoEvaluator);
        // And a gravity tier states its SCREEN at rest: weight pulls here now, by a
        // certificate that is computed the moment the tier is selected.
        let planet = Session::new(TierId::Planet);
        assert_eq!(planet.verdict(), Verdict::Certified);
        assert!(planet.weak_field().is_some());
    }

    /// The observer's claim binds EVERYWHERE, and the physics claim refines further
    /// where the throw lands.
    ///
    /// This test has now been wrong twice, and both wrongs are worth keeping in view.
    /// First it asserted a throw makes MORE cells than the resting scene — it makes
    /// fewer, and that was the demand working. Then it asserted the resting scene sits
    /// at a coarse preview grain — it no longer does, because the observer is a claimant
    /// and their claim does not relax. What is true in the current frame is what is
    /// checked here: at rest the scene is exactly acuity-fine, and a throw is acuity-fine
    /// everywhere AND finer than that at the impact.
    #[test]
    fn the_observer_claim_binds_everywhere_and_the_throw_refines_further() {
        let coarsest = |session: &Session| {
            session
                .nodes()
                .radius_m
                .iter()
                .copied()
                .fold(0.0_f64, f64::max)
                * 2.0
        };
        let finest = |session: &Session| {
            session
                .nodes()
                .radius_m
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min)
                * 2.0
        };

        let settled = Session::new(TierId::Sandbox);
        let acuity = settled.tier.acuity_m();
        assert!(
            coarsest(&settled) <= acuity,
            "a resting cell is {:e} across against an acuity of {acuity:e}; the scene is \
             coarser than the observer can see, which is the one thing it may never be",
            coarsest(&settled)
        );

        let mut thrown = Session::new(TierId::Sandbox);
        thrown.throw(0.5, 0.4, 0.6);
        assert!(
            coarsest(&thrown) <= acuity,
            "a thrown-at cell is {:e} across against an acuity of {acuity:e}; the throw \
             coarsened part of the view",
            coarsest(&thrown)
        );
        assert!(
            finest(&thrown) < finest(&settled),
            "the throw resolved no finer than rest ({:e} against {:e}); the physics \
             claim is doing nothing",
            finest(&thrown),
            finest(&settled)
        );
    }

    /// Every tier that declares a refusal must actually refuse    /// Every tier that declares a refusal must actually refuse, and must say which
    /// refusal it is. A demo whose refusals are decoration would be worse than one with
    /// no refusals at all.
    #[test]
    fn refusing_tiers_refuse_with_their_own_reason() {
        let mut crystal = Session::new(TierId::Crystal);
        crystal.throw(0.5, 0.5, 0.6);
        assert_eq!(crystal.verdict(), Verdict::NoEvaluator);
        assert_eq!(crystal.arena().len(), 1, "a refused tier grows nothing");

        // The gravity tiers no longer refuse wholesale — they refuse PER SCENE, and
        // each tier declares one that certifies and one that says something else.
        for (id, scene, expected) in [
            (TierId::Planet, 0, Verdict::Certified),
            (TierId::Planet, 1, Verdict::GrainFloor),
            (TierId::Galactic, 0, Verdict::Certified),
            (TierId::Cosmic, 0, Verdict::Certified),
            (TierId::Cosmic, 1, Verdict::WeakFieldRefused),
        ] {
            let mut session = Session::new(id);
            session.set_gravity_scene(scene);
            session.throw(0.5, 0.5, 0.6);
            assert_eq!(
                session.verdict(),
                expected,
                "{id:?} scene {scene} ({})",
                session.gravity_scene().unwrap().name
            );
        }

        // And the one that refuses names WHICH refusal, with its unlock.
        let mut cosmic = Session::new(TierId::Cosmic);
        cosmic.set_gravity_scene(1);
        cosmic.throw(0.5, 0.5, 0.6);
        let refusal = cosmic.weak_field_refusal().expect("a typed refusal");
        assert_eq!(refusal, WeakFieldRefusal::ExpansionScale);
        assert!(refusal.unlock().contains("FRW"));
        assert!(!refusal.is_ceiling(), "the FRW family lifts this one");
    }


    /// Replay: the same throw twice must produce the identical arena and readout. The
    /// generator is seeded by geometry alone, so this is a check that nothing
    /// non-deterministic has crept into the event path.
    #[test]
    fn a_throw_replays_bit_identically() {
        let run = || {
            let mut session = Session::new(TierId::Sandbox);
            session.throw(0.37, 0.42, 0.55);
            for _ in 0..30 {
                session.step(1.0 / 60.0);
            }
            (
                session.arena().holons().to_vec(),
                session.readout().observables,
                session.nodes().position.clone(),
            )
        };
        let (holons_a, observables_a, positions_a) = run();
        let (holons_b, observables_b, positions_b) = run();
        assert_eq!(holons_a, holons_b);
        for k in 0..OBSERVABLES {
            assert_eq!(observables_a[k].to_bits(), observables_b[k].to_bits());
        }
        for (a, b) in positions_a.iter().zip(positions_b.iter()) {
            assert_eq!(a[0].to_bits(), b[0].to_bits());
            assert_eq!(a[1].to_bits(), b[1].to_bits());
        }
    }
}

#[cfg(test)]
mod dissipation_ledger {
    use super::*;

    fn flown(id: TierId) -> Session {
        let mut session = Session::new(id);
        session.throw(0.5, 0.4, 0.6);
        for _ in 0..240 {
            session.step(1.0 / 60.0);
        }
        session
    }

    // ------------------------------------------------------------------ instrument controls

    /// **MUST NOT FIRE.** A session that has not stepped has dissipated nothing. Without
    /// this, every closing balance below could be a balance of zeros.
    #[test]
    fn an_unstepped_session_has_an_empty_ledger() {
        let session = Session::new(TierId::Sandbox);
        let l = session.energy_ledger();
        assert_eq!(l.total_j(), 0.0, "ledger was not empty before any step: {l:?}");
        assert_eq!(session.energy_residual_j(), 0.0);
    }

    /// **MUST FIRE.** Stepping a sandbox throw must move the damping and friction channels
    /// off zero. A ledger that stayed at zero would report a perfect balance and mean
    /// nothing.
    #[test]
    fn stepping_moves_the_named_channels_off_zero() {
        let l = flown(TierId::Sandbox).energy_ledger();
        assert!(l.contact_damping_j > 0.0, "contact damping never fired: {l:?}");
        assert!(l.contact_friction_j > 0.0, "contact friction never fired: {l:?}");
        assert!(l.total_j() > 0.0);
    }

    /// The channels distinguish scenes rather than reporting a constant. An instrument that
    /// reported the same value everywhere would be measuring itself.
    ///
    /// **This control used to assert `grain.contact_damping_j == 0.0`, and that zero was the
    /// bug rather than the physics.** The grain tier's only contact is the projectile's, and
    /// the projectile's contact damping was the channel that was in the force and not in the
    /// ledger — so the control was resting its separation on precisely the omission the
    /// ledger existed to catch, and passed. It now reads 8.84e-15 J.
    ///
    /// Re-based on a zero that is a fact about the scene: the grain tier is two cells and a
    /// projectile, so nothing SLIDES and nothing reaches a wall. Contact friction and wall
    /// restitution are exactly zero there and both are real numbers at the sandbox, and the
    /// damping channel that both tiers do have differs by twelve orders of magnitude.
    #[test]
    fn the_channels_separate_one_scene_from_another() {
        let sandbox = flown(TierId::Sandbox).energy_ledger();
        let grain = flown(TierId::Grain).energy_ledger();
        assert!(sandbox.contact_friction_j > 0.0 && sandbox.wall_restitution_j > 0.0);
        assert_eq!(grain.contact_friction_j, 0.0, "the grain tier was not expected to slide");
        assert_eq!(grain.wall_restitution_j, 0.0, "the grain tier was not expected to reach a wall");
        assert!(sandbox.contact_damping_j > 0.0 && grain.contact_damping_j > 0.0);
        assert!(
            sandbox.contact_damping_j / grain.contact_damping_j > 1.0e9,
            "the shared channel should differ by orders of magnitude between these scenes: \
             {:e} vs {:e}",
            sandbox.contact_damping_j,
            grain.contact_damping_j
        );
    }

    // ------------------------------------------------------------------------ the finding

    /// **THE ENGINE DOES NOT CREATE ENERGY. THE LEDGER DID, AND THIS IS THE CLOSED
    /// BALANCE.**
    ///
    /// The version of this test that shipped with the ledger's first build asserted the
    /// opposite — that the sandbox scene GAINS 1.14 J of 107 (+1.06%) and that the landscape
    /// tier's named channels explain only 24% of its dissipation — and pinned that gap rather
    /// than closing it, on the ground that asserting a balance which does not hold would be
    /// tuning the instrument to the answer. That was the right call about the gap and the
    /// wrong diagnosis of it. It named two candidates, semi-implicit Euler and the
    /// `.max(0.0)` force clamp. **Neither was the cause and neither number was an engine
    /// defect.** Both were defects in this instrument:
    ///
    /// * the overlap-potential term measured from `radius[i] + radius[j]` where the contact
    ///   force measures from `rest_gap` — worth +1.1534 J of a +1.1389 J "gain", switched on
    ///   pair by pair as the throw woke the scene, since the term is summed over the pairs
    ///   adjacent to awake cells;
    /// * `contact_damping_j` charged at grain-grain contacts and not at the projectile's,
    ///   which at the landscape tier is 4.82e7 J — 99% of what was reported as unexplained.
    ///
    /// Both corrected. Measured over the same 240 frames after the same throw:
    ///
    /// | tier | dissipated | named channels | explained | residual |
    /// |---|---|---|---|---|
    /// | Sandbox | +1.3368e-2 J | 1.2514e-2 J | **93.6%** | +6.4% |
    /// | Grain | +9.2630e-15 J | 8.8380e-15 J | **95.4%** | +4.6% |
    /// | Landscape | +6.3847e7 J | 6.3336e7 J | **99.2%** | +0.8% |
    ///
    /// # THE WARRANT, corrected — this is Noether, and it is NOT `Core/Habit.lean`
    ///
    /// An earlier version of this test justified itself by citing
    /// `production_eq_zero_iff_rate_injective`: a stable step is injective, an injective step
    /// produces nothing, so energy creation is a fact the theory does not predict.
    /// **WITHDRAWN — that conflates energy with entropy.** Injectivity is about
    /// *information*; production is the log-degree of the rate map. **An injective map can
    /// create energy freely**: scale every velocity by 1.01 and the map is a bijection that
    /// manufactures joules. `Habit.lean` says nothing about this ledger's finding, and the
    /// programme's own rule — one ledger per quantity, dissipated energy and produced entropy
    /// being two quantities — is exactly what that citation broke. Kept here, marked dead,
    /// rather than quietly deleted.
    ///
    /// The correct warrant is ordinary physics and it is narrower. **At the tiers this gate
    /// runs on, the chart is flat, Newtonian, of fixed volume and fixed particle number, and
    /// its gravity is static.** That chart has time-translation symmetry, so Noether's
    /// theorem gives a conserved energy in the continuum equations, and a secular gain is a
    /// defect: integrator, unaccounted declared channel, or bug. Not a lake theorem. Which is
    /// also why the gate must ask [`Session::balance_applicability`] first — see
    /// [`BalanceApplicability`], and `the_gate_refuses_where_the_chart_has_no_conserved_energy`.
    ///
    /// # THE ARGUMENT for accounting gap over integrator error, rather than the conclusion
    ///
    /// The discriminator is the design's own: `residual / (E(0) − E(t))`, and what happens to
    /// it. Three independent legs, and they point the same way.
    ///
    /// 1. **The gain was never in the dynamics.** `total_energy_j` is a `&self` read called
    ///    in exactly two places — `throw`, to set `E(0)`, and this gate. It cannot feed back.
    ///    Changing its zero point moved the reading by 1.152 J **on a state that had not been
    ///    stepped**, which is what `the_gain_was_in_the_observer_not_the_dynamics` measures
    ///    with zero substeps taken. An integrator leak lives in the trajectory; this lived in
    ///    the observer, and the two are distinguishable by exactly that test.
    /// 2. **The loss was recoverable by naming one channel.** Instrumenting the projectile's
    ///    contact damping recovered 4.82e7 J of the landscape's 4.87e7 J residual — **99.0%,
    ///    and the recovered quantity is that channel's own time integral**, not a fitted
    ///    remainder. Integrator error is not recoverable by naming a channel; that is the
    ///    difference between the two hypotheses, and the ratio went 76% → 0.8%.
    /// 3. **What remains has the wrong sign for the named candidate.** Semi-implicit Euler
    ///    injects on STIFF contacts, so that hypothesis predicts the residual fraction rises
    ///    with contact stiffness. Measured it falls: the landscape at `k = 2.44e13` N/m has
    ///    the smallest fraction (0.8%) and the sandbox at `k = 6.89e5` N/m — seven orders
    ///    softer — the largest (6.4%). Three tiers that differ in more than stiffness, so
    ///    this is a wrong-sign observation and not a measurement of the integrator's
    ///    contribution; it is reported at that strength.
    ///
    /// Every tier dissipates and every residual is a LOSS. That sign is load-bearing on its
    /// own terms: an explicit integrator and an uninstrumented sink can both only take energy
    /// out here, so a negative residual would be a fact neither explains.
    ///
    /// Ruled out and kept ruled out: adaptive materialization is not involved. The node count
    /// is constant across the flight (118,296 at the sandbox tier, before and after).
    ///
    /// What the 0.8–6.4% still holds is named in [`DissipationLedger`]'s header rather than
    /// left as slack: the projectile's contact potential is not in `E(t)`, a projectile
    /// contact against an asleep cell drops the momentum instead of delivering it, and
    /// semi-implicit Euler carries `O(dt)` error per step. The band is 90–100%, which is
    /// where these sit; if it moves, one of those three moved and someone should find out
    /// which.
    #[test]
    fn the_two_sided_balance_closes_where_the_chart_has_the_symmetry() {
        let mut checked = 0_usize;
        for id in TierId::ALL {
            if !Session::new(id).balance_applicability().holds() {
                continue;
            }
            checked += 1;
            let session = flown(id);
            let dissipated = session.opening_energy_j() - session.total_energy_j();
            assert!(
                dissipated > 0.0,
                "{id:?} GAINED energy: dissipated {dissipated:e} J. This chart has \
                 time-translation symmetry, so Noether makes conservation a theorem of the \
                 continuum equations and a secular gain is a defect — integrator, \
                 unaccounted declared channel, or bug. The last time this fired it was the \
                 ledger's own overlap zero point, not the engine."
            );
            let explained = session.energy_ledger().total_j() / dissipated;
            assert!(
                (0.90..=1.0).contains(&explained),
                "{id:?}: named channels explained {explained:.4} of its dissipation. Measured \
                 0.9361 / 0.9541 / 0.9920 for sandbox / grain / landscape."
            );
            let residual = session.energy_residual_j();
            assert!(
                residual > 0.0,
                "{id:?}: residual {residual:e} J is NEGATIVE — the unaccounted term is \
                 putting energy IN. Neither an explicit integrator nor a missing sink can do \
                 that here."
            );
            let fraction = session.energy_residual_fraction();
            assert!(
                (0.0..0.10).contains(&fraction),
                "{id:?}: residual is {fraction:.4} of the dissipation; measured 0.0639 / \
                 0.0459 / 0.0080"
            );
        }
        assert_eq!(
            checked, 3,
            "the balance was checked at {checked} tiers; three have the symmetry (grain, \
             sandbox, landscape). If this changed, a chart changed and the gate's warrant \
             has to be re-argued rather than the count edited."
        );
    }

    /// **THE GATE REFUSES WHERE THE CHART HAS NO CONSERVED ENERGY, and telling the two
    /// refusals apart is the point.**
    ///
    /// Energy conservation is chart-relative. It follows from time-translation symmetry via
    /// Noether, and this engine ships a chart that genuinely lacks it: the cosmic tier's
    /// oversized patch, where the expansion background `(H L/c)²` has grown past the screen.
    /// In an expanding universe total vacuum energy **grows with the volume** — predicted and
    /// observed — so a balance gate firing there would be enforcing a law the chart does not
    /// have and reporting correct physics as a defect. It refuses instead.
    ///
    /// **Static curvature is NOT that case and must not be folded into it.** A static metric
    /// has a timelike Killing vector, so a conserved energy exists; what is missing is only
    /// that `total_energy_j` sums the Newtonian expression rather than the Killing energy.
    /// One refusal is about the world and one is about the instrument, and only the second is
    /// fixable — merging them would hide that.
    ///
    /// Measured, same tier, two declared scenes, discriminated by the weak-field screen's own
    /// arithmetic rather than by tier name:
    ///
    /// | scene | ε_bg | screen | applicability |
    /// |---|---|---|---|
    /// | 30 Mpc comoving patch | 4.549e-5 | certifies | conserved quantity not computed |
    /// | 100 Mpc comoving patch | 5.054e-4 | `ExpansionScale` | **no time-translation symmetry** |
    ///
    /// **MUST FIRE both ways.** A refusal test that only checked the refusing case would pass
    /// on a gate that refused everywhere, which is the failure mode a blanket refusal is.
    #[test]
    fn the_gate_refuses_where_the_chart_has_no_conserved_energy() {
        // The expanding case: the law is absent, so there is nothing to gate.
        let mut oversized = Session::new(TierId::Cosmic);
        oversized.set_gravity_scene(1);
        assert_eq!(
            oversized.gravity_scene().map(|s| s.certify().refusal),
            Some(Some(WeakFieldRefusal::ExpansionScale)),
            "the 100 Mpc patch was expected to trip the expansion screen; if it no longer \
             does, this test is no longer about an expanding chart"
        );
        assert_eq!(
            oversized.balance_applicability(),
            BalanceApplicability::RefusedNoTimeTranslationSymmetry
        );

        // The static curved case: the law is present, this instrument is the wrong one.
        let patch = Session::new(TierId::Cosmic);
        assert_eq!(
            patch.balance_applicability(),
            BalanceApplicability::RefusedConservedQuantityNotComputed,
            "the 30 Mpc patch certifies against the expansion screen, so a static chart is \
             licensed and a timelike Killing vector exists there"
        );
        for id in [TierId::Planet, TierId::Galactic] {
            assert_eq!(
                Session::new(id).balance_applicability(),
                BalanceApplicability::RefusedConservedQuantityNotComputed,
                "{id:?} is a static weak-field chart: conserved energy exists, and it is the \
                 Killing energy rather than the sum total_energy_j takes"
            );
        }

        // And it does NOT refuse everywhere, which is what makes the refusals mean anything.
        for id in [TierId::Grain, TierId::Sandbox, TierId::Landscape] {
            assert!(
                Session::new(id).balance_applicability().holds(),
                "{id:?} is flat and Newtonian with static gravity; the balance is a law there"
            );
        }
        assert_eq!(
            Session::new(TierId::Gauge).balance_applicability(),
            BalanceApplicability::RefusedNoMechanicalEvaluator
        );

        // Every verdict says what it means in plain words, refusals included.
        for a in [
            BalanceApplicability::Holds,
            BalanceApplicability::RefusedNoTimeTranslationSymmetry,
            BalanceApplicability::RefusedConservedQuantityNotComputed,
            BalanceApplicability::RefusedNoMechanicalEvaluator,
        ] {
            assert!(a.plain().len() > 40, "{a:?} has no plain-language reading");
        }
    }

    /// **THE GAIN WAS IN THE OBSERVER, NOT THE DYNAMICS — measured with zero substeps
    /// taken.**
    ///
    /// This is the leg that separates an accounting gap from an integrator leak. An
    /// integrator leak lives in the trajectory and needs stepping to appear. This did not:
    /// it is a difference between two ways of reading ONE fixed state, and it is the whole of
    /// the reported energy creation.
    ///
    /// A settled scene is woken so its packed pairs enter `active_pairs`, and the overlap
    /// potential is then summed twice over that same untouched state — once from geometric
    /// touching, `radius[i] + radius[j]`, and once from the force law's own zero,
    /// `rest_gap`. The lattice sits at the frontier's spacing, so the second is ~zero and the
    /// first is not, and the difference is the fictitious energy. **No substep runs between
    /// the two readings.**
    ///
    /// **MUST FIRE.** If the two agree, the zero points have converged and the +1.15 J
    /// finding no longer has a mechanism.
    #[test]
    fn the_gain_was_in_the_observer_not_the_dynamics() {
        let session = flown(TierId::Sandbox);
        let before = session.time_s();
        let mut geometric = 0.0;
        let mut force_law = 0.0;
        let mut pairs = 0_usize;
        for (i, j) in session.active_pairs.iter().copied() {
            let dx = session.nodes.position[j][0] - session.nodes.position[i][0];
            let dy = session.nodes.position[j][1] - session.nodes.position[i][1];
            let distance = (dx * dx + dy * dy).sqrt();
            let touching = session.nodes.radius_m[i] + session.nodes.radius_m[j];
            if distance < touching {
                let overlap = touching - distance;
                geometric += 0.5 * session.contact_stiffness_n_m * overlap * overlap;
            }
            let zero = session.rest_gap(i, j);
            if distance < zero {
                let overlap = zero - distance;
                force_law += 0.5 * session.contact_stiffness_n_m * overlap * overlap;
            }
            pairs += 1;
        }
        assert!(pairs > 1000, "only {pairs} active pairs; the term has nothing to sum over");
        assert_eq!(before, session.time_s(), "a substep ran between the two readings");

        let fiction = geometric - force_law;
        assert!(
            (1.0..1.3).contains(&fiction),
            "the two zero points differ by {fiction:e} J on one untouched state; measured \
             1.152 J, which is the whole of the +1.139 J the balance reported as created"
        );
        assert!(
            force_law < 0.01 * geometric,
            "the force law's own zero should read ~nothing on a lattice sitting at the \
             frontier's spacing: {force_law:e} vs {geometric:e}"
        );
    }

    /// **The residual fraction has to be scale-relative, and the grain tier is the witness.**
    ///
    /// [`Session::energy_residual_fraction`] guarded its divide with `dissipated.abs() <
    /// 1e-12` in absolute joules. The grain tier's ENTIRE energy is 8.1e-10 J and it
    /// dissipates 9.3e-15 J, so the guard swallowed the whole measurement and reported a real
    /// 4.6% residual as a clean 0.000 — an instrument reading zero because the scene is
    /// smaller than its own threshold. Now relative to `E(0)`.
    ///
    /// **MUST FIRE**: the grain tier's fraction must be non-zero and must agree with the
    /// ratio computed by hand.
    #[test]
    fn the_residual_fraction_survives_a_small_scene() {
        let session = flown(TierId::Grain);
        let fraction = session.energy_residual_fraction();
        assert!(
            fraction > 0.0,
            "the grain tier reported a zero residual fraction; its energies are 1e-10 J and \
             an absolute joule guard swallows them"
        );
        let by_hand = session.energy_residual_j()
            / (session.opening_energy_j() - session.total_energy_j());
        assert!((fraction - by_hand).abs() < 1.0e-12, "{fraction} vs {by_hand}");
    }
}

#[cfg(test)]
mod merge_ledger {
    use super::*;

    fn flown(id: TierId) -> Session {
        let mut session = Session::new(id);
        session.throw(0.5, 0.4, 0.6);
        for _ in 0..240 {
            session.step(1.0 / 60.0);
        }
        session
    }

    /// **MUST NOT FIRE.** Nothing has been collapsed before anything is stepped.
    #[test]
    fn an_unstepped_session_has_destroyed_nothing() {
        let m = Session::new(TierId::Sandbox).merge_ledger();
        assert_eq!(m.total_nats(), 0.0);
        assert_eq!(m.total_events(), 0);
    }

    /// **MUST FIRE.** Both live merge sites must move off zero on a sandbox throw.
    #[test]
    fn the_merge_sites_fire() {
        let m = flown(TierId::Sandbox).merge_ledger();
        assert!(m.sleep_events > 0 && m.sleep_nats > 0.0, "sleep never merged: {m:?}");
        assert!(
            m.wall_clamp_events > 0 && m.wall_clamp_nats > 0.0,
            "wall clamp never merged: {m:?}"
        );
    }

    /// **THE PRICE OF A COLLAPSE IS THE MANTISSA, and that is the check that says the
    /// instrument is measuring what it claims.**
    ///
    /// Discarding a normal `f64` destroys the `|x|/ulp(x)` distinctions it could have
    /// carried — about `2^52`, so **≈ 36.4 nats (52.5 bits)** each. A sleep collapses TWO
    /// velocity components and must price at twice that; a wall clamp collapses ONE
    /// position overshoot and must price at once.
    ///
    /// Measured: sleep **72.78 nats/event** at the grain and landscape tiers, and the
    /// sandbox's mixed 48.15 is exactly the 2777-sleep / 5757-wall blend of the two. If
    /// these drift off the mantissa, the ledger has stopped counting collapsed
    /// distinctions and is counting something else.
    #[test]
    fn a_collapsed_double_costs_the_mantissa() {
        let m = flown(TierId::Grain).merge_ledger();
        assert!(m.sleep_events > 0);
        assert_eq!(m.wall_clamp_events, 0, "the grain tier was not expected to hit a wall");
        let per_event = m.sleep_nats / m.sleep_events as f64;
        assert!(
            (70.0..75.0).contains(&per_event),
            "a sleep collapses two doubles and should cost ~72.8 nats; measured {per_event:.3}"
        );
        // Two components, each the mantissa.
        let per_double_bits = per_event / 2.0 / core::f64::consts::LN_2;
        assert!(
            (50.0..54.0).contains(&per_double_bits),
            "one collapsed double should cost ~52.5 bits; measured {per_double_bits:.3}"
        );
    }

    /// **THE TWO LEDGERS DISAGREE ABOUT THE WALL, AND BOTH ARE RIGHT. This is the result.**
    ///
    /// Two adjacent lines of the same wall handler:
    ///
    /// * the **position clamp** sets an interval of positions to one point — many-to-one, so
    ///   it destroys information, and the sigma-ledger charges it. It dissipates no energy.
    /// * the **velocity flip** `v ↦ −v·e` dissipates energy, which the D-ledger charges to
    ///   `wall_restitution_j` — but it is **injective**, so it destroys no information at
    ///   all. It relabels a distinction rather than merging it.
    ///
    /// So the same wall shows up in both ledgers, for different lines, and neither number
    /// is derivable from the other. **Energy loss and information loss are different events**
    /// — which is the entire reason for keeping two currencies rather than converting one
    /// into the other.
    #[test]
    fn energy_loss_and_information_loss_are_different_events() {
        let session = flown(TierId::Sandbox);
        let joules = session.energy_ledger();
        let nats = session.merge_ledger();
        assert!(
            joules.wall_restitution_j > 0.0,
            "the wall dissipated no energy, so this comparison has nothing to compare"
        );
        assert!(
            nats.wall_clamp_nats > 0.0,
            "the wall destroyed no information, so this comparison has nothing to compare"
        );
        // Sleep is the site where BOTH fire, and they still measure different things: the
        // joules scale with velocity squared, the nats with the mantissa and not with the
        // velocity at all.
        assert!(joules.sleep_absorbed_j > 0.0);
        assert!(nats.sleep_nats > 0.0);
    }

    /// **THE VERDICT ON THE THREE SITES: the price is the FORMAT, not the scene — so at
    /// these sites the ledger is an event counter, and its nats are a ceiling.**
    ///
    /// `nats_collapsed` charges `ln(|x|/ulp(x))`, the mantissa, which is ≈36.4 nats for every
    /// normal `f64` in existence. So the per-event price cannot depend on the state, and it
    /// does not: a sleep collapses two doubles and prices at **72.76 / 72.78 / 72.80 nats**
    /// at the sandbox / grain / landscape tiers — three scenes whose energies span 26 orders
    /// of magnitude, agreeing to 0.05%.
    ///
    /// Set against the D-ledger on the SAME site, sleep, the contrast is the result: joules
    /// per sleep run **2.76e-12 J at the sandbox and 965.9 J at the landscape**, 29 orders
    /// apart, where the nats are flat. Energy loss and information loss are different events
    /// and no conversion factor relates them — and the same fact says this instrument
    /// **cannot separate a cheap collapse from an expensive one.** Its readings are a
    /// ceiling on distinguishability destroyed, not a measurement of it; see
    /// [`MergeLedger`]'s header for why the reachable set is what a real one would need.
    ///
    /// **MUST FIRE both halves.** A version of this that only checked the nats were equal
    /// would pass on an instrument that was broken and constant.
    #[test]
    fn the_price_is_the_format_and_not_the_scene() {
        let mut nats_per_sleep = Vec::new();
        let mut joules_per_sleep = Vec::new();
        for id in [TierId::Sandbox, TierId::Grain, TierId::Landscape] {
            let session = flown(id);
            let m = session.merge_ledger();
            assert!(m.sleep_events > 0, "{id:?} never slept a cell, so it prices nothing");
            nats_per_sleep.push(m.sleep_nats / m.sleep_events as f64);
            joules_per_sleep
                .push(session.energy_ledger().sleep_absorbed_j / m.sleep_events as f64);
        }

        // The nats are flat: the format sets the price.
        let lo = nats_per_sleep.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = nats_per_sleep.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!(
            hi / lo - 1.0 < 1.0e-3,
            "nats per sleep varied across tiers: {nats_per_sleep:?}. If this spreads, \
             nats_collapsed stopped charging the mantissa and is charging something else."
        );
        assert!((70.0..75.0).contains(&lo), "two collapsed doubles cost ~72.8 nats: {lo}");

        // The joules are not: the scene sets that price. Without this half the test would
        // pass on a constant.
        let jlo = joules_per_sleep.iter().cloned().fold(f64::INFINITY, f64::min);
        let jhi = joules_per_sleep.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!(
            jhi / jlo > 1.0e12,
            "joules per sleep should span the tiers' own scales, not track the nats: \
             {joules_per_sleep:?}"
        );
    }

    /// **The anchor site reads zero for a reason, not because it is dead.**
    ///
    /// The branch executes — floor cells are anchored and they are in the awake list — but
    /// an anchored cell's velocity is zeroed **before** force is applied to it, so it never
    /// accumulates anything there is to collapse. Contrast sleep, which zeroes a velocity
    /// that HAS accumulated and therefore does destroy distinctions.
    ///
    /// Pinned so a future zero here is read as this fact rather than as a broken counter.
    #[test]
    fn the_anchor_destroys_nothing_because_nothing_accumulates_there() {
        for id in [TierId::Sandbox, TierId::Grain, TierId::Landscape] {
            let m = flown(id).merge_ledger();
            assert_eq!(
                m.anchor_nats, 0.0,
                "{id:?}: the anchor collapsed something; velocity is supposed to be zeroed \
                 before force reaches an anchored cell"
            );
        }
    }
}



