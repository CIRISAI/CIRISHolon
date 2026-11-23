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
pub mod digest;
pub mod ladder;
pub mod twin;
pub mod ui;

pub use alloc::{Acuity, Allocator, Bits, Candidate, Ring};

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
    /// A rectangle of a HOST-SUPPLIED texture (`INTERFACE.md` §1, D12): a QR grid, or a
    /// media frame the host decoded. Laid out as a run: `size[0]` by `size[1]` centred at
    /// `pos`, sampling `aux[0], aux[1]` (u0, v0) to `aux[2], angle` (u1, v1) of the texture
    /// the host put in slot `variant`, multiplied by `rgba` (white is the texture as it is).
    /// The host chooses each slot's sampling (nearest for a QR grid). Screen tiles only.
    /// A TAGGED image must say what it is ([`Bind::Media`] or [`Bind::Qr`]); anything else
    /// is refused (L10: media is never drawn bare). A blurred frame (`__blur`) is a blurred
    /// texture the host supplies; the core does not blur.
    Image = 12,
}

pub const KINDS: [Kind; 12] = [Kind::Quad, Kind::Line, Kind::Arc, Kind::Disc, Kind::Box, Kind::Prism, Kind::Sphere, Kind::Rod, Kind::Patch, Kind::Glyph, Kind::Run, Kind::Image];

/// What a text provider says of how it drew a string.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Drawn {
    /// The provider did not say. Nothing is known about truncation.
    #[default]
    Unsaid,
    /// The whole string, as given.
    Exact,
    /// Cut short (wider than the provider's page, for one).
    Cut,
    /// Drawn, but not as given: case changed by its style, or whitespace collapsed by a flow.
    Changed,
}

impl Drawn {
    /// Two parts of one string: a definite fault wins, then not knowing.
    pub fn and(self, o: Drawn) -> Drawn {
        use Drawn::*;
        match (self, o) {
            (Cut, _) | (_, Cut) => Cut,
            (Changed, _) | (_, Changed) => Changed,
            (Unsaid, _) | (_, Unsaid) => Unsaid,
            _ => Exact,
        }
    }
}

/// What a mark is bound to beyond its record. It is never packed and never reaches a GPU.
/// The frame reads it, and the report returns it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Bind {
    #[default]
    None,
    /// A run's SOURCE STRING: the SHA-256 of the UTF-8 bytes handed to the text provider,
    /// supplied by whoever shaped it (the UI layer), and what the provider said of the drawing.
    /// The screen standard's L12 compares this with the digest of the value a write sends.
    Source { sha256: [u8; 32], drawn: Drawn },
    /// A tagged image that is MEDIA content: the handles of its alt text and of its AI
    /// disclosure. Each must be drawn in the same frame as a run that carries a source,
    /// or the frame is refused (L10).
    Media { alt: u32, disclosure: u32 },
    /// A tagged image that is a QR grid.
    Qr,
}

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
    /// Not packed: a run's source string, an image's media slots.
    pub bind: Bind,
}

impl Instance {
    pub fn new(kind: Kind, pos: [f32; 3], size: [f32; 3]) -> Instance {
        Instance { kind, pos, size, aux: [0.0; 3], angle: 0.0, rgba: [255; 4], tag: 0, detail: 0, pin: false, variant: 0, bind: Bind::None }
    }
    pub fn bound_to(mut self, b: Bind) -> Instance {
        self.bind = b;
        self
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
                let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                if self.kind == Kind::Arc {
                    // the curve lies in the hull of its ends and control point; the control point
                    // is the chord's midpoint moved hypot(size[1], size[2])
                    l.max(0.5 * l + (s[1] * s[1] + s[2] * s[2]).sqrt())
                } else {
                    l
                }
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
pub const PARAM_TEX: usize = 20; // the field's strength (0 off)
pub const PARAM_FLOOR_CASING: usize = 21; // least half width of a CASING line, in pixels (0 off)
pub const PARAM_FLOOR: usize = 24; // least half width of any other world line or arc, in pixels (0 off)
pub const PARAM_A_GROUND: usize = 25; // alpha multipliers by surface class, 1 neutral
pub const PARAM_A_CASING: usize = 26;
pub const PARAM_A_ROAD: usize = 27;
pub const PARAM_A_ROOF: usize = 28;
pub const PARAM_RING: usize = 29; // 29..31: a radial fade's centre x, y and radius in device pixels (radius 0 off)

/// Bit 0 of a world mark's `variant`: a line or arc, or a disc, in the CASING surface class
/// (a casing under its road; a disc such as ground mottling). Bit 2 of a disc's `variant`
/// gives it the canopy field (a tree's crown); `twin::DISC_CASING` and
/// `twin::DISC_CANOPY_FIELD` name the two for discs. The surface classes the backend derives
/// from kind, colour mode and these bits are listed in `web/draw.js`'s header.
pub const VARIANT_CASING: u8 = 0x01;
/// Bit 1 of a world line's or arc's `variant`: round ends (a cap of the line's half width,
/// after the floor); a polyline of round-ended segments has round joins. Without it a line
/// ends flush, as the twin's streets do at an exit.
pub const VARIANT_ROUND: u8 = 0x02;
/// Bit 3 of a box's or prism's `variant`: ORIENTED. `aux` is the box's local z axis (any
/// length; zero means world z): `size[2]` runs along it from `pos`, and `angle` turns its local
/// x about it, from world x laid across the axis (world y when the axis is along x). With
/// `aux = [0, 0, 1]` it is the box without the bit. A box tipped over this way needs no stack
/// of upright slices. In RAMP mode an oriented box reads `t = 0` (its `aux` is taken). The
/// pick's bounding sphere is about `pos` and turning about `pos` keeps it, so `bound` holds.
pub const VARIANT_ORIENT: u8 = 0x08;
/// Bits 4..5 of a LIT mark's `variant` (box, prism, sphere, rod): its material, a closed list.
/// KIND: the kind's own specular and rim. MATTE: no specular. METAL: a stronger, tighter
/// specular and rim. GLOW: unlit, the colour as given; it neither casts nor receives a shadow.
/// Unlit kinds ignore these bits. `web/draw.js`'s header has the shading.
pub const VARIANT_MATERIAL: u8 = 0x30;
pub const MATERIAL_KIND: u8 = 0x00;
pub const MATERIAL_MATTE: u8 = 0x10;
pub const MATERIAL_METAL: u8 = 0x20;
pub const MATERIAL_GLOW: u8 = 0x30;

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
    p[25] = 1.0;
    p[26] = 1.0;
    p[27] = 1.0;
    p[28] = 1.0;
    p
};

/// A mark's colour mode, in the top two bits of its `variant` byte for every kind but
/// `Run` (whose `variant` is its atlas page). LITERAL: `rgba`. RAMP: the mix of the tile's
/// RAMP0 and RAMP1 at `t`, alpha from `rgba`. TINT: `rgba.rgb` times the tile's TINT.
/// Where `t` is: `aux[0]` for the kinds whose `aux` is otherwise unused (world quad, patch,
/// disc, glyph, box and prism unless ORIENTED, sphere); `angle` for the kinds whose `aux` is a second point or a
/// corner radius and whose `angle` is unused (line, arc, rod, and every SCREEN mark).
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

    /// The pixels a sphere of radius `r` at world point `p` spans across, at its depth.
    pub fn pixels(&self, p: [f64; 3], r: f64) -> f64 {
        let m = &self.m;
        let w = |q: [f64; 3]| m[12] * q[0] + m[13] * q[1] + m[14] * q[2] + m[15];
        let (wf, wp) = (w(self.focus), w(p));
        if wp <= 0.0 {
            return 0.0;
        }
        2.0 * r * self.scale * wf / wp
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
    /// A tagged image that says neither [`Bind::Media`] nor [`Bind::Qr`] (L10).
    BareImage { tile: u64, index: usize },
    /// A media image whose alt text or AI disclosure (`missing`, a handle) is not drawn in
    /// this frame as a run with a source string (L10).
    MediaUnbound { tile: u64, index: usize, missing: u32 },
    /// An image in a world tile: no backend draws one there.
    ImageInWorld { tile: u64, index: usize },
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
    /// The tile was drawn COARSE by the allocator: a backend may use its cheaper meshes
    /// (spheres and rods) for every mark of the batch.
    pub coarse: bool,
}

/// What is said of one mark of a tagged instance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub tag: u32,
    pub kind: Kind,
    pub layer: u8,
    pub coarse: bool,
    pub on_screen: bool,
    /// Centre in pixels, and radius in pixels.
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub depth: f64,
    /// At least [`LEGIBLE_PX`] across (the size floor alone).
    pub legible: bool,
    /// A fold of this mark's packed bytes as drawn.
    pub shown: u64,
    /// A screen mark's exact rectangle (left, top, right, bottom); a pick inside it hits.
    pub rect: Option<[f64; 4]>,
    /// A screen mark's place in [`Frame::covers`] (its draw order); `u32::MAX` for a world mark.
    pub cover: u32,
    pub bind: Bind,
}

/// One screen mark as drawn, tagged or not: what the legibility test reads, in draw order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cover {
    pub tag: u32,
    pub kind: Kind,
    pub rect: [f64; 4],
    /// A quad's or patch's corner radius in pixels (0 for other kinds).
    pub radius: f64,
    /// Its colour as the backend computes it: the colour mode applied, then the tile's DIM
    /// (rgb times DIM's rgb, alpha times DIM's w), in 0..1.
    pub ink: [f64; 4],
}

/// Three answers: the core can say yes, can say no, or cannot know.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tri {
    Yes,
    No,
    Unknown,
}

impl Tri {
    fn and(self, o: Tri) -> Tri {
        match (self, o) {
            (Tri::No, _) | (_, Tri::No) => Tri::No,
            (Tri::Unknown, _) | (_, Tri::Unknown) => Tri::Unknown,
            _ => Tri::Yes,
        }
    }
}

/// The parts of "legible" (the screen standard's L12, `INTERFACE.md` §8 item 5) that the
/// core can attest for one instance, each on its own. "Tagged" holds for anything reported.
/// "The exact string that is sent" is the UI layer's comparison of [`Shown::source`] with
/// the value it sends. "On the frame actually presented" is the host's: it draws the frame
/// it was given.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Legibility {
    /// Every mark is wholly within the viewport (not only touching it).
    pub inside: bool,
    /// The largest mark is at least [`LEGIBLE_PX`] across.
    pub size: bool,
    /// SCREEN marks: no mark of another instance is drawn over any part of any of them (by
    /// rectangle, so a disc or glyph occludes by its bounding square: an over-estimate,
    /// never an under-estimate). WORLD marks: `No` when a screen mark covers one; otherwise
    /// `Unknown`, because world against world needs the backend's depth buffer, which the core
    /// does not have.
    pub unoccluded: Tri,
    /// Every run of it is on an OPAQUE ground the core can see whole (screen quads that
    /// contain the run's rectangle, the lowest of them opaque), and its ink reaches
    /// [`CONTRAST_MIN`] against that ground. `Unknown` when the ground is not uniform, or not
    /// opaque, so that the world shows through. `Yes` with no runs.
    pub contrast: Tri,
    /// The lowest contrast ratio of its runs (infinite with no runs, NaN where unknown).
    pub min_contrast: f64,
    /// Every run was drawn whole as given, as its text provider said ([`Drawn::Exact`]).
    /// `Unknown` when a provider did not say. `Yes` with no runs.
    pub untruncated: Tri,
    /// Every run of it carries one and the same source digest. `No` when two differ,
    /// `Unknown` when a run carries none. `Yes` with no runs.
    pub string: Tri,
}

/// What is said of one instance: every mark that carries its tag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shown {
    pub tag: u32,
    pub marks: u32,
    /// Every mark of it touches the viewport. ([`Legibility::inside`] is the stronger
    /// "wholly within".)
    pub on_screen: bool,
    /// Everything [`Legibility`] can say holds: on screen and wholly inside, the size floor,
    /// unoccluded, the contrast floor, untruncated, and one source string. Only then may a
    /// UI layer use it to license a send, after it has compared [`Shown::source`] with the
    /// value it sends.
    ///
    /// CORRECTION 2026-11-22 (issue #6): until this date `legible` was the size floor alone,
    /// and its doc said so. A world mark is now never legible, because its occlusion is not
    /// known. [`Legibility::size`] is the old meaning.
    pub legible: bool,
    pub why: Legibility,
    /// Any mark of it drawn coarse.
    pub coarse: bool,
    /// The screen box of its marks: left, top, right, bottom.
    pub bounds: [f64; 4],
    /// A fold of the bytes of all its marks as drawn on this frame, in draw order. It is
    /// evidence about a frame, and comparable to nothing in the screen standard (§9).
    pub shown: u64,
    /// The SHA-256 of the source string its runs were shaped from, when they all carry the same
    /// one ([`Legibility::string`] is `Yes` and there is at least one run).
    pub source: Option<[u8; 32]>,
}

/// A frame: what to draw, and what can be asked of it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Frame {
    pub batches: Vec<Batch>,
    pub marks: Vec<Mark>,
    /// Every screen mark, in the order drawn.
    pub covers: Vec<Cover>,
    /// The viewport, pixels.
    pub width: f64,
    pub height: f64,
    /// Tiles drawn fine, and tiles drawn coarse.
    pub fine: u32,
    pub coarse: u32,
    /// Tiles not drawn because their version is not the one asked for.
    pub stale: u32,
    pub modal: Option<u8>,
    /// The frame's acuity certificate: the largest number of pixels by which a tile drawn
    /// coarse differs from its fine drawing (0 when everything is fine).
    pub remainder_px: f64,
}

/// The fraction of a mark's size by which a backend's cheaper mesh falls short of the true
/// outline (`web/draw.js`: a coarse sphere has 10 segments around, `1 - cos(pi/10)`; a coarse
/// rod 8 sides, `1 - cos(pi/8)`). Marks a backend draws the same coarse or fine: 0.
pub fn coarse_shortfall(kind: Kind) -> f64 {
    match kind {
        Kind::Sphere => 1.0 - (std::f64::consts::PI / 10.0).cos(),
        Kind::Rod => 1.0 - (std::f64::consts::PI / 8.0).cos(),
        _ => 0.0,
    }
}

/// A mark under this many pixels across is drawn but not legible.
pub const LEGIBLE_PX: f64 = 4.0;

/// The least contrast ratio of a run's ink against its ground for it to count as legible:
/// WCAG 2.x success criterion 1.4.3 (AA) for text of normal size, `(L1 + 0.05) / (L2 + 0.05)`
/// with sRGB relative luminance. The large-text allowance (3:1) is not taken, because the core
/// does not know a run's point size.
pub const CONTRAST_MIN: f64 = 4.5;

/// A screen mark's rectangle in pixels: a line or arc by its two ends and its width (an arc
/// with its control point too), a disc or glyph by its radius, anything else by its size
/// about `pos`.
fn screen_rect(m: &Instance, origin: [f64; 3]) -> [f64; 4] {
    let (cx, cy) = (origin[0] + m.pos[0] as f64, origin[1] + m.pos[1] as f64);
    match m.kind {
        Kind::Disc | Kind::Glyph => {
            let r = m.size[0].abs() as f64;
            [cx - r, cy - r, cx + r, cy + r]
        }
        Kind::Line | Kind::Arc => {
            let (ex, ey) = (origin[0] + m.aux[0] as f64, origin[1] + m.aux[1] as f64);
            let mut hw = 0.5 * m.size[0].abs() as f64;
            if m.kind == Kind::Arc {
                hw += (m.size[1] as f64).hypot(m.size[2] as f64);
            }
            [cx.min(ex) - hw, cy.min(ey) - hw, cx.max(ex) + hw, cy.max(ey) + hw]
        }
        _ => {
            let (hw, hh) = (0.5 * m.size[0].abs() as f64, 0.5 * m.size[1].abs() as f64);
            [cx - hw, cy - hh, cx + hw, cy + hh]
        }
    }
}

/// A screen mark's colour as the backend's screen pass computes it (`web/draw.js`, SVS).
fn screen_ink(m: &Instance, p: &[f32; 32]) -> [f64; 4] {
    let c = |i: usize| m.rgba[i] as f64 / 255.0;
    let mut rgb = [c(0), c(1), c(2)];
    if !matches!(m.kind, Kind::Run | Kind::Image) {
        let t = (m.angle as f64).clamp(0.0, 1.0);
        match m.variant & 0xc0 {
            COLOUR_RAMP => {
                for (k, v) in rgb.iter_mut().enumerate() {
                    *v = p[PARAM_RAMP0 + k] as f64 * (1.0 - t) + p[PARAM_RAMP1 + k] as f64 * t;
                }
            }
            COLOUR_TINT => {
                for (k, v) in rgb.iter_mut().enumerate() {
                    *v *= p[PARAM_TINT + k] as f64;
                }
            }
            _ => {}
        }
    }
    let d = |k: usize| p[PARAM_DIM + k] as f64;
    [(rgb[0] * d(0)).clamp(0.0, 1.0), (rgb[1] * d(1)).clamp(0.0, 1.0), (rgb[2] * d(2)).clamp(0.0, 1.0), (c(3) * d(3)).clamp(0.0, 1.0)]
}

/// sRGB relative luminance (WCAG 2.x).
fn luminance(c: [f64; 3]) -> f64 {
    let lin = |v: f64| if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

/// The WCAG contrast ratio of two colours.
pub fn contrast_ratio(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Whether a point is inside a rectangle with rounded corners (the screen pass's distance).
fn in_rounded(rect: &[f64; 4], radius: f64, p: [f64; 2]) -> bool {
    let (cx, cy) = (0.5 * (rect[0] + rect[2]), 0.5 * (rect[1] + rect[3]));
    let (hx, hy) = (0.5 * (rect[2] - rect[0]), 0.5 * (rect[3] - rect[1]));
    let (qx, qy) = ((p[0] - cx).abs() - (hx - radius), (p[1] - cy).abs() - (hy - radius));
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - radius <= 1e-9
}

fn overlaps(a: &[f64; 4], b: &[f64; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

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
        // L10: the handles drawn as runs with a source string, which a media image may name
        let sourced: std::collections::HashSet<u32> = self
            .tiles
            .iter()
            .filter(|t| t.version == version)
            .flat_map(|t| t.marks.iter())
            .filter(|m| m.kind == Kind::Run && m.tag != 0 && matches!(m.bind, Bind::Source { .. }))
            .map(|m| m.tag)
            .collect();
        for t in &self.tiles {
            if t.version != version {
                stale += 1;
                continue;
            }
            for (i, m) in t.marks.iter().enumerate() {
                if !m.finite() {
                    return Err(Refusal::NotFinite { tile: t.id, index: i });
                }
                if m.kind == Kind::Image {
                    if !t.screen {
                        return Err(Refusal::ImageInWorld { tile: t.id, index: i });
                    }
                    if m.tag != 0 {
                        match m.bind {
                            Bind::Qr => {}
                            Bind::Media { alt, disclosure } => {
                                for h in [alt, disclosure] {
                                    if h == 0 || !sourced.contains(&h) {
                                        return Err(Refusal::MediaUnbound { tile: t.id, index: i, missing: h });
                                    }
                                }
                            }
                            _ => return Err(Refusal::BareImage { tile: t.id, index: i }),
                        }
                    }
                }
            }
            if t.screen {
                // always drawn whole; not the allocator's
                cands.push(Candidate { distance: 0.0, work: 0, read: 0, error_px: 0.0 });
                live.push((t, [t.origin[0], t.origin[1], t.origin[2]]));
                continue;
            }
            let d = [t.origin[0] - cam.focus[0], t.origin[1] - cam.focus[1], t.origin[2] - cam.focus[2]];
            // drawing a tile fine adds its detailed decoration and the fine forms of its tagged marks
            // and the marks a backend draws more cheaply when the tile is coarse
            let work = t.marks.iter().filter(|m| !m.pin && (m.detail > 0 || coarse_shortfall(m.kind) > 0.0)).count() as u64;
            let read = t.marks.iter().filter(|m| m.detail > 0 && !m.pin && m.tag != 0).count() as u64;
            // the pixels by which drawing this tile coarse is wrong: a dropped mark's whole size,
            // a mark drawn on a backend's cheaper mesh its outline's shortfall ([`COARSE_SHORTFALL`])
            let error_px = t
                .marks
                .iter()
                .filter(|m| !m.pin)
                .map(|m| {
                    let at = [t.origin[0] + m.pos[0] as f64, t.origin[1] + m.pos[1] as f64, t.origin[2] + m.pos[2] as f64];
                    if m.detail > 0 && m.tag == 0 {
                        cam.pixels(at, m.bound() as f64)
                    } else {
                        // a cheap mesh falls short across the mark's girth: a sphere's radius,
                        // a rod's radius (not its length)
                        cam.pixels(at, m.size[0].abs() as f64) * coarse_shortfall(m.kind)
                    }
                })
                .fold(0.0, f64::max);
            cands.push(Candidate { distance: (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt(), work, read, error_px });
            live.push((t, d));
        }
        let fine = alloc.allocate(&cands, budget);
        let remainder_px = Acuity::remainder(&cands, &fine);
        let mut frame = Frame { batches: Vec::new(), marks: Vec::new(), covers: Vec::new(), width: cam.width, height: cam.height, fine: 0, coarse: 0, stale, modal: self.modal, remainder_px };
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
            // one batch per kind; images one batch per texture slot, in the order the slots first appear
            let mut groups: Vec<(Kind, Option<u8>)> = Vec::new();
            for kind in KINDS {
                if kind == Kind::Image {
                    let mut slots: Vec<u8> = Vec::new();
                    for m in t.marks.iter().filter(|m| m.kind == Kind::Image) {
                        if !slots.contains(&m.variant) {
                            slots.push(m.variant);
                        }
                    }
                    groups.extend(slots.into_iter().map(|s| (kind, Some(s))));
                } else {
                    groups.push((kind, None));
                }
            }
            for (kind, slot) in groups {
                let mut bytes = Vec::new();
                let mut count = 0u32;
                // a pinned mark is never coarsened: in a coarse tile it rides in a batch of its own
                let mut pinned = Vec::new();
                let mut n_pinned = 0u32;
                for m in t.marks.iter().filter(|m| m.kind == kind && slot.map_or(true, |s| m.variant == s)) {
                    if !is_fine && !t.screen && m.pin {
                        let at = pinned.len();
                        m.pack_as(&mut pinned, false);
                        n_pinned += 1;
                        if m.tag != 0 {
                            let shown = fnv(0xcbf2_9ce4_8422_2325, &pinned[at..]);
                            let world = [t.origin[0] + m.pos[0] as f64, t.origin[1] + m.pos[1] as f64, t.origin[2] + m.pos[2] as f64];
                            let r = m.bound() as f64 * cam.scale;
                            let (x, y, depth, on) = match cam.project(world) {
                                Some(p) => (p[0], p[1], p[2], p[0] + r >= 0.0 && p[0] - r <= cam.width && p[1] + r >= 0.0 && p[1] - r <= cam.height),
                                None => (f64::NAN, f64::NAN, f64::INFINITY, false),
                            };
                            frame.marks.push(Mark { tag: m.tag, kind, layer: t.layer, coarse: false, on_screen: on, x, y, r, depth, legible: on && 2.0 * r >= LEGIBLE_PX, shown, rect: None, cover: u32::MAX, bind: m.bind });
                        }
                        continue;
                    }
                    let whole = is_fine || m.detail == 0 || m.pin;
                    if !whole && m.tag == 0 {
                        continue;
                    }
                    let at = bytes.len();
                    m.pack_as(&mut bytes, !whole);
                    count += 1;
                    if t.screen {
                        let rect = screen_rect(m, t.origin);
                        let radius = if matches!(kind, Kind::Quad | Kind::Patch) { (m.aux[0].max(0.0) as f64).min(0.5 * (rect[2] - rect[0]).min(rect[3] - rect[1])) } else { 0.0 };
                        let cover = frame.covers.len() as u32;
                        frame.covers.push(Cover { tag: m.tag, kind, rect, radius, ink: screen_ink(m, &t.params) });
                        if m.tag != 0 {
                            let shown = fnv(0xcbf2_9ce4_8422_2325, &bytes[at..]);
                            let (hw, hh) = (0.5 * (rect[2] - rect[0]), 0.5 * (rect[3] - rect[1]));
                            let on = rect[2] >= 0.0 && rect[0] <= cam.width && rect[3] >= 0.0 && rect[1] <= cam.height;
                            frame.marks.push(Mark { tag: m.tag, kind, layer: t.layer, coarse: false, on_screen: on, x: rect[0] + hw, y: rect[1] + hh, r: hw.max(hh), depth: f64::NEG_INFINITY, legible: on && hw.min(hh) >= 0.5 * LEGIBLE_PX, shown, rect: Some(rect), cover, bind: m.bind });
                        }
                    } else if m.tag != 0 {
                        let shown = fnv(0xcbf2_9ce4_8422_2325, &bytes[at..]);
                        let world = [t.origin[0] + m.pos[0] as f64, t.origin[1] + m.pos[1] as f64, t.origin[2] + m.pos[2] as f64];
                        let r = m.bound() as f64 * cam.scale;
                        let (x, y, depth, on) = match cam.project(world) {
                            Some(p) => (p[0], p[1], p[2], p[0] + r >= 0.0 && p[0] - r <= cam.width && p[1] + r >= 0.0 && p[1] - r <= cam.height),
                            None => (f64::NAN, f64::NAN, f64::INFINITY, false),
                        };
                        frame.marks.push(Mark { tag: m.tag, kind, layer: t.layer, coarse: !whole, on_screen: on, x, y, r, depth, legible: on && 2.0 * r >= LEGIBLE_PX, shown, rect: None, cover: u32::MAX, bind: m.bind });
                    }
                }
                if count > 0 {
                    frame.batches.push(Batch { tile: t.id, offset, kind, count, bytes, screen: t.screen, params: t.params, coarse: !t.screen && !is_fine });
                }
                if n_pinned > 0 {
                    frame.batches.push(Batch { tile: t.id, offset, kind, count: n_pinned, bytes: pinned, screen: false, params: t.params, coarse: false });
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

    /// Whether a mark of instance `tag` is drawn over by a mark of another instance.
    fn unoccluded(&self, m: &Mark) -> Tri {
        let box_ = m.rect.unwrap_or([m.x - m.r, m.y - m.r, m.x + m.r, m.y + m.r]);
        let after = if m.cover == u32::MAX { 0 } else { m.cover as usize + 1 };
        let hit = self.covers[after..].iter().any(|c| c.tag != m.tag && c.ink[3] > 0.0 && overlaps(&c.rect, &box_));
        match (hit, m.rect.is_some()) {
            (true, _) => Tri::No,
            (false, true) => Tri::Yes,
            // world against world needs a depth buffer
            (false, false) => Tri::Unknown,
        }
    }

    /// A run's contrast against the ground under it, or `None` when that ground is not one
    /// the core can see whole and opaque.
    fn contrast(&self, m: &Mark) -> Option<f64> {
        let r = m.rect?;
        if m.cover == u32::MAX {
            return None;
        }
        let own = self.covers[m.cover as usize];
        // the covers under it, nearest first, until one is opaque
        let mut layers: Vec<[f64; 4]> = Vec::new();
        for c in self.covers[..m.cover as usize].iter().rev() {
            if c.ink[3] <= 0.0 || !overlaps(&c.rect, &r) {
                continue;
            }
            // a quad (or patch) whose rounded rectangle holds the run is a uniform layer (it is
            // convex, so holding the run's four corners holds the run); anything else under any
            // part of it makes the ground unknown
            let holds = [[r[0], r[1]], [r[2], r[1]], [r[0], r[3]], [r[2], r[3]]].iter().all(|p| in_rounded(&c.rect, c.radius, *p));
            if !matches!(c.kind, Kind::Quad | Kind::Patch) || !holds {
                return None;
            }
            layers.push(c.ink);
            if c.ink[3] >= 1.0 {
                break;
            }
        }
        let base = layers.pop().filter(|l| l[3] >= 1.0)?;
        let mut g = [base[0], base[1], base[2]];
        for l in layers.iter().rev() {
            for k in 0..3 {
                g[k] = l[3] * l[k] + (1.0 - l[3]) * g[k];
            }
        }
        let a = own.ink[3];
        let ink = [a * own.ink[0] + (1.0 - a) * g[0], a * own.ink[1] + (1.0 - a) * g[1], a * own.ink[2] + (1.0 - a) * g[2]];
        Some(contrast_ratio(ink, g))
    }

    /// What is said of one instance, or `None` if no mark of this frame carries its tag.
    pub fn report(&self, tag: u32) -> Option<Shown> {
        let mut out: Option<Shown> = None;
        let mut widest = 0.0f64;
        let (mut inside, mut unoccluded, mut contrast, mut untruncated, mut string) = (true, Tri::Yes, Tri::Yes, Tri::Yes, Tri::Yes);
        let mut min_contrast = f64::INFINITY;
        let mut source: Option<[u8; 32]> = None;
        for m in self.marks.iter().filter(|m| m.tag == tag) {
            let b = m.rect.unwrap_or([m.x - m.r, m.y - m.r, m.x + m.r, m.y + m.r]);
            // the narrow side of a screen mark (a run's height), a world mark's diameter
            widest = widest.max(match m.rect {
                Some(r) => (r[2] - r[0]).min(r[3] - r[1]),
                None => 2.0 * m.r,
            });
            inside &= m.on_screen && b[0] >= 0.0 && b[1] >= 0.0 && b[2] <= self.width && b[3] <= self.height;
            unoccluded = unoccluded.and(self.unoccluded(m));
            if m.kind == Kind::Run {
                match self.contrast(m) {
                    Some(c) => {
                        min_contrast = min_contrast.min(c);
                        contrast = contrast.and(if c >= CONTRAST_MIN { Tri::Yes } else { Tri::No });
                    }
                    None => {
                        min_contrast = f64::NAN;
                        contrast = contrast.and(Tri::Unknown);
                    }
                }
                match m.bind {
                    Bind::Source { sha256, drawn } => {
                        untruncated = untruncated.and(match drawn {
                            Drawn::Exact => Tri::Yes,
                            Drawn::Unsaid => Tri::Unknown,
                            Drawn::Cut | Drawn::Changed => Tri::No,
                        });
                        match source {
                            None => source = Some(sha256),
                            Some(s) if s != sha256 => string = string.and(Tri::No),
                            _ => {}
                        }
                    }
                    _ => {
                        untruncated = untruncated.and(Tri::Unknown);
                        string = string.and(Tri::Unknown);
                    }
                }
            }
            out = Some(match out {
                None => Shown {
                    tag,
                    marks: 1,
                    on_screen: m.on_screen,
                    legible: false,
                    why: Legibility { inside: false, size: false, unoccluded: Tri::Unknown, contrast: Tri::Unknown, min_contrast: f64::NAN, untruncated: Tri::Unknown, string: Tri::Unknown },
                    coarse: m.coarse,
                    bounds: b,
                    shown: fnv(0xcbf2_9ce4_8422_2325, &m.shown.to_le_bytes()),
                    source: None,
                },
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
        out.map(|s| {
            let why = Legibility { inside, size: widest >= LEGIBLE_PX, unoccluded, contrast, min_contrast, untruncated, string };
            let legible = s.on_screen && why.inside && why.size && [why.unoccluded, why.contrast, why.untruncated, why.string].iter().all(|t| *t == Tri::Yes);
            Shown { legible, why, source: if string == Tri::Yes { source } else { None }, ..s }
        })
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
    #[test]
    fn an_arc_bound_holds_its_bend() {
        // A chord of 4 raised 3: the top of the curve is 1.5 above the midpoint, about 2.5 from pos.
        let mut a = Instance::new(Kind::Arc, [0.0, 0.0, 0.0], [0.1, 3.0, 0.0]);
        a.aux = [4.0, 0.0, 0.0];
        let top = (2.0f32 * 2.0 + 1.5 * 1.5).sqrt();
        assert!(a.bound() >= top, "bound {} must hold the curve's top {}", a.bound(), top);
        a.size = [0.1, 0.0, 0.0];
        assert_eq!(a.bound(), 4.0, "an unbent arc is its chord");
        assert_eq!(VARIANT_MATERIAL & (VARIANT_ORIENT | COLOUR_RAMP | COLOUR_TINT), 0, "material bits are their own");
        assert_eq!(MATERIAL_GLOW, VARIANT_MATERIAL);
    }

    #[test]
    fn an_oriented_box_keeps_its_bound() {
        let upright = Instance::new(Kind::Box, [1.0, 2.0, 0.0], [0.4, 0.3, 2.0]);
        let mut tipped = upright;
        tipped.aux = [1.0, 0.0, 0.2];
        tipped.variant = VARIANT_ORIENT;
        assert_eq!(upright.bound(), tipped.bound());
        assert_eq!(VARIANT_ORIENT & (VARIANT_CASING | VARIANT_ROUND | COLOUR_RAMP | COLOUR_TINT), 0, "a bit of its own");
    }

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
        assert!(r.marks == 2 && r.on_screen && r.why.size && !r.coarse);
        // a world mark is never legible enough to license a send: world against world is the depth buffer's
        assert!(!r.legible && r.why.unoccluded == Tri::Unknown);
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
        button.rgba = [0x23, 0x69, 0x57, 255];
        // the label is part of the button's instance, with its source string
        let label = Instance::new(Kind::Run, [-50.0, 0.0, 0.0], [52.0, 14.0, 0.0]).tagged(7).bound_to(Bind::Source { sha256: digest::sha256(b"confirm"), drawn: Drawn::Exact });
        s.tiles.push(Tile { id: 50, origin: [300.0, 420.0, 0.0], version: 7, layer: 0, screen: true, params: PARAMS_NEUTRAL, marks: vec![Instance::new(Kind::Quad, [0.0; 3], [200.0, 40.0, 0.0]), button, label] });
        let f = s.frame(&cam(), 7, &Ring, 1000).unwrap();
        assert_eq!(f.pick(260.0, 425.0), Some(7), "the button, inside its rectangle");
        assert_eq!(f.pick(212.0, 425.0), Some(7));
        assert_eq!(f.pick(205.0, 425.0), None, "outside the rectangle, on the untagged panel");
        // the building under the panel's right half is still picked: the panel is decoration
        assert_eq!(f.pick(336.0, 428.0), Some(113));
        let r = f.report(7).unwrap();
        assert_eq!(r.bounds, [210.0, 408.0, 290.0, 432.0]);
        assert!(r.legible, "{:?}", r.why);
        assert_eq!(r.source, Some(digest::sha256(b"confirm")));
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
    fn under_acuity_the_frame_spends_only_where_a_pixel_would_change_and_says_what_is_left() {
        // two tiles of buildings, one near the focus and one far; trees are 2 units across
        let s = scene(vec![tile(1, 0.0, 40, 100), tile(2, 5000.0, 40, 200)]);
        let c = cam(); // a map at 4 pixels a unit: everything is 4 px a unit, near or far
        let f = s.frame(&c, 7, &Acuity { eps_px: 1.0 }, u64::MAX).unwrap();
        assert_eq!((f.fine, f.coarse), (2, 0), "detail spans more than a pixel in both");
        assert_eq!(f.remainder_px, 0.0);
        // zoomed out to 0.1 px a unit, nothing detailed spans a pixel: all coarse, at no cost
        let far = Camera::map([50.0, 50.0, 0.0], 0.1, 800.0, 600.0);
        let f = s.frame(&far, 7, &Acuity { eps_px: 1.0 }, u64::MAX).unwrap();
        assert_eq!((f.fine, f.coarse), (0, 2));
        assert!(f.remainder_px <= 1.0, "{}", f.remainder_px);
        assert_eq!(f.marks.len(), 80, "the tagged buildings are still drawn, coarse");
        // the same zoom on a window twice the size: the same choices for the same tiles
        let big = Camera::map([50.0, 50.0, 0.0], 0.1, 1600.0, 1200.0);
        let g = s.frame(&big, 7, &Acuity { eps_px: 1.0 }, u64::MAX).unwrap();
        assert_eq!((g.fine, g.coarse, g.remainder_px), (f.fine, f.coarse, f.remainder_px));
    }

    #[test]
    fn a_kept_sphere_costs_only_its_meshes_shortfall_and_a_pinned_one_is_never_coarse() {
        // one tile of spheres 3 units across, all kept (detail 0), one pinned
        let mut marks: Vec<Instance> = (0..20).map(|i| Instance::new(Kind::Sphere, [i as f32 * 4.0, 0.0, 0.0], [1.5, 0.0, 0.0]).tagged(1 + i)).collect();
        marks[3] = marks[3].pinned();
        let s = scene(vec![Tile { id: 1, origin: [0.0; 3], version: 7, layer: 0, screen: false, params: PARAMS_NEUTRAL, marks }]);
        // at 4 px a unit a sphere is 12 px across; its cheap outline is 4.9 % short: 0.59 px
        let c = Camera::map([40.0, 0.0, 0.0], 4.0, 800.0, 600.0);
        let f = s.frame(&c, 7, &Acuity { eps_px: 1.0 }, u64::MAX).unwrap();
        assert_eq!(f.coarse, 1, "within a pixel: coarse");
        assert!(f.remainder_px > 0.5 && f.remainder_px < 1.0, "{}", f.remainder_px);
        let coarse: u32 = f.batches.iter().filter(|b| b.coarse).map(|b| b.count).sum();
        let fine: u32 = f.batches.iter().filter(|b| !b.coarse).map(|b| b.count).sum();
        assert_eq!((coarse, fine), (19, 1), "the pinned sphere rides alone, fine");
        assert!(!f.report(4).unwrap().coarse);
        // at 40 px a unit the shortfall is 5.9 px: fine
        let near = Camera::map([40.0, 0.0, 0.0], 40.0, 800.0, 600.0);
        let g = s.frame(&near, 7, &Acuity { eps_px: 1.0 }, u64::MAX).unwrap();
        assert_eq!((g.fine, g.coarse), (1, 0));
        assert!(g.batches.iter().all(|b| !b.coarse));
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

    // ---- L12 legibility and L10 media (issue #6) ----

    const WHITE: [u8; 4] = [255, 255, 255, 255];
    const BLACK: [u8; 4] = [0, 0, 0, 255];

    fn src(s: &str) -> Bind {
        Bind::Source { sha256: digest::sha256(s.as_bytes()), drawn: Drawn::Exact }
    }
    fn screen(id: u64, layer: u8, marks: Vec<Instance>) -> Tile {
        Tile { id, origin: [0.0; 3], version: 7, layer, screen: true, params: PARAMS_NEUTRAL, marks }
    }
    /// A white card 300 by 100 at (50, 50) and a value run (tag 21) 120 by 16 on it.
    fn card(run: Instance) -> Tile {
        screen(1, 0, vec![Instance::new(Kind::Quad, [200.0, 100.0, 0.0], [300.0, 100.0, 0.0]).colour(WHITE).tagged(20), run])
    }
    fn value() -> Instance {
        Instance::new(Kind::Run, [150.0, 100.0, 0.0], [120.0, 16.0, 0.0]).colour(BLACK).tagged(21).bound_to(src("42 tokens"))
    }
    fn shown(tiles: Vec<Tile>, tag: u32) -> Shown {
        Scene { tiles, modal: None }.frame(&cam(), 7, &Ring, u64::MAX).expect("drawn").report(tag).expect("reported")
    }

    #[test]
    fn a_value_on_an_opaque_card_is_legible_and_reports_its_source() {
        let s = shown(vec![card(value())], 21);
        assert!(s.legible, "{:?}", s.why);
        assert_eq!(s.source, Some(digest::sha256(b"42 tokens")));
        assert!((s.why.min_contrast - 21.0).abs() < 1e-9, "black on white is 21:1");
        // the card itself is under its own value: occluded by another instance, so not legible
        assert_eq!(shown(vec![card(value())], 20).why.unoccluded, Tri::No);
    }

    #[test]
    fn occlusion_by_a_later_mark_of_another_instance() {
        // a sheet in a higher layer over half the value
        let sheet = screen(2, 1, vec![Instance::new(Kind::Quad, [200.0, 100.0, 0.0], [20.0, 8.0, 0.0]).colour(WHITE).tagged(30)]);
        let s = shown(vec![card(value()), sheet.clone()], 21);
        assert_eq!(s.why.unoccluded, Tri::No);
        assert!(!s.legible);
        // the same sheet moved clear of it, and an invisible one over it, occlude nothing
        let mut clear = sheet.clone();
        clear.marks[0].pos = [400.0, 400.0, 0.0];
        assert_eq!(shown(vec![card(value()), clear], 21).why.unoccluded, Tri::Yes);
        let mut ghost = sheet;
        ghost.marks[0].rgba[3] = 0;
        assert_eq!(shown(vec![card(value()), ghost], 21).why.unoccluded, Tri::Yes);
        // a mark of the SAME instance drawn after it (a glyph beside a label) does not occlude it
        let mut t = card(value());
        t.marks.push(Instance::new(Kind::Glyph, [150.0, 100.0, 0.0], [5.0, 0.0, 0.0]).colour(BLACK).tagged(21));
        assert_eq!(shown(vec![t], 21).why.unoccluded, Tri::Yes);
        // a world mark under a panel is occluded; one clear of every panel is unknown, not yes
        let mut s = scene(vec![tile(1, 0.0, 40, 100)]);
        s.tiles.push(screen(9, 0, vec![Instance::new(Kind::Quad, [336.0, 428.0, 0.0], [30.0, 30.0, 0.0]).colour(WHITE)]));
        let f = s.frame(&cam(), 7, &Ring, u64::MAX).unwrap();
        assert_eq!(f.report(113).unwrap().why.unoccluded, Tri::No);
        assert_eq!(f.report(100).unwrap().why.unoccluded, Tri::Unknown);
    }

    #[test]
    fn contrast_is_computed_against_a_ground_the_core_can_see_whole() {
        // grey on white: about 1.6:1, under the floor
        let mut grey = value();
        grey.rgba = [200, 200, 200, 255];
        let s = shown(vec![card(grey)], 21);
        assert_eq!(s.why.contrast, Tri::No);
        assert!(s.why.min_contrast < CONTRAST_MIN && s.why.min_contrast > 1.0);
        // a translucent ground lets the world show through: unknown
        let mut t = card(value());
        t.marks[0].rgba[3] = 217;
        assert_eq!(shown(vec![t], 21).why.contrast, Tri::Unknown);
        // a translucent quad over an opaque one composes: black at 50% over white is mid grey
        let mut t = card(value());
        t.marks.insert(1, Instance::new(Kind::Quad, [150.0, 100.0, 0.0], [200.0, 60.0, 0.0]).colour([0, 0, 0, 128]));
        let s = shown(vec![t], 21);
        let g = 1.0 - 128.0 / 255.0;
        assert!((s.why.min_contrast - contrast_ratio([0.0; 3], [g; 3])).abs() < 1e-9, "{}", s.why.min_contrast);
        // a run that spills past its card has a ground the core cannot see whole
        let mut wide = value();
        wide.size[0] = 400.0;
        assert_eq!(shown(vec![card(wide)], 21).why.contrast, Tri::Unknown);
        // a run in the card's rounded corner is not on the card there
        let mut corner = card(value());
        corner.marks[0].aux[0] = 30.0;
        corner.marks[1].pos = [111.0, 59.0, 0.0];
        corner.marks[1].size = [120.0, 16.0, 0.0];
        assert_eq!(shown(vec![corner], 21).why.contrast, Tri::Unknown);
        // the tile's DIM darkens the card: white at a fifth is dark grey, and black on it fails
        let mut dim = card(value());
        dim.params[0] = 0.2;
        dim.params[1] = 0.2;
        dim.params[2] = 0.2;
        assert_eq!(shown(vec![dim], 21).why.contrast, Tri::No);
    }

    #[test]
    fn truncation_the_string_and_the_viewport() {
        let with = |drawn: Drawn| {
            let mut v = value();
            v.bind = Bind::Source { sha256: digest::sha256(b"42 tokens"), drawn };
            shown(vec![card(v)], 21)
        };
        assert_eq!(with(Drawn::Exact).why.untruncated, Tri::Yes);
        assert_eq!(with(Drawn::Cut).why.untruncated, Tri::No);
        assert_eq!(with(Drawn::Changed).why.untruncated, Tri::No);
        assert_eq!(with(Drawn::Unsaid).why.untruncated, Tri::Unknown);
        assert!(!with(Drawn::Unsaid).legible);
        // a run with no source: neither its string nor its drawing is known
        let mut bare = value();
        bare.bind = Bind::None;
        let s = shown(vec![card(bare)], 21);
        assert_eq!((s.why.string, s.why.untruncated, s.source), (Tri::Unknown, Tri::Unknown, None));
        // two runs of one instance from two strings
        let mut t = card(value());
        t.marks.push(Instance::new(Kind::Run, [150.0, 120.0, 0.0], [60.0, 16.0, 0.0]).colour(BLACK).tagged(21).bound_to(src("43 tokens")));
        let s = shown(vec![t], 21);
        assert_eq!((s.why.string, s.source, s.legible), (Tri::No, None, false));
        // a value partly off the viewport touches it but is not inside it
        let mut t = card(value());
        t.marks[0].pos[0] = 60.0;
        t.marks[1].pos[0] = 10.0;
        let s = shown(vec![t], 21);
        assert!(s.on_screen && !s.why.inside && !s.legible);
    }

    #[test]
    fn a_screen_line_is_reported_by_its_two_ends() {
        let mut l = Instance::new(Kind::Line, [10.0, 20.0, 0.0], [4.0, 0.0, 0.0]).tagged(5);
        l.aux = [110.0, 20.0, 0.0];
        let f = Scene { tiles: vec![screen(1, 0, vec![l])], modal: None }.frame(&cam(), 7, &Ring, u64::MAX).unwrap();
        assert_eq!(f.report(5).unwrap().bounds, [8.0, 18.0, 112.0, 22.0]);
        assert_eq!(f.pick(100.0, 20.0), Some(5), "its far end is on it");
    }

    fn image(tag: u32, slot: u8, bind: Bind) -> Instance {
        let mut m = Instance::new(Kind::Image, [200.0, 300.0, 0.0], [120.0, 90.0, 0.0]).tagged(tag).bound_to(bind);
        m.aux = [0.0, 0.0, 1.0];
        m.angle = 1.0;
        m.variant = slot;
        m
    }
    fn label(tag: u32, s: &str) -> Instance {
        Instance::new(Kind::Run, [200.0, 360.0, 0.0], [100.0, 14.0, 0.0]).colour(BLACK).tagged(tag).bound_to(src(s))
    }
    fn draw(marks: Vec<Instance>) -> Result<Frame, Refusal> {
        Scene { tiles: vec![screen(4, 0, marks)], modal: None }.frame(&cam(), 7, &Ring, u64::MAX)
    }

    #[test]
    fn media_is_never_drawn_bare() {
        let media = Bind::Media { alt: 41, disclosure: 42 };
        assert_eq!(draw(vec![image(40, 0, Bind::None)]).unwrap_err(), Refusal::BareImage { tile: 4, index: 0 });
        assert_eq!(draw(vec![image(40, 0, media), label(41, "a heron on a post")]).unwrap_err(), Refusal::MediaUnbound { tile: 4, index: 0, missing: 42 });
        // a disclosure drawn without a source string does not count
        let mut d = label(42, "made with a generator");
        d.bind = Bind::None;
        assert_eq!(draw(vec![image(40, 0, media), label(41, "a heron on a post"), d]).unwrap_err(), Refusal::MediaUnbound { tile: 4, index: 0, missing: 42 });
        // alt and disclosure drawn: the image is drawn, in its own batch
        let f = draw(vec![image(40, 0, media), label(41, "a heron on a post"), label(42, "not generated")]).expect("media with both slots");
        assert_eq!(f.batches.iter().filter(|b| b.kind == Kind::Image).count(), 1);
        assert!(f.report(40).is_some());
        // a QR grid and a decoration need neither
        assert!(draw(vec![image(50, 1, Bind::Qr)]).is_ok());
        assert!(draw(vec![image(0, 1, Bind::None)]).is_ok());
        // an image in a world tile is refused: no backend draws one there
        let mut w = scene(vec![tile(1, 0.0, 1, 100)]);
        w.tiles[0].marks.push(image(0, 0, Bind::None));
        assert_eq!(w.frame(&cam(), 7, &Ring, u64::MAX).unwrap_err(), Refusal::ImageInWorld { tile: 1, index: 3 });
    }

    #[test]
    fn images_are_batched_by_texture_slot_in_the_order_the_slots_appear() {
        let f = draw(vec![image(0, 3, Bind::None), image(0, 1, Bind::None), image(0, 3, Bind::None)]).unwrap();
        let imgs: Vec<(u8, u32)> = f.batches.iter().filter(|b| b.kind == Kind::Image).map(|b| (b.bytes[51], b.count)).collect();
        assert_eq!(imgs, [(3, 2), (1, 1)]);
    }
}
