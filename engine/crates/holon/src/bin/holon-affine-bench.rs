//! THE MAGIC-TIER PER-BRANCH INSTRUMENT.
//!
//! The QASM magic tier's cost is dominated by the per-branch affine evolution
//! (`affine.rs`), not by the branch enumeration around it. This binary measures
//! that cost on rig-style circuits — the same two families `conformance/qasm/
//! battlerig.py` runs the magic lane on — and prints the timing next to an exact
//! digest of the answer, so a speedup claim and a bit-identity claim come out of
//! the same run.
//!
//! Two families:
//!   `rand n t`  — random Clifford body of depth 4n with exactly t T-gates
//!                 spliced in (battlerig's `gen_fixed_t`).
//!   `hs n c`    — Maiorana–McFarland hidden shift with c cubic monomials,
//!                 t = 14c (battlerig's `hidden_shift`). `hs 1000 2` is the
//!                 largest structured point here: n = 1000, t = 28. Its
//!                 working set peaks at 128 branches, so it is NOT the
//!                 16384-branch run — that one is out of reach for the
//!                 unoptimized build inside a session, and a ratio is only
//!                 worth quoting where BOTH sides were measured.
//!
//! `prune` is the number the engine work lives in (the branch evolution and
//! the merge); `fold` is the amplitude read afterwards, run at ONE shard so a
//! busy host's scheduler is not part of the reading. `per_branch` divides
//! `prune` by the SURVIVING branch count, which is a per-answer figure and not
//! a per-unit-of-work one — `peak` is disclosed next to it because the work is
//! done on the working set, not on the survivors.
//!
//! The printed `digest` is the bit-identity witness: FNV-1a over the WHOLE
//! pruned sum — every branch's exact weight and canonical key, in order — plus
//! the folded amplitude in its exact ring coefficients. Nothing in it is a
//! float and nothing is a tolerance, so two builds agree on that string iff
//! they agree bit for bit on the object.
//!
//! Usage:
//!   holon-affine-bench                 # the default point set
//!   holon-affine-bench rand 256 12     # one point
//!   holon-affine-bench hs 1000 2 --reps 3
//!
//! The host this runs on is shared, so a single reading is worth little:
//! compare two builds INTERLEAVED and read the minimum over rounds.

use holon::ledger::Cyc;
use holon::mesh;
use holon::prune::{run_pruned, Gate, PruneConfig};

// ------------------------------------------------------------------ rng

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

// ------------------------------------------------------------------ circuits

/// Random Clifford body of depth `cdepth`, then exactly `t` T-gates spliced in
/// at random positions. `battlerig.py::gen_fixed_t`, in Rust.
fn gen_fixed_t(n: usize, t: usize, cdepth: usize, seed: u64) -> Vec<Gate> {
    let mut rng = Rng(seed);
    let mut ops: Vec<Gate> = Vec::with_capacity(cdepth + t);
    for _ in 0..cdepth {
        let q = rng.below(n);
        let mut q2 = rng.below(n);
        while q2 == q {
            q2 = rng.below(n);
        }
        ops.push(match rng.below(6) {
            0 => Gate::X(q),
            1 => Gate::Z(q),
            2 => Gate::H(q),
            3 => Gate::S(q),
            4 => Gate::Sdg(q),
            _ => Gate::Cx(q, q2),
        });
    }
    let mut spots: Vec<usize> = (0..t).map(|_| rng.below(ops.len() + 1)).collect();
    spots.sort_unstable();
    for (i, pos) in spots.into_iter().enumerate() {
        let q = rng.below(n);
        ops.insert(pos + i, Gate::T(q));
    }
    ops
}

fn cz(out: &mut Vec<Gate>, a: usize, b: usize) {
    out.push(Gate::H(b));
    out.push(Gate::Cx(a, b));
    out.push(Gate::H(b));
}

/// The textbook 7-T CCZ.
fn ccz(out: &mut Vec<Gate>, a: usize, b: usize, c: usize) {
    out.push(Gate::Cx(b, c));
    out.push(Gate::Tdg(c));
    out.push(Gate::Cx(a, c));
    out.push(Gate::T(c));
    out.push(Gate::Cx(b, c));
    out.push(Gate::Tdg(c));
    out.push(Gate::Cx(a, c));
    out.push(Gate::T(b));
    out.push(Gate::T(c));
    out.push(Gate::Cx(a, b));
    out.push(Gate::T(a));
    out.push(Gate::Tdg(b));
    out.push(Gate::Cx(a, b));
}

/// Maiorana–McFarland hidden shift, `battlerig.py::hidden_shift`. The final X
/// layer sends the deterministic outcome |s⟩ to |0…0⟩, so the all-zeros
/// amplitude is exactly 1 — which is this point's own correctness check.
fn hidden_shift(n: usize, n_cubic: usize, seed: u64) -> Vec<Gate> {
    assert!(n % 2 == 0, "hidden shift needs an even register");
    let m = n / 2;
    let mut rng = Rng(seed);
    let mut s: Vec<bool> = (0..n).map(|_| rng.below(2) == 1).collect();
    if !s.iter().any(|&b| b) {
        s[0] = true;
    }
    let monos: Vec<(usize, usize, usize)> =
        (0..n_cubic).map(|j| (3 * j, 3 * j + 1, 3 * j + 2)).collect();
    assert!(monos.iter().all(|&(_, _, c)| c < m), "cubic monomials must fit the block");

    let mut ops: Vec<Gate> = Vec::new();
    for i in 0..n {
        ops.push(Gate::H(i));
    }
    let xs: Vec<Gate> = (0..n).filter(|&i| s[i]).map(Gate::X).collect();
    ops.extend_from_slice(&xs);
    for i in 0..m {
        cz(&mut ops, i, m + i);
    }
    for &(a, b, c) in &monos {
        ccz(&mut ops, m + a, m + b, m + c);
    }
    ops.extend_from_slice(&xs);
    for i in 0..n {
        ops.push(Gate::H(i));
    }
    for i in 0..m {
        cz(&mut ops, i, m + i);
    }
    for &(a, b, c) in &monos {
        ccz(&mut ops, a, b, c);
    }
    for i in 0..n {
        ops.push(Gate::H(i));
    }
    for i in 0..n {
        if s[i] {
            ops.push(Gate::X(i));
        }
    }
    ops
}

// ------------------------------------------------------------------ harness

/// The bit-identity witness: FNV-1a over the WHOLE pruned sum — every branch's
/// exact weight and its canonical state key, in order — plus the amplitude the
/// fold returns. Nothing here is a float and nothing is a tolerance: two runs
/// agree on this string iff they agree bit for bit on the object.
fn digest(sum: &holon::prune::PrunedSum, amp: Cyc) -> String {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice(&(sum.branches.len() as u64).to_le_bytes());
    for b in &sum.branches {
        for c in b.weight.c {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        bytes.extend_from_slice(&b.weight.m.to_le_bytes());
        bytes.extend_from_slice(&b.state.canon_key());
    }
    for c in amp.c {
        bytes.extend_from_slice(&c.to_le_bytes());
    }
    bytes.extend_from_slice(&amp.m.to_le_bytes());
    format!(
        "{:016x} amp=[{},{},{},{}]/2^({}/2)",
        holon::affine::fnv1a(&bytes),
        amp.c[0],
        amp.c[1],
        amp.c[2],
        amp.c[3],
        amp.m
    )
}

struct Reading {
    prune_s: f64,
    fold_s: f64,
    branches: usize,
    peak: usize,
    t_count: usize,
    digest: String,
}

fn run_point(label: &str, n: usize, gates: &[Gate], reps: usize) {
    let y = vec![false; n];
    let cfg = PruneConfig::default();
    let mut readings: Vec<Reading> = Vec::with_capacity(reps);
    for _ in 0..reps {
        let t0 = std::time::Instant::now();
        let sum = run_pruned(n, gates, &cfg);
        let prune_s = t0.elapsed().as_secs_f64();
        let t1 = std::time::Instant::now();
        // shards = 1: the fold's parallelism is not what this instrument is
        // measuring, and a loaded machine makes it noise.
        let amp = mesh::fold_amplitude(&sum, &y, 1);
        readings.push(Reading {
            prune_s,
            fold_s: t1.elapsed().as_secs_f64(),
            branches: sum.branches.len(),
            peak: sum.stats.peak_working_set,
            t_count: sum.stats.t_count,
            digest: digest(&sum, amp),
        });
    }
    let med = |mut v: Vec<f64>| -> f64 {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    let prune = med(readings.iter().map(|r| r.prune_s).collect());
    let fold = med(readings.iter().map(|r| r.fold_s).collect());
    let br = readings[0].branches;
    assert!(
        readings.iter().all(|r| r.digest == readings[0].digest),
        "{label}: the same circuit gave two different exact amplitudes"
    );
    println!(
        "{label:<13} n={n:<5} t={:<3} gates={:<6} branches={br:<7} peak={:<7} \
prune={prune:8.4}s fold={fold:7.4}s per_branch={:9.4}ms  {}",
        readings[0].t_count,
        gates.len(),
        readings[0].peak,
        1e3 * prune / br.max(1) as f64,
        readings[0].digest,
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut reps = 3usize;
    let mut pos: Vec<String> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--reps" {
            reps = it.next().expect("--reps needs a value").parse().expect("reps");
        } else {
            pos.push(a.clone());
        }
    }
    if pos.len() == 3 {
        let n: usize = pos[1].parse().expect("n");
        let p: usize = pos[2].parse().expect("t or cubic-monomials");
        match pos[0].as_str() {
            "rand" => {
                let g = gen_fixed_t(n, p, 4 * n, 7000 + (n * 100 + p) as u64);
                run_point("rand", n, &g, reps);
            }
            "hs" => {
                let g = hidden_shift(n, p, 5000 + (n + p) as u64);
                run_point("hidden-shift", n, &g, reps);
            }
            other => panic!("unknown family {other}"),
        }
        return;
    }
    // The default set: the two families across the register sizes the rig runs,
    // ending at the largest structured point (n = 1000, t = 28).
    for n in [64usize, 256, 1000] {
        let g = gen_fixed_t(n, 12, 4 * n, 7000 + (n * 100 + 12) as u64);
        run_point("rand", n, &g, reps);
    }
    for n in [64usize, 256, 1000] {
        let g = hidden_shift(n, 1, 5000 + (n + 1) as u64);
        run_point("hidden-shift", n, &g, reps);
    }
    let g = hidden_shift(1000, 2, 5002);
    run_point("hidden-shift", 1000, &g, reps.min(1));
}
