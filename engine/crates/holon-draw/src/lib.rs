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
//! * **Two marks with one tag in a frame** ([`Refusal::DuplicateTag`]): a pick must name
//!   exactly one instance (the screen standard's L1). The frame is refused whole.
//! * **Dropping a pinned mark.** A pinned mark is drawn at every allocation (L12: a value a
//!   write can be sent from is on screen when it is sent).
//! * **A non-finite field** ([`Refusal::NotFinite`]): a mark whose number is not a number
//!   is not drawn as zero (L4).
//!
//! # What it does not do
//!
//! It holds no strings and shapes no text, has no clock and no randomness, and knows no
//! state of a read: a host gives it a scene only when there is one to draw. Level of detail
//! changes how finely a tile is drawn, never what a tagged mark means (L8): a tagged mark
//! that a coarse tile does not draw is REPORTED as not drawn, so its host can say so.
//!
//! The pick in this slice treats each mark as its bounding sphere projected to the screen.
//! That is exact for discs and spheres and an over-estimate for the rest; a per-kind test is
//! owed and the over-estimate is why the nearest centre wins a tie.

pub mod alloc;

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
}

pub const KINDS: [Kind; 10] = [Kind::Quad, Kind::Line, Kind::Arc, Kind::Disc, Kind::Box, Kind::Prism, Kind::Sphere, Kind::Rod, Kind::Patch, Kind::Glyph];

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
    /// 0: drawn at every allocation. 1 and above: drawn only when its tile is fine.
    pub detail: u8,
    /// Never elided or coarsened.
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
        for v in self.pos.iter().chain(&self.size).chain(&self.aux).chain(std::iter::once(&self.angle)) {
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend_from_slice(&self.rgba);
        out.extend_from_slice(&self.tag.to_le_bytes());
        out.extend_from_slice(&[self.kind as u8, self.detail, self.pin as u8, self.variant]);
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
    pub marks: Vec<Instance>,
}

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
    /// Two marks in one frame carry this tag.
    DuplicateTag(u32),
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
}

/// What is said of one tagged mark.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mark {
    pub tag: u32,
    pub drawn: bool,
    pub on_screen: bool,
    /// Centre in pixels, and radius in pixels.
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub depth: f64,
    pub legible: bool,
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
}

/// A mark under this many pixels across is drawn but not legible.
pub const LEGIBLE_PX: f64 = 4.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub tiles: Vec<Tile>,
}

impl Scene {
    /// Build a frame. `version` is the version of the inputs the host holds now; `budget` is
    /// the number of marks the allocator may spend on detail.
    pub fn frame(&self, cam: &Camera, version: u64, alloc: &dyn Allocator, budget: u64) -> Result<Frame, Refusal> {
        let mut seen = std::collections::HashSet::new();
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
                if m.tag != 0 && !seen.insert(m.tag) {
                    return Err(Refusal::DuplicateTag(m.tag));
                }
            }
            let d = [t.origin[0] - cam.focus[0], t.origin[1] - cam.focus[1], t.origin[2] - cam.focus[2]];
            let work = t.marks.iter().filter(|m| m.detail > 0 && !m.pin).count() as u64;
            let read = t.marks.iter().filter(|m| m.detail > 0 && !m.pin && m.tag != 0).count() as u64;
            cands.push(Candidate { distance: (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt(), work, read });
            live.push((t, d));
        }
        let fine = alloc.allocate(&cands, budget);
        let mut frame = Frame { batches: Vec::new(), marks: Vec::new(), fine: 0, coarse: 0, stale };
        for (k, (t, d)) in live.iter().enumerate() {
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
                    let drawn = is_fine || m.detail == 0 || m.pin;
                    if drawn {
                        m.pack(&mut bytes);
                        count += 1;
                    }
                    if m.tag != 0 {
                        let world = [t.origin[0] + m.pos[0] as f64, t.origin[1] + m.pos[1] as f64, t.origin[2] + m.pos[2] as f64];
                        let r = m.bound() as f64 * cam.scale;
                        let (x, y, depth, on) = match cam.project(world) {
                            Some(p) => (p[0], p[1], p[2], p[0] + r >= 0.0 && p[0] - r <= cam.width && p[1] + r >= 0.0 && p[1] - r <= cam.height),
                            None => (f64::NAN, f64::NAN, f64::INFINITY, false),
                        };
                        frame.marks.push(Mark { tag: m.tag, drawn, on_screen: drawn && on, x, y, r, depth, legible: drawn && on && 2.0 * r >= LEGIBLE_PX });
                    }
                }
                if count > 0 {
                    frame.batches.push(Batch { tile: t.id, offset, kind, count, bytes });
                }
            }
        }
        Ok(frame)
    }
}

impl Frame {
    /// The tagged mark under a point in pixels: the nearest in depth among those whose
    /// circle holds the point, the nearest centre on a tie. Pure.
    pub fn pick(&self, x: f64, y: f64) -> Option<u32> {
        let mut best: Option<(&Mark, f64)> = None;
        for m in self.marks.iter().filter(|m| m.on_screen) {
            let d2 = (m.x - x) * (m.x - x) + (m.y - y) * (m.y - y);
            let r = m.r.max(LEGIBLE_PX);
            if d2 > r * r {
                continue;
            }
            let better = match best {
                None => true,
                Some((b, bd2)) => m.depth < b.depth || (m.depth == b.depth && (d2 < bd2 || (d2 == bd2 && m.tag < b.tag))),
            };
            if better {
                best = Some((m, d2));
            }
        }
        best.map(|(m, _)| m.tag)
    }
    /// What is said of one tag, or `None` if no mark of this frame carries it.
    pub fn report(&self, tag: u32) -> Option<Mark> {
        self.marks.iter().find(|m| m.tag == tag).copied()
    }
    /// A fold of every batch's tile, kind, count and bytes (FNV-1a, 64-bit), in order.
    pub fn digest(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        let mut eat = |b: &[u8]| {
            for &x in b {
                h ^= x as u64;
                h = h.wrapping_mul(0x0000_0100_0000_01B3);
            }
        };
        for b in &self.batches {
            eat(&b.tile.to_le_bytes());
            eat(&[b.kind as u8]);
            eat(&b.count.to_le_bytes());
            eat(&b.bytes);
        }
        h
    }
    /// The instances drawn.
    pub fn drawn(&self) -> u64 {
        self.batches.iter().map(|b| b.count as u64).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile(id: u64, ox: f64, n: u32, tag0: u32) -> Tile {
        let mut marks = vec![Instance::new(Kind::Patch, [0.0; 3], [100.0, 100.0, 0.0])];
        for i in 0..n {
            let p = [10.0 + 8.0 * (i % 10) as f32, 10.0 + 8.0 * (i / 10) as f32, 0.0];
            marks.push(Instance::new(Kind::Prism, p, [3.0, 3.0, 6.0]).tagged(tag0 + i).detail(1));
        }
        Tile { id, origin: [ox, 0.0, 0.0], version: 7, marks }
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
        let s = Scene { tiles: vec![tile(1, 0.0, 40, 100), tile(2, 100.0, 40, 200)] };
        let (a, c) = (s.frame(&cam(), 7, &Ring, 1000).unwrap(), s.frame(&cam(), 7, &Ring, 1000).unwrap());
        assert_eq!(a, c);
        assert_eq!(a.digest(), c.digest());
        assert_eq!(a.drawn(), 82);
        assert!(a.batches.iter().all(|x| x.bytes.len() == x.count as usize * INSTANCE_BYTES));
        assert_eq!(a.batches.len(), 4, "one batch per tile and kind");
    }

    #[test]
    fn a_duplicate_tag_or_a_number_that_is_not_one_refuses_the_frame() {
        let mut s = Scene { tiles: vec![tile(1, 0.0, 5, 100), tile(2, 100.0, 5, 104)] };
        assert_eq!(s.frame(&cam(), 7, &Ring, 1000), Err(Refusal::DuplicateTag(104)));
        s.tiles[1] = tile(2, 100.0, 5, 200);
        s.tiles[0].marks[3].size[1] = f32::NAN;
        assert_eq!(s.frame(&cam(), 7, &Ring, 1000), Err(Refusal::NotFinite { tile: 1, index: 3 }));
    }

    #[test]
    fn a_pick_names_exactly_one_mark_and_decoration_is_never_picked() {
        let s = Scene { tiles: vec![tile(1, 0.0, 40, 100)] };
        let c = cam();
        let f = s.frame(&c, 7, &Ring, 1000).unwrap();
        // mark 100+13 sits at local (34, 18): on screen at ((34-50)*4+400, 300-(18-50)*4)
        assert_eq!(f.pick(336.0, 428.0), Some(113));
        assert_eq!(f.pick(336.0, 428.0), Some(113), "pure");
        // the ground patch is under every point and carries no tag
        assert_eq!(f.pick(700.0, 80.0), None);
        let m = f.report(113).unwrap();
        assert!(m.drawn && m.on_screen && m.legible);
        assert!(f.report(5).is_none());
    }

    #[test]
    fn detail_changes_how_finely_never_what_and_a_pinned_mark_is_always_drawn() {
        let mut s = Scene { tiles: vec![tile(1, 0.0, 40, 100), tile(2, 5000.0, 40, 200)] };
        s.tiles[1].marks[4] = s.tiles[1].marks[4].pinned();
        let c = cam();
        let fine = s.frame(&c, 7, &Ring, 1000).unwrap();
        let tight = s.frame(&c, 7, &Ring, 40).unwrap();
        assert_eq!((fine.fine, fine.coarse), (2, 0));
        assert_eq!((tight.fine, tight.coarse), (1, 1), "the budget covers the near tile alone");
        assert_eq!(tight.drawn(), 41 + 1 + 1, "the far tile keeps its ground and its pinned mark");
        // every tag is still reported; the ones not drawn say so, and none is picked
        assert_eq!(tight.marks.len(), fine.marks.len());
        assert!(tight.report(203).unwrap().drawn, "pinned");
        let dropped = tight.report(210).unwrap();
        assert!(!dropped.drawn && !dropped.on_screen && !dropped.legible);
        // what IS drawn in both is byte-identical: the near tile's batches
        let near = |f: &Frame| f.batches.iter().filter(|b| b.tile == 1).cloned().collect::<Vec<_>>();
        assert_eq!(near(&fine), near(&tight));
    }

    #[test]
    fn a_tile_of_another_version_is_not_drawn() {
        let mut s = Scene { tiles: vec![tile(1, 0.0, 10, 100), tile(2, 100.0, 10, 200)] };
        s.tiles[1].version = 6;
        let f = s.frame(&cam(), 7, &Ring, 1000).unwrap();
        assert_eq!(f.stale, 1);
        assert!(f.report(205).is_none() && f.batches.iter().all(|b| b.tile == 1));
    }

    #[test]
    fn a_far_origin_keeps_its_precision() {
        // 20,000 km from the world's origin, two marks 1 unit apart: the offset the GPU adds
        // is taken from the focus in 64-bit, so they land 4 pixels apart at 4 pixels a unit
        let far = 2.0e7;
        let t = Tile { id: 1, origin: [far, far, 0.0], version: 1, marks: vec![
            Instance::new(Kind::Disc, [0.0, 0.0, 0.0], [0.4, 0.0, 0.0]).tagged(1),
            Instance::new(Kind::Disc, [1.0, 0.0, 0.0], [0.4, 0.0, 0.0]).tagged(2),
        ] };
        let c = Camera::map([far + 0.5, far, 0.0], 4.0, 800.0, 600.0);
        let f = Scene { tiles: vec![t] }.frame(&c, 1, &Ring, 10).unwrap();
        let (a, b) = (f.report(1).unwrap(), f.report(2).unwrap());
        assert!((b.x - a.x - 4.0).abs() < 1e-6 && (a.x - 398.0).abs() < 1e-6, "{} {}", a.x, b.x);
        assert_eq!(f.batches[0].offset, [-0.5, 0.0, 0.0]);
    }
}
