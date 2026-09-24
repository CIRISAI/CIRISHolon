//! QVM-RANK-1 driver (`conformance/rank/QVM_RANK1_PREREG.md`).
//!
//! ```text
//! qvm_rank1 anneal  M RANK THREADS MINUTES OUTDIR [--seeds FILE] [--filter] [--galois]
//! qvm_rank1 exhaust4 OUTDIR                    # m = 4, rank 4, every canonical pivot pair
//! ```
//!
//! Run under `taskset -c 21-27` (prereg §5). Every witness is written in stabrank's format
//! to OUTDIR and must then pass stabrank's own verifier before anything is claimed.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use holon::stabrank::*;

fn now_utc() -> String {
    let out = std::process::Command::new("date").arg("-u").arg("+%Y-%m-%dT%H:%M:%SZ").output();
    out.map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
}

fn write_witness(dir: &str, tag: &str, w: &Witness, method: &str, wall_s: f64, cpu_s: f64) -> String {
    std::fs::create_dir_all(dir).unwrap();
    let path = format!("{dir}/{tag}.json");
    let compute = format!(
        "{{\"cpu_hours\": {:.4}, \"wall_clock_hours\": {:.4}, \"runs\": 1, \"hardware\": \"x86_64 Linux, cores 21-27 (taskset)\"}}",
        cpu_s / 3600.0,
        wall_s / 3600.0
    );
    let json = w.to_stabrank_json("QVM-RANK-1 (CIRISHolon)", method, &now_utc(), &compute);
    std::fs::write(&path, json).unwrap();
    path
}

fn parse_seed_file(path: &str) -> Vec<Vec<Vec<u8>>> {
    let text = std::fs::read_to_string(path).expect("seed file");
    parse_known(&text).into_iter().map(|d| d.3.iter().map(|s| s.phases()).collect()).collect()
}

fn cmd_anneal(args: &[String]) {
    let m: usize = args[0].parse().unwrap();
    let rank: usize = args[1].parse().unwrap();
    let threads: usize = args[2].parse().unwrap();
    let minutes: f64 = args[3].parse().unwrap();
    let outdir = args[4].clone();
    let filter = args.iter().any(|a| a == "--filter");
    let galois = args.iter().any(|a| a == "--galois");
    let moves: u64 = args
        .iter()
        .position(|a| a == "--moves")
        .map(|i| args[i + 1].parse().unwrap())
        .unwrap_or(200_000);
    let seeds: Vec<Vec<Vec<u8>>> = args
        .iter()
        .position(|a| a == "--seeds")
        .map(|i| parse_seed_file(&args[i + 1]))
        .unwrap_or_default();
    let found = Arc::new(AtomicBool::new(false));
    let total_moves = Arc::new(AtomicU64::new(0));
    let result: Arc<Mutex<Option<Witness>>> = Arc::new(Mutex::new(None));
    let t0 = Instant::now();
    eprintln!("anneal m={m} rank={rank} threads={threads} minutes={minutes} filter={filter} galois={galois} moves/restart={moves}");
    let mut hs = Vec::new();
    for t in 0..threads {
        let found = found.clone();
        let total_moves = total_moves.clone();
        let result = result.clone();
        let seeds: Vec<Vec<Vec<u8>>> = seeds.iter().skip(t).step_by(threads).cloned().collect();
        hs.push(std::thread::spawn(move || {
            let cfg = AnnealCfg {
                n: m,
                rank,
                seed: 1000 + t as u64,
                moves,
                t0: 0.02,
                t1: 0.0005,
                galois_objective: galois,
                term_filter: filter,
                exact_threshold: 1e-9,
                snap_threshold: 0.05,
            };
            let deadline = minutes * 60.0;
            let stop = |s: &AnnealStats| -> bool {
                total_moves.fetch_max(s.moves, Ordering::Relaxed);
                found.load(Ordering::Relaxed) || t0.elapsed().as_secs_f64() > deadline
            };
            let (w, st) = anneal(&cfg, &seeds, &stop);
            if let Some(w) = w {
                found.store(true, Ordering::Relaxed);
                let mut r = result.lock().unwrap();
                if r.is_none() {
                    *r = Some(w);
                }
            }
            st
        }));
    }
    let stats: Vec<AnnealStats> = hs.into_iter().map(|h| h.join().unwrap()).collect();
    let wall = t0.elapsed().as_secs_f64();
    let moves_total: u64 = stats.iter().map(|s| s.moves).sum();
    let best = stats.iter().map(|s| s.best).fold(f64::INFINITY, f64::min);
    let snaps: u64 = stats.iter().map(|s| s.snaps).sum();
    let restarts: u64 = stats.iter().map(|s| s.restarts).sum();
    let exact: u64 = stats.iter().map(|s| s.exact_tests).sum();
    println!(
        "anneal m={m} rank={rank}: wall {wall:.1} s, moves {moves_total}, restarts {restarts}, exact tests {exact}, snaps {snaps}, best residual {best:.3e}"
    );
    let r = result.lock().unwrap();
    if let Some(w) = r.as_ref() {
        assert!(w.accept_gi() && w.accept_cyc(), "a returned witness must pass both exact checks");
        let path = write_witness(
            &outdir,
            &format!("qubit_H-m{m}-rank{rank}-anneal"),
            w,
            "annealing (Pauli/Clifford/single-state moves) with exact Z[i] Bareiss acceptance and pivot-completion snap",
            wall,
            wall * threads as f64,
        );
        std::fs::write(
            format!("{outdir}/qubit_H-m{m}-rank{rank}-anneal.dec.txt"),
            dec_text(&w.terms, &format!("qvm_rank1:anneal-m{m}-rank{rank}")),
        )
        .unwrap();
        println!("WITNESS m={m} rank={rank} found in {wall:.1} s -> {path}");
        for (j, s) in w.terms.iter().enumerate() {
            println!("  term {j}: k={} dual_distance={}", s.k, s.dual_distance());
        }
    } else {
        println!("NO WITNESS m={m} rank={rank} in {wall:.1} s");
    }
}

fn cmd_exhaust4(args: &[String]) {
    let outdir = args[0].clone();
    let t0 = Instant::now();
    let d = enumerate_all(4);
    let g = symmetry_group(4);
    let act = action_table(&d, &g);
    let pairs = canonical_sets(&act, d.len(), 2);
    let (a, b) = target_ab(4);
    let mut full_rank = 0u64;
    let mut wits: Vec<Witness> = Vec::new();
    let mut classes: std::collections::BTreeSet<Vec<u32>> = std::collections::BTreeSet::new();
    let index: std::collections::HashMap<Stab, u32> = d.iter().enumerate().map(|(i, s)| (s.clone(), i as u32)).collect();
    for p in &pairs {
        let piv = vec![d[p[0] as usize].clone(), d[p[1] as usize].clone()];
        let mut cols: Vec<Vec<Gi>> = piv.iter().map(|s| s.gi_vec()).collect();
        cols.push(a.iter().map(|&x| Gi::new(x, 0)).collect());
        cols.push(b.iter().map(|&x| Gi::new(x, 0)).collect());
        if bareiss_rank(&cols).0 < 4 {
            continue;
        }
        full_rank += 1;
        for w in witnesses_from_pivot(&piv) {
            let mut ids: Vec<u32> = w.terms.iter().map(|s| index[s]).collect();
            ids.sort_unstable();
            // canonical 4-set under the full group
            let mut best = ids.clone();
            let mut img = vec![0u32; 4];
            for perm in &act {
                for (x, &i) in img.iter_mut().zip(&ids) {
                    *x = perm[i as usize];
                }
                img.sort_unstable();
                if img < best {
                    best.clone_from(&img);
                }
            }
            if classes.insert(best) {
                wits.push(w);
            }
        }
    }
    let wall = t0.elapsed().as_secs_f64();
    println!(
        "exhaust4: {} canonical pivot pairs, {} with rank(P,a,b) = 4, {} rank-4 decomposition classes of |H>^4 up to S4 x H^(subset), wall {:.1} s",
        pairs.len(),
        full_rank,
        classes.len(),
        wall
    );
    if let Some(w) = wits.first() {
        let path = write_witness(&outdir, "qubit_H-m4-rank4-exhaust", w, "pivot completion over every canonical pivot pair (exhaustive)", wall, wall);
        println!("WITNESS m=4 rank=4 -> {path}");
    }
    let mut txt = String::from("# every rank-4 decomposition class of |H>^4 found by qvm_rank1 exhaust4\n");
    for w in &wits {
        txt.push_str("dec 4 4 qvm_rank1:exhaust4\n");
        for s in &w.terms {
            let bits = |m: u32| -> String { (0..4).map(|c| char::from(b'0' + ((m >> c) & 1) as u8)).collect() };
            let wrows: Vec<String> = s.w.iter().map(|&r| bits(r)).collect();
            let k = s.k as usize;
            let qrows: Vec<String> = (0..k)
                .map(|i| (0..k).map(|j| if j > i && (s.q[i] >> j) & 1 == 1 { '1' } else { '0' }).collect())
                .collect();
            let ls: Vec<String> = s.l.iter().map(|x| x.to_string()).collect();
            txt.push_str(&format!(
                "k={} x0={} W={} Q={} l={}\n",
                k,
                bits(s.x0),
                if k == 0 { "-".into() } else { wrows.join(";") },
                if k == 0 { "-".into() } else { qrows.join(";") },
                if k == 0 { "-".into() } else { ls.join(",") }
            ));
        }
    }
    std::fs::write(format!("{outdir}/exhaust4_classes.txt"), txt).unwrap();
}

/// Lift one decomposition file's decompositions (flat format) one qubit up, report every lift.
fn cmd_lift(args: &[String]) {
    let text = std::fs::read_to_string(&args[0]).expect("decomposition file");
    let outdir = args[1].clone();
    for (m, r, src, terms) in parse_known(&text) {
        let base = exact_solve(&terms).unwrap_or_else(|| panic!("{src}: not exact"));
        let t0 = Instant::now();
        let (lifts, st) = lift(&base, 20260924);
        println!("lift {src} (m={m}, r={r}) -> m={}: {} lifts in {:.1} s, {:?}", m + 1, lifts.len(), t0.elapsed().as_secs_f64(), st);
        for (i, w) in lifts.iter().enumerate() {
            let path = write_witness(&outdir, &format!("qubit_H-m{}-rank{r}-lift-{i}", m + 1), w, "slice lift", 0.0, 0.0);
            std::fs::write(format!("{outdir}/qubit_H-m{}-rank{r}-lift-{i}.dec.txt", m + 1), dec_text(&w.terms, &format!("lift:{src}"))).unwrap();
            println!("WITNESS m={} rank={r} -> {path}", m + 1);
        }
    }
}

/// The shot (prereg STEP 3): both branches at once, checkpointed, logged every 30 minutes.
///
/// Roles (threads): `h5` harvest rank-6 decompositions of |H>^5 by annealing, dedupe under
/// S5 x H^(subset), lift each new class to |H>^6 (complete, exact); `h6` harvest rank-6
/// decompositions of |H>^6 directly; `l7` lift every new |H>^6 class to |H>^7 (complete,
/// exact) — THE exhaustive branch over the harvested bases; `a7` anneal |H>^7 at rank 6
/// directly with the per-term filter (property P) and exact acceptance.
fn cmd_tower(args: &[String]) {
    let outdir = args[0].clone();
    let hours: f64 = args[1].parse().unwrap();
    let get = |k: &str, d: usize| -> usize {
        args.iter().position(|a| a == k).map(|i| args[i + 1].parse().unwrap()).unwrap_or(d)
    };
    let (nh5, nh6, nl7, na7) = (get("--h5", 0), get("--h6", 0), get("--l7", 2), get("--a7", 1));
    let (nv6, nv7) = (get("--v6", 2), get("--v7", 2));
    std::fs::create_dir_all(&outdir).unwrap();
    let t0 = Instant::now();
    let deadline = hours * 3600.0;
    let found = Arc::new(AtomicBool::new(false));
    let g5 = Arc::new(symmetry_group(5));
    let g6 = Arc::new(symmetry_group(6));
    struct Shared {
        c5: std::collections::HashSet<Vec<Stab>>,
        c6: std::collections::HashSet<Vec<Stab>>,
        q6: std::collections::VecDeque<(Vec<Stab>, String)>,
        lifted7: u64,
        lift7_secs: f64,
        h5_found: u64,
        h6_found: u64,
        c6_from_lift: u64,
        c6_from_h6: u64,
        lifts56: u64,
        a7_moves: u64,
        a7_restarts: u64,
        a7_best: f64,
        a7_snaps: u64,
        h5_moves: u64,
        h6_moves: u64,
        v6_members: Vec<(Vec<Stab>, (Gi, Gi))>,
        v6_moves: u64,
        v6_joins: u64,
        v7_members: Vec<(Vec<Stab>, (Gi, Gi))>,
        v7_moves: u64,
        v7_joins: u64,
        v7_best: f64,
        v7_by_size: [u64; 6],
    }
    let sh = Arc::new(Mutex::new(Shared {
        c5: Default::default(),
        c6: Default::default(),
        q6: Default::default(),
        lifted7: 0,
        lift7_secs: 0.0,
        h5_found: 0,
        h6_found: 0,
        c6_from_lift: 0,
        c6_from_h6: 0,
        lifts56: 0,
        a7_moves: 0,
        a7_restarts: 0,
        a7_best: f64::INFINITY,
        a7_snaps: 0,
        h5_moves: 0,
        h6_moves: 0,
        v6_members: Vec::new(),
        v6_moves: 0,
        v6_joins: 0,
        v7_members: Vec::new(),
        v7_moves: 0,
        v7_joins: 0,
        v7_best: f64::INFINITY,
        v7_by_size: [0; 6],
    }));
    let append = |path: String, text: String| {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
        f.write_all(text.as_bytes()).unwrap();
    };
    // resume: previously checkpointed classes (lifted ones are not re-lifted)
    {
        let mut s = sh.lock().unwrap();
        if let Ok(t) = std::fs::read_to_string(format!("{outdir}/classes5.txt")) {
            for d in parse_known(&t) {
                s.c5.insert(d.3);
            }
        }
        let lifted: std::collections::HashSet<String> = std::fs::read_to_string(format!("{outdir}/lifted7.txt"))
            .map(|t| t.lines().filter_map(|l| l.split_whitespace().next().map(String::from)).collect())
            .unwrap_or_default();
        if let Ok(t) = std::fs::read_to_string(format!("{outdir}/classes6.txt")) {
            for d in parse_known(&t) {
                let key = d.2.clone();
                s.c6.insert(d.3.clone());
                if !lifted.contains(key.split(':').next().unwrap()) {
                    s.q6.push_back((d.3, key));
                }
            }
        }
        s.lifted7 = lifted.len() as u64;
        eprintln!("resume: {} m5 classes, {} m6 classes, {} already lifted to m7, {} queued", s.c5.len(), s.c6.len(), s.lifted7, s.q6.len());
    }
    let mut hs = Vec::new();
    let new_c6 = {
        let sh = sh.clone();
        let outdir = outdir.clone();
        move |key: Vec<Stab>, src: &str, from_lift: bool| {
            let mut s = sh.lock().unwrap();
            if s.c6.insert(key.clone()) {
                let id = format!("c6-{}", s.c6.len());
                if from_lift {
                    s.c6_from_lift += 1;
                } else {
                    s.c6_from_h6 += 1;
                }
                append(format!("{outdir}/classes6.txt"), dec_text(&key, &format!("{id}:{src}:t={:.0}s", t0.elapsed().as_secs_f64())));
                s.q6.push_back((key, format!("{id}:{src}:t={:.0}s", t0.elapsed().as_secs_f64())));
            }
        }
    };
    let new_c6 = Arc::new(new_c6);
    // h5 workers
    for t in 0..nh5 {
        let (sh, found, g5, g6, new_c6, outdir) = (sh.clone(), found.clone(), g5.clone(), g6.clone(), new_c6.clone(), outdir.clone());
        hs.push(std::thread::spawn(move || {
            let mut seed = 50_000 + 1000 * t as u64;
            while !found.load(Ordering::Relaxed) && t0.elapsed().as_secs_f64() < deadline {
                seed += 1;
                let cfg = AnnealCfg { n: 5, rank: 6, seed, moves: 100_000, t0: 0.02, t1: 0.0005, galois_objective: false, term_filter: false, exact_threshold: 1e-9, snap_threshold: 0.05 };
                let stop = |st: &AnnealStats| st.restarts > 1 || found.load(Ordering::Relaxed) || t0.elapsed().as_secs_f64() > deadline;
                let (w, st) = anneal(&cfg, &[], &stop);
                sh.lock().unwrap().h5_moves += st.moves;
                let Some(w) = w else { continue };
                let key = canonical_set_key(&w.terms, &g5);
                {
                    let mut s = sh.lock().unwrap();
                    s.h5_found += 1;
                    if !s.c5.insert(key.clone()) {
                        continue;
                    }
                    let id = s.c5.len();
                    append(format!("{outdir}/classes5.txt"), dec_text(&key, &format!("c5-{id}:seed={seed}:t={:.0}s", t0.elapsed().as_secs_f64())));
                }
                let base = exact_solve(&key).expect("class rep is exact");
                let (lifts, _) = lift(&base, seed);
                sh.lock().unwrap().lifts56 += lifts.len() as u64;
                for l in lifts {
                    let k6 = canonical_set_key(&l.terms, &g6);
                    new_c6(k6, &format!("lift-of-c5-seed{seed}"), true);
                }
            }
        }));
    }
    for t in 0..nh6 {
        let (sh, found, g6, new_c6) = (sh.clone(), found.clone(), g6.clone(), new_c6.clone());
        hs.push(std::thread::spawn(move || {
            let mut seed = 60_000 + 1000 * t as u64;
            while !found.load(Ordering::Relaxed) && t0.elapsed().as_secs_f64() < deadline {
                seed += 1;
                let cfg = AnnealCfg { n: 6, rank: 6, seed, moves: 300_000, t0: 0.02, t1: 0.0005, galois_objective: false, term_filter: true, exact_threshold: 1e-9, snap_threshold: 0.05 };
                let stop = |st: &AnnealStats| st.restarts > 1 || found.load(Ordering::Relaxed) || t0.elapsed().as_secs_f64() > deadline;
                let (w, st) = anneal(&cfg, &[], &stop);
                sh.lock().unwrap().h6_moves += st.moves;
                let Some(w) = w else { continue };
                sh.lock().unwrap().h6_found += 1;
                let k6 = canonical_set_key(&w.terms, &g6);
                new_c6(k6, &format!("anneal6-seed{seed}"), false);
            }
        }));
    }
    // v6: harvest rank-3 members of V at m = 6, join into rank-6 decompositions of |H>^6
    for t in 0..nv6 {
        let (sh, found, g6, new_c6, outdir) = (sh.clone(), found.clone(), g6.clone(), new_c6.clone(), outdir.clone());
        hs.push(std::thread::spawn(move || {
            let mut seed = 110_000 + 10_000 * t as u64;
            let term_ok = |_: &Stab| true;
            while !found.load(Ordering::Relaxed) && t0.elapsed().as_secs_f64() < deadline {
                seed += 1;
                let stop = |st: &AnnealStats| st.restarts > 1 || found.load(Ordering::Relaxed) || t0.elapsed().as_secs_f64() > deadline;
                let (members, st) = anneal_vmember(6, 3, seed, 200_000, &term_ok, &stop);
                sh.lock().unwrap().v6_moves += st.moves;
                for tup in members {
                    let dir = vmember_direction(&tup);
                    let others = {
                        let mut s = sh.lock().unwrap();
                        if s.v6_members.iter().any(|(q, _)| *q == tup) {
                            continue;
                        }
                        s.v6_members.push((tup.clone(), dir));
                        append(format!("{outdir}/members6.txt"), dec_text(&tup, &format!("v6:seed={seed}:dir=({},{})/({},{})", dir.0.re, dir.0.im, dir.1.re, dir.1.im)));
                        s.v6_members.clone()
                    };
                    for (q, qd) in others {
                        if q == tup || dir.0.mul(qd.1).sub(dir.1.mul(qd.0)).is_zero() {
                            continue;
                        }
                        sh.lock().unwrap().v6_joins += 1;
                        let mut terms = tup.clone();
                        terms.extend(q.iter().cloned());
                        if let Some(w) = exact_solve(&terms) {
                            let k6 = canonical_set_key(&w.terms, &g6);
                            new_c6(k6, &format!("v6-join-seed{seed}"), false);
                        }
                    }
                }
            }
        }));
    }
    // v7: harvest members of V at m = 7 (sizes 2, 3, 4, with property P on every term), join
    for t in 0..nv7 {
        let (sh, found, outdir) = (sh.clone(), found.clone(), outdir.clone());
        hs.push(std::thread::spawn(move || {
            let mut seed = 130_000 + 10_000 * t as u64;
            let term_ok = |s: &Stab| passes_term_filter(s, 7, 6);
            while !found.load(Ordering::Relaxed) && t0.elapsed().as_secs_f64() < deadline {
                seed += 1;
                let r1 = [3usize, 2, 4][(seed % 3) as usize];
                let stop = |st: &AnnealStats| st.restarts > 1 || found.load(Ordering::Relaxed) || t0.elapsed().as_secs_f64() > deadline;
                let (members, st) = anneal_vmember(7, r1, seed, 400_000, &term_ok, &stop);
                {
                    let mut s = sh.lock().unwrap();
                    s.v7_moves += st.moves;
                    if st.best < s.v7_best {
                        s.v7_best = st.best;
                    }
                }
                for tup in members {
                    let dir = vmember_direction(&tup);
                    let others = {
                        let mut s = sh.lock().unwrap();
                        if s.v7_members.iter().any(|(q, _)| *q == tup) {
                            continue;
                        }
                        s.v7_members.push((tup.clone(), dir));
                        s.v7_by_size[tup.len()] += 1;
                        append(format!("{outdir}/members7.txt"), dec_text(&tup, &format!("v7:seed={seed}:dir=({},{})/({},{})", dir.0.re, dir.0.im, dir.1.re, dir.1.im)));
                        s.v7_members.clone()
                    };
                    for (q, qd) in others {
                        if q.len() + tup.len() != 6 || q == tup || dir.0.mul(qd.1).sub(dir.1.mul(qd.0)).is_zero() {
                            continue;
                        }
                        sh.lock().unwrap().v7_joins += 1;
                        let mut terms = tup.clone();
                        terms.extend(q.iter().cloned());
                        if let Some(w) = exact_solve(&terms) {
                            found.store(true, Ordering::Relaxed);
                            let path = write_witness(&outdir, &format!("qubit_H-m7-rank6-WITNESS-v7-{seed}"), &w, "V-member split at m=7, exact acceptance", t0.elapsed().as_secs_f64(), 0.0);
                            std::fs::write(format!("{outdir}/qubit_H-m7-rank6-WITNESS-v7-{seed}.dec.txt"), dec_text(&w.terms, "v7")).unwrap();
                            println!("WITNESS m=7 rank=6 -> {path}");
                        }
                    }
                }
            }
        }));
    }
    for _ in 0..nl7 {
        let (sh, found, outdir) = (sh.clone(), found.clone(), outdir.clone());
        hs.push(std::thread::spawn(move || {
            while !found.load(Ordering::Relaxed) && t0.elapsed().as_secs_f64() < deadline {
                let job = sh.lock().unwrap().q6.pop_front();
                let Some((key, id)) = job else {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    continue;
                };
                let base = exact_solve(&key).expect("m6 class exact");
                let tl = Instant::now();
                let (lifts, st) = lift(&base, 7);
                let secs = tl.elapsed().as_secs_f64();
                let cls = id.split(':').next().unwrap().to_string();
                append(format!("{outdir}/lifted7.txt"), format!("{cls} lifts={} secs={secs:.1} window_hits={} float_matches={}\n", lifts.len(), st.window_hits, st.float_matches));
                {
                    let mut s = sh.lock().unwrap();
                    s.lifted7 += 1;
                    s.lift7_secs += secs;
                }
                for (i, w) in lifts.iter().enumerate() {
                    found.store(true, Ordering::Relaxed);
                    let path = write_witness(&outdir, &format!("qubit_H-m7-rank6-WITNESS-{cls}-{i}"), w, "slice lift of a rank-6 decomposition of |H>^6", t0.elapsed().as_secs_f64(), 0.0);
                    std::fs::write(format!("{outdir}/qubit_H-m7-rank6-WITNESS-{cls}-{i}.dec.txt"), dec_text(&w.terms, &format!("lift7:{id}"))).unwrap();
                    println!("WITNESS m=7 rank=6 -> {path}");
                }
            }
        }));
    }
    for t in 0..na7 {
        let (sh, found, outdir) = (sh.clone(), found.clone(), outdir.clone());
        hs.push(std::thread::spawn(move || {
            let mut seed = 70_000 + 1000 * t as u64;
            while !found.load(Ordering::Relaxed) && t0.elapsed().as_secs_f64() < deadline {
                seed += 1;
                let cfg = AnnealCfg { n: 7, rank: 6, seed, moves: 2_000_000, t0: 0.02, t1: 0.0005, galois_objective: t % 2 == 1, term_filter: true, exact_threshold: 1e-9, snap_threshold: 0.1 };
                let stop = |st: &AnnealStats| st.restarts > 1 || found.load(Ordering::Relaxed) || t0.elapsed().as_secs_f64() > deadline;
                let (w, st) = anneal(&cfg, &[], &stop);
                {
                    let mut s = sh.lock().unwrap();
                    s.a7_moves += st.moves;
                    s.a7_restarts += 1;
                    s.a7_snaps += st.snaps;
                    if st.best < s.a7_best {
                        s.a7_best = st.best;
                    }
                }
                if let Some(w) = w {
                    found.store(true, Ordering::Relaxed);
                    let path = write_witness(&outdir, &format!("qubit_H-m7-rank6-WITNESS-anneal-{seed}"), &w, "annealing at m=7 with exact acceptance", t0.elapsed().as_secs_f64(), 0.0);
                    std::fs::write(format!("{outdir}/qubit_H-m7-rank6-WITNESS-anneal-{seed}.dec.txt"), dec_text(&w.terms, "anneal7")).unwrap();
                    println!("WITNESS m=7 rank=6 -> {path}");
                }
            }
        }));
    }
    // logger
    let mut next_log = 0.0;
    loop {
        let el = t0.elapsed().as_secs_f64();
        let done = found.load(Ordering::Relaxed) || el > deadline;
        if el >= next_log || done {
            let s = sh.lock().unwrap();
            let line = format!(
                "{} t={:.2}h | m5: {} classes ({} anneal hits, {} moves) | m6: {} classes ({} from lifts of m5 [{} lifts], {} from direct anneal [{} hits, {} moves]) | m7 lift branch: {}/{} m6 classes lifted exactly, 0 lifts, {:.0} s | m7 anneal: {} restarts, {} moves, {} snaps, best residual {:.4e} | v6: {} members, {} joins, {} moves | v7: {} members (by size {:?}), {} joins, {} moves, best {:.4e} | witness: {}\n",
                now_utc(), el / 3600.0, s.c5.len(), s.h5_found, s.h5_moves, s.c6.len(), s.c6_from_lift, s.lifts56, s.c6_from_h6, s.h6_found, s.h6_moves,
                s.lifted7, s.c6.len(), s.lift7_secs, s.a7_restarts, s.a7_moves, s.a7_snaps, s.a7_best,
                s.v6_members.len(), s.v6_joins, s.v6_moves, s.v7_members.len(), &s.v7_by_size[2..5], s.v7_joins, s.v7_moves, s.v7_best,
                found.load(Ordering::Relaxed)
            );
            drop(s);
            append(format!("{outdir}/../rank1_progress.log"), line.clone());
            eprint!("{line}");
            next_log += 1800.0;
        }
        if done {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
    for h in hs {
        let _ = h.join();
    }
}

/// The split branch: harvest tuples whose span meets `V = span(|H>^m, |H^perp>^m)` (sizes in
/// `--sizes`, e.g. `3` or `2,4`), and join every pair of harvested tuples whose sizes sum to
/// RANK and whose members of V are independent — their union spans V, hence the target.
/// This is the shape of the QPG construction (two cat-like members of rank 3 at m = 6).
fn cmd_vsplit(args: &[String]) {
    let m: usize = args[0].parse().unwrap();
    let rank: usize = args[1].parse().unwrap();
    let threads: usize = args[2].parse().unwrap();
    let minutes: f64 = args[3].parse().unwrap();
    let outdir = args[4].clone();
    std::fs::create_dir_all(&outdir).unwrap();
    let filter = args.iter().any(|a| a == "--filter");
    let sizes: Vec<usize> = args
        .iter()
        .position(|a| a == "--sizes")
        .map(|i| args[i + 1].split(',').map(|x| x.parse().unwrap()).collect())
        .unwrap_or(vec![rank / 2]);
    let log = args.iter().position(|a| a == "--log").map(|i| args[i + 1].clone());
    let t0 = Instant::now();
    let found = Arc::new(AtomicBool::new(false));
    type Member = (Vec<Stab>, (Gi, Gi));
    let pool: Arc<Mutex<Vec<Member>>> = Arc::new(Mutex::new(Vec::new()));
    let stats = Arc::new(Mutex::new((0u64, 0u64, 0u64, f64::INFINITY, 0u64))); // moves, restarts, joins, best, members
    let result: Arc<Mutex<Option<Witness>>> = Arc::new(Mutex::new(None));
    let mut hs = Vec::new();
    for t in 0..threads {
        let (found, pool, stats, result, sizes, outdir) =
            (found.clone(), pool.clone(), stats.clone(), result.clone(), sizes.clone(), outdir.clone());
        hs.push(std::thread::spawn(move || {
            let mut seed = 90_000 + 10_000 * t as u64;
            let term_ok = |s: &Stab| !filter || passes_term_filter(s, m, rank);
            while !found.load(Ordering::Relaxed) && t0.elapsed().as_secs_f64() < minutes * 60.0 {
                seed += 1;
                let r1 = sizes[(seed as usize) % sizes.len()];
                let stop = |st: &AnnealStats| st.restarts > 1 || found.load(Ordering::Relaxed) || t0.elapsed().as_secs_f64() > minutes * 60.0;
                let (members, st) = anneal_vmember(m, r1, seed, 200_000, &term_ok, &stop);
                {
                    let mut s = stats.lock().unwrap();
                    s.0 += st.moves;
                    s.1 += st.restarts;
                    s.3 = s.3.min(st.best);
                }
                for tup in members {
                    let dir = vmember_direction(&tup);
                    let others: Vec<Member> = {
                        let mut p = pool.lock().unwrap();
                        if p.iter().any(|(q, _)| *q == tup) {
                            continue;
                        }
                        p.push((tup.clone(), dir));
                        stats.lock().unwrap().4 += 1;
                        std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(format!("{outdir}/members_m{m}.txt"))
                            .and_then(|mut f| {
                                use std::io::Write;
                                f.write_all(dec_text(&tup, &format!("vmember:seed={seed}:t={:.0}s:dir=({},{})/({},{})", t0.elapsed().as_secs_f64(), dir.0.re, dir.0.im, dir.1.re, dir.1.im)).as_bytes())
                            })
                            .unwrap();
                        p.clone()
                    };
                    for (q, qd) in others {
                        if q.len() + tup.len() != rank || q == tup {
                            continue;
                        }
                        // independent members: det [[α1, β1], [α2, β2]] != 0
                        if dir.0.mul(qd.1).sub(dir.1.mul(qd.0)).is_zero() {
                            continue;
                        }
                        stats.lock().unwrap().2 += 1;
                        let mut terms = tup.clone();
                        terms.extend(q.iter().cloned());
                        if let Some(w) = exact_solve(&terms) {
                            found.store(true, Ordering::Relaxed);
                            let mut r = result.lock().unwrap();
                            if r.is_none() {
                                *r = Some(w);
                            }
                        }
                    }
                }
            }
        }));
    }
    let mut next = 0.0;
    loop {
        let el = t0.elapsed().as_secs_f64();
        let done = found.load(Ordering::Relaxed) || el > minutes * 60.0;
        if el >= next || done {
            let s = *stats.lock().unwrap();
            let line = format!(
                "{} vsplit m={m} rank={rank} sizes={sizes:?} t={:.2}h | members of V harvested: {} | joins tested: {} | anneal restarts {} moves {} best {:.3e} | witness: {}\n",
                now_utc(), el / 3600.0, s.4, s.2, s.1, s.0, s.3, found.load(Ordering::Relaxed)
            );
            eprint!("{line}");
            if let Some(l) = &log {
                use std::io::Write;
                std::fs::OpenOptions::new().create(true).append(true).open(l).unwrap().write_all(line.as_bytes()).unwrap();
            }
            next += 1800.0;
        }
        if done {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    for h in hs {
        let _ = h.join();
    }
    let wall = t0.elapsed().as_secs_f64();
    let res = result.lock().unwrap().clone();
    if let Some(w) = res.as_ref() {
        assert!(w.accept_gi() && w.accept_cyc());
        let tag = format!("qubit_H-m{m}-rank{rank}-vsplit");
        let path = write_witness(&outdir, &tag, w, "V-member split: two harvested tuples whose spans meet span(|H>^m, |H^perp>^m) in independent lines, exact Z[i] acceptance", wall, wall * threads as f64);
        std::fs::write(format!("{outdir}/{tag}.dec.txt"), dec_text(&w.terms, &format!("qvm_rank1:vsplit-m{m}"))).unwrap();
        println!("WITNESS m={m} rank={rank} found in {wall:.1} s -> {path}");
    } else {
        println!("NO WITNESS m={m} rank={rank} in {wall:.1} s");
    }
}

fn fmt_dir(d: &(Gi, Gi)) -> String {
    let f = |g: &Gi| if g.im == 0 { format!("{}", g.re) } else { format!("{}{:+}i", g.re, g.im) };
    format!("({},{})", f(&d.0), f(&d.1))
}

/// The census of members of V of stabilizer rank <= 3 at M qubits, and the chain test:
/// a sub-tuple of <= 3 terms of a rank-6 decomposition of |H>^7 whose span meets V_7 along d
/// forces d, T(d), T^2(d) to be member directions of rank <= 3 at m = 5 (module docs).
fn cmd_census(args: &[String]) {
    let m: usize = args[0].parse().unwrap();
    let outdir = args[1].clone();
    std::fs::create_dir_all(&outdir).unwrap();
    let t0 = Instant::now();
    let dict = enumerate_all(m);
    let reps = orbit_reps(&dict, m);
    eprintln!("census m={m}: {} states, {} orbit representatives ({:.1} s)", dict.len(), reps.len(), t0.elapsed().as_secs_f64());
    let hits = member_census(m, &dict, &reps, 20260924);
    let mut by_rank: [std::collections::BTreeSet<(Gi, Gi)>; 4] = Default::default();
    let mut txt = String::new();
    for h in &hits {
        // close under the group's action on V (odd Hadamard subsets)
        by_rank[h.rank].insert(h.dir);
        by_rank[h.rank].insert(odd_h_dir(h.dir));
        txt.push_str(&dec_text(&h.terms, &format!("member:rank={}:dir={}", h.rank, fmt_dir(&h.dir))));
    }
    std::fs::write(format!("{outdir}/census_m{m}.txt"), txt).unwrap();
    let mut le3: std::collections::BTreeSet<(Gi, Gi)> = std::collections::BTreeSet::new();
    for r in 1..=3 {
        for d in &by_rank[r] {
            le3.insert(*d);
        }
        let v: Vec<String> = by_rank[r].iter().map(fmt_dir).collect();
        println!("census m={m}: rank-{r} member directions ({}): {}", v.len(), v.join(" "));
    }
    let mut chains = Vec::new();
    for d in &le3 {
        let t1 = slice_t(*d);
        let t2 = slice_t(t1);
        let two = le3.contains(&t1);
        let three = two && le3.contains(&t2);
        if two {
            chains.push(format!("{} -> {} -> {} [{}]", fmt_dir(d), fmt_dir(&t1), fmt_dir(&t2), if three { "d,T,T^2 all members" } else { "d,T members; T^2 not" }));
        }
    }
    println!("census m={m}: directions with d and T(d) both rank<=3 members: {}", chains.len());
    for c in &chains {
        println!("  {c}");
    }
    let full: Vec<&String> = chains.iter().filter(|c| c.contains("all members")).collect();
    println!(
        "census m={m}: chains d, T(d), T^2(d) all of rank <= 3: {} -> {}",
        full.len(),
        if full.is_empty() { "NO split (<=3 + rest) decomposition of rank 6 exists two qubits up (m+2) with property P" } else { "chain(s) exist; the split is not excluded by this test" }
    );
    println!("census m={m}: wall {:.1} s", t0.elapsed().as_secs_f64());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("anneal") => cmd_anneal(&args[2..]),
        Some("exhaust4") => cmd_exhaust4(&args[2..]),
        Some("lift") => cmd_lift(&args[2..]),
        Some("tower") => cmd_tower(&args[2..]),
        Some("vsplit") => cmd_vsplit(&args[2..]),
        Some("census") => cmd_census(&args[2..]),
        _ => {
            eprintln!("usage: qvm_rank1 anneal M RANK THREADS MINUTES OUTDIR [--filter] [--galois] [--moves N] [--seeds FILE] | exhaust4 OUTDIR");
            std::process::exit(2);
        }
    }
}
