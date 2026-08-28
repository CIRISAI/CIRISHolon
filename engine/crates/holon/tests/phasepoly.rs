//! The non-local pass must be EXACT (identical amplitudes on every basis
//! state) and must actually cancel magic that the local pass cannot reach.
use holon::phasepoly::{normalized_t_count, optimize};
use holon::qasm::Surface::{self, *};
use holon::run::amplitude;
use holon::simplify::magic_weight;

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

fn amps_equal(n: usize, a: &[Surface], b: &[Surface]) -> bool {
    let (ca, _) = holon::qasm::lower(a);
    let (cb, _) = holon::qasm::lower(b);
    (0..(1u32 << n)).all(|k| {
        let y: Vec<bool> = (0..n).map(|q| k >> q & 1 == 1).collect();
        holon::magic::cyc_eq(amplitude(n, &ca, &y), amplitude(n, &cb, &y))
    })
}

#[test]
fn phase_polynomial_pass_is_exact() {
    let mut rng = Rng(0xF0F0);
    for trial in 0..20 {
        let n = 3;
        let mut g = Vec::new();
        for _ in 0..30 {
            let q = rng.below(n);
            let mut q2 = rng.below(n);
            while q2 == q {
                q2 = rng.below(n);
            }
            let q3 = (0..n).find(|&x| x != q && x != q2).unwrap();
            g.push(match rng.below(8) {
                0 => Z(q),
                1 => S(q),
                2 => T(q),
                3 => Tdg(q),
                4 => Cx(q, q2),
                5 => Cz(q, q2),
                6 => Ccz(q, q2, q3),
                _ => H(q),
            });
        }
        let opt = optimize(n, &g);
        assert!(amps_equal(n, &g, &opt), "trial {trial}: phase-polynomial pass changed the amplitude");
    }
}

/// THE POINT: two T gates separated by a CNOT ladder that returns the frame
/// to itself contribute to the SAME linear form and must cancel — a
/// cancellation the local pass provably cannot see, because the gates never
/// share a diagonal run.
#[test]
fn cancels_magic_at_a_distance() {
    let n = 3;
    // Cx(0,1) changes qubit 1's linear form and leaves qubit 0's alone, so
    // T(0) and Tdg(0) contribute to the SAME form — yet they are separated
    // by non-diagonal gates that do NOT cancel each other adjacently, so no
    // local rewrite can bring them together.
    let prog = vec![
        H(0), H(1), H(2),
        T(0),
        Cx(0, 1),
        Tdg(0),
        Cx(0, 1),
        H(0),
    ];
    // the LOCAL pass cannot cancel these (they are separated by CNOTs)
    let local = holon::simplify::simplify(&prog);
    assert!(
        magic_weight(&local) >= 2,
        "local pass should still see two magic gates, got {}",
        magic_weight(&local)
    );
    // the NON-LOCAL pass must
    assert_eq!(normalized_t_count(n, &prog), 0, "the T/T† pair must cancel at a distance");
    let opt = optimize(n, &prog);
    assert!(amps_equal(n, &prog, &opt), "and it must still be exact");
}

/// A CCZ pair separated by Cliffords that preserve the frame must also
/// cancel — 14 T-equivalents removed non-locally.
#[test]
fn cancels_a_separated_ccz_pair() {
    let n = 3;
    // Same shape at CCZ scale: the intervening CNOT pair brackets the
    // second CCZ, so the two CCZs never meet in a diagonal run, but their
    // phase terms land on identical forms.
    let prog = vec![
        H(0), H(1), H(2),
        Ccz(0, 1, 2),
        Cx(0, 1),
        S(1),
        Sdg(1),
        Cx(0, 1),
        Ccz(2, 1, 0),
        H(2),
    ];
    assert_eq!(normalized_t_count(n, &prog), 0, "separated CCZ pair must cancel");
    assert!(amps_equal(n, &prog, &optimize(n, &prog)));
}
