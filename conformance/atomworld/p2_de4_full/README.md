# P2 four-body arm at FULL STRENGTH — the water result, banked

*Banked 2026-09-01, five of eight seeds complete; the remaining three
(0x…21, 0x…25, 0x…27) are appended on completion. Logs copied byte-identical
from the launch directory; each carries its own provenance header (commit
21e6be3, binary sha256 7790cf3d…, build exit 0, gate battery 4/4, loadavg
and clock at both ends) and its exit line.*

## What this run is

The first run of the exact four-body physics at design strength: the
(O,H,H,H) term evaluated by `quaternary::ohhh_fci_grad` — nine seeded dual
solves per recompute giving the exact Cartesian gradient, momentum zero by
construction, MBE3 subtracted from the same curves the other sectors serve —
under the gate battery landed at 21e6be3 (momentum exact, dE4-off control,
torque, force-is-the-gradient). Same seeds, box, quench protocol and
checkpoint discipline as the two prior arms. Not a reproduction of either:
the banked broken arm (../p2_de4_seeds/) ran these seeds with the four-body
force ~1837-29165x too weak under a fired momentum gate, and the fenced arm
ran no four-body term at all.

## The result

**Seed 0x53415422 made water under legitimate physics:**

    modal-O OH2   free O 0   molecules [H2 H2 OH2 O3H2]   dE4_evals 891
    worst drift / bound = 2.713e-5    worst |p| / bound = 4.670e-5

The three-arm comparison on this seed gives the four-body term a causal
role in water formation:

| arm | physics | seed 22's outcome |
|---|---|---|
| fenced (MBE3 only) | pair + three-body, conservation-clean | **OH** — hydroxyl, no water |
| broken four-body | term ~3-4 orders too weak, momentum gate FIRED | OH2, under a fired gate |
| **this arm** | exact gradient, full strength, all gates green | **OH2** — water, clean |

MBE3 alone stops at hydroxyl; the exact four-body correction carries it to
water. The broken arm's observation is vindicated in substance while its
physics stays voided.

## The conservation repair, confirmed in production

Every completed seed's momentum sits at 1e-5-scale of its roundoff bound —
the fenced arm's class, ten orders below the broken arm:

| seed | census | |p| / bound |
|---|---|---|
| 0x53415422 | 2xH2, OH2, O3H2 — **water** | 4.670e-5 |
| 0x53415423 | 3xH2, O3H2, one FREE O | 8.947e-5 |
| 0x53415424 | 3xH2, O4H2 | 5.170e-5 |
| 0x53415426 | 3xH2, O4H2 | 3.500e-5 |
| 0x53415428 | 3xH2, O4H2 | 5.180e-5 |

Beyond seed 22, the term's signature across seeds is anti-saturation: it
strips hydrogens off the large oxygen clusters MBE3 permitted (O4H4 →
O4H2 + H2 twice) and on one seed liberated the programme's first-ever free
oxygen atom.

## What this is NOT yet

A formula census at the final frame is not a closure certification. The
certification rung — is the OH2 a THING, held over the pre-staked window,
leg A and leg B — is the closure census's, running against this seed's
regenerated trajectory with the fenced arm as the one-variable baseline.
The stance's `water-holon` claim strengthens only when that instrument
says so.
