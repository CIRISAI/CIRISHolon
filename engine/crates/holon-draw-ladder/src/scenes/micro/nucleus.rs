//! Inside a hydrogen atom: the electron's cloud thinning out, then the nucleus, one proton.
use super::*;
use holon_draw::ladder::Status;

/// Femtometres.
pub const UNIT: i32 = -15;
/// The Bohr radius, in fm (CODATA 2018: 0.529 177 210 9 Å).
pub const BOHR_FM: f64 = 52_917.721_09;
/// The proton's charge radius, in fm (CODATA 2018: 0.8414).
pub const PROTON_RADIUS_FM: f32 = 0.8414;
const DOTS: usize = 6000;
pub(crate) const PROTON: [u8; 4] = [214, 84, 60, 255];

pub struct Nucleus {
    /// The cloud's dots, nearest the nucleus first: position (fm) and distance.
    dots: Vec<([f32; 3], f32)>,
}

impl Nucleus {
    pub fn new() -> Nucleus {
        let mut r = Rng::new(0x31_73);
        // The 1s density is exp(-2r/a0) / (pi a0^3): the distance from the nucleus has the
        // gamma density r^2 exp(-2r/a0), which is the sum of three exponentials of mean a0/2.
        let mut dots: Vec<([f32; 3], f32)> = (0..DOTS)
            .map(|_| {
                let u = (1.0 - r.f() as f64) * (1.0 - r.f() as f64) * (1.0 - r.f() as f64);
                let d = (-0.5 * BOHR_FM * u.ln()) as f32;
                let n = r.dir();
                ([n[0] * d, n[1] * d, n[2] * d], d)
            })
            .collect();
        dots.sort_by(|a, b| a.1.total_cmp(&b.1));
        Nucleus { dots }
    }
    /// The mean distance of the dots from the nucleus, in fm (the 1s state's is 1.5 a0).
    pub fn mean_distance(&self) -> f64 {
        self.dots.iter().map(|d| d.1 as f64).sum::<f64>() / self.dots.len() as f64
    }
}

impl Default for Nucleus {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for Nucleus {
    fn name(&self) -> &'static str {
        "inside the atom: the nucleus"
    }
    fn unit(&self) -> i32 {
        UNIT
    }
    fn status(&self) -> Status {
        Status::Declared
    }
    fn source(&self) -> &'static str {
        "Hydrogen. The nucleus is a DECLARED input of this repository (TIERS.md, the nucleus row: Z, mass, spin, charge radius; nothing here computes it): one proton, drawn at its charge radius, 0.84 fm (CODATA). The cloud is 6000 dots drawn from a seed with the textbook 1s density exp(-2r/a0)/(pi a0^3), a0 = 52.9 pm; it thins to nothing because the atom is that empty. The dots' size and the ring that marks where the proton is are not physical."
    }
    fn span(&self) -> (f64, f64) {
        (-14.5 - VIEW_DECADES, -10.3 - VIEW_DECADES)
    }
    fn tiles(&mut self, z: f64, _clock: f64, _alpha: f64) -> Vec<Tile> {
        let z = z + VIEW_DECADES; // the view's height, as the rest of this function has it
        let view = 10f64.powf(z - UNIT as f64) as f32;
        let mut solid = vec![sphere([0.0; 3], PROTON_RADIUS_FM, PROTON)];
        // A ring about the proton while the proton itself is under a few pixels across.
        let ring = 0.035 * view;
        let a = 1.0 - (3.0 * PROTON_RADIUS_FM / ring) as f64;
        if a > 0.0 {
            let c = alpha([237, 233, 223, 255], a);
            let n = 40;
            let at = |i: usize| {
                let t = i as f32 / n as f32 * std::f32::consts::TAU;
                [ring * t.cos(), ring * t.sin(), 0.0]
            };
            for i in 0..n {
                solid.push(line(at(i), at(i + 1), 0.0016 * view, c));
            }
        }
        // The dots within the view's far plane (24 view heights): each a small box, sized so
        // that it stays a few pixels across wherever it is.
        let reach = 26.0 * view;
        let n = self.dots.partition_point(|d| d.1 <= reach);
        let mut cloud = Vec::with_capacity(n);
        for (p, d) in &self.dots[..n] {
            let s = 0.0055 * (view + 0.6 * d);
            cloud.push(block([p[0], p[1], p[2] - 0.5 * s], [s, s, s], [170, 205, 255, 230]));
        }
        vec![thin(tile(1, 0, solid)), tile(2, 1, cloud)]
    }
}
