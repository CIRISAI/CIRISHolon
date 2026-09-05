//! FIELD-5's harvest (`conformance/water_observatory/FIELD5_PREREG.md` §0, §2, §5, §6):
//! EXCHANGE in the embedding. The Heitler–London referee on FIELD-3's six linear nodes, the
//! wall harvested from `E_exch(R)`, the penetration term re-fit on the OUTER nodes where
//! FIELD-4 showed the density field is a field, dispersion from what is left, G-C1 and plant
//! (i) on the engine, and a BENT hydrogen bond predicted forward — filed before its solve.
//!
//! ```text
//! cargo run --release -p holon-render --example field5_harvest -- exchange   [OUT_DIR]
//! cargo run --release -p holon-render --example field5_harvest -- invariance [OUT_DIR]
//! cargo run --release -p holon-render --example field5_harvest -- predict    [OUT_DIR]
//! ```
//!
//! `exchange`: the six `E_exch` readings and the 40-bohr limit (G-H1), H1, the three fits,
//! `wall5.json`, G-C1 and plant (i) by the engine, and `prediction.json` for the TILTED node
//! written BEFORE that node is solved. `invariance`: G-H0 (the dimer's full CI in the
//! orthogonalised basis against FIELD-3's `e_super`) and plant (ii) (the orthogonalisation
//! skipped) — two full solves. `predict`: refuses without `prediction.json`, solves the
//! tilted node exactly, then reads `E_exch` on it (and, free, on FIELD-4's flipped node).
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::{solve_embedded, supermolecule, water_dimer_linear, Fragment, ANGSTROM_TO_BOHR};
use holon_chem::fci::SolveExit;
use holon_chem::heitler_london::{fci_in_hl_basis, heitler_london, HlPlant, HlReading};
use holon_render::seam::{SeamModel, SeamPlant};
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/quartet.rs"]
#[allow(dead_code)]
mod quartet;

/// EMBED-1's water pins — the same numbers FIELD-3's and FIELD-4's runners carry.
const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;
/// The six linear nodes (Å), SHORTEST FIRST (the harvest reads prefixes).
const NODES_ANGSTROM: [f64; 6] = [2.5, 2.7, 2.9, 3.1, 3.4, 3.7];
/// The OUTER nodes begin here: 2.9, 3.1, 3.4, 3.7 Å — where FIELD-4 measured that the
/// density field is a field. The penetration and dispersion fits use these four only.
const OUTER_FROM: usize = 2;
/// S2's held-out geometry: the 2.9 Å linear dimer with the acceptor tilted 30° about x.
const TILT_ANGSTROM: f64 = 2.9;
const TILT_DEGREES: f64 = 30.0;
/// FIELD-4's flipped node — a FREE reading here, never a stake (§4).
const FLIPPED_ANGSTROM: f64 = 3.4;
/// The separation at which the acceptor is "away" (bohr): G-H1's limit and the engine's
/// reference on both sides of G-C1.
const FAR_BOHR: f64 = 40.0;

/// The residual bar every exact solve must meet (EMBED-3's).
const RESIDUAL_BAR: f64 = 1e-9;
/// The reading floor on every harvested residual (M-FLOOR-UNSTAKED).
const R_FLOOR: f64 = 1e-6;
/// S1's tolerance on the wall fit, as a fraction of `|ΔE_exact|`.
const FIT_TOL: f64 = 0.10;
/// S2's tolerance: `max(0.25·|ΔE_exact|, 5e-4)`.
const PRED_FRAC: f64 = 0.25;
const PRED_ABS: f64 = 5e-4;
/// The declared grid on the penetration exponent (per bohr): 0.50, 0.51, …, 4.00.
const C_MIN: f64 = 0.50;
const C_STEP: f64 = 0.01;
const C_STEPS: usize = 350;
/// The band the remainder's log-log slope must lie in for `C₆` to transfer.
const SLOPE_LO: f64 = -8.0;
const SLOPE_HI: f64 = -4.0;
/// G-H1: the product state's norm, and its 40-bohr limit.
const NORM_TOL: f64 = 1e-12;
const LIMIT_TOL: f64 = 1e-8;
/// G-H0: the full CI in the orthogonalised basis against FIELD-3's record.
const G_H0_TOL: f64 = 1e-8;
/// Plant (ii): the miss the skipped orthogonalisation must produce, and its carrier.
const PLANT_II_MIN: f64 = 1e-2;
const S_CROSS_MIN: f64 = 1e-3;
/// G-C1's tolerance, and plant (i)'s carrier.
const G_C1_TOL: f64 = 1e-10;
const PLANT_I_CARRIER: f64 = 1e-4;
/// The determinant count FIELD-3's supermolecule carries (EXACT).
const N_DET_DIMER: usize = 1_002_001;
/// M-CHEAPER-THAN-ITS-PRICE: a Heitler–London evaluation is priced at one Davidson
/// iteration of FIELD-3's record, 70–270 core-seconds; a reading under a TENTH of that
/// price is refused. Recorded per node.
const HL_PRICE_TENTH_CORE_S: f64 = 7.0;

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

/// S2's held-out geometry: the linear dimer with the ACCEPTOR rotated by `theta_degrees`
/// about the x-axis through its OWN oxygen. The donor and its O–H are untouched, and the
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
/// `solve_node` verbatim, so the tilted node's record is the same object FIELD-3 wrote.
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

fn field3_dir(out: &Path) -> PathBuf {
    let sib = out.parent().unwrap_or(Path::new(".")).join("field3");
    if sib.exists() {
        sib
    } else {
        PathBuf::from("../conformance/water_observatory/field3")
    }
}

fn field4_dir(out: &Path) -> PathBuf {
    let sib = out.parent().unwrap_or(Path::new(".")).join("field4");
    if sib.exists() {
        sib
    } else {
        PathBuf::from("../conformance/water_observatory/field4")
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

// ------------------------------------------------------------------------ the exchange phase

struct XNode {
    r_ang: f64,
    r_bohr: f64,
    de_exact: f64,
    /// FIELD-3's RAW near-box engine field of record (`wall.json`'s `e_field`).
    e_q_raw: f64,
    /// FIELD-4's penetration-and-induction residual of record (`wall4.json`'s per-node `p`).
    p4: f64,
    ho: Vec<f64>,
    hl: HlReading,
    cpu: f64,
}

/// Whether the winning exponent sits on the declared grid's boundary — a fit pinned at the
/// edge is a fit the grid did not contain, and the record says so rather than reading it.
fn at_edge(c: f64) -> bool {
    (c - C_MIN).abs() < 0.5 * C_STEP || (c - (C_MIN + C_STEP * C_STEPS as f64)).abs() < 0.5 * C_STEP
}

/// `S_c` at a node: the engine's own sum over cross-unit H–O pairs.
fn s_ho(n: &XNode, c: f64) -> f64 {
    n.ho.iter().map(|d| (-c * d).exp()).sum()
}

/// `(P, c)` by the declared grid: for each `c`, the weighted least-squares `P` of
/// `p ≈ −P·S_c`, weights `1/ΔE_exact²`; the `c` of least weighted residual. FIELD-4's
/// procedure verbatim, restricted to the nodes handed in.
fn fit_pen(used: &[&XNode]) -> (f64, f64, f64) {
    let mut best = (0.0f64, f64::NAN, f64::INFINITY);
    for i in 0..=C_STEPS {
        let c = C_MIN + C_STEP * i as f64;
        let mut num = 0.0;
        let mut den = 0.0;
        for n in used {
            let w = 1.0 / (n.de_exact * n.de_exact);
            let s = s_ho(n, c);
            num += w * n.p4 * s;
            den += w * s * s;
        }
        if den == 0.0 || !den.is_finite() {
            continue;
        }
        let p = -num / den;
        let resid: f64 = used
            .iter()
            .map(|n| {
                let w = 1.0 / (n.de_exact * n.de_exact);
                let e = n.p4 + p * s_ho(n, c);
                w * e * e
            })
            .sum();
        if resid < best.2 {
            best = (p, c, resid);
        }
    }
    best
}

fn run_exchange(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let f3 = field3_dir(out);
    let f4 = field4_dir(out);
    let wall3 = fs::read_to_string(f3.join("wall.json")).unwrap_or_else(|_| panic!("{}/wall.json missing: FIELD-3's engine field is the E_q of record", f3.display()));
    let wall4 = fs::read_to_string(f4.join("wall4.json")).unwrap_or_else(|_| panic!("{}/wall4.json missing: FIELD-4's p(R) is the penetration record", f4.display()));
    eprintln!("FIELD-5 exchange — the Heitler–London referee on FIELD-3's six linear nodes, {} threads", threads());

    // ---------------------------------------------------------------- the six readings
    let mut nodes: Vec<XNode> = Vec::new();
    for &r in NODES_ANGSTROM.iter() {
        let node_path = f3.join(format!("linear_R{r:.1}.json"));
        let t3 = fs::read_to_string(&node_path).unwrap_or_else(|_| panic!("{} missing", node_path.display()));
        let de_exact = json_num(&t3, "de_exact");
        let e_q_raw = node_field(&wall3, r, "e_field");
        let p4 = node_field(&wall4, r, "p");
        assert!(e_q_raw.is_finite(), "FIELD-3's wall.json carries no e_field at R = {r} Å");
        assert!(p4.is_finite(), "FIELD-4's wall4.json carries no p at R = {r} Å");

        let (a, b) = linear(o, h, r);
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let hl = heitler_london(&a, &b, HlPlant::None);
        let wall_s = t0.elapsed().as_secs_f64();
        let cpu = cpu_seconds() - c0;
        let ho = cross_ho(&a, &b);
        let r_bohr = cross_oo(&a, &b);
        let priced = cpu >= HL_PRICE_TENTH_CORE_S;
        let dets_expected = hl.n_det_a * hl.n_det_b;

        fs::write(
            out.join(format!("exchange_R{r:.1}.json")),
            format!(
                "{{\n  \"node\": \"exchange_R{r:.1}\", \"r_oo_angstrom\": {r:.3}, \"r_oo_bohr\": {r_bohr:.6},\n  \"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"e_exch\": {:+.12e},\n  \"e_hl_minus_monomers\": {:+.12e},\n  \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"n_det_a\": {}, \"n_det_b\": {}, \"nonzero_dets_expected\": {dets_expected},\n  \"s_cross_max\": {:.12e}, \"sigma_seconds\": {:.3},\n  \"de_exact\": {de_exact:+.12e}, \"e_q_raw_field3\": {e_q_raw:+.12e}, \"p_field4\": {p4:+.12e},\n  \"cross_ho_bohr\": [{}],\n  \"wall_seconds\": {wall_s:.3}, \"cpu_seconds\": {cpu:.3}, \"price_tenth_core_seconds\": {HL_PRICE_TENTH_CORE_S}, \"price_floor_met\": {priced}, \"threads\": {}\n}}\n",
                hl.e_hl,
                hl.e_a0,
                hl.e_b0,
                hl.e_es,
                hl.e_exch,
                hl.e_hl - hl.e_a0 - hl.e_b0,
                hl.norm,
                hl.nonzero_dets,
                hl.n_det,
                hl.n_det_a,
                hl.n_det_b,
                hl.s_cross_max,
                hl.sigma_seconds,
                ho.iter().map(|d| format!("{d:.6}")).collect::<Vec<_>>().join(", "),
                threads(),
            ),
        )
        .unwrap();
        eprintln!(
            "  R_OO {r:.1} Å: E_HL {:+.9e}  E_es {:+.6e}  E_exch {:+.6e}  norm {:.15}  nonzero {}/{}  S_max {:.3e}  σ {:.1} s, {cpu:.1} core-s",
            hl.e_hl, hl.e_es, hl.e_exch, hl.norm, hl.nonzero_dets, dets_expected, hl.s_cross_max, hl.sigma_seconds
        );
        nodes.push(XNode { r_ang: r, r_bohr, de_exact, e_q_raw, p4, ho, hl, cpu });
    }

    // ------------------------------------------------- G-H1: the product state is what it says
    // the 40-bohr limit, on the 2.9 Å node's geometry with the acceptor translated along x
    let (a29, b29) = linear(o, h, TILT_ANGSTROM);
    let b_far = b29.translated([FAR_BOHR, 0.0, 0.0]);
    let t0 = Instant::now();
    let c0 = cpu_seconds();
    let far = heitler_london(&a29, &b_far, HlPlant::None);
    let far_wall = t0.elapsed().as_secs_f64();
    let far_cpu = cpu_seconds() - c0;
    let far_miss = (far.e_hl - far.e_a0 - far.e_b0).abs();
    fs::write(
        out.join("exchange_far.json"),
        format!(
            "{{\n  \"node\": \"exchange_far_{FAR_BOHR:.0}bohr\", \"base_r_oo_angstrom\": {TILT_ANGSTROM:.3}, \"acceptor_translated_bohr\": [{FAR_BOHR:.1}, 0.0, 0.0], \"r_oo_bohr\": {:.6},\n  \"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"e_exch\": {:+.12e},\n  \"limit_miss\": {far_miss:.6e}, \"limit_tolerance\": {LIMIT_TOL:e},\n  \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"n_det_a\": {}, \"n_det_b\": {}, \"s_cross_max\": {:.12e},\n  \"sigma_seconds\": {:.3}, \"wall_seconds\": {far_wall:.3}, \"cpu_seconds\": {far_cpu:.3}\n}}\n",
            cross_oo(&a29, &b_far),
            far.e_hl,
            far.e_a0,
            far.e_b0,
            far.e_es,
            far.e_exch,
            far.norm,
            far.nonzero_dets,
            far.n_det,
            far.n_det_a,
            far.n_det_b,
            far.s_cross_max,
            far.sigma_seconds,
        ),
    )
    .unwrap();

    let norm_worst = nodes.iter().map(|n| (n.hl.norm - 1.0).abs()).fold(0.0f64, f64::max).max((far.norm - 1.0).abs());
    let norm_ok = norm_worst <= NORM_TOL;
    let dets_ok = nodes.iter().all(|n| n.hl.nonzero_dets == n.hl.n_det_a * n.hl.n_det_b);
    let dets_product = nodes[0].hl.n_det_a * nodes[0].hl.n_det_b;
    let limit_ok = far_miss <= LIMIT_TOL;
    let g_h1 = norm_ok && dets_ok && limit_ok;
    eprintln!("\nG-H1:");
    eprintln!("  norm: worst |‖Ψ‖ − 1| = {norm_worst:.3e} over seven readings (tol {NORM_TOL:e}) → {}", if norm_ok { "PASS" } else { "FAIL" });
    eprintln!(
        "  count: nonzero determinants {} = n_det(A)·n_det(B) = {} · {} = {dets_product} at every node (the freeze names 441 × 441 = 194,481) → {}",
        nodes[0].hl.nonzero_dets,
        nodes[0].hl.n_det_a,
        nodes[0].hl.n_det_b,
        if dets_ok { "PASS" } else { "FAIL" }
    );
    eprintln!("  limit: |E_HL(40 bohr) − E_A0 − E_B0| = {far_miss:.3e} Ha (tol {LIMIT_TOL:e}) → {}", if limit_ok { "PASS" } else { "FAIL" });
    eprintln!("G-H1 → {}", if g_h1 { "PASS" } else { "FAIL" });

    // ---------------------------------------------------- H1: exchange is a wall
    eprintln!("\n| R (Å) | R_OO (bohr) | ΔE_exact (Ha) | E_HL − E_A0 − E_B0 (Ha) | E_es (Ha) | E_exch (Ha) | σ (s) | core-s |");
    for n in &nodes {
        eprintln!(
            "| {:.1} | {:.4} | {:+.6e} | {:+.6e} | {:+.6e} | {:+.6e} | {:.1} | {:.1} |",
            n.r_ang,
            n.r_bohr,
            n.de_exact,
            n.hl.e_hl - n.hl.e_a0 - n.hl.e_b0,
            n.hl.e_es,
            n.hl.e_exch,
            n.hl.sigma_seconds,
            n.cpu
        );
    }
    let h1_positive = nodes.iter().all(|n| n.hl.e_exch > R_FLOOR);
    let h1_monotone = (1..nodes.len()).all(|i| nodes[i].hl.e_exch <= nodes[i - 1].hl.e_exch);
    let h1 = h1_positive && h1_monotone;
    eprintln!(
        "H1: E_exch > {R_FLOOR:e} at all six ({h1_positive}); non-increasing outward ({h1_monotone}) → {}",
        if h1 { "PASS" } else { "FAIL" }
    );
    let price_ok = nodes.iter().all(|n| n.cpu >= HL_PRICE_TENTH_CORE_S);
    eprintln!(
        "M-CHEAPER-THAN-ITS-PRICE (recorded): every reading at or above a tenth of its {HL_PRICE_TENTH_CORE_S:.0} core-second price: {price_ok} (cheapest {:.1} core-s)",
        nodes.iter().map(|n| n.cpu).fold(f64::INFINITY, f64::min)
    );

    // ------------------------------------------------- fit (a): the wall (A, b) on E_exch
    let positive: Vec<bool> = nodes.iter().map(|n| n.hl.e_exch > R_FLOOR).collect();
    let n_pos_prefix = positive.iter().take_while(|&&p| p).count();
    let mut best: Option<(usize, f64, f64)> = None;
    let mut attempts: Vec<String> = Vec::new();
    eprintln!("\nfit (a) the wall over E_exch — FIELD-3's rule: the largest contiguous set of the SHORTEST nodes, at least three, within {FIT_TOL}·|ΔE_exact| at each");
    for k in (3..=n_pos_prefix).rev() {
        let sub = &nodes[..k];
        let w: Vec<f64> = sub.iter().map(|n| 1.0 / (n.de_exact * n.de_exact)).collect();
        let sw: f64 = w.iter().sum();
        let mx = sub.iter().zip(&w).map(|(n, w)| w * n.r_bohr).sum::<f64>() / sw;
        let my = sub.iter().zip(&w).map(|(n, w)| w * n.hl.e_exch.ln()).sum::<f64>() / sw;
        let sxx: f64 = sub.iter().zip(&w).map(|(n, w)| w * (n.r_bohr - mx) * (n.r_bohr - mx)).sum();
        let sxy: f64 = sub.iter().zip(&w).map(|(n, w)| w * (n.r_bohr - mx) * (n.hl.e_exch.ln() - my)).sum();
        let slope = sxy / sxx; // ln E_exch = ln A − b R
        let b = -slope;
        let ln_a = my - slope * mx;
        let within = sub.iter().all(|n| ((ln_a - b * n.r_bohr).exp() - n.hl.e_exch).abs() <= FIT_TOL * n.de_exact.abs());
        let worst = sub.iter().map(|n| ((ln_a - b * n.r_bohr).exp() - n.hl.e_exch).abs() / n.de_exact.abs()).fold(0.0, f64::max);
        attempts.push(format!("{{\"k\": {k}, \"r_x_angstrom\": {:.1}, \"a\": {:.12e}, \"b\": {b:.12e}, \"worst_miss_over_de\": {worst:.6}, \"qualifies\": {within}}}", nodes[k - 1].r_ang, ln_a.exp()));
        eprintln!(
            "  attempt k = {k} (to {:.1} Å): A = {:.6e} Ha, b = {b:.6} /bohr, worst miss {worst:.4} of |ΔE_exact| (tol {FIT_TOL}) → {}",
            nodes[k - 1].r_ang,
            ln_a.exp(),
            if within { "QUALIFIES" } else { "does not qualify" }
        );
        if within {
            best = Some((k, ln_a, b));
            break;
        }
    }
    let (k, a_coef, b_coef) = match best {
        Some((k, ln_a, b)) => (k, ln_a.exp(), b),
        None => (0, 0.0, 0.0),
    };
    let r_x = if k > 0 { nodes[k - 1].r_ang } else { f64::NAN };
    let wall_at = |r_bohr: f64| if k > 0 { a_coef * (-b_coef * r_bohr).exp() } else { 0.0 };
    eprintln!("fit (a): positive prefix {n_pos_prefix} of 6; A = {a_coef:.9e} Ha, b = {b_coef:.6} /bohr over the shortest {k} nodes (R_x = {r_x} Å)");
    if k > 0 {
        for n in &nodes {
            let f = wall_at(n.r_bohr);
            eprintln!(
                "  {:.1} Å: E_exch {:+.6e}  wall {:+.6e}  miss {:+.6e} ({:.4} of |ΔE_exact|){}",
                n.r_ang,
                n.hl.e_exch,
                f,
                f - n.hl.e_exch,
                (f - n.hl.e_exch).abs() / n.de_exact.abs(),
                if nodes.iter().position(|m| (m.r_ang - n.r_ang).abs() < 1e-9).unwrap() < k { "  [in the fit]" } else { "" }
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
        format!("the positive prefix is {n_pos_prefix}, but no set of three or more is fitted within {FIT_TOL}·|ΔE_exact| — (c) by the harvest rule, NOT by the prefix count")
    };
    eprintln!(
        "S1: branch ({branch}) — {}",
        match branch {
            "a" => "the wall is one exponential over the whole range and is transferred in full".to_string(),
            "b" => format!("a prefix of {k} fits (R_x = {r_x} Å); the wall from the prefix, the outer miss reported"),
            _ => format!("VOID: no wall, the arms do not run. {branch_c_reason}"),
        }
    );

    // ------------------------- fit (b): the penetration term (P, c) on the OUTER nodes only
    let outer: Vec<&XNode> = nodes[OUTER_FROM..].iter().collect();
    let outer_readings: Vec<bool> = outer.iter().map(|n| n.p4.abs() >= R_FLOOR).collect();
    let used: Vec<&XNode> = outer.iter().zip(&outer_readings).filter(|(_, &rd)| rd).map(|(n, _)| *n).collect();
    let (p_coef, c_coef, resid_pen) = fit_pen(&used);
    eprintln!(
        "\nfit (b) the penetration term on the OUTER nodes only ({} of them, H–O placement, c-grid {C_MIN}–{} step {C_STEP}): P = {p_coef:.9e} Ha, c = {c_coef:.2} /bohr (weighted residual {resid_pen:.6e}, at grid edge {})",
        used.len(),
        C_MIN + C_STEP * C_STEPS as f64,
        at_edge(c_coef)
    );
    let p_fit = |n: &XNode| -p_coef * s_ho(n, c_coef);
    for n in &nodes {
        let f = p_fit(n);
        eprintln!(
            "  {:.1} Å: p(FIELD-4) {:+.6e}  fit {:+.6e}  miss {:+.6e} ({:+.4} of ΔE_exact){}",
            n.r_ang,
            n.p4,
            f,
            f - n.p4,
            (f - n.p4) / n.de_exact.abs(),
            if n.r_ang >= NODES_ANGSTROM[OUTER_FROM] - 1e-9 { "  [in the fit]" } else { "  [inner, NOT fit]" }
        );
    }

    // ------------------- fit (c): dispersion C6 from what is left on the OUTER nodes only
    let rem = |n: &XNode| n.de_exact - n.e_q_raw - p_fit(n) - n.hl.e_exch;
    let (mut num, mut den) = (0.0, 0.0);
    for n in &nodes[OUTER_FROM..] {
        let w = 1.0 / (n.de_exact * n.de_exact);
        let x = -1.0 / n.r_bohr.powi(6);
        num += w * rem(n) * x;
        den += w * x * x;
    }
    let mut c6 = if den > 0.0 { num / den } else { 0.0 };
    let mut slopes: Vec<(f64, f64)> = Vec::new();
    eprintln!("\nfit (c) dispersion from the remainder ΔE_exact − E_q − p_fit − E_exch, OUTER nodes only");
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
        "fit (c): C₆ = {c6:.9e} Ha·bohr⁶; every slope in [{SLOPE_LO}, {SLOPE_HI}] → {}",
        if c6_transferred { "TRANSFERRED" } else { "NOT transferred (C₆ = 0 recorded)" }
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
        if (n.r_ang - TILT_ANGSTROM).abs() < 1e-9 {
            let (e_pl, _, _) = engine_interaction(&a_f, &b_f, Some(model), SeamPlant::FlipPenetration);
            let observed = (e_pl - e_int).abs();
            let expected = 2.0 * pen.abs();
            let carrier = pen.abs();
            let fires = carrier >= PLANT_I_CARRIER && (observed - expected).abs() <= G_C1_TOL;
            plant_i = (observed, expected, carrier, fires);
            eprintln!(
                "plant (i) at {TILT_ANGSTROM:.1} Å: miss {observed:.6e} vs 2·|p_HO| {expected:.6e} (difference {:.3e}); carrier |p_HO| {carrier:.3e} ≥ {PLANT_I_CARRIER:e}: {} → {}",
                (observed - expected).abs(),
                carrier >= PLANT_I_CARRIER,
                if fires { "FIRES" } else { "does not fire" }
            );
        }
    }
    let g_c1 = g_c1_worst <= G_C1_TOL;
    eprintln!("G-C1: worst |engine − formula| = {g_c1_worst:.3e} (stake {G_C1_TOL:e}) → {}", if g_c1 { "PASS" } else { "FAIL" });

    // ------------------------------------------------------------------------ wall5.json
    let node_lines: Vec<String> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            format!(
                "{{\"r_angstrom\": {:.1}, \"r_bohr\": {:.6}, \"de_exact\": {:+.12e}, \"e_q_raw\": {:+.12e}, \"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"e_exch\": {:+.12e}, \"wall_fit\": {:+.12e}, \"in_wall_fit\": {}, \"p\": {:+.12e}, \"p_fit\": {:+.12e}, \"in_pen_fit\": {}, \"remainder\": {:+.12e}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"s_cross_max\": {:.6e}, \"sigma_seconds\": {:.3}, \"cpu_seconds\": {:.3}, \"cross_ho_bohr\": [{}]}}",
                n.r_ang,
                n.r_bohr,
                n.de_exact,
                n.e_q_raw,
                n.hl.e_hl,
                n.hl.e_a0,
                n.hl.e_b0,
                n.hl.e_es,
                n.hl.e_exch,
                wall_at(n.r_bohr),
                i < k,
                n.p4,
                p_fit(n),
                i >= OUTER_FROM,
                rem(n),
                n.hl.norm,
                n.hl.nonzero_dets,
                n.hl.n_det,
                n.hl.s_cross_max,
                n.hl.sigma_seconds,
                n.cpu,
                n.ho.iter().map(|d| format!("{d:.6}")).collect::<Vec<_>>().join(", ")
            )
        })
        .collect();
    let slope_lines: Vec<String> = slopes.iter().map(|(r, s)| format!("{{\"r_angstrom\": {r:.1}, \"loglog_slope_from_previous\": {s:.6}}}")).collect();
    fs::write(
        out.join("wall5.json"),
        format!(
            "{{\n  \"a\": {a_coef:.12e}, \"b\": {b_coef:.12e}, \"p\": {p_coef:.12e}, \"c\": {c_coef:.12e}, \"c6\": {c6:.12e},\n  \"r_x_angstrom\": {r_x}, \"fit_nodes\": {k}, \"s1_branch\": \"{branch}\", \"c6_transferred\": {c6_transferred},\n  \"positive_prefix\": {n_pos_prefix}, \"branch_c_reason\": \"{branch_c_reason}\",\n  \"g_h1\": {{\"pass\": {g_h1}, \"norm_worst\": {norm_worst:.6e}, \"norm_tolerance\": {NORM_TOL:e}, \"norm_ok\": {norm_ok}, \"nonzero_dets\": {}, \"n_det_a\": {}, \"n_det_b\": {}, \"nonzero_dets_expected\": {dets_product}, \"count_ok\": {dets_ok}, \"limit_miss\": {far_miss:.6e}, \"limit_tolerance\": {LIMIT_TOL:e}, \"limit_ok\": {limit_ok}, \"e_hl_far\": {:+.12e}, \"e_a0_far\": {:+.12e}, \"e_b0_far\": {:+.12e}}},\n  \"h1\": {{\"pass\": {h1}, \"all_above_floor\": {h1_positive}, \"non_increasing_outward\": {h1_monotone}, \"floor\": {R_FLOOR:e}}},\n  \"price\": {{\"tenth_of_price_core_seconds\": {HL_PRICE_TENTH_CORE_S}, \"every_reading_at_or_above\": {price_ok}, \"cheapest_core_seconds\": {:.3}}},\n  \"pen_fit\": {{\"placement\": \"cross-unit H–O\", \"nodes\": \"outer only, {} of 6\", \"weighted_residual\": {resid_pen:.12e}, \"c_grid\": \"{C_MIN} to {} step {C_STEP} per bohr\", \"c_at_grid_edge\": {}}},\n  \"g_c1\": {{\"pass\": {g_c1}, \"worst_miss\": {g_c1_worst:.6e}, \"tolerance\": {G_C1_TOL:e}, \"reference\": \"E_q(R) − E_q(40 bohr), the engine's own field on both sides\"}},\n  \"plant_i\": {{\"miss_observed\": {:.6e}, \"miss_expected\": {:.6e}, \"carrier_p_ho\": {:.6e}, \"carrier_floor\": {PLANT_I_CARRIER:e}, \"fires\": {}}},\n  \"wall_fit_attempts\": [{}],\n  \"remainder_slopes\": [{}],\n  \"g_c1_nodes\": [\n    {}\n  ],\n  \"nodes\": [\n    {}\n  ]\n}}\n",
            nodes[0].hl.nonzero_dets,
            nodes[0].hl.n_det_a,
            nodes[0].hl.n_det_b,
            far.e_hl,
            far.e_a0,
            far.e_b0,
            nodes.iter().map(|n| n.cpu).fold(f64::INFINITY, f64::min),
            used.len(),
            C_MIN + C_STEP * C_STEPS as f64,
            at_edge(c_coef),
            plant_i.0,
            plant_i.1,
            plant_i.2,
            plant_i.3,
            attempts.join(", "),
            slope_lines.join(", "),
            c1_lines.join(",\n    "),
            node_lines.join(",\n    ")
        ),
    )
    .unwrap();
    eprintln!("\nwall5.json written");

    // ------------------------------------------- prediction.json, BEFORE the tilted solve
    let (a_t, b_t) = tilted(o, h, TILT_ANGSTROM, TILT_DEGREES);
    let r_oo_t = cross_oo(&a_t, &b_t);
    assert!(
        (r_oo_t - TILT_ANGSTROM * ANGSTROM_TO_BOHR).abs() < 1e-9,
        "the tilt rotates about the acceptor's own oxygen: R_OO must be unchanged ({r_oo_t:.9} vs {:.9})",
        TILT_ANGSTROM * ANGSTROM_TO_BOHR
    );
    let (e_pred, e_q_t, e_seam_t) = engine_interaction(&a_t, &b_t, Some(model), SeamPlant::None);
    let (pen_t, wall_t, disp_t) = formula_terms(&a_t, &b_t, &model);
    let s_t = engine_dimer(&a_t, &b_t, Some(model), SeamPlant::None);
    let ho_t = cross_ho(&a_t, &b_t);
    fs::write(
        out.join("prediction.json"),
        format!(
            "{{\n  \"node\": \"tilted_R{TILT_ANGSTROM:.1}\", \"r_oo_angstrom\": {TILT_ANGSTROM:.3}, \"r_oo_bohr\": {r_oo_t:.6}, \"tilt_degrees\": {TILT_DEGREES:.1}, \"tilt_axis\": \"x, through the acceptor's own oxygen\", \"units\": {},\n  \"e_pred\": {e_pred:+.12e},\n  \"parts\": {{\"e_q_difference\": {e_q_t:+.12e}, \"p_ho\": {pen_t:+.12e}, \"wall\": {wall_t:+.12e}, \"disp\": {disp_t:+.12e}, \"engine_seam\": {e_seam_t:+.12e}}},\n  \"coefficients\": {{\"a\": {a_coef:.12e}, \"b\": {b_coef:.12e}, \"p\": {p_coef:.12e}, \"c\": {c_coef:.12e}, \"c6\": {c6:.12e}}},\n  \"wall_harvested\": {}, \"c6_transferred\": {c6_transferred}, \"s1_branch\": \"{branch}\",\n  \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\", \"tolerance_frac\": {PRED_FRAC}, \"tolerance_abs\": {PRED_ABS:e},\n  \"cross_ho_bohr\": [{}],\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
            s_t.seam_work.units,
            k > 0,
            ho_t.iter().map(|d| format!("{d:.6}")).collect::<Vec<_>>().join(", "),
            centers_json(&a_t),
            centers_json(&b_t)
        ),
    )
    .unwrap();
    eprintln!(
        "prediction.json filed BEFORE the tilted solve: E_pred {e_pred:+.6e} Ha — E_q(R) − E_q(40) {e_q_t:+.6e}, p_HO {pen_t:+.6e}, wall {wall_t:+.6e}{}, disp {disp_t:+.6e}; units {}",
        if k > 0 { "" } else { " (NOT harvested; 0.0 recorded)" },
        s_t.seam_work.units
    );
    fs::write(out.join("exchange.done"), "done\n").unwrap();
}

// ------------------------------------------------- the invariance phase (G-H0, plant (ii))

fn run_invariance(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let f3 = field3_dir(out);
    let rec = fs::read_to_string(f3.join(format!("linear_R{TILT_ANGSTROM:.1}.json"))).expect("FIELD-3's 2.9 Å node");
    let e_super = json_num(&rec, "e_super");
    eprintln!("FIELD-5 invariance — G-H0 and plant (ii) on the {TILT_ANGSTROM:.1} Å node, {} threads; FIELD-3's record E_super = {e_super:.12e} Ha", threads());
    let (a, b) = linear(o, h, TILT_ANGSTROM);

    // the carrier for plant (ii): the largest cross-fragment orbital overlap, read from the
    // unplanted instrument on the same node — asserted nonzero in the sector the plant acts on
    let hl = heitler_london(&a, &b, HlPlant::None);
    let carrier = hl.s_cross_max;
    let carrier_ok = carrier >= S_CROSS_MIN;
    eprintln!("plant (ii) carrier: largest cross-fragment orbital overlap {carrier:.6e} ≥ {S_CROSS_MIN:e} → {}", if carrier_ok { "present" } else { "ABSENT (the plant is void)" });

    // G-H0: the dimer's full CI in the orthogonalised basis
    let t0 = Instant::now();
    let c0 = cpu_seconds();
    let (e_ortho, sol) = fci_in_hl_basis(&a, &b, HlPlant::None);
    let wall_ortho = t0.elapsed().as_secs_f64();
    let cpu_ortho = cpu_seconds() - c0;
    let miss = (e_ortho - e_super).abs();
    let converged = matches!(sol.exit, SolveExit::Converged) && sol.residual <= RESIDUAL_BAR;
    let dets_ok = hl.n_det == N_DET_DIMER;
    let g_h0 = miss <= G_H0_TOL && converged && dets_ok;
    eprintln!(
        "G-H0: full CI over C′ = {e_ortho:.12e} Ha vs the record {e_super:.12e} — miss {miss:.3e} (tol {G_H0_TOL:e}); {} iters, residual {:.3e}, exit {}, {} determinants (want {N_DET_DIMER}); wall {wall_ortho:.0} s, {cpu_ortho:.0} core-s → {}",
        sol.davidson_iters,
        sol.residual,
        exit_name(&sol.exit),
        hl.n_det,
        if g_h0 { "PASS" } else { "FAIL" }
    );

    // plant (ii): the orthogonalisation skipped
    let t1 = Instant::now();
    let c1 = cpu_seconds();
    let (e_skip, sol_skip) = fci_in_hl_basis(&a, &b, HlPlant::SkipOrthogonalisation);
    let wall_skip = t1.elapsed().as_secs_f64();
    let cpu_skip = cpu_seconds() - c1;
    let skip_miss = (e_skip - e_super).abs();
    let skip_vs_ortho = (e_skip - e_ortho).abs();
    let fires = carrier_ok && skip_miss >= PLANT_II_MIN;
    eprintln!(
        "plant (ii): full CI with the block-diagonal orbitals used as if orthonormal = {e_skip:.12e} Ha — misses the record by {skip_miss:.6e} (≥ {PLANT_II_MIN:e} required), misses G-H0's own answer by {skip_vs_ortho:.6e}; {} iters, residual {:.3e}, exit {}, wall {wall_skip:.0} s, {cpu_skip:.0} core-s → {}",
        sol_skip.davidson_iters,
        sol_skip.residual,
        exit_name(&sol_skip.exit),
        if fires { "FIRES" } else { "does not fire" }
    );

    fs::write(
        out.join("invariance.json"),
        format!(
            "{{\n  \"node\": \"invariance_R{TILT_ANGSTROM:.1}\", \"r_oo_angstrom\": {TILT_ANGSTROM:.3}, \"threads\": {},\n  \"g_h0\": {{\"pass\": {g_h0}, \"e_fci_in_hl_basis\": {e_ortho:.12e}, \"e_super_field3\": {e_super:.12e}, \"miss\": {miss:.6e}, \"tolerance\": {G_H0_TOL:e}, \"davidson_iters\": {}, \"residual\": {:.6e}, \"exit\": \"{}\", \"converged\": {converged}, \"residual_bar\": {RESIDUAL_BAR:e}, \"n_det\": {}, \"n_det_expected\": {N_DET_DIMER}, \"n_det_ok\": {dets_ok}, \"wall_seconds\": {wall_ortho:.1}, \"cpu_seconds\": {cpu_ortho:.1}}},\n  \"plant_ii\": {{\"fires\": {fires}, \"e_fci_skip_orthogonalisation\": {e_skip:.12e}, \"miss_vs_record\": {skip_miss:.6e}, \"miss_vs_g_h0\": {skip_vs_ortho:.6e}, \"required\": {PLANT_II_MIN:e}, \"carrier_s_cross_max\": {carrier:.6e}, \"carrier_floor\": {S_CROSS_MIN:e}, \"carrier_present\": {carrier_ok}, \"davidson_iters\": {}, \"residual\": {:.6e}, \"exit\": \"{}\", \"wall_seconds\": {wall_skip:.1}, \"cpu_seconds\": {cpu_skip:.1}}}\n}}\n",
            threads(),
            sol.davidson_iters,
            sol.residual,
            exit_name(&sol.exit),
            hl.n_det,
            sol_skip.davidson_iters,
            sol_skip.residual,
            exit_name(&sol_skip.exit),
        ),
    )
    .unwrap();
    fs::write(out.join("invariance.done"), "done\n").unwrap();
    eprintln!("invariance.json written");
}

// --------------------------------------------------------------------------- predict (S2)

fn run_predict(out: &Path) {
    let pred_path = out.join("prediction.json");
    let Ok(pred) = fs::read_to_string(&pred_path) else {
        eprintln!("{} missing: the prediction is filed BEFORE the solve (run `exchange` first). Nothing written.", pred_path.display());
        std::process::exit(2);
    };
    let e_pred = json_num(&pred, "e_pred");
    let wall5 = fs::read_to_string(out.join("wall5.json")).expect("wall5.json: run `exchange` first");
    let (a_coef, b_coef) = (json_num(&wall5, "a"), json_num(&wall5, "b"));
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let (a, b) = tilted(o, h, TILT_ANGSTROM, TILT_DEGREES);
    let name = format!("tilted_R{TILT_ANGSTROM:.1}");
    eprintln!("FIELD-5 predict — the TILTED node ({TILT_DEGREES:.0}° about x through the acceptor's oxygen) on {} threads; E_pred {e_pred:+.12e} Ha", threads());

    // the exact solve first: the prediction is already on disk
    let ok = solve_node(out, &name, TILT_ANGSTROM, &a, &b, false);
    let t = fs::read_to_string(out.join(format!("{name}.json"))).unwrap();
    let de = json_num(&t, "de_exact");
    let tol = (PRED_FRAC * de.abs()).max(PRED_ABS);
    let miss = (e_pred - de).abs();

    // then the referee on the same geometry: where the miss lives
    let r_oo_t = cross_oo(&a, &b);
    let t0 = Instant::now();
    let c0 = cpu_seconds();
    let hl_t = heitler_london(&a, &b, HlPlant::None);
    let hl_wall_s = t0.elapsed().as_secs_f64();
    let hl_cpu = cpu_seconds() - c0;
    let wall_tilted = a_coef * (-b_coef * r_oo_t).exp();
    let wall_gap = (wall_tilted - hl_t.e_exch).abs();
    let s2 = if miss <= tol {
        "a"
    } else if wall_gap <= tol {
        "b"
    } else {
        "c"
    };
    eprintln!(
        "S2: ΔE_exact {de:+.6e} Ha, E_pred {e_pred:+.6e} — miss {miss:.3e} ({:.1} % of |ΔE_exact|) against {tol:.3e}; E_exch(tilted) {:+.6e} vs wall(R_OO = {r_oo_t:.4} bohr) {wall_tilted:+.6e}, difference {wall_gap:.3e} → branch ({s2})",
        100.0 * miss / de.abs(),
        hl_t.e_exch
    );
    fs::write(
        out.join("prediction_check.json"),
        format!(
            "{{\n  \"node\": \"{name}\", \"e_pred\": {e_pred:+.12e}, \"de_exact\": {de:+.12e},\n  \"miss\": {miss:.6e}, \"miss_fraction\": {:.6}, \"tolerance\": {tol:.6e}, \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\",\n  \"exact\": {{\"converged\": {ok}, \"exit\": \"{}\", \"davidson_iters\": {}, \"residual\": {:.3e}, \"n_det\": {}, \"cpu_seconds\": {:.1}, \"wall_seconds\": {:.1}}},\n  \"exchange_on_the_tilted_node\": {{\"r_oo_bohr\": {r_oo_t:.6}, \"e_exch\": {:+.12e}, \"e_hl\": {:+.12e}, \"e_es\": {:+.12e}, \"wall_value\": {wall_tilted:+.12e}, \"wall_minus_e_exch\": {:+.12e}, \"abs_difference\": {wall_gap:.6e}, \"within_tolerance\": {}, \"norm\": {:.15e}, \"nonzero_dets\": {}, \"sigma_seconds\": {:.3}, \"cpu_seconds\": {hl_cpu:.3}, \"wall_seconds\": {hl_wall_s:.3}}},\n  \"wall\": {{\"a\": {a_coef:.12e}, \"b\": {b_coef:.12e}}},\n  \"s2_branch\": \"{s2}\"\n}}\n",
            miss / de.abs(),
            json_str(&t, "exit"),
            json_num(&t, "davidson_iters") as u64,
            json_num(&t, "residual"),
            json_num(&t, "n_det") as u64,
            json_num(&t, "cpu_seconds"),
            json_num(&t, "wall_seconds"),
            hl_t.e_exch,
            hl_t.e_hl,
            hl_t.e_es,
            wall_tilted - hl_t.e_exch,
            wall_gap <= tol,
            hl_t.norm,
            hl_t.nonzero_dets,
            hl_t.sigma_seconds,
        ),
    )
    .unwrap();

    // FIELD-4's FLIPPED node: a FREE reading (§4), recorded beside the wall's value there.
    // NOT a stake — its exact value was already known when this was written.
    let (a_f, b_f) = flipped(o, h, FLIPPED_ANGSTROM);
    let r_oo_f = cross_oo(&a_f, &b_f);
    let t1 = Instant::now();
    let c1 = cpu_seconds();
    let hl_f = heitler_london(&a_f, &b_f, HlPlant::None);
    let f_wall_s = t1.elapsed().as_secs_f64();
    let f_cpu = cpu_seconds() - c1;
    let wall_flipped = a_coef * (-b_coef * r_oo_f).exp();
    eprintln!(
        "free reading — FIELD-4's flipped node at {FLIPPED_ANGSTROM:.1} Å (R_OO {r_oo_f:.4} bohr): E_exch {:+.6e} Ha vs wall {wall_flipped:+.6e}, difference {:+.6e}. NOT a stake.",
        hl_f.e_exch,
        wall_flipped - hl_f.e_exch
    );
    fs::write(
        out.join("flipped_exchange.json"),
        format!(
            "{{\n  \"node\": \"flipped_R{FLIPPED_ANGSTROM:.1}\", \"r_oo_angstrom\": {FLIPPED_ANGSTROM:.3}, \"r_oo_bohr\": {r_oo_f:.6},\n  \"stake\": false, \"note\": \"FIELD5_PREREG §4: a FREE reading beside the wall's value; the flipped node's exact value was already known\",\n  \"e_hl\": {:+.12e}, \"e_a0\": {:+.12e}, \"e_b0\": {:+.12e}, \"e_es\": {:+.12e}, \"e_exch\": {:+.12e},\n  \"wall_value\": {wall_flipped:+.12e}, \"wall_minus_e_exch\": {:+.12e},\n  \"norm\": {:.15e}, \"nonzero_dets\": {}, \"n_det\": {}, \"s_cross_max\": {:.6e}, \"sigma_seconds\": {:.3}, \"cpu_seconds\": {f_cpu:.3}, \"wall_seconds\": {f_wall_s:.3}\n}}\n",
            hl_f.e_hl,
            hl_f.e_a0,
            hl_f.e_b0,
            hl_f.e_es,
            hl_f.e_exch,
            wall_flipped - hl_f.e_exch,
            hl_f.norm,
            hl_f.nonzero_dets,
            hl_f.n_det,
            hl_f.s_cross_max,
            hl_f.sigma_seconds,
        ),
    )
    .unwrap();
    fs::write(out.join("predict.done"), "done\n").unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let what = args.get(1).map(String::as_str).unwrap_or("exchange");
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/field5".to_string()));
    fs::create_dir_all(&out).expect("out");
    match what {
        "exchange" => run_exchange(&out),
        "invariance" => run_invariance(&out),
        "predict" => run_predict(&out),
        other => eprintln!("unknown phase {other} (exchange | invariance | predict)"),
    }
}
