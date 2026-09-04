//! EMBED-3's runner (`conformance/water_observatory/EMBED3_PREREG.md`).
//!
//! ```text
//! cargo run --release -p holon-chem --example embed3_campaign -- hf4|water|all [OUT_DIR]
//! ```
use holon_chem::density_embed::*;
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::*;
use holon_chem::seam::hf_chain;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn sp(s: &str) -> Species {
    by_symbol(s).expect("species")
}
const R_HF: f64 = 1.879437929774;
const SHIFT: [f64; 3] = [0.37, 0.21, 0.5];
/// EMBED-2's harvested three-body residual of the HF chain at 5.0 Å (embed2/hf3_R5.0.json).
const R3_INF: f64 = -1.467178e-8;
/// EMBED-1's water pins (embed/pins.json) and its 8.0 Å exact dimer (embed/g0_price.json).
const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;
const E_DIMER_8A_RECORD: f64 = -150.0467322495;

fn cpu_seconds() -> f64 {
    let s = fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let tail = &s[s.rfind(')').map(|i| i + 2).unwrap_or(0)..];
    let f: Vec<&str> = tail.split_whitespace().collect();
    let ut: f64 = f.get(11).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    let st: f64 = f.get(12).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    (ut + st) / 100.0
}

fn max_dp(a: &[Vec<f64>], b: &[Vec<f64>]) -> f64 {
    a.iter().flatten().zip(b.iter().flatten()).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max)
}

/// A–B–C at 5 Å and D at `r_cd` beyond C, collinear.
fn four_chain(f: Species, h: Species, r_cd_ang: f64) -> Vec<Fragment> {
    let mut v = hf_chain(f, h, R_HF, 5.0 * ANGSTROM_TO_BOHR, 3);
    let d = v[0].translated([0.0, 0.0, (10.0 + r_cd_ang) * ANGSTROM_TO_BOHR]);
    v.push(d);
    v
}

/// r_3(ABC | D) on a fragment list, with the plant switch.
fn r3_in_field(frags: &[Fragment], dens: &[Vec<f64>], drop: Option<usize>) -> (f64, f64, f64) {
    let e = subset_in_field(frags, dens, &[0, 1, 2], drop).e_total;
    let pa = rho_pa_subset(frags, dens, &[0, 1, 2], drop).total;
    (e - pa, e, pa)
}

fn run_hf4(out: &PathBuf, threads: usize) {
    let (f, h) = (sp("F"), sp("H"));
    // G0 — the price of one trimer-in-field solve
    let g0 = out.join("hf4_g0.json");
    let wall = if g0.exists() {
        fs::read_to_string(&g0).unwrap().split("\"wall_seconds\": ").nth(1).and_then(|x| x.split(',').next()).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::INFINITY)
    } else {
        let frags = four_chain(f, h, 12.0);
        let z = embed_densities(&frags, DensityStart::Zero);
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let ds = subset_in_field(&frags, &z.densities, &[0, 1, 2], None);
        let wall = t0.elapsed().as_secs_f64();
        fs::write(&g0, format!("{{\"node\": \"hf4 R_CD=12 trimer in field\", \"n_det\": {}, \"e\": {:.12e}, \"residual\": {:.3e}, \"wall_seconds\": {:.3}, \"cpu_seconds\": {:.3}, \"threads\": {threads}, \"admitted\": {}}}\n", ds.gp.space.n_det, ds.e_total, ds.sol.residual, wall, cpu_seconds() - c0, wall <= 900.0)).unwrap();
        eprintln!("G0 hf4: {} dets, wall {wall:.1} s on {threads} threads → {}", ds.gp.space.n_det, if wall <= 900.0 { "admitted" } else { "REFUSED" });
        wall
    };
    if wall > 900.0 {
        eprintln!("System A refused by G0 ({wall:.0} s > 900 s)");
        return;
    }
    // G2 — D removed reproduces EMBED-2
    let g2 = out.join("hf4_g2.json");
    if !g2.exists() {
        let frags = hf_chain(f, h, R_HF, 5.0 * ANGSTROM_TO_BOHR, 3);
        let z = embed_densities(&frags, DensityStart::Zero);
        let (r3, e, pa) = r3_in_field(&frags, &z.densities, None);
        fs::write(&g2, format!("{{\"r3_no_D\": {r3:.12e}, \"r3_embed2\": {R3_INF:.12e}, \"g2\": {:.3e}, \"e_abc\": {e:.12e}, \"rho_pa\": {pa:.12e}}}\n", (r3 - R3_INF).abs())).unwrap();
        eprintln!("G2: r_3 with D removed {r3:.6e} vs EMBED-2 {R3_INF:.6e} → |Δ| = {:.1e}", (r3 - R3_INF).abs());
    }
    for r_cd in [4.0, 6.0, 8.0, 12.0] {
        let path = out.join(format!("hf4_Rcd{r_cd:.1}.json"));
        if path.exists() {
            eprintln!("  R_CD={r_cd:.1} Å: exists, skipped");
            continue;
        }
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let frags = four_chain(f, h, r_cd);
        let z = embed_densities(&frags, DensityStart::Zero);
        let i = embed_densities(&frags, DensityStart::Isolated);
        let dp = max_dp(&z.densities, &i.densities);
        let (r3, e_abc, pa) = r3_in_field(&frags, &z.densities, None);
        let pa_i = rho_pa_subset(&frags, &i.densities, &[0, 1, 2], None).total;
        let delta = r3 - R3_INF;
        // the floor: the moved four-chain, everything repeated
        let moved: Vec<Fragment> = frags.iter().map(|fr| fr.translated(SHIFT)).collect();
        let zm = embed_densities(&moved, DensityStart::Zero);
        let (r3m, _, _) = r3_in_field(&moved, &zm.densities, None);
        let floor = (r3 - r3m).abs();
        // plant (ii) at 12 Å only: D's nuclei dropped
        let plant = if r_cd == 12.0 {
            let (r3p, _, _) = r3_in_field(&frags, &z.densities, Some(3));
            format!("{{\"r3\": {r3p:.12e}, \"delta\": {:.12e}, \"carrier_nn_cd\": {:.6e}}}", r3p - R3_INF, nn_between(&frags[2], &frags[3]))
        } else { "null".to_string() };
        fs::write(&path, format!(
            "{{\n  \"r_cd_angstrom\": {r_cd:.3}, \"r3\": {r3:.12e}, \"r3_inf\": {R3_INF:.12e}, \"delta\": {delta:.12e}, \"delta_over_r3\": {:.6e}, \"e_abc_in_field\": {e_abc:.12e}, \"rho_pa_abc_in_field\": {pa:.12e},\n  \"g1\": {{\"converged\": {}, \"sweeps_zero\": {}, \"sweeps_isolated\": {}, \"dp\": {dp:.3e}, \"dpa\": {:.3e}}},\n  \"floor\": {{\"r3_moved\": {r3m:.12e}, \"floor\": {floor:.6e}, \"posable_ratio\": {:.3}}},\n  \"plant_ii\": {plant},\n  \"wall_seconds\": {:.1}, \"cpu_seconds\": {:.1}, \"threads\": {threads}\n}}\n",
            delta.abs() / R3_INF.abs(), z.converged && i.converged, z.sweeps, i.sweeps, (pa - pa_i).abs(), if floor > 0.0 { delta.abs() / floor } else { f64::INFINITY }, t0.elapsed().as_secs_f64(), cpu_seconds() - c0)).unwrap();
        eprintln!("  R_CD={r_cd:.1} Å: r_3 {r3:+.4e}  Δ {delta:+.3e} ({:.2e} of r_3)  floor {floor:.1e}  G1 Δρ {dp:.1e}  wall {:.0}s", delta.abs() / R3_INF.abs(), t0.elapsed().as_secs_f64());
    }
}

fn run_water(out: &PathBuf, threads: usize) {
    let (o, h) = (sp("O"), sp("H"));
    let mk = |r_oo: f64| water_dimer_linear(o, h, H2O_R, H2O_THETA, r_oo * ANGSTROM_TO_BOHR);
    // G0 — the price on the 6.0 Å node, wall time
    let g0 = out.join("water_g0.json");
    let wall = if g0.exists() {
        fs::read_to_string(&g0).unwrap().split("\"wall_seconds\": ").nth(1).and_then(|x| x.split(',').next()).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::INFINITY)
    } else {
        let (a, b) = mk(6.0);
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let sm = supermolecule(&a, &b);
        let wall = t0.elapsed().as_secs_f64();
        fs::write(&g0, format!("{{\"node\": \"water LINEAR R_OO=6.0\", \"n_det\": {}, \"e_super\": {:.12e}, \"residual\": {:.3e}, \"davidson_iters\": {}, \"wall_seconds\": {:.3}, \"cpu_seconds\": {:.3}, \"threads\": {threads}, \"admitted\": {}}}\n", sm.gp.space.n_det, sm.e_total, sm.sol.residual, sm.sol.davidson_iters, wall, cpu_seconds() - c0, wall <= 900.0)).unwrap();
        eprintln!("G0 water: {} dets, wall {wall:.1} s on {threads} threads → {}", sm.gp.space.n_det, if wall <= 900.0 { "admitted" } else { "REFUSED" });
        wall
    };
    if wall > 900.0 {
        eprintln!("System B refused by G0 ({wall:.0} s > 900 s)");
        return;
    }
    for r_oo in [4.0, 4.5, 5.0, 6.0, 8.0] {
        let sector = if r_oo >= 5.0 { "far" } else { "transition" };
        let path = out.join(format!("water_R{r_oo:.1}.json"));
        if path.exists() {
            eprintln!("  R_OO={r_oo:.1} Å: exists, skipped");
            continue;
        }
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let (a, b) = mk(r_oo);
        let e_a0 = solve_embedded(&a.species, &a.centers, &[]).e_total;
        let e_b0 = solve_embedded(&b.species, &b.centers, &[]).e_total;
        let sm = if r_oo == 6.0 && g0.exists() {
            // the G0 node's own energy is its record; re-solve anyway so every node's JSON is
            // self-contained (the price is the same either way)
            supermolecule(&a, &b)
        } else {
            supermolecule(&a, &b)
        };
        let de_exact = sm.e_total - e_a0 - e_b0;
        let record_check = if r_oo == 8.0 { (sm.e_total - E_DIMER_8A_RECORD).abs() } else { f64::NAN };
        // charges (EMBED-1's instrument) and its plant (i)
        let q = embed_pair(&a, &b, ChargeModel::DipoleExact, Start::Zero, false);
        let qp = embed_pair(&a, &b, ChargeModel::DipoleExact, Start::Zero, true);
        let de_q = q.e_emb - e_a0 - e_b0;
        let de_qp = qp.e_emb - e_a0 - e_b0;
        // densities
        let frags = vec![a.clone(), b.clone()];
        let z = embed_densities(&frags, DensityStart::Zero);
        let i = embed_densities(&frags, DensityStart::Isolated);
        let dp = max_dp(&z.densities, &i.densities);
        let ea = solve_in_densities(&frags[0], &[Partner::new(&frags[1], &z.densities[1])]).e_total;
        let eb = solve_in_densities(&frags[1], &[Partner::new(&frags[0], &z.densities[0])]).e_total;
        let es_ab = classical_interaction(&frags[0], &z.densities[0], &frags[1], &z.densities[1]);
        let es_ba = classical_interaction(&frags[1], &z.densities[1], &frags[0], &z.densities[0]);
        let de_rho = ea + eb - es_ab - e_a0 - e_b0;
        let rho_q = (de_exact - de_q).abs() / de_exact.abs();
        let rho_qp = (de_exact - de_qp).abs() / de_exact.abs();
        let rho_rho = (de_exact - de_rho).abs() / de_exact.abs();
        fs::write(&path, format!(
            "{{\n  \"r_oo_angstrom\": {r_oo:.3}, \"sector\": \"{sector}\", \"n_det_super\": {}, \"e_super\": {:.12e}, \"super_residual\": {:.3e}, \"super_davidson_iters\": {}, \"record_check_8A\": {record_check:.3e}, \"e_a0\": {e_a0:.12e}, \"e_b0\": {e_b0:.12e}, \"de_exact\": {de_exact:.12e},\n  \"charges\": {{\"converged\": {}, \"iterations\": {}, \"q_a\": [{}], \"e_qq\": {:.12e}, \"de_q\": {de_q:.12e}, \"rho_q\": {rho_q:.6e}, \"rho_q_plant_i\": {rho_qp:.6e}}},\n  \"densities\": {{\"converged\": {}, \"sweeps_zero\": {}, \"sweeps_isolated\": {}, \"dp\": {dp:.3e}, \"e_a_in_b\": {ea:.12e}, \"e_b_in_a\": {eb:.12e}, \"e_es\": {es_ab:.12e}, \"g4\": {:.3e}, \"de_rho\": {de_rho:.12e}, \"rho_rho\": {rho_rho:.6e}}},\n  \"wall_seconds\": {:.1}, \"cpu_seconds\": {:.1}, \"threads\": {threads}\n}}\n",
            sm.gp.space.n_det, sm.e_total, sm.sol.residual, sm.sol.davidson_iters, q.converged, q.iterations,
            q.q_a.iter().map(|x| format!("{x:.9e}")).collect::<Vec<_>>().join(", "), q.e_qq,
            z.converged && i.converged, z.sweeps, i.sweeps, (es_ab - es_ba).abs(), t0.elapsed().as_secs_f64(), cpu_seconds() - c0)).unwrap();
        eprintln!("  R_OO={r_oo:.1} Å ({sector}): ΔE_exact {de_exact:+.4e}  ρ_q {rho_q:.3e}  ρ_ρ {rho_rho:.3e}  plant(i) {rho_qp:.2}  G4 {:.1e}  wall {:.0}s", (es_ab - es_ba).abs(), t0.elapsed().as_secs_f64());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let what = args.get(1).map(String::as_str).unwrap_or("all");
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/embed3".to_string()));
    fs::create_dir_all(&out).expect("out");
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    if what == "hf4" || what == "all" {
        eprintln!("System A — the residual's field dependence");
        run_hf4(&out, threads);
    }
    if what == "water" || what == "all" {
        eprintln!("System B — the water dimer's far field");
        run_water(&out, threads);
    }
}
