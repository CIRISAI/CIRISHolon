//! THE EXACTNESS GATE for the native ZX pass and its circuit extractor.
//!
//! The contract is an equation, checked on the WHOLE MATRIX:
//!
//!   for every random Clifford+T circuit and every PAIR of basis states
//!   (x, y), the diagram's exact value — before rewriting, after Clifford
//!   simplification, after full reduction, and after extraction back to a
//!   circuit — equals `run::amplitude` on the original circuit, EXACTLY as an
//!   element of `Z[ω]·2^{−m/2}`. Not up to a global phase, not to a
//!   tolerance: equal.
//!
//! WHY BOTH INDICES, recorded because getting this wrong cost a whole round
//! of false confidence: an earlier version of this file swept only `y`, with
//! the input pinned at |0…0⟩. That tests ONE COLUMN of the matrix, and
//! |0…0⟩ is permutation-symmetric — so a wrong or missing OUTPUT PERMUTATION
//! is completely invisible to it. Three separate mutations of extraction's
//! permutation stage survived that gate. Sweeping `x` as well is what makes
//! the gate a gate.
//!
//! `⟨y|C|x⟩` is obtained as `⟨y|C·X^x|0…0⟩`, since the runner computes
//! amplitudes from the all-zero input.
use holon::ledger::Cyc;
use holon::qasm::Surface::{self, *};
use holon::run::amplitude;
use holon::zx::{cyc_eq, from_surface};

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

fn bits(n: usize, b: u32) -> Vec<bool> {
    (0..n).map(|q| b >> q & 1 == 1).collect()
}

/// `⟨y|C|x⟩` as the certified runner computes it.
fn reference(n: usize, surf: &[Surface], x: &[bool], y: &[bool]) -> Cyc {
    let mut prog: Vec<Surface> = (0..n).filter(|&q| x[q]).map(X).collect();
    prog.extend_from_slice(surf);
    let (core, _) = holon::qasm::lower(&prog);
    amplitude(n, &core, y)
}

/// The same amplitude as the ZX diagram computes it, under `stage`.
fn zx_amplitude(
    n: usize,
    surf: &[Surface],
    x: &[bool],
    y: &[bool],
    stage: fn(&mut holon::zx::ZxGraph),
) -> Cyc {
    let mut g = from_surface(n, surf).unwrap();
    g.plug_inputs(x);
    g.plug_outputs(y);
    stage(&mut g);
    g.eval()
}

/// Sweep the full 2^n × 2^n matrix.
fn sweep(seed: u64, trials: usize, n: usize, depth: usize, stage: fn(&mut holon::zx::ZxGraph)) {
    let mut rng = Rng(seed);
    for trial in 0..trials {
        let surf = rand_surface(&mut rng, n, depth);
        for bx in 0..(1u32 << n) {
            let x = bits(n, bx);
            for by in 0..(1u32 << n) {
                let y = bits(n, by);
                let want = reference(n, &surf, &x, &y);
                let got = zx_amplitude(n, &surf, &x, &y, stage);
                assert!(
                    cyc_eq(want, got),
                    "trial {trial} <{by:0w$b}|C|{bx:0w$b}>: runner {:?} vs zx {:?}\n\
                     circuit {surf:?}",
                    want.to_complex(),
                    got.to_complex(),
                    w = n
                );
            }
        }
    }
}

#[test]
fn construction_reproduces_the_runners_amplitude_exactly() {
    sweep(0x2ea1, 12, 3, 24, |_| {});
}

#[test]
fn clifford_simplification_preserves_every_amplitude_exactly() {
    sweep(0x2ea2, 12, 3, 24, |g| {
        g.clifford_simp();
    });
}

#[test]
fn full_reduction_preserves_every_amplitude_exactly() {
    sweep(0x2ea3, 12, 3, 24, |g| {
        g.full_reduce();
    });
}

#[test]
fn full_reduction_is_exact_on_wider_and_deeper_circuits() {
    sweep(0x2ea4, 5, 4, 60, |g| {
        g.full_reduce();
    });
}

/// The boundary path: reduce the OPEN diagram (boundaries still free, so
/// `gen_pivot` must unfuse them), THEN plug and evaluate. This is the rung
/// the first pass never reached, and it is where a wrong boundary rule shows.
#[test]
fn open_diagram_reduction_preserves_every_amplitude_exactly() {
    let mut rng = Rng(0x2ea5);
    for trial in 0..20 {
        let n = if trial % 2 == 0 { 3 } else { 4 };
        let surf = rand_surface(&mut rng, n, 24 + 12 * (trial % 3));
        let mut reduced = from_surface(n, &surf).unwrap();
        reduced.full_reduce();
        for bx in 0..(1u32 << n) {
            let x = bits(n, bx);
            for by in 0..(1u32 << n) {
                let y = bits(n, by);
                let want = reference(n, &surf, &x, &y);
                let mut g = reduced.clone();
                g.plug_inputs(&x);
                g.plug_outputs(&y);
                assert!(
                    cyc_eq(want, g.eval()),
                    "trial {trial} <{by:0w$b}|C|{bx:0w$b}>: open-reduced diagram disagrees",
                    w = n
                );
            }
        }
    }
}

/// Full reduction must never make the T-count worse, on either diagram.
#[test]
fn reduction_never_increases_t_count() {
    let mut rng = Rng(0x2ea6);
    for _ in 0..30 {
        let n = 4;
        let surf = rand_surface(&mut rng, n, 40);
        let raw = from_surface(n, &surf).unwrap().t_count();
        assert!(holon::zx::full_reduced_t_count(n, &surf).unwrap() <= raw);
        let zero = vec![false; n];
        assert!(holon::zx::amplitude_t_count(n, &surf, &zero, &zero).unwrap() <= raw);
    }
}

/// The Clifford layer never INCREASES the count (its graph rewrites are
/// parity-preserving; only fusion and the scalar rules remove), and full
/// reduction is at least as good as it — the gadget layer is the difference.
#[test]
fn full_reduction_dominates_the_clifford_layer() {
    let mut rng = Rng(0x2ea7);
    for _ in 0..20 {
        let n = 3;
        let surf = rand_surface(&mut rng, n, 30);
        let raw = from_surface(n, &surf).unwrap().t_count();
        let cliff = holon::zx::simplified_t_count(n, &surf).unwrap();
        let full = holon::zx::full_reduced_t_count(n, &surf).unwrap();
        assert!(cliff <= raw, "Clifford layer grew the T-count: {raw} -> {cliff}");
        assert!(full <= cliff, "full reduction lost to the Clifford layer: {cliff} -> {full}");
    }
}

// ---------------------------------------------------------------- extraction

/// T-count of a surface circuit: core gates whose phase is an odd multiple
/// of π/4, counted after lowering so CCZ/CCX are priced properly.
fn circuit_t_count(surf: &[Surface]) -> usize {
    let (core, _) = holon::qasm::lower(surf);
    core.iter().filter(|g| g.is_t()).count()
}

/// THE EXTRACTION GATE:  ⟨y|C|x⟩ == scalar · ⟨y|extracted|x⟩, whole matrix.
fn extraction_sweep(seed: u64, trials: usize, n: usize, depth: usize) {
    let mut rng = Rng(seed);
    for trial in 0..trials {
        let surf = rand_surface(&mut rng, n, depth);
        let ex = match holon::zx::extract_circuit(n, &surf) {
            Ok(e) => e,
            Err(e) => panic!("trial {trial}: extraction failed: {e}\ncircuit {surf:?}"),
        };
        for bx in 0..(1u32 << n) {
            let x = bits(n, bx);
            for by in 0..(1u32 << n) {
                let y = bits(n, by);
                let want = reference(n, &surf, &x, &y);
                let got = ex.scalar.mul(reference(n, &ex.gates, &x, &y));
                assert!(
                    cyc_eq(want, got),
                    "trial {trial} <{by:0w$b}|C|{bx:0w$b}>: original {:?} vs extracted·scalar {:?}\n\
                     circuit {surf:?}\nextracted {:?}",
                    want.to_complex(),
                    got.to_complex(),
                    ex.gates,
                    w = n
                );
            }
        }
    }
}

#[test]
fn extraction_preserves_every_amplitude_exactly() {
    extraction_sweep(0x3e01, 15, 3, 24);
}

#[test]
fn extraction_is_exact_on_four_qubits() {
    extraction_sweep(0x3e02, 6, 4, 40);
}

#[test]
fn extraction_is_exact_on_deeper_circuits() {
    extraction_sweep(0x3e03, 3, 4, 70);
}

/// Five qubits: deep enough that the FINAL PERMUTATION stage of extraction
/// engages on essentially every instance, and small enough that the 2^5 × 2^5
/// matrix is still checked entry by entry.
#[test]
fn extraction_is_exact_where_the_permutation_phase_engages() {
    extraction_sweep(0x3e07, 2, 5, 50);
}

/// Extraction must not CREATE non-Clifford gates: the circuit that comes out
/// carries exactly the T-count the oracle read off the reduced diagram.
#[test]
fn extraction_matches_the_oracles_t_count() {
    let mut rng = Rng(0x3e04);
    for trial in 0..25 {
        let n = 4;
        let surf = rand_surface(&mut rng, n, 40);
        let oracle = holon::zx::full_reduced_t_count(n, &surf).unwrap();
        let ex = holon::zx::extract_circuit(n, &surf).unwrap();
        assert_eq!(
            circuit_t_count(&ex.gates),
            oracle,
            "trial {trial}: extracted T-count disagrees with the oracle"
        );
    }
}

/// An all-Clifford circuit must extract to an all-Clifford circuit.
#[test]
fn clifford_circuits_extract_with_zero_t() {
    let mut rng = Rng(0x3e05);
    for _ in 0..15 {
        let n = 3;
        let mut surf = rand_surface(&mut rng, n, 30);
        surf.retain(|g| !matches!(g, T(_) | Tdg(_) | Ccz(..) | DiagPow(..)));
        let ex = holon::zx::extract_circuit(n, &surf).unwrap();
        assert_eq!(circuit_t_count(&ex.gates), 0);
    }
}

/// Permutations on purpose: swap-heavy circuits, half of them nothing BUT a
/// permutation, so the residual diagram is entirely the final stage's work.
#[test]
fn extraction_realises_output_permutations() {
    let mut rng = Rng(0x3e06);
    for trial in 0..30 {
        let n = 3 + trial % 2;
        let pure = trial % 2 == 0;
        let mut surf: Vec<Surface> = Vec::new();
        for _ in 0..(6 + trial % 5) {
            let a = rng.below(n);
            let mut b = rng.below(n);
            while b == a {
                b = rng.below(n);
            }
            surf.push(Swap(a, b));
            if !pure {
                surf.push(match rng.below(4) {
                    0 => H(a),
                    1 => S(b),
                    2 => T(a),
                    _ => Cx(a, b),
                });
            }
        }
        let ex = holon::zx::extract_circuit(n, &surf)
            .unwrap_or_else(|e| panic!("trial {trial}: extraction failed: {e}"));
        for bx in 0..(1u32 << n) {
            let x = bits(n, bx);
            for by in 0..(1u32 << n) {
                let y = bits(n, by);
                let want = reference(n, &surf, &x, &y);
                let got = ex.scalar.mul(reference(n, &ex.gates, &x, &y));
                assert!(
                    cyc_eq(want, got),
                    "trial {trial} <{by:0w$b}|C|{bx:0w$b}>: permutation lost\n\
                     circuit {surf:?}\nextracted {:?}",
                    ex.gates,
                    w = n
                );
            }
        }
    }
}

/// THE SIZE-FREE CERTIFICATE. The matrix sweeps above stop at five qubits
/// because the runner sums branches; the circuits this project reports on run
/// to fifty. `certify_extraction` composes a circuit with the adjoint of its
/// own extraction and requires the result to reduce to the identity, which
/// costs polynomial time at any size — so it is the check that reaches the
/// scale the reported numbers live at.
#[test]
fn certification_holds_beyond_the_runners_reach() {
    let mut rng = Rng(0x3e08);
    for trial in 0..12 {
        let n = 8 + 2 * (trial % 4);
        let surf = rand_surface(&mut rng, n, 20 * n);
        holon::zx::certify_extraction(n, &surf)
            .unwrap_or_else(|e| panic!("trial {trial} n={n}: {e}"));
    }
}

/// ...and the certificate's own predicate is two-sided, because a checker
/// that always says yes passes every honest case. A CORRUPTED extraction must
/// be rejected: `C · (extracted·X)†` is not the identity, and the reducer has
/// to say so.
#[test]
fn the_certificates_predicate_rejects_a_corrupted_extraction() {
    let mut rng = Rng(0x3e09);
    for trial in 0..10 {
        let n = 4;
        let surf = rand_surface(&mut rng, n, 30);
        let ex = holon::zx::extract_circuit(n, &surf).unwrap();

        // true positive: the honest composite reduces to the bare identity
        let mut good = surf.clone();
        good.extend(holon::zx::adjoint(&ex.gates));
        let mut g = from_surface(n, &good).unwrap();
        g.full_reduce();
        assert!(g.is_identity_wiring(), "trial {trial}: honest composite was not the identity");

        // true negative: one extra gate and it must not be
        for corrupt in [X(trial % n), H(trial % n), S(trial % n)] {
            let mut bad = ex.gates.clone();
            bad.push(corrupt);
            let mut composed = surf.clone();
            composed.extend(holon::zx::adjoint(&bad));
            let mut g = from_surface(n, &composed).unwrap();
            g.full_reduce();
            assert!(
                !g.is_identity_wiring(),
                "trial {trial}: the identity check is blind to a {corrupt:?}-corrupted extraction"
            );
        }
    }
}
