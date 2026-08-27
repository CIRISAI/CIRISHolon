//! The MESH's instrument: measured speedup or it didn't happen.
//!
//! Scaling of `holon::mesh::fold_amplitude` on a magic-tier branch sum with
//! n = 12 qubits and t = 14 T-gates — 16384 branches, each one a REAL affine
//! evolution (Dehaene–De Moor / Van den Nest form, the same update rules the
//! certified `holon-qasm` magic tier uses) followed by a real exact-amplitude
//! query (F₂ Gaussian solve + quadratic-form phase + `Z[ω]` ring multiplies).
//!
//! **Why the source is self-contained rather than the reference engine.**
//! `holon-qasm` is a DEV-dependency of this crate: tests and benches under
//! `tests/` may use it, a `src/bin` target may not. So this file carries its
//! own affine simulator. It is not a mock and it does not fake the work: the
//! X / Z / S / CX updates and the amplitude query are transcribed from
//! `holon_qasm::magic::Affine`, restricted to the circuit family
//! `H^⊗n · (Clifford+T with no further H)`. That restriction is what buys the
//! brevity — with no H after the opening layer the column count `k` never
//! changes, so the reference's `fold` / `dependent_subset` / `gauss_sum_out`
//! machinery is never reachable and is therefore not transcribed. (What IS
//! shared is shared: `i_pow` and the splitmix stream come from
//! `holon::affine`, a normal lib module this target may use — only the
//! bit-packed state, which is the thing being benchmarked, is local.) The opening
//! `H^⊗n` layer is branch-INDEPENDENT, so it is materialised once
//! (`Affine::plus_state`, which is exactly what n `h_gate` calls produce from
//! a fresh state: R = I, h = 0, d = 0, J = 0, γ = 2^{−n/2}); everything after
//! it is replayed per branch.
//!
//! It is not asserted to be faithful, either: `tests/mesh.rs` includes this
//! file with `#[path]` and checks this exact source against
//! `holon_qasm::magic::magic_amplitude` on the equivalent `Circuit`. The
//! fixture is certified by the same referee as the mesh itself.

use holon::affine::{i_pow, Rng};
use holon::ledger::Cyc;
use holon::mesh;
use holon::BranchSource;

// ------------------------------------------------------------------- affine

/// The affine branch state, bit-packed: `x = R u ⊕ h` with amplitude
/// `γ · i^{Σ d_a u_a} · (−1)^{Σ_{a<b} J_{ab} u_a u_b}`. Rows and J-rows are
/// `u64` masks, so `n, k ≤ 64` — this bench runs at n = k = 12.
#[derive(Clone)]
pub struct Affine {
    n: usize,
    k: usize,
    /// `rows[q]` bit `a` = R[q][a].
    rows: Vec<u64>,
    /// bit `q` = h[q].
    h: u64,
    /// d_a mod 4.
    d: Vec<u8>,
    /// `j[a]` bit `b` = J_{ab}, symmetric, diagonal unused.
    j: Vec<u64>,
    gamma: Cyc,
}

impl Affine {
    /// The state an `H` on every qubit produces from |0…0⟩: R = I, h = 0,
    /// d = 0, J = 0, γ = 1·2^{−n/2}.
    pub fn plus_state(n: usize) -> Self {
        assert!(n <= 64, "bit-packed affine rows cap at 64 qubits");
        Affine {
            n,
            k: n,
            rows: (0..n).map(|q| 1u64 << q).collect(),
            h: 0,
            d: vec![0; n],
            j: vec![0; n],
            gamma: Cyc { c: [1, 0, 0, 0], m: n as i32 },
        }
    }

    fn x(&mut self, q: usize) {
        self.h ^= 1 << q;
    }

    fn z(&mut self, q: usize) {
        if self.h >> q & 1 == 1 {
            self.gamma = self.gamma.mul(i_pow(2));
        }
        let mut s = self.rows[q];
        while s != 0 {
            let a = s.trailing_zeros() as usize;
            s &= s - 1;
            self.d[a] = (self.d[a] + 2) % 4;
        }
    }

    fn s_gate(&mut self, q: usize) {
        let a_set = self.rows[q];
        let hq = self.h >> q & 1 == 1;
        if hq {
            self.gamma = self.gamma.mul(i_pow(1));
        }
        let bump = if hq { 3 } else { 1 };
        let mut s = a_set;
        while s != 0 {
            let a = s.trailing_zeros() as usize;
            s &= s - 1;
            self.d[a] = (self.d[a] + bump) % 4;
            // J_{ab} ^= 1 for every b > a also in the support.
            let higher = a_set & !((1u64 << a) | ((1u64 << a) - 1));
            self.j[a] ^= higher;
            let mut hb = higher;
            while hb != 0 {
                let b = hb.trailing_zeros() as usize;
                hb &= hb - 1;
                self.j[b] ^= 1 << a;
            }
        }
    }

    fn cx(&mut self, c: usize, t: usize) {
        self.rows[t] ^= self.rows[c];
        let hc = self.h >> c & 1;
        self.h ^= hc << t;
    }

    /// Exact amplitude of basis state `y` (bit q = qubit q).
    fn amplitude(&self, y: u64) -> Cyc {
        // Solve R u = y ⊕ h over F₂.
        let target = y ^ self.h;
        let mut mask: Vec<u64> = self.rows.clone();
        let mut rhs: Vec<bool> = (0..self.n).map(|q| target >> q & 1 == 1).collect();
        let mut pivot_row = vec![usize::MAX; self.k];
        let mut rr = 0usize;
        for (col, pivot) in pivot_row.iter_mut().enumerate() {
            let bit = 1u64 << col;
            let Some(p) = (rr..self.n).find(|&p| mask[p] & bit != 0) else {
                continue;
            };
            mask.swap(rr, p);
            rhs.swap(rr, p);
            for p2 in 0..self.n {
                if p2 != rr && mask[p2] & bit != 0 {
                    mask[p2] ^= mask[rr];
                    rhs[p2] ^= rhs[rr];
                }
            }
            *pivot = rr;
            rr += 1;
        }
        if rhs[rr..].iter().any(|&b| b) {
            return Cyc::ZERO; // y is off the affine subspace
        }
        let mut u = 0u64;
        for (col, &pivot) in pivot_row.iter().enumerate() {
            // R is invertible for this circuit family, so every column pivots;
            // a free column would mean the state is a proper subspace and the
            // sum over free variables would be owed. Refuse rather than guess.
            assert!(pivot != usize::MAX, "affine: R has a free column");
            if rhs[pivot] {
                u |= 1 << col;
            }
        }
        let mut ip: u8 = 0;
        let mut sign = false;
        let mut s = u;
        while s != 0 {
            let a = s.trailing_zeros() as usize;
            s &= s - 1;
            ip = (ip + self.d[a]) % 4;
            let higher = u & !((1u64 << a) | ((1u64 << a) - 1));
            sign ^= (self.j[a] & higher).count_ones() & 1 == 1;
        }
        let mut amp = self.gamma.mul(i_pow(ip));
        if sign {
            amp = amp.mul(i_pow(2));
        }
        amp
    }
}

// ------------------------------------------------------------ branch source

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum G {
    X(usize),
    Z(usize),
    S(usize),
    Cx(usize, usize),
    T(usize),
}

/// A fixed `H^⊗n` + Clifford+T circuit, enumerated branch by branch. The
/// gate list is generated once from a seed and then frozen: every branch
/// replays the SAME gates, differing only in which T-gate leg it takes.
pub struct CircuitSource {
    n: usize,
    t_count: usize,
    gates: Vec<G>,
    base: Affine,
}

impl CircuitSource {
    pub fn new(n: usize, t_count: usize, clifford_depth: usize, seed: u64) -> Self {
        assert!(t_count < 63, "branch index is a u64");
        let mut rng = Rng::new(seed);
        let mut gates: Vec<G> = Vec::with_capacity(clifford_depth + t_count);
        // T-gates evenly spaced through the Clifford body, so the branch bits
        // enter early AND late (a T at the very end would leave most of the
        // per-branch work branch-independent and flatter the fold).
        let stride = (clifford_depth / (t_count + 1)).max(1);
        let mut placed = 0usize;
        for i in 0..clifford_depth {
            let q = rng.below(n);
            let mut q2 = rng.below(n);
            while q2 == q {
                q2 = rng.below(n);
            }
            gates.push(match rng.below(4) {
                0 => G::X(q),
                1 => G::Z(q),
                2 => G::S(q),
                _ => G::Cx(q, q2),
            });
            if placed < t_count && i % stride == stride - 1 {
                gates.push(G::T(rng.below(n)));
                placed += 1;
            }
        }
        while placed < t_count {
            gates.push(G::T(rng.below(n)));
            placed += 1;
        }
        CircuitSource { n, t_count, gates, base: Affine::plus_state(n) }
    }

    /// The frozen gate list, for the conformance test that maps it onto a
    /// `holon_qasm::Circuit` (prefixed by the `H^⊗n` layer).
    pub fn gates(&self) -> &[G] {
        &self.gates
    }
}

impl BranchSource for CircuitSource {
    fn n_branches(&self) -> u64 {
        1u64 << self.t_count
    }

    fn n_qubits(&self) -> usize {
        self.n
    }

    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        // T = (1+ω)/2 · I + (1−ω)/2 · Z: leg 0 is the identity leg, leg 1
        // takes the Z and the (1−ω)/2 coefficient.
        const CI: Cyc = Cyc { c: [1, 1, 0, 0], m: 2 };
        const CZ: Cyc = Cyc { c: [1, -1, 0, 0], m: 2 };
        let mut st = self.base.clone();
        let mut coeff = Cyc::ONE;
        let mut ti = 0usize;
        for g in &self.gates {
            match *g {
                G::X(q) => st.x(q),
                G::Z(q) => st.z(q),
                G::S(q) => st.s_gate(q),
                G::Cx(c, t) => st.cx(c, t),
                G::T(q) => {
                    if branch >> ti & 1 == 1 {
                        coeff = coeff.mul(CZ);
                        st.z(q);
                    } else {
                        coeff = coeff.mul(CI);
                    }
                    ti += 1;
                }
            }
        }
        let mut ybits = 0u64;
        for (q, &b) in y.iter().enumerate() {
            if b {
                ybits |= 1 << q;
            }
        }
        coeff.mul(st.amplitude(ybits))
    }
}

// ---------------------------------------------------------------- cpu clock

/// Process CPU time (utime + stime) in kernel clock ticks, from
/// `/proc/self/stat` — zero dependencies, Linux only, `None` elsewhere.
///
/// Wall time on a shared box measures the box; CPU time measures the CODE.
/// Time a thread spends waiting on a runqueue behind somebody else's job is
/// not charged here, so `cpu(s) / cpu(1)` is the threading overhead with the
/// neighbours subtracted out — the number that would still be true on an idle
/// machine (memory-bandwidth contention between the shards themselves DOES
/// still show up, as slower execution, which is correct: that cost is ours).
fn cpu_ticks() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/stat").ok()?;
    // The comm field may contain spaces and parens; fields resume after the last ')'.
    let rest = &s[s.rfind(')')? + 1..];
    let f: Vec<&str> = rest.split_whitespace().collect();
    // post-')' index 11 = utime, 12 = stime (proc(5)).
    Some(f.get(11)?.parse::<u64>().ok()? + f.get(12)?.parse::<u64>().ok()?)
}

// --------------------------------------------------------------------- main

fn main() {
    let n = 12usize;
    let t = 14usize;
    let depth = 120usize;
    let repeats = 25usize;
    let src = CircuitSource::new(n, t, depth, 0x_C1_5E_ED_9A_11_02_37_41);
    let y: Vec<bool> = (0..n).map(|q| q % 3 == 0).collect();

    println!("holon-mesh-bench — deterministic parallel branch fold");
    println!(
        "  source: n = {n} qubits, t = {t} T-gates -> {} branches, {} gates/branch",
        src.n_branches(),
        src.gates().len()
    );
    println!("  hardware threads: {}", std::thread::available_parallelism().map(|p| p.get()).unwrap_or(0));
    if let Ok(la) = std::fs::read_to_string("/proc/loadavg") {
        println!("  loadavg at start: {}", la.trim());
    }

    let shard_counts = [1usize, 2, 4, 8, 16];

    // INTERLEAVED, one repeat of every arm before the next repeat of any —
    // measured 2026-08-27: batching all repeats of one arm together lets a
    // burst of background load land entirely on the baseline and reports
    // 132% parallel efficiency, which is not a thing. Interleaving makes
    // every arm sample the same time window, and min-of-repeats then
    // estimates the uncontended cost rather than the day's average.
    let mut times: Vec<Vec<f64>> = vec![Vec::with_capacity(repeats); shard_counts.len()];
    let warm: Vec<Cyc> =
        shard_counts.iter().map(|&s| mesh::fold_amplitude(&src, &y, s)).collect();
    for _ in 0..repeats {
        for (k, &s) in shard_counts.iter().enumerate() {
            let t0 = std::time::Instant::now();
            let val = mesh::fold_amplitude(&src, &y, s);
            times[k].push(t0.elapsed().as_secs_f64());
            assert_eq!(val, warm[k], "mesh: run-to-run drift at shards = {s}");
        }
    }
    let results: Vec<(usize, Cyc, f64, f64)> = shard_counts
        .iter()
        .enumerate()
        .map(|(k, &s)| {
            let mut t = times[k].clone();
            t.sort_by(|a, b| a.partial_cmp(b).unwrap());
            (s, warm[k], t[0], t[t.len() / 2])
        })
        .collect();

    // Second pass, BLOCKED rather than interleaved: total CPU time per arm.
    // Blocking is fine here precisely because CPU time is contention-immune.
    let cpu: Vec<Option<u64>> = shard_counts
        .iter()
        .map(|&s| {
            let before = cpu_ticks()?;
            for _ in 0..repeats {
                std::hint::black_box(mesh::fold_amplitude(&src, &y, s));
            }
            Some(cpu_ticks()? - before)
        })
        .collect();

    // The certificate, at bench scale: every shard count returned the SAME
    // ledger entry — the same struct, not the same float.
    let reference = results[0].1;
    let all_equal = results.iter().all(|r| r.1 == reference);
    let (re, im) = reference.to_complex();

    println!("\n  amplitude <y|psi> = {re:.12} + {im:.12}i   |amp|^2 = {:.12}", re * re + im * im);
    println!("  exact ledger entry: c = {:?}, m = {}", reference.c, reference.m);
    println!(
        "  determinism at bench scale (shards {:?}, exact Cyc equality): {}",
        shard_counts,
        if all_equal { "PASS" } else { "FAIL" }
    );

    let base = results[0].2;
    let cpu_base = cpu[0].unwrap_or(0) as f64;
    println!(
        "\n  shards |   min (ms) |   med (ms) | wall speedup | wall eff | cpu inflation | cpu eff"
    );
    println!(
        "  -------+------------+------------+--------------+----------+---------------+--------"
    );
    for (k, &(s, _, min, med)) in results.iter().enumerate() {
        let sp = base / min;
        let (infl, ceff) = match cpu[k] {
            Some(t) if cpu_base > 0.0 => {
                let i = t as f64 / cpu_base;
                (format!("{i:>13.3}"), format!("{:>6.1}%", 100.0 / i))
            }
            _ => ("            —".into(), "     —".into()),
        };
        println!(
            "  {s:>6} | {:>10.3} | {:>10.3} | {sp:>12.3} | {:>7.1}% | {infl} | {ceff}",
            min * 1e3,
            med * 1e3,
            100.0 * sp / s as f64
        );
    }
    println!(
        "\n  wall eff = (wall speedup)/shards — what this box delivered today.\n  \
         cpu  eff = cpu(1)/cpu(shards) — the same fold's own overhead, with the\n  \
         neighbours subtracted out. Read cpu eff as the ceiling wall eff would\n  \
         approach on an idle machine, and the gap between them as contention."
    );
    if let Ok(la) = std::fs::read_to_string("/proc/loadavg") {
        println!("\n  loadavg at end: {}", la.trim());
    }
    println!(
        "  baseline (shards = 1) spawns no threads at all, so every thread cost is\n  \
         charged to the parallel rows, where it is actually paid. Efficiency below\n  \
         100% is real overhead: spawn/join, the merge, and whatever memory bandwidth\n  \
         {} concurrent affine solves contend for. Efficiency ABOVE 100% is not a\n  \
         result, it is a contended baseline — read the loadavg lines before the curve.",
        shard_counts.last().unwrap()
    );

    if !all_equal {
        std::process::exit(1);
    }
}
