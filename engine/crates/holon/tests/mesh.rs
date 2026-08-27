//! THE MESH'S CERTIFICATE.
//!
//! `mesh::fold_amplitude` claims two things, and this file is where they are
//! made to pay:
//!
//! 1. **Conformance** — the folded amplitude is the magic tier's amplitude.
//!    Referee: `holon_qasm::magic::magic_amplitude`, the certified reference
//!    engine (QASM-2, five of five), reached through a `BranchSource` adapter
//!    that drives the reference's own `Affine` one branch at a time. The
//!    adapter lives here and not in the library because `holon-qasm` is a
//!    dev-dependency: the mesh must not depend on its own referee.
//!
//! 2. **Determinism** — sharding the fold changes nothing. Not "nothing to
//!    within a tolerance": the same `Cyc` struct, coefficient vector and
//!    denominator exponent alike, at shards ∈ {1, 2, 3, 7, 16}. Exact `Z[ω]`
//!    addition is associative and commutative, so this OUGHT to hold; the
//!    point of the test is that "ought" is not a warrant.
//!
//! And it marks the boundary of claim 2 rather than leaving it implied:
//! `cancelling_partial_sums_are_the_representation_boundary` exhibits a
//! branch source on which two shardings return the same NUMBER in different
//! structs, and names the one mechanism that does it.

use holon::ledger::Cyc;
use holon::merge::{fold as merge_fold, MergeLedger};
use holon::mesh;
use holon::BranchSource;
use holon_qasm::magic::{Affine as RefAffine, Cyc as RefCyc};
use holon_qasm::{Circuit, Gate};

/// The bench's self-contained branch source, pulled in as a module so the
/// fixture the speedup curve is measured on is certified by the same referee
/// as the mesh. (`src/bin` targets cannot use dev-dependencies, which is why
/// that file carries its own affine simulator in the first place.)
#[allow(dead_code)]
#[path = "../src/bin/holon-mesh-bench.rs"]
mod bench_source;

const SHARDS: [usize; 5] = [1, 2, 3, 7, 16];

// ------------------------------------------------------------------ adapter

fn to_ledger(x: RefCyc) -> Cyc {
    Cyc { c: x.c, m: x.m }
}

/// `BranchSource` over the certified reference engine: branch `b` is the
/// circuit with each T-gate resolved to the leg named by bit `b`, evolved
/// through `holon_qasm::magic::Affine` — the reference's own state object,
/// its own update rules, its own exact amplitude query.
struct RefSource {
    c: Circuit,
    t_count: usize,
}

impl RefSource {
    fn new(c: Circuit) -> Self {
        let t_count = c
            .gates
            .iter()
            .filter(|g| matches!(g, Gate::T(_) | Gate::Tdg(_)))
            .count();
        assert!(t_count < 63, "branch index is a u64");
        RefSource { c, t_count }
    }
}

impl BranchSource for RefSource {
    fn n_branches(&self) -> u64 {
        1u64 << self.t_count
    }

    fn n_qubits(&self) -> usize {
        self.c.n_qubits
    }

    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        let mut st = RefAffine::new(self.c.n_qubits);
        let mut coeff = RefCyc::ONE;
        let mut ti = 0usize;
        for g in &self.c.gates {
            match *g {
                Gate::T(q) | Gate::Tdg(q) => {
                    // T = (1+ω)/2 · I + (1−ω)/2 · Z; Tdg with ω → ω⁻¹ = −ω³.
                    let dag = matches!(g, Gate::Tdg(_));
                    let (ci, cz) = if !dag {
                        (RefCyc { c: [1, 1, 0, 0], m: 2 }, RefCyc { c: [1, -1, 0, 0], m: 2 })
                    } else {
                        (RefCyc { c: [1, 0, 0, -1], m: 2 }, RefCyc { c: [1, 0, 0, 1], m: 2 })
                    };
                    if branch >> ti & 1 == 1 {
                        coeff = coeff.mul(cz);
                        st.apply(Gate::Z(q));
                    } else {
                        coeff = coeff.mul(ci);
                    }
                    ti += 1;
                }
                g => st.apply(g),
            }
        }
        // A killed branch returns Cyc::ZERO from `amplitude` itself, and
        // ZERO·coeff is ZERO, which `Cyc::add` skips — same arithmetic the
        // reference's `continue` performs.
        to_ledger(coeff.mul(st.amplitude(y)))
    }
}

// ---------------------------------------------------------------- fixtures

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
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

/// A random Clifford+T circuit with a capped T-count. H is included on
/// purpose: it is the gate that grows the affine column set and reaches the
/// reference's `fold` / `dependent_subset` / `gauss_sum_out` paths, so the
/// branches this fold sums are not all structurally alike.
fn random_circuit(seed: u64, n: usize, depth: usize, t_cap: usize) -> Circuit {
    let mut rng = Rng(seed);
    let mut gates = Vec::with_capacity(depth);
    let mut t_used = 0usize;
    for _ in 0..depth {
        let q = rng.below(n);
        let mut q2 = rng.below(n);
        while q2 == q {
            q2 = rng.below(n);
        }
        let pick = rng.below(8);
        gates.push(match pick {
            0 => Gate::X(q),
            1 => Gate::Z(q),
            2 => Gate::H(q),
            3 => Gate::S(q),
            4 => Gate::Sdg(q),
            5 => Gate::Cx(q, q2),
            6 | 7 if t_used < t_cap => {
                t_used += 1;
                if pick == 6 { Gate::T(q) } else { Gate::Tdg(q) }
            }
            _ => Gate::H(q),
        });
    }
    // Guarantee a nontrivial branch space even if the draw was unlucky.
    while t_used < 3 {
        gates.push(Gate::T(rng.below(n)));
        t_used += 1;
    }
    Circuit { n_qubits: n, n_clbits: n, gates, measures: Vec::new() }
}

fn basis(n: usize, idx: usize) -> Vec<bool> {
    (0..n).map(|q| idx >> q & 1 == 1).collect()
}

// -------------------------------------------------------------- the chart

#[test]
fn shard_ranges_partition_the_branch_space_exactly() {
    for &n in &[0u64, 1, 2, 3, 5, 16, 17, 4096, 16384] {
        for shards in [0usize, 1, 2, 3, 7, 16, 64, 10_000] {
            let r = mesh::shard_ranges(n, shards);
            if n == 0 {
                assert!(r.is_empty(), "no branches means no children");
                continue;
            }
            // One child per shard, clamped to the branch count: no empty
            // ranges, so every child does real work.
            assert_eq!(r.len() as u64, (shards.max(1) as u64).min(n));
            assert!(r.iter().all(|x| x.start < x.end), "empty range in {r:?}");
            assert_eq!(r[0].start, 0);
            assert_eq!(r[r.len() - 1].end, n);
            for w in r.windows(2) {
                assert_eq!(w[0].end, w[1].start, "gap or overlap in {r:?}");
            }
            // Balanced to within one branch — the shards are peers.
            let lo = r.iter().map(|x| x.end - x.start).min().unwrap();
            let hi = r.iter().map(|x| x.end - x.start).max().unwrap();
            assert!(hi - lo <= 1, "unbalanced cut {r:?}");
        }
    }
    // A pure function of its two arguments, which is where the determinism is.
    assert_eq!(mesh::shard_ranges(16384, 7), mesh::shard_ranges(16384, 7));
}

// ------------------------------------------------- conformance + determinism

/// THE CERTIFICATE. Same circuit, same `y`, five different shardings: one
/// exact ledger entry, and it is the reference engine's amplitude.
#[test]
fn fold_is_shard_invariant_and_matches_the_certified_reference() {
    let cases: Vec<(u64, usize, usize, usize)> = vec![
        // (seed, qubits, depth, T-cap)
        (1, 3, 18, 4),
        (2, 4, 26, 6),
        (3, 4, 30, 7),
        (4, 5, 34, 6),
        (5, 5, 40, 7),
        // Fewer branches than shards: the cut must clamp, not fabricate.
        (6, 4, 20, 2),
    ];
    let mut checked = 0usize;
    for (seed, n, depth, t_cap) in cases {
        let circ = random_circuit(seed, n, depth, t_cap);
        let src = RefSource::new(circ.clone());
        for idx in [0usize, 1, (1 << n) - 1] {
            let y = basis(n, idx);

            let folds: Vec<Cyc> =
                SHARDS.iter().map(|&s| mesh::fold_amplitude(&src, &y, s)).collect();
            for (k, f) in folds.iter().enumerate() {
                assert_eq!(
                    *f, folds[0],
                    "sharding changed the LEDGER ENTRY (not just the float): \
                     seed {seed}, y index {idx}, shards {} vs {}: {:?} vs {:?}",
                    SHARDS[k], SHARDS[0], f, folds[0]
                );
            }

            let (re, im) = folds[0].to_complex();
            let (rre, rim) = holon_qasm::magic::magic_amplitude(&circ, &y, false, false);
            assert!(
                (re - rre).abs() < 1e-12 && (im - rim).abs() < 1e-12,
                "fold disagrees with the certified reference: seed {seed}, y {idx}: \
                 ({re}, {im}) vs ({rre}, {rim})"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 18, "the case matrix shrank without anyone saying so");
}

/// Determinism is a claim about the shard COUNT, but the fold also has to be
/// stable run to run at a fixed count — thread scheduling is not an input.
#[test]
fn repeated_folds_at_the_same_shard_count_agree() {
    let circ = random_circuit(11, 5, 36, 8);
    let src = RefSource::new(circ);
    let y = basis(5, 21);
    let first = mesh::fold_amplitude(&src, &y, 7);
    for _ in 0..16 {
        assert_eq!(mesh::fold_amplitude(&src, &y, 7), first);
    }
}

#[test]
fn degenerate_shardings_are_defined() {
    struct Empty;
    impl BranchSource for Empty {
        fn n_branches(&self) -> u64 {
            0
        }
        fn n_qubits(&self) -> usize {
            2
        }
        fn amplitude_of(&self, _b: u64, _y: &[bool]) -> Cyc {
            unreachable!("an empty branch space has nothing to evaluate")
        }
    }
    let y = [false, false];
    for s in [0usize, 1, 8] {
        assert_eq!(mesh::fold_amplitude(&Empty, &y, s), Cyc::ZERO);
    }

    // shards = 0 is the serial fold, not a division by zero.
    let circ = random_circuit(9, 3, 16, 4);
    let src = RefSource::new(circ);
    let y = basis(3, 5);
    assert_eq!(
        mesh::fold_amplitude(&src, &y, 0),
        mesh::fold_amplitude(&src, &y, 1)
    );
    // More shards than branches: clamped, still exact.
    assert_eq!(
        mesh::fold_amplitude(&src, &y, 4096),
        mesh::fold_amplitude(&src, &y, 1)
    );
}

// ------------------------------------------------------- the bench fixture

/// The speedup curve is only worth reading if the thing being sped up is the
/// real computation. The bench binary's self-contained affine source is
/// checked here against the certified reference on the equivalent circuit:
/// `H^⊗n` followed by its frozen gate list.
#[test]
fn bench_source_matches_the_certified_reference() {
    let n = 5usize;
    let src = bench_source::CircuitSource::new(n, 6, 40, 0x_C1_5E_ED_9A_11_02_37_41);
    let mut gates: Vec<Gate> = (0..n).map(Gate::H).collect();
    for g in src.gates() {
        gates.push(match *g {
            bench_source::G::X(q) => Gate::X(q),
            bench_source::G::Z(q) => Gate::Z(q),
            bench_source::G::S(q) => Gate::S(q),
            bench_source::G::Cx(c, t) => Gate::Cx(c, t),
            bench_source::G::T(q) => Gate::T(q),
        });
    }
    let circ = Circuit { n_qubits: n, n_clbits: n, gates, measures: Vec::new() };

    for idx in 0..(1usize << n) {
        let y = basis(n, idx);
        let folds: Vec<Cyc> =
            SHARDS.iter().map(|&s| mesh::fold_amplitude(&src, &y, s)).collect();
        for f in &folds {
            assert_eq!(*f, folds[0], "bench source: sharding changed the ledger entry");
        }
        let (re, im) = folds[0].to_complex();
        let (rre, rim) = holon_qasm::magic::magic_amplitude(&circ, &y, false, false);
        assert!(
            (re - rre).abs() < 1e-12 && (im - rim).abs() < 1e-12,
            "bench source is not the reference's circuit at y = {idx}: \
             ({re}, {im}) vs ({rre}, {rim})"
        );
    }
}

/// Exact arithmetic earns a stronger check than agreement at one basis state:
/// the bench fixture's amplitudes must be a unit vector. `Σ_y |⟨y|ψ⟩|²` is 1
/// to floating-point rounding only because the sum itself is floating point —
/// every amplitude in it is exact.
#[test]
fn bench_source_amplitudes_are_normalised() {
    let n = 6usize;
    let src = bench_source::CircuitSource::new(n, 6, 48, 7);
    let total: f64 = (0..(1usize << n))
        .map(|idx| {
            let (re, im) = mesh::fold_amplitude(&src, &basis(n, idx), 3).to_complex();
            re * re + im * im
        })
        .sum();
    assert!((total - 1.0).abs() < 1e-12, "bench source is not unitary: {total}");
}

// ------------------------------------------------------------- the boundary

/// WHERE THE DETERMINISM CLAIM STOPS, exhibited rather than hedged.
///
/// `Cyc::normalize` halves only while `m ≥ 2`, so it never removes a lone
/// factor of `√2`: `1` is both `([1,0,0,0], m=0)` and `([0,1,0,−1], m=1)`,
/// two normalised faces of one number. `Cyc::add` aligns to the LARGER `m`,
/// which makes the coefficient vector path-independent — except that a
/// partial sum cancelling to exactly zero normalises to `m = 0` and forgets
/// the `m` it came from. Reshard around such a cancellation and the surviving
/// maximum `m` changes parity, so the answer comes back in the other face.
///
/// This is a characterisation, not an aspiration. If `ledger::Cyc` ever
/// learns to divide out `√2`, the inequality here fires — and that failure is
/// good news: delete this test and strengthen the module header.
#[test]
fn cancelling_partial_sums_are_the_representation_boundary() {
    // Three ring elements: a + b = 0 exactly, and c sits at a different m.
    let a = Cyc { c: [1, 0, 0, 0], m: 3 };
    let b = Cyc { c: [-1, 0, 0, 0], m: 3 };
    let c = Cyc { c: [1, 0, 0, 0], m: 0 };

    // ((a+b)+c) loses the m = 3 to the cancellation; (a+(b+c)) keeps it.
    let left = a.add(b).add(c);
    let right = a.add(b.add(c));
    assert_ne!(left, right, "the ledger canonicalised √2 — retire this test");
    let (lr, li) = left.to_complex();
    let (rr, ri) = right.to_complex();
    assert!(
        (lr - rr).abs() < 1e-15 && (li - ri).abs() < 1e-15,
        "the VALUE must be order-independent even where the struct is not"
    );

    // And through the mesh: a three-branch source, sharded two ways.
    struct Cancelling;
    impl BranchSource for Cancelling {
        fn n_branches(&self) -> u64 {
            3
        }
        fn n_qubits(&self) -> usize {
            1
        }
        fn amplitude_of(&self, b: u64, _y: &[bool]) -> Cyc {
            match b {
                0 => Cyc { c: [1, 0, 0, 0], m: 3 },
                1 => Cyc { c: [-1, 0, 0, 0], m: 3 },
                _ => Cyc { c: [1, 0, 0, 0], m: 0 },
            }
        }
    }
    let y = [false];
    let one = mesh::fold_amplitude(&Cancelling, &y, 1);
    let two = mesh::fold_amplitude(&Cancelling, &y, 2);
    assert_ne!(one, two, "the ledger canonicalised √2 — retire this test");
    let ((r1, i1), (r2, i2)) = (one.to_complex(), two.to_complex());
    assert!((r1 - r2).abs() < 1e-15 && (i1 - i2).abs() < 1e-15);

    // The same fixture is the only place the mesh's ORDERING choices are
    // observable at all. On a source with no cancelling partial sum, folding
    // a range backwards or merging the shards backwards changes nothing that
    // any assertion can see; here it changes the struct. So these two lines
    // are what stand behind "ascending within a shard, shard-index order
    // across shards" — without them that sentence is unenforced prose.
    assert_eq!(
        mesh::fold_amplitude(&Cancelling, &y, 3),
        one,
        "one child per branch must fold to the serial answer: the merge is \
         not running in shard-index order"
    );

    // The repair, and the proof it is one: canonicalising both faces makes
    // the shard count irrelevant again, without changing any value.
    assert_eq!(mesh::canonicalize(one), mesh::canonicalize(two));
    assert_eq!(mesh::canonicalize(left), mesh::canonicalize(right));
    let (cr, ci) = mesh::canonicalize(one).to_complex();
    assert!((cr - r1).abs() < 1e-15 && (ci - i1).abs() < 1e-15);
}

/// THE SAME BOUNDARY, STATED AGAINST THE SHARED LAW.
///
/// `src/merge.rs` says the merge law makes "ANY sharding, ordering, or
/// distribution of a fold deterministic without coordination", and
/// `tests/laws.rs::merge_laws_hold_for_every_ledger` checks that for `Cyc`
/// with EXACT equality — forward fold against reversed fold, on forty random
/// ring elements. It passes, but on the luck of the draw: three elements are
/// enough to break it, and this is them, in the law's own `law_check` shape.
///
/// The law is not wrong about what matters — the VALUE is order-independent,
/// which is what every consumer of the fold actually consumes. What it does
/// not have for the tier-2 ledger is order-independence of the
/// REPRESENTATION, and it is stated as though it does. Reported upstream as
/// a misfit; the repair is `ledger::Cyc::normalize`, not this file.
#[test]
fn the_merge_law_is_value_scoped_for_the_exact_ring() {
    let items = vec![
        Cyc { c: [1, 0, 0, 0], m: 3 },
        Cyc { c: [-1, 0, 0, 0], m: 3 },
        Cyc { c: [1, 0, 0, 0], m: 0 },
    ];
    let forward = merge_fold(items.clone());
    let mut reversed = items.clone();
    reversed.reverse();
    let backward = merge_fold(reversed);

    assert_ne!(
        forward, backward,
        "the exact-equality merge law now holds for Cyc — the ledger grew a \
         canonical form, so retire this test and strengthen merge.rs's wording"
    );
    let ((fr, fi), (br, bi)) = (forward.to_complex(), backward.to_complex());
    assert!(
        (fr - br).abs() < 1e-15 && (fi - bi).abs() < 1e-15,
        "the law's VALUE scope must hold even where its struct scope does not: \
         {forward:?} vs {backward:?}"
    );
    // Identity, the law's other clause, is unaffected.
    assert_eq!(Cyc::empty().merge(forward), forward);
    assert_eq!(forward.merge(Cyc::empty()), forward);
    // And the repair closes it.
    assert_eq!(mesh::canonicalize(forward), mesh::canonicalize(backward));
}

#[test]
fn canonicalize_is_value_preserving_and_idempotent() {
    let samples = [
        Cyc::ZERO,
        Cyc::ONE,
        Cyc { c: [0, 1, 0, -1], m: 1 },
        Cyc { c: [2, 0, 0, 0], m: 1 },
        Cyc { c: [1, 2, 0, -2], m: 3 },
        Cyc { c: [3, -1, 4, -1], m: 5 },
        Cyc { c: [0, 0, 0, 0], m: 7 },
        Cyc { c: [16, 16, 16, 16], m: 9 },
    ];
    for s in samples {
        let k = mesh::canonicalize(s);
        let ((a, b), (c, d)) = (s.to_complex(), k.to_complex());
        assert!(
            (a - c).abs() < 1e-12 && (b - d).abs() < 1e-12,
            "canonicalize moved the value: {s:?} -> {k:?}"
        );
        assert_eq!(mesh::canonicalize(k), k, "canonicalize is not idempotent on {s:?}");
    }
    // Equal values in different faces land on the same face.
    assert_eq!(
        mesh::canonicalize(Cyc { c: [1, 0, 0, 0], m: 0 }),
        mesh::canonicalize(Cyc { c: [0, 1, 0, -1], m: 1 })
    );
}

/// The boundary above is reachable in the RING; the question that matters is
/// whether a real branch sum reaches it. This is the wide sweep that answers
/// it empirically — every basis state of six random circuits, at eleven shard
/// counts including several that are coprime to the branch count and so cut
/// the space in genuinely different places. Any hit is a real defect report,
/// not a curiosity: it would mean a magic-tier amplitude's representation
/// depends on the thread budget.
#[test]
fn the_boundary_is_unreached_by_real_branch_sums() {
    let shard_sweep = [1usize, 2, 3, 4, 5, 6, 7, 9, 11, 16, 31];
    let mut combinations = 0usize;
    for seed in 21..=26u64 {
        let n = 4usize;
        let circ = random_circuit(seed, n, 28, 6);
        let src = RefSource::new(circ);
        for idx in 0..(1usize << n) {
            let y = basis(n, idx);
            let base = mesh::fold_amplitude(&src, &y, 1);
            for &s in &shard_sweep {
                assert_eq!(
                    mesh::fold_amplitude(&src, &y, s),
                    base,
                    "seed {seed}, y {idx}, shards {s}: the fold's REPRESENTATION \
                     depends on the shard count"
                );
                combinations += 1;
            }
        }
    }
    assert_eq!(combinations, 6 * 16 * 11);
}
