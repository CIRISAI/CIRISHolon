//! FIELD-3 part C (`conformance/water_observatory/FIELD3_PREREG.md`, AMENDMENT 1): the
//! exchange wall HARVESTED from the exact dimer's residual over the field.
//!
//! ```text
//! cargo run --release -p holon-render --example field3_harvest -- solve   [OUT_DIR]
//! cargo run --release -p holon-render --example field3_harvest -- fit     [OUT_DIR]
//! cargo run --release -p holon-render --example field3_harvest -- predict [OUT_DIR]
//! ```
//!
//! `solve`: the six LINEAR nodes on the seam programme's exact solver (`supermolecule`,
//! 1,002,001 determinants), the 2.9 Å price node first (G-C0, priced in core-seconds by
//! AMENDMENT 1), one JSON per node, skipping nodes already on disk. `fit`: the engine's
//! field on each node's geometry (the closure assignment, the pin charge), the residual,
//! the weighted log-linear wall fit over the shortest positive nodes it fits (S1), G-C1 and
//! plant (i) on the engine, `wall.json`, and `prediction.json` for the FLIPPED node —
//! written BEFORE that node is solved. `predict`: refuses to run without
//! `prediction.json`, then solves the flipped node and writes the comparison (S2).
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::{solve_embedded, supermolecule, water_dimer_linear, Fragment, ANGSTROM_TO_BOHR};
use holon_chem::fci::SolveExit;
use holon_render::seam::{SeamModel, SeamPlant};
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/quartet.rs"]
#[allow(dead_code)]
mod quartet;

/// EMBED-1's water pins — the same numbers `holon_render::field::WATER_PIN_*` carry.
const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;
/// The six linear nodes (Å), the price node first.
const NODES_ANGSTROM: [f64; 6] = [2.9, 2.5, 2.7, 3.1, 3.4, 3.7];
const FLIPPED_ANGSTROM: f64 = 3.4;
/// G-C0 as frozen (wall-seconds) and as amended (core-seconds, AMENDMENT 1).
const PRICE_CEILING_WALL_S: f64 = 1800.0;
const PRICE_CEILING_CORE_S: f64 = 57600.0;
const PRICE_FLOOR_CORE_S: f64 = 1450.0;
/// The residual bar every node must meet (EMBED-3's).
const RESIDUAL_BAR: f64 = 1e-9;
/// The harvest's floor on the residual `r(R)` (the freeze's M-FLOOR-UNSTAKED discharge).
const R_FLOOR: f64 = 1e-6;
/// S1's tolerance on the fit, as a fraction of `|ΔE_exact|`.
const FIT_TOL: f64 = 0.10;
/// S2's tolerance: `max(0.25·|ΔE_exact|, 5e-4)`.
const PRED_FRAC: f64 = 0.25;
const PRED_ABS: f64 = 5e-4;
const G_C1_TOL: f64 = 1e-10;

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

fn linear(o: Species, h: Species, r_oo_angstrom: f64) -> (Fragment, Fragment) {
    water_dimer_linear(o, h, H2O_R, H2O_THETA, r_oo_angstrom * ANGSTROM_TO_BOHR)
}

/// The FLIPPED dimer: the linear donor, and the acceptor rotated by π about the x-axis
/// through its oxygen — its hydrogens toward the donor.
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

fn centers_json(f: &Fragment) -> String {
    f.centers.iter().map(|c| format!("[{:.10}, {:.10}, {:.10}]", c[0], c[1], c[2])).collect::<Vec<_>>().join(", ")
}

fn exit_name(e: &SolveExit) -> String {
    format!("{e:?}")
}

/// One exact node: the supermolecule, the monomer references, the record.
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
    let admitted_wall = if price { wall <= PRICE_CEILING_WALL_S } else { true };
    let admitted_core = if price { cpu >= PRICE_FLOOR_CORE_S && cpu <= PRICE_CEILING_CORE_S } else { true };
    fs::write(
        &path,
        format!(
            "{{\n  \"node\": \"{name}\", \"r_oo_angstrom\": {r_oo_angstrom:.3}, \"r_oo_bohr\": {:.6},\n  \"n_det\": {}, \"e_super\": {:.12e}, \"e_a0\": {:.12e}, \"e_b0\": {:.12e}, \"de_exact\": {:.12e},\n  \"davidson_iters\": {}, \"residual\": {:.3e}, \"exit\": \"{}\", \"converged\": {converged},\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}],\n  \"wall_seconds\": {wall:.1}, \"cpu_seconds\": {cpu:.1}, \"threads\": {}, \"price_node\": {price}, \"admitted\": {admitted_wall}, \"admitted_core_seconds_amendment_1\": {admitted_core}\n}}\n",
            r_oo_angstrom * ANGSTROM_TO_BOHR,
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
        "  {name}: R_OO {r_oo_angstrom:.1} Å, {} dets, ΔE_exact {de:+.6e} Ha, {} iters, residual {:.1e}, exit {}, wall {wall:.0} s, {cpu:.0} core-s{}",
        sm.gp.space.n_det,
        sm.sol.davidson_iters,
        sm.sol.residual,
        exit_name(&sm.sol.exit),
        if price { if admitted_core { " — G-C0 ADMITTED (A1)" } else { " — G-C0 REFUSED (A1)" } } else { "" }
    );
    admitted_core && converged
}

fn run_solve(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    eprintln!("FIELD-3 C — the six linear nodes on {} threads", threads());
    for (k, &r) in NODES_ANGSTROM.iter().enumerate() {
        let (a, b) = linear(o, h, r);
        let ok = solve_node(out, &format!("linear_R{r:.1}"), r, &a, &b, k == 0);
        if k == 0 && !ok {
            eprintln!("the price node was refused or did not converge; the harvest stops here");
            fs::write(out.join("solve.done"), "REFUSED\n").unwrap();
            return;
        }
    }
    fs::write(out.join("solve.done"), "done\n").unwrap();
    eprintln!("solve: done");
}

// ------------------------------------------------------------------------------ fit

struct Node {
    r_ang: f64,
    r_bohr: f64,
    de_exact: f64,
    cpu: f64,
    converged: bool,
    exit: String,
    iters: u64,
    residual: f64,
    e_field: f64,
    r: f64,
}

fn json_num(t: &str, key: &str) -> f64 {
    t.split(&format!("\"{key}\": ")).nth(1).and_then(|x| x.split(|c| c == ',' || c == '\n' || c == '}').next()).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::NAN)
}
fn json_str(t: &str, key: &str) -> String {
    t.split(&format!("\"{key}\": \"")).nth(1).and_then(|x| x.split('"').next()).unwrap_or("").to_string()
}

/// The engine's dimer at a fragment pair's geometry: an open box, the field on with the pin
/// charge, forces computed once so the closure assignment and the rows are read.
fn engine_dimer(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, plant: SeamPlant) -> Box<Sim> {
    let mut species = a.species.clone();
    species.extend_from_slice(&b.species);
    let mut pos: Vec<[f64; 3]> = a.centers.iter().chain(b.centers.iter()).map(|c| [c[0] + 15.0, c[1] + 15.0, c[2] + 10.0]).collect();
    // keep the separated copy inside the box too
    for p in pos.iter_mut() {
        p[0] = p[0].max(0.5);
    }
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
    s.set_seam(seam).unwrap();
    s.refresh_pairs();
    s.compute_forces();
    s
}

/// The engine's seam-law interaction: `E(geometry) − E(acceptor moved 40 bohr along x)`, on
/// the rows the seam law serves between units (the field and the wall; the closures are
/// dropped across the seam by G-B4).
fn engine_interaction(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, plant: SeamPlant) -> (f64, f64, f64) {
    let s = engine_dimer(a, b, seam, plant);
    let near = (s.e_pair + s.e_three) + s.e_field + s.e_seam;
    let far_b = b.translated([40.0, 0.0, 0.0]);
    let f = engine_dimer(a, &far_b, seam, plant);
    let far = (f.e_pair + f.e_three) + f.e_field + f.e_seam;
    (near - far, s.e_field - f.e_field, s.e_seam - f.e_seam)
}

fn run_fit(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let mut nodes: Vec<Node> = Vec::new();
    let mut sorted = NODES_ANGSTROM.to_vec();
    sorted.sort_by(|x, y| x.partial_cmp(y).unwrap());
    for &r in &sorted {
        let path = out.join(format!("linear_R{r:.1}.json"));
        let t = fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} missing: run `solve` first", path.display()));
        let (a, b) = linear(o, h, r);
        let s = engine_dimer(&a, &b, None, SeamPlant::None);
        assert_eq!(s.seam_work.units, 2, "the closure assignment finds two units at R_OO = {r} Å");
        let e_field = s.e_field;
        let de = json_num(&t, "de_exact");
        nodes.push(Node {
            r_ang: r,
            r_bohr: json_num(&t, "r_oo_bohr"),
            de_exact: de,
            cpu: json_num(&t, "cpu_seconds"),
            converged: json_str(&t, "exit") == "Converged" && json_num(&t, "residual") <= RESIDUAL_BAR,
            exit: json_str(&t, "exit"),
            iters: json_num(&t, "davidson_iters") as u64,
            residual: json_num(&t, "residual"),
            e_field,
            r: de - e_field,
        });
    }
    // G-C0 (AMENDMENT 1): the price node in core-seconds
    let price = nodes.iter().find(|n| (n.r_ang - 2.9).abs() < 1e-9).unwrap();
    let g_c0 = price.cpu >= PRICE_FLOOR_CORE_S && price.cpu <= PRICE_CEILING_CORE_S && price.converged;
    eprintln!("G-C0 (A1): 2.9 Å node {:.0} core-s (floor {PRICE_FLOOR_CORE_S:.0}, ceiling {PRICE_CEILING_CORE_S:.0}), exit {}, residual {:.1e} → {}", price.cpu, price.exit, price.residual, if g_c0 { "ADMITTED" } else { "REFUSED" });
    let all_converged = nodes.iter().all(|n| n.converged);
    eprintln!("| R (Å) | R (bohr) | ΔE_exact (Ha) | E_field (Ha) | r = ΔE − E_field (Ha) | iters | exit | core-s |");
    for n in &nodes {
        eprintln!("| {:.1} | {:.3} | {:+.6e} | {:+.6e} | {:+.6e} | {} | {} | {:.0} |", n.r_ang, n.r_bohr, n.de_exact, n.e_field, n.r, n.iters, n.exit, n.cpu);
    }
    // S1: the largest contiguous set of the shortest nodes, all positive above the floor,
    // the weighted log-linear fit over them within FIT_TOL·|ΔE_exact| at each
    let positive: Vec<bool> = nodes.iter().map(|n| n.r > R_FLOOR).collect();
    let n_pos_prefix = positive.iter().take_while(|&&p| p).count();
    let mut best: Option<(usize, f64, f64)> = None; // (k, ln A, b)
    for k in (3..=n_pos_prefix).rev() {
        let sub = &nodes[..k];
        let w: Vec<f64> = sub.iter().map(|n| 1.0 / (n.de_exact * n.de_exact)).collect();
        let sw: f64 = w.iter().sum();
        let mx = sub.iter().zip(&w).map(|(n, w)| w * n.r_bohr).sum::<f64>() / sw;
        let my = sub.iter().zip(&w).map(|(n, w)| w * n.r.ln()).sum::<f64>() / sw;
        let sxx: f64 = sub.iter().zip(&w).map(|(n, w)| w * (n.r_bohr - mx) * (n.r_bohr - mx)).sum();
        let sxy: f64 = sub.iter().zip(&w).map(|(n, w)| w * (n.r_bohr - mx) * (n.r.ln() - my)).sum();
        let slope = sxy / sxx; // ln r = ln A − b R
        let b = -slope;
        let ln_a = my - slope * mx;
        let within = sub.iter().all(|n| ((ln_a - b * n.r_bohr).exp() - n.r).abs() <= FIT_TOL * n.de_exact.abs());
        if within {
            best = Some((k, ln_a, b));
            break;
        }
    }
    let branch = match best {
        Some((6, _, _)) => "a",
        Some((_, _, _)) => "b",
        None => {
            if nodes[0].r <= 0.0 || n_pos_prefix < 3 {
                "c"
            } else {
                "none-by-letter"
            }
        }
    };
    let (k, a, b) = match best {
        Some((k, ln_a, b)) => (k, ln_a.exp(), b),
        None => (0, 0.0, 0.0),
    };
    let r_x = if k > 0 { nodes[k - 1].r_ang } else { f64::NAN };
    eprintln!("S1: positive prefix {n_pos_prefix} of 6; fit over the shortest {k} nodes: A = {a:.6e} Ha, b = {b:.6} /bohr (R_x = {r_x} Å) → branch ({branch})");
    // the remainder beyond R_x: log-log slope between consecutive nodes
    let mut remainder_lines = Vec::new();
    if k > 0 {
        for i in k..nodes.len() {
            let rem = nodes[i].r - a * (-b * nodes[i].r_bohr).exp();
            let prev = nodes[i - 1].r - a * (-b * nodes[i - 1].r_bohr).exp();
            let slope = if rem != 0.0 && prev != 0.0 { (rem.abs() / prev.abs()).ln() / (nodes[i].r_bohr / nodes[i - 1].r_bohr).ln() } else { f64::NAN };
            remainder_lines.push(format!("{{\"r_angstrom\": {:.1}, \"remainder\": {:+.6e}, \"remainder_over_de\": {:+.4e}, \"loglog_slope_from_previous\": {:.3}}}", nodes[i].r_ang, rem, rem / nodes[i].de_exact, slope));
            eprintln!("  beyond R_x: {:.1} Å remainder {:+.4e} Ha ({:+.3} of ΔE), log-log slope {:.2}", nodes[i].r_ang, rem, rem / nodes[i].de_exact, slope);
        }
    }
    // G-C1 and plant (i): the engine's seam-law interaction at every node
    let model = SeamModel { a, b };
    let mut g_c1_worst = 0.0f64;
    let mut plant_i = (f64::NAN, f64::NAN, false);
    let mut c1_lines = Vec::new();
    if k > 0 {
        for n in &nodes {
            let (a_f, b_f) = linear(o, h, n.r_ang);
            let (e_int, e_f, e_w) = engine_interaction(&a_f, &b_f, Some(model), SeamPlant::None);
            let want = n.e_field + model.wall(n.r_bohr);
            let miss = (e_int - want).abs();
            g_c1_worst = g_c1_worst.max(miss);
            c1_lines.push(format!("{{\"r_angstrom\": {:.1}, \"engine_interaction\": {:+.12e}, \"field_plus_wall\": {:+.12e}, \"miss\": {:.3e}, \"engine_field\": {:+.12e}, \"engine_wall\": {:+.12e}}}", n.r_ang, e_int, want, miss, e_f, e_w));
            if (n.r_ang - 2.5).abs() < 1e-9 {
                let (e_pl, _, _) = engine_interaction(&a_f, &b_f, Some(model), SeamPlant::FlipSign);
                let expected = 2.0 * model.wall(n.r_bohr);
                let observed = (e_pl - want).abs();
                let carrier = model.wall(n.r_bohr) >= 1e-3;
                plant_i = (observed, expected, carrier && (observed - expected).abs() <= G_C1_TOL);
                eprintln!("plant (i) at 2.5 Å: miss {observed:.6e} vs 2·wall {expected:.6e}; carrier wall {:.3e} ≥ 1e-3: {carrier} → {}", model.wall(n.r_bohr), if plant_i.2 { "FIRES" } else { "does not fire" });
            }
        }
        eprintln!("G-C1: worst |engine − (field + wall)| = {g_c1_worst:.3e} (stake {G_C1_TOL:.0e}) → {}", if g_c1_worst <= G_C1_TOL { "PASS" } else { "FAIL" });
    }
    // the prediction for the flipped node, filed BEFORE its solve
    let pred = if k > 0 {
        let (a_f, b_f) = flipped(o, h, FLIPPED_ANGSTROM);
        let s = engine_dimer(&a_f, &b_f, Some(model), SeamPlant::None);
        let (e_int, e_f, e_w) = engine_interaction(&a_f, &b_f, Some(model), SeamPlant::None);
        let text = format!("{{\n  \"node\": \"flipped_R{FLIPPED_ANGSTROM:.1}\", \"r_oo_angstrom\": {FLIPPED_ANGSTROM:.3}, \"r_oo_bohr\": {:.6},\n  \"e_pred\": {e_int:+.12e}, \"engine_field\": {e_f:+.12e}, \"engine_wall\": {e_w:+.12e}, \"units\": {},\n  \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\", \"wall\": {{\"a\": {a:.12e}, \"b\": {b:.12e}}},\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n", FLIPPED_ANGSTROM * ANGSTROM_TO_BOHR, s.seam_work.units, centers_json(&a_f), centers_json(&b_f));
        fs::write(out.join("prediction.json"), text).unwrap();
        eprintln!("prediction.json filed: flipped dimer at {FLIPPED_ANGSTROM} Å → E_pred {e_int:+.6e} Ha (field {e_f:+.6e}, wall {e_w:+.6e})");
        Some(e_int)
    } else {
        None
    };
    let node_lines: Vec<String> = nodes.iter().map(|n| format!("{{\"r_angstrom\": {:.1}, \"r_bohr\": {:.6}, \"de_exact\": {:+.12e}, \"e_field\": {:+.12e}, \"r\": {:+.12e}, \"fit\": {:+.12e}, \"miss_over_de\": {:.4e}, \"in_fit\": {}, \"davidson_iters\": {}, \"exit\": \"{}\", \"residual\": {:.3e}, \"cpu_seconds\": {:.1}}}",
        n.r_ang, n.r_bohr, n.de_exact, n.e_field, n.r, if k > 0 { a * (-b * n.r_bohr).exp() } else { f64::NAN }, if k > 0 { (a * (-b * n.r_bohr).exp() - n.r).abs() / n.de_exact.abs() } else { f64::NAN }, nodes.iter().position(|m| std::ptr::eq(m, n)).unwrap() < k, n.iters, n.exit, n.residual, n.cpu)).collect();
    fs::write(out.join("wall.json"), format!("{{\n  \"a\": {a:.12e}, \"b\": {b:.12e}, \"fit_nodes\": {k}, \"r_x_angstrom\": {r_x}, \"s1_branch\": \"{branch}\",\n  \"g_c0_admitted_amendment_1\": {g_c0}, \"all_converged\": {all_converged}, \"g_c1_worst_miss\": {g_c1_worst:.3e}, \"g_c1_pass\": {},\n  \"plant_i\": {{\"miss_observed\": {:.6e}, \"miss_expected\": {:.6e}, \"fires\": {}}},\n  \"e_pred_flipped\": {},\n  \"nodes\": [\n    {}\n  ],\n  \"remainder_beyond_r_x\": [{}]\n}}\n",
        g_c1_worst <= G_C1_TOL, plant_i.0, plant_i.1, plant_i.2, pred.map_or("null".to_string(), |p| format!("{p:+.12e}")), node_lines.join(",\n    "), remainder_lines.join(", "))).unwrap();
    eprintln!("wall.json written");
}

// -------------------------------------------------------------------------- predict

fn run_predict(out: &Path) {
    let pred_path = out.join("prediction.json");
    let pred = fs::read_to_string(&pred_path).unwrap_or_else(|_| panic!("{} missing: the prediction is filed BEFORE the solve (run `fit` first)", pred_path.display()));
    let e_pred = json_num(&pred, "e_pred");
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let (a, b) = flipped(o, h, FLIPPED_ANGSTROM);
    let name = format!("flipped_R{FLIPPED_ANGSTROM:.1}");
    let ok = solve_node(out, &name, FLIPPED_ANGSTROM, &a, &b, false);
    let t = fs::read_to_string(out.join(format!("{name}.json"))).unwrap();
    let de = json_num(&t, "de_exact");
    let tol = (PRED_FRAC * de.abs()).max(PRED_ABS);
    let miss = (e_pred - de).abs();
    let s2 = if ok && miss <= tol { "a" } else { "b" };
    fs::write(out.join("prediction_check.json"), format!("{{\"e_pred\": {e_pred:+.12e}, \"de_exact\": {de:+.12e}, \"miss\": {miss:.6e}, \"miss_fraction\": {:.4}, \"tolerance\": {tol:.6e}, \"converged\": {ok}, \"s2_branch\": \"{s2}\"}}\n", miss / de.abs())).unwrap();
    eprintln!("S2: E_pred {e_pred:+.6e} vs ΔE_exact {de:+.6e} — miss {miss:.3e} ({:.1} %) against {tol:.3e} → branch ({s2})", 100.0 * miss / de.abs());
    fs::write(out.join("predict.done"), "done\n").unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let what = args.get(1).map(String::as_str).unwrap_or("solve");
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/field3".to_string()));
    fs::create_dir_all(&out).expect("out");
    match what {
        "solve" => run_solve(&out),
        "fit" => run_fit(&out),
        "predict" => run_predict(&out),
        other => eprintln!("unknown phase {other}"),
    }
}
