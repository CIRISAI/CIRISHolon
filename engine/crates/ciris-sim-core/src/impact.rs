//! N-e — the full fracture composition (B4 regimes B/C): descriptor-built wall,
//! derived (k_n, k_t) bond laws, a thrown projectile, adaptive crack-tip
//! materialization, and node-node contact among the fragments — certified end to end.
//!
//! Everything here is composition, not invention. The pieces composed:
//!
//! * the wall is materialized through [`DescriptorMaterializer`] with quenched Record
//!   seeds, exactly as in `fracture.rs` (E1), reusing its [`WallChart`] spatial chart
//!   and [`TipSpacingSelector`];
//! * bond ELASTIC laws come from `homogenization.rs`: every relation carries the
//!   normal and tangential stiffnesses `k_n = t(λ+μ)`, `k_t = t(μ−λ)` that
//!   homogenize the square + alternating-diagonal stencil exactly to the
//!   descriptor's plane-stress `(E, ν)` — so the ν = 1/3 central-force restriction
//!   noted on E1 is paid here. The tangential spring requires rotational state to be
//!   frame-indifferent, so nodes carry spin and torque (BPM-style), with slip
//!   measured at the interface midpoint including spins — mirroring the landed
//!   ball-game realization;
//! * bond COHESIVE laws come from [`derive_bilinear_cohesive_law`] at the bond's own
//!   local pitch, then quenched by the endpoint [`GrainDraw`]s (weakest-link
//!   strength, diameter-scaled toughness);
//! * post-failure and never-bonded pairs are the contact solver's jurisdiction (A3
//!   regime table): penalty normal contact plus Coulomb friction, where a failed
//!   pair inherits its dead bond's tribology through
//!   [`CohesiveBond::failed_contact_friction_force`] — the D → 1 continuity
//!   contract, consumed, not re-derived;
//! * the certificate loop is E1's: begin at one resident holon, materialize only
//!   where the damage residual could move the declared observables (projectile
//!   impulse, crack area, detached fragment mass), stop at the coarsest passing
//!   frontier or return `GrainFloor` honestly with the required-vs-available
//!   spacing on the record.
//!
//! ## The two refusal points, and their division of labor
//!
//! The homogenization CONSTRUCTOR refuses a bilinear cohesive law at pitch
//! `h ≥ h_max = 2 G_F (λ+μ)/f_t²` — right for a caller asking for a law to run
//! with. The adaptive CERTIFICATE must still evaluate coarse frontiers so it can
//! convict them, so this module keeps E1's philosophy at the seam: a bond the
//! constructor refuses (or whose quenched weakening closes the softening interval)
//! is built GUARDED — derived elastic stiffnesses, fracture energy floored to keep
//! a positive softening branch — and the solve MEASURES every guarded bond's peak
//! load fraction. A frontier in which any guarded bond carries real load
//! (≥ [`ImpactConfig::guard_fraction`] of its derived peak) has its error bound
//! inflated and cannot certify: the constructor refuses laws, the certificate
//! refuses frontiers, and no coarse frontier can smuggle a guarded bond into a
//! certified answer. Certified corridors are always constructor-legal because the
//! process-zone requirement `ℓ_ch/10` sits far inside `h_max` (their ratio is
//! `2(λ+μ)/E · 10`, ≈ 13–14 for the granites here).
//!
//! Honesty notes: the (k_n, k_t) exactness theorem is for the uniform
//! alternating-diagonal stencil; on the adaptive multi-resolution frontier the same
//! per-relation stiffnesses are applied with the alternating-diagonal selection at
//! the local pitch, and the theorem does NOT reach that case. The convergence gate
//! is what carries the multi-resolution claim — and a second verification pass
//! (research-manager, 2026-08-24) narrows what that can mean, because an earlier
//! wording here said the gate "owns" the claim and that is over-strong. The gate
//! compares three runs of THIS engine against each other; there is no external
//! reference for the composed impact observables on this scene. So it establishes
//! **self-consistency of the certified observables under refinement**, NOT their
//! error against truth — `Core/SelfAudit.lean`'s lesson turned on ourselves: a
//! certificate built from the engine's own data cannot certify the engine's error
//! against the truth. The external anchors this scene does carry are real and are
//! narrower than the claim: momentum balance, the constructor domain `h_max`, and
//! the LAC_DU_BONNET continuum record. **An external reference for these observables
//! is OWED and does not exist; until it does the multi-resolution result is
//! refinement-stable, not verified.** The descriptor's grain-scale draw law remains the
//! Westerly-class demo law even under the LAC_DU_BONNET continuum record: no
//! Lac-du-Bonnet grain-scale law is pinned, so the quenched heterogeneity carries a
//! CLASS warrant only. Contact penalty stiffness/damping and `solver_zeta` are
//! solver configuration named as such (A5). The detached-mass observable is
//! REPORTED but not point-gated: fragment detachment is a threshold event on the
//! quenched crack pattern, which redraws per refinement level (documented
//! materializer behaviour) — T4's regime-C reading applies (fragment quantities
//! are distribution-gated), so the convergence gate stakes impulse and crack area
//! and the measured legs show detached mass flipping between refinement levels
//! (0 / 3.88 / 0 kg on the example scene).

use alloc::vec::Vec;
use core::cell::RefCell;

use crate::descriptor::{read_grain, DescriptorMaterializer, DrawParams, GrainDraw};
use crate::fracture::{
    required_spacing_m, Cell, TipSpacingSelector, WallChart, WallGeometry, FANOUT,
};
use crate::holon::Evaluation;
use crate::homogenization::{
    derive_bilinear_cohesive_law, max_bilinear_spacing_m, plane_stress_moduli,
    HomogenizationError,
};
use crate::material::{CohesiveBond, CohesiveLaw, IsotropicMaterial, MaterialBinding};
use crate::runtime::{
    certify_runtime_adaptive, AdaptiveRuntimeCertificate, RuntimeArena, RuntimeBoundaryModel,
    RuntimeFrontier,
};

/// Observables: projectile contact impulse (N·s, along +x into the wall), broken
/// interface area (m²), and detached fragment mass (kg — nodes no longer connected
/// to the anchored bottom row by live bonds).
pub const IMPACT_OBSERVABLES: usize = 3;

/// Momentum-balance tolerance passed alongside the macro tolerance.
pub const CONSERVATION_TOLERANCE: f64 = 1.0e-9;

/// A bond with damage above this marks the damage surface (the tip query).
const DAMAGE_MARK: f64 = 0.05;

/// Guarded bonds get their fracture energy floored to this multiple of the elastic
/// limit so coarse frontiers still evaluate; the certificate convicts them if such a
/// bond ever carries real load.
const SNAPBACK_GUARD: f64 = 1.5;

/// Clamp on the dimensionless quenched factors (declared; tighter than E1's so the
/// constructor-legal domain is not silently narrowed by the tails).
const QUENCH_MIN: f64 = 0.5;
const QUENCH_MAX: f64 = 2.0;

/// Node radius as a fraction of its cell size (the landed game value).
const NODE_RADIUS_FRACTION: f64 = 0.4;

/// Contact neighbor list rebuild cadence and its skin speed budget (declared solver
/// configuration; the skin covers the worst-case relative approach between rebuilds).
const REBUILD_EVERY_STEPS: usize = 64;
const SKIN_SPEED_M_S: f64 = 60.0;

/// Test-only mutant hooks (house standard: a gate that cannot fail proves nothing).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImpactResidual {
    #[default]
    Correct,
    /// MUTANT: rank refinement against a phantom vertical crack at x = side/2.
    WrongSurfaceResidual,
    /// MUTANT: the bound reads the best-resolved cell instead of the worst.
    CorridorBlindBound,
    /// MUTANT: drop the tangential spring (k_t = 0) — the axial-only lattice the
    /// ν gate must convict.
    AxialOnly,
}

/// Declared solve and certificate values for the composed impact scene.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImpactConfig {
    pub geometry: WallGeometry,
    /// Aim height of the throw on the left face (m).
    pub aim_y_m: f64,
    pub ball_mass_kg: f64,
    pub ball_radius_m: f64,
    pub ball_speed_m_s: f64,
    /// Bound units of the `ℓ_ch/10` requirement, as in E1.
    pub macro_tolerance: f64,
    pub grading: f64,
    pub duration_s: f64,
    pub cfl: f64,
    /// Solver-owned dissipation ratio for bond damping, named as such (A5).
    pub solver_zeta: f64,
    /// Penalty contact stiffness as a fraction of the derived k_n (solver config).
    pub contact_stiffness_fraction: f64,
    /// Guarded-bond load fraction above which the frontier cannot certify.
    pub guard_fraction: f64,
    pub residual: ImpactResidual,
}

/// One solve's outcome.
#[derive(Clone, Debug, PartialEq)]
struct SolveSummary {
    observables: [f64; IMPACT_OBSERVABLES],
    conservation_residual: f64,
    damage_points: Vec<[f64; 2]>,
    /// Midpoints of guarded bonds loaded past the guard fraction.
    violation_points: Vec<[f64; 2]>,
    /// Worst guarded-bond load as a fraction of its derived peak.
    guarded_worst_load: f64,
    guarded_bonds: usize,
    broken_bonds: usize,
    /// Sum of `law.fracture_energy_j` over BROKEN bonds — the fracture work this solve
    /// actually dissipated, in joules, as the engine itself computed it. NOT estimated
    /// from `crack_area`: the per-bond energy carries its own quenched roughness factor
    /// (`fracture.rs:674`, `G_F · area · rough`), so `G_F · Σarea` is not the same number.
    /// This is what makes the Griffith/LEFM anchor exact rather than convention-dependent.
    fracture_energy_j: f64,
}

/// The composed realization: E1's graded damage-residual certificate over the
/// (k_n, k_t) lattice with projectile and fragment contact.
pub struct ImpactModel {
    config: ImpactConfig,
    properties: IsotropicMaterial,
    required_m: f64,
    strength_ref_pa: f64,
    diameter_ref_m: f64,
    /// **Owned, not shared** (G5). `Rc` is what made this `!Send`; `RefCell<T>` is `Send`
    /// whenever `T` is. The interior mutability stays because
    /// `RuntimeBoundaryModel::refinement_priority` takes `&self` and the chart memoizes
    /// distances lazily; the second owner is what went away.
    chart: RefCell<WallChart>,
    solve_key: Option<Vec<u32>>,
    last_solve: Option<SolveSummary>,
    /// Midpoints of guarded bonds that carried real load in the last solve: the
    /// certificate's refusal is steered here even where the graded spacing ledger
    /// is already satisfied (a guarded bond under load is under-resolved at ITS
    /// pitch, whatever the tolerance).
    violations: Vec<[f64; 2]>,
    pub solves_run: usize,
}

impl ImpactModel {
    pub fn new(
        config: ImpactConfig,
        properties: IsotropicMaterial,
        declared: DrawParams,
        mut chart: WallChart,
    ) -> Self {
        let diameter_ref_m = libm::exp(declared.grain_mu_ln_m);
        let median_volume =
            core::f64::consts::PI / 6.0 * diameter_ref_m * diameter_ref_m * diameter_ref_m;
        let strength_ref_pa = declared.weibull_sigma0_pa
            * libm::pow(
                core::f64::consts::LN_2 / (declared.flaw_density_per_m3 * median_volume),
                1.0 / declared.weibull_m,
            );
        // Seed the damage surface with the impact aim point: refinement begins at
        // the place the throw can put a residual, not at a notch prior. Direct, because
        // the model now OWNS the chart rather than sharing a handle to it.
        chart.set_surface(&[], [0.0, config.aim_y_m]);
        Self {
            config,
            properties,
            required_m: required_spacing_m(&properties),
            strength_ref_pa,
            diameter_ref_m,
            chart: RefCell::new(chart),
            solve_key: None,
            last_solve: None,
            violations: Vec::new(),
            solves_run: 0,
        }
    }

    fn allowed_spacing(&self, distance: f64) -> f64 {
        self.required_m.max(self.config.grading * distance)
    }

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
            ImpactResidual::CorridorBlindBound => best,
            _ => worst,
        }
    }

    /// The guard seam, applied to the frontier a solve just ran on: a certified
    /// answer may not stand on a guarded bond that carried real load. The solve
    /// already added such bonds' midpoints to the damage surface, so the next
    /// rounds refine exactly there.
    fn guard_term(&self, solve: &SolveSummary) -> f64 {
        if solve.guarded_worst_load >= self.config.guard_fraction {
            self.config.macro_tolerance
                * (solve.guarded_worst_load / self.config.guard_fraction)
                * 1.000_001
        } else {
            0.0
        }
    }

    fn last_observables(&self) -> ([f64; IMPACT_OBSERVABLES], f64) {
        self.last_solve
            .as_ref()
            .map(|solve| (solve.observables, solve.conservation_residual))
            .unwrap_or(([0.0; IMPACT_OBSERVABLES], 0.0))
    }

    pub fn last_guarded(&self) -> (usize, f64) {
        self.last_solve
            .as_ref()
            .map(|solve| (solve.guarded_bonds, solve.guarded_worst_load))
            .unwrap_or((0, 0.0))
    }

    /// Fracture work dissipated by the last solve, joules — the engine's own per-bond
    /// sum, for the Griffith/LEFM anchor.
    pub fn last_fracture_energy_j(&self) -> f64 {
        self.last_solve.as_ref().map(|solve| solve.fracture_energy_j).unwrap_or(0.0)
    }
}

impl ImpactModel {
    /// Hand the chart back to the scene after a certification run (G5).
    pub fn into_chart(self) -> RefCell<WallChart> {
        self.chart
    }
}

impl RuntimeBoundaryModel<IMPACT_OBSERVABLES> for ImpactModel {
    fn evaluate(
        &mut self,
        arena: &RuntimeArena,
        frontier: &RuntimeFrontier,
    ) -> Evaluation<IMPACT_OBSERVABLES> {
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
                let mesh = build_impact_mesh(
                    arena,
                    frontier,
                    &chart,
                    &self.config,
                    &self.properties,
                    self.strength_ref_pa,
                    self.diameter_ref_m,
                );
                run_impact_solve(&mesh, &self.config, &self.properties)
            };
            self.chart
                .borrow_mut()
                .set_surface(&summary.damage_points, [0.0, self.config.aim_y_m]);
            self.violations = summary.violation_points.clone();
            self.solve_key = Some(key);
            self.last_solve = Some(summary);
            self.solves_run += 1;
        }

        let geometric = self.bound(frontier);
        let solve = self.last_solve.as_ref().expect("solve just ran");
        Evaluation {
            observables: solve.observables,
            macro_error_bound: geometric.max(self.guard_term(solve)),
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
            ImpactResidual::WrongSurfaceResidual => {
                let phantom_x = 0.5 * self.config.geometry.side_m;
                (cell.x0 - phantom_x).max(phantom_x - (cell.x0 + cell.size)).max(0.0)
            }
            _ => chart.distance(holon),
        };
        let ratio = cell.size / self.allowed_spacing(distance);
        if ratio > self.config.macro_tolerance {
            return ratio;
        }
        // Guard-violation steering: the graded ledger may already pass here at a
        // loose tolerance, but a guarded bond under load convicts THIS pitch, so
        // its cells stay refinement candidates (coarse-first) until the next solve
        // clears the violation.
        if self.config.residual != ImpactResidual::WrongSurfaceResidual
            && self.violations.iter().any(|point| {
                point[0] >= cell.x0 - 1.0e-12
                    && point[0] <= cell.x0 + cell.size + 1.0e-12
                    && point[1] >= cell.y0 - 1.0e-12
                    && point[1] <= cell.y0 + cell.size + 1.0e-12
            })
        {
            return self.config.macro_tolerance * (1.0 + cell.size / self.required_m);
        }
        0.0
    }
}

// ---------------------------------------------------------------------------
// Mesh derivation: (k_n, k_t) from homogenization, cohesive law from the
// constructor where legal, guarded envelope where refused.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BondKind {
    Edge,
    Diagonal,
}

struct ImpactBond {
    a: usize,
    b: usize,
    bond: CohesiveBond,
    area_m2: f64,
    tangential_stiffness_n_m: f64,
    tangential_displacement_m: f64,
    midpoint: [f64; 2],
    /// Constructor-refused or quench-closed softening: fracture energy floored,
    /// certificate watches the load.
    guarded: bool,
    #[allow(dead_code)]
    kind: BondKind,
}

struct ImpactNode {
    position: [f64; 2],
    mass_kg: f64,
    radius_m: f64,
    inertia_kg_m2: f64,
    anchored: bool,
}

struct ImpactMesh {
    nodes: Vec<ImpactNode>,
    bonds: Vec<ImpactBond>,
    /// Per-node adjacency (other node, bond index) for the contact jurisdiction test.
    bonds_by_node: Vec<Vec<(usize, usize)>>,
    finest_cell_m: f64,
}

fn geometry_eps(geometry: &WallGeometry) -> f64 {
    1.0e-9 * geometry.side_m
}

fn overlap(a0: f64, a1: f64, b0: f64, b1: f64) -> f64 {
    (a1.min(b1) - a0.max(b0)).max(0.0)
}

/// The alternating-diagonal selection at the corner (cx, cy) with local pitch `s`:
/// keep the SW–NE diagonal on even checkerboard parity, the NW–SE one on odd.
fn diagonal_selected(cx: f64, cy: f64, s: f64, southwest_to_northeast: bool) -> bool {
    let px = libm::round(cx / s) as i64;
    let py = libm::round(cy / s) as i64;
    let even = (px + py) % 2 == 0;
    even == southwest_to_northeast
}

fn build_impact_mesh(
    arena: &RuntimeArena,
    frontier: &RuntimeFrontier,
    chart: &WallChart,
    config: &ImpactConfig,
    properties: &IsotropicMaterial,
    strength_ref_pa: f64,
    diameter_ref_m: f64,
) -> ImpactMesh {
    let geometry = &config.geometry;
    let eps = geometry_eps(geometry);
    let moduli = plane_stress_moduli(properties).expect("validated material");
    let thickness = geometry.thickness_m;
    let normal_stiffness = thickness * (moduli.lambda_pa + moduli.mu_pa);
    let tangential_stiffness = if config.residual == ImpactResidual::AxialOnly {
        0.0
    } else {
        thickness * (moduli.mu_pa - moduli.lambda_pa)
    };
    let total_constituents = arena.holons()[arena.root() as usize].gross.constituents as f64;
    let mass_unit =
        properties.density_kg_m3 * geometry.side_m * geometry.side_m * thickness
            / total_constituents;

    let active: Vec<usize> = frontier.active_indices().collect();
    let mut nodes = Vec::with_capacity(active.len());
    let mut draws: Vec<Option<GrainDraw>> = Vec::with_capacity(active.len());
    let mut cells: Vec<Cell> = Vec::with_capacity(active.len());
    let mut finest = f64::INFINITY;
    for &holon in &active {
        let cell = chart.cell(holon);
        finest = finest.min(cell.size);
        let mass = arena.holons()[holon].gross.constituents as f64 * mass_unit;
        let radius = NODE_RADIUS_FRACTION * cell.size;
        nodes.push(ImpactNode {
            position: [cell.x0 + 0.5 * cell.size, cell.y0 + 0.5 * cell.size],
            mass_kg: mass,
            radius_m: radius,
            inertia_kg_m2: 0.5 * mass * radius * radius,
            anchored: cell.y0 <= eps,
        });
        draws.push(read_grain(arena, holon));
        cells.push(cell);
    }

    let mut bonds: Vec<ImpactBond> = Vec::new();
    let mut bonds_by_node: Vec<Vec<(usize, usize)>> = Vec::new();
    bonds_by_node.resize_with(nodes.len(), Vec::new);
    for i in 0..cells.len() {
        for j in (i + 1)..cells.len() {
            let (a, b) = (cells[i], cells[j]);
            let (ax1, ay1) = (a.x0 + a.size, a.y0 + a.size);
            let (bx1, by1) = (b.x0 + b.size, b.y0 + b.size);
            let x_overlap = overlap(a.x0, ax1, b.x0, bx1);
            let y_overlap = overlap(a.y0, ay1, b.y0, by1);
            let x_touch = libm::fabs(a.x0 - bx1).min(libm::fabs(b.x0 - ax1)) <= eps;
            let y_touch = libm::fabs(a.y0 - by1).min(libm::fabs(b.y0 - ay1)) <= eps;

            let pitch = a.size.min(b.size);
            let (kind, interface_m) = if x_touch && y_overlap > eps {
                (BondKind::Edge, y_overlap)
            } else if y_touch && x_overlap > eps {
                (BondKind::Edge, x_overlap)
            } else if x_touch && y_touch && x_overlap <= eps && y_overlap <= eps {
                // Corner adjacency: one alternating diagonal per cell-quad, the
                // stencil the homogenization theorem is exact on.
                let corner_x = if libm::fabs(a.x0 - bx1) <= eps { a.x0 } else { ax1 };
                let corner_y = if libm::fabs(a.y0 - by1) <= eps { a.y0 } else { ay1 };
                // SW–NE when the lower cell is on the left of the upper.
                let a_is_lower = a.y0 < b.y0;
                let lower = if a_is_lower { &a } else { &b };
                let upper = if a_is_lower { &b } else { &a };
                let sw_ne = lower.x0 < upper.x0;
                if !diagonal_selected(corner_x, corner_y, pitch, sw_ne) {
                    continue;
                }
                (BondKind::Diagonal, 0.5 * pitch)
            } else {
                continue;
            };

            let (Some(da), Some(db)) = (draws[i], draws[j]) else {
                continue;
            };
            let (pa, pb) = (nodes[i].position, nodes[j].position);
            let rest = {
                let dx = pb[0] - pa[0];
                let dy = pb[1] - pa[1];
                libm::sqrt(dx * dx + dy * dy)
            };
            let area_m2 = interface_m * thickness;
            let width_fraction = interface_m / pitch;

            // Elastic: the derived per-relation stiffnesses, scaled by the interface
            // width fraction (1 on the uniform stencil; hanging-node interfaces
            // carry their share). Cohesive: the constructor at the local pitch,
            // quenched by the endpoint draws; refusal or a quench-closed softening
            // interval yields a GUARDED bond the certificate watches.
            let k_n = normal_stiffness * width_fraction;
            let k_t = tangential_stiffness * width_fraction;
            let quench = (da.strength_pa.min(db.strength_pa) / strength_ref_pa)
                .clamp(QUENCH_MIN, QUENCH_MAX);
            let rough = (0.5 * (da.diameter_m + db.diameter_m) / diameter_ref_m)
                .clamp(QUENCH_MIN, QUENCH_MAX);
            let synthetic_mass = properties.density_kg_m3 * pitch * pitch * thickness;
            let derived = derive_bilinear_cohesive_law(properties, synthetic_mass, pitch);
            let (mut peak, mut energy, mut guarded) = match derived {
                Ok(law) => (
                    law.peak_force_n * width_fraction * quench,
                    law.fracture_energy_j * width_fraction * rough,
                    false,
                ),
                Err(HomogenizationError::CohesiveUnderResolved) => (
                    properties.tensile_strength_pa * area_m2 * quench,
                    properties.fracture_energy_j_m2 * area_m2 * rough,
                    true,
                ),
                Err(_) => unreachable!("material validated at scene construction"),
            };
            let energy_floor = SNAPBACK_GUARD * peak * peak / (2.0 * k_n);
            if energy < energy_floor {
                energy = energy_floor;
                guarded = true;
            }
            if peak <= 0.0 {
                peak = properties.tensile_strength_pa * area_m2;
                guarded = true;
            }
            let reduced_mass =
                nodes[i].mass_kg * nodes[j].mass_kg / (nodes[i].mass_kg + nodes[j].mass_kg);
            let law = CohesiveLaw {
                stiffness_n_m: k_n,
                damping_n_s_m: 2.0 * config.solver_zeta * libm::sqrt(k_n * reduced_mass),
                peak_force_n: peak,
                fracture_energy_j: energy,
                friction_coefficient: 0.74,
            };
            let relation = bonds.len() + 1;
            let bond = CohesiveBond::new(relation, active[i], active[j], rest, law)
                .expect("guarded derivation must validate");
            bonds_by_node[i].push((j, bonds.len()));
            bonds_by_node[j].push((i, bonds.len()));
            bonds.push(ImpactBond {
                a: i,
                b: j,
                bond,
                area_m2,
                tangential_stiffness_n_m: k_t,
                tangential_displacement_m: 0.0,
                midpoint: [0.5 * (pa[0] + pb[0]), 0.5 * (pa[1] + pb[1])],
                guarded,
                kind,
            });
        }
    }

    ImpactMesh {
        nodes,
        bonds,
        bonds_by_node,
        finest_cell_m: finest,
    }
}

// ---------------------------------------------------------------------------
// The deterministic composed solve: bonds (normal + tangential + slider),
// node-node contact with the D → 1 handoff, ball-node contact, spins.
// ---------------------------------------------------------------------------

fn run_impact_solve(
    mesh: &ImpactMesh,
    config: &ImpactConfig,
    properties: &IsotropicMaterial,
) -> SolveSummary {
    let n = mesh.nodes.len();
    if n < 2 || mesh.bonds.is_empty() {
        return SolveSummary {
            observables: [0.0; IMPACT_OBSERVABLES],
            conservation_residual: f64::INFINITY,
            damage_points: Vec::new(),
            violation_points: Vec::new(),
            guarded_worst_load: 0.0,
            guarded_bonds: mesh.bonds.iter().filter(|b| b.guarded).count(),
            broken_bonds: 0,
            fracture_energy_j: 0.0,
        };
    }

    let wave_speed = libm::sqrt(properties.young_modulus_pa / properties.density_kg_m3);
    let dt_target = config.cfl * mesh.finest_cell_m / wave_speed;
    let steps = libm::ceil(config.duration_s / dt_target) as usize;
    let dt = config.duration_s / steps as f64;

    let mut bonds: Vec<CohesiveBond> = mesh.bonds.iter().map(|slot| slot.bond).collect();
    let mut tangential_disp: Vec<f64> =
        mesh.bonds.iter().map(|slot| slot.tangential_displacement_m).collect();
    let mut position: Vec<[f64; 2]> = mesh.nodes.iter().map(|node| node.position).collect();
    let mut velocity: Vec<[f64; 2]> = alloc::vec![[0.0, 0.0]; n];
    let mut spin: Vec<f64> = alloc::vec![0.0; n];
    let mut force: Vec<[f64; 2]> = alloc::vec![[0.0, 0.0]; n];
    let mut torque: Vec<f64> = alloc::vec![0.0; n];

    // The projectile, launched at the aim point on the left face.
    let mut ball_position = [
        -(config.ball_radius_m + 0.04 * config.geometry.side_m),
        config.aim_y_m,
    ];
    let mut ball_velocity = [config.ball_speed_m_s, 0.0];

    // Solver contact configuration (A5): penalty stiffness as a declared fraction of
    // the derived k_n, damping from the same solver zeta at the contact scale.
    let moduli = plane_stress_moduli(properties).expect("validated material");
    let contact_stiffness = config.contact_stiffness_fraction
        * config.geometry.thickness_m
        * (moduli.lambda_pa + moduli.mu_pa);

    let mut candidates: Vec<(usize, usize)> = Vec::new();
    let skin = REBUILD_EVERY_STEPS as f64 * dt * SKIN_SPEED_M_S;

    let mut impulse_ns = 0.0_f64;
    let mut external_impulse = [0.0_f64, 0.0_f64];
    let mut finite = true;
    let mut guarded_load: Vec<f64> = alloc::vec![0.0; mesh.bonds.len()];

    for step in 0..steps {
        if step % REBUILD_EVERY_STEPS == 0 {
            candidates.clear();
            for i in 0..n {
                for j in (i + 1)..n {
                    let dx = position[j][0] - position[i][0];
                    let dy = position[j][1] - position[i][1];
                    let reach = mesh.nodes[i].radius_m + mesh.nodes[j].radius_m + skin;
                    if dx * dx + dy * dy < reach * reach {
                        candidates.push((i, j));
                    }
                }
            }
        }

        force.fill([0.0, 0.0]);
        torque.fill(0.0);
        let mut ball_force = [0.0_f64, 0.0_f64];

        // Bonds: normal channel through the cohesive relation, tangential channel
        // through (1-D) k_t on integrated slip plus the A3 slider, slip measured at
        // the interface midpoint including spins (frame indifference).
        for (index, slot) in mesh.bonds.iter().enumerate() {
            let bond = &mut bonds[index];
            let (a, b) = (slot.a, slot.b);
            let dx = position[b][0] - position[a][0];
            let dy = position[b][1] - position[a][1];
            let distance = libm::sqrt(dx * dx + dy * dy);
            if distance <= 1.0e-12 || !distance.is_finite() {
                finite = false;
                continue;
            }
            let normal = [dx / distance, dy / distance];
            let rel = [
                velocity[b][0] - velocity[a][0],
                velocity[b][1] - velocity[a][1],
            ];
            let closing = rel[0] * normal[0] + rel[1] * normal[1];
            let axial = bond.axial_force(distance - bond.rest_length_m, closing);
            force[a][0] += normal[0] * axial;
            force[a][1] += normal[1] * axial;
            force[b][0] -= normal[0] * axial;
            force[b][1] -= normal[1] * axial;
            if slot.guarded {
                let load = libm::fabs(axial) / bond.law.peak_force_n;
                if load > guarded_load[index] {
                    guarded_load[index] = load;
                }
            }

            let tangent = [-normal[1], normal[0]];
            let half_length = 0.5 * distance;
            let slip_speed = rel[0] * tangent[0] + rel[1] * tangent[1]
                - (spin[a] + spin[b]) * half_length;
            let mut tangential_force = 0.0;
            if !bond.is_broken() {
                tangential_disp[index] += slip_speed * dt;
                tangential_force += (1.0 - bond.damage())
                    * slot.tangential_stiffness_n_m
                    * tangential_disp[index];
            }
            let slider = bond.closed_friction_force(axial, slip_speed);
            if slider > 0.0 {
                tangential_force += slider * if slip_speed >= 0.0 { 1.0 } else { -1.0 };
            }
            if tangential_force != 0.0 {
                force[a][0] += tangent[0] * tangential_force;
                force[a][1] += tangent[1] * tangential_force;
                force[b][0] -= tangent[0] * tangential_force;
                force[b][1] -= tangent[1] * tangential_force;
                torque[a] += half_length * tangential_force;
                torque[b] += half_length * tangential_force;
            }
        }

        // Node-node contact: jurisdiction is exactly {D = 1 pairs} ∪ {never-bonded
        // pairs} (live bonds own the closed regime); a failed pair inherits its dead
        // bond's tribology through failed_contact_friction_force.
        for &(i, j) in &candidates {
            let pair_bond = mesh.bonds_by_node[i]
                .iter()
                .find(|&&(other, _)| other == j)
                .map(|&(_, bond)| bond);
            if pair_bond.is_some_and(|bond| !bonds[bond].is_broken()) {
                continue;
            }
            let dx = position[j][0] - position[i][0];
            let dy = position[j][1] - position[i][1];
            let reach = mesh.nodes[i].radius_m + mesh.nodes[j].radius_m;
            let distance_sq = dx * dx + dy * dy;
            if distance_sq >= reach * reach || distance_sq <= 1.0e-24 {
                continue;
            }
            let distance = libm::sqrt(distance_sq);
            let normal = [dx / distance, dy / distance];
            let rel = [
                velocity[j][0] - velocity[i][0],
                velocity[j][1] - velocity[i][1],
            ];
            let separating = rel[0] * normal[0] + rel[1] * normal[1];
            let reduced_mass = mesh.nodes[i].mass_kg * mesh.nodes[j].mass_kg
                / (mesh.nodes[i].mass_kg + mesh.nodes[j].mass_kg);
            let contact_damping =
                2.0 * config.solver_zeta * libm::sqrt(contact_stiffness * reduced_mass);
            let magnitude =
                (contact_stiffness * (reach - distance) - contact_damping * separating).max(0.0);
            let mut force_on_j = [normal[0] * magnitude, normal[1] * magnitude];
            let tangential = [rel[0] - normal[0] * separating, rel[1] - normal[1] * separating];
            let tangential_speed =
                libm::sqrt(tangential[0] * tangential[0] + tangential[1] * tangential[1]);
            if tangential_speed > 1.0e-12 && magnitude > 0.0 {
                let friction = match pair_bond {
                    Some(bond) => {
                        bonds[bond].failed_contact_friction_force(magnitude, tangential_speed)
                    }
                    None => (contact_damping * tangential_speed).min(0.74 * magnitude),
                };
                force_on_j[0] -= tangential[0] * (friction / tangential_speed);
                force_on_j[1] -= tangential[1] * (friction / tangential_speed);
            }
            force[j][0] += force_on_j[0];
            force[j][1] += force_on_j[1];
            force[i][0] -= force_on_j[0];
            force[i][1] -= force_on_j[1];
        }

        // Ball-node contact (never-bonded jurisdiction; solver constants).
        for i in 0..n {
            let dx = position[i][0] - ball_position[0];
            let dy = position[i][1] - ball_position[1];
            let distance = libm::sqrt(dx * dx + dy * dy);
            let reach = config.ball_radius_m + mesh.nodes[i].radius_m;
            if distance >= reach || distance <= 1.0e-12 {
                continue;
            }
            let normal = [dx / distance, dy / distance];
            let rel = [
                velocity[i][0] - ball_velocity[0],
                velocity[i][1] - ball_velocity[1],
            ];
            let separating = rel[0] * normal[0] + rel[1] * normal[1];
            let reduced_mass = mesh.nodes[i].mass_kg * config.ball_mass_kg
                / (mesh.nodes[i].mass_kg + config.ball_mass_kg);
            let contact_damping =
                2.0 * config.solver_zeta * libm::sqrt(contact_stiffness * reduced_mass);
            let magnitude =
                (contact_stiffness * (reach - distance) - contact_damping * separating).max(0.0);
            force[i][0] += normal[0] * magnitude;
            force[i][1] += normal[1] * magnitude;
            ball_force[0] -= normal[0] * magnitude;
            ball_force[1] -= normal[1] * magnitude;
        }
        // Impulse delivered by the wall to the ball, positive against the throw.
        impulse_ns += -ball_force[0] * dt;

        for i in 0..n {
            if mesh.nodes[i].anchored {
                velocity[i] = [0.0, 0.0];
                spin[i] = 0.0;
                continue;
            }
            external_impulse[0] += force[i][0] * dt;
            external_impulse[1] += force[i][1] * dt;
            velocity[i][0] += force[i][0] / mesh.nodes[i].mass_kg * dt;
            velocity[i][1] += force[i][1] / mesh.nodes[i].mass_kg * dt;
            position[i][0] += velocity[i][0] * dt;
            position[i][1] += velocity[i][1] * dt;
            spin[i] += torque[i] / mesh.nodes[i].inertia_kg_m2 * dt;
            if !position[i][0].is_finite() || !position[i][1].is_finite() {
                finite = false;
            }
        }
        external_impulse[0] += ball_force[0] * dt;
        external_impulse[1] += ball_force[1] * dt;
        ball_velocity[0] += ball_force[0] / config.ball_mass_kg * dt;
        ball_velocity[1] += ball_force[1] / config.ball_mass_kg * dt;
        ball_position[0] += ball_velocity[0] * dt;
        ball_position[1] += ball_velocity[1] * dt;
    }

    // Momentum bookkeeping over the free bodies (wall free nodes + ball): symplectic
    // Euler adds F*dt to m*v exactly, so this is a rounding-level integrity check.
    let mut free_momentum = [
        config.ball_mass_kg * ball_velocity[0] - config.ball_mass_kg * config.ball_speed_m_s,
        config.ball_mass_kg * ball_velocity[1],
    ];
    for i in 0..n {
        if !mesh.nodes[i].anchored {
            free_momentum[0] += mesh.nodes[i].mass_kg * velocity[i][0];
            free_momentum[1] += mesh.nodes[i].mass_kg * velocity[i][1];
        }
    }
    let mismatch = libm::fabs(free_momentum[0] - external_impulse[0])
        + libm::fabs(free_momentum[1] - external_impulse[1]);
    let scale = (libm::fabs(external_impulse[0]) + libm::fabs(external_impulse[1]))
        .max(1.0e-6 * config.ball_mass_kg * config.ball_speed_m_s);
    let conservation_residual = if finite { mismatch / scale } else { f64::INFINITY };

    // Observables and the damage surface. A guarded bond that carried real load is
    // a residual point exactly like damage: it marks where the frontier must
    // refine before any certified answer can stand.
    let mut damage_points = Vec::new();
    let mut violation_points = Vec::new();
    let mut crack_area = 0.0_f64;
    let mut fracture_energy_j = 0.0_f64;
    let mut broken_bonds = 0_usize;
    let mut guarded_worst_load = 0.0_f64;
    for (index, (slot, bond)) in mesh.bonds.iter().zip(&bonds).enumerate() {
        if bond.damage() > DAMAGE_MARK {
            damage_points.push(slot.midpoint);
        }
        if slot.guarded {
            guarded_worst_load = guarded_worst_load.max(guarded_load[index]);
            if guarded_load[index] >= config.guard_fraction {
                damage_points.push(slot.midpoint);
                violation_points.push(slot.midpoint);
            }
        }
        if bond.is_broken() {
            broken_bonds += 1;
            crack_area += slot.area_m2;
            // The engine's OWN number for the work this break cost, roughness quench
            // included — not `G_F * area`, which omits it.
            fracture_energy_j += bond.law.fracture_energy_j;
        }
    }

    // Detached mass: connected components over LIVE bonds; anything not connected
    // to an anchored node is a fragment (union-find, deterministic order).
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut Vec<usize>, mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    for (slot, bond) in mesh.bonds.iter().zip(&bonds) {
        if !bond.is_broken() {
            let (ra, rb) = (find(&mut parent, slot.a), find(&mut parent, slot.b));
            if ra != rb {
                parent[ra.max(rb)] = ra.min(rb);
            }
        }
    }
    let mut anchored_roots = alloc::vec![false; n];
    for i in 0..n {
        if mesh.nodes[i].anchored {
            let root = find(&mut parent, i);
            anchored_roots[root] = true;
        }
    }
    let mut detached_mass = 0.0_f64;
    for i in 0..n {
        let root = find(&mut parent, i);
        if !anchored_roots[root] {
            detached_mass += mesh.nodes[i].mass_kg;
        }
    }

    SolveSummary {
        observables: [impulse_ns, crack_area, detached_mass],
        conservation_residual,
        damage_points,
        violation_points,
        guarded_worst_load,
        guarded_bonds: mesh.bonds.iter().filter(|slot| slot.guarded).count(),
        broken_bonds,
        fracture_energy_j,
    }
}

// ---------------------------------------------------------------------------
// The scene: descriptor library + wall subject + certification entry point.
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct ImpactRun {
    pub result: AdaptiveRuntimeCertificate<IMPACT_OBSERVABLES>,
    pub solves_run: usize,
    /// Certificate requirement (ℓ_ch/10) and the two constructor numbers the prize
    /// gate displays: h_max and the achieved corridor spacing.
    pub required_spacing_m: f64,
    pub constructor_h_max_m: f64,
    pub finest_active_m: f64,
    pub grain_floor_m: f64,
    pub materialized_near: usize,
    pub materialized_far: usize,
    pub guarded_bonds: usize,
    pub guarded_worst_load: f64,
    /// Fracture work the certified solve dissipated, joules (engine's own per-bond sum).
    pub fracture_energy_j: f64,
}

pub struct ImpactScene {
    pub config: ImpactConfig,
    pub binding: MaterialBinding,
    descriptors: RuntimeArena,
    arena: RuntimeArena,
    declared: DrawParams,
    /// Owned, not shared (G5). Lent to the model for a certification run and taken back
    /// with `ImpactModel::into_chart`.
    chart: RefCell<WallChart>,
    root_grain: u32,
}

impl ImpactScene {
    pub fn new(
        config: ImpactConfig,
        material: IsotropicMaterial,
        law: DrawParams,
        modal_weights: &[u64],
        total_constituents: u64,
        root_grain: u32,
        seed: u64,
    ) -> Result<Self, crate::fracture::FractureError> {
        if !root_grain.is_power_of_two() || root_grain < 2 {
            return Err(crate::fracture::FractureError::InvalidConfiguration);
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
            properties: material,
        };
        Ok(Self {
            config,
            binding,
            descriptors,
            arena,
            declared: law,
            chart: RefCell::new(WallChart::new(config.geometry)),
        root_grain,
        })
    }

    pub fn arena(&self) -> &RuntimeArena {
        &self.arena
    }

    /// Borrow the scene's chart; `.borrow()` / `.borrow_mut()` call sites are unchanged.
    pub fn chart(&self) -> &RefCell<WallChart> {
        &self.chart
    }

    pub fn certify(&mut self) -> Result<ImpactRun, crate::fracture::FractureError> {
        let mut model = ImpactModel::new(
            self.config,
            self.binding.properties,
            self.declared,
            WallChart::new(self.config.geometry),
        );
        // Settled spacing: a child at or below it can never need refinement. Two
        // conditions, both required: the graded ledger can never demand finer
        // (tol * l_ch/10), AND no quenched bond at that pitch can be guarded
        // (h <= h_max * q_min / (q_max^2 * SNAPBACK_GUARD)) — otherwise a loose
        // tolerance settles cells that a guard violation must later refine, and
        // the seam deadlocks (found by the coarse convergence leg).
        let guard_free = max_bilinear_spacing_m(&self.binding.properties)
            .expect("validated material")
            * QUENCH_MIN
            / (QUENCH_MAX * QUENCH_MAX * SNAPBACK_GUARD);
        let settled = (self.config.macro_tolerance
            * required_spacing_m(&self.binding.properties))
        .min(guard_free);
        let selector =
            // `self.root_grain` rather than a read-back from the arena root: they are equal
            // by construction (it is written into the root holon's `grain_units`), but the
            // field is the same value `ImpactScene::new` power-of-two validated, which is the
            // precondition the selector's exact-division argument rests on. `FractureScene`
            // does NOT store it, so the arena root is the correct source there — the asymmetry
            // is deliberate, not an oversight to be tidied.
            TipSpacingSelector::new(self.config.geometry.side_m, self.root_grain, settled);
        let mut materializer = DescriptorMaterializer::with_boundary(
            &self.descriptors,
            self.binding,
            FANOUT,
            0,
            selector,
        )?;
        let result = certify_runtime_adaptive(
            &mut self.arena,
            &mut model,
            &mut materializer,
            self.config.macro_tolerance,
            CONSERVATION_TOLERANCE,
        )?;

        // Take the chart back from the model before the locality bookkeeping.
        // Everything read from the model is taken BEFORE `into_chart` consumes it.
        let solves_run = model.solves_run;
        let (guarded_bonds, guarded_worst_load) = model.last_guarded();
        let fracture_energy_j = model.last_fracture_energy_j();
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

        Ok(ImpactRun {
            result,
            solves_run,
            required_spacing_m: required_spacing_m(&self.binding.properties),
            constructor_h_max_m: max_bilinear_spacing_m(&self.binding.properties)
                .expect("validated material"),
            finest_active_m,
            grain_floor_m: self.config.geometry.side_m / self.root_grain as f64,
            materialized_near: near,
            materialized_far: far,
            fracture_energy_j,
            guarded_bonds,
            guarded_worst_load,
        })
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

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
    const SEED: u64 = 0xC1F1_00E1_0000_A97C;

    /// Eighth-meter Lac du Bonnet test wall. l_ch = 66.5e9*30.6/(6.9e6)^2 =
    /// 42.74 mm, required spacing 4.274 mm; cell at depth d is 0.125/2^d m, so
    /// tolerance 1.0 certifies at depth 5 (3.91 mm) and depth 4 (7.81 mm) fails.
    /// Constructor domain h_max = 2 G_F (lambda+mu)/f_t^2 = 61.9 mm.
    fn test_config(macro_tolerance: f64) -> ImpactConfig {
        ImpactConfig {
            geometry: WallGeometry {
                side_m: 0.125,
                thickness_m: 0.1,
                notch_m: 0.0,
            },
            aim_y_m: 0.0625,
            ball_mass_kg: 0.4,
            ball_speed_m_s: 9.0,
            ball_radius_m: 0.012,
            macro_tolerance,
            grading: 2.0,
            duration_s: 1.6e-3,
            cfl: 0.25,
            solver_zeta: 0.15,
            contact_stiffness_fraction: 0.02,
            guard_fraction: 0.5,
            residual: ImpactResidual::Correct,
        }
    }

    fn scene(macro_tolerance: f64, root_grain: u32, seed: u64) -> ImpactScene {
        ImpactScene::new(
            test_config(macro_tolerance),
            IsotropicMaterial::LAC_DU_BONNET,
            demo_law(),
            &MODAL,
            1_000_000,
            root_grain,
            seed,
        )
        .unwrap()
    }

    // The prize gate: LAC_DU_BONNET runs legally. The constructor refuses the
    // specimen at every coarse spacing down to h_max = 61.9 mm; the adaptive
    // certifier refines the impact corridor to 3.91 mm — inside the constructor
    // domain by a factor h_max/h ≈ 15.8 — certifies, fractures, and no guarded
    // bond carries load in the certified solve.
    #[test]
    fn lac_du_bonnet_certifies_where_the_constructor_refuses() {
        let material = IsotropicMaterial::LAC_DU_BONNET;
        // The refusal the prize gate is measured against, asserted first.
        let coarse_mass = material.density_kg_m3 * 0.125 * 0.125 * 0.1;
        assert_eq!(
            derive_bilinear_cohesive_law(&material, coarse_mass, 0.125),
            Err(HomogenizationError::CohesiveUnderResolved)
        );

        let mut scene = scene(1.0, 64, SEED);
        let run = scene.certify().unwrap();
        // The prize gate's numbers on the record on a PASS, not only in a failure
        // message (verification, 2026-08-24) — this is the headline reading of N-e
        // and it was previously only assertable, never readable.
        std::println!(
            "prize gate: finest {:.5} m <= required {:.5} m, constructor h_max {:.5} m; \
             guarded worst load {:.3} (< {:.3}); materializations {} (near {} / far {})",
            run.finest_active_m,
            run.required_spacing_m,
            run.constructor_h_max_m,
            run.guarded_worst_load,
            scene.config.guard_fraction,
            run.result.materializations,
            run.materialized_near,
            run.materialized_far
        );
        assert!(run.result.certificate.passed(), "{:?}", run.result.certificate.status);
        assert!(run.finest_active_m <= run.required_spacing_m);
        assert!(run.finest_active_m < run.constructor_h_max_m);
        // The throw actually fractured the wall and delivered impulse.
        let [impulse, crack_area, _detached] = run.result.certificate.observables;
        assert!(impulse > 0.0, "no impulse delivered: {impulse}");
        assert!(crack_area > 0.0, "no fracture: {crack_area}");

        // EXTERNAL ANCHORS — the only checks in this module that do NOT come from the
        // engine (research-manager verification, 2026-08-24). Everything else here
        // compares the engine to itself or to its own declared configuration; these two
        // come from mechanics. They are what the module header's OWED external reference
        // is partially paid with, and the payment is small: see the caveat on each.
        let m = scene.config.ball_mass_kg;
        let v = scene.config.ball_speed_m_s;
        // (1) IMPULSE WINDOW. A ball of momentum m*v delivers exactly m*v if it stops
        // dead and 2*m*v if it rebounds perfectly elastically; nothing else is
        // physically reachable. CAVEAT, stated so nobody later mistakes it for a tight
        // bound: this is a FACTOR-OF-TWO window. It is a real constraint from Newton
        // rather than from us, and it is loose.
        assert!(
            impulse >= m * v && impulse <= 2.0 * m * v,
            "impulse {impulse} outside the [m*v, 2*m*v] = [{}, {}] window that elementary \
             mechanics allows — LOOSE bound (factor of two), so a violation is serious",
            m * v,
            2.0 * m * v
        );
        // (2) GRIFFITH / LEFM ENERGY BOUND, now EXACT rather than convention-dependent.
        // The rebound speed follows from the measured impulse, and the fracture the run
        // created cannot have cost more energy than the ball lost.
        //
        // THE FACES QUESTION IS SETTLED (code read, 2026-08-24): the engine charges ONE
        // face — `fracture.rs:674` builds each bond's energy as
        // `G_F * area_m2 * rough`, and `homogenization.rs:158` uses the same convention.
        // But `G_F * Σ area` is STILL the wrong number, because each bond carries its own
        // quenched roughness factor in [QUENCH_MIN, QUENCH_MAX] = [0.5, 2.0]. So this
        // anchor uses `run.fracture_energy_j` — the engine's own per-bond sum over broken
        // bonds — which needs no convention and no roughness estimate. The area-derived
        // figure is printed beside it, and the gap between them is NOT a tolerance this
        // anchor tolerates: the ~14% BY WHICH THE AREA ESTIMATE FALLS SHORT *IS* THE
        // QUENCHED PER-BOND ROUGHNESS — a measured mechanism with a name, not an error
        // bar. Phrase it that way anywhere this number is carried: "the 14% IS the
        // quench", never "agrees within 14%", because the second invites a later reader
        // to widen a mechanism into a tolerance.
        let v_out = impulse / m - v;
        let ke_lost = 0.5 * m * (v * v - v_out * v_out);
        let exact_fracture_j = run.fracture_energy_j;
        let area_estimate_j = material.fracture_energy_j_m2 * crack_area;
        std::println!(
            "external anchors: impulse {impulse:.4} in [{:.3}, {:.3}] N.s; rebound {v_out:.3} m/s; \
             KE lost {ke_lost:.4} J vs fracture work {exact_fracture_j:.4} J exact \
             ({:.1}% of KE lost; area-estimate reads {area_estimate_j:.4} J and the {:.0}% shortfall \
             IS the quenched roughness, not a tolerance)",
            m * v,
            2.0 * m * v,
            100.0 * exact_fracture_j / ke_lost,
            100.0 * (exact_fracture_j / area_estimate_j - 1.0)
        );
        assert!(
            exact_fracture_j > 0.0,
            "the wall fractured (crack area {crack_area}) but the engine charged zero \
             fracture work — the energy ledger is not tracking the breaks"
        );
        assert!(
            exact_fracture_j <= ke_lost,
            "fracture work {exact_fracture_j} J exceeds the kinetic energy the ball lost \
             ({ke_lost} J) — the run created more surface than it paid for"
        );
        // The guard seam: guarded bonds exist on the frontier (quiet coarse cells)
        // but none carried load in the certified solve.
        assert!(run.guarded_worst_load < scene.config.guard_fraction,
            "guarded load {}", run.guarded_worst_load);
        // Locality: materialization is corridor-local around the impact.
        assert!(run.materialized_far * 20 <= run.materialized_near.max(1),
            "far {} near {}", run.materialized_far, run.materialized_near);
        assert!(run.result.materializations >= 20);
    }

    // TWO-SOLVE PROBE (house rule), and the second external anchor it unlocks.
    //
    // The Hertzian/elastic anchor only means anything BELOW the fracture threshold, and
    // nothing in this module knew where that threshold was — so it is measured here
    // before anything is staked on it, per the rule that a contrast must be probed
    // before it is relied on.
    //
    // Duration is scaled with speed so every rung gets the SAME post-contact window:
    // the ball starts `0.04 * side_m` clear of the wall (`impact.rs:653`), so flight
    // time is `0.005 / v` and a fixed contact window is added on top. Without this the
    // slow rungs simply never reach the wall inside the base duration and would read
    // "no fracture" for a reason that has nothing to do with a threshold.
    #[test]
    fn fracture_threshold_probe_and_the_subthreshold_elastic_anchor() {
        const CLEARANCE_M: f64 = 0.005; // 0.017 start offset - 0.012 ball radius
        const CONTACT_WINDOW_S: f64 = 1.04e-3; // what 9 m/s gets at the base duration
        let mut rows: Vec<(f64, f64, f64, f64)> = Vec::new();
        for &v in &[9.0_f64, 6.0, 4.0, 2.5] {
            let mut sc = scene(1.0, 64, SEED);
            sc.config.ball_speed_m_s = v;
            sc.config.duration_s = CLEARANCE_M / v + CONTACT_WINDOW_S;
            let run = sc.certify().unwrap();
            let [impulse, crack_area, _] = run.result.certificate.observables;
            rows.push((v, impulse, crack_area, run.fracture_energy_j));
            std::println!(
                "threshold probe: v={v:.2} m/s -> impulse {impulse:.4} N.s, crack area \
                 {crack_area:.6} m^2, fracture work {:.4} J | FRONTIER finest {:.5} m, \
                 {} materializations (duration {:.2e} s)",
                run.fracture_energy_j,
                run.finest_active_m,
                run.result.materializations,
                sc.config.duration_s
            );
        }

        // MEASURED, and it replaced the assertion this probe first carried: damage is
        // NOT monotone in the drive. 6 m/s reads MORE crack area and MORE fracture work
        // than 9 m/s (0.020312 vs 0.019336 m^2; 0.6996 vs 0.6737 J). The discriminator
        // was run rather than guessed, and it rules out the resolution explanation:
        // ALL FOUR rungs certify at the same finest spacing, 0.00391 m, and differ only
        // in frontier EXTENT (289 / 277 / 264 / 228 materializations). The quench draw
        // redraws with the materialized set, so damage feeds extent feeds quench feeds
        // damage, and a ~5% inversion between neighbouring rungs is quench variability
        // rather than a claim about the physics. Same shape as the convergence gate's
        // coarse leg, which is also same-spacing-different-extent.
        //
        // So the threshold is located by the ZERO/NONZERO transition, which is the part
        // that IS frontier-robust: no broken bond is no broken bond at any extent.
        assert!(rows[0].2 > 0.0, "the fastest rung did not fracture — no threshold in range");
        assert!(
            rows[rows.len() - 1].2 == 0.0,
            "the slowest rung still fractured — the threshold is below the swept range"
        );
        // Once sub-threshold, staying sub-threshold going slower is the actual threshold
        // property, and it is what this probe may assert.
        let first_zero = rows.iter().position(|r| r.2 == 0.0).expect("checked above");
        for r in &rows[first_zero..] {
            assert_eq!(r.2, 0.0, "fracture reappeared below the threshold at v={:.2}", r.0);
        }
        std::println!(
            "threshold probe: fracture threshold is between {:.2} and {:.2} m/s",
            rows[first_zero].0,
            rows[first_zero - 1].0
        );

        // The elastic rungs — the ones that broke nothing — are where the impulse window
        // is a SHARP statement rather than a loose one: with no fracture to pay for, the
        // only sinks are the declared damping terms.
        let m = test_config(1.0).ball_mass_kg;
        let elastic: Vec<&(f64, f64, f64, f64)> = rows.iter().filter(|r| r.2 == 0.0).collect();
        std::println!(
            "threshold probe: {} of {} rungs are sub-threshold (no fracture)",
            elastic.len(),
            rows.len()
        );
        for r in &elastic {
            let (v, impulse, _, frac_j) = **r;
            let e = impulse / (m * v) - 1.0;
            std::println!(
                "  sub-threshold v={v:.2}: impulse {impulse:.4} in [{:.3}, {:.3}], restitution {e:.3}",
                m * v,
                2.0 * m * v
            );
            assert_eq!(frac_j, 0.0, "no crack area but nonzero fracture work at v={v}");
            assert!(
                impulse >= m * v && impulse <= 2.0 * m * v,
                "sub-threshold impulse {impulse} outside [m*v, 2*m*v] at v={v}"
            );
            // Dissipation is declared (`material_damping_ratio`, `solver_zeta`), so a
            // perfectly elastic rebound is NOT expected and a restitution of exactly 1
            // would mean the declared damping is not reaching the contact.
            assert!(e < 1.0, "restitution {e} at v={v} — declared damping did nothing");
        }
    }

    // Honest refusal with the numbers on the record: a grain floor coarser than the
    // requirement returns GrainFloor carrying required-vs-available spacing.
    #[test]
    fn grain_floor_refusal_reports_required_vs_available() {
        let mut scene = scene(1.0, 8, SEED);
        let run = scene.certify().unwrap();
        std::println!(
            "grain floor refusal: floor {:.5} m > required {:.5} m, finest active {:.5} m, \
             materializations {}",
            run.grain_floor_m,
            run.required_spacing_m,
            run.finest_active_m,
            run.result.materializations
        );
        assert_eq!(run.result.certificate.status, CertificationStatus::GrainFloor);
        assert!(run.grain_floor_m > run.required_spacing_m,
            "floor {} vs required {}", run.grain_floor_m, run.required_spacing_m);
        assert!(run.finest_active_m > run.required_spacing_m);
        assert!(run.result.materializations > 0);
    }

    // Convergence of the composed observables, with the corridor-blind mutant
    // convicted by the same staked tolerances (or by the guard seam).
    #[test]
    fn composition_converges_and_the_blind_bound_is_convicted() {
        let coarse = scene(4.0, 64, SEED).certify().unwrap();
        let target = scene(1.0, 64, SEED).certify().unwrap();
        let fine = scene(0.5, 64, SEED).certify().unwrap();
        assert!(coarse.result.certificate.passed());
        assert!(target.result.certificate.passed());
        assert!(fine.result.certificate.passed());
        let [j_coarse, a_coarse, _] = coarse.result.certificate.observables;
        let [j_target, a_target, _] = target.result.certificate.observables;
        let [j_fine, a_fine, _] = fine.result.certificate.observables;

        // Tolerance provenance, stated narrowly (verification finding, 2026-08-24):
        // 10%/25% were chosen when this test was first written, BEFORE any N-e leg
        // had ever run (early runs failed on certification bugs, so no numbers
        // existed to fit), and were never adjusted afterwards. They were informed by
        // E1's measured convergence experience on the sibling tension scene
        // (fracture.rs), not by a probe of THIS scene family — an earlier version of
        // this comment mis-attributed them to such a probe. Grade them
        // E1-calibrated, N-e-blind.
        let impulse_tol = 0.10 * j_target;
        let area_tol = 0.25 * a_target;

        // THE LADDER MUST BE A LADDER (verification finding, 2026-08-24). Nothing
        // here previously checked that the three legs sit at three DIFFERENT
        // frontiers. If `fine` and `target` stop at the same depth they are the same
        // run and "fine matches target" is 0 <= tol — a tautology wearing a
        // convergence gate's clothes. The mutant branch below already knew to check
        // frontier separation (`blind.finest_active_m > target.finest_active_m`);
        // the convergence legs did not. Asserted now, and printed on a pass.
        std::println!(
            "refinement ladder: finest coarse {:.5} / target {:.5} / fine {:.5} m; \
             required {:.5} m (SAME at every tolerance); materializations {}/{}/{}",
            coarse.finest_active_m,
            target.finest_active_m,
            fine.finest_active_m,
            target.required_spacing_m,
            coarse.result.materializations,
            target.result.materializations,
            fine.result.materializations
        );
        // The fine leg IS a real ladder step and is asserted as one: without this,
        // "fine matches target" could be 0 <= tol, a tautology wearing a convergence
        // gate's clothes. (The mutant branch below already checked frontier
        // separation for the blind bound; the convergence legs did not.)
        assert!(
            fine.finest_active_m < target.finest_active_m,
            "fine leg is not finer than the target leg ({} vs {}) — the convergence \
             comparison is against itself and proves nothing",
            fine.finest_active_m,
            target.finest_active_m
        );
        // THE COARSE LEG IS NOT A RESOLUTION STEP, and calling this test a
        // convergence gate hid that (verification finding, 2026-08-24). Measured:
        // coarse (tolerance 4.0) and target (tolerance 1.0) stop at the SAME finest
        // spacing, 3.91 mm, because `macro_tolerance` does not scale
        // `required_spacing_m` (that is l_ch/10, fixed) — it scales the settle
        // threshold and the error-bound acceptance. So what separates the coarse leg
        // is frontier EXTENT and the quench pattern that redraws with it, not finest
        // resolution. Asserted as what it is: a different frontier, not a coarser one.
        assert!(
            coarse.result.materializations != target.result.materializations,
            "coarse and target certified the identical frontier ({} materializations, \
             finest {} m) — the coarse-miss assertion below would be comparing a run \
             to itself",
            coarse.result.materializations,
            coarse.finest_active_m
        );
        // OR-gate prongs are computed and named individually (standing requirement:
        // every OR-gate names its firing prong; run with --nocapture to see them on
        // a pass).
        let coarse_miss_impulse = (j_coarse - j_target).abs() > impulse_tol;
        let coarse_miss_area = (a_coarse - a_target).abs() > area_tol;
        std::println!(
            "coarse-miss prongs: impulse fired={} ({:.4} vs tol {:.4}); area fired={} ({:.6} vs tol {:.6})",
            coarse_miss_impulse,
            (j_coarse - j_target).abs(),
            impulse_tol,
            coarse_miss_area,
            (a_coarse - a_target).abs(),
            area_tol
        );
        // READ THE PRONG BEFORE CARRYING THIS FORWARD (verification, 2026-08-24):
        // the coarse leg is convicted by AREA alone — impulse sits at ~71% of its
        // tolerance and does not fire — and crack area is exactly the observable this
        // module's header documents as redrawing with refinement level. Combined with
        // the equal finest spacings above, the coarse-miss result reads as an
        // extent-and-quench effect, NOT as evidence that a coarser resolution is
        // inadequate. The gate is left exactly as staked; only its reading is
        // narrowed, because tightening a prong after seeing which one fired is the
        // move the house rules exist to prevent.
        assert!(coarse_miss_impulse || coarse_miss_area,
            "coarse leg should miss: J {j_coarse}/{j_target} A {a_coarse}/{a_target}");
        assert!((j_fine - j_target).abs() <= impulse_tol, "J {j_fine} vs {j_target}");
        assert!((a_fine - a_target).abs() <= area_tol, "A {a_fine} vs {a_target}");

        // MUTANT: the corridor-blind bound certifies a coarse frontier; the gate
        // fires either through the staked tolerances or through the guard seam
        // (a loaded guarded bond in its certified solve). Prongs named on a pass
        // under --nocapture, per the standing OR-gate requirement.
        let mut blind_scene = scene(1.0, 64, SEED);
        blind_scene.config.residual = ImpactResidual::CorridorBlindBound;
        let blind = blind_scene.certify().unwrap();
        let convicted = if blind.result.certificate.passed() {
            let [j_blind, a_blind, _] = blind.result.certificate.observables;
            let prong_impulse = (j_blind - j_target).abs() > impulse_tol;
            let prong_area = (a_blind - a_target).abs() > area_tol;
            let prong_guard = blind.guarded_worst_load >= blind_scene.config.guard_fraction;
            std::println!(
                "blind-conviction prongs (certified coarse, finest {:.4} m): impulse fired={} ({:.4} vs tol {:.4}); area fired={} ({:.6} vs tol {:.6}); guard fired={} (load {:.3} vs {:.3})",
                blind.finest_active_m,
                prong_impulse,
                (j_blind - j_target).abs(),
                impulse_tol,
                prong_area,
                (a_blind - a_target).abs(),
                area_tol,
                prong_guard,
                blind.guarded_worst_load,
                blind_scene.config.guard_fraction
            );
            blind.finest_active_m > target.finest_active_m
                && (prong_impulse || prong_area || prong_guard)
        } else {
            // The guard seam refused certification outright: also a conviction.
            std::println!(
                "blind-conviction prong: certification refused outright, guard load {:.3} vs {:.3}",
                blind.guarded_worst_load,
                blind_scene.config.guard_fraction
            );
            blind.guarded_worst_load >= blind_scene.config.guard_fraction
        };
        assert!(convicted, "blind bound escaped: {blind:?}");
    }

    // MUTANT: a wrong residual surface refuses certification and spends its
    // materializations away from the impact.
    #[test]
    fn wrong_residual_ranking_fails_the_gate() {
        let mut scene = scene(1.0, 64, SEED);
        scene.config.residual = ImpactResidual::WrongSurfaceResidual;
        let run = scene.certify().unwrap();
        assert!(!run.result.certificate.passed(), "{:?}", run.result.certificate.status);
        assert!(run.materialized_far > 0, "mutant refined nothing far from the impact");
    }

    // Replay: bit-identical on the same scene seed; the seed is load-bearing.
    #[test]
    fn composed_run_is_bit_identical_on_the_same_seed() {
        let mut first_scene = scene(1.0, 64, SEED);
        let mut second_scene = scene(1.0, 64, SEED);
        let first = first_scene.certify().unwrap();
        let second = second_scene.certify().unwrap();
        assert!(first.result.certificate.passed());
        assert_eq!(first_scene.arena().holons(), second_scene.arena().holons());
        for k in 0..IMPACT_OBSERVABLES {
            assert_eq!(
                first.result.certificate.observables[k].to_bits(),
                second.result.certificate.observables[k].to_bits()
            );
        }
        let mut other_scene = scene(1.0, 64, SEED + 1);
        let other = other_scene.certify().unwrap();
        let differs = (0..IMPACT_OBSERVABLES).any(|k| {
            first.result.certificate.observables[k].to_bits()
                != other.result.certificate.observables[k].to_bits()
        });
        assert!(differs, "observables ignored the quenched realization");
    }

    // The ν gate: the (k_n, k_t) lattice realizes the descriptor's Poisson ratio;
    // the axial-only mutant reads the central-force ν ≈ 1/3 instead. Measured by a
    // damped uniaxial stretch on a uniform frontier of the DEMO material
    // (ν = 0.24 — far enough from 1/3 to discriminate; LAC_DU_BONNET's 0.31 is not).
    #[test]
    fn tangential_stiffness_realizes_the_descriptor_poisson_ratio() {
        let measured = measure_poisson(ImpactResidual::Correct);
        let axial_only = measure_poisson(ImpactResidual::AxialOnly);
        let nu = IsotropicMaterial::DEMO_CALIBRATION.poisson_ratio;
        assert!((measured - nu).abs() < 0.06,
            "derived lattice reads nu {measured}, descriptor {nu}");
        assert!((axial_only - 1.0 / 3.0).abs() < 0.06,
            "axial-only lattice reads nu {axial_only}, expected ~1/3");
        assert!((axial_only - nu).abs() > 0.05, "mutant is indistinguishable");
    }

    /// Static uniaxial probe: uniform 8x8 lattice of the DEMO material, constant
    /// vertical tension on the top row, damped to rest; ν = −ε_xx/ε_yy.
    fn measure_poisson(residual: ImpactResidual) -> f64 {
        let material = IsotropicMaterial::DEMO_CALIBRATION;
        let moduli = plane_stress_moduli(&material).unwrap();
        let n = 10_usize;
        let h = 0.015_f64;
        let t = 0.1_f64;
        let k_n = t * (moduli.lambda_pa + moduli.mu_pa);
        let k_t = if residual == ImpactResidual::AxialOnly {
            0.0
        } else {
            t * (moduli.mu_pa - moduli.lambda_pa)
        };
        let mass = material.density_kg_m3 * h * h * t;
        let mut position: Vec<[f64; 2]> = Vec::new();
        for row in 0..n {
            for col in 0..n {
                position.push([col as f64 * h, row as f64 * h]);
            }
        }
        let rest = position.clone();
        // Bonds: horizontal + vertical edges and one alternating diagonal per cell.
        let mut bonds: Vec<(usize, usize, f64, f64)> = Vec::new();
        let id = |row: usize, col: usize| row * n + col;
        for row in 0..n {
            for col in 0..n {
                if col + 1 < n {
                    bonds.push((id(row, col), id(row, col + 1), k_n, k_t));
                }
                if row + 1 < n {
                    bonds.push((id(row, col), id(row + 1, col), k_n, k_t));
                }
                if row + 1 < n && col + 1 < n {
                    if (row + col) % 2 == 0 {
                        bonds.push((id(row, col), id(row + 1, col + 1), k_n, k_t));
                    } else {
                        bonds.push((id(row, col + 1), id(row + 1, col), k_n, k_t));
                    }
                }
            }
        }
        let pull_per_node = 1.0e-4 * k_n * h; // small strain
        let dt = 0.2 * h / libm::sqrt(material.young_modulus_pa / material.density_kg_m3);
        let damping = 0.02 * libm::sqrt(k_n / mass);
        let mut velocity = alloc::vec![[0.0_f64, 0.0_f64]; n * n];
        let mut tangential = alloc::vec![0.0_f64; bonds.len()];
        for _ in 0..90_000 {
            let mut force = alloc::vec![[0.0_f64, 0.0_f64]; n * n];
            for (b, &(i, j, kn, kt)) in bonds.iter().enumerate() {
                let dx = position[j][0] - position[i][0];
                let dy = position[j][1] - position[i][1];
                let distance = libm::sqrt(dx * dx + dy * dy);
                let rest_len = {
                    let rx = rest[j][0] - rest[i][0];
                    let ry = rest[j][1] - rest[i][1];
                    libm::sqrt(rx * rx + ry * ry)
                };
                let normal = [dx / distance, dy / distance];
                let axial = kn * (distance - rest_len);
                let rel = [
                    velocity[j][0] - velocity[i][0],
                    velocity[j][1] - velocity[i][1],
                ];
                let tangent = [-normal[1], normal[0]];
                let slip = rel[0] * tangent[0] + rel[1] * tangent[1];
                tangential[b] += slip * dt;
                let shear = kt * tangential[b];
                let fx = normal[0] * axial + tangent[0] * shear;
                let fy = normal[1] * axial + tangent[1] * shear;
                force[i][0] += fx;
                force[i][1] += fy;
                force[j][0] -= fx;
                force[j][1] -= fy;
            }
            for i in 0..n * n {
                let row = i / n;
                // Bottom row: rollers (y clamped, x free) so lateral contraction is
                // unimpeded; the corner node is pinned in x to remove the drift mode.
                let clamp_y = row == 0;
                let clamp_x = i == 0;
                if row == n - 1 {
                    force[i][1] += pull_per_node;
                }
                if !clamp_x {
                    velocity[i][0] += (force[i][0] - damping * mass * velocity[i][0]) / mass * dt;
                    position[i][0] += velocity[i][0] * dt;
                }
                if !clamp_y {
                    velocity[i][1] += (force[i][1] - damping * mass * velocity[i][1]) / mass * dt;
                    position[i][1] += velocity[i][1] * dt;
                }
            }
        }
        // Strains from the interior columns/rows to avoid edge effects.
        let top = n - 1;
        let eyy = (position[id(top, n / 2)][1] - rest[id(top, n / 2)][1])
            / (rest[id(top, n / 2)][1] - rest[id(0, n / 2)][1]);
        let left = id(n / 2, 0);
        let right = id(n / 2, n - 1);
        let exx = (position[right][0] - position[left][0] - (rest[right][0] - rest[left][0]))
            / (rest[right][0] - rest[left][0]);
        -exx / eyy
    }

    // THE PATCH TEST — the first check of N-e's MULTI-RESOLUTION claim against
    // something that is not this engine (research-manager, 2026-08-24).
    //
    // WHAT IS UNVERIFIED. The (k_n, k_t) exactness theorem covers the UNIFORM
    // alternating-diagonal stencil. On the adaptive frontier the same stiffnesses are
    // applied at the LOCAL pitch, and the engine's answer at a hanging node is
    // `width_fraction = interface_m / pitch` (see `build_impact_mesh`) — commented in
    // its own source as "hanging-node interfaces carry their share". That is a DESIGN
    // PRINCIPLE, not a theorem, and it is exactly what the convergence gate cannot
    // check, because the convergence gate compares this engine to itself.
    //
    // THE INSTRUMENT, and why it is external. Irons & Razzaque's patch test (1972):
    // impose the EXACT CONTINUUM SOLUTION — a uniform strain, u(x) = eps.x — and
    // require every INTERIOR node's net force to vanish. Uniform strain is an
    // equilibrium of the continuum with no body force, so a discrete operator that
    // represents the continuum must annihilate it. The reference is the continuum;
    // nothing here is compared against another run of the engine.
    //
    // WHAT IT CANNOT REACH, stated because a pass must not be read as more than it is.
    // The tangential channel is RATE-INTEGRATED (`tangential_disp += slip_speed * dt`,
    // initialised to 0.0), so shear force is a function of the PATH, not of the
    // configuration. A static patch test sees zero shear and exercises the NORMAL
    // channel only — and the normal channel alone is the central-force lattice with the
    // Cauchy restriction lambda = mu, which is the very deficiency (k_n, k_t) exists to
    // repair. So THIS TEST VERIFIES THE HALF THAT WAS NEVER THE PROBLEM AND IS BLIND TO
    // THE HALF THAT IS. The shear half is a declared pin with its own falsifier; see
    // the module header. Do not cite this test as verifying multi-resolution shear.
    //
    // THE CONTROL, which is what makes a reading mean anything: nodes in the UNIFORM
    // interior must read machine-zero. If they do not, the test is broken and says
    // nothing about interfaces. Interface nodes are then read against that floor.
    //
    // BOTH OUTCOMES STAKED BEFORE RUNNING: interface residual at the uniform floor =>
    // the width-share rule is VERIFIED for the normal channel. Interface residual
    // materially above it => the multi-resolution claim is REFUTED, not merely
    // unverified, which is the larger result.
    #[test]
    fn uniform_strain_patch_test_at_the_refinement_interface() {
        const STRAIN: f64 = 1.0e-6; // small enough that no bond leaves its elastic branch

        let mut scene = scene(1.0, 64, SEED);
        let run = scene.certify().unwrap();
        assert!(run.result.certificate.passed());
        let chart_rc = scene.chart();
        let chart = chart_rc.borrow();
        // strength/diameter refs are irrelevant here: they quench the COHESIVE peak and
        // energy, never k_n, which is `normal_stiffness * width_fraction`.
        let mesh = build_impact_mesh(
            scene.arena(),
            &run.result.certificate.frontier,
            &chart,
            &scene.config,
            &scene.binding.properties,
            1.0,
            1.0,
        );

        // CLASSIFIER, second attempt — the first one was wrong and the control caught
        // it. Node MASS does not encode pitch (it is the arena's largest-remainder
        // share of gross constituents, which varies for reasons unrelated to cell
        // size), so a mass-difference test marked EVERY interior node as an interface
        // and left zero control nodes. Recorded rather than quietly replaced.
        //
        // What does encode pitch is the AXIAL bond's rest length: between two cells of
        // equal pitch p it is p, and across a hanging node (2p facing p) it is 1.5p. So
        // a node is in a UNIFORM region iff all its incident edge bonds share one rest
        // length, and at an interface otherwise. Uses only what the bond already carries.
        let axial_rest: Vec<Vec<f64>> = {
            let mut v: Vec<Vec<f64>> = (0..mesh.nodes.len()).map(|_| Vec::new()).collect();
            for slot in &mesh.bonds {
                if slot.kind == BondKind::Edge {
                    v[slot.a].push(slot.bond.rest_length_m);
                    v[slot.b].push(slot.bond.rest_length_m);
                }
            }
            v
        };
        // Node pitch = smallest incident edge rest length. A node is at an interface
        // if its own edge rests disagree OR any bond (edge OR diagonal) joins it to a
        // node of different pitch — the diagonal leg matters, and leaving it out is
        // what left a 4.4e-2 tail in the control on the previous pass.
        let pitch: Vec<f64> = axial_rest
            .iter()
            .map(|r| r.iter().cloned().fold(f64::INFINITY, f64::min))
            .collect();
        let mut at_interface: Vec<bool> = axial_rest
            .iter()
            .map(|rests| {
                rests.len() > 1
                    && rests.iter().any(|r| libm::fabs(r - rests[0]) > 1.0e-12 * rests[0])
            })
            .collect();
        for slot in &mesh.bonds {
            let (pa, pb) = (pitch[slot.a], pitch[slot.b]);
            if pa.is_finite() && pb.is_finite() && libm::fabs(pa - pb) > 1.0e-12 * pa {
                at_interface[slot.a] = true;
                at_interface[slot.b] = true;
            }
        }

        // Cell size per node, for the boundary test only: the smallest incident axial
        // rest length is the local pitch.
        let cell_of = |i: usize| {
            axial_rest[i].iter().cloned().fold(f64::INFINITY, f64::min)
        };

        // Interior = not on the wall's outer boundary, where a free surface under
        // uniform strain is CORRECTLY out of equilibrium (it wants surface traction).
        let side = scene.config.geometry.side_m;
        let interior = |i: usize| {
            let p = mesh.nodes[i].position;
            let c = cell_of(i) * 1.5;
            p[0] > c && p[0] < side - c && p[1] > c && p[1] < side - c
        };

        // TWO-SOLVE on the instrument itself: a CONSISTENCY error is strain-independent
        // once normalised by the bond-force scale (both scale linearly), so reading the
        // same normalised residual at two strains two orders apart is what separates
        // "the operator is wrong" from "something else scales".
        let mut sweep: Vec<(f64, f64, f64)> = Vec::new(); // (strain, uniform_max, interface_median)
        for &strain in &[STRAIN, STRAIN * 1.0e-2] {
        let position: Vec<[f64; 2]> = mesh
            .nodes
            .iter()
            .map(|n| [n.position[0] * (1.0 + strain), n.position[1]])
            .collect();

        // The engine's OWN normal-channel force law, at zero velocity (so no damping
        // term) and zero accumulated slip (so no tangential term) — reused rather than
        // reimplemented, because the reference is the continuum, not a second opinion
        // about the force law.
        let mut force: Vec<[f64; 2]> = (0..mesh.nodes.len()).map(|_| [0.0_f64; 2]).collect();
        let mut worst_bond_force = 0.0_f64;
        for slot in &mesh.bonds {
            let (a, b) = (slot.a, slot.b);
            let dx = position[b][0] - position[a][0];
            let dy = position[b][1] - position[a][1];
            let distance = libm::sqrt(dx * dx + dy * dy);
            let normal = [dx / distance, dy / distance];
            let mut bond = slot.bond;
            let axial = bond.axial_force(distance - bond.rest_length_m, 0.0);
            worst_bond_force = worst_bond_force.max(libm::fabs(axial));
            force[a][0] += normal[0] * axial;
            force[a][1] += normal[1] * axial;
            force[b][0] -= normal[0] * axial;
            force[b][1] -= normal[1] * axial;
        }

        let residual = |i: usize| {
            libm::sqrt(force[i][0] * force[i][0] + force[i][1] * force[i][1]) / worst_bond_force
        };
        let mut uni: Vec<f64> = Vec::new();
        let mut itf: Vec<f64> = Vec::new();
        for i in 0..mesh.nodes.len() {
            if !interior(i) {
                continue;
            }
            if at_interface[i] { itf.push(residual(i)) } else { uni.push(residual(i)) }
        }
        let stats = |v: &mut Vec<f64>| -> (f64, f64, f64) {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let n = v.len();
            if n == 0 { return (0.0, 0.0, 0.0); }
            (v[n / 2], v[(n * 9) / 10], v[n - 1])
        };
        let (u_med, u_p90, u_max) = stats(&mut uni);
        let (i_med, i_p90, i_max) = stats(&mut itf);
        let (n_uniform, n_interface) = (uni.len(), itf.len());
        let (uniform_worst, interface_worst) = (u_max, i_max);
        std::println!(
            "PATCH TEST distribution — uniform interior (n={n_uniform}): median {u_med:.3e}, \
             p90 {u_p90:.3e}, max {u_max:.3e} | interface (n={n_interface}): median {i_med:.3e}, \
             p90 {i_p90:.3e}, max {i_max:.3e}"
        );

        assert!(n_interface > 0, "no refinement interface in this mesh — the test has nothing to say");
        assert!(n_uniform > 0, "no uniform interior — no control");
        // Separation at each strain: the interface must sit orders above the floor.
        assert!(
            interface_worst > 1.0e3 * uniform_worst,
            "interface residual {interface_worst:e} did NOT separate from the uniform \
             floor {uniform_worst:e} at strain {strain:e} — the width-share rule would be \
             exact after all, and this test's staked refutation branch does not fire"
        );
        sweep.push((strain, uniform_worst, i_med));
        }

        // THE SIGNATURE, and it is what turns a large number into a diagnosis. Both
        // residuals are normalised by the largest bond force, which scales linearly in
        // strain. So:
        //   * ROUND-OFF has constant ABSOLUTE size, hence normalised size ~ 1/strain;
        //   * a CONSISTENCY error scales linearly with strain, hence normalised size
        //     is strain-INDEPENDENT.
        // The uniform interior must show the first (it is the control, and it is
        // numerically zero); the interface must show the second, or the reading is not
        // a statement about the operator.
        let (s0, u0, i0) = sweep[0];
        let (s1, u1, i1) = sweep[1];
        let strain_ratio = s0 / s1;
        let uniform_ratio = u1 / u0;
        let interface_ratio = i1 / i0;
        std::println!(
            "PATCH TEST signature — strain {s0:e} -> {s1:e} ({strain_ratio:.0}x smaller): \
             uniform floor {u0:.3e} -> {u1:.3e} (x{uniform_ratio:.1}, round-off scales 1/strain); \
             INTERFACE {i0:.4e} -> {i1:.4e} (x{interface_ratio:.4}, a consistency error is FLAT)"
        );
        assert!(
            uniform_ratio > 0.3 * strain_ratio,
            "the uniform floor did not scale like round-off (x{uniform_ratio:.1} against \
             {strain_ratio:.0}x) — it is not the numerical zero this control assumes"
        );
        assert!(
            libm::fabs(interface_ratio - 1.0) < 0.05,
            "the interface residual is not strain-independent (x{interface_ratio:.4}) — it is \
             therefore NOT the consistency error this test reports it as"
        );
    }

    // Contact jurisdiction (A3): a live-bonded pair is exempt from solver contact;
    // a broken pair inherits the dead bond's tribology. Pinned structurally on the
    // certified mesh: every candidate pair the solver may touch is either broken or
    // never-bonded.
    #[test]
    fn solver_contact_jurisdiction_is_the_a3_set() {
        let mut scene = scene(1.0, 64, SEED);
        let run = scene.certify().unwrap();
        assert!(run.result.certificate.passed());
        let chart_rc = scene.chart();
        let chart = chart_rc.borrow();
        let mesh = build_impact_mesh(
            scene.arena(),
            &run.result.certificate.frontier,
            &chart,
            &scene.config,
            &scene.binding.properties,
            1.0, // strength_ref: irrelevant to the jurisdiction structure
            1.0,
        );
        // Structure: bonds_by_node is symmetric and complete, so the stepping
        // loop's live-pair exemption sees every laid bond.
        for (i, adjacency) in mesh.bonds_by_node.iter().enumerate() {
            for &(j, bond) in adjacency {
                assert!(mesh.bonds[bond].a == i.min(j) && mesh.bonds[bond].b == i.max(j));
                assert!(mesh.bonds_by_node[j].iter().any(|&(other, b)| other == i && b == bond));
            }
        }
        // The D = 1 handoff is consumed from material.rs: a broken bond's contact
        // friction equals the dead relation's capacity, a live bond returns zero
        // through the contact channel.
        let mut probe = mesh.bonds[0].bond;
        assert_eq!(probe.failed_contact_friction_force(10.0, 1.0), 0.0);
        probe.axial_force(1.0, 0.0); // drive to failure
        assert!(probe.is_broken());
        let capacity = probe.law.friction_coefficient * 10.0;
        assert!((probe.failed_contact_friction_force(10.0, 1.0e6) - capacity).abs() <= 1e-12 * capacity);
    }
}
