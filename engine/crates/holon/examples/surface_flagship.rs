//! THE FLAGSHIP — a rotated surface code at scale, with real adaptive
//! syndrome extraction, on the column-major engine.
//!
//! Two modes, deliberately separated so the demonstration and the benchmark
//! are never the same run wearing two hats:
//!
//!   --mode qec    (default) the DEMONSTRATION: rounds of syndrome
//!                 extraction, injected X errors, a decoder that reads the
//!                 mid-circuit outcomes, feed-forward corrections, and four
//!                 verifications including the LOGICAL observable.
//!   --mode bench  the COMPARISON: R rounds of plain extraction and nothing
//!                 else — the exact shape `--stim` emits, so the head-to-head
//!                 runs on the identical circuit.
//!
//! Reproduce:
//!   cargo run --release --example surface_flagship -- --d 221 --rounds 3
//!   cargo run --release --example surface_flagship -- --d 221 --mode bench \
//!       --stim /tmp/sc221.stim
//!
//! The machine is shared. Every run states its working set before allocating
//! and REFUSES if MemAvailable cannot carry it with 2 GB left over.

use holon::coladaptive::ColAdaptive;
use holon::surface::{Kind, SurfaceCode};
use std::time::Instant;

fn mem_available() -> u64 {
    let s = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("MemAvailable:") {
            return rest.trim().trim_end_matches(" kB").trim().parse().unwrap_or(0) * 1024;
        }
    }
    0
}

fn peak_rss() -> u64 {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            return rest.trim().trim_end_matches(" kB").trim().parse().unwrap_or(0) * 1024;
        }
    }
    0
}

/// The honest working set: a column-major tableau AND the row-major
/// reference, both `n²/2` bytes.
fn working_set_bytes(n: usize) -> u64 {
    let n = n as u64;
    let col = n * (2 * n).div_ceil(64) * 8 * 2;
    let words = n.div_ceil(64);
    let row = 2 * n * 2 * words * 8;
    col + row
}

/// One full syndrome-extraction round: the four-step schedule on the column
/// engine, then the whole ancilla batch measured through one transpose, then
/// the deferred resets. Returns the syndrome and the phase timings.
fn round(a: &mut ColAdaptive, code: &SurfaceCode) -> (Vec<bool>, f64, f64) {
    let t0 = Instant::now();
    for s in &code.stabs {
        if s.kind == Kind::X {
            a.h(s.ancilla);
        }
    }
    for t in 0..4 {
        for s in &code.stabs {
            if let Some(q) = s.sched[t] {
                match s.kind {
                    Kind::Z => a.cx(q, s.ancilla),
                    Kind::X => a.cx(s.ancilla, q),
                }
            }
        }
    }
    for s in &code.stabs {
        if s.kind == Kind::X {
            a.h(s.ancilla);
        }
    }
    let gate_s = t0.elapsed().as_secs_f64();

    let t1 = Instant::now();
    a.begin_batch();
    let syn: Vec<bool> = code.stabs.iter().map(|s| a.measure(s.ancilla).0).collect();
    a.end_batch();
    let meas_s = t1.elapsed().as_secs_f64();

    // Deferred resets — exact, see coladaptive.rs's header.
    for (k, s) in code.stabs.iter().enumerate() {
        if syn[k] {
            a.x_gate(s.ancilla);
        }
    }
    (syn, gate_s, meas_s)
}

/// Per data qubit, the Z-plaquettes an X error there would light.
fn z_signatures(code: &SurfaceCode) -> Vec<Vec<usize>> {
    let mut sig = vec![Vec::new(); code.n_data()];
    for (k, s) in code.stabs.iter().enumerate() {
        if s.kind == Kind::Z {
            for q in s.sched.iter().flatten() {
                sig[*q].push(k);
            }
        }
    }
    sig
}

/// An isolated-error lookup decoder: a data qubit is corrected when EVERY
/// plaquette in its signature fired. Exact for the injected model (single X
/// errors separated by more than two lattice steps) and no more than that —
/// a full minimum-weight matching decoder is a different piece of work and is
/// not claimed here. What is demonstrated is the FEED-FORWARD: the correction
/// is computed from mid-circuit outcomes and applied to the live state.
fn decode(sig: &[Vec<usize>], flipped: &[bool]) -> (Vec<usize>, usize) {
    let mut remaining: Vec<bool> = flipped.to_vec();
    let mut corr = Vec::new();
    for (q, s) in sig.iter().enumerate() {
        if !s.is_empty() && s.iter().all(|&k| remaining[k]) {
            for &k in s {
                remaining[k] = false;
            }
            corr.push(q);
        }
    }
    (corr, remaining.iter().filter(|&&b| b).count())
}

/// Emit the identical circuit in stim's format, so the head-to-head compares
/// engines rather than circuit generators.
fn emit_stim(code: &SurfaceCode, rounds: usize) -> String {
    let mut out = String::new();
    let xs: Vec<usize> = code
        .stabs
        .iter()
        .filter(|s| s.kind == Kind::X)
        .map(|s| s.ancilla)
        .collect();
    let anc: Vec<usize> = code.stabs.iter().map(|s| s.ancilla).collect();
    let join = |v: &[usize]| {
        v.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    };
    for _ in 0..rounds {
        out.push_str(&format!("H {}\n", join(&xs)));
        for t in 0..4 {
            let mut pairs = Vec::new();
            for s in &code.stabs {
                if let Some(q) = s.sched[t] {
                    match s.kind {
                        Kind::Z => pairs.extend_from_slice(&[q, s.ancilla]),
                        Kind::X => pairs.extend_from_slice(&[s.ancilla, q]),
                    }
                }
            }
            if !pairs.is_empty() {
                out.push_str(&format!("CX {}\n", join(&pairs)));
            }
        }
        out.push_str(&format!("H {}\n", join(&xs)));
        out.push_str(&format!("M {}\n", join(&anc)));
        out.push_str(&format!("R {}\n", join(&anc)));
    }
    out
}

fn arg<T: std::str::FromStr>(args: &[String], key: &str, dflt: T) -> T {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(dflt)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let d: usize = arg(&args, "--d", 21);
    let rounds: usize = arg(&args, "--rounds", 3);
    let seed: u64 = arg(&args, "--seed", 1);
    let mode: String = arg(&args, "--mode", "qec".to_string());
    let n_err: usize = arg(&args, "--errors", 3);
    let stim_path: String = arg(&args, "--stim", String::new());
    let json_path: String = arg(&args, "--json", String::new());
    let no_guard = args.iter().any(|a| a == "--no-guard");
    // Test hook, symmetric to --no-guard: refuse unconditionally, so the
    // harness's per-size skip path can be EXERCISED rather than assumed. A
    // fallback nothing ever runs is an untested claim.
    let force_refuse = args.iter().any(|a| a == "--force-refuse");

    let build = Instant::now();
    let code = SurfaceCode::new(d);
    let build_s = build.elapsed().as_secs_f64();
    let n = code.n;

    if !stim_path.is_empty() {
        let s = emit_stim(&code, rounds);
        std::fs::write(&stim_path, &s).expect("write stim circuit");
        eprintln!("wrote stim circuit: {stim_path} ({} bytes)", s.len());
    }

    let need = working_set_bytes(n);
    let avail = mem_available();
    eprintln!(
        "surface code d={d}: n={n} qubits ({} data + {} ancilla), {} stabilizers",
        code.n_data(),
        code.stabs.len(),
        code.stabs.len()
    );
    eprintln!(
        "working set {:.2} GB (column engine + row-major reference), MemAvailable {:.2} GB",
        need as f64 / 1e9,
        avail as f64 / 1e9
    );
    if force_refuse {
        eprintln!("REFUSED: --force-refuse (test hook for the harness skip path)");
        std::process::exit(2);
    }
    if !no_guard && need + (2u64 << 30) > avail {
        eprintln!(
            "REFUSED: needs {:.2} GB + 2.0 GB reserve, only {:.2} GB available. \
             This box is shared — lower --d or wait.",
            need as f64 / 1e9,
            avail as f64 / 1e9
        );
        std::process::exit(2);
    }

    let alloc = Instant::now();
    let mut a = ColAdaptive::new(n, seed);
    let alloc_s = alloc.elapsed().as_secs_f64();
    eprintln!("allocated in {alloc_s:.3} s");

    let mut gate_total = 0.0;
    let mut meas_total = 0.0;
    let mut round_times = Vec::new();
    let mut checks: Vec<(String, bool)> = Vec::new();
    let t_all = Instant::now();

    if mode == "bench" {
        // Matched to `--stim`: R rounds, nothing else.
        for r in 0..rounds {
            let t = Instant::now();
            let (_syn, g, m) = round(&mut a, &code);
            gate_total += g;
            meas_total += m;
            let el = t.elapsed().as_secs_f64();
            round_times.push(el);
            eprintln!("  round {:2}: {:8.3} s  (gates {:.3} s, measure {:.3} s)", r + 1, el, g, m);
        }
    } else {
        // ---- round 1: establish the codestate ----
        let (s1, g, m) = round(&mut a, &code);
        gate_total += g;
        meas_total += m;
        round_times.push(g + m);
        eprintln!("  round  1 (establish): {:8.3} s  (gates {g:.3} s, measure {m:.3} s)", g + m);
        let z_silent = code
            .stabs
            .iter()
            .enumerate()
            .all(|(i, s)| s.kind != Kind::Z || !s1[i]);
        checks.push(("Z syndromes silent on |0...0>".into(), z_silent));

        let logical_before = a.z_string_value(&code.logical_z());
        checks.push((
            "logical Z determined and +1 before errors".into(),
            logical_before == Some(false),
        ));

        // ---- round 2: a noiseless repeat must reproduce round 1 exactly ----
        let (s2, g, m) = round(&mut a, &code);
        gate_total += g;
        meas_total += m;
        round_times.push(g + m);
        eprintln!("  round  2 (repeat):    {:8.3} s  (gates {g:.3} s, measure {m:.3} s)", g + m);
        checks.push(("noiseless round repeats exactly".into(), s2 == s1));

        // ---- inject well-separated X errors ----
        let step = (d / (n_err + 1)).max(3);
        let bad: Vec<usize> = (0..n_err)
            .map(|k| {
                let r = (k + 1) * step % d;
                let c = ((k + 1) * step + k) % d;
                code.data(r, c)
            })
            .collect();
        let mut bad_sorted = bad.clone();
        bad_sorted.sort_unstable();
        bad_sorted.dedup();
        for &q in &bad_sorted {
            a.x_gate(q);
        }
        eprintln!("  injected X errors on data qubits {bad_sorted:?}");

        // ---- round 3: the errors must show up, exactly where they are ----
        let (s3, g, m) = round(&mut a, &code);
        gate_total += g;
        meas_total += m;
        round_times.push(g + m);
        eprintln!("  round  3 (errors):    {:8.3} s  (gates {g:.3} s, measure {m:.3} s)", g + m);

        let sig = z_signatures(&code);
        let mut expect = vec![false; code.stabs.len()];
        for &q in &bad_sorted {
            for &k in &sig[q] {
                expect[k] ^= true;
            }
        }
        let flipped: Vec<bool> = s3.iter().zip(&s1).map(|(a, b)| a != b).collect();
        checks.push(("injected errors light exactly their plaquettes".into(), flipped == expect));

        // ---- decode from the mid-circuit outcomes, and feed forward ----
        let (corr, unmatched) = decode(&sig, &flipped);
        eprintln!("  decoder: {} corrections, {unmatched} unmatched syndromes", corr.len());
        checks.push(("decoder explains every fired plaquette".into(), unmatched == 0));
        for &q in &corr {
            a.x_gate(q);
        }

        // ---- round 4: back in the codespace ----
        let (s4, g, m) = round(&mut a, &code);
        gate_total += g;
        meas_total += m;
        round_times.push(g + m);
        eprintln!("  round  4 (corrected): {:8.3} s  (gates {g:.3} s, measure {m:.3} s)", g + m);
        checks.push(("syndromes return to their pre-error values".into(), s4 == s1));

        // ---- and the LOGICAL bit must have survived ----
        let logical_after = a.z_string_value(&code.logical_z());
        checks.push((
            "logical Z still determined and unchanged".into(),
            logical_after == logical_before && logical_after.is_some(),
        ));
    }

    let wall = t_all.elapsed().as_secs_f64();
    let st = a.stats;
    let total_meas = st.deterministic + st.random;
    let gate_ops = rounds_gate_ops(&code) * round_times.len();

    eprintln!();
    eprintln!("  wall {wall:.3} s   gates {gate_total:.3} s   measure {meas_total:.3} s");
    eprintln!(
        "  {} measurements: {} deterministic, {} random; scan fast {} / fallback {}",
        total_meas, st.deterministic, st.random, st.scan_fast, st.scan_fallback
    );
    eprintln!(
        "  destabilizer product terms: {} total, mean {:.2}; single-term {} ({:.1}%)",
        st.product_terms,
        st.product_terms as f64 / st.deterministic.max(1) as f64,
        st.single_term,
        100.0 * st.single_term as f64 / st.deterministic.max(1) as f64
    );
    eprintln!(
        "  transposes {}; cascade terms mean {:.1}; pivot X-weight mean {:.1} max {}",
        st.transposes,
        st.cascade_terms as f64 / st.random.max(1) as f64,
        st.pivot_weight as f64 / st.random.max(1) as f64,
        st.pivot_weight_max
    );
    eprintln!(
        "  throughput: {:.0} qubit-rounds/s, {:.0} measurements/s, {:.0} gate-ops/s",
        (n * round_times.len()) as f64 / wall,
        total_meas as f64 / wall,
        gate_ops as f64 / gate_total.max(1e-9)
    );
    eprintln!("  peak RSS {:.3} GB", peak_rss() as f64 / 1e9);

    let all_ok = checks.iter().all(|(_, ok)| *ok);
    if !checks.is_empty() {
        eprintln!();
        for (name, ok) in &checks {
            eprintln!("  [{}] {name}", if *ok { "PASS" } else { "FAIL" });
        }
    }

    // ---- the Qiskit Result schema, extras under metadata ----
    let checks_json = checks
        .iter()
        .map(|(k, v)| format!("\"{k}\": {v}"))
        .collect::<Vec<_>>()
        .join(", ");
    let rt = round_times
        .iter()
        .map(|t| format!("{t:.6}"))
        .collect::<Vec<_>>()
        .join(", ");
    let json = format!(
        "{{\"backend_name\": \"cirisholon\", \"backend_version\": \"0.1.0\", \
         \"success\": {all_ok}, \"results\": [{{\"shots\": 1, \"status\": \"DONE\", \
         \"header\": {{\"name\": \"rotated_surface_code_d{d}\", \"n_qubits\": {n}, \
         \"memory_slots\": {}}}, \
         \"data\": {{\"memory\": []}}, \
         \"metadata\": {{\"exact\": true, \"tier\": \"clifford-adaptive\", \
         \"engine\": \"coladaptive (column-major gates, row-major rowsums)\", \
         \"mode\": \"{mode}\", \"seed\": {seed}, \"distance\": {d}, \
         \"n_data\": {}, \"n_ancilla\": {}, \"rounds\": {}, \
         \"measurements\": {{\"total\": {total_meas}, \"deterministic\": {}, \
         \"random\": {}, \"scan_fast\": {}, \"scan_fallback\": {}, \
         \"product_terms_mean\": {:.4}}}, \
         \"timing_seconds\": {{\"build\": {build_s:.6}, \"alloc\": {alloc_s:.6}, \
         \"gates\": {gate_total:.6}, \"measure\": {meas_total:.6}, \"wall\": {wall:.6}, \
         \"per_round\": [{rt}]}}, \
         \"working_set_bytes\": {need}, \"peak_rss_bytes\": {}, \
         \"verification\": {{{checks_json}}}}}}}]}}",
        code.stabs.len(),
        code.n_data(),
        code.stabs.len(),
        round_times.len(),
        st.deterministic,
        st.random,
        st.scan_fast,
        st.scan_fallback,
        st.product_terms as f64 / st.deterministic.max(1) as f64,
        peak_rss(),
    );
    if json_path.is_empty() {
        println!("{json}");
    } else {
        std::fs::write(&json_path, &json).expect("write json");
        eprintln!("wrote {json_path}");
        println!("{json}");
    }

    if !all_ok {
        eprintln!("VERIFICATION FAILED");
        std::process::exit(1);
    }
}

/// Gate applications in one round: two H per X-plaquette plus one CX per
/// scheduled data qubit.
fn rounds_gate_ops(code: &SurfaceCode) -> usize {
    let hs = code.stabs.iter().filter(|s| s.kind == Kind::X).count() * 2;
    let cxs: usize = code.stabs.iter().map(|s| s.weight()).sum();
    hs + cxs
}
