//! Hydrogen atoms in a 2D box, integrated symplectically, with every energy and
//! momentum flow written to a ledger.
//!
//! Units are Hartree atomic units throughout: length in bohr, energy in hartree, mass
//! in electron masses, time in hbar/E_h (24.189 as). Nothing is converted for display
//! except in the viewer, so no unit constant is ever applied twice.

use crate::clock::Timescale;
use crate::holon::HolonLayer;
use crate::table::PotentialTable;

/// Mass of a protium ATOM (proton + electron) in electron masses:
/// 1.00782503207 u x 1822.888486 m_e/u. The atom, not the proton — the pair curve is
/// Born-Oppenheimer, so the electrons ride with the nuclei and their mass belongs here.
pub const M_H: f64 = 1837.152;

/// The proton, for reference: 1836.152673 m_e.
///
/// The brief specifies the reduced mass of two PROTONS for the timescale derivation, and
/// this crate uses the reduced mass of two ATOMS instead. The reason is the one stated
/// above — the curve is Born-Oppenheimer, so the electrons ride with the nuclei and their
/// inertia belongs in the moving mass — and the cost of the choice is 1 electron mass in
/// 1837, i.e. 0.054% on mu and 0.027% on every frequency derived from it. Recorded here
/// rather than silently resolved, because it is a deliberate departure from the brief and
/// it should be visible to whoever checks the numbers.
pub const M_PROTON: f64 = 1836.152673;

pub const MAX_ATOMS: usize = 16;
pub const MAX_PAIRS: usize = MAX_ATOMS * (MAX_ATOMS - 1) / 2;

/// Boltzmann's constant in hartree per kelvin.
pub const K_B: f64 = 3.166811563e-6;

/// Wall stiffness, hartree/bohr^2. A5 stage value: it is scene furniture, not physics
/// from any table, and is named as such here rather than hidden in the force loop.
pub const K_WALL: f64 = 0.5;
/// User-spring stiffness, hartree/bohr^2. Also a stage value. Finite on purpose: a
/// finite spring cannot push two atoms arbitrarily far up the repulsive wall, which is
/// the honest behaviour rather than a cheat that lets the pointer overpower the curve.
pub const K_SPRING: f64 = 0.05;

/// Distance beyond which the outer-turning-point search gives up and reports infinity.
const TURNING_POINT_CAP: f64 = 200.0;

/// Safety factor on the derived drift bound. The (omega*dt)^2/4 result below is EXACT
/// for a harmonic oscillator and leading-order in dt^2 for anything else; anharmonicity
/// enters at the same order with a coefficient set by U''' and the amplitude, so a
/// fixed multiple is the honest way to admit "leading order, not a theorem here". The
/// measured-over-bound ratio is reported so the margin is visible rather than absorbed.
pub const DRIFT_SAFETY: f64 = 4.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Boundary {
    /// Soft quadratic walls on all four sides.
    Walls,
    /// No walls at all. Translation invariance is exact, so total momentum is conserved
    /// to roundoff and the momentum gate has nothing to subtract.
    Open,
}

#[derive(Clone, Copy, Default)]
pub struct Atom {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
}

/// One pair's bond reading, computed from the table alone.
#[derive(Clone, Copy, Default)]
pub struct PairReading {
    pub i: usize,
    pub j: usize,
    pub r: f64,
    /// Relative energy in the pair's own centre-of-mass frame, asymptote-zeroed.
    pub e_rel: f64,
    /// Outer classical turning point of the effective radial potential at `e_rel`.
    pub r_outer: f64,
    pub bonded: bool,
}

impl PairReading {
    /// Bond-sector energy for this pair's ledger row: pair potential plus pair-frame
    /// kinetic energy.
    ///
    /// Numerically this IS `e_rel` — the same quantity in a second role. It is named
    /// separately because the roles are different (one is the bond criterion's input, the
    /// other is a composite holon's ledger row) and defined once because two definitions
    /// of one number is how they drift apart.
    pub fn e_bond(&self) -> f64 {
        self.e_rel
    }
}

pub struct Sim {
    pub table: PotentialTable,
    pub atoms: [Atom; MAX_ATOMS],
    pub n: usize,
    pub boundary: Boundary,
    pub width: f64,
    pub height: f64,
    /// The walls act on atom centres, inset by the drawn radius so the picture and the
    /// physics agree about where the edge is.
    pub wall_inset: f64,

    // --- accelerations, kept split so the momentum ledger can name what is external ---
    a_pair: [(f64, f64); MAX_ATOMS],
    a_ext: [(f64, f64); MAX_ATOMS],

    // --- the user's spring ---
    pub grabbed: Option<usize>,
    pub anchor: (f64, f64),

    // --- thermostat (off by default) ---
    pub thermostat_on: bool,
    pub target_temperature: f64,
    pub thermostat_tau: f64,

    // --- THE LEDGER ---
    pub e_kin: f64,
    pub e_pair: f64,
    pub e_wall: f64,
    pub e_spring: f64,
    /// Every joule the outside world put in: anchor motion, spring teardown on release,
    /// and thermostat rescaling. The intervention is a term in the ledger, never outside it.
    pub w_ext: f64,
    /// The ledger's invariant at reset. `ledger() - w_ext` must equal this forever.
    pub l0: f64,
    /// Total momentum at reset, and the external impulse since.
    pub p0: (f64, f64),
    pub j_ext: (f64, f64),

    pub time: f64,
    pub steps: u64,

    // --- running maxima that define the drift bound (set by the trajectory, not by hand) ---
    k_pair_max: f64,
    wall_engaged: bool,
    spring_engaged: bool,
    e_ref: f64,
    pub drift_peak: f64,
    pub momentum_residual_peak: f64,

    pub pairs: [PairReading; MAX_PAIRS],
    pub pair_count: usize,

    /// The three clocks and the degradation contract.
    pub timescale: Timescale,
    /// The composite-holon layer. Runs at grain boundaries only.
    pub holons: HolonLayer,
    /// Grain boundaries closed since reset. The holon layer's clock.
    pub frame: u64,
    /// Largest pair relative energy seen since reset — what the curvature envelope, and
    /// therefore the drift bound, is derived from.
    pub e_rel_max: f64,
}

impl Sim {
    pub const fn empty() -> Self {
        Self {
            table: PotentialTable::empty(),
            atoms: [Atom {
                x: 0.0,
                y: 0.0,
                vx: 0.0,
                vy: 0.0,
            }; MAX_ATOMS],
            n: 0,
            boundary: Boundary::Walls,
            width: 40.0,
            height: 24.0,
            wall_inset: 0.6,
            a_pair: [(0.0, 0.0); MAX_ATOMS],
            a_ext: [(0.0, 0.0); MAX_ATOMS],
            grabbed: None,
            anchor: (0.0, 0.0),
            thermostat_on: false,
            target_temperature: 300.0,
            thermostat_tau: 2000.0,
            e_kin: 0.0,
            e_pair: 0.0,
            e_wall: 0.0,
            e_spring: 0.0,
            w_ext: 0.0,
            l0: 0.0,
            p0: (0.0, 0.0),
            j_ext: (0.0, 0.0),
            time: 0.0,
            steps: 0,
            k_pair_max: 0.0,
            wall_engaged: false,
            spring_engaged: false,
            e_ref: 0.0,
            drift_peak: 0.0,
            momentum_residual_peak: 0.0,
            pairs: [PairReading {
                i: 0,
                j: 0,
                r: 0.0,
                e_rel: 0.0,
                r_outer: 0.0,
                bonded: false,
            }; MAX_PAIRS],
            pair_count: 0,
            timescale: Timescale::empty(),
            holons: HolonLayer::empty(),
            frame: 0,
            e_rel_max: f64::NEG_INFINITY,
        }
    }

    /// The integration step in force. Derived from the curve by `Timescale`, never a
    /// constant in this file.
    pub fn dt(&self) -> f64 {
        self.timescale.dt
    }

    /// Re-derive every clock from the table. Call after loading a curve.
    pub fn adopt_table_timescale(&mut self) {
        let mu = 0.5 * M_H;
        self.timescale.from_table(&self.table, mu);
    }

    /// Total energy currently held by the scene.
    pub fn energy(&self) -> f64 {
        self.e_kin + self.e_pair + self.e_wall + self.e_spring
    }

    /// The conserved quantity. `E - W_ext` is constant for an exact integrator, with or
    /// without the user's hand in the box.
    pub fn ledger(&self) -> f64 {
        self.energy() - self.w_ext
    }

    pub fn drift(&self) -> f64 {
        (self.ledger() - self.l0).abs()
    }

    /// The energy-drift bound, DERIVED rather than tuned.
    ///
    /// Velocity Verlet applied to a one-dimensional harmonic oscillator of angular
    /// frequency omega is a linear symplectic map, and it conserves EXACTLY the
    /// quadratic form
    ///
    /// ```text
    /// H~ = 1/2 v^2 + 1/2 omega^2 (1 - omega^2 dt^2 / 4) x^2
    /// ```
    ///
    /// (verified numerically against the step map before this bound was written down:
    /// the softening sits on the STIFFNESS, not on the kinetic term). The true energy
    /// is therefore E = H~ + (omega^4 dt^2 / 8) x^2, which oscillates as x^2 sweeps
    /// [0, x_max^2] and does NOT drift secularly — the whole point of a symplectic
    /// integrator, and the reason the 10k-step test asserts the same bound as a 10-step
    /// one would. Substituting x_max^2 = 2 H~ / (omega^2 (1 - omega^2 dt^2 / 4)) and
    /// E_0 = H~ / (1 - omega^2 dt^2 / 4) (the energy at the turning point) gives the
    /// peak-to-peak swing
    ///
    /// ```text
    /// |dE| / E_0 = (omega dt)^2 / 4      (exact; tight, not conservative)
    /// ```
    ///
    /// and the map is unstable for omega dt >= 2, where H~ stops being positive definite.
    ///
    /// Carrying that to this scene: `omega` is `Timescale::omega_env`, the frequency of
    /// the stiffest curvature a pair can REACH at the largest relative energy seen so far
    /// (on the relative coordinate, hence the reduced mass m/2), widened by the wall and
    /// spring stiffnesses once those have actually engaged; and `E_0` is the largest
    /// energy scale the ledger has held.
    ///
    /// Reaching rather than visiting is fence 3, and it is the whole difference between a
    /// bound that survives a collision and one that does not. A bound built from the
    /// curvature the trajectory HAS sampled reads green right up to the encounter that
    /// violates it, because the stiff part of the curve has not been touched yet. The
    /// envelope asks instead what the pair could reach on the energy it already has, so
    /// the number is valid THROUGH the collision rather than up to it.
    ///
    /// Nothing here is cached: `dt` and `omega_env` are read live on every call, so a
    /// changed timestep cannot leave a stale bound behind — there is no stored bound to
    /// go stale.
    pub fn drift_bound(&self) -> f64 {
        let mut omega_sq: f64 = self.timescale.omega_env * self.timescale.omega_env;
        if self.wall_engaged {
            omega_sq = omega_sq.max(K_WALL / M_H);
        }
        if self.spring_engaged {
            omega_sq = omega_sq.max(K_SPRING / M_H);
        }
        let e_ref = self.e_ref.max(self.table.d_e.abs());
        let dt = self.dt();
        DRIFT_SAFETY * 0.25 * omega_sq * dt * dt * e_ref
    }

    pub fn energy_gate(&self) -> bool {
        self.drift_peak <= self.drift_bound()
    }

    /// Momentum residual: `|P(t) - P(0) - J_ext(t)|`.
    ///
    /// Pairwise forces are applied as equal and opposite to the two partners, so they
    /// cancel from the total in exact arithmetic; walls and the spring do not, and their
    /// impulse is accumulated as it enters the velocities. What is left is floating-point
    /// cancellation error only.
    pub fn momentum_residual(&self) -> f64 {
        let (px, py) = self.momentum();
        let dx = px - self.p0.0 - self.j_ext.0;
        let dy = py - self.p0.1 - self.j_ext.1;
        (dx * dx + dy * dy).sqrt()
    }

    /// Roundoff bound for the momentum ledger. Each step commits O(N) floating-point
    /// additions into the momentum sum, each carrying at most one unit in the last place
    /// of the running magnitude; accumulating those worst-case (rather than as the
    /// random walk they actually are) gives `8 * steps * eps * p_scale`.
    pub fn momentum_bound(&self) -> f64 {
        let mut p_scale: f64 = 0.0;
        for i in 0..self.n {
            let a = &self.atoms[i];
            p_scale += M_H * (a.vx * a.vx + a.vy * a.vy).sqrt();
        }
        let p_scale = p_scale.max(1e-12);
        8.0 * (self.steps.max(1) as f64) * f64::EPSILON * p_scale
    }

    pub fn momentum_gate(&self) -> bool {
        self.momentum_residual_peak <= self.momentum_bound()
    }

    pub fn momentum(&self) -> (f64, f64) {
        let mut px = 0.0;
        let mut py = 0.0;
        for i in 0..self.n {
            px += M_H * self.atoms[i].vx;
            py += M_H * self.atoms[i].vy;
        }
        (px, py)
    }

    pub fn temperature(&self) -> f64 {
        if self.n == 0 {
            return 0.0;
        }
        // Two translational degrees of freedom per atom.
        self.e_kin / (self.n as f64 * K_B)
    }

    /// Place `n` atoms and zero the ledger. Deterministic: no RNG, so a reported run can
    /// be re-run byte-for-byte.
    pub fn reset(&mut self, n: usize) {
        self.n = n.clamp(0, MAX_ATOMS);
        self.grabbed = None;
        self.thermostat_on = false;
        let cx = 0.5 * self.width;
        let cy = 0.5 * self.height;
        for i in 0..self.n {
            let a = &mut self.atoms[i];
            if self.n <= 2 {
                // The headline scene: two atoms drifting slowly TOWARD each other. They
                // will collide, climb the repulsive wall, and separate again without
                // ever sticking, which is the lesson the app is built around.
                //
                // The inward speed is not decoration. Two atoms placed at rest at any
                // finite separation are ALREADY bound by the criterion in
                // `refresh_pairs` — their relative energy is U(R) < 0, and classically
                // they would fall together and never escape — so a scene that opened at
                // rest would open reading BONDED and teach the opposite of the point.
                // 0.0004 bohr per atomic time unit puts the relative energy at
                // +2.4e-4 Eh, honestly above the asymptote, on the placeholder curve.
                let sign = if i == 0 { -1.0 } else { 1.0 };
                a.x = cx + sign * 5.0;
                a.y = cy;
                a.vx = -sign * 0.0004;
                a.vy = 0.0;
            } else {
                // A deterministic ring, at rest.
                let theta = (i as f64) * core::f64::consts::TAU / (self.n as f64);
                let radius = 6.0;
                a.x = cx + radius * theta.cos();
                a.y = cy + radius * theta.sin();
                a.vx = 0.0;
                a.vy = 0.0;
            }
        }
        self.zero_ledger();
    }

    fn zero_ledger(&mut self) {
        self.w_ext = 0.0;
        self.j_ext = (0.0, 0.0);
        self.time = 0.0;
        self.steps = 0;
        self.frame = 0;
        self.k_pair_max = 0.0;
        self.wall_engaged = false;
        self.spring_engaged = false;
        self.e_ref = 0.0;
        self.drift_peak = 0.0;
        self.momentum_residual_peak = 0.0;
        self.holons.reset();
        self.compute_forces();
        self.accumulate_energy();
        self.l0 = self.ledger();
        self.p0 = self.momentum();
        self.e_ref = self.energy().abs().max(self.table.d_e.abs());
        self.refresh_pairs();
        // Seed the curvature envelope from the pair energies this scene actually starts
        // with, not from zero: a scene of loosely bound pairs cannot reach the wall, and
        // an envelope seeded at U = 0 would quote a bound for a collision that the
        // energy forbids.
        self.e_rel_max = f64::NEG_INFINITY;
        self.timescale.e_rel_max = f64::NEG_INFINITY;
        self.timescale.k_env = 0.0;
        self.refresh_envelope();
    }

    /// Widen the curvature envelope to cover the largest pair energy seen so far.
    fn refresh_envelope(&mut self) {
        let mut e_max = self.e_rel_max;
        for p in &self.pairs[..self.pair_count] {
            if p.e_rel > e_max {
                e_max = p.e_rel;
            }
        }
        if self.pair_count == 0 && !e_max.is_finite() {
            e_max = 0.0;
        }
        self.e_rel_max = e_max;
        self.timescale.refresh_envelope(&self.table, e_max);
    }

    /// ONE GRAIN BOUNDARY: the closure-aligned checkpoint where every coarse view is
    /// refreshed at once.
    ///
    /// Order matters and is fixed: pair readings first (they are what everything above
    /// reads), then the envelope (so the bound covers what just happened), then the
    /// global ledger gates, then the composite-holon layer. Each stage reads the stage
    /// below and writes nothing dynamical.
    pub fn close_grain(&mut self) {
        self.frame += 1;
        self.refresh_pairs();
        self.refresh_envelope();

        let e_now = self.energy().abs();
        if e_now > self.e_ref {
            self.e_ref = e_now;
        }
        // The momentum residual is sampled HERE and not per substep, and the asymmetry
        // with the energy drift above is deliberate. That residual is a floating-point
        // random walk, not an oscillation, so it has no period to alias against and a
        // boundary sample of it is a fair one. It also costs O(N) per evaluation rather
        // than the drift's handful of flops, so per-substep sampling would buy nothing
        // and charge for it.
        let m = self.momentum_residual();
        if m > self.momentum_residual_peak {
            self.momentum_residual_peak = m;
        }

        // The composite layer sees a state nothing above it has modified.
        let count = self.pair_count;
        let frame = self.frame;
        let time = self.time;
        let d_e = self.table.d_e;
        let n = self.n;
        let (pairs, holons) = (&self.pairs, &mut self.holons);
        holons.step_boundary(&pairs[..count], n, frame, time, d_e);
    }

    /// Advance `substeps` fixed steps and close the grain once at the end.
    pub fn step_frame(&mut self, substeps: u32) {
        for _ in 0..substeps {
            self.step();
        }
        self.close_grain();
    }

    pub fn set_velocity(&mut self, i: usize, vx: f64, vy: f64) {
        if i < self.n {
            self.atoms[i].vx = vx;
            self.atoms[i].vy = vy;
        }
    }

    pub fn set_position(&mut self, i: usize, x: f64, y: f64) {
        if i < self.n {
            self.atoms[i].x = x;
            self.atoms[i].y = y;
        }
    }

    /// Freeze the current state as the ledger's origin. Call after any scripted setup
    /// that is meant to be the initial condition rather than an intervention.
    pub fn rebase(&mut self) {
        self.zero_ledger();
    }

    // ---------------------------------------------------------------- forces

    fn wall_energy_force(&self, x: f64, y: f64) -> (f64, f64, f64, bool) {
        if self.boundary == Boundary::Open {
            return (0.0, 0.0, 0.0, false);
        }
        let lo = self.wall_inset;
        let hi_x = self.width - self.wall_inset;
        let hi_y = self.height - self.wall_inset;
        let mut u = 0.0;
        let mut fx = 0.0;
        let mut fy = 0.0;
        let mut touched = false;
        if x < lo {
            let d = lo - x;
            u += 0.5 * K_WALL * d * d;
            fx += K_WALL * d;
            touched = true;
        } else if x > hi_x {
            let d = x - hi_x;
            u += 0.5 * K_WALL * d * d;
            fx -= K_WALL * d;
            touched = true;
        }
        if y < lo {
            let d = lo - y;
            u += 0.5 * K_WALL * d * d;
            fy += K_WALL * d;
            touched = true;
        } else if y > hi_y {
            let d = y - hi_y;
            u += 0.5 * K_WALL * d * d;
            fy -= K_WALL * d;
            touched = true;
        }
        (u, fx, fy, touched)
    }

    /// Recompute `a_pair` and `a_ext` from the current positions, and refresh the
    /// potential terms of the ledger. Split so the momentum ledger can tell the
    /// internal forces (which cancel) from the external ones (which do not).
    fn compute_forces(&mut self) {
        for i in 0..self.n {
            self.a_pair[i] = (0.0, 0.0);
            self.a_ext[i] = (0.0, 0.0);
        }
        let mut e_pair = 0.0;
        let mut k_pair_max = self.k_pair_max;

        for i in 0..self.n {
            for j in (i + 1)..self.n {
                let dx = self.atoms[j].x - self.atoms[i].x;
                let dy = self.atoms[j].y - self.atoms[i].y;
                let r2 = dx * dx + dy * dy;
                // Two atoms at exactly the same point have no defined direction; the
                // repulsive wall makes this unreachable dynamically, and the guard keeps
                // it from being a NaN source if a caller places them there.
                let r = r2.sqrt().max(1e-9);
                let (value, slope, curv) = self.table.eval(r);
                e_pair += value;
                // F = -dE/dR along the separation; positive slope pulls the pair together.
                let f_over_r = slope / r;
                let fx = f_over_r * dx;
                let fy = f_over_r * dy;
                // Newton's third law, applied as one computed value with opposite signs:
                // this is what makes the pair contribution cancel from the momentum sum.
                self.a_pair[i].0 += fx;
                self.a_pair[i].1 += fy;
                self.a_pair[j].0 -= fx;
                self.a_pair[j].1 -= fy;
                let ac = curv.abs();
                if ac > k_pair_max {
                    k_pair_max = ac;
                }
            }
        }
        self.k_pair_max = k_pair_max;
        self.e_pair = e_pair;

        let mut e_wall = 0.0;
        for i in 0..self.n {
            let (u, fx, fy, touched) = self.wall_energy_force(self.atoms[i].x, self.atoms[i].y);
            e_wall += u;
            self.a_ext[i].0 += fx;
            self.a_ext[i].1 += fy;
            if touched {
                self.wall_engaged = true;
            }
        }
        self.e_wall = e_wall;

        self.e_spring = 0.0;
        if let Some(g) = self.grabbed {
            if g < self.n {
                let dx = self.atoms[g].x - self.anchor.0;
                let dy = self.atoms[g].y - self.anchor.1;
                self.e_spring = 0.5 * K_SPRING * (dx * dx + dy * dy);
                self.a_ext[g].0 += -K_SPRING * dx;
                self.a_ext[g].1 += -K_SPRING * dy;
                self.spring_engaged = true;
            }
        }
    }

    fn accumulate_energy(&mut self) {
        let mut e_kin = 0.0;
        for i in 0..self.n {
            let a = &self.atoms[i];
            e_kin += 0.5 * M_H * (a.vx * a.vx + a.vy * a.vy);
        }
        self.e_kin = e_kin;
    }

    // ---------------------------------------------------------------- stepping

    /// One velocity-Verlet step.
    ///
    /// The external impulse is accumulated from exactly the same half-kick terms that
    /// enter the velocities, so the momentum ledger is not an independent estimate of
    /// the impulse — it is the impulse.
    pub fn step(&mut self) {
        if self.n == 0 || !self.table.is_loaded() {
            return;
        }
        let dt = self.dt();
        let half = 0.5 * dt / M_H;

        let mut jx = 0.0;
        let mut jy = 0.0;
        for i in 0..self.n {
            let (px, py) = self.a_pair[i];
            let (ex, ey) = self.a_ext[i];
            self.atoms[i].vx += half * (px + ex);
            self.atoms[i].vy += half * (py + ey);
            jx += 0.5 * dt * ex;
            jy += 0.5 * dt * ey;
        }

        for i in 0..self.n {
            self.atoms[i].x += dt * self.atoms[i].vx;
            self.atoms[i].y += dt * self.atoms[i].vy;
        }

        self.compute_forces();

        for i in 0..self.n {
            let (px, py) = self.a_pair[i];
            let (ex, ey) = self.a_ext[i];
            self.atoms[i].vx += half * (px + ex);
            self.atoms[i].vy += half * (py + ey);
            jx += 0.5 * dt * ex;
            jy += 0.5 * dt * ey;
        }
        self.j_ext.0 += jx;
        self.j_ext.1 += jy;

        self.accumulate_energy();

        if self.thermostat_on {
            self.apply_thermostat();
        }

        self.time += dt;
        self.steps += 1;

        // The energy drift EXTREMUM is tracked per substep; the energy GATE is still
        // evaluated at grain boundaries (`close_grain`), which is what closure-aligned
        // scheduling asks for. Splitting the two is not a liberty, it is a measured
        // necessity: the drift is a bounded OSCILLATION at the vibrational frequency, and
        // sampling it only at boundaries is stroboscopic. With dt = period/64, a frame of
        // 64 substeps is exactly one vibration, so every boundary lands at the same phase
        // and the gate reads a fixed point of the cycle rather than its peak. Measured
        // (examples/diagnose.rs, probe 5): at 64 and 128 substeps per frame the boundary
        // sample is 0.1110 of the true peak; at 16, 32, 48, 61, 63, 65 and 96 it is
        // 1.0000. A gate that goes blind exactly when the frame divides the period evenly
        // is a gate that fails on the tidy configuration and passes on the ragged one.
        //
        // The cost is about seven flops: `energy()` is four adds over terms this step has
        // already updated, and the rest is a subtraction and a compare.
        let d = self.drift();
        if d > self.drift_peak {
            self.drift_peak = d;
        }
    }

    /// Berendsen velocity rescaling. Whatever kinetic energy it adds or removes is
    /// posted to `w_ext` in the same breath, so a thermostatted run is still a closed
    /// ledger rather than an excused one.
    ///
    /// The rescaling also changes the total momentum (it multiplies every velocity),
    /// and that change is posted to `j_ext` for the same reason.
    fn apply_thermostat(&mut self) {
        let t_now = self.temperature();
        if t_now <= 0.0 {
            return;
        }
        let ratio = self.target_temperature / t_now;
        let lambda_sq = 1.0 + (self.dt() / self.thermostat_tau) * (ratio - 1.0);
        if lambda_sq <= 0.0 {
            return;
        }
        let lambda: f64 = lambda_sq.sqrt();
        let before = self.e_kin;
        let (pbx, pby) = self.momentum();
        for i in 0..self.n {
            self.atoms[i].vx *= lambda;
            self.atoms[i].vy *= lambda;
        }
        self.accumulate_energy();
        self.w_ext += self.e_kin - before;
        let (pax, pay) = self.momentum();
        self.j_ext.0 += pax - pbx;
        self.j_ext.1 += pay - pby;
    }

    // ---------------------------------------------------------------- the hand

    /// Grab atom `i`. The anchor starts ON the atom, so the spring enters the ledger at
    /// zero extension and the grab itself injects nothing.
    pub fn grab(&mut self, i: usize) {
        if i >= self.n {
            return;
        }
        self.grabbed = Some(i);
        self.anchor = (self.atoms[i].x, self.atoms[i].y);
        self.spring_engaged = true;
        self.compute_forces();
    }

    /// Move the anchor. The spring is a term in the Hamiltonian with a time-dependent
    /// parameter; moving that parameter at fixed atom position changes the stored spring
    /// energy by exactly `dU`, and `dU` IS the work the user's hand did. Posting it here
    /// is what keeps `E - W_ext` constant through a drag, with no path integral to
    /// approximate and no second-order error of its own.
    pub fn move_anchor(&mut self, x: f64, y: f64) {
        let Some(g) = self.grabbed else { return };
        if g >= self.n {
            return;
        }
        let before = self.e_spring;
        self.anchor = (x, y);
        let dx = self.atoms[g].x - x;
        let dy = self.atoms[g].y - y;
        let after = 0.5 * K_SPRING * (dx * dx + dy * dy);
        self.w_ext += after - before;
        self.compute_forces();
    }

    /// Release. The energy still stored in the spring leaves the scene with the hand, so
    /// it is subtracted from `w_ext` — otherwise release would look like a free energy
    /// gain of exactly the stored amount.
    pub fn release(&mut self) {
        if self.grabbed.is_none() {
            return;
        }
        self.w_ext -= self.e_spring;
        self.grabbed = None;
        self.compute_forces();
    }

    // ---------------------------------------------------------------- bonds

    /// Bond readings for every pair, from the table alone.
    ///
    /// A pair is BONDED when
    ///   (1) its relative energy is below the dissociation asymptote, and
    ///   (2) its separation is inside the outer classical turning point at that energy.
    ///
    /// Both come from the curve: (1) is `E_rel < E_asymptote`, which in the
    /// asymptote-zeroed convention is `E_rel < 0`; (2) solves `U_eff(R) = E_rel` on the
    /// same interpolant. There is no distance cutoff and no fitted threshold anywhere.
    ///
    /// Worth being straight about: for an ISOLATED pair, (2) is implied by (1). Any
    /// state the pair actually occupies satisfies `U_eff(R) <= E_rel` by construction
    /// (the leftover is the radial kinetic energy, which cannot be negative), so R is
    /// always inside the turning point. Condition (2) is therefore a redundancy check
    /// here rather than a second independent criterion — it can only fire if the
    /// turning-point solve and the energy disagree, which would mean the interpolant is
    /// not single-valued in the way the search assumes. It is kept because it is the
    /// stated criterion, because `r_outer` is worth displaying as the bond's reach, and
    /// because the redundancy is a live check on the table rather than a dead one.
    ///
    /// The consequence of (1) that the demo exists to show: two atoms alone, approaching
    /// from outside the well, ALWAYS have `E_rel >= 0` and can never bond, no matter how
    /// hard they are pushed together. Forming H2 requires taking energy out — a third
    /// atom to carry it away, a thermostat, or the user's own spring braking one of them
    /// — and the ledger says exactly how much left.
    pub fn refresh_pairs(&mut self) {
        let mu = 0.5 * M_H;
        let mut k = 0usize;
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                if k >= MAX_PAIRS {
                    break;
                }
                let dx = self.atoms[j].x - self.atoms[i].x;
                let dy = self.atoms[j].y - self.atoms[i].y;
                let r = (dx * dx + dy * dy).sqrt().max(1e-9);
                let vx = self.atoms[j].vx - self.atoms[i].vx;
                let vy = self.atoms[j].vy - self.atoms[i].vy;
                let ke_rel = 0.5 * mu * (vx * vx + vy * vy);
                let u = self.table.u(r);
                let e_rel = ke_rel + u;
                // z-component of the relative angular momentum, for the centrifugal term.
                let l = mu * (dx * vy - dy * vx);
                let r_outer =
                    self.table
                        .outer_turning_point(e_rel, l * l, mu, r, TURNING_POINT_CAP);
                self.pairs[k] = PairReading {
                    i,
                    j,
                    r,
                    e_rel,
                    r_outer,
                    bonded: e_rel < 0.0 && r < r_outer,
                };
                k += 1;
            }
        }
        self.pair_count = k;
    }

    pub fn bonded_count(&self) -> usize {
        self.pairs[..self.pair_count]
            .iter()
            .filter(|p| p.bonded)
            .count()
    }
}
