//! QVM-ACUITY-1, the CHEAP half: plants PQ-1 and PQ-2, and stake S1.
//!
//! `conformance/qasm/QVM_ACUITY1_PREREG.md`. Three referees appear here and
//! they are deliberately three:
//!
//! * **this file's own statevector** (`sv`, below) — written here as the
//!   prereg's S1 demands, dense `2^n` complex amplitudes, and written in a
//!   DIFFERENT style from `sector::referee`'s (a dense matrix–vector product
//!   over the gate's support, against the referee's in-place swaps) so that
//!   agreement between the two is evidence and not an echo;
//! * **the tableau tier** — `holon_qasm::run_tableau`, the certified QASM-1
//!   record, which is a dev-dependency and therefore reachable from `tests/`
//!   and from nowhere else in this crate. PQ-1 and PQ-2 compare against it
//!   TO THE BIT, in the exact dyadic sense: a stabilizer amplitude's `|a|²`
//!   is `int · 2^{-m}` exactly (`sector::cyc_prob_exact`), and so is a
//!   tableau probability, so `==` on `f64` is the right operator and is what
//!   is used;
//! * **the engine's own tableau** — `tableau::PackedTableau`, against which
//!   the closure search's Pauli images are checked gate by gate.

use holon::affine::Gate;
use holon::magic::Circuit;
use holon::magic5::Magic5Source;
use holon::sector::{
    self, closure_of, closure_of_unitary, cyc_prob_exact, full_sum_exact, locate, Closure,
    Observable,
};
use holon::BranchSource;

type C = (f64, f64);

const FRAC_1_SQRT2: f64 = std::f64::consts::FRAC_1_SQRT_2;

fn cadd(a: C, b: C) -> C {
    (a.0 + b.0, a.1 + b.1)
}
fn cmul(a: C, b: C) -> C {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}
fn cabs(a: C) -> f64 {
    (a.0 * a.0 + a.1 * a.1).sqrt()
}

// ---------------------------------------------------------------------------
// THIS FILE'S OWN REFEREE — 2^n complex amplitudes, n ≤ 20
// ---------------------------------------------------------------------------

/// The gate's matrix on its own support, row-major `2^k × 2^k`. Spelled out
/// rather than derived, because this is the independent side of the check.
fn matrix_of(g: Gate) -> (Vec<usize>, Vec<C>) {
    let w = FRAC_1_SQRT2;
    let one = (1.0, 0.0);
    let zero = (0.0, 0.0);
    match g {
        Gate::X(q) => (vec![q], vec![zero, one, one, zero]),
        Gate::Z(q) => (vec![q], vec![one, zero, zero, (-1.0, 0.0)]),
        Gate::H(q) => (vec![q], vec![(w, 0.0), (w, 0.0), (w, 0.0), (-w, 0.0)]),
        Gate::S(q) => (vec![q], vec![one, zero, zero, (0.0, 1.0)]),
        Gate::Sdg(q) => (vec![q], vec![one, zero, zero, (0.0, -1.0)]),
        Gate::T(q) => (vec![q], vec![one, zero, zero, (w, w)]),
        Gate::Tdg(q) => (vec![q], vec![one, zero, zero, (w, -w)]),
        Gate::Cx(c, t) => (
            // basis order over (c, t) with c the LOW local wire: |00>,|10>,|01>,|11>
            // in the (bit0 = c, bit1 = t) indexing used by `apply_dense`.
            vec![c, t],
            vec![
                one, zero, zero, zero, // |00>
                zero, zero, zero, one, // |10> -> |11>
                zero, zero, one, zero, // |01>
                zero, one, zero, zero, // |11> -> |10>
            ],
        ),
    }
}

/// Apply `g` by a dense matrix–vector product over its support: gather the
/// `2^k` amplitudes of each block, multiply by the gate's matrix, scatter
/// them back. (Fixed-size buffers because `k ≤ 2` here; the arithmetic is the
/// general one, which is the point — it is not the referee's in-place swaps.)
fn apply_dense(st: &mut [C], n: usize, g: Gate) {
    let (sup, m) = matrix_of(g);
    let k = sup.len();
    let dim = 1usize << k;
    let bits: [usize; 2] = [1usize << sup[0], 1usize << sup[sup.len() - 1]];
    let mask: usize = if k == 1 { bits[0] } else { bits[0] | bits[1] };
    let mut idx = [0usize; 4];
    let mut old = [(0.0f64, 0.0f64); 4];
    for base in 0..(1usize << n) {
        if base & mask != 0 {
            continue;
        }
        for (l, slot) in idx.iter_mut().enumerate().take(dim) {
            let mut i = base;
            for (b, &bit) in bits.iter().enumerate().take(k) {
                if l >> b & 1 == 1 {
                    i |= bit;
                }
            }
            *slot = i;
        }
        for l in 0..dim {
            old[l] = st[idx[l]];
        }
        for r in 0..dim {
            let mut acc = (0.0, 0.0);
            for (c, o) in old.iter().enumerate().take(dim) {
                acc = cadd(acc, cmul(m[r * dim + c], *o));
            }
            st[idx[r]] = acc;
        }
    }
}

/// `C|0^n⟩`, this file's own.
fn sv(n: usize, gates: &[Gate]) -> Vec<C> {
    let mut st = vec![(0.0f64, 0.0f64); 1usize << n];
    st[0] = (1.0, 0.0);
    for g in gates {
        apply_dense(&mut st, n, *g);
    }
    st
}

fn amp_at(st: &[C], y: &[bool]) -> C {
    st[y.iter().enumerate().filter(|(_, &b)| b).map(|(q, _)| 1usize << q).sum::<usize>()]
}

fn marginal_at(st: &[C], qubits: &[usize], bits: &[bool]) -> f64 {
    st.iter()
        .enumerate()
        .filter(|(i, _)| qubits.iter().zip(bits).all(|(&q, &b)| (i >> q & 1 == 1) == b))
        .map(|(_, a)| a.0 * a.0 + a.1 * a.1)
        .sum()
}

fn max_dev(a: &[C], b: &[C]) -> f64 {
    a.iter().zip(b).map(|(x, y)| cabs((x.0 - y.0, x.1 - y.1))).fold(0.0, f64::max)
}

// ---------------------------------------------------------------------------
// THE TABLEAU TIER — the certified reference, reachable only from tests/
// ---------------------------------------------------------------------------

fn qasm_circuit(n: usize, gates: &[Gate], measured: &[usize]) -> holon_qasm::Circuit {
    use holon_qasm::Gate as Q;
    let gs = gates
        .iter()
        .map(|g| match *g {
            Gate::X(q) => Q::X(q),
            Gate::Z(q) => Q::Z(q),
            Gate::H(q) => Q::H(q),
            Gate::S(q) => Q::S(q),
            Gate::Sdg(q) => Q::Sdg(q),
            Gate::T(q) => Q::T(q),
            Gate::Tdg(q) => Q::Tdg(q),
            Gate::Cx(c, t) => Q::Cx(c, t),
        })
        .collect();
    holon_qasm::Circuit {
        n_qubits: n,
        n_clbits: measured.len(),
        gates: gs,
        measures: measured.iter().enumerate().map(|(cl, &q)| (q, cl)).collect(),
    }
}

/// The tableau tier's probability of one outcome pattern. `run_tableau` keys
/// on clbits most-significant-first, so the key is built the same way.
fn tableau_prob(n: usize, gates: &[Gate], measured: &[usize], bits: &[bool]) -> f64 {
    let c = qasm_circuit(n, gates, measured);
    let out = holon_qasm::run_tableau(&c, holon_qasm::Mutation::None);
    let key: String = (0..measured.len())
        .rev()
        .map(|cl| if bits[cl] { '1' } else { '0' })
        .collect();
    *out.get(&key).unwrap_or(&0.0)
}

// ---------------------------------------------------------------------------
// the search itself: cheap vs hard is decided on the MATRIX
// ---------------------------------------------------------------------------

/// Gates this crate's `Gate` enum cannot spell, put to the closure test
/// directly. This is what makes "a search, not a gate-type table" checkable.
#[test]
fn the_closure_test_reads_matrices_and_not_names() {
    let w = FRAC_1_SQRT2;
    let one = (1.0, 0.0);
    let zero = (0.0, 0.0);

    // CZ — a Clifford this enum has no variant for.
    let cz = vec![
        one, zero, zero, zero, zero, one, zero, zero, zero, zero, one, zero, zero, zero, zero,
        (-1.0, 0.0),
    ];
    assert!(closure_of_unitary(2, &cz).is_closed(), "CZ is cheap");

    // √X — Clifford, and nothing like the enum's spellings.
    let sx = vec![(0.5, 0.5), (0.5, -0.5), (0.5, -0.5), (0.5, 0.5)];
    assert!(closure_of_unitary(1, &sx).is_closed(), "√X is cheap");

    // Y — Clifford.
    let y = vec![zero, (0.0, -1.0), (0.0, 1.0), zero];
    assert!(closure_of_unitary(1, &y).is_closed(), "Y is cheap");

    // A Clifford wearing a GLOBAL PHASE the engine does not carry: still
    // cheap, because a global phase cancels in U P U†. This is the "Clifford
    // up to a phase the engine already tracks" clause, and it is not a
    // special case in the code — it falls out of the conjugation.
    let theta: f64 = 0.317;
    let ph = (theta.cos(), theta.sin());
    let h_phased: Vec<C> = [(w, 0.0), (w, 0.0), (w, 0.0), (-w, 0.0)]
        .iter()
        .map(|&e| cmul(e, ph))
        .collect();
    assert!(closure_of_unitary(1, &h_phased).is_closed(), "e^{{iθ}}·H is cheap");

    // T spelled as a diagonal, and T wearing a global phase: HARD both ways.
    let t_diag = vec![one, zero, zero, (w, w)];
    let t_phased: Vec<C> = t_diag.iter().map(|&e| cmul(e, ph)).collect();
    for (name, u) in [("diag(1,ω)", &t_diag), ("e^{iθ}·diag(1,ω)", &t_phased)] {
        match closure_of_unitary(1, u) {
            Closure::Split { terms, .. } => assert_eq!(terms, 2, "{name} splits X in two"),
            c => panic!("{name} must be hard: {c:?}"),
        }
    }

    // Controlled-S: non-Clifford on two wires, and not a T anywhere in sight.
    let cs = vec![
        one, zero, zero, zero, zero, one, zero, zero, zero, zero, one, zero, zero, zero, zero,
        (0.0, 1.0),
    ];
    assert!(!closure_of_unitary(2, &cs).is_closed(), "controlled-S is hard");

    // Toffoli: eight by eight, and hard.
    let mut ccx = vec![zero; 64];
    for i in 0..8usize {
        let j = if i & 3 == 3 { i ^ 4 } else { i };
        ccx[j * 8 + i] = one;
    }
    assert!(!closure_of_unitary(3, &ccx).is_closed(), "the Toffoli is hard");
}

/// The closure search's images ARE the tableau's update rule: what the search
/// predicts for each Pauli generator is what the engine's own
/// `PackedTableau` does to that row. And for `T` there is no image to check,
/// which is `tableau_not_closed_under_rotation` in the engineering face.
#[test]
fn the_closure_images_are_the_engines_tableau_update() {
    use holon::tableau::PackedTableau;
    // Gates written on their own local wires (`0..k` in support order), which
    // is the frame the images are reported in.
    for g in [Gate::X(0), Gate::Z(0), Gate::H(0), Gate::S(0), Gate::Sdg(0), Gate::Cx(0, 1)] {
        let k = sector::support(g).len();
        let images = match closure_of(g) {
            Closure::Closed { images } => images,
            c => panic!("{g:?} should be closed: {c:?}"),
        };
        let mut tab = PackedTableau::new(k);
        match g {
            Gate::X(q) => tab.x_gate(q),
            Gate::Z(q) => tab.z_gate(q),
            Gate::H(q) => tab.h(q),
            Gate::S(q) => tab.s(q),
            Gate::Sdg(q) => tab.sdg(q),
            Gate::Cx(c, t) => tab.cx(c, t),
            _ => unreachable!(),
        }
        for j in 0..k {
            // row j is the destabilizer that began as X_j; row k+j the
            // stabilizer that began as Z_j.
            for (row, img) in [(j, images[2 * j]), (k + j, images[2 * j + 1])] {
                let r = &tab.rows[row];
                for b in 0..k {
                    assert_eq!(r.x.get(b), img.x >> b & 1 == 1, "{g:?} row {row} x bit {b}");
                    assert_eq!(r.z.get(b), img.z >> b & 1 == 1, "{g:?} row {row} z bit {b}");
                }
                assert_eq!(r.r, img.phase, "{g:?} row {row} sign");
            }
        }
    }
    for g in [Gate::T(0), Gate::Tdg(0)] {
        assert!(!closure_of(g).is_closed(), "{g:?} has no tableau update to check");
    }
}

/// The driver's referee and this file's referee are the same function written
/// twice. If they ever disagree, every number below is suspect — so this runs
/// first among the value tests.
#[test]
fn the_two_referees_agree() {
    for (n, t, seed) in [(6usize, 0usize, 1u64), (6, 5, 2), (10, 8, 3), (12, 4, 4)] {
        let c = sector::random_instance(n, t, seed);
        let mine = sv(n, &c.gates);
        let theirs = sector::referee::statevector(n, &c.gates);
        assert!(
            max_dev(&mine, &theirs) < 1e-12,
            "the two referees disagree at n={n} t={t} seed={seed}: {}",
            max_dev(&mine, &theirs)
        );
    }
}

/// The exact branch sum is the same number as the referee — the value path
/// the driver reports, checked before any stake leans on it.
#[test]
fn the_branch_sum_is_the_referee() {
    for (n, t, seed) in [(6usize, 0usize, 11u64), (6, 4, 12), (8, 6, 13), (10, 8, 14)] {
        let c = sector::random_instance(n, t, seed);
        let st = sv(n, &c.gates);
        let y = sector::argmax_bitstring(n, &c.gates);
        let src = Magic5Source::new(&c);
        let got = full_sum_exact(&src, &y, 1).to_complex();
        let want = amp_at(&st, &y);
        assert!(
            cabs((got.0 - want.0, got.1 - want.1)) < 1e-12,
            "branch sum vs referee at n={n} t={t} seed={seed}: {got:?} vs {want:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// PQ-1 — a Clifford-only circuit
// ---------------------------------------------------------------------------

#[test]
fn pq1_clifford_only_has_an_empty_hard_sector_and_the_tableaus_answer() {
    for seed in [1u64, 2, 3, 4, 5] {
        let n = 6usize;
        let c = sector::random_instance(n, 0, seed);
        // The carrier is nonzero by construction (M-PLANT-SECTOR): y is where
        // the amplitude lives, not the all-zeros string that a coset state
        // usually misses.
        let y = sector::argmax_bitstring(n, &c.gates);
        let sec = locate(&c.gates, &Observable::Amplitude(y.clone()));

        assert!(sec.hard.is_empty(), "PQ-1: the hard sector must be EMPTY");
        assert!(sec.removed.is_empty(), "PQ-1: nothing to remove from a Clifford circuit");
        assert_eq!(sec.t_eff, 0);
        assert_eq!(sec.price(), 1, "PQ-1: the price is 1");
        assert_eq!(sec.clifford.len(), c.gates.len(), "PQ-1: every gate is cheap");

        // ... and the sum is ONE branch.
        let src = Magic5Source::new(&sec.reduced_circuit(&c.gates));
        assert_eq!(src.n_branches(), 1, "PQ-1: one branch");

        let exact = full_sum_exact(&src, &y, 1);
        let p = cyc_prob_exact(exact).expect("PQ-1: a stabilizer amplitude's |a|² is dyadic");
        let p_tab = tableau_prob(n, &c.gates, &(0..n).collect::<Vec<_>>(), &y);
        assert!(p > 0.0, "PQ-1: the carrier is nonzero (seed {seed})");
        assert_eq!(
            p, p_tab,
            "PQ-1: the located answer must equal the tableau tier's TO THE BIT (seed {seed})"
        );

        // and the amplitude itself against this file's statevector.
        let want = amp_at(&sv(n, &c.gates), &y);
        let got = exact.to_complex();
        assert!(cabs((got.0 - want.0, got.1 - want.1)) < 1e-12);
        println!("PQ-1 seed {seed}: p = {p} = tableau tier, one branch, price 1");
    }
}

// ---------------------------------------------------------------------------
// PQ-2 — every T outside the observable's light cone
// ---------------------------------------------------------------------------

/// The carrier, by construction: two blocks that never touch. The observable
/// is the marginal on `{0,1,2,3}`; block A entangles `{0,1,2,3,4}`, block B
/// carries every `T` on `{5,6,7}`. The two blocks are INTERLEAVED in time, so
/// the removal is a statement about support and not about "the tail of the
/// circuit".
fn pq2_circuit() -> (usize, Vec<Gate>) {
    let a = [
        Gate::H(0),
        Gate::Cx(0, 1),
        Gate::S(1),
        Gate::H(2),
        Gate::Cx(2, 3),
        Gate::Cx(1, 2),
        Gate::H(4),
        Gate::Cx(4, 0),
        Gate::Sdg(3),
        Gate::Cx(3, 4),
        Gate::H(1),
        Gate::Cx(0, 2),
        Gate::X(3),
    ];
    let b = [
        Gate::H(6),
        Gate::Cx(6, 7),
        Gate::T(7),
        Gate::H(5),
        Gate::Cx(7, 5),
        Gate::Tdg(5),
        Gate::T(6),
        Gate::Cx(5, 6),
        Gate::T(5),
        Gate::H(7),
    ];
    let mut gates = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < a.len() || j < b.len() {
        if i < a.len() {
            gates.push(a[i]);
            i += 1;
        }
        if j < b.len() {
            gates.push(b[j]);
            j += 1;
        }
    }
    (8, gates)
}

#[test]
fn pq2_t_gates_outside_the_cone_are_removed_and_the_answer_is_the_tableaus() {
    let (n, gates) = pq2_circuit();
    let obs = Observable::Marginal { qubits: vec![0, 1, 2, 3], bits: vec![false; 4] };
    let sec = locate(&gates, &obs);

    assert_eq!(sec.removed.len(), 4, "PQ-2: all four T gates lie outside the cone");
    assert_eq!(sec.t_eff, 0, "PQ-2: t_eff = 0 after removal");
    assert_eq!(sec.price(), 1, "PQ-2: the price after removal is 1");
    assert_eq!(sec.light_cone, vec![0, 1, 2, 3, 4], "PQ-2: the cone is block A");

    let reduced = sec.drop_removed(&gates);
    let cone_only = sec.light_cone_circuit(&gates);

    // THE CARRIER IS NONZERO (M-PLANT-SECTOR): the removed gates are real T
    // gates that really do something — the full state moves when they go.
    let full = sv(n, &gates);
    let red = sv(n, &reduced);
    assert!(
        max_dev(&full, &red) > 1e-3,
        "PQ-2: the removed gates must be NON-trivial on the state, else the plant is empty"
    );

    // ... and the observable does not move at all.
    let want = marginal_at(&full, &[0, 1, 2, 3], &[false; 4]);
    for (what, st) in [("drop-removed", &red), ("light-cone-only", &sv(n, &cone_only))] {
        let got = marginal_at(st, &[0, 1, 2, 3], &[false; 4]);
        assert!(
            (got - want).abs() <= 1e-12,
            "PQ-2: the {what} circuit moved the marginal by {}",
            (got - want).abs()
        );
    }

    // The answer equals the TABLEAU TIER's, to the bit. After removal the
    // circuit is Clifford, so the tableau tier can run it at all — which is
    // the plant's point.
    let p_tab = tableau_prob(n, &reduced, &[0, 1, 2, 3], &[false; 4]);
    // ... and through the located sector: one branch, summed over the cone's
    // free wires (qubit 4), exactly in the ring.
    let src = Magic5Source::new(&Circuit { n_qubits: n, gates: cone_only.clone() });
    assert_eq!(src.n_branches(), 1);
    let mut p_exact = 0.0f64;
    for m in 0..2usize {
        let mut y = vec![false; n];
        y[4] = m == 1;
        let a = full_sum_exact(&src, &y, 1);
        p_exact += cyc_prob_exact(a).expect("a stabilizer amplitude's |a|² is dyadic");
    }
    assert!(p_tab > 0.0, "PQ-2: the carrier is nonzero");
    assert_eq!(p_exact, p_tab, "PQ-2: the located answer equals the tableau tier's TO THE BIT");
    assert!((p_exact - want).abs() <= 1e-12, "PQ-2: and the statevector agrees");
    println!(
        "PQ-2: removed 4 of 4 T gates, t_eff = 0, price 1, p = {p_exact} = tableau tier, \
         state moved by {:.3} when the removed gates went",
        max_dev(&full, &red)
    );
}

// ---------------------------------------------------------------------------
// S1 — the located sector on the prereg's instances
// ---------------------------------------------------------------------------
//
// The stake: "on every instance the located sector reproduces the observable
// exactly when the removed gates are dropped (referee: the exact
// statevector); the removed count is reported. KILL: one instance where
// dropping a 'removed' gate changes the observable by more than 1e-12."
//
// The kill is stated per GATE, so it is tested per gate: every T in the
// circuit is dropped ON ITS OWN and the observable re-refereed. The same
// sweep gives the negative direction for free — for each dropped gate we
// learn whether the observable moved — so `hard` and `removed` are checked
// against each other on the same evidence.
//
// WHAT THE DEEP FAMILY TURNED OUT TO BE (measured, and reported rather than
// worked around): at the prereg's `20n` Clifford depth the 4-qubit marginal
// is EXACTLY 1/16 on every instance, and no T gate anywhere in the circuit
// moves it by more than 1e-15. The mechanism is not numerical:
// `p(0000) = (1/16)·Σ_{P ∈ ⟨Z₀,Z₁,Z₂,Z₃⟩} ⟨0|C†PC|0⟩`, and conjugating each
// of the 15 nontrivial `P` backwards through the circuit gives a sum of
// Paulis that all carry an X or Y somewhere (a `T` only ever mixes X with Y
// on its own wire), so every one of those expectations is exactly zero and
// stays exactly zero when a T is deleted. The deep marginal is a CONSTANT,
// and a stake settled against a constant is settled vacuously.
//
// So the marginal's own sensitivity is measured and reported, the NEGATIVE
// check is taken on the AMPLITUDE (which every instance's T gates do move —
// by ~0.65 |a|), and a SHALLOW family is added below at Clifford depth `2n`,
// where the marginal takes values from 0 to 0.85 and both directions of the
// claim have somewhere to fail. The prereg's own grid is run exactly as
// written and its numbers are in the table.

struct S1Row {
    n: usize,
    t: usize,
    seed: u64,
    mult: usize,
    removed_amp: usize,
    removed_marg: usize,
    cone: usize,
    p: f64,
    /// marginal, dropping every removed gate at once
    d_reduced: f64,
    /// marginal, keeping only the light cone (Cliffords too)
    d_cone: f64,
    /// marginal, worst over dropping each REMOVED gate on its own — the
    /// prereg's kill condition, per gate
    d_removed_each: f64,
    /// amplitude, best over dropping each KEPT gate on its own — the
    /// negative check
    neg_amp: f64,
    /// marginal, best over dropping each KEPT gate on its own
    neg_marg: f64,
    /// how far the STATE moved when the removed gates were dropped — 0 when
    /// there were none, and small when a removed T sits on an idle wire
    state_moved: f64,
}

fn s1_instance(n: usize, t: usize, seed: u64, mult: usize) -> S1Row {
    let c = sector::random_instance_depth(n, t, seed, mult * n);
    let gates = c.gates.clone();
    assert_eq!(gates.iter().filter(|g| g.is_t()).count(), t);
    let full = sv(n, &gates);

    // --- the AMPLITUDE observable ---
    let y = sector::argmax_bitstring(n, &gates);
    let sec_a = locate(&gates, &Observable::Amplitude(y.clone()));
    assert_eq!(sec_a.t_eff, t, "an amplitude reads every wire, so every T is in its cone");
    assert_eq!(sec_a.removed.len(), 0);
    assert_eq!(sec_a.light_cone.len(), n);
    assert_eq!(sec_a.drop_removed(&gates), gates, "nothing to drop, so nothing dropped");
    let amp_full = amp_at(&full, &y);
    assert!(cabs(amp_full) > 1e-9, "the declared y is not vacuous (|a| = {})", cabs(amp_full));

    // --- the MARGINAL observable ---
    let qs = [0usize, 1, 2, 3];
    let bits = [false; 4];
    let obs = Observable::Marginal { qubits: qs.to_vec(), bits: bits.to_vec() };
    let sec_m = locate(&gates, &obs);
    assert_eq!(sec_m.t_eff + sec_m.removed.len(), t, "every T is either kept or removed");
    let want = marginal_at(&full, &qs, &bits);

    // the stake, all removed gates dropped at once
    let reduced = sec_m.drop_removed(&gates);
    let d_reduced = (marginal_at(&sv(n, &reduced), &qs, &bits) - want).abs();
    assert!(
        d_reduced <= 1e-12,
        "S1 KILL: dropping the removed gates moved the observable by {d_reduced} \
         at n={n} t={t} seed={seed} depth={mult}n"
    );

    // the same theorem's stronger reduction: keep ONLY the light cone
    let cone_only = sec_m.light_cone_circuit(&gates);
    let d_cone = (marginal_at(&sv(n, &cone_only), &qs, &bits) - want).abs();
    assert!(
        d_cone <= 1e-12,
        "S1 (the stronger reduction): the light-cone circuit moved the observable by {d_cone}"
    );

    // Are the removed gates idle? With them gone the STATE should move even
    // though the observable does not — that is what makes the removal a
    // claim rather than a tautology. MEASURED, not asserted per instance:
    // on a shallow circuit a T can sit on a wire still in |0⟩, where it IS
    // the identity (measured: n=12, t=8, seed 1, depth 2n). The family-level
    // assertion is in the callers.
    let state_moved =
        if sec_m.removed.is_empty() { 0.0 } else { max_dev(&full, &sv(n, &reduced)) };

    // --- every T dropped ON ITS OWN: the kill condition, per gate, and the
    //     negative check, on the same evidence ---
    let removed_at: Vec<usize> = sec_m.removed.iter().map(|(i, _)| *i).collect();
    let mut d_removed_each = 0.0f64;
    let mut neg_amp = 0.0f64;
    let mut neg_marg = 0.0f64;
    // Every REMOVED gate is tried (that is the kill condition); the kept ones
    // are tried until the negative check has enough evidence, because at
    // n = 20 each try is a whole 2^20 statevector.
    let kept_budget = if n >= 20 { 4usize } else { usize::MAX };
    let mut kept_tried = 0usize;
    for (k, g) in gates.iter().enumerate() {
        if !g.is_t() {
            continue;
        }
        if !removed_at.contains(&k) {
            if kept_tried >= kept_budget {
                continue;
            }
            kept_tried += 1;
        }
        let mut cut: Vec<Gate> = gates.clone();
        cut.remove(k);
        let st = sv(n, &cut);
        let dm = (marginal_at(&st, &qs, &bits) - want).abs();
        let a = amp_at(&st, &y);
        let da = cabs((a.0 - amp_full.0, a.1 - amp_full.1));
        neg_amp = neg_amp.max(da);
        if removed_at.contains(&k) {
            d_removed_each = d_removed_each.max(dm);
            assert!(
                dm <= 1e-12,
                "S1 KILL: dropping the REMOVED gate at position {k} moved the observable by \
                 {dm} at n={n} t={t} seed={seed} depth={mult}n"
            );
        } else {
            neg_marg = neg_marg.max(dm);
        }
    }
    // The negative check is REPORTED here and asserted by the callers: on the
    // prereg's own grid every instance must move (and does), while a shallow
    // circuit can leave a wire in |0> where a T is literally the identity
    // (measured: n=12, t=8, seed 1, depth 2n — no T moves the amplitude at
    // all, because none of them acts on anything).

    S1Row {
        n,
        t,
        seed,
        mult,
        removed_amp: sec_a.removed.len(),
        removed_marg: sec_m.removed.len(),
        cone: sec_m.light_cone.len(),
        p: want,
        d_reduced,
        d_cone,
        d_removed_each,
        neg_amp,
        neg_marg,
        state_moved,
    }
}

fn s1_report(what: &str, rows: &[S1Row]) {
    println!(
        "{what}\n  n   t seed depth | rm(amp) rm(marg) cone |     p(0000) | Δdrop-rm  Δcone-only \
         Δrm-each | NEG amp   NEG marg"
    );
    for r in rows {
        println!(
            "{:3} {:3} {:4} {:4}n | {:7} {:8} {:4} | {:11.6} | {:9.2e} {:11.2e} {:8.2e} | \
             {:9.2e} {:9.2e}",
            r.n,
            r.t,
            r.seed,
            r.mult,
            r.removed_amp,
            r.removed_marg,
            r.cone,
            r.p,
            r.d_reduced,
            r.d_cone,
            r.d_removed_each,
            r.neg_amp,
            r.neg_marg
        );
    }
    let tot: usize = rows.iter().map(|r| r.removed_marg).sum();
    let worst = rows
        .iter()
        .map(|r| r.d_reduced.max(r.d_cone).max(r.d_removed_each))
        .fold(0.0, f64::max);
    let weak_amp = rows.iter().map(|r| r.neg_amp).fold(f64::INFINITY, f64::min);
    let sensitive = rows.iter().filter(|r| r.neg_marg > 1e-9).count();
    let pinned = rows.iter().filter(|r| (r.p - 1.0 / 16.0).abs() < 1e-12).count();
    let with_removals = rows.iter().filter(|r| r.removed_marg > 0).count();
    let moved = rows.iter().filter(|r| r.state_moved > 1e-9).count();
    println!(
        "S1: {} instances, {tot} T gates removed on the marginal, worst Δ = {worst:.3e} \
         (stake 1e-12); negative check on the amplitude: weakest {weak_amp:.3e}; \
         {sensitive}/{} instances have a marginal ANY kept T moves; {pinned}/{} have \
         p(0000) = 1/16 exactly; the removed gates moved the STATE on {moved} of the \
         {with_removals} instances that had any",
        rows.len(),
        rows.len(),
        rows.len()
    );
    assert!(
        with_removals == 0 || moved > 0,
        "S1: not one removed gate anywhere in this family did anything to the state — the \
         removal claim would be about gates that were already the identity"
    );
    let amp_moved = rows.iter().filter(|r| r.neg_amp > 1e-9).count();
    println!(
        "S1: the negative check fires on {amp_moved} of {} instances (a kept T that moves the \
         amplitude)",
        rows.len()
    );
    assert!(
        amp_moved > 0,
        "S1: not one kept T anywhere in this family moved the amplitude — the hard sector \
         would be vacuous"
    );
}

/// The prereg's grid at one width: report, then the per-instance form of the
/// negative check, which this family is dense enough to carry.
fn s1_prereg(n: usize) {
    let rows = s1_grid(n, 20);
    s1_report(&format!("S1 — the prereg's grid, n = {n}"), &rows);
    for r in &rows {
        assert!(
            r.neg_amp > 1e-9,
            "S1 negative check: no kept T moved the AMPLITUDE at n={} t={} seed={} (worst {})",
            r.n,
            r.t,
            r.seed,
            r.neg_amp
        );
    }
}

fn s1_grid(n: usize, mult: usize) -> Vec<S1Row> {
    let mut rows = Vec::new();
    for t in [8usize, 12, 16] {
        for seed in [1u64, 2, 3] {
            rows.push(s1_instance(n, t, seed, mult));
        }
    }
    rows
}

#[test]
fn s1_n12() {
    s1_prereg(12);
}

#[test]
fn s1_n16() {
    s1_prereg(16);
}

/// `#[ignore]`d 2026-09-22: the `n = 20` grid needs a 2^20 statevector per instance and took the
/// CI job past its 75-minute timeout (run 35681801681, cancelled twice); its result is banked in
/// `conformance/qasm/QVM_ACUITY1_RESULTS.md` (27 instances, worst change 2.6e-15). Run by hand:
/// `cargo test --release -p holon --test qvm_acuity_cheap s1_n20 -- --ignored`.
#[test]
#[ignore = "the n = 20 statevector grid; banked in QVM_ACUITY1_RESULTS.md, run by hand"]
fn s1_n20() {
    s1_prereg(20);
}

/// The SHALLOW family, added because the deep one's marginal is a constant
/// (see the note above this section). At Clifford depth `2n` the light cone
/// does not saturate, the marginal is not maximally mixed, and both
/// directions of S1's claim have room to fail — so the same assertions mean
/// something here that they cannot mean at `20n`.
#[test]
fn s1_shallow_where_the_marginal_can_move() {
    let mut rows = Vec::new();
    for n in [12usize, 16] {
        for t in [8usize, 12, 16] {
            for seed in [1u64, 2, 3, 4, 5] {
                rows.push(s1_instance(n, t, seed, 2));
            }
        }
    }
    s1_report("S1 — the added shallow family, Clifford depth 2n", &rows);
    let sensitive = rows.iter().filter(|r| r.neg_marg > 1e-9).count();
    assert!(
        sensitive > 0,
        "the shallow family was added so the marginal could move, and it did not"
    );
    let varied = rows.iter().filter(|r| (r.p - 1.0 / 16.0).abs() > 1e-12).count();
    assert!(
        varied > rows.len() / 2,
        "the shallow family's marginal is as pinned as the deep one's: {varied} of {}",
        rows.len()
    );
}

/// The price is stated from the LOCATED count, and the located count is what
/// the source actually enumerates. (S2 proper belongs to the other half; this
/// is the half of it the cheap side owns: `t_eff` is the number the price is
/// computed from, and `N_pred` is what the branch source builds.)
#[test]
fn the_price_is_computed_from_the_located_t_count() {
    for (n, t, seed) in [(8usize, 8usize, 21u64), (10, 12, 22), (12, 8, 23)] {
        let c = sector::random_instance(n, t, seed);
        let obs = Observable::Marginal { qubits: vec![0, 1, 2, 3], bits: vec![false; 4] };
        let sec = locate(&c.gates, &obs);
        let reduced = sec.reduced_circuit(&c.gates);
        assert_eq!(reduced.t_count(), sec.t_eff);
        let src = Magic5Source::new(&reduced);
        assert_eq!(
            src.n_branches(),
            sec.price(),
            "the price predicted the branch count at n={n} t={t} seed={seed}"
        );
        // and the price after removal is never worse than before it
        assert!(sec.price() <= holon::magic5::expected_branches(t));
    }
}

