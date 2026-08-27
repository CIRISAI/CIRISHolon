//! MUTATION TESTS.
//!
//! A gate that cannot fail proves nothing. Every test here deliberately corrupts the
//! exchange in one named way and demands that the [`ConservationGate`] catches it. The
//! test name says what it breaks.
//!
//! Two things are asserted for each mutation:
//!
//! 1. the honest run of the same configuration is clean (so the test is not passing
//!    because the configuration is broken to begin with);
//! 2. the corrupted run returns a *typed error*, never a panic, and — where the mutation
//!    is designed to isolate one leg — that error names the specific leg.
//!
//! Because the gate returns on its first failing leg, and legs run in the fixed order
//! L1 -> (L4, L6, L7 per shard) -> (L2, L3 per pair) -> L5, an error naming a late leg is
//! positive evidence that every earlier leg *passed* on that mutation. That is how the
//! "only L3 can see a symmetric corruption" claim is established rather than asserted.

use holon_swarm::error::Side;
use holon_swarm::exchange::resolve_side;
use holon_swarm::gate::{leg2_pair_antisymmetry, leg3_plan_conformance};
use holon_swarm::{FaultInjection, GateLevel, LedgerDelta, Swarm, SwarmError, SwarmSpec};

const SHARDS: usize = 4;
const LEAVES: usize = 32;
const PAIR: usize = 0;
const ROUNDS: u64 = 3;

fn swarm(level: GateLevel) -> Swarm {
    Swarm::new(&SwarmSpec::ring(SHARDS, LEAVES).with_gate(level)).expect("swarm builds")
}

/// Run the fault sequentially and return the gate's verdict.
fn run_seq(fault: FaultInjection, level: GateLevel) -> Result<(), SwarmError> {
    swarm(level).run_rounds_sequential(ROUNDS, fault)
}

/// Run the fault on the threaded path and return the gate's verdict. Every mutation is
/// checked on BOTH paths: a gate that only fires in the sequential reference would be
/// worthless for the design this crate exists to de-risk.
fn run_par(fault: FaultInjection, level: GateLevel) -> Result<(), SwarmError> {
    swarm(level).run_rounds_threaded(ROUNDS, 4, fault)
}

/// Every mutation must be caught on both paths and at every gate level that is supposed
/// to see it. `expect` names the leg the mutation is designed to isolate.
fn assert_caught_on_both_paths(fault: FaultInjection, level: GateLevel) -> (SwarmError, SwarmError) {
    let seq = run_seq(fault, level).expect_err("sequential gate must catch this mutation");
    let par = run_par(fault, level).expect_err("threaded gate must catch this mutation");
    (seq, par)
}

// ---------------------------------------------------------------- the control

#[test]
fn control_the_honest_exchange_passes_every_gate_level_on_both_paths() {
    for level in [GateLevel::Ledger, GateLevel::Full, GateLevel::Paranoid] {
        run_seq(FaultInjection::None, level).expect("honest sequential run must pass");
        run_par(FaultInjection::None, level).expect("honest threaded run must pass");
    }
}

#[test]
fn control_the_exchange_actually_moves_ledger_quantity() {
    // If no quantity ever crossed a boundary, every mutation test below would be vacuous.
    let mut s = swarm(GateLevel::Full);
    let opening: Vec<_> = s.shards().iter().map(|sh| sh.g0()).collect();
    s.run_rounds_sequential(ROUNDS, FaultInjection::None).unwrap();
    let closing = s.shard_ledgers();
    assert_ne!(
        opening, closing,
        "no shard's balance changed: the exchange transferred nothing"
    );
    // ... and the global is untouched, exactly.
    let total_open = opening
        .iter()
        .fold(ciris_sim_core::regplus::GrossState::ZERO, |a, b| a.combine(*b));
    assert_eq!(s.global_ledger().unwrap(), total_open);
}

// -------------------------------------------------- mutation 1: credit without debit

#[test]
fn mutation_credit_without_a_matching_debit_is_caught() {
    let (seq, par) = assert_caught_on_both_paths(
        FaultInjection::CreditWithoutDebit { pair: PAIR },
        GateLevel::Full,
    );
    for err in [&seq, &par] {
        assert!(
            matches!(
                err,
                SwarmError::ApplyInconsistent { .. } | SwarmError::GlobalLedgerNotConserved { .. }
            ),
            "expected L4 or L5, got {err}"
        );
    }
    // The cheapest gate level sees it too: this mutation moves the global sum.
    assert!(run_seq(
        FaultInjection::CreditWithoutDebit { pair: PAIR },
        GateLevel::Ledger
    )
    .is_err());
}

// -------------------------------------------------- mutation 2: transfer applied twice

#[test]
fn mutation_transfer_applied_twice_on_one_side_is_caught() {
    let (seq, par) = assert_caught_on_both_paths(
        FaultInjection::DoubleApplyOneSide { pair: PAIR },
        GateLevel::Full,
    );
    for err in [&seq, &par] {
        assert!(
            matches!(
                err,
                SwarmError::ApplyInconsistent { .. } | SwarmError::GlobalLedgerNotConserved { .. }
            ),
            "expected L4 or L5, got {err}"
        );
    }
}

#[test]
fn mutation_transfer_applied_twice_on_both_sides_is_caught_only_by_plan_conformance() {
    let (seq, par) = assert_caught_on_both_paths(
        FaultInjection::DoubleApplyBothSides { pair: PAIR },
        GateLevel::Paranoid,
    );
    // L3 firing means L1, L4, L6, L7 and L2 all PASSED: the corruption is perfectly
    // conservative and perfectly antisymmetric. Only re-planning from the snapshot sees it.
    for err in [&seq, &par] {
        assert!(
            matches!(err, SwarmError::ReceiptDoesNotMatchPlan { .. }),
            "expected L3 (plan conformance) to be the ONLY leg that fires, got {err}"
        );
    }
}

/// Direct evidence for the claim above, at the level of the legs themselves: a gate built
/// only from "the global sum is unchanged" — the obvious design — passes this corruption.
#[test]
fn mutation_symmetric_double_apply_defeats_a_global_sum_only_gate() {
    let plan = LedgerDelta {
        constituents: 1,
        occupancy: 6,
        momentum: [-3, 4],
    };
    let fault = FaultInjection::DoubleApplyBothSides { pair: 0 };
    let lo = resolve_side(plan, 0, Side::Lo, fault).unwrap();
    let hi = resolve_side(plan, 0, Side::Hi, fault).unwrap();

    // Conserved: the pair still nets to zero in every lane (a global-sum gate passes).
    assert_eq!(
        lo.applied.checked_add(hi.applied).unwrap(),
        LedgerDelta::ZERO
    );
    // L2 passes: the receipts are still exact negatives.
    assert!(leg2_pair_antisymmetry(0, 0, lo.receipted, hi.receipted).is_ok());
    // L4 would pass: each side wrote exactly what it receipted.
    assert_eq!(lo.applied, lo.receipted);
    assert_eq!(hi.applied, hi.receipted);
    // L3 is the only leg that can tell.
    assert!(leg3_plan_conformance(0, 0, Side::Hi, plan, hi.receipted).is_err());
}

// -------------------------------------------------- mutation 3: transfer dropped

#[test]
fn mutation_dropped_transfer_is_caught_only_by_plan_conformance() {
    let (seq, par) =
        assert_caught_on_both_paths(FaultInjection::DropTransfer { pair: PAIR }, GateLevel::Paranoid);
    for err in [&seq, &par] {
        assert!(
            matches!(err, SwarmError::ReceiptDoesNotMatchPlan { .. }),
            "expected L3 (plan conformance), got {err}"
        );
    }
}

/// A dropped or doubled transfer is only detectable if the transfer was non-zero, and the
/// suite only tests the lanes that actually cross a boundary. Pinned so those tests cannot
/// silently become vacuous if the seeding changes.
#[test]
fn control_every_ledger_lane_actually_crosses_the_boundary_under_test() {
    use holon_swarm::plan_transfer;
    let mut s = swarm(GateLevel::Full);
    let pair = s.pairs()[PAIR];

    // Opening imbalance on the pair under test.
    let lanes = |sw: &Swarm, endpoint: (usize, u32)| {
        holon_swarm::ledger::gross_to_lanes(sw.shards()[endpoint.0].ledger()[endpoint.1 as usize])
            .unwrap()
    };
    let plan = plan_transfer(lanes(&s, pair.lo), lanes(&s, pair.hi)).unwrap();
    assert!(!plan.is_zero(), "opening transfer for pair {PAIR} is zero");

    // Every lane must move at least once over the rounds the mutation tests run.
    let mut moved = [false; 4];
    for _ in 0..ROUNDS {
        s.run_rounds_sequential(1, FaultInjection::None).unwrap();
        for shard in s.shards() {
            for receipt in shard.receipts() {
                for (lane, seen) in receipt.to_lanes().iter().zip(moved.iter_mut()) {
                    *seen |= *lane != 0;
                }
            }
        }
    }
    assert_eq!(
        moved,
        [true; 4],
        "some ledger lane never crosses a boundary: {moved:?} \
         (order: constituents, occupancy, momentum0, momentum1)"
    );
}

// -------------------------------------------------- mutation 4: sign swapped

#[test]
fn mutation_swapped_transfer_sign_is_caught() {
    let (seq, par) = assert_caught_on_both_paths(
        FaultInjection::SwapSignOnLowSide { pair: PAIR },
        GateLevel::Full,
    );
    for err in [&seq, &par] {
        assert!(
            matches!(
                err,
                SwarmError::PairNotAntisymmetric { .. }
                    | SwarmError::GlobalLedgerNotConserved { .. }
                    | SwarmError::ReceiptDoesNotMatchPlan { .. }
                    | SwarmError::LedgerOverflow { .. }
            ),
            "expected L2/L3/L5 or a checked-arithmetic rejection, got {err}"
        );
    }
}

/// Leg-level evidence for the pairing leg, independent of the data the swarm happens to
/// carry: a one-sided sign swap makes both sides credit, and L2 is what sees that.
#[test]
fn mutation_swapped_sign_is_seen_by_the_pairing_leg_itself() {
    let plan = LedgerDelta {
        constituents: 2,
        occupancy: 6,
        momentum: [-3, 4],
    };
    let fault = FaultInjection::SwapSignOnLowSide { pair: 0 };
    let lo = resolve_side(plan, 0, Side::Lo, fault).unwrap();
    let hi = resolve_side(plan, 0, Side::Hi, fault).unwrap();
    assert_eq!(lo.receipted, hi.receipted, "both sides now credit");
    assert!(leg2_pair_antisymmetry(0, 0, lo.receipted, hi.receipted).is_err());
    // And the pair no longer nets to zero, so the global sum moves too.
    assert_ne!(
        lo.applied.checked_add(hi.applied).unwrap(),
        LedgerDelta::ZERO
    );
}

// -------------------------------------------------- mutation 5: accumulator overflow

#[test]
fn mutation_overflowed_momentum_accumulator_is_caught_and_does_not_panic() {
    let (seq, par) = assert_caught_on_both_paths(
        FaultInjection::OverflowMomentum { pair: PAIR },
        GateLevel::Full,
    );
    for err in [&seq, &par] {
        assert!(
            matches!(err, SwarmError::LedgerOverflow { .. }),
            "expected a checked-arithmetic rejection, got {err}"
        );
    }
}

// -------------------------------------------------- mutation 6: minted in the local step

#[test]
fn mutation_local_step_minting_from_nothing_is_caught() {
    let (seq, par) =
        assert_caught_on_both_paths(FaultInjection::MintInLocalStep { shard: 1 }, GateLevel::Full);
    for err in [&seq, &par] {
        assert!(
            matches!(err, SwarmError::LocalStepNotConserving { .. }),
            "expected L1, got {err}"
        );
    }
    // The mint keeps internal composition intact, so the composition leg is blind to it —
    // L1 is what earns its keep here.
    assert!(run_seq(
        FaultInjection::MintInLocalStep { shard: 1 },
        GateLevel::Ledger
    )
    .is_err());
}

// -------------------------------------------------- mutation 7: composition broken

#[test]
fn mutation_child_credited_without_its_parent_is_caught_by_the_composition_leg() {
    let (seq, par) =
        assert_caught_on_both_paths(FaultInjection::BreakComposition { shard: 2 }, GateLevel::Full);
    for err in [&seq, &par] {
        assert!(
            matches!(err, SwarmError::CompositionBroken { .. }),
            "expected L6, got {err}"
        );
    }
}

/// **REPORTED GATE WEAKNESS, pinned as a test.**
///
/// The ledger-only gate level does NOT catch a broken composition, and neither does the
/// structural form of L7 (`RuntimeArena::validate()`), because the exchange writes the
/// live ledger overlay while `validate()` reads the arena's stored headers. This is not
/// hidden: it is the reason `GateLevel::Full` and `GateLevel::Paranoid` exist, and the
/// reason `Shard::revalidate_through_core` exists.
#[test]
fn weakness_a_ledger_only_gate_is_blind_to_a_broken_composition() {
    let escaped = run_seq(
        FaultInjection::BreakComposition { shard: 2 },
        GateLevel::Ledger,
    );
    assert!(
        escaped.is_ok(),
        "documented weakness changed: GateLevel::Ledger now catches BreakComposition"
    );
    // And it is caught the moment the composition leg is switched on.
    assert!(run_seq(
        FaultInjection::BreakComposition { shard: 2 },
        GateLevel::Full
    )
    .is_err());
}

/// The paranoid level puts `ciris-sim-core`'s OWN validator over the live overlay, by
/// rebuilding the arena from it. Same mutation, core's own verdict.
#[test]
fn mutation_broken_composition_is_rejected_by_the_cores_own_validator() {
    let mut s = swarm(GateLevel::Paranoid);
    // One clean round first, so the rebuild is exercised on a moved ledger, not the
    // opening balance.
    s.run_rounds_sequential(1, FaultInjection::None).unwrap();
    let shard = &s.shards()[2];
    shard.revalidate_through_core(0).expect("clean shard rebuilds");

    let err = s
        .run_rounds_sequential(1, FaultInjection::BreakComposition { shard: 2 })
        .unwrap_err();
    assert!(
        matches!(
            err,
            SwarmError::CompositionBroken { .. } | SwarmError::ShardStructureInvalid { .. }
        ),
        "expected L6 or L7-paranoid, got {err}"
    );
}

// -------------------------------------------------- mutation 8: root credited alone

#[test]
fn mutation_root_credited_without_any_child_is_caught() {
    let (seq, par) =
        assert_caught_on_both_paths(FaultInjection::RootOnlyCredit { shard: 3 }, GateLevel::Full);
    for err in [&seq, &par] {
        assert!(
            matches!(
                err,
                SwarmError::ApplyInconsistent { .. }
                    | SwarmError::CompositionBroken { .. }
                    | SwarmError::GlobalLedgerNotConserved { .. }
            ),
            "expected L4, L6 or L5, got {err}"
        );
    }
    // This one moves the root, so even the cheapest level catches it.
    assert!(run_seq(
        FaultInjection::RootOnlyCredit { shard: 3 },
        GateLevel::Ledger
    )
    .is_err());
}

// -------------------------------------------------- the leg-by-leg matrix

/// Prints which leg fires first for every mutation at every gate level. Not an assertion
/// about a single leg — it is the evidence behind the matrix reported for this build, and
/// it fails if any mutation escapes every level.
#[test]
fn every_mutation_is_caught_at_the_full_gate_level() {
    let faults = [
        ("credit-without-debit", FaultInjection::CreditWithoutDebit { pair: PAIR }),
        ("double-apply-one-side", FaultInjection::DoubleApplyOneSide { pair: PAIR }),
        ("double-apply-both-sides", FaultInjection::DoubleApplyBothSides { pair: PAIR }),
        ("drop-transfer", FaultInjection::DropTransfer { pair: PAIR }),
        ("swap-sign", FaultInjection::SwapSignOnLowSide { pair: PAIR }),
        ("overflow-momentum", FaultInjection::OverflowMomentum { pair: PAIR }),
        ("mint-in-local-step", FaultInjection::MintInLocalStep { shard: 1 }),
        ("break-composition", FaultInjection::BreakComposition { shard: 2 }),
        ("root-only-credit", FaultInjection::RootOnlyCredit { shard: 3 }),
    ];
    let mut escaped = Vec::new();
    for (name, fault) in faults {
        for level in [GateLevel::Ledger, GateLevel::Full, GateLevel::Paranoid] {
            let seq = run_seq(fault, level);
            let par = run_par(fault, level);
            println!(
                "{name:24} {level:?}\n    seq: {}\n    par: {}",
                seq.as_ref().err().map(|e| e.to_string()).unwrap_or_else(|| "ESCAPED".into()),
                par.as_ref().err().map(|e| e.to_string()).unwrap_or_else(|| "ESCAPED".into()),
            );
            if level != GateLevel::Ledger && (seq.is_ok() || par.is_ok()) {
                escaped.push((name, level, seq.is_ok(), par.is_ok()));
            }
        }
    }
    assert!(
        escaped.is_empty(),
        "mutations escaped the gate: {escaped:?}"
    );
}
