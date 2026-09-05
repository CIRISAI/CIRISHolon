//! FIELD-9's harvest (`conformance/water_observatory/FIELD9_PREREG.md` §0, §2, §5, §6): THE LAW
//! WHERE THE DYNAMICS GO. FIELD-8 read three things off its own record — one exponent per class
//! cannot span 2.1–3.4 Å within a typed twentieth, the closure identity has a boundary near
//! H···O 1.95 bohr, and a law that dips by a fraction of `kT` before rising is not a hole. This
//! freeze fits the wall on the 48 readings at `R_OO ≥ 2.5` Å (where the dynamics go), keeps the
//! 18 close readings for boundedness ALONE, DERIVES the wall's tolerance from the exponent's
//! measured drift across the two families instead of typing one, fits the contact terms only on
//! the twelve exact geometries INSIDE the closure identity (the 2.1 Å node is NOT a fit point),
//! and predicts a DOUBLY BENT dimer forward — both monomers turned, a kind no fit point has.
//!
//! No new exchange reading is taken here: the 66 are FIELD-7's 24 (`field7/orient_*.json`),
//! FIELD-8's 18 close tilts (`field8/orient_close_*.json`) and FIELD-8's 24 twists
//! (`field8/orient_twist_*.json`), each geometry REBUILT here and checked against its record's own
//! centers (M-STALE-INSTRUMENT). The twelve exact geometries are likewise all of record.
//!
//! ```text
//! cargo run --release -p holon-render --example field9_harvest -- fit     [OUT_DIR]
//! cargo run --release -p holon-render --example field9_harvest -- predict [OUT_DIR]
//! ```
//!
//! `fit`: D0 (written to `d0.json` BEFORE any fit runs), the three-class wall on the 48 (S1),
//! plant (ii), G-B0 by the engine's own boundedness walk, the two-class contact term on the
//! twelve (C1), dispersion, G-C1 and plant (i), `wall9.json`, and `prediction.json` for the doubly
//! bent dimer written BEFORE that node is solved. `predict` (detached): refuses without
//! `prediction.json`, solves the doubly bent node exactly, reads the undeformed referee on it
//! against the harvested wall, and writes `prediction_check.json` (S2).
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::{solve_embedded, supermolecule, water_dimer_linear, Fragment, ANGSTROM_TO_BOHR};
use holon_chem::fci::SolveExit;
use holon_chem::heitler_london::heitler_london_undeformed;
use holon_render::seam::{SeamModel, SeamPlant};
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/quartet.rs"]
#[allow(dead_code)]
mod quartet;

/// EMBED-1's water pins — the same numbers FIELD-3's … FIELD-8's runners carry.
const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;

/// The six acceptor tilts, every family.
const TILT_DEG: [f64; 6] = [0.0, 30.0, 60.0, 90.0, 120.0, 180.0];
/// The TILT family's separations, shortest first (FIELD-8's three close ones, FIELD-7's four).
const TILT_R: [f64; 7] = [2.1, 2.3, 2.5, 2.7, 2.9, 3.1, 3.4];
/// The tilt separations whose readings live in `field8/orient_close_*` rather than `field7/`.
const CLOSE_R: [f64; 3] = [2.1, 2.3, 2.5];
/// The TWIST family's separations: the acceptor turned 90° about the O···O axis, then tilted.
const TWIST_R: [f64; 4] = [2.3, 2.7, 3.0, 3.4];
/// The twist applied about the O···O axis before every twist-family tilt.
const TWIST_ABOUT_Z_DEG: f64 = 90.0;

const N_T: usize = TILT_DEG.len();
const N_TILT: usize = TILT_R.len() * N_T; // 42
const N_TWIST: usize = TWIST_R.len() * N_T; // 24
const N_READ: usize = N_TILT + N_TWIST; // 66

/// §0's split: a reading at or beyond this separation is FIT; the rest are BOUNDEDNESS-ONLY.
const FIT_FROM_ANGSTROM: f64 = 2.5;
/// The FIT set: tilt 2.5, 2.7, 2.9, 3.1, 3.4 (30) and twist 2.7, 3.0, 3.4 (18).
const N_FIT: usize = 48;
/// The BOUNDEDNESS-ONLY set: tilt 2.1, 2.3 (12) and twist 2.3 (6) — every reading at `R_OO ≤ 2.3`.
const N_BOUND: usize = N_READ - N_FIT; // 18

/// The SEVEN linear exact nodes inside the closure identity, shortest first. `linear_R2.1` is
/// OUTSIDE the identity (FIELD-8's H···O 1.95 bohr boundary) and is NOT a fit point (§0).
const LINEAR_ANGSTROM: [f64; 7] = [2.3, 2.5, 2.7, 2.9, 3.1, 3.4, 3.7];
/// The linear node of record that lives in `field8/` rather than `field3/`.
const LINEAR_FROM_FIELD8: f64 = 2.3;
/// A linear node at or beyond this separation is one of FIELD-6's four OUTER dispersion nodes.
const OUTER_FROM_ANGSTROM: f64 = 2.9;
/// C1's second leg: the line where the bond lives must be within tolerance at all three.
const LINE_ANGSTROM: [f64; 3] = [2.7, 2.9, 3.1];
/// The node plant (i) is read at.
const REF_ANGSTROM: f64 = 2.9;

/// The five non-linear exact geometries of record (§0).
const TILT5_ANGSTROM: f64 = 2.9;
const TILT5_DEGREES: f64 = 30.0;
const TILT6_ANGSTROM: f64 = 3.1;
const TILT6_DEGREES: f64 = 45.0;
const FLIPPED_ANGSTROM: f64 = 3.4;
const TWISTED_ANGSTROM: f64 = 3.0;
const TWISTED_TWIST_DEGREES: f64 = 90.0;
const TWISTED_TILT_DEGREES: f64 = 60.0;
const BENTDONOR_ANGSTROM: f64 = 2.9;
const BENTDONOR_DEGREES: f64 = 30.0;

/// S2's held-out geometry: the linear dimer at 2.9 Å with the DONOR rotated 30° about the x-axis
/// through ITS OWN oxygen AND the ACCEPTOR rotated 30° about the x-axis through its own. Both
/// monomers bent — the kind no fit point has (§4, M-UNTESTED-GAP).
const DOUBLE_ANGSTROM: f64 = 2.9;
const DOUBLE_DONOR_DEGREES: f64 = 30.0;
const DOUBLE_ACCEPTOR_DEGREES: f64 = 30.0;

/// The separation at which the acceptor is "away" (bohr): the engine's reference on both sides of
/// G-C1 and of every `E_q` difference.
const FAR_BOHR: f64 = 40.0;

/// The residual bar every exact solve must meet (EMBED-3's).
const RESIDUAL_BAR: f64 = 1e-9;
/// The reading floor on every harvested reading (M-FLOOR-UNSTAKED).
const R_FLOOR: f64 = 1e-6;

/// The three-exponent wall grid (per bohr): `0.50 ..= 4.00` step `0.05` — 71 values, 71³ triples.
const NB: usize = 71;
/// The contact term's `c`-grid, per class (per bohr): `0.50 ..= 4.00` step `0.02` — 176 values.
const NC: usize = 176;

/// The wall's tolerance is DERIVED in D0, not typed: `max(δ/2·E_exch, WALL_TOL_ABS)`. Only the
/// absolute floor is a constant here (M-FLOOR-UNSTAKED: the floor is stated, the fraction measured).
const WALL_TOL_ABS: f64 = 1e-4;
/// S1 (b)'s floor: at least this many of the 48 within the DERIVED tolerance (80 %), else (c) VOID.
const S1_B_MIN: usize = 38;

/// C1's tolerance, per exact point: `max(0.25·|ΔE_exact|, 5e-4)`; at least ten of twelve, and the
/// line at 2.7, 2.9, 3.1 Å all within.
const C1_FRAC: f64 = 0.25;
const C1_ABS: f64 = 5e-4;
const C1_MIN: usize = 10;

/// S2's tolerance: `max(0.25·|ΔE_exact|, 5e-4)`.
const PRED_FRAC: f64 = 0.25;
const PRED_ABS: f64 = 5e-4;

/// The band the remainder's log-log slope must lie in for `C₆` to transfer (FIELD-6's rule).
const SLOPE_LO: f64 = -8.0;
const SLOPE_HI: f64 = -4.0;

/// The undeformed product's overlap window, recorded on every reading.
const NORM_LO: f64 = 0.8;
const NORM_HI: f64 = 1.0;
/// The determinant count FIELD-3's supermolecule carries (EXACT).
const N_DET_DIMER: usize = 1_002_001;
/// The count of monomer-product determinants the undeformed state is BUILT from.
const PRODUCT_DETS_STAKED: usize = 194_481;

/// G-B0's temperature, in hartree: `kT` at 293 K (§0).
const KT_293: f64 = 9.28e-4;

/// G-C1's tolerance, and plant (i)'s carrier.
const G_C1_TOL: f64 = 1e-10;
const PLANT_I_CARRIER: f64 = 1e-4;
/// Plant (ii): the SAME 48-reading fit judged at FIELD-8's typed twentieth must place at least
/// this many MORE readings outside tolerance than the derived rule does; its carrier is the
/// derived fraction itself being the wider of the two.
const PLANT_II_MIN_EXTRA: usize = 5;
const PLANT_II_TYPED_FRAC: f64 = 0.05;
const PLANT_II_CARRIER_FLOOR: f64 = 0.06;

/// Every exact solve's price band (§2).
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

/// A string safe to sit inside one JSON value.
fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ------------------------------------------------------------------------ the geometries

/// FIELD-3's `linear` verbatim.
fn linear(o: Species, h: Species, r_oo_angstrom: f64) -> (Fragment, Fragment) {
    water_dimer_linear(o, h, H2O_R, H2O_THETA, r_oo_angstrom * ANGSTROM_TO_BOHR)
}

/// FIELD-5's rotation, in one place: a fragment turned by `theta_degrees` about the x-axis through
/// ITS OWN oxygen. The arithmetic is FIELD-5's `tilted` and FIELD-8's `bent_donor` verbatim, so a
/// geometry built through it is bit-identical to the one its record was measured on.
fn rot_x(f: &Fragment, theta_degrees: f64) -> Fragment {
    let oi = f.species.iter().position(|s| s.z == 8).expect("an oxygen");
    let origin = f.centers[oi];
    let th = theta_degrees * std::f64::consts::PI / 180.0;
    let (s, c) = (th.sin(), th.cos());
    let centers: Vec<[f64; 3]> = f
        .centers
        .iter()
        .map(|p| {
            let (x, y, z) = (p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]);
            [origin[0] + x, origin[1] + y * c - z * s, origin[2] + y * s + z * c]
        })
        .collect();
    Fragment::new(f.species.clone(), centers, f.weights.clone())
}

/// FIELD-3's FLIPPED dimer verbatim: the linear donor, the acceptor rotated by π about the x-axis
/// through its oxygen. FIELD-4 solved it exactly at 3.4 Å.
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

/// FIELD-5's `tilted`: the linear dimer with the ACCEPTOR rotated by `theta_degrees` about the
/// x-axis through its OWN oxygen. The donor is untouched and `R_OO` is unchanged.
fn tilted(o: Species, h: Species, r_oo_angstrom: f64, theta_degrees: f64) -> (Fragment, Fragment) {
    let (donor, acc) = linear(o, h, r_oo_angstrom);
    let a = rot_x(&acc, theta_degrees);
    (donor, a)
}

/// FIELD-7's `twisted` verbatim, generalised to any tilt: the acceptor rotated by `twist_degrees`
/// about the O···O axis (z) through its own oxygen, and THEN tilted by `tilt_degrees` about its own
/// x-axis. Both rotations fix the acceptor's oxygen, so `R_OO` is unchanged.
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

/// FIELD-8's held-out geometry, now a fit point: the DONOR rotated about the x-axis through its own
/// oxygen, the acceptor untouched.
fn bent_donor(o: Species, h: Species, r_oo_angstrom: f64, theta_degrees: f64) -> (Fragment, Fragment) {
    let (donor, acc) = linear(o, h, r_oo_angstrom);
    (rot_x(&donor, theta_degrees), acc)
}

/// S2's held-out geometry: BOTH monomers turned about the x-axis through their OWN oxygens. Both
/// pivots are the oxygens, so `R_OO` is unchanged.
fn double_bent(o: Species, h: Species, r_oo_angstrom: f64, donor_degrees: f64, acceptor_degrees: f64) -> (Fragment, Fragment) {
    let (donor, acc) = linear(o, h, r_oo_angstrom);
    (rot_x(&donor, donor_degrees), rot_x(&acc, acceptor_degrees))
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

/// Every CROSS-UNIT pair distance (bohr) by class: `(O–O, H–O, H–H)`.
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

fn vmin(v: &[f64]) -> f64 {
    v.iter().cloned().fold(f64::INFINITY, f64::min)
}

// ------------------------------------------------------------------------------ the engine

/// FIELD-4's `engine_dimer` verbatim: an open box, the field on with the pin charge, the seam model
/// and its plant installed, forces computed once so the closure assignment and the rows are read.
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

/// `E(geometry) − E(acceptor moved 40 bohr along x)` on the rows the seam law serves between units:
/// the total, the FIELD part, the SEAM part. The FIELD part is exactly the freeze's
/// `E_q(g) − E_q(40)` — the SAME reference the formula side of G-C1 uses.
fn engine_interaction(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, plant: SeamPlant) -> (f64, f64, f64) {
    let s = engine_dimer(a, b, seam, plant);
    let near = (s.e_pair + s.e_three) + s.e_field + s.e_seam;
    let far_b = b.translated([FAR_BOHR, 0.0, 0.0]);
    let f = engine_dimer(a, &far_b, seam, plant);
    let far = (f.e_pair + f.e_three) + f.e_field + f.e_seam;
    (near - far, s.e_field - f.e_field, s.e_seam - f.e_seam)
}

/// The formula side of G-C1, class by class.
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
        pen_hh: hh.iter().map(|&r| m.contact_hh(r)).sum(),
        w_oo: oo.iter().map(|&r| m.wall(r)).sum(),
        w_oh: ho.iter().map(|&r| m.wall_oh(r)).sum(),
        w_hh: hh.iter().map(|&r| m.wall_hh(r)).sum(),
        disp: oo.iter().map(|&r| m.dispersion(r)).sum(),
    }
}

// ------------------------------------------------------------------------- the exact solve

/// One exact node: the supermolecule, the monomer references, the record (FIELD-3's `solve_node`).
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
/// elimination with partial pivoting. `None` when the active block is singular.
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

/// §0's non-negativity rule on three classes: drop the most negative amplitude to an exact `0.0`
/// and refit, one class per round. Returns `(amplitudes, classes kept, weighted residual)`.
fn fit_nonneg(a: &[[f64; 3]; 3], v: &[f64; 3], syy: f64) -> ([f64; 3], [bool; 3], f64) {
    let mut active = [true; 3];
    for c in 0..3 {
        if !(a[c][c] > 0.0) {
            active[c] = false;
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

/// The same rule on TWO classes (the contact term's `P_HO`, `P_HH`).
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

/// One of the 66 exchange readings, every one of them a RECORD (no reading is taken here).
struct Reading {
    family: &'static str,
    fit_set: bool,
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

/// The record file every one of the 66 lives in: the close tilts and every twist in `field8/`,
/// FIELD-7's four far tilt separations in `field7/`.
fn reading_path(out: &Path, family: &str, r: f64, deg: f64) -> PathBuf {
    if family == "twist" {
        sibling(out, "field8").join(format!("orient_twist_R{r:.1}_t{deg:.0}.json"))
    } else if CLOSE_R.iter().any(|&c| (c - r).abs() < 1e-9) {
        sibling(out, "field8").join(format!("orient_close_R{r:.1}_t{deg:.0}.json"))
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

/// Read all 66 readings from disk, each geometry REBUILT here and checked against its record's own
/// centers (M-STALE-INSTRUMENT). The cross-unit distances come from the rebuilt geometry.
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
            fit_set: r >= FIT_FROM_ANGSTROM - 1e-9,
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
    line_node: bool,
    source: String,
}

/// The TWELVE exact geometries INSIDE the closure identity (§0), each rebuilt here and CHECKED
/// against the record it is read from (M-STALE-INSTRUMENT): the line at 2.3–3.7 Å (seven nodes —
/// `linear_R2.1` is outside the identity and is NOT among them), FIELD-5's 30°-bent acceptor,
/// FIELD-6's 45°-bent acceptor, FIELD-4's flipped dimer, FIELD-7's twisted dimer, FIELD-8's bent
/// donor.
fn exact_records(out: &Path, o: Species, h: Species) -> Result<Vec<ENode>, Vec<String>> {
    let f3 = sibling(out, "field3");
    let f4 = sibling(out, "field4");
    let f5 = sibling(out, "field5");
    let f6 = sibling(out, "field6");
    let f7 = sibling(out, "field7");
    let f8 = sibling(out, "field8");
    let mut want: Vec<(String, &'static str, f64, PathBuf, Fragment, Fragment, bool, bool)> = Vec::new();
    for &r in LINEAR_ANGSTROM.iter() {
        let (a, b) = linear(o, h, r);
        let dir = if (r - LINEAR_FROM_FIELD8).abs() < 1e-9 { f8.clone() } else { f3.clone() };
        let line = LINE_ANGSTROM.iter().any(|&l| (l - r).abs() < 1e-9);
        want.push((format!("linear_R{r:.1}"), "linear", r, dir.join(format!("linear_R{r:.1}.json")), a, b, r >= OUTER_FROM_ANGSTROM - 1e-9, line));
    }
    let (a5, b5) = tilted(o, h, TILT5_ANGSTROM, TILT5_DEGREES);
    want.push((format!("tilted_R{TILT5_ANGSTROM:.1}"), "bent acceptor 30°", TILT5_ANGSTROM, f5.join(format!("tilted_R{TILT5_ANGSTROM:.1}.json")), a5, b5, false, false));
    let (a6, b6) = tilted(o, h, TILT6_ANGSTROM, TILT6_DEGREES);
    want.push((
        format!("tilted{TILT6_DEGREES:.0}_R{TILT6_ANGSTROM:.1}"),
        "bent acceptor 45°",
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
    let (a8, b8) = bent_donor(o, h, BENTDONOR_ANGSTROM, BENTDONOR_DEGREES);
    want.push((
        format!("bentdonor_R{BENTDONOR_ANGSTROM:.1}"),
        "bent donor 30°",
        BENTDONOR_ANGSTROM,
        f8.join(format!("bentdonor_R{BENTDONOR_ANGSTROM:.1}.json")),
        a8,
        b8,
        false,
        false,
    ));

    let mut missing: Vec<String> = Vec::new();
    let mut nodes: Vec<ENode> = Vec::new();
    for (name, kind, r_ang, path, a, b, outer, line) in want {
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
            line_node: line,
            source: path.display().to_string(),
        });
    }
    if missing.is_empty() {
        Ok(nodes)
    } else {
        Err(missing)
    }
}

// ------------------------------------------------------------------- the three-class wall fit

struct Wall3 {
    b_oo: f64,
    b_oh: f64,
    b_hh: f64,
    a_oo: f64,
    a_oh: f64,
    a_hh: f64,
    kept: [bool; 3],
    residual: f64,
    seconds: f64,
    n: usize,
}

impl Wall3 {
    fn value(&self, oo: &[f64], ho: &[f64], hh: &[f64]) -> f64 {
        self.a_oo * sum_exp(oo, self.b_oo) + self.a_oh * sum_exp(ho, self.b_oh) + self.a_hh * sum_exp(hh, self.b_hh)
    }
}

/// FIELD-7's three-class procedure: `71³` exponent triples on `0.50 ..= 4.00` step `0.05` per
/// class, weighted `1/E_exch²` least squares, amplitudes non-negative by drop-and-refit.
fn fit_wall3(rs: &[&Reading]) -> Wall3 {
    let n = rs.len();
    let y: Vec<f64> = rs.iter().map(|r| r.e_exch).collect();
    let w: Vec<f64> = y.iter().map(|v| 1.0 / (v * v)).collect();
    let syy: f64 = (0..n).map(|g| w[g] * y[g] * y[g]).sum();
    let mut soo = vec![0.0f64; NB * n];
    let mut soh = vec![0.0f64; NB * n];
    let mut shh = vec![0.0f64; NB * n];
    for i in 0..NB {
        let b = bgrid(i);
        for (g, r) in rs.iter().enumerate() {
            soo[i * n + g] = sum_exp(&r.oo, b);
            soh[i * n + g] = sum_exp(&r.ho, b);
            shh[i * n + g] = sum_exp(&r.hh, b);
        }
    }
    let mut a00 = vec![0.0f64; NB];
    let mut a11 = vec![0.0f64; NB];
    let mut a22 = vec![0.0f64; NB];
    let mut v0 = vec![0.0f64; NB];
    let mut v1 = vec![0.0f64; NB];
    let mut v2 = vec![0.0f64; NB];
    for i in 0..NB {
        for g in 0..n {
            let (p, q, s) = (soo[i * n + g], soh[i * n + g], shh[i * n + g]);
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
            for g in 0..n {
                x01 += w[g] * soo[i * n + g] * soh[j * n + g];
                x02 += w[g] * soo[i * n + g] * shh[j * n + g];
                x12 += w[g] * soh[i * n + g] * shh[j * n + g];
            }
            a01[i * NB + j] = x01;
            a02[i * NB + j] = x02;
            a12[i * NB + j] = x12;
        }
    }
    let t0 = Instant::now();
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
    Wall3 {
        b_oo: bgrid(bi),
        b_oh: bgrid(bj),
        b_hh: bgrid(bk),
        a_oo: amp[0],
        a_oh: amp[1],
        a_hh: amp[2],
        kept: act,
        residual: best_r,
        seconds: t0.elapsed().as_secs_f64(),
        n,
    }
}

/// The log-linear exponent of `E_exch` against `R_OO` (per bohr): `E = A·e^{−b·R}`, `b` the
/// negated ordinary-least-squares slope of `ln E` on `R`. `None` when the row is too short or a
/// reading is not positive.
fn loglin_exponent(pts: &[(f64, f64)]) -> Option<f64> {
    if pts.len() < 2 || pts.iter().any(|&(_, e)| !(e > 0.0)) {
        return None;
    }
    let n = pts.len() as f64;
    let mx = pts.iter().map(|p| p.0).sum::<f64>() / n;
    let my = pts.iter().map(|p| p.1.ln()).sum::<f64>() / n;
    let (mut num, mut den) = (0.0f64, 0.0f64);
    for &(x, e) in pts {
        num += (x - mx) * (e.ln() - my);
        den += (x - mx) * (x - mx);
    }
    if !(den > 0.0) {
        return None;
    }
    Some(-num / den)
}

/// The relative spread of two numbers about their mean.
fn rel_spread2(x: f64, y: f64) -> f64 {
    let m = 0.5 * (x + y);
    if m.abs() > 0.0 {
        (x - y).abs() / m.abs()
    } else {
        f64::NAN
    }
}

// --------------------------------------------------------------------------- the fit phase

fn run_fit(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    eprintln!("FIELD-9 fit — D0 (the drift, declared first), the three-class wall on the {N_FIT} readings at R_OO ≥ {FIT_FROM_ANGSTROM} Å (S1), plant (ii), G-B0, the two-class contact term on the twelve exact geometries inside the identity (C1), dispersion, G-C1, plant (i)");

    let exact = match exact_records(out, o, h) {
        Ok(v) => v,
        Err(missing) => {
            eprintln!("REFUSED — the contact term is staked on TWELVE exact geometries inside the identity (§0) and these are not on disk:");
            for m in &missing {
                eprintln!("  {m}");
            }
            eprintln!("Nothing written.");
            std::process::exit(3);
        }
    };
    assert_eq!(exact.len(), 12, "the exact record is twelve geometries INSIDE the identity (linear_R2.1 is not among them)");
    let readings = match load_readings(out, o, h) {
        Ok(v) => v,
        Err(missing) => {
            eprintln!("REFUSED — the reading set is {N_READ} records (§0) and these are not on disk:");
            for m in &missing {
                eprintln!("  {m}");
            }
            eprintln!("Nothing written.");
            std::process::exit(3);
        }
    };
    assert_eq!(readings.len(), N_READ, "the reading set is 66");

    let fit_idx: Vec<usize> = (0..N_READ).filter(|&g| readings[g].fit_set).collect();
    let bound_idx: Vec<usize> = (0..N_READ).filter(|&g| !readings[g].fit_set).collect();
    assert_eq!(fit_idx.len(), N_FIT, "the FIT set is the 48 readings at R_OO ≥ 2.5 Å");
    assert_eq!(bound_idx.len(), N_BOUND, "the BOUNDEDNESS-ONLY set is the 18 readings at R_OO ≤ 2.3 Å");
    let fit_rs: Vec<&Reading> = fit_idx.iter().map(|&g| &readings[g]).collect();
    let tilt_fit: Vec<&Reading> = fit_rs.iter().filter(|r| r.family == "tilt").cloned().collect();
    let twist_fit: Vec<&Reading> = fit_rs.iter().filter(|r| r.family == "twist").cloned().collect();
    eprintln!(
        "the reading set: {N_READ} records, none measured here.\n  FIT ({N_FIT}, R_OO ≥ {FIT_FROM_ANGSTROM} Å): tilt {} (2.5, 2.7, 2.9, 3.1, 3.4 Å × 6 tilts) + twist {} (2.7, 3.0, 3.4 Å × 6)\n  BOUNDEDNESS-ONLY ({N_BOUND}, R_OO ≤ 2.3 Å): tilt 12 (2.1, 2.3 Å × 6) + twist 6 (2.3 Å × 6) — the twist 2.3 Å row belongs to the bound set and is included in it",
        tilt_fit.len(),
        twist_fit.len()
    );
    eprintln!("the twelve exact geometries of record, each rebuilt here and matched to its record's centers:");
    for e in &exact {
        eprintln!("  {} ({}): R_OO {:.4} bohr, ΔE_exact {:.6e} Ha  [{}]", e.name, e.kind, e.r_oo_bohr, e.de_exact, e.source);
    }

    // ------------------------------------- the readings are what they say (a reading, recorded)
    let norm_ok = readings.iter().all(|n| n.norm > NORM_LO && n.norm <= NORM_HI);
    let floor_ok = readings.iter().all(|n| n.e_exch > R_FLOOR);
    let norm_lo_seen = readings.iter().map(|n| n.norm).fold(f64::INFINITY, f64::min);
    let norm_hi_seen = readings.iter().map(|n| n.norm).fold(f64::NEG_INFINITY, f64::max);
    let mut monotone_breaks = 0usize;
    let at_tilt = |ri: usize, ti: usize| -> &Reading { &readings[ri * N_T + ti] };
    let at_twist = |ri: usize, ti: usize| -> &Reading { &readings[N_TILT + ri * N_T + ti] };
    for ti in 0..N_T {
        for ri in 1..TILT_R.len() {
            if !(at_tilt(ri, ti).e_exch <= at_tilt(ri - 1, ti).e_exch) {
                monotone_breaks += 1;
            }
        }
        for ri in 1..TWIST_R.len() {
            if !(at_twist(ri, ti).e_exch <= at_twist(ri - 1, ti).e_exch) {
                monotone_breaks += 1;
            }
        }
    }
    eprintln!(
        "\nthe readings, as records (a reading, not a gate): overlap in ({NORM_LO}, {NORM_HI}] {norm_ok} (lowest {norm_lo_seen:.12}, highest {norm_hi_seen:.12}); E_exch > {R_FLOOR:e} everywhere {floor_ok}; E_exch non-increasing in R_OO along each (family, tilt): {monotone_breaks} breaks"
    );

    // ================================================================== D0 — the drift, declared
    eprintln!("\nD0 — the drift, measured and DECLARED before any wall is fit (§0, §2 D0).");
    eprintln!("  (a) the SINGLE-class log-linear exponent of E_exch against R_OO, per family, per tilt — 6 × 2 = 12:");
    let mut singles: Vec<(&'static str, f64, f64, usize)> = Vec::new();
    for (fam, rs) in [("tilt", &tilt_fit), ("twist", &twist_fit)] {
        for &deg in TILT_DEG.iter() {
            let pts: Vec<(f64, f64)> = rs.iter().filter(|r| (r.tilt_deg - deg).abs() < 1e-9).map(|r| (r.r_oo_bohr, r.e_exch)).collect();
            let b = loglin_exponent(&pts).unwrap_or(f64::NAN);
            eprintln!("    {fam:>5} family, tilt {deg:>5.1}°: b = {b:.6} /bohr  ({} points)", pts.len());
            singles.push((fam, deg, b, pts.len()));
        }
    }
    let sv: Vec<f64> = singles.iter().map(|s| s.2).collect();
    assert!(sv.iter().all(|x| x.is_finite()), "every one of the 12 single-class exponents must be finite");
    let s_max = sv.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let s_min = sv.iter().cloned().fold(f64::INFINITY, f64::min);
    let s_mean = sv.iter().sum::<f64>() / (sv.len() as f64);
    let delta_single = (s_max - s_min) / s_mean;
    eprintln!("    max {s_max:.6}, min {s_min:.6}, mean {s_mean:.6} → relative spread δ_single = {delta_single:.6}");

    eprintln!("  (b) the THREE-class wall fit on each family alone, and the per-class spread between them:");
    let w_tilt = fit_wall3(&tilt_fit);
    let w_twist = fit_wall3(&twist_fit);
    for (fam, w) in [("tilt", &w_tilt), ("twist", &w_twist)] {
        eprintln!(
            "    {fam:>5} family ({} readings, {:.1} s): b_OO {:.2} (A {:.6e}, kept {}), b_OH {:.2} (A {:.6e}, kept {}), b_HH {:.2} (A {:.6e}, kept {}); weighted residual {:.6e}",
            w.n, w.seconds, w.b_oo, w.a_oo, w.kept[0], w.b_oh, w.a_oh, w.kept[1], w.b_hh, w.a_hh, w.kept[2], w.residual
        );
    }
    let class_names = ["OO", "OH", "HH"];
    let tb = [w_tilt.b_oo, w_tilt.b_oh, w_tilt.b_hh];
    let wb = [w_twist.b_oo, w_twist.b_oh, w_twist.b_hh];
    let mut class_spreads: Vec<(usize, f64, bool)> = Vec::new();
    for c in 0..3 {
        let both = w_tilt.kept[c] && w_twist.kept[c];
        let s = rel_spread2(tb[c], wb[c]);
        eprintln!(
            "    class {}: b_tilt {:.2}, b_twist {:.2} → relative spread {s:.6}{}",
            class_names[c],
            tb[c],
            wb[c],
            if both { "" } else { "  (NOT counted: the class is dropped in at least one family's fit)" }
        );
        class_spreads.push((c, s, both));
    }
    let counted: Vec<&(usize, f64, bool)> = class_spreads.iter().filter(|x| x.2 && x.1.is_finite()).collect();
    let delta_class = counted.iter().map(|x| x.1).fold(f64::NEG_INFINITY, f64::max);
    let delta_class = if counted.is_empty() { 0.0 } else { delta_class };
    let delta = delta_single.max(delta_class);
    let tol_frac = 0.5 * delta;
    eprintln!(
        "  δ_single = {delta_single:.6}, δ_class = {delta_class:.6} (over the {} classes kept in both families) → δ = {delta:.6}",
        counted.len()
    );
    eprintln!("  D0 → the wall's tolerance rule is per reading  max({tol_frac:.6}·E_exch, {WALL_TOL_ABS:e})  —  δ/2 = {tol_frac:.6}");

    let single_lines: Vec<String> = singles
        .iter()
        .map(|(f, d, b, n)| format!("{{\"family\": \"{f}\", \"tilt_degrees\": {d:.1}, \"exponent_per_bohr\": {b:.9}, \"points\": {n}}}"))
        .collect();
    let class_lines: Vec<String> = class_spreads
        .iter()
        .map(|(c, s, both)| {
            format!(
                "{{\"class\": \"{}\", \"exponent_tilt_family\": {:.9}, \"exponent_twist_family\": {:.9}, \"relative_spread\": {:.9}, \"kept_in_both\": {both}, \"counted\": {}}}",
                class_names[*c],
                tb[*c],
                wb[*c],
                *s,
                *both && s.is_finite()
            )
        })
        .collect();
    let d0_json = format!(
        "{{\n  \"declared\": \"BEFORE the wall fit (§2 D0): the exponent's drift measured across the two families, and the tolerance rule that follows from it\",\n  \"fit_set\": {{\"count\": {N_FIT}, \"rule\": \"R_OO ≥ {FIT_FROM_ANGSTROM} Å\", \"tilt_family\": {}, \"twist_family\": {}}},\n  \"boundedness_only_set\": {{\"count\": {N_BOUND}, \"rule\": \"R_OO ≤ 2.3 Å — the 12 tilt readings at 2.1 and 2.3 Å AND the 6 twist readings at 2.3 Å; these enter G-B0 only and are never fit\", \"tilt_family\": 12, \"twist_family\": 6}},\n  \"single_class\": {{\"rule\": \"E_exch = A·e^(−b·R_OO), b the negated OLS slope of ln E_exch on R_OO in bohr, fit per family per tilt on the FIT set alone\", \"exponents\": [{}], \"max\": {s_max:.9}, \"min\": {s_min:.9}, \"mean\": {s_mean:.9}, \"relative_spread\": {delta_single:.9}}},\n  \"three_class\": {{\"rule\": \"FIELD-7's three-class wall fit on each family's FIT readings alone; the per-class exponents and their relative spread between the two families\", \"tilt_family\": {{\"readings\": {}, \"b_oo\": {:.9}, \"b_oh\": {:.9}, \"b_hh\": {:.9}, \"amp_oo\": {:.12e}, \"amp_oh\": {:.12e}, \"amp_hh\": {:.12e}, \"kept_oo\": {}, \"kept_oh\": {}, \"kept_hh\": {}, \"weighted_residual\": {:.12e}}}, \"twist_family\": {{\"readings\": {}, \"b_oo\": {:.9}, \"b_oh\": {:.9}, \"b_hh\": {:.9}, \"amp_oo\": {:.12e}, \"amp_oh\": {:.12e}, \"amp_hh\": {:.12e}, \"kept_oo\": {}, \"kept_oh\": {}, \"kept_hh\": {}, \"weighted_residual\": {:.12e}}}, \"per_class\": [{}], \"largest_counted_spread\": {delta_class:.9}}},\n  \"delta\": {delta:.9}, \"delta_rule\": \"the LARGER of the single-class spread over the 12 and the largest per-class spread across the two families\",\n  \"tolerance_fraction\": {tol_frac:.9}, \"tolerance_rule\": \"max(δ/2·E_exch, {WALL_TOL_ABS:e}) per reading\", \"tolerance_absolute_floor\": {WALL_TOL_ABS:e}\n}}\n",
        tilt_fit.len(),
        twist_fit.len(),
        single_lines.join(", "),
        w_tilt.n,
        w_tilt.b_oo,
        w_tilt.b_oh,
        w_tilt.b_hh,
        w_tilt.a_oo,
        w_tilt.a_oh,
        w_tilt.a_hh,
        w_tilt.kept[0],
        w_tilt.kept[1],
        w_tilt.kept[2],
        w_tilt.residual,
        w_twist.n,
        w_twist.b_oo,
        w_twist.b_oh,
        w_twist.b_hh,
        w_twist.a_oo,
        w_twist.a_oh,
        w_twist.a_hh,
        w_twist.kept[0],
        w_twist.kept[1],
        w_twist.kept[2],
        w_twist.residual,
        class_lines.join(", ")
    );
    fs::write(out.join("d0.json"), &d0_json).unwrap();
    eprintln!("d0.json written BEFORE the wall fit");

    // ============================================== the three-class wall on the 48 FIT readings
    let wall = fit_wall3(&fit_rs);
    let (a_oo, a_oh, a_hh) = (wall.a_oo, wall.a_oh, wall.a_hh);
    let (b_oo, b_oh, b_hh) = (wall.b_oo, wall.b_oh, wall.b_hh);
    eprintln!(
        "\nthe three-class wall over the {N_FIT} FIT readings — {} triples on 0.50..=4.00 step 0.05, weighted (1/E_exch²) least squares, amplitudes non-negative, in {:.1} s:",
        NB * NB * NB,
        wall.seconds
    );
    eprintln!("  b_OO = {b_oo:.2} /bohr, A_OO = {a_oo:.9e} Ha (kept: {})", wall.kept[0]);
    eprintln!("  b_OH = {b_oh:.2} /bohr, A_OH = {a_oh:.9e} Ha (kept: {})", wall.kept[1]);
    eprintln!("  b_HH = {b_hh:.2} /bohr, A_HH = {a_hh:.9e} Ha (kept: {})", wall.kept[2]);
    eprintln!("  weighted residual {:.9e}", wall.residual);

    let mut within = 0usize;
    let mut misses: Vec<String> = Vec::new();
    let mut miss_lines: Vec<String> = Vec::new();
    let mut typed_outside = 0usize;
    let mut derived_outside = 0usize;
    let mut typed_lines: Vec<String> = Vec::new();
    eprintln!("\n| family | R (Å) | tilt (°) | E_exch (Ha) | model (Ha) | miss (Ha) | miss/tol | within | typed 0.05·E |");
    for &g in fit_idx.iter() {
        let n = &readings[g];
        let m = wall.value(&n.oo, &n.ho, &n.hh);
        let miss = (m - n.e_exch).abs();
        let tol = (tol_frac * n.e_exch).max(WALL_TOL_ABS);
        let ok = miss <= tol;
        let typed_ok = miss <= PLANT_II_TYPED_FRAC * n.e_exch;
        if ok {
            within += 1;
        } else {
            derived_outside += 1;
            misses.push(format!("({}, R = {:.1} Å, tilt = {:.0}°)", n.family, n.r_ang, n.tilt_deg));
            miss_lines.push(format!(
                "{{\"family\": \"{}\", \"r_angstrom\": {:.1}, \"tilt_degrees\": {:.1}, \"e_exch\": {:.12e}, \"model\": {:.12e}, \"miss\": {miss:.12e}, \"tolerance\": {tol:.12e}, \"miss_over_tolerance\": {:.6}}}",
                n.family,
                n.r_ang,
                n.tilt_deg,
                n.e_exch,
                m,
                miss / tol
            ));
        }
        if !typed_ok {
            typed_outside += 1;
            typed_lines.push(format!(
                "{{\"family\": \"{}\", \"r_angstrom\": {:.1}, \"tilt_degrees\": {:.1}, \"e_exch\": {:.12e}, \"model\": {:.12e}, \"miss\": {miss:.12e}, \"typed_bar\": {:.12e}}}",
                n.family,
                n.r_ang,
                n.tilt_deg,
                n.e_exch,
                m,
                PLANT_II_TYPED_FRAC * n.e_exch
            ));
        }
        eprintln!(
            "| {} | {:.1} | {:.0} | {:.6e} | {:.6e} | {:.6e} | {:.4} | {} | {} |",
            n.family,
            n.r_ang,
            n.tilt_deg,
            n.e_exch,
            m,
            m - n.e_exch,
            miss / tol,
            ok,
            typed_ok
        );
    }
    let s1_branch = if within == N_FIT {
        "a"
    } else if within >= S1_B_MIN {
        "b"
    } else {
        "c"
    };
    eprintln!(
        "S1: {within} of {N_FIT} within the DERIVED tolerance max({tol_frac:.6}·E_exch, {WALL_TOL_ABS:e}) → branch ({s1_branch}) — {}",
        match s1_branch {
            "a" => "the three-class wall carries exchange across the dynamics' range at the drift-derived level".to_string(),
            "b" => format!("transferred, the {} misses reported: {}", misses.len(), misses.join(", ")),
            _ => format!("VOID: the arms do not run. {} misses: {}", misses.len(), misses.join(", ")),
        }
    );

    // ------------------ plant (ii): the SAME fit judged at FIELD-8's typed twentieth
    let extra = typed_outside.saturating_sub(derived_outside);
    let carrier_ii_ok = tol_frac >= PLANT_II_CARRIER_FLOOR;
    let plant_ii_fires = extra >= PLANT_II_MIN_EXTRA && carrier_ii_ok;
    eprintln!(
        "\nplant (ii) — the SAME {N_FIT}-reading fit judged at FIELD-8's typed twentieth ({PLANT_II_TYPED_FRAC}·E_exch, a pure fraction with no absolute floor): {typed_outside} outside, against {derived_outside} outside the derived rule — {extra} more (needs ≥ {PLANT_II_MIN_EXTRA}); carrier δ/2 = {tol_frac:.6} ≥ {PLANT_II_CARRIER_FLOOR}: {carrier_ii_ok} → {}",
        if plant_ii_fires { "FIRES" } else { "does not fire" }
    );

    // ------------------------------------- G-B0's r_min, from the FIT geometries (geometry only)
    let r_min_oo = fit_rs.iter().map(|r| vmin(&r.oo)).fold(f64::INFINITY, f64::min);
    let r_min_oh = fit_rs.iter().map(|r| vmin(&r.ho)).fold(f64::INFINITY, f64::min);
    let r_min_hh = fit_rs.iter().map(|r| vmin(&r.hh)).fold(f64::INFINITY, f64::min);
    let arg_min = |class: usize, target: f64| -> String {
        for r in fit_rs.iter() {
            let v = match class {
                0 => vmin(&r.oo),
                1 => vmin(&r.ho),
                _ => vmin(&r.hh),
            };
            if (v - target).abs() < 1e-12 {
                return format!("{} family, R_OO {:.1} Å, tilt {:.0}°", r.family, r.r_ang, r.tilt_deg);
            }
        }
        "unattributed".to_string()
    };
    let who_oo = arg_min(0, r_min_oo);
    let who_oh = arg_min(1, r_min_oh);
    let who_hh = arg_min(2, r_min_hh);
    eprintln!(
        "\nG-B0's r_min — the SHORTEST cross-unit distance of each class among the {N_FIT} FIT geometries (geometry alone; the walk itself runs once the contact terms and dispersion are harvested):\n  O–O {r_min_oo:.6} bohr  [{who_oo}]\n  H–O {r_min_oh:.6} bohr  [{who_oh}]\n  H–H {r_min_hh:.6} bohr  [{who_hh}]"
    );

    // ---------------------- the twelve are served with TWO units, or the contact fit is refused
    let mut not_two: Vec<String> = Vec::new();
    for e in &exact {
        let s = engine_dimer(&e.a, &e.b, None, SeamPlant::None);
        if s.seam_work.units != 2 {
            not_two.push(format!("{} (the engine assigns {} units, not 2)", e.name, s.seam_work.units));
        }
    }
    if !not_two.is_empty() {
        eprintln!("REFUSED — the contact term is fit only on geometries INSIDE the closure identity, and these are not served with two units:");
        for m in &not_two {
            eprintln!("  {m}");
        }
        eprintln!("d0.json stands; nothing else written.");
        std::process::exit(4);
    }
    eprintln!("every one of the twelve is served with TWO units by the engine (the closure identity holds at each) — the fit may use them");

    // ---------------- the contact term on TWO classes, with the wall HELD, on the twelve
    eprintln!("\nthe contact term on TWO classes with the wall HELD: remainder = ΔE_exact − [E_q(g) − E_q(40)]_engine − wall(g), on the twelve exact geometries inside the identity");
    let mut e_q: Vec<f64> = Vec::with_capacity(exact.len());
    let mut rem: Vec<f64> = Vec::with_capacity(exact.len());
    let mut wall_e: Vec<f64> = Vec::with_capacity(exact.len());
    for e in &exact {
        let (_, e_q_diff, _) = engine_interaction(&e.a, &e.b, None, SeamPlant::None);
        let wv = wall.value(&e.oo, &e.ho, &e.hh);
        let r = e.de_exact - e_q_diff - wv;
        eprintln!(
            "  {} ({}): ΔE_exact {:.6e}, E_q(g) − E_q(40) {:.6e}, wall {:.6e} → remainder {:.6e} Ha ({:+.4} of |ΔE_exact|)",
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
    let mut c1_line_within = 0usize;
    let mut c1_line_of = 0usize;
    let mut c1_misses: Vec<String> = Vec::new();
    let mut c1_lines: Vec<String> = Vec::new();
    let mut line_report: Vec<String> = Vec::new();
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
        if e.line_node {
            c1_line_of += 1;
            if ok {
                c1_line_within += 1;
            }
            line_report.push(format!("{}: miss {miss:.6e} vs tolerance {t:.6e} → {}", e.name, if ok { "within" } else { "MISS" }));
        }
        eprintln!("  {}: remainder {:.6e}, fit {f:+.6e}, miss {miss:.6e} ({:.4} of its tolerance {t:.3e}) → {}", e.name, rem[g], miss / t, if ok { "within" } else { "MISS" });
        c1_lines.push(format!(
            "{{\"node\": \"{}\", \"kind\": \"{}\", \"source\": \"{}\", \"r_angstrom\": {:.1}, \"r_oo_bohr\": {:.6}, \"line_node\": {}, \"outer_linear\": {}, \"de_exact\": {:.12e}, \"e_q_difference\": {:.12e}, \"wall\": {:.12e}, \"remainder\": {:.12e}, \"contact_fit\": {f:+.12e}, \"miss\": {miss:.12e}, \"tolerance\": {t:.12e}, \"within\": {ok}}}",
            e.name,
            e.kind,
            esc(&e.source),
            e.r_ang,
            e.r_oo_bohr,
            e.line_node,
            e.outer_linear,
            e.de_exact,
            e_q[g],
            wall_e[g],
            rem[g]
        ));
    }
    let c1_line_ok = c1_line_within == c1_line_of && c1_line_of == LINE_ANGSTROM.len();
    let c1 = c1_within >= C1_MIN && c1_line_ok;
    eprintln!(
        "C1: {c1_within} of {} within max({C1_FRAC}·|ΔE_exact|, {C1_ABS:e}) (needs ≥ {C1_MIN}), and the line at 2.7, 2.9, 3.1 Å all within: {c1_line_ok} ({c1_line_within} of {c1_line_of}) → {}{}",
        exact.len(),
        if c1 { "PASS" } else { "FAIL — the remainder's shape at the minimum is the open problem, and its size is named above" },
        if c1_misses.is_empty() { String::new() } else { format!("; misses: {}", c1_misses.join(", ")) }
    );
    for l in &line_report {
        eprintln!("  line node — {l}");
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
    eprintln!("\ndispersion — the remainder AFTER the two contact terms, on the {} outer linear nodes (R_OO ≥ {OUTER_FROM_ANGSTROM} Å):", outer.len());
    for (oi, &g) in outer.iter().enumerate() {
        eprintln!("  {:.1} Å: remainder after contact {:.6e} Ha ({:+.4} of |ΔE_exact|)", exact[g].r_ang, rem2[oi], rem2[oi] / exact[g].de_exact.abs());
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

    // ------------------------------------------------- the harvested law, then G-B0 by the engine
    let model = SeamModel { a: a_oo, b: b_oo, p: p_ho, c: c_ho, c6, a_oh, b_oh, a_hh, b_hh, p_hh, c_hh, ..SeamModel::NO_WALL };
    let q_h = holon_render::field::water_charge_at_pin();
    let r_min = [r_min_oo, r_min_oh, r_min_hh];
    let bounded = model.bounded(q_h, r_min, KT_293);
    let g_b0 = bounded.is_none();
    let hole = model.hole(q_h);
    eprintln!(
        "\nG-B0 — bounded: the engine walks each cross-unit class potential of the HARVESTED law from its own r_min (O–O {r_min_oo:.6}, H–O {r_min_oh:.6}, H–H {r_min_hh:.6} bohr) inward to 0.5 bohr on a 0.05 grid, kT = {KT_293:e} Ha, q_H = {q_h:.12e}"
    );
    match &bounded {
        None => eprintln!("  bounded() returned none → PASS: no class falls more than kT below its fit floor, and each is positive at contact; the arms may run"),
        Some(msg) => eprintln!("  bounded() named a fall, verbatim: {msg}\n  → the arms are VOID before they run (§2 G-B0). The harvest is still recorded in full."),
    }
    match &hole {
        None => eprintln!("  hole() (a READING beside the gate, not a gate): none — the law also rises monotonically inward from 3.0 bohr"),
        Some(msg) => eprintln!("  hole() (a READING beside the gate, not a gate) names a fall, verbatim: {msg}"),
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
            "  {}: engine {e_int:+.12e} vs formula {want:+.12e} — miss {miss:.3e} (E_q diff {e_field_diff:+.6e}, seam {e_seam_diff:+.6e}; pen_HO {:.6e}, pen_HH {:.6e}, wall_OO {:.6e}, wall_OH {:.6e}, wall_HH {:.6e}, disp {:.6e}; units {units}, O–O {oo_pairs}, H–O {ho_pairs})",
            e.name, t.pen_ho, t.pen_hh, t.w_oo, t.w_oh, t.w_hh, t.disp
        );
        g_c1_lines.push(format!(
            "{{\"node\": \"{}\", \"r_angstrom\": {:.1}, \"engine_interaction\": {e_int:+.12e}, \"formula\": {want:+.12e}, \"miss\": {miss:.3e}, \"e_q_difference\": {e_field_diff:+.12e}, \"engine_seam\": {e_seam_diff:+.12e}, \"contact_ho\": {:.12e}, \"contact_hh\": {:.12e}, \"wall_oo\": {:.12e}, \"wall_oh\": {:.12e}, \"wall_hh\": {:.12e}, \"disp\": {:.12e}, \"units\": {units}, \"oo_pairs\": {oo_pairs}, \"ho_pairs\": {ho_pairs}}}",
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

    // ------------------------------------------------------------------------ wall9.json
    let reading_lines: Vec<String> = readings
        .iter()
        .map(|n| {
            let m = wall.value(&n.oo, &n.ho, &n.hh);
            let tol = (tol_frac * n.e_exch).max(WALL_TOL_ABS);
            format!(
                "{{\"family\": \"{}\", \"r_angstrom\": {:.1}, \"tilt_degrees\": {:.1}, \"r_oo_bohr\": {:.6}, \"in_fit_set\": {}, \"source\": \"{}\", \"e_exch\": {:.12e}, \"model\": {:.12e}, \"miss\": {:.12e}, \"tolerance\": {tol:.12e}, \"within\": {}, \"within_typed_twentieth\": {}, \"e_hl\": {:.12e}, \"e_a0\": {:.12e}, \"e_b0\": {:.12e}, \"e_es\": {:.12e}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"n_det_a\": {}, \"n_det_b\": {}, \"product_dets\": {}, \"s_cross_max\": {:.6e}, \"sigma_seconds\": {:.3}, \"cpu_seconds\": {:.3}, \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}]}}",
                n.family,
                n.r_ang,
                n.tilt_deg,
                n.r_oo_bohr,
                n.fit_set,
                esc(&n.source),
                n.e_exch,
                m,
                m - n.e_exch,
                n.fit_set && (m - n.e_exch).abs() <= tol,
                n.fit_set && (m - n.e_exch).abs() <= PLANT_II_TYPED_FRAC * n.e_exch,
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
    let bounded_json = match &bounded {
        None => "null".to_string(),
        Some(msg) => format!("\"{}\"", esc(msg)),
    };
    let hole_json = match &hole {
        None => "null".to_string(),
        Some(msg) => format!("\"{}\"", esc(msg)),
    };

    let mut j = String::new();
    j.push_str("{\n");
    j.push_str(&format!(
        "  \"a\": {a_oo:.12e}, \"b\": {b_oo:.12e}, \"p\": {p_ho:.12e}, \"c\": {c_ho:.12e}, \"c6\": {c6:.12e}, \"a_oh\": {a_oh:.12e}, \"b_oh\": {b_oh:.12e}, \"a_hh\": {a_hh:.12e}, \"b_hh\": {b_hh:.12e}, \"p_hh\": {p_hh:.12e}, \"c_hh\": {c_hh:.12e},\n"
    ));
    j.push_str(&format!(
        "  \"r_min_oo\": {r_min_oo:.12e}, \"r_min_oh\": {r_min_oh:.12e}, \"r_min_hh\": {r_min_hh:.12e},\n  \"r_min_rule\": \"the SHORTEST cross-unit distance of each class among the {N_FIT} FIT geometries — the arms runner's own boundedness check uses these three with kT = {KT_293:e}\",\n  \"r_min_sources\": {{\"oo\": \"{}\", \"oh\": \"{}\", \"hh\": \"{}\"}},\n",
        esc(&who_oo),
        esc(&who_oh),
        esc(&who_hh)
    ));
    j.push_str(&format!("  \"d0\": {},\n", d0_json.trim_end()));
    j.push_str(&format!(
        "  \"s1\": {{\"branch\": \"{s1_branch}\", \"within\": {within}, \"of\": {N_FIT}, \"branch_b_minimum\": {S1_B_MIN}, \"tolerance_rule\": \"max(δ/2·E_exch, {WALL_TOL_ABS:e}) with δ/2 = {tol_frac:.9}, DERIVED in D0 and not typed\", \"misses\": [{}], \"miss_details\": [{}]}},\n",
        miss_names.join(", "),
        miss_lines.join(", ")
    ));
    j.push_str(&format!(
        "  \"wall_fit\": {{\"readings\": {N_FIT}, \"reading_set\": \"the 48 at R_OO ≥ {FIT_FROM_ANGSTROM} Å; the {N_BOUND} at R_OO ≤ 2.3 Å are NOT fit and enter G-B0 only\", \"grid\": \"0.50..=4.00 per bohr, step 0.05, per class\", \"triples\": {}, \"weights\": \"1/E_exch²\", \"nonnegativity\": \"a class whose fitted amplitude is negative is dropped (amplitude an exact 0.0) and the rest refit; the most negative first, one class per round\", \"weighted_residual\": {:.12e}, \"classes_kept\": {{\"oo\": {}, \"oh\": {}, \"hh\": {}}}, \"fit_seconds\": {:.3}}},\n",
        NB * NB * NB,
        wall.residual,
        wall.kept[0],
        wall.kept[1],
        wall.kept[2],
        wall.seconds
    ));
    j.push_str(&format!(
        "  \"plant_ii\": {{\"fires\": {plant_ii_fires}, \"plant\": \"the SAME {N_FIT}-reading fit judged at FIELD-8's typed twentieth instead of the derived rule\", \"typed_rule\": \"{PLANT_II_TYPED_FRAC}·E_exch (§5 (ii)'s own rule, a pure fraction with no absolute floor)\", \"outside_typed\": {typed_outside}, \"outside_derived\": {derived_outside}, \"extra\": {extra}, \"extra_required\": {PLANT_II_MIN_EXTRA}, \"carrier\": {tol_frac:.12e}, \"carrier_floor\": {PLANT_II_CARRIER_FLOOR}, \"carrier_present\": {carrier_ii_ok}, \"carrier_definition\": \"δ/2, the derived tolerance fraction itself — the sector the plant acts on is the tolerance\", \"outside_typed_details\": [{}]}},\n",
        typed_lines.join(", ")
    ));
    j.push_str(&format!(
        "  \"g_b0\": {{\"pass\": {g_b0}, \"bounded\": {bounded_json}, \"q_h\": {q_h:.15e}, \"q_h_source\": \"holon_render::field::water_charge_at_pin\", \"kt\": {KT_293:e}, \"kt_source\": \"kT at 293 K, hartree (§0)\", \"r_min\": [{r_min_oo:.12e}, {r_min_oh:.12e}, {r_min_hh:.12e}], \"r_min_order\": \"O–O, H–O, H–H\", \"rule\": \"SeamModel::bounded walks each cross-unit class potential from its own r_min inward to 0.5 bohr on a 0.05 grid and names the first value more than kT below the value at r_min, or a value at 0.5 bohr that is not positive; none means the arms may run, a name means the arms are VOID before they run\", \"verdict\": \"{}\", \"hole_reading\": {hole_json}, \"hole_is_a_reading_not_a_gate\": true}},\n",
        if g_b0 { "bounded — the arms may run" } else { "a fall named — the arms are VOID (§2 G-B0)" }
    ));
    j.push_str(&format!(
        "  \"c1\": {{\"pass\": {c1}, \"within\": {c1_within}, \"of\": {}, \"required\": {C1_MIN}, \"line_within\": {c1_line_within}, \"line_nodes\": {c1_line_of}, \"line_ok\": {c1_line_ok}, \"line_rule\": \"the linear nodes at 2.7, 2.9 and 3.1 Å — where the bond lives — must ALL be within\", \"tolerance_rule\": \"max({C1_FRAC}·|ΔE_exact|, {C1_ABS:e})\", \"misses\": [{}]}},\n",
        exact.len(),
        c1_miss_names.join(", ")
    ));
    j.push_str(&format!(
        "  \"contact_fit\": {{\"p_ho\": {p_ho:.12e}, \"c_ho\": {c_ho:.12e}, \"p_hh\": {p_hh:.12e}, \"c_hh\": {c_hh:.12e}, \"classes_kept\": {{\"ho\": {}, \"hh\": {}}}, \"weighted_residual\": {c_res:.12e}, \"grid\": \"0.50..=4.00 per bohr, step 0.02, per class\", \"grid_points_per_class\": {NC}, \"grid_pairs\": {}, \"placement\": \"cross-unit H–O and cross-unit H–H\", \"weights\": \"1/ΔE_exact²\", \"nonnegativity\": \"drop-and-refit, the most negative amplitude first\", \"points\": {}, \"points_rule\": \"the TWELVE exact geometries the engine serves with two units; the 2.1 Å node is outside the closure identity and is NOT a fit point\", \"units_two_on_every_point\": true, \"remainder\": \"ΔE_exact − [E_q(g) − E_q(40)]_engine − wall(g), the NEW wall HELD at this harvest\", \"fit_seconds\": {cfit_seconds:.3}, \"nodes\": [\n    {}\n  ]}},\n",
        ckeep[0],
        ckeep[1],
        NC * NC,
        exact.len(),
        c1_lines.join(",\n    ")
    ));
    j.push_str(&format!(
        "  \"dispersion\": {{\"nodes\": \"the {} outer linear nodes (R_OO ≥ {OUTER_FROM_ANGSTROM} Å), after both contact terms\", \"c6\": {c6:.12e}, \"transferred\": {c6_transferred}, \"slope_band\": [{SLOPE_LO}, {SLOPE_HI}], \"slopes\": [{}]}},\n",
        outer.len(),
        slope_lines.join(", ")
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
        "  \"readings_provenance\": {{\"count\": {N_READ}, \"measured_here\": 0, \"note\": \"no exchange reading is taken by this runner; every one is a record, its geometry rebuilt here and matched to the record's own centers within 1e-9 bohr (M-STALE-INSTRUMENT)\", \"overlap_window\": \"({NORM_LO}, {NORM_HI}]\", \"overlap_ok\": {norm_ok}, \"overlap_lowest\": {norm_lo_seen:.15e}, \"overlap_highest\": {norm_hi_seen:.15e}, \"floor\": {R_FLOOR:e}, \"floor_ok\": {floor_ok}, \"monotone_breaks\": {monotone_breaks}, \"product_dets_staked\": {PRODUCT_DETS_STAKED}}},\n"
    ));
    j.push_str(&format!(
        "  \"referee\": \"heitler_london_undeformed — the antisymmetrised product of the monomers' own wavefunctions, the monomers NOT deformed\",\n  \"reading_set\": {{\"tilt_family_r_angstrom\": [{}], \"twist_family_r_angstrom\": [{}], \"tilt_degrees\": [{}], \"twist_about_oo_axis_degrees\": {TWIST_ABOUT_Z_DEG:.1}, \"pivot\": \"the acceptor's own oxygen\", \"donor\": \"untouched\", \"fit_rule\": \"R_OO ≥ {FIT_FROM_ANGSTROM} Å\", \"fit_count\": {N_FIT}, \"boundedness_only_count\": {N_BOUND}}},\n",
        TILT_R.iter().map(|r| format!("{r:.1}")).collect::<Vec<_>>().join(", "),
        TWIST_R.iter().map(|r| format!("{r:.1}")).collect::<Vec<_>>().join(", "),
        TILT_DEG.iter().map(|t| format!("{t:.1}")).collect::<Vec<_>>().join(", ")
    ));
    j.push_str(&format!("  \"readings\": [\n    {}\n  ]\n}}\n", reading_lines.join(",\n    ")));
    fs::write(out.join("wall9.json"), j).unwrap();
    eprintln!("\nwall9.json written");

    // ------------------------------------ prediction.json, BEFORE the held-out solve
    let (a_p, b_p) = double_bent(o, h, DOUBLE_ANGSTROM, DOUBLE_DONOR_DEGREES, DOUBLE_ACCEPTOR_DEGREES);
    let r_oo_p = cross_oo(&a_p, &b_p);
    assert!(
        (r_oo_p - DOUBLE_ANGSTROM * ANGSTROM_TO_BOHR).abs() < 1e-9,
        "both monomers turn about their OWN oxygens: R_OO must be unchanged ({r_oo_p:.9} vs {:.9})",
        DOUBLE_ANGSTROM * ANGSTROM_TO_BOHR
    );
    let (e_pred, e_q_p, e_seam_p) = engine_interaction(&a_p, &b_p, Some(model), SeamPlant::None);
    let tp = formula_terms(&a_p, &b_p, &model);
    let s_p = engine_dimer(&a_p, &b_p, Some(model), SeamPlant::None);
    let (oo_p, ho_p, hh_p) = cross_classes(&a_p, &b_p);
    fs::write(
        out.join("prediction.json"),
        format!(
            "{{\n  \"node\": \"doublebent_R{DOUBLE_ANGSTROM:.1}\", \"r_oo_angstrom\": {DOUBLE_ANGSTROM:.3}, \"r_oo_bohr\": {r_oo_p:.6}, \"donor_bend_degrees\": {DOUBLE_DONOR_DEGREES:.1}, \"acceptor_bend_degrees\": {DOUBLE_ACCEPTOR_DEGREES:.1}, \"bend_axis\": \"x, through EACH monomer's own oxygen — the donor turned {DOUBLE_DONOR_DEGREES:.0}° as FIELD-8's bent donor, the acceptor turned {DOUBLE_ACCEPTOR_DEGREES:.0}° as FIELD-5's tilt; both pivots are oxygens, so R_OO is unchanged\", \"held_out\": true, \"kind\": \"a DOUBLY BENT dimer — both monomers turned, a kind no fit point contains (§4, M-UNTESTED-GAP)\",\n  \"e_pred\": {e_pred:+.12e},\n  \"parts\": {{\"e_q_difference\": {:.12e}, \"contact_ho\": {:.12e}, \"contact_hh\": {:.12e}, \"wall_oo\": {:.12e}, \"wall_oh\": {:.12e}, \"wall_hh\": {:.12e}, \"wall_total\": {:.12e}, \"disp\": {:.12e}, \"engine_seam\": {:.12e}}},\n  \"coefficients\": {{\"a\": {a_oo:.12e}, \"b\": {b_oo:.12e}, \"p\": {p_ho:.12e}, \"c\": {c_ho:.12e}, \"c6\": {c6:.12e}, \"a_oh\": {a_oh:.12e}, \"b_oh\": {b_oh:.12e}, \"a_hh\": {a_hh:.12e}, \"b_hh\": {b_hh:.12e}, \"p_hh\": {p_hh:.12e}, \"c_hh\": {c_hh:.12e}}},\n  \"s1_branch\": \"{s1_branch}\", \"c1_pass\": {c1}, \"c6_transferred\": {c6_transferred}, \"g_b0_pass\": {g_b0}, \"g_c1_pass\": {g_c1}, \"delta\": {delta:.9}, \"tolerance_fraction_of_the_wall\": {tol_frac:.9}, \"units\": {}, \"oo_pairs\": {}, \"ho_pairs\": {},\n  \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\", \"tolerance_frac\": {PRED_FRAC}, \"tolerance_abs\": {PRED_ABS:e},\n  \"exact_solve_stake\": {{\"n_det\": {N_DET_DIMER}, \"cpu_seconds_lo\": {S2_CPU_LO}, \"cpu_seconds_hi\": {S2_CPU_HI}, \"residual_bar\": {RESIDUAL_BAR:e}, \"exit\": \"Converged\"}},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}],\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
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
        "prediction.json filed BEFORE the held-out solve: E_pred {e_pred:+.6e} Ha — E_q(g) − E_q(40) {e_q_p:+.6e}, contact_HO {:.6e}, contact_HH {:.6e}, wall_OO {:.6e}, wall_OH {:.6e}, wall_HH {:.6e}, disp {:.6e}; units {}",
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
    let wall9 = fs::read_to_string(out.join("wall9.json")).expect("wall9.json: run `fit` first");
    let (a_oo, b_oo) = (json_num(&wall9, "a"), json_num(&wall9, "b"));
    let (a_oh, b_oh) = (json_num(&wall9, "a_oh"), json_num(&wall9, "b_oh"));
    let (a_hh, b_hh) = (json_num(&wall9, "a_hh"), json_num(&wall9, "b_hh"));
    assert!(
        a_oo.is_finite() && b_oo.is_finite() && a_oh.is_finite() && b_oh.is_finite() && a_hh.is_finite() && b_hh.is_finite(),
        "wall9.json carries no three-class wall"
    );
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let (a, b) = double_bent(o, h, DOUBLE_ANGSTROM, DOUBLE_DONOR_DEGREES, DOUBLE_ACCEPTOR_DEGREES);
    let name = format!("doublebent_R{DOUBLE_ANGSTROM:.1}");
    eprintln!(
        "FIELD-9 predict — the HELD-OUT DOUBLY BENT node (donor {DOUBLE_DONOR_DEGREES:.0}° and acceptor {DOUBLE_ACCEPTOR_DEGREES:.0}° about the x-axis through their OWN oxygens, R_OO {DOUBLE_ANGSTROM:.1} Å) on {} threads; E_pred {e_pred:+.12e} Ha",
        threads()
    );

    // the exact solve first: the prediction is already on disk
    let ok = solve_node(out, &name, DOUBLE_ANGSTROM, &a, &b, true);
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
        "S2: ΔE_exact {de:+.6e} Ha, E_pred {e_pred:+.6e} — miss {miss:.3e} ({:.1} % of |ΔE_exact|) against {tol:.3e}; E_exch(undeformed, doubly bent) {:.6e} vs the three-class wall {wall_held:+.6e} (O–O {w_oo:+.6e}, H–O {w_oh:+.6e}, H–H {w_hh:+.6e}), difference {wall_gap:.3e} → branch ({s2})",
        100.0 * miss / de.abs(),
        hl.e_exch
    );
    fs::write(
        out.join("prediction_check.json"),
        format!(
            "{{\n  \"node\": \"{name}\", \"e_pred\": {e_pred:+.12e}, \"de_exact\": {de:+.12e},\n  \"miss\": {miss:.6e}, \"miss_fraction\": {:.6}, \"tolerance\": {tol:.6e}, \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\",\n  \"s2_branch\": \"{s2}\", \"s2_branch_meaning\": \"(a) the prediction lands within tolerance; (b) it misses and the wall is within that tolerance of E_exch on the same geometry; (c) both miss\",\n  \"exact\": {{\"converged\": {ok}, \"exit\": \"{}\", \"davidson_iters\": {}, \"residual\": {:.3e}, \"residual_bar\": {RESIDUAL_BAR:e}, \"n_det\": {n_det}, \"n_det_expected\": {N_DET_DIMER}, \"n_det_ok\": {n_det_ok}, \"cpu_seconds\": {cpu:.1}, \"cpu_seconds_lo\": {S2_CPU_LO}, \"cpu_seconds_hi\": {S2_CPU_HI}, \"price_in_band\": {price_ok}, \"wall_seconds\": {:.1}}},\n  \"exchange_on_the_held_out_node\": {{\"r_oo_bohr\": {r_oo_p:.6}, \"e_exch\": {:.12e}, \"e_hl\": {:.12e}, \"e_a0\": {:.12e}, \"e_b0\": {:.12e}, \"e_es\": {:.12e}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det_a\": {}, \"n_det_b\": {}, \"product_dets\": {}, \"s_cross_max\": {:.6e}, \"sigma_seconds\": {:.3}, \"cpu_seconds\": {hl_cpu:.3}, \"wall_seconds\": {hl_wall:.3}}},\n  \"wall\": {{\"a\": {a_oo:.12e}, \"b\": {b_oo:.12e}, \"a_oh\": {a_oh:.12e}, \"b_oh\": {b_oh:.12e}, \"a_hh\": {a_hh:.12e}, \"b_hh\": {b_hh:.12e}, \"value\": {wall_held:+.12e}, \"oo\": {w_oo:+.12e}, \"oh\": {w_oh:+.12e}, \"hh\": {w_hh:+.12e}, \"minus_e_exch\": {:.12e}, \"abs_difference\": {wall_gap:.6e}, \"within_tolerance\": {}}},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}]\n}}\n",
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
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/field9".to_string()));
    fs::create_dir_all(&out).expect("out");
    match what {
        "fit" => run_fit(&out),
        "predict" => run_predict(&out),
        other => eprintln!("unknown phase {other} (fit | predict)"),
    }
}
