//! RUNG 2 — the fluid-element chart, run against the certified arms.
//!
//! Stakes: `conformance/water_observatory/RUNG2_PREREG.md`, frozen at `aee5317` before
//! this file existed. Every threshold printed below is one of its constants.
//!
//! ```text
//! cargo run --release -p holon-lens --example rung2 -- <traj-dir> [arm ...]
//! ```
//!
//! G1 (digest identity) is NOT performed here: this crate has zero dependencies and
//! therefore no sha256. The digests are verified by `sha256sum -c` against
//! `census_traj_manifest.sha256` before this runner is invoked, and the results document
//! cites that check's output. A pin names what it measured and nothing beside it.

use holon_lens::field::*;
use holon_lens::traj::Trajectory;
use std::collections::HashSet;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
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

    println!("\n===== COST (PREREG G11, work units, never wall clock) =====");
    println!("frames read:       {frames_read}");
    println!("chart evaluations: {chart_evals}");
    let _ = HashSet::<u8>::new();
}
