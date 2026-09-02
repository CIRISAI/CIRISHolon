//! NODE E's four staked scenes, and the raw-bit checkpoint dump taken on them.
//!
//! # Why this is a module and not an example
//!
//! Three callers need the identical scenes and the identical dump: `examples/node_e.rs`
//! (the campaign runner), `tests/node_e.rs` (the fast gates), and a build of this file at
//! the commit BEFORE `Sim::step` was factored, which is how the factoring is shown to
//! have moved no bit. Three implementations of one scene is how the three of them come to
//! disagree about what they are comparing, so there is one, included by `#[path]` — the
//! arrangement `tests/common/b1_dump.rs` already uses for the same reason.
//!
//! # Why bits
//!
//! `{:016x}` of `f64::to_bits` is the number itself. Printed decimal round-trips, but a
//! comparison written against a decimal string is a comparison the reader has to trust
//! the formatter for, and this file exists to serve gates whose criterion is EXACT.
//!
//! # The fixtures are carried, and the carry is gated
//!
//! `staked_s1` and `staked_s2` are VERBATIM copies of `staked_nve` in `tests/ledger.rs`
//! and `staked_nve_3d` in `tests/three_dimensions.rs`. A copy is a liability the moment
//! nobody checks it, so `tests/node_e.rs` extracts those two functions from their original
//! sources by name and byte-compares the bodies against these. The originals are NOT
//! edited: they belong to gates other lanes rely on, and a fixture that changes because
//! someone wanted a banner around it is a fixture that changed.

#![allow(dead_code)]

use holon_render::sim::{Boundary, Dims, Sim};

pub fn potential_source() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/viewer/h2_potential.json");
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("cannot read {path}: {e}. Run: cargo run -p holon-render --example make_placeholder")
    })
}

pub fn loaded_sim() -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(s.table_mut(), &potential_source()).expect("table loads");
    s.adopt_table_timescale();
    s
}

// ================================================================ THE STAKED SCENES

/// S1 — `staked_nve`, verbatim from `tests/ledger.rs`.
///
/// 2D, open, two hydrogens: a bound vibrating pair at R = 2.2 bohr with the centre of
/// mass drifting so the momentum gate has something non-zero to conserve. Held here
/// because a 2D scene pins `z` and `vz` at exact zeros, which is where the signed-zero
/// hazard of the freeze's §3.2 lives if it lives anywhere.
pub fn staked_s1() -> Box<Sim> {
    let mut s = loaded_sim();
    s.boundary = Boundary::Open;
    s.reset(2);
    let cx = 0.5 * s.width;
    let cy = 0.5 * s.height;
    s.set_position(0, cx - 1.1, cy);
    s.set_position(1, cx + 1.1, cy);
    s.set_velocity(0, 0.002, 0.001);
    s.set_velocity(1, -0.002, 0.001);
    s.rebase();
    s
}

/// The rotation `tests/three_dimensions.rs` tilts its scene by. Verbatim.
fn rotate(v: (f64, f64, f64)) -> (f64, f64, f64) {
    let (a, b) = (0.7f64, 1.3f64);
    let (x, y, z) = v;
    // Rx(a)
    let (y, z) = (y * a.cos() - z * a.sin(), y * a.sin() + z * a.cos());
    // Rz(b)
    let (x, y) = (x * b.cos() - y * b.sin(), x * b.sin() + y * b.cos());
    (x, y, z)
}

/// S2 — `staked_nve_3d`, verbatim from `tests/three_dimensions.rs`.
///
/// S1 tilted out of plane, so every coordinate, every velocity component and the pair's
/// angular momentum vector all carry all three components. This is the scene the ANGULAR
/// law is gated on, because it is the one with an angular momentum to conserve.
pub fn staked_s2() -> Box<Sim> {
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    s.boundary = Boundary::Open;
    s.reset(2);
    let c = (0.5 * s.width, 0.5 * s.height, 0.5 * s.depth);
    let half = rotate((1.1, 0.0, 0.0));
    let v_rel = rotate((0.002, 0.0, 0.0));
    let v_com = rotate((0.0, 0.001, 0.0));
    s.set_position_3d(0, c.0 - half.0, c.1 - half.1, c.2 - half.2);
    s.set_position_3d(1, c.0 + half.0, c.1 + half.1, c.2 + half.2);
    s.set_velocity_3d(0, v_rel.0 + v_com.0, v_rel.1 + v_com.1, v_rel.2 + v_com.2);
    s.set_velocity_3d(1, -v_rel.0 + v_com.0, -v_rel.1 + v_com.1, -v_rel.2 + v_com.2);
    s.rebase();
    s
}

/// S3 — walls-3, specified in the freeze §2.
///
/// Three hydrogens in a walled 3D box. The wall term is EXTERNAL, so `w_ext` and `j_ext`
/// carry something and a checkpoint comparison is comparing ledgers that are not three
/// zeros. No grab: the freeze refuses the hand, because on a ring it is ambiguous whether
/// it pulls the centroid or every bead. The angular row is VOID by construction here —
/// walls break rotational symmetry and `Sim::angular_gate` returns `None`.
pub fn staked_s3() -> Box<Sim> {
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    s.boundary = Boundary::Walls;
    s.reset(3);
    let (cx, cy, cz) = (0.5 * s.width, 0.5 * s.height, 0.5 * s.depth);
    s.set_position_3d(0, cx - 1.4, cy, cz);
    s.set_position_3d(1, cx + 1.4, cy, cz);
    s.set_position_3d(2, cx, cy + 3.0, cz);
    s.set_velocity_3d(0, 0.004, 0.0, 0.0);
    s.set_velocity_3d(1, -0.004, 0.0, 0.0);
    s.set_velocity_3d(2, 0.0, -0.004, 0.0);
    s.rebase();
    s
}

/// S4 — periodic-27, specified in the freeze §2.
///
/// A 3x3x3 cubic lattice at 3.0 bohr in a periodic box, built the way
/// `tests/t3_scale.rs::lattice` builds one — and then given velocities, because a
/// symmetric lattice is a FORCE-BALANCED FIXED POINT and a conservation gate on a scene
/// that does not move is vacuous (M-FIXED-POINT-TRAJECTORY, M-VACUOUS-SUCCESS). The
/// velocities are deterministic and RNG-free so a reported run re-runs byte for byte.
///
/// This is the only scene that exercises THE WRAP, the cell decomposition and the
/// minimum image, and therefore the only one on which the centroid-wrap rule means
/// anything.
pub fn staked_s4() -> Box<Sim> {
    const NX: usize = 3;
    const SPACING: f64 = 3.0;
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    s.boundary = Boundary::Periodic;
    s.width = NX as f64 * SPACING;
    s.height = NX as f64 * SPACING;
    s.depth = NX as f64 * SPACING;
    let n = NX * NX * NX;
    s.resize_storage(n);
    for i in 0..n {
        let ix = i % NX;
        let iy = (i / NX) % NX;
        let iz = i / (NX * NX);
        s.atoms[i].x = (ix as f64 + 0.5) * SPACING;
        s.atoms[i].y = (iy as f64 + 0.5) * SPACING;
        s.atoms[i].z = (iz as f64 + 0.5) * SPACING;
        let f = i as f64;
        s.atoms[i].vx = 0.002 * (f + 1.0).sin();
        s.atoms[i].vy = 0.002 * (2.0 * f + 1.0).sin();
        s.atoms[i].vz = 0.002 * (3.0 * f + 1.0).sin();
    }
    s.sync_species();
    s.rebase();
    s
}

/// The four, in freeze order, with their labels.
pub fn staked_scenes() -> Vec<(&'static str, Box<Sim>)> {
    vec![
        ("S1", staked_s1()),
        ("S2", staked_s2()),
        ("S3", staked_s3()),
        ("S4", staked_s4()),
    ]
}

// ================================================================ THE DUMP

/// The graded run length: the classical NVE gate's own staked 156 frames of 64 substeps,
/// taken rather than invented.
pub const FRAMES: usize = 156;
pub const SUBSTEPS: u32 = 64;

/// Every atom's six coordinates and every ledger the checkpoint carries, as raw bits,
/// after the graded run. One line per scene.
///
/// The checkpoint is the identity object rather than a positions-only digest because it
/// carries `w_ext`, the work columns, `l0`, `p0`, `j_ext` and `time` alongside the atoms —
/// so a path that got the IMPULSE BOOKKEEPING wrong is caught here even if every position
/// matched.
pub fn dump_classical() -> String {
    let mut out = String::new();
    for (label, mut s) in staked_scenes() {
        for _ in 0..FRAMES {
            s.step_frame(SUBSTEPS);
        }
        out.push_str(&dump_one(label, &s));
    }
    out
}

pub fn dump_one(label: &str, s: &Sim) -> String {
    let cp = s.checkpoint();
    let mut out = format!(
        "{label} n={} bytes={} digest={:016x} steps={} time={:016x} drift_peak={:016x}\n",
        s.n,
        cp.len(),
        cp.digest(),
        s.steps,
        s.time.to_bits(),
        s.drift_peak.to_bits(),
    );
    for i in 0..s.n {
        let a = &s.atoms[i];
        out.push_str(&format!(
            "{label} a{i} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}\n",
            a.x.to_bits(),
            a.y.to_bits(),
            a.z.to_bits(),
            a.vx.to_bits(),
            a.vy.to_bits(),
            a.vz.to_bits(),
        ));
    }
    out
}

/// Negative zeros produced by the classical arm, counted per scene over the graded run.
///
/// A SCOUTING measurement, taken on the classical engine alone and reported whatever it
/// says. `NormalModes::to_modes` computes `0.0 + 1.0 * x`, and `0.0 + (-0.0)` is `+0.0`,
/// so a negative-zero component would come back positive and move a checkpoint byte while
/// moving no physics. This counts how much of that hazard the staked scenes actually
/// carry. It cannot change any criterion in the freeze and is not a gate.
pub fn count_negative_zeros() -> String {
    let mut out = String::new();
    for (label, mut s) in staked_scenes() {
        let mut seen: u64 = 0;
        let mut samples: u64 = 0;
        for _ in 0..FRAMES {
            s.step_frame(SUBSTEPS);
            for i in 0..s.n {
                let a = &s.atoms[i];
                for v in [a.x, a.y, a.z, a.vx, a.vy, a.vz] {
                    samples += 1;
                    if v == 0.0 && v.is_sign_negative() {
                        seen += 1;
                    }
                }
            }
        }
        out.push_str(&format!(
            "{label} negative_zeros={seen} of samples={samples}\n"
        ));
    }
    out
}
