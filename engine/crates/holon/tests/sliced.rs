//! THE BRANCH-SLICED EVALUATOR'S CERTIFICATE.
//!
//! `sliced` claims that 64 branches can share one affine state, and it claims
//! it EXACTLY: not "agrees to 1e-12", not "agrees as a complex number" — the
//! same `Cyc` struct, coefficient vector and denominator exponent alike. This
//! file is where that is made to pay, on four rungs of increasing distance
//! from the module's own assumptions:
//!
//! 1. **The slicing itself, bit for bit.** Every one of the 64 lanes must
//!    equal `sliced::scalar_branch_amplitude` for that branch — the per-branch
//!    path doing exactly what the sliced path claims to be doing 64 at a time.
//!    A discrepancy here is a slicing bug and nothing else, which is why this
//!    rung exists separately from the ones below.
//! 2. **The structural theorem.** `d_a mod 2` is branch-independent for every
//!    column, at the end of every block. The whole module rests on it, so it
//!    is checked rather than argued.
//! 3. **The production path.** `sliced::amplitude` must equal
//!    `run::amplitude` — which prunes, dedups, merges, and folds through the
//!    mesh — as an exact ring element, across shard counts.
//! 4. **The certified referee.** `holon_qasm::magic::magic_amplitude` (QASM-2,
//!    five of five against qiskit), the engine neither path is derived from.
//!
//! # The one place struct equality is NOT claimed, and why
//!
//! Rungs 1 and 2 are struct equality. Rung 3 is exact ring equality
//! (`affine::cyc_eq`) plus struct equality after `mesh::canonicalize`, and the
//! difference is deliberate: `Cyc::normalize` removes only EVEN powers of two,
//! so one value wears two normalised faces differing by a factor of `√2`, and
//! a partial sum that cancels to exactly zero forgets which. The pruned path
//! and the sliced path fold DIFFERENT partial sums by construction — one sums
//! merged branches, the other sums 64-blocks of raw branches — so they can
//! land on different faces of the same number. That boundary is not this
//! module's invention: `mesh.rs` documents it, exhibits it, and ships
//! `mesh::canonicalize` as its tested remedy. `sliced_and_pruned_agree`
//! REPORTS whether the raw structs happened to agree, so the claim stays a
//! measurement rather than a hope.

use holon::affine::cyc_eq;
use holon::ledger::Cyc;
use holon::mesh;
use holon::prune::Gate;
use holon::run;
use holon::sliced::{self, BranchBlockSource, SlicedConfig, LANES};
use holon_qasm::{Circuit, Gate as QGate};

const SHARDS: [usize; 5] = [1, 2, 3, 7, 16];

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

/// A LIVE random Clifford+T circuit. H is in the alphabet on purpose: it is
/// the gate that grows the column set and reaches `fold`, `dependent_subset`
/// and `gauss_sum_out` — the three places where the branches could diverge if
/// the structural theorem were false. A test that never runs an H would be
/// testing the easy half.
fn random_circuit(seed: u64, n: usize, depth: usize, t_cap: usize) -> Vec<Gate> {
    let mut rng = Rng(seed);
    let mut gates = Vec::with_capacity(depth);
    let mut t_used = 0usize;
    for _ in 0..depth {
        let q = rng.below(n);
        let mut q2 = rng.below(n);
        while n > 1 && q2 == q {
            q2 = rng.below(n);
        }
        let pick = rng.below(8);
        gates.push(match pick {
            0 => Gate::X(q),
            1 => Gate::Z(q),
            2 => Gate::H(q),
            3 => Gate::S(q),
            4 => Gate::Sdg(q),
            5 if n > 1 => Gate::Cx(q, q2),
            6 | 7 if t_used < t_cap => {
                t_used += 1;
                if pick == 6 {
                    Gate::T(q)
                } else {
                    Gate::Tdg(q)
                }
            }
            _ => Gate::H(q),
        });
    }
    while t_used < 3 && t_cap >= 3 {
        gates.push(Gate::T(rng.below(n)));
        t_used += 1;
    }
    gates
}

fn t_count(gates: &[Gate]) -> usize {
    gates.iter().filter(|g| g.is_t()).count()
}

fn basis(n: usize, idx: usize) -> Vec<bool> {
    (0..n).map(|q| idx >> q & 1 == 1).collect()
}

fn to_qasm(n: usize, gates: &[Gate]) -> Circuit {
    Circuit {
        n_qubits: n,
        n_clbits: n,
        gates: gates
            .iter()
            .map(|g| match *g {
                Gate::X(q) => QGate::X(q),
                Gate::Z(q) => QGate::Z(q),
                Gate::S(q) => QGate::S(q),
                Gate::Sdg(q) => QGate::Sdg(q),
                Gate::H(q) => QGate::H(q),
                Gate::Cx(c, t) => QGate::Cx(c, t),
                Gate::T(q) => QGate::T(q),
                Gate::Tdg(q) => QGate::Tdg(q),
            })
            .collect(),
        measures: Vec::new(),
    }
}

// ------------------------------------------------------------------- rung 1

/// EVERY LANE, BIT FOR BIT. 64 branches share one `R`, one `J`, one column
/// count; each carries its own `h` bit, its own `d` high bit, its own `γ`.
/// The claim is that the lane is the branch — so each lane is compared with a
/// scalar `Affine` run of that same branch under the same schedule, and the
/// comparison is `assert_eq!` on the ledger struct, not on a float.
#[test]
fn every_lane_is_its_branch_exactly() {
    let cfg = SlicedConfig::default();
    let cases: Vec<(u64, usize, usize, usize)> = vec![
        // (seed, qubits, depth, T-cap) — t below, at, and above one block.
        (1, 3, 20, 3),
        (2, 4, 26, 6),
        (3, 5, 34, 7),
        (4, 6, 40, 8),
        (5, 8, 44, 9),
        (6, 4, 24, 2),
        (7, 1, 12, 4),
    ];
    let mut lanes_checked = 0usize;
    for (seed, n, depth, t_cap) in cases {
        let gates = random_circuit(seed, n, depth, t_cap);
        let t = t_count(&gates);
        let sum = sliced::build(n, &gates, &cfg);
        let mut buf = vec![Cyc::ZERO; LANES];
        for idx in [0usize, 1, (1 << n) - 1] {
            let y = basis(n, idx);
            for b in 0..sum.n_blocks() {
                sum.block_amplitudes(b, &y, &mut buf);
                for (l, got) in buf.iter().enumerate() {
                    let branch = b * LANES as u64 + l as u64;
                    if branch >= 1u64 << t {
                        assert_eq!(
                            *got,
                            Cyc::ZERO,
                            "seed {seed}: lane {l} of block {b} is not a branch and \
                             must read exactly zero"
                        );
                        continue;
                    }
                    let want = sliced::scalar_branch_amplitude(n, &gates, branch, &y, &cfg);
                    assert_eq!(
                        *got, want,
                        "seed {seed}, y {idx}, branch {branch}: the lane is NOT the branch"
                    );
                    lanes_checked += 1;
                }
            }
        }
    }
    assert_eq!(
        lanes_checked, 2964,
        "the case matrix shrank without anyone saying so"
    );
}

// ------------------------------------------------------------------- rung 2

/// THE STRUCTURAL THEOREM, CHECKED. `d_a mod 2` is the same in all 64 lanes,
/// for every column, at the end of every block — because `fold` reads it as a
/// DECISION (`J_ab ^= d_a & 1`) and would silently make the branches disagree
/// about `J` if it were ever false.
#[test]
fn d_parity_is_lane_uniform_on_live_circuits() {
    let cfg = SlicedConfig::default();
    let mut blocks_checked = 0usize;
    for seed in 1..24u64 {
        let n = 3 + (seed as usize % 6);
        let gates = random_circuit(seed, n, 30 + n * 3, 7);
        let sum = sliced::build(n, &gates, &cfg);
        for b in &sum.blocks {
            assert!(
                b.state.parity_is_lane_uniform(),
                "seed {seed}: the branch-slicing theorem is false on a live circuit"
            );
            blocks_checked += 1;
        }
    }
    assert!(blocks_checked >= 24);
}

/// THE TWO PHASE SCHEDULES ARE ONE. Posting powers of `i` into a lane plane
/// and cashing them once must give the same STRUCT as paying each multiply
/// where the per-branch engine pays it. Rung 1 already tests the deferred
/// path against an undeferred scalar branch; this pins the two sliced
/// schedules directly, so a regression names itself rather than showing up as
/// a mysterious lane mismatch.
#[test]
fn deferred_and_immediate_phase_schedules_agree() {
    let defer = SlicedConfig { defer_phase: true, ..SlicedConfig::default() };
    let now = SlicedConfig { defer_phase: false, ..SlicedConfig::default() };
    let mut checked = 0usize;
    for seed in 51..60u64 {
        let n = 3 + (seed as usize % 5);
        let gates = random_circuit(seed, n, 28 + n * 3, 8);
        let a = sliced::build(n, &gates, &defer);
        let b = sliced::build(n, &gates, &now);
        let (mut ba, mut bb) = (vec![Cyc::ZERO; LANES], vec![Cyc::ZERO; LANES]);
        for idx in [0usize, 1, (1 << n) - 1] {
            let y = basis(n, idx);
            for blk in 0..a.n_blocks() {
                a.block_amplitudes(blk, &y, &mut ba);
                b.block_amplitudes(blk, &y, &mut bb);
                assert_eq!(ba, bb, "seed {seed}, y {idx}, block {blk}");
                checked += LANES;
            }
        }
    }
    assert!(checked >= 4000);
}

// ------------------------------------------------------------------- rung 3

/// THE PRODUCTION PATHS AGREE. `sliced::amplitude` (raw `2^t` branches, 64 to
/// a word) against `run::amplitude` (pruned, deduped, merged, mesh-folded),
/// at every shard count, as EXACT ring elements.
#[test]
fn sliced_and_pruned_agree() {
    let cases: Vec<(u64, usize, usize, usize)> = vec![
        (11, 3, 20, 4),
        (12, 4, 28, 6),
        (13, 5, 34, 7),
        (14, 6, 40, 9),
        (15, 8, 48, 10),
        (16, 12, 60, 11),
        (17, 4, 22, 2),
    ];
    let mut struct_equal = 0usize;
    let mut compared = 0usize;
    for (seed, n, depth, t_cap) in cases {
        let gates = random_circuit(seed, n, depth, t_cap);
        for idx in [0usize, 1, (1 << n.min(12)) - 1] {
            let y = basis(n, idx);
            let want = run::amplitude(n, &gates, &y);
            for &s in &SHARDS {
                let got = sliced::amplitude(n, &gates, &y, s);
                assert!(
                    cyc_eq(got, want),
                    "seed {seed}, y {idx}, shards {s}: sliced {got:?} vs pruned {want:?}"
                );
                assert_eq!(
                    mesh::canonicalize(got),
                    mesh::canonicalize(want),
                    "seed {seed}, y {idx}, shards {s}: the two paths disagree even after \
                     the ring's odd-√2 reduction — that is a VALUE difference, not a face"
                );
                compared += 1;
                if got == want {
                    struct_equal += 1;
                }
            }
        }
    }
    assert!(compared >= 100);
    // A measurement, not a claim: the two paths fold different partial sums,
    // so a raw struct difference would be the documented √2 face and not an
    // error. Recorded so a change in it is visible.
    println!(
        "sliced vs pruned: {struct_equal}/{compared} agreed as raw structs \
         (the rest, if any, are √2 faces of the same value)"
    );
}

/// Sharding is not an input. Same circuit, same `y`, five shardings of BOTH
/// tiers of the recursion (the block build and the block fold): one struct.
#[test]
fn sliced_fold_is_shard_invariant() {
    let gates = random_circuit(21, 6, 44, 9);
    let n = 6;
    for idx in [0usize, 5, 63] {
        let y = basis(n, idx);
        let first = sliced::amplitude(n, &gates, &y, 1);
        for &s in &SHARDS {
            assert_eq!(
                sliced::amplitude(n, &gates, &y, s),
                first,
                "sharding changed the LEDGER ENTRY at shards {s}"
            );
        }
        // And stable run to run at a fixed count — thread scheduling is not
        // an input either.
        for _ in 0..8 {
            assert_eq!(sliced::amplitude(n, &gates, &y, 7), first);
        }
    }
}

/// The whole state vector, not one lucky amplitude: every basis state of a
/// small register, sliced against pruned.
#[test]
fn whole_state_vector_agrees() {
    for (seed, n, depth, t_cap) in [(31u64, 4usize, 26usize, 7usize), (32, 5, 32, 8)] {
        let gates = random_circuit(seed, n, depth, t_cap);
        for idx in 0..(1usize << n) {
            let y = basis(n, idx);
            let got = sliced::amplitude(n, &gates, &y, 3);
            let want = run::amplitude(n, &gates, &y);
            assert!(cyc_eq(got, want), "seed {seed}, y {idx}: {got:?} vs {want:?}");
        }
    }
}

// ------------------------------------------------------------------- rung 4

/// THE CERTIFIED REFEREE. `holon_qasm::magic::magic_amplitude` is the QASM-2
/// record's engine (five of five against qiskit) and is not an ancestor of
/// either path under test. It answers in floating point, so this rung is a
/// complex-number agreement by construction — which is exactly why it is the
/// LAST rung and not the only one.
#[test]
fn sliced_matches_the_certified_reference() {
    let mut checked = 0usize;
    for (seed, n, depth, t_cap) in [
        (41u64, 3usize, 20usize, 5usize),
        (42, 4, 28, 7),
        (43, 5, 34, 8),
        (44, 6, 38, 9),
    ] {
        let gates = random_circuit(seed, n, depth, t_cap);
        let circ = to_qasm(n, &gates);
        for idx in [0usize, 1, (1 << n) - 1] {
            let y = basis(n, idx);
            let (re, im) = sliced::amplitude(n, &gates, &y, 4).to_complex();
            let (rre, rim) = holon_qasm::magic::magic_amplitude(&circ, &y, false, false);
            assert!(
                (re - rre).abs() < 1e-12 && (im - rim).abs() < 1e-12,
                "seed {seed}, y {idx}: sliced ({re}, {im}) vs reference ({rre}, {rim})"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 12);
}

// ------------------------------------------------------------------- fences

/// A circuit with no T-gates is one branch in one block, and 63 lanes that do
/// not exist. They must read exactly zero rather than a plausible number.
#[test]
fn a_clifford_circuit_is_one_branch_in_one_block() {
    let gates: Vec<Gate> = vec![Gate::H(0), Gate::Cx(0, 1), Gate::S(1), Gate::H(1)];
    let sum = sliced::build(2, &gates, &SlicedConfig::default());
    assert_eq!(sum.n_blocks(), 1);
    let y = basis(2, 0);
    let mut buf = vec![Cyc::ZERO; LANES];
    sum.block_amplitudes(0, &y, &mut buf);
    for (l, v) in buf.iter().enumerate().skip(1) {
        assert_eq!(*v, Cyc::ZERO, "lane {l} is not a branch and must read zero");
    }
    assert!(cyc_eq(
        sliced::amplitude(2, &gates, &y, 3),
        run::amplitude(2, &gates, &y)
    ));
}

/// The lane masks partition the block: T-site `j` splits the 64 branch indices
/// on bit `j`, exactly, and the block index supplies every site from 6 up.
#[test]
fn t_site_masks_are_the_branch_index_bits() {
    for block in [0u64, 1, 2, 5, 37] {
        for site in 0..12usize {
            let m = sliced::t_site_mask(site, block);
            for lane in 0..64u64 {
                let branch = block * 64 + lane;
                let want = (branch >> site) & 1 == 1;
                assert_eq!(
                    (m >> lane) & 1 == 1,
                    want,
                    "site {site}, block {block}, lane {lane}"
                );
            }
        }
    }
    assert_eq!(sliced::block_count(0), 1);
    assert_eq!(sliced::block_count(6), 1);
    assert_eq!(sliced::block_count(12), 64);
    assert_eq!(sliced::active_lanes(0), 1);
    assert_eq!(sliced::active_lanes(5), 0xFFFF_FFFF);
    assert_eq!(sliced::active_lanes(6), u64::MAX);
    assert_eq!(sliced::active_lanes(20), u64::MAX);
}

/// Growth is refused, never silent — the same fence `PruneConfig` holds on
/// the per-branch axis, moved to the axis this path grows along.
#[test]
#[should_panic(expected = "would exceed max_blocks")]
fn block_growth_is_refused() {
    let gates: Vec<Gate> = (0..14).map(Gate::T).collect();
    let cfg = SlicedConfig { max_blocks: 4, ..SlicedConfig::default() };
    let _ = sliced::build(14, &gates, &cfg);
}

/// The larger register the brief names, at the T-count it names: n = 64 with
/// t = 12 is 4096 branches in 64 blocks, and the sliced path must still be
/// the pruned path's answer. One `y`, because `run::amplitude` is the
/// expensive side of this comparison, not the sliced one.
#[test]
fn wide_register_agrees() {
    let n = 64;
    let gates = random_circuit(101, n, 200, 12);
    assert_eq!(t_count(&gates), 12);
    let y = basis(n, 0);
    let want = run::amplitude(n, &gates, &y);
    for &s in &[1usize, 4] {
        let got = sliced::amplitude(n, &gates, &y, s);
        assert!(cyc_eq(got, want), "n = 64, shards {s}: {got:?} vs {want:?}");
    }
}
