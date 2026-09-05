//! FIELD-4 G-C1 and plant (i) (`conformance/water_observatory/FIELD4_PREREG.md` §2, §5):
//! with the harvested coefficients loaded, the engine's seam-law interaction on each linear
//! node equals `E_q + p_HO + wall + disp` from the formulas to `1e-10` hartree; with
//! `P → −P` the 2.9 Å node must miss by `2·|p_HO(2.9 Å)|`.
//!
//! ```text
//! cargo run --release -p holon-render --example field4_check -- [OUT_DIR]
//! ```
//!
//! Reads `OUT_DIR/wall4.json` (the harvest) and FIELD-3's node records for `E_q`; writes
//! `OUT_DIR/g_c1.json`.
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::{water_dimer_linear, Fragment, ANGSTROM_TO_BOHR};
use holon_render::seam::{SeamModel, SeamPlant};
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::{Path, PathBuf};

#[path = "../tests/common/quartet.rs"]
#[allow(dead_code)]
mod quartet;

const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;
const NODES_ANGSTROM: [f64; 6] = [2.5, 2.7, 2.9, 3.1, 3.4, 3.7];
const TOL: f64 = 1e-10;

fn json_num(t: &str, key: &str) -> f64 {
    t.split(&format!("\"{key}\": ")).nth(1).and_then(|x| x.split(|c| c == ',' || c == '\n' || c == '}').next()).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::NAN)
}

fn linear(o: Species, h: Species, r_oo_angstrom: f64) -> (Fragment, Fragment) {
    water_dimer_linear(o, h, H2O_R, H2O_THETA, r_oo_angstrom * ANGSTROM_TO_BOHR)
}

fn engine_dimer(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, plant: SeamPlant) -> Box<Sim> {
    let mut species = a.species.clone();
    species.extend_from_slice(&b.species);
    let pos: Vec<[f64; 3]> = a.centers.iter().chain(b.centers.iter()).map(|c| [c[0] + 15.0, c[1] + 15.0, c[2] + 10.0]).collect();
    let mut s = quartet::scene(&species, &pos, false);
    s.dims = Dims::Three;
    s.boundary = Boundary::Open;
    s.width = 80.0;
    s.height = 30.0;
    s.depth = 30.0;
    s.sync_species();
    s.adopt_table_timescale();
    s.rebase();
    s.set_field(true, None).expect("open box admits the field");
    s.seam_plant = plant;
    s.set_seam(seam).expect("no acuity frame");
    s.refresh_pairs();
    s.compute_forces();
    s
}

/// The engine's seam-law interaction `E(geometry) − E(acceptor moved 40 bohr along x)` on the
/// rows the seam serves between units, and its field and seam parts.
fn engine_interaction(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, plant: SeamPlant) -> (f64, f64, f64) {
    let s = engine_dimer(a, b, seam, plant);
    let near = (s.e_pair + s.e_three) + s.e_field + s.e_seam;
    let far_b = b.translated([40.0, 0.0, 0.0]);
    let f = engine_dimer(a, &far_b, seam, plant);
    let far = (f.e_pair + f.e_three) + f.e_field + f.e_seam;
    (near - far, s.e_field - f.e_field, s.e_seam - f.e_seam)
}

/// The formulas: the penetration term over the four cross-unit H–O pairs, the wall and the
/// dispersion on the O–O pair.
fn formula_terms(a: &Fragment, b: &Fragment, m: &SeamModel) -> (f64, f64, f64) {
    let d = |x: [f64; 3], y: [f64; 3]| ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2) + (x[2] - y[2]).powi(2)).sqrt();
    let r_oo = d(a.centers[0], b.centers[0]);
    let mut pen = 0.0;
    for h in 1..3 {
        pen += m.penetration(d(a.centers[h], b.centers[0]));
        pen += m.penetration(d(b.centers[h], a.centers[0]));
    }
    (pen, m.wall(r_oo), m.dispersion(r_oo))
}

fn main() {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "../conformance/water_observatory/field4".to_string()));
    let t = fs::read_to_string(out.join("wall4.json")).expect("wall4.json: run field4_harvest density first");
    let m = SeamModel { a: json_num(&t, "a"), b: json_num(&t, "b"), p: json_num(&t, "p"), c: json_num(&t, "c"), c6: json_num(&t, "c6"), ..SeamModel::NO_WALL };
    eprintln!("harvest: A {:.6e} b {:.6} P {:.6e} c {:.6} C6 {:.6e}", m.a, m.b, m.p, m.c, m.c6);
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let field3: &Path = Path::new("../conformance/water_observatory/field3");
    let mut worst = 0.0f64;
    let mut worst_corrected = 0.0f64;
    let mut lines = Vec::new();
    let mut plant = (f64::NAN, f64::NAN, false);
    for &r in &NODES_ANGSTROM {
        let (a, b) = linear(o, h, r);
        let node = fs::read_to_string(field3.join(format!("linear_R{r:.1}.json"))).expect("FIELD-3 node");
        let e_q_record = {
            // the engine's field on this geometry as FIELD-3 recorded it (wall.json's nodes)
            let w = fs::read_to_string(field3.join("wall.json")).expect("FIELD-3 wall.json");
            let key = format!("\"r_angstrom\": {r:.1},");
            let seg = w.split(&key).nth(1).unwrap_or("");
            json_num(seg, "e_field")
        };
        let _ = node;
        let (e_int, e_f, e_s) = engine_interaction(&a, &b, Some(m), SeamPlant::None);
        let (pen, wall, disp) = formula_terms(&a, &b, &m);
        let want = e_q_record + pen + wall + disp;
        let miss = (e_int - want).abs();
        // the reference convention: FIELD-3's `e_field` of record is the RAW field on the
        // geometry, the engine's interaction is the DIFFERENCE against the acceptor at 40 bohr,
        // where the field is not zero (a dipole pair at 40 bohr); the corrected miss subtracts
        // the engine's own far field, read here
        let far_field = {
            let far_b = b.translated([40.0, 0.0, 0.0]);
            let f = engine_dimer(&a, &far_b, Some(m), SeamPlant::None);
            f.e_field
        };
        let miss_corrected = (e_int - (want - far_field)).abs();
        worst_corrected = worst_corrected.max(miss_corrected);
        eprintln!("    reference: the engine's field at 40 bohr {far_field:+.6e}; miss after subtracting it {miss_corrected:.3e}");
        worst = worst.max(miss);
        eprintln!("  R {r:.1} Å: engine {e_int:+.12e} (field {e_f:+.6e}, seam {e_s:+.6e}) vs formula {want:+.12e} (E_q {e_q_record:+.6e} + pen {pen:+.6e} + wall {wall:+.6e} + disp {disp:+.6e}) — miss {miss:.3e}");
        lines.push(format!("{{\"r_angstrom\": {r:.1}, \"engine_interaction\": {e_int:+.12e}, \"formula\": {want:+.12e}, \"miss\": {miss:.3e}, \"e_q\": {e_q_record:+.12e}, \"pen\": {pen:+.12e}, \"wall\": {wall:+.12e}, \"disp\": {disp:+.12e}}}"));
        if (r - 2.9).abs() < 1e-9 {
            let (e_pl, _, _) = engine_interaction(&a, &b, Some(m), SeamPlant::FlipPenetration);
            let observed = (e_pl - e_int).abs();
            let expected = 2.0 * pen.abs();
            let carrier = pen.abs() >= 1e-4;
            let fires = carrier && (observed - expected).abs() <= TOL;
            plant = (observed, expected, fires);
            eprintln!("plant (i) at 2.9 Å: miss {observed:.6e} vs 2·|p_HO| {expected:.6e}; carrier |p_HO| {:.3e} ≥ 1e-4: {carrier} → {}", pen.abs(), if fires { "FIRES" } else { "does not fire" });
        }
    }
    let pass = worst <= TOL;
    eprintln!("G-C1: worst |engine − formula| = {worst:.3e} (stake {TOL:.0e}) → {}; after subtracting the engine's own field at the 40-bohr reference: {worst_corrected:.3e}", if pass { "PASS" } else { "FAIL" });
    fs::write(out.join("g_c1.json"), format!("{{\n  \"g_c1_worst_miss\": {worst:.3e}, \"g_c1_pass\": {pass}, \"g_c1_worst_miss_reference_corrected\": {worst_corrected:.3e},\n  \"plant_i\": {{\"miss_observed\": {:.6e}, \"miss_expected\": {:.6e}, \"fires\": {}}},\n  \"nodes\": [\n    {}\n  ]\n}}\n", plant.0, plant.1, plant.2, lines.join(",\n    "))).unwrap();
    println!("done");
}
