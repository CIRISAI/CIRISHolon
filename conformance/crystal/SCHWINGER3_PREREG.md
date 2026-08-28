# Pre-registration — SCHWINGER-3: the per-column grid

*Frozen 2026-08-28, committed ALONE. Successor to SCHWINGER-2 (VOID by its
own N-convergence premise), whose failure cause is now established by a
CONFIRMED forward prediction: the fixed-N grid under-resolves large x, and
the finite-volume standard N ≳ 20√x — recorded in our own prior-art sweep
from Bañuls–Cichy–Jansen–Cirac — must set N per column. This campaign is
the Ω ladder's rung-6 entry: an interacting field theory pass with only
boundary data changed.*

misfits: contacts M-VOLUME-SCALE (the campaign exists to discharge it: N
is staked PER COLUMN as ceil(20√x), ceil(28√x), ceil(36√x) rounded to
even), M-NULL-MISSTAKE (every convergence premise is staked on the
quantity the limit actually controls, per column), M-ONE-MODEL-DELTA (the
verdict compares to the continuum value, not to a fitted model),
M-STALE-INSTRUMENT (the runner, prereg and results are committed
together; checkpoints carry the record), M-PLANT-OBS and M-PLANT-SECTOR
(the instrument's two-sided gauge is inherited from SCHWINGER-2's
certification — ED plants matched to 5e-14, planted MPO mutation fired at
0.214; each of those plants' carriers was asserted nonzero in the sector
the plant acts on when certified, and that certification is cited here,
not re-run and not assumed silently), M-FIXED-POINT-TRAJECTORY and
M-NONBIJECTIVE-STEP and M-FINAL-VIEW-COLLISIONS (no trajectory or
bijectivity claim arises in a variational ground-state campaign),
M-PROBE-EIGENSTATE (DMRG initial states are random; no probe carrier),
M-GAUGE-LAUNDER and M-LOOP-BLIND and M-BARE-CHARGE and M-COND-PROBE and
M-ELECTRIC-BASIS and M-RING-MIXING and M-GAUGE-UNIFORM-MOMENTUM and
M-HOMOG and M-KINEMATIC-NONLOCAL (not contacted: no gauge-observable,
locality or channel claim is staked).

## Grid (per-column N, the discharge of M-VOLUME-SCALE)

| x | N (= ceil(k√x) even, k = 20, 28, 36) | χ |
|---|---|---|
| 4.0 | 40, 56, 72 | 40, 64 |
| 9.0 | 60, 84, 108 | 40, 64 |
| 16.0 | 80, 112, 144 | 40, 64 |

18 points, checkpointed per point, resumable; the amended SCHWINGER-2
schedule and runner machinery carry over unchanged except for the grid.

## Frozen premises and gates (unchanged in FORM from SCHWINGER-2)

- **χ-premise, per point** (EXACT band): |M(χ=64) − M(χ=40)| ≤ 1e-3, else
  that point VOIDs. witness: none (measured premise; χ was saturated at
  every SCHWINGER-2 point and is expected non-binding)
- **N-premise, per column**: |M(N₃) − M(N₂)| < 0.01 at χ=64, else that
  column VOIDs. witness: none (measured premise)
- **S1 — the physics gate**: with all three columns posable, extrapolate
  M/g in 1/N per column (three points now, not two), then in 1/√x across
  columns; the vector-mass verdict is **M_V/g within 1/√π ± 0.05**.
  Branch (a) inside the band ⇒ the crystal referee reproduces the
  continuum Schwinger vector mass. Branch (b) outside ⇒ the claim dies;
  report the extrapolated value and both fit residuals. Fewer than 3
  posable columns ⇒ VOID, never a pass by shrinkage.
  witness: none (the continuum value is Schwinger's, cited in
  SCHWINGER-2's freeze; no Lean object covers it)
- **Monotone-gap reading** (recorded, not a gate): the per-column
  N-convergence gaps should now all sit BELOW 0.01 by construction; their
  ordering is reported as a check on the volume standard itself.

## Meaning

S1(a) ⇒ the rung-6 entry stands: the same exact-first machinery that
passed the gravity sequence reproduces an interacting field theory's
continuum observable, with only boundary data (the grid) changed. S1(b)
⇒ the referee fails on physics and the failure is kept. VOID ⇒ the volume
standard itself needs re-examination, which the monotone-gap reading will
localize.
