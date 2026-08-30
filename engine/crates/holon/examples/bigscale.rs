//! BIGSCALE — the scaling probe: how far does the Clifford tier actually
//! reach on this box, and what dominates the cost at each n?
//!
//! The bake-off (BENCHMARKS entries six/eight/ten) stops at n = 4096. This
//! probe extends the measurement to the memory ceiling, and — the part the
//! bake-off never touched — separates the ADAPTIVE path's costs from the
//! unitary path's, because mid-circuit measurement is a different algorithm
//! with a different exponent.
//!
//! Discipline: every allocation is guarded against MemAvailable before it is
//! attempted. This box is shared; the probe REFUSES loudly rather than
//! letting the OOM killer pick a victim among the siblings.
//!
//! Run: cargo run --release --example bigscale -- <n> [<n> ...]

use holon::affine::Gate;
use holon::coltableau::ColTableau;
use holon::tableau::PackedTableau;
use std::time::Instant;

/// MemAvailable in bytes, straight from the kernel.
fn mem_available() -> u64 {
    let s = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("MemAvailable:") {
            let kb: u64 = rest
                .trim()
                .trim_end_matches(" kB")
                .trim()
                .parse()
                .unwrap_or(0);
            return kb * 1024;
        }
    }
    0
}

/// Peak RSS this process has ever held (VmHWM), in bytes.
fn peak_rss() -> u64 {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            let kb: u64 = rest
                .trim()
                .trim_end_matches(" kB")
                .trim()
                .parse()
                .unwrap_or(0);
            return kb * 1024;
        }
    }
    0
}

fn current_rss() -> u64 {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            let kb: u64 = rest.trim().trim_end_matches(" kB").trim().parse().unwrap_or(0);
            return kb * 1024;
        }
    }
    0
}

/// A tableau on n qubits is (2n)^2 bits in each of two planes... no: 2n rows,
/// each carrying an n-bit X plane and an n-bit Z plane = 4n^2 bits total.
fn tableau_bytes(n: usize) -> u64 {
    // 2n rows * 2 planes * ceil(n/64) words * 8 bytes
    let words = (n as u64).div_ceil(64);
    2 * (n as u64) * 2 * words * 8
}

/// Refuse rather than OOM a sibling. `headroom` is the multiple of the
/// allocation we insist on leaving free.
fn guard(n: usize, engines: u64, label: &str) -> bool {
    let need = tableau_bytes(n) * engines;
    let avail = mem_available();
    // Keep 2 GB clear for the rest of the box no matter what.
    let reserve = 2u64 << 30;
    if need + reserve > avail {
        println!(
            "  REFUSED {label}: needs {:.2} GB + {:.1} GB reserve, MemAvailable {:.2} GB",
            need as f64 / 1e9,
            reserve as f64 / 1e9,
            avail as f64 / 1e9
        );
        return false;
    }
    true
}

/// A deterministic pseudo-random gate stream — same circuit family the
/// bake-off uses (random Clifford alphabet), sized by n.
fn gate_stream(n: usize, count: usize, seed: u64) -> Vec<Gate> {
    let mut s = seed;
    let mut next = || {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (s >> 33) as usize
    };
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let q = next() % n;
        match next() % 4 {
            0 => out.push(Gate::H(q)),
            1 => out.push(Gate::S(q)),
            2 => out.push(Gate::X(q)),
            _ => {
                let mut t = next() % n;
                if t == q {
                    t = (q + 1) % n;
                }
                out.push(Gate::Cx(q, t));
            }
        }
    }
    out
}

fn apply_col(t: &mut ColTableau, g: Gate) {
    match g {
        Gate::H(q) => t.h(q),
        Gate::S(q) => t.s(q),
        Gate::Sdg(q) => t.sdg(q),
        Gate::X(q) => t.x_gate(q),
        Gate::Z(q) => t.z_gate(q),
        Gate::Cx(a, b) => t.cx(a, b),
        _ => panic!("clifford only"),
    }
}

fn apply_packed(t: &mut PackedTableau, g: Gate) {
    match g {
        Gate::H(q) => t.h(q),
        Gate::S(q) => t.s(q),
        Gate::Sdg(q) => t.sdg(q),
        Gate::X(q) => t.x_gate(q),
        Gate::Z(q) => t.z_gate(q),
        Gate::Cx(a, b) => t.cx(a, b),
        _ => panic!("clifford only"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let sizes: Vec<usize> = if args.is_empty() {
        vec![1024, 4096, 8192, 16384]
    } else {
        args.iter().filter_map(|a| a.parse().ok()).collect()
    };

    println!("BIGSCALE — Clifford tier scaling probe");
    println!("MemAvailable at start: {:.2} GB", mem_available() as f64 / 1e9);
    println!();

    for &n in &sizes {
        println!("n = {n}  (tableau {:.3} GB per engine)", tableau_bytes(n) as f64 / 1e9);

        // ---- COLUMN ENGINE: allocation + unitary gate throughput ----
        if guard(n, 1, "coltableau") {
            let t0 = Instant::now();
            let mut ct = ColTableau::new(n);
            let alloc = t0.elapsed();
            let rss_after_alloc = current_rss();

            // Fixed gate BUDGET (not 20n) so the per-gate number is
            // comparable across n without the wall time exploding.
            let ngates = 20_000usize;
            let gates = gate_stream(n, ngates, 99 + n as u64);
            let t1 = Instant::now();
            for &g in &gates {
                apply_col(&mut ct, g);
            }
            let gate_time = t1.elapsed();

            println!(
                "  col   alloc {:>8.3} s   rss {:>7.3} GB   gate {:>9.3} us/gate  ({} gates in {:.3} s)",
                alloc.as_secs_f64(),
                rss_after_alloc as f64 / 1e9,
                gate_time.as_secs_f64() * 1e6 / ngates as f64,
                ngates,
                gate_time.as_secs_f64()
            );
            drop(ct);
        }

        // ---- PACKED (row) ENGINE: allocation, gates, and the ADAPTIVE path ----
        if guard(n, 1, "packed") {
            let t0 = Instant::now();
            let mut pt = PackedTableau::new(n);
            let alloc = t0.elapsed();
            let rss_after_alloc = current_rss();

            let ngates = 2_000usize;
            let gates = gate_stream(n, ngates, 77 + n as u64);
            let t1 = Instant::now();
            for &g in &gates {
                apply_packed(&mut pt, g);
            }
            let gate_time = t1.elapsed();

            println!(
                "  row   alloc {:>8.3} s   rss {:>7.3} GB   gate {:>9.3} us/gate  ({} gates in {:.3} s)",
                alloc.as_secs_f64(),
                rss_after_alloc as f64 / 1e9,
                gate_time.as_secs_f64() * 1e6 / ngates as f64,
                ngates,
                gate_time.as_secs_f64()
            );

            // ---- THE ADAPTIVE PATH, measured separately ----
            // A LOCAL state, which is what QEC actually produces: reset the
            // tableau and build a short-range entangled state so the
            // measurement costs are the ones a surface code would pay.
            drop(pt);
            let mut pt = PackedTableau::new(n);
            // Nearest-neighbour Bell-ish layer over a BOUNDED prefix: the
            // state must be short-range entangled (that is what QEC
            // produces, and it is what makes the rowsum cascade short), but
            // prepping all n pairs on the row engine costs more than the
            // measurement we are here to time. 2048 pairs is plenty of
            // local structure and the measured qubits all live inside it.
            let prep_pairs = 2048.min((n - 1) / 2);
            for k in 0..prep_pairs {
                let q = 2 * k;
                pt.h(q);
                pt.cx(q, q + 1);
            }
            let span = 2 * prep_pairs;

            // measure_peek on a qubit whose outcome is RANDOM (H'd qubit) —
            // this is the early-exit path: it stops at the first
            // anticommuting stabilizer.
            let reps = 20usize;
            let t2 = Instant::now();
            let mut rand_hits = 0usize;
            for i in 0..reps {
                let q = (i * 101) % span;
                if pt.measure_peek(q).is_none() {
                    rand_hits += 1;
                }
            }
            let peek_time = t2.elapsed();

            // collapse: the O(n) rowsum cascade.
            let mut collapses = 0usize;
            let t3 = Instant::now();
            for i in 0..reps {
                let q = (i * 101) % span;
                if pt.measure_peek(q).is_none() {
                    pt.collapse(q, false);
                    collapses += 1;
                }
            }
            let collapse_time = t3.elapsed();

            // Now every measured qubit is DETERMINISTIC (collapsed to |0>):
            // this is the full-scan path with no early exit.
            let t4 = Instant::now();
            let mut det = 0usize;
            for i in 0..reps {
                let q = (i * 101) % span;
                if pt.measure_peek(q).is_some() {
                    det += 1;
                }
            }
            let det_time = t4.elapsed();

            println!(
                "  adapt peek(rand-exit) {:>9.3} ms   collapse {:>9.3} ms   peek(determ) {:>9.3} ms",
                peek_time.as_secs_f64() * 1e3 / reps as f64,
                if collapses > 0 {
                    collapse_time.as_secs_f64() * 1e3 / collapses as f64
                } else {
                    f64::NAN
                },
                det_time.as_secs_f64() * 1e3 / reps as f64,
            );
            println!(
                "        (rand {rand_hits}/{reps}, collapsed {collapses}, deterministic {det}/{reps})"
            );
            drop(pt);
        }

        println!("  peak RSS so far {:.3} GB", peak_rss() as f64 / 1e9);
        println!();
    }

    println!("done. peak RSS {:.3} GB", peak_rss() as f64 / 1e9);
}
