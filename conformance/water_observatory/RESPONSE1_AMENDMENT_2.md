# RESPONSE-1 — AMENDMENT 2: R1's grid is refused by the instrument, R1's floor does not fall with pooled windows, and the continuity read was never built

*Written 2026-09-19 afternoon, while the eight arms run and BEFORE any is read, on the
owner's question "anything else we can challenge while we wait?" — the zeros derived
before the read, per `FAST_AND_SLOW.md`. Nothing the arms record changes. Committed alone
before the reader is changed.*

## What was found, in order of weight

**1. R1's grid, `2×2×1`, is REFUSED by the instrument as built.** Amendment 6's plant PF-1
found that on an axis of exactly two periodic cells the opposite faces border the same
neighbour and the central flux cancels identically, and `continuity_admits` refuses such a
grid by name. The prereg (written the next morning) staked `2×2×1` anyway. Worse, the
doubling ladder on 432 waters offers NO admitted grid with `≥ 100` molecules per cell: its
rungs are `2×1×1, 2×2×1, 2×2×2, 4×2×2, 4×4×2` (all refused) and then `4×4×4` at `6.75` per
cell — where the pilot read `D_cont ≈ 1.0` on every seed, at the shot-noise floor. **R1 as
staked would have been REFUSED, not read.**

**2. `D_cont` does not fall with the number of windows pooled.** `FLUID_ELEMENT_RESPONSE.md`
§5 priced the driven floor as `D ≈ 1/(s√W)`. It is not: `D_cont` is the RMS ratio
`√(Σ(obs − pred)² / Σ obs²)` over windows and cells, and with signal `S` and noise `σ` per
window-cell in both the observed and the predicted side,

> `D_cont² ≈ (σ_obs² + σ_pred²) / (S² + σ_obs²)` — **independent of `W`**; pooling more raw
> windows sharpens `D`, it does not lower it. `D ≤ 0.2` needs `S/σ ≳ 5` in every window.

At 50 m/s the coherent crossings per full face per window are `7.2` (the response note's
`3.6` per half face) against a diffusive-crossing noise of about `6` per cell, `s ≈ 1.2`:
**`D_cont ≈ 0.64` on raw windows, and the stake `≤ 0.2` fails by arithmetic at 50 m/s
while separating from the blind (`≈ 1.0`) — branch (b), "open under drive", would have been
entered on a floor.** The tenth instance of a price taken from the wrong regime, the lead's,
the same afternoon the ninth was banked.

**3. The continuity read for R1 and R1′ was never built.** `response1_read`'s doc comment
promised it; the function read modes only. The stake had an instrument on paper.

## A1 — the driven grid

> R1 and R1′ are read on **`4×1×1`**: four cells along the drive axis, `108` molecules each,
> every face a full cross-section of the box. Three or more cells on the split axis (PF-1
> admitted), four cells and `≥ 100` per cell (G2 met), and no split on `y` or `z` — the
> standing wave is along `x`, so faces normal to `y` or `z` carry no coherent flux and would
> add shot noise to the ratio and nothing to the signal. The cadence is Amendment 5's,
> `τ = a/c_s` with `a = L/4 = 11.1` bohr: `392` fs, `39` readouts of 10 fs; a cycle holds
> eight windows.

## A2 — the cycle-aligned read, and the floor stated with it

> `D_cont` is read on the **sign-aligned average of the fields over a seed's twelve cycles**
> — each cell's occupancy departure from its time mean and its momentum, multiplied by the
> cycle's kick sign and averaged (continuity is linear, so the averaged fields obey it if
> each cycle's do) — then window-averaged at `τ` and fed to the continuity leg unchanged;
> the position-blind partition the same way. The pooled read over three seeds is the same
> average over thirty-six cycles.
>
> **The floor is stated beside every `D`:** `D_floor = 1/√(1 + s²)`, with `s` the aligned
> average's own signal-to-noise per window-cell — the RMS of the observed occupancy change
> over the cycle's first two windows against its RMS over the last two (relaxed). Derived
> in advance: `s ≈ 1.2` per cycle at 50 m/s, so `≈ 4.2` per seed (`D_floor ≈ 0.23`) and
> `≈ 7.2` over three seeds (`D_floor ≈ 0.14`); at 200 m/s `≈ 17` per seed (`≈ 0.06`).
>
> **The stake `D_cont(spatial) ≤ 0.2` is read on the three-seed aligned average at 50 m/s
> and on the per-seed average of the 200 m/s control.** A per-seed 50 m/s read above `0.2`
> that sits within `0.05` of its stated floor is AT FLOOR, not a kill; a read above `0.2`
> AND above its floor by more than `0.05` is the kill as staked. The separation stake
> (`blind − spatial ≥ 0.05`) is unchanged and is read on every average.

## A3 — the nulls, unchanged in substance

The in-run null (the last window of the aligned cycle, relaxed, `D_cont ≥ 0.8`), R1′ (arm T's
driven `D_cont` within `0.1` of its relaxed) and the blind partition are read on the same
grid and the same aligned averages.

## A4 — the midpoint law's own floor, and the grid it forces (found on building, same day)

The continuity leg predicts a face's flux as the mean of its two cells' momenta, and a
cell's momentum is the cell average of the field. On a single mode at `k = 2π/L` read on
`n` cells along the wave those two steps scale the prediction by `sinc(π/n) · cos(π/n)`, so
an EXACT fluid reads

> **`D_disc(n) = 1 − sinc(π/n) cos(π/n)` = `0.363` at 4 cells, `0.100` at 8, `0.026` at 16**

(plant PR-12, the exact continuum wave at zero noise, to 5 %). **On `4×1×1` the stake
`D ≤ 0.2` is under the leg's own floor for a perfect fluid.** R1 is therefore read on
**`8×1×1`** (54 molecules a cell — the leg's admission is three cells on the split axis and
twenty transitions, not G2's hundred, which gates the occupancy chart's binning), where the
spatial floor is `0.100`; `4×1×1` is read beside it with its floor printed. The faces are the
same full cross-sections, so the coherent crossings per face are unchanged; the cadence
halves (`τ = 196` fs, `20` readouts), so the per-cycle signal-to-noise falls to `≈ 0.85`, and
the floor over three seeds is `√((D_disc² s² + 1)/(s² + 1))` at `s ≈ 5`: **`≈ 0.22` at
50 m/s** — the stake as written sits AT its floor on the 50 m/s arms; the 200 m/s control
(`s ≈ 12`, floor `≈ 0.13`) is where it can be met with margin.

## A5 — the window-mean form of the leg cannot read a driven transient (found on building)

Amendment 6's leg compares consecutive WINDOW MEANS of the occupancy against the midpoint
of the two windows' mean momenta. That assumes the fields vary slowly over a window. The
driven density on the L arm rises in `1/λ₂ = 1/(ν_l k²) ≈ 40` fs — ten times faster than
the cell's cadence — because at `L = 2.35` nm the longitudinal mode is overdamped
(`Γ/ω ≈ 3.4`): the box is in the regime where the viscous time is SHORTER than the sound
time, the opposite of hydrodynamics. On the exact overdamped response the window-mean leg
reads **`D = 1.30` on 4 cells and `1.66` on 8 at zero noise** (PR-12) — an exact fluid
convicted by the leg's time-differencing. **The driven read uses the INTEGRAL form**: the
occupancy difference between two readouts a window apart against the face flux integrated
over every readout between them (trapezoid; the same spatial interpolation). It has no
temporal error beyond the readout spacing and reads the spatial floor on the same
transient (`0.360`, `0.095`). The read is over the **first two windows** of the aligned cycle
(the prereg's choice, now with its reason: pooling relaxed, noise-only windows into an RMS
ratio can only raise it — PR-9 shows twelve cycles' lead windows pooled read `0.64`
against one cycle's `0.57`, and the twelve aligned `0.27` at `s = 4.0` against a floor of
`0.26`). The noise for `s` is read from the **position-blind partition** over the same
windows (the same molecules, scrambled labels, no coherent part), because the eight-window
cycles do not relax (`16 %` of the peak remains at the cycle's end) and a tail-based noise
would be inflated by signal.

## Plants added

| plant | must |
|---|---|
| **PR-9** | synthetic occupancy and momentum fields on `4×1×1` with a coherent standing-wave flux at signal-to-noise `s = 1.2` per window-cell, Poisson noise, twelve cycles of alternating sign: `D_cont` on the raw pooled windows within `0.1` of `1/√(1 + s²)` (it does NOT fall with `W`); on the aligned average within `0.1` of `1/√(1 + 12 s²)` |
| **PR-10** | the same carrier with the flux zeroed (no drive): aligned `D_cont ≥ 0.8` — the alignment manufactures no closure |
| **PR-11** | `2×2×1` requested for R1: REFUSED by `continuity_admits`, by name |
| **PR-12** | the exact continuum standing wave, cell-averaged at zero noise: the window-mean leg reads `1 − sinc(π/n) cos(π/n)` to 5 % on 4 and 8 cells under a slow decay, reads `> 1` on the overdamped transient at the cell cadence, and the integral form reads the spatial floor on that transient |

## The correction to FLUID_ELEMENT_RESPONSE.md

Its §5 row "priced defect, derived shape: `≈ 1/(s√W)` under a drive" is wrong and is
corrected in place to `1/√(1 + s²)` on pooled windows, `1/√(1 + C s²)` on a `C`-cycle
aligned average, with a pointer here. Its 24-core-hour price stands: the signal is there,
but it is reached by aligning cycles, not by pooling windows.
