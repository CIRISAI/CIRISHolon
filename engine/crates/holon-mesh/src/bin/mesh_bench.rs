//! Scaling measurement: one scene, 1/2/4/8/16 threads, with the serial fraction reported
//! honestly rather than inferred from the speedup that flatters it most.
//!
//! Two numbers are reported per thread count, and the second is the one that matters:
//!
//! * **speedup** `S(p) = T(1)/T(p)` — what a scaling plot shows;
//! * **Karp–Flatt serial fraction** `f(p) = (1/S − 1/p) / (1 − 1/p)` — the experimentally
//!   determined serial fraction. It is reported because it distinguishes the two reasons
//!   scaling stalls: if `f` is roughly CONSTANT across `p`, the limit is genuine serial work
//!   and Amdahl's law applies; if `f` RISES with `p`, the limit is parallel overhead —
//!   barriers, in this design — and the fix is a different synchronisation structure, not a
//!   smaller serial section. `SANDBOX_4090` §6 already measured that the barrier, not the
//!   exchange, is what limits the CPU prototype (≥90% efficiency to 8 threads, 62% at 16),
//!   so a rising `f` is the predicted shape and this is the instrument that can confirm or
//!   refute it.
//!
//! **Wall-clock caveat, and it is not a formality.** This machine is shared. If the load
//! average is high the speedups are not defensible as hardware measurements, because the run
//! is not being given the cores it asks for. The load is printed with the results so a reader
//! can discount them, exactly as the prototype's own bench does.

use std::time::Instant;

use holon_mesh::{Grid, Mesh, MeshSpec};

/// Trials per configuration. Odd, so the median is an observed value rather than an average
/// of two.
const REPEATS: usize = 7;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let side = args
        .get(1)
        .and_then(|a| a.parse::<usize>().ok())
        .unwrap_or(256);
    let steps = args
        .get(2)
        .and_then(|a| a.parse::<usize>().ok())
        .unwrap_or(40);
    // Colour sweeps between halo refreshes. This is the `n` of Locality.lean's `n*r`, and it
    // is the scaling knob: raising it amortises the exchange's barriers over more work, and
    // the price is a halo `n` cells deep.
    let n = args
        .get(3)
        .and_then(|a| a.parse::<usize>().ok())
        .unwrap_or(1);

    let grid = Grid::new(side, side);
    let cells = grid.len();

    println!("holon-mesh scaling — ONE scene sharded across cores");
    println!(
        "scene: {side}x{side} = {cells} cells, {steps} colour sweeps, n={n} (halo depth {n})"
    );
    if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
        println!("host load average: {}", loadavg.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
    }
    println!(
        "\n{:>7} {:>7} {:>11} {:>7} {:>16} {:>9} {:>9} {:>11}",
        "threads", "shards", "median ms", "spread", "holon-steps/s", "speedup", "eff", "Karp-Flatt"
    );

    let mut baseline: Option<f64> = None;
    for threads in [1usize, 2, 4, 8, 16] {
        // More shards than threads, so the schedule has something to balance. Cuts stay
        // powers of two, which is what keeps a shard a whole sub-block of the tree.
        let (nx, ny) = shard_cuts(threads);
        let spec = MeshSpec::new(grid, nx, ny).with_colours_per_exchange(n);
        let shards = nx * ny;

        // Warm the allocator and the page cache so the first row is not measuring setup.
        {
            let mut warm = Mesh::new(spec.clone()).expect("mesh built");
            warm.run_threaded(2, threads).expect("warmup ran");
        }

        // MEDIAN of repeats, not a single trial. A first pass at this benchmark reported
        // single trials and its single-thread baseline moved 70% between runs on this shared
        // host — which makes every speedup derived from it meaningless. The median is the
        // cheapest defence; it is not a substitute for a quiet machine.
        let mut trials = Vec::with_capacity(REPEATS);
        for _ in 0..REPEATS {
            let mut mesh = Mesh::new(spec.clone()).expect("mesh built");
            let opening = mesh.opening_total();
            let start = Instant::now();
            mesh.run_threaded(steps, threads).expect("mesh ran");
            trials.push(start.elapsed().as_secs_f64());
            // The scaling number is worthless if the run was wrong, so the conservation gate
            // runs inside the benchmark rather than beside it.
            assert_eq!(
                mesh.total().expect("total"),
                opening,
                "conservation broke during the benchmark; the timing means nothing"
            );
        }
        trials.sort_by(f64::total_cmp);
        let elapsed = trials[REPEATS / 2];
        let spread = (trials[REPEATS - 1] - trials[0]) / elapsed;

        let holon_steps = (cells * steps) as f64;
        let rate = holon_steps / elapsed;
        let (speedup, eff, karp) = match baseline {
            None => {
                baseline = Some(elapsed);
                (1.0, 1.0, f64::NAN)
            }
            Some(t1) => {
                let s = t1 / elapsed;
                let p = threads as f64;
                (s, s / p, (1.0 / s - 1.0 / p) / (1.0 - 1.0 / p))
            }
        };
        println!(
            "{threads:>7} {shards:>7} {:>11.2} {:>6.0}% {:>16.3e} {speedup:>9.2} {eff:>9.2} {:>11}",
            elapsed * 1e3,
            spread * 100.0,
            rate,
            if karp.is_nan() {
                "—".to_string()
            } else {
                format!("{karp:.3}")
            }
        );
    }

    println!(
        "\nRead the Karp-Flatt column, not the speedup: constant => genuine serial work,\n\
         rising => parallel overhead (the barrier). Discount everything if the load average\n\
         above is not near zero."
    );
}

/// Shard cuts for a thread count: at least four shards per thread so a claim-based balancer
/// has something to move, and both axes cut so shards are blocks rather than slabs.
fn shard_cuts(threads: usize) -> (usize, usize) {
    match threads {
        1 => (2, 2),
        2 => (4, 2),
        4 => (4, 4),
        8 => (8, 4),
        _ => (8, 8),
    }
}
