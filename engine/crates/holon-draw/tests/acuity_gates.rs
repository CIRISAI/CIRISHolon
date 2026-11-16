//! Falsifier gates for the acuity allocator (the browser half is `tools/acuity_probe.py`;
//! what each gate shows and does not is in `results/ACUITY_PROBE.md`).
//!
//! The allocator's per-tile bound is the CERTIFICATE: an a-priori upper bound. A gate here
//! compares a coarse frame with a fine one. That can kill the bound; it cannot prove it.
//! A frame is read three ways at once (the pixels, the pick, the report), and a coarse draw
//! has to stay inside all three.
mod gates;

use gates::*;
use holon_draw::{coarse_shortfall, Acuity, Camera, Instance, Kind};
use std::f64::consts::PI;

#[test]
fn the_forced_allocators_force() {
    // two tiles with something to coarsen (a ball; a detailed tree) and one with nothing
    let tiles = vec![
        tile(1, vec![Instance::new(Kind::Sphere, [0.0; 3], [5.0, 0.0, 0.0]).tagged(1)]),
        tile(2, vec![Instance::new(Kind::Disc, [30.0, 0.0, 0.0], [4.0, 0.0, 0.0]).detail(1)]),
        tile(3, vec![Instance::new(Kind::Box, [60.0, 0.0, 0.0], [4.0, 4.0, 4.0]).tagged(3)]),
    ];
    let cam = Camera::map([0.0; 3], 1.0, 400.0, 300.0);
    let (f, c) = (frame(&tiles, &cam, 1, &AllFine), frame(&tiles, &cam, 1, &AllCoarse));
    assert_eq!((f.fine, f.coarse, f.remainder_px), (3, 0, 0.0));
    assert_eq!((c.fine, c.coarse), (1, 2), "a tile with nothing to coarsen is fine");
    assert_eq!(coarse_tiles(&c).into_iter().collect::<Vec<_>>(), vec![1], "the ball's batch is flagged; the tree is simply not drawn");
    // the frame's certificate is the largest of the tiles' own, each read from the core
    let own: Vec<f64> = tiles.iter().map(|t| tile_bound(t, &cam)).collect();
    assert_eq!(own[2], 0.0);
    assert_eq!(c.remainder_px, own[0].max(own[1]));
}

#[test]
fn the_certificate_covers_a_planted_shortfall_of_known_size() {
    // A PLANT OF KNOWN SIZE, on the shortfall branch: a ball 100 pixels in radius, looked at
    // down its pole, where the cheap mesh's outline is a decagon. Its edges' midpoints fall
    // short of the true circle by k = r (1 - cos(pi/10)) = 4.894 px.
    let ball = tile(1, vec![Instance::new(Kind::Sphere, [0.0; 3], [100.0, 0.0, 0.0])]);
    let cam = Camera::map([0.0; 3], 1.0, 400.0, 300.0);
    let k = 100.0 * (1.0 - (PI / 10.0).cos());
    let cert = tile_bound(&ball, &cam);
    assert!((cert - 2.0 * k).abs() < 1e-9, "the core prices the shortfall across the DIAMETER: {cert}");
    assert!(cert >= k, "the certificate {cert} covers the planted {k}");
    // under the real allocator at a pixel of budget the ball is drawn fine and nothing is left
    let f = frame(std::slice::from_ref(&ball), &cam, 1, &Acuity { eps_px: 1.0 });
    assert_eq!((f.coarse, f.remainder_px), (0, 0.0));
    // a ball of 10 px: 0.979 px, inside the budget, left coarse, and the frame says so
    let small = tile(1, vec![Instance::new(Kind::Sphere, [0.0; 3], [10.0, 0.0, 0.0])]);
    let f = frame(std::slice::from_ref(&small), &cam, 1, &Acuity { eps_px: 1.0 });
    assert!(f.coarse == 1 && (f.remainder_px - 0.2 * k).abs() < 1e-9 && f.remainder_px >= 0.1 * k);

    // A PLANT OF KNOWN SIZE, on the dropped-decoration branch: a detailed, untagged disc 20
    // pixels in radius is not drawn when its tile is coarse; the wrong region is 40 px wide.
    let tree = tile(2, vec![Instance::new(Kind::Disc, [0.0; 3], [20.0, 0.0, 0.0]).detail(1)]);
    assert_eq!(tile_bound(&tree, &cam), 40.0);
    assert_eq!(frame(std::slice::from_ref(&tree), &cam, 1, &AllCoarse).drawn(), 0);
}

/// `web/draw.js`'s `sphereMesh(segs, stacks)`, vertices and triangles, as that file builds it.
fn sphere_mesh(segs: usize, stacks: usize) -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
    let (mut v, mut ix) = (Vec::new(), Vec::new());
    for i in 0..=stacks {
        let t = PI * i as f64 / stacks as f64;
        for j in 0..=segs {
            let f = 2.0 * PI * j as f64 / segs as f64;
            v.push([t.sin() * f.cos(), t.sin() * f.sin(), t.cos()]);
        }
    }
    let w = segs + 1;
    for i in 0..stacks {
        for j in 0..segs {
            let (a, b) = (i * w + j, i * w + j + w);
            if i > 0 {
                ix.push([a, b + 1, a + 1]);
            }
            if i < stacks - 1 {
                ix.push([a, b, b + 1]);
            }
        }
    }
    (v, ix)
}

/// The least distance from the centre to a face of the mesh: seen edge-on to that face, the
/// outline of a unit ball drawn on this mesh comes in to exactly this (the outline of a convex
/// body in a direction is its support function there, and the least of a polyhedron's support
/// function is its nearest face).
fn inradius(v: &[[f64; 3]], ix: &[[usize; 3]]) -> f64 {
    let sub = |a: [f64; 3], b: [f64; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let mut least = f64::INFINITY;
    for t in ix {
        let (a, b, c) = (v[t[0]], v[t[1]], v[t[2]]);
        let (e, f) = (sub(b, a), sub(c, a));
        let n = [e[1] * f[2] - e[2] * f[1], e[2] * f[0] - e[0] * f[2], e[0] * f[1] - e[1] * f[0]];
        let l = dot(n, n).sqrt();
        if l < 1e-12 {
            continue;
        }
        let n = [n[0] / l, n[1] / l, n[2] / l];
        let d = dot(n, a);
        // the face's plane holds the whole mesh on one side: the mesh is convex there
        assert!(v.iter().all(|p| dot(n, *p) <= d + 1e-12), "not a supporting plane");
        least = least.min(d);
    }
    least
}

#[test]
fn the_cheap_meshes_worst_outline_is_inside_the_certificate_and_outside_the_named_fraction() {
    // `coarse_shortfall(Sphere)` is 1 - cos(pi/10) = 4.89 %: the cheap ball's outline seen
    // down its pole. That is not its worst outline. Seen edge-on to one of the faces beside
    // its equator the outline comes in by 1 - inradius = 7.84 % of the radius.
    let (v, ix) = sphere_mesh(10, 6);
    let worst = 1.0 - inradius(&v, &ix);
    let named = coarse_shortfall(Kind::Sphere);
    println!("cheap ball: worst outline shortfall {:.4} of the radius; named fraction {:.4}; certificate {:.4} of the radius", worst, named, 2.0 * named);
    assert!((worst - 0.0784).abs() < 5e-4, "{worst}");
    // THE CERTIFICATE HOLDS, because the core multiplies the fraction by the mark's DIAMETER
    // (`Camera::pixels` is a span across) while an outline moves by a share of the RADIUS:
    assert!(worst <= 2.0 * named);
    // and by a factor of 1.25 there, not 2. A reading of the fraction as "of the radius"
    // would be killed by this view:
    assert!(worst > named);
    assert!((2.0 * named / worst - 1.249).abs() < 5e-3);
    // The full mesh (28 by 14) is not the true ball either: 1.26 % at worst. A probe that
    // compares coarse against FINE therefore reads at most `worst`, and may read less.
    let (v, ix) = sphere_mesh(28, 14);
    let fine = 1.0 - inradius(&v, &ix);
    assert!(fine > 0.012 && fine < 0.013, "{fine}");
    // A rod: 8 sides against a circle, 1 - cos(pi/8) of its radius from every side-on view
    // and from its end; the certificate is twice that.
    assert!((coarse_shortfall(Kind::Rod) - (1.0 - (PI / 8.0).cos())).abs() < 1e-15);
}

#[test]
fn a_coarse_draw_that_moves_no_pixel_is_caught_by_the_pick_and_the_report() {
    let (tiles, cam) = zero_pixel_scene();
    let fine = frame(&tiles, &cam, 1, &AllFine);
    let coarse = frame(&tiles, &cam, 1, &AllCoarse);
    assert_eq!(coarse.coarse, 1);
    let points = pick_points(&cam, &[&fine, &coarse], 16);

    // CONTROL: the core's own coarse frame. Every tagged mark is still there, where it was.
    assert_eq!(views_disagree(&fine, &coarse, &points), Vec::<String>::new());
    assert!(coarse.report(2).unwrap().coarse && !fine.report(2).unwrap().coarse, "and its report says it is coarse");

    // PLANT 1: the coarse draw drops the tagged mark it should have flagged (tag 2, the box
    // hidden inside the larger one). THE GATE FIRES on both views.
    let dropped = plant_drop_coarse_tagged(&coarse);
    let d = views_disagree(&fine, &dropped, &points);
    println!("plant 1 (a hidden tagged mark dropped): {} disagreements, e.g. {:?}", d.len(), d.first());
    assert!(d.iter().any(|s| s.starts_with("report of tag 2")), "the report must fire: {d:?}");
    assert!(d.iter().any(|s| s.starts_with("pick at")), "the pick must fire: {d:?}");
    assert_eq!(fine.pick(110.0, 150.0), Some(2));
    assert_eq!(dropped.pick(110.0, 150.0), Some(1));
    // what it removed from the draw is one record, the hidden box; every other record is as
    // it was (that the removal moves no pixel is the browser probe's reading, not this one's)
    assert_eq!(dropped.drawn() + 1, coarse.drawn());

    // PLANT 2: the coarse draw hands two like marks each other's tag. THE GATE FIRES.
    let swapped = plant_swap_tags(&coarse, 3, 4);
    let d = views_disagree(&fine, &swapped, &points);
    println!("plant 2 (two tags exchanged): {} disagreements, e.g. {:?}", d.len(), d.first());
    assert!(d.iter().any(|s| s.starts_with("pick at")) && d.iter().any(|s| s.starts_with("report of tag 3")), "{d:?}");
    assert_eq!((fine.pick(240.0, 150.0), swapped.pick(240.0, 150.0)), (Some(3), Some(4)));
    // A PIXELS-ONLY PROBE MISSES IT, for certain and on any backend: the bytes a shader is
    // given (0..44 of each record, with each batch's kind, flag, offset and settings) are the
    // same in the planted frame as in the honest coarse one.
    assert!(drawn_bytes(&swapped) == drawn_bytes(&coarse));
    assert!(swapped.batches != coarse.batches, "though the records differ, in the tag");
}
