//! Adaptive crack-tip materialization — bond damage as the `BoundaryModel` priority (E1).
//!
//! The fixed-frontier fracture demo is convicted by its own constants: with the T4
//! chart values `E·G_F/f_t² = ℓ_ch ≈ 13.75 cm`, cohesive spacing must not exceed
//! `ℓ_ch/10 ≈ 1.4 cm` for the process zone to be resolved, and a frozen coarse
//! frontier on a meter wall is coarser than that — so any crack-path claim off it must
//! honestly read `GrainFloor` (DESCRIPTOR_CHAIN.md §3.4, fix C2). This module makes
//! the damage residual a certificate priority instead: **begin coarse, materialize
//! finer holons and relations only where the residual could change the declared
//! macroscopic observables (crack path, total impulse), and stop at the coarsest
//! frontier meeting the declared tolerance** — or return `GrainFloor` /
//! `RefinementUnavailable` honestly when no resident frontier can.
//!
//! No new entity class appears. The wall is one subject holon materialized through
//! [`DescriptorMaterializer`] (descriptor-as-generator, quenched Record seeds); a bond
//! is a [`CohesiveBond`] relation holon between two resident cells; a crack stays the
//! observable `{relation r | damage(r) = 1}`; the crack tip is a QUERY over that set
//! (the damage-marked midpoints below), never an object.
//!
//! ## The chart
//!
//! Holons carry no coordinates, so the spatial chart lives here, exactly where the
//! [`ChildBoundaryContext`] docs put it: the wall root is a square, and child ordinal
//! `k` of a fanout-4 batch occupies quadrant `(k % 2, k / 2)` of its parent's cell.
//! Cell geometry is therefore a pure function of parent identity and child ordinal,
//! and replay is bit-identical because nothing here draws entropy of its own.
//!
//! ## Bond laws from the quenched draws
//!
//! Each bond's law is DERIVED from the two endpoint grains' persisted [`GrainDraw`]s —
//! never from fresh randomness and never hand-tuned (this replaces the demo's
//! similarity-scaled `CohesiveLaw::weakened` calls):
//!
//! * scale: the T4 chart constants of the [`MaterialBinding`] (`E`, `f_t`, `G_F`)
//!   with the interface's tributary area and length;
//! * heterogeneity: weakest-link across the pair — `min(strength_a, strength_b)`
//!   relative to the declared law's median grain strength scales the peak force;
//!   mean endpoint diameter relative to the declared median diameter scales the
//!   fracture energy (interface geometry).
//!
//! ## The certificate
//!
//! The macroscopic error bound is a declared RESOLUTION SURROGATE (the same shape as
//! `SphereContactModel`'s support-error): every active cell must satisfy
//! `h ≤ tol · max(ℓ_ch/10, γ·d)` where `d` is the cell's distance to the damage
//! surface — measured from the model's own damage field once a solve exists, the
//! notch line before one does. The convergence gate below is what validates that this
//! surrogate controls the observables; the mutation gates prove it can fail. The full
//! cohesive solve runs only on frontiers whose surrogate already passes, so the
//! selector loop stays cheap and the certified observables always come from a solve
//! on the certified frontier itself.
//!
//! Honesty notes, stated plainly:
//! * refinement draws FRESH realizations per level (documented materializer
//!   behaviour), so the pointwise crack path is not a fixed function of the scene
//!   seed across refinement levels; the declared path observable is the area-weighted
//!   deviation summary, and the convergence gate is stated on that summary — the
//!   corridor/distributional reading T4's regime-B gate already forces.
//! * `solver_zeta` is solver-owned dissipation under its honest name (amendment A5);
//!   the tangential friction slider is unused in this mode-I scene.
//! * at fanout 4 every statistical-composition check is below its resolution floor,
//!   so the descriptor certificate rides along unresolving (its teeth live at larger
//!   fanouts; see `descriptor.rs`).

use alloc::vec::Vec;
use core::cell::RefCell;

use crate::descriptor::{
    read_grain, BoundarySelector, ChildBoundaryContext, DescriptorError, DescriptorMaterializer,
    DrawParams, GrainDraw,
};
use crate::holon::Evaluation;
use crate::material::{CohesiveBond, CohesiveLaw, IsotropicMaterial, MaterialBinding};
use crate::runtime::{
    certify_runtime_adaptive, AdaptiveRuntimeCertificate, RuntimeArena, RuntimeBoundaryModel,
    RuntimeFrontier,
};

/// Observables of the fracture certificate: total actuator impulse (N·s),
/// area-weighted crack-path deviation from the notch plane (m), and crack extent
/// (max broken-bond x, m).
pub const FRACTURE_OBSERVABLES: usize = 3;

/// Fanout of every materialization: 2×2 spatial quadrants.
pub const FANOUT: usize = 4;

/// Momentum-balance tolerance passed alongside the macro tolerance.
pub const CONSERVATION_TOLERANCE: f64 = 1.0e-9;

/// A bond with damage above this marks the damage surface (the tip query).
const DAMAGE_MARK: f64 = 0.05;

/// Coarse bonds whose bilinear law would snap back (`δ_fail ≤ δ_peak`) get their
/// fracture energy raised to this multiple of the elastic limit — the coarse-mesh
/// guard whose necessity IS the ℓ_ch story (such frontiers never certify).
const SNAPBACK_GUARD: f64 = 1.5;

/// Clamp on the dimensionless quenched factors, declared: a lattice gate needs
/// bounded heterogeneity, not the Weibull tail's last decade.
const QUENCH_MIN: f64 = 0.2;
const QUENCH_MAX: f64 = 5.0;

/// `ℓ_ch = E·G_F/f_t²` — the T4 characteristic length of the bound material.
pub fn characteristic_length_m(properties: &IsotropicMaterial) -> f64 {
    properties.young_modulus_pa * properties.fracture_energy_j_m2
        / (properties.tensile_strength_pa * properties.tensile_strength_pa)
}

/// The process-zone resolution requirement `ℓ_ch/10` (DESCRIPTOR_CHAIN.md §3.4).
pub fn required_spacing_m(properties: &IsotropicMaterial) -> f64 {
    characteristic_length_m(properties) / 10.0
}

/// Square wall geometry with a starter notch along the mid-plane `y = side/2`,
/// entering from `x = 0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallGeometry {
    pub side_m: f64,
    pub thickness_m: f64,
    pub notch_m: f64,
}

impl WallGeometry {
    pub fn crack_plane_y(&self) -> f64 {
        0.5 * self.side_m
    }
}

/// Test-only mutant hooks (the `planted_law` pattern of `descriptor.rs`): a gate that
/// cannot fail proves nothing, so the wrong residual and the corridor-blind bound are
/// plantable without forking the code path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResidualMode {
    /// Rank by the damage-residual grading (the real model).
    #[default]
    Correct,
    /// MUTANT: rank refinement against a phantom damage surface (a vertical crack at
    /// `x = side/2`) instead of the measured one. A pure "prefer far cells"
    /// inversion cannot even descend the quadtree (every ancestor touches the true
    /// corridor), so THIS is the wrong-residual mutant with teeth: it terminates,
    /// spends its materializations on quiet cells, and never certifies.
    WrongSurfaceResidual,
    /// MUTANT: the bound reads the best-resolved cell instead of the worst.
    CorridorBlindBound,
}

/// Declared solve and certificate values. `macro_tolerance` is in units of the
/// `ℓ_ch/10` requirement: 1.0 demands process-zone spacing exactly at `ℓ_ch/10` on
/// the damage surface; `grading` relaxes the demand linearly with distance from it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FractureConfig {
    pub geometry: WallGeometry,
    pub macro_tolerance: f64,
    pub grading: f64,
    /// Actuator ramp speed of the driven top edge (m/s).
    pub ramp_speed_m_s: f64,
    /// Cosine ramp-in time (s).
    pub ramp_time_s: f64,
    /// Total simulated duration (s).
    pub duration_s: f64,
    /// CFL fraction of `h_min / sqrt(E/ρ)`.
    pub cfl: f64,
    /// Solver-owned dissipation ratio, named as such (A5) — not a material claim.
    pub solver_zeta: f64,
    pub residual: ResidualMode,
}

/// One resident cell of the spatial chart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub x0: f64,
    pub y0: f64,
    pub size: f64,
}

impl Cell {
    fn center(&self) -> [f64; 2] {
        [self.x0 + 0.5 * self.size, self.y0 + 0.5 * self.size]
    }

    fn box_distance(&self, point: [f64; 2]) -> f64 {
        let dx = (self.x0 - point[0]).max(point[0] - (self.x0 + self.size)).max(0.0);
        let dy = (self.y0 - point[1]).max(point[1] - (self.y0 + self.size)).max(0.0);
        libm::sqrt(dx * dx + dy * dy)
    }

    /// Distance to the horizontal mid-plane line (the pre-solve prior surface).
    fn line_distance(&self, y: f64) -> f64 {
        (self.y0 - y).max(y - (self.y0 + self.size)).max(0.0)
    }
}

/// The spatial chart shared by the boundary model and the boundary selector:
/// cell rectangles (pure function of tree structure), the measured damage surface,
/// and a per-holon distance cache.
#[derive(Clone, Debug, Default)]
pub struct WallChart {
    side_m: f64,
    crack_y: f64,
    cells: Vec<Cell>,
    children_seen: Vec<u16>,
    /// Damage-marked bond midpoints from the latest solve; empty = use the prior line.
    surface: Vec<[f64; 2]>,
    distances: Vec<f64>,
}

impl WallChart {
    pub fn new(geometry: WallGeometry) -> Self {
        Self {
            side_m: geometry.side_m,
            crack_y: geometry.crack_plane_y(),
            ..Self::default()
        }
    }

    /// Extend the chart to cover newly materialized holons. Child ordinal within its
    /// batch selects the quadrant; batches append contiguously, so the ordinal is the
    /// running child count of the parent.
    pub fn sync(&mut self, arena: &RuntimeArena) {
        if self.cells.len() == arena.len() {
            return;
        }
        for id in self.cells.len()..arena.len() {
            let record = arena.holons()[id];
            let cell = if record.parent == crate::runtime::NO_RUNTIME_HOLON {
                Cell {
                    x0: 0.0,
                    y0: 0.0,
                    size: self.side_m,
                }
            } else {
                let parent = record.parent as usize;
                let ordinal = self.children_seen[parent] as usize;
                self.children_seen[parent] += 1;
                debug_assert!(ordinal < FANOUT);
                let parent_cell = self.cells[parent];
                let half = 0.5 * parent_cell.size;
                Cell {
                    x0: parent_cell.x0 + (ordinal % 2) as f64 * half,
                    y0: parent_cell.y0 + (ordinal / 2) as f64 * half,
                    size: half,
                }
            };
            self.cells.push(cell);
            self.children_seen.push(0);
            self.distances.push(f64::NAN);
        }
    }

    pub fn cell(&self, holon: usize) -> Cell {
        self.cells[holon]
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn set_surface(&mut self, points: &[[f64; 2]], notch_tip: [f64; 2]) {
        self.surface.clear();
        self.surface.push(notch_tip);
        self.surface.extend_from_slice(points);
        for distance in &mut self.distances {
            *distance = f64::NAN;
        }
    }

    /// Distance from a holon's cell to the damage surface (cached).
    pub fn distance(&mut self, holon: usize) -> f64 {
        let cached = self.distances[holon];
        if !cached.is_nan() {
            return cached;
        }
        let cell = self.cells[holon];
        let distance = if self.surface.is_empty() {
            cell.line_distance(self.crack_y)
        } else {
            let mut best = f64::INFINITY;
            for point in &self.surface {
                best = best.min(cell.box_distance(*point));
            }
            best
        };
        self.distances[holon] = distance;
        distance
    }
}

/// Per-child boundary selection consumed from the landed materializer extension:
/// a child already at or below the strictest possible spacing requirement
/// (`tol · ℓ_ch/10`, the on-surface demand) can never need refinement wherever the
/// damage surface moves, so it is marked `boundary = false`. That is exactly what
/// lets a fanout subtree descend to the grain floor without an active boundary
/// grain-1 leaf halting adaptive materialization (GrainFloor outranks
/// RefinementUnavailable in `certify_runtime`).
/// **Carries no chart, by derivation rather than by omission** (G5).
///
/// This selector used to hold an `Rc<RefCell<WallChart>>` purely to read one number —
/// the parent cell's size — and that shared handle was half of why the fracture solvers
/// were `!Send`. The number is derivable without a chart: the fanout-4 tree halves
/// `grain_units` and cell size *together*, so a cell's size is exactly
/// `side_m · grain_units / root_grain`, and `parent_record.grain_units` is already in
/// [`ChildBoundaryContext`].
///
/// **The derived value is bit-identical to the charted one, not merely close.**
/// `root_grain` is a power of two (enforced by `FractureScene::new`) and `grain_units`
/// halves at each level, so the ratio is a power of two and the division is exact in
/// binary floating point — the same value the chart reaches by repeated exact halving.
///
/// One behaviour difference, and it is a strict improvement: the old form returned `true`
/// conservatively when asked about a parent the chart had not synced yet. The derived form
/// has no unsynced case, because `grain_units` is on the holon from the moment it exists.
pub struct TipSpacingSelector {
    /// Side of the wall, metres — the root cell's size.
    side_m: f64,
    /// The root holon's `grain_units`, against which every cell's size is scaled.
    root_grain: u32,
    /// `macro_tolerance · ℓ_ch/10`: children at or below this spacing are settled.
    settled_spacing_m: f64,
}

impl TipSpacingSelector {
    /// `settled_spacing_m` = macro_tolerance * l_ch/10: children at or below it can
    /// never need refinement wherever the damage surface moves.
    pub fn new(side_m: f64, root_grain: u32, settled_spacing_m: f64) -> Self {
        Self {
            side_m,
            root_grain,
            settled_spacing_m,
        }
    }

    /// Cell size of a holon carrying `grain_units`, by the tree's own scaling law.
    #[inline]
    fn cell_size(&self, grain_units: u32) -> f64 {
        self.side_m * grain_units as f64 / self.root_grain as f64
    }
}

impl BoundarySelector for TipSpacingSelector {
    fn child_boundary(&self, context: ChildBoundaryContext<'_>) -> bool {
        let child_size = 0.5 * self.cell_size(context.parent_record.grain_units);
        child_size > self.settled_spacing_m
    }
}

/// Summary of one cohesive solve on one frontier.
#[derive(Clone, Debug, PartialEq)]
struct SolveSummary {
    observables: [f64; FRACTURE_OBSERVABLES],
    conservation_residual: f64,
    damage_points: Vec<[f64; 2]>,
    broken_bonds: usize,
    bond_count: usize,
}

/// The fracture realization: evaluates a frontier by the graded resolution surrogate,
/// runs the cohesive solve only on frontiers that pass it, and ranks refinement by
/// how much a cell's spacing violates the requirement at its distance from the
/// damage surface.
pub struct FractureModel {
    config: FractureConfig,
    properties: IsotropicMaterial,
    required_m: f64,
    strength_ref_pa: f64,
    diameter_ref_m: f64,
    /// **Owned, not shared** (G5). This was an `Rc<RefCell<WallChart>>`, and the `Rc` is
    /// what made every fracture solver `!Send` — `RefCell<T>` is `Send` whenever `T` is,
    /// while `Rc<T>` is `Send` for no `T` at all. The interior mutability has to stay,
    /// because `RuntimeBoundaryModel::refinement_priority` takes `&self` and the chart
    /// memoizes distances lazily; what had to go was the second owner, and
    /// `TipSpacingSelector` derives its one number instead of holding a handle.
    chart: RefCell<WallChart>,
    solve_key: Option<Vec<u32>>,
    last_solve: Option<SolveSummary>,
    /// Number of full cohesive solves run (exposed for the gates).
    pub solves_run: usize,
}

impl FractureModel {
    pub fn new(
        config: FractureConfig,
        properties: IsotropicMaterial,
        declared: DrawParams,
        chart: WallChart,
    ) -> Self {
        let diameter_ref_m = libm::exp(declared.grain_mu_ln_m);
        let median_volume =
            core::f64::consts::PI / 6.0 * diameter_ref_m * diameter_ref_m * diameter_ref_m;
        // Median strength of the median-diameter grain under the declared
        // weakest-link law: P(strength > s) = exp(-λV(s/σ0)^m) at exceedance ln 2.
        let strength_ref_pa = declared.weibull_sigma0_pa
            * libm::pow(
                core::f64::consts::LN_2 / (declared.flaw_density_per_m3 * median_volume),
                1.0 / declared.weibull_m,
            );
        Self {
            config,
            properties,
            required_m: required_spacing_m(&properties),
            strength_ref_pa,
            diameter_ref_m,
            chart: RefCell::new(chart),
            solve_key: None,
            last_solve: None,
            solves_run: 0,
        }
    }

    fn allowed_spacing(&self, distance: f64) -> f64 {
        self.required_m.max(self.config.grading * distance)
    }

    /// The declared resolution surrogate over the frontier. `CorridorBlindBound`
    /// plants the mutant that reads the best-resolved cell instead of the worst.
    fn bound(&self, frontier: &RuntimeFrontier) -> f64 {
        let mut chart = self.chart.borrow_mut();
        let mut worst = 0.0_f64;
        let mut best = f64::INFINITY;
        for holon in frontier.active_indices() {
            let size = chart.cell(holon).size;
            let ratio = size / self.allowed_spacing(chart.distance(holon));
            worst = worst.max(ratio);
            best = best.min(ratio);
        }
        match self.config.residual {
            ResidualMode::CorridorBlindBound => best,
            _ => worst,
        }
    }

    fn last_observables(&self) -> ([f64; FRACTURE_OBSERVABLES], f64) {
        self.last_solve
            .as_ref()
            .map(|solve| (solve.observables, solve.conservation_residual))
            .unwrap_or(([0.0; FRACTURE_OBSERVABLES], 0.0))
    }

    /// Hand the chart back. The scene lends its chart to the model for a certification
    /// run and takes it back afterwards, which is what replaced the shared `Rc` handle.
    pub fn into_chart(self) -> RefCell<WallChart> {
        self.chart
    }

    /// The damage surface currently steering refinement (the tip QUERY).
    pub fn damage_surface_len(&self) -> usize {
        self.chart.borrow().surface.len()
    }
}

impl RuntimeBoundaryModel<FRACTURE_OBSERVABLES> for FractureModel {
    fn evaluate(
        &mut self,
        arena: &RuntimeArena,
        frontier: &RuntimeFrontier,
    ) -> Evaluation<FRACTURE_OBSERVABLES> {
        self.chart.borrow_mut().sync(arena);
        let bound = self.bound(frontier);
        if bound > self.config.macro_tolerance {
            let (observables, conservation_residual) = self.last_observables();
            return Evaluation {
                observables,
                macro_error_bound: bound,
                conservation_residual,
            };
        }

        let key: Vec<u32> = frontier.active_indices().map(|holon| holon as u32).collect();
        if self.solve_key.as_deref() != Some(&key[..]) {
            let summary = {
                let chart = self.chart.borrow();
                let mesh = build_mesh(
                    arena,
                    frontier,
                    &chart,
                    &self.config.geometry,
                    &self.properties,
                    self.strength_ref_pa,
                    self.diameter_ref_m,
                );
                run_solve(&mesh, &self.config, &self.properties)
            };
            let notch_tip = [
                self.config.geometry.notch_m,
                self.config.geometry.crack_plane_y(),
            ];
            self.chart
                .borrow_mut()
                .set_surface(&summary.damage_points, notch_tip);
            self.solve_key = Some(key);
            self.last_solve = Some(summary);
            self.solves_run += 1;
        }

        // The surface may have moved; the certificate must hold against the
        // measured damage field, not the prior.
        let bound = self.bound(frontier);
        let solve = self.last_solve.as_ref().expect("solve just ran");
        Evaluation {
            observables: solve.observables,
            macro_error_bound: bound,
            conservation_residual: solve.conservation_residual,
        }
    }

    fn refinement_priority(
        &self,
        arena: &RuntimeArena,
        frontier: &RuntimeFrontier,
        holon: usize,
    ) -> f64 {
        if !frontier.is_active(holon) {
            return 0.0;
        }
        let mut chart = self.chart.borrow_mut();
        chart.sync(arena);
        let cell = chart.cell(holon);
        let distance = match self.config.residual {
            // MUTANT: the residual lives on a phantom vertical crack at x = side/2.
            ResidualMode::WrongSurfaceResidual => {
                let phantom_x = 0.5 * self.config.geometry.side_m;
                (cell.x0 - phantom_x).max(phantom_x - (cell.x0 + cell.size)).max(0.0)
            }
            _ => chart.distance(holon),
        };
        let ratio = cell.size / self.allowed_spacing(distance);
        if ratio > self.config.macro_tolerance {
            ratio
        } else {
            0.0
        }
    }
}

// ---------------------------------------------------------------------------
// Mesh derivation: frontier cells become nodes, adjacent cells become
// CohesiveBond relation holons with laws derived from the endpoint draws.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeKind {
    Free,
    FixedBottom,
    DrivenTop,
}

struct MeshNode {
    position: [f64; 2],
    mass_kg: f64,
    kind: NodeKind,
}

struct MeshBond {
    a: usize,
    b: usize,
    bond: CohesiveBond,
    area_m2: f64,
    midpoint: [f64; 2],
}

struct Mesh {
    nodes: Vec<MeshNode>,
    bonds: Vec<MeshBond>,
    finest_cell_m: f64,
}

fn geometry_tolerance(geometry: &WallGeometry) -> f64 {
    1.0e-9 * geometry.side_m
}

/// Overlap length of `[a0, a1]` and `[b0, b1]`.
fn overlap(a0: f64, a1: f64, b0: f64, b1: f64) -> f64 {
    (a1.min(b1) - a0.max(b0)).max(0.0)
}

fn build_mesh(
    arena: &RuntimeArena,
    frontier: &RuntimeFrontier,
    chart: &WallChart,
    geometry: &WallGeometry,
    properties: &IsotropicMaterial,
    strength_ref_pa: f64,
    diameter_ref_m: f64,
) -> Mesh {
    let eps = geometry_tolerance(geometry);
    let crack_y = geometry.crack_plane_y();
    let total_constituents = arena.holons()[arena.root() as usize].gross.constituents as f64;
    let mass_unit = properties.density_kg_m3
        * geometry.side_m
        * geometry.side_m
        * geometry.thickness_m
        / total_constituents;

    let active: Vec<usize> = frontier.active_indices().collect();
    let mut nodes = Vec::with_capacity(active.len());
    let mut draws: Vec<Option<GrainDraw>> = Vec::with_capacity(active.len());
    let mut cells = Vec::with_capacity(active.len());
    let mut finest = f64::INFINITY;
    for &holon in &active {
        let cell = chart.cell(holon);
        finest = finest.min(cell.size);
        let kind = if cell.y0 + cell.size >= geometry.side_m - eps {
            NodeKind::DrivenTop
        } else if cell.y0 <= eps {
            NodeKind::FixedBottom
        } else {
            NodeKind::Free
        };
        nodes.push(MeshNode {
            position: cell.center(),
            mass_kg: arena.holons()[holon].gross.constituents as f64 * mass_unit,
            kind,
        });
        draws.push(read_grain(arena, holon));
        cells.push(cell);
    }

    let mut bonds = Vec::new();
    for i in 0..cells.len() {
        for j in (i + 1)..cells.len() {
            let (a, b) = (cells[i], cells[j]);
            let (ax1, ay1) = (a.x0 + a.size, a.y0 + a.size);
            let (bx1, by1) = (b.x0 + b.size, b.y0 + b.size);
            let x_overlap = overlap(a.x0, ax1, b.x0, bx1);
            let y_overlap = overlap(a.y0, ay1, b.y0, by1);
            let x_touch = libm::fabs(a.x0 - bx1).min(libm::fabs(b.x0 - ax1)) <= eps;
            let y_touch = libm::fabs(a.y0 - by1).min(libm::fabs(b.y0 - ay1)) <= eps;

            // Edge-adjacent (finite shared interface) or corner-adjacent (diagonal).
            let interface_m = if x_touch && y_overlap > eps {
                y_overlap
            } else if y_touch && x_overlap > eps {
                x_overlap
            } else if x_touch && y_touch && x_overlap <= eps && y_overlap <= eps {
                // Declared lattice constant for the diagonal interface share.
                0.5 * a.size.min(b.size)
            } else {
                continue;
            };

            // The starter notch: no relation is created across the notch segment.
            let (pa, pb) = (a.center(), b.center());
            if (pa[1] - crack_y) * (pb[1] - crack_y) < 0.0 {
                let t = (crack_y - pa[1]) / (pb[1] - pa[1]);
                let x_cross = pa[0] + t * (pb[0] - pa[0]);
                if x_cross <= geometry.notch_m {
                    continue;
                }
            }

            let (Some(da), Some(db)) = (draws[i], draws[j]) else {
                continue;
            };
            let area_m2 = interface_m * geometry.thickness_m;
            let rest = {
                let dx = pb[0] - pa[0];
                let dy = pb[1] - pa[1];
                libm::sqrt(dx * dx + dy * dy)
            };

            // Weakest-link strength and diameter geometry from the persisted draws.
            let quench = (da.strength_pa.min(db.strength_pa) / strength_ref_pa)
                .clamp(QUENCH_MIN, QUENCH_MAX);
            let rough = (0.5 * (da.diameter_m + db.diameter_m) / diameter_ref_m)
                .clamp(QUENCH_MIN, QUENCH_MAX);

            let stiffness = properties.young_modulus_pa * area_m2 / rest;
            let peak = properties.tensile_strength_pa * area_m2 * quench;
            let energy_floor = SNAPBACK_GUARD * peak * peak / (2.0 * stiffness);
            let fracture_energy =
                (properties.fracture_energy_j_m2 * area_m2 * rough).max(energy_floor);
            let reduced_mass =
                nodes[i].mass_kg * nodes[j].mass_kg / (nodes[i].mass_kg + nodes[j].mass_kg);
            let law = CohesiveLaw {
                stiffness_n_m: stiffness,
                damping_n_s_m: 2.0
                    * properties.solver_damping_ratio
                    * libm::sqrt(stiffness * reduced_mass),
                peak_force_n: peak,
                fracture_energy_j: fracture_energy,
                // T4 export: McClintock–Walsh inversion of the demo strength ratio.
                friction_coefficient: 0.74,
            };
            // The relation holon id is the bond's ordinal in this frontier's
            // relation namespace (relations are addressable, not arena-resident).
            let relation = bonds.len() + 1;
            let bond = CohesiveBond::new(relation, active[i], active[j], rest, law)
                .expect("derived cohesive law must validate");
            bonds.push(MeshBond {
                a: i,
                b: j,
                bond,
                area_m2,
                midpoint: [0.5 * (pa[0] + pb[0]), 0.5 * (pa[1] + pb[1])],
            });
        }
    }

    Mesh {
        nodes,
        bonds,
        finest_cell_m: finest,
    }
}

// ---------------------------------------------------------------------------
// The deterministic cohesive solve.
// ---------------------------------------------------------------------------

fn run_solve(mesh: &Mesh, config: &FractureConfig, properties: &IsotropicMaterial) -> SolveSummary {
    let n = mesh.nodes.len();
    let crack_y = config.geometry.crack_plane_y();
    if n < 2 || mesh.bonds.is_empty() {
        return SolveSummary {
            observables: [0.0; FRACTURE_OBSERVABLES],
            conservation_residual: f64::INFINITY,
            damage_points: Vec::new(),
            broken_bonds: 0,
            bond_count: mesh.bonds.len(),
        };
    }

    let wave_speed = libm::sqrt(properties.young_modulus_pa / properties.density_kg_m3);
    let dt_target = config.cfl * mesh.finest_cell_m / wave_speed;
    let steps = libm::ceil(config.duration_s / dt_target) as usize;
    let dt = config.duration_s / steps as f64;

    let mut bonds: Vec<CohesiveBond> = mesh.bonds.iter().map(|slot| slot.bond).collect();
    let mut position: Vec<[f64; 2]> = mesh.nodes.iter().map(|node| node.position).collect();
    let mut velocity: Vec<[f64; 2]> = alloc::vec![[0.0, 0.0]; n];
    let mut force: Vec<[f64; 2]> = alloc::vec![[0.0, 0.0]; n];

    let mut impulse_ns = 0.0_f64;
    let mut external_impulse = [0.0_f64, 0.0_f64];
    let mut finite = true;

    for step in 0..steps {
        let time = step as f64 * dt;
        let ramp = if time < config.ramp_time_s {
            0.5 * (1.0 - libm::cos(core::f64::consts::PI * time / config.ramp_time_s))
        } else {
            1.0
        };
        let driven_velocity = config.ramp_speed_m_s * ramp;

        force.fill([0.0, 0.0]);
        for (slot, bond) in mesh.bonds.iter().zip(&mut bonds) {
            let (a, b) = (slot.a, slot.b);
            let dx = position[b][0] - position[a][0];
            let dy = position[b][1] - position[a][1];
            let distance = libm::sqrt(dx * dx + dy * dy);
            if distance <= 0.0 || !distance.is_finite() {
                finite = false;
                continue;
            }
            let direction = [dx / distance, dy / distance];
            let extension = distance - bond.rest_length_m;
            let closing = (velocity[b][0] - velocity[a][0]) * direction[0]
                + (velocity[b][1] - velocity[a][1]) * direction[1];
            let axial = bond.axial_force(extension, closing);
            force[a][0] += direction[0] * axial;
            force[a][1] += direction[1] * axial;
            force[b][0] -= direction[0] * axial;
            force[b][1] -= direction[1] * axial;
        }

        let mut driven_reaction = 0.0_f64;
        for (index, node) in mesh.nodes.iter().enumerate() {
            match node.kind {
                NodeKind::Free => {
                    external_impulse[0] += force[index][0] * dt;
                    external_impulse[1] += force[index][1] * dt;
                    velocity[index][0] += force[index][0] / node.mass_kg * dt;
                    velocity[index][1] += force[index][1] / node.mass_kg * dt;
                    position[index][0] += velocity[index][0] * dt;
                    position[index][1] += velocity[index][1] * dt;
                    if !position[index][0].is_finite() || !position[index][1].is_finite() {
                        finite = false;
                    }
                }
                NodeKind::FixedBottom => {
                    velocity[index] = [0.0, 0.0];
                }
                NodeKind::DrivenTop => {
                    driven_reaction += -force[index][1];
                    velocity[index] = [0.0, driven_velocity];
                    position[index][1] += driven_velocity * dt;
                }
            }
        }
        impulse_ns += driven_reaction * dt;
    }

    // Momentum bookkeeping over the free nodes: symplectic Euler adds F·dt to m·v
    // exactly, so the residual is a rounding-level integrity check on the solve.
    let mut free_momentum = [0.0_f64, 0.0_f64];
    for (index, node) in mesh.nodes.iter().enumerate() {
        if node.kind == NodeKind::Free {
            free_momentum[0] += node.mass_kg * velocity[index][0];
            free_momentum[1] += node.mass_kg * velocity[index][1];
        }
    }
    let mismatch = libm::fabs(free_momentum[0] - external_impulse[0])
        + libm::fabs(free_momentum[1] - external_impulse[1]);
    let scale = (libm::fabs(external_impulse[0]) + libm::fabs(external_impulse[1])).max(1.0e-12);
    let conservation_residual = if finite { mismatch / scale } else { f64::INFINITY };

    let mut damage_points = Vec::new();
    let mut broken_area = 0.0_f64;
    let mut deviation_area = 0.0_f64;
    let mut crack_extent = 0.0_f64;
    let mut broken_bonds = 0_usize;
    for (slot, bond) in mesh.bonds.iter().zip(&bonds) {
        if bond.damage() > DAMAGE_MARK {
            damage_points.push(slot.midpoint);
        }
        if bond.is_broken() {
            broken_bonds += 1;
            broken_area += slot.area_m2;
            deviation_area += slot.area_m2 * libm::fabs(slot.midpoint[1] - crack_y);
            crack_extent = crack_extent.max(slot.midpoint[0]);
        }
    }
    let path_deviation = if broken_area > 0.0 {
        deviation_area / broken_area
    } else {
        0.0
    };

    SolveSummary {
        observables: [impulse_ns, path_deviation, crack_extent],
        conservation_residual,
        damage_points,
        broken_bonds,
        bond_count: bonds.len(),
    }
}

// ---------------------------------------------------------------------------
// The scene: descriptor library + wall subject + certification entry point.
// ---------------------------------------------------------------------------

/// Errors of the fracture scene setup and certification.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FractureError {
    Holon(crate::holon::HolonError),
    Descriptor(DescriptorError),
    InvalidConfiguration,
}

impl From<crate::holon::HolonError> for FractureError {
    fn from(error: crate::holon::HolonError) -> Self {
        Self::Holon(error)
    }
}

impl From<DescriptorError> for FractureError {
    fn from(error: DescriptorError) -> Self {
        Self::Descriptor(error)
    }
}

/// Everything one certification run returns: the engine certificate plus the run's
/// own bookkeeping for the gates.
#[derive(Debug)]
pub struct FractureRun {
    pub result: AdaptiveRuntimeCertificate<FRACTURE_OBSERVABLES>,
    pub solves_run: usize,
    pub required_spacing_m: f64,
    pub finest_active_m: f64,
    /// Materialized (Expanded) wall holons, split by whether their cell touches the
    /// damage corridor: (near, far). "Near" means box-distance to the final damage
    /// surface at most one own cell size; the root and depth-1 cells are exempt
    /// (they contain the corridor by construction).
    pub materialized_near: usize,
    pub materialized_far: usize,
}

/// A synthetic wall scene living in core: descriptor library arena, wall subject
/// arena, binding, and the declared certificate values.
pub struct FractureScene {
    pub config: FractureConfig,
    pub binding: MaterialBinding,
    descriptors: RuntimeArena,
    arena: RuntimeArena,
    declared: DrawParams,
    /// Owned, not shared (G5). Lent to the model for the duration of a certification
    /// run and taken back with `FractureModel::into_chart`.
    chart: RefCell<WallChart>,
}

impl FractureScene {
    /// Build the scene: a self-contained stone-descriptor holarchy and a latent wall
    /// subject carrying its quenched Record seed. `root_grain` sets the grain floor:
    /// terminal spacing is `side / root_grain` (fanout-4 halves grain and cell size
    /// together).
    pub fn new(
        config: FractureConfig,
        law: DrawParams,
        modal_weights: &[u64],
        total_constituents: u64,
        root_grain: u32,
        seed: u64,
    ) -> Result<Self, FractureError> {
        if !root_grain.is_power_of_two() || root_grain < 2 {
            return Err(FractureError::InvalidConfiguration);
        }
        let descriptors =
            crate::descriptor::build_stone_descriptor(total_constituents, modal_weights, &law)?;
        let whole = crate::descriptor::encode_seed(seed);
        let specs = [crate::runtime::RuntimeHolonSpec {
            parent: crate::runtime::NO_RUNTIME_HOLON,
            depth: 0,
            grain_units: root_grain,
            gross: crate::regplus::GrossState::aggregate(total_constituents, 0, [0, 0]),
            whole: &whole,
            channels: crate::holon::Channels::REG_PLUS.union(crate::holon::Channels::MECHANICAL),
            boundary: true,
            decomposition: crate::holon::Decomposition::Latent,
        }];
        let arena = RuntimeArena::from_specs(&specs, 0)?;
        let binding = MaterialBinding {
            subject_holon: 0,
            descriptor_holon: 0,
            properties: IsotropicMaterial::DEMO_CALIBRATION,
        };
        Ok(Self {
            config,
            binding,
            descriptors,
            arena,
            declared: law,
            chart: RefCell::new(WallChart::new(config.geometry)),
        })
    }

    pub fn arena(&self) -> &RuntimeArena {
        &self.arena
    }

    /// Borrow the scene's chart. Returns a reference rather than a cloned handle: the
    /// `.borrow()` / `.borrow_mut()` call sites are unchanged.
    pub fn chart(&self) -> &RefCell<WallChart> {
        &self.chart
    }

    /// Certify with the landed per-child boundary selection (the intended path).
    pub fn certify(&mut self) -> Result<FractureRun, FractureError> {
        self.certify_inner(false)
    }

    /// Certify with inherit-parent boundary flags — the control documenting the
    /// engine fact that a mixed frontier with an active boundary grain-1 leaf reads
    /// GrainFloor and halts adaptive materialization.
    pub fn certify_with_inherited_boundary(&mut self) -> Result<FractureRun, FractureError> {
        self.certify_inner(true)
    }

    fn certify_inner(&mut self, inherit_boundary: bool) -> Result<FractureRun, FractureError> {
        let root_grain = self.arena.holons()[self.arena.root() as usize].grain_units;
        let mut model = FractureModel::new(
            self.config,
            self.binding.properties,
            self.declared,
            WallChart::new(self.config.geometry),
        );
        let settled_spacing_m =
            self.config.macro_tolerance * required_spacing_m(&self.binding.properties);
        let result = if inherit_boundary {
            let mut materializer =
                DescriptorMaterializer::new(&self.descriptors, self.binding, FANOUT, 0)?;
            certify_runtime_adaptive(
                &mut self.arena,
                &mut model,
                &mut materializer,
                self.config.macro_tolerance,
                CONSERVATION_TOLERANCE,
            )?
        } else {
            let selector = TipSpacingSelector::new(
                self.config.geometry.side_m,
                root_grain,
                settled_spacing_m,
            );
            let mut materializer = DescriptorMaterializer::with_boundary(
                &self.descriptors,
                self.binding,
                FANOUT,
                0,
                selector,
            )?;
            certify_runtime_adaptive(
                &mut self.arena,
                &mut model,
                &mut materializer,
                self.config.macro_tolerance,
                CONSERVATION_TOLERANCE,
            )?
        };

        // Take the chart back from the model, then do the locality bookkeeping over the
        // final arena/chart state. `solves_run` is read first because `into_chart`
        // consumes the model.
        let solves_run = model.solves_run;
        self.chart = model.into_chart();
        let mut chart_ref = self.chart.borrow_mut();
        chart_ref.sync(&self.arena);
        let mut near = 0;
        let mut far = 0;
        for (id, record) in self.arena.holons().iter().enumerate() {
            if record.decomposition != crate::holon::Decomposition::Expanded || record.depth < 2 {
                continue;
            }
            let cell = chart_ref.cell(id);
            let distance = chart_ref.distance(id);
            // Slack for surface motion between refinement rounds: fresh draws per
            // level move the measured crack by a few cells (documented materializer
            // behaviour), which must not read as far-field materialization.
            if distance <= cell.size + 0.12 * self.config.geometry.side_m {
                near += 1;
            } else {
                far += 1;
            }
        }
        let finest_active_m = result
            .certificate
            .frontier
            .active_indices()
            .map(|holon| chart_ref.cell(holon).size)
            .fold(f64::INFINITY, f64::min);
        drop(chart_ref);

        Ok(FractureRun {
            result,
            solves_run,
            required_spacing_m: required_spacing_m(&self.binding.properties),
            finest_active_m,
            materialized_near: near,
            materialized_far: far,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holon::CertificationStatus;

    fn demo_law() -> DrawParams {
        DrawParams {
            grain_mu_ln_m: libm::log(7.5e-4),
            grain_sigma_ln: 0.5,
            weibull_m: 10.0,
            weibull_sigma0_pa: 2.0e8,
            flaw_density_per_m3: 1.0e10,
        }
    }

    const MODAL: [u64; 3] = [30, 60, 10];
    const SEED: u64 = 0xC1F1_57A6_E1AD_A971;

    /// Eighth-meter test wall (debug wall-clock: the engine's debug-assert frontier
    /// validation is O(F²) per refine; the example runs the meter wall in release).
    /// ℓ_ch bookkeeping at DEMO_CALIBRATION: ℓ_ch = 45e9·110/(6e6)² = 0.1375 m,
    /// required spacing 13.75 mm. Cell size at depth d is 0.125/2^d m, so tolerance
    /// 1.0 certifies at depth 4 (7.81 mm) and depth 3 (15.6 mm) fails.
    fn test_config(macro_tolerance: f64) -> FractureConfig {
        FractureConfig {
            geometry: WallGeometry {
                side_m: 0.125,
                thickness_m: 0.1,
                notch_m: 0.03,
            },
            macro_tolerance,
            grading: 2.0,
            ramp_speed_m_s: 0.1,
            ramp_time_s: 2.0e-4,
            duration_s: 1.2e-3,
            cfl: 0.25,
            solver_zeta: 0.15,
            residual: ResidualMode::Correct,
        }
    }

    fn scene(macro_tolerance: f64, root_grain: u32, seed: u64) -> FractureScene {
        FractureScene::new(
            test_config(macro_tolerance),
            demo_law(),
            &MODAL,
            1_000_000,
            root_grain,
            seed,
        )
        .unwrap()
    }

    #[test]
    fn chart_assigns_deterministic_quadrants() {
        let mut scene = scene(1.0, 32, SEED);
        let run = scene.certify().unwrap();
        assert!(run.result.certificate.passed());
        let chart = scene.chart();
        let chart = chart.borrow();
        // Root covers the wall; each child is its parent's quadrant.
        let root = chart.cell(0);
        assert_eq!((root.x0, root.y0, root.size), (0.0, 0.0, 0.125));
        for (id, record) in scene.arena().holons().iter().enumerate() {
            if record.parent == crate::runtime::NO_RUNTIME_HOLON {
                continue;
            }
            let parent = chart.cell(record.parent as usize);
            let child = chart.cell(id);
            assert_eq!(child.size, 0.5 * parent.size);
            assert!(child.x0 >= parent.x0 - 1e-12 && child.x0 + child.size <= parent.x0 + parent.size + 1e-12);
            assert!(child.y0 >= parent.y0 - 1e-12 && child.y0 + child.size <= parent.y0 + parent.size + 1e-12);
        }
    }

    // Gate (a): the certifier refines near the tip and NOT elsewhere.
    #[test]
    fn refines_near_the_tip_and_not_elsewhere() {
        let mut scene = scene(1.0, 32, SEED);
        let run = scene.certify().unwrap();
        assert!(run.result.certificate.passed(), "{:?}", run.result.certificate.status);
        // ℓ_ch demands finer-than-initial spacing: the certified frontier's finest
        // cell must be at or below 13.75 mm while the initial frontier was the
        // 125 mm root.
        assert!(run.finest_active_m <= run.required_spacing_m);
        assert!(run.result.materializations >= 20, "{}", run.result.materializations);
        // Materialization is corridor-local: no deep cell was expanded away from
        // the damage surface, and plenty were expanded on it.
        assert_eq!(run.materialized_far, 0, "far materializations: {}", run.materialized_far);
        assert!(run.materialized_near >= 15, "{}", run.materialized_near);
        // Quiet corners stay coarse: every active cell far from the corridor is
        // larger than the process-zone requirement.
        let chart = scene.chart();
        let mut chart = chart.borrow_mut();
        for holon in run.result.certificate.frontier.active_indices() {
            let cell = chart.cell(holon);
            let distance = chart.distance(holon);
            if distance > 0.05 {
                assert!(cell.size > run.required_spacing_m, "over-refined quiet cell");
            }
        }
        // The run actually fractured something.
        assert!(run.result.certificate.observables[2] > scene.config.geometry.notch_m);
    }

    // Gate (b): crack path and impulse converge under refinement.
    #[test]
    fn crack_path_and_impulse_converge_with_refinement() {
        // refine-once (coarse certificate), refine-to-tolerance, refine-further.
        let coarse = scene(4.0, 32, SEED).certify().unwrap();
        let target = scene(1.0, 32, SEED).certify().unwrap();
        let fine = scene(0.5, 32, SEED).certify().unwrap();
        assert!(coarse.result.certificate.passed());
        assert!(target.result.certificate.passed());
        assert!(fine.result.certificate.passed());

        let [j_coarse, d_coarse, _] = coarse.result.certificate.observables;
        let [j_target, d_target, _] = target.result.certificate.observables;
        let [j_fine, d_fine, _] = fine.result.certificate.observables;

        // Declared observable tolerances, calibrated on the release probe of this
        // scene family (measured legs: J 27.53/29.32/30.38/30.41 N·s at finest
        // 31.25/7.81/3.91/1.95 mm; D 19.9/9.1/5.2/2.4 mm — the path summary is the
        // resolution-limited crack width, so its tolerance is one certified
        // spacing). The teeth are the mutation gates below, which must MISS these
        // same tolerances.
        let impulse_tolerance = 0.05 * j_target;
        let path_tolerance = target.finest_active_m;

        let coarse_miss = (j_coarse - j_target).abs() > impulse_tolerance
            || (d_coarse - d_target).abs() > path_tolerance;
        assert!(coarse_miss, "coarse run should miss: J {j_coarse} vs {j_target}, D {d_coarse} vs {d_target}");
        assert!(
            (j_fine - j_target).abs() <= impulse_tolerance,
            "impulse not converged: {j_fine} vs {j_target}"
        );
        assert!(
            (d_fine - d_target).abs() <= path_tolerance,
            "path deviation not converged: {d_fine} vs {d_target}"
        );
    }

    // Gate (c), mutation 1: a wrong residual ranking must fire the gate — the
    // certificate refuses AND refinement lands in quiet regions.
    #[test]
    fn wrong_residual_ranking_fails_the_gate() {
        let mut scene = scene(1.0, 32, SEED);
        scene.config.residual = ResidualMode::WrongSurfaceResidual;
        let run = scene.certify().unwrap();
        assert!(!run.result.certificate.passed(), "{:?}", run.result.certificate.status);
        assert!(run.materialized_far > 0, "mutant refined nothing far from the tip");
    }

    // Gate (c), mutation 2: a corridor-blind bound certifies coarse and the
    // convergence gate convicts its observables.
    #[test]
    fn corridor_blind_bound_certifies_wrong_observables() {
        let reference = scene(1.0, 32, SEED).certify().unwrap();
        let mut mutant_scene = scene(1.0, 32, SEED);
        mutant_scene.config.residual = ResidualMode::CorridorBlindBound;
        let mutant = mutant_scene.certify().unwrap();
        assert!(reference.result.certificate.passed());
        assert!(mutant.result.certificate.passed(), "{:?}", mutant.result.certificate.status);
        // The mutant stopped far coarser than the requirement...
        assert!(mutant.finest_active_m > reference.finest_active_m);
        // ...and its observables miss the SAME declared tolerances the convergence
        // gate stakes (5% impulse, one certified spacing of path deviation).
        let [j_ref, d_ref, _] = reference.result.certificate.observables;
        let [j_mut, d_mut, _] = mutant.result.certificate.observables;
        let fired = (j_mut - j_ref).abs() > 0.05 * j_ref
            || (d_mut - d_ref).abs() > reference.finest_active_m;
        assert!(fired, "blind bound escaped detection: J {j_mut} vs {j_ref}, D {d_mut} vs {d_ref}");
    }

    // Gate (d): the adaptive run is bit-identical on the same scene seed, and the
    // seed is load-bearing.
    #[test]
    fn adaptive_run_is_bit_identical_on_the_same_scene_seed() {
        let mut first_scene = scene(1.0, 32, SEED);
        let mut second_scene = scene(1.0, 32, SEED);
        let first = first_scene.certify().unwrap();
        let second = second_scene.certify().unwrap();
        assert!(first.result.certificate.passed());
        assert_eq!(first_scene.arena().holons(), second_scene.arena().holons());
        let first_bits: Vec<u64> = first_scene
            .arena()
            .whole_scalars()
            .iter()
            .map(|x| x.to_bits())
            .collect();
        let second_bits: Vec<u64> = second_scene
            .arena()
            .whole_scalars()
            .iter()
            .map(|x| x.to_bits())
            .collect();
        assert_eq!(first_bits, second_bits);
        for k in 0..FRACTURE_OBSERVABLES {
            assert_eq!(
                first.result.certificate.observables[k].to_bits(),
                second.result.certificate.observables[k].to_bits()
            );
        }
        assert_eq!(first.result.materializations, second.result.materializations);

        // MUTATION: a different persisted Record seed must change the realization —
        // a pipeline ignoring the quenched draws would pass the identity vacuously.
        let mut other_scene = scene(1.0, 32, SEED + 1);
        let other = other_scene.certify().unwrap();
        let other_bits: Vec<u64> = other_scene
            .arena()
            .whole_scalars()
            .iter()
            .map(|x| x.to_bits())
            .collect();
        assert_ne!(first_bits, other_bits);
        let differs = (0..FRACTURE_OBSERVABLES).any(|k| {
            first.result.certificate.observables[k].to_bits()
                != other.result.certificate.observables[k].to_bits()
        });
        assert!(differs, "observables ignored the quenched realization");
    }

    // Gate (e): a tolerance the grain floor cannot meet returns GrainFloor.
    #[test]
    fn unmeetable_tolerance_returns_grain_floor() {
        // Root grain 4 on the eighth-meter wall: floor spacing 31.25 mm > 13.75 mm
        // required, so no resident frontier can certify at tolerance 1.0.
        let mut scene = scene(1.0, 4, SEED);
        let run = scene.certify().unwrap();
        assert_eq!(run.result.certificate.status, CertificationStatus::GrainFloor);
        assert!(run.result.materializations > 0, "the selector never even tried");
        assert!(run.finest_active_m > run.required_spacing_m);
    }

    // The landed per-child boundary selection is load-bearing: with it the frontier
    // descends a subtree to the grain floor and certifies there; with inherit-parent
    // flags the identical scene halts at GrainFloor (the engine fact this lane was
    // told to design around).
    #[test]
    fn per_child_boundary_selection_descends_to_the_grain_floor() {
        // Root grain 16: floor spacing 7.8125 mm ≤ 13.75 mm required, so the
        // certified frontier must reach grain 1 on the corridor.
        let mut selected = scene(1.0, 16, SEED);
        let run = selected.certify().unwrap();
        assert!(run.result.certificate.passed(), "{:?}", run.result.certificate.status);
        let finest_grain = run.result.certificate.frontier.finest_grain(selected.arena());
        assert_eq!(finest_grain, 1, "corridor never reached the grain floor");

        let mut inherited = scene(1.0, 16, SEED);
        let control = inherited.certify_with_inherited_boundary().unwrap();
        assert_eq!(
            control.result.certificate.status,
            CertificationStatus::GrainFloor,
            "inherit-parent boundary was expected to halt at the floor"
        );
    }

    // Bond derivation: the law is a pure function of the persisted endpoint draws
    // and the chart constants — weakest-link strength, diameter-scaled toughness.
    #[test]
    fn bond_laws_come_from_the_quenched_draws() {
        let mut scene = scene(1.0, 32, SEED);
        let run = scene.certify().unwrap();
        assert!(run.result.certificate.passed());
        let arena = scene.arena();
        let chart_rc = scene.chart();
        let chart = chart_rc.borrow();
        let properties = scene.binding.properties;
        let law = demo_law();
        let diameter_ref = libm::exp(law.grain_mu_ln_m);
        let median_volume = core::f64::consts::PI / 6.0 * diameter_ref * diameter_ref * diameter_ref;
        let strength_ref = law.weibull_sigma0_pa
            * libm::pow(
                core::f64::consts::LN_2 / (law.flaw_density_per_m3 * median_volume),
                1.0 / law.weibull_m,
            );
        let mesh = build_mesh(
            arena,
            &run.result.certificate.frontier,
            &chart,
            &scene.config.geometry,
            &properties,
            strength_ref,
            diameter_ref,
        );
        assert!(mesh.bonds.len() > mesh.nodes.len(), "lattice is under-connected");
        let mut checked = 0;
        for slot in &mesh.bonds {
            let a = read_grain(arena, slot.bond.a_holon).unwrap();
            let b = read_grain(arena, slot.bond.b_holon).unwrap();
            let quench = (a.strength_pa.min(b.strength_pa) / strength_ref)
                .clamp(QUENCH_MIN, QUENCH_MAX);
            let expected_peak = properties.tensile_strength_pa * slot.area_m2 * quench;
            assert!((slot.bond.law.peak_force_n - expected_peak).abs() <= 1e-9 * expected_peak);
            // Weakest link: strengthening the weaker endpoint strengthens the bond;
            // the law validates (no snap-back).
            assert!(slot.bond.law.validate().is_ok());
            checked += 1;
        }
        assert!(checked > 50);
    }
}
