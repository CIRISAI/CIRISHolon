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
//! The style constants in [`Style`] are the twin's display choices as of its `bb58ce2` and
//! are inputs, not facts.

/// The twin's display constants.
#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    /// Persons per household (the twin's `TL_PPH`).
    pub pph: f64,
    /// A local street's colour for this leaf's density class.
    pub road: [f64; 3],
    /// Roof colours by tone, index 0 unused (neutral).
    pub tones: Vec<[f64; 3]>,
}

impl Default for Style {
    fn default() -> Style {
        let t = |r: f64, g: f64, b: f64| [r / 255.0, g / 255.0, b / 255.0];
        Style {
            pph: 2.5,
            road: [0.47, 0.49, 0.53],
            tones: vec![[0.0; 3], t(178.0, 98.0, 72.0), t(142.0, 122.0, 104.0), t(128.0, 112.0, 94.0), t(92.0, 100.0, 110.0), t(84.0, 82.0, 80.0), t(168.0, 172.0, 176.0), t(150.0, 150.0, 146.0)],
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
}

impl Out<'_> {
    #[allow(clippy::too_many_arguments)]
    fn vert(&mut self, x: f64, y: f64, nx: f64, ny: f64, wd: f64, c: [f64; 3], a: f64, k: f64) {
        let l = self.l;
        self.v.extend_from_slice(&[(l.ox + x * l.sx) as f32, (l.oy + y) as f32, nx as f32, ny as f32, wd as f32, c[0] as f32, c[1] as f32, c[2] as f32, a as f32, k as f32]);
    }
    fn quad(&mut self, p: [[f64; 2]; 4], c: [f64; 3], a: f64, k: f64) {
        for q in [0, 1, 2, 0, 2, 3] {
            self.vert(p[q][0], p[q][1], 0.0, 0.0, 0.0, c, a, k);
        }
    }
    fn disc(&mut self, x: f64, y: f64, r: f64, c: [f64; 3], a: f64, kd: f64) {
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
        for (px, py, sg) in [(x0, y0, -1.0), (x1, y1, -1.0), (x1, y1, 1.0), (x0, y0, -1.0), (x1, y1, 1.0), (x0, y0, 1.0)] {
            self.vert(px, py, nx * sg, ny * sg, wd, c, a, k);
        }
    }
}

/// The leaf's vertex stream in the twin's format: 10 floats a vertex.
pub fn build_leaf(l: &Leaf, style: &Style) -> Vec<f32> {
    let (w, h, hh) = (l.meta[0], l.meta[1], l.meta[2]);
    let mut o = Out { v: Vec::new(), l };
    let km2 = (w * h / 1e6).max(1e-6);
    o.quad([[-w / 2.0, -h / 2.0], [w / 2.0, -h / 2.0], [w / 2.0, h / 2.0], [-w / 2.0, h / 2.0]], [tone(hh / km2), 0.0, 0.0], 0.9, 0.0);
    let seed = to_int32(l.id % 2_147_483_647.0) as f64;
    let xor = |a: f64, b: f64| (to_int32(a) ^ to_int32(b)) as f64;
    // over the whole leaf: soft mottling and scattered trees, fewer where it is dense
    {
        let area = w * h;
        let pkm = hh / (area / 1e6).max(1e-6) * style.pph;
        let n_t = if pkm >= 6000.0 { 0.0 } else { round(area / if pkm >= 1500.0 { 1600.0 } else { 700.0 }).min(70.0) };
        let n_p = round(area / 9000.0).min(14.0);
        let mut k = 0.0;
        while k < n_p {
            let (x, y, r) = ((hsh(seed, 9000.0 + k) - 0.5) * w, (hsh(9000.0 + k, seed) - 0.5) * h, 8.0 + 18.0 * hsh(xor(seed, k), 77.0));
            o.disc(x, y, r, if hsh(k, seed) < 0.5 { [0.5, 0.55, 0.36] } else { [0.04, 0.07, 0.04] }, 0.07, 1.0);
            k += 1.0;
        }
        let mut k = 0.0;
        while k < n_t {
            let (x, y, r, gk) = ((hsh(seed, k) - 0.5) * w, (hsh(k, seed + 1.0) - 0.5) * h, 1.4 + 2.2 * hsh(seed + 2.0, k), hsh(k, 31.0));
            o.disc(x, y, r, [0.09 + 0.05 * gk, 0.17 + 0.08 * gk, 0.08], 0.8, 1.25);
            k += 1.0;
        }
    }
    let bb = l.buildings;
    let lots = hh * style.pph / km2 < 1500.0;
    if lots {
        for b in (0..bb.len()).step_by(8) {
            let (cx, cy, bw, bd, ca, sa) = (bb[b], bb[b + 1], bb[b + 2] / 2.0, bb[b + 3] / 2.0, bb[b + 4].cos(), bb[b + 4].sin());
            let (lw, ld) = (bw + 2.0 + 0.35 * bw, bd + 2.0 + 0.35 * bd);
            let at = |u: f64, v: f64| [cx + ca * u - sa * v, cy + sa * u + ca * v];
            o.quad([at(-lw, -ld), at(lw, -ld), at(lw, ld), at(-lw, ld)], [0.42, 0.5, 0.33], 0.16, 1.0);
            let bf = b as f64;
            let nt = (hsh(seed, bf) * 3.0).floor() as u32;
            for t in 0..nt {
                let t = t as f64;
                let side = to_int32(hsh(seed + t, bf) * 4.0);
                let f = hsh(bf, seed + 7.0 * t) * 2.0 - 1.0;
                let r = 1.6 + 1.8 * hsh(xor(seed, t), bf + 3.0);
                let (u, v) = match side {
                    0 => (f * lw, ld - r * 0.6),
                    1 => (f * lw, -ld + r * 0.6),
                    2 => (lw - r * 0.6, f * ld),
                    _ => (-lw + r * 0.6, f * ld),
                };
                let p = at(u, v);
                o.disc(p[0], p[1], r, [0.11, 0.2 + 0.06 * hsh(t, bf), 0.1], 0.85, 1.25);
            }
        }
    }
    // streets by width class, narrow first so wider ones cross over; casings, then streets
    let rank = |r: usize| if l.roads[r + 4] >= 11.0 { 2 } else if l.roads[r + 4] >= 7.0 { 1 } else { 0 };
    let mut order: Vec<usize> = (0..l.roads.len()).step_by(5).collect();
    order.sort_by_key(|&r| rank(r));
    for &r in &order {
        o.road(r, if rank(r) == 2 { 1.6 } else { 0.8 }, [0.07, 0.08, 0.095], 0.9, 1.0);
    }
    for &r in &order {
        let c = match rank(r) {
            2 => [0.86, 0.78, 0.56],
            1 => [0.66, 0.66, 0.64],
            _ => style.road,
        };
        o.road(r, 0.0, c, 1.0, 2.0);
    }
    for b in (0..bb.len()).step_by(8) {
        let (cx, cy, bw, bd, ca, sa, units) = (bb[b], bb[b + 1], bb[b + 2] / 2.0, bb[b + 3] / 2.0, bb[b + 4].cos(), bb[b + 4].sin(), bb[b + 6]);
        let at = |u: f64, v: f64, dx: f64, dy: f64| [cx + dx + ca * u - sa * v, cy + dy + sa * u + ca * v];
        let sh = 0.5 + 1.0 * bb[b + 5].max(1.0);
        o.quad([at(-bw, -bd, sh, -sh), at(bw, -bd, sh, -sh), at(bw, bd, sh, -sh), at(-bw, bd, sh, -sh)], [0.02, 0.025, 0.03], 0.5, 1.0);
        let toned = l.tones.and_then(|t| t.get(b / 8)).map(|&t| to_int32(t)).filter(|&t| t > 0).and_then(|t| style.tones.get(t as usize)).copied();
        let c0 = toned.unwrap_or(if units > 0.0 { [0.44, 0.41, 0.36] } else { [0.3, 0.29, 0.27] });
        let (lit, dark) = (c0.map(|v| v * 1.22), c0.map(|v| v * 0.74));
        if bw >= bd {
            o.quad([at(-bw, 0.0, 0.0, 0.0), at(bw, 0.0, 0.0, 0.0), at(bw, bd, 0.0, 0.0), at(-bw, bd, 0.0, 0.0)], lit, 1.0, 3.0);
            o.quad([at(-bw, -bd, 0.0, 0.0), at(bw, -bd, 0.0, 0.0), at(bw, 0.0, 0.0, 0.0), at(-bw, 0.0, 0.0, 0.0)], dark, 1.0, 3.0);
        } else {
            o.quad([at(-bw, -bd, 0.0, 0.0), at(0.0, -bd, 0.0, 0.0), at(0.0, bd, 0.0, 0.0), at(-bw, bd, 0.0, 0.0)], lit, 1.0, 3.0);
            o.quad([at(0.0, -bd, 0.0, 0.0), at(bw, -bd, 0.0, 0.0), at(bw, bd, 0.0, 0.0), at(0.0, bd, 0.0, 0.0)], dark, 1.0, 3.0);
        }
    }
    o.v
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
