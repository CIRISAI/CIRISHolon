//! Diagnostic: where does Magic5's per-branch time go, and how many
//! branches does the pruned path actually evaluate after dedup?
use holon::magic::Circuit;
use holon::magic5::Magic5Source;
use holon::prune::{run_pruned, Gate, PruneConfig};
use holon::BranchSource;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    fn below(&mut self, n: usize) -> usize { (self.next() % n as u64) as usize }
}

fn main() {
    for &(n, t) in &[(64usize, 16usize), (128, 20), (256, 24)] {
        let mut rng = Rng(0xC0DE + (n * 1000 + t) as u64);
        let mut g = Vec::new();
        let mut tc = 0;
        for _ in 0..12 * n {
            let q = rng.below(n);
            let mut q2 = rng.below(n);
            while q2 == q { q2 = rng.below(n); }
            match rng.below(8) {
                0 => g.push(Gate::X(q)),
                1 => g.push(Gate::Z(q)),
                2 => g.push(Gate::H(q)),
                3 => g.push(Gate::S(q)),
                4 | 5 if tc < t => { tc += 1; g.push(Gate::T(q)); }
                _ => g.push(Gate::Cx(q, q2)),
            }
        }
        while tc < t { g.push(Gate::T(rng.below(n))); tc += 1; }
        let y = vec![false; n];
        let sum = run_pruned(n, &g, &PruneConfig::default());
        let c = Circuit { n_qubits: n, gates: g.clone() };
        let src = Magic5Source::new(&c);
        let np = sum.n_branches().min(64);
        let t0 = std::time::Instant::now();
        for b in 0..np { let _ = sum.amplitude_of(b, &y); }
        let pb = t0.elapsed().as_secs_f64() / np as f64 * 1e6;
        let nm = src.n_branches().min(16);
        let t1 = std::time::Instant::now();
        for b in 0..nm { let _ = src.amplitude_of(b, &y); }
        let mb = t1.elapsed().as_secs_f64() / nm as f64 * 1e6;
        let tb = std::time::Instant::now();
        let ded = src.deduped();
        let build_ms = tb.elapsed().as_secs_f64() * 1e3;
        let nd = ded.n_branches().min(64);
        let t2 = std::time::Instant::now();
        for b in 0..nd { let _ = ded.amplitude_of(b, &y); }
        let db = t2.elapsed().as_secs_f64() / nd.max(1) as f64 * 1e6;
        println!(
            "n={} t={}  pruned br={}  magic5 br={} -> deduped br={} (collapse {:.1}x)  per-branch us: pruned={:.1} magic5={:.1} deduped={:.1}  dedup build={:.1} ms  per-query after: {:.3} ms vs magic5 {:.3} ms",
            n, t, sum.n_branches(), src.n_branches(), ded.n_branches(),
            src.n_branches() as f64 / ded.n_branches().max(1) as f64,
            pb, mb, db, build_ms,
            db * ded.n_branches() as f64 / 1e3,
            mb * src.n_branches() as f64 / 1e3
        );
    }
}
