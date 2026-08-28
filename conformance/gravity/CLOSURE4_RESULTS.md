# CLOSURE-4 — first run: branch (a) provisional, plant (i) construction failed

*2026-08-28. Q2 read BRANCH(a) (v_triple closed on the trajectory) but
plant (i) MISSED: the inclusion–exclusion superposition intended to be
pair-equal is not (its cross-terms do not cancel at pair order), so the
instrument's triple-sensitivity was not demonstrated and the campaign is
VOID pending the rerun. The reading is recorded provisional, not banked.*

## Amendment CLOSURE-4B (frozen here, before the rerun)

Plant (i) is replaced by the conviction the DATA already provides: the
corrected CLOSURE-3 measured that v_pair COLLIDES at trajectory points
(1, 7). The plant therefore uses x₁ and x₇ themselves as its carrier pair:
- assert (sector, per M-PLANT-SECTOR): v_pair(x₁) = v_pair(x₇) — the
  measured pair-equality, re-verified in-run, the sector the plant acts on
  being their triple-channel difference, asserted nonzero by the gate
  itself;
- the plant FIRES iff v_triple(x₁) ≠ v_triple(x₇) — the triple channel
  separating the exact collision the pair channel could not.
This is strictly stronger than the constructed-state form: the
sensitivity is demonstrated on the very states the verdict depends on.
Plant (ii) unchanged. All gates unchanged.
misfits: contacts M-PLANT-SECTOR and M-PLANT-OBS (the amendment's
substance), M-FINAL-VIEW-COLLISIONS (the collision pair is the carrier),
and the admitted freeze's full contact list otherwise unchanged.
witness: collision_refutes_memoryless (the (1,7) pair-fire that supplies
the carrier). M-ONE-MODEL-DELTA: contacted via the collision/minimax
vocabulary; the defect form is unchanged from the admitted freeze.

## CLOSURE-4B run — branch (a) on a validated instrument

```
Q2=BRANCH(a): v_triple closed on the trajectory — memory EXACTLY third-order
K1/K3/G0/B3=PASS
[plant i] FIRES (the measured (1,7) collision: pair-equal, triple-separated)
[plant ii] FIRES
```

**The memory ladder, complete and validated at every rung:**

| order | channel | verdict |
|---|---|---|
| 1 | Wilson triples | fires (three collisions) |
| 1 | + 't Hooft (full conjugate pair) | still fires |
| 2 | + all pair correlators | still fires — at (1, 7) |
| **3** | **+ all triple correlators** | **closed, plants firing** |

The plant is the strong form: the very pair of trajectory states the pair
channel could not tell apart is separated by the triple channel — the
sensitivity demonstrated on the states the verdict depends on. **On this
model the classical bookkeeping that closes the coarse view is exactly
third-order Wilson data**: beyond phase-space, beyond pairs, and bounded
at triples. The whole-pattern direction is again BOUNDED (at this size) —
one rung higher than CLOSURE-3 wrongly claimed, by an instrument that now
enumerates its own view's collisions as a matter of structure.
