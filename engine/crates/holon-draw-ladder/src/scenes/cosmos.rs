//! The rungs coarser than the globe: from the Earth and Moon to the cosmic web.
//!
//! Six scenes, laid end to end from `z = 4` to the top of the range (`z` is log10 of the
//! metres one device pixel spans; `zv = z + 3` is the same for the height of a view a
//! thousand pixels high, and is the scale this module's own thresholds are written on):
//!
//! | rung                    | span of z    | zv          | unit    | status       |
//! |-------------------------|--------------|-------------|---------|--------------|
//! | the Earth and the Moon  | 4.0 .. 6.5   | 7 .. 9.5    | 1e6 m   | catalogue    |
//! | the solar system        | 6.5 .. 11    | 9.5 .. 14   | 1e11 m  | catalogue    |
//! | the solar neighbourhood | 11 .. 15.5   | 14 .. 18.5  | 1e16 m  | catalogue    |
//! | the Milky Way           | 15.5 .. 18.5 | 18.5 .. 21.5| 1e18 m  | illustration |
//! | the Local Group         | 18.5 .. 21   | 21.5 .. 24  | 1e21 m  | illustration |
//! | the cosmic web          | 21 .. 24.5   | 24 .. 27.5  | 1e24 m  | illustration |
//!
//! # How the rungs meet
//!
//! The ladder's camera always looks at a rung's origin, so a rung that is about something
//! else than its neighbour would jump. Each scene therefore MOVES ITS OWN ORIGIN AND FRAME
//! with `z`, so that at each shared edge the two rungs draw the same thing in the same
//! place: the Earth rung turns from the Earth's own frame (z = the north pole, x through
//! the Greenwich meridian: the twin's globe) to the ecliptic; the solar system starts about
//! the Earth and slides to the Sun; the neighbourhood turns from the ecliptic to the
//! galactic frame; the Milky Way starts about the Sun and slides to the galactic centre.
//!
//! # What is drawn with what
//!
//! The backend lights spheres from the host's one light, which is not the Sun. A body whose
//! day and night matter (the Earth, the Moon, the planets) is therefore built here from
//! UNLIT tubes (`Kind::Line`) laid along its parallels, each coloured by this module from
//! the Sun's direction. A point (a star, a galaxy) is two short crossed tubes. Sizes given
//! "in pixels" are device pixels at the view's focus (`10^z` metres each).
//!
//! Nothing here was fetched: every catalogue number is entered by hand from the standard
//! published values named in each scene's `source()`, rounded as written.
use crate::Scene;
use holon_draw::ladder::{Status, Z_MAX};
use holon_draw::{Instance, Kind, Tile, PARAMS_NEUTRAL, PARAM_FLOOR};
use std::f64::consts::PI;

/// Every outward scene.
pub fn outward() -> Vec<Box<dyn Scene>> {
    vec![Box::new(EarthMoon::new()), Box::new(SolarSystem::new()), Box::new(Neighbourhood::new()), Box::new(MilkyWay::new()), Box::new(LocalGroup::new()), Box::new(CosmicWeb::new())]
}

// ---------------------------------------------------------------------------------------
// numbers
// ---------------------------------------------------------------------------------------

const DEG: f64 = PI / 180.0;
/// The astronomical unit (IAU 2012, exact) and the light year (Julian year times c), metres.
pub const AU_M: f64 = 1.495_978_707e11;
pub const LY_M: f64 = 9.460_730_472_580_8e15;
/// The Earth's mean radius, the Moon's mean radius and mean distance (centre to centre), metres.
pub const EARTH_R_M: f64 = 6.371e6;
pub const MOON_R_M: f64 = 1.7374e6;
pub const MOON_D_M: f64 = 3.844e8;
/// The inclination of the Moon's orbit to the ecliptic, degrees.
pub const MOON_INCL: f64 = 5.145;
/// The Sun's radius (IAU nominal), metres.
pub const SUN_R_M: f64 = 6.957e8;
/// The Sun's distance from the galactic centre, light years (about 8 kpc).
pub const SUN_GC_LY: f64 = 26_000.0;
/// The Milky Way's stellar disc, radius in light years (about 1e21 m across).
pub const DISC_R_LY: f64 = 52_850.0;
/// Andromeda's distance, light years.
pub const M31_LY: f64 = 2.5e6;
/// The radius of the observable universe today (the particle horizon), metres.
pub const HORIZON_M: f64 = 4.4e26;
/// The decades between `z` (log10 metres per device pixel) and the logarithm of the height in
/// metres of a view a thousand pixels high. Every threshold inside this module (where a
/// frame turns, where an origin slides, where a level of points fades) is written on that
/// second scale, `zv = z + 3`, because that is the one a reader can check against the sizes
/// of things: the Moon's orbit fills a view at `zv` = 8.9, the galaxy at 21.
pub const VIEW_DECADES: f64 = 3.0;

const PAPER: [u8; 3] = [237, 233, 223];
const AMBER: [u8; 3] = [240, 196, 120];
const BLUE: [u8; 3] = [140, 176, 214];

/// One device pixel in the rung's unit, at the view's focus.
fn px(z: f64, unit: i32) -> f64 {
    10f64.powf(z - unit as f64)
}

/// How far a scene's origin has moved from the thing its rung starts about (0) to the thing
/// it ends about (1), `gap` apart, when a pixel is `p` (both in the rung's unit). The first
/// stays within three tenths of a thousand-pixel view of the middle until the second has
/// come into the view, so the zoom never loses sight of where it came from; while the view
/// is under three hundredths of the gap the origin does not move at all.
fn slide(p: f64, gap: f64) -> f64 {
    let g = 1000.0 * p / gap;
    (0.3 * g * smooth(0.03, 0.6, g)).min(1.0)
}

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn rgba(c: [u8; 3], a: f64) -> [u8; 4] {
    [c[0], c[1], c[2], (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8]
}

fn scale_rgb(c: [u8; 3], k: f64) -> [u8; 3] {
    let f = |v: u8| (v as f64 * k).clamp(0.0, 255.0) as u8;
    [f(c[0]), f(c[1]), f(c[2])]
}

fn mix_rgb(a: [u8; 3], b: [u8; 3], t: f64) -> [u8; 3] {
    let f = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * t).clamp(0.0, 255.0) as u8;
    [f(a[0], b[0]), f(a[1], b[1]), f(a[2], b[2])]
}

// ---------------------------------------------------------------------------------------
// vectors and frames
// ---------------------------------------------------------------------------------------

type V3 = [f64; 3];
type M3 = [[f64; 3]; 3];
const I3: M3 = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn mulk(a: V3, k: f64) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn norm(a: V3) -> f64 {
    dot(a, a).sqrt()
}
fn unit(a: V3) -> V3 {
    let l = norm(a);
    if l > 0.0 {
        mulk(a, 1.0 / l)
    } else {
        [1.0, 0.0, 0.0]
    }
}
fn mv(m: &M3, v: V3) -> V3 {
    [dot(m[0], v), dot(m[1], v), dot(m[2], v)]
}
fn mm(a: &M3, b: &M3) -> M3 {
    let mut o = [[0.0; 3]; 3];
    for (r, row) in o.iter_mut().enumerate() {
        for (c, x) in row.iter_mut().enumerate() {
            *x = (0..3).map(|k| a[r][k] * b[k][c]).sum();
        }
    }
    o
}
fn tr(a: &M3) -> M3 {
    let mut o = [[0.0; 3]; 3];
    for (r, row) in o.iter_mut().enumerate() {
        for (c, x) in row.iter_mut().enumerate() {
            *x = a[c][r];
        }
    }
    o
}
fn rot_x(a: f64) -> M3 {
    let (s, c) = a.sin_cos();
    [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]
}
fn rot_z(a: f64) -> M3 {
    let (s, c) = a.sin_cos();
    [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
}
fn f3(v: V3) -> [f32; 3] {
    [v[0] as f32, v[1] as f32, v[2] as f32]
}
fn d3(v: [f32; 3]) -> V3 {
    [v[0] as f64, v[1] as f64, v[2] as f64]
}

/// The rotation `m` done by the fraction `s` of its angle about its own axis: the identity
/// at 0, `m` at 1. This is how a scene turns from one frame to the next as `z` passes.
pub fn rot_partial(m: &M3, s: f64) -> M3 {
    let cos = ((m[0][0] + m[1][1] + m[2][2] - 1.0) * 0.5).clamp(-1.0, 1.0);
    let angle = cos.acos();
    if angle < 1e-9 || s <= 0.0 {
        return I3;
    }
    let mut ax = [m[2][1] - m[1][2], m[0][2] - m[2][0], m[1][0] - m[0][1]];
    if norm(ax) < 1e-6 {
        // a half turn: the axis is the eigenvector of (m + 1) / 2 = a a^T
        let d = [((m[0][0] + 1.0) * 0.5).max(0.0).sqrt(), ((m[1][1] + 1.0) * 0.5).max(0.0).sqrt(), ((m[2][2] + 1.0) * 0.5).max(0.0).sqrt()];
        let k = if d[0] >= d[1] && d[0] >= d[2] { 0 } else if d[1] >= d[2] { 1 } else { 2 };
        ax = d;
        for j in 0..3 {
            if j != k && m[k][j] + m[j][k] < 0.0 {
                ax[j] = -ax[j];
            }
        }
    }
    let a = unit(ax);
    let (sn, cs) = (s * angle).sin_cos();
    let t = 1.0 - cs;
    [
        [cs + a[0] * a[0] * t, a[0] * a[1] * t - a[2] * sn, a[0] * a[2] * t + a[1] * sn],
        [a[1] * a[0] * t + a[2] * sn, cs + a[1] * a[1] * t, a[1] * a[2] * t - a[0] * sn],
        [a[2] * a[0] * t - a[1] * sn, a[2] * a[1] * t + a[0] * sn, cs + a[2] * a[2] * t],
    ]
}

/// Equatorial (J2000) to galactic: x to the galactic centre, z to the north galactic pole
/// (the Hipparcos catalogue's matrix, ESA SP-1200 vol. 1 eq. 1.5.11).
pub const EQ_TO_GAL: M3 = [[-0.054_875_560_4, -0.873_437_090_2, -0.483_835_015_5], [0.494_109_427_9, -0.444_829_630_0, 0.746_982_244_5], [-0.867_666_149_0, -0.198_076_373_4, 0.455_983_776_2]];
/// The obliquity of the ecliptic at J2000, degrees.
pub const OBLIQUITY: f64 = 23.4393;

/// A unit vector from a longitude and a latitude in degrees.
fn dir(lon: f64, lat: f64) -> V3 {
    let (sl, cl) = (lon * DEG).sin_cos();
    let (sb, cb) = (lat * DEG).sin_cos();
    [cb * cl, cb * sl, sb]
}
/// Equatorial (right ascension, declination, degrees) to the ecliptic frame of J2000.
fn eq_to_ecl(ra: f64, dec: f64) -> V3 {
    mv(&rot_x(-OBLIQUITY * DEG), dir(ra, dec))
}
/// Ecliptic to galactic.
fn ecl_to_gal() -> M3 {
    mm(&EQ_TO_GAL, &rot_x(OBLIQUITY * DEG))
}

// ---------------------------------------------------------------------------------------
// the clock: the Sun and the Moon at low precision
// ---------------------------------------------------------------------------------------

/// Days since J2000.0 (2000-01-01 12:00) of a Unix time in seconds. The 69 seconds between
/// UTC and terrestrial time are ignored (the Sun moves 0.0008 degrees in them).
pub fn days_j2000(unix: f64) -> f64 {
    unix / 86_400.0 - 10_957.5
}

/// The Sun as seen from the Earth, by the Astronomical Almanac's low-precision formula
/// (good to about 0.01 degrees between 1950 and 2050).
#[derive(Clone, Copy, Debug)]
pub struct Sun {
    /// Ecliptic longitude, radians.
    pub lambda: f64,
    /// Obliquity of the ecliptic, radians.
    pub eps: f64,
    pub ra: f64,
    pub dec: f64,
    /// Greenwich mean sidereal time, radians.
    pub gmst: f64,
}

pub fn sun(d: f64) -> Sun {
    let l = 280.460 + 0.985_647_4 * d;
    let g = (357.528 + 0.985_600_3 * d) * DEG;
    let lambda = (l + 1.915 * g.sin() + 0.020 * (2.0 * g).sin()) * DEG;
    let eps = (23.439 - 0.000_000_4 * d) * DEG;
    let ra = (eps.cos() * lambda.sin()).atan2(lambda.cos());
    let dec = (eps.sin() * lambda.sin()).asin();
    let gmst = ((280.460_618_37 + 360.985_647_366_29 * d) % 360.0) * DEG;
    Sun { lambda, eps, ra, dec, gmst }
}

/// The point on the Earth with the Sun overhead: latitude and east longitude in degrees.
pub fn subsolar(unix: f64) -> (f64, f64) {
    let s = sun(days_j2000(unix));
    let mut lon = (s.ra - s.gmst) / DEG % 360.0;
    if lon > 180.0 {
        lon -= 360.0;
    }
    if lon <= -180.0 {
        lon += 360.0;
    }
    (s.dec / DEG, lon)
}

/// The Moon's orbit and place on it in the ecliptic frame: the unit vector to the ascending
/// node, the unit vector a quarter turn on in the orbit's plane, and the Moon's angle from
/// the node. The longitude is the mean longitude with the six largest periodic terms (the
/// Almanac's low-precision series, good to about 0.3 degrees); the distance is held at the
/// mean, though it swings between 3.6e8 and 4.1e8 m.
pub fn moon(d: f64) -> (V3, V3, f64) {
    let t = d / 36_525.0;
    let lm = 218.316 + 481_267.881 * t;
    let m1 = (134.963 + 477_198.867 * t) * DEG; // the Moon's mean anomaly
    let m = (357.529 + 35_999.050 * t) * DEG; // the Sun's
    let el = (297.850 + 445_267.111 * t) * DEG; // mean elongation
    let f = (93.272 + 483_202.018 * t) * DEG; // argument of latitude
    let node = (125.045 - 1_934.136 * t) * DEG;
    let lon = (lm + 6.289 * m1.sin() + 1.274 * (2.0 * el - m1).sin() + 0.658 * (2.0 * el).sin() + 0.214 * (2.0 * m1).sin() - 0.186 * m.sin() - 0.114 * (2.0 * f).sin()) * DEG;
    let i = MOON_INCL * DEG;
    let n = [node.cos(), node.sin(), 0.0];
    let p = [-node.sin() * i.cos(), node.cos() * i.cos(), i.sin()];
    (n, p, lon - node)
}

/// The Earth's own frame (z the north pole, x through the Greenwich meridian) to the
/// ecliptic frame of date.
fn earth_to_ecl(s: &Sun) -> M3 {
    mm(&rot_x(-s.eps), &rot_z(s.gmst))
}

// ---------------------------------------------------------------------------------------
// marks
// ---------------------------------------------------------------------------------------

fn seg(a: V3, b: V3, w: f64, c: [u8; 4]) -> Instance {
    let mut i = Instance::new(Kind::Line, f3(a), [w as f32, 0.0, 0.0]).colour(c);
    i.aux = f3(b);
    i
}

/// A circle of `n` chords about `c` in the plane of `u` and `v`, with a vertex at `phase`.
#[allow(clippy::too_many_arguments)]
fn circle(out: &mut Vec<Instance>, c: V3, u: V3, v: V3, r: f64, n: usize, phase: f64, w: f64, col: [u8; 4]) {
    let at = |k: usize| {
        let a = phase + 2.0 * PI * k as f64 / n as f64;
        add(c, add(mulk(u, r * a.cos()), mulk(v, r * a.sin())))
    };
    let mut p = at(0);
    for k in 1..=n {
        let q = at(k);
        out.push(seg(p, q, w, col));
        p = q;
    }
}

/// A point: two short crossed tubes of radius `r`, so that it shows from every side.
fn point(out: &mut Vec<Instance>, p: V3, r: f64, c: [u8; 4]) {
    out.push(seg([p[0] - r, p[1], p[2]], [p[0] + r, p[1], p[2]], 2.0 * r, c));
    out.push(seg([p[0], p[1], p[2] - r], [p[0], p[1], p[2] + r], 2.0 * r, c));
}

/// A star: a point with three thin rays of half length `r`.
fn star(out: &mut Vec<Instance>, p: V3, r: f64, c: [u8; 3], a: f64) {
    point(out, p, 0.34 * r, rgba(c, a));
    let w = 0.16 * r;
    let ray = rgba(c, 0.6 * a);
    out.push(seg([p[0] - r, p[1], p[2]], [p[0] + r, p[1], p[2]], w, ray));
    out.push(seg([p[0], p[1] - r, p[2]], [p[0], p[1] + r, p[2]], w, ray));
    out.push(seg([p[0], p[1], p[2] - r], [p[0], p[1], p[2] + r], w, ray));
}

/// One chord of a ball of radius 1 built from tubes along its parallels.
#[derive(Clone, Copy)]
struct Chord {
    a: [f32; 3],
    b: [f32; 3],
    /// The outward normal at the chord's middle.
    n: [f32; 3],
    /// The tube's width.
    w: f32,
}

/// A ball of radius 1 as tubes along its parallels, one parallel for each band between the
/// latitudes `edges` (degrees, rising from -90 to 90), cut into chords about `chord` degrees
/// long. The tubes' outer faces lie on the unit sphere and each overlaps its neighbours.
fn ball_of(edges: &[f64], chord: f64) -> Vec<Chord> {
    let mut out = Vec::new();
    for e in edges.windows(2) {
        let w = (e[1] - e[0]) * DEG;
        let rho = 0.65 * w;
        let lat = 0.5 * (e[0] + e[1]) * DEG;
        let m = ((2.0 * PI * lat.cos() / (chord * DEG)).round() as usize).max(6);
        let rc = (1.0 - rho) * lat.cos();
        let zc = (1.0 - rho) * lat.sin();
        for j in 0..m {
            let (t0, t1) = (2.0 * PI * j as f64 / m as f64, 2.0 * PI * (j as f64 + 1.0) / m as f64);
            let (a, b) = ([rc * t0.cos(), rc * t0.sin(), zc], [rc * t1.cos(), rc * t1.sin(), zc]);
            let mid = mulk(add(a, b), 0.5);
            // a little overlap, so that the joints between chords are closed
            let a = add(mid, mulk(add(a, mulk(mid, -1.0)), 1.04));
            let b = add(mid, mulk(add(b, mulk(mid, -1.0)), 1.04));
            let tm = 0.5 * (t0 + t1);
            out.push(Chord { a: f3(a), b: f3(b), n: f3([lat.cos() * tm.cos(), lat.cos() * tm.sin(), lat.sin()]), w: (2.0 * rho) as f32 });
        }
    }
    out
}

/// A ball of `rings` equal parallels.
fn ball(rings: usize) -> Vec<Chord> {
    let edges: Vec<f64> = (0..=rings).map(|i| -90.0 + 180.0 * i as f64 / rings as f64).collect();
    ball_of(&edges, 180.0 / rings as f64)
}

/// A ball for a body whose axis is turned to the Sun, so that each parallel is a line of
/// equal light: parallels one degree apart where day turns to night, four elsewhere.
fn ball_for_daylight() -> Vec<Chord> {
    let mut edges = vec![-90.0];
    let mut lat: f64 = -90.0;
    while lat < 90.0 - 1e-9 {
        lat += if (-10.0..6.0).contains(&lat) { 1.0 } else { 4.0 };
        edges.push(lat.min(90.0));
    }
    ball_of(&edges, 4.0)
}

/// The frame whose z axis is `s`: a ball emitted through it has its parallels about `s`.
fn toward(s: V3) -> M3 {
    let k = if s[2].abs() < 0.9 { [0.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0] };
    let u = unit([k[1] * s[2] - k[2] * s[1], k[2] * s[0] - k[0] * s[2], k[0] * s[1] - k[1] * s[0]]);
    let v = [s[1] * u[2] - s[2] * u[1], s[2] * u[0] - s[0] * u[2], s[0] * u[1] - s[1] * u[0]];
    [[u[0], v[0], s[0]], [u[1], v[1], s[1]], [u[2], v[2], s[2]]]
}

/// Emit a ball at `c` with radius `r`, its axis turned by `q`, each chord coloured by `shade`
/// of its outward normal (already turned).
fn emit_ball(out: &mut Vec<Instance>, chords: &[Chord], c: V3, r: f64, q: &M3, shade: impl Fn(V3) -> [u8; 4]) {
    for ch in chords {
        let a = add(c, mulk(mv(q, d3(ch.a)), r));
        let b = add(c, mulk(mv(q, d3(ch.b)), r));
        out.push(seg(a, b, ch.w as f64 * r, shade(mv(q, d3(ch.n)))));
    }
}

/// A body lit by the Sun: `base` in full light, `floor` of it in the dark (a true night is
/// black; a floor above zero keeps a body that is drawn larger than life findable).
fn lit(base: [u8; 3], n: V3, to_sun: V3, floor: f64) -> [u8; 4] {
    let c = dot(n, to_sun);
    rgba(scale_rgb(base, floor + (1.0 - floor) * smooth(-0.05, 0.9, c)), 1.0)
}

/// A tile of this module. Its FLOOR setting is half a pixel: no tube is
/// drawn under one pixel wide, so a far orbit stays a line.
fn tile(id: u64, origin: V3, marks: Vec<Instance>) -> Tile {
    let mut params = PARAMS_NEUTRAL;
    params[PARAM_FLOOR] = 0.5;
    Tile { id, origin, version: 1, layer: 0, screen: false, params, marks }
}

// ---------------------------------------------------------------------------------------
// a small generator, so that every illustration is the same on every machine
// ---------------------------------------------------------------------------------------

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in [0, 1).
    fn f(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    /// About normal, unit variance (four uniforms).
    fn g(&mut self) -> f64 {
        (self.f() + self.f() + self.f() + self.f() - 2.0) * 3f64.sqrt()
    }
    fn on_sphere(&mut self) -> V3 {
        let z = 2.0 * self.f() - 1.0;
        let a = 2.0 * PI * self.f();
        let r = (1.0 - z * z).sqrt();
        [r * a.cos(), r * a.sin(), z]
    }
    fn in_ball(&mut self) -> V3 {
        mulk(self.on_sphere(), self.f().cbrt())
    }
}

/// A point of an illustration: where, what colour, how opaque and how large against its
/// neighbours.
#[derive(Clone, Copy)]
struct Dot {
    p: [f32; 3],
    c: [u8; 3],
    a: f32,
    s: f32,
}

/// Emit dots scaled by `k` about `c`, turned by `q`, each `r` in radius (times its own size,
/// and less with its distance from the focus: see [`depth`]). `origin` is the tile's.
#[allow(clippy::too_many_arguments)]
fn emit_dots(out: &mut Vec<Instance>, dots: &[Dot], c: V3, k: f64, q: Option<&M3>, r: f64, alpha: f64, px: f64, origin: V3) {
    if alpha <= 0.004 {
        return;
    }
    for d in dots {
        let p = mulk(d3(d.p), k);
        let p = match q {
            Some(m) => mv(m, p),
            None => p,
        };
        let at = add(c, p);
        point(out, at, r * d.s as f64 * depth(add(origin, at), px), rgba(d.c, alpha * d.a as f64));
    }
}

/// How much smaller a point is drawn for standing `at` from the view's focus (in the rung's
/// unit; `px` is one pixel). The view is a perspective from about a thousand pixels back, so
/// a point between the focus and the eye would swell to a blob; a scene does not know where
/// the eye is, so every point shrinks with its distance from the focus instead: to a quarter
/// by 800 pixels' worth. (The tile's FLOOR keeps a far point at least a pixel wide.)
fn depth(at: V3, px: f64) -> f64 {
    (1.0 - 0.75 * norm(at) / (800.0 * px)).max(0.25)
}

/// A barred spiral galaxy of `n` dots in a disc of radius 1 in the plane z = 0: a bulge and
/// a bar (amber), four trailing arms (blue white) and a smooth disc. Seen from +z it turns
/// clockwise, as the Milky Way does seen from its north pole. Nothing here is a star count.
fn spiral(seed: u64, n: usize) -> Vec<Dot> {
    let mut r = Rng(seed);
    let mut out = Vec::with_capacity(n);
    let pitch = (12.5 * DEG).tan();
    let bar = 27.0 * DEG; // the bar's near end leads the line from the centre to the Sun
    while out.len() < n {
        let u = r.f();
        let (p, c, a, s) = if u < 0.14 {
            // the bulge
            ([0.055 * r.g(), 0.055 * r.g(), 0.035 * r.g()], [244, 206, 150], 0.85, 1.0)
        } else if u < 0.24 {
            // the bar, along (-cos, sin) of its angle: toward the Sun's side at positive longitude
            let (t, w) = (0.13 * r.g(), 0.03 * r.g());
            ([-t * bar.cos() - w * bar.sin(), t * bar.sin() - w * bar.cos(), 0.02 * r.g()], [240, 200, 140], 0.8, 1.0)
        } else if u < 0.74 {
            // an arm: a logarithmic spiral, radius growing with the angle (trailing)
            let k = (r.f() * 4.0) as usize % 4;
            let rad = 0.17 - 0.30 * (1.0 - r.f() * (1.0 - (-0.83f64 / 0.30).exp())).ln();
            if rad > 1.0 {
                continue;
            }
            let th = k as f64 * 0.5 * PI + 0.6 + (rad / 0.17).ln() / pitch + 0.11 * r.g();
            let rad = rad * (1.0 + 0.035 * r.g());
            let young = r.f() < 0.5;
            ([rad * th.cos(), rad * th.sin(), 0.008 * r.g()], if young { [176, 204, 236] } else { [226, 226, 220] }, if young { 0.9 } else { 0.7 }, if r.f() < 0.12 { 1.5 } else { 1.0 })
        } else {
            // the smooth disc
            let rad = -0.26 * (1.0 - r.f() * (1.0 - (-1.0f64 / 0.26).exp())).ln();
            let th = 2.0 * PI * r.f();
            ([rad * th.cos(), rad * th.sin(), 0.014 * r.g()], [214, 208, 196], 0.5, 0.9)
        };
        out.push(Dot { p: f3(p), c, a, s });
    }
    out
}

/// A web: `nodes` knots scattered in a ball of the given radius, each tied to its three
/// nearest, with `n` dots strung along the threads and gathered at the knots.
fn web(seed: u64, radius: f64, nodes: usize, n: usize) -> Vec<Dot> {
    let mut r = Rng(seed);
    let knots: Vec<V3> = (0..nodes).map(|_| mulk(r.in_ball(), radius)).collect();
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for (i, a) in knots.iter().enumerate() {
        let mut near: Vec<(f64, usize)> = knots.iter().enumerate().filter(|(j, _)| *j != i).map(|(j, b)| (norm(add(*a, mulk(*b, -1.0))), j)).collect();
        near.sort_by(|x, y| x.0.total_cmp(&y.0));
        for &(_, j) in near.iter().take(3) {
            let e = (i.min(j), i.max(j));
            if !edges.contains(&e) {
                edges.push(e);
            }
        }
    }
    let spacing = radius * (4.19 / nodes as f64).cbrt();
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        if r.f() < 0.3 {
            // a cluster at a knot
            let k = knots[(r.f() * nodes as f64) as usize % nodes];
            let s = 0.05 * spacing;
            out.push(Dot { p: f3(add(k, [s * r.g(), s * r.g(), s * r.g()])), c: [240, 214, 170], a: 0.9, s: 1.15 });
        } else {
            let (i, j) = edges[(r.f() * edges.len() as f64) as usize % edges.len()];
            let t = r.f();
            let p = add(mulk(knots[i], 1.0 - t), mulk(knots[j], t));
            let s = 0.035 * spacing;
            out.push(Dot { p: f3(add(p, [s * r.g(), s * r.g(), s * r.g()])), c: [196, 206, 222], a: 0.7, s: 1.0 });
        }
    }
    out
}

// ---------------------------------------------------------------------------------------
// 1. the Earth and the Moon
// ---------------------------------------------------------------------------------------

/// The Earth as a ball with day and night where the Sun puts them now, and the Moon on its
/// orbit. Unit 1e6 m; the origin is the Earth's centre.
pub struct EarthMoon {
    earth: Vec<Chord>,
    /// The Earth once it is under 30 pixels in radius.
    earth_small: Vec<Chord>,
    moon: Vec<Chord>,
}

pub const EARTH_TILE: u64 = 7100;
pub const MOON_TILE: u64 = 7101;
pub const MOON_ORBIT_TILE: u64 = 7102;

impl EarthMoon {
    pub fn new() -> EarthMoon {
        EarthMoon { earth: ball_for_daylight(), earth_small: ball(20), moon: ball(12) }
    }
    /// The fraction of the turn from the Earth's own frame to the ecliptic done at `z`.
    fn turn(z: f64) -> f64 {
        smooth(8.3, 9.2, z)
    }
}

impl Default for EarthMoon {
    fn default() -> Self {
        Self::new()
    }
}

/// The colour of the sea at a place whose normal makes cosine `c` with the Sun's direction:
/// day, a band of twilight about 12 degrees wide, and night.
fn sea(c: f64, lift: f64) -> [u8; 4] {
    let day = scale_rgb([56, 112, 170], (0.62 + 0.38 * c.max(0.0)) * (1.0 + lift));
    let night = scale_rgb([14, 21, 44], 1.0 + 2.0 * lift);
    rgba(mix_rgb(night, day, smooth(-0.14, 0.07, c)), 1.0)
}

impl Scene for EarthMoon {
    fn name(&self) -> &'static str {
        "the Earth and the Moon"
    }
    fn unit(&self) -> i32 {
        6
    }
    fn status(&self) -> Status {
        Status::Catalogue
    }
    fn source(&self) -> &'static str {
        "The Earth at its mean radius (6371 km) with day and night from the Sun's place now (the Astronomical Almanac's low-precision formula, good to 0.01 degrees); the Moon at its mean radius (1737 km) and mean distance (384,400 km) on an orbit tilted 5.1 degrees, placed to about 0.3 degrees and never drawn under 3 pixels, its night side at three tenths of full light so that it can be found; a ring marks the Earth once it is a speck. No land, cloud or ice is drawn."
    }
    fn span(&self) -> (f64, f64) {
        (4.0, 6.5)
    }
    fn tiles(&mut self, z: f64, clock: f64, alpha: f64) -> Vec<Tile> {
        let (p, z) = (px(z, 6), z + VIEW_DECADES);
        let d = days_j2000(clock);
        let s = sun(d);
        let full = earth_to_ecl(&s);
        let q = rot_partial(&full, EarthMoon::turn(z));
        let from_ecl = mm(&q, &tr(&full));
        let to_sun = mv(&from_ecl, [s.lambda.cos(), s.lambda.sin(), 0.0]);
        let re = EARTH_R_M / 1e6;

        let mut earth = Vec::with_capacity(self.earth.len() + 1200);
        let chords = if re / p < 30.0 { &self.earth_small } else { &self.earth };
        emit_ball(&mut earth, chords, [0.0; 3], re, &toward(to_sun), |n| sea(dot(n, to_sun), 0.0));
        // the equator, four parallels and twelve meridians, a little lighter than the sea
        if alpha > 0.05 && re / p > 12.0 {
            let w = (1.1 * p).max(0.004 * re);
            let rg = re * 1.004;
            let (ex, ey, ez) = (mv(&q, [1.0, 0.0, 0.0]), mv(&q, [0.0, 1.0, 0.0]), mv(&q, [0.0, 0.0, 1.0]));
            let mut ring = |c: V3, u: V3, v: V3, r: f64, n: usize| {
                let at = |k: usize| {
                    let a = 2.0 * PI * k as f64 / n as f64;
                    add(c, add(mulk(u, r * a.cos()), mulk(v, r * a.sin())))
                };
                for k in 0..n {
                    let (a, b) = (at(k), at(k + 1));
                    let n = unit(add(a, b));
                    earth.push(seg(a, b, w, sea(dot(n, to_sun), 0.28)));
                }
            };
            for lat in [-60.0f64, -30.0, 0.0, 30.0, 60.0] {
                let (sl, cl) = (lat * DEG).sin_cos();
                ring(mulk(ez, rg * sl), ex, ey, rg * cl, 72);
            }
            for k in 0..6 {
                let a = k as f64 * 30.0 * DEG;
                ring([0.0; 3], add(mulk(ex, a.cos()), mulk(ey, a.sin())), ez, rg, 72);
            }
        }

        // a ring about the Earth once it is a speck
        let speck = 1.0 - smooth(4.0, 8.0, re / p);
        if speck > 0.0 {
            circle(&mut earth, [0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 10.0 * p, 48, 0.0, 1.2 * p, rgba(BLUE, 0.7 * speck));
        }

        // the Moon and its orbit
        let (n, pq, u) = moon(d);
        let (n, pq) = (mv(&from_ecl, n), mv(&from_ecl, pq));
        let dm = MOON_D_M / 1e6;
        let at = add(mulk(n, dm * u.cos()), mulk(pq, dm * u.sin()));
        let mut orbit = Vec::with_capacity(160);
        circle(&mut orbit, [0.0; 3], n, pq, dm, 160, u, 1.1 * p, rgba(PAPER, 0.34));
        let mut body = Vec::with_capacity(self.moon.len());
        let rm = (MOON_R_M / 1e6).max(3.0 * p);
        emit_ball(&mut body, &self.moon, [0.0; 3], rm, &toward(to_sun), |nn| lit([188, 184, 176], nn, to_sun, 0.3));
        vec![tile(EARTH_TILE, [0.0; 3], earth), tile(MOON_ORBIT_TILE, [0.0; 3], orbit), tile(MOON_TILE, at, body)]
    }
}

// ---------------------------------------------------------------------------------------
// 2. the solar system
// ---------------------------------------------------------------------------------------

/// A planet: its name, semi-major axis (au), mean longitude at J2000 (degrees) and its rate
/// (degrees a Julian century), mean radius (km), colour, and least radius on screen (pixels).
/// Elements: Standish's "Keplerian elements for approximate positions of the major planets"
/// (JPL, valid 1800 to 2050), rounded. Radii: IAU/NASA fact-sheet volumetric means.
pub struct Planet {
    pub name: &'static str,
    pub a: f64,
    pub l0: f64,
    pub rate: f64,
    pub radius_km: f64,
    colour: [u8; 3],
    least_px: f64,
}

pub const PLANETS: [Planet; 8] = [
    Planet { name: "Mercury", a: 0.387_10, l0: 252.250_3, rate: 149_472.674_1, radius_km: 2_439.7, colour: [156, 150, 142], least_px: 3.0 },
    Planet { name: "Venus", a: 0.723_33, l0: 181.979_1, rate: 58_517.815_4, radius_km: 6_051.8, colour: [224, 204, 158], least_px: 3.5 },
    Planet { name: "Earth", a: 1.000_00, l0: 100.464_6, rate: 35_999.372_4, radius_km: 6_371.0, colour: [84, 138, 192], least_px: 3.5 },
    Planet { name: "Mars", a: 1.523_71, l0: -4.553_4, rate: 19_140.302_7, radius_km: 3_389.5, colour: [196, 112, 84], least_px: 3.0 },
    Planet { name: "Jupiter", a: 5.202_89, l0: 34.396_4, rate: 3_034.746_1, radius_km: 69_911.0, colour: [208, 174, 134], least_px: 5.5 },
    Planet { name: "Saturn", a: 9.536_68, l0: 49.954_2, rate: 1_222.493_6, radius_km: 58_232.0, colour: [218, 198, 152], least_px: 5.0 },
    Planet { name: "Uranus", a: 19.189_16, l0: 313.238_1, rate: 428.482_0, radius_km: 25_362.0, colour: [160, 206, 212], least_px: 4.2 },
    Planet { name: "Neptune", a: 30.069_92, l0: -55.120_0, rate: 218.459_5, radius_km: 24_622.0, colour: [100, 134, 204], least_px: 4.2 },
];
/// Pluto: semi-major axis (au), mean longitude and rate as above, inclination and node
/// (degrees), radius (km; New Horizons).
pub const PLUTO: (f64, f64, f64, f64, f64, f64) = (39.482_1, 238.929_0, 145.207_8, 17.14, 110.30, 1_188.3);
/// The Kuiper belt as drawn: 30 to 50 au, thickest about its classical part (42 to 48 au).
pub const KUIPER_AU: (f64, f64) = (30.0, 50.0);

pub const SUN_TILE: u64 = 7200;
pub const PLANET_TILE: u64 = 7210; // + index; Pluto is + 8

/// The Sun, the planets on circles at their semi-major axes, Pluto and the Kuiper belt.
/// Unit 1e11 m; the ecliptic frame (x to the equinox, z to the ecliptic's north pole). The
/// origin is the EARTH below about zv = 9.7 and slides to the SUN by zv = 11.7 ([`slide`]).
pub struct SolarSystem {
    sun: Vec<Chord>,
    body: Vec<Chord>,
    kuiper: Vec<Dot>,
}

impl SolarSystem {
    pub fn new() -> SolarSystem {
        let mut r = Rng(0x4b55_4950);
        let au = AU_M / 1e11;
        let kuiper = (0..700)
            .map(|_| {
                // most between 39 and 48 au, the rest spread over the whole belt
                let rad = if r.f() < 0.7 { 43.5 + 2.6 * r.g() } else { KUIPER_AU.0 + (KUIPER_AU.1 - KUIPER_AU.0) * r.f() }.clamp(KUIPER_AU.0, KUIPER_AU.1);
                let th = 2.0 * PI * r.f();
                Dot { p: f3([au * rad * th.cos(), au * rad * th.sin(), au * 2.5 * r.g()]), c: [150, 168, 196], a: 0.55, s: 1.0 }
            })
            .collect();
        SolarSystem { sun: ball(16), body: ball(10), kuiper }
    }
    /// A planet's place at `d` days from J2000, in the rung's unit about the Sun: on a
    /// circle at its semi-major axis, at its MEAN longitude (so wrong by up to twice the
    /// eccentricity: 24 degrees for Mercury, 11 for Mars, under 6 for the rest). The Earth
    /// is placed opposite the Sun of [`sun`] instead, so that it agrees with the rung below.
    pub fn place(i: usize, d: f64) -> V3 {
        let p = &PLANETS[i];
        let a = p.a * AU_M / 1e11;
        let lon = if i == 2 { sun(d).lambda + PI } else { (p.l0 + p.rate * d / 36_525.0) * DEG };
        [a * lon.cos(), a * lon.sin(), 0.0]
    }
    /// Pluto's orbit (node, quarter turn on, radius in the unit) and its angle from the node.
    pub fn pluto(d: f64) -> (V3, V3, f64, f64) {
        let (a, l0, rate, incl, node, _) = PLUTO;
        let (i, o) = (incl * DEG, node * DEG);
        ([o.cos(), o.sin(), 0.0], [-o.sin() * i.cos(), o.cos() * i.cos(), i.sin()], a * AU_M / 1e11, (l0 + rate * d / 36_525.0) * DEG - o)
    }
    /// How far the origin has moved from the Earth (0) to the Sun (1) when a pixel is `p`.
    fn slide(p: f64) -> f64 {
        slide(p, PLANETS[2].a * AU_M / 1e11)
    }
}

impl Default for SolarSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for SolarSystem {
    fn name(&self) -> &'static str {
        "the solar system"
    }
    fn unit(&self) -> i32 {
        11
    }
    fn status(&self) -> Status {
        Status::Catalogue
    }
    fn source(&self) -> &'static str {
        "The Sun and the eight planets on circles at their semi-major axes, each at its mean longitude today (JPL's approximate elements, Standish; within 24 degrees for Mercury, 11 for Mars, 6 for the rest), and Pluto on its tilted orbit drawn as a circle. Bodies are at their true radii but never under 3 to 7 pixels, so their sizes are exaggerated at almost every zoom; their night sides are drawn at three tenths of full light, not black; a ring marks the Earth. The Kuiper belt's points (30 to 50 au) are an illustration."
    }
    fn span(&self) -> (f64, f64) {
        (6.5, 11.0)
    }
    fn tiles(&mut self, z: f64, clock: f64, _alpha: f64) -> Vec<Tile> {
        let d = days_j2000(clock);
        let (p, z) = (px(z, 11), z + VIEW_DECADES);
        let earth = SolarSystem::place(2, d);
        let shift = mulk(earth, -(1.0 - SolarSystem::slide(p)));
        let (ex, ey) = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let mut out = Vec::with_capacity(11);

        // the Sun, the orbits and the belt
        let mut m = Vec::with_capacity(self.sun.len() + 9 * 360 + 2 * self.kuiper.len());
        let rs = (SUN_R_M / 1e11).max(7.0 * p);
        emit_ball(&mut m, &self.sun, [0.0; 3], rs, &I3, |n| rgba(scale_rgb([255, 216, 146], 0.94 + 0.06 * n[2]), 1.0));
        let w = 1.1 * p;
        for (i, pl) in PLANETS.iter().enumerate() {
            let at = SolarSystem::place(i, d);
            let a = pl.a * AU_M / 1e11;
            // skip an orbit that is a speck, and the giants' while the view is about the Earth
            if a < 4.0 * p {
                continue;
            }
            circle(&mut m, [0.0; 3], ex, ey, a, 360, at[1].atan2(at[0]), w, rgba(mix_rgb(PAPER, pl.colour, 0.45), 0.42));
        }
        let (pn, pp, pa, pu) = SolarSystem::pluto(d);
        if pa > 40.0 * p {
            circle(&mut m, [0.0; 3], pn, pp, pa, 360, pu, w, rgba(PAPER, 0.2));
            emit_dots(&mut m, &self.kuiper, [0.0; 3], 1.0, None, 0.8 * p, smooth(11.6, 12.2, z), p, shift);
        }
        out.push(tile(SUN_TILE, shift, m));

        // the planets, each in its own tile about its own centre
        for (i, pl) in PLANETS.iter().enumerate() {
            let at = SolarSystem::place(i, d);
            let to_sun = unit(mulk(at, -1.0));
            let r = (pl.radius_km * 1e3 / 1e11).max(pl.least_px * p);
            let mut m = Vec::with_capacity(self.body.len() + 300);
            let sunward = toward(to_sun);
            emit_ball(&mut m, &self.body, [0.0; 3], r, &sunward, |n| lit(pl.colour, n, to_sun, 0.3));
            if i == 2 {
                // home: a ring about the Earth while it is a speck
                let speck = 1.0 - smooth(4.0, 8.0, r / p);
                if speck > 0.0 {
                    circle(&mut m, [0.0; 3], ex, ey, 10.0 * p, 48, 0.0, 1.2 * p, rgba(BLUE, 0.7 * speck));
                }
            }
            if i == 2 && z < 11.2 {
                // the Moon, as the rung below has it
                let (n, pq, u) = moon(d);
                let dm = MOON_D_M / 1e11;
                let fade = 1.0 - smooth(10.6, 11.2, z);
                circle(&mut m, [0.0; 3], n, pq, dm, 160, u, w, rgba(PAPER, 0.34 * fade));
                let c = add(mulk(n, dm * u.cos()), mulk(pq, dm * u.sin()));
                let rm = (MOON_R_M / 1e11).max(3.0 * p);
                emit_ball(&mut m, &self.body, c, rm, &sunward, |nn| {
                    let mut k = lit([188, 184, 176], nn, to_sun, 0.3);
                    k[3] = (fade * 255.0) as u8;
                    k
                });
            }
            out.push(tile(PLANET_TILE + i as u64, add(shift, at), m));
        }
        if pa > 40.0 * p {
            let at = add(mulk(pn, pa * pu.cos()), mulk(pp, pa * pu.sin()));
            let to_sun = unit(mulk(at, -1.0));
            let mut m = Vec::with_capacity(self.body.len());
            emit_ball(&mut m, &self.body, [0.0; 3], (PLUTO.5 * 1e3 / 1e11).max(2.5 * p), &toward(to_sun), |n| lit([190, 170, 150], n, to_sun, 0.3));
            out.push(tile(PLANET_TILE + 8, add(shift, at), m));
        }
        out
    }
}

// ---------------------------------------------------------------------------------------
// 3. the solar neighbourhood
// ---------------------------------------------------------------------------------------

/// A star: its name, right ascension and declination (J2000, degrees), distance (light
/// years), a colour for its spectral class, and a size on screen (pixels).
pub struct Star {
    pub name: &'static str,
    pub ra: f64,
    pub dec: f64,
    pub ly: f64,
    colour: [u8; 3],
    px: f64,
}

const fn st(name: &'static str, ra: f64, dec: f64, ly: f64, class: u8, px: f64) -> Star {
    let colour = match class {
        b'B' => [170, 196, 240],
        b'A' => [206, 220, 244],
        b'F' => [238, 236, 226],
        b'G' => [246, 228, 176],
        b'K' => [240, 190, 130],
        _ => [224, 150, 112], // M
    };
    Star { name, ra, dec, ly, colour, px }
}

/// Every star system within 12 light years that has a common name, and the brightest stars
/// out to 140. Positions are the catalogue's (Hipparcos; RECONS for the red dwarfs) rounded
/// to a hundredth of a degree, distances to three figures. Entered by hand.
pub const STARS: [Star; 29] = [
    st("Proxima Centauri", 217.43, -62.68, 4.25, b'M', 3.0),
    st("Alpha Centauri", 219.90, -60.83, 4.37, b'G', 5.5),
    st("Barnard's Star", 269.45, 4.69, 5.96, b'M', 3.5),
    st("Wolf 359", 164.12, 7.01, 7.86, b'M', 3.0),
    st("Lalande 21185", 165.83, 35.97, 8.31, b'M', 3.5),
    st("Sirius", 101.29, -16.72, 8.60, b'A', 6.5),
    st("Luyten 726-8", 24.76, -17.95, 8.73, b'M', 3.0),
    st("Ross 154", 282.46, -23.84, 9.70, b'M', 3.0),
    st("Ross 248", 355.48, 44.18, 10.3, b'M', 3.0),
    st("Epsilon Eridani", 53.23, -9.46, 10.5, b'K', 4.5),
    st("Lacaille 9352", 346.47, -35.85, 10.7, b'M', 3.5),
    st("Ross 128", 176.94, 0.80, 11.0, b'M', 3.0),
    st("61 Cygni", 316.72, 38.75, 11.4, b'K', 4.5),
    st("Procyon", 114.83, 5.22, 11.5, b'F', 6.0),
    st("Epsilon Indi", 330.84, -56.79, 11.9, b'K', 4.5),
    st("Tau Ceti", 26.02, -15.94, 11.9, b'G', 4.5),
    st("Altair", 297.70, 8.87, 16.7, b'A', 6.0),
    st("Vega", 279.23, 38.78, 25.0, b'A', 6.5),
    st("Fomalhaut", 344.41, -29.62, 25.1, b'A', 6.0),
    st("Pollux", 116.33, 28.03, 33.8, b'K', 6.0),
    st("Denebola", 177.26, 14.57, 36.0, b'A', 5.0),
    st("Arcturus", 213.92, 19.18, 36.7, b'K', 6.5),
    st("Capella", 79.17, 46.00, 42.9, b'G', 6.5),
    st("Rasalhague", 263.73, 12.56, 48.6, b'A', 5.0),
    st("Castor", 113.65, 31.89, 51.0, b'A', 5.5),
    st("Aldebaran", 68.98, 16.51, 65.0, b'K', 6.0),
    st("Regulus", 152.09, 11.97, 79.0, b'B', 6.0),
    st("Alkaid", 206.89, 49.31, 104.0, b'B', 5.5),
    st("Achernar", 24.43, -57.24, 139.0, b'B', 6.0),
];
/// Where Voyager 1 left the Sun's wind (August 2012), au.
pub const HELIOPAUSE_AU: f64 = 121.0;
/// The Oort cloud as drawn: 2,000 to 100,000 au. It has never been seen.
pub const OORT_AU: (f64, f64) = (2_000.0, 100_000.0);

pub const NEIGHBOUR_TILE: u64 = 7300;

/// The Sun, the edge of its wind, the Oort cloud as a faint shell, and the stars near it.
/// Unit 1e16 m (1.06 light years); the origin is the Sun. The frame is the ecliptic up to
/// about zv = 17.3 and turns to the galactic by zv = 18.2.
pub struct Neighbourhood {
    /// The stars in the ecliptic frame, in the unit.
    stars: Vec<V3>,
    oort: Vec<Dot>,
    to_gal: M3,
}

impl Neighbourhood {
    pub fn new() -> Neighbourhood {
        let stars = STARS.iter().map(|s| mulk(eq_to_ecl(s.ra, s.dec), s.ly * LY_M / 1e16)).collect();
        let mut r = Rng(0x4f4f_5254);
        let (lo, hi) = (OORT_AU.0 * AU_M / 1e16, OORT_AU.1 * AU_M / 1e16);
        let oort = (0..1500)
            .map(|_| {
                let rad = lo * (hi / lo).powf(r.f().powf(0.55));
                Dot { p: f3(mulk(r.on_sphere(), rad)), c: [150, 170, 204], a: 0.5, s: 1.0 }
            })
            .collect();
        Neighbourhood { stars, oort, to_gal: ecl_to_gal() }
    }
    fn turn(z: f64) -> f64 {
        smooth(17.3, 18.2, z)
    }
    /// A star's place in the scene's frame at `zv` (the view-height scale, `z + 3`).
    pub fn star(&self, i: usize, z: f64) -> V3 {
        mv(&rot_partial(&self.to_gal, Neighbourhood::turn(z)), self.stars[i])
    }
}

impl Default for Neighbourhood {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for Neighbourhood {
    fn name(&self) -> &'static str {
        "the solar neighbourhood"
    }
    fn unit(&self) -> i32 {
        16
    }
    fn status(&self) -> Status {
        Status::Catalogue
    }
    fn source(&self) -> &'static str {
        "Twenty-nine stars at their catalogue directions and distances (Hipparcos and RECONS values, rounded to 0.01 degrees and three figures, entered by hand): the named systems within 12 light years and the brightest out to 140; each stands on a stem from the plane, with rings at 1, 10 and 100 light years. Neptune's orbit and the edge of the Sun's wind where Voyager 1 crossed it (121 au) are measured. The Oort cloud (2,000 to 100,000 au) has never been observed: its points are an illustration. Star sizes are not to scale."
    }
    fn span(&self) -> (f64, f64) {
        (11.0, 15.5)
    }
    fn tiles(&mut self, z: f64, _clock: f64, _alpha: f64) -> Vec<Tile> {
        let (p, z) = (px(z, 16), z + VIEW_DECADES);
        let q = rot_partial(&self.to_gal, Neighbourhood::turn(z));
        let (ex, ey) = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let (qx, qy) = (mv(&q, ex), mv(&q, ey));
        let mut m = Vec::with_capacity(4000);
        let w = 1.1 * p;
        let view = 10f64.powf(z - 16.0);
        // Neptune's orbit and the heliopause, in the ecliptic
        let au = AU_M / 1e16;
        for (r, c, a) in [(PLANETS[7].a * au, PLANETS[7].colour, 0.5), (HELIOPAUSE_AU * au, BLUE, 0.45)] {
            if r > 3.0 * p && r < 4.0 * view {
                circle(&mut m, [0.0; 3], qx, qy, r, 128, 0.0, w, rgba(c, a));
            }
        }
        // the Oort cloud
        let oort = smooth(14.6, 15.3, z) * (1.0 - smooth(17.2, 17.9, z));
        emit_dots(&mut m, &self.oort, [0.0; 3], 1.0, None, 0.85 * p, oort, p, [0.0; 3]);
        // rings at 1, 10 and 100 light years, in the scene's plane
        let ly = LY_M / 1e16;
        for r in [ly, 10.0 * ly, 100.0 * ly] {
            if r > 20.0 * p && r < 3.0 * view {
                circle(&mut m, [0.0; 3], ex, ey, r, 180, 0.0, 0.9 * p, rgba(PAPER, 0.16));
            }
        }
        // the Sun and the stars, each on a stem from the plane
        star(&mut m, [0.0; 3], 6.0 * p, AMBER, 1.0);
        let stars = smooth(15.6, 16.2, z);
        if stars > 0.0 {
            for (s, v) in STARS.iter().zip(&self.stars) {
                let at = mv(&q, *v);
                if norm(at) > 4.0 * view {
                    continue;
                }
                m.push(seg([at[0], at[1], 0.0], at, 0.7 * p, rgba(PAPER, 0.17 * stars)));
                star(&mut m, at, s.px * p * depth(at, p), s.colour, stars);
            }
        }
        vec![tile(NEIGHBOUR_TILE, [0.0; 3], m)]
    }
}

// ---------------------------------------------------------------------------------------
// 4. the Milky Way
// ---------------------------------------------------------------------------------------

pub const GALAXY_TILE: u64 = 7400; // + level
pub const GALAXY_SUN_TILE: u64 = 7410;

/// A drawn spiral disc of the Milky Way's size with the Sun marked. Unit 1e18 m (106 light
/// years); the galactic frame (x from the Sun to the centre, z to the north galactic pole).
/// The origin is the SUN below about zv = 18.9 and slides to the CENTRE by zv = 20.9 ([`slide`]).
pub struct MilkyWay {
    /// Three levels of dots about the centre, in the unit: the whole disc; the Sun's
    /// quarter; the Sun's near surroundings.
    whole: Vec<Dot>,
    near: Vec<Dot>,
    local: Vec<Dot>,
}

/// The Sun's place about the galactic centre, in units of 1e18 m.
pub fn sun_in_galaxy() -> V3 {
    [-SUN_GC_LY * LY_M / 1e18, 0.0, 0.0]
}

impl MilkyWay {
    pub fn new() -> MilkyWay {
        let rd = DISC_R_LY * LY_M / 1e18;
        let sun = sun_in_galaxy();
        let scale = |d: &Dot| Dot { p: f3(mulk(d3(d.p), rd)), ..*d };
        let whole: Vec<Dot> = spiral(0x4d57, 3800).iter().map(scale).collect();
        // the same picture drawn again, more densely, within 15,000 light years of the Sun
        let reach = 15_000.0 * LY_M / 1e18;
        let near: Vec<Dot> = spiral(0x4d58, 26_000).iter().map(scale).filter(|d| norm(add(d3(d.p), mulk(sun, -1.0))) < reach).take(1800).collect();
        // and a slab of field stars within 6,000 light years, thinning outward as 1 / distance
        let mut r = Rng(0x4d59);
        let local = (0..2200)
            .map(|_| {
                let rad = 6_000.0 * LY_M / 1e18 * r.f();
                let th = 2.0 * PI * r.f();
                let warm = r.f() < 0.6;
                Dot { p: f3(add(sun, [rad * th.cos(), rad * th.sin(), 3.0 * r.g()])), c: if warm { [232, 214, 184] } else { [190, 208, 236] }, a: 0.75, s: if r.f() < 0.1 { 1.5 } else { 1.0 } }
            })
            .collect();
        MilkyWay { whole, near, local }
    }
    /// How far the origin has moved from the Sun (0) to the centre (1) when a pixel is `p`.
    fn slide(p: f64) -> f64 {
        slide(p, norm(sun_in_galaxy()))
    }
}

impl Default for MilkyWay {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for MilkyWay {
    fn name(&self) -> &'static str {
        "the Milky Way"
    }
    fn unit(&self) -> i32 {
        18
    }
    fn status(&self) -> Status {
        Status::Illustration
    }
    fn source(&self) -> &'static str {
        "A drawing, not a map: a barred spiral of four arms made by rule, at the Milky Way's measured size (a disc about 100,000 light years, 1e21 m, across) with the Sun ringed at its measured 26,000 light years from the centre and its orbit drawn. No point is a real star, and the points are far fewer than the stars."
    }
    fn span(&self) -> (f64, f64) {
        (15.5, 18.5)
    }
    fn tiles(&mut self, z: f64, _clock: f64, _alpha: f64) -> Vec<Tile> {
        let (p, z) = (px(z, 18), z + VIEW_DECADES);
        let sun = sun_in_galaxy();
        let shift = mulk(sun, -(1.0 - MilkyWay::slide(p)));
        let mut out = Vec::with_capacity(4);
        let levels: [(&[Dot], f64, f64); 3] = [
            (&self.local, 1.0 - smooth(19.4, 19.9, z), 1.25),
            (&self.near, smooth(19.2, 19.7, z) * (1.0 - smooth(20.2, 20.7, z)), 1.2),
            (&self.whole, smooth(19.9, 20.5, z), 1.25),
        ];
        for (k, (dots, a, r)) in levels.into_iter().enumerate() {
            if a <= 0.004 {
                continue;
            }
            let mut m = Vec::with_capacity(2 * dots.len());
            emit_dots(&mut m, dots, [0.0; 3], 1.0, None, r * p, a, p, shift);
            out.push(tile(GALAXY_TILE + k as u64, shift, m));
        }
        // the Sun: a point, a ring about it, and its orbit about the centre
        let mut m = Vec::with_capacity(600);
        let (ex, ey) = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        point(&mut m, [0.0; 3], 2.2 * p, rgba(AMBER, 1.0));
        circle(&mut m, [0.0; 3], ex, ey, 11.0 * p, 48, 0.0, 1.3 * p, rgba(AMBER, 0.9));
        let orbit = smooth(19.9, 20.5, z);
        if orbit > 0.0 {
            circle(&mut m, mulk(sun, -1.0), ex, ey, norm(sun), 360, PI, 1.0 * p, rgba(AMBER, 0.3 * orbit));
        }
        out.push(tile(GALAXY_SUN_TILE, add(shift, sun), m));
        out
    }
}

// ---------------------------------------------------------------------------------------
// 5. the Local Group
// ---------------------------------------------------------------------------------------

/// A neighbour galaxy: its name, galactic longitude and latitude (degrees), distance (light
/// years), radius as drawn (light years), and how many points draw it.
pub struct Galaxy {
    pub name: &'static str,
    pub l: f64,
    pub b: f64,
    pub ly: f64,
    pub radius_ly: f64,
    points: usize,
    spiral: bool,
}

/// The Local Group's large members and the Milky Way's two bright companions. Directions
/// from the NASA/IPAC Extragalactic Database's galactic coordinates, rounded to 0.1 degree;
/// distances and sizes the commonly quoted ones, good to about a tenth.
pub const GALAXIES: [Galaxy; 4] = [
    Galaxy { name: "Andromeda (M31)", l: 121.2, b: -21.6, ly: M31_LY, radius_ly: 110_000.0, points: 1800, spiral: true },
    Galaxy { name: "Triangulum (M33)", l: 133.6, b: -31.3, ly: 2.73e6, radius_ly: 30_000.0, points: 420, spiral: true },
    Galaxy { name: "Large Magellanic Cloud", l: 280.5, b: -32.9, ly: 163_000.0, radius_ly: 16_000.0, points: 150, spiral: false },
    Galaxy { name: "Small Magellanic Cloud", l: 302.8, b: -44.3, ly: 200_000.0, radius_ly: 9_000.0, points: 80, spiral: false },
];
/// The Virgo cluster's centre (M87): galactic longitude and latitude, distance in light years.
pub const VIRGO: (f64, f64, f64) = (283.8, 74.5, 5.4e7);

pub const LOCAL_GROUP_TILE: u64 = 7500;

/// The Milky Way, Andromeda, Triangulum and the Magellanic Clouds, then a scatter of
/// galaxies out to the Virgo cluster. Unit 1e21 m (106,000 light years); the galactic frame
/// about the Milky Way's centre.
pub struct LocalGroup {
    milky_way: Vec<Dot>,
    members: Vec<(V3, f64, M3, Vec<Dot>)>,
    field: Vec<Dot>,
}

impl LocalGroup {
    pub fn new() -> LocalGroup {
        let ly = LY_M / 1e21;
        let mut r = Rng(0x4c47);
        let members = GALAXIES
            .iter()
            .enumerate()
            .map(|(i, g)| {
                let dots = if g.spiral {
                    spiral(0x4d33 + i as u64, g.points)
                } else {
                    (0..g.points).map(|_| Dot { p: f3([0.4 * r.g(), 0.4 * r.g(), 0.25 * r.g()]), c: [206, 214, 232], a: 0.7, s: 1.0 }).collect()
                };
                // each disc's tilt is chosen, not measured
                let tilt = mm(&rot_z(1.1 + 1.7 * i as f64), &rot_x(0.9 + 0.5 * i as f64));
                (mulk(dir(g.l, g.b), g.ly * ly), g.radius_ly * ly, tilt, dots)
            })
            .collect();
        // galaxies beyond the group, as single points: a thin scatter and the Virgo cluster
        let mut field = Vec::with_capacity(1500);
        while field.len() < 520 {
            let p = mulk(r.in_ball(), 950.0);
            if norm(p) > 45.0 {
                field.push(Dot { p: f3(p), c: if r.f() < 0.5 { [226, 220, 204] } else { [198, 212, 236] }, a: 0.95, s: 1.5 });
            }
        }
        // and small groups of a few galaxies each, thicker toward the middle, as galaxies gather
        for _ in 0..70 {
            let c = mulk(r.on_sphere(), 70.0 + 600.0 * r.f().powf(1.2));
            let n = 4 + (r.f() * 9.0) as usize;
            for _ in 0..n {
                field.push(Dot { p: f3(add(c, [9.0 * r.g(), 9.0 * r.g(), 9.0 * r.g()])), c: if r.f() < 0.5 { [236, 222, 190] } else { [204, 216, 238] }, a: 0.95, s: 1.5 });
            }
        }
        let virgo = mulk(dir(VIRGO.0, VIRGO.1), VIRGO.2 * ly);
        for _ in 0..220 {
            field.push(Dot { p: f3(add(virgo, [38.0 * r.g(), 38.0 * r.g(), 38.0 * r.g()])), c: [240, 212, 166], a: 0.8, s: 1.1 });
        }
        LocalGroup { milky_way: spiral(0x4d57, 1300), members, field }
    }
    /// A member's centre, in the unit.
    pub fn member(&self, i: usize) -> V3 {
        self.members[i].0
    }
}

impl Default for LocalGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for LocalGroup {
    fn name(&self) -> &'static str {
        "the Local Group"
    }
    fn unit(&self) -> i32 {
        21
    }
    fn status(&self) -> Status {
        Status::Illustration
    }
    fn source(&self) -> &'static str {
        "The Milky Way, Andromeda (2.5 million light years away), Triangulum and the two Magellanic Clouds, each a drawing at its measured place, distance and rough size (galactic coordinates from NED, rounded); the tilt of each disc is chosen, not measured. Once they are specks the measured ones wear a thin ring. The galaxies and small groups beyond are scattered by rule, none of them a real one, except that the Virgo cluster is put at its measured direction and distance (54 million light years)."
    }
    fn span(&self) -> (f64, f64) {
        (18.5, 21.0)
    }
    fn tiles(&mut self, z: f64, _clock: f64, _alpha: f64) -> Vec<Tile> {
        let (p, z) = (px(z, 21), z + VIEW_DECADES);
        let mut m = Vec::with_capacity(8000);
        let galaxy = |m: &mut Vec<Instance>, c: V3, radius: f64, tilt: Option<&M3>, dots: &[Dot], tint: [u8; 3]| {
            if radius < 3.0 * p {
                // a speck: a point, in a thin ring that says it is placed by measurement
                point(m, c, 1.7 * p, rgba(tint, 0.95));
                if norm(c) > 0.0 {
                    circle(m, c, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 7.0 * p * depth(c, p), 24, 0.0, 1.0 * p, rgba(PAPER, 0.55));
                }
            } else {
                // fewer points while the galaxy is small on screen
                let keep = ((radius / p / 60.0).clamp(0.12, 1.0) * dots.len() as f64) as usize;
                emit_dots(m, &dots[..keep.min(dots.len())], c, radius, tilt, 1.05 * p, 1.0, p, [0.0; 3]);
            }
        };
        galaxy(&mut m, [0.0; 3], DISC_R_LY * LY_M / 1e21, None, &self.milky_way, AMBER);
        for (c, radius, tilt, dots) in &self.members {
            galaxy(&mut m, *c, *radius, Some(tilt), dots, PAPER);
        }
        // a ring about the Milky Way, so that home can be found
        circle(&mut m, [0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 12.0 * p, 48, 0.0, 1.2 * p, rgba(AMBER, 0.8 * smooth(22.3, 22.8, z)));
        emit_dots(&mut m, &self.field, [0.0; 3], 1.0, None, 1.4 * p, smooth(22.3, 22.9, z), p, [0.0; 3]);
        vec![tile(LOCAL_GROUP_TILE, [0.0; 3], m)]
    }
}

// ---------------------------------------------------------------------------------------
// 6. the cosmic web
// ---------------------------------------------------------------------------------------

pub const WEB_TILE: u64 = 7600; // + level
pub const WEB_MARK_TILE: u64 = 7610;

/// A drawn web of threads and knots out to the edge of the observable universe. Unit
/// 1e24 m (106 million light years); the galactic frame about the Local Group.
pub struct CosmicWeb {
    /// Three webs, each ten times the last in reach and in the size of its cells.
    levels: [Vec<Dot>; 3],
    virgo: Vec<Dot>,
}

/// The reach of each web, in units of 1e24 m. The innermost has cells of about 1e24 m (100
/// million light years), which is the true size; the outer two are enlarged 10 and 100 times.
pub const WEB_REACH: [f64; 3] = [4.4, 44.0, HORIZON_M / 1e24];

impl CosmicWeb {
    pub fn new() -> CosmicWeb {
        let levels = [web(0x5745_4230, WEB_REACH[0], 220, 2600), web(0x5745_4231, WEB_REACH[1], 260, 2800), web(0x5745_4232, WEB_REACH[2], 300, 3200)];
        let mut r = Rng(0x5649);
        let c = mulk(dir(VIRGO.0, VIRGO.1), VIRGO.2 * LY_M / 1e24);
        let virgo = (0..70).map(|_| Dot { p: f3(add(c, [0.04 * r.g(), 0.04 * r.g(), 0.04 * r.g()])), c: [240, 212, 166], a: 0.85, s: 1.1 }).collect();
        CosmicWeb { levels, virgo }
    }
}

impl Default for CosmicWeb {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for CosmicWeb {
    fn name(&self) -> &'static str {
        "the cosmic web"
    }
    fn unit(&self) -> i32 {
        24
    }
    fn status(&self) -> Status {
        Status::Illustration
    }
    fn source(&self) -> &'static str {
        "A drawing, not a survey: threads and knots of galaxies made by rule, inside a sphere at the edge of the observable universe (4.4e26 m, 46 billion light years, from the standard cosmology). The cells are true to size (about 100 million light years) only near the middle; farther out they are enlarged ten and a hundred times so that a web can be seen at all. Only the ring about the Local Group and the Virgo cluster beside it are placed by measurement. This repository computes nothing at this scale: its cosmological rung is a wager (TIERS.md section 4)."
    }
    fn span(&self) -> (f64, f64) {
        (21.0, Z_MAX)
    }
    fn tiles(&mut self, z: f64, _clock: f64, _alpha: f64) -> Vec<Tile> {
        let (p, z) = (px(z, 24), z + VIEW_DECADES);
        let mut out = Vec::with_capacity(4);
        let fades = [1.0 - smooth(25.0, 25.6, z), smooth(24.7, 25.2, z) * (1.0 - smooth(26.0, 26.5, z)), smooth(25.7, 26.2, z)];
        for (k, (dots, a)) in self.levels.iter().zip(fades).enumerate() {
            if a <= 0.004 {
                continue;
            }
            let mut m = Vec::with_capacity(2 * dots.len());
            emit_dots(&mut m, dots, [0.0; 3], 1.0, None, 1.25 * p, a, p, [0.0; 3]);
            out.push(tile(WEB_TILE + k as u64, [0.0; 3], m));
        }
        let mut m = Vec::with_capacity(900);
        let (ex, ey, ez) = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]);
        // home, the Virgo cluster, and the horizon as three great circles
        point(&mut m, [0.0; 3], 2.0 * p, rgba(AMBER, 1.0));
        circle(&mut m, [0.0; 3], ex, ey, 11.0 * p, 48, 0.0, 1.2 * p, rgba(AMBER, 0.85));
        emit_dots(&mut m, &self.virgo, [0.0; 3], 1.0, None, 1.2 * p, 1.0 - smooth(24.8, 25.4, z), p, [0.0; 3]);
        let edge = smooth(25.9, 26.4, z);
        if edge > 0.0 {
            let r = HORIZON_M / 1e24;
            for (u, v) in [(ex, ey), (ey, ez), (ez, ex)] {
                circle(&mut m, [0.0; 3], u, v, r, 240, 0.0, 1.1 * p, rgba(PAPER, 0.36 * edge));
            }
        }
        out.push(tile(WEB_MARK_TILE, [0.0; 3], m));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-11-15 12:00 UTC.
    const NOW: f64 = 1_794_744_000.0;

    fn find(tiles: &[Tile], id: u64) -> &Tile {
        tiles.iter().find(|t| t.id == id).unwrap_or_else(|| panic!("no tile {id}"))
    }
    /// The farthest reach of a tile's tubes from its origin.
    fn reach(t: &Tile) -> f64 {
        t.marks.iter().map(|m| norm(d3(m.pos)).max(norm(d3(m.aux))) + 0.5 * m.size[0] as f64).fold(0.0, f64::max)
    }
    fn close(a: f64, b: f64, rel: f64) -> bool {
        (a - b).abs() <= rel * b.abs()
    }

    #[test]
    fn the_rungs_tile_from_the_globe_to_the_top_and_say_where_they_come_from() {
        let mut s = outward();
        s.sort_by(|a, b| a.span().0.total_cmp(&b.span().0));
        assert_eq!(s.len(), 6);
        assert_eq!(s[0].span().0, 4.0);
        assert_eq!(s[5].span().1, Z_MAX);
        for w in s.windows(2) {
            assert_eq!(w[0].span().1, w[1].span().0, "{} meets {}", w[0].name(), w[1].name());
        }
        for x in &s {
            assert!(!x.name().is_empty() && x.source().len() > 80 && !x.status().label().is_empty());
            // a view a thousand pixels high, in the rung's unit, stays between a hundredth and a few thousand
            let (lo, hi) = x.span();
            assert!(lo + VIEW_DECADES - x.unit() as f64 >= -2.0 && hi + VIEW_DECADES - x.unit() as f64 <= 3.5, "{}", x.name());
        }
        // the three measured rungs and the three drawn ones
        let st: Vec<Status> = s.iter().map(|x| x.status()).collect();
        assert_eq!(st, [Status::Catalogue, Status::Catalogue, Status::Catalogue, Status::Illustration, Status::Illustration, Status::Illustration]);
        assert!(s[5].source().contains("wager") && s[5].source().contains("TIERS.md"));
        assert!(s[1].source().contains("exaggerated"));
    }

    #[test]
    fn every_rung_draws_finite_marks_across_its_span_and_not_too_many() {
        for mut s in outward() {
            let (lo, hi) = s.span();
            let mut k = lo - 0.25;
            while k <= hi + 0.25 {
                for clock in [NOW, 946_728_000.0, 0.0] {
                    let tiles = s.tiles(k, clock, 1.0);
                    let n: usize = tiles.iter().map(|t| t.marks.len()).sum();
                    assert!(n > 0, "{} draws nothing at {k}", s.name());
                    assert!(n < 30_000, "{} draws {n} marks at {k}", s.name());
                    for t in &tiles {
                        assert!(t.origin.iter().all(|v| v.is_finite()));
                        for m in &t.marks {
                            assert!(m.pos.iter().chain(&m.size).chain(&m.aux).all(|v| v.is_finite()), "{} at {k}", s.name());
                            assert!(m.size[0] > 0.0);
                        }
                    }
                }
                k += 0.25;
            }
        }
    }

    #[test]
    fn the_sun_is_overhead_where_the_almanac_says() {
        // J2000.0: declination -23.03, and the Sun 3.3 minutes short of the Greenwich meridian
        let (lat, lon) = subsolar(946_728_000.0);
        assert!((lat + 23.03).abs() < 0.03, "{lat}");
        assert!((lon - 0.83).abs() < 0.1, "{lon}");
        // the equinox of 2024-03-20 03:06 UTC and the solstice of 2024-06-20 20:51 UTC
        let (lat, lon) = subsolar(1_710_903_960.0);
        assert!(lat.abs() < 0.03, "{lat}");
        assert!((lon - 135.4).abs() < 1.0, "{lon}"); // 03:06 UTC, equation of time -7.5 minutes
        let (lat, _) = subsolar(1_718_916_660.0);
        assert!((lat - 23.44).abs() < 0.01, "{lat}");
    }

    #[test]
    fn the_earth_and_the_moon_have_their_sizes_and_the_day_side_faces_the_sun() {
        let mut s = EarthMoon::new();
        let t = s.tiles(4.6, NOW, 1.0);
        // the Earth's sea reaches 6.371 units (thousand km); the graticule sits 0.6 % above
        let earth = find(&t, EARTH_TILE);
        let sea_reach = earth.marks.iter().filter(|m| m.size[0] > 0.1).map(|m| norm(d3(m.pos)) + 0.5 * m.size[0] as f64).fold(0.0, f64::max);
        assert!(close(sea_reach, 6.371, 0.01), "{sea_reach}");
        assert!(reach(earth) < 6.371 * 1.02);
        // the Moon: its centre at 384.4, its radius 1.737, its orbit a circle through it
        let moon_t = find(&t, MOON_TILE);
        assert!(close(norm(moon_t.origin), 384.4, 1e-9), "{}", norm(moon_t.origin));
        assert!(close(reach(moon_t), 1.7374, 0.03), "{}", reach(moon_t));
        let orbit = find(&t, MOON_ORBIT_TILE);
        assert!(orbit.marks.iter().all(|m| close(norm(d3(m.pos)), 384.4, 1e-5)));
        assert!(norm(add(d3(orbit.marks[0].pos), mulk(moon_t.origin, -1.0))) < 1e-3);
        // in the Earth's own frame (low z) the brightest sea is at the sub-solar point (colours
        // are bytes, so "at" is within the 14 degrees over which the brightest byte holds)
        let (lat, lon) = subsolar(NOW);
        let sub = dir(lon, lat);
        let lum = |m: &Instance| m.rgba[0] as f64 + m.rgba[1] as f64 + m.rgba[2] as f64;
        let brightest = earth.marks.iter().filter(|m| m.size[0] > 0.1).max_by(|a, b| lum(a).total_cmp(&lum(b))).unwrap();
        let mid = unit(add(d3(brightest.pos), d3(brightest.aux)));
        assert!(dot(mid, sub) > 0.97, "{mid:?} against {sub:?}");
        // and the far side is dark
        let darkest = earth.marks.iter().filter(|m| m.size[0] > 0.1).min_by(|a, b| lum(a).total_cmp(&lum(b))).unwrap();
        assert!(dot(unit(add(d3(darkest.pos), d3(darkest.aux))), sub) < -0.1);
        assert!(lum(brightest) > 4.0 * lum(darkest));
    }

    #[test]
    fn the_moon_is_where_the_fuller_series_puts_it() {
        // at J2000.0 the Moon's ecliptic longitude was 223.3 degrees and its latitude +5.2
        let (n, p, u) = moon(0.0);
        let at = add(mulk(n, u.cos()), mulk(p, u.sin()));
        let lon = at[1].atan2(at[0]) / DEG + 360.0;
        assert!((lon - 223.3).abs() < 0.8, "{lon}");
        assert!((at[2].asin() / DEG - 5.17).abs() < 0.4, "{}", at[2].asin() / DEG);
        // the two rungs that draw it agree at their shared edge
        let (mut a, mut b) = (EarthMoon::new(), SolarSystem::new());
        let ta = a.tiles(6.5, NOW, 0.5);
        let tb = b.tiles(6.5, NOW, 0.5);
        let ma = find(&ta, MOON_TILE).origin; // units of 1e6 m about the Earth
        let earth = find(&tb, PLANET_TILE + 2);
        assert!(norm(earth.origin) < 1e-12, "the solar system starts about the Earth");
        let n = b.body.len();
        let moon_ball = earth.marks.iter().rev().take(n).fold([0.0; 3], |s, m| add(s, mulk(add(d3(m.pos), d3(m.aux)), 0.5 / n as f64)));
        assert!(norm(add(mulk(moon_ball, 1e5), mulk(ma, -1.0))) < 0.02 * 384.4, "{moon_ball:?} {ma:?}");
    }

    #[test]
    fn the_planets_are_on_their_orbits() {
        let mut s = SolarSystem::new();
        let t = s.tiles(10.0, NOW, 1.0);
        // by zv = 13 the origin is the Sun
        assert_eq!(find(&t, SUN_TILE).origin, [0.0; 3]);
        let want = [0.5791, 1.0821, 1.4960, 2.2794, 7.7834, 14.267, 28.707, 44.984]; // 1e11 m
        for (i, w) in want.iter().enumerate() {
            let o = find(&t, PLANET_TILE + i as u64).origin;
            assert!(close(norm(o), *w, 2e-4), "{}: {} against {w}", PLANETS[i].name, norm(o));
            assert_eq!(o[2], 0.0);
        }
        // each orbit is a circle through its planet: 360 chords at the planet's distance
        let sun_t = find(&t, SUN_TILE);
        for w in [1.4960, 44.984] {
            let n = sun_t.marks.iter().filter(|m| close(norm(d3(m.pos)), w, 1e-4) && close(norm(d3(m.aux)), w, 1e-4)).count();
            assert_eq!(n, 360, "{w}");
        }
        // Pluto, on its tilted orbit
        let pluto = find(&t, PLANET_TILE + 8).origin;
        assert!(close(norm(pluto), 39.4821 * 1.495978707, 1e-6));
        // sizes are exaggerated here: Jupiter's ball is 5.5 pixels, far over its true 7e-4 units
        let jup = reach(find(&t, PLANET_TILE + 4));
        assert!(close(jup, 5.5 * px(10.0, 11), 0.03) && jup > 100.0 * 6.9911e-4);
        // and true where there is room: the Sun at zv = 9.6 is 6.957e-3 units
        let t = s.tiles(6.6, NOW, 1.0);
        assert!(close(reach_ball(find(&t, SUN_TILE)), 6.957e-3, 0.02));
        // the Earth is opposite the Sun of the rung below, and near where its mean longitude puts it
        let d = days_j2000(NOW);
        let e = SolarSystem::place(2, d);
        let mean = (PLANETS[2].l0 + PLANETS[2].rate * d / 36_525.0) * DEG;
        let sep = (e[1].atan2(e[0]) - mean).sin().asin() / DEG;
        assert!(sep.abs() < 2.1, "{sep}");
        // Jupiter at J2000 was at longitude 34.4
        let j = SolarSystem::place(4, 0.0);
        assert!((j[1].atan2(j[0]) / DEG - 34.3964).abs() < 1e-6);
    }

    /// The reach of the first ball of a tile (its widest tubes), leaving out thin lines.
    fn reach_ball(t: &Tile) -> f64 {
        t.marks.iter().take(100).map(|m| norm(mulk(add(d3(m.pos), d3(m.aux)), 0.5)) + 0.5 * m.size[0] as f64).fold(0.0, f64::max)
    }

    #[test]
    fn a_ball_of_tubes_is_closed_and_touches_its_radius() {
        for chords in [ball(10), ball(16), ball(64), ball_for_daylight()] {
            let out = chords.iter().map(|c| norm(mulk(add(d3(c.a), d3(c.b)), 0.5)) + 0.5 * c.w as f64).fold(0.0, f64::max);
            assert!(out <= 1.0 + 1e-6 && out > 0.985, "{out}");
            assert!(chords.iter().all(|c| close(norm(d3(c.n)), 1.0, 1e-5)));
            // every direction is under some tube: the nearest chord's middle is within its width
            let mut r = Rng(3);
            for _ in 0..300 {
                let v = r.on_sphere();
                let near = chords.iter().map(|c| (norm(add(unit(mulk(add(d3(c.a), d3(c.b)), 0.5)), mulk(v, -1.0))), c.w as f64, norm(add(d3(c.a), mulk(d3(c.b), -1.0))))).min_by(|a, b| a.0.total_cmp(&b.0)).unwrap();
                assert!(near.0 < 0.5 * near.1 + 0.5 * near.2, "{v:?}: {near:?}");
            }
        }
        assert!(ball_for_daylight().len() < 5000);
        // a ball turned toward a direction has that direction as its pole
        let s = unit([0.3, -0.5, 0.8]);
        let q = toward(s);
        assert!(norm(add(mv(&q, [0.0, 0.0, 1.0]), mulk(s, -1.0))) < 1e-12);
        let qq = mm(&q, &tr(&q));
        assert!((0..3).all(|i| (0..3).all(|j| (qq[i][j] - I3[i][j]).abs() < 1e-12)));
    }

    #[test]
    fn the_frames_turn_from_one_to_the_next() {
        // the galactic frame: the pole is z, the centre is x
        let pole = mv(&EQ_TO_GAL, dir(192.859_48, 27.128_25));
        let centre = mv(&EQ_TO_GAL, dir(266.405, -28.936));
        assert!(pole[2] > 0.999_999 && centre[0] > 0.999_999, "{pole:?} {centre:?}");
        // a partial turn is the identity at 0, the whole at 1, and a rotation between
        for m in [ecl_to_gal(), earth_to_ecl(&sun(9_815.0)), rot_z(PI), rot_x(PI - 1e-9), mm(&rot_z(1.0), &rot_x(PI))] {
            let (a, b, h) = (rot_partial(&m, 0.0), rot_partial(&m, 1.0), rot_partial(&m, 0.5));
            let hh = mm(&h, &h);
            let hth = mm(&h, &tr(&h));
            for r in 0..3 {
                for c in 0..3 {
                    assert!((a[r][c] - I3[r][c]).abs() < 1e-12);
                    assert!((b[r][c] - m[r][c]).abs() < 1e-6, "{m:?}");
                    assert!((hh[r][c] - m[r][c]).abs() < 1e-6, "two halves make the whole");
                    assert!((hth[r][c] - I3[r][c]).abs() < 1e-9);
                }
            }
        }
        // the Earth rung is in the Earth's frame at the globe and in the ecliptic at its top:
        // there the Moon's orbit lies within 5.2 degrees of the plane z = 0
        let mut s = EarthMoon::new();
        let top = s.tiles(6.4, NOW, 1.0);
        let tilt = find(&top, MOON_ORBIT_TILE).marks.iter().map(|m| (m.pos[2] as f64 / 384.4).abs()).fold(0.0, f64::max);
        assert!(close(tilt, (MOON_INCL * DEG).sin(), 0.02), "{tilt}");
        let low = s.tiles(4.4, NOW, 1.0);
        let tilt = find(&low, MOON_ORBIT_TILE).marks.iter().map(|m| (m.pos[2] as f64 / 384.4).abs()).fold(0.0, f64::max);
        assert!(tilt > (18.0 * DEG).sin() && tilt < (29.0 * DEG).sin(), "against the equator the orbit tilts 18 to 29 degrees: {tilt}");
    }

    #[test]
    fn the_stars_are_at_their_distances_and_the_brightest_are_where_they_should_be() {
        let s = Neighbourhood::new();
        let ly = LY_M / 1e16;
        for (i, st) in STARS.iter().enumerate() {
            assert!(close(norm(s.star(i, 16.5)), st.ly * ly, 1e-9), "{}", st.name);
            assert!(close(norm(s.star(i, 18.4)), st.ly * ly, 1e-9), "{}", st.name);
            assert!((-90.0..=90.0).contains(&st.dec) && (0.0..360.0).contains(&st.ra));
        }
        assert!(STARS.windows(2).all(|w| w[0].ly <= w[1].ly), "listed nearest first");
        let i = |n: &str| STARS.iter().position(|s| s.name == n).unwrap();
        // Alpha Centauri: 4.37 light years is 4.134 units of 1e16 m
        assert!(close(norm(s.star(i("Alpha Centauri"), 16.0)), 4.134, 1e-3));
        // Proxima is 2.2 degrees from Alpha Centauri; Sirius and Procyon are 25.7 apart
        let sep = |a: &str, b: &str| dot(unit(s.star(i(a), 16.0)), unit(s.star(i(b), 16.0))).acos() / DEG;
        assert!((sep("Proxima Centauri", "Alpha Centauri") - 2.18).abs() < 0.05, "{}", sep("Proxima Centauri", "Alpha Centauri"));
        assert!((sep("Sirius", "Procyon") - 25.7).abs() < 0.2, "{}", sep("Sirius", "Procyon"));
        // in the galactic frame (the top of the rung): Alpha Centauri at l 315.7, b -0.7;
        // Vega at l 67.4, b +19.2; Sirius at l 227.2, b -8.9
        for (n, l, b) in [("Alpha Centauri", 315.73, -0.68), ("Vega", 67.45, 19.24), ("Sirius", 227.23, -8.89)] {
            let v = unit(s.star(i(n), 18.4));
            let (gl, gb) = ((v[1].atan2(v[0]) / DEG + 360.0) % 360.0, v[2].asin() / DEG);
            assert!((gl - l).abs() < 0.1 && (gb - b).abs() < 0.1, "{n}: {gl} {gb}");
        }
        // in the ecliptic frame (the bottom): Regulus lies within half a degree of the ecliptic
        let v = unit(s.star(i("Regulus"), 15.0));
        assert!((v[2].asin() / DEG - 0.46).abs() < 0.1, "{}", v[2].asin() / DEG);
        // the Oort cloud is a shell from 2,000 to 100,000 au
        let au = AU_M / 1e16;
        let (lo, hi) = s.oort.iter().map(|d| norm(d3(d.p))).fold((f64::MAX, 0.0f64), |a, r| (a.0.min(r), a.1.max(r)));
        assert!(lo >= 2_000.0 * au * 0.999 && hi <= 100_000.0 * au * 1.001 && hi > 50_000.0 * au, "{lo} {hi}");
    }

    #[test]
    fn the_milky_way_has_its_size_and_the_sun_its_place() {
        let mut s = MilkyWay::new();
        // 1e21 m across: 1000 units of 1e18 m
        let across = 2.0 * s.whole.iter().map(|d| norm(d3(d.p))).fold(0.0, f64::max);
        assert!(close(across, 1000.0, 0.03), "{across}");
        // the disc is thin: nine points in ten within 3 % of its radius of the plane
        let thin = s.whole.iter().filter(|d| d.p[2].abs() < 15.0).count();
        assert!(thin * 10 > s.whole.len() * 9);
        // the Sun: 26,000 light years, 246 units, from the centre
        assert!(close(norm(sun_in_galaxy()), 245.98, 1e-3), "{}", norm(sun_in_galaxy()));
        // low in the rung the Sun is the origin, high in it the centre is
        let t = s.tiles(15.6, NOW, 1.0);
        assert!(norm(find(&t, GALAXY_SUN_TILE).origin) < 1e-12);
        assert!(close(norm(find(&t, GALAXY_TILE).origin), 245.98, 1e-3));
        let t = s.tiles(18.0, NOW, 1.0);
        assert!(close(norm(find(&t, GALAXY_SUN_TILE).origin), 245.98, 1e-3));
        assert_eq!(find(&t, GALAXY_TILE + 2).origin, [0.0; 3]);
        // the near levels really are about the Sun
        let sun = sun_in_galaxy();
        assert!(s.local.iter().all(|d| norm(add(d3(d.p), mulk(sun, -1.0))) < 6_300.0 * LY_M / 1e18));
        assert!(s.near.len() > 1500 && s.near.iter().all(|d| norm(add(d3(d.p), mulk(sun, -1.0))) < 15_000.0 * LY_M / 1e18));
    }

    #[test]
    fn andromeda_virgo_and_the_horizon_are_at_their_distances() {
        let s = LocalGroup::new();
        // 2.5 million light years is 23.65 units of 1e21 m
        assert!(close(norm(s.member(0)), 23.652, 1e-3), "{}", norm(s.member(0)));
        // Andromeda and Triangulum are 14.8 degrees apart on the sky as seen from here
        let sep = dot(unit(s.member(0)), unit(s.member(1))).acos() / DEG;
        assert!((sep - 14.8).abs() < 0.5, "{sep}");
        // the Clouds are 21 degrees apart
        let sep = dot(unit(s.member(2)), unit(s.member(3))).acos() / DEG;
        assert!((sep - 21.0).abs() < 1.5, "{sep}");
        // the horizon: 4.4e26 m is 440 units of 1e24 m, and nothing of the web is beyond it
        let mut w = CosmicWeb::new();
        assert_eq!(WEB_REACH[2], 440.0);
        let far = w.levels[2].iter().map(|d| norm(d3(d.p))).fold(0.0, f64::max);
        assert!(far < 440.0 * 1.06 && far > 300.0, "{far}");
        let t = w.tiles(23.8, NOW, 1.0);
        let marks = find(&t, WEB_MARK_TILE);
        let ring = marks.marks.iter().filter(|m| close(norm(d3(m.pos)), 440.0, 1e-5)).count();
        assert_eq!(ring, 3 * 240);
        // the Virgo cluster: 54 million light years is 0.51 units
        let c = w.virgo.iter().fold([0.0; 3], |s, d| add(s, mulk(d3(d.p), 1.0 / w.virgo.len() as f64)));
        assert!(close(norm(c), 0.511, 0.05), "{}", norm(c));
        // the innermost web's cells are about one unit (100 million light years)
        let cell = WEB_REACH[0] * (4.19 / 220.0f64).cbrt();
        assert!(cell > 0.8 && cell < 1.5, "{cell}");
    }

    #[test]
    fn an_origin_slides_without_losing_sight_of_where_it_came_from() {
        let gap = 1.496;
        let mut last = 0.0;
        for k in 0..400 {
            let p = 10f64.powf(-6.0 + 0.02 * k as f64);
            let s = slide(p, gap);
            assert!((0.0..=1.0).contains(&s) && s >= last);
            // the thing left behind is within 0.3 of a thousand-pixel view of the middle
            assert!(s * gap <= 0.3 * 1000.0 * p + 1e-12 || s == 1.0);
            last = s;
        }
        assert_eq!(slide(1e-6, gap), 0.0);
        assert_eq!(slide(0.01, gap), 1.0);
        // at the edge shared with the rung below (zv = 9.75 at most) the Earth is still the middle
        assert!(SolarSystem::slide(px(6.75, 11)) * 1.496 < 2.0 * px(6.75, 11));
        assert!(MilkyWay::slide(px(15.75, 18)) * 246.0 < 2.0 * px(15.75, 18));
    }

    #[test]
    fn what_is_drawn_depends_on_the_pixel_alone() {
        // A scene is given `z` (metres per device pixel) and never the window's size, so a
        // phone and a wall at the same `z` get the same marks: two scenes built apart agree
        // mark for mark, and every size chosen "in pixels" is that many times 10^z metres.
        for (mut a, mut b) in outward().into_iter().zip(outward()) {
            let (lo, hi) = a.span();
            for z in [lo, 0.5 * (lo + hi), hi] {
                let (ta, tb) = (a.tiles(z, NOW, 1.0), b.tiles(z, NOW, 1.0));
                assert!(ta == tb, "{} at {z}", a.name());
            }
        }
        // the Moon's least radius is 3 pixels at any zoom where its true radius is less
        let mut s = EarthMoon::new();
        for z in [6.0, 6.4] {
            let t = s.tiles(z, NOW, 1.0);
            assert!(close(reach(find(&t, MOON_TILE)), 3.0 * 10f64.powf(z - 6.0), 0.03));
        }
    }

    #[test]
    fn what_a_rung_draws_at_a_zoom_does_not_depend_on_the_windows_size() {
        // through the ladder, as the inward rungs test it: a phone and a large screen at one z
        // get the same world batches, byte for byte; only the camera differs
        use crate::Ladder;
        use holon_draw::ladder::Band;
        let band = Band { z_lo: -1.0, z_hi: 4.2, clamp: false };
        let (mut small, mut large) = (Ladder::standard(band), Ladder::standard(band)); // with the inward rungs, so that the band is where it is on the page
        for k in 0..=41 {
            let z = 4.0 + k as f64 * 0.5;
            let a = small.passes(z, 0.8, 0.42, 390.0, 844.0, 3.0, NOW);
            let b = large.passes(z, 0.8, 0.42, 2560.0, 1440.0, 1.0, NOW);
            assert_eq!(a.len(), b.len(), "z = {z}");
            assert!(!a.is_empty() || z < 4.5, "z = {z}");
            for (p, q) in a.iter().zip(&b) {
                assert_eq!((p.rung, p.weight), (q.rung, q.weight));
                let world = |f: &holon_draw::Frame| f.batches.iter().filter(|b| !b.screen).map(|b| (b.tile, b.kind, b.coarse, b.bytes.clone(), b.params)).collect::<Vec<_>>();
                assert!(world(&p.frame) == world(&q.frame), "z = {z}, rung {}", small.rungs[p.rung].name);
            }
        }
    }

    /// Run with `cargo test --release -p holon-draw-ladder timing -- --ignored --nocapture`.
    #[test]
    #[ignore = "a timing, to be read in release"]
    fn timing_of_a_scenes_tiles_and_of_the_cores_packing() {
        use crate::Ladder;
        use holon_draw::ladder::Band;
        use std::time::Instant;
        const N: u32 = 200;
        let band = Band { z_lo: -1.0, z_hi: 4.2, clamp: false };
        let mut ladder = Ladder::standard(band);
        let mut worst = (0.0f64, 0.0f64);
        println!("    z  rungs   marks  tiles() ms  passes() ms  packing ms");
        for k in 0..=40 {
            let z = 4.5 + k as f64 * 0.5;
            // the scenes the ladder draws at z, timed alone
            let live: Vec<usize> = holon_draw::ladder::visible(&ladder.rungs, z).into_iter().map(|x| x.0).collect();
            let mut marks = 0;
            let t = Instant::now();
            for _ in 0..N {
                marks = 0;
                for &i in &live {
                    marks += ladder.scenes[i].tiles(z, NOW, 1.0).iter().map(|t| t.marks.len()).sum::<usize>();
                }
            }
            let tiles = t.elapsed().as_secs_f64() * 1e3 / N as f64;
            let t = Instant::now();
            for _ in 0..N {
                std::hint::black_box(ladder.passes(z, 0.8, 0.42, 1200.0, 800.0, 1.0, NOW));
            }
            let passes = t.elapsed().as_secs_f64() * 1e3 / N as f64;
            println!("{z:5.1}  {:5}  {marks:6}  {tiles:10.3}  {passes:11.3}  {:10.3}", live.len(), passes - tiles);
            worst = (worst.0.max(tiles), worst.1.max(passes));
        }
        println!("worst: tiles() {:.3} ms, passes() {:.3} ms", worst.0, worst.1);
        if !cfg!(debug_assertions) {
            assert!(worst.0 < 1.0, "a scene's tiles() took {:.3} ms", worst.0);
        }
    }

    #[test]
    fn the_same_picture_every_time() {
        let (a, b) = (spiral(7, 500), spiral(7, 500));
        assert!(a.iter().zip(&b).all(|(x, y)| x.p == y.p && x.c == y.c));
        let (a, b) = (web(9, 10.0, 40, 300), web(9, 10.0, 40, 300));
        assert!(a.iter().zip(&b).all(|(x, y)| x.p == y.p));
        // a galaxy's arms trail: seen from +z it turns clockwise, so radius grows with angle
        // along an arm. Its bulge is in the middle and its disc is within radius 1.
        assert!(spiral(7, 3000).iter().all(|d| norm(d3(d.p)) < 1.2));
    }
}
