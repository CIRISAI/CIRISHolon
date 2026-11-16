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
use holon_render::sim::{Boundary, Dims, Sim, DEFAULT_SCENE_ATOMS};
use std::cell::RefCell;

pub const BOX_SIDE: f64 = 24.0;
pub const ATOM_RADIUS: f64 = 0.6;
const CURVE_R_MIN: f64 = 0.3;
const CURVE_R_MAX: f64 = 10.0;
const CURVE_KNOTS: usize = 492;

/// The presets, in `holon-render-3d`'s order.
pub const PRESETS: [&str; 12] = ["H2", "Quench16", "LiH", "HF", "Li2", "N2", "F2", "CO", "He2", "Ne2", "HCl", "MixedGas"];

pub struct World {
    pub sim: Box<Sim>,
    pub status: u32,
    pub preset: u32,
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
        World { sim, status: u32::MAX, preset: 0, frame: None, camera: None, vp: [0.0; 16] }
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
        }
    }

    /// One frame of `wall_dt` measured seconds, as `AtomWorld::advance`.
    pub fn advance(&mut self, wall_dt: f64) -> u32 {
        if !self.ready() {
            return 0;
        }
        let budget = self.sim.timescale.substep_budget(wall_dt);
        let plan = self.sim.timescale.plan_frame(wall_dt, budget);
        self.sim.step_frame(plan.substeps);
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
        Scene { tiles: vec![Tile { id: 1, origin: [0.0; 3], version: 1, layer: 0, marks }], modal: None }
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
}
