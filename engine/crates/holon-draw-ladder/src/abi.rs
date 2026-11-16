//! The ladder's wasm boundary: plain `extern "C"`, the same pattern as holon-draw-atoms.
//!
//! A host sets its band once (`hl_band`), then each frame: `hl_advance(dt)`, `hl_view(...)`
//! which builds the passes and returns how many, then for each pass its matrix, weight and
//! batches. The caption (the rung's name, status, source and scale bar) is drawn by the
//! engine as a screen panel in the LAST pass, through the host's text provider (the `env`
//! import `host_text`, as the atoms page has it).
use crate::{Ladder, Pass};
use holon_draw::ladder::{host_weight, Band};
use holon_draw::ui::{Text, Ui};
use holon_draw::{Instance, Kind, Tile, PARAMS_NEUTRAL};
use holon_draw_atoms::HostText;
use std::cell::RefCell;

struct State {
    ladder: Ladder,
    passes: Vec<Pass>,
    vp: Vec<[f32; 16]>,
    text: HostText,
    caption_key: String,
}

thread_local! {
    static S: RefCell<Option<State>> = const { RefCell::new(None) };
}

fn with<R>(none: R, f: impl FnOnce(&mut State) -> R) -> R {
    S.with(|s| s.borrow_mut().as_mut().map(f).unwrap_or(none))
}

/// Set the host's band and switch (`clamp` 1 = ON: the host's band only). Builds the ladder.
#[no_mangle]
pub extern "C" fn hl_band(z_lo: f64, z_hi: f64, clamp: u32) {
    let band = Band { z_lo, z_hi, clamp: clamp != 0 };
    S.with(|s| {
        let mut s = s.borrow_mut();
        match s.as_mut() {
            Some(st) => {
                st.ladder.band = band;
                let scenes = std::mem::take(&mut st.ladder.scenes);
                st.ladder = Ladder::new(scenes, band);
            }
            None => *s = Some(State { ladder: Ladder::standard(band), passes: Vec::new(), vp: Vec::new(), text: HostText::new(1.0), caption_key: String::new() }),
        }
    })
}

/// The zoom a host may reach under its switch.
#[no_mangle]
pub extern "C" fn hl_clamp_z(z: f64) -> f64 {
    with(z, |s| s.ladder.band.clamp_z(z))
}

/// The host's own weight at `z` (1 inside its band; fading beyond it when the switch is off).
#[no_mangle]
pub extern "C" fn hl_host_weight(z: f64) -> f64 {
    with(1.0, |s| host_weight(&s.ladder.band, z))
}

#[no_mangle]
pub extern "C" fn hl_advance(dt: f64) {
    with((), |s| s.ladder.advance(dt))
}

/// The number of rungs, and each rung's name, span and status as text (for a host's own
/// legend); `hl_rung_text(i)` returns the length of a UTF-8 string at `hl_text_ptr()`.
#[no_mangle]
pub extern "C" fn hl_rungs() -> u32 {
    with(0, |s| s.ladder.rungs.len() as u32)
}

thread_local! {
    static OUT: RefCell<String> = const { RefCell::new(String::new()) };
    static SWITCH_LINE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// A line the host wants under the caption (its zoom and switch state), as UTF-8 at `ptr`;
/// `len` 0 removes it. The host owns the words.
#[no_mangle]
pub extern "C" fn hl_caption_line(ptr: *const u8, len: u32) {
    let line = if len == 0 || ptr.is_null() {
        None
    } else {
        // SAFETY: the host wrote `len` bytes of UTF-8 at `ptr` in this module's memory.
        let b = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
        std::str::from_utf8(b).ok().map(str::to_string)
    };
    SWITCH_LINE.with(|l| *l.borrow_mut() = line);
}

/// Scratch memory for the host to write a string into.
#[no_mangle]
pub extern "C" fn hl_scratch(len: u32) -> *mut u8 {
    SCRATCH.with(|s| {
        let mut s = s.borrow_mut();
        s.clear();
        s.resize(len as usize, 0);
        s.as_mut_ptr()
    })
}

thread_local! {
    static SCRATCH: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

#[no_mangle]
pub extern "C" fn hl_rung_text(i: u32) -> u32 {
    let t = with(String::new(), |s| {
        let (Some(r), Some(sc)) = (s.ladder.rungs.get(i as usize), s.ladder.scenes.get(i as usize)) else { return String::new() };
        format!("{}\t{:.2}\t{:.2}\t{}\t{}", r.name, r.z_lo, r.z_hi, r.status.label(), sc.source())
    });
    OUT.with(|o| {
        *o.borrow_mut() = t;
        o.borrow().len() as u32
    })
}

#[no_mangle]
pub extern "C" fn hl_text_ptr() -> *const u8 {
    OUT.with(|o| o.borrow().as_ptr())
}

/// Build the passes for `z` (log10 metres per device pixel, already clamped by the host).
/// Returns how many there are.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn hl_view(z: f64, yaw: f64, pitch: f64, width: f64, height: f64, dpr: f64, clock: f64) -> u32 {
    with(0, |s| {
        s.passes = s.ladder.passes(z, yaw, pitch, width, height, dpr, clock);
        s.vp = s.passes.iter().map(|p| p.camera.relative()).collect();
        // the caption, drawn by the engine, rides in the last pass as a screen tile
        if let Some(cap) = s.ladder.caption(z, height) {
            if s.text.dpr != dpr as f32 {
                s.text = HostText::new(dpr as f32);
            }
            let key = format!("{}|{}|{}", cap.0, cap.3, dpr);
            if key != s.caption_key {
                s.caption_key = key;
            }
            let tile = caption_tile(&mut s.text, cap, width as f32, height as f32, dpr as f32);
            if let Some(last) = s.passes.last_mut() {
                let mut scene = holon_draw::Scene { tiles: vec![tile], modal: None };
                if let Ok(f) = scene.frame(&last.camera, 1, &holon_draw::Ring, u64::MAX) {
                    last.frame.batches.extend(f.batches);
                }
                scene.tiles.clear();
            }
        }
        s.passes.len() as u32
    })
}

fn caption_tile(text: &mut dyn Text, cap: (&str, &str, &str, String, f64), width: f32, height: f32, dpr: f32) -> Tile {
    let mut ui = Ui::new(text, dpr);
    let pad = ui.px(10.0);
    let w = (ui.px(360.0)).min(width - 2.0 * ui.px(16.0));
    let x = ui.px(16.0);
    // measure the source line's wrapped height by laying it out once off-screen
    let mut probe = Ui::new(ui.text, dpr);
    let src_h = probe.flow(cap.2, 4, [0; 4], 0.0, 0.0, w - 2.0 * pad, false);
    let extra = if SWITCH_LINE.with(|l| l.borrow().is_some()) { ui.line(4) } else { 0.0 };
    let h = pad * 2.0 + ui.line(1) + ui.line(0) + src_h + extra + ui.px(26.0);
    let y = height - h - ui.px(16.0);
    ui.quad(x, y, w, h, ui.px(6.0), [23, 32, 29, 214], 0);
    let mut cy = y + pad;
    ui.run(cap.0, 1, [112, 166, 142, 255], x + pad, cy);
    cy += ui.line(1);
    ui.run(cap.1, 0, [237, 233, 223, 255], x + pad, cy);
    cy += ui.line(0);
    cy += ui.flow(cap.2, 4, [105, 115, 110, 255], x + pad, cy, w - 2.0 * pad, false);
    if let Some(line) = SWITCH_LINE.with(|l| l.borrow().clone()) {
        ui.run(&line, 4, [237, 233, 223, 200], x + pad, cy);
        cy += ui.line(4);
    }
    // the scale bar: a line of the stated length and its name
    let name = cap.3.clone();
    let by = cy + ui.px(12.0);
    let len = (cap.4 as f32).round().min(w - 2.0 * pad - ui.px(90.0)).max(1.0);
    let mut bar = Instance::new(Kind::Line, [x + pad, by, 0.0], [ui.px(2.0), 0.0, 0.0]).colour([237, 233, 223, 230]);
    bar.aux = [x + pad + len, by, 0.0];
    ui.marks.push(bar);
    ui.run(&name, 4, [237, 233, 223, 255], x + pad + len + ui.px(8.0), by - 0.5 * ui.line(4));
    Tile { id: 9999, origin: [0.0; 3], version: 1, layer: 250, screen: true, params: PARAMS_NEUTRAL, marks: std::mem::take(&mut ui.marks) }
}

fn pass<R>(p: u32, none: R, f: impl FnOnce(&Pass, &[f32; 16]) -> R) -> R {
    with(none, |s| match (s.passes.get(p as usize), s.vp.get(p as usize)) {
        (Some(x), Some(v)) => f(x, v),
        _ => panic!("no such pass"),
    })
}

#[no_mangle]
pub extern "C" fn hl_pass_weight(p: u32) -> f64 {
    pass(p, 0.0, |x, _| x.weight)
}
#[no_mangle]
pub extern "C" fn hl_pass_vp(p: u32) -> *const f32 {
    with(std::ptr::null(), |s| s.vp.get(p as usize).map_or(std::ptr::null(), |v| v.as_ptr()))
}
#[no_mangle]
pub extern "C" fn hl_pass_batches(p: u32) -> u32 {
    with(0, |s| s.passes.get(p as usize).map_or(0, |x| x.frame.batches.len() as u32))
}
fn batch<R>(p: u32, i: u32, none: R, f: impl FnOnce(&holon_draw::Batch) -> R) -> R {
    with(none, |s| s.passes.get(p as usize).and_then(|x| x.frame.batches.get(i as usize)).map(f).unwrap_or_else(|| panic!("no such batch")))
}
#[no_mangle]
pub extern "C" fn hl_batch_kind(p: u32, i: u32) -> u32 {
    batch(p, i, 0, |b| b.kind as u32)
}
#[no_mangle]
pub extern "C" fn hl_batch_count(p: u32, i: u32) -> u32 {
    batch(p, i, 0, |b| b.count)
}
#[no_mangle]
pub extern "C" fn hl_batch_ptr(p: u32, i: u32) -> *const u8 {
    batch(p, i, std::ptr::null(), |b| b.bytes.as_ptr())
}
#[no_mangle]
pub extern "C" fn hl_batch_screen(p: u32, i: u32) -> u32 {
    batch(p, i, 0, |b| b.screen as u32)
}
/// 1 if the batch's tile was drawn coarse (a backend may use its cheaper meshes).
#[no_mangle]
pub extern "C" fn hl_batch_coarse(p: u32, i: u32) -> u32 {
    batch(p, i, 0, |b| b.coarse as u32)
}
#[no_mangle]
pub extern "C" fn hl_batch_offset(p: u32, i: u32, axis: u32) -> f32 {
    batch(p, i, 0.0, |b| b.offset.get(axis as usize).copied().unwrap_or(0.0))
}
#[no_mangle]
pub extern "C" fn hl_batch_params(p: u32, i: u32) -> *const f32 {
    batch(p, i, std::ptr::null(), |b| b.params.as_ptr())
}
/// After the host's atlas was emptied: forget the caption's placements.
#[no_mangle]
pub extern "C" fn hl_text_stale() {
    with((), |s| s.text.forget())
}
