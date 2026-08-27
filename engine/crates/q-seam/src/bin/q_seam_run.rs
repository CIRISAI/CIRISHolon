//! The Q5 sweep runner: exactness gates, chart, certificate, mutants, joint gate.
//!
//! Writes `sim_engine/output/q_seam/q5.json` by hand (the crate has zero runtime dependencies).

use q_seam::audit::Stability;
use q_seam::certificate::{best_fixed_cutoff, best_scaling_cutoff, score, Configuration, Criterion};
use q_seam::chart::Chart;
use q_seam::dense::jacobi;
use q_seam::hubbard::Hubbard;
use q_seam::lanczos::ground_state;
use q_seam::observables::ExactObservables;
use std::io::Write;

fn main() {
    let mut configs: Vec<Configuration> = Vec::new();
    let mut voids: Vec<(usize, f64, String)> = Vec::new();
    let mut lambda_zero = std::collections::BTreeMap::new();

    for &n in &q_seam::SWEEP_SITES {
        let c0 = Chart::best(n, 1.0, 0.0).expect("U=0 chart must converge");
        lambda_zero.insert(n, Stability::of(&c0).lambda_min);
    }

    for &n in &q_seam::SWEEP_SITES {
        for &u in &q_seam::SWEEP_U {
            let h = Hubbard::new(n, 1.0, u);
            let g = match ground_state(&h) {
                Some(g) => g,
                None => {
                    voids.push((n, u, "lanczos residual gate".into()));
                    continue;
                }
            };
            let gap = if h.dim() <= 400 {
                let e = jacobi(h.to_dense(), h.dim());
                e.values[1] - e.values[0]
            } else {
                g.first_excited - g.energy
            };
            if gap < 1e-6 {
                voids.push((n, u, format!("in-sector gap {gap:e}")));
                continue;
            }
            let o = ExactObservables::measure(&h, &g.vector);
            let chart = match Chart::best(n, 1.0, u) {
                Some(c) => c,
                None => {
                    voids.push((n, u, "no SCF guess converged".into()));
                    continue;
                }
            };
            if chart.idempotency > 1e-12 || chart.energy < g.energy - 1e-10 {
                voids.push((n, u, "chart gate G-C2/G-C3".into()));
                continue;
            }
            configs.push(Configuration::assemble(
                n, u, &o, g.energy, g.residual, gap, &chart,
            ));
            eprintln!("done N={n} U={u}");
        }
    }

    println!("VOID configurations: {} of 70", voids.len());
    for (n, u, why) in &voids {
        println!("  VOID N={n} U={u}: {why}");
    }

    let honest6 = configs.iter().filter(|c| c.honest6()).count();
    let honest5 = configs.iter().filter(|c| c.honest5()).count();
    println!("\nchart-honest: {honest6} of {} (six-observable, Q5)", configs.len());
    println!("chart-honest: {honest5} of {} (five-observable, Q6 per A1/H2)", configs.len());

    println!("\n{:>3} {:>6} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>8} {:>6}",
        "N", "U", "e", "d", "n_i", "m_i", "bond", "D_bool", "E_tot", "honest");
    for c in &configs {
        println!("{:>3} {:>6} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>8.3} {:>6}",
            c.sites, c.u, c.normalized[0], c.normalized[1], c.normalized[2],
            c.normalized[3], c.normalized[4], c.normalized[5], c.e_tot,
            if c.honest6() { "yes" } else { "no" });
    }

    println!("\n=== JOINT GATE ===");
    let mut scores = Vec::new();
    for crit in [Criterion::C1, Criterion::C2, Criterion::C3, Criterion::C4,
                 Criterion::M1, Criterion::M2] {
        let s = score(crit.label(), &configs, |c| {
            crit.certifies(c, lambda_zero[&c.sites])
        });
        println!("{:<26} {}", crit.label(), s.clause_report());
        println!("{:<26}   boundary by N {:?}", "", s.certified_boundary);
        for (n, u, obs, e) in &s.false_positives {
            println!("{:<26}   FP N={n} U={u} worst=O{} E_tot={e:.2}", "", obs + 1);
        }
        scores.push((crit, s));
    }

    println!("\n=== SEVERITY BASELINES (post-hoc optimal, pin A1/P3) ===");
    match best_fixed_cutoff(&configs) {
        Some((ustar, s)) => println!("M3 best fixed cutoff U<={ustar}: {}", s.clause_report()),
        None => println!("M3 INFEASIBLE: no fixed U cutoff achieves FP=0 with the plant refused"),
    }
    match best_scaling_cutoff(&configs) {
        Some((a, b, s)) => println!("M4 best U<={a}+{b}/(N+1): {}", s.clause_report()),
        None => println!("M4 INFEASIBLE: no scaling cutoff achieves FP=0 with the plant refused"),
    }

    // ROBUSTNESS CLAUSE (team-lead ruling 2, clause 1). Under the FROZEN G-E4b (raw dense
    // eigenvalue vs Lanczos <= 1e-14) the gate fires at N=6 and only at N=6 — measured over all
    // 14 U: worst raw disagreement 1.221e-15 (N=2), 4.885e-15 (N=4), 1.155e-13 (N=6). N=8 and
    // N=10 carry no dense cross-check, so G-E4b does not reach them either way. The frozen
    // reading therefore VOIDs the 14 N=6 configurations. If the Q5 adjudication differs between
    // the two readings, the kill is UNADJUDICATED and the amendment must not be what decides it.
    println!("\n=== ROBUSTNESS: the FROZEN G-E4b reading (N=6 VOID, 14 configurations) ===");
    let frozen: Vec<Configuration> = configs.iter().filter(|c| c.sites != 6).cloned().collect();
    println!("configurations under the frozen reading: {} (honest {})",
        frozen.len(), frozen.iter().filter(|c| c.honest6()).count());
    let mut frozen_pass = Vec::new();
    for crit in [Criterion::C1, Criterion::C2, Criterion::C3, Criterion::C4,
                 Criterion::M1, Criterion::M2] {
        let s = score(crit.label(), &frozen, |c| crit.certifies(c, lambda_zero[&c.sites]));
        println!("{:<26} {}", crit.label(), s.clause_report());
        if s.passes() { frozen_pass.push(crit.label()); }
    }
    let f3 = best_fixed_cutoff(&frozen);
    let f4 = best_scaling_cutoff(&frozen);
    match &f3 { Some((u, s)) => println!("M3 best fixed cutoff U<={u}: {}", s.clause_report()),
                None => println!("M3 INFEASIBLE") }
    match &f4 { Some((a, b, s)) => println!("M4 best U<={a}+{b}/(N+1): {}", s.clause_report()),
                None => println!("M4 INFEASIBLE") }
    let frozen_kill = frozen_pass.is_empty();
    let full_kill = false; // C4 passes on the full set; recomputed below for the record.
    let full_pass: Vec<&str> = [Criterion::C1, Criterion::C2, Criterion::C3, Criterion::C4]
        .iter()
        .filter(|crit| score(crit.label(), &configs, |c| crit.certifies(c, lambda_zero[&c.sites])).passes())
        .map(|crit| crit.label())
        .collect();
    println!("\nfull reading   : criteria passing = {full_pass:?}; kill fires = {}", full_pass.is_empty());
    println!("frozen reading : criteria passing = {frozen_pass:?}; kill fires = {frozen_kill}");
    let _ = full_kill;
    if full_pass.is_empty() != frozen_kill {
        println!("ADJUDICATIONS DIFFER -> the Q5 kill is UNADJUDICATED.");
    } else {
        println!("ADJUDICATIONS AGREE -> the Q5 kill verdict does not depend on amendment A2.");
        // And the severity reading must agree too, or the headline changes.
        if let (Some((_, s3)), true) = (&f3, !frozen_pass.is_empty()) {
            let c4 = score("C4", &frozen, |c| Criterion::C4.certifies(c, lambda_zero[&c.sites]));
            println!("severity under the frozen reading: C4 cov {:.3} vs M3 cov {:.3} -> {}",
                c4.coverage, s3.coverage,
                if s3.coverage >= c4.coverage { "still CORRECT BUT UNINFORMATIVE" } else { "headline would differ" });
        }
    }

    // JSON by hand: zero runtime dependencies.
    let dir = "/home/emoore/CIRISOntology/sim_engine/output/q_seam";
    std::fs::create_dir_all(dir).unwrap();
    let mut f = std::fs::File::create(format!("{dir}/q5.json")).unwrap();
    writeln!(f, "{{\"configurations\":[").unwrap();
    for (k, c) in configs.iter().enumerate() {
        writeln!(f, "{}{{\"N\":{},\"U\":{},\"e_tot\":{},\"e5\":{},\"honest6\":{},\"honest5\":{},\
\"norm\":[{},{},{},{},{},{}],\"audit\":[{},{},{},{},{},{}],\"lambda_min\":{},\"lambda_proj\":{},\
\"null_modes\":{},\"break_spin\":{},\"d_bool\":{},\"residual\":{},\"gap\":{},\"E_exact\":{},\"E_chart\":{}}}",
            if k == 0 { "" } else { "," },
            c.sites, c.u, c.e_tot, c.e5, c.honest6(), c.honest5(),
            c.normalized[0], c.normalized[1], c.normalized[2], c.normalized[3], c.normalized[4], c.normalized[5],
            c.audit.as_vector()[0], c.audit.as_vector()[1], c.audit.as_vector()[2],
            c.audit.as_vector()[3], c.audit.as_vector()[4], c.audit.as_vector()[5],
            c.stability.lambda_min, c.stability.lambda_min_projected, c.stability.null_modes,
            c.breaking.0, c.d_bool_exact, c.residual, c.in_sector_gap, c.exact_energy, c.chart_energy).unwrap();
    }
    writeln!(f, "],\"voids\":{}}}", voids.len()).unwrap();
    println!("\nwrote {dir}/q5.json");
}
