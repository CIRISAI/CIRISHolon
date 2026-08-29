//! The world: one [`Sim`] from `holon-render`, wrapped as a Bevy resource, plus the
//! frame advance and the on-load calibration burst.
//!
//! NO rendering types appear here. This module compiles in the `headless` build, which
//! is what CI and `tests/headless.rs` link, and that is deliberate: the gates are a
//! property of the physics, so the test that asserts them must be able to run with no
//! GPU, no window and no wgpu in the dependency graph.
//!
//! Nothing in this file integrates anything. Every line that moves an atom is in
//! `holon-render`; what is here is the *scene* (a box, a count, a curve) and the
//! *schedule* (how many substeps this frame is allowed), both of which are the shell's
//! business and neither of which is physics.

use bevy::ecs::resource::Resource;
use holon_render::clock::{Rung, AU_TO_FS};
use holon_render::sim::{Boundary, Dims, Sim, MAX_ATOMS};

/// The box, in bohr. A CUBE, unlike the canvas shell's 40 x 24 letterbox: a 3D scene has
/// no preferred axis to be wide along, and an orbiting camera looks at the short side as
/// often as the long one.
pub const BOX_SIDE: f64 = 24.0;

/// Drawn atom radius, bohr. This is the same number as [`Sim::wall_inset`] on purpose —
/// the walls act on atom centres inset by exactly the drawn radius, so the picture and
/// the physics agree about where the edge of the box is. At the H2 equilibrium of
/// 1.4 bohr two bonded spheres of this radius very nearly touch, which is the correct
/// reading of a bond and not a coincidence worth tuning away.
pub const ATOM_RADIUS: f64 = 0.6;

/// The separation range and knot count the curve is generated on — the same request the
/// browser shell makes, so the two shells are demonstrably showing one curve.
pub const CURVE_R_MIN: f64 = 0.3;
pub const CURVE_R_MAX: f64 = 10.0;
pub const CURVE_KNOTS: usize = 492;

/// Substeps in the calibration burst. Large enough that the measurement is not dominated
/// by the clock's own resolution, small enough not to stall the first frame.
pub const CALIBRATION_SUBSTEPS: u32 = 20_000;

/// Where the device measurement has got to. The burst runs ONCE, on a frame of its own,
/// and the result is the authority for `N_max` on this device — never a projection, and
/// never this developer's machine.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Calibration {
    /// Not yet run. The next frame runs it.
    Pending,
    /// Run and recorded.
    Done,
    /// The host has no usable monotonic clock, so nothing was recorded. Reported rather
    /// than faked: an uncalibrated `substep_budget` returns "as many as asked for", which
    /// is the honest answer when capacity is unknown.
    Unavailable,
}

/// The atom world, as the Bevy app sees it.
#[derive(Resource)]
pub struct AtomWorld {
    pub sim: Sim,
    pub calibration: Calibration,
    /// Substeps the last frame actually took, for the clocks readout.
    pub last_substeps: u32,
    /// Measured wall interval of the last frame, seconds.
    pub last_frame_seconds: f64,
    /// Set when the curve failed to generate; the HUD says so instead of drawing a
    /// scene whose forces came from nowhere.
    pub table_status: u32,
}

impl AtomWorld {
    /// Build the scene: generate the exact curve, adopt the clocks it implies, put the
    /// atoms in a cube.
    ///
    /// `holon_chem` solves H2 in the STO-3G basis exactly (full CI) from closed-form
    /// Gaussian integrals and differentiates it analytically, here, at startup. The
    /// shell does not play a curve somebody computed; it solves the one it is showing.
    pub fn new(atoms: usize) -> Self {
        let mut sim = Sim::empty();
        let status = holon_render::generate_table(&mut sim, CURVE_R_MIN, CURVE_R_MAX, CURVE_KNOTS);
        // THE THIRD BODY MUST PAY HERE TOO. `generate_trimer_table` was written with
        // this exact caller named in its doc comment — and nothing called it, so the
        // 3D shell ran pair-only physics while the 2D shell ran MBE3, and a field
        // screenshot of a compact linked H4 is what surfaced it. That is standing
        // question 1 (is the thing that passes the thing that RUNS?) failing in
        // production the same night it was written down. The headless suite now
        // asserts the table is loaded INSIDE this constructor's product, per the
        // question's own enforcement rule — and that gate demonstrated its failing
        // case against this exact line removed, before it was trusted.
        if status == holon_render::TABLE_OK {
            holon_render::generate_trimer_table(&mut sim);
        }
        sim.dims = Dims::Three;
        sim.boundary = Boundary::Walls;
        sim.width = BOX_SIDE;
        sim.height = BOX_SIDE;
        sim.depth = BOX_SIDE;
        sim.wall_inset = ATOM_RADIUS;
        sim.reset(atoms.clamp(2, MAX_ATOMS));
        Self {
            sim,
            calibration: Calibration::Pending,
            last_substeps: 0,
            last_frame_seconds: 0.0,
            table_status: status,
        }
    }

    pub fn table_ok(&self) -> bool {
        self.table_status == holon_render::TABLE_OK && self.sim.table.is_loaded()
    }

    /// Advance one frame of `wall_dt` MEASURED wall-seconds and close the grain.
    ///
    /// The interval is the one the frame actually took; nothing here assumes 60 Hz, or
    /// any Hz. dt is never stretched to fit — a shortfall comes back as dilation on the
    /// rung readout, which is the tuner's contract and not this function's decision.
    pub fn advance(&mut self, wall_dt: f64) {
        if !self.table_ok() {
            return;
        }
        let budget = self.sim.timescale.substep_budget(wall_dt);
        let plan = self.sim.timescale.plan_frame(wall_dt, budget);
        self.sim.step_frame(plan.substeps);
        self.last_substeps = plan.substeps;
        self.last_frame_seconds = wall_dt;
    }

    /// Run `substeps` of PURE PHYSICS at the calibration scene (N = MAX_ATOMS, walls
    /// off, no grain closure), then restore the caller's scene. Mirrors
    /// `holon_calibration_burst`; the caller times it and calls
    /// [`AtomWorld::record_calibration`].
    pub fn calibration_burst(&mut self, substeps: u32) {
        let restore_n = self.sim.n;
        let restore_boundary = self.sim.boundary;
        self.sim.boundary = Boundary::Open;
        self.sim.reset(MAX_ATOMS);
        for _ in 0..substeps {
            self.sim.step();
        }
        self.sim.boundary = restore_boundary;
        self.sim.reset(restore_n);
    }

    pub fn record_calibration(&mut self, substeps_per_second: f64) {
        if substeps_per_second.is_finite() && substeps_per_second > 0.0 {
            self.sim.timescale.substeps_per_second = substeps_per_second;
            self.sim.timescale.calibrated = true;
            self.calibration = Calibration::Done;
        } else {
            self.calibration = Calibration::Unavailable;
        }
    }

    /// Pair evaluations per second on this device: the calibration rate times the pair
    /// count of the calibration scene. This is what the O(N^2) force loop spends.
    pub fn pairs_per_second(&self) -> f64 {
        let pairs = (MAX_ATOMS * (MAX_ATOMS - 1) / 2) as f64;
        self.sim.timescale.substeps_per_second * pairs
    }

    /// Largest atom count this device sustains at the current sim-speed and accuracy.
    pub fn n_max(&self) -> f64 {
        if !self.sim.timescale.calibrated {
            return MAX_ATOMS as f64;
        }
        holon_render::clock::n_max(
            self.pairs_per_second(),
            self.sim.timescale.required_substeps_per_second(),
        )
    }

    /// Reset to `n` atoms, CLAMPED to what this device was measured to sustain. An atom
    /// count the device cannot carry would be delivered as time dilation, which is a
    /// worse answer than saying so.
    pub fn reset(&mut self, n: usize) {
        let cap = (self.n_max() as usize).max(2);
        self.sim.reset(n.clamp(2, MAX_ATOMS).min(cap));
    }

    /// Wall-seconds one vibration takes at the current sim-speed — the clock-3 reading a
    /// person can actually check against the picture.
    pub fn wall_seconds_per_vibration(&self) -> f64 {
        let fs = self.sim.timescale.period * AU_TO_FS;
        let speed = self.sim.timescale.sim_speed_fs_per_wallsec;
        if speed > 0.0 {
            fs / speed
        } else {
            f64::INFINITY
        }
    }

    /// The degradation rung, in the words the overlay uses.
    pub fn rung_label(&self) -> (&'static str, &'static str) {
        match self.sim.timescale.rung {
            Rung::Exact => ("EXACT", "requested speed delivered at the derived dt"),
            Rung::TimeDilated => (
                "TIME DILATED",
                "accuracy held; there are fewer steps per second",
            ),
            Rung::AccuracyDeclared => (
                "ACCURACY DECLARED",
                "dt grew by your toggle; the bound below is the re-derived one",
            ),
            Rung::Refused => (
                "REFUSED",
                "omega*dt reached the stability limit; nothing lawful remains",
            ),
        }
    }
}
