//! REPLACE-1's driver — the fluid-element tier on the staggered chart, with a removability
//! gate (`conformance/replace1/REPLACE1_PREREG.md`). Runs G1–G5 and PR-1..PR-5 from the
//! engine's own code path on the walks and the chains, and prints a verdict table.
//!
//! ```text
//! cargo run --release -p holon-closure --example replace1 -- \
//!     [--walks DIR] [--chains FILE] [--only pr5,g1,g2,g3,g4,g5,plants,hbond,chains]
//! ```
//!
//! `--walks` is the directory holding `response1_L200_seed{0,1,2}/`, `slow1/T293_seed{0,1,2}/`,
//! `slow1/T400_seed0/`, `molsearch1*/` and `molsearch2_5ps/` (default: the main checkout's
//! `conformance/water_observatory/replace0`, where the walks live — they are not in git);
//! `--chains` is `accord_traces.jsonl`. Nothing is written; the record is this program's
//! stdout, banked as `conformance/replace1/replace1_read.txt`.

mod chains;

use holon::affine::Gate;
use holon::sector::{self, referee, Observable};
use holon_closure::removable::{
    self, admit, refuse_drop, Admission, Candidate, Folds, Horizon, Mat, Nulls, NumpyPcg64, Question, Ridge, Swap, Unit,
    Verdict,
};
use holon_lens::staggered::{self, FluidChart, GridRead};
use holon_lens::walk::{self, AtomWalk, OrderFeatures, RigidWalk, C_LS, C_Q, C_STRUCT, STRUCT_FIELDS};
use std::path::{Path, PathBuf};
use std::time::Instant;

const DEFAULT_WALKS: &str = "/home/emoore/CIRISHolon/conformance/water_observatory/replace0";
const DEFAULT_CHAINS: &str = "/home/emoore/RATCHET/release/data_scrubbed_v1/accord_traces.jsonl";
/// G2's budget, and the one every fluid admission here is read at.
const BETA: f64 = 0.02;

struct Row {
    stake: String,
    staked: String,
    read: String,
    verdict: String,
}

struct Out {
    rows: Vec<Row>,
    plants_ok: Vec<(String, bool)>,
}

impl Out {
    fn row(&mut self, stake: &str, staked: &str, read: String, verdict: &str) {
        self.rows.push(Row { stake: stake.into(), staked: staked.into(), read, verdict: verdict.into() });
    }
}

fn main() {
    let mut walks = PathBuf::from(DEFAULT_WALKS);
    let mut chains_path = DEFAULT_CHAINS.to_string();
    let mut only: Option<Vec<String>> = None;
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--walks" => {
                walks = PathBuf::from(&args[i + 1]);
                i += 1;
            }
            "--chains" => {
                chains_path = args[i + 1].clone();
                i += 1;
            }
            "--only" => {
                only = Some(args[i + 1].split(',').map(|s| s.to_string()).collect());
                i += 1;
            }
            a => panic!("unknown argument {a}"),
        }
        i += 1;
    }
    let run = |s: &str| only.as_ref().is_none_or(|o| o.iter().any(|x| x == s));
    let t0 = Instant::now();
    let mut out = Out { rows: Vec::new(), plants_ok: Vec::new() };
    println!("# REPLACE-1 — the fluid-element tier on the staggered chart, with a removability gate");
    println!("# engine read (holon-closure::removable, holon-lens::staggered/walk, holon::sector); walks {}", walks.display());

    if run("pr5") || run("g1") {
        charts(&walks, &mut out);
    }
    let need_slow = ["g2", "g3", "g5", "plants", "a1"].iter().any(|s| run(s));
    let feats: Vec<(String, OrderFeatures)> = if need_slow {
        ["T293_seed0", "T293_seed1", "T293_seed2", "T400_seed0"]
            .iter()
            .map(|s| {
                let t = Instant::now();
                let w = RigidWalk::load(&walks.join("slow1").join(s), None).expect("SLOW-1 walk");
                let f = walk::order_features(&w);
                println!(
                    "# {s}: {} waters x {} readouts at {} fs = {:.2} ps, box {:.3} A, <q> {:.4}; dictionary built in {:.1} s",
                    w.n,
                    w.frames,
                    f.dt_fs,
                    (w.frames - 1) as f64 * f.dt_fs / 1000.0,
                    w.box_bohr * walk::BOHR_A_SLOW1,
                    f.q_mean,
                    t.elapsed().as_secs_f64()
                );
                (s.to_string(), f)
            })
            .collect()
    } else {
        Vec::new()
    };
    if run("plants") {
        plants(&feats, &mut out);
    }
    let mut g3_fluid: Vec<(String, Admission, Vec<Mat>)> = Vec::new();
    if run("g2") {
        g2(&feats, &walks, run("hbond") || only.is_none(), &mut out);
    }
    if run("g3") || run("g5") {
        g3_fluid = g3(&feats, &mut out);
        if run("chains") || only.is_none() || run("g3") {
            g3_chains(&chains_path, &mut out);
        }
    }
    if run("g5") {
        g5(&g3_fluid, &mut out);
    }
    if run("g4") {
        g4(&mut out);
    }
    if only.as_ref().is_some_and(|o| o.iter().any(|x| x == "a1")) {
        amend1(&feats, &chains_path);
        return;
    }

    println!("\n# VERDICT TABLE");
    for r in &out.rows {
        println!("{} | staked: {} | read: {} -> {}", r.stake, r.staked, r.read, r.verdict);
    }
    let plants_all = out.plants_ok.iter().all(|(_, ok)| *ok);
    println!(
        "plants: {}",
        out.plants_ok.iter().map(|(n, ok)| format!("{n} {}", if *ok { "PASS" } else { "FAIL" })).collect::<Vec<_>>().join(", ")
    );
    let v = |s: &str| out.rows.iter().filter(|r| r.stake.starts_with(s)).map(|r| r.verdict.clone()).collect::<Vec<_>>();
    let kill = |s: &str| v(s).iter().any(|x| x.starts_with("KILL"));
    let met = |s: &str| !v(s).is_empty() && v(s).iter().all(|x| x.starts_with("MET"));
    let branch = if !plants_all {
        "(e) a plant fails - nothing is read".to_string()
    } else if kill("G1") {
        "(b) G1 killed by the port - the Python reader and the engine disagree".to_string()
    } else if kill("G2") || kill("G3") {
        "(c) G2 or G3 killed - the removability statistic does not reproduce the banked reads".to_string()
    } else if ["G1", "G2", "G3", "G4", "G5"].iter().all(|s| met(s)) {
        "(a) G1-G5 met - the fluid-element tier runs on the staggered chart with an admission certificate".to_string()
    } else {
        format!(
            "none of (a)-(e) cleanly: {}",
            ["G1", "G2", "G3", "G4", "G5"].iter().map(|s| format!("{s} {:?}", v(s))).collect::<Vec<_>>().join("; ")
        )
    };
    if only.is_none() {
        println!("BRANCH: {branch}");
    } else {
        println!("BRANCH: not graded (--only ran a subset)");
    }
    println!("# wall {:.1} s", t0.elapsed().as_secs_f64());
}

// ---------------------------------------------------------------------------------------------
// PR-5 and G1: the charts
// ---------------------------------------------------------------------------------------------

fn grid_text(reads: &[GridRead], pooled: Option<usize>) -> String {
    let mut s = String::new();
    for g in reads {
        s.push('\n');
        s.push_str(&staggered::format_grid(g, pooled));
    }
    s
}

fn compare_lines(mine: &str, banked: &str) -> (usize, usize, Vec<(String, String)>) {
    let a: Vec<&str> = mine.lines().collect();
    let b: Vec<&str> = banked.lines().collect();
    let mut same = 0;
    let mut diffs = Vec::new();
    for k in 0..a.len().max(b.len()) {
        let (x, y) = (a.get(k).copied().unwrap_or(""), b.get(k).copied().unwrap_or(""));
        if x == y {
            same += 1;
        } else {
            diffs.push((x.to_string(), y.to_string()));
        }
    }
    (same, a.len().max(b.len()), diffs)
}

fn charts(walks: &Path, out: &mut Out) {
    println!("\n## PR-5 and G1 — the cell chart and the staggered chart through one interface (holon_lens::staggered)");
    let charts = FluidChart::r1_rows();
    let mut arms: Vec<Vec<GridRead>> = Vec::new();
    for k in 0..3 {
        let d = walks.join(format!("response1_L200_seed{k}"));
        let t = Instant::now();
        let w = RigidWalk::load(&d, None).expect("RESPONSE-1 arm");
        let reads: Vec<GridRead> = [4usize, 8, 16].iter().map(|&nx| staggered::closure_read(&w, nx, &charts, 12, 314, 2)).collect();
        let txt = grid_text(&reads, None);
        println!(
            "# response1_L200_seed{k}: {} rows, {} oxygens, L = {:.2} bohr = {:.1} A ({:.1} s){}",
            w.frames,
            w.n,
            w.box_bohr,
            w.box_bohr * walk::BOHR_A_SHORT,
            t.elapsed().as_secs_f64(),
            txt
        );
        if k == 0 {
            let banked = std::fs::read_to_string(d.join("r1_closure_test.txt")).expect("banked r1_closure_test.txt");
            let banked_body: String = banked.lines().skip(1).map(|l| format!("{l}\n")).collect();
            let (same, total, diffs) = compare_lines(&txt, &banked_body);
            let ok = diffs.is_empty();
            println!("PR-5: {same} of {total} lines of the per-arm table identical to the banked r1_closure_test.txt (to the printed digit) -> {}", if ok { "PASS" } else { "FAIL" });
            for (a, b) in diffs.iter().take(10) {
                println!("   engine: {a}\n   banked: {b}");
            }
            out.plants_ok.push(("PR-5".into(), ok));
        }
        arms.push(reads);
    }
    // the pool
    let pooled: Vec<GridRead> = (0..3).map(|g| staggered::pool(&[&arms[0][g], &arms[1][g], &arms[2][g]])).collect();
    let n = pooled[0].rows[0].cycles.len();
    let ptxt = grid_text(&pooled, Some(n));
    println!("\n# POOLED over 3 arms (each arm's own signs, equal weight per cycle){ptxt}");
    let banked = std::fs::read_to_string(walks.join("r1pp_pooled_L200.txt")).expect("banked r1pp_pooled_L200.txt");
    let from = banked.find("\n== POOLED 4 cells").expect("pooled section");
    let to = banked.find("\n# R1'' on the pool").expect("R1'' line");
    let (same, total, diffs) = compare_lines(&ptxt, &banked[from..to]);
    println!("   pooled tables: {same} of {total} lines identical to the banked r1pp_pooled_L200.txt");
    for (a, b) in diffs.iter().take(10) {
        println!("   engine: {a}\n   banked: {b}");
    }
    // floors, reported
    for g in &pooled {
        for r in &g.rows {
            if matches!(r.chart, FluidChart::Cell { lag: 0 }) || r.chart == FluidChart::OF_RECORD {
                let (s, floor) = staggered::two_sided_floor(r, g.nx, g.box_bohr);
                println!(
                    "   floor, {} cells, {:18}: D_lead {:.3} against the two-sided floor {:.3} (s {:.2}; D_disc {:.4}) -> excess {:+.3}",
                    g.nx,
                    r.chart.label(),
                    r.d_lead,
                    floor,
                    s,
                    r.chart.spatial_floor(g.nx, g.box_bohr),
                    r.d_lead - floor
                );
            }
        }
    }
    let g8 = &pooled[1];
    let row = g8.rows.iter().find(|r| r.chart == FluidChart::OF_RECORD).expect("the chart of record");
    let cell = g8.rows.iter().find(|r| r.chart == FluidChart::Cell { lag: 0 }).unwrap();
    let (se, sed) = staggered::jackknife(row);
    println!(
        "# R1'' from the engine (face chart h = 0.25 A, 8 cells, integral form, {n} cycles): D_lead = {:.3} (jackknife SE {se:.3}), D_tail = {:.3}, alpha = {:.2}, R2 = {:.2}; |D_lead - D_tail| = {:.3} (SE {sed:.3}); the cell chart on the same pool {:.3}",
        row.d_lead,
        row.d_tail,
        row.alpha,
        row.r2,
        (row.d_lead - row.d_tail).abs(),
        cell.d_lead
    );
    let port = (row.d_lead - 0.180).abs();
    let ab = (0.8..=1.25).contains(&row.alpha);
    let verdict = if port > 0.01 || row.d_lead > 0.3 {
        "KILL"
    } else if row.d_lead <= 0.2 && ab {
        "MET"
    } else {
        "BETWEEN"
    };
    out.row(
        "G1 the chart",
        "36-cycle pooled D <= 0.2, alpha in [0.8, 1.25], engine = banked 0.180 to 0.01",
        format!(
            "D = {:.4} (banked 0.180, |diff| {:.4}), SE {se:.3} (banked 0.058), alpha {:.3}, R2 {:.2}; cell chart {:.3} (banked 0.310)",
            row.d_lead, port, row.alpha, row.r2, cell.d_lead
        ),
        verdict,
    );
}

// ---------------------------------------------------------------------------------------------
// The fluid gate's pieces
// ---------------------------------------------------------------------------------------------

fn fluid_q(lag: usize) -> Question {
    Question { lag, horizon: Horizon::Future, folds: Folds::TimeBlocks { k: 5 }, ridge: Ridge::ORDER1 }
}

fn chains_of(f: &OrderFeatures) -> Vec<Mat> {
    walk::order_chains(&f.h, &f.fields)
}

fn rd(ps: f64, dt_fs: f64) -> usize {
    ((ps * 1000.0 / dt_fs).round() as usize).max(1)
}

/// One candidate's admission against `kept`, with the time-shifted null and the donor swap.
#[allow(clippy::too_many_arguments)] // the gate's question, spelled out at every call site
fn admit_one(chains: &[Mat], donor: Option<&[Mat]>, kept: &[usize], target: &[usize], name: &str, cols: &[usize], lag: usize, budget: f64) -> Admission {
    let us = walk::units(chains, kept, target);
    let c = Candidate::new(name, walk::cand(chains, cols));
    let swap = donor.map(|d| Swap::Donor(walk::cand(d, cols)));
    admit(&us, &[c], &fluid_q(lag), &Nulls { shift: true, swap }, budget)
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map_or("-".into(), |x| format!("{x:+.4}"))
}

// ---------------------------------------------------------------------------------------------
// G2 — the drops
// ---------------------------------------------------------------------------------------------

/// ORDER-1's banked O2 read under Amendment 1 (`replace0/order1/order1_read_amend1.txt`):
/// per 293 K seed, (increment, null time-shifted, null seed-swapped) at 1 ps and 5 ps.
const ORDER1_BANKED: [[(f64, f64, f64); 2]; 3] = [
    [(-0.0032, -0.0008, -0.0001), (-0.0126, -0.0148, -0.0066)],
    [(-0.0007, 0.0001, -0.0035), (-0.0049, -0.0022, 0.0049)],
    [(-0.0033, -0.0047, -0.0026), (-0.0089, -0.0144, -0.0008)],
];

fn g2(feats: &[(String, OrderFeatures)], walks: &Path, hbond: bool, out: &mut Out) {
    println!("\n## G2 — the drops: target (rho_k, jL_k) at the lag, kept (rho, jL), budget {BETA}; ORDER-1's statistic (5 time blocks, ridge 1e-3 N, standardised target, pooled)");
    let ch: Vec<Vec<Mat>> = feats.iter().map(|(_, f)| chains_of(f)).collect();
    let mut worst_diff = 0.0f64;
    let mut all_dropped = true;
    let mut max_inc = f64::NEG_INFINITY;
    let mut max_null = f64::NEG_INFINITY;
    let mut q_line = Vec::new();
    let mut s_line = Vec::new();
    for i in 0..3 {
        let donor = &ch[(i + 1) % 3];
        for (li, ps) in [1.0, 5.0].iter().enumerate() {
            let l = rd(*ps, feats[i].1.dt_fs);
            let mut blocks: Vec<(String, Vec<usize>)> =
                vec![("Q (ORDER-1's column)".into(), C_Q.to_vec()), ("structural block".into(), C_STRUCT.to_vec())];
            for (f, name) in STRUCT_FIELDS.iter().enumerate().skip(1) {
                blocks.push((format!("  {name} alone"), vec![8 + 2 * f, 9 + 2 * f]));
            }
            for (name, cols) in &blocks {
                let adm = admit_one(&ch[i], Some(donor), &C_LS, &C_LS, name, cols, l, BETA);
                let p = &adm.prices[0];
                let graded = !name.starts_with("  ");
                let mut extra = String::new();
                if name.starts_with("Q ") {
                    let b = ORDER1_BANKED[i][li];
                    let d = (p.increment - b.0).abs().max((p.null_shift.unwrap() - b.1).abs()).max((p.null_swap.unwrap() - b.2).abs());
                    worst_diff = worst_diff.max(d);
                    extra = format!(" | banked {:+.4} / {:+.4} / {:+.4}, worst |diff| {d:.4}", b.0, b.1, b.2);
                    q_line.push(format!("{:+.4}", p.increment));
                }
                if name == "structural block" {
                    s_line.push(format!("{:+.4}", p.increment));
                }
                if graded {
                    all_dropped &= p.verdict == Verdict::Dropped;
                    max_inc = max_inc.max(p.increment);
                    max_null = max_null.max(p.null_max().unwrap());
                }
                println!(
                    "   {} {ps:>3} ps ({l:3} rd) {:28} R2 {:.4} -> {:.4}, increment {:+.4} +/- {:.4} | null shifted {} swapped {} -> {:?}{extra}",
                    feats[i].0,
                    name,
                    p.r2_kept,
                    p.r2_with,
                    p.increment,
                    p.se,
                    fmt_opt(p.null_shift),
                    fmt_opt(p.null_swap),
                    p.verdict
                );
            }
        }
    }
    // the 400 K arm, reported
    let (n4, f4) = &feats[3];
    for ps in [1.0, 5.0] {
        let l = rd(ps, f4.dt_fs);
        let c4 = chains_of(f4);
        let adm = admit_one(&c4, None, &C_LS, &C_LS, "Q", &C_Q, l, BETA);
        let p = &adm.prices[0];
        println!("   {n4} {ps:>3} ps (reported) Q: increment {:+.4}, null shifted {} -> {:?} (ORDER-1 banked {})", p.increment, fmt_opt(p.null_shift), p.verdict, if ps == 1.0 { "-0.0004" } else { "+0.0001" });
    }
    // the network block on HBOND's own carriers
    let mut net_line = Vec::new();
    let mut net_ok = true;
    if hbond {
        println!("\n   the NETWORK block (H-bond counts donated/accepted) cannot be built on the SLOW-1 walks, which carry oxygens only;");
        println!("   it is read on HBOND-SEARCH-1's own all-atom carriers against HBOND's own target (Delta vcom over 50 fs, kept vcom, molecule folds, HBOND's ridge):");
        for (dir, banked) in [("molsearch1", -0.000), ("molsearch1_warm_396K", -0.001), ("molsearch2_5ps", 0.000)] {
            let t = Instant::now();
            let aw = AtomWalk::load(&walks.join(dir)).expect("all-atom walk");
            let f = walk::hbond_features(&aw);
            let lag = ((50.0 / aw.dt_fs).round() as usize).max(1);
            let us: Vec<Unit> = f
                .iter()
                .map(|m| {
                    let mm = Mat { rows: aw.frames, cols: 5, data: m.clone() };
                    Unit { kept: mm.select_cols(&[2, 3, 4]), target: mm.select_cols(&[2, 3, 4]) }
                })
                .collect();
            let cand: Vec<Mat> = f.iter().map(|m| Mat { rows: aw.frames, cols: 5, data: m.clone() }.select_cols(&[0, 1])).collect();
            let q = Question { lag, horizon: Horizon::Increment, folds: Folds::Units { k: 4 }, ridge: Ridge::HBOND };
            let n = us.len();
            let adm = admit(&us, &[Candidate::new("network block", cand)], &q, &Nulls { shift: true, swap: Some(Swap::Derange { shift: n / 2 }) }, BETA);
            let p = &adm.prices[0];
            let bonds = f.iter().map(|m| (0..aw.frames).map(|t| m[t * 5]).sum::<f64>()).sum::<f64>() / (n * aw.frames) as f64;
            let d = (p.increment - banked).abs();
            net_ok &= d <= 0.005 && p.verdict == Verdict::Dropped && p.increment <= 0.01 && p.null_max().unwrap() <= 0.01;
            max_inc = max_inc.max(p.increment);
            max_null = max_null.max(p.null_max().unwrap());
            all_dropped &= p.verdict == Verdict::Dropped;
            worst_diff = worst_diff.max(d);
            net_line.push(format!("{:+.4}", p.increment));
            println!(
                "   {dir}: {n} molecules x {} rows at {:.2} fs, mean bonds donated {bonds:.2}; R2 velocity alone {:.4}, + bonds {:.4}, increment {:+.4} +/- {:.4} (banked {banked:+.3}, |diff| {d:.4}) | null shifted {} swapped {} -> {:?} ({:.1} s)",
                aw.frames,
                aw.dt_fs,
                p.r2_kept,
                p.r2_with,
                p.increment,
                p.se,
                fmt_opt(p.null_shift),
                fmt_opt(p.null_swap),
                p.verdict,
                t.elapsed().as_secs_f64()
            );
        }
    }
    let ok = all_dropped && max_inc <= 0.01 && max_null <= 0.01 && worst_diff <= 0.005 && net_ok;
    out.row(
        "G2 the drops",
        "structural and network blocks DROPPED at beta 0.02, increments and nulls <= 0.01, = ORDER-1 / HBOND to 0.005",
        format!(
            "Q increments (s0 1,5 ps; s1; s2) [{}]; structural block [{}]; network block [{}]; max increment {max_inc:+.4}, max null {max_null:+.4}; worst |engine - banked| {worst_diff:.4}",
            q_line.join(", "),
            s_line.join(", "),
            net_line.join(", ")
        ),
        if ok { "MET" } else if !all_dropped || worst_diff > 0.005 { "KILL" } else { "BETWEEN" },
    );
}

// ---------------------------------------------------------------------------------------------
// G3 — the keeps
// ---------------------------------------------------------------------------------------------

fn g3(feats: &[(String, OrderFeatures)], out: &mut Out) -> Vec<(String, Admission, Vec<Mat>)> {
    println!("\n## G3 (fluid) — the hydrodynamic block (rho, jL) as a CANDIDATE against its own next state, the structural block kept; graded at 1 ps, 0.1 and 5 ps reported");
    let ch: Vec<Vec<Mat>> = feats.iter().map(|(_, f)| chains_of(f)).collect();
    let mut keep = Vec::new();
    let mut graded_ok = true;
    let mut line = Vec::new();
    for i in 0..3 {
        for ps in [0.1, 1.0, 5.0] {
            let l = rd(ps, feats[i].1.dt_fs);
            let adm = admit_one(&ch[i], Some(&ch[(i + 1) % 3]), &C_STRUCT, &C_LS, "hydrodynamic block", &C_LS, l, BETA);
            let p = &adm.prices[0];
            println!(
                "   {} {ps:>3} ps ({l:3} rd): R2 structural alone {:.4} -> + (rho, jL) {:.4}, increment {:+.4} +/- {:.4} | null shifted {} swapped {} -> {:?}{}",
                feats[i].0,
                p.r2_kept,
                p.r2_with,
                p.increment,
                p.se,
                fmt_opt(p.null_shift),
                fmt_opt(p.null_swap),
                p.verdict,
                if ps == 1.0 { "  [graded]" } else { "" }
            );
            if ps == 1.0 {
                graded_ok &= p.verdict == Verdict::Carried;
                line.push(format!("{:+.4}", p.increment));
                keep.push((feats[i].0.clone(), adm.clone(), ch[i].clone()));
            }
        }
    }
    // Diagnostics, reported and not graded (added after the graded read above came in DROPPED):
    // which kept column makes (rho, jL) removable at 1 ps? The structural block holds three
    // coarse DENSITY COUNTS (n33, n50, nb), each a second chart of the density mode.
    println!("   diagnostics at 1 ps (reported, not graded): the hydrodynamic block's increment with other kept sets");
    for i in 0..3 {
        let l = rd(1.0, feats[i].1.dt_fs);
        let mut parts = Vec::new();
        for (label, kept) in [
            ("nothing kept", vec![]),
            ("Q kept", C_Q.to_vec()),
            ("Q+s2 kept", vec![8, 9, 10, 11]),
            ("n33+n50+nb kept", (12..18).collect::<Vec<usize>>()),
        ] {
            let adm = admit_one(&ch[i], None, &kept, &C_LS, "hydrodynamic block", &C_LS, l, BETA);
            let p = &adm.prices[0];
            parts.push(format!("{label}: {:+.4} (R2 kept {:.4}) {:?}", p.increment, p.r2_kept, p.verdict));
        }
        println!("      {}: {}", feats[i].0, parts.join(" | "));
    }
    out.row(
        "G3a the keeps (fluid)",
        "the hydrodynamic block CARRIED against its own next state, the others kept",
        format!("increments at 1 ps per 293 K seed [{}] against beta {BETA}", line.join(", ")),
        if graded_ok { "MET" } else { "KILL" },
    );
    keep
}

fn g3_chains(path: &str, out: &mut Out) {
    println!("\n## G3 (chains) — the conscience (DMA) block on reason_search0b's parent->child transitions");
    let t = Instant::now();
    let ch = match chains::load(path) {
        Ok(c) => c,
        Err(e) => {
            println!("   chains unreadable ({e}): G3's second half NOT READ");
            out.row("G3b the keeps (chains)", "conscience block CARRIED; 8.9x above null to 10 %", "not read: the chain file is absent".into(), "NOT READ");
            return;
        }
    };
    let n_tr: usize = ch.iter().map(|c| c.len() - 1).sum();
    let mut hist = [0usize; 9];
    ch.iter().for_each(|c| hist[c.len().min(8)] += 1);
    println!("   {} chains, {n_tr} parent->child transitions ({:.1} s); length counts 2..8+: {:?}", ch.len(), t.elapsed().as_secs_f64(), &hist[2..]);
    let mats: Vec<Mat> = ch.iter().map(|c| Mat::from_rows(c)).collect();
    // the banked statistic: VAMP held-out score at full k against the cross-chain re-paired null
    const DMA: [usize; 4] = [0, 1, 2, 3];
    const PROC: [usize; 3] = [4, 5, 6];
    let ho = |cols: &[usize], null: bool| -> f64 {
        let mut rng = NumpyPcg64::new(0);
        let mut acc = 0.0;
        for j in 0..4 {
            let pairs = |sel: &dyn Fn(usize) -> bool| -> (Mat, Mat) {
                let mut a = Vec::new();
                let mut b = Vec::new();
                for (i, m) in mats.iter().enumerate() {
                    if sel(i) {
                        for r in 0..m.rows - 1 {
                            a.push(cols.iter().map(|&c| m.at(r, c)).collect::<Vec<f64>>());
                            b.push(cols.iter().map(|&c| m.at(r + 1, c)).collect::<Vec<f64>>());
                        }
                    }
                }
                (Mat::from_rows(&a), Mat::from_rows(&b))
            };
            let (tra, mut trb) = pairs(&|i| i % 4 != j);
            let (tea, mut teb) = pairs(&|i| i % 4 == j);
            if null {
                trb = trb.permute_rows(&rng.permutation(trb.rows));
                teb = teb.permute_rows(&rng.permutation(teb.rows));
            }
            acc += removable::vamp_heldout_full(&tra, &trb, &tea, &teb);
        }
        acc / 4.0
    };
    let mut ratio_dma = f64::NAN;
    for (name, cols, banked) in [("depth alone", vec![4usize], (0.998, 0.010)), ("DMA sector", DMA.to_vec(), (1.576, 0.177)), ("process sector", PROC.to_vec(), (1.757, 0.096)), ("all", (0..7).collect(), (3.260, 0.530))] {
        let (real, nul) = (ho(&cols, false), ho(&cols, true));
        println!("   {name:16} k={}  held-out {real:.3}  re-paired null {nul:.3}  real/null {:.2}   (banked {:.3} / {:.3} = {:.2})", cols.len(), real / nul, banked.0, banked.1, banked.0 / banked.1);
        if name == "DMA sector" {
            ratio_dma = real / nul;
        }
    }
    // the prereg's literal form: target = the next thought's process sector, kept = the process sector
    let us: Vec<Unit> = mats.iter().map(|m| Unit { kept: m.select_cols(&PROC), target: m.select_cols(&PROC) }).collect();
    let cand: Vec<Mat> = mats.iter().map(|m| m.select_cols(&DMA)).collect();
    let q = Question { lag: 1, horizon: Horizon::Future, folds: Folds::Units { k: 4 }, ridge: Ridge::ORDER1 };
    let adm = admit(&us, &[Candidate::new("conscience (DMA) block", cand)], &q, &Nulls { shift: true, swap: Some(Swap::Permute { seed: 0 }) }, BETA);
    let p = &adm.prices[0];
    println!("   removable: target = next thought's process sector, kept = this thought's process sector: {p}");
    let within = (ratio_dma / 8.90 - 1.0).abs() <= 0.10;
    let ok = within && p.verdict == Verdict::Carried;
    out.row(
        "G3b the keeps (chains)",
        "conscience block CARRIED toward the next process sector; banked 8.9x above null to 10 %",
        format!(
            "DMA real/null {ratio_dma:.2} (banked 8.90, {:+.1} %); gate increment {:+.4} +/- {:.4}, nulls {} / {} -> {:?}",
            100.0 * (ratio_dma / 8.90 - 1.0),
            p.increment,
            p.se,
            fmt_opt(p.null_shift),
            fmt_opt(p.null_swap),
            p.verdict
        ),
        if ok { "MET" } else { "KILL" },
    );
}

// ---------------------------------------------------------------------------------------------
// G5 — refusal by price
// ---------------------------------------------------------------------------------------------

fn g5(keeps: &[(String, Admission, Vec<Mat>)], out: &mut Out) {
    println!("\n## G5 — refusal by price, on the hydrodynamic block's 1 ps admission");
    let mut ok = keeps.len() == 3;
    let mut worst = 0.0f64;
    for (name, adm, ch) in keeps {
        let p = &adm.prices[0];
        let refused = refuse_drop(adm, "hydrodynamic block", 0.0);
        match &refused {
            Err(r) => println!("   {name}: budget 0: {r}"),
            Ok(_) => println!("   {name}: budget 0: DROPPED - the refusal did not fire"),
        }
        let above = (p.increment * 2.0).max(0.05);
        let receipt = refuse_drop(adm, "hydrodynamic block", above);
        // the closure defect with and without the block, read independently
        let q = fluid_q(50);
        let full: Vec<Unit> = walk::units(ch, &[C_STRUCT.to_vec(), C_LS.to_vec()].concat(), &C_LS);
        let dropped: Vec<Unit> = walk::units(ch, &C_STRUCT, &C_LS);
        let d_full = 1.0 - removable::closure_r2(&full, &q);
        let d_drop = 1.0 - removable::closure_r2(&dropped, &q);
        let rise = d_drop - d_full;
        worst = worst.max((rise - p.increment).abs());
        match &receipt {
            Ok(r) => println!(
                "   {name}: budget {above:.4}: DROPPED at price {:+.4}; closure defect toward (rho, jL)(t + 1 ps) {d_full:.4} -> {d_drop:.4}, rise {rise:+.4} (the price {:+.4})",
                r.price, p.increment
            ),
            Err(r) => println!("   {name}: budget {above:.4}: {r} - should have dropped"),
        }
        ok &= refused.is_err() && receipt.is_ok();
    }
    ok &= worst <= 1e-9;
    out.row(
        "G5 refusal by price",
        "budget below the carried block's increment: REFUSED by name with the price; above it: dropped and the defect rises by the price",
        format!("refusal fired on {} of 3 seeds; worst |defect rise - price| {worst:.1e}", keeps.iter().filter(|(_, a, _)| refuse_drop(a, "hydrodynamic block", 0.0).is_err()).count()),
        if ok { "MET" } else { "KILL" },
    );
}

// ---------------------------------------------------------------------------------------------
// G4 — the circuit case is the same function
// ---------------------------------------------------------------------------------------------

fn g4(out: &mut Out) {
    println!("\n## G4 — holon::sector's light-cone removal through removable::admit_certified (budget {:e}), on QVM-ACUITY-1's grid (n 12/16/20, t 8/12/16, seeds 1-3, depth 20n; marginal on qubits 0-3)", sector::REMOVAL_BUDGET);
    let reference = |circuit: &[Gate], obs: &Observable| -> Vec<usize> {
        // the pre-REPLACE-1 body of sector::locate's removal, verbatim
        let n = sector::width(circuit, obs);
        let mut cone = vec![false; n];
        for q in obs.support(n) {
            cone[q] = true;
        }
        let mut keep = vec![false; circuit.len()];
        for i in (0..circuit.len()).rev() {
            let s = sector::support(circuit[i]);
            if s.iter().any(|&q| cone[q]) {
                for q in s {
                    cone[q] = true;
                }
                keep[i] = true;
            }
        }
        (0..circuit.len()).filter(|&i| !sector::closure_of(circuit[i]).is_closed() && !keep[i]).collect()
    };
    let (mut total, mut mismatches, mut worst, mut weakest_kept, mut moved) = (0usize, 0usize, 0.0f64, f64::INFINITY, 0usize);
    let mut instances = 0usize;
    let (mut moved_all, mut kept_total, mut kept_moving) = (0usize, 0usize, 0usize);
    for n in [12usize, 16, 20] {
        for t in [8usize, 12, 16] {
            for seed in [1u64, 2, 3] {
                let tm = Instant::now();
                let c = sector::random_instance_depth(n, t, seed, 20 * n);
                let gates = c.gates.clone();
                let obs = Observable::Marginal { qubits: vec![0, 1, 2, 3], bits: vec![false; 4] };
                let sec = sector::locate(&gates, &obs);
                let adm = sector::light_cone_admission(&gates, &obs);
                let got: Vec<usize> = sec.removed.iter().map(|(i, _)| *i).collect();
                let refr = reference(&gates, &obs);
                if got != refr {
                    mismatches += 1;
                }
                let from_gate: Vec<usize> = adm.dropped.iter().map(|s| s.parse().unwrap()).collect();
                if from_gate != got {
                    mismatches += 1;
                }
                total += got.len();
                // the referee: every removed gate dropped on its own moves the marginal by <= 1e-12
                let full = referee::statevector(n, &gates);
                let want = referee::marginal(&full, &[0, 1, 2, 3], &[false; 4]);
                let mut w_inst = 0.0f64;
                for &k in &got {
                    let mut cut = gates.clone();
                    cut.remove(k);
                    let dm = (referee::marginal(&referee::statevector(n, &cut), &[0, 1, 2, 3], &[false; 4]) - want).abs();
                    w_inst = w_inst.max(dm);
                }
                let red = sec.drop_removed(&gates);
                w_inst = w_inst.max((referee::marginal(&referee::statevector(n, &red), &[0, 1, 2, 3], &[false; 4]) - want).abs());
                worst = worst.max(w_inst);
                // the negative check on the amplitude: the best kept T moves it by > 1e-3
                let y = sector::argmax_bitstring(n, &gates);
                let a0 = referee::amplitude(&full, &y);
                // ACUITY-1's own method: the best kept T, trying every kept T below n = 20 and the
                // first four at n = 20 (its test's budget); and beside it every kept T at every n
                let mut best = 0.0f64;
                let mut best_all = 0.0f64;
                let budget = if n >= 20 { 4 } else { usize::MAX };
                let mut tried = 0;
                for (k, g) in gates.iter().enumerate() {
                    if !g.is_t() || got.contains(&k) {
                        continue;
                    }
                    let mut cut = gates.clone();
                    cut.remove(k);
                    let a = referee::amplitude(&referee::statevector(n, &cut), &y);
                    let mv = ((a.0 - a0.0).powi(2) + (a.1 - a0.1).powi(2)).sqrt();
                    if tried < budget {
                        best = best.max(mv);
                    }
                    tried += 1;
                    best_all = best_all.max(mv);
                    kept_total += 1;
                    if mv > 1e-3 {
                        kept_moving += 1;
                    }
                }
                weakest_kept = weakest_kept.min(best);
                if best > 1e-3 {
                    moved += 1;
                }
                if best_all > 1e-3 {
                    moved_all += 1;
                }
                instances += 1;
                println!(
                    "   n {n:2} t {t:2} seed {seed}: removed {:2} (reference {:2}, gate {:2}) | worst change dropping removed {w_inst:.2e} | best kept-T amplitude move, ACUITY-1's budget {best:.2e}, every kept T {best_all:.2e} ({:.1} s)",
                    got.len(),
                    refr.len(),
                    from_gate.len(),
                    tm.elapsed().as_secs_f64()
                );
            }
        }
    }
    // Graded on the kill (the removed set) and the numbers it reproduces; the kept-gate clause is
    // reported under both methods (declared before the every-kept-T read was run).
    let ok = mismatches == 0 && worst <= 1e-12 && total == 76;
    out.row(
        "G4 the circuit case",
        "removed set identical (kill: any difference); 76 removed, worst change 2.6e-15; kept gates move the amplitude > 1e-3",
        format!(
            "{instances} instances, {total} removed (banked 76), removed-set differences {mismatches}, worst change {worst:.2e} (banked 2.6e-15); best kept T moves the amplitude > 1e-3 on {moved} of {instances} under ACUITY-1's n = 20 budget of four (weakest {weakest_kept:.2e}), on {moved_all} of {instances} trying every kept T; {kept_moving} of {kept_total} kept T gates individually > 1e-3"
        ),
        if ok { "MET" } else if mismatches > 0 { "KILL" } else { "BETWEEN" },
    );
}

// ---------------------------------------------------------------------------------------------
// The plants
// ---------------------------------------------------------------------------------------------

fn plants(feats: &[(String, OrderFeatures)], out: &mut Out) {
    let (name, f) = &feats[0];
    println!("\n## Plants on the carrier of record ({name}); target (rho, jL), kept (rho, jL), budget {BETA}");
    let dt = f.dt_fs;
    let (l1, l5) = (rd(1.0, dt), rd(5.0, dt));
    // PR-1: the PO-4 field at unit amplitude (ORDER-1 Amendment 1), 5 ps memory, 1 ps latency
    let (hp, qp) = walk::plant_po4(&f.h, &f.fields[0], 5000.0 / dt, rd(1.0, dt), 0.05, 1.0, 5);
    let mut fields = vec![qp.clone()];
    fields.extend(f.fields.iter().cloned());
    let ch = walk::order_chains(&hp, &fields); // [H(8) | planted(2) | Q S2 N33 N50 NB (10)]
    let planted = [8usize, 9];
    let mut pr1 = false;
    let mut pr1_line = Vec::new();
    let mut joint = true;
    for l in [l1, l5] {
        let us = walk::units(&ch, &C_LS, &C_LS);
        let c = Candidate::new("planted", walk::cand(&ch, &planted));
        let adm = admit(&us, &[c], &fluid_q(l), &Nulls { shift: true, swap: Some(Swap::Derange { shift: 2 }) }, BETA);
        let p = &adm.prices[0];
        let fires = p.increment >= 0.05 && p.null_max().unwrap() <= 0.01;
        pr1 |= fires;
        if l == l1 {
            // M-JOINT-PASS-REGION: the same plant passes G2's admission rule and G5's refusal
            joint = p.verdict == Verdict::Carried && refuse_drop(&adm, "planted", 0.0).is_err();
        }
        pr1_line.push(format!("{:.0} ps {:+.4} (nulls {} / {})", l as f64 * dt / 1000.0, p.increment, fmt_opt(p.null_shift), fmt_opt(p.null_swap)));
        println!("   PR-1 at {:.0} ps: {p}", l as f64 * dt / 1000.0);
    }
    println!("   PR-1: ADMITTED with increment >= 0.05 and nulls <= 0.01 at one lag -> {}; joint pass region (carried at beta {BETA} AND refused at beta 0, 1 ps) -> {}", if pr1 { "PASS" } else { "FAIL" }, if joint { "exhibited" } else { "NOT exhibited" });
    out.plants_ok.push(("PR-1".into(), pr1 && joint));
    // PR-2: the same field from another k-vector's chain (partner swap)
    let mut pr2 = true;
    for l in [l1, l5] {
        let us = walk::units(&ch, &C_LS, &C_LS);
        let sw: Vec<Mat> = (0..ch.len()).map(|u| ch[(u + 2) % ch.len()].select_cols(&planted)).collect();
        let adm = admit(&us, &[Candidate::new("planted, partner-swapped", sw)], &fluid_q(l), &Nulls { shift: true, swap: None }, BETA);
        let p = &adm.prices[0];
        pr2 &= p.verdict == Verdict::Dropped && p.increment <= 0.01;
        println!("   PR-2 at {:.0} ps: {p}", l as f64 * dt / 1000.0);
    }
    println!("   PR-2: DROPPED with increment <= 0.01 -> {}", if pr2 { "PASS" } else { "FAIL" });
    out.plants_ok.push(("PR-2".into(), pr2));
    // PR-3: a linear copy of a kept column
    let mut pr3 = true;
    for l in [l1, l5] {
        let us = walk::units(&ch, &C_LS, &C_LS);
        let cp: Vec<Mat> = ch
            .iter()
            .map(|c| {
                let v: Vec<Vec<f64>> = (0..c.rows).map(|t| vec![2.0 * c.at(t, 0) - 3.0, 0.5 * c.at(t, 1) - 2.0 * c.at(t, 3) + 1.0]).collect();
                Mat::from_rows(&v)
            })
            .collect();
        let adm = admit(&us, &[Candidate::new("linear copy of kept columns", cp)], &fluid_q(l), &Nulls { shift: false, swap: None }, BETA);
        let p = &adm.prices[0];
        pr3 &= p.verdict == Verdict::Dropped && (0.0..=1e-10).contains(&p.increment);
        println!("   PR-3 at {:.0} ps: {p}", l as f64 * dt / 1000.0);
    }
    println!("   PR-3: DROPPED with increment in [0, 1e-10] -> {}", if pr3 { "PASS" } else { "FAIL" });
    out.plants_ok.push(("PR-3".into(), pr3));
    // PR-4: the time-shuffled dictionary (one permutation of the readouts for every chain)
    let perm = NumpyPcg64::new(11).permutation(ch[0].rows);
    let sh: Vec<Mat> = ch.iter().map(|c| c.permute_rows(&perm)).collect();
    let mut pr4 = true;
    for l in [l1, l5] {
        let cands = [("planted", planted.to_vec()), ("Q", vec![10, 11]), ("structural block", (10..20).collect::<Vec<usize>>())];
        for (nm, cols) in &cands {
            let us = walk::units(&sh, &C_LS, &C_LS);
            let adm = admit(&us, &[Candidate::new(nm, walk::cand(&sh, cols))], &fluid_q(l), &Nulls { shift: true, swap: None }, BETA);
            let p = &adm.prices[0];
            pr4 &= adm.carried.is_empty() && p.increment <= 0.01;
            println!("   PR-4 at {:.0} ps: {p}", l as f64 * dt / 1000.0);
        }
        let us = walk::units(&sh, &(10..20).collect::<Vec<usize>>(), &C_LS);
        let adm = admit(&us, &[Candidate::new("hydrodynamic block", walk::cand(&sh, &C_LS))], &fluid_q(l), &Nulls { shift: true, swap: None }, BETA);
        let p = &adm.prices[0];
        pr4 &= adm.carried.is_empty() && p.increment <= 0.01;
        println!("   PR-4 at {:.0} ps: {p}", l as f64 * dt / 1000.0);
    }
    println!("   PR-4: the time-shuffled dictionary admits nothing (<= 0.01 on every candidate) -> {}", if pr4 { "PASS" } else { "FAIL" });
    out.plants_ok.push(("PR-4".into(), pr4));
    out.row("PR-1..4 plants", "see the plant lines", format!("PR-1 {}", pr1_line.join("; ")), if pr1 && joint && pr2 && pr3 && pr4 { "PLANTS PASS" } else { "PLANT FAILS" });
}

// ---------------------------------------------------------------------------------------------
// The re-read under Amendment 1: ORDERED admission (backward elimination with a joint price).
// Run with `--only a1`; it prints its own verdict table and branch.
// ---------------------------------------------------------------------------------------------

fn order_line(a: &Admission) -> String {
    let steps: Vec<String> = a
        .order
        .iter()
        .map(|s| format!("remove '{}' at {:+.4} (nulls {} / {})", s.price.name, s.price.increment, fmt_opt(s.price.null_shift), fmt_opt(s.price.null_swap)))
        .collect();
    let surv: Vec<String> = a
        .prices
        .iter()
        .skip(a.order.len())
        .map(|p| format!("'{}' survives, removal price {:+.4} +/- {:.4} (nulls {} / {}) {:?}", p.name, p.increment, p.se, fmt_opt(p.null_shift), fmt_opt(p.null_swap), p.verdict))
        .collect();
    let sep = if steps.is_empty() || surv.is_empty() { "" } else { " | " };
    format!("{}{sep}{}", steps.join("; "), surv.join("; "))
}

fn empty_base(chains: &[Mat], target: &[usize]) -> Vec<Unit> {
    chains.iter().map(|c| Unit { kept: Mat::zeros(c.rows, 0), target: c.select_cols(target) }).collect()
}

fn fluid_dict(ch: &[Mat]) -> Vec<Candidate> {
    let counts: Vec<usize> = (12..18).collect();
    let qs2: Vec<usize> = (8..12).collect();
    vec![
        Candidate::new("hydrodynamic (rho, jL)", walk::cand(ch, &C_LS)),
        Candidate::new("count block (n33, n50, nb)", walk::cand(ch, &counts)),
        Candidate::new("structural (q, s2)", walk::cand(ch, &qs2)),
    ]
}

fn marginal_line(m: &Admission) -> String {
    m.prices.iter().map(|p| format!("'{}' {:+.4} {:?}", p.name, p.increment, p.verdict)).collect::<Vec<_>>().join("; ")
}

fn amend1(feats: &[(String, OrderFeatures)], chains_path: &str) {
    println!("\n## RE-READ UNDER AMENDMENT 1 — admission is ORDERED (backward elimination with a joint price)");
    let mut rows: Vec<(String, String, String)> = Vec::new();
    let nulls = Nulls { shift: true, swap: Some(Swap::Derange { shift: 2 }) };
    let hc = ["hydrodynamic (rho, jL)", "count block (n33, n50, nb)"];
    // G3a
    println!("   G3a: SLOW-1 293 K, target (rho_k, jL_k) at 1 ps, beta {BETA}, nothing kept outside the dictionary; declared order: hydrodynamic, count block, (q, s2); nulls: time-shifted, and the chain two over (another axis)");
    let (mut g3a_ok, mut g3a_kill) = (true, false);
    let mut g3a_line = Vec::new();
    let mut seed0: Option<(Vec<Mat>, Admission)> = None;
    for (name, f) in feats.iter().take(3) {
        let ch = chains_of(f);
        let q = fluid_q(rd(1.0, f.dt_fs));
        let base = empty_base(&ch, &C_LS);
        let dict = fluid_dict(&ch);
        let o = Admission::ordered(&base, &dict, &q, &nulls, BETA);
        let m = Admission::marginal(&base, &dict, &q, &nulls, BETA);
        println!("   {name} ORDERED: {}", order_line(&o));
        println!("   {name} marginal (the record): {}", marginal_line(&m));
        let surv: Vec<&String> = o.carried.iter().filter(|c| hc.contains(&c.as_str())).collect();
        let mut refused_ok = !surv.is_empty();
        for c in &surv {
            match refuse_drop(&o, c, BETA) {
                Err(r) => {
                    println!("   {name}: {r}");
                    refused_ok &= r.price >= 0.04;
                }
                Ok(_) => refused_ok = false,
            }
        }
        let qs2_step = o.order.iter().find(|s| s.price.name == "structural (q, s2)").map(|s| s.price.increment);
        g3a_kill |= surv.is_empty();
        g3a_ok &= refused_ok && qs2_step.is_some_and(|v| v <= 0.01);
        g3a_line.push(format!(
            "{name}: survivors {:?}, (q, s2) removed at {}",
            o.carried,
            qs2_step.map_or("never".into(), |v| format!("{v:+.4}"))
        ));
        if seed0.is_none() {
            seed0 = Some((ch, o));
        }
    }
    let g3a_v = if g3a_kill { "KILL" } else if g3a_ok { "MET" } else { "BETWEEN (not killed; a bar missed)" };
    rows.push(("G3a (ordered)".into(), g3a_line.join(" | "), g3a_v.into()));
    // G3b
    let (mut g3b_line, mut g3b_v) = ("chains unreadable".to_string(), "NOT READ".to_string());
    if let Ok(ch) = chains::load(chains_path) {
        let mats: Vec<Mat> = ch.iter().map(|c| Mat::from_rows(c)).collect();
        let base: Vec<Unit> = mats.iter().map(|m| Unit { kept: Mat::zeros(m.rows, 0), target: m.select_cols(&[4, 5, 6]) }).collect();
        let dict = vec![
            Candidate::new("process sector", mats.iter().map(|m| m.select_cols(&[4, 5, 6])).collect()),
            Candidate::new("conscience (DMA) block", mats.iter().map(|m| m.select_cols(&[0, 1, 2, 3])).collect()),
        ];
        let q = Question { lag: 1, horizon: Horizon::Future, folds: Folds::Units { k: 4 }, ridge: Ridge::ORDER1 };
        let nn = Nulls { shift: false, swap: Some(Swap::Permute { seed: 0 }) };
        let o = Admission::ordered(&base, &dict, &q, &nn, 0.01);
        let m = Admission::marginal(&base, &dict, &q, &nn, 0.01);
        println!("   G3b: {} chains, target = the next thought's process sector, lag 1 thought, beta 0.01, the partner-swapped (permutation) null only; declared order: process sector, conscience block", mats.len());
        println!("   G3b ORDERED: {}", order_line(&o));
        println!("   G3b marginal (the record): {}", marginal_line(&m));
        let p = o.price("conscience (DMA) block").expect("priced");
        let removed = o.order.iter().any(|s| s.price.name == "conscience (DMA) block");
        let null = p.null_swap.expect("swap null");
        g3b_line = format!(
            "conscience block {} with removal price {:+.4} +/- {:.4}, null {:+.4} (bars: price >= 0.02, null <= 0.005)",
            if removed { "REMOVED" } else { "survives" },
            p.increment,
            p.se,
            null
        );
        g3b_v = if removed {
            "KILL".into()
        } else if p.increment >= 0.02 && null <= 0.005 {
            "MET".into()
        } else {
            "BETWEEN (survives; a bar missed)".into()
        };
    }
    rows.push(("G3b (ordered)".into(), g3b_line, g3b_v));
    // PR-6 on the carrier of record
    let (name0, f0) = &feats[0];
    let dt = f0.dt_fs;
    let (hp, qp) = walk::plant_po4(&f0.h, &f0.fields[0], 5000.0 / dt, rd(1.0, dt), 0.05, 1.0, 5);
    let pch = walk::order_chains(&hp, &[qp]);
    let base = walk::units(&pch, &C_LS, &C_LS);
    let a = walk::cand(&pch, &[8, 9]);
    let b: Vec<Mat> = a.iter().map(|m| Mat { rows: m.rows, cols: m.cols, data: m.data.iter().map(|v| 2.0 * v - 1.0).collect() }).collect();
    let dict = vec![Candidate::new("planted A", a.clone()), Candidate::new("planted B = 2A - 1", b)];
    let q1 = fluid_q(rd(1.0, dt));
    let o = Admission::ordered(&base, &dict, &q1, &nulls, BETA);
    let m = Admission::marginal(&base, &dict, &q1, &nulls, BETA);
    let single = admit(&base, &[Candidate::new("planted", a)], &q1, &nulls, BETA).prices[0].increment;
    let kept_price = o.carried.first().and_then(|c| o.price(c)).map_or(f64::NAN, |p| p.increment);
    let pr6 = o.carried.len() == 1 && (kept_price - single).abs() < 1e-12 && m.carried.is_empty();
    println!(
        "   PR-6 on {name0} (kept (rho, jL); two exact copies of the PO-4 field): ORDERED {} | marginal {} | the single column's price {single:+.4} -> {}",
        order_line(&o),
        marginal_line(&m),
        if pr6 { "PASS" } else { "FAIL" }
    );
    // PR-7: determinism, and independence of the declared order, on G3a's seed-0 dictionary
    let (ch0, o0) = seed0.expect("seed 0 read");
    let base0 = empty_base(&ch0, &C_LS);
    let q = fluid_q(rd(1.0, dt));
    let again = Admission::ordered(&base0, &fluid_dict(&ch0), &q, &nulls, BETA);
    let mut rev = fluid_dict(&ch0);
    rev.reverse();
    let reversed = Admission::ordered(&base0, &rev, &q, &nulls, BETA);
    let mut rot = fluid_dict(&ch0);
    rot.rotate_left(1);
    let rotated = Admission::ordered(&base0, &rot, &q, &nulls, BETA);
    let names = |a: &Admission| (a.order.iter().map(|s| s.price.name.clone()).collect::<Vec<_>>(), a.carried.clone());
    let same_prices = |a: &Admission| a.order.iter().zip(&o0.order).all(|(x, y)| (x.price.increment - y.price.increment).abs() < 1e-12);
    let pr7 = again == o0 && names(&reversed) == names(&o0) && names(&rotated) == names(&o0) && same_prices(&reversed) && same_prices(&rotated);
    println!(
        "   PR-7 on {name0}: re-run bit-identical {}; removal order and survivors, declared {:?}, reversed {:?}, rotated {:?}; step prices equal to 1e-12 across the three -> {}",
        again == o0,
        names(&o0),
        names(&reversed),
        names(&rotated),
        if pr7 { "PASS" } else { "FAIL" }
    );
    println!("\n# VERDICT TABLE UNDER AMENDMENT 1");
    for (st, r, v) in &rows {
        println!("{st} | read: {r} -> {v}");
    }
    println!("plants: PR-6 {}, PR-7 {}", if pr6 { "PASS" } else { "FAIL" }, if pr7 { "PASS" } else { "FAIL" });
    let branch = if !(pr6 && pr7) {
        "(e) a plant fails".to_string()
    } else if rows.iter().any(|r| r.2.starts_with("KILL")) {
        "(c) G3 still killed".to_string()
    } else if rows.iter().all(|r| r.2 == "MET") {
        "(a) G3 met under ordered admission, PR-6 and PR-7 firing".to_string()
    } else {
        format!("none of (a)/(c)/(e) cleanly: G3a {}, G3b {}", rows[0].2, rows[1].2)
    };
    println!("BRANCH (Amendment 1): {branch}");
}
