//! EMBED-2's runner (`conformance/water_observatory/EMBED2_PREREG.md`).
//!
//! ```text
//! cargo run --release -p holon-chem --example embed2_campaign -- [SEAM_DIR] [OUT_DIR]
//! ```
//!
//! No trimer is solved: the exact energies are SEAM-1's, read by node from `SEAM_DIR`
//! (`hf3_R*.json`, and `floor_R*.json` for the moved chain), and G0 refuses a node whose
//! own bare pairwise sum differs from the record's by more than 1e-10 hartree.
use holon_chem::density_embed::*;
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::*;
use holon_chem::seam::{bare_pa, hf_chain};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn sp(s: &str) -> Species {
    by_symbol(s).expect("species")
}
const R_HF: f64 = 1.879437929774;
const SHIFT: [f64; 3] = [0.37, 0.21, 0.5];

fn get_f(s: &str, key: &str) -> Option<f64> {
    s.split(&format!("\"{key}\": ")).nth(1).and_then(|x| x.split(|c| c == ',' || c == '}').next()).and_then(|x| x.trim().parse().ok())
}
fn get_in(s: &str, section: &str, key: &str) -> Option<f64> {
    s.split(&format!("\"{section}\": ")).nth(1).and_then(|x| get_f(x, key))
}

fn cpu_seconds() -> f64 {
    let s = fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let tail = &s[s.rfind(')').map(|i| i + 2).unwrap_or(0)..];
    let f: Vec<&str> = tail.split_whitespace().collect();
    let ut: f64 = f.get(11).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    let st: f64 = f.get(12).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    (ut + st) / 100.0
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seam = PathBuf::from(args.get(1).cloned().unwrap_or_else(|| "../conformance/water_observatory/seam".to_string()));
    let out = PathBuf::from(args.get(2).cloned().unwrap_or_else(|| "../conformance/water_observatory/embed2".to_string()));
    fs::create_dir_all(&out).expect("out");
    let (f, h) = (sp("F"), sp("H"));
    for r_ang in [2.6, 2.8, 3.0, 3.5, 4.0, 5.0, 6.0, 8.0] {
        let sector = if r_ang >= 5.0 { "far" } else if r_ang <= 3.0 { "near" } else { "transition" };
        let path = out.join(format!("hf3_R{r_ang:.1}.json"));
        if path.exists() {
            eprintln!("  R={r_ang:.1} Å: exists, skipped");
            continue;
        }
        let rec = fs::read_to_string(seam.join(format!("hf3_R{r_ang:.1}.json"))).expect("SEAM-1 record");
        let e_super = get_f(&rec, "e_super").expect("e_super");
        let bare_rec = get_in(&rec, "bare", "total").expect("bare total");
        let de3 = get_f(&rec, "de3_bare").expect("de3");
        let kappa_q = get_in(&rec, "dipole_exact", "kappa").expect("kappa_q");
        let t0 = Instant::now();
        let c0 = cpu_seconds();
        let frags = hf_chain(f, h, R_HF, r_ang * ANGSTROM_TO_BOHR, 3);
        // G0: the referee is the record's
        let bare = bare_pa(&frags);
        let g0 = (bare.total - bare_rec).abs();
        if g0 > 1e-10 {
            eprintln!("  R={r_ang:.1} Å: G0 REFUSED — bare sum differs from the record by {g0:.2e}");
            fs::write(&path, format!("{{\"r_angstrom\": {r_ang:.3}, \"refused\": \"G0\", \"g0\": {g0:.3e}}}\n")).unwrap();
            continue;
        }
        let z = embed_densities(&frags, DensityStart::Zero);
        let i = embed_densities(&frags, DensityStart::Isolated);
        let dp = z.densities.iter().flatten().zip(i.densities.iter().flatten()).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        let ez = rho_pa(&frags, &z.densities);
        let ei = rho_pa(&frags, &i.densities);
        let r_rho = e_super - ez.total;
        let kappa_rho = r_rho.abs() / de3.abs();
        let plant = rho_pa_with(&frags, &z.densities, false, true);
        let kappa_plant = (e_super - plant.total).abs() / de3.abs();
        // the floor on the far sector, from the moved chain and SEAM-1's moved trimer
        let mut floor_txt = String::from("null");
        if sector == "far" {
            let fl = fs::read_to_string(seam.join(format!("floor_R{r_ang:.1}.json"))).expect("floor record");
            let e_super_moved = get_in(&fl, "moved", "e_super").expect("moved e_super");
            let moved: Vec<Fragment> = frags.iter().map(|fr| fr.translated(SHIFT)).collect();
            let zm = embed_densities(&moved, DensityStart::Zero);
            let em = rho_pa(&moved, &zm.densities);
            let r_moved = e_super_moved - em.total;
            let floor = (r_rho - r_moved).abs();
            floor_txt = format!("{{\"r_rho_moved\": {r_moved:.12e}, \"floor\": {floor:.6e}, \"posable_ratio\": {:.3}, \"sweeps_moved\": {}}}", if floor > 0.0 { r_rho.abs() / floor } else { f64::INFINITY }, zm.sweeps);
        }
        let json = format!(
            "{{\n  \"r_angstrom\": {r_ang:.3}, \"sector\": \"{sector}\", \"e_super_record\": {e_super:.12e}, \"de3_bare\": {de3:.12e}, \"kappa_q\": {kappa_q:.6e}, \"g0\": {g0:.3e},\n  \"g2\": {{\"converged\": {}, \"sweeps_zero\": {}, \"sweeps_isolated\": {}, \"dp\": {dp:.3e}, \"de\": {:.3e}}},\n  \"rho_pa\": {:.12e}, \"e_mono\": [{}], \"e_dimer\": [{}],\n  \"r_rho\": {r_rho:.12e}, \"kappa_rho\": {kappa_rho:.6e}, \"kappa_plant_ii\": {kappa_plant:.6e},\n  \"floor\": {floor_txt},\n  \"wall_seconds\": {:.2}, \"cpu_seconds\": {:.2}\n}}\n",
            z.converged && i.converged, z.sweeps, i.sweeps, (ez.total - ei.total).abs(), ez.total,
            ez.e_mono.iter().map(|x| format!("{x:.12e}")).collect::<Vec<_>>().join(", "),
            ez.e_dimer.iter().map(|d| format!("[{}, {}, {:.12e}]", d.0, d.1, d.2)).collect::<Vec<_>>().join(", "),
            t0.elapsed().as_secs_f64(), cpu_seconds() - c0
        );
        fs::write(&path, json).unwrap();
        eprintln!("  R={r_ang:.1} Å ({sector}): κ_q {kappa_q:.3e}  κ_ρ {kappa_rho:.3e}  r_ρ {r_rho:+.3e}  plant(ii) κ {kappa_plant:.2}  G2 Δρ {dp:.1e}  wall {:.0}s", t0.elapsed().as_secs_f64());
    }
    let mut names: Vec<_> = fs::read_dir(&out).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).collect();
    names.sort();
    println!("R(Å)   sector      ΔE3_bare        κ_q         κ_ρ         r_ρ            floor       posable");
    for p in names {
        let s = fs::read_to_string(&p).unwrap();
        let g = |k: &str| get_f(&s, k).map(|x| format!("{x:.3e}")).unwrap_or("—".into());
        let sec = s.split("\"sector\": \"").nth(1).and_then(|x| x.split('"').next()).unwrap_or("?");
        let fl = get_in(&s, "floor", "floor").map(|x| format!("{x:.1e}")).unwrap_or("—".into());
        let pr = get_in(&s, "floor", "posable_ratio").map(|x| format!("{x:.0}")).unwrap_or("—".into());
        println!("{:>5}  {:<10}  {:>14}  {:>10}  {:>10}  {:>14}  {:>10}  {:>7}", g("r_angstrom"), sec, g("de3_bare"), g("kappa_q"), g("kappa_rho"), g("r_rho"), fl, pr);
    }
}
