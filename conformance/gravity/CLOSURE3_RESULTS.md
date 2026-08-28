# CLOSURE-3 — ALL GATES PASS: the missing memory is second-order

*2026-08-28. Prereg admitted first-pass; instrument and results committed
together; plants both fire (the pair-sensitivity plant's constructed states
are one-body-equal and pair-distinct, so the channel's discriminating power
is demonstrated, not assumed).*

```
K1=PASS (the three firing collisions reproduce)
K2=BRANCH(a): v_pair separates EVERY firing pair — memory is second-order
K3=PASS (inheritance exact)   G0/B3=PASS   plants both FIRE
```

## The channel ladder, complete on this model

| channel | content | verdict |
|---|---|---|
| v_conf | one-body Wilson triples | defect > 0 (CLOSURE-2B, exact) |
| v_conf + 't Hooft | the full one-body conjugate pair | STILL blind (CLOSURE-2B, validated) |
| **+ pair correlators** | all second-order Wilson data | **closes every measured collision** |

**The coarse classical bookkeeping that closes this model is (one-body +
pair).** The memory the configuration channel discards is genuinely not
phase-space data (2B's finding stands) — but it is SHALLOW: second-order
correlations suffice. The whole-pattern reading recorded in 2B is
therefore BOUNDED by this measurement: on this model, at these collisions,
nothing beyond pair order is needed. Whether richer models (non-abelian,
punctured, larger complexes) push the memory deeper is now a measurable
question with an instrument that has answered it once.

## Requirement 3, toy-scale summary as it now stands

Closure derives coarse dynamics where it holds (ClosureDerives' theorems;
ADM-1C's universal descent); where it fails, the failure is exactly
computable (the collision defect) and the minimal restoring memory is
LOCATED (second-order, here). The ADM instantiation and
deriving-the-dynamics-from-closure remain the named open remainder.


## Re-adjudication after external re-review: the second-order verdict was an OVERCALL

Confirmed by direct enumeration: **v_pair has its OWN firing collision at
(1, 7)** — equal pair-views, different successors. The frozen K2 checked
that v_pair separates the COARSE view's firing collisions (it does) but
never enumerated the refined view's own collisions, which is what "the
pair channel restores closure" actually requires. Registered as
M-FINAL-VIEW-COLLISIONS. **Corrected verdict: the missing memory is AT
LEAST THIRD-ORDER on this model.** CLOSURE-2B's whole-pattern reading,
which CLOSURE-3 claimed to bound, is UN-bounded: the ladder stands at
one-body fails → one-body+pair fails → open upward. The ladder theorems
(ClosureLadder.lean) are unaffected — they never asserted pair
sufficiency, only the logic of separation; the overcall was the
instrument's gate, and the process lesson is the reviewer's phrase,
adopted: a green process gate must not hide a scientific overcall.
witness: collision_refutes_memoryless (the (1,7) fire is itself an
instance). misfits: contacts M-FINAL-VIEW-COLLISIONS (registered here),
M-ONE-MODEL-DELTA, M-PLANT-OBS, M-PLANT-SECTOR, M-LOOP-BLIND (as before).
