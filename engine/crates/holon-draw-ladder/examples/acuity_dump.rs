//! Writes the cases `holon-draw/tools/acuity_probe.py` draws in a browser: the plants, a ball
//! seen from every side, and the standard ladder on a zoom, each as the fine frame and the
//! frame under test with every tile's certificate read from the core.
//!
//!     cargo run -p holon-draw-ladder --example acuity_dump -- <out dir> [survey]
//!
//! Writes `<out dir>/cases.json` and `<out dir>/cases.bin`. With `survey`, prints what the
//! ladder leaves coarse at each zoom and writes nothing.
#[path = "../../holon-draw/tests/gates/mod.rs"]
mod gates;

use gates::*;
use holon_draw::ladder::{rung_camera, Band, Rung, Status, Z_MAX, Z_MIN};
use holon_draw::{Acuity, Camera, Frame, Instance, Kind, Tile, INSTANCE_BYTES, PARAM_DIM};
use holon_draw_ladder::{screen_axes, Ladder};
use std::collections::BTreeMap;
use std::f64::consts::{FRAC_PI_4, PI};

const NOW: f64 = 1_794_000_000.0;
/// The windows: a desk, a phone at three device pixels a point, a 4K screen.
const WINDOWS: [(f64, f64, &str); 3] = [(1280.0, 800.0, "1280x800"), (1170.0, 2532.0, "1170x2532"), (3840.0, 2160.0, "3840x2160")];

fn num(v: f64) -> String {
    format!("{v}")
}

/// The plants of gate (a): errors of known size.
fn plants(d: &mut Dump) {
    let cam = Camera::map([0.0; 3], 1.0, 400.0, 300.0);
    // SHIFT: a disc and a box, moved k pixels to the right only in the frame under test. The
    // core prices this tile at 0 (nothing in it is coarsened), so the probe must read k AND
    // find the certificate killed: the falsifier firing on a planted kill.
    let marks = |dx: f32| vec![Instance::new(Kind::Disc, [-80.0 + dx, 0.0, 0.0], [60.0, 0.0, 0.0]).colour([200, 90, 60, 255]), Instance::new(Kind::Box, [90.0 + dx, 0.0, 0.0], [50.0, 50.0, 20.0]).colour([70, 130, 180, 255])];
    let still = vec![tile(1, marks(0.0))];
    let fine = frame(&still, &cam, 1, &AllFine);
    for k in [1.0f32, 2.0, 3.0, 5.0, 8.0, 0.25, 0.5] {
        let moved = frame(&[tile(1, marks(k))], &cam, 1, &AllFine);
        let whole = k.fract() == 0.0;
        // a close look at the disc's right edge for the shifts under a pixel
        let probes = if whole { Vec::new() } else { vec![Probe { x: 180.0, y: 150.0, side: 24.0, bound_px: 0.0, kind: Kind::Disc }] };
        let info = [TileInfo { id: 1, bound_px: tile_bound(&still[0], &cam), coarse: true, probes }];
        d.case(&format!("shift {k} px"), "plant_shift", &cam, &fine, &moved, &info, &[("measure", quote(if whole { "native" } else { "crop" })), ("expect_px", num(k as f64)), ("expect_kill", "true".into())]);
    }
    // BALL: 100 px in radius seen down its pole, where the cheap mesh's outline is a decagon
    // whose edges fall short of the circle by k = r (1 - cos(pi/10)); and one of 10 px, under
    // a pixel, read through a close look.
    for (r, measure) in [(100.0f32, "native"), (10.0, "crop")] {
        let t = [tile(1, vec![Instance::new(Kind::Sphere, [0.0; 3], [r, 0.0, 0.0]).colour([204, 58, 50, 255])])];
        let (fine, coarse) = (frame(&t, &cam, 1, &AllFine), frame(&t, &cam, 1, &AllCoarse));
        let k = r as f64 * (1.0 - (PI / 10.0).cos());
        let mut info = tile_infos(&t, &cam, &coarse, 1);
        if measure == "native" {
            info[0].probes.clear();
        }
        // against the FULL mesh, not the true ball: its own outline is up to r (1 - cos(pi/28)) inside
        d.case(&format!("ball {r} px, pole on"), "plant_ball", &cam, &fine, &coarse, &info, &[("measure", quote(measure)), ("expect_px", num(k)), ("expect_slack_px", num(r as f64 * (1.0 - (PI / 28.0).cos()))), ("expect_kill", "false".into())]);
    }
    // DROP: a detailed, untagged disc 20 px in radius beside a pinned one; coarse drops it.
    let t = [tile(1, vec![Instance::new(Kind::Disc, [-60.0, 0.0, 0.0], [20.0, 0.0, 0.0]).detail(1).colour([60, 160, 90, 255]), Instance::new(Kind::Disc, [60.0, 0.0, 0.0], [20.0, 0.0, 0.0]).pinned().colour([60, 160, 90, 255])])];
    let (fine, coarse) = (frame(&t, &cam, 1, &AllFine), frame(&t, &cam, 1, &AllCoarse));
    let info = [TileInfo { id: 1, bound_px: tile_bound(&t[0], &cam), coarse: true, probes: Vec::new() }];
    d.case("a dropped disc 40 px across", "plant_drop", &cam, &fine, &coarse, &info, &[("measure", quote("native")), ("expect_px", num(40.0)), ("expect_slack_px", num(40.0 * (1.0 - (PI / 24.0).cos()))), ("expect_kill", "false".into())]);
}

/// The plants of gate (b): a coarse draw that moves no pixel. The verdict of the pick-and-
/// report gate is computed here and travels with the case; the browser reads the pixels.
fn zero_pixel(d: &mut Dump) {
    let (tiles, cam) = zero_pixel_scene();
    let fine = frame(&tiles, &cam, 1, &AllFine);
    let coarse = frame(&tiles, &cam, 1, &AllCoarse);
    let points = pick_points(&cam, &[&fine, &coarse], 16);
    let info = [TileInfo { id: 1, bound_px: tile_bound(&tiles[0], &cam), coarse: true, probes: Vec::new() }];
    for (name, f) in [("control: the core's own coarse frame", coarse.clone()), ("plant: a hidden tagged mark dropped", plant_drop_coarse_tagged(&coarse)), ("plant: two tags exchanged", plant_swap_tags(&coarse, 3, 4))] {
        let dis = views_disagree(&fine, &f, &points);
        let list = format!("[{}]", dis.iter().take(4).map(|s| quote(s)).collect::<Vec<_>>().join(","));
        d.case(name, "plant_zero", &cam, &fine, &f, &info, &[("measure", quote("rgba")), ("views_gate_disagreements", num(dis.len() as f64)), ("views_gate_first", list), ("points_picked", num(points.len() as f64)), ("records_fine", num(fine.drawn() as f64)), ("records_test", num(f.drawn() as f64))]);
    }
}

/// One ball seen from every side: where the cheap mesh's outline is worst.
fn ball_views(d: &mut Dump) {
    // FROM FAR (a view narrow enough to be orthographic to a part in a million): the mesh's
    // own effect, 100 px of radius, every 5 degrees of pitch and 6 of yaw over one face.
    let t = [tile(1, vec![Instance::new(Kind::Sphere, [0.0; 3], [1.0, 0.0, 0.0]).colour([204, 58, 50, 255])])];
    for ip in 0..=18 {
        for iy in 0..=6 {
            let (pitch, yaw) = ((5.0 * ip as f64).to_radians(), (6.0 * iy as f64).to_radians());
            let cam = Camera::orbit([0.0; 3], yaw, pitch, 2000.0, 2.0 * (0.001f64).atan(), 400.0, 400.0);
            let (fine, coarse) = (frame(&t, &cam, 1, &AllFine), frame(&t, &cam, 1, &AllCoarse));
            let mut info = tile_infos(&t, &cam, &coarse, 1);
            info[0].probes.clear();
            d.case(&format!("ball from far, pitch {} yaw {}", 5 * ip, 6 * iy), "ball_far", &cam, &fine, &coarse, &info, &[("measure", quote("native")), ("r_px", num(100.0)), ("pitch_deg", num(5.0 * ip as f64)), ("yaw_deg", num(6.0 * iy as f64))]);
        }
    }
    // WITH THE LADDER'S CAMERA (`rung_camera`: the eye's distance fixed by the zoom, the field
    // of view widening with the window): a ball of 60 px nominal radius at the view's centre
    // and in its top right corner, on each window. Off the axis a perspective view stretches a
    // ball along the line from the centre; `Camera::pixels` prices it at its nominal size.
    let rung = Rung { name: "", z_lo: -1.0, z_hi: 1.0, unit: 0, status: Status::Illustration };
    let r = 60.0f64;
    for (w, h, win) in [(1280.0, 800.0, "1280x800"), (1920.0, 1080.0, "1920x1080"), (1170.0, 2532.0, "1170x2532"), (2560.0, 1440.0, "2560x1440"), (3840.0, 2160.0, "3840x2160")] {
        for (place, fx, fy) in [("centre", 0.0, 0.0), ("corner", 1.0, 1.0)] {
            for ip in 0..=8 {
                for iy in 0..=3 {
                    let (pitch, yaw) = ((10.0 * ip as f64).to_radians(), (9.0 * iy as f64).to_radians());
                    let cam = rung_camera(&rung, 0.0, [0.0; 3], yaw, pitch, FRAC_PI_4, w, h);
                    // in the focal plane a pixel is a unit: put the ball 2.6 radii in from the corner
                    let (a, b) = (fx * (0.5 * w - 2.6 * r), fy * (0.5 * h - 2.6 * r));
                    let (rt, up) = screen_axes(yaw, pitch);
                    let pos = [0, 1, 2].map(|i| (a * rt[i] + b * up[i]) as f32);
                    let t = [tile(1, vec![Instance::new(Kind::Sphere, pos, [r as f32, 0.0, 0.0]).colour([204, 58, 50, 255])])];
                    let (fine, coarse) = (frame(&t, &cam, 1, &AllFine), frame(&t, &cam, 1, &AllCoarse));
                    let mut info = tile_infos(&t, &cam, &coarse, 1);
                    let p = cam.project(pos.map(|v| v as f64)).unwrap();
                    info[0].probes = vec![Probe { x: p[0], y: p[1], side: 5.0 * r, bound_px: info[0].bound_px, kind: Kind::Sphere }];
                    let off = (a * a + b * b).sqrt();
                    d.case(&format!("ball in the {place} of {win}, pitch {} yaw {}", 10 * ip, 9 * iy), "ball_window", &cam, &fine, &coarse, &info, &[("measure", quote("crop")), ("r_px", num(r)), ("window", quote(win)), ("place", quote(place)), ("off_axis_px", num(off)), ("pitch_deg", num(10.0 * ip as f64)), ("yaw_deg", num(9.0 * iy as f64))]);
                }
            }
        }
    }
}

/// Records a backend draws on its cheap mesh whose report does not say so: tagged spheres and
/// rods in a coarse batch, with the coarse bit of their own flags clear.
fn cheap_and_unsaid(f: &Frame) -> u32 {
    f.batches.iter().filter(|b| b.coarse && matches!(b.kind, Kind::Sphere | Kind::Rod)).map(|b| b.bytes.chunks(INSTANCE_BYTES).filter(|r| r[44..48] != [0; 4] && r[50] & 2 == 0).count() as u32).sum()
}

/// The standard ladder on a zoom, as the page flies it: every frame the ladder draws with a
/// tile left coarse becomes a case.
fn ladder(d: &mut Dump, survey: bool) -> String {
    let mut by_rung: BTreeMap<String, (u32, u32, f64, f64, f64, u32)> = BTreeMap::new();
    let mut unsaid = 0u32;
    // the page's opening pose on each window; on the desk window two more poses, coarser in z
    let mut flights: Vec<(f64, f64, f64, f64, &str, f64)> = WINDOWS.iter().map(|&(w, h, n)| (0.8, 0.42, w, h, n, 0.05)).collect();
    flights.push((-1.9, 0.1, 1280.0, 800.0, "1280x800", 0.15));
    flights.push((2.6, 1.2, 1280.0, 800.0, "1280x800", 0.15));
    for (yaw, pitch, w, h, win, step) in flights {
        let mut l = Ladder::standard(Band { z_lo: 0.0, z_hi: 0.0, clamp: false });
        let mut z = Z_MIN + 0.05;
        let mut k = 0u32;
        while z < Z_MAX {
            // a second of the page between poses, so the simulated rungs are where a run has them
            for _ in 0..6 {
                l.advance(1.0 / 60.0);
                let _ = l.passes(z, yaw, pitch, w, h, 1.0, NOW + k as f64);
            }
            let clock = NOW + k as f64 * 0.37;
            for p in l.passes(z, yaw, pitch, w, h, 1.0, clock) {
                let mut tiles: Vec<Tile> = l.scenes[p.rung].tiles(z, clock, p.weight);
                for t in tiles.iter_mut() {
                    t.params[PARAM_DIM + 3] *= p.weight as f32;
                }
                let real = frame(&tiles, &p.camera, 1, &Acuity { eps_px: 1.0 });
                assert!(real.batches == p.frame.batches && real.remainder_px == p.frame.remainder_px, "not the frame the ladder drew");
                let fine = frame(&tiles, &p.camera, 1, &AllFine);
                let info = tile_infos(&tiles, &p.camera, &real, 2);
                let coarse: Vec<&TileInfo> = info.iter().filter(|t| t.coarse).collect();
                let name = l.rungs[p.rung].name;
                let e = by_rung.entry(name.to_string()).or_insert((0, 0, f64::INFINITY, f64::NEG_INFINITY, 0.0, 0));
                e.0 += 1;
                if coarse.iter().all(|t| t.bound_px == 0.0) {
                    continue;
                }
                e.1 += coarse.len() as u32;
                (e.2, e.3, e.4) = (e.2.min(z), e.3.max(z), e.4.max(real.remainder_px));
                e.5 += 1;
                unsaid += cheap_and_unsaid(&real);
                if survey {
                    continue;
                }
                d.case(&format!("{name}, z {z:.2}, {win}, yaw {yaw} pitch {pitch}"), "ladder", &p.camera, &fine, &real, &info, &[("measure", quote(if win == "1280x800" { "both" } else { "crop" })), ("rung", quote(name)), ("z", num((z * 100.0).round() / 100.0)), ("window", quote(win)), ("yaw", num(yaw)), ("pitch", num(pitch)), ("weight", num(p.weight)), ("cheap_and_unsaid", num(cheap_and_unsaid(&real) as f64))]);
            }
            z += step;
            k += 1;
        }
    }
    let mut s = String::from("[");
    for (i, (name, (frames, tiles, lo, hi, rem, cases))) in by_rung.iter().enumerate() {
        println!("{name:44} {frames:5} frames, {cases:4} with a coarse tile that has a bound: {tiles:5} such tiles, z {lo:.2} .. {hi:.2}, largest certificate {rem:.3} px");
        s += &format!("{}{{\"rung\":{},\"frames\":{frames},\"cases\":{cases},\"coarse_tiles\":{tiles},\"largest_certificate_px\":{rem}}}", if i > 0 { "," } else { "" }, quote(name));
    }
    println!("tagged marks drawn on a cheap mesh whose report says they are not coarse: {unsaid}");
    s + "]"
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args.get(1).expect("usage: acuity_dump <out dir> [survey]");
    let survey = args.get(2).is_some_and(|a| a == "survey");
    let mut d = Dump::default();
    plants(&mut d);
    zero_pixel(&mut d);
    ball_views(&mut d);
    let rungs = ladder(&mut d, survey);
    if survey {
        return;
    }
    std::fs::create_dir_all(out).unwrap();
    std::fs::write(format!("{out}/cases.bin"), &d.bin).unwrap();
    std::fs::write(format!("{out}/cases.json"), d.json(&[("eps_px", "1".into()), ("ladder_rungs", rungs)])).unwrap();
    println!("{} cases, {} bytes of records, to {out}", d.cases.len(), d.bin.len());
}
