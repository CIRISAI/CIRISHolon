//! What is measured. Every function here is gauge invariant site by site.
//!
//! The field strength is the "clover": the average of the four plaquettes that meet at a site
//! in a plane, made anti-Hermitian and traceless (Lüscher, arXiv:1006.4518, fig. 1). From it,
//! the action density `E = G^a_{mu nu} G^a_{mu nu} / 4` (eq. 2.1 there) and the topological
//! charge density `q = -eps_{mu nu rho sigma} tr(G_{mu nu} G_{rho sigma}) / (32 pi^2)`
//! (M. Lüscher and S. Schaefer, "Lattice QCD without topology barriers", JHEP 07 (2011) 036,
//! arXiv:1105.4749, eq. 2.6; anti-Hermitian generators with `tr(T^a T^b) = -delta^{ab}/2`).
use crate::lattice::Lattice;
use crate::su3::{C64, Su3};

/// The six planes, in the order `G` is returned by [`Lattice::field_strength`].
pub const PLANES: [(usize, usize); 6] = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];

impl Lattice {
    /// The mean of `Re tr U_p / 3` over all plaquettes.
    pub fn mean_plaquette(&self) -> f64 {
        let mut sum = 0.0;
        for s in 0..self.volume() {
            for (mu, nu) in PLANES {
                sum += self.plaquette(s, mu, nu).trace().re;
            }
        }
        sum / (18.0 * self.volume() as f64)
    }
    /// The Wilson action density at each site: `sum_{mu < nu} (1 - Re tr U_{mu nu}(s) / 3)`
    /// over the six plaquettes with their corner at the site.
    pub fn plaquette_action_density(&self) -> Vec<f64> {
        (0..self.volume()).map(|s| PLANES.iter().map(|&(mu, nu)| 1.0 - self.plaquette(s, mu, nu).trace().re / 3.0).sum()).collect()
    }
    /// The clover field strength at site `s` in the six [`PLANES`].
    pub fn field_strength(&self, s: usize) -> [Su3; 6] {
        let mut g = [Su3::ZERO; 6];
        for (k, (mu, nu)) in PLANES.into_iter().enumerate() {
            let (sm, sn) = (self.dn(s, mu), self.dn(s, nu));
            // the four leaves, each a loop from s back to s with the same sense
            let p1 = self.plaquette(s, mu, nu);
            let p2 = self.link(s, nu).mul_dag(self.link(self.up(sm, nu), mu)).mul_dag(self.link(sm, nu)).mul(self.link(sm, mu));
            let smn = self.dn(sm, nu);
            let p3 = self.link(sm, mu).dag().mul_dag(self.link(smn, nu)).mul(self.link(smn, mu)).mul(self.link(sn, nu));
            let p4 = self.link(sn, nu).dag().mul(self.link(sn, mu)).mul(self.link(self.up(sn, mu), nu)).mul_dag(self.link(s, mu));
            g[k] = p1.add(&p2).add(&p3).add(&p4).scale(0.25).traceless_antihermitian();
        }
        g
    }
    /// The clover action density `E(s) = -sum_{mu < nu} tr(G_{mu nu} G_{mu nu})` at each site.
    pub fn action_density(&self) -> Vec<f64> {
        (0..self.volume()).map(|s| -self.field_strength(s).iter().map(|g| g.re_tr_mul(g)).sum::<f64>()).collect()
    }
    /// The topological charge density at each site (its sum is the topological charge).
    pub fn topological_charge_density(&self) -> Vec<f64> {
        let k = -1.0 / (4.0 * std::f64::consts::PI * std::f64::consts::PI);
        (0..self.volume())
            .map(|s| {
                let g = self.field_strength(s);
                // (01)(23) - (02)(13) + (03)(12)
                k * (g[0].re_tr_mul(&g[5]) - g[1].re_tr_mul(&g[4]) + g[2].re_tr_mul(&g[3]))
            })
            .collect()
    }
    /// Both densities in one pass over the field strength: `(E, q)`.
    pub fn densities(&self) -> (Vec<f64>, Vec<f64>) {
        let k = -1.0 / (4.0 * std::f64::consts::PI * std::f64::consts::PI);
        (0..self.volume())
            .map(|s| {
                let g = self.field_strength(s);
                (-g.iter().map(|g| g.re_tr_mul(g)).sum::<f64>(), k * (g[0].re_tr_mul(&g[5]) - g[1].re_tr_mul(&g[4]) + g[2].re_tr_mul(&g[3])))
            })
            .unzip()
    }
    /// The topological charge: the sum of its density.
    pub fn topological_charge(&self) -> f64 {
        self.topological_charge_density().iter().sum()
    }
    /// The Polyakov loop at each spatial site `x0 + n0 (x1 + n1 x2)`: a third of the trace of
    /// the product of the links in direction 3 once around the lattice.
    pub fn polyakov_loops(&self) -> Vec<C64> {
        let v3 = self.dims[0] * self.dims[1] * self.dims[2];
        (0..v3)
            .map(|s0| {
                let (mut p, mut s) = (Su3::ONE, s0);
                for _ in 0..self.dims[3] {
                    p = p.mul(self.link(s, 3));
                    s = self.up(s, 3);
                }
                p.trace().scale(1.0 / 3.0)
            })
            .collect()
    }
}
