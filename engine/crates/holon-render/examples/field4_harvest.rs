//! FIELD-4's harvest (`conformance/water_observatory/FIELD4_PREREG.md` §0, §2, §6): the
//! DENSITY field on FIELD-3's six linear nodes, the penetration-and-induction residual over
//! the engine's point charges, the wall over the density field, the dispersion beyond it,
//! and the flipped node's prediction filed BEFORE that node is solved.
//!
//! ```text
//! cargo run --release -p holon-render --example field4_harvest -- density [OUT_DIR]
//! cargo run --release -p holon-render --example field4_harvest -- predict [OUT_DIR]
//! ```
//!
//! `density`: for each linear node, EMBED-3's frozen-density fixed point from
//! `DensityStart::Zero` and its `ΔE_ρ = E_A[ρ_B] + E_B[ρ_A] − E_es − E_A0 − E_B0`; the
//! engine's point-charge field `E_q` read from FIELD-3's `wall.json`; `p = ΔE_ρ − E_q` and
//! `r_ρ = ΔE_exact − ΔE_ρ`. Then C1, then the three fits in the ledger's order — `(P, c)`
//! from `p(R)` on the cross-unit H–O contacts, `(A, b)` from `r_ρ(R)` by FIELD-3's rule,
//! `C₆` from what is left beyond `R_x` — into `wall4.json`, and `prediction.json` for the
//! FLIPPED node with BOTH placements. `predict`: refuses without `prediction.json`, then
//! solves the flipped node exactly and writes `prediction_check.json` (S2).
use holon_chem::density_embed::{classical_interaction, embed_densities, solve_in_densities, DensityStart, Partner};
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::{solve_embedded, supermolecule, water_dimer_linear, Fragment, ANGSTROM_TO_BOHR};
use holon_chem::fci::SolveExit;
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/quartet.rs"]
#[allow(dead_code)]
mod quartet;

/// EMBED-1's water pins — the same numbers FIELD-3's runner carries.
const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;
/// The six linear nodes (Å), SHORTEST FIRST (the harvest reads prefixes).
const NODES_ANGSTROM: [f64; 6] = [2.5, 2.7, 2.9, 3.1, 3.4, 3.7];
const FLIPPED_ANGSTROM: f64 = 3.4;
/// The residual bar every exact node must meet (EMBED-3's).
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
/// The band the remainder's log-log slope must lie in for `C₆` to transfer (S1 (b)).
const SLOPE_LO: f64 = -8.0;
const SLOPE_HI: f64 = -4.0;
/// The monomer references are re-solved and checked against FIELD-3's record at this bar.
const MONOMER_BAR: f64 = 1e-9;

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

fn linear(o: Species, h: Species, r_oo_angstrom: f64) -> (Fragment, Fragment) {
    water_dimer_linear(o, h, H2O_R, H2O_THETA, r_oo_angstrom * ANGSTROM_TO_BOHR)
}

/// The FLIPPED dimer, FIELD-3's geometry verbatim: the linear donor, and the acceptor
/// rotated by π about the x-axis through its oxygen — its hydrogens toward the donor.
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

/// The engine's dimer, FIELD-3's `engine_dimer` with the seam OFF: an open box, the field on
/// with the pin charge, forces computed once so the closure assignment and the rows are read.
fn engine_dimer(a: &Fragment, b: &Fragment) -> Box<Sim> {
    let mut species = a.species.clone();
    species.extend_from_slice(&b.species);
    let mut pos: Vec<[f64; 3]> = a.centers.iter().chain(b.centers.iter()).map(|c| [c[0] + 15.0, c[1] + 15.0, c[2] + 10.0]).collect();
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
    s.set_seam(None).unwrap();
    s.refresh_pairs();
    s.compute_forces();
    s
}

/// `E(geometry) − E(acceptor moved 40 bohr along x)` on the rows the seam law serves between
/// units: the total, the FIELD part, the SEAM part. With the seam off the third is zero.
fn engine_interaction(a: &Fragment, b: &Fragment) -> (f64, f64, f64) {
    let s = engine_dimer(a, b);
    let near = (s.e_pair + s.e_three) + s.e_field + s.e_seam;
    let far_b = b.translated([40.0, 0.0, 0.0]);
    let f = engine_dimer(a, &far_b);
    let far = (f.e_pair + f.e_three) + f.e_field + f.e_seam;
    (near - far, s.e_field - f.e_field, s.e_seam - f.e_seam)
}

/// One exact node: the supermolecule, the monomer references, the record. FIELD-3's
/// `solve_node` verbatim, so the flipped node's record is the same object FIELD-3 wrote.
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
        "  {name}: R_OO {r_oo_angstrom:.1} Å, {} dets, ΔE_exact {de:+.6e} Ha, {} iters, residual {:.1e}, exit {}, wall {wall:.0} s, {cpu:.0} core-s",
        sm.gp.space.n_det,
        sm.sol.davidson_iters,
        sm.sol.residual,
        exit_name(&sm.sol.exit),
    );
    converged
}

// ------------------------------------------------------------------------- the density phase

struct DNode {
    r_ang: f64,
    r_bohr: f64,
    de_exact: f64,
    e_q: f64,
    de_rho: f64,
    p: f64,
    r_rho: f64,
    ho: Vec<f64>,
    converged: bool,
    sweeps: usize,
}

/// Whether the winning exponent sits on the declared grid's boundary — a fit pinned at the
/// edge is a fit the grid did not contain, and the record says so rather than reading it.
fn at_edge(c: f64) -> bool {
    (c - C_MIN).abs() < 0.5 * C_STEP || (c - (C_MIN + C_STEP * C_STEPS as f64)).abs() < 0.5 * C_STEP
}

/// `S_c` at a node: the engine's own sum over cross-unit H–O pairs, or the O–O placement.
fn s_of(n: &DNode, c: f64, on_ho: bool) -> f64 {
    if on_ho {
        n.ho.iter().map(|d| (-c * d).exp()).sum()
    } else {
        (-c * n.r_bohr).exp()
    }
}

/// `(P, c)` by the declared grid: for each `c`, the weighted least-squares `P` of
/// `p ≈ −P·S_c`, weights `1/ΔE_exact²`; the `c` of least weighted residual.
fn fit_pen(used: &[&DNode], on_ho: bool) -> (f64, f64, f64) {
    let mut best = (0.0f64, f64::NAN, f64::INFINITY);
    for i in 0..=C_STEPS {
        let c = C_MIN + C_STEP * i as f64;
        let mut num = 0.0;
        let mut den = 0.0;
        for n in used {
            let w = 1.0 / (n.de_exact * n.de_exact);
            let s = s_of(n, c, on_ho);
            num += w * n.p * s;
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
                let e = n.p + p * s_of(n, c, on_ho);
                w * e * e
            })
            .sum();
        if resid < best.2 {
            best = (p, c, resid);
        }
    }
    best
}

fn run_density(out: &Path) {
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let f3 = field3_dir(out);
    let wall3 = fs::read_to_string(f3.join("wall.json")).unwrap_or_else(|_| panic!("{}/wall.json missing: FIELD-3's engine field is the E_q of record", f3.display()));
    eprintln!("FIELD-4 density — the frozen-density field on FIELD-3's six linear nodes, {} threads", threads());

    let mut nodes: Vec<DNode> = Vec::new();
    for &r in NODES_ANGSTROM.iter() {
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let node_path = f3.join(format!("linear_R{r:.1}.json"));
        let t3 = fs::read_to_string(&node_path).unwrap_or_else(|_| panic!("{} missing", node_path.display()));
        let de_exact = json_num(&t3, "de_exact");
        let e_a0_json = json_num(&t3, "e_a0");
        let e_b0_json = json_num(&t3, "e_b0");
        // E_q: the engine's point-charge field on this geometry, FIELD-3's wall.json, matched
        // by "r_angstrom" inside the node list.
        let e_q = e_field_from_wall3(&wall3, r);
        assert!(e_q.is_finite(), "FIELD-3's wall.json carries no e_field at R = {r} Å");

        let (a, b) = linear(o, h, r);
        // the monomer references, re-solved and checked against FIELD-3's record
        let e_a0 = solve_embedded(&a.species, &a.centers, &[]).e_total;
        let e_b0 = solve_embedded(&b.species, &b.centers, &[]).e_total;
        assert!((e_a0 - e_a0_json).abs() <= MONOMER_BAR, "E_A0 {e_a0:.12e} vs FIELD-3 {e_a0_json:.12e}");
        assert!((e_b0 - e_b0_json).abs() <= MONOMER_BAR, "E_B0 {e_b0:.12e} vs FIELD-3 {e_b0_json:.12e}");

        // EMBED-3's density field, verbatim
        let frags = vec![a.clone(), b.clone()];
        let z = embed_densities(&frags, DensityStart::Zero);
        let ea = solve_in_densities(&frags[0], &[Partner::new(&frags[1], &z.densities[1])]).e_total;
        let eb = solve_in_densities(&frags[1], &[Partner::new(&frags[0], &z.densities[0])]).e_total;
        let es_ab = classical_interaction(&frags[0], &z.densities[0], &frags[1], &z.densities[1]);
        let de_rho = ea + eb - es_ab - e_a0 - e_b0;
        let p = de_rho - e_q;
        let r_rho = de_exact - de_rho;
        let ho = cross_ho(&a, &b);
        let r_bohr = cross_oo(&a, &b);
        let wall_s = t0.elapsed().as_secs_f64();
        let cpu_s = cpu_seconds() - c0;

        fs::write(
            out.join(format!("density_R{r:.1}.json")),
            format!(
                "{{\n  \"node\": \"density_R{r:.1}\", \"r_oo_angstrom\": {r:.3}, \"r_oo_bohr\": {r_bohr:.6},\n  \"de_exact\": {de_exact:+.12e}, \"e_q\": {e_q:+.12e}, \"de_rho\": {de_rho:+.12e},\n  \"p\": {p:+.12e}, \"r_rho\": {r_rho:+.12e},\n  \"densities\": {{\"start\": \"Zero\", \"converged\": {}, \"sweeps\": {}, \"last_delta\": {:.3e}, \"e_a_in_b\": {ea:.12e}, \"e_b_in_a\": {eb:.12e}, \"e_es\": {es_ab:.12e}}},\n  \"monomers\": {{\"e_a0\": {e_a0:.12e}, \"e_b0\": {e_b0:.12e}, \"e_a0_field3\": {e_a0_json:.12e}, \"e_b0_field3\": {e_b0_json:.12e}, \"max_monomer_delta\": {:.3e}}},\n  \"cross_ho_bohr\": [{}],\n  \"wall_seconds\": {wall_s:.1}, \"cpu_seconds\": {cpu_s:.1}, \"threads\": {}\n}}\n",
                z.converged,
                z.sweeps,
                z.last_delta,
                (e_a0 - e_a0_json).abs().max((e_b0 - e_b0_json).abs()),
                ho.iter().map(|d| format!("{d:.6}")).collect::<Vec<_>>().join(", "),
                threads(),
            ),
        )
        .unwrap();
        eprintln!("  R_OO {r:.1} Å: ΔE_ρ {de_rho:+.6e}  E_q {e_q:+.6e}  p {p:+.6e}  r_ρ {r_rho:+.6e}  converged {} in {} sweeps  ({wall_s:.0} s)", z.converged, z.sweeps);
        nodes.push(DNode { r_ang: r, r_bohr, de_exact, e_q, de_rho, p, r_rho, ho, converged: z.converged, sweeps: z.sweeps });
    }

    // ------------------------------------------------------------------------------ C1
    eprintln!("\n| R (Å) | R_OO (bohr) | ΔE_exact (Ha) | E_q (Ha) | ΔE_ρ (Ha) | p (Ha) | r_ρ (Ha) | converged | sweeps |");
    for n in &nodes {
        eprintln!("| {:.1} | {:.4} | {:+.6e} | {:+.6e} | {:+.6e} | {:+.6e} | {:+.6e} | {} | {} |", n.r_ang, n.r_bohr, n.de_exact, n.e_q, n.de_rho, n.p, n.r_rho, n.converged, n.sweeps);
    }
    let readings: Vec<bool> = nodes.iter().map(|n| n.p.abs() >= R_FLOOR).collect();
    let sign_ok = nodes.iter().zip(&readings).all(|(n, &rd)| !rd || n.p <= 0.0);
    let peak = (0..nodes.len()).max_by(|&i, &j| nodes[i].p.abs().partial_cmp(&nodes[j].p.abs()).unwrap()).unwrap();
    let monotone_ok = (peak + 1..nodes.len()).all(|i| nodes[i].p.abs() <= nodes[i - 1].p.abs());
    let converged_ok = nodes.iter().all(|n| n.converged);
    let c1 = sign_ok && monotone_ok && converged_ok;
    eprintln!(
        "C1: p ≤ 0 at every reading ({sign_ok}); |p| non-increasing outward from its largest node, R = {:.1} Å ({monotone_ok}); every fixed point converged ({converged_ok}) → {}",
        nodes[peak].r_ang,
        if c1 { "PASS" } else { "FAIL" }
    );
    let excluded: Vec<String> = nodes.iter().zip(&readings).filter(|(_, &rd)| !rd).map(|(n, _)| format!("{:.1} Å", n.r_ang)).collect();
    if !excluded.is_empty() {
        eprintln!("C1: nodes under the {R_FLOOR:e} floor and NOT readings: {}", excluded.join(", "));
    }

    // ------------------------------------------------- fit (a): the penetration term (P, c)
    let used: Vec<&DNode> = nodes.iter().zip(&readings).filter(|(_, &rd)| rd).map(|(n, _)| n).collect();
    let (p_coef, c_coef, resid_ho) = fit_pen(&used, true);
    eprintln!(
        "\nfit (a) H–O placement: P = {p_coef:.9e} Ha, c = {c_coef:.2} /bohr over {} of 6 nodes (weighted residual {resid_ho:.6e})",
        used.len()
    );
    for n in &nodes {
        let f = -p_coef * s_of(n, c_coef, true);
        eprintln!("  {:.1} Å: p {:+.6e}  fit {:+.6e}  miss {:+.6e} ({:+.3} of ΔE_exact){}", n.r_ang, n.p, f, f - n.p, (f - n.p) / n.de_exact.abs(), if n.p.abs() >= R_FLOOR { "" } else { "  [excluded, under the floor]" });
    }
    // the named alternative, fit here so `prediction.json` can carry both placements
    let (p_oo, c_oo, resid_oo) = fit_pen(&used, false);
    eprintln!("fit (a′) O–O placement (the named alternative): P = {p_oo:.9e} Ha, c = {c_oo:.2} /bohr (weighted residual {resid_oo:.6e})");

    // ------------------------------------------------------- fit (b): the wall (A, b) on r_ρ
    let positive: Vec<bool> = nodes.iter().map(|n| n.r_rho > R_FLOOR).collect();
    let n_pos_prefix = positive.iter().take_while(|&&p| p).count();
    let mut best: Option<(usize, f64, f64)> = None;
    let mut attempts: Vec<String> = Vec::new();
    for k in (3..=n_pos_prefix).rev() {
        let sub = &nodes[..k];
        let w: Vec<f64> = sub.iter().map(|n| 1.0 / (n.de_exact * n.de_exact)).collect();
        let sw: f64 = w.iter().sum();
        let mx = sub.iter().zip(&w).map(|(n, w)| w * n.r_bohr).sum::<f64>() / sw;
        let my = sub.iter().zip(&w).map(|(n, w)| w * n.r_rho.ln()).sum::<f64>() / sw;
        let sxx: f64 = sub.iter().zip(&w).map(|(n, w)| w * (n.r_bohr - mx) * (n.r_bohr - mx)).sum();
        let sxy: f64 = sub.iter().zip(&w).map(|(n, w)| w * (n.r_bohr - mx) * (n.r_rho.ln() - my)).sum();
        let slope = sxy / sxx; // ln r_ρ = ln A − b R
        let b = -slope;
        let ln_a = my - slope * mx;
        let within = sub.iter().all(|n| ((ln_a - b * n.r_bohr).exp() - n.r_rho).abs() <= FIT_TOL * n.de_exact.abs());
        // every attempt is recorded, so a failure says WHY and not only that it failed
        let worst = sub.iter().map(|n| ((ln_a - b * n.r_bohr).exp() - n.r_rho).abs() / n.de_exact.abs()).fold(0.0, f64::max);
        attempts.push(format!("{{\"k\": {k}, \"r_x_angstrom\": {:.1}, \"a\": {:.12e}, \"b\": {b:.12e}, \"worst_miss_over_de\": {worst:.6}, \"qualifies\": {within}}}", nodes[k - 1].r_ang, ln_a.exp()));
        eprintln!("  attempt k = {k} (to {:.1} Å): A = {:.6e} Ha, b = {b:.6} /bohr, worst miss {worst:.4} of |ΔE_exact| (tol {FIT_TOL}) → {}", nodes[k - 1].r_ang, ln_a.exp(), if within { "QUALIFIES" } else { "does not qualify" });
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
    eprintln!("\nfit (b) the wall over the density field: positive prefix {n_pos_prefix} of 6; A = {a_coef:.9e} Ha, b = {b_coef:.6} /bohr over the shortest {k} nodes (R_x = {r_x} Å)");
    if k > 0 {
        for n in &nodes[..k] {
            let f = a_coef * (-b_coef * n.r_bohr).exp();
            eprintln!("  {:.1} Å: r_ρ {:+.6e}  wall {:+.6e}  miss {:+.6e} ({:.4} of ΔE_exact, tol {FIT_TOL})", n.r_ang, n.r_rho, f, f - n.r_rho, (f - n.r_rho).abs() / n.de_exact.abs());
        }
    }

    // ----------------------------------------- fit (c): C6 from the remainder beyond R_x
    let rem = |n: &DNode| if k > 0 { n.r_rho - a_coef * (-b_coef * n.r_bohr).exp() } else { f64::NAN };
    let mut slopes: Vec<(f64, f64, bool)> = Vec::new(); // (R, slope, both endpoints beyond R_x)
    let mut c6 = 0.0f64;
    let mut c6_transferred = false;
    if k > 0 && k < nodes.len() {
        let (mut num, mut den) = (0.0, 0.0);
        for n in &nodes[k..] {
            let w = 1.0 / (n.de_exact * n.de_exact);
            let x = -1.0 / n.r_bohr.powi(6);
            num += w * rem(n) * x;
            den += w * x * x;
        }
        c6 = if den > 0.0 { num / den } else { 0.0 };
        for i in k..nodes.len() {
            let (a1, a0) = (rem(&nodes[i]), rem(&nodes[i - 1]));
            let s = if a1 != 0.0 && a0 != 0.0 { (a1.abs() / a0.abs()).ln() / (nodes[i].r_bohr / nodes[i - 1].r_bohr).ln() } else { f64::NAN };
            slopes.push((nodes[i].r_ang, s, i - 1 >= k));
            eprintln!("  beyond R_x: {:.1} Å remainder {:+.6e} Ha ({:+.4} of ΔE_exact), log-log slope from {:.1} Å {:.3}{}", nodes[i].r_ang, a1, a1 / nodes[i].de_exact, nodes[i - 1].r_ang, s, if i - 1 >= k { "" } else { "  [from the last FIT node]" });
        }
        // the gate: the slopes between two beyond-R_x nodes if any exist, else the single
        // slope carried from the last fit node — which set was used is recorded.
        let pure: Vec<f64> = slopes.iter().filter(|s| s.2).map(|s| s.1).collect();
        let gate: Vec<f64> = if pure.is_empty() { slopes.iter().map(|s| s.1).collect() } else { pure };
        c6_transferred = !gate.is_empty() && gate.iter().all(|s| s.is_finite() && *s >= SLOPE_LO && *s <= SLOPE_HI);
        eprintln!("fit (c) dispersion: C₆ = {c6:.9e} Ha·bohr⁶ from {} node(s) beyond R_x; gate slopes {:?} in [{SLOPE_LO}, {SLOPE_HI}] → {}", nodes.len() - k, gate.iter().map(|s| format!("{s:.3}")).collect::<Vec<_>>(), if c6_transferred { "TRANSFERRED" } else { "NOT transferred (C₆ = 0 recorded)" });
        if !c6_transferred {
            c6 = 0.0;
        }
    } else if k > 0 {
        eprintln!("fit (c) dispersion: no node beyond R_x; C₆ = 0 recorded, nothing to transfer");
    }

    let branch = if k == nodes.len() {
        "a"
    } else if k >= 3 {
        "b"
    } else {
        "c"
    };
    // (c) is reached two ways and the record keeps them apart: the prereg's letter names the
    // short positive prefix, but a prefix of three or more that no line fits within the
    // tolerance lands here too (FIELD-3's runner called that case "none-by-letter").
    let branch_c_reason = if branch != "c" {
        "n/a".to_string()
    } else if n_pos_prefix < 3 {
        format!("the positive prefix is {n_pos_prefix}, under three — the prereg's own (c)")
    } else {
        format!("the positive prefix is {n_pos_prefix}, but no set of three or more is fitted within {FIT_TOL}·|ΔE_exact| — (c) by the harvest rule, NOT by the prefix count")
    };
    eprintln!(
        "\nS1: branch ({branch}) — {}",
        match branch {
            "a" => "the wall fits all six nodes; exchange is the whole remainder at this level".to_string(),
            "b" => format!("a positive prefix of {k} fits; the wall from the prefix, C₆ {}", if c6_transferred { "from the remainder, TRANSFERRED" } else { "NOT transferred" }),
            _ => format!("VOID: no wall, the arms do not run. {branch_c_reason}"),
        }
    );

    // G-C1 and plant (i): evaluated by the lead's phase once SeamModel carries the new terms
    eprintln!("G-C1: deferred to the engine phase");

    // The engine's field convention, checked: FIELD-3's stored `e_field` is the RAW near-box
    // field; the flipped prediction reads the DIFFERENCE `E(near) − E(40 bohr apart)`. If
    // those disagree the two numbers are not the same quantity, so the gap is measured.
    let mut e_q_diff: Vec<f64> = Vec::new();
    for n in &nodes {
        let (a_f, b_f) = linear(o, h, n.r_ang);
        let (_, e_diff, _) = engine_interaction(&a_f, &b_f);
        e_q_diff.push(e_diff - n.e_q);
    }
    let e_q_conv = e_q_diff.iter().fold(0.0f64, |m, d| m.max(d.abs()));
    eprintln!("E_q convention: worst |E(near) − E(far) − wall.json's e_field| over the six nodes = {e_q_conv:.3e} Ha (the fit reads the RAW field of record, the flipped prediction reads the DIFFERENCE)");

    // ------------------------------------------------------------------------- wall4.json
    let node_lines: Vec<String> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            format!(
                "{{\"r_angstrom\": {:.1}, \"r_bohr\": {:.6}, \"de_exact\": {:+.12e}, \"e_q\": {:+.12e}, \"de_rho\": {:+.12e}, \"p\": {:+.12e}, \"p_fit\": {:+.12e}, \"r_rho\": {:+.12e}, \"wall_fit\": {:+.12e}, \"remainder\": {:+.12e}, \"in_wall_fit\": {}, \"in_pen_fit\": {}, \"converged\": {}, \"sweeps\": {}, \"e_q_difference_minus_raw\": {:+.6e}, \"cross_ho_bohr\": [{}]}}",
                n.r_ang,
                n.r_bohr,
                n.de_exact,
                n.e_q,
                n.de_rho,
                n.p,
                -p_coef * s_of(n, c_coef, true),
                n.r_rho,
                if k > 0 { a_coef * (-b_coef * n.r_bohr).exp() } else { f64::NAN },
                rem(n),
                i < k,
                readings[i],
                n.converged,
                n.sweeps,
                e_q_diff[i],
                n.ho.iter().map(|d| format!("{d:.6}")).collect::<Vec<_>>().join(", ")
            )
        })
        .collect();
    let slope_lines: Vec<String> = slopes.iter().map(|(r, s, pure)| format!("{{\"r_angstrom\": {r:.1}, \"loglog_slope_from_previous\": {s:.6}, \"both_beyond_r_x\": {pure}}}")).collect();
    fs::write(
        out.join("wall4.json"),
        format!(
            "{{\n  \"a\": {a_coef:.12e}, \"b\": {b_coef:.12e}, \"p\": {p_coef:.12e}, \"c\": {c_coef:.12e}, \"c6\": {c6:.12e},\n  \"r_x_angstrom\": {r_x}, \"fit_nodes\": {k}, \"s1_branch\": \"{branch}\", \"c6_transferred\": {c6_transferred},\n  \"positive_prefix\": {n_pos_prefix}, \"branch_c_reason\": \"{branch_c_reason}\",\n  \"pen_fit\": {{\"placement\": \"cross-unit H–O\", \"nodes_used\": {}, \"weighted_residual\": {resid_ho:.12e}, \"c_grid\": \"{C_MIN} to {} step {C_STEP} per bohr\", \"c_at_grid_edge\": {}}},\n  \"pen_fit_oo\": {{\"placement\": \"cross-unit O–O (the named alternative)\", \"p\": {p_oo:.12e}, \"c\": {c_oo:.12e}, \"weighted_residual\": {resid_oo:.12e}, \"c_at_grid_edge\": {}}},\n  \"c1\": {{\"pass\": {c1}, \"sign_ok\": {sign_ok}, \"monotone_ok\": {monotone_ok}, \"converged_ok\": {converged_ok}, \"peak_r_angstrom\": {:.1}, \"floor\": {R_FLOOR:e}}},\n  \"e_q_convention_max_diff\": {e_q_conv:.6e},\n  \"g_c1\": \"deferred to the engine phase\", \"plant_i\": \"deferred to the engine phase\",\n  \"wall_fit_attempts\": [{}],\n  \"remainder_beyond_r_x\": [{}],\n  \"nodes\": [\n    {}\n  ]\n}}\n",
            used.len(),
            C_MIN + C_STEP * C_STEPS as f64,
            at_edge(c_coef),
            at_edge(c_oo),
            nodes[peak].r_ang,
            attempts.join(", "),
            slope_lines.join(", "),
            node_lines.join(",\n    ")
        ),
    )
    .unwrap();
    eprintln!("wall4.json written");

    // ------------------------------------------- prediction.json, BEFORE the flipped solve
    let (a_f, b_f) = flipped(o, h, FLIPPED_ANGSTROM);
    let (_, e_q_flip, _) = engine_interaction(&a_f, &b_f);
    let s_flip = engine_dimer(&a_f, &b_f);
    let ho_flip = cross_ho(&a_f, &b_f);
    let r_oo_flip = cross_oo(&a_f, &b_f);
    let s_ho_flip: f64 = ho_flip.iter().map(|d| (-c_coef * d).exp()).sum();
    let p_ho_flip = -p_coef * s_ho_flip;
    let p_oo_flip = -p_oo * (-c_oo * r_oo_flip).exp();
    let wall_flip = if k > 0 { a_coef * (-b_coef * r_oo_flip).exp() } else { 0.0 };
    let disp_flip = if c6 == 0.0 { 0.0 } else { -c6 / r_oo_flip.powi(6) };
    let e_pred_ho = e_q_flip + p_ho_flip + wall_flip + disp_flip;
    let e_pred_oo = e_q_flip + p_oo_flip + wall_flip + disp_flip;
    fs::write(
        out.join("prediction.json"),
        format!(
            "{{\n  \"node\": \"flipped_R{FLIPPED_ANGSTROM:.1}\", \"r_oo_angstrom\": {FLIPPED_ANGSTROM:.3}, \"r_oo_bohr\": {r_oo_flip:.6}, \"units\": {},\n  \"e_pred_ho\": {e_pred_ho:+.12e}, \"e_pred_oo\": {e_pred_oo:+.12e},\n  \"parts\": {{\"e_q\": {e_q_flip:+.12e}, \"p_ho\": {p_ho_flip:+.12e}, \"p_oo\": {p_oo_flip:+.12e}, \"wall\": {wall_flip:+.12e}, \"disp\": {disp_flip:+.12e}}},\n  \"coefficients\": {{\"p\": {p_coef:.12e}, \"c\": {c_coef:.12e}, \"p_oo_placement\": {p_oo:.12e}, \"c_oo_placement\": {c_oo:.12e}, \"a\": {a_coef:.12e}, \"b\": {b_coef:.12e}, \"c6\": {c6:.12e}}},\n  \"wall_harvested\": {}, \"c6_transferred\": {c6_transferred}, \"s1_branch\": \"{branch}\",\n  \"placement_gap\": {:.6e}, \"placement_gap_over_tolerance_floor\": {:.4}, \"placement_separable\": {},\n  \"cross_ho_bohr\": [{}], \"s_c_flipped\": {s_ho_flip:.12e},\n  \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\", \"tolerance_frac\": {PRED_FRAC}, \"tolerance_abs\": {PRED_ABS:e},\n  \"donor_centers\": [{}], \"acceptor_centers\": [{}]\n}}\n",
            s_flip.seam_work.units,
            k > 0,
            (e_pred_ho - e_pred_oo).abs(),
            (e_pred_ho - e_pred_oo).abs() / PRED_ABS,
            (e_pred_ho - e_pred_oo).abs() > PRED_ABS,
            ho_flip.iter().map(|d| format!("{d:.6}")).collect::<Vec<_>>().join(", "),
            centers_json(&a_f),
            centers_json(&b_f)
        ),
    )
    .unwrap();
    eprintln!(
        "prediction.json filed BEFORE the flipped solve: E_pred(H–O) {e_pred_ho:+.6e} Ha, E_pred(O–O) {e_pred_oo:+.6e} Ha — E_q {e_q_flip:+.6e}, p_HO {p_ho_flip:+.6e}, p_OO {p_oo_flip:+.6e}, wall {wall_flip:+.6e}{}, disp {disp_flip:+.6e}",
        if k > 0 { "" } else { " (NOT harvested; 0.0 recorded)" }
    );
    fs::write(out.join("density.done"), "done\n").unwrap();
}

/// FIELD-3's `wall.json` node list, matched by `r_angstrom`, its `e_field` returned.
fn e_field_from_wall3(wall3: &str, r: f64) -> f64 {
    for chunk in wall3.split("{\"r_angstrom\": ").skip(1) {
        let rr: f64 = chunk.split(',').next().and_then(|x| x.trim().parse().ok()).unwrap_or(f64::NAN);
        if (rr - r).abs() < 1e-9 {
            return json_num(chunk, "e_field");
        }
    }
    f64::NAN
}

fn field3_dir(out: &Path) -> PathBuf {
    let sib = out.parent().unwrap_or(Path::new(".")).join("field3");
    if sib.exists() {
        sib
    } else {
        PathBuf::from("../conformance/water_observatory/field3")
    }
}

// --------------------------------------------------------------------------- predict (S2)

fn run_predict(out: &Path) {
    let pred_path = out.join("prediction.json");
    let Ok(pred) = fs::read_to_string(&pred_path) else {
        eprintln!("{} missing: the prediction is filed BEFORE the solve (run `density` first). Nothing written.", pred_path.display());
        std::process::exit(2);
    };
    let e_pred_ho = json_num(&pred, "e_pred_ho");
    let e_pred_oo = json_num(&pred, "e_pred_oo");
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let (a, b) = flipped(o, h, FLIPPED_ANGSTROM);
    let name = format!("flipped_R{FLIPPED_ANGSTROM:.1}");
    eprintln!("FIELD-4 predict — the flipped node on {} threads", threads());
    let ok = solve_node(out, &name, FLIPPED_ANGSTROM, &a, &b, false);
    let t = fs::read_to_string(out.join(format!("{name}.json"))).unwrap();
    let de = json_num(&t, "de_exact");
    let tol = (PRED_FRAC * de.abs()).max(PRED_ABS);
    let miss_ho = (e_pred_ho - de).abs();
    let miss_oo = (e_pred_oo - de).abs();
    let s2 = if miss_ho <= tol {
        "a"
    } else if miss_oo <= tol {
        "b"
    } else {
        "c"
    };
    fs::write(
        out.join("prediction_check.json"),
        format!(
            "{{\n  \"node\": \"{name}\", \"e_pred_ho\": {e_pred_ho:+.12e}, \"e_pred_oo\": {e_pred_oo:+.12e}, \"de_exact\": {de:+.12e},\n  \"miss_ho\": {miss_ho:.6e}, \"miss_ho_fraction\": {:.6}, \"miss_oo\": {miss_oo:.6e}, \"miss_oo_fraction\": {:.6},\n  \"tolerance\": {tol:.6e}, \"tolerance_rule\": \"max({PRED_FRAC}·|ΔE_exact|, {PRED_ABS:e})\",\n  \"converged\": {ok}, \"exit\": \"{}\", \"davidson_iters\": {}, \"residual\": {:.3e}, \"cpu_seconds\": {:.1}, \"wall_seconds\": {:.1},\n  \"s2_branch\": \"{s2}\"\n}}\n",
            miss_ho / de.abs(),
            miss_oo / de.abs(),
            json_str(&t, "exit"),
            json_num(&t, "davidson_iters") as u64,
            json_num(&t, "residual"),
            json_num(&t, "cpu_seconds"),
            json_num(&t, "wall_seconds"),
        ),
    )
    .unwrap();
    eprintln!(
        "S2: ΔE_exact {de:+.6e} Ha — H–O miss {miss_ho:.3e} ({:.1} %), O–O miss {miss_oo:.3e} ({:.1} %), tolerance {tol:.3e} → branch ({s2})",
        100.0 * miss_ho / de.abs(),
        100.0 * miss_oo / de.abs()
    );
    fs::write(out.join("predict.done"), "done\n").unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let what = args.get(1).map(String::as_str).unwrap_or("density");
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/field4".to_string()));
    fs::create_dir_all(&out).expect("out");
    match what {
        "density" => run_density(&out),
        "predict" => run_predict(&out),
        other => eprintln!("unknown phase {other}"),
    }
}
