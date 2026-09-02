//! THE RUNG-1 RUNNER: point it at trajectories, get the network tier's verdict table.
//!
//! Every threshold is a `PREREG_*` constant inherited from the census or a `RUNG1_*`
//! constant staked in `conformance/water_observatory/RUNG1_PREREG.md`, frozen before this
//! file existed. Nothing is tuned here and nothing is passed in on the command line except
//! which trajectories to read and how many surrogates to spend.
//!
//! ```text
//! cargo run --release -p holon-lens --example rung1 -- <dir-or-file> [...] [--surrogates=N]
//! ```
//!
//! **G-ID is checked by this runner, on the fenced arm, and it is the reason to trust
//! every other number it prints.** The census banked eight bonded-partition defects in
//! `CENSUS_RESULTS.md` §11.2. This instrument's chart C6 IS that view, computed through
//! the census's own `closure_leg`, so those eight numbers must come back to four decimals.
//! If they do not, this is not the census's instrument and no reading below means anything.

use holon_lens::census::Stakes;
use holon_lens::network::{self, Chart, Rung1, StructVerdict};
use holon_lens::traj::Trajectory;
use std::path::PathBuf;

/// The census's banked bonded-partition readings, keyed by **(arm, seed)** and not by seed
/// alone.
///
/// Keying by seed alone was this runner's own first version and the gate caught it: the
/// three mixed arms REUSE seed `0x…5422`, so a seed-keyed lookup checked two dE₄
/// trajectories against the fenced arm's row and reported MISMATCH on numbers that were
/// never supposed to agree. A lookup keyed by less than what distinguishes the things it
/// looks up will give one thing two verdicts under two names.
///
/// `fenced` rows are `CENSUS_RESULTS.md` §11.2. The `de4_off` row is §13.1, which banks
/// three quantities for that arm — 43 distinct readings, 124 witness pairs, defect
/// 0.1365 — so that arm is checked on all three rather than on the defect alone.
/// `de4_on` has NO banked row: §13 was written with arm A still running, so this runner
/// says "not banked" rather than inventing a comparison.
struct Banked {
    arm: &'static str,
    seed_low: u8,
    defect: f64,
    distinct: Option<usize>,
    witness_pairs: Option<usize>,
}

const CENSUS_BANKED: [Banked; 9] = [
    Banked { arm: "fenced", seed_low: 0x21, defect: 0.1128, distinct: None, witness_pairs: None },
    Banked { arm: "fenced", seed_low: 0x22, defect: 0.1328, distinct: None, witness_pairs: None },
    Banked { arm: "fenced", seed_low: 0x23, defect: 0.1339, distinct: None, witness_pairs: None },
    Banked { arm: "fenced", seed_low: 0x24, defect: 0.1453, distinct: None, witness_pairs: None },
    Banked { arm: "fenced", seed_low: 0x25, defect: 0.1410, distinct: None, witness_pairs: None },
    Banked { arm: "fenced", seed_low: 0x26, defect: 0.0815, distinct: None, witness_pairs: None },
    Banked { arm: "fenced", seed_low: 0x27, defect: 0.1460, distinct: None, witness_pairs: None },
    Banked { arm: "fenced", seed_low: 0x28, defect: 0.1287, distinct: None, witness_pairs: None },
    Banked {
        arm: "de4_off",
        seed_low: 0x22,
        defect: 0.1365,
        distinct: Some(43),
        witness_pairs: Some(124),
    },
];

/// The arm a trajectory belongs to, read from its parent directory. The arm is part of a
/// trajectory's identity and the manifest is organised by it.
fn arm_of(p: &std::path::Path) -> String {
    p.parent()
        .and_then(|d| d.file_name())
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "?".into())
}

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
    let surrogates: usize = args
        .iter()
        .find_map(|a| a.strip_prefix("--surrogates="))
        .map(|s| s.parse().expect("integer surrogate count"))
        .unwrap_or(network::RUNG1_SURROGATES);
    let paths = collect(
        &args
            .iter()
            .filter(|a| !a.starts_with("--"))
            .cloned()
            .collect::<Vec<_>>(),
    );
    if paths.is_empty() {
        eprintln!("usage: rung1 <dir-or-file> [...] [--surrogates=N]");
        std::process::exit(2);
    }
    let st = Stakes::default();

    println!("# RUNG 1 — THE H-BOND NETWORK TIER");
    println!(
        "# stakes (RUNG1_PREREG.md, frozen before this instrument existed):\n\
         #   window          W = {} fs, beta = {}, breach run <= {} fs   [INHERITED from the census]\n\
         #   moving carrier  rms >= {} bohr, separation excursion >= {} bohr   [INHERITED]\n\
         #   work count      >= {} informative transitions   [INHERITED]\n\
         #   dynamism        >= {} changes, >= {} distinct readings   [RUNG-1]\n\
         #   closure budget  delta* = {}   [RUNG-1, a declared judgment from holon.rs CLOSURE_DEFECT_MAX]\n\
         #   non-expansion   D(2nd half) <= {} x D(1st half)   [INHERITED, OBJECT.md rule 1]\n\
         #   contamination   <= {}   [RUNG-1]\n\
         #   control floor   p_data <= max(p_null_p95, {}), {} surrogates   [AMENDMENT-1]",
        st.window_fs,
        st.beta,
        st.flicker_fs,
        st.min_rms_bohr,
        st.min_sep_var_bohr,
        st.min_informative,
        network::RUNG1_MIN_CHANGES,
        network::RUNG1_MIN_DISTINCT,
        network::RUNG1_DELTA_STAR,
        st.nonexpansion,
        network::RUNG1_MAX_CONTAMINATION,
        network::RUNG1_MAX_Q,
        surrogates,
    );
    println!("# the scale fence: these scenes are 34.6 x 20.8 bohr = 1.83 x 1.10 nm in TWO");
    println!("# dimensions with four oxygens. The band is nominally ~10 nm. No verdict below");
    println!("# licenses a claim at that scale (RUNG1_PREREG.md §7.2).");

    let mut g_id: Vec<(String, u64, usize, f64, usize)> = Vec::new();
    let mut g_n2_violations: Vec<String> = Vec::new();
    let mut refusals = 0usize;

    for path in &paths {
        let arm = arm_of(path);
        let name = format!("{arm}/{}", path.file_name().unwrap().to_string_lossy());
        let traj = match Trajectory::read(path) {
            Ok(t) => t,
            Err(e) => {
                println!("\n## {name}\nREAD FAILED: {e}");
                continue;
            }
        };
        println!("\n## {name}");
        match network::run(&traj, &st, surrogates) {
            Rung1::Refused { gate, reason } => {
                refusals += 1;
                println!("REFUSED at [{gate}]");
                println!("  {reason}");
            }
            Rung1::Report(r) => {
                println!(
                    "seed 0x{:016x}  {} atoms ({} O)  {} frames  {:.1} fs  complete={}",
                    r.seed,
                    r.n_atoms,
                    r.n_oxygens,
                    r.n_frames,
                    r.span_fs,
                    traj.is_complete()
                );

                // ---- G-N3, and branch (E)'s own quantity ----------------------------
                println!(
                    "G-N3 contamination {:.4} = {}/{} H-bond records on a covalently bonded \
                     O-O pair  [{}]",
                    r.contamination,
                    r.hb_contaminated,
                    r.hb_records,
                    if r.hb_records == 0 {
                        "VOID(empty chart)".to_string()
                    } else if r.contamination > network::RUNG1_MAX_CONTAMINATION {
                        "VOID(chart does not contain its variable)".to_string()
                    } else {
                        "ok".to_string()
                    }
                );
                println!(
                    "     frames carrying >= 1 INTER-molecular H-bond: {} of {} ({:.4})",
                    r.frames_with_intermolecular,
                    r.n_frames,
                    r.frames_with_intermolecular as f64 / r.n_frames as f64
                );
                // WHICH CLAUSE IS BINDING. Three nested conditions, so the drop between
                // two columns names the one doing the work rather than leaving a low
                // H-bond count to be explained by guesswork.
                println!(
                    "     criterion decomposed: O-O within {} bohr in {} frames -> plus O-H \
                     within {} bohr in {} -> plus angle < {}° in the {} records above",
                    holon_lens::lens::HB_R_OO_BOHR,
                    r.frames_oo_in_range,
                    holon_lens::lens::HB_R_OH_BOHR,
                    r.frames_oh_in_range,
                    holon_lens::lens::HB_ANGLE_DEG,
                    r.hb_records
                );
                // THE SCOPE NUMBER for a network claim: a network of molecules needs at
                // least two molecules.
                println!(
                    "     oxygen-bearing molecules per frame: {}  ({:.2}% of frames hold >= 2)",
                    r.oxygen_blocks_hist
                        .iter()
                        .enumerate()
                        .map(|(k, n)| format!("{k}:{n}"))
                        .collect::<Vec<_>>()
                        .join("  "),
                    100.0 * r.oxygen_blocks_hist.iter().skip(2).sum::<usize>() as f64
                        / r.n_frames as f64
                );
                println!(
                    "     DISCRIMINATOR (diagnostic, not a gate): under a RELAXED donor rule \
                     (any O within 2.5 bohr of the H, not only the nearest) the same three \
                     clauses fire {} times against {}",
                    r.records_any_donor, r.hb_records
                );
                // IS THE SCENE THE SCENE IT DECLARES? Measured and printed, never a gate:
                // found after the freeze, and gating on it afterwards would move a stake.
                match r.first_out_of_plane {
                    None => println!(
                        "     planarity: dims=2 and z holds its placement value EXACTLY in \
                         all {} frames",
                        r.n_frames
                    ),
                    Some(f) => println!(
                        "     planarity: *** dims=2 but the scene LEAVES THE PLANE at frame \
                         {f}, reaching |z-z0| = {:.3} bohr against a box half-depth of {:.1}. \
                         This trajectory is not the scene it declares; see RUNG1_RESULTS.md.",
                        r.worst_out_of_plane,
                        traj.header.box_d / 2.0
                    ),
                }

                // ---- Leg B-N and Leg F, per chart ------------------------------------
                println!(
                    "\n  {:<20} {:>8} {:>8} {:>9} {:>8} {:>8} {:>7} {:>9} {:>8}  verdict",
                    "chart", "distinct", "changes", "informat.", "defect", "1st/2nd", "nonexp", "F", "F-wit"
                );
                let mut coll: std::collections::HashMap<&str, usize> =
                    std::collections::HashMap::new();
                for c in &r.charts {
                    coll.insert(c.chart.tag(), c.collisions);
                    let cl = &c.closure;
                    let verdict = if c.vacuous {
                        "VOID(G-N7 anti-vacuity)".to_string()
                    } else if cl.defect > network::RUNG1_DELTA_STAR {
                        format!("NOT CLOSED (G-N4: {:.4} > {})", cl.defect, network::RUNG1_DELTA_STAR)
                    } else if !cl.nonexpansion_ok {
                        "NOT CLOSED (G-N5 non-expansion)".to_string()
                    } else {
                        "within budget".to_string()
                    };
                    println!(
                        "  {:<20} {:>8} {:>8} {:>9} {:>8.4} {:>8} {:>7} {:>9.4} {:>8}  {}",
                        c.chart.tag(),
                        c.distinct,
                        c.changes,
                        cl.informative_transitions,
                        cl.defect,
                        format!("{:.3}/{:.3}", cl.defect_first_half, cl.defect_second_half),
                        if cl.nonexpansion_ok { "ok" } else { "BREACH" },
                        c.factor.defect,
                        c.factor.witness_count,
                        verdict
                    );
                    if let Some(why) = &c.void_reason {
                        println!("        cause: {why}");
                    }
                    if !c.factor.witnesses.is_empty() {
                        println!(
                            "        Leg F witnesses (frames): {:?}",
                            &c.factor.witnesses[..c.factor.witnesses.len().min(4)]
                        );
                    }
                }

                // ---- G-ID: does chart C6 reproduce the census's banked readings? -----
                let c6 = r
                    .charts
                    .iter()
                    .find(|c| c.chart == Chart::MolPart)
                    .expect("C6 present");
                g_id.push((
                    arm.clone(),
                    r.seed,
                    c6.distinct,
                    c6.closure.defect,
                    c6.closure.witness_pair_count,
                ));

                // ---- G-N2: the ladders are nested, so collisions are monotone --------
                for (fine, coarse) in [
                    (Chart::HbFull.tag(), Chart::HbAdj.tag()),
                    (Chart::HbAdj.tag(), Chart::HbPart.tag()),
                    (Chart::MolNetId.tag(), Chart::MolNetFormula.tag()),
                ] {
                    if coll[fine] > coll[coarse] {
                        g_n2_violations.push(format!(
                            "{name}: {fine} ({}) > {coarse} ({})",
                            coll[fine], coll[coarse]
                        ));
                    }
                }

                // ---- Leg A-N ---------------------------------------------------------
                if r.structures.is_empty() {
                    println!("\n  Leg A-N: no H-bond component of two or more oxygens EVER forms.");
                } else {
                    println!(
                        "\n  {:<8} {:>4} {:>8} {:>10} {:>7} {:>8} {:>8} {:>8} {:>8} {:>8} {:>5}  verdict",
                        "struct", "size", "present", "longest fs", "of run", "rms", "sep var",
                        "p_data", "p_null", "bar", "maxocc"
                    );
                    for s in &r.structures {
                        let f = s.floor;
                        println!(
                            "  {:>8} {:>4} {:>8} {:>10.1} {:>6.1}% {:>8.3} {:>8.3} {:>8} {:>8} {:>8} {:>5}  {}",
                            mask_hex(&s.structure),
                            s.size,
                            s.frames_present,
                            s.longest_run_fs,
                            100.0 * s.frames_present as f64 / r.n_frames as f64,
                            s.rms_internal,
                            s.max_sep_variation,
                            f.map(|f| format!("{:.4}", f.p_data)).unwrap_or_else(|| "-".into()),
                            f.map(|f| format!("{:.4}", f.p_null_p95)).unwrap_or_else(|| "-".into()),
                            f.map(|f| format!("{:.4}", f.bar)).unwrap_or_else(|| "-".into()),
                            f.map(|f| format!("{:.3}", f.max_edge_occupancy))
                                .unwrap_or_else(|| "-".into()),
                            s.verdict.tag()
                        );
                        if let StructVerdict::VoidNoSeparation { p_data, bar } = s.verdict {
                            println!(
                                "        cause: G-N11 control floor, p_data {p_data:.4} > bar {bar:.4}"
                            );
                        }
                        if let StructVerdict::VoidFrozenCarrier { rms, sep_var } = s.verdict {
                            println!(
                                "        cause: G-N10 moving carrier, rms {rms:.4} (>= {}), \
                                 sep var {sep_var:.4} (>= {})",
                                st.min_rms_bohr, st.min_sep_var_bohr
                            );
                        }
                        if let Some(f) = f {
                            println!(
                                "        floor diagnostics (NOT binding): q_any {:.3}, pool {}",
                                f.q_any, f.pool
                            );
                        }
                    }
                }
            }
        }
    }

    // ================================================================ the gate summary
    println!("\n\n# =============================== GATE SUMMARY");
    println!("\n## G-ID — does chart C6 reproduce the census's banked readings, EXACT to 4 dp?");
    let mut id_ok = true;
    let mut id_checked = 0usize;
    for (arm, seed, distinct, defect, wp) in &g_id {
        let banked = CENSUS_BANKED
            .iter()
            .find(|b| b.arm == arm && b.seed_low == (seed & 0xff) as u8);
        match banked {
            Some(b) => {
                id_checked += 1;
                let mut fails: Vec<String> = Vec::new();
                if format!("{defect:.4}") != format!("{:.4}", b.defect) {
                    fails.push(format!("defect {defect:.4} != {:.4}", b.defect));
                }
                if let Some(d) = b.distinct {
                    if *distinct != d {
                        fails.push(format!("distinct {distinct} != {d}"));
                    }
                }
                if let Some(w) = b.witness_pairs {
                    if *wp != w {
                        fails.push(format!("witness pairs {wp} != {w}"));
                    }
                }
                if !fails.is_empty() {
                    id_ok = false;
                }
                println!(
                    "  {arm:<8} 0x{seed:016x}  defect {defect:.4} (banked {:.4}){}  {}",
                    b.defect,
                    match (b.distinct, b.witness_pairs) {
                        (Some(d), Some(w)) =>
                            format!("  distinct {distinct} (banked {d})  witness pairs {wp} (banked {w})"),
                        _ => String::new(),
                    },
                    if fails.is_empty() { "MATCH".to_string() } else { format!("MISMATCH: {}", fails.join("; ")) }
                );
            }
            None => println!(
                "  {arm:<8} 0x{seed:016x}  defect {defect:.4}  distinct {distinct}  witness pairs {wp}  \
                 — NOT BANKED (the census has no row for this arm; nothing to check against)"
            ),
        }
    }
    println!(
        "  G-ID: {} over {} checkable seeds",
        if id_checked == 0 {
            "NOT EXERCISED"
        } else if id_ok {
            "PASS"
        } else {
            "FAIL — this is not the census's instrument and nothing above stands"
        },
        id_checked
    );

    println!("\n## G-N2 — refinement_removes_collisions: the nested ladders are monotone");
    if g_n2_violations.is_empty() {
        println!("  PASS — no violation on any trajectory (EXACT, no tolerance)");
    } else {
        println!("  FAIL — the instrument is convicted, not the physics:");
        for v in &g_n2_violations {
            println!("    {v}");
        }
    }
    if refusals > 0 {
        println!("\n## G-N12 — {refusals} trajectory(ies) REFUSED for want of the variable");
    }
}

/// A structure's oxygen-position mask as the hex the record has always printed, or its
/// member count where it is wider than a 128-bit word (a scene the record never had).
fn mask_hex(m: &holon_lens::partition::Mask) -> String {
    let mut v: u128 = 0;
    for i in m.iter() {
        if i >= 128 {
            return format!("{{{} ox}}", m.popcount());
        }
        v |= 1u128 << i;
    }
    format!("0x{v:04x}")
}
