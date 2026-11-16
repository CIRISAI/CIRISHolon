//! Hydrogen, simulated live: `holon-render`'s `Sim` through `holon_draw_atoms::World`.
use super::*;
use holon_draw::ladder::Status;
use holon_draw_atoms::World;

/// Ångströms. The simulation's own unit is the bohr; marks are converted.
pub const UNIT: i32 = -10;
pub const ANGSTROM_PER_BOHR: f64 = 0.529_177_210_9;
/// The pair curve the atoms page ships (`docs/atoms/h2_potential.json`, 6,000 bytes), built
/// in. It is the page's FILE door: a Morse curve fitted to H2's spectroscopic constants, a
/// placeholder by its own provenance line, and not the STO-3G full-CI curve the engine can
/// solve (that solve is what the host's load budget forbids).
const CURVE: &str = include_str!("../../../../../../docs/atoms/h2_potential.json");
/// Boltzmann's constant in hartree per kelvin.
const K_B: f64 = 3.166_811_563e-6;

pub struct Atoms {
    pub world: World,
    /// Set by `tiles`, cleared by `advance`: the simulation steps only while it is on screen.
    seen: bool,
    /// Substeps taken over the scene's life.
    pub substeps: u64,
}

/// The number after `"key":` in a flat JSON object.
fn number(json: &str, key: &str) -> Option<f64> {
    let at = json.find(&format!("\"{key}\""))? + key.len() + 2;
    let rest = json[at..].trim_start().strip_prefix(':')?.trim_start();
    let end = rest.find(|c: char| !(c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E'))).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// The array of numbers after `"key":`.
fn array(json: &str, key: &str) -> Option<Vec<f64>> {
    let at = json.find(&format!("\"{key}\""))? + key.len() + 2;
    let rest = json[at..].trim_start().strip_prefix(':')?.trim_start().strip_prefix('[')?;
    let body = &rest[..rest.find(']')?];
    body.split(',').map(|v| v.trim().parse().ok()).collect()
}

impl Atoms {
    /// The shipped curve, parsed: R (bohr), E (hartree), F (hartree per bohr), then R_e, D_e
    /// and the asymptote.
    #[allow(clippy::type_complexity)]
    pub fn curve() -> Option<(Vec<f64>, Vec<f64>, Vec<f64>, f64, f64, f64)> {
        Some((array(CURVE, "R_grid_bohr")?, array(CURVE, "E_hartree")?, array(CURVE, "F_hartree_per_bohr")?, number(CURVE, "R_e")?, number(CURVE, "D_e")?, number(CURVE, "E_asymptote")?))
    }

    pub fn new() -> Atoms {
        let mut world = World::new();
        if let Some((r, e, f, r_e, d_e, e_inf)) = Self::curve() {
            if world.load_curve(&r, &e, &f, r_e, d_e, e_inf, 2) == holon_render_ok() && world.sim.n == 2 {
                // One H2 molecule at the middle of the box, bound: stretched 15% beyond its
                // equilibrium length and at rest along the bond, so it vibrates; turning with
                // kT of rotational energy at 300 K about an axis tilted out of the page.
                let c = 0.5 * world.sim.width;
                let half = 0.5 * 1.15 * r_e;
                let m = world.sim.atoms[0].mass();
                let v = 0.5 * (2.0 * K_B * 300.0 / (0.5 * m)).sqrt();
                let (ty, tz) = (35f64.to_radians().cos(), 35f64.to_radians().sin());
                let at = &mut world.sim.atoms;
                (at[0].x, at[0].y, at[0].z) = (c - half, c, c);
                (at[1].x, at[1].y, at[1].z) = (c + half, c, c);
                (at[0].vx, at[0].vy, at[0].vz) = (0.0, v * ty, v * tz);
                (at[1].vx, at[1].vy, at[1].vz) = (0.0, -v * ty, -v * tz);
                world.sim.rebase();
            }
        }
        Atoms { world, seen: false, substeps: 0 }
    }

    /// The distance between the two atoms, in Å.
    pub fn bond(&self) -> f64 {
        let a = &self.world.sim.atoms;
        ((a[1].x - a[0].x).powi(2) + (a[1].y - a[0].y).powi(2) + (a[1].z - a[0].z).powi(2)).sqrt() * ANGSTROM_PER_BOHR
    }
}

fn holon_render_ok() -> u32 {
    1 // holon_render::TABLE_OK; this crate does not depend on holon-render by name
}

impl Default for Atoms {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for Atoms {
    fn name(&self) -> &'static str {
        "hydrogen, atom by atom"
    }
    fn unit(&self) -> i32 {
        UNIT
    }
    fn status(&self) -> Status {
        Status::Simulated
    }
    fn source(&self) -> &'static str {
        "Computed as you watch by this repository's engine (holon-render's Sim, through holon-draw-atoms): one H2 molecule, started stretched 15% and turning with kT at 300 K, in the engine's 24-bohr walled box (the frame), followed by its first atom. The pair curve is the one the atoms page ships, docs/atoms/h2_potential.json: a Morse curve fitted to H2's measured constants (D_e 0.17449 Eh, R_e 1.40112 bohr), a placeholder by its own provenance, loaded instead of the engine's own STO-3G solve, which takes over a second. No three-body table is loaded; two atoms need none. Atoms are drawn as the engine draws them, as balls of 0.694 bohr (half the bond length of its own solved curve)."
    }
    fn span(&self) -> (f64, f64) {
        (-10.3 - VIEW_DECADES, -9.0 - VIEW_DECADES)
    }
    fn advance(&mut self, dt: f64) {
        if self.seen && dt.is_finite() && dt > 0.0 {
            self.substeps += self.world.advance(dt.min(0.1)) as u64;
        }
        self.seen = false;
    }
    fn tiles(&mut self, _z: f64, _clock: f64, _alpha: f64) -> Vec<Tile> {
        self.seen = true;
        if !self.world.ready() || self.world.sim.n == 0 {
            return Vec::new();
        }
        // the engine's own marks (atoms, a rod per bonded pair, the box), moved to the first
        // atom and converted from bohr to Å
        let a = &self.world.sim.atoms[0];
        // (the engine's marks are 32-bit; the centre is taken at the same precision, so the
        // first atom is exactly the origin)
        let (c, k) = ([a.x as f32 as f64, a.y as f32 as f64, a.z as f32 as f64], ANGSTROM_PER_BOHR);
        let to = |p: [f32; 3]| [((p[0] as f64 - c[0]) * k) as f32, ((p[1] as f64 - c[1]) * k) as f32, ((p[2] as f64 - c[2]) * k) as f32];
        let mut tiles = self.world.scene().tiles;
        for t in tiles.iter_mut() {
            for m in t.marks.iter_mut() {
                m.pos = to(m.pos);
                if matches!(m.kind, Kind::Rod | Kind::Line) {
                    m.aux = to(m.aux);
                }
                m.size[0] *= k as f32;
            }
        }
        tiles
    }
}
