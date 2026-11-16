//! The acuity gates on the real ladder (`holon-draw/tests/gates/mod.rs` holds what they
//! share with the core's gates and with the browser probe; `holon-draw/results/ACUITY_PROBE.md`
//! says what each shows and does not).
//!
//! Two gates here, both on states and poses a run of the ladder visits:
//!
//! * the pick and the report of every rung's frame are the same whether the allocator draws
//!   it as it chooses, all fine, or all coarse, and what it leaves coarse is within its budget
//!   (and the same gate fires when two tags are exchanged in one of those frames);
//! * DRAWING NEVER WRITES: the simulated rungs, run frame by frame with everything drawn fine
//!   and again with everything drawn coarse, end in the same state bit for bit.
#[path = "../../holon-draw/tests/gates/mod.rs"]
mod gates;

use gates::*;
use holon_draw::ladder::{rung_camera, Band, Rung, Z_MAX, Z_MIN};
use holon_draw::{Acuity, Allocator, Camera, Frame, Tile, PARAM_DIM};
use holon_draw_ladder::scenes::micro::{Atoms, Hadron};
use holon_draw_ladder::{Ladder, Scene};
use std::sync::Mutex;

/// The hadron band's solver is one per process: tests that step it take turns.
static SOLVER: Mutex<()> = Mutex::new(());
const NOW: f64 = 1_794_000_000.0;
const NO_HOST: Band = Band { z_lo: 0.0, z_hi: 0.0, clamp: false };

/// The tiles a pass was built from: the scene asked again exactly as `Ladder::passes` asked
/// it, with the cross-fade weight put where `passes` puts it.
fn tiles_of(l: &mut Ladder, rung: usize, z: f64, clock: f64, weight: f64) -> Vec<Tile> {
    let mut tiles = l.scenes[rung].tiles(z, clock, weight);
    for t in tiles.iter_mut() {
        t.params[PARAM_DIM + 3] *= weight as f32;
    }
    tiles
}

/// Two frames the same, to the bit (a mark behind the view has no place on screen, and a
/// number that is not one never equals itself, so `==` will not do).
fn same_frame(a: &Frame, b: &Frame) -> bool {
    let bits = |f: &Frame| f.marks.iter().map(|m| (m.tag, m.layer, m.coarse, m.on_screen, [m.x, m.y, m.r, m.depth].map(f64::to_bits), m.legible, m.shown, m.rect.map(|r| r.map(f64::to_bits)))).collect::<Vec<_>>();
    a.batches == b.batches && (a.fine, a.coarse, a.stale, a.modal) == (b.fine, b.coarse, b.stale, b.modal) && a.remainder_px.to_bits() == b.remainder_px.to_bits() && bits(a) == bits(b)
}

#[test]
fn on_the_ladder_a_coarse_frame_keeps_the_pick_and_the_report_and_stays_within_its_budget() {
    let _turn = SOLVER.lock().unwrap_or_else(|p| p.into_inner());
    let mut l = Ladder::standard(NO_HOST);
    let (mut frames, mut coarse_tiles_seen, mut tagged, mut worst) = (0u32, 0u32, 0u32, 0.0f64);
    // the page's opening pose, a low one and a steep one; a desk window and a phone's
    let mut planted = 0u32;
    for (yaw, pitch, w, h, step) in [(0.8, 0.42, 1280.0, 800.0, 0.05), (-1.9, 0.1, 1280.0, 800.0, 0.2), (2.6, 1.2, 1170.0, 2532.0, 0.2)] {
        let mut z = Z_MIN + 0.05;
        let mut k = 0u32;
        while z < Z_MAX {
            l.advance(1.0 / 60.0);
            let clock = NOW + k as f64 / 60.0;
            for p in l.passes(z, yaw, pitch, w, h, 1.0, clock) {
                let tiles = tiles_of(&mut l, p.rung, z, clock, p.weight);
                let real = frame(&tiles, &p.camera, 1, &Acuity { eps_px: 1.0 });
                assert!(same_frame(&real, &p.frame), "z = {z}: the gate is not looking at the frame the ladder drew");
                let (fine, forced) = (frame(&tiles, &p.camera, 1, &AllFine), frame(&tiles, &p.camera, 1, &AllCoarse));
                let points = pick_points(&p.camera, &[&fine, &real], 12);
                for (name, f) in [("as allocated", &real), ("all coarse", &forced)] {
                    let d = views_disagree(&fine, f, &points);
                    assert!(d.is_empty(), "{} at z = {z}, {name}: {d:?}", l.rungs[p.rung].name);
                }
                // THE PLANT, on the ladder's own frames: wherever two tagged marks are on screen,
                // a coarse draw that exchanged their tags must make the same gate fire
                let on: Vec<u32> = real.marks.iter().filter(|m| m.on_screen).map(|m| m.tag).collect();
                if on.len() >= 2 && on[0] != on[1] {
                    assert!(!views_disagree(&fine, &plant_swap_tags(&real, on[0], on[1]), &points).is_empty(), "z = {z}: the gate did not fire on the plant");
                    planted += 1;
                }
                // the certificate: what is left coarse is within the pixel the allocator was given,
                // and it is the largest of the coarse tiles' own bounds
                assert!(real.remainder_px <= 1.0, "z = {z}: {}", real.remainder_px);
                let coarse = coarse_tiles(&real);
                let own = tiles.iter().filter(|t| coarse.contains(&t.id)).map(|t| tile_bound(t, &p.camera)).fold(0.0, f64::max);
                assert!(own <= real.remainder_px, "z = {z}: a coarse tile's bound {own} is over the frame's {}", real.remainder_px);
                frames += 1;
                coarse_tiles_seen += coarse.len() as u32;
                tagged += real.marks.len() as u32;
                worst = worst.max(real.remainder_px);
            }
            z += step;
            k += 1;
        }
    }
    println!("{frames} frames, {coarse_tiles_seen} coarse tiles, {tagged} tagged marks; the largest certificate {worst:.3} px");
    println!("the plant (two tags exchanged) fired on all {planted} frames that show two tagged marks");
    assert!(frames > 1200 && coarse_tiles_seen > 500 && tagged > 60 && planted > 20, "the sweep must reach coarse tiles and tagged marks, or it gates nothing");
}

/// One simulated rung run for `n` frames from a fresh scene, every frame framed by `alloc`
/// at a zoom that walks across the rung's span. Returns, per frame, the state's bits and the
/// frame's reports' bits.
fn run<S: Scene>(mut scene: S, state: impl Fn(&S) -> Vec<u64>, alloc: &dyn Allocator, n: usize) -> (Vec<Vec<u64>>, Vec<Vec<u64>>, Vec<Frame>, u32) {
    let (lo, hi) = scene.span();
    let rung = Rung { name: scene.name(), z_lo: lo, z_hi: hi, unit: scene.unit(), status: scene.status() };
    let (mut states, mut reports, mut frames, mut coarse) = (Vec::new(), Vec::new(), Vec::new(), 0);
    for k in 0..n {
        scene.advance(1.0 / 60.0);
        let z = lo + (hi - lo) * (0.1 + 0.8 * k as f64 / n as f64);
        let cam: Camera = rung_camera(&rung, z, [0.0; 3], 0.8 + 0.01 * k as f64, 0.42, std::f64::consts::FRAC_PI_4, 1280.0, 800.0);
        let tiles = scene.tiles(z, NOW + k as f64 / 60.0, 1.0);
        let f = frame(&tiles, &cam, 1, alloc);
        coarse += f.coarse;
        // every report value that is not the allocation's own statement of itself
        let mut tags: Vec<u32> = f.marks.iter().map(|m| m.tag).collect();
        tags.sort_unstable();
        tags.dedup();
        let mut r = Vec::new();
        for t in tags {
            let s = f.report(t).unwrap();
            r.extend([s.tag as u64, s.marks as u64, s.on_screen as u64, s.legible as u64]);
            r.extend(s.bounds.map(f64::to_bits));
        }
        states.push(state(&scene));
        reports.push(r);
        frames.push(f);
    }
    (states, reports, frames, coarse)
}

fn atoms_state(a: &Atoms) -> Vec<u64> {
    let mut s = vec![a.substeps, a.world.sim.n as u64, a.bond().to_bits()];
    for at in a.world.sim.atoms.iter().take(a.world.sim.n) {
        s.extend([at.x, at.y, at.z, at.vx, at.vy, at.vz].map(f64::to_bits));
    }
    s
}

fn hadron_state(h: &Hadron) -> Vec<u64> {
    h.occ.iter().chain(&h.links).chain([&h.time, &h.norm, &h.energy]).map(|v| v.to_bits()).collect()
}

#[test]
fn drawing_never_writes_the_hydrogen_rung() {
    const N: usize = 240;
    let (fine, fine_reports, fine_frames, c0) = run(Atoms::new(), atoms_state, &AllFine, N);
    let (coarse, coarse_reports, coarse_frames, c1) = run(Atoms::new(), atoms_state, &AllCoarse, N);
    let (again, ..) = run(Atoms::new(), atoms_state, &AllFine, N);
    // the gate is not empty: the simulation moved, one run drew nothing coarse and the other did
    assert!(fine[0] != fine[N - 1], "the simulation did not step");
    assert!(c0 == 0 && c1 as usize >= N, "{c0} {c1}");
    assert!(fine == again, "CONTROL: the same seed and clock must give the same run");
    assert!(fine == coarse, "the simulation's state depends on how it was drawn");
    assert!(fine_reports == coarse_reports, "a report value depends on how it was drawn");
    // the two views, frame for frame, and the records themselves
    for (k, (a, b)) in fine_frames.iter().zip(&coarse_frames).enumerate() {
        let cam = Camera::map([0.0; 3], 1.0, 1280.0, 800.0);
        assert!(views_disagree(a, b, &pick_points(&cam, &[a, b], 8)).is_empty(), "frame {k}");
        assert!(a.digest() == b.digest(), "frame {k}: the records drawn differ");
    }
    println!("hydrogen: {N} frames, {} substeps, the bond ends at {:.6} Å in both", fine[N - 1][0], f64::from_bits(fine[N - 1][2]));
}

#[test]
fn drawing_never_writes_the_hadron_chain() {
    let _turn = SOLVER.lock().unwrap_or_else(|p| p.into_inner());
    const N: usize = 240;
    let (fine, fine_reports, fine_frames, c0) = run(Hadron::new(), hadron_state, &AllFine, N);
    let (coarse, coarse_reports, coarse_frames, c1) = run(Hadron::new(), hadron_state, &AllCoarse, N);
    let (again, ..) = run(Hadron::new(), hadron_state, &AllFine, N);
    assert!(fine[0] != fine[N - 1], "the chain did not step");
    assert!(c0 == 0 && c1 as usize >= N, "{c0} {c1}");
    assert!(fine == again, "CONTROL: the same seed and clock must give the same run");
    assert!(fine == coarse, "the chain's state depends on how it was drawn");
    assert!(fine_reports == coarse_reports);
    for (k, (a, b)) in fine_frames.iter().zip(&coarse_frames).enumerate() {
        assert!(a.digest() == b.digest(), "frame {k}: the records drawn differ");
    }
    let last = &fine[N - 1];
    println!("hadron chain: {N} frames, model time {:.4}, norm {:.12} in both", f64::from_bits(last[last.len() - 3]), f64::from_bits(last[last.len() - 2]));
}

#[test]
fn drawing_never_writes_a_whole_ladder_on_a_zoom() {
    // The whole standard ladder flown from the ground to the end of the record, twice: every
    // pass also framed all fine in one flight and all coarse in the other. What each scene
    // hands the allocator on every frame (its tiles, which are its state as drawn) must be the
    // same in both flights, to the bit.
    let _turn = SOLVER.lock().unwrap_or_else(|p| p.into_inner());
    let fly = |alloc: &dyn Allocator| {
        let mut l = Ladder::standard(NO_HOST);
        let (mut fold, mut coarse, mut n) = (Vec::new(), 0u32, 0u32);
        let mut z = -2.0;
        let mut k = 0u32;
        while z > Z_MIN + 0.1 {
            l.advance(1.0 / 60.0);
            let clock = NOW + k as f64 / 60.0;
            for p in l.passes(z, 0.8, 0.42, 1280.0, 800.0, 1.0, clock) {
                let tiles = tiles_of(&mut l, p.rung, z, clock, p.weight);
                let f = frame(&tiles, &p.camera, 1, alloc);
                coarse += f.coarse;
                n += 1;
                // the tiles' own bits: every mark's packed record, each tile's origin and settings
                let mut bytes = Vec::new();
                for t in &tiles {
                    bytes.extend(t.id.to_le_bytes());
                    bytes.extend(t.origin.iter().flat_map(|v| v.to_le_bytes()));
                    bytes.extend(t.params.iter().flat_map(|v| v.to_le_bytes()));
                    for m in &t.marks {
                        m.pack(&mut bytes);
                    }
                }
                fold.push((p.rung, bytes));
            }
            z -= 0.02;
            k += 1;
        }
        (fold, coarse, n)
    };
    let (fine, c0, n) = fly(&AllFine);
    let (coarse, c1, _) = fly(&AllCoarse);
    assert!(c0 == 0 && c1 > 1000 && n > 800, "{c0} {c1} {n}");
    assert!(fine.len() == coarse.len());
    for (k, (a, b)) in fine.iter().zip(&coarse).enumerate() {
        assert!(a == b, "pass {k} (rung {}): what the scene handed over differs between the two flights", a.0);
    }
    println!("whole ladder: {n} passes in each flight, {c1} tiles drawn coarse in the second; every scene's tiles identical");
}
