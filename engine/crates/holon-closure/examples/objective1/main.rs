//! OBJECTIVE-1's driver — is the carried set of ordered admission invariant across the
//! observer's budget and cadence? (`conformance/replace1/OBJECTIVE1_PREREG.md`).
//!
//! ```text
//! taskset -c 18-20 cargo run --release -p holon-closure --example objective1 -- [--walks DIR] [--chains FILE]
//! ```
//!
//! The dictionaries, target, ridge, split and nulls are REPLACE-1 Amendment 1's (`examples/
//! replace1`'s `fluid_dict`, `fluid_q`, the chains' G3b question), unchanged. Per (seed, lag) the
//! gate is run once with the budget at `+∞` (the full removal path, every block priced) and once
//! at each swept `β` (the cells, as staked); the cells are checked against the path. Nothing is
//! written; the record is stdout, banked as `conformance/replace1/objective1_read.txt`.

#[path = "../replace1/chains.rs"]
mod chains;

use holon_closure::removable::{Admission, Candidate, Folds, Horizon, Mat, Nulls, NumpyPcg64, Question, Ridge, Swap, Unit};
use holon_lens::walk::{self, OrderFeatures, RigidWalk, C_LS};
use std::path::PathBuf;
use std::time::Instant;

const DEFAULT_WALKS: &str = "/home/emoore/CIRISHolon/conformance/water_observatory/replace0";
const DEFAULT_CHAINS: &str = "/home/emoore/RATCHET/release/data_scrubbed_v1/accord_traces.jsonl";
const BETAS: [f64; 4] = [0.005, 0.01, 0.02, 0.05];
const LAGS_PS: [f64; 4] = [0.5, 1.0, 2.0, 5.0];
const CHAIN_BETAS: [f64; 3] = [0.005, 0.01, 0.02];
const HYDRO: &str = "hydrodynamic (rho, jL)";
const CONSCIENCE: &str = "conscience (DMA) block";
const PROCESS: &str = "process sector";
const PLANTED: &str = "planted";

fn fluid_q(lag: usize) -> Question {
    Question { lag, horizon: Horizon::Future, folds: Folds::TimeBlocks { k: 5 }, ridge: Ridge::ORDER1 }
}

fn rd(ps: f64, dt_fs: f64) -> usize {
    ((ps * 1000.0 / dt_fs).round() as usize).max(1)
}

fn empty_base(chains: &[Mat], target: &[usize]) -> Vec<Unit> {
    chains.iter().map(|c| Unit { kept: Mat::zeros(c.rows, 0), target: c.select_cols(target) }).collect()
}

/// REPLACE-1 Amendment 1's dictionary on `[H(8) | q s2 (4) | n33 n50 nb (6)]` chains, with the
/// column offset of the structural fields (`off = 8` unplanted, `10` with a planted field at 8..10).
fn fluid_dict(dict_ch: &[Mat], off: usize) -> Vec<Candidate> {
    let qs2: Vec<usize> = (off..off + 4).collect();
    let counts: Vec<usize> = (off + 4..off + 10).collect();
    vec![
        Candidate::new(HYDRO, walk::cand(dict_ch, &C_LS)),
        Candidate::new("count block (n33, n50, nb)", walk::cand(dict_ch, &counts)),
        Candidate::new("structural (q, s2)", walk::cand(dict_ch, &qs2)),
    ]
}

/// The target's held-out R² from the FULL dictionary (every block kept) — the definedness read.
fn full_r2(base: &[Unit], dict: &[Candidate], q: &Question) -> f64 {
    let mut idx: Vec<usize> = (0..dict.len()).collect();
    idx.sort_by(|&a, &b| dict[a].name.cmp(&dict[b].name));
    let us: Vec<Unit> = base
        .iter()
        .enumerate()
        .map(|(u, un)| {
            let mut kept = un.kept.clone();
            for &i in &idx {
                kept = kept.hcat(&dict[i].cols[u]);
            }
            Unit { kept, target: un.target.clone() }
        })
        .collect();
    holon_closure::removable::closure_r2(&us, q)
}

fn short(name: &str) -> &str {
    match name {
        HYDRO => "H",
        "count block (n33, n50, nb)" => "C",
        "structural (q, s2)" => "S",
        PLANTED => "P",
        PROCESS => "proc",
        CONSCIENCE => "DMA",
        other => other,
    }
}

fn set_str(v: &[String]) -> String {
    if v.is_empty() {
        "{}".into()
    } else {
        let mut s: Vec<&str> = v.iter().map(|x| short(x)).collect();
        s.sort();
        format!("{{{}}}", s.join(","))
    }
}

fn fo(v: Option<f64>) -> String {
    v.map_or("-".into(), |x| format!("{x:+.4}"))
}

/// The full removal path (`β = +∞`) and each block's `β*`: the largest budget at which it
/// survives = the maximum step price up to and including the step that removes it.
struct Path {
    adm: Admission,
    beta_star: Vec<(String, f64)>,
}

fn path_of(base: &[Unit], dict: &[Candidate], q: &Question, nulls: &Nulls) -> Path {
    let adm = Admission::ordered(base, dict, q, nulls, f64::INFINITY);
    let mut run = f64::NEG_INFINITY;
    let beta_star = adm
        .order
        .iter()
        .map(|s| {
            run = run.max(s.price.increment);
            (s.price.name.clone(), run)
        })
        .collect();
    Path { adm, beta_star }
}

/// The survivors the path predicts at `β`.
fn predicted(path: &Path, beta: f64) -> Vec<String> {
    path.beta_star.iter().filter(|(_, b)| *b > beta).map(|(n, _)| n.clone()).collect()
}

fn path_lines(p: &Path) -> Vec<String> {
    p.adm
        .order
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let alts: Vec<String> = s
                .alternatives
                .iter()
                .map(|a| format!("{} {:+.4}±{:.4}", short(&a.name), a.increment, a.se))
                .collect();
            format!(
                "step {}: remove {} at {:+.4} ± {:.4} (nulls {} / {}); standing prices [{}]",
                i + 1,
                short(&s.price.name),
                s.price.increment,
                s.price.se,
                fo(s.price.null_shift),
                fo(s.price.null_swap),
                alts.join(", ")
            )
        })
        .collect()
}

struct Cell {
    beta: f64,
    survivors: Vec<String>,
    order: Vec<String>,
    agrees_with_path: bool,
}

struct SeedLag {
    seed: String,
    lag_ps: f64,
    r2_full: f64,
    path: Path,
    cells: Vec<Cell>,
}

impl SeedLag {
    fn defined(&self) -> bool {
        self.r2_full > 0.0
    }
}

fn sweep(seed: &str, base: &[Unit], dict: &[Candidate], q: &Question, nulls: &Nulls, lag_ps: f64, betas: &[f64]) -> SeedLag {
    let r2_full = full_r2(base, dict, q);
    let path = path_of(base, dict, q, nulls);
    let cells = betas
        .iter()
        .map(|&beta| {
            let o = Admission::ordered(base, dict, q, nulls, beta);
            let order: Vec<String> = o.order.iter().map(|s| s.price.name.clone()).collect();
            let mut surv = o.carried.clone();
            // a survivor that is Unresolved (a null over β) is listed with the dropped by the
            // gate but is NOT removed; the carried SET for invariance is what was not removed
            surv.extend(o.prices.iter().skip(o.order.len()).filter(|p| !o.carried.contains(&p.name)).map(|p| p.name.clone()));
            let mut a = surv.clone();
            a.sort();
            let mut b = predicted(&path, beta);
            b.sort();
            let prefix = order.iter().zip(&path.adm.order).all(|(x, s)| *x == s.price.name);
            Cell { beta, survivors: surv, order, agrees_with_path: a == b && prefix }
        })
        .collect();
    SeedLag { seed: seed.into(), lag_ps, r2_full, path, cells }
}

fn print_seedlag(s: &SeedLag) {
    println!(
        "   {} lag {} ps: full-dictionary R² {:+.4} -> {}",
        s.seed,
        s.lag_ps,
        s.r2_full,
        if s.defined() { "DEFINED" } else { "UNDEFINED (not graded)" }
    );
    for l in path_lines(&s.path) {
        println!("      {l}");
    }
    println!(
        "      beta*: {}",
        s.path.beta_star.iter().map(|(n, b)| format!("{} {:+.4}", short(n), b)).collect::<Vec<_>>().join(", ")
    );
    for c in &s.cells {
        println!(
            "      beta {:<5}: survivors {:<8} order [{}] | path check {}",
            c.beta,
            set_str(&c.survivors),
            c.order.iter().map(|x| short(x)).collect::<Vec<_>>().join(" > "),
            if c.agrees_with_path { "ok" } else { "MISMATCH" }
        );
    }
}

fn load_feats(walks: &std::path::Path) -> Vec<(String, OrderFeatures)> {
    let names = ["T293_seed0", "T293_seed1", "T293_seed2", "T400_seed0"];
    std::thread::scope(|sc| {
        let hs: Vec<_> = names
            .iter()
            .map(|s| {
                let dir = walks.join("slow1").join(s);
                sc.spawn(move || {
                    let t = Instant::now();
                    let w = RigidWalk::load(&dir, None).expect("SLOW-1 walk");
                    let f = walk::order_features(&w);
                    println!("# {s}: {} waters x {} readouts at {} fs, <q> {:.4}; dictionary built in {:.1} s", w.n, w.frames, f.dt_fs, f.q_mean, t.elapsed().as_secs_f64());
                    (s.to_string(), f)
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("walk thread")).collect()
    })
}

fn main() {
    let mut walks = PathBuf::from(DEFAULT_WALKS);
    let mut chains_path = DEFAULT_CHAINS.to_string();
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
            a => panic!("unknown argument {a}"),
        }
        i += 1;
    }
    let t0 = Instant::now();
    println!("# OBJECTIVE-1 — the carried set of ordered admission across budget and cadence");
    println!("# instrument: holon-closure::removable::Admission::ordered (REPLACE-1 Amendment 1), dictionaries as examples/replace1 builds them; walks {}", walks.display());
    let feats = load_feats(&walks);
    let nulls = Nulls { shift: true, swap: Some(Swap::Derange { shift: 2 }) };

    // ------------------------------------------------------------------ the fluid grid
    println!("\n## The fluid: dictionary {{H = (rho, jL), C = (n33, n50, nb), S = (q, s2)}}, target (rho_k, jL_k) at the lag, nulls time-shifted / chain two over");
    let mut grid: Vec<SeedLag> = Vec::new();
    for (name, f) in &feats {
        let ch = walk::order_chains(&f.h, &f.fields);
        let base = empty_base(&ch, &C_LS);
        let dict = fluid_dict(&ch, 8);
        for &lp in &LAGS_PS {
            let s = sweep(name, &base, &dict, &fluid_q(rd(lp, f.dt_fs)), &nulls, lp, &BETAS);
            print_seedlag(&s);
            grid.push(s);
        }
    }

    println!("\n## The grid (survivor set per cell; U = undefined, full-dictionary R² <= 0)");
    println!("| seed | lag | R² full | beta 0.005 | 0.01 | 0.02 | 0.05 |");
    println!("|---|---|---|---|---|---|---|");
    for s in &grid {
        let cells: Vec<String> =
            s.cells.iter().map(|c| if s.defined() { set_str(&c.survivors) } else { format!("U {}", set_str(&c.survivors)) }).collect();
        println!("| {} | {} ps | {:+.4} | {} |", s.seed, s.lag_ps, s.r2_full, cells.join(" | "));
    }
    let path_ok = grid.iter().all(|s| s.cells.iter().all(|c| c.agrees_with_path));
    println!("every cell agrees with its (seed, lag) path: {path_ok}");

    // Q1
    let hydro_only = vec![HYDRO.to_string()];
    let mut q1_kill = false;
    let mut q1_all_hydro = true;
    let mut q1_lines = Vec::new();
    for seed in ["T293_seed0", "T293_seed1", "T293_seed2"] {
        let sets: Vec<(f64, f64, String)> = grid
            .iter()
            .filter(|s| s.seed == seed && s.defined())
            .flat_map(|s| s.cells.iter().map(move |c| (s.lag_ps, c.beta, set_str(&c.survivors))))
            .collect();
        let distinct: std::collections::BTreeSet<&String> = sets.iter().map(|x| &x.2).collect();
        q1_kill |= distinct.len() > 1;
        q1_all_hydro &= distinct.iter().all(|x| **x == set_str(&hydro_only));
        let off: Vec<String> =
            sets.iter().filter(|x| x.2 != set_str(&hydro_only)).map(|x| format!("({} ps, {}) {}", x.0, x.1, x.2)).collect();
        q1_lines.push(format!(
            "{seed}: {} defined cells, survivor sets {:?}{}",
            sets.len(),
            distinct,
            if off.is_empty() { String::new() } else { format!("; cells not {{H}}: {}", off.join(", ")) }
        ));
    }
    let q1 = if q1_kill {
        "KILL"
    } else if q1_all_hydro {
        "MET"
    } else {
        "BETWEEN (one set per seed, not H alone)"
    };

    // Q2
    let q2_lines: Vec<String> = grid
        .iter()
        .filter(|s| s.seed == "T400_seed0")
        .map(|s| {
            format!(
                "{} ps R² {:+.4} {}: {}",
                s.lag_ps,
                s.r2_full,
                if s.defined() { "defined" } else { "UNDEFINED" },
                s.cells.iter().map(|c| format!("{} {}", c.beta, set_str(&c.survivors))).collect::<Vec<_>>().join(", ")
            )
        })
        .collect();

    // Q4
    let mut q4_kill = false;
    let mut q4_lines = Vec::new();
    for &lp in &LAGS_PS {
        let at: Vec<&SeedLag> = grid.iter().filter(|s| s.lag_ps == lp && s.seed.starts_with("T293")).collect();
        if !at.iter().all(|s| s.defined()) {
            q4_lines.push(format!("{lp} ps: not defined on all three seeds (R² {}) - not graded", at.iter().map(|s| format!("{:+.4}", s.r2_full)).collect::<Vec<_>>().join(" / ")));
            continue;
        }
        for (bi, &beta) in BETAS.iter().enumerate() {
            let sets: Vec<String> = at.iter().map(|s| set_str(&s.cells[bi].survivors)).collect();
            let set_same = sets.iter().all(|x| *x == sets[0]);
            let mut notes = Vec::new();
            let mut order_kill = false;
            for a in 0..3 {
                for b in a + 1..3 {
                    let (oa, ob) = (&at[a].cells[bi].order, &at[b].cells[bi].order);
                    if let Some(k) = (0..oa.len().min(ob.len())).find(|&k| oa[k] != ob[k]) {
                        let (x, y) = (&oa[k], &ob[k]);
                        let tie_on = |s: &SeedLag| {
                            let st = &s.path.adm.order[k];
                            let px = st.alternatives.iter().find(|p| &p.name == x);
                            let py = st.alternatives.iter().find(|p| &p.name == y);
                            match (px, py) {
                                (Some(px), Some(py)) => {
                                    let d = (px.increment - py.increment).abs();
                                    let se = (px.se.powi(2) + py.se.powi(2)).sqrt();
                                    (d <= se, format!("{}: {} {:+.4} vs {} {:+.4}, |d| {:.4} vs SE {:.4}", short(&s.seed), short(x), px.increment, short(y), py.increment, d, se))
                                }
                                _ => (false, format!("{}: a price missing", s.seed)),
                            }
                        };
                        let (ta, na) = tie_on(at[a]);
                        let (tb, nb) = tie_on(at[b]);
                        let tie = ta && tb;
                        order_kill |= !tie;
                        notes.push(format!(
                            "{} vs {} diverge at step {} ({} vs {}) -> {} [{na}; {nb}]",
                            at[a].seed,
                            at[b].seed,
                            k + 1,
                            short(x),
                            short(y),
                            if tie { "tie" } else { "NOT a tie" }
                        ));
                    } else if oa.len() != ob.len() {
                        notes.push(format!("{} vs {}: one order is a prefix of the other ({} vs {} steps)", at[a].seed, at[b].seed, oa.len(), ob.len()));
                    }
                }
            }
            q4_kill |= !set_same || order_kill;
            q4_lines.push(format!(
                "{lp} ps, beta {beta}: survivors {} {}{}",
                sets.join(" / "),
                if set_same { "agree" } else { "DIFFER" },
                if notes.is_empty() { "; orders agree".to_string() } else { format!("; {}", notes.join("; ")) }
            ));
        }
    }
    let q4 = if q4_kill { "KILL" } else { "MET" };

    // ------------------------------------------------------------------ the chains
    println!("\n## The chains: dictionary {{process sector, conscience (DMA) block}}, target = next thought's process sector, lag 1 thought, permutation null");
    let (mut q3, mut q3_line) = ("NOT READ".to_string(), "chains unreadable".to_string());
    match chains::load(&chains_path) {
        Ok(chs) => {
            let mats: Vec<Mat> = chs.iter().map(|c| Mat::from_rows(c)).collect();
            let base: Vec<Unit> = mats.iter().map(|m| Unit { kept: Mat::zeros(m.rows, 0), target: m.select_cols(&[4, 5, 6]) }).collect();
            let dict = vec![
                Candidate::new(PROCESS, mats.iter().map(|m| m.select_cols(&[4, 5, 6])).collect()),
                Candidate::new(CONSCIENCE, mats.iter().map(|m| m.select_cols(&[0, 1, 2, 3])).collect()),
            ];
            let q = Question { lag: 1, horizon: Horizon::Future, folds: Folds::Units { k: 4 }, ridge: Ridge::ORDER1 };
            let nn = Nulls { shift: false, swap: Some(Swap::Permute { seed: 0 }) };
            println!("   {} chains", mats.len());
            let s = sweep("chains", &base, &dict, &q, &nn, 0.0, &CHAIN_BETAS);
            print_seedlag(&s);
            // the survivors' own removal prices at each cell, from the cell's admission
            for &beta in &CHAIN_BETAS {
                let o = Admission::ordered(&base, &dict, &q, &nn, beta);
                let surv: Vec<String> = o
                    .prices
                    .iter()
                    .skip(o.order.len())
                    .map(|p| format!("{} {:+.4} ± {:.4} (null {}) {:?}", short(&p.name), p.increment, p.se, fo(p.null_swap), p.verdict))
                    .collect();
                println!("      beta {beta}: survivors' removal prices {}", surv.join("; "));
            }
            let has = |bi: usize, n: &str| s.cells[bi].survivors.iter().any(|x| x == n);
            let proc_all = (0..3).all(|b| has(b, PROCESS));
            let framing_kill = has(2, CONSCIENCE) || !has(0, CONSCIENCE);
            let as_staked = proc_all && has(0, CONSCIENCE) && has(1, CONSCIENCE) && !has(2, CONSCIENCE);
            q3 = if !s.defined() {
                "UNDEFINED".into()
            } else if framing_kill {
                "KILL (framing)".into()
            } else if as_staked {
                "AS STAKED (level-relative)".into()
            } else {
                "BETWEEN".into()
            };
            q3_line = format!(
                "R² full {:+.4}; {}",
                s.r2_full,
                s.cells.iter().map(|c| format!("beta {} {}", c.beta, set_str(&c.survivors))).collect::<Vec<_>>().join(", ")
            );
        }
        Err(e) => println!("   chains unreadable: {e}"),
    }

    // ------------------------------------------------------------------ the plants
    let (name0, f0) = &feats[0];
    let dt = f0.dt_fs;
    println!("\n## Plants on {name0} (PO-4 field: 5 ps memory, 1 ps latency, noise 0.05, plant seed 5), dictionary {{H, C, S, P}} on the planted carrier");
    let plant_betas = [0.005, 0.01, 0.02, 0.05, 0.2];
    let mut plant_ok = Vec::new();
    for (tag, amp, must) in [("PI-1", 1.0, [true, true, true, true, false]), ("PI-3", 0.25, [true, true, false, false, false])] {
        let (hp, qp) = walk::plant_po4(&f0.h, &f0.fields[0], 5000.0 / dt, rd(1.0, dt), 0.05, amp, 5);
        let mut fields = vec![qp];
        fields.extend(f0.fields.iter().cloned());
        let ch = walk::order_chains(&hp, &fields); // [H(8) | P(2) | q s2 n33 n50 nb (10)]
        let base = empty_base(&ch, &C_LS);
        let mut dict = fluid_dict(&ch, 10);
        dict.push(Candidate::new(PLANTED, walk::cand(&ch, &[8, 9])));
        let mut ok = true;
        for &lp in &LAGS_PS {
            let s = sweep(&format!("{tag} (amplitude {amp})"), &base, &dict, &fluid_q(rd(lp, dt)), &nulls, lp, &plant_betas);
            print_seedlag(&s);
            if lp == 1.0 {
                let got: Vec<bool> = s.cells.iter().map(|c| c.survivors.iter().any(|x| x == PLANTED)).collect();
                ok = got == must;
                let pstar = s.path.beta_star.iter().find(|(n, _)| n == PLANTED).map_or(f64::NAN, |x| x.1);
                println!(
                    "   {tag} at 1 ps: planted beta* {pstar:+.4}; survives at beta {:?} = {:?}, must {:?} -> {}",
                    plant_betas,
                    got,
                    must,
                    if ok { "PASS" } else { "FAIL" }
                );
            }
        }
        plant_ok.push((tag, ok));
    }
    // PI-2: the time-shuffled dictionary, target not shuffled
    let ch0 = walk::order_chains(&f0.h, &f0.fields);
    let perm = NumpyPcg64::new(11).permutation(ch0[0].rows);
    let sh: Vec<Mat> = ch0.iter().map(|c| c.permute_rows(&perm)).collect();
    let base0 = empty_base(&ch0, &C_LS);
    let dict_sh = fluid_dict(&sh, 8);
    let mut pi2 = true;
    let mut pi2_max = f64::NEG_INFINITY;
    for &lp in &LAGS_PS {
        let s = sweep("PI-2 (shuffled dictionary)", &base0, &dict_sh, &fluid_q(rd(lp, dt)), &nulls, lp, &BETAS);
        print_seedlag(&s);
        pi2 &= s.cells.iter().all(|c| c.survivors.is_empty());
        pi2_max = pi2_max.max(s.path.adm.order.iter().map(|x| x.price.increment).fold(f64::NEG_INFINITY, f64::max));
    }
    println!("   PI-2: nothing kept at any beta and lag -> {} (largest step price {pi2_max:+.4})", if pi2 { "PASS" } else { "FAIL" });
    plant_ok.insert(1, ("PI-2", pi2));

    // ------------------------------------------------------------------ verdicts
    println!("\n# VERDICT TABLE");
    println!("Q1 (invariance, fluid 293 K) | {} -> {q1}", q1_lines.join(" | "));
    println!("Q2 (400 K, reported) | {}", q2_lines.join(" | "));
    println!("Q3 (chains) | {q3_line} -> {q3}");
    println!("Q4 (seeds) | {} -> {q4}", q4_lines.join(" | "));
    println!("plants: {}", plant_ok.iter().map(|(n, ok)| format!("{n} {}", if *ok { "PASS" } else { "FAIL" })).collect::<Vec<_>>().join(", "));
    println!("cells agree with paths: {path_ok}");
    let plants_all = plant_ok.iter().all(|x| x.1);
    let branch = if !plants_all {
        "(e) a plant fails - nothing is read".to_string()
    } else if q3.starts_with("KILL") {
        "(c) Q3's framing killed - the chains' prices are not where REPLACE-1 read them".to_string()
    } else if q1 == "KILL" {
        format!("(b) Q1 killed - the fluid's carried set is budget-relative too (Q3 {q3}, Q4 {q4})")
    } else if q1 == "MET" && q4 == "MET" && q3.starts_with("AS STAKED") {
        "(a) Q1 and Q4 met, Q3 as staked".to_string()
    } else {
        format!("none cleanly: Q1 {q1}, Q3 {q3}, Q4 {q4}")
    };
    println!("BRANCH: {branch}");
    println!("# wall {:.1} s", t0.elapsed().as_secs_f64());
}
