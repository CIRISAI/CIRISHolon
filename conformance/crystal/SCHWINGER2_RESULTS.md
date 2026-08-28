# SCHWINGER-2 — VOID by its own frozen premise, and the cause was in our own sweep

*2026-08-28, called at checkpoint 12/18 rather than at the end, because the
verdict is already determined by the freeze.*

## The call

The prereg stakes, per x column: N-convergence |M(N=64) - M(N=48)| < 0.01,
else that x VOIDs; and fewer than 3 posable x => the campaign VOIDs (never
a pass by shrinkage).

| x | chi-premise | N-premise | posable? | M/g (N->inf) |
|---|---|---|---|---|
| 4.0 | PASS (6 digits) | PASS (0.00489) | yes | 0.660724 |
| 9.0 | PASS (8 digits) | FAIL (0.01169 > 0.01) | VOID | 0.608460 |
| 16.0 | - | - | (running) | - |

With x=9 VOID, at most two columns can be posable, so the campaign VOIDs
regardless of x=16. Called now; the record does not wait for arithmetic
that cannot change its own verdict.

## The cause, and it was avoidable

The grid stakes N in {32,48,64} for EVERY x. But x is the inverse lattice
spacing: at fixed N a larger x means a SMALLER physical volume, so the
finite-volume requirement grows with x. The standard is N >~ 20*sqrt(x),
and it is not obscure - it is in OUR OWN prior-art sweep, recorded from
Banuls-Cichy-Jansen-Cirac's scope on the day the crystal ladder was staked.

| x | required N | grid provides |
|---|---|---|
| 4 | 40 | 64 - comfortable, and it passed |
| 9 | 60 | 64 - marginal, and it failed |
| 16 | 80 | 64 - UNDER-RESOLVED BEFORE IT RAN |

So the x=9 failure is neither noise nor physics: the grid was
under-specified in a way our own sweep had already flagged, and I did not
apply it when freezing. x=16 was doomed at freeze time.

## What survives

The x=4.0 column is fully posable and stands: chi-saturated to 7e-10, clean
1/N scaling, M/g(N->inf) = 0.6607 at that spacing. The instrument is
certified two-sided (ED plants match to 5e-14; the planted MPO mutation
fires at 0.214) and the amended checkpointing schedule works. Nothing about
the DMRG is impugned - only the grid.

## The successor

SCHWINGER-3, frozen fresh: N scaled PER COLUMN as N >~ 20*sqrt(x) (roughly
{40,56,72} at x=4, {60,80,100} at x=9, {80,104,128} at x=16); chi unchanged,
since it was never the binding constraint - saturated at every point
measured; same S1 band and kill. The checkpoint machinery carries over, so
recomputation is the only cost.
