//! SATURATION-3 G2, the CPU arm: what the dominant kernel actually costs on the real
//! table path.
//!
//! G2 asks whether the sparse Hamiltonian matvec at the `(O,O,O)` scale wins on the
//! 4090. That question has no meaning without the CPU number it would be beating, and
//! the CPU number has to come from the SAME assembly the tables use — `geometry_problem`
//! into `sigma_direct` — rather than from a synthetic sparse matrix with a plausible
//! sparsity pattern. M-FOREIGN-DOMAIN-CORROBORATION: the gate exercises the actual path.
//!
//! What it reports, and why each number is here:
//!
//! * `n_det`, against the prereg's COMMITTED arithmetic (207,025 for `(O,O,O)`). The
//!   prereg staked the hole-count arithmetic before any solve, so this is a test of the
//!   arithmetic and not a memory of it.
//! * the per-call wall time of `sigma_direct`, which is what a GPU kernel would replace.
//! * the Davidson iteration count for a cold solve, so the per-solve kernel budget is
//!   `iters x sigma` rather than a guess — the quantity that decides whether a GPU
//!   kernel could matter at all.
//! * the vector footprint, because a kernel whose operands fit in cache is a different
//!   engineering problem from one whose operands do not, and PCIe transfer is priced
//!   against exactly this.
//!
//! M-VACUOUS-SUCCESS: every timed section asserts its work count, so a section that
//! silently did nothing cannot report a fast time.

use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::{ci_ints, sigma_direct, Order};
use holon_chem::sigma_op::SigmaOp;
use holon_chem::pair::geometry_problem;
use std::time::Instant;

fn at(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // The species triple, by SYMBOL on the command line and by Z into every formula
    // (M-TAG-AS-PROPERTY: the name is an argument, never a branch).
    let symbols: Vec<String> = if args.is_empty() {
        vec!["O".into(), "O".into(), "O".into()]
    } else {
        args[0].split(',').map(|s| s.to_string()).collect()
    };
    assert_eq!(symbols.len(), 3, "this probe prices TRIPLES; give three symbols");

    let species: Vec<_> = symbols
        .iter()
        .map(|s| by_symbol(s).unwrap_or_else(|| panic!("unknown species symbol {s}")))
        .collect();

    // A compact geometry: the cost of the kernel is combinatorial in the space, not in
    // the geometry, but a compact triangle is the honest place to price it because it
    // is where the Davidson has the most work to do and where the staked worst-case
    // geometries live.
    let s = 2.4_f64;
    let centers = vec![
        at(0.0, 0.0, 0.0),
        at(s, 0.0, 0.0),
        at(0.5 * s, 0.75_f64.sqrt() * s, 0.0),
    ];

    let t0 = Instant::now();
    let (space, mo, _nuc) = geometry_problem(&species, centers);
    let assembly = t0.elapsed();

    let n_orb = space.n_orb;
    let n_det = space.n_det;
    println!("species        {}", symbols.join(","));
    println!("n_orb          {n_orb}");
    println!("n_alpha_str    {}", space.alpha().len());
    println!("n_beta_str     {}", space.beta().len());
    println!("n_det          {n_det}");
    println!("assembly       {:.3} s", assembly.as_secs_f64());
    println!(
        "vector_bytes   {} ({:.2} MiB per f64 vector)",
        n_det * 8,
        (n_det * 8) as f64 / (1024.0 * 1024.0)
    );

    let ci0 = ci_ints(&mo, Order::Value);
    let t0 = Instant::now();
    let diag = space.diagonal(&ci0);
    let diag_t = t0.elapsed();
    assert_eq!(diag.len(), n_det, "diagonal must cover the whole space");
    println!("diagonal       {:.3} s", diag_t.as_secs_f64());

    // The kernel itself. A deterministic dense-ish input vector, because a vector with
    // structural zeros would let `sigma_direct`'s `if cv == 0.0 { continue }` skip real
    // work and report a time the production path never sees.
    let mut c = vec![0.0f64; n_det];
    let mut seed = 0x243f_6a88_85a3_08d3u64;
    for x in c.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *x = ((seed >> 33) as f64 / (1u64 << 31) as f64) - 0.5;
    }
    let nonzero = c.iter().filter(|x| **x != 0.0).count();
    assert_eq!(nonzero, n_det, "the probe vector must be structurally dense");

    let reps: usize = std::env::var("S3_SIGMA_REPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    let mut out = vec![0.0f64; n_det];
    // The operator is built ONCE, outside the timing: `sigma_direct` builds the lane tables
    // per call, and a loop over it would time the table build. One untimed call first, so the
    // timed reps measure the kernel and not the first touch of `out`.
    let mut op = holon_chem::lanes::LaneSigma::for_ci(&space, &ci0);
    op.apply(&c, &mut out);
    let warm_checksum = out.iter().fold(0.0f64, |a, b| a + b);
    assert!(
        warm_checksum.is_finite() && warm_checksum != 0.0,
        "sigma produced nothing to time (checksum {warm_checksum})"
    );

    let t0 = Instant::now();
    for _ in 0..reps {
        op.apply(&c, &mut out);
    }
    let per_call = t0.elapsed().as_secs_f64() / reps as f64;
    println!("lane sigma     {per_call:.4} s per call  (mean of {reps}; tables built once, {} host shards)", op.threads);
    println!("loadavg        {:.2}", loadavg());

    // ---- the PARALLEL CPU arm, which is the one G2 has to be compared against.
    //
    // The table path parallelises across NODES on the mesh, not inside one solve. So the
    // quantity a GPU would have to beat is not one core's sigma but the whole machine's
    // sigma THROUGHPUT, and that is not `threads x single-core` — these kernels contend
    // for memory bandwidth and the contention is exactly what a per-core extrapolation
    // would hide. Measured rather than multiplied.
    let thread_counts: Vec<usize> = std::env::var("S3_THREADS")
        .ok()
        .map(|v| v.split(',').filter_map(|t| t.parse().ok()).collect())
        .unwrap_or_else(|| vec![1, 8, 16, 32]);
    println!();
    println!("--- aggregate CPU throughput (independent nodes in parallel) ---");
    for &t in &thread_counts {
        // Each worker gets its OWN vectors, as separate nodes would.
        let t0 = Instant::now();
        let done: usize = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..t)
                .map(|_| {
                    let space = &space;
                    let ci0 = &ci0;
                    let c = &c;
                    scope.spawn(move || {
                        let mut o = vec![0.0f64; space.n_det];
                        for _ in 0..reps {
                            sigma_direct(space, ci0, c, &mut o);
                        }
                        // M-VACUOUS-SUCCESS: the worker asserts it did the work.
                        assert!(o.iter().any(|x| *x != 0.0), "worker produced no sigma");
                        reps
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).sum()
        });
        assert_eq!(done, t * reps, "every worker must have completed its reps");
        let secs = t0.elapsed().as_secs_f64();
        let per = secs / (t * reps) as f64;
        println!(
            "threads {t:3}   {:8.2} sigma/s aggregate   {per:.4} s per call   \
             scaling {:.2}x",
            done as f64 / secs,
            per_call / per
        );
    }
    println!("loadavg        {:.2}  (this machine runs other campaigns)", loadavg());

    // What a solve costs in kernel calls, so the GPU budget is priced against the real
    // per-solve figure. Davidson calls sigma once per new basis vector.
    println!();
    println!("--- per-solve kernel budget ---");
    println!(
        "a cold Davidson at ~{} sigma calls would spend ~{:.1} s in the kernel alone",
        60,
        60.0 * per_call
    );
    println!(
        "(the full solve also pays cg_response for the second derivative; this probe \
         prices the KERNEL, which is the only thing a GPU would replace)"
    );
}

/// `/proc/loadavg`'s first field. Every CPU timing here carries one: this machine runs
/// other campaigns concurrently, and a CPU number without the load it was taken under is
/// an anecdote rather than a measurement.
fn loadavg() -> f64 {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse().ok())
        .unwrap_or(f64::NAN)
}
