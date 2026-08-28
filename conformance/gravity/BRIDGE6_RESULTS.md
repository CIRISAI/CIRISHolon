# BRIDGE-6 — VOID, but three gates passed and the dressed charge is EARNED

*2026-08-28. Prereg 97d2958 (frozen with M14's fix pre-verified), instrument
committed after it, log `bridge6_run.log`. Verdict: **VOID** — R2 and R3
fired and plant (ii) missed. Kept, marked, no rescue.*

## What PASSED, on both graphs — and this is real progress

- **G0 — the invariance gate.** The Wilson-dressed projector commutes with
  the joint Gauss action at every vertex, and survives the pump. M14's fix
  is confirmed in the instrument, not just in the pre-freeze diagnostic:
  **the charged sector is now genuinely gauge-invariant.** BRIDGE-5 could
  not get past this point.
- **R1 — ENDOGENOUS.** The matter channel reading MOVES under iterating
  `step` alone, with `G_p` a term of T and no operator applied by hand.
  The reviewer's third overclaim ("a conditioned probe, not endogenous
  dynamics") is **discharged**: this is the same joint evolution doing both
  jobs.
- **B3.** Joint Gauss holds on every registry state at every step.

## What fired, and the honest split: two construction defects, one physics

A diagnostic settles all three, and it refuted my first hypothesis (I
expected the physical space to BE the dressed channel):

```
physical nonzero ≈ 982,016   OFF-channel nonzero ≈ 860,160   (6/6 trials)
```

The off-channel sector is **large**, so channel-conditioning is not
vacuous. Therefore:

1. **R2 fired on a CARRIER DEFECT, not a physics limit.** My off-channel
   state was built by seeding one matter component and Gauss-projecting,
   which happens to annihilate. The correct construction is to take a
   generic physical state and project OUT the dressed channel — which the
   diagnostic shows is abundant. Cheap fix, no design change.
2. **Plant (ii) missed for the SAME reason** — its carrier was the pure
   dressed state, whose off-channel component is zero, so the pump it
   plants literally never acts. That is misfit M8 for the FOURTH time
   (observability is instrument-relative), and it VOIDs the campaign
   independently. The lesson is now mechanical, not advisory: a plant's
   carrier must be checked nonzero IN THE SECTOR THE PLANT ACTS ON, not
   merely nonzero overall. BRIDGE-6 asserted the latter and it was not
   enough.
3. **R3 is a genuine finding, and the one worth keeping.** near == far
   exactly (9007199254 on base, 7378697629 on refined): the spokes-only
   electric term did NOT break the parity homogeneity as designed. The
   response moves (base → near differs, so geometry does act on matter) but
   a DISTANT plaquette gives the identical response, so locality is still
   symmetric rather than spatial. **The homogeneity is more robust than one
   term choice** — a real property of this fan-disk model, and the
   successor must break it structurally (a larger graph where plaquettes
   have genuinely different holonomy content, not merely a different
   electric support).

## Standing after six campaigns

Earned so far, on frozen gates: exact dynamical curvature (BRIDGE-2 ω),
the closure failure and its collision theorem, geometry→matter response
(BRIDGE-3), gauge-invariant Wilson-dressed charge (G0), and endogenous
reciprocity (R1). Still owed: **charged sourcing posed on a live
off-channel carrier** (a fix, not a redesign) and **spatial locality**,
which BRIDGE-6 shows needs a structurally inhomogeneous graph rather than a
term substitution.
