//! THE ICE SEEDS' GATE BATTERY: six gates on the three ordered polymorphs
//! `holon_render::lattice` builds, and a plant that proves two of them can fire.
//!
//! Every distance below is a MINIMUM-IMAGE distance in the box the builder returned.
//! That is not a stylistic choice: the builders wrap hydrogens across faces, so a gate
//! measuring raw coordinate differences would read a molecule at the far side of the box
//! as dissociated and pass a structure that had actually fallen apart.
//!
//! The nominal separations the geometry gate compares against are DERIVED HERE from the
//! lattice constants by crystallography, not imported from the builder. A gate that
//! re-read the number the builder used would be checking that a constant equals itself.
//!
//! # Why the plant lives in this file
//!
//! The commissioning note asked for a `#[cfg(test)]` mutated builder. An integration
//! test is a separate crate and cannot see a `#[cfg(test)]` item inside `holon-render`,
//! so a plant put there would be unreachable from these gates — which is the exact
//! failure the plant exists to rule out. It is therefore defined below, where it is
//! test-only by construction, as a mutation OF the shipped builder's output rather than
//! a re-implementation of it: a hand-written wrong lattice would prove that the gates
//! reject a lattice nobody ships.

use holon_render::lattice::{
    ice_viii, ice_x, ice_xi, A_ICE_VIII_BOHR, A_ICE_XI_BOHR, A_ICE_X_BOHR, C_ICE_XI_BOHR,
    R_OH_BOHR,
};
use holon_render::sim::{Boundary, Dims, Sim};

type Scene = (Vec<(u8, f64, f64, f64)>, (f64, f64, f64));

const CELLS: [(usize, usize, usize); 2] = [(2, 2, 2), (3, 3, 3)];

// ------------------------------------------------------------------ shared measuring

fn mimg(p: (f64, f64, f64), q: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    let mut d = (q.0 - p.0, q.1 - p.1, q.2 - p.2);
    d.0 -= b.0 * (d.0 / b.0).round();
    d.1 -= b.1 * (d.1 / b.1).round();
    d.2 -= b.2 * (d.2 / b.2).round();
    (d.0 * d.0 + d.1 * d.1 + d.2 * d.2).sqrt()
}

fn split(sc: &Scene) -> (Vec<(f64, f64, f64)>, Vec<(f64, f64, f64)>, usize) {
    let mut o = Vec::new();
    let mut h = Vec::new();
    let mut other = 0usize;
    for &(z, x, y, zz) in sc.0.iter() {
        match z {
            8 => o.push((x, y, zz)),
            1 => h.push((x, y, zz)),
            _ => other += 1,
        }
    }
    (o, h, other)
}

/// Sorted minimum-image distances from `p` to every point of `pts`, self-hit dropped.
fn shell(p: (f64, f64, f64), pts: &[(f64, f64, f64)], b: (f64, f64, f64)) -> Vec<f64> {
    let mut d: Vec<f64> = pts
        .iter()
        .map(|&q| mimg(p, q, b))
        .filter(|&x| x > 1e-9)
        .collect();
    d.sort_by(|a, c| a.partial_cmp(c).unwrap());
    d
}

/// The O–O nearest-neighbour separations each polymorph is built to have, derived from
/// the declared lattice constants.
///
/// XI has TWO of them and that is real, not slop: the `c`-axis bond is `3c/8` and the
/// three lateral bonds are `√(a²/3 + c²/64)`, unequal because the mineral's `c/a` is not
/// the ideal tetrahedral ratio. The gate takes the pair as an interval.
fn nn_xi(s: f64) -> (f64, f64) {
    let (a, c) = (A_ICE_XI_BOHR * s, C_ICE_XI_BOHR * s);
    (3.0 * c / 8.0, (a * a / 3.0 + c * c / 64.0).sqrt())
}
/// bcc with cube edge `a/2`: the eight neighbours sit at `(a/2)·√3/2 = a√3/4`.
fn nn_cubic(a: f64) -> f64 {
    a * 3.0f64.sqrt() / 4.0
}

// ------------------------------------------------------------------ G1 stoichiometry

fn g1(name: &str, sc: &Scene) {
    let (o, h, other) = split(sc);
    assert_eq!(other, 0, "{name}: a species that is neither H nor O is present");
    assert!(!o.is_empty(), "{name}: the scene has no oxygen");
    assert_eq!(
        h.len(),
        2 * o.len(),
        "{name}: stoichiometry is not H2O — {} H against {} O",
        h.len(),
        o.len()
    );
}

#[test]
fn g1_stoichiometry_is_water() {
    for &c in CELLS.iter() {
        g1(&format!("XI {c:?}"), &ice_xi(c, 1.0));
        g1(&format!("VIII {c:?}"), &ice_viii(c, 1.0));
        g1(&format!("X {c:?}"), &ice_x(c, 1.0));
    }
    // The cell count multiplies the 8- and 16-oxygen cells exactly, which is the
    // periodic-tiling claim in counting form.
    assert_eq!(split(&ice_xi((2, 2, 2), 1.0)).0.len(), 8 * 8);
    assert_eq!(split(&ice_xi((3, 3, 3), 1.0)).0.len(), 8 * 27);
    assert_eq!(split(&ice_viii((2, 2, 2), 1.0)).0.len(), 16 * 8);
    assert_eq!(split(&ice_x((3, 3, 3), 1.0)).0.len(), 16 * 27);
}

// ------------------------------------------------------------------ G2 geometry

/// Every oxygen's four nearest oxygens land inside `[lo, hi]` widened by 1%.
/// Returns the measured window so the report can carry the numbers.
fn oo_window(name: &str, sc: &Scene, k: usize, lo: f64, hi: f64) -> (f64, f64) {
    let (o, _, _) = split(sc);
    let b = sc.1;
    let (mut mn, mut mx) = (f64::INFINITY, 0.0f64);
    for &p in o.iter() {
        for &d in shell(p, &o, b).iter().take(k) {
            mn = mn.min(d);
            mx = mx.max(d);
        }
    }
    assert!(
        mn >= 0.99 * lo && mx <= 1.01 * hi,
        "{name}: O-O first shell [{mn:.6}, {mx:.6}] bohr is outside 1% of the built \
         separations [{lo:.6}, {hi:.6}]"
    );
    println!("{name}: O-O first shell = [{mn:.6}, {mx:.6}] bohr (nominal [{lo:.6}, {hi:.6}])");
    (mn, mx)
}

/// The two nearest hydrogens to each oxygen, which for XI and VIII are its covalent
/// pair. Taking "the two nearest" rather than "everything inside 2.2 bohr" is what lets
/// this gate see a hydrogen that has been pushed OUT of the covalent shell — a
/// window-based gate would simply stop counting it and report nothing wrong.
fn covalent_err(sc: &Scene) -> f64 {
    let (o, h, _) = split(sc);
    let b = sc.1;
    let mut worst = 0.0f64;
    for &p in o.iter() {
        for &d in shell(p, &h, b).iter().take(2) {
            worst = worst.max((d - R_OH_BOHR).abs());
        }
    }
    worst
}

#[test]
fn g2_geometry_matches_the_declared_lattice() {
    for &c in CELLS.iter() {
        let (lo, hi) = nn_xi(1.0);
        let sc = ice_xi(c, 1.0);
        oo_window(&format!("XI {c:?}"), &sc, 4, lo.min(hi), lo.max(hi));
        let e = covalent_err(&sc);
        assert!(e < 1e-6, "XI {c:?}: covalent O-H off 1.81 bohr by {e:.3e}");
        println!("XI {c:?}: max |O-H - 1.81| = {e:.3e} bohr");

        let n = nn_cubic(A_ICE_VIII_BOHR);
        let sc = ice_viii(c, 1.0);
        // bcc: EIGHT neighbours at the nearest-neighbour distance, four bonded and four
        // not. Checking only four would leave the other-network contact unmeasured.
        oo_window(&format!("VIII {c:?}"), &sc, 8, n, n);
        let e = covalent_err(&sc);
        assert!(e < 1e-6, "VIII {c:?}: covalent O-H off 1.81 bohr by {e:.3e}");
        println!("VIII {c:?}: max |O-H - 1.81| = {e:.3e} bohr");

        let n = nn_cubic(A_ICE_X_BOHR);
        let sc = ice_x(c, 1.0);
        oo_window(&format!("X {c:?}"), &sc, 8, n, n);
        let e = midpoint_err(&sc);
        assert!(e < 1e-6, "X {c:?}: an H is off its O-O midpoint by {e:.3e}");
        println!("X {c:?}: max midpoint error = {e:.3e} bohr (O-H = {:.6})", n / 2.0);
    }
}

/// For ice X: each hydrogen's two nearest oxygens must be equidistant, and at half the
/// nearest-neighbour separation. Both halves are needed — equidistant alone is satisfied
/// by a hydrogen sitting anywhere on the bisecting plane, including far off the axis.
fn midpoint_err(sc: &Scene) -> f64 {
    let (o, h, _) = split(sc);
    let b = sc.1;
    let half = nn_cubic(A_ICE_X_BOHR) / 2.0;
    let mut worst = 0.0f64;
    for &p in h.iter() {
        let d = shell(p, &o, b);
        worst = worst.max((d[0] - d[1]).abs()).max((d[0] - half).abs());
    }
    worst
}

#[test]
fn g2_scale_is_a_uniform_dilation() {
    // s is claimed to be a LINEAR scale on every length. Stated as a gate because a
    // builder that scaled the cell but not the bond would pass every gate above at
    // s = 1 and silently ship a stretched molecule at any other s.
    let s = 1.37;
    let sc = ice_xi((2, 2, 2), s);
    let (lo, hi) = nn_xi(s);
    oo_window("XI s=1.37", &sc, 4, lo.min(hi), lo.max(hi));
    let (o, h, _) = split(&sc);
    let mut worst = 0.0f64;
    for &p in o.iter() {
        for &d in shell(p, &h, sc.1).iter().take(2) {
            worst = worst.max((d - R_OH_BOHR * s).abs());
        }
    }
    assert!(worst < 1e-6, "XI s=1.37: O-H off 1.81*s by {worst:.3e}");
    let sc = ice_x((2, 2, 2), s);
    oo_window("X s=1.37", &sc, 8, nn_cubic(A_ICE_X_BOHR * s), nn_cubic(A_ICE_X_BOHR * s));
}

// ------------------------------------------------------------------ G3 the ice rules

/// `(covalent count, acceptor count)` per oxygen, by the Bernal–Fowler windows.
fn bf_counts(sc: &Scene, s: f64) -> Vec<(usize, usize)> {
    let (o, h, _) = split(sc);
    let b = sc.1;
    o.iter()
        .map(|&p| {
            let (mut cov, mut acc) = (0usize, 0usize);
            for &q in h.iter() {
                let d = mimg(p, q, b);
                if d < 2.2 * s {
                    cov += 1;
                } else if d < 4.0 * s {
                    acc += 1;
                }
            }
            (cov, acc)
        })
        .collect()
}

fn assert_ice_rules(name: &str, sc: &Scene, s: f64) {
    let c = bf_counts(sc, s);
    let bad: Vec<_> = c.iter().enumerate().filter(|(_, &x)| x != (2, 2)).collect();
    assert!(
        bad.is_empty(),
        "{name}: {} of {} oxygens break the ice rules (want 2 donated, 2 accepted); \
         first offender: site {} reads {:?}",
        bad.len(),
        c.len(),
        bad[0].0,
        bad[0].1
    );
    println!("{name}: all {} oxygens read (2 covalent, 2 accepted)", c.len());
}

#[test]
fn g3_bernal_fowler_receipt() {
    // XI and VIII only. Ice X has no covalent/H-bond distinction to count, which is the
    // point of the polymorph and not an omission: every one of its O-H contacts is the
    // same length, so both windows would answer the same question wrongly.
    for &c in CELLS.iter() {
        assert_ice_rules(&format!("XI {c:?}"), &ice_xi(c, 1.0), 1.0);
        assert_ice_rules(&format!("VIII {c:?}"), &ice_viii(c, 1.0), 1.0);
    }
}

#[test]
fn g3_the_orderings_are_the_ones_claimed() {
    // The summed O->H vector: XI is ferroelectric along c, VIII antiferroelectric and
    // exactly zero. Gated because "proton-ordered" is not one structure — an ordering
    // that satisfied the ice rules with the wrong polarisation would pass every count
    // above and be a different crystal.
    let pol = |sc: &Scene| {
        let (o, h, _) = split(sc);
        let b = sc.1;
        let mut p = (0.0f64, 0.0f64, 0.0f64);
        for &q in h.iter() {
            // the donor is the nearest oxygen
            let (mut best, mut bd) = ((0.0, 0.0, 0.0), f64::INFINITY);
            for &c in o.iter() {
                let d = mimg(c, q, b);
                if d < bd {
                    bd = d;
                    best = c;
                }
            }
            let mut v = (q.0 - best.0, q.1 - best.1, q.2 - best.2);
            v.0 -= b.0 * (v.0 / b.0).round();
            v.1 -= b.1 * (v.1 / b.1).round();
            v.2 -= b.2 * (v.2 / b.2).round();
            p = (p.0 + v.0, p.1 + v.1, p.2 + v.2);
        }
        p
    };
    let p = pol(&ice_xi((2, 2, 2), 1.0));
    println!("XI  sum(O->H) = ({:.6}, {:.6}, {:.6})", p.0, p.1, p.2);
    assert!(p.0.abs() < 1e-9 && p.1.abs() < 1e-9, "XI: polarisation is along c only");
    assert!(p.2 > 1.0, "XI is FERROELECTRIC along c; sum z = {:.6}", p.2);

    let p = pol(&ice_viii((2, 2, 2), 1.0));
    println!("VIII sum(O->H) = ({:.6}, {:.6}, {:.6})", p.0, p.1, p.2);
    assert!(
        p.0.abs() < 1e-9 && p.1.abs() < 1e-9 && p.2.abs() < 1e-9,
        "VIII is ANTIFERROELECTRIC: the summed dipole must be exactly zero, read {p:?}"
    );
}

// ------------------------------------------------------------------ G4 no overlaps

fn min_pair(sc: &Scene) -> f64 {
    let p: Vec<(f64, f64, f64)> = sc.0.iter().map(|&(_, x, y, z)| (x, y, z)).collect();
    let b = sc.1;
    let mut mn = f64::INFINITY;
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            let d = mimg(p[i], p[j], b);
            if d < mn {
                mn = d;
            }
        }
    }
    mn
}

#[test]
fn g4_nothing_opens_inside_the_repulsive_core() {
    for &c in CELLS.iter() {
        for (name, sc, floor) in [
            (format!("XI {c:?}"), ice_xi(c, 1.0), 1.3),
            (format!("VIII {c:?}"), ice_viii(c, 1.0), 1.3),
            (format!("X {c:?}"), ice_x(c, 1.0), 0.9),
        ] {
            let m = min_pair(&sc);
            println!("{name}: min interatomic (minimum image) = {m:.6} bohr, floor {floor}");
            assert!(
                m > floor,
                "{name}: two atoms open {m:.6} bohr apart, inside the {floor} bohr floor"
            );
        }
    }
}

// ------------------------------------------------------------------ G5 loadability

#[test]
fn g5_the_scene_loads_into_a_periodic_sim() {
    // The setup idiom of tests/scale_box.rs, with ONE deliberate departure: no dynamics
    // are run. viewer/h2_potential.json is an H-H curve, and the O-O and O-H curves an
    // ice scene needs are not loadable in a unit test — so this gate asserts that the
    // scene can be PLACED in a periodic box and that every coordinate the builder
    // returned is finite and inside it. Stepping this Sim would be reading a hydrogen
    // curve for an oxygen pair and calling the number physics; the gate stops short of
    // that on purpose rather than faking a table.
    for &c in CELLS.iter() {
        for (name, sc) in [
            (format!("XI {c:?}"), ice_xi(c, 1.0)),
            (format!("VIII {c:?}"), ice_viii(c, 1.0)),
            (format!("X {c:?}"), ice_x(c, 1.0)),
        ] {
            let (atoms, b) = sc;
            let mut s = Box::new(Sim::empty());
            holon_render::json::load_into(
                s.table_mut(),
                &std::fs::read_to_string(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/viewer/h2_potential.json"
                ))
                .expect("placeholder curve readable"),
            )
            .expect("table loads");
            s.adopt_table_timescale();
            s.dims = Dims::Three;
            s.boundary = Boundary::Periodic;
            s.width = b.0;
            s.height = b.1;
            s.depth = b.2;
            s.reset(atoms.len());
            for (i, &(z, x, y, zz)) in atoms.iter().enumerate() {
                s.atoms[i].species =
                    holon_chem::elements::by_z(z as u32).expect("H and O are in the registry");
                s.set_position_3d(i, x, y, zz);
                s.set_velocity_3d(i, 0.0, 0.0, 0.0);
            }
            assert!(s.sync_species(), "{name}: H and O both fit the bank's species cap");
            s.rebase();

            assert_eq!(s.n, atoms.len(), "{name}: every atom made it into the scene");
            for (i, a) in s.atoms.iter().enumerate() {
                for (axis, (v, l)) in [(a.x, b.0), (a.y, b.1), (a.z, b.2)].iter().enumerate() {
                    assert!(v.is_finite(), "{name}: atom {i} axis {axis} is not finite");
                    assert!(
                        *v >= 0.0 && *v < *l,
                        "{name}: atom {i} axis {axis} at {v} is outside [0, {l})"
                    );
                }
            }
            println!("{name}: {} atoms placed in a periodic box {:?}", s.n, b);
        }
    }
}

// ------------------------------------------------------------------ G6 the plant

/// The shipped builder's output with ONE hydrogen moved by `d` bohr, in a direction
/// chosen by `along`: along its own O–H axis when true, perpendicular to it when false.
///
/// Which direction is used decides which gate can see the damage, and the two tests
/// below name that rather than leaving it to luck. A perpendicular 0.5 bohr shift puts
/// the hydrogen at √(1.81² + 0.5²) = 1.878 bohr, still INSIDE G3's 2.2 bohr covalent
/// window, so G3's counts do not change and only G2 fires. A shift of the same size
/// along the axis puts it at 2.31 bohr, outside that window, and fires both.
fn plant(sc: Scene, d: f64, along: bool) -> Scene {
    let (mut atoms, b) = sc;
    // Index 0 is an oxygen and index 1 is one of its two hydrogens, by construction of
    // every builder in the module.
    let o = (atoms[0].1, atoms[0].2, atoms[0].3);
    let h = (atoms[1].1, atoms[1].2, atoms[1].3);
    assert_eq!(atoms[0].0, 8);
    assert_eq!(atoms[1].0, 1);
    let mut u = (h.0 - o.0, h.1 - o.1, h.2 - o.2);
    let n = (u.0 * u.0 + u.1 * u.1 + u.2 * u.2).sqrt();
    u = (u.0 / n, u.1 / n, u.2 / n);
    let dir = if along {
        u
    } else {
        // any unit vector orthogonal to u: cross with the least-aligned axis
        let e = if u.0.abs() < 0.9 {
            (1.0, 0.0, 0.0)
        } else {
            (0.0, 1.0, 0.0)
        };
        let c = (
            u.1 * e.2 - u.2 * e.1,
            u.2 * e.0 - u.0 * e.2,
            u.0 * e.1 - u.1 * e.0,
        );
        let m = (c.0 * c.0 + c.1 * c.1 + c.2 * c.2).sqrt();
        (c.0 / m, c.1 / m, c.2 / m)
    };
    atoms[1].1 = (h.0 + d * dir.0).rem_euclid(b.0);
    atoms[1].2 = (h.1 + d * dir.1).rem_euclid(b.1);
    atoms[1].3 = (h.2 + d * dir.2).rem_euclid(b.2);
    (atoms, b)
}

#[test]
fn g6_the_plant_fires_g2() {
    for (name, sc) in [
        ("XI", ice_xi((2, 2, 2), 1.0)),
        ("VIII", ice_viii((2, 2, 2), 1.0)),
    ] {
        for along in [true, false] {
            let bad = plant(sc.clone(), 0.5, along);
            let e = covalent_err(&bad);
            println!("{name} plant (along={along}): max |O-H - 1.81| = {e:.6} bohr");
            assert!(
                e >= 1e-6,
                "{name}: G2 did NOT fire on a 0.5 bohr displacement (along={along}) — \
                 the gate is one-directional and proves nothing"
            );
        }
    }
    // The same plant on ice X must move its own gate, the midpoint condition.
    let bad = plant(ice_x((2, 2, 2), 1.0), 0.5, false);
    let e = midpoint_err(&bad);
    println!("X plant (perpendicular): midpoint error = {e:.6} bohr");
    assert!(e >= 1e-6, "X: G2's midpoint condition did NOT fire on a displaced H");
}

#[test]
fn g6_the_plant_fires_g3_when_it_leaves_the_covalent_shell() {
    // The ALONG plant leaves the 2.2 bohr window, so the ice-rule counts must break.
    for (name, sc) in [
        ("XI", ice_xi((2, 2, 2), 1.0)),
        ("VIII", ice_viii((2, 2, 2), 1.0)),
    ] {
        let bad = plant(sc.clone(), 0.5, true);
        let c = bf_counts(&bad, 1.0);
        let broken: Vec<_> = c.iter().enumerate().filter(|(_, &x)| x != (2, 2)).collect();
        println!(
            "{name} plant (along): {} oxygens break (2,2); first = site {} reads {:?}",
            broken.len(),
            broken[0].0,
            broken[0].1
        );
        assert!(
            !broken.is_empty(),
            "{name}: G3 did NOT fire on a hydrogen pushed out of the covalent shell"
        );

        // And the PERPENDICULAR plant must NOT fire G3 — naming which prong each
        // mutation reaches, instead of claiming one mutation reaches both.
        let bad = plant(sc.clone(), 0.5, false);
        let c = bf_counts(&bad, 1.0);
        let broken = c.iter().filter(|&&x| x != (2, 2)).count();
        println!("{name} plant (perpendicular): {broken} oxygens break (2,2) — expected 0");
        assert_eq!(
            broken, 0,
            "{name}: a 0.5 bohr perpendicular shift stays inside the 2.2 bohr window, so \
             G3 is blind to it by design; if it fired, the window moved"
        );
    }
}
