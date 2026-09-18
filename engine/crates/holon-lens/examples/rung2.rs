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
    args.retain(|a| a != "--amend1" && a != "--amend4" && a != "--amend5");
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
    let mut sigma: Vec<(u64, Grid3, f64)> = Vec::new();
    for (arm, _, t) in &files {
        if arm != "flexible" { continue; }
        for grid in &ladder {
            if let Ok(cs) = cell_series3(t, *grid, Kind::Spatial) {
                sigma.push((t.header.seed, *grid, occupancy_sigma(&cs, grid.cells())));
            }
        }
    }
    println!("# RUNG2_AMENDMENT_4: density bin for a liquid — route (i) External S(0)={WATER_S0} vs route (ii) Calibrated on held-out seeds; momentum/energy per Amendment 3");
    println!("# calibration pool (flexible arms, 2-cell grid): {}", sigma.iter().filter(|s| s.1.cells() == 2).map(|s| format!("seed …{:x} sigma(n)={:.2}", s.0 & 0xff, s.2)).collect::<Vec<_>>().join("; "));
    for (arm, path, traj) in &files {
        let h = &traj.header;
        println!("\n===== ARM {arm} — {} =====", path.file_name().unwrap().to_string_lossy());
        println!("-- seed 0x{:016x}  n={} dims={} frames={}", h.seed, h.n_atoms, h.dims, traj.frames.len());
        for grid in &ladder {
            // PD-4: the calibration comes from OTHER seeds only, and there must be at least one
            let held_out: Vec<f64> = sigma.iter().filter(|s| s.0 != h.seed && s.1 == *grid).map(|s| s.2).collect();
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
