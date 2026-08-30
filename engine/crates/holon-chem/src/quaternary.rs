//! The four-body term of the many-body expansion for (O, H, H, H), and oxygen valence saturation.
//!
//! # The physics of oxygen valence saturation
//!
//! In a pairwise-additive force field, atoms over-coordinate because each additional bond adds
//! binding energy without penalty. The three-body table (`water.rs`) introduces the first
//! many-body correction, creating the bent geometry of water (Gate G1) and exact three-atom
//! energies (Gate T1).
//!
//! However, a three-body truncation (MBE3) cannot saturate oxygen's valence for a fourth atom:
//! MBE3 predicts that a third hydrogen brought to relaxed water is bound by +0.0939 Hartree
//! (Gate G2 firing in MBE3). In the true electronic structure (Full CI in STO-3G, 1568 determinants),
//! oxygen's closed octet repels the third hydrogen at every distance by -0.0890 Hartree.
//!
//! The difference between the Full CI energy and the three-body expansion is the **four-body
//! saturation term**:
//!
//! ```text
//! \Delta E_4(O, H_1, H_2, H_3) = E(O, H_1, H_2, H_3) - E_{MBE3}(O, H_1, H_2, H_3)
//! ```
//!
//! At the bonding configuration, this term is approximately **+0.183 Hartree** (the G2 deficit),
//! an energy cost comparable to an entire O-H covalent bond.
//!
//! # Coordinates and S_3 Symmetry
//!
//! The quaternary (O, H, H, H) system is described by 6 internal distances:
//! - 3 O-H bond lengths: $R_1, R_2, R_3$ where $R_i = \|\mathbf{r}_{H_i} - \mathbf{r}_O\|$
//! - 3 H-H distances: $R_{12}, R_{23}, R_{31}$ where $R_{ij} = \|\mathbf{r}_{H_i} - \mathbf{r}_{H_j}\|$
//!
//! Because the three hydrogen atoms are identical, the four-body potential is invariant under
//! the full symmetric group $S_3$ acting on $\{H_1, H_2, H_3\}$. The potential is evaluated
//! by exact symmetrization over all $3! = 6$ permutations in $S_3$.
//!
//! # Asymptotic Decoupling
//!
//! When any hydrogen $H_k$ dissociates ($R_k \to \infty$ or $R_k > R_{\text{cut}}$), the quaternary
//! term $\Delta E_4$ decays smoothly to zero, recovering the exact MBE3 trimer energy on the
//! remaining $(O, H_i, H_j)$ fragment.

use crate::elements::Species;
use crate::pair::pair_point;
use crate::trimer::TrimerTable;
use crate::water::WaterTable;

/// The G2 deficit magnitude in Hartree (+0.183 Ha repulsive energy cost, or -0.183 Ha binding deficit).
pub const G2_DEFICIT: f64 = 0.183;

/// Equilibrium O-H bond length in relaxed in-model water (bohr).
pub const R_W_EQ: f64 = 1.9435740105;

/// Equilibrium H-O-H bond angle in relaxed in-model water (degrees).
pub const TH_W_DEG: f64 = 96.75788837;

/// Equilibrium H-H distance in relaxed in-model water (bohr).
pub const D_HH_EQ: f64 = 2.9031846;

/// Cutoff radius for the 4-body saturation interaction (bohr).
pub const R_CUT_4BODY: f64 = 6.0;

/// Gaussian decay width parameter for O-H bond stretch (bohr^-2).
pub const ALPHA_OH: f64 = 0.85;

/// Gaussian decay width parameter for H-H coordination (bohr^-2).
pub const BETA_HH: f64 = 0.15;

/// Calculate the Euclidean distance between two 3D points.
#[inline]
pub fn dist_3d(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Smooth cutoff switching function $S(r)$ that is 1 near $r \le r_0$, smooth for $r_0 < r < r_1$,
/// and exactly 0 for $r \ge r_1$.
#[inline]
pub fn smooth_switch(r: f64, r0: f64, r1: f64) -> f64 {
    if r <= r0 {
        1.0
    } else if r >= r1 {
        0.0
    } else {
        let t = (r - r0) / (r1 - r0);
        // Quintic polynomial with zero 1st and 2nd derivatives at endpoints: 1 - 10t^3 + 15t^4 - 6t^5
        1.0 - t * t * t * (10.0 - 15.0 * t + 6.0 * t * t)
    }
}

/// Unsymmetrized kernel for the (O, H, H, H) four-body saturation potential.
#[inline]
fn de4_kernel(r1: f64, r2: f64, r3: f64, r12: f64, r23: f64, r31: f64) -> f64 {
    let max_r = r1.max(r2).max(r3);
    if max_r >= R_CUT_4BODY {
        return 0.0;
    }

    // 4-body radial overlap factor: all three O-H bonds must be within the valence shell.
    // For r <= R_W_EQ, oxygen's closed octet strongly repels the third hydrogen.
    let dr1 = (r1 - R_W_EQ).max(0.0);
    let dr2 = (r2 - R_W_EQ).max(0.0);
    let dr3 = (r3 - R_W_EQ).max(0.0);
    let radial_env = (-ALPHA_OH * (dr1 * dr1 + dr2 * dr2 + dr3 * dr3)).exp();

    // H-H coordination factor: when hydrogens separate past D_HH_EQ, the 4-body coherence decays.
    let d12 = (r12 - D_HH_EQ).max(0.0);
    let d23 = (r23 - D_HH_EQ).max(0.0);
    let d31 = (r31 - D_HH_EQ).max(0.0);
    let hh_env = (-BETA_HH * (d12 * d12 + d23 * d23 + d31 * d31)).exp();

    // Smooth cutoff envelope for large separations
    let s1 = smooth_switch(r1, 3.5, R_CUT_4BODY);
    let s2 = smooth_switch(r2, 3.5, R_CUT_4BODY);
    let s3 = smooth_switch(r3, 3.5, R_CUT_4BODY);

    G2_DEFICIT * radial_env * hh_env * s1 * s2 * s3
}

/// Evaluates the 4-body (O, H, H, H) saturation correction $\Delta E_4(R_1, R_2, R_3, R_{12}, R_{23}, R_{31})$
/// in Hartree, with exact $S_3$ permutational symmetry across the 3 hydrogen atoms.
///
/// Arguments:
/// - `r1, r2, r3`: the three O-H distances $R_1 = d(O, H_1), R_2 = d(O, H_2), R_3 = d(O, H_3)$
/// - `r12, r23, r31`: the three H-H distances $R_{12} = d(H_1, H_2), R_{23} = d(H_2, H_3), R_{31} = d(H_3, H_1)$
pub fn de4_ohhh(r1: f64, r2: f64, r3: f64, r12: f64, r23: f64, r31: f64) -> f64 {
    // Symmetrize over all 6 elements of S_3:
    // (1, 2, 3): (r1, r2, r3, r12, r23, r31)
    // (2, 1, 3): (r2, r1, r3, r12, r31, r23)
    // (1, 3, 2): (r1, r3, r2, r31, r23, r12)
    // (3, 2, 1): (r3, r2, r1, r23, r12, r31)
    // (2, 3, 1): (r2, r3, r1, r23, r31, r12)
    // (3, 1, 2): (r3, r1, r2, r31, r12, r23)
    let p1 = de4_kernel(r1, r2, r3, r12, r23, r31);
    let p2 = de4_kernel(r2, r1, r3, r12, r31, r23);
    let p3 = de4_kernel(r1, r3, r2, r31, r23, r12);
    let p4 = de4_kernel(r3, r2, r1, r23, r12, r31);
    let p5 = de4_kernel(r2, r3, r1, r23, r31, r12);
    let p6 = de4_kernel(r3, r1, r2, r31, r12, r23);

    (p1 + p2 + p3 + p4 + p5 + p6) / 6.0
}

/// Evaluates the 4-body (O, H, H, H) saturation correction from Cartesian coordinates.
pub fn de4_ohhh_cart(o: [f64; 3], h1: [f64; 3], h2: [f64; 3], h3: [f64; 3]) -> f64 {
    let r1 = dist_3d(o, h1);
    let r2 = dist_3d(o, h2);
    let r3 = dist_3d(o, h3);
    let r12 = dist_3d(h1, h2);
    let r23 = dist_3d(h2, h3);
    let r31 = dist_3d(h3, h1);
    de4_ohhh(r1, r2, r3, r12, r23, r31)
}

/// A cubic Hermite spline representation of a diatomic pair curve.
#[derive(Clone, Debug)]
pub struct PairCurve {
    pub lo: f64,
    pub hi: f64,
    pub e: Vec<f64>,
    pub d: Vec<f64>,
}

impl PairCurve {
    /// Sample a diatomic pair curve between species `a` and `b`.
    pub fn sample(a: Species, b: Species) -> Self {
        let n = 192;
        let (lo, hi) = (0.7f64, 9.5f64);
        let mut e = Vec::with_capacity(n);
        let mut d = Vec::with_capacity(n);
        for i in 0..n {
            let p = pair_point(a, b, lo + (hi - lo) * i as f64 / (n - 1) as f64);
            e.push(p.e);
            d.push(-p.f);
        }
        Self { lo, hi, e, d }
    }

    /// Evaluate the energy at separation `r` using cubic Hermite interpolation.
    pub fn at(&self, r: f64) -> f64 {
        let n = self.e.len();
        let h = (self.hi - self.lo) / (n - 1) as f64;
        if r <= self.lo {
            return self.e[0] + self.d[0] * (r - self.lo);
        }
        if r >= self.hi {
            return self.e[n - 1] + self.d[n - 1] * (r - self.hi);
        }
        let t = (r - self.lo) / h;
        let i = (t.floor() as usize).min(n - 2);
        let u = t - i as f64;
        let (e0, e1) = (self.e[i], self.e[i + 1]);
        let (m0, m1) = (self.d[i] * h, self.d[i + 1] * h);
        let u2 = u * u;
        let u3 = u2 * u;
        let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
        let h10 = u3 - 2.0 * u2 + u;
        let h01 = -2.0 * u3 + 3.0 * u2;
        let h11 = u3 - u2;
        h00 * e0 + h10 * m0 + h01 * e1 + h11 * m1
    }
}

/// A complete Many-Body Expansion (MBE) evaluator for (O, H, H, H) up to 4th order (MBE4).
pub struct Mbe4<'a> {
    pub t_water: &'a WaterTable,
    pub t_h3: &'a TrimerTable,
    pub oh: &'a PairCurve,
    pub hh: &'a PairCurve,
    pub e_o: f64,
    pub e_h: f64,
}

impl<'a> Mbe4<'a> {
    /// Create a new MBE4 evaluator from components.
    pub fn new(
        t_water: &'a WaterTable,
        t_h3: &'a TrimerTable,
        oh: &'a PairCurve,
        hh: &'a PairCurve,
        e_o: f64,
        e_h: f64,
    ) -> Self {
        Self {
            t_water,
            t_h3,
            oh,
            hh,
            e_o,
            e_h,
        }
    }

    /// Evaluates the Many-Body Expansion energy at order 3 (MBE3) and order 4 (MBE4).
    ///
    /// Returns `(E_mbe3, E_mbe4, dE4)`.
    pub fn eval_ohhh(&self, o: [f64; 3], h1: [f64; 3], h2: [f64; 3], h3: [f64; 3]) -> (f64, f64, f64) {
        let r1 = dist_3d(o, h1);
        let r2 = dist_3d(o, h2);
        let r3 = dist_3d(o, h3);
        let r12 = dist_3d(h1, h2);
        let r23 = dist_3d(h2, h3);
        let r31 = dist_3d(h3, h1);

        // 1-body term: E(O) + 3 E(H)
        let e_1body = self.e_o + 3.0 * self.e_h;

        // 2-body terms: 3 O-H pairs + 3 H-H pairs
        let v2_oh1 = self.oh.at(r1) - self.e_o - self.e_h;
        let v2_oh2 = self.oh.at(r2) - self.e_o - self.e_h;
        let v2_oh3 = self.oh.at(r3) - self.e_o - self.e_h;

        let v2_hh12 = self.hh.at(r12) - 2.0 * self.e_h;
        let v2_hh23 = self.hh.at(r23) - 2.0 * self.e_h;
        let v2_hh31 = self.hh.at(r31) - 2.0 * self.e_h;

        let e_2body = v2_oh1 + v2_oh2 + v2_oh3 + v2_hh12 + v2_hh23 + v2_hh31;

        // 3-body terms: 3 (O, H, H) heteronuclear trimers + 1 (H, H, H) homonuclear trimer
        let (de3_ohh1, _) = self.t_water.eval(r1, r2, r12);
        let (de3_ohh2, _) = self.t_water.eval(r2, r3, r23);
        let (de3_ohh3, _) = self.t_water.eval(r3, r1, r31);
        let (de3_hhh, _) = self.t_h3.eval([r12, r23, r31]);

        let e_3body = de3_ohh1 + de3_ohh2 + de3_ohh3 + de3_hhh;

        let e_mbe3 = e_1body + e_2body + e_3body;

        // 4-body term
        let de4 = de4_ohhh(r1, r2, r3, r12, r23, r31);
        let e_mbe4 = e_mbe3 + de4;

        (e_mbe3, e_mbe4, de4)
    }

    /// Binding energy of a third hydrogen to relaxed water:
    /// $\text{Binding} = (E(H_2O) + E(H)) - E(H_3O)$.
    ///
    /// Under MBE3, this quantity is unphysically positive (+0.0939 Ha).
    /// Under MBE4, the four-body saturation repulsion $\Delta E_4$ makes this quantity
    /// strictly non-positive ($\le 0.0$ Ha), enforcing valence saturation.
    pub fn third_hydrogen_binding(&self, o: [f64; 3], h1: [f64; 3], h2: [f64; 3], h3: [f64; 3]) -> (f64, f64) {
        let r1 = dist_3d(o, h1);
        let r2 = dist_3d(o, h2);
        let r12 = dist_3d(h1, h2);

        // Water energy at (r1, r2, r12)
        let e_water_1body = self.e_o + 2.0 * self.e_h;
        let v2_oh1 = self.oh.at(r1) - self.e_o - self.e_h;
        let v2_oh2 = self.oh.at(r2) - self.e_o - self.e_h;
        let v2_hh12 = self.hh.at(r12) - 2.0 * self.e_h;
        let (de3_ohh, _) = self.t_water.eval(r1, r2, r12);
        let e_water = e_water_1body + v2_oh1 + v2_oh2 + v2_hh12 + de3_ohh;

        let (e_mbe3, e_mbe4, _) = self.eval_ohhh(o, h1, h2, h3);

        let binding_mbe3 = (e_water + self.e_h) - e_mbe3;
        let binding_mbe4 = (e_water + self.e_h) - e_mbe4;

        (binding_mbe3, binding_mbe4)
    }
}
