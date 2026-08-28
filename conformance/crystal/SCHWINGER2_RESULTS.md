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

## A forward prediction, staked at checkpoint 13/18 — before the data

The diagnosis above ("the grid's fixed N under-resolves large x, and the
requirement is N ≳ 20√x") makes a testable claim about the column now
running, and the run is being allowed to finish for exactly that reason.

**Staked now, before x=16's N-convergence is computed:** since x=16 needs
N ≳ 80 and the grid provides only 64, its N-convergence gap must be
**WORSE than x=9's 0.01169** — the under-resolution is monotone in x.

- Gap > 0.01169 → the diagnosis is confirmed by forward prediction (house
  rule 6: support comes only from confirmed advance predictions), and
  SCHWINGER-3's per-column N is warranted rather than merely plausible.
- Gap ≤ 0.01169 → the diagnosis is WRONG or incomplete, the x=9 failure
  needs another explanation, and SCHWINGER-3 must not be frozen on this
  reasoning. That would be the more interesting outcome and it is recorded
  as such in advance.

Either way the campaign's verdict is unchanged: SCHWINGER-2 is VOID. This
prediction adjudicates the CAUSE, not the campaign.

## The forward prediction: CONFIRMED at 18/18

Staked at checkpoint 13, before the data: x=16's N-convergence gap must be
WORSE than x=9's 0.01169, because the grid's fixed N under-resolves large
x (N ≳ 20√x). Measured at 18/18: **0.02096 > 0.01169 — CONFIRMED**, and
the runner's own frozen adjudication returns VOID (fewer than 3 posable
x), matching the early call. Per house rule 6, the under-resolution
diagnosis is now SUPPORTED by a confirmed advance prediction, and
SCHWINGER-3's per-column N grid is warranted rather than merely plausible.
The gap ordering across the grid — 0.0049 (x=4, resolved), 0.0117 (x=9,
marginal), 0.0210 (x=16, under-resolved) — is monotone in x exactly as
the finite-volume standard requires.
