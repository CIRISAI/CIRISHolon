//! THE BAROSTAT: isotropic MTK (Martyna–Tobias–Klein) NPT, as extended-system dynamics.
//!
//! FSD-W1 **WB-2.2**: "the pressure control IS the barostat: at molecular tiers the box
//! rescales under the set pressure (NPT)". WB-7.2 names what the mock shell did instead —
//! `P^−0.05` box scaling — and calls it placeholder. This module is the replacement, and
//! the difference is not cosmetic.
//!
//! # Why a rescale is not a barostat
//!
//! A box-rescale hack multiplies every coordinate by a factor derived from the pressure
//! error. It moves the volume in the right direction and it is not dynamics: there is no
//! equation of motion for the volume, so there is no conserved quantity, no defined
//! ensemble, and nothing to check. The volume fluctuations it produces are whatever the
//! feedback constant makes them, not the ones the NPT ensemble prescribes — so any
//! observable that depends on those fluctuations (compressibility, most obviously) is
//! wrong by an amount nobody can compute.
//!
//! MTK adds the volume to the system as a genuine degree of freedom. With
//! `ε = ⅓ ln(V/V₀)` and its conjugate momentum `p_ε` of mass `W`:
//!
//! ```text
//! ε̇  = p_ε / W
//! ṗ_ε = 3V(P_int − P_ext) + (3/N_f)·Σ p²/m        [the MTK correction term]
//! ṙ_i = p_i/m_i + (p_ε/W)·r_i
//! ṗ_i = F_i − (1 + 3/N_f)(p_ε/W)·p_i
//! ```
//!
//! and the extended system conserves
//!
//! ```text
//! H' = H + p_ε²/2W + P_ext·V + Σ_chains (p_ξ²/2Q + N k T ξ)
//! ```
//!
//! which is a NUMBER THIS MODULE REPORTS ([`Sim::h_prime`]) and a gate checks the drift of.
//! That is the whole reason to prefer MTK over the hack: it produces something falsifiable.
//!
//! The `3/N_f` term is MTK's correction to the earlier Andersen/Hoover form and it is what
//! makes the sampled distribution the correct NPT one at finite N rather than only as
//! N → ∞. It is small and it is easy to drop; it is written out below with its name on it.
//!
//! # The thermostat is not optional here
//!
//! NPT is a constant-TEMPERATURE ensemble as well as a constant-pressure one, and the
//! barostat needs a thermostat for a reason beyond the ensemble: `p_ε` is a degree of
//! freedom that exchanges energy with the particles, and left unthermostatted it rings.
//! MTK's prescription is Nosé–Hoover CHAINS — plural, because a single Nosé–Hoover
//! variable is famously non-ergodic on stiff systems (the harmonic oscillator is the
//! standard counterexample) — on the particles and a SEPARATE chain on the barostat.
//! Both are here, at length 3.
//!
//! # ARTIFACTS, documented as WB-2.2 requires rather than discovered later
//!
//! 1. **The trajectory is not Newtonian.** Under NPT the particles move on the extended
//!    system's trajectory, not the physical one. Dynamical quantities — diffusion
//!    coefficients, spectra, correlation times — are perturbed by the coupling. Measure
//!    them in NVE, from a configuration NPT produced.
//! 2. **`τ_p` is a real parameter and it has a wrong end.** Too small and the box rings at
//!    the barostat frequency, contaminating any observable with that period; too large and
//!    the volume takes longer to equilibrate than the run. It is exposed, defaulted to
//!    1000 a.u. (~24 fs), and NOT fitted.
//! 3. **Isotropic only.** One `ε`, so the box stays the shape it started. A crystal that
//!    wants to change symmetry cannot, and its stress will read anisotropic while the box
//!    refuses to follow. That is a REFUSAL to model a thing rather than a wrong model of
//!    it, and the ice-polymorph ladder (WB-2.2) will need the full Parrinello–Rahman cell
//!    before it can be believed. Named here so nobody reads a polymorph result off an
//!    isotropic box.
//! 4. **The pressure is the internal virial only**, so the box must be periodic. Under
//!    walls the container carries momentum flux the virial does not see, and the barostat
//!    refuses rather than controlling a number that is not the pressure.
//! 5. **A truncated pair potential truncates the virial too.** With a declared pair cutoff
//!    the long-range tail's contribution to the pressure is dropped along with its
//!    contribution to the energy, and the dropped part is systematically NEGATIVE (the tail
//!    is attractive). No tail correction is applied. The size is the truncation budget's,
//!    per pair, and `Sim::truncation_floor` reports it.
//!
//! # Integration, and how it meets the energy ledger
//!
//! The Trotter factorization is the standard MTK one, symmetric about the force
//! evaluation. The barostat and the chains move the PHYSICAL energy — they rescale
//! velocities and they move the box under the atoms — and every hartree they move is posted
//! to `Sim::work.barostat`, exactly as the Berendsen thermostat posts to
//! `Sim::work.thermostat`. So `E − W_ext` is still the constant the energy gate checks, and
//! the extended `H'` is a SECOND, independent conserved quantity with its own gate. One
//! gate per conservation law, and NPT brings its own law.

use crate::sim::{Boundary, Sim, K_B};

/// Nosé–Hoover chain length. Three is the standard choice and the reason is ergodicity, not
/// accuracy: one variable fails on a harmonic system (Hoover's own counterexample), two
/// fixes the textbook cases, three is cheap insurance. Longer chains cost one more
/// thermostat update each and buy progressively nothing.
pub const CHAIN_LENGTH: usize = 3;

/// Suzuki–Yoshida weights for the order-3 decomposition of the chain propagator.
///
/// The chain equations are stiff — `Q` for the outer links is small — so the chain is
/// integrated with a higher-order splitting inside each half-step rather than with the same
/// `dt` as the particles. These are the standard weights; a single Euler step here is the
/// usual place an MTK implementation quietly loses its conserved quantity.
const SY_WEIGHTS: [f64; 3] = [
    1.351_207_191_959_657_6,
    -1.702_414_383_919_315_2,
    1.351_207_191_959_657_6,
];

/// One Nosé–Hoover chain.
#[derive(Clone, Debug)]
pub struct Chain {
    /// The thermostat coordinates. Only their SUM enters `H'`; they are carried
    /// individually because each link's own coordinate is what its conserved term needs.
    pub xi: [f64; CHAIN_LENGTH],
    /// The conjugate momenta.
    pub p_xi: [f64; CHAIN_LENGTH],
    /// The masses. `Q[0]` couples to `N_f` degrees of freedom, the rest to one each.
    pub q: [f64; CHAIN_LENGTH],
    /// Degrees of freedom the first link thermostats.
    pub n_f: f64,
}

impl Chain {
    /// A chain sized for `n_f` degrees of freedom at temperature `t` with time constant
    /// `tau`. The masses are `Q₀ = N_f k T τ²` and `Qᵢ = k T τ²` — the standard
    /// prescription, and the only place `τ` enters.
    pub fn new(n_f: f64, t: f64, tau: f64) -> Chain {
        let kt = K_B * t;
        let mut q = [kt * tau * tau; CHAIN_LENGTH];
        q[0] = n_f * kt * tau * tau;
        Chain {
            xi: [0.0; CHAIN_LENGTH],
            p_xi: [0.0; CHAIN_LENGTH],
            q,
            n_f,
        }
    }

    /// The chain's contribution to the conserved quantity `H'`.
    pub fn energy(&self, t: f64) -> f64 {
        let kt = K_B * t;
        let mut e = 0.0;
        for k in 0..CHAIN_LENGTH {
            e += 0.5 * self.p_xi[k] * self.p_xi[k] / self.q[k];
        }
        // The first link carries `N_f k T ξ₀`; the rest carry `k T ξᵢ`.
        e += self.n_f * kt * self.xi[0];
        for k in 1..CHAIN_LENGTH {
            e += kt * self.xi[k];
        }
        e
    }

    /// Advance the chain by `dt` against a kinetic energy of `two_ke = Σ p²/m`, returning
    /// the factor by which the thermostatted momenta must be scaled.
    ///
    /// The chain does not touch the momenta itself — it returns the scale and the caller
    /// applies it, because the caller is the one that knows which momenta and the one that
    /// has to post the energy change to the ledger.
    pub fn advance(&mut self, dt: f64, two_ke: f64, t_target: f64) -> f64 {
        let kt = K_B * t_target;
        let mut scale = 1.0;
        let mut two_ke = two_ke;
        for &w in SY_WEIGHTS.iter() {
            let d = w * dt / SY_WEIGHTS.len() as f64;
            let (d2, d4, d8) = (d * 0.5, d * 0.25, d * 0.125);

            // The forces on the chain, outermost link first.
            let mut g = [0.0f64; CHAIN_LENGTH];
            g[CHAIN_LENGTH - 1] = (self.p_xi[CHAIN_LENGTH - 2] * self.p_xi[CHAIN_LENGTH - 2]
                / self.q[CHAIN_LENGTH - 2]
                - kt)
                / self.q[CHAIN_LENGTH - 1];
            for k in (1..CHAIN_LENGTH - 1).rev() {
                g[k] = (self.p_xi[k - 1] * self.p_xi[k - 1] / self.q[k - 1] - kt) / self.q[k];
            }
            g[0] = (two_ke - self.n_f * kt) / self.q[0];

            // Outward half-kick, innermost last (the standard nested update).
            self.p_xi[CHAIN_LENGTH - 1] += d4 * g[CHAIN_LENGTH - 1] * self.q[CHAIN_LENGTH - 1];
            for k in (0..CHAIN_LENGTH - 1).rev() {
                let damp = (-d8 * self.p_xi[k + 1] / self.q[k + 1]).exp();
                self.p_xi[k] = self.p_xi[k] * damp * damp + d4 * g[k] * self.q[k] * damp;
            }

            // The particle scaling, and the coordinates.
            let s = (-d2 * self.p_xi[0] / self.q[0]).exp();
            scale *= s;
            two_ke *= s * s;
            for k in 0..CHAIN_LENGTH {
                self.xi[k] += d2 * self.p_xi[k] / self.q[k];
            }

            // Inward half-kick, mirroring the first.
            g[0] = (two_ke - self.n_f * kt) / self.q[0];
            for k in 0..CHAIN_LENGTH - 1 {
                let damp = (-d8 * self.p_xi[k + 1] / self.q[k + 1]).exp();
                self.p_xi[k] = self.p_xi[k] * damp * damp + d4 * g[k] * self.q[k] * damp;
                g[k + 1] = (self.p_xi[k] * self.p_xi[k] / self.q[k] - kt) / self.q[k + 1];
            }
            self.p_xi[CHAIN_LENGTH - 1] += d4 * g[CHAIN_LENGTH - 1] * self.q[CHAIN_LENGTH - 1];
        }
        scale
    }
}

/// Why the barostat refused to run.
/// Why a manual box scale was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleRefusal {
    /// The factor is not a positive finite number.
    BadFactor,
    /// The scaled box would collapse below twice the wall inset on some axis.
    CollapsesBox,
    /// The scaled box would put the pair list's cutoff past half the shortest edge of a
    /// wrapping boundary — the [`crate::sim::Sim::pbc_ok`] condition — so an atom would
    /// sit inside the cutoff of two images of the same partner and the reduction would
    /// silently drop one force. Found by B2's G9: `scale_box` could walk a scene out of
    /// the configuration it was admitted in, with no complaint anywhere. The door now
    /// opens only onto legal states; the numbers behind the refusal are
    /// [`crate::sim::Sim::pbc_margin`].
    BreaksPeriodicImages,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BarostatRefusal {
    /// The box has walls, so the internal virial is not the pressure.
    NotPeriodic,
    /// Fewer than two atoms: there is no virial and no meaningful pressure.
    TooFewAtoms,
    /// The box has no volume.
    DegenerateBox,
}

impl BarostatRefusal {
    pub fn plain(self) -> &'static str {
        match self {
            BarostatRefusal::NotPeriodic => {
                "a barostat needs a periodic box: with walls the container carries momentum \
                 flux that the internal virial cannot see, so the number being controlled \
                 would not be the pressure"
            }
            BarostatRefusal::TooFewAtoms => {
                "fewer than two atoms have no interactions and therefore no virial; there is \
                 nothing here for a barostat to control"
            }
            BarostatRefusal::DegenerateBox => "the box has no volume to change",
        }
    }
}

/// The isotropic MTK barostat's own state.
#[derive(Clone, Debug)]
pub struct Barostat {
    pub enabled: bool,
    /// Target pressure, hartree/bohr³. One atmosphere is 3.3989e-9.
    pub target_pressure: f64,
    /// Target temperature, kelvin. NPT controls both; a barostat without a thermostat is
    /// not an ensemble.
    pub target_temperature: f64,
    /// Barostat time constant, atomic units. See artifact (2) in the module header.
    pub tau_p: f64,
    /// Thermostat time constant, atomic units.
    pub tau_t: f64,
    /// `ε = ⅓ ln(V/V₀)`, the strain variable. Carried for the record; the box itself is the
    /// state of record and `ε` is re-derived from it on every use, so the two cannot drift.
    pub v0: f64,
    /// `p_ε`, the strain momentum.
    pub p_eps: f64,
    /// `W`, the barostat mass. Derived from `τ_p`, never tuned: `W = (N_f + 3) k T τ_p²`.
    pub w: f64,
    /// Nosé–Hoover chain on the particles.
    pub particles: Chain,
    /// A SEPARATE chain on the barostat, as MTK prescribes. Sharing one chain between the
    /// particles and the barostat is a common simplification and it is wrong: the two have
    /// different characteristic frequencies, and one chain tuned for either is badly tuned
    /// for the other.
    pub strain: Chain,
}

/// The atomic unit of pressure, pascals: `E_h / a₀³`.
///
/// CODATA 2018. Written out as its own named constant rather than folded into [`ONE_ATM`]
/// because it is a MEASURED conversion and the place it came from should be visible — the
/// first version of this file carried `2.9421912e13`, which is wrong in the sixth digit
/// (3.0e-5 relative), and a wrong unit constant is wrong in every number derived from it
/// while every gate around it still passes.
pub const AU_PRESSURE_PA: f64 = 2.942_101_569_7e13;

/// One standard atmosphere (101325 Pa exactly, by definition) in hartree per cubic bohr.
pub const ONE_ATM: f64 = 101_325.0 / AU_PRESSURE_PA;

impl Barostat {
    /// A barostat for `n_f` translational degrees of freedom at the given target state.
    pub fn new(n_f: f64, pressure: f64, temperature: f64, tau_p: f64, tau_t: f64) -> Barostat {
        let kt = K_B * temperature;
        Barostat {
            enabled: false,
            target_pressure: pressure,
            target_temperature: temperature,
            tau_p,
            tau_t,
            v0: 1.0,
            p_eps: 0.0,
            // MTK's own prescription. `+3` because the barostat's own degree of freedom is
            // one of the ones being carried.
            w: (n_f + 3.0) * kt * tau_p * tau_p,
            particles: Chain::new(n_f, temperature, tau_t),
            strain: Chain::new(1.0, temperature, tau_t),
        }
    }

    /// The barostat's own contribution to `H'`: the strain kinetic energy, the `PV` term,
    /// and both chains.
    pub fn energy(&self, volume: f64) -> f64 {
        0.5 * self.p_eps * self.p_eps / self.w
            + self.target_pressure * volume
            + self.particles.energy(self.target_temperature)
            + self.strain.energy(self.target_temperature)
    }
}

impl Sim {
    /// Turn the barostat on at a target pressure and temperature, or say why not.
    ///
    /// The refusals are the point. A barostat on a walled box would control the internal
    /// virial and call it pressure, which is a number that moves in roughly the right
    /// direction and is not the quantity named — the shape WB-7.1 exists to forbid.
    pub fn enable_barostat(
        &mut self,
        pressure: f64,
        temperature: f64,
    ) -> Result<(), BarostatRefusal> {
        if self.boundary != Boundary::Periodic {
            return Err(BarostatRefusal::NotPeriodic);
        }
        if self.n < 2 {
            return Err(BarostatRefusal::TooFewAtoms);
        }
        if !(self.volume() > 0.0) {
            return Err(BarostatRefusal::DegenerateBox);
        }
        let n_f = self.dims.dof() * self.n as f64;
        let tau_p = self
            .barostat
            .as_ref()
            .map(|b| b.tau_p)
            .unwrap_or(DEFAULT_TAU_P);
        let tau_t = self
            .barostat
            .as_ref()
            .map(|b| b.tau_t)
            .unwrap_or(DEFAULT_TAU_T);
        let mut b = Barostat::new(n_f, pressure, temperature, tau_p, tau_t);
        b.enabled = true;
        b.v0 = self.volume();
        self.barostat = Some(Box::new(b));
        // The Berendsen thermostat and the MTK chains are two thermostats, and running both
        // would control the temperature twice with two different ensembles. NPT owns the
        // temperature while it is on, and says so here rather than letting the two fight.
        self.thermostat_on = false;
        Ok(())
    }

    pub fn disable_barostat(&mut self) {
        if let Some(b) = self.barostat.as_mut() {
            b.enabled = false;
        }
    }

    pub fn barostat_on(&self) -> bool {
        self.barostat.as_ref().map(|b| b.enabled).unwrap_or(false)
    }

    /// THE EXTENDED CONSERVED QUANTITY.
    ///
    /// `H' = E + p_ε²/2W + P_ext·V + chains`. Constant along an exact MTK trajectory, and
    /// therefore the gate that says whether the integration is doing what it claims —
    /// independently of the ordinary energy ledger, which the barostat legitimately moves.
    ///
    /// Equal to the plain energy when the barostat is off, so a caller can report it
    /// unconditionally.
    pub fn h_prime(&self) -> f64 {
        match self.barostat.as_ref() {
            Some(b) if b.enabled => self.energy() + b.energy(self.volume()),
            _ => self.energy(),
        }
    }

    /// ONE MTK NPT STEP.
    ///
    /// The Trotter factorization, outermost first:
    /// chains(dt/2) · barostat-momentum(dt/2) · velocity(dt/2) · position+box(dt) ·
    /// FORCES · velocity(dt/2) · barostat-momentum(dt/2) · chains(dt/2).
    ///
    /// Symmetric about the force evaluation, which is what makes it time-reversible and
    /// what makes `H'` conserved rather than merely bounded.
    pub(crate) fn step_npt(&mut self) {
        let dt = self.dt();
        let e_before = self.energy();
        let Some(mut b) = self.barostat.take() else {
            return;
        };

        let n_f = self.dims.dof() * self.n as f64;
        let dt2 = 0.5 * dt;

        // --- chains, first half ---
        let two_ke = 2.0 * self.e_kin;
        let s_particles = b.particles.advance(dt2, two_ke, b.target_temperature);
        self.scale_velocities(s_particles);
        let two_ke_eps = b.p_eps * b.p_eps / b.w;
        let s_strain = b.strain.advance(dt2, two_ke_eps, b.target_temperature);
        b.p_eps *= s_strain;

        // --- barostat momentum, first half ---
        self.accumulate_energy();
        b.p_eps += dt2 * self.strain_force(&b, n_f);

        // --- velocities, first half, with the MTK damping ---
        let v_eps = b.p_eps / b.w;
        let damp = (-(1.0 + 3.0 / n_f) * v_eps * dt2).exp();
        self.kick(dt2);
        self.scale_velocities(damp);

        // --- positions and the box ---
        //
        // `r(t+dt) = r(t)·e^{v_ε dt} + p/m · e^{v_ε dt/2} · sinh(v_ε dt/2)/(v_ε dt/2) · dt`
        // is the exact solution of `ṙ = p/m + v_ε r` for constant `v_ε`. The `sinh(x)/x`
        // factor is written as a series near zero because the closed form loses every digit
        // there — and `v_ε = 0` is not a corner case, it is what the barostat does whenever
        // the pressure is on target.
        let x = v_eps * dt2;
        let expx = x.exp();
        let sinhc = if x.abs() < 1e-8 {
            1.0 + x * x / 6.0
        } else {
            x.sinh() / x
        };
        for i in 0..self.n {
            let a = &mut self.atoms[i];
            a.x = a.x * expx * expx + a.vx * dt * expx * sinhc;
            a.y = a.y * expx * expx + a.vy * dt * expx * sinhc;
            a.z = a.z * expx * expx + a.vz * dt * expx * sinhc;
        }
        self.width *= expx * expx;
        self.height *= expx * expx;
        self.depth *= expx * expx;
        if self.boundary.wraps() {
            let geom = self.geom();
            for i in 0..self.n {
                let (x, y, z) = geom.wrap((self.atoms[i].x, self.atoms[i].y, self.atoms[i].z));
                self.atoms[i].x = x;
                self.atoms[i].y = y;
                self.atoms[i].z = z;
            }
        }

        self.compute_forces();

        // --- velocities, second half ---
        self.scale_velocities(damp);
        self.kick(dt2);
        self.accumulate_energy();

        // --- barostat momentum, second half ---
        b.p_eps += dt2 * self.strain_force(&b, n_f);

        // --- chains, second half ---
        let two_ke_eps = b.p_eps * b.p_eps / b.w;
        let s_strain = b.strain.advance(dt2, two_ke_eps, b.target_temperature);
        b.p_eps *= s_strain;
        let two_ke = 2.0 * self.e_kin;
        let s_particles = b.particles.advance(dt2, two_ke, b.target_temperature);
        self.scale_velocities(s_particles);
        self.accumulate_energy();

        self.barostat = Some(b);
        self.time += dt;
        self.steps += 1;

        // WB-4.3: every hartree the barostat and its chains moved goes on the receipt.
        // `E − W_ext` is therefore still the constant the energy gate checks, and `H'` is a
        // second conserved quantity with its own gate — one per law.
        let moved = self.energy() - e_before;
        self.w_ext += moved;
        self.work.barostat += moved;

        let d = self.drift();
        if d > self.drift_peak {
            self.drift_peak = d;
        }
        let m = self.mode_energy();
        if m > self.e_ref {
            self.e_ref = m;
        }
    }

    /// `ṗ_ε = 3V(P_int − P_ext) + (3/N_f)·2K` — the MTK strain force, with its correction
    /// term named.
    fn strain_force(&self, b: &Barostat, n_f: f64) -> f64 {
        let v = self.volume();
        3.0 * v * (self.pressure() - b.target_pressure) + (3.0 / n_f) * 2.0 * self.e_kin
    }

    /// One velocity half-kick from the current forces. Shared by the NVE and NPT
    /// integrators so there is one statement of `v += (F/m)·dt/2`.
    fn kick(&mut self, half_dt: f64) {
        for i in 0..self.n {
            let (px, py, pz) = self.a_pair_at(i);
            let (ex, ey, ez) = self.a_ext_at(i);
            let h = half_dt / self.atoms[i].mass();
            self.atoms[i].vx += h * (px + ex);
            self.atoms[i].vy += h * (py + ey);
            self.atoms[i].vz += h * (pz + ez);
        }
    }

    fn scale_velocities(&mut self, s: f64) {
        if s == 1.0 {
            return;
        }
        for i in 0..self.n {
            self.atoms[i].vx *= s;
            self.atoms[i].vy *= s;
            self.atoms[i].vz *= s;
        }
    }
}

/// Default barostat time constant, atomic units (~24 fs). Long against a hydrogen
/// vibration (~8 fs) so the box does not ring at the bond frequency, short enough that a
/// few thousand steps equilibrate the volume. STAKED, not fitted.
pub const DEFAULT_TAU_P: f64 = 1000.0;
/// Default thermostat time constant, atomic units. STAKED, not fitted.
pub const DEFAULT_TAU_T: f64 = 500.0;

impl crate::sim::Sim {
    /// THE HAND ON THE BOX: scale the container and everything in it by `factor`,
    /// with the energy cost measured directly and posted to the ledger's hand
    /// column. This is WB-2.2's mechanism — the page changes the SIZE of the box
    /// to change the pressure, and reads the pressure back; no controller, no
    /// setpoint, no unledgered work. Positions scale affinely (the standard
    /// volume move), velocities are untouched, and the four-body displacement
    /// cache is invalidated with everything else by the fresh force pass.
    pub fn scale_box(&mut self, factor: f64) -> Result<(), ScaleRefusal> {
        if !(factor.is_finite() && factor > 0.0) {
            return Err(ScaleRefusal::BadFactor);
        }
        let floor = 2.0 * self.wall_inset;
        if self.width * factor <= floor
            || self.height * factor <= floor
            || (self.dims == crate::sim::Dims::Three && self.depth * factor <= floor)
        {
            return Err(ScaleRefusal::CollapsesBox);
        }
        if self.boundary.wraps() {
            let (cut, half_edge) = self.pbc_margin();
            if !(cut.is_finite() && cut <= half_edge * factor) {
                return Err(ScaleRefusal::BreaksPeriodicImages);
            }
        }
        let before = self.energy();
        self.width *= factor;
        self.height *= factor;
        if self.dims == crate::sim::Dims::Three {
            self.depth *= factor;
        }
        for a in &mut self.atoms {
            a.x *= factor;
            a.y *= factor;
            a.z *= factor;
        }
        if self.grabbed.is_some() {
            self.anchor.0 *= factor;
            self.anchor.1 *= factor;
            self.anchor.2 *= factor;
        }
        self.compute_forces();
        self.accumulate_energy();
        let after = self.energy();
        // Both ledger lines, the house pattern: `w_ext` is the total the drift gate
        // reads, `work.hand` the receipt column saying WHO paid, and `work_columns_ok`
        // checks they never part.
        self.w_ext += after - before;
        self.work.hand += after - before;
        Ok(())
    }
}
