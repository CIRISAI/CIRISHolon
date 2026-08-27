//! Q7b: per-region certification on the symmetric three-level box, where the regimes ARE distinct.
//! `Q7B_SEAM_PREREG.md` with A1(Q7b). Exactness gates first, always.
//!
//! The family was licensed by the mandatory spread pre-check (11 of 16 probe points split at the
//! staked margin, against Q7's 0 of 84) before this prereg was frozen.

use q_seam::chart::Chart;
use q_seam::hubbard::{free_reference, Hubbard};
use q_seam::lanczos::ground_state;
use q_seam::observables::ExactObservables;
use q_seam::region::{
    best_global, best_per_region, best_scaling, regions, score, Cand, Configuration, RegionInstance,
};
use std::io::Write;

fn main() {
    let mut configs: Vec<Configuration> = Vec::new();
    let mut voids: Vec<(usize, f64, f64, String)> = Vec::new();
    let mut worst_ruler = 0.0f64;
    let mut worst_mirror = 0.0f64;
    let mut worst_sym = 0.0f64;
    let mut worst_s2 = 0.0f64;

    for &n in &[q_seam::Q7B_SITES] {
        for &a in &q_seam::Q7B_V {
            let v = q_seam::q7b_box(a);
            let vneg: Vec<f64> = v.iter().map(|x| -x).collect();
            for &u in &q_seam::Q7B_U {
                let h = Hubbard::with_potential(n, 1.0, u, &v);
                let g = match ground_state(&h) {
                    Some(g) => g,
                    None => { voids.push((n, u, a, "lanczos residual gate".into())); continue; }
                };
                // G-E6: uniqueness in sector. P-VOID-a expects this to fire at deep trap + large U.
                let gap = g.first_excited - g.energy;
                if gap < 1e-6 {
                    voids.push((n, u, a, format!("in-sector gap {gap:e} (P-VOID-a)")));
                    continue;
                }
                let o = ExactObservables::measure(&h, &g.vector);

                // G-E5b: the spin anchor's own residual. If this blows, the computed vector is a
                // mixture and the configuration is VOID - never evidence against the anchor.
                let sym = o.magnetization.iter().map(|x| x.abs()).fold(0.0, f64::max);
                if sym > 1e-11 {
                    voids.push((n, u, a, format!("spin residual {sym:e} (quasi-degeneracy)")));
                    continue;
                }
                worst_sym = worst_sym.max(sym);
                worst_s2 = worst_s2.max(o.s_squared);

                // G7-E7: the free-fermion ruler, at U = 0, every a.
                if u == 0.0 {
                    let fr = free_reference(n, 1.0, &v);
                    worst_ruler = worst_ruler
                        .max((fr.energy - g.energy).abs() / g.energy.abs().max(1.0));
                }
                // G7-E9: the demoted particle-hole identity, E0(v) = E0(-v) + 2*sum(v).
                let hneg = Hubbard::with_potential(n, 1.0, u, &vneg);
                if let Some(gn) = ground_state(&hneg) {
                    let pred = gn.energy + 2.0 * v.iter().sum::<f64>();
                    worst_mirror = worst_mirror.max((pred - g.energy).abs() / g.energy.abs().max(1.0));
                }

                let chart = match Chart::best_with(n, 1.0, u, &v) {
                    Some(c) => c,
                    None => { voids.push((n, u, a, "no SCF guess converged".into())); continue; }
                };
                if chart.idempotency > 1e-12 || chart.energy < g.energy - 1e-10 {
                    voids.push((n, u, a, "chart gate G-C2/G-C3".into()));
                    continue;
                }

                let rs: Vec<RegionInstance> = regions(n)
                    .into_iter()
                    .enumerate()
                    .map(|(k, b)| RegionInstance::measure(n, u, a, k, b, &o, &chart))
                    .collect();
                configs.push(Configuration { sites: n, u, a, regions: rs });
                eprintln!("done N={n} a={a} U={u}");
            }
        }
    }

    println!("=== EXACTNESS (Q7) ===");
    println!("G7-E7 free-fermion ruler, worst rel: {worst_ruler:.3e}  (gate 1e-12)");
    println!("G7-E9 mirror identity E0(v)=E0(-v)+2sum(v), worst rel: {worst_mirror:.3e}  (gate 1e-11)");
    println!("G-E5b spin residual, worst: {worst_sym:.3e}  (gate 1e-11)");
    println!("<S^2> worst (MEASURED, no anchor depends on it): {worst_s2:.3e}");

    let total = q_seam::Q7B_U.len() * q_seam::Q7B_V.len();
    println!("\n=== VOID BUDGET (A1(Q7)/P4) ===");
    println!("VOID: {} of {total}  (underpowered if > 12)", voids.len());
    for (n, u, a, why) in &voids { println!("  VOID N={n} U={u} V={a}: {why}"); }
    for &a in &q_seam::Q7B_V {
        let c = voids.iter().filter(|v| v.2 == a).count();
        if c > 3 { println!("  V-COLUMN V={a} UNUSABLE: {c} of 7 VOID"); }
    }

    let split: Vec<&Configuration> = configs.iter().filter(|c| c.spatially_split()).collect();
    println!("\n=== G7-FIT: the family-fitness precondition ===");
    println!("spatially split (min E_r <= 0.5 AND max E_r >= 2.0): {} of {total}  (need >= 8)", split.len());
    println!("verdict: {}", if split.len() >= 8 { "PASS - the family posed the question" }
             else { "FAIL -> Q7 is VOID, not killed: the family did not pose the question" });
    for c in split.iter().take(12) {
        let e: Vec<String> = c.regions.iter().map(|r| format!("{:.2}", r.e_r)).collect();
        println!("  split N={} U={} a={}: E_r = [{}]", c.sites, c.u, c.a, e.join(" "));
    }

    println!("\n=== THE REFUSAL MAP, at V=4 ===");
    for c in configs.iter().filter(|c| c.a == 4.0) {
        let d3: String = c.regions.iter().map(|r| if Cand::D3.certifies(r) { 'C' } else { 'R' }).collect();
        let d4: String = c.regions.iter().map(|r| if Cand::D4.certifies(r) { 'C' } else { 'R' }).collect();
        let truth: String = c.regions.iter().map(|r| if r.honest() { '.' } else { 'X' }).collect();
        println!("  V=4 U={:>4}: truth {truth}   D3 {d3}   D4 {d4}", c.u);
    }

    println!("\n=== JOINT GATE (5 clauses) ===");
    for cand in [Cand::D1, Cand::D1b, Cand::D2, Cand::D3, Cand::D4, Cand::D5, Cand::N1, Cand::N2] {
        let s = score(cand.label(), &configs, |r| cand.certifies(r));
        println!("{:<26} {}", cand.label(), s.clause_report());
        for (n, u, a, obs, e) in s.false_positives.iter().take(4) {
            println!("{:<26}   FP N={n} U={u} a={a} worst=R{} E_r={e:.2}", "", obs + 1);
        }
        if s.false_positives.len() > 4 {
            println!("{:<26}   ... and {} more FPs", "", s.false_positives.len() - 4);
        }
    }

    println!("\n=== SEVERITY BASELINES (post-hoc optimal, A1(Q7)/P3) ===");
    match best_global(&configs, &q_seam::Q7B_U) {
        Some((u, s)) => println!("N3 global U<={u} [1 param]: {}", s.clause_report()),
        None => println!("N3 INFEASIBLE"),
    }
    match best_per_region(&configs, &q_seam::Q7B_U) {
        Some((th, s)) => {
            println!("N4 per-region U<=u*(r) [{} params, joint across all a]: {}", th.len(), s.clause_report());
            println!("   thresholds (N, region, u*): {th:?}");
        }
        None => println!("N4 INFEASIBLE"),
    }
    match best_scaling(&configs) {
        Some((al, be, s)) => println!("N5 U<={al}+{be}*d(r) [2 params]: {}", s.clause_report()),
        None => println!("N5 INFEASIBLE"),
    }

    let dir = "/home/emoore/CIRISOntology/sim_engine/output/q7b_seam";
    std::fs::create_dir_all(dir).unwrap();
    let mut f = std::fs::File::create(format!("{dir}/q7b.json")).unwrap();
    writeln!(f, "{{\"voids\":{},\"split\":{},\"regions\":[", voids.len(), split.len()).unwrap();
    let mut first = true;
    for c in &configs {
        for r in &c.regions {
            writeln!(f, "{}{{\"N\":{},\"U\":{},\"a\":{},\"r\":{},\"e_r\":{},\"honest\":{},\
\"break_spin\":{},\"break_refl\":{},\"self_audit\":{},\"dbool_exact\":{},\"dbool_chart\":{},\
\"norm\":[{},{},{},{},{}]}}",
                if first { first = false; "" } else { "," },
                r.sites, r.u, r.a, r.index, r.e_r, r.honest(), r.break_spin, r.break_refl,
                r.self_audit, r.dbool_exact, r.dbool_chart,
                r.normalized[0], r.normalized[1], r.normalized[2], r.normalized[3], r.normalized[4]).unwrap();
        }
    }
    writeln!(f, "]}}").unwrap();
    println!("\nwrote {dir}/q7b.json");
}
