//! Three-way tier conformance, in-crate: every cheap tier must agree with the
//! statevector carrier exactly on its own stratum. This is the QASM suite's
//! CI backbone (the external qiskit-refereed record lives upstream in
//! CIRISOntology scratchpad/qasm: QASM-1 seven-of-seven, QASM-2 five-of-five).

use holon_qasm::*;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        // splitmix64
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

fn rand_circuit(rng: &mut Rng, stratum: &str, n: usize, depth: usize) -> Circuit {
    let mut gates = Vec::new();
    let mut t_count = 0;
    for _ in 0..depth {
        let q = rng.below(n);
        let mut q2 = rng.below(n);
        while n > 1 && q2 == q {
            q2 = rng.below(n);
        }
        let g = match stratum {
            "classical" => match rng.below(3) {
                0 => Gate::X(q),
                1 if n > 1 => Gate::Cx(q, q2),
                _ => Gate::X(q),
            },
            "clifford" => match rng.below(6) {
                0 => Gate::X(q),
                1 => Gate::Z(q),
                2 => Gate::H(q),
                3 => Gate::S(q),
                4 => Gate::Sdg(q),
                _ if n > 1 => Gate::Cx(q, q2),
                _ => Gate::H(q),
            },
            _ => match rng.below(8) {
                0 => Gate::X(q),
                1 => Gate::Z(q),
                2 => Gate::H(q),
                3 => Gate::S(q),
                4 => Gate::Sdg(q),
                5 | 6 if t_count < 6 => {
                    t_count += 1;
                    if rng.below(2) == 0 {
                        Gate::T(q)
                    } else {
                        Gate::Tdg(q)
                    }
                }
                _ if n > 1 => Gate::Cx(q, q2),
                _ => Gate::H(q),
            },
        };
        gates.push(g);
    }
    let measures = (0..n).map(|q| (q, q)).collect();
    Circuit { n_qubits: n, n_clbits: n, gates, measures }
}

fn max_err(
    a: &std::collections::BTreeMap<String, f64>,
    b: &std::collections::BTreeMap<String, f64>,
) -> f64 {
    let keys: std::collections::BTreeSet<_> = a.keys().chain(b.keys()).collect();
    keys.into_iter()
        .map(|k| (a.get(k).unwrap_or(&0.0) - b.get(k).unwrap_or(&0.0)).abs())
        .fold(0.0, f64::max)
}

#[test]
fn classical_tier_matches_carrier() {
    let mut rng = Rng(1);
    for _ in 0..40 {
        let n = 2 + rng.below(5);
        let d = 4 + rng.below(20);
        let c = rand_circuit(&mut rng, "classical", n, d);
        let e = max_err(&run_classical(&c, Mutation::None), &run_statevector(&c));
        assert!(e <= 1e-12, "classical tier drifted from carrier: {e}");
    }
}

#[test]
fn tableau_tier_matches_carrier() {
    let mut rng = Rng(2);
    for _ in 0..40 {
        let n = 2 + rng.below(4);
        let d = 4 + rng.below(30);
        let c = rand_circuit(&mut rng, "clifford", n, d);
        let e = max_err(&run_tableau(&c, Mutation::None), &run_statevector(&c));
        assert!(e <= 1e-12, "tableau tier drifted from carrier: {e}");
    }
}

#[test]
fn magic_tier_matches_carrier() {
    let mut rng = Rng(3);
    for _ in 0..40 {
        let n = 2 + rng.below(4);
        let d = 4 + rng.below(16);
        let c = rand_circuit(&mut rng, "magic", n, d);
        let e = max_err(&magic::run_magic(&c, false, false), &run_statevector(&c));
        assert!(e <= 1e-12, "magic tier drifted from carrier: {e}");
    }
}

#[test]
fn planted_mutations_are_detected() {
    // Deterministic witnesses: each mutation must move some distribution.
    let mut rng = Rng(4);
    let mut fired = [false; 3];
    for _ in 0..400 {
        let n = 2 + rng.below(3);
        let d = 3 + rng.below(12);
        let c = rand_circuit(&mut rng, "clifford", n, d);
        if max_err(&run_tableau(&c, Mutation::TableauSPhase), &run_statevector(&c)) > 0.2 {
            fired[0] = true;
        }
        if max_err(&run_tableau(&c, Mutation::TableauCxPhase), &run_statevector(&c)) > 0.2 {
            fired[1] = true;
        }
        let nm = 2 + rng.below(3);
        let dm = 4 + rng.below(14);
        let cm = rand_circuit(&mut rng, "magic", nm, dm);
        if max_err(&magic::run_magic(&cm, false, true), &run_statevector(&cm)) > 0.2 {
            fired[2] = true;
        }
        if fired.iter().all(|&f| f) {
            return;
        }
    }
    panic!("mutation detection incomplete: {fired:?}");
}
