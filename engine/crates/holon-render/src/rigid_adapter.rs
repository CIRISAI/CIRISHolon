//! THE ADAPTER between a [`Sim`]'s water units and `holon-runtime`'s rigid-water operator
//! (GANTT2, the fourth review, WP2). The same shape as `closure.rs`: thin, over readings the
//! engine already makes, adding no rule of its own and touching nothing in `sim.rs`.
//!
//! What it does: name a unit's three atoms from [`Sim::units_reading`]; read them as the
//! operator's [`Fine`] state (positions UNWRAPPED around the unit's own oxygen by the box's
//! minimum image, velocities, the engine's masses); read the force the last pass left on
//! each of them as the operator's [`SiteForces`]; and write a [`Fine`] state back. The
//! reference body is the engine's own pinned monomer in the engine's own masses.
//!
//! What it does NOT do: integrate. No `Sim` frame is replaced by an operator step here — that
//! is the `replace0` runner's work, and until it exists nothing in this crate runs a rigid
//! water. The adapter is the reading; the replacement is a freeze.

use crate::field::{WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD};
use crate::seam::FREE;
use crate::sim::{Sim, M_H};
use holon_chem::elements::OXYGEN;
use holon_runtime::rigid::{Body, Fine, RigidWater, SiteForces, Sites};
use holon_runtime::{Discarded, Operator, Refusal};

/// The engine's pinned monomer as the operator's reference body: oxygen at the origin, the
/// hydrogens at `WATER_PIN_R_BOHR` and `WATER_PIN_THETA_RAD` in the xy plane, masses the
/// engine's (`M_H`; oxygen's `mass_me()`). The arrangement is immaterial — the operator fits a
/// fine state by its own declared frame — the geometry and the masses are what count.
pub fn reference_body() -> Result<Body, Refusal> {
    reference_body_from(WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD)
}

/// A reference body at a DECLARED geometry - the liquid's own mean O-H length and H-O-H
/// angle, measured at a branch point, is the one a rigid replacement of the liquid should
/// carry (REPLACE-0's first matched run: snapping every monomer to the gas-phase pin released
/// ~0.7 kT per water of potential, because the liquid's donor O-H bonds are longer than the
/// pin's and every hydrogen bond was weakened at once). Masses the engine's.
pub fn reference_body_from(r_oh_bohr: f64, theta_rad: f64) -> Result<Body, Refusal> {
    let (s, c) = (0.5 * theta_rad).sin_cos();
    let r = r_oh_bohr;
    Body::from_monomer([[0.0, 0.0, 0.0], [r * c, r * s, 0.0], [r * c, -r * s, 0.0]], [OXYGEN.mass_me(), M_H, M_H])
}

/// The scene's MEAN monomer geometry over its water units: the mean O-H length over both
/// bonds of every unit and the mean H-O-H angle, with their standard deviations, read
/// through the box's own minimum image. A measurement, for a record and for
/// [`reference_body_from`]; the pin is not consulted.
pub fn mean_monomer_geometry(sim: &Sim, units: &[UnitMembers]) -> MonomerGeometry {
    let g = sim.geom();
    let at = |i: usize| (sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z);
    let mut r = Vec::with_capacity(2 * units.len());
    let mut th = Vec::with_capacity(units.len());
    for m in units {
        let o = at(m.o);
        let d1 = g.delta(o, at(m.h[0]));
        let d2 = g.delta(o, at(m.h[1]));
        let r1 = (d1.0 * d1.0 + d1.1 * d1.1 + d1.2 * d1.2).sqrt();
        let r2 = (d2.0 * d2.0 + d2.1 * d2.1 + d2.2 * d2.2).sqrt();
        r.push(r1);
        r.push(r2);
        let cos = (d1.0 * d2.0 + d1.1 * d2.1 + d1.2 * d2.2) / (r1 * r2);
        th.push(cos.clamp(-1.0, 1.0).acos());
    }
    let stat = |v: &[f64]| {
        let n = v.len().max(1) as f64;
        let mean = v.iter().sum::<f64>() / n;
        let var = v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / n;
        (mean, var.sqrt())
    };
    let (r_mean, r_sd) = stat(&r);
    let (t_mean, t_sd) = stat(&th);
    MonomerGeometry { r_oh_bohr: r_mean, r_oh_sd_bohr: r_sd, theta_rad: t_mean, theta_sd_rad: t_sd, units: units.len() }
}

/// See [`mean_monomer_geometry`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonomerGeometry {
    pub r_oh_bohr: f64,
    pub r_oh_sd_bohr: f64,
    pub theta_rad: f64,
    pub theta_sd_rad: f64,
    pub units: usize,
}

/// One water unit's atoms, by the engine's own indices: the oxygen root and its two hydrogens
/// in ascending index order (the operator's DECLARED site order O, H, H).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitMembers {
    pub o: usize,
    pub h: [usize; 2],
}

/// Every unit with exactly two hydrogens, in ascending oxygen order, from a
/// [`Sim::units_reading`] (`units[o] == o` marks a root, `units[h] == o` a member, `FREE` an
/// atom in none). Units with any other hydrogen count are NOT water and are returned apart,
/// by root, so a caller cannot silently rigidify an H3O+ or an OH-.
pub fn unit_members(units: &[u32]) -> (Vec<UnitMembers>, Vec<usize>) {
    let n = units.len();
    let mut water = Vec::new();
    let mut other = Vec::new();
    for o in 0..n {
        if units[o] != o as u32 {
            continue;
        }
        let hs: Vec<usize> = (0..n).filter(|&h| h != o && units[h] != FREE && units[h] == o as u32).collect();
        if hs.len() == 2 {
            water.push(UnitMembers { o, h: [hs[0], hs[1]] });
        } else {
            other.push(o);
        }
    }
    (water, other)
}

/// The unit as the operator's fine state. Positions are the oxygen's own plus the box's
/// minimum-image displacement to each hydrogen, so a unit straddling a face is read whole;
/// the oxygen itself is left where the box keeps it.
pub fn fine_of(sim: &Sim, m: &UnitMembers, body: &Body) -> Fine {
    let g = sim.geom();
    let at = |i: usize| (sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z);
    let o = at(m.o);
    let mut pos = [[o.0, o.1, o.2], [0.0; 3], [0.0; 3]];
    let mut vel = [[0.0; 3]; 3];
    let mut mass = [0.0; 3];
    for (k, &i) in [m.o, m.h[0], m.h[1]].iter().enumerate() {
        if k > 0 {
            let (dx, dy, dz) = g.delta(o, at(i));
            pos[k] = [o.0 + dx, o.1 + dy, o.2 + dz];
        }
        vel[k] = [sim.atoms[i].vx, sim.atoms[i].vy, sim.atoms[i].vz];
        mass[k] = sim.atoms[i].mass();
    }
    Fine { sites: Sites { pos, vel, mass }, reference: *body }
}

/// The force the LAST PASS left on the unit's three atoms, internal plus external, hartree
/// per bohr — the operator's exchange. Read, never recomputed: a caller that wants the force
/// at a new geometry calls [`Sim::compute_forces`] first.
pub fn site_forces_of(sim: &Sim, m: &UnitMembers) -> SiteForces {
    let mut f = [[0.0; 3]; 3];
    for (k, &i) in [m.o, m.h[0], m.h[1]].iter().enumerate() {
        let (ax, ay, az) = sim.internal_force(i);
        let (bx, by, bz) = sim.external_force(i);
        f[k] = [ax + bx, ay + by, az + bz];
    }
    SiteForces { f }
}

/// Write a fine state back onto the unit's atoms through the engine's own setters. Positions
/// are written as given (unwrapped around the oxygen); the box's own image rule takes them
/// from there, as it does for any placed atom.
pub fn write_back(sim: &mut Sim, m: &UnitMembers, fine: &Fine) {
    for (k, &i) in [m.o, m.h[0], m.h[1]].iter().enumerate() {
        let p = fine.sites.pos[k];
        let v = fine.sites.vel[k];
        sim.set_position_3d(i, p[0], p[1], p[2]);
        sim.set_velocity_3d(i, v[0], v[1], v[2]);
    }
}

/// Every water unit of the scene projected onto the operator, with what each projection
/// discarded. Refuses on the first unit the operator refuses, by name.
pub fn project_all(sim: &Sim, body: &Body) -> Result<(Vec<(UnitMembers, RigidWater, Discarded)>, Vec<usize>), Refusal> {
    let (water, other) = unit_members(&sim.units_reading());
    let mut out = Vec::with_capacity(water.len());
    for m in water {
        let (w, d) = RigidWater::project(&fine_of(sim, &m, body))?;
        out.push((m, w, d));
    }
    Ok((out, other))
}
