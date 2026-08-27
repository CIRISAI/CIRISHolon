//! THE CERTIFICATE for the GPU tier: a tier without its square is not a tier.
//!
//! Four things have to hold, and each is a separate test with a separate way to
//! fail:
//!
//! 1. **Schedule-independence.** The same batch, folded at five different
//!    block/grid configurations, returns the same `Cyc` STRUCT — not the same
//!    float, not the same value up to the ring's two normalized faces. The
//!    module header of `gpu` argues this from integer associativity; this
//!    measures it.
//! 2. **Agreement with the CPU mesh.** The same `BranchSource`, folded by
//!    `holon::mesh::fold_amplitude` at shards in {1,2,3,7,16}, returns the same
//!    struct the GPU returns — on a REAL circuit's `PrunedSum`, through holon's
//!    own `Affine::amplitude`, not through this crate's packed twin.
//! 3. **The decode is not inventing anything.** Every descriptor decoded out of
//!    `Affine::canon_key` is driven back against the `Affine` it came from, on a
//!    determining set of basis states.
//! 4. **The checks have teeth.** A planted single-bit defect in one branch of a
//!    million must move the answer; a comparison that cannot see that is not a
//!    comparison. (LESSONS: a planted defect must be observable.)
//!
//! These need a CUDA device. `ci-gates.sh` cannot reach this crate — that is
//! what its empty `[workspace]` table is for — so this suite is run by hand and
//! its results are recorded in GPU.md.

use holon::ledger::Cyc;
use holon::mesh;
use holon::prune::{self, Gate, PruneConfig};
use holon_gpu::desc::{AffineDesc, DescSource, R_ZERO};
use holon_gpu::{cpu, synth, GpuBatch, GpuFolder, Shape};

/// The five launch shapes. They differ in block size (which changes the shuffle
/// tree's depth AND how many warps feed the block reduction), in grid size
/// (which changes how many branches each thread sees through the grid-stride
/// loop), and in whether the grid covers the batch in one pass at all.
const SHAPES: [Shape; 5] = [
    Shape { block: 32, grid: 1 },      // one warp, one block: every branch, one thread each in turn
    Shape { block: 64, grid: 7 },      // a prime grid, so the stride does not divide the batch
    Shape { block: 256, grid: 512 },
    Shape { block: 512, grid: 133 },
    Shape { block: 1024, grid: 4096 }, // more threads than branches in the small batches
];

fn folder() -> GpuFolder {
    GpuFolder::new(0).expect("no CUDA device 0 — this suite needs one")
}

fn bools(y: u64, n: usize) -> Vec<bool> {
    (0..n).map(|q| y >> q & 1 == 1).collect()
}

/// A random Clifford+T circuit with enough T gates to make branches.
fn circuit(n: usize, t: usize, seed: u64) -> Vec<Gate> {
    let mut rng = synth::Rng::new(seed);
    let mut gates = Vec::new();
    for _ in 0..4 {
        for q in 0..n {
            match rng.below(4) {
                0 => gates.push(Gate::H(q)),
                1 => gates.push(Gate::S(q)),
                2 => gates.push(Gate::Cx(q, (q + 1 + rng.below(n - 1)) % n)),
                _ => {}
            }
        }
    }
    for _ in 0..t {
        gates.push(Gate::T(rng.below(n)));
    }
    for _ in 0..2 {
        for q in 0..n {
            if rng.below(2) == 0 {
                gates.push(Gate::H(q));
            }
            if rng.below(3) == 0 {
                gates.push(Gate::Cx(q, (q + 1 + rng.below(n - 1)) % n));
            }
        }
    }
    gates
}

fn pruned(n: usize, t: usize, seed: u64) -> prune::PrunedSum {
    let gates = circuit(n, t, seed);
    let cfg = PruneConfig { merge_every: t.max(1), ..PruneConfig::default() };
    prune::run_pruned(n, &gates, &cfg)
}

// ---------------------------------------------------------------- 1. schedule

#[test]
fn five_launch_shapes_return_the_same_struct() {
    let f = folder();
    let n = 28;
    let descs = synth::batch(n, 20, 200_000, 0xD37E_1131);
    let batch = GpuBatch::upload(&f, &descs).expect("upload");
    assert!(batch.exponent_uniform, "the synthetic generator promises a uniform exponent");

    // Several `y`, including ones on a branch coset (so the sum is not all
    // zeros) and ones off it (so the zero path is exercised in the same fold).
    // NOT `h`: `y = h` is the `u = 0` point, where no `d[a]` and no `J[a][b]`
    // is read at all. Coset points at a nonzero `u` exercise the phase
    // polynomial; the arbitrary ones exercise the off-coset zero path.
    let ys: Vec<u64> = (0..6)
        .map(|i| {
            if i % 2 == 0 {
                descs[i * 977].point(0x2Bu64 + i as u64 * 7)
            } else {
                0xA5A5_5A5A & ((1u64 << n) - 1)
            }
        })
        .collect();

    for &y in &ys {
        let vals: Vec<Cyc> = SHAPES
            .iter()
            .map(|&s| batch.fold(&f, y, s).expect("fold"))
            .collect();
        for (i, v) in vals.iter().enumerate() {
            assert_eq!(
                *v, vals[0],
                "y = {y:#x}: shape {:?} disagreed with shape {:?} — as a STRUCT",
                SHAPES[i], SHAPES[0]
            );
        }
        // ... and against the CPU, at both tiers of the CPU fold.
        let yb = bools(y, n);
        let src = DescSource { descs: descs.clone(), n };
        let cpu_serial = mesh::fold_amplitude(&src, &yb, 1);
        let cpu_shards = mesh::fold_amplitude(&src, &yb, 16);
        assert_eq!(vals[0], cpu_serial, "y = {y:#x}: GPU vs CPU mesh (1 shard)");
        assert_eq!(vals[0], cpu_shards, "y = {y:#x}: GPU vs CPU mesh (16 shards)");
        assert_eq!(vals[0], cpu::fold_packed(&descs, y, 8), "y = {y:#x}: GPU vs packed CPU fold");
    }
}

#[test]
fn repeated_launches_of_one_shape_do_not_drift() {
    let f = folder();
    let n = 24;
    let descs = synth::batch(n, 18, 50_000, 0x1234_5678);
    let batch = GpuBatch::upload(&f, &descs).expect("upload");
    let y = descs[13].point(0x15);
    let first = batch.fold(&f, y, SHAPES[2]).expect("fold");
    for run in 1..5 {
        assert_eq!(batch.fold(&f, y, SHAPES[2]).expect("fold"), first, "run {run} drifted");
    }
}

// ---------------------------------------------------------------- 2. vs holon

#[test]
fn gpu_matches_the_cpu_mesh_on_a_real_circuit() {
    let f = folder();
    let n = 14;
    let sum = pruned(n, 12, 0xBEEF_0001);
    assert!(sum.branches.len() > 16, "want a real branch count, got {}", sum.branches.len());

    let descs: Vec<AffineDesc> = sum
        .branches
        .iter()
        .map(|b| AffineDesc::from_branch(b.weight, &b.state).expect("decode"))
        .collect();
    let batch = GpuBatch::upload(&f, &descs).expect("upload");

    // Every basis state of a 14-qubit register is 16384 folds — enough to hit
    // the on-coset and off-coset paths of every branch, and it makes the test a
    // statement about the WHOLE state vector rather than about a lucky `y`.
    let mut nonzero = 0usize;
    for y in 0u64..(1 << n) {
        let g = batch.fold(&f, y, SHAPES[2]).expect("fold");
        let c = mesh::fold_amplitude(&sum, &bools(y, n), 1);
        assert_eq!(g, c, "y = {y:#x}: GPU vs holon's own PrunedSum fold");
        if c != Cyc::ZERO {
            nonzero += 1;
        }
    }
    assert!(nonzero > 0, "every amplitude was zero: the test proved nothing");

    // ... and the sharded CPU fold, on the states that actually carry weight.
    for y in 0u64..64 {
        let g = batch.fold(&f, y, SHAPES[1]).expect("fold");
        for shards in [1usize, 2, 3, 7, 16] {
            assert_eq!(
                g,
                mesh::fold_amplitude(&sum, &bools(y, n), shards),
                "y = {y:#x}, shards = {shards}"
            );
        }
    }
}

#[test]
fn per_branch_rotation_codes_match_branch_by_branch() {
    // A sum can agree while two branch errors cancel. This is the check that
    // rules that out: every branch's rotation code, device against host.
    let f = folder();
    let n = 12;
    let sum = pruned(n, 10, 0xBEEF_0002);
    let descs: Vec<AffineDesc> = sum
        .branches
        .iter()
        .map(|b| AffineDesc::from_branch(b.weight, &b.state).expect("decode"))
        .collect();
    let batch = GpuBatch::upload(&f, &descs).expect("upload");

    let mut saw_zero = false;
    let mut saw_nonzero = false;
    for y in 0u64..(1 << n) {
        let gpu = batch.rotation_codes(&f, y, SHAPES[3]).expect("codes");
        for (b, d) in descs.iter().enumerate() {
            let host = d.rotation_code(y);
            assert_eq!(gpu[b], host, "branch {b}, y = {y:#x}");
            if host == R_ZERO {
                saw_zero = true;
            } else {
                saw_nonzero = true;
            }
        }
    }
    assert!(saw_zero && saw_nonzero, "the sweep never exercised both paths");
}

// ---------------------------------------------------------------- 3. decode

#[test]
fn decoded_descriptors_agree_with_the_affine_they_came_from() {
    // The decode reads `Affine::canon_key`'s byte layout. This drives the
    // decoded descriptor back against `Affine::amplitude` itself — holon's own
    // Vec<Vec<bool>> solver — so a misread field cannot pass quietly.
    for (n, t, seed) in [(8usize, 6usize, 1u64), (12, 9, 2), (16, 8, 3), (20, 7, 4)] {
        let sum = pruned(n, t, seed);
        assert!(!sum.branches.is_empty());
        for (i, b) in sum.branches.iter().enumerate() {
            let d = AffineDesc::from_branch(b.weight, &b.state)
                .unwrap_or_else(|e| panic!("n={n} branch {i}: {e}"));
            assert_eq!(d.n, n);
            assert!(
                d.agrees_with(b.weight, &b.state, 4096),
                "n={n} branch {i}: the decoded descriptor and the Affine disagree"
            );
        }
    }
}

// ---------------------------------------------------------------- 4. teeth

#[test]
fn a_single_planted_bit_moves_the_answer() {
    // If one flipped bit in 200_000 branches does not change the fold, then the
    // fold is not reading what it claims to read and every agreement above is
    // worth nothing.
    //
    // The first version of this test flipped `d[0]`'s low bit and did not fire,
    // and the reason was not a kernel bug: `d[a]` is only read when column `a`
    // is in the solution `u`, so a defect on a column this `y` does not select
    // is silent BY CONSTRUCTION. That is LESSONS' "a planted defect must be
    // observable" arriving on schedule. So observability is now a CHECKED
    // PRECONDITION: the host twin has to see the plant move the rotation code
    // before the device is asked whether it sees the fold move. A plant the host
    // cannot see is skipped and counted, not silently passed.
    let f = folder();
    let n = 24;
    let descs = synth::batch(n, 16, 200_000, 0xF00D);
    let y = descs[7].point(0x9C3);
    let clean = GpuBatch::upload(&f, &descs).expect("upload").fold(&f, y, SHAPES[2]).expect("fold");
    assert_ne!(clean, Cyc::ZERO, "the clean fold was zero; nothing could move");

    let mut fired = 0usize;
    let mut attempts = 0usize;
    for &b in &[7usize, 999, 100_000, 199_999] {
        // Branch `b` must actually contribute at this `y`.
        if descs[b].rotation_code(y) == R_ZERO {
            continue;
        }
        let before = descs[b].rotation_code(y);
        // Find a `d` bit this `y` actually reads: column `a` is in the solution
        // iff perturbing `d[a]` moves the code.
        let plant = (0..descs[b].k).find(|&a| {
            let mut probe = descs[b].clone();
            probe.d[a >> 5] ^= 1u64 << ((a & 31) * 2);
            probe.rotation_code(y) != before
        });
        let Some(a) = plant else { continue };

        attempts += 1;
        let mut bent = descs.clone();
        bent[b].d[a >> 5] ^= 1u64 << ((a & 31) * 2);
        let got = GpuBatch::upload(&f, &bent).expect("upload").fold(&f, y, SHAPES[2]).expect("fold");
        if got != clean {
            fired += 1;
        }
    }
    assert!(attempts > 0, "no probed branch carried an observable plant; the probe was vacuous");
    assert_eq!(fired, attempts, "a planted, host-visible single-bit defect did not move the fold");
}

#[test]
fn a_planted_defect_in_the_kernels_own_reads_is_visible() {
    // The companion to the test above, on the three fields the plant above does
    // not touch. Each is perturbed on ONE branch, and each must move the fold —
    // this is what says the kernel reads R, h and J at all rather than getting
    // the right answer from the phase alone.
    let f = folder();
    let n = 20;
    let descs = synth::batch(n, 12, 50_000, 0x51DE);
    let y = descs[3].point(0x4D1);
    let clean = GpuBatch::upload(&f, &descs).expect("upload").fold(&f, y, SHAPES[1]).expect("fold");
    assert_ne!(clean, Cyc::ZERO);

    let bend = |label: &str, mutate: &dyn Fn(&mut AffineDesc)| {
        let mut bent = descs.clone();
        // Branch 3 is on the coset of `y` by construction, so it contributes.
        mutate(&mut bent[3]);
        let got = GpuBatch::upload(&f, &bent).expect("upload").fold(&f, y, SHAPES[1]).expect("fold");
        assert_ne!(got, clean, "planting in {label} did not move the fold");
    };

    // h: moves the coset, so branch 3 stops contributing (or contributes another
    // rotation) — visible either way.
    bend("h", &|d| d.h ^= 1);
    // base: the ring content itself.
    bend("base", &|d| d.base = Cyc { c: [d.base.c[0] + 1, d.base.c[1], d.base.c[2], d.base.c[3]], m: d.base.m });
    // J: the quadratic half of the phase. Only observable when the solution has
    // two set columns, so pick a pair the host confirms is read.
    let before = descs[3].rotation_code(y);
    let found = (0..descs[3].k).flat_map(|a| (a + 1..descs[3].k).map(move |b| (a, b))).find(|&(a, b)| {
        let mut probe = descs[3].clone();
        probe.j_rows[a] ^= 1u64 << b;
        probe.rotation_code(y) != before
    });
    match found {
        Some((a, b)) => bend("J", &move |d| d.j_rows[a] ^= 1u64 << b),
        None => panic!("no J entry was read at this y: the quadratic phase went untested"),
    }
}

#[test]
fn an_off_coset_state_folds_to_exact_zero_on_both_sides() {
    // The zero path, checked as a value and not inferred. Every branch shares
    // one coset here, so a `y` outside it must return ZERO from every arm.
    let f = folder();
    let n = 20;
    let mut descs = synth::batch(n, 8, 4096, 0x0FF0);
    // Force one common coset and one common R, so "off the coset" is well
    // defined for the whole batch at once.
    let (r0, h0) = (descs[0].r_rows.clone(), descs[0].h);
    for d in descs.iter_mut() {
        d.r_rows = r0.clone();
        d.h = h0;
    }
    // A row outside the column space of R: find a qubit that is not touched by
    // any column, and flip it. (With k = 8 and n = 20 there is always one.)
    let mut touched = 0u64;
    for (row, &r) in r0.iter().enumerate() {
        if r != 0 {
            touched |= 1 << row;
        }
    }
    let free = (!touched & ((1u64 << n) - 1)).trailing_zeros();
    let y = h0 ^ (1 << free);

    let batch = GpuBatch::upload(&f, &descs).expect("upload");
    assert_eq!(batch.fold(&f, y, SHAPES[0]).expect("fold"), Cyc::ZERO);
    assert_eq!(batch.fold(&f, y, SHAPES[4]).expect("fold"), Cyc::ZERO);
    assert_eq!(cpu::fold_packed(&descs, y, 1), Cyc::ZERO);
    // ... and the on-coset state does NOT, or the test above is vacuous.
    assert_ne!(batch.fold(&f, descs[0].point(0x2D), SHAPES[2]).expect("fold"), Cyc::ZERO);
}

#[test]
fn an_exactly_cancelling_batch_returns_zero_from_every_shape() {
    // The one crack `holon::mesh`'s header names: a partial sum that cancels to
    // exactly zero forgets the `m` it came from. Different shapes produce
    // different partial sums, so this is where two shapes could return equal
    // numbers in unequal structs. Build a batch that cancels ENTIRELY and check
    // that it does not.
    let f = folder();
    let n = 16;
    let half = synth::batch(n, 10, 8192, 0xCA11);
    let mut descs = half.clone();
    for d in half.iter() {
        // The same branch with a negated weight: contributes -amp(y) for every y.
        let mut neg = d.clone();
        neg.base = Cyc { c: [-d.base.c[0], -d.base.c[1], -d.base.c[2], -d.base.c[3]], m: d.base.m };
        descs.push(neg);
    }
    let batch = GpuBatch::upload(&f, &descs).expect("upload");
    for &s in &SHAPES {
        assert_eq!(batch.fold(&f, descs[3].point(0x2A), s).expect("fold"), Cyc::ZERO, "shape {s:?}");
    }
    assert_eq!(cpu::fold_packed(&descs, descs[3].point(0x2A), 4), Cyc::ZERO);
}

// ---------------------------------------------------------------- guards

#[test]
fn a_batch_that_could_overflow_is_refused_not_wrapped() {
    let f = folder();
    let n = 8;
    let mut descs = synth::batch(n, 4, 1024, 0xBAD0);
    for d in descs.iter_mut() {
        d.base = Cyc { c: [i128::MAX / 4, 0, 0, 1], m: 0 };
    }
    match GpuBatch::upload(&f, &descs) {
        Err(holon_gpu::GpuError::WouldOverflow { branches, .. }) => assert_eq!(branches, 1024),
        other => panic!("expected a refusal, got {other:?}", other = other.map(|b| b.b)),
    }
}

#[test]
fn a_block_size_the_shuffle_cannot_honour_is_refused() {
    let f = folder();
    let descs = synth::batch(8, 4, 64, 7);
    let batch = GpuBatch::upload(&f, &descs).expect("upload");
    for bad in [1u32, 31, 100, 2048] {
        assert!(
            batch.fold(&f, 0, Shape { block: bad, grid: 4 }).is_err(),
            "block {bad} should have been refused: the full shuffle mask assumes whole warps"
        );
    }
}

// ------------------------------------------------- the exponent-parity fence

#[test]
fn a_mixed_exponent_batch_is_value_equal_and_the_struct_question_is_measured() {
    // `gpu`'s module header states a fence: the GPU aligns the batch to
    // `M = max_b m_b`, while the sequential CPU fold's final `m` carries the
    // parity its own path arrived at, and `Cyc::normalize` removes only EVEN
    // powers of two — so a value has two normalized faces and the two folds
    // could agree as numbers and differ as structs.
    //
    // A fence stated and never tested is a hedge. This builds batches that
    // deliberately mix exponent PARITIES (where the risk lives; mixing by even
    // gaps cannot change parity) and reports what actually happens: value
    // equality is asserted, struct equality is COUNTED. If the count is ever
    // short of the total, that is the fence firing and it belongs in GPU.md,
    // not in an assertion that hides it.
    let f = folder();
    let n = 20;
    let mut struct_equal = 0usize;
    let mut trials = 0usize;

    for spread in [1i32, 3, 5] {
        let mut descs = synth::batch(n, 14, 20_000, 0x0DD_0000 + spread as u64);
        // Every third branch gets an ODD exponent gap, which is the only way the
        // parity of the batch maximum can disagree with the parity the CPU's
        // running accumulator reaches.
        for (i, d) in descs.iter_mut().enumerate() {
            if i % 3 == 0 {
                let lifted = holon_gpu::ring::align_to(d.base, d.base.m + spread);
                d.base = Cyc { c: lifted, m: d.base.m + spread };
            }
        }
        let batch = GpuBatch::upload(&f, &descs).expect("upload");
        assert!(!batch.exponent_uniform, "the point of this test is a mixed batch");

        for probe in 0..6u64 {
            let y = descs[probe as usize * 331].point(0x1B + probe * 5);
            let gpu = batch.fold(&f, y, SHAPES[2]).expect("fold");
            let cpu = cpu::fold_packed(&descs, y, 1);
            trials += 1;

            // VALUE equality, always: post the credit against the debit through
            // the ledger's own addition, which is faithful where the derived
            // PartialEq is not.
            let diff = holon::merge::MergeLedger::merge(
                gpu,
                Cyc { c: [-cpu.c[0], -cpu.c[1], -cpu.c[2], -cpu.c[3]], m: cpu.m },
            );
            assert!(
                diff.c.iter().all(|&v| v == 0),
                "spread {spread}, y = {y:#x}: GPU and CPU differ in VALUE, not just in face"
            );
            // ... and the canonical face, which is the documented remedy.
            assert_eq!(
                mesh::canonicalize(gpu),
                mesh::canonicalize(cpu),
                "spread {spread}, y = {y:#x}: even the canonical faces differ"
            );
            if gpu == cpu {
                struct_equal += 1;
            }
        }
        // Every shape must still agree with every other shape, mixed or not:
        // that is a property of the GPU reduction alone and the fence does not
        // touch it.
        let y = descs[7].point(0x2C);
        let first = batch.fold(&f, y, SHAPES[0]).expect("fold");
        for &s in &SHAPES {
            assert_eq!(batch.fold(&f, y, s).expect("fold"), first, "shape {s:?} on a mixed batch");
        }
    }

    println!(
        "MIXED-EXPONENT FENCE: struct-equal on {struct_equal} of {trials} probes \
         (value-equal on all {trials})"
    );
}
