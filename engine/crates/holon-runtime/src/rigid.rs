//! ORIENTED RIGID WATER: the first executable holon (GANTT2, the fourth review, WP2).
//!
//! One water molecule as ONE body: a centre of mass, an orientation (a unit quaternion,
//! body frame to lab), a linear momentum and a body-frame angular momentum. The three
//! interaction sites are RECONSTRUCTED from the body's declared geometry each time the
//! intermolecular law is asked, so the law is the fine model's own, unchanged; what is
//! removed is the O–H and H–O–H motion and, with it, every input to the fine clock.
//!
//! **What this buys and what it does not.** Per pass, nothing: the same three sites meet the
//! same law. The whole gain is the step, read by [`crate::clock`] from the retained modes
//! under the same accuracy hold as the fine clock. **Do not assume that rigidification proves
//! statistical equivalence to flexible water**; that is REPLACE-0's measurement, not this
//! module's claim, and the module's [`crate::Validity`] starts at `InvariantsOnly`.
//!
//! The lift is DECLARED: the body frame is the H–O–H bisector (from oxygen toward the
//! hydrogens' midpoint), the plane normal, and their cross product — the monomer's own C2v
//! principal frame — and a fine state is fitted to it by that rule, not by least squares.
//! Deformation and internal kinetic energy the rule cannot represent are RECORDED in
//! [`Discarded`], never silently dropped.
//!
//! Integration is velocity-Verlet on the centre and the symmetric free-rotor splitting of
//! Dullweber, Leimkuhler and McLachlan (J. Chem. Phys. 107, 5840 (1997)) on the orientation:
//! rotations about the body's principal axes in the order 1-2-3-2-1, each exact, so the lab
//! angular momentum of a free body is conserved to rounding and the map is symplectic and
//! time-reversible. Established prior art; the contribution here is the measured replacement
//! error and certificate, not the integrator.

use crate::clock::{self, ClockReading, StiffnessEnvelope};
use crate::{Discarded, Operator, Refusal, Totals};

/// Three sites in the DECLARED order oxygen, hydrogen, hydrogen: positions, velocities,
/// masses. Atomic units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sites {
    pub pos: [[f64; 3]; 3],
    pub vel: [[f64; 3]; 3],
    pub mass: [f64; 3],
}

/// The fine state this operator replaces: the sites AND the reference body they are read
/// against. The reference travels with the state so that a re-projection after a refresh is
/// fitted to the SAME rigid geometry and not to whatever shape the fine sub-steps left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fine {
    pub sites: Sites,
    pub reference: Body,
}

/// The exchange across the body's boundary: a lab-frame force on each site.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SiteForces {
    pub f: [[f64; 3]; 3],
}

/// The body: its geometry in its own principal frame, built ONCE from a reference monomer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub mass: f64,
    pub site_mass: [f64; 3],
    /// Site positions in the principal frame, origin at the centre of mass.
    pub sites_body: [[f64; 3]; 3],
    /// Principal moments of inertia.
    pub inertia: [f64; 3],
    /// The largest site distance from the centre: the lever the clock uses.
    pub lever_max: f64,
}

/// Tolerance on the off-diagonal inertia, relative to the trace, for the declared frame to
/// count as principal. The monomer's C2v symmetry makes it exactly principal; a reference
/// handed in with unequal O–H bonds is not, and is refused rather than silently diagonalised.
pub const PRINCIPAL_TOLERANCE: f64 = 1e-9;

impl Body {
    /// Build the body from a reference monomer's sites and masses.
    pub fn from_monomer(pos: [[f64; 3]; 3], mass: [f64; 3]) -> Result<Body, Refusal> {
        for (i, &m) in mass.iter().enumerate() {
            if !(m.is_finite() && m > 0.0) {
                return Err(Refusal::BadMass(i));
            }
        }
        let total = mass[0] + mass[1] + mass[2];
        let com = centre_of_mass(&pos, &mass, total);
        let r = frame(&pos)?;
        let mut sites_body = [[0.0; 3]; 3];
        for i in 0..3 {
            sites_body[i] = mat_t_vec(&r, &sub(pos[i], com));
        }
        // the inertia tensor in the declared frame; principal iff off-diagonal vanishes
        let mut t = [[0.0f64; 3]; 3];
        for i in 0..3 {
            let s = sites_body[i];
            let m = mass[i];
            let rr = dot(s, s);
            for a in 0..3 {
                for b in 0..3 {
                    t[a][b] += m * (if a == b { rr } else { 0.0 } - s[a] * s[b]);
                }
            }
        }
        let trace = t[0][0] + t[1][1] + t[2][2];
        let off = t[0][1].abs().max(t[0][2].abs()).max(t[1][2].abs());
        if off > PRINCIPAL_TOLERANCE * trace {
            return Err(Refusal::NotPrincipal { off_diagonal: off, tolerance: PRINCIPAL_TOLERANCE * trace });
        }
        let lever_max = sites_body.iter().map(|s| dot(*s, *s).sqrt()).fold(0.0, f64::max);
        Ok(Body { mass: total, site_mass: mass, sites_body, inertia: [t[0][0], t[1][1], t[2][2]], lever_max })
    }
}

/// The operator's state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigidWater {
    pub body: Body,
    /// Centre of mass, lab frame.
    pub com: [f64; 3],
    /// Orientation, body to lab, as a unit quaternion `(w, x, y, z)`.
    pub q: [f64; 4],
    /// Linear momentum, lab frame.
    pub p: [f64; 3],
    /// Angular momentum in the BODY frame (the free-rotor splitting works there).
    pub l_body: [f64; 3],
}

impl RigidWater {
    /// The rotation matrix body -> lab.
    pub fn rotation(&self) -> [[f64; 3]; 3] {
        quat_to_mat(&self.q)
    }

    /// Angular velocity, lab frame: `R I^-1 l_body`.
    pub fn omega_lab(&self) -> [f64; 3] {
        let r = self.rotation();
        let w_body = [
            self.l_body[0] / self.body.inertia[0],
            self.l_body[1] / self.body.inertia[1],
            self.l_body[2] / self.body.inertia[2],
        ];
        mat_vec(&r, &w_body)
    }

    /// The site positions the law is asked at: `com + R s_i`.
    pub fn site_positions(&self) -> [[f64; 3]; 3] {
        let r = self.rotation();
        let mut out = [[0.0; 3]; 3];
        for i in 0..3 {
            out[i] = add(self.com, mat_vec(&r, &self.body.sites_body[i]));
        }
        out
    }

    /// ACCUMULATE the exchange: the net force and the lab-frame torque about the centre.
    pub fn accumulate(&self, ex: &SiteForces) -> ([f64; 3], [f64; 3]) {
        let pos = self.site_positions();
        let mut f = [0.0; 3];
        let mut tau = [0.0; 3];
        for i in 0..3 {
            f = add(f, ex.f[i]);
            tau = add(tau, cross(sub(pos[i], self.com), ex.f[i]));
        }
        (f, tau)
    }

    /// Half-kick: momenta under the exchange for `dt_half`.
    pub fn kick(&mut self, dt_half: f64, ex: &SiteForces) {
        let (f, tau) = self.accumulate(ex);
        let r = self.rotation();
        let tau_body = mat_t_vec(&r, &tau);
        for k in 0..3 {
            self.p[k] += f[k] * dt_half;
            self.l_body[k] += tau_body[k] * dt_half;
        }
    }

    /// Free flight for `dt`: the centre drifts; the orientation takes the 1-2-3-2-1 splitting.
    pub fn free_flight(&mut self, dt: f64) {
        for k in 0..3 {
            self.com[k] += self.p[k] / self.body.mass * dt;
        }
        self.rotate_axis(0, 0.5 * dt);
        self.rotate_axis(1, 0.5 * dt);
        self.rotate_axis(2, dt);
        self.rotate_axis(1, 0.5 * dt);
        self.rotate_axis(0, 0.5 * dt);
        normalize(&mut self.q);
    }

    /// EXACT rotation about body axis `k` for time `dt` at the current angular momentum: the
    /// body turns by `phi = dt l_k / I_k`, so the body-frame angular momentum turns by `-phi`
    /// and the lab angular momentum `R l` does not move at all.
    fn rotate_axis(&mut self, k: usize, dt: f64) {
        let phi = dt * self.l_body[k] / self.body.inertia[k];
        let (s, c) = phi.sin_cos();
        let a = (k + 1) % 3;
        let b = (k + 2) % 3;
        let la = self.l_body[a];
        let lb = self.l_body[b];
        self.l_body[a] = la * c + lb * s;
        self.l_body[b] = -la * s + lb * c;
        let (hs, hc) = (0.5 * phi).sin_cos();
        let mut axis = [0.0; 4];
        axis[0] = hc;
        axis[1 + k] = hs;
        self.q = quat_mul(&self.q, &axis);
    }

    /// Rotational kinetic energy alone.
    pub fn rotational_kinetic(&self) -> f64 {
        (0..3).map(|k| self.l_body[k] * self.l_body[k] / (2.0 * self.body.inertia[k])).sum()
    }

    /// Translational kinetic energy alone.
    pub fn translational_kinetic(&self) -> f64 {
        dot(self.p, self.p) / (2.0 * self.body.mass)
    }
}

impl Operator for RigidWater {
    type Fine = Fine;
    type Exchange = SiteForces;

    fn project(fine: &Fine) -> Result<(RigidWater, Discarded), Refusal> {
        let body = fine.reference;
        let s = &fine.sites;
        for i in 0..3 {
            if !(s.mass[i].is_finite() && s.mass[i] > 0.0) || (s.mass[i] - body.site_mass[i]).abs() > 1e-9 * body.site_mass[i] {
                return Err(Refusal::BadMass(i));
            }
        }
        let com = centre_of_mass(&s.pos, &s.mass, body.mass);
        let mut p = [0.0; 3];
        for i in 0..3 {
            p = add(p, scale(s.vel[i], s.mass[i]));
        }
        let vcom = scale(p, 1.0 / body.mass);
        let r = frame(&s.pos)?;
        let q = mat_to_quat(&r);
        let mut l_lab = [0.0; 3];
        for i in 0..3 {
            l_lab = add(l_lab, scale(cross(sub(s.pos[i], com), sub(s.vel[i], vcom)), s.mass[i]));
        }
        let l_body = mat_t_vec(&r, &l_lab);
        let me = RigidWater { body, com, q, p, l_body };
        // what the lift cannot represent, RECORDED
        let back = me.reconstruct();
        let mut d2 = 0.0;
        let mut ke_int = 0.0;
        for i in 0..3 {
            let dp = sub(s.pos[i], back.sites.pos[i]);
            d2 += dot(dp, dp);
            let dv = sub(s.vel[i], back.sites.vel[i]);
            ke_int += 0.5 * s.mass[i] * dot(dv, dv);
        }
        Ok((me, Discarded { deformation_rms: (d2 / 3.0).sqrt(), internal_kinetic: ke_int }))
    }

    fn reconstruct(&self) -> Fine {
        let pos = self.site_positions();
        let vcom = scale(self.p, 1.0 / self.body.mass);
        let w = self.omega_lab();
        let mut vel = [[0.0; 3]; 3];
        for i in 0..3 {
            vel[i] = add(vcom, cross(w, sub(pos[i], self.com)));
        }
        Fine { sites: Sites { pos, vel, mass: self.body.site_mass }, reference: self.body }
    }

    fn advance(&mut self, dt: f64, now: &SiteForces, eval: &mut dyn FnMut(&Self) -> SiteForces) -> SiteForces {
        self.kick(0.5 * dt, now);
        self.free_flight(dt);
        let next = eval(self);
        self.kick(0.5 * dt, &next);
        next
    }

    fn clock(&self, envelope: &StiffnessEnvelope) -> ClockReading {
        clock::derive(self.body.mass, self.body.inertia, self.body.lever_max, envelope)
    }

    fn dof(&self) -> usize {
        6
    }

    fn kinetic(&self) -> f64 {
        self.translational_kinetic() + self.rotational_kinetic()
    }

    fn totals(&self) -> Totals {
        let r = self.rotation();
        Totals {
            mass: self.body.mass,
            momentum: self.p,
            angular_momentum: add(cross(self.com, self.p), mat_vec(&r, &self.l_body)),
        }
    }
}

// ------------------------------------------------------------------ the declared frame

/// THE DECLARED FRAME: e1 the bisector from oxygen toward the hydrogens' midpoint, e3 the
/// plane normal `(H1 - O) x (H2 - O)`, e2 = e3 x e1. Returned as the matrix whose COLUMNS
/// are e1, e2, e3, i.e. the rotation body -> lab.
pub fn frame(pos: &[[f64; 3]; 3]) -> Result<[[f64; 3]; 3], Refusal> {
    let o = pos[0];
    let mid = scale(add(pos[1], pos[2]), 0.5);
    let b = sub(mid, o);
    let nb = dot(b, b).sqrt();
    if !(nb > 1e-12) {
        return Err(Refusal::DegenerateFrame("the hydrogens' midpoint coincides with the oxygen".into()));
    }
    let e1 = scale(b, 1.0 / nb);
    let n = cross(sub(pos[1], o), sub(pos[2], o));
    let nn = dot(n, n).sqrt();
    if !(nn > 1e-12) {
        return Err(Refusal::DegenerateFrame("the three sites are collinear".into()));
    }
    let e3 = scale(n, 1.0 / nn);
    let e2 = cross(e3, e1);
    Ok([[e1[0], e2[0], e3[0]], [e1[1], e2[1], e3[1]], [e1[2], e2[2], e3[2]]])
}

fn centre_of_mass(pos: &[[f64; 3]; 3], mass: &[f64; 3], total: f64) -> [f64; 3] {
    let mut c = [0.0; 3];
    for i in 0..3 {
        c = add(c, scale(pos[i], mass[i]));
    }
    scale(c, 1.0 / total)
}

// ------------------------------------------------------------------ small algebra

pub fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn scale(a: [f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}
pub fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
pub fn mat_vec(r: &[[f64; 3]; 3], v: &[f64; 3]) -> [f64; 3] {
    [
        r[0][0] * v[0] + r[0][1] * v[1] + r[0][2] * v[2],
        r[1][0] * v[0] + r[1][1] * v[1] + r[1][2] * v[2],
        r[2][0] * v[0] + r[2][1] * v[1] + r[2][2] * v[2],
    ]
}
pub fn mat_t_vec(r: &[[f64; 3]; 3], v: &[f64; 3]) -> [f64; 3] {
    [
        r[0][0] * v[0] + r[1][0] * v[1] + r[2][0] * v[2],
        r[0][1] * v[0] + r[1][1] * v[1] + r[2][1] * v[2],
        r[0][2] * v[0] + r[1][2] * v[1] + r[2][2] * v[2],
    ]
}

/// `(w, x, y, z)` to the rotation matrix, the standard active right-handed convention.
pub fn quat_to_mat(q: &[f64; 4]) -> [[f64; 3]; 3] {
    let (w, x, y, z) = (q[0], q[1], q[2], q[3]);
    [
        [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - w * z), 2.0 * (x * z + w * y)],
        [2.0 * (x * y + w * z), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - w * x)],
        [2.0 * (x * z - w * y), 2.0 * (y * z + w * x), 1.0 - 2.0 * (x * x + y * y)],
    ]
}

/// Rotation matrix to `(w, x, y, z)`, Shepperd's branch on the largest diagonal term so no
/// division is by a small number.
pub fn mat_to_quat(r: &[[f64; 3]; 3]) -> [f64; 4] {
    let tr = r[0][0] + r[1][1] + r[2][2];
    let mut q = if tr > 0.0 {
        let s = (tr + 1.0).sqrt() * 2.0;
        [0.25 * s, (r[2][1] - r[1][2]) / s, (r[0][2] - r[2][0]) / s, (r[1][0] - r[0][1]) / s]
    } else if r[0][0] > r[1][1] && r[0][0] > r[2][2] {
        let s = (1.0 + r[0][0] - r[1][1] - r[2][2]).sqrt() * 2.0;
        [(r[2][1] - r[1][2]) / s, 0.25 * s, (r[0][1] + r[1][0]) / s, (r[0][2] + r[2][0]) / s]
    } else if r[1][1] > r[2][2] {
        let s = (1.0 + r[1][1] - r[0][0] - r[2][2]).sqrt() * 2.0;
        [(r[0][2] - r[2][0]) / s, (r[0][1] + r[1][0]) / s, 0.25 * s, (r[1][2] + r[2][1]) / s]
    } else {
        let s = (1.0 + r[2][2] - r[0][0] - r[1][1]).sqrt() * 2.0;
        [(r[1][0] - r[0][1]) / s, (r[0][2] + r[2][0]) / s, (r[1][2] + r[2][1]) / s, 0.25 * s]
    };
    normalize(&mut q);
    q
}

/// Hamilton product: `R(a b) = R(a) R(b)`, so a right-multiplication is a body-frame rotation.
pub fn quat_mul(a: &[f64; 4], b: &[f64; 4]) -> [f64; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

pub fn normalize(q: &mut [f64; 4]) {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if n > 0.0 {
        for k in 0..4 {
            q[k] /= n;
        }
    }
}

// ------------------------------------------------------------------ the tests

#[cfg(test)]
mod tests {
    use super::*;

    /// The engine's pinned monomer (`holon-render/src/field.rs`): O-H 1.94357384 bohr,
    /// H-O-H 1.6887434 rad; masses in electron masses, hydrogen the engine's `M_H` and oxygen
    /// 15.9949146196 u.
    const R_OH: f64 = 1.9435738400;
    const THETA: f64 = 1.6887434037;
    const M_H: f64 = 1837.152;
    const M_O: f64 = 15.9949146196 * 1822.888486209;
    const K_B: f64 = 3.166811563e-6;

    fn monomer() -> ([[f64; 3]; 3], [f64; 3]) {
        let (s, c) = (0.5 * THETA).sin_cos();
        ([[0.0, 0.0, 0.0], [R_OH * c, R_OH * s, 0.0], [R_OH * c, -R_OH * s, 0.0]], [M_O, M_H, M_H])
    }

    fn body() -> Body {
        let (p, m) = monomer();
        Body::from_monomer(p, m).expect("the pinned monomer builds a body")
    }

    /// A deterministic generator (an LCG and Box-Muller), so every test is reproducible.
    struct Lcg(u64);
    impl Lcg {
        fn uniform(&mut self) -> f64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((self.0 >> 11) as f64) / ((1u64 << 53) as f64)
        }
        fn gauss(&mut self) -> f64 {
            let u1 = self.uniform().max(1e-300);
            let u2 = self.uniform();
            (-2.0 * u1.ln()).sqrt() * (core::f64::consts::TAU * u2).cos()
        }
        fn unit_quat(&mut self) -> [f64; 4] {
            let mut q = [self.gauss(), self.gauss(), self.gauss(), self.gauss()];
            normalize(&mut q);
            q
        }
    }

    fn a_body_in_motion(seed: u64) -> RigidWater {
        let mut g = Lcg(seed);
        RigidWater {
            body: body(),
            com: [10.0 * g.uniform(), 10.0 * g.uniform(), 10.0 * g.uniform()],
            q: g.unit_quat(),
            p: [g.gauss(), g.gauss(), g.gauss()],
            l_body: [3.0 * g.gauss(), 3.0 * g.gauss(), 3.0 * g.gauss()],
        }
    }

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol * (1.0 + a.abs().max(b.abs()))
    }

    #[test]
    fn the_pinned_monomer_is_principal_in_the_declared_frame_and_planar() {
        let b = body();
        assert!(b.inertia.iter().all(|&i| i > 0.0));
        // the perpendicular-axis theorem for a planar body: I_normal = I_bisector + I_in-plane
        assert!(close(b.inertia[2], b.inertia[0] + b.inertia[1], 1e-12), "{:?}", b.inertia);
        assert!(close(b.mass, M_O + 2.0 * M_H, 1e-15));
        // the centre is on the bisector, closer to the oxygen
        assert!(b.sites_body[0][0] < 0.0 && b.sites_body[1][0] > 0.0 && b.sites_body[2][0] > 0.0);
        assert!(close(b.sites_body[1][1], -b.sites_body[2][1], 1e-12));
        assert!(b.lever_max > R_OH * 0.5 && b.lever_max < R_OH);
    }

    #[test]
    fn an_asymmetric_reference_is_refused_as_not_principal() {
        let (mut p, m) = monomer();
        p[1][0] *= 1.02;
        match Body::from_monomer(p, m) {
            Err(Refusal::NotPrincipal { .. }) => {}
            other => panic!("expected NotPrincipal, got {other:?}"),
        }
    }

    #[test]
    fn degenerate_frames_are_refused_by_name() {
        let (_, m) = monomer();
        let collinear = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        assert!(matches!(Body::from_monomer(collinear, m), Err(Refusal::DegenerateFrame(_))));
        assert!(matches!(Body::from_monomer([[0.0; 3]; 3], m), Err(Refusal::DegenerateFrame(_))));
        assert!(matches!(Body::from_monomer(monomer().0, [M_O, 0.0, M_H]), Err(Refusal::BadMass(1))));
    }

    #[test]
    fn project_and_reconstruct_are_inverses_on_a_rigid_state() {
        for seed in 1..=8u64 {
            let a = a_body_in_motion(seed);
            let fine = a.reconstruct();
            let (b, d) = RigidWater::project(&fine).expect("a rigid state projects");
            assert!(d.deformation_rms < 1e-12, "deformation {}", d.deformation_rms);
            assert!(d.internal_kinetic < 1e-12, "internal KE {}", d.internal_kinetic);
            for k in 0..3 {
                assert!(close(a.com[k], b.com[k], 1e-12));
                assert!(close(a.p[k], b.p[k], 1e-12));
                assert!(close(a.l_body[k], b.l_body[k], 1e-10), "l_body {:?} vs {:?}", a.l_body, b.l_body);
            }
            // the quaternion is recovered up to its sign
            let s = if a.q[0] * b.q[0] + a.q[1] * b.q[1] + a.q[2] * b.q[2] + a.q[3] * b.q[3] < 0.0 { -1.0 } else { 1.0 };
            for k in 0..4 {
                assert!(close(a.q[k], s * b.q[k], 1e-12), "q {:?} vs {:?}", a.q, b.q);
            }
            let back = b.reconstruct();
            for i in 0..3 {
                for k in 0..3 {
                    assert!(close(fine.sites.pos[i][k], back.sites.pos[i][k], 1e-12));
                    assert!(close(fine.sites.vel[i][k], back.sites.vel[i][k], 1e-10));
                }
            }
        }
    }

    #[test]
    fn a_deformed_state_projects_with_its_deformation_recorded_and_the_lift_restores_the_reference_geometry() {
        let a = a_body_in_motion(3);
        let mut fine = a.reconstruct();
        // stretch the first O-H by one per cent along the bond
        let bond = sub(fine.sites.pos[1], fine.sites.pos[0]);
        fine.sites.pos[1] = add(fine.sites.pos[1], scale(bond, 0.01));
        let (b, d) = RigidWater::project(&fine).expect("projects");
        assert!(d.deformation_rms > 0.005 && d.deformation_rms < 0.02, "deformation {}", d.deformation_rms);
        let back = b.reconstruct();
        let r1 = dot(sub(back.sites.pos[1], back.sites.pos[0]), sub(back.sites.pos[1], back.sites.pos[0])).sqrt();
        let r2 = dot(sub(back.sites.pos[2], back.sites.pos[0]), sub(back.sites.pos[2], back.sites.pos[0])).sqrt();
        assert!(close(r1, R_OH, 1e-12) && close(r2, R_OH, 1e-12), "the lift is the reference geometry: {r1} {r2}");
        // mass and momentum are the totals the projection keeps exactly
        let t = b.totals();
        let mut p = [0.0; 3];
        for i in 0..3 {
            p = add(p, scale(fine.sites.vel[i], fine.sites.mass[i]));
        }
        for k in 0..3 {
            assert!(close(t.momentum[k], p[k], 1e-12));
        }
    }

    #[test]
    fn internal_kinetic_energy_is_what_the_rigid_field_cannot_carry() {
        let a = a_body_in_motion(5);
        let mut fine = a.reconstruct();
        // a radial velocity on one hydrogen: partly rigid (it moves p and L), partly not
        let bond = sub(fine.sites.pos[1], fine.sites.pos[0]);
        let u = scale(bond, 1.0 / dot(bond, bond).sqrt());
        let v = 2.0e-3;
        fine.sites.vel[1] = add(fine.sites.vel[1], scale(u, v));
        let (_, d) = RigidWater::project(&fine).expect("projects");
        let added = 0.5 * M_H * v * v;
        assert!(d.internal_kinetic > 0.1 * added && d.internal_kinetic < added, "internal {} of added {added}", d.internal_kinetic);
    }

    /// A tethered potential on the sites: `U = sum_i k/2 |r_i - a_i|^2`. Its force on the body
    /// and its torque about the centre are what `accumulate` must return.
    struct Tether {
        anchors: [[f64; 3]; 3],
        k: f64,
    }
    impl Tether {
        fn energy(&self, pos: &[[f64; 3]; 3]) -> f64 {
            (0..3).map(|i| 0.5 * self.k * dot(sub(pos[i], self.anchors[i]), sub(pos[i], self.anchors[i]))).sum()
        }
        fn forces(&self, pos: &[[f64; 3]; 3]) -> SiteForces {
            let mut f = [[0.0; 3]; 3];
            for i in 0..3 {
                f[i] = scale(sub(pos[i], self.anchors[i]), -self.k);
            }
            SiteForces { f }
        }
    }

    fn tether_for(seed: u64, k: f64) -> (RigidWater, Tether) {
        let a = a_body_in_motion(seed);
        let mut g = Lcg(seed + 100);
        let mut anchors = a.site_positions();
        for i in 0..3 {
            anchors[i] = add(anchors[i], [0.3 * g.gauss(), 0.3 * g.gauss(), 0.3 * g.gauss()]);
        }
        (a, Tether { anchors, k })
    }

    #[test]
    fn the_accumulated_torque_is_the_finite_difference_of_the_site_potential_under_rotation() {
        let (a, t) = tether_for(7, 0.05);
        let (_, tau) = a.accumulate(&t.forces(&a.site_positions()));
        for axis in 0..3 {
            let delta = 1e-5;
            let rotated = |d: f64| {
                let (hs, hc) = (0.5 * d).sin_cos();
                let mut qn = [hc, 0.0, 0.0, 0.0];
                qn[1 + axis] = hs;
                // a LAB-frame rotation about the centre: R' = Rot_n(d) R
                let mut b = a;
                b.q = quat_mul(&qn, &a.q);
                t.energy(&b.site_positions())
            };
            let du = (rotated(delta) - rotated(-delta)) / (2.0 * delta);
            assert!(close(-tau[axis], du, 1e-6), "axis {axis}: -tau {} vs dU/dphi {du}", -tau[axis]);
        }
        // and the net force is the finite difference under translation
        let (f, _) = a.accumulate(&t.forces(&a.site_positions()));
        for axis in 0..3 {
            let delta = 1e-5;
            let moved = |d: f64| {
                let mut b = a;
                b.com[axis] += d;
                t.energy(&b.site_positions())
            };
            let du = (moved(delta) - moved(-delta)) / (2.0 * delta);
            assert!(close(-f[axis], du, 1e-6));
        }
    }

    #[test]
    fn the_free_rotor_conserves_lab_angular_momentum_to_rounding_and_energy_to_second_order() {
        let a0 = a_body_in_motion(11);
        let omega = a0.omega_lab();
        let w = dot(omega, omega).sqrt();
        let run = |dt: f64, steps: usize| -> (f64, f64) {
            let mut a = a0;
            let l0 = a.totals().angular_momentum;
            let e0 = a.rotational_kinetic();
            let mut worst_l = 0.0f64;
            let mut worst_e = 0.0f64;
            for _ in 0..steps {
                a.free_flight(dt);
                let l = a.totals().angular_momentum;
                let dl = sub(l, l0);
                worst_l = worst_l.max(dot(dl, dl).sqrt() / dot(l0, l0).sqrt());
                worst_e = worst_e.max((a.rotational_kinetic() - e0).abs() / e0);
            }
            (worst_l, worst_e)
        };
        let dt = 0.05 / w;
        let (l1, e1) = run(dt, 4000);
        let (l2, e2) = run(0.5 * dt, 8000);
        assert!(l1 < 1e-11 && l2 < 1e-11, "lab L drift {l1} {l2}");
        assert!(e1 < 1e-2, "energy excursion {e1}");
        let ratio = e1 / e2;
        assert!(ratio > 3.0 && ratio < 5.5, "second order: halving dt should quarter the excursion, got {ratio}");
    }

    #[test]
    fn a_tethered_body_conserves_total_energy_to_second_order_in_the_step() {
        let run = |dt: f64, steps: usize| -> f64 {
            let (mut a, t) = tether_for(13, 0.02);
            let mut ex = t.forces(&a.site_positions());
            let e0 = a.kinetic() + t.energy(&a.site_positions());
            let mut worst = 0.0f64;
            for _ in 0..steps {
                ex = a.advance(dt, &ex, &mut |b: &RigidWater| t.forces(&b.site_positions()));
                let e = a.kinetic() + t.energy(&a.site_positions());
                worst = worst.max((e - e0).abs() / e0);
            }
            worst
        };
        let env = StiffnessEnvelope { k_max: 0.02 };
        let dt = a_body_in_motion(13).clock(&env).dt;
        let e1 = run(dt, 2000);
        let e2 = run(0.5 * dt, 4000);
        assert!(e1 < 1e-2, "energy excursion at the clock's step {e1}");
        let ratio = e1 / e2;
        assert!(ratio > 3.0 && ratio < 5.5, "second order: {ratio}");
    }

    #[test]
    fn two_bodies_under_a_pair_force_conserve_momentum_exactly_and_angular_momentum_to_second_order() {
        // a spring between hydrogen 1 of body A and oxygen of body B: equal and opposite site
        // forces, so the total momentum is exact under the kicks and the total angular momentum
        // is conserved by the central force up to the splitting's own order
        let mut a = a_body_in_motion(21);
        let mut b = a_body_in_motion(22);
        b.com = add(a.com, [6.0, 0.5, -0.3]);
        let k = 0.01;
        let pair = |a: &RigidWater, b: &RigidWater| -> (SiteForces, SiteForces) {
            let pa = a.site_positions();
            let pb = b.site_positions();
            let d = sub(pa[1], pb[0]);
            let f = scale(d, -k);
            let mut fa = SiteForces::default();
            let mut fb = SiteForces::default();
            fa.f[1] = f;
            fb.f[0] = scale(f, -1.0);
            (fa, fb)
        };
        let total = |a: &RigidWater, b: &RigidWater| {
            let ta = a.totals();
            let tb = b.totals();
            (add(ta.momentum, tb.momentum), add(ta.angular_momentum, tb.angular_momentum))
        };
        let (p0, l0) = total(&a, &b);
        let dt = a.clock(&StiffnessEnvelope { k_max: k }).dt;
        let (mut fa, mut fb) = pair(&a, &b);
        let mut worst_l = 0.0f64;
        for _ in 0..2000 {
            // both bodies half-kick, fly, then the pair force is re-evaluated ONCE for both
            a.kick(0.5 * dt, &fa);
            b.kick(0.5 * dt, &fb);
            a.free_flight(dt);
            b.free_flight(dt);
            let (na, nb) = pair(&a, &b);
            fa = na;
            fb = nb;
            a.kick(0.5 * dt, &fa);
            b.kick(0.5 * dt, &fb);
            let (p, l) = total(&a, &b);
            for kk in 0..3 {
                assert!(close(p[kk], p0[kk], 1e-11), "momentum {:?} vs {:?}", p, p0);
            }
            let dl = sub(l, l0);
            worst_l = worst_l.max(dot(dl, dl).sqrt() / dot(l0, l0).sqrt());
        }
        assert!(worst_l < 1e-3, "angular momentum excursion {worst_l}");
    }

    #[test]
    fn six_degrees_of_freedom_and_equipartition_at_the_retained_modes() {
        let a = a_body_in_motion(31);
        assert_eq!(a.dof(), 6);
        let t = 293.0;
        let kt = K_B * t;
        let mut g = Lcg(99);
        let n = 200_000;
        let mut sum = 0.0;
        for _ in 0..n {
            let mut b = a;
            for k in 0..3 {
                b.p[k] = (a.body.mass * kt).sqrt() * g.gauss();
                b.l_body[k] = (a.body.inertia[k] * kt).sqrt() * g.gauss();
            }
            sum += b.kinetic();
        }
        let mean = sum / n as f64;
        assert!(close(mean, 0.5 * 6.0 * kt, 1e-2), "mean kinetic {mean} against 3 kT {}", 3.0 * kt);
    }

    #[test]
    fn the_clock_reads_the_body_it_is_asked_about() {
        let a = a_body_in_motion(41);
        let r = a.clock(&StiffnessEnvelope { k_max: 0.05 });
        assert!(r.dt > 0.0 && r.dt.is_finite());
        assert!(r.omega_dt <= clock::accuracy_target() * (1.0 + 1e-12));
        // the fine clock on this box is 1.077481 au; the rigid body's is above it at a
        // hydrogen-bond-order stiffness, which is the whole reason this operator exists
        assert!(r.dt > 1.077481, "rigid dt {} au against the fine 1.077481", r.dt);
    }

    #[test]
    fn quaternion_algebra_agrees_with_itself() {
        let mut g = Lcg(5);
        for _ in 0..50 {
            let q = g.unit_quat();
            let r = quat_to_mat(&q);
            // orthonormal, right-handed
            for i in 0..3 {
                for j in 0..3 {
                    let d: f64 = (0..3).map(|k| r[k][i] * r[k][j]).sum();
                    assert!(close(d, if i == j { 1.0 } else { 0.0 }, 1e-12));
                }
            }
            let det = r[0][0] * (r[1][1] * r[2][2] - r[1][2] * r[2][1]) - r[0][1] * (r[1][0] * r[2][2] - r[1][2] * r[2][0])
                + r[0][2] * (r[1][0] * r[2][1] - r[1][1] * r[2][0]);
            assert!(close(det, 1.0, 1e-12));
            // round trip up to sign
            let q2 = mat_to_quat(&r);
            let s = if q[0] * q2[0] + q[1] * q2[1] + q[2] * q2[2] + q[3] * q2[3] < 0.0 { -1.0 } else { 1.0 };
            for k in 0..4 {
                assert!(close(q[k], s * q2[k], 1e-12));
            }
            // composition: R(ab) = R(a) R(b)
            let b = g.unit_quat();
            let rab = quat_to_mat(&quat_mul(&q, &b));
            let rb = quat_to_mat(&b);
            for i in 0..3 {
                for j in 0..3 {
                    let m: f64 = (0..3).map(|k| r[i][k] * rb[k][j]).sum();
                    assert!(close(rab[i][j], m, 1e-12));
                }
            }
        }
    }
}
