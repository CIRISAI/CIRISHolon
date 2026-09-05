//! FIELD-7's harvest (`conformance/water_observatory/FIELD7_PREREG.md` §0, §2, §5, §6): the
//! wall on the three cross-unit ATOM-PAIR classes, harvested over ORIENTATIONS from the
//! undeformed Heitler–London referee; the remainder transferred into the CONTACT term (one
//! attractive exponential on the cross-unit H–O contact) over every exact geometry of record;
//! and a TWISTED hydrogen bond predicted forward before it is solved.
//!
//! FIELD-6's runner with four changes and nothing else: the reading set is 24 ORIENTATIONS
//! (four separations × six acceptor tilts) rather than six collinear nodes; the wall is a
//! THREE-CLASS model (O–O, H–O, H–H) fit by a grid over the three exponents with non-negative
//! amplitudes rather than one log-linear line; the contact term is RE-FIT here (FIELD-5's
//! `(P, c)` is not reused) on the nine exact geometries with the wall held; and the held-out
//! geometry is a TWIST — a kind no fit point has (§4, M-UNTESTED-GAP).
//!
//! ```text
//! cargo run --release -p holon-render --example field7_harvest -- orient  [OUT_DIR]
//! cargo run --release -p holon-render --example field7_harvest -- predict [OUT_DIR]
//! ```
//!
//! `orient`: the 24 undeformed readings, W0, the three-class wall fit (S1), plant (ii), the
//! contact fit (C1) and dispersion, `wall7.json`, G-C1 and plant (i) by the engine, and
//! `prediction.json` for the twisted node written BEFORE that node is solved. `predict`:
//! refuses without `prediction.json`, solves the twisted node exactly, reads the undeformed
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

/// EMBED-1's water pins — the same numbers FIELD-3's … FIELD-6's runners carry.
const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;

/// §0's declared ORIENTATION set: four separations × six acceptor tilts about the x-axis
/// through the acceptor's OWN oxygen, the donor untouched. 24 geometries.
const ORIENT_R: [f64; 4] = [2.7, 2.9, 3.1, 3.4];
const ORIENT_TILT: [f64; 6] = [0.0, 30.0, 60.0, 90.0, 120.0, 180.0];
const N_R: usize = ORIENT_R.len();
const N_T: usize = ORIENT_TILT.len();
const N_ORIENT: usize = N_R * N_T;

/// FIELD-3's six linear nodes (Å), SHORTEST FIRST — the exact record the contact term is fit on.
const NODES_ANGSTROM: [f64; 6] = [2.5, 2.7, 2.9, 3.1, 3.4, 3.7];
/// The OUTER linear nodes begin here: 2.9, 3.1, 3.4, 3.7 Å — FIELD-6's dispersion set.
const OUTER_FROM: usize = 2;
/// The node plant (i) is read at.
const REF_ANGSTROM: f64 = 2.9;
/// The three BENT/FLIPPED exact geometries of record (§0).
const TILT5_ANGSTROM: f64 = 2.9;
const TILT5_DEGREES: f64 = 30.0;
const TILT6_ANGSTROM: f64 = 3.1;
const TILT6_DEGREES: f64 = 45.0;
const FLIPPED_ANGSTROM: f64 = 3.4;

/// S2's held-out geometry: the linear dimer at 3.0 Å, the acceptor TWISTED 90° about the
/// O···O axis (z) through its own oxygen and THEN tilted 60° about its own x-axis.
const TWIST_ANGSTROM: f64 = 3.0;
const TWIST_DEGREES: f64 = 90.0;
const TWIST_TILT_DEGREES: f64 = 60.0;

/// The separation at which the acceptor is "away" (bohr): the engine's reference on both
/// sides of G-C1 and of every `E_q` difference.
const FAR_BOHR: f64 = 40.0;

/// The residual bar every exact solve must meet (EMBED-3's).
const RESIDUAL_BAR: f64 = 1e-9;
/// The reading floor on every harvested reading (M-FLOOR-UNSTAKED).
const R_FLOOR: f64 = 1e-6;

/// The three-exponent grid (per bohr): `0.50 ..= 4.00` step `0.05` — 71 values, 71³ triples.
const NB: usize = 71;
/// The contact term's `c`-grid (per bohr): `0.50 ..= 4.00` step `0.01` — 351 values.
const NC: usize = 351;

/// S1's tolerance, per geometry: `max(0.05·E_exch, 1e-4)` hartree (FIELD-6's derived rule).
const WALL_TOL_FRAC: f64 = 0.05;
const WALL_TOL_ABS: f64 = 1e-4;
/// S1 (b)'s floor: at least this many of the 24 within tolerance, else (c) VOID.
const S1_B_MIN: usize = 18;

/// C1's tolerance, per exact point: `max(0.25·|ΔE_exact|, 5e-4)`; at least seven of nine.
const C1_FRAC: f64 = 0.25;
const C1_ABS: f64 = 5e-4;
const C1_MIN: usize = 7;

/// S2's tolerance: `max(0.25·|ΔE_exact|, 5e-4)`.
const PRED_FRAC: f64 = 0.25;
const PRED_ABS: f64 = 5e-4;

/// The band the remainder's log-log slope must lie in for `C₆` to transfer (FIELD-6's rule).
const SLOPE_LO: f64 = -8.0;
const SLOPE_HI: f64 = -4.0;

/// W0: the undeformed product's norm window, and the determinant count §2 names.
const NORM_LO: f64 = 0.8;
const NORM_HI: f64 = 1.0;
/// §2's stated count. NOTE (reported, never silently repaired): the UNDEFORMED referee's
/// product state is spread over the FULL dimer space by the Löwdin transform, so its
/// `nonzero_dets` is the space's `n_det`; `n_det_a · n_det_b` is the count of monomer-product
/// determinants the state is BUILT from, and it is that product which is 194,481 (FIELD-6's
/// own records carry it as `nonzero_dets_expected`). Both legs are measured and recorded.
const NONZERO_DETS_STAKED: usize = 194_481;
/// The determinant count FIELD-3's supermolecule carries (EXACT).
const N_DET_DIMER: usize = 1_002_001;

/// G-C1's tolerance, and plant (i)'s carrier.
const G_C1_TOL: f64 = 1e-10;
const PLANT_I_CARRIER: f64 = 1e-4;
/// Plant (ii): the O–O-only wall must FAIL S1's tolerance on at least this many of the 24,
/// and its carrier is the orientation contrast at fixed `R_OO`.
const PLANT_II_MIN_FAIL: usize = 6;
const PLANT_II_CARRIER_RATIO: f64 = 2.0;

/// M-CHEAPER-THAN-ITS-PRICE: FIELD-6 measured 55–60 core-seconds per undeformed reading. A
/// reading under a TENTH of 55 is recorded as under its price.
const HL_PRICE_TENTH_CORE_S: f64 = 5.5;
/// S2's exact solve price band (§2).
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

/// S2's TWISTED geometry: the linear dimer with the acceptor rotated by `twist_degrees` about
/// the O···O axis (z) through its own oxygen, and THEN tilted by `tilt_degrees` about its own
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

fn centers_json(f: &Fragment) -> String {
    f.centers.iter().map(|c| format!("[{:.10}, {:.10}, {:.10}]", c[0], c[1], c[2])).collect::<Vec<_>>().join(", ")
}

fn exit_name(e: &SolveExit) -> String {
    format!("{e:?}")
}

fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Every CROSS-UNIT pair distance (bohr) by class: `(O–O, H–O, H–H)`. For a water dimer that
/// is one, four and four — the same enumeration the engine's seam loop makes.
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

// ------------------------------------------------------------------------------ the engine

/// FIELD-4's `engine_dimer` verbatim: an open box, the field on with the pin charge, the seam
/// model and its plant installed, forces computed once so the closure assignment and the rows
/// are read.
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

/// The formula side of G-C1, class by class: the contact term over the cross-unit H–O pairs,
/// the O–O wall, the dispersion, and FIELD-7's two new walls.
fn formula_terms(a: &Fragment, b: &Fragment, m: &SeamModel) -> (f64, f64, f64, f64, f64) {
    let (oo, ho, hh) = cross_classes(a, b);
    let pen: f64 = ho.iter().map(|&r| m.penetration(r)).sum();
    let w_oo: f64 = oo.iter().map(|&r| m.wall(r)).sum();
    let disp: f64 = oo.iter().map(|&r| m.dispersion(r)).sum();
    let w_oh: f64 = ho.iter().map(|&r| m.wall_oh(r)).sum();
    let w_hh: f64 = hh.iter().map(|&r| m.wall_hh(r)).sum();
    (pen, w_oo, disp, w_oh, w_hh)
}

// ------------------------------------------------------------------------- the exact solve

/// One exact node: the supermolecule, the monomer references, the record. FIELD-3's
/// `solve_node` verbatim, so the held-out node's record is the same object FIELD-3 wrote.
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
    fs::write(
        &path,
        format!(
            "{{\n  \"node\": \"{name}\", \"r_oo_angstrom\": {r_oo_angstrom:.3}, \"r_oo_bohr\": {:.6},\n  \"n_det\": {}, \"e_super\": {:.12e}, \"e_a0\": {:.12e}, \"e_b0\": {:.12e}, \"de_exact\": {:.12e},\n  \"davidson_iters\": {}, \"residual\": {:.3e}, \"exit\": \"{}\", \"converged\": {converged},\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}],\n  \"wall_seconds\": {wall:.1}, \"cpu_seconds\": {cpu:.1}, \"threads\": {}, \"price_node\": {price}\n}}\n",
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
        "  {name}: R_OO {r_oo_angstrom:.1} Å, {} dets, ΔE_exact {de:+.6e} Ha, {} iters, residual {:.1e}, exit {}, wall {wall:.0} s, {cpu:.0} core-s",
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

// --------------------------------------------------------------- the three-class wall fit

/// The exponent grid, per bohr: `0.50 ..= 4.00` step `0.05`, built so the ends are exact.
fn bgrid(i: usize) -> f64 {
    ((10 + i) as f64) * 0.05
}
/// The contact grid, per bohr: `0.50 ..= 4.00` step `0.01`, built so the ends are exact.
fn cgrid(i: usize) -> f64 {
    ((50 + i) as f64) * 0.01
}

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

/// §0's non-negativity rule: the weighted least-squares amplitudes; if any is negative, DROP
/// that class (amplitude an exact `0.0`) and refit the rest, repeating until all are
/// non-negative. When several are negative at once the MOST negative is dropped first — the
/// rule applied to one class at a time, which terminates in at most three rounds.
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

// ------------------------------------------------------------------------ the orient phase

struct ONode {
    r_ang: f64,
    tilt_deg: f64,
    r_oo_bohr: f64,
    oo: Vec<f64>,
    ho: Vec<f64>,
    hh: Vec<f64>,
    hl: HlReading,
    cpu: f64,
}

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
}

/// The nine exact geometries of record (§0), each rebuilt here and CHECKED against the record
/// it is read from (M-STALE-INSTRUMENT): the six linear nodes, the 30°-bent bond at 2.9 Å, the
/// 45°-bent bond at 3.1 Å, the flipped dimer at 3.4 Å.
fn exact_records(out: &Path, o: Species, h: Species) -> Result<Vec<ENode>, Vec<String>> {
    let mut want: Vec<(String, &'static str, f64, PathBuf, Fragment, Fragment, bool)> = Vec::new();
    let f3 = sibling(out, "field3");
    let f4 = sibling(out, "field4");
    let f5 = sibling(out, "field5");
    let f6 = sibling(out, "field6");
    for (i, &r) in NODES_ANGSTROM.iter().enumerate() {
        let (a, b) = linear(o, h, r);
        want.push((format!("linear_R{r:.1}"), "linear", r, f3.join(format!("linear_R{r:.1}.json")), a, b, i >= OUTER_FROM));
    }
    let (a5, b5) = tilted(o, h, TILT5_ANGSTROM, TILT5_DEGREES);
    want.push((format!("tilted_R{TILT5_ANGSTROM:.1}"), "bent 30°", TILT5_ANGSTROM, f5.join(format!("tilted_R{TILT5_ANGSTROM:.1}.json")), a5, b5, false));
    let (a6, b6) = tilted(o, h, TILT6_ANGSTROM, TILT6_DEGREES);
    want.push((
        format!("tilted{TILT6_DEGREES:.0}_R{TILT6_ANGSTROM:.1}"),
        "bent 45°",
        TILT6_ANGSTROM,
        f6.join(format!("tilted{TILT6_DEGREES:.0}_R{TILT6_ANGSTROM:.1}.json")),
        a6,
        b6,
        false,
    ));
    let (a4, b4) = flipped(o, h, FLIPPED_ANGSTROM);
    want.push((format!("flipped_R{FLIPPED_ANGSTROM:.1}"), "flipped 180°", FLIPPED_ANGSTROM, f4.join(format!("flipped_R{FLIPPED_ANGSTROM:.1}.json")), a4, b4, false));

    let mut missing: Vec<String> = Vec::new();
    let mut nodes: Vec<ENode> = Vec::new();
    for (name, kind, r_ang, path, a, b, outer) in want {
        let Ok(t) = fs::read_to_string(&path) else {
            missing.push(path.display().to_string());
            continue;
        };
        let de = json_num(&t, "de_exact");
        if !de.is_finite() {
            missing.push(format!("{} (no de_exact)", path.display()));
            continue;
        }
        // the record's geometry must be the geometry rebuilt here, to the record's precision
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
        nodes.push(ENode { name, kind, r_ang, r_oo_bohr: cross_oo(&a, &b), de_exact: de, a, b, oo, ho, hh, outer_linear: outer });
    }
    if missing.is_empty() {
        Ok(nodes)
    } else {
        Err(missing)
    }
}

fn run_orient(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    eprintln!("FIELD-7 orient — the UNDEFORMED Heitler–London referee over {N_ORIENT} ORIENTATIONS, {} threads", threads());

    // The nine exact records are read FIRST: the contact term is fit on them, and a harvest
    // that cannot make wall7.json's `(P, c)` must not spend the readings.
    let exact = match exact_records(out, o, h) {
        Ok(v) => v,
        Err(missing) => {
            eprintln!("REFUSED — the contact term is staked on NINE exact geometries (§0) and these are not on disk:");
            for m in &missing {
                eprintln!("  {m}");
            }
            eprintln!("Nothing written. Re-run `orient` when the records exist.");
            std::process::exit(3);
        }
    };
    eprintln!("the nine exact geometries of record, each rebuilt here and matched to its record's centers:");
    for e in &exact {
        eprintln!("  {} ({}): R_OO {:.4} bohr, ΔE_exact {:+.6e} Ha", e.name, e.kind, e.r_oo_bohr, e.de_exact);
    }

    // ------------------------------------------------------- the 24 orientation readings
    let mut nodes: Vec<ONode> = Vec::with_capacity(N_ORIENT);
    for &r in ORIENT_R.iter() {
        for &deg in ORIENT_TILT.iter() {
            let (a, b) = tilted(o, h, r, deg);
            let r_oo = cross_oo(&a, &b);
            assert!(
                (r_oo - r * ANGSTROM_TO_BOHR).abs() < 1e-9,
                "the tilt is about the acceptor's own oxygen: R_OO must be unchanged ({r_oo:.9} vs {:.9})",
                r * ANGSTROM_TO_BOHR
            );
            let (oo, ho, hh) = cross_classes(&a, &b);
            let t0 = Instant::now();
            let c0 = cpu_seconds();
            let hl = heitler_london_undeformed(&a, &b);
            let wall_s = t0.elapsed().as_secs_f64();
            let cpu = cpu_seconds() - c0;
            let product_dets = hl.n_det_a * hl.n_det_b;
            fs::write(
                out.join(format!("orient_R{r:.1}_t{deg:.0}.json")),
                format!(
                    "{{\n  \"node\": \"orient_R{r:.1}_t{deg:.0}\", \"r_oo_angstrom\": {r:.3}, \"r_oo_bohr\": {r_oo:.6}, \"tilt_degrees\": {deg:.1}, \"tilt_axis\": \"x, through the acceptor's own oxygen\",\n  \"referee\": \"heitler_london_undeformed\",\n  \"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"e_exch\": {:+.12e},\n  \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"n_det_a\": {}, \"n_det_b\": {}, \"product_dets\": {product_dets}, \"nonzero_dets_staked\": {NONZERO_DETS_STAKED},\n  \"s_cross_max\": {:.12e}, \"sigma_seconds\": {:.3}, \"wall_seconds\": {wall_s:.3}, \"cpu_seconds\": {cpu:.3}, \"threads\": {},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}],\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
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
                    hl.s_cross_max,
                    hl.sigma_seconds,
                    threads(),
                    list_json(&oo),
                    list_json(&ho),
                    list_json(&hh),
                    centers_json(&a),
                    centers_json(&b),
                ),
            )
            .unwrap();
            eprintln!(
                "  R_OO {r:.1} Å, tilt {deg:>5.1}°: E_exch {:+.6e} Ha, norm {:.12}, nonzero {} (product {product_dets}), σ {:.1} s, {cpu:.1} core-s",
                hl.e_exch, hl.norm, hl.nonzero_dets, hl.sigma_seconds
            );
            nodes.push(ONode { r_ang: r, tilt_deg: deg, r_oo_bohr: r_oo, oo, ho, hh, hl, cpu });
        }
    }
    let at = |ri: usize, ti: usize| -> &ONode { &nodes[ri * N_T + ti] };

    // ------------------------------------------------- W0: the readings are what they say
    let norm_ok = nodes.iter().all(|n| n.hl.norm > NORM_LO && n.hl.norm <= NORM_HI);
    let count_ok_literal = nodes.iter().all(|n| n.hl.nonzero_dets == NONZERO_DETS_STAKED);
    let count_ok_product = nodes.iter().all(|n| n.hl.n_det_a * n.hl.n_det_b == NONZERO_DETS_STAKED);
    let floor_ok = nodes.iter().all(|n| n.hl.e_exch > R_FLOOR);
    let mut monotone_ok = true;
    let mut monotone_breaks: Vec<String> = Vec::new();
    for ti in 0..N_T {
        for ri in 1..N_R {
            if !(at(ri, ti).hl.e_exch <= at(ri - 1, ti).hl.e_exch) {
                monotone_ok = false;
                monotone_breaks.push(format!(
                    "{{\"tilt_degrees\": {:.1}, \"r_from\": {:.1}, \"r_to\": {:.1}, \"e_exch_from\": {:+.12e}, \"e_exch_to\": {:+.12e}}}",
                    ORIENT_TILT[ti],
                    ORIENT_R[ri - 1],
                    ORIENT_R[ri],
                    at(ri - 1, ti).hl.e_exch,
                    at(ri, ti).hl.e_exch
                ));
            }
        }
    }
    let w0 = norm_ok && count_ok_literal && floor_ok && monotone_ok;
    let w0_product_reading = norm_ok && count_ok_product && floor_ok && monotone_ok;
    let norm_lo_seen = nodes.iter().map(|n| n.hl.norm).fold(f64::INFINITY, f64::min);
    let norm_hi_seen = nodes.iter().map(|n| n.hl.norm).fold(f64::NEG_INFINITY, f64::max);
    eprintln!("\nW0 — the readings are what they say:");
    eprintln!("  norm: every ⟨v|v⟩ in ({NORM_LO}, {NORM_HI}] — lowest {norm_lo_seen:.12}, highest {norm_hi_seen:.12} → {}", if norm_ok { "PASS" } else { "FAIL" });
    eprintln!(
        "  count: nonzero_dets == {NONZERO_DETS_STAKED} at every reading → {} (the undeformed product is spread over the full space by the Löwdin transform; its nonzero count is n_det = {}, and n_det_a·n_det_b == {NONZERO_DETS_STAKED} → {})",
        if count_ok_literal { "PASS" } else { "FAIL" },
        nodes[0].hl.n_det,
        if count_ok_product { "PASS" } else { "FAIL" }
    );
    eprintln!("  floor: E_exch > {R_FLOOR:e} at every reading → {}", if floor_ok { "PASS" } else { "FAIL" });
    eprintln!("  order: E_exch non-increasing in R_OO along each tilt → {} ({} breaks)", if monotone_ok { "PASS" } else { "FAIL" }, monotone_breaks.len());
    eprintln!("W0 → {} (as staked); with the product-space reading of the count leg → {}", if w0 { "PASS" } else { "FAIL" }, if w0_product_reading { "PASS" } else { "FAIL" });
    let price_ok = nodes.iter().all(|n| n.cpu >= HL_PRICE_TENTH_CORE_S);
    eprintln!(
        "M-CHEAPER-THAN-ITS-PRICE (recorded): every reading at or above a tenth of its {HL_PRICE_TENTH_CORE_S:.1} core-second price: {price_ok} (cheapest {:.1} core-s, dearest {:.1})",
        nodes.iter().map(|n| n.cpu).fold(f64::INFINITY, f64::min),
        nodes.iter().map(|n| n.cpu).fold(0.0f64, f64::max)
    );

    // ------------------------------------------------ the three-class wall over the 24
    let y: Vec<f64> = nodes.iter().map(|n| n.hl.e_exch).collect();
    let w: Vec<f64> = y.iter().map(|v| 1.0 / (v * v)).collect();
    let tol: Vec<f64> = y.iter().map(|v| (WALL_TOL_FRAC * v).max(WALL_TOL_ABS)).collect();
    let syy: f64 = (0..N_ORIENT).map(|g| w[g] * y[g] * y[g]).sum();

    // S_class(g) per grid value, precomputed: the triple loop then touches no geometry
    let mut soo = vec![0.0f64; NB * N_ORIENT];
    let mut soh = vec![0.0f64; NB * N_ORIENT];
    let mut shh = vec![0.0f64; NB * N_ORIENT];
    for i in 0..NB {
        let b = bgrid(i);
        for (g, n) in nodes.iter().enumerate() {
            soo[i * N_ORIENT + g] = n.oo.iter().map(|&r| (-b * r).exp()).sum();
            soh[i * N_ORIENT + g] = n.ho.iter().map(|&r| (-b * r).exp()).sum();
            shh[i * N_ORIENT + g] = n.hh.iter().map(|&r| (-b * r).exp()).sum();
        }
    }
    // the weighted Gram entries: diagonals and right-hand sides per grid value, the three
    // cross blocks per grid PAIR. Every one of the 71³ triples then costs a 3×3 solve.
    let mut a00 = vec![0.0f64; NB];
    let mut a11 = vec![0.0f64; NB];
    let mut a22 = vec![0.0f64; NB];
    let mut v0 = vec![0.0f64; NB];
    let mut v1 = vec![0.0f64; NB];
    let mut v2 = vec![0.0f64; NB];
    for i in 0..NB {
        for g in 0..N_ORIENT {
            let (p, q, s) = (soo[i * N_ORIENT + g], soh[i * N_ORIENT + g], shh[i * N_ORIENT + g]);
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
            for g in 0..N_ORIENT {
                x01 += w[g] * soo[i * N_ORIENT + g] * soh[j * N_ORIENT + g];
                x02 += w[g] * soo[i * N_ORIENT + g] * shh[j * N_ORIENT + g];
                x12 += w[g] * soh[i * N_ORIENT + g] * shh[j * N_ORIENT + g];
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
    eprintln!(
        "\nthe three-class wall over {N_ORIENT} orientations — {} triples on 0.50..=4.00 step 0.05, weighted (1/E_exch²) least squares with amplitudes constrained non-negative, in {:.1} s:",
        NB * NB * NB,
        t_fit.elapsed().as_secs_f64()
    );
    eprintln!("  b_OO = {b_oo:.2} /bohr, A_OO = {a_oo:.9e} Ha (kept: {})", act[0]);
    eprintln!("  b_OH = {b_oh:.2} /bohr, A_OH = {a_oh:.9e} Ha (kept: {})", act[1]);
    eprintln!("  b_HH = {b_hh:.2} /bohr, A_HH = {a_hh:.9e} Ha (kept: {})", act[2]);
    eprintln!("  weighted residual {best_r:.9e}");
    let wall3 = |n: &ONode| -> f64 {
        a_oo * n.oo.iter().map(|&r| (-b_oo * r).exp()).sum::<f64>()
            + a_oh * n.ho.iter().map(|&r| (-b_oh * r).exp()).sum::<f64>()
            + a_hh * n.hh.iter().map(|&r| (-b_hh * r).exp()).sum::<f64>()
    };
    let mut within = 0usize;
    let mut misses: Vec<String> = Vec::new();
    let mut miss_lines: Vec<String> = Vec::new();
    eprintln!("\n| R (Å) | tilt (°) | E_exch (Ha) | model (Ha) | miss (Ha) | miss/tol | within |");
    for (g, n) in nodes.iter().enumerate() {
        let m = wall3(n);
        let miss = (m - y[g]).abs();
        let ok = miss <= tol[g];
        if ok {
            within += 1;
        } else {
            misses.push(format!("(R = {:.1} Å, tilt = {:.0}°)", n.r_ang, n.tilt_deg));
            miss_lines.push(format!(
                "{{\"r_angstrom\": {:.1}, \"tilt_degrees\": {:.1}, \"e_exch\": {:+.12e}, \"model\": {:+.12e}, \"miss\": {miss:.12e}, \"tolerance\": {:.12e}, \"miss_over_tolerance\": {:.6}}}",
                n.r_ang,
                n.tilt_deg,
                y[g],
                m,
                tol[g],
                miss / tol[g]
            ));
        }
        eprintln!("| {:.1} | {:.0} | {:+.6e} | {:+.6e} | {:+.6e} | {:.4} | {} |", n.r_ang, n.tilt_deg, y[g], m, m - y[g], miss / tol[g], ok);
    }
    let s1_branch = if within == N_ORIENT {
        "a"
    } else if within >= S1_B_MIN {
        "b"
    } else {
        "c"
    };
    eprintln!(
        "S1: {within} of {N_ORIENT} within max({WALL_TOL_FRAC}·E_exch, {WALL_TOL_ABS:e}) → branch ({s1_branch}) — {}",
        match s1_branch {
            "a" => "the three-class wall carries exchange across orientation at this level".to_string(),
            "b" => format!("transferred, the {} misses reported: {}", misses.len(), misses.join(", ")),
            _ => format!("VOID: the arms do not run. {} misses: {}", misses.len(), misses.join(", ")),
        }
    );

    // ---------------------------------- plant (ii): the wall on the OXYGENS ONLY
    let (mut p2_r, mut p2_i, mut p2_amp) = (f64::INFINITY, 0usize, 0.0f64);
    for i in 0..NB {
        let x = if a00[i] > 0.0 { (v0[i] / a00[i]).max(0.0) } else { 0.0 };
        let r = (syy - 2.0 * x * v0[i] + x * x * a00[i]).max(0.0);
        if r < p2_r {
            p2_r = r;
            p2_i = i;
            p2_amp = x;
        }
    }
    let p2_b = bgrid(p2_i);
    let mut p2_fail = 0usize;
    let mut p2_fail_lines: Vec<String> = Vec::new();
    for (g, n) in nodes.iter().enumerate() {
        let m = p2_amp * n.oo.iter().map(|&r| (-p2_b * r).exp()).sum::<f64>();
        let miss = (m - y[g]).abs();
        if miss > tol[g] {
            p2_fail += 1;
            p2_fail_lines.push(format!(
                "{{\"r_angstrom\": {:.1}, \"tilt_degrees\": {:.1}, \"e_exch\": {:+.12e}, \"model\": {:+.12e}, \"miss\": {miss:.12e}, \"tolerance\": {:.12e}}}",
                n.r_ang, n.tilt_deg, y[g], m, tol[g]
            ));
        }
    }
    let ri34 = ORIENT_R.iter().position(|&r| (r - 3.4).abs() < 1e-9).expect("the 3.4 Å row");
    let ti180 = ORIENT_TILT.iter().position(|&t| (t - 180.0).abs() < 1e-9).expect("the 180° column");
    let ti0 = ORIENT_TILT.iter().position(|&t| t == 0.0).expect("the 0° column");
    let carrier_ii = at(ri34, ti180).hl.e_exch / at(ri34, ti0).hl.e_exch;
    let carrier_ii_ok = carrier_ii >= PLANT_II_CARRIER_RATIO;
    let plant_ii_fires = p2_fail >= PLANT_II_MIN_FAIL && carrier_ii_ok;
    eprintln!("\nplant (ii) — the wall on the OXYGENS ONLY, the same rule on the same 24:");
    eprintln!("  b_OO = {p2_b:.2} /bohr, A_OO = {p2_amp:.9e} Ha, weighted residual {p2_r:.9e}");
    eprintln!(
        "  fails the tolerance on {p2_fail} of {N_ORIENT} (needs ≥ {PLANT_II_MIN_FAIL}); carrier E_exch(3.4 Å, 180°)/E_exch(3.4 Å, 0°) = {carrier_ii:.4} ≥ {PLANT_II_CARRIER_RATIO}: {carrier_ii_ok} → {}",
        if plant_ii_fires { "FIRES" } else { "does not fire" }
    );

    // -------------------------- the contact term, with the wall HELD, on the nine exact
    let wall3e = |e: &ENode| -> f64 {
        a_oo * e.oo.iter().map(|&r| (-b_oo * r).exp()).sum::<f64>()
            + a_oh * e.ho.iter().map(|&r| (-b_oh * r).exp()).sum::<f64>()
            + a_hh * e.hh.iter().map(|&r| (-b_hh * r).exp()).sum::<f64>()
    };
    eprintln!("\nthe contact term with the wall HELD: remainder = ΔE_exact − [E_q(g) − E_q(40)]_engine − wall(g), on the nine exact geometries");
    let mut e_q: Vec<f64> = Vec::with_capacity(exact.len());
    let mut rem: Vec<f64> = Vec::with_capacity(exact.len());
    let mut wall_e: Vec<f64> = Vec::with_capacity(exact.len());
    for e in &exact {
        // the engine's OWN field difference, the same reference on both sides, seam OFF
        let (_, e_q_diff, _) = engine_interaction(&e.a, &e.b, None, SeamPlant::None);
        let wv = wall3e(e);
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
    let (mut c_best, mut p_best, mut c_res) = (f64::NAN, 0.0f64, f64::INFINITY);
    for ci in 0..NC {
        let c = cgrid(ci);
        // the contact term is `−P·Σ_HO e^{−c r}`, so the design column is `X = −Σ_HO e^{−c r}`
        let (mut num, mut den) = (0.0f64, 0.0f64);
        for (g, e) in exact.iter().enumerate() {
            let x = -e.ho.iter().map(|&r| (-c * r).exp()).sum::<f64>();
            num += we[g] * rem[g] * x;
            den += we[g] * x * x;
        }
        if !(den > 0.0) {
            continue;
        }
        let p = num / den;
        let mut r = 0.0f64;
        for (g, e) in exact.iter().enumerate() {
            let x = -e.ho.iter().map(|&r2| (-c * r2).exp()).sum::<f64>();
            let d = p * x - rem[g];
            r += we[g] * d * d;
        }
        if r < c_res {
            c_res = r;
            c_best = c;
            p_best = p;
        }
    }
    let contact = |e: &ENode| -> f64 { -p_best * e.ho.iter().map(|&r| (-c_best * r).exp()).sum::<f64>() };
    let mut c1_within = 0usize;
    let mut c1_misses: Vec<String> = Vec::new();
    let mut c1_lines: Vec<String> = Vec::new();
    eprintln!("\nthe contact fit: P = {p_best:.9e} Ha, c = {c_best:.2} /bohr, weighted residual {c_res:.9e}");
    for (g, e) in exact.iter().enumerate() {
        let f = contact(e);
        let miss = (rem[g] - f).abs();
        let t = (C1_FRAC * e.de_exact.abs()).max(C1_ABS);
        let ok = miss <= t;
        if ok {
            c1_within += 1;
        } else {
            c1_misses.push(e.name.clone());
        }
        eprintln!("  {}: remainder {:+.6e}, fit {f:+.6e}, miss {miss:.6e} ({:.4} of its tolerance {t:.3e}) → {}", e.name, rem[g], miss / t, if ok { "within" } else { "MISS" });
        c1_lines.push(format!(
            "{{\"node\": \"{}\", \"kind\": \"{}\", \"r_angstrom\": {:.1}, \"r_oo_bohr\": {:.6}, \"de_exact\": {:+.12e}, \"e_q_difference\": {:+.12e}, \"wall\": {:+.12e}, \"remainder\": {:+.12e}, \"contact_fit\": {f:+.12e}, \"miss\": {miss:.12e}, \"tolerance\": {t:.12e}, \"within\": {ok}}}",
            e.name,
            e.kind,
            e.r_ang,
            e.r_oo_bohr,
            e.de_exact,
            e_q[g],
            wall_e[g],
            rem[g]
        ));
    }
    let c1 = c1_within >= C1_MIN;
    eprintln!(
        "C1: {c1_within} of {} within max({C1_FRAC}·|ΔE_exact|, {C1_ABS:e}) (needs ≥ {C1_MIN}) → {}{}",
        exact.len(),
        if c1 { "PASS" } else { "FAIL — the contact term is not one exponential across bends; the arms still run, S3 read with the caveat" },
        if c1_misses.is_empty() { String::new() } else { format!("; misses: {}", c1_misses.join(", ")) }
    );

    // ------------------- dispersion: what is left on the FOUR OUTER LINEAR nodes only
    let outer: Vec<usize> = (0..exact.len()).filter(|&g| exact[g].outer_linear).collect();
    let rem2: Vec<f64> = outer.iter().map(|&g| rem[g] - contact(&exact[g])).collect();
    let (mut num, mut den) = (0.0f64, 0.0f64);
    for (oi, &g) in outer.iter().enumerate() {
        let x = -1.0 / exact[g].r_oo_bohr.powi(6);
        num += we[g] * rem2[oi] * x;
        den += we[g] * x * x;
    }
    let mut c6 = if den > 0.0 { num / den } else { 0.0 };
    let mut slopes: Vec<(f64, f64)> = Vec::new();
    eprintln!("\ndispersion — the remainder AFTER the contact term, on the four outer linear nodes:");
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

    // ------------------------------------------ G-C1 and plant (i), by the engine
    let model = SeamModel { a: a_oo, b: b_oo, p: p_best, c: c_best, c6, a_oh, b_oh, a_hh, b_hh, ..SeamModel::NO_WALL };
    eprintln!("\nG-C1 — the harvest is the engine's arithmetic, ONE reference (E_q(g) − E_q(40) from the engine itself), on the nine exact geometries");
    let mut g_c1_worst = 0.0f64;
    let mut g_c1_lines: Vec<String> = Vec::new();
    let mut plant_i = (f64::NAN, f64::NAN, f64::NAN, false);
    let mut units_ok = true;
    for e in &exact {
        let (e_int, e_field_diff, e_seam_diff) = engine_interaction(&e.a, &e.b, Some(model), SeamPlant::None);
        let (pen, w_oo, disp, w_oh, w_hh) = formula_terms(&e.a, &e.b, &model);
        let want = e_field_diff + pen + w_oo + disp + w_oh + w_hh;
        let miss = (e_int - want).abs();
        g_c1_worst = g_c1_worst.max(miss);
        let s = engine_dimer(&e.a, &e.b, Some(model), SeamPlant::None);
        let (units, oo_pairs, ho_pairs) = (s.seam_work.units, s.seam_work.oo_pairs, s.seam_work.ho_pairs);
        if units != 2 || oo_pairs != 1 || ho_pairs != 4 {
            units_ok = false;
        }
        eprintln!(
            "  {}: engine {e_int:+.12e} vs formula {want:+.12e} — miss {miss:.3e} (E_q diff {e_field_diff:+.6e}, seam {e_seam_diff:+.6e}; pen {pen:+.6e}, wall_OO {w_oo:+.6e}, wall_OH {w_oh:+.6e}, wall_HH {w_hh:+.6e}, disp {disp:+.6e}; units {units}, O–O {oo_pairs}, H–O {ho_pairs})",
            e.name
        );
        g_c1_lines.push(format!(
            "{{\"node\": \"{}\", \"r_angstrom\": {:.1}, \"engine_interaction\": {e_int:+.12e}, \"formula\": {want:+.12e}, \"miss\": {miss:.3e}, \"e_q_difference\": {e_field_diff:+.12e}, \"engine_seam\": {e_seam_diff:+.12e}, \"contact\": {pen:+.12e}, \"wall_oo\": {w_oo:+.12e}, \"wall_oh\": {w_oh:+.12e}, \"wall_hh\": {w_hh:+.12e}, \"disp\": {disp:+.12e}, \"units\": {units}, \"oo_pairs\": {oo_pairs}, \"ho_pairs\": {ho_pairs}}}",
            e.name, e.r_ang
        ));
        if e.kind == "linear" && (e.r_ang - REF_ANGSTROM).abs() < 1e-9 {
            let (e_pl, _, _) = engine_interaction(&e.a, &e.b, Some(model), SeamPlant::FlipPenetration);
            let observed = (e_pl - e_int).abs();
            let expected = 2.0 * pen.abs();
            let carrier = pen.abs();
            let fires = carrier >= PLANT_I_CARRIER && (observed - expected).abs() <= G_C1_TOL;
            plant_i = (observed, expected, carrier, fires);
            eprintln!(
                "plant (i) at the linear {REF_ANGSTROM:.1} Å node: miss {observed:.6e} vs 2·|p(2.9)| {expected:.6e} (difference {:.3e}); carrier |p(2.9)| {carrier:.3e} ≥ {PLANT_I_CARRIER:e}: {} → {}",
                (observed - expected).abs(),
                carrier >= PLANT_I_CARRIER,
                if fires { "FIRES" } else { "does not fire" }
            );
        }
    }
    let g_c1 = g_c1_worst <= G_C1_TOL;
    eprintln!("G-C1: worst |engine − formula| = {g_c1_worst:.3e} (stake {G_C1_TOL:e}) → {}", if g_c1 { "PASS" } else { "FAIL" });
    eprintln!("M-VACUOUS-SUCCESS: every G-C1 geometry served two units, one cross O–O pair and four cross H–O pairs: {units_ok}");

    // ------------------------------------------------------------------------ wall7.json
    let orient_lines: Vec<String> = nodes
        .iter()
        .enumerate()
        .map(|(g, n)| {
            let m = wall3(n);
            format!(
                "{{\"r_angstrom\": {:.1}, \"tilt_degrees\": {:.1}, \"r_oo_bohr\": {:.6}, \"e_exch\": {:+.12e}, \"model\": {:+.12e}, \"miss\": {:+.12e}, \"tolerance\": {:.12e}, \"within\": {}, \"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"n_det_a\": {}, \"n_det_b\": {}, \"product_dets\": {}, \"s_cross_max\": {:.6e}, \"sigma_seconds\": {:.3}, \"cpu_seconds\": {:.3}, \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}]}}",
                n.r_ang,
                n.tilt_deg,
                n.r_oo_bohr,
                n.hl.e_exch,
                m,
                m - n.hl.e_exch,
                tol[g],
                (m - n.hl.e_exch).abs() <= tol[g],
                n.hl.e_hl,
                n.hl.e_a0,
                n.hl.e_b0,
                n.hl.e_es,
                n.hl.norm,
                n.hl.nonzero_dets,
                n.hl.n_det,
                n.hl.n_det_a,
                n.hl.n_det_b,
                n.hl.n_det_a * n.hl.n_det_b,
                n.hl.s_cross_max,
                n.hl.sigma_seconds,
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
    fs::write(
        out.join("wall7.json"),
        format!(
            "{{\n  \"a\": {a_oo:.12e}, \"b\": {b_oo:.12e}, \"p\": {p_best:.12e}, \"c\": {c_best:.12e}, \"c6\": {c6:.12e}, \"a_oh\": {a_oh:.12e}, \"b_oh\": {b_oh:.12e}, \"a_hh\": {a_hh:.12e}, \"b_hh\": {b_hh:.12e},\n  \"s1_branch\": \"{s1_branch}\", \"within_count\": {within}, \"of\": {N_ORIENT}, \"misses\": [{}],\n  \"miss_details\": [{}],\n  \"plant_ii\": {{\"fires\": {plant_ii_fires}, \"model\": \"the wall on the OXYGENS ONLY, the same grid and rule\", \"b_oo\": {p2_b:.12e}, \"a_oo\": {p2_amp:.12e}, \"weighted_residual\": {p2_r:.12e}, \"failures\": {p2_fail}, \"failures_required\": {PLANT_II_MIN_FAIL}, \"carrier\": {carrier_ii:.6}, \"carrier_floor\": {PLANT_II_CARRIER_RATIO}, \"carrier_present\": {carrier_ii_ok}, \"carrier_definition\": \"E_exch(3.4 Å, tilt 180°) / E_exch(3.4 Å, tilt 0°), the undeformed referee\", \"failure_details\": [{}]}},\n  \"c1\": {{\"pass\": {c1}, \"within\": {c1_within}, \"of\": {}, \"required\": {C1_MIN}, \"tolerance_rule\": \"max({C1_FRAC}·|ΔE_exact|, {C1_ABS:e})\", \"misses\": [{}]}},\n  \"contact_fit\": {{\"p\": {p_best:.12e}, \"c\": {c_best:.12e}, \"weighted_residual\": {c_res:.12e}, \"grid\": \"0.50..=4.00 per bohr, step 0.01\", \"grid_points\": {NC}, \"placement\": \"cross-unit H–O\", \"weights\": \"1/ΔE_exact²\", \"points\": {}, \"refit_here\": true, \"remainder\": \"ΔE_exact − [E_q(g) − E_q(40)]_engine − wall(g), the wall HELD at the harvest\", \"nodes\": [\n    {}\n  ]}},\n  \"wall_fit\": {{\"grid\": \"0.50..=4.00 per bohr, step 0.05, per class\", \"triples\": {}, \"weights\": \"1/E_exch²\", \"nonnegativity\": \"a class whose fitted amplitude is negative is dropped (amplitude an exact 0.0) and the rest refit; the most negative first, one class per round\", \"weighted_residual\": {best_r:.12e}, \"classes_kept\": {{\"oo\": {}, \"oh\": {}, \"hh\": {}}}, \"tolerance_rule\": \"max({WALL_TOL_FRAC}·E_exch, {WALL_TOL_ABS:e})\", \"branch_b_minimum\": {S1_B_MIN}, \"fit_seconds\": {:.3}}},\n  \"w0\": {{\"pass\": {w0}, \"norm_ok\": {norm_ok}, \"norm_window\": \"({NORM_LO}, {NORM_HI}]\", \"norm_lowest\": {norm_lo_seen:.15e}, \"norm_highest\": {norm_hi_seen:.15e}, \"count_ok_as_staked\": {count_ok_literal}, \"count_ok_product_space\": {count_ok_product}, \"nonzero_dets_staked\": {NONZERO_DETS_STAKED}, \"nonzero_dets_measured\": {}, \"n_det\": {}, \"product_dets_measured\": {}, \"count_note\": \"the UNDEFORMED referee's product state is spread over the full dimer space by the Löwdin transform, so nonzero_dets is n_det; n_det_a·n_det_b is the count of monomer-product determinants it is built from, and FIELD-6's own records carry that as nonzero_dets_expected. Both legs measured, neither repaired.\", \"floor_ok\": {floor_ok}, \"floor\": {R_FLOOR:e}, \"monotone_ok\": {monotone_ok}, \"monotone_breaks\": [{}], \"pass_with_product_space_count\": {w0_product_reading}}},\n  \"price\": {{\"tenth_of_price_core_seconds\": {HL_PRICE_TENTH_CORE_S}, \"every_reading_at_or_above\": {price_ok}, \"cheapest_core_seconds\": {:.3}, \"dearest_core_seconds\": {:.3}}},\n  \"dispersion\": {{\"nodes\": \"the four outer linear nodes, after the contact term\", \"c6\": {c6:.12e}, \"transferred\": {c6_transferred}, \"slope_band\": [{SLOPE_LO}, {SLOPE_HI}], \"slopes\": [{}]}},\n  \"g_c1\": {{\"pass\": {g_c1}, \"worst_miss\": {g_c1_worst:.6e}, \"tolerance\": {G_C1_TOL:e}, \"reference\": \"E_q(g) − E_q(40 bohr), the engine's own field on both sides\", \"units_and_pair_counts_ok\": {units_ok}, \"nodes\": [\n    {}\n  ]}},\n  \"plant_i\": {{\"miss_observed\": {:.6e}, \"miss_expected\": {:.6e}, \"carrier_p_ho\": {:.6e}, \"carrier_floor\": {PLANT_I_CARRIER:e}, \"fires\": {}, \"node\": \"linear_R{REF_ANGSTROM:.1}\", \"plant\": \"FlipPenetration (P → −P)\"}},\n  \"referee\": \"heitler_london_undeformed — the antisymmetrised product of the monomers' own wavefunctions, the monomers NOT deformed\",\n  \"orientation_set\": {{\"r_angstrom\": [{}], \"tilt_degrees\": [{}], \"pivot\": \"the acceptor's own oxygen\", \"axis\": \"x\", \"donor\": \"untouched\"}},\n  \"orientations\": [\n    {}\n  ]\n}}\n",
            miss_names.join(", "),
            miss_lines.join(", "),
            p2_fail_lines.join(", "),
            exact.len(),
            c1_miss_names.join(", "),
            exact.len(),
            c1_lines.join(",\n    "),
            NB * NB * NB,
            act[0],
            act[1],
            act[2],
            t_fit.elapsed().as_secs_f64(),
            nodes[0].hl.nonzero_dets,
            nodes[0].hl.n_det,
            nodes[0].hl.n_det_a * nodes[0].hl.n_det_b,
            monotone_breaks.join(", "),
            nodes.iter().map(|n| n.cpu).fold(f64::INFINITY, f64::min),
            nodes.iter().map(|n| n.cpu).fold(0.0f64, f64::max),
            slope_lines.join(", "),
            g_c1_lines.join(",\n    "),
            plant_i.0,
            plant_i.1,
            plant_i.2,
            plant_i.3,
            ORIENT_R.iter().map(|r| format!("{r:.1}")).collect::<Vec<_>>().join(", "),
            ORIENT_TILT.iter().map(|t| format!("{t:.1}")).collect::<Vec<_>>().join(", "),
            orient_lines.join(",\n    "),
        ),
    )
    .unwrap();
    eprintln!("\nwall7.json written");

    // ------------------------------------ prediction.json, BEFORE the held-out solve
    let (a_t, b_t) = twisted(o, h, TWIST_ANGSTROM, TWIST_DEGREES, TWIST_TILT_DEGREES);
    let r_oo_t = cross_oo(&a_t, &b_t);
    assert!(
        (r_oo_t - TWIST_ANGSTROM * ANGSTROM_TO_BOHR).abs() < 1e-9,
        "both rotations fix the acceptor's own oxygen: R_OO must be unchanged ({r_oo_t:.9} vs {:.9})",
        TWIST_ANGSTROM * ANGSTROM_TO_BOHR
    );
    let (e_pred, e_q_t, e_seam_t) = engine_interaction(&a_t, &b_t, Some(model), SeamPlant::None);
    let (pen_t, w_oo_t, disp_t, w_oh_t, w_hh_t) = formula_terms(&a_t, &b_t, &model);
    let s_t = engine_dimer(&a_t, &b_t, Some(model), SeamPlant::None);
    let (oo_t, ho_t, hh_t) = cross_classes(&a_t, &b_t);
    fs::write(
        out.join("prediction.json"),
        format!(
            "{{\n  \"node\": \"twisted_R{TWIST_ANGSTROM:.1}\", \"r_oo_angstrom\": {TWIST_ANGSTROM:.3}, \"r_oo_bohr\": {r_oo_t:.6}, \"twist_degrees\": {TWIST_DEGREES:.1}, \"twist_axis\": \"z, the O···O axis, through the acceptor's own oxygen\", \"tilt_degrees\": {TWIST_TILT_DEGREES:.1}, \"tilt_axis\": \"x, the acceptor's own, AFTER the twist\", \"held_out\": true, \"kind\": \"a TWIST — neither of the two kinds the contact term is fit on (§4, M-UNTESTED-GAP)\",\n  \"e_pred\": {e_pred:+.12e},\n  \"parts\": {{\"e_q_difference\": {e_q_t:+.12e}, \"contact\": {pen_t:+.12e}, \"wall_oo\": {w_oo_t:+.12e}, \"wall_oh\": {w_oh_t:+.12e}, \"wall_hh\": {w_hh_t:+.12e}, \"wall_total\": {:+.12e}, \"disp\": {disp_t:+.12e}, \"engine_seam\": {e_seam_t:+.12e}}},\n  \"coefficients\": {{\"a\": {a_oo:.12e}, \"b\": {b_oo:.12e}, \"p\": {p_best:.12e}, \"c\": {c_best:.12e}, \"c6\": {c6:.12e}, \"a_oh\": {a_oh:.12e}, \"b_oh\": {b_oh:.12e}, \"a_hh\": {a_hh:.12e}, \"b_hh\": {b_hh:.12e}}},\n  \"s1_branch\": \"{s1_branch}\", \"c6_transferred\": {c6_transferred}, \"units\": {}, \"oo_pairs\": {}, \"ho_pairs\": {},\n  \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\", \"tolerance_frac\": {PRED_FRAC}, \"tolerance_abs\": {PRED_ABS:e},\n  \"exact_solve_stake\": {{\"n_det\": {N_DET_DIMER}, \"cpu_seconds_lo\": {S2_CPU_LO}, \"cpu_seconds_hi\": {S2_CPU_HI}, \"residual_bar\": {RESIDUAL_BAR:e}, \"exit\": \"Converged\"}},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}],\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
            w_oo_t + w_oh_t + w_hh_t,
            s_t.seam_work.units,
            s_t.seam_work.oo_pairs,
            s_t.seam_work.ho_pairs,
            list_json(&oo_t),
            list_json(&ho_t),
            list_json(&hh_t),
            centers_json(&a_t),
            centers_json(&b_t),
        ),
    )
    .unwrap();
    eprintln!(
        "prediction.json filed BEFORE the held-out solve: E_pred {e_pred:+.6e} Ha — E_q(g) − E_q(40) {e_q_t:+.6e}, contact {pen_t:+.6e}, wall_OO {w_oo_t:+.6e}, wall_OH {w_oh_t:+.6e}, wall_HH {w_hh_t:+.6e}, disp {disp_t:+.6e}; units {}",
        s_t.seam_work.units
    );
    fs::write(out.join("orient.done"), "done\n").unwrap();
}

// --------------------------------------------------------------------------- predict (S2)

fn run_predict(out: &Path) {
    let pred_path = out.join("prediction.json");
    let Ok(pred) = fs::read_to_string(&pred_path) else {
        eprintln!("{} missing: the prediction is filed BEFORE the solve (run `orient` first). Nothing written.", pred_path.display());
        std::process::exit(2);
    };
    let e_pred = json_num(&pred, "e_pred");
    let wall7 = fs::read_to_string(out.join("wall7.json")).expect("wall7.json: run `orient` first");
    let (a_oo, b_oo) = (json_num(&wall7, "a"), json_num(&wall7, "b"));
    let (a_oh, b_oh) = (json_num(&wall7, "a_oh"), json_num(&wall7, "b_oh"));
    let (a_hh, b_hh) = (json_num(&wall7, "a_hh"), json_num(&wall7, "b_hh"));
    assert!(
        a_oo.is_finite() && b_oo.is_finite() && a_oh.is_finite() && b_oh.is_finite() && a_hh.is_finite() && b_hh.is_finite(),
        "wall7.json carries no three-class wall"
    );
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let (a, b) = twisted(o, h, TWIST_ANGSTROM, TWIST_DEGREES, TWIST_TILT_DEGREES);
    let name = format!("twisted_R{TWIST_ANGSTROM:.1}");
    eprintln!(
        "FIELD-7 predict — the HELD-OUT TWISTED node ({TWIST_DEGREES:.0}° about the O···O axis then {TWIST_TILT_DEGREES:.0}° about the acceptor's own x, R_OO {TWIST_ANGSTROM:.1} Å) on {} threads; E_pred {e_pred:+.12e} Ha",
        threads()
    );

    // the exact solve first: the prediction is already on disk
    let ok = solve_node(out, &name, TWIST_ANGSTROM, &a, &b, true);
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
    let r_oo_t = cross_oo(&a, &b);
    let t0 = Instant::now();
    let c0 = cpu_seconds();
    let hl = heitler_london_undeformed(&a, &b);
    let hl_wall = t0.elapsed().as_secs_f64();
    let hl_cpu = cpu_seconds() - c0;
    let w_oo: f64 = a_oo * oo.iter().map(|&r| (-b_oo * r).exp()).sum::<f64>();
    let w_oh: f64 = a_oh * ho.iter().map(|&r| (-b_oh * r).exp()).sum::<f64>();
    let w_hh: f64 = a_hh * hh.iter().map(|&r| (-b_hh * r).exp()).sum::<f64>();
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
        "S2: ΔE_exact {de:+.6e} Ha, E_pred {e_pred:+.6e} — miss {miss:.3e} ({:.1} % of |ΔE_exact|) against {tol:.3e}; E_exch(undeformed, twisted) {:+.6e} vs the three-class wall {wall_held:+.6e} (O–O {w_oo:+.6e}, H–O {w_oh:+.6e}, H–H {w_hh:+.6e}), difference {wall_gap:.3e} → branch ({s2})",
        100.0 * miss / de.abs(),
        hl.e_exch
    );
    fs::write(
        out.join("prediction_check.json"),
        format!(
            "{{\n  \"node\": \"{name}\", \"e_pred\": {e_pred:+.12e}, \"de_exact\": {de:+.12e},\n  \"miss\": {miss:.6e}, \"miss_fraction\": {:.6}, \"tolerance\": {tol:.6e}, \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\",\n  \"s2_branch\": \"{s2}\",\n  \"exact\": {{\"converged\": {ok}, \"exit\": \"{}\", \"davidson_iters\": {}, \"residual\": {:.3e}, \"residual_bar\": {RESIDUAL_BAR:e}, \"n_det\": {n_det}, \"n_det_expected\": {N_DET_DIMER}, \"n_det_ok\": {n_det_ok}, \"cpu_seconds\": {cpu:.1}, \"cpu_seconds_lo\": {S2_CPU_LO}, \"cpu_seconds_hi\": {S2_CPU_HI}, \"price_in_band\": {price_ok}, \"wall_seconds\": {:.1}}},\n  \"exchange_on_the_held_out_node\": {{\"r_oo_bohr\": {r_oo_t:.6}, \"e_exch\": {:+.12e}, \"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det_a\": {}, \"n_det_b\": {}, \"product_dets\": {}, \"s_cross_max\": {:.6e}, \"sigma_seconds\": {:.3}, \"cpu_seconds\": {hl_cpu:.3}, \"wall_seconds\": {hl_wall:.3}}},\n  \"wall\": {{\"a\": {a_oo:.12e}, \"b\": {b_oo:.12e}, \"a_oh\": {a_oh:.12e}, \"b_oh\": {b_oh:.12e}, \"a_hh\": {a_hh:.12e}, \"b_hh\": {b_hh:.12e}, \"value\": {wall_held:+.12e}, \"oo\": {w_oo:+.12e}, \"oh\": {w_oh:+.12e}, \"hh\": {w_hh:+.12e}, \"minus_e_exch\": {:+.12e}, \"abs_difference\": {wall_gap:.6e}, \"within_tolerance\": {}}},\n  \"cross_oo_bohr\": [{}], \"cross_ho_bohr\": [{}], \"cross_hh_bohr\": [{}]\n}}\n",
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
    let what = args.get(1).map(String::as_str).unwrap_or("orient");
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/field7".to_string()));
    fs::create_dir_all(&out).expect("out");
    match what {
        "orient" => run_orient(&out),
        "predict" => run_predict(&out),
        other => eprintln!("unknown phase {other} (orient | predict)"),
    }
}
