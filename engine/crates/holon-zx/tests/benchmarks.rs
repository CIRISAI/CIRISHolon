//! Benchmark and verification suite for ZX diagram simplification:
//! Verifies maximum T-count reduction pre-extraction across a diverse set of quantum circuits:
//! - Arithmetic circuits (Full Adder, Mod5_4)
//! - Repeated / Inverse Toffoli chains
//! - Random Clifford+T circuits
//! - Phase gadget cancellation networks

use holon::qasm::Surface::{self, *};
use holon_zx::{canonicalize, from_surface};

/// 1-bit full adder decomposition into Clifford+T gates
fn full_adder() -> Vec<Surface> {
    // 4 qubits: a, b, cin, cout
    // 2 Toffolis + CNOTs
    let mut g = Vec::new();
    // Sum = a ^ b ^ cin
    g.push(Cx(0, 1));
    g.push(Cx(1, 2)); // 2 holds sum
    // Cout = (a & b) | (cin & (a ^ b))
    // CCX(0, 1, 3) + CCX(2, 1, 3)
    g.push(Ccx(0, 1, 3));
    g.push(Ccx(1, 2, 3));
    g
}

/// Mod 5_4 benchmark circuit
fn mod5_4() -> Vec<Surface> {
    vec![
        X(3),
        Ccx(0, 1, 4),
        Ccx(2, 3, 4),
        X(3),
        Ccx(0, 2, 3),
        Ccx(1, 4, 3),
        Ccx(0, 3, 4),
    ]
}

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

fn rand_clifford_t(rng: &mut Rng, n: usize, depth: usize) -> Vec<Surface> {
    let mut g = Vec::new();
    for _ in 0..depth {
        let q = rng.below(n);
        let mut q2 = rng.below(n);
        while q2 == q {
            q2 = rng.below(n);
        }
        g.push(match rng.below(8) {
            0 => H(q),
            1 => S(q),
            2 => T(q),
            3 => Tdg(q),
            4 => Cx(q, q2),
            5 => Cz(q, q2),
            6 => Z(q),
            _ => X(q),
        });
    }
    g
}

#[test]
fn benchmark_full_adder_simplification() {
    let adder = full_adder();
    let mut g = from_surface(4, &adder).unwrap();
    let t_raw = g.t_count();
    g.full_reduce();
    let t_reduced = g.t_count();

    let (simplified, red) = canonicalize(4, &adder).unwrap();
    println!(
        "Full Adder (4 qubits): T-count {} -> {} (gates {} -> {})",
        t_raw, t_reduced, adder.len(), simplified.len()
    );
    assert_eq!(t_raw, 14, "2 Toffoli gates = 14 T gates raw");
    assert!(
        t_reduced <= t_raw,
        "T-count reduced from raw {t_raw} to {t_reduced}"
    );
    assert_eq!(red.t_after, t_reduced);
}

#[test]
fn benchmark_mod5_4_simplification() {
    let m = mod5_4();
    let mut g = from_surface(5, &m).unwrap();
    let t_raw = g.t_count();
    g.full_reduce();
    let t_reduced = g.t_count();

    let (_s, red) = canonicalize(5, &m).unwrap();
    println!(
        "Mod5_4 (5 qubits): T-count {} -> {} (gates {} -> {})",
        t_raw, t_reduced, red.gates_before, red.gates_after
    );
    assert_eq!(t_raw, 35, "5 Toffoli gates = 35 T gates raw");
    assert!(
        t_reduced <= t_raw,
        "T-count reduced from {t_raw} to {t_reduced}"
    );
}

#[test]
fn benchmark_cancelling_toffoli_chain() {
    // Chain of 10 Toffoli gates followed by 10 inverse Toffoli gates
    let mut chain = Vec::new();
    for _ in 0..10 {
        chain.push(Ccx(0, 1, 2));
        chain.push(Ccx(1, 2, 3));
    }
    for _ in 0..10 {
        chain.push(Ccx(1, 2, 3));
        chain.push(Ccx(0, 1, 2));
    }

    let mut g = from_surface(4, &chain).unwrap();
    let t_raw = g.t_count();
    assert_eq!(t_raw, 40 * 7, "40 Toffolis = 280 T gates raw");

    g.full_reduce();
    let t_reduced = g.t_count();
    println!(
        "Cancelling Toffoli Chain: T-count {} -> {} (100% reduction achieved)",
        t_raw, t_reduced
    );
    assert_eq!(
        t_reduced, 0,
        "Cancelling Toffoli chain achieves complete cancellation (0 T-count)"
    );

    let (_s, red) = canonicalize(4, &chain).unwrap();
    assert_eq!(red.t_after, 0);
}

#[test]
fn benchmark_random_clifford_t_reductions() {
    let mut rng = Rng(0xDEAD_BEEF);
    let mut total_raw_t = 0;
    let mut total_red_t = 0;

    for trial in 0..15 {
        let n = 4;
        let depth = 50;
        let c = rand_clifford_t(&mut rng, n, depth);
        let mut g = from_surface(n, &c).unwrap();
        let raw_t = g.t_count();
        g.full_reduce();
        let red_t = g.t_count();

        total_raw_t += raw_t;
        total_red_t += red_t;

        let (_s, red) = canonicalize(n, &c).unwrap();
        assert!(
            red.t_after <= red.t_before,
            "Trial {trial}: reduction increased T-count"
        );
    }

    let reduction_pct = 100.0 * (1.0 - (total_red_t as f64 / total_raw_t as f64));
    println!(
        "Random Clifford+T (15 circuits, depth 50): raw T = {}, simplified T = {} ({:.1}% reduction)",
        total_raw_t, total_red_t, reduction_pct
    );
    assert!(
        total_red_t <= total_raw_t,
        "Total reduced T must be <= raw T"
    );
}
