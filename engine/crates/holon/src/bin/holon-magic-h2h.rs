//! Magic5 vs pruned head-to-head on the rewritten engine — the tuner's
//! named Unswept::Magic5VersusPruned, measured. Same circuits, exact
//! agreement asserted, per-arm engine-only timing, medians of 5.
use holon::magic::{cyc_eq, Circuit};
use holon::magic5::Magic5Source;
use holon::mesh;
use holon::prune::{run_pruned, Gate, PruneConfig};

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

fn rand_circuit(rng: &mut Rng, n: usize, depth: usize, t: usize) -> Vec<Gate> {
    let mut g = Vec::new();
    let mut tc = 0;
    for _ in 0..depth {
        let q = rng.below(n);
        let mut q2 = rng.below(n);
        while q2 == q {
            q2 = rng.below(n);
        }
        match rng.below(8) {
            0 => g.push(Gate::X(q)),
            1 => g.push(Gate::Z(q)),
            2 => g.push(Gate::H(q)),
            3 => g.push(Gate::S(q)),
            4 | 5 if tc < t => {
                tc += 1;
                g.push(Gate::T(q));
            }
            _ => g.push(Gate::Cx(q, q2)),
        }
    }
    // pad to exact t
    while tc < t {
        g.push(Gate::T(rng.below(n)));
        tc += 1;
    }
    g
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let shards = std::thread::available_parallelism().map(|p| p.get()).unwrap_or(4);
    println!("# magic5 vs pruned, exact agreement asserted, medians of 5, shards={shards}");
    println!("{:>4} {:>4} {:>12} {:>12} {:>18} {:>10}", "n", "t", "pruned ms", "magic5 ms", "magic5 branches", "ratio p/m");
    for &(n, t) in &[(16usize, 12usize), (32, 16), (64, 16), (64, 20), (128, 20), (256, 20), (256, 24)] {
        let mut rng = Rng(0xC0DE + (n * 1000 + t) as u64);
        let gates = rand_circuit(&mut rng, n, 12 * n, t);
        let mut y = vec![false; n];
        for b in y.iter_mut() {
            if rng.below(2) == 1 {
                *b = true;
            }
        }
        let mut tp = Vec::new();
        let mut tm = Vec::new();
        let mut a_p = None;
        let mut a_m = None;
        let mut branches = 0u64;
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let sum = run_pruned(n, &gates, &PruneConfig::default());
            let ap = mesh::fold_amplitude(&sum, &y, shards);
            tp.push(t0.elapsed().as_secs_f64() * 1e3);
            a_p = Some(ap);

            let t1 = std::time::Instant::now();
            let c = Circuit { n_qubits: n, gates: gates.clone() };
            let src = Magic5Source::new(&c);
            use holon::BranchSource;
            branches = src.n_branches();
            let am = mesh::fold_amplitude(&src, &y, shards);
            tm.push(t1.elapsed().as_secs_f64() * 1e3);
            a_m = Some(am);
        }
        assert!(cyc_eq(a_p.unwrap(), a_m.unwrap()), "routes disagree at n={n} t={t}");
        let (p, m) = (median(tp), median(tm));
        println!("{:>4} {:>4} {:>12.3} {:>12.3} {:>18} {:>10.3}", n, t, p, m, branches, p / m);
    }
    println!("# ratio > 1 means magic5 leads; exact agreement held on every row");
}
