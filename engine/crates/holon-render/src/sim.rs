//! Hydrogen atoms in a box, integrated symplectically, with every energy and momentum
//! flow written to a ledger.
//!
//! Units are Hartree atomic units throughout: length in bohr, energy in hartree, mass
//! in electron masses, time in hbar/E_h (24.189 as). Nothing is converted for display
//! except in the viewer, so no unit constant is ever applied twice.
//!
//! # Two dimensions and three, in ONE integrator
//!
//! The state carries three components per atom and the box has three pairs of faces.
//! The 2D scene is not a separate code path: it is the exact z = depth/2 SLICE of the
//! same 3D world. Every atom starts on that plane with `vz = 0`; the pair force along z
//! is `(slope/r) * dz` with `dz = 0`, the z faces are never reached, so `az` is
//! identically `0.0` and the plane is invariant — not approximately, exactly, because a
//! float times zero is zero and adding zero to a finite float changes no bit. Every
//! sum that grew a third term grew it in the order `(xx + yy) + zz`, so the 2D
//! arithmetic is bit-for-bit what it was before the lift. That is what lets the canvas
//! shell, the browser ABI and the existing gate tests carry over untouched.
//!
//! Exactly two things are genuinely dimension-dependent, and both are named rather than
//! inferred: the equipartition denominator in [`Sim::temperature`] (2 translational
//! degrees of freedom per atom against 3), and the opening scene in [`Sim::reset`].
//! Both read [`Sim::dims`]. Everything else — the curve, the force law, the bond
//! predicate, the turning point, the drift bound, the clocks — is RADIAL, a function of
//! the scalar separation alone, and so carries into 3D with nothing to re-derive.

use crate::clock::Timescale;
use crate::holon::HolonLayer;
use crate::table::PotentialTable;
use holon_chem::trimer::TrimerTable;

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
    /// Soft quadratic walls on every face of the box: four sides in 2D, six in 3D.
    Walls,
    /// No walls at all. Translation invariance is exact, so total momentum is conserved
    /// to roundoff and the momentum gate has nothing to subtract.
    Open,
}

/// How many spatial dimensions the SCENE uses. The integrator always carries three
/// components; this says how many of them the scene is allowed to move in, and it is
/// read by exactly the two places where the answer differs (see the module header).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dims {
    /// The z = depth/2 plane. The default, and what the canvas shell draws.
    Two,
    /// The full box.
    Three,
}

impl Dims {
    /// Translational degrees of freedom per atom — the equipartition denominator.
    pub fn dof(self) -> f64 {
        match self {
            Dims::Two => 2.0,
            Dims::Three => 3.0,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Atom {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
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
    /// The three-body surface. Empty until [`crate::generate_trimer_table`] fills it, and
    /// an empty one contributes an EXACT zero to every term below — so a scene that never
    /// asks for it is bit-for-bit the scene this file simulated before the term existed.
    pub trimer: TrimerTable,
    pub atoms: [Atom; MAX_ATOMS],
    pub n: usize,
    pub boundary: Boundary,
    pub width: f64,
    pub height: f64,
    /// The box's z extent. Unreachable in [`Dims::Two`], where every atom sits on the
    /// mid-plane and no force can move it off — kept anyway, because the mid-plane is
    /// defined as `depth / 2` and a scene that flips to [`Dims::Three`] must find a box
    /// already centred on it rather than one that starts at a face.
    pub depth: f64,
    /// The walls act on atom centres, inset by the drawn radius so the picture and the
    /// physics agree about where the edge is.
    pub wall_inset: f64,
    /// Which dimensions the scene moves in. See the module header.
    pub dims: Dims,

    // --- accelerations, kept split so the momentum ledger can name what is external ---
    a_pair: [(f64, f64, f64); MAX_ATOMS],
    a_ext: [(f64, f64, f64); MAX_ATOMS],

    // --- the user's spring ---
    pub grabbed: Option<usize>,
    pub anchor: (f64, f64, f64),

    // --- thermostat (off by default) ---
    pub thermostat_on: bool,
    pub target_temperature: f64,
    pub thermostat_tau: f64,

    // --- THE LEDGER ---
    pub e_kin: f64,
    pub e_pair: f64,
    /// The many-body sector: the sum of the tabulated three-body term over every triple
    /// inside the table's domain. Its OWN ledger row, never folded into `e_pair` — one
    /// reader per term, because a combined number cannot say which sector moved.
    pub e_three: f64,
    pub e_wall: f64,
    pub e_spring: f64,
    /// Every joule the outside world put in: anchor motion, spring teardown on release,
    /// and thermostat rescaling. The intervention is a term in the ledger, never outside it.
    pub w_ext: f64,
    /// The ledger's invariant at reset. `ledger() - w_ext` must equal this forever.
    pub l0: f64,
    /// Total momentum at reset, and the external impulse since.
    pub p0: (f64, f64, f64),
    pub j_ext: (f64, f64, f64),

    pub time: f64,
    pub steps: u64,

    // --- running maxima that define the drift bound (set by the trajectory, not by hand) ---
    k_pair_max: f64,
    /// Largest PER-ATOM summed three-body stiffness the force loop has actually
    /// evaluated. See [`Sim::k_three`] for the derivation. LIVE, like `k_pair_max`, and
    /// for the same reason: a static envelope taken from the table alone cannot know which
    /// triples the trajectory brings together, nor how many of them.
    k_three_max: f64,
    wall_engaged: bool,
    spring_engaged: bool,
    /// Largest energy scale the ledger has held; the bound's amplitude factor.
    pub e_ref: f64,
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
            trimer: TrimerTable::empty(),
            atoms: [Atom {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                vx: 0.0,
                vy: 0.0,
                vz: 0.0,
            }; MAX_ATOMS],
            n: 0,
            boundary: Boundary::Walls,
            width: 40.0,
            height: 24.0,
            depth: 24.0,
            wall_inset: 0.6,
            dims: Dims::Two,
            a_pair: [(0.0, 0.0, 0.0); MAX_ATOMS],
            a_ext: [(0.0, 0.0, 0.0); MAX_ATOMS],
            grabbed: None,
            anchor: (0.0, 0.0, 0.0),
            thermostat_on: false,
            target_temperature: 300.0,
            thermostat_tau: 2000.0,
            e_kin: 0.0,
            e_pair: 0.0,
            e_three: 0.0,
            e_wall: 0.0,
            e_spring: 0.0,
            w_ext: 0.0,
            l0: 0.0,
            p0: (0.0, 0.0, 0.0),
            j_ext: (0.0, 0.0, 0.0),
            time: 0.0,
            steps: 0,
            k_pair_max: 0.0,
            k_three_max: 0.0,
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

    /// The scene's MODE-ENERGY scale: the amplitude factor the drift bound needs.
    ///
    /// This is deliberately NOT `energy()`. The harmonic derivation bounds the total
    /// error by the sum over modes of each mode's own energy, and `energy()` is the
    /// SIGNED total, in which kinetic energy and (negative) bond potential cancel. In a
    /// scene with bonds they cancel almost exactly — which is precisely the situation the
    /// gate is meant to police — so the signed total tracks the CONSERVED quantity while
    /// the oscillation amplitudes it is supposed to stand for grow underneath it.
    ///
    /// Measured on the field-report repro (examples/gate_repro.rs, N = 11 walls on):
    /// `|E| = 0.49` against modes carrying 5.3 Eh, and up to 37x apart on the
    /// configuration that actually breached. Summing magnitudes is positive-definite, so
    /// no cancellation is possible.
    ///
    /// It is an OVER-estimate by construction: a pair resting at the bottom of its well
    /// carries no vibrational energy but contributes `D_e` here. That slack is bounded
    /// (one `D_e` per bonded pair) and it errs toward a wider bound, which is the safe
    /// direction for a term that multiplies a bound.
    pub fn mode_energy(&self) -> f64 {
        self.e_kin + self.e_pair.abs() + self.e_three.abs() + self.e_wall + self.e_spring
    }

    /// The INTERNAL force on atom `i`, hartree/bohr: the pair loop's contribution plus the
    /// triple loop's, which are the two that cancel from the momentum sum. Exposed so a
    /// gate can check that the force the integrator pushes with is minus the gradient of
    /// the energy the ledger sums — the precondition that makes an energy gate a
    /// measurement of integration error rather than of an inconsistency.
    pub fn internal_force(&self, i: usize) -> (f64, f64, f64) {
        if i < self.n {
            self.a_pair[i]
        } else {
            (0.0, 0.0, 0.0)
        }
    }

    /// Largest pair curvature the force loop has actually evaluated since reset. Exposed
    /// so the attribution probe can separate the two halves of the drift-bound fix.
    pub fn k_pair_max(&self) -> f64 {
        self.k_pair_max
    }

    /// The three-body stiffness the drift bound uses, hartree/bohr^2: the largest
    /// PER-ATOM total the force loop has evaluated since reset.
    ///
    /// # The derivation
    ///
    /// The bound needs `|d2E/dx_i^2|` — the stiffest curvature one atom's displacement can
    /// meet. For a single triple, with `E = F(s_a, s_b, s_c)` a function of the three
    /// sides,
    ///
    /// ```text
    /// d2F/dx_i^2 = sum_{a,b} F_ab (ds_a/dx_i)(ds_b/dx_i) + sum_a F_a (d2 s_a/dx_i^2)
    /// ```
    ///
    /// Atom `i` touches exactly TWO of the three sides, `|ds_a/dx_i| <= 1` because each is
    /// a component of a unit vector, and `||d2 s_a/dx_i^2|| <= 2/s_a` for a distance. So
    /// per triple
    ///
    /// ```text
    /// |d2F/dx_i^2| <= 4 G2 + 2 sum_a |F_a| / s_a
    /// ```
    ///
    /// with the second sum taken over ALL THREE sides rather than the two at `i`, which
    /// only widens it. `G2` is the table's own second-derivative envelope, and it is taken
    /// as the SMALLER of the table's absolute cap and its local one,
    /// `curvature_per_gradient * max_a |F_a|` — both measured from the interpolant and
    /// widened. Taking the local form is what keeps the bound a reading of the
    /// configuration: a dispersed scene has tiny three-body gradients, so a tiny
    /// three-body stiffness, where the absolute cap alone would quote the compact corner's
    /// number forever.
    ///
    /// Curvatures ADD over the triples an atom belongs to, so the force loop accumulates
    /// the per-triple bound into a per-atom total and keeps the largest. Bounding instead
    /// by `C(n-1, 2)` times the worst single triple — every triple simultaneously at the
    /// worst geometry — is valid and was the first form written here; it is a factor of
    /// tens looser on any scene that is not a single compact droplet, which is a bound
    /// that cannot fail rather than a bound that says anything.
    ///
    /// Zero when no table is loaded or the scene has fewer than three atoms, so the pair
    /// bound is returned unchanged — adding an exact zero to a finite float changes no bit.
    pub fn k_three(&self) -> f64 {
        if !self.trimer.loaded || self.n < 3 {
            return 0.0;
        }
        self.k_three_max
    }

    /// The same number, exposed under the `_max` name for the attribution probe, so it can
    /// separate the two halves of the drift bound the way it separates `k_pair_max`.
    pub fn k_three_max(&self) -> f64 {
        self.k_three_max
    }

    /// Total energy currently held by the scene.
    pub fn energy(&self) -> f64 {
        self.e_kin + self.e_pair + self.e_three + self.e_wall + self.e_spring
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
        // Reachable curvature (the envelope, from the largest pair energy seen) OR
        // VISITED curvature (the running max the force loop has actually evaluated),
        // whichever is larger. The envelope is normally the bigger of the two, but it is
        // refreshed from pair energies sampled at grain BOUNDARIES, so a brief excursion
        // between two boundaries can be stiffer than anything the envelope knows about.
        // `k_pair_max` costs nothing — the force loop already computes every curvature it
        // maximises over — and it closes that gap.
        let mu = 0.5 * M_H;
        // The three-body stiffness is ADDED to the pair envelope rather than maxed with
        // it: both potentials act on the same coordinate, so their curvatures add, and a
        // max would understate the sum. With no table loaded `k_three()` is an exact zero
        // and the sum is bit-for-bit the pair bound this line computed before.
        let k = self.timescale.k_env.max(self.k_pair_max) + self.k_three();
        let mut omega_sq: f64 = k / mu;
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
        let (px, py, pz) = self.momentum();
        let dx = px - self.p0.0 - self.j_ext.0;
        let dy = py - self.p0.1 - self.j_ext.1;
        let dz = pz - self.p0.2 - self.j_ext.2;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Roundoff bound for the momentum ledger. Each step commits O(N) floating-point
    /// additions into the momentum sum, each carrying at most one unit in the last place
    /// of the running magnitude; accumulating those worst-case (rather than as the
    /// random walk they actually are) gives `8 * steps * eps * p_scale`.
    pub fn momentum_bound(&self) -> f64 {
        let mut p_scale: f64 = 0.0;
        for i in 0..self.n {
            let a = &self.atoms[i];
            p_scale += M_H * (a.vx * a.vx + a.vy * a.vy + a.vz * a.vz).sqrt();
        }
        let p_scale = p_scale.max(1e-12);
        8.0 * (self.steps.max(1) as f64) * f64::EPSILON * p_scale
    }

    pub fn momentum_gate(&self) -> bool {
        self.momentum_residual_peak <= self.momentum_bound()
    }

    pub fn momentum(&self) -> (f64, f64, f64) {
        let mut px = 0.0;
        let mut py = 0.0;
        let mut pz = 0.0;
        for i in 0..self.n {
            px += M_H * self.atoms[i].vx;
            py += M_H * self.atoms[i].vy;
            pz += M_H * self.atoms[i].vz;
        }
        (px, py, pz)
    }

    /// Kinetic temperature by equipartition: `E_kin = (dof/2) N k_B T`.
    ///
    /// DIMENSION-DEPENDENT, and one of only two places in this file that is. The
    /// degrees of freedom are the scene's, not the state vector's: a 2D scene has two
    /// per atom even though the integrator carries three components, because the third
    /// is frozen at zero and a frozen coordinate holds no thermal energy. At `dof = 2`
    /// the factor `0.5 * dof` is exactly `1.0`, so the 2D reading is the same float it
    /// has always been.
    pub fn temperature(&self) -> f64 {
        if self.n == 0 {
            return 0.0;
        }
        self.e_kin / (0.5 * self.dims.dof() * self.n as f64 * K_B)
    }

    /// Place `n` atoms and zero the ledger. Deterministic: no RNG, so a reported run can
    /// be re-run byte-for-byte.
    pub fn reset(&mut self, n: usize) {
        self.n = n.clamp(0, MAX_ATOMS);
        self.grabbed = None;
        self.thermostat_on = false;
        let cx = 0.5 * self.width;
        let cy = 0.5 * self.height;
        // The mid-plane. In `Dims::Two` this is the plane the whole scene lives on and
        // never leaves; in `Dims::Three` it is just the box's centre.
        let cz = 0.5 * self.depth;
        let three = self.dims == Dims::Three;
        for i in 0..self.n {
            let a = &mut self.atoms[i];
            a.z = cz;
            a.vz = 0.0;
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
            } else if three {
                // The 3D counterpart of the ring: a deterministic Fibonacci SPHERE.
                // No RNG, so a reported run re-runs byte-for-byte, and near-uniform
                // spacing so no pair opens inside the repulsive wall. Velocities are
                // assigned by the expansion pass below the loop — a shell at REST
                // opens with every pair already reading BONDED (E_rel = U(R) < 0 at
                // any finite separation, and exactly ON its turning point besides),
                // which a field report rightly called out: an opener that hands out
                // bonds nobody paid for contradicts the capture plant's own lesson.
                //
                // 6 bohr is the ring's radius kept: at N = 16 the nearest-neighbour
                // spacing is ~3.4 bohr, comfortably outside the wall and inside the well.
                let n = self.n as f64;
                let golden = core::f64::consts::PI * (3.0 - 5.0f64.sqrt());
                let w = 1.0 - 2.0 * (i as f64 + 0.5) / n;
                let rho = (1.0 - w * w).max(0.0).sqrt();
                let phi = (i as f64) * golden;
                let radius = 6.0;
                a.x = cx + radius * rho * phi.cos();
                a.y = cy + radius * rho * phi.sin();
                a.z = cz + radius * w;
                a.vx = 0.0;
                a.vy = 0.0;
            } else {
                // A deterministic ring; velocities come from the expansion pass below.
                let theta = (i as f64) * core::f64::consts::TAU / (self.n as f64);
                let radius = 6.0;
                a.x = cx + radius * theta.cos();
                a.y = cy + radius * theta.sin();
                a.vx = 0.0;
                a.vy = 0.0;
            }
        }
        // THE OPENER HANDS OUT NO BONDS. For n > 2 the scene opens in uniform
        // (Hubble-style) expansion about the centre: v_i = v * (x_i - c) / R, so
        // every pairwise separation grows in proportion and pair (i, j)'s relative
        // speed is v * d_ij / R. Unboundness for every pair needs
        //     0.5 * mu * (v * d / R)^2 > |U(d)|,
        // and v is DERIVED from that inequality by scanning the actual opening
        // pairs against the loaded curve — worst pair wins, margin 1.5 on the
        // speed. No fitted constant, no distance cutoff, deterministic; and being
        // strictly unbound also clears the measure-zero at-rest boundary where a
        // pair sits exactly ON its outer turning point and the strict criterion
        // falls by solver rounding. Bonds then cost what they always cost: energy,
        // paid out through a third body, the spring, or the thermostat.
        //
        // The two-atom headline scene keeps its own deliberate approach and is
        // untouched; with no curve loaded there is no U to clear and the scene
        // stays at rest (there are no forces either).
        if self.n > 2 && self.table.is_loaded() {
            let mu = 0.5 * M_H;
            let shell_r = 6.0; // both openers place atoms on a 6-bohr shell
            let mut v2_needed = 0.0f64;
            for i in 0..self.n {
                for j in (i + 1)..self.n {
                    let dx = self.atoms[j].x - self.atoms[i].x;
                    let dy = self.atoms[j].y - self.atoms[i].y;
                    let dz = self.atoms[j].z - self.atoms[i].z;
                    let d2 = dx * dx + dy * dy + dz * dz;
                    let d = d2.sqrt().max(1e-9);
                    let u = self.table.u(d);
                    if u < 0.0 {
                        v2_needed = v2_needed.max(2.0 * (-u) * shell_r * shell_r / (mu * d2));
                    }
                }
            }
            let v = 1.5 * v2_needed.sqrt();
            for i in 0..self.n {
                let a = &mut self.atoms[i];
                a.vx = v * (a.x - cx) / shell_r;
                a.vy = v * (a.y - cy) / shell_r;
                a.vz = v * (a.z - cz) / shell_r;
            }
        }
        self.zero_ledger();
    }

    fn zero_ledger(&mut self) {
        self.w_ext = 0.0;
        self.j_ext = (0.0, 0.0, 0.0);
        self.time = 0.0;
        self.steps = 0;
        self.frame = 0;
        self.k_pair_max = 0.0;
        self.k_three_max = 0.0;
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
        self.e_ref = self.mode_energy().max(self.table.d_e.abs());
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

        let e_now = self.mode_energy();
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

    /// Set an atom's in-plane velocity, leaving `vz` alone. On the mid-plane `vz` is
    /// zero and stays zero, which is what keeps a scripted 2D scene two-dimensional.
    pub fn set_velocity(&mut self, i: usize, vx: f64, vy: f64) {
        if i < self.n {
            self.atoms[i].vx = vx;
            self.atoms[i].vy = vy;
        }
    }

    pub fn set_velocity_3d(&mut self, i: usize, vx: f64, vy: f64, vz: f64) {
        if i < self.n {
            self.atoms[i].vx = vx;
            self.atoms[i].vy = vy;
            self.atoms[i].vz = vz;
        }
    }

    /// Set an atom's in-plane position, leaving `z` alone — same reasoning as
    /// [`Sim::set_velocity`].
    pub fn set_position(&mut self, i: usize, x: f64, y: f64) {
        if i < self.n {
            self.atoms[i].x = x;
            self.atoms[i].y = y;
        }
    }

    pub fn set_position_3d(&mut self, i: usize, x: f64, y: f64, z: f64) {
        if i < self.n {
            self.atoms[i].x = x;
            self.atoms[i].y = y;
            self.atoms[i].z = z;
        }
    }

    /// Freeze the current state as the ledger's origin. Call after any scripted setup
    /// that is meant to be the initial condition rather than an intervention.
    pub fn rebase(&mut self) {
        self.zero_ledger();
    }

    // ---------------------------------------------------------------- forces

    /// The soft quadratic box: `U = K_WALL * d^2 / 2` per face the atom has passed.
    ///
    /// The z faces are applied UNCONDITIONALLY, with no `dims` branch, and that is the
    /// lift's load-bearing simplification rather than an oversight. A 2D scene sits at
    /// `z = depth/2`, which is inside `[inset, depth - inset]` for any box deeper than
    /// twice the inset, so neither z branch is taken, `u` and `fz` keep the exact zeros
    /// they were initialised with, and `touched` is decided by x and y alone. The 2D
    /// wall energy is therefore the same float it was before the box grew a lid — and
    /// the box needs no mode flag to know which world it is in.
    fn wall_energy_force(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64, f64, bool) {
        if self.boundary == Boundary::Open {
            return (0.0, 0.0, 0.0, 0.0, false);
        }
        let lo = self.wall_inset;
        let hi_x = self.width - self.wall_inset;
        let hi_y = self.height - self.wall_inset;
        let hi_z = self.depth - self.wall_inset;
        let mut u = 0.0;
        let mut fx = 0.0;
        let mut fy = 0.0;
        let mut fz = 0.0;
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
        if z < lo {
            let d = lo - z;
            u += 0.5 * K_WALL * d * d;
            fz += K_WALL * d;
            touched = true;
        } else if z > hi_z {
            let d = z - hi_z;
            u += 0.5 * K_WALL * d * d;
            fz -= K_WALL * d;
            touched = true;
        }
        (u, fx, fy, fz, touched)
    }

    /// Recompute `a_pair` and `a_ext` from the current positions, and refresh the
    /// potential terms of the ledger. Split so the momentum ledger can tell the
    /// internal forces (which cancel) from the external ones (which do not).
    fn compute_forces(&mut self) {
        for i in 0..self.n {
            self.a_pair[i] = (0.0, 0.0, 0.0);
            self.a_ext[i] = (0.0, 0.0, 0.0);
        }
        let mut e_pair = 0.0;
        let mut k_pair_max = self.k_pair_max;

        for i in 0..self.n {
            for j in (i + 1)..self.n {
                let dx = self.atoms[j].x - self.atoms[i].x;
                let dy = self.atoms[j].y - self.atoms[i].y;
                let dz = self.atoms[j].z - self.atoms[i].z;
                // `(xx + yy) + zz`, in that order: on the mid-plane `zz` is an exact
                // zero and adding it changes no bit of the 2D result.
                let r2 = dx * dx + dy * dy + dz * dz;
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
                let fz = f_over_r * dz;
                // Newton's third law, applied as one computed value with opposite signs:
                // this is what makes the pair contribution cancel from the momentum sum.
                self.a_pair[i].0 += fx;
                self.a_pair[i].1 += fy;
                self.a_pair[i].2 += fz;
                self.a_pair[j].0 -= fx;
                self.a_pair[j].1 -= fy;
                self.a_pair[j].2 -= fz;
                let ac = curv.abs();
                if ac > k_pair_max {
                    k_pair_max = ac;
                }
            }
        }
        self.k_pair_max = k_pair_max;
        self.e_pair = e_pair;

        self.accumulate_three_body();

        let mut e_wall = 0.0;
        for i in 0..self.n {
            let (u, fx, fy, fz, touched) =
                self.wall_energy_force(self.atoms[i].x, self.atoms[i].y, self.atoms[i].z);
            e_wall += u;
            self.a_ext[i].0 += fx;
            self.a_ext[i].1 += fy;
            self.a_ext[i].2 += fz;
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
                let dz = self.atoms[g].z - self.anchor.2;
                self.e_spring = 0.5 * K_SPRING * (dx * dx + dy * dy + dz * dz);
                self.a_ext[g].0 += -K_SPRING * dx;
                self.a_ext[g].1 += -K_SPRING * dy;
                self.a_ext[g].2 += -K_SPRING * dz;
                self.spring_engaged = true;
            }
        }
    }

    /// THE MANY-BODY SECTOR: the tabulated three-body term over every triple, and the
    /// forces it exerts.
    ///
    /// Nothing here is a new constant. The value comes from the interpolant, the three
    /// side-derivatives come from differentiating that same interpolant analytically, and
    /// the force on each atom is assembled from them by the chain rule
    /// `dE/dx_i = sum_a (dE/ds_a)(ds_a/dx_i)`, where `ds_a/dx_i` is a unit vector along
    /// the side. Each side contributes to its TWO atoms as one computed value with
    /// opposite signs — exactly the shape the pair loop uses — so the triple's total force
    /// is zero by construction and the momentum ledger has nothing new to subtract.
    ///
    /// The accelerations go into `a_pair`, which holds INTERNAL forces (those that cancel
    /// from the momentum sum) as opposed to `a_ext` (walls, spring, thermostat). The
    /// energy is kept in its own ledger row.
    ///
    /// A triple whose middle side is past the table's domain returns an exact zero and
    /// costs one comparison; in a dispersed gas that is almost every triple, which is what
    /// keeps the N^3 loop from being the whole budget when there is nothing to compute.
    fn accumulate_three_body(&mut self) {
        self.e_three = 0.0;
        if !self.trimer.loaded || self.n < 3 {
            return;
        }
        // One distance matrix, read three times per triple instead of nine square roots.
        // Indexed rather than iterated: each separation is written to BOTH `d[i][j]` and
        // `d[j][i]`, which an iterator over one row cannot express.
        let mut d = [[0.0f64; MAX_ATOMS]; MAX_ATOMS];
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                let dx = self.atoms[j].x - self.atoms[i].x;
                let dy = self.atoms[j].y - self.atoms[i].y;
                let dz = self.atoms[j].z - self.atoms[i].z;
                let r = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-9);
                d[i][j] = r;
                d[j][i] = r;
            }
        }
        let env_abs = self.trimer.curvature_envelope;
        let env_per_grad = self.trimer.curvature_per_gradient;
        let mut e_three = 0.0;
        // Per-atom stiffness totals: curvatures ADD over the triples an atom is in.
        let mut k_atom = [0.0f64; MAX_ATOMS];
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                for k in (j + 1)..self.n {
                    let (rij, rik, rjk) = (d[i][j], d[i][k], d[j][k]);
                    let (v, g) = self.trimer.eval([rij, rik, rjk]);
                    if v == 0.0 && g[0] == 0.0 && g[1] == 0.0 && g[2] == 0.0 {
                        continue;
                    }
                    e_three += v;
                    self.push_side(i, j, g[0], rij);
                    self.push_side(i, k, g[1], rik);
                    self.push_side(j, k, g[2], rjk);
                    // The per-triple stiffness the drift bound is built from; the
                    // derivation is in `Sim::k_three`.
                    let gmax = g[0].abs().max(g[1].abs()).max(g[2].abs());
                    let g2 = env_abs.min(env_per_grad * gmax);
                    let kt = 4.0 * g2
                        + 2.0 * (g[0].abs() / rij + g[1].abs() / rik + g[2].abs() / rjk);
                    k_atom[i] += kt;
                    k_atom[j] += kt;
                    k_atom[k] += kt;
                }
            }
        }
        self.e_three = e_three;
        for k in k_atom[..self.n].iter() {
            if *k > self.k_three_max {
                self.k_three_max = *k;
            }
        }
    }

    /// One side's share of a triple's force, applied equal and opposite. `g` is
    /// `dE/dr_ab`, the same convention the pair loop's `slope` carries, so the sign logic
    /// is the one line it already is there and not a second one to keep true.
    #[inline]
    fn push_side(&mut self, a: usize, b: usize, g: f64, r: f64) {
        let f_over_r = g / r;
        let fx = f_over_r * (self.atoms[b].x - self.atoms[a].x);
        let fy = f_over_r * (self.atoms[b].y - self.atoms[a].y);
        let fz = f_over_r * (self.atoms[b].z - self.atoms[a].z);
        self.a_pair[a].0 += fx;
        self.a_pair[a].1 += fy;
        self.a_pair[a].2 += fz;
        self.a_pair[b].0 -= fx;
        self.a_pair[b].1 -= fy;
        self.a_pair[b].2 -= fz;
    }

    fn accumulate_energy(&mut self) {
        let mut e_kin = 0.0;
        for i in 0..self.n {
            let a = &self.atoms[i];
            e_kin += 0.5 * M_H * (a.vx * a.vx + a.vy * a.vy + a.vz * a.vz);
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
        let mut jz = 0.0;
        for i in 0..self.n {
            let (px, py, pz) = self.a_pair[i];
            let (ex, ey, ez) = self.a_ext[i];
            self.atoms[i].vx += half * (px + ex);
            self.atoms[i].vy += half * (py + ey);
            self.atoms[i].vz += half * (pz + ez);
            jx += 0.5 * dt * ex;
            jy += 0.5 * dt * ey;
            jz += 0.5 * dt * ez;
        }

        for i in 0..self.n {
            self.atoms[i].x += dt * self.atoms[i].vx;
            self.atoms[i].y += dt * self.atoms[i].vy;
            self.atoms[i].z += dt * self.atoms[i].vz;
        }

        self.compute_forces();

        for i in 0..self.n {
            let (px, py, pz) = self.a_pair[i];
            let (ex, ey, ez) = self.a_ext[i];
            self.atoms[i].vx += half * (px + ex);
            self.atoms[i].vy += half * (py + ey);
            self.atoms[i].vz += half * (pz + ez);
            jx += 0.5 * dt * ex;
            jy += 0.5 * dt * ey;
            jz += 0.5 * dt * ez;
        }
        self.j_ext.0 += jx;
        self.j_ext.1 += jy;
        self.j_ext.2 += jz;

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
        // The amplitude factor is tracked here, not only at boundaries, for the same
        // reason and at the same price: a collision that peaks between two boundaries
        // raises the mode energy the bound has to cover, and a boundary sample of it
        // would miss exactly the events that matter.
        let m = self.mode_energy();
        if m > self.e_ref {
            self.e_ref = m;
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
        let (pbx, pby, pbz) = self.momentum();
        for i in 0..self.n {
            self.atoms[i].vx *= lambda;
            self.atoms[i].vy *= lambda;
            self.atoms[i].vz *= lambda;
        }
        self.accumulate_energy();
        self.w_ext += self.e_kin - before;
        let (pax, pay, paz) = self.momentum();
        self.j_ext.0 += pax - pbx;
        self.j_ext.1 += pay - pby;
        self.j_ext.2 += paz - pbz;
    }

    // ---------------------------------------------------------------- the hand

    /// Grab atom `i`. The anchor starts ON the atom, so the spring enters the ledger at
    /// zero extension and the grab itself injects nothing.
    pub fn grab(&mut self, i: usize) {
        if i >= self.n {
            return;
        }
        self.grabbed = Some(i);
        self.anchor = (self.atoms[i].x, self.atoms[i].y, self.atoms[i].z);
        self.spring_engaged = true;
        self.compute_forces();
    }

    /// Move the anchor. The spring is a term in the Hamiltonian with a time-dependent
    /// parameter; moving that parameter at fixed atom position changes the stored spring
    /// energy by exactly `dU`, and `dU` IS the work the user's hand did. Posting it here
    /// is what keeps `E - W_ext` constant through a drag, with no path integral to
    /// approximate and no second-order error of its own.
    ///
    /// The 2D form holds the anchor's z, which on the mid-plane is the atom's own z, so
    /// `dz` stays an exact zero and the work posted is the float it always was.
    pub fn move_anchor(&mut self, x: f64, y: f64) {
        self.move_anchor_3d(x, y, self.anchor.2);
    }

    /// [`Sim::move_anchor`] with the third component. The work accounting is identical —
    /// it is `dU` of one spring term either way.
    pub fn move_anchor_3d(&mut self, x: f64, y: f64, z: f64) {
        let Some(g) = self.grabbed else { return };
        if g >= self.n {
            return;
        }
        let before = self.e_spring;
        self.anchor = (x, y, z);
        let dx = self.atoms[g].x - x;
        let dy = self.atoms[g].y - y;
        let dz = self.atoms[g].z - z;
        let after = 0.5 * K_SPRING * (dx * dx + dy * dy + dz * dz);
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
                let dz = self.atoms[j].z - self.atoms[i].z;
                let r = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-9);
                let vx = self.atoms[j].vx - self.atoms[i].vx;
                let vy = self.atoms[j].vy - self.atoms[i].vy;
                let vz = self.atoms[j].vz - self.atoms[i].vz;
                let ke_rel = 0.5 * mu * (vx * vx + vy * vy + vz * vz);
                let u = self.table.u(r);
                let e_rel = ke_rel + u;
                // |L|^2 of the relative motion, for the centrifugal term. In 3D the
                // relative motion of an isolated pair is planar but the plane is not the
                // scene's, so the full cross product is needed — and it costs the 2D case
                // nothing, because on the mid-plane `dz` and `vz` are exact zeros, the
                // two transverse components are exactly `0.0`, and `l_sq` reduces to the
                // `L_z^2` this line used to compute, bit for bit.
                let lx = mu * (dy * vz - dz * vy);
                let ly = mu * (dz * vx - dx * vz);
                let lz = mu * (dx * vy - dy * vx);
                let l_sq = lx * lx + ly * ly + lz * lz;
                let r_outer = self
                    .table
                    .outer_turning_point(e_rel, l_sq, mu, r, TURNING_POINT_CAP);
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

    /// The CLUSTER reading: connected components of the bonded-pair graph.
    ///
    /// `bonded_count()` counts PAIRS, and the pair criterion is deliberately two-body:
    /// a pair reads BONDED when that pair, considered alone, is a bound system. Both
    /// facts are correct and together they mislead — 16 atoms collapsed into one cold
    /// droplet read 120 BONDED, because every one of the C(16,2) pairs genuinely is
    /// mutually bound (delete the other fourteen atoms and any pair you kept would stay
    /// bound; the 12-bohr tail of the well is still ~6e-6 Ha deep, so a cold pair at any
    /// separation the box allows has `E_rel < 0`). A field screenshot asking "16 atoms,
    /// 120 bonds?" is what surfaced the mismatch between the number and the noun.
    ///
    /// The chemically meaningful headline object is the component, not the edge: that
    /// droplet is ONE cluster of 16 atoms. This reading introduces no new criterion —
    /// the edge set is exactly the pairs already reading `bonded`, so it cannot disagree
    /// with the pair layer, and there is still no distance cutoff and no fitted
    /// threshold anywhere. Union-find with path halving; components of one atom are
    /// free atoms, not clusters.
    ///
    /// Returns `(clusters, atoms_in_clusters)`. Distinct from the census's MOLECULE
    /// count on purpose: a cluster is a statement about boundness, a molecule row is a
    /// statement about closure, and how far those disagree (the droplet: one cluster,
    /// few or no closed pair-composites, rejections climbing) is the boundness-vs-
    /// closure fence made visible.
    pub fn cluster_count(&self) -> (usize, usize) {
        let size = self.cluster_sizes();
        let clusters = size[..self.n].iter().filter(|&&s| s >= 2).count();
        let atoms = size[..self.n].iter().filter(|&&s| s >= 2).sum();
        (clusters, atoms)
    }

    /// The component SIZES behind [`Sim::cluster_count`], indexed by the component's root
    /// atom: entry `i` is the number of atoms in the component rooted at `i`, and zero for
    /// an atom that is not a root. Entries of 1 are free atoms; entries of 2 or more are
    /// clusters.
    ///
    /// Split out rather than duplicated so the quench's histogram and the headline count
    /// read ONE union-find over ONE edge set. Two implementations of a cluster reading is
    /// how the two of them come to disagree.
    pub fn cluster_sizes(&self) -> [usize; MAX_ATOMS] {
        let mut parent: [usize; MAX_ATOMS] = [0; MAX_ATOMS];
        for (i, p) in parent.iter_mut().enumerate() {
            *p = i;
        }
        fn find(parent: &mut [usize; MAX_ATOMS], mut i: usize) -> usize {
            while parent[i] != i {
                parent[i] = parent[parent[i]]; // path halving
                i = parent[i];
            }
            i
        }
        for p in self.pairs[..self.pair_count].iter().filter(|p| p.bonded) {
            let (a, b) = (find(&mut parent, p.i), find(&mut parent, p.j));
            if a != b {
                parent[a] = b;
            }
        }
        let mut size = [0usize; MAX_ATOMS];
        for i in 0..self.n {
            size[find(&mut parent, i)] += 1;
        }
        size
    }
}
