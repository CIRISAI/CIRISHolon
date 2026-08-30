//! **The plant set (RESOURCE_DESIGN §8, D13): every firing rule demonstrates a failing case.**
//!
//! A rule that has never fired has never been demonstrated to gate. Same discipline as G1's split
//! mutation pair, and for the same reason — a gate asserted only in the direction that passes is
//! satisfied by an implementation that does nothing.
//!
//! **Every plant asserts its CARRIER non-empty before it is scored** (M-PLANT-SECTOR). That is
//! not ceremony: forgetting it cost the sibling campaign two false alarms in one afternoon — a
//! wrong-warm-start plant on a 9-determinant space where the trap could not occur, and again on
//! (O,O,O) where the wrong start found the right eigenvector and the probe reported the guard as
//! having failed. A plant on an empty sector VOIDs rather than passes.

use holon_resource::lease::LeaseError;
use holon_resource::{
    Arena, LeaseState, ProbeVerdict, ReapVerdict, Reaper, Receipt, ResourceKind, ScriptedProbe,
    ScriptedWorld, MAX_DEPTH,
};

/// **PLANT D7 — a request past the recursion cap VOIDs, WITH THE CHAIN.**
///
/// "Too deep" without the path tells you a rule fired and nothing about what fired it, which is
/// why the chain is part of the requirement rather than a nicety.
#[test]
fn plant_d7_depth_five_voids_and_names_the_chain() {
    let mut a = Arena::new();
    let mut p = ScriptedProbe::always_pass();

    // THE CARRIER: the cap must be reachable at all. Build the full legal ladder first — if
    // depth 4 could not be leased, "depth 5 is refused" would prove nothing.
    let mut parent = None;
    let mut chain = Vec::new();
    for d in 0..MAX_DEPTH {
        let id = a
            .lease(&mut p, parent, ResourceKind::Worker, 1)
            .unwrap_or_else(|e| panic!("the legal ladder failed at depth {d}: {}", e.message()));
        assert_eq!(a.get(id).unwrap().depth, d);
        chain.push(id);
        parent = Some(id);
    }
    assert_eq!(chain.len(), MAX_DEPTH as usize, "the carrier ladder is short");

    // THE PLANT: one rung further.
    let err = a
        .lease(&mut p, parent, ResourceKind::Worker, 1)
        .expect_err("a depth-5 lease was GRANTED; the cap is not a cap");
    match &err {
        LeaseError::DepthExceeded { cap, chain: got } => {
            assert_eq!(*cap, MAX_DEPTH);
            // The chain must actually name the ancestors, not merely exist.
            for id in &chain {
                assert!(
                    got.contains(id),
                    "the VOID message omitted ancestor {id}; the chain is decorative"
                );
            }
            let m = err.message();
            assert!(m.contains("VOID") && m.contains("->"), "{m}");
        }
        other => panic!("wrong refusal: {other:?}"),
    }

    // The refusal left no entry: a refused lease is not an opened one.
    assert_eq!(a.ledger().opened, MAX_DEPTH as u64);
    assert!(a.balances());
}

/// **PLANT D9 — a convicted child SURFACES in the parent's ledger rather than vanishing.**
#[test]
fn plant_d9_a_convicted_child_surfaces_in_the_parents_books() {
    let mut a = Arena::new();
    let mut p = ScriptedProbe::always_pass();

    let root = a.lease(&mut p, None, ResourceKind::Ram, 1024).unwrap();
    let child = a.lease(&mut p, Some(root), ResourceKind::Ram, 512).unwrap();
    let grandchild = a.lease(&mut p, Some(child), ResourceKind::Ram, 256).unwrap();

    // THE CARRIER: the child really was leased and really is live, so there is something to lose.
    assert_eq!(a.ledger().opened, 3);
    assert_eq!(a.live_count(), 3);
    assert_eq!(a.get(child).unwrap().state, LeaseState::Leased);

    // THE PLANT: convict the middle lease. Its own child goes with it — it was held THROUGH it.
    a.convict(child, "the resource went away underneath it").unwrap();

    assert_eq!(a.get(child).unwrap().state, LeaseState::Convicted);
    assert_eq!(
        a.get(grandchild).unwrap().state,
        LeaseState::Convicted,
        "a grandchild survived its parent's conviction; it was held through a lease that no \
         longer exists"
    );
    assert_eq!(
        a.ledger().convicted,
        2,
        "the conviction did not reach the books — this is exactly the vanishing D9 forbids"
    );

    // The evidence is recorded, not just the count.
    let convictions = a.convictions();
    assert_eq!(convictions.len(), 2);
    assert!(convictions.iter().any(|(id, why)| *id == child
        && why.contains("resource went away")));

    // And the identity still holds: 3 opened = 0 released + 2 convicted + 1 live.
    assert_eq!(a.live_count(), 1);
    assert!(a.balances(), "ledger {:?}", a.ledger());

    a.release(root).unwrap();
    assert!(a.balances());
    assert_eq!(a.live_count(), 0);
}

/// **PLANT D10 — a reaper facing a STALLED MACHINE stands down and reclaims nothing.**
///
/// The founding case: with the root filesystem at 100%, every writer on this box was blocked and
/// none was progressing. A timeout-based reaper would have destroyed the machine's entire
/// workload while correctly observing that nothing was advancing.
///
/// Both halves are required. Without the control below, "stands down always" would pass.
#[test]
fn plant_d10_a_stalled_machine_makes_the_reaper_stand_down() {
    let mut a = Arena::new();
    let mut p = ScriptedProbe::always_pass();
    let id = a.lease(&mut p, None, ResourceKind::Disk, 1).unwrap();

    // THE CARRIER: rungs 1 and 2 both point at conviction. If they did not, rung 3 would never
    // be consulted and this plant would be testing nothing.
    let world = ScriptedWorld {
        grace_expired: true,
        holder_scheduling: false,
    };

    // THE PLANT: the reaper's OWN attempt at the same operation class fails — the machine.
    let mut reaper = Reaper::new(world, ScriptedProbe::always_fail("disk full"));
    let verdict = reaper.sweep_one(&mut a, id, ResourceKind::Disk);

    match &verdict {
        ReapVerdict::StandDown { evidence, .. } => {
            assert!(evidence.grace_expired && !evidence.holder_scheduling,
                "the carrier is empty: rungs 1 and 2 did not point at conviction");
            assert!(!evidence.reaper_own_probe.passed());
        }
        other => panic!(
            "the reaper did not stand down on a stalled machine: {other:?}. On the 2026-08-30 \
             disk-full window this verdict would have reclaimed every holder on the box."
        ),
    }
    // Nothing was reclaimed.
    assert_eq!(a.get(id).unwrap().state, LeaseState::Leased);
    assert_eq!(a.ledger().convicted, 0);
    assert_eq!(a.ledger().reaped, 0);
    assert!(verdict.message(id).contains("STOOD DOWN"));

    // THE CONTROL: identical holder, HEALTHY machine. Now it must reap, or the rule above is
    // just "never reap" wearing a reason.
    let mut b = Arena::new();
    let mut p2 = ScriptedProbe::always_pass();
    let id2 = b.lease(&mut p2, None, ResourceKind::Disk, 1).unwrap();
    let mut healthy = Reaper::new(
        ScriptedWorld {
            grace_expired: true,
            holder_scheduling: false,
        },
        ScriptedProbe::always_pass(),
    );
    let v2 = healthy.sweep_one(&mut b, id2, ResourceKind::Disk);
    assert!(
        v2.reaped(),
        "the reaper did not reap a genuinely idle holder on a healthy machine: {v2:?} — the \
         stand-down rule has swallowed the mechanism it was guarding"
    );
    assert_eq!(b.get(id2).unwrap().state, LeaseState::Convicted);
    assert_eq!(b.ledger().reaped, 1);
    assert!(b.balances());
}

/// The reaper keeps a holder that is merely SLOW — past its grace period but still scheduling.
/// This is the distinction D10 is named for, and it must not collapse into the stand-down case.
#[test]
fn a_slow_holder_is_kept_and_the_reason_is_recorded() {
    let mut a = Arena::new();
    let mut p = ScriptedProbe::always_pass();
    let id = a.lease(&mut p, None, ResourceKind::Worker, 1).unwrap();

    let mut reaper = Reaper::new(
        ScriptedWorld {
            grace_expired: true,
            holder_scheduling: true, // slow, not idle
        },
        ScriptedProbe::always_pass(),
    );
    let v = reaper.sweep_one(&mut a, id, ResourceKind::Worker);
    assert!(!v.reaped());
    assert!(v.message(id).contains("KEPT"));
    assert!(matches!(v, ReapVerdict::Keep { .. }));
    assert_eq!(a.get(id).unwrap().state, LeaseState::Leased);
}

/// **D4 — a failed probe REFUSES and leaves no entry.** A refusal is normal and cheap; what it
/// must never be is a silent fallback that opens a lease anyway.
#[test]
fn a_failed_probe_refuses_and_opens_nothing() {
    let mut a = Arena::new();
    let mut p = ScriptedProbe::always_fail("no VRAM on this host");

    let err = a
        .lease(&mut p, None, ResourceKind::Vram, 512)
        .expect_err("a lease was granted over a failing probe");
    assert!(matches!(err, LeaseError::Refused { .. }));
    assert!(err.message().contains("no VRAM on this host"));

    assert_eq!(a.ledger().opened, 0, "a refused lease was recorded as opened");
    assert_eq!(a.live_count(), 0);
    assert!(a.balances());
}

/// Receipts are the rent (§9 Q1): a lease refreshes by recording REAL WORK, and a holder that
/// produces nothing is Idle whatever else it is doing.
#[test]
fn receipts_are_the_rent_and_an_empty_receipt_is_not_payment() {
    let mut a = Arena::new();
    let mut p = ScriptedProbe::always_pass();
    let id = a.lease(&mut p, None, ResourceKind::Worker, 1).unwrap();

    a.pay_rent(id, Receipt(32)).unwrap();
    assert_eq!(a.get(id).unwrap().state, LeaseState::Active);
    assert_eq!(a.get(id).unwrap().rent, Receipt(32));

    // A zero receipt is a heartbeat with no work product. It is not rent.
    a.pay_rent(id, Receipt::ZERO).unwrap();
    assert_eq!(
        a.get(id).unwrap().state,
        LeaseState::Idle,
        "a holder producing nothing stayed Active; a heartbeat with no work product was accepted \
         as rent"
    );
    assert_eq!(a.get(id).unwrap().rent, Receipt(32), "the total moved on an empty receipt");
    assert_eq!(a.ledger().rent, Receipt(32));
}

/// Release is leaf-to-root, and the whole subtree is accounted.
#[test]
fn release_takes_the_children_first_and_the_books_balance() {
    let mut a = Arena::new();
    let mut p = ScriptedProbe::always_pass();
    let root = a.lease(&mut p, None, ResourceKind::Ram, 8).unwrap();
    let c1 = a.lease(&mut p, Some(root), ResourceKind::Ram, 4).unwrap();
    let c2 = a.lease(&mut p, Some(root), ResourceKind::Ram, 4).unwrap();
    let g = a.lease(&mut p, Some(c1), ResourceKind::Ram, 2).unwrap();

    assert_eq!(a.ledger().opened, 4);
    let released = a.release(root).unwrap();
    assert_eq!(released, 4, "the whole subtree was not released");
    for id in [root, c1, c2, g] {
        assert_eq!(a.get(id).unwrap().state, LeaseState::Released);
    }
    assert!(a.balances());
    assert_eq!(a.live_count(), 0);

    // Acting on an ended lease is an error, not a silent no-op.
    assert!(matches!(
        a.release(root),
        Err(LeaseError::AlreadyEnded(_, LeaseState::Released))
    ));
}

/// A probe that passes must actually have been consulted — M-VACUOUS-SUCCESS at this layer.
#[test]
fn the_probe_is_consulted_once_per_lease() {
    let mut a = Arena::new();
    let mut p = ScriptedProbe::always_pass();
    for _ in 0..5 {
        a.lease(&mut p, None, ResourceKind::Ram, 1).unwrap();
    }
    assert_eq!(p.calls, 5, "leases were granted without probing");

    // And a depth refusal short-circuits BEFORE the probe, so a cap violation costs nothing.
    let mut b = Arena::new();
    let mut q = ScriptedProbe::always_pass();
    let mut parent = None;
    for _ in 0..MAX_DEPTH {
        parent = Some(b.lease(&mut q, parent, ResourceKind::Ram, 1).unwrap());
    }
    let before = q.calls;
    let _ = b.lease(&mut q, parent, ResourceKind::Ram, 1);
    assert_eq!(q.calls, before, "the cap check ran a probe it did not need");
}

/// ProbeVerdict carries WHAT WAS CHECKED into the lease, because that record is the part a lease
/// guarantees forever (D3(4)).
#[test]
fn a_lease_records_what_admitted_it() {
    let mut a = Arena::new();
    let mut p = ScriptedProbe {
        answers: vec![ProbeVerdict::Pass("wrote and removed a byte")],
        calls: 0,
        default: ProbeVerdict::Fail("exhausted"),
    };
    let id = a.lease(&mut p, None, ResourceKind::Disk, 1).unwrap();
    assert_eq!(a.get(id).unwrap().admitted_on, "wrote and removed a byte");
}
