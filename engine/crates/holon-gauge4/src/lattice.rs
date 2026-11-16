//! The lattice: an SU(3) matrix on every link of a periodic four-dimensional box.
use crate::rng::Rng;
use crate::su3::{c, Su3};

/// A gauge configuration. Site `(x0, x1, x2, x3)` has index `x0 + n0 (x1 + n1 (x2 + n2 x3))`;
/// direction 3 is taken as Euclidean time by the observables. The link from site `s` in
/// direction `mu` is `links[4 s + mu]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Lattice {
    pub dims: [usize; 4],
    pub links: Vec<Su3>,
    up: Vec<u32>,
    dn: Vec<u32>,
}

impl Lattice {
    /// Every link the identity (a "cold" start).
    pub fn cold(dims: [usize; 4]) -> Lattice {
        assert!(dims.iter().all(|&n| n >= 2), "every side at least two sites");
        let v = dims.iter().product::<usize>();
        assert!(v < (u32::MAX / 4) as usize);
        let (mut up, mut dn) = (vec![0u32; 4 * v], vec![0u32; 4 * v]);
        let stride = [1, dims[0], dims[0] * dims[1], dims[0] * dims[1] * dims[2]];
        for s in 0..v {
            for mu in 0..4 {
                let x = (s / stride[mu]) % dims[mu];
                up[4 * s + mu] = (s - x * stride[mu] + ((x + 1) % dims[mu]) * stride[mu]) as u32;
                dn[4 * s + mu] = (s - x * stride[mu] + ((x + dims[mu] - 1) % dims[mu]) * stride[mu]) as u32;
            }
        }
        Lattice { dims, links: vec![Su3::ONE; 4 * v], up, dn }
    }
    /// Every link drawn from the group at random (a "hot" start), from `seed`.
    pub fn hot(dims: [usize; 4], seed: u64) -> Lattice {
        let mut l = Lattice::cold(dims);
        let mut r = Rng::new(seed);
        for u in l.links.iter_mut() {
            *u = random_su3(&mut r);
        }
        l
    }
    #[inline]
    pub fn volume(&self) -> usize {
        self.links.len() / 4
    }
    /// The neighbour of site `s` one step forward in direction `mu`.
    #[inline]
    pub fn up(&self, s: usize, mu: usize) -> usize {
        self.up[4 * s + mu] as usize
    }
    /// The neighbour one step back.
    #[inline]
    pub fn dn(&self, s: usize, mu: usize) -> usize {
        self.dn[4 * s + mu] as usize
    }
    #[inline]
    pub fn link(&self, s: usize, mu: usize) -> &Su3 {
        &self.links[4 * s + mu]
    }
    /// The sum of the six staples about the link `(s, mu)`: the matrix `A` for which the
    /// plaquettes through that link `U` have traces summing to `tr(U A)`.
    pub fn staple(&self, s: usize, mu: usize) -> Su3 {
        let mut a = Su3::ZERO;
        let sm = self.up(s, mu);
        for nu in 0..4 {
            if nu == mu {
                continue;
            }
            // forward: U_nu(s+mu) U_mu(s+nu)^dag U_nu(s)^dag
            let sn = self.up(s, nu);
            a = a.add(&self.link(sm, nu).mul_dag(self.link(sn, mu)).mul_dag(self.link(s, nu)));
            // backward: U_nu(s+mu-nu)^dag U_mu(s-nu)^dag U_nu(s-nu)
            let sd = self.dn(s, nu);
            let smd = self.dn(sm, nu);
            a = a.add(&self.link(smd, nu).dag().mul_dag(self.link(sd, mu)).mul(self.link(sd, nu)));
        }
        a
    }
    /// The plaquette in the `(mu, nu)` plane with its corner at `s`, as a matrix based at `s`.
    #[inline]
    pub fn plaquette(&self, s: usize, mu: usize, nu: usize) -> Su3 {
        self.link(s, mu).mul(self.link(self.up(s, mu), nu)).mul_dag(self.link(self.up(s, nu), mu)).mul_dag(self.link(s, nu))
    }
    /// Put every link back onto the group (rounding drifts off it by an ulp per product).
    pub fn reunitarise(&mut self) {
        for u in self.links.iter_mut() {
            *u = u.reunitarise();
        }
    }
    /// The largest departure of any link from unitarity and from determinant one.
    pub fn group_defect(&self) -> (f64, f64) {
        let mut d = (0.0f64, 0.0f64);
        for u in &self.links {
            d.0 = d.0.max(u.unitarity_defect());
            let det = u.det();
            d.1 = d.1.max(((det.re - 1.0).powi(2) + det.im * det.im).sqrt());
        }
        d
    }
    /// The gauge transformation `g`: every link `U_mu(s)` becomes `g(s) U_mu(s) g(s+mu)^dag`.
    /// Nothing physical may change under it.
    pub fn gauge_transform(&mut self, g: &[Su3]) {
        assert_eq!(g.len(), self.volume());
        for s in 0..self.volume() {
            for mu in 0..4 {
                let t = self.up(s, mu);
                self.links[4 * s + mu] = g[s].mul(&self.links[4 * s + mu]).mul_dag(&g[t]);
            }
        }
    }
    /// A random gauge transformation: an independent group element on every site, from `seed`.
    pub fn random_gauge(&self, seed: u64) -> Vec<Su3> {
        let mut r = Rng::new(seed);
        (0..self.volume()).map(|_| random_su3(&mut r)).collect()
    }
    /// The largest entrywise difference between this configuration's links and another's.
    pub fn distance(&self, o: &Lattice) -> f64 {
        self.links.iter().zip(&o.links).map(|(a, b)| a.distance(b)).fold(0.0, f64::max)
    }
}

/// A group element with the Haar measure: a matrix of independent complex normal entries,
/// orthonormalised, is unitary and Haar-distributed up to the phases Gram-Schmidt fixes; the
/// third row is then set by the first two, which leaves the measure on SU(3) invariant.
pub fn random_su3(r: &mut Rng) -> Su3 {
    let mut m = [c(0.0, 0.0); 9];
    for z in m.iter_mut() {
        *z = c(r.normal(), r.normal());
    }
    Su3(m).reunitarise()
}
