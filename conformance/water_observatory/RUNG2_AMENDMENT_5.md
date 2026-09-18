# RUNG 2 — AMENDMENT 5: a fluid element is read at its own relaxation time, and its fields are averages over that window

*Written 2026-09-18, committed alone, BEFORE the instrument is changed and BEFORE the scout it
sizes is launched. Amends `RUNG2_PREREG.md` §2.2–2.3 in one place — WHEN a field is read and
over WHAT interval — and leaves the chart's fields, legs, gates, controls, budget and branches
unchanged. Written because the 2026-09-17 reading (`replace0/rung2_mom/`) found the half-box
momentum decorrelating within a 10 fs readout, and that is not a property of the liquid; it is
the readout asking a coarse law to predict thermal jitter.*

## The fault, from the record

At 128 waters and a 10 fs readout, density + momentum at `2×1×1` read `D_A ≈ 0.94`: two frames
with the same reading go to different readings `94 %` of the time. The cell's momentum is a
sum over `64` oxygens whose velocities decorrelate on the collision time (`~50` fs); sampled
every `10` fs it is, to the chart, a fresh thermal draw. A fluid element's momentum is not
that. It is the SLOW part — the hydrodynamic mode of period `~a/c_s` — with the thermal part
averaged out over the element's own relaxation. The freeze read an instantaneous field at an
arbitrary cadence. Both are scales fixed outside the physics of the cell, and this is the
fourth such scale found in this instrument and the first in TIME.

## The rule

> **A5.1 — the cadence.** A chart of cell edge `a` is read every **`τ = a / c_s`**, the
> cell's sound-crossing time, with `c_s = 1497 m s⁻¹` (water, 25 °C, CRC) an external
> protocol constant entering exactly as `T_target` and `S(0)` do.
>
> **A5.2 — the window.** Every field of the chart is the TIME AVERAGE of the instantaneous
> field over the `τ` window ending at the readout: occupancy, momentum and kinetic energy per
> cell, each averaged over the banked frames in the window. Windows do not overlap.
>
> **A5.3 — the bins.** The averaged fields are binned at their own held-out fluctuation:
> `Δ_field = σ(field_avg)` measured on the flexible arms of the seeds NOT being graded, at the
> same grid and window — Amendment 4's route (ii), now for all three fields. Route (i) has no
> external form for an averaged field's spread (it would need the velocity autocorrelation
> time) and is not extended.
>
> **A5.4 — three momentum components on a 3D grid.** The freeze binned two because its scene
> was two-dimensional. On `n_z > 1` the third is read; on `n_z = 1` the chart is the freeze's.

Nothing lowers a bar. The collision form, both legs, G2–G9, `β`, the controls and every
branch stand. The instantaneous chart at the fine cadence is still printed as the control.

## What it costs, stated before the run because it changes the run

The cadence sets the trajectory length: G4 needs `≥ 200` informative transitions and windows
do not overlap, so a carrier needs of order `250 τ` of production.

| carrier | coarsest admissible grid | `a` | `τ` | `250 τ` | rigid operator, per seed |
|---|---|---|---|---|---|
| 128 (banked) | none admissible; `2×1×1` | 14.8 bohr | 523 fs | — | the 3 ps run holds **6 windows** |
| **432** | `2×2×1`, 108 per cell | 22.2 bohr | 785 fs | **196 ps** | **~180 core-h** + settling |
| 1024 | `2×2×2`, 128 per cell | 29.6 bohr | 1,046 fs | 262 ps | ~560 core-h + settling |

The lead's earlier "800 waters, ~25 core-hours" was a 3 ps run — the transport run's length
carried to a bigger box — and under this amendment it would have bought six windows and the
same verdict at a larger size. **The scout is 432 waters, on the rigid operator, sized to
`~200 ps` per seed**; the runner's lattice is `2n³` so 432 is the first admissible box, and
1024 the first admissible cube, which this scout does not attempt.

## What the scout can and cannot say, staked

- It can say whether **G7 separates at four admissible cells** with fields read at the cell's
  own time. If the scrambled chart is still as closed as the spatial one there, branch (e)
  holds at the first admissible size and the tier's fence carries that number.
- It can say **which way `D_A` moves** from `2×2×1` at 432 against `2×1×1` at 128, both read
  under A5 — the first two points of the exit `N*` that `RUNG2_RESULTS.md` §6 left
  UNDETERMINED.
- It is read on the **rigid operator**, whose density and momentum charts matched the fine
  model's within seed spread at every grid on 128 waters. A verdict on it is a verdict on the
  operator's liquid; the amendment's own fence is that one fine-model seed at 432 is owed
  before any closure is called the model's.
- It cannot certify a fluid element: that needs G7 separated AND a dynamic chart inside `β`,
  and no chart has landed inside `β` on any carrier. If one does here, it is the first, and
  the record will treat it as a promotion candidate needing the fine-model seed above.

## Plants

| plant | must |
|---|---|
| **PE-1** | at window `1` the averaged chart equals the instantaneous chart bit for bit, every rung, grid and kind — the freeze is the `τ = one frame` case |
| **PE-2** | P-2's closed-by-construction chart, window-averaged over any `w` that divides its period, still `CertifiedStrict` — averaging manufactures no defect |
| **PE-3** | a hidden variable that flips faster than the window is BLURRED by the averaged chart (its spread falls by more than half) and STILL FIRES — averaging removes variance, not unpredictability; one that flips slower still fires. *(First written as "its firings fall"; the plant read 0.91 against 0.86 and corrected the sentence, not the instrument.)* |
| **PE-4** | on a zero-sum thermal carrier the averaged momentum's spread falls as `1/√w` — the averaging is doing what A5.2 says |
| **PE-5** | `refines(instantaneous, averaged)` fails and is NOT asserted: an average is not a coarsening of a frame, and the ladder self-check applies within a cadence, never across two |
