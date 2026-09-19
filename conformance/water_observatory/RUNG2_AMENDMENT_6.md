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
