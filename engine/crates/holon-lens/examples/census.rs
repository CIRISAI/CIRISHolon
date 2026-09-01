//! THE CENSUS RUNNER: point it at trajectories, get the verdict table.
//!
//! The table it prints is the one the campaign needs and the one `waterquench` cannot
//! print: FORMULA beside CLOSURE. Connected-component naming says what a component's
//! composition is at the final frame; the census says whether any set of atoms held
//! together across the staked window. Where the two disagree, the disagreement is the
//! result.
//!
//! Every threshold is `holon_lens::census`'s `PREREG_*` constants, which are
//! `conformance/water_observatory/CENSUS_PREREG.md`'s stakes, frozen before this ran.
//!
//! ```text
//! cargo run --release -p holon-lens --example census -- <dir-or-file> [...]
//! ```

use holon_lens::census::{self, BlockVerdict, Census, Stakes};
use holon_lens::classifier;
use holon_lens::lens;
use holon_lens::traj::Trajectory;
use std::path::PathBuf;

fn collect(args: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for a in args {
        let p = PathBuf::from(a);
        if p.is_dir() {
            let mut v: Vec<PathBuf> = std::fs::read_dir(&p)
                .expect("readable directory")
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().map(|e| e == "traj").unwrap_or(false))
                .collect();
            v.sort();
            out.extend(v);
        } else {
            out.push(p);
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let paths = collect(&args);
    if paths.is_empty() {
        eprintln!("usage: census <dir-or-file> [...]");
        std::process::exit(2);
    }
    let st = Stakes::default();
    println!("# THE CLOSURE CENSUS");
    println!(
        "# stakes (CENSUS_PREREG.md, frozen before this instrument existed):\n\
         #   window        W = {} fs\n\
         #   budget        beta = {}, breach run <= {} fs\n\
         #   moving carrier rms >= {} bohr, separation excursion >= {} bohr\n\
         #   Leg B work count >= {} informative transitions\n\
         #   control floor  <= {} of the same-composition pool",
        st.window_fs,
        st.beta,
        st.flicker_fs,
        st.min_rms_bohr,
        st.min_sep_var_bohr,
        st.min_informative,
        st.control_max_rate
    );

    let mut totals = (0usize, 0usize, 0usize, 0usize); // strict, budgeted, transient, void
    for p in &paths {
        let traj = match Trajectory::read(p) {
            Ok(t) => t,
            Err(e) => {
                println!("\n# {}: UNREADABLE ({e})", p.display());
                continue;
            }
        };
        println!("\n{}", "=".repeat(78));
        println!(
            "# {}\n# seed {:#018x}  {} atoms  {} frames  dt-derived frame = {:.4} fs  complete: {}",
            p.display(),
            traj.header.seed,
            traj.header.n_atoms,
            traj.frames.len(),
            traj.header.frame_fs(),
            traj.is_complete()
        );

        let rep = match census::run(&traj, &st) {
            Census::Refused { gate, reason } => {
                println!("# REFUSED at gate [{gate}]\n#   {reason}");
                continue;
            }
            Census::Report(r) => r,
        };

        println!(
            "# window = {} frames, flicker cap = {} frames",
            rep.window_frames, rep.flicker_frames
        );

        // ---- what connected-component naming reports -------------------------------
        println!(
            "#\n# FORMULA READER (what waterquench prints, final frame): {}",
            if rep.final_frame_molecules.is_empty() {
                "-".to_string()
            } else {
                rep.final_frame_molecules
                    .iter()
                    .map(|(_, f)| f.clone())
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        );

        // ---- the closure reader -----------------------------------------------------
        println!("#\n# CLOSURE READER (blocks by longest held run):");
        println!(
            "#   {:<8} {:<6} {:>9} {:>10} {:>8} {:>8} {:>7}  {:<22} {}",
            "formula", "block", "held run", "held (fs)", "rms", "sep var", "ctrl", "verdict", "named?"
        );
        let mut shown = 0;
        for b in &rep.blocks {
            match b.verdict {
                BlockVerdict::CertifiedStrict { .. } => totals.0 += 1,
                BlockVerdict::CertifiedBudgeted { .. } => totals.1 += 1,
                BlockVerdict::Transient { .. } => totals.2 += 1,
                _ => totals.3 += 1,
            }
            // Print every certified or final-frame-named row, plus the ten longest.
            if !(b.verdict.is_certified() || b.named_at_final_frame || shown < 10) {
                continue;
            }
            shown += 1;
            println!(
                "#   {:<8} {:#06x} {:>9} {:>10.1} {:>8.3} {:>8.3} {:>7}  {:<22} {}",
                b.formula,
                b.block,
                b.longest_run,
                b.longest_run_fs,
                b.rms_internal,
                b.max_sep_variation,
                b.control_rate
                    .map(|r| format!("{:.3}", r))
                    .unwrap_or_else(|| "-".into()),
                b.verdict.tag(),
                if b.named_at_final_frame { "YES" } else { "" }
            );
        }
        println!("#   ({} blocks in all; rows above are the certified, the named, and the ten longest)", rep.blocks.len());

        // ---- Leg B ------------------------------------------------------------------
        let c = &rep.closure;
        println!(
            "#\n# LEG B, the fiber-invariance test on the full partition view:\n\
             #   distinct partition readings   : {}\n\
             #   transitions                   : {} total, {} INFORMATIVE (work count; staked >= {})\n\
             #   witness pairs                 : {}\n\
             #   defect                        : {:.6}   (first half {:.6}, second half {:.6})\n\
             #   non-expansion (<= {:.2}x)       : {}\n\
             #   VERDICT                       : {}",
            c.distinct_readings,
            c.total_transitions,
            c.informative_transitions,
            st.min_informative,
            c.witness_pair_count,
            c.defect,
            c.defect_first_half,
            c.defect_second_half,
            st.nonexpansion,
            if c.nonexpansion_ok { "ok" } else { "BREACHED" },
            if c.void {
                "VOID (work count below the stake)".to_string()
            } else if c.distinct_readings <= 1 {
                "VACUOUS: one partition reading for the whole run, closed by h = id \
                 (M-FIXED-POINT-TRAJECTORY)"
                    .to_string()
            } else if c.witness_pair_count == 0 {
                "no witness pair found at this resolution (NOT a proof of closure)".to_string()
            } else {
                format!("NOT CLOSED: {} witness pairs exhibited", c.witness_pair_count)
            }
        );
        for (a, b) in c.witness_pairs.iter().take(5) {
            println!(
                "#     witness pair: frames {a} and {b} read the same partition, frames {} and {} do not",
                a + 1,
                b + 1
            );
        }

        // ---- the lens stack ---------------------------------------------------------
        println!("#\n# LENS STACK on the final frame (refusals are readings too):");
        let f = traj.frames.last().unwrap();
        let d = traj.header.dims;
        let nb4: Vec<[f64; 3]> = lens::k_nearest(&f.pos, 0, 4).iter().map(|&j| f.pos[j]).collect();
        report_lens("q-tetrahedral", lens::q_tetrahedral(d, f.pos[0], &nb4));
        let nb6: Vec<[f64; 3]> = lens::k_nearest(&f.pos, 0, 6).iter().map(|&j| f.pos[j]).collect();
        report_lens("steinhardt q6", lens::steinhardt_q(6, d, f.pos[0], &nb6));
        report_lens("hexatic psi6", lens::hexatic_psi6(d, f.pos[0], &nb6));
        let max_lag = (traj.frames.len() / 10).max(2);
        report_lens("diffusion", lens::diffusion(&traj, max_lag));
        match lens::hbonds(&f.pos, &traj.header.z) {
            Ok(v) => println!("#   H-bond census (Luzar-Chandler)   : {} bonds", v.len()),
            Err(e) => println!("#   H-bond census (Luzar-Chandler)   : REFUSED [{}] {}", e.gate, e.reason),
        }
        let edges: Vec<(usize, usize)> = {
            let n = traj.header.n_atoms;
            let mut v = Vec::new();
            for i in 0..n {
                for j in (i + 1)..n {
                    if f.is_bonded(n, i, j) {
                        v.push((i, j));
                    }
                }
            }
            v
        };
        println!(
            "#   largest domain (bonded-pair graph): {} atoms over {} edges",
            lens::largest_domain(traj.header.n_atoms, &edges),
            edges.len()
        );

        // ---- the closure-defect lens, applied to each MACRO VIEW --------------------
        //
        // Leg B above asks whether the full partition view carries its own dynamics. This
        // asks the same question of the coarser readings a report would actually quote:
        // is "the largest domain is 8 atoms" a quantity that predicts its own next value,
        // or does it need the micro state it threw away? Each view is binned, and the bin
        // count is printed with the defect because a view is only as closed as its
        // resolution (M-FINAL-VIEW-COLLISIONS: a refined view has its OWN collisions, and
        // they have to be counted rather than assumed away).
        println!("#\n# CLOSURE DEFECT BY MACRO VIEW (binned; a view is only as closed as its resolution):");
        println!(
            "#   {:<26} {:>5} {:>8} {:>13} {:>10}  {}",
            "view", "bins", "defect", "informative", "witnesses", "reading"
        );
        let n = traj.header.n_atoms;
        let edges_of = |f: &holon_lens::traj::Frame| -> Vec<(usize, usize)> {
            let mut v = Vec::new();
            for i in 0..n {
                for j in (i + 1)..n {
                    if f.is_bonded(n, i, j) {
                        v.push((i, j));
                    }
                }
            }
            v
        };
        let largest: Vec<f64> = traj
            .frames
            .iter()
            .map(|f| lens::largest_domain(n, &edges_of(f)) as f64)
            .collect();
        let bondcount: Vec<f64> = traj
            .frames
            .iter()
            .map(|f| f.bonded.count_ones() as f64)
            .collect();
        let hb: Vec<f64> = traj
            .frames
            .iter()
            .map(|f| lens::hbonds(&f.pos, &traj.header.z).map(|v| v.len() as f64).unwrap_or(0.0))
            .collect();
        let stride = (traj.frames.len() / 4000).max(1);
        let psi: Vec<f64> = traj
            .frames
            .iter()
            .step_by(stride)
            .map(|f| {
                let mut acc = 0.0;
                let mut c = 0usize;
                for i in 0..n {
                    let nb: Vec<[f64; 3]> =
                        lens::k_nearest(&f.pos, i, 6).iter().map(|&j| f.pos[j]).collect();
                    if let Ok(v) = lens::hexatic_psi6(traj.header.dims, f.pos[i], &nb) {
                        acc += v;
                        c += 1;
                    }
                }
                if c == 0 { 0.0 } else { acc / c as f64 }
            })
            .collect();

        let views: [(&str, &Vec<f64>, usize, f64, f64); 4] = [
            ("largest domain (atoms)", &largest, n + 1, 0.0, (n + 1) as f64),
            ("bonded pair count", &bondcount, 67, 0.0, 67.0),
            ("H-bond count", &hb, 20, 0.0, 20.0),
            ("mean hexatic psi6", &psi, 10, 0.0, 1.0),
        ];
        for (name, v, bins, lo, hi) in views {
            if v.is_empty() {
                println!("#   {name:<26} {:>5} {:>8} {:>13} {:>10}  no samples", bins, "-", 0, "-");
                continue;
            }
            let leg = lens::binned_closure_defect(v, bins, lo, hi, &st);
            println!(
                "#   {name:<26} {:>5} {:>8.5} {:>13} {:>10}  {}",
                bins,
                leg.defect,
                leg.informative_transitions,
                leg.witness_pair_count,
                if leg.void {
                    "VOID (work count below stake)"
                } else if leg.distinct_readings <= 1 {
                    // A view with ONE reading is closed by h = id and has said nothing.
                    // M-FIXED-POINT-TRAJECTORY: a closure gate on a carrier that never
                    // moves is vacuous, and a constant macro view is exactly that.
                    "VACUOUS (one reading; the view never moved)"
                } else if leg.witness_pair_count == 0 {
                    "no witness pair at this resolution"
                } else {
                    "NOT CLOSED"
                }
            );
        }

        // ---- the blind classifier ---------------------------------------------------
        let cl = classifier::classify(&traj);
        println!(
            "#\n# BLIND CLASSIFIER: {}\n\
             #   free fraction {:.3}  order {:.3}  mobility {:.4}  interior atoms {} ({} samples)",
            match &cl.verdict {
                classifier::Verdict::Phase(p) => format!("{p:?}"),
                classifier::Verdict::Refused { gate, reason } =>
                    format!("REFUSED [{gate}]\n#   {reason}"),
            },
            cl.free_fraction,
            cl.order,
            cl.mobility,
            cl.interior_atoms,
            cl.interior_samples
        );
    }

    println!("\n{}", "=".repeat(78));
    println!(
        "# TOTALS over {} trajectories: {} certified-strict, {} certified-budgeted, \
         {} transient, {} void",
        paths.len(),
        totals.0,
        totals.1,
        totals.2,
        totals.3
    );
}

fn report_lens(name: &str, r: lens::Reading<f64>) {
    match r {
        Ok(v) => println!("#   {name:<33}: {v:.4}"),
        Err(e) => println!("#   {name:<33}: REFUSED [{}] {}", e.gate, e.reason),
    }
}
