//! The Wilson (gradient) flow.
//!
//! M. Lüscher, "Properties and uses of the Wilson flow in lattice QCD", JHEP 08 (2010) 071,
//! arXiv:1006.4518, doi:10.1007/JHEP08(2010)071 (erratum JHEP 03 (2014) 092): the flow
//! `dV/dt = -g0^2 {d S_w(V)} V` (eq. 1.4) started from the configuration, in units where the
//! lattice spacing is one; integrated by the paper's third-order Runge-Kutta scheme
//! (appendix C, eqs. C.2 and C.3). At flow time `t` the field is the original one averaged
//! over a radius of about `sqrt(8 t)` (after eq. 2.12).
//!
//! In terms of the staple sum `A` of a link `U`, the generator is `Z = -P(U A)` with `P` the
//! traceless anti-Hermitian part. The flow is gauge covariant and lowers the Wilson action
//! monotonically.
use crate::lattice::Lattice;
use crate::su3::Su3;

impl Lattice {
    /// `eps Z(V)` on every link.
    fn flow_force(&self, eps: f64) -> Vec<Su3> {
        let mut z = Vec::with_capacity(self.links.len());
        for s in 0..self.volume() {
            for mu in 0..4 {
                z.push(self.links[4 * s + mu].mul(&self.staple(s, mu)).traceless_antihermitian().scale(-eps));
            }
        }
        z
    }
    /// One Runge-Kutta step of size `eps` (Lüscher's eq. C.2); the error is of order `eps^4`
    /// a step. Links are put back on the group at the end of the step.
    pub fn flow_step(&mut self, eps: f64) {
        // W1 = exp(Z0 / 4) W0
        let z0 = self.flow_force(eps);
        for (u, z) in self.links.iter_mut().zip(&z0) {
            *u = z.scale(0.25).exp().mul(u);
        }
        // W2 = exp(8/9 Z1 - 17/36 Z0) W1
        let mut z1 = self.flow_force(eps);
        for ((u, z1), z0) in self.links.iter_mut().zip(z1.iter_mut()).zip(&z0) {
            *z1 = z1.scale(8.0 / 9.0).sub(&z0.scale(17.0 / 36.0));
            *u = z1.exp().mul(u);
        }
        // V = exp(3/4 Z2 - 8/9 Z1 + 17/36 Z0) W2
        let z2 = self.flow_force(eps);
        for ((u, z2), z1) in self.links.iter_mut().zip(&z2).zip(&z1) {
            *u = z2.scale(0.75).sub(z1).exp().mul(u).reunitarise();
        }
    }
    /// Flow for a time `t` in `steps` equal steps.
    pub fn flow(&mut self, t: f64, steps: usize) {
        for _ in 0..steps {
            self.flow_step(t / steps as f64);
        }
    }
}
