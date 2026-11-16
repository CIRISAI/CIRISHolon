//! The ground underfoot: a paving edge, a kerb, soil and grass, at true size.
use super::*;
use holon_draw::ladder::Status;

/// Centimetres.
pub const UNIT: i32 = -2;
/// A paving slab's side and the joint between two, in cm.
pub const SLAB_PITCH: f32 = 40.0;
pub const JOINT: f32 = 0.8;
/// The paving ends this far (cm) from the point the zoom goes into.
const EDGE: f32 = 14.0;

pub struct Ground {
    far: Tile,
    near: Tile,
    fine: Tile,
}

impl Ground {
    pub fn new() -> Ground {
        let mut r = Rng::new(0x67_726f_756e_64);
        // the paving's top: the drop stands on it and its top is the origin
        let z0 = -(DROP_APEX_M * 100.0) as f32;
        let far_w = 100_000.0f32; // 1 km of each surface, to the horizon of any view the rung has
        let mut far = Vec::new();
        // the paved side (x below EDGE): a far sheet, the dark joints under the slabs, the slabs
        far.push(quad([EDGE - 0.5 * far_w, 0.0, z0 - 2.0], far_w, 2.0 * far_w, [141, 139, 133, 255]));
        let (cols, rows) = (14i32, 32i32);
        far.push(quad([EDGE - 0.5 * cols as f32 * SLAB_PITCH, 5.0, z0 - 1.0], cols as f32 * SLAB_PITCH, rows as f32 * SLAB_PITCH, [72, 70, 66, 255]));
        for i in 0..cols {
            for j in -rows / 2..rows / 2 {
                let k = if i == 0 && j == 0 { 1.0 } else { 0.93 + 0.12 * r.f() };
                far.push(block([EDGE - (i as f32 + 0.5) * SLAB_PITCH, j as f32 * SLAB_PITCH + 5.0, z0 - 4.0], [SLAB_PITCH - JOINT, SLAB_PITCH - JOINT, 4.0], shade(SLAB, k)));
            }
        }
        // the kerb: a metre at a time near, one long stone beyond
        far.push(block([EDGE + 5.0, 0.0, z0 - 12.0], [10.0, 2.0 * far_w, 13.8], [168, 164, 156, 255]));
        for j in -8..8 {
            far.push(block([EDGE + 5.0, j as f32 * 100.0 + 50.0, z0 - 12.0], [10.0, 99.2, 14.0], shade([178, 174, 166, 255], 0.95 + 0.08 * r.f())));
        }
        // soil, then lawn
        far.push(quad([EDGE + 10.0 + 0.5 * far_w, 0.0, z0 - 1.6], far_w, 2.0 * far_w, [78, 60, 42, 255]));
        far.push(quad([EDGE + 28.0 + 0.5 * far_w, 0.0, z0 - 0.8], far_w, 2.0 * far_w, [62, 96, 46, 255]));

        // soil crumbs and grass blades
        let mut near = Vec::new();
        for _ in 0..300 {
            let s = r.range(0.12, 0.7);
            let k = r.range(0.75, 1.25);
            near.push(sphere([EDGE + 10.0 + r.range(0.5, 19.0), r.range(-260.0, 260.0), z0 - 1.6 + 0.5 * s], s, shade([92, 70, 48, 255], k)));
        }
        for n in 0..3600 {
            // thin among the soil at the lawn's edge, thick further in
            let x = if n % 6 == 0 { EDGE + r.range(14.0, 30.0) } else { EDGE + r.range(27.0, 300.0) };
            let y = r.range(-300.0, 300.0);
            let h = r.range(4.0, 10.0);
            let lean = [r.s() * 0.35 * h, r.s() * 0.35 * h];
            let base = [x, y, z0 - 1.2];
            let mid = [x + 0.35 * lean[0], y + 0.35 * lean[1], z0 - 1.2 + 0.6 * h];
            let tip = [x + lean[0], y + lean[1], z0 - 1.2 + h];
            let c = shade([74, 128, 52, 255], r.range(0.75, 1.3));
            near.push(line(base, mid, 0.36, shade(c, 0.85)));
            near.push(line(mid, tip, 0.2, c));
        }

        // grit on the slab, and the drop
        let mut fine = Vec::new();
        for g in grit() {
            let c = if g[3] < 0.5 { shade(SLAB, 0.55 + g[3] as f32 * 0.6) } else { shade(SLAB, 0.7 + g[3] as f32 * 0.55) };
            fine.push(disc([(g[0] * 100.0) as f32, (g[1] * 100.0) as f32, z0 + 0.002], (g[2] * 100.0) as f32, c));
        }
        let rd = (DROP_RADIUS_M * 100.0) as f32;
        fine.push(sphere([0.0, 0.0, -rd], rd, WATER));
        Ground { far: tile(1, 0, far), near: tile(2, 1, near), fine: tile(3, 2, fine) }
    }
}

impl Default for Ground {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for Ground {
    fn name(&self) -> &'static str {
        "the ground underfoot"
    }
    fn unit(&self) -> i32 {
        UNIT
    }
    fn status(&self) -> Status {
        Status::Illustration
    }
    fn source(&self) -> &'static str {
        "A picture at true size, of no particular place: 40 cm paving slabs with 8 mm joints, a 10 cm kerb, soil crumbs of 1 to 7 mm, grass blades 4 to 10 cm tall, grit, and a dew drop 2 mm across on the slab. Every position is drawn from a seed; nothing is measured or computed."
    }
    fn span(&self) -> (f64, f64) {
        (-2.0 - VIEW_DECADES, 2.0 - VIEW_DECADES)
    }
    fn tiles(&mut self, z: f64, _clock: f64, _alpha: f64) -> Vec<Tile> {
        let z = z + VIEW_DECADES; // the view's height, as the rest of this function has it
        let mut out = vec![self.far.clone()];
        // blades are under a pixel when the view is tens of metres high; grit under a metre
        let near = ramp(z, 1.5, 1.2);
        if near > 0.0 {
            out.push(faded(self.near.clone(), near));
        }
        let fine = ramp(z, 0.3, 0.0);
        if fine > 0.0 {
            out.push(faded(self.fine.clone(), fine));
        }
        out
    }
}
