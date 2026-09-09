//! THE RIGID ADAPTER ON THE REAL BOX (GANTT2, the fourth review, WP2): `holon-runtime`'s
//! rigid-water operator read from and written to the engine's own 128-water liquid box, on the
//! engine's own law. Nothing here integrates a rigid step — the adapter is the reading — and
//! the one physics claim tested is the one the operator makes: its accumulated torque and net
//! force are minus the gradient of the ENGINE's energy under a rigid rotation and translation
//! of one unit, to finite-difference precision.

use holon_render::channel::Row;
use holon_render::rigid_adapter::{fine_of, project_all, reference_body, site_forces_of, unit_members, write_back};
use holon_render::seam::SeamModel;
use holon_render::sim::{Boundary, Sim};
use holon_render::waterbox::liquid_box;
use holon_runtime::rigid::{add, cross, dot, normalize, quat_mul, scale, sub, RigidWater};
use holon_runtime::Operator;

#[path = "common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::scene;

const DENSITY_G_CM3: f64 = 0.997;
const N_CELLS: usize = 4;
const SEED: u64 = 0x4c49_5155_4944;

/// FIELD-9's harvested law, as `tests/liquid.rs` declares it.
fn declared_law() -> SeamModel {
    SeamModel { a: 948.0, b: 2.4, p: 16.26, c: 2.06, a_oh: 22.59, b_oh: 2.2, a_hh: 1.525, b_hh: 1.75, ..SeamModel::NO_WALL }
}

fn the_box() -> (Box<Sim>, f64) {
    let (sp, pos, l) = liquid_box(N_CELLS, DENSITY_G_CM3, SEED);
    let mut s = scene(&sp, &pos, l, 293.0);
    s.set_field(true, None).expect("the open box admits the field");
    s.set_seam(Some(declared_law())).expect("no acuity frame");
    s.boundary = Boundary::Periodic;
    s.compute_forces();
    (s, l)
}

fn potential(s: &Sim) -> f64 {
    s.energy() - s.row(Row::Kin)
}

fn min_image_dist(a: [f64; 3], b: [f64; 3], l: f64) -> f64 {
    let mut acc = 0.0;
    for k in 0..3 {
        let mut d = b[k] - a[k];
        d -= l * (d / l).round();
        acc += d * d;
    }
    acc.sqrt()
}

#[test]
fn every_unit_of_the_start_box_is_water_and_projects_rigidly() {
    let (s, _) = the_box();
    let body = reference_body().expect("the pinned monomer is principal");
    let (units, other) = project_all(&s, &body).expect("every unit projects");
    assert_eq!(units.len(), 128, "128 waters");
    assert!(other.is_empty(), "no unit with a hydrogen count other than two: {other:?}");
    let mut worst = 0.0f64;
    for (m, w, d) in &units {
        // the box places the pinned monomer exactly, so the rigid lift loses nothing
        assert!(d.deformation_rms < 1e-9, "unit at oxygen {}: deformation {}", m.o, d.deformation_rms);
        worst = worst.max(d.deformation_rms);
        let fine = fine_of(&s, m, &body);
        let back = w.reconstruct();
        for i in 0..3 {
            for k in 0..3 {
                assert!((fine.sites.pos[i][k] - back.sites.pos[i][k]).abs() < 1e-9);
            }
        }
        // the random start velocities are not a rigid field; what they carry beyond one is recorded
        assert!(d.internal_kinetic >= 0.0);
        assert_eq!(w.dof(), 6);
    }
    eprintln!("start box: 128 units, worst deformation {worst:.3e} bohr");
}

#[test]
fn the_operators_torque_and_force_are_minus_the_gradient_of_the_engines_law() {
    let (mut s, _) = the_box();
    let body = reference_body().unwrap();
    let (water, _) = unit_members(&s.units_reading());
    let m = water[0];
    let fine0 = fine_of(&s, &m, &body);
    let (w, _) = RigidWater::project(&fine0).unwrap();
    let (f, tau) = w.accumulate(&site_forces_of(&s, &m));
    let delta = 1e-4;
    let u_at = |s: &mut Sim, b: &RigidWater| -> f64 {
        write_back(s, &m, &b.reconstruct());
        s.compute_forces();
        potential(s)
    };
    for axis in 0..3 {
        // a LAB rotation about the unit's own centre: R' = Rot_n(d) R
        let rotated = |d: f64| {
            let (hs, hc) = (0.5 * d).sin_cos();
            let mut qn = [hc, 0.0, 0.0, 0.0];
            qn[1 + axis] = hs;
            let mut b = w;
            b.q = quat_mul(&qn, &w.q);
            normalize(&mut b.q);
            b
        };
        let up = u_at(&mut s, &rotated(delta));
        let dn = u_at(&mut s, &rotated(-delta));
        let du = (up - dn) / (2.0 * delta);
        let scale_ = tau[axis].abs().max(du.abs()).max(1e-9);
        assert!(
            ((-tau[axis]) - du).abs() <= 1e-4 * scale_,
            "axis {axis}: -tau {:.6e} against dU/dphi {du:.6e} (rel {:.2e})",
            -tau[axis],
            ((-tau[axis]) - du).abs() / scale_
        );
        let moved = |d: f64| {
            let mut b = w;
            b.com[axis] += d;
            b
        };
        let up = u_at(&mut s, &moved(delta));
        let dn = u_at(&mut s, &moved(-delta));
        let du = (up - dn) / (2.0 * delta);
        let scale_ = f[axis].abs().max(du.abs()).max(1e-9);
        assert!(((-f[axis]) - du).abs() <= 1e-4 * scale_, "axis {axis}: -F {:.6e} against dU/dx {du:.6e}", -f[axis]);
    }
    // the unit is put back where it was and the pass reproduces its energy
    write_back(&mut s, &m, &fine0);
    s.compute_forces();
    let (w2, _) = RigidWater::project(&fine_of(&s, &m, &body)).unwrap();
    for k in 0..3 {
        assert!((w2.com[k] - w.com[k]).abs() < 1e-12);
    }
    eprintln!("unit at oxygen {}: |F| {:.3e} Ha/bohr, |tau| {:.3e} Ha", m.o, dot(f, f).sqrt(), dot(tau, tau).sqrt());
}

#[test]
fn after_fine_frames_the_deformation_and_internal_kinetic_energy_are_recorded() {
    let (mut s, _) = the_box();
    let body = reference_body().unwrap();
    for _ in 0..50 {
        s.step_frame(1);
    }
    let (units, other) = project_all(&s, &body).expect("every unit still projects");
    assert_eq!(units.len() + other.len(), 128);
    let worst = units.iter().map(|(_, _, d)| d.deformation_rms).fold(0.0, f64::max);
    let mean_int = units.iter().map(|(_, _, d)| d.internal_kinetic).sum::<f64>() / units.len() as f64;
    assert!(worst > 1e-4, "fifty fine frames deform a flexible monomer: worst {worst:.3e}");
    assert!(worst < 0.5, "and not into something that is no longer that monomer: {worst:.3e}");
    assert!(mean_int > 0.0);
    eprintln!("after 50 frames: worst deformation {worst:.3e} bohr, mean internal KE {mean_int:.3e} Ha, non-water units {}", other.len());
}

#[test]
fn write_back_then_read_is_the_identity_up_to_the_boxs_image() {
    let (mut s, l) = the_box();
    let body = reference_body().unwrap();
    let (water, _) = unit_members(&s.units_reading());
    let m = water[5];
    let (w, _) = RigidWater::project(&fine_of(&s, &m, &body)).unwrap();
    // move it by a rigid motion that puts a hydrogen through a face, then read it back
    let mut b = w;
    b.com = add(b.com, scale(sub([l, l, l], b.com), 0.999));
    let recon = b.reconstruct();
    write_back(&mut s, &m, &recon);
    let again = fine_of(&s, &m, &body);
    for i in 0..3 {
        assert!(min_image_dist(recon.sites.pos[i], again.sites.pos[i], l) < 1e-12, "site {i}");
        for k in 0..3 {
            assert!((recon.sites.vel[i][k] - again.sites.vel[i][k]).abs() < 1e-15);
        }
    }
    // and the geometry read back is the reference geometry, whole across the face
    let r1 = dot(sub(again.sites.pos[1], again.sites.pos[0]), sub(again.sites.pos[1], again.sites.pos[0])).sqrt();
    assert!((r1 - holon_render::field::WATER_PIN_R_BOHR).abs() < 1e-9, "O-H across the face {r1}");
    let _ = cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
}
