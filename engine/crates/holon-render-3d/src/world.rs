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
use holon_chem::elements::{
    Species, CARBON, CHLORINE, FLUORINE, HELIUM, HYDROGEN, LITHIUM, NEON, NITROGEN, OXYGEN,
};
use holon_render::clock::{Rung, AU_TO_FS};
use holon_render::sim::{Boundary, Dims, Sim, MAX_ATOMS};

/// Scene preset selector for the 3D atom world.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Preset {
    /// 2-atom H₂ approach and collision.
    H2,
    /// 16-atom quench gas cluster with 3-body saturation active.
    Quench16,
    /// LiH: Lithium Hydride (Li + H).
    LiH,
    /// HF: Hydrogen Fluoride (H + F).
    HF,
    /// Li₂: Dilithium (Li + Li).
    Li2,
    /// N₂: Dinitrogen (N + N).
    N2,
    /// F₂: Difluorine (F + F).
    F2,
    /// CO: Carbon Monoxide (C + O).
    CO,
    /// He₂: Helium dimer (closed-shell negative control, non-binding).
    He2,
    /// Ne₂: Neon dimer (closed-shell negative control, non-binding).
    Ne2,
    /// HCl: hydrogen chloride, MIXTURES-1's own heteronuclear dimer. Two species, one
    /// curve — the smallest scene the pair-table bank is needed for.
    HCl,
    /// A MIXED GAS: eight hydrogens and eight heliums in the box. THREE pair types at
    /// once — H-H, H-He and He-He — so the bank is dispatching on every force evaluation,
    /// and the reading is vivid: the hydrogens pair off and the heliums bind to nothing,
    /// in one scene, under one integrator.
    ///
    /// Helium rather than chlorine because a preset has to LOAD: Cl2 is eighteen basis
    /// functions and about a hundred seconds of solve, which is not an interactive scene
    /// change. The campaign's own H + Cl gas is the `mixquench` example, where the curves
    /// are generated once and the run is detached.
    MixedGas,
}

impl Preset {
    pub const ALL: [Preset; 12] = [
        Preset::H2,
        Preset::Quench16,
        Preset::LiH,
        Preset::HF,
        Preset::Li2,
        Preset::N2,
        Preset::F2,
        Preset::CO,
        Preset::He2,
        Preset::Ne2,
        Preset::HCl,
        Preset::MixedGas,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Preset::H2 => "H₂ (2 atoms, approach)",
            Preset::Quench16 => "16-H Quench Gas (3-body active)",
            Preset::LiH => "LiH (Lithium Hydride)",
            Preset::HF => "HF (Hydrogen Fluoride)",
            Preset::Li2 => "Li₂ (Dilithium)",
            Preset::N2 => "N₂ (Dinitrogen)",
            Preset::F2 => "F₂ (Difluorine)",
            Preset::CO => "CO (Carbon Monoxide)",
            Preset::He2 => "He₂ (Closed-shell bounce)",
            Preset::Ne2 => "Ne₂ (Closed-shell bounce)",
            Preset::HCl => "HCl (Hydrogen Chloride, 2 species)",
            Preset::MixedGas => "Mixed Gas: 8 H + 8 He (3 pair types)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Preset::H2 => "H₂",
            Preset::Quench16 => "16-H Quench",
            Preset::LiH => "LiH",
            Preset::HF => "HF",
            Preset::Li2 => "Li₂",
            Preset::N2 => "N₂",
            Preset::F2 => "F₂",
            Preset::CO => "CO",
            Preset::He2 => "He₂",
            Preset::Ne2 => "Ne₂",
            Preset::HCl => "HCl",
            Preset::MixedGas => "8H+8He",
        }
    }

    pub fn next(&self) -> Preset {
        match self {
            Preset::H2 => Preset::Quench16,
            Preset::Quench16 => Preset::LiH,
            Preset::LiH => Preset::HF,
            Preset::HF => Preset::Li2,
            Preset::Li2 => Preset::N2,
            Preset::N2 => Preset::F2,
            Preset::F2 => Preset::CO,
            Preset::CO => Preset::He2,
            Preset::He2 => Preset::Ne2,
            Preset::Ne2 => Preset::HCl,
            Preset::HCl => Preset::MixedGas,
            Preset::MixedGas => Preset::H2,
        }
    }
}

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
    /// BOXED, and the box is load-bearing rather than stylistic.
    ///
    /// `Sim` is 331,656 bytes since MIXTURES-1 gave it a pair BANK — six potential tables
    /// where there was one, 193 KB of the total. `new_with_preset` builds one, moves it
    /// into this struct, and returns the struct by value; the debug profile elides none
    /// of those moves, so an unboxed field put roughly 2 MB of `Sim` copies on the stack
    /// and `all_presets_load_and_conserve_energy` aborted with a stack overflow — in the
    /// DEBUG profile only, which is how it survived a release-profile suite run.
    ///
    /// One pointer here is the difference. The same class of defect as
    /// `Box::new(<big array literal>)` in the wasm build: a value that will live on the
    /// heap is still assembled on the stack unless the type says otherwise.
    pub sim: Box<Sim>,
    pub calibration: Calibration,
    /// Substeps the last frame actually took, for the clocks readout.
    pub last_substeps: u32,
    /// Measured wall interval of the last frame, seconds.
    pub last_frame_seconds: f64,
    /// Set when the curve failed to generate; the HUD says so instead of drawing a
    /// scene whose forces came from nowhere.
    pub table_status: u32,
    pub preset: Preset,
}

impl AtomWorld {
    /// Build the scene: generate the exact curve, adopt the clocks it implies, put the
    /// atoms in a cube.
    ///
    /// `holon_chem` solves H2 in the STO-3G basis exactly (full CI) from closed-form
    /// Gaussian integrals and differentiates it analytically, here, at startup. The
    /// shell does not play a curve somebody computed; it solves the one it is showing.
    pub fn new(atoms: usize) -> Self {
        let preset = if atoms > 2 {
            Preset::Quench16
        } else {
            Preset::H2
        };
        Self::new_with_preset(preset)
    }

    /// Construct a new [`AtomWorld`] with a specific scene preset.
    pub fn new_with_preset(preset: Preset) -> Self {
        // Boxed on the first line, not at the end: an unboxed local here is a second
        // 324 KB stack frame that the debug profile keeps. See the field's own note.
        let mut sim = Box::new(Sim::empty());
        sim.dims = Dims::Three;
        sim.boundary = Boundary::Walls;
        sim.width = BOX_SIDE;
        sim.height = BOX_SIDE;
        sim.depth = BOX_SIDE;
        sim.wall_inset = ATOM_RADIUS;
        let mut world = Self {
            sim,
            calibration: Calibration::Pending,
            last_substeps: 0,
            last_frame_seconds: 0.0,
            table_status: 0,
            preset,
        };
        world.load_preset(preset);
        world
    }

    /// Load a scene preset dynamically.
    ///
    /// The bank is CLEARED first. It holds three species at a time, so a shell that cycled
    /// presets without clearing would be full after the third one and would refuse the
    /// fourth — and the refusal would arrive as an unexplained missing curve rather than
    /// as anything a user could read. Clearing returns it to the hydrogen-seeded state.
    pub fn load_preset(&mut self, preset: Preset) {
        self.sim.clear_bank();
        let status = match preset {
            Preset::H2 => {
                let s = holon_render::generate_table(
                    &mut self.sim,
                    CURVE_R_MIN,
                    CURVE_R_MAX,
                    CURVE_KNOTS,
                );
                if s == holon_render::TABLE_OK {
                    holon_render::generate_trimer_table(&mut self.sim);
                }
                self.sim.reset(2);
                for i in 0..self.sim.n {
                    self.sim.atoms[i].species = HYDROGEN;
                }
                s
            }
            Preset::Quench16 => {
                let s = holon_render::generate_table(
                    &mut self.sim,
                    CURVE_R_MIN,
                    CURVE_R_MAX,
                    CURVE_KNOTS,
                );
                if s == holon_render::TABLE_OK {
                    holon_render::generate_trimer_table(&mut self.sim);
                }
                self.sim.reset(MAX_ATOMS);
                for i in 0..self.sim.n {
                    self.sim.atoms[i].species = HYDROGEN;
                }
                s
            }
            Preset::LiH => {
                let s = holon_render::generate_pair_table(
                    &mut self.sim,
                    LITHIUM,
                    HYDROGEN,
                    64,
                );
                self.setup_dimer(LITHIUM, HYDROGEN);
                s
            }
            Preset::HF => {
                let s = holon_render::generate_pair_table(
                    &mut self.sim,
                    HYDROGEN,
                    FLUORINE,
                    64,
                );
                self.setup_dimer(HYDROGEN, FLUORINE);
                s
            }
            Preset::Li2 => {
                let s = holon_render::generate_pair_table(
                    &mut self.sim,
                    LITHIUM,
                    LITHIUM,
                    64,
                );
                self.setup_dimer(LITHIUM, LITHIUM);
                s
            }
            Preset::N2 => {
                let s = holon_render::generate_pair_table(
                    &mut self.sim,
                    NITROGEN,
                    NITROGEN,
                    64,
                );
                self.setup_dimer(NITROGEN, NITROGEN);
                s
            }
            Preset::F2 => {
                let s = holon_render::generate_pair_table(
                    &mut self.sim,
                    FLUORINE,
                    FLUORINE,
                    64,
                );
                self.setup_dimer(FLUORINE, FLUORINE);
                s
            }
            Preset::CO => {
                let s = holon_render::generate_pair_table(
                    &mut self.sim,
                    CARBON,
                    OXYGEN,
                    64,
                );
                self.setup_dimer(CARBON, OXYGEN);
                s
            }
            Preset::He2 => {
                let s = holon_render::generate_pair_table(
                    &mut self.sim,
                    HELIUM,
                    HELIUM,
                    64,
                );
                self.setup_dimer(HELIUM, HELIUM);
                s
            }
            Preset::Ne2 => {
                let s = holon_render::generate_pair_table(
                    &mut self.sim,
                    NEON,
                    NEON,
                    64,
                );
                self.setup_dimer(NEON, NEON);
                s
            }
            Preset::HCl => {
                let s = holon_render::generate_pair_table(
                    &mut self.sim,
                    HYDROGEN,
                    CHLORINE,
                    64,
                );
                self.setup_dimer(HYDROGEN, CHLORINE);
                s
            }
            Preset::MixedGas => self.setup_mixed_gas(),
        };
        self.preset = preset;
        self.table_status = status;
    }

    /// THE MIXED SCENE: eight hydrogens and eight heliums, every pair type banked.
    ///
    /// Three curves are needed and all three are generated here — H-H, H-He and He-He —
    /// because `Sim::pairs_ready` refuses to step a scene that is missing any of them.
    /// That refusal is the point of the bank: the old question, "is the table loaded",
    /// answers yes for a scene with one curve out of three.
    ///
    /// Atoms alternate along a Fibonacci sphere, so neither species is clustered on one
    /// side and every helium has hydrogen neighbours.
    fn setup_mixed_gas(&mut self) -> u32 {
        let mut status = holon_render::TABLE_OK;
        for (a, b) in [(HYDROGEN, HYDROGEN), (HYDROGEN, HELIUM), (HELIUM, HELIUM)] {
            let s = holon_render::generate_pair_table(&mut self.sim, a, b, 64);
            if s != holon_render::TABLE_OK {
                status = s;
            }
        }
        self.sim.reset(MAX_ATOMS);
        for i in 0..self.sim.n {
            let sp = if i % 2 == 0 { HYDROGEN } else { HELIUM };
            assert!(self.sim.set_species(i, sp), "the bank refused the mixed scene");
        }
        // The opener's escape-speed derivation reads each pair's OWN curve, so it has to
        // run after the species are assigned, not before.
        self.sim.reset(MAX_ATOMS);
        self.sim.adopt_table_timescale();
        self.sim.rebase();
        status
    }

    fn setup_dimer(&mut self, sp_a: Species, sp_b: Species) {
        self.sim.reset(2);
        assert!(
            self.sim.set_species(0, sp_a) && self.sim.set_species(1, sp_b),
            "the bank refused {}{}: it holds {} species and this preset needs two",
            sp_a.symbol,
            sp_b.symbol,
            holon_render::bank::MAX_SPECIES
        );
        let cx = 0.5 * self.sim.width;
        let cy = 0.5 * self.sim.height;
        let cz = 0.5 * self.sim.depth;
        self.sim.atoms[0].x = cx - 5.0;
        self.sim.atoms[0].y = cy;
        self.sim.atoms[0].z = cz;
        self.sim.atoms[1].x = cx + 5.0;
        self.sim.atoms[1].y = cy;
        self.sim.atoms[1].z = cz;
        let ma = self.sim.atoms[0].mass();
        let mb = self.sim.atoms[1].mass();
        let mu = (ma * mb) / (ma + mb);
        let v_rel = 0.0004;
        self.sim.atoms[0].vx = v_rel * (mu / ma);
        self.sim.atoms[0].vy = 0.0;
        self.sim.atoms[0].vz = 0.0;
        self.sim.atoms[1].vx = -v_rel * (mu / mb);
        self.sim.atoms[1].vy = 0.0;
        self.sim.atoms[1].vz = 0.0;
        self.sim.adopt_table_timescale();
        self.sim.rebase();
    }

    /// Cycle to the next scene preset.
    pub fn next_preset(&mut self) {
        self.load_preset(self.preset.next());
    }

    /// Whether the scene has every curve it needs.
    ///
    /// `pairs_ready` and not `table().is_loaded()`: a mixed scene needs a curve for EVERY
    /// pair type its atoms form, and the old question — is THE table loaded — answers yes
    /// for a scene with one curve out of three.
    pub fn table_ok(&self) -> bool {
        self.table_status == holon_render::TABLE_OK && self.sim.pairs_ready()
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
