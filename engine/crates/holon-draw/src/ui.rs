//! Panels, text and buttons as SCREEN tiles: a small immediate-mode layout.
//!
//! The core holds no string and no font. A host's text provider rasterises a string into an
//! atlas and says how big it came out ([`Text::place`]); this module only decides WHERE each
//! run, panel and button goes, in device pixels, and emits them as marks of one screen
//! [`Tile`]: rounded [`Kind::Quad`]s for panels and buttons, [`Kind::Run`]s for text. A
//! button's quad (and a panel's background) is TAGGED with the caller's handle, so
//! [`crate::Frame::pick`] finds it by its exact rectangle.
//!
//! Within one tile the core draws every quad before every run, and quads in the order they
//! were pushed; that is the only layering this module relies on.
//!
//! Two placements ([`layout`]): on a wide viewport a left and a right column of panels with
//! the status line and the button row along the bottom; on a narrow one (under
//! [`NARROW_CSS_PX`], or when the columns do not fit) one column of COLLAPSIBLE panels, each
//! heading a toggle carrying a one-word summary, above a button row that wraps.
use crate::{digest::sha256, Bind, Drawn, Instance, Kind, Tile};

/// What a text provider says of a string it has rasterised: its size in device pixels and
/// where it sits in the atlas.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Placed {
    pub w: f32,
    pub h: f32,
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
    pub page: u8,
    /// What the provider says of the drawing: whole, cut, changed, or nothing said.
    pub drawn: Drawn,
}

/// The host's text provider. `style` is one of [`BODY`], [`HEADING`], [`STATUS`],
/// [`BUTTON`], [`SMALL`]; sizes are CSS pixels times the device pixel ratio, which the
/// provider knows.
pub trait Text {
    fn place(&mut self, text: &str, style: u8) -> Placed;
}

/// Monospace 13 px.
pub const BODY: u8 = 0;
/// Sans bold capitals 11 px.
pub const HEADING: u8 = 1;
/// Monospace bold 16 px.
pub const STATUS: u8 = 2;
/// Sans 13 px.
pub const BUTTON: u8 = 3;
/// Monospace 11 px.
pub const SMALL: u8 = 4;
/// The nominal font size of each style, CSS pixels.
pub const STYLE_PX: [f32; 5] = [13.0, 11.0, 16.0, 13.0, 11.0];

pub const INK: [u8; 4] = [0x17, 0x20, 0x1d, 217];
pub const EDGE: [u8; 4] = [0x70, 0xa6, 0x8e, 71];
pub const PAPER: [u8; 4] = [0xed, 0xe9, 0xdf, 255];
pub const MUTED: [u8; 4] = [0x69, 0x73, 0x6e, 255];
pub const GREEN: [u8; 4] = [0x23, 0x69, 0x57, 230];
pub const PASS: [u8; 4] = [0x70, 0xa6, 0x8e, 255];
pub const FAIL: [u8; 4] = [0xbb, 0x4c, 0x2e, 255];
pub const AMBER: [u8; 4] = [0xd4, 0x9a, 0x36, 255];
/// A button that is a switch, while it is off.
pub const IDLE: [u8; 4] = [0x2c, 0x38, 0x33, 230];

/// Under this viewport width (CSS pixels) the panels are one collapsible column.
pub const NARROW_CSS_PX: f32 = 640.0;
/// A wide column's width, CSS pixels.
pub const COLUMN_CSS_PX: f32 = 300.0;
/// A button's height, CSS pixels: about the smallest a fingertip hits reliably.
pub const BUTTON_CSS_PX: f32 = 34.0;

/// One thing in a panel.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    /// A status line in a colour (a verdict), wrapped.
    Status(String, [u8; 4]),
    /// A label and its value; the values of a panel share one column.
    Row(String, String),
    /// A paragraph, wrapped by measuring its words.
    Para(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Panel {
    pub heading: String,
    /// Tags the panel's background: a pick on the panel answers this handle (so the world
    /// under a panel is not picked through it), and on a narrow viewport it is the toggle.
    pub handle: u32,
    /// Narrow viewports only: a closed panel draws its heading and summary alone.
    pub open: bool,
    /// The one-word reading shown at the right of a closed panel's heading.
    pub summary: String,
    pub summary_colour: [u8; 4],
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Button {
    pub label: String,
    pub handle: u32,
    /// A switch that is off is drawn in [`IDLE`]; everything else in [`GREEN`].
    pub active: bool,
}

/// A whole overlay: two groups of panels, a status line and a row of buttons.
#[derive(Clone, Debug, PartialEq)]
pub struct Hud {
    pub left: Vec<Panel>,
    pub right: Vec<Panel>,
    /// False hides the panels and keeps the status line and the buttons.
    pub panels: bool,
    pub status: String,
    pub status_colour: [u8; 4],
    pub buttons: Vec<Button>,
}

/// Where everything went. Rectangles are left, top, right, bottom in device pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub tile: Tile,
    /// The single collapsible column was used.
    pub narrow: bool,
    pub panels: Vec<(u32, [f32; 4])>,
    pub buttons: Vec<(u32, [f32; 4])>,
    pub status: [f32; 4],
    /// The largest rectangle no panel, button or status line covers: where the scene shows.
    pub scene: [f32; 4],
}

/// The immediate-mode surface: quads and runs pushed in device pixels.
///
/// Text pushed with a TAG ([`Ui::text`], [`Ui::text_flow`], and a button's label, which takes
/// the button's handle) carries [`Bind::Source`]: the SHA-256 of the string as given, and what the
/// provider said of the drawing. That is what lets the core report a value's string (L12).
/// Untagged text carries nothing.
pub struct Ui<'a> {
    pub text: &'a mut dyn Text,
    pub dpr: f32,
    pub marks: Vec<Instance>,
    space: [f32; 5],
}

struct Word {
    p: Placed,
    x: f32,
}

impl<'a> Ui<'a> {
    pub fn new(text: &'a mut dyn Text, dpr: f32) -> Ui<'a> {
        Ui { text, dpr: if dpr.is_finite() && dpr > 0.0 { dpr } else { 1.0 }, marks: Vec::new(), space: [-1.0; 5] }
    }
    /// CSS pixels to device pixels, whole.
    pub fn px(&self, css: f32) -> f32 {
        (css * self.dpr).round()
    }
    /// The height of a line of a style.
    pub fn line(&self, style: u8) -> f32 {
        (STYLE_PX[style as usize % 5] * 1.15 * self.dpr).ceil()
    }
    /// The width of a space of a style, measured: `"x x"` less two `"x"`.
    fn space(&mut self, style: u8) -> f32 {
        let k = style as usize % 5;
        if self.space[k] < 0.0 {
            let (a, b) = (self.text.place("x x", style).w, self.text.place("x", style).w);
            self.space[k] = (a - 2.0 * b).max(0.0);
        }
        self.space[k]
    }
    /// A rounded rectangle, by its left, top, width and height.
    pub fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, radius: f32, colour: [u8; 4], tag: u32) {
        let mut m = Instance::new(Kind::Quad, [x + 0.5 * w, y + 0.5 * h, 0.0], [w, h, 0.0]).colour(colour).tagged(tag);
        m.aux[0] = radius;
        self.marks.push(m);
    }
    #[allow(clippy::too_many_arguments)]
    fn push_run(&mut self, p: Placed, left: f32, top: f32, line: f32, colour: [u8; 4], tag: u32, bind: Bind) {
        if p.w <= 0.0 || p.h <= 0.0 {
            return;
        }
        let (l, t) = (left.round(), (top + 0.5 * (line - p.h)).round());
        let mut m = Instance::new(Kind::Run, [l + 0.5 * p.w, t + 0.5 * p.h, 0.0], [p.w, p.h, 0.0]).colour(colour);
        m.aux = [p.u0, p.v0, p.u1];
        m.angle = p.v1;
        m.variant = p.page;
        m.tag = tag;
        m.bind = bind;
        self.marks.push(m);
    }
    /// One run, not wrapped, its left edge at `x` and its line's top at `y`; returns its width.
    pub fn run(&mut self, s: &str, style: u8, colour: [u8; 4], x: f32, y: f32) -> f32 {
        self.run_as(s, style, colour, x, y, 0)
    }
    /// As [`Ui::run`], the run tagged `tag` and bound to the source string `s`.
    pub fn text(&mut self, s: &str, style: u8, colour: [u8; 4], x: f32, y: f32, tag: u32) -> f32 {
        self.run_as(s, style, colour, x, y, tag)
    }
    fn run_as(&mut self, s: &str, style: u8, colour: [u8; 4], x: f32, y: f32, tag: u32) -> f32 {
        if s.trim().is_empty() {
            return 0.0;
        }
        let p = self.text.place(s, style);
        let line = self.line(style);
        let bind = if tag == 0 { Bind::None } else { Bind::Source { sha256: sha256(s.as_bytes()), drawn: p.drawn } };
        self.push_run(p, x, y, line, colour, tag, bind);
        p.w
    }
    /// The width a string would take as one run.
    pub fn measure(&mut self, s: &str, style: u8) -> f32 {
        if s.trim().is_empty() {
            0.0
        } else {
            self.text.place(s, style).w
        }
    }
    /// Text wrapped to `max_w` by measuring its words (each word is its own run, so nothing
    /// is rasterised only to be measured); `\n` breaks a line. Returns the height used.
    #[allow(clippy::too_many_arguments)]
    pub fn flow(&mut self, s: &str, style: u8, colour: [u8; 4], x: f32, y: f32, max_w: f32, centred: bool) -> f32 {
        self.flow_as(s, style, colour, x, y, max_w, centred, 0)
    }
    /// As [`Ui::flow`], every run tagged `tag` and bound to the WHOLE string `s` (a wrapped
    /// value is one instance with one source). A flow draws words with single spaces, so a
    /// string with any other whitespace (doubled, leading, trailing, a tab) reports
    /// [`Drawn::Changed`]; a line break is a `\n`, which the flow keeps.
    #[allow(clippy::too_many_arguments)]
    pub fn text_flow(&mut self, s: &str, style: u8, colour: [u8; 4], x: f32, y: f32, max_w: f32, centred: bool, tag: u32) -> f32 {
        self.flow_as(s, style, colour, x, y, max_w, centred, tag)
    }
    #[allow(clippy::too_many_arguments)]
    fn flow_as(&mut self, s: &str, style: u8, colour: [u8; 4], x: f32, y: f32, max_w: f32, centred: bool, tag: u32) -> f32 {
        let (line, space) = (self.line(style), self.space(style));
        let mut drawn = Drawn::Exact;
        if s.split('\n').any(|para| para.split(' ').any(|w| w.is_empty() || w.chars().any(char::is_whitespace)) && !para.is_empty()) {
            drawn = Drawn::Changed;
        }
        let mut rows: Vec<(Vec<Word>, f32)> = Vec::new();
        for para in s.split('\n') {
            let mut row: Vec<Word> = Vec::new();
            let mut w = 0.0f32;
            for word in para.split_whitespace() {
                let p = self.text.place(word, style);
                drawn = drawn.and(p.drawn);
                let at = if row.is_empty() { 0.0 } else { w + space };
                if !row.is_empty() && at + p.w > max_w {
                    rows.push((std::mem::take(&mut row), w));
                    row.push(Word { p, x: 0.0 });
                    w = p.w;
                } else {
                    row.push(Word { p, x: at });
                    w = at + p.w;
                }
            }
            rows.push((row, w));
        }
        let bind = if tag == 0 { Bind::None } else { Bind::Source { sha256: sha256(s.as_bytes()), drawn } };
        for (k, (row, w)) in rows.iter().enumerate() {
            let x0 = if centred { x + 0.5 * (max_w - w) } else { x };
            for word in row {
                self.push_run(word.p, x0 + word.x, y + k as f32 * line, line, colour, tag, bind);
            }
        }
        rows.len() as f32 * line
    }

    /// A panel with its left top at `x`, `y` and `w` wide; `open` false draws the heading and
    /// the summary alone. Returns its height.
    pub fn panel(&mut self, p: &Panel, x: f32, y: f32, w: f32, open: bool, summary: bool) -> f32 {
        let (pad, gap) = (self.px(if summary { 6.0 } else { 7.0 }), self.px(3.0));
        let inner = w - 2.0 * pad;
        // the content first (its height sizes the background); quads draw under runs whatever the order
        let head = self.line(HEADING).max(if summary { self.line(BODY) } else { 0.0 });
        self.run(&p.heading, HEADING, MUTED, x + pad, y + pad + 0.5 * (head - self.line(HEADING)));
        if summary {
            let sw = self.measure(&p.summary, BODY);
            self.run(&p.summary, BODY, p.summary_colour, x + w - pad - sw, y + pad + 0.5 * (head - self.line(BODY)));
        }
        let mut cy = y + pad + head;
        if open {
            let col = p.items.iter().fold(0.0f32, |c, it| match it {
                Item::Row(l, _) => c.max(self.measure(l, BODY)),
                _ => c,
            }) + self.space(BODY) * 2.0;
            let col = col.min(0.6 * inner);
            cy += gap;
            for it in &p.items {
                cy += match it {
                    Item::Status(s, c) => self.flow(s, STATUS, *c, x + pad, cy, inner, false) + gap,
                    Item::Row(l, v) => {
                        self.run(l, BODY, PAPER, x + pad, cy);
                        self.flow(v, BODY, PAPER, x + pad + col, cy, inner - col, false).max(self.line(BODY))
                    }
                    Item::Para(s) => self.flow(s, BODY, PAPER, x + pad, cy, inner, false),
                };
            }
        }
        let h = cy + pad - y;
        let (r, b) = (self.px(4.0), self.px(1.0).max(1.0));
        // the border and the background: two quads, both carrying the handle
        self.quad(x, y, w, h, r, EDGE, p.handle);
        self.quad(x + b, y + b, w - 2.0 * b, h - 2.0 * b, (r - b).max(0.0), INK, p.handle);
        h
    }

    /// A button: a rounded quad tagged with its handle and its label centred on it.
    pub fn button(&mut self, b: &Button, x: f32, y: f32, w: f32, h: f32) {
        self.quad(x, y, w, h, self.px(4.0), if b.active { GREEN } else { IDLE }, b.handle);
        let lw = self.measure(&b.label, BUTTON);
        let line = self.line(BUTTON);
        // the label is part of the button's instance, so its report says what the button reads
        self.text(&b.label, BUTTON, PAPER, x + 0.5 * (w - lw), y + 0.5 * (h - line), b.handle);
    }

    /// A row of buttons that wraps within `max_w`, each line centred, its BOTTOM at `bottom`.
    /// Returns the top of the row and each button's rectangle.
    pub fn button_row(&mut self, buttons: &[Button], x: f32, bottom: f32, max_w: f32) -> (f32, Vec<(u32, [f32; 4])>) {
        let (h, gap, pad) = (self.px(BUTTON_CSS_PX), self.px(4.0), self.px(10.0));
        let widths: Vec<f32> = buttons.iter().map(|b| (self.measure(&b.label, BUTTON) + 2.0 * pad).min(max_w)).collect();
        let mut lines: Vec<(Vec<usize>, f32)> = vec![(Vec::new(), 0.0)];
        for (i, &w) in widths.iter().enumerate() {
            let last = lines.last_mut().expect("one line at least");
            let at = if last.0.is_empty() { 0.0 } else { last.1 + gap };
            if !last.0.is_empty() && at + w > max_w {
                lines.push((vec![i], w));
            } else {
                last.0.push(i);
                last.1 = at + w;
            }
        }
        let n = lines.len() as f32;
        let top = bottom - n * h - (n - 1.0) * gap;
        let mut rects = Vec::with_capacity(buttons.len());
        for (k, (line, w)) in lines.iter().enumerate() {
            let mut bx = (x + 0.5 * (max_w - w)).round();
            let by = top + k as f32 * (h + gap);
            for &i in line {
                self.button(&buttons[i], bx, by, widths[i], h);
                rects.push((buttons[i].handle, [bx, by, bx + widths[i], by + h]));
                bx += widths[i] + gap;
            }
        }
        (top, rects)
    }
}

/// Lay a whole overlay out for a viewport of `width` by `height` DEVICE pixels at `dpr`, as
/// one screen tile of the given id, version and layer.
pub fn layout(hud: &Hud, text: &mut dyn Text, width: f32, height: f32, dpr: f32, id: u64, version: u64, layer: u8) -> Layout {
    let css_w = width / if dpr > 0.0 { dpr } else { 1.0 };
    let all = || hud.left.iter().chain(&hud.right);
    // the bottom bar is the same in both placements; the panels are tried wide first
    let bar = |ui: &mut Ui, m: f32| {
        let (bar_top, buttons) = ui.button_row(&hud.buttons, m, height - m, width - 2.0 * m);
        let gap = ui.px(6.0);
        // measure the status by flowing it, then move it up by its own height
        let at = ui.marks.len();
        let sh = ui.flow(&hud.status, BODY, hud.status_colour, m, 0.0, width - 2.0 * m, true);
        let top = bar_top - gap - sh;
        for mk in &mut ui.marks[at..] {
            mk.pos[1] += top;
        }
        (top, buttons, [m, top, width - m, top + sh])
    };
    let mut narrow = css_w < NARROW_CSS_PX;
    loop {
        let mut ui = Ui::new(text, dpr);
        let m = ui.px(if narrow { 6.0 } else { 10.0 });
        let gap = ui.px(if narrow { 4.0 } else { 6.0 });
        let (bar_top, buttons, status) = bar(&mut ui, m);
        let mut panels = Vec::new();
        let mut scene = [0.0, 0.0, width, bar_top];
        if hud.panels && narrow {
            let mut y = m;
            for p in all() {
                let h = ui.panel(p, m, y, width - 2.0 * m, p.open, true);
                panels.push((p.handle, [m, y, width - m, y + h]));
                y += h + gap;
            }
            scene[1] = (y - gap + m).min(bar_top);
        } else if hud.panels {
            let w = ui.px(COLUMN_CSS_PX);
            let mut fits = 2.0 * w + 3.0 * m <= width;
            for (group, x) in [(&hud.left, m), (&hud.right, width - m - w)] {
                let mut y = m;
                for p in group {
                    let h = ui.panel(p, x, y, w, true, false);
                    panels.push((p.handle, [x, y, x + w, y + h]));
                    y += h + gap;
                }
                fits &= y - gap <= bar_top - gap;
            }
            if !fits {
                // the columns do not fit this viewport: the collapsible column instead
                narrow = true;
                continue;
            }
            scene = [m + w, 0.0, width - m - w, bar_top];
        }
        let tile = Tile { id, origin: [0.0; 3], version, layer, screen: true, params: crate::PARAMS_NEUTRAL, marks: ui.marks };
        return Layout { tile, narrow, panels, buttons, status, scene };
    }
}

/// A stand-in provider for tests and native builds: every character `0.6` of the style's
/// size wide, every run one line-box high, no atlas.
pub struct FakeText {
    pub dpr: f32,
    pub placed: u64,
}

impl Text for FakeText {
    fn place(&mut self, text: &str, style: u8) -> Placed {
        self.placed += 1;
        let px = STYLE_PX[style as usize % 5] * self.dpr;
        Placed { w: (text.chars().count() as f32 * 0.6 * px).ceil(), h: (px * 1.15).ceil(), u0: 0.0, v0: 0.0, u1: 1.0, v1: 1.0, page: 0, drawn: Drawn::Exact }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Camera, Ring, Scene};

    fn panel(k: u32, rows: usize, open: bool) -> Panel {
        let mut items = vec![Item::Status("PASS  E - W_ext is constant".into(), PASS)];
        for r in 0..rows {
            items.push(Item::Row(format!("label {r}"), format!("{:+.6e}", 1.25 * r as f64)));
        }
        items.push(Item::Para("A separate gate on purpose: energy can be right while momentum is wrong.".into()));
        Panel { heading: format!("GATE {k} / ENERGY"), handle: 100 + k, open, summary: "PASS".into(), summary_colour: PASS, items }
    }
    fn hud(open: bool) -> Hud {
        let labels = ["reset", "preset: H2", "-", "+  (2 atoms)", "slower", "faster  (3.334 fs/s)", "walls ON", "thermostat OFF", "composite ON", "dt held", "hud"];
        Hud {
            left: (1..=3).map(|k| panel(k, 8, open)).collect(),
            right: (4..=6).map(|k| panel(k, 8, open)).collect(),
            panels: true,
            status: "[H2]  t = 1851.2 a.u. (44.779 fs)  |  2 atoms  |  0 clusters  |  0 bonded pairs  |  drag an atom; the spring is on the ledger".into(),
            status_colour: PAPER,
            buttons: labels.iter().enumerate().map(|(i, l)| Button { label: l.to_string(), handle: 200 + i as u32, active: i != 7 }).collect(),
        }
    }
    fn lay(h: &Hud, w: f32, ht: f32, dpr: f32) -> Layout {
        layout(h, &mut FakeText { dpr, placed: 0 }, w, ht, dpr, 7, 1, 0)
    }
    fn apart(a: &[f32; 4], b: &[f32; 4]) -> bool {
        a[2] <= b[0] || b[2] <= a[0] || a[3] <= b[1] || b[3] <= a[1]
    }
    fn inside(r: &[f32; 4], w: f32, h: f32) -> bool {
        r[0] >= 0.0 && r[1] >= 0.0 && r[2] <= w && r[3] <= h && r[2] > r[0] && r[3] > r[1]
    }
    /// Every mark's rectangle is within the viewport, and nothing covers anything else.
    fn sound(l: &Layout, w: f32, h: f32) {
        for m in &l.tile.marks {
            let r = [m.pos[0] - 0.5 * m.size[0], m.pos[1] - 0.5 * m.size[1], m.pos[0] + 0.5 * m.size[0], m.pos[1] + 0.5 * m.size[1]];
            assert!(inside(&r, w, h), "a {:?} at {r:?} leaves {w} x {h}", m.kind);
        }
        let mut boxes: Vec<[f32; 4]> = l.panels.iter().map(|p| p.1).chain(l.buttons.iter().map(|b| b.1)).collect();
        boxes.push(l.status);
        for (i, a) in boxes.iter().enumerate() {
            assert!(inside(a, w, h), "{a:?}");
            for b in &boxes[i + 1..] {
                assert!(apart(a, b), "{a:?} overlaps {b:?}");
            }
            assert!(apart(a, &l.scene), "{a:?} is over the scene {:?}", l.scene);
        }
        assert!(l.scene[2] - l.scene[0] >= 0.3 * w && l.scene[3] - l.scene[1] >= 0.25 * h, "the scene is left {:?} of {w} x {h}", l.scene);
    }

    #[test]
    fn a_wide_viewport_has_two_columns_a_status_line_and_a_button_row_and_nothing_overlaps() {
        let l = lay(&hud(false), 1280.0, 800.0, 1.0);
        assert!(!l.narrow);
        assert_eq!((l.panels.len(), l.buttons.len()), (6, 11));
        sound(&l, 1280.0, 800.0);
        // the left three share a left edge, the right three a right edge; one row of buttons
        assert!(l.panels[..3].iter().all(|p| p.1[0] == 10.0) && l.panels[3..].iter().all(|p| p.1[2] == 1270.0));
        assert!(l.buttons.iter().all(|b| b.1[1] == l.buttons[0].1[1]), "the buttons fit one row at 1280");
        assert!(l.status[3] <= l.buttons[0].1[1]);
        // wide panels are open whatever the flag says: Bevy's cannot be closed
        assert!(l.panels.iter().all(|p| p.1[3] - p.1[1] > 100.0));
    }

    #[test]
    fn a_phone_viewport_is_one_column_of_collapsible_panels_above_a_wrapping_button_row() {
        let (w, h) = (1170.0, 2532.0); // 390 x 844 at dpr 3
        let closed = lay(&hud(false), w, h, 3.0);
        assert!(closed.narrow);
        sound(&closed, w, h);
        let rows: std::collections::BTreeSet<i32> = closed.buttons.iter().map(|b| b.1[1] as i32).collect();
        assert!(rows.len() > 1, "the row wraps at phone width");
        assert!(closed.buttons.iter().all(|b| b.1[3] - b.1[1] >= 34.0 * 3.0), "a fingertip's height");
        // a collapsed panel draws its heading and its summary and nothing else
        let runs = closed.tile.marks.iter().filter(|m| m.kind == Kind::Run && m.pos[1] < closed.scene[1]).count();
        assert_eq!(runs, 12, "six headings and six summaries");
        assert!(closed.panels.iter().all(|p| p.1[3] - p.1[1] < 40.0 * 3.0));
        // one opened: taller, the rest pushed down, still sound
        let mut one = hud(false);
        one.left[1].open = true;
        let open = lay(&one, w, h, 3.0);
        sound(&open, w, h);
        assert!(open.panels[1].1[3] - open.panels[1].1[1] > 4.0 * (closed.panels[1].1[3] - closed.panels[1].1[1]));
        assert!(open.panels[2].1[1] > closed.panels[2].1[1]);
        assert!(open.tile.marks.len() > closed.tile.marks.len());
    }

    #[test]
    fn a_short_wide_viewport_falls_back_to_the_collapsible_column() {
        let l = lay(&hud(false), 1280.0, 420.0, 1.0);
        assert!(l.narrow, "two columns of open panels do not fit 420 px");
        sound(&l, 1280.0, 420.0);
    }

    #[test]
    fn buttons_and_panels_are_picked_by_their_rectangles_through_the_core() {
        for (w, h, dpr) in [(1280.0f32, 800.0f32, 1.0f32), (1170.0, 2532.0, 3.0)] {
            let l = lay(&hud(false), w, h, dpr);
            let cam = Camera::orbit([12.0; 3], 0.7, 0.4, 50.0, 0.8, w as f64, h as f64);
            let scene = Scene { tiles: vec![l.tile.clone()], modal: None };
            let f = scene.frame(&cam, 1, &Ring, u64::MAX).expect("finite");
            for (handle, r) in l.buttons.iter().chain(&l.panels) {
                let (cx, cy) = (0.5 * (r[0] + r[2]) as f64, 0.5 * (r[1] + r[3]) as f64);
                assert_eq!(f.pick(cx, cy), Some(*handle));
                assert_eq!(f.pick(r[0] as f64 + 1.5, r[1] as f64 + 1.5), Some(*handle), "its corner");
                let b = f.report(*handle).expect("drawn").bounds;
                assert!((b[0] - r[0] as f64).abs() <= 1.0 && (b[3] - r[3] as f64).abs() <= 1.0, "{b:?} against {r:?}");
            }
            let s = l.scene;
            assert_eq!(f.pick(0.5 * (s[0] + s[2]) as f64, 0.5 * (s[1] + s[3]) as f64), None, "the scene is not covered");
            // quads are drawn before runs, and the tile's batches are screen batches
            assert!(f.batches.iter().all(|b| b.screen));
            assert_eq!(f.batches.iter().map(|b| b.kind).collect::<Vec<_>>(), [Kind::Quad, Kind::Run]);
        }
    }

    #[test]
    fn hidden_panels_leave_the_status_line_and_the_buttons() {
        let mut h = hud(true);
        h.panels = false;
        let l = lay(&h, 1280.0, 800.0, 1.0);
        assert!(l.panels.is_empty() && l.buttons.len() == 11);
        assert_eq!(l.tile.marks.iter().filter(|m| m.kind == Kind::Quad).count(), 11);
        sound(&l, 1280.0, 800.0);
    }

    #[test]
    fn values_share_a_column_and_a_long_value_wraps_inside_its_panel() {
        let p = Panel {
            heading: "THE CURVE / U(R)".into(),
            handle: 9,
            open: true,
            summary: String::new(),
            summary_colour: PAPER,
            items: vec![Item::Row("preset".into(), "16-H Quench Gas (3-body active) and then some more words".into()), Item::Row("asymptote".into(), "-0.933164 Eh".into())],
        };
        let mut t = FakeText { dpr: 1.0, placed: 0 };
        let mut ui = Ui::new(&mut t, 1.0);
        let h = ui.panel(&p, 10.0, 10.0, 300.0, true, false);
        let runs: Vec<&Instance> = ui.marks.iter().filter(|m| m.kind == Kind::Run).collect();
        let left = |m: &Instance| m.pos[0] - 0.5 * m.size[0];
        // heading, "preset", its value's words, "asymptote", its value's two words
        let preset = runs.iter().position(|m| (left(m) - 17.0).abs() < 1.0 && m.pos[1] > 30.0).expect("the first label");
        let column = left(runs[preset + 1]);
        assert!(column > 17.0 + 9.0 * 7.0, "past the longest label");
        let value_lines: std::collections::BTreeSet<i32> = runs[preset + 1..].iter().map(|m| m.pos[1] as i32).collect();
        assert!(value_lines.len() >= 3, "the long value wrapped: {value_lines:?}");
        for m in &runs {
            assert!(m.pos[0] + 0.5 * m.size[0] <= 10.0 + 300.0 - 7.0 + 0.5, "a run leaves the panel");
            assert!(m.pos[1] + 0.5 * m.size[1] <= 10.0 + h);
        }
        // every line of a value starts at the column
        let starts: Vec<f32> = value_lines.iter().map(|y| runs[preset + 1..].iter().filter(|m| m.pos[1] as i32 == *y && left(m) >= column - 0.5).map(|m| left(m)).fold(f32::MAX, f32::min)).collect();
        assert!(starts.iter().all(|x| (x - column).abs() < 1.0), "{starts:?} against {column}");
    }

    #[test]
    fn tagged_text_carries_the_whole_strings_digest_and_how_it_was_drawn() {
        use crate::{digest::sha256, Bind, Drawn, Tri};
        let mut t = FakeText { dpr: 1.0, placed: 0 };
        let mut ui = Ui::new(&mut t, 1.0);
        let s = "forty two tokens to the steward of the commons";
        ui.quad(0.0, 0.0, 200.0, 200.0, 0.0, PAPER, 0);
        ui.text_flow(s, BODY, [0x17, 0x20, 0x1d, 255], 10.0, 10.0, 120.0, false, 9);
        let runs: Vec<&Instance> = ui.marks.iter().filter(|m| m.kind == Kind::Run).collect();
        assert!(runs.len() > 3, "wrapped into words");
        assert!(runs.iter().all(|m| m.tag == 9 && m.bind == Bind::Source { sha256: sha256(s.as_bytes()), drawn: Drawn::Exact }));
        // untagged text carries nothing
        ui.flow("decoration", BODY, PAPER, 10.0, 150.0, 120.0, false);
        assert_eq!(ui.marks.last().unwrap().bind, Bind::None);
        // whitespace a flow cannot draw as given
        for odd in ["two  spaces", " leading", "trailing ", "a\ttab"] {
            ui.text_flow(odd, BODY, PAPER, 10.0, 170.0, 300.0, false, 10);
            assert!(matches!(ui.marks.last().unwrap().bind, Bind::Source { drawn: Drawn::Changed, .. }), "{odd:?}");
        }
        ui.text_flow("two\nlines", BODY, PAPER, 10.0, 170.0, 300.0, false, 11);
        assert!(matches!(ui.marks.last().unwrap().bind, Bind::Source { drawn: Drawn::Exact, .. }), "a line break is kept");
        // the whole value reports one source, legible on its opaque ground
        let marks = ui.marks.clone();
        let tile = Tile { id: 1, origin: [0.0; 3], version: 1, layer: 0, screen: true, params: crate::PARAMS_NEUTRAL, marks };
        let f = Scene { tiles: vec![tile], modal: None }.frame(&Camera::map([0.0; 3], 1.0, 800.0, 600.0), 1, &Ring, u64::MAX).unwrap();
        let r = f.report(9).unwrap();
        assert_eq!(r.source, Some(sha256(s.as_bytes())));
        assert!(r.legible, "{:?}", r.why);
        assert_eq!(f.report(10).unwrap().why.untruncated, Tri::No);
    }

    #[test]
    fn a_buttons_label_is_part_of_the_button() {
        use crate::{digest::sha256, Bind};
        let mut t = FakeText { dpr: 1.0, placed: 0 };
        let mut ui = Ui::new(&mut t, 1.0);
        ui.button(&Button { label: "confirm".into(), handle: 77, active: true }, 10.0, 10.0, 120.0, 34.0);
        let run = ui.marks.iter().find(|m| m.kind == Kind::Run).unwrap();
        assert_eq!((run.tag, run.bind), (77, Bind::Source { sha256: sha256(b"confirm"), drawn: crate::Drawn::Exact }));
    }
}
