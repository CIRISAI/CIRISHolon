//! SEAM-1's runner (`conformance/water_observatory/SEAM_PREREG.md`).
//!
//! ```text
//! cargo run --release -p holon-chem --example seam_campaign -- [OUT_DIR]
//! ```
//!
//! G1, then G0 on one trimer node (wall time, thread count, device class recorded),
//! then the admitted nodes, one JSON each, skipping nodes already on disk. Every solve
//! `solve_determinant` on the host.
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::*;
use holon_chem::seam::*;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn sp(s: &str) -> Species {
    by_symbol(s).expect("species")
}

fn cpu_seconds() -> f64 {
    let s = fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let tail = &s[s.rfind(')').map(|i| i + 2).unwrap_or(0)..];
    let f: Vec<&str> = tail.split_whitespace().collect();
    let ut: f64 = f.get(11).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    let st: f64 = f.get(12).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    (ut + st) / 100.0
}

fn fmt_vec(v: &[f64]) -> String {
    format!("[{}]", v.iter().map(|x| format!("{x:.12e}")).collect::<Vec<_>>().join(", "))
}

fn pa_json(p: &PaResult) -> String {
    format!(
        "{{\"e_mono\": {}, \"e_dimer\": [{}], \"total\": {:.12e}}}",
        fmt_vec(&p.e_mono),
        p.e_dimer.iter().map(|d| format!("[{}, {}, {:.12e}]", d.0, d.1, d.2)).collect::<Vec<_>>().join(", "),
        p.total
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = PathBuf::from(args.get(1).cloned().unwrap_or_else(|| "../conformance/water_observatory/seam".to_string()));
    fs::create_dir_all(&out).expect("out dir");
    let (f, h) = (sp("F"), sp("H"));
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);

    // G1
    let (r_hf, g_hf) = pin_hf(f, h);
    eprintln!("G1: R_HF = {r_hf:.10} bohr, |dE/dR| = {:.2e}", g_hf.abs());
    if g_hf.abs() > 1e-6 {
        eprintln!("G1 FAILED; nothing run");
        return;
    }

    // G0 — the price in the clock the budget is spent in
    let g0_path = out.join("g0_price.json");
    let wall = if g0_path.exists() {
        let s = fs::read_to_string(&g0_path).unwrap();
        s.split("\"wall_seconds\": ").nth(1).and_then(|x| x.split(',').next()).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::INFINITY)
    } else {
        let frags = hf_chain(f, h, r_hf, 8.0 * ANGSTROM_TO_BOHR, 3);
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let sm = supermolecule_all(&frags);
        let wall = t0.elapsed().as_secs_f64();
        let cpu = cpu_seconds() - c0;
        let admitted = if wall <= 900.0 { "all eight" } else if wall <= 1800.0 { "the far sector plus 3.0" } else { "none" };
        fs::write(&g0_path, format!("{{\n  \"node\": \"hf chain R_FF=8.0\", \"n_det\": {}, \"e_super\": {:.12e}, \"residual\": {:.3e}, \"davidson_iters\": {}, \"wall_seconds\": {:.3}, \"cpu_seconds\": {:.3}, \"threads_available\": {}, \"device_class\": \"host f64, solve_determinant\", \"admitted\": \"{}\"\n}}\n",
            sm.gp.space.n_det, sm.e_total, sm.sol.residual, sm.sol.davidson_iters, wall, cpu, threads, admitted)).unwrap();
        eprintln!("G0: {} determinants, wall {wall:.1} s, cpu {cpu:.1} s on {threads} threads → {admitted}", sm.gp.space.n_det);
        wall
    };
    let grid: Vec<f64> = if wall <= 900.0 {
        vec![2.6, 2.8, 3.0, 3.5, 4.0, 5.0, 6.0, 8.0]
    } else if wall <= 1800.0 {
        vec![3.0, 5.0, 6.0, 8.0]
    } else {
        eprintln!("G0 refused the grid ({wall:.0} s > 1800 s); said so, not run");
        return;
    };

    for r_ang in grid {
        let sector = if r_ang >= 5.0 { "far" } else if r_ang <= 3.0 { "near" } else { "transition" };
        let path = out.join(format!("hf3_R{r_ang:.1}.json"));
        if path.exists() {
            eprintln!("  R={r_ang:.1} Å: exists, skipped (resume)");
            continue;
        }
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let frags = hf_chain(f, h, r_hf, r_ang * ANGSTROM_TO_BOHR, 3);
        let sm = supermolecule_all(&frags);
        let bare = bare_pa(&frags);
        let de3_bare = sm.e_total - bare.total;
        let zeros: Vec<Vec<f64>> = frags.iter().map(|fr| vec![0.0; fr.species.len()]).collect();
        let g3 = (ee_pa(&frags, &zeros).total - bare.total).abs();
        let mut models = Vec::new();
        for (model, name) in [(ChargeModel::DipoleExact, "dipole_exact"), (ChargeModel::Mulliken, "mulliken")] {
            let z = embed_many(&frags, model, Start::Zero);
            let i = embed_many(&frags, model, Start::Isolated);
            let dq = z.charges.iter().flatten().zip(i.charges.iter().flatten()).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
            let ez = ee_pa(&frags, &z.charges);
            let ei = ee_pa(&frags, &i.charges);
            let r_emb = sm.e_total - ez.total;
            let kappa = r_emb.abs() / de3_bare.abs();
            let plant = ee_pa_with(&frags, &z.charges, false);
            let r_plant = sm.e_total - plant.total;
            let d_ab = (ez.e_dimer[0].2 - plant.e_dimer[0].2).abs();
            models.push(format!(
                "    \"{name}\": {{\"converged\": {}, \"sweeps_zero\": {}, \"sweeps_isolated\": {}, \"g2_dq\": {:.3e}, \"g2_de\": {:.3e}, \"charges\": [{}], \"dipoles_z\": {}, \"ee_pa\": {}, \"r_emb\": {:.12e}, \"kappa\": {:.6e}, \"plant_i_r\": {:.12e}, \"plant_i_kappa\": {:.6e}, \"plant_i_carrier_dAB\": {:.3e}}}",
                z.converged && i.converged, z.iterations, i.iterations, dq, (ez.total - ei.total).abs(),
                z.charges.iter().map(|q| fmt_vec(q)).collect::<Vec<_>>().join(", "),
                fmt_vec(&z.monomers.iter().map(|m| m.dipole[2]).collect::<Vec<_>>()),
                pa_json(&ez), r_emb, kappa, r_plant, r_plant.abs() / de3_bare.abs(), d_ab
            ));
        }
        let json = format!(
            "{{\n  \"system\": \"hf3 chain\", \"r_angstrom\": {r_ang:.3}, \"sector\": \"{sector}\", \"n_det_super\": {}, \"e_super\": {:.12e}, \"super_residual\": {:.3e}, \"super_davidson_iters\": {}, \"bare\": {}, \"de3_bare\": {:.12e}, \"g3_identity\": {:.3e}, \"wall_seconds\": {:.3}, \"cpu_seconds\": {:.3}, \"threads_available\": {threads},\n  \"models\": {{\n{}\n  }}\n}}\n",
            sm.gp.space.n_det, sm.e_total, sm.sol.residual, sm.sol.davidson_iters, pa_json(&bare), de3_bare, g3,
            t0.elapsed().as_secs_f64(), cpu_seconds() - c0, models.join(",\n")
        );
        fs::write(&path, json).expect("write node");
        eprintln!("  R={r_ang:.1} Å ({sector}): ΔE3_bare {de3_bare:.6e}  G3 {g3:.1e}  wall {:.0}s", t0.elapsed().as_secs_f64());
    }
    // the S1 table
    let mut names: Vec<_> = fs::read_dir(&out).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.file_name().map(|n| n.to_string_lossy().starts_with("hf3_R")).unwrap_or(false)).collect();
    names.sort();
    println!("R(Å)   sector      ΔE3_bare        r_emb(dip)      κ(dip)     κ(mull)    κ(plant i)  G3");
    for p in names {
        let s = fs::read_to_string(&p).unwrap();
        let get = |k: &str, from: &str| -> String { from.split(&format!("\"{k}\": ")).nth(1).and_then(|x| x.split(|c| c == ',' || c == '}').next()).unwrap_or("?").trim().trim_matches('"').to_string() };
        let dip = s.split("\"dipole_exact\": ").nth(1).unwrap_or("");
        let mul = s.split("\"mulliken\": ").nth(1).unwrap_or("");
        println!("{:>5}  {:<10}  {:>14}  {:>14}  {:>9}  {:>9}  {:>9}  {}", get("r_angstrom", &s), get("sector", &s), get("de3_bare", &s), get("r_emb", dip), get("kappa", dip), get("kappa", mul), get("plant_i_kappa", dip), get("g3_identity", &s));
    }
}
