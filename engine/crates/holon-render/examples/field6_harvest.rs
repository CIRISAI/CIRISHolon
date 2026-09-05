//! FIELD-6's harvest (`conformance/water_observatory/FIELD6_PREREG.md` §0, §2, §5, §6):
//! EXCHANGE with the monomers LEFT ALONE. FIELD-5's runner with four changes and nothing
//! else: the readings come from `heitler_london_undeformed` (the orthogonalised reading of
//! FIELD-5 is taken beside it at every node — it orders G-U0 and it IS plant (ii)); the wall
//! fit's tolerance is DERIVED per node, `max(0.10·|ΔE_exact|, 0.05·E_exch)`; the penetration
//! term is NOT re-fit but read from `field5/wall5.json` as a frozen record; and the held-out
//! geometry is NEW — the 3.1 Å linear dimer with the acceptor rotated 45° about x.
//!
//! ```text
//! cargo run --release -p holon-render --example field6_harvest -- exchange [OUT_DIR]
//! cargo run --release -p holon-render --example field6_harvest -- predict  [OUT_DIR]
//! ```
//!
//! `exchange`: the six undeformed-and-orthogonalised readings and the 40-bohr limit (G-U0),
//! H1, the derived-tolerance wall fit, the frozen penetration term, dispersion, plant (ii),
//! `wall6.json`, G-C1 and plant (i) by the engine, and `prediction.json` for the tilted-45°
//! node written BEFORE that node is solved. `predict`: refuses without `prediction.json`,
//! solves the held-out node exactly, reads both referees on it against the wall, and records
//! the two FREE readings (FIELD-5's tilted-30° node, FIELD-4's flipped node).
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::{solve_embedded, supermolecule, water_dimer_linear, Fragment, ANGSTROM_TO_BOHR};
use holon_chem::fci::SolveExit;
use holon_chem::heitler_london::{heitler_london, heitler_london_undeformed, HlPlant, HlReading};
use holon_render::seam::{SeamModel, SeamPlant};
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/quartet.rs"]
#[allow(dead_code)]
mod quartet;

/// EMBED-1's water pins — the same numbers FIELD-3's, FIELD-4's and FIELD-5's runners carry.
const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;
/// The six linear nodes (Å), SHORTEST FIRST (the harvest reads prefixes).
const NODES_ANGSTROM: [f64; 6] = [2.5, 2.7, 2.9, 3.1, 3.4, 3.7];
/// The OUTER nodes begin here: 2.9, 3.1, 3.4, 3.7 Å — where FIELD-4 measured that the
/// density field is a field. FIELD-5's penetration fit was made on these four; the
/// dispersion fit here uses the same four.
const OUTER_FROM: usize = 2;
/// The node H1's ratio leg and plant (i) are read at.
const REF_ANGSTROM: f64 = 2.9;
/// S2's held-out geometry (NEW in this freeze): the 3.1 Å linear dimer with the acceptor
/// rotated 45° about the x-axis through its OWN oxygen.
const HELD_ANGSTROM: f64 = 3.1;
const HELD_DEGREES: f64 = 45.0;
/// FIELD-5's tilted node — a FREE reading here (§4), never a stake.
const TILT5_ANGSTROM: f64 = 2.9;
const TILT5_DEGREES: f64 = 30.0;
/// FIELD-4's flipped node — a FREE reading here (§4), never a stake.
const FLIPPED_ANGSTROM: f64 = 3.4;
/// The separation at which the acceptor is "away" (bohr): G-U0's limit and the engine's
/// reference on both sides of G-C1.
const FAR_BOHR: f64 = 40.0;

/// The residual bar every exact solve must meet (EMBED-3's).
const RESIDUAL_BAR: f64 = 1e-9;
/// The reading floor on every harvested residual (M-FLOOR-UNSTAKED).
const R_FLOOR: f64 = 1e-6;
/// S1's DERIVED tolerance, per node: `max(FIT_TOL_DE·|ΔE_exact|, FIT_TOL_EXCH·E_exch)`. The
/// second arm is half of FIELD-5's measured exponent drift (9 % over the range), NOT typed.
const FIT_TOL_DE: f64 = 0.10;
const FIT_TOL_EXCH: f64 = 0.05;
/// S2's tolerance: `max(0.25·|ΔE_exact|, 5e-4)`.
const PRED_FRAC: f64 = 0.25;
const PRED_ABS: f64 = 5e-4;
/// The band the remainder's log-log slope must lie in for `C₆` to transfer.
const SLOPE_LO: f64 = -8.0;
const SLOPE_HI: f64 = -4.0;
/// G-U0: the undeformed product's norm window, its ordering slack (the record's printed
/// precision), and the 40-bohr limit.
const NORM_LO: f64 = 0.8;
const NORM_HI: f64 = 1.0;
const ORDER_SLACK: f64 = 1e-10;
const LIMIT_TOL: f64 = 1e-8;
/// H1's ratio leg: the undeformed exchange at 2.9 Å must be under this share of FIELD-5's.
const H1_RATIO: f64 = 0.5;
/// Plant (ii)'s carrier: the deformation penalty at 2.9 Å.
const PLANT_II_CARRIER: f64 = 1e-3;
/// G-C1's tolerance, and plant (i)'s carrier.
const G_C1_TOL: f64 = 1e-10;
const PLANT_I_CARRIER: f64 = 1e-4;
/// The determinant count FIELD-3's supermolecule carries (EXACT).
const N_DET_DIMER: usize = 1_002_001;
/// M-CHEAPER-THAN-ITS-PRICE: FIELD-5 measured 55–59 core-seconds per orthogonalised reading,
/// and the undeformed reading adds the contraction. A reading under a TENTH of 55 is refused.
const HL_PRICE_TENTH_CORE_S: f64 = 5.5;

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

/// FIELD-3's `linear` verbatim.
fn linear(o: Species, h: Species, r_oo_angstrom: f64) -> (Fragment, Fragment) {
    water_dimer_linear(o, h, H2O_R, H2O_THETA, r_oo_angstrom * ANGSTROM_TO_BOHR)
}

/// FIELD-3's FLIPPED dimer verbatim: the linear donor, the acceptor rotated by π about the
/// x-axis through its oxygen. A free reading here (§4), never a stake.
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

/// The linear dimer with the ACCEPTOR rotated by `theta_degrees` about the x-axis through its
/// OWN oxygen. FIELD-5's `tilted` verbatim: the donor and its O–H are untouched, and the
/// O···O separation is unchanged (the rotation fixes the acceptor's oxygen).
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
    let tilt = Fragment::new(acc.species.clone(), centers, acc.weights.clone());
    (donor, tilt)
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

/// Every CROSS-UNIT hydrogen–oxygen distance (bohr): the engine's placement for the
/// penetration term — where the two densities overlap.
fn cross_ho(a: &Fragment, b: &Fragment) -> Vec<f64> {
    let mut v = Vec::new();
    for (sa, ca) in a.species.iter().zip(a.centers.iter()) {
        for (sb, cb) in b.species.iter().zip(b.centers.iter()) {
            if (sa.z == 1 && sb.z == 8) || (sa.z == 8 && sb.z == 1) {
                v.push(dist(ca, cb));
            }
        }
    }
    v
}

/// The cross-unit O–O distance (bohr).
fn cross_oo(a: &Fragment, b: &Fragment) -> f64 {
    let ca = a.centers[a.species.iter().position(|s| s.z == 8).expect("an oxygen")];
    let cb = b.centers[b.species.iter().position(|s| s.z == 8).expect("an oxygen")];
    dist(&ca, &cb)
}

// ------------------------------------------------------------------------------ the engine

/// FIELD-4's `field4_check::engine_dimer` verbatim: an open box, the field on with the pin
/// charge, the seam model and its plant installed, forces computed once so the closure
/// assignment and the rows are read.
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
/// `E_q(R) − E_q(40)` — the SAME reference the formula side of G-C1 uses.
fn engine_interaction(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, plant: SeamPlant) -> (f64, f64, f64) {
    let s = engine_dimer(a, b, seam, plant);
    let near = (s.e_pair + s.e_three) + s.e_field + s.e_seam;
    let far_b = b.translated([FAR_BOHR, 0.0, 0.0]);
    let f = engine_dimer(a, &far_b, seam, plant);
    let far = (f.e_pair + f.e_three) + f.e_field + f.e_seam;
    (near - far, s.e_field - f.e_field, s.e_seam - f.e_seam)
}

/// FIELD-4's `field4_check::formula_terms` verbatim: the penetration term over the four
/// cross-unit H–O pairs, the wall and the dispersion on the O–O pair.
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

/// A per-node field of a `{"r_angstrom": …, …}` list, matched by `r_angstrom`.
fn node_field(text: &str, r: f64, key: &str) -> f64 {
    for chunk in text.split("{\"r_angstrom\": ").skip(1) {
        let rr: f64 = chunk.split(',').next().and_then(|x| x.trim().parse().ok()).unwrap_or(f64::NAN);
        if (rr - r).abs() < 1e-9 {
            return json_num(chunk, key);
        }
    }
    f64::NAN
}

// ----------------------------------------------------------------------- the exchange phase

struct XNode {
    r_ang: f64,
    r_bohr: f64,
    de_exact: f64,
    /// FIELD-3's record of the supermolecule's total energy — G-U0's lower end.
    e_super: f64,
    /// FIELD-3's RAW near-box engine field of record (`wall.json`'s `e_field`).
    e_q_raw: f64,
    ho: Vec<f64>,
    /// The UNDEFORMED referee — this freeze's instrument.
    hlu: HlReading,
    /// FIELD-5's ORTHOGONALISED referee — kept as the plant, and G-U0's upper end.
    hlo: HlReading,
    cpu_u: f64,
    cpu_o: f64,
}

/// One wall-fit sweep: the largest contiguous set of the SHORTEST nodes, at least three,
/// whose weighted log-linear fit lies within the per-node tolerance handed in. Returns the
/// winning `(k, ln A, b)` if any, every attempt as a JSON object, and the positive prefix.
fn fit_wall(r_ang: &[f64], r_bohr: &[f64], de: &[f64], val: &[f64], tol: &[f64], label: &str) -> (Option<(usize, f64, f64)>, Vec<String>, usize) {
    let n_pos_prefix = val.iter().take_while(|&&v| v > R_FLOOR).count();
    let mut best: Option<(usize, f64, f64)> = None;
    let mut attempts: Vec<String> = Vec::new();
    for k in (3..=n_pos_prefix).rev() {
        let w: Vec<f64> = (0..k).map(|i| 1.0 / (de[i] * de[i])).collect();
        let sw: f64 = w.iter().sum();
        let mx = (0..k).map(|i| w[i] * r_bohr[i]).sum::<f64>() / sw;
        let my = (0..k).map(|i| w[i] * val[i].ln()).sum::<f64>() / sw;
        let sxx: f64 = (0..k).map(|i| w[i] * (r_bohr[i] - mx) * (r_bohr[i] - mx)).sum();
        let sxy: f64 = (0..k).map(|i| w[i] * (r_bohr[i] - mx) * (val[i].ln() - my)).sum();
        let slope = sxy / sxx; // ln E_exch = ln A − b R
        let b = -slope;
        let ln_a = my - slope * mx;
        let miss = |i: usize| ((ln_a - b * r_bohr[i]).exp() - val[i]).abs();
        let within = (0..k).all(|i| miss(i) <= tol[i]);
        let worst = (0..k).map(|i| miss(i) / tol[i]).fold(0.0, f64::max);
        attempts.push(format!(
            "{{\"k\": {k}, \"r_x_angstrom\": {:.1}, \"a\": {:.12e}, \"b\": {b:.12e}, \"worst_miss_over_tolerance\": {worst:.6}, \"qualifies\": {within}}}",
            r_ang[k - 1],
            ln_a.exp()
        ));
        eprintln!(
            "  [{label}] attempt k = {k} (to {:.1} Å): A = {:.6e} Ha, b = {b:.6} /bohr, worst miss {worst:.4} of its DERIVED tolerance → {}",
            r_ang[k - 1],
            ln_a.exp(),
            if within { "QUALIFIES" } else { "does not qualify" }
        );
        if within {
            best = Some((k, ln_a, b));
            break;
        }
    }
    (best, attempts, n_pos_prefix)
}

fn run_exchange(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let f3 = sibling(out, "field3");
    let f5 = sibling(out, "field5");
    let wall3 = fs::read_to_string(f3.join("wall.json")).unwrap_or_else(|_| panic!("{}/wall.json missing: FIELD-3's engine field is the E_q of record", f3.display()));
    let wall5 = fs::read_to_string(f5.join("wall5.json")).unwrap_or_else(|_| panic!("{}/wall5.json missing: FIELD-5's penetration fit is the frozen record this freeze reuses", f5.display()));
    // FIELD-5's penetration fit, REUSED AS FROZEN — not re-fit here.
    let p_coef = json_num(&wall5, "p");
    let c_coef = json_num(&wall5, "c");
    assert!(p_coef.is_finite() && c_coef.is_finite(), "field5/wall5.json carries no (p, c)");
    const PEN_NOTE: &str = "FIELD-5's fit on the outer four nodes, reused as frozen";
    eprintln!("FIELD-6 exchange — the UNDEFORMED Heitler–London referee on FIELD-3's six linear nodes, {} threads", threads());
    eprintln!("penetration: P = {p_coef:.9e} Ha, c = {c_coef:.2} /bohr — {PEN_NOTE}");

    // ---------------------------------------------------------------- the six readings
    let mut nodes: Vec<XNode> = Vec::new();
    for &r in NODES_ANGSTROM.iter() {
        let node_path = f3.join(format!("linear_R{r:.1}.json"));
        let t3 = fs::read_to_string(&node_path).unwrap_or_else(|_| panic!("{} missing", node_path.display()));
        let de_exact = json_num(&t3, "de_exact");
        let e_super = json_num(&t3, "e_super");
        let e_q_raw = node_field(&wall3, r, "e_field");
        assert!(e_super.is_finite(), "FIELD-3's {} carries no e_super", node_path.display());
        assert!(e_q_raw.is_finite(), "FIELD-3's wall.json carries no e_field at R = {r} Å");

        let (a, b) = linear(o, h, r);
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let hlu = heitler_london_undeformed(&a, &b);
        let wall_u = t0.elapsed().as_secs_f64();
        let cpu_u = cpu_seconds() - c0;
        let t1 = Instant::now();
        let c1 = cpu_seconds();
        let hlo = heitler_london(&a, &b, HlPlant::None);
        let wall_o = t1.elapsed().as_secs_f64();
        let cpu_o = cpu_seconds() - c1;
        let ho = cross_ho(&a, &b);
        let r_bohr = cross_oo(&a, &b);
        let priced = cpu_u >= HL_PRICE_TENTH_CORE_S;
        let dets_expected = hlu.n_det_a * hlu.n_det_b;

        fs::write(
            out.join(format!("exchange_R{r:.1}.json")),
            format!(
                "{{\n  \"node\": \"exchange_R{r:.1}\", \"r_oo_angstrom\": {r:.3}, \"r_oo_bohr\": {r_bohr:.6},\n  \"undeformed\": {{\"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"e_exch\": {:+.12e}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"n_det_a\": {}, \"n_det_b\": {}, \"nonzero_dets_expected\": {dets_expected}, \"s_cross_max\": {:.12e}, \"sigma_seconds\": {:.3}, \"wall_seconds\": {wall_u:.3}, \"cpu_seconds\": {cpu_u:.3}}},\n  \"orthogonalised\": {{\"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"e_exch\": {:+.12e}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"s_cross_max\": {:.12e}, \"sigma_seconds\": {:.3}, \"wall_seconds\": {wall_o:.3}, \"cpu_seconds\": {cpu_o:.3}}},\n  \"deformation_penalty\": {:+.12e},\n  \"e_super_field3\": {e_super:.12e}, \"de_exact\": {de_exact:+.12e}, \"e_q_raw_field3\": {e_q_raw:+.12e},\n  \"cross_ho_bohr\": [{}],\n  \"price_tenth_core_seconds\": {HL_PRICE_TENTH_CORE_S}, \"price_floor_met\": {priced}, \"threads\": {}\n}}\n",
                hlu.e_hl,
                hlu.e_a0,
                hlu.e_b0,
                hlu.e_es,
                hlu.e_exch,
                hlu.norm,
                hlu.nonzero_dets,
                hlu.n_det,
                hlu.n_det_a,
                hlu.n_det_b,
                hlu.s_cross_max,
                hlu.sigma_seconds,
                hlo.e_hl,
                hlo.e_a0,
                hlo.e_b0,
                hlo.e_es,
                hlo.e_exch,
                hlo.norm,
                hlo.nonzero_dets,
                hlo.n_det,
                hlo.s_cross_max,
                hlo.sigma_seconds,
                hlo.e_exch - hlu.e_exch,
                ho.iter().map(|d| format!("{d:.6}")).collect::<Vec<_>>().join(", "),
                threads(),
            ),
        )
        .unwrap();
        eprintln!(
            "  R_OO {r:.1} Å: undeformed E_HL {:+.9e} E_exch {:+.6e} norm {:.12} ({cpu_u:.1} core-s) | orthogonalised E_HL {:+.9e} E_exch {:+.6e} ({cpu_o:.1} core-s) | penalty {:+.6e}",
            hlu.e_hl, hlu.e_exch, hlu.norm, hlo.e_hl, hlo.e_exch, hlo.e_exch - hlu.e_exch
        );
        nodes.push(XNode { r_ang: r, r_bohr, de_exact, e_super, e_q_raw, ho, hlu, hlo, cpu_u, cpu_o });
    }

    // ------------------------------------------- G-U0: the undeformed state is what it says
    // the 40-bohr limit, on the 2.9 Å node's geometry with the acceptor translated along x
    let (a29, b29) = linear(o, h, REF_ANGSTROM);
    let b_far = b29.translated([FAR_BOHR, 0.0, 0.0]);
    let t0 = Instant::now();
    let c0 = cpu_seconds();
    let far_u = heitler_london_undeformed(&a29, &b_far);
    let far_u_wall = t0.elapsed().as_secs_f64();
    let far_u_cpu = cpu_seconds() - c0;
    let t1 = Instant::now();
    let c1 = cpu_seconds();
    let far_o = heitler_london(&a29, &b_far, HlPlant::None);
    let far_o_wall = t1.elapsed().as_secs_f64();
    let far_o_cpu = cpu_seconds() - c1;
    let far_exch = far_u.e_exch.abs();
    let far_resid = (far_u.e_hl - far_u.e_a0 - far_u.e_b0 - far_u.e_es).abs();
    fs::write(
        out.join("exchange_far.json"),
        format!(
            "{{\n  \"node\": \"exchange_far_{FAR_BOHR:.0}bohr\", \"base_r_oo_angstrom\": {REF_ANGSTROM:.3}, \"acceptor_translated_bohr\": [{FAR_BOHR:.1}, 0.0, 0.0], \"r_oo_bohr\": {:.6},\n  \"undeformed\": {{\"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"e_exch\": {:+.12e}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"s_cross_max\": {:.12e}, \"sigma_seconds\": {:.3}, \"wall_seconds\": {far_u_wall:.3}, \"cpu_seconds\": {far_u_cpu:.3}}},\n  \"orthogonalised\": {{\"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"e_exch\": {:+.12e}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"sigma_seconds\": {:.3}, \"wall_seconds\": {far_o_wall:.3}, \"cpu_seconds\": {far_o_cpu:.3}}},\n  \"limit_abs_e_exch\": {far_exch:.6e}, \"limit_abs_residual\": {far_resid:.6e}, \"limit_tolerance\": {LIMIT_TOL:e}\n}}\n",
            cross_oo(&a29, &b_far),
            far_u.e_hl,
            far_u.e_a0,
            far_u.e_b0,
            far_u.e_es,
            far_u.e_exch,
            far_u.norm,
            far_u.nonzero_dets,
            far_u.n_det,
            far_u.s_cross_max,
            far_u.sigma_seconds,
            far_o.e_hl,
            far_o.e_a0,
            far_o.e_b0,
            far_o.e_es,
            far_o.e_exch,
            far_o.norm,
            far_o.nonzero_dets,
            far_o.n_det,
            far_o.sigma_seconds,
        ),
    )
    .unwrap();

    let norm_ok = nodes.iter().all(|n| n.hlu.norm > NORM_LO && n.hlu.norm <= NORM_HI);
    let norm_lowest = nodes.iter().map(|n| n.hlu.norm).fold(f64::INFINITY, f64::min);
    let norm_highest = nodes.iter().map(|n| n.hlu.norm).fold(f64::NEG_INFINITY, f64::max);
    let order_ok = nodes.iter().all(|n| n.e_super <= n.hlu.e_hl + ORDER_SLACK && n.hlu.e_hl <= n.hlo.e_hl + ORDER_SLACK);
    let limit_ok = far_exch <= LIMIT_TOL && far_resid <= LIMIT_TOL;
    let g_u0 = norm_ok && order_ok && limit_ok;
    eprintln!("\nG-U0 — the undeformed state is what it says:");
    eprintln!(
        "  norm: every undeformed ⟨v|v⟩ in ({NORM_LO}, {NORM_HI}] — lowest {norm_lowest:.12}, highest {norm_highest:.12} → {} (the far reading's norm is {:.15}, {:+.3e} from 1; the orthogonalised norm is 1 by construction, worst {:.3e})",
        if norm_ok { "PASS" } else { "FAIL" },
        far_u.norm,
        far_u.norm - 1.0,
        nodes.iter().map(|n| (n.hlo.norm - 1.0).abs()).fold(0.0f64, f64::max)
    );
    eprintln!("  order: E_super(FIELD-3) ≤ E_HL(undeformed) ≤ E_HL(orthogonalised) at every node (slack {ORDER_SLACK:e}) → {}", if order_ok { "PASS" } else { "FAIL" });
    for n in &nodes {
        eprintln!(
            "    {:.1} Å: {:.12e} ≤ {:.12e} ≤ {:.12e}  (gaps {:+.6e}, {:+.6e})",
            n.r_ang,
            n.e_super,
            n.hlu.e_hl,
            n.hlo.e_hl,
            n.hlu.e_hl - n.e_super,
            n.hlo.e_hl - n.hlu.e_hl
        );
    }
    eprintln!(
        "  limit: at {FAR_BOHR:.0} bohr |E_exch| = {far_exch:.3e} and |E_HL − E_A0 − E_B0 − E_es| = {far_resid:.3e} (tol {LIMIT_TOL:e}) → {}",
        if limit_ok { "PASS" } else { "FAIL" }
    );
    eprintln!("G-U0 → {}", if g_u0 { "PASS" } else { "FAIL" });

    // ---------------------------------------------------- H1: exchange is a wall
    eprintln!("\n| R (Å) | R_OO (bohr) | ΔE_exact (Ha) | E_exch undeformed (Ha) | E_exch orthogonalised (Ha) | norm undeformed | σ (s) | core-s |");
    for n in &nodes {
        eprintln!(
            "| {:.1} | {:.4} | {:+.6e} | {:+.6e} | {:+.6e} | {:.12} | {:.1} | {:.1} |",
            n.r_ang,
            n.r_bohr,
            n.de_exact,
            n.hlu.e_exch,
            n.hlo.e_exch,
            n.hlu.norm,
            n.hlu.sigma_seconds,
            n.cpu_u
        );
    }
    let iref = nodes.iter().position(|n| (n.r_ang - REF_ANGSTROM).abs() < 1e-9).expect("the 2.9 Å node");
    let h1_positive = nodes.iter().all(|n| n.hlu.e_exch > R_FLOOR);
    let h1_monotone = (1..nodes.len()).all(|i| nodes[i].hlu.e_exch <= nodes[i - 1].hlu.e_exch);
    let h1_ratio_value = nodes[iref].hlu.e_exch / nodes[iref].hlo.e_exch;
    let h1_ratio = nodes[iref].hlu.e_exch < H1_RATIO * nodes[iref].hlo.e_exch;
    let h1 = h1_positive && h1_monotone && h1_ratio;
    eprintln!(
        "H1: E_exch > {R_FLOOR:e} at all six ({h1_positive}); non-increasing outward ({h1_monotone}); E_exch({REF_ANGSTROM:.1} Å) = {:.6e} < {H1_RATIO}·E_exch^orth = {:.6e} ({h1_ratio}, ratio {h1_ratio_value:.4}) → {}",
        nodes[iref].hlu.e_exch,
        H1_RATIO * nodes[iref].hlo.e_exch,
        if h1 { "PASS" } else { "FAIL" }
    );
    let price_ok = nodes.iter().all(|n| n.cpu_u >= HL_PRICE_TENTH_CORE_S);
    eprintln!(
        "M-CHEAPER-THAN-ITS-PRICE (recorded): every undeformed reading at or above a tenth of its {HL_PRICE_TENTH_CORE_S:.1} core-second price: {price_ok} (cheapest {:.1} core-s)",
        nodes.iter().map(|n| n.cpu_u).fold(f64::INFINITY, f64::min)
    );

    // ------------------------------ the wall on the UNDEFORMED E_exch, DERIVED tolerance
    let r_ang: Vec<f64> = nodes.iter().map(|n| n.r_ang).collect();
    let r_bohr: Vec<f64> = nodes.iter().map(|n| n.r_bohr).collect();
    let de: Vec<f64> = nodes.iter().map(|n| n.de_exact).collect();
    let val_u: Vec<f64> = nodes.iter().map(|n| n.hlu.e_exch).collect();
    let val_o: Vec<f64> = nodes.iter().map(|n| n.hlo.e_exch).collect();
    let tol_u: Vec<f64> = nodes.iter().map(|n| (FIT_TOL_DE * n.de_exact.abs()).max(FIT_TOL_EXCH * n.hlu.e_exch)).collect();
    let tol_o: Vec<f64> = nodes.iter().map(|n| (FIT_TOL_DE * n.de_exact.abs()).max(FIT_TOL_EXCH * n.hlo.e_exch)).collect();
    eprintln!("\nthe wall over the UNDEFORMED E_exch — FIELD-3's rule, the largest contiguous set of the SHORTEST nodes (≥ 3), within the DERIVED tolerance max({FIT_TOL_DE}·|ΔE_exact|, {FIT_TOL_EXCH}·E_exch) at each:");
    for (i, n) in nodes.iter().enumerate() {
        eprintln!(
            "  {:.1} Å: tolerance {:.6e} Ha = max({:.6e}, {:.6e}) — the {} arm",
            n.r_ang,
            tol_u[i],
            FIT_TOL_DE * n.de_exact.abs(),
            FIT_TOL_EXCH * n.hlu.e_exch,
            if FIT_TOL_DE * n.de_exact.abs() >= FIT_TOL_EXCH * n.hlu.e_exch { "ΔE_exact" } else { "E_exch" }
        );
    }
    let (best, attempts, n_pos_prefix) = fit_wall(&r_ang, &r_bohr, &de, &val_u, &tol_u, "undeformed");
    let (k, a_coef, b_coef) = match best {
        Some((k, ln_a, b)) => (k, ln_a.exp(), b),
        None => (0, 0.0, 0.0),
    };
    let r_x = if k > 0 { nodes[k - 1].r_ang } else { f64::NAN };
    let wall_at = |rb: f64| if k > 0 { a_coef * (-b_coef * rb).exp() } else { 0.0 };
    eprintln!("the wall: positive prefix {n_pos_prefix} of 6; A = {a_coef:.9e} Ha, b = {b_coef:.6} /bohr over the shortest {k} nodes (R_x = {r_x} Å)");
    if k > 0 {
        for (i, n) in nodes.iter().enumerate() {
            let f = wall_at(n.r_bohr);
            eprintln!(
                "  {:.1} Å: E_exch {:+.6e}  wall {:+.6e}  miss {:+.6e} ({:.4} of its tolerance {:.3e}){}",
                n.r_ang,
                n.hlu.e_exch,
                f,
                f - n.hlu.e_exch,
                (f - n.hlu.e_exch).abs() / tol_u[i],
                tol_u[i],
                if i < k { "  [in the fit]" } else { "" }
            );
        }
    }
    let branch = if k == nodes.len() {
        "a"
    } else if k >= 3 {
        "b"
    } else {
        "c"
    };
    let branch_c_reason = if branch != "c" {
        "n/a".to_string()
    } else if n_pos_prefix < 3 {
        format!("the positive prefix is {n_pos_prefix}, under three — the prereg's own (c)")
    } else {
        format!("the positive prefix is {n_pos_prefix}, but no set of three or more is fitted within the derived tolerance — (c) by the harvest rule, NOT by the prefix count")
    };
    eprintln!(
        "S1: branch ({branch}) — {}",
        match branch {
            "a" => "the wall is one exponential over the whole range and is transferred in full".to_string(),
            "b" => format!("a prefix of {k} fits (R_x = {r_x} Å); the wall from the prefix, the outer miss reported"),
            _ => format!("VOID: no wall, the arms do not run. {branch_c_reason}"),
        }
    );

    // ------------------------------------- the penetration term: FIELD-5's fit, FROZEN
    let s_ho = |n: &XNode| -> f64 { n.ho.iter().map(|d| (-c_coef * d).exp()).sum() };
    let p_fit = |n: &XNode| -p_coef * s_ho(n);
    eprintln!("\nthe penetration term is NOT re-fit: P = {p_coef:.9e} Ha, c = {c_coef:.2} /bohr — {PEN_NOTE}");
    for n in &nodes {
        eprintln!("  {:.1} Å: p_fit {:+.6e} Ha ({:+.4} of ΔE_exact)", n.r_ang, p_fit(n), p_fit(n) / n.de_exact.abs());
    }

    // ---------------------- dispersion C6 from what is left on the OUTER nodes only
    let wall_in_remainder = k > 0;
    let rem = |n: &XNode| n.de_exact - n.e_q_raw - p_fit(n) - n.hlu.e_exch - wall_at(n.r_bohr);
    let (mut num, mut den) = (0.0, 0.0);
    for n in &nodes[OUTER_FROM..] {
        let w = 1.0 / (n.de_exact * n.de_exact);
        let x = -1.0 / n.r_bohr.powi(6);
        num += w * rem(n) * x;
        den += w * x * x;
    }
    let mut c6 = if den > 0.0 { num / den } else { 0.0 };
    let mut slopes: Vec<(f64, f64)> = Vec::new();
    eprintln!(
        "\ndispersion from the remainder ΔE_exact − E_q − p_fit − E_exch(undeformed){}, OUTER nodes only",
        if wall_in_remainder { " − wall_fit" } else { " (the wall was NOT harvested — the remainder is on E_exch alone, as FIELD-5 did)" }
    );
    for n in &nodes {
        eprintln!(
            "  {:.1} Å: remainder {:+.6e} Ha ({:+.4} of ΔE_exact){}",
            n.r_ang,
            rem(n),
            rem(n) / n.de_exact.abs(),
            if n.r_ang >= NODES_ANGSTROM[OUTER_FROM] - 1e-9 { "  [in the fit]" } else { "  [inner, NOT fit]" }
        );
    }
    for i in OUTER_FROM + 1..nodes.len() {
        let (a1, a0) = (rem(&nodes[i]), rem(&nodes[i - 1]));
        let s = if a1 != 0.0 && a0 != 0.0 { (a1.abs() / a0.abs()).ln() / (nodes[i].r_bohr / nodes[i - 1].r_bohr).ln() } else { f64::NAN };
        eprintln!("  log-log slope {:.1} → {:.1} Å: {s:.3}", nodes[i - 1].r_ang, nodes[i].r_ang);
        slopes.push((nodes[i].r_ang, s));
    }
    let c6_transferred = !slopes.is_empty() && slopes.iter().all(|(_, s)| s.is_finite() && *s >= SLOPE_LO && *s <= SLOPE_HI);
    if !c6_transferred {
        c6 = 0.0;
    }
    eprintln!(
        "dispersion: C₆ = {c6:.9e} Ha·bohr⁶; every slope in [{SLOPE_LO}, {SLOPE_HI}] → {}",
        if c6_transferred { "TRANSFERRED" } else { "NOT transferred (C₆ = 0 recorded)" }
    );

    // ------------------------------------- plant (ii): the DEFORMED referee in its place
    eprintln!("\nplant (ii) — FIELD-5's orthogonalised E_exch in place of the undeformed one, the same derived tolerance rule computed from ITS values:");
    for (i, n) in nodes.iter().enumerate() {
        eprintln!("  {:.1} Å: E_exch^orth {:+.6e}, tolerance {:.6e}", n.r_ang, n.hlo.e_exch, tol_o[i]);
    }
    let (best_o, attempts_o, prefix_o) = fit_wall(&r_ang, &r_bohr, &de, &val_o, &tol_o, "orthogonalised");
    let carrier_ii = nodes[iref].hlo.e_exch - nodes[iref].hlu.e_exch;
    let carrier_ii_ok = carrier_ii >= PLANT_II_CARRIER;
    let plant_ii_fires = best_o.is_none() && carrier_ii_ok;
    eprintln!(
        "plant (ii): carrier E_exch^orth({REF_ANGSTROM:.1}) − E_exch({REF_ANGSTROM:.1}) = {carrier_ii:.6e} Ha ≥ {PLANT_II_CARRIER:e}: {carrier_ii_ok}; every set of three or more FAILS: {} (positive prefix {prefix_o}) → {}",
        best_o.is_none(),
        if plant_ii_fires { "FIRES" } else { "does not fire" }
    );

    // ----------------------------------------------- G-C1 and plant (i), on the engine
    let model = SeamModel { a: a_coef, b: b_coef, p: p_coef, c: c_coef, c6, ..SeamModel::NO_WALL };
    eprintln!("\nG-C1 — the harvest is the engine's arithmetic, the SAME reference on both sides (E_q(R) − E_q(40) from the engine itself)");
    let mut g_c1_worst = 0.0f64;
    let mut c1_lines: Vec<String> = Vec::new();
    let mut plant_i = (f64::NAN, f64::NAN, f64::NAN, false);
    for n in &nodes {
        let (a_f, b_f) = linear(o, h, n.r_ang);
        let (e_int, e_field_diff, e_seam_diff) = engine_interaction(&a_f, &b_f, Some(model), SeamPlant::None);
        let (pen, wall, disp) = formula_terms(&a_f, &b_f, &model);
        let want = e_field_diff + pen + wall + disp;
        let miss = (e_int - want).abs();
        g_c1_worst = g_c1_worst.max(miss);
        eprintln!(
            "  R {:.1} Å: engine {e_int:+.12e} (field diff {e_field_diff:+.6e}, seam {e_seam_diff:+.6e}) vs formula {want:+.12e} (E_q(R) − E_q(40) {e_field_diff:+.6e} + pen {pen:+.6e} + wall {wall:+.6e} + disp {disp:+.6e}) — miss {miss:.3e}",
            n.r_ang
        );
        c1_lines.push(format!(
            "{{\"r_angstrom\": {:.1}, \"engine_interaction\": {e_int:+.12e}, \"formula\": {want:+.12e}, \"miss\": {miss:.3e}, \"e_q_difference\": {e_field_diff:+.12e}, \"engine_seam\": {e_seam_diff:+.12e}, \"pen\": {pen:+.12e}, \"wall\": {wall:+.12e}, \"disp\": {disp:+.12e}}}",
            n.r_ang
        ));
        if (n.r_ang - REF_ANGSTROM).abs() < 1e-9 {
            let (e_pl, _, _) = engine_interaction(&a_f, &b_f, Some(model), SeamPlant::FlipPenetration);
            let observed = (e_pl - e_int).abs();
            let expected = 2.0 * pen.abs();
            let carrier = pen.abs();
            let fires = carrier >= PLANT_I_CARRIER && (observed - expected).abs() <= G_C1_TOL;
            plant_i = (observed, expected, carrier, fires);
            eprintln!(
                "plant (i) at {REF_ANGSTROM:.1} Å: miss {observed:.6e} vs 2·|p_HO| {expected:.6e} (difference {:.3e}); carrier |p_HO| {carrier:.3e} ≥ {PLANT_I_CARRIER:e}: {} → {}",
                (observed - expected).abs(),
                carrier >= PLANT_I_CARRIER,
                if fires { "FIRES" } else { "does not fire" }
            );
        }
    }
    let g_c1 = g_c1_worst <= G_C1_TOL;
    eprintln!("G-C1: worst |engine − formula| = {g_c1_worst:.3e} (stake {G_C1_TOL:e}) → {}", if g_c1 { "PASS" } else { "FAIL" });

    // ------------------------------------------------------------------------ wall6.json
    let node_lines: Vec<String> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            format!(
                "{{\"r_angstrom\": {:.1}, \"r_bohr\": {:.6}, \"de_exact\": {:+.12e}, \"e_super\": {:.12e}, \"e_q_raw\": {:+.12e}, \"e_hl_undeformed\": {:+.12e}, \"e_hl_orth\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"e_exch_undeformed\": {:+.12e}, \"e_exch_orth\": {:+.12e}, \"norm_undeformed\": {:.15e}, \"norm_orth\": {:.15e}, \"fit_tolerance\": {:.12e}, \"wall_fit\": {:+.12e}, \"in_wall_fit\": {}, \"p_fit\": {:+.12e}, \"in_pen_fit\": {}, \"remainder\": {:+.12e}, \"nonzero_dets\": {}, \"n_det\": {}, \"s_cross_max\": {:.6e}, \"sigma_seconds\": {:.3}, \"cpu_seconds\": {:.3}, \"cpu_seconds_orth\": {:.3}, \"cross_ho_bohr\": [{}]}}",
                n.r_ang,
                n.r_bohr,
                n.de_exact,
                n.e_super,
                n.e_q_raw,
                n.hlu.e_hl,
                n.hlo.e_hl,
                n.hlu.e_a0,
                n.hlu.e_b0,
                n.hlu.e_es,
                n.hlu.e_exch,
                n.hlo.e_exch,
                n.hlu.norm,
                n.hlo.norm,
                tol_u[i],
                wall_at(n.r_bohr),
                i < k,
                p_fit(n),
                i >= OUTER_FROM,
                rem(n),
                n.hlu.nonzero_dets,
                n.hlu.n_det,
                n.hlu.s_cross_max,
                n.hlu.sigma_seconds,
                n.cpu_u,
                n.cpu_o,
                n.ho.iter().map(|d| format!("{d:.6}")).collect::<Vec<_>>().join(", ")
            )
        })
        .collect();
    let slope_lines: Vec<String> = slopes.iter().map(|(r, s)| format!("{{\"r_angstrom\": {r:.1}, \"loglog_slope_from_previous\": {s:.6}}}")).collect();
    fs::write(
        out.join("wall6.json"),
        format!(
            "{{\n  \"a\": {a_coef:.12e}, \"b\": {b_coef:.12e}, \"p\": {p_coef:.12e}, \"c\": {c_coef:.12e}, \"c6\": {c6:.12e},\n  \"r_x_angstrom\": {r_x}, \"fit_nodes\": {k}, \"s1_branch\": \"{branch}\", \"c6_transferred\": {c6_transferred},\n  \"positive_prefix\": {n_pos_prefix}, \"branch_c_reason\": \"{branch_c_reason}\",\n  \"referee\": \"heitler_london_undeformed — the antisymmetrised product of the monomers' own wavefunctions, the monomers NOT deformed\",\n  \"g_u0\": {{\"pass\": {g_u0}, \"norm_ok\": {norm_ok}, \"norm_window\": \"({NORM_LO}, {NORM_HI}]\", \"norm_lowest\": {norm_lowest:.15e}, \"norm_highest\": {norm_highest:.15e}, \"norm_far\": {:.15e}, \"norm_orth_worst_from_one\": {:.6e}, \"order_ok\": {order_ok}, \"order_slack\": {ORDER_SLACK:e}, \"limit_ok\": {limit_ok}, \"limit_abs_e_exch\": {far_exch:.6e}, \"limit_abs_residual\": {far_resid:.6e}, \"limit_tolerance\": {LIMIT_TOL:e}, \"e_hl_far_undeformed\": {:+.12e}, \"e_hl_far_orth\": {:+.12e}, \"e_a0_far\": {:+.12e}, \"e_b0_far\": {:+.12e}}},\n  \"h1\": {{\"pass\": {h1}, \"all_above_floor\": {h1_positive}, \"non_increasing_outward\": {h1_monotone}, \"floor\": {R_FLOOR:e}, \"ratio_node_angstrom\": {REF_ANGSTROM:.1}, \"ratio_required_below\": {H1_RATIO}, \"ratio_measured\": {h1_ratio_value:.6}, \"ratio_ok\": {h1_ratio}}},\n  \"price\": {{\"tenth_of_price_core_seconds\": {HL_PRICE_TENTH_CORE_S}, \"every_reading_at_or_above\": {price_ok}, \"cheapest_core_seconds\": {:.3}}},\n  \"fit_tolerance_rule\": {{\"derived\": true, \"rule\": \"max({FIT_TOL_DE}·|ΔE_exact(R)|, {FIT_TOL_EXCH}·E_exch(R))\", \"frac_de\": {FIT_TOL_DE}, \"frac_exch\": {FIT_TOL_EXCH}}},\n  \"pen_fit\": {{\"refit\": false, \"source\": \"field5/wall5.json\", \"note\": \"{PEN_NOTE}\", \"placement\": \"cross-unit H–O\", \"p\": {p_coef:.12e}, \"c\": {c_coef:.12e}}},\n  \"dispersion\": {{\"nodes\": \"outer only, {} of 6\", \"remainder\": \"de_exact − e_q_raw − p_fit − e_exch_undeformed{}\", \"wall_in_remainder\": {wall_in_remainder}, \"c6\": {c6:.12e}, \"transferred\": {c6_transferred}, \"slope_band\": [{SLOPE_LO}, {SLOPE_HI}]}},\n  \"g_c1\": {{\"pass\": {g_c1}, \"worst_miss\": {g_c1_worst:.6e}, \"tolerance\": {G_C1_TOL:e}, \"reference\": \"E_q(R) − E_q(40 bohr), the engine's own field on both sides\"}},\n  \"plant_i\": {{\"miss_observed\": {:.6e}, \"miss_expected\": {:.6e}, \"carrier_p_ho\": {:.6e}, \"carrier_floor\": {PLANT_I_CARRIER:e}, \"fires\": {}}},\n  \"plant_ii\": {{\"fires\": {plant_ii_fires}, \"every_set_of_three_or_more_fails\": {}, \"positive_prefix\": {prefix_o}, \"carrier\": {carrier_ii:.6e}, \"carrier_floor\": {PLANT_II_CARRIER:e}, \"carrier_present\": {carrier_ii_ok}, \"carrier_definition\": \"E_exch^orth({REF_ANGSTROM:.1} Å) − E_exch^undeformed({REF_ANGSTROM:.1} Å)\", \"attempts\": [{}]}},\n  \"wall_fit_attempts\": [{}],\n  \"remainder_slopes\": [{}],\n  \"g_c1_nodes\": [\n    {}\n  ],\n  \"nodes\": [\n    {}\n  ]\n}}\n",
            far_u.norm,
            nodes.iter().map(|n| (n.hlo.norm - 1.0).abs()).fold(0.0f64, f64::max),
            far_u.e_hl,
            far_o.e_hl,
            far_u.e_a0,
            far_u.e_b0,
            nodes.iter().map(|n| n.cpu_u).fold(f64::INFINITY, f64::min),
            nodes.len() - OUTER_FROM,
            if wall_in_remainder { " − wall_fit" } else { "" },
            plant_i.0,
            plant_i.1,
            plant_i.2,
            plant_i.3,
            best_o.is_none(),
            attempts_o.join(", "),
            attempts.join(", "),
            slope_lines.join(", "),
            c1_lines.join(",\n    "),
            node_lines.join(",\n    ")
        ),
    )
    .unwrap();
    eprintln!("\nwall6.json written");

    // --------------------------------------- prediction.json, BEFORE the held-out solve
    let (a_t, b_t) = tilted(o, h, HELD_ANGSTROM, HELD_DEGREES);
    let r_oo_t = cross_oo(&a_t, &b_t);
    assert!(
        (r_oo_t - HELD_ANGSTROM * ANGSTROM_TO_BOHR).abs() < 1e-9,
        "the rotation is about the acceptor's own oxygen: R_OO must be unchanged ({r_oo_t:.9} vs {:.9})",
        HELD_ANGSTROM * ANGSTROM_TO_BOHR
    );
    let (e_pred, e_q_t, e_seam_t) = engine_interaction(&a_t, &b_t, Some(model), SeamPlant::None);
    let (pen_t, wall_t, disp_t) = formula_terms(&a_t, &b_t, &model);
    let s_t = engine_dimer(&a_t, &b_t, Some(model), SeamPlant::None);
    let ho_t = cross_ho(&a_t, &b_t);
    fs::write(
        out.join("prediction.json"),
        format!(
            "{{\n  \"node\": \"tilted{HELD_DEGREES:.0}_R{HELD_ANGSTROM:.1}\", \"r_oo_angstrom\": {HELD_ANGSTROM:.3}, \"r_oo_bohr\": {r_oo_t:.6}, \"tilt_degrees\": {HELD_DEGREES:.1}, \"tilt_axis\": \"x, through the acceptor's own oxygen\", \"held_out\": true, \"units\": {},\n  \"e_pred\": {e_pred:+.12e},\n  \"parts\": {{\"e_q_difference\": {e_q_t:+.12e}, \"p_ho\": {pen_t:+.12e}, \"wall\": {wall_t:+.12e}, \"disp\": {disp_t:+.12e}, \"engine_seam\": {e_seam_t:+.12e}}},\n  \"coefficients\": {{\"a\": {a_coef:.12e}, \"b\": {b_coef:.12e}, \"p\": {p_coef:.12e}, \"c\": {c_coef:.12e}, \"c6\": {c6:.12e}}},\n  \"wall_harvested\": {}, \"c6_transferred\": {c6_transferred}, \"s1_branch\": \"{branch}\",\n  \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\", \"tolerance_frac\": {PRED_FRAC}, \"tolerance_abs\": {PRED_ABS:e},\n  \"cross_ho_bohr\": [{}],\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
            s_t.seam_work.units,
            k > 0,
            ho_t.iter().map(|d| format!("{d:.6}")).collect::<Vec<_>>().join(", "),
            centers_json(&a_t),
            centers_json(&b_t)
        ),
    )
    .unwrap();
    eprintln!(
        "prediction.json filed BEFORE the held-out solve: E_pred {e_pred:+.6e} Ha — E_q(R) − E_q(40) {e_q_t:+.6e}, p_HO {pen_t:+.6e}, wall {wall_t:+.6e}{}, disp {disp_t:+.6e}; units {}",
        if k > 0 { "" } else { " (NOT harvested; 0.0 recorded)" },
        s_t.seam_work.units
    );
    fs::write(out.join("exchange.done"), "done\n").unwrap();
}

// --------------------------------------------------------------------------- predict (S2)

/// One free reading: the undeformed referee on a geometry whose exact value was already
/// known when this freeze was written, beside the wall's value there. NOT a stake (§4).
fn free_reading(name: &str, note: &str, a: &Fragment, b: &Fragment, a_coef: f64, b_coef: f64) -> String {
    let r_oo = cross_oo(a, b);
    let t0 = Instant::now();
    let c0 = cpu_seconds();
    let hl = heitler_london_undeformed(a, b);
    let wall_s = t0.elapsed().as_secs_f64();
    let cpu = cpu_seconds() - c0;
    let wall_v = a_coef * (-b_coef * r_oo).exp();
    eprintln!(
        "free reading — {name} (R_OO {r_oo:.4} bohr): E_exch(undeformed) {:+.6e} Ha vs wall {wall_v:+.6e}, difference {:+.6e}. NOT a stake.",
        hl.e_exch,
        wall_v - hl.e_exch
    );
    format!(
        "{{\"node\": \"{name}\", \"note\": \"{note}\", \"stake\": false, \"r_oo_bohr\": {r_oo:.6}, \"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"e_exch\": {:+.12e}, \"wall_value\": {wall_v:+.12e}, \"wall_minus_e_exch\": {:+.12e}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"s_cross_max\": {:.6e}, \"sigma_seconds\": {:.3}, \"cpu_seconds\": {cpu:.3}, \"wall_seconds\": {wall_s:.3}}}",
        hl.e_hl,
        hl.e_a0,
        hl.e_b0,
        hl.e_es,
        hl.e_exch,
        wall_v - hl.e_exch,
        hl.norm,
        hl.nonzero_dets,
        hl.n_det,
        hl.s_cross_max,
        hl.sigma_seconds,
    )
}

fn run_predict(out: &Path) {
    let pred_path = out.join("prediction.json");
    let Ok(pred) = fs::read_to_string(&pred_path) else {
        eprintln!("{} missing: the prediction is filed BEFORE the solve (run `exchange` first). Nothing written.", pred_path.display());
        std::process::exit(2);
    };
    let e_pred = json_num(&pred, "e_pred");
    let wall6 = fs::read_to_string(out.join("wall6.json")).expect("wall6.json: run `exchange` first");
    let (a_coef, b_coef) = (json_num(&wall6, "a"), json_num(&wall6, "b"));
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let (a, b) = tilted(o, h, HELD_ANGSTROM, HELD_DEGREES);
    let name = format!("tilted{HELD_DEGREES:.0}_R{HELD_ANGSTROM:.1}");
    eprintln!("FIELD-6 predict — the HELD-OUT node ({HELD_DEGREES:.0}° about x through the acceptor's oxygen, R_OO {HELD_ANGSTROM:.1} Å) on {} threads; E_pred {e_pred:+.12e} Ha", threads());

    // the exact solve first: the prediction is already on disk
    let ok = solve_node(out, &name, HELD_ANGSTROM, &a, &b, true);
    let t = fs::read_to_string(out.join(format!("{name}.json"))).unwrap();
    let de = json_num(&t, "de_exact");
    let n_det_ok = (json_num(&t, "n_det") as usize) == N_DET_DIMER;
    let tol = (PRED_FRAC * de.abs()).max(PRED_ABS);
    let miss = (e_pred - de).abs();

    // then both referees on the same geometry: where the miss lives
    let r_oo_t = cross_oo(&a, &b);
    let t0 = Instant::now();
    let c0 = cpu_seconds();
    let hl_u = heitler_london_undeformed(&a, &b);
    let hl_u_wall = t0.elapsed().as_secs_f64();
    let hl_u_cpu = cpu_seconds() - c0;
    let t1 = Instant::now();
    let c1 = cpu_seconds();
    let hl_o = heitler_london(&a, &b, HlPlant::None);
    let hl_o_wall = t1.elapsed().as_secs_f64();
    let hl_o_cpu = cpu_seconds() - c1;
    let wall_held = a_coef * (-b_coef * r_oo_t).exp();
    let wall_gap = (wall_held - hl_u.e_exch).abs();
    let s2 = if miss <= tol {
        "a"
    } else if wall_gap <= tol {
        "b"
    } else {
        "c"
    };
    eprintln!(
        "S2: ΔE_exact {de:+.6e} Ha, E_pred {e_pred:+.6e} — miss {miss:.3e} ({:.1} % of |ΔE_exact|) against {tol:.3e}; E_exch(undeformed, held-out) {:+.6e} (orthogonalised {:+.6e}) vs wall(R_OO = {r_oo_t:.4} bohr) {wall_held:+.6e}, difference {wall_gap:.3e} → branch ({s2})",
        100.0 * miss / de.abs(),
        hl_u.e_exch,
        hl_o.e_exch
    );
    fs::write(
        out.join("prediction_check.json"),
        format!(
            "{{\n  \"node\": \"{name}\", \"e_pred\": {e_pred:+.12e}, \"de_exact\": {de:+.12e},\n  \"miss\": {miss:.6e}, \"miss_fraction\": {:.6}, \"tolerance\": {tol:.6e}, \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\",\n  \"exact\": {{\"converged\": {ok}, \"exit\": \"{}\", \"davidson_iters\": {}, \"residual\": {:.3e}, \"residual_bar\": {RESIDUAL_BAR:e}, \"n_det\": {}, \"n_det_expected\": {N_DET_DIMER}, \"n_det_ok\": {n_det_ok}, \"cpu_seconds\": {:.1}, \"wall_seconds\": {:.1}}},\n  \"exchange_on_the_held_out_node\": {{\"r_oo_bohr\": {r_oo_t:.6}, \"e_exch_undeformed\": {:+.12e}, \"e_exch_orth\": {:+.12e}, \"e_hl_undeformed\": {:+.12e}, \"e_hl_orth\": {:+.12e}, \"e_es\": {:+.12e}, \"norm_undeformed\": {:.15e}, \"wall_value\": {wall_held:+.12e}, \"wall_minus_e_exch\": {:+.12e}, \"abs_difference\": {wall_gap:.6e}, \"within_tolerance\": {}, \"nonzero_dets\": {}, \"sigma_seconds\": {:.3}, \"cpu_seconds_undeformed\": {hl_u_cpu:.3}, \"wall_seconds_undeformed\": {hl_u_wall:.3}, \"cpu_seconds_orth\": {hl_o_cpu:.3}, \"wall_seconds_orth\": {hl_o_wall:.3}}},\n  \"wall\": {{\"a\": {a_coef:.12e}, \"b\": {b_coef:.12e}}},\n  \"s2_branch\": \"{s2}\"\n}}\n",
            miss / de.abs(),
            json_str(&t, "exit"),
            json_num(&t, "davidson_iters") as u64,
            json_num(&t, "residual"),
            json_num(&t, "n_det") as u64,
            json_num(&t, "cpu_seconds"),
            json_num(&t, "wall_seconds"),
            hl_u.e_exch,
            hl_o.e_exch,
            hl_u.e_hl,
            hl_o.e_hl,
            hl_u.e_es,
            hl_u.norm,
            wall_held - hl_u.e_exch,
            wall_gap <= tol,
            hl_u.nonzero_dets,
            hl_u.sigma_seconds,
        ),
    )
    .unwrap();

    // ------------------------------------------------------- the two FREE readings (§4)
    let (a5, b5) = tilted(o, h, TILT5_ANGSTROM, TILT5_DEGREES);
    let (a4, b4) = flipped(o, h, FLIPPED_ANGSTROM);
    let r5 = free_reading(
        &format!("tilted{TILT5_DEGREES:.0}_R{TILT5_ANGSTROM:.1}"),
        "FIELD6_PREREG §4: FIELD-5's tilted node, its exact value already known — a FREE reading beside the wall's value",
        &a5,
        &b5,
        a_coef,
        b_coef,
    );
    let r4 = free_reading(
        &format!("flipped_R{FLIPPED_ANGSTROM:.1}"),
        "FIELD6_PREREG §4: FIELD-4's flipped node, its exact value already known — a FREE reading beside the wall's value",
        &a4,
        &b4,
        a_coef,
        b_coef,
    );
    fs::write(
        out.join("free_readings.json"),
        format!("{{\n  \"stake\": false, \"referee\": \"heitler_london_undeformed\", \"wall\": {{\"a\": {a_coef:.12e}, \"b\": {b_coef:.12e}}},\n  \"readings\": [\n    {r5},\n    {r4}\n  ]\n}}\n"),
    )
    .unwrap();
    fs::write(out.join("predict.done"), "done\n").unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let what = args.get(1).map(String::as_str).unwrap_or("exchange");
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/field6".to_string()));
    fs::create_dir_all(&out).expect("out");
    match what {
        "exchange" => run_exchange(&out),
        "predict" => run_predict(&out),
        other => eprintln!("unknown phase {other} (exchange | predict)"),
    }
}
