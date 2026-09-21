# RUNG 2 — AMENDMENT 6: a second leg against a NAMED law — continuity — beside the collision form

*Written 2026-09-18, committed alone, BEFORE the leg is built and BEFORE any trajectory is
read under it. Adds one leg to `RUNG2_PREREG.md` §3.4; changes nothing in the collision form,
its budget, its controls or its branches. The collision form stays the primary and this leg
is reported beside it, never instead of it.*

## Why a named law, and why the freeze avoided one

The freeze graded closure by the collision form alone, deliberately: a defect against one
chosen model earns only "worse than that model" (`M-ONE-MODEL-DELTA`), while the collision
form earns "best memoryless" — which is what `Closed` needs, since `h` is quantified
existentially. That neutrality is bought with samples: `≥ 200` repeated readings at the
element's own time, `~250` windows, and at 432 waters on one core that is a week.

For a fluid element the chosen model is not a choice. The tier's law IS the continuum
equations, and the first of them — continuity — has **no constitutive input at all**: mass
is conserved exactly, so the density of a cell can change only by momentum crossing its
faces. A chart whose momentum field explains its own density change is closed under
continuity; one whose momentum field explains it no better than a scrambled field is not.
This leg asks that, in ~10–20 windows instead of 250.

## The leg

For every window `k` and cell `c` of a chart read under Amendment 5 (fields averaged over the
window `τ`, three momentum components on a 3D grid):

> **predicted change**   `Δn_c^pred = −(τ / m̄) · Σ_faces ( p̄_face · n̂ ) / a_face`, with the
> face momentum the arithmetic mean of the two adjacent cells' window-averaged momentum
> density (the standard finite-volume closure, parameter-free) and periodic faces wrapping;
>
> **observed change**   `Δn_c^obs = n_c(k+1) − n_c(k)`;
>
> **the continuity defect**   `D_cont = RMS_{c,k}(Δn^obs − Δn^pred) / RMS_{c,k}(Δn^obs)`.

`D_cont = 0` means the momentum field accounts for every particle that changed cells;
`D_cont = 1` means it accounts for none. No threshold is invented: the verdict is the
freeze's G7 form applied to this leg — **the spatial chart's `D_cont` must beat the
position-blind chart's by `0.05` absolute** — and the absolute value is published beside it.
The blind control here is exactly what it was for the collision form: the same cells with
membership scrambled by the fixed per-atom permutation, so momentum and density no longer
belong to the same region.

## A fence the first plant found, before any reading was trusted

Plant PF-1 — exact advection, on which continuity holds by construction — read
`D_cont = 1.000` on a `2×2×2` grid, the same number every real file gave. The cause is exact:
**on an axis of two periodic cells the `+` and `−` faces border the same neighbour, and a
central face flux cancels identically.** The leg carries no information there. It needs at
least **three cells on every split axis**, and the instrument REFUSES a grid without them
rather than printing a number. Consequence, stated plainly: G2 admissibility (`≥ 100` per
cell) and this leg cannot both hold below `3×3×3 × 100 ≈ 2,700` waters. On the 432-water
scout the leg reads on `4×4×4` (`6.75` per cell — a fluctuating chart, but a testable
flux), and the collision form reads on `2×2×1`; the two legs sit on different grids until
the box is an order of magnitude larger, and the record says so.

## A second thing the plants taught: what the leg's number means on a liquid

PF-1's first carrier was a divergence-free flow at uniform density — on which continuity
predicts ZERO change in every cell, the observed change is discreteness noise from particles
near faces, and `D_cont = 1` is the correct reading. The positive plant is now a COMPRESSIBLE
flow, where the flux has a signal. The lesson for the liquid: the momentum a cell's chart can
use to predict its density change is the coherent, hydrodynamic part of the cell's momentum;
the thermal part sums to noise. An equilibrium liquid whose chart does not resolve that part
reads `D_cont ≈ 1` — not because the leg is broken but because, at that cell size and cadence,
there is no fluid element to see. That is the same question the collision form asks, from
the named law's side, and it is why the two are reported together.

## The third thing the plants taught, and it bounds the leg's power on every liquid

On any particle carrier the crossings through a face in a window are a COUNT, so `D_cont` has
a shot-noise floor of about `1/√(crossings per face per window)` — and refining the grid
RAISES it (16,000 particles on a smooth standing wave read `0.42 → 0.46 → 0.69` with the
grid). The positive plant needs a million particles to put the floor under the discretisation
error, and there the closure converges (`0.38 → 0.15` from four to eight cells per wavelength).

For a liquid this is the leg's power limit. At 432 waters on `4×4×4` a face sees a few
crossings per window, and the floor is near `0.7`; a reading of `1.0` there is a reading
**at the floor**, and says the chart's momentum field carries no more of the flux than
noise does — which is the honest answer at that size, not an instrument failure. To grade a
fluid element by continuity to `10 %` needs of order a hundred crossings per face per
window: thousands of molecules per cell, tens of thousands in the box. **The named-law leg
does not escape the size fence; it prices it differently.**

## What it can and cannot say

- It **can** say whether the chart's momentum field predicts its density field's motion —
  the first half of "this cell is a fluid element" — on the trajectory lengths already
  banked (the 128-water momentum run holds 6 windows at `2×1×1` and 24 at half that
  cadence; the 432 pilot holds 6).
- It **cannot** say the chart is closed in the freeze's sense: continuity is one equation of
  the tier's law and the collision form remains the certificate. A chart can pass this leg
  and fail the collision form (momentum predicts density but nothing predicts momentum).
- It **cannot** test the momentum equation, which needs a stress field the chart does not
  carry (PREREG §2.2 refuses the potential as a field). That leg would need the per-cell
  virial banked, and is named here as the next one, not built.

## Plants

| plant | carrier | must |
|---|---|---|
| **PF-1** | particles advected by a smooth, divergence-free velocity field on a periodic cube, windows shorter than the field's period | `D_cont` under `0.1` — continuity holds exactly on this carrier and the finite-volume closure resolves it |
| **PF-2** | independent random walkers with thermal velocities uncorrelated to their displacement | `D_cont` near `1` and no separation from the blind chart — the leg refuses a carrier whose momentum field carries no transport |
| **PF-3** | PF-1's carrier with membership scrambled (`BlindLabel`) | `D_cont` near `1` — the control loses on a carrier the real chart passes |
| **PF-4** | a `dims = 2` carrier | the third momentum component is absent and the face sum runs over four faces, not six |

## Its first reading

The banked 128-water momentum run (three seeds, both arms, 10 fs readouts) and the 432
pilot, at zero compute. Expected and stated now: the pilot's six windows are too few for a
graded reading and the number will be reported as VOID by count with the absolute `D_cont`
beside it; the 128-water run at `2×1×1` has enough windows at half the cadence to grade
G7-form separation, and whether it separates is the reading and is not predicted.

---
*Audit footer, added 2026-09-21 for `Audit/prereg_audit.py` after CI read red since 2026-09-19; no stake, gate, plant or number above moved.*
witness: none (a measured campaign: its gates are numeric and its closure algebra is `Closed` and `StatClosure.lean`, cited in words above; no gate is a Lean theorem of its own)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR — the registered ids this text contacts by keyword, cited at the audit's demand.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier in its own row, and the sector the plant acts on is nonzero in that carrier by construction of the plant.
