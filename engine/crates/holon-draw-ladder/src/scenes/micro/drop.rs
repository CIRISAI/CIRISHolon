//! A drop of water on the paving, then its surface.
use super::*;
use holon_draw::ladder::Status;

/// Micrometres.
pub const UNIT: i32 = -6;

pub struct Drop {
    tiles: Vec<Tile>,
}

/// How far the drop's surface lies below its top at a distance `rho` from the axis (µm).
fn sag(rho: f64) -> f64 {
    let r = DROP_RADIUS_M * 1e6;
    r - (r * r - rho * rho).max(0.0).sqrt()
}

impl Drop {
    /// The top of the drop when the view is 10^`zv` metres high: the radius (µm) and alpha of
    /// one flat disc on the tangent plane, or none. A sphere's mesh is flat-sided, so close
    /// to its top the surface is drawn as the plane it is there, out to the drop's own
    /// horizon from a view's height above it: sqrt(R x view). Wider views show the sphere.
    pub fn top(zv: f64) -> Option<(f32, f64)> {
        let view = 10f64.powf(zv + 6.0);
        let a = ramp(view.log10(), 150f64.log10(), 60f64.log10());
        (a > 0.0).then(|| ((DROP_RADIUS_M * 1e6 * view).sqrt().min(300.0) as f32, a))
    }

    pub fn new() -> Drop {
        let mut r = Rng::new(0x64_726f_70);
        let rd = DROP_RADIUS_M * 1e6;
        let floor = -(DROP_APEX_M * 1e6) as f32;
        let mut base = vec![quad([0.0, 0.0, floor], 2.0e5, 2.0e5, SLAB)];
        for g in grit() {
            let c = if g[3] < 0.5 { shade(SLAB, 0.55 + g[3] as f32 * 0.6) } else { shade(SLAB, 0.7 + g[3] as f32 * 0.55) };
            base.push(disc([(g[0] * 1e6) as f32, (g[1] * 1e6) as f32, floor + 20.0], (g[2] * 1e6) as f32, c));
        }
        base.push(sphere([0.0, 0.0, -rd as f32], rd as f32, WATER));
        // Specks afloat on the surface, at every distance from 0.4 µm to 0.5 mm from the
        // point the zoom goes into, each about a twentieth of its distance across, so that there
        // is a thing of known size in view at every zoom: from a pollen grain to soot.
        let mut specks = Vec::new();
        let colours = [[196u8, 170, 96, 255], [88, 80, 70, 255], [150, 132, 110, 255], [60, 58, 56, 255], [176, 150, 120, 255]];
        let mut d = 0.4f64;
        while d < 500.0 {
            for _ in 0..3 {
                let rho = d * (1.0 + 0.6 * r.f() as f64);
                let a = r.f() as f64 * std::f64::consts::TAU;
                let size = (rho * (0.03 + 0.05 * r.f() as f64)).min(12.0);
                let c = colours[(r.u64() % colours.len() as u64) as usize];
                specks.push(sphere([(rho * a.cos()) as f32, (rho * a.sin()) as f32, (-sag(rho) + 0.4 * size) as f32], size as f32, c));
            }
            d *= 1.6;
        }
        Drop { tiles: vec![tile(1, 0, base), tile(3, 2, specks)] }
    }
}

impl Default for Drop {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for Drop {
    fn name(&self) -> &'static str {
        "a drop of water"
    }
    fn unit(&self) -> i32 {
        UNIT
    }
    fn status(&self) -> Status {
        Status::Illustration
    }
    fn source(&self) -> &'static str {
        "A picture at true size: a dew drop 2 mm across on the paving, then its surface, which is smooth at every scale down to the molecules. The specks afloat on it (25 nm to 24 µm across, soot to a pollen grain) are drawn from a seed so that something of known size is always in view. Nothing is measured or computed."
    }
    fn span(&self) -> (f64, f64) {
        (-7.5 - VIEW_DECADES, -2.0 - VIEW_DECADES)
    }
    fn tiles(&mut self, z: f64, _clock: f64, _alpha: f64) -> Vec<Tile> {
        let mut out = self.tiles.clone();
        if let Some((radius, a)) = Self::top(z + VIEW_DECADES) {
            out.insert(1, faded(tile(2, 1, vec![disc([0.0; 3], radius, shade(WATER, 1.05))]), a));
        }
        out
    }
}
