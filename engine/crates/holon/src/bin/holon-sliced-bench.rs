//! THE BRANCH-SLICER'S INSTRUMENT: measured per-branch throughput or it didn't
//! happen.
//!
//! One question, asked honestly: how many branches per second does the magic
//! tier evaluate, per-branch versus 64-to-a-word, on the SAME circuit to the
//! SAME exact answer? Everything else here exists to keep that question from
//! being answered by accident.
//!
//! Three arms, all serial unless a row says otherwise, all timed by median of
//! `REPS` runs (a median, because one background compile is enough to ruin a
//! mean and this machine is shared):
//!
//! * `pruned` — `run::amplitude`: the production per-branch path. Prunes,
//!   canonicalizes, fingerprints, dedups, folds through the mesh.
//! * `naive` — `prune::run_naive` + the same fold: the same per-branch
//!   evolution with the DEDUP TURNED OFF. It is here because the sliced path
//!   does not dedup either, so without this arm the headline factor would be
//!   quietly charging the per-branch path for hash work the sliced path never
//!   does. `naive` is the like-for-like baseline; `pruned` is the honest
//!   production comparison. Both are reported.
//! * `sliced` — `sliced::build` + the same fold through `Blocks`.
//! * `same-alg` — the sliced path with `defer_phase = false`: the SAME phase
//!   schedule the per-branch engine runs, so this column is slicing and
//!   nothing else. The gap between it and `sliced` is what deferring the
//!   powers of `i` into a lane plane buys — a deferral the per-branch engine
//!   could also adopt, which is why it is reported separately instead of
//!   being folded into the headline.
//!
//! # Where the 64× goes, measured rather than guessed
//!
//! A block's cost splits into work SHARED by the 64 branches (all the F₂
//! linear algebra: reduced column echelon form, the dependence repair, the
//! amplitude solve) and work that is genuinely PER-LANE (exact `Z[ω]`
//! multiplies on `γ` and on the branch weight — 128-bit integer coefficients
//! are not bits and do not slice). Writing `S` for the shared part of one
//! pass and `L` for one lane's ring part:
//!
//! ```text
//!   per-branch cost  ≈ S + L          (a scalar branch pays both, alone)
//!   per-block cost   ≈ S + 64·L       (64 lanes share S, pay L each)
//!   ideal factor     = 64             (only if L = 0)
//!   actual factor    = 64·(S + L) / (S + 64·L)
//! ```
//!
//! Both sides of that are measured directly — `S + L` is the `naive` arm's
//! time per branch, `S + 64·L` is the sliced arm's time per block — so the
//! ring fraction `L/(S+L)` is solved for and printed. It is the ceiling, and
//! it is the honest reason the headline is not 64.
//!
//! Run: `cargo run --release -p holon --bin holon-sliced-bench`.

use holon::mesh;
use holon::merge;
use holon::prune::{self, Gate, PruneConfig};
use holon::run;
use holon::sliced::{self, Blocks, SlicedConfig, LANES};
use std::time::{Duration, Instant};

const REPS: usize = 9;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// A live random Clifford+T circuit with H in the alphabet — the gate that
/// grows the column set and reaches the expensive paths. A fixture without H
/// would measure the easy half and report it as the whole.
fn random_circuit(seed: u64, n: usize, depth: usize, t: usize) -> Vec<Gate> {
    let mut rng = Rng(seed);
    let mut body: Vec<Gate> = Vec::with_capacity(depth + t);
    for _ in 0..depth {
        let q = rng.below(n);
        body.push(match rng.below(6) {
            0 => Gate::H(q),
            1 => Gate::S(q),
            2 => Gate::Sdg(q),
            3 => Gate::X(q),
            4 => Gate::Z(q),
            _ => {
                let mut c = rng.below(n);
                if c == q {
                    c = (q + 1) % n;
                }
                Gate::Cx(c, q)
            }
        });
    }
    for _ in 0..t {
        let pos = rng.below(body.len() + 1);
        let q = rng.below(n);
        body.insert(pos, if rng.below(2) == 0 { Gate::T(q) } else { Gate::Tdg(q) });
    }
    body
}

fn median(mut xs: Vec<Duration>) -> Duration {
    xs.sort();
    xs[xs.len() / 2]
}

fn time<T>(f: impl Fn() -> T) -> (Duration, T) {
    // One untimed warm pass, then REPS timed ones; the median is reported.
    let mut last = f();
    let mut runs = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        let t0 = Instant::now();
        last = f();
        runs.push(t0.elapsed());
    }
    (median(runs), last)
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "unavailable".into())
}

fn main() {
    println!("# branch slicing — per-branch-equivalent throughput");
    println!("# lanes per word: {LANES}");
    println!("# loadavg at start: {}", loadavg());
    println!("# medians of {REPS} runs, serial (shards = 1) unless stated");
    println!();
    println!(
        "{:>3} {:>5} {:>3} {:>8} {:>11} {:>11} {:>11} {:>11} {:>8} {:>8} {:>9} {:>8}",
        "n", "depth", "t", "branch", "pruned ns", "naive ns", "sameAlg ns", "sliced ns",
        "×pruned", "×naive", "×sameAlg", "ringfrac"
    );

    let cases: [(u64, usize, usize, usize); 9] = [
        // (seed, n, clifford depth, T-count)
        (1, 8, 120, 8),
        (2, 8, 120, 10),
        (3, 8, 120, 12),
        (4, 16, 200, 10),
        (5, 16, 200, 12),
        (6, 24, 300, 12),
        (7, 32, 400, 12),
        (8, 48, 500, 12),
        (9, 64, 600, 12),
    ];

    let mut factors_naive = Vec::new();
    let mut factors_pruned = Vec::new();
    let mut factors_same = Vec::new();

    for (seed, n, depth, t) in cases {
        let gates = random_circuit(seed, n, depth, t);
        let y: Vec<bool> = (0..n).map(|q| q % 3 == 0).collect();
        let branches = 1u64 << t;

        let (d_pruned, a_pruned) = time(|| run::amplitude_sharded(n, &gates, &y, 1));
        let naive_cfg = PruneConfig { disable_merge: true, ..PruneConfig::default() };
        let (d_naive, a_naive) = time(|| {
            let sum = prune::run_pruned(n, &gates, &naive_cfg);
            mesh::fold_amplitude(&sum, &y, 1)
        });
        let sliced_cfg = SlicedConfig::default();
        let (d_sliced, a_sliced) = time(|| {
            let sum = sliced::build(n, &gates, &sliced_cfg);
            mesh::fold_amplitude(&Blocks(&sum), &y, 1)
        });
        let same_cfg = SlicedConfig { defer_phase: false, ..SlicedConfig::default() };
        let (d_same, a_same) = time(|| {
            let sum = sliced::build(n, &gates, &same_cfg);
            mesh::fold_amplitude(&Blocks(&sum), &y, 1)
        });
        assert_eq!(a_same, a_sliced, "n={n} t={t}: the two phase schedules disagree");

        // The measurement is worth nothing if the arms disagree.
        assert_eq!(a_pruned, a_naive, "n={n} t={t}: pruned and naive disagree");
        assert!(
            holon::affine::cyc_eq(a_sliced, a_pruned),
            "n={n} t={t}: sliced and pruned disagree — the bench is measuring two different answers"
        );

        let ns = |d: Duration| d.as_secs_f64() * 1e9;
        let per_pruned = ns(d_pruned) / branches as f64;
        let per_naive = ns(d_naive) / branches as f64;
        let per_sliced = ns(d_sliced) / branches as f64;
        let per_same = ns(d_same) / branches as f64;
        let f_pruned = per_pruned / per_sliced;
        let f_naive = per_naive / per_sliced;
        let f_same = per_naive / per_same;

        // S + L = naive per branch; S + 64L = the SAME-ALGORITHM sliced cost
        // per BLOCK. Solve for the ring fraction L/(S+L) — the part of a
        // branch's work that cannot be word-parallel. The deferred column is
        // excluded from this solve on purpose: it changes L, so putting it
        // here would be measuring two things and reporting one.
        let sl = per_naive; // S + L
        let s64l = per_same * LANES as f64; // S + 64L
        let ring_frac = ((s64l - sl) / (LANES as f64 - 1.0)) / sl;

        println!(
            "{n:>3} {depth:>5} {t:>3} {branches:>8} {per_pruned:>11.1} {per_naive:>11.1} \
             {per_same:>11.1} {per_sliced:>11.1} {f_pruned:>7.1}× {f_naive:>7.1}× \
             {f_same:>8.1}× {:>7.1}%",
            ring_frac * 100.0
        );
        factors_naive.push(f_naive);
        factors_pruned.push(f_pruned);
        factors_same.push(f_same);
    }

    let med = |mut v: Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    println!();
    println!(
        "median per-branch-equivalent factor: {:.1}× vs the like-for-like per-branch path \
         ({:.1}× of it from slicing alone, on one algorithm), {:.1}× vs the production \
         pruned path (which reduces the branch COUNT and so is not a like-for-like \
         comparison at all)",
        med(factors_naive.clone()),
        med(factors_same.clone()),
        med(factors_pruned.clone())
    );

    // ---------------------------------------------------------- the two tiers
    //
    // The build and the fold are separately timed, because they lose the 64×
    // for different reasons: the build carries the per-lane ring multiplies,
    // the amplitude query is very nearly pure word-parallelism (one F₂
    // elimination for 64 right-hand sides).
    println!();
    println!("# the two tiers, separated (n = 16, t = 12, {} branches)", 1 << 12);
    let gates = random_circuit(5, 16, 200, 12);
    let n = 16;
    let y: Vec<bool> = (0..n).map(|q| q % 3 == 0).collect();
    let branches = 1u64 << 12;

    let naive_cfg = PruneConfig { disable_merge: true, ..PruneConfig::default() };
    let (d_build_scalar, sum_scalar) = time(|| prune::run_pruned(n, &gates, &naive_cfg));
    let (d_query_scalar, _) = time(|| mesh::fold_amplitude(&sum_scalar, &y, 1));
    let sliced_cfg = SlicedConfig::default();
    let (d_build_sliced, sum_sliced) = time(|| sliced::build(n, &gates, &sliced_cfg));
    let (d_query_sliced, _) = time(|| mesh::fold_amplitude(&Blocks(&sum_sliced), &y, 1));

    let ns = |d: Duration| d.as_secs_f64() * 1e9 / branches as f64;
    println!(
        "  evolution (branch → state): {:.1} ns/br per-branch, {:.1} ns/br sliced — {:.1}×",
        ns(d_build_scalar),
        ns(d_build_sliced),
        ns(d_build_scalar) / ns(d_build_sliced)
    );
    println!(
        "  amplitude query (one y):    {:.1} ns/br per-branch, {:.1} ns/br sliced — {:.1}×",
        ns(d_query_scalar),
        ns(d_query_sliced),
        ns(d_query_scalar) / ns(d_query_sliced)
    );
    println!(
        "  states held:                {} branch states vs {} blocks of {LANES}",
        sum_scalar.branches.len(),
        sum_sliced.blocks.len()
    );

    // -------------------------------------------------------------- amortised
    //
    // A state vector asks 2^n amplitudes of one evolved circuit, so it charges
    // the query and not the build. This is the regime the sliced query was
    // built for, and it is reported separately rather than folded into the
    // headline.
    println!();
    println!("# amortised query: 256 basis states of one evolved circuit (n = 16, t = 12)");
    let ys: Vec<Vec<bool>> = (0..256)
        .map(|i: usize| (0..n).map(|q| (i >> (q % 8)) & 1 == 1).collect())
        .collect();
    let (d_many_scalar, _) = time(|| {
        merge::fold(ys.iter().map(|y| mesh::fold_amplitude(&sum_scalar, y, 1)))
    });
    let (d_many_sliced, _) = time(|| {
        merge::fold(ys.iter().map(|y| mesh::fold_amplitude(&Blocks(&sum_sliced), y, 1)))
    });
    let per = |d: Duration| d.as_secs_f64() * 1e9 / (branches as f64 * 256.0);
    println!(
        "  {:.1} ns/br per-branch, {:.1} ns/br sliced — {:.1}×",
        per(d_many_scalar),
        per(d_many_sliced),
        per(d_many_scalar) / per(d_many_sliced)
    );

    // ------------------------------------------------------------- threaded
    println!();
    let shards = std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1);
    println!("# both tiers sharded across {shards} threads (n = 16, t = 12)");
    let (d_par, a_par) = time(|| sliced::amplitude(n, &gates, &y, shards));
    let (d_ser, a_ser) = time(|| sliced::amplitude(n, &gates, &y, 1));
    assert_eq!(a_par, a_ser, "sharding changed the ledger entry");
    println!(
        "  serial {:.1} ns/br, sharded {:.1} ns/br — {:.1}× on top of the slicing",
        d_ser.as_secs_f64() * 1e9 / branches as f64,
        d_par.as_secs_f64() * 1e9 / branches as f64,
        d_ser.as_secs_f64() / d_par.as_secs_f64()
    );
    println!();
    println!("# loadavg at end: {}", loadavg());
}
