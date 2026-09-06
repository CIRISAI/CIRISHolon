//! CT-1's harvest (`conformance/water_observatory/CT1_PREREG.md` and `CT1_AMENDMENT_1.md`): THE
//! SIXTH CHANNEL — charge transfer measured as the energy the two closures gain by OPENING.
//!
//! FIELD-6 read one number at the hydrogen-bond minimum: `−10.4` mHa the exact dimer has and the
//! undeformed Heitler–London product does not. This freeze splits it in three. The BLOCK-LOCALISED
//! sector (AMENDMENT 1: the span of the products of the two monomers' determinants in their OWN
//! orbitals, `441 × 441` on the water dimer — NOT the orthogonalised sector, which the amendment's
//! own order gate refuted on a hydrogen pair) is the best the dimer can do without an electron
//! crossing, so
//!
//! ```text
//! E_CT(g) = E_exact(g) − E_noCT(g)
//! ```
//!
//! is charge transfer by itself, `E_noCT − E_HL(undeformed)` is what the closed sector gains by
//! polarising and correlating without opening, and `ΔE_HL` is electrostatics plus exchange.
//!
//! ```text
//! cargo run --release -p holon-render --example ct1_harvest -- sector  [OUT_DIR]
//! cargo run --release -p holon-render --example ct1_harvest -- far     [OUT_DIR]
//! cargo run --release -p holon-render --example ct1_harvest -- fit     [OUT_DIR]
//! cargo run --release -p holon-render --example ct1_harvest -- predict [OUT_DIR]
//! ```
//!
//! `sector` (detached, resumable): the block-localised solve on the twelve exact geometries inside
//! the closure identity and on the 40-bohr reference, each priced (M-CHEAPER-THAN-ITS-PRICE) and
//! recorded with its exit and iteration count (M-EXIT-DISCRIMINATOR); then plant (i) — the same
//! Davidson on the FULL space from the same start — at the linear 2.9 Å node. `far` (detached):
//! the EXACT solve of that same 40-bohr geometry, which T0's far leg reads and which no record
//! carries. `fit`: T0, T1, T2,
//! the CT term (S1), the contact term re-fit on what remains (C1), dispersion, G-B0, G-C1, both
//! plants, `wall_ct.json`, and `prediction.json` for the twisted-and-bent held-out node, filed
//! BEFORE that node is solved. `predict`: the held-out node solved exactly, its own sector solve,
//! and `prediction_check.json` (S2).
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::{solve_embedded, supermolecule, water_dimer_linear, Fragment, ANGSTROM_TO_BOHR};
use holon_chem::fci::SolveExit;
use holon_chem::heitler_london::{fci_block_localised, fci_full_from_product};
use holon_render::seam::{SeamModel, SeamPlant};
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/quartet.rs"]
#[allow(dead_code)]
mod quartet;

/// EMBED-1's water pins — the same numbers FIELD-3's … FIELD-9's runners carry.
const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;

/// The SEVEN linear exact nodes inside the closure identity, shortest first (FIELD-9's list).
const LINEAR_ANGSTROM: [f64; 7] = [2.3, 2.5, 2.7, 2.9, 3.1, 3.4, 3.7];
/// The linear node of record that lives in `field8/` rather than `field3/`.
const LINEAR_FROM_FIELD8: f64 = 2.3;
/// A linear node at or beyond this separation is one of FIELD-6's four OUTER dispersion nodes.
const OUTER_FROM_ANGSTROM: f64 = 2.9;
/// C1's second leg: the line where the bond lives must be within tolerance at all three.
const LINE_ANGSTROM: [f64; 3] = [2.7, 2.9, 3.1];
/// The node both plants and T2 are read at.
const REF_ANGSTROM: f64 = 2.9;

/// The five non-linear exact geometries of record (FIELD-9 §0).
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

/// S2's held-out geometry (§2 S2): the linear dimer at `R_OO = 3.2` Å with the ACCEPTOR twisted
/// `90°` about the O···O axis and the DONOR bent `20°` about the x-axis through its own oxygen.
const PRED_ANGSTROM: f64 = 3.2;
const PRED_TWIST_DEGREES: f64 = 90.0;
const PRED_TILT_DEGREES: f64 = 0.0;
const PRED_DONOR_BEND_DEGREES: f64 = 20.0;
const PRED_NAME: &str = "twistbent_R3.2";

/// The separation at which the acceptor is "away" (bohr): the engine's reference on both sides of
/// G-C1 and of every `E_q` difference, and the geometry the 40-bohr sector reading is taken on.
const FAR_BOHR: f64 = 40.0;
/// The name the 40-bohr sector reading is recorded under.
const FAR_NAME: &str = "linear_far40";

/// The residual bar every EXACT solve must meet (EMBED-3's).
const RESIDUAL_BAR: f64 = 1e-9;
/// T0's residual bar on the SECTOR solve (AMENDMENT 1: the generalised residual).
const SECTOR_RESIDUAL_BAR: f64 = 1e-8;
/// T0's subspace dimension on the water dimer, EXACT: `441 × 441`.
const SECTOR_DIM_STAKED: usize = 194_481;
/// T0's order slack, on TOTAL energies: `E_exact ≤ E_noCT ≤ E_HL(undeformed)`.
const ORDER_TOL: f64 = 1e-10;
/// T0's far limit: `|E_CT|` at 40 bohr.
const FAR_CT_TOL: f64 = 1e-8;

/// T1: charge transfer is attractive on the line at 2.3–3.4 Å (M-FLOOR-UNSTAKED: the CT floor).
const CT_FLOOR: f64 = 1e-6;
/// T1's line: the linear nodes charge transfer must be attractive at.
const T1_LINE_LO: f64 = 2.3;
const T1_LINE_HI: f64 = 3.4;
/// T1: the log-log slope of `|E_CT|` against `R_OO` beyond 2.9 Å must be steeper than this.
const T1_SLOPE_MAX: f64 = -6.0;

/// FIELD-6's remainder at the linear 2.9 Å node, quoted beside T2's three parts (hartree).
const FIELD6_REMAINDER: f64 = -1.04e-2;

/// The contact and CT `c`-grids, per class (per bohr): `0.50 ..= 4.00` step `0.02` — 176 values.
const NC: usize = 176;
/// THIS FREEZE'S CORRECTION OF THE WEIGHTING RULE (§0): `1/max(|ΔE_exact|, 5e-3)²`. FIELD-7/8/9's
/// `1/ΔE_exact²` blew up at the 2.5 Å zero crossing and pinned every contact fit there.
const WEIGHT_FLOOR: f64 = 5e-3;

/// S1's tolerance, per exact node: `max(0.25·|ΔE_exact|, 5e-4)`; (a) all twelve, (b) at least nine.
const S1_FRAC: f64 = 0.25;
const S1_ABS: f64 = 5e-4;
const S1_B_MIN: usize = 9;

/// C1's tolerance is S1's; at least ten of twelve, and the line at 2.7, 2.9, 3.1 Å all within.
const C1_MIN: usize = 10;

/// The band the remainder's log-log slope must lie in for `C₆` to transfer (FIELD-6's rule).
const SLOPE_LO: f64 = -8.0;
const SLOPE_HI: f64 = -4.0;

/// G-B0's temperature, in hartree (this freeze's `kT`).
const KT: f64 = 9.278758e-4;

/// G-C1's tolerance, and both plants' carriers and bars.
const G_C1_TOL: f64 = 1e-10;
/// Plant (ii): `P_CT → −P_CT` must move G-C1 by `2·|CT(2.9 Å)|`; its carrier floor.
const PLANT_II_CARRIER: f64 = 1e-4;
/// Plant (i): the mask dropped must reproduce `E_exact` to this, with this carrier floor on `E_CT`.
const PLANT_I_TOL: f64 = 1e-8;
const PLANT_I_CARRIER: f64 = 1e-3;

/// S2's tolerance on the total, and on the CT term by itself (branch (b)).
const PRED_FRAC: f64 = 0.25;
const PRED_ABS: f64 = 5e-4;
const PRED_CT_FRAC: f64 = 0.25;
const PRED_CT_ABS: f64 = 2e-4;

/// The determinant count FIELD-3's supermolecule carries (EXACT).
const N_DET_DIMER: usize = 1_002_001;
/// Every EXACT solve's price band (FIELD-9 §2).
const S2_CPU_LO: f64 = 1450.0;
const S2_CPU_HI: f64 = 57600.0;
/// M-CHEAPER-THAN-ITS-PRICE: FIELD-5's measured floor, core-seconds per Hamiltonian application,
/// and the tenth of its own iteration count a sector solve is refused under.
const SIGMA_PRICE_FLOOR: f64 = 55.0;
const PRICE_TENTH: f64 = 0.1;

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

/// Several banked records carry numbers with a leading `+` (`+1.7e-1`), which is not valid JSON;
/// this string-split reader takes them as they are. NOTHING THIS RUNNER WRITES DOES THAT.
fn json_num(t: &str, key: &str) -> f64 {
    t.split(&format!("\"{key}\": ")).nth(1).and_then(|x| x.split(|c| c == ',' || c == '\n' || c == '}').next()).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::NAN)
}
fn json_str(t: &str, key: &str) -> String {
    t.split(&format!("\"{key}\": \"")).nth(1).and_then(|x| x.split('"').next()).unwrap_or("").to_string()
}
fn json_bool(t: &str, key: &str) -> bool {
    t.split(&format!("\"{key}\": ")).nth(1).map(|x| x.trim_start().starts_with("true")).unwrap_or(false)
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

/// ONE JSON NUMBER, valid whatever the arithmetic did (M-FORMAT-FLOOR): twelve digits, never a
/// leading `+`, and `null` rather than `NaN`/`inf` — a reader must not be handed a broken file.
fn jn(x: f64) -> String {
    if x.is_finite() {
        format!("{x:.12e}")
    } else {
        "null".to_string()
    }
}
/// The same, in fixed notation (distances, counts of seconds).
fn jd(x: f64) -> String {
    if x.is_finite() {
        format!("{x:.6}")
    } else {
        "null".to_string()
    }
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

/// FIELD-8's bent donor: the DONOR rotated about the x-axis through its own oxygen, the acceptor
/// untouched.
fn bent_donor(o: Species, h: Species, r_oo_angstrom: f64, theta_degrees: f64) -> (Fragment, Fragment) {
    let (donor, acc) = linear(o, h, r_oo_angstrom);
    (rot_x(&donor, theta_degrees), acc)
}

/// S2's held-out geometry (§2 S2): the acceptor TWISTED `90°` about the O···O axis (FIELD-7's
/// builder, tilt `0°`) and the donor BENT `20°` about the x-axis through its own oxygen. Both
/// pivots are oxygens, so `R_OO` is unchanged.
fn twist_bent(o: Species, h: Species) -> (Fragment, Fragment) {
    let (donor, acc) = twisted(o, h, PRED_ANGSTROM, PRED_TWIST_DEGREES, PRED_TILT_DEGREES);
    (rot_x(&donor, PRED_DONOR_BEND_DEGREES), acc)
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
    v.iter().map(|d| jd(*d)).collect::<Vec<_>>().join(", ")
}

fn sum_exp(rs: &[f64], b: f64) -> f64 {
    rs.iter().map(|&r| (-b * r).exp()).sum()
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

/// The formula side of G-C1, class by class — CT-1 adds the sixth channel's own row.
struct Terms {
    pen_ho: f64,
    pen_hh: f64,
    w_oo: f64,
    w_oh: f64,
    w_hh: f64,
    disp: f64,
    ct: f64,
}

impl Terms {
    fn total(&self) -> f64 {
        self.pen_ho + self.pen_hh + self.w_oo + self.w_oh + self.w_hh + self.disp + self.ct
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
        ct: ho.iter().map(|&r| m.charge_transfer(r)).sum(),
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
            "{{\n  \"node\": \"{name}\", \"r_oo_angstrom\": {r_oo_angstrom:.3}, \"r_oo_bohr\": {},\n  \"n_det\": {}, \"e_super\": {}, \"e_a0\": {}, \"e_b0\": {}, \"de_exact\": {},\n  \"davidson_iters\": {}, \"residual\": {:.3e}, \"exit\": \"{}\", \"converged\": {converged},\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}],\n  \"wall_seconds\": {wall:.1}, \"cpu_seconds\": {cpu:.1}, \"cpu_seconds_lo\": {S2_CPU_LO}, \"cpu_seconds_hi\": {S2_CPU_HI}, \"admitted\": {admitted},\n  \"threads\": {}, \"price_node\": {price}\n}}\n",
            jd(cross_oo(a, b)),
            sm.gp.space.n_det,
            jn(sm.e_total),
            jn(e_a0.e_total),
            jn(e_b0.e_total),
            jn(de),
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
        "  {name}: R_OO {r_oo_angstrom:.1} Å, {} dets, ΔE_exact {de:.6e} Ha, {} iters, residual {:.1e}, exit {}, wall {wall:.0} s, {cpu:.0} core-s, admitted {admitted} (band {S2_CPU_LO}–{S2_CPU_HI})",
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

// ---------------------------------------------------------------- the non-negative fit

/// FIELD-8's non-negativity rule on TWO classes (the contact term's `P_HO`, `P_HH`): drop the most
/// negative amplitude to an exact `0.0` and refit. Returns `(amplitudes, kept, weighted residual)`.
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

/// The contact grid, per bohr: `0.50 ..= 4.00` step `0.02`, built so the ends are exact.
fn cgrid(i: usize) -> f64 {
    ((25 + i) as f64) * 0.02
}

// ------------------------------------------------------------------------ the exact record

struct ENode {
    name: String,
    kind: &'static str,
    r_ang: f64,
    r_oo_bohr: f64,
    de_exact: f64,
    e_super: f64,
    e_a0_rec: f64,
    e_b0_rec: f64,
    a: Fragment,
    b: Fragment,
    oo: Vec<f64>,
    ho: Vec<f64>,
    hh: Vec<f64>,
    outer_linear: bool,
    line_node: bool,
    source: String,
}

/// The TWELVE exact geometries INSIDE the closure identity (FIELD-9 §0), each rebuilt here and
/// CHECKED against the record it is read from (M-STALE-INSTRUMENT): the line at 2.3–3.7 Å (seven
/// nodes), FIELD-5's 30°-bent acceptor, FIELD-6's 45°-bent acceptor, FIELD-4's flipped dimer,
/// FIELD-7's twisted dimer, FIELD-8's bent donor. TOTAL energies are carried alongside `ΔE_exact`:
/// T0's order gate reads on totals.
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
        let e_super = json_num(&t, "e_super");
        if !de.is_finite() || !e_super.is_finite() {
            missing.push(format!("{} (no de_exact / e_super)", path.display()));
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
            e_super,
            e_a0_rec: json_num(&t, "e_a0"),
            e_b0_rec: json_num(&t, "e_b0"),
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

// ------------------------------------------------------------------------ the sector record

/// One block-localised reading, read back from `sector_<node>.json`.
struct Sector {
    e_noct: f64,
    e_hl_undeformed: f64,
    e_a0: f64,
    e_b0: f64,
    sector_dim: usize,
    n_det: usize,
    davidson_iters: usize,
    residual: f64,
    converged: bool,
    metric_min_eigenvalue: f64,
    cpu_seconds: f64,
    wall_seconds: f64,
    price_admitted: bool,
    source: String,
}

fn sector_path(out: &Path, name: &str) -> PathBuf {
    out.join(format!("sector_{name}.json"))
}

fn load_sector(out: &Path, name: &str) -> Result<Sector, String> {
    let path = sector_path(out, name);
    let Ok(t) = fs::read_to_string(&path) else {
        return Err(path.display().to_string());
    };
    let e_noct = json_num(&t, "e_noct");
    if !e_noct.is_finite() {
        return Err(format!("{} (no e_noct)", path.display()));
    }
    Ok(Sector {
        e_noct,
        e_hl_undeformed: json_num(&t, "e_hl_undeformed"),
        e_a0: json_num(&t, "e_a0"),
        e_b0: json_num(&t, "e_b0"),
        sector_dim: json_num(&t, "sector_dim") as usize,
        n_det: json_num(&t, "n_det") as usize,
        davidson_iters: json_num(&t, "davidson_iters") as usize,
        residual: json_num(&t, "residual"),
        converged: json_bool(&t, "converged"),
        metric_min_eigenvalue: json_num(&t, "metric_min_eigenvalue"),
        cpu_seconds: json_num(&t, "cpu_seconds"),
        wall_seconds: json_num(&t, "wall_seconds"),
        price_admitted: json_bool(&t, "price_admitted"),
        source: path.display().to_string(),
    })
}

/// The block-localised solve on one geometry, priced and recorded. Skips a node already on disk.
fn sector_node(out: &Path, name: &str, r_ang: f64, a: &Fragment, b: &Fragment) {
    let path = sector_path(out, name);
    if path.exists() {
        eprintln!("  {name}: exists, skipped");
        return;
    }
    let t0 = Instant::now();
    let c0 = cpu_seconds();
    let r = fci_block_localised(a, b);
    let harness_wall = t0.elapsed().as_secs_f64();
    let cpu = cpu_seconds() - c0;
    let price_expected = PRICE_TENTH * (r.davidson_iters as f64) * SIGMA_PRICE_FLOOR;
    let price_admitted = cpu >= price_expected;
    let e_ct_vs_product = r.e_noct - r.e_hl_undeformed;
    fs::write(
        &path,
        format!(
            "{{\n  \"node\": \"{name}\", \"r_oo_angstrom\": {}, \"r_oo_bohr\": {},\n  \"e_noct\": {}, \"e_hl_undeformed\": {}, \"e_a0\": {}, \"e_b0\": {},\n  \"closed_sector_gain\": {},\n  \"sector_dim\": {}, \"sector_dim_staked\": {SECTOR_DIM_STAKED}, \"n_det\": {}, \"davidson_iters\": {}, \"residual\": {}, \"converged\": {},\n  \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}, \"metric_min_eigenvalue\": {},\n  \"wall_seconds\": {}, \"sigma_seconds\": {}, \"harness_wall_seconds\": {}, \"cpu_seconds\": {}, \"threads\": {},\n  \"price_floor_core_seconds_per_sigma\": {SIGMA_PRICE_FLOOR}, \"price_rule\": \"cpu_seconds >= {PRICE_TENTH} * davidson_iters * {SIGMA_PRICE_FLOOR} (M-CHEAPER-THAN-ITS-PRICE)\", \"price_expected_core_seconds\": {}, \"price_admitted\": {price_admitted},\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
            if r_ang.is_finite() { format!("{r_ang:.3}") } else { "null".to_string() },
            jd(cross_oo(a, b)),
            jn(r.e_noct),
            jn(r.e_hl_undeformed),
            jn(r.e_a0),
            jn(r.e_b0),
            jn(e_ct_vs_product),
            r.sector_dim,
            r.n_det,
            r.davidson_iters,
            jn(r.residual),
            r.converged,
            jn(r.metric_min_eigenvalue),
            jd(r.wall_seconds),
            jd(r.sigma_seconds),
            jd(harness_wall),
            jd(cpu),
            threads(),
            jd(price_expected),
            centers_json(a),
            centers_json(b),
        ),
    )
    .unwrap();
    eprintln!(
        "  {name}: E_noCT {:.12e}, E_HL(undeformed) {:.12e}, gain {:.6e} Ha, sector {} of {} dets, {} iters, residual {:.3e}, converged {}, metric min {:.3e}, wall {:.0} s ({:.0} s in sigma), {:.0} core-s, price floor {:.0} core-s, admitted {price_admitted}",
        r.e_noct,
        r.e_hl_undeformed,
        e_ct_vs_product,
        r.sector_dim,
        r.n_det,
        r.davidson_iters,
        r.residual,
        r.converged,
        r.metric_min_eigenvalue,
        r.wall_seconds,
        r.sigma_seconds,
        cpu,
        price_expected,
    );
}

// --------------------------------------------------------------------------- the sector phase

fn run_sector(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    eprintln!(
        "CT-1 sector — the BLOCK-LOCALISED solve (CT1_AMENDMENT_1: the span of the products of the two monomers' determinants in their OWN orbitals, {SECTOR_DIM_STAKED} states) on the twelve exact geometries inside the closure identity and on the {FAR_BOHR:.0}-bohr reference, then plant (i) at the linear {REF_ANGSTROM:.1} Å node, on {} threads",
        threads()
    );
    let exact = match exact_records(out, o, h) {
        Ok(v) => v,
        Err(missing) => {
            eprintln!("REFUSED — the sector is read on the TWELVE exact geometries of record and these are not on disk:");
            for m in &missing {
                eprintln!("  {m}");
            }
            eprintln!("Nothing written.");
            std::process::exit(3);
        }
    };
    assert_eq!(exact.len(), 12, "the exact record is twelve geometries INSIDE the identity");
    for e in &exact {
        eprintln!("  {} ({}): R_OO {:.4} bohr, ΔE_exact {:.6e} Ha, E_exact {:.12e}  [{}]", e.name, e.kind, e.r_oo_bohr, e.de_exact, e.e_super, e.source);
    }

    // the 40-bohr reference: the SAME far geometry the engine's E_q(40) uses — the reference node's
    // acceptor translated 40 bohr along x (FIELD-5's and FIELD-6's `exchange_far` convention)
    let refi = exact
        .iter()
        .position(|e| e.kind == "linear" && (e.r_ang - REF_ANGSTROM).abs() < 1e-9)
        .expect("the linear 2.9 Å node is of record");
    let far_b = exact[refi].b.translated([FAR_BOHR, 0.0, 0.0]);
    eprintln!(
        "the {FAR_BOHR:.0}-bohr reference `{FAR_NAME}`: the linear {REF_ANGSTROM:.1} Å node's acceptor translated [{FAR_BOHR:.1}, 0, 0] bohr — the engine's own E_q(40) geometry; its O–O separation is {:.6} bohr",
        cross_oo(&exact[refi].a, &far_b)
    );

    for e in &exact {
        sector_node(out, &e.name, e.r_ang, &e.a, &e.b);
    }
    sector_node(out, FAR_NAME, f64::NAN, &exact[refi].a, &far_b);

    // ------------------------------------------------- plant (i): the sector restriction removed
    let plant_path = out.join("plant_i_full.json");
    if plant_path.exists() {
        eprintln!("plant (i): plant_i_full.json exists, skipped");
    } else {
        eprintln!(
            "plant (i) — the mask dropped: the SAME Davidson on the FULL space ({N_DET_DIMER} determinants) from the SAME start (the undeformed product), at the linear {REF_ANGSTROM:.1} Å node. It must reproduce E_exact of record ({:.12e}) to {PLANT_I_TOL:e}",
            exact[refi].e_super
        );
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let (e_full, iters, residual, converged) = fci_full_from_product(&exact[refi].a, &exact[refi].b);
        let wall = t0.elapsed().as_secs_f64();
        let cpu = cpu_seconds() - c0;
        let miss = (e_full - exact[refi].e_super).abs();
        fs::write(
            &plant_path,
            format!(
                "{{\n  \"node\": \"{}\", \"plant\": \"(i) the mask dropped — the same Davidson on the FULL space from the undeformed product start\",\n  \"e_full\": {}, \"e_exact_record\": {}, \"miss\": {}, \"tolerance\": {PLANT_I_TOL:e}, \"within\": {},\n  \"davidson_iters\": {iters}, \"residual\": {}, \"converged\": {converged},\n  \"n_det\": {N_DET_DIMER}, \"wall_seconds\": {}, \"cpu_seconds\": {}, \"threads\": {},\n  \"exact_source\": \"{}\"\n}}\n",
                exact[refi].name,
                jn(e_full),
                jn(exact[refi].e_super),
                jn(miss),
                miss <= PLANT_I_TOL,
                jn(residual),
                jd(wall),
                jd(cpu),
                threads(),
                esc(&exact[refi].source),
            ),
        )
        .unwrap();
        eprintln!(
            "plant (i): E_full {e_full:.12e} vs E_exact {:.12e} — miss {miss:.3e} (bar {PLANT_I_TOL:e}), {iters} iters, residual {residual:.3e}, converged {converged}, wall {wall:.0} s, {cpu:.0} core-s",
            exact[refi].e_super
        );
    }

    fs::write(out.join("sector.done"), "done\n").unwrap();
    eprintln!("sector.done written");
}

// ------------------------------------------------------------ far (T0's far leg, EXACT)

/// T0's far leg is `|E_exact − E_noCT| ≤ 1e-8` at 40 bohr, and NO record carries `E_exact` there:
/// FIELD-6's `exchange_far.json` is the Heitler–London referee, not a full-space solve. The
/// non-interacting limit `E_A0 + E_B0` cannot stand in for it — at 40 bohr the exact dimer still
/// carries the linear dimer's electrostatics, of order `1e-5` hartree, and the block-localised
/// sector carries it too, so the stand-in would pass or fail the gate for the wrong reason. This
/// phase solves that one geometry EXACTLY, at the same price band and residual bar as the twelve,
/// on exactly the geometry the sector phase read (`{FAR_NAME}`).
fn run_far(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let exact = match exact_records(out, o, h) {
        Ok(v) => v,
        Err(missing) => {
            eprintln!("REFUSED — the far geometry is BUILT from the linear {REF_ANGSTROM:.1} Å node of record and these are not on disk:");
            for m in &missing {
                eprintln!("  {m}");
            }
            eprintln!("Nothing written.");
            std::process::exit(3);
        }
    };
    let refi = exact
        .iter()
        .position(|e| e.kind == "linear" && (e.r_ang - REF_ANGSTROM).abs() < 1e-9)
        .expect("the linear 2.9 Å node is of record");
    let far_b = exact[refi].b.translated([FAR_BOHR, 0.0, 0.0]);
    let r_bohr = cross_oo(&exact[refi].a, &far_b);
    let r_ang = r_bohr / ANGSTROM_TO_BOHR;
    eprintln!(
        "CT-1 far — the EXACT solve of `{FAR_NAME}` on {} threads: the linear {REF_ANGSTROM:.1} Å node's acceptor translated [{FAR_BOHR:.1}, 0, 0] bohr — the engine's own E_q({FAR_BOHR:.0}) geometry, O–O {r_bohr:.6} bohr ({r_ang:.4} Å), {N_DET_DIMER} determinants, residual bar {RESIDUAL_BAR:e}, price band {S2_CPU_LO}–{S2_CPU_HI} core-seconds.\n  T0's far leg reads E_exact from THIS file; E_A0 + E_B0 is not E_exact at 40 bohr (the electrostatics of the linear dimer survive there, of order 1e-5 hartree) and is only REPORTED beside the gate.",
        threads()
    );
    let ok = solve_node(out, FAR_NAME, r_ang, &exact[refi].a, &far_b, true);
    eprintln!("far: the exact solve converged within its bar: {ok}");
    fs::write(out.join("far.done"), "done\n").unwrap();
    eprintln!("far.done written");
}

// --------------------------------------------------------------------------- the fit phase

fn run_fit(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    eprintln!("CT-1 fit — T0 (the sector is what it says), T1 (attractive and short-ranged), T2 (the decomposition), S1 (the CT term), C1 (the contact term re-fit on what remains), dispersion, G-B0, G-C1, plants (i) and (ii)");

    let exact = match exact_records(out, o, h) {
        Ok(v) => v,
        Err(missing) => {
            eprintln!("REFUSED — the twelve exact geometries inside the identity are not all on disk:");
            for m in &missing {
                eprintln!("  {m}");
            }
            eprintln!("Nothing written.");
            std::process::exit(3);
        }
    };
    assert_eq!(exact.len(), 12, "the exact record is twelve geometries INSIDE the identity");
    let n = exact.len();

    // the sector records, one per node, plus the 40-bohr reference
    let mut missing: Vec<String> = Vec::new();
    let mut sectors: Vec<Sector> = Vec::with_capacity(n);
    for e in &exact {
        match load_sector(out, &e.name) {
            Ok(s) => sectors.push(s),
            Err(m) => missing.push(m),
        }
    }
    let far = match load_sector(out, FAR_NAME) {
        Ok(s) => Some(s),
        Err(m) => {
            missing.push(m);
            None
        }
    };
    if !missing.is_empty() {
        eprintln!("REFUSED — the sector readings are not on disk (run `sector` first):");
        for m in &missing {
            eprintln!("  {m}");
        }
        eprintln!("Nothing written.");
        std::process::exit(3);
    }
    let far = far.expect("checked above");

    // T0's far leg reads E_exact at 40 bohr from its OWN exact solve (the `far` phase). The
    // non-interacting limit E_A0 + E_B0 is NOT E_exact there — the linear dimer's electrostatics
    // survive at 40 bohr, of order 1e-5 hartree, and the block-localised sector carries them too —
    // so the limit is REPORTED beside the gate below and is never the gate.
    let far_exact_path = out.join(format!("{FAR_NAME}.json"));
    let far_text = match fs::read_to_string(&far_exact_path) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("REFUSED — T0's far leg is |E_exact − E_noCT| ≤ {FAR_CT_TOL:e} at {FAR_BOHR:.0} bohr, and the EXACT solve of `{FAR_NAME}` is not on disk:");
            eprintln!("  {}", far_exact_path.display());
            eprintln!("  Run `ct1_harvest far` first. E_A0 + E_B0 is NOT E_exact at {FAR_BOHR:.0} bohr: the exact dimer still carries the linear dimer's electrostatics there, of order 1e-5 hartree, and the block-localised sector carries them too, so the limit would pass or fail the gate for the wrong reason. No record supplies this number — FIELD-6's exchange_far.json is the Heitler–London referee, not a full-space solve.");
            eprintln!("Nothing written.");
            std::process::exit(3);
        }
    };
    let far_exact = json_num(&far_text, "e_super");
    let far_de_exact = json_num(&far_text, "de_exact");
    let far_exact_converged = json_bool(&far_text, "converged");
    assert!(far_exact.is_finite(), "{} carries no e_super", far_exact_path.display());

    // FIELD-9's wall, held
    let wall9_path = sibling(out, "field9").join("wall9.json");
    let wall9 = fs::read_to_string(&wall9_path).unwrap_or_else(|_| panic!("{} is the wall this harvest HOLDS; run FIELD-9's fit first", wall9_path.display()));
    let (a_oo, b_oo) = (json_num(&wall9, "a"), json_num(&wall9, "b"));
    let (a_oh, b_oh) = (json_num(&wall9, "a_oh"), json_num(&wall9, "b_oh"));
    let (a_hh, b_hh) = (json_num(&wall9, "a_hh"), json_num(&wall9, "b_hh"));
    let (r_min_oo, r_min_oh, r_min_hh) = (json_num(&wall9, "r_min_oo"), json_num(&wall9, "r_min_oh"), json_num(&wall9, "r_min_hh"));
    let q_h_record = json_num(&wall9, "q_h");
    assert!(
        a_oo.is_finite() && b_oo.is_finite() && a_oh.is_finite() && b_oh.is_finite() && a_hh.is_finite() && b_hh.is_finite() && r_min_oo.is_finite() && r_min_oh.is_finite() && r_min_hh.is_finite(),
        "wall9.json carries no three-class wall and no r_min"
    );
    eprintln!(
        "\nFIELD-9's wall, HELD [{}]:\n  a = {a_oo:.9e}, b = {b_oo:.6}; a_oh = {a_oh:.9e}, b_oh = {b_oh:.6}; a_hh = {a_hh:.9e}, b_hh = {b_hh:.6}\n  r_min: O–O {r_min_oo:.6}, H–O {r_min_oh:.6}, H–H {r_min_hh:.6} bohr; q_H of record {q_h_record:.12e}",
        wall9_path.display()
    );
    let wall_at = |e: &ENode| -> f64 { a_oo * sum_exp(&e.oo, b_oo) + a_oh * sum_exp(&e.ho, b_oh) + a_hh * sum_exp(&e.hh, b_hh) };

    // ================================================================= T0 — the sector is what it says
    eprintln!("\nT0 — the sector is what it says (CT1_AMENDMENT_1): the generalised residual ≤ {SECTOR_RESIDUAL_BAR:e}, exit Converged, the subspace dimension {SECTOR_DIM_STAKED} EXACT, E_exact ≤ E_noCT ≤ E_HL(undeformed) on TOTAL energies within {ORDER_TOL:e}, and |E_CT| ≤ {FAR_CT_TOL:e} at {FAR_BOHR:.0} bohr.");
    let mut e_ct: Vec<f64> = Vec::with_capacity(n);
    let mut t0_fails: Vec<String> = Vec::new();
    let mut t0_lines: Vec<String> = Vec::new();
    let mut price_refused: Vec<String> = Vec::new();
    eprintln!("| node | E_exact | E_noCT | E_HL(undef) | E_CT | order | dim | iters | residual | converged | core-s | priced |");
    for (g, e) in exact.iter().enumerate() {
        let s = &sectors[g];
        let ct = e.e_super - s.e_noct;
        e_ct.push(ct);
        let order_ok = e.e_super <= s.e_noct + ORDER_TOL && s.e_noct <= s.e_hl_undeformed + ORDER_TOL;
        let dim_ok = s.sector_dim == SECTOR_DIM_STAKED;
        let res_ok = s.residual <= SECTOR_RESIDUAL_BAR;
        let ok = order_ok && dim_ok && res_ok && s.converged;
        if !ok {
            t0_fails.push(format!(
                "{} (converged {}, residual {:.3e}, dim {}, order {order_ok})",
                e.name, s.converged, s.residual, s.sector_dim
            ));
        }
        if !s.price_admitted {
            price_refused.push(format!("{} ({:.0} core-s for {} iterations)", e.name, s.cpu_seconds, s.davidson_iters));
        }
        eprintln!(
            "| {} | {:.12e} | {:.12e} | {:.12e} | {ct:.6e} | {order_ok} | {} | {} | {:.3e} | {} | {:.0} | {} |",
            e.name, e.e_super, s.e_noct, s.e_hl_undeformed, s.sector_dim, s.davidson_iters, s.residual, s.converged, s.cpu_seconds, s.price_admitted
        );
        t0_lines.push(format!(
            "{{\"node\": \"{}\", \"converged\": {}, \"residual\": {}, \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}, \"residual_ok\": {res_ok}, \"sector_dim\": {}, \"sector_dim_ok\": {dim_ok}, \"n_det\": {}, \"davidson_iters\": {}, \"metric_min_eigenvalue\": {}, \"e_exact_total\": {}, \"e_noct\": {}, \"e_hl_undeformed\": {}, \"order_ok\": {order_ok}, \"e_ct\": {}, \"cpu_seconds\": {}, \"wall_seconds\": {}, \"price_admitted\": {}, \"source\": \"{}\", \"exact_source\": \"{}\"}}",
            e.name,
            s.converged,
            jn(s.residual),
            s.sector_dim,
            s.n_det,
            s.davidson_iters,
            jn(s.metric_min_eigenvalue),
            jn(e.e_super),
            jn(s.e_noct),
            jn(s.e_hl_undeformed),
            jn(ct),
            jn(s.cpu_seconds),
            jn(s.wall_seconds),
            s.price_admitted,
            esc(&s.source),
            esc(&e.source),
        ));
    }
    // the 40-bohr leg, on the EXACT solve of that geometry (the `far` phase)
    let far_ct = far_exact - far.e_noct;
    let far_noct_minus_monomers = far.e_noct - (far.e_a0 + far.e_b0);
    let far_ok = far_ct.abs() <= FAR_CT_TOL && far.converged && far.residual <= SECTOR_RESIDUAL_BAR && far.sector_dim == SECTOR_DIM_STAKED && far_exact_converged;
    eprintln!(
        "  {FAR_NAME}: E_exact {far_exact:.12e} (the EXACT solve of this geometry, converged {far_exact_converged} [{}]), E_noCT {:.12e}, E_HL(undeformed) {:.12e} → |E_CT| = {:.3e} (bar {FAR_CT_TOL:e}); sector dim {}, {} iters, residual {:.3e}, converged {}, {:.0} core-s, priced {}",
        far_exact_path.display(),
        far.e_noct,
        far.e_hl_undeformed,
        far_ct.abs(),
        far.sector_dim,
        far.davidson_iters,
        far.residual,
        far.converged,
        far.cpu_seconds,
        far.price_admitted
    );
    eprintln!(
        "  reported beside the gate, NEVER as it: at {FAR_BOHR:.0} bohr the exact dimer keeps the linear dimer's electrostatics, E_exact − (E_A0 + E_B0) = {far_de_exact:.6e} Ha, and the block-localised sector keeps them too, E_noCT − (E_A0 + E_B0) = {far_noct_minus_monomers:.6e} Ha; their difference IS the E_CT above, which is why the monomer limit cannot stand in for E_exact here"
    );
    if !far.price_admitted {
        price_refused.push(format!("{FAR_NAME} ({:.0} core-s for {} iterations)", far.cpu_seconds, far.davidson_iters));
    }
    let t0 = t0_fails.is_empty() && far_ok;
    eprintln!(
        "T0: {} — {} of {n} nodes pass every leg, the {FAR_BOHR:.0}-bohr limit {}{}",
        if t0 { "PASS" } else { "FAIL" },
        n - t0_fails.len(),
        if far_ok { "holds" } else { "FAILS" },
        if t0_fails.is_empty() { String::new() } else { format!("; failures: {}", t0_fails.join(", ")) }
    );
    eprintln!(
        "M-CHEAPER-THAN-ITS-PRICE: every sector solve at or above a tenth of its own iteration count times {SIGMA_PRICE_FLOOR} core-seconds: {}{}",
        price_refused.is_empty(),
        if price_refused.is_empty() { String::new() } else { format!(" — REFUSED: {}", price_refused.join(", ")) }
    );

    // ============================================================ T1 — attractive and short-ranged
    let lin: Vec<usize> = (0..n).filter(|&g| exact[g].kind == "linear").collect();
    eprintln!("\nT1 — charge transfer is attractive on the line at {T1_LINE_LO}–{T1_LINE_HI} Å (E_CT < −{CT_FLOOR:e}), non-increasing in magnitude outward along the whole line, and its log-log slope beyond {OUTER_FROM_ANGSTROM} Å steeper than {T1_SLOPE_MAX}:");
    let mut t1_attractive = true;
    let mut t1_attr_misses: Vec<String> = Vec::new();
    for &g in lin.iter() {
        let r = exact[g].r_ang;
        if r >= T1_LINE_LO - 1e-9 && r <= T1_LINE_HI + 1e-9 {
            let ok = e_ct[g] < -CT_FLOOR;
            if !ok {
                t1_attractive = false;
                t1_attr_misses.push(exact[g].name.clone());
            }
            eprintln!("  {}: E_CT {:.6e} Ha < −{CT_FLOOR:e}: {ok}", exact[g].name, e_ct[g]);
        } else {
            eprintln!("  {}: E_CT {:.6e} Ha (outside the 2.3–3.4 Å line — the magnitude order still reads on it)", exact[g].name, e_ct[g]);
        }
    }
    let mut t1_monotone = true;
    let mut mono_breaks: Vec<String> = Vec::new();
    for i in 1..lin.len() {
        let (p, c) = (lin[i - 1], lin[i]);
        let ok = e_ct[c].abs() <= e_ct[p].abs();
        if !ok {
            t1_monotone = false;
            mono_breaks.push(format!("{} → {}", exact[p].name, exact[c].name));
        }
        eprintln!("  |E_CT| {:.1} Å {:.6e} → {:.1} Å {:.6e}: non-increasing {ok}", exact[p].r_ang, e_ct[p].abs(), exact[c].r_ang, e_ct[c].abs());
    }
    let mut t1_slopes: Vec<(f64, f64, f64)> = Vec::new();
    let mut t1_slope_ok = true;
    for i in 1..lin.len() {
        let (p, c) = (lin[i - 1], lin[i]);
        if exact[p].r_ang < OUTER_FROM_ANGSTROM - 1e-9 {
            continue;
        }
        let s = if e_ct[c] != 0.0 && e_ct[p] != 0.0 { (e_ct[c].abs() / e_ct[p].abs()).ln() / (exact[c].r_oo_bohr / exact[p].r_oo_bohr).ln() } else { f64::NAN };
        let ok = s.is_finite() && s < T1_SLOPE_MAX;
        if !ok {
            t1_slope_ok = false;
        }
        eprintln!("  log-log slope {:.1} → {:.1} Å: {s:.4} (needs < {T1_SLOPE_MAX}): {ok}", exact[p].r_ang, exact[c].r_ang);
        t1_slopes.push((exact[p].r_ang, exact[c].r_ang, s));
    }
    let t1 = t1_attractive && t1_monotone && t1_slope_ok;
    eprintln!(
        "T1: {} — attractive on the line {t1_attractive}{}, non-increasing outward {t1_monotone}{}, every slope beyond {OUTER_FROM_ANGSTROM} Å steeper than {T1_SLOPE_MAX} {t1_slope_ok}",
        if t1 { "PASS" } else { "FAIL" },
        if t1_attr_misses.is_empty() { String::new() } else { format!(" (misses: {})", t1_attr_misses.join(", ")) },
        if mono_breaks.is_empty() { String::new() } else { format!(" (breaks: {})", mono_breaks.join(", ")) }
    );

    // ==================================================================== T2 — the decomposition
    let refi = lin.iter().cloned().find(|&g| (exact[g].r_ang - REF_ANGSTROM).abs() < 1e-9).expect("the linear 2.9 Å node");
    let sr = &sectors[refi];
    let de_hl = sr.e_hl_undeformed - (sr.e_a0 + sr.e_b0);
    let gain = sr.e_noct - sr.e_hl_undeformed;
    let ct_ref = e_ct[refi];
    let remainder_field6 = exact[refi].e_super - sr.e_hl_undeformed;
    let identity_miss = ((de_hl + gain + ct_ref) - de_hl - remainder_field6).abs();
    eprintln!("\nT2 — the decomposition at the linear {REF_ANGSTROM:.1} Å node, in interaction energies (hartree, and mHa beside):");
    eprintln!("  ΔE_HL   = E_HL(undeformed) − (E_A0 + E_B0)  = {de_hl:.9e}  ({:.4} mHa)   — electrostatics and exchange", de_hl * 1e3);
    eprintln!("  gain    = E_noCT − E_HL(undeformed)         = {gain:.9e}  ({:.4} mHa)   — the closed sector's polarisation and correlation (channels 2 and 3 at this level)", gain * 1e3);
    eprintln!("  E_CT    = E_exact − E_noCT                  = {ct_ref:.9e}  ({:.4} mHa)   — charge transfer, channel 6", ct_ref * 1e3);
    eprintln!(
        "  their sum minus ΔE_HL = {:.9e} ({:.4} mHa) against E_exact − E_HL = {remainder_field6:.9e} ({:.4} mHa) — identity miss {identity_miss:.3e} (bar {ORDER_TOL:e}): {}",
        gain + ct_ref,
        (gain + ct_ref) * 1e3,
        remainder_field6 * 1e3,
        identity_miss <= ORDER_TOL
    );
    eprintln!("  beside FIELD-6's remainder at this node, {:.4} mHa: this harvest reads {:.4} mHa, of which {:.1} % is charge transfer", FIELD6_REMAINDER * 1e3, remainder_field6 * 1e3, 100.0 * ct_ref / remainder_field6);
    eprintln!(
        "  the monomer references agree with the exact record: E_A0 {:.12e} vs {:.12e}, E_B0 {:.12e} vs {:.12e} (differences {:.3e}, {:.3e})",
        sr.e_a0,
        exact[refi].e_a0_rec,
        sr.e_b0,
        exact[refi].e_b0_rec,
        (sr.e_a0 - exact[refi].e_a0_rec).abs(),
        (sr.e_b0 - exact[refi].e_b0_rec).abs()
    );
    let t2 = identity_miss <= ORDER_TOL;

    // ======================================================================== S1 — the CT term
    let we: Vec<f64> = exact.iter().map(|e| 1.0 / (e.de_exact.abs().max(WEIGHT_FLOOR)).powi(2)).collect();
    eprintln!(
        "\nthe CT term: CT(g) = −P_CT·Σ_{{cross-unit H–O}} exp(−c_CT·r) fit to E_CT on the twelve, c on 0.50..=4.00 step 0.02 ({NC} values), weights 1/max(|ΔE_exact|, {WEIGHT_FLOOR:e})² (THIS FREEZE'S CORRECTION of FIELD-7/8/9's 1/ΔE_exact², which pinned every fit at the 2.5 Å zero crossing), P_CT non-negative:"
    );
    let syy_ct: f64 = (0..n).map(|g| we[g] * e_ct[g] * e_ct[g]).sum();
    let t_ctfit = Instant::now();
    let (mut ct_res, mut ct_best) = (f64::INFINITY, (0usize, 0.0f64, 0.0f64));
    for i in 0..NC {
        let c = cgrid(i);
        let (mut num, mut den) = (0.0f64, 0.0f64);
        for (g, e) in exact.iter().enumerate() {
            let x = -sum_exp(&e.ho, c);
            num += we[g] * x * e_ct[g];
            den += we[g] * x * x;
        }
        let p_raw = if den > 0.0 { num / den } else { 0.0 };
        let p_use = if p_raw >= 0.0 { p_raw } else { 0.0 };
        let r: f64 = (0..n)
            .map(|g| {
                let m = -p_use * sum_exp(&exact[g].ho, c);
                we[g] * (e_ct[g] - m) * (e_ct[g] - m)
            })
            .sum();
        if r < ct_res {
            ct_res = r;
            ct_best = (i, p_use, p_raw);
        }
    }
    let (cti, p_ct, p_ct_raw) = ct_best;
    let c_ct = cgrid(cti);
    let ct_transferred = p_ct > 0.0;
    let ct_seconds = t_ctfit.elapsed().as_secs_f64();
    eprintln!(
        "  P_CT = {p_ct:.9e} Ha (unclamped {p_ct_raw:.9e}), c_CT = {c_ct:.2} /bohr, weighted residual {ct_res:.9e} (against Σw·E_CT² = {syy_ct:.9e}), in {ct_seconds:.1} s → {}",
        if ct_transferred { "TRANSFERRED" } else { "NOT transferred (P_CT clamped to an exact 0.0)" }
    );
    let ct_term = |ho: &[f64]| -> f64 { -p_ct * sum_exp(ho, c_ct) };
    let mut s1_within = 0usize;
    let mut s1_misses: Vec<String> = Vec::new();
    let mut ct_fit_v: Vec<f64> = Vec::with_capacity(n);
    eprintln!("| node | E_CT | CT fit | miss | tolerance | within |");
    for (g, e) in exact.iter().enumerate() {
        let f = ct_term(&e.ho);
        ct_fit_v.push(f);
        let miss = (f - e_ct[g]).abs();
        let tol = (S1_FRAC * e.de_exact.abs()).max(S1_ABS);
        let ok = miss <= tol;
        if ok {
            s1_within += 1;
        } else {
            s1_misses.push(e.name.clone());
        }
        eprintln!("| {} | {:.6e} | {f:.6e} | {miss:.6e} | {tol:.6e} | {ok} |", e.name, e_ct[g]);
    }
    let s1_branch = if s1_within == n {
        "a"
    } else if s1_within >= S1_B_MIN {
        "b"
    } else {
        "c"
    };
    eprintln!(
        "S1: {s1_within} of {n} within max({S1_FRAC}·|ΔE_exact|, {S1_ABS:e}) → branch ({s1_branch}) — {}",
        match s1_branch {
            "a" => "the CT term is one exponential on the contact across all twelve".to_string(),
            "b" => format!("transferred, the {} misses named: {}", s1_misses.len(), s1_misses.join(", ")),
            _ => format!("the term is NOT transferred and its shape is read: {} misses: {}", s1_misses.len(), s1_misses.join(", ")),
        }
    );

    // ============================================ C1 — the contact term re-fit on what remains
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
        eprintln!("Nothing written.");
        std::process::exit(4);
    }
    eprintln!("\nevery one of the twelve is served with TWO units by the engine (the closure identity holds at each) — the contact re-fit may use them");
    eprintln!("the contact term re-fit: remainder = ΔE_exact − [E_q(g) − E_q({FAR_BOHR:.0})]_engine − wall9(g) − CT(g), two classes (H–O, H–H), same grid and weights:");
    let mut e_q: Vec<f64> = Vec::with_capacity(n);
    let mut rem: Vec<f64> = Vec::with_capacity(n);
    let mut wall_e: Vec<f64> = Vec::with_capacity(n);
    for (g, e) in exact.iter().enumerate() {
        let (_, e_q_diff, _) = engine_interaction(&e.a, &e.b, None, SeamPlant::None);
        let wv = wall_at(e);
        let r = e.de_exact - e_q_diff - wv - ct_fit_v[g];
        eprintln!(
            "  {} ({}): ΔE_exact {:.6e}, E_q(g) − E_q({FAR_BOHR:.0}) {e_q_diff:.6e}, wall9 {wv:.6e}, CT {:.6e} → remainder {r:.6e} Ha ({:.4} of |ΔE_exact|)",
            e.name,
            e.kind,
            e.de_exact,
            ct_fit_v[g],
            r / e.de_exact.abs()
        );
        e_q.push(e_q_diff);
        wall_e.push(wv);
        rem.push(r);
    }
    let syy_c: f64 = (0..n).map(|g| we[g] * rem[g] * rem[g]).sum();
    let mut cho = vec![0.0f64; NC * n];
    let mut chh = vec![0.0f64; NC * n];
    for i in 0..NC {
        let c = cgrid(i);
        for (g, e) in exact.iter().enumerate() {
            cho[i * n + g] = -sum_exp(&e.ho, c);
            chh[i * n + g] = -sum_exp(&e.hh, c);
        }
    }
    let t_cfit = Instant::now();
    let (mut c_res, mut c_best) = (f64::INFINITY, (0usize, 0usize, [0.0f64; 2], [false; 2]));
    for i in 0..NC {
        for j in 0..NC {
            let (mut m00, mut m01, mut m11, mut r0, mut r1) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
            for g in 0..n {
                let (x0, x1) = (cho[i * n + g], chh[j * n + g]);
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
        "\nthe two-class contact re-fit — {} pairs on 0.50..=4.00 step 0.02 per class, weights 1/max(|ΔE_exact|, {WEIGHT_FLOOR:e})², non-negative by drop-and-refit, in {cfit_seconds:.1} s:",
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
    let mut contact_v: Vec<f64> = Vec::with_capacity(n);
    let mut c1_ok_v: Vec<bool> = Vec::with_capacity(n);
    for (g, e) in exact.iter().enumerate() {
        let f = contact(&e.ho, &e.hh);
        contact_v.push(f);
        let miss = (rem[g] - f).abs();
        let t = (S1_FRAC * e.de_exact.abs()).max(S1_ABS);
        let ok = miss <= t;
        c1_ok_v.push(ok);
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
        }
        eprintln!("  {}: remainder {:.6e}, fit {f:.6e}, miss {miss:.6e} ({:.4} of its tolerance {t:.3e}) → {}", e.name, rem[g], miss / t, if ok { "within" } else { "MISS" });
    }
    let c1_line_ok = c1_line_within == c1_line_of && c1_line_of == LINE_ANGSTROM.len();
    let c1 = c1_within >= C1_MIN && c1_line_ok;
    eprintln!(
        "C1: {c1_within} of {n} within max({S1_FRAC}·|ΔE_exact|, {S1_ABS:e}) (needs ≥ {C1_MIN}), and the line at 2.7, 2.9, 3.1 Å all within: {c1_line_ok} ({c1_line_within} of {c1_line_of}) → {}{}",
        if c1 { "PASS" } else { "FAIL" },
        if c1_misses.is_empty() { String::new() } else { format!("; misses: {}", c1_misses.join(", ")) }
    );

    // ------------------- dispersion: what is left on the FOUR OUTER LINEAR nodes only (FIELD-6)
    let outer: Vec<usize> = (0..n).filter(|&g| exact[g].outer_linear).collect();
    let rem2: Vec<f64> = outer.iter().map(|&g| rem[g] - contact_v[g]).collect();
    let (mut num, mut den) = (0.0f64, 0.0f64);
    for (oi, &g) in outer.iter().enumerate() {
        let x = -1.0 / exact[g].r_oo_bohr.powi(6);
        num += we[g] * rem2[oi] * x;
        den += we[g] * x * x;
    }
    let mut c6 = if den > 0.0 { num / den } else { 0.0 };
    let mut slopes: Vec<(f64, f64)> = Vec::new();
    eprintln!("\ndispersion — the remainder AFTER both contact terms, on the {} outer linear nodes (R_OO ≥ {OUTER_FROM_ANGSTROM} Å):", outer.len());
    for (oi, &g) in outer.iter().enumerate() {
        eprintln!("  {:.1} Å: remainder after contact {:.6e} Ha ({:.4} of |ΔE_exact|)", exact[g].r_ang, rem2[oi], rem2[oi] / exact[g].de_exact.abs());
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

    // ------------------------------------------------ the full law, then G-B0 by the engine
    let model = SeamModel { a: a_oo, b: b_oo, p: p_ho, c: c_ho, c6, a_oh, b_oh, a_hh, b_hh, p_hh, c_hh, p_ct, c_ct, ..SeamModel::NO_WALL };
    let q_h = holon_render::field::water_charge_at_pin();
    let r_min = [r_min_oo, r_min_oh, r_min_hh];
    let bounded = model.bounded(q_h, r_min, KT);
    let g_b0 = bounded.is_none();
    let arms_void = bounded.is_some();
    let hole = model.hole(q_h);
    eprintln!(
        "\nG-B0 — bounded: the engine walks each cross-unit class potential of the FULL law (the CT term IN) from its own r_min (O–O {r_min_oo:.6}, H–O {r_min_oh:.6}, H–H {r_min_hh:.6} bohr) inward to 0.5 bohr on a 0.05 grid, kT = {KT:e} Ha, q_H = {q_h:.12e} (wall9's record {q_h_record:.12e}, difference {:.3e})",
        (q_h - q_h_record).abs()
    );
    match &bounded {
        None => eprintln!("  bounded() returned none → PASS: no class falls more than kT below its fit floor, and each is positive at contact; the arms may run"),
        Some(msg) => eprintln!("  bounded() named a fall, verbatim: {msg}\n  → the arms are VOID before they run (§2 G-B0). The harvest is still recorded in full."),
    }
    match &hole {
        None => eprintln!("  hole() (a READING beside the gate, not a gate): none — the law also rises monotonically inward from 3.0 bohr"),
        Some(msg) => eprintln!("  hole() (a READING beside the gate, not a gate) names a fall, verbatim: {msg}"),
    }

    // ------------------------------------------ G-C1, plant (ii), and plant (i) read back
    eprintln!("\nG-C1 — the harvest is the engine's arithmetic, ONE reference (E_q(g) − E_q({FAR_BOHR:.0}) from the engine itself), on the twelve, with the CT term in the formula");
    let mut g_c1_worst = 0.0f64;
    let mut units_ok = true;
    let mut g_c1_v: Vec<(f64, f64, f64, u64, u64, u64)> = Vec::with_capacity(n);
    let mut plant_ii = (f64::NAN, f64::NAN, f64::NAN, false);
    for (g, e) in exact.iter().enumerate() {
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
            "  {}: engine {e_int:.12e} vs formula {want:.12e} — miss {miss:.3e} (E_q diff {e_field_diff:.6e}, seam {e_seam_diff:.6e}; pen_HO {:.6e}, pen_HH {:.6e}, wall_OO {:.6e}, wall_OH {:.6e}, wall_HH {:.6e}, disp {:.6e}, CT {:.6e}; units {units}, O–O {oo_pairs}, H–O {ho_pairs})",
            e.name, t.pen_ho, t.pen_hh, t.w_oo, t.w_oh, t.w_hh, t.disp, t.ct
        );
        g_c1_v.push((e_int, want, miss, units, oo_pairs, ho_pairs));
        if g == refi {
            // plant (ii): P_CT → −P_CT must move the engine by exactly 2·|CT(2.9 Å)|
            let (e_pl, _, _) = engine_interaction(&e.a, &e.b, Some(model), SeamPlant::FlipChargeTransfer);
            let observed = (e_pl - e_int).abs();
            let expected = 2.0 * t.ct.abs();
            let carrier = t.ct.abs();
            let fires = carrier >= PLANT_II_CARRIER && (observed - expected).abs() <= G_C1_TOL;
            plant_ii = (observed, expected, carrier, fires);
            eprintln!(
                "plant (ii) at the linear {REF_ANGSTROM:.1} Å node (P_CT → −P_CT): miss {observed:.6e} vs 2·|CT(2.9)| {expected:.6e} (difference {:.3e}, bar {G_C1_TOL:e}); carrier |CT(2.9)| {carrier:.3e} ≥ {PLANT_II_CARRIER:e}: {} → {}",
                (observed - expected).abs(),
                carrier >= PLANT_II_CARRIER,
                if fires { "FIRES" } else { "does not fire" }
            );
        }
    }
    let g_c1 = g_c1_worst <= G_C1_TOL;
    eprintln!("G-C1: worst |engine − formula| = {g_c1_worst:.3e} (stake {G_C1_TOL:e}) → {}", if g_c1 { "PASS" } else { "FAIL" });
    eprintln!("M-VACUOUS-SUCCESS: every G-C1 geometry served two units, one cross O–O pair and four cross H–O pairs: {units_ok}");

    // plant (i), from the sector phase's own record
    let plant_i_path = out.join("plant_i_full.json");
    let plant_i_text = fs::read_to_string(&plant_i_path).unwrap_or_default();
    let e_full = json_num(&plant_i_text, "e_full");
    let plant_i_miss = (e_full - exact[refi].e_super).abs();
    let plant_i_carrier = ct_ref.abs();
    let plant_i_fires = e_full.is_finite() && plant_i_miss <= PLANT_I_TOL && plant_i_carrier >= PLANT_I_CARRIER && json_bool(&plant_i_text, "converged");
    eprintln!(
        "\nplant (i) — the mask dropped (from {}): E_full {e_full:.12e} vs E_exact of record {:.12e} — miss {plant_i_miss:.3e} (bar {PLANT_I_TOL:e}), converged {}, {} iterations, residual {:.3e}; carrier |E_CT(2.9)| {plant_i_carrier:.3e} ≥ {PLANT_I_CARRIER:e}: {} → {}",
        plant_i_path.display(),
        exact[refi].e_super,
        json_bool(&plant_i_text, "converged"),
        json_num(&plant_i_text, "davidson_iters"),
        json_num(&plant_i_text, "residual"),
        plant_i_carrier >= PLANT_I_CARRIER,
        if plant_i_fires { "FIRES" } else { "does not fire" }
    );

    // ------------------------------------------------------------------------ wall_ct.json
    let mut node_lines: Vec<String> = Vec::with_capacity(n);
    for (g, e) in exact.iter().enumerate() {
        let s = &sectors[g];
        let ct_miss = (ct_fit_v[g] - e_ct[g]).abs();
        let tol = (S1_FRAC * e.de_exact.abs()).max(S1_ABS);
        let c1_miss = (rem[g] - contact_v[g]).abs();
        let (e_int, want, miss, units, oo_pairs, ho_pairs) = g_c1_v[g];
        node_lines.push(format!(
            "{{\"node\": \"{}\", \"kind\": \"{}\", \"r_angstrom\": {:.1}, \"r_oo_bohr\": {}, \"line_node\": {}, \"outer_linear\": {}, \"exact_source\": \"{}\", \"sector_source\": \"{}\", \"e_exact_total\": {}, \"e_noct\": {}, \"e_hl_undeformed\": {}, \"e_a0\": {}, \"e_b0\": {}, \"de_exact\": {}, \"e_ct\": {}, \"delta_e_hl\": {}, \"closed_sector_gain\": {}, \"identity_miss\": {}, \"ct_fit\": {}, \"ct_miss\": {}, \"tolerance\": {}, \"ct_within\": {}, \"e_q_difference\": {}, \"wall9\": {}, \"remainder\": {}, \"contact_fit\": {}, \"c1_miss\": {}, \"c1_within\": {}, \"g_c1_engine\": {}, \"g_c1_formula\": {}, \"g_c1_miss\": {}, \"units\": {units}, \"oo_pairs\": {oo_pairs}, \"ho_pairs\": {ho_pairs}, \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}]}}",
            e.name,
            e.kind,
            e.r_ang,
            jd(e.r_oo_bohr),
            e.line_node,
            e.outer_linear,
            esc(&e.source),
            esc(&s.source),
            jn(e.e_super),
            jn(s.e_noct),
            jn(s.e_hl_undeformed),
            jn(s.e_a0),
            jn(s.e_b0),
            jn(e.de_exact),
            jn(e_ct[g]),
            jn(s.e_hl_undeformed - (s.e_a0 + s.e_b0)),
            jn(s.e_noct - s.e_hl_undeformed),
            jn(((s.e_noct - s.e_hl_undeformed) + e_ct[g] - (e.e_super - s.e_hl_undeformed)).abs()),
            jn(ct_fit_v[g]),
            jn(ct_miss),
            jn(tol),
            ct_miss <= tol,
            jn(e_q[g]),
            jn(wall_e[g]),
            jn(rem[g]),
            jn(contact_v[g]),
            jn(c1_miss),
            c1_ok_v[g],
            jn(e_int),
            jn(want),
            jn(miss),
            list_json(&e.oo),
            list_json(&e.ho),
            list_json(&e.hh),
        ));
    }
    let t1_slope_lines: Vec<String> = t1_slopes.iter().map(|(r0, r1, s)| format!("{{\"from_angstrom\": {r0:.1}, \"to_angstrom\": {r1:.1}, \"loglog_slope\": {}, \"steeper_than_max\": {}}}", jn(*s), s.is_finite() && *s < T1_SLOPE_MAX)).collect();
    let slope_lines: Vec<String> = slopes.iter().map(|(r, s)| format!("{{\"r_angstrom\": {r:.1}, \"loglog_slope_from_previous\": {}}}", jn(*s))).collect();
    let s1_miss_names: Vec<String> = s1_misses.iter().map(|m| format!("\"{m}\"")).collect();
    let c1_miss_names: Vec<String> = c1_misses.iter().map(|m| format!("\"{m}\"")).collect();
    let t0_fail_names: Vec<String> = t0_fails.iter().map(|m| format!("\"{}\"", esc(m))).collect();
    let price_names: Vec<String> = price_refused.iter().map(|m| format!("\"{}\"", esc(m))).collect();
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
        "  \"a\": {}, \"b\": {}, \"p\": {}, \"c\": {}, \"c6\": {}, \"a_oh\": {}, \"b_oh\": {}, \"a_hh\": {}, \"b_hh\": {}, \"p_hh\": {}, \"c_hh\": {}, \"p_ct\": {}, \"c_ct\": {},\n",
        jn(a_oo),
        jn(b_oo),
        jn(p_ho),
        jn(c_ho),
        jn(c6),
        jn(a_oh),
        jn(b_oh),
        jn(a_hh),
        jn(b_hh),
        jn(p_hh),
        jn(c_hh),
        jn(p_ct),
        jn(c_ct)
    ));
    j.push_str(&format!(
        "  \"r_min_oo\": {}, \"r_min_oh\": {}, \"r_min_hh\": {}, \"q_h\": {}, \"q_h_record\": {}, \"kt\": {},\n  \"wall_source\": \"{}\", \"wall_rule\": \"FIELD-9's three-class wall is HELD at this harvest; only the contact terms and the dispersion are re-fit, beside the new CT term\",\n",
        jn(r_min_oo),
        jn(r_min_oh),
        jn(r_min_hh),
        jn(q_h),
        jn(q_h_record),
        jn(KT),
        esc(&wall9_path.display().to_string())
    ));
    j.push_str(&format!(
        "  \"t0\": {{\"pass\": {t0}, \"rule\": \"CT1_AMENDMENT_1: the generalised residual ≤ {SECTOR_RESIDUAL_BAR:e}, exit Converged, subspace dimension {SECTOR_DIM_STAKED} EXACT, E_exact ≤ E_noCT ≤ E_HL(undeformed) on TOTAL energies within {ORDER_TOL:e}, |E_CT| ≤ {FAR_CT_TOL:e} at {FAR_BOHR:.0} bohr\", \"nodes_passing\": {}, \"nodes\": {n}, \"failures\": [{}], \"order_tolerance\": {ORDER_TOL:e}, \"sector_dim_staked\": {SECTOR_DIM_STAKED},\n    \"far\": {{\"node\": \"{FAR_NAME}\", \"geometry\": \"the linear {REF_ANGSTROM:.1} Å node's acceptor translated [{FAR_BOHR:.1}, 0, 0] bohr — the engine's own E_q({FAR_BOHR:.0}) geometry\", \"e_exact\": {}, \"e_exact_rule\": \"the EXACT full-space solve of THIS geometry (the `far` phase), which is what T0's far leg reads; no prior record carries it — FIELD-6's exchange_far.json is the Heitler–London referee, not a full-space solve\", \"e_exact_converged\": {}, \"e_exact_source\": \"{}\", \"e_noct\": {}, \"e_hl_undeformed\": {}, \"e_ct\": {}, \"tolerance\": {FAR_CT_TOL:e}, \"within\": {}, \"e_exact_minus_monomers_at_40\": {}, \"e_noct_minus_monomers_at_40\": {}, \"monomer_limit_is_reported_not_gated\": \"E_A0 + E_B0 is NOT E_exact at {FAR_BOHR:.0} bohr: the linear dimer's electrostatics survive there in BOTH the exact dimer and the block-localised sector, so the two numbers beside this line are reported for comparison and never used as the gate\", \"converged\": {}, \"residual\": {}, \"sector_dim\": {}, \"davidson_iters\": {}, \"cpu_seconds\": {}, \"price_admitted\": {}, \"source\": \"{}\"}},\n    \"price\": {{\"rule\": \"cpu_seconds ≥ {PRICE_TENTH} · davidson_iters · {SIGMA_PRICE_FLOOR} core-seconds (M-CHEAPER-THAN-ITS-PRICE)\", \"all_admitted\": {}, \"refused\": [{}]}},\n    \"per_node\": [{}]}},\n",
        n - t0_fails.len(),
        t0_fail_names.join(", "),
        jn(far_exact),
        far_exact_converged,
        esc(&far_exact_path.display().to_string()),
        jn(far.e_noct),
        jn(far.e_hl_undeformed),
        jn(far_ct),
        far_ct.abs() <= FAR_CT_TOL,
        jn(far_de_exact),
        jn(far_noct_minus_monomers),
        far.converged,
        jn(far.residual),
        far.sector_dim,
        far.davidson_iters,
        jn(far.cpu_seconds),
        far.price_admitted,
        esc(&far.source),
        price_refused.is_empty(),
        price_names.join(", "),
        t0_lines.join(", ")
    ));
    j.push_str(&format!(
        "  \"t1\": {{\"pass\": {t1}, \"attractive_on_the_line\": {t1_attractive}, \"line_angstrom\": [{T1_LINE_LO}, {T1_LINE_HI}], \"floor\": {CT_FLOOR:e}, \"non_increasing_outward\": {t1_monotone}, \"slopes_steeper_than\": {T1_SLOPE_MAX}, \"slopes_ok\": {t1_slope_ok}, \"slopes\": [{}]}},\n",
        t1_slope_lines.join(", ")
    ));
    j.push_str(&format!(
        "  \"t2\": {{\"pass\": {t2}, \"node\": \"{}\", \"delta_e_hl\": {}, \"closed_sector_gain\": {}, \"e_ct\": {}, \"sum_minus_delta_e_hl\": {}, \"e_exact_minus_e_hl\": {}, \"identity_miss\": {}, \"tolerance\": {ORDER_TOL:e}, \"field6_remainder\": {}, \"charge_transfer_fraction_of_the_remainder\": {}}},\n",
        exact[refi].name,
        jn(de_hl),
        jn(gain),
        jn(ct_ref),
        jn(gain + ct_ref),
        jn(remainder_field6),
        jn(identity_miss),
        jn(FIELD6_REMAINDER),
        jn(ct_ref / remainder_field6)
    ));
    j.push_str(&format!(
        "  \"s1\": {{\"branch\": \"{s1_branch}\", \"within\": {s1_within}, \"of\": {n}, \"branch_b_minimum\": {S1_B_MIN}, \"tolerance_rule\": \"max({S1_FRAC}·|ΔE_exact|, {S1_ABS:e})\", \"misses\": [{}]}},\n",
        s1_miss_names.join(", ")
    ));
    j.push_str(&format!(
        "  \"ct_fit\": {{\"p_ct\": {}, \"c_ct\": {}, \"p_ct_unclamped\": {}, \"transferred\": {ct_transferred}, \"placement\": \"cross-unit H–O\", \"shape\": \"−P_CT·exp(−c_CT·r), summed over the cross-unit H–O pairs\", \"grid\": \"0.50..=4.00 per bohr, step 0.02\", \"grid_points\": {NC}, \"weights\": \"1/max(|ΔE_exact|, {WEIGHT_FLOOR:e})² — this freeze's stated correction of FIELD-7/8/9's 1/ΔE_exact²\", \"weight_floor\": {WEIGHT_FLOOR:e}, \"nonnegativity\": \"P_CT below zero is clamped to an exact 0.0 and the term is NOT transferred\", \"weighted_residual\": {}, \"weighted_total\": {}, \"fit_seconds\": {}}},\n",
        jn(p_ct),
        jn(c_ct),
        jn(p_ct_raw),
        jn(ct_res),
        jn(syy_ct),
        jd(ct_seconds)
    ));
    j.push_str(&format!(
        "  \"c1\": {{\"pass\": {c1}, \"within\": {c1_within}, \"of\": {n}, \"required\": {C1_MIN}, \"line_within\": {c1_line_within}, \"line_nodes\": {c1_line_of}, \"line_ok\": {c1_line_ok}, \"line_rule\": \"the linear nodes at 2.7, 2.9 and 3.1 Å — where the bond lives — must ALL be within\", \"tolerance_rule\": \"max({S1_FRAC}·|ΔE_exact|, {S1_ABS:e})\", \"misses\": [{}]}},\n",
        c1_miss_names.join(", ")
    ));
    j.push_str(&format!(
        "  \"contact_fit\": {{\"p_ho\": {}, \"c_ho\": {}, \"p_hh\": {}, \"c_hh\": {}, \"classes_kept\": {{\"ho\": {}, \"hh\": {}}}, \"weighted_residual\": {}, \"grid\": \"0.50..=4.00 per bohr, step 0.02, per class\", \"grid_points_per_class\": {NC}, \"grid_pairs\": {}, \"placement\": \"cross-unit H–O and cross-unit H–H\", \"weights\": \"1/max(|ΔE_exact|, {WEIGHT_FLOOR:e})²\", \"nonnegativity\": \"drop-and-refit, the most negative amplitude first\", \"points\": {n}, \"remainder\": \"ΔE_exact − [E_q(g) − E_q({FAR_BOHR:.0})]_engine − wall9(g) − CT(g)\", \"units_two_on_every_point\": true, \"fit_seconds\": {}}},\n",
        jn(p_ho),
        jn(c_ho),
        jn(p_hh),
        jn(c_hh),
        ckeep[0],
        ckeep[1],
        jn(c_res),
        NC * NC,
        jd(cfit_seconds)
    ));
    j.push_str(&format!(
        "  \"dispersion\": {{\"nodes\": \"the {} outer linear nodes (R_OO ≥ {OUTER_FROM_ANGSTROM} Å), after both contact terms and the CT term\", \"c6\": {}, \"transferred\": {c6_transferred}, \"slope_band\": [{SLOPE_LO}, {SLOPE_HI}], \"slopes\": [{}]}},\n",
        outer.len(),
        jn(c6),
        slope_lines.join(", ")
    ));
    j.push_str(&format!(
        "  \"g_b0\": {{\"pass\": {g_b0}, \"bounded\": {bounded_json}, \"q_h\": {}, \"q_h_source\": \"holon_render::field::water_charge_at_pin\", \"kt\": {}, \"r_min\": [{}, {}, {}], \"r_min_order\": \"O–O, H–O, H–H\", \"r_min_source\": \"{}\", \"rule\": \"SeamModel::bounded walks each cross-unit class potential of the FULL law (the CT term in) from its own r_min inward to 0.5 bohr on a 0.05 grid and names the first value more than kT below the value at r_min, or a value at 0.5 bohr that is not positive\", \"verdict\": \"{}\", \"hole_reading\": {hole_json}, \"hole_is_a_reading_not_a_gate\": true}},\n",
        jn(q_h),
        jn(KT),
        jn(r_min_oo),
        jn(r_min_oh),
        jn(r_min_hh),
        esc(&wall9_path.display().to_string()),
        if g_b0 { "bounded — the arms may run" } else { "a fall named — the arms are VOID (§2 G-B0)" }
    ));
    j.push_str(&format!(
        "  \"g_c1\": {{\"pass\": {g_c1}, \"worst_miss\": {}, \"tolerance\": {G_C1_TOL:e}, \"reference\": \"E_q(g) − E_q({FAR_BOHR:.0} bohr), the engine's own field on both sides\", \"points\": {n}, \"units_and_pair_counts_ok\": {units_ok}}},\n",
        jn(g_c1_worst)
    ));
    j.push_str(&format!(
        "  \"plant_i\": {{\"fires\": {plant_i_fires}, \"plant\": \"(i) the mask dropped — the same Davidson on the FULL space from the undeformed product start\", \"node\": \"{}\", \"e_full\": {}, \"e_exact_record\": {}, \"miss\": {}, \"tolerance\": {PLANT_I_TOL:e}, \"converged\": {}, \"davidson_iters\": {}, \"residual\": {}, \"carrier_e_ct\": {}, \"carrier_floor\": {PLANT_I_CARRIER:e}, \"carrier_present\": {}, \"source\": \"{}\"}},\n",
        exact[refi].name,
        jn(e_full),
        jn(exact[refi].e_super),
        jn(plant_i_miss),
        json_bool(&plant_i_text, "converged"),
        json_num(&plant_i_text, "davidson_iters"),
        jn(json_num(&plant_i_text, "residual")),
        jn(plant_i_carrier),
        plant_i_carrier >= PLANT_I_CARRIER,
        esc(&plant_i_path.display().to_string())
    ));
    j.push_str(&format!(
        "  \"plant_ii\": {{\"fires\": {}, \"plant\": \"(ii) P_CT → −P_CT in the engine (SeamPlant::FlipChargeTransfer)\", \"node\": \"{}\", \"miss_observed\": {}, \"miss_expected\": {}, \"difference\": {}, \"tolerance\": {G_C1_TOL:e}, \"carrier_ct_term\": {}, \"carrier_floor\": {PLANT_II_CARRIER:e}, \"carrier_present\": {}}},\n",
        plant_ii.3,
        exact[refi].name,
        jn(plant_ii.0),
        jn(plant_ii.1),
        jn((plant_ii.0 - plant_ii.1).abs()),
        jn(plant_ii.2),
        plant_ii.2 >= PLANT_II_CARRIER
    ));
    j.push_str(&format!("  \"arms_void\": {arms_void},\n"));
    j.push_str(&format!("  \"nodes\": [\n    {}\n  ]\n}}\n", node_lines.join(",\n    ")));
    fs::write(out.join("wall_ct.json"), j).unwrap();
    eprintln!("\nwall_ct.json written — arms_void {arms_void}");

    // ------------------------------------ prediction.json, BEFORE the held-out solve
    let (a_p, b_p) = twist_bent(o, h);
    let r_oo_p = cross_oo(&a_p, &b_p);
    assert!(
        (r_oo_p - PRED_ANGSTROM * ANGSTROM_TO_BOHR).abs() < 1e-9,
        "both turns pivot on oxygens: R_OO must be unchanged ({r_oo_p:.9} vs {:.9})",
        PRED_ANGSTROM * ANGSTROM_TO_BOHR
    );
    let (e_pred, e_q_p, e_seam_p) = engine_interaction(&a_p, &b_p, Some(model), SeamPlant::None);
    let tp = formula_terms(&a_p, &b_p, &model);
    let s_p = engine_dimer(&a_p, &b_p, Some(model), SeamPlant::None);
    let (oo_p, ho_p, hh_p) = cross_classes(&a_p, &b_p);
    fs::write(
        out.join("prediction.json"),
        format!(
            "{{\n  \"node\": \"{PRED_NAME}\", \"r_oo_angstrom\": {PRED_ANGSTROM:.3}, \"r_oo_bohr\": {}, \"acceptor_twist_degrees\": {PRED_TWIST_DEGREES:.1}, \"acceptor_tilt_degrees\": {PRED_TILT_DEGREES:.1}, \"donor_bend_degrees\": {PRED_DONOR_BEND_DEGREES:.1}, \"geometry\": \"the linear dimer at {PRED_ANGSTROM:.1} Å with the ACCEPTOR twisted {PRED_TWIST_DEGREES:.0}° about the O···O axis (FIELD-7's builder, tilt {PRED_TILT_DEGREES:.0}°) and the DONOR bent {PRED_DONOR_BEND_DEGREES:.0}° about the x-axis through its own oxygen; both pivots are oxygens, so R_OO is unchanged\", \"held_out\": true, \"kind\": \"a TWISTED-AND-BENT dimer — no fit point turns both monomers this way (§4, M-UNTESTED-GAP)\",\n  \"e_pred\": {},\n  \"parts\": {{\"e_q_difference\": {}, \"contact_ho\": {}, \"contact_hh\": {}, \"wall_oo\": {}, \"wall_oh\": {}, \"wall_hh\": {}, \"wall_total\": {}, \"disp\": {}, \"ct_term\": {}, \"engine_seam\": {}}},\n  \"coefficients\": {{\"a\": {}, \"b\": {}, \"p\": {}, \"c\": {}, \"c6\": {}, \"a_oh\": {}, \"b_oh\": {}, \"a_hh\": {}, \"b_hh\": {}, \"p_hh\": {}, \"c_hh\": {}, \"p_ct\": {}, \"c_ct\": {}}},\n  \"s1_branch\": \"{s1_branch}\", \"c1_pass\": {c1}, \"t0_pass\": {t0}, \"t1_pass\": {t1}, \"t2_pass\": {t2}, \"g_b0_pass\": {g_b0}, \"g_c1_pass\": {g_c1}, \"ct_transferred\": {ct_transferred}, \"c6_transferred\": {c6_transferred}, \"arms_void\": {arms_void}, \"units\": {}, \"oo_pairs\": {}, \"ho_pairs\": {},\n  \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\", \"tolerance_frac\": {PRED_FRAC}, \"tolerance_abs\": {PRED_ABS:e},\n  \"ct_tolerance_rule\": \"branch (b): |CT_term − E_CT| ≤ max({PRED_CT_FRAC}·|E_CT|, {PRED_CT_ABS:e})\", \"ct_tolerance_frac\": {PRED_CT_FRAC}, \"ct_tolerance_abs\": {PRED_CT_ABS:e},\n  \"exact_solve_stake\": {{\"n_det\": {N_DET_DIMER}, \"cpu_seconds_lo\": {S2_CPU_LO}, \"cpu_seconds_hi\": {S2_CPU_HI}, \"residual_bar\": {RESIDUAL_BAR:e}, \"exit\": \"Converged\"}},\n  \"sector_solve_stake\": {{\"sector_dim\": {SECTOR_DIM_STAKED}, \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}}},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}],\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
            jd(r_oo_p),
            jn(e_pred),
            jn(e_q_p),
            jn(tp.pen_ho),
            jn(tp.pen_hh),
            jn(tp.w_oo),
            jn(tp.w_oh),
            jn(tp.w_hh),
            jn(tp.wall_total()),
            jn(tp.disp),
            jn(tp.ct),
            jn(e_seam_p),
            jn(a_oo),
            jn(b_oo),
            jn(p_ho),
            jn(c_ho),
            jn(c6),
            jn(a_oh),
            jn(b_oh),
            jn(a_hh),
            jn(b_hh),
            jn(p_hh),
            jn(c_hh),
            jn(p_ct),
            jn(c_ct),
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
        "prediction.json filed BEFORE the held-out solve: {PRED_NAME} at R_OO {r_oo_p:.6} bohr — E_pred {e_pred:.6e} Ha; E_q(g) − E_q({FAR_BOHR:.0}) {e_q_p:.6e}, contact_HO {:.6e}, contact_HH {:.6e}, wall_OO {:.6e}, wall_OH {:.6e}, wall_HH {:.6e}, disp {:.6e}, CT {:.6e}; units {}, O–O {}, H–O {}",
        tp.pen_ho,
        tp.pen_hh,
        tp.w_oo,
        tp.w_oh,
        tp.w_hh,
        tp.disp,
        tp.ct,
        s_p.seam_work.units,
        s_p.seam_work.oo_pairs,
        s_p.seam_work.ho_pairs
    );
    fs::write(out.join("fit.done"), "done\n").unwrap();
    eprintln!("fit.done written");
}

// --------------------------------------------------------------------------- predict (S2)

fn run_predict(out: &Path) {
    let pred_path = out.join("prediction.json");
    let Ok(pred) = fs::read_to_string(&pred_path) else {
        eprintln!("{} missing: the prediction is filed BEFORE the solve (run `fit` first). Nothing written.", pred_path.display());
        std::process::exit(2);
    };
    let e_pred = json_num(&pred, "e_pred");
    let ct_pred = json_num(&pred, "ct_term");
    assert!(e_pred.is_finite() && ct_pred.is_finite(), "prediction.json carries no e_pred / ct_term");
    let wall_ct_path = out.join("wall_ct.json");
    let wall_ct = fs::read_to_string(&wall_ct_path).expect("wall_ct.json: run `fit` first");
    let (p_ct, c_ct) = (json_num(&wall_ct, "p_ct"), json_num(&wall_ct, "c_ct"));
    assert!(p_ct.is_finite() && c_ct.is_finite(), "wall_ct.json carries no CT term");

    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let (a, b) = twist_bent(o, h);
    let (oo, ho, hh) = cross_classes(&a, &b);
    let r_oo_p = cross_oo(&a, &b);
    // the CT term rebuilt from the harvested coefficients on the geometry rebuilt here
    let ct_rebuilt = -p_ct * sum_exp(&ho, c_ct);
    eprintln!(
        "CT-1 predict — the HELD-OUT node {PRED_NAME} (acceptor twisted {PRED_TWIST_DEGREES:.0}° about the O···O axis, donor bent {PRED_DONOR_BEND_DEGREES:.0}° about its own x-axis, R_OO {r_oo_p:.6} bohr) on {} threads; E_pred {e_pred:.12e} Ha, CT term of the filed prediction {ct_pred:.12e} (rebuilt from wall_ct.json here: {ct_rebuilt:.12e}, difference {:.3e})",
        threads(),
        (ct_rebuilt - ct_pred).abs()
    );

    // the exact solve first: the prediction is already on disk
    let ok = solve_node(out, PRED_NAME, PRED_ANGSTROM, &a, &b, true);
    let t = fs::read_to_string(out.join(format!("{PRED_NAME}.json"))).unwrap();
    let de = json_num(&t, "de_exact");
    let e_super = json_num(&t, "e_super");
    let n_det = json_num(&t, "n_det") as usize;
    let cpu = json_num(&t, "cpu_seconds");
    let n_det_ok = n_det == N_DET_DIMER;
    let price_ok = cpu >= S2_CPU_LO && cpu <= S2_CPU_HI;

    // then the sector solve on the same geometry: E_CT there, measured
    sector_node(out, PRED_NAME, PRED_ANGSTROM, &a, &b);
    let sec = load_sector(out, PRED_NAME).expect("the sector reading was just written");
    let e_ct_measured = e_super - sec.e_noct;
    let order_ok = e_super <= sec.e_noct + ORDER_TOL && sec.e_noct <= sec.e_hl_undeformed + ORDER_TOL;

    let tol = (PRED_FRAC * de.abs()).max(PRED_ABS);
    let miss = (e_pred - de).abs();
    let ct_tol = (PRED_CT_FRAC * e_ct_measured.abs()).max(PRED_CT_ABS);
    let ct_miss = (ct_pred - e_ct_measured).abs();
    let s2 = if miss <= tol {
        "a"
    } else if ct_miss <= ct_tol {
        "b"
    } else {
        "c"
    };
    eprintln!(
        "S2: ΔE_exact {de:.6e} Ha, E_pred {e_pred:.6e} — miss {miss:.3e} ({:.1} % of |ΔE_exact|) against {tol:.3e}; measured E_CT = E_exact − E_noCT = {e_ct_measured:.6e} against the predicted CT term {ct_pred:.6e} — miss {ct_miss:.3e} against {ct_tol:.3e}; the sector's order E_exact ≤ E_noCT ≤ E_HL: {order_ok} → branch ({s2})",
        100.0 * miss / de.abs()
    );
    fs::write(
        out.join("prediction_check.json"),
        format!(
            "{{\n  \"node\": \"{PRED_NAME}\", \"r_oo_angstrom\": {PRED_ANGSTROM:.3}, \"r_oo_bohr\": {},\n  \"e_pred\": {}, \"de_exact\": {}, \"miss\": {}, \"miss_fraction\": {}, \"tolerance\": {}, \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\",\n  \"e_ct_measured\": {}, \"e_ct_measured_rule\": \"E_exact(total) − E_noCT(total), both on this geometry\", \"ct_term_predicted\": {}, \"ct_term_rebuilt_here\": {}, \"ct_miss\": {}, \"ct_tolerance\": {}, \"ct_tolerance_rule\": \"max({PRED_CT_FRAC}·|E_CT|, {PRED_CT_ABS:e})\",\n  \"s2_branch\": \"{s2}\", \"s2_branch_meaning\": \"(a) the total lands within tolerance; (b) the total misses but the CT term is within tolerance of the measured E_CT; (c) both miss\",\n  \"exact\": {{\"converged\": {ok}, \"exit\": \"{}\", \"davidson_iters\": {}, \"residual\": {}, \"residual_bar\": {RESIDUAL_BAR:e}, \"n_det\": {n_det}, \"n_det_expected\": {N_DET_DIMER}, \"n_det_ok\": {n_det_ok}, \"e_super\": {}, \"cpu_seconds\": {}, \"cpu_seconds_lo\": {S2_CPU_LO}, \"cpu_seconds_hi\": {S2_CPU_HI}, \"price_in_band\": {price_ok}, \"wall_seconds\": {}}},\n  \"sector\": {{\"e_noct\": {}, \"e_hl_undeformed\": {}, \"e_a0\": {}, \"e_b0\": {}, \"delta_e_hl\": {}, \"closed_sector_gain\": {}, \"sector_dim\": {}, \"sector_dim_staked\": {SECTOR_DIM_STAKED}, \"davidson_iters\": {}, \"residual\": {}, \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}, \"converged\": {}, \"order_ok\": {order_ok}, \"metric_min_eigenvalue\": {}, \"cpu_seconds\": {}, \"price_admitted\": {}, \"source\": \"{}\"}},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}]\n}}\n",
            jd(r_oo_p),
            jn(e_pred),
            jn(de),
            jn(miss),
            jn(miss / de.abs()),
            jn(tol),
            jn(e_ct_measured),
            jn(ct_pred),
            jn(ct_rebuilt),
            jn(ct_miss),
            jn(ct_tol),
            json_str(&t, "exit"),
            json_num(&t, "davidson_iters") as u64,
            jn(json_num(&t, "residual")),
            jn(e_super),
            jn(cpu),
            jn(json_num(&t, "wall_seconds")),
            jn(sec.e_noct),
            jn(sec.e_hl_undeformed),
            jn(sec.e_a0),
            jn(sec.e_b0),
            jn(sec.e_hl_undeformed - (sec.e_a0 + sec.e_b0)),
            jn(sec.e_noct - sec.e_hl_undeformed),
            sec.sector_dim,
            sec.davidson_iters,
            jn(sec.residual),
            sec.converged,
            jn(sec.metric_min_eigenvalue),
            jn(sec.cpu_seconds),
            sec.price_admitted,
            esc(&sec.source),
            list_json(&oo),
            list_json(&ho),
            list_json(&hh),
        ),
    )
    .unwrap();
    fs::write(out.join("predict.done"), "done\n").unwrap();
    eprintln!("prediction_check.json and predict.done written");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let what = args.get(1).map(String::as_str).unwrap_or("fit");
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/ct1".to_string()));
    fs::create_dir_all(&out).expect("out");
    match what {
        "sector" => run_sector(&out),
        "far" => run_far(&out),
        "fit" => run_fit(&out),
        "predict" => run_predict(&out),
        other => eprintln!("unknown phase {other} (sector | far | fit | predict)"),
    }
}
