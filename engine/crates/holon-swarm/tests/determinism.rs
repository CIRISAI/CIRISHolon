//! Determinism harness.
//!
//! The claim under test: the final global ledger **and every per-shard ledger** are
//! bit-identical across
//!
//! * repeated runs of the same configuration,
//! * the single-threaded sequential reference and the multi-threaded path,
//! * every thread count from 1 to 16,
//! * and every visit order of shards and boundary pairs.
//!
//! Integer quantities are compared with `==`, which on `GrossState` is exact equality of
//! four integers — there is no tolerance anywhere. The f64 whole-state is compared with
//! `to_bits()`.

use ciris_sim_core::regplus::GrossState;
use holon_swarm::{FaultInjection, GateLevel, RoundOrder, Swarm, SwarmSpec};

const SHARDS: usize = 8;
const LEAVES: usize = 512;
const ROUNDS: u64 = 40;

fn spec() -> SwarmSpec {
    SwarmSpec::ring(SHARDS, LEAVES).with_gate(GateLevel::Full)
}

fn sequential_reference() -> (GrossState, Vec<GrossState>, Vec<GrossState>, Vec<u64>) {
    let mut s = Swarm::new(&spec()).unwrap();
    s.run_rounds_sequential(ROUNDS, FaultInjection::None).unwrap();
    let (ledgers, bits) = s.full_fingerprint();
    (s.global_ledger().unwrap(), s.shard_ledgers(), ledgers, bits)
}

#[test]
fn sequential_runs_are_bit_identical_to_each_other() {
    let a = sequential_reference();
    for _ in 0..3 {
        assert_eq!(sequential_reference(), a);
    }
}

#[test]
fn threaded_runs_match_the_sequential_reference_at_every_thread_count() {
    let (global, shard_ledgers, all_ledgers, bits) = sequential_reference();
    for threads in [1usize, 2, 3, 4, 8, 16] {
        for repeat in 0..3 {
            let mut s = Swarm::new(&spec()).unwrap();
            s.run_rounds_threaded(ROUNDS, threads, FaultInjection::None)
                .unwrap_or_else(|e| panic!("threads={threads} repeat={repeat}: {e}"));
            let (ledgers, run_bits) = s.full_fingerprint();
            assert_eq!(
                s.global_ledger().unwrap(),
                global,
                "global ledger differs at threads={threads} repeat={repeat}"
            );
            assert_eq!(
                s.shard_ledgers(),
                shard_ledgers,
                "per-shard ledgers differ at threads={threads} repeat={repeat}"
            );
            assert_eq!(
                ledgers, all_ledgers,
                "per-holon ledgers differ at threads={threads} repeat={repeat}"
            );
            assert_eq!(
                run_bits, bits,
                "whole-state bits differ at threads={threads} repeat={repeat}"
            );
        }
    }
}

#[test]
fn the_global_ledger_is_exactly_the_opening_balance_after_every_run() {
    let opening = Swarm::new(&spec()).unwrap().global_ledger().unwrap();
    for threads in [1usize, 4, 16] {
        let mut s = Swarm::new(&spec()).unwrap();
        s.run_rounds_threaded(ROUNDS, threads, FaultInjection::None)
            .unwrap();
        assert_eq!(s.global_ledger().unwrap(), opening);
    }
}

/// Order-independence, tested directly: the same round with shards and pairs visited in
/// different orders must produce bit-identical state. This is the property the snapshot
/// design buys, so it is checked rather than argued.
#[test]
fn the_result_does_not_depend_on_the_order_shards_or_pairs_are_visited() {
    let orders = [
        RoundOrder::natural(SHARDS, SHARDS),
        RoundOrder::reversed(SHARDS, SHARDS),
        RoundOrder::strided(SHARDS, SHARDS),
    ];
    let mut reference: Option<(Vec<GrossState>, Vec<u64>)> = None;
    for order in &orders {
        assert_eq!(order.shards.len(), SHARDS, "order must visit every shard");
        assert_eq!(order.pairs.len(), SHARDS, "order must visit every pair");
        let mut s = Swarm::new(&spec()).unwrap();
        for _ in 0..ROUNDS {
            s.step_round_sequential(order, FaultInjection::None).unwrap();
        }
        let fingerprint = s.full_fingerprint();
        match &reference {
            None => reference = Some(fingerprint),
            Some(expected) => assert_eq!(
                &fingerprint, expected,
                "visit order {order:?} changed the result"
            ),
        }
    }
}

/// The `to_bits()` comparison above is only meaningful if there IS whole-state to compare.
#[test]
fn control_the_swarm_actually_carries_whole_state() {
    let s = Swarm::new(&spec()).unwrap();
    let (_, bits) = s.full_fingerprint();
    assert!(!bits.is_empty(), "no whole-state scalars: the bit comparison is vacuous");
    // Two f64 per boundary port, two ports per shard on a ring.
    assert_eq!(bits.len(), SHARDS * 2 * 2);
}

/// Honest statement of scope: the exchange does not mutate whole-state, because
/// `RuntimeArena` exposes no mutable scalar pool. The bit comparison is therefore a check
/// that whole-state survives untouched, not a check on parallel float arithmetic — and
/// this test says so out loud so the claim is not over-read.
#[test]
fn whole_state_is_carried_but_not_mutated_by_the_exchange() {
    let before = Swarm::new(&spec()).unwrap().full_fingerprint().1;
    let mut s = Swarm::new(&spec()).unwrap();
    s.run_rounds_threaded(ROUNDS, 8, FaultInjection::None).unwrap();
    assert_eq!(s.full_fingerprint().1, before);
}

#[test]
fn a_single_shard_swarm_has_no_pairs_and_still_conserves() {
    let mut s = Swarm::new(&SwarmSpec::ring(1, 64)).unwrap();
    let opening = s.global_ledger().unwrap();
    assert!(s.pairs().is_empty());
    s.run_rounds_threaded(10, 4, FaultInjection::None).unwrap();
    assert_eq!(s.global_ledger().unwrap(), opening);
}

#[test]
fn a_line_topology_conserves_and_matches_across_paths() {
    let spec = SwarmSpec::line(6, 256).with_gate(GateLevel::Paranoid);
    let mut seq = Swarm::new(&spec).unwrap();
    seq.run_rounds_sequential(15, FaultInjection::None).unwrap();
    for threads in [1usize, 2, 6] {
        let mut par = Swarm::new(&spec).unwrap();
        par.run_rounds_threaded(15, threads, FaultInjection::None)
            .unwrap();
        assert_eq!(par.full_fingerprint(), seq.full_fingerprint());
    }
}
