//! RUNG 2 / A2 — the LATTICE-GAS chart, run against the certified arms.
//!
//! Stakes: `conformance/water_observatory/RUNG2_PREREG_A2.md`, frozen at `e5bd812` before
//! this file existed. Every threshold printed below is one of its constants.
//!
//! **Why this example lives in `holon-mesh` and not beside the banked runner.** The chart's
//! `(N, P)` label must come from `ciris_sim_core::regplus::sector` — the one implementation,
//! whose own test reproduces `Core/Lattice.lean`'s 53 sectors with histogram 44/7/2 in-tree.
//! `holon-lens` has zero dependencies by design and cannot import it. `holon-mesh` already
//! depends on `ciris-sim-core`, so this is the crate that can see BOTH the labeller and the
//! trajectory reader, and no second sector implementation is created anywhere.
//!
//! ```text
//! cargo run --release -p holon-mesh --example rung2_lg -- <traj-dir> [arm ...]
//! ```

use ciris_sim_core::regplus;
use holon_lens::field::{
    self, grade, leg_a, leg_b, prereg, refines, Kind, FROZEN_GRIDS,
};
use holon_lens::field_lg::*;
use holon_lens::traj::Trajectory;
use std::path::PathBuf;

/// The one labeller. `regplus::sector` is `Core/Lattice.lean`'s object in the runtime; this
/// adapter only reshapes its return, and computes nothing.
fn label(w: u8) -> (u8, [i8; 2]) {
    let s = regplus::sector(w);
    (s.occupancy, s.momentum)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: rung2_lg <traj-dir> [arm ...]");
        std::process::exit(2);
    }
    let root = PathBuf::from(&args[0]);
    let arms: Vec<String> = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        vec!["fenced".into(), "hydrogen".into()]
    };

    // The pin, asserted at start-up rather than trusted: this crate can see both sides.
    assert_eq!(
        DIRECTIONS_AXIAL, regplus::DIRECTIONS,
        "holon-lens's pinned FHP directions have drifted from regplus::DIRECTIONS"
    );

    println!("# RUNG 2 / A2 — the lattice-gas (N,P) chart");
    println!("# stakes: RUNG2_PREREG_A2.md @ e5bd812");
    println!("# chart: FHP-6 (dims=2 carrier); engine/MESH_DESIGN.md §2.1's 3D FCHC-24 not exercised");
    println!("# labeller: ciris_sim_core::regplus::sector (53 sectors, 44/7/2, in-tree test)");
    println!(
        "# bars: A1 distinct words >= {MIN_DISTINCT_WORDS}; transport >= {}; informative >= {}; \
         beta = {}; separation >= {}",
        prereg::MIN_TRANSPORT,
        prereg::MIN_INFORMATIVE,
        prereg::BETA,
        prereg::MIN_SEPARATION
    );

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
            println!(
                "\n-- seed 0x{:016x}  n={} frames={}",
                traj.header.seed,
                traj.header.n_atoms,
                traj.frames.len()
            );

            for grid in FROZEN_GRIDS {
                // Transport is a property of the CELLS and is shared with the banked
                // cell-field chart by construction — that is what keeps the comparison
                // one-variable.
                let cs = match field::cell_series(&traj, grid, Kind::Spatial) {
                    Ok(c) => c,
                    Err(e) => {
                        println!("   grid {}x{}: REFUSED — {e:?}", grid.nx, grid.ny);
                        continue;
                    }
                };
                let transport = field::transport_fraction(&cs);
                let (mean_occ, fluct) = field::occupancy_stats(&cs, grid.cells());
                let g2 = mean_occ >= prereg::ADMISSIBLE_OCCUPANCY
                    && grid.cells() >= prereg::ADMISSIBLE_CELLS
                    && fluct <= prereg::ADMISSIBLE_FLUCTUATION;
                println!(
                    "   grid {}x{} cells={} atoms/cell={:.3} fluct={:.3} transport={:.4}  \
                     G2 admissible: {}",
                    grid.nx,
                    grid.ny,
                    grid.cells(),
                    mean_occ,
                    fluct,
                    transport,
                    if g2 { "YES" } else { "NO" }
                );

                for kind in [Kind::Spatial, Kind::BlindLabel, Kind::GlobalRelabel] {
                    let (words, st) = match local_words(&traj, grid, kind, MapKind::Velocity) {
                        Ok(v) => v,
                        Err(e) => {
                            println!("      {kind:?}: REFUSED — {e:?}");
                            continue;
                        }
                    };
                    // A2g — the map's cost, disclosed for every grid whatever the verdict.
                    // A1 — map non-degeneracy.
                    let a1 = st.distinct_words >= MIN_DISTINCT_WORDS;
                    println!(
                        "      {kind:?} MAP saturation={:.4} lost_atoms={:.4} zero_vel={} \
                         words={} | A1 {}",
                        st.saturation(),
                        st.lost_fraction(),
                        st.zero_velocity_atoms,
                        st.distinct_words,
                        if a1 { "ok" } else { "VOID (degenerate map)" }
                    );

                    let mut counts: Vec<u128> = Vec::new();
                    let mut prev: Option<Vec<field::Reading>> = None;
                    for rung in LG_LADDER {
                        let r = readings_from_words(&words, rung, label);
                        chart_evals += r.len() as u64;
                        let a = leg_a(&r);
                        let b = leg_b(&r);
                        counts.push(a.collisions);
                        let ref_ok = match &prev {
                            None => true,
                            Some(p) => refines(&r, p),
                        };
                        let v = if !a1 {
                            "VoidDegenerateMap".to_string()
                        } else {
                            format!(
                                "{:?}",
                                grade(grid.cells() >= prereg::MIN_CELLS, transport, &a)
                            )
                        };
                        println!(
                            "      {kind:?} {rung:?} coll={} fire={} D_A={} info={} distinct={} \
                             | D_B={} cov={:.3} | refines={} | {v}",
                            a.collisions,
                            a.firing,
                            a.defect().map(|x| format!("{x:.6}")).unwrap_or("n/a".into()),
                            a.informative,
                            a.distinct,
                            b.defect().map(|x| format!("{x:.6}")).unwrap_or("n/a".into()),
                            b.coverage(),
                            ref_ok
                        );

                        // A3 — the phase sweep, on the operator's chart (w2) and the real
                        // chart only. Door (c) reads this and nothing else.
                        if rung == LgRung::W2 && kind == Kind::Spatial {
                            let mut best: Option<(usize, usize, f64, usize)> = None;
                            let mut zero_found = false;
                            for p in PHASE_PERIODS {
                                for res in 0..p {
                                    let ph = phase_defect(&r, p, res);
                                    if ph.informative < prereg::MIN_INFORMATIVE {
                                        continue;
                                    }
                                    if let Some(d) = ph.defect() {
                                        if d == 0.0 {
                                            zero_found = true;
                                        }
                                        if best.map(|b| d < b.2).unwrap_or(true) {
                                            best = Some((p, res, d, ph.informative));
                                        }
                                    }
                                }
                            }
                            match best {
                                Some((p, res, d, info)) => println!(
                                    "      A3 phase sweep: best (p={p}, r={res}) D_A={d:.6} \
                                     info={info} | grain boundary found: {}",
                                    if zero_found { "YES" } else { "NO" }
                                ),
                                None => println!(
                                    "      A3 phase sweep: no (p,r) reached the work count — VOID"
                                ),
                            }
                        }
                        prev = Some(r);
                    }
                    println!(
                        "      {kind:?} G8 weak (monotone counts): {}",
                        field::ladder_monotone(&counts)
                    );
                }
            }
        }
    }

    println!("\n===== COST (A2 §7, work units, never wall clock) =====");
    println!("frames read:       {frames_read}");
    println!("chart evaluations: {chart_evals}");
}
