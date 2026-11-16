use super::*;
use crate::Ladder;
use holon_draw::ladder::{visible, Band, Status, Z_MIN};
use std::time::Instant;

/// The marks when the view is 10^`zv` metres high (a thousand pixels of 10^(zv - 3) m).
fn marks(s: &mut dyn Scene, zv: f64, clock: f64) -> Vec<Instance> {
    s.tiles(zv - VIEW_DECADES, clock, 1.0).into_iter().flat_map(|t| t.marks).collect()
}

/// A clock like a host's: seconds since 1970.
const NOW: f64 = 1.7e9;

#[test]
fn the_rungs_cover_the_vacuum_to_the_street_with_no_gap() {
    let mut spans: Vec<(f64, f64)> = inward().iter().map(|s| s.span()).collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    // from a Planck length across a thousand pixels (the ladder's Z_MIN) to a street: 0.1 m a pixel
    assert_eq!(spans[0].0, Z_MIN);
    assert_eq!(spans.last().unwrap().1, -1.0);
    for w in spans.windows(2) {
        assert_eq!(w[0].1, w[1].0, "a gap or an overlap between {:?} and {:?}", w[0], w[1]);
    }
    // on the ladder, under a host whose street is at 0.1 m a pixel: every z below it is drawn, the
    // weights and the host's summing to one
    let l = Ladder::new(inward(), Band { z_lo: -1.0, z_hi: 4.2, clamp: false });
    for k in 0..=3640 {
        let z = Z_MIN + 0.3 + k as f64 * 0.01;
        let host = holon_draw::ladder::host_weight(&l.band, z);
        let sum: f64 = visible(&l.rungs, z).iter().map(|x| x.1).sum();
        assert!((host + sum - 1.0).abs() < 1e-9, "z = {z}: host {host} + rungs {sum}");
    }
}

#[test]
fn every_rung_says_what_it_is_and_where_its_numbers_come_from() {
    let scenes = inward();
    assert_eq!(scenes.len(), 11);
    for s in &scenes {
        assert!(!s.name().is_empty());
        assert!(s.source().len() > 80, "{}: a source line that says something", s.name());
        assert!(!s.source().contains('\n'));
        // the unit is within the rung's own span of scales
        let (lo, hi) = s.span();
        let (lo, hi) = (lo + VIEW_DECADES, hi + VIEW_DECADES);
        assert!((s.unit() as f64) >= lo - 2.0 && (s.unit() as f64) <= hi + 2.0, "{}", s.name());
    }
    let status = |name: &str| scenes.iter().find(|s| s.name() == name).unwrap().status();
    // only the live simulation says simulated; only the declared input says declared
    assert_eq!(status("hydrogen, atom by atom"), Status::Simulated);
    assert_eq!(status("inside the atom: the nucleus"), Status::Declared);
    for n in ["the ground underfoot", "a drop of water", "liquid water, molecule by molecule", "inside the proton", "the vacuum"] {
        assert_eq!(status(n), Status::Illustration, "{n}");
    }
    assert_eq!(scenes.iter().filter(|s| s.status() == Status::Simulated).count(), 1);
    // the honest words are in the lines
    let source = |name: &str| scenes.iter().find(|s| s.name() == name).unwrap().source();
    assert!(source("liquid water, molecule by molecule").contains("NOT read from the measured trajectories"));
    assert!(source("hydrogen, atom by atom").contains("STO-3G") && source("hydrogen, atom by atom").contains("computed ahead of time") && source("hydrogen, atom by atom").contains("No three-body table"));
    assert!(source("inside the proton").contains("one space dimension only"));
    assert!(source("the vacuum").contains("Nothing in this repository computes"));
}

#[test]
fn every_mark_is_finite_and_of_a_size_its_unit_can_hold_at_every_zoom() {
    for mut s in inward() {
        let (lo, hi) = s.span();
        let mut n = 0;
        for k in 0..=24 {
            // through the span and the fade beyond each edge
            let z = lo - 0.25 + (hi - lo + 0.5) * k as f64 / 24.0;
            for clock in [0.0, 12.34, NOW] {
                for t in s.tiles(z, clock, 1.0) {
                    assert_eq!(t.version, 1);
                    assert!(!t.screen);
                    assert!(t.params.iter().all(|v| v.is_finite()));
                    for m in &t.marks {
                        let all = m.pos.iter().chain(&m.size).chain(&m.aux);
                        assert!(all.clone().all(|v| v.is_finite()), "{} at z = {z}", s.name());
                        assert!(all.clone().all(|v| v.abs() < 1.0e6), "{} at z = {z}: {m:?}", s.name());
                        assert!(m.size[0] > 1.0e-6, "{} at z = {z}: {m:?}", s.name());
                        n += 1;
                    }
                }
            }
        }
        assert!(n > 0, "{} draws something", s.name());
    }
}

#[test]
fn a_scene_is_the_same_for_the_same_clock_and_moves_with_it() {
    for (mut a, mut b) in inward().into_iter().zip(inward()) {
        let z = 0.5 * (a.span().0 + a.span().1);
        assert_eq!(a.tiles(z, NOW, 1.0), b.tiles(z, NOW, 1.0), "{}", a.name());
    }
    for name in ["liquid water, molecule by molecule", "inside the proton", "the vacuum"] {
        let mut s = inward().into_iter().find(|s| s.name() == name).unwrap();
        let z = 0.5 * (s.span().0 + s.span().1);
        assert_ne!(s.tiles(z, NOW, 1.0), s.tiles(z, NOW + 0.5, 1.0), "{name} moves");
    }
}

#[test]
fn the_ground_and_the_drop_are_at_true_size_and_show_one_drop() {
    // the ground is in centimetres, the drop in micrometres: a 2 mm drop whose top is the origin
    let mut g = Ground::new();
    let ball = |m: &[Instance]| m.iter().filter(|m| m.kind == Kind::Sphere && m.pos[0] == 0.0 && m.pos[1] == 0.0).map(|m| (m.size[0], m.pos[2])).next().unwrap();
    let (r, zc) = ball(&marks(&mut g, -1.0, 0.0));
    assert!((r - 0.1).abs() < 1e-6 && (zc + r).abs() < 1e-6, "0.1 cm, its top at the origin");
    let mut d = Drop::new();
    let all = marks(&mut d, -4.0, 0.0);
    let (r, zc) = ball(&all);
    assert!((r - 1000.0).abs() < 1e-3 && (zc + r).abs() < 1e-3, "1000 µm, its top at the origin");
    // close to its top the surface is one disc on the tangent plane, out to the drop's horizon
    assert!(Drop::top(-3.0).is_none(), "a millimetre view shows the sphere");
    let (rho, a) = Drop::top(-6.0).unwrap();
    assert!((rho as f64 - 1000f64.sqrt()).abs() < 1e-4 && a == 1.0);
    assert!(marks(&mut d, -6.0, 0.0).iter().any(|m| m.kind == Kind::Disc && m.pos == [0.0; 3] && m.size[0] == rho));
    // the same grit, in both units
    let cm = marks(&mut g, -1.0, 0.0).iter().filter(|m| m.kind == Kind::Disc).count();
    let um = all.iter().filter(|m| m.kind == Kind::Disc && m.pos[2] < -1500.0).count();
    assert_eq!(cm, um);
    // far out the blades and grit are not built
    assert!(marks(&mut g, 1.9, 0.0).len() < 600);
    // a slab is 40 cm and a blade under 11 cm tall
    assert!(marks(&mut g, 0.5, 0.0).iter().any(|m| m.kind == Kind::Box && (m.size[0] - (ground::SLAB_PITCH - ground::JOINT)).abs() < 1e-4));
    assert!(marks(&mut g, 0.5, 0.0).iter().filter(|m| m.kind == Kind::Line).all(|m| (m.aux[2] - m.pos[2]).abs() < 11.0));
}

#[test]
fn water_is_rigid_molecules_at_the_liquids_density() {
    let mut w = Water::new();
    // every molecule, moving or still, at any clock: O-H 0.9572 Å and 104.52 degrees
    for i in (0..w.molecules()).step_by(7) {
        for clock in [0.0, 3.3, NOW] {
            let (a, b, angle) = water::geometry(&w, i, clock);
            assert!((a - water::OH).abs() < 2e-5 && (b - water::OH).abs() < 2e-5, "{i}: {a} {b}");
            assert!((angle - water::HOH_DEG).abs() < 0.02, "{i}: {angle}");
        }
    }
    // 33.4 molecules per nm^3: the lattice the slab is thrown from, and the slab as built
    assert!((1.0 / Water::spacing().powi(3) - water::DENSITY).abs() < 1e-12);
    // counted in a block inside the slab, 50 by 50 Å and two layers (6.2 Å) deep
    let inside = (0..w.molecules()).map(|i| water::oxygen(&w, i)).filter(|o| o[0].abs() < 25.0 && o[1].abs() < 25.0 && o[2] > -8.7 && o[2] <= -2.5).count();
    let density = inside as f64 / (50.0 * 50.0 * 6.2);
    assert!((density / water::DENSITY - 1.0).abs() < 0.05, "{density} per cubic Å");
    // no two oxygens closer than 2.3 Å among the nearest 400 (the liquid's first peak is at 2.8)
    for i in 0..400 {
        for j in 0..i {
            let (p, q) = (water::oxygen(&w, i), water::oxygen(&w, j));
            let d = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
            assert!(d > 2.3, "{i} {j}: {d}");
        }
    }
    // the hydrogen the zoom goes into is at the origin, above its oxygen
    let all = marks(&mut w, -8.8, NOW);
    assert!(all.iter().any(|m| m.kind == Kind::Sphere && m.size[0] == water::R_H && m.pos == [0.0, 0.0, 0.0]));
    let stick = |m: &Instance| m.kind == Kind::Rod && m.size[0] < 0.2;
    assert!(all.iter().any(stick), "sticks when close");
    assert!(!marks(&mut w, -7.6, NOW).iter().any(stick), "none when far");
    // nothing above the surface but that hydrogen's neighbours' atoms: the vapour is empty
    assert!(all.iter().filter(|m| m.kind == Kind::Sphere).all(|m| m.pos[2] < 2.5));
    // close in, only the near molecules are drawn; further out, all of them
    assert_eq!(marks(&mut w, -8.95, NOW).iter().filter(|m| m.kind == Kind::Sphere).count(), 900);
    assert_eq!(marks(&mut w, -8.0, NOW).iter().filter(|m| m.kind == Kind::Sphere && m.size[0] == water::R_O).count(), w.molecules());
}

#[test]
fn the_atoms_rung_loads_the_engines_own_curve_without_solving_and_steps() {
    let (r, e, f, e2, r_e, d_e, e_inf) = Atoms::curve().expect("the built-in curve parses");
    assert_eq!((r.len(), e.len(), f.len(), e2.len()), (492, 492, 492, 492));
    assert_eq!((r[0], r[491]), (0.3, 10.0));
    assert_eq!((r_e, d_e, e_inf), (1.3886940180177756, 0.20414235210759035, -0.933163699114551));
    assert!(r.windows(2).all(|w| w[1] > w[0]));
    assert!(Atoms::provenance().contains("STO-3G") && Atoms::provenance().contains("492 knots") && Atoms::provenance().contains("commit"));
    let t = Instant::now();
    let mut a = Atoms::new();
    let built = t.elapsed();
    assert!(a.world.ready(), "the curve was accepted: status {}", a.world.status);
    assert_eq!(a.world.sim.n, 2);
    if !cfg!(debug_assertions) {
        assert!(built.as_secs_f64() < 0.05, "built in {built:?}");
    }
    println!("atoms rung built in {built:?}");
    // stretched 15% beyond R_e at the start
    let start = a.bond();
    assert!((start - 1.15 * 1.3886940180177756 * atoms::ANGSTROM_PER_BOHR).abs() < 1e-9, "{start}");
    // off screen it does not step
    a.advance(0.016);
    assert_eq!(a.substeps, 0);
    // on screen it does, the bond stays bound and oscillates about R_e (0.7349 Å)
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    let t = Instant::now();
    for _ in 0..600 {
        let m = marks(&mut a, -9.6, 0.0);
        assert_eq!(m.iter().filter(|m| m.kind == Kind::Sphere).count(), 2);
        // the first atom is the origin; the other is a bond away, in Å
        let s: Vec<&Instance> = m.iter().filter(|m| m.kind == Kind::Sphere).collect();
        assert_eq!(s[0].pos, [0.0, 0.0, 0.0]);
        let d = (s[1].pos[0].powi(2) + s[1].pos[1].powi(2) + s[1].pos[2].powi(2)).sqrt() as f64;
        assert!((d - a.bond()).abs() < 1e-5);
        assert!((s[0].size[0] as f64 - 0.694_347 * atoms::ANGSTROM_PER_BOHR).abs() < 1e-6, "the engine's radius for hydrogen, in Å");
        a.advance(1.0 / 60.0);
        lo = lo.min(a.bond());
        hi = hi.max(a.bond());
    }
    let per_frame = t.elapsed() / 600;
    println!("atoms rung: {} substeps in 600 frames, {per_frame:?} a frame, bond {lo:.4} to {hi:.4} Å", a.substeps);
    assert!(a.substeps > 0);
    assert!(lo > 0.55 && lo < 0.7349 && hi > 0.7349 && hi < 0.95, "bond from {lo} to {hi}");
    // the box is the engine's 24 bohr, drawn in Å
    let m = marks(&mut a, -9.6, 0.0);
    let edge = m.iter().filter(|m| m.kind == Kind::Line).map(|m| ((m.aux[0] - m.pos[0]).powi(2) + (m.aux[1] - m.pos[1]).powi(2) + (m.aux[2] - m.pos[2]).powi(2)).sqrt()).fold(0.0f32, f32::max);
    assert!((edge as f64 - 24.0 * atoms::ANGSTROM_PER_BOHR).abs() < 1e-4, "{edge}");
}

#[test]
fn the_nucleus_is_a_proton_at_its_radius_in_a_cloud_of_the_right_size() {
    let mut n = Nucleus::new();
    // 1.5 Bohr radii is the 1s state's mean distance; 6000 dots give it to a few percent
    assert!((n.mean_distance() / (1.5 * nucleus::BOHR_FM) - 1.0).abs() < 0.03, "{}", n.mean_distance());
    for z in [-10.3, -12.0, -14.4] {
        let m = marks(&mut n, z, 0.0);
        let p: Vec<&Instance> = m.iter().filter(|m| m.kind == Kind::Sphere).collect();
        assert_eq!(p.len(), 1, "hydrogen: one proton");
        assert_eq!((p[0].pos, p[0].size[0]), ([0.0; 3], 0.8414), "0.84 fm, in fm");
    }
    // the cloud is there at the atom's scale and gone at the nucleus's; the ring the other way
    let dots = |n: &mut Nucleus, z: f64| marks(n, z, 0.0).iter().filter(|m| m.kind == Kind::Box).count();
    assert!(dots(&mut n, -10.3) > 5900);
    assert!(dots(&mut n, -12.0) > 20 && dots(&mut n, -12.0) < 1500);
    assert_eq!(dots(&mut n, -14.0), 0);
    let ring = |n: &mut Nucleus, z: f64| marks(n, z, 0.0).iter().filter(|m| m.kind == Kind::Line).count();
    assert!(ring(&mut n, -12.0) > 0);
    assert_eq!(ring(&mut n, -14.2), 0);
}

#[test]
fn the_proton_is_three_quarks_inside_its_radius_and_the_tubes_meet() {
    assert!((hadron::PROTON_RADIUS - 8.414).abs() < 1e-5, "0.84 fm in tenths of a fm");
    let mut h = Hadron::new();
    for clock in [0.0, 7.7, NOW, NOW + 100.3] {
        let q = Hadron::quarks(clock);
        for p in q {
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!(r < hadron::PROTON_RADIUS - 0.8, "{r}");
        }
        // the junction is where the summed length is least: no nearby point does better
        let j = Hadron::junction(&q);
        let total = |x: [f32; 3]| q.iter().map(|p| ((p[0] - x[0]).powi(2) + (p[1] - x[1]).powi(2) + (p[2] - x[2]).powi(2)).sqrt()).sum::<f32>();
        for d in [[0.2, 0.0, 0.0], [-0.2, 0.0, 0.0], [0.0, 0.2, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.2], [0.0, 0.0, -0.2]] {
            assert!(total(j) <= total([j[0] + d[0], j[1] + d[1], j[2] + d[2]]) + 1e-4);
        }
        let m = marks(&mut h, -15.2, clock);
        assert_eq!(m.iter().filter(|m| m.kind == Kind::Rod).count(), 3);
        assert_eq!(m.iter().filter(|m| m.kind == Kind::Sphere).count(), 5, "three quarks, the junction, the shell");
        assert!(m.iter().any(|m| m.kind == Kind::Sphere && m.size[0] == hadron::PROTON_RADIUS && m.rgba[3] < 80));
    }
}

#[test]
fn the_vacuum_is_one_picture_over_five_rungs_and_stops_at_the_planck_length() {
    let rungs = Vacuum::rungs();
    assert_eq!(rungs.len(), 5);
    // the Planck length in the last rung's unit (1e-33 m)
    assert!((rungs[4].planck() - 0.016_162_55).abs() < 1e-9);
    // the weights of the meshes change without a jump as the zoom passes a half-decade
    for k in 0..3000 {
        let z = -30.0 + k as f64 * 0.004;
        let (a, b) = (Vacuum::meshes(z), Vacuum::meshes(z + 0.004));
        for (ka, wa) in &a {
            let wb = b.iter().find(|x| x.0 == *ka).map_or(0.0, |x| x.1);
            assert!((wa - wb).abs() < 0.02, "z = {z}: mesh {ka} from {wa} to {wb}");
        }
    }
    // no mesh finer than the Planck length; at the very bottom only the bar is left
    for k in 0..400 {
        let z = -35.25 + k as f64 * 0.01;
        assert!(Vacuum::meshes(z).iter().all(|(k, _)| 10f64.powf(*k as f64 / 2.0) >= vacuum::PLANCK_M));
    }
    let mut last = Vacuum::rungs().pop().unwrap();
    let bar = |m: &[Instance]| m.iter().filter(|m| m.kind == Kind::Line && m.rgba == [250, 250, 246, 255]).map(|m| (m.aux[0] - m.pos[0]) as f64).fold(0.0, f64::max);
    assert!((bar(&marks(&mut last, -34.0, NOW)) - last.planck()).abs() < 1e-8, "the bar is one Planck length");
    assert_eq!(bar(&marks(&mut last, -31.5, NOW)), 0.0, "and is not drawn where it would be a speck");
    // where two rungs meet they draw the same picture, each in its own unit
    let (mut a, mut b) = { let mut r = Vacuum::rungs(); let b = r.remove(2); let a = r.remove(1); (a, b) };
    let (ma, mb) = (marks(&mut a, -23.0, NOW), marks(&mut b, -23.0, NOW));
    assert_eq!(ma.len(), mb.len());
    let scale = 10f32.powi(a.unit() - b.unit());
    for (x, y) in ma.iter().zip(&mb) {
        assert_eq!((x.kind, x.rgba), (y.kind, y.rgba));
        for c in 0..3 {
            assert!((x.pos[c] * scale - y.pos[c]).abs() <= 2e-4 * y.pos[c].abs().max(1e-3), "{x:?} {y:?}");
        }
    }
}

#[test]
fn building_every_rung_and_drawing_a_frame_of_each_is_fast() {
    let t = Instant::now();
    let mut scenes = inward();
    let built = t.elapsed();
    println!("all eleven inward rungs built in {built:?}");
    let mut worst = (0.0f64, "");
    for s in scenes.iter_mut() {
        let (lo, hi) = s.span();
        for z in [lo + 0.1, 0.5 * (lo + hi), hi - 0.1] {
            s.tiles(z, NOW, 1.0);
            let t = Instant::now();
            let mut n = 0;
            for k in 0..20 {
                n += s.tiles(z, NOW + k as f64 * 0.016, 1.0).iter().map(|t| t.marks.len()).sum::<usize>();
                s.advance(0.016);
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0 / 20.0;
            println!("{:36} z = {z:6.1}: {:6} marks, {ms:.3} ms a frame", s.name(), n / 20);
            if ms > worst.0 {
                worst = (ms, s.name());
            }
        }
    }
    if !cfg!(debug_assertions) {
        assert!(built.as_secs_f64() < 0.05, "built in {built:?}");
        assert!(worst.0 < 5.0, "{} took {} ms a frame", worst.1, worst.0);
    }
}

#[test]
fn what_a_rung_draws_at_a_zoom_does_not_depend_on_the_windows_size() {
    // z is metres per device pixel: a phone and a large screen at one z are handed the same
    // marks (a scene is never told the window's size), and only the camera differs
    let band = Band { z_lo: -1.0, z_hi: 4.2, clamp: false };
    let (mut small, mut large) = (Ladder::new(inward(), band), Ladder::new(inward(), band));
    for k in 0..=74 {
        let z = -37.9 + k as f64 * 0.5;
        let a = small.passes(z, 0.8, 0.42, 390.0, 844.0, 3.0, NOW);
        let b = large.passes(z, 0.8, 0.42, 2560.0, 1440.0, 1.0, NOW);
        assert_eq!(a.len(), b.len(), "z = {z}");
        for (p, q) in a.iter().zip(&b) {
            assert_eq!((p.rung, p.weight), (q.rung, q.weight));
            let world = |f: &holon_draw::Frame| f.batches.iter().filter(|b| !b.screen).map(|b| (b.tile, b.kind, b.coarse, b.bytes.clone(), b.params)).collect::<Vec<_>>();
            assert!(world(&p.frame) == world(&q.frame), "z = {z}, rung {}", small.rungs[p.rung].name);
        }
    }
}

#[test]
fn the_built_in_curve_is_the_curve_the_engine_solves() {
    // the solve (over a second in release; it is why the rung does not do it) against the file
    let mut solved = holon_draw_atoms::World::new();
    assert_eq!(solved.load(0), 1);
    let mut loaded = Atoms::new();
    let read = |w: &mut holon_draw_atoms::World| {
        let t = w.sim.table_mut();
        (t.knots(), (0..=2000).map(|k| t.eval(0.3 + 9.7 * k as f64 / 2000.0)).collect::<Vec<_>>())
    };
    // energy, slope and curvature at 2001 separations: the same floats
    assert!(read(&mut solved) == read(&mut loaded.world));
    // and the clocks that are functions of the curve alone (dt itself is refined by each
    // scene's own energy envelope, and the two scenes differ)
    let (s, l) = (&solved.sim.timescale, &loaded.world.sim.timescale);
    assert_eq!((s.dt_reference, s.period, s.omega_e), (l.dt_reference, l.period, l.omega_e));
}

/// Triangles the backend draws for a frame's world batches (`holon-draw/web/draw.js`: a
/// sphere is 28 by 14, or 10 by 6 when its batch is coarse; a rod 20 sides, or 8; a box 12,
/// a disc 24, a line 12), and the same if every batch were drawn fine.
fn triangles(f: &holon_draw::Frame) -> (u64, u64) {
    let (mut drawn, mut fine) = (0u64, 0u64);
    for b in f.batches.iter().filter(|b| !b.screen) {
        let (full, cheap) = match b.kind {
            Kind::Sphere => (728, 100),
            Kind::Rod => (80, 32),
            Kind::Box | Kind::Prism | Kind::Line | Kind::Arc => (12, 12),
            Kind::Disc | Kind::Glyph => (24, 24),
            _ => (2, 2),
        };
        drawn += b.count as u64 * if b.coarse { cheap } else { full };
        fine += b.count as u64 * full;
    }
    (drawn, fine)
}

#[test]
fn water_far_from_the_eye_is_drawn_on_the_cheap_mesh_and_near_it_on_the_full_one() {
    let band = Band { z_lo: -1.0, z_hi: 4.2, clamp: false };
    let mut l = Ladder::new(inward(), band);
    let water = l.rungs.iter().position(|r| r.name == "liquid water, molecule by molecule").unwrap();
    let mut seen = Vec::new();
    for k in 0..=30 {
        let z = -12.0 + 0.05 * k as f64;
        let passes = l.passes(z, 0.8, 0.42, 1100.0, 720.0, 1.0, NOW);
        let p = passes.iter().find(|p| p.rung == water).unwrap();
        let (drawn, fine) = triangles(&p.frame);
        println!("water z = {z:6.2}: {:2} tiles fine, {:2} coarse, {drawn:8} triangles ({fine:8} if all were fine), remainder {:.2} px", p.frame.fine, p.frame.coarse, p.frame.remainder_px);
        // what is left coarse is within the pixel the allocator was given
        assert!(p.frame.remainder_px <= 1.0);
        assert!(drawn <= fine);
        // nothing is dropped but the probes: every ball is in the frame
        let balls: u32 = p.frame.batches.iter().filter(|b| b.kind == Kind::Sphere).map(|b| b.count).sum();
        assert!(balls == 900 || balls == 3 * 1083, "{balls}");
        seen.push((z, drawn, fine, p.frame.fine, p.frame.coarse));
    }
    // closest in, everything drawn is near and on the full mesh
    assert_eq!(seen[0].4, 0);
    // by the time the whole patch is in view every molecule tile is coarse (the one fine
    // tile is the floor, which has no detail) and the frame is a seventh of the triangles
    let wide = seen.last().unwrap();
    assert_eq!(wide.3, 1);
    assert!(wide.1 * 6 < wide.2, "{wide:?}");
    // never more than 0.9 million triangles
    assert!(seen.iter().all(|s| s.1 < 900_000), "{seen:?}");
    // on a phone the choices are the same: the camera's distance is set by z alone
    let mut phone = Ladder::new(inward(), band);
    for (z, _, _, fine, coarse) in &seen {
        let passes = phone.passes(*z, 0.8, 0.42, 390.0, 844.0, 3.0, NOW);
        let p = passes.iter().find(|p| p.rung == water).unwrap();
        assert_eq!((p.frame.fine, p.frame.coarse), (*fine, *coarse), "z = {z}");
    }
}
