//! EMBED-1's runner (`conformance/water_observatory/EMBED_PREREG.md`).
//!
//! ```text
//! cargo run --release -p holon-chem --example embed_campaign -- hf|water|all [OUT_DIR]
//! ```
//!
//! Writes one JSON per node under `OUT_DIR` (default
//! `conformance/water_observatory/embed`), SKIPS a node whose JSON already exists (that is
//! the resume), and prints the S3 table at the end. `hf` runs G1's pins, System 1's ten
//! nodes and the control; `water` runs G1's water pin, G0's price on one node, and the five
//! water nodes only if G0 admits them. Every solve is `solve_determinant` on the host.
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::*;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

fn sp(s: &str) -> Species {
    by_symbol(s).expect("species")
}

/// Processor time (user + system) of this process in seconds, from /proc/self/stat at the
/// conventional 100 ticks per second; wall time is reported beside it, never instead.
fn cpu_seconds() -> f64 {
    let s = fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let tail = &s[s.rfind(')').map(|i| i + 2).unwrap_or(0)..];
    let f: Vec<&str> = tail.split_whitespace().collect();
    // after the ')' the fields start at index 0 = state (field 3); utime is field 14, stime 15
    let ut: f64 = f.get(11).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    let st: f64 = f.get(12).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    (ut + st) / 100.0
}

fn fmt_vec(v: &[f64]) -> String {
    format!("[{}]", v.iter().map(|x| format!("{x:.12e}")).collect::<Vec<_>>().join(", "))
}

struct Node {
    system: &'static str,
    r_ang: f64,
    sector: &'static str,
}

fn run_node(out: &PathBuf, node: &Node, a: &Fragment, b: &Fragment) {
    let path = out.join(format!("{}_R{:.1}.json", node.system, node.r_ang));
    if path.exists() {
        eprintln!("  {} R={:.1} Å: exists, skipped (resume)", node.system, node.r_ang);
        return;
    }
    let t0 = Instant::now();
    let c0 = cpu_seconds();
    let ea = solve_embedded(&a.species, &a.centers, &[]).e_total;
    let eb = solve_embedded(&b.species, &b.centers, &[]).e_total;
    let sm = supermolecule(a, b);
    let de_exact = sm.e_total - ea - eb;
    let mut rows = Vec::new();
    for (model, name) in [(ChargeModel::DipoleExact, "dipole_exact"), (ChargeModel::Mulliken, "mulliken")] {
        let z = embed_pair(a, b, model, Start::Zero, false);
        let i = embed_pair(a, b, model, Start::Isolated, false);
        let pl = embed_pair(a, b, model, Start::Zero, true);
        let dq = z.q_a.iter().zip(i.q_a.iter()).chain(z.q_b.iter().zip(i.q_b.iter())).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max);
        let de_emb = z.e_emb - ea - eb;
        let de_plant = pl.e_emb - ea - eb;
        let rho = (de_exact - de_emb).abs() / de_exact.abs();
        let rho_plant = (de_exact - de_plant).abs() / de_exact.abs();
        rows.push(format!(
            "    \"{name}\": {{\"converged\": {}, \"iterations_zero\": {}, \"iterations_isolated\": {}, \"g4_dq\": {:.3e}, \"g4_de\": {:.3e}, \"q_a\": {}, \"q_b\": {}, \"e_a\": {:.12e}, \"e_b\": {:.12e}, \"e_qq\": {:.12e}, \"e_emb\": {:.12e}, \"de_emb\": {:.12e}, \"residual\": {:.12e}, \"rho\": {:.6e}, \"rho_plant_ii\": {:.6e}, \"dipole_a\": {}, \"dipole_b\": {}}}",
            z.converged && i.converged, z.iterations, i.iterations, dq, (z.e_emb - i.e_emb).abs(),
            fmt_vec(&z.q_a), fmt_vec(&z.q_b), z.e_a, z.e_b, z.e_qq, z.e_emb, de_emb, de_exact - de_emb, rho, rho_plant, fmt_vec(&z.a.dipole), fmt_vec(&z.b.dipole)
        ));
    }
    let json = format!(
        "{{\n  \"system\": \"{}\", \"r_angstrom\": {:.3}, \"sector\": \"{}\", \"n_det_super\": {}, \"e_a0\": {:.12e}, \"e_b0\": {:.12e}, \"e_super\": {:.12e}, \"de_exact\": {:.12e}, \"super_residual\": {:.3e}, \"super_davidson_iters\": {}, \"cpu_seconds\": {:.3}, \"wall_seconds\": {:.3},\n  \"models\": {{\n{}\n  }}\n}}\n",
        node.system, node.r_ang, node.sector, sm.gp.space.n_det, ea, eb, sm.e_total, de_exact, sm.sol.residual, sm.sol.davidson_iters,
        cpu_seconds() - c0, t0.elapsed().as_secs_f64(), rows.join(",\n")
    );
    fs::write(&path, json).expect("write node");
    eprintln!("  {} R={:.1} Å ({}): ΔE_exact {:.6e}  cpu {:.1}s", node.system, node.r_ang, node.sector, de_exact, cpu_seconds() - c0);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let what = args.get(1).map(String::as_str).unwrap_or("hf");
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/embed".to_string()));
    fs::create_dir_all(&out).expect("out dir");
    let (f, h, o) = (sp("F"), sp("H"), sp("O"));

    // G1 — the pins, always (cheap), written once
    let pins_path = out.join("pins.json");
    let (r_hf, g_hf) = pin_hf(f, h);
    eprintln!("G1 HF: R = {r_hf:.10} bohr, |dE/dR| = {:.2e}", g_hf.abs());
    let mut pins = format!("{{\n  \"hf_r_bohr\": {r_hf:.12e}, \"hf_gradient\": {g_hf:.3e}");
    let mut water_pin = None;
    if what == "water" || what == "all" {
        let (r, t, gr, gt) = pin_water(o, h);
        eprintln!("G1 H2O: r = {r:.10} bohr, θ = {t:.10} rad ({:.4}°), |dE/dr| = {:.2e}, |dE/dθ| = {:.2e}", t.to_degrees(), gr.abs(), gt.abs());
        pins.push_str(&format!(",\n  \"h2o_r_bohr\": {r:.12e}, \"h2o_theta_rad\": {t:.12e}, \"h2o_gradient_r\": {gr:.3e}, \"h2o_gradient_theta\": {gt:.3e}"));
        water_pin = Some((r, t, gr, gt));
    }
    pins.push_str("\n}\n");
    fs::write(&pins_path, pins).expect("pins");

    if what == "hf" || what == "all" {
        eprintln!("System 1 — the HF dimer");
        for r_ang in [2.4, 2.6, 2.8, 3.0, 3.5, 4.0, 5.0, 6.0, 8.0, 10.0] {
            let sector = if r_ang >= 5.0 { "far" } else if r_ang <= 3.0 { "near" } else { "transition" };
            let (a, b) = hf_dimer(f, h, r_hf, r_ang * ANGSTROM_TO_BOHR);
            run_node(&out, &Node { system: "hf", r_ang, sector }, &a, &b);
        }
    }
    if what == "water" || what == "all" {
        let (r, t, gr, gt) = water_pin.expect("pin");
        if gr.abs() > 1e-6 || gt.abs() > 1e-6 {
            eprintln!("G1 water pin FAILED its gradient bound; System 2 is not run");
            return;
        }
        // G0 — the price on ONE node before the knots are admitted
        let g0_path = out.join("g0_price.json");
        let cpu = if g0_path.exists() {
            let s = fs::read_to_string(&g0_path).unwrap();
            s.split("\"cpu_seconds\": ").nth(1).and_then(|x| x.split(',').next()).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::INFINITY)
        } else {
            let (a, b) = water_dimer_linear(o, h, r, t, 8.0 * ANGSTROM_TO_BOHR);
            let c0 = cpu_seconds();
            let t0 = Instant::now();
            let sm = supermolecule(&a, &b);
            let cpu = cpu_seconds() - c0;
            fs::write(&g0_path, format!("{{\n  \"node\": \"h2o LINEAR R_OO=8.0\", \"n_det\": {}, \"e_super\": {:.12e}, \"residual\": {:.3e}, \"davidson_iters\": {}, \"cpu_seconds\": {:.3}, \"wall_seconds\": {:.3}, \"admitted\": {}\n}}\n",
                sm.gp.space.n_det, sm.e_total, sm.sol.residual, sm.sol.davidson_iters, cpu, t0.elapsed().as_secs_f64(), cpu <= 1800.0)).unwrap();
            eprintln!("G0: {} determinants, cpu {:.1} s, wall {:.1} s → {}", sm.gp.space.n_det, cpu, t0.elapsed().as_secs_f64(), if cpu <= 1800.0 { "ADMITTED" } else { "REFUSED (System 2 dropped)" });
            cpu
        };
        if cpu > 1800.0 {
            eprintln!("G0 refused System 2 ({cpu:.0} s > 1800 s per node); said so, not run");
        } else {
            eprintln!("System 2 — the water dimer, LINEAR");
            for r_ang in [4.0, 4.5, 5.0, 6.0, 8.0] {
                let sector = if r_ang >= 5.0 { "far" } else { "transition" };
                let (a, b) = water_dimer_linear(o, h, r, t, r_ang * ANGSTROM_TO_BOHR);
                run_node(&out, &Node { system: "h2o", r_ang, sector }, &a, &b);
            }
        }
    }
    // the S3 table, from whatever JSON is on disk
    let mut table = String::from("system  R(Å)   sector      ΔE_exact       ΔE_emb(dip)    ρ(dip)    ρ(mull)   ρ(plant ii)\n");
    let mut names: Vec<_> = fs::read_dir(&out).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.file_name().map(|n| n.to_string_lossy().contains("_R")).unwrap_or(false)).collect();
    names.sort();
    for p in names {
        let s = fs::read_to_string(&p).unwrap();
        let get = |k: &str, from: &str| -> String { from.split(&format!("\"{k}\": ")).nth(1).and_then(|x| x.split(|c| c == ',' || c == '}').next()).unwrap_or("?").trim().trim_matches('"').to_string() };
        let dip = s.split("\"dipole_exact\": ").nth(1).unwrap_or("");
        let mul = s.split("\"mulliken\": ").nth(1).unwrap_or("");
        table.push_str(&format!("{:<6}  {:>5}  {:<10}  {:>14}  {:>14}  {:>8}  {:>8}  {:>8}\n", get("system", &s), get("r_angstrom", &s), get("sector", &s), get("de_exact", &s), get("de_emb", dip), get("rho", dip), get("rho", mul), get("rho_plant_ii", dip)));
    }
    print!("{table}");
    let _ = std::io::stdout().flush();
}
