//! FIELD-8's harvest (`conformance/water_observatory/FIELD8_PREREG.md` §0, §2, §5, §6): DATA
//! AT CONTACT. FIELD-7's wall and contact term were fit at `r ≥ 3.4` bohr and then summed to an
//! attraction all the way in, so the pair fused while the lens counted a bond
//! (M-EXTRAPOLATED-HOLE). This freeze puts readings where the miss was — the CLOSE tilt family
//! and the TWIST family in the cheap exchange harvest, two CLOSE exact nodes on the line — fits
//! the contact term on TWO classes over twelve exact geometries, and, before any arm runs, asks
//! the engine's own `hole()` whether the harvested law still falls inward.
//!
//! FIELD-7's runner with five changes and nothing else: the reading set is 66 (its 24 REUSED as
//! records, plus 18 close tilts and 24 twists); the exact record is twelve (its nine plus the two
//! close linear nodes solved here and its own twisted node); the contact term carries a SECOND
//! class, `−P_HH·Σ_{H–H} e^{−c_HH r}`, fit jointly on a `176²` grid; `G-N0` walks the summed
//! cross-unit class potentials in from `3.0` bohr and VOIDS the arms if one falls; and the
//! held-out geometry is a BENT DONOR, a kind no fit point contains (§4, M-UNTESTED-GAP).
//!
//! ```text
//! cargo run --release -p holon-render --example field8_harvest -- close   [OUT_DIR]
//! cargo run --release -p holon-render --example field8_harvest -- fit     [OUT_DIR]
//! cargo run --release -p holon-render --example field8_harvest -- predict [OUT_DIR]
//! ```
//!
//! `close` (detached): the 42 NEW undeformed readings, then the two CLOSE exact nodes.
//! `fit`: W0, the three-class wall over 66 (S1), plant (ii), the two-class contact term over the
//! twelve exact points (C1), dispersion, G-N0, G-C1 and plant (i), `wall8.json`, and
//! `prediction.json` for the bent donor written BEFORE that node is solved. `predict` (detached):
//! refuses without `prediction.json`, solves the bent-donor node exactly, reads the undeformed
//! referee on it against the harvested wall, and writes `prediction_check.json` (S2).
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::{solve_embedded, supermolecule, water_dimer_linear, Fragment, ANGSTROM_TO_BOHR};
use holon_chem::fci::SolveExit;
use holon_chem::heitler_london::{heitler_london_undeformed, HlReading};
use holon_render::seam::{SeamModel, SeamPlant};
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/quartet.rs"]
#[allow(dead_code)]
mod quartet;

/// EMBED-1's water pins — the same numbers FIELD-3's … FIELD-7's runners carry.
const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;

/// The six acceptor tilts, every family (§0).
const TILT_DEG: [f64; 6] = [0.0, 30.0, 60.0, 90.0, 120.0, 180.0];
/// The TILT family's separations, shortest first: FIELD-8's three CLOSE ones and FIELD-7's four.
const TILT_R: [f64; 7] = [2.1, 2.3, 2.5, 2.7, 2.9, 3.1, 3.4];
/// The separations whose tilt readings are NEW here (the CLOSE tilt family).
const CLOSE_R: [f64; 3] = [2.1, 2.3, 2.5];
/// The TWIST family's separations: the acceptor turned 90° about the O···O axis, then tilted.
const TWIST_R: [f64; 4] = [2.3, 2.7, 3.0, 3.4];
/// The twist applied about the O···O axis before every twist-family tilt.
const TWIST_ABOUT_Z_DEG: f64 = 90.0;

const N_T: usize = TILT_DEG.len();
const N_TILT: usize = TILT_R.len() * N_T; // 42
const N_TWIST: usize = TWIST_R.len() * N_T; // 24
const N_READ: usize = N_TILT + N_TWIST; // 66
/// The readings this runner MEASURES (the rest are FIELD-7's, reused as records).
const N_NEW: usize = CLOSE_R.len() * N_T + N_TWIST; // 42

/// The eight LINEAR exact nodes (Å), SHORTEST FIRST. The first two are solved by `close`.
const LINEAR_ANGSTROM: [f64; 8] = [2.1, 2.3, 2.5, 2.7, 2.9, 3.1, 3.4, 3.7];
/// The two CLOSE exact nodes, in the order `close` solves them (the dearer one first).
const CLOSE_NODES: [f64; 2] = [2.3, 2.1];
/// A linear node at or beyond this separation is one of FIELD-6's four OUTER dispersion nodes.
const OUTER_FROM_ANGSTROM: f64 = 2.9;
/// The node plant (i) is read at.
const REF_ANGSTROM: f64 = 2.9;

/// The four non-linear exact geometries of record (§0).
const TILT5_ANGSTROM: f64 = 2.9;
const TILT5_DEGREES: f64 = 30.0;
const TILT6_ANGSTROM: f64 = 3.1;
const TILT6_DEGREES: f64 = 45.0;
const FLIPPED_ANGSTROM: f64 = 3.4;
const TWISTED_ANGSTROM: f64 = 3.0;
const TWISTED_TWIST_DEGREES: f64 = 90.0;
const TWISTED_TILT_DEGREES: f64 = 60.0;

/// S2's held-out geometry: the linear dimer at 2.9 Å with the DONOR rotated 30° about the
/// x-axis through ITS OWN oxygen. The acceptor is untouched and `R_OO` is unchanged.
const BENT_ANGSTROM: f64 = 2.9;
const BENT_DEGREES: f64 = 30.0;

/// The separation at which the acceptor is "away" (bohr): the engine's reference on both sides
/// of G-C1 and of every `E_q` difference.
const FAR_BOHR: f64 = 40.0;

/// The residual bar every exact solve must meet (EMBED-3's).
const RESIDUAL_BAR: f64 = 1e-9;
/// The reading floor on every harvested reading (M-FLOOR-UNSTAKED).
const R_FLOOR: f64 = 1e-6;

/// The three-exponent wall grid (per bohr): `0.50 ..= 4.00` step `0.05` — 71 values, 71³ triples.
const NB: usize = 71;
/// The contact term's `c`-grid, per class (per bohr): `0.50 ..= 4.00` step `0.02` — 176 values.
const NC: usize = 176;

/// S1's tolerance, per geometry: `max(0.05·E_exch, 1e-4)` hartree (FIELD-6's derived rule).
const WALL_TOL_FRAC: f64 = 0.05;
const WALL_TOL_ABS: f64 = 1e-4;
/// S1 (b)'s floor: at least this many of the 66 within tolerance (80 %), else (c) VOID.
const S1_B_MIN: usize = 53;

/// C1's tolerance, per exact point: `max(0.25·|ΔE_exact|, 5e-4)`; at least ten of twelve, and
/// BOTH close nodes within it.
const C1_FRAC: f64 = 0.25;
const C1_ABS: f64 = 5e-4;
const C1_MIN: usize = 10;

/// S2's tolerance: `max(0.25·|ΔE_exact|, 5e-4)`.
const PRED_FRAC: f64 = 0.25;
const PRED_ABS: f64 = 5e-4;

/// The band the remainder's log-log slope must lie in for `C₆` to transfer (FIELD-6's rule).
const SLOPE_LO: f64 = -8.0;
const SLOPE_HI: f64 = -4.0;

/// W0: the undeformed product's overlap window.
const NORM_LO: f64 = 0.8;
const NORM_HI: f64 = 1.0;
/// The determinant count FIELD-3's supermolecule carries (EXACT).
const N_DET_DIMER: usize = 1_002_001;
/// The count of monomer-product determinants the undeformed state is BUILT from.
const PRODUCT_DETS_STAKED: usize = 194_481;

/// G-C1's tolerance, and plant (i)'s carrier.
const G_C1_TOL: f64 = 1e-10;
const PLANT_I_CARRIER: f64 = 1e-4;
/// Plant (ii): FIELD-7's far-data wall must MISS at least this many of the 18 CLOSE readings, by
/// more than `0.05·E_exch` (§5 (ii)'s own rule — a pure fraction, no absolute floor); its carrier
/// is exchange at contact.
const PLANT_II_MIN_FAIL: usize = 9;
const PLANT_II_TOL_FRAC: f64 = 0.05;
const PLANT_II_CARRIER_FLOOR: f64 = 0.1;
const PLANT_II_CARRIER_R: f64 = 2.1;
const PLANT_II_CARRIER_TILT: f64 = 0.0;

/// M-CHEAPER-THAN-ITS-PRICE: FIELD-7 measured 55–105 core-seconds per undeformed reading. A
/// reading under a TENTH of 55 is recorded as under its price.
const HL_PRICE_LO_CORE_S: f64 = 55.0;
const HL_PRICE_HI_CORE_S: f64 = 105.0;
const HL_PRICE_TENTH_CORE_S: f64 = 5.5;
/// Every exact solve's price band (§2), the close nodes included.
const S2_CPU_LO: f64 = 1450.0;
const S2_CPU_HI: f64 = 57600.0;

fn cpu_seconds() -> f64 {
    let s = fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let tail = &s[s.rfind(')').map(|i| i + 2).unwrap_or(0)..];
    let f: Vec<&str> = tail.split_whitespace().collect();
    let ut: f64 = f.get(11).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    let st: f64 = f.get(12).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    (ut + st) / 100.0
}

fn threads() -> usize {
    std::env::var("LANE_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1))
}

fn json_num(t: &str, key: &str) -> f64 {
    t.split(&format!("\"{key}\": ")).nth(1).and_then(|x| x.split(|c| c == ',' || c == '\n' || c == '}').next()).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::NAN)
}
fn json_str(t: &str, key: &str) -> String {
    t.split(&format!("\"{key}\": \"")).nth(1).and_then(|x| x.split('"').next()).unwrap_or("").to_string()
}

/// The `[[x, y, z], …]` list under `key` in one of the record files (printed at `{:.10}`).
fn json_centers(t: &str, key: &str) -> Vec<[f64; 3]> {
    let Some(rest) = t.split(&format!("\"{key}\": [")).nth(1) else {
        return Vec::new();
    };
    let Some(end) = rest.find("]]") else {
        return Vec::new();
    };
    let body = &rest[..end + 1];
    let nums: Vec<f64> = body.split(|c: char| c == '[' || c == ']' || c == ',').filter_map(|x| x.trim().parse::<f64>().ok()).collect();
    nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
}

// ------------------------------------------------------------------------ the geometries

/// FIELD-3's `linear` verbatim.
fn linear(o: Species, h: Species, r_oo_angstrom: f64) -> (Fragment, Fragment) {
    water_dimer_linear(o, h, H2O_R, H2O_THETA, r_oo_angstrom * ANGSTROM_TO_BOHR)
}

/// FIELD-3's FLIPPED dimer verbatim: the linear donor, the acceptor rotated by π about the
/// x-axis through its oxygen. FIELD-4 solved it exactly at 3.4 Å.
fn flipped(o: Species, h: Species, r_oo_angstrom: f64) -> (Fragment, Fragment) {
    let (donor, _) = linear(o, h, r_oo_angstrom);
    let (s, c) = ((0.5 * H2O_THETA).sin(), (0.5 * H2O_THETA).cos());
    let r = H2O_R;
    let acc = Fragment::new(
        vec![o, h, h],
        vec![[0.0; 3], [r * s, 0.0, -r * c], [-r * s, 0.0, -r * c]],
        vec![-2.0, 1.0, 1.0],
    )
    .translated([0.0, 0.0, r_oo_angstrom * ANGSTROM_TO_BOHR]);
    (donor, acc)
}

/// FIELD-5's `tilted` verbatim: the linear dimer with the ACCEPTOR rotated by `theta_degrees`
/// about the x-axis through its OWN oxygen. The donor is untouched and `R_OO` is unchanged.
fn tilted(o: Species, h: Species, r_oo_angstrom: f64, theta_degrees: f64) -> (Fragment, Fragment) {
    let (donor, acc) = linear(o, h, r_oo_angstrom);
    let oi = acc.species.iter().position(|s| s.z == 8).expect("an acceptor oxygen");
    let origin = acc.centers[oi];
    let th = theta_degrees * std::f64::consts::PI / 180.0;
    let (s, c) = (th.sin(), th.cos());
    let centers: Vec<[f64; 3]> = acc
        .centers
        .iter()
        .map(|p| {
            let (x, y, z) = (p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]);
            [origin[0] + x, origin[1] + y * c - z * s, origin[2] + y * s + z * c]
        })
        .collect();
    (donor, Fragment::new(acc.species.clone(), centers, acc.weights.clone()))
}

/// FIELD-7's `twisted` verbatim, generalised to any tilt: the linear dimer with the acceptor
/// rotated by `twist_degrees` about the O···O axis (z) through its own oxygen, and THEN tilted by
/// `tilt_degrees` about its own x-axis. Both rotations fix the acceptor's oxygen, so `R_OO` is
/// unchanged.
fn twisted(o: Species, h: Species, r_oo_angstrom: f64, twist_degrees: f64, tilt_degrees: f64) -> (Fragment, Fragment) {
    let (donor, acc) = linear(o, h, r_oo_angstrom);
    let oi = acc.species.iter().position(|s| s.z == 8).expect("an acceptor oxygen");
    let origin = acc.centers[oi];
    let tw = twist_degrees * std::f64::consts::PI / 180.0;
    let ti = tilt_degrees * std::f64::consts::PI / 180.0;
    let (sw, cw) = (tw.sin(), tw.cos());
    let (st, ct) = (ti.sin(), ti.cos());
    let centers: Vec<[f64; 3]> = acc
        .centers
        .iter()
        .map(|p| {
            let (x, y, z) = (p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]);
            // the twist about z first
            let (x1, y1, z1) = (x * cw - y * sw, x * sw + y * cw, z);
            // then the tilt about the acceptor's own x-axis
            let (x2, y2, z2) = (x1, y1 * ct - z1 * st, y1 * st + z1 * ct);
            [origin[0] + x2, origin[1] + y2, origin[2] + z2]
        })
        .collect();
    (donor, Fragment::new(acc.species.clone(), centers, acc.weights.clone()))
}

/// S2's held-out geometry: the linear dimer with the DONOR rotated by `theta_degrees` about the
/// x-axis through ITS OWN oxygen — its O–H swung off the O···O axis. The ACCEPTOR is untouched;
/// the pivot is the donor's oxygen, so `R_OO` is unchanged. A kind no fit point contains.
fn bent_donor(o: Species, h: Species, r_oo_angstrom: f64, theta_degrees: f64) -> (Fragment, Fragment) {
    let (donor, acc) = linear(o, h, r_oo_angstrom);
    let oi = donor.species.iter().position(|s| s.z == 8).expect("a donor oxygen");
    let origin = donor.centers[oi];
    let th = theta_degrees * std::f64::consts::PI / 180.0;
    let (s, c) = (th.sin(), th.cos());
    let centers: Vec<[f64; 3]> = donor
        .centers
        .iter()
        .map(|p| {
            let (x, y, z) = (p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]);
            [origin[0] + x, origin[1] + y * c - z * s, origin[2] + y * s + z * c]
        })
        .collect();
    (Fragment::new(donor.species.clone(), centers, donor.weights.clone()), acc)
}

fn centers_json(f: &Fragment) -> String {
    f.centers.iter().map(|c| format!("[{:.10}, {:.10}, {:.10}]", c[0], c[1], c[2])).collect::<Vec<_>>().join(", ")
}

fn exit_name(e: &SolveExit) -> String {
    format!("{e:?}")
}

fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Every CROSS-UNIT pair distance (bohr) by class: `(O–O, H–O, H–H)`. For a water dimer that is
/// one, four and four — the same enumeration the engine's seam loop makes.
fn cross_classes(a: &Fragment, b: &Fragment) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let (mut oo, mut ho, mut hh) = (Vec::new(), Vec::new(), Vec::new());
    for (sa, ca) in a.species.iter().zip(a.centers.iter()) {
        for (sb, cb) in b.species.iter().zip(b.centers.iter()) {
            let d = dist(ca, cb);
            match (sa.z, sb.z) {
                (8, 8) => oo.push(d),
                (8, 1) | (1, 8) => ho.push(d),
                (1, 1) => hh.push(d),
                _ => {}
            }
        }
    }
    (oo, ho, hh)
}

/// The cross-unit O–O distance (bohr).
fn cross_oo(a: &Fragment, b: &Fragment) -> f64 {
    let ca = a.centers[a.species.iter().position(|s| s.z == 8).expect("an oxygen")];
    let cb = b.centers[b.species.iter().position(|s| s.z == 8).expect("an oxygen")];
    dist(&ca, &cb)
}

fn list_json(v: &[f64]) -> String {
    v.iter().map(|d| format!("{d:.6}")).collect::<Vec<_>>().join(", ")
}

fn sum_exp(rs: &[f64], b: f64) -> f64 {
    rs.iter().map(|&r| (-b * r).exp()).sum()
}

// ------------------------------------------------------------------------------ the engine

/// FIELD-4's `engine_dimer` verbatim: an open box, the field on with the pin charge, the seam
/// model and its plant installed, forces computed once so the closure assignment and the rows are
/// read.
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

/// `E(geometry) − E(acceptor moved 40 bohr along x)` on the rows the seam law serves between
/// units: the total, the FIELD part, the SEAM part. The FIELD part is exactly the freeze's
/// `E_q(g) − E_q(40)` — the SAME reference the formula side of G-C1 uses.
fn engine_interaction(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, plant: SeamPlant) -> (f64, f64, f64) {
    let s = engine_dimer(a, b, seam, plant);
    let near = (s.e_pair + s.e_three) + s.e_field + s.e_seam;
    let far_b = b.translated([FAR_BOHR, 0.0, 0.0]);
    let f = engine_dimer(a, &far_b, seam, plant);
    let far = (f.e_pair + f.e_three) + f.e_field + f.e_seam;
    (near - far, s.e_field - f.e_field, s.e_seam - f.e_seam)
}

/// The formula side of G-C1, class by class: the H–O contact term, the H–H contact term (FIELD-8),
/// the O–O wall, the dispersion, and FIELD-7's two other walls.
struct Terms {
    pen_ho: f64,
    pen_hh: f64,
    w_oo: f64,
    w_oh: f64,
    w_hh: f64,
    disp: f64,
}

impl Terms {
    fn total(&self) -> f64 {
        self.pen_ho + self.pen_hh + self.w_oo + self.w_oh + self.w_hh + self.disp
    }
    fn wall_total(&self) -> f64 {
        self.w_oo + self.w_oh + self.w_hh
    }
}

fn formula_terms(a: &Fragment, b: &Fragment, m: &SeamModel) -> Terms {
    let (oo, ho, hh) = cross_classes(a, b);
    Terms {
        pen_ho: ho.iter().map(|&r| m.penetration(r)).sum(),
        // the FIELD-8 class, written here in the freeze's own words rather than borrowed
        pen_hh: hh.iter().map(|&r| -m.p_hh * (-m.c_hh * r).exp()).sum(),
        w_oo: oo.iter().map(|&r| m.wall(r)).sum(),
        w_oh: ho.iter().map(|&r| m.wall_oh(r)).sum(),
        w_hh: hh.iter().map(|&r| m.wall_hh(r)).sum(),
        disp: oo.iter().map(|&r| m.dispersion(r)).sum(),
    }
}

// ------------------------------------------------------------------------- the exact solve

/// One exact node: the supermolecule, the monomer references, the record. FIELD-3's `solve_node`
/// with the price band recorded on every node (§2): `admitted` is the record's own verdict on
/// whether the solve was bought at the freeze's price.
fn solve_node(out: &Path, name: &str, r_oo_angstrom: f64, a: &Fragment, b: &Fragment, price: bool) -> bool {
    let path = out.join(format!("{name}.json"));
    if path.exists() {
        eprintln!("  {name}: exists, skipped");
        return true;
    }
    let t0 = Instant::now();
    let c0 = cpu_seconds();
    let e_a0 = solve_embedded(&a.species, &a.centers, &[]);
    let e_b0 = solve_embedded(&b.species, &b.centers, &[]);
    let sm = supermolecule(a, b);
    let wall = t0.elapsed().as_secs_f64();
    let cpu = cpu_seconds() - c0;
    let de = sm.e_total - e_a0.e_total - e_b0.e_total;
    let converged = matches!(sm.sol.exit, SolveExit::Converged) && sm.sol.residual <= RESIDUAL_BAR;
    let admitted = cpu >= S2_CPU_LO && cpu <= S2_CPU_HI;
    fs::write(
        &path,
        format!(
            "{{\n  \"node\": \"{name}\", \"r_oo_angstrom\": {r_oo_angstrom:.3}, \"r_oo_bohr\": {:.6},\n  \"n_det\": {}, \"e_super\": {:.12e}, \"e_a0\": {:.12e}, \"e_b0\": {:.12e}, \"de_exact\": {:.12e},\n  \"davidson_iters\": {}, \"residual\": {:.3e}, \"exit\": \"{}\", \"converged\": {converged},\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}],\n  \"wall_seconds\": {wall:.1}, \"cpu_seconds\": {cpu:.1}, \"cpu_seconds_lo\": {S2_CPU_LO}, \"cpu_seconds_hi\": {S2_CPU_HI}, \"admitted\": {admitted},\n  \"threads\": {}, \"price_node\": {price}\n}}\n",
            cross_oo(a, b),
            sm.gp.space.n_det,
            sm.e_total,
            e_a0.e_total,
            e_b0.e_total,
            de,
            sm.sol.davidson_iters,
            sm.sol.residual,
            exit_name(&sm.sol.exit),
            centers_json(a),
            centers_json(b),
            threads(),
        ),
    )
    .unwrap();
    eprintln!(
        "  {name}: R_OO {r_oo_angstrom:.1} Å, {} dets, ΔE_exact {de:+.6e} Ha, {} iters, residual {:.1e}, exit {}, wall {wall:.0} s, {cpu:.0} core-s, admitted {admitted} (band {S2_CPU_LO}–{S2_CPU_HI})",
        sm.gp.space.n_det,
        sm.sol.davidson_iters,
        sm.sol.residual,
        exit_name(&sm.sol.exit),
    );
    converged
}

// ---------------------------------------------------------------------- the frozen records

fn sibling(out: &Path, name: &str) -> PathBuf {
    let sib = out.parent().unwrap_or(Path::new(".")).join(name);
    if sib.exists() {
        sib
    } else {
        PathBuf::from(format!("../conformance/water_observatory/{name}"))
    }
}

// ---------------------------------------------------------------- the non-negative fits

/// Solve the weighted normal equations on the ACTIVE classes only (`n ≤ 3`), by Gaussian
/// elimination with partial pivoting. `None` when the active block is singular. Allocation-free.
fn solve_active(a: &[[f64; 3]; 3], v: &[f64; 3], active: [bool; 3]) -> Option<[f64; 3]> {
    let mut idx = [0usize; 3];
    let mut n = 0usize;
    for (c, on) in active.iter().enumerate() {
        if *on {
            idx[n] = c;
            n += 1;
        }
    }
    if n == 0 {
        return Some([0.0; 3]);
    }
    let mut m = [[0.0f64; 4]; 3];
    for r in 0..n {
        for c in 0..n {
            m[r][c] = a[idx[r]][idx[c]];
        }
        m[r][3] = v[idx[r]];
    }
    // the right-hand side lives in column 3 whatever `n` is
    for k in 0..n {
        let mut p = k;
        for r in (k + 1)..n {
            if m[r][k].abs() > m[p][k].abs() {
                p = r;
            }
        }
        if !(m[p][k].abs() > 0.0) {
            return None;
        }
        m.swap(k, p);
        let piv = m[k][k];
        for r in (k + 1)..n {
            let f = m[r][k] / piv;
            if f == 0.0 {
                continue;
            }
            for c in k..n {
                let d = f * m[k][c];
                m[r][c] -= d;
            }
            let d = f * m[k][3];
            m[r][3] -= d;
        }
    }
    let mut sol = [0.0f64; 3];
    for k in (0..n).rev() {
        let mut s = m[k][3];
        for c in (k + 1)..n {
            s -= m[k][c] * sol[c];
        }
        let x = s / m[k][k];
        if !x.is_finite() {
            return None;
        }
        sol[k] = x;
    }
    let mut out = [0.0f64; 3];
    for r in 0..n {
        out[idx[r]] = sol[r];
    }
    Some(out)
}

/// §0's non-negativity rule: the weighted least-squares amplitudes; if any is negative, DROP that
/// class (amplitude an exact `0.0`) and refit the rest, repeating until all are non-negative. When
/// several are negative at once the MOST negative is dropped first — the rule applied to one class
/// at a time, which terminates in at most three rounds.
/// Returns `(amplitudes, classes kept, weighted residual)`.
fn fit_nonneg(a: &[[f64; 3]; 3], v: &[f64; 3], syy: f64) -> ([f64; 3], [bool; 3], f64) {
    let mut active = [true; 3];
    for c in 0..3 {
        if !(a[c][c] > 0.0) {
            active[c] = false; // a class with no pairs, or an underflowed one
        }
    }
    loop {
        match solve_active(a, v, active) {
            None => {
                let mut dropped = false;
                for c in (0..3).rev() {
                    if active[c] {
                        active[c] = false;
                        dropped = true;
                        break;
                    }
                }
                if !dropped {
                    return ([0.0; 3], [false; 3], syy);
                }
            }
            Some(x) => {
                let (mut worst, mut wv) = (usize::MAX, 0.0f64);
                for c in 0..3 {
                    if active[c] && x[c] < 0.0 && x[c] < wv {
                        wv = x[c];
                        worst = c;
                    }
                }
                if worst == usize::MAX {
                    let mut r = syy;
                    for c in 0..3 {
                        r -= 2.0 * x[c] * v[c];
                        for d in 0..3 {
                            r += x[c] * a[c][d] * x[d];
                        }
                    }
                    return (x, active, r.max(0.0));
                }
                active[worst] = false;
            }
        }
    }
}

/// The same rule on TWO classes (the contact term's `P_HO`, `P_HH`): the weighted least-squares
/// amplitudes, the most negative dropped to an exact `0.0` and the rest refit.
/// Returns `(amplitudes, classes kept, weighted residual)`.
fn fit_nonneg2(a: [[f64; 2]; 2], v: [f64; 2], syy: f64) -> ([f64; 2], [bool; 2], f64) {
    let mut active = [a[0][0] > 0.0, a[1][1] > 0.0];
    loop {
        let x = match (active[0], active[1]) {
            (false, false) => Some([0.0f64, 0.0f64]),
            (true, false) => Some([v[0] / a[0][0], 0.0]),
            (false, true) => Some([0.0, v[1] / a[1][1]]),
            (true, true) => {
                let det = a[0][0] * a[1][1] - a[0][1] * a[0][1];
                if det.abs() > 0.0 {
                    Some([(v[0] * a[1][1] - v[1] * a[0][1]) / det, (a[0][0] * v[1] - a[0][1] * v[0]) / det])
                } else {
                    None
                }
            }
        };
        match x {
            None => {
                // singular: drop the second class and refit the first alone
                active[1] = false;
            }
            Some(x) if !x[0].is_finite() || !x[1].is_finite() => {
                if active[1] {
                    active[1] = false;
                } else if active[0] {
                    active[0] = false;
                } else {
                    return ([0.0; 2], [false; 2], syy);
                }
            }
            Some(x) => {
                let (mut worst, mut wv) = (usize::MAX, 0.0f64);
                for c in 0..2 {
                    if active[c] && x[c] < 0.0 && x[c] < wv {
                        wv = x[c];
                        worst = c;
                    }
                }
                if worst == usize::MAX {
                    let mut r = syy;
                    for c in 0..2 {
                        r -= 2.0 * x[c] * v[c];
                        for d in 0..2 {
                            r += x[c] * a[c][d] * x[d];
                        }
                    }
                    return (x, active, r.max(0.0));
                }
                active[worst] = false;
            }
        }
    }
}

/// The wall exponent grid, per bohr: `0.50 ..= 4.00` step `0.05`, built so the ends are exact.
fn bgrid(i: usize) -> f64 {
    ((10 + i) as f64) * 0.05
}
/// The contact grid, per bohr: `0.50 ..= 4.00` step `0.02`, built so the ends are exact.
fn cgrid(i: usize) -> f64 {
    ((25 + i) as f64) * 0.02
}

// ------------------------------------------------------------------------ the reading set

/// One of the 66 exchange readings — FIELD-7's 24 among them, read from their records.
struct Reading {
    family: &'static str,
    measured_here: bool,
    r_ang: f64,
    tilt_deg: f64,
    r_oo_bohr: f64,
    oo: Vec<f64>,
    ho: Vec<f64>,
    hh: Vec<f64>,
    e_exch: f64,
    e_hl: f64,
    e_a0: f64,
    e_b0: f64,
    e_es: f64,
    norm: f64,
    nonzero_dets: usize,
    n_det: usize,
    n_det_a: usize,
    n_det_b: usize,
    s_cross_max: f64,
    sigma_seconds: f64,
    cpu: f64,
    source: String,
}

/// The record file every one of the 66 lives in: the CLOSE tilts and every TWIST here, FIELD-7's
/// four far tilt separations in `field7/`.
fn reading_path(out: &Path, family: &str, r: f64, deg: f64) -> PathBuf {
    if family == "twist" {
        out.join(format!("orient_twist_R{r:.1}_t{deg:.0}.json"))
    } else if CLOSE_R.iter().any(|&c| (c - r).abs() < 1e-9) {
        out.join(format!("orient_close_R{r:.1}_t{deg:.0}.json"))
    } else {
        sibling(out, "field7").join(format!("orient_R{r:.1}_t{deg:.0}.json"))
    }
}

/// Build the geometry a reading names, whichever family it is in.
fn reading_geometry(o: Species, h: Species, family: &str, r: f64, deg: f64) -> (Fragment, Fragment) {
    if family == "twist" {
        twisted(o, h, r, TWIST_ABOUT_Z_DEG, deg)
    } else {
        tilted(o, h, r, deg)
    }
}

/// Write one undeformed reading's record.
#[allow(clippy::too_many_arguments)]
fn write_reading(out: &Path, name: &str, family: &str, r: f64, deg: f64, a: &Fragment, b: &Fragment, hl: &HlReading, wall_s: f64, cpu: f64) {
    let r_oo = cross_oo(a, b);
    let (oo, ho, hh) = cross_classes(a, b);
    let axis = if family == "twist" {
        "z by 90° (the O···O axis) then x, both through the acceptor's own oxygen"
    } else {
        "x, through the acceptor's own oxygen"
    };
    fs::write(
        out.join(format!("{name}.json")),
        format!(
            "{{\n  \"node\": \"{name}\", \"family\": \"{family}\", \"r_oo_angstrom\": {r:.3}, \"r_oo_bohr\": {r_oo:.6}, \"tilt_degrees\": {deg:.1}, \"twist_degrees\": {}, \"tilt_axis\": \"{axis}\",\n  \"referee\": \"heitler_london_undeformed\",\n  \"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"e_exch\": {:+.12e},\n  \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"n_det_a\": {}, \"n_det_b\": {}, \"product_dets\": {}, \"product_dets_staked\": {PRODUCT_DETS_STAKED},\n  \"s_cross_max\": {:.12e}, \"sigma_seconds\": {:.3}, \"wall_seconds\": {wall_s:.3}, \"cpu_seconds\": {cpu:.3}, \"threads\": {},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}],\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
            if family == "twist" { format!("{TWIST_ABOUT_Z_DEG:.1}") } else { "0.0".to_string() },
            hl.e_hl,
            hl.e_a0,
            hl.e_b0,
            hl.e_es,
            hl.e_exch,
            hl.norm,
            hl.nonzero_dets,
            hl.n_det,
            hl.n_det_a,
            hl.n_det_b,
            hl.n_det_a * hl.n_det_b,
            hl.s_cross_max,
            hl.sigma_seconds,
            threads(),
            list_json(&oo),
            list_json(&ho),
            list_json(&hh),
            centers_json(a),
            centers_json(b),
        ),
    )
    .unwrap();
}

/// Read all 66 readings from disk, each geometry REBUILT here and checked against its record's own
/// centers (M-STALE-INSTRUMENT). The cross-unit distances are taken from the rebuilt geometry —
/// exact, and the record's printed lists are the same numbers rounded.
fn load_readings(out: &Path, o: Species, h: Species) -> Result<Vec<Reading>, Vec<String>> {
    let mut specs: Vec<(&'static str, f64, f64)> = Vec::with_capacity(N_READ);
    for &r in TILT_R.iter() {
        for &deg in TILT_DEG.iter() {
            specs.push(("tilt", r, deg));
        }
    }
    for &r in TWIST_R.iter() {
        for &deg in TILT_DEG.iter() {
            specs.push(("twist", r, deg));
        }
    }
    let mut missing: Vec<String> = Vec::new();
    let mut out_v: Vec<Reading> = Vec::with_capacity(N_READ);
    for (family, r, deg) in specs {
        let path = reading_path(out, family, r, deg);
        let Ok(t) = fs::read_to_string(&path) else {
            missing.push(path.display().to_string());
            continue;
        };
        let (a, b) = reading_geometry(o, h, family, r, deg);
        let rec_a = json_centers(&t, "donor_centers");
        let rec_b = json_centers(&t, "acceptor_centers");
        let ok = rec_a.len() == a.centers.len()
            && rec_b.len() == b.centers.len()
            && rec_a.iter().zip(a.centers.iter()).all(|(p, q)| dist(p, q) < 1e-9)
            && rec_b.iter().zip(b.centers.iter()).all(|(p, q)| dist(p, q) < 1e-9);
        if !ok {
            missing.push(format!("{} (its centers are NOT the geometry this runner builds)", path.display()));
            continue;
        }
        let e_exch = json_num(&t, "e_exch");
        if !e_exch.is_finite() {
            missing.push(format!("{} (no e_exch)", path.display()));
            continue;
        }
        let (oo, ho, hh) = cross_classes(&a, &b);
        out_v.push(Reading {
            family,
            measured_here: family == "twist" || CLOSE_R.iter().any(|&c| (c - r).abs() < 1e-9),
            r_ang: r,
            tilt_deg: deg,
            r_oo_bohr: cross_oo(&a, &b),
            oo,
            ho,
            hh,
            e_exch,
            e_hl: json_num(&t, "e_hl"),
            e_a0: json_num(&t, "e_a0"),
            e_b0: json_num(&t, "e_b0"),
            e_es: json_num(&t, "e_es"),
            norm: json_num(&t, "norm"),
            nonzero_dets: json_num(&t, "nonzero_dets") as usize,
            n_det: json_num(&t, "n_det") as usize,
            n_det_a: json_num(&t, "n_det_a") as usize,
            n_det_b: json_num(&t, "n_det_b") as usize,
            s_cross_max: json_num(&t, "s_cross_max"),
            sigma_seconds: json_num(&t, "sigma_seconds"),
            cpu: json_num(&t, "cpu_seconds"),
            source: path.display().to_string(),
        });
    }
    if missing.is_empty() {
        Ok(out_v)
    } else {
        Err(missing)
    }
}

// ------------------------------------------------------------------------ the exact record

struct ENode {
    name: String,
    kind: &'static str,
    r_ang: f64,
    r_oo_bohr: f64,
    de_exact: f64,
    a: Fragment,
    b: Fragment,
    oo: Vec<f64>,
    ho: Vec<f64>,
    hh: Vec<f64>,
    outer_linear: bool,
    close_node: bool,
    source: String,
}

/// The TWELVE exact geometries of record (§0), each rebuilt here and CHECKED against the record it
/// is read from (M-STALE-INSTRUMENT): FIELD-3's six linear nodes, FIELD-8's two CLOSE linear
/// nodes, FIELD-5's 30°-bent bond, FIELD-6's 45°-bent bond, FIELD-4's flipped dimer, FIELD-7's
/// twisted dimer.
fn exact_records(out: &Path, o: Species, h: Species) -> Result<Vec<ENode>, Vec<String>> {
    let f3 = sibling(out, "field3");
    let f4 = sibling(out, "field4");
    let f5 = sibling(out, "field5");
    let f6 = sibling(out, "field6");
    let f7 = sibling(out, "field7");
    let mut want: Vec<(String, &'static str, f64, PathBuf, Fragment, Fragment, bool, bool)> = Vec::new();
    for &r in LINEAR_ANGSTROM.iter() {
        let (a, b) = linear(o, h, r);
        let close = CLOSE_NODES.iter().any(|&c| (c - r).abs() < 1e-9);
        let dir = if close { out.to_path_buf() } else { f3.clone() };
        want.push((format!("linear_R{r:.1}"), "linear", r, dir.join(format!("linear_R{r:.1}.json")), a, b, r >= OUTER_FROM_ANGSTROM - 1e-9, close));
    }
    let (a5, b5) = tilted(o, h, TILT5_ANGSTROM, TILT5_DEGREES);
    want.push((format!("tilted_R{TILT5_ANGSTROM:.1}"), "bent 30°", TILT5_ANGSTROM, f5.join(format!("tilted_R{TILT5_ANGSTROM:.1}.json")), a5, b5, false, false));
    let (a6, b6) = tilted(o, h, TILT6_ANGSTROM, TILT6_DEGREES);
    want.push((
        format!("tilted{TILT6_DEGREES:.0}_R{TILT6_ANGSTROM:.1}"),
        "bent 45°",
        TILT6_ANGSTROM,
        f6.join(format!("tilted{TILT6_DEGREES:.0}_R{TILT6_ANGSTROM:.1}.json")),
        a6,
        b6,
        false,
        false,
    ));
    let (a4, b4) = flipped(o, h, FLIPPED_ANGSTROM);
    want.push((format!("flipped_R{FLIPPED_ANGSTROM:.1}"), "flipped 180°", FLIPPED_ANGSTROM, f4.join(format!("flipped_R{FLIPPED_ANGSTROM:.1}.json")), a4, b4, false, false));
    let (a7, b7) = twisted(o, h, TWISTED_ANGSTROM, TWISTED_TWIST_DEGREES, TWISTED_TILT_DEGREES);
    want.push((format!("twisted_R{TWISTED_ANGSTROM:.1}"), "twisted 90°+60°", TWISTED_ANGSTROM, f7.join(format!("twisted_R{TWISTED_ANGSTROM:.1}.json")), a7, b7, false, false));

    let mut missing: Vec<String> = Vec::new();
    let mut nodes: Vec<ENode> = Vec::new();
    for (name, kind, r_ang, path, a, b, outer, close) in want {
        let Ok(t) = fs::read_to_string(&path) else {
            missing.push(path.display().to_string());
            continue;
        };
        let de = json_num(&t, "de_exact");
        if !de.is_finite() {
            missing.push(format!("{} (no de_exact)", path.display()));
            continue;
        }
        let rec_a = json_centers(&t, "donor_centers");
        let rec_b = json_centers(&t, "acceptor_centers");
        let ok = rec_a.len() == a.centers.len()
            && rec_b.len() == b.centers.len()
            && rec_a.iter().zip(a.centers.iter()).all(|(p, q)| dist(p, q) < 1e-9)
            && rec_b.iter().zip(b.centers.iter()).all(|(p, q)| dist(p, q) < 1e-9);
        if !ok {
            missing.push(format!("{} (its centers are NOT the geometry this runner builds)", path.display()));
            continue;
        }
        let (oo, ho, hh) = cross_classes(&a, &b);
        nodes.push(ENode {
            name,
            kind,
            r_ang,
            r_oo_bohr: cross_oo(&a, &b),
            de_exact: de,
            a,
            b,
            oo,
            ho,
            hh,
            outer_linear: outer,
            close_node: close,
            source: path.display().to_string(),
        });
    }
    if missing.is_empty() {
        Ok(nodes)
    } else {
        Err(missing)
    }
}

// ------------------------------------------------------------------------- the close phase

fn run_close(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    eprintln!("FIELD-8 close — the {N_NEW} NEW undeformed Heitler–London readings (18 CLOSE tilt, {N_TWIST} TWIST) and the two CLOSE exact nodes, {} threads", threads());

    let mut close_rows: Vec<(f64, f64, f64)> = Vec::new();
    let mut twist_rows: Vec<(f64, f64, f64)> = Vec::new();

    // ------------------------------------------------- the 18 CLOSE tilt-family readings
    for &r in CLOSE_R.iter() {
        for &deg in TILT_DEG.iter() {
            let (a, b) = tilted(o, h, r, deg);
            let r_oo = cross_oo(&a, &b);
            assert!(
                (r_oo - r * ANGSTROM_TO_BOHR).abs() < 1e-9,
                "the tilt is about the acceptor's own oxygen: R_OO must be unchanged ({r_oo:.9} vs {:.9})",
                r * ANGSTROM_TO_BOHR
            );
            let t0 = Instant::now();
            let c0 = cpu_seconds();
            let hl = heitler_london_undeformed(&a, &b);
            let wall_s = t0.elapsed().as_secs_f64();
            let cpu = cpu_seconds() - c0;
            write_reading(out, &format!("orient_close_R{r:.1}_t{deg:.0}"), "tilt", r, deg, &a, &b, &hl, wall_s, cpu);
            eprintln!(
                "  CLOSE R_OO {r:.1} Å, tilt {deg:>5.1}°: E_exch {:+.6e} Ha, norm {:.12}, σ {:.1} s, {cpu:.1} core-s",
                hl.e_exch, hl.norm, hl.sigma_seconds
            );
            close_rows.push((r, deg, hl.e_exch));
        }
    }

    // ------------------------------------------------------- the 24 TWIST-family readings
    for &r in TWIST_R.iter() {
        for &deg in TILT_DEG.iter() {
            let (a, b) = twisted(o, h, r, TWIST_ABOUT_Z_DEG, deg);
            let r_oo = cross_oo(&a, &b);
            assert!(
                (r_oo - r * ANGSTROM_TO_BOHR).abs() < 1e-9,
                "both rotations fix the acceptor's own oxygen: R_OO must be unchanged ({r_oo:.9} vs {:.9})",
                r * ANGSTROM_TO_BOHR
            );
            let t0 = Instant::now();
            let c0 = cpu_seconds();
            let hl = heitler_london_undeformed(&a, &b);
            let wall_s = t0.elapsed().as_secs_f64();
            let cpu = cpu_seconds() - c0;
            write_reading(out, &format!("orient_twist_R{r:.1}_t{deg:.0}"), "twist", r, deg, &a, &b, &hl, wall_s, cpu);
            eprintln!(
                "  TWIST R_OO {r:.1} Å, tilt {deg:>5.1}°: E_exch {:+.6e} Ha, norm {:.12}, σ {:.1} s, {cpu:.1} core-s",
                hl.e_exch, hl.norm, hl.sigma_seconds
            );
            twist_rows.push((r, deg, hl.e_exch));
        }
    }

    eprintln!("\nthe CLOSE tilt family, E_exch (Ha) — rows R_OO (Å), columns tilt (°):");
    eprintln!("| R (Å) | {} |", TILT_DEG.iter().map(|t| format!("{t:>13.0}")).collect::<Vec<_>>().join(" | "));
    for &r in CLOSE_R.iter() {
        let cells: Vec<String> = TILT_DEG.iter().map(|&t| format!("{:>13.6e}", close_rows.iter().find(|x| (x.0 - r).abs() < 1e-9 && (x.1 - t).abs() < 1e-9).map(|x| x.2).unwrap_or(f64::NAN))).collect();
        eprintln!("| {r:.1} | {} |", cells.join(" | "));
    }
    eprintln!("\nthe TWIST family (90° about the O···O axis, then the tilt), E_exch (Ha):");
    eprintln!("| R (Å) | {} |", TILT_DEG.iter().map(|t| format!("{t:>13.0}")).collect::<Vec<_>>().join(" | "));
    for &r in TWIST_R.iter() {
        let cells: Vec<String> = TILT_DEG.iter().map(|&t| format!("{:>13.6e}", twist_rows.iter().find(|x| (x.0 - r).abs() < 1e-9 && (x.1 - t).abs() < 1e-9).map(|x| x.2).unwrap_or(f64::NAN))).collect();
        eprintln!("| {r:.1} | {} |", cells.join(" | "));
    }
    fs::write(out.join("close_readings.done"), format!("{N_NEW} readings\n")).unwrap();

    // ------------------------------------------------------------- the two CLOSE exact nodes
    eprintln!("\nthe two CLOSE exact nodes on the LINE, sequentially — where the contact term must be MEASURED and not extrapolated:");
    for &r in CLOSE_NODES.iter() {
        let (a, b) = linear(o, h, r);
        solve_node(out, &format!("linear_R{r:.1}"), r, &a, &b, false);
    }
    fs::write(out.join("close.done"), "done\n").unwrap();
}

// --------------------------------------------------------------------------- the fit phase

fn run_fit(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    eprintln!("FIELD-8 fit — W0, the three-class wall over {N_READ}, plant (ii), the two-class contact term over twelve exact points, G-N0, G-C1");

    let exact = match exact_records(out, o, h) {
        Ok(v) => v,
        Err(missing) => {
            eprintln!("REFUSED — the contact term is staked on TWELVE exact geometries (§0) and these are not on disk:");
            for m in &missing {
                eprintln!("  {m}");
            }
            eprintln!("Nothing written. Re-run `fit` when the records exist (`close` writes the two CLOSE linear nodes).");
            std::process::exit(3);
        }
    };
    let readings = match load_readings(out, o, h) {
        Ok(v) => v,
        Err(missing) => {
            eprintln!("REFUSED — the wall is staked on {N_READ} readings (§0) and these are not on disk:");
            for m in &missing {
                eprintln!("  {m}");
            }
            eprintln!("Nothing written. Re-run `fit` when `close` has finished its {N_NEW} readings.");
            std::process::exit(3);
        }
    };
    assert_eq!(readings.len(), N_READ, "the reading set is 66");
    eprintln!("the twelve exact geometries of record, each rebuilt here and matched to its record's centers:");
    for e in &exact {
        eprintln!("  {} ({}): R_OO {:.4} bohr, ΔE_exact {:+.6e} Ha  [{}]", e.name, e.kind, e.r_oo_bohr, e.de_exact, e.source);
    }
    let reused = readings.iter().filter(|r| !r.measured_here).count();
    eprintln!("the {N_READ} readings: {} measured by this runner, {reused} reused from FIELD-7's records", readings.len() - reused);

    // ------------------------------------------------- W0: the readings are what they say
    let at_tilt = |ri: usize, ti: usize| -> &Reading { &readings[ri * N_T + ti] };
    let at_twist = |ri: usize, ti: usize| -> &Reading { &readings[N_TILT + ri * N_T + ti] };
    let norm_ok = readings.iter().all(|n| n.norm > NORM_LO && n.norm <= NORM_HI);
    let floor_ok = readings.iter().all(|n| n.e_exch > R_FLOOR);
    let mut monotone_ok = true;
    let mut monotone_breaks: Vec<String> = Vec::new();
    let mut push_break = |family: &str, tilt: f64, r0: f64, r1: f64, e0: f64, e1: f64| {
        monotone_breaks.push(format!(
            "{{\"family\": \"{family}\", \"tilt_degrees\": {tilt:.1}, \"r_from\": {r0:.1}, \"r_to\": {r1:.1}, \"e_exch_from\": {e0:+.12e}, \"e_exch_to\": {e1:+.12e}}}"
        ));
    };
    for ti in 0..N_T {
        for ri in 1..TILT_R.len() {
            if !(at_tilt(ri, ti).e_exch <= at_tilt(ri - 1, ti).e_exch) {
                monotone_ok = false;
                push_break("tilt", TILT_DEG[ti], TILT_R[ri - 1], TILT_R[ri], at_tilt(ri - 1, ti).e_exch, at_tilt(ri, ti).e_exch);
            }
        }
        for ri in 1..TWIST_R.len() {
            if !(at_twist(ri, ti).e_exch <= at_twist(ri - 1, ti).e_exch) {
                monotone_ok = false;
                push_break("twist", TILT_DEG[ti], TWIST_R[ri - 1], TWIST_R[ri], at_twist(ri - 1, ti).e_exch, at_twist(ri, ti).e_exch);
            }
        }
    }
    // the same order leg read with the tilt family SPLIT at FIELD-7's boundary (its own §0
    // grouping: a CLOSE tilt family and FIELD-7's far one). Recorded beside the joined reading,
    // never instead of it.
    let mut monotone_split_breaks = 0usize;
    for ti in 0..N_T {
        for ri in 1..TILT_R.len() {
            let boundary = (TILT_R[ri - 1] - CLOSE_R[CLOSE_R.len() - 1]).abs() < 1e-9;
            if boundary {
                continue;
            }
            if !(at_tilt(ri, ti).e_exch <= at_tilt(ri - 1, ti).e_exch) {
                monotone_split_breaks += 1;
            }
        }
        for ri in 1..TWIST_R.len() {
            if !(at_twist(ri, ti).e_exch <= at_twist(ri - 1, ti).e_exch) {
                monotone_split_breaks += 1;
            }
        }
    }
    let w0 = norm_ok && floor_ok && monotone_ok;
    let norm_lo_seen = readings.iter().map(|n| n.norm).fold(f64::INFINITY, f64::min);
    let norm_hi_seen = readings.iter().map(|n| n.norm).fold(f64::NEG_INFINITY, f64::max);
    eprintln!("\nW0 — the readings are what they say ({N_READ} readings):");
    eprintln!("  overlap: every ⟨v|v⟩ in ({NORM_LO}, {NORM_HI}] — lowest {norm_lo_seen:.12}, highest {norm_hi_seen:.12} → {}", if norm_ok { "PASS" } else { "FAIL" });
    eprintln!("  floor: E_exch > {R_FLOOR:e} at every reading → {}", if floor_ok { "PASS" } else { "FAIL" });
    eprintln!(
        "  order: E_exch non-increasing in R_OO along each (family, tilt) → {} ({} breaks; with the tilt family split at 2.5/2.7 Å, {monotone_split_breaks})",
        if monotone_ok { "PASS" } else { "FAIL" },
        monotone_breaks.len()
    );
    for b in &monotone_breaks {
        eprintln!("    break {b}");
    }
    eprintln!("W0 → {}", if w0 { "PASS" } else { "FAIL" });
    let new_cpu: Vec<f64> = readings.iter().filter(|r| r.measured_here).map(|r| r.cpu).collect();
    let price_ok = new_cpu.iter().all(|&c| c >= HL_PRICE_TENTH_CORE_S);
    eprintln!(
        "M-CHEAPER-THAN-ITS-PRICE (recorded): FIELD-7's price is {HL_PRICE_LO_CORE_S}–{HL_PRICE_HI_CORE_S} core-seconds; every NEW reading at or above a tenth of the low end: {price_ok} (cheapest {:.1}, dearest {:.1})",
        new_cpu.iter().cloned().fold(f64::INFINITY, f64::min),
        new_cpu.iter().cloned().fold(0.0f64, f64::max)
    );

    // --------------------------------------------- the three-class wall over the 66
    let y: Vec<f64> = readings.iter().map(|n| n.e_exch).collect();
    let w: Vec<f64> = y.iter().map(|v| 1.0 / (v * v)).collect();
    let tol: Vec<f64> = y.iter().map(|v| (WALL_TOL_FRAC * v).max(WALL_TOL_ABS)).collect();
    let syy: f64 = (0..N_READ).map(|g| w[g] * y[g] * y[g]).sum();

    let mut soo = vec![0.0f64; NB * N_READ];
    let mut soh = vec![0.0f64; NB * N_READ];
    let mut shh = vec![0.0f64; NB * N_READ];
    for i in 0..NB {
        let b = bgrid(i);
        for (g, n) in readings.iter().enumerate() {
            soo[i * N_READ + g] = sum_exp(&n.oo, b);
            soh[i * N_READ + g] = sum_exp(&n.ho, b);
            shh[i * N_READ + g] = sum_exp(&n.hh, b);
        }
    }
    let mut a00 = vec![0.0f64; NB];
    let mut a11 = vec![0.0f64; NB];
    let mut a22 = vec![0.0f64; NB];
    let mut v0 = vec![0.0f64; NB];
    let mut v1 = vec![0.0f64; NB];
    let mut v2 = vec![0.0f64; NB];
    for i in 0..NB {
        for g in 0..N_READ {
            let (p, q, s) = (soo[i * N_READ + g], soh[i * N_READ + g], shh[i * N_READ + g]);
            a00[i] += w[g] * p * p;
            a11[i] += w[g] * q * q;
            a22[i] += w[g] * s * s;
            v0[i] += w[g] * p * y[g];
            v1[i] += w[g] * q * y[g];
            v2[i] += w[g] * s * y[g];
        }
    }
    let mut a01 = vec![0.0f64; NB * NB];
    let mut a02 = vec![0.0f64; NB * NB];
    let mut a12 = vec![0.0f64; NB * NB];
    for i in 0..NB {
        for j in 0..NB {
            let (mut x01, mut x02, mut x12) = (0.0, 0.0, 0.0);
            for g in 0..N_READ {
                x01 += w[g] * soo[i * N_READ + g] * soh[j * N_READ + g];
                x02 += w[g] * soo[i * N_READ + g] * shh[j * N_READ + g];
                x12 += w[g] * soh[i * N_READ + g] * shh[j * N_READ + g];
            }
            a01[i * NB + j] = x01;
            a02[i * NB + j] = x02;
            a12[i * NB + j] = x12;
        }
    }
    let t_fit = Instant::now();
    let (mut best_r, mut best) = (f64::INFINITY, (0usize, 0usize, 0usize, [0.0f64; 3], [false; 3]));
    for i in 0..NB {
        for j in 0..NB {
            let x01 = a01[i * NB + j];
            for k in 0..NB {
                let x02 = a02[i * NB + k];
                let x12 = a12[j * NB + k];
                let am = [[a00[i], x01, x02], [x01, a11[j], x12], [x02, x12, a22[k]]];
                let vv = [v0[i], v1[j], v2[k]];
                let (x, act, r) = fit_nonneg(&am, &vv, syy);
                if r < best_r {
                    best_r = r;
                    best = (i, j, k, x, act);
                }
            }
        }
    }
    let (bi, bj, bk, amp, act) = best;
    let (b_oo, b_oh, b_hh) = (bgrid(bi), bgrid(bj), bgrid(bk));
    let (a_oo, a_oh, a_hh) = (amp[0], amp[1], amp[2]);
    let fit_seconds = t_fit.elapsed().as_secs_f64();
    eprintln!(
        "\nthe three-class wall over {N_READ} readings — {} triples on 0.50..=4.00 step 0.05, weighted (1/E_exch²) least squares with amplitudes constrained non-negative, in {fit_seconds:.1} s:",
        NB * NB * NB
    );
    eprintln!("  b_OO = {b_oo:.2} /bohr, A_OO = {a_oo:.9e} Ha (kept: {})", act[0]);
    eprintln!("  b_OH = {b_oh:.2} /bohr, A_OH = {a_oh:.9e} Ha (kept: {})", act[1]);
    eprintln!("  b_HH = {b_hh:.2} /bohr, A_HH = {a_hh:.9e} Ha (kept: {})", act[2]);
    eprintln!("  weighted residual {best_r:.9e}");
    let wall3 = |oo: &[f64], ho: &[f64], hh: &[f64]| -> f64 { a_oo * sum_exp(oo, b_oo) + a_oh * sum_exp(ho, b_oh) + a_hh * sum_exp(hh, b_hh) };

    let mut within = 0usize;
    let mut misses: Vec<String> = Vec::new();
    let mut miss_lines: Vec<String> = Vec::new();
    eprintln!("\n| family | R (Å) | tilt (°) | E_exch (Ha) | model (Ha) | miss (Ha) | miss/tol | within |");
    for (g, n) in readings.iter().enumerate() {
        let m = wall3(&n.oo, &n.ho, &n.hh);
        let miss = (m - y[g]).abs();
        let ok = miss <= tol[g];
        if ok {
            within += 1;
        } else {
            misses.push(format!("({}, R = {:.1} Å, tilt = {:.0}°)", n.family, n.r_ang, n.tilt_deg));
            miss_lines.push(format!(
                "{{\"family\": \"{}\", \"r_angstrom\": {:.1}, \"tilt_degrees\": {:.1}, \"e_exch\": {:+.12e}, \"model\": {:+.12e}, \"miss\": {miss:.12e}, \"tolerance\": {:.12e}, \"miss_over_tolerance\": {:.6}}}",
                n.family,
                n.r_ang,
                n.tilt_deg,
                y[g],
                m,
                tol[g],
                miss / tol[g]
            ));
        }
        eprintln!("| {} | {:.1} | {:.0} | {:+.6e} | {:+.6e} | {:+.6e} | {:.4} | {} |", n.family, n.r_ang, n.tilt_deg, y[g], m, m - y[g], miss / tol[g], ok);
    }
    let s1_branch = if within == N_READ {
        "a"
    } else if within >= S1_B_MIN {
        "b"
    } else {
        "c"
    };
    eprintln!(
        "S1: {within} of {N_READ} within max({WALL_TOL_FRAC}·E_exch, {WALL_TOL_ABS:e}) → branch ({s1_branch}) — {}",
        match s1_branch {
            "a" => "the three-class wall carries exchange across orientation and separation at this level".to_string(),
            "b" => format!("transferred, the {} misses reported: {}", misses.len(), misses.join(", ")),
            _ => format!("VOID: the arms do not run. {} misses: {}", misses.len(), misses.join(", ")),
        }
    );

    // ------------------- plant (ii): FIELD-7's FAR-DATA wall on the 18 CLOSE readings
    let w7path = sibling(out, "field7").join("wall7.json");
    let w7 = fs::read_to_string(&w7path).unwrap_or_else(|_| panic!("plant (ii) reads FIELD-7's wall from {}", w7path.display()));
    let (p2_a_oo, p2_b_oo) = (json_num(&w7, "a"), json_num(&w7, "b"));
    let (p2_a_oh, p2_b_oh) = (json_num(&w7, "a_oh"), json_num(&w7, "b_oh"));
    let (p2_a_hh, p2_b_hh) = (json_num(&w7, "a_hh"), json_num(&w7, "b_hh"));
    assert!(
        p2_a_oo.is_finite() && p2_b_oo.is_finite() && p2_a_oh.is_finite() && p2_b_oh.is_finite() && p2_a_hh.is_finite() && p2_b_hh.is_finite(),
        "field7/wall7.json carries no three-class wall"
    );
    let close_idx: Vec<usize> = (0..N_READ).filter(|&g| readings[g].family == "tilt" && CLOSE_R.iter().any(|&c| (c - readings[g].r_ang).abs() < 1e-9)).collect();
    let mut p2_fail = 0usize;
    let mut p2_fail_lines: Vec<String> = Vec::new();
    eprintln!("\nplant (ii) — FIELD-7's FAR-DATA wall (fit on its 24 readings at R ≥ 2.7 Å) evaluated on the {} CLOSE readings:", close_idx.len());
    eprintln!("  b_OO {p2_b_oo:.2}, A_OO {p2_a_oo:.6e}; b_OH {p2_b_oh:.2}, A_OH {p2_a_oh:.6e}; b_HH {p2_b_hh:.2}, A_HH {p2_a_hh:.6e}");
    for &g in close_idx.iter() {
        let n = &readings[g];
        let m = p2_a_oo * sum_exp(&n.oo, p2_b_oo) + p2_a_oh * sum_exp(&n.ho, p2_b_oh) + p2_a_hh * sum_exp(&n.hh, p2_b_hh);
        let miss = (m - n.e_exch).abs();
        let bar = PLANT_II_TOL_FRAC * n.e_exch;
        let fails = miss > bar;
        if fails {
            p2_fail += 1;
        }
        eprintln!("  R {:.1} Å, tilt {:>5.1}°: E_exch {:+.6e}, FIELD-7 wall {:+.6e}, miss {miss:.6e} vs {PLANT_II_TOL_FRAC}·E_exch {bar:.6e} → {}", n.r_ang, n.tilt_deg, n.e_exch, m, if fails { "MISSES" } else { "within" });
        p2_fail_lines.push(format!(
            "{{\"r_angstrom\": {:.1}, \"tilt_degrees\": {:.1}, \"e_exch\": {:+.12e}, \"model\": {:+.12e}, \"miss\": {miss:.12e}, \"bar\": {bar:.12e}, \"misses\": {fails}}}",
            n.r_ang, n.tilt_deg, n.e_exch, m
        ));
    }
    let carrier_ii = readings
        .iter()
        .find(|n| n.family == "tilt" && (n.r_ang - PLANT_II_CARRIER_R).abs() < 1e-9 && (n.tilt_deg - PLANT_II_CARRIER_TILT).abs() < 1e-9)
        .map(|n| n.e_exch)
        .unwrap_or(f64::NAN);
    let carrier_ii_ok = carrier_ii >= PLANT_II_CARRIER_FLOOR;
    let plant_ii_fires = p2_fail >= PLANT_II_MIN_FAIL && carrier_ii_ok;
    eprintln!(
        "  misses on {p2_fail} of {} (needs ≥ {PLANT_II_MIN_FAIL}); carrier E_exch({PLANT_II_CARRIER_R:.1} Å, {PLANT_II_CARRIER_TILT:.0}°) = {carrier_ii:.6e} Ha ≥ {PLANT_II_CARRIER_FLOOR}: {carrier_ii_ok} → {}",
        close_idx.len(),
        if plant_ii_fires { "FIRES" } else { "does not fire" }
    );

    // ---------------- the contact term on TWO classes, with the NEW wall HELD, on the twelve
    eprintln!("\nthe contact term on TWO classes with the wall HELD: remainder = ΔE_exact − [E_q(g) − E_q(40)]_engine − wall(g), on the twelve exact geometries");
    let mut e_q: Vec<f64> = Vec::with_capacity(exact.len());
    let mut rem: Vec<f64> = Vec::with_capacity(exact.len());
    let mut wall_e: Vec<f64> = Vec::with_capacity(exact.len());
    for e in &exact {
        let (_, e_q_diff, _) = engine_interaction(&e.a, &e.b, None, SeamPlant::None);
        let wv = wall3(&e.oo, &e.ho, &e.hh);
        let r = e.de_exact - e_q_diff - wv;
        eprintln!(
            "  {} ({}): ΔE_exact {:+.6e}, E_q(g) − E_q(40) {:+.6e}, wall {:+.6e} → remainder {:+.6e} Ha ({:+.4} of |ΔE_exact|)",
            e.name,
            e.kind,
            e.de_exact,
            e_q_diff,
            wv,
            r,
            r / e.de_exact.abs()
        );
        e_q.push(e_q_diff);
        wall_e.push(wv);
        rem.push(r);
    }
    let we: Vec<f64> = exact.iter().map(|e| 1.0 / (e.de_exact * e.de_exact)).collect();
    let syy_c: f64 = (0..exact.len()).map(|g| we[g] * rem[g] * rem[g]).sum();
    // the design columns are `X_HO = −Σ_HO e^{−c_HO r}` and `X_HH = −Σ_HH e^{−c_HH r}`
    let mut cho = vec![0.0f64; NC * exact.len()];
    let mut chh = vec![0.0f64; NC * exact.len()];
    for i in 0..NC {
        let c = cgrid(i);
        for (g, e) in exact.iter().enumerate() {
            cho[i * exact.len() + g] = -sum_exp(&e.ho, c);
            chh[i * exact.len() + g] = -sum_exp(&e.hh, c);
        }
    }
    let t_cfit = Instant::now();
    let (mut c_res, mut c_best) = (f64::INFINITY, (0usize, 0usize, [0.0f64; 2], [false; 2]));
    for i in 0..NC {
        for j in 0..NC {
            let (mut m00, mut m01, mut m11, mut r0, mut r1) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
            for g in 0..exact.len() {
                let (x0, x1) = (cho[i * exact.len() + g], chh[j * exact.len() + g]);
                m00 += we[g] * x0 * x0;
                m01 += we[g] * x0 * x1;
                m11 += we[g] * x1 * x1;
                r0 += we[g] * x0 * rem[g];
                r1 += we[g] * x1 * rem[g];
            }
            let (x, keep, r) = fit_nonneg2([[m00, m01], [m01, m11]], [r0, r1], syy_c);
            if r < c_res {
                c_res = r;
                c_best = (i, j, x, keep);
            }
        }
    }
    let (ci, cj, camp, ckeep) = c_best;
    let (c_ho, c_hh) = (cgrid(ci), cgrid(cj));
    let (p_ho, p_hh) = (camp[0], camp[1]);
    let cfit_seconds = t_cfit.elapsed().as_secs_f64();
    eprintln!(
        "\nthe two-class contact fit — {} pairs on 0.50..=4.00 step 0.02 per class, weighted (1/ΔE_exact²), non-negative by drop-and-refit, in {cfit_seconds:.1} s:",
        NC * NC
    );
    eprintln!("  P_HO = {p_ho:.9e} Ha, c_HO = {c_ho:.2} /bohr (kept: {})", ckeep[0]);
    eprintln!("  P_HH = {p_hh:.9e} Ha, c_HH = {c_hh:.2} /bohr (kept: {})", ckeep[1]);
    eprintln!("  weighted residual {c_res:.9e}");
    let contact = |ho: &[f64], hh: &[f64]| -> f64 { -p_ho * sum_exp(ho, c_ho) - p_hh * sum_exp(hh, c_hh) };

    let mut c1_within = 0usize;
    let mut c1_close_within = 0usize;
    let mut c1_close_of = 0usize;
    let mut c1_misses: Vec<String> = Vec::new();
    let mut c1_lines: Vec<String> = Vec::new();
    let mut close_miss_report: Vec<String> = Vec::new();
    for (g, e) in exact.iter().enumerate() {
        let f = contact(&e.ho, &e.hh);
        let miss = (rem[g] - f).abs();
        let t = (C1_FRAC * e.de_exact.abs()).max(C1_ABS);
        let ok = miss <= t;
        if ok {
            c1_within += 1;
        } else {
            c1_misses.push(e.name.clone());
        }
        if e.close_node {
            c1_close_of += 1;
            if ok {
                c1_close_within += 1;
            }
            close_miss_report.push(format!("{}: miss {miss:.6e} vs tolerance {t:.6e} → {}", e.name, if ok { "within" } else { "MISS" }));
        }
        eprintln!("  {}: remainder {:+.6e}, fit {f:+.6e}, miss {miss:.6e} ({:.4} of its tolerance {t:.3e}) → {}", e.name, rem[g], miss / t, if ok { "within" } else { "MISS" });
        c1_lines.push(format!(
            "{{\"node\": \"{}\", \"kind\": \"{}\", \"source\": \"{}\", \"r_angstrom\": {:.1}, \"r_oo_bohr\": {:.6}, \"close_node\": {}, \"outer_linear\": {}, \"de_exact\": {:+.12e}, \"e_q_difference\": {:+.12e}, \"wall\": {:+.12e}, \"remainder\": {:+.12e}, \"contact_fit\": {f:+.12e}, \"miss\": {miss:.12e}, \"tolerance\": {t:.12e}, \"within\": {ok}}}",
            e.name,
            e.kind,
            e.source,
            e.r_ang,
            e.r_oo_bohr,
            e.close_node,
            e.outer_linear,
            e.de_exact,
            e_q[g],
            wall_e[g],
            rem[g]
        ));
    }
    let c1_close_ok = c1_close_within == c1_close_of;
    let c1 = c1_within >= C1_MIN && c1_close_ok;
    eprintln!(
        "C1: {c1_within} of {} within max({C1_FRAC}·|ΔE_exact|, {C1_ABS:e}) (needs ≥ {C1_MIN}), and BOTH close nodes within: {c1_close_ok} ({c1_close_within} of {c1_close_of}) → {}{}",
        exact.len(),
        if c1 { "PASS" } else { "FAIL — the contact term is not two exponentials across these kinds" },
        if c1_misses.is_empty() { String::new() } else { format!("; misses: {}", c1_misses.join(", ")) }
    );
    for l in &close_miss_report {
        eprintln!("  close node — {l}");
    }

    // ------------------- dispersion: what is left on the FOUR OUTER LINEAR nodes only
    let outer: Vec<usize> = (0..exact.len()).filter(|&g| exact[g].outer_linear).collect();
    let rem2: Vec<f64> = outer.iter().map(|&g| rem[g] - contact(&exact[g].ho, &exact[g].hh)).collect();
    let (mut num, mut den) = (0.0f64, 0.0f64);
    for (oi, &g) in outer.iter().enumerate() {
        let x = -1.0 / exact[g].r_oo_bohr.powi(6);
        num += we[g] * rem2[oi] * x;
        den += we[g] * x * x;
    }
    let mut c6 = if den > 0.0 { num / den } else { 0.0 };
    let mut slopes: Vec<(f64, f64)> = Vec::new();
    eprintln!("\ndispersion — the remainder AFTER the two contact terms, on the four outer linear nodes:");
    for (oi, &g) in outer.iter().enumerate() {
        eprintln!("  {:.1} Å: remainder after contact {:+.6e} Ha ({:+.4} of |ΔE_exact|)", exact[g].r_ang, rem2[oi], rem2[oi] / exact[g].de_exact.abs());
    }
    for oi in 1..outer.len() {
        let (a1, a0) = (rem2[oi], rem2[oi - 1]);
        let (r1, r0) = (exact[outer[oi]].r_oo_bohr, exact[outer[oi - 1]].r_oo_bohr);
        let s = if a1 != 0.0 && a0 != 0.0 { (a1.abs() / a0.abs()).ln() / (r1 / r0).ln() } else { f64::NAN };
        eprintln!("  log-log slope {:.1} → {:.1} Å: {s:.3}", exact[outer[oi - 1]].r_ang, exact[outer[oi]].r_ang);
        slopes.push((exact[outer[oi]].r_ang, s));
    }
    let c6_transferred = !slopes.is_empty() && slopes.iter().all(|(_, s)| s.is_finite() && *s >= SLOPE_LO && *s <= SLOPE_HI);
    if !c6_transferred {
        c6 = 0.0;
    }
    eprintln!(
        "dispersion: C₆ = {c6:.9e} Ha·bohr⁶; every slope in [{SLOPE_LO}, {SLOPE_HI}] → {}",
        if c6_transferred { "TRANSFERRED" } else { "NOT transferred (C₆ = 0 recorded)" }
    );

    // ------------------------------------------------------- the harvested law, and G-N0
    let model = SeamModel { a: a_oo, b: b_oo, p: p_ho, c: c_ho, c6, a_oh, b_oh, a_hh, b_hh, p_hh, c_hh, ..SeamModel::NO_WALL };
    let q_h = holon_render::field::water_charge_at_pin();
    let hole = model.hole(q_h);
    let g_n0 = hole.is_none();
    eprintln!("\nG-N0 — no hole: the engine walks every cross-unit class potential of the HARVESTED law in from 3.0 bohr, q_H = {q_h:.12e}");
    match &hole {
        None => eprintln!("  hole() returned none → PASS: the law rises monotonically inward on the grid; the arms may run"),
        Some(msg) => eprintln!("  hole() named a fall, verbatim: {msg}\n  → the arms are VOID before they run (§2 G-N0). The harvest is still recorded in full."),
    }

    // ------------------------------------------ G-C1 and plant (i), by the engine
    eprintln!("\nG-C1 — the harvest is the engine's arithmetic, ONE reference (E_q(g) − E_q(40) from the engine itself), on the twelve exact geometries");
    let mut g_c1_worst = 0.0f64;
    let mut g_c1_lines: Vec<String> = Vec::new();
    let mut plant_i = (f64::NAN, f64::NAN, f64::NAN, false);
    let mut units_ok = true;
    for e in &exact {
        let (e_int, e_field_diff, e_seam_diff) = engine_interaction(&e.a, &e.b, Some(model), SeamPlant::None);
        let t = formula_terms(&e.a, &e.b, &model);
        let want = e_field_diff + t.total();
        let miss = (e_int - want).abs();
        g_c1_worst = g_c1_worst.max(miss);
        let s = engine_dimer(&e.a, &e.b, Some(model), SeamPlant::None);
        let (units, oo_pairs, ho_pairs) = (s.seam_work.units, s.seam_work.oo_pairs, s.seam_work.ho_pairs);
        if units != 2 || oo_pairs != 1 || ho_pairs != 4 {
            units_ok = false;
        }
        eprintln!(
            "  {}: engine {e_int:+.12e} vs formula {want:+.12e} — miss {miss:.3e} (E_q diff {e_field_diff:+.6e}, seam {e_seam_diff:+.6e}; pen_HO {:+.6e}, pen_HH {:+.6e}, wall_OO {:+.6e}, wall_OH {:+.6e}, wall_HH {:+.6e}, disp {:+.6e}; units {units}, O–O {oo_pairs}, H–O {ho_pairs})",
            e.name, t.pen_ho, t.pen_hh, t.w_oo, t.w_oh, t.w_hh, t.disp
        );
        g_c1_lines.push(format!(
            "{{\"node\": \"{}\", \"r_angstrom\": {:.1}, \"engine_interaction\": {e_int:+.12e}, \"formula\": {want:+.12e}, \"miss\": {miss:.3e}, \"e_q_difference\": {e_field_diff:+.12e}, \"engine_seam\": {e_seam_diff:+.12e}, \"contact_ho\": {:+.12e}, \"contact_hh\": {:+.12e}, \"wall_oo\": {:+.12e}, \"wall_oh\": {:+.12e}, \"wall_hh\": {:+.12e}, \"disp\": {:+.12e}, \"units\": {units}, \"oo_pairs\": {oo_pairs}, \"ho_pairs\": {ho_pairs}}}",
            e.name, e.r_ang, t.pen_ho, t.pen_hh, t.w_oo, t.w_oh, t.w_hh, t.disp
        ));
        if e.kind == "linear" && (e.r_ang - REF_ANGSTROM).abs() < 1e-9 {
            let (e_pl, _, _) = engine_interaction(&e.a, &e.b, Some(model), SeamPlant::FlipPenetration);
            let observed = (e_pl - e_int).abs();
            let expected = 2.0 * t.pen_ho.abs();
            let carrier = t.pen_ho.abs();
            let fires = carrier >= PLANT_I_CARRIER && (observed - expected).abs() <= G_C1_TOL;
            plant_i = (observed, expected, carrier, fires);
            eprintln!(
                "plant (i) at the linear {REF_ANGSTROM:.1} Å node: miss {observed:.6e} vs 2·|p_HO(2.9)| {expected:.6e} (difference {:.3e}); carrier |p_HO(2.9)| {carrier:.3e} ≥ {PLANT_I_CARRIER:e}: {} → {}",
                (observed - expected).abs(),
                carrier >= PLANT_I_CARRIER,
                if fires { "FIRES" } else { "does not fire" }
            );
        }
    }
    let g_c1 = g_c1_worst <= G_C1_TOL;
    eprintln!("G-C1: worst |engine − formula| = {g_c1_worst:.3e} (stake {G_C1_TOL:e}) → {}", if g_c1 { "PASS" } else { "FAIL" });
    eprintln!("M-VACUOUS-SUCCESS: every G-C1 geometry served two units, one cross O–O pair and four cross H–O pairs: {units_ok}");

    // ------------------------------------------------------------------------ wall8.json
    let reading_lines: Vec<String> = readings
        .iter()
        .enumerate()
        .map(|(g, n)| {
            let m = wall3(&n.oo, &n.ho, &n.hh);
            format!(
                "{{\"family\": \"{}\", \"r_angstrom\": {:.1}, \"tilt_degrees\": {:.1}, \"r_oo_bohr\": {:.6}, \"measured_here\": {}, \"source\": \"{}\", \"e_exch\": {:+.12e}, \"model\": {:+.12e}, \"miss\": {:+.12e}, \"tolerance\": {:.12e}, \"within\": {}, \"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"n_det_a\": {}, \"n_det_b\": {}, \"product_dets\": {}, \"s_cross_max\": {:.6e}, \"sigma_seconds\": {:.3}, \"cpu_seconds\": {:.3}, \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}]}}",
                n.family,
                n.r_ang,
                n.tilt_deg,
                n.r_oo_bohr,
                n.measured_here,
                n.source,
                n.e_exch,
                m,
                m - n.e_exch,
                tol[g],
                (m - n.e_exch).abs() <= tol[g],
                n.e_hl,
                n.e_a0,
                n.e_b0,
                n.e_es,
                n.norm,
                n.nonzero_dets,
                n.n_det,
                n.n_det_a,
                n.n_det_b,
                n.n_det_a * n.n_det_b,
                n.s_cross_max,
                n.sigma_seconds,
                n.cpu,
                list_json(&n.oo),
                list_json(&n.ho),
                list_json(&n.hh),
            )
        })
        .collect();
    let slope_lines: Vec<String> = slopes.iter().map(|(r, s)| format!("{{\"r_angstrom\": {r:.1}, \"loglog_slope_from_previous\": {s:.6}}}")).collect();
    let miss_names: Vec<String> = misses.iter().map(|m| format!("\"{m}\"")).collect();
    let c1_miss_names: Vec<String> = c1_misses.iter().map(|m| format!("\"{m}\"")).collect();
    let hole_json = match &hole {
        None => "null".to_string(),
        Some(msg) => format!("\"{}\"", msg.replace('\\', "\\\\").replace('"', "\\\"")),
    };

    let mut j = String::new();
    j.push_str("{\n");
    j.push_str(&format!(
        "  \"a\": {a_oo:.12e}, \"b\": {b_oo:.12e}, \"p\": {p_ho:.12e}, \"c\": {c_ho:.12e}, \"c6\": {c6:.12e}, \"a_oh\": {a_oh:.12e}, \"b_oh\": {b_oh:.12e}, \"a_hh\": {a_hh:.12e}, \"b_hh\": {b_hh:.12e}, \"p_hh\": {p_hh:.12e}, \"c_hh\": {c_hh:.12e},\n"
    ));
    j.push_str(&format!(
        "  \"w0\": {{\"pass\": {w0}, \"readings\": {N_READ}, \"measured_here\": {N_NEW}, \"reused_from_field7\": {reused}, \"overlap_ok\": {norm_ok}, \"overlap_window\": \"({NORM_LO}, {NORM_HI}]\", \"overlap_lowest\": {norm_lo_seen:.15e}, \"overlap_highest\": {norm_hi_seen:.15e}, \"floor_ok\": {floor_ok}, \"floor\": {R_FLOOR:e}, \"monotone_ok\": {monotone_ok}, \"monotone_rule\": \"E_exch non-increasing in R_OO along each (family, tilt); the tilt family JOINED across FIELD-7's boundary (7 separations), the twist family its own (4)\", \"monotone_breaks\": [{}], \"monotone_breaks_with_tilt_family_split\": {monotone_split_breaks}, \"count_leg\": \"retired by §2 — the undeformed state is nonzero on the full space by construction; nonzero_dets and product_dets are recorded per reading\", \"n_det\": {}, \"product_dets_staked\": {PRODUCT_DETS_STAKED}}},\n",
        monotone_breaks.join(", "),
        readings[0].n_det
    ));
    j.push_str(&format!("  \"s1_branch\": \"{s1_branch}\", \"within_count\": {within}, \"of\": {N_READ}, \"branch_b_minimum\": {S1_B_MIN}, \"misses\": [{}],\n", miss_names.join(", ")));
    j.push_str(&format!("  \"miss_details\": [{}],\n", miss_lines.join(", ")));
    j.push_str(&format!(
        "  \"wall_fit\": {{\"readings\": {N_READ}, \"grid\": \"0.50..=4.00 per bohr, step 0.05, per class\", \"triples\": {}, \"weights\": \"1/E_exch²\", \"nonnegativity\": \"a class whose fitted amplitude is negative is dropped (amplitude an exact 0.0) and the rest refit; the most negative first, one class per round\", \"weighted_residual\": {best_r:.12e}, \"classes_kept\": {{\"oo\": {}, \"oh\": {}, \"hh\": {}}}, \"tolerance_rule\": \"max({WALL_TOL_FRAC}·E_exch, {WALL_TOL_ABS:e})\", \"fit_seconds\": {fit_seconds:.3}}},\n",
        NB * NB * NB,
        act[0],
        act[1],
        act[2]
    ));
    j.push_str(&format!(
        "  \"plant_ii\": {{\"fires\": {plant_ii_fires}, \"model\": \"FIELD-7's three-class wall, fit on its 24 FAR readings, evaluated on the {} CLOSE readings\", \"source\": \"{}\", \"a\": {p2_a_oo:.12e}, \"b\": {p2_b_oo:.12e}, \"a_oh\": {p2_a_oh:.12e}, \"b_oh\": {p2_b_oh:.12e}, \"a_hh\": {p2_a_hh:.12e}, \"b_hh\": {p2_b_hh:.12e}, \"failures\": {p2_fail}, \"of\": {}, \"failures_required\": {PLANT_II_MIN_FAIL}, \"bar_rule\": \"|wall − E_exch| > {PLANT_II_TOL_FRAC}·E_exch (§5 (ii)'s own rule, a pure fraction with no absolute floor)\", \"carrier\": {carrier_ii:.12e}, \"carrier_floor\": {PLANT_II_CARRIER_FLOOR}, \"carrier_present\": {carrier_ii_ok}, \"carrier_definition\": \"E_exch({PLANT_II_CARRIER_R:.1} Å, tilt {PLANT_II_CARRIER_TILT:.0}°), the undeformed referee — exchange at contact, the sector the plant acts on\", \"failure_details\": [{}]}},\n",
        close_idx.len(),
        w7path.display(),
        close_idx.len(),
        p2_fail_lines.join(", ")
    ));
    j.push_str(&format!(
        "  \"c1\": {{\"pass\": {c1}, \"within\": {c1_within}, \"of\": {}, \"required\": {C1_MIN}, \"close_nodes_within\": {c1_close_within}, \"close_nodes\": {c1_close_of}, \"close_nodes_ok\": {c1_close_ok}, \"tolerance_rule\": \"max({C1_FRAC}·|ΔE_exact|, {C1_ABS:e})\", \"misses\": [{}]}},\n",
        exact.len(),
        c1_miss_names.join(", ")
    ));
    j.push_str(&format!(
        "  \"contact_fit\": {{\"p_ho\": {p_ho:.12e}, \"c_ho\": {c_ho:.12e}, \"p_hh\": {p_hh:.12e}, \"c_hh\": {c_hh:.12e}, \"classes_kept\": {{\"ho\": {}, \"hh\": {}}}, \"weighted_residual\": {c_res:.12e}, \"grid\": \"0.50..=4.00 per bohr, step 0.02, per class\", \"grid_points_per_class\": {NC}, \"grid_pairs\": {}, \"placement\": \"cross-unit H–O and cross-unit H–H\", \"weights\": \"1/ΔE_exact²\", \"nonnegativity\": \"drop-and-refit, the most negative amplitude first\", \"points\": {}, \"remainder\": \"ΔE_exact − [E_q(g) − E_q(40)]_engine − wall(g), the NEW wall HELD at this harvest\", \"fit_seconds\": {cfit_seconds:.3}, \"nodes\": [\n    {}\n  ]}},\n",
        ckeep[0],
        ckeep[1],
        NC * NC,
        exact.len(),
        c1_lines.join(",\n    ")
    ));
    j.push_str(&format!(
        "  \"dispersion\": {{\"nodes\": \"the four outer linear nodes (R_OO ≥ {OUTER_FROM_ANGSTROM} Å), after both contact terms\", \"c6\": {c6:.12e}, \"transferred\": {c6_transferred}, \"slope_band\": [{SLOPE_LO}, {SLOPE_HI}], \"slopes\": [{}]}},\n",
        slope_lines.join(", ")
    ));
    j.push_str(&format!(
        "  \"g_n0\": {{\"pass\": {g_n0}, \"hole\": {hole_json}, \"q_h\": {q_h:.15e}, \"q_h_source\": \"holon_render::field::water_charge_at_pin\", \"rule\": \"SeamModel::hole walks each cross-unit class potential from 3.0 bohr inward to 0.5 on a 0.05 grid and names the first fall; none means the arms may run, a name means the arms are VOID before they run\", \"verdict\": \"{}\"}},\n",
        if g_n0 { "no hole — the arms may run" } else { "a fall named — the arms are VOID (§2 G-N0)" }
    ));
    j.push_str(&format!(
        "  \"g_c1\": {{\"pass\": {g_c1}, \"worst_miss\": {g_c1_worst:.6e}, \"tolerance\": {G_C1_TOL:e}, \"reference\": \"E_q(g) − E_q(40 bohr), the engine's own field on both sides\", \"points\": {}, \"units_and_pair_counts_ok\": {units_ok}, \"nodes\": [\n    {}\n  ]}},\n",
        exact.len(),
        g_c1_lines.join(",\n    ")
    ));
    j.push_str(&format!(
        "  \"plant_i\": {{\"miss_observed\": {:.6e}, \"miss_expected\": {:.6e}, \"carrier_p_ho\": {:.6e}, \"carrier_floor\": {PLANT_I_CARRIER:e}, \"fires\": {}, \"node\": \"linear_R{REF_ANGSTROM:.1}\", \"plant\": \"FlipPenetration (P_HO → −P_HO)\"}},\n",
        plant_i.0, plant_i.1, plant_i.2, plant_i.3
    ));
    j.push_str(&format!(
        "  \"price\": {{\"reading_price_core_seconds\": [{HL_PRICE_LO_CORE_S}, {HL_PRICE_HI_CORE_S}], \"tenth_of_price_core_seconds\": {HL_PRICE_TENTH_CORE_S}, \"every_new_reading_at_or_above\": {price_ok}, \"cheapest_new_core_seconds\": {:.3}, \"dearest_new_core_seconds\": {:.3}, \"exact_price_band_core_seconds\": [{S2_CPU_LO}, {S2_CPU_HI}]}},\n",
        new_cpu.iter().cloned().fold(f64::INFINITY, f64::min),
        new_cpu.iter().cloned().fold(0.0f64, f64::max)
    ));
    j.push_str(&format!(
        "  \"referee\": \"heitler_london_undeformed — the antisymmetrised product of the monomers' own wavefunctions, the monomers NOT deformed\",\n  \"reading_set\": {{\"tilt_family_r_angstrom\": [{}], \"twist_family_r_angstrom\": [{}], \"tilt_degrees\": [{}], \"twist_about_oo_axis_degrees\": {TWIST_ABOUT_Z_DEG:.1}, \"pivot\": \"the acceptor's own oxygen\", \"donor\": \"untouched\"}},\n",
        TILT_R.iter().map(|r| format!("{r:.1}")).collect::<Vec<_>>().join(", "),
        TWIST_R.iter().map(|r| format!("{r:.1}")).collect::<Vec<_>>().join(", "),
        TILT_DEG.iter().map(|t| format!("{t:.1}")).collect::<Vec<_>>().join(", ")
    ));
    j.push_str(&format!("  \"readings\": [\n    {}\n  ]\n}}\n", reading_lines.join(",\n    ")));
    fs::write(out.join("wall8.json"), j).unwrap();
    eprintln!("\nwall8.json written");

    // ------------------------------------ prediction.json, BEFORE the held-out solve
    let (a_p, b_p) = bent_donor(o, h, BENT_ANGSTROM, BENT_DEGREES);
    let r_oo_p = cross_oo(&a_p, &b_p);
    assert!(
        (r_oo_p - BENT_ANGSTROM * ANGSTROM_TO_BOHR).abs() < 1e-9,
        "the donor turns about its OWN oxygen: R_OO must be unchanged ({r_oo_p:.9} vs {:.9})",
        BENT_ANGSTROM * ANGSTROM_TO_BOHR
    );
    let (e_pred, e_q_p, e_seam_p) = engine_interaction(&a_p, &b_p, Some(model), SeamPlant::None);
    let tp = formula_terms(&a_p, &b_p, &model);
    let s_p = engine_dimer(&a_p, &b_p, Some(model), SeamPlant::None);
    let (oo_p, ho_p, hh_p) = cross_classes(&a_p, &b_p);
    fs::write(
        out.join("prediction.json"),
        format!(
            "{{\n  \"node\": \"bentdonor_R{BENT_ANGSTROM:.1}\", \"r_oo_angstrom\": {BENT_ANGSTROM:.3}, \"r_oo_bohr\": {r_oo_p:.6}, \"bend_degrees\": {BENT_DEGREES:.1}, \"bend_axis\": \"x, through the DONOR's own oxygen; the acceptor untouched\", \"held_out\": true, \"kind\": \"a bent DONOR — a kind no fit point contains (§4, M-UNTESTED-GAP)\",\n  \"e_pred\": {e_pred:+.12e},\n  \"parts\": {{\"e_q_difference\": {:+.12e}, \"contact_ho\": {:+.12e}, \"contact_hh\": {:+.12e}, \"wall_oo\": {:+.12e}, \"wall_oh\": {:+.12e}, \"wall_hh\": {:+.12e}, \"wall_total\": {:+.12e}, \"disp\": {:+.12e}, \"engine_seam\": {:+.12e}}},\n  \"coefficients\": {{\"a\": {a_oo:.12e}, \"b\": {b_oo:.12e}, \"p\": {p_ho:.12e}, \"c\": {c_ho:.12e}, \"c6\": {c6:.12e}, \"a_oh\": {a_oh:.12e}, \"b_oh\": {b_oh:.12e}, \"a_hh\": {a_hh:.12e}, \"b_hh\": {b_hh:.12e}, \"p_hh\": {p_hh:.12e}, \"c_hh\": {c_hh:.12e}}},\n  \"s1_branch\": \"{s1_branch}\", \"c1_pass\": {c1}, \"c6_transferred\": {c6_transferred}, \"g_n0_pass\": {g_n0}, \"g_c1_pass\": {g_c1}, \"units\": {}, \"oo_pairs\": {}, \"ho_pairs\": {},\n  \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\", \"tolerance_frac\": {PRED_FRAC}, \"tolerance_abs\": {PRED_ABS:e},\n  \"exact_solve_stake\": {{\"n_det\": {N_DET_DIMER}, \"cpu_seconds_lo\": {S2_CPU_LO}, \"cpu_seconds_hi\": {S2_CPU_HI}, \"residual_bar\": {RESIDUAL_BAR:e}, \"exit\": \"Converged\"}},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}],\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
            e_q_p,
            tp.pen_ho,
            tp.pen_hh,
            tp.w_oo,
            tp.w_oh,
            tp.w_hh,
            tp.wall_total(),
            tp.disp,
            e_seam_p,
            s_p.seam_work.units,
            s_p.seam_work.oo_pairs,
            s_p.seam_work.ho_pairs,
            list_json(&oo_p),
            list_json(&ho_p),
            list_json(&hh_p),
            centers_json(&a_p),
            centers_json(&b_p),
        ),
    )
    .unwrap();
    eprintln!(
        "prediction.json filed BEFORE the held-out solve: E_pred {e_pred:+.6e} Ha — E_q(g) − E_q(40) {e_q_p:+.6e}, contact_HO {:+.6e}, contact_HH {:+.6e}, wall_OO {:+.6e}, wall_OH {:+.6e}, wall_HH {:+.6e}, disp {:+.6e}; units {}",
        tp.pen_ho,
        tp.pen_hh,
        tp.w_oo,
        tp.w_oh,
        tp.w_hh,
        tp.disp,
        s_p.seam_work.units
    );
    fs::write(out.join("fit.done"), "done\n").unwrap();
}

// --------------------------------------------------------------------------- predict (S2)

fn run_predict(out: &Path) {
    let pred_path = out.join("prediction.json");
    let Ok(pred) = fs::read_to_string(&pred_path) else {
        eprintln!("{} missing: the prediction is filed BEFORE the solve (run `fit` first). Nothing written.", pred_path.display());
        std::process::exit(2);
    };
    let e_pred = json_num(&pred, "e_pred");
    assert!(e_pred.is_finite(), "prediction.json carries no e_pred");
    let wall8 = fs::read_to_string(out.join("wall8.json")).expect("wall8.json: run `fit` first");
    let (a_oo, b_oo) = (json_num(&wall8, "a"), json_num(&wall8, "b"));
    let (a_oh, b_oh) = (json_num(&wall8, "a_oh"), json_num(&wall8, "b_oh"));
    let (a_hh, b_hh) = (json_num(&wall8, "a_hh"), json_num(&wall8, "b_hh"));
    assert!(
        a_oo.is_finite() && b_oo.is_finite() && a_oh.is_finite() && b_oh.is_finite() && a_hh.is_finite() && b_hh.is_finite(),
        "wall8.json carries no three-class wall"
    );
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let (a, b) = bent_donor(o, h, BENT_ANGSTROM, BENT_DEGREES);
    let name = format!("bentdonor_R{BENT_ANGSTROM:.1}");
    eprintln!(
        "FIELD-8 predict — the HELD-OUT BENT-DONOR node ({BENT_DEGREES:.0}° about the x-axis through the DONOR's own oxygen, R_OO {BENT_ANGSTROM:.1} Å) on {} threads; E_pred {e_pred:+.12e} Ha",
        threads()
    );

    // the exact solve first: the prediction is already on disk
    let ok = solve_node(out, &name, BENT_ANGSTROM, &a, &b, true);
    let t = fs::read_to_string(out.join(format!("{name}.json"))).unwrap();
    let de = json_num(&t, "de_exact");
    let n_det = json_num(&t, "n_det") as usize;
    let cpu = json_num(&t, "cpu_seconds");
    let n_det_ok = n_det == N_DET_DIMER;
    let price_ok = cpu >= S2_CPU_LO && cpu <= S2_CPU_HI;
    let tol = (PRED_FRAC * de.abs()).max(PRED_ABS);
    let miss = (e_pred - de).abs();

    // then the referee on the same geometry: where the miss lives
    let (oo, ho, hh) = cross_classes(&a, &b);
    let r_oo_p = cross_oo(&a, &b);
    let t0 = Instant::now();
    let c0 = cpu_seconds();
    let hl = heitler_london_undeformed(&a, &b);
    let hl_wall = t0.elapsed().as_secs_f64();
    let hl_cpu = cpu_seconds() - c0;
    let w_oo: f64 = a_oo * sum_exp(&oo, b_oo);
    let w_oh: f64 = a_oh * sum_exp(&ho, b_oh);
    let w_hh: f64 = a_hh * sum_exp(&hh, b_hh);
    let wall_held = w_oo + w_oh + w_hh;
    let wall_gap = (wall_held - hl.e_exch).abs();
    let s2 = if miss <= tol {
        "a"
    } else if wall_gap <= tol {
        "b"
    } else {
        "c"
    };
    eprintln!(
        "S2: ΔE_exact {de:+.6e} Ha, E_pred {e_pred:+.6e} — miss {miss:.3e} ({:.1} % of |ΔE_exact|) against {tol:.3e}; E_exch(undeformed, bent donor) {:+.6e} vs the three-class wall {wall_held:+.6e} (O–O {w_oo:+.6e}, H–O {w_oh:+.6e}, H–H {w_hh:+.6e}), difference {wall_gap:.3e} → branch ({s2})",
        100.0 * miss / de.abs(),
        hl.e_exch
    );
    fs::write(
        out.join("prediction_check.json"),
        format!(
            "{{\n  \"node\": \"{name}\", \"e_pred\": {e_pred:+.12e}, \"de_exact\": {de:+.12e},\n  \"miss\": {miss:.6e}, \"miss_fraction\": {:.6}, \"tolerance\": {tol:.6e}, \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\",\n  \"s2_branch\": \"{s2}\",\n  \"exact\": {{\"converged\": {ok}, \"exit\": \"{}\", \"davidson_iters\": {}, \"residual\": {:.3e}, \"residual_bar\": {RESIDUAL_BAR:e}, \"n_det\": {n_det}, \"n_det_expected\": {N_DET_DIMER}, \"n_det_ok\": {n_det_ok}, \"cpu_seconds\": {cpu:.1}, \"cpu_seconds_lo\": {S2_CPU_LO}, \"cpu_seconds_hi\": {S2_CPU_HI}, \"price_in_band\": {price_ok}, \"wall_seconds\": {:.1}}},\n  \"exchange_on_the_held_out_node\": {{\"r_oo_bohr\": {r_oo_p:.6}, \"e_exch\": {:+.12e}, \"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det_a\": {}, \"n_det_b\": {}, \"product_dets\": {}, \"s_cross_max\": {:.6e}, \"sigma_seconds\": {:.3}, \"cpu_seconds\": {hl_cpu:.3}, \"wall_seconds\": {hl_wall:.3}}},\n  \"wall\": {{\"a\": {a_oo:.12e}, \"b\": {b_oo:.12e}, \"a_oh\": {a_oh:.12e}, \"b_oh\": {b_oh:.12e}, \"a_hh\": {a_hh:.12e}, \"b_hh\": {b_hh:.12e}, \"value\": {wall_held:+.12e}, \"oo\": {w_oo:+.12e}, \"oh\": {w_oh:+.12e}, \"hh\": {w_hh:+.12e}, \"minus_e_exch\": {:+.12e}, \"abs_difference\": {wall_gap:.6e}, \"within_tolerance\": {}}},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}]\n}}\n",
            miss / de.abs(),
            json_str(&t, "exit"),
            json_num(&t, "davidson_iters") as u64,
            json_num(&t, "residual"),
            json_num(&t, "wall_seconds"),
            hl.e_exch,
            hl.e_hl,
            hl.e_a0,
            hl.e_b0,
            hl.e_es,
            hl.norm,
            hl.nonzero_dets,
            hl.n_det_a,
            hl.n_det_b,
            hl.n_det_a * hl.n_det_b,
            hl.s_cross_max,
            hl.sigma_seconds,
            wall_held - hl.e_exch,
            wall_gap <= tol,
            list_json(&oo),
            list_json(&ho),
            list_json(&hh),
        ),
    )
    .unwrap();
    fs::write(out.join("predict.done"), "done\n").unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let what = args.get(1).map(String::as_str).unwrap_or("fit");
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/field8".to_string()));
    fs::create_dir_all(&out).expect("out");
    match what {
        "close" => run_close(&out),
        "fit" => run_fit(&out),
        "predict" => run_predict(&out),
        other => eprintln!("unknown phase {other} (close | fit | predict)"),
    }
}
