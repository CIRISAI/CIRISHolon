//! Rust/WASM Newtonian material-fracture gate for CIRISHolon.
//!
//! The browser owns input and pixels. This crate owns every state transition, contact,
//! cohesive relation, and crack. The 10,000-holon ball and 1,000,000-holon wall retain
//! exact REG+ constituent counts while a fixed resident frontier resolves the impact.

use ciris_sim_core::homogenization::{derive_bilinear_cohesive_law, derive_lattice_elastic_law};
use ciris_sim_core::material::{
    CohesiveBond, CohesiveLaw, IsotropicMaterial, MaterialBinding, RigidChartExport,
};
use ciris_sim_core::regplus::GrossState;
use std::sync::{Mutex, MutexGuard};

pub const BALL_HOLONS: u64 = 10_000;
pub const WALL_HOLONS: u64 = 1_000_000;
pub const ROOT_HOLON: usize = 0;
pub const BALL_HOLON: usize = 1;
pub const WALL_HOLON: usize = 2;
pub const STONE_DESCRIPTOR_HOLON: usize = 3;
pub const BALL_GROSS: GrossState = GrossState::aggregate(BALL_HOLONS, 0, [0, 0]);
pub const WALL_GROSS: GrossState = GrossState::aggregate(WALL_HOLONS, 0, [0, 0]);
pub const SCENE_GROSS: GrossState = BALL_GROSS.combine(WALL_GROSS);

const WALL_COLUMNS: usize = 18;
const WALL_ROWS: usize = 16;
const WALL_NODE_COUNT: usize = WALL_COLUMNS * WALL_ROWS;
const WALL_X: f64 = 7.05;
const WALL_Y: f64 = 1.28;
// P2 values consequence: the (k_n, k_t) homogenization bounds the bilinear cohesive
// chart at h_max = 2 G_F (lambda+mu) / f_t^2 = 0.181 m for DEMO_CALIBRATION — a CELL
// bound, thickness-independent, refusing the old 0.245 m spacing for every bond
// family (homogenization.rs). Spacing is the only lever, so it moved: 0.14 m keeps
// the same 18x16 topology and node masses (the gross ledger is untouched) with a
// softening ratio h_max/h = 1.29. Node radius scales with the lattice.
const WALL_SPACING: f64 = 0.14;
const NODE_RADIUS: f64 = 0.4 * WALL_SPACING;
const NODE_MASS: f64 = 0.72;
const NODE_MOMENT_OF_INERTIA: f64 = 0.5 * NODE_MASS * NODE_RADIUS * NODE_RADIUS;
const BALL_RADIUS: f64 = 0.34;
// The resident mass ratio follows the gross count ratio: the ball is approximately one
// percent of the wall, rather than being enlarged merely to make the demo dramatic.
const BALL_MASS: f64 = 2.1;
const BALL_START_X: f64 = 1.25;
const BALL_START_Y: f64 = 3.18;
// Gravity is SCENE-CHART data, not holon data: the equivalence principle makes a
// uniform g frame-equivalent to an accelerated chart, and universality of free fall
// means no holon may carry its own g. Every massive body in the scene reads THIS
// value through chart_gravity_force; any per-body deviation is a stage knob that must
// be named where it acts, never hidden in an expression.
//
// STAGE VALUE (A5 idiom): 1.8 m/s^2 is demo pacing, not Earth's 9.81 — unwarranted
// by any physics tier, and named as such.
const CHART_GRAVITY_M_S2: f64 = 1.8;
const FIXED_STEP: f64 = 1.0 / 600.0;
const MAX_FRAME_STEP: f64 = 1.0 / 20.0;

// A5 — solver damping, named as such. These stabilize the explicit integrator and the
// penalty contact; they are solver configuration with no physics-tier ancestor, and
// they are declared here instead of hiding inline in the stepping loop. Material
// dissipation (the descriptor's `material_damping_ratio`, granite band ζ ~ 5e-4–5e-3)
// is deliberately NOT consumed by this demo's integrator.
const SOLVER_VELOCITY_DAMPING_PER_S: f64 = 0.10;
const SOLVER_CONTACT_STIFFNESS_N_M: f64 = 2_150.0;
const SOLVER_CONTACT_DAMPING_N_S_M: f64 = 13.0;

// Byerlee-class rock friction is 0.6–0.85; 0.74 is the T4 spec's McClintock–Walsh
// inversion of the demo strength ratio (DESCRIPTOR_CHAIN §3.4, C3-corrected form).
const BOND_FRICTION_MU: f64 = 0.74;

// Coulomb friction for solver contact between NEVER-BONDED node pairs. The values are
// deliberately EQUAL to the bond law's (friction_coefficient, damping_n_s_m) — a
// failed pair inherits its dead bond's tribology by the D = 1 handoff contract
// (regime table on CohesiveBond), and never-bonded faces of the same wall must not
// slide differently from failed ones. A test pins this equality.
const SOLVER_CONTACT_FRICTION_MU: f64 = BOND_FRICTION_MU;
const SOLVER_CONTACT_FRICTION_DAMPING_N_S_M: f64 = STAGE_BOND_DAMPING_N_S_M;

// STAGE KNOB, named as such (A5 idiom) and MEASURED before keeping — remeasured on
// the DERIVED (k_n, k_t) wall (P3, 2026-08-23): under uniform chart gravity the
// cantilevered wall still cannot carry its own weight. Sweep of the stage peak-force
// anchor at factor 1.0: 12 N (derived) through 80 N all tear off the anchor column
// within seconds (265-272 free nodes detached, then free fall); survival needs
// 96 N — a ~7-8x strength margin — and even the survivor droops ~2 m at the
// playability-anchored 510 N/m stiffness. So the knob stays, with its cost stated
// plainly: it BREAKS universality of free fall in this demo — severed fragments
// fall at 0.035 g while the ball falls at g. Deleting it is a VALUES change needing
// BOTH ~8x strength and a much stiffer stage map (a different game feel — Eric's
// call), never a hidden multiplier. The universality gate pins the chart as uniform
// and this factor as the single named deviation, applied at exactly one call site.
const STAGE_WALL_GRAVITY_FACTOR: f64 = 0.035;

// P2 — the bond law is DERIVED, not hand-tuned: homogenization.rs turns the
// descriptor's (E, nu, f_t, G_F) and the lattice geometry into per-relation
// (k_n, k_t) and the cohesive (peak, G); the NAMED stage similarity map
// (CohesiveLaw::stage_scaled) then relabels force and opening axes. The two anchors
// below are the map's only free choices, pinned to the pre-P2 stage values so the
// throw feel survives; everything else — fracture energy, failure opening,
// brittleness, the k_t/k_n ratio that carries nu — follows from the continuum.
const STAGE_BOND_STIFFNESS_N_M: f64 = 510.0; // playability anchor (pre-P2 stage value)
const STAGE_BOND_PEAK_N: f64 = 12.0; // playability anchor (pre-P2 stage value)
const STAGE_BOND_DAMPING_N_S_M: f64 = 2.6; // solver regularization at stage scale (A5)

/// The stage-scaled lattice realization: the cohesive law shared by every bond plus
/// the tangential elastic stiffness that breaks the stencil's Cauchy nu = 1/3
/// restriction. nu depends only on k_t/k_n, which the similarity map preserves, so
/// the stage wall realizes the descriptor's nu exactly (gated by test).
struct StageLattice {
    law: CohesiveLaw,
    tangential_stiffness_n_m: f64,
}

fn stage_lattice() -> StageLattice {
    let material = STONE_BINDING.properties;
    let elastic = derive_lattice_elastic_law(&material, NODE_MASS, WALL_SPACING)
        .expect("demo stencil realizes the descriptor's (E, nu)");
    let cohesive = derive_bilinear_cohesive_law(&material, NODE_MASS, WALL_SPACING)
        .expect("demo spacing is inside the bilinear validity domain");
    let si_law = CohesiveLaw {
        stiffness_n_m: elastic.normal_stiffness_n_m,
        damping_n_s_m: STAGE_BOND_DAMPING_N_S_M,
        peak_force_n: cohesive.peak_force_n,
        fracture_energy_j: cohesive.fracture_energy_j,
        friction_coefficient: BOND_FRICTION_MU,
    };
    let force_scale = STAGE_BOND_PEAK_N / cohesive.peak_force_n;
    let opening_scale = elastic.normal_stiffness_n_m * force_scale / STAGE_BOND_STIFFNESS_N_M;
    StageLattice {
        law: si_law.stage_scaled(force_scale, opening_scale),
        tangential_stiffness_n_m: elastic.tangential_stiffness_n_m * force_scale
            / opening_scale,
    }
}

const STONE_BINDING: MaterialBinding = MaterialBinding {
    subject_holon: WALL_HOLON,
    descriptor_holon: STONE_DESCRIPTOR_HOLON,
    properties: IsotropicMaterial::DEMO_CALIBRATION,
};

#[derive(Clone, Copy, Debug, Default)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    const ZERO: Self = Self { x: 0.0, y: 0.0 };

    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    fn normalized(self) -> Self {
        let length = self.length();
        if length > 1.0e-12 {
            self * (1.0 / length)
        } else {
            Self::new(1.0, 0.0)
        }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

#[derive(Clone, Copy, Debug)]
struct Node {
    position: Vec2,
    velocity: Vec2,
    force: Vec2,
    /// Spin about the out-of-plane axis. Required for a frame-indifferent tangential
    /// bond spring: with point masses alone, relative tangential motion of a pair is
    /// indistinguishable from rigid rotation, so the (k_n, k_t) realization carries
    /// rotational state exactly as the BPM literature does.
    angular_velocity: f64,
    torque: f64,
    anchored: bool,
}

#[derive(Clone, Copy, Debug)]
struct Ball {
    position: Vec2,
    velocity: Vec2,
    force: Vec2,
    launched: bool,
}

#[derive(Clone, Debug)]
struct Bond {
    a: usize,
    b: usize,
    relation: CohesiveBond,
    weak_interface: bool,
    /// Accumulated interface slip carried as a co-rotating scalar along the bond's
    /// current tangential direction; the tangential elastic spring acts on it.
    tangential_displacement_m: f64,
}

#[derive(Debug)]
struct Simulation {
    nodes: Vec<Node>,
    bonds: Vec<Bond>,
    /// Per-node list of (other node, bond index), built at reset. The contact solver
    /// consults it to exempt pairs still joined by a live (D < 1) bond.
    bonds_by_node: Vec<Vec<(usize, usize)>>,
    /// Stage-scaled k_t shared by every live relation (see StageLattice).
    stage_tangential_stiffness_n_m: f64,
    ball: Ball,
    time: f64,
    impacts: u32,
    peak_contact_force: f64,
    initialized: bool,
}

impl Simulation {
    const fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            bonds: Vec::new(),
            bonds_by_node: Vec::new(),
            stage_tangential_stiffness_n_m: 0.0,
            ball: Ball {
                position: Vec2 {
                    x: BALL_START_X,
                    y: BALL_START_Y,
                },
                velocity: Vec2::ZERO,
                force: Vec2::ZERO,
                launched: false,
            },
            time: 0.0,
            impacts: 0,
            peak_contact_force: 0.0,
            initialized: false,
        }
    }

    fn reset(&mut self) {
        debug_assert!(STONE_BINDING.validate().is_ok());
        self.nodes.clear();
        self.bonds.clear();
        self.bonds_by_node.clear();
        self.time = 0.0;
        self.impacts = 0;
        self.peak_contact_force = 0.0;
        self.ball = Ball {
            position: Vec2::new(BALL_START_X, BALL_START_Y),
            velocity: Vec2::ZERO,
            force: Vec2::ZERO,
            launched: false,
        };

        for row in 0..WALL_ROWS {
            for column in 0..WALL_COLUMNS {
                self.nodes.push(Node {
                    position: Vec2::new(
                        WALL_X + column as f64 * WALL_SPACING,
                        WALL_Y + row as f64 * WALL_SPACING,
                    ),
                    velocity: Vec2::ZERO,
                    force: Vec2::ZERO,
                    angular_velocity: 0.0,
                    torque: 0.0,
                    anchored: column == WALL_COLUMNS - 1,
                });
            }
        }

        let stage = stage_lattice();
        self.stage_tangential_stiffness_n_m = stage.tangential_stiffness_n_m;
        for row in 0..WALL_ROWS {
            for column in 0..WALL_COLUMNS {
                let a = node_index(column, row);
                if column + 1 < WALL_COLUMNS {
                    self.add_bond(a, node_index(column + 1, row), stage.law, column == 8);
                }
                if row + 1 < WALL_ROWS {
                    let flaw = deterministic_flaw(column, row);
                    self.add_bond(a, node_index(column, row + 1), stage.law, flaw);
                }
                if column + 1 < WALL_COLUMNS && row + 1 < WALL_ROWS {
                    let crossing_seam = column == 8;
                    if (column + row) % 2 == 0 {
                        self.add_bond(a, node_index(column + 1, row + 1), stage.law, crossing_seam);
                    } else {
                        self.add_bond(
                            node_index(column + 1, row),
                            node_index(column, row + 1),
                            stage.law,
                            crossing_seam,
                        );
                    }
                }
            }
        }
        self.initialized = true;
    }

    fn add_bond(&mut self, a: usize, b: usize, base: CohesiveLaw, weak: bool) {
        let delta = self.nodes[b].position - self.nodes[a].position;
        let law = if weak {
            base.weakened(0.42, 0.30)
        } else {
            base
        };
        let relation_holon = 10_000 + self.bonds.len();
        let relation = CohesiveBond::new(relation_holon, 100 + a, 100 + b, delta.length(), law)
            .expect("game cohesive law is valid");
        if self.bonds_by_node.len() < self.nodes.len() {
            self.bonds_by_node.resize(self.nodes.len(), Vec::new());
        }
        let bond_index = self.bonds.len();
        self.bonds_by_node[a].push((b, bond_index));
        self.bonds_by_node[b].push((a, bond_index));
        self.bonds.push(Bond {
            a,
            b,
            relation,
            weak_interface: weak,
            tangential_displacement_m: 0.0,
        });
    }

    /// The bond joining two nodes, if one was ever laid. Each node carries at most
    /// eight lattice bonds, so the scan is O(1) and runs only for pairs already found
    /// overlapping.
    fn bond_between(&self, i: usize, j: usize) -> Option<usize> {
        self.bonds_by_node[i]
            .iter()
            .find(|&&(other, _)| other == j)
            .map(|&(_, bond)| bond)
    }

    /// Whether a live (D < 1) bond joins the two nodes — such a pair is exempt from
    /// solver contact, because the bond owns the closed regime (regime table on
    /// `CohesiveBond`). The stepping loop inlines this via `bond_between` (it also
    /// needs the dead bond's law); tests use it to define the solver-owned pair set.
    #[cfg(test)]
    fn live_bond_between(&self, i: usize, j: usize) -> bool {
        self.bond_between(i, j)
            .is_some_and(|bond| !self.bonds[bond].relation.is_broken())
    }

    fn ensure_initialized(&mut self) {
        if !self.initialized {
            self.reset();
        }
    }

    fn launch(&mut self, target_y: f64, speed: f64) {
        self.reset();
        let target = Vec2::new(WALL_X + 0.65, target_y.clamp(WALL_Y, wall_top()));
        let direction = (target - self.ball.position).normalized();
        self.ball.velocity = direction * speed.clamp(7.0, 18.0);
        self.ball.launched = true;
    }

    fn advance(&mut self, elapsed: f64) {
        self.ensure_initialized();
        if !self.ball.launched {
            return;
        }
        let mut remaining = elapsed.clamp(0.0, MAX_FRAME_STEP);
        while remaining > 1.0e-12 {
            let dt = remaining.min(FIXED_STEP);
            self.substep(dt);
            remaining -= dt;
        }
    }

    fn substep(&mut self, dt: f64) {
        for node in &mut self.nodes {
            node.force = if node.anchored {
                Vec2::ZERO
            } else {
                chart_gravity_force(NODE_MASS) * STAGE_WALL_GRAVITY_FACTOR
            };
            node.torque = 0.0;
        }
        self.ball.force = chart_gravity_force(BALL_MASS);

        for bond in &mut self.bonds {
            let a = self.nodes[bond.a];
            let b = self.nodes[bond.b];
            let delta = b.position - a.position;
            let distance = delta.length();
            if distance <= 1.0e-12 {
                continue;
            }
            let normal = delta * (1.0 / distance);
            let relative_velocity = b.velocity - a.velocity;
            let relative_speed = relative_velocity.dot(normal);
            let magnitude = bond
                .relation
                .axial_force(distance - bond.relation.rest_length_m, relative_speed);
            let force = normal * magnitude;
            self.nodes[bond.a].force += force;
            self.nodes[bond.b].force -= force;

            // Tangential channel of the live interface. Slip is measured at the
            // interface midpoint INCLUDING node spins, so a rigid rotation of a
            // fragment produces exactly zero slip (frame indifference — the reason
            // nodes carry rotational state). Two contributions, (1-D)-split like the
            // normal channel: the tangential ELASTIC spring k_t on the still-bonded
            // fraction (the (k_n, k_t) realization of the descriptor's nu,
            // homogenization.rs), and the A3 Coulomb slider on the decohered
            // fraction. The bond owns both while D < 1; the contact solver owns
            // post-failure contact.
            let tangential_direction = Vec2::new(-normal.y, normal.x);
            let half_length = 0.5 * distance;
            let slip_speed = relative_velocity.dot(tangential_direction)
                - (self.nodes[bond.a].angular_velocity + self.nodes[bond.b].angular_velocity)
                    * half_length;
            let mut force_on_a_tangential = 0.0;
            if !bond.relation.is_broken() {
                bond.tangential_displacement_m += slip_speed * dt;
                force_on_a_tangential += (1.0 - bond.relation.damage())
                    * self.stage_tangential_stiffness_n_m
                    * bond.tangential_displacement_m;
            }
            let slider = bond.relation.closed_friction_force(magnitude, slip_speed);
            if slider > 0.0 {
                force_on_a_tangential += slider * slip_speed.signum();
            }
            if force_on_a_tangential != 0.0 {
                let force = tangential_direction * force_on_a_tangential;
                let torque = half_length * force_on_a_tangential;
                if !self.nodes[bond.a].anchored {
                    self.nodes[bond.a].force += force;
                    self.nodes[bond.a].torque += torque;
                }
                if !self.nodes[bond.b].anchored {
                    self.nodes[bond.b].force -= force;
                    self.nodes[bond.b].torque += torque;
                }
            }
        }

        // A3, D = 1 row made real: the contact solver that owns fully-failed
        // interfaces now exists for node-node pairs, so severed fragments contact
        // instead of interpenetrating. Pairs still joined by a live (D < 1) bond are
        // EXEMPT — the bond owns the closed regime — so the solver's jurisdiction is
        // exactly {D = 1 pairs} ∪ {never-bonded pairs}. Same penalty form as the
        // ball-node law below; the dissipation is per-pair solver configuration (A5),
        // not a material constant. Cost, measured native release build mid-impact:
        // the naive all-pairs scan over 288 nodes (41,328 candidates) costs ~34 µs of
        // a ~42 µs total substep (7.9 µs without it), ~2% of the real-time budget at
        // 600 Hz substeps — no broadphase needed at this frontier size.
        let contact_diameter = 2.0 * NODE_RADIUS;
        for (i, j) in node_contact_candidates(self.nodes.len()) {
            let delta = self.nodes[j].position - self.nodes[i].position;
            let distance_sq = delta.dot(delta);
            if distance_sq >= contact_diameter * contact_diameter || distance_sq <= 1.0e-24 {
                continue;
            }
            let pair_bond = self.bond_between(i, j);
            if pair_bond.is_some_and(|bond| !self.bonds[bond].relation.is_broken()) {
                continue;
            }
            let distance = distance_sq.sqrt();
            let normal = delta * (1.0 / distance);
            let relative_velocity = self.nodes[j].velocity - self.nodes[i].velocity;
            let separating_speed = relative_velocity.dot(normal);
            let magnitude = (SOLVER_CONTACT_STIFFNESS_N_M * (contact_diameter - distance)
                - SOLVER_CONTACT_DAMPING_N_S_M * separating_speed)
                .max(0.0);
            // Force on j: pushed apart along +normal, sliding opposed along the
            // tangential direction; i receives the exact opposite.
            let mut force_on_j = normal * magnitude;

            // D = 1 handoff contract: a failed pair's crack face inherits the dead
            // bond's tribology, so the Coulomb capacity starts at exactly the
            // μ·|F_n| the live slider tended to; never-bonded pairs use the named
            // solver constants (pinned equal to the stage law's by test).
            let tangential = relative_velocity - normal * separating_speed;
            let tangential_speed = tangential.length();
            if tangential_speed > 1.0e-12 && magnitude > 0.0 {
                let friction = match pair_bond {
                    Some(bond) => self.bonds[bond]
                        .relation
                        .failed_contact_friction_force(magnitude, tangential_speed),
                    None => (SOLVER_CONTACT_FRICTION_DAMPING_N_S_M * tangential_speed)
                        .min(SOLVER_CONTACT_FRICTION_MU * magnitude),
                };
                force_on_j -= tangential * (friction / tangential_speed);
            }
            if !self.nodes[j].anchored {
                self.nodes[j].force += force_on_j;
            }
            if !self.nodes[i].anchored {
                self.nodes[i].force -= force_on_j;
            }
        }

        let mut contact_force_total = 0.0;
        let mut contact = false;
        for node in &mut self.nodes {
            let delta = node.position - self.ball.position;
            let distance = delta.length();
            let overlap = BALL_RADIUS + NODE_RADIUS - distance;
            if overlap <= 0.0 {
                continue;
            }
            let normal = delta.normalized();
            let separating_speed = (node.velocity - self.ball.velocity).dot(normal);
            let magnitude = (SOLVER_CONTACT_STIFFNESS_N_M * overlap
                - SOLVER_CONTACT_DAMPING_N_S_M * separating_speed)
                .max(0.0);
            let force = normal * magnitude;
            if !node.anchored {
                node.force += force;
            }
            self.ball.force -= force;
            contact_force_total += magnitude;
            contact = true;
        }
        if contact && self.peak_contact_force == 0.0 {
            self.impacts += 1;
        }
        self.peak_contact_force = self.peak_contact_force.max(contact_force_total);

        for node in &mut self.nodes {
            if node.anchored {
                node.velocity = Vec2::ZERO;
                node.angular_velocity = 0.0;
                continue;
            }
            node.velocity += node.force * (dt / NODE_MASS);
            node.velocity = node.velocity * (1.0 / (1.0 + SOLVER_VELOCITY_DAMPING_PER_S * dt));
            node.position += node.velocity * dt;
            node.angular_velocity += node.torque * (dt / NODE_MOMENT_OF_INERTIA);
            node.angular_velocity /= 1.0 + SOLVER_VELOCITY_DAMPING_PER_S * dt;
        }
        self.ball.velocity += self.ball.force * (dt / BALL_MASS);
        self.ball.position += self.ball.velocity * dt;
        self.time += dt;
    }

    fn cracked_bonds(&self) -> usize {
        self.bonds
            .iter()
            .filter(|bond| bond.relation.is_broken())
            .count()
    }

    fn maximum_damage(&self) -> f64 {
        self.bonds
            .iter()
            .map(|bond| bond.relation.damage())
            .fold(0.0, f64::max)
    }

    /// A4: the wall's rigid export is the engine's OWN chart over the same holon,
    /// carrying the relation network's damage Record — never a Rapier object (Rapier
    /// stays the limiting-case control). See `RigidChartExport` for the
    /// `repairable_does_not_factor` reading of the tag.
    fn rigid_export(&self) -> RigidChartExport {
        RigidChartExport::over(
            WALL_HOLON,
            WALL_GROSS,
            WALL_NODE_COUNT as f64 * NODE_MASS,
            self.bonds.iter().map(|bond| &bond.relation),
        )
        .expect("wall rigid chart is valid")
    }
}

/// The scene chart's gravity on a massive body. Uniform in the mass by construction:
/// acceleration is CHART_GRAVITY_M_S2 for every body, which is the universality of
/// free fall, gated by test.
fn chart_gravity_force(mass_kg: f64) -> Vec2 {
    Vec2::new(0.0, -CHART_GRAVITY_M_S2 * mass_kg)
}

fn node_index(column: usize, row: usize) -> usize {
    row * WALL_COLUMNS + column
}

/// Pair-candidate source for node-node solver contact. Today this is every pair of
/// the fixed 288-node resident frontier; the adaptive crack-tip lane (E1) will draw
/// candidates from the refined frontier instead, which is why the source is a
/// function and not a double loop baked into the stepper.
fn node_contact_candidates(count: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..count).flat_map(move |i| ((i + 1)..count).map(move |j| (i, j)))
}

fn wall_top() -> f64 {
    WALL_Y + (WALL_ROWS - 1) as f64 * WALL_SPACING
}

fn deterministic_flaw(column: usize, row: usize) -> bool {
    // Sparse, repeatable weaker relations make this a heterogeneous stone wall rather
    // than an implausibly perfect crystal. The visible centre seam remains the dominant
    // intentionally weak interface.
    (column * 17 + row * 31 + 7).is_multiple_of(43)
}

pub mod meet;

/// Shared experiment machinery for the MEET-2 seam tests (meet.rs): the resident
/// wall released from its anchors in uniform effective gravity, exported through the
/// rigid chart at both ends of the flight.
#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    pub(crate) fn fixed_step() -> f64 {
        FIXED_STEP
    }

    pub(crate) fn wall_effective_g() -> f64 {
        CHART_GRAVITY_M_S2 * STAGE_WALL_GRAVITY_FACTOR
    }

    pub(crate) fn solver_velocity_damping_per_s() -> f64 {
        SOLVER_VELOCITY_DAMPING_PER_S
    }

    /// Free the wall, optionally pre-damage `pre_damage_bonds` bonds to 0.9 of
    /// critical (all D < 1), fly `steps` substeps, and return (export at launch,
    /// export at landing, COM drop, |lateral COM drift|, start height).
    pub(crate) fn free_wall_com_flight(
        pre_damage_bonds: usize,
        steps: u32,
    ) -> (RigidChartExport, RigidChartExport, f64, f64, f64) {
        let mut sim = Simulation::empty();
        sim.reset();
        for node in &mut sim.nodes {
            node.anchored = false;
        }
        for bond in sim.bonds.iter_mut().take(pre_damage_bonds) {
            let peak = bond.relation.law.opening_at_peak();
            let failure = bond.relation.law.opening_at_failure();
            bond.relation.axial_force(peak + 0.9 * (failure - peak), 0.0);
        }
        let com = |sim: &Simulation| {
            let mut sum = Vec2::ZERO;
            for node in &sim.nodes {
                sum += node.position;
            }
            sum * (1.0 / sim.nodes.len() as f64)
        };
        let start = com(&sim);
        let exported = sim.rigid_export();
        for _ in 0..steps {
            sim.substep(FIXED_STEP);
        }
        let end = com(&sim);
        let returned = sim.rigid_export();
        (
            exported,
            returned,
            start.y - end.y,
            (end.x - start.x).abs(),
            start.y,
        )
    }
}

static SIMULATION: Mutex<Simulation> = Mutex::new(Simulation::empty());

fn simulation() -> MutexGuard<'static, Simulation> {
    SIMULATION
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[no_mangle]
pub extern "C" fn ciris_reset() {
    simulation().reset();
}

#[no_mangle]
pub extern "C" fn ciris_launch(target_y: f64, speed: f64) {
    simulation().launch(target_y, speed);
}

#[no_mangle]
pub extern "C" fn ciris_step(elapsed_seconds: f64) {
    simulation().advance(elapsed_seconds);
}

#[no_mangle]
pub extern "C" fn ciris_node_count() -> u32 {
    WALL_NODE_COUNT as u32
}

#[no_mangle]
pub extern "C" fn ciris_node_x(index: u32) -> f64 {
    simulation()
        .nodes
        .get(index as usize)
        .map_or(0.0, |node| node.position.x)
}

#[no_mangle]
pub extern "C" fn ciris_node_y(index: u32) -> f64 {
    simulation()
        .nodes
        .get(index as usize)
        .map_or(0.0, |node| node.position.y)
}

#[no_mangle]
pub extern "C" fn ciris_node_anchored(index: u32) -> u32 {
    simulation()
        .nodes
        .get(index as usize)
        .map_or(0, |node| node.anchored as u32)
}

#[no_mangle]
pub extern "C" fn ciris_node_terminal_holons(index: u32) -> u32 {
    let base = WALL_HOLONS / WALL_NODE_COUNT as u64;
    let remainder = WALL_HOLONS % WALL_NODE_COUNT as u64;
    (base + u64::from((index as u64) < remainder)) as u32
}

#[no_mangle]
pub extern "C" fn ciris_bond_count() -> u32 {
    let mut sim = simulation();
    sim.ensure_initialized();
    sim.bonds.len() as u32
}

#[no_mangle]
pub extern "C" fn ciris_bond_a(index: u32) -> u32 {
    simulation()
        .bonds
        .get(index as usize)
        .map_or(0, |bond| bond.a as u32)
}

#[no_mangle]
pub extern "C" fn ciris_bond_b(index: u32) -> u32 {
    simulation()
        .bonds
        .get(index as usize)
        .map_or(0, |bond| bond.b as u32)
}

#[no_mangle]
pub extern "C" fn ciris_bond_damage(index: u32) -> f64 {
    simulation()
        .bonds
        .get(index as usize)
        .map_or(0.0, |bond| bond.relation.damage())
}

#[no_mangle]
pub extern "C" fn ciris_bond_is_weak(index: u32) -> u32 {
    simulation()
        .bonds
        .get(index as usize)
        .map_or(0, |bond| bond.weak_interface as u32)
}

#[no_mangle]
pub extern "C" fn ciris_ball_x() -> f64 {
    simulation().ball.position.x
}

#[no_mangle]
pub extern "C" fn ciris_ball_y() -> f64 {
    simulation().ball.position.y
}

#[no_mangle]
pub extern "C" fn ciris_ball_speed() -> f64 {
    simulation().ball.velocity.length()
}

#[no_mangle]
pub extern "C" fn ciris_time() -> f64 {
    simulation().time
}

#[no_mangle]
pub extern "C" fn ciris_cracked_bonds() -> u32 {
    simulation().cracked_bonds() as u32
}

#[no_mangle]
pub extern "C" fn ciris_maximum_damage() -> f64 {
    simulation().maximum_damage()
}

#[no_mangle]
pub extern "C" fn ciris_peak_contact_force() -> f64 {
    simulation().peak_contact_force
}

#[no_mangle]
pub extern "C" fn ciris_impact_count() -> u32 {
    simulation().impacts
}

#[no_mangle]
pub extern "C" fn ciris_wall_holons() -> u32 {
    WALL_HOLONS as u32
}

#[no_mangle]
pub extern "C" fn ciris_ball_holons() -> u32 {
    BALL_HOLONS as u32
}

#[no_mangle]
pub extern "C" fn ciris_wall_rigid_mass() -> f64 {
    let mut sim = simulation();
    sim.ensure_initialized();
    sim.rigid_export().mass_kg
}

#[no_mangle]
pub extern "C" fn ciris_wall_rigid_mean_damage() -> f64 {
    let mut sim = simulation();
    sim.ensure_initialized();
    sim.rigid_export().record.mean_damage
}

#[no_mangle]
pub extern "C" fn ciris_wall_rigid_max_damage() -> f64 {
    let mut sim = simulation();
    sim.ensure_initialized();
    sim.rigid_export().record.max_damage
}

#[no_mangle]
pub extern "C" fn ciris_wall_rigid_broken_bonds() -> u32 {
    let mut sim = simulation();
    sim.ensure_initialized();
    sim.rigid_export().record.broken_count
}

#[no_mangle]
pub extern "C" fn ciris_material_density() -> f64 {
    STONE_BINDING.properties.density_kg_m3
}

#[no_mangle]
pub extern "C" fn ciris_material_young_modulus() -> f64 {
    STONE_BINDING.properties.young_modulus_pa
}

#[no_mangle]
pub extern "C" fn ciris_material_tensile_strength() -> f64 {
    STONE_BINDING.properties.tensile_strength_pa
}

#[no_mangle]
pub extern "C" fn ciris_material_fracture_energy() -> f64 {
    STONE_BINDING.properties.fracture_energy_j_m2
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciris_sim_core::homogenization::{effective_plane_stress_constants, LatticeElasticLaw};

    /// Nodes with no live-bond path to any anchored node: the severed fragments.
    fn detached_node_count(sim: &Simulation) -> usize {
        let mut reached = vec![false; sim.nodes.len()];
        let mut queue: Vec<usize> = (0..sim.nodes.len())
            .filter(|&i| sim.nodes[i].anchored)
            .collect();
        for &i in &queue {
            reached[i] = true;
        }
        while let Some(i) = queue.pop() {
            for &(other, bond) in &sim.bonds_by_node[i] {
                if !reached[other] && !sim.bonds[bond].relation.is_broken() {
                    reached[other] = true;
                    queue.push(other);
                }
            }
        }
        reached.iter().filter(|r| !**r).count()
    }

    /// Deepest interpenetration among pairs the contact solver owns (no live bond).
    fn max_unbonded_overlap(sim: &Simulation) -> f64 {
        let mut deepest = 0.0f64;
        for (i, j) in node_contact_candidates(sim.nodes.len()) {
            if sim.live_bond_between(i, j) {
                continue;
            }
            let distance = (sim.nodes[j].position - sim.nodes[i].position).length();
            deepest = deepest.max(2.0 * NODE_RADIUS - distance);
        }
        deepest
    }

    #[test]
    fn free_fall_acceleration_is_universal_in_the_chart() {
        // Equivalence-principle gate: the chart's gravitational acceleration is
        // identical for bodies of any mass to 1e-12 — no holon carries its own g.
        let masses = [NODE_MASS, BALL_MASS, 17.3];
        for mass in masses {
            let acceleration = chart_gravity_force(mass) * (1.0 / mass);
            assert!(acceleration.x.abs() <= 1.0e-12);
            assert!(
                (acceleration.y + CHART_GRAVITY_M_S2).abs() <= 1.0e-12 * CHART_GRAVITY_M_S2,
                "mass {mass} falls at {} instead of the chart's g",
                -acceleration.y
            );
        }

        // Dynamics half: the ball free-falls at exactly chart g, and a fully
        // detached wall node at exactly STAGE_WALL_GRAVITY_FACTOR times it (through
        // the declared solver velocity damper) — pinning the stage knob as the
        // single named deviation from universality.
        let mut sim = Simulation::empty();
        sim.reset();
        let corner = node_index(0, 0);
        for &(_, bond) in sim.bonds_by_node[corner].clone().iter() {
            let failure = sim.bonds[bond].relation.law.opening_at_failure();
            sim.bonds[bond].relation.axial_force(failure + 1.0, 0.0);
            assert!(sim.bonds[bond].relation.is_broken());
        }
        sim.substep(FIXED_STEP);

        let ball_acceleration = sim.ball.velocity.y / FIXED_STEP;
        assert!(
            (ball_acceleration + CHART_GRAVITY_M_S2).abs() <= 1.0e-12 * CHART_GRAVITY_M_S2
        );
        let damper = 1.0 + SOLVER_VELOCITY_DAMPING_PER_S * FIXED_STEP;
        let node_acceleration = sim.nodes[corner].velocity.y * damper / FIXED_STEP;
        let expected = -CHART_GRAVITY_M_S2 * STAGE_WALL_GRAVITY_FACTOR;
        assert!(
            (node_acceleration - expected).abs() <= 1.0e-12 * expected.abs(),
            "detached node fell at {node_acceleration}, expected {expected}"
        );
    }

    #[test]
    fn never_bonded_faces_share_the_failed_interface_tribology() {
        // D = 1 handoff contract, uniformity clause: never-bonded wall faces must
        // slide exactly like failed ones, so the solver friction constants are pinned
        // to the derived stage law's — and weakened() preserves both, so every bond
        // in the wall (weak seam included) hands off to the same tribology.
        let law = stage_lattice().law;
        assert_eq!(SOLVER_CONTACT_FRICTION_MU, law.friction_coefficient);
        assert_eq!(SOLVER_CONTACT_FRICTION_DAMPING_N_S_M, law.damping_n_s_m);
        let weakened = law.weakened(0.42, 0.30);
        assert_eq!(weakened.friction_coefficient, law.friction_coefficient);
        assert_eq!(weakened.damping_n_s_m, law.damping_n_s_m);
    }

    #[test]
    fn stage_wall_realizes_the_descriptor_poisson_ratio() {
        // The point of the (k_n, k_t) wiring: nu depends only on the k_t/k_n ratio,
        // which the stage similarity map preserves exactly, so the stage wall
        // realizes the descriptor's nu = 0.24 instead of the central-force stencil's
        // forced 1/3 — and the stage law's shape identities survive the map.
        let material = STONE_BINDING.properties;
        let stage = stage_lattice();
        let si = derive_lattice_elastic_law(&material, NODE_MASS, WALL_SPACING).unwrap();
        let stage_elastic = LatticeElasticLaw {
            thickness_m: si.thickness_m,
            normal_stiffness_n_m: stage.law.stiffness_n_m,
            tangential_stiffness_n_m: stage.tangential_stiffness_n_m,
        };
        let (_e, nu) = effective_plane_stress_constants(&stage_elastic);
        assert!(
            (nu - material.poisson_ratio).abs() <= 1.0e-12,
            "stage wall realizes nu = {nu}, descriptor says {}",
            material.poisson_ratio
        );
        // Anchors hold, and the brittleness ratio equals h_max/h from the continuum.
        assert!((stage.law.stiffness_n_m - STAGE_BOND_STIFFNESS_N_M).abs() <= 1.0e-9);
        assert!((stage.law.peak_force_n - STAGE_BOND_PEAK_N).abs() <= 1.0e-9);
        let ratio = stage.law.opening_at_failure() / stage.law.opening_at_peak();
        assert!((ratio - 0.180_921_052_631_578_93 / WALL_SPACING).abs() <= 1.0e-9);
    }

    #[test]
    fn severed_fragments_do_not_interpenetrate() {
        // The D = 1 regime gate: a hard centre throw fully severs the left half of
        // the wall at the weak seam (144 detached nodes, gauged before staking); the
        // fragments must then CONTACT the anchored half instead of passing through
        // it. Tolerance 0.25·NODE_RADIUS was staked after gauging the ruler: the
        // healthy solver settles to ~1e-3 m overlap, the contact-disabled mutant
        // sits at 0.15–0.19 m (near full diameter) indefinitely — dropping the
        // contact law makes this test fail by two orders of magnitude.
        let mut sim = Simulation::empty();
        sim.launch((WALL_Y + wall_top()) * 0.5, 18.0);
        for _ in 0..480 {
            sim.advance(1.0 / 60.0);
        }
        let detached = detached_node_count(&sim);
        assert!(
            detached > 0,
            "the 18 m/s centre throw must sever a chunk from the anchored wall"
        );
        let overlap = max_unbonded_overlap(&sim);
        assert!(
            overlap <= 0.25 * NODE_RADIUS,
            "solver-owned pairs must not interpenetrate: max overlap {overlap} m \
             across {detached} detached nodes"
        );
    }

    #[test]
    fn node_contact_is_internal_to_the_momentum_ledger() {
        // Momentum honesty: node-node contact (and every other free-free internal
        // force) must sum to zero. With two never-bonded free nodes teleported into
        // overlap far from the anchored column, one substep must change the wall's
        // total momentum by exactly gravity times dt (scaled by the declared solver
        // damper) and the ball's by exactly ball gravity — any contact-force sign
        // error shows up as a leak here.
        let mut sim = Simulation::empty();
        sim.reset();
        let i = node_index(0, 0);
        let j = node_index(2, 0);
        sim.nodes[j].position = sim.nodes[i].position + Vec2::new(1.5 * NODE_RADIUS, 0.0);
        sim.nodes[i].velocity = Vec2::new(0.4, 0.0);
        sim.nodes[j].velocity = Vec2::new(-0.4, 0.0);

        let free_count = sim.nodes.iter().filter(|node| !node.anchored).count();
        let momentum = |sim: &Simulation| {
            sim.nodes
                .iter()
                .filter(|node| !node.anchored)
                .fold(Vec2::ZERO, |sum, node| sum + node.velocity * NODE_MASS)
        };
        let nodes_before = momentum(&sim);
        let ball_before = sim.ball.velocity * BALL_MASS;

        sim.substep(FIXED_STEP);

        let gravity_impulse = chart_gravity_force(NODE_MASS)
            * STAGE_WALL_GRAVITY_FACTOR
            * (free_count as f64 * FIXED_STEP);
        let damping = 1.0 / (1.0 + SOLVER_VELOCITY_DAMPING_PER_S * FIXED_STEP);
        let expected_nodes = (nodes_before + gravity_impulse) * damping;
        let expected_ball = ball_before + chart_gravity_force(BALL_MASS) * FIXED_STEP;

        let nodes_after = momentum(&sim);
        let ball_after = sim.ball.velocity * BALL_MASS;
        assert!(
            (nodes_after - expected_nodes).length() <= 1.0e-12,
            "node-node contact leaked momentum: {:?} vs {:?}",
            nodes_after,
            expected_nodes
        );
        assert!(
            (ball_after - expected_ball).length() <= 1.0e-12,
            "ball momentum bookkeeping broke: {:?} vs {:?}",
            ball_after,
            expected_ball
        );
    }

    #[test]
    fn resident_frontier_preserves_exact_wall_gross_count() {
        let total: u64 = (0..WALL_NODE_COUNT)
            .map(|index| {
                let base = WALL_HOLONS / WALL_NODE_COUNT as u64;
                let remainder = WALL_HOLONS % WALL_NODE_COUNT as u64;
                base + u64::from((index as u64) < remainder)
            })
            .sum();
        assert_eq!(total, WALL_HOLONS);
        assert_eq!(WALL_GROSS.constituents, WALL_HOLONS);
        assert_eq!(SCENE_GROSS, BALL_GROSS.combine(WALL_GROSS));
        assert_eq!(SCENE_GROSS.constituents, 1_010_000);
    }

    #[test]
    fn centre_throw_impacts_and_produces_a_crack_without_instability() {
        let mut sim = Simulation::empty();
        sim.launch((WALL_Y + wall_top()) * 0.5, 13.8);
        // Restaked for the DERIVED law (P2): the homogenized wall is more brittle
        // (softening ratio h_max/h = 1.29 vs the hand law's 2.48), so the centre
        // throw now completes the designed seam failure by t ~ 3 s instead of 5 s.
        // The honest claims, gauged before staking: the crack is LOCAL at t = 2 s
        // (measured 3 detached), the total damage stays bounded (measured 90 broken
        // of 797), and by t = 8 s the wall fails along the DESIGNED weak seam,
        // releasing exactly the 9x16 left panel — seam-selective failure, not a
        // shatter.
        for _ in 0..120 {
            sim.advance(1.0 / 60.0);
        }
        assert!(sim.impacts > 0);
        assert!(sim.cracked_bonds() > 0);
        assert!(
            detached_node_count(&sim) <= 8,
            "the crack must still be local at t = 2 s: {} detached",
            detached_node_count(&sim)
        );
        for _ in 0..360 {
            sim.advance(1.0 / 60.0);
        }
        assert!(
            sim.cracked_bonds() < 160,
            "centre throw should fail the seam, not shatter the wall: {} failures",
            sim.cracked_bonds()
        );
        assert_eq!(
            detached_node_count(&sim),
            9 * WALL_ROWS,
            "the failure must be seam-selective: the designed left panel and nothing else"
        );
        assert!(sim.ball.position.x.is_finite() && sim.ball.position.y.is_finite());
        assert!(sim
            .nodes
            .iter()
            .all(|node| node.position.x.is_finite() && node.position.y.is_finite()));
    }

    #[test]
    fn subcritical_wall_damage_reaches_the_rigid_export() {
        // A4: pre-load the wall to sub-critical damage (every D strictly below 1) and
        // assert the rigid export DIFFERS from pristine, through the Record tag alone.
        // Dropping the tag from the export makes this test fail: mass, holon, and the
        // crack observable {r | D = 1} are identical between the two walls.
        let mut sim = Simulation::empty();
        sim.reset();
        let pristine = sim.rigid_export();

        for bond in sim.bonds.iter_mut().take(40) {
            let peak = bond.relation.law.opening_at_peak();
            let failure = bond.relation.law.opening_at_failure();
            bond.relation.axial_force(peak + 0.9 * (failure - peak), 0.0);
        }
        assert!(
            sim.bonds.iter().all(|bond| bond.relation.damage() < 1.0),
            "the pre-load must stay sub-critical"
        );

        let loaded = sim.rigid_export();
        assert_eq!(loaded.subject_holon, pristine.subject_holon);
        assert_eq!(loaded.mass_kg, pristine.mass_kg);
        assert_eq!(loaded.record.broken_count, 0);
        assert_ne!(
            loaded, pristine,
            "a wall pre-loaded to 0.9 of critical must not export like a pristine one"
        );
        assert!(loaded.record.mean_damage > 0.0);
    }

    #[test]
    fn reset_is_deterministic() {
        let run = || {
            let mut sim = Simulation::empty();
            sim.launch(3.0, 12.5);
            for _ in 0..120 {
                sim.advance(1.0 / 60.0);
            }
            (
                sim.ball.position.x.to_bits(),
                sim.ball.position.y.to_bits(),
                sim.cracked_bonds(),
            )
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn complete_player_aim_and_speed_envelope_produces_valid_impacts() {
        let target_heights = [
            WALL_Y,
            WALL_Y + (wall_top() - WALL_Y) * 0.25,
            (WALL_Y + wall_top()) * 0.5,
            WALL_Y + (wall_top() - WALL_Y) * 0.75,
            wall_top(),
        ];
        let speeds = [8.0, 13.8, 18.0];

        for target_y in target_heights {
            for speed in speeds {
                let mut sim = Simulation::empty();
                sim.launch(target_y, speed);
                for _ in 0..270 {
                    sim.advance(1.0 / 60.0);
                }
                assert!(
                    sim.impacts > 0,
                    "throw at y={target_y}, speed={speed} missed the wall"
                );
                assert!(
                    sim.ball.position.x.is_finite()
                        && sim.ball.position.y.is_finite()
                        && sim.ball.velocity.x.is_finite()
                        && sim.ball.velocity.y.is_finite(),
                    "throw at y={target_y}, speed={speed} produced a non-finite ball"
                );
                assert!(
                    sim.nodes.iter().all(|node| node.position.x.is_finite()
                        && node.position.y.is_finite()
                        && node.velocity.x.is_finite()
                        && node.velocity.y.is_finite()),
                    "throw at y={target_y}, speed={speed} produced a non-finite wall"
                );
            }
        }
    }
}
