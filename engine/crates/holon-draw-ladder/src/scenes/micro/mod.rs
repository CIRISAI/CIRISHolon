//! The rungs finer than the street: from the ground underfoot to the vacuum.
//!
//! Seven kinds of scene, eleven rungs (the vacuum is one picture laid over five rungs, each
//! with its own unit, so that no mark's coordinate leaves the range a 32-bit float and a
//! shader's guards are happy with):
//!
//! | view height, log10 m (z + 3) | scene | unit | status |
//! |---|---|---|---|
//! | -2 .. 2        | the ground       | 1 cm   | illustration |
//! | -7.5 .. -2     | a drop of water  | 1 µm   | illustration |
//! | -9 .. -7.5     | liquid water     | 1 Å    | illustration |
//! | -10.3 .. -9    | hydrogen, live   | 1 Å    | simulated    |
//! | -14.5 .. -10.3 | atom to nucleus  | 1 fm   | declared     |
//! | -16 .. -14.5   | the proton       | 0.1 fm | illustration |
//! | -35 .. -16     | the vacuum (x5)  | varies | illustration |
//!
//! Every rung is centred on ONE point going in: the top of a dew drop on the paving, then a
//! hydrogen atom of a water molecule at the drop's surface, then that atom's nucleus, then a
//! point inside the proton. The hydrogen rung is the exception it has to be: this
//! repository's live simulation is of H2, not of water, and the rung says so.
//!
//! Nothing here is random at run time: every position comes from a seed, every motion from
//! the host's clock.
use crate::Scene;
use holon_draw::{Instance, Kind, Tile, PARAMS_NEUTRAL};

mod atoms;
mod drop;
mod ground;
mod hadron;
mod nucleus;
mod vacuum;
mod water;

pub use atoms::Atoms;
pub use drop::Drop;
pub use ground::Ground;
pub use hadron::Hadron;
pub use nucleus::Nucleus;
pub use vacuum::Vacuum;
pub use water::Water;

/// Every inward scene.
pub fn inward() -> Vec<Box<dyn Scene>> {
    let mut s: Vec<Box<dyn Scene>> = vec![Box::new(Ground::new()), Box::new(Drop::new()), Box::new(Water::new()), Box::new(Atoms::new()), Box::new(Nucleus::new()), Box::new(Hadron::new())];
    for v in Vacuum::rungs() {
        s.push(Box::new(v));
    }
    s
}

/// A scene is handed `z` as log10 of the metres one device pixel spans. The scenes here are
/// laid out by the height of the view, taking a view a thousand pixels high: a scene's own
/// zoom is `z + VIEW_DECADES`, and a feature `s` metres across is `s / 10^z` pixels.
pub(crate) const VIEW_DECADES: f64 = 3.0;

// ---- what the ground and the drop share, in metres, so the two rungs show one thing ----

/// The dew drop's radius.
pub(crate) const DROP_RADIUS_M: f64 = 1.0e-3;
/// The height of the drop's top above the paving it sits on (a cap of a 1 mm sphere).
pub(crate) const DROP_APEX_M: f64 = 1.6e-3;
/// The paving slab's colour under the drop.
pub(crate) const SLAB: [u8; 4] = [150, 148, 142, 255];
pub(crate) const WATER: [u8; 4] = [128, 178, 214, 255];

/// Grit on the slab about the drop: x, y, radius (metres) and a shade (0..1).
pub(crate) fn grit() -> Vec<[f64; 4]> {
    let mut r = Rng::new(0x6772_6974);
    (0..520)
        .map(|_| {
            // log-uniform in distance from 0.3 mm to 9 cm
            let d = 3.0e-4 * (300.0f64).powf(r.f() as f64);
            let a = r.f() as f64 * std::f64::consts::TAU;
            let size = (d * (0.008 + 0.022 * r.f() as f64)).clamp(2.0e-5, 8.0e-4);
            [d * a.cos(), d * a.sin(), size, r.f() as f64]
        })
        .collect()
}

// ---- seeds ----

/// splitmix64: a seeded stream, the same on every machine.
pub(crate) struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }
    pub fn u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        mix(self.0)
    }
    /// In [0, 1).
    pub fn f(&mut self) -> f32 {
        (self.u64() >> 40) as f32 / (1u64 << 24) as f32
    }
    /// In [-1, 1).
    pub fn s(&mut self) -> f32 {
        2.0 * self.f() - 1.0
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
    /// A direction, uniform on the sphere.
    pub fn dir(&mut self) -> [f32; 3] {
        let z = self.s();
        let p = self.f() * std::f32::consts::TAU;
        let s = (1.0 - z * z).max(0.0).sqrt();
        [s * p.cos(), s * p.sin(), z]
    }
}

pub(crate) fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// A hash of four integers, for a value that belongs to a place rather than to a stream.
pub(crate) fn hash(a: i64, b: i64, c: i64, d: i64) -> u64 {
    mix(mix(mix(mix(a as u64 ^ 0x51_7c_c1_b7_27_22_0a_95).wrapping_add(b as u64)).wrapping_add(c as u64)).wrapping_add(d as u64))
}

/// A hash as a number in [0, 1).
pub(crate) fn unit(h: u64) -> f64 {
    (h >> 11) as f64 / (1u64 << 53) as f64
}

// ---- marks ----

pub(crate) fn sphere(p: [f32; 3], r: f32, c: [u8; 4]) -> Instance {
    Instance::new(Kind::Sphere, p, [r, 0.0, 0.0]).colour(c)
}
pub(crate) fn rod(a: [f32; 3], b: [f32; 3], r: f32, c: [u8; 4]) -> Instance {
    let mut m = Instance::new(Kind::Rod, a, [r, 0.0, 0.0]).colour(c);
    m.aux = b;
    m
}
pub(crate) fn line(a: [f32; 3], b: [f32; 3], w: f32, c: [u8; 4]) -> Instance {
    let mut m = Instance::new(Kind::Line, a, [w, 0.0, 0.0]).colour(c);
    m.aux = b;
    m
}
pub(crate) fn disc(p: [f32; 3], r: f32, c: [u8; 4]) -> Instance {
    Instance::new(Kind::Disc, p, [r, 0.0, 0.0]).colour(c)
}
/// A flat rectangle centred at `p`, `w` by `h`.
pub(crate) fn quad(p: [f32; 3], w: f32, h: f32, c: [u8; 4]) -> Instance {
    Instance::new(Kind::Quad, p, [w, h, 0.0]).colour(c)
}
/// A box standing on `p` (the centre of its base).
pub(crate) fn block(p: [f32; 3], size: [f32; 3], c: [u8; 4]) -> Instance {
    Instance::new(Kind::Box, p, size).colour(c)
}
pub(crate) fn tile(id: u64, layer: u8, marks: Vec<Instance>) -> Tile {
    Tile { id, origin: [0.0; 3], version: 1, layer, screen: false, params: PARAMS_NEUTRAL, marks }
}
/// A tile's own alpha (the ladder multiplies its cross-fade weight into the same number).
pub(crate) fn faded(mut t: Tile, alpha: f64) -> Tile {
    t.params[holon_draw::PARAM_DIM + 3] = alpha.clamp(0.0, 1.0) as f32;
    t
}
/// Lines of this tile are never drawn under 1.2 pixels wide.
pub(crate) fn thin(mut t: Tile) -> Tile {
    t.params[holon_draw::PARAM_FLOOR] = 0.6;
    t
}
/// 0 at `a`, 1 at `b`, straight between (either order).
pub(crate) fn ramp(x: f64, a: f64, b: f64) -> f64 {
    ((x - a) / (b - a)).clamp(0.0, 1.0)
}
pub(crate) fn shade(c: [u8; 4], k: f32) -> [u8; 4] {
    let f = |v: u8| (v as f32 * k).clamp(0.0, 255.0) as u8;
    [f(c[0]), f(c[1]), f(c[2]), c[3]]
}
pub(crate) fn alpha(c: [u8; 4], a: f64) -> [u8; 4] {
    [c[0], c[1], c[2], (c[3] as f64 * a.clamp(0.0, 1.0)) as u8]
}

#[cfg(test)]
mod tests;
