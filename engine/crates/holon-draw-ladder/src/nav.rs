//! The navigation control, drawn by the engine (the lead: "it should have the usual 3d control
//! object to pan/tilt/zoom"): a compass ring to turn (its N resets north), a pad of four pan
//! arrows, tilt up and down, zoom in and out, a vertical zoom slider spanning the whole ladder
//! with a tick per decade and the thumb at the present zoom, home, and the host-band switch.
//! Every control is a tagged screen mark, so a host picks it through the frame like any other.
use holon_draw::ladder::{Z_MAX, Z_MIN};
use holon_draw::ui::{Text, Ui};
use holon_draw::{Instance, Kind, Tile, PARAMS_NEUTRAL};

/// The controls' handles. A host maps a picked handle to an action.
pub const NAV: u32 = 0x5000_0000;
pub const RING: u32 = NAV + 1;
pub const NORTH: u32 = NAV + 2;
pub const TILT_UP: u32 = NAV + 3;
pub const TILT_DOWN: u32 = NAV + 4;
pub const PAN_UP: u32 = NAV + 5;
pub const PAN_DOWN: u32 = NAV + 6;
pub const PAN_LEFT: u32 = NAV + 7;
pub const PAN_RIGHT: u32 = NAV + 8;
pub const ZOOM_IN: u32 = NAV + 9;
pub const ZOOM_OUT: u32 = NAV + 10;
pub const SLIDER: u32 = NAV + 11;
pub const HOME: u32 = NAV + 12;
pub const SWITCH: u32 = NAV + 13;

/// Where the continuous controls are, in device pixels, for a host's dragging: the ring's
/// centre and radius, and the slider track's x, top and bottom (top is `Z_MAX`, the universe;
/// bottom is `Z_MIN`, the vacuum).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Geometry {
    pub ring: [f32; 3],
    pub slider: [f32; 3],
}

impl Geometry {
    /// The zoom a point on the slider track stands for.
    pub fn z_at(&self, y: f32) -> f64 {
        let t = ((y - self.slider[1]) / (self.slider[2] - self.slider[1]).max(1.0)).clamp(0.0, 1.0) as f64;
        Z_MAX - t * (Z_MAX - Z_MIN)
    }
}

const PANEL: [u8; 4] = [23, 32, 29, 214];
const RIM: [u8; 4] = [58, 72, 66, 255];
const KEY: [u8; 4] = [44, 58, 52, 255];
const INK: [u8; 4] = [237, 233, 223, 255];
const GREEN: [u8; 4] = [112, 166, 142, 255];

fn disc(ui: &mut Ui, x: f32, y: f32, r: f32, c: [u8; 4], tag: u32) {
    ui.marks.push(Instance::new(Kind::Disc, [x, y, 0.0], [r, 0.0, 0.0]).colour(c).tagged(tag));
}

fn line(ui: &mut Ui, a: [f32; 2], b: [f32; 2], w: f32, c: [u8; 4]) {
    let mut m = Instance::new(Kind::Line, [a[0], a[1], 0.0], [w, 0.0, 0.0]).colour(c);
    m.aux = [b[0], b[1], 0.0];
    ui.marks.push(m);
}

/// A square key with a centred label.
fn key(ui: &mut Ui, x: f32, y: f32, s: f32, label: &str, tag: u32, on: bool) {
    let r = ui.px(5.0);
    ui.quad(x, y, s, s, r, if on { [35, 105, 87, 255] } else { KEY }, tag);
    let w = ui.measure(label, 3);
    let line_h = ui.line(3);
    ui.run(label, 3, INK, x + 0.5 * (s - w), y + 0.5 * (s - line_h));
}

/// The control as one screen tile, at the bottom right of a `width` by `height` viewport.
#[allow(clippy::too_many_arguments)]
pub fn build(text: &mut dyn Text, width: f32, height: f32, dpr: f32, z: f64, yaw: f64, pitch: f64, switch_on: bool) -> (Vec<Tile>, Geometry) {
    let mut ui = Ui::new(text, dpr);
    let px = |c: f32| (c * dpr.max(0.1)).round();
    let (k, gap, pad) = (px(26.0), px(4.0), px(8.0));
    let w = 3.0 * k + 2.0 * gap + 2.0 * pad;
    let slider_h = (height - px(16.0) * 2.0 - (pad * 2.0 + w + 5.0 * (k + gap))).clamp(px(60.0), px(260.0));
    let h = pad * 2.0 + (w - 2.0 * pad) + gap + 3.0 * (k + gap) + gap + k + gap + slider_h + gap + k + gap + k;
    let x0 = width - w - px(16.0);
    let y0 = (height - h - px(16.0)).max(px(16.0));
    ui.quad(x0 - 1.0, y0 - 1.0, w + 2.0, h + 2.0, px(9.0), RIM, 0);
    ui.quad(x0, y0, w, h, px(8.0), PANEL, 0);
    let cx = x0 + 0.5 * w;
    let mut y = y0 + pad;
    // the compass ring: drag around it to turn; its N resets north
    let rr = 0.5 * (w - 2.0 * pad);
    let ring_c = [cx, y + rr];
    disc(&mut ui, ring_c[0], ring_c[1], rr, RIM, RING);
    disc(&mut ui, ring_c[0], ring_c[1], rr - px(7.0), PANEL, 0);
    // a needle pointing to the north of the view, and the N key on the ring
    // the needle and the N ride in a tile of their own over the ring (within one tile lines are
    // drawn before discs, so the ring's face would cover them)
    let over_from = ui.marks.len();
    // the world's +y (north) appears on screen at an angle of -yaw from the screen's right
    let a = -(yaw as f32);
    let tip = [ring_c[0] + (rr - px(11.0)) * a.cos(), ring_c[1] - (rr - px(11.0)) * a.sin()];
    line(&mut ui, ring_c, tip, px(2.5), GREEN);
    disc(&mut ui, ring_c[0], ring_c[1], px(3.0), INK, 0);
    let n = [ring_c[0] + (rr - px(3.5)) * a.cos(), ring_c[1] - (rr - px(3.5)) * a.sin()];
    disc(&mut ui, n[0], n[1], px(7.0), [35, 105, 87, 255], NORTH);
    let nw = ui.measure("N", 4);
    let nl = ui.line(4);
    ui.run("N", 4, INK, n[0] - 0.5 * nw, n[1] - 0.5 * nl);
    let over: Vec<Instance> = ui.marks.drain(over_from..).collect();
    // the pitch, as a short arc of text under the ring
    y += 2.0 * rr + gap;
    // the pan pad: a 3 by 3 grid with arrows on the cross and tilt in two corners
    let gx = x0 + pad;
    key(&mut ui, gx, y, k, "↥", TILT_UP, false);
    key(&mut ui, gx + k + gap, y, k, "▲", PAN_UP, false);
    key(&mut ui, gx + 2.0 * (k + gap), y, k, "⌂", HOME, false);
    y += k + gap;
    key(&mut ui, gx, y, k, "◀", PAN_LEFT, false);
    let pitch_txt = format!("{:.0}°", pitch.to_degrees());
    let pw = ui.measure(&pitch_txt, 4);
    let pl = ui.line(4);
    ui.run(&pitch_txt, 4, [105, 115, 110, 255], gx + k + gap + 0.5 * (k - pw), y + 0.5 * (k - pl));
    key(&mut ui, gx + 2.0 * (k + gap), y, k, "▶", PAN_RIGHT, false);
    y += k + gap;
    key(&mut ui, gx, y, k, "↧", TILT_DOWN, false);
    key(&mut ui, gx + k + gap, y, k, "▼", PAN_DOWN, false);
    key(&mut ui, gx + 2.0 * (k + gap), y, k, "S", SWITCH, switch_on);
    y += k + 2.0 * gap;
    // zoom: + at the top (inward), the slider, − at the bottom (outward)... the universe is up the
    // slider, so + (closer) is BELOW: the slider reads as a scale from the universe to the vacuum
    let kx = cx - 0.5 * k;
    key(&mut ui, kx, y, k, "−", ZOOM_OUT, false);
    y += k + gap;
    let (sx, top, bottom) = (cx, y + px(6.0), y + slider_h - px(6.0));
    ui.quad(sx - px(8.0), y, px(16.0), slider_h, px(4.0), KEY, SLIDER);
    line(&mut ui, [sx, top], [sx, bottom], px(2.0), RIM);
    // a tick per decade, longer every ten
    let span = (Z_MAX - Z_MIN) as f32;
    let mut d = Z_MIN.ceil() as i32;
    while d as f64 <= Z_MAX {
        let ty = bottom - (d as f32 - Z_MIN as f32) / span * (bottom - top);
        let half = if d % 10 == 0 { px(7.0) } else { px(3.0) };
        line(&mut ui, [sx - half, ty], [sx + half, ty], px(1.0), [105, 115, 110, 255]);
        d += 1;
    }
    // the metre (z = 0 on a 1-pixel scale): a reference mark
    let ty0 = bottom - (0.0 - Z_MIN as f32) / span * (bottom - top);
    line(&mut ui, [sx - px(9.0), ty0], [sx + px(9.0), ty0], px(1.5), GREEN);
    let thumb = bottom - ((z - Z_MIN) as f32 / span) * (bottom - top);
    disc(&mut ui, sx, thumb, px(7.0), INK, SLIDER);
    y += slider_h + gap;
    key(&mut ui, kx, y, k, "+", ZOOM_IN, false);
    let tile = Tile { id: 9998, origin: [0.0; 3], version: 1, layer: 251, screen: true, params: PARAMS_NEUTRAL, marks: std::mem::take(&mut ui.marks) };
    let needle = Tile { id: 9997, origin: [0.0; 3], version: 1, layer: 252, screen: true, params: PARAMS_NEUTRAL, marks: over };
    (vec![tile, needle], Geometry { ring: [ring_c[0], ring_c[1], rr], slider: [sx, top, bottom] })
}

#[cfg(test)]
mod tests {
    use super::*;
    use holon_draw::ui::FakeText;
    use holon_draw::{Acuity, Camera, Scene};

    #[test]
    fn every_control_is_on_screen_and_picked_where_it_is_drawn() {
        for (w, h, dpr) in [(1200.0f32, 800.0f32, 1.0f32), (1170.0, 2532.0, 3.0), (2485.0, 1567.0, 1.8)] {
            let mut t = FakeText { dpr, placed: 0 };
            let (tiles, g) = build(&mut t, w, h, dpr, -2.2, 0.8, 0.42, false);
            let cam = Camera::map([0.0; 3], 1.0, w as f64, h as f64);
            let f = Scene { tiles, modal: None }.frame(&cam, 1, &Acuity { eps_px: 1.0 }, u64::MAX).unwrap();
            for tag in [RING, NORTH, TILT_UP, TILT_DOWN, PAN_UP, PAN_DOWN, PAN_LEFT, PAN_RIGHT, ZOOM_IN, ZOOM_OUT, SLIDER, HOME, SWITCH] {
                let r = f.report(tag).unwrap_or_else(|| panic!("{tag:x} missing at {w}x{h}"));
                assert!(r.on_screen && r.legible, "{tag:x} at {w}x{h}");
                let (cx, cy) = (0.5 * (r.bounds[0] + r.bounds[2]), 0.5 * (r.bounds[1] + r.bounds[3]));
                if tag != RING && tag != SLIDER {
                    assert_eq!(f.pick(cx, cy), Some(tag), "{tag:x} at {w}x{h}");
                }
            }
            // the slider maps its ends to the ends of the ladder, and its thumb sits at the zoom
            assert_eq!(g.z_at(g.slider[1]), Z_MAX);
            assert_eq!(g.z_at(g.slider[2]), Z_MIN);
            assert!((g.z_at(g.slider[1] + 0.5 * (g.slider[2] - g.slider[1])) - 0.5 * (Z_MAX + Z_MIN)).abs() < 1e-6);
            assert!(g.ring[2] > 10.0);
        }
    }
}
