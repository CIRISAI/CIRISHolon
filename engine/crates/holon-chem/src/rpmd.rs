//! C1's real physics: quantum nuclei on the banked pair curve.
//!
//! # What this module is
//!
//! [`crate::tower`] declares the C1 carrier — its state, its operator, its transport to
//! and from C0 — and nothing in it solves a quantum-nuclear problem. This module is the
//! physics behind that declaration, and it is built as an INSTRUMENT PLUS ITS REFEREE,
//! because a ring-polymer number with nothing to hit is not a measurement:
//!
//! * [`SincDvr`] / [`dvr_reference`] — the EXACT referee. A Colbert–Miller sinc discrete
//!   variable representation of the J = 0 nuclear Schrödinger equation on the engine's
//!   own H–H curve, which asserts its OWN convergence (grid, box, and an independent
//!   [`numerov_levels`] cross-check) and REFUSES rather than returning an unconverged
//!   number.
//! * [`Pimd1D`] — the instrument. A real path-integral molecular dynamics run: the
//!   `P`-bead ring-polymer Hamiltonian, exact free-ring-polymer evolution in normal
//!   modes, the PILE thermostat of Ceriotti–Parrinello–Markland–Manolopoulos, and both
//!   the primitive and the centroid-virial energy estimators.
//! * [`harmonic_ring_energy`] / [`MorsePes`] — the PLANTS. Two potentials whose
//!   `P`-bead ring-polymer energy and whose exact eigenvalues are known in closed form,
//!   so both instrument and referee are gauged against arithmetic before either is
//!   pointed at the banked curve. The Morse plant is not decoration: it is anharmonic,
//!   which is the sector the gate reads, and a harmonic-only plant would be a plant that
//!   does not act where the claim lives.
//!
//! # Scope, stated once
//!
//! ONE DIMENSION, `J = 0`. The observable is the vibrational zero-point energy of a
//! diatomic in its relative coordinate, with the reduced mass built from the engine's
//! own declared atomic masses. Rotation is NOT included and neither is the centre of
//! mass; a thermal 3D ring polymer at 300 K would carry a quantum rotational energy that
//! no 1D reference can grade, so the reference and the instrument are held to the same
//! Hamiltonian instead. The 3D machinery that DOES run here — [`classical_step_3d`],
//! [`ring_step_3d`], [`centroid_defect`] — is used for the classical limit and the
//! bead-forgetting commuting square, neither of which needs a spectral reference.
//!
//! # Which curve
//!
//! Both. [`ExactPes`] calls [`crate::h2::h2_point`] — the STO-3G FCI solver itself, no
//! interpolation anywhere. [`BankedPes`] is the engine's own [`crate::table::Table`],
//! the cubic Hermite interpolant on the `R^{-1/4}` knot grid that `holon-render`
//! integrates. The exact solver costs ~65 µs a call and a converged PIMD run needs
//! ~10^9 of them, so the sampling runs on the banked table — and the referee is run on
//! BOTH, which turns the interpolation systematic into a number in the gate's own
//! currency (hartree of zero-point energy) rather than a hope.

use crate::elements::{Species, M_E_PER_U};
use crate::h2::h2_point;
use crate::table::{generate_table, Table};
use crate::tower::{ClassicalState, RingPolymerState};

// ============================================================================
// 0. Declared inputs
// ============================================================================

/// Boltzmann's constant, hartree per kelvin. A MEASURED input, in the same class as
/// [`crate::elements::M_E_PER_U`]: nothing here computes it and nothing fits it.
///
/// Stated once. [`crate::tower`]'s `RingPolymerOp` and `bead_spring_constant` carried the
/// same literal twice between them, which is two places for one fact to drift.
pub const K_B_HARTREE_PER_KELVIN: f64 = 3.166811563e-6;

/// Atomic mass of deuterium (`2H`), unified atomic mass units. A MEASURED input, exactly
/// like every `Species::mass_u` in [`crate::elements`], and carrying the same convention:
/// the NUCLEUS PLUS ITS ELECTRON, because the pair curves are Born–Oppenheimer and the
/// electron rides with the nucleus.
///
/// Not in `elements.rs` because that table is one row per ELEMENT at its most abundant
/// isotope, and deuterium is the same element. The isotope shift is C1's free prediction,
/// so the number it turns on is declared where the prediction is made.
pub const MASS_U_DEUTERIUM: f64 = 2.01410177812;

/// Hartree to wavenumbers, `cm^-1` per hartree. A MEASURED input, used for REPORTING
/// only: no gate is stated in wavenumbers, so this constant cannot move a verdict.
pub const HARTREE_TO_CM_INV: f64 = 219474.6313632;

// ============================================================================
// 1. The potential energy surface, as something a sampler can call
// ============================================================================

/// A one-dimensional potential and its first derivative.
///
/// `eval` returns `(V, dV/dR)` together because every caller here needs both and the
/// banked curve carries both — splitting them would double the cost of the sampling loop
/// for no gain.
pub trait Pes: Sync {
    fn eval(&self, r: f64) -> (f64, f64);
    /// Name for reports. A LABEL — nothing computes from it.
    fn label(&self) -> &'static str;
    /// Evaluations that left the surface's declared domain. Zero for a closed-form
    /// plant; the banked table counts them, and a run that reports a nonzero value has
    /// been extrapolating rather than sampling.
    fn excursions(&self) -> u64 {
        0
    }
}

/// The STO-3G FCI H–H curve, solved at every call. No interpolation, no table.
#[derive(Clone, Copy, Debug)]
pub struct ExactPes;

impl Pes for ExactPes {
    #[inline]
    fn eval(&self, r: f64) -> (f64, f64) {
        let p = h2_point(r);
        // The table's `f` column is the FORCE, so `dE/dR = -f`.
        (p.e, -p.f)
    }
    fn label(&self) -> &'static str {
        "exact STO-3G FCI (h2::h2_point)"
    }
}

/// The engine's own banked curve: the cubic Hermite interpolant on the `R^{-1/4}` knot
/// grid that [`crate::table`] emits and `holon-render` integrates.
///
/// The index lookup INVERTS [`crate::table::grid_point`]'s map rather than searching, so
/// a call is `O(1)`; the interval is then checked and corrected by at most one step, so a
/// `powf` round-trip at a knot cannot silently select the wrong cubic.
pub struct BankedPes {
    table: Table,
    u_hi: f64,
    u_lo: f64,
    /// Evaluations that fell outside the table's range. REPORTED, never silently
    /// clamped into a lie: a run with a nonzero count is a run whose potential was
    /// extrapolated, and the gate refuses it.
    ///
    /// An atomic rather than a `Cell` behind an `unsafe impl Sync`: each sampling chain
    /// owns its own surface today, so the promise would hold — but it is a promise about
    /// how callers behave, written where no caller can see it, and `Relaxed` on a counter
    /// that is read once at the end costs nothing measurable.
    excursions: std::sync::atomic::AtomicU64,
}

impl BankedPes {
    /// Generate the banked table for the H–H curve at `n_knots` and wrap it.
    pub fn h2(n_knots: usize) -> BankedPes {
        let (r_min, r_max) = banked_range();
        let table = generate_table(r_min, r_max, n_knots).expect("banked H-H grid is usable");
        BankedPes::from_table(table)
    }

    pub fn from_table(table: Table) -> BankedPes {
        let u_hi = table.meta.r_min.powf(-0.25);
        let u_lo = table.meta.r_max.powf(-0.25);
        BankedPes { table, u_hi, u_lo, excursions: std::sync::atomic::AtomicU64::new(0) }
    }

    pub fn table(&self) -> &Table {
        &self.table
    }

    pub fn reset_excursions(&self) {
        self.excursions.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// The interpolant's own minimum: `(R, V)`, located by golden-section on the
    /// interpolant itself rather than inherited from the model. The two differ by the
    /// interpolation error, and that difference is exactly what a zero-point energy
    /// referred to "the bottom of the curve" is sensitive to.
    pub fn minimum(&self) -> (f64, f64) {
        let (mut a, mut b) = (0.8f64, 2.4f64);
        let phi = (5f64.sqrt() - 1.0) / 2.0;
        let (mut c, mut d) = (b - phi * (b - a), a + phi * (b - a));
        let (mut fc, mut fd) = (self.eval(c).0, self.eval(d).0);
        for _ in 0..200 {
            if fc < fd {
                b = d;
                d = c;
                fd = fc;
                c = b - phi * (b - a);
                fc = self.eval(c).0;
            } else {
                a = c;
                c = d;
                fc = fd;
                d = a + phi * (b - a);
                fd = self.eval(d).0;
            }
        }
        let r = 0.5 * (a + b);
        (r, self.eval(r).0)
    }
}

impl Pes for BankedPes {
    #[inline]
    fn eval(&self, r: f64) -> (f64, f64) {
        let t = &self.table;
        let n = t.r.len();
        if r < t.meta.r_min || r > t.meta.r_max {
            self.excursions.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        let u = r.powf(-0.25);
        let x = (u - self.u_hi) / (self.u_lo - self.u_hi) * ((n - 1) as f64);
        let mut i = if x.is_finite() { x.floor() as isize } else { 0 };
        if i < 0 {
            i = 0;
        }
        if i > (n - 2) as isize {
            i = (n - 2) as isize;
        }
        let mut i = i as usize;
        // At most one correction: the endpoint pinning in `grid_point` means the u-map
        // and the stored knots can disagree in the last bit at the two ends.
        if i + 1 < n - 1 && r > t.r[i + 1] {
            i += 1;
        } else if i > 0 && r < t.r[i] {
            i -= 1;
        }
        let (r0, r1) = (t.r[i], t.r[i + 1]);
        let h = r1 - r0;
        let s = (r - r0) / h;
        let (y0, y1) = (t.e[i], t.e[i + 1]);
        // Stored column is the FORCE; the interpolant is built on dE/dR.
        let (d0, d1) = (-t.f[i], -t.f[i + 1]);
        let s2 = s * s;
        let s3 = s2 * s;
        let v = (2.0 * s3 - 3.0 * s2 + 1.0) * y0
            + (s3 - 2.0 * s2 + s) * h * d0
            + (-2.0 * s3 + 3.0 * s2) * y1
            + (s3 - s2) * h * d1;
        // The interpolant's OWN derivative, so the force the sampler feels is the exact
        // gradient of the energy it accumulates. Anything else makes the ring-polymer
        // Hamiltonian non-conservative for reasons that have nothing to do with physics.
        let dv = ((6.0 * s2 - 6.0 * s) * y0 + (-6.0 * s2 + 6.0 * s) * y1) / h
            + (3.0 * s2 - 4.0 * s + 1.0) * d0
            + (3.0 * s2 - 2.0 * s) * d1;
        (v, dv)
    }
    fn label(&self) -> &'static str {
        "banked STO-3G FCI table (cubic Hermite on the R^-1/4 grid)"
    }
    fn excursions(&self) -> u64 {
        self.excursions.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// The grid range the engine derives for its own H–H table.
///
/// DELEGATED, not re-derived. [`crate::pair::derive_range`] already states once how a
/// grid range follows from the two DECLARED energies ([`crate::pair::WALL_CEILING`] above
/// the asymptote at the inner end, [`crate::pair::TAIL_TOLERANCE`] of it at the outer);
/// writing that walk a second time here would be a second place for the rule to drift,
/// and `examples/emit_curve.rs`'s hardcoded `(0.3, 10.0)` is already a third statement of
/// the same intent that nobody has to keep in step.
pub fn banked_range() -> (f64, f64) {
    let e_asym = 2.0 * crate::pair::atom_energy(Species::HYDROGEN);
    crate::pair::derive_range(Species::HYDROGEN, Species::HYDROGEN, e_asym)
}

/// A harmonic plant: `V = V0 + k (R - R0)^2 / 2`. Its ring-polymer energy at every bead
/// count is known in closed form ([`harmonic_ring_energy`]), which is what makes it a
/// gauge rather than an illustration.
#[derive(Clone, Copy, Debug)]
pub struct HarmonicPes {
    pub k: f64,
    pub r0: f64,
    pub v0: f64,
}

impl Pes for HarmonicPes {
    #[inline]
    fn eval(&self, r: f64) -> (f64, f64) {
        let d = r - self.r0;
        (self.v0 + 0.5 * self.k * d * d, self.k * d)
    }
    fn label(&self) -> &'static str {
        "harmonic plant"
    }
}

/// A Morse plant: `V = V0 + D_e (1 - exp(-a (R - R_e)))^2`.
///
/// THE PLANT THAT ACTS IN THE SECTOR THE GATE READS. Its bound spectrum is exact and
/// ANHARMONIC ([`morse_levels`]), so a referee that is right on it is right about the
/// quantity C1 is being graded on. A harmonic-only plant would certify an instrument in
/// a sector where the claim is not made (M-PLANT-SECTOR).
#[derive(Clone, Copy, Debug)]
pub struct MorsePes {
    pub d_e: f64,
    pub a: f64,
    pub r_e: f64,
    pub v0: f64,
}

impl Pes for MorsePes {
    #[inline]
    fn eval(&self, r: f64) -> (f64, f64) {
        let x = (-self.a * (r - self.r_e)).exp();
        let y = 1.0 - x;
        (self.v0 + self.d_e * y * y, 2.0 * self.d_e * y * self.a * x)
    }
    fn label(&self) -> &'static str {
        "Morse plant"
    }
}

/// Exact bound eigenvalues of [`MorsePes`], relative to `V0`.
///
/// `E_n = w (n + 1/2) - w^2 (n + 1/2)^2 / (4 D_e)` with `w = a sqrt(2 D_e / mu)`, for
/// `n` below the dissociation limit. Closed form, not a fit: this is the arithmetic the
/// DVR referee is graded against.
pub fn morse_levels(pes: &MorsePes, mu: f64, k: usize) -> Vec<f64> {
    let w = pes.a * (2.0 * pes.d_e / mu).sqrt();
    let lambda = (2.0 * mu * pes.d_e).sqrt() / pes.a;
    (0..k)
        .map(|n| {
            let v = n as f64 + 0.5;
            assert!(v < lambda, "Morse level {n} is above dissociation for this plant");
            w * v - w * w * v * v / (4.0 * pes.d_e)
        })
        .collect()
}

// ============================================================================
// 2. The one-dimensional vibrational problem
// ============================================================================

/// A `J = 0` diatomic vibrational problem: a reduced mass and a curve.
pub struct Vib1D<'a> {
    /// Reduced mass in ELECTRON masses, the unit the whole engine works in.
    pub mu: f64,
    pub pes: &'a dyn Pes,
    /// Isotopologue label. A LABEL.
    pub name: &'static str,
}

impl<'a> Vib1D<'a> {
    /// Reduced mass of a diatomic from its two DECLARED atomic masses, in electron
    /// masses. `mu = m_a m_b / (m_a + m_b)`, computed, never tabulated.
    pub fn reduced_mass_me(mass_a_u: f64, mass_b_u: f64) -> f64 {
        let (a, b) = (mass_a_u * M_E_PER_U, mass_b_u * M_E_PER_U);
        a * b / (a + b)
    }

    pub fn homonuclear(mass_u: f64, pes: &'a dyn Pes, name: &'static str) -> Vib1D<'a> {
        Vib1D { mu: Vib1D::reduced_mass_me(mass_u, mass_u), pes, name }
    }

    /// H2 on the given curve, with the mass the engine declares for hydrogen.
    pub fn h2(pes: &'a dyn Pes) -> Vib1D<'a> {
        Vib1D::homonuclear(Species::HYDROGEN.mass_u, pes, "H2")
    }

    /// D2 on the given curve. THE FREE PREDICTION: the curve is unchanged (the
    /// Born–Oppenheimer surface does not know about isotopes) and the ONLY thing that
    /// moves is the reduced mass.
    pub fn d2(pes: &'a dyn Pes) -> Vib1D<'a> {
        Vib1D::homonuclear(MASS_U_DEUTERIUM, pes, "D2")
    }
}

// ============================================================================
// 3. The exact referee: Colbert-Miller sinc DVR
// ============================================================================

/// A uniform-grid sinc discrete variable representation of `-(1/2mu) d^2/dR^2 + V(R)`.
///
/// Colbert & Miller, JCP 96, 1982 (1992), the `(-inf, inf)` kernel restricted to a box:
/// the truncation imposes the vanishing boundary condition, and the potential is diagonal
/// on the grid. The kinetic matrix is EXACT on the sinc basis, so the discretisation error
/// is spectral in the spacing and exponential in the box — which is why convergence can
/// be asserted from two grids rather than fitted.
#[derive(Clone, Copy, Debug)]
pub struct SincDvr {
    pub r_min: f64,
    pub r_max: f64,
    pub n: usize,
}

impl SincDvr {
    pub fn spacing(&self) -> f64 {
        (self.r_max - self.r_min) / ((self.n - 1) as f64)
    }

    pub fn grid(&self) -> Vec<f64> {
        let d = self.spacing();
        (0..self.n).map(|i| self.r_min + d * i as f64).collect()
    }

    /// The dense symmetric Hamiltonian, row-major.
    pub fn hamiltonian(&self, sys: &Vib1D) -> Vec<f64> {
        let n = self.n;
        let d = self.spacing();
        let pref = 1.0 / (2.0 * sys.mu * d * d);
        let grid = self.grid();
        let mut h = vec![0.0f64; n * n];
        for i in 0..n {
            for j in i..n {
                let t = if i == j {
                    pref * core::f64::consts::PI * core::f64::consts::PI / 3.0
                } else {
                    let dij = (i as f64) - (j as f64);
                    let sign = if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
                    pref * 2.0 * sign / (dij * dij)
                };
                h[i * n + j] = t;
                h[j * n + i] = t;
            }
            h[i * n + i] += sys.pes.eval(grid[i]).0;
        }
        h
    }

    /// The `k` lowest eigenvalues, by Lanczos with full reorthogonalisation.
    ///
    /// Returns `(levels, residual)`; `residual` is the largest Ritz residual over the
    /// returned levels, so a caller can refuse an unconverged answer instead of quoting
    /// it. The Krylov dimension is chosen from the spectral range rather than guessed.
    pub fn levels(&self, sys: &Vib1D, k: usize) -> (Vec<f64>, f64) {
        let h = self.hamiltonian(sys);
        lanczos_lowest(&h, self.n, k)
    }
}

/// Lowest `k` eigenvalues of a dense symmetric matrix by Lanczos with full
/// reorthogonalisation, plus the largest Ritz residual over those `k`.
///
/// The Krylov dimension is not guessed: it starts at 400 and DOUBLES until the Ritz
/// residual is inside `tol` or the whole space is spanned, at which point the answer is
/// exact by construction. Guessing it is how the first version of this referee reported
/// a residual of 1.5e-2 on a grid whose eigenvalues were correct to 2e-13 — the number
/// was honest and the instrument was under-resourced, which a fixed dimension cannot
/// tell you apart from a hard problem.
pub fn lanczos_lowest(h: &[f64], n: usize, k: usize) -> (Vec<f64>, f64) {
    lanczos_lowest_tol(h, n, k, 1e-11)
}

/// [`lanczos_lowest`] with an explicit residual target driving the restarts.
pub fn lanczos_lowest_tol(h: &[f64], n: usize, k: usize, tol: f64) -> (Vec<f64>, f64) {
    let mut m = core::cmp::min(n, core::cmp::max(4 * k + 40, 400));
    loop {
        let (vals, resid) = lanczos_fixed(h, n, k, m);
        if resid <= tol || m >= n {
            return (vals, resid);
        }
        m = core::cmp::min(n, m * 2);
    }
}

fn lanczos_fixed(h: &[f64], n: usize, k: usize, m: usize) -> (Vec<f64>, f64) {
    assert!(k >= 1 && k < n);
    let mut q: Vec<Vec<f64>> = Vec::with_capacity(m + 1);
    let mut alpha = Vec::with_capacity(m);
    let mut beta = Vec::with_capacity(m);
    // Deterministic start vector: a fixed pseudo-random draw, so the referee is
    // reproducible bit for bit and cannot be lucky on one run and not the next.
    let mut rng = Rng::new(0x5643_4952_5F44_5652);
    let mut v: Vec<f64> = (0..n).map(|_| rng.uniform() - 0.5).collect();
    normalise(&mut v);
    q.push(v);
    let mut w = vec![0.0f64; n];
    let mut used = 0usize;
    for j in 0..m {
        matvec(h, n, &q[j], &mut w);
        let a = dot(&w, &q[j]);
        alpha.push(a);
        for t in 0..n {
            w[t] -= a * q[j][t];
        }
        if j > 0 {
            let b = beta[j - 1];
            for t in 0..n {
                w[t] -= b * q[j - 1][t];
            }
        }
        // Full reorthogonalisation, twice. Once is not enough at this spectral range and
        // the cost is `O(m n)` against the `O(n^2)` matvec.
        for _ in 0..2 {
            for qi in q.iter() {
                let c = dot(&w, qi);
                for t in 0..n {
                    w[t] -= c * qi[t];
                }
            }
        }
        let b = norm(&w);
        used = j + 1;
        if b < 1e-13 {
            break;
        }
        beta.push(b);
        let mut nv = w.clone();
        for t in 0..n {
            nv[t] /= b;
        }
        q.push(nv);
    }
    let (vals, vecs) = tridiag_eigh(&alpha[..used], &beta[..used.saturating_sub(1)]);
    let bm = *beta.get(used.saturating_sub(1)).unwrap_or(&0.0);
    let mut resid: f64 = 0.0;
    for i in 0..k {
        // Standard Lanczos residual bound: |beta_m| times the last component of the Ritz
        // vector. This is the number that says the level is converged, and it is REPORTED
        // rather than assumed.
        resid = resid.max((bm * vecs[(used - 1) * used + i]).abs());
    }
    (vals[..k].to_vec(), resid)
}

fn matvec(h: &[f64], n: usize, x: &[f64], out: &mut [f64]) {
    for i in 0..n {
        let row = &h[i * n..i * n + n];
        let mut s = 0.0;
        for j in 0..n {
            s += row[j] * x[j];
        }
        out[i] = s;
    }
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn norm(a: &[f64]) -> f64 {
    dot(a, a).sqrt()
}

fn normalise(a: &mut [f64]) {
    let n = norm(a);
    for x in a.iter_mut() {
        *x /= n;
    }
}

/// Eigenvalues and eigenvectors of a real symmetric tridiagonal matrix, ascending.
/// Implicit-shift QL. Returns `(values, vectors)` with vectors column-major in a flat
/// `m*m` array: `vectors[row * m + col]`.
pub fn tridiag_eigh(diag: &[f64], off: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let m = diag.len();
    let mut d = diag.to_vec();
    let mut e = vec![0.0f64; m];
    e[..off.len()].copy_from_slice(off);
    let mut z = vec![0.0f64; m * m];
    for i in 0..m {
        z[i * m + i] = 1.0;
    }
    for l in 0..m {
        let mut iter = 0;
        loop {
            let mut mm = l;
            while mm + 1 < m {
                let dd = d[mm].abs() + d[mm + 1].abs();
                if e[mm].abs() <= f64::EPSILON * dd {
                    break;
                }
                mm += 1;
            }
            if mm == l {
                break;
            }
            iter += 1;
            assert!(iter < 60, "tridiagonal QL failed to converge");
            let mut g = (d[l + 1] - d[l]) / (2.0 * e[l]);
            let mut r = g.hypot(1.0);
            g = d[mm] - d[l] + e[l] / (g + if g >= 0.0 { r.abs() } else { -r.abs() });
            let (mut s, mut c) = (1.0f64, 1.0f64);
            let mut p = 0.0f64;
            for i in (l..mm).rev() {
                let mut f = s * e[i];
                let b = c * e[i];
                r = f.hypot(g);
                e[i + 1] = r;
                if r == 0.0 {
                    d[i + 1] -= p;
                    e[mm] = 0.0;
                    break;
                }
                s = f / r;
                c = g / r;
                g = d[i + 1] - p;
                r = (d[i] - g) * s + 2.0 * c * b;
                p = s * r;
                d[i + 1] = g + p;
                g = c * r - b;
                for kk in 0..m {
                    f = z[kk * m + i + 1];
                    z[kk * m + i + 1] = s * z[kk * m + i] + c * f;
                    z[kk * m + i] = c * z[kk * m + i] - s * f;
                }
            }
            d[l] -= p;
            e[l] = g;
            e[mm] = 0.0;
        }
    }
    // Ascending sort, vectors carried with the values.
    let mut order: Vec<usize> = (0..m).collect();
    order.sort_by(|&a, &b| d[a].partial_cmp(&d[b]).unwrap());
    let vals: Vec<f64> = order.iter().map(|&i| d[i]).collect();
    let mut vecs = vec![0.0f64; m * m];
    for (newc, &oldc) in order.iter().enumerate() {
        for row in 0..m {
            vecs[row * m + newc] = z[row * m + oldc];
        }
    }
    (vals, vecs)
}

// ============================================================================
// 4. The independent second instrument: Numerov
// ============================================================================

/// Bound levels by Numerov integration with two-sided matching.
///
/// A DIFFERENT DISCRETISATION FAMILY from the DVR — sixth-order finite difference against
/// a spectral basis — sharing only the Hamiltonian. Agreement between them is evidence
/// about the referee; agreement of the DVR with itself on two grids is not.
pub fn numerov_levels(sys: &Vib1D, r_min: f64, r_max: f64, n: usize, k: usize) -> Vec<f64> {
    let h = (r_max - r_min) / ((n - 1) as f64);
    let grid: Vec<f64> = (0..n).map(|i| r_min + h * i as f64).collect();
    let v: Vec<f64> = grid.iter().map(|&r| sys.pes.eval(r).0).collect();
    let v_min = v.iter().cloned().fold(f64::INFINITY, f64::min);
    let v_edge = v[0].min(v[n - 1]);

    // Sign changes of the outward Numerov solution of the Dirichlet problem on
    // [r_min, r_max]. The count is `n_nodes(E)`, a nondecreasing step function whose
    // jumps ARE the eigenvalues — so bisecting the COUNT converges on an eigenvalue
    // without ever needing a matching condition, and cannot be fooled by the divergent
    // forbidden-region tail (rescaling divides by a positive number and preserves sign).
    let nodes = |e: f64| -> usize {
        let f = |i: usize| 1.0 + h * h * 2.0 * sys.mu * (e - v[i]) / 12.0;
        let (mut ym1, mut y0) = (0.0f64, 1e-30f64);
        let mut count = 0usize;
        for i in 1..n - 1 {
            let mut y1 = ((12.0 - 10.0 * f(i)) * y0 - f(i - 1) * ym1) / f(i + 1);
            if y1 * y0 < 0.0 {
                count += 1;
            }
            if y1.abs() > 1e100 {
                let sc = y1.abs();
                y1 /= sc;
                y0 /= sc;
            }
            ym1 = y0;
            y0 = y1;
        }
        count
    };

    // One scan, all brackets. The scan is dense enough that two levels cannot share a
    // cell for any spectrum this referee is asked about, and a level that fails to
    // bracket is a REFUSAL by panic rather than a quietly returned neighbour.
    let scan = 8000usize;
    let mut brackets: Vec<(f64, f64)> = vec![(f64::NAN, f64::NAN); k];
    let mut prev_e = v_min;
    let mut prev_n = nodes(prev_e);
    for st in 1..=scan {
        let e = v_min + (v_edge - v_min) * (st as f64) / (scan as f64);
        let nd = nodes(e);
        if nd > prev_n {
            for target in prev_n..nd.min(k) {
                if brackets[target].0.is_nan() {
                    brackets[target] = (prev_e, e);
                }
            }
        }
        prev_e = e;
        prev_n = nd;
        if prev_n > k {
            break;
        }
    }

    let mut levels = Vec::with_capacity(k);
    for (target, &(a0, b0)) in brackets.iter().enumerate().take(k) {
        assert!(
            !a0.is_nan(),
            "Numerov: level {target} is not bracketed inside [{r_min}, {r_max}] — the box \
             does not hold the state the referee was asked for"
        );
        let (mut a, mut b) = (a0, b0);
        for _ in 0..200 {
            let mid = 0.5 * (a + b);
            if nodes(mid) <= target {
                a = mid;
            } else {
                b = mid;
            }
        }
        levels.push(0.5 * (a + b));
    }
    levels
}

// ============================================================================
// 5. The referee that refuses
// ============================================================================

/// Why a spectral reference was refused. A referee that returns a number it has not
/// convinced itself of is worse than one that returns nothing.
#[derive(Clone, Debug, PartialEq)]
pub enum RefereeRefusal {
    /// The Lanczos Ritz residual on a requested level exceeded the tolerance.
    Unconverged { residual: f64, tolerance: f64 },
    /// Halving the grid spacing moved a level by more than the tolerance.
    GridNotConverged { shift: f64, tolerance: f64 },
    /// Widening the box moved a level by more than the tolerance.
    BoxNotConverged { shift: f64, tolerance: f64 },
    /// The independent Numerov instrument disagreed by more than the tolerance.
    InstrumentsDisagree { gap: f64, tolerance: f64 },
}

impl core::fmt::Display for RefereeRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unconverged { residual, tolerance } => {
                write!(f, "REFUSAL: Lanczos residual {residual:.3e} > {tolerance:.3e}")
            }
            Self::GridNotConverged { shift, tolerance } => {
                write!(f, "REFUSAL: grid halving moved a level by {shift:.3e} > {tolerance:.3e}")
            }
            Self::BoxNotConverged { shift, tolerance } => {
                write!(f, "REFUSAL: box widening moved a level by {shift:.3e} > {tolerance:.3e}")
            }
            Self::InstrumentsDisagree { gap, tolerance } => {
                write!(f, "REFUSAL: DVR and Numerov disagree by {gap:.3e} > {tolerance:.3e}")
            }
        }
    }
}

/// A spectral reference that has ASSERTED ITS OWN CONVERGENCE.
#[derive(Clone, Debug)]
pub struct DvrReference {
    /// The lowest `k` vibrational levels, hartree, absolute (including `V` at its zero).
    pub levels: Vec<f64>,
    pub r_min: f64,
    pub r_max: f64,
    pub n: usize,
    /// Largest Lanczos Ritz residual over the returned levels.
    pub ritz_residual: f64,
    /// Largest level shift on halving the grid spacing.
    pub grid_shift: f64,
    /// Largest level shift on widening the box.
    pub box_shift: f64,
    /// Largest disagreement with the independent Numerov instrument.
    pub numerov_gap: f64,
    /// THE WORK COUNT. How many independent eigenproblems this reference actually solved,
    /// and how many potential evaluations it consumed. A convergence claim under a banner
    /// with no work behind it is the failure this field exists to make impossible.
    pub solves: usize,
    pub potential_calls: usize,
}

impl DvrReference {
    /// Zero-point energy relative to a stated potential minimum.
    pub fn zpe(&self, v_min: f64) -> f64 {
        self.levels[0] - v_min
    }

    /// Thermal energy of the vibrational mode at inverse temperature `beta`, from the
    /// reference's own spectrum. This is the quantity a thermostatted PIMD run measures;
    /// the zero-point energy is its `T -> 0` limit, and the difference between them is
    /// reported rather than assumed negligible.
    pub fn thermal_energy(&self, beta: f64, v_min: f64) -> f64 {
        let e0 = self.levels[0];
        let mut z = 0.0;
        let mut ez = 0.0;
        for &e in &self.levels {
            let w = (-beta * (e - e0)).exp();
            z += w;
            ez += w * (e - v_min);
        }
        ez / z
    }
}

/// Build a spectral reference and REFUSE it unless it converges.
///
/// The tolerances are absolute, in hartree, and every one of the four is checked:
/// the Lanczos residual, a grid halving, a box widening, and an independent Numerov
/// solve. Four checks, four refusals, and the count of solves is carried out with the
/// answer.
pub fn dvr_reference(
    sys: &Vib1D,
    r_min: f64,
    r_max: f64,
    r_floor: f64,
    n: usize,
    k: usize,
    tol: f64,
) -> Result<DvrReference, RefereeRefusal> {
    let count = std::sync::atomic::AtomicUsize::new(0);
    let counting = CountingPes { inner: sys.pes, count: &count };
    let sys_c = Vib1D { mu: sys.mu, pes: &counting, name: sys.name };

    let base = SincDvr { r_min, r_max, n };
    let (levels, resid) = base.levels(&sys_c, k);
    if resid > tol {
        return Err(RefereeRefusal::Unconverged { residual: resid, tolerance: tol });
    }

    // Grid: halve the spacing over the same box.
    let fine = SincDvr { r_min, r_max, n: 2 * n - 1 };
    let (lf, rf) = fine.levels(&sys_c, k);
    if rf > tol {
        return Err(RefereeRefusal::Unconverged { residual: rf, tolerance: tol });
    }
    let grid_shift = levels
        .iter()
        .zip(&lf)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f64, f64::max);
    if grid_shift > tol {
        return Err(RefereeRefusal::GridNotConverged { shift: grid_shift, tolerance: tol });
    }

    // Box: widen both ends by 20% of the span at the SAME spacing, so the two runs differ
    // in the box and not in the discretisation.
    let d = base.spacing();
    let pad = 0.2 * (r_max - r_min);
    // THE FENCE: `r_floor` is the smallest separation the surface is DEFINED at — the
    // banked table's inner knot, or minus infinity for a closed-form plant. Widening past
    // it would extrapolate, and an extrapolated box test certifies the extrapolation
    // rather than the box.
    let wr_min = (r_min - pad).max(r_floor);
    let wr_max = r_max + pad;
    let wn = (((wr_max - wr_min) / d).round() as usize) + 1;
    let wide = SincDvr { r_min: wr_min, r_max: wr_min + d * ((wn - 1) as f64), n: wn };
    let (lw, rw) = wide.levels(&sys_c, k);
    if rw > tol {
        return Err(RefereeRefusal::Unconverged { residual: rw, tolerance: tol });
    }
    let box_shift = levels
        .iter()
        .zip(&lw)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f64, f64::max);
    if box_shift > tol {
        return Err(RefereeRefusal::BoxNotConverged { shift: box_shift, tolerance: tol });
    }

    // The independent instrument.
    let nn = 20001;
    let ln = numerov_levels(&sys_c, r_min, r_max, nn, k);
    let numerov_gap = levels
        .iter()
        .zip(&ln)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f64, f64::max);
    let numerov_tol = tol * 1e4;
    if numerov_gap > numerov_tol {
        return Err(RefereeRefusal::InstrumentsDisagree {
            gap: numerov_gap,
            tolerance: numerov_tol,
        });
    }

    Ok(DvrReference {
        levels,
        r_min,
        r_max,
        n,
        ritz_residual: resid.max(rf).max(rw),
        grid_shift,
        box_shift,
        numerov_gap,
        solves: 4,
        potential_calls: count.load(std::sync::atomic::Ordering::Relaxed),
    })
}

struct CountingPes<'a> {
    inner: &'a dyn Pes,
    count: &'a std::sync::atomic::AtomicUsize,
}

impl<'a> Pes for CountingPes<'a> {
    #[inline]
    fn eval(&self, r: f64) -> (f64, f64) {
        self.count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.inner.eval(r)
    }
    fn label(&self) -> &'static str {
        self.inner.label()
    }
    fn excursions(&self) -> u64 {
        self.inner.excursions()
    }
}

// ============================================================================
// 6. Deterministic randomness
// ============================================================================

/// xoshiro256** seeded through splitmix64. Written here because this crate has zero
/// dependencies and that is load-bearing, and because a sampling run whose stream is not
/// reproducible cannot be re-refereed.
#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
    spare: Option<f64>,
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        let mut z = seed;
        let mut next = || {
            z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut x = z;
            x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            x ^ (x >> 31)
        };
        Rng { s: [next(), next(), next(), next()], spare: None }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let r = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        r
    }

    /// Uniform on `[0, 1)`.
    #[inline]
    pub fn uniform(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) * (1.0 / 9007199254740992.0)
    }

    /// Standard normal, Marsaglia polar (no trigonometry, no rejection bias).
    #[inline]
    pub fn normal(&mut self) -> f64 {
        if let Some(v) = self.spare.take() {
            return v;
        }
        loop {
            let u = 2.0 * self.uniform() - 1.0;
            let v = 2.0 * self.uniform() - 1.0;
            let s = u * u + v * v;
            if s > 0.0 && s < 1.0 {
                let f = (-2.0 * s.ln() / s).sqrt();
                self.spare = Some(v * f);
                return u * f;
            }
        }
    }
}

// ============================================================================
// 7. The ring polymer, exactly: normal modes and the free propagator
// ============================================================================

/// The `P`-bead free-ring-polymer normal-mode basis.
///
/// Frequencies `omega_k = 2 (P / (beta hbar)) sin(k pi / P)`, and the orthogonal
/// transform of Ceriotti et al. (JCP 133, 124104 (2010), eq. 21). The free ring polymer
/// is propagated EXACTLY in this basis, so the time step is limited by the physical
/// potential alone and never by the spring frequencies — which is the whole reason a
/// 256-bead run at 300 K is affordable.
pub struct NormalModes {
    pub p: usize,
    /// `c[j * p + k]`, orthogonal.
    pub c: Vec<f64>,
    /// `omega_k`, atomic units.
    pub omega: Vec<f64>,
}

/// The exact free-ring-polymer propagator for one time step, on POSITIONS and
/// VELOCITIES, with its trigonometry precomputed.
///
/// ONE statement, used by both the 1D sampler ([`run_pimd`]) and the 3D dynamics
/// ([`ring_step_3d`]). It was written twice — once on momenta, once on velocities — and
/// the momentum copy's multiply-then-divide by the mass is not the identity in f64, which
/// cost the `P = 1` classical-limit gate its bit-exactness. A second copy of a propagator
/// is a second place for it to be wrong.
#[derive(Clone, Debug)]
pub struct FreeRingPropagator {
    pub dt: f64,
    cs: Vec<f64>,
    sn: Vec<f64>,
}

impl FreeRingPropagator {
    pub fn new(nm: &NormalModes, dt: f64) -> FreeRingPropagator {
        FreeRingPropagator {
            dt,
            cs: nm.omega.iter().map(|w| (w * dt).cos()).collect(),
            sn: nm.omega.iter().map(|w| (w * dt).sin()).collect(),
        }
    }

    /// Advance one step in place. Mode 0 is the free centroid — exactly the drift step of
    /// velocity Verlet, which is why `P = 1` reproduces the classical trajectory bit for
    /// bit rather than merely closely.
    #[inline]
    pub fn apply(&self, nm: &NormalModes, qt: &mut [f64], vt: &mut [f64]) {
        qt[0] += vt[0] * self.dt;
        for k in 1..nm.p {
            let w = nm.omega[k];
            let (cs, sn) = (self.cs[k], self.sn[k]);
            let vk = vt[k];
            let qk = qt[k];
            vt[k] = cs * vk - w * sn * qk;
            qt[k] = sn * vk / w + cs * qk;
        }
    }
}

impl NormalModes {
    pub fn new(p: usize, beta: f64) -> NormalModes {
        let pf = p as f64;
        let omega_p = pf / beta; // hbar = 1
        let mut c = vec![0.0f64; p * p];
        let mut omega = vec![0.0f64; p];
        for k in 0..p {
            omega[k] = 2.0 * omega_p * ((k as f64) * core::f64::consts::PI / pf).sin();
            for j in 0..p {
                let jf = j as f64;
                let kf = k as f64;
                let v = if k == 0 {
                    (1.0 / pf).sqrt()
                } else if k < p / 2 || (p % 2 == 1 && k <= p / 2) {
                    (2.0 / pf).sqrt() * (2.0 * core::f64::consts::PI * jf * kf / pf).cos()
                } else if p % 2 == 0 && k == p / 2 {
                    (1.0 / pf).sqrt() * if j % 2 == 0 { 1.0 } else { -1.0 }
                } else {
                    (2.0 / pf).sqrt() * (2.0 * core::f64::consts::PI * jf * kf / pf).sin()
                };
                c[j * p + k] = v;
            }
        }
        NormalModes { p, c, omega }
    }

    /// `out_k = sum_j c[j][k] x_j`.
    pub fn to_modes(&self, x: &[f64], out: &mut [f64]) {
        let p = self.p;
        for k in 0..p {
            let mut s = 0.0;
            for j in 0..p {
                s += self.c[j * p + k] * x[j];
            }
            out[k] = s;
        }
    }

    /// `out_j = sum_k c[j][k] xt_k`.
    pub fn from_modes(&self, xt: &[f64], out: &mut [f64]) {
        let p = self.p;
        for j in 0..p {
            let mut s = 0.0;
            for k in 0..p {
                s += self.c[j * p + k] * xt[k];
            }
            out[j] = s;
        }
    }
}

// ============================================================================
// 8. PIMD: the instrument
// ============================================================================

/// A ring-polymer sampling run, fully specified.
#[derive(Clone, Copy, Debug)]
pub struct PimdConfig {
    pub p: usize,
    pub temperature_k: f64,
    /// Time step, atomic units.
    pub dt: f64,
    /// Centroid Langevin friction, atomic units. The internal modes take the PILE-L
    /// optimum `gamma_k = 2 omega_k` and are not a free choice; the centroid's friction
    /// is, and it is DECLARED rather than tuned to an answer.
    pub gamma_centroid: f64,
    pub steps_equil: u64,
    pub steps_sample: u64,
    pub seed: u64,
}

/// What one run measured, with the work it did and the error it carries.
#[derive(Clone, Debug)]
pub struct PimdReport {
    pub p: usize,
    pub beta: f64,
    pub dt: f64,
    /// Centroid-virial energy estimator, hartree. THE primary estimator: its variance
    /// does not grow with `P`.
    pub e_virial: f64,
    pub e_virial_err: f64,
    /// Primitive estimator, hartree. Same expectation, much larger variance — carried
    /// because agreement between two estimators of one quantity is a check the run
    /// cannot pass by accident.
    pub e_primitive: f64,
    pub e_primitive_err: f64,
    pub steps: u64,
    pub samples: u64,
    /// Integrated autocorrelation time of the virial estimator, in samples, from the
    /// blocking plateau. The reason the error bar is not the naive one.
    pub tau_int: f64,
    /// Mean centroid position, bohr.
    pub centroid: f64,
    /// Mean ring radius of gyration, bohr — the quantum delocalisation.
    pub radius_of_gyration: f64,
    /// Potential evaluations outside the banked table's range. Any nonzero value voids
    /// the run.
    pub excursions: u64,
    pub potential_calls: u64,
    pub chains: usize,
}

/// Run one chain. Deterministic in `cfg.seed`.
///
/// The state is carried IN NORMAL MODES and leaves them only to be handed to the
/// potential: the thermostat, the springs and the free propagator are all diagonal
/// there. That is two transforms a step instead of eight, and — because positions move
/// only in the free-evolution step — ONE force evaluation a step, reused by the second
/// half-kick of this step and the first half-kick of the next.
pub fn run_pimd(sys: &Vib1D, cfg: &PimdConfig, r_start: f64) -> PimdReport {
    let p = cfg.p;
    let pf = p as f64;
    let beta = 1.0 / (K_B_HARTREE_PER_KELVIN * cfg.temperature_k);
    let beta_n = beta / pf;
    let m = sys.mu;
    let nm = NormalModes::new(p, beta);
    let mut rng = Rng::new(cfg.seed);

    // Start on the free-ring-polymer distribution about `r_start`, so equilibration does
    // not have to inflate a collapsed ring through the stiffest modes it owns.
    let mut qt = vec![0.0f64; p];
    let mut pt = vec![0.0f64; p];
    qt[0] = r_start * pf.sqrt(); // centroid mode: q~_0 = sqrt(P) * q_centroid
    for k in 1..p {
        qt[k] = rng.normal() / (beta_n * m * nm.omega[k] * nm.omega[k]).sqrt();
    }
    // Velocities, not momenta: the free propagator is shared with `ring_step_3d` and is
    // written on velocities so no mass round trip can enter it.
    let sigma_v = 1.0 / (beta_n * m).sqrt();
    for x in pt.iter_mut() {
        *x = sigma_v * rng.normal();
    }

    let mut q = vec![0.0f64; p];
    let mut force = vec![0.0f64; p];
    let mut ft = vec![0.0f64; p];
    let mut vpot = vec![0.0f64; p];
    let mut calls = 0u64;

    nm.from_modes(&qt, &mut q);
    for j in 0..p {
        let (v, dv) = sys.pes.eval(q[j]);
        vpot[j] = v;
        force[j] = -dv;
    }
    calls += p as u64;
    nm.to_modes(&force, &mut ft);

    // PILE friction: the PILE-L optimum `2 omega_k` on the internal modes, the DECLARED
    // value on the centroid. The internal-mode choice is not free and is not tuned.
    let gamma: Vec<f64> = (0..p)
        .map(|k| if k == 0 { cfg.gamma_centroid } else { 2.0 * nm.omega[k] })
        .collect();
    let half = 0.5 * cfg.dt;
    // BAOAB, not OBABO: one FULL thermostat step in the middle rather than two halves at
    // the ends. Same cost — one force evaluation and two transforms a step either way —
    // and the configurational sampling error drops from O(dt^2) to O(dt^4). This was not
    // a preference: the plant-3 gauge measured OBABO's dt^2 bias at +2e-5 to +4e-5 Ha at
    // dt = 4, which is 0.2-0.3% of the zero-point energy the gate reads, and a systematic
    // that size eats most of a 0.4% band before any physics is tested.
    let c1: Vec<f64> = gamma.iter().map(|&g| (-g * cfg.dt).exp()).collect();
    let c2: Vec<f64> = c1.iter().map(|&c| (1.0 - c * c).sqrt()).collect();

    let mut samples_v: Vec<f64> = Vec::with_capacity(cfg.steps_sample as usize);
    let mut samples_p: Vec<f64> = Vec::with_capacity(cfg.steps_sample as usize);
    let mut rg_acc = 0.0f64;
    let mut qc_acc = 0.0f64;
    let total = cfg.steps_equil + cfg.steps_sample;
    let inv_sqrt_p = 1.0 / pf.sqrt();

    let prop = FreeRingPropagator::new(&nm, half);

    for step in 0..total {
        // B: half kick (force from the end of the previous step, where the position is).
        for k in 0..p {
            pt[k] += half * ft[k] / m;
        }
        // A: exact free ring-polymer evolution, half step.
        prop.apply(&nm, &mut qt, &mut pt);
        // O: thermostat, FULL step (diagonal in normal modes, exact Ornstein-Uhlenbeck).
        for k in 0..p {
            pt[k] = c1[k] * pt[k] + c2[k] * sigma_v * rng.normal();
        }
        // A: exact free ring-polymer evolution, half step.
        prop.apply(&nm, &mut qt, &mut pt);
        // The only excursion out of the normal-mode picture: the potential.
        nm.from_modes(&qt, &mut q);
        for j in 0..p {
            let (v, dv) = sys.pes.eval(q[j]);
            vpot[j] = v;
            force[j] = -dv;
        }
        calls += p as u64;
        nm.to_modes(&force, &mut ft);
        // B: half kick.
        for k in 0..p {
            pt[k] += half * ft[k] / m;
        }

        if step >= cfg.steps_equil {
            let qc = qt[0] * inv_sqrt_p;
            let mut v_mean = 0.0;
            let mut vir = 0.0;
            let mut rg2 = 0.0;
            for j in 0..p {
                v_mean += vpot[j];
                // force = -dV/dR, so (q - qc) dV/dR = -(q - qc) force
                vir -= (q[j] - qc) * force[j];
                rg2 += (q[j] - qc) * (q[j] - qc);
            }
            v_mean /= pf;
            vir /= pf;
            // The spring energy is DIAGONAL in normal modes: sum_k (1/2) m omega_k^2 q~_k^2.
            // Exact, and it costs P multiplies instead of a transform.
            let mut spring = 0.0;
            for k in 1..p {
                spring += 0.5 * m * nm.omega[k] * nm.omega[k] * qt[k] * qt[k];
            }
            // Centroid-virial estimator, one degree of freedom:
            //   E_cv = 1/(2 beta) + <V>_beads + (1/2) <(q_k - q_c) dV/dq_k>_beads
            samples_v.push(0.5 / beta + v_mean + 0.5 * vir);
            // Primitive estimator, one degree of freedom:
            //   E_pr = P/(2 beta) - <spring>/P + <V>_beads
            // Same expectation, variance growing with P — carried so the two can be
            // required to agree, which no single estimator can check about itself.
            samples_p.push(0.5 * pf / beta - spring / pf + v_mean);
            rg_acc += (rg2 / pf).sqrt();
            qc_acc += qc;
        }
    }

    let n = samples_v.len() as f64;
    let (mv, ev, tau) = blocked_mean(&samples_v);
    let (mp, ep, _) = blocked_mean(&samples_p);
    PimdReport {
        p,
        beta,
        dt: cfg.dt,
        e_virial: mv,
        e_virial_err: ev,
        e_primitive: mp,
        e_primitive_err: ep,
        steps: total,
        samples: samples_v.len() as u64,
        tau_int: tau,
        centroid: qc_acc / n,
        radius_of_gyration: rg_acc / n,
        excursions: sys.pes.excursions(),
        potential_calls: calls,
        chains: 1,
    }
}

/// Mean, standard error from the blocking plateau, and the implied integrated
/// autocorrelation time.
///
/// The naive standard error of a correlated chain is a number that looks like a result
/// and is not one; blocking is what makes it one, and the plateau is REPORTED so a run
/// whose blocks never plateau can be seen rather than believed.
pub fn blocked_mean(x: &[f64]) -> (f64, f64, f64) {
    let n = x.len();
    let mean = x.iter().sum::<f64>() / n as f64;
    if n < 32 {
        return (mean, f64::INFINITY, f64::INFINITY);
    }
    let var0 = x.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / ((n - 1) as f64);
    let naive = (var0 / n as f64).sqrt();
    let mut best = naive;
    let mut b = 2usize;
    while n / b >= 16 {
        let nb = n / b;
        let mut bm = Vec::with_capacity(nb);
        for i in 0..nb {
            bm.push(x[i * b..(i + 1) * b].iter().sum::<f64>() / b as f64);
        }
        let m2 = bm.iter().sum::<f64>() / nb as f64;
        let v2 = bm.iter().map(|v| (v - m2) * (v - m2)).sum::<f64>() / ((nb - 1) as f64);
        let se = (v2 / nb as f64).sqrt();
        if se > best {
            best = se;
        }
        b *= 2;
    }
    let tau = if naive > 0.0 { 0.5 * (best / naive) * (best / naive) } else { 0.0 };
    (mean, best, tau)
}

/// Run `chains` independent chains in parallel and combine them.
///
/// The combined error bar is the LARGER of the pooled within-chain blocking error and the
/// between-chain standard error. Two estimators of one uncertainty, and the pessimistic
/// one is reported: a blocking plateau that has not actually plateaued is invisible from
/// inside one chain and obvious between chains.
pub fn run_pimd_chains(
    mu: f64,
    name: &'static str,
    cfg: &PimdConfig,
    chains: usize,
    r_start: f64,
    make_pes: &(dyn Fn() -> Box<dyn Pes> + Sync),
) -> PimdReport {
    let reports: Vec<PimdReport> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..chains)
            .map(|c| {
                let mut cc = *cfg;
                cc.seed = cfg.seed.wrapping_add(0x9e37_79b9_7f4a_7c15u64.wrapping_mul(c as u64 + 1));
                s.spawn(move || {
                    let pes = make_pes();
                    let local = Vib1D { mu, pes: pes.as_ref(), name };
                    run_pimd(&local, &cc, r_start)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("PIMD chain panicked")).collect()
    });

    let k = reports.len() as f64;
    let mv = reports.iter().map(|r| r.e_virial).sum::<f64>() / k;
    let mp = reports.iter().map(|r| r.e_primitive).sum::<f64>() / k;
    let pooled_v = (reports.iter().map(|r| r.e_virial_err * r.e_virial_err).sum::<f64>()).sqrt() / k;
    let pooled_p =
        (reports.iter().map(|r| r.e_primitive_err * r.e_primitive_err).sum::<f64>()).sqrt() / k;
    let between_v = if reports.len() > 1 {
        (reports.iter().map(|r| (r.e_virial - mv).powi(2)).sum::<f64>() / (k - 1.0) / k).sqrt()
    } else {
        0.0
    };
    let between_p = if reports.len() > 1 {
        (reports.iter().map(|r| (r.e_primitive - mp).powi(2)).sum::<f64>() / (k - 1.0) / k).sqrt()
    } else {
        0.0
    };
    PimdReport {
        p: cfg.p,
        beta: reports[0].beta,
        dt: cfg.dt,
        e_virial: mv,
        e_virial_err: pooled_v.max(between_v),
        e_primitive: mp,
        e_primitive_err: pooled_p.max(between_p),
        steps: reports.iter().map(|r| r.steps).sum(),
        samples: reports.iter().map(|r| r.samples).sum(),
        tau_int: reports.iter().map(|r| r.tau_int).fold(0.0, f64::max),
        centroid: reports.iter().map(|r| r.centroid).sum::<f64>() / k,
        radius_of_gyration: reports.iter().map(|r| r.radius_of_gyration).sum::<f64>() / k,
        excursions: reports.iter().map(|r| r.excursions).sum(),
        potential_calls: reports.iter().map(|r| r.potential_calls).sum(),
        chains: reports.len(),
    }
}

/// EXACT ring-polymer energy of a harmonic oscillator at `P` beads.
///
/// Derived, not quoted. Integrating the momenta and the `P` coupled Gaussian modes gives
/// `Z_P = prod_k [beta_P hbar sqrt(omega_k^2 + omega^2)]^{-1}` with
/// `omega_k = (2P/beta) sin(k pi / P)`, and `E_P = -d ln Z_P / d beta` collapses to
///
/// ```text
///   E_P = (1/beta) sum_{k=0}^{P-1} omega^2 / (omega_k^2 + omega^2)
/// ```
///
/// which is `kT` at `P = 1` (classical) and `(omega/2) coth(beta omega / 2)` as
/// `P -> infinity` (quantum). THE PLANT: a sampler that does not reproduce this at every
/// `P` is broken in the bead sector, which is the sector the convergence ladder reads.
///
/// Its low-temperature tail is `E_P = omega/2 - beta^2 omega^3 / (16 P^2) + O(P^-4)`,
/// measured against this sum at ratio 0.99959 (P = 512) rising to 0.99999 (P = 4096).
/// The coefficient is `1/16` and NOT the `1/48` this module first documented: that came
/// from differentiating a form of the partition function that treats the ring frequency
/// `omega~` as independent of `beta`, which it is not. The tell is the classical limit —
/// any expression for `E_P` that does not give exactly `kT` at `P = 1` is not the energy
/// of this ensemble — and `tests/c1_quantum_nuclei.rs` now asserts both.
pub fn harmonic_ring_energy(omega: f64, beta: f64, p: usize) -> f64 {
    let pf = p as f64;
    let mut s = 0.0;
    for k in 0..p {
        let wk = 2.0 * (pf / beta) * ((k as f64) * core::f64::consts::PI / pf).sin();
        s += omega * omega / (wk * wk + omega * omega);
    }
    s / beta
}

/// The `P -> infinity` limit of [`harmonic_ring_energy`]: the exact quantum thermal energy
/// of a harmonic oscillator.
pub fn harmonic_exact_energy(omega: f64, beta: f64) -> f64 {
    0.5 * omega / (0.5 * beta * omega).tanh()
}

// ============================================================================
// 9. Three-dimensional dynamics on the tower's own state types
// ============================================================================
//
// UNITS, stated once because two conventions meet here: positions are BOHR, velocities
// bohr per atomic time unit, and `masses` are UNIFIED ATOMIC MASS UNITS — the unit
// `tower::RingPolymerOp::evaluate_energy` and `C1_RingPolymer::bead_spring_constant`
// already assume, converted with `elements::M_E_PER_U`.
//
// THE DYNAMICAL HAMILTONIAN is the standard ring-polymer one (Craig & Manolopoulos, JCP
// 121, 3368 (2004)):
//
//   H_P = sum_k [ p_k^2 / 2m + (1/2) m omega_P^2 (q_k - q_{k+1})^2 + V(q_k) ],
//   omega_P = P / (beta hbar).
//
// `tower::RingPolymerOp::evaluate_energy` returns the SAME object divided by P, which is
// the energy conjugate to beta (so that `exp(-beta E_tower) = exp(-beta_P H_P)`) and NOT
// the generator of the dynamics. Both are correct for their jobs; dividing by P rescales
// time uniformly and would give the same trajectory at a different clock, which is
// exactly the sort of silent factor a commuting square exists to expose.

/// Pair force on a classical configuration: `(potential energy, forces)`, forces in
/// hartree per bohr.
pub fn pair_forces_3d(positions: &[[f64; 3]], pes: &dyn Pes) -> (f64, Vec<[f64; 3]>) {
    let n = positions.len();
    let mut f = vec![[0.0f64; 3]; n];
    let mut e = 0.0;
    for i in 0..n {
        for j in (i + 1)..n {
            let d = [
                positions[j][0] - positions[i][0],
                positions[j][1] - positions[i][1],
                positions[j][2] - positions[i][2],
            ];
            let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let (v, dv) = pes.eval(r);
            e += v;
            for a in 0..3 {
                let c = dv * d[a] / r;
                f[j][a] -= c;
                f[i][a] += c;
            }
        }
    }
    (e, f)
}

/// One velocity-Verlet step of the classical Born–Oppenheimer carrier (C0).
pub fn classical_step_3d(state: &mut ClassicalState, dt: f64, pes: &dyn Pes) {
    let n = state.positions.len();
    let (_, f0) = pair_forces_3d(&state.positions, pes);
    for i in 0..n {
        let m = state.masses[i] * M_E_PER_U;
        for a in 0..3 {
            state.velocities[i][a] += 0.5 * dt * f0[i][a] / m;
            state.positions[i][a] += dt * state.velocities[i][a];
        }
    }
    let (_, f1) = pair_forces_3d(&state.positions, pes);
    for i in 0..n {
        let m = state.masses[i] * M_E_PER_U;
        for a in 0..3 {
            state.velocities[i][a] += 0.5 * dt * f1[i][a] / m;
        }
    }
}

/// One step of NVE ring-polymer molecular dynamics on the C1 carrier: a half kick from
/// the physical potential, EXACT free-ring-polymer evolution in normal modes, and a
/// second half kick. No thermostat — RPMD's dynamics is Hamiltonian, and a thermostat
/// here would be a different claim.
///
/// The free propagator is written on VELOCITIES rather than momenta, so the mass never
/// enters it. That is not tidiness: dividing the propagated momentum back by `m` is a
/// multiply-then-divide round trip that is not the identity in f64, and it cost the
/// `P = 1` classical-limit gate its exactness — 1.05e-11 bohr of drift over 5000 steps
/// where the two integrators are the same algorithm and the answer should be bit-identical.
pub fn ring_step_3d(state: &mut RingPolymerState, dt: f64, beta: f64, pes: &dyn Pes) {
    let p = state.beads_pos.len();
    let n = state.masses.len();
    let nm = NormalModes::new(p, beta);
    let prop = FreeRingPropagator::new(&nm, dt);
    let kick = |st: &mut RingPolymerState| {
        for k in 0..p {
            let (_, f) = pair_forces_3d(&st.beads_pos[k], pes);
            for i in 0..n {
                let m = st.masses[i] * M_E_PER_U;
                for a in 0..3 {
                    st.beads_vel[k][i][a] += 0.5 * dt * f[i][a] / m;
                }
            }
        }
    };
    kick(state);
    // Exact free ring polymer, one Cartesian coordinate of one atom at a time.
    let mut q = vec![0.0f64; p];
    let mut v = vec![0.0f64; p];
    let mut qt = vec![0.0f64; p];
    let mut vt = vec![0.0f64; p];
    for i in 0..n {
        for a in 0..3 {
            for k in 0..p {
                q[k] = state.beads_pos[k][i][a];
                v[k] = state.beads_vel[k][i][a];
            }
            nm.to_modes(&q, &mut qt);
            nm.to_modes(&v, &mut vt);
            prop.apply(&nm, &mut qt, &mut vt);
            nm.from_modes(&qt, &mut q);
            nm.from_modes(&vt, &mut v);
            for k in 0..p {
                state.beads_pos[k][i][a] = q[k];
                state.beads_vel[k][i][a] = v[k];
            }
        }
    }
    kick(state);
}

/// The centroid retract, done properly: BOTH positions and velocities averaged over
/// beads.
///
/// [`crate::tower::make_c0_to_c1_transport`]'s retract averages the positions and then
/// takes bead 0's velocity, which is the centroid velocity only when every bead already
/// carries the same one — true of a freshly lifted state and false of every state a
/// trajectory reaches. The commuting square is measured against THIS map.
pub fn centroid_state(rp: &RingPolymerState) -> ClassicalState {
    let p = rp.beads_pos.len();
    let n = rp.masses.len();
    let mut positions = Vec::with_capacity(n);
    let mut velocities = Vec::with_capacity(n);
    for i in 0..n {
        let mut cp = [0.0f64; 3];
        let mut cv = [0.0f64; 3];
        for k in 0..p {
            for a in 0..3 {
                cp[a] += rp.beads_pos[k][i][a];
                cv[a] += rp.beads_vel[k][i][a];
            }
        }
        for a in 0..3 {
            cp[a] /= p as f64;
            cv[a] /= p as f64;
        }
        positions.push(cp);
        velocities.push(cv);
    }
    ClassicalState { positions, velocities, masses: rp.masses.clone() }
}

/// The measured budget of the bead-forgetting square.
///
/// The square is `centroid ∘ T_RP` against `T_C0 ∘ centroid`. It is CLOSED at `P = 1`
/// (the ring is the point and both paths are the same arithmetic) and NOT closed above
/// it, because the centroid feels the bead-AVERAGED force and the classical chart feels
/// the force at the centroid. `force_gap` is that mechanism, measured; `defect_pos` and
/// `defect_vel` are the consequence, measured.
#[derive(Clone, Debug)]
pub struct CommutingBudget {
    pub p: usize,
    pub dt: f64,
    /// `max |centroid(T_RP x) - T_C0(centroid x)|` over atoms and axes, bohr.
    pub defect_pos: f64,
    /// The same on velocities, bohr per atomic time unit.
    pub defect_vel: f64,
    /// Mean ring radius of gyration over atoms, bohr — the size of the object the
    /// centroid chart is discarding.
    pub radius_of_gyration: f64,
    /// `max |mean_k F(q_k) - F(q_centroid)|` over atoms and axes, hartree per bohr.
    pub force_gap: f64,
}

/// Measure one step of the bead-forgetting square.
pub fn commuting_budget(
    rp: &RingPolymerState,
    dt: f64,
    beta: f64,
    pes: &dyn Pes,
) -> CommutingBudget {
    let p = rp.beads_pos.len();
    let n = rp.masses.len();

    let mut up = rp.clone();
    ring_step_3d(&mut up, dt, beta, pes);
    let via_ring = centroid_state(&up);

    let mut across = centroid_state(rp);
    classical_step_3d(&mut across, dt, pes);

    let mut defect_pos = 0.0f64;
    let mut defect_vel = 0.0f64;
    for i in 0..n {
        for a in 0..3 {
            defect_pos = defect_pos.max((via_ring.positions[i][a] - across.positions[i][a]).abs());
            defect_vel = defect_vel.max((via_ring.velocities[i][a] - across.velocities[i][a]).abs());
        }
    }

    let c = centroid_state(rp);
    let (_, fc) = pair_forces_3d(&c.positions, pes);
    let mut fbar = vec![[0.0f64; 3]; n];
    for k in 0..p {
        let (_, f) = pair_forces_3d(&rp.beads_pos[k], pes);
        for i in 0..n {
            for a in 0..3 {
                fbar[i][a] += f[i][a] / p as f64;
            }
        }
    }
    let mut force_gap = 0.0f64;
    for i in 0..n {
        for a in 0..3 {
            force_gap = force_gap.max((fbar[i][a] - fc[i][a]).abs());
        }
    }

    let mut rg = 0.0f64;
    for i in 0..n {
        rg += rp.radius_of_gyration(i);
    }
    rg /= n as f64;

    CommutingBudget { p, dt, defect_pos, defect_vel, radius_of_gyration: rg, force_gap }
}

/// Total energy of a classical configuration, hartree: kinetic plus the pair potential.
pub fn classical_energy_3d(state: &ClassicalState, pes: &dyn Pes) -> f64 {
    let (v, _) = pair_forces_3d(&state.positions, pes);
    let mut k = 0.0;
    for i in 0..state.positions.len() {
        let m = state.masses[i] * M_E_PER_U;
        let s: f64 = state.velocities[i].iter().map(|x| x * x).sum();
        k += 0.5 * m * s;
    }
    k + v
}

/// The ring-polymer Hamiltonian `H_P`, hartree — the generator of [`ring_step_3d`], so
/// its drift over a trajectory is the integrator's conservation error and nothing else.
pub fn ring_energy_3d(state: &RingPolymerState, beta: f64, pes: &dyn Pes) -> f64 {
    let p = state.beads_pos.len();
    let n = state.masses.len();
    let omega_p = (p as f64) / beta;
    let mut e = 0.0;
    for k in 0..p {
        let (v, _) = pair_forces_3d(&state.beads_pos[k], pes);
        e += v;
        for i in 0..n {
            let m = state.masses[i] * M_E_PER_U;
            let s: f64 = state.beads_vel[k][i].iter().map(|x| x * x).sum();
            e += 0.5 * m * s;
            let kn = (k + 1) % p;
            let mut d2 = 0.0;
            for a in 0..3 {
                let d = state.beads_pos[kn][i][a] - state.beads_pos[k][i][a];
                d2 += d * d;
            }
            e += 0.5 * m * omega_p * omega_p * d2;
        }
    }
    e
}
