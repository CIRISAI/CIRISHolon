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
use holon_draw::ui::{self, Button, Hud, Item, Panel, Placed, Text};
use holon_draw::{Camera, Frame, Instance, Kind, Ring, Scene, Tile};
use holon_render::clock::{Rung, AU_TO_FS};
use holon_render::sim::{Boundary, Dims, Sim, DEFAULT_SCENE_ATOMS};
use std::cell::RefCell;
use std::collections::HashMap;

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

/// `holon-render-3d`'s `Preset::name` and `Preset::short_name`, in [`PRESETS`]' order.
pub const PRESET_NAMES: [&str; 12] = ["H₂ (2 atoms, approach)", "16-H Quench Gas (3-body active)", "LiH (Lithium Hydride)", "HF (Hydrogen Fluoride)", "Li₂ (Dilithium)", "N₂ (Dinitrogen)", "F₂ (Difluorine)", "CO (Carbon Monoxide)", "He₂ (Closed-shell bounce)", "Ne₂ (Closed-shell bounce)", "HCl (Hydrogen Chloride, 2 species)", "Mixed Gas: 8 H + 8 He (3 pair types)"];
pub const PRESET_SHORT: [&str; 12] = ["H₂", "16-H Quench", "LiH", "HF", "Li₂", "N₂", "F₂", "CO", "He₂", "Ne₂", "HCl", "8H+8He"];

/// HUD handles sit above every atom's tag (an atom's is its index plus one).
pub const HUD_BASE: u32 = 0x4000_0000;
/// A panel's handle is this plus its index (0 energy, 1 momentum, 2 clocks, 3 curve,
/// 4 census, 5 device).
pub const HANDLE_PANEL: u32 = HUD_BASE + 1;
/// A button's handle is this plus its action (`ACT_*`).
pub const HANDLE_BUTTON: u32 = HUD_BASE + 16;
pub const ACT_RESET: u32 = 0;
pub const ACT_PRESET: u32 = 1;
pub const ACT_FEWER: u32 = 2;
pub const ACT_MORE: u32 = 3;
pub const ACT_SLOWER: u32 = 4;
pub const ACT_FASTER: u32 = 5;
pub const ACT_WALLS: u32 = 6;
pub const ACT_THERMOSTAT: u32 = 7;
pub const ACT_COMPOSITE: u32 = 8;
pub const ACT_DT: u32 = 9;
pub const ACT_HUD: u32 = 10;
/// The overlay's text is rebuilt at most this often (and at once after a press, a load, a
/// resize or the calibration); between rebuilds the same screen tile is drawn again.
/// Bevy's overlay rewrites its text every frame.
pub const HUD_REBUILD_SECONDS: f64 = 0.25;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// The host's text provider (module `env`): rasterise `len` UTF-8 bytes at `ptr` in
    /// `style` and write `[w, h, u0, v0, u1, v1, page]` (7 f32, device pixels) at `out`.
    fn host_text(ptr: *const u8, len: u32, style: u32, out: *mut f32);
}

/// The text provider as this crate sees it: the host's on wasm, a stand-in natively (tests),
/// with a cache by string and style that lives for one rebuild of the overlay.
pub struct HostText {
    pub dpr: f32,
    cache: HashMap<(u8, String), Placed>,
    /// Calls that reached the provider, over the world's life.
    pub asked: u64,
}

impl HostText {
    /// A provider for `dpr` with an empty cache (the ladder's caption uses one of its own).
    pub fn new(dpr: f32) -> HostText {
        HostText { dpr, cache: HashMap::new(), asked: 0 }
    }
    /// Forget every placement (after the host's atlas was emptied).
    pub fn forget(&mut self) {
        self.cache.clear();
    }
}

impl Text for HostText {
    fn place(&mut self, text: &str, style: u8) -> Placed {
        if let Some(p) = self.cache.get(&(style, text.to_string())) {
            return *p;
        }
        self.asked += 1;
        #[cfg(target_arch = "wasm32")]
        let p = {
            let mut o = [0.0f32; 7];
            // SAFETY: the host reads `len` bytes at `ptr` and writes 7 f32 at `out`, both in this module's memory and alive for the call.
            unsafe { host_text(text.as_ptr(), text.len() as u32, style as u32, o.as_mut_ptr()) };
            Placed { w: o[0], h: o[1], u0: o[2], v0: o[3], u1: o[4], v1: o[5], page: o[6] as u8 }
        };
        #[cfg(not(target_arch = "wasm32"))]
        let p = ui::FakeText { dpr: self.dpr, placed: 0 }.place(text, style);
        self.cache.insert((style, text.to_string()), p);
        p
    }
}

/// The overlay's own state: what is open, what was last laid out, and when.
pub struct Overlay {
    pub dpr: f64,
    /// False hides the panels (the status line and the buttons stay), as Bevy's `hud` button.
    pub visible: bool,
    /// Narrow viewports: which panel is open. One at a time, so the scene stays in view.
    pub open: [bool; 6],
    pub layout: Option<ui::Layout>,
    pub text: HostText,
    /// Wall seconds the host has advanced by; the rebuild interval is measured on it.
    pub clock: f64,
    built_at: f64,
    built_for: [f64; 3],
    pub dirty: bool,
    pub rebuilds: u64,
}

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
    pub overlay: Overlay,
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
        World { sim, status: u32::MAX, preset: 0, calibration: CALIBRATION_PENDING, last_substeps: 0, last_frame_seconds: 0.0, hud: [0.0; HUD_LEN], overlay: Overlay { dpr: 1.0, visible: true, open: [false; 6], layout: None, text: HostText { dpr: 1.0, cache: HashMap::new(), asked: 0 }, clock: 0.0, built_at: 0.0, built_for: [0.0; 3], dirty: true, rebuilds: 0 }, frame: None, camera: None, vp: [0.0; 16] }
    }

    fn dimer(&mut self, a: Species, b: Species) -> u32 {
        let s = holon_render::generate_pair_table(&mut self.sim, a, b, 64);
        self.sim.reset(2);
        if !(self.sim.set_species(0, a) && self.sim.set_species(1, b)) {
            return u32::MAX - 1;
        }
        let (cx, cy, cz) = (0.5 * self.sim.width, 0.5 * self.sim.height, 0.5 * self.sim.depth);
        // `AtomWorld::setup_dimer`'s opening: 10 bohr apart, approaching at 0.0004 a.u. of
        // relative speed shared by mass, so the centre of mass is at rest
        let half = 5.0;
        let (ma, mb) = (self.sim.atoms[0].mass(), self.sim.atoms[1].mass());
        let mu = (ma * mb) / (ma + mb);
        let v_rel = 0.0004;
        let at = &mut self.sim.atoms;
        (at[0].x, at[0].y, at[0].z) = (cx - half, cy, cz);
        (at[1].x, at[1].y, at[1].z) = (cx + half, cy, cz);
        (at[0].vx, at[0].vy, at[0].vz) = (v_rel * (mu / ma), 0.0, 0.0);
        (at[1].vx, at[1].vy, at[1].vz) = (-v_rel * (mu / mb), 0.0, 0.0);
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
        self.overlay.dirty = true;
        status
    }

    pub fn ready(&self) -> bool {
        self.status == holon_render::TABLE_OK && self.sim.pairs_ready()
    }

    /// Load hydrogen's pair curve from knots the caller already holds, WITHOUT solving it,
    /// and place `n` hydrogen atoms: the file door of `holon-render` (`holon_table_begin`,
    /// `holon_table_knot`, `holon_table_finish`), for a host that cannot spend the solve's
    /// second on load. `force[i]` is the force at `r[i]` (so `dE/dR = -force`); `curvature`
    /// is the optional `d2E/dR2` column, pushed as `holon_table_knot_curvature` does (the
    /// engine's own generator supplies it). No three-body table is loaded, and the curve's
    /// provenance in the bank is left undeclared, as for any curve through the file door. Returns the curve's status (`holon_render::TABLE_OK` is
    /// success); the preset is left at 0 (H2). [`World::load`] is unchanged and still solves.
    #[allow(clippy::too_many_arguments)]
    pub fn load_curve(&mut self, r: &[f64], e: &[f64], force: &[f64], curvature: Option<&[f64]>, r_e: f64, d_e: f64, e_asymptote: f64, n: usize) -> u32 {
        self.sim.clear_bank();
        let refused = u32::MAX - 1;
        if r.len() != e.len() || r.len() != force.len() || curvature.is_some_and(|c| c.len() != r.len()) {
            self.status = refused;
            return refused;
        }
        let t = self.sim.table_mut();
        if !t.begin(r.len()) {
            self.status = refused;
            return refused;
        }
        for i in 0..r.len() {
            if !t.knot(i, r[i], e[i], force[i]) || curvature.is_some_and(|c| !t.knot_curvature(i, c[i])) {
                self.status = refused;
                return refused;
            }
        }
        let status = if t.finish(r_e, d_e, e_asymptote) == holon_render::table::LoadStatus::Ok { holon_render::TABLE_OK } else { refused };
        if status == holon_render::TABLE_OK {
            self.sim.adopt_table_timescale();
        }
        self.sim.reset(n);
        for i in 0..self.sim.n {
            self.sim.atoms[i].species = HYDROGEN;
        }
        self.preset = 0;
        self.status = status;
        self.overlay.dirty = true;
        status
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
        self.overlay.dirty = true;
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
        if wall_dt.is_finite() && wall_dt > 0.0 {
            self.overlay.clock += wall_dt;
        }
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
        Scene { tiles: vec![Tile { id: 1, origin: [0.0; 3], version: 1, layer: 0, screen: false, params: holon_draw::PARAMS_NEUTRAL, marks }], modal: None }
    }

    /// The overlay as `holon-draw`'s `ui` takes it: Bevy's six panels, its status line and
    /// its button row, every number from the HUD array ([`World::fill_hud`]) and every line
    /// in the form `holon-render-3d/src/hud.rs` prints it.
    pub fn hud_model(&mut self) -> Hud {
        use hud::*;
        self.fill_hud();
        let h = &self.hud;
        let s = &self.sim;
        let n = |v: f64| v as u64;
        let row = |l: &str, v: String| Item::Row(l.to_string(), v);
        let verdict = |pass: bool, what: &str| Item::Status(format!("{}  {what}", if pass { "PASS" } else { "FAIL" }), if pass { ui::PASS } else { ui::FAIL });
        let word = |pass: bool| (if pass { "PASS" } else { "FAIL" }.to_string(), if pass { ui::PASS } else { ui::FAIL });
        let panel = |k: u32, heading: &str, summary: (String, [u8; 4]), items: Vec<Item>| Panel { heading: heading.to_string(), handle: HANDLE_PANEL + k, open: self.overlay.open[k as usize], summary: summary.0, summary_colour: summary.1, items };
        let (ep, mp) = (h[ENERGY_PASS] == 1.0, h[MOMENTUM_PASS] == 1.0);
        let energy = vec![
            verdict(ep, "E - W_ext is constant"),
            row("E_kin", format!("{:+.6e}", h[E_KIN])),
            row("E_pair", format!("{:+.6e}", h[E_PAIR])),
            row("E_wall", format!("{:+.6e}", h[E_WALL])),
            row("E_spring", format!("{:+.6e}", h[E_SPRING])),
            row("W_ext", format!("{:+.6e}  (your hand)", h[W_EXT])),
            row("E - W_ext", format!("{:+.6e}", h[LEDGER])),
            row("origin", format!("{:+.6e}", h[ORIGIN])),
            row("drift peak", format!("{:.3e}", h[DRIFT_PEAK])),
            row("bound", format!("{:.3e}", h[DRIFT_BOUND])),
            row("ratio", format!("{:.4}", h[DRIFT_RATIO])),
        ];
        let momentum = vec![
            verdict(mp, "P - J_ext is constant"),
            row("|P|", format!("{:.6e}", h[P_MAG])),
            row("Px", format!("{:+.4e}", h[PX])),
            row("Py", format!("{:+.4e}", h[PY])),
            row("Pz", format!("{:+.4e}", h[PZ])),
            row("residual", format!("{:.3e}", h[P_RESIDUAL])),
            row("roundoff", format!("{:.3e}", h[P_BOUND])),
            row("ratio", format!("{:.4}", h[P_RATIO])),
            Item::Para("A separate gate on purpose: energy can be right while momentum is wrong.".into()),
        ];
        let (rung, rung_note, rung_colour) = match s.timescale.rung {
            Rung::Exact => ("EXACT", "requested speed delivered at the derived dt", ui::PASS),
            Rung::TimeDilated => ("TIME DILATED", "accuracy held; there are fewer steps per second", ui::AMBER),
            Rung::AccuracyDeclared => ("ACCURACY DECLARED", "dt grew by your toggle; the bound below is the re-derived one", ui::AMBER),
            Rung::Refused => ("REFUSED", "omega*dt reached the stability limit; nothing lawful remains", ui::FAIL),
        };
        let clocks = vec![
            Item::Status(format!("{rung}\n{rung_note}"), rung_colour),
            row("1 | physics dt", format!("{:.5} a.u. (derived)", h[DT])),
            row("|  reference", format!("{:.5} = period/64", h[DT_REFERENCE])),
            row("|  w_e at R_e", format!("{:.6}", h[OMEGA_E])),
            row("|  w_env", format!("{:.6} (envelope)", h[OMEGA_ENV])),
            row("|  w_env * dt", format!("{:.5} (<2 stable)", h[OMEGA_DT])),
            row("2 | frame", format!("{:.1} ms", h[FRAME_MS])),
            row("|  substeps", format!("{}", n(h[SUBSTEPS]))),
            row("3 | sim-speed", format!("{:.4} fs/s", h[SIM_SPEED])),
            row("|  1 vibration", format!("{:.2} s", h[VIBRATION_S])),
            row("|  dilation", format!("{:.4}", h[DILATION])),
        ];
        let preset = (self.preset as usize).min(PRESETS.len() - 1);
        let mut curve = vec![
            row("preset", PRESET_NAMES[preset].to_string()),
            row("R_e", format!("{:.5} bohr", h[R_E])),
            row("D_e", format!("{:.6} Eh", h[D_E])),
            row("asymptote", format!("{:.6} Eh", h[ASYMPTOTE])),
            row("knots", format!("{}", n(h[KNOTS]))),
        ];
        if h[PAIR_PRESENT] == 1.0 {
            let (i, j) = (n(h[PAIR_I]) as usize, n(h[PAIR_J]) as usize);
            curve.extend([
                row("closest pair", format!("{}{}-{}{}", s.atoms[i].species.symbol, i, s.atoms[j].species.symbol, j)),
                row("|  R", format!("{:.4} bohr", h[PAIR_R])),
                row("|  E_rel", format!("{:+.4e} Eh", h[PAIR_E_REL])),
                row("|  r_outer", format!("{:.4}", h[PAIR_R_OUTER])),
                row("|  bonded", if h[PAIR_BONDED] == 1.0 { "yes" } else { "no" }.to_string()),
            ]);
        } else {
            curve.push(row("closest pair", "(none)".to_string()));
        }
        curve.push(row("T", format!("{:.1} K  ({} d.o.f./atom)", h[TEMPERATURE], n(h[DOF]))));
        curve.push(Item::Para("Bonded means E_rel below the asymptote and R inside the turning point. No distance cutoff anywhere.".into()));
        let census = vec![
            row("micro . atoms", format!("{}", n(h[CENSUS_ATOMS]))),
            row("composite . molecules", format!("{}", n(h[CENSUS_MOLECULES]))),
            row("candidate . evals/fr", format!("{}", n(h[CENSUS_CANDIDATES]))),
            row("global . views", format!("{}", n(h[CENSUS_VIEWS]))),
            row("formations", format!("{}", n(h[CENSUS_FORMATIONS]))),
            row("dissolutions", format!("{}", n(h[CENSUS_DISSOLUTIONS]))),
            row("bound but not closed", format!("{}", n(h[CENSUS_NOT_CLOSED]))),
            row("bond-sector energy", format!("{:+.3e}", h[BOND_SECTOR_ENERGY])),
            Item::Para("Being matter is expensive - the O(N*N) force loop is the whole budget. Being a holon is cheap.".into()),
        ];
        let cal = match self.calibration {
            CALIBRATION_PENDING => "measuring...",
            CALIBRATION_DONE => "measured on load",
            _ => "no clock; budget unlimited",
        };
        let device = vec![
            row("substeps / sec", format!("{:.4e}", h[SUBSTEPS_PER_SEC])),
            row("pairs / sec", format!("{:.4e}", h[PAIRS_PER_SEC])),
            row("required s/sec", format!("{:.4e}", h[REQUIRED_PER_SEC])),
            row("N_max here", format!("{}", n(h[N_MAX]))),
            row("calibration", cal.to_string()),
            Item::Para("Not a guess, and not this developer's machine.".into()),
        ];
        let ok = h[TABLE_OK] == 1.0;
        let status = if ok {
            format!("[{}]  t = {:.1} a.u. ({:.3} fs)  |  {} atoms  |  {} clusters  |  {} bonded pairs  |  drag an atom; the spring is on the ledger", PRESET_SHORT[preset], h[TIME_AU], h[TIME_FS], n(h[ATOMS]), n(h[CLUSTERS]), n(h[BONDED_PAIRS]))
        } else {
            format!("THE CURVE DID NOT LOAD (status {}) - nothing is being integrated", self.status)
        };
        let button = |act: u32, label: String, active: bool| Button { label, handle: HANDLE_BUTTON + act, active };
        let on = |b: bool| if b { "ON" } else { "OFF" };
        let buttons = vec![
            button(ACT_RESET, "reset".into(), true),
            button(ACT_PRESET, format!("preset: {}", PRESET_SHORT[preset]), true),
            button(ACT_FEWER, "-".into(), true),
            button(ACT_MORE, format!("+  ({} atoms)", s.n), true),
            button(ACT_SLOWER, "slower".into(), true),
            button(ACT_FASTER, format!("faster  ({:.3} fs/s)", h[SIM_SPEED]), true),
            button(
                ACT_WALLS,
                match s.boundary {
                    Boundary::Walls => "walls ON",
                    Boundary::Open => "walls OFF",
                    Boundary::Periodic => "periodic (wrapping)",
                }
                .into(),
                s.boundary != Boundary::Open,
            ),
            button(ACT_THERMOSTAT, format!("thermostat {}", on(s.thermostat_on)), s.thermostat_on),
            button(ACT_COMPOSITE, format!("composite {}", on(s.holons.enabled)), s.holons.enabled),
            button(ACT_DT, if s.timescale.allow_dt_growth { "dt GROWN (accuracy declared)" } else { "dt held" }.into(), s.timescale.allow_dt_growth),
            button(ACT_HUD, "hud".into(), self.overlay.visible),
        ];
        Hud {
            left: vec![panel(0, "GATE 1 / ENERGY", word(ep), energy), panel(1, "GATE 2 / MOMENTUM", word(mp), momentum), panel(2, "THE THREE CLOCKS", (rung.to_string(), rung_colour), clocks)],
            right: vec![
                panel(3, "THE CURVE / U(R)", (PRESET_SHORT[preset].to_string(), ui::PAPER), curve),
                panel(4, "HOLON CENSUS", (format!("{} atoms, {} mol", n(h[CENSUS_ATOMS]), n(h[CENSUS_MOLECULES])), ui::PAPER), census),
                panel(5, "THIS DEVICE (MEASURED)", (if self.calibration == CALIBRATION_PENDING { "measuring...".to_string() } else { format!("N_max {}", n(h[N_MAX])) }, ui::PAPER), device),
            ],
            panels: self.overlay.visible,
            status,
            status_colour: if ok { ui::PAPER } else { ui::FAIL },
            buttons,
        }
    }

    /// The overlay's screen tile for a viewport in device pixels: laid out again when
    /// something changed it (a press, a load, a resize, the calibration) or when
    /// [`HUD_REBUILD_SECONDS`] have passed, and otherwise the tile of the last layout.
    pub fn overlay_tile(&mut self, width: f64, height: f64) -> Tile {
        let o = &self.overlay;
        let key = [width, height, o.dpr];
        if o.layout.is_none() || o.dirty || o.built_for != key || o.clock - o.built_at >= HUD_REBUILD_SECONDS {
            let model = self.hud_model();
            let o = &mut self.overlay;
            o.text.cache.clear();
            o.text.dpr = o.dpr as f32;
            o.layout = Some(ui::layout(&model, &mut o.text, width as f32, height as f32, o.dpr as f32, 2, 1, 0));
            o.built_at = o.clock;
            o.built_for = key;
            o.dirty = false;
            o.rebuilds += 1;
        }
        self.overlay.layout.as_ref().expect("laid out above").tile.clone()
    }

    /// What a HUD handle does when pressed: a button is `holon-render-3d`'s `handle_actions`
    /// arm; a panel's heading opens or closes it on a narrow viewport (one open at a time)
    /// and is inert on a wide one, where a panel only stops a pick going through it.
    pub fn press(&mut self, handle: u32) {
        self.overlay.dirty = true;
        if (HANDLE_PANEL..HANDLE_PANEL + 6).contains(&handle) {
            if self.overlay.layout.as_ref().is_some_and(|l| l.narrow) {
                let k = (handle - HANDLE_PANEL) as usize;
                let was = self.overlay.open[k];
                self.overlay.open = [false; 6];
                self.overlay.open[k] = !was;
            }
            return;
        }
        match handle.wrapping_sub(HANDLE_BUTTON) {
            ACT_RESET => {
                let n = self.sim.n;
                self.reset(n)
            }
            ACT_PRESET => {
                self.next_preset();
            }
            ACT_FEWER => self.fewer_atoms(),
            ACT_MORE => self.more_atoms(),
            ACT_SLOWER => self.sim.timescale.sim_speed_fs_per_wallsec /= SPEED_STEP,
            ACT_FASTER => self.sim.timescale.sim_speed_fs_per_wallsec *= SPEED_STEP,
            ACT_WALLS => self.cycle_boundary(),
            ACT_THERMOSTAT => self.sim.thermostat_on = !self.sim.thermostat_on,
            ACT_COMPOSITE => self.sim.holons.enabled = !self.sim.holons.enabled,
            ACT_DT => self.toggle_dt_growth(),
            ACT_HUD => self.overlay.visible = !self.overlay.visible,
            _ => {}
        }
    }

    /// A pointer going down at a pixel: a HUD button acts and its handle is returned; a
    /// panel returns its handle and takes the press; otherwise the atom there is grabbed
    /// (its index plus one), or 0. A handle is at least [`HUD_BASE`]; an atom never is.
    pub fn click(&mut self, x: f64, y: f64) -> u32 {
        match self.frame.as_ref().and_then(|f| f.pick(x, y)) {
            Some(t) if t >= HUD_BASE => {
                self.press(t);
                t
            }
            _ => self.grab_at(x, y),
        }
    }

    /// Build the frame for an orbit view about the box's centre: the atoms' tile and, over
    /// it, the overlay's screen tile.
    pub fn view(&mut self, yaw: f64, pitch: f64, dist: f64, width: f64, height: f64) -> usize {
        let h = 0.5 * BOX_SIDE;
        let cam = Camera::orbit([h, h, h], yaw, pitch, dist, std::f64::consts::FRAC_PI_4, width, height);
        let mut scene = self.scene();
        scene.tiles.push(self.overlay_tile(width, height));
        let f = scene.frame(&cam, 1, &Ring, u64::MAX).expect("the atoms scene has no non-finite field");
        self.vp = cam.relative();
        let n = f.batches.len();
        self.frame = Some(f);
        self.camera = Some(cam);
        n
    }

    /// Grab the atom under a pixel; returns its index plus one, or 0.
    pub fn grab_at(&mut self, x: f64, y: f64) -> u32 {
        let Some(t) = self.frame.as_ref().and_then(|f| f.pick(x, y)).filter(|&t| t < HUD_BASE) else { return 0 };
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

/// A pointer down at a pixel: a HUD button acts and returns its handle (at least
/// [`HUD_BASE`]); a panel returns its handle; otherwise the atom there is grabbed and its
/// index plus one returned, or 0.
#[no_mangle]
pub extern "C" fn ha_click(x: f64, y: f64) -> u32 {
    with(|w| w.click(x, y))
}
/// The device pixel ratio the overlay is laid out at (the text provider's own).
#[no_mangle]
pub extern "C" fn ha_set_dpr(dpr: f64) {
    with(|w| {
        if dpr.is_finite() && dpr > 0.0 {
            w.overlay.dpr = dpr;
            w.overlay.dirty = true;
        }
    })
}
/// The host's text atlas dropped a page (its epoch moved): lay the overlay out again.
#[no_mangle]
pub extern "C" fn ha_hud_stale() {
    with(|w| w.overlay.dirty = true)
}
/// How many times the overlay has been laid out.
#[no_mangle]
pub extern "C" fn ha_hud_rebuilds() -> f64 {
    with(|w| w.overlay.rebuilds as f64)
}
/// Where a tagged instance (an atom's index plus one, or a HUD handle) is on the last frame:
/// `edge` 0 left, 1 top, 2 right, 3 bottom, in device pixels; NaN if it is not drawn.
#[no_mangle]
pub extern "C" fn ha_bounds(tag: u32, edge: u32) -> f64 {
    with(|w| w.frame.as_ref().and_then(|f| f.report(tag)).and_then(|r| r.bounds.get(edge as usize).copied()).unwrap_or(f64::NAN))
}
/// 1 when the overlay's last layout was the single collapsible column.
#[no_mangle]
pub extern "C" fn ha_hud_narrow() -> u32 {
    with(|w| w.overlay.layout.as_ref().is_some_and(|l| l.narrow) as u32)
}
/// 1 for a screen tile's batch (device pixels, drawn over the world).
#[no_mangle]
pub extern "C" fn ha_batch_screen(i: u32) -> u32 {
    batch(i, |b| b.screen as u32, 0)
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
            assert_eq!(f.marks.iter().filter(|m| m.rect.is_none()).count(), w.sim.n, "every atom is a tagged instance");
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

    fn texts(w: &mut World) -> Vec<String> {
        let m = w.hud_model();
        let mut out = vec![m.status.clone()];
        for p in m.left.iter().chain(&m.right) {
            out.push(p.heading.clone());
            for it in &p.items {
                out.push(match it {
                    Item::Status(s, _) | Item::Para(s) => s.clone(),
                    Item::Row(l, v) => format!("{l} = {v}"),
                });
            }
        }
        out.extend(m.buttons.iter().map(|b| b.label.clone()));
        out
    }

    #[test]
    fn the_overlay_is_bevys_six_panels_status_line_and_eleven_buttons_and_fits_both_viewports() {
        let mut w = World::new();
        w.load(0);
        w.calibrated(2.0e5);
        for _ in 0..120 {
            w.advance(1.0 / 60.0);
        }
        let t = texts(&mut w);
        for want in [
            "GATE 1 / ENERGY", "PASS  E - W_ext is constant", "GATE 2 / MOMENTUM", "PASS  P - J_ext is constant", "THE THREE CLOCKS", "EXACT\nrequested speed delivered at the derived dt",
            "THE CURVE / U(R)", "preset = H₂ (2 atoms, approach)", "knots = 492", "closest pair = H0-H1", "|  bonded = no", "HOLON CENSUS", "micro . atoms = 2", "THIS DEVICE (MEASURED)",
            "calibration = measured on load", "reset", "preset: H₂", "-", "+  (2 atoms)", "slower", "walls ON", "thermostat OFF", "composite ON", "dt held", "hud",
        ] {
            assert!(t.iter().any(|s| s == want), "the overlay lacks {want:?}");
        }
        assert!(t[0].starts_with("[H₂]  t = ") && t[0].ends_with("|  2 atoms  |  0 clusters  |  0 bonded pairs  |  drag an atom; the spring is on the ledger"), "{}", t[0]);
        assert!(t.iter().any(|s| s.starts_with("W_ext = +0.000000e0  (your hand)")));
        assert!(t.iter().any(|s| s.starts_with("faster  (") && s.ends_with(" fs/s)")));
        let m = w.hud_model();
        assert_eq!((m.left.len(), m.right.len(), m.buttons.len()), (3, 3, 11));
        // wide: two columns that fit 1280 by 800 with the bar under them
        w.view(0.6, 0.35, 60.0, 1280.0, 800.0);
        let l = w.overlay.layout.as_ref().unwrap();
        assert!(!l.narrow, "Bevy's panels fit 1280 x 800 in two columns");
        assert!(l.panels.iter().all(|p| p.1[3] <= l.status[1]) && l.buttons.iter().all(|b| b.1[3] <= 800.0));
        // the longest rung note (dt grown) still fits: the placement does not flip on a press
        w.press(HANDLE_BUTTON + ACT_DT);
        w.advance(1.0 / 60.0);
        w.view(0.6, 0.35, 60.0, 1280.0, 800.0);
        assert!(w.sim.timescale.rung == Rung::AccuracyDeclared && !w.overlay.layout.as_ref().unwrap().narrow);
        w.press(HANDLE_BUTTON + ACT_DT);
        // phone: one column of closed headings, then one opened at a time
        w.overlay.dpr = 3.0;
        w.view(0.6, 0.35, 60.0, 1170.0, 2532.0);
        let l = w.overlay.layout.clone().unwrap();
        assert!(l.narrow && l.panels.iter().all(|p| p.1[3] - p.1[1] < 120.0));
        assert!(l.scene[3] - l.scene[1] > 0.5 * 2532.0, "closed, the panels leave the scene more than half the screen: {:?}", l.scene);
        for k in 0..6u32 {
            let r = w.overlay.layout.as_ref().unwrap().panels[k as usize].1;
            assert_eq!(w.click(0.5 * (r[0] + r[2]) as f64, r[1] as f64 + 20.0), HANDLE_PANEL + k);
            w.view(0.6, 0.35, 60.0, 1170.0, 2532.0);
            let open = w.overlay.layout.as_ref().unwrap();
            assert_eq!(w.overlay.open.iter().filter(|o| **o).count(), 1, "one open at a time");
            assert!(open.panels[k as usize].1[3] - open.panels[k as usize].1[1] > 300.0);
            let bar_top = open.status[1];
            assert!(open.panels.iter().all(|p| p.1[3] <= bar_top), "panel {k} open runs into the status line");
            assert!(open.scene[3] - open.scene[1] > 0.12 * 2532.0, "panel {k} open leaves {:?}", open.scene);
        }
    }

    #[test]
    fn a_click_presses_a_button_takes_a_panel_or_falls_through_to_an_atom() {
        let mut w = World::new();
        w.load(0);
        w.calibrated(2.0e5);
        w.view(0.6, 0.35, 60.0, 1280.0, 800.0);
        let l = w.overlay.layout.clone().unwrap();
        let at = |act: u32| {
            let r = l.buttons.iter().find(|b| b.0 == HANDLE_BUTTON + act).unwrap().1;
            (0.5 * (r[0] + r[2]) as f64, 0.5 * (r[1] + r[3]) as f64)
        };
        let (x, y) = at(ACT_MORE);
        assert_eq!(w.click(x, y), HANDLE_BUTTON + ACT_MORE);
        assert_eq!(w.sim.n, 3);
        assert!(w.sim.grabbed.is_none(), "a button is not a grab");
        let (x, y) = at(ACT_WALLS);
        assert_eq!(w.click(x, y), HANDLE_BUTTON + ACT_WALLS);
        assert!(w.sim.boundary == Boundary::Open);
        let (x, y) = at(ACT_HUD);
        w.click(x, y);
        w.view(0.6, 0.35, 60.0, 1280.0, 800.0);
        assert!(w.overlay.layout.as_ref().unwrap().panels.is_empty(), "hud hides the panels");
        assert_eq!(w.overlay.layout.as_ref().unwrap().buttons.len(), 11, "and keeps the buttons");
        w.click(x, y);
        w.view(0.6, 0.35, 60.0, 1280.0, 800.0);
        // a wide panel takes the press and does nothing with it; an atom under it is not grabbed
        let p = l.panels[0].1;
        assert_eq!(w.click(p[0] as f64 + 30.0, p[1] as f64 + 30.0), HANDLE_PANEL);
        assert_eq!(w.grab_at(p[0] as f64 + 30.0, p[1] as f64 + 30.0), 0);
        assert!(w.sim.grabbed.is_none() && w.overlay.open == [false; 6]);
        // the scene: an atom where it is drawn
        w.view(0.6, 0.35, 60.0, 1280.0, 800.0);
        let a = w.sim.atoms[0];
        let q = w.camera.as_ref().unwrap().project([a.x, a.y, a.z]).unwrap();
        assert_eq!(w.click(q[0], q[1]), 1);
        assert_eq!(w.sim.grabbed, Some(0));
    }

    #[test]
    fn the_overlay_is_laid_out_again_four_times_a_second_and_after_a_press_and_reused_between() {
        let mut w = World::new();
        w.load(0);
        w.calibrated(2.0e5);
        w.view(0.6, 0.35, 60.0, 1280.0, 800.0);
        let (r0, asked0) = (w.overlay.rebuilds, w.overlay.text.asked);
        assert!(asked0 > 100, "the first layout places its runs");
        for _ in 0..60 {
            w.advance(1.0 / 60.0);
            w.view(0.6, 0.35, 60.0, 1280.0, 800.0);
        }
        let per_second = w.overlay.rebuilds - r0;
        assert!((3..=5).contains(&per_second), "{per_second} layouts in a second of frames");
        // the frame carries the overlay as screen batches after the world's, every frame
        let f = w.frame.as_ref().unwrap();
        let first_screen = f.batches.iter().position(|b| b.screen).unwrap();
        assert!(first_screen >= 2 && f.batches[first_screen..].iter().all(|b| b.screen));
        assert_eq!(f.batches[first_screen..].iter().map(|b| b.kind).collect::<Vec<_>>(), [Kind::Quad, Kind::Run]);
        let before = w.overlay.rebuilds;
        w.view(0.6, 0.35, 60.0, 1280.0, 800.0);
        assert_eq!(w.overlay.rebuilds, before, "no time passed: the same tile");
        w.press(HANDLE_BUTTON + ACT_FASTER);
        w.view(0.6, 0.35, 60.0, 1280.0, 800.0);
        assert_eq!(w.overlay.rebuilds, before + 1, "a press shows at once");
        w.view(0.6, 0.35, 60.0, 1000.0, 800.0);
        assert_eq!(w.overlay.rebuilds, before + 2, "so does a resize");
    }

    #[test]
    fn a_dimer_opens_as_bevys_does_ten_bohr_apart_and_approaching_with_no_net_momentum() {
        // hydrogen through the dimer path: the heavier pairs take the same lines and minutes to solve
        let mut w = World::new();
        assert_eq!(w.dimer(HYDROGEN, HELIUM), holon_render::TABLE_OK);
        let (a, b) = (w.sim.atoms[0], w.sim.atoms[1]);
        assert_eq!((a.x, b.x), (0.5 * BOX_SIDE - 5.0, 0.5 * BOX_SIDE + 5.0));
        assert_eq!((a.y, a.z, b.y, b.z), (12.0, 12.0, 12.0, 12.0));
        assert!((a.vx - b.vx - 0.0004).abs() < 1e-15 && a.vx > 0.0 && b.vx < 0.0, "closing at 0.0004");
        assert!((a.mass() * a.vx + b.mass() * b.vx).abs() < 1e-12 && a.mass() != b.mass());
        assert_eq!((a.vy, a.vz, b.vy, b.vz), (0.0, 0.0, 0.0, 0.0));
    }
}
