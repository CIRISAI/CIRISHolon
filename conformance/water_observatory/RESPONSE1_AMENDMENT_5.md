# RESPONSE-1 — AMENDMENT 5: the residual is the molecular layering, and the closed chart puts the momentum on the faces

*Written 2026-09-21 from the face-centred and lagged closure test on the two full longitudinal
arms (`r1_closure_test.py`; outputs in each arm's `r1_closure_test.txt`), BEFORE the remaining
eight arms are read. R1 as staked — the `(n̄, p̄)` cell chart — KILLS on both seeds and that
verdict stands; this amendment names what the residual is and stakes the corrected chart on
the arms still to land.*

## What the test did

Each cell's sign-aligned occupancy change over the lead windows was predicted three ways: (A)
the slab-mean momentum at the faces (the chart as staked); (B) the momentum of the molecules
within `±h` of each face, over `2h`; (C) the slab-mean momentum lagged by 30–100 fs. Amplitude
fidelity `α = ⟨obs·pred⟩/⟨pred²⟩`, `R²`, and `D` on the lead windows and on the relaxed tail.

| arm, grid | A slab-mean: α, R², D | C lagged (best) | **B face-centred, h = 0.25 Å: α, R², D** |
|---|---|---|---|
| 200 m/s, 8 cells | 0.89, 0.47, 0.735 | 0.83, 0.44, 0.758 | **1.05, 0.97, 0.164** |
| 200 m/s, 16 cells | 0.89, 0.20, 0.896 | worse | **1.01, 0.79, 0.460** |
| 200 m/s, 4 cells | 1.25, 0.28, 0.857 | worse | 1.05, 0.82, 0.429 |
| 50 m/s, 8 cells | 2.06, 0.39, 0.846 | worse | 1.24, 0.79, 0.492 |
| 50 m/s, 16 cells | 0.72, 0.04, 0.984 | worse | 0.99, 0.41, 0.767 |

The fidelity degrades monotonically as `h` grows toward the slab (`h = 2 Å` is the slab-mean
again). Lagging the slab momentum never helps. **The residual is spatial at the molecular
scale, not a memory**: the molecules that cross a face in a window are those within about
half a rattle amplitude of it, and their coherent motion is not the slab average when the
slab is one molecular diameter. The commit message of `1b892e1` ("not sub-cell spatial
structure") inferred from the sinusoid's floor and was wrong: the structure is the
layering, which refinement exposes rather than removes. The nineteenth instance.

## What it means

- The `(n̄, p̄)` chart at one molecular diameter is not closed within `β = 0.2`; its price is
  the transfer function `α ≈ 0.45` at 8 cells, `≈ 0.2` at 16, `≈ 1` at 4 — the momentum
  density's molecular layering, generalized hydrodynamics seen from the flux side.
- **The closed chart is the staggered one**: density in cells, momentum on the faces within
  `h = 0.25 Å`. On the 200 m/s control at 8 cells it reads `D = 0.164`, inside the budget,
  with `R² = 0.97` and `α = 1.05`. That is finite-volume hydrodynamics' own choice, and the
  reason for it is now measured on this model.
- The face momentum also predicts part of the THERMAL crossings (tail `D` `0.5–0.6` against
  `1.0` for the slab): the staggered chart carries more of the Record than the cell chart.

## Staked on the eight arms still to land (R1″, beside R1 as written)

> **R1″:** the face-centred chart (`h = 0.25 Å`, 8 cells along the wave, the integral form)
> reads `D ≤ 0.2` on the three-seed aligned average of the 200 m/s longitudinal arms, with
> `α` within `[0.8, 1.25]`; on the 50 m/s arms it is reported at its floor. **Kill:** `D > 0.3`
> on the 200 m/s average, or `α` outside the band. **Null:** the same chart on the transverse
> arms' driven windows within `0.1` of their relaxed windows (R1′ for the face chart).
> `h` is fixed at `0.25 Å` by this test and is not tuned on the remaining arms; `h = 0.5` is
> printed beside it as the sensitivity.
