//! Inside the proton: three quarks and the flux tubes that join them. A picture.
use super::nucleus::{PROTON, PROTON_RADIUS_FM};
use super::*;
use holon_draw::ladder::Status;

/// Tenths of a femtometre.
pub const UNIT: i32 = -16;
/// The proton's charge radius in this rung's unit.
pub const PROTON_RADIUS: f32 = PROTON_RADIUS_FM * 10.0;
/// The size a quark is DRAWN at (no size has been measured; the bound is under 1e-18 m).
const QUARK: f32 = 0.55;
const TUBE: f32 = 0.34;
/// The centre the quarks turn about. The zoom goes in beside the tubes, not into one: the
/// origin is a point of the proton's inside with nothing drawn at it.
const CENTRE: [f32; 3] = [2.2, -1.6, 1.2];

pub struct Hadron;

impl Hadron {
    pub fn new() -> Hadron {
        Hadron
    }
    /// The three quarks at `clock`, in this rung's unit: slow orbits on the clock about a point 0.3 fm from the proton's centre.
    pub fn quarks(clock: f64) -> [[f32; 3]; 3] {
        let mut q = [[0.0f32; 3]; 3];
        for (i, p) in q.iter_mut().enumerate() {
            let k = i as f64 * std::f64::consts::TAU / 3.0;
            let t = 0.35 * clock + k;
            let tilt = 0.6 * (0.23 * clock + 1.7 * i as f64).sin();
            let r = 3.6 + 0.7 * (0.51 * clock + 2.1 * i as f64).sin();
            *p = [(r * t.cos() * tilt.cos()) as f32 + CENTRE[0], (r * t.sin() * tilt.cos()) as f32 + CENTRE[1], (r * tilt.sin()) as f32 + CENTRE[2]];
        }
        q
    }
    /// The point whose distances to the three quarks sum to the least: where the tubes meet.
    pub fn junction(q: &[[f32; 3]; 3]) -> [f32; 3] {
        let mut j = [(q[0][0] + q[1][0] + q[2][0]) / 3.0, (q[0][1] + q[1][1] + q[2][1]) / 3.0, (q[0][2] + q[1][2] + q[2][2]) / 3.0];
        for _ in 0..12 {
            let (mut n, mut w) = ([0.0f32; 3], 0.0f32);
            for p in q {
                let d = ((p[0] - j[0]).powi(2) + (p[1] - j[1]).powi(2) + (p[2] - j[2]).powi(2)).sqrt().max(1e-3);
                for c in 0..3 {
                    n[c] += p[c] / d;
                }
                w += 1.0 / d;
            }
            j = [n[0] / w, n[1] / w, n[2] / w];
        }
        j
    }
}

impl Default for Hadron {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for Hadron {
    fn name(&self) -> &'static str {
        "inside the proton"
    }
    fn unit(&self) -> i32 {
        UNIT
    }
    fn status(&self) -> Status {
        Status::Illustration
    }
    fn source(&self) -> &'static str {
        "A picture, not a computation: a proton as three quarks (two up, one down) joined by flux tubes that meet at a point, inside its 0.84 fm charge radius (CODATA; the shell and rings). The quarks' positions, motion and drawn size are invented (no size of a quark has been measured). What this repository HAS measured below the atom is in one space dimension only (TIERS.md, the fold below the atom: SCHWINGER-3, M_V/g = 0.553116, 2.0% from the continuum; GF0 SCHWINGER-4, the screened pair's residual decaying at the meson mass to 0.6%). Nothing here computes a hadron in three."
    }
    fn span(&self) -> (f64, f64) {
        (-16.0 - VIEW_DECADES, -14.5 - VIEW_DECADES)
    }
    fn tiles(&mut self, z: f64, clock: f64, _alpha: f64) -> Vec<Tile> {
        let z = z + VIEW_DECADES; // the view's height, as the rest of this function has it
        let view = 10f64.powf(z - UNIT as f64) as f32;
        let q = Self::quarks(clock);
        let j = Self::junction(&q);
        let colours = [[226u8, 70, 64, 255], [78, 196, 96, 255], [78, 118, 236, 255]];
        let mut marks = Vec::with_capacity(160);
        // the tubes breathe a little, on the clock
        let tube = TUBE * (1.0 + 0.12 * ((2.3 * clock).sin() as f32));
        for (p, c) in q.iter().zip(colours) {
            marks.push(rod(*p, j, tube, [238, 214, 150, 255]));
            marks.push(sphere(*p, QUARK, c));
        }
        marks.push(sphere(j, 1.25 * tube, [238, 214, 150, 255]));
        // the charge radius: three rings, and a faint shell drawn last
        let n = 48;
        let w = (0.0022 * view).max(0.02);
        for plane in 0..3 {
            let at = |i: usize| {
                let t = i as f32 / n as f32 * std::f32::consts::TAU;
                let (a, b) = (PROTON_RADIUS * t.cos(), PROTON_RADIUS * t.sin());
                match plane {
                    0 => [a, b, 0.0],
                    1 => [a, 0.0, b],
                    _ => [0.0, a, b],
                }
            };
            for i in 0..n {
                marks.push(line(at(i), at(i + 1), w, [PROTON[0], PROTON[1], PROTON[2], 150]));
            }
        }
        let shell = vec![sphere([0.0; 3], PROTON_RADIUS, [PROTON[0], PROTON[1], PROTON[2], 54])];
        vec![thin(tile(1, 0, marks)), tile(2, 1, shell)]
    }
}
