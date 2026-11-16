//! The CEWP twin's street leaf, built on this side of the seam (`INTERFACE.md` §4 rule 2, §7).
//!
//! # What it is
//!
//! The twin builds a leaf's street geometry in JavaScript on the main thread, one vertex at
//! a time (its `tlBuildLeaf`). This module builds THE SAME vertex stream from the same
//! inputs: the kernel's layout buffers for the leaf (roads, buildings, meta, the street
//! graph's end kinds, roof tones) and where the leaf sits in its tile. The output is the
//! twin's own vertex format, 10 floats a vertex (position 2, normal 2, half width, colour
//! 3, alpha, kind), so the twin's program, uniforms and draw order are untouched.
//!
//! It is a compatibility backend, on purpose: the first gate for a replacement is "the same
//! instances", and the cheapest form of that gate is the same bytes. The instanced form of
//! the same marks (a sixth of the bytes) comes after this one is shown equal.
//!
//! # What it has to reproduce exactly
//!
//! The twin places yards, patches and trees by an integer hash of the leaf's id and an
//! index. [`hsh`] is that hash, with JavaScript's `Math.imul` and its conversion of a number
//! to a 32-bit integer (which wraps; it does not saturate). The order of the geometry is the
//! twin's: ground; patches and trees over the whole leaf; a lot and garden trees for each
//! building where the leaf is sparse; street casings then streets, narrow first; then for
//! each building a shadow and a roof in two tones.
//!
//! # What is not claimed
//!
//! Equality with the twin's builder is MEASURED by `tools/twin_leaf_check.mjs`, which runs
//! the twin's own function from the twin's tree; it is not asserted here. Cosine, sine and
//! the base-10 logarithm come from two different libraries on the two sides, so a float may
//! differ in its last place before it is rounded to 32 bits; the check counts such floats.
//! Every display constant is an input ([`Style`], passed across the boundary as one flat
//! array; the twin lead's request, 2026-11-15: one source of truth). [`Style::default`] is
//! the twin's values at its `bb58ce2`, used by the check.

/// The twin's display constants, ALL of them, as inputs. The twin's rule is one source of
/// truth for every model constant; [`Style::from_flat`] takes them as one flat array in the
/// order of [`STYLE_FIELDS`], so the twin can keep the numbers in one place and hand them to
/// both builders. [`Style::default`] is the twin's values at its `bb58ce2`, for the check.
#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    /// Persons per household (the twin's `TL_PPH`).
    pub pph: f64,
    /// A local street's colour for this leaf's density class.
    pub road: [f64; 3],
    /// Roof colours by tone, index 0 unused (neutral).
    pub tones: Vec<[f64; 3]>,
    pub ground_alpha: f64,
    /// Persons a square km above which a leaf has no scattered trees, and above which they
    /// are sparser.
    pub no_trees_pkm: f64,
    pub sparse_trees_pkm: f64,
    /// Square metres of leaf per scattered tree, sparse and otherwise; the cap.
    pub tree_area_sparse: f64,
    pub tree_area: f64,
    pub tree_cap: f64,
    /// Square metres per mottling patch; the cap.
    pub patch_area: f64,
    pub patch_cap: f64,
    pub patch_radius: [f64; 2],
    pub patch_light: [f64; 3],
    pub patch_dark: [f64; 3],
    pub patch_alpha: f64,
    pub tree_radius: [f64; 2],
    /// A tree's colour is `base + gk * slope`.
    pub tree_base: [f64; 3],
    pub tree_slope: [f64; 3],
    pub tree_alpha: f64,
    /// Persons a square km below which buildings have yards.
    pub lots_pkm: f64,
    pub lot_margin: [f64; 2],
    pub lot_colour: [f64; 3],
    pub lot_alpha: f64,
    pub garden_trees_max: f64,
    pub garden_radius: [f64; 2],
    pub garden_base: [f64; 3],
    pub garden_slope_g: f64,
    pub garden_alpha: f64,
    pub arterial_width: f64,
    pub collector_width: f64,
    pub casing_extra: [f64; 2],
    pub casing_colour: [f64; 3],
    pub casing_alpha: f64,
    pub arterial_colour: [f64; 3],
    pub collector_colour: [f64; 3],
    pub shadow: [f64; 2],
    pub shadow_colour: [f64; 3],
    pub shadow_alpha: f64,
    pub roof_units: [f64; 3],
    pub roof_no_units: [f64; 3],
    pub roof_lit: f64,
    pub roof_dark: f64,
}

/// The order of the flat array [`Style::from_flat`] reads, with each field's length.
pub const STYLE_FIELDS: [(&str, usize); 42] = [
    ("pph", 1), ("road", 3), ("tones", 24), ("ground_alpha", 1), ("no_trees_pkm", 1), ("sparse_trees_pkm", 1),
    ("tree_area_sparse", 1), ("tree_area", 1), ("tree_cap", 1), ("patch_area", 1), ("patch_cap", 1), ("patch_radius", 2),
    ("patch_light", 3), ("patch_dark", 3), ("patch_alpha", 1), ("tree_radius", 2), ("tree_base", 3), ("tree_slope", 3),
    ("tree_alpha", 1), ("lots_pkm", 1), ("lot_margin", 2), ("lot_colour", 3), ("lot_alpha", 1), ("garden_trees_max", 1),
    ("garden_radius", 2), ("garden_base", 3), ("garden_slope_g", 1), ("garden_alpha", 1), ("arterial_width", 1),
    ("collector_width", 1), ("casing_extra", 2), ("casing_colour", 3), ("casing_alpha", 1), ("arterial_colour", 3),
    ("collector_colour", 3), ("shadow", 2), ("shadow_colour", 3), ("shadow_alpha", 1), ("roof_units", 3),
    ("roof_no_units", 3), ("roof_lit", 1), ("roof_dark", 1),
];

/// The length of the flat style array.
pub fn style_len() -> usize {
    STYLE_FIELDS.iter().map(|f| f.1).sum()
}

impl Style {
    /// The style from a flat array in [`STYLE_FIELDS`]' order, or `None` if it is not
    /// exactly [`style_len`] long. The tones are 8 colours, the first unused.
    pub fn from_flat(v: &[f64]) -> Option<Style> {
        if v.len() != style_len() {
            return None;
        }
        let mut at = 0usize;
        let mut one = || {
            at += 1;
            v[at - 1]
        };
        let mut s = Style::default();
        s.pph = one();
        s.road = [one(), one(), one()];
        s.tones = (0..8).map(|_| [one(), one(), one()]).collect();
        macro_rules! f {
            ($($x:ident : $n:tt),*) => { $( f!(@ $x $n); )* };
            (@ $x:ident 1) => { s.$x = one(); };
            (@ $x:ident 2) => { s.$x = [one(), one()]; };
            (@ $x:ident 3) => { s.$x = [one(), one(), one()]; };
        }
        f!(ground_alpha: 1, no_trees_pkm: 1, sparse_trees_pkm: 1, tree_area_sparse: 1, tree_area: 1, tree_cap: 1, patch_area: 1, patch_cap: 1, patch_radius: 2,
           patch_light: 3, patch_dark: 3, patch_alpha: 1, tree_radius: 2, tree_base: 3, tree_slope: 3, tree_alpha: 1, lots_pkm: 1, lot_margin: 2, lot_colour: 3,
           lot_alpha: 1, garden_trees_max: 1, garden_radius: 2, garden_base: 3, garden_slope_g: 1, garden_alpha: 1, arterial_width: 1, collector_width: 1,
           casing_extra: 2, casing_colour: 3, casing_alpha: 1, arterial_colour: 3, collector_colour: 3, shadow: 2, shadow_colour: 3, shadow_alpha: 1,
           roof_units: 3, roof_no_units: 3, roof_lit: 1, roof_dark: 1);
        Some(s)
    }
}

impl Default for Style {
    fn default() -> Style {
        let t = |r: f64, g: f64, b: f64| [r / 255.0, g / 255.0, b / 255.0];
        Style {
            pph: 2.5,
            road: [0.47, 0.49, 0.53],
            tones: vec![[0.0; 3], t(178.0, 98.0, 72.0), t(142.0, 122.0, 104.0), t(128.0, 112.0, 94.0), t(92.0, 100.0, 110.0), t(84.0, 82.0, 80.0), t(168.0, 172.0, 176.0), t(150.0, 150.0, 146.0)],
            ground_alpha: 0.9,
            no_trees_pkm: 6000.0,
            sparse_trees_pkm: 1500.0,
            tree_area_sparse: 1600.0,
            tree_area: 700.0,
            tree_cap: 70.0,
            patch_area: 9000.0,
            patch_cap: 14.0,
            patch_radius: [8.0, 18.0],
            patch_light: [0.5, 0.55, 0.36],
            patch_dark: [0.04, 0.07, 0.04],
            patch_alpha: 0.07,
            tree_radius: [1.4, 2.2],
            tree_base: [0.09, 0.17, 0.08],
            tree_slope: [0.05, 0.08, 0.0],
            tree_alpha: 0.8,
            lots_pkm: 1500.0,
            lot_margin: [2.0, 0.35],
            lot_colour: [0.42, 0.5, 0.33],
            lot_alpha: 0.16,
            garden_trees_max: 3.0,
            garden_radius: [1.6, 1.8],
            garden_base: [0.11, 0.2, 0.1],
            garden_slope_g: 0.06,
            garden_alpha: 0.85,
            arterial_width: 11.0,
            collector_width: 7.0,
            casing_extra: [1.6, 0.8],
            casing_colour: [0.07, 0.08, 0.095],
            casing_alpha: 0.9,
            arterial_colour: [0.86, 0.78, 0.56],
            collector_colour: [0.66, 0.66, 0.64],
            shadow: [0.5, 1.0],
            shadow_colour: [0.02, 0.025, 0.03],
            shadow_alpha: 0.5,
            roof_units: [0.44, 0.41, 0.36],
            roof_no_units: [0.3, 0.29, 0.27],
            roof_lit: 1.22,
            roof_dark: 0.74,
        }
    }
}

/// One leaf's inputs: the kernel's buffers and its place in its tile.
#[derive(Clone, Debug, PartialEq)]
pub struct Leaf<'a> {
    /// The leaf's packed id, as the number the twin holds.
    pub id: f64,
    /// 5 per road: x0, y0, x1, y1, width.
    pub roads: &'a [f64],
    /// 8 per building: cx, cy, w, d, angle, floors, units, street.
    pub buildings: &'a [f64],
    /// At least 3: width, height, households.
    pub meta: &'a [f64],
    /// 3 per road: arterial, start kind, end kind (0 is an exit, drawn flush).
    pub streets: &'a [f64],
    /// One roof tone per building, or none.
    pub tones: Option<&'a [f64]>,
    /// The leaf's origin in its tile's frame, and the east-west stretch.
    pub ox: f64,
    pub oy: f64,
    pub sx: f64,
}

/// JavaScript's conversion of a number to a signed 32-bit integer: truncate, then wrap.
pub fn to_int32(x: f64) -> i32 {
    if !x.is_finite() {
        return 0;
    }
    let t = x.trunc();
    let m = t.rem_euclid(4_294_967_296.0);
    (m as u64) as u32 as i32
}

/// The twin's hash of two numbers to a fraction in 0..1.
pub fn hsh(a: f64, b: f64) -> f64 {
    let (a, b) = (to_int32(a), to_int32(b));
    let mut x = (a.wrapping_mul(0x9e37_79b1u32 as i32)) ^ (b.wrapping_mul(0x85eb_ca77u32 as i32));
    x ^= ((x as u32) >> 15) as i32;
    x = x.wrapping_mul(0x2c1b_3c6d);
    x ^= ((x as u32) >> 12) as i32;
    (x as u32) as f64 / 4_294_967_296.0
}

/// JavaScript's `Math.round`: halves go up.
fn round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// The ground's density tone (the twin's `tlTone`).
pub fn tone(density: f64) -> f64 {
    ((density.max(1.0).log10() - 1.0) / 2.5).clamp(0.0, 1.0)
}

struct Out<'a> {
    v: Vec<f32>,
    l: &'a Leaf<'a>,
    /// When set, every primitive is also recorded as an instance (see [`build_leaf_instances`]).
    inst: Option<Vec<Rec>>,
    /// The section being built, which tells a lot's quad from a shadow's and a garden tree
    /// from a scattered one.
    section: Section,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Leaf,
    Lots,
    Streets,
    Buildings,
}

/// One instance with the paint group it belongs to and where its primitive starts in the
/// vertex stream (for the proof that the two are the same geometry).
#[derive(Clone, Copy, Debug)]
pub struct Rec {
    pub group: usize,
    pub inst: crate::Instance,
    pub start: usize,
}

/// The paint groups of a leaf, in the twin's paint order, and the one kind each holds.
pub const GROUPS: [(&str, crate::Kind); 8] = [
    ("ground", crate::Kind::Patch),
    ("mottling and scattered trees", crate::Kind::Disc),
    ("yards", crate::Kind::Quad),
    ("garden trees", crate::Kind::Disc),
    ("casings", crate::Kind::Line),
    ("streets", crate::Kind::Line),
    ("shadows", crate::Kind::Quad),
    ("roofs", crate::Kind::Quad),
];

/// A disc's `variant`: bit 0 puts it in the CASING surface class (the twin's kind 1); bit 2
/// gives it the canopy field (the twin's kind 1.25 is both).
pub const DISC_CASING: u8 = 0x01;
pub const DISC_CANOPY_FIELD: u8 = 0x04;

fn rgba(c: [f64; 3], a: f64) -> [u8; 4] {
    let q = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [q(c[0]), q(c[1]), q(c[2]), q(a)]
}

impl Out<'_> {
    #[allow(clippy::too_many_arguments)]
    fn vert(&mut self, x: f64, y: f64, nx: f64, ny: f64, wd: f64, c: [f64; 3], a: f64, k: f64) {
        let l = self.l;
        self.v.extend_from_slice(&[(l.ox + x * l.sx) as f32, (l.oy + y) as f32, nx as f32, ny as f32, wd as f32, c[0] as f32, c[1] as f32, c[2] as f32, a as f32, k as f32]);
    }
    /// A point of the leaf in the tile's frame, as the vertex stream places it.
    fn tile(&self, x: f64, y: f64) -> [f32; 3] {
        [(self.l.ox + x * self.l.sx) as f32, (self.l.oy + y) as f32, 0.0]
    }
    fn record(&mut self, group: usize, inst: crate::Instance, start: usize) {
        if let Some(v) = self.inst.as_mut() {
            v.push(Rec { group, inst, start });
        }
    }
    fn quad(&mut self, p: [[f64; 2]; 4], c: [f64; 3], a: f64, k: f64) {
        if self.inst.is_some() {
            // the rectangle from its corners: centre on the diagonal, sides from corner 0
            let (cx, cy) = (0.5 * (p[0][0] + p[2][0]), 0.5 * (p[0][1] + p[2][1]));
            let (wx, wy, hx, hy) = (p[1][0] - p[0][0], p[1][1] - p[0][1], p[3][0] - p[0][0], p[3][1] - p[0][1]);
            let mut m = crate::Instance::new(crate::Kind::Quad, self.tile(cx, cy), [wx.hypot(wy) as f32, hx.hypot(hy) as f32, 0.0]);
            m.angle = wy.atan2(wx) as f32;
            let group = if k == 0.0 {
                m.kind = crate::Kind::Patch;
                m.variant = crate::COLOUR_RAMP;
                m.aux[0] = c[0] as f32;
                m.rgba = rgba([1.0, 1.0, 1.0], a);
                0
            } else if k == 3.0 {
                m.variant = crate::COLOUR_TINT;
                m.rgba = rgba(c, a);
                7
            } else {
                m.rgba = rgba(c, a);
                if self.section == Section::Buildings { 6 } else { 2 }
            };
            let start = self.v.len();
            self.record(group, m, start);
        }
        for q in [0, 1, 2, 0, 2, 3] {
            self.vert(p[q][0], p[q][1], 0.0, 0.0, 0.0, c, a, k);
        }
    }
    fn disc(&mut self, x: f64, y: f64, r: f64, c: [f64; 3], a: f64, kd: f64) {
        if self.inst.is_some() {
            let mut m = crate::Instance::new(crate::Kind::Disc, self.tile(x, y), [r as f32, 0.0, 0.0]).colour(rgba(c, a));
            m.variant = if kd == 1.25 { DISC_CASING | DISC_CANOPY_FIELD } else { DISC_CASING };
            let group = if self.section == Section::Lots { 3 } else { 1 };
            let start = self.v.len();
            self.record(group, m, start);
        }
        for k in 0..6 {
            let (a0, a1) = (k as f64 * std::f64::consts::PI / 3.0, (k + 1) as f64 * std::f64::consts::PI / 3.0);
            self.vert(x, y, 0.0, 0.0, 0.0, c, a, kd);
            self.vert(x + r * a0.cos(), y + r * a0.sin(), 0.0, 0.0, 0.0, c, a, kd);
            self.vert(x + r * a1.cos(), y + r * a1.sin(), 0.0, 0.0, 0.0, c, a, kd);
        }
    }
    fn road(&mut self, r: usize, extra: f64, c: [f64; 3], a: f64, k: f64) {
        let rr = self.l.roads;
        let (mut x0, mut y0, mut x1, mut y1, wd) = (rr[r], rr[r + 1], rr[r + 2], rr[r + 3], rr[r + 4] / 2.0 + extra);
        let len = (x1 - x0).hypot(y1 - y0);
        if len < 1e-6 {
            return;
        }
        let (ux, uy) = ((x1 - x0) / len, (y1 - y0) / len);
        // an exit meets the neighbour's street flush; any other end gets a square cap. A
        // missing end kind is not an exit (the twin's `undefined !== 0`).
        let st = (r / 5) * 3;
        let s = self.l.streets;
        if s.get(st + 1).is_none_or(|&v| v != 0.0) {
            x0 -= ux * wd;
            y0 -= uy * wd;
        }
        if s.get(st + 2).is_none_or(|&v| v != 0.0) {
            x1 += ux * wd;
            y1 += uy * wd;
        }
        let (nx, ny) = (-uy, ux);
        if self.inst.is_some() {
            let mut m = crate::Instance::new(crate::Kind::Line, self.tile(x0, y0), [(2.0 * wd) as f32, 0.0, 0.0]).colour(rgba(c, a));
            m.aux = self.tile(x1, y1);
            m.variant = if k == 1.0 { crate::VARIANT_CASING } else { 0 };
            let start = self.v.len();
            self.record(if k == 1.0 { 4 } else { 5 }, m, start);
        }
        for (px, py, sg) in [(x0, y0, -1.0), (x1, y1, -1.0), (x1, y1, 1.0), (x0, y0, -1.0), (x1, y1, 1.0), (x0, y0, 1.0)] {
            self.vert(px, py, nx * sg, ny * sg, wd, c, a, k);
        }
    }
}

/// The leaf's vertex stream in the twin's format: 10 floats a vertex.
pub fn build_leaf(l: &Leaf, style: &Style) -> Vec<f32> {
    build(l, style, false).0
}

/// The same leaf as instance records for `draw.js`, in the twin's paint order by group
/// ([`GROUPS`]): positions in the TILE's frame; quads, discs and lines with their sizes in
/// metres. The records are the vertex stream's geometry: [`expand`] turns them back into
/// the twin's triangles, and the tests compare those with [`build_leaf`]'s within the leaf's
/// east-west stretch (sizes are not stretched; centres and line ends are exact).
pub fn build_leaf_instances(l: &Leaf, style: &Style) -> Vec<Rec> {
    let (_, mut r) = build(l, style, true);
    r.sort_by_key(|x| x.group); // stable: within a group, the twin's order
    r
}

fn build(l: &Leaf, style: &Style, instances: bool) -> (Vec<f32>, Vec<Rec>) {
    let (w, h, hh) = (l.meta[0], l.meta[1], l.meta[2]);
    let mut o = Out { v: Vec::new(), l, inst: if instances { Some(Vec::new()) } else { None }, section: Section::Leaf };
    let km2 = (w * h / 1e6).max(1e-6);
    o.quad([[-w / 2.0, -h / 2.0], [w / 2.0, -h / 2.0], [w / 2.0, h / 2.0], [-w / 2.0, h / 2.0]], [tone(hh / km2), 0.0, 0.0], style.ground_alpha, 0.0);
    let seed = to_int32(l.id % 2_147_483_647.0) as f64;
    let xor = |a: f64, b: f64| (to_int32(a) ^ to_int32(b)) as f64;
    // over the whole leaf: soft mottling and scattered trees, fewer where it is dense
    {
        let area = w * h;
        let pkm = hh / (area / 1e6).max(1e-6) * style.pph;
        let n_t = if pkm >= style.no_trees_pkm { 0.0 } else { round(area / if pkm >= style.sparse_trees_pkm { style.tree_area_sparse } else { style.tree_area }).min(style.tree_cap) };
        let n_p = round(area / style.patch_area).min(style.patch_cap);
        let mut k = 0.0;
        while k < n_p {
            let (x, y, r) = ((hsh(seed, 9000.0 + k) - 0.5) * w, (hsh(9000.0 + k, seed) - 0.5) * h, style.patch_radius[0] + style.patch_radius[1] * hsh(xor(seed, k), 77.0));
            o.disc(x, y, r, if hsh(k, seed) < 0.5 { style.patch_light } else { style.patch_dark }, style.patch_alpha, 1.0);
            k += 1.0;
        }
        let mut k = 0.0;
        while k < n_t {
            let (x, y, r, gk) = ((hsh(seed, k) - 0.5) * w, (hsh(k, seed + 1.0) - 0.5) * h, style.tree_radius[0] + style.tree_radius[1] * hsh(seed + 2.0, k), hsh(k, 31.0));
            let (tb, ts) = (style.tree_base, style.tree_slope);
            o.disc(x, y, r, [tb[0] + ts[0] * gk, tb[1] + ts[1] * gk, tb[2] + ts[2] * gk], style.tree_alpha, 1.25);
            k += 1.0;
        }
    }
    let bb = l.buildings;
    let lots = hh * style.pph / km2 < style.lots_pkm;
    o.section = Section::Lots;
    if lots {
        for b in (0..bb.len()).step_by(8) {
            let (cx, cy, bw, bd, ca, sa) = (bb[b], bb[b + 1], bb[b + 2] / 2.0, bb[b + 3] / 2.0, bb[b + 4].cos(), bb[b + 4].sin());
            let (lw, ld) = (bw + style.lot_margin[0] + style.lot_margin[1] * bw, bd + style.lot_margin[0] + style.lot_margin[1] * bd);
            let at = |u: f64, v: f64| [cx + ca * u - sa * v, cy + sa * u + ca * v];
            o.quad([at(-lw, -ld), at(lw, -ld), at(lw, ld), at(-lw, ld)], style.lot_colour, style.lot_alpha, 1.0);
            let bf = b as f64;
            let nt = (hsh(seed, bf) * style.garden_trees_max).floor() as u32;
            for t in 0..nt {
                let t = t as f64;
                let side = to_int32(hsh(seed + t, bf) * 4.0);
                let f = hsh(bf, seed + 7.0 * t) * 2.0 - 1.0;
                let r = style.garden_radius[0] + style.garden_radius[1] * hsh(xor(seed, t), bf + 3.0);
                let (u, v) = match side {
                    0 => (f * lw, ld - r * 0.6),
                    1 => (f * lw, -ld + r * 0.6),
                    2 => (lw - r * 0.6, f * ld),
                    _ => (-lw + r * 0.6, f * ld),
                };
                let p = at(u, v);
                let g = style.garden_base;
                o.disc(p[0], p[1], r, [g[0], g[1] + style.garden_slope_g * hsh(t, bf), g[2]], style.garden_alpha, 1.25);
            }
        }
    }
    // streets by width class, narrow first so wider ones cross over; casings, then streets
    let rank = |r: usize| if l.roads[r + 4] >= style.arterial_width { 2 } else if l.roads[r + 4] >= style.collector_width { 1 } else { 0 };
    o.section = Section::Streets;
    let mut order: Vec<usize> = (0..l.roads.len()).step_by(5).collect();
    order.sort_by_key(|&r| rank(r));
    for &r in &order {
        o.road(r, if rank(r) == 2 { style.casing_extra[0] } else { style.casing_extra[1] }, style.casing_colour, style.casing_alpha, 1.0);
    }
    for &r in &order {
        let c = match rank(r) {
            2 => style.arterial_colour,
            1 => style.collector_colour,
            _ => style.road,
        };
        o.road(r, 0.0, c, 1.0, 2.0);
    }
    o.section = Section::Buildings;
    for b in (0..bb.len()).step_by(8) {
        let (cx, cy, bw, bd, ca, sa, units) = (bb[b], bb[b + 1], bb[b + 2] / 2.0, bb[b + 3] / 2.0, bb[b + 4].cos(), bb[b + 4].sin(), bb[b + 6]);
        let at = |u: f64, v: f64, dx: f64, dy: f64| [cx + dx + ca * u - sa * v, cy + dy + sa * u + ca * v];
        let sh = style.shadow[0] + style.shadow[1] * bb[b + 5].max(1.0);
        o.quad([at(-bw, -bd, sh, -sh), at(bw, -bd, sh, -sh), at(bw, bd, sh, -sh), at(-bw, bd, sh, -sh)], style.shadow_colour, style.shadow_alpha, 1.0);
        let toned = l.tones.and_then(|t| t.get(b / 8)).map(|&t| to_int32(t)).filter(|&t| t > 0).and_then(|t| style.tones.get(t as usize)).copied();
        let c0 = toned.unwrap_or(if units > 0.0 { style.roof_units } else { style.roof_no_units });
        let (lit, dark) = (c0.map(|v| v * style.roof_lit), c0.map(|v| v * style.roof_dark));
        if bw >= bd {
            o.quad([at(-bw, 0.0, 0.0, 0.0), at(bw, 0.0, 0.0, 0.0), at(bw, bd, 0.0, 0.0), at(-bw, bd, 0.0, 0.0)], lit, 1.0, 3.0);
            o.quad([at(-bw, -bd, 0.0, 0.0), at(bw, -bd, 0.0, 0.0), at(bw, 0.0, 0.0, 0.0), at(-bw, 0.0, 0.0, 0.0)], dark, 1.0, 3.0);
        } else {
            o.quad([at(-bw, -bd, 0.0, 0.0), at(0.0, -bd, 0.0, 0.0), at(0.0, bd, 0.0, 0.0), at(-bw, bd, 0.0, 0.0)], lit, 1.0, 3.0);
            o.quad([at(0.0, -bd, 0.0, 0.0), at(bw, -bd, 0.0, 0.0), at(bw, bd, 0.0, 0.0), at(0.0, bd, 0.0, 0.0)], dark, 1.0, 3.0);
        }
    }
    let inst = o.inst.take().unwrap_or_default();
    (o.v, inst)
}

/// The twin's triangles from instance records, for the proof: each record turned back into
/// the vertices its primitive had (6 for a quad or line, 18 for a disc), in the twin's format.
pub fn expand(r: &Rec, kind_code: f32) -> Vec<f32> {
    let m = &r.inst;
    let c = |i: usize| m.rgba[i] as f32 / 255.0;
    let mut v = Vec::new();
    let mut vert = |x: f32, y: f32, nx: f32, ny: f32, wd: f32, col: [f32; 3], a: f32| v.extend_from_slice(&[x, y, nx, ny, wd, col[0], col[1], col[2], a, kind_code]);
    match m.kind {
        crate::Kind::Quad | crate::Kind::Patch => {
            let (ca, sa) = (m.angle.cos(), m.angle.sin());
            let (hw, hd) = (0.5 * m.size[0], 0.5 * m.size[1]);
            let at = |u: f32, w: f32| [m.pos[0] + ca * u - sa * w, m.pos[1] + sa * u + ca * w];
            let p = [at(-hw, -hd), at(hw, -hd), at(hw, hd), at(-hw, hd)];
            let col = if m.kind == crate::Kind::Patch { [m.aux[0], 0.0, 0.0] } else { [c(0), c(1), c(2)] };
            for q in [0, 1, 2, 0, 2, 3] {
                vert(p[q][0], p[q][1], 0.0, 0.0, 0.0, col, c(3));
            }
        }
        crate::Kind::Disc => {
            for k in 0..6 {
                let (a0, a1) = (k as f64 * std::f64::consts::PI / 3.0, (k + 1) as f64 * std::f64::consts::PI / 3.0);
                let r = m.size[0] as f64;
                vert(m.pos[0], m.pos[1], 0.0, 0.0, 0.0, [c(0), c(1), c(2)], c(3));
                vert(m.pos[0] + (r * a0.cos()) as f32, m.pos[1] + (r * a0.sin()) as f32, 0.0, 0.0, 0.0, [c(0), c(1), c(2)], c(3));
                vert(m.pos[0] + (r * a1.cos()) as f32, m.pos[1] + (r * a1.sin()) as f32, 0.0, 0.0, 0.0, [c(0), c(1), c(2)], c(3));
            }
        }
        crate::Kind::Line => {
            let (dx, dy) = (m.aux[0] - m.pos[0], m.aux[1] - m.pos[1]);
            let len = dx.hypot(dy);
            let (nx, ny) = (-dy / len, dx / len);
            let wd = 0.5 * m.size[0];
            for (px, py, sg) in [(m.pos[0], m.pos[1], -1.0f32), (m.aux[0], m.aux[1], -1.0), (m.aux[0], m.aux[1], 1.0), (m.pos[0], m.pos[1], -1.0), (m.aux[0], m.aux[1], 1.0), (m.pos[0], m.pos[1], 1.0)] {
                vert(px, py, nx * sg, ny * sg, wd, [c(0), c(1), c(2)], c(3));
            }
        }
        _ => {}
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_becomes_a_32_bit_integer_as_javascript_makes_it() {
        assert_eq!(to_int32(5.9), 5);
        assert_eq!(to_int32(-5.9), -5);
        assert_eq!(to_int32(2_147_483_648.0), -2_147_483_648);
        assert_eq!(to_int32(4_294_967_296.0 + 7.0), 7);
        assert_eq!(to_int32(-1.0), -1);
        assert_eq!(to_int32(f64::NAN), 0);
        // values of the twin's hash computed by node (Math.imul), recorded here
        assert_eq!(hsh(1.0, 2.0), 0.83724956563673913);
        assert_eq!(hsh(123456789.0, 9003.0), 0.20617327722720802);
        assert_eq!(hsh(2147483646.0 + 2.0, 5.0), 0.29413622175343335);
    }

    #[test]
    fn a_leaf_has_its_parts_in_the_twins_order() {
        let roads = [0.0, 0.0, 40.0, 0.0, 12.0, 0.0, 10.0, 0.0, 30.0, 5.0];
        let buildings = [10.0, 8.0, 8.0, 6.0, 0.3, 2.0, 1.0, 0.0, -10.0, 8.0, 5.0, 9.0, 0.0, 1.0, 0.0, 0.0];
        let (meta, streets) = ([60.0, 50.0, 4.0], [1.0, 0.0, 1.0, 0.0, 2.0, 2.0]);
        let l = Leaf { id: 3.0 * 140737488355328.0 + 12345.0, roads: &roads, buildings: &buildings, meta: &meta, streets: &streets, tones: Some(&[0.0, 4.0]), ox: 5.0, oy: -3.0, sx: 1.0 };
        let v = build_leaf(&l, &Style::default());
        assert_eq!(v.len() % 10, 0);
        let kinds: Vec<f32> = v.chunks(10).map(|c| c[9]).collect();
        // ground first (6 vertices of kind 0), roofs last (4 quads of kind 3)
        assert!(kinds[..6].iter().all(|&k| k == 0.0) && kinds[6] != 0.0);
        assert!(kinds[kinds.len() - 12..].iter().all(|&k| k == 3.0));
        // 2 casings and 2 streets, 6 vertices each; the narrow street's casing comes first
        let casing: Vec<&[f32]> = v.chunks(10).filter(|c| c[9] == 1.0 && c[4] > 0.0).collect();
        assert_eq!(casing.len(), 12);
        assert_eq!(casing[0][4], 2.5 + 0.8);
        assert_eq!(casing[6][4], 6.0 + 1.6);
        // the arterial starts at an exit (flush at x0) and is capped at its far end
        let art: Vec<&[f32]> = v.chunks(10).filter(|c| c[9] == 2.0 && c[4] == 6.0).collect();
        assert_eq!((art[0][0], art[1][0]), (5.0, 5.0 + 40.0 + 6.0));
        // the second building carries tone 4 (grey slate), lit half first
        let roof = &v[v.len() - 120..];
        let slate = 92.0f64 / 255.0;
        assert_eq!(roof[5], (slate * 1.22) as f32);
        assert_eq!(roof[65], (slate * 0.74) as f32);
        // the same inputs give the same bytes
        assert_eq!(build_leaf(&l, &Style::default()), v);
    }

    #[test]
    fn the_style_is_one_flat_array_and_the_default_round_trips() {
        let d = Style::default();
        let mut flat = vec![d.pph];
        flat.extend(d.road);
        for t in &d.tones {
            flat.extend(t);
        }
        assert_eq!(style_len(), 97);
        assert!(Style::from_flat(&flat).is_none(), "too short");
        // build the rest by reading the default back through a known-good path: every
        // other field, in order
        let rest: Vec<f64> = vec![d.ground_alpha, d.no_trees_pkm, d.sparse_trees_pkm, d.tree_area_sparse, d.tree_area, d.tree_cap, d.patch_area, d.patch_cap]
            .into_iter().chain(d.patch_radius).chain(d.patch_light).chain(d.patch_dark).chain([d.patch_alpha]).chain(d.tree_radius).chain(d.tree_base).chain(d.tree_slope)
            .chain([d.tree_alpha, d.lots_pkm]).chain(d.lot_margin).chain(d.lot_colour).chain([d.lot_alpha, d.garden_trees_max]).chain(d.garden_radius).chain(d.garden_base)
            .chain([d.garden_slope_g, d.garden_alpha, d.arterial_width, d.collector_width]).chain(d.casing_extra).chain(d.casing_colour).chain([d.casing_alpha])
            .chain(d.arterial_colour).chain(d.collector_colour).chain(d.shadow).chain(d.shadow_colour).chain([d.shadow_alpha]).chain(d.roof_units).chain(d.roof_no_units)
            .chain([d.roof_lit, d.roof_dark]).collect();
        flat.extend(rest);
        assert_eq!(Style::from_flat(&flat), Some(d));
    }

    /// A small deterministic generator for synthetic leaves (not the twin's hash).
    fn leaves(n: usize) -> Vec<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, f64, f64, f64, f64)> {
        let mut s = 0x2545_f491u64;
        let mut rnd = move || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            (s >> 11) as f64 / (1u64 << 53) as f64
        };
        (0..n)
            .map(|k| {
                let (w, h) = (40.0 + rnd() * 400.0, 40.0 + rnd() * 400.0);
                let hh = [0.0, 3.0, 12.0, 60.0, 200.0, 900.0][k % 6] * (0.5 + rnd());
                let (mut roads, mut streets, mut b, mut tones) = (vec![], vec![], vec![], vec![]);
                for _ in 0..(rnd() * 8.0) as usize {
                    roads.extend([(rnd() - 0.5) * w, (rnd() - 0.5) * h, (rnd() - 0.5) * w, (rnd() - 0.5) * h, [4.0, 7.0, 9.0, 14.0][(rnd() * 4.0) as usize]]);
                    streets.extend([0.0, (rnd() * 3.0).floor(), (rnd() * 3.0).floor()]);
                }
                for _ in 0..(rnd() * 30.0) as usize {
                    b.extend([(rnd() - 0.5) * w, (rnd() - 0.5) * h, 3.0 + rnd() * 30.0, 3.0 + rnd() * 30.0, rnd() * 6.0 - 3.0, (rnd() * 10.0).floor(), [0.0, 1.0, 8.0][(rnd() * 3.0) as usize], 0.0]);
                    tones.push((rnd() * 8.0).floor());
                }
                let id = 3.0 * 140737488355328.0 + (rnd() * 1e12).floor();
                let sx = 1.0 + (rnd() - 0.5) * 2e-4;
                (roads, b, vec![w, h, hh], streets, tones, id, (rnd() - 0.5) * 2000.0, (rnd() - 0.5) * 2000.0, sx)
            })
            .collect()
    }

    #[test]
    fn the_instance_records_are_the_vertex_streams_geometry() {
        let style = Style::default();
        let (mut prims, mut worst) = (0usize, 0.0f32);
        let mut groups = [0usize; 8];
        for (roads, b, meta, streets, tones, id, ox, oy, sx) in leaves(300) {
            let l = Leaf { id, roads: &roads, buildings: &b, meta: &meta, streets: &streets, tones: Some(&tones), ox, oy, sx };
            let v = build_leaf(&l, &style);
            let recs = build_leaf_instances(&l, &style);
            // every primitive of the stream is one record, and the groups are in paint order
            let n_prims: usize = recs.iter().map(|r| if r.inst.kind == crate::Kind::Disc { 18 } else { 6 }).sum::<usize>();
            assert_eq!(n_prims * 10, v.len());
            assert!(recs.windows(2).all(|p| p[0].group <= p[1].group));
            for r in &recs {
                groups[r.group] += 1;
                assert_eq!(GROUPS[r.group].1, r.inst.kind);
                let kind_code = v[r.start + 9];
                let e = expand(r, kind_code);
                let orig = &v[r.start..r.start + e.len()];
                let size = r.inst.size[0].max(r.inst.size[1]);
                // positions: exact centres and ends; sizes unstretched by sx; f32 rounding
                let tol = 2e-3 + (sx as f32 - 1.0).abs() * size + 1e-6 * (ox.abs().max(oy.abs()) as f32);
                for (a, b) in e.chunks(10).zip(orig.chunks(10)) {
                    for i in [0usize, 1] {
                        let d = (a[i] - b[i]).abs();
                        worst = worst.max(d - tol);
                        assert!(d <= tol, "group {} position off by {d} (tolerance {tol})", GROUPS[r.group].0);
                    }
                    for i in [2usize, 3, 4] {
                        assert!((a[i] - b[i]).abs() <= 1e-3, "group {} normal or width: {} against {}", GROUPS[r.group].0, a[i], b[i]);
                    }
                    for i in [5usize, 6, 7, 8] {
                        assert!((a[i] - b[i]).abs() <= 0.5 / 255.0 + 1e-6, "group {} colour {i}: {} against {}", GROUPS[r.group].0, a[i], b[i]);
                    }
                    assert_eq!(a[9], b[9]);
                }
                prims += 1;
            }
        }
        assert!(prims > 5000 && groups.iter().all(|&g| g > 0), "{prims} {groups:?}");
        assert!(worst <= 0.0);
    }

    #[test]
    fn a_dense_leaf_has_no_yards_and_no_trees() {
        let buildings = [0.0, 0.0, 20.0, 20.0, 0.0, 8.0, 40.0, 0.0];
        let meta = [100.0, 100.0, 60.0]; // 6,000 households a square km: 15,000 persons
        let l = Leaf { id: 77.0, roads: &[], buildings: &buildings, meta: &meta, streets: &[], tones: None, ox: 0.0, oy: 0.0, sx: 1.0 };
        let v = build_leaf(&l, &Style::default());
        assert!(v.chunks(10).all(|c| c[9] != 1.25), "no canopy");
        // ground, one patch (18 vertices), a shadow and two roof halves
        assert_eq!(v.len() / 10, 6 + 18 + 6 + 12);
    }
}
