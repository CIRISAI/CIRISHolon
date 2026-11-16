//! Monte Carlo for the Wilson action `S = beta sum_p (1 - Re tr U_p / 3)`.
//!
//! * The heat bath on SU(2): M. Creutz, "Monte Carlo study of quantized SU(2) gauge theory",
//!   Phys. Rev. D 21, 2308 (1980), doi:10.1103/PhysRevD.21.2308.
//! * SU(3) through its SU(2) subgroups: N. Cabibbo and E. Marinari, "A new method for
//!   updating SU(N) matrices in computer simulations of gauge theories", Phys. Lett. B 119,
//!   387 (1982), doi:10.1016/0370-2693(82)90696-7.
//! * The sampling of the SU(2) heat-bath distribution used here: A. D. Kennedy and
//!   B. J. Pendleton, "Improved heatbath method for Monte Carlo calculations in lattice gauge
//!   theories", Phys. Lett. B 156, 393 (1985), doi:10.1016/0370-2693(85)91632-6.
//! * Overrelaxation: M. Creutz, "Overrelaxation and Monte Carlo simulation", Phys. Rev. D
//!   36, 515 (1987), doi:10.1103/PhysRevD.36.515; F. R. Brown and T. J. Woch, "Overrelaxed
//!   heat-bath and Metropolis algorithms for accelerating pure gauge Monte Carlo
//!   calculations", Phys. Rev. Lett. 58, 2394 (1987), doi:10.1103/PhysRevLett.58.2394.
//!
//! Nothing here is new. A link `U` with staple sum `A` has weight `exp(beta Re tr(U A) / 3)`.
//! For each of the three SU(2) subgroups in turn, `U` is multiplied from the left by an
//! element `X` of that subgroup; the weight of `X` depends only on the quaternion part `k V`
//! of the subgroup's block of `U A` (`V` a unit quaternion, `k >= 0`), and is
//! `exp(alpha (X V)_0)` with `alpha = 2 beta k / 3`.
use crate::lattice::Lattice;
use crate::rng::Rng;
use crate::su3::quat_mul;

const SUBGROUPS: [(usize, usize); 3] = [(0, 1), (0, 2), (1, 2)];

/// A unit quaternion `(x0, x)` with density `sqrt(1 - x0^2) exp(alpha x0)` in `x0` and `x`
/// uniform in direction (Kennedy and Pendleton's rejection, exact at every `alpha > 0`).
fn kennedy_pendleton(alpha: f64, r: &mut Rng) -> [f64; 4] {
    let x0 = loop {
        let c = (std::f64::consts::TAU * r.f()).cos();
        let lambda2 = -(r.open().ln() + c * c * r.open().ln()) / (2.0 * alpha);
        let u = r.f();
        if u * u <= 1.0 - lambda2 {
            break 1.0 - 2.0 * lambda2;
        }
    };
    let rad = (1.0 - x0 * x0).max(0.0).sqrt();
    let z = 2.0 * r.f() - 1.0;
    let phi = std::f64::consts::TAU * r.f();
    let s = (1.0 - z * z).max(0.0).sqrt();
    [x0, rad * s * phi.cos(), rad * s * phi.sin(), rad * z]
}

impl Lattice {
    /// One heat-bath sweep: every link in turn, each of its three subgroups given a fresh
    /// element from the exact conditional distribution. Links are put back on the group.
    pub fn heatbath_sweep(&mut self, beta: f64, r: &mut Rng) {
        for s in 0..self.volume() {
            for mu in 0..4 {
                let a = self.staple(s, mu);
                let mut u = self.links[4 * s + mu];
                let mut w = u.mul(&a);
                for (i, j) in SUBGROUPS {
                    let q = w.su2_part(i, j);
                    let k = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
                    if k < 1e-300 {
                        continue;
                    }
                    let v_dag = [q[0] / k, -q[1] / k, -q[2] / k, -q[3] / k];
                    let x = quat_mul(kennedy_pendleton(2.0 * beta * k / 3.0, r), v_dag);
                    u.left_su2(i, j, x);
                    w.left_su2(i, j, x);
                }
                self.links[4 * s + mu] = u.reunitarise();
            }
        }
    }
    /// One overrelaxation sweep: every link reflected in each subgroup (`X = V^dag V^dag`),
    /// which keeps the action exactly and moves the configuration far. Not ergodic alone.
    pub fn overrelax_sweep(&mut self) {
        for s in 0..self.volume() {
            for mu in 0..4 {
                let a = self.staple(s, mu);
                let mut u = self.links[4 * s + mu];
                let mut w = u.mul(&a);
                for (i, j) in SUBGROUPS {
                    let q = w.su2_part(i, j);
                    let k = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
                    if k < 1e-300 {
                        continue;
                    }
                    let v_dag = [q[0] / k, -q[1] / k, -q[2] / k, -q[3] / k];
                    let x = quat_mul(v_dag, v_dag);
                    u.left_su2(i, j, x);
                    w.left_su2(i, j, x);
                }
                self.links[4 * s + mu] = u.reunitarise();
            }
        }
    }
    /// One compound sweep: a heat bath and then `n_or` overrelaxations.
    pub fn sweep(&mut self, beta: f64, n_or: usize, r: &mut Rng) {
        self.heatbath_sweep(beta, r);
        for _ in 0..n_or {
            self.overrelax_sweep();
        }
    }
}
