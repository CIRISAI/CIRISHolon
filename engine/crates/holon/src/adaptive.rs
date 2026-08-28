//! ADAPTIVE CIRCUITS — the tier-0/1 completion debt, paid.
//!
//! Mid-circuit measurement with feed-forward is the last gap in the tier we
//! already lead (BENCHMARKS: ahead of stim at 7/7 sizes), and it is the
//! capability the referee thesis actually rests on: fault-tolerant machines
//! are ~99.9% Clifford with adaptive syndrome extraction, so a simulator
//! that cannot branch on a measurement cannot referee QEC at all.
//!
//! The physics that makes this free (Aaronson–Gottesman §III, credited):
//! measuring a stabilizer state in the computational basis has only two
//! cases. If no stabilizer anticommutes with `Z_q` the outcome is
//! DETERMINED, and the tableau already knows it. Otherwise the outcome is a
//! FAIR COIN and the update is a tableau operation. Either way the state
//! stays a stabilizer state — so the whole adaptive machinery is Clifford,
//! and the tier's cost model does not move.
//!
//! Determinism discipline, kept: random outcomes come from a SEEDED stream
//! and the seed is part of the result, so an adaptive run is unpredictable
//! in advance and replayable afterwards — the same contract `sample_born`
//! established for terminal measurement.
//!
//! Classical control is a real register, not a fiction: `Measure` writes a
//! bit, `IfBit` gates a Clifford operation on it, and the recorded outcome
//! vector is returned so a caller can audit exactly which branch ran.

use crate::affine::Gate;
use crate::tableau::PackedTableau;

/// An adaptive program: Clifford gates, mid-circuit measurement into a
/// classical bit, and gates conditioned on a bit's value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Step {
    /// A Clifford gate (T/T† are refused — this is the stabilizer tier).
    Gate(Gate),
    /// Measure qubit `q`, writing the outcome to classical bit `c`.
    Measure { q: usize, c: usize },
    /// Reset qubit `q` to |0⟩ (measure and correct — the QEC primitive).
    Reset { q: usize },
    /// Apply `gate` only if classical bit `c` equals `want`.
    IfBit { c: usize, want: bool, gate: Gate },
}

/// The outcome of an adaptive run.
#[derive(Clone, Debug)]
pub struct AdaptiveRun {
    /// Classical register after the run.
    pub bits: Vec<bool>,
    /// Per-measurement record: `(qubit, bit, outcome, was_deterministic)` —
    /// the audit trail that distinguishes a forced outcome from a coin.
    pub record: Vec<(usize, usize, bool, bool)>,
    /// The seed the coins came from (replay is exact).
    pub seed: u64,
    /// The final stabilizer state.
    pub state: PackedTableau,
}

fn splitmix(state: &mut u64) -> bool {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) & 1 == 1
}

/// Run an adaptive Clifford program. `n_bits` sizes the classical register.
pub fn run(n: usize, n_bits: usize, prog: &[Step], seed: u64) -> AdaptiveRun {
    let mut t = PackedTableau::new(n);
    let mut bits = vec![false; n_bits];
    let mut record = Vec::new();
    let mut rng = seed;

    let apply = |t: &mut PackedTableau, g: Gate| match g {
        Gate::X(q) => t.x_gate(q),
        Gate::Z(q) => t.z_gate(q),
        Gate::H(q) => t.h(q),
        Gate::S(q) => t.s(q),
        Gate::Sdg(q) => t.sdg(q),
        Gate::Cx(a, b) => t.cx(a, b),
        Gate::T(_) | Gate::Tdg(_) => {
            panic!("adaptive tier is Clifford: route magic through the branch engines")
        }
    };

    for step in prog {
        match *step {
            Step::Gate(g) => apply(&mut t, g),
            Step::Measure { q, c } => {
                let (outcome, deterministic) = match t.measure_peek(q) {
                    Some(b) => (b, true),
                    None => {
                        let b = splitmix(&mut rng);
                        t.collapse(q, b);
                        (b, false)
                    }
                };
                if c < bits.len() {
                    bits[c] = outcome;
                }
                record.push((q, c, outcome, deterministic));
            }
            Step::Reset { q } => {
                // Measure, then correct if it came out 1 — the QEC primitive,
                // built from the two operations the tier already has.
                let (outcome, deterministic) = match t.measure_peek(q) {
                    Some(b) => (b, true),
                    None => {
                        let b = splitmix(&mut rng);
                        t.collapse(q, b);
                        (b, false)
                    }
                };
                if outcome {
                    t.x_gate(q);
                }
                record.push((q, usize::MAX, outcome, deterministic));
            }
            Step::IfBit { c, want, gate } => {
                if c < bits.len() && bits[c] == want {
                    apply(&mut t, gate);
                }
            }
        }
    }
    AdaptiveRun { bits, record, seed, state: t }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TELEPORTATION — the canonical adaptive protocol, and the strongest
    /// available test: it is CORRECT only if measurement, feed-forward and
    /// the classical register all behave, and it must work for EVERY seed.
    #[test]
    fn teleportation_works_for_every_seed() {
        // q0 = the state to send (prepared |+⟩), q1/q2 = the Bell pair.
        for seed in 0..32u64 {
            let prog = vec![
                Step::Gate(Gate::H(0)),          // |+⟩ to teleport
                Step::Gate(Gate::H(1)),          // Bell pair on 1,2
                Step::Gate(Gate::Cx(1, 2)),
                Step::Gate(Gate::Cx(0, 1)),      // Bell measurement
                Step::Gate(Gate::H(0)),
                Step::Measure { q: 0, c: 0 },
                Step::Measure { q: 1, c: 1 },
                Step::IfBit { c: 1, want: true, gate: Gate::X(2) },
                Step::IfBit { c: 0, want: true, gate: Gate::Z(2) },
            ];
            let r = run(3, 2, &prog, seed);
            // q2 must now be |+⟩: measuring it in the X basis is DETERMINISTIC
            let mut t = r.state;
            t.h(2);
            assert_eq!(
                t.measure_peek(2),
                Some(false),
                "seed {seed}: teleported state is not |+⟩ (bits {:?})",
                r.bits
            );
        }
    }

    /// Deterministic outcomes must be recorded as deterministic, and random
    /// ones as random — the audit trail has to distinguish them.
    #[test]
    fn the_record_separates_forced_outcomes_from_coins() {
        // |0⟩ measured in Z is forced; |+⟩ measured in Z is a coin.
        let prog = vec![
            Step::Measure { q: 0, c: 0 },      // forced 0
            Step::Gate(Gate::H(1)),
            Step::Measure { q: 1, c: 1 },      // coin
        ];
        let r = run(2, 2, &prog, 7);
        assert_eq!(r.record[0].2, false, "|0⟩ must measure 0");
        assert!(r.record[0].3, "and must be recorded as deterministic");
        assert!(!r.record[1].3, "|+⟩ must be recorded as random");
    }

    /// Reset is a real reset: after it, the qubit measures 0 deterministically
    /// no matter what state it was in.
    #[test]
    fn reset_returns_the_qubit_to_zero() {
        for seed in 0..8u64 {
            let prog = vec![
                Step::Gate(Gate::H(0)),
                Step::Gate(Gate::Cx(0, 1)),
                Step::Reset { q: 0 },
            ];
            let mut r = run(2, 0, &prog, seed);
            assert_eq!(r.state.measure_peek(0), Some(false), "seed {seed}: reset failed");
        }
    }

    /// The seeded stream makes an adaptive run replayable exactly, and
    /// different seeds actually explore different branches.
    #[test]
    fn runs_are_replayable_and_seeds_matter() {
        let prog = vec![
            Step::Gate(Gate::H(0)),
            Step::Measure { q: 0, c: 0 },
            Step::Gate(Gate::H(1)),
            Step::Measure { q: 1, c: 1 },
        ];
        let a = run(2, 2, &prog, 42);
        let b = run(2, 2, &prog, 42);
        assert_eq!(a.bits, b.bits, "same seed must replay exactly");
        let outcomes: std::collections::HashSet<Vec<bool>> =
            (0..24u64).map(|s| run(2, 2, &prog, s).bits).collect();
        assert!(outcomes.len() > 1, "different seeds must explore different branches");
    }

    /// A QEC-shaped workload: repeated syndrome extraction with feed-forward
    /// correction on a 3-qubit repetition code must return the data qubits
    /// to the codespace, every seed.
    #[test]
    fn repetition_code_syndrome_cycle_corrects() {
        // data 0,1,2 ; ancillas 3,4. Encode |0_L⟩, inject X on qubit 1,
        // extract both syndromes, correct.
        for seed in 0..16u64 {
            let prog = vec![
                Step::Gate(Gate::X(1)), // the error
                // syndrome 1: parity of data 0,1 into ancilla 3
                Step::Gate(Gate::Cx(0, 3)),
                Step::Gate(Gate::Cx(1, 3)),
                Step::Measure { q: 3, c: 0 },
                // syndrome 2: parity of data 1,2 into ancilla 4
                Step::Gate(Gate::Cx(1, 4)),
                Step::Gate(Gate::Cx(2, 4)),
                Step::Measure { q: 4, c: 1 },
                // both syndromes flag ⇒ the error is on qubit 1
                Step::IfBit { c: 0, want: true, gate: Gate::X(1) },
            ];
            let mut r = run(5, 2, &prog, seed);
            assert_eq!(r.bits, vec![true, true], "seed {seed}: syndromes must both fire");
            for q in 0..3 {
                assert_eq!(
                    r.state.measure_peek(q),
                    Some(false),
                    "seed {seed}: data qubit {q} not corrected"
                );
            }
        }
    }
}
