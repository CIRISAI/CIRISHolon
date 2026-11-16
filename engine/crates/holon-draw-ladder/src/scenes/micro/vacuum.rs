//! Below the proton, down to the Planck length: a picture of a fluctuating field.
//!
//! One picture over five rungs. It is defined in METRES (a mesh for every half-decade of
//! scale, numbered by `k` with spacing 10^(k/2) m), so two neighbouring rungs draw the same
//! thing in their two units and the cross-fade between them is of a picture with itself.
use super::*;
use holon_draw::ladder::Status;

/// The Planck length in metres (CODATA 2018).
pub const PLANCK_M: f64 = 1.616_255e-35;
/// Half the mesh's width, in cells.
const HALF: i32 = 10;
const PAIRS: i64 = 10;
const COLOURS: [[u8; 3]; 4] = [[94, 200, 190], [150, 130, 230], [233, 196, 106], [220, 110, 140]];

pub struct Vacuum {
    unit: i32,
    span: (f64, f64),
}

impl Vacuum {
    /// The five rungs, from the proton down to the Planck length.
    pub fn rungs() -> Vec<Vacuum> {
        [(-19.5, -16.0, -18), (-23.0, -19.5, -21), (-27.0, -23.0, -25), (-31.0, -27.0, -29), (-35.0, -31.0, -33)].into_iter().map(|(lo, hi, unit): (f64, f64, i32)| Vacuum { unit, span: (lo - VIEW_DECADES, hi - VIEW_DECADES) }).collect()
    }
    /// The Planck length in this rung's unit.
    pub fn planck(&self) -> f64 {
        PLANCK_M / 10f64.powi(self.unit)
    }
    /// The meshes drawn when the view is 10^`z` metres high, as (k, weight): the mesh with 4 to 12.6 cells across the
    /// view, the finer one fading out and the coarser one fading in. None finer than the
    /// Planck length.
    pub fn meshes(z: f64) -> Vec<(i64, f64)> {
        let e = 2.0 * (z - 4f64.log10());
        let k0 = e.floor();
        let frac = e - k0;
        let k0 = k0 as i64;
        [(k0 - 1, 1.0 - frac), (k0, 1.0), (k0 + 1, frac)].into_iter().filter(|(k, w)| *w > 0.0 && 10f64.powf(*k as f64 / 2.0) >= PLANCK_M).collect()
    }
}

impl Scene for Vacuum {
    fn name(&self) -> &'static str {
        "the vacuum"
    }
    fn unit(&self) -> i32 {
        self.unit
    }
    fn status(&self) -> Status {
        Status::Illustration
    }
    fn source(&self) -> &'static str {
        "A picture only. Nothing in this repository computes physics at these scales, and below about 1e-19 m no experiment has resolved anything. Drawn: a mesh for every half-decade of scale whose nodes ride seeded sines of the clock, and pairs of dots that appear and vanish, to stand for a field that is never still. The meshes stop at the Planck length, 1.616e-35 m (CODATA), the white bar, below which no tested theory describes space."
    }
    fn span(&self) -> (f64, f64) {
        self.span
    }
    fn tiles(&mut self, z: f64, clock: f64, _alpha: f64) -> Vec<Tile> {
        let z = z + VIEW_DECADES; // the view's height
        let to_unit = |k: i64| 10f64.powf(k as f64 / 2.0 - self.unit as f64);
        let view = 10f64.powf(z - self.unit as f64);
        let mut lines = Vec::with_capacity(2600);
        let mut dots = Vec::with_capacity(64);
        let side = (2 * HALF + 1) as usize;
        let mut height = vec![0.0f32; side * side];
        for (k, weight) in Self::meshes(z) {
            let g = to_unit(k);
            let rgb = COLOURS[k.rem_euclid(4) as usize];
            // each node rides two sines whose rates and phases belong to the node
            for j in -HALF..=HALF {
                for i in -HALF..=HALF {
                    let h = hash(k, i as i64, j as i64, 1);
                    let (p1, p2, w1) = (unit(h) * std::f64::consts::TAU, unit(mix(h)) * std::f64::consts::TAU, 0.6 + 1.5 * unit(mix(h ^ 0x5bd1)));
                    let y = 0.6 * (w1 * clock + p1).sin() + 0.4 * (1.9 * w1 * clock + p2).sin();
                    height[((j + HALF) as usize) * side + (i + HALF) as usize] = (0.3 * g * y) as f32;
                }
            }
            let node = |i: i32, j: i32| [(i as f64 * g) as f32, (j as f64 * g) as f32, height[((j + HALF) as usize) * side + (i + HALF) as usize]];
            // fainter toward the mesh's edge, so that it has none
            let edge = |i: f32, j: f32| (1.0 - (i * i + j * j) / (HALF * HALF) as f32).clamp(0.0, 1.0) as f64;
            let w = (0.0016 * view) as f32;
            for j in -HALF..=HALF {
                for i in -HALF..HALF {
                    let a = weight * edge(i as f32 + 0.5, j as f32);
                    if a > 0.02 {
                        lines.push(line(node(i, j), node(i + 1, j), w, [rgb[0], rgb[1], rgb[2], (a * 235.0) as u8]));
                        lines.push(line(node(j, i), node(j, i + 1), w, [rgb[0], rgb[1], rgb[2], (a * 235.0) as u8]));
                    }
                }
            }
            // pairs: each lives for one cycle of its own period, somewhere new every cycle
            for m in 0..PAIRS {
                let h = hash(k, m, 0, 2);
                let period = 2.5 + 3.0 * unit(h);
                let at = clock / period + unit(mix(h));
                let cycle = at.floor();
                let life = (std::f64::consts::PI * (at - cycle)).sin();
                if life < 0.06 {
                    continue;
                }
                let s = hash(k, m, cycle as i64, 3);
                let mut r = Rng::new(s);
                let c = [r.s() as f64 * 6.5 * g, r.s() as f64 * 6.5 * g, r.s() as f64 * 2.2 * g];
                let d = r.dir();
                let half = 0.45 * g * life;
                let (p, q) = ([(c[0] + half * d[0] as f64) as f32, (c[1] + half * d[1] as f64) as f32, (c[2] + half * d[2] as f64) as f32], [(c[0] - half * d[0] as f64) as f32, (c[1] - half * d[1] as f64) as f32, (c[2] - half * d[2] as f64) as f32]);
                let a = (weight * 255.0) as u8;
                lines.push(line(p, q, (0.03 * g * life).max(0.0008 * view) as f32, [237, 233, 223, (weight * 150.0) as u8]));
                dots.push(sphere(p, (0.11 * g * life) as f32, [255, 150, 90, a]));
                dots.push(sphere(q, (0.11 * g * life) as f32, [110, 170, 255, a]));
            }
        }
        // the Planck length: a white bar with an end stop at each end
        let lp = self.planck();
        if lp / view > 0.004 && lp / view < 60.0 {
            let (h, w, t) = ((0.5 * lp) as f32, (0.004 * view) as f32, (0.03 * view) as f32);
            let c = [250, 250, 246, 255];
            lines.push(line([-h, 0.0, 0.0], [h, 0.0, 0.0], w, c));
            lines.push(line([-h, 0.0, -t], [-h, 0.0, t], w, c));
            lines.push(line([h, 0.0, -t], [h, 0.0, t], w, c));
        }
        vec![thin(tile(1, 0, lines)), tile(2, 1, dots)]
    }
}
