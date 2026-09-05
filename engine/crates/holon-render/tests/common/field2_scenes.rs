//! FIELD-2's scenes (`examples/field2_hbonds.rs`), shared with FIELD-3's runners and gates
//! by `#[path]` so the starts, seeds and the lens read are ONE definition: the dimer, the
//! cyclic tetramer, FIELD-1's parallel square, the thermostatted open-box scene, and the
//! rung-1 lens count. Moved here verbatim under FIELD-3; FIELD-2's numbers are unchanged.
#![allow(dead_code)]
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::embed::water_centers;
use holon_render::field::{WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD};
use holon_render::sim::{Boundary, Dims, Sim};

#[path = "quartet.rs"]
#[allow(dead_code)]
pub mod quartet;

pub const SETTLE: usize = 2000;
pub const COUNT: usize = 20000;
pub const R_OO: f64 = 5.5;
pub const K_B: f64 = 3.166811563e-6; // hartree per kelvin

pub fn unit(v: [f64; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / n, v[1] / n, v[2] / n]
}
pub fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

/// A water at `o` donating its first hydrogen along `dir`, the second hydrogen in the plane
/// spanned by `dir` and `side` at the pin angle.
pub fn water_at(o: [f64; 3], dir: [f64; 3], side: [f64; 3]) -> [[f64; 3]; 3] {
    let d = unit(dir);
    let n = unit(cross(cross(d, side), d)); // side made perpendicular to d
    let (r, t) = (WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD);
    let h1 = [o[0] + r * d[0], o[1] + r * d[1], o[2] + r * d[2]];
    let (c, s) = (t.cos(), t.sin());
    let h2 = [o[0] + r * (c * d[0] + s * n[0]), o[1] + r * (c * d[1] + s * n[1]), o[2] + r * (c * d[2] + s * n[2])];
    [o, h1, h2]
}

pub fn dimer_positions() -> (Vec<holon_chem::elements::Species>, Vec<[f64; 3]>) {
    let c = 15.0;
    let donor = water_at([c, c, c], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    // the acceptor: its C2 axis on the O···O axis, hydrogens on the far side
    let mono = water_centers(WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD);
    let acc: Vec<[f64; 3]> = mono.iter().map(|m| [c + m[0], c + m[1], c + R_OO + m[2]]).collect();
    let mut species = Vec::new();
    let mut pos = Vec::new();
    for (k, p) in donor.iter().chain(acc.iter()).enumerate() {
        species.push(if k % 3 == 0 { OXYGEN } else { HYDROGEN });
        pos.push(*p);
    }
    (species, pos)
}

pub fn ring_positions() -> (Vec<holon_chem::elements::Species>, Vec<[f64; 3]>) {
    let c = 17.0;
    let h = 0.5 * R_OO;
    let corners = [[c - h, c - h, c], [c + h, c - h, c], [c + h, c + h, c], [c - h, c + h, c]];
    let mut species = Vec::new();
    let mut pos = Vec::new();
    for k in 0..4 {
        let o = corners[k];
        let next = corners[(k + 1) % 4];
        let dir = [next[0] - o[0], next[1] - o[1], next[2] - o[2]];
        let side = if k % 2 == 0 { [0.0, 0.0, 1.0] } else { [0.0, 0.0, -1.0] };
        for (m, p) in water_at(o, dir, side).iter().enumerate() {
            species.push(if m == 0 { OXYGEN } else { HYDROGEN });
            pos.push(*p);
        }
    }
    (species, pos)
}

/// FIELD-1's parallel-dipole square, for plant (ii).
pub fn square_positions() -> (Vec<holon_chem::elements::Species>, Vec<[f64; 3]>) {
    let mono = water_centers(WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD);
    let oxygens = [[13.0, 13.0, 17.0], [20.0, 13.0, 17.0], [13.0, 20.0, 17.0], [20.0, 20.0, 17.0]];
    let mut species = Vec::new();
    let mut pos = Vec::new();
    for (k, o) in oxygens.iter().enumerate() {
        let flip = if k % 2 == 0 { 1.0 } else { -1.0 };
        for (m, c) in mono.iter().enumerate() {
            species.push(if m == 0 { OXYGEN } else { HYDROGEN });
            pos.push([o[0] + flip * c[0], o[1] + c[2], o[2] + c[1]]);
        }
    }
    (species, pos)
}

pub fn scene(species: &[holon_chem::elements::Species], pos: &[[f64; 3]], box_edge: f64, temp: f64) -> Box<Sim> {
    let mut s = quartet::scene(species, pos, false);
    s.dims = Dims::Three;
    s.boundary = Boundary::Open;
    s.width = box_edge;
    s.height = box_edge;
    s.depth = box_edge;
    let mut st: u64 = 0x4649_454c_3200;
    let mut lcg = || {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((st >> 11) as f64) / ((1u64 << 53) as f64)
    };
    let n = s.n;
    let (mut px, mut py, mut pz) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let m = s.atoms[i].mass();
        let scale = (K_B * temp / m).sqrt();
        s.atoms[i].vx = scale * (2.0 * lcg() - 1.0) * 1.7;
        s.atoms[i].vy = scale * (2.0 * lcg() - 1.0) * 1.7;
        s.atoms[i].vz = scale * (2.0 * lcg() - 1.0) * 1.7;
        px += m * s.atoms[i].vx;
        py += m * s.atoms[i].vy;
        pz += m * s.atoms[i].vz;
    }
    let mtot: f64 = (0..n).map(|i| s.atoms[i].mass()).sum();
    for i in 0..n {
        s.atoms[i].vx -= px / mtot;
        s.atoms[i].vy -= py / mtot;
        s.atoms[i].vz -= pz / mtot;
    }
    s.sync_species();
    s.adopt_table_timescale();
    s.thermostat_on = true;
    s.target_temperature = temp;
    s.rebase();
    s
}

pub fn hbonds_now(s: &Sim) -> usize {
    let z: Vec<u32> = (0..s.n).map(|i| s.atoms[i].species.z).collect();
    let pos: Vec<[f64; 3]> = (0..s.n).map(|i| [s.atoms[i].x, s.atoms[i].y, s.atoms[i].z]).collect();
    holon_lens::lens::hbonds(&pos, &z).map(|v| v.len()).unwrap_or(0)
}

