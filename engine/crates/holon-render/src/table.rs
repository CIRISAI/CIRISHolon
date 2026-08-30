//! The exact pair potential, as a piecewise cubic Hermite table.
//!
//! # The contract
//!
//! The curve arrives as `h2_potential.json` with the fields
//! `R_grid_bohr` / `E_hartree` / `F_hartree_per_bohr` / `R_e` / `D_e` / `E_asymptote`,
//! in Hartree atomic units (length bohr, energy hartree, force hartree/bohr). This
//! module never parses that file: the loader (JS in the browser, `json.rs` natively)
//! pushes the knots through `begin` / `knot` / `finish`, so exactly one interpolator
//! serves both, and the file can be replaced without touching any code.
//!
//! # Sign convention, and why it is checked rather than assumed
//!
//! `F` is the FORCE, so `dE/dR = -F`. That is an assumption about someone else's file,
//! and a silent sign error there would produce a curve that still conserves energy
//! perfectly (see below) while being the wrong curve — the failure mode that no
//! conservation gate can see. `finish` therefore measures how well the supplied
//! derivatives reproduce the supplied values, under BOTH sign hypotheses, and reports
//! the pair; `residual` should be near zero and `residual_alt` should be large. If they
//! swap, the file uses the other convention and the UI says so instead of quietly
//! simulating a mirror-image molecule.
//!
//! # Why the ledger closes even if the table is inconsistent
//!
//! The Hermite interpolant `H(R)` built from the knots IS the potential, by definition.
//! Forces are `-H'(R)`, differentiated analytically from the same coefficients, so the
//! force is the exact negative gradient of the energy the ledger sums no matter what
//! the table's own `E`/`F` consistency looks like. Table quality and energy
//! conservation are therefore independent questions, and they get independent gates.

/// Upper bound on knot count. A fixed array keeps the crate allocation-free, which is
/// what keeps the wasm small and the physics core free of an allocator.
pub const MAX_KNOTS: usize = 1024;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoadStatus {
    Empty,
    Ok,
    TooManyKnots,
    TooFewKnots,
    NotIncreasing,
    NotFinite,
}

/// Piecewise cubic Hermite interpolant on a (possibly non-uniform) knot grid.
///
/// C1 by construction: each interval reproduces the value and the derivative at both of
/// its ends, so value and slope agree across every knot.
pub struct PotentialTable {
    r: [f64; MAX_KNOTS],
    /// Energy at the knot, ASYMPTOTE-ZEROED (`E_file - E_asymptote`).
    ///
    /// Storing the offset rather than the file's absolute energy is not a convenience:
    /// electronic-structure tables quote absolute energies of order 1 hartree while the
    /// interaction that matters out in the tail is of order 1e-7, so forming `U` as
    /// `(E_asymptote + u) - E_asymptote` at evaluation time loses everything below
    /// `eps * |E_asymptote|`. Measured on the placeholder before this changed: the force
    /// disagreed with `-dU/dR` by 2.9e-4 RELATIVE beyond 14 bohr, pure cancellation
    /// noise, injected straight into the ledger the gates then have to absorb. Zeroing
    /// once at load time removes it at the source.
    e: [f64; MAX_KNOTS],
    /// dE/dR at the knot. Stored as the DERIVATIVE, converted from the file's force on
    /// the way in, so every formula below reads in one convention only.
    d: [f64; MAX_KNOTS],
    /// d2E/dR2 at the knot, when the contract supplies it (`d2E_hartree_per_bohr2`).
    /// OPTIONAL by design: the curvature envelope that sets the drift bound is exact
    /// from the interpolant alone (see `curvature_envelope`), so a file without this
    /// column loses nothing. When it IS present it is used as a CHECK on the
    /// interpolant, never as a substitute for it -- the forces come from the Hermite
    /// coefficients, so the bound must describe the function actually being integrated.
    d2: [f64; MAX_KNOTS],
    has_d2: bool,
    /// Worst relative disagreement between a supplied d2 column and the interpolant's
    /// own second derivative at the knots. Reported, not enforced: cubic Hermite is C1,
    /// so its curvature is NOT continuous at knots and a mismatch here is expected
    /// structure rather than an error.
    pub d2_mismatch: f64,
    n: usize,
    filling: usize,
    pub status: LoadStatus,

    pub r_e: f64,
    pub d_e: f64,
    pub e_asymptote: f64,

    /// RMS mismatch between the supplied derivatives and the secant slopes of the
    /// supplied values, relative to the RMS secant slope. Assumed convention.
    pub residual: f64,
    /// The same statistic under the opposite sign hypothesis (`dE/dR = +F`).
    pub residual_alt: f64,

    /// Short-range extrapolation, matched C1 at the first knot: `E = e_asymptote + a*exp(-b*(R-r0))`.
    lo_a: f64,
    lo_b: f64,
    lo_linear: bool,
    /// Long-range extrapolation, matched C1 at the last knot, same functional form.
    hi_a: f64,
    hi_b: f64,
    hi_linear: bool,
}

impl PotentialTable {
    pub const fn empty() -> Self {
        Self {
            r: [0.0; MAX_KNOTS],
            e: [0.0; MAX_KNOTS],
            d: [0.0; MAX_KNOTS],
            d2: [0.0; MAX_KNOTS],
            has_d2: false,
            d2_mismatch: 0.0,
            n: 0,
            filling: 0,
            status: LoadStatus::Empty,
            r_e: 0.0,
            d_e: 0.0,
            e_asymptote: 0.0,
            residual: 0.0,
            residual_alt: 0.0,
            lo_a: 0.0,
            lo_b: 0.0,
            // FALSE, and the value is never read — see below. It is `false` rather than
            // the `true` it used to be so that `empty()` is ALL ZEROS, which is what lets
            // a static array of these live in zero-initialised memory instead of in a
            // data segment.
            //
            // Measured: with two `true` bytes per table, a ten-table `PairBank` inside the
            // static `Sim` cost the browser artifact 330,206 bytes — 253 KB to 610 KB —
            // because the linker emitted the whole 329 KB region as initialised data
            // rather than the twenty bytes that were actually non-zero. Zeroing them
            // returns the artifact to its size.
            //
            // Never read: `eval` returns early when `!is_loaded()`, and `finish` calls
            // `build_extrapolations`, which assigns both flags unconditionally. So every
            // table that reaches a branch testing them has set them itself.
            lo_linear: false,
            hi_a: 0.0,
            hi_b: 0.0,
            hi_linear: false,
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.status == LoadStatus::Ok && self.n >= 2
    }

    pub fn knots(&self) -> usize {
        self.n
    }

    /// Knot accessors. The viewer draws the curve from `eval`, but the gates need the
    /// raw knot data to check that the interpolant actually reproduces it.
    pub fn knot_r(&self, i: usize) -> f64 {
        if i < self.n {
            self.r[i]
        } else {
            0.0
        }
    }

    /// Knot energy, asymptote-zeroed (so `knot_e` is `U`, not the file's absolute `E`).
    pub fn knot_u(&self, i: usize) -> f64 {
        if i < self.n {
            self.e[i]
        } else {
            0.0
        }
    }

    /// dU/dR at the knot (the NEGATIVE of the file's force; the constant shift does not
    /// change the derivative).
    pub fn knot_d(&self, i: usize) -> f64 {
        if i < self.n {
            self.d[i]
        } else {
            0.0
        }
    }

    pub fn r_min(&self) -> f64 {
        if self.n == 0 {
            0.0
        } else {
            self.r[0]
        }
    }

    pub fn r_max(&self) -> f64 {
        if self.n == 0 {
            0.0
        } else {
            self.r[self.n - 1]
        }
    }

    pub fn begin(&mut self, count: usize) -> bool {
        if count > MAX_KNOTS {
            self.status = LoadStatus::TooManyKnots;
            return false;
        }
        if count < 2 {
            self.status = LoadStatus::TooFewKnots;
            return false;
        }
        self.n = 0;
        self.filling = count;
        self.has_d2 = false;
        self.d2_mismatch = 0.0;
        self.status = LoadStatus::Empty;
        true
    }

    /// Push the optional second-derivative column for a knot already pushed by `knot`.
    /// Values arrive as d2E/dR2 (the file's `d2E_hartree_per_bohr2`); a constant shift
    /// of E does not change it, so no asymptote correction applies.
    pub fn knot_curvature(&mut self, index: usize, d2: f64) -> bool {
        if index >= self.filling || !d2.is_finite() {
            return false;
        }
        self.d2[index] = d2;
        self.has_d2 = true;
        true
    }

    pub fn has_supplied_curvature(&self) -> bool {
        self.has_d2
    }

    /// Push one knot. `force` is the file's `F_hartree_per_bohr`; the derivative stored
    /// is `-force` (see the module header on the sign convention).
    pub fn knot(&mut self, index: usize, r: f64, e: f64, force: f64) -> bool {
        if index >= self.filling {
            return false;
        }
        if !r.is_finite() || !e.is_finite() || !force.is_finite() {
            self.status = LoadStatus::NotFinite;
            return false;
        }
        self.r[index] = r;
        self.e[index] = e;
        self.d[index] = -force;
        if index + 1 > self.n {
            self.n = index + 1;
        }
        true
    }

    pub fn finish(&mut self, r_e: f64, d_e: f64, e_asymptote: f64) -> LoadStatus {
        if self.n != self.filling || self.n < 2 {
            self.status = LoadStatus::TooFewKnots;
            return self.status;
        }
        for i in 1..self.n {
            // The negation is deliberate, not a stylistic slip: `!(a > b)` rejects a NaN
            // grid point as well as a non-increasing one, where `a <= b` would silently
            // accept NaN and hand it to the interval search.
            #[allow(clippy::neg_cmp_op_on_partial_ord)]
            if !(self.r[i] > self.r[i - 1]) {
                self.status = LoadStatus::NotIncreasing;
                return self.status;
            }
        }
        if !r_e.is_finite() || !d_e.is_finite() || !e_asymptote.is_finite() {
            self.status = LoadStatus::NotFinite;
            return self.status;
        }
        self.r_e = r_e;
        self.d_e = d_e;
        self.e_asymptote = e_asymptote;

        // Data-quality gate. On each interval compare the secant slope of the values
        // with the mean of the two endpoint derivatives; both are estimates of the same
        // slope, so they agree for a self-consistent table. `residual_alt` repeats the
        // comparison with the derivatives negated, which is what the table would look
        // like had the file meant `dE/dR = +F`.
        let mut num = 0.0f64;
        let mut num_alt = 0.0f64;
        let mut den = 0.0f64;
        for i in 1..self.n {
            let h = self.r[i] - self.r[i - 1];
            let secant = (self.e[i] - self.e[i - 1]) / h;
            let mean_d = 0.5 * (self.d[i] + self.d[i - 1]);
            num += (secant - mean_d) * (secant - mean_d);
            num_alt += (secant + mean_d) * (secant + mean_d);
            den += secant * secant;
        }
        let den = if den > 0.0 { den } else { 1.0 };
        self.residual = (num / den).sqrt();
        self.residual_alt = (num_alt / den).sqrt();

        // Shift to the asymptote-zeroed convention. A constant shift changes neither the
        // secants nor the derivatives, so the residuals just computed are unaffected.
        for i in 0..self.n {
            self.e[i] -= e_asymptote;
        }

        self.build_extrapolations();
        self.status = LoadStatus::Ok;

        if self.has_d2 {
            let mut worst: f64 = 0.0;
            let mut scale: f64 = 0.0;
            for i in 0..self.n {
                scale = scale.max(self.d2[i].abs());
            }
            let scale = if scale > 0.0 { scale } else { 1.0 };
            for i in 0..self.n {
                let mine = self.eval(self.r[i]).2;
                worst = worst.max((mine - self.d2[i]).abs() / scale);
            }
            self.d2_mismatch = worst;
        }
        self.status
    }

    /// Match `U = a*exp(-b*(R - r_edge))` in value and slope at each end
    /// knot. This is C1 with the interpolant and, being an explicit analytic function,
    /// is differentiated exactly like the interior — so extrapolated regions conserve
    /// energy on the same footing as the table's own span.
    ///
    /// The exponential form needs `a` and `b` to come out positive-definite in the
    /// right sense; where the end knot does not supply that (a repulsive wall whose
    /// first knot already sits below the asymptote, say) it degenerates to a
    /// constant-force linear ramp, which is C1 too and can never blow up.
    fn build_extrapolations(&mut self) {
        let n = self.n;

        let a0 = self.e[0];
        let d0 = self.d[0];
        // Decaying-outward toward the asymptote as R grows means a0 and d0 must have
        // opposite signs; b > 0 is then automatic.
        if a0.abs() > 0.0 && (-d0 / a0) > 0.0 && (-d0 / a0).is_finite() {
            self.lo_a = a0;
            self.lo_b = -d0 / a0;
            self.lo_linear = false;
        } else {
            self.lo_linear = true;
        }

        let an = self.e[n - 1];
        let dn = self.d[n - 1];
        if an.abs() > 0.0 && (-dn / an) > 0.0 && (-dn / an).is_finite() {
            self.hi_a = an;
            self.hi_b = -dn / an;
            self.hi_linear = false;
        } else {
            self.hi_linear = true;
        }
    }

    fn interval(&self, r: f64) -> usize {
        // Binary search for the interval containing r; callers guarantee r is inside.
        let mut lo = 0usize;
        let mut hi = self.n - 1;
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if r < self.r[mid] {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        lo
    }

    /// Energy, its derivative, and its second derivative at `r`, all from one analytic
    /// expression per region. Returns `(U, dU/dR, d2U/dR2)` with the asymptote as the
    /// zero — the convention the ledger and the bond predicate both want, and the one
    /// that keeps the tail free of cancellation noise (see the `e` field).
    pub fn eval(&self, r: f64) -> (f64, f64, f64) {
        if !self.is_loaded() {
            return (0.0, 0.0, 0.0);
        }
        let n = self.n;
        if r <= self.r[0] {
            let dr = r - self.r[0];
            if self.lo_linear {
                return (self.e[0] + self.d[0] * dr, self.d[0], 0.0);
            }
            let ex = (-self.lo_b * dr).exp();
            let v = self.lo_a * ex;
            return (v, -self.lo_b * v, self.lo_b * self.lo_b * v);
        }
        if r >= self.r[n - 1] {
            let dr = r - self.r[n - 1];
            if self.hi_linear {
                return (self.e[n - 1] + self.d[n - 1] * dr, self.d[n - 1], 0.0);
            }
            let ex = (-self.hi_b * dr).exp();
            let v = self.hi_a * ex;
            return (v, -self.hi_b * v, self.hi_b * self.hi_b * v);
        }

        let i = self.interval(r);
        let h = self.r[i + 1] - self.r[i];
        let t = (r - self.r[i]) / h;
        let t2 = t * t;
        let t3 = t2 * t;

        let (e0, e1) = (self.e[i], self.e[i + 1]);
        let (d0, d1) = (self.d[i], self.d[i + 1]);

        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;
        let value = h00 * e0 + h10 * h * d0 + h01 * e1 + h11 * h * d1;

        let g00 = 6.0 * t2 - 6.0 * t;
        let g10 = 3.0 * t2 - 4.0 * t + 1.0;
        let g01 = -6.0 * t2 + 6.0 * t;
        let g11 = 3.0 * t2 - 2.0 * t;
        let slope = (g00 * e0 + g01 * e1) / h + g10 * d0 + g11 * d1;

        let c00 = 12.0 * t - 6.0;
        let c10 = 6.0 * t - 4.0;
        let c01 = -12.0 * t + 6.0;
        let c11 = 6.0 * t - 2.0;
        let curv = (c00 * e0 + c01 * e1) / (h * h) + (c10 * d0 + c11 * d1) / h;

        (value, slope, curv)
    }

    /// Pair potential with the asymptote as the zero: two atoms infinitely far apart
    /// contribute EXACTLY zero to the ledger, so the total energy of a dissociated
    /// scene does not depend on how many non-interacting pairs it happens to contain.
    /// This is also what makes `U < 0` mean "bound" without any further convention.
    pub fn u(&self, r: f64) -> f64 {
        self.eval(r).0
    }

    /// Magnitude of the central force. Positive = repulsive (pushes the pair apart).
    pub fn force(&self, r: f64) -> f64 {
        -self.eval(r).1
    }

    pub fn curvature(&self, r: f64) -> f64 {
        self.eval(r).2
    }

    /// Innermost separation reachable by a pair whose relative energy is `e_rel`.
    ///
    /// On the repulsive branch `U` is steeply decreasing in R, so `U(R) = e_rel` has a
    /// unique inner root; bisection on `[r_probe, R_e]`. A pair cannot get closer than
    /// this without more energy than it has, which is what makes the envelope below a
    /// bound rather than a sample.
    pub fn inner_turning_point(&self, e_rel: f64) -> f64 {
        if !self.is_loaded() {
            return self.r_min();
        }
        let hi = self.r_e.max(self.r_min());
        // Walk inward until the wall is above e_rel, then bisect. The extrapolated wall
        // is exponential, so this terminates quickly; the floor keeps a pathological
        // table (or an enormous energy) from looping.
        let mut lo = hi;
        for _ in 0..200 {
            let next = lo * 0.9;
            if next < 1e-4 {
                return next;
            }
            if self.u(next) > e_rel {
                lo = next;
                break;
            }
            lo = next;
        }
        let mut a = lo;
        let mut b = hi;
        for _ in 0..80 {
            let mid = 0.5 * (a + b);
            if self.u(mid) > e_rel {
                a = mid;
            } else {
                b = mid;
            }
        }
        0.5 * (a + b)
    }

    /// The CURVATURE ENVELOPE: the largest `|d2U/dR2|` anywhere a pair with relative
    /// energy up to `e_rel_max` can reach.
    ///
    /// This is fence 3, and it is why the displayed drift bound stays valid THROUGH a
    /// collision. `U''(R_e)` describes the bottom of the well; the repulsive wall is far
    /// stiffer, and a bound derived from the equilibrium curvature alone reads green
    /// right through the encounter that violates it.
    ///
    /// The scan is EXACT rather than sampled, and that is a property of the interpolant:
    /// on each interval the Hermite polynomial is a cubic, so `U''` is LINEAR in the
    /// interval parameter and its extremes over that interval are attained at the
    /// endpoints. Scanning the knots that lie in range, plus the range's own endpoints,
    /// therefore finds the true maximum -- no sampling density to argue about. In the
    /// extrapolated regions `U'' = b^2 * a * exp(-b*(R - r_edge))` is monotone, so the
    /// inner edge of the range dominates there too.
    pub fn curvature_envelope(&self, e_rel_max: f64) -> (f64, f64) {
        if !self.is_loaded() {
            return (0.0, 0.0);
        }
        let r_inner = self.inner_turning_point(e_rel_max);
        let mut worst = self.eval(r_inner).2.abs();
        for i in 0..self.n {
            if self.r[i] >= r_inner {
                worst = worst.max(self.eval(self.r[i]).2.abs());
                // C1, not C2: the curvature jumps at a knot, so both one-sided values
                // count. Approaching from the left samples the previous interval's
                // polynomial, which the loop above never evaluates at this knot.
                if i > 0 {
                    worst = worst.max(self.eval(self.r[i] - 1e-9).2.abs());
                }
            }
        }
        (worst, r_inner)
    }

    /// Outer classical turning point of the effective radial potential
    /// `U_eff(R) = U(R) + L^2 / (2 mu R^2)` at relative energy `e_rel`.
    ///
    /// Returns `f64::INFINITY` when no turning point is found out to `r_cap` (an
    /// unbound pair, or one so marginally bound that its outer turning point is beyond
    /// any distance the scene can express).
    pub fn outer_turning_point(
        &self,
        e_rel: f64,
        l_sq: f64,
        mu: f64,
        r_start: f64,
        r_cap: f64,
    ) -> f64 {
        let u_eff = |r: f64| self.u(r) + l_sq / (2.0 * mu * r * r);
        if e_rel >= 0.0 {
            return f64::INFINITY;
        }
        let mut lo = r_start.max(self.r_min());
        if u_eff(lo) > e_rel {
            // Already outside: the state is not on a radial branch that reaches here.
            return lo;
        }
        let mut hi = lo * 2.0;
        let mut found = false;
        for _ in 0..64 {
            if hi > r_cap {
                break;
            }
            if u_eff(hi) > e_rel {
                found = true;
                break;
            }
            lo = hi;
            hi *= 1.5;
        }
        if !found {
            return f64::INFINITY;
        }
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if u_eff(mid) > e_rel {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        0.5 * (lo + hi)
    }
}
