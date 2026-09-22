//! MESH-CLIFFORD-1 — the gates and the plants, one test per prereg item.
//!
//! `conformance/bigqvm/MESH_CLIFFORD_PREREG.md` freezes them:
//!
//! * **G1** bit-identity across shard counts — the measurement record and the
//!   final tableau identical at `S ∈ {1,2,4,8}` to the `S = 1` engine, which
//!   is itself identical to the row-major `PackedTableau` reference.
//! * **G2** the crossing count declared and bounded (`< 0.25` at `S = 8`).
//! * **P1** a corrupted shard is CONVICTED, by shard and column.
//! * **P2** a scrambled fold order changes nothing.
//! * **P3** a stream consumed out of circuit order changes the record — so
//!   the stream-order rule is load-bearing and not decorative.
//! * **P4** `S = 1` through the sharded path reproduces the unsharded engine
//!   on every test the crate already carries.
//!
//! G3 (the head-to-head) and G4 (peak RSS) are wall-clock gates and live in
//! the harness, not here; nothing in this file times anything.

use holon::adaptive::{self, Step};
use holon::affine::Gate;
use holon::coladaptive::{z_string_value_of, ColAdaptive};
use holon::sharded::{crossing_count, record_hash, ShardCut, ShardedColAdaptive};
use holon::surface::{Kind, SurfaceCode};
use holon::tableau::PackedTableau;

/// The shard counts the prereg names.
const SHARDS: [usize; 4] = [1, 2, 4, 8];

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// The reference stream, byte for byte `coladaptive.rs`'s — the test needs it
/// to drive `PackedTableau::collapse` in the same order the engine draws.
fn splitmix(state: &mut u64) -> bool {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) & 1 == 1
}

#[derive(Clone, Copy, Debug)]
enum G {
    H(usize),
    S(usize),
    Sdg(usize),
    X(usize),
    Z(usize),
    Cx(usize, usize),
}

/// The bake-off's alphabet ({x,z,h,s,sdg,cx}), seeded, with every qubit
/// measured at the end — `conformance/qasm/battlerig.py::gen_random`'s shape,
/// generated here so the test needs no python and no file.
fn random_clifford(n: usize, depth: usize, seed: u64) -> Vec<G> {
    let mut rng = Rng(0x9E37_79B9 ^ seed.wrapping_mul(0x1000_0001B3) ^ (n as u64) << 17);
    (0..depth)
        .map(|_| {
            let q = rng.below(n);
            let mut q2 = rng.below(n);
            while q2 == q {
                q2 = rng.below(n);
            }
            match rng.below(6) {
                0 => G::H(q),
                1 => G::S(q),
                2 => G::Sdg(q),
                3 => G::X(q),
                4 => G::Z(q),
                _ => G::Cx(q, q2),
            }
        })
        .collect()
}

fn apply_sharded(a: &mut ShardedColAdaptive, g: G) {
    match g {
        G::H(q) => a.h(q),
        G::S(q) => a.s(q),
        G::Sdg(q) => a.sdg(q),
        G::X(q) => a.x_gate(q),
        G::Z(q) => a.z_gate(q),
        G::Cx(c, t) => a.cx(c, t),
    }
}

fn apply_unsharded(a: &mut ColAdaptive, g: G) {
    match g {
        G::H(q) => a.h(q),
        G::S(q) => a.s(q),
        G::Sdg(q) => a.sdg(q),
        G::X(q) => a.x_gate(q),
        G::Z(q) => a.z_gate(q),
        G::Cx(c, t) => a.cx(c, t),
    }
}

fn apply_reference(t: &mut PackedTableau, g: G) {
    match g {
        G::H(q) => t.h(q),
        G::S(q) => t.s(q),
        G::Sdg(q) => t.sdg(q),
        G::X(q) => t.x_gate(q),
        G::Z(q) => t.z_gate(q),
        G::Cx(c, t2) => t.cx(c, t2),
    }
}

fn same_tableau(got: &PackedTableau, want: &PackedTableau, what: &str) {
    assert_eq!(got.n, want.n, "{what}: qubit counts differ");
    for (i, (a, b)) in got.rows.iter().zip(&want.rows).enumerate() {
        assert_eq!(a.x, b.x, "{what}: row {i} X plane differs");
        assert_eq!(a.z, b.z, "{what}: row {i} Z plane differs");
        assert_eq!(a.r % 4, b.r % 4, "{what}: row {i} sign differs");
    }
}

// ---------------------------------------------------------------------------
// The surface-code workload, run on either engine — the flagship's own round,
// reduced to what `--mode bench` does (R rounds of plain extraction).
// ---------------------------------------------------------------------------

fn bench_rounds_sharded(
    code: &SurfaceCode,
    rounds: usize,
    seed: u64,
    shards: usize,
) -> (Vec<bool>, PackedTableau, ShardedColAdaptive) {
    let mut a = ShardedColAdaptive::new(code.n, seed, shards);
    let mut record = Vec::new();
    for _ in 0..rounds {
        for s in &code.stabs {
            if s.kind == Kind::X {
                a.h(s.ancilla);
            }
        }
        for t in 0..4 {
            for s in &code.stabs {
                if let Some(q) = s.sched[t] {
                    match s.kind {
                        Kind::Z => a.cx(q, s.ancilla),
                        Kind::X => a.cx(s.ancilla, q),
                    }
                }
            }
        }
        for s in &code.stabs {
            if s.kind == Kind::X {
                a.h(s.ancilla);
            }
        }
        a.begin_batch();
        let syn: Vec<bool> = code.stabs.iter().map(|s| a.measure(s.ancilla).0).collect();
        a.end_batch();
        for (k, s) in code.stabs.iter().enumerate() {
            if syn[k] {
                a.x_gate(s.ancilla);
            }
        }
        record.extend_from_slice(&syn);
    }
    let packed = a.to_packed();
    (record, packed, a)
}

fn bench_rounds_unsharded(code: &SurfaceCode, rounds: usize, seed: u64) -> (Vec<bool>, PackedTableau) {
    let mut a = ColAdaptive::new(code.n, seed);
    let mut record = Vec::new();
    for _ in 0..rounds {
        for s in &code.stabs {
            if s.kind == Kind::X {
                a.h(s.ancilla);
            }
        }
        for t in 0..4 {
            for s in &code.stabs {
                if let Some(q) = s.sched[t] {
                    match s.kind {
                        Kind::Z => a.cx(q, s.ancilla),
                        Kind::X => a.cx(s.ancilla, q),
                    }
                }
            }
        }
        for s in &code.stabs {
            if s.kind == Kind::X {
                a.h(s.ancilla);
            }
        }
        a.begin_batch();
        let syn: Vec<bool> = code.stabs.iter().map(|s| a.measure(s.ancilla).0).collect();
        a.end_batch();
        for (k, s) in code.stabs.iter().enumerate() {
            if syn[k] {
                a.x_gate(s.ancilla);
            }
        }
        record.extend_from_slice(&syn);
    }
    let packed = a.to_packed();
    (record, packed)
}

// ---------------------------------------------------------------------------
// G1 — bit-identity across shard counts.
// ---------------------------------------------------------------------------

/// G1, the surface code: `d ∈ {21, 45}`, bench mode, 3 rounds, seeds 1–3,
/// `S ∈ {1,2,4,8}`. The record and the final tableau must be identical to the
/// unsharded engine's — which `coladaptive.rs`'s own conformance module gates
/// against the row-major reference, so identity here is identity with the
/// reference. Both numberings are run: the cut must not depend on which
/// qubits happen to sit where, only its CROSSING COUNT may.
#[test]
fn g1_surface_code_is_bit_identical_across_shard_counts() {
    for d in [21usize, 45] {
        for (layout, code) in [
            ("banded", SurfaceCode::banded(d)),
            ("natural", SurfaceCode::new(d)),
        ] {
            for seed in 1..=3u64 {
                let (want_rec, want_tab) = bench_rounds_unsharded(&code, 3, seed);
                let want_hash = record_hash(&want_rec);
                for shards in SHARDS {
                    let (rec, tab, eng) = bench_rounds_sharded(&code, 3, seed, shards);
                    let what = format!("d={d} {layout} seed={seed} S={shards}");
                    assert_eq!(rec, want_rec, "{what}: measurement record differs");
                    assert_eq!(record_hash(&rec), want_hash, "{what}: record digest differs");
                    same_tableau(&tab, &want_tab, &what);
                    // ...and the cut it actually ran, so a pass at a clamped
                    // shard count cannot be read as a pass at the one asked
                    // for.
                    assert_eq!(
                        eng.shards(),
                        ShardCut::new(code.n, shards).shards(),
                        "{what}: engine ran a different cut than the chart declares"
                    );
                }
            }
        }
    }
}

/// G1, the bake-off's random Clifford circuits: `n ∈ {256, 1024}`, five seeds,
/// every qubit measured at the end. `set_parallel_min_work(0)` forces the
/// threaded path even on layers too small to be worth it, so the gate-phase
/// fold is exercised here and not only on the big surface codes.
#[test]
fn g1_random_clifford_circuits_are_bit_identical_across_shard_counts() {
    for n in [256usize, 1024] {
        for seed in 0..5u64 {
            let circuit = random_clifford(n, 20 * n, seed);

            let mut refr = ColAdaptive::new(n, seed);
            for &g in &circuit {
                apply_unsharded(&mut refr, g);
            }
            refr.begin_batch();
            let want_rec: Vec<bool> = (0..n).map(|q| refr.measure(q).0).collect();
            refr.end_batch();
            let want_tab = refr.to_packed();
            let want_hash = record_hash(&want_rec);

            for shards in SHARDS {
                let mut a = ShardedColAdaptive::new(n, seed, shards);
                a.set_parallel_min_work(0);
                for &g in &circuit {
                    apply_sharded(&mut a, g);
                }
                a.begin_batch();
                let rec: Vec<bool> = (0..n).map(|q| a.measure(q).0).collect();
                a.end_batch();
                let what = format!("n={n} seed={seed} S={shards}");
                assert_eq!(rec, want_rec, "{what}: measurement record differs");
                assert_eq!(record_hash(&rec), want_hash, "{what}: record digest differs");
                same_tableau(&a.to_packed(), &want_tab, &what);
            }
        }
    }
}

/// G1's floor: the `S = 1` sharded engine against the CERTIFIED ROW-MAJOR
/// REFERENCE directly, not merely against `ColAdaptive`. The reference runs
/// the interleaved peek/collapse form off the same seeded stream, which is
/// the obligation `coladaptive.rs` carries and the one this file inherits.
#[test]
fn g1_sharded_engine_matches_the_row_major_reference() {
    for n in [64usize, 256] {
        for seed in 0..3u64 {
            let circuit = random_clifford(n, 8 * n, seed);
            for shards in SHARDS {
                let mut a = ShardedColAdaptive::new(n, seed, shards);
                let mut refr = PackedTableau::new(n);
                for &g in &circuit {
                    apply_sharded(&mut a, g);
                    apply_reference(&mut refr, g);
                }
                a.begin_batch();
                let got: Vec<bool> = (0..n).map(|q| a.measure(q).0).collect();
                a.end_batch();

                let mut stream = seed;
                let want: Vec<bool> = (0..n)
                    .map(|q| match refr.measure_peek(q) {
                        Some(b) => b,
                        None => {
                            let b = splitmix(&mut stream);
                            refr.collapse(q, b);
                            b
                        }
                    })
                    .collect();
                let what = format!("n={n} seed={seed} S={shards} vs PackedTableau");
                assert_eq!(got, want, "{what}: outcome streams differ");
                same_tableau(&a.to_packed(), &refr, &what);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// G2 — the crossing count is declared and bounded.
// ---------------------------------------------------------------------------

/// The CX pairs of one extraction round, in schedule order.
fn round_pairs(code: &SurfaceCode) -> Vec<(usize, usize)> {
    (0..4)
        .flat_map(|t| {
            code.stabs.iter().filter_map(move |s| {
                s.sched[t].map(|q| match s.kind {
                    Kind::Z => (q, s.ancilla),
                    Kind::X => (s.ancilla, q),
                })
            })
        })
        .collect()
}

/// G2. The fraction is PRINTED at every `(d, S)` and asserted below 0.25 at
/// `S = 8` — and the same assertion is shown FAILING on the flagship's natural
/// numbering, which is the prereg's branch (c) in the open: no choice of
/// boundary fixes a numbering that puts every data qubit on one side and every
/// ancilla on the other. See the prereg's `### Notes on building`.
#[test]
fn g2_crossing_fraction_is_declared_and_bounded() {
    for d in [21usize, 45] {
        let banded = SurfaceCode::banded(d);
        let natural = SurfaceCode::new(d);
        for shards in SHARDS {
            for (name, code) in [("banded", &banded), ("natural", &natural)] {
                let cut = ShardCut::new(code.n, shards);
                let (c, t) = crossing_count(&cut, round_pairs(code));
                let frac = c as f64 / t as f64;
                println!(
                    "G2 d={d} {name} S={} (asked {shards}): crossing {c}/{t} = {frac:.5}",
                    cut.shards()
                );
                if shards == 8 {
                    if name == "banded" {
                        assert!(frac < 0.25, "d={d} banded S=8: crossing fraction {frac} ≥ 0.25");
                    } else {
                        assert!(
                            frac > 0.9,
                            "d={d} natural S=8: the natural numbering is supposed to be the \
                             BAD cut ({frac}) — if it is now good, the note in the prereg is \
                             stale"
                        );
                    }
                }
            }
        }
        // The engine's own counter must agree with the chart's arithmetic.
        let cut = ShardCut::new(banded.n, 8);
        let (c, t) = crossing_count(&cut, round_pairs(&banded));
        let (_rec, _tab, eng) = bench_rounds_sharded(&banded, 1, 1, 8);
        assert_eq!(
            (eng.mesh.cx_crossing, eng.mesh.cx_total),
            (c, t),
            "d={d}: the engine counted a different crossing set than the cut declares"
        );
    }
}

/// G2 at the flagship's own distance. `#[ignore]`d for wall time: building the
/// code alone is `O(stabilizers²)` in `verify_commuting` (~19880 stabilizers
/// at d = 141), and the run is minutes, not seconds. Run it by hand:
///
///     cargo test --release -p holon --test mesh_clifford_plants -- \
///         --ignored g2_crossing_fraction_at_d141
///
/// or take the same number off the flagship itself, which prints it:
///
///     surface_flagship --d 141 --mode bench --shards 8 --layout banded
#[test]
#[ignore = "d=141: minutes, and the box is shared — run it deliberately"]
fn g2_crossing_fraction_at_d141_s8_is_under_a_quarter() {
    let code = SurfaceCode::banded(141);
    let cut = ShardCut::new(code.n, 8);
    let (c, t) = crossing_count(&cut, round_pairs(&code));
    let frac = c as f64 / t as f64;
    println!(
        "G2 d=141 banded S={} : crossing {c}/{t} = {frac:.5}  (n={})",
        cut.shards(),
        code.n
    );
    assert!(frac < 0.25, "d=141 S=8: crossing fraction {frac} ≥ 0.25");
    // ...and the same number off a real run, not only off the schedule.
    let (_rec, _tab, eng) = bench_rounds_sharded(&code, 1, 1, 8);
    assert_eq!((eng.mesh.cx_crossing, eng.mesh.cx_total), (c, t));
}

// ---------------------------------------------------------------------------
// P1 — a corrupted shard is convicted BY NAME.
// ---------------------------------------------------------------------------

/// P1. One bit of one shard's columns is flipped after the gate phase. The
/// identity check must (a) fire, and (b) say WHICH SHARD and WHICH COLUMN —
/// a gate that only says "differs" cannot tell a corrupted shard from a
/// corrupted merge, which is the whole reason the plant exists.
///
/// Carrier, stated (M-PLANT-SECTOR): the carrier is one shard's column range,
/// and the sector acted on is column `c` of the X plane, which is nonzero in
/// that carrier by construction — the plant picks a column each shard owns.
#[test]
fn p1_a_corrupted_shard_is_convicted_by_shard_and_column() {
    let n = 600;
    let circuit = random_clifford(n, 6 * n, 11);
    for shards in [2usize, 4, 8] {
        let cut = ShardCut::new(n, shards);
        // A clean run is the reference every plant is judged against.
        let mut clean = ShardedColAdaptive::new(n, 11, shards);
        for &g in &circuit {
            apply_sharded(&mut clean, g);
        }
        let reference = clean.to_packed();

        for victim in 0..cut.shards() {
            let range = cut.range(victim);
            let column = range.start + (range.len() / 3);
            let row = (2 * n) / 3;
            let mut planted = ShardedColAdaptive::new(n, 11, shards);
            for &g in &circuit {
                apply_sharded(&mut planted, g);
            }
            planted.plant_x_bit_flip(column, row);

            let verdict = planted
                .first_divergence(&reference)
                .expect("P1: the identity check did not fire on a corrupted shard");
            assert_eq!(
                verdict.shard,
                Some(victim),
                "P1 S={shards}: convicted the wrong shard ({verdict})"
            );
            assert_eq!(
                verdict.column,
                Some(column),
                "P1 S={shards}: convicted the wrong column ({verdict})"
            );
            assert_eq!(verdict.row, row, "P1 S={shards}: convicted the wrong row ({verdict})");
            assert_eq!(verdict.plane, "X");
            // And the plant must be VISIBLE to G1's own comparison, not only
            // to the audit path.
            assert_ne!(
                planted.to_packed().rows[row].x,
                reference.rows[row].x,
                "P1 S={shards}: the corruption did not reach the tableau"
            );
        }
        // The clean engine must NOT be convicted — a check that convicts
        // everything convicts nothing.
        let mut clean2 = ShardedColAdaptive::new(n, 11, shards);
        for &g in &circuit {
            apply_sharded(&mut clean2, g);
        }
        assert_eq!(
            clean2.first_divergence(&reference),
            None,
            "P1 S={shards}: the identity check convicted a clean run"
        );
    }
}

// ---------------------------------------------------------------------------
// P2 — the fold order is not an input.
// ---------------------------------------------------------------------------

/// P2. The shard partials — the gate phase's sign register and the rowsum's
/// `(plus, minus)` pair — are folded in a SCRAMBLED order. The record and the
/// final tableau, signs and all, must be unchanged: the fold is over a
/// commutative monoid (XOR; integer addition), so the declared order buys
/// determinism, not correctness, and this is the test that says which.
///
/// Carrier: the fold order itself, which is nonzero in the sense that matters
/// — every scrambled order used here actually differs from `0..S`.
#[test]
fn p2_a_scrambled_fold_order_changes_nothing() {
    let n = 600;
    let circuit = random_clifford(n, 6 * n, 5);
    for shards in [2usize, 4, 8] {
        let mut straight = ShardedColAdaptive::new(n, 5, shards);
        straight.set_parallel_min_work(0);
        for &g in &circuit {
            apply_sharded(&mut straight, g);
        }
        straight.begin_batch();
        let want: Vec<bool> = (0..n).map(|q| straight.measure(q).0).collect();
        straight.end_batch();
        let want_tab = straight.to_packed();

        let s = straight.shards();
        for scramble in [
            (0..s).rev().collect::<Vec<usize>>(),
            (0..s).map(|i| (i * 3 + 1) % s).collect::<Vec<usize>>(),
        ] {
            // A "scramble" that is the identity would make the plant vacuous.
            let mut order = scramble.clone();
            order.sort_unstable();
            assert_eq!(order, (0..s).collect::<Vec<usize>>(), "not a permutation");
            if s > 1 {
                assert_ne!(scramble, (0..s).collect::<Vec<usize>>(), "P2 plant is vacuous");
            }

            let mut a = ShardedColAdaptive::new(n, 5, shards);
            a.set_parallel_min_work(0);
            a.set_fold_order(scramble.clone());
            for &g in &circuit {
                apply_sharded(&mut a, g);
            }
            a.begin_batch();
            let got: Vec<bool> = (0..n).map(|q| a.measure(q).0).collect();
            a.end_batch();
            let what = format!("S={shards} fold order {scramble:?}");
            assert_eq!(got, want, "P2 {what}: the record moved");
            same_tableau(&a.to_packed(), &want_tab, &what);
        }
    }
}

// ---------------------------------------------------------------------------
// P3 — the stream order IS load-bearing.
// ---------------------------------------------------------------------------

/// P3. The declared random stream is consumed out of circuit order on
/// purpose (pairs swapped). The record must DIFFER — otherwise "one stream,
/// consumed in circuit order, independent of S" would be a decorative rule
/// and G1 would be passing for a reason nobody had checked.
///
/// Carrier: the stream. The sector is the coin flips themselves, nonzero by
/// construction because the test asserts the circuit produced at least two.
#[test]
fn p3_consuming_the_stream_out_of_order_changes_the_record() {
    let n = 256;
    let circuit = random_clifford(n, 6 * n, 3);
    for shards in SHARDS {
        let mut honest = ShardedColAdaptive::new(n, 3, shards);
        for &g in &circuit {
            apply_sharded(&mut honest, g);
        }
        honest.begin_batch();
        let want: Vec<bool> = (0..n).map(|q| honest.measure(q).0).collect();
        honest.end_batch();
        assert!(
            honest.stats.random >= 2,
            "P3 S={shards}: fewer than two coins — the plant would be vacuous"
        );

        let mut scrambled = ShardedColAdaptive::new(n, 3, shards);
        scrambled.set_stream_scramble(true);
        for &g in &circuit {
            apply_sharded(&mut scrambled, g);
        }
        scrambled.begin_batch();
        let got: Vec<bool> = (0..n).map(|q| scrambled.measure(q).0).collect();
        scrambled.end_batch();
        assert_ne!(
            got, want,
            "P3 S={shards}: the record survived an out-of-order stream — \
             the stream-order rule is doing nothing"
        );
        assert_ne!(
            record_hash(&got),
            record_hash(&want),
            "P3 S={shards}: the digest survived an out-of-order stream"
        );
    }
}

// ---------------------------------------------------------------------------
// P4 — S = 1 through the sharded path, on every test the crate carries.
// ---------------------------------------------------------------------------

/// P4 / `matches_the_row_major_reference_exactly`: a random adaptive program
/// — gates, then a measurement batch, repeated — with the deferred resets,
/// against the reference running the interleaved form.
#[test]
fn p4_adaptive_program_matches_the_row_major_reference() {
    for &shards in &SHARDS {
        for n in [4usize, 9, 17, 40, 65, 130] {
            for seed in 0..6u64 {
                let mut rng = Rng(0xADAF_0000 ^ (n as u64) << 8 ^ seed);
                let mut ours = ShardedColAdaptive::new(n, seed, shards);
                ours.set_parallel_min_work(0);
                let mut refr = PackedTableau::new(n);
                let mut ref_rng = seed;
                let mut ours_out = Vec::new();
                let mut ref_out = Vec::new();

                for _round in 0..4 {
                    for _ in 0..(6 * n) {
                        let q = rng.below(n);
                        let mut q2 = rng.below(n);
                        while q2 == q {
                            q2 = rng.below(n);
                        }
                        let g = match rng.below(6) {
                            0 => G::H(q),
                            1 => G::S(q),
                            2 => G::Sdg(q),
                            3 => G::X(q),
                            4 => G::Z(q),
                            _ => G::Cx(q, q2),
                        };
                        apply_sharded(&mut ours, g);
                        apply_reference(&mut refr, g);
                    }
                    let targets: Vec<usize> = (0..n).step_by(2).collect();
                    ours.begin_batch();
                    let mut round: Vec<bool> = Vec::new();
                    for &q in &targets {
                        round.push(ours.measure(q).0);
                    }
                    ours.end_batch();
                    for (k, &q) in targets.iter().enumerate() {
                        if round[k] {
                            ours.x_gate(q);
                        }
                    }
                    ours_out.extend_from_slice(&round);
                    for &q in &targets {
                        let outcome = match refr.measure_peek(q) {
                            Some(b) => b,
                            None => {
                                let b = splitmix(&mut ref_rng);
                                refr.collapse(q, b);
                                b
                            }
                        };
                        if outcome {
                            refr.x_gate(q);
                        }
                        ref_out.push(outcome);
                    }
                }
                let what = format!("S={shards} n={n} seed={seed}");
                assert_eq!(ours_out, ref_out, "{what}: outcome streams differ");
                same_tableau(&ours.to_packed(), &refr, &what);
            }
        }
    }
}

/// P4 / `the_fast_scan_actually_carries_the_deterministic_traffic`.
#[test]
fn p4_the_fast_scan_carries_the_deterministic_traffic() {
    for &shards in &SHARDS {
        let n = 64;
        let mut a = ShardedColAdaptive::new(n, 7, shards);
        a.h(0);
        for q in 1..n {
            a.cx(0, q);
        }
        a.begin_batch();
        let outs: Vec<(bool, bool)> = (0..n).map(|q| a.measure(q)).collect();
        a.end_batch();
        assert!(!outs[0].1, "S={shards}: first GHZ measurement must be a coin");
        for (q, &(o, det)) in outs.iter().enumerate().skip(1) {
            assert!(det, "S={shards}: qubit {q} must be forced after the first collapse");
            assert_eq!(o, outs[0].0, "S={shards}: GHZ outcomes must all agree");
        }
        assert!(a.stats.scan_fast >= 1, "S={shards}: the column scan never ran");
        assert_eq!(a.stats.random, 1, "S={shards}: exactly one coin in a GHZ state");
        assert_eq!(a.stats.deterministic, (n - 1) as u64);
    }
}

/// P4 / `a_deterministic_batch_never_falls_back` — including the claim that a
/// single-term batch allocates no row-major reference at all, which the cut
/// must not quietly undo.
#[test]
fn p4_a_deterministic_batch_never_falls_back() {
    for &shards in &SHARDS {
        let n = 48;
        let mut a = ShardedColAdaptive::new(n, 3, shards);
        for q in (0..n).step_by(3) {
            a.x_gate(q);
        }
        a.begin_batch();
        for q in 0..n {
            let (o, det) = a.measure(q);
            assert!(det, "S={shards}: qubit {q} must be deterministic");
            assert_eq!(o, q % 3 == 0, "S={shards}: qubit {q} outcome");
        }
        a.end_batch();
        assert_eq!(a.stats.scan_fallback, 0, "S={shards}: a coin-free batch fell back");
        assert_eq!(a.stats.scan_fast, n as u64);
        assert_eq!(a.stats.single_term, n as u64, "S={shards}: the O(1) path must carry it");
        assert_eq!(a.stats.transposes, 0, "S={shards}: a single-term batch must not transpose");
        assert!(!a.reference_allocated(), "S={shards}: it allocated the reference anyway");
    }
}

/// P4 / `single_term_shortcut_agrees_with_the_general_product`.
#[test]
fn p4_single_term_shortcut_agrees_with_the_general_product() {
    let mut rng = Rng(0x51E1_7E12);
    let mut saw_multi = false;
    let mut saw_single = false;
    for &shards in &SHARDS {
        for n in [6usize, 12, 33, 64, 70] {
            for seed in 0..8u64 {
                let mut ours = ShardedColAdaptive::new(n, seed, shards);
                let mut refr = PackedTableau::new(n);
                for _ in 0..(4 * n) {
                    let q = rng.below(n);
                    let mut q2 = rng.below(n);
                    while q2 == q {
                        q2 = rng.below(n);
                    }
                    let g = match rng.below(5) {
                        0 => G::X(q),
                        1 => G::Z(q),
                        2 => G::S(q),
                        3 if q.is_multiple_of(3) => G::H(q),
                        _ => G::Cx(q, q2),
                    };
                    apply_sharded(&mut ours, g);
                    apply_reference(&mut refr, g);
                }
                ours.begin_batch();
                for q in 0..n {
                    let want = refr.measure_peek(q);
                    let (got, det) = ours.measure(q);
                    match want {
                        Some(b) => {
                            assert!(det, "S={shards} n={n} q={q}: forced outcome read as a coin");
                            assert_eq!(got, b, "S={shards} n={n} q={q}: wrong forced outcome");
                        }
                        None => {
                            assert!(!det, "S={shards} n={n} q={q}: coin read as forced");
                            refr.collapse(q, got);
                        }
                    }
                }
                ours.end_batch();
                if ours.stats.single_term > 0 {
                    saw_single = true;
                }
                if ours.stats.deterministic > ours.stats.single_term {
                    saw_multi = true;
                }
            }
        }
    }
    assert!(saw_single, "the single-term path never fired — test is vacuous");
    assert!(saw_multi, "the multi-term path never fired — test is vacuous");
}

/// P4 / `z_string_reads_the_logical_observable`.
#[test]
fn p4_z_string_reads_the_logical_observable() {
    for &shards in &SHARDS {
        let mut a = ShardedColAdaptive::new(8, 1, shards);
        assert_eq!(a.z_string_value(&[0, 1, 2, 3]), Some(false));

        let mut b = ShardedColAdaptive::new(8, 1, shards);
        b.x_gate(2);
        assert_eq!(b.z_string_value(&[0, 1, 2, 3]), Some(true));
        b.x_gate(1);
        assert_eq!(b.z_string_value(&[0, 1, 2, 3]), Some(false));
        b.x_gate(6);
        assert_eq!(b.z_string_value(&[0, 1, 2, 3]), Some(false));

        let mut c = ShardedColAdaptive::new(8, 1, shards);
        c.h(0);
        for q in 1..4 {
            c.cx(0, q);
        }
        assert_eq!(c.z_string_value(&[0]), None, "S={shards}: a GHZ qubit alone is a coin");
        assert_eq!(c.z_string_value(&[0, 1, 2, 3]), Some(false), "S={shards}: GHZ parity");
    }
}

/// P4 / `z_string_column_path_agrees_with_the_row_major_reference`.
#[test]
fn p4_z_string_column_path_agrees_with_the_reference() {
    let mut rng = Rng(0x2570_1146);
    let (mut determined, mut undetermined) = (0u32, 0u32);
    for &shards in &SHARDS {
        for n in [4usize, 9, 20, 64, 70] {
            for seed in 0..6u64 {
                let mut a = ShardedColAdaptive::new(n, seed, shards);
                for _ in 0..(4 * n) {
                    let q = rng.below(n);
                    let mut q2 = rng.below(n);
                    while q2 == q {
                        q2 = rng.below(n);
                    }
                    let g = match rng.below(5) {
                        0 => G::H(q),
                        1 => G::S(q),
                        2 => G::X(q),
                        3 => G::Z(q),
                        _ => G::Cx(q, q2),
                    };
                    apply_sharded(&mut a, g);
                }
                let reference = a.to_packed();
                for _ in 0..8 {
                    let k = 1 + rng.below(n);
                    let mut qs: Vec<usize> = (0..k).map(|_| rng.below(n)).collect();
                    qs.sort_unstable();
                    qs.dedup();
                    let want = z_string_value_of(&reference, &qs);
                    let got = a.z_string_value(&qs);
                    assert_eq!(got, want, "S={shards} n={n} seed={seed} string {qs:?}");
                    match want {
                        Some(_) => determined += 1,
                        None => undetermined += 1,
                    }
                }
            }
        }
    }
    assert!(determined > 0, "no determined case — test is vacuous");
    assert!(undetermined > 0, "no undetermined case — test is vacuous");
}

/// P4 / `replayable_and_seed_sensitive`: the seeded stream is still a stream,
/// and — the cut's own clause — the SAME seed at a DIFFERENT shard count is
/// the same stream too.
#[test]
fn p4_replayable_seed_sensitive_and_shard_insensitive() {
    let run = |seed: u64, shards: usize| {
        let n = 24;
        let mut a = ShardedColAdaptive::new(n, seed, shards);
        for q in 0..n {
            a.h(q);
        }
        a.begin_batch();
        let v: Vec<bool> = (0..n).map(|q| a.measure(q).0).collect();
        a.end_batch();
        v
    };
    for &shards in &SHARDS {
        assert_eq!(run(11, shards), run(11, shards), "S={shards}: same seed must replay");
        assert_eq!(run(11, shards), run(11, 1), "S={shards}: the cut moved the stream");
        let set: std::collections::HashSet<Vec<bool>> =
            (0..16u64).map(|s| run(s, shards)).collect();
        assert!(set.len() > 1, "S={shards}: different seeds must diverge");
    }
}

/// P4 / `teleportation_works_on_the_column_engine`.
#[test]
fn p4_teleportation_works_on_the_sharded_engine() {
    for &shards in &SHARDS {
        for seed in 0..32u64 {
            let mut a = ShardedColAdaptive::new(3, seed, shards);
            a.h(0);
            a.h(1);
            a.cx(1, 2);
            a.cx(0, 1);
            a.h(0);
            a.begin_batch();
            let m0 = a.measure(0).0;
            let m1 = a.measure(1).0;
            a.end_batch();
            if m1 {
                a.x_gate(2);
            }
            if m0 {
                a.z_gate(2);
            }
            a.h(2);
            let t = a.to_packed();
            assert_eq!(
                t.measure_peek(2),
                Some(false),
                "S={shards} seed {seed}: teleported state is not |+⟩"
            );
        }
    }
}

/// P4 / `the_patched_mirror_stays_bit_identical_to_the_reference` — the
/// mid-batch invariant, checked where it happens rather than where it would
/// eventually show up.
#[test]
fn p4_the_patched_mirror_stays_bit_identical_to_the_reference() {
    let mut rng = Rng(0x8817_6072);
    let mut collapses = 0u64;
    let mut checked = 0u64;
    for &shards in &SHARDS {
        for n in [5usize, 13, 32, 64, 96] {
            for seed in 0..5u64 {
                let mut a = ShardedColAdaptive::new(n, seed, shards);
                for _ in 0..(5 * n) {
                    let q = rng.below(n);
                    let mut q2 = rng.below(n);
                    while q2 == q {
                        q2 = rng.below(n);
                    }
                    let g = match rng.below(4) {
                        0 => G::H(q),
                        1 => G::S(q),
                        2 => G::X(q),
                        _ => G::Cx(q, q2),
                    };
                    apply_sharded(&mut a, g);
                }
                a.begin_batch();
                for q in 0..n {
                    let (_o, det) = a.measure(q);
                    if !det {
                        collapses += 1;
                    }
                    if a.mirror_x_valid() && a.packed_valid() {
                        let packed = a.packed_ref().expect("packed_valid implies present");
                        checked += 1;
                        for c in 0..n {
                            let col = a.x_column(c);
                            for row in 0..2 * n {
                                let mirror = col[row >> 6] >> (row & 63) & 1 == 1;
                                assert_eq!(
                                    mirror,
                                    packed.rows[row].x.get(c),
                                    "S={shards} n={n} seed={seed} after measuring {q}: \
                                     mirror X[{row}][{c}] diverged"
                                );
                            }
                        }
                    }
                }
                a.end_batch();
            }
        }
    }
    assert!(collapses > 0, "no collapse happened — the test is vacuous");
    assert!(checked > 0, "the mirror was never compared — the test is vacuous");
}

/// P4 / `reset_returns_the_qubit_to_zero_on_the_column_engine`.
#[test]
fn p4_reset_returns_the_qubit_to_zero() {
    for &shards in &SHARDS {
        for seed in 0..8u64 {
            let mut a = ShardedColAdaptive::new(2, seed, shards);
            a.h(0);
            a.cx(0, 1);
            a.begin_batch();
            let (o, det) = a.measure(0);
            a.end_batch();
            assert!(!det, "S={shards} seed {seed}: a Bell-pair qubit must be a coin");
            if o {
                a.x_gate(0);
            }
            assert_eq!(a.z_string_value(&[0]), Some(false), "S={shards} seed {seed}: reset failed");
        }
    }
}

/// P4 / `repetition_code_syndrome_cycle_corrects_on_the_column_engine` — the
/// test that most resembles the flagship.
#[test]
fn p4_repetition_code_syndrome_cycle_corrects() {
    for &shards in &SHARDS {
        for seed in 0..16u64 {
            let mut a = ShardedColAdaptive::new(5, seed, shards);
            a.x_gate(1);
            a.cx(0, 3);
            a.cx(1, 3);
            a.cx(1, 4);
            a.cx(2, 4);
            a.begin_batch();
            let s0 = a.measure(3).0;
            let s1 = a.measure(4).0;
            a.end_batch();
            assert!(s0 && s1, "S={shards} seed {seed}: both syndromes must fire");
            a.x_gate(1);
            for q in 0..3 {
                assert_eq!(
                    a.z_string_value(&[q]),
                    Some(false),
                    "S={shards} seed {seed}: data qubit {q} not corrected"
                );
            }
        }
    }
}

/// P4 / `agrees_with_adaptive_run_on_teleportation`: the reference's own
/// adaptive runner and the sharded engine meeting on `adaptive.rs`'s ground.
#[test]
fn p4_agrees_with_adaptive_run_on_teleportation() {
    for &shards in &SHARDS {
        for seed in 0..16u64 {
            let prog = vec![
                Step::Gate(Gate::H(0)),
                Step::Gate(Gate::H(1)),
                Step::Gate(Gate::Cx(1, 2)),
                Step::Gate(Gate::Cx(0, 1)),
                Step::Gate(Gate::H(0)),
                Step::Measure { q: 0, c: 0 },
                Step::Measure { q: 1, c: 1 },
            ];
            let r = adaptive::run(3, 2, &prog, seed);
            let mut a = ShardedColAdaptive::new(3, seed, shards);
            a.h(0);
            a.h(1);
            a.cx(1, 2);
            a.cx(0, 1);
            a.h(0);
            a.begin_batch();
            let m0 = a.measure(0).0;
            let m1 = a.measure(1).0;
            a.end_batch();
            assert_eq!(vec![m0, m1], r.bits, "S={shards} seed {seed}: outcomes differ");
        }
    }
}
