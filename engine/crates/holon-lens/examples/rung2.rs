//! RUNG 2 — the fluid-element chart, run against the certified arms.
//!
//! Stakes: `conformance/water_observatory/RUNG2_PREREG.md`, frozen at `aee5317` before
//! this file existed. Every threshold printed below is one of its constants.
//!
//! ```text
//! cargo run --release -p holon-lens --example rung2 -- <traj-dir> [arm ...] [--amend1]
//! ```
//!
//! `--amend1` runs `RUNG2_AMENDMENT_1.md`'s chart INSTEAD of the frozen one: the grid ladder
//! derived from the carrier (A3), cells with a third axis (A2), and the density field
//! read BOTH ways on every grid — the freeze's exact occupancy and the amendment's
//! Poisson bin — so the two verdicts sit side by side on identical frames. Without the
//! flag this file is the freeze's driver, unchanged.
//!
//! G1 (digest identity) is NOT performed here: this crate has zero dependencies and
//! therefore no sha256. The digests are verified by `sha256sum -c` against
//! `census_traj_manifest.sha256` before this runner is invoked, and the results document
//! cites that check's output. A pin names what it measured and nothing beside it.

use holon_lens::field::*;
use holon_lens::traj::Trajectory;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let amend1 = args.iter().any(|a| a == "--amend1");
    let amend4 = args.iter().any(|a| a == "--amend4");
    let amend5 = args.iter().any(|a| a == "--amend5");
    let amend6 = args.iter().any(|a| a == "--amend6");
    let response = args.iter().position(|a| a == "--response").map(|i| args.get(i + 1).cloned().unwrap_or_default());
    let response_cycles: usize = args.iter().position(|a| a == "--cycles").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(12);
    let response_relax: usize = args.iter().position(|a| a == "--relax").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(314);
    if let Some(i) = args.iter().position(|a| a == "--response") { args.drain(i..(i + 2).min(args.len())); }
    if let Some(i) = args.iter().position(|a| a == "--cycles") { args.drain(i..(i + 2).min(args.len())); }
    if let Some(i) = args.iter().position(|a| a == "--relax") { args.drain(i..(i + 2).min(args.len())); }
    args.retain(|a| a != "--amend1" && a != "--amend4" && a != "--amend5" && a != "--amend6");
    if args.is_empty() {
        eprintln!("usage: rung2 <traj-dir> [arm ...]");
        std::process::exit(2);
    }
    let root = PathBuf::from(&args[0]);
    let arms: Vec<String> = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        vec!["fenced".into(), "hydrogen".into()]
    };

    println!("# RUNG 2 — the fluid-element chart");
    println!("# stakes: RUNG2_PREREG.md @ aee5317");
    println!("# dp = {:.6} au (frozen {DP_AU_FROZEN})", dp_au());
    println!("# de = {:.9} Ha (frozen {DE_HA_FROZEN})", de_ha());
    println!(
        "# bars: admissible occ >= {}, cells >= {}, fluct <= {}; transport >= {}; \
         informative >= {}; beta = {}; separation >= {}",
        prereg::ADMISSIBLE_OCCUPANCY,
        prereg::ADMISSIBLE_CELLS,
        prereg::ADMISSIBLE_FLUCTUATION,
        prereg::MIN_TRANSPORT,
        prereg::MIN_INFORMATIVE,
        prereg::BETA,
        prereg::MIN_SEPARATION
    );

    // The cost model of PREREG G11, counted rather than timed.
    let mut frames_read: u64 = 0;
    let mut chart_evals: u64 = 0;
    if let Some(arm) = response {
        response1_read(&root, &arms, &arm, response_cycles, response_relax, &mut frames_read);
        println!("\n===== COST (PREREG G11, work units, never wall clock) =====");
        println!("frames read:       {frames_read}");
        return;
    }
    if amend6 {
        amendment6_read(&root, &arms, &mut frames_read);
        println!("\n===== COST (PREREG G11, work units, never wall clock) =====");
        println!("frames read:       {frames_read}");
        return;
    }
    if amend5 {
        amendment5_read(&root, &arms, &mut frames_read, &mut chart_evals);
        println!("\n===== COST (PREREG G11, work units, never wall clock) =====");
        println!("frames read:       {frames_read}");
        println!("chart evaluations: {chart_evals}");
        return;
    }
    if amend4 {
        amendment4_read(&root, &arms, &mut frames_read, &mut chart_evals);
        println!("\n===== COST (PREREG G11, work units, never wall clock) =====");
        println!("frames read:       {frames_read}");
        println!("chart evaluations: {chart_evals}");
        return;
    }

    for arm in &arms {
        let dir = root.join(arm);
        let mut files: Vec<PathBuf> = match std::fs::read_dir(&dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().map(|e| e == "traj").unwrap_or(false))
                .collect(),
            Err(e) => {
                println!("ARM {arm}: REFUSED — {e}");
                continue;
            }
        };
        files.sort();
        println!("\n===== ARM {arm} ({} trajectories) =====", files.len());

        for path in &files {
            let traj = match Trajectory::read(path) {
                Ok(t) => t,
                Err(e) => {
                    println!("  {} REFUSED — {e}", path.display());
                    continue;
                }
            };
            frames_read += traj.frames.len() as u64;
            let h = &traj.header;
            let mut sp: Vec<u32> = h.z.clone();
            sp.sort_unstable();
            sp.dedup();
            println!(
                "\n-- seed 0x{:016x}  n={} dims={} frames={} box={:.1}x{:.1} species={:?}",
                h.seed,
                h.n_atoms,
                h.dims,
                traj.frames.len(),
                h.box_w,
                h.box_h,
                sp
            );

            // G9a — the one field whose conservation the dynamics actually constrains.
            println!("   G9a species/arity constant: {}", species_conserved(&traj));
            // G9b/G9c — reported as drift, because the ledger legs are not computable
            // from this artifact (no forces, no intervention ledger in the dump).
            match drift(&traj) {
                Ok(d) => {
                    let p0 = (d.p_first[0] * d.p_first[0] + d.p_first[1] * d.p_first[1]).sqrt();
                    let p1 = (d.p_last[0] * d.p_last[0] + d.p_last[1] * d.p_last[1]).sqrt();
                    println!(
                        "   G9b |p| first={p0:.6e} last={p1:.6e}   \
                         G9c Ek first={:.6e} last={:.6e}  [ledger leg UNDISCHARGED: \
                         the dump carries no forces]",
                        d.ek_first, d.ek_last
                    );
                }
                Err(e) => println!("   drift REFUSED — {e:?}"),
            }

            if amend1 {
                amended_read(&traj, &mut chart_evals);
                continue;
            }
            for grid in FROZEN_GRIDS {
                let cs = match cell_series(&traj, grid, Kind::Spatial) {
                    Ok(c) => c,
                    Err(e) => {
                        println!("   grid {}x{}: REFUSED — {e:?}", grid.nx, grid.ny);
                        continue;
                    }
                };
                let (mean_occ, fluct) = occupancy_stats(&cs, grid.cells());
                let transport = transport_fraction(&cs);
                // G2 — admissibility. Expected to fail; printed regardless, because a bar
                // quoted without its measured value is not a bar.
                let g2 = mean_occ >= prereg::ADMISSIBLE_OCCUPANCY
                    && grid.cells() >= prereg::ADMISSIBLE_CELLS
                    && fluct <= prereg::ADMISSIBLE_FLUCTUATION;
                println!(
                    "   grid {}x{} cells={} occ={:.3} fluct={:.3} transport={:.4}  \
                     G2 admissible: {}",
                    grid.nx,
                    grid.ny,
                    grid.cells(),
                    mean_occ,
                    fluct,
                    transport,
                    if g2 { "YES" } else { "NO" }
                );

                for kind in [
                    Kind::Spatial,
                    Kind::BlindLabel,
                    Kind::BlindIndex,
                    Kind::GlobalRelabel,
                ] {
                    let cs_k = match cell_series(&traj, grid, kind) {
                        Ok(c) => c,
                        Err(e) => {
                            println!("      {kind:?}: REFUSED — {e:?}");
                            continue;
                        }
                    };
                    let tr_k = transport_fraction(&cs_k);
                    let mut counts: Vec<u128> = Vec::new();
                    let mut prev: Option<Vec<Reading>> = None;
                    for rung in LADDER {
                        let r = match readings(&traj, grid, rung, kind) {
                            Ok(r) => r,
                            Err(e) => {
                                println!("      {kind:?} {rung:?}: REFUSED — {e:?}");
                                continue;
                            }
                        };
                        chart_evals += r.len() as u64;
                        let a = leg_a(&r);
                        let b = leg_b(&r);
                        counts.push(a.collisions);
                        // G8 strong form, checked pairwise up the ladder.
                        let refines_ok = match &prev {
                            None => true,
                            Some(p) => refines(&r, p),
                        };
                        let v = grade(grid.cells() >= prereg::MIN_CELLS, tr_k, &a);
                        let d = a
                            .defect()
                            .map(|x| format!("{x:.6}"))
                            .unwrap_or_else(|| "n/a".into());
                        let db = b
                            .defect()
                            .map(|x| format!("{x:.6}"))
                            .unwrap_or_else(|| "n/a".into());
                        println!(
                            "      {kind:?} {rung:?} coll={} fire={} D_A={d} info={} \
                             distinct={} | D_B={db} cov={:.3} | refines={} | {v:?}",
                            a.collisions,
                            a.firing,
                            a.informative,
                            a.distinct,
                            b.coverage(),
                            refines_ok
                        );
                        if !a.witnesses.is_empty() {
                            let w: Vec<String> = a
                                .witnesses
                                .iter()
                                .take(3)
                                .map(|(i, j)| format!("({i},{j})"))
                                .collect();
                            println!(
                                "         witnesses (listing capped at {WITNESS_CAP}, \
                                 count above is exact): {}",
                                w.join(" ")
                            );
                        }
                        prev = Some(r);
                    }
                    println!(
                        "      {kind:?} G8 weak (monotone counts): {}",
                        ladder_monotone(&counts)
                    );
                }
            }
        }
    }

    if amend1 {
        println!("\n# RUNG2_AMENDMENT_1/2/3.md were in force for every grid above: derived ladder, 3D cells, every field read Exact (the freeze), Poisson (A1), CellScale (A2, superseded, kept as the control) and Derived (A3) side by side");
    }
    println!("\n===== COST (PREREG G11, work units, never wall clock) =====");
    println!("frames read:       {frames_read}");
    println!("chart evaluations: {chart_evals}");
    let _ = HashSet::<u8>::new();
}


/// RUNG2_AMENDMENT_1.md's reading of one trajectory. Every bar is the freeze's; what is
/// amended is the grid ladder, the cell's third axis, and that the density field is read
/// both ways on every grid. The exact line is the control, the Poisson line is the reading,
/// and both are graded by the SAME `grade`.
fn amended_read(traj: &Trajectory, chart_evals: &mut u64) {
    let h = &traj.header;
    let ladder = doubling_ladder(h.n_atoms, h.dims);
    println!(
        "   amendment 1: ladder of {} grids derived from N={} dims={}: {}",
        ladder.len(),
        h.n_atoms,
        h.dims,
        ladder.iter().map(|g| format!("{}x{}x{}", g.nx, g.ny, g.nz)).collect::<Vec<_>>().join(" ")
    );
    for grid in ladder {
        let cs = match cell_series3(traj, grid, Kind::Spatial) {
            Ok(c) => c,
            Err(e) => {
                println!("   grid {}x{}x{}: REFUSED — {e:?}", grid.nx, grid.ny, grid.nz);
                continue;
            }
        };
        let (mean_occ, fluct) = occupancy_stats(&cs, grid.cells());
        let transport = transport_fraction(&cs);
        let g2 = mean_occ >= prereg::ADMISSIBLE_OCCUPANCY
            && grid.cells() >= prereg::ADMISSIBLE_CELLS
            && fluct <= prereg::ADMISSIBLE_FLUCTUATION;
        let dn = ((h.n_atoms as f64) / (grid.cells() as f64)).sqrt().max(1.0);
        let m_bar = h.z.iter().map(|z| mass_me(*z).unwrap_or(0.0)).sum::<f64>() / h.n_atoms as f64;
        let (kp, ke) = derived_multiples(h.n_atoms as f64 / grid.cells() as f64, h.n_atoms as f64, m_bar);
        println!(
            "   grid {}x{}x{} cells={} occ={:.3} fluct={:.3} transport={:.4} dn={:.2} derived: dp={}x{:.4} de={}x{:.3e}  G2 admissible: {}",
            grid.nx, grid.ny, grid.nz, grid.cells(), mean_occ, fluct, transport, dn, kp, dp_au(), ke, de_ha(),
            if g2 { "YES" } else { "NO" }
        );
        for kind in [Kind::Spatial, Kind::BlindLabel, Kind::BlindIndex, Kind::GlobalRelabel] {
            let tr_k = match cell_series3(traj, grid, kind) {
                Ok(c) => transport_fraction(&c),
                Err(e) => {
                    println!("      {kind:?}: REFUSED — {e:?}");
                    continue;
                }
            };
            for density in [Density::Exact, Density::Poisson, Density::CellScale, Density::Derived] {
                let mut prev: Option<Vec<Reading>> = None;
                for rung in LADDER {
                    let r = match readings3(traj, grid, rung, kind, density) {
                        Ok(r) => r,
                        Err(e) => {
                            println!("      {kind:?} {density:?} {rung:?}: REFUSED — {e:?}");
                            continue;
                        }
                    };
                    *chart_evals += r.len() as u64;
                    let a = leg_a(&r);
                    let b = leg_b(&r);
                    let refines_ok = match &prev { None => true, Some(p) => refines(&r, p) };
                    let v = grade(grid.cells() >= prereg::MIN_CELLS, tr_k, &a);
                    let d = a.defect().map(|x| format!("{x:.6}")).unwrap_or_else(|| "n/a".into());
                    let db = b.defect().map(|x| format!("{x:.6}")).unwrap_or_else(|| "n/a".into());
                    println!(
                        "      {kind:?} {density:?} {rung:?} coll={} fire={} D_A={d} info={} distinct={} | D_B={db} cov={:.3} | refines={} | {v:?}",
                        a.collisions, a.firing, a.informative, a.distinct, b.coverage(), refines_ok
                    );
                    if density == Density::Poisson && rung == Rung::Occ && !a.witnesses.is_empty() {
                        let w: Vec<String> = a.witnesses.iter().take(3).map(|(i, j)| format!("({i},{j})")).collect();
                        println!("         witnesses (listing capped at {WITNESS_CAP}, count above is exact): {}", w.join(" "));
                    }
                    prev = Some(r);
                }
            }
            // PA-5 in the field: the exact chart must refine its own bin on this trajectory.
            if let (Ok(e), Ok(b)) = (
                readings3(traj, grid, Rung::Occ, kind, Density::Exact),
                readings3(traj, grid, Rung::Occ, kind, Density::Poisson),
            ) {
                if !refines(&e, &b) {
                    println!("      {kind:?} SELF-CHECK FIRED: exact occupancy does not refine its Poisson bin — the instrument is convicted on this grid");
                }
            }
        }
    }
}


/// RUNG2_AMENDMENT_4.md: the density bin for a liquid, two declared routes read side by side
/// on identical frames. Route (i) `External` uses water's `S(0)`; route (ii) `Calibrated`
/// uses `σ(n)` measured on the FLEXIBLE arms of the seeds NOT being graded, at the same grid,
/// pooled in quadrature. A calibration with no held-out seed is REFUSED (plant PD-4).
/// Momentum and energy are Amendment 3's under both routes.
fn amendment4_read(root: &Path, arms: &[String], frames_read: &mut u64, chart_evals: &mut u64) {
    let mut files: Vec<(String, PathBuf, Trajectory)> = Vec::new();
    for arm in arms {
        let dir = root.join(arm);
        let mut paths: Vec<PathBuf> = match std::fs::read_dir(&dir) {
            Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|e| e == "traj").unwrap_or(false)).collect(),
            Err(e) => { println!("ARM {arm}: REFUSED — {e}"); continue; }
        };
        paths.sort();
        for p in paths {
            match Trajectory::read(&p) {
                Ok(t) => { *frames_read += t.frames.len() as u64; files.push((arm.clone(), p, t)); }
                Err(e) => println!("  {} REFUSED — {e}", p.display()),
            }
        }
    }
    let n_atoms = files.first().map(|f| f.2.header.n_atoms).unwrap_or(0);
    let dims = files.first().map(|f| f.2.header.dims).unwrap_or(3);
    let ladder = doubling_ladder(n_atoms, dims);
    // the calibration pool: σ(n) per (seed, grid) on the flexible arms
    // per (arm, seed, grid): a file is calibrated on the OTHER seeds of its OWN arm
    let mut sigma: Vec<(String, u64, Grid3, f64)> = Vec::new();
    for (arm, _, t) in &files {
        for grid in &ladder {
            if let Ok(cs) = cell_series3(t, *grid, Kind::Spatial) {
                sigma.push((arm.clone(), t.header.seed, *grid, occupancy_sigma(&cs, grid.cells())));
            }
        }
    }
    println!("# RUNG2_AMENDMENT_4: density bin for a liquid — route (i) External S(0)={WATER_S0} vs route (ii) Calibrated on held-out seeds; momentum/energy per Amendment 3");
    println!("# calibration pool (per arm, other seeds): {}", sigma.iter().filter(|s| s.2.cells() == 2).map(|s| format!("{} seed …{:x} 2-cell sigma(n)={:.2}", s.0, s.1 & 0xff, s.3)).collect::<Vec<_>>().join("; "));
    for (arm, path, traj) in &files {
        let h = &traj.header;
        println!("\n===== ARM {arm} — {} =====", path.file_name().unwrap().to_string_lossy());
        println!("-- seed 0x{:016x}  n={} dims={} frames={}", h.seed, h.n_atoms, h.dims, traj.frames.len());
        for grid in &ladder {
            // PD-4: the calibration comes from OTHER seeds only, and there must be at least one
            let held_out: Vec<f64> = sigma.iter().filter(|s| s.0 == *arm && s.1 != h.seed && s.2 == *grid).map(|s| s.3).collect();
            if held_out.is_empty() {
                println!("   grid {}x{}x{}: route (ii) REFUSED — no held-out seed to calibrate on", grid.nx, grid.ny, grid.nz);
                continue;
            }
            let cal = (held_out.iter().map(|x| x * x).sum::<f64>() / held_out.len() as f64).sqrt().max(1.0);
            let cs = match cell_series3(traj, *grid, Kind::Spatial) { Ok(c) => c, Err(e) => { println!("   grid: REFUSED — {e:?}"); continue; } };
            let (mean_occ, fluct) = occupancy_stats(&cs, grid.cells());
            let n_bar = h.n_atoms as f64 / grid.cells() as f64;
            let dn_ext = (WATER_S0 * n_bar * (1.0 - n_bar / h.n_atoms as f64)).sqrt().max(1.0);
            let own = occupancy_sigma(&cs, grid.cells());
            println!(
                "   grid {}x{}x{} cells={} occ={:.2} fluct={:.3} | own sigma(n)={:.2} (NOT used) | dn(i) External={:.2}  dn(ii) Calibrated={:.2} from {} held-out seed(s) | ratio (ii)/(i)={:.2}",
                grid.nx, grid.ny, grid.nz, grid.cells(), mean_occ, fluct, own, dn_ext, cal, held_out.len(), cal / dn_ext
            );
            for (label, density) in [("(i) External", Density::External), ("(ii) Calibrated", Density::Calibrated(cal))] {
                let mut occ_defects: Vec<(Kind, Option<f64>)> = Vec::new();
                for kind in [Kind::Spatial, Kind::BlindLabel] {
                    let tr_k = match cell_series3(traj, *grid, kind) { Ok(c) => transport_fraction(&c), Err(_) => 0.0 };
                    for rung in [Rung::Occ, Rung::Mom] {
                        let r = match readings3(traj, *grid, rung, kind, density) { Ok(r) => r, Err(e) => { println!("      {label} {kind:?} {rung:?}: REFUSED — {e:?}"); continue; } };
                        *chart_evals += r.len() as u64;
                        let a = leg_a(&r);
                        let v = grade(grid.cells() >= prereg::MIN_CELLS, tr_k, &a);
                        let d = a.defect().map(|x| format!("{x:.4}")).unwrap_or_else(|| "n/a".into());
                        println!("      {label:15} {kind:?} {rung:?} coll={} fire={} D_A={d} info={} distinct={} | {v:?}", a.collisions, a.firing, a.informative, a.distinct);
                        if rung == Rung::Occ { occ_defects.push((kind, a.defect())); }
                    }
                }
                let sp = occ_defects.iter().find(|k| k.0 == Kind::Spatial).and_then(|k| k.1);
                let bl = occ_defects.iter().find(|k| k.0 == Kind::BlindLabel).and_then(|k| k.1);
                match (sp, bl) {
                    (Some(ds), Some(db)) => println!("      {label:15} G7 at Occ: blind − spatial = {:+.4} (needs ≥ +0.05) → {}", db - ds, if db - ds >= prereg::MIN_SEPARATION { "SEPARATED" } else { "NO SEPARATION (branch e)" }),
                    _ => println!("      {label:15} G7 at Occ: undecidable (a chart has no collisions)"),
                }
            }
        }
    }
}


/// RUNG2_AMENDMENT_5.md: the chart read at the cell's own cadence `τ = a / c_s`, every
/// field the average over the window, every bin the held-out σ of the AVERAGED field on
/// other seeds' flexible arms at the same grid and window (A5.3). The instantaneous chart
/// under Amendment 4 route (ii) is printed beside it as the control. Non-overlapping
/// windows; a trailing partial window is dropped.
fn amendment5_read(root: &Path, arms: &[String], frames_read: &mut u64, chart_evals: &mut u64) {
    let mut files: Vec<(String, PathBuf, Trajectory)> = Vec::new();
    for arm in arms {
        let dir = root.join(arm);
        let mut paths: Vec<PathBuf> = match std::fs::read_dir(&dir) {
            Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|e| e == "traj").unwrap_or(false)).collect(),
            Err(e) => { println!("ARM {arm}: REFUSED — {e}"); continue; }
        };
        paths.sort();
        for p in paths {
            match Trajectory::read(&p) {
                Ok(t) => { *frames_read += t.frames.len() as u64; files.push((arm.clone(), p, t)); }
                Err(e) => println!("  {} REFUSED — {e}", p.display()),
            }
        }
    }
    let Some(first) = files.first() else { println!("no trajectories"); return; };
    let n_atoms = first.2.header.n_atoms;
    let dims = first.2.header.dims;
    let readout_fs = first.2.header.dt * 0.024188843265857;
    let ladder = doubling_ladder(n_atoms, dims);
    println!("# RUNG2_AMENDMENT_5: cadence tau = a / c_s (c_s = 1497 m/s, CRC), fields averaged over the window, bins = held-out sigma of the averaged fields on the SAME arm's other seeds; readout spacing {readout_fs:.2} fs");
    // per (seed, grid): the averaged fields' sigmas on the flexible arm — the calibration pool
    // The pool is every file of the SAME ARM as the one graded, from OTHER seeds: a flexible
    // file is calibrated on other flexible seeds, a rigid one on other rigid seeds — so a
    // scout run on the operator alone still has a hold-out, and no file ever sees its own σ.
    struct Cal { arm: String, seed: u64, grid: Grid3, window: usize, sig: (f64, f64, f64), windows: usize }
    let mut pool: Vec<Cal> = Vec::new();
    for (arm, _, t) in &files {
        for grid in &ladder {
            if grid.cells() < 2 { continue; }
            let tau = cadence_fs(t, *grid);
            let window = ((tau / readout_fs).round() as usize).max(1);
            if let Ok(f) = fields3(t, *grid, Kind::Spatial) {
                let avg = window_mean(&f, window);
                let comps = if grid.nz > 1 { 3 } else { 2 };
                pool.push(Cal { arm: arm.clone(), seed: t.header.seed, grid: *grid, window, sig: field_sigmas(&avg, comps), windows: avg.len() });
            }
        }
    }
    for (arm, path, traj) in &files {
        let h = &traj.header;
        println!("\n===== ARM {arm} — {} =====", path.file_name().unwrap().to_string_lossy());
        println!("-- seed 0x{:016x}  n={} dims={} frames={}", h.seed, h.n_atoms, h.dims, traj.frames.len());
        for grid in &ladder {
            if grid.cells() < 2 { continue; }
            let tau = cadence_fs(traj, *grid);
            let window = ((tau / readout_fs).round() as usize).max(1);
            let held: Vec<&Cal> = pool.iter().filter(|c| c.arm == *arm && c.seed != h.seed && c.grid == *grid).collect();
            if held.is_empty() { println!("   grid {}x{}x{}: REFUSED — no held-out seed to calibrate on", grid.nx, grid.ny, grid.nz); continue; }
            let q = |f: &dyn Fn(&Cal) -> f64| (held.iter().map(|c| f(c).powi(2)).sum::<f64>() / held.len() as f64).sqrt();
            let cal = Density::Calibrated3 { n: q(&|c| c.sig.0), p: q(&|c| c.sig.1), e: q(&|c| c.sig.2) };
            let comps = if grid.nz > 1 { 3 } else { 2 };
            let f = match fields3(traj, *grid, Kind::Spatial) { Ok(f) => f, Err(e) => { println!("   grid: REFUSED — {e:?}"); continue; } };
            let own = field_sigmas(&window_mean(&f, window), comps);
            let nwin = traj.frames.len() / window;
            let (sn, sp, se) = match cal { Density::Calibrated3 { n, p, e } => (n, p, e), _ => unreachable!() };
            println!(
                "   grid {}x{}x{} cells={} tau={:.0} fs window={} frames -> {} windows | held-out sigma(n,p,e)=({:.3}, {:.2}, {:.2e}) from {} seed(s) | own ({:.3}, {:.2}, {:.2e}) NOT used",
                grid.nx, grid.ny, grid.nz, grid.cells(), tau, window, nwin, sn, sp, se, held.len(), own.0, own.1, own.2
            );
            let mut occ_defects: Vec<(Kind, Option<f64>)> = Vec::new();
            for kind in [Kind::Spatial, Kind::BlindLabel] {
                let tr_k = match cell_series3(traj, *grid, kind) { Ok(c) => transport_fraction(&c), Err(_) => 0.0 };
                for rung in [Rung::Occ, Rung::Mom] {
                    let r = match readings_windowed(traj, *grid, rung, kind, cal, window) { Ok(r) => r, Err(e) => { println!("      {kind:?} {rung:?}: REFUSED — {e:?}"); continue; } };
                    *chart_evals += r.len() as u64;
                    let a = leg_a(&r);
                    let v = grade(grid.cells() >= prereg::MIN_CELLS, tr_k, &a);
                    let d = a.defect().map(|x| format!("{x:.4}")).unwrap_or_else(|| "n/a".into());
                    println!("      A5 {kind:?} {rung:?} windows={} coll={} fire={} D_A={d} info={} distinct={} | {v:?}", r.len(), a.collisions, a.firing, a.informative, a.distinct);
                    if rung == Rung::Occ { occ_defects.push((kind, a.defect())); }
                }
            }
            let sp_ = occ_defects.iter().find(|k| k.0 == Kind::Spatial).and_then(|k| k.1);
            let bl = occ_defects.iter().find(|k| k.0 == Kind::BlindLabel).and_then(|k| k.1);
            match (sp_, bl) {
                (Some(ds), Some(db)) => println!("      A5 G7 at Occ: blind − spatial = {:+.4} (needs ≥ +0.05) → {}", db - ds, if db - ds >= prereg::MIN_SEPARATION { "SEPARATED" } else { "NO SEPARATION" }),
                _ => println!("      A5 G7 at Occ: undecidable (a chart has no collisions)"),
            }
            // the required length, from this file's own collision rate: informative per window
            let inf = readings_windowed(traj, *grid, Rung::Occ, Kind::Spatial, cal, window).map(|r| leg_a(&r).informative).unwrap_or(0);
            if inf > 0 && inf < prereg::MIN_INFORMATIVE {
                let need_windows = (prereg::MIN_INFORMATIVE as f64 / (inf as f64 / nwin.max(1) as f64)).ceil();
                println!("      A5 length: {inf} informative in {nwin} windows -> G4 needs ~{need_windows:.0} windows = {:.0} ps at this cadence", need_windows * tau / 1000.0);
            }
        }
    }
}


/// RUNG2_AMENDMENT_6.md: the continuity leg, read at Amendment 5's cadence on every grid of
/// the derived ladder with more than one cell. Prints `D_cont` for the spatial chart and the
/// position-blind control, the G7-form separation, and the window count — VOID by count is
/// said, and the absolute number is printed beside it either way.
fn amendment6_read(root: &Path, arms: &[String], frames_read: &mut u64) {
    for arm in arms {
        let dir = root.join(arm);
        let mut paths: Vec<PathBuf> = match std::fs::read_dir(&dir) {
            Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|e| e == "traj").unwrap_or(false)).collect(),
            Err(e) => { println!("ARM {arm}: REFUSED — {e}"); continue; }
        };
        paths.sort();
        println!("\n===== ARM {arm} ({} trajectories) — continuity leg =====", paths.len());
        for path in &paths {
            let traj = match Trajectory::read(path) { Ok(t) => t, Err(e) => { println!("  {} REFUSED — {e}", path.display()); continue; } };
            *frames_read += traj.frames.len() as u64;
            let h = &traj.header;
            let readout_au = h.dt;
            let readout_fs = readout_au * 0.024188843265857;
            let m_bar = h.z.iter().map(|z| mass_me(*z).unwrap_or(0.0)).sum::<f64>() / h.n_atoms as f64;
            println!("-- {}  seed 0x{:016x} n={} frames={} readout {readout_fs:.1} fs", path.file_name().unwrap().to_string_lossy(), h.seed, h.n_atoms, traj.frames.len());
            for grid in doubling_ladder(h.n_atoms, h.dims) {
                if grid.cells() < 2 { continue; }
                if let Err(why) = continuity_admits(grid) {
                    println!("   grid {}x{}x{}: REFUSED for the continuity leg — {why}", grid.nx, grid.ny, grid.nz);
                    continue;
                }
                let tau = cadence_fs(&traj, grid);
                let window = ((tau / readout_fs).round() as usize).max(1);
                let boxe = [h.box_w, h.box_h, h.box_d];
                let mut d: Vec<(Kind, Option<f64>, usize)> = Vec::new();
                for kind in [Kind::Spatial, Kind::BlindLabel] {
                    match fields3(&traj, grid, kind) {
                        Ok(f) => {
                            let avg = window_mean(&f, window);
                            let c = continuity(&avg, grid, boxe, m_bar, window as f64 * readout_au);
                            d.push((kind, c.defect(), c.windows_compared));
                        }
                        Err(e) => println!("   grid {}x{}x{} {kind:?}: REFUSED — {e:?}", grid.nx, grid.ny, grid.nz),
                    }
                }
                let sp = d.iter().find(|x| x.0 == Kind::Spatial); let bl = d.iter().find(|x| x.0 == Kind::BlindLabel);
                if let (Some((_, Some(ds), nw)), Some((_, Some(db), _))) = (sp, bl) {
                    let enough = *nw >= 20;
                    println!(
                        "   grid {}x{}x{} tau={:.0} fs window={} -> {} transitions | D_cont spatial={:.3} blind={:.3} | blind − spatial = {:+.3} (needs ≥ +0.05) → {}{}",
                        grid.nx, grid.ny, grid.nz, tau, window, nw, ds, db, db - ds,
                        if db - ds >= prereg::MIN_SEPARATION { "SEPARATED" } else { "no separation" },
                        if enough { "" } else { "  [VOID by count: fewer than 20 transitions; the number is printed, not graded]" }
                    );
                } else {
                    println!("   grid {}x{}x{}: undecidable (no density change in a window)", grid.nx, grid.ny, grid.nz);
                }
            }
        }
    }
}


/// RESPONSE1_PREREG.md: the driven reads. `arm` is "L" or "T"; the trajectory is `cycles`
/// kicks each followed by `relax` readouts. Per cycle: the mode (ρ_k for L, j_k for T) is
/// fitted from the kick; the continuity leg is read on the first two windows (the drive) and
/// the last (relaxed, the in-run null); the scrambled partition's mode amplitude is the
/// placebo's placebo. Pooled over cycles with the spread; the stakes graded in place.
fn response1_read(root: &Path, arms: &[String], arm: &str, cycles: usize, relax: usize, frames_read: &mut u64) {
    // RESPONSE1_AMENDMENT_1: the driven current mode is the SIN quadrature (the kick is a sin
    // velocity mode), the density response is the COS one; the undriven quadrature of each
    // is the null R4′; the response is the sign-aligned average over cycles and the
    // per-cycle fits that pass are the spread; the density is fitted from its peak.
    let axis = if arm == "T" { 1 } else { 0 };
    let bohr = 5.29177210903e-11;
    let fmt = |r: &Relaxation, k: f64| -> String {
        match r {
            Relaxation::Overdamped { lambda, amplitude, residual, points } => format!("OVERDAMPED  λ = {lambda:.3e} /s (τ = {:.0} fs)  A = {amplitude:.3e}  resid {residual:.2e}  pts {points}", 1e15 / lambda),
            Relaxation::Underdamped { gamma, omega, amplitude, .. } => format!("UNDERDAMPED Γ = {gamma:.3e} /s  ω = {omega:.3e} /s (period {:.0} fs, c_s = ω/k = {:.0} m/s)  A = {amplitude:.3e}", 2.0 * std::f64::consts::PI / omega * 1e15, omega / k),
            Relaxation::Refused { amplitude, noise } => format!("REFUSED — kick amplitude {amplitude:.3e} under 3 x noise {noise:.3e}"),
        }
    };
    let rate = |r: &Relaxation| -> Option<f64> { match r { Relaxation::Overdamped { lambda, .. } => Some(*lambda), Relaxation::Underdamped { gamma, .. } => Some(*gamma), _ => None } };
    for dir_arm in arms {
        let dir = root.join(dir_arm);
        let mut paths: Vec<PathBuf> = match std::fs::read_dir(&dir) {
            Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map(|e| e == "traj").unwrap_or(false)).collect(),
            Err(e) => { println!("ARM {dir_arm}: REFUSED — {e}"); continue; }
        };
        paths.sort();
        println!("\n===== RESPONSE-1 arm {arm} on {dir_arm} ({} trajectories): {cycles} cycles x {relax} readouts; Amendment 1's read =====", paths.len());
        let mut seed_rates: Vec<f64> = Vec::new();
        for path in &paths {
            let traj = match Trajectory::read(path) { Ok(t) => t, Err(e) => { println!("  {} REFUSED — {e}", path.display()); continue; } };
            *frames_read += traj.frames.len() as u64;
            let h = &traj.header;
            let dt_fs = h.dt * 0.024188843265857;
            let dt_s = dt_fs * 1e-15;
            let l_m = h.box_w * bohr; let k = 2.0 * std::f64::consts::PI / l_m;
            let sp = modes_both(&traj, axis, Kind::Spatial).unwrap();
            let bl = modes_both(&traj, axis, Kind::BlindLabel).unwrap();
            println!("-- {}  seed 0x{:016x} n={} readouts={} dt={dt_fs:.1} fs  k=2π/L={k:.3e} /m", path.file_name().unwrap().to_string_lossy(), h.seed, h.n_atoms, traj.frames.len());
            // the driven current mode (both arms), its null, and the blind partition's version
            let cur = align_cycles(&sp.cur_sin, cycles, relax, 1);
            let cur_null = align_cycles(&sp.cur_cos, cycles, relax, 1);
            let cur_bl = align_cycles(&bl.cur_sin, cycles, relax, 1);
            if cur.per_cycle.is_empty() { println!("   no complete cycle in {} readouts; nothing read", traj.frames.len()); continue; }
            let c_used = cur.per_cycle.len();
            let unrelaxed = cur.tail_sigma.iter().filter(|s| **s > 3.0).count();
            println!("   {c_used} cycles aligned; current mode noise per cycle {:.3e} au, aligned {:.3e} au; kick amplitude aligned {:.3e} au (SNR {:.1}); tails relaxed: {} of {c_used} (aligned tail mean {:.1} SE){}",
                cur.noise_per_cycle, cur.noise, cur.mean[0], cur.mean[0].abs() / cur.noise, c_used - unrelaxed, cur.tail_sigma_mean, if unrelaxed > 0 { " — UNRELAXED cycles: the mode has not returned to zero by the cycle's end; the fit's window is what it is and the rebound is a finding" } else { "" });
            let per: Vec<Relaxation> = cur.per_cycle.iter().map(|c| fit_relaxation(c, dt_s, cur.noise_per_cycle)).collect();
            for (c, r) in per.iter().enumerate() { println!("   cycle {c:2}: {}  | tail mean {:.1} SE", fmt(r, k), cur.tail_sigma[c]); }
            let passed: Vec<f64> = per.iter().filter_map(rate).collect();
            let fit = fit_relaxation(&cur.mean, dt_s, cur.noise);
            println!("   ALIGNED current mode j_k^s: {}", fmt(&fit, k));
            // the spread: per-cycle fits when three or more pass, else leave-one-cycle-out
            let (spread, spread_from) = if passed.len() >= 3 {
                (passed.iter().cloned().fold(f64::MIN, f64::max) - passed.iter().cloned().fold(f64::MAX, f64::min), format!("{} per-cycle fits", passed.len()))
            } else {
                let mut loo = Vec::new();
                for drop in 0..c_used {
                    let mean: Vec<f64> = (0..relax).map(|i| cur.per_cycle.iter().enumerate().filter(|(j, _)| *j != drop).map(|(_, c)| c[i]).sum::<f64>() / (c_used - 1).max(1) as f64).collect();
                    if let Some(r) = rate(&fit_relaxation(&mean, dt_s, cur.noise * ((c_used as f64) / (c_used as f64 - 1.0).max(1.0)).sqrt())) { loo.push(r); }
                }
                (if loo.len() >= 2 { loo.iter().cloned().fold(f64::MIN, f64::max) - loo.iter().cloned().fold(f64::MAX, f64::min) } else { f64::NAN }, format!("leave-one-out over {} of {c_used} cycles", loo.len()))
            };
            // R4 and R4′ on the aligned amplitudes
            let a_sp = cur.mean[0].abs();
            let r4 = cur_bl.mean[0].abs() / a_sp.max(1e-300);
            let r4p = cur_null.mean[0].abs();
            // Amendment 3 A1: a noise allowance. The placebo has no dynamics, so its noise is
            // its fluctuation over the WHOLE aligned cycle, not its last quarter (a tail can
            // sit low by chance: on the L0 partial the density blind's tail read 0.24 against
            // a whole-cycle sd of 1.9, and a 2.3-count value at the peak "fired" against it)
            let sd_all = |v: &[f64]| { let m = v.iter().sum::<f64>() / v.len() as f64; (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt() };
            let r4_bar = (0.1 * a_sp).max(3.0 * sd_all(&cur_bl.mean));
            println!("   R4:  blind/spatial aligned kick amplitude {r4:.3}; blind {:.3e} au vs max(0.1 × spatial, 3σ_blind) = {r4_bar:.3e}: {}", cur_bl.mean[0].abs(), if cur_bl.mean[0].abs() < r4_bar { "no mode on the scrambled partition" } else { "FIRES — the scrambled partition carries the mode" });
            println!("   R4′: undriven quadrature j_k^c aligned amplitude {r4p:.3e} au vs 3σ {:.3e}: {}", 3.0 * cur_null.noise, if r4p < 3.0 * cur_null.noise { "at noise, as it must be" } else { "FIRES — the undriven quadrature carries a kick" });
            if let Some(g) = rate(&fit) {
                seed_rates.push(g);
                if axis == 1 {
                    let rho_kg = h.n_atoms as f64 * 18.01528 * 1.66053906660e-27 / l_m.powi(3);
                    let eta = rho_kg * g / (k * k);
                    let (lo, hi) = (rho_kg * (g - spread / 2.0) / (k * k), rho_kg * (g + spread / 2.0) / (k * k));
                    let kind = if matches!(fit, Relaxation::Underdamped { .. }) { " (UNDERDAMPED: a shear wave at this k — ρΓ/k² is the k-dependent viscosity, ω/k its speed; the classification is the finding)" } else { "" };
                    println!("   R3:  Γ_s = {g:.3e} ± {spread:.2e} /s ({spread_from}) -> η = ρ Γ_s / k² = {eta:.3e} Pa s  [{lo:.2e}, {hi:.2e}]  stake [2.0e-4, 3.0e-3]: {}{kind}", if eta >= 2.0e-4 && eta <= 3.0e-3 { "IN BAND" } else { "OUTSIDE — the model's viscosity is the reading and the band the finding" });
                } else {
                    println!("   longitudinal current j_k^s: rate {g:.3e} ± {spread:.2e} /s ({spread_from}) = ν_l k² if overdamped -> ν_l = {:.3e} m²/s (no band; the fast mode of A3)", g / (k * k));
                }
            } else {
                println!("   {}: the aligned current mode did not read on this seed", if axis == 1 { "R3" } else { "longitudinal current" });
            }
            // R1 / R1′ (Amendment 2): the continuity leg in its integral form on the cycle-aligned
            // fields, 8×1×1 graded and 4×1×1 beside it, the first two windows of the aligned cycle,
            // the noise from the blind partition, the floor printed; the in-run null on the last window
            {
                let m_bar = h.z.iter().map(|z| mass_me(*z).unwrap_or(0.0)).sum::<f64>() / h.n_atoms as f64;
                let boxe = [h.box_w, h.box_h, h.box_d];
                let label = if axis == 0 { "R1 " } else { "R1′" };
                for nx in [8usize, 4] {
                    let grid = Grid3 { nx, ny: 1, nz: 1 };
                    if let Err(why) = continuity_admits(grid) { println!("   {label} grid {nx}x1x1: REFUSED — {why}"); continue; }
                    let tau = cadence_fs(&traj, grid);
                    let w = ((tau / dt_fs).round() as usize).max(1);
                    let lead = 2usize;
                    let d_disc = continuity_spatial_floor(nx);
                    let (Ok(fs_sp), Ok(fs_bl)) = (fields3(&traj, grid, Kind::Spatial), fields3(&traj, grid, Kind::BlindLabel)) else { println!("   {label} grid {nx}x1x1: fields REFUSED"); continue; };
                    let (al_sp, used) = align_fields(&fs_sp, cycles, relax, 1);
                    let (al_bl, _) = align_fields(&fs_bl, cycles, relax, 1);
                    if used == 0 || relax < (lead + 1) * w + 1 { println!("   {label} grid {nx}x1x1: no complete cycle or too few windows (relax {relax}, window {w})"); continue; }
                    let c_sp = continuity_integral(&al_sp[..=lead * w], grid, boxe, m_bar, h.dt, w);
                    let c_bl = continuity_integral(&al_bl[..=lead * w], grid, boxe, m_bar, h.dt, w);
                    let (s_b, _) = driven_floor_from_blind(&c_sp, &c_bl);
                    // the blind partition is re-scrambled per frame, so its observed occupancy
                    // changes are LARGER than the physical crossings and s_b is an underestimate
                    // (the first partial arms read s_b = 0 with D = 0.74 and a separation of
                    // +0.26); the tail-based estimate is printed beside it and the LARGER of the
                    // two s (the smaller floor) is the one graded against, since both are lower
                    // bounds on the signal
                    let (s_t, _) = driven_floor(&al_sp, w, lead);
                    let s1 = if s_t.is_finite() { s_b.max(s_t) } else { s_b };
                    let floor1 = ((d_disc * d_disc * s1 * s1 + 1.0) / (s1 * s1 + 1.0)).sqrt();
                    // Amendment 4: the two-sided floor, noises read on the relaxed last two windows
                    let tail_c = continuity_integral(&al_sp[relax - 1 - lead * w..], grid, boxe, m_bar, h.dt, w);
                    let (s, floor) = driven_floor_two_sided(&c_sp, &tail_c, d_disc);
                    let (d_sp, d_bl) = (c_sp.defect().unwrap_or(f64::NAN), c_bl.defect().unwrap_or(f64::NAN));
                    // the in-run null: the last window of the aligned cycle, relaxed
                    let tail_start = relax - w - 1;
                    let d_tail = continuity_integral(&al_sp[tail_start..], grid, boxe, m_bar, h.dt, w).defect().unwrap_or(f64::NAN);
                    let graded = nx == 8;
                    println!("   {label} grid {nx}x1x1 (τ {tau:.0} fs, window {w}, {used} cycles aligned, lead {lead} windows): D_cont spatial {d_sp:.3}  blind {d_bl:.3}  separation {:+.3} | s {s:.2} (one-sided s {s1:.2}: blind {s_b:.2}, tail {s_t:.2}), floor two-sided {floor:.3} (one-sided {floor1:.3}; D_disc {d_disc:.3}; σ_o {:.3e}, σ_p {:.3e}) | relaxed last window {d_tail:.3}{}",
                        d_bl - d_sp, tail_c.rms_observed, tail_c.rms_predicted, if graded { "" } else { "  [beside the graded grid]" });
                    if graded {
                        if axis == 0 {
                            let sep = d_bl - d_sp >= 0.05;
                            let verdict = if d_sp <= 0.2 && sep { "MET (D ≤ 0.2 and separated)" }
                                else if !sep { "separation FAILS — branch (c): the chart does not beat its placebo under drive" }
                                else if d_sp <= floor + 0.05 { "AT FLOOR (over 0.2 but within 0.05 of the stated floor; not a kill — the arithmetic, not the fluid)" }
                                else { "KILL as staked: D over 0.2 and over its floor by more than 0.05" };
                            println!("        R1 verdict on this seed: {verdict}; null (relaxed ≥ 0.8): {}", if d_tail >= 0.8 { "holds" } else { "FAILS — the tail is not relaxed or the leg reads the tail" });
                        } else {
                            // Amendment 3 A1: the allowance is max(0.1, 2 SE) with the SE from leave-one-cycle-out
                            let mut loo = Vec::new();
                            for skip in 0..used {
                                let (al, _) = align_fields_skip(&fs_sp, cycles, relax, 1, skip);
                                if let Some(d) = continuity_integral(&al[..=lead * w], grid, boxe, m_bar, h.dt, w).defect() { loo.push(d); }
                            }
                            let se = if loo.len() >= 2 { let m = loo.iter().sum::<f64>() / loo.len() as f64; ((loo.iter().map(|d| (d - m).powi(2)).sum::<f64>() / (loo.len() - 1) as f64).sqrt() * ((loo.len() - 1) as f64).sqrt()) } else { f64::NAN };
                            let bar = if se.is_finite() { (2.0 * se).max(0.1) } else { 0.1 };
                            println!("        R1′ verdict: driven {d_sp:.3} vs relaxed {d_tail:.3}, |Δ| = {:.3} vs max(0.1, 2 SE = {:.3}): {}", (d_sp - d_tail).abs(), 2.0 * se, if (d_sp - d_tail).abs() < bar { "continuity sees nothing under shear, as it must" } else { "FIRES — the leg is reading momentum, not flux" });
                        }
                    }
                }
            }
            // R2, the density mode on the L arm: cos quadrature, fitted from its peak (A3); its null is the sin quadrature
            if axis == 0 {
                let rho = align_cycles(&sp.rho_cos, cycles, relax, 1);
                let rho_null = align_cycles(&sp.rho_sin, cycles, relax, 1);
                let rho_bl = align_cycles(&bl.rho_cos, cycles, relax, 1);
                let rd = fit_rise_decay(&rho.mean, dt_s, rho.noise);
                let peak_v = rho.mean[rd.peak];
                println!("   density mode ρ_k^c: noise per cycle {:.3e}, aligned {:.3e} counts; peak {:.3e} at readout {} ({:.0} fs), SNR {:.1}", rho.noise_per_cycle, rho.noise, peak_v, rd.peak, rd.peak as f64 * dt_fs, peak_v.abs() / rho.noise);
                println!("   R2:  from the peak: {}{}", fmt(&rd.slow, k), match rd.fast { Some(l2) => format!("; two-exponential λ₂ = {l2:.3e} /s -> ν_l = λ₂/k² = {:.3e} m²/s (no band)", l2 / (k * k)), None => String::new() });
                if let Relaxation::Overdamped { lambda, .. } = rd.slow { println!("        λ₁ = {lambda:.3e} /s = c_s²/ν_l if overdamped (no band; A3 said 50 m/s likely refuses)"); }
                let r4d = rho_bl.mean[rd.peak].abs() / peak_v.abs().max(1e-300);
                let r4d_bar = (0.1 * peak_v.abs()).max(3.0 * sd_all(&rho_bl.mean));
                println!("   R4 (density): blind/spatial at the peak {r4d:.3}; blind {:.3e} vs max(0.1 × spatial, 3 sd_blind) = {r4d_bar:.3e} (blind whole-cycle sd {:.3e}): {}", rho_bl.mean[rd.peak].abs(), sd_all(&rho_bl.mean), if rho_bl.mean[rd.peak].abs() < r4d_bar { "no mode on the scrambled partition" } else if peak_v.abs() < 3.0 * rho.noise { "not read — the spatial peak is itself under 3σ" } else { "FIRES" });
                // Amendment 3 A2: both forms, the residual chooses
                let (g_c, w_c, res_c) = fit_damped_cosine(&rho.mean, dt_s, rd.peak);
                let res_o = match rd.slow { Relaxation::Overdamped { residual, .. } => residual, _ => f64::NAN };
                let choice = if res_o.is_nan() { "overdamped form not fitted" } else if res_c < 0.9 * res_o { "OSCILLATORY (damped cosine better by > 10 %)" } else if res_o < 0.9 * res_c { "OVERDAMPED (exponential from the peak better by > 10 %)" } else { "UNDECIDED (residuals within 10 %); both banked" };
                println!("   R2 (both forms): damped cosine Γ = {g_c:.3e} /s, ω = {w_c:.3e} /s (period {:.0} fs, c_s = ω/k = {:.0} m/s), resid {res_c:.3e} | exponential from the peak resid {res_o:.3e} -> {choice}", 2.0 * std::f64::consts::PI / w_c * 1e15, w_c / k);
                println!("   R4′ (density): undriven ρ_k^s at the peak {:.3e} vs 3σ {:.3e}: {}", rho_null.mean[rd.peak].abs(), 3.0 * rho_null.noise, if rho_null.mean[rd.peak].abs() < 3.0 * rho_null.noise { "at noise" } else { "FIRES" });
            }
        }
        if seed_rates.len() >= 2 {
            let m = seed_rates.iter().sum::<f64>() / seed_rates.len() as f64;
            let sp = seed_rates.iter().cloned().fold(f64::MIN, f64::max) - seed_rates.iter().cloned().fold(f64::MAX, f64::min);
            println!("   POOLED over {} seeds: rate {m:.3e} /s, seed spread {sp:.2e} /s ({:.0} %)", seed_rates.len(), 100.0 * sp / m);
        }
    }
}
