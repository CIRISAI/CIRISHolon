//! **The (O,O,O) FCI sigma, GPU against CPU, with the device class declared and the placement
//! pinned.**
//!
//! Two misfits shape every line of output here, and they are the same rule one level apart:
//!
//! * **M-DEVICE-CLASS** — a bit-gated artifact must declare its DEVICE class, because the
//!   trailing bits are a function of it. Every row below says which class produced it.
//! * **M-PLACEMENT-LOTTERY** — a citable TIMING must equally declare its CORE class, because
//!   the ratio is a function of it. This box is an i9-13900HX: P-cores 0-15, E-cores 16-31,
//!   scaling 57%, and a d=101 head-to-head verdict has already FLIPPED with placement on it
//!   (0.822 unpinned "we lead", 1.201 on a P-core "they lead"). So the CPU arm is PINNED, the
//!   pin is ECHOED as the process actually has it rather than as the launcher intended, and
//!   both core types get their own run.
//!
//! Gating on CPU time as well as wall follows from the same misfit: descheduling inflates wall
//! and does not inflate `utime + stime`, and this machine is not quiet.
//!
//! Usage:
//! ```text
//! taskset -c 0  fci_bench --core-type P --reps 20
//! taskset -c 16 fci_bench --core-type E --reps 20
//! ```

use std::time::Instant;

use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::{ci_ints, sigma_direct, FciSpace, MoIntegrals, Order};
use holon_chem::pair::geometry_problem;
use holon_chem::sigma_op::{bit_identity_over_runs, CpuProvider, DeviceClass, SigmaOp, SigmaProvider};
use holon_gpu::{GpuSigmaProvider, VramProbe};
use holon_resource::probe::{Probe, ResourceKind};
use holon_resource::registry::{Determinism, Entry, Registry, SpotCheck, WorkloadKey};

/// Which class a CPU belongs to on a hybrid part, **derived from the machine and CHECKED
/// against the citation**.
///
/// The kernel publishes the split on hybrid Intel parts: `/sys/devices/cpu_core/cpus` and
/// `/sys/devices/cpu_atom/cpus`. `conformance/gravity/MISFITS.md`'s M-PLACEMENT-LOTTERY entry
/// states it for this box as P 0-15, E 16-31.
///
/// Writing `if cpu < 16` would have been a hardcoded per-machine branch — the shape WB-8.7
/// clause (2) refuses — and reading sysfs *instead of* the citation would have made a
/// benchmark's core labels depend on a file nobody checked. So it does both and REFUSES on
/// disagreement: on this box the two must agree, and on a different box they will not, at
/// which point the citation does not apply and a row labelled from it would be wrong.
fn core_class(cpu: usize) -> &'static str {
    let members = |path: &str| -> Option<Vec<usize>> {
        let s = std::fs::read_to_string(path).ok()?;
        let mut out = Vec::new();
        for part in s.trim().split(',') {
            let mut ends = part.split('-').map(|x| x.parse::<usize>());
            match (ends.next(), ends.next()) {
                (Some(Ok(a)), Some(Ok(b))) => out.extend(a..=b),
                (Some(Ok(a)), None) => out.push(a),
                _ => return None,
            }
        }
        Some(out)
    };
    let p = members("/sys/devices/cpu_core/cpus");
    let e = members("/sys/devices/cpu_atom/cpus");
    match (p, e) {
        (Some(p), Some(e)) => {
            // The citation, cross-checked. If the machine and M-PLACEMENT-LOTTERY disagree,
            // this is not the box the entry describes and the run must say so rather than
            // label its rows from an entry that does not apply.
            let cited_p: Vec<usize> = (0..16).collect();
            let cited_e: Vec<usize> = (16..32).collect();
            assert!(
                p == cited_p && e == cited_e,
                "this machine reports P-cores {p:?} and E-cores {e:?}, and \
                 conformance/gravity/MISFITS.md's M-PLACEMENT-LOTTERY entry describes an \
                 i9-13900HX with P 0-15, E 16-31. This is a different box, so that entry's \
                 core-class labels do not apply to these rows. Update the entry or run the \
                 benchmark where it does."
            );
            if p.contains(&cpu) {
                "P"
            } else if e.contains(&cpu) {
                "E"
            } else {
                "?"
            }
        }
        // Not a hybrid part, or a kernel that does not publish the split. Refusing to guess
        // is the honest answer: an unlabelled row is recoverable, a wrongly labelled one is
        // the confound M-PLACEMENT-LOTTERY exists to name.
        _ => "?",
    }
}

/// The affinity this process ACTUALLY has, not the one the launcher meant to give it.
///
/// A benchmark that echoes its intended pin rather than its real one is a diagnostic that does
/// not echo its parameters, and that has already cost this repository a day: two commands meant
/// two things in two trees and bit-identical output was the only tell.
fn affinity() -> String {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Cpus_allowed_list:"))
                .map(|l| l.split_whitespace().nth(1).unwrap_or("?").to_string())
        })
        .unwrap_or_else(|| "?".into())
}

/// `utime + stime` for this process, in seconds.
fn cpu_seconds() -> f64 {
    let ticks = std::fs::read_to_string("/proc/self/stat")
        .ok()
        .and_then(|s| {
            let close = s.rfind(')')?;
            let rest: Vec<&str> = s[close + 1..].split_whitespace().collect();
            let ut: u64 = rest.get(11)?.parse().ok()?;
            let st: u64 = rest.get(12)?.parse().ok()?;
            Some(ut + st)
        })
        .unwrap_or(0);
    ticks as f64 / 100.0
}

fn at(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

fn build(symbols: &[String]) -> (FciSpace, MoIntegrals) {
    let species: Vec<_> = symbols
        .iter()
        .map(|s| by_symbol(s).unwrap_or_else(|| panic!("unknown species {s}")))
        .collect();
    let s = 2.4_f64;
    let centers = vec![
        at(0.0, 0.0, 0.0),
        at(s, 0.0, 0.0),
        at(0.5 * s, 0.75_f64.sqrt() * s, 0.0),
    ];
    let (space, mo, _) = geometry_problem(&species, centers);
    (space, mo)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut reps = 20usize;
    let mut species = vec!["O".to_string(), "O".to_string(), "O".to_string()];
    let mut core_type_arg = String::from("unset");
    let mut i = 0;
    while i < args.len() {
        let v = || args.get(i + 1).unwrap_or_else(|| panic!("{} needs a value", args[i])).clone();
        match args[i].as_str() {
            "--reps" => reps = v().parse().expect("--reps"),
            "--species" => species = v().split(',').map(|s| s.to_string()).collect(),
            "--core-type" => core_type_arg = v(),
            other => panic!("unknown argument {other}; this binary refuses what it cannot parse"),
        }
        i += 2;
    }

    let aff = affinity();
    // The claimed core type is CHECKED against the real affinity. A launcher that said P and a
    // taskset that landed on an E-core is exactly the confound M-PLACEMENT-LOTTERY names, and
    // a benchmark that took the launcher's word for it would carry the confound into the table.
    let first_cpu: usize = aff
        .split(&[',', '-'][..])
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(usize::MAX);
    let real_class = if first_cpu == usize::MAX { "?" } else { core_class(first_cpu) };
    if core_type_arg != "unset" && core_type_arg != real_class {
        panic!(
            "--core-type {core_type_arg} was claimed and the process is actually pinned to \
             cpus {aff}, which is class {real_class}. The pin is part of the measurement; a \
             row labelled with the wrong core class is worse than an unlabelled one."
        );
    }

    let load = holon_gpu::loadavg();
    println!("# fci_bench — the FCI sigma, both device classes, placement declared");
    println!("# species        {}", species.join(","));
    println!("# reps           {reps}");
    println!("# cpus_allowed   {aff}   (core class {real_class}, i9-13900HX: P 0-15, E 16-31)");
    println!("# loadavg        {load:.2}   (this machine is NOT quiet; CPU-time is the gate)");

    let t_setup = Instant::now();
    let (space, mo) = build(&species);
    let ci = ci_ints(&mo, Order::Value);
    println!(
        "# problem        n_orb {}, {} x {} strings, {} determinants  (built in {:.1} s)",
        space.n_orb,
        space.alpha.len(),
        space.beta.len(),
        space.n_det,
        t_setup.elapsed().as_secs_f64()
    );

    let mut c = vec![0.0f64; space.n_det];
    let mut seed = 0x243f_6a88_85a3_08d3u64;
    for x in c.iter_mut() {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *x = ((seed >> 33) as f64 / (1u64 << 31) as f64) - 0.5;
    }

    // ---------------- the CPU arm, PINNED, gated on CPU time ----------------
    let mut cpu_sigma = vec![0.0f64; space.n_det];
    sigma_direct(&space, &ci, &c, &mut cpu_sigma); // warm the caches; not timed
    let w0 = Instant::now();
    let p0 = cpu_seconds();
    for _ in 0..reps {
        sigma_direct(&space, &ci, &c, &mut cpu_sigma);
    }
    let cpu_wall = w0.elapsed().as_secs_f64() / reps as f64;
    let cpu_cpu = (cpu_seconds() - p0) / reps as f64;

    // ---------------- the GPU arm ----------------
    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    let (free_mib, total_mib) = {
        let mut p = VramProbe::on(provider.context().clone());
        p.mem_info_mib().expect("device memory unreadable")
    };
    // D2: the probe ATTEMPTS the thing before the benchmark claims the device.
    let need_mib = holon_gpu::lease::LeasedGpuProvider::mib_for(&space);
    let mut probe = VramProbe::on(provider.context().clone());
    let verdict = probe.probe(ResourceKind::Vram, need_mib);
    println!(
        "# device         {free_mib} MiB free of {total_mib}; probe for {need_mib} MiB: {verdict:?}"
    );
    assert!(verdict.passed(), "the VRAM probe refused; no timing is reported for a path not taken");

    let mut op = provider.build(&space, &ci).expect("could not build the device operator");
    let mut gpu_sigma = vec![0.0f64; space.n_det];
    op.apply(&c, &mut gpu_sigma);

    // ---- AGREEMENT FIRST. A fast wrong answer is not a result.
    let scale = cpu_sigma.iter().fold(0.0f64, |m, x| m.max(x.abs()));
    let max_abs = cpu_sigma
        .iter()
        .zip(gpu_sigma.iter())
        .fold(0.0f64, |m, (a, b)| m.max((a - b).abs()));
    let bitdiff = cpu_sigma
        .iter()
        .zip(gpu_sigma.iter())
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count();
    println!("\n--- agreement with the CPU's own sigma ---");
    println!("max |d|                   {max_abs:.6e}");
    println!("relative to |sigma| max   {:.3e}", max_abs / scale);
    println!(
        "entries differing BITWISE {bitdiff} of {} ({:.1}%)  <- this is D0's whole content",
        space.n_det,
        100.0 * bitdiff as f64 / space.n_det as f64
    );
    assert!(
        max_abs / scale < 1e-12,
        "REFUSED: the device does not reproduce the CPU sigma, so no timing is reported"
    );

    // ---- THE DETERMINISM GATE, per class, five runs, bitwise.
    bit_identity_over_runs(&mut op, &c, 5).expect("the GPU arm failed its bit-identity gate");
    {
        let mut host = CpuProvider.op_for(&space, &ci).unwrap();
        bit_identity_over_runs(host.as_mut(), &c, 5).expect("the CPU arm failed its gate");
    }
    println!("\n--- the determinism gate, 5 runs, PER CLASS ---");
    println!("cpu  bit-identical over 5 runs: YES");
    println!("gpu  bit-identical over 5 runs: YES  (atomics-free; cuBLAS pinned to pedantic");
    println!("                                      math with a fixed 4 MiB workspace)");

    // ---- timing.
    //
    // WARM FIRST, and this is not a courtesy. Measured on this card the first timed loop reads
    // **52.6 sigma/s** and the loops after it read **69.5** — a 32% cold-start gap, from clock
    // and power state rather than from anything in the kernel. Registering one and spot-checking
    // the other CONVICTS AN HONEST ENTRY, which is the registry's own version of a false reap:
    // the first version of this benchmark did exactly that and the plant's control caught it.
    //
    // So every reported number below is warm, the discard is declared, and the cold reading is
    // printed rather than thrown away — a registration is a memory of a measurement, and this is
    // what that memory has to be a memory OF.
    op.preload(&c).expect("preload");
    const WARMUP_LOOPS: usize = 3;
    let gpu_cold = op.time_kernel_only(reps).expect("cold timing");
    for _ in 0..WARMUP_LOOPS {
        op.time_kernel_only(reps).expect("warmup");
    }
    let gpu_kernel = op.time_kernel_only(reps).expect("kernel timing");
    println!(
        "\n(cold first loop {:.1} sigma/s; after {WARMUP_LOOPS} warm-up loops {:.1} sigma/s \
         — {:.0}% cold-start gap, which is why every row below is warm)",
        1.0 / gpu_cold,
        1.0 / gpu_kernel,
        100.0 * (gpu_kernel.recip() - gpu_cold.recip()) / gpu_cold.recip()
    );
    let w0 = Instant::now();
    for _ in 0..reps {
        op.apply(&c, &mut gpu_sigma);
    }
    let gpu_roundtrip = w0.elapsed().as_secs_f64() / reps as f64;

    let n = space.n_orb;
    let n2 = (n * n) as f64;
    let na = space.alpha.len() as f64;
    let nb = space.beta.len() as f64;
    let ns_a = space.alpha.singles[0].len() as f64;
    let gflop = 2.0 * ns_a * n2 * nb * na / 1e9 + 2.0 * na * nb * nb / 1e9 + 2.0 * na * na * nb / 1e9;

    println!("\n--- sigma/s, every row declaring BOTH classes ---");
    println!("| arm | device class | core class | sigma/s | GFLOP/s FP64 |");
    println!("|---|---|---|---:|---:|");
    println!(
        "| GPU, kernel only | {} | n/a | {:.1} | {:.1} |",
        DeviceClass::Gpu,
        1.0 / gpu_kernel,
        gflop / gpu_kernel
    );
    println!(
        "| GPU, incl. host round trip | {} | n/a | {:.1} | — |",
        DeviceClass::Gpu,
        1.0 / gpu_roundtrip
    );
    println!(
        "| CPU, sigma_direct, 1 thread PINNED (wall) | {} | {real_class} (cpus {aff}) | {:.2} | {:.1} |",
        DeviceClass::Cpu,
        1.0 / cpu_wall,
        gflop / cpu_wall
    );
    println!(
        "| CPU, sigma_direct, 1 thread PINNED (CPU time) | {} | {real_class} (cpus {aff}) | {:.2} | {:.1} |",
        DeviceClass::Cpu,
        1.0 / cpu_cpu,
        gflop / cpu_cpu
    );
    println!(
        "\nflops per sigma {gflop:.3} GFLOP; wall/CPU-time ratio on the CPU arm {:.3} \
         (>1 means the arm was descheduled and the wall number is the machine, not the code)",
        cpu_wall / cpu_cpu
    );

    // ---------------- the dispatch registry: the GPU sigma as a citizen ----------------
    //
    // D11 says dispatch CONSULTS a registered measurement. D12 says the registry must not be
    // trusted about itself. Both are exercised here against the rate MEASURED IN THIS PROCESS,
    // not a number carried forward from a previous session — a registration is a memory of a
    // measurement, and calibrations are rented.
    let gpu_rate = 1.0 / gpu_kernel;
    let cpu_rate = 1.0 / cpu_cpu;
    let mut spread_runs = Vec::new();
    for _ in 0..5 {
        spread_runs.push(1.0 / op.time_kernel_only(reps.max(5)).expect("spread timing"));
    }
    let mean = spread_runs.iter().sum::<f64>() / spread_runs.len() as f64;
    let spread = (spread_runs.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
        / (spread_runs.len() - 1) as f64)
        .sqrt();
    println!(
        "\n--- registration (mean AND spread, per RESOURCE_DESIGN section 9 Q4) ---\n\
         gpu sigma/s over {} timing runs: mean {mean:.2}, sd {spread:.3}",
        spread_runs.len()
    );

    let key = WorkloadKey {
        workload: "fci_sigma",
        size: space.n_det as u64,
        device: DeviceClass::Gpu,
    };
    let entry = Entry {
        key,
        mean,
        spread: spread.max(1e-9),
        k: 3.0,
        instrument: "holon-gpu/examples/fci_bench.rs, measured in-process",
        determinism: Determinism::FixedOrder { runs_checked: 5 },
    };
    let mut registry = Registry::new();
    registry.register(entry);
    registry.register(Entry {
        key: WorkloadKey {
            workload: "fci_sigma",
            size: space.n_det as u64,
            device: DeviceClass::Cpu,
        },
        mean: cpu_rate,
        spread: (cpu_rate * 0.1).max(1e-9),
        k: 3.0,
        instrument: "holon-gpu/examples/fci_bench.rs, sigma_direct pinned, CPU-time",
        determinism: Determinism::FixedOrder { runs_checked: 5 },
    });

    // THE CARRIER for the mis-registration plant, asserted before it is scored: the two devices'
    // rates must differ by more than the spot-check tolerance, or a lie could not be told apart
    // from the honest other device and the plant sits in an empty sector (M-PLANT-SECTOR).
    let tolerance = entry.k * entry.spread;
    assert!(
        (gpu_rate - cpu_rate).abs() > tolerance,
        "PLANT VOID (empty sector): the two devices are within {tolerance:.3} sigma/s of each \
         other, so the 10x plant would prove nothing"
    );

    // THE CONTROL: the honest entry, RE-TIMED, is not convicted.
    //
    // D12 says the spot-check RE-TIMES the workload — it does not compare the registration to
    // some earlier number that happens to be lying around. Using a stale reading here is what
    // convicted an honest entry on the first run of this benchmark, and the difference between
    // the two is the whole point: a registration is a memory, and a spot-check is a fresh
    // measurement to check the memory against.
    let observed = 1.0 / op.time_kernel_only(reps).expect("spot-check timing");
    let honest = registry.spot_check(&key, observed).expect("no entry");
    assert!(
        matches!(honest, SpotCheck::Consistent { .. }),
        "the honestly-registered GPU entry was convicted by a fresh re-timing: {honest:?}. \
         Either the kernel's spread is wider than the registration measured, or something \
         between registration and spot-check changed the machine."
    );

    // THE PLANT: register the LIVE measured rate at 10x and re-time. The registry must convict
    // its own entry.
    let mut lying = Registry::new();
    let mut liar = entry;
    liar.mean = mean * 10.0;
    lying.register(liar);
    let verdict = lying.spot_check(&key, observed).expect("no entry");
    match verdict {
        SpotCheck::Convicted { observed, mean: m, tolerance } => println!(
            "PLANT D12 FIRED: registered {m:.1} sigma/s, observed {observed:.1}, tolerance \
             {tolerance:.3} -> the ENTRY is convicted (not the run)"
        ),
        other => panic!(
            "a 10x mis-registration of the LIVE measurement survived the spot-check: {other:?}"
        ),
    }
    println!("PLANT CONTROL held: the honest entry at its own rate is {honest:?}");
}
