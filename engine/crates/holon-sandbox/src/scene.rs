//! One tier's scene: the arena, the generator that grows it, the model that certifies
//! it, and the one physics that runs on every tier that has one.
//!
//! # Sand is stone with the bonds already broken
//!
//! There is exactly one relation type here and exactly one solver. The difference
//! between the sandbox and the landscape is the INITIAL DAMAGE on the relation holons:
//! stone starts at `D = 0` and may break; sand starts at `D = 1` and cannot un-break.
//! That is not an analogy dressed up as a design. `material.rs`'s own jurisdiction rule
//! says a `D = 1` relation "returns zero forever" and hands its pair to the contact
//! solver, and that the solver's jurisdiction is "exactly the union of fully failed
//! (D = 1) pairs and never-bonded pairs". Granular contact is therefore the shipped
//! cohesive model at one particular value of one field, which is what "one holon,
//! values only" is supposed to mean when it is meant literally.
//!
//! # What the certificate is about
//!
//! The macro claim is the throw's IMPULSE and the extent of the disturbance it causes.
//! The resolution surrogate demands process-zone spacing near where the interaction is
//! and relaxes it linearly with distance, exactly as `fracture.rs` does around a damage
//! surface — with the throw's contact point standing in for the surface before anything
//! has been disturbed. Where a tier's own `l_ch/10` cannot be met at that tier's grain
//! floor, the answer is `GrainFloor` and no impulse is claimed.

use ciris_sim_core::holon::{Channels, Decomposition, HolonError};
use ciris_sim_core::homogenization::{derive_bilinear_cohesive_law, HomogenizationError};
use ciris_sim_core::material::{CohesiveBond, CohesiveLaw};
use ciris_sim_core::regplus::GrossState;
use ciris_sim_core::runtime::{
    RuntimeArena, RuntimeHolonSpec, RuntimeMaterializer, NO_RUNTIME_HOLON,
};

use crate::chart::{apportion, Cell, Chart, FANOUT};
use crate::incremental::{CellwiseModel, Settled};
use crate::tier::{Evaluator, Tier};

/// Observables the certificate carries: total contact impulse (N·s), disturbance extent
/// (m), and the count of relation holons that reached full damage.
pub const OBSERVABLES: usize = 3;

/// Momentum-balance tolerance, matching `fracture::CONSERVATION_TOLERANCE`.
pub const CONSERVATION_TOLERANCE: f64 = 1.0e-9;

/// Uniform chart gravity, m/s².
///
/// Gravity is CHART data and never a per-holon field — the equivalence principle makes
/// uniform g frame-equivalent to an accelerated chart, and universality of free fall
/// forbids a per-holon value (INTEGRATION_FRAME.md, 2026-08-23). One value, one place,
/// no stage knob: this demo's bonds are derived from the descriptor rather than
/// hand-scaled, so it does not need the `STAGE_WALL_GRAVITY_FACTOR` the wall demo
/// declares.
pub const CHART_GRAVITY_M_S2: f64 = 9.806_65;

/// Whole-state layout of a scene holon. Slot 0 is the fraction of the cell that is
/// MATTER, which is the one ensemble fact about a cell that does not factor through any
/// single child: knowing every sub-cell's fill tells you the parent's, but knowing the
/// parent's tells you nothing about which sub-cells hold it.
pub const WHOLE_LEN: usize = 1;
pub const WHOLE_FILL: usize = 0;

/// Grows a latent cell into four quadrants, apportioning the REG+ ledger by how much
/// MATTER each quadrant contains rather than splitting it evenly.
///
/// An even split would be the easy thing and it would be wrong: a quadrant of air would
/// receive a quarter of the sand. The generator reads the parent's own fill fraction and
/// the geometry of the matter line, apportions by largest remainder so the shares
/// compose exactly, and the core's transactional check then enforces that composition
/// without being told about any of it.
pub struct QuadrantMaterializer {
    chart: Chart,
    /// Height of the matter line in tier metres. Everything below is matter.
    matter_line_m: f64,
    /// Cells at or below this size are settled: no demand can ask more of them, so they
    /// are flagged non-boundary.
    ///
    /// This flag is load-bearing rather than cosmetic. `GrainFloor` outranks
    /// `RefinementUnavailable`, so one active boundary holon sitting at grain 1 halts
    /// adaptive materialization for the whole scene — the reason
    /// `fracture::TipSpacingSelector` exists. Without it the sandbox stops growing the
    /// moment any corner of it bottoms out.
    settled_size_m: f64,
}

impl QuadrantMaterializer {
    pub fn new(domain_m: f64, matter_line_m: f64, settled_size_m: f64) -> Self {
        Self {
            chart: Chart::new(domain_m),
            matter_line_m,
            settled_size_m,
        }
    }

    pub fn chart(&self) -> &Chart {
        &self.chart
    }

    pub fn chart_mut(&mut self) -> &mut Chart {
        &mut self.chart
    }
}

impl RuntimeMaterializer for QuadrantMaterializer {
    fn materialize(&mut self, arena: &mut RuntimeArena, holon: usize) -> Result<bool, HolonError> {
        self.chart.sync(arena);
        let record = *arena.holon(holon).ok_or(HolonError::InvalidParent)?;
        if record.decomposition != Decomposition::Latent || record.grain_units == 1 {
            return Ok(false);
        }
        let parent_cell = self.chart.cell(holon);
        let half = 0.5 * parent_cell.size;
        let grain_units = (record.grain_units / 2).max(1);
        let decomposition = if grain_units == 1 {
            Decomposition::Leaf
        } else {
            Decomposition::Latent
        };
        let depth = record.depth.checked_add(1).ok_or(HolonError::InvalidDepth)?;

        // The child map is the CHART's, not a second copy of it living here. The two
        // used to be written out separately and nothing checked they agreed — see
        // `Cell::child`.
        let quadrants: Vec<Cell> = (0..FANOUT).map(|ordinal| parent_cell.child(ordinal)).collect();
        let fills: Vec<f64> = quadrants
            .iter()
            .map(|cell| cell.fraction_below(self.matter_line_m))
            .collect();

        let constituents = apportion(record.gross.constituents, &fills);
        let occupancy = apportion(record.gross.occupancy, &fills);
        let wholes: Vec<[f64; WHOLE_LEN]> = fills.iter().map(|fill| [*fill]).collect();
        let specs: Vec<RuntimeHolonSpec<'_>> = (0..FANOUT)
            .map(|i| RuntimeHolonSpec {
                parent: holon as u32,
                depth,
                grain_units,
                gross: GrossState::aggregate(
                    constituents[i],
                    occupancy[i],
                    // Momentum is carried by the solver's node state, not apportioned
                    // geometrically: a cell's share of the ledger's momentum is a
                    // dynamical fact, and inventing one here would be a number with no
                    // ancestor. The scene's roots carry zero, so this composes exactly.
                    [0, 0],
                ),
                whole: &wholes[i],
                channels: record.channels,
                boundary: half > self.settled_size_m && constituents[i] > 0,
                decomposition,
            })
            .collect();
        arena.materialize(holon, &specs)?;
        self.chart.sync(arena);
        Ok(true)
    }
}

/// The resolution surrogate: how badly this cell's size violates what the tier's own
/// `l_ch/10` demands at its distance from the interaction.
pub struct ResolutionModel {
    chart: Chart,
    /// `l_ch / 10` for this tier's chart values, in metres.
    required_m: f64,
    /// The OBSERVER's claim: no cell in view may be coarser than this, ever. The join
    /// of this and the physics demand is what gets certified.
    acuity_m: f64,
    /// How fast the demand relaxes with distance from the interaction.
    grading: f64,
    /// Where the interaction is, or `None` when there is no interaction at all. With
    /// no focus only the observer's claim applies, which is exactly the resting scene:
    /// nothing is happening, so nothing but the eye is asking for anything.
    focus: Option<[f64; 2]>,
    epoch: u64,
    /// Readout of the last solve, or zeros before one has run.
    last: Settled<OBSERVABLES>,
    /// Set when a solve has been run against the current frontier, so the same frontier
    /// is not re-solved on a later round.
    solved_key: Option<Vec<usize>>,
    solve: Option<SolveRequest>,
    pub solves_run: usize,
    pub calls: usize,
    pub settles: usize,
}

/// What a settle asks the host to run. Kept as data rather than a callback so the
/// solver stays out of the certification loop's borrow graph.
#[derive(Clone, Debug, PartialEq)]
pub struct SolveRequest {
    pub active: Vec<usize>,
}

impl ResolutionModel {
    pub fn new(
        domain_m: f64,
        required_m: f64,
        acuity_m: f64,
        grading: f64,
        focus: [f64; 2],
    ) -> Self {
        Self {
            chart: Chart::new(domain_m),
            required_m,
            acuity_m,
            grading,
            focus: Some(focus),
            epoch: 0,
            last: Settled {
                observables: [0.0; OBSERVABLES],
                conservation_residual: 0.0,
            },
            solved_key: None,
            solve: None,
            solves_run: 0,
            calls: 0,
            settles: 0,
        }
    }

    pub fn chart(&self) -> &Chart {
        &self.chart
    }

    /// The spacing this cell is allowed: the JOIN of two claims.
    ///
    /// The physics claim relaxes with distance from the interaction — a cell far from
    /// anything happening may be coarse. The OBSERVER's claim does not relax at all: a
    /// cell in view may never be coarser than what a viewer can distinguish. The
    /// tighter of the two binds, everywhere.
    ///
    /// Taking only the physics claim is what made the scene coarsen away from the impact
    /// corridor. That was correct about the physics and wrong about the frame: the
    /// observer is a claimant too, and their demand is uniform because their eyes are.
    fn allowed_spacing(&self, distance: Option<f64>) -> f64 {
        match distance {
            Some(distance) => self
                .acuity_m
                .min(self.required_m.max(self.grading * distance)),
            None => self.acuity_m,
        }
    }

    /// Move the interaction focus, e.g. once a contact point has actually been
    /// measured. Bumps the epoch, because every cell error is measured from it.
    pub fn set_focus(&mut self, focus: Option<[f64; 2]>) {
        if self.focus != focus {
            self.focus = focus;
            self.epoch += 1;
            self.solved_key = None;
        }
    }

    /// Record the readout of a solve the host ran for the frontier this model asked
    /// about.
    pub fn record(&mut self, active: Vec<usize>, settled: Settled<OBSERVABLES>) {
        self.last = settled;
        self.solved_key = Some(active);
        self.solves_run += 1;
    }

    /// The frontier a settle wants solved, if any. Taken, not peeked: a request is
    /// served once.
    pub fn take_solve_request(&mut self) -> Option<SolveRequest> {
        self.solve.take()
    }
}

impl CellwiseModel<OBSERVABLES> for ResolutionModel {
    fn cell_error(&mut self, arena: &RuntimeArena, holon: usize) -> f64 {
        self.calls += 1;
        self.chart.sync(arena);
        let cell = self.chart.cell(holon);
        // An empty cell has nothing to resolve. Demanding process-zone spacing of air
        // would refine the whole domain and certify nothing.
        if arena.holons()[holon].gross.constituents == 0 {
            return 0.0;
        }
        let distance = self.focus.map(|focus| cell.distance_to(focus));
        (cell.size / self.allowed_spacing(distance) - 1.0).max(0.0)
    }

    fn settle(
        &mut self,
        _arena: &RuntimeArena,
        active: &[usize],
        _bound: f64,
    ) -> Settled<OBSERVABLES> {
        self.settles += 1;
        if self.solved_key.as_deref() != Some(active) {
            self.solve = Some(SolveRequest {
                active: active.to_vec(),
            });
        }
        self.last
    }

    fn epoch(&self) -> u64 {
        self.epoch
    }
}

/// Why a tier's relations could not be given a cohesive law.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LawRefusal {
    /// The cell spacing is at or above `2 l_ch`, where the bilinear law's failure
    /// opening would not exceed its peak opening. `ciris_sim_core::homogenization`
    /// refuses rather than returning a snapped-back law, and so does this.
    UnderResolved,
    /// The tier declares no material chart at all.
    NoMaterial,
}

/// Derive the relation law for cells of `spacing_m` at this tier, or say why not.
///
/// The refusal here is the shipped homogenizer's, not a rule invented for the demo:
/// `derive_bilinear_cohesive_law` rejects any spacing at which no positive softening
/// branch exists, on the grounds that "any returned number would be papering over P2".
///
/// Nothing in this function knows which tier is sand. It does not need to: at the
/// sandbox tier the cells are 0.5 mm and quartz's own `2 l_ch` is 4.59 um, so the
/// homogenizer refuses by a factor of 109 and what is left is contact. "Sand does not
/// crack, it pours" is therefore a MEASURED consequence of quartz's chart values at that
/// grain, not a property the tier was handed. `tests::the_sandbox_earns_its_contact_only
/// _verdict` is that check.
pub fn relation_law(
    tier: &Tier,
    spacing_m: f64,
    node_mass_kg: f64,
) -> Result<CohesiveLaw, LawRefusal> {
    let material = tier.material.ok_or(LawRefusal::NoMaterial)?;
    let derived = derive_bilinear_cohesive_law(&material, node_mass_kg, spacing_m).map_err(
        |error| match error {
            HomogenizationError::CohesiveUnderResolved => LawRefusal::UnderResolved,
            _ => LawRefusal::NoMaterial,
        },
    )?;
    let stiffness = derived.peak_force_n / derived.opening_at_peak_m;
    Ok(CohesiveLaw {
        stiffness_n_m: stiffness,
        // Solver-owned dissipation, named as such (amendment A5): critical damping of
        // one node on one relation, scaled by the material's declared SOLVER ratio.
        damping_n_s_m: 2.0
            * material.solver_damping_ratio
            * (stiffness * node_mass_kg).sqrt(),
        peak_force_n: derived.peak_force_n,
        fracture_energy_j: derived.fracture_energy_j,
        friction_coefficient: 0.74,
    })
}

/// The resident mechanical state the solver steps. One entry per active cell.
#[derive(Clone, Debug, Default)]
pub struct Nodes {
    pub holon: Vec<usize>,
    pub position: Vec<[f64; 2]>,
    pub velocity: Vec<[f64; 2]>,
    pub mass_kg: Vec<f64>,
    pub radius_m: Vec<f64>,
    /// Cells resting on the domain floor are held: the box has a bottom.
    pub anchored: Vec<bool>,
    /// Whether this cell is being integrated at all.
    ///
    /// A scene at rest is a scene at rest: the sand has been sitting in the box, and
    /// nothing about it is moving until something disturbs it. Asleep cells are RESIDENT
    /// and DRAWN — each one is still a holon, and the certificate still covers it — they
    /// are simply not stepped. That is what makes an acuity-pinned scene affordable:
    /// 157,804 cells resident, a few hundred of them awake.
    ///
    /// This is not a rendering trick. The alternative — integrating a hundred thousand
    /// cells that are provably not moving — would buy nothing and cost the frame.
    pub awake: Vec<bool>,
    /// Consecutive substeps this cell has been slow enough to sleep.
    pub still: Vec<u16>,
}

impl Nodes {
    pub fn len(&self) -> usize {
        self.holon.len()
    }

    pub fn is_empty(&self) -> bool {
        self.holon.is_empty()
    }

    pub fn clear(&mut self) {
        self.holon.clear();
        self.position.clear();
        self.velocity.clear();
        self.mass_kg.clear();
        self.radius_m.clear();
        self.anchored.clear();
        self.awake.clear();
        self.still.clear();
    }

    /// Wake a cell, if it is not already awake.
    pub fn wake(&mut self, index: usize) {
        self.awake[index] = true;
        self.still[index] = 0;
    }
}

/// Build the resident mechanical state from a certified frontier.
///
/// A cell's mass is its OWN ledger entry times the tier's mass per terminal holon, so
/// the resident nodes weigh exactly what the latent holons they stand for weigh. That
/// is the composition check paying for itself: a frontier that conserves the ledger
/// conserves the mass without a separate assertion.
pub fn build_nodes(
    arena: &RuntimeArena,
    chart: &Chart,
    active: &[usize],
    mass_per_constituent_kg: f64,
    domain_m: f64,
) -> Nodes {
    let mut nodes = Nodes::default();
    for holon in active {
        let record = arena.holons()[*holon];
        if record.gross.constituents == 0 {
            continue;
        }
        let Some(cell) = chart.get(*holon) else {
            continue;
        };
        let centre = cell.centre();
        // A cell is only partly full; it sits at the centre of the MATTER it holds, not
        // the centre of its rectangle.
        let fill = arena
            .whole_state(*holon)
            .and_then(|whole| whole.get(WHOLE_FILL).copied())
            .unwrap_or(1.0)
            .clamp(0.0, 1.0);
        let matter_height = fill * cell.size;
        nodes.holon.push(*holon);
        nodes
            .position
            .push([centre[0], cell.y0 + 0.5 * matter_height.max(1.0e-12)]);
        nodes.velocity.push([0.0, 0.0]);
        nodes
            .mass_kg
            .push(record.gross.constituents as f64 * mass_per_constituent_kg);
        nodes.radius_m.push(0.5 * cell.size);
        nodes.anchored.push(cell.y0 <= 1.0e-9 * domain_m);
        // Everything starts asleep. The scene is a configuration that already exists;
        // nothing in it is moving until a throw disturbs it.
        nodes.awake.push(false);
        nodes.still.push(0);
    }
    nodes
}

/// The relation holons joining resident cells.
///
/// Where a cohesive law exists the pairs are BONDED and the bond owns them, including
/// while closed. Where the homogenizer refused, the pairs are NEVER-BONDED and the
/// contact solver owns them outright. That is not two models: `material.rs`'s
/// jurisdiction corollary says in as many words that the solver's jurisdiction is
/// "exactly the union of fully failed (D = 1) pairs and never-bonded pairs", so a
/// never-bonded pair is a case the shipped regime table already covers.
#[derive(Clone, Debug, Default)]
pub struct Relations {
    pub bonds: Vec<CohesiveBond>,
    /// Index into [`Nodes`] for each bond's two ends.
    pub ends: Vec<[usize; 2]>,
    /// Why there are no bonds, when there are none.
    pub refusal: Option<LawRefusal>,
    /// Bond indices touching each node, so "does a live relation own this pair?" is a
    /// short scan of one node's own relations instead of a search of all of them.
    ///
    /// It is also what lets the awake working set be rebuilt from the awake NODES rather
    /// than by filtering every relation in the scene.
    ///
    /// Without this the contact loop is `O(N^2 * E)`, which measured 5.3 SECONDS per
    /// frame on a 200-node, 462-bond landscape frontier. It was the single largest cost
    /// in the whole demo and it was invisible until the per-tier timings were printed.
    adjacency: Vec<Vec<u32>>,
}

impl Relations {
    /// Whether a live (`D < 1`) relation owns this pair. While a bond lives it owns the
    /// closed regime too, so the contact solver must stay off the pair entirely —
    /// `material.rs`'s jurisdiction corollary.
    /// Bond indices touching `node`.
    pub fn touching(&self, node: usize) -> &[u32] {
        self.adjacency
            .get(node)
            .map(|bonds| bonds.as_slice())
            .unwrap_or(&[])
    }

    pub fn owns(&self, a: usize, b: usize) -> bool {
        let Some(bonds) = self.adjacency.get(a) else {
            return false;
        };
        bonds.iter().any(|index| {
            let ends = self.ends[*index as usize];
            (ends[0] == b || ends[1] == b) && !self.bonds[*index as usize].is_broken()
        })
    }

    pub fn len(&self) -> usize {
        self.bonds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bonds.is_empty()
    }

    /// Relation holons at full damage. A crack is not painted and not declared: it IS
    /// this set (`MATERIALS_AND_FRACTURE.md`).
    pub fn cracked(&self) -> usize {
        self.bonds.iter().filter(|bond| bond.damage() >= 1.0).count()
    }
}

/// Join every candidate pair of resident cells whose matter is close enough to touch,
/// and give each join a relation holon.
///
/// `pairs` comes from the grid broadphase, built with a margin wide enough to cover
/// `reach` beyond touching, so no bond that should exist is missed.
///
/// The relation holon ids continue past the scene arena's own ids, which is what
/// `MATERIALS_AND_FRACTURE.md` means by a connection being "itself addressable": a bond
/// is not an edge in a side table, it is a holon with an id, and the demo can point at
/// one.
///
/// When the law was refused there are no bonds at all — not zero-strength bonds. A
/// zero-peak `CohesiveLaw` would not even validate, and manufacturing one so that every
/// tier could have a uniform-looking bond list is exactly the papering-over the frame
/// forbids. The pairs are simply never-bonded, and the contact solver owns them.
pub fn build_relations(
    nodes: &Nodes,
    arena: &RuntimeArena,
    pairs: &[(usize, usize)],
    law: Result<CohesiveLaw, LawRefusal>,
    reach: f64,
) -> Relations {
    let mut relations = Relations::default();
    relations.adjacency = vec![Vec::new(); nodes.len()];
    let law = match law {
        Ok(law) => law,
        Err(refusal) => {
            relations.refusal = Some(refusal);
            return relations;
        }
    };
    let mut next_relation_holon = arena.len();
    for (i, j) in pairs.iter().copied() {
        let dx = nodes.position[j][0] - nodes.position[i][0];
        let dy = nodes.position[j][1] - nodes.position[i][1];
        let square = dx * dx + dy * dy;
        let touching = reach * (nodes.radius_m[i] + nodes.radius_m[j]);
        if square > touching * touching || square <= 0.0 {
            continue;
        }
        let separation = square.sqrt();
        let Ok(bond) = CohesiveBond::new(
            next_relation_holon,
            nodes.holon[i],
            nodes.holon[j],
            separation,
            law,
        ) else {
            continue;
        };
        next_relation_holon += 1;
        let index = relations.bonds.len() as u32;
        relations.adjacency[i].push(index);
        relations.adjacency[j].push(index);
        relations.bonds.push(bond);
        relations.ends.push([i, j]);
    }
    relations
}

/// Mass of one terminal holon at this tier, kg.
///
/// At the sandbox tier that is one grain of sand; at the landscape tier one centimetre
/// of rock. Both are the tier's declared `g0` cubed times its density — the same
/// arithmetic, different values, which is the whole claim.
pub fn mass_per_constituent_kg(tier: &Tier) -> f64 {
    let density = tier
        .material
        .map(|material| material.density_kg_m3)
        .unwrap_or(0.0);
    let g0 = tier.g0_m;
    if !g0.is_finite() {
        return 0.0;
    }
    // A grain is a sphere; a continuum cell is a cube. Both are the tier's own terminal
    // holon and the shape follows from what that holon IS.
    match tier.evaluator {
        Evaluator::GranularContact => core::f64::consts::PI / 6.0 * g0 * g0 * g0 * density,
        _ => g0 * g0 * g0 * density,
    }
}

/// The root holon of a tier's scene: one latent cell carrying the whole ledger.
pub fn root_scene(tier: &Tier) -> Result<RuntimeArena, HolonError> {
    let whole = [tier.fill];
    RuntimeArena::from_specs(
        &[RuntimeHolonSpec {
            parent: NO_RUNTIME_HOLON,
            depth: 0,
            grain_units: tier.root_grain_units,
            gross: GrossState::aggregate(tier.constituents, 0, [0, 0]),
            whole: &whole,
            channels: Channels::REG_PLUS.union(Channels::MECHANICAL),
            boundary: true,
            decomposition: if tier.root_grain_units == 1 {
                Decomposition::Leaf
            } else {
                Decomposition::Latent
            },
        }],
        0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tier::{tier, TierId};

    #[test]
    fn every_tier_with_a_domain_builds_a_root_that_carries_its_whole_ledger() {
        for id in TierId::ALL {
            let tier = tier(id);
            if !tier.domain_m.is_finite() {
                continue;
            }
            let arena = root_scene(&tier).expect("root scene");
            assert_eq!(arena.holons()[0].gross.constituents, tier.constituents);
            arena.validate().expect("root scene validates");
        }
    }

    /// Materialization must conserve the ledger exactly, including through cells that
    /// are entirely air. The core enforces this; the test is here because an
    /// even-splitting generator would pass the core's check while putting sand in the
    /// sky, and only the fill weights prevent that.
    #[test]
    fn quadrants_conserve_the_ledger_and_leave_air_empty() {
        let sandbox = tier(TierId::Sandbox);
        let mut arena = root_scene(&sandbox).unwrap();
        let matter_line = sandbox.fill * sandbox.domain_m;
        let mut materializer = QuadrantMaterializer::new(sandbox.domain_m, matter_line, 0.0);
        assert!(materializer.materialize(&mut arena, 0).unwrap());
        arena.validate().expect("ledger composes");

        let chart = materializer.chart();
        let mut empty = 0;
        for holon in 1..arena.len() {
            let cell = chart.cell(holon);
            let constituents = arena.holons()[holon].gross.constituents;
            if cell.y0 >= matter_line {
                assert_eq!(
                    constituents, 0,
                    "cell at y0 {} is entirely above the matter line and must be empty",
                    cell.y0
                );
                empty += 1;
            }
        }
        assert!(empty > 0, "the sandbox must have air in it");
        let total: u64 = (1..arena.len())
            .map(|holon| arena.holons()[holon].gross.constituents)
            .sum();
        assert_eq!(total, sandbox.constituents);
    }

    /// The chart places a child exactly where the generator apportioned its ledger.
    ///
    /// Both sides now call `Cell::child`, so the formula cannot disagree with itself —
    /// but they index it differently: the generator uses the ordinal it is building,
    /// and the chart uses the running child count of the parent as the holons arrive.
    /// Those agree only because runtime materialization is append-only and appends the
    /// specs in the order they were given. That is an assumption about the CORE, held
    /// in this crate, and it is the assumption a fanout change is most likely to break
    /// silently — so it is a gate rather than a comment.
    #[test]
    fn the_chart_places_a_child_where_the_generator_apportioned_it() {
        let sandbox = tier(TierId::Sandbox);
        let mut arena = root_scene(&sandbox).unwrap();
        let matter_line = sandbox.fill * sandbox.domain_m;
        let mut materializer = QuadrantMaterializer::new(sandbox.domain_m, matter_line, 0.0);
        for holon in 0..8 {
            let _ = materializer.materialize(&mut arena, holon);
        }
        assert!(arena.len() > 8, "the scene did not grow");

        let chart = materializer.chart();
        let mut seen = vec![0_usize; arena.len()];
        let mut checked = 0;
        for id in 1..arena.len() {
            let parent = arena.holons()[id].parent as usize;
            let ordinal = seen[parent];
            seen[parent] += 1;
            assert!(ordinal < FANOUT, "holon {parent} has more than FANOUT children");
            assert_eq!(
                chart.cell(id),
                chart.cell(parent).child(ordinal),
                "holon {id} is charted somewhere other than child {ordinal} of {parent}"
            );
            checked += 1;
        }
        assert!(checked >= FANOUT, "nothing was actually checked");
    }

    /// The landscape tier's own values refuse a cohesive law at the sandbox's grain,
    /// and that refusal comes from the shipped homogenizer rather than from this crate.
    #[test]
    fn a_continuum_law_is_refused_below_its_own_resolution_limit() {
        let landscape = tier(TierId::Landscape);
        let material = landscape.material.unwrap();
        let coarse = 20.0;
        let node_mass = material.density_kg_m3 * coarse * coarse * 0.1;
        assert_eq!(
            relation_law(&landscape, coarse, node_mass),
            Err(LawRefusal::UnderResolved),
            "a 20 m cell is far above 2*l_ch and must be refused"
        );

        let fine = 1.0e-2;
        let fine_mass = material.density_kg_m3 * fine * fine * 0.1;
        assert!(
            relation_law(&landscape, fine, fine_mass).is_ok(),
            "a 1 cm cell is below 2*l_ch and must derive"
        );
    }

    /// The sandbox tier's contact-only verdict is EARNED, not declared. Nothing tells
    /// `relation_law` which tier is sand; quartz's own chart values refuse a cohesive
    /// law at a 0.5 mm cell by two orders of magnitude, and contact is what remains.
    #[test]
    fn the_sandbox_earns_its_contact_only_verdict() {
        let sandbox = tier(TierId::Sandbox);
        let grain_mass = mass_per_constituent_kg(&sandbox);
        assert_eq!(
            relation_law(&sandbox, sandbox.g0_m, grain_mass),
            Err(LawRefusal::UnderResolved),
            "a 0.5 mm quartz cell must be refused a cohesive law by the homogenizer"
        );
        // The tier's declared evaluator must agree with what the arithmetic says.
        assert_eq!(sandbox.evaluator, Evaluator::GranularContact);

        // The margin, so the verdict is not resting on a rounding edge: quartz's
        // 2*l_ch is ~4.6 um and the cell is 0.5 mm.
        let material = sandbox.material.unwrap();
        let two_ell_ch = 2.0 * material.young_modulus_pa * material.fracture_energy_j_m2
            / (material.tensile_strength_pa * material.tensile_strength_pa);
        assert!(
            sandbox.g0_m / two_ell_ch > 50.0,
            "the sandbox cell should be far above 2*l_ch; ratio {:.1}",
            sandbox.g0_m / two_ell_ch
        );
    }

    /// And the declaration is checked in BOTH directions: a tier that declares a
    /// cohesive evaluator must actually be able to derive one at its own grain.
    #[test]
    fn cohesive_tiers_can_derive_a_law_at_their_own_grain() {
        for id in TierId::ALL {
            let tier = tier(id);
            if tier.evaluator != Evaluator::Cohesive {
                continue;
            }
            let mass = mass_per_constituent_kg(&tier);
            assert!(
                relation_law(&tier, tier.g0_m, mass).is_ok(),
                "{} declares a cohesive evaluator but cannot derive a law at its own \
                 grain {:e} m",
                tier.name,
                tier.g0_m
            );
        }
    }

    /// The sandbox chart writes NOTHING into the occupancy and momentum lanes, which is
    /// the only reason `tier`'s headline cap is the constituent one. If a future chart
    /// starts writing REG+ occupancy here, the binding lane moves and the cap tightens
    /// six-fold — so this is a gate, not an observation.
    #[test]
    fn the_sandbox_chart_writes_no_occupancy() {
        let sandbox = tier(TierId::Sandbox);
        let mut arena = root_scene(&sandbox).unwrap();
        assert_eq!(arena.holons()[0].gross.occupancy, 0);
        assert_eq!(arena.holons()[0].gross.momentum, [0, 0]);

        let matter_line = sandbox.fill * sandbox.domain_m;
        let mut materializer = QuadrantMaterializer::new(sandbox.domain_m, matter_line, 0.0);
        // Grow several levels, so this covers the generator and not just the root.
        for holon in 0..12 {
            let _ = materializer.materialize(&mut arena, holon);
        }
        assert!(arena.len() > 12, "the scene did not grow");
        for (id, holon) in arena.holons().iter().enumerate() {
            assert_eq!(
                holon.gross.occupancy, 0,
                "holon {id} wrote {} into the occupancy lane; the tier module's cap \
                 headline assumes this lane stays empty",
                holon.gross.occupancy
            );
            assert_eq!(holon.gross.momentum, [0, 0], "holon {id} wrote momentum");
        }
    }

    #[test]
    fn a_tier_terminal_holon_weighs_what_its_grain_weighs() {
        let sandbox = tier(TierId::Sandbox);
        let grain_kg = mass_per_constituent_kg(&sandbox);
        // A 0.5 mm quartz sphere is 1.734e-7 kg.
        assert!(
            (grain_kg / 1.734e-7 - 1.0).abs() < 0.01,
            "one grain should weigh ~1.73e-7 kg, got {grain_kg:e}"
        );
        // And the whole sandbox weighs the grain times the ledger, with no separate
        // mass bookkeeping anywhere.
        let box_kg = grain_kg * sandbox.constituents as f64;
        assert!(
            (60.0..160.0).contains(&box_kg),
            "0.6 m of sand at 45% fill should be ~100 kg, got {box_kg:.1} kg"
        );
    }
}
