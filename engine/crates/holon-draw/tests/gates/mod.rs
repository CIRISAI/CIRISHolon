//! What the acuity gates share: the forced allocators, the per-tile bound, the three-view
//! gate, the plants, and the writer of the cases the browser probe draws.
//!
//! Used by `holon-draw/tests/acuity_gates.rs`, and (through `#[path]`) by
//! `holon-draw-ladder/tests/acuity_ladder.rs` and `holon-draw-ladder/examples/acuity_dump.rs`,
//! so that the Rust gates and the browser probe judge the SAME scenes. Nothing in `src/` was
//! changed for it: `Allocator` is a public trait and a tile framed alone gives its own bound.
//!
//! THE WORDS. The allocator's per-tile bound (`Candidate::error_px`, surfaced as
//! `Frame::remainder_px`) is an a-priori UPPER bound and is the certificate. Everything here
//! that compares a coarse frame with a fine one is a FALSIFIER: a sampled lower bound that can
//! kill the certificate and can never prove it.
#![allow(dead_code)]

use holon_draw::{coarse_shortfall, Allocator, Camera, Candidate, Frame, Instance, Kind, Scene, Tile, INSTANCE_BYTES, PARAMS_NEUTRAL};
use std::collections::{BTreeSet, HashMap};
use std::fmt::Write as _;

/// Every tile fine, whatever it costs. A gate's allocator, not a consumer's.
pub struct AllFine;
/// Every tile coarse that has anything to coarsen (the trait's contract: a tile with no
/// work is always fine, there being nothing to save).
pub struct AllCoarse;

impl Allocator for AllFine {
    fn allocate(&self, cands: &[Candidate], _budget: u64) -> Vec<bool> {
        vec![true; cands.len()]
    }
}
impl Allocator for AllCoarse {
    fn allocate(&self, cands: &[Candidate], _budget: u64) -> Vec<bool> {
        cands.iter().map(|c| c.work == 0).collect()
    }
}

pub fn frame(tiles: &[Tile], cam: &Camera, version: u64, alloc: &dyn Allocator) -> Frame {
    Scene { tiles: tiles.to_vec(), modal: None }.frame(cam, version, alloc, u64::MAX).expect("a finite scene")
}

/// A tile's own certificate at this camera: what `Scene::frame` prices it at if drawn coarse.
/// Read from the core itself (the tile framed alone, forced coarse), not recomputed here.
pub fn tile_bound(tile: &Tile, cam: &Camera) -> f64 {
    frame(std::slice::from_ref(tile), cam, tile.version, &AllCoarse).remainder_px
}

/// The tiles a frame drew coarse (a coarse tile's unpinned batches carry the flag).
pub fn coarse_tiles(f: &Frame) -> BTreeSet<u64> {
    f.batches.iter().filter(|b| b.coarse).map(|b| b.tile).collect()
}

// ------------------------------------------------------------------ the three-view gate

/// Where a coarse frame's PICK or REPORT differs from the fine frame's.
///
/// Compared: the pick at every given point; and for every tag either frame carries, whether
/// it is reported at all, its mark count, `on_screen`, `legible` and `bounds`. NOT compared:
/// `Shown::coarse` and `Shown::shown`, which state the allocation itself (the coarse flag is
/// in the packed bytes the digest folds) and so differ by design wherever a mark is coarsened.
pub fn views_disagree(fine: &Frame, coarse: &Frame, points: &[[f64; 2]]) -> Vec<String> {
    let mut out = Vec::new();
    for p in points {
        let (a, b) = (fine.pick(p[0], p[1]), coarse.pick(p[0], p[1]));
        if a != b {
            out.push(format!("pick at ({:.1}, {:.1}): fine {a:?}, coarse {b:?}", p[0], p[1]));
        }
    }
    let tags: BTreeSet<u32> = fine.marks.iter().chain(&coarse.marks).map(|m| m.tag).collect();
    for t in tags {
        match (fine.report(t), coarse.report(t)) {
            (Some(a), Some(b)) => {
                if (a.marks, a.on_screen, a.legible) != (b.marks, b.on_screen, b.legible) || a.bounds.map(f64::to_bits) != b.bounds.map(f64::to_bits) {
                    out.push(format!("report of tag {t}: fine {:?} {:?}, coarse {:?} {:?}", (a.marks, a.on_screen, a.legible), a.bounds, (b.marks, b.on_screen, b.legible), b.bounds));
                }
            }
            (a, b) => out.push(format!("report of tag {t}: fine {}, coarse {}", if a.is_some() { "present" } else { "absent" }, if b.is_some() { "present" } else { "absent" })),
        }
    }
    out
}

/// Points to pick at: a grid over the viewport, and for every tagged mark of either frame its
/// centre and four points at 0.7 of its radius (where a neighbour's circle may contest it).
pub fn pick_points(cam: &Camera, frames: &[&Frame], grid: usize) -> Vec<[f64; 2]> {
    let mut p = Vec::new();
    for i in 0..grid {
        for j in 0..grid {
            p.push([(i as f64 + 0.5) / grid as f64 * cam.width, (j as f64 + 0.5) / grid as f64 * cam.height]);
        }
    }
    for f in frames {
        for m in f.marks.iter().filter(|m| m.x.is_finite() && m.y.is_finite()) {
            let r = 0.7 * m.r.max(holon_draw::LEGIBLE_PX);
            p.extend([[m.x, m.y], [m.x + r, m.y], [m.x - r, m.y], [m.x, m.y + r], [m.x, m.y - r]]);
        }
    }
    p
}

/// What a backend can draw from: per world batch, its tile, kind, coarse flag, offset,
/// settings, and bytes 0..44 of every record (`web/draw.js`: "Only bytes 0..43 reach a
/// shader"; the tag and the flags are uploaded and never pointed at). Two frames equal here
/// are drawn pixel for pixel alike by any backend that reads no more than that.
#[allow(clippy::type_complexity)]
pub fn drawn_bytes(f: &Frame) -> Vec<(u64, u8, bool, [u32; 3], Vec<u32>, Vec<u8>)> {
    f.batches
        .iter()
        .map(|b| {
            let mut v = Vec::with_capacity(b.bytes.len());
            for r in b.bytes.chunks(INSTANCE_BYTES) {
                v.extend_from_slice(&r[..44]);
            }
            (b.tile, b.kind as u8, b.coarse, b.offset.map(f32::to_bits), b.params.iter().map(|x| x.to_bits()).collect(), v)
        })
        .collect()
}

// ------------------------------------------------------------------ the plants

/// PLANT (the fault the 2026-11-15 CORRECTION in `lib.rs` removed, put back): a coarse draw
/// that DROPS the tagged marks it should only have flagged. Applied to a coarse frame.
pub fn plant_drop_coarse_tagged(f: &Frame) -> Frame {
    let mut g = f.clone();
    for b in g.batches.iter_mut() {
        let mut kept = Vec::with_capacity(b.bytes.len());
        let mut n = 0u32;
        for r in b.bytes.chunks(INSTANCE_BYTES) {
            let tag = u32::from_le_bytes([r[44], r[45], r[46], r[47]]);
            if tag != 0 && r[50] & 2 != 0 {
                continue;
            }
            kept.extend_from_slice(r);
            n += 1;
        }
        b.bytes = kept;
        b.count = n;
    }
    g.batches.retain(|b| b.count > 0);
    g.marks.retain(|m| !m.coarse);
    g
}

/// PLANT: a coarse draw that hands two marks each other's tag (a cheaper path that lost which
/// record was whose). Bytes 44..47 of each record and the frame's own mark table.
pub fn plant_swap_tags(f: &Frame, a: u32, b: u32) -> Frame {
    let swap = |t: u32| if t == a { b } else if t == b { a } else { t };
    let mut g = f.clone();
    for batch in g.batches.iter_mut() {
        for r in batch.bytes.chunks_mut(INSTANCE_BYTES) {
            let t = swap(u32::from_le_bytes([r[44], r[45], r[46], r[47]]));
            r[44..48].copy_from_slice(&t.to_le_bytes());
        }
    }
    for m in g.marks.iter_mut() {
        m.tag = swap(m.tag);
    }
    g
}

pub fn tile(id: u64, marks: Vec<Instance>) -> Tile {
    Tile { id, origin: [0.0; 3], version: 1, layer: 0, screen: false, params: PARAMS_NEUTRAL, marks }
}

/// The scene of the zero-pixel plants, on a map of one pixel a unit, 400 by 300, centred on
/// the origin. Two things in it are invisible in pixels and visible to the other two views:
///
/// * tag 2: a small box (detail 1) wholly INSIDE the opaque box tagged 1. The depth test
///   hides every pixel of it, coarse or fine; the pick and the report still name it.
/// * tags 3 and 4: two discs of one colour and one size, side by side.
///
/// Nothing in the tile is a sphere or a rod, so the backend draws coarse and fine alike.
pub fn zero_pixel_scene() -> (Vec<Tile>, Camera) {
    let big = Instance::new(Kind::Box, [-90.0, 0.0, 0.0], [80.0, 80.0, 40.0]).tagged(1).colour([70, 130, 180, 255]);
    let small = Instance::new(Kind::Box, [-90.0, 0.0, 10.0], [20.0, 20.0, 10.0]).tagged(2).detail(1).colour([250, 200, 40, 255]);
    let d = |x: f32, tag: u32| Instance::new(Kind::Disc, [x, 0.0, 0.0], [30.0, 0.0, 0.0]).tagged(tag).colour([200, 90, 60, 255]);
    (vec![tile(1, vec![big, small, d(40.0, 3), d(120.0, 4)])], Camera::map([0.0; 3], 1.0, 400.0, 300.0))
}

// ------------------------------------------------------------------ where to look closely

/// A window of the view to magnify: centre and side in device pixels (origin top left).
#[derive(Clone, Copy, Debug)]
pub struct Probe {
    pub x: f64,
    pub y: f64,
    pub side: f64,
    /// This mark's own term of the tile's bound.
    pub bound_px: f64,
    pub kind: Kind,
}

/// The marks of a tile where the probe looks closely: of the spheres and rods the allocator
/// prices (unpinned, the close look wholly inside the viewport), the `most` with the largest
/// term of the bound, and the `most` with the largest term times the stretch a perspective
/// view gives a ball off its axis (1 / cos of the angle from the axis, along the line from the
/// view's centre; 1 on a map). A SAMPLE, aimed where the error should be largest: the tile's
/// true error is at least what is read there.
pub fn probes(t: &Tile, cam: &Camera, most: usize) -> Vec<Probe> {
    // the view's focal length in pixels: the scale at the focus times the focus's depth
    let m = &cam.m;
    let ortho = m[12] == 0.0 && m[13] == 0.0 && m[14] == 0.0;
    let focal = cam.scale * (m[12] * cam.focus[0] + m[13] * cam.focus[1] + m[14] * cam.focus[2] + m[15]);
    let mut all: Vec<(Probe, f64)> = Vec::new();
    for mk in t.marks.iter().filter(|mk| !mk.pin && coarse_shortfall(mk.kind) > 0.0) {
        let at = [t.origin[0] + mk.pos[0] as f64, t.origin[1] + mk.pos[1] as f64, t.origin[2] + mk.pos[2] as f64];
        let girth = cam.pixels(at, mk.size[0].abs() as f64);
        let mid = if mk.kind == Kind::Rod { [0, 1, 2].map(|i| t.origin[i] + 0.5 * (mk.pos[i] as f64 + mk.aux[i] as f64)) } else { at };
        let Some(p) = cam.project(mid) else { continue };
        let side = if mk.kind == Kind::Rod { (4.0 * girth).max(12.0) } else { (1.4 * girth).max(12.0) };
        if p[0] - 0.5 * side < 0.0 || p[0] + 0.5 * side > cam.width || p[1] - 0.5 * side < 0.0 || p[1] + 0.5 * side > cam.height || p[2].abs() > 1.0 {
            continue;
        }
        let off2 = (p[0] - 0.5 * cam.width).powi(2) + (p[1] - 0.5 * cam.height).powi(2);
        let stretch = if ortho { 1.0 } else { (1.0 + off2 / (focal * focal)).sqrt() };
        let bound_px = girth * coarse_shortfall(mk.kind);
        all.push((Probe { x: p[0], y: p[1], side: side * stretch, bound_px, kind: mk.kind }, bound_px * stretch));
    }
    // up to `most` not yet taken, in the order given
    fn take(out: &mut Vec<Probe>, all: &[(Probe, f64)], most: usize) {
        let mut added = 0;
        for (p, _) in all {
            if added == most {
                break;
            }
            if out.iter().all(|q| q.x != p.x || q.y != p.y) {
                out.push(*p);
                added += 1;
            }
        }
    }
    let mut out = Vec::new();
    all.sort_by(|a, b| b.0.bound_px.total_cmp(&a.0.bound_px));
    take(&mut out, &all, most);
    all.sort_by(|a, b| b.1.total_cmp(&a.1));
    take(&mut out, &all, most);
    out
}

// ------------------------------------------------------------------ the cases the browser draws

/// One tile of a case: its certificate, whether the frame under test drew it coarse, and
/// where to look closely.
pub struct TileInfo {
    pub id: u64,
    pub bound_px: f64,
    pub coarse: bool,
    pub probes: Vec<Probe>,
}

/// The cases, as one JSON index and one blob of packed records (a batch's bytes are stored
/// once however many frames repeat them).
#[derive(Default)]
pub struct Dump {
    pub bin: Vec<u8>,
    seen: HashMap<Vec<u8>, usize>,
    pub cases: Vec<String>,
}

fn nums<T: std::fmt::Display>(v: &[T]) -> String {
    let mut s = String::from("[");
    for (i, x) in v.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        write!(s, "{x}").unwrap();
    }
    s.push(']');
    s
}

pub fn quote(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => write!(o, "\\u{:04x}", c as u32).unwrap(),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

impl Dump {
    fn batches(&mut self, f: &Frame) -> String {
        let mut s = String::from("[");
        let mut first = true;
        for b in f.batches.iter().filter(|b| !b.screen) {
            assert!(b.offset.iter().chain(&b.params).all(|v| v.is_finite()));
            let at = match self.seen.get(&b.bytes) {
                Some(&at) => at,
                None => {
                    let at = self.bin.len();
                    self.bin.extend_from_slice(&b.bytes);
                    self.seen.insert(b.bytes.clone(), at);
                    at
                }
            };
            if !first {
                s.push(',');
            }
            first = false;
            write!(s, "{{\"tile\":{},\"kind\":{},\"count\":{},\"at\":{},\"len\":{},\"coarse\":{},\"offset\":{},\"params\":{}}}", b.tile, b.kind as u8, b.count, at, b.bytes.len(), b.coarse, nums(&b.offset), nums(&b.params)).unwrap();
        }
        s.push(']');
        s
    }

    /// Add a case: the view, the fine frame, the frame under test, the tiles' certificates,
    /// and `extra`, further `"key":value` members already in JSON (what the case expects, where
    /// it comes from).
    pub fn case(&mut self, name: &str, group: &str, cam: &Camera, fine: &Frame, test: &Frame, tiles: &[TileInfo], extra: &[(&str, String)]) {
        let mut s = String::new();
        write!(s, "{{\"name\":{},\"group\":{},\"w\":{},\"h\":{},\"vp\":{},\"remainder_px\":{}", quote(name), quote(group), cam.width, cam.height, nums(&cam.relative()), test.remainder_px).unwrap();
        let (a, b) = (self.batches(fine), self.batches(test));
        write!(s, ",\"fine\":{a},\"test\":{b},\"tiles\":[").unwrap();
        for (i, t) in tiles.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            write!(s, "{{\"id\":{},\"bound_px\":{},\"coarse\":{},\"probes\":[", t.id, t.bound_px, t.coarse).unwrap();
            for (j, p) in t.probes.iter().enumerate() {
                if j > 0 {
                    s.push(',');
                }
                write!(s, "{{\"x\":{},\"y\":{},\"side\":{},\"bound_px\":{},\"kind\":{}}}", p.x, p.y, p.side, p.bound_px, p.kind as u8).unwrap();
            }
            s.push_str("]}");
        }
        s.push(']');
        for (k, v) in extra {
            write!(s, ",{}:{}", quote(k), v).unwrap();
        }
        s.push('}');
        self.cases.push(s);
    }

    pub fn json(&self, extra: &[(&str, String)]) -> String {
        let mut s = String::from("{");
        for (k, v) in extra {
            write!(s, "{}:{},", quote(k), v).unwrap();
        }
        write!(s, "\"bin_bytes\":{},\"cases\":[\n{}\n]}}\n", self.bin.len(), self.cases.join(",\n")).unwrap();
        s
    }
}

/// The certificates and close looks for every tile of a scene under a frame's allocation.
pub fn tile_infos(tiles: &[Tile], cam: &Camera, test: &Frame, most: usize) -> Vec<TileInfo> {
    let coarse = coarse_tiles(test);
    tiles.iter().filter(|t| !t.screen).map(|t| TileInfo { id: t.id, bound_px: tile_bound(t, cam), coarse: coarse.contains(&t.id), probes: if coarse.contains(&t.id) { probes(t, cam, most) } else { Vec::new() } }).collect()
}
