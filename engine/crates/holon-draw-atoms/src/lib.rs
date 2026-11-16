//! The 3D atoms scene drawn through `holon-draw` instead of Bevy.
//!
//! The physics is `holon-render`'s [`Sim`], set up exactly as `holon-render-3d`'s
//! `AtomWorld` sets it up (a 24-bohr cube with walls, the same presets, the same curve
//! request, the same calibration burst), so the two pages show one simulation and differ
//! only in how it is drawn. What is drawn: a sphere per atom (tagged, so it can be picked),
//! a rod per bonded pair (decoration), the cube's twelve edges (decoration), and the hand's
//! anchor while an atom is held.
//!
//! The boundary is plain `extern "C"` (`ha_*`), as `holon-draw`'s is.
use holon_chem::elements::{Species, CARBON, CHLORINE, FLUORINE, HELIUM, HYDROGEN, LITHIUM, NEON, NITROGEN, OXYGEN};
use holon_draw::{Camera, Frame, Instance, Kind, Ring, Scene, Tile};
use holon_render::clock::{Rung, AU_TO_FS};
use holon_render::sim::{Boundary, Dims, Sim, DEFAULT_SCENE_ATOMS};
use std::cell::RefCell;

pub const BOX_SIDE: f64 = 24.0;
pub const ATOM_RADIUS: f64 = 0.6;
const CURVE_R_MIN: f64 = 0.3;
const CURVE_R_MAX: f64 = 10.0;
const CURVE_KNOTS: usize = 492;

/// The presets, in `holon-render-3d`'s order.
pub const PRESETS: [&str; 12] = ["H2", "Quench16", "LiH", "HF", "Li2", "N2", "F2", "CO", "He2", "Ne2", "HCl", "MixedGas"];

/// Multiplicative step per press of the sim-speed buttons (`holon-render-3d`'s `SPEED_STEP`).
pub const SPEED_STEP: f64 = 1.5;
/// Substeps in the calibration burst (`holon-render-3d`'s `CALIBRATION_SUBSTEPS`).
pub const CALIBRATION_SUBSTEPS: u32 = 20_000;

/// The length of the array [`World::fill_hud`] fills and `ha_hud` points at.
pub const HUD_LEN: usize = 71;

/// Where each number of the overlay sits in the HUD array. Every one is READ from the `Sim`
/// by the same function or field `holon-render-3d/src/hud.rs` reads; none is recomputed.
/// Flags are 0.0 or 1.0; counts and indices are whole numbers held in an f64.
pub mod hud {
    // the status line
    pub const TABLE_OK: usize = 0; // every curve the scene needs is loaded
    pub const TABLE_STATUS: usize = 1; // the curve's status code (0 is success)
    pub const PRESET: usize = 2; // index into PRESETS
    pub const ATOMS: usize = 3; // sim.n
    pub const TIME_AU: usize = 4; // sim.time
    pub const TIME_FS: usize = 5; // sim.time * AU_TO_FS
    pub const CLUSTERS: usize = 6; // sim.cluster_count().0
    pub const BONDED_PAIRS: usize = 7; // sim.bonded_count()
    // gate 1, energy
    pub const ENERGY_PASS: usize = 8; // sim.energy_gate()
    pub const E_KIN: usize = 9;
    pub const E_PAIR: usize = 10;
    pub const E_WALL: usize = 11;
    pub const E_SPRING: usize = 12;
    pub const W_EXT: usize = 13;
    pub const LEDGER: usize = 14; // sim.ledger(), E - W_ext
    pub const ORIGIN: usize = 15; // sim.l0
    pub const DRIFT_PEAK: usize = 16;
    pub const DRIFT_BOUND: usize = 17; // sim.drift_bound()
    pub const DRIFT_RATIO: usize = 18; // peak / bound, 0 when the bound is not positive
    // gate 2, momentum
    pub const MOMENTUM_PASS: usize = 19; // sim.momentum_gate()
    pub const P_MAG: usize = 20;
    pub const PX: usize = 21;
    pub const PY: usize = 22;
    pub const PZ: usize = 23;
    pub const P_RESIDUAL: usize = 24; // sim.momentum_residual_peak
    pub const P_BOUND: usize = 25; // sim.momentum_bound(), "roundoff"
    pub const P_RATIO: usize = 26;
    // the three clocks
    pub const RUNG: usize = 27; // 0 exact, 1 time dilated, 2 accuracy declared, 3 refused
    pub const DT: usize = 28;
    pub const DT_REFERENCE: usize = 29;
    pub const OMEGA_E: usize = 30;
    pub const OMEGA_ENV: usize = 31;
    pub const OMEGA_DT: usize = 32;
    pub const FRAME_MS: usize = 33; // the last advanced frame's wall interval
    pub const SUBSTEPS: usize = 34; // substeps that frame took
    pub const SIM_SPEED: usize = 35; // fs per wall second
    pub const VIBRATION_S: usize = 36; // wall seconds per vibration (+inf at zero speed)
    pub const DILATION: usize = 37;
    // the curve
    pub const R_E: usize = 38;
    pub const D_E: usize = 39;
    pub const ASYMPTOTE: usize = 40;
    pub const KNOTS: usize = 41;
    pub const PAIR_PRESENT: usize = 42; // 0 when the scene has no pair
    pub const PAIR_I: usize = 43;
    pub const PAIR_J: usize = 44;
    pub const PAIR_Z_I: usize = 45; // nuclear charge of atom i (the host maps it to a symbol)
    pub const PAIR_Z_J: usize = 46;
    pub const PAIR_R: usize = 47;
    pub const PAIR_E_REL: usize = 48;
    pub const PAIR_R_OUTER: usize = 49; // +inf for an unbound pair, as the Sim holds it
    pub const PAIR_BONDED: usize = 50;
    pub const TEMPERATURE: usize = 51;
    pub const DOF: usize = 52;
    // the holon census
    pub const CENSUS_ATOMS: usize = 53;
    pub const CENSUS_MOLECULES: usize = 54;
    pub const CENSUS_CANDIDATES: usize = 55;
    pub const CENSUS_VIEWS: usize = 56;
    pub const CENSUS_FORMATIONS: usize = 57;
    pub const CENSUS_DISSOLUTIONS: usize = 58;
    pub const CENSUS_NOT_CLOSED: usize = 59;
    pub const BOND_SECTOR_ENERGY: usize = 60;
    // this device
    pub const SUBSTEPS_PER_SEC: usize = 61;
    pub const PAIRS_PER_SEC: usize = 62;
    pub const REQUIRED_PER_SEC: usize = 63;
    pub const N_MAX: usize = 64;
    pub const CALIBRATION: usize = 65; // 0 pending, 1 measured, 2 no usable clock
    // the button labels
    pub const BOUNDARY: usize = 66; // 0 walls, 1 open, 2 periodic
    pub const THERMOSTAT: usize = 67;
    pub const COMPOSITE: usize = 68; // sim.holons.enabled
    pub const DT_GROWTH: usize = 69; // timescale.allow_dt_growth
    pub const GRABBED: usize = 70; // the held atom's index plus one, or 0
}

/// `holon-render-3d`'s `Calibration`, as the number the HUD array carries.
pub const CALIBRATION_PENDING: u32 = 0;
pub const CALIBRATION_DONE: u32 = 1;
pub const CALIBRATION_UNAVAILABLE: u32 = 2;

pub struct World {
    pub sim: Box<Sim>,
    pub status: u32,
    pub preset: u32,
    /// Where the device measurement has got to (`CALIBRATION_*`).
    pub calibration: u32,
    /// Substeps the last advanced frame took, and its measured wall interval.
    pub last_substeps: u32,
    pub last_frame_seconds: f64,
    pub hud: [f64; HUD_LEN],
    pub frame: Option<Frame>,
    pub camera: Option<Camera>,
    pub vp: [f32; 16],
}

impl World {
    pub fn new() -> World {
        let mut sim = Box::new(Sim::empty());
        sim.dims = Dims::Three;
        sim.boundary = Boundary::Walls;
        sim.width = BOX_SIDE;
        sim.height = BOX_SIDE;
        sim.depth = BOX_SIDE;
        sim.wall_inset = ATOM_RADIUS;
        World { sim, status: u32::MAX, preset: 0, calibration: CALIBRATION_PENDING, last_substeps: 0, last_frame_seconds: 0.0, hud: [0.0; HUD_LEN], frame: None, camera: None, vp: [0.0; 16] }
    }

    fn dimer(&mut self, a: Species, b: Species) -> u32 {
        let s = holon_render::generate_pair_table(&mut self.sim, a, b, 64);
        self.sim.reset(2);
        if !(self.sim.set_species(0, a) && self.sim.set_species(1, b)) {
            return u32::MAX - 1;
        }
        let (cx, cy, cz) = (0.5 * self.sim.width, 0.5 * self.sim.height, 0.5 * self.sim.depth);
        let half = 2.5;
        let at = &mut self.sim.atoms;
        (at[0].x, at[0].y, at[0].z) = (cx - half, cy, cz);
        (at[1].x, at[1].y, at[1].z) = (cx + half, cy, cz);
        for a in at.iter_mut().take(2) {
            (a.vx, a.vy, a.vz) = (0.0, 0.0, 0.0);
        }
        self.sim.adopt_table_timescale();
        self.sim.rebase();
        s
    }

    /// Load a preset by its index in [`PRESETS`]; returns the curve's status
    /// (`holon_render::TABLE_OK` is success).
    pub fn load(&mut self, preset: u32) -> u32 {
        self.sim.clear_bank();
        let hydrogen = |w: &mut World, n: usize| {
            let s = holon_render::generate_table(&mut w.sim, CURVE_R_MIN, CURVE_R_MAX, CURVE_KNOTS);
            if s == holon_render::TABLE_OK {
                holon_render::generate_trimer_table(&mut w.sim);
            }
            w.sim.reset(n);
            for i in 0..w.sim.n {
                w.sim.atoms[i].species = HYDROGEN;
            }
            s
        };
        let status = match preset {
            0 => hydrogen(self, 2),
            1 => hydrogen(self, DEFAULT_SCENE_ATOMS),
            2 => self.dimer(LITHIUM, HYDROGEN),
            3 => self.dimer(HYDROGEN, FLUORINE),
            4 => self.dimer(LITHIUM, LITHIUM),
            5 => self.dimer(NITROGEN, NITROGEN),
            6 => self.dimer(FLUORINE, FLUORINE),
            7 => self.dimer(CARBON, OXYGEN),
            8 => self.dimer(HELIUM, HELIUM),
            9 => self.dimer(NEON, NEON),
            10 => self.dimer(HYDROGEN, CHLORINE),
            11 => {
                let mut st = holon_render::TABLE_OK;
                for (a, b) in [(HYDROGEN, HYDROGEN), (HYDROGEN, HELIUM), (HELIUM, HELIUM)] {
                    let s = holon_render::generate_pair_table(&mut self.sim, a, b, 64);
                    if s != holon_render::TABLE_OK {
                        st = s;
                    }
                }
                self.sim.reset(DEFAULT_SCENE_ATOMS);
                for i in 0..self.sim.n {
                    if !self.sim.set_species(i, if i % 2 == 0 { HYDROGEN } else { HELIUM }) {
                        st = u32::MAX - 1;
                    }
                }
                self.sim.reset(DEFAULT_SCENE_ATOMS);
                self.sim.adopt_table_timescale();
                self.sim.rebase();
                st
            }
            _ => return u32::MAX,
        };
        self.preset = preset;
        self.status = status;
        status
    }

    pub fn ready(&self) -> bool {
        self.status == holon_render::TABLE_OK && self.sim.pairs_ready()
    }

    /// The calibration burst of `holon-render-3d` (`AtomWorld::calibration_burst`): the
    /// host times it and hands the rate back with [`World::calibrated`].
    pub fn burst(&mut self, substeps: u32) {
        let (n, b) = (self.sim.n, self.sim.boundary);
        self.sim.boundary = Boundary::Open;
        self.sim.reset(DEFAULT_SCENE_ATOMS);
        for _ in 0..substeps {
            self.sim.step();
        }
        self.sim.boundary = b;
        self.sim.reset(n);
    }

    pub fn calibrated(&mut self, substeps_per_second: f64) {
        if substeps_per_second.is_finite() && substeps_per_second > 0.0 {
            self.sim.timescale.substeps_per_second = substeps_per_second;
            self.sim.timescale.calibrated = true;
            self.calibration = CALIBRATION_DONE;
        } else {
            self.calibration = CALIBRATION_UNAVAILABLE;
        }
    }

    /// `AtomWorld::pairs_per_second`: the calibration rate times the calibration scene's pairs.
    pub fn pairs_per_second(&self) -> f64 {
        self.sim.timescale.substeps_per_second * holon_render::sim::complete_pairs(DEFAULT_SCENE_ATOMS) as f64
    }

    /// `AtomWorld::n_max`: the default scene size until the device has been measured.
    pub fn n_max(&self) -> f64 {
        if !self.sim.timescale.calibrated {
            return DEFAULT_SCENE_ATOMS as f64;
        }
        holon_render::clock::n_max(self.pairs_per_second(), self.sim.timescale.required_substeps_per_second())
    }

    /// `AtomWorld::reset`: `n` atoms, clamped to what this device was measured to sustain.
    pub fn reset(&mut self, n: usize) {
        let cap = (self.n_max() as usize).max(2);
        self.sim.reset(n.max(2).min(cap));
    }

    /// `AtomWorld::wall_seconds_per_vibration`.
    pub fn wall_seconds_per_vibration(&self) -> f64 {
        let speed = self.sim.timescale.sim_speed_fs_per_wallsec;
        if speed > 0.0 {
            self.sim.timescale.period * AU_TO_FS / speed
        } else {
            f64::INFINITY
        }
    }

    // The button row of `holon-render-3d/src/hud.rs` (`handle_actions`), arm for arm.

    pub fn next_preset(&mut self) -> u32 {
        self.load((self.preset + 1) % PRESETS.len() as u32)
    }
    pub fn fewer_atoms(&mut self) {
        let n = self.sim.n.saturating_sub(1).max(2);
        self.reset(n);
    }
    pub fn more_atoms(&mut self) {
        let n = (self.sim.n + 1).min(DEFAULT_SCENE_ATOMS);
        self.reset(n);
    }
    /// Walls -> Open -> Periodic -> Walls, and the ledger re-based, since the edit changes
    /// the scene and the drift should measure the integrator, not the edit.
    pub fn cycle_boundary(&mut self) {
        self.sim.boundary = match self.sim.boundary {
            Boundary::Walls => Boundary::Open,
            Boundary::Open => Boundary::Periodic,
            Boundary::Periodic => Boundary::Walls,
        };
        self.sim.rebase();
    }
    /// Rung (ii) by explicit toggle; turning it off re-derives dt from the envelope at once.
    pub fn toggle_dt_growth(&mut self) {
        let on = !self.sim.timescale.allow_dt_growth;
        self.sim.timescale.allow_dt_growth = on;
        if on {
            self.sim.timescale.set_dt_multiplier(2.0);
        } else {
            let e = self.sim.timescale.e_rel_max;
            self.sim.timescale.e_rel_max = f64::NEG_INFINITY;
            self.sim.timescale.k_env = 0.0;
            let table = core::mem::replace(self.sim.table_mut(), holon_render::table::PotentialTable::empty());
            self.sim.timescale.refresh_envelope(&table, e);
            *self.sim.table_mut() = table;
        }
    }

    /// Fill the HUD array (layout: [`hud`]) from the `Sim`, as `update_hud` reads it.
    pub fn fill_hud(&mut self) {
        use hud::*;
        let ratio = |measured: f64, bound: f64| if bound > 0.0 { measured / bound } else { 0.0 };
        let flag = |b: bool| if b { 1.0 } else { 0.0 };
        let mut h = [0.0f64; HUD_LEN];
        let s = &self.sim;
        h[TABLE_OK] = flag(self.ready());
        h[TABLE_STATUS] = self.status as f64;
        h[PRESET] = self.preset as f64;
        h[ATOMS] = s.n as f64;
        h[TIME_AU] = s.time;
        h[TIME_FS] = s.time * AU_TO_FS;
        h[CLUSTERS] = s.cluster_count().0 as f64;
        h[BONDED_PAIRS] = s.bonded_count() as f64;

        let (drift_bound, momentum_bound) = (s.drift_bound(), s.momentum_bound());
        h[ENERGY_PASS] = flag(s.energy_gate());
        h[E_KIN] = s.e_kin;
        h[E_PAIR] = s.e_pair;
        h[E_WALL] = s.e_wall;
        h[E_SPRING] = s.e_spring;
        h[W_EXT] = s.w_ext;
        h[LEDGER] = s.ledger();
        h[ORIGIN] = s.l0;
        h[DRIFT_PEAK] = s.drift_peak;
        h[DRIFT_BOUND] = drift_bound;
        h[DRIFT_RATIO] = ratio(s.drift_peak, drift_bound);

        let (px, py, pz) = s.momentum();
        h[MOMENTUM_PASS] = flag(s.momentum_gate());
        h[P_MAG] = (px * px + py * py + pz * pz).sqrt();
        h[PX] = px;
        h[PY] = py;
        h[PZ] = pz;
        h[P_RESIDUAL] = s.momentum_residual_peak;
        h[P_BOUND] = momentum_bound;
        h[P_RATIO] = ratio(s.momentum_residual_peak, momentum_bound);

        let t = &s.timescale;
        h[RUNG] = match t.rung {
            Rung::Exact => 0.0,
            Rung::TimeDilated => 1.0,
            Rung::AccuracyDeclared => 2.0,
            Rung::Refused => 3.0,
        };
        h[DT] = t.dt;
        h[DT_REFERENCE] = t.dt_reference;
        h[OMEGA_E] = t.omega_e;
        h[OMEGA_ENV] = t.omega_env;
        h[OMEGA_DT] = t.omega_dt();
        h[FRAME_MS] = self.last_frame_seconds * 1000.0;
        h[SUBSTEPS] = self.last_substeps as f64;
        h[SIM_SPEED] = t.sim_speed_fs_per_wallsec;
        h[VIBRATION_S] = self.wall_seconds_per_vibration();
        h[DILATION] = t.dilation;

        h[R_E] = s.table().r_e;
        h[D_E] = s.table().d_e;
        h[ASYMPTOTE] = s.table().e_asymptote;
        h[KNOTS] = s.table().knots() as f64;
        // the closest pair: the one about to do something
        if let Some(p) = s.pairs[..s.pair_count].iter().min_by(|a, b| a.r.total_cmp(&b.r)) {
            h[PAIR_PRESENT] = 1.0;
            h[PAIR_I] = p.i as f64;
            h[PAIR_J] = p.j as f64;
            h[PAIR_Z_I] = s.atoms[p.i].species.z as f64;
            h[PAIR_Z_J] = s.atoms[p.j].species.z as f64;
            h[PAIR_R] = p.r;
            h[PAIR_E_REL] = p.e_rel;
            h[PAIR_R_OUTER] = p.r_outer;
            h[PAIR_BONDED] = flag(p.bonded);
        }
        h[TEMPERATURE] = s.temperature();
        h[DOF] = s.dims.dof();

        let c = &s.holons.census;
        h[CENSUS_ATOMS] = c.atoms as f64;
        h[CENSUS_MOLECULES] = c.molecules as f64;
        h[CENSUS_CANDIDATES] = c.candidate_evaluations as f64;
        h[CENSUS_VIEWS] = c.global_views as f64;
        h[CENSUS_FORMATIONS] = c.formations as f64;
        h[CENSUS_DISSOLUTIONS] = c.dissolutions as f64;
        h[CENSUS_NOT_CLOSED] = c.closure_rejections as f64;
        h[BOND_SECTOR_ENERGY] = s.holons.bond_sector_energy();

        h[SUBSTEPS_PER_SEC] = t.substeps_per_second;
        h[PAIRS_PER_SEC] = self.pairs_per_second();
        h[REQUIRED_PER_SEC] = t.required_substeps_per_second();
        h[N_MAX] = self.n_max() as u64 as f64;
        h[CALIBRATION] = self.calibration as f64;

        h[BOUNDARY] = match s.boundary {
            Boundary::Walls => 0.0,
            Boundary::Open => 1.0,
            Boundary::Periodic => 2.0,
        };
        h[THERMOSTAT] = flag(s.thermostat_on);
        h[COMPOSITE] = flag(s.holons.enabled);
        h[DT_GROWTH] = flag(t.allow_dt_growth);
        h[GRABBED] = s.grabbed.filter(|&g| g < s.n).map_or(0.0, |g| g as f64 + 1.0);
        self.hud = h;
    }

    /// One frame of `wall_dt` measured seconds, as `AtomWorld::advance`.
    pub fn advance(&mut self, wall_dt: f64) -> u32 {
        if !self.ready() {
            return 0;
        }
        let budget = self.sim.timescale.substep_budget(wall_dt);
        let plan = self.sim.timescale.plan_frame(wall_dt, budget);
        self.sim.step_frame(plan.substeps);
        self.last_substeps = plan.substeps;
        self.last_frame_seconds = wall_dt;
        plan.substeps
    }

    /// The scene as marks: one tile, origin at the world's origin.
    pub fn scene(&self) -> Scene {
        let s = &self.sim;
        let mut marks = Vec::with_capacity(s.n + s.pair_count + 13);
        let rgb = |sp: Species| {
            let (r, g, b) = sp.colour_rgb();
            [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255]
        };
        for i in 0..s.n {
            let a = &s.atoms[i];
            let mut c = rgb(a.species);
            if s.grabbed == Some(i) {
                c = [233, 196, 106, 255];
            }
            marks.push(Instance::new(Kind::Sphere, [a.x as f32, a.y as f32, a.z as f32], [a.radius() as f32, 0.0, 0.0]).tagged(i as u32 + 1).colour(c));
        }
        for p in &s.pairs[..s.pair_count] {
            if p.bonded && p.i < s.n && p.j < s.n {
                let (a, b) = (&s.atoms[p.i], &s.atoms[p.j]);
                let mut m = Instance::new(Kind::Rod, [a.x as f32, a.y as f32, a.z as f32], [0.16, 0.0, 0.0]).colour([60, 72, 68, 255]);
                m.aux = [b.x as f32, b.y as f32, b.z as f32];
                marks.push(m);
            }
        }
        let l = BOX_SIDE as f32;
        let corners = |i: u32| [((i & 1) as f32) * l, (((i >> 1) & 1) as f32) * l, (((i >> 2) & 1) as f32) * l];
        for a in 0..8u32 {
            for bit in [1u32, 2, 4] {
                if a & bit == 0 {
                    let mut m = Instance::new(Kind::Line, corners(a), [0.05, 0.0, 0.0]).colour([105, 115, 110, 255]);
                    m.aux = corners(a | bit);
                    marks.push(m);
                }
            }
        }
        if let Some(g) = s.grabbed.filter(|&g| g < s.n) {
            let (x, y, z) = s.anchor;
            let a = &s.atoms[g];
            let mut m = Instance::new(Kind::Line, [a.x as f32, a.y as f32, a.z as f32], [0.06, 0.0, 0.0]).colour([233, 196, 106, 255]);
            m.aux = [x as f32, y as f32, z as f32];
            marks.push(m);
            marks.push(Instance::new(Kind::Sphere, [x as f32, y as f32, z as f32], [0.18, 0.0, 0.0]).colour([233, 196, 106, 255]));
        }
        Scene { tiles: vec![Tile { id: 1, origin: [0.0; 3], version: 1, layer: 0, screen: false, marks }], modal: None }
    }

    /// Build the frame for an orbit view about the box's centre.
    pub fn view(&mut self, yaw: f64, pitch: f64, dist: f64, width: f64, height: f64) -> usize {
        let h = 0.5 * BOX_SIDE;
        let cam = Camera::orbit([h, h, h], yaw, pitch, dist, std::f64::consts::FRAC_PI_4, width, height);
        let f = self.scene().frame(&cam, 1, &Ring, u64::MAX).expect("the atoms scene has no non-finite field");
        self.vp = cam.relative();
        let n = f.batches.len();
        self.frame = Some(f);
        self.camera = Some(cam);
        n
    }

    /// Grab the atom under a pixel; returns its index plus one, or 0.
    pub fn grab_at(&mut self, x: f64, y: f64) -> u32 {
        let Some(t) = self.frame.as_ref().and_then(|f| f.pick(x, y)) else { return 0 };
        self.sim.grab(t as usize - 1);
        t
    }

    /// Move the held atom's anchor to the pixel, on the plane through the anchor facing the
    /// camera.
    pub fn drag_to(&mut self, x: f64, y: f64) {
        let (Some(cam), Some(_)) = (self.camera.as_ref(), self.sim.grabbed) else { return };
        let a = self.sim.anchor;
        let Some(p) = cam.project([a.0, a.1, a.2]) else { return };
        // two screen-parallel directions at the anchor, by finite difference of the projection
        let e = 0.01;
        let probe = |d: [f64; 3]| cam.project([a.0 + d[0], a.1 + d[1], a.2 + d[2]]).map(|q| [(q[0] - p[0]) / e, (q[1] - p[1]) / e]);
        let (Some(jx), Some(jy), Some(jz)) = (probe([e, 0.0, 0.0]), probe([0.0, e, 0.0]), probe([0.0, 0.0, e])) else { return };
        // least-norm world move whose screen image is the pixel offset: J^T (J J^T)^-1 d
        let j = [[jx[0], jy[0], jz[0]], [jx[1], jy[1], jz[1]]];
        let m = [[j[0].iter().map(|v| v * v).sum::<f64>(), (0..3).map(|k| j[0][k] * j[1][k]).sum::<f64>()], [0.0, j[1].iter().map(|v| v * v).sum::<f64>()]];
        let (a11, a12, a22) = (m[0][0], m[0][1], m[1][1]);
        let det = a11 * a22 - a12 * a12;
        if det.abs() < 1e-12 {
            return;
        }
        let d = [x - p[0], y - p[1]];
        let w = [(a22 * d[0] - a12 * d[1]) / det, (-a12 * d[0] + a11 * d[1]) / det];
        let mv: Vec<f64> = (0..3).map(|k| j[0][k] * w[0] + j[1][k] * w[1]).collect();
        let c = |v: f64| v.clamp(ATOM_RADIUS, BOX_SIDE - ATOM_RADIUS);
        self.sim.move_anchor_3d(c(a.0 + mv[0]), c(a.1 + mv[1]), c(a.2 + mv[2]));
    }
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}

thread_local! {
    static W: RefCell<World> = RefCell::new(World::new());
}

fn with<R>(f: impl FnOnce(&mut World) -> R) -> R {
    W.with(|w| f(&mut w.borrow_mut()))
}

#[no_mangle]
pub extern "C" fn ha_load(preset: u32) -> u32 {
    with(|w| w.load(preset))
}
#[no_mangle]
pub extern "C" fn ha_ready() -> u32 {
    with(|w| w.ready() as u32)
}
#[no_mangle]
pub extern "C" fn ha_burst(substeps: u32) {
    with(|w| w.burst(substeps))
}
#[no_mangle]
pub extern "C" fn ha_calibrated(substeps_per_second: f64) {
    with(|w| w.calibrated(substeps_per_second))
}
#[no_mangle]
pub extern "C" fn ha_advance(wall_dt: f64) -> u32 {
    with(|w| w.advance(wall_dt))
}
#[no_mangle]
pub extern "C" fn ha_atoms() -> u32 {
    with(|w| w.sim.n as u32)
}
#[no_mangle]
pub extern "C" fn ha_bonds() -> u32 {
    with(|w| w.sim.pairs[..w.sim.pair_count].iter().filter(|p| p.bonded).count() as u32)
}
#[no_mangle]
pub extern "C" fn ha_view(yaw: f64, pitch: f64, dist: f64, width: f64, height: f64) -> u32 {
    with(|w| w.view(yaw, pitch, dist, width, height) as u32)
}
fn batch<R>(i: u32, f: impl FnOnce(&holon_draw::Batch) -> R, none: R) -> R {
    with(|w| w.frame.as_ref().and_then(|fr| fr.batches.get(i as usize)).map(f).unwrap_or(none))
}
#[no_mangle]
pub extern "C" fn ha_batch_kind(i: u32) -> u32 {
    batch(i, |b| b.kind as u32, 0)
}
#[no_mangle]
pub extern "C" fn ha_batch_count(i: u32) -> u32 {
    batch(i, |b| b.count, 0)
}
#[no_mangle]
pub extern "C" fn ha_batch_ptr(i: u32) -> *const u8 {
    batch(i, |b| b.bytes.as_ptr(), std::ptr::null())
}
#[no_mangle]
pub extern "C" fn ha_batch_offset(i: u32, axis: u32) -> f32 {
    batch(i, |b| b.offset.get(axis as usize).copied().unwrap_or(0.0), 0.0)
}
/// Where the 16 floats of the view's GPU matrix are (column-major).
#[no_mangle]
pub extern "C" fn ha_vp() -> *const f32 {
    W.with(|w| w.borrow().vp.as_ptr())
}
#[no_mangle]
pub extern "C" fn ha_grab_at(x: f64, y: f64) -> u32 {
    with(|w| w.grab_at(x, y))
}
#[no_mangle]
pub extern "C" fn ha_pick(x: f64, y: f64) -> u32 {
    with(|w| w.frame.as_ref().and_then(|f| f.pick(x, y)).unwrap_or(0))
}
#[no_mangle]
pub extern "C" fn ha_drag_to(x: f64, y: f64) {
    with(|w| w.drag_to(x, y))
}
#[no_mangle]
pub extern "C" fn ha_release() {
    with(|w| w.sim.release())
}
#[no_mangle]
pub extern "C" fn ha_frame_digest() -> f64 {
    with(|w| w.frame.as_ref().map_or(0.0, |f| (f.digest() >> 11) as f64))
}

/// Fill the HUD array from the `Sim` and return where its [`HUD_LEN`] f64 are (layout:
/// the [`hud`] module). The address is fixed; the host calls this when it wants fresh text.
#[no_mangle]
pub extern "C" fn ha_hud() -> *const f64 {
    W.with(|w| {
        let mut w = w.borrow_mut();
        w.fill_hud();
        w.hud.as_ptr()
    })
}
#[no_mangle]
pub extern "C" fn ha_hud_len() -> u32 {
    HUD_LEN as u32
}
/// 0 pending, 1 measured, 2 the host had no usable clock.
#[no_mangle]
pub extern "C" fn ha_calibration() -> u32 {
    with(|w| w.calibration)
}
#[no_mangle]
pub extern "C" fn ha_calibration_substeps() -> u32 {
    CALIBRATION_SUBSTEPS
}
// The button row, each doing what `holon-render-3d`'s `handle_actions` does.
/// Reset the scene at its present atom count.
#[no_mangle]
pub extern "C" fn ha_reset() {
    with(|w| {
        let n = w.sim.n;
        w.reset(n)
    })
}
/// Load the next preset; returns the curve's status.
#[no_mangle]
pub extern "C" fn ha_next_preset() -> u32 {
    with(|w| w.next_preset())
}
#[no_mangle]
pub extern "C" fn ha_fewer_atoms() {
    with(|w| w.fewer_atoms())
}
#[no_mangle]
pub extern "C" fn ha_more_atoms() {
    with(|w| w.more_atoms())
}
#[no_mangle]
pub extern "C" fn ha_slower() {
    with(|w| w.sim.timescale.sim_speed_fs_per_wallsec /= SPEED_STEP)
}
#[no_mangle]
pub extern "C" fn ha_faster() {
    with(|w| w.sim.timescale.sim_speed_fs_per_wallsec *= SPEED_STEP)
}
/// Walls -> open -> periodic -> walls.
#[no_mangle]
pub extern "C" fn ha_cycle_boundary() {
    with(|w| w.cycle_boundary())
}
#[no_mangle]
pub extern "C" fn ha_toggle_thermostat() {
    with(|w| w.sim.thermostat_on = !w.sim.thermostat_on)
}
#[no_mangle]
pub extern "C" fn ha_toggle_composite() {
    with(|w| w.sim.holons.enabled = !w.sim.holons.enabled)
}
#[no_mangle]
pub extern "C" fn ha_toggle_dt_growth() {
    with(|w| w.toggle_dt_growth())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hydrogen_presets_load_and_draw_one_sphere_per_atom() {
        // the two hydrogen scenes only: the heavier dimers solve their curves at load (the
        // twelve together took 629 s in a release test), which is the browser's wait, not CI's
        let mut w = World::new();
        for p in [0u32, 1] {
            assert_eq!(w.load(p), holon_render::TABLE_OK, "{}", PRESETS[p as usize]);
            assert!(w.ready());
            w.calibrated(2.0e5);
            for _ in 0..3 {
                w.advance(1.0 / 60.0);
            }
            let n = w.view(0.6, 0.35, 60.0, 1280.0, 800.0);
            let f = w.frame.as_ref().unwrap();
            let spheres: u32 = f.batches.iter().filter(|b| b.kind == Kind::Sphere).map(|b| b.count).sum();
            assert_eq!(spheres as usize, w.sim.n, "{}", PRESETS[p as usize]);
            assert!(n >= 2, "spheres and the box's lines");
            assert_eq!(f.marks.len(), w.sim.n, "every atom is a tagged instance");
        }
    }

    #[test]
    fn an_atom_is_picked_where_it_is_drawn_and_can_be_held_and_moved() {
        let mut w = World::new();
        w.load(0);
        w.view(0.6, 0.35, 60.0, 1280.0, 800.0);
        let a = w.sim.atoms[0];
        let p = w.camera.as_ref().unwrap().project([a.x, a.y, a.z]).unwrap();
        assert_eq!(w.grab_at(p[0], p[1]), 1);
        assert_eq!(w.sim.grabbed, Some(0));
        let before = w.sim.anchor;
        w.drag_to(p[0] + 40.0, p[1]);
        let after = w.sim.anchor;
        let moved = ((after.0 - before.0).powi(2) + (after.1 - before.1).powi(2) + (after.2 - before.2).powi(2)).sqrt();
        assert!(moved > 0.5, "{moved}");
        // the anchor lands under the pointer
        let q = w.camera.as_ref().unwrap().project([after.0, after.1, after.2]).unwrap();
        assert!((q[0] - (p[0] + 40.0)).abs() < 2.0 && (q[1] - p[1]).abs() < 2.0, "{q:?}");
        w.sim.release();
        assert!(w.sim.grabbed.is_none());
        assert_eq!(w.grab_at(5.0, 5.0), 0, "a corner of the screen holds no atom");
    }

    /// Every slot finite, bar the two the `Sim` itself holds as +inf.
    fn assert_hud_finite(w: &World, when: &str) {
        for (i, v) in w.hud.iter().enumerate() {
            let may_be_inf = i == hud::PAIR_R_OUTER || i == hud::VIBRATION_S;
            assert!(v.is_finite() || (may_be_inf && *v == f64::INFINITY), "{when}: hud[{i}] = {v}");
        }
    }

    #[test]
    fn the_hud_array_is_finite_and_both_gates_pass_on_h2() {
        let mut w = World::new();
        assert_eq!(w.load(0), holon_render::TABLE_OK);
        w.fill_hud();
        assert_hud_finite(&w, "before calibration");
        assert_eq!(w.hud[hud::CALIBRATION], CALIBRATION_PENDING as f64);
        assert_eq!(w.hud[hud::N_MAX], DEFAULT_SCENE_ATOMS as f64, "unmeasured: the default scene");
        w.burst(200);
        w.calibrated(2.0e5);
        for _ in 0..600 {
            w.advance(1.0 / 60.0);
        }
        w.fill_hud();
        assert_hud_finite(&w, "after 600 frames");
        let h = &w.hud;
        assert_eq!(h[hud::TABLE_OK], 1.0);
        assert_eq!(h[hud::ATOMS], 2.0);
        assert!(h[hud::TIME_AU] > 0.0 && h[hud::SUBSTEPS] >= 1.0);
        assert_eq!(h[hud::ENERGY_PASS], 1.0, "drift {} against bound {}", h[hud::DRIFT_PEAK], h[hud::DRIFT_BOUND]);
        assert!(h[hud::DRIFT_BOUND] > 0.0 && h[hud::DRIFT_RATIO] <= 1.0);
        assert_eq!(h[hud::MOMENTUM_PASS], 1.0, "residual {} against bound {}", h[hud::P_RESIDUAL], h[hud::P_BOUND]);
        // the array is what the Sim says, not a second derivation
        assert_eq!(h[hud::ENERGY_PASS] == 1.0, w.sim.energy_gate());
        assert_eq!(h[hud::LEDGER], w.sim.ledger());
        assert_eq!(h[hud::KNOTS], CURVE_KNOTS as f64);
        assert_eq!(h[hud::PAIR_PRESENT], 1.0);
        assert_eq!((h[hud::PAIR_Z_I], h[hud::PAIR_Z_J]), (1.0, 1.0));
        assert_eq!(h[hud::CALIBRATION], CALIBRATION_DONE as f64);
        assert_eq!(h[hud::DOF], 3.0);
    }

    #[test]
    fn the_buttons_do_what_the_bevy_handlers_do() {
        let mut w = World::new();
        w.load(0);
        w.calibrated(2.0e5);
        w.more_atoms();
        assert_eq!(w.sim.n, 3);
        w.fewer_atoms();
        w.fewer_atoms();
        assert_eq!(w.sim.n, 2, "never fewer than two");
        for _ in 0..40 {
            w.more_atoms();
        }
        assert_eq!(w.sim.n, DEFAULT_SCENE_ATOMS.min(w.n_max() as usize));
        let speed = w.sim.timescale.sim_speed_fs_per_wallsec;
        w.sim.timescale.sim_speed_fs_per_wallsec *= SPEED_STEP;
        w.sim.timescale.sim_speed_fs_per_wallsec /= SPEED_STEP;
        assert!((w.sim.timescale.sim_speed_fs_per_wallsec - speed).abs() < 1e-12 * speed);
        let mut seen = Vec::new();
        for _ in 0..3 {
            w.cycle_boundary();
            w.fill_hud();
            seen.push(w.hud[hud::BOUNDARY]);
            assert_eq!(w.sim.drift_peak, 0.0, "the ledger is re-based by the edit");
        }
        assert_eq!(seen, [1.0, 2.0, 0.0], "walls -> open -> periodic -> walls");
        let dt = w.sim.timescale.dt;
        w.toggle_dt_growth();
        assert!(w.sim.timescale.allow_dt_growth && w.sim.timescale.dt > dt);
        w.toggle_dt_growth();
        assert!(!w.sim.timescale.allow_dt_growth);
        assert!((w.sim.timescale.dt - dt).abs() < 1e-12, "dt is re-derived, not left grown");
        assert_eq!(w.next_preset(), holon_render::TABLE_OK);
        assert_eq!((w.preset, w.sim.n), (1, DEFAULT_SCENE_ATOMS));
        w.calibrated(f64::NAN);
        assert_eq!(w.calibration, CALIBRATION_UNAVAILABLE);
    }
}
