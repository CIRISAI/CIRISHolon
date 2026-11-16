//! One zoom from the vacuum to the universe (`holon_draw::ladder`), as scenes.
//!
//! A [`Scene`] draws one rung in that rung's own unit about its own origin. [`Ladder`] keeps
//! the rungs in order, tiles the whole range of `z` outside a host's band with them, and
//! builds one PASS per visible rung: its tiles, its camera, its cross-fade weight (put into
//! every tile's `PARAM_DIM` alpha), and a screen tile with the rung's name, a scale bar and
//! its status, so what is drawn always says where it comes from.
//!
//! The scenes live in `scenes/`, one module each. Adding a rung is adding a module and a line
//! in [`Ladder::standard`].
use holon_draw::ladder::{host_weight, rung_camera, visible, Band, Rung, Status, REF_PX, Z_MAX, Z_MIN};
use holon_draw::{Acuity, Camera, Frame, Scene as DrawScene, Tile, PARAM_DIM};

pub mod abi;
pub mod nav;
pub mod scenes;

/// One rung's scene.
pub trait Scene {
    /// The rung's name, unit and status. Its band of `z` is assigned by the [`Ladder`].
    fn name(&self) -> &'static str;
    /// log10 of the scene's unit in metres.
    fn unit(&self) -> i32;
    fn status(&self) -> Status;
    /// One line saying what the scene shows and where its numbers come from.
    fn source(&self) -> &'static str;
    /// The finest and coarsest `z` (log10 metres per device pixel) the scene is meant for (the
    /// ladder may stretch the outermost rungs to meet a host's band or the ends of the range).
    fn span(&self) -> (f64, f64);
    /// True when the boundary with the next FINER rung is a CUT: a hard switch with no
    /// cross-fade. A fade is honest only when the coarser picture is a function of the finer
    /// one; between two different theories (the 1+1D chain and the 3+1D gauge field) it would
    /// show frames that are states of neither, so the ladder cuts there (the CIRISOntology
    /// session's reading of its fibre results, as relayed, 2026-11-16).
    fn cut_below(&self) -> bool {
        false
    }
    /// True when the boundary with the next COARSER rung is a cut.
    fn cut_above(&self) -> bool {
        false
    }
    /// Advance by `dt` seconds of wall time (a simulated scene steps; others may animate).
    fn advance(&mut self, _dt: f64) {}
    /// The scene's tiles at zoom `z` (log10 metres per device pixel; marks in the scene's unit
    /// about its origin). A scene decides its detail by what a pixel resolves: a feature of
    /// size `s` metres spans `s / 10^z` pixels, and one under about a pixel is not drawn. `clock` is
    /// the host's time in seconds; `alpha` is the cross-fade weight, already applied by the
    /// ladder through each tile's settings, given in case a scene wants to skip detail.
    fn tiles(&mut self, z: f64, clock: f64, alpha: f64) -> Vec<Tile>;
}

/// One pass: the frame of one visible rung, with the camera it was built for.
pub struct Pass {
    pub rung: usize,
    pub weight: f64,
    pub camera: Camera,
    pub frame: Frame,
}

pub struct Ladder {
    pub scenes: Vec<Box<dyn Scene>>,
    pub rungs: Vec<Rung>,
    pub band: Band,
    /// The view's pan, in METRES along the screen's right and up at the focus. Metres, not a
    /// rung's units, so a pan carries across rungs; as the zoom widens the same pan is a smaller
    /// share of the view.
    pub pan: [f64; 2],
    /// A host that draws its own band on the same canvas and says what its own weight is
    /// ([`Host`]). `None`: the ladder works its share out itself from the band.
    pub host: Option<Host>,
    /// The `z` of each boundary between rungs that is a cut (see [`Scene::cut_below`]).
    pub cuts: Vec<f64>,
}

/// A host on whose canvas the ladder draws (the CEWP twin).
///
/// The host's `weight` is its own share of the picture at this frame (1 in its band, falling
/// to 0 past either edge by its own rule); the ladder draws at one minus that and does not
/// compute a fade of its own. With `far_eye` the rungs coarser than the band are seen at the
/// band's edge from [`FAR_EYE`] times the usual distance, which is an orthographic view to
/// within a hundredth of a percent, and the eye comes in to its usual place over the decade
/// beyond ([`Ladder::eye_factor`]): a host whose globe is drawn orthographically hands over to
/// an Earth of the same size and the same visible half.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Host {
    pub weight: f64,
    pub far_eye: bool,
}

/// How many times the usual distance the eye stands at the edge of a far-eyed host's band.
pub const FAR_EYE: f64 = 30.0;

/// An orbit camera as `holon_draw::ladder::rung_camera` makes it, with the eye `far` times as
/// far and the field of view narrowed to match, so that a device pixel still spans `10^z`
/// metres at the focus; and with NORTH (the rung's z axis) up at every pitch short of the
/// pole itself, where the core's orbit swaps its up axis three degrees early. `far` 1 at a
/// pitch under 87 degrees is the core's camera exactly.
#[allow(clippy::too_many_arguments)]
pub fn hosted_camera(r: &Rung, z: f64, pan: [f64; 3], yaw: f64, pitch: f64, fov_y: f64, width: f64, height: f64, far: f64) -> Camera {
    let mpp = 10f64.powf(z - r.unit as f64);
    let dist = far * 0.5 * mpp * REF_PX / (0.5 * fov_y).tan();
    let fov = 2.0 * (0.5 * mpp * height / dist).atan();
    let (cp, sp, cy, sy) = (pitch.cos(), pitch.sin(), yaw.cos(), yaw.sin());
    let eye = [pan[0] + dist * cp * cy, pan[1] + dist * cp * sy, pan[2] + dist * sp];
    let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let cross = |a: [f64; 3], b: [f64; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let f = [-cp * cy, -cp * sy, -sp];
    // the screen's right is level and a quarter turn on from the eye's meridian; at the pole
    // itself the same formula still holds (it does not depend on the pitch)
    let s = [-sy, cy, 0.0];
    let u = cross(s, f);
    let view = [s[0], s[1], s[2], -dot(s, eye), u[0], u[1], u[2], -dot(u, eye), -f[0], -f[1], -f[2], dot(f, eye), 0.0, 0.0, 0.0, 1.0];
    let (near, far_plane) = (dist * 0.02, dist * 20.0);
    let t = 1.0 / (0.5 * fov).tan();
    let aspect = width / height;
    let proj = [t / aspect, 0.0, 0.0, 0.0, 0.0, t, 0.0, 0.0, 0.0, 0.0, (far_plane + near) / (near - far_plane), 2.0 * far_plane * near / (near - far_plane), 0.0, 0.0, -1.0, 0.0];
    let mut m = [0.0; 16];
    for r in 0..4 {
        for c in 0..4 {
            m[4 * r + c] = (0..4).map(|k| proj[4 * r + k] * view[4 * k + c]).sum();
        }
    }
    Camera { m, width, height, scale: height * t / (2.0 * dist), focus: pan }
}

/// The world direction of the screen's right and up for an orbit at `yaw` and `pitch` (z up).
pub fn screen_axes(yaw: f64, pitch: f64) -> ([f64; 3], [f64; 3]) {
    let (cp, sp, cy, sy) = (pitch.cos(), pitch.sin(), yaw.cos(), yaw.sin());
    let f = [-cp * cy, -cp * sy, -sp]; // from the eye toward the focus
    let right = {
        let r = [f[1], -f[0], 0.0]; // f x z
        let l = (r[0] * r[0] + r[1] * r[1]).sqrt().max(1e-12);
        [r[0] / l, r[1] / l, 0.0]
    };
    let up = [right[1] * f[2] - right[2] * f[1], right[2] * f[0] - right[0] * f[2], right[0] * f[1] - right[1] * f[0]];
    (right, up)
}

impl Ladder {
    /// Lay the scenes end to end over `z`: each rung covers its scene's span, and the edges
    /// between neighbours are moved to meet halfway, so the rungs tile the range with no gap.
    /// Rungs wholly inside the host's band are dropped; the rung below the band ends at its
    /// lower edge and the rung above starts at its upper edge.
    pub fn new(scenes: Vec<Box<dyn Scene>>, band: Band) -> Ladder {
        let mut order: Vec<usize> = (0..scenes.len()).collect();
        order.sort_by(|&a, &b| scenes[a].span().0.total_cmp(&scenes[b].span().0));
        let scenes: Vec<Box<dyn Scene>> = {
            let mut s: Vec<Option<Box<dyn Scene>>> = scenes.into_iter().map(Some).collect();
            order.iter().map(|&i| s[i].take().unwrap()).collect()
        };
        let mut rungs: Vec<Rung> = scenes.iter().map(|s| Rung { name: s.name(), z_lo: s.span().0, z_hi: s.span().1, unit: s.unit(), status: s.status() }).collect();
        // meet halfway between neighbours
        for i in 1..rungs.len() {
            let mid = 0.5 * (rungs[i - 1].z_hi + rungs[i].z_lo);
            rungs[i - 1].z_hi = mid;
            rungs[i].z_lo = mid;
        }
        if let Some(r) = rungs.first_mut() {
            r.z_lo = Z_MIN;
        }
        if let Some(r) = rungs.last_mut() {
            r.z_hi = Z_MAX;
        }
        // the host's band: a rung whose own span lies below the band ends at its lower edge, one
        // whose span lies above starts at its upper edge, one inside it is never drawn (decided
        // on the scene's OWN span, so a ladder built from one half alone is clipped on the
        // right side; found by the outward scene worker)
        // an empty band means no host on the page: the rungs meet each other and nothing is clipped
        for (r, sc) in rungs.iter_mut().zip(scenes.iter()).filter(|_| band.z_hi > band.z_lo) {
            let mid = 0.5 * (sc.span().0 + sc.span().1);
            if mid < band.z_lo {
                r.z_hi = r.z_hi.min(band.z_lo);
            } else if mid > band.z_hi {
                r.z_lo = r.z_lo.max(band.z_hi);
            } else {
                r.z_hi = r.z_lo; // inside the host's band: never drawn
            }
            if r.z_hi < r.z_lo {
                r.z_hi = r.z_lo;
            }
        }
        let cuts = (1..rungs.len()).filter(|&i| scenes[i].cut_below() || scenes[i - 1].cut_above()).map(|i| rungs[i].z_lo).collect();
        Ladder { scenes, rungs, band, pan: [0.0; 2], host: None, cuts }
    }

    /// The standard ladder: every scene of `scenes::micro` and `scenes::cosmos`, around a
    /// host's band.
    pub fn standard(band: Band) -> Ladder {
        let mut s = scenes::micro::inward();
        s.extend(scenes::cosmos::outward());
        Ladder::new(s, band)
    }

    pub fn advance(&mut self, dt: f64) {
        for s in self.scenes.iter_mut() {
            s.advance(dt);
        }
    }

    /// The host's share at `z`: what a host on the canvas says it is, else the band's own fade.
    pub fn host_share(&self, z: f64) -> f64 {
        match self.host {
            Some(_) if self.band.clamp => 1.0,
            Some(h) if h.weight.is_finite() => h.weight.clamp(0.0, 1.0),
            _ => host_weight(&self.band, z),
        }
    }

    /// How many times the usual distance the eye stands for rung `i` at `z`: [`FAR_EYE`] from
    /// the upper edge of a far-eyed host's band to a quarter decade past it (where the host
    /// has half faded), easing to 1 by a whole decade past. 1 for every rung finer than the
    /// band, and with no such host.
    pub fn eye_factor(&self, i: usize, z: f64) -> f64 {
        match self.host {
            Some(h) if h.far_eye && self.band.z_hi > self.band.z_lo && self.rungs[i].z_lo >= self.band.z_hi => {
                let t = ((z - self.band.z_hi - 0.25) / 0.75).clamp(0.0, 1.0);
                FAR_EYE.powf(1.0 - t * t * (3.0 - 2.0 * t))
            }
            _ => 1.0,
        }
    }

    /// The passes to draw at `z`, coarsest scale first is irrelevant: each pass clears depth.
    #[allow(clippy::too_many_arguments)]
    pub fn passes(&mut self, z: f64, yaw: f64, pitch: f64, width: f64, height: f64, dpr: f64, clock: f64) -> Vec<Pass> {
        let host = self.host_share(z);
        let mut out = Vec::new();
        if host >= 1.0 {
            return out;
        }
        let mut live: Vec<(usize, f64)> = visible(&self.rungs, z).into_iter().filter(|(i, _)| self.rungs[*i].z_hi > self.rungs[*i].z_lo).collect();
        // at a cut, no fade: only the rung whose span holds z is drawn, at full weight
        if live.len() == 2 && self.cuts.iter().any(|&c| (c - self.rungs[live[1].0].z_lo).abs() < 1e-12) {
            live.retain(|(i, _)| self.rungs[*i].z_lo <= z && z < self.rungs[*i].z_hi);
            for l in live.iter_mut() {
                l.1 = 1.0;
            }
        }
        let total: f64 = live.iter().map(|x| x.1).sum::<f64>().max(1e-12);
        for (i, w) in live {
            // the ladder's share is (1 - host), split between its visible rungs
            let weight = (1.0 - host) * w / total;
            let r = self.rungs[i].clone();
            let (rt, up) = screen_axes(yaw, pitch);
            let k = 10f64.powi(-r.unit); // metres to the rung's units
            let pan = [0, 1, 2].map(|i| (self.pan[0] * rt[i] + self.pan[1] * up[i]) * k);
            let cam = match self.host {
                Some(_) => hosted_camera(&r, z, pan, yaw, pitch, std::f64::consts::FRAC_PI_4, width, height, self.eye_factor(i, z)),
                None => rung_camera(&r, z, pan, yaw, pitch, std::f64::consts::FRAC_PI_4, width, height),
            };
            let mut tiles = self.scenes[i].tiles(z, clock, weight);
            for t in tiles.iter_mut() {
                t.params[PARAM_DIM + 3] *= weight as f32;
            }
            let _ = dpr;
            // the pixel is the acuity: a tile whose detail spans under a pixel is drawn coarse
            let frame = DrawScene { tiles, modal: None }.frame(&cam, 1, &Acuity { eps_px: 1.0 }, u64::MAX).unwrap_or(Frame { batches: Vec::new(), marks: Vec::new(), fine: 0, coarse: 0, stale: 0, modal: None, remainder_px: 0.0 });
            out.push(Pass { rung: i, weight, camera: cam, frame });
        }
        out
    }

    /// The caption of the rung that carries most weight at `z`: its name, its status, its
    /// source line, and the scale bar's length as words (for example "1 nm").
    /// The fifth field is the scale bar's length in device pixels.
    pub fn caption(&self, z: f64, height: f64) -> Option<(&'static str, &'static str, &'static str, String, f64)> {
        let best = visible(&self.rungs, z).into_iter().filter(|(i, _)| self.rungs[*i].z_hi > self.rungs[*i].z_lo).max_by(|a, b| a.1.total_cmp(&b.1))?;
        let s = &self.scenes[best.0];
        let (px, name) = scale_bar(z, height);
        Some((s.name(), s.status().label(), s.source(), name, px))
    }
}

/// A scale bar at `z` (log10 metres per device pixel) on a viewport `height` pixels high: the
/// largest 1, 2 or 5 times a power of ten that is at most a fifth of the view, its length in
/// pixels, and its name.
pub fn scale_bar(z: f64, height: f64) -> (f64, String) {
    let z = z + height.log10(); // the view's height in metres, as a logarithm
    const LY: f64 = 9.4607e15;
    let target = 10f64.powf(z) / 5.0;
    // a round number of light years beyond a tenth of one, else of metres
    let unit = if target >= 0.1 * LY { LY } else { 1.0 };
    let t = target / unit;
    let e = t.log10().floor();
    let base = 10f64.powf(e);
    let m = if t >= 5.0 * base { 5.0 } else if t >= 2.0 * base { 2.0 } else { 1.0 };
    let metres = m * base * unit;
    let px = metres / 10f64.powf(z) * height;
    (px, name_length(metres))
}

/// A length in words, in the unit that reads best: from yoctometres to light years.
pub fn name_length(m: f64) -> String {
    const LY: f64 = 9.4607e15;
    if m >= 0.1 * LY {
        let ly = m / LY;
        for (u, n) in [(1e9, " billion"), (1e6, " million")] {
            if ly >= u {
                return format!("{}{} light years", trim(ly / u), n);
            }
        }
        return format!("{} light year{}", trim(ly), if ly == 1.0 { "" } else { "s" });
    }
    let units = [(1e24, "Ym"), (1e21, "Zm"), (1e18, "Em"), (1e15, "Pm"), (1e12, "Tm"), (1e9, "Gm"), (1e6, "Mm"), (1e3, "km"), (1.0, "m"), (1e-2, "cm"), (1e-3, "mm"), (1e-6, "µm"), (1e-9, "nm"), (1e-12, "pm"), (1e-15, "fm"), (1e-18, "am"), (1e-21, "zm"), (1e-24, "ym")];
    for (u, n) in units {
        if m >= u {
            return format!("{} {}", trim(m / u), n);
        }
    }
    format!("{:.0e} m", m)
}

fn trim(x: f64) -> String {
    let s = if x >= 100.0 { format!("{:.0}", x) } else if x >= 10.0 { format!("{:.1}", x) } else { format!("{:.2}", x) };
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy(&'static str, f64, f64, i32);
    impl Scene for Dummy {
        fn name(&self) -> &'static str {
            self.0
        }
        fn unit(&self) -> i32 {
            self.3
        }
        fn status(&self) -> Status {
            Status::Illustration
        }
        fn source(&self) -> &'static str {
            "test"
        }
        fn span(&self) -> (f64, f64) {
            (self.1, self.2)
        }
        fn tiles(&mut self, _z: f64, _c: f64, _a: f64) -> Vec<Tile> {
            vec![Tile { id: 1, origin: [0.0; 3], version: 1, layer: 0, screen: false, params: holon_draw::PARAMS_NEUTRAL, marks: vec![holon_draw::Instance::new(holon_draw::Kind::Sphere, [0.0; 3], [1.0, 0.0, 0.0]).tagged(1)] }]
        }
    }

    fn ladder(clamp: bool) -> Ladder {
        let s: Vec<Box<dyn Scene>> = vec![Box::new(Dummy("far", 5.0, 11.0, 11)), Box::new(Dummy("vacuum", -38.0, -19.0, -25)), Box::new(Dummy("atoms", -14.0, -11.0, -10)), Box::new(Dummy("drop", -11.0, -3.0, -4))];
        Ladder::new(s, Band { z_lo: -1.0, z_hi: 4.2, clamp })
    }

    #[test]
    fn the_rungs_tile_the_whole_range_outside_the_host_with_no_gap() {
        let l = ladder(false);
        let names: Vec<&str> = l.rungs.iter().map(|r| r.name).collect();
        assert_eq!(names, ["vacuum", "atoms", "drop", "far"]);
        assert_eq!(l.rungs[0].z_lo, Z_MIN);
        assert_eq!(l.rungs[3].z_hi, Z_MAX);
        for w in l.rungs.windows(2) {
            assert!(w[0].z_hi <= w[1].z_lo + 1e-12);
        }
        // the drop rung ends where the host begins, the far rung starts where it ends
        assert_eq!(l.rungs[2].z_hi, -1.0);
        assert_eq!(l.rungs[3].z_lo, 4.2);
        // every z outside the host's band is covered by the ladder or the host, summing to 1
        for k in 0..620 {
            let z = Z_MIN + k as f64 * 0.1;
            let host = host_weight(&l.band, z);
            let rungs: f64 = visible(&l.rungs, z).iter().map(|x| x.1).sum();
            if z > Z_MIN + 0.3 && z < Z_MAX - 0.3 {
                assert!((host + rungs - 1.0).abs() < 1e-9 || (host == 1.0), "{z}: host {host} rungs {rungs}");
            }
        }
    }

    struct Cut(&'static str, f64, f64, bool);
    impl Scene for Cut {
        fn name(&self) -> &'static str {
            self.0
        }
        fn unit(&self) -> i32 {
            -15
        }
        fn status(&self) -> Status {
            Status::Simulated
        }
        fn source(&self) -> &'static str {
            "test"
        }
        fn span(&self) -> (f64, f64) {
            (self.1, self.2)
        }
        fn cut_below(&self) -> bool {
            self.3
        }
        fn tiles(&mut self, _z: f64, _c: f64, _a: f64) -> Vec<Tile> {
            Vec::new()
        }
    }

    #[test]
    fn at_a_cut_one_rung_is_drawn_whole_and_elsewhere_neighbours_fade() {
        // a field below, a chain above it (a cut between: a different theory), a nucleus above
        // the chain (a fade)
        let s: Vec<Box<dyn Scene>> = vec![Box::new(Cut("field", -20.0, -19.0, false)), Box::new(Cut("chain", -19.0, -18.0, true)), Box::new(Cut("nucleus", -18.0, -16.0, false))];
        let mut l = Ladder::new(s, Band { z_lo: 0.0, z_hi: 0.0, clamp: false });
        assert_eq!(l.cuts, vec![-19.0]);
        for (z, want) in [(-19.1, vec![("field", 1.0)]), (-18.95, vec![("chain", 1.0)]), (-19.0, vec![("chain", 1.0)])] {
            let p = l.passes(z, 0.4, 0.3, 800.0, 600.0, 1.0, 0.0);
            let got: Vec<(&str, f64)> = p.iter().map(|x| (l.rungs[x.rung].name, x.weight)).collect();
            assert_eq!(got, want, "at {z}");
        }
        // the chain to nucleus boundary still fades
        assert_eq!(l.passes(-18.0, 0.4, 0.3, 800.0, 600.0, 1.0, 0.0).len(), 2);
    }

    #[test]
    fn a_pan_moves_the_view_by_its_metres_across_the_screen() {
        let mut l = ladder(false);
        let (w, h) = (800.0, 600.0);
        let z = -12.5; // the atoms rung: a pixel is 10^-12.5 m
        let before = l.passes(z, 0.5, 0.3, w, h, 1.0, 0.0);
        let p0 = before[0].camera.project([0.0; 3]).unwrap();
        // pan 100 pixels' worth of metres to the right: the scene's origin moves 100 px left
        l.pan = [100.0 * 10f64.powf(z), 0.0];
        let after = l.passes(z, 0.5, 0.3, w, h, 1.0, 0.0);
        let p1 = after[0].camera.project([0.0; 3]).unwrap();
        assert!((p0[0] - p1[0] - 100.0).abs() < 1e-6 && (p0[1] - p1[1]).abs() < 1e-6, "{p0:?} {p1:?}");
    }

    #[test]
    fn with_no_host_the_rungs_meet_each_other_and_cover_everything() {
        let s: Vec<Box<dyn Scene>> = vec![Box::new(Dummy("far", 5.0, 11.0, 11)), Box::new(Dummy("vacuum", -38.0, -19.0, -25)), Box::new(Dummy("atoms", -14.0, -11.0, -10)), Box::new(Dummy("drop", -11.0, -3.0, -4))];
        let mut l = Ladder::new(s, Band { z_lo: 0.0, z_hi: 0.0, clamp: false });
        assert_eq!(l.rungs[2].z_hi, l.rungs[3].z_lo, "the drop meets the far rung");
        for k in 0..600 {
            let z = Z_MIN + 0.3 + k as f64 * 0.1;
            if z > Z_MAX - 0.3 {
                break;
            }
            assert!(!l.passes(z, 0.5, 0.3, 800.0, 600.0, 1.0, 0.0).is_empty(), "nothing drawn at {z}");
        }
    }

    #[test]
    fn with_the_switch_on_nothing_is_drawn_and_off_a_pass_per_visible_rung() {
        let mut on = ladder(true);
        assert!(on.passes(-12.5, 0.5, 0.3, 800.0, 600.0, 1.0, 0.0).is_empty());
        let mut off = ladder(false);
        let p = off.passes(-12.5, 0.5, 0.3, 800.0, 600.0, 1.0, 0.0);
        assert_eq!(p.len(), 1);
        assert_eq!(off.rungs[p[0].rung].name, "atoms");
        assert!((p[0].weight - 1.0).abs() < 1e-12);
        // at the edge between two rungs, two passes whose weights sum to 1
        let edge = off.rungs[1].z_hi;
        let p = off.passes(edge, 0.5, 0.3, 800.0, 600.0, 1.0, 0.0);
        assert_eq!(p.len(), 2);
        assert!((p.iter().map(|x| x.weight).sum::<f64>() - 1.0).abs() < 1e-9);
        // the weight is in every world batch's alpha
        assert!(p[0].frame.batches.iter().all(|b| (b.params[PARAM_DIM + 3] as f64 - p[0].weight).abs() < 1e-6));
    }

    #[test]
    fn a_host_on_the_canvas_sets_the_ladders_share_and_sees_its_globe_orthographically() {
        use holon_draw::ladder::Band;
        // the twin's band: street 0.1 m a pixel, globe at 10^4.2 m a pixel
        let band = Band { z_lo: -1.0, z_hi: 4.2, clamp: false };
        let mut l = Ladder::standard(band);
        let earth = l.rungs.iter().position(|r| r.name == "the Earth and the Moon").unwrap();
        assert_eq!(l.rungs[earth].z_lo, 4.2);
        // the host's weight falls PAST the edge (the ladder's own fade is centred on it): at the
        // edge the host says 1 and nothing is drawn; a quarter decade past, a half each
        l.host = Some(Host { weight: 1.0, far_eye: true });
        assert!(l.passes(4.2, 0.3, 0.5, 780.0, 1688.0, 2.0, 0.0).is_empty());
        l.host = Some(Host { weight: 0.5, far_eye: true });
        let p = l.passes(4.45, 0.3, 0.5, 780.0, 1688.0, 2.0, 0.0);
        assert_eq!((p.len(), p[0].rung, p[0].weight), (1, earth, 0.5));
        assert!(p[0].frame.batches.iter().filter(|b| !b.screen).all(|b| b.params[PARAM_DIM + 3] == 0.5));
        l.host = Some(Host { weight: 0.25, far_eye: true });
        assert_eq!(l.passes(4.3, 0.3, 0.5, 780.0, 1688.0, 2.0, 0.0)[0].weight, 0.75);
        // the same going in, on the finest rung below the street
        let p = l.passes(-1.2, 0.3, 0.5, 780.0, 1688.0, 2.0, 0.0);
        assert!((p.iter().map(|x| x.weight).sum::<f64>() - 0.75).abs() < 1e-12);
        // with the switch on, nothing, whatever the host says
        l.band.clamp = true;
        assert!(l.passes(4.6, 0.3, 0.5, 780.0, 1688.0, 2.0, 0.0).is_empty());
        l.band.clamp = false;

        // THE EARTH'S SIZE AT THE HAND-OFF. The twin's globe is orthographic: the Earth's limb
        // is 6,371,000 / 10^z pixels from the centre. Seen in perspective from the usual
        // distance the limb is some 6 % wider; from FAR_EYE times as far it is within a tenth
        // of a pixel (a third of one on the 4K window), on a phone and on a desk (390 x 844 at 2 and 1440 x 900 at 1, where the
        // twin's globe has radius k = 0.86 * min(w, h) / 2 CSS pixels at zoom 1, and on a
        // 4K window, the largest globe)
        let limb = |cam: &Camera, yaw: f64, pitch: f64| {
            // the widest any point of the sphere is drawn from the centre, in pixels
            let (rt, up) = screen_axes(yaw, pitch);
            let eye = [pitch.cos() * yaw.cos(), pitch.cos() * yaw.sin(), pitch.sin()];
            let c = cam.project([0.0; 3]).unwrap();
            (0..=2000).map(|k| {
                let th = std::f64::consts::PI * k as f64 / 2000.0;
                let p = [0, 1, 2].map(|i| 6.371 * (th.sin() * (0.6 * rt[i] + 0.8 * up[i]) + th.cos() * eye[i]));
                let q = cam.project(p).unwrap();
                ((q[0] - c[0]).powi(2) + (q[1] - c[1]).powi(2)).sqrt()
            }).fold(0.0, f64::max)
        };
        for (w, h, scale) in [(390.0, 844.0, 2.0), (1440.0, 900.0, 1.0), (3840.0, 2160.0, 1.0)] {
            let k = 0.86 * f64::min(w, h) / 2.0 * scale; // device pixels
            let z = (6.371e6 / k).log10();
            let mut l = Ladder::standard(Band { z_lo: -1.0, z_hi: z, clamp: false });
            l.host = Some(Host { weight: 0.5, far_eye: true });
            let (yaw, pitch) = (-1.3, 0.7);
            for dz in [0.0, 0.25] {
                let p = l.passes(z + dz, yaw, pitch, w * scale, h * scale, scale, 0.0);
                let want = k / 10f64.powf(dz);
                let got = limb(&p[0].camera, yaw, pitch);
                assert!((got - want).abs() < if w > 2000.0 { 0.5 } else { 0.1 }, "{w} x {h}: the limb at {got} px, the twin's at {want}");
                // the view's centre is the point of the Earth at longitude yaw, latitude pitch
                let c = p[0].camera.project([0.0; 3]).unwrap();
                let sub = p[0].camera.project([6.371 * pitch.cos() * yaw.cos(), 6.371 * pitch.cos() * yaw.sin(), 6.371 * pitch.sin()]).unwrap();
                assert!((sub[0] - c[0]).abs() < 1e-6 && (sub[1] - c[1]).abs() < 1e-6);
                // and north is up: the pole is straight above the centre
                let pole = p[0].camera.project([0.0, 0.0, 6.371]).unwrap();
                assert!((pole[0] - c[0]).abs() < 1e-6 && pole[1] < c[1]);
            }
            // without the far eye the same Earth would be drawn too large by more than a pixel
            l.host = Some(Host { weight: 0.5, far_eye: false });
            let p = l.passes(z, yaw, pitch, w * scale, h * scale, scale, 0.0);
            assert!(limb(&p[0].camera, yaw, pitch) - k > 1.0);
            // the eye is back at its usual place a decade past the edge, and was never moved
            // for the rungs finer than the band
            l.host = Some(Host { weight: 0.0, far_eye: true });
            let e = l.rungs.iter().position(|r| r.name == "the Earth and the Moon").unwrap();
            assert_eq!((l.eye_factor(e, z), l.eye_factor(e, z + 0.25), l.eye_factor(e, z + 1.0), l.eye_factor(0, -3.0)), (FAR_EYE, FAR_EYE, 1.0, 1.0));
            assert!(l.eye_factor(e, z + 0.6) > 1.0 && l.eye_factor(e, z + 0.6) < FAR_EYE);
        }
        // at the usual distance and an ordinary pitch the hosted camera is the core's camera
        let r = Rung { name: "", z_lo: 0.0, z_hi: 9.0, unit: 6, status: Status::Catalogue };
        let (a, b) = (hosted_camera(&r, 5.0, [1.0, 2.0, 3.0], 0.7, -0.4, 0.785, 800.0, 600.0, 1.0), rung_camera(&r, 5.0, [1.0, 2.0, 3.0], 0.7, -0.4, 0.785, 800.0, 600.0));
        assert!(a.m.iter().zip(&b.m).all(|(x, y)| (x - y).abs() <= 1e-12 * y.abs().max(1.0)) && a.scale == b.scale);
        // and at the pole north is still up (the pole is the centre; Greenwich's far meridian is above it)
        let c = hosted_camera(&r, 5.0, [0.0; 3], 0.0, std::f64::consts::FRAC_PI_2, 0.785, 800.0, 600.0, FAR_EYE);
        let (centre, away) = (c.project([0.0, 0.0, 6.371]).unwrap(), c.project([-3.0, 0.0, 5.0]).unwrap());
        assert!((centre[0] - 400.0).abs() < 1e-6 && (centre[1] - 300.0).abs() < 1e-6 && (away[0] - 400.0).abs() < 1e-6 && away[1] < 300.0);
    }

    #[test]
    fn a_scale_bar_reads_in_words_at_every_scale() {
        // z is metres per pixel: on a 1000-pixel viewport the view is 10^(z+3) metres high
        assert_eq!(scale_bar(-3.0, 1000.0).1, "20 cm");
        assert_eq!(scale_bar(-12.0, 1000.0).1, "200 pm");
        assert_eq!(scale_bar(-18.0, 1000.0).1, "200 am");
        assert_eq!(scale_bar(9.0, 1000.0).1, "200 Gm");
        assert_eq!(scale_bar(18.0, 1000.0).1, "20000 light years");
        assert_eq!(scale_bar(23.9, 1000.0).1, "10 billion light years");
        // the same z on a window twice as tall: the same metres per pixel, so the same bar is
        // the same number of pixels long; the taller window just shows more of the scene
        assert_eq!(scale_bar(-3.0, 2000.0).1, "20 cm");
        assert!((scale_bar(-3.0, 2000.0).0 - scale_bar(-3.0, 1000.0).0).abs() < 1e-9);
        let (px, _) = scale_bar(0.3, 900.0);
        assert!(px > 900.0 / 10.0 && px <= 900.0 / 5.0 + 1e-9);
    }
}
