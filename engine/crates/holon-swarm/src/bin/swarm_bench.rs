//! Measurement harness for the sharded holon swarm.
//!
//! Wall time on this machine, `--release`. Run:
//!
//! ```text
//! cargo run --release --manifest-path .../holon-swarm/Cargo.toml --bin swarm_bench
//! ```
//!
//! ## Measurement hygiene, stated before the numbers
//!
//! **This machine is shared and heavily loaded.** The 1-minute load average is printed at
//! the top and the bottom; when it approaches or exceeds `nproc`, a wall-clock speedup
//! curve measures the scheduler as much as it measures the code. Two defences are used:
//!
//! 1. every timing is the **minimum over `SAMPLES` runs**, because interference can only
//!    make a run slower, never faster — the minimum is the least-contaminated estimator;
//! 2. every timing also reports **cores obtained** = (process CPU time) / (wall time) over
//!    the same window, read from `/proc/self/stat`. This separates the two questions that
//!    a raw speedup number fuses together: *did the algorithm parallelise* (cores obtained
//!    should track the thread count) and *did the machine have cores to give* (it did not).
//!    A run that asks for 16 threads and obtains 5 cores is not evidence about the code.
//!
//! Threads are NOT pinned (no external crates, so no affinity API), and the CPU is hybrid:
//! 8 performance cores (16 threads, 5.2-5.4 GHz) plus 16 efficiency cores (3.9 GHz). A
//! 16-thread run may land on any mixture, which is a second, irreducible source of spread.
//!
//! ## Tables
//!   A. exchange-round wall time vs shard count N at a FIXED TOTAL holon count. Three
//!      columns keep two effects apart: `seq` isolates the LOCALITY gain from sharding
//!      (same work, one thread, smaller working set), `seq/par` the PARALLEL gain at that N.
//!   B. 16 shards fixed, thread count varied — thread scaling with geometry frozen.
//!   B2. boundaryless swarm — separates the shards' internal work from the boundary protocol.
//!   C. gate cost by level.
//!   D. per-exchange cost, as the marginal slope on a boundary-dominated configuration.
//!   E. conservation and determinism verdicts (these are exact, not timings, and are
//!      unaffected by load).

use std::time::Instant;

use holon_swarm::{FaultInjection, GateLevel, RoundOrder, Swarm, SwarmSpec};

/// Total terminal holons across the whole swarm, held fixed as N varies.
const TOTAL_LEAVES: usize = 262_144;
const WARMUP: u64 = 2;
const ROUNDS: u64 = 20;
const SAMPLES: usize = 15;
/// `USER_HZ`, the unit of `/proc/self/stat` utime/stime. 100 on every Linux x86_64 build;
/// `sysconf(_SC_CLK_TCK)` would need libc, which this crate deliberately does not take.
const USER_HZ: f64 = 100.0;

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(str::to_owned))
        .unwrap_or_else(|| "?".into())
}

/// Process-wide CPU seconds (all threads), from `/proc/self/stat` fields 14 and 15.
/// Resolution is one tick = 10 ms, so measurement windows are sized to be >= ~100 ms.
fn cpu_seconds() -> f64 {
    let Ok(stat) = std::fs::read_to_string("/proc/self/stat") else {
        return 0.0;
    };
    // The comm field can contain spaces and parentheses; split after the last ')'.
    let Some(rest) = stat.rsplit_once(')') else {
        return 0.0;
    };
    let fields: Vec<&str> = rest.1.split_whitespace().collect();
    // After the ')' the first field is state; utime is field 14 overall = index 11 here.
    let utime: f64 = fields.get(11).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let stime: f64 = fields.get(12).and_then(|s| s.parse().ok()).unwrap_or(0.0);
    (utime + stime) / USER_HZ
}

#[derive(Clone, Copy)]
struct Sample {
    /// Wall microseconds per exchange round.
    us: f64,
    /// Process CPU microseconds per exchange round, summed over all worker threads. This
    /// is the TOTAL WORK, and it is contention-invariant: another process stealing a core
    /// makes wall time worse but does not add CPU time to this process. It is therefore
    /// the only number in this report that can be compared across table rows measured
    /// minutes apart on a machine whose load is moving.
    cpu_us: f64,
    /// (process CPU time) / (wall time) over the window: how many cores this run obtained.
    cores: f64,
}

struct Timing {
    best: Sample,
    worst_us: f64,
    min_cpu_us: f64,
}

impl Timing {
    fn spread(&self) -> f64 {
        self.worst_us / self.best.us
    }
    /// Contention-invariant work, in CPU nanoseconds per holon per round.
    fn cpu_ns_per_holon(&self, holons: usize) -> f64 {
        self.min_cpu_us * 1000.0 / holons as f64
    }
}

fn measure(mut run: impl FnMut() -> Sample) -> Timing {
    let mut best = Sample {
        us: f64::INFINITY,
        cpu_us: 0.0,
        cores: 0.0,
    };
    let mut worst_us: f64 = 0.0;
    let mut min_cpu_us = f64::INFINITY;
    for _ in 0..SAMPLES {
        let s = run();
        if s.us < best.us {
            best = s;
        }
        worst_us = worst_us.max(s.us);
        min_cpu_us = min_cpu_us.min(s.cpu_us);
    }
    Timing {
        best,
        worst_us,
        min_cpu_us,
    }
}

fn threaded(spec: &SwarmSpec, threads: usize, rounds: u64) -> Sample {
    let mut swarm = Swarm::new(spec).expect("swarm builds");
    swarm
        .run_rounds_threaded(WARMUP, threads, FaultInjection::None)
        .expect("warmup passes the gate");
    let c0 = cpu_seconds();
    let w0 = Instant::now();
    swarm
        .run_rounds_threaded(rounds, threads, FaultInjection::None)
        .expect("timed run passes the gate");
    let wall = w0.elapsed().as_secs_f64();
    let cpu = cpu_seconds() - c0;
    Sample {
        us: wall * 1.0e6 / rounds as f64,
        cpu_us: cpu * 1.0e6 / rounds as f64,
        cores: cpu / wall.max(1.0e-9),
    }
}

fn sequential(spec: &SwarmSpec, rounds: u64) -> Sample {
    let mut swarm = Swarm::new(spec).expect("swarm builds");
    swarm
        .run_rounds_sequential(WARMUP, FaultInjection::None)
        .expect("warmup passes the gate");
    let c0 = cpu_seconds();
    let w0 = Instant::now();
    swarm
        .run_rounds_sequential(rounds, FaultInjection::None)
        .expect("timed run passes the gate");
    let wall = w0.elapsed().as_secs_f64();
    let cpu = cpu_seconds() - c0;
    Sample {
        us: wall * 1.0e6 / rounds as f64,
        cpu_us: cpu * 1.0e6 / rounds as f64,
        cores: cpu / wall.max(1.0e-9),
    }
}

fn main() {
    let nproc = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(0);
    println!("holon-swarm measurement report");
    println!("build: cargo --release (opt-level 3)");
    println!("nproc (available_parallelism): {nproc}");
    println!("1-min load average at start: {}   <-- read this before any speedup number", loadavg());
    println!(
        "total terminal holons held fixed at {TOTAL_LEAVES}; \
         {ROUNDS} rounds per sample after {WARMUP} warmup; min of {SAMPLES} samples"
    );
    println!(
        "GrossState = {} bytes; the live ledger overlay is a flat {}-byte-stride buffer",
        core::mem::size_of::<ciris_sim_core::regplus::GrossState>(),
        core::mem::size_of::<ciris_sim_core::regplus::GrossState>()
    );
    println!();

    // ---------------------------------------------------------------- Table A
    println!("== A. exchange round vs shard count N (N shards, N threads, gate=Full) ==");
    println!(
        "{:>3} {:>10} {:>6} {:>10} {:>7} {:>7} {:>10} {:>9} {:>9} {:>9} {:>10}",
        "N", "leaves/sh", "pairs", "par (us)", "spread", "cores", "seq (us)", "par gain", "seq gain",
        "ns/holon", "cpuns/hol"
    );
    let mut seq_base = 0.0f64;
    for n in [1usize, 2, 4, 8, 16] {
        let spec = SwarmSpec::ring(n, TOTAL_LEAVES / n).with_gate(GateLevel::Full);
        let par = measure(|| threaded(&spec, n, ROUNDS));
        let seq = measure(|| sequential(&spec, ROUNDS));
        let pairs = Swarm::new(&spec).unwrap().pairs().len();
        if n == 1 {
            seq_base = seq.best.us;
        }
        println!(
            "{:>3} {:>10} {:>6} {:>10.1} {:>6.2}x {:>7.2} {:>10.1} {:>8.2}x {:>8.2}x {:>9.2} {:>10.2}",
            n,
            TOTAL_LEAVES / n,
            pairs,
            par.best.us,
            par.spread(),
            par.best.cores,
            seq.best.us,
            seq.best.us / par.best.us,
            seq_base / seq.best.us,
            par.best.us * 1000.0 / TOTAL_LEAVES as f64,
            par.cpu_ns_per_holon(TOTAL_LEAVES),
        );
    }
    println!("  'par gain' = seq(N)/par(N): parallelism only, at this shard geometry.");
    println!("  'seq gain' = seq(1)/seq(N): sharding's cache-locality gain, single-threaded.");
    println!("  'cores'    = CPU seconds / wall seconds during the best sample: the cores the");
    println!("               machine actually granted. Compare against N before reading 'par gain'.");
    println!("  'cpuns/hol'= CPU nanoseconds of TOTAL WORK per holon per round. Contention-");
    println!("               invariant: it does not move when another process steals a core.");
    println!("               It is NOT core-type invariant: this CPU is hybrid (8 P-cores at");
    println!("               5.2-5.4 GHz, 16 E-cores at 3.9 GHz), so the same work costs more");
    println!("               CPU-seconds on an E-core. Read a rise in this column at high thread");
    println!("               counts as 'work migrated onto E-cores', not as 'the code got worse'.");
    println!();

    // ---------------------------------------------------------------- Table B
    println!("== B. 16 shards fixed, thread count varied (gate=Full, 16384 holons/shard) ==");
    println!(
        "{:>8} {:>11} {:>7} {:>7} {:>9} {:>11} {:>13} {:>10}",
        "threads", "round (us)", "spread", "cores", "speedup", "eff vs req", "eff vs cores",
        "cpuns/hol"
    );
    let spec16 = SwarmSpec::ring(16, TOTAL_LEAVES / 16).with_gate(GateLevel::Full);
    let mut base16 = 0.0f64;
    for threads in [1usize, 2, 4, 8, 16] {
        let t = measure(|| threaded(&spec16, threads, ROUNDS));
        if threads == 1 {
            base16 = t.best.us;
        }
        let speedup = base16 / t.best.us;
        println!(
            "{:>8} {:>11.1} {:>6.2}x {:>7.2} {:>8.2}x {:>10.0}% {:>12.0}% {:>10.2}",
            threads,
            t.best.us,
            t.spread(),
            t.best.cores,
            speedup,
            100.0 * speedup / threads as f64,
            100.0 * speedup / t.best.cores.max(1.0),
            t.cpu_ns_per_holon(TOTAL_LEAVES),
        );
    }
    println!("  'eff vs req'   = speedup / threads requested (contaminated by machine load).");
    println!("  'eff vs cores' = speedup / cores actually obtained (the algorithm's own efficiency).");
    println!("  'cpuns/hol'    = total CPU work per holon per round: flat means threading adds no work.");
    println!("                   Its rise at 16 threads is the hybrid CPU (E-cores cost more");
    println!("                   CPU-seconds per unit work), plus barrier wake-ups; the barrier");
    println!("                   also makes every round as slow as its slowest shard, which is");
    println!("                   why 'eff vs cores' falls even though the shards never contend.");
    println!();

    // ---------------------------------------------------------------- Table B2
    println!("== B2. where the work is: boundaryless swarm vs ring, SINGLE THREAD ==");
    println!("  PAIRED sampling: the two configurations are measured back-to-back inside each");
    println!("  sample, so both meet the same contention, and the reported statistic is the");
    println!("  MEDIAN of the per-sample ratios. Comparing two separately-taken minima on a");
    println!("  loaded machine produced a nonsense -30% in an earlier run; this is the fix.");
    let isolated = SwarmSpec::isolated(16, TOTAL_LEAVES / 16).with_gate(GateLevel::Full);
    let mut ratios = Vec::new();
    let mut local_min = f64::INFINITY;
    let mut full_min = f64::INFINITY;
    for _ in 0..SAMPLES {
        let l = threaded(&isolated, 1, ROUNDS);
        let f = threaded(&spec16, 1, ROUNDS);
        ratios.push(f.us / l.us);
        local_min = local_min.min(l.us);
        full_min = full_min.min(f.us);
    }
    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_ratio = ratios[ratios.len() / 2];
    println!(
        "  16 shards, no boundaries  : {:>9.1} us/round (best) = {:.2} ns/holon",
        local_min,
        local_min * 1000.0 / TOTAL_LEAVES as f64
    );
    println!(
        "  16 shards, ring (16 pairs): {:>9.1} us/round (best) = {:.2} ns/holon",
        full_min,
        full_min * 1000.0 / TOTAL_LEAVES as f64
    );
    println!(
        "  paired median ratio ring/boundaryless: {:.4}  (range {:.3}..{:.3} over {} samples)",
        median_ratio,
        ratios[0],
        ratios[ratios.len() - 1],
        ratios.len()
    );
    println!("  Expected effect from table D: 16 pairs x ~0.1 us = ~1.6 us on a ~10000 us round");
    println!("  = ~0.016%. That is ~3 orders of magnitude below this measurement's spread, so the");
    println!("  honest conclusion is: at 16384 holons per pair the boundary protocol is BELOW");
    println!("  RESOLUTION here. Table D measures it directly, where it is not swamped.");
    println!();

    // ---------------------------------------------------------------- Table C
    println!("== C. gate cost by level (8 shards, SINGLE THREAD, 32768 holons/shard) ==");
    println!("  Single-threaded on purpose: comparing gate levels across runs that obtained");
    println!("  different core counts would measure the scheduler, not the gate.");
    println!(
        "{:>10} {:>11} {:>7} {:>15} {:>11} {:>16}",
        "level", "round (us)", "spread", "vs Ledger (us)", "cpuns/hol", "gate share"
    );
    let mut ledger_us = 0.0f64;
    let mut ledger_cpu = 0.0f64;
    for level in [GateLevel::Ledger, GateLevel::Full, GateLevel::Paranoid] {
        let spec = SwarmSpec::ring(8, TOTAL_LEAVES / 8).with_gate(level);
        let t = measure(|| threaded(&spec, 1, ROUNDS));
        if level == GateLevel::Ledger {
            ledger_us = t.best.us;
            ledger_cpu = t.min_cpu_us;
        }
        println!(
            "{:>10} {:>11.1} {:>6.2}x {:>15.1} {:>11.2} {:>15.1}%",
            format!("{level:?}"),
            t.best.us,
            t.spread(),
            t.best.us - ledger_us,
            t.cpu_ns_per_holon(TOTAL_LEAVES),
            100.0 * (t.min_cpu_us - ledger_cpu) / t.min_cpu_us,
        );
    }
    println!("  'gate share' = the extra CPU work this level adds, as a share of its own round.");
    println!("  GateLevel::Ledger is the floor here, not zero: it still runs L1-L5 every round.");
    println!();

    // ---------------------------------------------------------------- Table D
    println!("== D. per-exchange cost (boundary-dominated: 4 leaves/shard, 1 thread) ==");
    println!(
        "{:>5} {:>7} {:>12} {:>7} {:>12}",
        "N", "pairs", "round (us)", "spread", "us/pair"
    );
    let mut points: Vec<(f64, f64)> = Vec::new();
    for n in [2usize, 4, 16, 64, 256, 1024] {
        let spec = SwarmSpec::ring(n, 4).with_gate(GateLevel::Ledger);
        let t = measure(|| threaded(&spec, 1, 4000));
        let pairs = Swarm::new(&spec).unwrap().pairs().len();
        points.push((pairs as f64, t.best.us));
        println!(
            "{:>5} {:>7} {:>12.3} {:>6.2}x {:>12.4}",
            n,
            pairs,
            t.best.us,
            t.spread(),
            t.best.us / pairs.max(1) as f64
        );
    }
    let n = points.len() as f64;
    let sx: f64 = points.iter().map(|p| p.0).sum();
    let sy: f64 = points.iter().map(|p| p.1).sum();
    let sxx: f64 = points.iter().map(|p| p.0 * p.0).sum();
    let sxy: f64 = points.iter().map(|p| p.0 * p.1).sum();
    let slope = (n * sxy - sx * sy) / (n * sxx - sx * sx);
    let intercept = (sy - slope * sx) / n;
    println!(
        "  MARGINAL cost per exchange (least-squares slope): {:.1} ns per boundary pair per round",
        slope * 1000.0
    );
    println!(
        "  fixed per-round cost (intercept): {:.2} us  [gate reduction + 3 Vec allocations]",
        intercept
    );
    println!("  each pair costs: 2 plans, 2 checked port writes, 2 checked root writes,");
    println!("  2x4 atomic snapshot stores, 2x4 atomic receipt stores, and gate legs L2+L3.");
    println!();

    // ---------------------------------------------------------------- Table E
    println!("== E. conservation and determinism (exact; unaffected by machine load) ==");
    let spec = SwarmSpec::ring(8, 4096).with_gate(GateLevel::Paranoid);
    let opening = Swarm::new(&spec).unwrap().global_ledger().unwrap();
    println!("opening global ledger: {opening:?}");

    let mut reference = Swarm::new(&spec).unwrap();
    reference
        .run_rounds_sequential(80, FaultInjection::None)
        .expect("80 sequential rounds pass the gate at level Paranoid (L1-L7)");
    let reference_fp = reference.full_fingerprint();
    let closing = reference.global_ledger().unwrap();
    println!("closing global ledger after 80 rounds: {closing:?}");
    println!(
        "  CONSERVATION: exact, bit-identical in all four integer lanes: {}",
        closing == opening
    );
    println!("  the gate passed on all 80 rounds at level Paranoid (L1-L7), or this line would not print");
    println!(
        "  the exchange did move quantity: {} of 8 shard balances differ from their g0",
        reference
            .shards()
            .iter()
            .filter(|s| s.root_ledger() != s.g0())
            .count()
    );

    let mut mismatch = Vec::new();
    for threads in [1usize, 2, 3, 4, 8, 16, 32] {
        for repeat in 0..4 {
            let mut run = Swarm::new(&spec).unwrap();
            run.run_rounds_threaded(80, threads, FaultInjection::None)
                .unwrap();
            if run.full_fingerprint() != reference_fp {
                mismatch.push((threads, repeat));
            }
        }
    }
    println!(
        "  DETERMINISM: threaded == sequential, bit-identical per-HOLON ledger and\n   \
         whole-state to_bits(), threads in {{1,2,3,4,8,16,32}} x 4 repeats: {}",
        if mismatch.is_empty() {
            "IDENTICAL".to_string()
        } else {
            format!("MISMATCHES {mismatch:?}")
        }
    );

    let mut order_mismatch = 0;
    for order in [
        RoundOrder::natural(8, 8),
        RoundOrder::reversed(8, 8),
        RoundOrder::strided(8, 8),
    ] {
        let mut s = Swarm::new(&spec).unwrap();
        for _ in 0..80 {
            s.step_round_sequential(&order, FaultInjection::None)
                .unwrap();
        }
        if s.full_fingerprint() != reference_fp {
            order_mismatch += 1;
        }
    }
    println!(
        "  ORDER INDEPENDENCE: identical under natural/reversed/strided visit order: {}",
        order_mismatch == 0
    );
    println!();
    println!("1-min load average at end: {}", loadavg());
}
