//! EWALD-1 diagnostic: the ledger's drift under a wrapping boundary, with and without the
//! field, on FIELD-1's 17×17×10 scene (which the engine's own image rule refuses) and on a
//! 32-bohr cube that honours it.
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::embed::water_centers;
use holon_render::field::{WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD};
use holon_render::sim::{Boundary, Dims, Sim};
#[path = "../tests/common/channel_scenes.rs"]
#[allow(dead_code)]
mod channel_scenes;

fn cube(edge: f64) -> Box<Sim> {
    let mono = water_centers(WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD);
    let c = edge / 2.0;
    let oxygens = [[c - 3.5, c - 3.5, c], [c + 3.5, c - 3.5, c], [c - 3.5, c + 3.5, c], [c + 3.5, c + 3.5, c]];
    let mut species = Vec::new();
    let mut pos = Vec::new();
    for (k, o) in oxygens.iter().enumerate() {
        let flip = if k % 2 == 0 { 1.0 } else { -1.0 };
        for (m, cc) in mono.iter().enumerate() {
            species.push(if m == 0 { OXYGEN } else { HYDROGEN });
            pos.push([o[0] + flip * cc[0], o[1] + cc[2], o[2] + cc[1]]);
        }
    }
    let mut s = channel_scenes::quartet::scene(&species, &pos, false);
    s.dims = Dims::Three;
    s.width = edge;
    s.height = edge;
    s.depth = edge;
    s.set_boundary(Boundary::Periodic).expect("a legal periodic cell");
    let mut st: u64 = 0x4649_454c;
    let mut lcg = || {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((st >> 11) as f64) / ((1u64 << 53) as f64)
    };
    let n = s.n;
    let (mut px, mut py, mut pz) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let scale = if s.atoms[i].species.z == 8 { 2.5e-4 } else { 1.0e-3 };
        s.atoms[i].vx = scale * (2.0 * lcg() - 1.0);
        s.atoms[i].vy = scale * (2.0 * lcg() - 1.0);
        s.atoms[i].vz = scale * (2.0 * lcg() - 1.0);
        let m = s.atoms[i].mass();
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
    s.rebase();
    s
}

fn run_and_report(name: &str, mut s: Box<Sim>, field: bool) {
    if field {
        s.set_field(true, None).unwrap();
    }
    let mut peak = 0.0f64;
    let l0 = { s.compute_forces(); s.ledger() };
    for _ in 0..125 {
        s.step_frame(channel_scenes::SUBSTEPS);
        peak = peak.max((s.ledger() - l0).abs());
    }
    eprintln!("{name}: field {}: drift peak (own) {peak:.3e}, engine drift_peak {:.3e}, e_field {:+.4e}, transition {:+.3e}, momentum residual {:.2e} / bound {:.2e}, cols {}",
        if field { "ON " } else { "OFF" }, s.drift_peak, s.e_field, s.work.field, s.momentum_residual(), s.momentum_bound(), s.work_columns_ok());
}

fn main() {
    let s = channel_scenes::four_waters(Boundary::Periodic);
    eprintln!("FIELD-1 scene 17x17x10: three-body reach {:.1} vs half-edge 5.0 — set_boundary(Periodic) would say: {:?}", s.three_body_cutoff(), { let mut t = channel_scenes::four_waters(Boundary::Walls); t.set_boundary(Boundary::Periodic).err() });
    run_and_report("17x17x10", channel_scenes::four_waters(Boundary::Periodic), false);
    run_and_report("17x17x10", channel_scenes::four_waters(Boundary::Periodic), true);
    run_and_report("42-cube  ", cube(42.0), false);
    run_and_report("42-cube  ", cube(42.0), true);
}
