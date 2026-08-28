//! ROBUSTNESS SWEEP for the extractor: does it succeed, and is it exact, on
//! circuits nobody chose for it?
//!
//! Extraction can legitimately FAIL — a diagram without a gflow is not
//! extractable, and the published algorithm says so rather than guessing. The
//! question this answers is whether reduced Clifford+T circuit diagrams ever
//! land there in practice, and whether the ones that extract are right. Every
//! instance is certified by `certify_extraction`, which composes the circuit
//! with the adjoint of its own extraction and demands the bare identity back
//! — a check that costs polynomial time, so the sweep can run at sizes the
//! branch-summing runner cannot reach.
use holon::qasm::Surface::{self, *};

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

fn rand_surface(rng: &mut Rng, n: usize, len: usize) -> Vec<Surface> {
    let mut g = Vec::new();
    for _ in 0..len {
        let q = rng.below(n);
        let mut q2 = rng.below(n);
        while q2 == q {
            q2 = rng.below(n);
        }
        let q3 = (0..n).find(|&x| x != q && x != q2).unwrap_or(q);
        g.push(match rng.below(12) {
            0 => X(q),
            1 => Z(q),
            2 => H(q),
            3 => S(q),
            4 => Sdg(q),
            5 => T(q),
            6 => Tdg(q),
            7 => Cx(q, q2),
            8 => Cz(q, q2),
            9 => Ccz(q, q2, q3),
            10 => DiagPow(rng.below(8) as i64, q),
            _ => Swap(q, q2),
        });
    }
    g
}

fn main() {
    let (mut certified, mut failed, mut refused) = (0, 0, 0);
    let mut worst = String::new();
    for &n in &[3usize, 5, 8, 12, 20, 32, 50] {
        for &mult in &[4usize, 10, 25] {
            let depth = n * mult;
            let mut rng = Rng(0xc0ffee ^ (n as u64) << 12 ^ mult as u64);
            let (mut c, mut f, mut r) = (0, 0, 0);
            let t0 = std::time::Instant::now();
            for _ in 0..12 {
                let surf = rand_surface(&mut rng, n, depth);
                match holon::zx::certify_extraction(n, &surf) {
                    Ok(_) => c += 1,
                    Err(e) if e.starts_with("zx:") => {
                        r += 1;
                        if worst.is_empty() {
                            worst = format!("n={n} d={depth}: {e}");
                        }
                    }
                    Err(e) => {
                        f += 1;
                        worst = format!("n={n} d={depth}: {e}");
                    }
                }
            }
            certified += c;
            failed += f;
            refused += r;
            println!(
                "n={n:3} depth={depth:5}   certified {c:2}/12  refused {r}  WRONG {f}   \
                 ({:.2}s)",
                t0.elapsed().as_secs_f64()
            );
        }
    }
    println!(
        "\n{certified} certified exact, {refused} refused (extraction declined, no answer given), \
         {failed} WRONG"
    );
    if !worst.is_empty() {
        println!("first non-success: {worst}");
    }
}
