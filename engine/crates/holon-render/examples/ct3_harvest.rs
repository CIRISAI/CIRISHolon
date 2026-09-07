//! CT-3's harvest (`conformance/water_observatory/CT3_PREREG.md`): THE TERM AS A TABLE.
//!
//! CT-2 harvested charge transfer on sixty-four dimer geometries and then fitted a DECLARED
//! angular family to them. The family missed: `S1` read branch (c), 41 of 64 within tolerance,
//! and the map said why — the transfer follows the acceptor's PLANE NORMAL and rises when the
//! acceptor turns away, where the family's two lone pairs predict a peak and a fall. This
//! harvest removes the family. The map is served as the term, the way the pair curves are
//! served: knots in, an interpolant out, and the interpolant IS the potential.
//!
//! ```text
//! y  = (r, cos θ_d, u·b̂, q)              the contact's four coordinates (seam::ct_coords)
//! q  = 2(u·n̂)² + (u·b̂)² − 1              the acceptor's azimuth, smooth at both poles
//! E  = −S(ỹ)·exp(−c₀ r),  S the cubic polyharmonic spline through the map's own E_CT
//! ```
//!
//! ```text
//! cargo run --release -p holon-render --example ct3_harvest -- gate    [OUT_DIR]
//! cargo run --release -p holon-render --example ct3_harvest -- predict [OUT_DIR]
//! cargo run --release -p holon-render --example ct3_harvest -- solve   [OUT_DIR]
//! cargo run --release -p holon-render --example ct3_harvest -- read    [OUT_DIR]
//! ```
//!
//! `gate` builds the table from the sixty-four records and puts it through every gate that does
//! not need a new solve: the knots reproduce, the served law's force is its own derivative, the
//! boundedness walk, the plant, the leave-one-out gauge, and the held-out node's coordinates.
//! `predict` files the prediction for that node BEFORE it is solved. `solve` solves it, exactly
//! and in the closed sector. `read` scores the prediction against the solve.
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::{water_dimer_linear, Fragment, ANGSTROM_TO_BOHR};
use holon_chem::heitler_london::{fci_block_localised, fci_full_from_product, BlwReading};
use holon_render::seam::{ct_coords, CtLoad, CtTable, SeamModel, SeamPlant, CT_DIM};
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/quartet.rs"]
#[allow(dead_code)]
mod quartet;

// ------------------------------------------------------------------------------- the constants

/// EMBED-1's water pins — the same numbers FIELD-3's … CT-2's runners carry.
const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;

/// The map's own count (CT-2 §0, corrected before freezing): sixty-four DISTINCT geometries.
const MAP_NODES: usize = 64;

/// THE POLE RULE (§0 of the freeze). Two knots closer than this in the SCALED coordinate box are
/// one site: the acceptor's azimuth is genuinely undefined where `u` lies along the bisector, so
/// `q = 0` there for BOTH sheets and the map's own duplicate readings collide. They are merged to
/// their mean and the SPREAD is recorded as the coordinate system's own resolution — never
/// averaged away in silence.
const POLE_MERGE: f64 = 1e-9;

/// The engine's reference on both sides of every difference (bohr): the acceptor moved away.
const FAR_BOHR: f64 = 40.0;

/// G-T0's bar: the interpolant reproduces the knot it was built on. This is a statement about the
/// LINEAR SOLVE's arithmetic, not about the physics — an interpolant interpolates by definition.
const KNOT_TOL: f64 = 1e-12;

/// G-C1's bar: the engine's own `e_seam` against this runner's independent evaluation.
const G_C1_TOL: f64 = 1e-10;

/// C1's exponent grid, per class and per bohr: `0.50 ..= 4.00` step `0.02` — 176 values, CT-2's
/// own grid, carried.
const NC: usize = 176;
/// CT-2's weighting rule, carried: `1/max(|ΔE_exact|, 5e-3)²`.
const WEIGHT_FLOOR: f64 = 5e-3;
/// The clamp: bisection on `[0, P_ls]` to this RELATIVE width, at most this many rounds of the
/// two classes against each other. CT-2's numbers, carried.
const CLAMP_REL: f64 = 1e-6;
const CLAMP_ROUNDS: usize = 20;
/// C1's reading: a constrained/unconstrained residual ratio above this says the data want a shape
/// the gate forbids (named, not a failure).
const RATIO_NAMED: f64 = 2.0;
/// C1's branch (a) needs at least this many of the sixty-four inside CT-2's own S1 tolerance, and
/// the line at 2.7/2.9/3.1 Å all within.
const C1_MIN: usize = 50;
const LINE_NODES: [&str; 3] = ["linear_R2.7", "linear_R2.9", "linear_R3.1"];
/// CT-2 S2's own letter, carried as S2's comparative bar here.
const S2_FRAC: f64 = 0.25;
const S2_ABS: f64 = 5e-4;

/// G-B0's angular scan: points per axis, coarse pass then a fine pass over one coarse cell.
/// A minimum over a grid is an UPPER bound on the true minimum, so the resolution is reported
/// with the depth and never left implicit.
const SERVED_GRID: usize = 41;

/// G-B3's bar: the analytic gradient against a central difference at `h`.
const FD_H: f64 = 1e-5;
const FD_TOL: f64 = 1e-8;

/// The seam reach budget the door reads (LIQUID-1).
const REACH_BUDGET: f64 = 1e-10;

/// The held-out node (§2 S1 of the freeze), chosen by the freeze's own rule and named by the
/// map's naming convention: `twisted(2.7 Å, 20°)` then tilted `85°`, the donor unbent.
const HO_ANGSTROM: f64 = 2.7;
const HO_TWIST_DEGREES: f64 = 20.0;
const HO_TILT_DEGREES: f64 = 85.0;
const HO_DONOR_DEGREES: f64 = 0.0;
const HO_NAME: &str = "twistbent_R2.7_tw20_t85_d0";

/// The map's own solver bars (CT-2 §2 T0), carried.
const SECTOR_RESIDUAL_BAR: f64 = 1e-8;
const SECTOR_DIM_STAKED: usize = 194_481;
const N_DET_DIMER: usize = 1_002_001;
const ORDER_TOL: f64 = 1e-10;
/// M-CHEAPER-THAN-ITS-PRICE: FIELD-5's measured floor, core-seconds per Hamiltonian application.
const SIGMA_PRICE_FLOOR: f64 = 55.0;
const PRICE_TENTH: f64 = 0.1;
/// M-EXIT-DISCRIMINATOR: a solve at the cap is VOID.
const DAVIDSON_CAP: usize = 300;

/// M-FLOOR-UNSTAKED: the transfer floor, carried from CT-2.
const CT_FLOOR: f64 = 1e-6;

// ----------------------------------------------------------------------------------- plumbing

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

/// Several banked records carry numbers with a leading `+`, which is not valid JSON; this
/// string-split reader takes them as they are. NOTHING THIS RUNNER WRITES DOES THAT.
fn json_num(t: &str, key: &str) -> f64 {
    t.split(&format!("\"{key}\": ")).nth(1).and_then(|x| x.split(|c| c == ',' || c == '\n' || c == '}').next()).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::NAN)
}

fn json_str(t: &str, key: &str) -> String {
    t.split(&format!("\"{key}\": \"")).nth(1).and_then(|x| x.split('"').next()).unwrap_or("").to_string()
}

fn json_centers(t: &str, key: &str) -> Vec<[f64; 3]> {
    let Some(rest) = t.split(&format!("\"{key}\": [")).nth(1) else { return Vec::new() };
    let Some(end) = rest.find("]]") else { return Vec::new() };
    let nums: Vec<f64> = rest[..end + 1].split(|c: char| c == '[' || c == ']' || c == ',').filter_map(|x| x.trim().parse::<f64>().ok()).collect();
    nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// ONE JSON NUMBER, valid whatever the arithmetic did (M-FORMAT-FLOOR): twelve digits, never a
/// leading `+`, and `null` rather than `NaN`/`inf`.
fn jn(x: f64) -> String {
    if x.is_finite() { format!("{x:.12e}") } else { "null".to_string() }
}

fn jd(x: f64) -> String {
    if x.is_finite() { format!("{x:.6}") } else { "null".to_string() }
}

fn list_json(v: &[f64]) -> String {
    v.iter().map(|d| jd(*d)).collect::<Vec<_>>().join(", ")
}

/// The contact grid, per bohr: `0.50 ..= 4.00` step `0.02`, built so the ends are exact.
fn cgrid(i: usize) -> f64 {
    ((25 + i) as f64) * 0.02
}

/// CT-2's two-class non-negative least squares, carried verbatim: the closed-form solve with a
/// class dropped whenever its amplitude would go negative.
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
            None => active[1] = false,
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

/// THE CLAMP (CT-2 §0, carried, with the SERVED walk in place of the family's linear value): the
/// largest amplitude of one contact class the boundedness gate admits, by bisection on
/// `[0, p_ls]` to `CLAMP_REL` relative, with every other term of the law held — the transfer row
/// among them, at the deepest reading the TABLE returns at each radius. Returns
/// `(amplitude, clamped, walks, the zero law is admitted)`.
fn clamp_amplitude(p_ls: f64, q_h: f64, r_min: [f64; 3], kt: f64, ct: &dyn Fn(f64) -> f64, make: &dyn Fn(f64) -> SeamModel) -> (f64, bool, usize, bool) {
    let admits = |p: f64| make(p).bounded_ct(q_h, r_min, kt, ct).is_none();
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

fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn sibling(out: &Path, name: &str) -> PathBuf {
    let sib = out.parent().unwrap_or(Path::new(".")).join(name);
    if sib.exists() { sib } else { PathBuf::from(format!("../conformance/water_observatory/{name}")) }
}

// ------------------------------------------------------------------------------ the geometries

/// FIELD-3's `linear` verbatim.
fn linear(o: Species, h: Species, r_oo_angstrom: f64) -> (Fragment, Fragment) {
    water_dimer_linear(o, h, H2O_R, H2O_THETA, r_oo_angstrom * ANGSTROM_TO_BOHR)
}

/// FIELD-5's rotation: a fragment turned about the x-axis through ITS OWN oxygen.
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

/// FIELD-7's `twisted` verbatim: the acceptor turned `twist` about the O···O axis through its own
/// oxygen and THEN `tilt` about its own x-axis; CT-2's `twist_and_bend` adds the donor's bend.
fn twist_and_bend(o: Species, h: Species, r_oo_angstrom: f64, twist_degrees: f64, tilt_degrees: f64, donor_degrees: f64) -> (Fragment, Fragment) {
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
    (rot_x(&donor, donor_degrees), Fragment::new(acc.species.clone(), centers, acc.weights.clone()))
}

fn centers_json(f: &Fragment) -> String {
    f.centers.iter().map(|c| format!("[{:.10}, {:.10}, {:.10}]", c[0], c[1], c[2])).collect::<Vec<_>>().join(", ")
}

fn oxygen_of(f: &Fragment) -> usize {
    f.species.iter().position(|s| s.z == 8).expect("an oxygen")
}

fn hydrogens_of(f: &Fragment) -> [usize; 2] {
    let hs: Vec<usize> = (0..f.species.len()).filter(|&i| f.species[i].z == 1).collect();
    assert_eq!(hs.len(), 2, "a water unit is an oxygen with exactly two hydrogens");
    [hs[0], hs[1]]
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

/// THE SERVING RULE, in one place (the engine's `accumulate_seam` runs the same one): for an
/// ORDERED pair of units, the contact is the DONOR's hydrogen nearest the ACCEPTOR's oxygen, and
/// the coordinates are that contact's by `seam::ct_coords`. Returns the coordinates and the five
/// atoms' positions in the engine's order `[H, O_a, O_d, h₁, h₂]`.
fn contact_of(donor: &Fragment, acceptor: &Fragment) -> ([f64; CT_DIM], [[f64; 3]; 5]) {
    let od = donor.centers[oxygen_of(donor)];
    let oa = acceptor.centers[oxygen_of(acceptor)];
    let [a1, a2] = hydrogens_of(acceptor);
    let [d1, d2] = hydrogens_of(donor);
    let hi = if dist(&donor.centers[d1], &oa) <= dist(&donor.centers[d2], &oa) { d1 } else { d2 };
    let pts = [donor.centers[hi], oa, od, acceptor.centers[a1], acceptor.centers[a2]];
    (ct_coords(pts[0], pts[1], pts[2], pts[3], pts[4]).0, pts)
}

/// The node's OWN contact: the map's geometries name a donor and an acceptor, and the node's
/// `E_CT` is that dimer's. The direction is fixed by the shorter contact, which is the same rule
/// the engine uses for each ordered pair.
fn node_contact(a: &Fragment, b: &Fragment) -> ([f64; CT_DIM], [[f64; 3]; 5]) {
    let (ya, pa) = contact_of(a, b);
    let (yb, pb) = contact_of(b, a);
    if ya[0] <= yb[0] { (ya, pa) } else { (yb, pb) }
}

// ---------------------------------------------------------------------------------- the engine

/// FIELD-4's `engine_dimer` verbatim, with CT-3's table installed and the transfer mode selected.
fn engine_dimer(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, plant: SeamPlant, table: Option<&CtTable>) -> Box<Sim> {
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
    if let Some(t) = table {
        s.ct_table = t.clone();
    }
    s.seam_plant = plant;
    s.set_seam(seam).expect("no acuity frame");
    s.refresh_pairs();
    s.compute_forces();
    s
}

/// `E(geometry) − E(acceptor moved 40 bohr along x)` on the rows the seam serves between units.
fn engine_interaction(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, plant: SeamPlant, table: Option<&CtTable>) -> (f64, f64, f64) {
    let s = engine_dimer(a, b, seam, plant, table);
    let near = (s.e_pair + s.e_three) + s.e_field + s.e_seam;
    let far_b = b.translated([FAR_BOHR, 0.0, 0.0]);
    let f = engine_dimer(a, &far_b, seam, plant, table);
    let far = (f.e_pair + f.e_three) + f.e_field + f.e_seam;
    (near - far, s.e_field - f.e_field, s.e_seam - f.e_seam)
}

// ----------------------------------------------------------------------------- the law of record

/// CT-2's admitted law (`wall_ct2.json`), with its transfer row read separately: the table
/// REPLACES `p_ct`/`c_ct`/`m_ct`, and `c_ct` becomes the exponent the table divides out.
struct Law {
    m: SeamModel,
    c0: f64,
    p_ct: f64,
    m_ct: u8,
    q_h: f64,
    kt: f64,
    r_min: [f64; 3],
    source: String,
}

fn load_law(out: &Path) -> Law {
    let p = sibling(out, "ct2/wall_ct2.json");
    let t = fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let g = |k: &str| json_num(&t, k);
    Law {
        m: SeamModel {
            a: g("a"),
            b: g("b"),
            p: g("p"),
            c: g("c"),
            c6: g("c6"),
            a_oh: g("a_oh"),
            b_oh: g("b_oh"),
            a_hh: g("a_hh"),
            b_hh: g("b_hh"),
            p_hh: g("p_hh"),
            c_hh: g("c_hh"),
            // the transfer row is the TABLE's; these are held at zero so that nothing serves twice
            p_ct: 0.0,
            c_ct: 0.0,
            m_ct: 0,
            k_ct: 0,
            lambda_ct: 0.0,
            r_cut: 0.0,
            ct_table_on: true,
        },
        c0: g("c_ct"),
        p_ct: g("p_ct"),
        m_ct: g("m_ct") as u8,
        q_h: g("q_h"),
        kt: g("kt"),
        r_min: [g("r_min_oo"), g("r_min_oh"), g("r_min_hh")],
        source: p.display().to_string(),
    }
}

/// CT-2's fitted family, evaluated on all four cross-unit H–O pairs — the thing the table must
/// beat. `k = 0` at the fit, so no acceptor factor and `λ` is immaterial.
fn family_ct(a: &Fragment, b: &Fragment, p_ct: f64, c0: f64, m_ct: u8) -> f64 {
    let mut tot = 0.0;
    for (donor, acceptor) in [(a, b), (b, a)] {
        let od = donor.centers[oxygen_of(donor)];
        let oa = acceptor.centers[oxygen_of(acceptor)];
        for &hi in hydrogens_of(donor).iter() {
            let xh = donor.centers[hi];
            let r = dist(&xh, &oa);
            let av = [od[0] - xh[0], od[1] - xh[1], od[2] - xh[2]];
            let bv = [oa[0] - xh[0], oa[1] - xh[1], oa[2] - xh[2]];
            let (na, nb) = ((av.iter().map(|x| x * x).sum::<f64>()).sqrt(), (bv.iter().map(|x| x * x).sum::<f64>()).sqrt());
            let cd = ((0..3).map(|i| av[i] * bv[i]).sum::<f64>() / (na * nb)).clamp(-1.0, 1.0);
            tot += (-c0 * r).exp() * (0.5 * (1.0 - cd)).powi(m_ct as i32);
        }
    }
    -p_ct * tot
}

// -------------------------------------------------------------------------------- the map's nodes

struct MapNode {
    name: String,
    family: String,
    a: Fragment,
    b: Fragment,
    e_ct: f64,
    de_exact: f64,
    e_exact: f64,
    e_noct: f64,
    y: [f64; CT_DIM],
    pts: [[f64; 3]; 5],
    exact_source: String,
    sector_source: String,
    r_oo: f64,
}

/// EVERY node of CT-2's map, with `E_CT` by the records' own rule. Fifty-one carry it in the node
/// record; the thirteen of record carry the exact total in `gd0_*.json` and the closed-sector
/// total in CT-1's `sector_*.json`, and `E_CT` is their difference — never a third source.
fn load_map(out: &Path, o: Species, h: Species) -> Vec<MapNode> {
    let ct2 = sibling(out, "ct2");
    let ct1 = sibling(out, "ct1");
    let mut files: Vec<PathBuf> = fs::read_dir(&ct2)
        .unwrap_or_else(|e| panic!("{}: {e}", ct2.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let n = p.file_name().and_then(|x| x.to_str()).unwrap_or("");
            (n.starts_with("node_") || n.starts_with("gd0_")) && n.ends_with(".json")
        })
        .collect();
    files.sort();
    let mut out_nodes = Vec::new();
    for f in files {
        let t = fs::read_to_string(&f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        let name = json_str(&t, "node");
        let dc = json_centers(&t, "donor_centers");
        let ac = json_centers(&t, "acceptor_centers");
        assert_eq!(dc.len(), 3, "{name}: a donor is an oxygen and two hydrogens");
        assert_eq!(ac.len(), 3, "{name}: an acceptor is an oxygen and two hydrogens");
        let a = Fragment::new(vec![o, h, h], dc, vec![-2.0, 1.0, 1.0]);
        let b = Fragment::new(vec![o, h, h], ac, vec![-2.0, 1.0, 1.0]);
        let new_node = f.file_name().and_then(|x| x.to_str()).unwrap_or("").starts_with("node_");
        let (e_ct, de_exact, e_exact, e_noct, sector_source) = if new_node {
            (json_num(&t, "e_ct"), json_num(&t, "de_exact"), json_num(t.split("\"exact\": {").nth(1).unwrap_or(&t), "e_total"), json_num(t.split("\"sector\": {").nth(1).unwrap_or(&t), "e_noct"), f.display().to_string())
        } else {
            let sec = ct1.join(format!("sector_{name}.json"));
            let ts = fs::read_to_string(&sec).unwrap_or_else(|e| panic!("{}: {e}", sec.display()));
            let ex = json_num(&t, "e_full");
            let nc = json_num(&ts, "e_noct");
            // the monomer references come from the SECTOR record's own setup, which is the rule
            // every node record of the map already states for `de_exact`
            (ex - nc, ex - (json_num(&ts, "e_a0") + json_num(&ts, "e_b0")), ex, nc, sec.display().to_string())
        };
        let (y, pts) = node_contact(&a, &b);
        out_nodes.push(MapNode {
            name,
            family: json_str(&t, "family"),
            r_oo: dist(&a.centers[oxygen_of(&a)], &b.centers[oxygen_of(&b)]),
            a,
            b,
            e_ct,
            de_exact,
            e_exact,
            e_noct,
            y,
            pts,
            exact_source: f.display().to_string(),
            sector_source,
        });
    }
    out_nodes
}

/// One knot of the table, and which nodes it carries.
struct Site {
    y: [f64; CT_DIM],
    value: f64,
    members: Vec<usize>,
    spread: f64,
}

/// THE POLE RULE, applied: nodes whose coordinates agree to `POLE_MERGE` are ONE site. The map
/// has four such groups and every one of them is a POLE of the acceptor's azimuth — `u` along the
/// bisector, where `q = 0` for both sheets by construction. The merged value is the mean and the
/// spread is kept; a table cannot carry two values at one site, and averaging one away without
/// saying so would launder a coordinate degeneracy into an interpolation.
fn sites_of(nodes: &[MapNode], lo: [f64; CT_DIM], rng: [f64; CT_DIM]) -> Vec<Site> {
    let scaled = |y: [f64; CT_DIM]| -> [f64; CT_DIM] {
        let mut s = [0.0; CT_DIM];
        for j in 0..CT_DIM {
            s[j] = (y[j] - lo[j]) / rng[j];
        }
        s
    };
    let mut sites: Vec<Site> = Vec::new();
    for (i, n) in nodes.iter().enumerate() {
        let si = scaled(n.y);
        let hit = sites.iter().position(|s| {
            let sj = scaled(s.y);
            (0..CT_DIM).map(|j| (si[j] - sj[j]) * (si[j] - sj[j])).sum::<f64>().sqrt() < POLE_MERGE
        });
        match hit {
            Some(k) => sites[k].members.push(i),
            None => sites.push(Site { y: n.y, value: 0.0, members: vec![i], spread: 0.0 }),
        }
    }
    for s in sites.iter_mut() {
        let vals: Vec<f64> = s.members.iter().map(|&i| nodes[i].e_ct).collect();
        s.value = vals.iter().sum::<f64>() / vals.len() as f64;
        s.spread = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - vals.iter().cloned().fold(f64::INFINITY, f64::min);
    }
    sites
}

fn axis_box(nodes: &[MapNode]) -> ([f64; CT_DIM], [f64; CT_DIM]) {
    let mut lo = [f64::INFINITY; CT_DIM];
    let mut hi = [f64::NEG_INFINITY; CT_DIM];
    for n in nodes {
        for j in 0..CT_DIM {
            lo[j] = lo[j].min(n.y[j]);
            hi[j] = hi[j].max(n.y[j]);
        }
    }
    let mut rng = [0.0; CT_DIM];
    for j in 0..CT_DIM {
        rng[j] = hi[j] - lo[j];
    }
    (lo, rng)
}

fn table_from(sites: &[Site], c0: f64) -> CtTable {
    let mut t = CtTable::empty();
    assert!(t.begin(sites.len(), c0), "the table refused {} knots at c0 = {c0}", sites.len());
    for (i, s) in sites.iter().enumerate() {
        assert!(t.knot(i, s.y, s.value), "knot {i}");
    }
    let st = t.finish();
    assert_eq!(st, CtLoad::Ok, "the table did not load: {st:?}");
    t
}

// ------------------------------------------------------------------------------------- the gate

fn run_gate(out: &Path) {
    let t0 = Instant::now();
    let cpu0 = cpu_seconds();
    let (o, h) = (by_symbol("O").expect("O"), by_symbol("H").expect("H"));
    let law = load_law(out);
    let nodes = load_map(out, o, h);
    assert_eq!(nodes.len(), MAP_NODES, "the map is sixty-four nodes; {} were read", nodes.len());
    let (lo, rng) = axis_box(&nodes);
    let sites = sites_of(&nodes, lo, rng);
    let table = table_from(&sites, law.c0);
    eprintln!("{} nodes -> {} sites; table loaded, worst knot miss {:.3e}", nodes.len(), sites.len(), table.worst_knot_miss);

    // --- G-T0: the interpolant reproduces every knot it was built on ---------------------------
    let mut worst_knot = 0.0f64;
    for s in sites.iter() {
        worst_knot = worst_knot.max((table.eval(s.y) - s.value).abs());
    }
    let g_t0 = worst_knot <= KNOT_TOL;

    // --- G-T1: every NODE of the map, against the table it was built from -----------------------
    // The four merged sites read their mean, so their misses are half their own spread. Named,
    // per node, with the spread beside them; nothing is asserted zero that is not.
    let mut node_rows = Vec::new();
    let mut worst_node = 0.0f64;
    let mut merged_worst = 0.0f64;
    for (i, n) in nodes.iter().enumerate() {
        let e = table.eval(n.y);
        let miss = (e - n.e_ct).abs();
        let site = sites.iter().find(|s| s.members.contains(&i)).expect("every node has a site");
        let merged = site.members.len() > 1;
        if merged {
            merged_worst = merged_worst.max(miss);
        } else {
            worst_node = worst_node.max(miss);
        }
        node_rows.push((n, e, miss, merged, site.spread));
    }
    let g_t1 = worst_node <= KNOT_TOL;

    // --- LOO: the gauge that says what the table is worth where it has no knot -------------------
    // Each site removed, the table rebuilt from the rest, the removed site predicted. This is the
    // only number that says anything about a geometry the map does not carry, and every stake on
    // the held-out node is derived from it.
    let mut loo = vec![0.0f64; sites.len()];
    for k in 0..sites.len() {
        let rest: Vec<Site> = sites.iter().enumerate().filter(|(i, _)| *i != k).map(|(_, s)| Site { y: s.y, value: s.value, members: s.members.clone(), spread: s.spread }).collect();
        let t = table_from(&rest, law.c0);
        loo[k] = t.eval(sites[k].y);
    }
    let mut loo_miss: Vec<(String, f64)> = sites.iter().enumerate().map(|(k, s)| (s.members.iter().map(|&i| nodes[i].name.clone()).collect::<Vec<_>>().join("/"), (loo[k] - s.value).abs())).collect();
    let loo_worst = loo_miss.iter().map(|x| x.1).fold(0.0f64, f64::max);
    let mut sorted: Vec<f64> = loo_miss.iter().map(|x| x.1).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let loo_median = sorted[sorted.len() / 2];
    loo_miss.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // the family's own miss on the same sixty-four, for the head-to-head the freeze stakes S1 on
    let mut fam_within = 0usize;
    let mut tab_within = 0usize;
    let mut fam_worst = 0.0f64;
    let mut beats = 0usize;
    for (i, n) in nodes.iter().enumerate() {
        let fam = family_ct(&n.a, &n.b, law.p_ct, law.c0, law.m_ct);
        // CT-2's OWN S1 tolerance, so the two counts are comparable: max(0.25*|ΔE_exact|, 5e-4)
        let tol = (0.25 * n.de_exact.abs()).max(5e-4);
        let fm = (fam - n.e_ct).abs();
        let site = sites.iter().position(|s| s.members.contains(&i)).expect("a site");
        let tm = (loo[site] - n.e_ct).abs();
        fam_worst = fam_worst.max(fm);
        if fm <= tol {
            fam_within += 1;
        }
        if tm <= tol {
            tab_within += 1;
        }
        if tm < fm {
            beats += 1;
        }
    }

    // --- G-B3: the force is the derivative of the served term ------------------------------------
    // On every node of the map, a central difference of the table's own energy against its posted
    // gradient, on every coordinate of all five atoms, and the gradients' sum (translation).
    let mut fd_worst = 0.0f64;
    let mut trans_worst = 0.0f64;
    let mut fd_node = String::new();
    for n in nodes.iter() {
        let pts = n.pts;
        let (_, g) = table.serve(pts[0], pts[1], pts[2], pts[3], pts[4]);
        for i in 0..5 {
            for c in 0..3 {
                let mut pp = pts;
                pp[i][c] += FD_H;
                let (ep, _) = table.serve(pp[0], pp[1], pp[2], pp[3], pp[4]);
                pp[i][c] -= 2.0 * FD_H;
                let (em, _) = table.serve(pp[0], pp[1], pp[2], pp[3], pp[4]);
                let fd = (ep - em) / (2.0 * FD_H);
                let rel = (g[i][c] - fd).abs() / (1.0 + fd.abs());
                if rel > fd_worst {
                    fd_worst = rel;
                    fd_node = format!("{} atom {i} coord {c}", n.name);
                }
            }
        }
        for c in 0..3 {
            trans_worst = trans_worst.max((0..5).map(|i| g[i][c]).sum::<f64>().abs());
        }
    }
    let g_b3 = fd_worst <= FD_TOL;

    // --- G-C1: the engine serves what the table says ---------------------------------------------
    // The engine's `e_seam` against this runner's own sum, class by class, on every node — and the
    // REVERSE-DIRECTION LEAK named beside it: the engine reads one contact per ORDERED pair, so a
    // dimer gets two readings and the map's `E_CT` is one of them. The second is measured, never
    // assumed zero (M-VACUOUS-SUCCESS).
    let mut c1_worst = 0.0f64;
    let mut leak_worst = 0.0f64;
    let mut leak_node = String::new();
    let mut units_ok = true;
    for n in nodes.iter() {
        let (_, _, seam) = engine_interaction(&n.a, &n.b, Some(law.m), SeamPlant::None, Some(&table));
        let (oo, ho, hh) = cross_classes(&n.a, &n.b);
        let (yf, _) = contact_of(&n.a, &n.b);
        let (yr, _) = contact_of(&n.b, &n.a);
        let (y1, y2) = if yf[0] <= yr[0] { (yf, yr) } else { (yr, yf) };
        let formula: f64 = oo.iter().map(|&r| law.m.wall(r) + law.m.dispersion(r)).sum::<f64>()
            + ho.iter().map(|&r| law.m.penetration(r) + law.m.wall_oh(r)).sum::<f64>()
            + hh.iter().map(|&r| law.m.wall_hh(r) - law.m.p_hh * (-law.m.c_hh * r).exp()).sum::<f64>()
            + table.eval(y1);
        c1_worst = c1_worst.max((seam - formula).abs());
        // what serving the OTHER direction as well would have added — the double count the
        // serving rule refuses, measured on every node rather than argued away
        let leak = table.eval(y2);
        if leak.abs() > leak_worst {
            leak_worst = leak.abs();
            leak_node = n.name.clone();
        }
        let s = engine_dimer(&n.a, &n.b, Some(law.m), SeamPlant::None, Some(&table));
        if s.seam_work.units != 2 {
            units_ok = false;
        }
    }
    let g_c1 = c1_worst <= G_C1_TOL;

    // --- G-B0: the boundedness walk, both legs named ----------------------------------------------
    // The table is not a pair term, so the walk takes the DEEPEST reading the table can return at
    // each separation (`CtTable::deepest`) — strictly stronger than CT-2's, which took its family
    // at the linear value because for that family the linear value WAS the deepest. The linear leg
    // is walked beside it and reported, so a law admitted on one and refused on the other cannot
    // hide behind the choice (M-FIRST-VIOLATION-ONLY).
    // THE GATE (the lead's ruling, 2026-09-06): the walk takes the SERVED term — the
    // interpolant itself at the walk's own radius, minimised over every angular geometry a
    // real contact can present, including the table's own behaviour below its innermost knot.
    // The strict leg (an outer knot's shape carried inward under exp(−c₀ r)) and the near leg
    // are kept beside it, numbers unchanged, and neither is hidden behind the other.
    let served = law.m.bounded_ct(law.q_h, law.r_min, law.kt, &|r| table.deepest_served(r, SERVED_GRID).0);
    let hole_served = law.m.hole_ct(law.q_h, &|r| table.deepest_served(r, SERVED_GRID).0);
    let deep = law.m.bounded_ct(law.q_h, law.r_min, law.kt, &|r| table.deepest(r));
    let lin_y = |r: f64| -> [f64; CT_DIM] { [r, -1.0, -1.0, 0.0] };
    let lin = law.m.bounded_ct(law.q_h, law.r_min, law.kt, &|r| table.eval(lin_y(r)));
    // the near leg: the deepest shape the table carries AT OR INSIDE the H–O class's own fit
    // floor, carried inward the strict leg's way
    let near_shape = sites
        .iter()
        .filter(|s| s.y[0] <= law.r_min[1])
        .map(|s| -s.value * (law.c0 * s.y[0]).exp())
        .fold(f64::NEG_INFINITY, f64::max);
    let near = law.m.bounded_ct(law.q_h, law.r_min, law.kt, &|r| -near_shape * (-law.c0 * r).exp());
    // THE DECLARED INWARD FENCE, walked only if the served walk refuses (§2 G-B0): the shape is
    // read at the innermost knot's separation for every r below it while the prefactor keeps
    // running. Nothing is chosen; the value held is the interpolant's own at the knot floor.
    let mut fenced_table = table.clone();
    fenced_table.r_clamp = table.r_knot_floor();
    let fenced = law.m.bounded_ct(law.q_h, law.r_min, law.kt, &|r| fenced_table.deepest_served(r, SERVED_GRID).0);
    let hole_fenced = law.m.hole_ct(law.q_h, &|r| fenced_table.deepest_served(r, SERVED_GRID).0);
    // the fence's price: the force discontinuity it leaves at r_clamp, measured on the deepest
    // geometry there. The energy is continuous; dS/dr is not.
    let rc = table.r_knot_floor();
    let (_, arg_rc) = table.deepest_served(rc, SERVED_GRID);
    let fence_jump = {
        let e = 1e-6;
        let below = [rc - e, arg_rc[1], arg_rc[2], arg_rc[3]];
        let above = [rc + e, arg_rc[1], arg_rc[2], arg_rc[3]];
        (fenced_table.eval_grad(below).1[0] - fenced_table.eval_grad(above).1[0]).abs()
    };
    // the served walk's own profile, so the record shows where it goes rather than only whether
    // it passed (M-VACUOUS-SUCCESS)
    let profile: Vec<(f64, f64, f64, [f64; CT_DIM])> = [law.r_min[1], 2.4028, 2.0, 1.5, 1.0, 0.5]
        .iter()
        .map(|&r| {
            let (a, y) = table.deepest_served(r, SERVED_GRID);
            (r, a, fenced_table.deepest_served(r, SERVED_GRID).0, y)
        })
        .collect();
    let g_b0 = served.is_none();
    let reach = table.reach(REACH_BUDGET).max(law.m.reach(REACH_BUDGET));

    // --- C1: the two contact terms re-fit UNDER the gate, with the TABLE held ---------------------
    // CT-2's contact terms were clamped against CT-2's OWN transfer row, which is about half the
    // table's depth at these radii, and the served walk above refuses that law. The lever is
    // CT-2's own C1 machinery applied to CT-3's transfer row, and it is IN this freeze because
    // nothing about the held-out node has been seen. The remainder each class is fit on is
    // `ΔE_exact − [E_q(g) − E_q(40)]_engine − wall(g) − TABLE(g)`, and the table is EXACT at the
    // knots, so the remainder is the map's own residual and not a fit's.
    let t_c1 = Instant::now();
    let walls_only = SeamModel { p: 0.0, c: 0.0, p_hh: 0.0, c_hh: 0.0, ct_table_on: false, ..law.m };
    let nn_nodes = nodes.len();
    let mut rem = vec![0.0f64; nn_nodes];
    let mut we = vec![0.0f64; nn_nodes];
    let mut e_q = vec![0.0f64; nn_nodes];
    let mut wall_of = vec![0.0f64; nn_nodes];
    let mut ct_of_node = vec![0.0f64; nn_nodes];
    let mut cho = vec![0.0f64; NC * nn_nodes];
    let mut chh = vec![0.0f64; NC * nn_nodes];
    let mut syy_c = 0.0f64;
    for (g, n) in nodes.iter().enumerate() {
        let (_, field, _) = engine_interaction(&n.a, &n.b, Some(walls_only), SeamPlant::None, None);
        let (oo, ho, hh) = cross_classes(&n.a, &n.b);
        let w: f64 = oo.iter().map(|&r| law.m.wall(r)).sum::<f64>() + ho.iter().map(|&r| law.m.wall_oh(r)).sum::<f64>() + hh.iter().map(|&r| law.m.wall_hh(r)).sum::<f64>();
        let (y, _) = node_contact(&n.a, &n.b);
        let ct = table.eval(y);
        e_q[g] = field;
        wall_of[g] = w;
        ct_of_node[g] = ct;
        rem[g] = n.de_exact - field - w - ct;
        we[g] = 1.0 / (n.de_exact.abs().max(WEIGHT_FLOOR)).powi(2);
        syy_c += we[g] * rem[g] * rem[g];
        for i in 0..NC {
            cho[i * nn_nodes + g] = -ho.iter().map(|&r| (-cgrid(i) * r).exp()).sum::<f64>();
            chh[i * nn_nodes + g] = -hh.iter().map(|&r| (-cgrid(i) * r).exp()).sum::<f64>();
        }
    }
    // the served transfer row, MEMOISED on the walk's own radii: the clamp runs 30,976 grid
    // points x up to 20 bisections x two classes, and the table's angular minimum at one radius
    // does not depend on any contact amplitude. Without this the fit is not affordable and with
    // it the walk is bit-identical to the one G-B0 ran.
    let memo: std::cell::RefCell<std::collections::HashMap<u64, f64>> = std::cell::RefCell::new(std::collections::HashMap::new());
    let ct_served = |r: f64| -> f64 {
        let k = r.to_bits();
        if let Some(v) = memo.borrow().get(&k) {
            return *v;
        }
        let v = table.deepest_served(r, SERVED_GRID).0;
        memo.borrow_mut().insert(k, v);
        v
    };
    let (mut c_res, mut c_best) = (f64::INFINITY, (0usize, 0usize, 0.0f64, 0.0f64, [false; 2], 1usize));
    let (mut u_res, mut u_best) = (f64::INFINITY, (0usize, 0usize, [0.0f64; 2], [false; 2]));
    let (mut clamp_calls, mut floor_refused, mut clamped_points) = (0u64, 0u64, 0u64);
    for i in 0..NC {
        let c_ho_i = cgrid(i);
        for j in 0..NC {
            let c_hh_j = cgrid(j);
            let (mut m00, mut m01, mut m11, mut r0v, mut r1v) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
            for g in 0..nn_nodes {
                let (x0, x1) = (cho[i * nn_nodes + g], chh[j * nn_nodes + g]);
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
            let mk = |p: f64, q: f64| SeamModel { p, c: c_ho_i, p_hh: q, c_hh: c_hh_j, ..law.m };
            let (mut ph, mut pj) = (xu[0], xu[1]);
            let (mut clamped, mut rounds, mut floor_ok) = ([false; 2], 1usize, true);
            for round in 1..=CLAMP_ROUNDS {
                let ls_ho = if m00 > 0.0 { ((r0v - m01 * pj) / m00).max(0.0) } else { 0.0 };
                let (nh, ch, k1, f1) = clamp_amplitude(ls_ho, law.q_h, law.r_min, law.kt, &ct_served, &|p| mk(p, pj));
                let ls_hh = if m11 > 0.0 { ((r1v - m01 * nh) / m11).max(0.0) } else { 0.0 };
                let (nj, cj2, k2, f2) = clamp_amplitude(ls_hh, law.q_h, law.r_min, law.kt, &ct_served, &|p| mk(nh, p));
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
            for g in 0..nn_nodes {
                let d = rem[g] - (ph * cho[i * nn_nodes + g] + pj * chh[j * nn_nodes + g]);
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
    let c1_ratio = if u_res > 0.0 { c_res / u_res } else { f64::NAN };
    let c1_seconds = t_c1.elapsed().as_secs_f64();
    // the re-fit law, and its per-node miss under CT-2's OWN S1 tolerance
    let refit = SeamModel { p: p_ho, c: c_ho, p_hh, c_hh, ..law.m };
    let mut c1_within = 0usize;
    let mut c1_line = 0usize;
    let mut c1_fit_worst = 0.0f64;
    let mut c1_fit_worst_node = String::new();
    let mut c1_fit_worst_frac = 0.0f64;
    let mut c1_fit_worst_frac_node = String::new();
    let mut c1_misses: Vec<String> = Vec::new();
    for (g, n) in nodes.iter().enumerate() {
        let fitted = p_ho * cho[ci * nn_nodes + g] + p_hh * chh[cj * nn_nodes + g];
        let miss = (rem[g] - fitted).abs();
        let tol = (0.25 * n.de_exact.abs()).max(5e-4);
        if miss <= tol {
            c1_within += 1;
            if LINE_NODES.contains(&n.name.as_str()) {
                c1_line += 1;
            }
        } else {
            c1_misses.push(n.name.clone());
        }
        if miss > c1_fit_worst {
            c1_fit_worst = miss;
            c1_fit_worst_node = n.name.clone();
        }
        let frac = miss / n.de_exact.abs().max(WEIGHT_FLOOR);
        if frac > c1_fit_worst_frac {
            c1_fit_worst_frac = frac;
            c1_fit_worst_frac_node = n.name.clone();
        }
    }
    let c1_branch = if c1_line < LINE_NODES.len() { "c" } else if c1_within >= C1_MIN { "a" } else { "b" };
    // --- G-B0W: the WHOLE served law, walked ------------------------------------------------------
    // RUN even though the clamp makes it pass by construction, because a gate that is assumed is
    // not a gate (M-VACUOUS-SUCCESS). This is LIQUID-2's admission gate.
    let whole = refit.bounded_ct(law.q_h, law.r_min, law.kt, &ct_served);
    let whole_hole = refit.hole_ct(law.q_h, &ct_served);
    let g_b0w = whole.is_none();
    // --- S2: the held-out node's FULL dimer energy under the re-fit law ---------------------------
    let (s2a, s2b) = twist_and_bend(o, h, HO_ANGSTROM, HO_TWIST_DEGREES, HO_TILT_DEGREES, HO_DONOR_DEGREES);
    let (s2_total, s2_field, s2_seam) = engine_interaction(&s2a, &s2b, Some(refit), SeamPlant::None, Some(&table));
    // THE BAND, both ways. The absolute worst in-sample miss is dominated by the innermost node,
    // where every term is large; the same worst expressed as a FRACTION of that node's own
    // |ΔE_exact| and applied to the predicted total is the scaled reading. Branch (a) takes the
    // TIGHTER of the two, because which is tighter is a measurement and not an assumption
    // (a per-unit ratio needs its fixed-target control beside it).
    let s2_band_abs = c1_fit_worst;
    let s2_band_frac = c1_fit_worst_frac * s2_total.abs();
    let s2_band = s2_band_abs.min(s2_band_frac);
    let s2_bar = (S2_FRAC * s2_total.abs()).max(S2_ABS);

    // --- the plant: one knot moved by its own tolerance moves the served value there ---------------
    // Carrier: the knot's own tolerance, asserted nonzero at the site the plant acts on.
    let plant_i = {
        let k = sites.iter().position(|s| s.members.iter().any(|&i| nodes[i].name == "linear_R2.9")).expect("linear_R2.9 is a site");
        let carrier = (0.25 * sites[k].value.abs()).max(5e-4);
        let mut moved: Vec<Site> = sites.iter().map(|s| Site { y: s.y, value: s.value, members: s.members.clone(), spread: s.spread }).collect();
        moved[k].value += carrier;
        let t2 = table_from(&moved, law.c0);
        let before = table.eval(sites[k].y);
        let after = t2.eval(sites[k].y);
        (carrier, before, after, (after - before - carrier).abs())
    };
    // --- the plant: the sign flipped in the engine ------------------------------------------------
    let plant_ii = {
        let n = nodes.iter().find(|n| n.name == "linear_R2.9").expect("linear_R2.9");
        let (_, _, s0) = engine_interaction(&n.a, &n.b, Some(law.m), SeamPlant::None, Some(&table));
        let (_, _, s1) = engine_interaction(&n.a, &n.b, Some(law.m), SeamPlant::FlipChargeTransfer, Some(&table));
        let (y, _) = node_contact(&n.a, &n.b);
        let expect = 2.0 * table.eval(y).abs();
        (s1 - s0, expect, ((s1 - s0).abs() - expect).abs(), table.eval(y).abs())
    };

    // --- the argmin's own seam, measured -----------------------------------------------------------
    // Where the donor's two hydrogens are equidistant from the acceptor's oxygen the contact
    // switches and the served value jumps. The freeze names it; this measures it, on the map's own
    // separations, at the tie the linear dimer's donor reaches when it is turned to bisect.
    let mut tie_worst = 0.0f64;
    let mut tie_at = String::new();
    for &r_ang in [2.7f64, 2.9, 3.1, 3.4].iter() {
        // the donor swept through every orientation about its own oxygen: wherever the CONTACT
        // atom changes hands the served value jumps, and the largest such jump is the seam
        let (donor0, acc) = linear(o, h, r_ang);
        let mut prev: Option<(usize, f64)> = None;
        let steps = 3600;
        for k in 0..=steps {
            let deg = 360.0 * (k as f64) / (steps as f64);
            let d = rot_x(&donor0, deg);
            let (y, pts) = node_contact(&d, &acc);
            let _ = y;
            // which atom carries the contact, by its position (the fragments are rebuilt each step)
            let who = if dist(&pts[0], &d.centers[hydrogens_of(&d)[0]]) < 1e-12 {
                0
            } else if dist(&pts[0], &d.centers[hydrogens_of(&d)[1]]) < 1e-12 {
                1
            } else {
                2
            };
            let e = table.eval(node_contact(&d, &acc).0);
            if let Some((pw, pe)) = prev {
                if pw != who && (e - pe).abs() > tie_worst {
                    tie_worst = (e - pe).abs();
                    tie_at = format!("the donor turned {deg:.2}° about its own oxygen at R_OO = {r_ang} Å, the contact passing from atom {pw} to atom {who}");
                }
            }
            prev = Some((who, e));
        }
    }

    // --- the held-out node ------------------------------------------------------------------------
    let (ha, hb) = twist_and_bend(o, h, HO_ANGSTROM, HO_TWIST_DEGREES, HO_TILT_DEGREES, HO_DONOR_DEGREES);
    let (hy, _) = node_contact(&ha, &hb);
    let ho_table = table.eval(hy);
    let ho_family = family_ct(&ha, &hb, law.p_ct, law.c0, law.m_ct);
    let mut ho_dmin = f64::INFINITY;
    let mut ho_near = String::new();
    for s in sites.iter() {
        let d: f64 = (0..CT_DIM).map(|j| ((hy[j] - s.y[j]) / rng[j]).powi(2)).sum::<f64>().sqrt();
        if d < ho_dmin {
            ho_dmin = d;
            ho_near = s.members.iter().map(|&i| nodes[i].name.clone()).collect::<Vec<_>>().join("/");
        }
    }
    // the knots' own nearest-neighbour spacings — the band the held-out node was chosen inside
    let mut nn: Vec<f64> = Vec::new();
    for (i, si) in sites.iter().enumerate() {
        let mut best = f64::INFINITY;
        for (j, sj) in sites.iter().enumerate() {
            if i == j {
                continue;
            }
            let d: f64 = (0..CT_DIM).map(|k| ((si.y[k] - sj.y[k]) / rng[k]).powi(2)).sum::<f64>().sqrt();
            best = best.min(d);
        }
        nn.push(best);
    }
    let nn_mean = nn.iter().sum::<f64>() / nn.len() as f64;
    let nn_max = nn.iter().cloned().fold(0.0f64, f64::max);

    let seconds = t0.elapsed().as_secs_f64();
    let cpu = cpu_seconds() - cpu0;

    // ---- the records ------------------------------------------------------------------------------
    fs::write(
        out.join("ct_table.json"),
        format!(
            "{{\n  \"knots\": {}, \"nodes\": {}, \"c0_per_bohr\": {}, \"c0_source\": \"{}\",\n  \
             \"kernel\": \"cubic polyharmonic spline r^3 with a linear polynomial tail, on the four coordinates scaled to the knots' own box\",\n  \
             \"coordinates\": \"(r, cos theta_d, u.b, q) with q = 2(u.n)^2 + (u.b)^2 - 1; every definition seam::ct_angular's own\",\n  \
             \"shape_rule\": \"S_i = -E_CT_i * exp(c0 * r_i); the served energy is -S(y) * exp(-c0 * r)\",\n  \
             \"pole_rule\": \"two knots within {} in the scaled box are ONE site at the mean, and the spread is recorded; every such group is a pole of the acceptor's azimuth\",\n  \
             \"axis_lo\": [{}], \"axis_hi\": [{}],\n  \"worst_knot_miss\": {}, \"deepest_shape\": {}, \"reach_1e-10_bohr\": {},\n  \
             \"nearest_neighbour_scaled\": {{\"min\": {}, \"mean\": {}, \"max\": {}}},\n  \"sites\": [\n{}\n  ]\n}}\n",
            sites.len(),
            nodes.len(),
            jn(law.c0),
            esc(&law.source),
            jn(POLE_MERGE),
            (0..CT_DIM).map(|j| jn(lo[j])).collect::<Vec<_>>().join(", "),
            (0..CT_DIM).map(|j| jn(lo[j] + rng[j])).collect::<Vec<_>>().join(", "),
            jn(table.worst_knot_miss),
            jn(table.deepest_shape),
            jd(table.reach(REACH_BUDGET)),
            jn(nn.iter().cloned().fold(f64::INFINITY, f64::min)),
            jn(nn_mean),
            jn(nn_max),
            sites
                .iter()
                .enumerate()
                .map(|(k, s)| format!(
                    "    {{\"site\": {}, \"members\": [{}], \"families\": [{}], \"r\": {}, \"cos_theta_d\": {}, \"u_dot_b\": {}, \"q\": {}, \"e_ct\": {}, \"e_exact\": [{}], \"e_noct\": [{}], \"r_oo_bohr\": [{}], \"spread\": {}, \"loo\": {}, \"loo_miss\": {}, \"exact_sources\": [{}], \"sector_sources\": [{}]}}",
                    k,
                    s.members.iter().map(|&i| format!("\"{}\"", esc(&nodes[i].name))).collect::<Vec<_>>().join(", "),
                    s.members.iter().map(|&i| format!("\"{}\"", esc(&nodes[i].family))).collect::<Vec<_>>().join(", "),
                    jn(s.y[0]),
                    jn(s.y[1]),
                    jn(s.y[2]),
                    jn(s.y[3]),
                    jn(s.value),
                    s.members.iter().map(|&i| jn(nodes[i].e_exact)).collect::<Vec<_>>().join(", "),
                    s.members.iter().map(|&i| jn(nodes[i].e_noct)).collect::<Vec<_>>().join(", "),
                    s.members.iter().map(|&i| jd(nodes[i].r_oo)).collect::<Vec<_>>().join(", "),
                    jn(s.spread),
                    jn(loo[k]),
                    jn((loo[k] - s.value).abs()),
                    s.members.iter().map(|&i| format!("\"{}\"", esc(&nodes[i].exact_source))).collect::<Vec<_>>().join(", "),
                    s.members.iter().map(|&i| format!("\"{}\"", esc(&nodes[i].sector_source))).collect::<Vec<_>>().join(", ")
                ))
                .collect::<Vec<_>>()
                .join(",\n")
        ),
    )
    .unwrap();

    fs::write(
        out.join("gate.json"),
        format!(
            "{{\n  \"phase\": \"gate\", \"law_source\": \"{}\", \"threads\": {}, \"seconds\": {}, \"cpu_seconds\": {},\n  \
             \"g_t0\": {{\"pass\": {}, \"rule\": \"|table(y_i) - value_i| <= {} at every SITE\", \"worst\": {}, \"sites\": {}}},\n  \
             \"g_t1\": {{\"pass\": {}, \"rule\": \"|table(y_i) - e_ct_i| <= {} at every unmerged NODE; a merged node reads its site's mean and its miss is half that site's spread\", \"worst_unmerged\": {}, \"worst_merged\": {}, \"merged_groups\": {}, \"nodes\": {}}},\n  \
             \"loo\": {{\"rule\": \"each site removed, the table rebuilt from the rest, the site predicted\", \"worst\": {}, \"median\": {}, \"worst_sites\": [{}]}},\n  \
             \"head_to_head\": {{\"tolerance_rule\": \"max(0.25*|e_ct|, 5e-4)\", \"family_within\": {}, \"table_loo_within\": {}, \"of\": {}, \"family_worst\": {}, \"table_loo_worst\": {}, \"table_beats_family_on\": {}}},\n  \
             \"g_b3\": {{\"pass\": {}, \"rule\": \"worst relative |analytic - central difference| at h = {} over 5 atoms x 3 coordinates x {} nodes <= {}\", \"worst\": {}, \"at\": \"{}\", \"translation_worst\": {}}},\n  \
             \"g_c1\": {{\"pass\": {}, \"rule\": \"|engine e_seam - the runner's own sum| <= {} on every node, the table entering BOTH sides as ONE reading per UNORDERED pair of units, at that pair's own contact\", \"worst\": {}, \"units_two_everywhere\": {}}},\n  \
             \"reverse_leak\": {{\"rule\": \"what serving the OTHER direction as well would have added, i.e. the double count the serving rule refuses: measured on every node, never argued away\", \"worst\": {}, \"at\": \"{}\"}},\n  \
             \"g_b0\": {{\"pass\": {}, \"rule\": \"THE GATE is the SERVED walk: bounded(q_h, r_min, kT) on the full law with the transfer row the interpolant ITSELF at each walk radius, minimised over every angular geometry a real contact can present (|q| <= 1 - p^2, cos theta_d over its full physical [-1,1]), including the table's own behaviour below its innermost knot. The STRICT leg (the largest knot shape carried inward under exp(-c0 r)), the NEAR leg (the largest shape at or inside r_min_oh, carried the same way) and the LINEAR leg (CT-2's own letter) are walked beside it and none is hidden behind another. If the served walk refuses, the DECLARED FENCE is walked: the shape read at the innermost knot's separation for every r below it, the prefactor still running -- a stated fence, nothing chosen, nothing fit.\",\n  \"served_leg\": {}, \"served_pass\": {}, \"served_monotone_walk\": {}, \"angular_grid_points_per_axis\": {}, \"angular_grid_rule\": \"a coarse pass then a fine pass over one coarse cell; a minimum over a grid is an UPPER bound on the true minimum, so the resolution is stated with the depth\",\n  \"fence_r_clamp_bohr\": {}, \"fenced_leg\": {}, \"fenced_pass\": {}, \"fenced_monotone_walk\": {}, \"fence_force_jump_at_clamp\": {}, \"fence_price\": \"the energy is continuous at r_clamp and dS/dr is not; this is the jump in dE/dr there, on the deepest geometry at that radius\",\n  \"strict_leg\": {}, \"deepest_shape\": {}, \"near_leg\": {}, \"near_shape\": {}, \"near_pass\": {}, \"linear_leg\": {}, \"linear_pass\": {}, \"monotone_walk\": {}, \"q_h\": {}, \"kt\": {}, \"r_min\": [{}],\n  \"served_profile\": [{}]}},\n  \
             \"reach\": {{\"budget\": {}, \"law_bohr\": {}, \"table_bohr\": {}, \"together_bohr\": {}}},\n  \
             \"plant_i\": {{\"rule\": \"one knot moved by its own tolerance moves the served value there by exactly that\", \"site\": \"linear_R2.9\", \"carrier\": {}, \"before\": {}, \"after\": {}, \"miss\": {}, \"fires\": {}}},\n  \
             \"plant_ii\": {{\"rule\": \"P -> -P through SeamPlant::FlipChargeTransfer moves the engine's seam energy by 2*|table|\", \"delta\": {}, \"expected\": {}, \"miss\": {}, \"carrier\": {}, \"fires\": {}}},\n  \
             \"argmin_seam\": {{\"rule\": \"the contact is an argmin over the donor's two hydrogens and jumps where they tie; measured, not asserted small\", \"worst_jump\": {}, \"at\": \"{}\"}},\n  \
             \"c1\": {{\"branch\": \"{}\", \"branch_rule\": \"(a) the line at 2.7/2.9/3.1 A all within AND at least {} of 64 within; (b) the line within but fewer than that; (c) the clamp leaves the line unserved\",\n  \"remainder_rule\": \"dE_exact - [E_q(g) - E_q(40)]_engine - wall(g) - TABLE(g); the table is EXACT at the knots, so this remainder is the map's own residual and not a fit's\",\n  \"why_here\": \"the served walk on CT-2's contact terms was RUN and REFUSED before this freeze (g_b0.served_leg below): CT-2 clamped its amplitudes against its own transfer row, which is shallower than the table, so the clamp is re-run with the table held\",\n  \"tolerance_rule\": \"CT-2's own S1 tolerance max(0.25*|dE_exact|, 5e-4)\", \"within\": {}, \"of\": {}, \"line_within\": {}, \"line_of\": {}, \"worst_miss\": {}, \"worst_node\": \"{}\", \"misses\": [{}],\n  \"constrained\": {{\"p_ho\": {}, \"c_ho\": {}, \"p_hh\": {}, \"c_hh\": {}, \"clamped\": {{\"ho\": {}, \"hh\": {}}}, \"fixed_point_rounds\": {}, \"weighted_residual\": {}, \"grid_edge\": {{\"c_ho\": {}, \"c_hh\": {}}}}},\n  \"unconstrained\": {{\"p_ho\": {}, \"c_ho\": {}, \"p_hh\": {}, \"c_hh\": {}, \"classes_kept\": {{\"ho\": {}, \"hh\": {}}}, \"weighted_residual\": {}}}, \"residual_ratio\": {}, \"residual_ratio_named_above\": {},\n  \"clamp\": {{\"rule\": \"for each grid exponent pair each class's amplitude is the largest the SERVED walk admits, by bisection on [0, P_ls] to {} relative, the two classes iterated to a fixed point (at most {} rounds)\", \"held_inside_the_clamp\": \"FIELD-9's three walls; the transfer row at the table's deepest SERVED reading at each radius; the dispersion at an exact 0, because FIELD-6's rule fits C6 after both contacts\", \"grid_points\": {}, \"grid_points_with_a_clamp\": {}, \"boundedness_walks\": {}, \"grid_points_refused_at_zero\": {}, \"served_radii_memoised\": {}, \"fit_seconds\": {}}}}},\n  \
             \"g_b0w\": {{\"pass\": {}, \"rule\": \"bounded(q_h, r_min, kT) on the WHOLE served law - the table plus the re-fit contacts plus the walls plus the charges - returns None. LIQUID-2's admission gate. RUN even though the clamp makes it pass by construction, because a gate that is assumed is not a gate.\", \"leg\": {}, \"monotone_walk\": {}}},\n  \
             \"held_out\": {{\"name\": \"{}\", \"r_oo_angstrom\": {}, \"twist_degrees\": {}, \"tilt_degrees\": {}, \"donor_bend_degrees\": {}, \"r\": {}, \"cos_theta_d\": {}, \"u_dot_b\": {}, \"q\": {}, \"table\": {}, \"family\": {}, \"gap\": {}, \"s2_total_predicted\": {}, \"s2_field\": {}, \"s2_seam\": {}, \"s2_band\": {}, \"s2_band_absolute\": {}, \"s2_band_fractional\": {}, \"s2_worst_in_sample_fraction\": {}, \"s2_worst_in_sample_fraction_node\": \"{}\", \"s2_band_rule\": \"the re-fit law's own WORST in-sample miss over the sixty-four, taken the TIGHTER of two ways: absolute, and the worst miss as a FRACTION of that node's own |dE_exact| applied to the predicted total. Which is tighter is measured, not assumed.\", \"s2_comparative_bar\": {}, \"s2_comparative_bar_rule\": \"CT-2 S2's own letter max(0.25*|dE_exact|, 5e-4) on the PREDICTED total, replaced by the measured one at read; the programme's two landed forward predictions missed by 10.9 percent (CT-1 S2) and 22.9 percent (CT-2 S2)\", \"s2_branch_rule\": \"(a) inside the TIGHTER of band and comparative bar; (b) inside the looser only; (c) neither\", \"scaled_distance\": {}, \"nearest_site\": \"{}\", \"band_floor\": {}, \"band_cap\": {}, \"donor_centers\": [{}], \"acceptor_centers\": [{}], \"cross_ho_bohr\": [{}]}}\n}}\n",
            esc(&law.source),
            threads(),
            jd(seconds),
            jd(cpu),
            g_t0,
            jn(KNOT_TOL),
            jn(worst_knot),
            sites.len(),
            g_t1,
            jn(KNOT_TOL),
            jn(worst_node),
            jn(merged_worst),
            sites.iter().filter(|s| s.members.len() > 1).count(),
            nodes.len(),
            jn(loo_worst),
            jn(loo_median),
            loo_miss.iter().take(8).map(|(n, m)| format!("{{\"site\": \"{}\", \"miss\": {}}}", esc(n), jn(*m))).collect::<Vec<_>>().join(", "),
            fam_within,
            tab_within,
            nodes.len(),
            jn(fam_worst),
            jn(loo_miss.first().map(|x| x.1).unwrap_or(f64::NAN)),
            beats,
            g_b3,
            jn(FD_H),
            nodes.len(),
            jn(FD_TOL),
            jn(fd_worst),
            esc(&fd_node),
            jn(trans_worst),
            g_c1,
            jn(G_C1_TOL),
            jn(c1_worst),
            units_ok,
            jn(leak_worst),
            esc(&leak_node),
            g_b0,
            match &served { Some(s) => format!("\"{}\"", esc(s)), None => "null".to_string() },
            served.is_none(),
            match &hole_served { Some(s) => format!("\"{}\"", esc(s)), None => "null".to_string() },
            SERVED_GRID,
            jn(rc),
            match &fenced { Some(s) => format!("\"{}\"", esc(s)), None => "null".to_string() },
            fenced.is_none(),
            match &hole_fenced { Some(s) => format!("\"{}\"", esc(s)), None => "null".to_string() },
            jn(fence_jump),
            match &deep { Some(s) => format!("\"{}\"", esc(s)), None => "null".to_string() },
            jn(table.deepest_shape),
            match &near { Some(s) => format!("\"{}\"", esc(s)), None => "null".to_string() },
            jn(near_shape),
            near.is_none(),
            match &lin { Some(s) => format!("\"{}\"", esc(s)), None => "null".to_string() },
            lin.is_none(),
            match &law.m.hole_ct(law.q_h, &|r| table.deepest(r)) { Some(s) => format!("\"{}\"", esc(s)), None => "null".to_string() },
            jn(law.q_h),
            jn(law.kt),
            (0..3).map(|i| jn(law.r_min[i])).collect::<Vec<_>>().join(", "),
            profile
                .iter()
                .map(|(r, a, b, y)| format!(
                    "{{\"r\": {}, \"served\": {}, \"fenced\": {}, \"at\": {{\"cos_theta_d\": {}, \"u_dot_b\": {}, \"q\": {}}}}}",
                    jd(*r),
                    jn(*a),
                    jn(*b),
                    jn(y[1]),
                    jn(y[2]),
                    jn(y[3])
                ))
                .collect::<Vec<_>>()
                .join(", "),
            jn(REACH_BUDGET),
            jd(law.m.reach(REACH_BUDGET)),
            jd(table.reach(REACH_BUDGET)),
            jd(reach),
            jn(plant_i.0),
            jn(plant_i.1),
            jn(plant_i.2),
            jn(plant_i.3),
            plant_i.3 <= 1e-12 && plant_i.0 > 0.0,
            jn(plant_ii.0),
            jn(plant_ii.1),
            jn(plant_ii.2),
            jn(plant_ii.3),
            plant_ii.2 <= 1e-12 && plant_ii.3 >= CT_FLOOR,
            jn(tie_worst),
            esc(&tie_at),
            c1_branch,
            C1_MIN,
            c1_within,
            nn_nodes,
            c1_line,
            LINE_NODES.len(),
            jn(c1_fit_worst),
            esc(&c1_fit_worst_node),
            c1_misses.iter().map(|m| format!("\"{}\"", esc(m))).collect::<Vec<_>>().join(", "),
            jn(p_ho),
            jd(c_ho),
            jn(p_hh),
            jd(c_hh),
            c_clamped[0],
            c_clamped[1],
            c_rounds,
            jn(c_res),
            ci == 0 || ci == NC - 1,
            cj == 0 || cj == NC - 1,
            jn(uamp[0]),
            jd(cgrid(ui)),
            jn(uamp[1]),
            jd(cgrid(uj)),
            ukeep[0],
            ukeep[1],
            jn(u_res),
            jn(c1_ratio),
            jn(RATIO_NAMED),
            jn(CLAMP_REL),
            CLAMP_ROUNDS,
            NC * NC,
            clamped_points,
            clamp_calls,
            floor_refused,
            memo.borrow().len(),
            jd(c1_seconds),
            g_b0w,
            match &whole { Some(s) => format!("\"{}\"", esc(s)), None => "null".to_string() },
            match &whole_hole { Some(s) => format!("\"{}\"", esc(s)), None => "null".to_string() },
            esc(HO_NAME),
            jd(HO_ANGSTROM),
            jd(HO_TWIST_DEGREES),
            jd(HO_TILT_DEGREES),
            jd(HO_DONOR_DEGREES),
            jn(hy[0]),
            jn(hy[1]),
            jn(hy[2]),
            jn(hy[3]),
            jn(ho_table),
            jn(ho_family),
            jn((ho_table - ho_family).abs()),
            jn(s2_total),
            jn(s2_field),
            jn(s2_seam),
            jn(s2_band),
            jn(s2_band_abs),
            jn(s2_band_frac),
            jn(c1_fit_worst_frac),
            esc(&c1_fit_worst_frac_node),
            jn(s2_bar),
            jn(ho_dmin),
            esc(&ho_near),
            jn(nn_mean),
            jn(nn_max),
            centers_json(&ha),
            centers_json(&hb),
            list_json(&cross_classes(&ha, &hb).1)
        ),
    )
    .unwrap();

    eprintln!("G-T0 {}  worst {:.3e}", if g_t0 { "PASS" } else { "FAIL" }, worst_knot);
    eprintln!("G-T1 {}  worst unmerged {:.3e}, worst merged {:.3e}", if g_t1 { "PASS" } else { "FAIL" }, worst_node, merged_worst);
    eprintln!("LOO  worst {:.4} mHa, median {:.4} mHa", loo_worst * 1e3, loo_median * 1e3);
    eprintln!("head to head: family {fam_within}/{}, table (leave-one-out) {tab_within}/{}, table nearer on {beats}", nodes.len(), nodes.len());
    eprintln!("G-B3 {}  worst relative {:.3e} at {fd_node}", if g_b3 { "PASS" } else { "FAIL" }, fd_worst);
    eprintln!("G-C1 {}  worst {:.3e}; reverse leak worst {:.4} mHa at {leak_node}", if g_c1 { "PASS" } else { "FAIL" }, c1_worst, leak_worst * 1e3);
    eprintln!("G-B0 {}  SERVED walk (grid {SERVED_GRID}/axis, coarse+fine): {:?}", if g_b0 { "PASS (None)" } else { "REFUSED" }, served);
    eprintln!("     fenced walk (r_clamp = {:.6} bohr): {:?}  [force jump at the clamp {:.3e}]", rc, fenced, fence_jump);
    eprintln!("     strict leg (shape {:.4} carried inward): {:?}", table.deepest_shape, deep);
    eprintln!("     near leg (shape {:.4} at or inside r_min_oh = {:.3}): {:?}", near_shape, law.r_min[1], near);
    eprintln!("     linear leg: {lin:?}");
    for (r, a, b, y) in profile.iter() {
        eprintln!("     served at r = {r:.4}: {:.6e} Ha (fenced {:.6e}) at cos_td {:.4}, u.b {:.4}, q {:.4}", a, b, y[1], y[2], y[3]);
    }
    eprintln!("reach {:.3} bohr (law {:.3}, table {:.3})", reach, law.m.reach(REACH_BUDGET), table.reach(REACH_BUDGET));
    eprintln!("plant (i) carrier {:.3e}, miss {:.3e}", plant_i.0, plant_i.3);
    eprintln!("plant (ii) delta {:.6e} vs {:.6e}, miss {:.3e}, carrier {:.3e}", plant_ii.0, plant_ii.1, plant_ii.2, plant_ii.3);
    eprintln!("argmin jump worst {:.4} mHa ({tie_at})", tie_worst * 1e3);
    eprintln!("C1 branch ({c1_branch}): P_HO = {p_ho:.9e} at c = {c_ho:.2} (clamped {}), P_HH = {p_hh:.9e} at c = {c_hh:.2} (clamped {}); {c1_within} of {nn_nodes} within, line {c1_line}/3; worst {c1_fit_worst:.6e} at {c1_fit_worst_node}; ratio {c1_ratio:.6}; {c_rounds} round(s); {clamp_calls} walks over {} memoised radii in {c1_seconds:.1} s", c_clamped[0], c_clamped[1], memo.borrow().len());
    eprintln!("G-B0W {}  WHOLE served law: {:?}", if g_b0w { "PASS (None)" } else { "REFUSED" }, whole);
    eprintln!("S2 filed: total {s2_total:.6e} Ha (field {s2_field:.6e}, seam {s2_seam:.6e}); band {s2_band:.6e} = min(absolute {s2_band_abs:.6e}, fractional {s2_band_frac:.6e} = {c1_fit_worst_frac:.4} x |total|, worst fraction at {c1_fit_worst_frac_node}); CT-2's comparative bar {s2_bar:.6e}");
    eprintln!("held out {HO_NAME}: table {:.4} mHa, family {:.4} mHa, gap {:.4} mHa, d {:.4} in [{:.4}, {:.4}], nearest {ho_near}", ho_table * 1e3, ho_family * 1e3, (ho_table - ho_family).abs() * 1e3, ho_dmin, nn_mean, nn_max);
    // THE LAW CT-3 SERVES, as its own record — the artifact LIQUID-2 and `field3_hbonds.rs` read,
    // and the one `predict` rebuilds its prediction from, so no phase can drift from `gate`.
    fs::write(
        out.join("wall_ct3.json"),
        format!(
            "{{\n  \"a\": {}, \"b\": {}, \"p\": {}, \"c\": {}, \"c6\": {}, \"a_oh\": {}, \"b_oh\": {}, \"a_hh\": {}, \"b_hh\": {}, \"p_hh\": {}, \"c_hh\": {},\n  \
             \"p_ct\": {}, \"c_ct\": {}, \"m_ct\": 0, \"k_ct\": 0, \"lambda_ct\": {}, \"r_cut\": {}, \"ct_table_on\": true,\n  \
             \"transfer_rule\": \"channel 6 is the TABLE (ct_table.json), one reading per unordered pair of units at its shortest cross-unit H-O contact; p_ct and c_ct are held at an exact 0 so that nothing serves twice, and c0 below is the exponent the table divides out\",\n  \
             \"c0_per_bohr\": {}, \"table\": \"ct_table.json\", \"table_knots\": {},\n  \
             \"wall_rule\": \"FIELD-9's three walls are HELD (via ct2/wall_ct2.json); the two contact terms are CT-3's C1, re-fit on the map's remainder after the table and clamped under the SERVED boundedness walk; the dispersion is an exact 0 (FIELD-6's rule)\",\n  \
             \"c1_branch\": \"{}\", \"c1_within\": {}, \"c1_of\": {}, \"c1_line_within\": {}, \"c1_clamped\": {{\"ho\": {}, \"hh\": {}}}, \"c1_residual_ratio\": {},\n  \
             \"g_b0w_pass\": {}, \"r_min_oo\": {}, \"r_min_oh\": {}, \"r_min_hh\": {}, \"q_h\": {}, \"kt\": {},\n  \
             \"wall_source\": \"{}\", \"gate_source\": \"gate.json\"\n}}\n",
            jn(refit.a), jn(refit.b), jn(refit.p), jn(refit.c), jn(refit.c6),
            jn(refit.a_oh), jn(refit.b_oh), jn(refit.a_hh), jn(refit.b_hh), jn(refit.p_hh), jn(refit.c_hh),
            jn(0.0), jn(0.0), jn(0.0), jn(refit.r_cut),
            jn(law.c0), table.knots(),
            c1_branch, c1_within, nn_nodes, c1_line, c_clamped[0], c_clamped[1], jn(c1_ratio),
            g_b0w,
            jn(law.r_min[0]), jn(law.r_min[1]), jn(law.r_min[2]), jn(law.q_h), jn(law.kt),
            esc(&law.source),
        ),
    )
    .unwrap();
    fs::write(out.join("gate.done"), "done\n").unwrap();
}

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

/// The table, the law and the held-out geometry, rebuilt from the records in one place, so that
/// `predict`, `solve` and `read` cannot drift from `gate`.
fn assemble(out: &Path) -> (Law, CtTable, Fragment, Fragment, f64) {
    let (o, h) = (by_symbol("O").expect("O"), by_symbol("H").expect("H"));
    let mut law = load_law(out);
    // THE LAW IS CT-3'S, not CT-2's: the contact terms come from C1, which `gate` fit and wrote.
    // A prediction filed against CT-2's contacts would be a prediction of a law this campaign
    // does not serve, and the served walk refused that law (§0).
    let w3 = out.join("wall_ct3.json");
    let t3 = fs::read_to_string(&w3).unwrap_or_else(|e| panic!("{}: {e} — run `gate` first; C1's contact terms live there", w3.display()));
    law.m = SeamModel {
        p: json_num(&t3, "p"),
        c: json_num(&t3, "c"),
        p_hh: json_num(&t3, "p_hh"),
        c_hh: json_num(&t3, "c_hh"),
        ..law.m
    };
    law.source = format!("{} (contacts) + {} (walls)", w3.display(), law.source);
    let nodes = load_map(out, o, h);
    assert_eq!(nodes.len(), MAP_NODES, "the map is sixty-four nodes; {} were read", nodes.len());
    let (lo, rng) = axis_box(&nodes);
    let sites = sites_of(&nodes, lo, rng);
    let table = table_from(&sites, law.c0);
    // the leave-one-out worst, which is the band S1 is staked at — recomputed here rather than
    // read from `gate.json`, so a prediction cannot be filed against a stale gauge
    let mut loo_worst = 0.0f64;
    for k in 0..sites.len() {
        let rest: Vec<Site> = sites.iter().enumerate().filter(|(i, _)| *i != k).map(|(_, s)| Site { y: s.y, value: s.value, members: s.members.clone(), spread: s.spread }).collect();
        loo_worst = loo_worst.max((table_from(&rest, law.c0).eval(sites[k].y) - sites[k].value).abs());
    }
    let (ha, hb) = twist_and_bend(o, h, HO_ANGSTROM, HO_TWIST_DEGREES, HO_TILT_DEGREES, HO_DONOR_DEGREES);
    (law, table, ha, hb, loo_worst)
}

// ---------------------------------------------------------------------------- the predict phase

/// S1's prediction, FILED BEFORE THE SOLVE, with every part: the table's transfer term, the
/// family's on the same geometry, the full law's total, and both tolerances.
fn run_predict(out: &Path) {
    let (law, table, ha, hb, loo_worst) = assemble(out);
    let (hy, _) = node_contact(&ha, &hb);
    let ct_table = table.eval(hy);
    let ct_family = family_ct(&ha, &hb, law.p_ct, law.c0, law.m_ct);
    // the total under the full law, the engine's own rows, with the table serving the transfer
    let (total, field, seam) = engine_interaction(&ha, &hb, Some(law.m), SeamPlant::None, Some(&table));
    let (oo, ho, hh) = cross_classes(&ha, &hb);
    let tol_ct = (0.25 * ct_table.abs()).max(2e-4);
    let tol_total = (0.25 * total.abs()).max(5e-4);
    // S2's two bars, both from `gate`'s own record so a prediction cannot be filed against a
    // gauge this campaign did not measure
    let gt = fs::read_to_string(out.join("gate.json")).expect("gate.json (run `gate` first)");
    let s2_band_abs = json_num(&gt, "worst_miss");
    let s2_band_frac = json_num(&gt, "s2_worst_in_sample_fraction") * total.abs();
    let s2_band = s2_band_abs.min(s2_band_frac);
    let s2_bar = (S2_FRAC * total.abs()).max(S2_ABS);
    fs::write(
        out.join("prediction.json"),
        format!(
            "{{\n  \"node\": \"{}\", \"filed\": \"BEFORE the solve (CT3_PREREG §2 S1)\",\n  \
             \"r_oo_angstrom\": {}, \"twist_degrees\": {}, \"tilt_degrees\": {}, \"donor_bend_degrees\": {},\n  \
             \"coordinates\": {{\"r\": {}, \"cos_theta_d\": {}, \"u_dot_b\": {}, \"q\": {}}},\n  \
             \"e_ct_table\": {}, \"e_ct_family\": {}, \"gap\": {},\n  \
             \"family_rule\": \"CT-2's fitted family P = {} , c = {} , m = {}, k = 0, on all four cross-unit H-O pairs\",\n  \
             \"e_total_predicted\": {}, \"e_field\": {}, \"e_seam\": {},\n  \
             \"tolerance_ct\": {}, \"tolerance_ct_rule\": \"max(0.25*|E_CT predicted|, 2e-4), CT-2 S2's branch-(b) bar\",\n  \
             \"tolerance_total\": {}, \"tolerance_total_rule\": \"max(0.25*|E_total predicted|, 5e-4), CT-2 S2's branch-(a) bar\",\n  \
             \"loo_band\": {}, \"loo_band_rule\": \"the table's own WORST leave-one-out miss over its {} sites; S1 (a) requires the miss inside it\",\n  \
             \"s2_band\": {}, \"s2_band_absolute\": {}, \"s2_band_fractional\": {}, \"s2_band_rule\": \"the re-fit law's own worst in-sample miss over the sixty-four, the TIGHTER of absolute and fractional-applied-to-the-predicted-total (gate.json c1.worst_miss and s2_worst_in_sample_fraction)\",\n  \
             \"s2_comparative_bar\": {}, \"s2_comparative_bar_rule\": \"CT-2 S2's own letter max(0.25*|dE_exact|, 5e-4), on the PREDICTED total here and on the MEASURED one at read\",\n  \
             \"law\": \"{}\",\n  \
             \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}],\n  \
             \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
            esc(HO_NAME),
            jd(HO_ANGSTROM),
            jd(HO_TWIST_DEGREES),
            jd(HO_TILT_DEGREES),
            jd(HO_DONOR_DEGREES),
            jn(hy[0]),
            jn(hy[1]),
            jn(hy[2]),
            jn(hy[3]),
            jn(ct_table),
            jn(ct_family),
            jn((ct_table - ct_family).abs()),
            jn(law.p_ct),
            jn(law.c0),
            law.m_ct,
            jn(total),
            jn(field),
            jn(seam),
            jn(tol_ct),
            jn(tol_total),
            jn(loo_worst),
            table.knots(),
            jn(s2_band),
            jn(s2_band_abs),
            jn(s2_band_frac),
            jn(s2_bar),
            esc(&law.source),
            list_json(&oo),
            list_json(&ho),
            list_json(&hh),
            centers_json(&ha),
            centers_json(&hb),
        ),
    )
    .unwrap();
    fs::write(out.join("predict.done"), "done\n").unwrap();
    eprintln!("prediction.json filed: table {:.6e}, family {:.6e}, total {:.6e}; tolerances {:.3e} (CT) and {:.3e} (total); LOO band {:.3e}", ct_table, ct_family, total, tol_ct, tol_total, loo_worst);
}

// ------------------------------------------------------------------------------ the solve phase

/// The held-out node, solved exactly and in the closed sector — CT-2's `map_node` letter, on one
/// geometry. REFUSED unless `prediction.json` is already on disk (a prediction filed after its
/// measurement is not a prediction).
fn run_solve(out: &Path) {
    let pred = out.join("prediction.json");
    assert!(pred.exists(), "{}: file the prediction BEFORE the solve (CT3_PREREG §6)", pred.display());
    let node_path = out.join(format!("node_{HO_NAME}.json"));
    if node_path.exists() {
        eprintln!("{}: exists, skipped", node_path.display());
        return;
    }
    let (o, h) = (by_symbol("O").expect("O"), by_symbol("H").expect("H"));
    let (a, b) = twist_and_bend(o, h, HO_ANGSTROM, HO_TWIST_DEGREES, HO_TILT_DEGREES, HO_DONOR_DEGREES);
    // the closure identity first, before any solve (FIELD-9's rule, M-EMPTY-SECTOR)
    let scene = engine_dimer(&a, &b, None, SeamPlant::None, None);
    let units = scene.seam_work.units;
    let reading = scene.units_reading();
    let reading_json = reading.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(", ");
    drop(scene);
    if units < 2 {
        fs::write(
            out.join(format!("outside_{HO_NAME}.json")),
            format!(
                "{{\n  \"node\": \"{}\", \"units\": {units}, \"units_reading\": [{reading_json}], \"units_free_marker\": 4294967295, \"solved\": false,\n  \"rule\": \"a geometry at which the engine's closure reading finds fewer than two units is OUTSIDE the closure identity, is NOT solved, and is named (FIELD-9's rule; M-EMPTY-SECTOR)\",\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
                esc(HO_NAME),
                centers_json(&a),
                centers_json(&b)
            ),
        )
        .unwrap();
        eprintln!("{HO_NAME}: the closure reading finds {units} unit(s) — OUTSIDE the identity, NOT solved; S1 is VOID");
        fs::write(out.join("solve.done"), "outside\n").unwrap();
        return;
    }
    // the exact solve, checkpointed on its own so a death between the two repeats neither
    let exact_path = out.join(format!("exact_{HO_NAME}.json"));
    let (e_full, iters, residual, converged, ex_wall, ex_cpu) = if let Ok(t) = fs::read_to_string(&exact_path) {
        let rec = json_centers(&t, "donor_centers");
        let ok = rec.len() == a.centers.len() && rec.iter().zip(a.centers.iter()).all(|(p, q)| dist(p, q) < 1e-9);
        assert!(ok, "{}: the checkpoint's centers are not this geometry (M-STALE-INSTRUMENT)", exact_path.display());
        (json_num(&t, "e_total"), json_num(&t, "davidson_iters") as usize, json_num(&t, "residual"), t.contains("\"converged\": true"), json_num(&t, "wall_seconds"), json_num(&t, "cpu_seconds"))
    } else {
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let (e, it, res, conv) = fci_full_from_product(&a, &b);
        let wall = t0.elapsed().as_secs_f64();
        let cpu = cpu_seconds() - c0;
        fs::write(
            &exact_path,
            format!(
                "{{\n  \"node\": \"{}\", \"solver\": \"holon_chem::heitler_london::fci_full_from_product\",\n  \"e_total\": {}, \"davidson_iters\": {it}, \"residual\": {}, \"converged\": {conv}, \"capped\": {},\n  \"n_det\": {N_DET_DIMER}, \"wall_seconds\": {}, \"cpu_seconds\": {}, \"threads\": {},\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
                esc(HO_NAME),
                jn(e),
                jn(res),
                it >= DAVIDSON_CAP,
                jd(wall),
                jd(cpu),
                threads(),
                centers_json(&a),
                centers_json(&b)
            ),
        )
        .unwrap();
        (e, it, res, conv, wall, cpu)
    };
    let ex_price = PRICE_TENTH * (iters as f64) * SIGMA_PRICE_FLOOR;
    let t1 = Instant::now();
    let c1 = cpu_seconds();
    let r = fci_block_localised(&a, &b);
    let harness_wall = t1.elapsed().as_secs_f64();
    let sec_cpu = cpu_seconds() - c1;
    let de_exact = e_full - (r.e_a0 + r.e_b0);
    let e_ct = e_full - r.e_noct;
    let (oo, ho, hh) = cross_classes(&a, &b);
    let order_ok = e_full <= r.e_noct + ORDER_TOL && r.e_noct <= r.e_hl_undeformed + ORDER_TOL;
    fs::write(
        &node_path,
        format!(
            "{{\n  \"node\": \"{}\", \"family\": \"twistbent\", \"kind\": \"the held-out node: acceptor twisted {}° then tilted {}°, donor bent {}°, at {} Å\",\n  \"r_oo_angstrom\": {}, \"r_oo_bohr\": {}, \"twist_degrees\": {}, \"tilt_degrees\": {}, \"donor_bend_degrees\": {},\n  \"units\": {units}, \"units_reading\": [{reading_json}], \"units_free_marker\": 4294967295,\n  \"exact\": {{\"solver\": \"holon_chem::heitler_london::fci_full_from_product\", \"e_total\": {}, \"davidson_iters\": {iters}, \"residual\": {}, \"residual_bar\": {SECTOR_RESIDUAL_BAR:e}, \"converged\": {converged}, \"capped\": {}, \"n_det\": {N_DET_DIMER}, \"wall_seconds\": {}, \"cpu_seconds\": {}, \"price_expected_core_seconds\": {}, \"price_admitted\": {}}},\n  \"sector\": {{{}}},\n  \"e_a0\": {}, \"e_b0\": {}, \"de_exact\": {}, \"de_exact_rule\": \"E_exact(total) - (E_A0 + E_B0)\",\n  \"e_ct\": {}, \"e_ct_rule\": \"E_exact(total) - E_noCT(total)\", \"order_ok\": {order_ok}, \"order_tolerance\": {ORDER_TOL:e},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}],\n  \"threads\": {}, \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
            esc(HO_NAME),
            jd(HO_TWIST_DEGREES),
            jd(HO_TILT_DEGREES),
            jd(HO_DONOR_DEGREES),
            jd(HO_ANGSTROM),
            jd(HO_ANGSTROM),
            jd(dist(&a.centers[oxygen_of(&a)], &b.centers[oxygen_of(&b)])),
            jd(HO_TWIST_DEGREES),
            jd(HO_TILT_DEGREES),
            jd(HO_DONOR_DEGREES),
            jn(e_full),
            jn(residual),
            iters >= DAVIDSON_CAP,
            jd(ex_wall),
            jd(ex_cpu),
            jd(ex_price),
            ex_cpu >= ex_price,
            sector_body(&r, sec_cpu, harness_wall),
            jn(r.e_a0),
            jn(r.e_b0),
            jn(de_exact),
            jn(e_ct),
            list_json(&oo),
            list_json(&ho),
            list_json(&hh),
            threads(),
            centers_json(&a),
            centers_json(&b)
        ),
    )
    .unwrap();
    let _ = fs::remove_file(&exact_path);
    fs::write(out.join("solve.done"), "done\n").unwrap();
    eprintln!("{HO_NAME}: ΔE_exact {de_exact:.6e}, E_CT {e_ct:.6e} Ha | exact {iters} iters residual {residual:.1e} {ex_cpu:.0} core-s | sector {} iters residual {:.1e} {sec_cpu:.0} core-s | order {order_ok}", r.davidson_iters, r.residual);
}

// ------------------------------------------------------------------------------- the read phase

/// S1 scored: the prediction filed before the solve against the solve, and the FAMILY's miss on
/// the same geometry beside it — which is the kill (GANTT §Molecular: the table misses the
/// held-out node by more than the family did).
fn run_read(out: &Path) {
    let pt = fs::read_to_string(out.join("prediction.json")).expect("prediction.json (file it before the solve)");
    let nt = fs::read_to_string(out.join(format!("node_{HO_NAME}.json"))).expect("the held-out node's record");
    let e_ct = json_num(&nt, "e_ct");
    let de = json_num(&nt, "de_exact");
    let p_ct = json_num(&pt, "e_ct_table");
    let f_ct = json_num(&pt, "e_ct_family");
    let p_tot = json_num(&pt, "e_total_predicted");
    let band = json_num(&pt, "loo_band");
    let tol_ct = (0.25 * e_ct.abs()).max(2e-4);
    let tol_tot = (0.25 * de.abs()).max(5e-4);
    let miss_t = (p_ct - e_ct).abs();
    let miss_f = (f_ct - e_ct).abs();
    let miss_tot = (p_tot - de).abs();
    let branch = if miss_t < miss_f && miss_t <= band { "a" } else if miss_t < miss_f { "b" } else { "c" };
    // S2: the FULL dimer energy, against two derived bars, the tighter first — which is tighter
    // is measured, and at read the comparative bar is recomputed on the MEASURED total
    let s2_band = json_num(&pt, "s2_band");
    let s2_bar = (S2_FRAC * de.abs()).max(S2_ABS);
    let s2_tight = s2_band.min(s2_bar);
    let s2_loose = s2_band.max(s2_bar);
    let s2_branch = if miss_tot <= s2_tight { "a" } else if miss_tot <= s2_loose { "b" } else { "c" };
    fs::write(
        out.join("prediction_check.json"),
        format!(
            "{{\n  \"node\": \"{}\", \"branch\": \"{}\",\n  \
             \"branch_rule\": \"(a) the table is nearer than the family AND inside the leave-one-out band; (b) nearer but outside it; (c) the family is nearer — the kill FIRES\",\n  \
             \"e_ct_measured\": {}, \"e_ct_table_predicted\": {}, \"e_ct_family_predicted\": {},\n  \
             \"table_miss\": {}, \"family_miss\": {}, \"table_beats_family\": {},\n  \
             \"loo_band\": {}, \"inside_the_band\": {},\n  \
             \"tolerance_ct\": {}, \"table_within_tolerance\": {}, \"family_within_tolerance\": {},\n  \
             \"de_exact_measured\": {}, \"e_total_predicted\": {}, \"total_miss\": {}, \"tolerance_total\": {}, \"total_within\": {},\n  \
             \"s2\": {{\"branch\": \"{}\", \"branch_rule\": \"(a) inside the TIGHTER of the band and the comparative bar; (b) inside the looser only; (c) neither\", \"miss\": {}, \"band\": {}, \"comparative_bar\": {}, \"tighter\": {}, \"looser\": {}, \"which_is_tighter\": \"{}\"}},\n  \
             \"sector_residual\": {}, \"sector_converged\": {}, \"sector_dim\": {}, \"order_ok\": {}\n}}\n",
            esc(HO_NAME),
            branch,
            jn(e_ct),
            jn(p_ct),
            jn(f_ct),
            jn(miss_t),
            jn(miss_f),
            miss_t < miss_f,
            jn(band),
            miss_t <= band,
            jn(tol_ct),
            miss_t <= tol_ct,
            miss_f <= tol_ct,
            jn(de),
            jn(p_tot),
            jn(miss_tot),
            jn(tol_tot),
            miss_tot <= tol_tot,
            s2_branch,
            jn(miss_tot),
            jn(s2_band),
            jn(s2_bar),
            jn(s2_tight),
            jn(s2_loose),
            if s2_band <= s2_bar { "the band" } else { "CT-2's comparative bar" },
            jn(json_num(nt.split("\"sector\": {").nth(1).unwrap_or(&nt), "residual")),
            nt.split("\"sector\": {").nth(1).map(|s| s.contains("\"converged\": true")).unwrap_or(false),
            json_num(nt.split("\"sector\": {").nth(1).unwrap_or(&nt), "sector_dim") as usize,
            nt.contains("\"order_ok\": true"),
        ),
    )
    .unwrap();
    fs::write(out.join("read.done"), "done\n").unwrap();
    eprintln!("S1 branch ({branch}): measured {e_ct:.6e}; table {p_ct:.6e} (miss {miss_t:.3e}), family {f_ct:.6e} (miss {miss_f:.3e}); band {band:.3e}, tolerance {tol_ct:.3e}");
    eprintln!("S2 branch ({s2_branch}): measured total {de:.6e}; predicted {p_tot:.6e} (miss {miss_tot:.3e}); band {s2_band:.3e}, comparative bar {s2_bar:.3e}, tighter {s2_tight:.3e}");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let what = args.get(1).map(String::as_str).unwrap_or("gate");
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/ct3".to_string()));
    fs::create_dir_all(&out).expect("out");
    match what {
        "gate" => run_gate(&out),
        "predict" => run_predict(&out),
        "solve" => run_solve(&out),
        "read" => run_read(&out),
        other => eprintln!("unknown phase {other} (gate | predict | solve | read)"),
    }
}
