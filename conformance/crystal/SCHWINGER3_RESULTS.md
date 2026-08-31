# SCHWINGER-3 — adjudicated: S1 BRANCH (a), the crystal referee HOLDS

*2026-08-31. Prereg frozen in this directory (`SCHWINGER3_PREREG.md`);
instrument at CIRISOntology `scratchpad/crystal/dmrg_schwinger.py`, whose
running bytes were banked mid-flight at ac072ce (amendment A1's warm-start
χ-ladder and adaptive tolerance, plus the eigsh rung repair, all pre-data
or instrument-schedule-only per the amendment text). Run log:
`scratchpad/crystal/schwinger3_run.log`. Zero VOIDs.*

## The verdict

**M_V/g = 0.553116, against the staked continuum target 1/√π = 0.564190
± 0.05 — inside the band (|Δ| = 0.0111, 2.0% relative, 22% of the band's
half-width). S1 branch (a).**

Per the freeze's own meaning clause: the rung-6 entry stands — the same
exact-first machinery that carries the engine's tiers reproduces a
continuum quantum-field observable, the Schwinger model's vector-meson
mass, with only boundary data (the grid) changed. **Mass from the vacuum
is banked.** Stance consequence: W4 ("the crystal referee will hold")
stops being a wager; the crystal referee is now measured, and its kill
retires fired-empty.

## The grid, complete (18/18)

| x | N ladder | M/g at χ=64 | M_∞(x) (N→∞) | fit δ |
|---|---|---|---|---|
| 4.0 | 40, 56, 72 | 0.689405 → 0.681667 → 0.678386 | 0.664187 | 3.3e-3 |
| 9.0 | 60, 84, 108 | 0.655253 → 0.646645 → 0.643017 | 0.627240 | 3.6e-3 |
| 16.0 | 80, 112, 144 | 0.638043 → 0.628995 → 0.625193 | 0.608620 | 3.8e-3 |

Continuum extrapolation over the three M_∞(x) → 0.553116.

**χ-premise: PASSED at every point, with two orders of margin.** The
staked band was |M(χ=64) − M(χ=40)| ≤ 1e-3; the worst measured
difference on the grid is 6e-6 (the final point, N=144), and most points
agree to 1e-6 or better. The bond-dimension ladder was never the
limiting error; the N→∞ and continuum extrapolations carry the budget,
and their fit residuals are reported above per the freeze.

## Cost and incident history, kept

The final point (x=16, N=144, χ=64) alone ran ~22 hours under fleet
load — consistent with the measured loaded-box inflation the resource
campaign registered the same weekend (M-IDLE-CALIBRATED-TIMEOUT), and
bounded throughout by A1's sweep cap. Earlier in the campaign the run
died at checkpoint 17 on an ARPACK non-convergence (the error is kept at
the top of the run log as history); the repair — a retry ladder with
raised maxiter/ncv at tightened tolerance — is part of the banked
instrument, and the run resumed from checkpoints without loss. The
checkpoint-resume discipline (detached compute, DONE markers) is what
made a multi-day, multi-incident grid completable; the [checkpoint] tags
in the log mark replayed points.

## What this banks, and its fence

Banked: rung 6's entry — a gauge-coupled matter observable computed by
the exact-first stack agreeing with the continuum analytic value
(Schwinger 1962) at 2%, under frozen gates, with the χ-budget
demonstrated slack and the extrapolation residuals printed. The crystal
tier's referee for "the engine's lattice machinery reads real field
theory" is now measured, not wagered.

Fence, unchanged from the freeze: this is QED₂ — one spatial dimension,
one flavour, the model's own continuum limit as the referee. It licenses
the MACHINERY (exact-first DMRG on gauge-coupled chains reproducing
continuum physics), not any claim about 3+1D. The geometry↔matter
coupling that T6 of the water tasking waits on cites THIS result as its
prerequisite bank, and inherits exactly this fence.
