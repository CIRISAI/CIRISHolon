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
use holon_draw::ladder::{host_weight, rung_camera, visible, Band, Rung, Status, Z_MAX, Z_MIN};
use holon_draw::{Acuity, Camera, Frame, Scene as DrawScene, Tile, PARAM_DIM};

pub mod abi;
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
        for (r, sc) in rungs.iter_mut().zip(scenes.iter()) {
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
        Ladder { scenes, rungs, band }
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

    /// The passes to draw at `z`, coarsest scale first is irrelevant: each pass clears depth.
    #[allow(clippy::too_many_arguments)]
    pub fn passes(&mut self, z: f64, yaw: f64, pitch: f64, width: f64, height: f64, dpr: f64, clock: f64) -> Vec<Pass> {
        let host = host_weight(&self.band, z);
        let mut out = Vec::new();
        if host >= 1.0 {
            return out;
        }
        let live: Vec<(usize, f64)> = visible(&self.rungs, z).into_iter().filter(|(i, _)| self.rungs[*i].z_hi > self.rungs[*i].z_lo).collect();
        let total: f64 = live.iter().map(|x| x.1).sum::<f64>().max(1e-12);
        for (i, w) in live {
            // the ladder's share is (1 - host), split between its visible rungs
            let weight = (1.0 - host) * w / total;
            let r = self.rungs[i].clone();
            let cam = rung_camera(&r, z, [0.0; 3], yaw, pitch, std::f64::consts::FRAC_PI_4, width, height);
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
