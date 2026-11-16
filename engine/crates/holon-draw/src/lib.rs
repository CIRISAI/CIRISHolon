//! `holon-draw`: a generic draw core (`INTERFACE.md` §1, profile A).
//!
//! # What it is
//!
//! A scene is a set of TILES. A tile has a local origin, a version and a list of MARKS. A
//! mark is one fixed-size INSTANCE record (52 bytes) with a tag, a colour, a detail level
//! and a pin. The core does four things and draws nothing itself:
//!
//! * [`Scene::frame`] decides what is drawn fine and what coarse through an [`Allocator`],
//!   and PACKS the chosen instances into one byte buffer per tile and mark kind, ready for
//!   one upload and one instanced draw call each;
//! * [`Frame::pick`] answers which tagged mark is under a point: pure, exactly one or none;
//! * [`Frame::report`] says, for each tagged mark, whether it is drawn, where, and whether
//!   it is legible;
//! * [`Frame::digest`] folds the packed bytes, so two renderers can be compared before a
//!   pixel is.
//!
//! # What it refuses
//!
//! * **Dropping a tagged mark.** An INSTANCE is every mark that carries one tag, in any
//!   tile; a pick and a report name the instance (the screen standard's L1 is one tag per
//!   instance, not per mark). A tagged mark is drawn at every allocation: detail may
//!   coarsen it (its `coarse` flag is set in the packed record) and never removes it. Only
//!   decoration (tag 0) is dropped when a tile is coarse.
//! * **A pick through a modal layer.** While [`Scene::modal`] names a layer, a pick sees
//!   that layer alone (an open sheet takes every pick).
//! * **A non-finite field** ([`Refusal::NotFinite`]): a mark whose number is not a number
//!   is not drawn as zero (L4).
//!
//! # What it does not do
//!
//! It holds no strings and shapes no text, has no clock and no randomness, and knows no
//! state of a read: a host gives it a scene only when there is one to draw. Level of detail
//! changes how finely a tile is drawn, never what a tagged mark means (L8): a tagged mark
//! is drawn at every allocation, and its report carries a digest of exactly the bytes shown.
//!
//! CORRECTION 2026-11-15 (the constitution session's review of `INTERFACE.md`): the first
//! form of this file refused two marks with one tag and let a coarse tile drop tagged marks.
//! Both were wrong against the standard and are replaced as above.
//!
//! The pick in this slice treats each mark as its bounding sphere projected to the screen.
//! That is exact for discs and spheres and an over-estimate for the rest; a per-kind test is
//! owed and the over-estimate is why the nearest centre wins a tie.

pub mod abi;
pub mod alloc;
pub mod ladder;
pub mod twin;
pub mod ui;

pub use alloc::{Allocator, Bits, Candidate, Ring};

/// The kinds of mark, in the order of `INTERFACE.md` §1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Kind {
    Quad = 1,
    Line = 2,
    Arc = 3,
    Disc = 4,
    Box = 5,
    Prism = 6,
    Sphere = 7,
    Rod = 8,
    Patch = 9,
    Glyph = 10,
    /// A run of text already shaped and rasterised by the host's text provider into an atlas:
    /// a rectangle `size[0]` by `size[1]` centred at `pos`, sampling the atlas rectangle
    /// `aux[0], aux[1]` (u0, v0) to `aux[2], angle` (u1, v1) on atlas page `variant`, tinted
    /// by `rgba`. The core holds no string and no font (`INTERFACE.md` §1, D11).
    Run = 11,
}

pub const KINDS: [Kind; 11] = [Kind::Quad, Kind::Line, Kind::Arc, Kind::Disc, Kind::Box, Kind::Prism, Kind::Sphere, Kind::Rod, Kind::Patch, Kind::Glyph, Kind::Run];

/// The bytes of one packed instance.
pub const INSTANCE_BYTES: usize = 52;

/// One mark. Positions are in the tile's local frame, in the tile's units.
///
/// `size` is the mark's extent (width, depth, height; or radius in `size[0]`). `aux` is the
/// second point of a line, rod or arc, or a control point. `tag` 0 is DECORATION: it is
/// never picked and never reported.
///
/// `tag` is a HANDLE, not a name. A consumer whose marks have derived string tags (the
/// screen standard's `prefix_surface_instance`) keeps the table from handle to tag and is
/// the only one who makes a tag; this crate never mints one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instance {
    pub kind: Kind,
    pub pos: [f32; 3],
    pub size: [f32; 3],
    pub aux: [f32; 3],
    pub angle: f32,
    pub rgba: [u8; 4],
    pub tag: u32,
    /// 0: the same at every allocation. 1 and above: decoration is drawn only when its tile
    /// is fine; a tagged mark is drawn coarse (flagged) when its tile is not.
    pub detail: u8,
    /// Never coarsened.
    pub pin: bool,
    /// A kind's own small choice: a prism's top, a glyph's shape, a patch's field.
    pub variant: u8,
}

impl Instance {
    pub fn new(kind: Kind, pos: [f32; 3], size: [f32; 3]) -> Instance {
        Instance { kind, pos, size, aux: [0.0; 3], angle: 0.0, rgba: [255; 4], tag: 0, detail: 0, pin: false, variant: 0 }
    }
    pub fn tagged(mut self, tag: u32) -> Instance {
        self.tag = tag;
        self
    }
    pub fn detail(mut self, d: u8) -> Instance {
        self.detail = d;
        self
    }
    pub fn pinned(mut self) -> Instance {
        self.pin = true;
        self
    }
    pub fn colour(mut self, rgba: [u8; 4]) -> Instance {
        self.rgba = rgba;
        self
    }
    fn finite(&self) -> bool {
        self.pos.iter().chain(&self.size).chain(&self.aux).chain(std::iter::once(&self.angle)).all(|v| v.is_finite())
    }
    /// The radius of a sphere about `pos` that holds the mark.
    pub fn bound(&self) -> f32 {
        let s = self.size;
        let own = match self.kind {
            Kind::Disc | Kind::Sphere | Kind::Glyph => s[0].abs(),
            _ => (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt() * 0.5,
        };
        let reach = match self.kind {
            Kind::Line | Kind::Rod | Kind::Arc => {
                let d = [self.aux[0] - self.pos[0], self.aux[1] - self.pos[1], self.aux[2] - self.pos[2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
            }
            _ => 0.0,
        };
        own.max(reach)
    }
    /// 52 little-endian bytes: pos, size, aux (9 f32), angle (f32), rgba, tag (u32), then
    /// kind, detail, pin, variant.
    pub fn pack(&self, out: &mut Vec<u8>) {
        self.pack_as(out, false)
    }
    /// As [`Instance::pack`]; `coarse` sets bit 1 of the flags byte (bit 0 is the pin).
    pub fn pack_as(&self, out: &mut Vec<u8>, coarse: bool) {
        for v in self.pos.iter().chain(&self.size).chain(&self.aux).chain(std::iter::once(&self.angle)) {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&self.rgba);
        out.extend_from_slice(&self.tag.to_le_bytes());
        out.extend_from_slice(&[self.kind as u8, self.detail, self.pin as u8 | (coarse as u8) << 1, self.variant]);
    }
}

/// A tile: a local origin in the world (64-bit, never sent to a GPU), a version of the
/// inputs it was built from, and its marks.
#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub id: u64,
    pub origin: [f64; 3],
    /// The version of the inputs this tile was built from. A host that drops an answer bumps
    /// the version it asks for, and a tile of another version is not drawn (L11).
    pub version: u64,
    /// Draw order and pick order: a higher layer is over a lower one.
    pub layer: u8,
    /// A SCREEN tile's marks are in device pixels from the viewport's top left (`origin` is
    /// added), drawn after every world tile with no depth test, in order. Panels, text and
    /// buttons are screen tiles. A world tile's marks are in world units.
    pub screen: bool,
    /// The tile's SETTINGS: 8 groups of 4 numbers the backend hands to the shader for every
    /// batch of this tile (`PARAM_*` names the groups). A mark whose colour mode is not
    /// [`COLOUR_LITERAL`] takes its colour from them, so a host can recolour a whole tile
    /// (night, terrain, a roof factor) by changing 32 numbers and no instance.
    pub params: [f32; 32],
    pub marks: Vec<Instance>,
}

/// The settings groups of a tile, by index of their first number.
pub const PARAM_DIM: usize = 0; // rgb multiplier on every colour of the tile (night); w multiplies alpha (a cross-fade)
pub const PARAM_RAMP0: usize = 4; // the colour at t = 0 of COLOUR_RAMP
pub const PARAM_RAMP1: usize = 8; // the colour at t = 1 of COLOUR_RAMP
pub const PARAM_TINT: usize = 12; // rgb multiplier for COLOUR_TINT (roofs)
pub const PARAM_NOISE: usize = 16; // the procedural field's origin and scale: x0, y0, sx, sy
pub const PARAM_TEX: usize = 20; // x: the field's strength (0 off); yzw: the host's
pub const PARAM_FREE: usize = 24; // 24..32: the host's own

/// The tile settings that leave colours exactly as their records give them.
pub const PARAMS_NEUTRAL: [f32; 32] = {
    let mut p = [0.0f32; 32];
    p[0] = 1.0;
    p[1] = 1.0;
    p[2] = 1.0;
    p[3] = 1.0;
    p[12] = 1.0;
    p[13] = 1.0;
    p[14] = 1.0;
    p
};

/// A mark's colour mode, in the top two bits of its `variant` byte for every kind but
/// `Run` (whose `variant` is its atlas page). LITERAL: `rgba`. RAMP: the mix of the tile's
/// RAMP0 and RAMP1 at `t = aux[0]`, alpha from `rgba`. TINT: `rgba.rgb` times the tile's TINT.
/// In every mode the result is then multiplied by the tile's DIM.
pub const COLOUR_LITERAL: u8 = 0;
pub const COLOUR_RAMP: u8 = 1 << 6;
pub const COLOUR_TINT: u8 = 2 << 6;

/// A view: a 4 by 4 matrix (row-major) from world to clip space, and the viewport in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub m: [f64; 16],
    pub width: f64,
    pub height: f64,
    /// Pixels per world unit at the focus, for a mark's size on screen.
    pub scale: f64,
    /// The world point the view is centred on (the allocator measures distance from it).
    pub focus: [f64; 3],
}

impl Camera {
    /// A map view: orthographic, looking down the third axis, `scale` pixels per world unit.
    pub fn map(focus: [f64; 3], scale: f64, width: f64, height: f64) -> Camera {
        let (sx, sy) = (2.0 * scale / width, 2.0 * scale / height);
        let m = [sx, 0.0, 0.0, -sx * focus[0], 0.0, sy, 0.0, -sy * focus[1], 0.0, 0.0, -1e-6, 0.0, 0.0, 0.0, 0.0, 1.0];
        Camera { m, width, height, scale, focus }
    }
    /// A perspective view from a point on a sphere about `focus`, with z up: `yaw` about z
    /// from the x axis, `pitch` above the ground plane, `dist` from the focus, `fov_y` the
    /// vertical field of view (radians).
    #[allow(clippy::too_many_arguments)]
    pub fn orbit(focus: [f64; 3], yaw: f64, pitch: f64, dist: f64, fov_y: f64, width: f64, height: f64) -> Camera {
        let (cp, sp, cy, sy) = (pitch.cos(), pitch.sin(), yaw.cos(), yaw.sin());
        let eye = [focus[0] + dist * cp * cy, focus[1] + dist * cp * sy, focus[2] + dist * sp];
        let norm = |v: [f64; 3]| {
            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            [v[0] / l, v[1] / l, v[2] / l]
        };
        let cross = |a: [f64; 3], b: [f64; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let f = norm([focus[0] - eye[0], focus[1] - eye[1], focus[2] - eye[2]]);
        let up = if f[2].abs() > 0.999 { [0.0, 1.0, 0.0] } else { [0.0, 0.0, 1.0] };
        let s = norm(cross(f, up));
        let u = cross(s, f);
        let view = [s[0], s[1], s[2], -dot(s, eye), u[0], u[1], u[2], -dot(u, eye), -f[0], -f[1], -f[2], dot(f, eye), 0.0, 0.0, 0.0, 1.0];
        let (near, far) = (dist * 0.02, dist * 20.0);
        let t = 1.0 / (0.5 * fov_y).tan();
        let aspect = width / height;
        let proj = [t / aspect, 0.0, 0.0, 0.0, 0.0, t, 0.0, 0.0, 0.0, 0.0, (far + near) / (near - far), 2.0 * far * near / (near - far), 0.0, 0.0, -1.0, 0.0];
        let mut m = [0.0; 16];
        for r in 0..4 {
            for c in 0..4 {
                m[4 * r + c] = (0..4).map(|k| proj[4 * r + k] * view[4 * k + c]).sum();
            }
        }
        Camera { m, width, height, scale: height * t / (2.0 * dist), focus }
    }

    /// The matrix a GPU uses: from a position RELATIVE to the focus (a batch's offset plus a
    /// mark's position) to clip space, column-major, 32-bit. The focus is folded in here in
    /// 64-bit, so a far origin does not lose its precision on the GPU.
    pub fn relative(&self) -> [f32; 16] {
        let m = &self.m;
        let mut out = [0.0f32; 16];
        for r in 0..4 {
            for c in 0..3 {
                out[c * 4 + r] = m[4 * r + c] as f32;
            }
            let t = m[4 * r + 3] + m[4 * r] * self.focus[0] + m[4 * r + 1] * self.focus[1] + m[4 * r + 2] * self.focus[2];
            out[12 + r] = t as f32;
        }
        out
    }

    /// World to pixels (origin top left) and a depth; `None` behind the view.
    pub fn project(&self, p: [f64; 3]) -> Option<[f64; 3]> {
        let m = &self.m;
        let row = |r: usize| m[4 * r] * p[0] + m[4 * r + 1] * p[1] + m[4 * r + 2] * p[2] + m[4 * r + 3];
        let w = row(3);
        if w <= 0.0 {
            return None;
        }
        let (x, y, z) = (row(0) / w, row(1) / w, row(2) / w);
        Some([(x + 1.0) * 0.5 * self.width, (1.0 - y) * 0.5 * self.height, z])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// A mark of this tile holds a number that is not finite.
    NotFinite { tile: u64, index: usize },
}

/// One upload and one instanced draw call.
#[derive(Clone, Debug, PartialEq)]
pub struct Batch {
    pub tile: u64,
    /// The tile's origin minus the camera's focus, in 32-bit: what the GPU adds to a mark.
    pub offset: [f32; 3],
    pub kind: Kind,
    pub count: u32,
    pub bytes: Vec<u8>,
    /// A screen tile's batch: positions are device pixels, `offset` is the tile's origin.
    pub screen: bool,
    /// The tile's settings ([`Tile::params`]).
    pub params: [f32; 32],
}

/// What is said of one mark of a tagged instance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub tag: u32,
    pub layer: u8,
    pub coarse: bool,
    pub on_screen: bool,
    /// Centre in pixels, and radius in pixels.
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub depth: f64,
    pub legible: bool,
    /// A fold of this mark's packed bytes as drawn.
    pub shown: u64,
    /// A screen mark's exact rectangle (left, top, right, bottom); a pick inside it hits.
    pub rect: Option<[f64; 4]>,
}

/// What is said of one instance: every mark that carries its tag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shown {
    pub tag: u32,
    pub marks: u32,
    /// Every mark of it is within the viewport.
    pub on_screen: bool,
    /// On screen, and its largest mark is at least [`LEGIBLE_PX`] across. This is the
    /// GEOMETRIC part of "legible" alone: occlusion, contrast, truncation and the string
    /// itself are not known to this crate and are owed by the layer that knows them.
    pub legible: bool,
    /// Any mark of it drawn coarse.
    pub coarse: bool,
    /// The screen box of its marks: left, top, right, bottom.
    pub bounds: [f64; 4],
    /// A fold of the bytes of all its marks as drawn on this frame, in draw order.
    pub shown: u64,
}

/// A frame: what to draw, and what can be asked of it.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub batches: Vec<Batch>,
    pub marks: Vec<Mark>,
    /// Tiles drawn fine, and tiles drawn coarse.
    pub fine: u32,
    pub coarse: u32,
    /// Tiles not drawn because their version is not the one asked for.
    pub stale: u32,
    pub modal: Option<u8>,
}

/// A mark under this many pixels across is drawn but not legible.
pub const LEGIBLE_PX: f64 = 4.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub tiles: Vec<Tile>,
    /// While set, a pick sees this layer alone.
    pub modal: Option<u8>,
}

impl Scene {
    /// Build a frame. `version` is the version of the inputs the host holds now; `budget` is
    /// the number of marks the allocator may spend on detail.
    pub fn frame(&self, cam: &Camera, version: u64, alloc: &dyn Allocator, budget: u64) -> Result<Frame, Refusal> {
        let mut cands = Vec::new();
        let mut live = Vec::new();
        let mut stale = 0u32;
        for t in &self.tiles {
            if t.version != version {
                stale += 1;
                continue;
            }
            for (i, m) in t.marks.iter().enumerate() {
                if !m.finite() {
                    return Err(Refusal::NotFinite { tile: t.id, index: i });
                }
            }
            if t.screen {
                // always drawn whole; not the allocator's
                cands.push(Candidate { distance: 0.0, work: 0, read: 0 });
                live.push((t, [t.origin[0], t.origin[1], t.origin[2]]));
                continue;
            }
            let d = [t.origin[0] - cam.focus[0], t.origin[1] - cam.focus[1], t.origin[2] - cam.focus[2]];
            // drawing a tile fine adds its detailed decoration and the fine forms of its tagged marks
            let work = t.marks.iter().filter(|m| m.detail > 0 && !m.pin).count() as u64;
            let read = t.marks.iter().filter(|m| m.detail > 0 && !m.pin && m.tag != 0).count() as u64;
            cands.push(Candidate { distance: (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt(), work, read });
            live.push((t, d));
        }
        let fine = alloc.allocate(&cands, budget);
        let mut frame = Frame { batches: Vec::new(), marks: Vec::new(), fine: 0, coarse: 0, stale, modal: self.modal };
        let mut order: Vec<usize> = (0..live.len()).collect();
        order.sort_by_key(|&k| (live[k].0.screen, live[k].0.layer, k));
        for k in order {
            let (t, d) = &live[k];
            let is_fine = fine.get(k).copied().unwrap_or(false);
            if is_fine {
                frame.fine += 1;
            } else {
                frame.coarse += 1;
            }
            let offset = [d[0] as f32, d[1] as f32, d[2] as f32];
            for kind in KINDS {
                let mut bytes = Vec::new();
                let mut count = 0u32;
                for m in t.marks.iter().filter(|m| m.kind == kind) {
                    let whole = is_fine || m.detail == 0 || m.pin;
                    if !whole && m.tag == 0 {
                        continue;
                    }
                    let at = bytes.len();
                    m.pack_as(&mut bytes, !whole);
                    count += 1;
                    if m.tag != 0 && t.screen {
                        let shown = fnv(0xcbf2_9ce4_8422_2325, &bytes[at..]);
                        let (cx, cy) = (t.origin[0] + m.pos[0] as f64, t.origin[1] + m.pos[1] as f64);
                        let (hw, hh) = match m.kind {
                            Kind::Disc | Kind::Glyph => (m.size[0].abs() as f64, m.size[0].abs() as f64),
                            _ => (0.5 * m.size[0].abs() as f64, 0.5 * m.size[1].abs() as f64),
                        };
                        let rect = [cx - hw, cy - hh, cx + hw, cy + hh];
                        let on = rect[2] >= 0.0 && rect[0] <= cam.width && rect[3] >= 0.0 && rect[1] <= cam.height;
                        frame.marks.push(Mark { tag: m.tag, layer: t.layer, coarse: false, on_screen: on, x: cx, y: cy, r: hw.max(hh), depth: f64::NEG_INFINITY, legible: on && hh >= 0.5 * LEGIBLE_PX, shown, rect: Some(rect) });
                    } else if m.tag != 0 {
                        let shown = fnv(0xcbf2_9ce4_8422_2325, &bytes[at..]);
                        let world = [t.origin[0] + m.pos[0] as f64, t.origin[1] + m.pos[1] as f64, t.origin[2] + m.pos[2] as f64];
                        let r = m.bound() as f64 * cam.scale;
                        let (x, y, depth, on) = match cam.project(world) {
                            Some(p) => (p[0], p[1], p[2], p[0] + r >= 0.0 && p[0] - r <= cam.width && p[1] + r >= 0.0 && p[1] - r <= cam.height),
                            None => (f64::NAN, f64::NAN, f64::INFINITY, false),
                        };
                        frame.marks.push(Mark { tag: m.tag, layer: t.layer, coarse: !whole, on_screen: on, x, y, r, depth, legible: on && 2.0 * r >= LEGIBLE_PX, shown, rect: None });
                    }
                }
                if count > 0 {
                    frame.batches.push(Batch { tile: t.id, offset, kind, count, bytes, screen: t.screen, params: t.params });
                }
            }
        }
        Ok(frame)
    }
}

impl Frame {
    /// The instance under a point in pixels: among marks whose circle holds the point, the
    /// highest layer, then the nearest in depth, then the nearest centre. While a modal layer
    /// is set, that layer alone. Pure.
    pub fn pick(&self, x: f64, y: f64) -> Option<u32> {
        let mut best: Option<(&Mark, f64)> = None;
        for m in self.marks.iter().filter(|m| m.on_screen && self.modal.map_or(true, |l| m.layer == l)) {
            let d2 = (m.x - x) * (m.x - x) + (m.y - y) * (m.y - y);
            match m.rect {
                Some(r) => {
                    if x < r[0] || x > r[2] || y < r[1] || y > r[3] {
                        continue;
                    }
                }
                None => {
                    let r = m.r.max(LEGIBLE_PX);
                    if d2 > r * r {
                        continue;
                    }
                }
            }
            let better = match best {
                None => true,
                Some((b, bd2)) => {
                    // a screen mark (on a panel) is over the world under it
                    let (ms, bs) = (m.rect.is_some(), b.rect.is_some());
                    (ms && !bs)
                        || (ms == bs
                            && (m.layer > b.layer || (m.layer == b.layer && (m.depth < b.depth || (m.depth == b.depth && (d2 < bd2 || (d2 == bd2 && m.tag < b.tag)))))))
                }
            };
            if better {
                best = Some((m, d2));
            }
        }
        best.map(|(m, _)| m.tag)
    }
    /// What is said of one instance, or `None` if no mark of this frame carries its tag.
    pub fn report(&self, tag: u32) -> Option<Shown> {
        let mut out: Option<Shown> = None;
        let mut widest = 0.0f64;
        for m in self.marks.iter().filter(|m| m.tag == tag) {
            let b = m.rect.unwrap_or([m.x - m.r, m.y - m.r, m.x + m.r, m.y + m.r]);
            widest = widest.max(2.0 * m.r);
            out = Some(match out {
                None => Shown { tag, marks: 1, on_screen: m.on_screen, legible: false, coarse: m.coarse, bounds: b, shown: fnv(0xcbf2_9ce4_8422_2325, &m.shown.to_le_bytes()) },
                Some(s) => Shown {
                    marks: s.marks + 1,
                    on_screen: s.on_screen && m.on_screen,
                    coarse: s.coarse || m.coarse,
                    bounds: [s.bounds[0].min(b[0]), s.bounds[1].min(b[1]), s.bounds[2].max(b[2]), s.bounds[3].max(b[3])],
                    shown: fnv(s.shown, &m.shown.to_le_bytes()),
                    ..s
                },
            });
        }
        out.map(|s| Shown { legible: s.on_screen && widest >= LEGIBLE_PX, ..s })
    }
    /// A fold of every batch's tile, kind, count and bytes (FNV-1a, 64-bit), in order.
    pub fn digest(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for b in &self.batches {
            h = fnv(h, &b.tile.to_le_bytes());
            h = fnv(h, &[b.kind as u8]);
            h = fnv(h, &b.count.to_le_bytes());
            h = fnv(h, &b.bytes);
        }
        h
    }
    /// The instances drawn.
    pub fn drawn(&self) -> u64 {
        self.batches.iter().map(|b| b.count as u64).sum()
    }
}

/// FNV-1a, 64-bit, continued from `h`.
fn fnv(mut h: u64, b: &[u8]) -> u64 {
    for &x in b {
        h ^= x as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(tiles: Vec<Tile>) -> Scene {
        Scene { tiles, modal: None }
    }

    fn tile(id: u64, ox: f64, n: u32, tag0: u32) -> Tile {
        let mut marks = vec![Instance::new(Kind::Patch, [0.0; 3], [100.0, 100.0, 0.0])];
        for i in 0..n {
            let p = [10.0 + 8.0 * (i % 10) as f32, 10.0 + 8.0 * (i / 10) as f32, 0.0];
            marks.push(Instance::new(Kind::Prism, p, [3.0, 3.0, 6.0]).tagged(tag0 + i).detail(1));
            // a garden tree beside it: decoration, detailed
            marks.push(Instance::new(Kind::Disc, [p[0] + 3.0, p[1], 0.0], [1.0, 0.0, 0.0]).detail(1));
        }
        Tile { id, origin: [ox, 0.0, 0.0], version: 7, layer: 0, screen: false, params: PARAMS_NEUTRAL, marks }
    }
    fn cam() -> Camera {
        Camera::map([50.0, 50.0, 0.0], 4.0, 800.0, 600.0)
    }

    #[test]
    fn an_instance_packs_to_52_bytes_and_a_frame_repeats() {
        let mut b = Vec::new();
        Instance::new(Kind::Rod, [1.0, 2.0, 3.0], [0.5, 0.0, 0.0]).tagged(9).pack(&mut b);
        assert_eq!(b.len(), INSTANCE_BYTES);
        assert_eq!(&b[0..4], &1.0f32.to_le_bytes());
        assert_eq!(&b[44..48], &9u32.to_le_bytes());
        assert_eq!(b[48], Kind::Rod as u8);
        let s = scene(vec![tile(1, 0.0, 40, 100), tile(2, 100.0, 40, 200)]);
        let (a, c) = (s.frame(&cam(), 7, &Ring, 1000).unwrap(), s.frame(&cam(), 7, &Ring, 1000).unwrap());
        assert_eq!(a, c);
        assert_eq!(a.digest(), c.digest());
        assert_eq!(a.drawn(), 162);
        assert!(a.batches.iter().all(|x| x.bytes.len() == x.count as usize * INSTANCE_BYTES));
        assert_eq!(a.batches.len(), 6, "one batch per tile and kind");
    }

    #[test]
    fn a_number_that_is_not_one_refuses_the_frame() {
        let mut s = scene(vec![tile(1, 0.0, 5, 100)]);
        s.tiles[0].marks[3].size[1] = f32::NAN;
        assert_eq!(s.frame(&cam(), 7, &Ring, 1000), Err(Refusal::NotFinite { tile: 1, index: 3 }));
    }

    #[test]
    fn an_instance_is_every_mark_with_its_tag_and_a_pick_names_it() {
        let mut s = scene(vec![tile(1, 0.0, 40, 100)]);
        // instance 113 gains a second mark (a flag beside the building), in a second tile
        s.tiles.push(Tile { id: 2, origin: [0.0; 3], version: 7, layer: 0, screen: false, params: PARAMS_NEUTRAL, marks: vec![Instance::new(Kind::Glyph, [40.0, 18.0, 0.0], [1.0, 0.0, 0.0]).tagged(113)] });
        let f = s.frame(&cam(), 7, &Ring, 1000).unwrap();
        // mark 100+13 sits at local (34, 18): on screen at ((34-50)*4+400, 300-(18-50)*4)
        assert_eq!(f.pick(336.0, 428.0), Some(113));
        assert_eq!(f.pick(360.0, 428.0), Some(113), "its second mark names the same instance");
        assert_eq!(f.pick(336.0, 428.0), Some(113), "pure");
        // the ground patch and the trees carry no tag
        assert_eq!(f.pick(700.0, 80.0), None);
        let r = f.report(113).unwrap();
        assert!(r.marks == 2 && r.on_screen && r.legible && !r.coarse);
        assert!(r.bounds[0] < 336.0 && r.bounds[2] > 360.0);
        assert_eq!(f.report(114).unwrap().marks, 1);
        assert!(f.report(5).is_none());
    }

    #[test]
    fn detail_coarsens_a_tagged_mark_and_never_removes_it() {
        let mut s = scene(vec![tile(1, 0.0, 40, 100), tile(2, 5000.0, 40, 200)]);
        s.tiles[1].marks[7] = s.tiles[1].marks[7].pinned();
        let c = cam();
        let fine = s.frame(&c, 7, &Ring, 1000).unwrap();
        let tight = s.frame(&c, 7, &Ring, 80).unwrap();
        assert_eq!((fine.fine, fine.coarse), (2, 0));
        assert_eq!((tight.fine, tight.coarse), (1, 1), "the budget covers the near tile alone");
        // the far tile keeps its ground and all 40 tagged marks; its 40 trees are dropped
        assert_eq!(tight.drawn(), 81 + 1 + 40);
        assert_eq!(tight.marks.len(), fine.marks.len(), "every tagged mark is drawn in both");
        let (pinned, plain) = (tight.report(203).unwrap(), tight.report(210).unwrap());
        assert!(!pinned.coarse && plain.coarse);
        // what is shown differs exactly where the form differs
        assert_eq!(pinned.shown, fine.report(203).unwrap().shown);
        assert_ne!(plain.shown, fine.report(210).unwrap().shown);
        // with budget enough nothing is coarse: no tagged mark is unreachable (L9)
        assert!(fine.marks.iter().all(|m| !m.coarse));
        // what is drawn in both is byte-identical: the near tile's batches
        let near = |f: &Frame| f.batches.iter().filter(|b| b.tile == 1).cloned().collect::<Vec<_>>();
        assert_eq!(near(&fine), near(&tight));
    }

    #[test]
    fn an_open_modal_layer_takes_every_pick() {
        let mut s = scene(vec![tile(1, 0.0, 40, 100)]);
        let sheet = Tile { id: 9, origin: [0.0; 3], version: 7, layer: 5, screen: false, params: PARAMS_NEUTRAL, marks: vec![Instance::new(Kind::Quad, [70.0, 70.0, 0.0], [6.0, 4.0, 0.0]).tagged(900).pinned()] };
        s.tiles.push(sheet);
        let f = s.frame(&cam(), 7, &Ring, 1000).unwrap();
        assert_eq!(f.pick(336.0, 428.0), Some(113), "no modal: the scene is picked");
        assert_eq!(f.pick(480.0, 220.0), Some(900), "a higher layer is over a lower one");
        s.modal = Some(5);
        let f = s.frame(&cam(), 7, &Ring, 1000).unwrap();
        assert_eq!(f.pick(336.0, 428.0), None, "under an open sheet nothing is picked");
        assert_eq!(f.pick(480.0, 220.0), Some(900));
        assert_eq!(f.batches.last().unwrap().tile, 9, "the highest layer is drawn last");
    }

    #[test]
    fn a_screen_tile_is_over_the_world_and_is_picked_by_its_exact_rectangle() {
        let mut s = scene(vec![tile(1, 0.0, 40, 100)]);
        // a panel 200 by 40 px centred at (300, 420), holding a button (tag 7) 80 by 24 at
        // its left, over the building whose centre is at (336, 428)
        let mut button = Instance::new(Kind::Quad, [-50.0, 0.0, 0.0], [80.0, 24.0, 0.0]).tagged(7);
        button.aux = [6.0, 0.0, 0.0];
        let label = Instance::new(Kind::Run, [-50.0, 0.0, 0.0], [52.0, 14.0, 0.0]);
        s.tiles.push(Tile { id: 50, origin: [300.0, 420.0, 0.0], version: 7, layer: 0, screen: true, params: PARAMS_NEUTRAL, marks: vec![Instance::new(Kind::Quad, [0.0; 3], [200.0, 40.0, 0.0]), button, label] });
        let f = s.frame(&cam(), 7, &Ring, 1000).unwrap();
        assert_eq!(f.pick(260.0, 425.0), Some(7), "the button, inside its rectangle");
        assert_eq!(f.pick(212.0, 425.0), Some(7));
        assert_eq!(f.pick(205.0, 425.0), None, "outside the rectangle, on the untagged panel");
        // the building under the panel's right half is still picked: the panel is decoration
        assert_eq!(f.pick(336.0, 428.0), Some(113));
        let r = f.report(7).unwrap();
        assert_eq!(r.bounds, [210.0, 408.0, 290.0, 432.0]);
        assert!(r.legible);
        // screen batches come last, flagged, with the origin as their offset
        let last = f.batches.last().unwrap();
        assert!(last.screen && last.kind == Kind::Run && last.offset == [300.0, 420.0, 0.0]);
        assert!(f.batches.iter().take_while(|b| !b.screen).count() >= 2);
    }

    #[test]
    fn a_tile_carries_its_settings_to_every_batch_and_a_mark_its_colour_mode() {
        let mut t = tile(1, 0.0, 3, 100);
        t.params[PARAM_DIM] = 0.62;
        let mut ground = Instance::new(Kind::Patch, [0.0; 3], [10.0, 10.0, 0.0]);
        ground.variant = COLOUR_RAMP;
        ground.aux[0] = 0.37;
        t.marks.push(ground);
        let f = scene(vec![t]).frame(&cam(), 7, &Ring, 1000).unwrap();
        assert!(f.batches.iter().all(|b| b.params[PARAM_DIM] == 0.62 && b.params[PARAM_TINT] == 1.0));
        let patch = f.batches.iter().find(|b| b.kind == Kind::Patch).unwrap();
        let last = &patch.bytes[patch.bytes.len() - INSTANCE_BYTES..];
        assert_eq!(last[51] & 0xC0, COLOUR_RAMP);
        assert_eq!(&last[24..28], &0.37f32.to_le_bytes());
    }

    #[test]
    fn a_tile_of_another_version_is_not_drawn() {
        let mut s = scene(vec![tile(1, 0.0, 10, 100), tile(2, 100.0, 10, 200)]);
        s.tiles[1].version = 6;
        let f = s.frame(&cam(), 7, &Ring, 1000).unwrap();
        assert_eq!(f.stale, 1);
        assert!(f.report(205).is_none() && f.batches.iter().all(|b| b.tile == 1));
    }

    #[test]
    fn an_orbit_camera_sees_its_focus_at_the_centre_and_nearer_is_shallower() {
        let c = Camera::orbit([12.0, 12.0, 12.0], 0.7, 0.4, 40.0, 0.8, 1280.0, 800.0);
        let p = c.project([12.0, 12.0, 12.0]).unwrap();
        assert!((p[0] - 640.0).abs() < 1e-9 && (p[1] - 400.0).abs() < 1e-9);
        // a point toward the eye is nearer in depth; up on the world is up on the screen
        let toward = c.project([12.0 + 5.0 * 0.4f64.cos() * 0.7f64.cos(), 12.0 + 5.0 * 0.4f64.cos() * 0.7f64.sin(), 12.0 + 5.0 * 0.4f64.sin()]).unwrap();
        assert!(toward[2] < p[2]);
        assert!(c.project([12.0, 12.0, 14.0]).unwrap()[1] < 400.0);
        // the relative matrix applied to (world - focus) agrees with the absolute one
        let r = c.relative();
        let q = [3.0f32, -2.0, 1.5];
        let clip: Vec<f32> = (0..4).map(|row| r[row] * q[0] + r[4 + row] * q[1] + r[8 + row] * q[2] + r[12 + row]).collect();
        let a = c.project([15.0, 10.0, 13.5]).unwrap();
        let px = (clip[0] / clip[3] + 1.0) * 0.5 * 1280.0;
        assert!((px as f64 - a[0]).abs() < 1e-3, "{px} {}", a[0]);
        // a sphere of radius 1 at the focus is about `scale` pixels in radius
        let edge = c.project([12.0, 12.0, 13.0]).unwrap();
        assert!(((400.0 - edge[1]) / c.scale - 1.0).abs() < 0.1);
    }

    #[test]
    fn a_far_origin_keeps_its_precision() {
        // 20,000 km from the world's origin, two marks 1 unit apart: the offset the GPU adds
        // is taken from the focus in 64-bit, so they land 4 pixels apart at 4 pixels a unit
        let far = 2.0e7;
        let t = Tile { id: 1, origin: [far, far, 0.0], version: 1, layer: 0, screen: false, params: PARAMS_NEUTRAL, marks: vec![
            Instance::new(Kind::Disc, [0.0, 0.0, 0.0], [0.4, 0.0, 0.0]).tagged(1),
            Instance::new(Kind::Disc, [1.0, 0.0, 0.0], [0.4, 0.0, 0.0]).tagged(2),
        ] };
        let c = Camera::map([far + 0.5, far, 0.0], 4.0, 800.0, 600.0);
        let f = scene(vec![t]).frame(&c, 1, &Ring, 10).unwrap();
        let (a, b) = (f.report(1).unwrap(), f.report(2).unwrap());
        let mid = |s: Shown| 0.5 * (s.bounds[0] + s.bounds[2]);
        assert!((mid(b) - mid(a) - 4.0).abs() < 1e-6 && (mid(a) - 398.0).abs() < 1e-6, "{} {}", mid(a), mid(b));
        assert_eq!(f.batches[0].offset, [-0.5, 0.0, 0.0]);
    }
}
