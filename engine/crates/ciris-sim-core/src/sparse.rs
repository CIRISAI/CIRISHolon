//! Sparse conservative mechanics and a minimal deterministic contact layer.
//!
//! This module is deliberately separate from [`crate::structure::Structure`]. The
//! ontology-specialised K11 path keeps its dense metric, eigensystem and proved symmetry
//! machinery. General sparse scenes should not pay for those quantities merely to step
//! a spring network.
//!
//! The spring kernel stores exactly `E` edges and evaluates forces in `O(E)` time with
//! no allocator. `SparseSystem::from_edges` derives the same weighted-degree mass model
//! used by E2. The integration scheme is velocity Verlet, matching the conservative
//! dense path.
//!
//! Contacts are intentionally smaller in scope than Rapier: frictionless spheres with
//! coefficient of restitution `e`. Resolution is an impulse satisfying linear-momentum
//! conservation plus a mass-weighted positional correction for overlap. Pair iteration
//! order is fixed, so replay is deterministic. Broad phase is still all-pairs; sparse
//! spring scaling is solved here, contact broad-phase scaling is a separate problem.

use crate::dynamics::State;

/// Numerical guard for normalisation.
const EPS: f64 = 1.0e-12;

/// One undirected spring edge, stored once with `i < j` by convention.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    pub i: usize,
    pub j: usize,
    pub stiffness: f64,
    pub rest_length: f64,
}

impl Edge {
    pub const ZERO: Edge = Edge { i: 0, j: 0, stiffness: 0.0, rest_length: 0.0 };

    pub const fn new(i: usize, j: usize, stiffness: f64, rest_length: f64) -> Edge {
        Edge { i, j, stiffness, rest_length }
    }
}

/// Fixed-capacity sparse spring system: `N` point masses and exactly `E` spring edges.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SparseSystem<const N: usize, const E: usize> {
    pub edges: [Edge; E],
    /// Inertial masses. By default these are weighted degrees `Σ_j c_ij`; isolated
    /// nodes fall back to unit mass so acceleration remains defined.
    pub mass: [f64; N],
}

impl<const N: usize, const E: usize> SparseSystem<N, E> {
    /// Construct from edges and derive weighted-degree masses in one `O(E)` pass.
    pub fn from_edges(edges: [Edge; E]) -> Self {
        let mut mass = [0.0; N];
        let mut e = 0;
        while e < E {
            let edge = edges[e];
            assert!(edge.i < N && edge.j < N && edge.i != edge.j);
            assert!(edge.stiffness >= 0.0);
            mass[edge.i] += edge.stiffness;
            mass[edge.j] += edge.stiffness;
            e += 1;
        }
        let mut i = 0;
        while i < N {
            if mass[i] <= 0.0 {
                mass[i] = 1.0;
            }
            i += 1;
        }
        SparseSystem { edges, mass }
    }

    /// Conservative spring force in fixed edge order. Complexity: `O(E)`.
    pub fn forces(&self, state: &State<N>) -> [[f64; 3]; N] {
        let mut f = [[0.0; 3]; N];
        for edge in self.edges.iter() {
            if edge.stiffness == 0.0 {
                continue;
            }
            let d = [
                state.pos[edge.j][0] - state.pos[edge.i][0],
                state.pos[edge.j][1] - state.pos[edge.i][1],
                state.pos[edge.j][2] - state.pos[edge.i][2],
            ];
            let r2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            let scale = if edge.rest_length == 0.0 {
                edge.stiffness
            } else {
                let r = libm::sqrt(r2);
                if r < EPS { 0.0 } else { edge.stiffness * (1.0 - edge.rest_length / r) }
            };
            if scale != 0.0 {
                for a in 0..3 {
                    let q = scale * d[a];
                    f[edge.i][a] += q;
                    f[edge.j][a] -= q;
                }
            }
        }
        f
    }

    /// Exact spring potential corresponding to [`SparseSystem::forces`].
    pub fn potential_energy(&self, state: &State<N>) -> f64 {
        let mut u = 0.0;
        for edge in self.edges.iter() {
            if edge.stiffness == 0.0 {
                continue;
            }
            let dx = state.pos[edge.j][0] - state.pos[edge.i][0];
            let dy = state.pos[edge.j][1] - state.pos[edge.i][1];
            let dz = state.pos[edge.j][2] - state.pos[edge.i][2];
            let r2 = dx * dx + dy * dy + dz * dz;
            let stretch2 = if edge.rest_length == 0.0 {
                r2
            } else {
                let e = libm::sqrt(r2) - edge.rest_length;
                e * e
            };
            u += 0.5 * edge.stiffness * stretch2;
        }
        u
    }

    pub fn kinetic_energy(&self, state: &State<N>) -> f64 {
        let mut t = 0.0;
        for i in 0..N {
            let v = state.vel[i];
            t += 0.5 * self.mass[i] * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
        }
        t
    }

    pub fn total_energy(&self, state: &State<N>) -> f64 {
        self.kinetic_energy(state) + self.potential_energy(state)
    }

    /// One mass-aware velocity-Verlet step. Complexity: `O(E + N)`.
    pub fn step(&self, state: &mut State<N>, dt: f64) {
        let f0 = self.forces(state);
        for i in 0..N {
            let half = 0.5 * dt / self.mass[i];
            for a in 0..3 {
                state.vel[i][a] += half * f0[i][a];
                state.pos[i][a] += dt * state.vel[i][a];
            }
        }

        let f1 = self.forces(state);
        for i in 0..N {
            let half = 0.5 * dt / self.mass[i];
            for a in 0..3 {
                state.vel[i][a] += half * f1[i][a];
            }
        }
    }

    /// Verlet step followed by frictionless sphere-contact resolution.
    pub fn step_with_contacts(
        &self,
        state: &mut State<N>,
        dt: f64,
        contacts: &ContactParams,
    ) -> usize {
        self.step(state, dt);
        resolve_sphere_contacts(state, &self.mass, contacts)
    }
}

/// Frictionless equal-radius sphere contacts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContactParams {
    /// Radius of every point body. Set to zero to disable contacts.
    pub radius: f64,
    /// Normal coefficient of restitution: 1 = elastic, 0 = perfectly inelastic normal
    /// response. Values are clamped to [0, 1].
    pub restitution: f64,
    /// Fraction of geometric overlap removed on each resolver pass, clamped to [0, 1].
    pub correction: f64,
}

impl Default for ContactParams {
    fn default() -> Self {
        Self { radius: 0.05, restitution: 1.0, correction: 1.0 }
    }
}

/// Resolve all overlapping sphere pairs once, in fixed `i < j` order.
///
/// Returns the number of overlapping pairs visited. Linear momentum is conserved by
/// each impulse. For restitution 1 and a perfectly resolved collision normal, the
/// impulse also preserves kinetic energy along that normal.
pub fn resolve_sphere_contacts<const N: usize>(
    state: &mut State<N>,
    mass: &[f64; N],
    params: &ContactParams,
) -> usize {
    if params.radius <= 0.0 {
        return 0;
    }
    let diameter = 2.0 * params.radius;
    let diameter2 = diameter * diameter;
    let restitution = params.restitution.clamp(0.0, 1.0);
    let correction = params.correction.clamp(0.0, 1.0);
    let mut count = 0;

    for i in 0..N {
        for j in (i + 1)..N {
            let d = [
                state.pos[j][0] - state.pos[i][0],
                state.pos[j][1] - state.pos[i][1],
                state.pos[j][2] - state.pos[i][2],
            ];
            let r2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            if r2 >= diameter2 {
                continue;
            }
            count += 1;

            // Coincident centres have no geometric contact normal. There is no
            // symmetry-preserving direction to choose, so leave the pair unresolved
            // rather than inject an arbitrary axis into an otherwise deterministic run.
            if r2 < EPS * EPS {
                continue;
            }

            let r = libm::sqrt(r2);
            let n = [d[0] / r, d[1] / r, d[2] / r];
            let inv_i = 1.0 / mass[i];
            let inv_j = 1.0 / mass[j];
            let inv_sum = inv_i + inv_j;

            let rv = [
                state.vel[j][0] - state.vel[i][0],
                state.vel[j][1] - state.vel[i][1],
                state.vel[j][2] - state.vel[i][2],
            ];
            let vn = rv[0] * n[0] + rv[1] * n[1] + rv[2] * n[2];
            if vn < 0.0 {
                let impulse = -(1.0 + restitution) * vn / inv_sum;
                for a in 0..3 {
                    let q = impulse * n[a];
                    state.vel[i][a] -= q * inv_i;
                    state.vel[j][a] += q * inv_j;
                }
            }

            let penetration = diameter - r;
            if penetration > 0.0 && correction > 0.0 {
                let move_i = correction * penetration * inv_i / inv_sum;
                let move_j = correction * penetration * inv_j / inv_sum;
                for a in 0..3 {
                    state.pos[i][a] -= move_i * n[a];
                    state.pos[j][a] += move_j * n[a];
                }
            }
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_harmonic_force_is_edge_local() {
        let sys = SparseSystem::<3, 1>::from_edges([Edge::new(0, 1, 2.0, 0.0)]);
        let s = State::at_rest([[1.0, 0.0, 0.0], [3.0, 0.0, 0.0], [99.0, 0.0, 0.0]]);
        let f = sys.forces(&s);
        assert_eq!(f[0], [4.0, 0.0, 0.0]);
        assert_eq!(f[1], [-4.0, 0.0, 0.0]);
        assert_eq!(f[2], [0.0, 0.0, 0.0]);
    }

    #[test]
    fn weighted_degree_mass_is_derived_from_edges() {
        let sys = SparseSystem::<3, 2>::from_edges([
            Edge::new(0, 1, 2.0, 0.0),
            Edge::new(1, 2, 3.0, 0.0),
        ]);
        assert_eq!(sys.mass, [2.0, 5.0, 3.0]);
    }

    #[test]
    fn elastic_equal_mass_head_on_collision_swaps_velocities() {
        let sys = SparseSystem::<2, 0>::from_edges([]);
        let mut s = State {
            pos: [[-0.09, 0.0, 0.0], [0.09, 0.0, 0.0]],
            vel: [[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]],
        };
        let p = ContactParams { radius: 0.1, restitution: 1.0, correction: 1.0 };
        let p0 = sys.mass[0] * s.vel[0][0] + sys.mass[1] * s.vel[1][0];
        let e0 = sys.kinetic_energy(&s);
        assert_eq!(resolve_sphere_contacts(&mut s, &sys.mass, &p), 1);
        assert!((s.vel[0][0] + 1.0).abs() < 1.0e-12);
        assert!((s.vel[1][0] - 1.0).abs() < 1.0e-12);
        let p1 = sys.mass[0] * s.vel[0][0] + sys.mass[1] * s.vel[1][0];
        let e1 = sys.kinetic_energy(&s);
        assert!((p1 - p0).abs() < 1.0e-12);
        assert!((e1 - e0).abs() < 1.0e-12);
    }

    #[test]
    fn verlet_energy_error_is_bounded_for_two_body_spring() {
        let sys = SparseSystem::<2, 1>::from_edges([Edge::new(0, 1, 1.0, 0.0)]);
        let mut s = State::at_rest([[0.5, 0.0, 0.0], [-0.5, 0.0, 0.0]]);
        let e0 = sys.total_energy(&s);
        let mut worst = 0.0_f64;
        for _ in 0..10_000 {
            sys.step(&mut s, 0.001);
            let drift = (sys.total_energy(&s) - e0).abs();
            if drift > worst { worst = drift; }
        }
        assert!(worst < 1.0e-5, "energy band too wide: {worst}");
    }
}
