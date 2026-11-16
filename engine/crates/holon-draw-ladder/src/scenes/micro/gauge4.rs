//! The gluon field: quenched SU(3) on a four-dimensional lattice, computed by this
//! repository's own engine (`holon-gauge4`) ahead of time and read from its record.
//!
//! WHAT IS DRAWN. One three-dimensional slice, at fixed Euclidean time, of ONE configuration
//! of the Euclidean ensemble at `beta = 6.0`. On each lattice site of the slice a ball: its
//! size is the clover action density there, relative to the slice's largest, and its colour
//! the clover topological charge density (warm where positive, cool where negative, pale
//! near zero). Both are gauge invariant; nothing gauge variant is drawn (a test builds the
//! marks from a configuration and from its random gauge transform and requires the same
//! bytes). The picture is in the style of D. B. Leinweber's visualisations of the QCD vacuum
//! (CSSM, Adelaide: arXiv:hep-lat/0004025; F. D. R. Bonnet, P. Fitzhenry, D. B. Leinweber,
//! M. R. Stanford and A. G. Williams, Phys. Rev. D 62, 094509 (2000), arXiv:hep-lat/0001018).
//! The Polyakov loop is in the record and is not drawn.
//!
//! WHAT IT IS NOT. Quenched: no quarks in the weight, so no sea quarks. A configuration is a
//! sample of a Euclidean ensemble, a static field, not a moment in a hadron's life; it is not
//! the proton. The methods are standard and are credited in `holon-gauge4`.
//!
//! THE ZOOM IS THE FLOW TIME (the CIRISOntology session's advice, which is the standard
//! reading of the Wilson flow: M. Lüscher, JHEP 08 (2010) 071, arXiv:1006.4518). At flow time
//! `t` the field is the original one averaged over a radius of about `sqrt(8 t)`. A pixel
//! here is a hundredth of a lattice spacing, and a lattice holds nothing finer than its
//! spacing, so the radius cannot be one pixel; it is held at [`SMOOTH_PX`] device pixels
//! instead. Zooming out therefore shows a smoother field, and the coarser picture is a
//! function of the finer one. The flow time moves in steps of a tenth ([`LEVELS_PER_DECADE`]
//! to the decade), interpolated between the record's eleven stored times, and the caption
//! says which is on screen. Zoomed in past the lattice spacing the field is the unflowed one,
//! the links appear, and there is nothing finer: the record ends.
//!
//! THE SCALE is the literature's: `a = 0.0931 fm` at `beta = 6.0` (`holon_gauge4`'s notes).
//! The lattice is periodic, and is drawn with its largest lump at the centre.
//!
//! This rung does not cross-fade with the 1+1D chain's rungs above it: they are different
//! theories and neither is a function of the other. The boundary is a cut on both sides.
use super::*;
use holon_draw::ladder::{Status, Z_MIN};
use holon_gauge4::{Record, A_FM_BETA_6};
use std::sync::{Mutex, OnceLock};

/// Tenths of a femtometre.
pub const UNIT: i32 = -16;
/// The rung's span of `z`: from the end of the ladder to a view a femtometre high, where the
/// 1.12 fm box fills it.
pub const SPAN: (f64, f64) = (Z_MIN, -15.0 - VIEW_DECADES);
/// The smoothing radius `sqrt(8 t)` on screen, in device pixels, at every zoom.
pub const SMOOTH_PX: f64 = 416.0;
/// The flow time is drawn at this many levels to the decade (so the caption can name it).
pub const LEVELS_PER_DECADE: f64 = 24.0;
/// Below this flow time the unflowed field is drawn.
pub const T_FLOOR: f64 = 0.008;
/// A site is drawn when its action density is over this share of the slice's largest.
pub const THRESHOLD: f32 = 0.42;
/// The largest ball's radius, in lattice spacings.
pub const BALL: f32 = 0.62;

/// The record shipped with the engine (`holon-gauge4/data`, written by its `export` example).
static BYTES: &[u8] = include_bytes!("../../../../holon-gauge4/data/gluon_b600_l12.hg4f");

fn shipped() -> &'static Record {
    static R: OnceLock<Record> = OnceLock::new();
    R.get_or_init(|| Record::decode(BYTES).expect("the shipped record decodes"))
}

pub struct Gluon {
    rec: Record,
    /// The site drawn at the centre: where the action density is largest at the longest flow.
    centre: [usize; 3],
    /// A key for the caption cache: which record this is.
    key: u64,
    /// The flow time on screen, and the caption that names it.
    pub t: f64,
    caption: &'static str,
    /// Scratch: the interpolated fields.
    e: Vec<f32>,
    q: Vec<f32>,
}

/// The flow time drawn at `z`: the smoothing radius `sqrt(8 t)` is [`SMOOTH_PX`] pixels,
/// stepped to a level, no more than the record's longest flow and nothing under
/// [`T_FLOOR`].
pub fn flow_time(z: f64, t_max: f64) -> f64 {
    let r = SMOOTH_PX * 10f64.powf(z) / (A_FM_BETA_6 * 1e-15); // in lattice spacings
    let t = r * r / 8.0;
    if t < T_FLOOR {
        return 0.0;
    }
    // levels below the longest flow: t_max * 10^(-k / LEVELS_PER_DECADE)
    let k = ((t_max / t).log10() * LEVELS_PER_DECADE).round().max(0.0);
    t_max * 10f64.powf(-k / LEVELS_PER_DECADE)
}

impl Gluon {
    /// The shipped record's rung.
    pub fn new() -> Gluon {
        Gluon::from_record(shipped().clone())
    }
    /// A rung over any record at `beta = 6.0` (the tests draw a small one of their own).
    pub fn from_record(rec: Record) -> Gluon {
        assert!(rec.beta == 6.0 && rec.snapshots.len() >= 2, "a record at beta 6.0 with at least two flow times");
        let last = rec.snapshots.last().unwrap();
        let top = (0..last.action.len()).max_by(|&a, &b| last.action[a].total_cmp(&last.action[b])).unwrap();
        let centre = [top % rec.dims[0], (top / rec.dims[0]) % rec.dims[1], top / (rec.dims[0] * rec.dims[1])];
        // FNV-1a over the provenance and the first snapshot's fields
        let mut key = 0xcbf2_9ce4_8422_2325u64;
        for b in rec.provenance.bytes().chain(rec.snapshots[0].action.iter().flat_map(|x| x.to_le_bytes())) {
            key = (key ^ b as u64).wrapping_mul(0x0100_0000_01b3);
        }
        let n = rec.volume3();
        let mut g = Gluon { rec, centre, key, t: -1.0, caption: "", e: vec![0.0; n], q: vec![0.0; n] };
        g.set_flow(g.t_max());
        g
    }
    pub fn t_max(&self) -> f64 {
        self.rec.snapshots.last().unwrap().t
    }
    pub fn record(&self) -> &Record {
        &self.rec
    }
    /// The lattice spacing in this rung's unit.
    pub fn spacing() -> f32 {
        (A_FM_BETA_6 * 1e-15 / 10f64.powi(UNIT)) as f32
    }
    /// Interpolate the stored fields to flow time `t` (linear in `t` between the two stored
    /// times that bracket it) and name it in the caption. Returns the topological charge of
    /// the whole lattice at `t`, interpolated the same way.
    fn set_flow(&mut self, t: f64) -> f64 {
        let s = &self.rec.snapshots;
        let i = s.partition_point(|x| x.t <= t).clamp(1, s.len() - 1);
        let (a, b) = (&s[i - 1], &s[i]);
        let w = ((t - a.t) / (b.t - a.t)).clamp(0.0, 1.0);
        let q_total = a.q_total + (b.q_total - a.q_total) * w;
        if t == self.t {
            return q_total;
        }
        let w = w as f32;
        for k in 0..self.e.len() {
            self.e[k] = a.action[k] + (b.action[k] - a.action[k]) * w;
            self.q[k] = a.charge[k] + (b.charge[k] - a.charge[k]) * w;
        }
        self.t = t;
        self.caption = caption(self.key, &self.rec, t, q_total);
        q_total
    }
    /// The position of site `(i, j, k)` in the rung's unit: the lattice is periodic, and is
    /// turned so that `centre` is at the origin.
    fn place(&self, i: usize, j: usize, k: usize) -> [f32; 3] {
        let a = Self::spacing();
        let d = self.rec.dims;
        let wrap = |x: usize, c: usize, n: usize| ((x + n + n / 2 - c) % n) as f32 - (n / 2) as f32;
        [wrap(i, self.centre[0], d[0]) * a, wrap(j, self.centre[1], d[1]) * a, wrap(k, self.centre[2], d[2]) * a]
    }
}

impl Default for Gluon {
    fn default() -> Self {
        Self::new()
    }
}

/// The caption for a record at a flow time, kept for the life of the process (a scene's
/// source line is a `&'static str`; there are at most a few dozen levels a record).
fn caption(key: u64, rec: &Record, t: f64, q: f64) -> &'static str {
    static MADE: Mutex<Vec<(u64, u64, &'static str)>> = Mutex::new(Vec::new());
    let mut made = MADE.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(c) = made.iter().find(|c| c.0 == key && c.1 == t.to_bits()) {
        return c.2;
    }
    let l = rec.dims[0];
    let r = (8.0 * t).sqrt();
    let flow = if t > 0.0 {
        format!("FLOW TIME t = {t:.3} a^2: the field averaged over a radius sqrt(8t) = {r:.2} lattice spacings = {:.2} fm, held at {SMOOTH_PX:.0} pixels, so zooming out smooths it (the coarser picture is a function of the finer). Topological charge of the whole lattice at this flow time: {q:+.2}.", r * A_FM_BETA_6)
    } else {
        format!("FLOW TIME t = 0: the configuration as generated, rough at the scale of the lattice (topological charge sum {q:+.2}, not yet near an integer). Zoom out and the Wilson flow smooths it. One SU(3) matrix sits on each link (the grey lines); a link has no inside, and nothing in this repository computes anything finer: the record ends here.")
    };
    let s = format!(
        "{flow} WHAT THIS IS: one configuration of a Euclidean ensemble of QUENCHED SU(3) gauge theory (Wilson action, no sea quarks) on a {l}^4 lattice at beta = 6.0, a = 0.093 fm, a box {:.2} fm across; one three-dimensional slice at fixed Euclidean time. A ball on a lattice site: its size is the action density there, its colour the topological charge density (warm positive, cool negative, pale near zero); both gauge invariant. A static gluon field, a sample, not a moment in a hadron's life, and NOT the proton. Computed ahead of time by this repository's own engine (holon-gauge4) and read from its record; the methods are standard and not ours: Cabibbo-Marinari heat bath (Creutz; Kennedy-Pendleton) with overrelaxation, Lüscher's Wilson flow, the scale from Sommer's r0, the picture after Leinweber's (CSSM Adelaide). The box is periodic and drawn with its largest lump at the centre.",
        l as f64 * A_FM_BETA_6
    );
    let s: &'static str = Box::leak(s.into_boxed_str());
    made.push((key, t.to_bits(), s));
    s
}

const PALE: [f32; 3] = [226.0, 220.0, 204.0];
const WARM: [f32; 3] = [240.0, 96.0, 44.0];
const COOL: [f32; 3] = [56.0, 132.0, 240.0];
const FRAME: [u8; 4] = [148, 163, 184, 255];

impl Scene for Gluon {
    fn name(&self) -> &'static str {
        "the gluon field: quenched SU(3) on a 4D lattice"
    }
    fn unit(&self) -> i32 {
        UNIT
    }
    fn status(&self) -> Status {
        Status::Measured
    }
    fn source(&self) -> &'static str {
        self.caption
    }
    fn span(&self) -> (f64, f64) {
        SPAN
    }
    /// Above is the 1+1D chain: another theory. A cut, not a fade.
    fn cut_above(&self) -> bool {
        true
    }
    fn tiles(&mut self, z: f64, _clock: f64, _alpha: f64) -> Vec<Tile> {
        let px = 10f64.powf(z - UNIT as f64) as f32; // this rung's units to the pixel
        self.set_flow(flow_time(z, self.t_max()));
        let a = Self::spacing();
        let d = self.rec.dims;
        let (emax, qmax) = (self.e.iter().fold(f32::MIN_POSITIVE, |m, x| m.max(*x)), self.q.iter().fold(f32::MIN_POSITIVE, |m, x| m.max(x.abs())));
        // a ball is never drawn wider than 90 pixels: zoomed in past the lattice spacing it
        // marks its site, and its size on the lattice's scale is not a size of anything
        let cap = 90.0 * px;
        let mut balls = Vec::with_capacity(self.e.len() / 2);
        for k in 0..d[2] {
            for j in 0..d[1] {
                for i in 0..d[0] {
                    let s = i + d[0] * (j + d[1] * k);
                    let en = self.e[s] / emax;
                    if en < THRESHOLD {
                        continue;
                    }
                    let r = (BALL * a * ((en - THRESHOLD) / (1.0 - THRESHOLD)).sqrt()).min(cap);
                    // the pixel is the acuity: a ball under a pixel across is not drawn
                    if r < 0.5 * px {
                        continue;
                    }
                    let qn = self.q[s] / qmax;
                    let (hue, w) = (if qn >= 0.0 { WARM } else { COOL }, qn.abs().sqrt());
                    let c = [0, 1, 2].map(|n| (PALE[n] + (hue[n] - PALE[n]) * w) as u8);
                    balls.push(sphere(self.place(i, j, k), r, [c[0], c[1], c[2], 255]));
                }
            }
        }
        // the box: its twelve edges, half a spacing beyond the outermost sites
        let h = [0, 1, 2].map(|n| 0.5 * d[n] as f32 * a);
        let o = [0, 1, 2].map(|n| if d[n] % 2 == 0 { -0.5 * a } else { 0.0 });
        let w = (0.01 * a).clamp(1.0 * px, 2.0 * px);
        let mut lines = Vec::new();
        for n in 0..3 {
            let (u, v) = ((n + 1) % 3, (n + 2) % 3);
            for (su, sv) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let (mut p, mut q) = ([0.0f32; 3], [0.0f32; 3]);
                (p[n], q[n]) = (o[n] - h[n], o[n] + h[n]);
                (p[u], q[u]) = (o[u] + su * h[u], o[u] + su * h[u]);
                (p[v], q[v]) = (o[v] + sv * h[v], o[v] + sv * h[v]);
                lines.push(line(p, q, w, alpha(FRAME, 0.7)));
            }
        }
        // the links, once a spacing is over 150 pixels: from each site to its three forward
        // neighbours (the box is periodic; the links that wrap are not drawn)
        let shown = ramp((a / px) as f64, 150.0, 400.0);
        if shown > 0.0 {
            let c = alpha(FRAME, 0.55 * shown);
            let at = |i: usize, n: usize| (i as f32 - (d[n] / 2) as f32) * a;
            for k in 0..d[2] {
                for j in 0..d[1] {
                    for i in 0..d[0] {
                        let p = [at(i, 0), at(j, 1), at(k, 2)];
                        if i + 1 < d[0] {
                            lines.push(line(p, [p[0] + a, p[1], p[2]], w, c));
                        }
                        if j + 1 < d[1] {
                            lines.push(line(p, [p[0], p[1] + a, p[2]], w, c));
                        }
                        if k + 1 < d[2] {
                            lines.push(line(p, [p[0], p[1], p[2] + a], w, c));
                        }
                    }
                }
            }
        }
        vec![tile(1, 0, balls), thin(tile(2, 0, lines))]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ladder;
    use holon_draw::ladder::{rung_camera, Band, Rung};
    use holon_draw::{Acuity, Scene as DrawScene};
    use holon_gauge4::{Lattice, Rng, Snapshot};
    use std::time::Instant;

    /// A small configuration of the tests' own: 6^4 at beta 6.0, 40 sweeps from a hot start.
    fn small() -> Lattice {
        let mut l = Lattice::hot([6; 4], 4);
        let mut r = Rng::new(40);
        for _ in 0..40 {
            l.sweep(6.0, 2, &mut r);
        }
        l
    }
    const TIMES: [f64; 5] = [0.0, 0.03, 0.09, 0.25, 0.6];
    fn record(l: &Lattice) -> Record {
        Record::flowed(l, 6.0, 1, &TIMES, 0.01, "a test".into()).quantised()
    }
    /// Every byte the draw core would upload for the rung at `z`, and the marks themselves.
    fn drawn(g: &mut Gluon, z: f64) -> (Vec<Vec<u8>>, Vec<Instance>) {
        let tiles = g.tiles(z, 0.0, 1.0);
        let marks = tiles.iter().flat_map(|t| t.marks.clone()).collect();
        let r = Rung { name: "", z_lo: SPAN.0, z_hi: SPAN.1, unit: UNIT, status: Status::Measured };
        let cam = rung_camera(&r, z, [0.0; 3], 0.7, 0.4, std::f64::consts::FRAC_PI_4, 1200.0, 900.0);
        let f = DrawScene { tiles, modal: None }.frame(&cam, 1, &Acuity { eps_px: 1.0 }, u64::MAX).unwrap();
        (f.batches.into_iter().map(|b| b.bytes).collect(), marks)
    }
    const ZS: [f64; 7] = [-18.0, -18.2, -18.45, -18.7, -19.0, -19.4, -19.9];

    #[test]
    fn the_drawn_marks_do_not_change_under_a_random_gauge_transformation_and_a_link_trace_would() {
        let l = small();
        let mut t = l.clone();
        t.gauge_transform(&l.random_gauge(77));
        assert!(l.distance(&t) > 0.5);
        let (mut a, mut b) = (Gluon::from_record(record(&l)), Gluon::from_record(record(&t)));
        let mut n = 0;
        for z in ZS {
            let (x, y) = (drawn(&mut a, z), drawn(&mut b, z));
            assert!(x.0 == y.0 && x.1 == y.1, "the picture changed under a gauge transformation at z = {z}");
            assert!(x.0.iter().map(|b| b.len()).sum::<usize>() > 0);
            n += x.1.len();
        }
        println!("gauge invariance of the drawn marks: {n} marks over {} zooms, identical bytes", ZS.len());
        // THE PLANT: the same rung fed a gauge-VARIANT field (the trace of one link at each
        // site, where the action density should be) draws different bytes after the same
        // transformation, so the comparison above can fire.
        let variant = |l: &Lattice| {
            let mut r = record(l);
            for s in r.snapshots.iter_mut() {
                let n = s.action.len();
                *s = Snapshot { action: (0..n).map(|k| 1.0 + l.link(n + k, 0).trace().re as f32 / 3.0).collect(), ..s.clone() };
            }
            r.quantised()
        };
        let (mut a, mut b) = (Gluon::from_record(variant(&l)), Gluon::from_record(variant(&t)));
        assert!(ZS.iter().all(|&z| drawn(&mut a, z).0 != drawn(&mut b, z).0), "the plant did not fire");
    }

    #[test]
    fn the_flow_time_follows_the_zoom_and_the_caption_names_it() {
        let mut g = Gluon::new();
        assert_eq!((g.record().dims, g.t_max()), ([12; 4], 2.5));
        // at the top of the span the smoothing radius is the longest stored; it falls with
        // the zoom, a hundredfold in flow time to the decade, and is zero past the floor
        let mut last = f64::MAX;
        for k in 0..=200 {
            let z = SPAN.1 - 0.01 * k as f64;
            let t = flow_time(z, 2.5);
            assert!(t <= last && t <= 2.5);
            last = t;
            if t > 0.0 && t < 2.5 {
                let r_px = (8.0 * t).sqrt() * A_FM_BETA_6 * 1e-15 / 10f64.powf(z);
                assert!((r_px / SMOOTH_PX - 1.0).abs() < 0.06, "z = {z}: the radius is {r_px} px");
            }
        }
        assert_eq!(flow_time(SPAN.1, 2.5), 2.5);
        assert_eq!(flow_time(-19.5, 2.5), 0.0);
        // the caption says the flow time on screen and what the picture is
        g.tiles(-18.3, 0.0, 1.0);
        let t = g.t;
        assert!(t > 0.3 && t < 1.0, "{t}");
        let s = g.source();
        assert!(s.contains(&format!("FLOW TIME t = {t:.3}")) && s.contains("sqrt(8t)"), "{s}");
        for words in ["QUENCHED", "no sea quarks", "one configuration of a Euclidean ensemble", "not a moment in a hadron's life", "NOT the proton", "this repository's own engine", "standard and not ours", "Cabibbo-Marinari", "Kennedy-Pendleton", "Creutz", "Lüscher", "Leinweber", "gauge invariant"] {
            assert!(s.contains(words), "the caption lacks {words:?}");
        }
        g.tiles(-19.6, 0.0, 1.0);
        assert!(g.t == 0.0 && g.source().contains("FLOW TIME t = 0:") && g.source().contains("the record ends here"));
        // the same caption is the same string, not a new one each frame
        let p = g.source().as_ptr();
        g.tiles(-18.3, 0.0, 1.0);
        g.tiles(-19.6, 0.0, 1.0);
        assert_eq!(g.source().as_ptr(), p);
        assert_eq!((g.status(), g.unit(), g.cut_above(), g.cut_below()), (Status::Measured, -16, true, false));
    }

    #[test]
    fn the_coarser_picture_is_smoother_and_the_box_is_at_its_published_size() {
        let mut g = Gluon::new();
        let balls = |g: &mut Gluon, z: f64| g.tiles(z, 0.0, 1.0)[0].marks.clone();
        // every ball is on a lattice site: a whole number of spacings from the centre
        let a = Gluon::spacing();
        assert!((a - 0.931).abs() < 1e-6, "0.0931 fm in tenths of a femtometre");
        for z in ZS {
            let m = balls(&mut g, z);
            assert!(!m.is_empty() && m.len() <= 1728, "z = {z}: {}", m.len());
            for b in &m {
                assert!(b.pos.iter().all(|x| ((x / a).round() - x / a).abs() < 1e-4 && (x / a).round().abs() <= 6.0));
                assert!(b.size[0] <= BALL * a * 1.0001 && b.size[0] >= 0.5 * 10f32.powf(z as f32 + 16.0));
            }
        }
        // the largest lump is at the centre at the longest flow
        let m = balls(&mut g, SPAN.1);
        let top = m.iter().max_by(|x, y| x.size[0].total_cmp(&y.size[0])).unwrap();
        assert_eq!(top.pos, [0.0; 3]);
        // the box's edges: 12 lattice spacings, 1.117 fm
        let lines = g.tiles(SPAN.1, 0.0, 1.0)[1].marks.clone();
        assert_eq!(lines.len(), 12);
        let len = |m: &Instance| ((m.aux[0] - m.pos[0]).powi(2) + (m.aux[1] - m.pos[1]).powi(2) + (m.aux[2] - m.pos[2]).powi(2)).sqrt();
        assert!(lines.iter().all(|m| (len(m) - 11.172).abs() < 1e-3));
        // zoomed in past the spacing the links are drawn: 3 * 12 * 12 * 11 of them
        assert_eq!(g.tiles(-19.5, 0.0, 1.0)[1].marks.len(), 12 + 3 * 12 * 12 * 11);
        // smoother when coarser: neighbouring sites' action densities are closer, relative to
        // the field's spread, at a longer flow time
        let rough = |g: &mut Gluon, z: f64| {
            g.tiles(z, 0.0, 1.0);
            let mean = g.e.iter().sum::<f32>() / g.e.len() as f32;
            let var = g.e.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / g.e.len() as f32;
            let step = (0..g.e.len()).filter(|s| s % 12 != 11).map(|s| (g.e[s + 1] - g.e[s]).powi(2)).sum::<f32>() / (g.e.len() * 11 / 12) as f32;
            step / var
        };
        let r: Vec<f32> = [-19.6, -18.8, -18.4, -18.0].iter().map(|&z| rough(&mut g, z)).collect();
        println!("neighbour difference over spread, by zoom (finest first): {r:?}");
        assert!(r.windows(2).all(|w| w[1] < w[0]), "{r:?}");
    }

    #[test]
    fn the_rung_sits_below_the_chain_across_a_cut_and_ends_the_ladder() {
        let mut l = Ladder::new(inward(), Band { z_lo: -1.0, z_hi: 4.2, clamp: false });
        let i = l.rungs.iter().position(|r| r.name == "the gluon field: quenched SU(3) on a 4D lattice").unwrap();
        assert_eq!(i, 0, "the finest rung");
        assert_eq!((l.rungs[0].z_lo, l.rungs[0].z_hi, l.rungs[0].unit, l.rungs[0].status), (Z_MIN, -18.0, -16, Status::Measured));
        assert_eq!(l.rungs[1].name, "the gauge vacuum: the sea");
        assert!(l.cuts.contains(&-18.0));
        // either side of the cut one rung is drawn, whole; never both
        for (z, want) in [(-18.2, 0), (-18.0001, 0), (-18.0, 1), (-17.9, 1)] {
            let p = l.passes(z, 0.8, 0.4, 1200.0, 900.0, 1.0, 0.0);
            assert_eq!((p.len(), p[0].rung, p[0].weight), (1, want, 1.0), "z = {z}");
        }
    }

    #[test]
    fn the_window_does_not_change_what_is_drawn_and_a_frame_is_cheap() {
        let band = Band { z_lo: -1.0, z_hi: 4.2, clamp: false };
        let (mut small, mut large) = (Ladder::new(inward(), band), Ladder::new(inward(), band));
        for z in ZS {
            let z = z - 0.01;
            let a = small.passes(z, 0.8, 0.42, 390.0, 844.0, 3.0, 0.0);
            let b = large.passes(z, 0.8, 0.42, 2560.0, 1440.0, 1.0, 0.0);
            assert_eq!((a.len(), b.len(), a[0].rung, b[0].rung), (1, 1, 0, 0));
            let world = |f: &holon_draw::Frame| f.batches.iter().filter(|b| !b.screen).map(|b| (b.tile, b.kind, b.coarse, b.bytes.clone(), b.params)).collect::<Vec<_>>();
            assert!(world(&a[0].frame) == world(&b[0].frame), "z = {z}");
        }
        let t = Instant::now();
        let mut g = Gluon::new();
        let built = t.elapsed();
        let mut worst = 0.0f64;
        for z in ZS {
            g.tiles(z, 0.0, 1.0);
            // a frame at a held zoom, and a frame in which the flow time changes
            let t = Instant::now();
            let mut n = 0;
            for k in 0..20 {
                n += g.tiles(z - 0.03 * (k % 2) as f64, 0.0, 1.0).iter().map(|t| t.marks.len()).sum::<usize>();
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0 / 20.0;
            println!("gluon field z = {z:6.2}: t = {:.3}, {:5} marks, {ms:.3} ms a frame", g.t, n / 20);
            worst = worst.max(ms);
        }
        println!("gluon rung built in {built:?}; worst frame {worst:.3} ms");
        if !cfg!(debug_assertions) {
            assert!(built.as_secs_f64() < 0.02 && worst < 2.0, "built in {built:?}, {worst} ms a frame");
        }
    }
}
