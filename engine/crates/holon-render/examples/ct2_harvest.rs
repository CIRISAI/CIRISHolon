//! CT-2's harvest (`conformance/water_observatory/CT2_PREREG.md`): THE ANGLE IN THE TRANSFER,
//! and the contacts fit UNDER the boundedness gate.
//!
//! CT-1 measured charge transfer by itself (`E_CT = E_exact − E_noCT`) on twelve geometries and
//! read two things about carrying it: one exponential on cross-unit H–O distances over-counts the
//! twisted dimer by 60 %, because charge transfer has an angle and the term has none; and the
//! contact terms fit on what remains were holes below their data. This freeze answers both. The
//! transfer term gets the bond's alignment on BOTH sides as a DECLARED family
//!
//! ```text
//! r    = |H − O_a|,  θ_d = the angle O_d–H···O_a,  u = (H − O_a)/r
//! b    = unit((h₁ − O_a) + (h₂ − O_a)),  n = unit((h₁ − O_a) × (h₂ − O_a))
//! l±   = −cos λ · b ± sin λ · n
//! f_d  = ((1 − cos θ_d)/2)^m
//! g_a  = [((1 + u·l₊)/2)^k + ((1 + u·l₋)/2)^k] / [2·((1 + cos λ)/2)^k]
//! CT(g) = −P · Σ_{cross-unit H–O} exp(−c·r) · f_d · g_a
//! ```
//!
//! harvested on an orientation map of exact and closed-sector solves; and the contact terms are
//! fit INSIDE the boundedness gate, each amplitude clamped at the largest value `bounded` admits,
//! so a law that fails G-B0 cannot be produced.
//!
//! ```text
//! cargo run --release -p holon-render --example ct2_harvest -- map     [OUT_DIR]
//! cargo run --release -p holon-render --example ct2_harvest -- fit     [OUT_DIR]
//! cargo run --release -p holon-render --example ct2_harvest -- predict [OUT_DIR]
//! ```
//!
//! `map` (detached, resumable): G-D0 — `fci_full_from_product` against the thirteen exact nodes of
//! record — then the new geometries of the freeze's table, each first read for the closure identity
//! (fewer than two units ⇒ named, NOT solved) and then solved exactly and in the block-localised
//! sector, each priced and recorded with its exit and iteration count. `fit`: G-D0's verdict, T0,
//! A1, P1, the angular family fit (S1), the two-class contact fit UNDER the gate (C1) with the
//! unconstrained fit beside it, dispersion, G-B0, G-C1, both plants, `wall_ct2.json`, and
//! `prediction.json` for the held-out node BEFORE that node is solved. `predict`: S2.
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::{solve_embedded, water_dimer_linear, Fragment, ANGSTROM_TO_BOHR};
use holon_chem::heitler_london::{fci_block_localised, fci_full_from_product, BlwReading};
use holon_render::seam::{SeamModel, SeamPlant};
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/quartet.rs"]
#[allow(dead_code)]
mod quartet;

/// EMBED-1's water pins — the same numbers FIELD-3's … CT-1's runners carry.
const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;

/// The SEVEN linear exact nodes of record, shortest first (FIELD-9's list).
const LINEAR_ANGSTROM: [f64; 7] = [2.3, 2.5, 2.7, 2.9, 3.1, 3.4, 3.7];
/// The linear node of record that lives in `field8/` rather than `field3/`.
const LINEAR_FROM_FIELD8: f64 = 2.3;
/// A linear node at or beyond this separation is one of FIELD-6's four OUTER dispersion nodes.
const OUTER_FROM_ANGSTROM: f64 = 2.9;
/// C1's second leg: the line where the bond lives must be within tolerance at all three.
const LINE_ANGSTROM: [f64; 3] = [2.7, 2.9, 3.1];
/// The node both plants and the decomposition are read at.
const REF_ANGSTROM: f64 = 2.9;

/// The five non-linear exact geometries of CT-1's twelve (FIELD-9 §0).
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

/// CT-1's S2 node, the thirteenth exact node of record: the acceptor twisted `90°` about the
/// O···O axis at `R_OO = 3.2` Å with the donor bent `20°` about its own x-axis.
const CT1S2_ANGSTROM: f64 = 3.2;
const CT1S2_TWIST_DEGREES: f64 = 90.0;
const CT1S2_TILT_DEGREES: f64 = 0.0;
const CT1S2_DONOR_DEGREES: f64 = 20.0;
const CT1S2_NAME: &str = "twistbent_R3.2";

/// The freeze's tilt family: `tilted(R, θ)`, the `2.9 Å / 30°` node of record excluded.
const TILT_R: [f64; 4] = [2.7, 2.9, 3.1, 3.4];
const TILT_T: [f64; 6] = [30.0, 60.0, 90.0, 120.0, 150.0, 180.0];
/// The freeze's twist family: `twisted(R, 90°, θ)`, the `3.0 Å / 60°` node of record excluded.
const TWIST_R: [f64; 3] = [2.7, 3.0, 3.4];
const TWIST_T: [f64; 6] = [0.0, 30.0, 60.0, 90.0, 120.0, 180.0];
const TWIST_PHI: f64 = 90.0;
/// The freeze's donor family: `bent_donor(R, β)`.
const DONOR_R: [f64; 2] = [2.9, 3.1];
const DONOR_B: [f64; 4] = [15.0, 45.0, 60.0, 90.0];
/// The freeze's doubly varied geometries: `double_bent(R, β_donor, θ_acceptor)`.
const DBENT: [(f64, f64, f64); 3] = [(2.9, 15.0, 60.0), (2.9, 45.0, 45.0), (2.9, 30.0, 90.0)];
/// …and `twisted(3.0, 90°, 30°)` with the donor bent `20°`.
const DTWIST: (f64, f64, f64, f64) = (3.0, 90.0, 30.0, 20.0);

/// S2's held-out geometry (§2 S2): `twisted(3.1 Å, 45°, 45°)` with the DONOR bent `15°`.
const PRED_ANGSTROM: f64 = 3.1;
const PRED_TWIST_DEGREES: f64 = 45.0;
const PRED_TILT_DEGREES: f64 = 45.0;
const PRED_DONOR_DEGREES: f64 = 15.0;
const PRED_NAME: &str = "twisttiltbent_R3.1";

/// The separation at which the acceptor is "away" (bohr): the engine's reference on both sides of
/// G-C1 and of every `E_q` difference.
const FAR_BOHR: f64 = 40.0;

/// G-D0's bar: `|E_full − E_record| ≤ 1e-8`, on THIRTEEN nodes.
const GD0_TOL: f64 = 1e-8;
const GD0_NODES: usize = 13;
/// T0's residual bar on the SECTOR solve, and on `fci_full_from_product` (its own Davidson bar).
const SECTOR_RESIDUAL_BAR: f64 = 1e-8;
/// The Davidson cap inside `fci_full_from_product` / `fci_block_localised` — a solve at the cap
/// is VOID (M-EXIT-DISCRIMINATOR).
const DAVIDSON_CAP: usize = 300;
/// T0's subspace dimension on the water dimer, EXACT: `441 × 441`.
const SECTOR_DIM_STAKED: usize = 194_481;
/// T0's order slack, on TOTAL energies: `E_exact ≤ E_noCT ≤ E_HL(undeformed)`.
const ORDER_TOL: f64 = 1e-10;

/// A1: charge transfer is attractive at every node whose shortest cross-unit H···O is at or under
/// this separation; beyond it a null is a null, not a miss (M-NULL-MISSTAKE).
const A1_CONTACT_BOHR: f64 = 5.0;
/// M-FLOOR-UNSTAKED: the charge-transfer floor.
const CT_FLOOR: f64 = 1e-6;

/// The `c`-grids, per class (per bohr): `0.50 ..= 4.00` step `0.02` — 176 values.
const NC: usize = 176;
/// The angular family's exponent grids and lone-pair angles (§0).
const M_GRID: [i32; 4] = [0, 1, 2, 4];
const K_GRID: [i32; 4] = [0, 1, 2, 4];
const LAM_DEG: [f64; 5] = [0.0, 30.0, 45.0, 55.0, 70.0];
/// An exponent at this value sits at the EDGE of its grid (M-FIRST-VIOLATION-ONLY).
const MK_EDGE: i32 = 4;
/// CT-1's weighting rule, carried: `1/max(|ΔE_exact|, 5e-3)²`.
const WEIGHT_FLOOR: f64 = 5e-3;

/// S1's tolerance, per node: `max(0.25·|ΔE_exact|, 5e-4)`.
const S1_FRAC: f64 = 0.25;
const S1_ABS: f64 = 5e-4;
/// S1's branch-(b) BAR is the freeze's literal — "at least 80 % (52 of 64)", which is 80 % of the
/// freeze's distinct total. The bar is that literal; 80 % of the MEASURED total is computed and
/// recorded beside it, so a node named outside the closure identity is visible in the record
/// rather than quietly lowering the bar. C1, whose letter names no count, takes 80 % of the
/// measured total.
const EIGHTY: f64 = 0.8;
const S1_B_LITERAL: usize = 52;
/// The freeze's stated counts (§0's table), quoted beside the measured ones.
const FREEZE_TOTAL: usize = 64;
const FREEZE_NEW: usize = 51;

/// The band the remainder's log-log slope must lie in for `C₆` to transfer (FIELD-6's rule).
const SLOPE_LO: f64 = -8.0;
const SLOPE_HI: f64 = -4.0;

/// G-B0's temperature, in hartree.
const KT: f64 = 9.278758e-4;
/// The contact clamp: bisection on `[0, P_ls]` to this RELATIVE width, at most this many rounds
/// of the two classes against each other.
const CLAMP_REL: f64 = 1e-6;
const CLAMP_ROUNDS: usize = 20;
/// C1's reading: a constrained/unconstrained residual ratio above this says the data want a shape
/// the gate forbids (named, not a failure).
const RATIO_NAMED: f64 = 2.0;

/// G-C1's tolerance.
const G_C1_TOL: f64 = 1e-10;
/// Plant (ii): `P_CT → −P_CT` must move G-C1 by `2·|CT(2.9 Å)|`; its carrier floor.
const PLANT_II_CARRIER: f64 = 1e-4;
/// Plant (i): the family at `m = k = 0` must MISS the twisted node by at least this, with this
/// carrier floor on the same difference.
const PLANT_I_MIN_MISS: f64 = 2e-3;
const PLANT_I_CARRIER: f64 = 1e-3;

/// P1: the fit restricted to `m = k = 0` on CT-1's twelve reproduces CT-1's numbers to `1e-9`.
const CT1_P: f64 = 1.474879879;
const CT1_C: f64 = 1.46;
const P1_TOL: f64 = 1e-9;

/// S2's tolerance on the total, and on the transfer term by itself (branch (b)).
const PRED_FRAC: f64 = 0.25;
const PRED_ABS: f64 = 5e-4;
const PRED_CT_FRAC: f64 = 0.25;
const PRED_CT_ABS: f64 = 2e-4;

/// The determinant count FIELD-3's supermolecule carries (EXACT).
const N_DET_DIMER: usize = 1_002_001;
/// M-CHEAPER-THAN-ITS-PRICE: FIELD-5's measured floor, core-seconds per Hamiltonian application,
/// and the tenth of its own iteration count a solve is refused under.
const SIGMA_PRICE_FLOOR: f64 = 55.0;
const PRICE_TENTH: f64 = 0.1;

// --------------------------------------------------------------------------------- plumbing

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
/// The slice of a record that BEGINS at one named object (`"<key>": {`). `json_num` and friends
/// take the FIRST occurrence of a key, and a node record carries `davidson_iters`, `residual`,
/// `converged`, `cpu_seconds` and `price_admitted` in BOTH its `exact` and its `sector` block —
/// without this slice the sector's legs would be read off the exact solve. A record with no such
/// object reads whole (the sector files of record are one flat object).
fn section<'a>(t: &'a str, key: &str) -> &'a str {
    match t.split_once(&format!("\"{key}\": {{")) {
        Some((_, rest)) => rest,
        None => t,
    }
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
/// leading `+`, and `null` rather than `NaN`/`inf`.
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

fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn list_json(v: &[f64]) -> String {
    v.iter().map(|d| jd(*d)).collect::<Vec<_>>().join(", ")
}

fn sum_exp(rs: &[f64], b: f64) -> f64 {
    rs.iter().map(|&r| (-b * r).exp()).sum()
}

fn min_or_nan(v: &[f64]) -> f64 {
    v.iter().cloned().fold(f64::INFINITY, f64::min)
}

/// The contact grid, per bohr: `0.50 ..= 4.00` step `0.02`, built so the ends are exact.
fn cgrid(i: usize) -> f64 {
    ((25 + i) as f64) * 0.02
}

fn sibling(out: &Path, name: &str) -> PathBuf {
    let sib = out.parent().unwrap_or(Path::new(".")).join(name);
    if sib.exists() {
        sib
    } else {
        PathBuf::from(format!("../conformance/water_observatory/{name}"))
    }
}

// ------------------------------------------------------------------------ the geometries

/// FIELD-3's `linear` verbatim.
fn linear(o: Species, h: Species, r_oo_angstrom: f64) -> (Fragment, Fragment) {
    water_dimer_linear(o, h, H2O_R, H2O_THETA, r_oo_angstrom * ANGSTROM_TO_BOHR)
}

/// FIELD-5's rotation, in one place: a fragment turned by `theta_degrees` about the x-axis through
/// ITS OWN oxygen — FIELD-5's `tilted` and FIELD-8's `bent_donor` arithmetic verbatim.
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
/// through its oxygen.
fn flipped(o: Species, h: Species, r_oo_angstrom: f64) -> (Fragment, Fragment) {
    let (donor, _) = linear(o, h, r_oo_angstrom);
    let (s, c) = ((0.5 * H2O_THETA).sin(), (0.5 * H2O_THETA).cos());
    let r = H2O_R;
    let acc = Fragment::new(vec![o, h, h], vec![[0.0; 3], [r * s, 0.0, -r * c], [-r * s, 0.0, -r * c]], vec![-2.0, 1.0, 1.0])
        .translated([0.0, 0.0, r_oo_angstrom * ANGSTROM_TO_BOHR]);
    (donor, acc)
}

/// FIELD-5's `tilted`: the ACCEPTOR rotated by `theta_degrees` about the x-axis through its OWN
/// oxygen. The donor is untouched and `R_OO` is unchanged.
fn tilted(o: Species, h: Species, r_oo_angstrom: f64, theta_degrees: f64) -> (Fragment, Fragment) {
    let (donor, acc) = linear(o, h, r_oo_angstrom);
    let a = rot_x(&acc, theta_degrees);
    (donor, a)
}

/// FIELD-7's `twisted` verbatim: the acceptor rotated by `twist_degrees` about the O···O axis (z)
/// through its own oxygen, and THEN tilted by `tilt_degrees` about its own x-axis.
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
            let (x1, y1, z1) = (x * cw - y * sw, x * sw + y * cw, z);
            let (x2, y2, z2) = (x1, y1 * ct - z1 * st, y1 * st + z1 * ct);
            [origin[0] + x2, origin[1] + y2, origin[2] + z2]
        })
        .collect();
    (donor, Fragment::new(acc.species.clone(), centers, acc.weights.clone()))
}

/// FIELD-8's bent donor: the DONOR rotated about the x-axis through its own oxygen.
fn bent_donor(o: Species, h: Species, r_oo_angstrom: f64, theta_degrees: f64) -> (Fragment, Fragment) {
    let (donor, acc) = linear(o, h, r_oo_angstrom);
    (rot_x(&donor, theta_degrees), acc)
}

/// FIELD-9's `double_bent`: the donor bent `donor_degrees`, the acceptor tilted `acceptor_degrees`.
fn double_bent(o: Species, h: Species, r_oo_angstrom: f64, donor_degrees: f64, acceptor_degrees: f64) -> (Fragment, Fragment) {
    let (donor, acc) = linear(o, h, r_oo_angstrom);
    (rot_x(&donor, donor_degrees), rot_x(&acc, acceptor_degrees))
}

/// The acceptor twisted and tilted by FIELD-7's builder AND the donor bent about its own x-axis —
/// CT-1's S2 node's shape, and this freeze's doubly varied fourth and held-out node.
fn twist_and_bend(o: Species, h: Species, r_oo_angstrom: f64, twist_degrees: f64, tilt_degrees: f64, donor_degrees: f64) -> (Fragment, Fragment) {
    let (donor, acc) = twisted(o, h, r_oo_angstrom, twist_degrees, tilt_degrees);
    (rot_x(&donor, donor_degrees), acc)
}

fn centers_json(f: &Fragment) -> String {
    f.centers.iter().map(|c| format!("[{:.10}, {:.10}, {:.10}]", c[0], c[1], c[2])).collect::<Vec<_>>().join(", ")
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

/// Two geometries are the SAME node when every atom of each fragment agrees within this (bohr).
const SAME_GEOMETRY_TOL: f64 = 1e-9;

fn geometry_deviation(a1: &Fragment, b1: &Fragment, a2: &Fragment, b2: &Fragment) -> f64 {
    if a1.centers.len() != a2.centers.len() || b1.centers.len() != b2.centers.len() {
        return f64::INFINITY;
    }
    let da = a1.centers.iter().zip(a2.centers.iter()).map(|(p, q)| dist(p, q)).fold(0.0f64, f64::max);
    let db = b1.centers.iter().zip(b2.centers.iter()).map(|(p, q)| dist(p, q)).fold(0.0f64, f64::max);
    da.max(db)
}

// ------------------------------------------- the transfer family, written from the freeze's text
//
// THIS IS THE FORMULA SIDE OF G-C1 and the fit's own arithmetic. It is written from CT2_PREREG §0
// and NEVER calls `SeamModel::ct_angular` — G-C1 compares the engine's implementation against this
// independent one, so sharing the code would make the gate vacuous (M-VACUOUS-SUCCESS).

/// One cross-unit H–O pair, reduced to the four numbers the family needs: the separation, the
/// donor angle's cosine, and the two components of `u` in the acceptor's own frame.
struct CtPair {
    r: f64,
    cos_td: f64,
    /// `u·b̂`, the acceptor's bisector direction (toward its hydrogens).
    ub: f64,
    /// `u·n̂`, the acceptor's plane normal.
    un: f64,
}

fn v_sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn v_add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn v_dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn v_cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn v_norm(a: [f64; 3]) -> f64 {
    v_dot(a, a).sqrt()
}
fn v_unit(a: [f64; 3]) -> [f64; 3] {
    let n = v_norm(a).max(1e-9);
    [a[0] / n, a[1] / n, a[2] / n]
}

fn oxygen_of(f: &Fragment) -> usize {
    f.species.iter().position(|s| s.z == 8).expect("an oxygen")
}
fn hydrogens_of(f: &Fragment) -> [usize; 2] {
    let hs: Vec<usize> = (0..f.species.len()).filter(|&i| f.species[i].z == 1).collect();
    assert_eq!(hs.len(), 2, "a water unit is an oxygen with exactly two hydrogens");
    [hs[0], hs[1]]
}

/// Every cross-unit H–O pair of a two-unit geometry, with the DONOR's hydrogens against the
/// ACCEPTOR's oxygen and then the other way round: four pairs on a water dimer, each carrying its
/// own acceptor frame. `g_a` is symmetric in the acceptor's two hydrogens (the sum over `l₊`, `l₋`
/// swaps when `n̂` flips), so their order is immaterial.
fn ct_pairs(a: &Fragment, b: &Fragment) -> Vec<CtPair> {
    let mut out = Vec::new();
    for (donor, acceptor) in [(a, b), (b, a)] {
        let od = donor.centers[oxygen_of(donor)];
        let oa = acceptor.centers[oxygen_of(acceptor)];
        let [i1, i2] = hydrogens_of(acceptor);
        let v1 = v_sub(acceptor.centers[i1], oa);
        let v2 = v_sub(acceptor.centers[i2], oa);
        let bh = v_unit(v_add(v1, v2));
        let nh = v_unit(v_cross(v1, v2));
        for &hi in hydrogens_of(donor).iter() {
            let xh = donor.centers[hi];
            let d = v_sub(xh, oa);
            let r = v_norm(d).max(1e-9);
            let u = [d[0] / r, d[1] / r, d[2] / r];
            let av = v_sub(od, xh);
            let bv = v_sub(oa, xh);
            let (na, nb) = (v_norm(av).max(1e-9), v_norm(bv).max(1e-9));
            let cos_td = (v_dot(av, bv) / (na * nb)).clamp(-1.0, 1.0);
            out.push(CtPair { r, cos_td, ub: v_dot(u, bh), un: v_dot(u, nh) });
        }
    }
    out
}

/// The angular factor `f_d · g_a` of one pair at one `(m, k, λ)` — the freeze's equations.
fn ct_angular_factor(q: &CtPair, m: i32, k: i32, cl: f64, sl: f64) -> f64 {
    let f = if m == 0 { 1.0 } else { (0.5 * (1.0 - q.cos_td)).powi(m) };
    let g = if k == 0 {
        1.0
    } else {
        let sp = -cl * q.ub + sl * q.un;
        let sm = -cl * q.ub - sl * q.un;
        let norm_g = 2.0 * (0.5 * (1.0 + cl)).powi(k);
        ((0.5 * (1.0 + sp)).powi(k) + (0.5 * (1.0 + sm)).powi(k)) / norm_g
    };
    f * g
}

/// `CT(g) = −P · Σ exp(−c·r)·f_d·g_a` over a geometry's cross-unit H–O pairs.
fn ct_value(pairs: &[CtPair], p: f64, c: f64, m: i32, k: i32, lambda: f64) -> f64 {
    let (cl, sl) = (lambda.cos(), lambda.sin());
    -p * pairs.iter().map(|q| (-c * q.r).exp() * ct_angular_factor(q, m, k, cl, sl)).sum::<f64>()
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
/// the total, the FIELD part, the SEAM part.
fn engine_interaction(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, plant: SeamPlant) -> (f64, f64, f64) {
    let s = engine_dimer(a, b, seam, plant);
    let near = (s.e_pair + s.e_three) + s.e_field + s.e_seam;
    let far_b = b.translated([FAR_BOHR, 0.0, 0.0]);
    let f = engine_dimer(a, &far_b, seam, plant);
    let far = (f.e_pair + f.e_three) + f.e_field + f.e_seam;
    (near - far, s.e_field - f.e_field, s.e_seam - f.e_seam)
}

/// The formula side of G-C1, class by class — the transfer row is CT-2's angular family.
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
    let pairs = ct_pairs(a, b);
    Terms {
        pen_ho: ho.iter().map(|&r| m.penetration(r)).sum(),
        pen_hh: hh.iter().map(|&r| m.contact_hh(r)).sum(),
        w_oo: oo.iter().map(|&r| m.wall(r)).sum(),
        w_oh: ho.iter().map(|&r| m.wall_oh(r)).sum(),
        w_hh: hh.iter().map(|&r| m.wall_hh(r)).sum(),
        disp: oo.iter().map(|&r| m.dispersion(r)).sum(),
        ct: ct_value(&pairs, m.p_ct, m.c_ct, m.m_ct as i32, m.k_ct as i32, m.lambda_ct),
    }
}

// -------------------------------------------------------------------------- the node catalogue

/// One geometry of the campaign, built here and named here.
struct Spec {
    name: String,
    family: &'static str,
    kind: String,
    r_ang: f64,
    twist_deg: f64,
    tilt_deg: f64,
    donor_deg: f64,
    a: Fragment,
    b: Fragment,
    of_record: bool,
    /// The record the exact energy is read from (the thirteen), or `None` for a new node.
    exact_record: Option<PathBuf>,
    /// The block-localised record (the thirteen live in `ct1/`).
    sector_record: Option<PathBuf>,
}

impl Spec {
    fn linear_node(&self) -> bool {
        self.family == "linear"
    }
    fn line_node(&self) -> bool {
        self.linear_node() && LINE_ANGSTROM.iter().any(|&l| (l - self.r_ang).abs() < 1e-9)
    }
    fn outer_linear(&self) -> bool {
        self.linear_node() && self.r_ang >= OUTER_FROM_ANGSTROM - 1e-9
    }
}

/// THE NAMING, stated once and carried into every record this runner writes.
const NAMING: &str = "tilt_R<R>_t<tilt> = tilted(R, tilt); twist_R<R>_t<tilt> = twisted(R, 90, tilt); donor_R<R>_b<bend> = bent_donor(R, bend); dbent_R<R>_d<donor bend>_a<acceptor tilt> = double_bent(R, donor bend, acceptor tilt); twistbent_R<R>_tw<twist>_t<tilt>_d<donor bend> = twisted(R, twist, tilt) with the donor rot_x(donor bend). R in angstrom to one decimal, every angle in whole degrees. The thirteen of record keep the names their own records carry.";

/// The THIRTEEN exact geometries of record: CT-1's twelve (FIELD-9 §0) and CT-1's S2 node.
fn record_specs(out: &Path, o: Species, h: Species) -> Vec<Spec> {
    let f3 = sibling(out, "field3");
    let f4 = sibling(out, "field4");
    let f5 = sibling(out, "field5");
    let f6 = sibling(out, "field6");
    let f7 = sibling(out, "field7");
    let f8 = sibling(out, "field8");
    let ct1 = sibling(out, "ct1");
    let sec = |n: &str| Some(ct1.join(format!("sector_{n}.json")));
    let mut v: Vec<Spec> = Vec::new();
    for &r in LINEAR_ANGSTROM.iter() {
        let (a, b) = linear(o, h, r);
        let dir = if (r - LINEAR_FROM_FIELD8).abs() < 1e-9 { f8.clone() } else { f3.clone() };
        let name = format!("linear_R{r:.1}");
        v.push(Spec {
            kind: format!("the linear dimer at {r:.1} Å"),
            family: "linear",
            r_ang: r,
            twist_deg: 0.0,
            tilt_deg: 0.0,
            donor_deg: 0.0,
            a,
            b,
            of_record: true,
            exact_record: Some(dir.join(format!("{name}.json"))),
            sector_record: sec(&name),
            name,
        });
    }
    let (a5, b5) = tilted(o, h, TILT5_ANGSTROM, TILT5_DEGREES);
    v.push(Spec {
        name: format!("tilted_R{TILT5_ANGSTROM:.1}"),
        family: "tilt",
        kind: format!("bent acceptor {TILT5_DEGREES:.0}°"),
        r_ang: TILT5_ANGSTROM,
        twist_deg: 0.0,
        tilt_deg: TILT5_DEGREES,
        donor_deg: 0.0,
        a: a5,
        b: b5,
        of_record: true,
        exact_record: Some(f5.join(format!("tilted_R{TILT5_ANGSTROM:.1}.json"))),
        sector_record: sec(&format!("tilted_R{TILT5_ANGSTROM:.1}")),
    });
    let (a6, b6) = tilted(o, h, TILT6_ANGSTROM, TILT6_DEGREES);
    v.push(Spec {
        name: format!("tilted{TILT6_DEGREES:.0}_R{TILT6_ANGSTROM:.1}"),
        family: "tilt",
        kind: format!("bent acceptor {TILT6_DEGREES:.0}°"),
        r_ang: TILT6_ANGSTROM,
        twist_deg: 0.0,
        tilt_deg: TILT6_DEGREES,
        donor_deg: 0.0,
        a: a6,
        b: b6,
        of_record: true,
        exact_record: Some(f6.join(format!("tilted{TILT6_DEGREES:.0}_R{TILT6_ANGSTROM:.1}.json"))),
        sector_record: sec(&format!("tilted{TILT6_DEGREES:.0}_R{TILT6_ANGSTROM:.1}")),
    });
    let (a4, b4) = flipped(o, h, FLIPPED_ANGSTROM);
    v.push(Spec {
        name: format!("flipped_R{FLIPPED_ANGSTROM:.1}"),
        family: "tilt",
        kind: "flipped acceptor 180°".to_string(),
        r_ang: FLIPPED_ANGSTROM,
        twist_deg: 0.0,
        tilt_deg: 180.0,
        donor_deg: 0.0,
        a: a4,
        b: b4,
        of_record: true,
        exact_record: Some(f4.join(format!("flipped_R{FLIPPED_ANGSTROM:.1}.json"))),
        sector_record: sec(&format!("flipped_R{FLIPPED_ANGSTROM:.1}")),
    });
    let (a7, b7) = twisted(o, h, TWISTED_ANGSTROM, TWISTED_TWIST_DEGREES, TWISTED_TILT_DEGREES);
    v.push(Spec {
        name: format!("twisted_R{TWISTED_ANGSTROM:.1}"),
        family: "twist",
        kind: format!("twisted {TWISTED_TWIST_DEGREES:.0}° + tilted {TWISTED_TILT_DEGREES:.0}°"),
        r_ang: TWISTED_ANGSTROM,
        twist_deg: TWISTED_TWIST_DEGREES,
        tilt_deg: TWISTED_TILT_DEGREES,
        donor_deg: 0.0,
        a: a7,
        b: b7,
        of_record: true,
        exact_record: Some(f7.join(format!("twisted_R{TWISTED_ANGSTROM:.1}.json"))),
        sector_record: sec(&format!("twisted_R{TWISTED_ANGSTROM:.1}")),
    });
    let (a8, b8) = bent_donor(o, h, BENTDONOR_ANGSTROM, BENTDONOR_DEGREES);
    v.push(Spec {
        name: format!("bentdonor_R{BENTDONOR_ANGSTROM:.1}"),
        family: "donor",
        kind: format!("bent donor {BENTDONOR_DEGREES:.0}°"),
        r_ang: BENTDONOR_ANGSTROM,
        twist_deg: 0.0,
        tilt_deg: 0.0,
        donor_deg: BENTDONOR_DEGREES,
        a: a8,
        b: b8,
        of_record: true,
        exact_record: Some(f8.join(format!("bentdonor_R{BENTDONOR_ANGSTROM:.1}.json"))),
        sector_record: sec(&format!("bentdonor_R{BENTDONOR_ANGSTROM:.1}")),
    });
    let (a13, b13) = twist_and_bend(o, h, CT1S2_ANGSTROM, CT1S2_TWIST_DEGREES, CT1S2_TILT_DEGREES, CT1S2_DONOR_DEGREES);
    v.push(Spec {
        name: CT1S2_NAME.to_string(),
        family: "twistbent",
        kind: format!("twisted {CT1S2_TWIST_DEGREES:.0}° + bent donor {CT1S2_DONOR_DEGREES:.0}° (CT-1's S2 node)"),
        r_ang: CT1S2_ANGSTROM,
        twist_deg: CT1S2_TWIST_DEGREES,
        tilt_deg: CT1S2_TILT_DEGREES,
        donor_deg: CT1S2_DONOR_DEGREES,
        a: a13,
        b: b13,
        of_record: true,
        exact_record: Some(ct1.join(format!("{CT1S2_NAME}.json"))),
        sector_record: sec(CT1S2_NAME),
    });
    assert_eq!(v.len(), GD0_NODES, "the exact record is thirteen geometries");
    v
}

/// The freeze's NEW geometries (§0's table), before the duplicate check: the tilt family less its
/// node of record, the twist family less its node of record, the donor family, and the four
/// doubly varied.
fn new_specs(o: Species, h: Species) -> Vec<Spec> {
    let mut v: Vec<Spec> = Vec::new();
    for &r in TILT_R.iter() {
        for &t in TILT_T.iter() {
            if (r - TILT5_ANGSTROM).abs() < 1e-9 && (t - TILT5_DEGREES).abs() < 1e-9 {
                continue; // the 2.9 Å / 30° node is of record (the freeze's own exclusion)
            }
            let (a, b) = tilted(o, h, r, t);
            v.push(Spec {
                name: format!("tilt_R{r:.1}_t{t:.0}"),
                family: "tilt",
                kind: format!("acceptor tilted {t:.0}° at {r:.1} Å"),
                r_ang: r,
                twist_deg: 0.0,
                tilt_deg: t,
                donor_deg: 0.0,
                a,
                b,
                of_record: false,
                exact_record: None,
                sector_record: None,
            });
        }
    }
    for &r in TWIST_R.iter() {
        for &t in TWIST_T.iter() {
            if (r - TWISTED_ANGSTROM).abs() < 1e-9 && (t - TWISTED_TILT_DEGREES).abs() < 1e-9 {
                continue; // the 3.0 Å / 60° node is of record (the freeze's own exclusion)
            }
            let (a, b) = twisted(o, h, r, TWIST_PHI, t);
            v.push(Spec {
                name: format!("twist_R{r:.1}_t{t:.0}"),
                family: "twist",
                kind: format!("acceptor twisted {TWIST_PHI:.0}° then tilted {t:.0}° at {r:.1} Å"),
                r_ang: r,
                twist_deg: TWIST_PHI,
                tilt_deg: t,
                donor_deg: 0.0,
                a,
                b,
                of_record: false,
                exact_record: None,
                sector_record: None,
            });
        }
    }
    for &r in DONOR_R.iter() {
        for &beta in DONOR_B.iter() {
            let (a, b) = bent_donor(o, h, r, beta);
            v.push(Spec {
                name: format!("donor_R{r:.1}_b{beta:.0}"),
                family: "donor",
                kind: format!("donor bent {beta:.0}° at {r:.1} Å"),
                r_ang: r,
                twist_deg: 0.0,
                tilt_deg: 0.0,
                donor_deg: beta,
                a,
                b,
                of_record: false,
                exact_record: None,
                sector_record: None,
            });
        }
    }
    for &(r, beta, theta) in DBENT.iter() {
        let (a, b) = double_bent(o, h, r, beta, theta);
        v.push(Spec {
            name: format!("dbent_R{r:.1}_d{beta:.0}_a{theta:.0}"),
            family: "dbent",
            kind: format!("donor bent {beta:.0}° and acceptor tilted {theta:.0}° at {r:.1} Å"),
            r_ang: r,
            twist_deg: 0.0,
            tilt_deg: theta,
            donor_deg: beta,
            a,
            b,
            of_record: false,
            exact_record: None,
            sector_record: None,
        });
    }
    let (r, tw, tl, db) = DTWIST;
    let (a, b) = twist_and_bend(o, h, r, tw, tl, db);
    v.push(Spec {
        name: format!("twistbent_R{r:.1}_tw{tw:.0}_t{tl:.0}_d{db:.0}"),
        family: "twistbent",
        kind: format!("acceptor twisted {tw:.0}° then tilted {tl:.0}°, donor bent {db:.0}°, at {r:.1} Å"),
        r_ang: r,
        twist_deg: tw,
        tilt_deg: tl,
        donor_deg: db,
        a,
        b,
        of_record: false,
        exact_record: None,
        sector_record: None,
    });
    v
}

// ---------------------------------------------------------------------- the frozen records

/// One exact record of the thirteen, READ and CHECKED against the geometry this runner builds
/// (M-STALE-INSTRUMENT).
struct ExactRecord {
    e_super: f64,
    de_exact: f64,
    e_a0: f64,
    e_b0: f64,
    davidson_iters: usize,
    residual: f64,
    converged: bool,
    exit: String,
    cpu_seconds: f64,
    wall_seconds: f64,
    source: String,
}

fn load_exact_record(s: &Spec) -> Result<ExactRecord, String> {
    let path = s.exact_record.clone().ok_or_else(|| format!("{}: no record path", s.name))?;
    let t = fs::read_to_string(&path).map_err(|_| path.display().to_string())?;
    let e_super = json_num(&t, "e_super");
    let de = json_num(&t, "de_exact");
    if !e_super.is_finite() || !de.is_finite() {
        return Err(format!("{} (no e_super / de_exact)", path.display()));
    }
    let rec_a = json_centers(&t, "donor_centers");
    let rec_b = json_centers(&t, "acceptor_centers");
    let ok = rec_a.len() == s.a.centers.len()
        && rec_b.len() == s.b.centers.len()
        && rec_a.iter().zip(s.a.centers.iter()).all(|(p, q)| dist(p, q) < SAME_GEOMETRY_TOL)
        && rec_b.iter().zip(s.b.centers.iter()).all(|(p, q)| dist(p, q) < SAME_GEOMETRY_TOL);
    if !ok {
        return Err(format!("{} (its centers are NOT the geometry this runner builds — M-STALE-INSTRUMENT)", path.display()));
    }
    Ok(ExactRecord {
        e_super,
        de_exact: de,
        e_a0: json_num(&t, "e_a0"),
        e_b0: json_num(&t, "e_b0"),
        davidson_iters: json_num(&t, "davidson_iters") as usize,
        residual: json_num(&t, "residual"),
        converged: json_bool(&t, "converged"),
        exit: json_str(&t, "exit"),
        cpu_seconds: json_num(&t, "cpu_seconds"),
        wall_seconds: json_num(&t, "wall_seconds"),
        source: path.display().to_string(),
    })
}

/// One block-localised reading, read back from a `sector_*.json` of record or from this
/// campaign's own `node_*.json`.
struct Sector {
    e_noct: f64,
    e_hl_undeformed: f64,
    e_a0: f64,
    e_b0: f64,
    sector_dim: usize,
    davidson_iters: usize,
    residual: f64,
    converged: bool,
    metric_min_eigenvalue: f64,
    cpu_seconds: f64,
    wall_seconds: f64,
    price_admitted: bool,
    source: String,
}

fn sector_from_text(t: &str, source: &str) -> Result<Sector, String> {
    let e_noct = json_num(t, "e_noct");
    if !e_noct.is_finite() {
        return Err(format!("{source} (no e_noct)"));
    }
    Ok(Sector {
        e_noct,
        e_hl_undeformed: json_num(t, "e_hl_undeformed"),
        e_a0: json_num(t, "e_a0"),
        e_b0: json_num(t, "e_b0"),
        sector_dim: json_num(t, "sector_dim") as usize,
        davidson_iters: json_num(t, "davidson_iters") as usize,
        residual: json_num(t, "residual"),
        converged: json_bool(t, "converged"),
        metric_min_eigenvalue: json_num(t, "metric_min_eigenvalue"),
        cpu_seconds: json_num(t, "cpu_seconds"),
        wall_seconds: json_num(t, "wall_seconds"),
        price_admitted: json_bool(t, "price_admitted"),
        source: source.to_string(),
    })
}

/// The block-localised reading's own JSON body (the fields, no braces), shared by `map`'s node
/// records and `predict`'s sector record.
fn sector_body(r: &BlwReading, cpu: f64, harness_wall: f64) -> String {
    let price_expected = PRICE_TENTH * (r.davidson_iters as f64) * SIGMA_PRICE_FLOOR;
    let price_admitted = cpu >= price_expected;
    format!(
        "\"e_noct\": {}, \"e_hl_undeformed\": {}, \"e_a0\": {}, \"e_b0\": {}, \"closed_sector_gain\": {},\n  \"sector_dim\": {}, \"sector_dim_staked\": {SECTOR_DIM_STAKED}, \"n_det\": {}, \"davidson_iters\": {}, \"residual\": {}, \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}, \"converged\": {}, \"capped\": {},\n  \"metric_min_eigenvalue\": {}, \"wall_seconds\": {}, \"sigma_seconds\": {}, \"harness_wall_seconds\": {}, \"cpu_seconds\": {}, \"threads\": {},\n  \"price_floor_core_seconds_per_sigma\": {SIGMA_PRICE_FLOOR}, \"price_rule\": \"cpu_seconds >= {PRICE_TENTH} * davidson_iters * {SIGMA_PRICE_FLOOR} (M-CHEAPER-THAN-ITS-PRICE)\", \"price_expected_core_seconds\": {}, \"price_admitted\": {price_admitted}",
        jn(r.e_noct),
        jn(r.e_hl_undeformed),
        jn(r.e_a0),
        jn(r.e_b0),
        jn(r.e_noct - r.e_hl_undeformed),
        r.sector_dim,
        r.n_det,
        r.davidson_iters,
        jn(r.residual),
        r.converged,
        r.davidson_iters >= DAVIDSON_CAP,
        jn(r.metric_min_eigenvalue),
        jd(r.wall_seconds),
        jd(r.sigma_seconds),
        jd(harness_wall),
        jd(cpu),
        threads(),
        jd(price_expected),
    )
}

// --------------------------------------------------------------------------- the map phase

/// G-D0 on one node of record: the SAME geometry rebuilt, its centers checked, and
/// `fci_full_from_product` run against the record's own energy.
fn gd0_node(out: &Path, s: &Spec, rec: &ExactRecord) {
    let path = out.join(format!("gd0_{}.json", s.name));
    if path.exists() {
        eprintln!("  G-D0 {}: exists, skipped", s.name);
        return;
    }
    let t0 = Instant::now();
    let c0 = cpu_seconds();
    let (e_full, iters, residual, converged) = fci_full_from_product(&s.a, &s.b);
    let wall = t0.elapsed().as_secs_f64();
    let cpu = cpu_seconds() - c0;
    let miss = (e_full - rec.e_super).abs();
    let within = miss <= GD0_TOL;
    let price_expected = PRICE_TENTH * (iters as f64) * SIGMA_PRICE_FLOOR;
    let price_admitted = cpu >= price_expected;
    fs::write(
        &path,
        format!(
            "{{\n  \"node\": \"{}\", \"kind\": \"{}\", \"family\": \"{}\", \"r_oo_angstrom\": {:.3}, \"r_oo_bohr\": {},\n  \"solver\": \"holon_chem::heitler_london::fci_full_from_product (the Davidson on the FULL space from the undeformed Heitler-London product)\",\n  \"e_full\": {}, \"e_record\": {}, \"miss\": {}, \"tolerance\": {GD0_TOL:e}, \"within\": {within},\n  \"davidson_iters\": {iters}, \"residual\": {}, \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}, \"converged\": {converged}, \"capped\": {},\n  \"n_det\": {N_DET_DIMER}, \"wall_seconds\": {}, \"cpu_seconds\": {}, \"threads\": {},\n  \"price_rule\": \"cpu_seconds >= {PRICE_TENTH} * davidson_iters * {SIGMA_PRICE_FLOOR} (M-CHEAPER-THAN-ITS-PRICE)\", \"price_expected_core_seconds\": {}, \"price_admitted\": {price_admitted},\n  \"stock\": {{\"cpu_seconds\": {}, \"davidson_iters\": {}, \"residual\": {}, \"exit\": \"{}\", \"converged\": {}, \"wall_seconds\": {}, \"e_a0\": {}, \"e_b0\": {}, \"de_exact\": {}, \"source\": \"{}\"}},\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
            s.name,
            esc(&s.kind),
            s.family,
            s.r_ang,
            jd(cross_oo(&s.a, &s.b)),
            jn(e_full),
            jn(rec.e_super),
            jn(miss),
            jn(residual),
            iters >= DAVIDSON_CAP,
            jd(wall),
            jd(cpu),
            threads(),
            jd(price_expected),
            jd(rec.cpu_seconds),
            rec.davidson_iters,
            jn(rec.residual),
            esc(&rec.exit),
            rec.converged,
            jd(rec.wall_seconds),
            jn(rec.e_a0),
            jn(rec.e_b0),
            jn(rec.de_exact),
            esc(&rec.source),
            centers_json(&s.a),
            centers_json(&s.b),
        ),
    )
    .unwrap();
    eprintln!(
        "  G-D0 {}: E_full {e_full:.12e} vs record {:.12e} — miss {miss:.3e} (bar {GD0_TOL:e}, within {within}), {iters} iters, residual {residual:.3e}, converged {converged}, {cpu:.0} core-s (stock {:.0} core-s at {} iterations)",
        s.name, rec.e_super, rec.cpu_seconds, rec.davidson_iters
    );
}

/// One NEW node: the closure reading first (fewer than two units ⇒ named, NOT solved), then the
/// exact solve from the product start and the block-localised sector solve, each priced.
fn map_node(out: &Path, s: &Spec) {
    let node_path = out.join(format!("node_{}.json", s.name));
    let outside_path = out.join(format!("outside_{}.json", s.name));
    if node_path.exists() {
        eprintln!("  {}: exists, skipped", s.name);
        return;
    }
    if outside_path.exists() {
        eprintln!("  {}: named OUTSIDE the identity already, skipped", s.name);
        return;
    }
    // ------------------------------------------------- the closure identity, before any solve
    let scene = engine_dimer(&s.a, &s.b, None, SeamPlant::None);
    let units = scene.seam_work.units;
    let reading = scene.units_reading();
    let reading_json = reading.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(", ");
    drop(scene);
    if units < 2 {
        fs::write(
            &outside_path,
            format!(
                "{{\n  \"node\": \"{}\", \"family\": \"{}\", \"kind\": \"{}\", \"naming\": \"{}\",\n  \"r_oo_angstrom\": {:.3}, \"r_oo_bohr\": {}, \"twist_degrees\": {:.1}, \"tilt_degrees\": {:.1}, \"donor_bend_degrees\": {:.1},\n  \"units\": {units}, \"units_reading\": [{reading_json}], \"units_free_marker\": 4294967295,\n  \"solved\": false,\n  \"rule\": \"a geometry at which the engine's closure reading finds fewer than two units is OUTSIDE the closure identity, is NOT solved, and is named (CT2_PREREG §0, FIELD-9's rule; M-EMPTY-SECTOR)\",\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
                s.name,
                s.family,
                esc(&s.kind),
                esc(NAMING),
                s.r_ang,
                jd(cross_oo(&s.a, &s.b)),
                s.twist_deg,
                s.tilt_deg,
                s.donor_deg,
                centers_json(&s.a),
                centers_json(&s.b),
            ),
        )
        .unwrap();
        eprintln!("  {}: the closure reading finds {units} unit(s) — OUTSIDE the identity, NOT solved (named in outside_{}.json)", s.name, s.name);
        return;
    }

    // ---------------------------------------------- the exact solve, checkpointed on its own
    let exact_path = out.join(format!("exact_{}.json", s.name));
    let (e_full, iters, residual, converged, ex_wall, ex_cpu) = if let Ok(t) = fs::read_to_string(&exact_path) {
        let rec_a = json_centers(&t, "donor_centers");
        let ok = rec_a.len() == s.a.centers.len() && rec_a.iter().zip(s.a.centers.iter()).all(|(p, q)| dist(p, q) < SAME_GEOMETRY_TOL);
        assert!(ok, "{}: the checkpoint's centers are not this geometry (M-STALE-INSTRUMENT)", exact_path.display());
        eprintln!("  {}: the exact solve is on disk already ({}), re-using it", s.name, exact_path.display());
        (
            json_num(&t, "e_total"),
            json_num(&t, "davidson_iters") as usize,
            json_num(&t, "residual"),
            json_bool(&t, "converged"),
            json_num(&t, "wall_seconds"),
            json_num(&t, "cpu_seconds"),
        )
    } else {
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let (e, it, res, conv) = fci_full_from_product(&s.a, &s.b);
        let wall = t0.elapsed().as_secs_f64();
        let cpu = cpu_seconds() - c0;
        let price_expected = PRICE_TENTH * (it as f64) * SIGMA_PRICE_FLOOR;
        fs::write(
            &exact_path,
            format!(
                "{{\n  \"node\": \"{}\", \"solver\": \"holon_chem::heitler_london::fci_full_from_product\",\n  \"e_total\": {}, \"davidson_iters\": {it}, \"residual\": {}, \"converged\": {conv}, \"capped\": {},\n  \"n_det\": {N_DET_DIMER}, \"wall_seconds\": {}, \"cpu_seconds\": {}, \"threads\": {},\n  \"price_expected_core_seconds\": {}, \"price_admitted\": {},\n  \"checkpoint\": \"the exact half of node_{}.json, written the moment it is known so a death between the two solves does not repeat this one\",\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
                s.name,
                jn(e),
                jn(res),
                it >= DAVIDSON_CAP,
                jd(wall),
                jd(cpu),
                threads(),
                jd(price_expected),
                cpu >= price_expected,
                s.name,
                centers_json(&s.a),
                centers_json(&s.b),
            ),
        )
        .unwrap();
        (e, it, res, conv, wall, cpu)
    };
    let ex_price_expected = PRICE_TENTH * (iters as f64) * SIGMA_PRICE_FLOOR;
    let ex_price_admitted = ex_cpu >= ex_price_expected;

    // -------------------------------------------------------------- the block-localised sector
    let t1 = Instant::now();
    let c1 = cpu_seconds();
    let r = fci_block_localised(&s.a, &s.b);
    let harness_wall = t1.elapsed().as_secs_f64();
    let sec_cpu = cpu_seconds() - c1;
    let de_exact = e_full - (r.e_a0 + r.e_b0);
    let e_ct = e_full - r.e_noct;
    let (oo, ho, hh) = cross_classes(&s.a, &s.b);
    let order_ok = e_full <= r.e_noct + ORDER_TOL && r.e_noct <= r.e_hl_undeformed + ORDER_TOL;
    fs::write(
        &node_path,
        format!(
            "{{\n  \"node\": \"{}\", \"family\": \"{}\", \"kind\": \"{}\", \"naming\": \"{}\",\n  \"r_oo_angstrom\": {:.3}, \"r_oo_bohr\": {}, \"twist_degrees\": {:.1}, \"tilt_degrees\": {:.1}, \"donor_bend_degrees\": {:.1},\n  \"units\": {units}, \"units_reading\": [{reading_json}], \"units_free_marker\": 4294967295,\n  \"exact\": {{\"solver\": \"holon_chem::heitler_london::fci_full_from_product\", \"e_total\": {}, \"davidson_iters\": {iters}, \"residual\": {}, \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}, \"converged\": {converged}, \"capped\": {}, \"n_det\": {N_DET_DIMER}, \"wall_seconds\": {}, \"cpu_seconds\": {}, \"price_expected_core_seconds\": {}, \"price_admitted\": {ex_price_admitted}}},\n  \"sector\": {{{}}},\n  \"e_a0\": {}, \"e_b0\": {}, \"de_exact\": {}, \"de_exact_rule\": \"E_exact(total) - (E_A0 + E_B0), the monomer references from the sector solve's own setup (the same solve_embedded call the records used)\",\n  \"e_ct\": {}, \"e_ct_rule\": \"E_exact(total) - E_noCT(total)\", \"order_ok\": {order_ok}, \"order_tolerance\": {ORDER_TOL:e},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}], \"min_cross_ho_bohr\": {},\n  \"threads\": {}, \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
            s.name,
            s.family,
            esc(&s.kind),
            esc(NAMING),
            s.r_ang,
            jd(cross_oo(&s.a, &s.b)),
            s.twist_deg,
            s.tilt_deg,
            s.donor_deg,
            jn(e_full),
            jn(residual),
            iters >= DAVIDSON_CAP,
            jd(ex_wall),
            jd(ex_cpu),
            jd(ex_price_expected),
            sector_body(&r, sec_cpu, harness_wall),
            jn(r.e_a0),
            jn(r.e_b0),
            jn(de_exact),
            jn(e_ct),
            list_json(&oo),
            list_json(&ho),
            list_json(&hh),
            jd(min_or_nan(&ho)),
            threads(),
            centers_json(&s.a),
            centers_json(&s.b),
        ),
    )
    .unwrap();
    let _ = fs::remove_file(&exact_path);
    eprintln!(
        "  {}: units {units}, ΔE_exact {de_exact:.6e}, E_CT {e_ct:.6e} Ha | exact {iters} iters, residual {residual:.1e}, converged {converged}, {ex_cpu:.0} core-s (priced {ex_price_admitted}) | sector dim {}, {} iters, residual {:.1e}, converged {}, {sec_cpu:.0} core-s | order {order_ok}",
        s.name, r.sector_dim, r.davidson_iters, r.residual, r.converged
    );
}

fn run_map(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    eprintln!("CT-2 map — G-D0 on the {GD0_NODES} exact nodes of record, then the new geometries of the freeze's table, on {} threads", threads());

    let records = record_specs(out, o, h);
    let mut recs: Vec<ExactRecord> = Vec::new();
    let mut missing: Vec<String> = Vec::new();
    for s in &records {
        match load_exact_record(s) {
            Ok(r) => recs.push(r),
            Err(m) => missing.push(m),
        }
    }
    if !missing.is_empty() {
        eprintln!("REFUSED — G-D0 is read against the {GD0_NODES} exact records and these are not on disk (or are not this runner's geometry):");
        for m in &missing {
            eprintln!("  {m}");
        }
        eprintln!("Nothing written.");
        std::process::exit(3);
    }
    eprintln!("\nG-D0 — `fci_full_from_product` against the {GD0_NODES} records, bar {GD0_TOL:e}; every geometry REBUILT here and matched to its record's centers (M-STALE-INSTRUMENT):");
    for (s, r) in records.iter().zip(recs.iter()) {
        gd0_node(out, s, r);
    }

    // ---------------------------------------------------------- the new geometries, deduplicated
    let news = new_specs(o, h);
    eprintln!("\nthe freeze's new geometries: {} candidates (the freeze's table says {FREEZE_NEW} new solves of {FREEZE_TOTAL} nodes)", news.len());
    let mut to_solve: Vec<&Spec> = Vec::new();
    for s in &news {
        let mut dup: Option<(&Spec, f64)> = None;
        for r in &records {
            let d = geometry_deviation(&s.a, &s.b, &r.a, &r.b);
            if d < SAME_GEOMETRY_TOL {
                dup = Some((r, d));
                break;
            }
        }
        match dup {
            Some((r, d)) => {
                let path = out.join(format!("duplicate_{}.json", s.name));
                if !path.exists() {
                    fs::write(
                        &path,
                        format!(
                            "{{\n  \"node\": \"{}\", \"family\": \"{}\", \"kind\": \"{}\",\n  \"duplicates\": \"{}\", \"duplicate_kind\": \"{}\", \"max_center_deviation_bohr\": {}, \"tolerance_bohr\": {SAME_GEOMETRY_TOL:e},\n  \"solved\": false,\n  \"rule\": \"CT2_PREREG §0: a candidate that reproduces a node of record to 1e-9 bohr is NOT solved, is written here naming the record, and is counted ONCE in every fit and gate under the record's name. The freeze's table names three such coincidences: the tilt family's 2.9 A / 30 deg, its 3.4 A / 180 deg (which IS flipped_R3.4, the acceptor turned by pi about x through its own oxygen), and the twist family's 3.0 A / 60 deg.\",\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
                            s.name,
                            s.family,
                            esc(&s.kind),
                            r.name,
                            esc(&r.kind),
                            jn(d),
                            centers_json(&s.a),
                            centers_json(&s.b),
                        ),
                    )
                    .unwrap();
                }
                eprintln!("  {}: IS the node of record `{}` (worst center deviation {d:.3e} bohr) — NOT solved again, named in duplicate_{}.json", s.name, r.name, s.name);
            }
            None => to_solve.push(s),
        }
    }
    eprintln!("\n{} new geometries to solve after the duplicate check:", to_solve.len());
    for s in &to_solve {
        map_node(out, s);
    }
    fs::write(out.join("map.done"), "done\n").unwrap();
    eprintln!("map.done written");
}

// --------------------------------------------------------------------------- the fit phase

/// One node as the fit sees it: its geometry, its two energies, and the cross-unit distances and
/// H–O pair frames the terms are built on.
struct FNode {
    spec: Spec,
    e_exact: f64,
    de_exact: f64,
    sector: Sector,
    exact_iters: usize,
    exact_residual: f64,
    exact_converged: bool,
    exact_cpu: f64,
    exact_price_admitted: bool,
    exact_source: String,
    is_new: bool,
    oo: Vec<f64>,
    ho: Vec<f64>,
    hh: Vec<f64>,
    pairs: Vec<CtPair>,
}

impl FNode {
    fn e_ct(&self) -> f64 {
        self.e_exact - self.sector.e_noct
    }
    fn tolerance(&self) -> f64 {
        (S1_FRAC * self.de_exact.abs()).max(S1_ABS)
    }
    fn weight(&self) -> f64 {
        1.0 / (self.de_exact.abs().max(WEIGHT_FLOOR)).powi(2)
    }
    fn min_ho(&self) -> f64 {
        min_or_nan(&self.ho)
    }
}

fn finish_node(spec: Spec, e_exact: f64, de_exact: f64, sector: Sector, exact_iters: usize, exact_residual: f64, exact_converged: bool, exact_cpu: f64, exact_price_admitted: bool, exact_source: String, is_new: bool) -> FNode {
    let (oo, ho, hh) = cross_classes(&spec.a, &spec.b);
    let pairs = ct_pairs(&spec.a, &spec.b);
    FNode { spec, e_exact, de_exact, sector, exact_iters, exact_residual, exact_converged, exact_cpu, exact_price_admitted, exact_source, is_new, oo, ho, hh, pairs }
}

/// Every node the fit reads: the thirteen of record (their own exact records and CT-1's sector
/// readings) and this campaign's new nodes (`node_*.json`). A geometry named OUTSIDE the identity
/// or named a DUPLICATE of a record is reported and not loaded.
fn load_nodes(out: &Path, o: Species, h: Species) -> (Vec<FNode>, Vec<String>, Vec<String>, Vec<String>) {
    let mut nodes: Vec<FNode> = Vec::new();
    let mut outside: Vec<String> = Vec::new();
    let mut duplicate: Vec<String> = Vec::new();
    let mut missing: Vec<String> = Vec::new();
    for s in record_specs(out, o, h) {
        let rec = match load_exact_record(&s) {
            Ok(r) => r,
            Err(m) => {
                missing.push(m);
                continue;
            }
        };
        let sp = s.sector_record.clone().expect("a record node names its sector reading");
        let sector = match fs::read_to_string(&sp) {
            Ok(t) => match sector_from_text(&t, &sp.display().to_string()) {
                Ok(x) => x,
                Err(m) => {
                    missing.push(m);
                    continue;
                }
            },
            Err(_) => {
                missing.push(sp.display().to_string());
                continue;
            }
        };
        let (e, de, src) = (rec.e_super, rec.de_exact, rec.source.clone());
        let is_new = !s.of_record;
        nodes.push(finish_node(s, e, de, sector, rec.davidson_iters, rec.residual, rec.converged, rec.cpu_seconds, true, src, is_new));
    }
    for s in new_specs(o, h) {
        if out.join(format!("outside_{}.json", s.name)).exists() {
            outside.push(s.name.clone());
            continue;
        }
        if out.join(format!("duplicate_{}.json", s.name)).exists() {
            duplicate.push(s.name.clone());
            continue;
        }
        let p = out.join(format!("node_{}.json", s.name));
        let Ok(t) = fs::read_to_string(&p) else {
            missing.push(p.display().to_string());
            continue;
        };
        let rec_a = json_centers(&t, "donor_centers");
        let rec_b = json_centers(&t, "acceptor_centers");
        let ok = rec_a.len() == s.a.centers.len()
            && rec_b.len() == s.b.centers.len()
            && rec_a.iter().zip(s.a.centers.iter()).all(|(x, y)| dist(x, y) < SAME_GEOMETRY_TOL)
            && rec_b.iter().zip(s.b.centers.iter()).all(|(x, y)| dist(x, y) < SAME_GEOMETRY_TOL);
        if !ok {
            missing.push(format!("{} (its centers are NOT the geometry this runner builds — M-STALE-INSTRUMENT)", p.display()));
            continue;
        }
        let sector = match sector_from_text(section(&t, "sector"), &p.display().to_string()) {
            Ok(x) => x,
            Err(m) => {
                missing.push(m);
                continue;
            }
        };
        let ex = section(&t, "exact");
        let e = json_num(ex, "e_total");
        let de = json_num(&t, "de_exact");
        if !e.is_finite() || !de.is_finite() {
            missing.push(format!("{} (no e_total / de_exact)", p.display()));
            continue;
        }
        let src = p.display().to_string();
        let is_new = !s.of_record;
        nodes.push(finish_node(
            s,
            e,
            de,
            sector,
            json_num(ex, "davidson_iters") as usize,
            json_num(ex, "residual"),
            json_bool(ex, "converged"),
            json_num(ex, "cpu_seconds"),
            json_bool(ex, "price_admitted"),
            src,
            is_new,
        ));
    }
    (nodes, outside, duplicate, missing)
}

/// FIELD-8's non-negativity rule on TWO classes: drop the most negative amplitude to an exact
/// `0.0` and refit. Returns `(amplitudes, kept, weighted residual)`.
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

/// THE CLAMP (CT2_PREREG §0): the largest amplitude of one contact class the boundedness gate
/// admits, by bisection on `[0, p_ls]` to `CLAMP_REL` RELATIVE, with every other term of the law
/// held. Returns `(amplitude, clamped, bounded-calls, the zero law is admitted)`.
fn clamp_amplitude(p_ls: f64, q_h: f64, r_min: [f64; 3], make: &dyn Fn(f64) -> SeamModel) -> (f64, bool, usize, bool) {
    let admits = |p: f64| make(p).bounded(q_h, r_min, KT).is_none();
    if !(p_ls > 0.0) {
        return (0.0, false, 1, admits(0.0));
    }
    if admits(p_ls) {
        return (p_ls, false, 1, true);
    }
    if !admits(0.0) {
        return (0.0, true, 2, false);
    }
    let (mut lo, mut hi) = (0.0f64, p_ls);
    let mut calls = 2usize;
    while hi - lo > CLAMP_REL * p_ls && calls < 200 {
        let mid = 0.5 * (lo + hi);
        if admits(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
        calls += 1;
    }
    (lo, true, calls, true)
}

fn run_fit(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    eprintln!("CT-2 fit — G-D0's verdict, T0, A1, P1, the angular family (S1), the two-class contact fit UNDER the boundedness gate (C1) with the unconstrained fit beside it, dispersion, G-B0, G-C1, plants (i) and (ii)");

    let (nodes, outside, duplicate, missing) = load_nodes(out, o, h);
    if !missing.is_empty() {
        eprintln!("REFUSED — these readings are not on disk (run `map` first):");
        for m in &missing {
            eprintln!("  {m}");
        }
        eprintln!("Nothing written.");
        std::process::exit(3);
    }
    let n = nodes.len();
    let n_new = nodes.iter().filter(|x| x.is_new).count();
    let n_record = n - n_new;
    let eighty = (EIGHTY * n as f64).ceil() as usize;
    let s1_b_min = S1_B_LITERAL;
    let c1_min = eighty;
    eprintln!(
        "\nthe node set: {n} distinct geometries — {n_record} of record, {n_new} new; the freeze's §0 table stakes {FREEZE_TOTAL} distinct and {FREEZE_NEW} new (its family count column sums to 67 with three geometries shared with the record).\n  named OUTSIDE the closure identity (not solved, not fit): {}\n  named a DUPLICATE of a node of record (not solved again, fit once under the record's name): {}",
        if outside.is_empty() { "none".to_string() } else { outside.join(", ") },
        if duplicate.is_empty() { "none".to_string() } else { duplicate.join(", ") }
    );
    eprintln!("  branch (b) needs {s1_b_min} of {n} — the freeze's literal ({S1_B_LITERAL} of {FREEZE_TOTAL}, its 80 %); 80 % of the MEASURED total is {eighty}, recorded beside it; C1 needs {c1_min}, 80 % of the measured total");

    // ------------------------------------------------------------------ the wall and CT-1's fit
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
    let r_min = [r_min_oo, r_min_oh, r_min_hh];
    let q_h = holon_render::field::water_charge_at_pin();
    eprintln!(
        "\nFIELD-9's wall, HELD [{}]:\n  a = {a_oo:.9e}, b = {b_oo:.6}; a_oh = {a_oh:.9e}, b_oh = {b_oh:.6}; a_hh = {a_hh:.9e}, b_hh = {b_hh:.6}\n  r_min: O–O {r_min_oo:.6}, H–O {r_min_oh:.6}, H–H {r_min_hh:.6} bohr; q_H {q_h:.12e} (wall9's record {q_h_record:.12e}, difference {:.3e}); kT {KT:e}",
        wall9_path.display(),
        (q_h - q_h_record).abs()
    );
    let wall_ct1_path = sibling(out, "ct1").join("wall_ct.json");
    let wall_ct1 = fs::read_to_string(&wall_ct1_path).unwrap_or_else(|_| panic!("{} is CT-1's fit, which P1 is read against", wall_ct1_path.display()));
    let (p_ct1_rec, c_ct1_rec) = (json_num(&wall_ct1, "p_ct"), json_num(&wall_ct1, "c_ct"));
    eprintln!("CT-1's transfer term of record [{}]: P_CT = {p_ct1_rec:.12e}, c_CT = {c_ct1_rec:.6} /bohr (the freeze quotes {CT1_P}, {CT1_C})", wall_ct1_path.display());

    // ===================================================================== G-D0 — the solver
    eprintln!("\nG-D0 — `fci_full_from_product` is the stock solver's equal on the {GD0_NODES} exact nodes of record (bar {GD0_TOL:e}), its price beside the stock's:");
    let mut gd0_lines: Vec<String> = Vec::new();
    let mut gd0_within = 0usize;
    let mut gd0_files = 0usize;
    let mut gd0_misses: Vec<String> = Vec::new();
    eprintln!("| node | E_full | E_record | miss | within | iters | residual | converged | core-s | stock core-s | stock iters |");
    for x in nodes.iter().filter(|x| !x.is_new) {
        let p = out.join(format!("gd0_{}.json", x.spec.name));
        let Ok(t) = fs::read_to_string(&p) else {
            gd0_misses.push(format!("{} (no gd0 record)", x.spec.name));
            continue;
        };
        gd0_files += 1;
        let (e_full, e_rec, miss) = (json_num(&t, "e_full"), json_num(&t, "e_record"), json_num(&t, "miss"));
        let (it, res, conv) = (json_num(&t, "davidson_iters") as usize, json_num(&t, "residual"), json_bool(&t, "converged"));
        let ok = miss <= GD0_TOL && conv && res <= SECTOR_RESIDUAL_BAR && it < DAVIDSON_CAP;
        if ok {
            gd0_within += 1;
        } else {
            gd0_misses.push(format!("{} (miss {miss:.3e}, residual {res:.3e}, converged {conv}, {it} iters)", x.spec.name));
        }
        eprintln!(
            "| {} | {e_full:.12e} | {e_rec:.12e} | {miss:.3e} | {ok} | {it} | {res:.3e} | {conv} | {:.0} | {:.0} | {} |",
            x.spec.name,
            json_num(&t, "cpu_seconds"),
            x.exact_cpu,
            x.exact_iters
        );
        gd0_lines.push(format!(
            "{{\"node\": \"{}\", \"e_full\": {}, \"e_record\": {}, \"miss\": {}, \"within\": {ok}, \"davidson_iters\": {it}, \"residual\": {}, \"converged\": {conv}, \"cpu_seconds\": {}, \"wall_seconds\": {}, \"price_admitted\": {}, \"stock_cpu_seconds\": {}, \"stock_davidson_iters\": {}, \"stock_residual\": {}, \"source\": \"{}\"}}",
            x.spec.name,
            jn(e_full),
            jn(e_rec),
            jn(miss),
            jn(res),
            jn(json_num(&t, "cpu_seconds")),
            jn(json_num(&t, "wall_seconds")),
            json_bool(&t, "price_admitted"),
            jn(x.exact_cpu),
            x.exact_iters,
            jn(x.exact_residual),
            esc(&p.display().to_string()),
        ));
    }
    let g_d0 = gd0_within == GD0_NODES && gd0_files == GD0_NODES;
    eprintln!(
        "G-D0: {} — {gd0_within} of {GD0_NODES} within {GD0_TOL:e} with a converged exit and a residual at or under {SECTOR_RESIDUAL_BAR:e} ({gd0_files} records read){}",
        if g_d0 { "PASS" } else { "FAIL" },
        if gd0_misses.is_empty() { String::new() } else { format!("; misses: {}", gd0_misses.join(", ")) }
    );

    // ======================================================== T0 — the sector is what it says
    eprintln!("\nT0 — on every NEW node: residual ≤ {SECTOR_RESIDUAL_BAR:e}, exit Converged, dimension {SECTOR_DIM_STAKED} EXACT, E_exact ≤ E_noCT ≤ E_HL(undeformed) on TOTALS within {ORDER_TOL:e}. The {n_record} of record are reported beside the gate (CT-1's T0 read them).");
    let mut t0_fails: Vec<String> = Vec::new();
    let mut price_refused: Vec<String> = Vec::new();
    let mut t0_lines: Vec<String> = Vec::new();
    let e_ct: Vec<f64> = nodes.iter().map(|x| x.e_ct()).collect();
    eprintln!("| node | new | E_exact | E_noCT | E_HL(undef) | E_CT | order | dim | iters | residual | converged | core-s | priced |");
    for (g, x) in nodes.iter().enumerate() {
        let s = &x.sector;
        let order_ok = x.e_exact <= s.e_noct + ORDER_TOL && s.e_noct <= s.e_hl_undeformed + ORDER_TOL;
        let dim_ok = s.sector_dim == SECTOR_DIM_STAKED;
        let res_ok = s.residual <= SECTOR_RESIDUAL_BAR;
        let capped = s.davidson_iters >= DAVIDSON_CAP;
        let ok = order_ok && dim_ok && res_ok && s.converged && !capped;
        if !ok && x.is_new {
            t0_fails.push(format!("{} (converged {}, residual {:.3e}, dim {}, order {order_ok}, capped {capped})", x.spec.name, s.converged, s.residual, s.sector_dim));
        }
        if !s.price_admitted {
            price_refused.push(format!("{} sector ({:.0} core-s for {} iterations)", x.spec.name, s.cpu_seconds, s.davidson_iters));
        }
        if x.is_new && !x.exact_price_admitted {
            price_refused.push(format!("{} exact ({:.0} core-s for {} iterations)", x.spec.name, x.exact_cpu, x.exact_iters));
        }
        eprintln!(
            "| {} | {} | {:.12e} | {:.12e} | {:.12e} | {:.6e} | {order_ok} | {} | {} | {:.3e} | {} | {:.0} | {} |",
            x.spec.name, x.is_new, x.e_exact, s.e_noct, s.e_hl_undeformed, e_ct[g], s.sector_dim, s.davidson_iters, s.residual, s.converged, s.cpu_seconds, s.price_admitted
        );
        t0_lines.push(format!(
            "{{\"node\": \"{}\", \"new\": {}, \"converged\": {}, \"residual\": {}, \"residual_ok\": {res_ok}, \"sector_dim\": {}, \"sector_dim_ok\": {dim_ok}, \"davidson_iters\": {}, \"capped\": {capped}, \"metric_min_eigenvalue\": {}, \"order_ok\": {order_ok}, \"e_ct\": {}, \"sector_cpu_seconds\": {}, \"sector_wall_seconds\": {}, \"sector_price_admitted\": {}, \"exact_cpu_seconds\": {}, \"exact_davidson_iters\": {}, \"exact_converged\": {}, \"exact_price_admitted\": {}, \"pass\": {ok}}}",
            x.spec.name,
            x.is_new,
            s.converged,
            jn(s.residual),
            s.sector_dim,
            s.davidson_iters,
            jn(s.metric_min_eigenvalue),
            jn(e_ct[g]),
            jn(s.cpu_seconds),
            jn(s.wall_seconds),
            s.price_admitted,
            jn(x.exact_cpu),
            x.exact_iters,
            x.exact_converged,
            x.exact_price_admitted,
        ));
    }
    let t0 = t0_fails.is_empty();
    eprintln!(
        "T0: {} — {} of {n_new} new nodes pass every leg{}",
        if t0 { "PASS" } else { "FAIL" },
        n_new - t0_fails.len(),
        if t0_fails.is_empty() { String::new() } else { format!("; failures: {}", t0_fails.join(", ")) }
    );
    eprintln!(
        "M-CHEAPER-THAN-ITS-PRICE: every solve at or above a tenth of its own iteration count times {SIGMA_PRICE_FLOOR} core-seconds: {}{}",
        price_refused.is_empty(),
        if price_refused.is_empty() { String::new() } else { format!(" — REFUSED: {}", price_refused.join(", ")) }
    );

    // ============================================================================= A1 — the map
    eprintln!("\nA1 — the expectation of §1, written before any new solve: E_CT < −{CT_FLOOR:e} at every node whose shortest cross-unit H···O is at or under {A1_CONTACT_BOHR} bohr; a node beyond that may read null, and reads it as a NULL, not a miss (M-NULL-MISSTAKE).");
    let mut a1_misses: Vec<String> = Vec::new();
    let mut a1_in = 0usize;
    let mut a1_nulls: Vec<String> = Vec::new();
    for (g, x) in nodes.iter().enumerate() {
        let m = x.min_ho();
        if m <= A1_CONTACT_BOHR {
            a1_in += 1;
            if !(e_ct[g] < -CT_FLOOR) {
                a1_misses.push(format!("{} (E_CT {:.6e}, shortest H···O {m:.4} bohr)", x.spec.name, e_ct[g]));
            }
        } else if !(e_ct[g] < -CT_FLOOR) {
            a1_nulls.push(format!("{} (E_CT {:.6e}, shortest H···O {m:.4} bohr)", x.spec.name, e_ct[g]));
        }
    }
    let a1 = a1_misses.is_empty();
    eprintln!(
        "A1: {} — {} of {a1_in} nodes inside {A1_CONTACT_BOHR} bohr read attractive{}{}",
        if a1 { "PASS" } else { "FAIL" },
        a1_in - a1_misses.len(),
        if a1_misses.is_empty() { String::new() } else { format!("; misses: {}", a1_misses.join(", ")) },
        if a1_nulls.is_empty() { String::new() } else { format!("; beyond {A1_CONTACT_BOHR} bohr and reading null (a null, not a miss): {}", a1_nulls.join(", ")) }
    );

    // the four tables
    let mut table_json: Vec<String> = Vec::new();
    let mut group = |title: &str, key: &str, rows: Vec<usize>, angle: &dyn Fn(&Spec) -> f64| {
        if rows.is_empty() {
            return;
        }
        eprintln!("  {title}");
        eprintln!("  | node | angle (deg) | R_OO (Å) | shortest H···O (bohr) | E_CT (Ha) | ΔE_exact (Ha) | of record |");
        let mut entries: Vec<String> = Vec::new();
        for &g in rows.iter() {
            let x = &nodes[g];
            eprintln!(
                "  | {} | {:.0} | {:.1} | {:.4} | {:.6e} | {:.6e} | {} |",
                x.spec.name,
                angle(&x.spec),
                x.spec.r_ang,
                x.min_ho(),
                e_ct[g],
                x.de_exact,
                !x.is_new
            );
            entries.push(format!(
                "{{\"node\": \"{}\", \"angle_degrees\": {:.1}, \"r_angstrom\": {:.1}, \"min_cross_ho_bohr\": {}, \"e_ct\": {}, \"de_exact\": {}, \"of_record\": {}}}",
                x.spec.name,
                angle(&x.spec),
                x.spec.r_ang,
                jd(x.min_ho()),
                jn(e_ct[g]),
                jn(x.de_exact),
                !x.is_new
            ));
        }
        table_json.push(format!("\"{key}\": {{\"title\": \"{}\", \"rows\": [{}]}}", esc(title), entries.join(", ")));
    };
    eprintln!("\nA1's tables — E_CT against the angle, family by family (the linear node at each separation is the tilt family's 0° row, marked of record):");
    for &r in TILT_R.iter() {
        let mut rows: Vec<usize> = (0..n).filter(|&g| (nodes[g].spec.family == "tilt" || nodes[g].spec.linear_node()) && (nodes[g].spec.r_ang - r).abs() < 1e-9).collect();
        rows.sort_by(|&i, &j| nodes[i].spec.tilt_deg.partial_cmp(&nodes[j].spec.tilt_deg).unwrap());
        group(&format!("E_CT against acceptor tilt at R_OO = {r:.1} Å"), &format!("tilt_R{r:.1}"), rows, &|s: &Spec| s.tilt_deg);
    }
    for &r in TWIST_R.iter() {
        let mut rows: Vec<usize> = (0..n).filter(|&g| nodes[g].spec.family == "twist" && (nodes[g].spec.r_ang - r).abs() < 1e-9).collect();
        rows.sort_by(|&i, &j| nodes[i].spec.tilt_deg.partial_cmp(&nodes[j].spec.tilt_deg).unwrap());
        group(&format!("E_CT against tilt after a {TWIST_PHI:.0}° twist at R_OO = {r:.1} Å"), &format!("twist_R{r:.1}"), rows, &|s: &Spec| s.tilt_deg);
    }
    for &r in DONOR_R.iter() {
        let mut rows: Vec<usize> = (0..n).filter(|&g| nodes[g].spec.family == "donor" && (nodes[g].spec.r_ang - r).abs() < 1e-9).collect();
        rows.sort_by(|&i, &j| nodes[i].spec.donor_deg.partial_cmp(&nodes[j].spec.donor_deg).unwrap());
        group(&format!("E_CT against donor bend at R_OO = {r:.1} Å"), &format!("donor_R{r:.1}"), rows, &|s: &Spec| s.donor_deg);
    }
    let mut rows: Vec<usize> = (0..n).filter(|&g| nodes[g].spec.family == "dbent" || nodes[g].spec.family == "twistbent").collect();
    rows.sort_by(|&i, &j| nodes[i].spec.name.cmp(&nodes[j].spec.name));
    group("E_CT on the doubly varied geometries", "doubly_varied", rows, &|s: &Spec| s.donor_deg);

    // ============================================== the angular family, fit on every node (S1)
    let we: Vec<f64> = nodes.iter().map(|x| x.weight()).collect();
    let syy_ct: f64 = (0..n).map(|g| we[g] * e_ct[g] * e_ct[g]).sum();
    // the pairs of every node, flattened, and the exponential table over the c-grid
    let mut pair_of: Vec<(usize, usize)> = Vec::new(); // (node, index within the node)
    for (g, x) in nodes.iter().enumerate() {
        for i in 0..x.pairs.len() {
            pair_of.push((g, i));
        }
    }
    let np = pair_of.len();
    let mut ex = vec![0.0f64; NC * np];
    for i in 0..NC {
        let c = cgrid(i);
        for (p, &(g, k)) in pair_of.iter().enumerate() {
            ex[i * np + p] = (-c * nodes[g].pairs[k].r).exp();
        }
    }
    eprintln!(
        "\nthe transfer family (§0), fit on all {n} nodes: c on 0.50..=4.00 step 0.02 ({NC} values), m ∈ {M_GRID:?}, k ∈ {K_GRID:?}, λ ∈ {LAM_DEG:?}°, P by weighted least squares in closed form per grid point, weights 1/max(|ΔE_exact|, {WEIGHT_FLOOR:e})², P ≥ 0. {} grid points over {np} cross-unit H–O pairs; ties go to the first point in the order m, k, λ, c ascending.",
        NC * M_GRID.len() * K_GRID.len() * LAM_DEG.len()
    );
    let t_fit = Instant::now();
    let mut angf = vec![0.0f64; np];
    let mut xg = vec![0.0f64; n];
    let (mut best_res, mut best) = (f64::INFINITY, (0.0f64, 0.0f64, 0i32, 0i32, 0.0f64, 0.0f64));
    // P1's restricted fit, gathered in the same sweep: m = k = 0 on CT-1's twelve
    let ct1_twelve: Vec<usize> = (0..n).filter(|&g| !nodes[g].is_new && nodes[g].spec.name != CT1S2_NAME).collect();
    let (mut p1_res, mut p1_best) = (f64::INFINITY, (0.0f64, 0.0f64));
    for &m in M_GRID.iter() {
        for &k in K_GRID.iter() {
            for &ldeg in LAM_DEG.iter() {
                let lam = ldeg * std::f64::consts::PI / 180.0;
                let (cl, sl) = (lam.cos(), lam.sin());
                for (p, &(g, j)) in pair_of.iter().enumerate() {
                    angf[p] = ct_angular_factor(&nodes[g].pairs[j], m, k, cl, sl);
                }
                for i in 0..NC {
                    for v in xg.iter_mut() {
                        *v = 0.0;
                    }
                    for (p, &(g, _)) in pair_of.iter().enumerate() {
                        xg[g] -= ex[i * np + p] * angf[p];
                    }
                    let (mut num, mut den) = (0.0f64, 0.0f64);
                    for g in 0..n {
                        num += we[g] * xg[g] * e_ct[g];
                        den += we[g] * xg[g] * xg[g];
                    }
                    let p_raw = if den > 0.0 { num / den } else { 0.0 };
                    let p_use = p_raw.max(0.0);
                    let mut res = 0.0f64;
                    for g in 0..n {
                        let d = e_ct[g] - p_use * xg[g];
                        res += we[g] * d * d;
                    }
                    if res < best_res {
                        best_res = res;
                        best = (p_use, cgrid(i), m, k, lam, p_raw);
                    }
                    if m == 0 && k == 0 && ldeg == LAM_DEG[0] {
                        let (mut n2, mut d2) = (0.0f64, 0.0f64);
                        for &g in ct1_twelve.iter() {
                            n2 += we[g] * xg[g] * e_ct[g];
                            d2 += we[g] * xg[g] * xg[g];
                        }
                        let pr = if d2 > 0.0 { (n2 / d2).max(0.0) } else { 0.0 };
                        let mut r2 = 0.0f64;
                        for &g in ct1_twelve.iter() {
                            let d = e_ct[g] - pr * xg[g];
                            r2 += we[g] * d * d;
                        }
                        if r2 < p1_res {
                            p1_res = r2;
                            p1_best = (pr, cgrid(i));
                        }
                    }
                }
            }
        }
    }
    let (p_ct, c_ct, m_ct, k_ct, lambda_ct, p_ct_raw) = best;
    let lambda_deg = lambda_ct * 180.0 / std::f64::consts::PI;
    let lambda_matters = k_ct != 0;
    let ct_transferred = p_ct > 0.0;
    let fit_seconds = t_fit.elapsed().as_secs_f64();
    let edge_m = m_ct == MK_EDGE;
    let edge_k = k_ct == MK_EDGE;
    let edge_lambda = lambda_matters && ((lambda_deg - LAM_DEG[0]).abs() < 1e-9 || (lambda_deg - LAM_DEG[LAM_DEG.len() - 1]).abs() < 1e-9);
    let at_edge = edge_m || edge_k || edge_lambda;
    eprintln!(
        "  P = {p_ct:.9e} Ha (unclamped {p_ct_raw:.9e}), c = {c_ct:.2} /bohr, m = {m_ct}, k = {k_ct}, λ = {lambda_deg:.1}° ({lambda_ct:.9} rad{}), weighted residual {best_res:.9e} against Σw·E_CT² = {syy_ct:.9e}, in {fit_seconds:.1} s → {}",
        if lambda_matters { "" } else { ", IMMATERIAL at k = 0 — λ enters only through g_a" },
        if ct_transferred { "TRANSFERRED" } else { "NOT transferred (P clamped to an exact 0.0)" }
    );
    eprintln!(
        "  grid edges (M-FIRST-VIOLATION-ONLY): m at its edge ({MK_EDGE}) {edge_m}, k at its edge {edge_k}, λ at an edge ({}° or {}°) {edge_lambda}{} → {}",
        LAM_DEG[0],
        LAM_DEG[LAM_DEG.len() - 1],
        if lambda_matters { "" } else { " (λ is not read: k = 0)" },
        if at_edge { "the fit is read as (b) at best (§0)" } else { "no parameter sits at a grid edge" }
    );

    // ---------------------------------------------------------------- P1 — the family contains CT-1
    let (p1_p, p1_c) = p1_best;
    let p1_dp = (p1_p - CT1_P).abs();
    let p1_dc = (p1_c - CT1_C).abs();
    let p1 = p1_dp <= P1_TOL && p1_dc <= P1_TOL;
    eprintln!(
        "\nP1 — the family at m = k = 0 on CT-1's twelve: P = {p1_p:.12e} (CT-1's {CT1_P}, difference {p1_dp:.3e}), c = {p1_c:.12} (CT-1's {CT1_C}, difference {p1_dc:.3e}), bar {P1_TOL:e} → {}; the record's own numbers are P = {p_ct1_rec:.12e}, c = {c_ct1_rec:.6} (differences {:.3e}, {:.3e})",
        if p1 { "PASS" } else { "FAIL" },
        (p1_p - p_ct1_rec).abs(),
        (p1_c - c_ct1_rec).abs()
    );

    // ------------------------------------------------------------------------- S1, node by node
    let ct_fit_v: Vec<f64> = nodes.iter().map(|x| ct_value(&x.pairs, p_ct, c_ct, m_ct, k_ct, lambda_ct)).collect();
    let mut s1_within = 0usize;
    let mut s1_misses: Vec<String> = Vec::new();
    let mut s1_ok_v: Vec<bool> = Vec::with_capacity(n);
    eprintln!("\nS1 — the angular term against the measured E_CT, tolerance max({S1_FRAC}·|ΔE_exact|, {S1_ABS:e}):");
    eprintln!("| node | E_CT | CT fit | miss | tolerance | within |");
    for (g, x) in nodes.iter().enumerate() {
        let miss = (ct_fit_v[g] - e_ct[g]).abs();
        let tol = x.tolerance();
        let ok = miss <= tol;
        s1_ok_v.push(ok);
        if ok {
            s1_within += 1;
        } else {
            s1_misses.push(x.spec.name.clone());
        }
        eprintln!("| {} | {:.6e} | {:.6e} | {miss:.6e} | {tol:.6e} | {ok} |", x.spec.name, e_ct[g], ct_fit_v[g]);
    }
    let s1_raw = if s1_within == n {
        "a"
    } else if s1_within >= s1_b_min {
        "b"
    } else {
        "c"
    };
    let s1_branch = if s1_raw == "a" && at_edge { "b" } else { s1_raw };
    // the twisted dimer of record, named separately (CT-1's 60 % miss)
    let tw = (0..n).find(|&g| nodes[g].spec.name == format!("twisted_R{TWISTED_ANGSTROM:.1}"));
    let (tw_miss, tw_tol, tw_ok, tw_ct) = match tw {
        Some(g) => ((ct_fit_v[g] - e_ct[g]).abs(), nodes[g].tolerance(), s1_ok_v[g], e_ct[g]),
        None => (f64::NAN, f64::NAN, false, f64::NAN),
    };
    eprintln!(
        "S1: {s1_within} of {n} within tolerance (branch (b) needs {s1_b_min}) → branch ({s1_branch}){}{}\n  the twisted dimer of record (twisted_R{TWISTED_ANGSTROM:.1}, CT-1's 60 % miss): measured E_CT {tw_ct:.6e}, fit miss {tw_miss:.6e} against {tw_tol:.6e} → {}",
        if s1_raw == s1_branch { String::new() } else { format!(" (the fit reads ({s1_raw}) on the count, downgraded by the grid edge)") },
        if s1_misses.is_empty() { String::new() } else { format!("; misses: {}", s1_misses.join(", ")) },
        if tw_ok { "within" } else { "MISS" }
    );
    eprintln!("  the bar is the freeze's literal {S1_B_LITERAL} of {FREEZE_TOTAL}; 80 % of the measured total ({eighty} of {n}) is recorded beside it, never in place of it");

    // ================================== C1 — the contacts, fit INSIDE the boundedness gate
    let mut not_two: Vec<String> = Vec::new();
    for x in nodes.iter() {
        let s = engine_dimer(&x.spec.a, &x.spec.b, None, SeamPlant::None);
        if s.seam_work.units != 2 {
            not_two.push(format!("{} (the engine assigns {} units, not 2)", x.spec.name, s.seam_work.units));
        }
    }
    if !not_two.is_empty() {
        eprintln!("REFUSED — the contact terms are fit only on geometries INSIDE the closure identity, and these are not served with two units:");
        for m in &not_two {
            eprintln!("  {m}");
        }
        eprintln!("Nothing written.");
        std::process::exit(4);
    }
    eprintln!("\nevery one of the {n} nodes is served with TWO units by the engine — the contact fit may use them");
    eprintln!("the remainder: ΔE_exact − [E_q(g) − E_q({FAR_BOHR:.0})]_engine − wall9(g) − CT(g), with the angular transfer term held:");
    let mut e_q: Vec<f64> = Vec::with_capacity(n);
    let mut wall_e: Vec<f64> = Vec::with_capacity(n);
    let mut rem: Vec<f64> = Vec::with_capacity(n);
    for (g, x) in nodes.iter().enumerate() {
        let (_, e_q_diff, _) = engine_interaction(&x.spec.a, &x.spec.b, None, SeamPlant::None);
        let wv = a_oo * sum_exp(&x.oo, b_oo) + a_oh * sum_exp(&x.ho, b_oh) + a_hh * sum_exp(&x.hh, b_hh);
        let r = x.de_exact - e_q_diff - wv - ct_fit_v[g];
        eprintln!(
            "  {} ({}): ΔE_exact {:.6e}, E_q diff {e_q_diff:.6e}, wall9 {wv:.6e}, CT {:.6e} → remainder {r:.6e} Ha ({:.4} of |ΔE_exact|)",
            x.spec.name,
            x.spec.kind,
            x.de_exact,
            ct_fit_v[g],
            r / x.de_exact.abs()
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
        for (g, x) in nodes.iter().enumerate() {
            cho[i * n + g] = -sum_exp(&x.ho, c);
            chh[i * n + g] = -sum_exp(&x.hh, c);
        }
    }
    let base = SeamModel { a: a_oo, b: b_oo, p: 0.0, c: 0.0, c6: 0.0, a_oh, b_oh, a_hh, b_hh, p_hh: 0.0, c_hh: 0.0, p_ct, c_ct, m_ct: 0, k_ct: 0, lambda_ct: 0.0, r_cut: 0.0, ct_table_on: false };
    eprintln!(
        "\nthe two-class contact fit UNDER the gate: {} exponent pairs on 0.50..=4.00 step 0.02 per class, weights 1/max(|ΔE_exact|, {WEIGHT_FLOOR:e})², each amplitude CLAMPED at the largest value `SeamModel::bounded(q_H, r_min, kT)` admits (bisection on [0, P_ls] to {CLAMP_REL:e} relative, the two classes iterated to a fixed point, at most {CLAMP_ROUNDS} rounds).\n  held inside the clamp: FIELD-9's wall; the transfer term at its LINEAR value (m = k = 0 inside `bounded`, which is what the engine's own `bounded` reads); the dispersion at an exact 0 — FIELD-6's rule fits C₆ AFTER both contacts, so it is not yet a coefficient of the law when the clamp runs, and G-B0 below is RUN on the full law with C₆ in.",
        NC * NC
    );
    let t_cfit = Instant::now();
    let (mut c_res, mut c_best) = (f64::INFINITY, (0usize, 0usize, 0.0f64, 0.0f64, [false; 2], 1usize));
    let (mut u_res, mut u_best) = (f64::INFINITY, (0usize, 0usize, [0.0f64; 2], [false; 2]));
    let mut clamp_calls: u64 = 0;
    let mut floor_refused: u64 = 0;
    let mut clamped_points: u64 = 0;
    for i in 0..NC {
        let c_ho_i = cgrid(i);
        for j in 0..NC {
            let c_hh_j = cgrid(j);
            let (mut m00, mut m01, mut m11, mut r0v, mut r1v) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
            for g in 0..n {
                let (x0, x1) = (cho[i * n + g], chh[j * n + g]);
                m00 += we[g] * x0 * x0;
                m01 += we[g] * x0 * x1;
                m11 += we[g] * x1 * x1;
                r0v += we[g] * x0 * rem[g];
                r1v += we[g] * x1 * rem[g];
            }
            let (xu, keepu, ru) = fit_nonneg2([[m00, m01], [m01, m11]], [r0v, r1v], syy_c);
            if ru < u_res {
                u_res = ru;
                u_best = (i, j, xu, keepu);
            }
            let mk = |p: f64, q: f64| SeamModel { p, c: c_ho_i, p_hh: q, c_hh: c_hh_j, ..base };
            let (mut ph, mut pj) = (xu[0], xu[1]);
            let (mut clamped, mut rounds, mut floor_ok) = ([false; 2], 1usize, true);
            for round in 1..=CLAMP_ROUNDS {
                let ls_ho = if m00 > 0.0 { ((r0v - m01 * pj) / m00).max(0.0) } else { 0.0 };
                let (nh, ch, k1, f1) = clamp_amplitude(ls_ho, q_h, r_min, &|p| mk(p, pj));
                let ls_hh = if m11 > 0.0 { ((r1v - m01 * nh) / m11).max(0.0) } else { 0.0 };
                let (nj, cj2, k2, f2) = clamp_amplitude(ls_hh, q_h, r_min, &|p| mk(nh, p));
                clamp_calls += (k1 + k2) as u64;
                clamped = [ch, cj2];
                floor_ok = f1 && f2;
                rounds = round;
                let settled = (nh - ph).abs() <= 1e-14 * (1.0 + ph.abs()) && (nj - pj).abs() <= 1e-14 * (1.0 + pj.abs());
                ph = nh;
                pj = nj;
                if settled {
                    break;
                }
            }
            if clamped[0] || clamped[1] {
                clamped_points += 1;
            }
            if !floor_ok {
                floor_refused += 1;
            }
            let mut res = 0.0f64;
            for g in 0..n {
                let mv = ph * cho[i * n + g] + pj * chh[j * n + g];
                let d = rem[g] - mv;
                res += we[g] * d * d;
            }
            if res < c_res {
                c_res = res;
                c_best = (i, j, ph, pj, clamped, rounds);
            }
        }
    }
    let (ci, cj, p_ho, p_hh, c_clamped, c_rounds) = c_best;
    let (c_ho, c_hh) = (cgrid(ci), cgrid(cj));
    let (ui, uj, uamp, ukeep) = u_best;
    let (u_c_ho, u_c_hh) = (cgrid(ui), cgrid(uj));
    let cfit_seconds = t_cfit.elapsed().as_secs_f64();
    let ratio = if u_res > 0.0 { c_res / u_res } else { f64::NAN };
    let edge_c_ho = ci == 0 || ci == NC - 1;
    let edge_c_hh = cj == 0 || cj == NC - 1;
    eprintln!(
        "  CONSTRAINED: P_HO = {p_ho:.9e} Ha, c_HO = {c_ho:.2} /bohr (clamped {}); P_HH = {p_hh:.9e} Ha, c_HH = {c_hh:.2} /bohr (clamped {}); {c_rounds} round(s) to the fixed point; weighted residual {c_res:.9e}",
        c_clamped[0], c_clamped[1]
    );
    eprintln!(
        "  UNCONSTRAINED (recorded beside it, NEVER the law): P_HO = {:.9e}, c_HO = {u_c_ho:.2} (kept {}); P_HH = {:.9e}, c_HH = {u_c_hh:.2} (kept {}); weighted residual {u_res:.9e}",
        uamp[0], ukeep[0], uamp[1], ukeep[1]
    );
    eprintln!(
        "  the price of the clamp: constrained/unconstrained residual ratio {ratio:.6} — {}",
        if ratio > RATIO_NAMED { format!("ABOVE {RATIO_NAMED}: the remainder after charge transfer wants a shape a bounded exponential cannot take, NAMED (§3), not a failure") } else { format!("at or under {RATIO_NAMED}: the gate costs the fit little") }
    );
    eprintln!(
        "  grid edges: c_HO at an edge {edge_c_ho}, c_HH at an edge {edge_c_hh}; {clamped_points} of {} grid points had at least one amplitude clamped, {clamp_calls} boundedness walks in {cfit_seconds:.1} s{}",
        NC * NC,
        if floor_refused > 0 { format!("; {floor_refused} grid points where even a zero amplitude is refused by the gate (the fixed terms alone are unbounded there)") } else { String::new() }
    );

    let contact = |ho: &[f64], hh: &[f64]| -> f64 { -p_ho * sum_exp(ho, c_ho) - p_hh * sum_exp(hh, c_hh) };
    let mut c1_within = 0usize;
    let mut c1_line_within = 0usize;
    let mut c1_line_of = 0usize;
    let mut c1_misses: Vec<String> = Vec::new();
    let mut contact_v: Vec<f64> = Vec::with_capacity(n);
    let mut c1_ok_v: Vec<bool> = Vec::with_capacity(n);
    eprintln!("\nC1 — the constrained contact fit against the remainder, tolerance max({S1_FRAC}·|ΔE_exact|, {S1_ABS:e}):");
    for (g, x) in nodes.iter().enumerate() {
        let f = contact(&x.ho, &x.hh);
        contact_v.push(f);
        let miss = (rem[g] - f).abs();
        let t = x.tolerance();
        let ok = miss <= t;
        c1_ok_v.push(ok);
        if ok {
            c1_within += 1;
        } else {
            c1_misses.push(x.spec.name.clone());
        }
        if x.spec.line_node() {
            c1_line_of += 1;
            if ok {
                c1_line_within += 1;
            }
        }
        eprintln!("  {}: remainder {:.6e}, fit {f:.6e}, miss {miss:.6e} ({:.4} of its tolerance {t:.3e}) → {}", x.spec.name, rem[g], miss / t, if ok { "within" } else { "MISS" });
    }
    let c1_line_ok = c1_line_within == c1_line_of && c1_line_of == LINE_ANGSTROM.len();
    let c1 = c1_within >= c1_min && c1_line_ok;
    eprintln!(
        "C1: {} — {c1_within} of {n} within tolerance (needs ≥ {c1_min}, 80 % of the measured total), and the line at 2.7, 2.9, 3.1 Å all within: {c1_line_ok} ({c1_line_within} of {c1_line_of}){}",
        if c1 { "PASS" } else { "FAIL" },
        if c1_misses.is_empty() { String::new() } else { format!("; misses: {}", c1_misses.join(", ")) }
    );

    // ------------------- dispersion: what is left on the OUTER LINEAR nodes only (FIELD-6's rule)
    let outer: Vec<usize> = (0..n).filter(|&g| nodes[g].spec.outer_linear()).collect();
    let rem2: Vec<f64> = outer.iter().map(|&g| rem[g] - contact_v[g]).collect();
    let (mut num6, mut den6) = (0.0f64, 0.0f64);
    for (oi, &g) in outer.iter().enumerate() {
        let x = -1.0 / cross_oo(&nodes[g].spec.a, &nodes[g].spec.b).powi(6);
        num6 += we[g] * rem2[oi] * x;
        den6 += we[g] * x * x;
    }
    let mut c6 = if den6 > 0.0 { num6 / den6 } else { 0.0 };
    let mut slopes: Vec<(f64, f64)> = Vec::new();
    eprintln!("\ndispersion (FIELD-6's rule, after BOTH contacts) on the {} outer linear nodes (R_OO ≥ {OUTER_FROM_ANGSTROM} Å):", outer.len());
    for (oi, &g) in outer.iter().enumerate() {
        eprintln!("  {:.1} Å: remainder after contact {:.6e} Ha ({:.4} of |ΔE_exact|)", nodes[g].spec.r_ang, rem2[oi], rem2[oi] / nodes[g].de_exact.abs());
    }
    for oi in 1..outer.len() {
        let (a1v, a0v) = (rem2[oi], rem2[oi - 1]);
        let (r1v, r0v) = (cross_oo(&nodes[outer[oi]].spec.a, &nodes[outer[oi]].spec.b), cross_oo(&nodes[outer[oi - 1]].spec.a, &nodes[outer[oi - 1]].spec.b));
        let s = if a1v != 0.0 && a0v != 0.0 { (a1v.abs() / a0v.abs()).ln() / (r1v / r0v).ln() } else { f64::NAN };
        eprintln!("  log-log slope {:.1} → {:.1} Å: {s:.3}", nodes[outer[oi - 1]].spec.r_ang, nodes[outer[oi]].spec.r_ang);
        slopes.push((nodes[outer[oi]].spec.r_ang, s));
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
    let model = SeamModel {
        a: a_oo,
        b: b_oo,
        p: p_ho,
        c: c_ho,
        c6,
        a_oh,
        b_oh,
        a_hh,
        b_hh,
        p_hh,
        c_hh,
        p_ct,
        c_ct,
        m_ct: m_ct as u8,
        k_ct: k_ct as u8,
        lambda_ct,
        r_cut: 0.0,
        ct_table_on: false,
    };
    let bounded = model.bounded(q_h, r_min, KT);
    let g_b0 = bounded.is_none();
    let arms_void = bounded.is_some();
    let hole = model.hole(q_h);
    eprintln!(
        "\nG-B0 — RUN on the constrained law even though the clamp makes it pass by construction (M-VACUOUS-SUCCESS): `bounded` walks each cross-unit class of the FULL law from its own r_min (O–O {r_min_oo:.6}, H–O {r_min_oh:.6}, H–H {r_min_hh:.6} bohr) inward to 0.5 bohr on a 0.05 grid, kT = {KT:e} Ha, q_H = {q_h:.12e}"
    );
    match &bounded {
        None => eprintln!("  bounded() returned none → PASS: no class falls more than kT below its fit floor, and each is positive at contact; the arms may run"),
        Some(msg) => eprintln!("  bounded() named a fall, VERBATIM: {msg}\n  → the constrained fit is broken and the campaign stops here (§2 G-B0); the arms are VOID. The harvest is still recorded in full."),
    }
    match &hole {
        None => eprintln!("  hole() (a READING beside the gate, not a gate): none — the law also rises monotonically inward from 3.0 bohr"),
        Some(msg) => eprintln!("  hole() (a READING beside the gate, not a gate) names a fall, verbatim: {msg}"),
    }

    // ------------------------------------------------- G-C1, and the two plants
    eprintln!("\nG-C1 — the engine's arithmetic against THIS runner's independent evaluation of §0's equations (the formula side never calls SeamModel::ct_angular), ONE reference (E_q(g) − E_q({FAR_BOHR:.0}) from the engine itself), on every node:");
    let mut g_c1_worst = 0.0f64;
    let mut units_ok = true;
    let mut g_c1_v: Vec<(f64, f64, f64, u64, u64, u64)> = Vec::with_capacity(n);
    let mut ct_term_v: Vec<f64> = Vec::with_capacity(n);
    let mut plant_ii = (f64::NAN, f64::NAN, f64::NAN, f64::NAN, false);
    let refi = (0..n).find(|&g| nodes[g].spec.linear_node() && (nodes[g].spec.r_ang - REF_ANGSTROM).abs() < 1e-9);
    for (g, x) in nodes.iter().enumerate() {
        let (e_int, e_field_diff, e_seam_diff) = engine_interaction(&x.spec.a, &x.spec.b, Some(model), SeamPlant::None);
        let t = formula_terms(&x.spec.a, &x.spec.b, &model);
        let want = e_field_diff + t.total();
        let miss = (e_int - want).abs();
        g_c1_worst = g_c1_worst.max(miss);
        ct_term_v.push(t.ct);
        let s = engine_dimer(&x.spec.a, &x.spec.b, Some(model), SeamPlant::None);
        let (units, oo_pairs, ho_pairs) = (s.seam_work.units, s.seam_work.oo_pairs, s.seam_work.ho_pairs);
        drop(s);
        if units != 2 || oo_pairs != 1 || ho_pairs != 4 {
            units_ok = false;
        }
        eprintln!(
            "  {}: engine {e_int:.12e} vs formula {want:.12e} — miss {miss:.3e} (E_q diff {e_field_diff:.6e}, seam {e_seam_diff:.6e}; pen_HO {:.6e}, pen_HH {:.6e}, wall_OO {:.6e}, wall_OH {:.6e}, wall_HH {:.6e}, disp {:.6e}, CT {:.6e}; units {units}, O–O {oo_pairs}, H–O {ho_pairs})",
            x.spec.name, t.pen_ho, t.pen_hh, t.w_oo, t.w_oh, t.w_hh, t.disp, t.ct
        );
        g_c1_v.push((e_int, want, miss, units, oo_pairs, ho_pairs));
        if Some(g) == refi {
            let (e_pl, e_field_pl, _) = engine_interaction(&x.spec.a, &x.spec.b, Some(model), SeamPlant::FlipChargeTransfer);
            let observed = (e_pl - (e_field_pl + t.total())).abs();
            let engine_to_engine = (e_pl - e_int).abs();
            let expected = 2.0 * t.ct.abs();
            let carrier = t.ct.abs();
            let fires = carrier >= PLANT_II_CARRIER && (observed - expected).abs() <= G_C1_TOL;
            plant_ii = (observed, expected, carrier, engine_to_engine, fires);
            eprintln!(
                "plant (ii) at the linear {REF_ANGSTROM:.1} Å node (P_CT → −P_CT in the engine): the G-C1 miss under the plant {observed:.6e} vs 2·|CT({REF_ANGSTROM:.1})| {expected:.6e} (difference {:.3e}, bar {G_C1_TOL:e}); engine-to-engine {engine_to_engine:.6e}; carrier |CT| {carrier:.3e} ≥ {PLANT_II_CARRIER:e}: {} → {}",
                (observed - expected).abs(),
                carrier >= PLANT_II_CARRIER,
                if fires { "FIRES" } else { "does not fire" }
            );
        }
    }
    let g_c1 = g_c1_worst <= G_C1_TOL;
    eprintln!("G-C1: worst |engine − formula| = {g_c1_worst:.3e} (stake {G_C1_TOL:e}) → {}", if g_c1 { "PASS" } else { "FAIL" });
    eprintln!("M-VACUOUS-SUCCESS: every G-C1 geometry served two units, one cross O–O pair and four cross H–O pairs: {units_ok}");

    // plant (i): the angle removed — CT-1's term, CT-1's coefficients, on the twisted node
    let (pi_ct1, pi_miss, pi_fit_miss, pi_fires) = match tw {
        Some(g) => {
            let c0 = ct_value(&nodes[g].pairs, CT1_P, CT1_C, 0, 0, 0.0);
            let miss = (e_ct[g] - c0).abs();
            let fit_miss = (ct_fit_v[g] - e_ct[g]).abs();
            let fires = miss >= PLANT_I_MIN_MISS && miss >= PLANT_I_CARRIER;
            eprintln!(
                "\nplant (i) — the angle removed: CT-1's term (m = k = 0, P = {CT1_P}, c = {CT1_C}) on twisted_R{TWISTED_ANGSTROM:.1} reads {c0:.6e} against the measured E_CT {:.6e} — miss {miss:.6e}, which must be at least {PLANT_I_MIN_MISS:e}; carrier |E_CT − CT_(0,0)| {miss:.3e} ≥ {PLANT_I_CARRIER:e}: {}; the FITTED family's miss beside it: {fit_miss:.6e} → {}",
                e_ct[g],
                miss >= PLANT_I_CARRIER,
                if fires { "FIRES" } else { "does not fire" }
            );
            (c0, miss, fit_miss, fires)
        }
        None => {
            eprintln!("\nplant (i): the twisted node of record is not in the node set — the plant cannot be read");
            (f64::NAN, f64::NAN, f64::NAN, false)
        }
    };

    // ------------------------------------------------------------------------ wall_ct2.json
    let mut node_lines: Vec<String> = Vec::with_capacity(n);
    for (g, x) in nodes.iter().enumerate() {
        let s = &x.sector;
        let (e_int, want, miss, units, oo_pairs, ho_pairs) = g_c1_v[g];
        node_lines.push(format!(
            "{{\"node\": \"{}\", \"family\": \"{}\", \"kind\": \"{}\", \"new\": {}, \"r_angstrom\": {:.1}, \"r_oo_bohr\": {}, \"twist_degrees\": {:.1}, \"tilt_degrees\": {:.1}, \"donor_bend_degrees\": {:.1}, \"line_node\": {}, \"outer_linear\": {}, \"exact_source\": \"{}\", \"sector_source\": \"{}\", \"e_exact_total\": {}, \"e_noct\": {}, \"e_hl_undeformed\": {}, \"e_a0\": {}, \"e_b0\": {}, \"de_exact\": {}, \"e_ct\": {}, \"delta_e_hl\": {}, \"closed_sector_gain\": {}, \"ct_fit\": {}, \"ct_miss\": {}, \"tolerance\": {}, \"ct_within\": {}, \"e_q_difference\": {}, \"wall9\": {}, \"remainder\": {}, \"contact_fit\": {}, \"c1_miss\": {}, \"c1_within\": {}, \"g_c1_engine\": {}, \"g_c1_formula\": {}, \"g_c1_miss\": {}, \"g_c1_ct_term\": {}, \"units\": {units}, \"oo_pairs\": {oo_pairs}, \"ho_pairs\": {ho_pairs}, \"min_cross_ho_bohr\": {}, \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}]}}",
            x.spec.name,
            x.spec.family,
            esc(&x.spec.kind),
            x.is_new,
            x.spec.r_ang,
            jd(cross_oo(&x.spec.a, &x.spec.b)),
            x.spec.twist_deg,
            x.spec.tilt_deg,
            x.spec.donor_deg,
            x.spec.line_node(),
            x.spec.outer_linear(),
            esc(&x.exact_source),
            esc(&s.source),
            jn(x.e_exact),
            jn(s.e_noct),
            jn(s.e_hl_undeformed),
            jn(s.e_a0),
            jn(s.e_b0),
            jn(x.de_exact),
            jn(e_ct[g]),
            jn(s.e_hl_undeformed - (s.e_a0 + s.e_b0)),
            jn(s.e_noct - s.e_hl_undeformed),
            jn(ct_fit_v[g]),
            jn((ct_fit_v[g] - e_ct[g]).abs()),
            jn(x.tolerance()),
            s1_ok_v[g],
            jn(e_q[g]),
            jn(wall_e[g]),
            jn(rem[g]),
            jn(contact_v[g]),
            jn((rem[g] - contact_v[g]).abs()),
            c1_ok_v[g],
            jn(e_int),
            jn(want),
            jn(miss),
            jn(ct_term_v[g]),
            jd(x.min_ho()),
            list_json(&x.oo),
            list_json(&x.ho),
            list_json(&x.hh),
        ));
    }
    let slope_lines: Vec<String> = slopes.iter().map(|(r, s)| format!("{{\"r_angstrom\": {r:.1}, \"loglog_slope_from_previous\": {}}}", jn(*s))).collect();
    let quoted = |v: &[String]| -> String { v.iter().map(|m| format!("\"{}\"", esc(m))).collect::<Vec<_>>().join(", ") };
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
        "  \"a\": {}, \"b\": {}, \"p\": {}, \"c\": {}, \"c6\": {}, \"a_oh\": {}, \"b_oh\": {}, \"a_hh\": {}, \"b_hh\": {}, \"p_hh\": {}, \"c_hh\": {}, \"p_ct\": {}, \"c_ct\": {}, \"m_ct\": {m_ct}, \"k_ct\": {k_ct}, \"lambda_ct\": {}, \"lambda_ct_degrees\": {},\n",
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
        jn(lambda_ct),
        jn(lambda_deg)
    ));
    j.push_str(&format!(
        "  \"lambda_ct_units\": \"radians (lambda_ct_degrees is the same angle in degrees)\", \"lambda_ct_matters\": {lambda_matters},\n  \"r_min_oo\": {}, \"r_min_oh\": {}, \"r_min_hh\": {}, \"q_h\": {}, \"q_h_record\": {}, \"kt\": {},\n  \"wall_source\": \"{}\", \"ct1_source\": \"{}\", \"wall_rule\": \"FIELD-9's three-class wall is HELD; the transfer term, the two contact terms and the dispersion are this harvest's\",\n",
        jn(r_min_oo),
        jn(r_min_oh),
        jn(r_min_hh),
        jn(q_h),
        jn(q_h_record),
        jn(KT),
        esc(&wall9_path.display().to_string()),
        esc(&wall_ct1_path.display().to_string())
    ));
    j.push_str(&format!(
        "  \"node_count\": {{\"measured_total\": {n}, \"of_record\": {n_record}, \"new\": {n_new}, \"freeze_total\": {FREEZE_TOTAL}, \"freeze_new\": {FREEZE_NEW}, \"outside_the_identity\": [{}], \"duplicates_of_record\": [{}], \"note\": \"the freeze (§0) stakes {FREEZE_TOTAL} distinct nodes and {FREEZE_NEW} new solves; its family count column sums to 67 because three geometries are shared with the record. Every duplicate named here is a candidate of the freeze's table that reproduces a node of record to 1e-9 bohr: it is not solved again and is counted ONCE, under the record's name. The measured total above is what every fit and gate actually read.\", \"naming\": \"{}\"}},\n",
        quoted(&outside),
        quoted(&duplicate),
        esc(NAMING)
    ));
    j.push_str(&format!(
        "  \"g_d0\": {{\"pass\": {g_d0}, \"rule\": \"|E_full − E_record| ≤ {GD0_TOL:e} on the {GD0_NODES} exact nodes of record, exit Converged, residual ≤ {SECTOR_RESIDUAL_BAR:e}\", \"within\": {gd0_within}, \"of\": {GD0_NODES}, \"records_read\": {gd0_files}, \"misses\": [{}], \"per_node\": [{}]}},\n",
        quoted(&gd0_misses),
        gd0_lines.join(", ")
    ));
    j.push_str(&format!(
        "  \"t0\": {{\"pass\": {t0}, \"rule\": \"on every NEW node: residual ≤ {SECTOR_RESIDUAL_BAR:e}, exit Converged, subspace dimension {SECTOR_DIM_STAKED} EXACT, E_exact ≤ E_noCT ≤ E_HL(undeformed) on TOTAL energies within {ORDER_TOL:e}\", \"new_nodes\": {n_new}, \"failures\": [{}], \"price\": {{\"rule\": \"cpu_seconds ≥ {PRICE_TENTH} · davidson_iters · {SIGMA_PRICE_FLOOR} core-seconds (M-CHEAPER-THAN-ITS-PRICE)\", \"all_admitted\": {}, \"refused\": [{}]}}, \"per_node\": [{}]}},\n",
        quoted(&t0_fails),
        price_refused.is_empty(),
        quoted(&price_refused),
        t0_lines.join(", ")
    ));
    j.push_str(&format!(
        "  \"a1\": {{\"pass\": {a1}, \"rule\": \"E_CT < −{CT_FLOOR:e} at every node whose shortest cross-unit H···O is ≤ {A1_CONTACT_BOHR} bohr; a node beyond that reads a NULL, not a miss\", \"nodes_inside\": {a1_in}, \"attractive_inside\": {}, \"misses\": [{}], \"nulls_beyond\": [{}], \"tables\": {{{}}}}},\n",
        a1_in - a1_misses.len(),
        quoted(&a1_misses),
        quoted(&a1_nulls),
        table_json.join(", ")
    ));
    j.push_str(&format!(
        "  \"p1\": {{\"pass\": {p1}, \"rule\": \"the fit restricted to m = k = 0 on CT-1's twelve reproduces CT-1's P_CT and c_CT to {P1_TOL:e}\", \"p\": {}, \"c\": {}, \"p_ct1\": {CT1_P}, \"c_ct1\": {CT1_C}, \"p_difference\": {}, \"c_difference\": {}, \"tolerance\": {P1_TOL:e}, \"p_ct1_record\": {}, \"c_ct1_record\": {}, \"nodes\": {}, \"weighted_residual\": {}}},\n",
        jn(p1_p),
        jn(p1_c),
        jn(p1_dp),
        jn(p1_dc),
        jn(p_ct1_rec),
        jn(c_ct1_rec),
        ct1_twelve.len(),
        jn(p1_res)
    ));
    j.push_str(&format!(
        "  \"s1\": {{\"branch\": \"{s1_branch}\", \"branch_on_the_count\": \"{s1_raw}\", \"within\": {s1_within}, \"of\": {n}, \"branch_b_minimum\": {s1_b_min}, \"branch_b_minimum_rule\": \"the freeze's literal: at least 80 % (52 of 64)\", \"freeze_total\": {FREEZE_TOTAL}, \"eighty_percent_of_the_measured_total\": {eighty}, \"measured_total\": {n}, \"tolerance_rule\": \"max({S1_FRAC}·|ΔE_exact|, {S1_ABS:e})\", \"misses\": [{}], \"grid_edge\": {{\"at_edge\": {at_edge}, \"m\": {edge_m}, \"k\": {edge_k}, \"lambda\": {edge_lambda}, \"lambda_matters\": {lambda_matters}, \"rule\": \"any of m, k, λ at an edge of its grid is reported as the grid's number and the fit is read as (b) at best (§0)\"}}, \"twisted_node\": {{\"node\": \"twisted_R{TWISTED_ANGSTROM:.1}\", \"e_ct\": {}, \"miss\": {}, \"tolerance\": {}, \"within\": {tw_ok}}}}},\n",
        quoted(&s1_misses),
        jn(tw_ct),
        jn(tw_miss),
        jn(tw_tol)
    ));
    j.push_str(&format!(
        "  \"ct_fit\": {{\"p\": {}, \"c\": {}, \"m\": {m_ct}, \"k\": {k_ct}, \"lambda_radians\": {}, \"lambda_degrees\": {}, \"p_unclamped\": {}, \"transferred\": {ct_transferred}, \"placement\": \"every cross-unit H–O pair, the acceptor's frame from ITS unit's two hydrogens\", \"shape\": \"−P·Σ exp(−c·r)·((1−cos θ_d)/2)^m·[((1+u·l₊)/2)^k + ((1+u·l₋)/2)^k]/[2((1+cos λ)/2)^k]\", \"grid\": {{\"c\": \"0.50..=4.00 per bohr, step 0.02\", \"c_points\": {NC}, \"m\": {M_GRID:?}, \"k\": {K_GRID:?}, \"lambda_degrees\": {LAM_DEG:?}, \"points\": {}}}, \"tie_rule\": \"the first grid point of least weighted residual in the order m, k, λ, c ascending\", \"weights\": \"1/max(|ΔE_exact|, {WEIGHT_FLOOR:e})²\", \"weight_floor\": {WEIGHT_FLOOR:e}, \"nonnegativity\": \"P below zero is clamped to an exact 0.0 and the term is NOT transferred\", \"weighted_residual\": {}, \"weighted_total\": {}, \"nodes\": {n}, \"cross_ho_pairs\": {np}, \"fit_seconds\": {}}},\n",
        jn(p_ct),
        jn(c_ct),
        jn(lambda_ct),
        jn(lambda_deg),
        jn(p_ct_raw),
        NC * M_GRID.len() * K_GRID.len() * LAM_DEG.len(),
        jn(best_res),
        jn(syy_ct),
        jd(fit_seconds)
    ));
    j.push_str(&format!(
        "  \"c1\": {{\"pass\": {c1}, \"within\": {c1_within}, \"of\": {n}, \"required\": {c1_min}, \"required_rule\": \"80 % of the measured total\", \"line_within\": {c1_line_within}, \"line_nodes\": {c1_line_of}, \"line_ok\": {c1_line_ok}, \"line_rule\": \"the linear nodes at 2.7, 2.9 and 3.1 Å must ALL be within\", \"tolerance_rule\": \"max({S1_FRAC}·|ΔE_exact|, {S1_ABS:e})\", \"misses\": [{}], \"residual_ratio\": {}, \"residual_ratio_named_above\": {RATIO_NAMED}, \"residual_ratio_reading\": \"{}\"}},\n",
        quoted(&c1_misses),
        jn(ratio),
        if ratio > RATIO_NAMED { "above the naming threshold: the remainder after charge transfer wants a shape a bounded exponential cannot take (§3) — named, not a failure" } else { "at or under the naming threshold: the gate costs the fit little" }
    ));
    j.push_str(&format!(
        "  \"contact_fit\": {{\"constrained\": {{\"p_ho\": {}, \"c_ho\": {}, \"p_hh\": {}, \"c_hh\": {}, \"clamped\": {{\"ho\": {}, \"hh\": {}}}, \"fixed_point_rounds\": {c_rounds}, \"weighted_residual\": {}, \"grid_edge\": {{\"c_ho\": {edge_c_ho}, \"c_hh\": {edge_c_hh}}}}}, \"unconstrained\": {{\"p_ho\": {}, \"c_ho\": {}, \"p_hh\": {}, \"c_hh\": {}, \"classes_kept\": {{\"ho\": {}, \"hh\": {}}}, \"weighted_residual\": {}}}, \"residual_ratio\": {}, \"clamp\": {{\"rule\": \"for each grid exponent pair each class's amplitude is the largest `SeamModel::bounded(q_H, r_min, kT)` admits, by bisection on [0, P_ls] to {CLAMP_REL:e} relative, the two classes iterated to a fixed point (at most {CLAMP_ROUNDS} rounds)\", \"held_inside_the_clamp\": \"FIELD-9's wall; the transfer term at its LINEAR value (m = k = 0, which is what `bounded` reads); the dispersion at an exact 0, because FIELD-6's rule fits C₆ after both contacts\", \"grid_points_with_a_clamp\": {clamped_points}, \"grid_points\": {}, \"boundedness_walks\": {clamp_calls}, \"grid_points_refused_at_zero\": {floor_refused}, \"fit_seconds\": {}}}, \"placement\": \"cross-unit H–O and cross-unit H–H\", \"weights\": \"1/max(|ΔE_exact|, {WEIGHT_FLOOR:e})²\", \"remainder\": \"ΔE_exact − [E_q(g) − E_q({FAR_BOHR:.0})]_engine − wall9(g) − CT(g)\", \"points\": {n}, \"units_two_on_every_point\": true}},\n",
        jn(p_ho),
        jn(c_ho),
        jn(p_hh),
        jn(c_hh),
        c_clamped[0],
        c_clamped[1],
        jn(c_res),
        jn(uamp[0]),
        jn(u_c_ho),
        jn(uamp[1]),
        jn(u_c_hh),
        ukeep[0],
        ukeep[1],
        jn(u_res),
        jn(ratio),
        NC * NC,
        jd(cfit_seconds)
    ));
    j.push_str(&format!(
        "  \"dispersion\": {{\"nodes\": \"the {} outer linear nodes (R_OO ≥ {OUTER_FROM_ANGSTROM} Å), after both contact terms and the transfer term\", \"c6\": {}, \"transferred\": {c6_transferred}, \"slope_band\": [{SLOPE_LO}, {SLOPE_HI}], \"slopes\": [{}]}},\n",
        outer.len(),
        jn(c6),
        slope_lines.join(", ")
    ));
    j.push_str(&format!(
        "  \"g_b0\": {{\"pass\": {g_b0}, \"bounded\": {bounded_json}, \"run_even_though_it_passes_by_construction\": true, \"q_h\": {}, \"q_h_source\": \"holon_render::field::water_charge_at_pin\", \"kt\": {}, \"r_min\": [{}, {}, {}], \"r_min_order\": \"O–O, H–O, H–H\", \"r_min_source\": \"{}\", \"verdict\": \"{}\", \"hole_reading\": {hole_json}, \"hole_is_a_reading_not_a_gate\": true}},\n",
        jn(q_h),
        jn(KT),
        jn(r_min_oo),
        jn(r_min_oh),
        jn(r_min_hh),
        esc(&wall9_path.display().to_string()),
        if g_b0 { "bounded — the arms may run" } else { "a fall named — the constrained fit is broken and the arms are VOID (§2 G-B0)" }
    ));
    j.push_str(&format!(
        "  \"g_c1\": {{\"pass\": {g_c1}, \"worst_miss\": {}, \"tolerance\": {G_C1_TOL:e}, \"reference\": \"E_q(g) − E_q({FAR_BOHR:.0} bohr), the engine's own field on both sides\", \"formula_side\": \"this runner's own evaluation of CT2_PREREG §0's equations; it never calls SeamModel::ct_angular\", \"points\": {n}, \"units_and_pair_counts_ok\": {units_ok}}},\n",
        jn(g_c1_worst)
    ));
    j.push_str(&format!(
        "  \"plant_i\": {{\"fires\": {pi_fires}, \"plant\": \"(i) the angle removed — the family at m = k = 0 with CT-1's coefficients\", \"node\": \"twisted_R{TWISTED_ANGSTROM:.1}\", \"ct_without_the_angle\": {}, \"e_ct_measured\": {}, \"miss\": {}, \"miss_required\": {PLANT_I_MIN_MISS:e}, \"carrier\": {}, \"carrier_floor\": {PLANT_I_CARRIER:e}, \"carrier_present\": {}, \"fitted_family_miss\": {}}},\n",
        jn(pi_ct1),
        jn(tw_ct),
        jn(pi_miss),
        jn(pi_miss),
        pi_miss >= PLANT_I_CARRIER,
        jn(pi_fit_miss)
    ));
    j.push_str(&format!(
        "  \"plant_ii\": {{\"fires\": {}, \"plant\": \"(ii) P_CT → −P_CT in the engine (SeamPlant::FlipChargeTransfer)\", \"node\": \"linear_R{REF_ANGSTROM:.1}\", \"miss_observed\": {}, \"miss_expected\": {}, \"difference\": {}, \"tolerance\": {G_C1_TOL:e}, \"engine_to_engine\": {}, \"carrier_ct_term\": {}, \"carrier_floor\": {PLANT_II_CARRIER:e}, \"carrier_present\": {}}},\n",
        plant_ii.4,
        jn(plant_ii.0),
        jn(plant_ii.1),
        jn((plant_ii.0 - plant_ii.1).abs()),
        jn(plant_ii.3),
        jn(plant_ii.2),
        plant_ii.2 >= PLANT_II_CARRIER
    ));
    j.push_str(&format!("  \"arms_void\": {arms_void},\n"));
    j.push_str(&format!("  \"nodes\": [\n    {}\n  ]\n}}\n", node_lines.join(",\n    ")));
    fs::write(out.join("wall_ct2.json"), j).unwrap();
    eprintln!("\nwall_ct2.json written — arms_void {arms_void}");

    // ------------------------------------ prediction.json, BEFORE the held-out solve
    let (a_p, b_p) = twist_and_bend(o, h, PRED_ANGSTROM, PRED_TWIST_DEGREES, PRED_TILT_DEGREES, PRED_DONOR_DEGREES);
    let r_oo_p = cross_oo(&a_p, &b_p);
    assert!(
        (r_oo_p - PRED_ANGSTROM * ANGSTROM_TO_BOHR).abs() < 1e-9,
        "every turn pivots on an oxygen: R_OO must be unchanged ({r_oo_p:.9} vs {:.9})",
        PRED_ANGSTROM * ANGSTROM_TO_BOHR
    );
    let (e_pred, e_q_p, e_seam_p) = engine_interaction(&a_p, &b_p, Some(model), SeamPlant::None);
    let tp = formula_terms(&a_p, &b_p, &model);
    let s_p = engine_dimer(&a_p, &b_p, Some(model), SeamPlant::None);
    let (units_p, oo_pairs_p, ho_pairs_p) = (s_p.seam_work.units, s_p.seam_work.oo_pairs, s_p.seam_work.ho_pairs);
    drop(s_p);
    let (oo_p, ho_p, hh_p) = cross_classes(&a_p, &b_p);
    fs::write(
        out.join("prediction.json"),
        format!(
            "{{\n  \"node\": \"{PRED_NAME}\", \"r_oo_angstrom\": {PRED_ANGSTROM:.3}, \"r_oo_bohr\": {}, \"acceptor_twist_degrees\": {PRED_TWIST_DEGREES:.1}, \"acceptor_tilt_degrees\": {PRED_TILT_DEGREES:.1}, \"donor_bend_degrees\": {PRED_DONOR_DEGREES:.1},\n  \"geometry\": \"twisted({PRED_ANGSTROM:.1} Å, {PRED_TWIST_DEGREES:.0}°, {PRED_TILT_DEGREES:.0}°) with the DONOR bent {PRED_DONOR_DEGREES:.0}° about the x-axis through its own oxygen — a twist, a tilt and a bend at once, at a distance and angles no fit point has; every pivot is an oxygen, so R_OO is unchanged\", \"held_out\": true,\n  \"e_pred\": {},\n  \"parts\": {{\"e_q_difference\": {}, \"contact_ho\": {}, \"contact_hh\": {}, \"wall_oo\": {}, \"wall_oh\": {}, \"wall_hh\": {}, \"wall_total\": {}, \"disp\": {}, \"ct_term\": {}, \"engine_seam\": {}}},\n  \"ct_term_rule\": \"the angular family of §0 at the fitted P, c, m, k, λ, summed over this geometry's four cross-unit H–O pairs\",\n  \"coefficients\": {{\"a\": {}, \"b\": {}, \"p\": {}, \"c\": {}, \"c6\": {}, \"a_oh\": {}, \"b_oh\": {}, \"a_hh\": {}, \"b_hh\": {}, \"p_hh\": {}, \"c_hh\": {}, \"p_ct\": {}, \"c_ct\": {}, \"m_ct\": {m_ct}, \"k_ct\": {k_ct}, \"lambda_ct\": {}, \"lambda_ct_degrees\": {}}},\n  \"s1_branch\": \"{s1_branch}\", \"c1_pass\": {c1}, \"g_d0_pass\": {g_d0}, \"t0_pass\": {t0}, \"a1_pass\": {a1}, \"p1_pass\": {p1}, \"g_b0_pass\": {g_b0}, \"g_c1_pass\": {g_c1}, \"ct_transferred\": {ct_transferred}, \"c6_transferred\": {c6_transferred}, \"arms_void\": {arms_void}, \"units\": {units_p}, \"oo_pairs\": {oo_pairs_p}, \"ho_pairs\": {ho_pairs_p},\n  \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\", \"tolerance_frac\": {PRED_FRAC}, \"tolerance_abs\": {PRED_ABS:e},\n  \"ct_tolerance_rule\": \"branch (b): |CT_term − E_CT| ≤ max({PRED_CT_FRAC}·|E_CT|, {PRED_CT_ABS:e})\", \"ct_tolerance_frac\": {PRED_CT_FRAC}, \"ct_tolerance_abs\": {PRED_CT_ABS:e},\n  \"exact_solve_stake\": {{\"solver\": \"holon_chem::heitler_london::fci_full_from_product (G-D0's solver)\", \"n_det\": {N_DET_DIMER}, \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}, \"exit\": \"Converged\"}},\n  \"sector_solve_stake\": {{\"sector_dim\": {SECTOR_DIM_STAKED}, \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}}},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}], \"min_cross_ho_bohr\": {},\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
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
            jn(lambda_ct),
            jn(lambda_deg),
            list_json(&oo_p),
            list_json(&ho_p),
            list_json(&hh_p),
            jd(min_or_nan(&ho_p)),
            centers_json(&a_p),
            centers_json(&b_p),
        ),
    )
    .unwrap();
    eprintln!(
        "prediction.json filed BEFORE the held-out solve: {PRED_NAME} at R_OO {r_oo_p:.6} bohr (shortest cross H···O {:.4}) — E_pred {e_pred:.6e} Ha; E_q diff {e_q_p:.6e}, contact_HO {:.6e}, contact_HH {:.6e}, wall_OO {:.6e}, wall_OH {:.6e}, wall_HH {:.6e}, disp {:.6e}, CT {:.6e}; units {units_p}, O–O {oo_pairs_p}, H–O {ho_pairs_p}",
        min_or_nan(&ho_p),
        tp.pen_ho,
        tp.pen_hh,
        tp.w_oo,
        tp.w_oh,
        tp.w_hh,
        tp.disp,
        tp.ct
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
    let wall_path = out.join("wall_ct2.json");
    let wall = fs::read_to_string(&wall_path).expect("wall_ct2.json: run `fit` first");
    let (p_ct, c_ct) = (json_num(&wall, "p_ct"), json_num(&wall, "c_ct"));
    let (m_ct, k_ct, lambda_ct) = (json_num(&wall, "m_ct") as i32, json_num(&wall, "k_ct") as i32, json_num(&wall, "lambda_ct"));
    assert!(p_ct.is_finite() && c_ct.is_finite() && lambda_ct.is_finite(), "wall_ct2.json carries no transfer term");

    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let (a, b) = twist_and_bend(o, h, PRED_ANGSTROM, PRED_TWIST_DEGREES, PRED_TILT_DEGREES, PRED_DONOR_DEGREES);
    let (oo, ho, hh) = cross_classes(&a, &b);
    let pairs = ct_pairs(&a, &b);
    let r_oo_p = cross_oo(&a, &b);
    let ct_rebuilt = ct_value(&pairs, p_ct, c_ct, m_ct, k_ct, lambda_ct);
    // the geometry the prediction was filed on, checked against the one rebuilt here
    let rec_a = json_centers(&pred, "donor_centers");
    let rec_b = json_centers(&pred, "acceptor_centers");
    let same = rec_a.len() == a.centers.len()
        && rec_b.len() == b.centers.len()
        && rec_a.iter().zip(a.centers.iter()).all(|(p, q)| dist(p, q) < SAME_GEOMETRY_TOL)
        && rec_b.iter().zip(b.centers.iter()).all(|(p, q)| dist(p, q) < SAME_GEOMETRY_TOL);
    assert!(same, "the filed prediction's centers are NOT the geometry this runner builds (M-STALE-INSTRUMENT)");
    eprintln!(
        "CT-2 predict — the HELD-OUT node {PRED_NAME} (acceptor twisted {PRED_TWIST_DEGREES:.0}° then tilted {PRED_TILT_DEGREES:.0}°, donor bent {PRED_DONOR_DEGREES:.0}°, R_OO {r_oo_p:.6} bohr) on {} threads; E_pred {e_pred:.12e} Ha, the filed transfer term {ct_pred:.12e} (rebuilt from wall_ct2.json here: {ct_rebuilt:.12e}, difference {:.3e})",
        threads(),
        (ct_rebuilt - ct_pred).abs()
    );

    // ------------------------------------------------------------------- the exact solve (G-D0's)
    let exact_path = out.join(format!("{PRED_NAME}.json"));
    if !exact_path.exists() {
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let (e_full, iters, residual, converged) = fci_full_from_product(&a, &b);
        let wall_s = t0.elapsed().as_secs_f64();
        let cpu = cpu_seconds() - c0;
        let e_a0 = solve_embedded(&a.species, &a.centers, &[]);
        let e_b0 = solve_embedded(&b.species, &b.centers, &[]);
        let de = e_full - e_a0.e_total - e_b0.e_total;
        let price_expected = PRICE_TENTH * (iters as f64) * SIGMA_PRICE_FLOOR;
        fs::write(
            &exact_path,
            format!(
                "{{\n  \"node\": \"{PRED_NAME}\", \"r_oo_angstrom\": {PRED_ANGSTROM:.3}, \"r_oo_bohr\": {},\n  \"solver\": \"holon_chem::heitler_london::fci_full_from_product (G-D0's solver)\",\n  \"n_det\": {N_DET_DIMER}, \"e_super\": {}, \"e_a0\": {}, \"e_b0\": {}, \"de_exact\": {},\n  \"davidson_iters\": {iters}, \"residual\": {}, \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}, \"converged\": {converged}, \"capped\": {},\n  \"wall_seconds\": {}, \"cpu_seconds\": {}, \"threads\": {}, \"price_expected_core_seconds\": {}, \"price_admitted\": {},\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
                jd(r_oo_p),
                jn(e_full),
                jn(e_a0.e_total),
                jn(e_b0.e_total),
                jn(de),
                jn(residual),
                iters >= DAVIDSON_CAP,
                jd(wall_s),
                jd(cpu),
                threads(),
                jd(price_expected),
                cpu >= price_expected,
                centers_json(&a),
                centers_json(&b),
            ),
        )
        .unwrap();
        eprintln!("  the exact solve: E {e_full:.12e}, ΔE_exact {de:.6e} Ha, {iters} iters, residual {residual:.3e}, converged {converged}, wall {wall_s:.0} s, {cpu:.0} core-s");
    } else {
        eprintln!("  {}: exists, skipped", exact_path.display());
    }
    let t = fs::read_to_string(&exact_path).unwrap();
    let de = json_num(&t, "de_exact");
    let e_super = json_num(&t, "e_super");
    let exact_converged = json_bool(&t, "converged");

    // ------------------------------------------------------------------- the sector solve
    let sector_path = out.join(format!("sector_{PRED_NAME}.json"));
    if !sector_path.exists() {
        let t1 = Instant::now();
        let c1 = cpu_seconds();
        let r = fci_block_localised(&a, &b);
        let harness_wall = t1.elapsed().as_secs_f64();
        let cpu = cpu_seconds() - c1;
        fs::write(
            &sector_path,
            format!(
                "{{\n  \"node\": \"{PRED_NAME}\", \"r_oo_angstrom\": {PRED_ANGSTROM:.3}, \"r_oo_bohr\": {},\n  {},\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
                jd(r_oo_p),
                sector_body(&r, cpu, harness_wall),
                centers_json(&a),
                centers_json(&b),
            ),
        )
        .unwrap();
        eprintln!("  the sector solve: E_noCT {:.12e}, dim {}, {} iters, residual {:.3e}, converged {}, {cpu:.0} core-s", r.e_noct, r.sector_dim, r.davidson_iters, r.residual, r.converged);
    } else {
        eprintln!("  {}: exists, skipped", sector_path.display());
    }
    let st = fs::read_to_string(&sector_path).unwrap();
    let sec = sector_from_text(&st, &sector_path.display().to_string()).expect("the sector reading was just written");

    // ------------------------------------------------------------------------ S2's verdict
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
        "S2: ΔE_exact {de:.6e} Ha, E_pred {e_pred:.6e} — miss {miss:.3e} ({:.1} % of |ΔE_exact|) against {tol:.3e}; measured E_CT = E_exact − E_noCT = {e_ct_measured:.6e} against the predicted transfer term {ct_pred:.6e} — miss {ct_miss:.3e} against {ct_tol:.3e}; the sector's order E_exact ≤ E_noCT ≤ E_HL: {order_ok} → branch ({s2})",
        100.0 * miss / de.abs()
    );
    fs::write(
        out.join("prediction_check.json"),
        format!(
            "{{\n  \"node\": \"{PRED_NAME}\", \"r_oo_angstrom\": {PRED_ANGSTROM:.3}, \"r_oo_bohr\": {},\n  \"e_pred\": {}, \"de_exact\": {}, \"miss\": {}, \"miss_fraction\": {}, \"tolerance\": {}, \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\",\n  \"e_ct_measured\": {}, \"e_ct_measured_rule\": \"E_exact(total) − E_noCT(total), both on this geometry\", \"ct_term_predicted\": {}, \"ct_term_rebuilt_here\": {}, \"ct_miss\": {}, \"ct_tolerance\": {}, \"ct_tolerance_rule\": \"max({PRED_CT_FRAC}·|E_CT|, {PRED_CT_ABS:e})\",\n  \"s2_branch\": \"{s2}\", \"s2_branch_meaning\": \"(a) the total lands within tolerance; (b) the total misses but the transfer term is within tolerance of the measured E_CT; (c) both miss\",\n  \"exact\": {{\"solver\": \"fci_full_from_product\", \"converged\": {exact_converged}, \"davidson_iters\": {}, \"residual\": {}, \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}, \"capped\": {}, \"n_det\": {N_DET_DIMER}, \"e_super\": {}, \"e_a0\": {}, \"e_b0\": {}, \"cpu_seconds\": {}, \"wall_seconds\": {}, \"price_admitted\": {}, \"source\": \"{}\"}},\n  \"sector\": {{\"e_noct\": {}, \"e_hl_undeformed\": {}, \"e_a0\": {}, \"e_b0\": {}, \"delta_e_hl\": {}, \"closed_sector_gain\": {}, \"sector_dim\": {}, \"sector_dim_staked\": {SECTOR_DIM_STAKED}, \"davidson_iters\": {}, \"residual\": {}, \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}, \"converged\": {}, \"order_ok\": {order_ok}, \"metric_min_eigenvalue\": {}, \"cpu_seconds\": {}, \"price_admitted\": {}, \"source\": \"{}\"}},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}]\n}}\n",
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
            json_num(&t, "davidson_iters") as u64,
            jn(json_num(&t, "residual")),
            json_num(&t, "davidson_iters") as usize >= DAVIDSON_CAP,
            jn(e_super),
            jn(json_num(&t, "e_a0")),
            jn(json_num(&t, "e_b0")),
            jn(json_num(&t, "cpu_seconds")),
            jn(json_num(&t, "wall_seconds")),
            json_bool(&t, "price_admitted"),
            esc(&exact_path.display().to_string()),
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
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/ct2".to_string()));
    fs::create_dir_all(&out).expect("out");
    match what {
        "map" => run_map(&out),
        "fit" => run_fit(&out),
        "predict" => run_predict(&out),
        other => eprintln!("unknown phase {other} (map | fit | predict)"),
    }
}
